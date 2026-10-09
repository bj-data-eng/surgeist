Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Values and Units Module Level 4](https://www.w3.org/TR/2021/WD-css-values-4-20210715/).

Original copyright notice: Copyright © 2021 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Values and Units Module Level 4

Source snapshot: https://www.w3.org/TR/2021/WD-css-values-4-20210715/

Snapshot SHA-256: 23e1c33c648cb460d972bd5a0e190d7ff5565c953fbd1f5fe81725aa21a31bda

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- 2 complex or multi-paragraph tables use source-checked readable field, case, grid or matrix layouts. Explicit header/span relationships and source cell mappings are retained; no raw HTML tables or flattened row/cell dumps remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Values and Units Module Level 4

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2021 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This CSS module describes the common values and units that CSS properties accept and the syntax used for describing them in CSS property definitions.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.&#xA;&#x9;Other documents may supersede this document.&#xA;&#x9;A list of current W3C publications&#xA;&#x9;and the latest revision of this technical report&#xA;&#x9;can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/Style/CSS/) as a <strong>Working Draft</strong>. Publication as a Working Draft does not imply endorsement by the W3C Membership.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-values” in the title, like this: “\[css-values\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-values%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [15 September 2020 W3C Process Document](https://www.w3.org/2020/Process-20200915/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-funcdef-attr"></a>

  <a id="ref-for-funcdef-toggle"></a>

  [toggle()](#funcdef-toggle), [attr()](#funcdef-attr)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

<a id="ref-for-length-value"></a>

The value definition field of each CSS property can contain keywords, data types (which appear between \< and \>), and information on how they can be combined. Generic data types ([\<length\>](#length-value) being the most widely used) that can be used by many properties are described in this specification, while more specific data types (e.g., \<spacing-limit\>) are described in the corresponding modules.

### <a id="placement"></a>1.1.  Module Interactions

This module replaces and extends the data type definitions in [\[CSS21\]](#biblio-css21) sections [1.4.2.1](https://www.w3.org/TR/CSS21/about.html#value-defs), [4.3](https://www.w3.org/TR/CSS21/syndata.html#values), and [A.2](https://www.w3.org/TR/CSS21/aural.html#aural-intro).

## <a id="value-defs"></a>2.  Value Definition Syntax

The <a id="css-value-definition-syntax"></a>value definition syntax described here is used to define the set of valid values for CSS properties (and the valid syntax of many other parts of CSS). A value so described can have one or more components.

### <a id="component-types"></a>2.1.  Component Value Types

Component value types are designated in several ways:

1.  <a id="ref-for-disc"></a>

    [keyword](#keywords) values (such as auto, [disc](https://www.w3.org/TR/css-counter-styles-3/#disc), etc.), which appear literally, without quotes (e.g. `auto`)

2.  <a id="ref-for-numeric-data-types"></a>

    <a id="ref-for-percentage-value"></a>

    <a id="ref-for-length-value①"></a>

    basic data types, which appear between \< and \> (e.g., [\<length\>](#length-value), [\<percentage\>](#percentage-value), etc.). For [numeric data types](#numeric-data-types), this type notation can annotate any range restrictions using the [bracketed range notation](#numeric-ranges) described below.

3.  <a id="ref-for-integer-value②"></a>

    <a id="ref-for-identifier-value②"></a>

    <a id="ref-for-integer-value①"></a>

    <a id="ref-for-identifier-value①"></a>

    <a id="ref-for-integer-value"></a>

    <a id="ref-for-identifier-value"></a>

    <a id="ref-for-mult-comma"></a>

    <a id="ref-for-valdef-all-inherit"></a>

    <a id="ref-for-propdef-background-attachment"></a>

    <a id="ref-for-propdef-border-width"></a>

    types that have the same range of values as a property bearing the same name (e.g., [\<'border-width'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width), [\<'background-attachment'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-attachment), etc.). In this case, the type name is the property name (complete with quotes) between the brackets. Such a type does <em>not</em> include [CSS-wide keywords](#common-keywords) such as [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit), and also does not include any top-level [comma-separated-list multiplier](#mult-comma) (i.e. if a property named pairing is defined as \[ [\<custom-ident\>](#identifier-value) [\<integer\>](#integer-value)? \]#, then \<'pairing'\> is equivalent to \[ [\<custom-ident\>](#identifier-value) [\<integer\>](#integer-value)? \], not \[ [\<custom-ident\>](#identifier-value) [\<integer\>](#integer-value)? \]#).

4.  <a id="ref-for-propdef-border-width①"></a>

    <a id="ref-for-value-def-border-width"></a>

    non-terminals that do not share the same name as a property. In this case, the non-terminal name appears between \< and \>, as in \<spacing-limit\>. Notice the distinction between [\<border-width\>](https://www.w3.org/TR/CSS2/box.html#value-def-border-width) and [\<'border-width'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width): the latter is defined as the value of the <a id="ref-for-propdef-border-width②"></a>border-width property, the former requires an explicit expansion elsewhere. The definition of a non-terminal is typically located near its first appearance in the specification.

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

All CSS properties also accept the [CSS-wide keyword values](#common-keywords) as the sole component of their property value. For readability these are not listed explicitly in the property value syntax definitions. For example, the full value definition of [border-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-color) is `<color>{1,4} | inherit | initial | unset` (even though it is listed as `<color>{1,4}`).

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
### <a id="component-multipliers"></a>2.3.  Component Value Multipliers

Every type, keyword, or bracketed group may be followed by one of the following modifiers:

- An asterisk (<a id="mult-zero-plus"></a>\*) indicates that the preceding type, word, or group occurs zero or more times.
- A plus (<a id="mult-one-plus"></a>+) indicates that the preceding type, word, or group occurs one or more times.
- A question mark (<a id="mult-opt"></a>?) indicates that the preceding type, word, or group is optional (occurs zero or one times).
- A single number in curly braces (<a id="mult-num"></a>{<var>A</var>}) indicates that the preceding type, word, or group occurs <var>A</var> times.
- A comma-separated pair of numbers in curly braces (<a id="mult-num-range"></a>{<var>A</var>,<var>B</var>}) indicates that the preceding type, word, or group occurs at least <var>A</var> and at most <var>B</var> times. The <var>B</var> may be omitted ({<var>A</var>,}) to indicate that there must be at least <var>A</var> repetitions, with no upper bound on the number of repetitions.
- A hash mark (<a id="mult-comma"></a>\#) indicates that the preceding type, word, or group occurs one or more times, separated by comma tokens (which may optionally be surrounded by [white space](https://www.w3.org/TR/css-syntax/#whitespace) and/or comments). It may optionally be followed by the curly brace forms, above, to indicate precisely how many times the repetition occurs, like \<length\>#{1,4}.
- An exclamation point (<a id="mult-req"></a>!) after a group indicates that the group is required and must produce at least one value; even if the grammar of the items within the group would otherwise allow the entire contents to be omitted, at least one component value must not be omitted.

For repeated component values (indicated by \*, +, or \#), UAs must support at least 20 repetitions of the component. If a property value contains more than the supported number of repetitions, the declaration must be ignored as if it were invalid.

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
> 
>
> | Property                                                                                              | Value definition field                                                                                         | Example value                                                                           |
> |-------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------|
> | <a id="ref-for-propdef-orphans"></a>[orphans](https://www.w3.org/TR/css-break-3/#propdef-orphans)                      | \<integer\>                                                                                                    | 3                                                                                       |
> | <a id="ref-for-propdef-text-align"></a>[text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align)                 | left \| right \| center \| justify                                                                             | <a id="ref-for-valdef-text-align-center"></a>[center](https://www.w3.org/TR/css-text-3/#valdef-text-align-center) |
> | <a id="ref-for-propdef-padding-top"></a>[padding-top](https://www.w3.org/TR/css-box-4/#propdef-padding-top)                | \<length\> \| \<percentage\>                                                                                   | 5%                                                                                      |
> | <a id="ref-for-propdef-outline-color"></a>[outline-color](https://www.w3.org/TR/css-ui-3/#propdef-outline-color)             | \<color\> \| invert                                                                                            | \#fefefe                                                                                |
> | <a id="ref-for-propdef-text-decoration"></a>[text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration) | none \| underline \|\| overline \|\| line-through \|\| blink                                                   | overline underline                                                                      |
> | <a id="ref-for-propdef-font-family"></a>[font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family)              | \[ \<family-name\> \| \<generic-family\> \]#                                                                   | "Gill Sans", Futura, sans-serif                                                         |
> | <a id="ref-for-propdef-border-width③"></a>[border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width)      | \[ \<length\> \| thick \| medium \| thin \]{1,4}                                                               | 2px medium 4px                                                                          |
> | <a id="ref-for-propdef-box-shadow"></a>[box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow)          | \[ inset? &#x26;&#x26; \<length\>{2,4} &#x26;&#x26; \<color\>? \]# \| none | 3px 3px rgba(50%, 50%, 50%, 50%), lemonchiffon 0 0 4px inset                            |
>
> 

## <a id="combining-values"></a>3.  Combining Values: Interpolation, Addition, and Accumulation

<a id="ref-for-computed-value"></a>

Some procedures, for example [transitions](https://www.w3.org/TR/css-transitions/) and [animations](https://www.w3.org/TR/css-animations/), <a id="combine"></a>combine two CSS property values. The following combining operations—on the two [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) <var>V<sub>a</sub></var> and <var>V<sub>B</sub></var> yielding the <a id="ref-for-computed-value①"></a>computed value <var>V<sub>result</sub></var>—are defined:

<a id="interpolation"></a>interpolation  
Given two property values <var>V<sub>a</sub></var> and <var>V<sub>B</sub></var>, produces an intermediate value <var>V<sub>result</sub></var> at a distance of <var>p</var> along the interval between <var>V<sub>a</sub></var> and <var>V<sub>B</sub></var> such that <var>p</var> = 0 produces <var>V<sub>a</sub></var> and <var>p</var> = 1 produces <var>V</var><sub>B</sub>.

<a id="ref-for-easing-function"></a>

The range of <var>p</var> is (−∞, ∞) due to the effect of [timing functions](https://www.w3.org/TR/css-easing-1/#easing-function). As a result, this procedure must also define extrapolation behavior for <var>p</var> outside \[0, 1\].

<a id="addition"></a>addition  
Given two property values <var>V<sub>a</sub></var> and <var>V<sub>B</sub></var>, returns the sum of the two properties, <var>V</var><sub>result</sub>. For addition that is not commutative (for example, matrix multiplication) <var>V<sub>a</sub></var> represents the first term of the operation and <var>V<sub>B</sub></var> represents the second.

<a id="ref-for-addition"></a>

<a id="ref-for-interpolation"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While [addition](#addition) can often be expressed in terms of the same weighted sum function used to define [interpolation](#interpolation), this is not always the case. For example, interpolation of transform matrices involves decomposing and interpolating the matrix components whilst addition relies on matrix multiplication.

<a id="accumulation"></a>accumulation  
Given two property values <var>V<sub>a</sub></var> and <var>V<sub>B</sub></var>, returns the result, <var>V<sub>result</sub></var>, of combining the two operands such that <var>V<sub>B</sub></var> is treated as a <em>delta</em> from <var>V<sub>a</sub></var>. For accumulation that is not commutative (for example, accumulation of mismatched transform lists) <var>V<sub>a</sub></var> represents the first term of the operation and <var>V<sub>B</sub></var> represents the second.

<a id="ref-for-accumulation"></a>

<a id="ref-for-addition①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For many types of animation such as numbers or lengths, [accumulation](#accumulation) is defined to be identical to [addition](#addition).
> <a id="ref-for-addition②"></a>
>
> <a id="ref-for-accumulation①"></a>
>
> A common case where the definitions differ is for list-based types where [addition](#addition) may be defined as appending to a list whilst [accumulation](#accumulation) may be defined as component-based addition. For example, the filter list values blur(2) and blur(3), when <a id="ref-for-addition③"></a>added together would produce blur(2) blur(3), but when <a id="ref-for-accumulation②"></a>accumulated would produce blur(5).

<a id="ref-for-computed-value②"></a>

<a id="ref-for-length-value②"></a>

<a id="ref-for-canonical-unit"></a>

These operations are only defined on [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value). (As a result, it is not necessary to define, for example, how to add a [\<length\>](#length-value) value of 15pt with 5em since such values will be resolved to their [canonical unit](#canonical-unit) before being passed to any of the above procedures.)

<a id="ref-for-addition④"></a>

If a value type does not define a specific procedure for [addition](#addition) or is defined as <a id="not-additive"></a>not additive, its <a id="ref-for-addition⑤"></a>addition operation is simply <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var>.

<a id="ref-for-accumulation③"></a>

<a id="ref-for-addition⑥"></a>

If a value type does not define a specific procedure for [accumulation](#accumulation), its <a id="ref-for-accumulation④"></a>accumulation operation is identical to [addition](#addition).

### <a id="combining-range"></a>3.1.  Range Checking

<a id="ref-for-easing-function①"></a>

<a id="ref-for-math-function"></a>

Interpolation can result in a value outside the valid range for a property, even if all of the inputs to interpolation are valid; this especially happens when <var>p</var> is outside the \[0, 1\] range, but some [easing functions](https://www.w3.org/TR/css-easing-1/#easing-function) can cause this to occur even within that range. If the final result <em>after</em> interpolation, addition, and accumulation is out-of-range for the target context the value is being used in, it does not cause the declaration to be invalid. Instead, the value must be clamped to the range allowed in the target context, exactly the same as [math functions](#math-function) (see [§ 11.12 Range Checking](#calc-range)).

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

CSS <a id="css-css-identifier"></a>identifiers, generically denoted by <a id="typedef-ident"></a>\<ident\>, consist of a sequence of characters conforming to the [\<ident-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-ident-token) grammar. [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3) Identifiers cannot be quoted; otherwise they would be interpreted as strings. CSS properties accept two classes of identifiers: [pre-defined keywords](#keywords) and [author-defined identifiers](#custom-idents).

<a id="ref-for-typedef-ident"></a>

<a id="ref-for-identifier-value③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<ident\>](#typedef-ident) production is not meant for property value definitions—[\<custom-ident\>](#identifier-value) should be used instead. It is provided as a convenience for defining other syntactic constructs.

<a id="ref-for-interpolation①"></a>

<a id="ref-for-discrete"></a>

<a id="ref-for-not-additive"></a>

All textual data types [interpolate](#interpolation) as [discrete](https://www.w3.org/TR/web-animations-1/#discrete) and are [not additive](#not-additive).

### <a id="keywords"></a>4.1.  Pre-defined Keywords

<a id="ref-for-ascii-case-insensitive"></a>

In the value definition fields, <a id="css-keyword"></a>keywords with a pre-defined meaning appear literally. Keywords are CSS identifiers and are interpreted [ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive) (i.e., \[a-z\] and \[A-Z\] are equivalent).

<a id="ref-for-propdef-border-collapse"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9d0ae504"></a> For example, here is the value definition for the [border-collapse](https://www.w3.org/TR/CSS2/tables.html#propdef-border-collapse) property:
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

As defined [above](#component-types), all properties accept the <a id="css-wide-keywords"></a>CSS-wide keywords, which represent value computations common to all CSS properties.

<a id="ref-for-valdef-all-initial①"></a>

<a id="ref-for-valdef-all-inherit②"></a>

<a id="ref-for-valdef-all-unset①"></a>

The [initial](https://www.w3.org/TR/css-cascade-5/#valdef-all-initial) keyword represents the value specified as the property’s initial value. The [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit) keyword represents the computed value of the property on the element’s parent. The [unset](https://www.w3.org/TR/css-cascade-5/#valdef-all-unset) keyword acts as either <a id="ref-for-valdef-all-inherit③"></a>inherit or <a id="ref-for-valdef-all-initial②"></a>initial, depending on whether the property is inherited or not. All of these keywords are normatively defined in the Cascade module. [\[CSS3CASCADE\]](#biblio-css3cascade)

Other CSS specifications can define additional CSS-wide keywords.

<a id="ref-for-identifier-value④"></a>

### <a id="custom-idents"></a>4.2.  Author-defined Identifiers: the [\<custom-ident\>](#identifier-value) type

<a id="ref-for-string-is"></a>

Some properties accept arbitrary author-defined identifiers as a component value. This generic data type is denoted by <a id="identifier-value"></a>\<custom-ident\>, and represents any valid CSS identifier that would not be misinterpreted as a pre-defined keyword in that property’s value definition. Such identifiers are fully case-sensitive (meaning they’re compared using the "[identical to](https://infra.spec.whatwg.org/#string-is)" operation), even in the ASCII range (e.g. example and EXAMPLE are two different, unrelated user-defined identifiers).

<a id="ref-for-css-wide-keywords"></a>

<a id="ref-for-identifier-value⑤"></a>

<a id="ref-for-valdef-cursor-default"></a>

<a id="ref-for-ascii-case-insensitive①"></a>

The [CSS-wide keywords](#css-wide-keywords) are not valid [\<custom-ident\>](#identifier-value)s. The [default](https://www.w3.org/TR/css-ui-3/#valdef-cursor-default) keyword is reserved and is also not a valid <a id="ref-for-identifier-value⑥"></a>\<custom-ident\>. Specifications using <a id="ref-for-identifier-value⑦"></a>\<custom-ident\> must specify clearly what other keywords are excluded from <a id="ref-for-identifier-value⑧"></a>\<custom-ident\>, if any—for example by saying that any pre-defined keywords in that property’s value definition are excluded. Excluded keywords are excluded in all [ASCII case permutations](https://infra.spec.whatwg.org/#ascii-case-insensitive).

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
> Note: When designing grammars with [\<custom-ident\>](#identifier-value), the <a id="ref-for-identifier-value①②"></a>\<custom-ident\> should always be "positionally unambiguous", so that it’s impossible to conflict with any keyword values in the property.

<a id="ref-for-typedef-dashed-ident"></a>

### <a id="dashed-idents"></a>4.3.  <em>Explicitly</em> Author-defined Identifiers: the [\<dashed-ident\>](#typedef-dashed-ident) type

Some contexts accept <em>both</em> author-defined identifiers <em>and</em> CSS-defined identifiers. If not handled carefully, this can result in difficulties adding new CSS-defined values; UAs have to study existing usage and gamble that there are sufficiently few author-defined identifiers in use matching the new CSS-defined one, so giving the new value a special CSS-defined meaning won’t break existing pages.

<a id="ref-for-typedef-dashed-ident①"></a>

While there are many legacy cases in CSS that mix these two values spaces in exactly this fraught way, the [\<dashed-ident\>](#typedef-dashed-ident) type is meant to be an easy way to distinguish author-defined identifiers from CSS-defined identifiers.

<a id="ref-for-typedef-dashed-ident②"></a>

<a id="ref-for-identifier-value①③"></a>

The <a id="typedef-dashed-ident"></a>[\<dashed-ident\>](#typedef-dashed-ident) production is a [\<custom-ident\>](#identifier-value), with all the case-sensitivity that implies, with the additional restriction that it must start with two dashes (U+002D HYPHEN-MINUS).

<a id="ref-for-typedef-dashed-ident③"></a>

[\<dashed-ident\>](#typedef-dashed-ident)s are reserved solely for use as author-defined names. CSS will never define a <a id="ref-for-typedef-dashed-ident④"></a>\<dashed-ident\> for its own use.

<a id="ref-for-custom-property"></a>

<a id="ref-for-typedef-dashed-ident⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a91b3e3f"></a> For example, [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) need to be distinguishable from CSS-defined properties, as new properties are added to CSS regularly. To allow this, <a id="ref-for-custom-property①"></a>custom property names are required to be [\<dashed-ident\>](#typedef-dashed-ident)s, as in this example:
>
> ```text
> .foo {
>   --fg-color: blue;
> }
> ```
<a id="ref-for-typedef-dashed-ident⑥"></a>

<a id="ref-for-at-ruledef-profile"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-340c7d34"></a> [\<dashed-ident\>](#typedef-dashed-ident)s are also used in the [@color-profile](https://www.w3.org/TR/css-color-4/#at-ruledef-profile) rule, to separate author-defined color profiles from pre-defined ones like device-cmyk, and allow CSS to define more pre-defined (but overridable) profiles in the future without fear of clashing with author-defined profiles:
>
> ```text
> @color-profile --foo { src: url(https://example.com/foo.icc); }
> .foo {
>   color: color(--foo 1 0 .5 / .2);
> }
> ```
<a id="ref-for-typedef-dashed-ident⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-32974307"></a> CSS will use [\<dashed-ident\>](#typedef-dashed-ident) more in the future, as more author-controlled syntax is added. CSS authoring tools, such as preprocessors that turn custom syntax into standard CSS, <em>should</em> use <a id="ref-for-typedef-dashed-ident⑧"></a>\<dashed-ident\> as well, to avoid clashing with future CSS design.
>
> For example, if a CSS preprocessor added a new "custom" at-rule, it <em>shouldn’t</em> spell it @custom, as this would clash with a future official @custom rule added by CSS. Instead, it should use @--custom, which is guaranteed to never clash with anything defined by CSS.
>
> Even better, it should use @--library1-custom, so that if Library2 adds their own "custom" at-rule (spelled @--library2-custom), there’s no possibility of clash. Ideally this prefix should be customizable, if allowed by the tooling, so authors can manually avoid clashes on their own.

<a id="ref-for-string-value①"></a>

### <a id="strings"></a>4.4.  Quoted Strings: the [\<string\>](#string-value) type

<a id="ref-for-string"></a>

<a id="ref-for-typedef-string-token"></a>

[Strings](https://infra.spec.whatwg.org/#string) are denoted by <a id="string-value"></a>\<string\> and consist of a sequence of characters delimited by double quotes or single quotes. They correspond to the [\<string-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-string-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9204e1e4"></a> Double quotes cannot occur inside double quotes, unless [escaped](https://www.w3.org/TR/CSS21/syndata.html#escaped-characters) (as `"\""` or as `"\22"`). Analogously for single quotes (`'\''` or `'\27'`).
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

The [\<url\>](#url-value) type represents a [URL](https://url.spec.whatwg.org/#concept-url), which is a pointer to a resource.

<a id="ref-for-url-value③"></a>

<a id="ref-for-functional-notation"></a>

Typically, a [\<url\>](#url-value) is written with the <a id="funcdef-url"></a>url() or <a id="funcdef-src"></a>src() [functional notations](#functional-notation):

<a id="url-value"></a>

<a id="ref-for-string-value②"></a>

<a id="ref-for-typedef-url-modifier"></a>

<a id="ref-for-mult-zero-plus"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-string-value③"></a>

<a id="ref-for-typedef-url-modifier①"></a>

<a id="ref-for-mult-zero-plus①"></a>

```text
<url> = url( <string> <url-modifier>* ) |
        src( <string> <url-modifier>* )
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

<a id="ref-for-funcdef-src"></a>

<a id="ref-for-funcdef-var"></a>

For legacy reasons, a [url()](#funcdef-url) can be written without quotation marks around the URL itself, in which case it is [specially-parsed](https://www.w3.org/TR/css-syntax-3/#consume-a-url-token) as a [\<url-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-url-token) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). Because of this special parsing, <a id="ref-for-funcdef-url①"></a>url() is only able to specify its URL literally; [src()](#funcdef-src) lacks this special parsing rule, and so its URL can be provided by functions, such as [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var).

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

<a id="ref-for-funcdef-url②"></a>

<a id="ref-for-typedef-url-modifier②"></a>

<a id="ref-for-string-value④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The unquoted [url()](#funcdef-url) syntax cannot accept a [\<url-modifier\>](#typedef-url-modifier) argument and has extra escaping requirements: parentheses, [whitespace](https://www.w3.org/TR/css-syntax/#whitespace) characters, single quotes (') and double quotes (") appearing in a URL must be escaped with a backslash, e.g. url(open&#x5C;(parens), url(close&#x5C;)parens). (In quoted [\<string\>](#string-value) <a id="ref-for-funcdef-url③"></a>url()s, only newlines and the character used to quote the string need to be escaped.) Depending on the type of URL, it might also be possible to write these characters as URL-escapes (e.g. url(open%28parens) or url(close%29parens)) as described in [\[URL\]](#biblio-url).
>
> <a id="ref-for-funcdef-url④"></a>
>
> The precise requirements for parsing the unquoted [url()](#funcdef-url) syntax are normatively defined in [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3).

<a id="ref-for-at-ruledef-import"></a>

<a id="ref-for-url-value④"></a>

<a id="ref-for-string-value⑤"></a>

<a id="ref-for-funcdef-url⑤"></a>

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

When a [\<url\>](#url-value) appears in the computed value of a property, it is resolved to an absolute URL, as described in the preceding paragraph. The computed value of a URL that the UA cannot resolve to an absolute URL is the specified value.

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

To work around some common eccentricities in browser URL handling, CSS has special behavior for fragment-only urls.

<a id="ref-for-funcdef-url⑥"></a>

If a [url()](#funcdef-url)’s value starts with a U+0023 NUMBER SIGN (`#`) character, parse it as per normal for URLs, but additionally set the <a id="url-local-url-flag"></a>local url flag of the <a id="ref-for-funcdef-url⑦"></a>url().

<a id="ref-for-funcdef-url⑧"></a>

<a id="ref-for-url-local-url-flag"></a>

When matching a [url()](#funcdef-url) with the [local url flag](#url-local-url-flag) set, ignore everything but the URL’s fragment, and resolve that fragment against the current document that relative URLs are resolved against. This reference must always be treated as same-document (rather than cross-document).

<a id="ref-for-funcdef-url⑨"></a>

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
> Theoretically, browsers should re-resolve any relative URLs, including fragment-only URLs, whenever the document’s base URL changes (such as through mutation of the <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-base-element">base</a></code> element, or calling <code><a href="https://html.spec.whatwg.org/multipage/history.html#dom-history-pushstate">pushState()</a></code>). In many cases they don’t, however, and so without special handling, fragment-only URLs will suddenly become cross-document references (pointing at the previous base URL) and break in many of the places they’re used.
>
> Since fragment-only URLs express a clear semantic of wanting to refer to the current document regardless of what its current URL is, this hack preserves the expected behavior at least in these cases.

#### <a id="url-empty"></a>4.5.2.  Empty URLs

<a id="ref-for-funcdef-url①⓪"></a>

If the value of the [url()](#funcdef-url) is the empty string (like url("") or <a id="ref-for-funcdef-url①①"></a>url()), the url must resolve to an invalid resource (similar to what the url about:invalid does).

<a id="ref-for-funcdef-url①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This matches the behavior of empty urls for embedded resources elsewhere in the web platform, and avoids excess traffic re-requesting the stylesheet or host document due to editing mistakes leaving the [url()](#funcdef-url) value empty, which are almost certain to be invalid resources for whatever the <a id="ref-for-funcdef-url①③"></a>url() shows up in. Linking on the web platform <em>does</em> allow empty urls, so if/when CSS gains some functionality to control hyperlinks, this restriction can be relaxed in those contexts.

#### <a id="url-modifiers"></a>4.5.3.  URL Modifiers

<a id="ref-for-funcdef-url①④"></a>

<a id="ref-for-typedef-url-modifier③"></a>

<a id="ref-for-typedef-ident①"></a>

<a id="ref-for-functional-notation①"></a>

The [url()](#funcdef-url) function supports specifying additional <a id="typedef-url-modifier"></a>\<url-modifier\>s, which change the meaning or the interpretation of the URL somehow. A [\<url-modifier\>](#typedef-url-modifier) is either an [\<ident\>](#typedef-ident) or a [functional notation](#functional-notation).

<a id="ref-for-typedef-url-modifier④"></a>

This specification does not define any [\<url-modifier\>](#typedef-url-modifier)s, but other specs may do so.

<a id="ref-for-url-value⑥"></a>

<a id="ref-for-funcdef-url①⑤"></a>

<a id="ref-for-typedef-url-modifier⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A [\<url\>](#url-value) that is either unquoted or not wrapped in [url()](#funcdef-url) notation cannot accept any [\<url-modifier\>](#typedef-url-modifier)s.

## <a id="numeric-types"></a>5.  Numeric Data Types

<a id="ref-for-specified-value①"></a>

<a id="ref-for-computed-value④"></a>

Numeric data types are used to represent quantities, indexes, positions, and other such values. Although many syntactic variations can exist in expressing the quantity (numeric aspect) in a given numeric value, the [specified](https://www.w3.org/TR/css-cascade-5/#specified-value) and [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) do not distinguish these variations: they represent the value’s abstract quantity, not its syntactic representation.

<a id="ref-for-integer-value③"></a>

<a id="ref-for-number-value"></a>

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-dimension"></a>

<a id="ref-for-length-value③"></a>

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

The precision and supported range of numeric values in CSS is <em>explicitly undefined</em>, and can vary based on the property or other context a value is used in. However, within the CSS specifications, infinite precision and range is assumed. When a value cannot be explicitly supported due to range/precision limitations, it must be converted to the closest value supported by the implementation, but how the implementation defines "closest" is explicitly undefined as well.

<a id="ref-for-angle-value①"></a>

If an [\<angle\>](#angle-value) must be converted due to exceeding the implementation-defined range of supported values, it must be clamped to the nearest supported multiple of 360deg.

### <a id="numeric-ranges"></a>5.1.  Range Restrictions and Range Definition Notation

<a id="ref-for-integer-value④"></a>

Properties can restrict numeric values to some range. If the value is outside the allowed range, then unless otherwise specified, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore). Range restrictions can be annotated in the numeric type notation using <a id="css-bracketed-range-notation"></a>CSS bracketed range notation—<code><c->&#x5B;</c-><var>min</var><c->,</c-><var>max</var><c->&#x5D;</c-></code>—within the angle brackets, after the identifying keyword, indicating a closed range between (and including) <var>min</var> and <var>max</var>. For example, [\<integer \[0,10\]\>](#integer-value) indicates an integer between 0 and 10, inclusive.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: CSS values generally do not allow open ranges; thus only square-bracket notation is used.

<a id="ref-for-length-value④"></a>

CSS theoretically supports infinite precision and infinite ranges for all value types; however in reality implementations have finite capacity. UAs should support reasonably useful ranges and precisions. Range extremes that are ideally unlimited are indicated using ∞ or −∞ as appropriate. For example, [\<length \[0,∞\]\>](#length-value) indicates a non-negative length.

<a id="ref-for-css-bracketed-range-notation"></a>

If no range is indicated, either by using the [bracketed range notation](#css-bracketed-range-notation) or in the property description, then `[−∞,∞]` is assumed.

<a id="ref-for-css-bracketed-range-notation①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: At the time of writing, the [bracketed range notation](#css-bracketed-range-notation) is new; thus in most CSS specifications any range limitations are described only in prose. (For example, “Negative values are not allowed” or “Negative values are invalid” indicate a `[0,∞]` range.) This does not make them any less binding.

<a id="ref-for-integer-value⑤"></a>

### <a id="integers"></a>5.2.  Integers: the [\<integer\>](#integer-value) type

Integer values are denoted by <a id="integer-value"></a>\<integer\>.

<a id="ref-for-typedef-number-token"></a>

When written literally, an <a id="integer"></a>integer is one or more decimal digits 0 through 9 and corresponds to a subset of the [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token) production in the CSS Syntax Module [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). The first digit of an integer may be immediately preceded by - or + to indicate the integer’s sign.

<a id="ref-for-integer-value⑥"></a>

#### <a id="combine-integers"></a>5.2.1.  Combination of [\<integer\>](#integer-value)

<a id="ref-for-interpolation②"></a>

<a id="ref-for-integer-value⑦"></a>

<a id="ref-for-number-value①"></a>

[Interpolation](#interpolation) of [\<integer\>](#integer-value) is defined as <var>V</var><sub>result</sub> = round((1 - <var>p</var>) × <var>V<sub>a</sub></var> + <var>p</var> × <var>V<sub>b</sub></var>); that is, interpolation happens in the real number space as for [\<number\>](#number-value)s, and the result is converted to an <a id="ref-for-integer-value⑧"></a>\<integer\> by rounding to the nearest integer, with values halfway between adjacent integers rounded towards positive infinity.

<a id="ref-for-addition⑦"></a>

<a id="ref-for-integer-value⑨"></a>

[Addition](#addition) of [\<integer\>](#integer-value) is defined as <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var> + <var>V<sub>b</sub></var>

<a id="ref-for-number-value②"></a>

### <a id="numbers"></a>5.3.  Real Numbers: the [\<number\>](#number-value) type

Number values are denoted by <a id="number-value"></a>\<number\>, and represent real numbers, possibly with a fractional component.

<a id="ref-for-integer"></a>

<a id="ref-for-typedef-number-token①"></a>

When written literally, a <a id="number"></a>number is either an [integer](#integer), or zero or more decimal digits followed by a dot (.) followed by one or more decimal digits and optionally an exponent composed of "e" or "E" and an integer. It corresponds to the [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). As with integers, the first character of a number may be immediately preceded by - or + to indicate the number’s sign.

<a id="ref-for-number"></a>

<a id="ref-for-number-value③"></a>

<a id="ref-for-zero-value"></a>

<a id="ref-for-typedef-number-token②"></a>

The value <a id="zero-value"></a>\<zero\> represents a literal [number](#number) with the value 0. Expressions that merely evaluate to a [\<number\>](#number-value) with the value 0 (for example, calc(0)) do not match [\<zero\>](#zero-value); only literal [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token)s do.

<a id="ref-for-number-value④"></a>

#### <a id="combine-numbers"></a>5.3.1.  Combination of [\<number\>](#number-value)

<a id="ref-for-interpolation③"></a>

<a id="ref-for-number-value⑤"></a>

[Interpolation](#interpolation) of [\<number\>](#number-value) is defined as <var>V</var><sub>result</sub> = (1 - <var>p</var>) × <var>V<sub>a</sub></var> + <var>p</var> × <var>V<sub>b</sub></var>

<a id="ref-for-addition⑧"></a>

<a id="ref-for-number-value⑥"></a>

[Addition](#addition) of [\<number\>](#number-value) is defined as <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var> + <var>V<sub>b</sub></var>

<a id="ref-for-dimension②"></a>

### <a id="dimensions"></a>5.4.  Numbers with Units: [dimension](#dimension) values

The general term <a id="dimension"></a>dimension refers to a number with a unit attached to it; and is denoted by <a id="typedef-dimension"></a>\<dimension\>.

<a id="ref-for-dimension③"></a>

<a id="ref-for-number①"></a>

<a id="ref-for-typedef-dimension-token①"></a>

<a id="ref-for-ascii-case-insensitive②"></a>

When written literally, a [dimension](#dimension) is a [number](#number) immediately followed by a unit identifier, which is an identifier. It corresponds to the [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). Like keywords, unit identifiers are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive).

<a id="ref-for-typedef-dimension"></a>

<a id="ref-for-length-value⑤"></a>

<a id="ref-for-time-value①"></a>

<a id="ref-for-frequency-value①"></a>

<a id="ref-for-resolution-value①"></a>

CSS uses [\<dimension\>](#typedef-dimension)s to specify distances ([\<length\>](#length-value)), durations ([\<time\>](#time-value)), frequencies ([\<frequency\>](#frequency-value)), resolutions ([\<resolution\>](#resolution-value)), and other quantities.

#### <a id="compat"></a>5.4.1.  Compatible Units

<a id="ref-for-computed-value⑤"></a>

<a id="ref-for-px"></a>

<a id="ref-for-in"></a>

<a id="ref-for-propdef-font-size"></a>

<a id="ref-for-em"></a>

<a id="ref-for-canonical-unit①"></a>

When [serializing](https://www.w3.org/TR/cssom-1/#serializing-css-values) [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) [\[CSSOM\]](#biblio-cssom), <a id="compatible-units"></a>compatible units (those related by a static multiplicative factor, like the 96:1 factor between [px](#px) and [in](#in), or the computed [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) factor between [em](#em) and <a id="ref-for-px①"></a>px) are converted into a single <a id="canonical-unit"></a>canonical unit. Each group of compatible units defines which among them is the [canonical unit](#canonical-unit) that will be used for serialization.

<a id="ref-for-used-value"></a>

<a id="ref-for-compatible-units"></a>

<a id="ref-for-dimension④"></a>

When serializing [resolved values](https://www.w3.org/TR/cssom-1/#resolved-values) that are [used values](https://www.w3.org/TR/css-cascade-5/#used-value), all value types (percentages, numbers, keywords, etc.) that represent lengths are considered [compatible](#compatible-units) with lengths. Likewise any future API that returns <a id="ref-for-used-value①"></a>used values must consider any values that represent distances/durations/frequencies/etc. as <a id="ref-for-compatible-units①"></a>compatible with the relevant class of [dimensions](#dimension), and canonicalize accordingly.

#### <a id="combine-dimensions"></a>5.4.2.  Combination of Dimensions

<a id="ref-for-interpolation④"></a>

<a id="ref-for-compatible-units②"></a>

<a id="ref-for-dimension⑤"></a>

<a id="ref-for-length-value⑥"></a>

[Interpolation](#interpolation) of [compatible](#compatible-units) [dimensions](#dimension) (for example, two [\<length\>](#length-value) values) is defined as <var>V</var><sub>result</sub> = (1 - <var>p</var>) × <var>V<sub>a</sub></var> + <var>p</var> × <var>V<sub>b</sub></var>

<a id="ref-for-addition⑨"></a>

<a id="ref-for-compatible-units③"></a>

<a id="ref-for-dimension⑥"></a>

[Addition](#addition) of [compatible](#compatible-units) [dimensions](#dimension) is defined as <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var> + <var>V<sub>b</sub></var>

<a id="ref-for-percentage-value②"></a>

### <a id="percentages"></a>5.5.  Percentages: the [\<percentage\>](#percentage-value) type

Percentage values are denoted by <a id="percentage-value"></a>\<percentage\>, and indicates a value that is some fraction of another reference value.

<a id="ref-for-number②"></a>

<a id="ref-for-typedef-percentage-token"></a>

When written literally, a <a id="percentage"></a>percentage consists of a [number](#number) immediately followed by a percent sign %. It corresponds to the [\<percentage-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-percentage-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3).

<a id="ref-for-containing-block"></a>

Percentage values are always relative to another quantity, for example a length. Each property that allows percentages also defines the quantity to which the percentage refers. This quantity can be a value of another property for the same element, the value of a property for an ancestor element, a measurement of the formatting context (e.g., the width of a [containing block](https://www.w3.org/TR/css-display-3/#containing-block)), or something else.

<a id="ref-for-percentage-value③"></a>

#### <a id="combine-percentages"></a>5.5.1.  Combination of [\<percentage\>](#percentage-value)

<a id="ref-for-interpolation⑤"></a>

<a id="ref-for-percentage-value④"></a>

[Interpolation](#interpolation) of [\<percentage\>](#percentage-value) is defined as <var>V</var><sub>result</sub> = (1 - <var>p</var>) × <var>V<sub>a</sub></var> + <var>p</var> × <var>V<sub>b</sub></var>

<a id="ref-for-addition①⓪"></a>

<a id="ref-for-percentage-value⑤"></a>

[Addition](#addition) of [\<percentage\>](#percentage-value) is defined as <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var> + <var>V<sub>b</sub></var>

### <a id="mixed-percentages"></a>5.6.  Mixing Percentages and Dimensions

<a id="ref-for-percentage-value⑥"></a>

<a id="ref-for-dimension⑦"></a>

<a id="ref-for-component-value①"></a>

<a id="ref-for-funcdef-calc"></a>

In cases where a [\<percentage\>](#percentage-value) can represent the same quantity as a [dimension](#dimension) in the same [component value](https://www.w3.org/TR/css-syntax-3/#component-value) position, and can therefore be combined with them in a [calc()](#funcdef-calc) expression, the following convenience notations may be used in the property grammar:

<a id="typedef-length-percentage"></a>\<length-percentage\>  
<a id="ref-for-length-value⑦"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-percentage-value⑦"></a>

<a id="ref-for-percentage-value⑧"></a>

<a id="ref-for-length-value⑧"></a>

Equivalent to <code><c->&#x5B;</c->&#x20;<a href="#length-value" title="Expands to: advance measure | cap | ch | cm | dvb | dvh | dvi | dvmax | dvmin | dvw | em | ex | ic | in | lh | lvb | lvh | lvi | lvmax | lvmin | lvw | mm | pc | pt | px | q | rem | rlh | svb | svh | svi | svmax | svmin | svw | vb | vh | vi | vmax | vmin | vw">&lt;length&gt;</a>&#x20;<a href="#comb-one">&#x7C;</a>&#x20;<a href="#percentage-value">&lt;percentage&gt;</a>&#x20;<c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<length\>](#length-value).

<a id="typedef-frequency-percentage"></a>\<frequency-percentage\>  
<a id="ref-for-frequency-value②"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-percentage-value⑨"></a>

<a id="ref-for-percentage-value①⓪"></a>

<a id="ref-for-frequency-value③"></a>

Equivalent to <code><c->&#x5B;</c->&#x20;<a href="#frequency-value" title="Expands to: hz | khz">&lt;frequency&gt;</a>&#x20;<a href="#comb-one">&#x7C;</a>&#x20;<a href="#percentage-value">&lt;percentage&gt;</a>&#x20;<c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<frequency\>](#frequency-value).

<a id="typedef-angle-percentage"></a>\<angle-percentage\>  
<a id="ref-for-angle-value②"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-percentage-value①①"></a>

<a id="ref-for-percentage-value①②"></a>

<a id="ref-for-angle-value③"></a>

Equivalent to <code><c->&#x5B;</c->&#x20;<a href="#angle-value" title="Expands to: deg | grad | rad | turn">&lt;angle&gt;</a>&#x20;<a href="#comb-one">&#x7C;</a>&#x20;<a href="#percentage-value">&lt;percentage&gt;</a>&#x20;<c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to an [\<angle\>](#angle-value).

<a id="typedef-time-percentage"></a>\<time-percentage\>  
<a id="ref-for-time-value②"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-percentage-value①③"></a>

<a id="ref-for-percentage-value①④"></a>

<a id="ref-for-time-value③"></a>

Equivalent to <code><c->&#x5B;</c->&#x20;<a href="#time-value" title="Expands to: ms | s">&lt;time&gt;</a>&#x20;<a href="#comb-one">&#x7C;</a>&#x20;<a href="#percentage-value">&lt;percentage&gt;</a>&#x20;<c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<time\>](#time-value).

<a id="ref-for-propdef-width"></a>

<a id="ref-for-length-value⑨"></a>

<a id="ref-for-percentage-value①⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ac6b1005"></a> For example, the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property can accept a [\<length\>](#length-value) or a [\<percentage\>](#percentage-value), both representing a measure of distance. This means that <a id="ref-for-propdef-width①"></a>width: calc(500px + 50%); is allowed—both values are converted to absolute lengths and added. If the containing block is 1000px wide, then <a id="ref-for-propdef-width②"></a>width: 50%; is equivalent to <a id="ref-for-propdef-width③"></a>width: 500px, and <a id="ref-for-propdef-width④"></a>width: calc(50% + 500px) thus ends up equivalent to <a id="ref-for-propdef-width⑤"></a>width: calc(500px + 500px) or <a id="ref-for-propdef-width⑥"></a>width: 1000px.
>
> <a id="ref-for-funcdef-hsl"></a>
>
> <a id="ref-for-percentage-value①⑥"></a>
>
> <a id="ref-for-funcdef-calc①"></a>
>
> On the other hand, the second and third arguments of the [hsl()](https://www.w3.org/TR/css-color-5/#funcdef-hsl) function can only be expressed as [\<percentage\>](#percentage-value)s. Although [calc()](#funcdef-calc) productions are allowed in their place, they can only combine percentages with themselves, as in calc(10% + 20%).

<a id="ref-for-percentage-value①⑦"></a>

<a id="ref-for-compatible-units④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Specifications should never alternate [\<percentage\>](#percentage-value) in place of a dimension in a grammar unless they are [compatible](#compatible-units).

<a id="ref-for-number-value⑦"></a>

<a id="ref-for-percentage-value①⑧"></a>

<a id="ref-for-funcdef-calc②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: More \<<var>type</var>-percentage\> productions can be added in the future as needed. A \<number-percentage\> will never be added, as [\<number\>](#number-value) and [\<percentage\>](#percentage-value) can’t be combined in [calc()](#funcdef-calc).

#### <a id="combine-mixed"></a>5.6.1.  Combination of Percentage and Dimension Mixes

<a id="ref-for-interpolation⑥"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="ref-for-typedef-frequency-percentage"></a>

<a id="ref-for-typedef-angle-percentage"></a>

<a id="ref-for-typedef-time-percentage"></a>

[Interpolation](#interpolation) of percengage-dimension value combinations (e.g. [\<length-percentage\>](#typedef-length-percentage), [\<frequency-percentage\>](#typedef-frequency-percentage), [\<angle-percentage\>](#typedef-angle-percentage), [\<time-percentage\>](#typedef-time-percentage) or equivalent notations) is defined as

- <a id="ref-for-length-value①⓪"></a>

  <a id="ref-for-interpolation⑦"></a>

  equivalent to [interpolation](#interpolation) of [\<length\>](#length-value) if both <var>V<sub>a</sub></var> and <var>V<sub>b</sub></var> are pure <a id="ref-for-length-value①①"></a>\<length\> values

- <a id="ref-for-percentage-value①⑨"></a>

  <a id="ref-for-interpolation⑧"></a>

  equivalent to [interpolation](#interpolation) of [\<percentage\>](#percentage-value) if both <var>V<sub>a</sub></var> and <var>V<sub>b</sub></var> are pure <a id="ref-for-percentage-value②⓪"></a>\<percentage\> values

- <a id="ref-for-percentage-value②①"></a>

  <a id="ref-for-time-value④"></a>

  <a id="ref-for-angle-value④"></a>

  <a id="ref-for-frequency-value④"></a>

  <a id="ref-for-length-value①②"></a>

  <a id="ref-for-interpolation⑨"></a>

  <a id="ref-for-funcdef-calc③"></a>

  equivalent to converting both values into a [calc()](#funcdef-calc) expression representing the sum of the dimension type and a percentage (each possibly zero) and [interpolating](#interpolation) each component individually (as a [\<length\>](#length-value)/[\<frequency\>](#frequency-value)/[\<angle\>](#angle-value)/[\<time\>](#time-value) and as a [\<percentage\>](#percentage-value), respectively)

<a id="ref-for-addition①①"></a>

<a id="ref-for-percentage-value②②"></a>

<a id="ref-for-interpolation①⓪"></a>

[Addition](#addition) of [\<percentage\>](#percentage-value) is defined the same as [interpolation](#interpolation) except by <a id="ref-for-addition①②"></a>adding each component rather than <a id="ref-for-interpolation①①"></a>interpolating it.

<a id="ref-for-ratio-value"></a>

### <a id="ratios"></a>5.7.  Ratios: the [\<ratio\>](#ratio-value) type

Ratio values are denoted by <a id="ratio-value"></a>\<ratio\>, and represent the ratio of two numeric values. It most often represents an aspect ratio, relating a width (first) to a height (second).

When written literally, a <a id="ratio"></a>ratio has the syntax:

<a id="ref-for-ratio-value①"></a>

<a id="ref-for-number-value⑧"></a>

<a id="ref-for-number-value⑨"></a>

<a id="ref-for-mult-opt③"></a>

```text
<ratio> = <number [0,∞]> [ / <number [0,∞]> ]?
```
<a id="ref-for-number-value①⓪"></a>

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

<a id="ref-for-length-value①③"></a>

## <a id="lengths"></a>6.  Distance Units: the [\<length\>](#length-value) type

<a id="ref-for-dimension⑧"></a>

Lengths refer to distance measurements and are denoted by <a id="length-value"></a>\<length\> in the property definitions. A length is a [dimension](#dimension).

<a id="ref-for-number-value①①"></a>

<a id="ref-for-length-value①④"></a>

<a id="ref-for-propdef-line-height"></a>

For zero lengths the unit identifier is optional (i.e. can be syntactically represented as the [\<number\>](#number-value) 0). However, if a 0 could be parsed as either a <a id="ref-for-number-value①②"></a>\<number\> or a [\<length\>](#length-value) in a property (such as [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height)), it must parse as a <a id="ref-for-number-value①③"></a>\<number\>.

Properties may restrict the length value to some range. If the value is outside the allowed range, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore).

While some properties allow negative length values, this may complicate the formatting and there may be implementation-specific limits. If a negative length value is allowed but cannot be supported, it must be converted to the nearest value that can be supported.

<a id="ref-for-used-value②"></a>

<a id="ref-for-actual-value"></a>

In cases where the [used](https://www.w3.org/TR/css-cascade-5/#used-value) length cannot be supported, user agents must approximate it in the [actual](https://www.w3.org/TR/css-cascade-5/#actual-value) value.

<a id="ref-for-relative-length"></a>

<a id="ref-for-absolute-length"></a>

There are two types of length units: [relative](#relative-length) and [absolute](#absolute-length).

### <a id="relative-lengths"></a>6.1.  Relative Lengths

<a id="relative-length"></a>Relative length units specify a length relative to another length. Style sheets that use relative units can more easily scale from one output environment to another.

The relative units are:



| unit                                           | relative to                                                                                                                                                                        |
|------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <a id="ref-for-em①"></a>[em](#em)                   | font size of the element                                                                                                                                                           |
| <a id="ref-for-ex"></a>[ex](#ex)                   | x-height of the element’s font                                                                                                                                                     |
| <a id="ref-for-cap"></a>[cap](#cap)                 | cap height (the nominal height of capital letters) of the element’s font                                                                                                           |
| <a id="ref-for-ch"></a>[ch](#ch)                   | <a id="ref-for-length-advance-measure"></a>typical [character advance](#length-advance-measure) of a narrow glyph in the element’s font, as represented by the “0” (ZERO, U+0030) glyph                    |
| <a id="ref-for-ic"></a>[ic](#ic)                   | <a id="ref-for-length-advance-measure①"></a>typical [character advance](#length-advance-measure) of a fullwidth glyph in the element’s font, as represented by the “水” (CJK water ideograph, U+6C34) glyph |
| <a id="ref-for-rem"></a>[rem](#rem)                 | font size of the root element                                                                                                                                                      |
| <a id="ref-for-lh"></a>[lh](#lh)                   | line height of the element                                                                                                                                                         |
| <a id="ref-for-rlh"></a>[rlh](#rlh)                 | line height of the root element                                                                                                                                                    |
| <a id="ref-for-valdef-length-vw"></a>[vw](#valdef-length-vw)     | 1% of viewport’s width                                                                                                                                                             |
| <a id="ref-for-valdef-length-vh"></a>[vh](#valdef-length-vh)     | 1% of viewport’s height                                                                                                                                                            |
| <a id="ref-for-valdef-length-vi"></a>[vi](#valdef-length-vi)     | <a id="ref-for-inline-axis"></a>1% of viewport’s size in the root element’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis)                                               |
| <a id="ref-for-valdef-length-vb"></a>[vb](#valdef-length-vb)     | <a id="ref-for-block-axis"></a>1% of viewport’s size in the root element’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis)                                                 |
| <a id="ref-for-valdef-length-vmin"></a>[vmin](#valdef-length-vmin) | 1% of viewport’s smaller dimension                                                                                                                                                 |
| <a id="ref-for-valdef-length-vmax"></a>[vmax](#valdef-length-vmax) | 1% of viewport’s larger dimension                                                                                                                                                  |

Informative Summary of Relative Units



<a id="ref-for-computed-value⑥"></a>

Child elements do not inherit the relative values as specified for their parent; they inherit the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

<a id="ref-for-em②"></a>

<a id="ref-for-ex①"></a>

<a id="ref-for-cap①"></a>

<a id="ref-for-ch①"></a>

<a id="ref-for-ic①"></a>

<a id="ref-for-rem①"></a>

<a id="ref-for-lh①"></a>

<a id="ref-for-rlh①"></a>

#### <a id="font-relative-lengths"></a>6.1.1.  Font-relative Lengths: the [em](#em), [ex](#ex), [cap](#cap), [ch](#ch), [ic](#ic), [rem](#rem), [lh](#lh), [rlh](#rlh) units

<a id="ref-for-rem②"></a>

<a id="ref-for-rlh②"></a>

The <a id="font-relative-length"></a>font-relative lengths refer to the font metrics of the element on which they are used—or, in the case of [rem](#rem) and [rlh](#rlh), the metrics of the root element.

![The word 'Sphinx' annotated with various font metrics: ascender height, to the top of the h’s serif; cap height, to the visually approximate top of the S; the x height, to the visually approximate top of the x; the baseline, along the bottom of S, h, i, n, and x; and the descender height, to the bottom fo the p.](https://www.w3.org/TR/2021/WD-css-values-4-20210715/images/Typography_Line_Terms.svg)

Common typographic metrics

<a id="em"></a>em unit  
<a id="ref-for-propdef-font-size①"></a>

Equal to the computed value of the [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) property of the element on which it is used.

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

<a id="ex"></a>ex unit  
<a id="ref-for-ex②"></a>

Equal to the used x-height of the [first available font](https://www.w3.org/TR/css3-fonts/#first-available-font) [\[CSS3-FONTS\]](#biblio-css3-fonts). The x-height is so called because it is often equal to the height of the lowercase "x". However, an [ex](#ex) is defined even for fonts that do not contain an "x". The x-height of a font can be found in different ways. Some fonts contain reliable metrics for the x-height. If reliable font metrics are not available, UAs may determine the x-height from the height of a lowercase glyph. One possible heuristic is to look at how far the glyph for the lowercase "o" extends below the baseline, and subtract that value from the top of its bounding box. In the cases where it is impossible or impractical to determine the x-height, a value of 0.5em must be assumed.

<a id="cap"></a>cap unit  
<a id="ref-for-cap②"></a>

Equal to the used cap-height of the [first available font](https://www.w3.org/TR/css3-fonts/#first-available-font) [\[CSS3-FONTS\]](#biblio-css3-fonts). The cap-height is so called because it is approximately equal to the height of a capital Latin letter. However, a [cap](#cap) is defined even for fonts that do not contain Latin letters. The cap-height of a font can be found in different ways. Some fonts contain reliable metrics for the cap-height. If reliable font metrics are not available, UAs may determine the cap-height from the height of an uppercase glyph. One possible heuristic is to look at how far the glyph for the uppercase “O” extends below the baseline, and subtract that value from the top of its bounding box. In the cases where it is impossible or impractical to determine the cap-height, the font’s ascent must be used.

<a id="ch"></a>ch unit  
<a id="ref-for-length-advance-measure②"></a>

Equal to the used [advance measure](#length-advance-measure) of the “0” (ZERO, U+0030) glyph in the font used to render it. (The <a id="length-advance-measure"></a>advance measure of a glyph is its advance width or height, whichever is in the inline axis of the element.)

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

<a id="ic"></a>ic unit  
<a id="ref-for-length-advance-measure④"></a>

Equal to the used [advance measure](#length-advance-measure) of the “水” (CJK water ideograph, U+6C34) glyph found in the font used to render it.

<a id="ref-for-length-advance-measure⑤"></a>

This measurement is a typically an exact measure (in the few fonts with proportional fullwidth glyphs, an approximation) of a single [fullwidth](http://unicode.org/reports/tr11/#Definitions) glyph’s [advance measure](#length-advance-measure), thus allowing measurements based on an expected glyph count.

In the cases where it is impossible or impractical to determine the ideographic advance measure, it must be assumed to be 1em.

<a id="rem"></a>rem unit  
<a id="ref-for-propdef-font-size②"></a>

Equal to the computed value of [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) on the root element. When specified in the <a id="ref-for-propdef-font-size③"></a>font-size property of the root element, or in a document with no root element, 1rem is equal to the initial value of the <a id="ref-for-propdef-font-size④"></a>font-size property.

<a id="lh"></a>lh unit  
<a id="ref-for-valdef-line-height-normal"></a>

<a id="ref-for-propdef-line-height①"></a>

Equal to the computed value of the [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) property of the element on which it is used, converting [normal](https://www.w3.org/TR/css-inline-3/#valdef-line-height-normal) to an absolute length by using only the metrics of the [first available font](https://www.w3.org/TR/css3-fonts/#first-available-font).

<a id="rlh"></a>rlh unit  
<a id="ref-for-valdef-line-height-normal①"></a>

<a id="ref-for-propdef-line-height②"></a>

Equal to the computed value of [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) property on the root element, converting [normal](https://www.w3.org/TR/css-inline-3/#valdef-line-height-normal) to an absolute length as above.

<a id="ref-for-propdef-height"></a>

<a id="ref-for-lh②"></a>

<a id="ref-for-rlh③"></a>

<a id="ref-for-propdef-max-lines"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Setting the [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) of an element using either the [lh](#lh) or the [rlh](#rlh) units does not enable authors to control the actual number of lines in that element. These units only enable length calculations based on the theoretical size of an ideal empty line; the size of actual lines boxes may differ based on their content. In cases where an author wants to limit the number of actual lines in an element, the [max-lines](https://www.w3.org/TR/css-overflow-4/#propdef-max-lines) property can be used instead.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-7dc6ac0d"></a> We can potentially add more typographic units, like cicero, didot, etc. They’re just absolute units, and so can be done with the existing units, but is there enough desire for them (potentially for printing use-cases) that it would be worth adding them? Or should we just wait for Houdini Custom Units?

<a id="ref-for-used-value③"></a>

Some user-agents allow users to apply additional restrictions to font sizes in a document, such as setting minimum font sizes to ensure readability. When used in the context of an element, these additional restrictions must be applied to the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of these properties only; they <em>must not</em> affect the resolution of relative units.

<a id="ref-for-media-query"></a>

<a id="ref-for-propdef-font"></a>

<a id="ref-for-propdef-line-height③"></a>

When used outside the context of an element (such as in [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query)), these units refer to the metrics corresponding to the initial values of the [font](https://www.w3.org/TR/css-fonts-3/#propdef-font) and [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) properties. In this context, the units must apply any additional restrictions to the values, contrary to the normal behavior mentioned above.

<a id="ref-for-propdef-font-size⑤"></a>

<a id="ref-for-propdef-font①"></a>

<a id="ref-for-propdef-line-height④"></a>

<a id="ref-for-lh③"></a>

<a id="ref-for-rlh④"></a>

When used in the value of the [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) property on the element they refer to, they resolve against the computed metrics of the parent element—or against the computed metrics corresponding to the initial values of the [font](https://www.w3.org/TR/css-fonts-3/#propdef-font) and [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) properties, if the element has no parent. Additionally, when [lh](#lh) or [rlh](#rlh) units are used in the value of the <a id="ref-for-propdef-line-height⑤"></a>line-height property on the element they refer to, they resolve against the computed <a id="ref-for-propdef-line-height⑥"></a>line-height and font metrics of the parent element—or the computed metrics corresponding to the initial values of the <a id="ref-for-propdef-font②"></a>font and <a id="ref-for-propdef-line-height⑦"></a>line-height properties, if the element has no parent. (The other font-relative units continue to resolve against the element’s own metrics when used in <a id="ref-for-propdef-line-height⑧"></a>line-height.)

#### <a id="viewport-relative-lengths"></a>6.1.2.  Viewport-percentage Lengths: the \*vw, \*vh, \*vi, \*vb, \*vmin, \*vmax units

<a id="ref-for-continuous-media"></a>

<a id="ref-for-page-area"></a>

<a id="ref-for-paged-media"></a>

The <a id="viewport-percentage-lengths"></a>viewport-percentage lengths are relative to the size of the [initial containing block](https://www.w3.org/TR/CSS21/visudet.html#containing-block-details)—which is itself based on the size of either the viewport (for [continuous media](https://www.w3.org/TR/mediaqueries-5/#continuous-media)) or the [page area](https://www.w3.org/TR/css-page-3/#page-area) (for [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media)). When the height or width of the initial containing block is changed, they are scaled accordingly.

##### <a id="viewport-variants"></a>6.1.2.1.  The Large, Small, and Dynamic Viewport Sizes

<a id="ref-for-viewport-percentage-lengths"></a>

There are four variants of the [viewport-percentage length](#viewport-percentage-lengths) units, corresponding to four (possibly identical) notions of the viewport size.

UA-default viewport  
<a id="ref-for-small-viewport-size"></a>

<a id="ref-for-large-viewport-size"></a>

The <a id="ua-default-viewport-percentage-units"></a>UA-default viewport-percentage units (v\*) are defined with respect to a UA-defined <a id="ua-default-viewport-size"></a>UA-default viewport size, which for any given document should be equivalent to the [large viewport size](#large-viewport-size), [small viewport size](#small-viewport-size), or some intermediary size.

<a id="ref-for-large-viewport-size①"></a>

<a id="ref-for-small-viewport-size①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Implementations that choose a size other than the [large viewport size](#large-viewport-size) or [small viewport size](#small-viewport-size) are encouraged to explain their choice to the CSSWG for consideration in future specification updates.

large viewport  
The <a id="large-viewport-percentage-units"></a>large viewport-percentage units (lv\*) are defined with respect to the <a id="large-viewport-size"></a>large viewport size: the viewport sized assuming any UA interfaces that are dynamically expanded and retracted to be retracted. This allows authors to size content such that it is guaranteed to fill the viewport, noting that such content might be hidden behind such interfaces when they are expanded.

<a id="ref-for-large-viewport-percentage-units"></a>

The sizes of the [large viewport-percentage units](#large-viewport-percentage-units) are fixed (and therefore stable) unless the viewport itself is resized.

<a id="ref-for-large-viewport-percentage-units①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-18205b29"></a> For example, on phones, where screen real-estate is at a premium, browsers will often hide part or all of the title and address bar once the user starts scrolling the page. The [large viewport-percentage units](#large-viewport-percentage-units) are sized relative to this larger everything-retracted space, so content using these units will fill the entire visible page when these UI elements are hidden. However, when these retractable elements are shown, they can obscure content that is sized or positioned using these units.

small viewport  
The <a id="small-viewport-percentage-units"></a>small viewport-percentage units (sv\*) are defined with respect to the <a id="small-viewport-size"></a>small viewport size: the viewport sized assuming any UA interfaces that are dynamically expanded and retracted to be expanded. This allows authors to size content such that it can fit within the viewport even when such interfaces are present, noting that such content might not fill the viewport when such interfaces are retracted.

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
The <a id="dynamic-viewport-percentage-units"></a>dynamic viewport-percentage units (dv\*) are defined with respect to the <a id="dynamic-viewport-size"></a>dynamic viewport size: the viewport sized with dynamic consideration of any UA interfaces that are dynamically expanded and retracted. This allows authors to size content such that it can exactly fit within the viewport whether or not such interfaces are present.

<a id="ref-for-dynamic-viewport-percentage-units"></a>

The sizes of the [dynamic viewport-percentage units](#dynamic-viewport-percentage-units) <em>are not stable</em> even while the viewport itself is unchanged. Using these units can cause content to resize e.g. while the user scrolls the page. Depending on usage, this can be disturbing to the user and/or costly in terms of performance.

<a id="ref-for-dynamic-viewport-percentage-units①"></a>

The UA is not required to animate the [dynamic viewport-percentage units](#dynamic-viewport-percentage-units) while expanding and retracting any relevant interfaces, and may instead calculate the units as if the relevant interface was fully expanded or retracted during the UI animation. (It is recommended that UAs assume the fully-retracted size for this duration.)

<a id="ref-for-viewport-percentage-lengths①"></a>

<a id="ref-for-initial-containing-block"></a>

<a id="ref-for-large-viewport-size②"></a>

<a id="ref-for-small-viewport-size②"></a>

Whether the expansion/retraction of a particular interface (A) changes the sizes of all of the [viewport-percentage lengths](#viewport-percentage-lengths) (and the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block)) simultaneously or (B) contributes to the differences between the [large viewport size](#large-viewport-size) and [small viewport size](#small-viewport-size) is largely UA-dependent. However:

- Changes in interface that happen as a result of scrolling or other frequent page interactions that would disturb the user if they resulted in substantial layout changes must be categorized as the former (A).

- Changes in interface that have a sufficiently steady state that re-laying out the document into the adjusted space would be beneficial to the user must be categorized as the latter (B).

- <a id="ref-for-viewport-percentage-lengths②"></a>

  Additionally, UAs may have some dynamically-shown interfaces that intentionally overlay content and do not cause any shifts in layout—and therefore have no effect on any of the [viewport-percentage lengths](#viewport-percentage-lengths). (Typically on-screen keyboards will fit into this category.)

<a id="ref-for-initial-containing-block①"></a>

In all cases, scrollbars are assumed not to exist. <strong data-conversion-semantic="note">Note:</strong> Note however that the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block)'s size <em>is</em> affected by the presence of scrollbars on the viewport.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-56221f3d"></a> [Level 3 assumes scrollbars never exist](https://www.w3.org/TR/css-values-3/#viewport-relative-lengths) because it was hard to implement and only Firefox bothered to do so. This is [making authors unhappy](https://github.com/w3c/csswg-drafts/issues/1766#issuecomment-460470368). Can we improve here?

##### <a id="viewport-relative-units"></a>6.1.2.2.  The Various Viewport-relative Units

<a id="ref-for-viewport-percentage-lengths③"></a>

The [viewport-percentage length](#viewport-percentage-lengths) units are:

<a id="valdef-length-vw"></a>vw unit  
<a id="valdef-length-svw"></a>svw unit  
<a id="valdef-length-lvw"></a>lvw unit  
<a id="valdef-length-dvw"></a>dvw unit  
<a id="ref-for-dynamic-viewport-size"></a>

<a id="ref-for-large-viewport-size③"></a>

<a id="ref-for-small-viewport-size③"></a>

<a id="ref-for-ua-default-viewport-size"></a>

Equal to 1% of the width of the [UA-default viewport size](#ua-default-viewport-size), [small viewport size](#small-viewport-size), [large viewport size](#large-viewport-size), and [dynamic viewport size](#dynamic-viewport-size), respectively.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6068ee5d"></a> In the example below, if the width of the viewport is 200mm, the font size of `h1` elements will be 16mm (i.e. (8×200mm)/100).
> ```text
> h1 { font-size: 8vw }
> ```
<a id="valdef-length-vh"></a>vh unit  
<a id="valdef-length-svh"></a>svh unit  
<a id="valdef-length-lvh"></a>lvh unit  
<a id="valdef-length-dvh"></a>dvh unit  
<a id="ref-for-dynamic-viewport-size①"></a>

<a id="ref-for-large-viewport-size④"></a>

<a id="ref-for-small-viewport-size④"></a>

<a id="ref-for-ua-default-viewport-size①"></a>

Equal to 1% of the height of the [UA-default viewport size](#ua-default-viewport-size), [small viewport size](#small-viewport-size), [large viewport size](#large-viewport-size), and [dynamic viewport size](#dynamic-viewport-size), respectively.

<a id="valdef-length-vi"></a>vi unit  
<a id="valdef-length-svi"></a>svi unit  
<a id="valdef-length-lvi"></a>lvi unit  
<a id="valdef-length-dvi"></a>dvi unit  
<a id="ref-for-dynamic-viewport-size②"></a>

<a id="ref-for-small-viewport-size⑤"></a>

<a id="ref-for-large-viewport-size⑤"></a>

Equal to 1% of the size of the [large viewport size](#large-viewport-size), [small viewport size](#small-viewport-size), and [dynamic viewport size](#dynamic-viewport-size) (respectively) in the direction of the root element’s inline axis.

<a id="valdef-length-vb"></a>vb unit  
<a id="valdef-length-svb"></a>svb unit  
<a id="valdef-length-lvb"></a>lvb unit  
<a id="valdef-length-dvb"></a>dvb unit  
<a id="ref-for-dynamic-viewport-size③"></a>

<a id="ref-for-large-viewport-size⑥"></a>

<a id="ref-for-small-viewport-size⑥"></a>

<a id="ref-for-ua-default-viewport-size②"></a>

Equal to 1% of the size of the initial containing block [UA-default viewport size](#ua-default-viewport-size), [small viewport size](#small-viewport-size), [large viewport size](#large-viewport-size), and [dynamic viewport size](#dynamic-viewport-size) (respectively) in the direction of the root element’s block axis.

<a id="valdef-length-vmin"></a>vmin unit  
<a id="valdef-length-svmin"></a>svmin unit  
<a id="valdef-length-lvmin"></a>lvmin unit  
<a id="valdef-length-dvmin"></a>dvmin unit  
Equal to the smaller of \*vw or \*vh.

<a id="valdef-length-vmax"></a>vmax unit  
<a id="valdef-length-svmax"></a>svmax unit  
<a id="valdef-length-lvmax"></a>lvmax unit  
<a id="valdef-length-dvmax"></a>dvmax unit  
Equal to the larger of \*vw or \*vh.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9b4752a7"></a> Originally the (unprefixed) viewport units were defined relative to the viewport size in general. The dynamism of browser chrome shifting in and out during scrolling was invented later, and following Safari’s lead, most UAs mapped these units to the larger size. Defining it this way is prettier in many cases, but can also block critical content (such as toolbars, headers, and footers) in others. It’s therefore not entirely clear whether this is the best mapping.

<a id="ref-for-media-query①"></a>

<a id="ref-for-propdef-writing-mode①"></a>

In situations where there is no root element or it hasn’t yet been styled (such as when evaluating [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query)), the \*vi and \*vb units use the initial value of the [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) property to determine which axis they correspond to.

<a id="ref-for-cm"></a>

<a id="ref-for-mm"></a>

<a id="ref-for-Q"></a>

<a id="ref-for-in①"></a>

<a id="ref-for-pt"></a>

<a id="ref-for-pc"></a>

<a id="ref-for-px②"></a>

### <a id="absolute-lengths"></a>6.2.  Absolute Lengths: the [cm](#cm), [mm](#mm), [Q](#Q), [in](#in), [pt](#pt), [pc](#pc), [px](#px) units

<a id="ref-for-anchor-unit"></a>

<a id="ref-for-in②"></a>

<a id="ref-for-cm①"></a>

<a id="ref-for-mm①"></a>

<a id="ref-for-pt①"></a>

<a id="ref-for-pc①"></a>

<a id="ref-for-Q①"></a>

<a id="ref-for-px③"></a>

The <a id="absolute-length"></a>absolute length units are fixed in relation to each other and [anchored](#anchor-unit) to some physical measurement. They are mainly useful when the output environment is known. The absolute units consist of the <a id="physical-unit"></a>physical units ([in](#in), [cm](#cm), [mm](#mm), [pt](#pt), [pc](#pc), [Q](#Q)) and the <a id="visual-angle-unit"></a>visual angle unit (pixel unit) ([px](#px)):



| unit                | name                | equivalence         |
|---------------------|---------------------|---------------------|
| <strong><dfn><span><a id="cm"></a></span>cm</dfn> &#xA;      </strong> | centimeters         | 1cm = 96px/2.54     |
| <strong><dfn><span><a id="mm"></a></span>mm</dfn> &#xA;      </strong> | millimeters         | 1mm = 1/10th of 1cm |
| <strong><dfn><span><a id="Q"></a></span>Q</dfn> &#xA;      </strong> | quarter-millimeters | 1Q = 1/40th of 1cm  |
| <strong><dfn><span><a id="in"></a></span>in</dfn> &#xA;      </strong> | inches              | 1in = 2.54cm = 96px |
| <strong><dfn><span><a id="pc"></a></span>pc</dfn> &#xA;      </strong> | picas               | 1pc = 1/6th of 1in  |
| <strong><dfn><span><a id="pt"></a></span>pt</dfn> &#xA;      </strong> | points              | 1pt = 1/72nd of 1in |
| <strong><dfn><span><a id="px"></a></span>px</dfn> &#xA;      </strong> | pixels              | 1px = 1/96th of 1in |



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
<a id="ref-for-compatible-units⑤"></a>

<a id="ref-for-px④"></a>

<a id="ref-for-canonical-unit②"></a>

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

For print media at typical viewing distances, the [anchor unit](#anchor-unit) should be one of the [physical units](#physical-unit) (inches, centimeters, etc). For screen media (including high-resolution devices), low-resolution devices, and devices with unusual viewing distances, it is recommended instead that the <a id="ref-for-anchor-unit②"></a>anchor unit be the [pixel unit](#visual-angle-unit). For such devices it is recommended that the <a id="ref-for-visual-angle-unit②"></a>pixel unit refer to the whole number of device pixels that best approximates the reference pixel.

<a id="ref-for-anchor-unit③"></a>

<a id="ref-for-visual-angle-unit③"></a>

<a id="ref-for-physical-unit②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the [anchor unit](#anchor-unit) is the [pixel unit](#visual-angle-unit), the [physical units](#physical-unit) might not match their physical measurements. Alternatively if the <a id="ref-for-anchor-unit④"></a>anchor unit is a <a id="ref-for-physical-unit③"></a>physical unit, the <a id="ref-for-visual-angle-unit④"></a>pixel unit might not map to a whole number of device pixels.

<a id="ref-for-visual-angle-unit⑤"></a>

<a id="ref-for-physical-unit④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This definition of the [pixel unit](#visual-angle-unit) and the [physical units](#physical-unit) differs from the earlier editions of CSS1 and CSS2. In particular, in previous versions of CSS the <a id="ref-for-visual-angle-unit⑥"></a>pixel unit and the <a id="ref-for-physical-unit⑤"></a>physical units were not related by a fixed ratio: the <a id="ref-for-physical-unit⑥"></a>physical units were always tied to their physical measurements while the <a id="ref-for-visual-angle-unit⑦"></a>pixel unit would vary to most closely match the reference pixel. (This unfortunate change was made because too much existing content relies on the assumption of 96dpi, and breaking that assumption broke the content.)

<a id="ref-for-ascii-case-insensitive③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Units are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) and serialize as lower case, for example 1Q serializes as 1q.

The <a id="reference-pixel"></a>reference pixel is the visual angle of one pixel on a device with a pixel density of 96dpi and a distance from the reader of an arm’s length. For a nominal arm’s length of 28 inches, the visual angle is therefore about 0.0213 degrees. For reading at arm’s length, 1px thus corresponds to about 0.26 mm (1/96 inch).

The image below illustrates the effect of viewing distance on the size of a reference pixel: a reading distance of 71 cm (28 inches) results in a reference pixel of 0.26 mm, while a reading distance of 3.5 m (12 feet) results in a reference pixel of 1.3 mm.

![This diagram illustrates how the definition of a pixel depends on the users distance from the viewing surface (paper or screen). The image depicts the user looking at two planes, one 28 inches (71 cm) from the user, the second 140 inches (3.5 m) from the user. An expanding cone is projected from the user’s eye onto each plane. Where the cone strikes the first plane, the projected pixel is 0.26 mm high. Where the cone strikes the second plane, the projected pixel is 1.4 mm high.](https://www.w3.org/TR/2021/WD-css-values-4-20210715/images/pixel1.png)

Showing that pixels must become larger if the viewing distance increases

This second image illustrates the effect of a device’s resolution on the pixel unit: an area of 1px by 1px is covered by a single dot in a low-resolution device (e.g. a typical computer display), while the same area is covered by 16 dots in a higher resolution device (such as a printer).

![This diagram illustrates the relationship between the reference pixel and device pixels (called "dots" below). The image depicts a high resolution (large dot density) laser printer output on the left and a low resolution monitor screen on the right. For the laser printer, one square reference pixel is implemented by 16 dots. For the monitor screen, one square reference pixel is implemented by a single dot.](https://www.w3.org/TR/2021/WD-css-values-4-20210715/images/pixel2.png)

Showing that more device pixels (dots) are needed to cover a 1px by 1px area on a high-resolution device than on a lower-resolution one (of the same approximate viewing distance)

## <a id="other-units"></a>7.  Other Quantities

<a id="ref-for-angle-value⑤"></a>

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

<a id="ref-for-angle-value⑥"></a>

<a id="ref-for-compatible-units⑥"></a>

<a id="ref-for-deg①"></a>

<a id="ref-for-canonical-unit③"></a>

All [\<angle\>](#angle-value) units are [compatible](#compatible-units), and [deg](#deg) is their [canonical unit](#canonical-unit).

> <strong data-conversion-semantic="note">Note</strong>
>
> By convention, when an angle denotes a direction in CSS, it is typically interpreted as a <a id="bearing-angle"></a>bearing angle, where 0deg is "up" or "north" on the screen, and larger angles are more clockwise (so 90deg is "right" or "east").
>
> <a id="ref-for-funcdef-linear-gradient"></a>
>
> <a id="ref-for-angle-value⑦"></a>
>
> For example, in the [linear-gradient()](https://www.w3.org/TR/css-images-3/#funcdef-linear-gradient) function, the [\<angle\>](#angle-value) that determines the direction of the gradient is interpreted as a bearing angle.

<a id="ref-for-angle-value⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For legacy reasons, some uses of [\<angle\>](#angle-value) allow a bare 0 to mean 0deg. This is not true in general, however, and will not occur in future uses of the <a id="ref-for-angle-value⑨"></a>\<angle\> type.

<a id="ref-for-time-value⑤"></a>

<a id="ref-for-s"></a>

<a id="ref-for-ms"></a>

### <a id="time"></a>7.2.  Duration Units: the [\<time\>](#time-value) type and [s](#s), [ms](#ms) units

<a id="ref-for-dimension⑨"></a>

Time values are [dimensions](#dimension) denoted by <a id="time-value"></a>\<time\>. The time unit identifiers are:

<a id="s"></a>s  
Seconds.

<a id="ms"></a>ms  
Milliseconds. There are 1000 milliseconds in a second.

<a id="ref-for-time-value⑥"></a>

<a id="ref-for-compatible-units⑦"></a>

<a id="ref-for-s①"></a>

<a id="ref-for-canonical-unit④"></a>

All [\<time\>](#time-value) units are [compatible](#compatible-units), and [s](#s) is their [canonical unit](#canonical-unit).

Properties may restrict the time value to some range. If the value is outside the allowed range, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore).

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

<a id="ref-for-canonical-unit⑤"></a>

All [\<frequency\>](#frequency-value) units are [compatible](#compatible-units), and [hz](#Hz) is their [canonical unit](#canonical-unit).

<a id="ref-for-ascii-case-insensitive④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Units are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) and serialize as lower case, for example 1Hz serializes as 1hz.

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
<a id="ref-for-px⑤"></a>

Dots per [px](#px) unit.

<a id="ref-for-resolution-value③"></a>

<a id="ref-for-in③"></a>

<a id="ref-for-cm②"></a>

<a id="ref-for-px⑥"></a>

<a id="ref-for-propdef-image-resolution"></a>

The [\<resolution\>](#resolution-value) unit represents the size of a single "dot" in a graphical representation by indicating how many of these dots fit in a CSS [in](#in), [cm](#cm), or [px](#px). For uses, see e.g. the resolution media query in [\[MEDIAQ\]](#biblio-mediaq) or the [image-resolution](https://www.w3.org/TR/css-images-4/#propdef-image-resolution) property defined in [\[CSS3-IMAGES\]](#biblio-css3-images).

<a id="ref-for-resolution-value④"></a>

<a id="ref-for-compatible-units⑨"></a>

<a id="ref-for-dppx①"></a>

<a id="ref-for-canonical-unit⑥"></a>

All [\<resolution\>](#resolution-value) units are [compatible](#compatible-units), and [dppx](#dppx) is their [canonical unit](#canonical-unit).

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
## <a id="defined-elsewhere"></a>8.  Data Types Defined Elsewhere

Some data types are defined in their own modules. This example talks about some of the most common ones used across several specifications.

<a id="ref-for-typedef-color"></a>

### <a id="colors"></a>8.1.  Colors: the [\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color) type

<a id="ref-for-typedef-color①"></a>

The [\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color) data type is defined in [\[CSS-COLOR-4\]](#biblio-css-color-4). UAs must interpret <a id="ref-for-typedef-color②"></a>\<color\> as defined therein.

<a id="ref-for-typedef-color③"></a>

#### <a id="combine-colors"></a>8.1.1.  Combination of [\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color)

<a id="ref-for-interpolation①②"></a>

<a id="ref-for-typedef-color④"></a>

[Interpolation](#interpolation) of [\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color) is defined in [CSS Color 4 §13 Interpolation](https://www.w3.org/TR/css-color-4/#interpolation). Interpolation is done between premultiplied colors, as defined in [CSS Color 4 §13.2 Interpolating with alpha](https://www.w3.org/TR/css-color-4/#interpolation-alpha).

<a id="ref-for-addition①③"></a>

<a id="ref-for-typedef-color⑤"></a>

<a id="ref-for-number-value①④"></a>

[Addition](#addition) of [\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color) is likewise defined as the independent <a id="ref-for-addition①④"></a>addition of each component as a [\<number\>](#number-value) in premultiplied space.

<a id="ref-for-typedef-image"></a>

### <a id="images"></a>8.2.  Images: the [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) type

<a id="ref-for-typedef-image①"></a>

<a id="ref-for-url-value⑦"></a>

The [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) data type is defined in [\[CSS3-IMAGES\]](#biblio-css3-images). UAs that support CSS Images Level 3 or its successor must interpret <a id="ref-for-typedef-image②"></a>\<image\> as defined therein. UAs that do not yet support CSS Images Level 3 must interpret <a id="ref-for-typedef-image③"></a>\<image\> as [\<url\>](#url-value).

<a id="ref-for-typedef-image④"></a>

#### <a id="combine-images"></a>8.2.1.  Combination of [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)

<a id="ref-for-typedef-image⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Interpolation of [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) is defined in [CSS Images 3 §6 Interpolation](https://www.w3.org/TR/css-images-3/#interpolation).

<a id="ref-for-not-additive①"></a>

Images are [not additive](#not-additive).

<a id="ref-for-typedef-position"></a>

### <a id="position"></a>8.3.  2D Positioning: the [\<position\>](#typedef-position) type

<a id="ref-for-typedef-position①"></a>

<a id="ref-for-propdef-background-position"></a>

The <a id="typedef-position"></a>[\<position\>](#typedef-position) value specifies the position of a object area (e.g. background image) inside a positioning area (e.g. background positioning area). It is interpreted as specified for [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position). [\[CSS3-BACKGROUND\]](#biblio-css3-background)

<a id="ref-for-typedef-position②"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-comb-any"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-typedef-length-percentage①"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-typedef-length-percentage②"></a>

<a id="ref-for-mult-opt④"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-typedef-length-percentage④"></a>

```text
<position> = [
  [ left | center | right ] || [ top | center | bottom ]
|
  [ left | center | right | <length-percentage> ]
  [ top | center | bottom | <length-percentage> ]?
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

<a id="ref-for-length-value①⑤"></a>

<a id="ref-for-percentage-value②③"></a>

<a id="ref-for-typedef-position③"></a>

When specified in a grammar alongside other keywords, [\<length\>](#length-value)s, or [\<percentage\>](#percentage-value)s, [\<position\>](#typedef-position) is <em>greedily</em> parsed; it consumes as many components as possible.

<a id="ref-for-propdef-transform-origin"></a>

<a id="ref-for-typedef-position④"></a>

<a id="ref-for-length-value①⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-aa45b932"></a> For example, [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin) defines a 3D position as (effectively) ''[\<position\>](#typedef-position) [\<length\>](#length-value)?''. A value such as left 50px will be parsed as a 2-value <a id="ref-for-typedef-position⑤"></a>\<position\>, with an omitted z-component; on the other hand, a value such as top 50px will be parsed as a single-value <a id="ref-for-typedef-position⑥"></a>\<position\> followed by a <a id="ref-for-length-value①⑦"></a>\<length\>.

<a id="ref-for-typedef-position⑦"></a>

#### <a id="combine-positions"></a>8.3.1.  Combination of [\<position\>](#typedef-position)

<a id="ref-for-interpolation①③"></a>

<a id="ref-for-typedef-position⑧"></a>

<a id="ref-for-typedef-length-percentage⑤"></a>

[Interpolation](#interpolation) of [\<position\>](#typedef-position) is defined as the independent interpolation of each component (x, y) normalized as an offset from the top left corner as a [\<length-percentage\>](#typedef-length-percentage).

<a id="ref-for-addition①⑤"></a>

<a id="ref-for-typedef-position⑨"></a>

<a id="ref-for-typedef-length-percentage⑥"></a>

[Addition](#addition) of [\<position\>](#typedef-position) is likewise defined as the independent <a id="ref-for-addition①⑥"></a>addition each component (x, y) normalized as an offset from the top left corner as a [\<length-percentage\>](#typedef-length-percentage).

## <a id="functional-notations"></a>9.  Functional Notations

<a id="ref-for-typedef-function-token"></a>

A <a id="functional-notation"></a>functional notation is a type of component value that can represent more complex types or invoke special processing. The syntax starts with the name of the function immediately followed by a left parenthesis (i.e. a [\<function-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-function-token)) followed by the argument(s) to the notation followed by a right parenthesis. [White space](https://www.w3.org/TR/css-syntax/#whitespace) is allowed, but optional, immediately inside the parentheses. Functions can take multiple arguments, which are formatted similarly to a CSS property value.

<a id="ref-for-functional-notation②"></a>

<a id="ref-for-funcdef-rgba"></a>

Some legacy [functional notations](#functional-notation), such as [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba), use commas unnecessarily, but generally commas are only used to separate items in a list, or pieces of a grammar that would be ambiguous otherwise. If a comma is used to separate arguments, [white space](https://www.w3.org/TR/css-syntax/#whitespace) is optional before and after the comma.

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

The [math functions](#math-function) are defined in [§ 11 Mathematical Expressions](#math).

<a id="ref-for-funcdef-toggle①"></a>

### <a id="toggle-notation"></a>9.1.  Toggling Between Values: [toggle()](#funcdef-toggle)

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
<a id="ref-for-funcdef-toggle②"></a>

The syntax of the [toggle()](#funcdef-toggle) expression is:

<a id="ref-for-typedef-toggle-value"></a>

```text
toggle( <toggle-value># )
```
<a id="ref-for-funcdef-toggle③"></a>

where <a id="typedef-toggle-value"></a>\<toggle-value\> is any CSS value that is valid where the expression is placed, and that doesn’t contain any top-level commas. If any of the values inside are not valid, then the entire [toggle()](#funcdef-toggle) expression is invalid. The <a id="ref-for-funcdef-toggle④"></a>toggle() expression may be used as the value of any property, but must be the only component in that property’s value.

<a id="ref-for-funcdef-toggle⑤"></a>

<a id="ref-for-funcdef-attr①"></a>

<a id="ref-for-funcdef-calc④"></a>

The [toggle()](#funcdef-toggle) notation is not allowed to be nested; nor may it contain [attr()](#funcdef-attr) or [calc()](#funcdef-calc) notations. Declarations containing such constructs are invalid.

<a id="ref-for-funcdef-toggle⑥"></a>

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
<a id="ref-for-funcdef-toggle⑦"></a>

<a id="ref-for-inherited-value"></a>

To determine the computed value of [toggle()](#funcdef-toggle), first evaluate each argument as if it were the sole value of the property in which <a id="ref-for-funcdef-toggle⑧"></a>toggle() is placed to determine the computed value that each represents, called <var>C<sub>n</sub></var> for the <var>n</var>-th argument to <a id="ref-for-funcdef-toggle⑨"></a>toggle(). Then, compare the property’s [inherited value](https://www.w3.org/TR/css-cascade-5/#inherited-value) with each <var>C<sub>n</sub></var>. For the earliest <var>C<sub>n</sub></var> that matches the <a id="ref-for-inherited-value①"></a>inherited value, the computed value of <a id="ref-for-funcdef-toggle①⓪"></a>toggle() is <var>C<sub>n+1</sub></var>. If the match was the last argument in the list, or there was no match, the computed value of <a id="ref-for-funcdef-toggle①①"></a>toggle() is the computed value that the first argument represents.

<a id="ref-for-funcdef-toggle①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that repeating values in a [toggle()](#funcdef-toggle) short-circuits the list. For example toggle(1em, 2em, 1em, 4em) will be equivalent to toggle(1em, 2em).

<a id="ref-for-funcdef-toggle①③"></a>

<a id="ref-for-valdef-all-inherit④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That [toggle()](#funcdef-toggle) explicitly looks at the computed value of the parent, so it works even on non-inherited properties. This is similar to the [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit) keyword, which works even on non-inherited properties.

<a id="ref-for-propdef-background-position②"></a>

<a id="ref-for-propdef-background-position③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That the [computed value](https://www.w3.org/TR/CSS21/cascade.html#computed-value) of a property is an abstract set of values, not a particular serialization [\[CSS21\]](#biblio-css21), so comparison between computed values should always be unambiguous and have the expected result. For example, a Level 2 [background-position](https://www.w3.org/TR/CSS2/colors.html#propdef-background-position) computed value is just two offsets, each represented as an absolute length or a percentage, so the declarations [background-position: top center](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) and <a id="ref-for-propdef-background-position④"></a>background-position: 50% 0% produce identical computed values. If the "Computed Value" line of a property definition seems to define something ambiguous or overly strict, please [provide feedback](#status) so we can fix it.

<a id="ref-for-funcdef-toggle①④"></a>

<a id="ref-for-shorthand-property"></a>

If [toggle()](#funcdef-toggle) is used on a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property), it sets each of its longhands to a <a id="ref-for-funcdef-toggle①⑤"></a>toggle() value with arguments corresponding to what the longhand would have received had each of the original <a id="ref-for-funcdef-toggle①⑥"></a>toggle() arguments been the sole value of the <a id="ref-for-shorthand-property①"></a>shorthand.

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

<a id="ref-for-funcdef-attr②"></a>

## <a id="attr-notation"></a>10.  Attribute References: the [attr()](#funcdef-attr) function

<a id="ref-for-concept-attribute"></a>

<a id="ref-for-concept-element"></a>

<a id="ref-for-funcdef-var①"></a>

<a id="ref-for-custom-property②"></a>

The <a id="funcdef-attr"></a>attr() function substitutes the value of an [attribute](https://dom.spec.whatwg.org/#concept-attribute) on an [element](https://dom.spec.whatwg.org/#concept-element) into a property, similar to how the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function substitutes a [custom property](https://www.w3.org/TR/css-variables-1/#custom-property) value into a function.

<a id="ref-for-typedef-wq-name"></a>

<a id="ref-for-typedef-attr-type"></a>

<a id="ref-for-mult-opt⑤"></a>

<a id="ref-for-comb-comma②"></a>

<a id="ref-for-typedef-declaration-value"></a>

<a id="ref-for-mult-opt⑥"></a>

<a id="typedef-attr-type"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-comb-one②③"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-comb-one②⑤"></a>

<a id="ref-for-comb-one②⑥"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-typedef-dimension-unit"></a>

```text
attr() = attr( <wq-name> <attr-type>? , <declaration-value>?)

<attr-type> = string | url | ident | color | number | percentage |
              length | angle | time | frequency | flex | <dimension-unit>
```
<a id="ref-for-typedef-delim-token"></a>

<a id="ref-for-length-value①⑧"></a>

<a id="ref-for-angle-value①⓪"></a>

<a id="ref-for-time-value⑦"></a>

<a id="ref-for-frequency-value⑦"></a>

<a id="ref-for-typedef-flex"></a>

<a id="ref-for-px⑨"></a>

<a id="ref-for-ms①"></a>

The <a id="typedef-dimension-unit"></a>\<dimension-unit\> production matches a literal "%" character (that is, a [\<delim-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-delim-token) with a value of "%") or an ident whose value is any of the CSS units for [\<length\>](#length-value), [\<angle\>](#angle-value), [\<time\>](#time-value), [\<frequency\>](#frequency-value), or [\<flex\>](https://www.w3.org/TR/css-grid-2/#typedef-flex) values (such as [px](#px) or [ms](#ms)).

<a id="ref-for-funcdef-attr③"></a>

The arguments of [attr()](#funcdef-attr) are:

<a id="ref-for-typedef-wq-name①"></a>

[\<wq-name\>](https://www.w3.org/TR/selectors-4/#typedef-wq-name)

Gives the name of the attribute being referenced.

<a id="ref-for-attribute-selector"></a>

<a id="ref-for-typedef-wq-name②"></a>

If no namespace is specified (just an identifier is given, like attr(foo)), the null namespace is implied. (This is usually what’s desired, as namespaced attributes are rare. In particular, HTML and SVG do not contain namespaced attributes.) As with [attribute selectors](https://www.w3.org/TR/selectors-4/#attribute-selector), the case-sensitivity of [\<wq-name\>](https://www.w3.org/TR/selectors-4/#typedef-wq-name) depends on the document language.

<a id="ref-for-funcdef-attr④"></a>

<a id="ref-for-originating-element"></a>

If [attr()](#funcdef-attr) is used in a property applied to an element, it references the attribute of the given name on that element; if applied to a pseudo-element, the attribute is looked up on the pseudo-element’s [originating element](https://www.w3.org/TR/selectors-4/#originating-element).

<a id="ref-for-typedef-attr-type①"></a>

[\<attr-type\>](#typedef-attr-type)

<a id="ref-for-funcdef-attr⑤"></a>

Specifies what kind of CSS value the attribute’s value will be interpreted into (the [attr()](#funcdef-attr)’s <a id="attr-substitution-value"></a>substitution value) and what, if any, special parsing will be done to the value.

The possible values and their behavior are defined in [§ 10.1 attr() Types](#attr-types).

<a id="ref-for-valdef-attr-string"></a>

Defaults to [string](#valdef-attr-string) if omitted.

<a id="ref-for-typedef-declaration-value①"></a>

[\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value)

<a id="ref-for-funcdef-attr⑥"></a>

Specifies a fallback value for the [attr()](#funcdef-attr), which will be substituted instead of the attribute’s value if the attribute is missing or fails to parse as the specified type.

<a id="ref-for-typedef-attr-type②"></a>

<a id="ref-for-valdef-attr-string①"></a>

<a id="ref-for-guaranteed-invalid-value"></a>

If the [\<attr-type\>](#typedef-attr-type) argument is [string](#valdef-attr-string), defaults to the empty string if omitted; otherwise, defaults to the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-1/#guaranteed-invalid-value) if omitted.

<a id="ref-for-funcdef-attr⑦"></a>

<a id="ref-for-substitute-an-attr"></a>

If a property contains one or more [attr()](#funcdef-attr) functions, and those functions are syntactically valid, the entire property’s grammar must be assumed to be valid at parse time. It is only syntax-checked at computed-value time, after <a id="ref-for-funcdef-attr⑧"></a>attr() functions have been [substituted](#substitute-an-attr).

<a id="ref-for-px①⓪"></a>

<a id="ref-for-propdef-width⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the default value need not be of the type given. For instance, if the type required of the attribute by the author is [px](#px), the default could still be auto, like in [width: attr(size px, auto);](https://www.w3.org/TR/css-sizing-3/#propdef-width).

<a id="ref-for-funcdef-attr⑨"></a>

### <a id="attr-types"></a>10.1.  [attr()](#funcdef-attr) Types

<a id="ref-for-funcdef-attr①⓪"></a>

<a id="ref-for-typedef-attr-type③"></a>

The behavior of the [attr()](#funcdef-attr) function depends partially on the value of the [\<attr-type\>](#typedef-attr-type) argument:

<a id="valdef-attr-string"></a>string

<a id="ref-for-attr-substitution-value"></a>

The [substitution value](#attr-substitution-value) is a CSS string, whose value is the literal value of the attribute. (No CSS parsing or "cleanup" of the value is performed.)

No value triggers fallback.

<a id="valdef-attr-url"></a>url

<a id="ref-for-attr-substitution-value①"></a>

<a id="ref-for-url-value⑧"></a>

The [substitution value](#attr-substitution-value) is a CSS [\<url\>](#url-value) value, whose url is the literal value of the attribute. (No CSS parsing or "cleanup" of the value is performed.)

<a id="ref-for-funcdef-url①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If [url()](#funcdef-url) was syntactically capable of containing functions, attr(foo url) would be identical to url(attr(foo string)).

No value triggers fallback.

<a id="valdef-attr-ident"></a>ident

<a id="ref-for-attr-substitution-value②"></a>

<a id="ref-for-identifier-value①④"></a>

<a id="ref-for-strip-leading-and-trailing-ascii-whitespace"></a>

The [substitution value](#attr-substitution-value) is a CSS [\<custom-ident\>](#identifier-value), whose value is the literal value of the attribute, with [leading and trailing ASCII whitespace stripped](https://infra.spec.whatwg.org/#strip-leading-and-trailing-ascii-whitespace). (No CSS parsing of the value is performed.)

<a id="ref-for-attr-substitution-value③"></a>

If the attribute value, after trimming, is the empty string, there is instead no [substitution value](#attr-substitution-value).

<a id="ref-for-identifier-value①⑤"></a>

<a id="ref-for-css-wide-keywords①"></a>

<a id="ref-for-attr-substitution-value④"></a>

If the [\<custom-ident\>](#identifier-value)’s value is a [CSS-wide keyword](#css-wide-keywords) or default, there is instead no [substitution value](#attr-substitution-value).

<a id="valdef-attr-color"></a>color

<a id="ref-for-parse-a-component-value"></a>

<a id="ref-for-typedef-hex-color"></a>

<a id="ref-for-named-color"></a>

<a id="ref-for-attr-substitution-value⑤"></a>

<a id="ref-for-typedef-color⑥"></a>

[Parse a component value](https://www.w3.org/TR/css-syntax-3/#parse-a-component-value) from the attribute’s value. If the result is a [\<hex-color\>](https://www.w3.org/TR/css-color-4/#typedef-hex-color) or a [named color](https://www.w3.org/TR/css-color-4/#named-color) ident, the [substitution value](#attr-substitution-value) is that result as a [\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color).

<a id="ref-for-attr-substitution-value⑥"></a>

Otherwise there is no [substitution value](#attr-substitution-value).

<a id="valdef-attr-number"></a>number

<a id="ref-for-parse-a-component-value①"></a>

<a id="ref-for-typedef-number-token③"></a>

<a id="ref-for-attr-substitution-value⑦"></a>

[Parse a component value](https://www.w3.org/TR/css-syntax-3/#parse-a-component-value) from the attribute’s value. If the result is a [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token), the result is the [substitution value](#attr-substitution-value).

<a id="ref-for-attr-substitution-value⑧"></a>

Otherwise, there is no [substitution value](#attr-substitution-value).

<a id="valdef-attr-percentage"></a>percentage

<a id="ref-for-parse-a-component-value②"></a>

<a id="ref-for-typedef-percentage-token①"></a>

<a id="ref-for-attr-substitution-value⑨"></a>

[Parse a component value](https://www.w3.org/TR/css-syntax-3/#parse-a-component-value) from the attribute’s value. If the result is a [\<percentage-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-percentage-token), the result is the [substitution value](#attr-substitution-value).

<a id="ref-for-attr-substitution-value①⓪"></a>

Otherwise, there is no [substitution value](#attr-substitution-value).

<a id="valdef-attr-length"></a>length

<a id="valdef-attr-angle"></a>angle

<a id="valdef-attr-time"></a>time

<a id="valdef-attr-frequency"></a>frequency

<a id="valdef-attr-flex"></a>flex

<a id="ref-for-parse-a-component-value③"></a>

<a id="ref-for-typedef-dimension-token②"></a>

<a id="ref-for-attr-substitution-value①①"></a>

[Parse a component value](https://www.w3.org/TR/css-syntax-3/#parse-a-component-value) from the attribute’s value. If the result is a [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token) whose unit matches the given type, the result is the [substitution value](#attr-substitution-value).

<a id="ref-for-attr-substitution-value①②"></a>

Otherwise, there is no [substitution value](#attr-substitution-value).

<a id="ref-for-typedef-dimension-unit①"></a>

<a id="valdef-attr-dimension-unit"></a>[\<dimension-unit\>](#typedef-dimension-unit)

<a id="ref-for-parse-a-component-value④"></a>

<a id="ref-for-typedef-number-token④"></a>

<a id="ref-for-attr-substitution-value①③"></a>

[Parse a component value](https://www.w3.org/TR/css-syntax-3/#parse-a-component-value) from the attribute’s value. If the result is a [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token), the [substitution value](#attr-substitution-value) is a dimension with the result’s value, and the given unit.

<a id="ref-for-attr-substitution-value①④"></a>

Otherwise, there is no [substitution value](#attr-substitution-value).

<a id="ref-for-math-function②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-168aea69"></a> Do we want to allow [math functions](#math-function) as attr values for all the numeric types? And color functions for "color"? I think we do, but I’d have to check the contents to make sure they don’t contain further reference functions; `foo="rgb(var(--red), 0, 0)"` needs to be illegal for attr(foo color).

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
<a id="ref-for-funcdef-attr①①"></a>

### <a id="attr-substitution"></a>10.2.  [attr()](#funcdef-attr) Substitution

<a id="ref-for-substitute-a-var"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9dd81cfe"></a> attr() and var() substitute at the same time, so I should probably rewrite [substitute a var()](https://www.w3.org/TR/css-variables-1/#substitute-a-var) to be more generally about "substitute a reference" and just use that for both of these functions.

<a id="ref-for-funcdef-attr①②"></a>

<a id="ref-for-substitute-an-attr①"></a>

<a id="ref-for-invalid-at-computed-value-time"></a>

[attr()](#funcdef-attr) functions are [substituted](#substitute-an-attr) at computed-value time. If a declaration, once all <a id="ref-for-funcdef-attr①③"></a>attr() functions are substituted in, does not match its declared grammar, the declaration is [invalid at computed-value time](https://drafts.csswg.org/css-variables-1/#invalid-at-computed-value-time).

<a id="ref-for-funcdef-attr①④"></a>

To <a id="substitute-an-attr"></a>substitute an [attr()](#funcdef-attr):

1.  <a id="ref-for-funcdef-attr①⑤"></a>

    <a id="ref-for-attr-substitution-value①⑤"></a>

    If the [attr()](#funcdef-attr) function has a [substitution value](#attr-substitution-value), replace the <a id="ref-for-funcdef-attr①⑥"></a>attr() function by the <a id="ref-for-attr-substitution-value①⑥"></a>substitution value.

2.  <a id="ref-for-funcdef-attr①⑦"></a>

    <a id="ref-for-funcdef-var②"></a>

    <a id="ref-for-substitute-an-attr②"></a>

    Otherwise, if the [attr()](#funcdef-attr) function has a fallback value as its last argument, replace the <a id="ref-for-funcdef-attr①⑧"></a>attr() function by the fallback value. If there are any [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) or <a id="ref-for-funcdef-attr①⑨"></a>attr() references in the fallback, [substitute](#substitute-an-attr) them as well.

3.  <a id="ref-for-funcdef-attr②⓪"></a>

    <a id="ref-for-invalid-at-computed-value-time①"></a>

    Otherwise, the property containing the [attr()](#funcdef-attr) function is [invalid at computed-value time](https://drafts.csswg.org/css-variables-1/#invalid-at-computed-value-time).

## <a id="math"></a>11.  Mathematical Expressions<a id="calc-notation"></a>

<a id="ref-for-funcdef-calc⑤"></a>

<a id="ref-for-funcdef-clamp"></a>

<a id="ref-for-funcdef-sin"></a>

The <a id="math-function"></a>math functions ([calc()](#funcdef-calc), [clamp()](#funcdef-clamp), [sin()](#funcdef-sin), and others defined in this chapter) allow numeric CSS values to be written as mathematical expressions.

<a id="ref-for-math-function③"></a>

A [math function](#math-function) represents a numeric value, one of:

- <a id="ref-for-length-value①⑨"></a>

  [\<length\>](#length-value),

- <a id="ref-for-frequency-value⑧"></a>

  [\<frequency\>](#frequency-value),

- <a id="ref-for-angle-value①①"></a>

  [\<angle\>](#angle-value),

- <a id="ref-for-time-value⑧"></a>

  [\<time\>](#time-value),

- <a id="ref-for-typedef-flex①"></a>

  [\<flex\>](https://www.w3.org/TR/css-grid-2/#typedef-flex),

- <a id="ref-for-resolution-value⑤"></a>

  [\<resolution\>](#resolution-value),

- <a id="ref-for-percentage-value②④"></a>

  [\<percentage\>](#percentage-value),

- <a id="ref-for-number-value①⑤"></a>

  [\<number\>](#number-value),

- <a id="ref-for-integer-value①⓪"></a>

  [\<integer\>](#integer-value)

<a id="ref-for-typedef-length-percentage⑦"></a>

...or the [\<length-percentage\>](#typedef-length-percentage)/etc mixed types, and can be used wherever such a value would be valid.

<a id="ref-for-funcdef-calc⑥"></a>

### <a id="calc-func"></a>11.1.  Basic Arithmetic: [calc()](#funcdef-calc)

<a id="ref-for-math-function④"></a>

The <a id="funcdef-calc"></a>calc() function is a [math function](#math-function) that allows basic arithmetic to be performed on numerical values, using addition (+), subtraction (-), multiplication (\*), division (/), and parentheses.

<a id="ref-for-funcdef-calc⑦"></a>

<a id="ref-for-typedef-calc-sum"></a>

<a id="ref-for-calc-calculation"></a>

A [calc()](#funcdef-calc) function contains a single <a id="calc-calculation"></a>calculation, which is a sequence of values interspersed with operators, and possibly grouped by parentheses (matching the [\<calc-sum\>](#typedef-calc-sum) grammar), which represents the result of evaluating the expression using standard operator precedence rules (\* and / bind tighter than + and -, and operators are otherwise evaluated left-to-right). The <a id="ref-for-funcdef-calc⑧"></a>calc() function represents the result of its contained [calculation](#calc-calculation).

<a id="ref-for-calc-calculation①"></a>

<a id="ref-for-math-function⑤"></a>

<a id="ref-for-funcdef-attr②①"></a>

<a id="ref-for-length-value②⓪"></a>

Components of a [calculation](#calc-calculation) can be literal values (such as 5px), other [math functions](#math-function), or other expressions, such as [attr()](#funcdef-attr), that evaluate to a valid argument type (like [\<length\>](#length-value)).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1daf921b"></a>
>
> <a id="ref-for-math-function⑥"></a>
>
> <a id="ref-for-propdef-box-sizing"></a>
>
> [Math functions](#math-function) can be used to combine value that use different units. In this example the author wants the <em>margin box</em> of each section to take up 1/3 of the space, so they start with 100%/3, then subtract the element’s borders and margins. ([box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing) can automatically achieve this effect for borders and padding, but a <a id="ref-for-math-function⑦"></a>math function is needed if you want to include margins.)
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
> <a id="ref-for-math-function⑧"></a>
>
> <a id="ref-for-propdef-font-size⑥"></a>
>
> [Math functions](#math-function) can also be useful just to express values in a more natural, readable fashion, rather than as an obscure decimal. For example, the following sets the [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) so that exactly 35em fits within the viewport, ensuring that roughly the same amount of text always fills the screen no matter the screen size.
>
> ```css
> :root {
>   font-size: calc(100vw / 35);
> }
> ```
>
> <a id="ref-for-propdef-font-size⑦"></a>
>
> Functionality-wise, this is identical to just writing [font-size: 2.857vw](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), but then the intent (that 35em fills the viewport) is much less clear to someone reading the code; the later reader will have to reverse the math themselves to figure out that 2.857 is meant to approximate 100/35.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-364b1cfd"></a>
>
> Standard mathematical precedence rules for the operators apply: calc(2 + 3 \* 4) is equal to 14, not 20.
>
> Parentheses can be used to manipulate precedence: calc((2 + 3) \* 4) is instead equal to 20.
>
> <a id="ref-for-funcdef-calc⑨"></a>
>
> <a id="ref-for-funcdef-var③"></a>
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
> <a id="ref-for-propdef-width⑧"></a>
>
> <a id="ref-for-funcdef-calc①⓪"></a>
>
> Altho --ar <em>could</em> have been written as simply --ar: (16 / 9);, --w is used both on its own (in [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)) and as a [calc()](#funcdef-calc) component (in --h), so it has to be written as a full <a id="ref-for-funcdef-calc①①"></a>calc() function itself.

<a id="ref-for-funcdef-min"></a>

<a id="ref-for-funcdef-max"></a>

<a id="ref-for-funcdef-clamp①"></a>

### <a id="comp-func"></a>11.2.  Comparison Functions: [min()](#funcdef-min), [max()](#funcdef-max), and [clamp()](#funcdef-clamp)

<a id="ref-for-funcdef-min①"></a>

<a id="ref-for-funcdef-max①"></a>

<a id="ref-for-funcdef-clamp②"></a>

<a id="ref-for-calc-calculation②"></a>

The comparison functions of [min()](#funcdef-min), [max()](#funcdef-max), and [clamp()](#funcdef-clamp) compare multiple [calculations](#calc-calculation) and represent the value of one of them.

<a id="ref-for-calc-calculation③"></a>

The <a id="funcdef-min"></a>min() or <a id="funcdef-max"></a>max() functions contain one or more comma-separated [calculations](#calc-calculation), and represent the smallest (most negative) or largest (most positive) of them, respectively.

<a id="ref-for-calc-calculation④"></a>

The <a id="funcdef-clamp"></a>clamp() function takes three [calculations](#calc-calculation)—a minimum value, a central value, and a maximum value—and represents its central calculation, clamped according to its min and max calculations, favoring the min calculation if it conflicts with the max. (That is, given clamp(MIN, VAL, MAX), it represents exactly the same value as max(MIN, min(VAL, MAX))).

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
> <a id="ref-for-propdef-font-size⑧"></a>
>
> [min()](#funcdef-min), [max()](#funcdef-max), and [clamp()](#funcdef-clamp) can be used to make sure a value doesn’t exceed a "safe" limit: For example, "responsive type" that sets [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) with viewport units might still want a minimum size to ensure readability:
>
> ```css
> .type {
>   /* Set font-size to 10x the average of vw and vh,
>      but don’t let it go below 12px. */
>   font-size: max(10 * (1vw + 1vh) / 2, 12px);
> }
> ```
>
> <a id="ref-for-funcdef-calc①②"></a>
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
> <a id="example-3f7f1368"></a> An occasional point of confusion when using [min()](#funcdef-min)/[max()](#funcdef-max) is that you use <a id="ref-for-funcdef-max④"></a>max() to impose a minimum value on something (that is, properties like [min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width) effectively use <a id="ref-for-funcdef-max⑤"></a>max()), and <a id="ref-for-funcdef-min④"></a>min() to impose a maximum value on something; it’s easy to accidentally reach for the opposite function and try to use <a id="ref-for-funcdef-min⑤"></a>min() to add a minimum size. Using [clamp()](#funcdef-clamp) can make the code read more naturally, as the value is nestled between its minimum and maximum:
>
> ```css
> .type {
>   /* Force the font-size to stay between 12px and 100px */
>   font-size: clamp(12px, 10 * (1vw + 1vh) / 2, 100px);
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
> clamp(min(MIN, MAX), VAL, MAX). If you want to avoid repeating the MAX calculation, you can just reverse the nesting of functions that [clamp()](#funcdef-clamp) is defined against—min(MAX, max(MIN, VAL)).
>
> To have MAX and MIN "swap" when they’re in the wrong order:  
> clamp(min(MIN, MAX), VAL, max(MIN, MAX)). Unfortunately, there’s no easy way to do this without repeating the MIN and MAX terms.

<a id="ref-for-funcdef-round"></a>

<a id="ref-for-funcdef-mod"></a>

<a id="ref-for-funcdef-rem"></a>

### <a id="round-func"></a>11.3.  Stepped Value Functions: [round()](#funcdef-round), [mod()](#funcdef-mod), and [rem()](#funcdef-rem)

<a id="ref-for-funcdef-round①"></a>

<a id="ref-for-funcdef-mod①"></a>

<a id="ref-for-funcdef-rem①"></a>

The stepped-value functions, [round()](#funcdef-round), [mod()](#funcdef-mod), and [rem()](#funcdef-rem), all transform a given value according to another "step value", in different ways.

<a id="ref-for-typedef-rounding-strategy"></a>

<a id="ref-for-calc-calculation⑤"></a>

<a id="ref-for-number-value①⑥"></a>

<a id="ref-for-typedef-dimension②"></a>

<a id="ref-for-percentage-value②⑤"></a>

<a id="ref-for-determine-the-type-of-a-calculation"></a>

<a id="ref-for-cssnumericvalue-type"></a>

The <a id="funcdef-round"></a>round([\<rounding-strategy\>](#typedef-rounding-strategy)?, A, B) function contains an optional rounding strategy, and two [calculations](#calc-calculation) A and B, and returns the value of A, rounded according to the rounding strategy, to the nearest integer multiple of B either above or below A. The argument <a id="ref-for-calc-calculation⑥"></a>calculations can resolve to any [\<number\>](#number-value), [\<dimension\>](#typedef-dimension), or [\<percentage\>](#percentage-value), but must have the <em>same</em> [type](#determine-the-type-of-a-calculation), or else the function is invalid; the result will have the same [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) as the arguments.

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

If [\<rounding-strategy\>](#typedef-rounding-strategy) is omitted, it defaults to [nearest](#valdef-rounding-strategy-nearest).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-1a929fd0"></a>[w3c/csswg-drafts/5689](https://github.com/w3c/csswg-drafts/issues/5689)[\[cssom\] Is \`round\` special in CSSOM?](https://github.com/w3c/csswg-drafts/issues/5689)

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

<a id="ref-for-calc-calculation⑦"></a>

<a id="ref-for-number-value①⑦"></a>

<a id="ref-for-typedef-dimension③"></a>

<a id="ref-for-percentage-value②⑥"></a>

<a id="ref-for-determine-the-type-of-a-calculation①"></a>

<a id="ref-for-cssnumericvalue-type①"></a>

The modulus functions <a id="funcdef-mod"></a>mod(A, B) and <a id="funcdef-rem"></a>rem(A, B) similarly contain two [calculations](#calc-calculation) A and B, and return the difference between A and the nearest integer multiple of B either above or below A. The argument <a id="ref-for-calc-calculation⑧"></a>calculations can resolve to any [\<number\>](#number-value), [\<dimension\>](#typedef-dimension), or [\<percentage\>](#percentage-value), but must have the <em>same</em> [type](#determine-the-type-of-a-calculation), or else the function is invalid; the result will have the same [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) as the arguments.

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

#### <a id="round-infinities"></a>11.3.1.  Argument Ranges

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

In mod(A, B) only, if B is infinite and A is non-zero and has opposite sign to B, the result is NaN.

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

### <a id="trig-funcs"></a>11.4.  Trigonometric Functions: [sin()](#funcdef-sin), [cos()](#funcdef-cos), [tan()](#funcdef-tan), [asin()](#funcdef-asin), [acos()](#funcdef-acos), [atan()](#funcdef-atan), and [atan2()](#funcdef-atan2)

<a id="ref-for-funcdef-sin②"></a>

<a id="ref-for-funcdef-cos①"></a>

<a id="ref-for-funcdef-tan①"></a>

<a id="ref-for-funcdef-asin①"></a>

<a id="ref-for-funcdef-acos①"></a>

<a id="ref-for-funcdef-atan①"></a>

<a id="ref-for-funcdef-atan2①"></a>

The trigonometric functions—[sin()](#funcdef-sin), [cos()](#funcdef-cos), [tan()](#funcdef-tan), [asin()](#funcdef-asin), [acos()](#funcdef-acos), [atan()](#funcdef-atan), and [atan2()](#funcdef-atan2)—compute the various basic trigonometric relationships.

<a id="ref-for-calc-calculation⑨"></a>

<a id="ref-for-number-value①⑧"></a>

<a id="ref-for-angle-value①②"></a>

<a id="ref-for-funcdef-sin③"></a>

<a id="ref-for-funcdef-cos②"></a>

<a id="ref-for-funcdef-tan②"></a>

<a id="ref-for-math-function⑨"></a>

The <a id="funcdef-sin"></a>sin(A), <a id="funcdef-cos"></a>cos(A), and <a id="funcdef-tan"></a>tan(A) functions all contain a single [calculation](#calc-calculation) which must resolve to either a [\<number\>](#number-value) or an [\<angle\>](#angle-value), and compute their corresponding function by interpreting the result of their argument as radians. (That is, sin(45deg), sin(.125turn), and sin(3.14159 / 4) all represent the same value, approximately .707.) They all represent a <a id="ref-for-number-value①⑨"></a>\<number\>; [sin()](#funcdef-sin) and [cos()](#funcdef-cos) will always return a number between −1 and 1, while [tan()](#funcdef-tan) can return any number between −∞ and +∞. (See [§ 11.9 Type Checking](#calc-type-checking) for details on how [math functions](#math-function) handle ∞.)

<a id="ref-for-calc-calculation①⓪"></a>

<a id="ref-for-number-value②⓪"></a>

<a id="ref-for-angle-value①③"></a>

<a id="ref-for-funcdef-asin②"></a>

<a id="ref-for-funcdef-acos②"></a>

<a id="ref-for-funcdef-atan②"></a>

The <a id="funcdef-asin"></a>asin(A), <a id="funcdef-acos"></a>acos(A), and <a id="funcdef-atan"></a>atan(A) functions are the "arc" or "inverse" trigonometric functions, representing the inverse function to their corresponding "normal" trig functions. All of them contain a single [calculation](#calc-calculation) which must resolve to a [\<number\>](#number-value), and compute their corresponding function, interpreting their result as a number of radians, representing an [\<angle\>](#angle-value). The angle returned by [asin()](#funcdef-asin) must be normalized to the range \[-90deg, 90deg\]; the angle returned by [acos()](#funcdef-acos) to the range \[0deg, 180deg\]; and the angle returned by [atan()](#funcdef-atan) to the range \[-90deg, 90deg\].

<a id="ref-for-calc-calculation①①"></a>

<a id="ref-for-number-value②①"></a>

<a id="ref-for-typedef-dimension④"></a>

<a id="ref-for-percentage-value②⑦"></a>

<a id="ref-for-determine-the-type-of-a-calculation②"></a>

<a id="ref-for-angle-value①④"></a>

The <a id="funcdef-atan2"></a>atan2(A, B) function contains two comma-separated [calculations](#calc-calculation), A and B. A and B can resolve to any [\<number\>](#number-value), [\<dimension\>](#typedef-dimension), or [\<percentage\>](#percentage-value), but must have the <em>same</em> [type](#determine-the-type-of-a-calculation), or else the function is invalid. The function returns the [\<angle\>](#angle-value) between the positive X-axis and the point (B,A). The returned angle must be normalized to the interval (-180deg, 180deg\] (that is, greater than -180deg, and less than or equal to 180deg).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: atan2(Y, X) is <em>generally</em> equivalent to atan(Y / X), but it gives a better answer when the point in question may include negative components. atan2(1, -1), corresponding to the point (-1, 1), returns 135deg, distinct from atan2(-1, 1), corresponding to the point (1, -1), which returns -45deg. In contrast, atan(1 / -1) and atan(-1 / 1) both return-45deg, because the internal calculation resolves to -1 for both.

#### <a id="trig-infinities"></a>11.4.1.  Argument Ranges

<a id="ref-for-math-function①⓪"></a>

In sin(A), cos(A), or tan(A), if A is infinite, the result is NaN. (See [§ 11.9 Type Checking](#calc-type-checking) for details on how [math functions](#math-function) handle NaN.)

In sin(A) or tan(A), if A is 0⁻, the result is 0⁻.

In tan(A), if A is one of the asymptote values (such as 90deg, 270deg, etc), the result must be +∞ for 90deg and all values a multiple of 360deg from that (such as -270deg or 450deg), and −∞ for -90deg and all values a multiple of 360deg from that (such as -450deg or 270deg).

<a id="ref-for-deg②"></a>

<a id="ref-for-grad①"></a>

<a id="ref-for-rad①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is only relevant for units that can exactly represent the asymptotic values, such as [deg](#deg) or [grad](#grad). [rad](#rad) cannot, and so whether the result is a very large negative or positive value can depend on rounding and precise details of how numbers are internally stored. It’s recommended you don’t depend on this behavior if using such units.

In asin(A) or acos(A), if A is less than -1 or greater than 1, the result is NaN.

In acos(A), if A is exactly 1, the result is 0.

In asin(A) or atan(A), if A is 0⁻, the result is 0⁻.

In atan(A), if A is +∞, the result is 90deg; if A is −∞, the result is -90deg.

In atan2(Y, X), the following table gives the results for all unusual argument combinations:

**Table 5**

Representation note: complete merged-header paths are explicit; inherited span values are repeated where they apply. Native HTML span and row-header accessibility semantics are not available in GFM.

**Source header labels**

X

−∞ / -finite / 0⁻ / 0⁺ / +finite / +∞

| Y | X / −∞ | X / -finite | X / 0⁻ | X / 0⁺ | X / +finite | X / +∞ |
| --- | --- | --- | --- | --- | --- | --- |
| **−∞** | -135deg | -90deg | -90deg | -90deg | -90deg | -45deg |
| **-finite** | -180deg | (normal) | -90deg | -90deg | (normal) | 0⁻deg |
| **0⁻** | -180deg | -180deg | -180deg | 0⁻deg | 0⁻deg | 0⁻deg |
| **0⁺** | 180deg | 180deg | 180deg | 0⁺deg | 0⁺deg | 0⁺deg |
| **+finite** | 180deg | (normal) | 90deg | 90deg | (normal) | 0⁺deg |
| **+∞** | 135deg | 90deg | 90deg | 90deg | 90deg | 45deg |

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: All of these behaviors are intended to match the "standard" definitions of these functions as implemented by most programming languages, in particular as implemented in JS.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The behavior of tan(90deg), while not constrained by JS behavior (because the JS function’s input is in radians, and one cannot perfectly express a value of π/2 in JS numbers), is defined so that roundtripping of values works; tan(atan(infinity)) yields +∞, tan(atan(-infinity)) yields −∞, atan(tan(90deg)) yields 90deg, and atan(tan(-90deg)) yields -90deg.

<a id="ref-for-funcdef-pow"></a>

<a id="ref-for-funcdef-sqrt"></a>

<a id="ref-for-funcdef-hypot"></a>

<a id="ref-for-funcdef-log"></a>

<a id="ref-for-funcdef-exp"></a>

### <a id="exponent-funcs"></a>11.5.  Exponential Functions: [pow()](#funcdef-pow), [sqrt()](#funcdef-sqrt), [hypot()](#funcdef-hypot), [log()](#funcdef-log), [exp()](#funcdef-exp)

<a id="ref-for-funcdef-pow①"></a>

<a id="ref-for-funcdef-sqrt①"></a>

<a id="ref-for-funcdef-hypot①"></a>

<a id="ref-for-funcdef-log①"></a>

<a id="ref-for-funcdef-exp①"></a>

The exponential functions—[pow()](#funcdef-pow), [sqrt()](#funcdef-sqrt), [hypot()](#funcdef-hypot), [log()](#funcdef-log), and [exp()](#funcdef-exp)—compute various exponential functions with their arguments.

<a id="ref-for-calc-calculation①②"></a>

<a id="ref-for-number-value②②"></a>

The <a id="funcdef-pow"></a>pow(A, B) function contains two comma-separated [calculations](#calc-calculation) A and B, both of which must resolve to [\<number\>](#number-value)s, and returns the result of raising A to the power of B, returning the value as a <a id="ref-for-number-value②③"></a>\<number\>.

<a id="ref-for-calc-calculation①③"></a>

<a id="ref-for-number-value②④"></a>

<a id="ref-for-funcdef-sqrt②"></a>

The <a id="funcdef-sqrt"></a>sqrt(A) function contains a single [calculation](#calc-calculation) which must resolve to a [\<number\>](#number-value), and returns the square root of the value as a <a id="ref-for-number-value②⑤"></a>\<number\>. (sqrt(X) and pow(X, .5) are basically equivalent, differing only in some error-handling; [sqrt()](#funcdef-sqrt) is a common enough function that it is provided as a convenience.)

<a id="ref-for-calc-calculation①④"></a>

<a id="ref-for-number-value②⑥"></a>

<a id="ref-for-typedef-dimension⑤"></a>

<a id="ref-for-percentage-value②⑧"></a>

<a id="ref-for-determine-the-type-of-a-calculation③"></a>

<a id="ref-for-cssnumericvalue-type②"></a>

The <a id="funcdef-hypot"></a>hypot(A, …) function contains one or more comma-separated [calculations](#calc-calculation), and returns the length of an N-dimensional vector with components equal to each of the <a id="ref-for-calc-calculation①⑤"></a>calculations. (That is, the square root of the sum of the squares of its arguments.) The argument <a id="ref-for-calc-calculation①⑥"></a>calculations can resolve to any [\<number\>](#number-value), [\<dimension\>](#typedef-dimension), or [\<percentage\>](#percentage-value), but must have the <em>same</em> [type](#determine-the-type-of-a-calculation), or else the function is invalid; the result will have the same [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) as the arguments.

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
> <a id="ref-for-cssnumericvalue-type③"></a>
>
> <a id="ref-for-cssnumericvalue-match"></a>
>
> <a id="ref-for-length-value②①"></a>
>
> First, numerical precision. For a [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) to [match](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-match) a CSS production like [\<length\>](#length-value), it needs to have a single unit with its exponent set to exactly 1. Theoretically, expressions like pow(pow(30px, 3), 1/3) should result in exactly that: the inner pow(30px, 3) would resolve to a value of 27000 with a <a id="ref-for-cssnumericvalue-type④"></a>type of «\[ "length" → 3 \]» (aka <a id="ref-for-length-value②②"></a>\<length\>³), and then the pow(X, 1/3) would cube-root the value back down to 30 and multiply the exponent by 1/3, giving «\[ "length" → 1 \]», which <a id="ref-for-cssnumericvalue-match①"></a>matches <a id="ref-for-length-value②③"></a>\<length\>.
>
> <a id="ref-for-math-function①①"></a>
>
> In the realm of pure mathematics, that’s guaranteed to work out; in the real-world of computers using binary floating-point arithmetic, in some cases the powers might not exactly cancel out, leaving you with an invalid [math function](#math-function) for confusing, hard-to-track-down reasons. (For a JS example, evaluate `Math.pow(Math.pow(30, 10/3), .1+.1+.1)`; the result is not exactly 30, because `.1+.1+.1` is not exactly 3/10. Instead, `(10/3) * (.1 + .1 + .1)` is <em>slightly greater</em> than 1.)
>
> <a id="ref-for-length-value②④"></a>
>
> Requiring authors to cast their value down into a number, do all the math on the raw number, then finally send it back to the desired unit, while inconvenient, ensures that numerical precision won’t bite anyone: calc(pow(pow(30px / 1px, 3), 1/3) \* 1px) is guaranteed to resolve to a [\<length\>](#length-value), with a value that, if not exactly 30, is at least very close to 30, even if numerical precision actually prevents the powers from exactly canceling.
>
> <a id="ref-for-canonical-unit⑦"></a>
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

<a id="ref-for-calc-calculation①⑦"></a>

<a id="ref-for-number-value②⑦"></a>

The <a id="funcdef-log"></a>log(A, B?) function contains one or two [calculations](#calc-calculation) (representing the value to be logarithmed, and the base of the logarithm, defaulting to e), which must resolve to [\<number\>](#number-value)s, and returns the logarithm base B of the value A, as a <a id="ref-for-number-value②⑧"></a>\<number\>.

<a id="ref-for-calc-calculation①⑧"></a>

<a id="ref-for-number-value②⑨"></a>

The <a id="funcdef-exp"></a>exp(A) function contains one [calculation](#calc-calculation) which must resolve to a [\<number\>](#number-value), and returns the same value as pow(e, A) as a <a id="ref-for-number-value③⓪"></a>\<number\>.

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

#### <a id="exponent-infinities"></a>11.5.1.  Argument Ranges

In pow(A, B), if A is negative and finite, and B is finite, B must be an integer, or else the result is NaN.

If A or B are infinite or 0, the following tables give the results:

**Table 6**

Representation note: complete merged-header paths are explicit; inherited span values are repeated where they apply. Native HTML span and row-header accessibility semantics are not available in GFM.

| B / A | A is −∞ | A is 0⁻ | A is 0⁺ | A is +∞ |
| --- | --- | --- | --- | --- |
| **B is −finite** | 0⁻ if B is an odd integer, 0⁺ otherwise | −∞ if B is an odd integer, +∞ otherwise | +∞ | 0⁺ |
| **B is 0** | always 1 | always 1 | always 1 | always 1 |
| **B is +finite** | −∞ if B is an odd integer, +∞ otherwise | 0⁻ if B is an odd integer, 0⁺ otherwise | 0⁺ | +∞ |



|                     | A is \< -1   | A is -1       | -1 \< A \< 1 | A is 1        | A is \> 1    |
|---------------------|--------------|---------------|--------------|---------------|--------------|
| <strong>B is +∞ &#xA;      </strong> | result is +∞ | result is NaN | result is 0⁺ | result is NaN | result is +∞ |
| <strong>B is −∞ &#xA;      </strong> | result is 0⁺ | result is NaN | result is +∞ | result is NaN | result is 0⁺ |



In sqrt(A), if A is +∞, the result is +∞. If A is 0⁻, the result is 0⁻. If A is less than 0, the result is NaN.

In hypot(A, …), if any of the inputs are infinite, the result is +∞.

In log(A, B), if B is 1 or negative, <strong data-conversion-semantic="note">Note:</strong> B values <em>between</em> 0 and 1, or greater than 1, are valid. the result is NaN. If A is negative, the result is NaN. If A is 0⁺ or 0⁻, the result is −∞. If A is 1, the result is 0⁺. If A is +∞, the result is +∞.

In exp(A), if A is +∞, the result is +∞. If A is −∞, the result is 0⁺.

<a id="ref-for-math-function①②"></a>

(See [§ 11.9 Type Checking](#calc-type-checking) for details on how [math functions](#math-function) handle NaN and infinities.)

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
> <a id="ref-for-calc-calculation①⑨"></a>
>
> Because this is an error corner case, JS isn’t consistent on the matter, and NaN recognition/handling of [calculations](#calc-calculation) is likely done at a higher CSS level rather than in the internal math functions anyway, consistency in CSS was chosen to be more important, so all functions were defined to have "infectious" NaN.

<a id="ref-for-funcdef-abs"></a>

<a id="ref-for-funcdef-sign"></a>

### <a id="sign-funcs"></a>11.6.  Sign-Related Functions: [abs()](#funcdef-abs), [sign()](#funcdef-sign)

<a id="ref-for-funcdef-abs①"></a>

<a id="ref-for-funcdef-sign①"></a>

The sign-related functions—[abs()](#funcdef-abs) and [sign()](#funcdef-sign)—compute various functions related to the sign of their argument.

<a id="ref-for-calc-calculation②⓪"></a>

<a id="ref-for-cssnumericvalue-type⑤"></a>

The <a id="funcdef-abs"></a>abs(A) function contains one [calculation](#calc-calculation) A, and returns the absolute value of A, as the same [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) as the input: if A’s numeric value is positive or 0⁺, just A again; otherwise -1 \* A.

<a id="ref-for-calc-calculation②①"></a>

The <a id="funcdef-sign"></a>sign(A) function contains one [calculation](#calc-calculation) A, and returns -1 if A’s numeric value is negative, +1 if A’s numeric value is positive, 0⁺ if A’s numeric value is 0⁺, and 0⁻ if A’s numeric value is 0⁻.

<a id="ref-for-propdef-background-position⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Both of these functions operate on the fully simplified/resolved form of their arguments, which may give unintuitive results at first glance. In particular, an expression like 10% might be positive <em>or</em> negative once it’s resolved, depending on what value it’s resolved against. For example, in [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) positive percentages resolve to a negative length, and vice versa, if the background image is larger than the background area. Thus sign(10%) might return 1 <em>or</em> -1, depending on how the percentage is resolved! (Or even 0, if it’s resolved against a zero length.)

<a id="ref-for-valdef-calc-e"></a>

<a id="ref-for-valdef-calc-pi"></a>

### <a id="calc-constants"></a>11.7.  Numeric Constants: [e](#valdef-calc-e), [pi](#valdef-calc-pi)

While the trigonometric and exponential functions handle many complex numeric operations, some reasonable calculations must be put together more manually, and many times these include well-known constants, such as <i>e</i> and <i>π</i>.

Rather than require authors to manually type out several digits of these constants, a few of them are provided directly:

<a id="valdef-calc-e"></a>e is the base of the natural logarithm, approximately equal to 2.7182818284590452354.

<a id="valdef-calc-pi"></a>pi is the ratio of a circle’s circumference to its diameter, approximately equal to 3.1415926535897932.

<a id="ref-for-propdef-animation-name①"></a>

<a id="ref-for-propdef-line-height⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These keywords are only usable within a calculation, such as calc(pow(e, pi) - pi), or min(pi, 5, e). If used outside of a calculation, they’re treated like any other keyword: [animation-name: pi;](https://www.w3.org/TR/css-animations-1/#propdef-animation-name) refers to an animation named "pi"; [line-height: e;](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) is invalid (<em>not</em> similar to <a id="ref-for-propdef-line-height①⓪"></a>line-height: 2.7, but <a id="ref-for-propdef-line-height①①"></a>line-height: calc(e); is).

<a id="ref-for-valdef-calc-infinity"></a>

<a id="ref-for-valdef-calc--infinity"></a>

<a id="ref-for-valdef-calc-nan"></a>

#### <a id="calc-error-constants"></a>11.7.1.  Degenerate Numeric Constants: [infinity](#valdef-calc-infinity), [-infinity](#valdef-calc--infinity), [NaN](#valdef-calc-nan)

<a id="ref-for-calc-calculation②②"></a>

When a [calculation](#calc-calculation) or a subtree of a <a id="ref-for-calc-calculation②③"></a>calculation becomes infinite or NaN, representing it with a numeric value is no longer possible. To aid in serialization of these degenerate values, the additional math constants <a id="valdef-calc-infinity"></a>infinity (with the value +∞), <a id="valdef-calc--infinity"></a>-infinity (with the value −∞), and <a id="valdef-calc-nan"></a>NaN (with the value NaN) are defined.

<a id="ref-for-ascii-case-insensitive⑤"></a>

<a id="ref-for-valdef-calc-nan①"></a>

As usual for CSS keywords, these are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive). <strong data-conversion-semantic="note">Note:</strong> Thus, calc(InFiNiTy) is perfectly valid. However, [NaN](#valdef-calc-nan) must be serialized with this canonical casing.

<a id="ref-for-valdef-calc-e①"></a>

<a id="ref-for-valdef-calc-pi①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While not <em>technically</em> numbers, these keywords act as numeric values, similar to [e](#valdef-calc-e) and [pi](#valdef-calc-pi). Thus to get an infinite length, for example, requires an expression like calc(infinity \* 1px).

<a id="ref-for-valdef-calc-infinity①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These constants are defined <em>mostly</em> to make serialization of infinite/NaN values simpler and more obvious, but <em>can</em> be used to indicate a "largest possible value", since an infinite value gets clamped to the allowed range. It’s rare for this to be reasonable, but when it is, using [infinity](#valdef-calc-infinity) is clearer in its intent than just putting an enormous number in one’s stylesheet.

### <a id="calc-syntax"></a>11.8.  Syntax

<a id="ref-for-math-function①③"></a>

The syntax of a [math function](#math-function) is:

<a id="ref-for-funcdef-calc①③"></a>

<a id="ref-for-typedef-calc-sum①"></a>

<a id="ref-for-funcdef-min⑦"></a>

<a id="ref-for-typedef-calc-sum②"></a>

<a id="ref-for-mult-comma④"></a>

<a id="ref-for-funcdef-max⑦"></a>

<a id="ref-for-typedef-calc-sum③"></a>

<a id="ref-for-mult-comma①"></a>

<a id="ref-for-funcdef-clamp⑧"></a>

<a id="ref-for-typedef-calc-sum④"></a>

<a id="ref-for-mult-comma②"></a>

<a id="ref-for-funcdef-round③"></a>

<a id="ref-for-typedef-rounding-strategy⑤"></a>

<a id="ref-for-mult-opt⑦"></a>

<a id="ref-for-comb-comma③"></a>

<a id="ref-for-typedef-calc-sum⑤"></a>

<a id="ref-for-comb-comma④"></a>

<a id="ref-for-typedef-calc-sum⑥"></a>

<a id="ref-for-funcdef-mod⑥"></a>

<a id="ref-for-typedef-calc-sum⑦"></a>

<a id="ref-for-comb-comma⑤"></a>

<a id="ref-for-typedef-calc-sum⑧"></a>

<a id="ref-for-funcdef-rem⑨"></a>

<a id="ref-for-typedef-calc-sum⑨"></a>

<a id="ref-for-comb-comma⑥"></a>

<a id="ref-for-typedef-calc-sum①⓪"></a>

<a id="ref-for-funcdef-sin④"></a>

<a id="ref-for-typedef-calc-sum①①"></a>

<a id="ref-for-funcdef-cos③"></a>

<a id="ref-for-typedef-calc-sum①②"></a>

<a id="ref-for-funcdef-tan③"></a>

<a id="ref-for-typedef-calc-sum①③"></a>

<a id="ref-for-funcdef-asin③"></a>

<a id="ref-for-typedef-calc-sum①④"></a>

<a id="ref-for-funcdef-acos③"></a>

<a id="ref-for-typedef-calc-sum①⑤"></a>

<a id="ref-for-funcdef-atan③"></a>

<a id="ref-for-typedef-calc-sum①⑥"></a>

<a id="ref-for-funcdef-atan2②"></a>

<a id="ref-for-typedef-calc-sum①⑦"></a>

<a id="ref-for-comb-comma⑦"></a>

<a id="ref-for-typedef-calc-sum①⑧"></a>

<a id="ref-for-funcdef-pow④"></a>

<a id="ref-for-typedef-calc-sum①⑨"></a>

<a id="ref-for-comb-comma⑧"></a>

<a id="ref-for-typedef-calc-sum②⓪"></a>

<a id="ref-for-funcdef-sqrt④"></a>

<a id="ref-for-typedef-calc-sum②①"></a>

<a id="ref-for-funcdef-hypot⑥"></a>

<a id="ref-for-typedef-calc-sum②②"></a>

<a id="ref-for-mult-comma③"></a>

<a id="ref-for-funcdef-log③"></a>

<a id="ref-for-typedef-calc-sum②③"></a>

<a id="ref-for-comb-comma⑨"></a>

<a id="ref-for-typedef-calc-sum②④"></a>

<a id="ref-for-mult-opt⑧"></a>

<a id="ref-for-funcdef-exp②"></a>

<a id="ref-for-typedef-calc-sum②⑤"></a>

<a id="ref-for-funcdef-abs②"></a>

<a id="ref-for-typedef-calc-sum②⑥"></a>

<a id="ref-for-funcdef-sign②"></a>

<a id="ref-for-typedef-calc-sum②⑦"></a>

<a id="typedef-calc-sum"></a>

<a id="ref-for-typedef-calc-product"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-typedef-calc-product①"></a>

<a id="ref-for-mult-zero-plus②"></a>

<a id="typedef-calc-product"></a>

<a id="ref-for-typedef-calc-value"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-typedef-calc-value①"></a>

<a id="ref-for-mult-zero-plus③"></a>

<a id="typedef-calc-value"></a>

<a id="ref-for-number-value③①"></a>

<a id="ref-for-comb-one③②"></a>

<a id="ref-for-typedef-dimension⑥"></a>

<a id="ref-for-comb-one③③"></a>

<a id="ref-for-percentage-value②⑨"></a>

<a id="ref-for-comb-one③④"></a>

<a id="ref-for-typedef-calc-constant"></a>

<a id="ref-for-comb-one③⑤"></a>

<a id="ref-for-typedef-calc-sum②⑧"></a>

<a id="typedef-calc-constant"></a>

<a id="ref-for-comb-one③⑥"></a>

<a id="ref-for-comb-one③⑦"></a>

<a id="ref-for-comb-one③⑧"></a>

<a id="ref-for-comb-one③⑨"></a>

```text
<calc()>  = calc( <calc-sum> )
<min()>   = min( <calc-sum># )
<max()>   = max( <calc-sum># )
<clamp()> = clamp( <calc-sum>#{3} )
<round()> = round( <rounding-strategy>?, <calc-sum>, <calc-sum> )
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
               <calc-constant> | ( <calc-sum> )
<calc-constant> = e | pi | infinity | -infinity | NaN
```
<a id="ref-for-whitespace"></a>

In addition, [whitespace](https://www.w3.org/TR/css-syntax-3/#whitespace) is required on both sides of the + and - operators. (The \* and / operators can be used without white space around them.)

<a id="ref-for-typedef-calc-sum②⑨"></a>

Several of the math functions above have additional constraints on what their [\<calc-sum\>](#typedef-calc-sum) arguments can contain. Check the definitions of the individual functions for details.

<a id="ref-for-calc-calculation②④"></a>

<a id="ref-for-typedef-calc-value②"></a>

UAs must support [calculations](#calc-calculation) of at least 20 [\<calc-value\>](#typedef-calc-value) terms. If a <a id="ref-for-calc-calculation②⑤"></a>calculation contains more than the supported number of terms, it must be treated as if it were invalid.

### <a id="calc-type-checking"></a>11.9.  Type Checking

<a id="ref-for-math-function①④"></a>

<a id="ref-for-length-value②⑤"></a>

<a id="ref-for-number-value③②"></a>

<a id="ref-for-calc-calculation②⑥"></a>

A [math function](#math-function) can be many possible types, such as [\<length\>](#length-value), [\<number\>](#number-value), etc., depending on the [calculations](#calc-calculation) it contains, as defined below. It can be used anywhere a value of that type is allowed.

<a id="ref-for-propdef-width⑨"></a>

<a id="ref-for-length-value②⑥"></a>

<a id="ref-for-math-function①⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-40312766"></a> For example, the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property accepts [\<length\>](#length-value) values, so a [math function](#math-function) that resolves to a <a id="ref-for-length-value②⑦"></a>\<length\>, such as calc(5px + 1em), can be used in <a id="ref-for-propdef-width①⓪"></a>width.

<a id="ref-for-math-function①⑥"></a>

<a id="ref-for-number-value③③"></a>

<a id="ref-for-integer-value①①"></a>

Additionally, [math functions](#math-function) that resolve to [\<number\>](#number-value) can be used in any place that only accepts [\<integer\>](#integer-value). (It gets rounded to the nearest integer, as specified in [§ 11.12 Range Checking](#calc-range).)

Operators form sub-expressions, which gain types based on their arguments.

<a id="ref-for-length-value②⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In previous versions of this specification, multiplication and division were limited in what arguments they could take, to avoid producing more complex intermediate results (such as 1px \* 1em, which is [\<length\>](#length-value)²) and to make division-by-zero detectable at parse time. This version now relaxes those restrictions.

<a id="ref-for-calc-calculation②⑦"></a>

To <a id="determine-the-type-of-a-calculation"></a>determine the type of a [calculation](#calc-calculation):

- <a id="ref-for-cssnumericvalue-add-two-types"></a>

  <a id="ref-for-calc-calculation②⑧"></a>

  <a id="ref-for-cssnumericvalue-type⑥"></a>

  At a + or - sub-expression, attempt to [add the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-add-two-types) of the left and right arguments. If this returns failure, the entire [calculation’s](#calc-calculation) type is failure. Otherwise, the sub-expression’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is the returned type.

- <a id="ref-for-cssnumericvalue-multiply-two-types"></a>

  <a id="ref-for-cssnumericvalue-type⑦"></a>

  At a \* sub-expression, [multiply the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-multiply-two-types) of the left and right arguments. The sub-expression’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is the returned result.

- <a id="ref-for-cssnumericvalue-type⑧"></a>

  <a id="ref-for-cssnumericvalue-invert-a-type"></a>

  At a / sub-expression, let <var>left type</var> be the result of finding the [types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of its left argument, and <var>right type</var> be the result of finding the <a id="ref-for-cssnumericvalue-type⑨"></a>types of its right argument and then [inverting](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-invert-a-type) it.

  <a id="ref-for-cssnumericvalue-type①⓪"></a>

  <a id="ref-for-cssnumericvalue-multiply-two-types①"></a>

  The sub-expression’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is the result of [multiplying](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-multiply-two-types) the <var>left type</var> and <var>right type</var>.

- <a id="ref-for-cssnumericvalue-type①①"></a>

  Anything else is a terminal value, whose [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is determined based on its CSS type:

  <a id="ref-for-number-value③④"></a>

  [\<number\>](#number-value)

  <a id="ref-for-integer-value①②"></a>

  [\<integer\>](#integer-value)

  <a id="ref-for-cssnumericvalue-type①②"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ \]» (empty map)

  <a id="ref-for-length-value②⑨"></a>

  [\<length\>](#length-value)

  <a id="ref-for-cssnumericvalue-type①③"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "length" → 1 \]»

  <a id="ref-for-angle-value①⑤"></a>

  [\<angle\>](#angle-value)

  <a id="ref-for-cssnumericvalue-type①④"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "angle" → 1 \]»

  <a id="ref-for-time-value⑨"></a>

  [\<time\>](#time-value)

  <a id="ref-for-cssnumericvalue-type①⑤"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "time" → 1 \]»

  <a id="ref-for-frequency-value⑨"></a>

  [\<frequency\>](#frequency-value)

  <a id="ref-for-cssnumericvalue-type①⑥"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "frequency" → 1 \]»

  <a id="ref-for-resolution-value⑥"></a>

  [\<resolution\>](#resolution-value)

  <a id="ref-for-cssnumericvalue-type①⑦"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "resolution" → 1 \]»

  <a id="ref-for-typedef-flex②"></a>

  [\<flex\>](https://www.w3.org/TR/css-grid-2/#typedef-flex)

  <a id="ref-for-cssnumericvalue-type①⑧"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "flex" → 1 \]»

  <a id="ref-for-typedef-calc-constant①"></a>

  [\<calc-constant\>](#typedef-calc-constant)

  <a id="ref-for-cssnumericvalue-type①⑨"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ \]» (empty map)

  <a id="ref-for-percentage-value③⓪"></a>

  [\<percentage\>](#percentage-value)

  <a id="ref-for-math-function①⑦"></a>

  <a id="ref-for-calc-calculation②⑨"></a>

  <a id="ref-for-percentage-value③①"></a>

  <a id="ref-for-propdef-width①①"></a>

  <a id="ref-for-length-value③⓪"></a>

  <a id="ref-for-number-value③⑤"></a>

  <a id="ref-for-cssnumericvalue-type②⓪"></a>

  If, in the context in which the [math function](#math-function) containing this [calculation](#calc-calculation) is placed, [\<percentage\>](#percentage-value)s are resolved relative to another type of value (such as in [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), where <a id="ref-for-percentage-value③②"></a>\<percentage\> is resolved against a [\<length\>](#length-value)), and that other type is <em>not</em> [\<number\>](#number-value), the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is determined as the other type.

  <a id="ref-for-cssnumericvalue-type②①"></a>

  Otherwise, the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "percent" → 1 \]».

  anything else

  <a id="ref-for-calc-calculation③⓪"></a>

  The [calculation’s](#calc-calculation) type is failure.

  <a id="ref-for-cssnumericvalue-percent-hint"></a>

  In all cases, the associated [percent hint](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-percent-hint) is null.

<a id="ref-for-math-function①⑧"></a>

<a id="ref-for-cssnumericvalue-type②②"></a>

<a id="ref-for-calc-calculation③①"></a>

[Math functions](#math-function) themselves have [types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type), according to their contained [calculations](#calc-calculation):

- <a id="ref-for-cssnumericvalue-type②③"></a>

  <a id="ref-for-funcdef-calc①④"></a>

  <a id="ref-for-funcdef-abs③"></a>

  <a id="ref-for-calc-calculation③②"></a>

  The [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of a [calc()](#funcdef-calc) or [abs()](#funcdef-abs) expression is the <a id="ref-for-cssnumericvalue-type②④"></a>type of its contained [calculation](#calc-calculation).

- <a id="ref-for-cssnumericvalue-type②⑤"></a>

  <a id="ref-for-funcdef-min⑧"></a>

  <a id="ref-for-funcdef-max⑧"></a>

  <a id="ref-for-funcdef-clamp⑨"></a>

  <a id="ref-for-cssnumericvalue-add-two-types①"></a>

  <a id="ref-for-calc-calculation③③"></a>

  The [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of a [min()](#funcdef-min), [max()](#funcdef-max), or [clamp()](#funcdef-clamp) expression is the result of [adding the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-add-two-types) of its comma-separated [calculations](#calc-calculation).

- <a id="ref-for-cssnumericvalue-type②⑥"></a>

  <a id="ref-for-funcdef-sign③"></a>

  The [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of a [sign()](#funcdef-sign) expression is «\[ "number" → 1 \]».

- <a id="ref-for-cssnumericvalue-type②⑦"></a>

  <a id="ref-for-funcdef-sin⑤"></a>

  <a id="ref-for-funcdef-cos④"></a>

  <a id="ref-for-funcdef-tan④"></a>

  The [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of a [sin()](#funcdef-sin), [cos()](#funcdef-cos), or [tan()](#funcdef-tan) expression is «\[ "number" → 1 \]».

- <a id="ref-for-cssnumericvalue-type②⑧"></a>

  <a id="ref-for-funcdef-asin④"></a>

  <a id="ref-for-funcdef-acos④"></a>

  <a id="ref-for-funcdef-atan④"></a>

  <a id="ref-for-funcdef-atan2③"></a>

  The [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of an [asin()](#funcdef-asin), [acos()](#funcdef-acos), [atan()](#funcdef-atan), or [atan2()](#funcdef-atan2) expression is «\[ "angle" → 1 \]».

- <a id="ref-for-cssnumericvalue-type②⑨"></a>

  <a id="ref-for-funcdef-pow⑤"></a>

  <a id="ref-for-funcdef-sqrt⑤"></a>

  <a id="ref-for-funcdef-log④"></a>

  <a id="ref-for-funcdef-exp③"></a>

  The [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of a [pow()](#funcdef-pow), [sqrt()](#funcdef-sqrt), [log()](#funcdef-log), or [exp()](#funcdef-exp) expression is «\[ "number" → 1 \]».

- <a id="ref-for-cssnumericvalue-type③⓪"></a>

  <a id="ref-for-funcdef-hypot⑦"></a>

  <a id="ref-for-funcdef-round④"></a>

  <a id="ref-for-funcdef-mod⑦"></a>

  <a id="ref-for-funcdef-rem①⓪"></a>

  <a id="ref-for-cssnumericvalue-add-two-types②"></a>

  <a id="ref-for-calc-calculation③④"></a>

  The [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of a [hypot()](#funcdef-hypot), [round()](#funcdef-round), [mod()](#funcdef-mod), or [rem()](#funcdef-rem) expression is the result of [adding the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-add-two-types) of its comma-separated [calculations](#calc-calculation).

<a id="ref-for-cssnumericvalue-type③①"></a>

<a id="ref-for-math-function①⑨"></a>

For each of the above, if the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is failure, the [math function](#math-function) is invalid.

<a id="ref-for-math-function②⓪"></a>

<a id="ref-for-number-value③⑥"></a>

<a id="ref-for-length-value③①"></a>

<a id="ref-for-angle-value①⑥"></a>

<a id="ref-for-time-value①⓪"></a>

<a id="ref-for-frequency-value①⓪"></a>

<a id="ref-for-resolution-value⑦"></a>

<a id="ref-for-typedef-flex③"></a>

<a id="ref-for-percentage-value③③"></a>

<a id="ref-for-cssnumericvalue-type③②"></a>

<a id="ref-for-cssnumericvalue-match②"></a>

A [math function](#math-function) resolves to [\<number\>](#number-value), [\<length\>](#length-value), [\<angle\>](#angle-value), [\<time\>](#time-value), [\<frequency\>](#frequency-value), [\<resolution\>](#resolution-value), [\<flex\>](https://www.w3.org/TR/css-grid-2/#typedef-flex), or [\<percentage\>](#percentage-value) according to which of those productions its [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) [matches](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-match). (These categories are mutually exclusive.) If it can’t <a id="ref-for-cssnumericvalue-match③"></a>match any of these, the <a id="ref-for-math-function②①"></a>math function is invalid.

<a id="ref-for-math-function②②"></a>

Division by zero is possible, which introduces certain complications. [Math functions](#math-function) follow IEEE-754 semantics for these operations:

- Dividing a positive value by zero produces +∞.

- Dividing a negative value by zero produces −∞.

- Adding or subtracting ±∞ to anything produces the appropriate infinity, unless a following rule would define it as producing NaN.

- Multiplying any value by ±∞ produces the appropriate infinity, unless a following rule would define it as producing NaN.

- Dividing any value by ±∞ produces zero, unless a following rule would define it as producing NaN.

- Dividing zero by zero, dividing ±∞ by ±∞, multiplying 0 by ±∞, adding +∞ to −∞ (or the equivalent subtractions) produces NaN.

- Any operation with at least one NaN argument produces NaN.

Additionally, IEEE-754 introduces the concept of "negative zero", which must be tracked within a calculation and between nested calculations:

- <a id="ref-for-math-function②③"></a>

  Negative zero (0⁻) can be produced by a multiplication or division that produces zero with exactly one negative argument (such as -5 \* 0 or 1 / (-infinity)), or by certain argument combinations in the other [math functions](#math-function).

  <a id="ref-for-math-function②④"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: Note that negative zeros don’t escape a [math function](#math-function); as detailed below, they’re "censored" away into an "unsigned" zero.

- 0⁻ + 0⁻ or 0⁻ - 0 produces 0⁻. All other additions or subtractions that would produce a zero produce 0⁺.

- Multiplying or dividing 0⁻ with a positive number (including 0⁺) produces a negative result (either 0⁻ or −∞), while multiplying or dividing 0⁻ with a negative number produces a positive result.

  (In other words, multiplying or dividing with 0⁻ follows standard sign rules.)

- When comparing 0⁺ and 0⁻, 0⁻ is less than 0⁺. For example, min(0⁺, 0⁻) must produce 0⁻, max(0⁺, 0⁻) must produce 0⁺, and clamp(0⁺, 0⁻, 1) must produce 0⁺.

<a id="ref-for-math-function②⑤"></a>

<a id="ref-for-top-level-calculation"></a>

If a <a id="top-level-calculation"></a>top-level calculation (a [math function](#math-function) not nested inside of another <a id="ref-for-math-function②⑥"></a>math function) would produce a value whose numeric part is NaN, it instead act as though the numeric part is +∞. If a [top-level calculation](#top-level-calculation) would produce a value whose numeric part is 0⁻, it instead acts as though the numeric part is the standard "unsigned" zero.

<a id="ref-for-top-level-calculation①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-40b6820d"></a> For example, calc(-5 \* 0) produces an unsigned zero—the calculation resolves to 0⁻, but as it’s a [top-level calculation](#top-level-calculation), it’s then censored to an unsigned zero.
>
> <a id="ref-for-top-level-calculation②"></a>
>
> On the other hand, calc(1 / calc(-5 \* 0)) produces −∞, same as calc(1 / (-5 \* 0))—the inner calc resolves to 0⁻, and as it’s not a [top-level calculation](#top-level-calculation), it passes it up unchanged to the outer calc to produce −∞. If it was censored into an unsigned zero, it would instead produce +∞.

<a id="ref-for-math-function②⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Algebraic simplifications do not affect the validity of a [math function](#math-function) or its resolved type. For example, calc(5px - 5px + 10s) and calc(0 \* 5px + 10s) are both invalid due to the attempt to add a length and a time.

<a id="ref-for-percentage-value③④"></a>

<a id="ref-for-number-value③⑦"></a>

<a id="ref-for-propdef-opacity"></a>

<a id="ref-for-typedef-dimension⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that [\<percentage\>](#percentage-value)s relative to [\<number\>](#number-value)s, such as in [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), are not <em>combinable</em> with those numbers—<a id="ref-for-propdef-opacity①"></a>opacity: calc(.25 + 25%) is invalid. Allowing this causes significant problems with "unit algebra" (allowing multiplication/division of [\<dimension\>](#typedef-dimension)s), and in every case so far, doesn’t provide any new functionality. (For example, <a id="ref-for-propdef-opacity②"></a>opacity: 25% is identical to <a id="ref-for-propdef-opacity③"></a>opacity: .25; it’s just a trivial syntax transform.) You can still perform other operations with them, such as <a id="ref-for-propdef-opacity④"></a>opacity: calc(100% / 3);, which is valid.

<a id="ref-for-typedef-number-token⑤"></a>

<a id="ref-for-number-value③⑧"></a>

<a id="ref-for-integer-value①③"></a>

<a id="ref-for-length-value③②"></a>

<a id="ref-for-math-function②⑧"></a>

<a id="ref-for-propdef-width①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token)s are always interpreted as [\<number\>](#number-value)s or [\<integer\>](#integer-value)s, "unitless 0" [\<length\>](#length-value)s aren’t supported in [math functions](#math-function). That is, [width: calc(0 + 5px);](https://www.w3.org/TR/css-sizing-3/#propdef-width) is invalid, because it’s trying to add a <a id="ref-for-number-value③⑨"></a>\<number\> to a <a id="ref-for-length-value③③"></a>\<length\>, even though both <a id="ref-for-propdef-width①③"></a>width: 0; and <a id="ref-for-propdef-width①④"></a>width: 5px; are valid.

<a id="ref-for-number-value④⓪"></a>

<a id="ref-for-length-value③④"></a>

<a id="ref-for-propdef-line-height①②"></a>

<a id="ref-for-propdef-tab-size"></a>

<a id="ref-for-funcdef-calc①⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Altho there are a few properties in which a bare [\<number\>](#number-value) becomes a [\<length\>](#length-value) at used-value time (specifically, [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) and [tab-size](https://www.w3.org/TR/css-text-3/#propdef-tab-size)), <a id="ref-for-number-value④①"></a>\<number\>s never become "length-like" in [calc()](#funcdef-calc). They always stay as <a id="ref-for-number-value④②"></a>\<number\>s.

<a id="ref-for-length-value③⑤"></a>

<a id="ref-for-number-value④③"></a>

<a id="ref-for-px①①"></a>

<a id="ref-for-math-function②⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In Quirks Mode [\[quirks\]](#biblio-quirks), some properties that would normally only accept [\<length\>](#length-value)s are defined to also accept [\<number\>](#number-value)s, interpreting them as [px](#px) lengths. Like unitless zeroes, this has no effect on the parsing or behavior of [math functions](#math-function), tho a <a id="ref-for-math-function③⓪"></a>math function that resolves to a <a id="ref-for-number-value④④"></a>\<number\> value might become valid in Quirks Mode (and have its result interpreted as a <a id="ref-for-px①②"></a>px length).

### <a id="calc-internal"></a>11.10.  Internal Representation

<a id="ref-for-css-internal-representation"></a>

<a id="ref-for-math-function③①"></a>

<a id="ref-for-calc-calculation③⑤"></a>

The [internal representation](https://drafts.css-houdini.org/css-typed-om-1/#css-internal-representation) of a [math function](#math-function) is a <a id="calculation-tree"></a>calculation tree: a tree where the branch nodes are <a id="calculation-tree-operator-nodes"></a>operator nodes corresponding either to <a id="ref-for-math-function③②"></a>math functions (such as Min, Cos, Sqrt, etc) or to operators in a [calculation](#calc-calculation) (Sum, Product, Negate, and Invert, the <a id="calculation-tree-calc-operator-nodes"></a>calc-operator nodes), and the leaf nodes are either numeric values (such as numbers, dimensions, and percentages) or non-<a id="ref-for-math-function③③"></a>math functions that resolve to a numeric type.

<a id="ref-for-math-function③④"></a>

<a id="ref-for-calculation-tree"></a>

[Math functions](#math-function) are turned into [calculation trees](#calculation-tree) depending on the function:

calc()

<a id="ref-for-css-internal-representation①"></a>

<a id="ref-for-funcdef-calc①⑥"></a>

<a id="ref-for-parse-a-calculation"></a>

The [internal representation](https://drafts.css-houdini.org/css-typed-om-1/#css-internal-representation) of a [calc()](#funcdef-calc) function is the result of [parsing a calculation](#parse-a-calculation) from its argument.

<a id="ref-for-math-function③⑤"></a>

any other [math function](#math-function)

<a id="ref-for-css-internal-representation②"></a>

<a id="ref-for-calculation-tree-operator-nodes"></a>

<a id="ref-for-parse-a-calculation①"></a>

The [internal representation](https://drafts.css-houdini.org/css-typed-om-1/#css-internal-representation) is an [operator node](#calculation-tree-operator-nodes) with the same name as the function, whose children are the result of [parsing a calculation](#parse-a-calculation) from each of the function’s arguments, in the order they appear.

<a id="ref-for-calc-calculation③⑥"></a>

<a id="ref-for-component-value②"></a>

<a id="ref-for-calculation-tree①"></a>

To <a id="parse-a-calculation"></a>parse a calculation, given a [calculation](#calc-calculation) <var>values</var> represented as a list of [component values](https://www.w3.org/TR/css-syntax-3/#component-value), and returning a [calculation tree](#calculation-tree):

1.  <a id="ref-for-typedef-whitespace-token"></a>

    Discard any [\<whitespace-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-whitespace-token)s from <var>values</var>.

2.  <a id="ref-for-typedef-delim-token①"></a>

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

#### <a id="calc-simplification"></a>11.10.1.  Simplification

<a id="ref-for-css-internal-representation④"></a>

<a id="ref-for-math-function③⑦"></a>

[Internal representations](https://drafts.css-houdini.org/css-typed-om-1/#css-internal-representation) of [math functions](#math-function) are eagerly simplified to the extent possible, using standard algebraic simplifications (distributing multiplication over sums, combining similar units, etc.).

To <a id="simplify-a-calculation-tree"></a>simplify a calculation tree <var>root</var>:

1.  If <var>root</var> is a numeric value:

    1.  <a id="ref-for-canonical-unit⑧"></a>

        If <var>root</var> is a percentage that will be resolved against another value, and there is enough information available to resolve it, do so, and express the resulting numeric value in the appropriate [canonical unit](#canonical-unit). Return the value.

    2.  <a id="ref-for-canonical-unit⑨"></a>

        If <var>root</var> is a dimension that is not expressed in its [canonical unit](#canonical-unit), and there is enough information available to convert it to the <a id="ref-for-canonical-unit①⓪"></a>canonical unit, do so, and return the value.

    3.  <a id="ref-for-typedef-calc-constant②"></a>

        If <var>root</var> is a [\<calc-constant\>](#typedef-calc-constant), return its numeric value.

    4.  Otherwise, return <var>root</var>.

2.  If <var>root</var> is any other leaf node (not an operator node):

    1.  <a id="ref-for-canonical-unit①①"></a>

        If there is enough information available to determine its numeric value, return its value, expressed in the value’s [canonical unit](#canonical-unit).

    2.  Otherwise, return <var>root</var>.

3.  <a id="ref-for-calculation-tree-operator-nodes①"></a>

    <a id="ref-for-simplify-a-calculation-tree①"></a>

    At this point, <var>root</var> is an [operator node](#calculation-tree-operator-nodes). [Simplify](#simplify-a-calculation-tree) all the children of <var>root</var>.

4.  <a id="ref-for-calculation-tree-operator-nodes②"></a>

    <a id="ref-for-calculation-tree-calc-operator-nodes"></a>

    <a id="ref-for-canonical-unit①②"></a>

    If <var>root</var> is an [operator node](#calculation-tree-operator-nodes) that’s not one of the [calc-operator nodes](#calculation-tree-calc-operator-nodes), and all of its children are numeric values with enough information to compute the operation <var>root</var> represents, return the result of running <var>root</var>’s operation using its children, expressed in the result’s [canonical unit](#canonical-unit).

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > If a percentage is left at this point, it will <em>usually</em> block simplification of the node, since it needs to be resolved against another value using information not currently available. (Otherwise, it would have been converted to a different value in an earlier step.) This includes operations such as "min", since percentages might resolve against a negative basis, and thus end up with an opposite comparative relationship than the raw percentage value would seem to indicate.
    >
    > <a id="ref-for-propdef-opacity⑤"></a>
    >
    > However, "raw" percentages—ones which do not resolve against another value, such as in [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity)—might not block simplification.

5.  If <var>root</var> is a Min or Max node, attempt to <em>partially</em> simplify it:

    1.  <a id="ref-for-list-iterate"></a>

        [For each](https://infra.spec.whatwg.org/#list-iterate) node <var>child</var> of <var>root</var>’s children:

        If <var>child</var> is a numeric value with enough information to compare magnitudes with another child of the same unit (see note in previous step), and there are other children of <var>root</var> that are numeric values with the same unit, combine all such children with the appropriate operator per <var>root</var>, and replace <var>child</var> with the result, removing all other child nodes involved.

    2.  Return <var>root</var>.

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

9.  If <var>root</var> is a Product node:

    1.  For each of <var>root</var>’s children that are Product nodes, replace them with their children.

    2.  If <var>root</var> has multiple children that are numbers (not percentages or dimensions), remove them and replace them with a single number containing the product of the removed nodes.

    3.  If <var>root</var> contains only two children, one of which is a number (not a percentage or dimension) and the other of which is a Sum whose children are all numeric values, multiply all of the Sum’s children by the number, then return the Sum.

    4.  <a id="ref-for-cssnumericvalue-multiply-two-types②"></a>

        <a id="ref-for-cssnumericvalue-invert-a-type①"></a>

        <a id="ref-for-cssnumericvalue-match④"></a>

        <a id="ref-for-math-function③⑧"></a>

        <a id="ref-for-canonical-unit①③"></a>

        If <var>root</var> contains only numeric values and/or Invert nodes containing numeric values, and [multiplying the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-multiply-two-types) of all the children (noting that the type of an Invert node is the [inverse](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-invert-a-type) of its child’s type) results in a type that [matches](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-match) any of the types that a [math function](#math-function) can resolve to, return the result of multiplying all the values of the children (noting that the value of an Invert node is the reciprocal of its child’s value), expressed in the result’s [canonical unit](#canonical-unit).

    5.  Return <var>root</var>.

### <a id="calc-computed-value"></a>11.11.  Computed Value

<a id="ref-for-computed-value⑦"></a>

<a id="ref-for-math-function③⑨"></a>

<a id="ref-for-calculation-tree②"></a>

<a id="ref-for-simplify-a-calculation-tree②"></a>

<a id="ref-for-em③"></a>

<a id="ref-for-px①③"></a>

The [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a [math function](#math-function) is its [calculation tree](#calculation-tree) [simplified](#simplify-a-calculation-tree), using all the information available at <a id="ref-for-computed-value⑧"></a>computed value time. (Such as the [em](#em) to [px](#px) ratio, how to resolve percentages in some properties, etc.)

<a id="ref-for-math-function④⓪"></a>

Where percentages are not resolved at computed-value time, they are not resolved in [math functions](#math-function), e.g. calc(100% - 100% + 1px) resolves to calc(0% + 1px), not to 1px. If there are special rules for computing percentages in a value (e.g. [the height property](https://www.w3.org/TR/CSS21/visudet.html#the-height-property)), they apply whenever a <a id="ref-for-math-function④①"></a>math function contains percentages.

<a id="ref-for-calculation-tree③"></a>

<a id="ref-for-used-value④"></a>

<a id="ref-for-math-function④②"></a>

The [calculation tree](#calculation-tree) is again simplified at [used value](https://www.w3.org/TR/css-cascade-5/#used-value) time; with <a id="ref-for-used-value⑤"></a>used value time information, a [math function](#math-function) always simplifies down to a single numeric value.

<a id="ref-for-propdef-font-size⑨"></a>

<a id="ref-for-computed-value⑨"></a>

<a id="ref-for-font-relative-length"></a>

<a id="ref-for-propdef-background-position⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-023dad93"></a> For example, whereas [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) computes percentage values at [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) time so that [font-relative length](#font-relative-length) units can be computed, [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) has layout-dependent behavior for percentage values, and thus does not resolve percentages until used-value time.
>
> <a id="ref-for-propdef-background-position⑦"></a>
>
> <a id="ref-for-funcdef-calc①⑦"></a>
>
> <a id="ref-for-propdef-font-size①⓪"></a>
>
> Due to this, [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) computation preserves the percentage in a [calc()](#funcdef-calc) whereas [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) will compute such expressions directly into a length.

<a id="ref-for-valdef-width-auto"></a>

Given the complexities of width and height calculations on table cells and table elements, math expressions mixing both percentages and lengths for widths and heights on table columns, table column groups, table rows, table row groups, and table cells in both auto and fixed layout tables MUST be treated as if [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) had been specified.

### <a id="calc-range"></a>11.12.  Range Checking

<a id="ref-for-math-function④③"></a>

<a id="ref-for-computed-value①⓪"></a>

<a id="ref-for-used-value⑥"></a>

<a id="ref-for-specified-value②"></a>

Parse-time range-checking of values is not performed within [math functions](#math-function), and therefore out-of-range values do not cause the declaration to become invalid. However, the value resulting from an expression must be clamped to the range allowed in the target context. Clamping is performed on [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) to the extent possible, and also on [used values](https://www.w3.org/TR/css-cascade-5/#used-value) if computation was unable to sufficiently simplify the expression to allow range-checking. (Clamping is not performed on [specified values](https://www.w3.org/TR/css-cascade-5/#specified-value).)

<a id="ref-for-funcdef-calc①⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This requires all contexts accepting [calc()](#funcdef-calc) to define their allowable values as a closed (not open) interval.

<a id="ref-for-valdef-calc-infinity②"></a>

<a id="ref-for-propdef-animation-iteration-count"></a>

<a id="ref-for-math-function④④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: By definition, ±∞ are outside the allowed range for any property, and will clamp to the minimum/maximum value allowed. Even for properties that explicitly allow [infinity](#valdef-calc-infinity) as a keyword value, such as [animation-iteration-count](https://www.w3.org/TR/css-animations-1/#propdef-animation-iteration-count), will end up clamping ±∞, as [math functions](#math-function) can’t resolve to keyword values; the <em>numeric</em> part of the property’s syntax still has a minimum/maximum value.

<a id="ref-for-math-function④⑤"></a>

<a id="ref-for-number-value④⑤"></a>

<a id="ref-for-integer-value①④"></a>

<a id="ref-for-computed-value①①"></a>

<a id="ref-for-used-value⑦"></a>

Additionally, if a [math function](#math-function) that resolves to [\<number\>](#number-value) is used somewhere that only accepts [\<integer\>](#integer-value), the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) and [used value](https://www.w3.org/TR/css-cascade-5/#used-value) are rounded to the nearest integer, in the same manner as clamping, above. The rounding method must be the same as is used for animations of integer values.

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
> <a id="ref-for-propdef-width①⑤"></a>
>
> <a id="ref-for-funcdef-calc①⑨"></a>
>
> Note however that [width: -5px](https://www.w3.org/TR/css-sizing-3/#propdef-width) is not equivalent to <a id="ref-for-propdef-width①⑥"></a>width: calc(-5px)! Out-of-range values <em>outside</em> [calc()](#funcdef-calc) are syntactically invalid, and cause the entire declaration to be dropped.

### <a id="calc-serialize"></a>11.13.  Serialization

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f5bc4b00"></a> This section is still [under discussion](https://lists.w3.org/Archives/Member/w3c-css-wg/2016AprJun/0239.html).

To <a id="serialize-a-math-function"></a>serialize a math function <var>fn</var>:

1.  <a id="ref-for-calculation-tree④"></a>

    <a id="ref-for-computed-value①②"></a>

    If the root of the [calculation tree](#calculation-tree) <var>fn</var> represents is a numeric value (number, percentage, or dimension), and the serialization being produced is of a [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) or later, then clamp the value to the range allowed for its context (if necessary), then serialize the value as normal and return the result.

2.  If <var>fn</var> represents an infinite or NaN value:

    1.  <a id="ref-for-string①"></a>

        Let <var>s</var> be the [string](https://infra.spec.whatwg.org/#string) "calc(".

    2.  <a id="ref-for-valdef-calc-infinity③"></a>

        <a id="ref-for-valdef-calc--infinity①"></a>

        <a id="ref-for-valdef-calc-nan②"></a>

        Serialize the keyword [infinity](#valdef-calc-infinity), [-infinity](#valdef-calc--infinity), or [NaN](#valdef-calc-nan), as appropriate to represent the value, and append it to <var>s</var>.

    3.  <a id="ref-for-cssnumericvalue-type③③"></a>

        <a id="ref-for-number-value④⑥"></a>

        <a id="ref-for-canonical-unit①④"></a>

        <a id="ref-for-px①④"></a>

        <a id="ref-for-length-value③⑥"></a>

        If <var>fn</var>’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is anything other than «\[ \]» (empty, representing a [\<number\>](#number-value)), append " \* " to <var>s</var>. Create a numeric value in the [canonical unit](#canonical-unit) for <var>fn</var>’s <a id="ref-for-cssnumericvalue-type③④"></a>type (such as [px](#px) for [\<length\>](#length-value)), with a value of 1. Serialize this numeric value and append it to <var>s</var>.

    4.  Return <var>s</var>.

3.  <a id="ref-for-calculation-tree⑤"></a>

    <a id="ref-for-calculation-tree-calc-operator-nodes①"></a>

    If the [calculation tree’s](#calculation-tree) root node is a numeric value, or a [calc-operator node](#calculation-tree-calc-operator-nodes), let <var>s</var> be a string initially containing "calc(".

    Otherwise, let <var>s</var> be a string initially containing the name of the root node, lowercased (such as "sin" or "max"), followed by a "(" (open parenthesis).

4.  <a id="ref-for-serialize-a-calculation-tree"></a>

    <a id="ref-for-string-concatenate"></a>

    For each child of the root node, [serialize the calculation tree](#serialize-a-calculation-tree). If a result of this serialization starts with a "(" (open parenthesis) and ends with a ")" (close parenthesis), remove those characters from the result. [Concatenate](https://infra.spec.whatwg.org/#string-concatenate) all of the results using ", " (comma followed by space), then append the result to <var>s</var>.

5.  Append ")" (close parenthesis) to <var>s</var>.

6.  Return <var>s</var>.

To <a id="serialize-a-calculation-tree"></a>serialize a calculation tree:

1.  <a id="ref-for-calculation-tree⑥"></a>

    Let <var>root</var> be the root node of the [calculation tree](#calculation-tree).

2.  <a id="ref-for-math-function④⑥"></a>

    If <var>root</var> is a numeric value, or a non-[math function](#math-function), serialize <var>root</var> per the normal rules for it and return the result.

3.  <a id="ref-for-serialize-a-math-function"></a>

    <a id="ref-for-calc-calculation③⑦"></a>

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

4.  <a id="ref-for-ascii-case-insensitive⑥"></a>

    If <var>nodes</var> contains any dimensions, remove them from <var>nodes</var>, sort them by their units, ordered [ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive), and append them to <var>ret</var>.

5.  If <var>nodes</var> still contains any items, append them to <var>ret</var> in the same order.

6.  Return <var>ret</var>.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c3db2475"></a> For example, calc(20px + 30px) would serialize as calc(50px) as a specified value, or as 50px as a computed value.
>
> <a id="ref-for-funcdef-calc②⓪"></a>
>
> A value like calc(20px + 0%) would serialize as calc(0% + 20px), maintaining both terms in the serialized value. (It’s important to maintain zero-valued terms, so the [calc()](#funcdef-calc) doesn’t suddenly "change shape" in the middle of a transition when one of the values happens to have a zero value temporarily. This also removes the need to "pick a unit" when all the terms are zero.)
>
> <a id="ref-for-em④"></a>
>
> <a id="ref-for-funcdef-calc②①"></a>
>
> A value like calc(20px + 2em) would serialize as calc(2em + 20px) as a specified value (maintaining both units as they’re incompatible at specified-value time, but sorting them alphabetically), or as something like 52px as a computed value ([em](#em) values are converted to absolute lengths at computed-value time, so assuming 1em = 16px, they combine into 52px, which then drops the [calc()](#funcdef-calc) wrapper.)

See [\[CSSOM\]](#biblio-cssom) for further information on serialization.

### <a id="combine-math"></a>11.14.  Combination of Math Functions

<a id="ref-for-interpolation①④"></a>

<a id="ref-for-math-function④⑦"></a>

<a id="ref-for-simplify-a-calculation-tree③"></a>

[Interpolation](#interpolation) of [math functions](#math-function), with each other or with numeric values and other numeric-valued functions, is defined as V<sub>result</sub> = calc((1 - p) \* V<sub>a</sub> + p \* V<sub>b</sub>). ([Simplification](#simplify-a-calculation-tree) of the value might then reduce the expression to a smaller, simpler form.)

<a id="ref-for-addition①⑦"></a>

<a id="ref-for-math-function④⑧"></a>

<a id="ref-for-simplify-a-calculation-tree④"></a>

[Addition](#addition) of [math functions](#math-function), with each other or with numeric values and other numeric-valued functions, is defined as V<sub>result</sub> = calc(V<sub>a</sub> + V<sub>b</sub>). ([Simplification](#simplify-a-calculation-tree) of the value might then reduce the expression to a smaller, simpler form.)

## <a id="iana"></a> Appendix A: IANA Considerations

### <a id="about-invalid"></a> Registration for the `about:invalid` URL scheme

This sections defines and registers the `about:invalid` URL, in accordance with the registration procedure defined in [\[RFC6694\]](#biblio-rfc6694).

The official record of this registration can be found at [http&#x3A;&#x2F;&#x2F;www&#x2E;iana&#x2E;org&#x2F;assignments&#x2F;about-uri-tokens&#x2F;about-uri-tokens&#x2E;xhtml](http://www.iana.org/assignments/about-uri-tokens/about-uri-tokens.xhtml)&#x2E;



| Field               | Definition                                                                                                                                                                                                  |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Registered Token &#xA;      </strong> | <code>invalid</code>                                                                                                                                                                                           |
| <strong>Intended Usage &#xA;      </strong> | The <code><c->about</c-><c->:</c->invalid</code> URL references a non-existent document with a generic error condition. It can be used when a URL is necessary, but the default value shouldn’t be resolvable as any type of document. |
| <strong>Contact/Change controller &#xA;      </strong> | CSS WG \<<www-style@w3.org>\> (on behalf of W3C)                                                                                                                                                            |
| <strong>Specification &#xA;      </strong> | [CSS Values and Units Module Level 3](https://www.w3.org/TR/css3-values/)                                                                                                                                   |



## <a id="acknowledgments"></a> Acknowledgments

Firstly, the editors would like to thank all of the contributors to the [previous level](https://www.w3.org/TR/css-values-3/#acknowledgements) of this module.

Secondly, we would like to acknowledge Anthony Frehner, Koji Ishii, and Xidorn Quan for their comments and suggestions, which have improved Level 4.

## <a id="changes"></a> Changes

### <a id="changes-recent"></a> Recent Changes

Changes since [11 November 2020 WD](https://www.w3.org/TR/2020/WD-css-values-4-20201111/) (this is a subset of [Additions Since Level 3](#additions-L3)):

- Updated interpolation of colors to reference [\[CSS-COLOR-4\]](#biblio-css-color-4) instead of [\[CSS-COLOR-3\]](#biblio-css-color-3).

- <a id="ref-for-dynamic-viewport-percentage-units②"></a>

  <a id="ref-for-valdef-length-dvmax"></a>

  <a id="ref-for-valdef-length-dvmin"></a>

  <a id="ref-for-valdef-length-dvb"></a>

  <a id="ref-for-valdef-length-dvi"></a>

  <a id="ref-for-valdef-length-dvw"></a>

  <a id="ref-for-valdef-length-dvh"></a>

  <a id="ref-for-large-viewport-percentage-units②"></a>

  <a id="ref-for-valdef-length-lvmax"></a>

  <a id="ref-for-valdef-length-lvmin"></a>

  <a id="ref-for-valdef-length-lvb"></a>

  <a id="ref-for-valdef-length-lvi"></a>

  <a id="ref-for-valdef-length-lvw"></a>

  <a id="ref-for-valdef-length-lvh"></a>

  <a id="ref-for-small-viewport-percentage-units②"></a>

  <a id="ref-for-valdef-length-svmax"></a>

  <a id="ref-for-valdef-length-svmin"></a>

  <a id="ref-for-valdef-length-svb"></a>

  <a id="ref-for-valdef-length-svi"></a>

  <a id="ref-for-valdef-length-svw"></a>

  <a id="ref-for-valdef-length-svh"></a>

  Added the [svh](#valdef-length-svh), [svw](#valdef-length-svw), [svi](#valdef-length-svi), [svb](#valdef-length-svb), [svmin](#valdef-length-svmin), and [svmax](#valdef-length-svmax) [small viewport-percentage units](#small-viewport-percentage-units); [lvh](#valdef-length-lvh), [lvw](#valdef-length-lvw), [lvi](#valdef-length-lvi), [lvb](#valdef-length-lvb), [lvmin](#valdef-length-lvmin), and [lvmax](#valdef-length-lvmax) [large viewport-percentage units](#large-viewport-percentage-units); and [dvh](#valdef-length-dvh), [dvw](#valdef-length-dvw), [dvi](#valdef-length-dvi), [dvb](#valdef-length-dvb), [dvmin](#valdef-length-dvmin), and [dvmax](#valdef-length-dvmax) [dynamic viewport-percentage units](#dynamic-viewport-percentage-units). ([Issue 4329](https://github.com/w3c/csswg-drafts/issues/4329) and [Issue 6113](https://github.com/w3c/csswg-drafts/issues/6113))

- <a id="ref-for-angle-value①⑦"></a>

  Clamped excessively large [\<angle\>](#angle-value) values to multiples of 360deg. ([Issue 6105](https://github.com/w3c/csswg-drafts/issues/6105))

- Added back [rules on range-checking combined values](#combining-range) lost during move from the [CSS Transitions](https://www.w3.org/TR/css-transitions-1/) specification. ([Issue 6097](https://github.com/w3c/csswg-drafts/issues/6097))

- <a id="ref-for-font-relative-length①"></a>

  <a id="ref-for-propdef-font-size①①"></a>

  Specified that UA-imposed minimum font sizes apply to the used [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) and not to resolution of [font-relative lengths](#font-relative-length). ([Issue 5858](https://github.com/w3c/csswg-drafts/issues/5858))

- <a id="ref-for-funcdef-max⑨"></a>

  <a id="ref-for-funcdef-min⑨"></a>

  Clarified how [min()](#funcdef-min) and [max()](#funcdef-max) percentages can partially simplify. ([Issue 6293](https://github.com/w3c/csswg-drafts/issues/6298))

### <a id="additions-L3"></a> Additions Since Level 3

Changes since [CSS Values and Units Level 3](https://www.w3.org/TR/css-values-3/):

- Explicitly undefined numeric precision/range.
- Added rules for interpolation per value type, and their clarified computed values.
- Updated interpolation of colors to reference [\[CSS-COLOR-4\]](#biblio-css-color-4).

Additions since [CSS Values and Units Level 3](https://www.w3.org/TR/css-values-3/):

- <a id="ref-for-typedef-dashed-ident⑨"></a>

  Defined the [\<dashed-ident\>](#typedef-dashed-ident) type.

- <a id="ref-for-ratio-value①③"></a>

  Defined the [\<ratio\>](#ratio-value) type.

- <a id="ref-for-url-value⑨"></a>

  <a id="ref-for-funcdef-src①"></a>

  Added [src()](#funcdef-src) to the [\<url\>](#url-value) type.

- <a id="ref-for-rlh⑤"></a>

  <a id="ref-for-lh④"></a>

  <a id="ref-for-cap③"></a>

  <a id="ref-for-ic②"></a>

  <a id="ref-for-valdef-length-vb①"></a>

  <a id="ref-for-valdef-length-vi①"></a>

  Added the [vi](#valdef-length-vi), [vb](#valdef-length-vb), [ic](#ic), [cap](#cap), [lh](#lh) and [rlh](#rlh) length units.

- <a id="ref-for-dynamic-viewport-percentage-units③"></a>

  <a id="ref-for-valdef-length-dvmax①"></a>

  <a id="ref-for-valdef-length-dvmin①"></a>

  <a id="ref-for-valdef-length-dvb①"></a>

  <a id="ref-for-valdef-length-dvi①"></a>

  <a id="ref-for-valdef-length-dvw①"></a>

  <a id="ref-for-valdef-length-dvh①"></a>

  <a id="ref-for-small-viewport-percentage-units③"></a>

  <a id="ref-for-valdef-length-svmax①"></a>

  <a id="ref-for-valdef-length-svmin①"></a>

  <a id="ref-for-valdef-length-svb①"></a>

  <a id="ref-for-valdef-length-svi①"></a>

  <a id="ref-for-valdef-length-svw①"></a>

  <a id="ref-for-valdef-length-svh①"></a>

  Added the [svh](#valdef-length-svh), [svw](#valdef-length-svw), [svi](#valdef-length-svi), [svb](#valdef-length-svb), [svmin](#valdef-length-svmin), and [svmax](#valdef-length-svmax) [small viewport-percentage units](#small-viewport-percentage-units) and [dvh](#valdef-length-dvh), [dvw](#valdef-length-dvw), [dvi](#valdef-length-dvi), [dvb](#valdef-length-dvb), [dvmin](#valdef-length-dvmin), and [dvmax](#valdef-length-dvmax) [dynamic viewport-percentage units](#dynamic-viewport-percentage-units).

- <a id="ref-for-dppx②"></a>

  <a id="ref-for-x"></a>

  Added the [x](#x) alias to [dppx](#dppx).

- <a id="ref-for-funcdef-clamp①⓪"></a>

  <a id="ref-for-funcdef-max①⓪"></a>

  <a id="ref-for-funcdef-min①⓪"></a>

  Added [min()](#funcdef-min), [max()](#funcdef-max), and [clamp()](#funcdef-clamp) [comparison functions](#comp-func).

- <a id="ref-for-funcdef-sign④"></a>

  <a id="ref-for-funcdef-abs④"></a>

  <a id="ref-for-funcdef-exp④"></a>

  <a id="ref-for-funcdef-log⑤"></a>

  <a id="ref-for-funcdef-hypot⑧"></a>

  <a id="ref-for-funcdef-sqrt⑥"></a>

  <a id="ref-for-funcdef-pow⑥"></a>

  <a id="ref-for-funcdef-atan2④"></a>

  <a id="ref-for-funcdef-atan⑤"></a>

  <a id="ref-for-funcdef-acos⑤"></a>

  <a id="ref-for-funcdef-asin⑤"></a>

  <a id="ref-for-funcdef-tan⑤"></a>

  <a id="ref-for-funcdef-cos⑤"></a>

  <a id="ref-for-funcdef-sin⑥"></a>

  <a id="ref-for-funcdef-rem①①"></a>

  <a id="ref-for-funcdef-mod⑧"></a>

  <a id="ref-for-funcdef-round⑤"></a>

  Added [round()](#funcdef-round), [mod()](#funcdef-mod), [rem()](#funcdef-rem), [sin()](#funcdef-sin), [cos()](#funcdef-cos), [tan()](#funcdef-tan), [asin()](#funcdef-asin), [acos()](#funcdef-acos), [atan()](#funcdef-atan), [atan2()](#funcdef-atan2), [pow()](#funcdef-pow), [sqrt()](#funcdef-sqrt), [hypot()](#funcdef-hypot), [log()](#funcdef-log), [exp()](#funcdef-exp), [abs()](#funcdef-abs), [sign()](#funcdef-sign) math functions.

- <a id="ref-for-funcdef-calc②②"></a>

  <a id="ref-for-valdef-calc-nan③"></a>

  <a id="ref-for-valdef-calc--infinity②"></a>

  <a id="ref-for-valdef-calc-infinity④"></a>

  <a id="ref-for-valdef-calc-pi②"></a>

  <a id="ref-for-valdef-calc-e②"></a>

  Added [e](#valdef-calc-e), [pi](#valdef-calc-pi), [infinity](#valdef-calc-infinity), [-infinity](#valdef-calc--infinity), [NaN](#valdef-calc-nan) constants for use in [calc()](#funcdef-calc).

- <a id="ref-for-dimension①②"></a>

  <a id="ref-for-funcdef-calc②③"></a>

  Added [unit algebra](#calc-type-checking) to [calc()](#funcdef-calc), allowing multiplication and division of [dimensions](#dimension).

- <a id="ref-for-funcdef-toggle①⑦"></a>

  Added back [toggle()](#funcdef-toggle) (punted from level 3 originally).

- <a id="ref-for-integer-value①⑤"></a>

  A non-integer in a calc() automatically rounds to the nearest integer when used where an [\<integer\>](#integer-value) is required.

- <a id="ref-for-math-function④⑨"></a>

  Defined [serialization](#calc-serialize) of [math functions](#math-function).

## <a id="sec-pri"></a> Security and Privacy Considerations

This specification mostly just defines units that are common to CSS specifications, and which present no security concerns.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Does URL handling have a security concern? Probably.

This specification defines units that expose the user’s screen size and default font size, but both are trivially observable from JS, so they do not constitute a new privacy risk.

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

- [!](#mult-req), in §2.3
- [\#](#mult-comma), in §2.3
- [&#x26;&#x26;](#comb-all), in §2.2
- [\*](#mult-zero-plus), in §2.3
- [+](#mult-one-plus), in §2.3
- [,](#comb-comma), in §2.1
- [?](#mult-opt), in §2.3
- [\|](#comb-one), in §2.2
- [\|\|](#comb-any), in §2.2
- [{A}](#mult-num), in §2.3
- [{A,B}](#mult-num-range), in §2.3
- [abs()](#funcdef-abs), in §11.6
- [absolute length](#absolute-length), in §6.2
- [accumulate](#accumulation), in §3
- [accumulation](#accumulation), in §3
- [accumulation procedure](#accumulation), in §3
- [acos()](#funcdef-acos), in §11.4
- [add](#addition), in §3
- [addition](#addition), in §3
- [addition procedure](#addition), in §3
- [advance measure](#length-advance-measure), in §6.1.1
- [anchor](#anchor-unit), in §6.2
- [anchor unit](#anchor-unit), in §6.2
- [\<angle\>](#angle-value), in §7.1
- [angle](#valdef-attr-angle), in §10.1
- [\<angle-percentage\>](#typedef-angle-percentage), in §5.6
- [asin()](#funcdef-asin), in §11.4
- [atan()](#funcdef-atan), in §11.4
- [atan2()](#funcdef-atan2), in §11.4
- [attr()](#funcdef-attr), in §10
- [\<attr-type\>](#typedef-attr-type), in §10
- [bearing angle](#bearing-angle), in §7.1
- [between zero and B](#between-zero-and-b), in §11.3
- [bracketed range notation](#css-bracketed-range-notation), in §5.1
- [calc()](#funcdef-calc), in §11.1
- [\<calc-constant\>](#typedef-calc-constant), in §11.8
- [calc-operator nodes](#calculation-tree-calc-operator-nodes), in §11.10
- [\<calc-product\>](#typedef-calc-product), in §11.8
- [\<calc-sum\>](#typedef-calc-sum), in §11.8
- [calculation](#calc-calculation), in §11.1
- [calculation tree](#calculation-tree), in §11.10
- [\<calc-value\>](#typedef-calc-value), in §11.8
- [canonical](#canonical-unit), in §5.4.1
- [canonical unit](#canonical-unit), in §5.4.1
- [cap](#cap), in §6.1.1
- [ch](#ch), in §6.1.1
- [clamp()](#funcdef-clamp), in §11.2
- [cm](#cm), in §6.2
- [color](#valdef-attr-color), in §10.1
- [combine](#combine), in §3
- [compatible](#compatible-units), in §5.4.1
- [compatible units](#compatible-units), in §5.4.1
- [cos()](#funcdef-cos), in §11.4
- [CSS bracketed range notation](#css-bracketed-range-notation), in §5.1
- [CSS ident](#css-css-identifier), in §4
- [CSS identifier](#css-css-identifier), in §4
- [CSS-wide keywords](#css-wide-keywords), in §4.1.1
- [\<custom-ident\>](#identifier-value), in §4.2
- [\<dashed-ident\>](#typedef-dashed-ident), in §4.3
- [deg](#deg), in §7.1
- [degenerate ratio](#degenerate-ratio), in §5.7
- [determine the type of a calculation](#determine-the-type-of-a-calculation), in §11.9
- [\<dimension\>](#typedef-dimension), in §5.4
- [dimension](#dimension), in §5.4
- \<dimension-unit\>
  - [(type)](#typedef-dimension-unit), in §10
  - [value for attr()](#valdef-attr-dimension-unit), in §10.1
- [down](#valdef-rounding-strategy-down), in §11.3
- [dpcm](#dpcm), in §7.4
- [dpi](#dpi), in §7.4
- [dppx](#dppx), in §7.4
- [dvb](#valdef-length-dvb), in §6.1.2.2
- [dvh](#valdef-length-dvh), in §6.1.2.2
- [dvi](#valdef-length-dvi), in §6.1.2.2
- [dvmax](#valdef-length-dvmax), in §6.1.2.2
- [dvmin](#valdef-length-dvmin), in §6.1.2.2
- [dvw](#valdef-length-dvw), in §6.1.2.2
- [dynamic viewport-percentage units](#dynamic-viewport-percentage-units), in §6.1.2.1
- [dynamic viewport size](#dynamic-viewport-size), in §6.1.2.1
- [e](#valdef-calc-e), in §11.7
- [em](#em), in §6.1.1
- [ex](#ex), in §6.1.1
- [exp()](#funcdef-exp), in §11.5
- [flex](#valdef-attr-flex), in §10.1
- [font-relative lengths](#font-relative-length), in §6.1.1
- [\<frequency\>](#frequency-value), in §7.3
- [frequency](#valdef-attr-frequency), in §10.1
- [\<frequency-percentage\>](#typedef-frequency-percentage), in §5.6
- [functional notation](#functional-notation), in §9
- [grad](#grad), in §7.1
- [hypot()](#funcdef-hypot), in §11.5
- [Hz](#Hz), in §7.3
- [ic](#ic), in §6.1.1
- [\<ident\>](#typedef-ident), in §4
- ident
  - [dfn for CSS](#css-css-identifier), in §4
  - [value for attr()](#valdef-attr-ident), in §10.1
- [identifier](#css-css-identifier), in §4
- [in](#in), in §6.2
- [-infinity](#valdef-calc--infinity), in §11.7.1
- [infinity](#valdef-calc-infinity), in §11.7.1
- [\<integer\>](#integer-value), in §5.2
- [integer](#integer), in §5.2
- [interpolate](#interpolation), in §3
- [interpolation](#interpolation), in §3
- [interpolation procedure](#interpolation), in §3
- [keyword](#css-keyword), in §4.1
- [kHz](#kHz), in §7.3
- [large viewport-percentage units](#large-viewport-percentage-units), in §6.1.2.1
- [large viewport size](#large-viewport-size), in §6.1.2.1
- [\<length\>](#length-value), in §6
- [length](#valdef-attr-length), in §10.1
- [\<length-percentage\>](#typedef-length-percentage), in §5.6
- [lh](#lh), in §6.1.1
- [local url flag](#url-local-url-flag), in §4.5.1.1
- [log()](#funcdef-log), in §11.5
- [lvb](#valdef-length-lvb), in §6.1.2.2
- [lvh](#valdef-length-lvh), in §6.1.2.2
- [lvi](#valdef-length-lvi), in §6.1.2.2
- [lvmax](#valdef-length-lvmax), in §6.1.2.2
- [lvmin](#valdef-length-lvmin), in §6.1.2.2
- [lvw](#valdef-length-lvw), in §6.1.2.2
- [math function](#math-function), in §11
- [max()](#funcdef-max), in §11.2
- [min()](#funcdef-min), in §11.2
- [mm](#mm), in §6.2
- [mod()](#funcdef-mod), in §11.3
- [ms](#ms), in §7.2
- [NaN](#valdef-calc-nan), in §11.7.1
- [nearest](#valdef-rounding-strategy-nearest), in §11.3
- [not additive](#not-additive), in §3
- [\<number\>](#number-value), in §5.3
- number
  - [definition of](#number), in §5.3
  - [value for attr()](#valdef-attr-number), in §10.1
- [numeric data types](#numeric-data-types), in §5
- [operator nodes](#calculation-tree-operator-nodes), in §11.10
- [parse a calculation](#parse-a-calculation), in §11.10
- [parsing a calculation](#parse-a-calculation), in §11.10
- [pc](#pc), in §6.2
- [\<percentage\>](#percentage-value), in §5.5
- percentage
  - [definition of](#percentage), in §5.5
  - [value for attr()](#valdef-attr-percentage), in §10.1
- [physical unit](#physical-unit), in §6.2
- [pi](#valdef-calc-pi), in §11.7
- [pixel unit](#visual-angle-unit), in §6.2
- [\<position\>](#typedef-position), in §8.3
- [pow()](#funcdef-pow), in §11.5
- [pt](#pt), in §6.2
- [px](#px), in §6.2
- [Q](#Q), in §6.2
- [rad](#rad), in §7.1
- [\<ratio\>](#ratio-value), in §5.7
- [ratio](#ratio), in §5.7
- [reference pixel](#reference-pixel), in §6.2
- [relative length](#relative-length), in §6.1
- [rem](#rem), in §6.1.1
- [rem()](#funcdef-rem), in §11.3
- [\<resolution\>](#resolution-value), in §7.4
- [rlh](#rlh), in §6.1.1
- [round()](#funcdef-round), in §11.3
- [\<rounding-strategy\>](#typedef-rounding-strategy), in §11.3
- [s](#s), in §7.2
- [serialize a calculation tree](#serialize-a-calculation-tree), in §11.13
- [serialize a math function](#serialize-a-math-function), in §11.13
- [serialize the calculation tree](#serialize-a-calculation-tree), in §11.13
- [serializing a calculation tree](#serialize-a-calculation-tree), in §11.13
- [serializing the calculation tree](#serialize-a-calculation-tree), in §11.13
- [sign()](#funcdef-sign), in §11.6
- [simplify](#simplify-a-calculation-tree), in §11.10.1
- [simplify a calculation tree](#simplify-a-calculation-tree), in §11.10.1
- [simplifying a calculation tree](#simplify-a-calculation-tree), in §11.10.1
- [sin()](#funcdef-sin), in §11.4
- [small viewport-percentage units](#small-viewport-percentage-units), in §6.1.2.1
- [small viewport size](#small-viewport-size), in §6.1.2.1
- [sort a calculation’s children](#sort-a-calculations-children), in §11.13
- [sqrt()](#funcdef-sqrt), in §11.5
- [src()](#funcdef-src), in §4.5
- [\<string\>](#string-value), in §4.4
- [string](#valdef-attr-string), in §10.1
- [substitute an attr()](#substitute-an-attr), in §10.2
- [substitution value](#attr-substitution-value), in §10
- [svb](#valdef-length-svb), in §6.1.2.2
- [svh](#valdef-length-svh), in §6.1.2.2
- [svi](#valdef-length-svi), in §6.1.2.2
- [svmax](#valdef-length-svmax), in §6.1.2.2
- [svmin](#valdef-length-svmin), in §6.1.2.2
- [svw](#valdef-length-svw), in §6.1.2.2
- [tan()](#funcdef-tan), in §11.4
- [textual data types](#css-textual-data-types), in §4
- [\<time\>](#time-value), in §7.2
- [time](#valdef-attr-time), in §10.1
- [\<time-percentage\>](#typedef-time-percentage), in §5.6
- [toggle()](#funcdef-toggle), in §9.1
- [\<toggle-value\>](#typedef-toggle-value), in §9.1
- [top-level calculation](#top-level-calculation), in §11.9
- [to-zero](#valdef-rounding-strategy-to-zero), in §11.3
- [turn](#turn), in §7.1
- [UA-default viewport-percentage units](#ua-default-viewport-percentage-units), in §6.1.2.1
- [UA-default viewport size](#ua-default-viewport-size), in §6.1.2.1
- [up](#valdef-rounding-strategy-up), in §11.3
- [\<url\>](#url-value), in §4.5
- [url](#valdef-attr-url), in §10.1
- [url()](#funcdef-url), in §4.5
- [\<url-modifier\>](#typedef-url-modifier), in §4.5.3
- [value accumulation](#accumulation), in §3
- [value addition](#addition), in §3
- [value definition syntax](#css-value-definition-syntax), in §2
- [value interpolation](#interpolation), in §3
- [vb](#valdef-length-vb), in §6.1.2.2
- [vh](#valdef-length-vh), in §6.1.2.2
- [vi](#valdef-length-vi), in §6.1.2.2
- [viewport-percentage lengths](#viewport-percentage-lengths), in §6.1.2
- [visual angle unit](#visual-angle-unit), in §6.2
- [vmax](#valdef-length-vmax), in §6.1.2.2
- [vmin](#valdef-length-vmin), in §6.1.2.2
- [vw](#valdef-length-vw), in §6.1.2.2
- [x](#x), in §7.4
- [\<zero\>](#zero-value), in §5.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-animations-1\] defines the following terms:
  - <a id="term-for-propdef-animation"></a>animation
  - <a id="term-for-propdef-animation-iteration-count"></a>animation-iteration-count
  - <a id="term-for-propdef-animation-name"></a>animation-name
  - <a id="term-for-propdef-animation-timing-function"></a>animation-timing-function
- \[css-box-4\] defines the following terms:
  - <a id="term-for-propdef-padding-top"></a>padding-top
- \[css-break-3\] defines the following terms:
  - <a id="term-for-propdef-orphans"></a>orphans
- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-at-ruledef-import"></a>@import
  - <a id="term-for-actual-value"></a>actual value
  - <a id="term-for-computed-value"></a>computed value
  - <a id="term-for-valdef-all-inherit"></a>inherit
  - <a id="term-for-inherited-value"></a>inherited value
  - <a id="term-for-valdef-all-initial"></a>initial
  - <a id="term-for-shorthand-property"></a>shorthand
  - <a id="term-for-shorthand-property①"></a>shorthand property
  - <a id="term-for-specified-value"></a>specified value
  - <a id="term-for-valdef-all-unset"></a>unset
  - <a id="term-for-used-value"></a>used value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="term-for-typedef-color"></a>\<color\>
  - <a id="term-for-typedef-hex-color"></a>\<hex-color\>
  - <a id="term-for-at-ruledef-profile"></a>@color-profile
  - <a id="term-for-named-color"></a>named color
  - <a id="term-for-propdef-opacity"></a>opacity
  - <a id="term-for-funcdef-rgba"></a>rgba()
- \[css-color-5\] defines the following terms:
  - <a id="term-for-funcdef-hsl"></a>hsl()
- \[css-counter-styles-3\] defines the following terms:
  - <a id="term-for-disc"></a>disc
- \[css-display-3\] defines the following terms:
  - <a id="term-for-containing-block"></a>containing block
  - <a id="term-for-initial-containing-block"></a>initial containing block
- \[css-easing-1\] defines the following terms:
  - <a id="term-for-typedef-easing-function"></a>\<easing-function\>
  - <a id="term-for-valdef-cubic-bezier-easing-function-ease-in"></a>ease-in
  - <a id="term-for-valdef-cubic-bezier-easing-function-ease-out"></a>ease-out
  - <a id="term-for-easing-function"></a>easing function
  - <a id="term-for-easing-function①"></a>timing function
- \[css-grid-2\] defines the following terms:
  - <a id="term-for-typedef-flex"></a>\<flex\>
  - <a id="term-for-valdef-flex-fr"></a>fr
- \[css-images-4\] defines the following terms:
  - <a id="term-for-propdef-image-resolution"></a>image-resolution
- \[css-inline-3\] defines the following terms:
  - <a id="term-for-valdef-line-height-normal"></a>normal
- \[css-overflow-4\] defines the following terms:
  - <a id="term-for-propdef-max-lines"></a>max-lines
- \[css-page-3\] defines the following terms:
  - <a id="term-for-page-area"></a>page area
- \[css-rhythm-1\] defines the following terms:
  - <a id="term-for-propdef-block-step-size"></a>block-step-size
- \[css-sizing-3\] defines the following terms:
  - <a id="term-for-valdef-width-auto"></a>auto
  - <a id="term-for-propdef-box-sizing"></a>box-sizing
  - <a id="term-for-propdef-height"></a>height
  - <a id="term-for-propdef-min-width"></a>min-width
  - <a id="term-for-propdef-width"></a>width
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="term-for-typedef-declaration-value"></a>\<declaration-value\>
  - <a id="term-for-typedef-delim-token"></a>\<delim-token\>
  - <a id="term-for-typedef-dimension-token"></a>\<dimension-token\>
  - <a id="term-for-typedef-function-token"></a>\<function-token\>
  - <a id="term-for-typedef-ident-token"></a>\<ident-token\>
  - <a id="term-for-typedef-number-token"></a>\<number-token\>
  - <a id="term-for-typedef-percentage-token"></a>\<percentage-token\>
  - <a id="term-for-typedef-string-token"></a>\<string-token\>
  - <a id="term-for-typedef-url-token"></a>\<url-token\>
  - <a id="term-for-typedef-whitespace-token"></a>\<whitespace-token\>
  - <a id="term-for-component-value"></a>component value
  - <a id="term-for-consume-a-url-token"></a>consume a url token
  - <a id="term-for-parse-a-component-value"></a>parse a component value
  - <a id="term-for-simple-block"></a>simple block
  - <a id="term-for-whitespace"></a>whitespace
- \[css-text-3\] defines the following terms:
  - <a id="term-for-valdef-text-align-center"></a>center
  - <a id="term-for-propdef-tab-size"></a>tab-size
  - <a id="term-for-propdef-text-align"></a>text-align
- \[css-text-decor-3\] defines the following terms:
  - <a id="term-for-propdef-text-decoration"></a>text-decoration
- \[css-transforms-1\] defines the following terms:
  - <a id="term-for-propdef-transform-origin"></a>transform-origin
- \[css-typed-om-1\] defines the following terms:
  - <a id="term-for-cssnumericvalue-add-two-types"></a>add two types
  - <a id="term-for-css-internal-representation"></a>internal representation
  - <a id="term-for-cssnumericvalue-invert-a-type"></a>invert a type
  - <a id="term-for-cssnumericvalue-match"></a>match
  - <a id="term-for-cssnumericvalue-multiply-two-types"></a>multiply two types
  - <a id="term-for-cssnumericvalue-percent-hint"></a>percent hint
  - <a id="term-for-cssnumericvalue-type"></a>type
- \[css-ui-3\] defines the following terms:
  - <a id="term-for-valdef-cursor-default"></a>default
  - <a id="term-for-propdef-outline-color"></a>outline-color
- \[css-variables-1\] defines the following terms:
  - <a id="term-for-custom-property"></a>custom property
  - <a id="term-for-guaranteed-invalid-value"></a>guaranteed-invalid value
  - <a id="term-for-invalid-at-computed-value-time"></a>invalid at computed-value time
  - <a id="term-for-substitute-a-var"></a>substitute a var()
  - <a id="term-for-funcdef-var"></a>var()
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-block-axis"></a>block axis
  - <a id="term-for-inline-axis"></a>inline axis
  - <a id="term-for-propdef-text-orientation"></a>text-orientation
  - <a id="term-for-valdef-text-orientation-upright"></a>upright
  - <a id="term-for-valdef-writing-mode-vertical-lr"></a>vertical-lr
  - <a id="term-for-valdef-writing-mode-vertical-rl"></a>vertical-rl
  - <a id="term-for-propdef-writing-mode"></a>writing-mode
- \[CSS21\] defines the following terms:
  - <a id="term-for-value-def-border-width"></a>\<border-width\>
  - <a id="term-for-propdef-background-position"></a>background-position
  - <a id="term-for-propdef-border-collapse"></a>border-collapse
  - <a id="term-for-value-def-circle"></a>circle
  - <a id="term-for-value-def-disc"></a>disc
  - <a id="term-for-propdef-line-height"></a>line-height
  - <a id="term-for-value-def-square"></a>square
- \[CSS3-BACKGROUND\] defines the following terms:
  - <a id="term-for-propdef-background"></a>background
  - <a id="term-for-propdef-background-attachment"></a>background-attachment
  - <a id="term-for-propdef-background-position①"></a>background-position
  - <a id="term-for-propdef-border-color"></a>border-color
  - <a id="term-for-propdef-border-width"></a>border-width
  - <a id="term-for-propdef-box-shadow"></a>box-shadow
- \[CSS3-FONTS\] defines the following terms:
  - <a id="term-for-propdef-font"></a>font
  - <a id="term-for-propdef-font-family"></a>font-family
  - <a id="term-for-propdef-font-size"></a>font-size
- \[CSS3-IMAGES\] defines the following terms:
  - <a id="term-for-typedef-image"></a>\<image\>
  - <a id="term-for-funcdef-linear-gradient"></a>linear-gradient()
- \[DOM\] defines the following terms:
  - <a id="term-for-concept-attribute"></a>attribute
  - <a id="term-for-concept-element"></a>element
- \[HTML\] defines the following terms:
  - <a id="term-for-the-base-element"></a>base
  - <a id="term-for-dom-history-pushstate"></a>pushState(data, unused)
- \[INFRA\] defines the following terms:
  - <a id="term-for-ascii-case-insensitive"></a>ascii case-insensitive
  - <a id="term-for-string-concatenate"></a>concatenate
  - <a id="term-for-list-iterate"></a>for each
  - <a id="term-for-string-is"></a>identical to
  - <a id="term-for-string"></a>string
  - <a id="term-for-strip-leading-and-trailing-ascii-whitespace"></a>strip leading and trailing ascii whitespace
- \[mediaqueries-5\] defines the following terms:
  - <a id="term-for-continuous-media"></a>continuous media
  - <a id="term-for-media-query"></a>media query
  - <a id="term-for-paged-media"></a>paged media
- \[selectors-4\] defines the following terms:
  - <a id="term-for-typedef-wq-name"></a>\<wq-name\>
  - <a id="term-for-attribute-selector"></a>attribute selector
  - <a id="term-for-originating-element"></a>originating element
- \[URL\] defines the following terms:
  - <a id="term-for-concept-url"></a>url
- \[web-animations-1\] defines the following terms:
  - <a id="term-for-discrete"></a>discrete

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 8 June 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 1 June 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-counter-styles-3"></a>\[CSS-COUNTER-STYLES-3\]  
Tab Atkins Jr.. [CSS Counter Styles Level 3](https://www.w3.org/TR/css-counter-styles-3/). 14 December 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-counter-styles-3&#x2F;](https://www.w3.org/TR/css-counter-styles-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-easing-1"></a>\[CSS-EASING-1\]  
Brian Birtles; et al. [CSS Easing Functions Level 1](https://www.w3.org/TR/css-easing-1/). 1 April 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-1&#x2F;](https://www.w3.org/TR/css-easing-1/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Image Values and Replaced Content Module Level 4](https://www.w3.org/TR/css-images-4/). 13 April 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad; Steve Zilles. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 27 August 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-page-3"></a>\[CSS-PAGE-3\]  
Elika Etemad; Simon Sapin. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 18 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 16 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-typed-om-1"></a>\[CSS-TYPED-OM-1\]  
Shane Stephens; Tab Atkins Jr.; Naina Raisinghani. [CSS Typed OM Level 1](https://www.w3.org/TR/css-typed-om-1/). 10 April 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-typed-om-1&#x2F;](https://www.w3.org/TR/css-typed-om-1/)

<a id="biblio-css-ui-3"></a>\[CSS-UI-3\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 21 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 3 December 2015. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css3-background"></a>\[CSS3-BACKGROUND\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 22 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css3-fonts"></a>\[CSS3-FONTS\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css3-images"></a>\[CSS3-IMAGES\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 17 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css3cascade"></a>\[CSS3CASCADE\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://www.w3.org/TR/css-cascade-3/). 11 February 2021. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-3&#x2F;](https://www.w3.org/TR/css-cascade-3/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Simon Pieters; Glenn Adams. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 17 March 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; Florian Rivoal; Tab Atkins Jr.. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 31 July 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 21 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-unicode"></a>\[UNICODE\]  
[The Unicode Standard](https://www.unicode.org/versions/latest/). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;versions&#x2F;latest&#x2F;](https://www.unicode.org/versions/latest/)

<a id="biblio-url"></a>\[URL\]  
Anne van Kesteren. [URL Standard](https://url.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;url&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://url.spec.whatwg.org/)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 18 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

### <a id="informative"></a>Informative References

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
Dean Jackson; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-color-3"></a>\[CSS-COLOR-3\]  
Tantek Çelik; Chris Lilley; David Baron. [CSS Color Module Level 3](https://www.w3.org/TR/css-color-3/). 19 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-3&#x2F;](https://www.w3.org/TR/css-color-3/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 1 June 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-grid-1"></a>\[CSS-GRID-1\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css-overflow-4"></a>\[CSS-OVERFLOW-4\]  
David Baron; Florian Rivoal. [CSS Overflow Module Level 4](https://www.w3.org/TR/css-overflow-4/). 13 June 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-4&#x2F;](https://www.w3.org/TR/css-overflow-4/)

<a id="biblio-css-rhythm-1"></a>\[CSS-RHYTHM-1\]  
Koji Ishii; Elika Etemad. [CSS Rhythmic Sizing](https://www.w3.org/TR/css-rhythm-1/). 2 March 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-rhythm-1&#x2F;](https://www.w3.org/TR/css-rhythm-1/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 22 April 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-text-decor-3"></a>\[CSS-TEXT-DECOR-3\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 3](https://www.w3.org/TR/css-text-decor-3/). 13 August 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-3&#x2F;](https://www.w3.org/TR/css-text-decor-3/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-mediaq"></a>\[MEDIAQ\]  
Florian Rivoal; Tab Atkins Jr.. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 21 July 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-4&#x2F;](https://www.w3.org/TR/mediaqueries-4/)

<a id="biblio-quirks"></a>\[QUIRKS\]  
Simon Pieters. [Quirks Mode Standard](https://quirks.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;quirks&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://quirks.spec.whatwg.org/)

<a id="biblio-rfc6694"></a>\[RFC6694\]  
S. Moonesamy, Ed.. [The "about" URI Scheme](https://datatracker.ietf.org/doc/html/rfc6694). August 2012. Informational. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc6694](https://datatracker.ietf.org/doc/html/rfc6694)

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> We can potentially add more typographic units, like cicero, didot, etc. They’re just absolute units, and so can be done with the existing units, but is there enough desire for them (potentially for printing use-cases) that it would be worth adding them? Or should we just wait for Houdini Custom Units? [↵](#issue-7dc6ac0d)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [Level 3 assumes scrollbars never exist](https://www.w3.org/TR/css-values-3/#viewport-relative-lengths) because it was hard to implement and only Firefox bothered to do so. This is [making authors unhappy](https://github.com/w3c/csswg-drafts/issues/1766#issuecomment-460470368). Can we improve here? [↵](#issue-56221f3d)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Originally the (unprefixed) viewport units were defined relative to the viewport size in general. The dynamism of browser chrome shifting in and out during scrolling was invented later, and following Safari’s lead, most UAs mapped these units to the larger size. Defining it this way is prettier in many cases, but can also block critical content (such as toolbars, headers, and footers) in others. It’s therefore not entirely clear whether this is the best mapping. [↵](#issue-9b4752a7)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Do we want to allow [math functions](#math-function) as attr values for all the numeric types? And color functions for "color"? I think we do, but I’d have to check the contents to make sure they don’t contain further reference functions; `foo="rgb(var(--red), 0, 0)"` needs to be illegal for attr(foo color). [↵](#issue-168aea69)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> attr() and var() substitute at the same time, so I should probably rewrite [substitute a var()](https://www.w3.org/TR/css-variables-1/#substitute-a-var) to be more generally about "substitute a reference" and just use that for both of these functions. [↵](#issue-9dd81cfe)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/5689](https://github.com/w3c/csswg-drafts/issues/5689)[\[cssom\] Is \`round\` special in CSSOM?](https://github.com/w3c/csswg-drafts/issues/5689) [↵](#issue-1a929fd0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section is still [under discussion](https://lists.w3.org/Archives/Member/w3c-css-wg/2016AprJun/0239.html). [↵](#issue-f5bc4b00)
