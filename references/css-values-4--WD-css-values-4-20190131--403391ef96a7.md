Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Values and Units Module Level 4](https://www.w3.org/TR/2019/WD-css-values-4-20190131/).

Original copyright notice: Copyright © 2019 W3C ® ( MIT , ERCIM , Keio , Beihang ). W3C liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Values and Units Module Level 4

Source snapshot: https://www.w3.org/TR/2019/WD-css-values-4-20190131/

Snapshot SHA-256: 403391ef96a7c7d6a9b8c4e0a37622e793821b48dba073184969c04a058526e2

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Values and Units Module Level 4

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2019 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](http://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This CSS module describes the common values and units that CSS properties accept and the syntax used for describing them in CSS property definitions.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of&#xA;   its publication. Other documents may supersede this document. A list of&#xA;   current W3C publications and the latest revision of this technical report&#xA;   can be found in the <a href="https://www.w3.org/TR/">W3C technical reports&#xA;   index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

Publication as a Working Draft does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

[GitHub Issues](https://github.com/w3c/csswg-drafts/issues) are preferred for discussion of this specification. When filing an issue, please put the text “css-values” in the title, preferably like this: “\[css-values\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/), and there is also a [historical archive](https://lists.w3.org/Archives/Public/www-style/).

This document was produced by the [CSS Working Group](https://www.w3.org/Style/CSS/members).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/#sec-Disclosure).

<a id="w3c_process_revision"></a>

This document is governed by the [1 February 2018 W3C Process Document](https://www.w3.org/2018/Process-20180201/).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-funcdef-attr"></a>

  <a id="ref-for-funcdef-toggle"></a>

  [toggle()](#funcdef-toggle), [attr()](#funcdef-attr)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

<a id="ref-for-selectordef-child"></a>

<a id="ref-for-length-value"></a>

The value definition field of each CSS property can contain keywords, data types (which appear between \< and [\>](https://www.w3.org/TR/selectors4/#selectordef-child)), and information on how they can be combined. Generic data types ([\<length\>](#length-value) being the most widely used) that can be used by many properties are described in this specification, while more specific data types (e.g., \<spacing-limit\>) are described in the corresponding modules.

### <a id="placement"></a>1.1.  Module Interactions

This module replaces and extends the data type definitions in [\[CSS21\]](#biblio-css21) sections [1.4.2.1](https://www.w3.org/TR/CSS21/about.html#value-defs), [4.3](https://www.w3.org/TR/CSS21/syndata.html#values), and [A.2](https://www.w3.org/TR/CSS21/aural.html#aural-intro).

## <a id="value-defs"></a>2.  Value Definition Syntax

The syntax described here is used to define the set of valid values for CSS properties. A property value can have one or more components.

### <a id="component-types"></a>2.1.  Component value types

Component value types are designated in several ways:

1.  <a id="ref-for-disc"></a>

    [keyword](#keywords) values (such as auto, [disc](https://www.w3.org/TR/css-counter-styles-3/#disc), etc.), which appear literally, without quotes (e.g. `auto`)

2.  <a id="ref-for-percentage-value"></a>

    <a id="ref-for-length-value①"></a>

    <a id="ref-for-selectordef-child①"></a>

    basic data types, which appear between \< and [\>](https://www.w3.org/TR/selectors4/#selectordef-child) (e.g., [\<length\>](#length-value), [\<percentage\>](#percentage-value), etc.).

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

    types that have the same range of values as a property bearing the same name (e.g., [\<‘border-width’\>](https://www.w3.org/TR/css3-background/#propdef-border-width), [\<‘background-attachment’\>](https://www.w3.org/TR/css3-background/#propdef-background-attachment), etc.). In this case, the type name is the property name (complete with quotes) between the brackets. Such a type does <em>not</em> include [CSS-wide keywords](#common-keywords) such as [inherit](https://www.w3.org/TR/css-cascade-4/#valdef-all-inherit), and also does not include any top-level [comma-separated-list multiplier](#mult-comma) (i.e. if property pairing is defined as \[ [\<custom-ident\>](#identifier-value) [\<integer\>](#integer-value)? \]#, then \<‘pairing’\> is equivalent to \[ [\<custom-ident\>](#identifier-value) [\<integer\>](#integer-value)? \], not [\<custom-ident\>](#identifier-value) [\<integer\>](#integer-value) \]#).

4.  <a id="ref-for-propdef-border-width①"></a>

    <a id="ref-for-value-def-border-width"></a>

    <a id="ref-for-selectordef-child②"></a>

    non-terminals that do not share the same name as a property. In this case, the non-terminal name appears between \< and [\>](https://www.w3.org/TR/selectors4/#selectordef-child), as in \<spacing-limit\>. Notice the distinction between [\<border-width\>](https://www.w3.org/TR/CSS21/box.html#value-def-border-width) and [\<‘border-width’\>](https://www.w3.org/TR/css3-background/#propdef-border-width): the latter is defined as the value of the <a id="ref-for-propdef-border-width②"></a>border-width property, the former requires an explicit expansion elsewhere. The definition of a non-terminal is typically located near its first appearance in the specification.

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

All CSS properties also accept the [CSS-wide keyword values](#common-keywords) as the sole component of their property value. For readability these are not listed explicitly in the property value syntax definitions. For example, the full value definition of [border-color](https://www.w3.org/TR/css3-background/#propdef-border-color) is `<color>{1,4} | inherit | initial | unset` (even though it is listed as `<color>{1,4}`).

<a id="ref-for-propdef-background"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This implies that, in general, combining these keywords with other component values in the same declaration results in an invalid declaration. For example, [background: url(corner.png) no-repeat, inherit;](https://www.w3.org/TR/css3-background/#propdef-background) is invalid.

### <a id="component-combinators"></a>2.2.  Component value combinators

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
### <a id="component-multipliers"></a>2.3.  Component value multipliers

Every type, keyword, or bracketed group may be followed by one of the following modifiers:

- An asterisk (<a id="mult-zero-plus"></a>\*) indicates that the preceding type, word, or group occurs zero or more times.
- A plus (<a id="mult-one-plus"></a>+) indicates that the preceding type, word, or group occurs one or more times.
- A question mark (<a id="mult-opt"></a>?) indicates that the preceding type, word, or group is optional (occurs zero or one times).
- A single number in curly braces (<a id="mult-num"></a>{<var>A</var>}) indicates that the preceding type, word, or group occurs <var>A</var> times.
- A comma-separated pair of numbers in curly braces (<a id="mult-num-range"></a>{<var>A</var>,<var>B</var>}) indicates that the preceding type, word, or group occurs at least <var>A</var> and at most <var>B</var> times. The <var>B</var> may be omitted ({<var>A</var>,}) to indicate that there must be at least <var>A</var> repetitions, with no upper bound on the number of repetitions.
- A hash mark (<a id="mult-comma"></a>\#) indicates that the preceding type, word, or group occurs one or more times, separated by comma tokens (which may optionally be surrounded by [white space](https://www.w3.org/TR/css-syntax/#whitespace) and/or comments). It may optionally be followed by the curly brace forms, above, to indicate precisely how many times the repetition occurs, like \<length\>#{1,4}.
- An exclamation point (<a id="mult-req"></a>!) after a group indicates that the group is required and must produce at least one value; even if the grammar of the items within the group would otherwise allow the entire contents to be omitted, at least one component value must not be omitted.

<a id="ref-for-x"></a>

<a id="ref-for-selectordef-adjacent"></a>

For repeated component values (indicated by [\*](https://www.w3.org/TR/css3-selectors/#x), [+](https://www.w3.org/TR/selectors4/#selectordef-adjacent), or \#), UAs must support at least 20 repetitions of the component. If a property value contains more than the supported number of repetitions, the declaration must be ignored as if it were invalid.

### <a id="combinator-multiplier-patterns"></a>2.4.  Combinator and Multiplier Patterns

<a id="ref-for-component-value"></a>

There are a small set of common ways to combine multiple independent [component values](https://drafts.csswg.org/css-syntax-3/#component-value) in particular numbers and orders. In particular, it’s common to want to express that, from a set of component value, the author must select zero or more, one or more, or all of them, and in either the order specified in the grammar or in any order.

All of these can be easily expressed using simple patterns of [combinators](#component-combinators) and [multipliers](#component-multipliers):



|                     | in order          | any order         |
|---------------------|-------------------|-------------------|
| <strong>zero or more &#xA;      </strong> | <code>A?&#x20;B?&#x20;C?</code> | <code>A?&#x20;&#x7C;&#x7C;&#x20;B?&#x20;&#x7C;&#x7C;&#x20;C?</code> |
| <strong>one or more &#xA;      </strong> | <code>&#x5B;&#x20;A?&#x20;B?&#x20;C?&#x20;&#x5D;!</code> | <code>A&#x20;&#x7C;&#x7C;&#x20;B&#x20;&#x7C;&#x7C;&#x20;C</code> |
| <strong>all &#xA;      </strong> | <code>A&#x20;B&#x20;C&#x20;</code> | <code>A&#x20;&amp;&amp;&#x20;B&#x20;&amp;&amp;&#x20;C</code> |



Note that all of the "any order" possibilities are expressed using combinators, while the "in order" possibilities are all variants on juxtaposition.

### <a id="component-whitespace"></a>2.5.  Component values and white space

Unless otherwise specified, [white space](https://www.w3.org/TR/css-syntax/#whitespace) and/or comments may appear before, after, and/or between components combined using the above [combinators](#component-combinators) and [multipliers](#component-multipliers).

<a id="ref-for-typedef-dimension-token"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In many cases, spaces will in fact be <em>required</em> between components in order to distinguish them from each other. For example, the value 1em2em would be parsed as a single [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token) with the number 1 and the identifier em2em, which is an invalid unit. In this case, a space would be required before the 2 to get this parsed as the two lengths 1em and 2em.

### <a id="value-examples"></a>2.6.  Property value examples

Below are some examples of properties with their corresponding value definition fields

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0263a08c"></a>
>
> <a id="propvalues"></a>
>
> 
>
> | Property                                                                                              | Value definition field                                                                                               | Example value                                                                           |
> |-------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------|
> | <a id="ref-for-propdef-orphans"></a>[orphans](https://www.w3.org/TR/css3-break/#propdef-orphans)                       | \<integer\>                                                                                                          | 3                                                                                       |
> | <a id="ref-for-propdef-text-align"></a>[text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align)                 | left \| right \| center \| justify                                                                                   | <a id="ref-for-valdef-text-align-center"></a>[center](https://www.w3.org/TR/css-text-3/#valdef-text-align-center) |
> | <a id="ref-for-propdef-padding-top"></a>[padding-top](https://www.w3.org/TR/css-box-3/#propdef-padding-top)                | \<length\> \| \<percentage\>                                                                                         | 5%                                                                                      |
> | <a id="ref-for-propdef-outline-color"></a>[outline-color](https://www.w3.org/TR/css3-ui/#propdef-outline-color)              | \<color\> \| invert                                                                                                  | \#fefefe                                                                                |
> | <a id="ref-for-propdef-text-decoration"></a>[text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration) | none \| underline \|\| overline \|\| line-through \|\| blink                                                         | overline underline                                                                      |
> | <a id="ref-for-propdef-font-family"></a>[font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family)              | \[ \<family-name\> \| \<generic-family\> \]#                                                                         | "Gill Sans", Futura, sans-serif                                                         |
> | <a id="ref-for-propdef-border-width③"></a>[border-width](https://www.w3.org/TR/css3-background/#propdef-border-width)        | \[ \<length\> \| thick \| medium \| thin \]{1,4}                                                                     | 2px medium 4px                                                                          |
> | <a id="ref-for-propdef-text-shadow"></a>[text-shadow](https://www.w3.org/TR/css-text-decor-3/#propdef-text-shadow)         | \[ inset? &#x26;&#x26; \[ \<length\>{2,4} &#x26;&#x26; \<color\>? \] \]# \| none | 3px 3px rgba(50%, 50%, 50%, 50%), lemonchiffon 0 0 4px inset                            |
>
> 

## <a id="combining-values"></a>3.  Combining Values: Interpolation, Addition, and Accumulation

<a id="ref-for-computed-value"></a>

Some procedures, for example [transitions](https://www.w3.org/TR/css-transitions/) and [animations](https://www.w3.org/TR/css-animations/), combine two CSS property values. The following combining operations—on the two [computed values](https://www.w3.org/TR/css-cascade-4/#computed-value) <var>V<sub>a</sub></var> and <var>V<sub>B</sub></var> yielding the <a id="ref-for-computed-value①"></a>computed value <var>V<sub>result</sub></var>—are defined:

<a id="interpolation"></a>interpolation  
Given two property values <var>V<sub>a</sub></var> and <var>V<sub>B</sub></var>, produces an intermediate value <var>V<sub>result</sub></var> at a distance of <var>p</var> along the interval between <var>V<sub>a</sub></var> and <var>V<sub>B</sub></var> such that <var>p</var> = 0 produces <var>V<sub>a</sub></var> and <var>p</var> = 1 produces <var>V</var><sub>B</sub>.

<a id="ref-for-timing-function"></a>

The range of <var>p</var> is (−∞, ∞) due to the effect of [timing functions](https://www.w3.org/TR/css-easing-1/#timing-function). As a result, this procedure must also define extrapolation behavior for <var>p</var> outside \[0, 1\].

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

These operations are only defined on [computed values](https://www.w3.org/TR/css-cascade-4/#computed-value). (As a result, it is not necessary to define, for example, how to add a [\<length\>](#length-value) value of 15pt with 5em since such values will be resolved to their [canonical unit](#canonical-unit) before being passed to any of the above procedures.)

<a id="ref-for-addition④"></a>

If a value type does not define a specific procedure for [addition](#addition) or is defined as <a id="not-additive"></a>not additive, its <a id="ref-for-addition⑤"></a>addition operation is simply <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var>.

<a id="ref-for-accumulation③"></a>

<a id="ref-for-addition⑥"></a>

If a value types does not define a specific procedure for [accumulation](#accumulation), its <a id="ref-for-accumulation④"></a>accumulation operation is identical to [addition](#addition).

## <a id="textual-values"></a>4.  Textual Data Types

<a id="ref-for-specified-value"></a>

<a id="ref-for-computed-value③"></a>

Textual data types are used to represent identifiers or text. Aside from the casing of [pre-defined keywords](#keywords) or as explicitly defined for a given property, no normalization is performed, not even Unicode normalization: the [specified](https://www.w3.org/TR/css-cascade-4/#specified-value) and [computed value](https://www.w3.org/TR/css-cascade-4/#computed-value) of a property are exactly the provided Unicode values after parsing (which includes character set conversion and [escaping](https://www.w3.org/TR/css-syntax-3/#escaping)). [\[UNICODE\]](#biblio-unicode) [\[CSS3SYN\]](#biblio-css3syn)

<a id="ref-for-typedef-ident-token"></a>

<a id="ref-for-css-identifier"></a>

<a id="css-identifier"></a>CSS identifiers, generically denoted by <a id="typedef-ident"></a>\<ident\>, consist of a sequence of characters conforming to the [\<ident-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-ident-token) grammar. [\[CSS3SYN\]](#biblio-css3syn) Identifiers cannot be quoted; otherwise they would be interpreted as strings. CSS properties accept two classes of [identifiers](#css-identifier): [pre-defined keywords](#keywords) and [author-defined identifiers](#custom-idents).

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

<a id="ref-for-css-identifier①"></a>

<a id="ref-for-ascii-case-insensitive"></a>

In the value definition fields, <a id="keyword"></a>keywords with a pre-defined meaning appear literally. Keywords are [CSS identifiers](#css-identifier) and are interpreted [ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive) (i.e., \[a-z\] and \[A-Z\] are equivalent).

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

#### <a id="common-keywords"></a>4.1.1.  CSS-wide keywords: [initial](https://www.w3.org/TR/css-cascade-4/#valdef-all-initial), [inherit](https://www.w3.org/TR/css-cascade-4/#valdef-all-inherit) and [unset](https://www.w3.org/TR/css-cascade-4/#valdef-all-unset)

As defined [above](#component-types), all properties accept the <a id="css-wide-keywords"></a>CSS-wide keywords, which represent value computations common to all CSS properties.

<a id="ref-for-valdef-all-initial①"></a>

<a id="ref-for-valdef-all-inherit②"></a>

<a id="ref-for-valdef-all-unset①"></a>

The [initial](https://www.w3.org/TR/css-cascade-4/#valdef-all-initial) keyword represents the value specified as the property’s initial value. The [inherit](https://www.w3.org/TR/css-cascade-4/#valdef-all-inherit) keyword represents the computed value of the property on the element’s parent. The [unset](https://www.w3.org/TR/css-cascade-4/#valdef-all-unset) keyword acts as either <a id="ref-for-valdef-all-inherit③"></a>inherit or <a id="ref-for-valdef-all-initial②"></a>initial, depending on whether the property is inherited or not. All of these keywords are normatively defined in the Cascade module. [\[CSS3CASCADE\]](#biblio-css3cascade)

Other CSS specifications can define additional CSS-wide keywords.

<a id="ref-for-identifier-value④"></a>

### <a id="custom-idents"></a>4.2.  Author-defined Identifiers: the [\<custom-ident\>](#identifier-value) type

<a id="ref-for-css-identifier②"></a>

<a id="ref-for-case-sensitive"></a>

Some properties accept arbitrary author-defined identifiers as a component value. This generic data type is denoted by <a id="identifier-value"></a>\<custom-ident\>, and represents any valid CSS [identifier](#css-identifier) that would not be misinterpreted as a pre-defined keyword in that property’s value definition. Such identifiers are fully [case-sensitive](https://html.spec.whatwg.org/multipage/infrastructure.html#case-sensitive) (meaning they’re compared by codepoint), even in the ASCII range (e.g. example and EXAMPLE are two different, unrelated user-defined identifiers).

<a id="ref-for-css-wide-keywords"></a>

<a id="ref-for-identifier-value⑤"></a>

<a id="ref-for-valdef-cursor-default"></a>

<a id="ref-for-ascii-case-insensitive①"></a>

The [CSS-wide keywords](#css-wide-keywords) are not valid [\<custom-ident\>](#identifier-value)s. The [default](https://www.w3.org/TR/css3-ui/#valdef-cursor-default) keyword is reserved and is also not a valid <a id="ref-for-identifier-value⑥"></a>\<custom-ident\>. Specifications using <a id="ref-for-identifier-value⑦"></a>\<custom-ident\> must specify clearly what other keywords are excluded from <a id="ref-for-identifier-value⑧"></a>\<custom-ident\>, if any—for example by saying that any pre-defined keywords in that property’s value definition are excluded. Excluded keywords are excluded in all [ASCII case permutations](https://infra.spec.whatwg.org/#ascii-case-insensitive).

<a id="ref-for-identifier-value⑨"></a>

When parsing positionally-ambiguous keywords in a property value, a [\<custom-ident\>](#identifier-value) production can only claim the keyword if no other unfulfilled production can claim it.

<a id="ref-for-propdef-animation"></a>

<a id="ref-for-propdef-animation-timing-function"></a>

<a id="ref-for-valdef-cubic-bezier-timing-function-ease-in"></a>

<a id="ref-for-typedef-timing-function"></a>

<a id="ref-for-valdef-cubic-bezier-timing-function-ease-out"></a>

<a id="ref-for-identifier-value①⓪"></a>

<a id="ref-for-propdef-animation-name"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4758cf8c"></a> For example, the shorthand declaration [animation: ease-in ease-out](https://www.w3.org/TR/css3-animations/#propdef-animation) is equivalent to the longhand declarations [animation-timing-function: ease-in; animation-name: ease-out;](https://www.w3.org/TR/css3-animations/#propdef-animation-timing-function). [ease-in](https://www.w3.org/TR/css-easing-1/#valdef-cubic-bezier-timing-function-ease-in) is claimed by the [\<timing-function\>](https://www.w3.org/TR/css-easing-1/#typedef-timing-function) production belonging to <a id="ref-for-propdef-animation-timing-function①"></a>animation-timing-function, leaving [ease-out](https://www.w3.org/TR/css-easing-1/#valdef-cubic-bezier-timing-function-ease-out) to be claimed by the [\<custom-ident\>](#identifier-value) production belonging to [animation-name](https://www.w3.org/TR/css3-animations/#propdef-animation-name).

<a id="ref-for-identifier-value①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When designing grammars with [\<custom-ident\>](#identifier-value), the <a id="ref-for-identifier-value①②"></a>\<custom-ident\> should always be "positionally unambiguous", so that it’s impossible to conflict with any keyword values in the property.

<a id="ref-for-string-value"></a>

### <a id="strings"></a>4.3.  Quoted Strings: the [\<string\>](#string-value) type

<a id="ref-for-typedef-string-token"></a>

<a id="string"></a>Strings are denoted by <a id="string-value"></a>\<string\> and consist of a sequence of characters delimited by double quotes or single quotes. They correspond to the [\<string-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-string-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS3SYN\]](#biblio-css3syn).

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

<a id="ref-for-url-value"></a>

### <a id="urls"></a>4.4.  Resource Locators: the [\<url\>](#url-value) type

<a id="ref-for-functional-notation"></a>

<a id="ref-for-url-value①"></a>

<a id="ref-for-concept-url"></a>

The <a id="funcdef-url"></a>url() [functional notation](#functional-notation), denoted by [\<url\>](#url-value), represents a [URL](https://url.spec.whatwg.org/#concept-url), which is a pointer to a resource. The typical syntax of a <a id="ref-for-url-value②"></a>\<url\> is:

<a id="url-value"></a>

<a id="ref-for-string-value①"></a>

<a id="ref-for-typedef-url-modifier"></a>

<a id="ref-for-mult-zero-plus"></a>

```text
<url> = url( <string> <url-modifier>* )
```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5917b301"></a> Below is an example of a URL being used as a background image:
>
> ```text
> body { background: url("http://www.example.com/pinkish.gif") }
> ```
<a id="ref-for-url-value③"></a>

<a id="ref-for-consume-a-url-token0"></a>

<a id="ref-for-typedef-url-token"></a>

A [\<url\>](#url-value) may alternately be written without quotation marks around the URL itself, in which case it is [specially-parsed](https://www.w3.org/TR/css-syntax-3/#consume-a-url-token0) as a [\<url-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-url-token) [\[CSS3SYN\]](#biblio-css3syn).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2e04cf61"></a> For example, the following declarations are identical:
>
> ```text
> background: url("http://www.example.com/pinkish.gif");
> background: url(http://www.example.com/pinkish.gif);
> ```
<a id="ref-for-typedef-url-modifier①"></a>

<a id="ref-for-string-value②"></a>

<a id="ref-for-funcdef-url"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This unquoted syntax is cannot accept a [\<url-modifier\>](#typedef-url-modifier) argument and has extra escaping requirements: parentheses, [whitespace](https://www.w3.org/TR/css-syntax/#whitespace) characters, single quotes (') and double quotes (") appearing in a URL must be escaped with a backslash, e.g. url(open&#x5C;(parens), url(close&#x5C;)parens). (In quoted [\<string\>](#string-value) [url()](#funcdef-url)s, only newlines and the character used to quote the string need to be escaped.) Depending on the type of URL, it might also be possible to write these characters as URL-escapes (e.g. url(open%28parens) or url(close%29parens)) as described in [\[URL\]](#biblio-url).

<a id="ref-for-at-ruledef-import"></a>

<a id="ref-for-url-value④"></a>

<a id="ref-for-string-value③"></a>

<a id="ref-for-funcdef-url①"></a>

Some CSS contexts (such as [@import](https://www.w3.org/TR/css-cascade-4/#at-ruledef-import)) also allow a [\<url\>](#url-value) to be represented by a bare [\<string\>](#string-value), without the [url()](#funcdef-url) wrapper. In such cases the string behaves identically to a <a id="ref-for-funcdef-url②"></a>url() function containing that string.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d264fc07"></a> For example, the following statements are identical:
>
> ```text
> @import url("base-theme.css");
> @import "base-theme.css";
> ```
#### <a id="relative-urls"></a>4.4.1.  Relative URLs

In order to create modular style sheets that are not dependent on the absolute location of a resource, authors should use relative URLs. Relative URLs (as defined in [\[URL\]](#biblio-url)) are resolved to full URLs using a base URL. RFC 3986, section 3, defines the normative algorithm for this process. For CSS style sheets, the base URL is that of the style sheet itself, not that of the styled source document. Style sheets embedded within a document have the base URL associated with their container.

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

##### <a id="local-urls"></a>4.4.1.1.  Fragment URLs

To work around some common eccentriticites in browser URL handling, CSS has special behavior for fragment-only urls.

<a id="ref-for-funcdef-url③"></a>

If a [url()](#funcdef-url)’s value starts with a U+0023 NUMBER SIGN (`#`) character, parse it as per normal for URLs, but additionally set the <a id="url-local-url-flag"></a>local url flag of the <a id="ref-for-funcdef-url④"></a>url().

<a id="ref-for-funcdef-url⑤"></a>

<a id="ref-for-url-local-url-flag"></a>

When matching a [url()](#funcdef-url) with the [local url flag](#url-local-url-flag) set, ignore everything but the URL’s fragment, and resolve that fragment against the current document that relative URLs are resolved against. This reference must always be treated as same-document (rather than cross-document).

<a id="ref-for-funcdef-url⑥"></a>

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

#### <a id="url-empty"></a>4.4.2.  Empty URLs

<a id="ref-for-funcdef-url⑦"></a>

If the value of the [url()](#funcdef-url) is the empty string (like url("") or <a id="ref-for-funcdef-url⑧"></a>url()), the url must resolve to an invalid resource (similar to what the url about:invalid does).

<a id="ref-for-funcdef-url⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This matches the behavior of empty urls for embedded resources elsewhere in the web platform, and avoids excess traffic re-requesting the stylesheet or host document due to editting mistakes leaving the [url()](#funcdef-url) value empty, which are almost certain to be invalid resources for whatever the <a id="ref-for-funcdef-url①⓪"></a>url() shows up in. Linking on the web platform <em>does</em> allow empty urls, so if/when CSS gains some functionality to control hyperlinks, this restriction can be relaxed in those contexts.

#### <a id="url-modifiers"></a>4.4.3.  URL Modifiers

<a id="ref-for-funcdef-url①①"></a>

<a id="ref-for-typedef-url-modifier②"></a>

<a id="ref-for-typedef-ident①"></a>

<a id="ref-for-functional-notation①"></a>

The [url()](#funcdef-url) function supports specifying additional <a id="typedef-url-modifier"></a>\<url-modifier\>s, which change the meaning or the interpretation of the URL somehow. A [\<url-modifier\>](#typedef-url-modifier) is either an [\<ident\>](#typedef-ident) or a [functional notation](#functional-notation).

<a id="ref-for-typedef-url-modifier③"></a>

This specification does not define any [\<url-modifier\>](#typedef-url-modifier)s, but other specs may do so.

<a id="ref-for-url-value⑥"></a>

<a id="ref-for-funcdef-url①②"></a>

<a id="ref-for-typedef-url-modifier④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A [\<url\>](#url-value) that is either unquoted or not wrapped in [url()](#funcdef-url) notation cannot accept any [\<url-modifier\>](#typedef-url-modifier)s.

## <a id="numeric-types"></a>5.  Numeric Data Types

<a id="ref-for-specified-value①"></a>

<a id="ref-for-computed-value④"></a>

Numeric data types are used to represent quantities, indexes, positions, and other such values. Although many syntactic variations can exist in expressing the quantity (numeric aspect) in a given numeric value, the [specified](https://www.w3.org/TR/css-cascade-4/#specified-value) and [computed value](https://www.w3.org/TR/css-cascade-4/#computed-value) do not distinguish these variations: they represent the value’s abstract quantity, not its syntactic representation.

Properties may restrict numeric values to some range. If the value is outside the allowed range, unless otherwise specified, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore).

CSS theoretically supports infinite precision and infinite ranges for all value types; however in reality implementations have finite capacity. UAs should support reasonably useful ranges and precisions.

<a id="ref-for-integer-value③"></a>

### <a id="integers"></a>5.1.  Integers: the [\<integer\>](#integer-value) type

Integer values are denoted by <a id="integer-value"></a>\<integer\>.

<a id="ref-for-typedef-number-token"></a>

When written literally, an <a id="integer"></a>integer is one or more decimal digits 0 through 9 and corresponds to a subset of the [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token) production in the CSS Syntax Module [\[CSS3SYN\]](#biblio-css3syn). The first digit of an integer may be immediately preceded by - or + to indicate the integer’s sign.

<a id="ref-for-integer-value④"></a>

#### <a id="combine-integers"></a>5.1.1.  Combination of [\<integer\>](#integer-value)

<a id="ref-for-interpolation②"></a>

<a id="ref-for-integer-value⑤"></a>

<a id="ref-for-number-value"></a>

[Interpolation](#interpolation) of [\<integer\>](#integer-value) is defined as <var>V</var><sub>result</sub> = round((1 - <var>p</var>) × <var>V<sub>a</sub></var> + <var>p</var> × <var>V<sub>b</sub></var>); that is, interpolation happens in the real number space as for [\<number\>](#number-value)s, and the result is converted to an <a id="ref-for-integer-value⑥"></a>\<integer\> by rounding to the nearest integer, with values halfway between adjacent integers rounded towards positive infinity.

<a id="ref-for-addition⑦"></a>

<a id="ref-for-number-value①"></a>

[Addition](#addition) of [\<number\>](#number-value) is defined as <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var> + <var>V<sub>b</sub></var>

<a id="ref-for-number-value②"></a>

### <a id="numbers"></a>5.2.  Real Numbers: the [\<number\>](#number-value) type

Number values are denoted by <a id="number-value"></a>\<number\>, and represent real numbers, possibly with a fractional component.

<a id="ref-for-integer"></a>

<a id="ref-for-typedef-number-token①"></a>

<a id="ref-for-selectordef-adjacent①"></a>

When written literally, a <a id="number"></a>number is either an [integer](#integer), or zero or more decimal digits followed by a dot (.) followed by one or more decimal digits and optionally an exponent composed of "e" or "E" and an integer. It corresponds to the [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS3SYN\]](#biblio-css3syn). As with integers, the first character of a number may be immediately preceded by - or [+](https://www.w3.org/TR/selectors4/#selectordef-adjacent) to indicate the number’s sign.

<a id="ref-for-number"></a>

<a id="ref-for-number-value③"></a>

<a id="ref-for-zero-value"></a>

<a id="ref-for-typedef-number-token②"></a>

The value <a id="zero-value"></a>\<zero\> represents a literal [number](#number) with the value 0. Expressions that merely evaluate to a [\<number\>](#number-value) with the value 0 (for example, calc(0)) do not match [\<zero\>](#zero-value); only literal [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token)s do.

<a id="ref-for-number-value④"></a>

#### <a id="combine-numbers"></a>5.2.1.  Combination of [\<number\>](#number-value)

<a id="ref-for-interpolation③"></a>

<a id="ref-for-number-value⑤"></a>

[Interpolation](#interpolation) of [\<number\>](#number-value) is defined as <var>V</var><sub>result</sub> = (1 - <var>p</var>) × <var>V<sub>a</sub></var> + <var>p</var> × <var>V<sub>b</sub></var>

<a id="ref-for-addition⑧"></a>

<a id="ref-for-number-value⑥"></a>

[Addition](#addition) of [\<number\>](#number-value) is defined as <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var> + <var>V<sub>b</sub></var>

<a id="ref-for-dimension"></a>

### <a id="dimensions"></a>5.3.  Numbers with Units: [dimension](#dimension) values

The general term <a id="dimension"></a>dimension refers to a number with a unit attached to it; and is denoted by <a id="typedef-dimension"></a>\<dimension\>.

<a id="ref-for-dimension①"></a>

<a id="ref-for-number①"></a>

<a id="ref-for-css-identifier③"></a>

<a id="ref-for-typedef-dimension-token①"></a>

<a id="ref-for-ascii-case-insensitive②"></a>

When written literally, a [dimension](#dimension) is a [number](#number) immediately followed by a unit identifier, which is an [identifier](#css-identifier). It corresponds to the [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS3SYN\]](#biblio-css3syn). Like keywords, unit identifiers are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive).

<a id="ref-for-typedef-dimension"></a>

<a id="ref-for-length-value③"></a>

<a id="ref-for-time-value"></a>

<a id="ref-for-frequency-value"></a>

<a id="ref-for-resolution-value"></a>

CSS uses [\<dimension\>](#typedef-dimension)s to specify distances ([\<length\>](#length-value)), durations ([\<time\>](#time-value)), frequencies ([\<frequency\>](#frequency-value)), resolutions ([\<resolution\>](#resolution-value)), and other quantities.

#### <a id="compat"></a>5.3.1.  Compatible Units

<a id="ref-for-computed-value⑤"></a>

<a id="ref-for-px"></a>

<a id="ref-for-in"></a>

<a id="ref-for-propdef-font-size"></a>

<a id="ref-for-em"></a>

<a id="ref-for-canonical-unit①"></a>

When [serializing](https://www.w3.org/TR/cssom-1/#serializing-css-values) [computed values](https://www.w3.org/TR/css-cascade-4/#computed-value) [\[CSSOM\]](#biblio-cssom), <a id="compatible-units"></a>compatible units (those related by a static multiplicative factor, like the 96:1 factor between [px](#px) and [in](#in), or the the computed [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) factor between [em](#em) and <a id="ref-for-px①"></a>px) are converted into a single <a id="canonical-unit"></a>canonical unit. Each group of compatible units defines which among them is the [canonical unit](#canonical-unit) that will be used for serialization.

<a id="ref-for-used-value"></a>

<a id="ref-for-compatible-units"></a>

<a id="ref-for-dimension②"></a>

When serializing [resolved values](https://www.w3.org/TR/cssom-1/#resolved-values) that are [used values](https://www.w3.org/TR/css-cascade-4/#used-value), all value types (percentages, numbers, keywords, etc.) that represent lengths are considered [compatible](#compatible-units) with lengths. Likewise any future API that returns <a id="ref-for-used-value①"></a>used values must consider any values represent distances/durations/frequencies/etc. as <a id="ref-for-compatible-units①"></a>compatible with the relevant class of [dimensions](#dimension), and canonicalize accordingly.

#### <a id="combine-dimensions"></a>5.3.2.  Combination of Dimensions

<a id="ref-for-interpolation④"></a>

<a id="ref-for-compatible-units②"></a>

<a id="ref-for-dimension③"></a>

<a id="ref-for-length-value④"></a>

[Interpolation](#interpolation) of [compatible](#compatible-units) [dimensions](#dimension) (for example, two [\<length\>](#length-value) values) is defined as <var>V</var><sub>result</sub> = (1 - <var>p</var>) × <var>V<sub>a</sub></var> + <var>p</var> × <var>V<sub>b</sub></var>

<a id="ref-for-addition⑨"></a>

<a id="ref-for-compatible-units③"></a>

<a id="ref-for-dimension④"></a>

[Addition](#addition) of [compatible](#compatible-units) [dimensions](#dimension) is defined as <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var> + <var>V<sub>b</sub></var>

<a id="ref-for-percentage-value①"></a>

### <a id="percentages"></a>5.4.  Percentages: the [\<percentage\>](#percentage-value) type

Percentage values are denoted by <a id="percentage-value"></a>\<percentage\>, and indicates a value that is some fraction of another reference value.

<a id="ref-for-number②"></a>

<a id="ref-for-valdef-type-or-value"></a>

<a id="ref-for-typedef-percentage-token"></a>

When written literally, a <a id="percentage"></a>percentage consists of a [number](#number) immediately followed by a percent sign [%](#valdef-type-or-value). It corresponds to the [\<percentage-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-percentage-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS3SYN\]](#biblio-css3syn).

<a id="ref-for-containing-block"></a>

Percentage values are always relative to another quantity, for example a length. Each property that allows percentages also defines the quantity to which the percentage refers. This quantity can be a value of another property for the same element, the value of a property for an ancestor element, a measurement of the formatting context (e.g., the width of a [containing block](https://www.w3.org/TR/css-display-3/#containing-block)), or something else.

<a id="ref-for-percentage-value②"></a>

#### <a id="combine-percentages"></a>5.4.1.  Combination of [\<percentage\>](#percentage-value)

<a id="ref-for-interpolation⑤"></a>

<a id="ref-for-percentage-value③"></a>

[Interpolation](#interpolation) of [\<percentage\>](#percentage-value) is defined as <var>V</var><sub>result</sub> = (1 - <var>p</var>) × <var>V<sub>a</sub></var> + <var>p</var> × <var>V<sub>b</sub></var>

<a id="ref-for-addition①⓪"></a>

<a id="ref-for-percentage-value④"></a>

[Addition](#addition) of [\<percentage\>](#percentage-value) is defined as <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var> + <var>V<sub>b</sub></var>

### <a id="mixed-percentages"></a>5.5.  Mixing Percentages and Dimensions

<a id="ref-for-percentage-value⑤"></a>

<a id="ref-for-dimension⑤"></a>

<a id="ref-for-component-value①"></a>

<a id="ref-for-funcdef-calc"></a>

In cases where a [\<percentage\>](#percentage-value) can represent the same quantity as a [dimension](#dimension) in the same [component value](https://drafts.csswg.org/css-syntax-3/#component-value) position, and can therefore be combined with them in a [calc()](#funcdef-calc) expression, the following convenience notations may be used in the property grammar:

<a id="typedef-length-percentage"></a>\<length-percentage\>  
<a id="ref-for-length-value⑤"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-percentage-value⑥"></a>

<a id="ref-for-percentage-value⑦"></a>

<a id="ref-for-length-value⑥"></a>

Equivalent to <code>&#x5B;&#x20;<a href="#length-value" title="Expands to: em | vb | ch | cm | vh | vi | in | ex | vw | ic | pt | px | lh | pc | rem | rlh | vmax | advance measure | vmin | mm | cap | q">&lt;length&gt;</a>&#x20;<a href="#comb-one">&#x7C;</a>&#x20;<a href="#percentage-value">&lt;percentage&gt;</a>&#x20;&#x5D;</code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<length\>](#length-value).

<a id="typedef-frequency-percentage"></a>\<frequency-percentage\>  
<a id="ref-for-frequency-value①"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-percentage-value⑧"></a>

<a id="ref-for-percentage-value⑨"></a>

<a id="ref-for-frequency-value②"></a>

Equivalent to <code>&#x5B;&#x20;<a href="#frequency-value" title="Expands to: hz | khz">&lt;frequency&gt;</a>&#x20;<a href="#comb-one">&#x7C;</a>&#x20;<a href="#percentage-value">&lt;percentage&gt;</a>&#x20;&#x5D;</code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<frequency\>](#frequency-value).

<a id="typedef-angle-percentage"></a>\<angle-percentage\>  
<a id="ref-for-angle-value"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-percentage-value①⓪"></a>

<a id="ref-for-percentage-value①①"></a>

<a id="ref-for-angle-value①"></a>

Equivalent to <code>&#x5B;&#x20;<a href="#angle-value" title="Expands to: turn | rad | grad | deg">&lt;angle&gt;</a>&#x20;<a href="#comb-one">&#x7C;</a>&#x20;<a href="#percentage-value">&lt;percentage&gt;</a>&#x20;&#x5D;</code>, where the [\<percentage\>](#percentage-value) will resolve to an [\<angle\>](#angle-value).

<a id="typedef-time-percentage"></a>\<time-percentage\>  
<a id="ref-for-time-value①"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-percentage-value①②"></a>

<a id="ref-for-percentage-value①③"></a>

<a id="ref-for-time-value②"></a>

Equivalent to <code>&#x5B;&#x20;<a href="#time-value" title="Expands to: s | ms">&lt;time&gt;</a>&#x20;<a href="#comb-one">&#x7C;</a>&#x20;<a href="#percentage-value">&lt;percentage&gt;</a>&#x20;&#x5D;</code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<time\>](#time-value).

<a id="ref-for-propdef-width"></a>

<a id="ref-for-length-value⑦"></a>

<a id="ref-for-percentage-value①④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ac6b1005"></a> For example, the [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width) property can accept a [\<length\>](#length-value) or a [\<percentage\>](#percentage-value), both representing a measure of distance. This means that <a id="ref-for-propdef-width①"></a>width: calc(500px + 50%); is allowed—both values are converted to absolute lengths and added. If the containing block is 1000px wide, then <a id="ref-for-propdef-width②"></a>width: 50%; is equivalent to <a id="ref-for-propdef-width③"></a>width: 500px, and <a id="ref-for-propdef-width④"></a>width: calc(50% + 500px) thus ends up equivalent to <a id="ref-for-propdef-width⑤"></a>width: calc(500px + 500px) or <a id="ref-for-propdef-width⑥"></a>width: 1000px.
>
> <a id="ref-for-funcdef-hsl"></a>
>
> <a id="ref-for-percentage-value①⑤"></a>
>
> <a id="ref-for-funcdef-calc①"></a>
>
> On the other hand, the second and third arguments of the [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) function can only be expressed as [\<percentage\>](#percentage-value)s. Although [calc()](#funcdef-calc) productions are allowed in their place, they can only combine percentages with themselves, as in calc(10% + 20%).

<a id="ref-for-percentage-value①⑥"></a>

<a id="ref-for-compatible-units④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Specifications should never alternate [\<percentage\>](#percentage-value) in place of a dimension in a grammar unless they are [compatible](#compatible-units).

<a id="ref-for-number-value⑦"></a>

<a id="ref-for-percentage-value①⑦"></a>

<a id="ref-for-funcdef-calc②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: More \<TYPE-percentage\> productions can be added in the future as needed. A \<number-percentage\> will never be added, as [\<number\>](#number-value) and [\<percentage\>](#percentage-value) can’t be combined in [calc()](#funcdef-calc).

#### <a id="combine-mixed"></a>5.5.1.  Combination of Percentage and Dimension Mixes

<a id="ref-for-interpolation⑥"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="ref-for-typedef-frequency-percentage"></a>

<a id="ref-for-typedef-angle-percentage"></a>

<a id="ref-for-typedef-time-percentage"></a>

[Interpolation](#interpolation) of percengage-dimension value combinations (e.g. [\<length-percentage\>](#typedef-length-percentage), [\<frequency-percentage\>](#typedef-frequency-percentage), [\<angle-percentage\>](#typedef-angle-percentage), [\<time-percentage\>](#typedef-time-percentage) or equivalent notations) is defined as

- <a id="ref-for-length-value⑧"></a>

  <a id="ref-for-interpolation⑦"></a>

  equivalent to [interpolation](#interpolation) of [\<length\>](#length-value) if both <var>V<sub>a</sub></var> and <var>V<sub>b</sub></var> are pure <a id="ref-for-length-value⑨"></a>\<length\> values

- <a id="ref-for-percentage-value①⑧"></a>

  <a id="ref-for-interpolation⑧"></a>

  equivalent to [interpolation](#interpolation) of [\<percentage\>](#percentage-value) if both <var>V<sub>a</sub></var> and <var>V<sub>b</sub></var> are pure <a id="ref-for-percentage-value①⑨"></a>\<percentage\> values

- <a id="ref-for-percentage-value②⓪"></a>

  <a id="ref-for-time-value③"></a>

  <a id="ref-for-angle-value②"></a>

  <a id="ref-for-frequency-value③"></a>

  <a id="ref-for-length-value①⓪"></a>

  <a id="ref-for-interpolation⑨"></a>

  <a id="ref-for-funcdef-calc③"></a>

  equivalent to converting both values into a [calc()](#funcdef-calc) expression representing the sum of the dimension type and a percentage (each possibly zero) and [interpolating](#interpolation) each component individually (as a [\<length\>](#length-value)/[\<frequency\>](#frequency-value)/[\<angle\>](#angle-value)/[\<time\>](#time-value) and as a [\<percentage\>](#percentage-value), respectively)

<a id="ref-for-addition①①"></a>

<a id="ref-for-percentage-value②①"></a>

<a id="ref-for-interpolation①⓪"></a>

[Addition](#addition) of [\<percentage\>](#percentage-value) is defined the same as [interpolation](#interpolation) except by <a id="ref-for-addition①②"></a>adding each component rather than <a id="ref-for-interpolation①①"></a>interpolating it.

<a id="ref-for-length-value①①"></a>

## <a id="lengths"></a>6.  Distance Units: the [\<length\>](#length-value) type

<a id="ref-for-dimension⑥"></a>

Lengths refer to distance measurements and are denoted by <a id="length-value"></a>\<length\> in the property definitions. A length is a [dimension](#dimension).

<a id="ref-for-number-value⑧"></a>

<a id="ref-for-length-value①②"></a>

<a id="ref-for-propdef-line-height"></a>

For zero lengths the unit identifier is optional (i.e. can be syntactically represented as the [\<number\>](#number-value) 0). However, if a 0 could be parsed as either a <a id="ref-for-number-value⑨"></a>\<number\> or a [\<length\>](#length-value) in a property (such as [line-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-line-height)), it must parse as a <a id="ref-for-number-value①⓪"></a>\<number\>.

Properties may restrict the length value to some range. If the value is outside the allowed range, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore).

While some properties allow negative length values, this may complicate the formatting and there may be implementation-specific limits. If a negative length value is allowed but cannot be supported, it must be converted to the nearest value that can be supported.

<a id="ref-for-used-value②"></a>

<a id="ref-for-actual-value"></a>

In cases where the [used](https://www.w3.org/TR/css-cascade-4/#used-value) length cannot be supported, user agents must approximate it in the [actual](https://www.w3.org/TR/css-cascade-4/#actual-value) value.

<a id="ref-for-relative-length"></a>

<a id="ref-for-absolute-length"></a>

There are two types of length units: [relative](#relative-length) and [absolute](#absolute-length).

### <a id="relative-lengths"></a>6.1.  Relative lengths

<a id="relative-length"></a>Relative length units specify a length relative to another length. Style sheets that use relative units can more easily scale from one output environment to another.

The relative units are:



| unit                                       | relative to                                                                                                                                                                        |
|--------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <a id="ref-for-em①"></a>[em](#em)               | font size of the element                                                                                                                                                           |
| <a id="ref-for-ex"></a>[ex](#ex)               | x-height of the element’s font                                                                                                                                                     |
| <a id="ref-for-cap"></a>[cap](#cap)             | cap height (the nominal height of capital letters) of the element’s font                                                                                                           |
| <a id="ref-for-ch"></a>[ch](#ch)               | <a id="ref-for-length-advance-measure"></a>average [character advance](#length-advance-measure) of a narrow glyph in the element’s font, as represented by the “0” (ZERO, U+0030) glyph                    |
| <a id="ref-for-ic"></a>[ic](#ic)               | <a id="ref-for-length-advance-measure①"></a>average [character advance](#length-advance-measure) of a fullwidth glyph in the element’s font, as represented by the “水” (CJK water ideograph, U+6C34) glyph |
| <a id="ref-for-rem"></a>[rem](#rem)             | font size of the root element                                                                                                                                                      |
| <a id="ref-for-lh"></a>[lh](#lh)               | line height of the element                                                                                                                                                         |
| <a id="ref-for-rlh"></a>[rlh](#rlh)             | line height of the root element                                                                                                                                                    |
| <a id="ref-for-vw"></a>[vw](#vw)               | 1% of viewport’s width                                                                                                                                                             |
| <a id="ref-for-vh"></a>[vh](#vh)               | 1% of viewport’s height                                                                                                                                                            |
| <a id="ref-for-valdef-length-vi"></a>[vi](#valdef-length-vi) | <a id="ref-for-inline-axis"></a>1% of viewport’s size in the root element’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis)                                               |
| <a id="ref-for-valdef-length-vb"></a>[vb](#valdef-length-vb) | <a id="ref-for-block-axis"></a>1% of viewport’s size in the root element’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis)                                                 |
| <a id="ref-for-vmin"></a>[vmin](#vmin)           | 1% of viewport’s smaller dimension                                                                                                                                                 |
| <a id="ref-for-vmax"></a>[vmax](#vmax)           | 1% of viewport’s larger dimension                                                                                                                                                  |

Informative Summary of Relative Units



<a id="ref-for-computed-value⑥"></a>

Child elements do not inherit the relative values as specified for their parent; they inherit the [computed values](https://www.w3.org/TR/css-cascade-4/#computed-value).

<a id="ref-for-em②"></a>

<a id="ref-for-ex①"></a>

<a id="ref-for-cap①"></a>

<a id="ref-for-ch①"></a>

<a id="ref-for-ic①"></a>

<a id="ref-for-rem①"></a>

<a id="ref-for-lh①"></a>

<a id="ref-for-rlh①"></a>

#### <a id="font-relative-lengths"></a>6.1.1.  Font-relative lengths: the [em](#em), [ex](#ex), [cap](#cap), [ch](#ch), [ic](#ic), [rem](#rem), [lh](#lh), [rlh](#rlh) units

<a id="ref-for-rem②"></a>

<a id="ref-for-rlh②"></a>

The <a id="font-relative-length"></a>font-relative lengths refer to the font metrics of the element on which they are used—or, in the case of [rem](#rem) and [rlh](#rlh), the metrics of the root element.

![The word 'Sphinx' annotated with various font metrics: ascender height, to the top of the h’s serif; cap height, to the visually approximate top of the S; the x height, to the visually approximate top of the x; the baseline, along the bottom of S, h, i, n, and x; and the descender height, to the bottom fo the p.](https://www.w3.org/TR/2019/WD-css-values-4-20190131/images/Typography_Line_Terms.svg)

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

This measurement is an approximation (and in monospace fonts, an exact measure) of a single narrow glyph’s [advance measure](#length-advance-measure), thus allowing measurements based on an expected glyph count.

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
<a id="ref-for-rem③"></a>

<a id="ref-for-propdef-font-size②"></a>

Equal to the computed value of [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) on the root element. When specified on the <a id="ref-for-propdef-font-size③"></a>font-size property of the root element, the [rem](#rem) units refer to the property’s <em>initial value</em>.

<a id="lh"></a>lh unit  
<a id="ref-for-valdef-line-height-normal"></a>

<a id="ref-for-propdef-line-height①"></a>

Equal to the computed value of the [line-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-line-height) property of the element on which it is used, converting [normal](https://drafts.csswg.org/css-inline-3/#valdef-line-height-normal) to an absolute length by using only the metrics of the [first available font](https://www.w3.org/TR/css3-fonts/#first-available-font).

<a id="rlh"></a>rlh unit  
<a id="ref-for-valdef-line-height-normal①"></a>

<a id="ref-for-propdef-line-height②"></a>

Equal to the computed value of [line-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-line-height) property on the root element, converting [normal](https://drafts.csswg.org/css-inline-3/#valdef-line-height-normal) to an absolute length as above.

<a id="ref-for-propdef-height"></a>

<a id="ref-for-lh②"></a>

<a id="ref-for-rlh③"></a>

<a id="ref-for-propdef-max-lines"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Setting the [height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height) of an element using either the [lh](#lh) or the [rlh](#rlh) units does not enable authors to control the actual number of lines in that element. These units only enable length calculations based on the theoretical size of an ideal empty line; the size of actual lines boxes may differ based on their content. In cases where an author wants to limit the number of actual lines in an element, the [max-lines](https://www.w3.org/TR/css-overflow-4/#propdef-max-lines) property can be used instead.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-7dc6ac0d"></a> We can potentially add more typographic units, like cicero, didot, etc. They’re just absolute units, and so can be done with the existing units, but is there enough desire for them (potentially for printing use-cases) that it would be worth adding them? Or should we just wait for Houdini Custom Units?

<a id="ref-for-media-query"></a>

<a id="ref-for-propdef-font"></a>

<a id="ref-for-propdef-line-height③"></a>

<a id="ref-for-propdef-font-size④"></a>

<a id="ref-for-lh③"></a>

<a id="ref-for-rlh④"></a>

When used outside the context of an element (such as in [media queries](https://www.w3.org/TR/mediaqueries-4/#media-query)), these units refer to the metrics corresponding to the initial values of the [font](https://www.w3.org/TR/css-fonts-3/#propdef-font) and [line-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-line-height) properties. When used in the value of the [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) property on the element they refer to, they resolve against the computed metrics of the parent element—or against the computed metrics corresponding to the initial values of the <a id="ref-for-propdef-font①"></a>font and <a id="ref-for-propdef-line-height④"></a>line-height properties, if the element has no parent. Additionally, when [lh](#lh) or [rlh](#rlh) units are used in the value of the <a id="ref-for-propdef-line-height⑤"></a>line-height property on the element they refer to, they resolve against the computed <a id="ref-for-propdef-line-height⑥"></a>line-height and font metrics of the parent element—or the computed metrics corresponding to the initial values of the <a id="ref-for-propdef-font②"></a>font and <a id="ref-for-propdef-line-height⑦"></a>line-height properties, if the element has no parent. (The other font-relative units continue to resolve against the element’s own metrics when used in <a id="ref-for-propdef-line-height⑧"></a>line-height.)

<a id="ref-for-vw①"></a>

<a id="ref-for-vh①"></a>

<a id="ref-for-valdef-length-vi①"></a>

<a id="ref-for-valdef-length-vb①"></a>

<a id="ref-for-vmin①"></a>

<a id="ref-for-vmax①"></a>

#### <a id="viewport-relative-lengths"></a>6.1.2.  Viewport-percentage lengths: the [vw](#vw), [vh](#vh), [vi](#valdef-length-vi), [vb](#valdef-length-vb), [vmin](#vmin), [vmax](#vmax) units

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-valdef-overflow-auto"></a>

The <a id="viewport-percentage-lengths"></a>viewport-percentage lengths are relative to the size of the [initial containing block](https://www.w3.org/TR/CSS21/visudet.html#containing-block-details). When the height or width of the initial containing block is changed, they are scaled accordingly. However, when the value of [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) on the root element is [auto](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-auto), any scroll bars are assumed not to exist. <strong data-conversion-semantic="note">Note:</strong> Note that the initial containing block’s size is affected by the presence of scrollbars on the viewport.

For paged media, the exact definition of the viewport-percentage lengths is deferred to [\[CSS3PAGE\]](#biblio-css3page).

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

<a id="valdef-length-vi"></a>vi unit  
Equal to 1% of the size of the initial containing block in the direction of the root element’s inline axis.

<a id="valdef-length-vb"></a>vb unit  
Equal to 1% of the size of the initial containing block in the direction of the root element’s block axis.

<a id="vmin"></a>vmin unit  
<a id="ref-for-vh②"></a>

<a id="ref-for-vw②"></a>

Equal to the smaller of [vw](#vw) or [vh](#vh).

<a id="vmax"></a>vmax unit  
<a id="ref-for-vh③"></a>

<a id="ref-for-vw③"></a>

Equal to the larger of [vw](#vw) or [vh](#vh).

<a id="ref-for-media-query①"></a>

<a id="ref-for-valdef-length-vi②"></a>

<a id="ref-for-valdef-length-vb②"></a>

<a id="ref-for-propdef-writing-mode①"></a>

In situations where there is no root element or it hasn’t yet been styled (such as when evaluating [media queries](https://www.w3.org/TR/mediaqueries-4/#media-query)), the [vi](#valdef-length-vi) and [vb](#valdef-length-vb) units use the initial value of the [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) property to determine which axis they correspond to.

<a id="ref-for-cm"></a>

<a id="ref-for-mm"></a>

<a id="ref-for-Q"></a>

<a id="ref-for-in①"></a>

<a id="ref-for-pt"></a>

<a id="ref-for-pc"></a>

<a id="ref-for-px②"></a>

### <a id="absolute-lengths"></a>6.2.  Absolute lengths: the [cm](#cm), [mm](#mm), [Q](#Q), [in](#in), [pt](#pt), [pc](#pc), [px](#px) units

<a id="ref-for-in②"></a>

<a id="ref-for-cm①"></a>

<a id="ref-for-mm①"></a>

<a id="ref-for-pt①"></a>

<a id="ref-for-pc①"></a>

<a id="ref-for-Q①"></a>

<a id="ref-for-px③"></a>

The <a id="absolute-length"></a>absolute length units are fixed in relation to each other and anchored to some physical measurement. They are mainly useful when the output environment is known. The absolute units consist of the <a id="physical-unit"></a>physical units ([in](#in), [cm](#cm), [mm](#mm), [pt](#pt), [pc](#pc), [Q](#Q)) and the <a id="pixel-unit"></a>visual angle/pixel unit ([px](#px)):



| unit                | name                | equivalence         |
|---------------------|---------------------|---------------------|
| <strong><dfn><span><a id="cm"></a></span>cm</dfn> &#xA;      </strong> | centimeters         | 1cm = 96px/2.54     |
| <strong><dfn><span><a id="mm"></a></span>mm</dfn> &#xA;      </strong> | millimeters         | 1mm = 1/10th of 1cm |
| <strong><dfn><span><a id="Q"></a></span>Q</dfn> &#xA;      </strong> | quarter-millimeters | 1Q = 1/40th of 1cm  |
| <strong><dfn><span><a id="in"></a></span>in</dfn> &#xA;      </strong> | inches              | 1in = 2.54cm = 96px |
| <strong><dfn><span><a id="pc"></a></span>pc</dfn> &#xA;      </strong> | picas               | 1pc = 1/6th of 1in  |
| <strong><dfn><span><a id="pt"></a></span>pt</dfn> &#xA;      </strong> | points              | 1pt = 1/72th of 1in |
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

    <a id="ref-for-pixel-unit"></a>

    by relating the [pixel unit](#pixel-unit) to the [reference pixel](#reference-pixel).

<a id="ref-for-anchor-unit"></a>

<a id="ref-for-physical-unit①"></a>

<a id="ref-for-pixel-unit①"></a>

For print media at typical viewing distances, the [anchor unit](#anchor-unit) should be one of the [physical units](#physical-unit) (inches, centimeters, etc). For screen media (including high-resolution devices), low-resolution devices, and devices with unusual viewing distances), it is recommended instead that the <a id="ref-for-anchor-unit①"></a>anchor unit be the [pixel unit](#pixel-unit). For such devices it is recommended that the <a id="ref-for-pixel-unit②"></a>pixel unit refer to the whole number of device pixels that best approximates the reference pixel.

<a id="ref-for-anchor-unit②"></a>

<a id="ref-for-pixel-unit③"></a>

<a id="ref-for-physical-unit②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the [anchor unit](#anchor-unit) is the [pixel unit](#pixel-unit), the [physical units](#physical-unit) might not match their physical measurements. Alternatively if the <a id="ref-for-anchor-unit③"></a>anchor unit is a <a id="ref-for-physical-unit③"></a>physical unit, the <a id="ref-for-pixel-unit④"></a>pixel unit might not map to a whole number of device pixels.

<a id="ref-for-pixel-unit⑤"></a>

<a id="ref-for-physical-unit④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This definition of the [pixel unit](#pixel-unit) and the [physical units](#physical-unit) differs from previous versions of CSS. In particular, in previous versions of CSS the <a id="ref-for-pixel-unit⑥"></a>pixel unit and the <a id="ref-for-physical-unit⑤"></a>physical units were not related by a fixed ratio: the <a id="ref-for-physical-unit⑥"></a>physical units were always tied to their physical measurements while the <a id="ref-for-pixel-unit⑦"></a>pixel unit would vary to most closely match the reference pixel. (This change was made because too much existing content relies on the assumption of 96dpi, and breaking that assumption broke the content.)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Values are case-insensitive and serialize as lower case, for example 1Q serializes as 1q.

The <a id="reference-pixel"></a>reference pixel is the visual angle of one pixel on a device with a pixel density of 96dpi and a distance from the reader of an arm’s length. For a nominal arm’s length of 28 inches, the visual angle is therefore about 0.0213 degrees. For reading at arm’s length, 1px thus corresponds to about 0.26 mm (1/96 inch).

The image below illustrates the effect of viewing distance on the size of a reference pixel: a reading distance of 71 cm (28 inches) results in a reference pixel of 0.26 mm, while a reading distance of 3.5 m (12 feet) results in a reference pixel of 1.3 mm.

![This diagram illustrates how the definition of a pixel depends on the users distance from the viewing surface (paper or screen). The image depicts the user looking at two planes, one 28 inches (71 cm) from the user, the second 140 inches (3.5 m) from the user. An expanding cone is projected from the user’s eye onto each plane. Where the cone strikes the first plane, the projected pixel is 0.26 mm high. Where the cone strikes the second plane, the projected pixel is 1.4 mm high.](https://www.w3.org/TR/2019/WD-css-values-4-20190131/images/pixel1.png)

Showing that pixels must become larger if the viewing distance increases

This second image illustrates the effect of a device’s resolution on the pixel unit: an area of 1px by 1px is covered by a single dot in a low-resolution device (e.g. a typical computer display), while the same area is covered by 16 dots in a higher resolution device (such as a printer).

![This diagram illustrates the relationship between the reference pixel and device pixels (called "dots" below). The image depicts a high resolution (large dot density) laser printer output on the left and a low resolution monitor screen on the right. For the laser printer, one square reference pixel is implemented by 16 dots. For the monitor screen, one square reference pixel is implemented by a single dot.](https://www.w3.org/TR/2019/WD-css-values-4-20190131/images/pixel2.png)

Showing that more device pixels (dots) are needed to cover a 1px by 1px area on a high-resolution device than on a lower-resolution one (of the same approximate viewing distance)

## <a id="other-units"></a>7.  Other Quantities

<a id="ref-for-angle-value③"></a>

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

<a id="ref-for-angle-value④"></a>

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
> <a id="ref-for-angle-value⑤"></a>
>
> For example, in the [linear-gradient()](https://drafts.csswg.org/css-images-3/#funcdef-linear-gradient) function, the [\<angle\>](#angle-value) that determines the direction of the gradient is interpreted as a bearing angle.

<a id="ref-for-angle-value⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For legacy reasons, some uses of [\<angle\>](#angle-value) allow a bare 0 to mean 0deg. This is not true in general, however, and will not occur in future uses of the <a id="ref-for-angle-value⑦"></a>\<angle\> type.

<a id="ref-for-time-value④"></a>

<a id="ref-for-s"></a>

<a id="ref-for-ms"></a>

### <a id="time"></a>7.2.  Duration Units: the [\<time\>](#time-value) type and [s](#s), [ms](#ms) units

<a id="ref-for-dimension⑦"></a>

Time values are [dimensions](#dimension) denoted by <a id="time-value"></a>\<time\>. The time unit identifiers are:

<a id="s"></a>s  
Seconds.

<a id="ms"></a>ms  
Milliseconds. There are 1000 milliseconds in a second.

<a id="ref-for-time-value⑤"></a>

<a id="ref-for-compatible-units⑦"></a>

<a id="ref-for-s①"></a>

<a id="ref-for-canonical-unit④"></a>

All [\<time\>](#time-value) units are [compatible](#compatible-units), and [s](#s) is their [canonical unit](#canonical-unit).

Properties may restrict the time value to some range. If the value is outside the allowed range, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore).

<a id="ref-for-frequency-value④"></a>

<a id="ref-for-Hz"></a>

<a id="ref-for-kHz"></a>

### <a id="frequency"></a>7.3.  Frequency Units: the [\<frequency\>](#frequency-value) type and [Hz](#Hz), [kHz](#kHz) units

<a id="ref-for-dimension⑧"></a>

Frequency values are [dimensions](#dimension) denoted by <a id="frequency-value"></a>\<frequency\>. The frequency unit identifiers are:

<a id="Hz"></a>Hz  
Hertz. It represents the number of occurrences per second.

<a id="kHz"></a>kHz  
KiloHertz. A kiloHertz is 1000 Hertz.

For example, when representing sound pitches, 200Hz (or 200hz) is a bass sound, and 6kHz (or 6khz) is a treble sound.

<a id="ref-for-frequency-value⑤"></a>

<a id="ref-for-compatible-units⑧"></a>

<a id="ref-for-Hz①"></a>

<a id="ref-for-canonical-unit⑤"></a>

All [\<frequency\>](#frequency-value) units are [compatible](#compatible-units), and [hz](#Hz) is their [canonical unit](#canonical-unit).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Values are case-insensitive and serialize as lower case, for example 1Hz serializes as 1hz.

<a id="ref-for-resolution-value①"></a>

<a id="ref-for-dpi"></a>

<a id="ref-for-dpcm"></a>

<a id="ref-for-dppx"></a>

### <a id="resolution"></a>7.4.  Resolution Units: the [\<resolution\>](#resolution-value) type and [dpi](#dpi), [dpcm](#dpcm), [dppx](#dppx) units

<a id="ref-for-dimension⑨"></a>

Resolution units are [dimensions](#dimension) denoted by <a id="resolution-value"></a>\<resolution\>. The resolution unit identifiers are:

<a id="dpi"></a>dpi  
Dots per inch.

<a id="dpcm"></a>dpcm  
Dots per centimeter.

<a id="dppx"></a>dppx  
<a id="x"></a>x  
<a id="ref-for-px⑤"></a>

Dots per [px](#px) unit.

<a id="ref-for-resolution-value②"></a>

<a id="ref-for-in③"></a>

<a id="ref-for-cm②"></a>

<a id="ref-for-px⑥"></a>

<a id="ref-for-propdef-image-resolution"></a>

The [\<resolution\>](#resolution-value) unit represents the size of a single "dot" in a graphical representation by indicating how many of these dots fit in a CSS [in](#in), [cm](#cm), or [px](#px). For uses, see e.g. the resolution media query in [\[MEDIAQ\]](#biblio-mediaq) or the [image-resolution](https://www.w3.org/TR/css4-images/#propdef-image-resolution) property defined in [\[CSS3-IMAGES\]](#biblio-css3-images).

<a id="ref-for-resolution-value③"></a>

<a id="ref-for-compatible-units⑨"></a>

<a id="ref-for-dppx①"></a>

<a id="ref-for-canonical-unit⑥"></a>

All [\<resolution\>](#resolution-value) units are [compatible](#compatible-units), and [dppx](#dppx) is their [canonical unit](#canonical-unit).

<a id="ref-for-in④"></a>

<a id="ref-for-px⑦"></a>

<a id="ref-for-propdef-image-resolution①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that due to the 1:96 fixed ratio of CSS [in](#in) to CSS [px](#px), 1dppx is equivalent to 96dpi. This corresponds to the default resolution of images displayed in CSS: see [image-resolution](https://www.w3.org/TR/css4-images/#propdef-image-resolution).

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

<a id="ref-for-valuea-def-color"></a>

### <a id="colors"></a>8.1.  Colors: the [\<color\>](https://www.w3.org/TR/css3-color/#valuea-def-color) type

<a id="ref-for-valuea-def-color①"></a>

The [\<color\>](https://www.w3.org/TR/css3-color/#valuea-def-color) data type is defined in [\[CSS3COLOR\]](#biblio-css3color). UAs that support CSS Color Level 3 or its successor must interpret <a id="ref-for-valuea-def-color②"></a>\<color\> as defined therein.

<a id="ref-for-valuea-def-color③"></a>

#### <a id="combine-colors"></a>8.1.1.  Combination of [\<color\>](https://www.w3.org/TR/css3-color/#valuea-def-color)

<a id="ref-for-interpolation①②"></a>

<a id="ref-for-valuea-def-color④"></a>

<a id="ref-for-number-value①①"></a>

[Interpolation](#interpolation) of [\<color\>](https://www.w3.org/TR/css3-color/#valuea-def-color) is defined as the independent interpolation of each component (red, green, blue, alpha) as a [\<number\>](#number-value). Interpolation is done between premultiplied colors (that is, colors for which the red, green, and blue components specified have been multiplied by the alpha).

<a id="ref-for-addition①③"></a>

<a id="ref-for-number-value①②"></a>

[Addition](#addition) of [\<number\>](#number-value) is likewise defined as the independent <a id="ref-for-addition①④"></a>addition of each component as a <a id="ref-for-number-value①③"></a>\<number\> in premultiplied space.

<a id="ref-for-valdef-color-currentcolor"></a>

<a id="ref-for-propdef-text-emphasis-color"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-7fabb422"></a> Computed value needs to be able to represent combinations of [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) and an actual color. Consider the value of [text-emphasis-color](https://www.w3.org/TR/css-text-decor-3/#propdef-text-emphasis-color) in `div { text-emphasis: circle; transition: all 2s; } div:hover { text-emphasis-color: lime; } em { color: red; }` See [Issue 445](https://github.com/w3c/csswg-drafts/issues/445).

<a id="ref-for-image-type"></a>

### <a id="images"></a>8.2.  Images: the [\<image\>](https://www.w3.org/TR/css3-images/#image-type) type

<a id="ref-for-image-type①"></a>

<a id="ref-for-url-value⑦"></a>

The [\<image\>](https://www.w3.org/TR/css3-images/#image-type) data type is defined in [\[CSS3-IMAGES\]](#biblio-css3-images). UAs that support CSS Images Level 3 or its successor must interpret <a id="ref-for-image-type②"></a>\<image\> as defined therein. UAs that do not yet support CSS Images Level 3 must interpret <a id="ref-for-image-type③"></a>\<image\> as [\<url\>](#url-value).

<a id="ref-for-image-type④"></a>

#### <a id="combine-images"></a>8.2.1.  Combination of [\<image\>](https://www.w3.org/TR/css3-images/#image-type)

<a id="ref-for-image-type⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Interpolation of [\<image\>](https://www.w3.org/TR/css3-images/#image-type) is defined in [CSS Images 3 §6 Interpolation](https://drafts.csswg.org/css-images-3/#interpolation).

<a id="ref-for-not-additive①"></a>

Images are [not additive](#not-additive).

<a id="ref-for-typedef-position"></a>

### <a id="position"></a>8.3.  2D Positioning: the [\<position\>](#typedef-position) type

<a id="ref-for-typedef-position①"></a>

<a id="ref-for-propdef-background-position"></a>

The <a id="typedef-position"></a>[\<position\>](#typedef-position) value specifies the position of a object area (e.g. background image) inside a positioning area (e.g. background positioning area). It is interpreted as specified for [background-position](https://www.w3.org/TR/css3-background/#propdef-background-position). [\[CSS3-BACKGROUND\]](#biblio-css3-background)

<a id="ref-for-typedef-position②"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-comb-any"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-typedef-length-percentage①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-typedef-length-percentage②"></a>

<a id="ref-for-mult-opt③"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-comb-one①⑦"></a>

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
> Note: The [background-position](https://www.w3.org/TR/css3-background/#propdef-background-position) property also accepts a three-value syntax. This has been disallowed generically because it creates parsing ambiguities when combined with other length or percentage components in a property value.

The canonical order when serializing is the horizontal component followed by the vertical component.

<a id="ref-for-length-value①③"></a>

<a id="ref-for-percentage-value②②"></a>

<a id="ref-for-typedef-position③"></a>

When specified in a grammar alongside other keywords, [\<length\>](#length-value)s, or [\<percentage\>](#percentage-value)s, [\<position\>](#typedef-position) is <em>greedily</em> parsed; it consumes as many components as possible.

<a id="ref-for-propdef-transform-origin"></a>

<a id="ref-for-typedef-position④"></a>

<a id="ref-for-length-value①④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-aa45b932"></a> For example, [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin) defines a 3D position as (effectively) ''[\<position\>](#typedef-position) [\<length\>](#length-value)?''. A value such as left 50px will be parsed as a 2-value <a id="ref-for-typedef-position⑤"></a>\<position\>, with an omitted z-component; on the other hand, a value such as top 50px will be parsed as a single-value <a id="ref-for-typedef-position⑥"></a>\<position\> followed by a <a id="ref-for-length-value①⑤"></a>\<length\>.

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
<a id="ref-for-funcdef-calc④"></a>

<a id="ref-for-funcdef-min"></a>

<a id="ref-for-funcdef-max"></a>

<a id="ref-for-funcdef-clamp"></a>

### <a id="calc-notation"></a>9.1.  Mathematical Expressions: [calc()](#funcdef-calc), [min()](#funcdef-min), [max()](#funcdef-max), and [clamp()](#funcdef-clamp)

<a id="ref-for-selectordef-adjacent②"></a>

<a id="ref-for-x①"></a>

<a id="ref-for-funcdef-calc⑤"></a>

<a id="ref-for-funcdef-min①"></a>

<a id="ref-for-funcdef-max①"></a>

<a id="ref-for-funcdef-clamp①"></a>

The <a id="math-function"></a>math functions, <a id="funcdef-calc"></a>calc(), <a id="funcdef-min"></a>min(), <a id="funcdef-max"></a>max(), and <a id="funcdef-clamp"></a>clamp() allow mathematical expressions with addition ([+](https://www.w3.org/TR/selectors4/#selectordef-adjacent)), subtraction (-), multiplication ([\*](https://www.w3.org/TR/css3-selectors/#x)), and division (/) to be used as component values. A [calc()](#funcdef-calc) function represents the result of the mathematical calculation it contains, using standard operator precedence rules; a [min()](#funcdef-min) or [max()](#funcdef-max) function represents the smallest (most negative) or largest (most positive), respectively, comma-separated calculation it contains; a [clamp()](#funcdef-clamp) function represents its central calculation, clamped according to its min and max calculations (given clamp(MIN, VAL, MAX), it is resolved exactly identically to max(MIN, min(VAL, MAX))).

<a id="ref-for-math-function"></a>

<a id="ref-for-length-value①⑥"></a>

<a id="ref-for-frequency-value⑥"></a>

<a id="ref-for-angle-value⑧"></a>

<a id="ref-for-time-value⑥"></a>

<a id="ref-for-typedef-flex"></a>

<a id="ref-for-resolution-value④"></a>

<a id="ref-for-percentage-value②③"></a>

<a id="ref-for-number-value①④"></a>

<a id="ref-for-integer-value⑦"></a>

<a id="ref-for-funcdef-attr①"></a>

A [math function](#math-function) can be used wherever [\<length\>](#length-value), [\<frequency\>](#frequency-value), [\<angle\>](#angle-value), [\<time\>](#time-value), [\<flex\>](https://www.w3.org/TR/css-grid-1/#typedef-flex), [\<resolution\>](#resolution-value), [\<percentage\>](#percentage-value), [\<number\>](#number-value), or [\<integer\>](#integer-value) values are allowed. Components of a <a id="ref-for-math-function①"></a>math function can be literal values, other <a id="ref-for-math-function②"></a>math functions, or other expressions, such as [attr()](#funcdef-attr), that evaluate to a valid argument type (like <a id="ref-for-length-value①⑦"></a>\<length\>).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1edf37aa"></a>
>
> <a id="ref-for-math-function③"></a>
>
> <a id="ref-for-propdef-box-sizing"></a>
>
> [Math functions](#math-function) can be used to combine value that use different units. In this example the author wants the <em>margin box</em> of each section to take up 1/3 of the space, so they start with 100%/3, then subtract the element’s borders and margins. ([box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing) can automatically achieve this effect for borders and padding, but a <a id="ref-for-math-function④"></a>math function is needed if you want to include margins.)
>
> ```css
> section {
>   float: left;
>   margin: 1em; border: solid 1px;
>   width: calc(100%/3 - 2*1em - 2*1px);
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
> <a id="ref-for-math-function⑤"></a>
>
> <a id="ref-for-propdef-font-size⑤"></a>
>
> [Math functions](#math-function) can also be useful just to express values in a more natural, readable fashion, rather than as an obscure decimal. For example, the following sets the [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) so that exactly 35em fits within the viewport, ensuring that roughly the same amount of text always fills the screen no matter the screen size.
>
> ```css
> :root {
>   font-size: calc(100vw / 35);
> }
> ```
>
> <a id="ref-for-propdef-font-size⑥"></a>
>
> Functionality-wise, this is identical to just writing [font-size: 2.857vw](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), but then the intent (that 35em fills the viewport) is much less clear to someone reading the code; the later reader will have to reverse the math themselves to figure out that 2.857 is meant to approximate 100/35.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-67532822"></a>
>
> <a id="ref-for-funcdef-min②"></a>
>
> <a id="ref-for-funcdef-max②"></a>
>
> <a id="ref-for-funcdef-clamp②"></a>
>
> <a id="ref-for-propdef-font-size⑦"></a>
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
> <a id="ref-for-funcdef-calc⑥"></a>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: Full math expressions are allowed in each of the arguments; there’s no need to nest a [calc()](#funcdef-calc) inside! You can also provide more than two arguments, if you have multiple constraints to apply.

<a id="ref-for-funcdef-min③"></a>

<a id="ref-for-funcdef-max③"></a>

<a id="ref-for-propdef-min-width"></a>

<a id="ref-for-funcdef-clamp③"></a>

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
<a id="ref-for-funcdef-clamp④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that [clamp()](#funcdef-clamp), matching CSS conventions elsewhere, has its minimum value "win" over its maximum value if the two are in the "wrong order". That is, clamp(100px, ..., 50px) will resolve to 100px, exceeding its stated "max" value.
>
> <a id="ref-for-funcdef-clamp⑤"></a>
>
> <a id="ref-for-funcdef-min⑥"></a>
>
> <a id="ref-for-funcdef-max⑥"></a>
>
> If alternate resolution mechanics are desired they can be achieved by combining [clamp()](#funcdef-clamp) with [min()](#funcdef-min) or [max()](#funcdef-max):
>
> To have MAX win over MIN:  
> <a id="ref-for-funcdef-clamp⑥"></a>
>
> clamp(min(MIN, MAX), VAL, MAX). If you want to avoid repeating the MAX calculation, you can just reverse the nesting of functions that [clamp()](#funcdef-clamp) is defined against—min(MAX, max(MIN, VAL)).
>
> To have MAX and MIN "swap" when they’re in the wrong order:  
> clamp(min(MIN, MAX), VAL, max(MIN, MAX)). Unfortunately, there’s no easy way to do this without repeating the MIN and MAX terms.

#### <a id="calc-syntax"></a>9.1.1.  Syntax

<a id="ref-for-math-function⑥"></a>

The syntax of a [math function](#math-function) is:

<a id="ref-for-funcdef-calc⑦"></a>

<a id="ref-for-typedef-calc-sum"></a>

<a id="ref-for-funcdef-min⑦"></a>

<a id="ref-for-typedef-calc-sum①"></a>

<a id="ref-for-mult-comma③"></a>

<a id="ref-for-funcdef-max⑦"></a>

<a id="ref-for-typedef-calc-sum②"></a>

<a id="ref-for-mult-comma①"></a>

<a id="ref-for-funcdef-clamp⑦"></a>

<a id="ref-for-typedef-calc-sum③"></a>

<a id="ref-for-mult-comma②"></a>

<a id="typedef-calc-sum"></a>

<a id="ref-for-typedef-calc-product"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-typedef-calc-product①"></a>

<a id="ref-for-mult-zero-plus①"></a>

<a id="typedef-calc-product"></a>

<a id="ref-for-typedef-calc-value"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-typedef-calc-value①"></a>

<a id="ref-for-mult-zero-plus②"></a>

<a id="typedef-calc-value"></a>

<a id="ref-for-number-value①⑤"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-typedef-dimension②"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-percentage-value②④"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-typedef-calc-sum④"></a>

```text
<calc()>  = calc( <calc-sum> )
<min()>   = min( <calc-sum># )
<max()>   = max( <calc-sum># )
<clamp()> = clamp( <calc-sum>#{3} )
<calc-sum> = <calc-product> [ [ '+' | '-' ] <calc-product> ]*
<calc-product> = <calc-value> [ [ '*' | '/' ] <calc-value> ]*
<calc-value> = <number> | <dimension> | <percentage> | ( <calc-sum> )
```
<a id="ref-for-whitespace"></a>

<a id="ref-for-selectordef-adjacent③"></a>

<a id="ref-for-x②"></a>

In addition, [whitespace](https://www.w3.org/TR/css-syntax-3/#whitespace) is required on both sides of the [+](https://www.w3.org/TR/selectors4/#selectordef-adjacent) and - operators. (The [\*](https://www.w3.org/TR/css3-selectors/#x) and / operaters can be used without white space around them.)

<a id="ref-for-math-function⑦"></a>

UAs must support [math function](#math-function) expressions of at least 20 terms, where each `NUMBER`, `DIMENSION`, or `PERCENTAGE` is a term. If a <a id="ref-for-math-function⑧"></a>math function contains more than the supported number of terms, it must be treated as if it were invalid.

#### <a id="calc-type-checking"></a>9.1.2.  Type Checking

<a id="ref-for-math-function⑨"></a>

<a id="ref-for-length-value①⑧"></a>

<a id="ref-for-number-value①⑥"></a>

A [math function](#math-function) can be many possible types, such as [\<length\>](#length-value), [\<number\>](#number-value), etc., depending on the expression it contains, as defined below. It can be used anywhere a value of that type is allowed.

<a id="ref-for-propdef-width⑦"></a>

<a id="ref-for-length-value①⑨"></a>

<a id="ref-for-math-function①⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-40312766"></a> For example, the [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width) property accepts [\<length\>](#length-value) values, so a [math function](#math-function) that resolves to a <a id="ref-for-length-value②⓪"></a>\<length\>, such as calc(5px + 1em), can be used in <a id="ref-for-propdef-width⑧"></a>width.

<a id="ref-for-math-function①①"></a>

<a id="ref-for-number-value①⑦"></a>

<a id="ref-for-integer-value⑧"></a>

Additionally, [math functions](#math-function) that resolve to [\<number\>](#number-value) can be used in any place that only accepts [\<integer\>](#integer-value). (It gets rounded to the nearest integer, as specified in [§9.1.4 Range Checking](#calc-range).)

Operators form sub-expressions, which gain types based on their arguments.

<a id="ref-for-length-value②①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In previous versions of this specification, multiplication and division were limited in what arguments they could take, to avoid producing more complex intermediate results (such as 1px \* 1em, which is [\<length\>](#length-value)²) and to make division-by-zero detectable at parse time. This version now relaxes those restrictions.

<a id="ref-for-math-function①②"></a>

[Math functions](#math-function) allow arbitrary expressions, so long as the expression as a whole resolves to a singular unit according to the following rules:

- <a id="ref-for-cssnumericvalue-add-two-types"></a>

  <a id="ref-for-math-function①③"></a>

  <a id="ref-for-cssnumericvalue-type"></a>

  At a + or - sub-expression, attempt to [add the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-add-two-types) of the left and right arguments. If this returns failure, the entire [math function](#math-function) is invalid. Otherwise, the sub-expression’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is the returned type.

- <a id="ref-for-cssnumericvalue-multiply-two-types"></a>

  <a id="ref-for-cssnumericvalue-type①"></a>

  At a \* sub-expression, [multiply the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-multiply-two-types) of the left and right arguments. The sub-expression’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is the returned result.

- <a id="ref-for-cssnumericvalue-type②"></a>

  <a id="ref-for-cssnumericvalue-invert-a-type"></a>

  At a / sub-expression, let <var>left type</var> be the result of finding the [types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of its left argument, and <var>right type</var> be the result of finding the <a id="ref-for-cssnumericvalue-type③"></a>types of its right argument and then [inverting](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-invert-a-type) it.

  <a id="ref-for-cssnumericvalue-type④"></a>

  <a id="ref-for-cssnumericvalue-multiply-two-types①"></a>

  The sub-expression’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is the result of [multiplying](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-multiply-two-types) the <var>left type</var> and <var>right type</var>.

- <a id="ref-for-cssnumericvalue-type⑤"></a>

  Anything else is a terminal value, whose [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is determined based on its CSS type:

  <a id="ref-for-number-value①⑧"></a>

  [\<number\>](#number-value)

  <a id="ref-for-integer-value⑨"></a>

  [\<integer\>](#integer-value)

  <a id="ref-for-cssnumericvalue-type⑥"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ \]» (empty map)

  <a id="ref-for-length-value②②"></a>

  [\<length\>](#length-value)

  <a id="ref-for-cssnumericvalue-type⑦"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "length" → 1 \]»

  <a id="ref-for-angle-value⑨"></a>

  [\<angle\>](#angle-value)

  <a id="ref-for-cssnumericvalue-type⑧"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "angle" → 1 \]»

  <a id="ref-for-time-value⑦"></a>

  [\<time\>](#time-value)

  <a id="ref-for-cssnumericvalue-type⑨"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "time" → 1 \]»

  <a id="ref-for-frequency-value⑦"></a>

  [\<frequency\>](#frequency-value)

  <a id="ref-for-cssnumericvalue-type①⓪"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "frequency" → 1 \]»

  <a id="ref-for-resolution-value⑤"></a>

  [\<resolution\>](#resolution-value)

  <a id="ref-for-cssnumericvalue-type①①"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "resolution" → 1 \]»

  <a id="ref-for-typedef-flex①"></a>

  [\<flex\>](https://www.w3.org/TR/css-grid-1/#typedef-flex)

  <a id="ref-for-cssnumericvalue-type①②"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "flex" → 1 \]»

  <a id="ref-for-percentage-value②⑤"></a>

  [\<percentage\>](#percentage-value)

  <a id="ref-for-math-function①④"></a>

  <a id="ref-for-percentage-value②⑥"></a>

  <a id="ref-for-propdef-width⑨"></a>

  <a id="ref-for-length-value②③"></a>

  <a id="ref-for-number-value①⑨"></a>

  <a id="ref-for-cssnumericvalue-type①③"></a>

  If, in the context in which the [math function](#math-function) is placed, [\<percentage\>](#percentage-value)s are resolved relative to another type of value (such as in [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width), where <a id="ref-for-percentage-value②⑦"></a>\<percentage\> is resolved against a [\<length\>](#length-value)), and that other type is <em>not</em> [\<number\>](#number-value), the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is determined as the other type.

  <a id="ref-for-cssnumericvalue-type①④"></a>

  Otherwise, the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "percent" → 1 \]».

  anything else

  <a id="ref-for-math-function①⑤"></a>

  The [math function](#math-function) is invalid.

  <a id="ref-for-cssnumericvalue-percent-hint"></a>

  In all cases, the associated [percent hint](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-percent-hint) is null.

- <a id="ref-for-cssnumericvalue-type①⑤"></a>

  <a id="ref-for-funcdef-calc⑧"></a>

  <a id="ref-for-funcdef-min⑧"></a>

  <a id="ref-for-funcdef-max⑧"></a>

  <a id="ref-for-funcdef-clamp⑧"></a>

  <a id="ref-for-cssnumericvalue-add-two-types①"></a>

  <a id="ref-for-math-function①⑥"></a>

  The [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of a [calc()](#funcdef-calc) expression is the <a id="ref-for-cssnumericvalue-type①⑥"></a>type of its contained expression. The <a id="ref-for-cssnumericvalue-type①⑦"></a>type of a [min()](#funcdef-min), [max()](#funcdef-max), or [clamp()](#funcdef-clamp) expression is the result of [adding the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-add-two-types) of its comma-separated expressions. If the result is failure, the entire [math function](#math-function) is invalid.

<a id="ref-for-math-function①⑦"></a>

<a id="ref-for-number-value②⓪"></a>

<a id="ref-for-length-value②④"></a>

<a id="ref-for-angle-value①⓪"></a>

<a id="ref-for-time-value⑧"></a>

<a id="ref-for-frequency-value⑧"></a>

<a id="ref-for-resolution-value⑥"></a>

<a id="ref-for-typedef-flex②"></a>

<a id="ref-for-percentage-value②⑧"></a>

<a id="ref-for-cssnumericvalue-type①⑧"></a>

<a id="ref-for-cssnumericvalue-match"></a>

A [math function](#math-function) resolves to [\<number\>](#number-value), [\<length\>](#length-value), [\<angle\>](#angle-value), [\<time\>](#time-value), [\<frequency\>](#frequency-value), [\<resolution\>](#resolution-value), [\<flex\>](https://www.w3.org/TR/css-grid-1/#typedef-flex), or [\<percentage\>](#percentage-value) according to which of those productions its [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) [matches](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-match). (These categories are mutually exclusive.) If it can’t <a id="ref-for-cssnumericvalue-match①"></a>match any of these, the <a id="ref-for-math-function①⑧"></a>math function is invalid.

<a id="ref-for-math-function①⑨"></a>

Division by zero is possible, which introduces certain complications. [Math functions](#math-function) follow IEEE-754 semantics for these operations:

- Dividing a positive value by zero produces +∞.

- Dividing a negative value by zero produces −∞.

- Adding or subtracting ±∞ to anything produces the appropriate infinity, unless a following rule would define it as producing NaN.

- Multiplying any value by ±∞ produces the appropriate infinity, unless a following rule would define it as producing NaN.

- Dividing any value by ±∞ produces zero, unless a following rule would define it as producing NaN.

- Dividing zero by zero, dividing ±∞ by ±∞, multiplying 0 by ±∞, adding +∞ to −∞ (or the equivalent subtractions) produces NaN.

- Any operation with at least one NaN argument produces NaN.

Additionally, IEEE-754 introduces the concept of "negative zero", which must be tracked within a calculation and between nested calculations:

- Negative zero (0<sup>-</sup>) can be produced literally by negating a zero (-0), or by a multiplication or division that produces zero with exactly one negative argument (such as -5 \* 0 or 1 / (-1 / 0)).

  <a id="ref-for-math-function②⓪"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: Note that, outside of [math functions](#math-function), -0 just produces a "standard" zero, identical to 0—CSS as a whole doesn’t recognize the concept of signed zeros. Negative zeros also don’t escape a <a id="ref-for-math-function②①"></a>math function; as detailed below, they’re "censored" away into an "unsigned" zero.

- -0 + -0 or -0 - 0 produces 0<sup>-</sup>. All other additions or subtractions that would produce a zero produce 0<sup>+</sup>.

- Multiplying or dividing 0<sup>-</sup> with a positive number (including 0<sup>+</sup>) produces a negative result (either 0<sup>-</sup> or −∞), while multiplying or dividing 0<sup>-</sup> with a negative number produces a positive result.

  (In other words, multiplying or dividing with 0<sup>-</sup> follows standard sign rules.)

- When comparing 0<sup>+</sup> and 0<sup>-</sup>, 0<sup>-</sup> is less than 0<sup>+</sup>. For example, min(0, -0) must produce 0<sup>-</sup>, max(0, -0) must produce 0<sup>+</sup>, and clamp(0, -0, 1) must produce 0<sup>+</sup>.

<a id="ref-for-math-function②②"></a>

<a id="ref-for-top-level-calculation"></a>

If a <a id="top-level-calculation"></a>top-level calculation (a [math function](#math-function) not nested inside of another <a id="ref-for-math-function②③"></a>math function) would produce a NaN, it instead produces +∞. If a [top-level calculation](#top-level-calculation) would produce 0<sup>-</sup>, it instead produces the standard "unsigned" zero.

<a id="ref-for-top-level-calculation①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3d047d16"></a> For example, calc(-5 \* 0) produces an unsigned zero—the calculation resolves to 0<sup>-</sup>, but as it’s a [top-level calculation](#top-level-calculation), it’s then censored to an unsigned zero.
>
> <a id="ref-for-top-level-calculation②"></a>
>
> On the other hand, calc(1 / calc(-5 \* 0)) produces −∞, same as calc(1 / (-5 \* 0))—the inner calc resolves to 0<sup>-</sup>, and as it’s not a [top-level calculation](#top-level-calculation), it passes it up unchanged to the outer calc to produce −∞. If it was censored into an unsigned zero, it would instead produce +∞.

<a id="ref-for-math-function②④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Algebraic simplifications do not affect the validity of a [math function](#math-function) or its resolved type. For example, calc(5px - 5px + 10s) and calc(0 \* 5px + 10s) are both invalid due to the attempt to add a length and a time.

<a id="ref-for-percentage-value②⑨"></a>

<a id="ref-for-number-value②①"></a>

<a id="ref-for-propdef-opacity"></a>

<a id="ref-for-typedef-dimension③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that [\<percentage\>](#percentage-value)s relative to [\<number\>](#number-value)s, such as in [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), are not <em>combinable</em> with those numbers—<a id="ref-for-propdef-opacity①"></a>opacity: calc(.25 + 25%) is invalid. Allowing this causes significant problems with "unit algebra" (allowing multiplication/division of [\<dimension\>](#typedef-dimension)s), and in every case so far, doesn’t provide any new functionality. (For example, <a id="ref-for-propdef-opacity②"></a>opacity: 25% is identical to <a id="ref-for-propdef-opacity③"></a>opacity: .25; it’s just a trivial syntax transform.) You can still perform other operations with them, such as <a id="ref-for-propdef-opacity④"></a>opacity: calc(100% / 3);, which is valid.

<a id="ref-for-typedef-number-token③"></a>

<a id="ref-for-number-value②②"></a>

<a id="ref-for-integer-value①⓪"></a>

<a id="ref-for-length-value②⑤"></a>

<a id="ref-for-math-function②⑤"></a>

<a id="ref-for-propdef-width①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token)s are always interpreted as [\<number\>](#number-value)s or [\<integer\>](#integer-value)s, "unitless 0" [\<length\>](#length-value)s aren’t supported in [math functions](#math-function). That is, [width: calc(0 + 5px);](https://www.w3.org/TR/CSS21/visudet.html#propdef-width) is invalid, because it’s trying to add a <a id="ref-for-number-value②③"></a>\<number\> to a <a id="ref-for-length-value②⑥"></a>\<length\>, even though both <a id="ref-for-propdef-width①①"></a>width: 0; and <a id="ref-for-propdef-width①②"></a>width: 5px; are valid.

<a id="ref-for-number-value②④"></a>

<a id="ref-for-length-value②⑦"></a>

<a id="ref-for-propdef-line-height⑨"></a>

<a id="ref-for-propdef-tab-size"></a>

<a id="ref-for-funcdef-calc⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Altho there are a few properties in which a bare [\<number\>](#number-value) becomes a [\<length\>](#length-value) at used-value time (specifically, [line-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-line-height) and [tab-size](https://www.w3.org/TR/css-text-3/#propdef-tab-size)), <a id="ref-for-number-value②⑤"></a>\<number\>s never become "length-like" in [calc()](#funcdef-calc). They always stay as <a id="ref-for-number-value②⑥"></a>\<number\>s.

#### <a id="calc-computed-value"></a>9.1.3.  Computed Value

<a id="ref-for-funcdef-calc①⓪"></a>

<a id="ref-for-funcdef-min⑨"></a>

<a id="ref-for-funcdef-max⑨"></a>

<a id="ref-for-funcdef-clamp⑨"></a>

The computed value of a [calc()](#funcdef-calc) function is the expression with all components computed. The computed value of a [min()](#funcdef-min), [max()](#funcdef-max), or [clamp()](#funcdef-clamp) function is the comma-separated list of expressions, with each expression having all its component computed.

<a id="ref-for-math-function②⑥"></a>

Where percentages are not resolved at computed-value time, they are not resolved in [math functions](#math-function), e.g. calc(100% - 100% + 1em) resolves to calc(1em + 0%), not to 1em. If there are special rules for computing percentages in a value (e.g. [the height property](https://www.w3.org/TR/CSS21/visudet.html#the-height-property)), they apply whenever a <a id="ref-for-math-function②⑦"></a>math function contains percentages.

<a id="ref-for-math-function②⑧"></a>

<a id="ref-for-number-value②⑦"></a>

<a id="ref-for-percentage-value③⓪"></a>

<a id="ref-for-typedef-dimension④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The serialization rules do not preserve the structure of the computation, so implementations can simplify the expressions further than what is required here when storing the values internally; in particular, all [math function](#math-function) expressions can be reduced to a sum of a [\<number\>](#number-value), a [\<percentage\>](#percentage-value), and some [\<dimension\>](#typedef-dimension)s, eliminating all multiplication or division, and combining terms with identical units.
>
> <a id="ref-for-math-function②⑨"></a>
>
> <a id="ref-for-number-value②⑧"></a>
>
> <a id="ref-for-percentage-value③①"></a>
>
> <a id="ref-for-typedef-dimension⑤"></a>
>
> At this time, all units can be absolutized to a single unit per type at computed-value time, so at that point the [math function](#math-function) can be reduced to just a [\<number\>](#number-value), a [\<percentage\>](#percentage-value), and a single absolute [\<dimension\>](#typedef-dimension) of the appropriate type, per expression.

<a id="ref-for-propdef-font-size⑧"></a>

<a id="ref-for-computed-value⑦"></a>

<a id="ref-for-font-relative-length"></a>

<a id="ref-for-propdef-background-position②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-023dad93"></a> For example, whereas [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) computes percentage values at [computed value](https://www.w3.org/TR/css-cascade-4/#computed-value) time so that [font-relative length](#font-relative-length) units can be computed, [background-position](https://www.w3.org/TR/css3-background/#propdef-background-position) has layout-dependent behavior for percentage values, and thus does not resolve percentages until used-value time.
>
> <a id="ref-for-propdef-background-position③"></a>
>
> <a id="ref-for-funcdef-calc①①"></a>
>
> <a id="ref-for-propdef-font-size⑨"></a>
>
> Due to this, [background-position](https://www.w3.org/TR/css3-background/#propdef-background-position) computation preserves the percentage in a [calc()](#funcdef-calc) whereas [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) will compute such expressions directly into a length.

<a id="ref-for-valdef-width-auto"></a>

Given the complexities of width and height calculations on table cells and table elements, math expressions mixing both percentages and lengths for widths and heights on table columns, table column groups, table rows, table row groups, and table cells in both auto and fixed layout tables MAY be treated as if [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) had been specified.

#### <a id="calc-range"></a>9.1.4.  Range Checking

<a id="ref-for-math-function③⓪"></a>

<a id="ref-for-computed-value⑧"></a>

<a id="ref-for-used-value③"></a>

<a id="ref-for-specified-value②"></a>

Parse-time range-checking of values is not performed within [math functions](#math-function), and therefore out-of-range values do not cause the declaration to become invalid. However, the value resulting from an expression must be clamped to the range allowed in the target context. Clamping is performed on [computed values](https://www.w3.org/TR/css-cascade-4/#computed-value) to the extent possible, and also on [used values](https://www.w3.org/TR/css-cascade-4/#used-value) if computation was unable to sufficiently simplify the expression to allow range-checking. (Clamping is not performed on [specified values](https://www.w3.org/TR/css-cascade-4/#specified-value).)

<a id="ref-for-funcdef-calc①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This requires all contexts accepting [calc()](#funcdef-calc) to define their allowable values as a closed (not open) interval.

<a id="ref-for-propdef-animation-iteration-count"></a>

<a id="ref-for-math-function③①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: By definition, ±∞ are outside the allowed range for any property, and will clamp to the minimum/maximum value allowed. Even for properties that explicitly allow infinity as a keyword value, such as [animation-iteration-count](https://www.w3.org/TR/css3-animations/#propdef-animation-iteration-count), will end up clamping ±∞, as [math functions](#math-function) can’t resolve to keyword values; the <em>numeric</em> part of the property’s syntax still has a minimum/maximum value.

<a id="ref-for-math-function③②"></a>

<a id="ref-for-number-value②⑨"></a>

<a id="ref-for-integer-value①①"></a>

<a id="ref-for-computed-value⑨"></a>

<a id="ref-for-used-value④"></a>

Additionally, if a [math function](#math-function) that resolves to [\<number\>](#number-value) is used somewhere that only accepts [\<integer\>](#integer-value), the [computed value](https://www.w3.org/TR/css-cascade-4/#computed-value) and [used value](https://www.w3.org/TR/css-cascade-4/#used-value) are rounded to the nearest integer, in the same manner as clamping, above. The rounding method must be the same as is used for animations of integer values.

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
> <a id="ref-for-propdef-width①③"></a>
>
> <a id="ref-for-funcdef-calc①③"></a>
>
> Note however that [width: -5px](https://www.w3.org/TR/CSS21/visudet.html#propdef-width) is not equivalent to <a id="ref-for-propdef-width①④"></a>width: calc(-5px)! Out-of-range values <em>outside</em> [calc()](#funcdef-calc) are syntactically invalid, and cause the entire declaration to be dropped.

#### <a id="calc-serialize"></a>9.1.5.  Serialization

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f5bc4b00"></a> This section is still [under discussion](https://lists.w3.org/Archives/Member/w3c-css-wg/2016AprJun/0239.html).

<a id="ref-for-funcdef-calc①④"></a>

To <a id="serialize-a-calc-value"></a>serialize a [calc()](#funcdef-calc) value

1.  <a id="ref-for-math-function-simplify-an-expression"></a>

    [Simplify the expression](#math-function-simplify-an-expression) inside of it.

2.  <a id="ref-for-number-value③⓪"></a>

    <a id="ref-for-typedef-dimension⑥"></a>

    <a id="ref-for-percentage-value③②"></a>

    <a id="ref-for-computed-value①⓪"></a>

    <a id="ref-for-funcdef-calc①⑤"></a>

    If this simplification process results in only a single value (one [\<number\>](#number-value), one [\<dimension\>](#typedef-dimension), or one [\<percentage\>](#percentage-value)), and the value being serialized is a [computed value](https://www.w3.org/TR/css-cascade-4/#computed-value) or later, serialize it just as that one value, without the [calc()](#funcdef-calc) wrapper. If this value is outside the allowed range for the context, it must be clamped to the nearest allowed value.

3.  <a id="ref-for-math-function-serialize-a-summation"></a>

    Otherwise, [serialize the summation](#math-function-serialize-a-summation), prefix the result with "calc(" and suffix it with ")", then return it.

<a id="ref-for-funcdef-min①⓪"></a>

<a id="ref-for-funcdef-max①⓪"></a>

To <a id="serialize-a-min-value"></a>serialize a [min()](#funcdef-min) or [max()](#funcdef-max) value:

1.  <a id="ref-for-math-function-simplify-an-expression①"></a>

    For each comma-separated expression inside of it, [simplify the expression](#math-function-simplify-an-expression).

2.  Let <var>s</var> initially be "min(" or "max(", as appropriate.

3.  <a id="ref-for-math-function-serialize-a-summation①"></a>

    [Serialize each summation](#math-function-serialize-a-summation), then join them into a single string, with ", " between each term. Append the result to <var>s</var>.

4.  Append ")" to <var>s</var>, then return it.

To <a id="math-function-simplify-an-expression"></a>simplify an expression:

1.  <a id="ref-for-funcdef-calc①⑥"></a>

    Replace any [calc()](#funcdef-calc) values with parentheses containing their contents.

2.  Resolve all multiplications and divisions.

3.  Combine identical units.

4.  <a id="ref-for-funcdef-min①①"></a>

    <a id="ref-for-funcdef-max①①"></a>

    Recurse into [min()](#funcdef-min) or [max()](#funcdef-max) values.

5.  Return the result.

<a id="ref-for-em③"></a>

<a id="ref-for-px⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The value-computation process can transform disparate units into identical ones. For example, [em](#em) and [px](#px) are obviously different at specified-value time, but at computed-value time they’re both absolutized to <a id="ref-for-px①⓪"></a>px.

<a id="ref-for-math-function③③"></a>

The result must be a summation of unique units and/or [math functions](#math-function). (Terms with a value of zero <strong>must</strong> be preserved in this summation.)

To <a id="math-function-serialize-a-summation"></a>serialize a summation:

1.  Sort the terms in the following order:

    1.  The number, if present

    2.  The percentage, if present

    3.  <a id="ref-for-ascii-case-insensitive③"></a>

        The dimensions, ordered by their units [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) alphabetically

    4.  <a id="ref-for-funcdef-min①②"></a>

        <a id="ref-for-funcdef-max①②"></a>

        The [min()](#funcdef-min) and [max()](#funcdef-max) functions, in the order they appeared in the original expression.

2.  Serialize all the terms, then join them into a single string, with " + " between each term. Return the result.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c3db2475"></a> For example, calc(20px + 30px) would serialize as calc(50px) as a specified value, or as 50px as a computed value.
>
> <a id="ref-for-funcdef-calc①⑦"></a>
>
> A value like calc(20px + 0%) would serialize as calc(0% + 20px), maintaining both terms in the serialized value. (It’s important to maintain zero-valued terms, so the [calc()](#funcdef-calc) doesn’t suddenly "change shape" in the middle of a transition when one of the values happens to have a zero value temporarily. This also removes the need to "pick a unit" when all the terms are zero.)
>
> <a id="ref-for-em④"></a>
>
> <a id="ref-for-funcdef-calc①⑧"></a>
>
> A value like calc(20px + 2em) would serialize as calc(2em + 20px) as a specified value (maintaining both units as they’re incompatible at specified-value time, but sorting them alphabetically), or as something like 52px as a computed value ([em](#em) values are converted to absolute lengths at computed-value time, so assuming 1em = 16px, they combine into 52px, which then drops the [calc()](#funcdef-calc) wrapper.)

See [\[CSSOM\]](#biblio-cssom) for further information on serialization.

<a id="ref-for-funcdef-toggle①"></a>

### <a id="toggle-notation"></a>9.2.  Toggling Between Values: [toggle()](#funcdef-toggle)

The <a id="funcdef-toggle"></a>toggle() expression allows descendant elements to cycle over a list of values instead of inheriting the same value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-43c760b5"></a> The following example makes `<em>` elements italic in general, but makes them normal if they’re inside something that’s italic:
>
> ```text
> em { font-style: toggle(italic, normal); }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fa2213e4"></a> The following example cycles markers for nested lists, so that a top level list has disc-shaped markers, but nested lists use circle, then square, then box, and then repeat through the list of marker shapes, starting again (for the 5th list deep) with disc.
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

<a id="ref-for-funcdef-attr②"></a>

<a id="ref-for-funcdef-calc①⑨"></a>

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

To determine the computed value of [toggle()](#funcdef-toggle), first evaluate each argument as if it were the sole value of the property in which <a id="ref-for-funcdef-toggle⑧"></a>toggle() is placed to determine the computed value that each represents, called <var>C<sub>n</sub></var> for the <var>n</var>-th argument to <a id="ref-for-funcdef-toggle⑨"></a>toggle(). Then, compare the property’s [inherited value](https://www.w3.org/TR/css-cascade-4/#inherited-value) with each <var>C<sub>n</sub></var>. For the earliest <var>C<sub>n</sub></var> that matches the <a id="ref-for-inherited-value①"></a>inherited value, the computed value of <a id="ref-for-funcdef-toggle①⓪"></a>toggle() is <var>C<sub>n+1</sub></var>. If the match was the last argument in the list, or there was no match, the computed value of <a id="ref-for-funcdef-toggle①①"></a>toggle() is the computed value that the first argument represents.

<a id="ref-for-funcdef-toggle①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that repeating values in a [toggle()](#funcdef-toggle) short-circuits the list. For example toggle(1em, 2em, 1em, 4em) will be equivalent to toggle(1em, 2em).

<a id="ref-for-funcdef-toggle①③"></a>

<a id="ref-for-valdef-all-inherit④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That [toggle()](#funcdef-toggle) explicitly looks at the computed value of the parent, so it works even on non-inherited properties. This is similar to the [inherit](https://www.w3.org/TR/css-cascade-4/#valdef-all-inherit) keyword, which works even on non-inherited properties.

<a id="ref-for-propdef-background-position④"></a>

<a id="ref-for-propdef-background-position⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That the [computed value](https://www.w3.org/TR/CSS21/cascade.html#computed-value) of a property is an abstract set of values, not a particular serialization [\[CSS21\]](#biblio-css21), so comparison between computed values should always be unambiguous and have the expected result. For example, a Level 2 [background-position](https://www.w3.org/TR/CSS21/colors.html#propdef-background-position) computed value is just two offsets, each represented as an absolute length or a percentage, so the declarations [background-position: top center](https://www.w3.org/TR/css3-background/#propdef-background-position) and <a id="ref-for-propdef-background-position⑥"></a>background-position: 50% 0% produce identical computed values. If the "Computed Value" line of a property definition seems to define something ambiguous or overly strict, please [provide feedback](#status) so we can fix it.

<a id="ref-for-funcdef-toggle①④"></a>

<a id="ref-for-shorthand-property"></a>

If [toggle()](#funcdef-toggle) is used on a [shorthand property](https://www.w3.org/TR/css-cascade-4/#shorthand-property), it sets each of its longhands to a <a id="ref-for-funcdef-toggle①⑤"></a>toggle() value with arguments corresponding to what the longhand would have received had each of the original <a id="ref-for-funcdef-toggle①⑥"></a>toggle() arguments been the sole value of the <a id="ref-for-shorthand-property①"></a>shorthand.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-70a83e83"></a> For example, the following shorthand declaration:
>
> ```text
> margin: toggle(1px 2px, 4px, 1px 5px 3px);
> ```
>
> is equivalent to the following longhand declarations:
>
> ```text
> margin-top:    toggle(1px, 4px, 1px);
> margin-right:  toggle(2px, 4px, 5px);
> margin-bottom: toggle(1px, 4px, 3px);
> margin-left:   toggle(2px, 4px, 5px);
> ```
>
> Note that, since 1px appears twice in the top and bottom margins, they will cycle between only two values while the left and right margins cycle through three. In other words, the declarations above will yield the same computed values as the longhand declarations below:
>
> ```text
> margin-top:    toggle(1px, 3px);
> margin-right:  toggle(2px, 4px, 5px);
> margin-bottom: toggle(1px, 3px);
> margin-left:   toggle(2px, 4px, 5px);
> ```
>
> which may not be what was intended.

<a id="ref-for-funcdef-attr③"></a>

### <a id="attr-notation"></a>9.3.  Attribute References: [attr()](#funcdef-attr)

The <a id="funcdef-attr"></a>attr() function is allowed as a component value in properties applied to an element or pseudo-element. It returns the value of an attribute on the element. If used on a pseudo-element, it returns the value of the attribute on the pseudo-element’s originating element.

<a id="ref-for-funcdef-attr④"></a>

The computed value of the [attr()](#funcdef-attr) expression is the value of the attribute with the specified name on the element, according to the rules given below.

<a id="ref-for-funcdef-attr⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In CSS2.1 [\[CSS21\]](#biblio-css21), the [attr()](#funcdef-attr) expression always returns a string. In CSS3, the <a id="ref-for-funcdef-attr⑥"></a>attr() expression can return many different types. The <a id="ref-for-funcdef-attr⑦"></a>attr() expression cannot return everything, for example it cannot do counters, named strings, quotes, or keyword values such as auto, nowrap, or baseline. This is intentional, as the intent of the <a id="ref-for-funcdef-attr⑧"></a>attr() expression is not to make it possible to describe a presentational language’s formatting using CSS, but to enable CSS to take semantic data into account.

<a id="ref-for-funcdef-attr⑨"></a>

The new syntax for the [attr()](#funcdef-attr) expression is:

<a id="ref-for-typedef-attr-name"></a>

<a id="ref-for-typedef-type-or-unit"></a>

<a id="ref-for-typedef-attr-fallback"></a>

```text
attr( <attr-name> <type-or-unit>? [ , <attr-fallback> ]? )
```
<a id="ref-for-typedef-attr-name①"></a>

where <a id="typedef-attr-name"></a>\<attr-name\> is a [CSS qualified name](https://drafts.csswg.org/css3-namespace/#css-qnames) (the qname production in [\[CSS3NAMESPACE\]](#biblio-css3namespace)) that represents an attribute name. (In the absence of namespacing, this will just be a CSS identifier.) As with [attribute selectors](https://www.w3.org/TR/selectors/#attribute-selectors), the case-sensitivity of [\<attr-name\>](#typedef-attr-name) depends on the document language.

<a id="ref-for-valdef-type-or-value-string"></a>

The optional <a id="typedef-type-or-unit"></a>\<type-or-unit\> argument is a keyword drawn from the list below that tells the UA how to interpret the attribute value, and defines a type for the attr() expression. If omitted, [string](#valdef-type-or-value-string) is implied.

<a id="ref-for-typedef-type-or-unit①"></a>

The optional <a id="typedef-attr-fallback"></a>\<attr-fallback\> argument represents a fallback value, which is used if the named attribute is missing, or its value cannot be parsed into the given type or is invalid/out-of-range for the property. If it’s absent, the default value for the given [\<type-or-unit\>](#typedef-type-or-unit) (from the list below) is implied.

<a id="ref-for-typedef-toggle-value①"></a>

<a id="ref-for-funcdef-attr①⓪"></a>

<a id="ref-for-typedef-attr-fallback①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unlike [\<toggle-value\>](#typedef-toggle-value)s, an [attr()](#funcdef-attr) [\<attr-fallback\>](#typedef-attr-fallback) value may contain top-level commas, as it is always the last argument in the functional notation.

The attr() expression is only valid if:

- the attr() expression’s type is valid where the attr() expression is placed,

- the namespace prefix of the attribute name, if any, is defined,

- <a id="ref-for-typedef-attr-fallback②"></a>

  the [\<attr-fallback\>](#typedef-attr-fallback) is valid where the attr() expression is placed,

- <a id="ref-for-typedef-attr-fallback③"></a>

  the [\<attr-fallback\>](#typedef-attr-fallback) does not contain another attr() expression,

- <a id="ref-for-typedef-attr-fallback④"></a>

  and, if the attr() expression is not the sole component value of a property, the [\<attr-fallback\>](#typedef-attr-fallback) matches the attr()’s type

<a id="ref-for-px①①"></a>

<a id="ref-for-propdef-width①⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the default value need not be of the type given, if the attr() expression is the entire property value. For instance, if the type required of the attribute by the author is [px](#px), the default could still be auto, like in [width: attr(size px, auto);](https://www.w3.org/TR/CSS21/visudet.html#propdef-width).
>
> <a id="ref-for-propdef-box-shadow"></a>
>
> <a id="ref-for-px①②"></a>
>
> <a id="ref-for-shadow-inset"></a>
>
> If the attr() is used alongside other values to form the full property value, however, then the default value must match the attr()'s type. For example, [box-shadow: attr(size px, inset) 5px 10px blue;](https://www.w3.org/TR/css3-background/#propdef-box-shadow) is invalid, even though it would create a valid declaration if you substituted the attr() expression with either a [px](#px) length <em>or</em> the [inset](https://www.w3.org/TR/css3-background/#shadow-inset) keyword.

<a id="ref-for-typedef-type-or-unit②"></a>

<a id="ref-for-valdef-type-or-value-string①"></a>

If the specified attribute exists on the element, the value of the attribute must be parsed as required by the [\<type-or-unit\>](#typedef-type-or-unit) argument (as defined in the list below). Unless the type is [string](#valdef-type-or-value-string), it must first be stripped of leading and trailing [white space](https://www.w3.org/TR/css-syntax/#whitespace). The resulting value is the attr() expression’s value. If the value did not parse as required, the attr() expression’s value is its fallback value.

<a id="ref-for-typedef-type-or-unit③"></a>

The [\<type-or-unit\>](#typedef-type-or-unit) keywords are:

<a id="valdef-type-or-value-string"></a>string

<a id="ref-for-string-value④"></a>

The attribute value is taken as the contents of a CSS [\<string\>](#string-value). The default is the empty string.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This does not reparse the attribute value with the CSS parser. So, for example, an attribute whose value is "&#x5C;51" will produce a string containing those three characters, not a string containing "Q" (the character that the escape would evaluate to).

<a id="valdef-type-or-value-color"></a>color

<a id="ref-for-valdef-color-currentcolor①"></a>

<a id="ref-for-valuea-def-color⑤"></a>

<a id="ref-for-typedef-ident-token①"></a>

<a id="ref-for-typedef-hash-token"></a>

The attribute value must parse as a [\<hash-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-hash-token) or [\<ident-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-ident-token), and be successfully interpreted as a [\<color\>](https://www.w3.org/TR/css3-color/#valuea-def-color). The default is [currentcolor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor).

<a id="valdef-type-or-value-url"></a>url

<a id="ref-for-funcdef-url①③"></a>

<a id="ref-for-string-value⑤"></a>

The attribute value is taken as the contents of a CSS [\<string\>](#string-value). It is interpreted as a quoted string within the [url()](#funcdef-url) notation. The default is about:invalid, which is a URL defined ([in Appendix A](#about-invalid)) to point to a non-existent document with a generic error condition. Relative URLs must be made absolute according to the rules of the document language as applied to URLs originating from the element; they are not relative to the style sheet.

<a id="valdef-type-or-value-integer"></a>integer

<a id="ref-for-integer-value①②"></a>

<a id="ref-for-typedef-number-token④"></a>

The attribute value must parse as a [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token), and be successfully interpreted as an [\<integer\>](#integer-value). The default is 0, or else the property’s minimum value if 0 is not valid for the property. The default must also be used if the property in question only accepts integers within a certain range and the attribute is out of range.

<a id="valdef-type-or-value-number"></a>number

<a id="ref-for-number-value③①"></a>

<a id="ref-for-typedef-number-token⑤"></a>

The attribute value must parse as a [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token), and is interpreted as an [\<number\>](#number-value). The default is 0, or else the property’s minimum value if 0 is not valid for the property. The default must also be used if the property in question only accepts integers within a certain range and the attribute is out of range.

<a id="valdef-type-or-value-length"></a>length

<a id="valdef-type-or-value-angle"></a>angle

<a id="valdef-type-or-value-time"></a>time

<a id="valdef-type-or-value-frequency"></a>frequency

<a id="ref-for-typedef-dimension-token②"></a>

The attribute value must parse as a [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token), and be successfully interpreted as the specified type. The default is 0 in the relevant units, or else the property’s minimum value if 0 in the relevant units is not valid for the property. The default must also be used if the property in question only accepts values within a certain range (e.g. positive lengths or angles from 0 to 90deg) and the attribute is out of range (e.g. a negative length or 180deg). If the unit is a relative length, it must be computed to an absolute length.

<a id="valdef-type-or-value"></a>%

<a id="ref-for-frequency-value⑨"></a>

<a id="ref-for-time-value⑨"></a>

<a id="ref-for-angle-value①①"></a>

<a id="ref-for-length-value②⑧"></a>

A keyword matching one of the [\<length\>](#length-value), [\<angle\>](#angle-value), [\<time\>](#time-value), or [\<frequency\>](#frequency-value) units

<a id="ref-for-dimension①⓪"></a>

<a id="ref-for-typedef-number-token⑥"></a>

The attribute value must parse as a [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token), and is interpreted as a [dimension](#dimension) with the specified unit. The default is 0 in the relevant units, or else the property’s minimum value if 0 in the relevant units is not valid for the property. The default must also be used if the property in question only accepts values within a certain range (e.g. positive lengths or angles from 0 to 90deg) and the attribute is out of range (e.g. a negative length or 180deg). If the unit is a relative length, it must be computed to an absolute length.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6ce916fb"></a> This example shows the use of attr() to visually illustrate data in an XML file:
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
>   width: attr(length em); /* default 0 */
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
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-38451edc"></a> All of the following examples are invalid and would cause a parse-time error, and thus cause the relevant declaration—in this case all of them—to be ignored:
>
> ```text
> content: attr(title color); /* 'content' doesn’t accept colors */
> 
> content: attr(end-of-quote string, inherit) close-quote;
> /* the 'inherit' value is not allowed there, since the result would be
> 'inherit close-quote', which is invalid. */
> 
> margin: attr(vertical length) attr(horizontal deg);
> /* deg units are not valid at that point */
> 
> color: attr(color); /* 'color' doesn’t accept strings */
> ```
<a id="ref-for-funcdef-attr①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [attr()](#funcdef-attr) expression cannot currently fall back onto another attribute. Future versions of CSS may extend <a id="ref-for-funcdef-attr①②"></a>attr() in this direction.

## <a id="iana"></a>10.  Appendix A: IANA Considerations

### <a id="about-invalid"></a>10.1.  Registration for the `about:invalid` URL scheme

This sections defines and registers the `about:invalid` URL, in accordance with the registration procedure defined in [\[RFC6694\]](#biblio-rfc6694).

The official record of this registration can be found at [http&#x3A;&#x2F;&#x2F;www&#x2E;iana&#x2E;org&#x2F;assignments&#x2F;about-uri-tokens&#x2F;about-uri-tokens&#x2E;xhtml](http://www.iana.org/assignments/about-uri-tokens/about-uri-tokens.xhtml)&#x2E;



| Field               | Definition                                                                                                                                                                                                   |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Registered Token &#xA;      </strong> | <code>invalid</code>                                                                                                                                                                                            |
| <strong>Intended Usage &#xA;      </strong> | The <code>about:invalid</code> URL references a non-existent document with a generic error condition. It can be used when a URL is necessary, but the default value shouldn’t be resolveable as any type of document. |
| <strong>Contact/Change controller &#xA;      </strong> | CSS WG \<<www-style@w3.org>\> (on behalf of W3C)                                                                                                                                                             |
| <strong>Specification &#xA;      </strong> | [CSS Values and Units Module Level 3](https://www.w3.org/TR/css3-values/)                                                                                                                                    |



## <a id="acknowledgments"></a> Acknowledgments

Firstly, the editors would like to thank all of the contributors to the [previous level](https://www.w3.org/TR/css-values-3/#acknowledgements) of this module.

Secondly, we would like to acknowledge Koji Ishii and Xidorn Quan for their comments and suggestions, which have improved Level 4.

## <a id="changes"></a> Changes

Changes since the [10 October 2018 Working Draft](https://www.w3.org/TR/2018/WD-css-values-4-20181010/) consist of synchronizing with the [recent changes in CSS Level 3](https://www.w3.org/TR/2019/CR-css-values-3-20190131/#changes).

Changes since the [14 August 2018 Working Draft](https://www.w3.org/TR/2018/WD-css-values-4-20180814/):

- Added rules for interpolation per value type, and clarified computed values.

Changes since [Level 3](https://www.w3.org/TR/css-values-3/):

- <a id="ref-for-rlh⑤"></a>

  <a id="ref-for-lh④"></a>

  <a id="ref-for-cap③"></a>

  <a id="ref-for-ic②"></a>

  <a id="ref-for-valdef-length-vb③"></a>

  <a id="ref-for-valdef-length-vi③"></a>

  Added the [vi](#valdef-length-vi), [vb](#valdef-length-vb), [ic](#ic), [cap](#cap), [lh](#lh) and [rlh](#rlh) units.

- <a id="ref-for-funcdef-clamp①⓪"></a>

  <a id="ref-for-funcdef-max①③"></a>

  <a id="ref-for-funcdef-min①③"></a>

  Added [min()](#funcdef-min), [max()](#funcdef-max), and [clamp()](#funcdef-clamp) functional notations.

- <a id="ref-for-funcdef-calc②⓪"></a>

  Added unit arithmetic to [calc()](#funcdef-calc).

- <a id="ref-for-funcdef-toggle①⑦"></a>

  Added [toggle()](#funcdef-toggle) (punted from level 3 originally).

- Added [unit algebra](#calc-type-checking), cribbing from [\[css-typed-om-1\]](#biblio-css-typed-om-1).

- <a id="ref-for-integer-value①③"></a>

  A non-integer in a calc() automatically rounds to the nearest integer when used where an [\<integer\>](#integer-value) is required.

- <a id="ref-for-math-function③④"></a>

  Defined [serialization](#calc-serialize) of [math functions](#math-function).

## <a id="sec-pri"></a> Security and Privacy Considerations

This specification mostly just defines units that are common to CSS specifications, and which present no security concerns.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Does URL handling have a security concern? Probably.

This specification defines units that expose the user’s screen size and default font size, but both are trivially observable from JS, so they do not constitutate a new privacy risk.

## <a id="conformance"></a> Conformance

### <a id="document-conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae2b6bc0"></a>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong> UAs MUST provide an accessible alternative. </strong>

### <a id="conform-classes"></a> Conformance classes

Conformance to this specification is defined for three conformance classes:

style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS2/conform.html#style-sheet).

renderer  
A [UA](https://www.w3.org/TR/CSS2/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

authoring tool  
A [UA](https://www.w3.org/TR/CSS2/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="conform-responsible"></a> Requirements for Responsible Implementation of CSS

The following sections define several conformance requirements for implementing CSS responsibly, in a way that promotes interoperability in the present and future.

#### <a id="conform-partial"></a> Partial Implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, <strong>CSS renderers <em>must</em> treat as invalid&#xA;&#x9;&#x9;(and <a href="https://www.w3.org/TR/CSS2/conform.html#ignore">ignore as appropriate</a>)&#xA;&#x9;&#x9;any at-rules, properties, property values, keywords, and other syntactic constructs&#xA;&#x9;&#x9;for which they have no usable level of support</strong>. In particular, user agents <em>must not</em> selectively ignore unsupported property values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

#### <a id="conform-testing"></a> Implementations of CR-level Features

Once a specification reaches the Candidate Recommendation stage, implementers should release an [unprefixed](https://www.w3.org/TR/CSS/#vendor-prefix) implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec, and should avoid exposing a prefixed variant of that feature.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [\|\|](#comb-any), in §2.2
- [,](#comb-comma), in §2.1
- [\#](#mult-comma), in §2.3
- [+](#mult-one-plus), in §2.3
- [?](#mult-opt), in §2.3
- [&#x26;&#x26;](#comb-all), in §2.2
- [\|](#comb-one), in §2.2
- [\*](#mult-zero-plus), in §2.3
- [!](#mult-req), in §2.3
- [%](#valdef-type-or-value), in §9.3
- [{A}](#mult-num), in §2.3
- [{A,B}](#mult-num-range), in §2.3
- [absolute length](#absolute-length), in §6.2
- [accumulate](#accumulation), in §3
- [accumulation](#accumulation), in §3
- [accumulation procedure](#accumulation), in §3
- [add](#addition), in §3
- [addition](#addition), in §3
- [addition procedure](#addition), in §3
- [advance measure](#length-advance-measure), in §6.1.1
- [anchor unit](#anchor-unit), in §6.2
- [\<angle\>](#angle-value), in §7.1
- [angle](#valdef-type-or-value-angle), in §9.3
- [\<angle-percentage\>](#typedef-angle-percentage), in §5.5
- [attr()](#funcdef-attr), in §9.3
- [\<attr-fallback\>](#typedef-attr-fallback), in §9.3
- [\<attr-name\>](#typedef-attr-name), in §9.3
- [bearing angle](#bearing-angle), in §7.1
- [calc()](#funcdef-calc), in §9.1
- [\<calc-product\>](#typedef-calc-product), in §9.1.1
- [\<calc-sum\>](#typedef-calc-sum), in §9.1.1
- [\<calc-value\>](#typedef-calc-value), in §9.1.1
- [canonical](#canonical-unit), in §5.3.1
- [canonical unit](#canonical-unit), in §5.3.1
- [cap](#cap), in §6.1.1
- [ch](#ch), in §6.1.1
- [clamp()](#funcdef-clamp), in §9.1
- [cm](#cm), in §6.2
- [color](#valdef-type-or-value-color), in §9.3
- [compatible](#compatible-units), in §5.3.1
- [compatible units](#compatible-units), in §5.3.1
- [CSS ident](#css-identifier), in §4
- [CSS identifier](#css-identifier), in §4
- [CSS-wide keywords](#css-wide-keywords), in §4.1.1
- [\<custom-ident\>](#identifier-value), in §4.2
- [deg](#deg), in §7.1
- [\<dimension\>](#typedef-dimension), in §5.3
- [dimension](#dimension), in §5.3
- [dpcm](#dpcm), in §7.4
- [dpi](#dpi), in §7.4
- [dppx](#dppx), in §7.4
- [em](#em), in §6.1.1
- [ex](#ex), in §6.1.1
- [font-relative lengths](#font-relative-length), in §6.1.1
- [frequency](#valdef-type-or-value-frequency), in §9.3
- [\<frequency\>](#frequency-value), in §7.3
- [\<frequency-percentage\>](#typedef-frequency-percentage), in §5.5
- [functional notation](#functional-notation), in §9
- [grad](#grad), in §7.1
- [Hz](#Hz), in §7.3
- [ic](#ic), in §6.1.1
- [\<ident\>](#typedef-ident), in §4
- [ident](#css-identifier), in §4
- [identifier](#css-identifier), in §4
- [in](#in), in §6.2
- [\<integer\>](#integer-value), in §5.1
- integer
  - [definition of](#integer), in §5.1
  - [value for \<type-or-value\>](#valdef-type-or-value-integer), in §9.3
- [interpolate](#interpolation), in §3
- [interpolation](#interpolation), in §3
- [interpolation procedure](#interpolation), in §3
- [keyword](#keyword), in §4.1
- [kHz](#kHz), in §7.3
- [\<length\>](#length-value), in §6
- [length](#valdef-type-or-value-length), in §9.3
- [\<length-percentage\>](#typedef-length-percentage), in §5.5
- [lh](#lh), in §6.1.1
- [local url flag](#url-local-url-flag), in §4.4.1.1
- [math function](#math-function), in §9.1
- [max()](#funcdef-max), in §9.1
- [min()](#funcdef-min), in §9.1
- [mm](#mm), in §6.2
- [ms](#ms), in §7.2
- [not additive](#not-additive), in §3
- [\<number\>](#number-value), in §5.2
- number
  - [definition of](#number), in §5.2
  - [value for \<type-or-value\>](#valdef-type-or-value-number), in §9.3
- [pc](#pc), in §6.2
- [\<percentage\>](#percentage-value), in §5.4
- [percentage](#percentage), in §5.4
- [physical unit](#physical-unit), in §6.2
- [pixel unit](#pixel-unit), in §6.2
- [\<position\>](#typedef-position), in §8.3
- [pt](#pt), in §6.2
- [px](#px), in §6.2
- [Q](#Q), in §6.2
- [rad](#rad), in §7.1
- [reference pixel](#reference-pixel), in §6.2
- [relative length](#relative-length), in §6.1
- [rem](#rem), in §6.1.1
- [\<resolution\>](#resolution-value), in §7.4
- [rlh](#rlh), in §6.1.1
- [s](#s), in §7.2
- [serialize a calc() value](#serialize-a-calc-value), in §9.1.5
- [serialize a max() value](#serialize-a-min-value), in §9.1.5
- [serialize a min() value](#serialize-a-min-value), in §9.1.5
- [serialize a summation](#math-function-serialize-a-summation), in §9.1.5
- [serialize the summation](#math-function-serialize-a-summation), in §9.1.5
- [simplify an expression](#math-function-simplify-an-expression), in §9.1.5
- [simplify the expression](#math-function-simplify-an-expression), in §9.1.5
- string
  - [definition of](#string), in §4.3
  - [value for \<type-or-value\>](#valdef-type-or-value-string), in §9.3
- [\<string\>](#string-value), in §4.3
- [\<time\>](#time-value), in §7.2
- [time](#valdef-type-or-value-time), in §9.3
- [\<time-percentage\>](#typedef-time-percentage), in §5.5
- [toggle()](#funcdef-toggle), in §9.2
- [\<toggle-value\>](#typedef-toggle-value), in §9.2
- [top-level calculation](#top-level-calculation), in §9.1.2
- [turn](#turn), in §7.1
- [\<type-or-unit\>](#typedef-type-or-unit), in §9.3
- [url()](#funcdef-url), in §4.4
- [\<url\>](#url-value), in §4.4
- [url](#valdef-type-or-value-url), in §9.3
- [\<url-modifier\>](#typedef-url-modifier), in §4.4.3
- [value accumulation](#accumulation), in §3
- [value addition](#addition), in §3
- [value interpolation](#interpolation), in §3
- [vb](#valdef-length-vb), in §6.1.2
- [vh](#vh), in §6.1.2
- [vi](#valdef-length-vi), in §6.1.2
- [viewport-percentage lengths](#viewport-percentage-lengths), in §6.1.2
- [visual angle unit](#pixel-unit), in §6.2
- [vmax](#vmax), in §6.1.2
- [vmin](#vmin), in §6.1.2
- [vw](#vw), in §6.1.2
- [x](#x), in §7.4
- [\<zero\>](#zero-value), in §5.2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-animations-1\] defines the following terms:
  - <a id="term-for-propdef-animation"></a>animation
  - <a id="term-for-propdef-animation-iteration-count"></a>animation-iteration-count
  - <a id="term-for-propdef-animation-name"></a>animation-name
  - <a id="term-for-propdef-animation-timing-function"></a>animation-timing-function
- \[css-box-3\] defines the following terms:
  - <a id="term-for-propdef-padding-top"></a>padding-top
- \[css-break-3\] defines the following terms:
  - <a id="term-for-propdef-orphans"></a>orphans
- \[css-cascade-4\] defines the following terms:
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
- \[css-color-4\] defines the following terms:
  - <a id="term-for-valdef-color-currentcolor"></a>currentcolor
  - <a id="term-for-funcdef-hsl"></a>hsl()
  - <a id="term-for-propdef-opacity"></a>opacity
  - <a id="term-for-funcdef-rgba"></a>rgba()
- \[css-counter-styles-3\] defines the following terms:
  - <a id="term-for-disc"></a>disc
- \[css-display-3\] defines the following terms:
  - <a id="term-for-containing-block"></a>containing block
- \[css-easing-1\] defines the following terms:
  - <a id="term-for-typedef-timing-function"></a>\<timing-function\>
  - <a id="term-for-valdef-cubic-bezier-timing-function-ease-in"></a>ease-in
  - <a id="term-for-valdef-cubic-bezier-timing-function-ease-out"></a>ease-out
  - <a id="term-for-timing-function"></a>timing function
- \[css-grid-1\] defines the following terms:
  - <a id="term-for-typedef-flex"></a>\<flex\>
- \[css-images-4\] defines the following terms:
  - <a id="term-for-propdef-image-resolution"></a>image-resolution
- \[css-inline-3\] defines the following terms:
  - <a id="term-for-valdef-line-height-normal"></a>normal
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-valdef-overflow-auto"></a>auto
  - <a id="term-for-propdef-overflow"></a>overflow
- \[css-overflow-4\] defines the following terms:
  - <a id="term-for-propdef-max-lines"></a>max-lines
- \[css-sizing-3\] defines the following terms:
  - <a id="term-for-valdef-width-auto"></a>auto
  - <a id="term-for-propdef-box-sizing"></a>box-sizing
  - <a id="term-for-propdef-min-width"></a>min-width
- \[css-text-3\] defines the following terms:
  - <a id="term-for-valdef-text-align-center"></a>center
  - <a id="term-for-propdef-tab-size"></a>tab-size
  - <a id="term-for-propdef-text-align"></a>text-align
- \[css-text-decor-3\] defines the following terms:
  - <a id="term-for-propdef-text-decoration"></a>text-decoration
  - <a id="term-for-propdef-text-emphasis-color"></a>text-emphasis-color
  - <a id="term-for-propdef-text-shadow"></a>text-shadow
- \[css-transforms-1\] defines the following terms:
  - <a id="term-for-propdef-transform-origin"></a>transform-origin
- \[css-typed-om-1\] defines the following terms:
  - <a id="term-for-cssnumericvalue-add-two-types"></a>add two types
  - <a id="term-for-cssnumericvalue-invert-a-type"></a>invert a type
  - <a id="term-for-cssnumericvalue-match"></a>match
  - <a id="term-for-cssnumericvalue-multiply-two-types"></a>multiply two types
  - <a id="term-for-cssnumericvalue-percent-hint"></a>percent hint
  - <a id="term-for-cssnumericvalue-type"></a>type
- \[css-ui-3\] defines the following terms:
  - <a id="term-for-valdef-cursor-default"></a>default
  - <a id="term-for-propdef-outline-color"></a>outline-color
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
  - <a id="term-for-propdef-height"></a>height
  - <a id="term-for-propdef-line-height"></a>line-height
  - <a id="term-for-propdef-width"></a>width
- \[CSS3-BACKGROUND\] defines the following terms:
  - <a id="term-for-propdef-background"></a>background
  - <a id="term-for-propdef-background-attachment"></a>background-attachment
  - <a id="term-for-propdef-background-position①"></a>background-position
  - <a id="term-for-propdef-border-color"></a>border-color
  - <a id="term-for-propdef-border-width"></a>border-width
  - <a id="term-for-propdef-box-shadow"></a>box-shadow
  - <a id="term-for-shadow-inset"></a>inset
- \[CSS3-FONTS\] defines the following terms:
  - <a id="term-for-propdef-font"></a>font
  - <a id="term-for-propdef-font-family"></a>font-family
  - <a id="term-for-propdef-font-size"></a>font-size
- \[CSS3-IMAGES\] defines the following terms:
  - <a id="term-for-image-type"></a>\<image\>
  - <a id="term-for-funcdef-linear-gradient"></a>linear-gradient()
- \[CSS3COLOR\] defines the following terms:
  - <a id="term-for-valuea-def-color"></a>\<color\>
- \[CSS3SYN\] defines the following terms:
  - <a id="term-for-typedef-dimension-token"></a>\<dimension-token\>
  - <a id="term-for-typedef-function-token"></a>\<function-token\>
  - <a id="term-for-typedef-hash-token"></a>\<hash-token\>
  - <a id="term-for-typedef-ident-token"></a>\<ident-token\>
  - <a id="term-for-typedef-number-token"></a>\<number-token\>
  - <a id="term-for-typedef-percentage-token"></a>\<percentage-token\>
  - <a id="term-for-typedef-string-token"></a>\<string-token\>
  - <a id="term-for-typedef-url-token"></a>\<url-token\>
  - <a id="term-for-component-value"></a>component value
  - <a id="term-for-consume-a-url-token0"></a>consume a url token
  - <a id="term-for-whitespace"></a>whitespace
- \[HTML\] defines the following terms:
  - <a id="term-for-the-base-element"></a>base
  - <a id="term-for-case-sensitive"></a>case-sensitive
  - <a id="term-for-dom-history-pushstate"></a>pushState(data, title)
- \[INFRA\] defines the following terms:
  - <a id="term-for-ascii-case-insensitive"></a>ascii case-insensitive
- \[MEDIAQ\] defines the following terms:
  - <a id="term-for-media-query"></a>media query
- \[selectors-3\] defines the following terms:
  - <a id="term-for-x"></a>\*
- \[selectors-4\] defines the following terms:
  - <a id="term-for-selectordef-adjacent"></a>+
  - <a id="term-for-selectordef-child"></a>\>
- \[URL\] defines the following terms:
  - <a id="term-for-concept-url"></a>url
- \[web-animations-1\] defines the following terms:
  - <a id="term-for-discrete"></a>discrete

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 5 July 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-counter-styles-3"></a>\[CSS-COUNTER-STYLES-3\]  
Tab Atkins Jr.. [CSS Counter Styles Level 3](https://www.w3.org/TR/css-counter-styles-3/). 14 December 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-counter-styles-3&#x2F;](https://www.w3.org/TR/css-counter-styles-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-easing-1"></a>\[CSS-EASING-1\]  
Brian Birtles; Dean Jackson; Matt Rakow. [CSS Easing Functions Level 1](https://www.w3.org/TR/css-easing-1/). 9 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-1&#x2F;](https://www.w3.org/TR/css-easing-1/)

<a id="biblio-css-grid-1"></a>\[CSS-GRID-1\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 14 December 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Image Values and Replaced Content Module Level 4](https://www.w3.org/TR/css-images-4/). 13 April 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad; Steve Zilles. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 8 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 31 July 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Intrinsic &#x26; Extrinsic Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 4 March 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-text-decor-3"></a>\[CSS-TEXT-DECOR-3\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 3](https://www.w3.org/TR/css-text-decor-3/). 3 July 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-3&#x2F;](https://www.w3.org/TR/css-text-decor-3/)

<a id="biblio-css-typed-om-1"></a>\[CSS-TYPED-OM-1\]  
Shane Stephens; Tab Atkins Jr.; Naina Raisinghani. [CSS Typed OM Level 1](https://www.w3.org/TR/css-typed-om-1/). 10 April 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-typed-om-1&#x2F;](https://www.w3.org/TR/css-typed-om-1/)

<a id="biblio-css-ui-3"></a>\[CSS-UI-3\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 21 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 24 May 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3-background"></a>\[CSS3-BACKGROUND\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 17 October 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css3-fonts"></a>\[CSS3-FONTS\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css3-images"></a>\[CSS3-IMAGES\]  
Elika Etemad; Tab Atkins Jr.. [CSS Image Values and Replaced Content Module Level 3](https://www.w3.org/TR/css3-images/). 17 April 2012. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css3-images&#x2F;](https://www.w3.org/TR/css3-images/)

<a id="biblio-css3cascade"></a>\[CSS3CASCADE\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://www.w3.org/TR/css-cascade-3/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-3&#x2F;](https://www.w3.org/TR/css-cascade-3/)

<a id="biblio-css3color"></a>\[CSS3COLOR\]  
Tantek Çelik; Chris Lilley; David Baron. [CSS Color Module Level 3](https://www.w3.org/TR/css-color-3/). 19 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-3&#x2F;](https://www.w3.org/TR/css-color-3/)

<a id="biblio-css3namespace"></a>\[CSS3NAMESPACE\]  
Elika Etemad. [CSS Namespaces Module Level 3](https://www.w3.org/TR/css-namespaces-3/). 20 March 2014. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-namespaces-3&#x2F;](https://www.w3.org/TR/css-namespaces-3/)

<a id="biblio-css3page"></a>\[CSS3PAGE\]  
Elika Etemad; Simon Sapin. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 18 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css3syn"></a>\[CSS3SYN\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 20 February 2014. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Simon Pieters; Glenn Adams. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 17 March 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-mediaq"></a>\[MEDIAQ\]  
Florian Rivoal; Tab Atkins Jr.. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 5 September 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-4&#x2F;](https://www.w3.org/TR/mediaqueries-4/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

<a id="biblio-selectors-3"></a>\[SELECTORS-3\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 2 February 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-unicode"></a>\[UNICODE\]  
[The Unicode Standard](https://www.unicode.org/versions/latest/). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;versions&#x2F;latest&#x2F;](https://www.unicode.org/versions/latest/)

<a id="biblio-url"></a>\[URL\]  
Anne van Kesteren. [URL Standard](https://url.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;url&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://url.spec.whatwg.org/)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

### <a id="informative"></a>Informative References

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
Dean Jackson; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-box-3"></a>\[CSS-BOX-3\]  
Elika Etemad. [CSS Box Model Module Level 3](https://www.w3.org/TR/css-box-3/). 9 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-3&#x2F;](https://www.w3.org/TR/css-box-3/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 9 February 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-overflow-4"></a>\[CSS-OVERFLOW-4\]  
David Baron; Florian Rivoal. [CSS Overflow Module Level 4](https://www.w3.org/TR/css-overflow-4/). 13 June 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-4&#x2F;](https://www.w3.org/TR/css-overflow-4/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 20 September 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 30 November 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-rfc6694"></a>\[RFC6694\]  
S. Moonesamy, Ed.. [The "about" URI Scheme](https://tools.ietf.org/html/rfc6694). August 2012. Informational. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc6694](https://tools.ietf.org/html/rfc6694)

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> We can potentially add more typographic units, like cicero, didot, etc. They’re just absolute units, and so can be done with the existing units, but is there enough desire for them (potentially for printing use-cases) that it would be worth adding them? Or should we just wait for Houdini Custom Units? [↵](#issue-7dc6ac0d)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Computed value needs to be able to represent combinations of [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) and an actual color. Consider the value of [text-emphasis-color](https://www.w3.org/TR/css-text-decor-3/#propdef-text-emphasis-color) in `div { text-emphasis: circle; transition: all 2s; } div:hover { text-emphasis-color: lime; } em { color: red; }` See [Issue 445](https://github.com/w3c/csswg-drafts/issues/445). [↵](#issue-7fabb422)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section is still [under discussion](https://lists.w3.org/Archives/Member/w3c-css-wg/2016AprJun/0239.html). [↵](#issue-f5bc4b00)
