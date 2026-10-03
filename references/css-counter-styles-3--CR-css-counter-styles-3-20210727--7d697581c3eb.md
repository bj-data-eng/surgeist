Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Counter Styles Level 3](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/).

Original copyright notice: Copyright © 2021 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Counter Styles Level 3

Source snapshot: https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/

Snapshot SHA-256: 7d697581c3eb238c3e8d18bcf0c03aa889380041fa208c8bf06c51dd07be9807

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 14 source tables are presented as readable Markdown tables or explicit labeled layouts: 12 ordinary table conversions, 2 complex-table layouts. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Counter Styles Level 3

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2021 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-at-ruledef-counter-style"></a>

This module introduces the [@counter-style](#at-ruledef-counter-style) rule, which allows authors to define their own custom counter styles for use with CSS list-marker and generated-content counters [\[CSS-LISTS-3\]](#biblio-css-lists-3). It also predefines a set of common counter styles, including the ones present in CSS2 and CSS2.1.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	Other documents may supersede this document.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/Style/CSS/) as a <strong>Candidate Recommendation Snapshot</strong>. Publication as a Candidate Recommendation does not imply endorsement by the W3C Membership. A Candidate Recommendation Snapshot has received [wide review](https://www.w3.org/2020/Process-20200915/#dfn-wide-review) and is intended to gather implementation experience. This document is intended to become a W3C Recommendation; it will remain a Candidate Recommendation at least until 27 October 2021 to gather additional feedback.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-counter-styles” in the title, like this: “\[css-counter-styles\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-counter-styles%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [15 September 2020 W3C Process Document](https://www.w3.org/2020/Process-20200915/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-typedef-symbol"></a>

  <a id="ref-for-typedef-image"></a>

  the [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) value in [\<symbol\>](#typedef-symbol)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

CSS 1 defined a handful of useful counter styles based on the styles that HTML traditionally allowed on ordered and unordered lists. While this was expanded slightly by CSS2.1, it doesn’t address the needs of worldwide typography.

<a id="ref-for-at-ruledef-counter-style①"></a>

<a id="ref-for-propdef-list-style-type"></a>

<a id="ref-for-funcdef-counter"></a>

<a id="ref-for-funcdef-counters"></a>

This module introduces the [@counter-style](#at-ruledef-counter-style) rule which allows CSS to address this in an open-ended manner, by allowing the author to define their own counter styles. These styles can then be used in the [list-style-type](https://www.w3.org/TR/css-lists-3/#propdef-list-style-type) property or in the [counter()](https://www.w3.org/TR/css-lists-3/#funcdef-counter) and [counters()](https://www.w3.org/TR/css-lists-3/#funcdef-counters) functions. It also defines some additional predefined counter styles, particularly ones which are common but complicated to represent with <a id="ref-for-at-ruledef-counter-style②"></a>@counter-style.

## <a id="counter-styles"></a>2.  Counter Styles

A <a id="counter-style"></a>counter style defines how to convert a counter value into a string. Counter styles are composed of:

- a name, to identify the style
- an algorithm, which transforms integer counter values into a basic string representation
- a negative sign, which is prepended or appended to the representation of a negative counter value.
- a prefix, to prepend to the representation
- a suffix to append to the representation
- a range, which limits the values that a counter style handles
- a spoken form, which describes how to read out the counter style in a speech synthesizer
- and a fallback style, to render the representation with when the counter value is outside the counter style’s range or the counter style otherwise can’t render the counter value

When asked to <a id="generate-a-counter"></a>generate a counter representation using a particular counter style for a particular counter value, follow these steps:

1.  <a id="ref-for-decimal"></a>

    <a id="ref-for-generate-a-counter"></a>

    If the counter style is unknown, exit this algorithm and instead [generate a counter representation](#generate-a-counter) using the [decimal](#decimal) style and the same counter value.

2.  <a id="ref-for-generate-a-counter①"></a>

    <a id="ref-for-descdef-counter-style-range"></a>

    If the counter value is outside the [range](#descdef-counter-style-range) of the counter style, exit this algorithm and instead [generate a counter representation](#generate-a-counter) using the counter style’s fallback style and the same counter value.

3.  <a id="ref-for-use-a-negative-sign"></a>

    Using the counter value and the counter algorithm for the counter style, generate an <a id="initial-representation-for-the-counter-value"></a>initial representation for the counter value. If the counter value is negative and the counter style [uses a negative sign](#use-a-negative-sign), instead generate an initial representation using the absolute value of the counter value.

4.  <a id="ref-for-descdef-counter-style-pad"></a>

    Prepend symbols to the representation as specified in the [pad](#descdef-counter-style-pad) descriptor.

5.  <a id="ref-for-descdef-counter-style-negative"></a>

    <a id="ref-for-use-a-negative-sign①"></a>

    If the counter value is negative and the counter style [uses a negative sign](#use-a-negative-sign), wrap the representation in the counter style’s negative sign as specified in the [negative](#descdef-counter-style-negative) descriptor.

6.  Return the representation.

<a id="ref-for-descdef-counter-style-prefix"></a>

<a id="ref-for-descdef-counter-style-suffix"></a>

<a id="ref-for-propdef-content"></a>

<a id="ref-for-selectordef-marker"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [prefix](#descdef-counter-style-prefix) and [suffix](#descdef-counter-style-suffix) don’t play a part in this algorithm. This is intentional; the prefix and suffix aren’t part of the string returned by the counter() or counters() functions. Instead, the prefix and suffix are added by the algorithm that constructs the value of the [content](https://www.w3.org/TR/css-content-3/#propdef-content) property for the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element. This also implies that the prefix and suffix always come from the specified counter-style, even if the actual representation is constructed by a fallback style.

<a id="ref-for-descdef-counter-style-system"></a>

<a id="ref-for-valdef-system-symbolic"></a>

<a id="ref-for-valdef-counter-style-system-additive"></a>

<a id="ref-for-descdef-counter-style-pad①"></a>

Some values of [system](#descdef-counter-style-system) ([symbolic](#valdef-system-symbolic), [additive](#valdef-counter-style-system-additive)) and some descriptors ([pad](#descdef-counter-style-pad)) can generate representations with size linear to an author-supplied number. This can potentially be abused to generate excessively large representations and consume undue amounts of the user’s memory or even hang their browser. User agents must support representations at least 60 Unicode codepoints long, but they may choose to instead use the fallback style for representations that would be longer than 60 codepoints.

<a id="ref-for-at-ruledef-counter-style③"></a>

## <a id="the-counter-style-rule"></a>3.  Defining Custom Counter Styles: the [@counter-style](#at-ruledef-counter-style) rule

<a id="ref-for-counter-style"></a>

<a id="ref-for-at-ruledef-counter-style④"></a>

<a id="ref-for-valdef-text-emphasis-skip-symbols"></a>

The <a id="at-ruledef-counter-style"></a>@counter-style rule allows authors to define a custom [counter style](#counter-style). The components of a <a id="ref-for-counter-style①"></a>counter style are specified by descriptors in the [@counter-style](#at-ruledef-counter-style) rule. The algorithm is specified implicitly by a combination of the system, [symbols](https://drafts.csswg.org/css-text-decor-4/#valdef-text-emphasis-skip-symbols), and additive-symbols properties.

<a id="ref-for-at-ruledef-counter-style⑤"></a>

The general form of an [@counter-style](#at-ruledef-counter-style) rule is:

<a id="ref-for-typedef-counter-style-name"></a>

<a id="ref-for-typedef-declaration-list"></a>

```text
@counter-style <counter-style-name> { <declaration-list> }
```
<a id="ref-for-identifier-value"></a>

<a id="ref-for-ascii-case-insensitive"></a>

<a id="ref-for-valdef-list-style-type-none"></a>

<a id="ref-for-typedef-counter-style-name①"></a>

<a id="ref-for-css-tree-scoped-name"></a>

<a id="typedef-counter-style-name"></a>\<counter-style-name\> is a [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) that is not an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for [none](https://www.w3.org/TR/css-lists-3/#valdef-list-style-type-none). The [\<counter-style-name\>](#typedef-counter-style-name) is a [tree-scoped name](https://drafts.csswg.org/css-scoping-1/#css-tree-scoped-name).

<a id="ref-for-decimal①"></a>

<a id="ref-for-disc"></a>

<a id="ref-for-square"></a>

<a id="ref-for-circle"></a>

<a id="ref-for-disclosure-open"></a>

<a id="ref-for-disclosure-closed"></a>

<a id="ref-for-typedef-counter-style-name②"></a>

The keywords [decimal](#decimal), [disc](#disc), [square](#square), [circle](#circle), [disclosure-open](#disclosure-open), and [disclosure-closed](#disclosure-closed) are valid [\<counter-style-name\>](#typedef-counter-style-name)s, but are invalid when used here to name a counter style rule; doing so makes the rule invalid. (They can be used in other contexts, such as in the extend system.)

<a id="ref-for-identifier-value①"></a>

<a id="ref-for-css-wide-keywords"></a>

<a id="ref-for-valdef-list-style-position-inside"></a>

<a id="ref-for-propdef-list-style"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) also automatically excludes the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords). In addition, some names, like [inside](https://www.w3.org/TR/css-lists-3/#valdef-list-style-position-inside), are valid as counter style names, but conflict with the existing values of properties like [list-style](https://www.w3.org/TR/css-lists-3/#propdef-list-style), and so won’t be usable there.

<a id="ref-for-propdef-list-style①"></a>

<a id="ref-for-at-ruledef-counter-style⑥"></a>

<a id="ref-for-funcdef-counter①"></a>

Counter style names are case-sensitive. However, the names defined in this specification are ASCII lower-cased on parse wherever they are used as counter styles, e.g. in the [list-style](https://www.w3.org/TR/css-lists-3/#propdef-list-style) set of properties, in the [@counter-style](#at-ruledef-counter-style) rule, and in the [counter()](https://www.w3.org/TR/css-lists-3/#funcdef-counter) functions.

<a id="ref-for-at-ruledef-counter-style⑦"></a>

Each [@counter-style](#at-ruledef-counter-style) rule specifies a value for every counter-style descriptor, either implicitly or explicitly. Those not given explicit value in the rule take the initial value listed with each descriptor in this specification. These descriptors apply solely within the context of the <a id="ref-for-at-ruledef-counter-style⑧"></a>@counter-style rule in which they are defined, and do not apply to document language elements. There is no notion of which elements the descriptors apply to or whether the values are inherited by child elements. When a given descriptor occurs multiple times in a given <a id="ref-for-at-ruledef-counter-style⑨"></a>@counter-style rule, only the last-specified valid value is used; all prior values for that descriptor must be ignored.

<a id="ref-for-at-ruledef-counter-style①⓪"></a>

Defining a [@counter-style](#at-ruledef-counter-style) makes it available to the entire document in which it is included. If multiple <a id="ref-for-at-ruledef-counter-style①①"></a>@counter-style rules are defined with the same name, only one wins, according to standard cascade rules. <a id="ref-for-at-ruledef-counter-style①②"></a>@counter-style rules cascade "atomically": if one replaces another of the same name, it replaces it <em>entirely</em>, rather than just replacing the specific descriptors it specifies.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that even the predefined counter styles can be overridden; the UA stylesheet occurs before any other stylesheets, so the predefined ones always lose in the cascade.

<a id="ref-for-at-ruledef-counter-style①③"></a>

This at-rule conforms with the forward-compatible parsing requirement of CSS; conformant parsers that don’t understand these rules will ignore them without error. Any descriptors that are not recognized or implemented by a given user agent, or whose value does not match the grammars given here or in a future version of this specification, must be ignored in their entirety; they do not make the [@counter-style](#at-ruledef-counter-style) rule invalid.

<a id="ref-for-descdef-counter-style-system①"></a>

### <a id="counter-style-system"></a>3.1.  Counter algorithms: the [system](#descdef-counter-style-system) descriptor

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-counter-style-system"></a>system                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-counter-style①④"></a>[@counter-style](#at-ruledef-counter-style)                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-typedef-counter-style-name③"></a><a id="ref-for-mult-opt"></a><a id="ref-for-integer-value"></a><a id="ref-for-comb-one"></a>cyclic [\|](https://www.w3.org/TR/css-values-4/#comb-one) numeric <a id="ref-for-comb-one①"></a>\| alphabetic <a id="ref-for-comb-one②"></a>\| symbolic <a id="ref-for-comb-one③"></a>\| additive <a id="ref-for-comb-one④"></a>\| \[fixed [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value)[?](https://www.w3.org/TR/css-values-4/#mult-opt)\] <a id="ref-for-comb-one⑤"></a>\| \[ extends [\<counter-style-name\>](#typedef-counter-style-name) \] |
| <strong>Initial:&#xA;      </strong> | symbolic                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |

<a id="ref-for-descdef-counter-style-system②"></a>

<a id="ref-for-valdef-counter-style-system-cyclic"></a>

<a id="ref-for-valdef-counter-style-system-numeric"></a>

The [system](#descdef-counter-style-system) descriptor specifies which algorithm will be used to construct the counter’s representation based on the counter value. For example, [cyclic](#valdef-counter-style-system-cyclic) counter styles just cycle through their symbols repeatedly, while [numeric](#valdef-counter-style-system-numeric) counter styles interpret their symbols as digits and build their representation accordingly. The systems are defined in the following subsections.

<a id="ref-for-descdef-counter-style-system③"></a>

<a id="ref-for-descdef-counter-style-symbols"></a>

<a id="ref-for-descdef-counter-style-additive-symbols"></a>

<a id="ref-for-at-ruledef-counter-style①⑤"></a>

<a id="ref-for-counter-style②"></a>

Each [system](#descdef-counter-style-system) value is associated with either the [symbols](#descdef-counter-style-symbols) or [additive-symbols](#descdef-counter-style-additive-symbols) descriptors, and has a minimum length that the appropriate descriptor must have; each entry below defines what this is. If a [@counter-style](#at-ruledef-counter-style) rule fails to meet this requirement, it does not define a [counter style](#counter-style). (The rule is still syntactically valid, but has no effect.)

<a id="ref-for-valdef-counter-style-system-cyclic①"></a>

#### <a id="cyclic-system"></a>3.1.1.  Cycling Symbols: the [cyclic](#valdef-counter-style-system-cyclic) system

<a id="ref-for-counter-symbol"></a>

The <a id="valdef-counter-style-system-cyclic"></a>cyclic counter system cycles repeatedly through its provided symbols, looping back to the beginning when it reaches the end of the list. It can be used for simple bullets (just provide a single [counter symbol](#counter-symbol)), or for cycling through multiple symbols. The first <a id="ref-for-counter-symbol①"></a>counter symbol is used as the representation of the value 1, the second <a id="ref-for-counter-symbol②"></a>counter symbol (if it exists) is used as the representation of the value 2, etc.

<a id="ref-for-valdef-counter-style-system-cyclic②"></a>

<a id="ref-for-descdef-counter-style-symbols①"></a>

<a id="ref-for-counter-symbol③"></a>

If the system is [cyclic](#valdef-counter-style-system-cyclic), the [symbols](#descdef-counter-style-symbols) descriptor must contain at least one [counter symbol](#counter-symbol). This system is defined over all counter values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bb402b2d"></a> A "triangle bullet" counter style can be defined as:
>
> <a id="triangle"></a>
>
> ```text
> @counter-style triangle {
>   system: cyclic;
>   symbols: ‣;
>   suffix: " ";
> }
> ```
>
> It will then produce lists that look like:
>
> ```text
> ‣  One
> ‣  Two
> ‣  Three
> ```
<a id="ref-for-counter-symbol④"></a>

If there are <var>N</var> [counter symbols](#counter-symbol) and a representation is being constructed for the integer <var>value</var>, the representation is the <a id="ref-for-counter-symbol⑤"></a>counter symbol at index ( (<var>value</var>-1) mod <var>N</var>) of the list of <a id="ref-for-counter-symbol⑥"></a>counter symbols (0-indexed).

<a id="ref-for-valdef-counter-style-system-fixed"></a>

#### <a id="fixed-system"></a>3.1.2.  Exhaustible Symbols: the [fixed](#valdef-counter-style-system-fixed) system

The <a id="valdef-counter-style-system-fixed"></a>fixed counter system runs through its list of counter symbols once, then falls back. It is useful for representing counter styles that only have a finite number of representations. For example, Unicode defines several limited-length runs of special characters meant for lists, such as circled digits.

<a id="ref-for-valdef-counter-style-system-fixed①"></a>

<a id="ref-for-descdef-counter-style-symbols②"></a>

<a id="ref-for-counter-symbol⑦"></a>

<a id="ref-for-first-symbol-value"></a>

If the system is [fixed](#valdef-counter-style-system-fixed), the [symbols](#descdef-counter-style-symbols) descriptor must contain at least one [counter symbol](#counter-symbol). This system is defined over counter values in a finite range, starting with the [first symbol value](#first-symbol-value) and having a length equal to the length of the list of <a id="ref-for-counter-symbol⑧"></a>counter symbols.

<a id="ref-for-first-symbol-value①"></a>

When this system is specified, it may optionally have an integer provided after it, which sets the <a id="first-symbol-value"></a>first symbol value. If it is omitted, the [first symbol value](#first-symbol-value) is 1.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-24b2a03a"></a> A "box-corner" counter style can be defined as:
>
> <a id="box-corner"></a>
>
> ```text
> @counter-style box-corner {
>   system: fixed;
>   symbols: ◰ ◳ ◲ ◱;
>   suffix: ': ';
> }
> ```
>
> It will then produce lists that look like:
>
> ```text
> ◰:  One
> ◳:  Two
> ◲:  Three
> ◱:  Four
> 5:  Five
> 6:  Six
> ```
<a id="ref-for-counter-symbol⑨"></a>

<a id="ref-for-first-symbol-value②"></a>

The first [counter symbol](#counter-symbol) is the representation for the [first symbol value](#first-symbol-value), and subsequent counter values are represented by subsequent <a id="ref-for-counter-symbol①⓪"></a>counter symbols. Once the list of <a id="ref-for-counter-symbol①①"></a>counter symbols is exhausted, further values cannot be represented by this counter style, and must instead be represented by the fallback counter style.

<a id="ref-for-valdef-system-symbolic①"></a>

#### <a id="symbolic-system"></a>3.1.3.  Repeating Symbols: the [symbolic](#valdef-system-symbolic) system

<a id="ref-for-valdef-counter-style-system-alphabetic"></a>

The <a id="valdef-system-symbolic"></a>symbolic counter system cycles repeatedly through its provided symbols, doubling, tripling, etc. the symbols on each successive pass through the list. For example, if the original symbols were "\*" and "†", then on the second pass they would instead be "\*\*" and "††", while on the third they would be "\*\*\*"and "†††", etc. It can be used for footnote-style markers, and is also sometimes used for alphabetic-style lists for a slightly different presentation than what the [alphabetic](#valdef-counter-style-system-alphabetic) system presents.

<a id="ref-for-valdef-system-symbolic②"></a>

<a id="ref-for-descdef-counter-style-symbols③"></a>

<a id="ref-for-counter-symbol①②"></a>

If the system is [symbolic](#valdef-system-symbolic), the [symbols](#descdef-counter-style-symbols) descriptor must contain at least one [counter symbol](#counter-symbol). This system is defined only over strictly positive counter values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a76d671e"></a> An "footnote" counter style can be defined as:
>
> <a id="footnote"></a>
>
> ```text
> @counter-style footnote {
>   system: symbolic;
>   symbols: '*' ⁑ † ‡;
>   suffix: " ";
> }
> ```
>
> It will then produce lists that look like:
>
> ```text
> *   One
> ⁑   Two
> †   Three
> ‡   Four
> **  Five
> ⁑⁑  Six
> ```
<a id="ref-for-upper-alpha"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5b0bbd83"></a> Some style guides mandate a list numbering that looks similar to [upper-alpha](#upper-alpha), but repeats differently after the first 26 values, instead going "AA", "BB", "CC", etc. This can be achieved with the symbolic system:
>
> <a id="upper-alpha-legal"></a>
>
> ```text
> @counter-style upper-alpha-legal {
>   system: symbolic;
>   symbols: A B C D E F G H I J K L M
>            N O P Q R S T U V W X Y Z;
> }
> ```
>
> <a id="ref-for-upper-alpha①"></a>
>
> This style is identical to [upper-alpha](#upper-alpha) through the first 27 values, but they diverge after that, with <a id="ref-for-upper-alpha②"></a>upper-alpha going "AB", "AC", "AD", etc. Starting at the 53rd value, <a id="ref-for-upper-alpha③"></a>upper-alpha goes "BA", "BB", "BC", etc., while this style jumps into triple digits with "AAA", "BBB", "CCC", etc.

To construct the representation, run the following algorithm:

<a id="ref-for-counter-symbol①③"></a>

Let <var>N</var> be the length of the list of [counter symbols](#counter-symbol), <var>value</var> initially be the counter value, <var>S</var> initially be the empty string, and <var>symbol(n)</var> be the nth <a id="ref-for-counter-symbol①④"></a>counter symbol in the list of <a id="ref-for-counter-symbol①⑤"></a>counter symbols (0-indexed).

1.  Let the <var>chosen symbol</var> be <code>symbol( (<var>value</var> - 1) mod <var>N</var>)</code>.
2.  Let the <var>representation length</var> be <code>ceil( <var>value</var> / <var>N</var> )</code>.
3.  Append the <var>chosen symbol</var> to <var>S</var> a number of times equal to the <var>representation length</var>.

Finally, return <var>S</var>.

<a id="ref-for-valdef-counter-style-system-alphabetic①"></a>

#### <a id="alphabetic-system"></a>3.1.4.  Bijective Numerals: the [alphabetic](#valdef-counter-style-system-alphabetic) system

<a id="ref-for-counter-symbol①⑥"></a>

<a id="ref-for-lower-alpha"></a>

The <a id="valdef-counter-style-system-alphabetic"></a>alphabetic counter system interprets the list of [counter symbols](#counter-symbol) as digits to an <em>alphabetic</em> numbering system, similar to the default [lower-alpha](#lower-alpha) counter style, which wraps from "a", "b", "c", to "aa", "ab", "ac". Alphabetic numbering systems do not contain a digit representing 0; so the first value when a new digit is added is composed solely of the first digit. Alphabetic numbering systems are commonly used for lists, and also appear in many spreadsheet programs to number columns. The first <a id="ref-for-counter-symbol①⑦"></a>counter symbol in the list is interpreted as the digit 1, the second as the digit 2, and so on.

<a id="ref-for-valdef-counter-style-system-alphabetic②"></a>

<a id="ref-for-descdef-counter-style-symbols④"></a>

<a id="ref-for-counter-symbol①⑧"></a>

If the system is [alphabetic](#valdef-counter-style-system-alphabetic), the [symbols](#descdef-counter-style-symbols) descriptor must contain at least two [counter symbols](#counter-symbol). This system is defined only over strictly positive counter values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8007bc6e"></a> A counter style using go stones can be defined as:
>
> <a id="go"></a>
>
> ```text
> @counter-style go {
>   system: alphabetic;
>   symbols: url(white.svg) url(black.svg);
>   suffix: " ";
> }
> ```
>
> It will then produce lists that look like:
>
> ![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/white.svg) One  
> ![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/black.svg) Two  
> ![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/white.svg)![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/white.svg) Three  
> ![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/white.svg)![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/black.svg) Four  
> ![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/black.svg)![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/white.svg) Five  
> ![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/black.svg)![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/black.svg) Six  
> ![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/white.svg)![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/white.svg)![](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/images/white.svg) Seven
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: This example requires support for SVG images to display correctly.

<a id="ref-for-counter-symbol①⑨"></a>

If there are <var>N</var> [counter symbols](#counter-symbol), the representation is a base <var>N</var> alphabetic number using the <a id="ref-for-counter-symbol②⓪"></a>counter symbols as digits. To construct the representation, run the following algorithm:

<a id="ref-for-counter-symbol②①"></a>

Let <var>N</var> be the length of the list of [counter symbols](#counter-symbol), <var>value</var> initially be the counter value, <var>S</var> initially be the empty string, and <var>symbol(n)</var> be the nth <a id="ref-for-counter-symbol②②"></a>counter symbol in the list of <a id="ref-for-counter-symbol②③"></a>counter symbols (0-indexed).

While <var>value</var> is not equal to 0:

1.  Set <var>value</var> to <code><var>value</var> - 1</code>.
2.  Prepend <var>symbol( <var>value</var> mod <var>N</var> )</var> to <var>S</var>.
3.  Set <var>value</var> to <code>floor( <var>value</var> / <var>N</var> )</code>.

Finally, return <var>S</var>.

<a id="ref-for-valdef-counter-style-system-numeric①"></a>

#### <a id="numeric-system"></a>3.1.5.  Positional Numerals: the [numeric](#valdef-counter-style-system-numeric) system

<a id="ref-for-counter-symbol②④"></a>

<a id="ref-for-decimal②"></a>

The <a id="valdef-counter-style-system-numeric"></a>numeric counter system interprets the list of [counter symbols](#counter-symbol) as digits to a "place-value" numbering system, similar to the default [decimal](#decimal) counter style. The first <a id="ref-for-counter-symbol②⑤"></a>counter symbol in the list is interpreted as the digit 0, the second as the digit 1, and so on.

<a id="ref-for-valdef-counter-style-system-numeric②"></a>

<a id="ref-for-descdef-counter-style-symbols⑤"></a>

<a id="ref-for-counter-symbol②⑥"></a>

If the system is [numeric](#valdef-counter-style-system-numeric), the [symbols](#descdef-counter-style-symbols) descriptor must contain at least two [counter symbols](#counter-symbol). This system is defined over all counter values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-95bf6e12"></a> A "trinary" counter style can be defined as:
>
> <a id="trinary"></a>
>
> ```text
> @counter-style trinary {
>   system: numeric;
>   symbols: '0' '1' '2';
> }
> ```
>
> It will then produce lists that look like:
>
> ```text
> 1.   One
> 2.   Two
> 10.  Three
> 11.  Four
> 12.  Five
> 20.  Six
> ```
<a id="ref-for-counter-symbol②⑦"></a>

If there are <var>N</var> [counter symbols](#counter-symbol), the representation is a base <var>N</var> number using the <a id="ref-for-counter-symbol②⑧"></a>counter symbols as digits. To construct the representation, run the following algorithm:

<a id="ref-for-counter-symbol②⑨"></a>

Let <var>N</var> be the length of the list of [counter symbols](#counter-symbol), <var>value</var> initially be the counter value, <var>S</var> initially be the empty string, and <var>symbol(n)</var> be the nth <a id="ref-for-counter-symbol③⓪"></a>counter symbol in the list of <a id="ref-for-counter-symbol③①"></a>counter symbols (0-indexed).

1.  If <var>value</var> is 0, append `symbol(0)` to <var>S</var> and return <var>S</var>.
2.  While <var>value</var> is not equal to 0:
    1.  Prepend <var>symbol( <var>value</var> mod <var>N</var> )</var> to <var>S</var>.
    2.  Set <var>value</var> to <code>floor( <var>value</var> / <var>N</var> )</code>.
3.  Return <var>S</var>.

<a id="ref-for-valdef-counter-style-system-additive①"></a>

#### <a id="additive-system"></a>3.1.6.  Accumulating Numerals: the [additive](#valdef-counter-style-system-additive) system

The <a id="valdef-counter-style-system-additive"></a>additive counter system is used to represent "sign-value" numbering systems, which, rather than reusing digits in different positions to change their value, define additional digits with much larger values, so that the value of the number can be obtained by adding all the digits together. This is used in Roman numerals and other numbering systems around the world.

<a id="ref-for-valdef-counter-style-system-additive②"></a>

<a id="ref-for-descdef-counter-style-additive-symbols①"></a>

<a id="ref-for-additive-tuple"></a>

If the system is [additive](#valdef-counter-style-system-additive), the [additive-symbols](#descdef-counter-style-additive-symbols) descriptor must contain at least one [additive tuple](#additive-tuple). This system is nominally defined over all counter values (see algorithm, below, for exact details).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ffca8577"></a> A "dice" counter style can be defined as:
>
> <a id="dice"></a>
>
> ```text
> @counter-style dice {
>   system: additive;
>   additive-symbols: 6 ⚅, 5 ⚄, 4 ⚃, 3 ⚂, 2 ⚁, 1 ⚀;
>   suffix: " ";
> }
> ```
>
> It will then produce lists that look like:
>
> ```text
>   ⚀  One
>   ⚁  Two
>   ⚂  Three
> ...
>  ⚅⚄  Eleven
>  ⚅⚅  Twelve
> ⚅⚅⚀  Thirteen
> ```
To construct the representation:

1.  <a id="ref-for-additive-tuple①"></a>

    Let <var>value</var> initially be the counter value, <var>S</var> initially be the empty string, and <var>symbol list</var> initially be the list of [additive tuples](#additive-tuple).

2.  If <var>value</var> is zero:

    1.  <a id="ref-for-counter-symbol③②"></a>

        If <var>symbol list</var> contains a tuple with a weight of zero, append that tuple’s [counter symbol](#counter-symbol) to <var>S</var> and return <var>S</var>.

    2.  Otherwise, the given counter value cannot be represented by this counter style, and must instead be represented by the fallback counter style.

3.  For each <var>tuple</var> in <var>symbol list</var>:

    1.  <a id="ref-for-counter-symbol③③"></a>

        Let <var>symbol</var> and <var>weight</var> be <var>tuple</var>’s [counter symbol](#counter-symbol) and weight, respectively.

    2.  <a id="ref-for-iteration-continue"></a>

        If <var>weight</var> is zero, or <var>weight</var> is greater than <var>value</var>, [continue](https://infra.spec.whatwg.org/#iteration-continue).

    3.  Let <var>reps</var> be <code>floor( <var>value</var> / <var>weight</var> )</code>.

    4.  Append <var>symbol</var> to <var>S</var> <var>reps</var> times.

    5.  Decrement <var>value</var> by <code><var>weight</var> &#x2A; <var>reps</var></code>.

    6.  If <var>value</var> is zero, return <var>S</var>.

4.  Assertion: <var>value</var> is still non-zero.

    The given counter value cannot be represented by this counter style, and must instead be represented by the fallback counter style.

<a id="ref-for-at-ruledef-counter-style①⑥"></a>

<a id="ref-for-descdef-counter-style-additive-symbols②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: All of the predefined additive [@counter-style](#at-ruledef-counter-style) rules in this specification produce representations for every value in their range, but it’s possible to produce values for additive-symbols that will fail to find a representation with the algorithm defined above, even though theoretically a representation could be found. For example, if a <a id="ref-for-at-ruledef-counter-style①⑦"></a>@counter-style was defined with [additive-symbols: 3 "a", 2 "b";](#descdef-counter-style-additive-symbols), the algorithm defined above will fail to find a representation for a counter value of 4, even though theoretically a "bb" representation would work. While unfortunate, this is required to maintain the property that the algorithm runs in linear time relative to the size of the counter value.

<a id="ref-for-valdef-counter-style-system-extends"></a>

#### <a id="extends-system"></a>3.1.7.  Building from Existing Counter Styles: the [extends](#valdef-counter-style-system-extends) system <a id="override-system"></a>

<a id="ref-for-valdef-counter-style-system-extends①"></a>

The <a id="valdef-counter-style-system-extends"></a>extends system allows an author to use the algorithm of another counter style, but alter other aspects, such as the negative sign or the suffix. If a counter style uses the [extends](#valdef-counter-style-system-extends) system, any unspecified descriptors must be taken from the extended counter style specified, rather than taking their initial values.

<a id="ref-for-at-ruledef-counter-style①⑧"></a>

<a id="ref-for-valdef-counter-style-system-extends②"></a>

<a id="ref-for-descdef-counter-style-symbols⑥"></a>

<a id="ref-for-descdef-counter-style-additive-symbols③"></a>

If a [@counter-style](#at-ruledef-counter-style) uses the [extends](#valdef-counter-style-system-extends) system, it must not contain a [symbols](#descdef-counter-style-symbols) or [additive-symbols](#descdef-counter-style-additive-symbols) descriptor, or else the <a id="ref-for-at-ruledef-counter-style①⑨"></a>@counter-style rule is invalid.

<a id="ref-for-typedef-counter-style-name④"></a>

<a id="ref-for-ascii-case-insensitive①"></a>

<a id="ref-for-disc①"></a>

<a id="ref-for-circle①"></a>

<a id="ref-for-square①"></a>

<a id="ref-for-disclosure-open①"></a>

<a id="ref-for-disclosure-closed①"></a>

If the specified [\<counter-style-name\>](#typedef-counter-style-name) is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for [disc](#disc), [circle](#circle), [square](#square), [disclosure-open](#disclosure-open), or [disclosure-closed](#disclosure-closed) (any of the predefined symbolic counter styles), using extend extends from the “standard” definition of the rules provided in the normative stylesheet (rather than the exception allowing them to be drawn in a different, user-agent-specific fashion.)

<a id="ref-for-decimal③"></a>

<a id="ref-for-at-ruledef-counter-style②⓪"></a>

<a id="ref-for-valdef-counter-style-system-extends③"></a>

If the specified counter style name isn’t the name of any defined counter style, it must be treated as if it was extending the [decimal](#decimal) counter style. If one or more [@counter-style](#at-ruledef-counter-style) rules form a cycle with their [extends](#valdef-counter-style-system-extends) values, all of the counter styles participating in the cycle must be treated as if they were extending the <a id="ref-for-decimal④"></a>decimal counter style instead.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4eacf761"></a> For example, if you wanted a counter style that was identical to decimal, but used a parenthesis rather than a period after it, like:
>
> ```text
> 1) first item
> 2) second item
> 3) third item
> ```
>
> <a id="ref-for-decimal⑤"></a>
>
> Rather than writing up an entirely new counter style, this can be done by just extending [decimal](#decimal):
>
> ```text
> @counter-style decimal-paren {
>   system: extends decimal;
>   suffix: ") ";
> }
> ```
<a id="ref-for-descdef-counter-style-negative①"></a>

### <a id="counter-style-negative"></a>3.2.  Formatting negative values: the [negative](#descdef-counter-style-negative) descriptor

| Field               | Definition                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-counter-style-negative"></a>negative                                                                                                                        |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-counter-style②①"></a>[@counter-style](#at-ruledef-counter-style)                                                                                     |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-opt①"></a><a id="ref-for-typedef-symbol①"></a>[\<symbol\>](#typedef-symbol) <a id="ref-for-typedef-symbol②"></a>\<symbol\>[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong>Initial:&#xA;      </strong> | "&#x5C;2D" ("-" hyphen-minus)                                                                                                             |

<a id="ref-for-descdef-counter-style-negative②"></a>

The [negative](#descdef-counter-style-negative) descriptor defines how to alter the representation when the counter value is negative.

<a id="ref-for-typedef-symbol③"></a>

The first [\<symbol\>](#typedef-symbol) in the value is prepended to the representation when the counter value is negative. The second <a id="ref-for-typedef-symbol④"></a>\<symbol\>, if specified, is appended to the representation when the counter value is negative.

<a id="ref-for-descdef-counter-style-negative③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cea7bd84"></a> For example, specifying [negative: "(" ")";](#descdef-counter-style-negative) will make negative values be wrapped in parentheses, which is sometimes used in financial contexts, like "(2) (1) 0 1 2 3...".

<a id="ref-for-descdef-counter-style-system④"></a>

<a id="ref-for-valdef-system-symbolic③"></a>

<a id="ref-for-valdef-counter-style-system-alphabetic③"></a>

<a id="ref-for-valdef-counter-style-system-numeric③"></a>

<a id="ref-for-valdef-counter-style-system-additive③"></a>

<a id="ref-for-valdef-counter-style-system-extends④"></a>

<a id="ref-for-use-a-negative-sign②"></a>

<a id="ref-for-generate-a-counter②"></a>

Not all [system](#descdef-counter-style-system) values use a negative sign. In particular, a counter style <a id="use-a-negative-sign"></a>uses a negative sign if its <a id="ref-for-descdef-counter-style-system⑤"></a>system value is [symbolic](#valdef-system-symbolic), [alphabetic](#valdef-counter-style-system-alphabetic), [numeric](#valdef-counter-style-system-numeric), [additive](#valdef-counter-style-system-additive), or [extends](#valdef-counter-style-system-extends) if the extended counter style itself [uses a negative sign](#use-a-negative-sign). If a counter style does not <a id="ref-for-use-a-negative-sign③"></a>use a negative sign, it ignores the negative sign when [generating a counter representation](#generate-a-counter).

<a id="ref-for-descdef-counter-style-prefix①"></a>

### <a id="counter-style-prefix"></a>3.3.  Symbols before the marker: the [prefix](#descdef-counter-style-prefix) descriptor

| Field               | Definition                                                     |
|---------------------|----------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-counter-style-prefix"></a>prefix                                      |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-counter-style②②"></a>[@counter-style](#at-ruledef-counter-style) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-typedef-symbol⑤"></a>[\<symbol\>](#typedef-symbol)               |
| <strong>Initial:&#xA;      </strong> | "" (the empty string)                                          |

<a id="ref-for-descdef-counter-style-prefix②"></a>

<a id="ref-for-typedef-symbol⑥"></a>

The [prefix](#descdef-counter-style-prefix) descriptor specifies a [\<symbol\>](#typedef-symbol) that is prepended to the marker representation. Prefixes come before any negative sign.

<a id="ref-for-selectordef-marker①"></a>

<a id="ref-for-funcdef-counter②"></a>

<a id="ref-for-funcdef-counters①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Prefixes are only added by the algorithm for constructing the default contents of the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element; the prefix is not added automatically when the [counter()](https://www.w3.org/TR/css-lists-3/#funcdef-counter) or [counters()](https://www.w3.org/TR/css-lists-3/#funcdef-counters) functions are used.

<a id="ref-for-descdef-counter-style-suffix①"></a>

### <a id="counter-style-suffix"></a>3.4.  Symbols after the marker: the [suffix](#descdef-counter-style-suffix) descriptor

| Field               | Definition                                                               |
|---------------------|--------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-counter-style-suffix"></a>suffix                                                |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-counter-style②③"></a>[@counter-style](#at-ruledef-counter-style)           |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-typedef-symbol⑦"></a>[\<symbol\>](#typedef-symbol)                         |
| <strong>Initial:&#xA;      </strong> | "&#x5C;2E&#x5C;20" ("." full stop followed by a space) |

<a id="ref-for-descdef-counter-style-suffix②"></a>

<a id="ref-for-typedef-symbol⑧"></a>

The [suffix](#descdef-counter-style-suffix) descriptor specifies a [\<symbol\>](#typedef-symbol) that is appended to the marker representation. Suffixes are added to the representation after negative signs.

<a id="ref-for-selectordef-marker②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Suffixes are only added by the algorithm for constructing the default contents of the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element; the suffix is not added automatically when the counter() or counters() functions are used.

<a id="ref-for-descdef-counter-style-range①"></a>

### <a id="counter-style-range"></a>3.5.  Limiting the counter scope: the [range](#descdef-counter-style-range) descriptor

| Field               | Definition                                                                                                                                                                                                                                                                                                                                             |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-counter-style-range"></a>range                                                                                                                                                                                                                                                                                                                               |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-counter-style②④"></a>[@counter-style](#at-ruledef-counter-style)                                                                                                                                                                                                                                                                                         |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-comma"></a><a id="ref-for-mult-num"></a><a id="ref-for-comb-one⑥"></a><a id="ref-for-integer-value①"></a>\[ \[ [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) infinite \][{2}](https://www.w3.org/TR/css-values-4/#mult-num) \][\#](https://www.w3.org/TR/css-values-4/#mult-comma) <a id="ref-for-comb-one⑦"></a>\| auto |
| <strong>Initial:&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                   |

<a id="ref-for-descdef-counter-style-range②"></a>

The [range](#descdef-counter-style-range) descriptor defines the ranges over which the counter style is defined. If a counter style is used to represent a counter value outside of its ranges, the counter style instead drops down to its fallback counter style.

<a id="valdef-counter-style-range-auto"></a>auto

The range depends on the counter system:

- <a id="ref-for-valdef-counter-style-system-fixed②"></a>

  <a id="ref-for-valdef-counter-style-system-numeric④"></a>

  <a id="ref-for-valdef-counter-style-system-cyclic③"></a>

  For [cyclic](#valdef-counter-style-system-cyclic), [numeric](#valdef-counter-style-system-numeric), and [fixed](#valdef-counter-style-system-fixed) systems, the range is negative infinity to positive infinity.

- <a id="ref-for-valdef-system-symbolic④"></a>

  <a id="ref-for-valdef-counter-style-system-alphabetic④"></a>

  For [alphabetic](#valdef-counter-style-system-alphabetic) and [symbolic](#valdef-system-symbolic) systems, the range is 1 to positive infinity.

- <a id="ref-for-valdef-counter-style-system-additive④"></a>

  For [additive](#valdef-counter-style-system-additive) systems, the range is 0 to positive infinity.

- <a id="ref-for-valdef-counter-style-range-auto"></a>

  <a id="ref-for-valdef-counter-style-system-extends⑤"></a>

  For [extends](#valdef-counter-style-system-extends) systems, the range is whatever [auto](#valdef-counter-style-range-auto) would produce for the extended system; if extending a complex predefined style ([§ 7 Complex Predefined Counter Styles](#complex-predefined-counters)), the range is the style’s defined range.

<a id="ref-for-integer-value②"></a>

\[ \[ [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) \| infinite \]{2} \]#

This defines a comma-separated list of ranges. For each individual range, the first value is the lower bound and the second value is the upper bound. This range is inclusive - it contains both the lower and upper bound numbers. If infinite is used as the first value in a range, it represents negative infinity; if used as the second value, it represents positive infinity. The range of the counter style is the union of all the ranges defined in the list.

If the lower bound of any range is higher than the upper bound, the entire descriptor is invalid and must be ignored.

Implementations must support ranges with a lower bound of at least -2<sup>15</sup> and an upper bound of at least 2<sup>15</sup>-1 (the range of a signed 2-byte int). They may support higher ranges. If any specified bound is outside of the implementation’s supported bounds, it must be treated as the closest bound that the implementation does support.

<a id="ref-for-descdef-counter-style-pad②"></a>

### <a id="counter-style-pad"></a>3.6.  Zero-Padding and Constant-Width Representations: the [pad](#descdef-counter-style-pad) descriptor

| Field               | Definition                                                                                                                                                                                                                                     |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-counter-style-pad"></a>pad                                                                                                                                                                                                                         |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-counter-style②⑤"></a>[@counter-style](#at-ruledef-counter-style)                                                                                                                                                                                 |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-typedef-symbol⑨"></a><a id="ref-for-comb-all"></a><a id="ref-for-integer-value③"></a>[\<integer \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#integer-value) [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) [\<symbol\>](#typedef-symbol) |
| <strong>Initial:&#xA;      </strong> | 0 ""                                                                                                                                                                                                                                           |

<a id="ref-for-descdef-counter-style-pad③"></a>

<a id="ref-for-typedef-symbol①⓪"></a>

The [pad](#descdef-counter-style-pad) descriptor allows an author to specify a "fixed-width" counter style, where representations shorter than the pad value are padded with a particular [\<symbol\>](#typedef-symbol). Representations larger than the specified pad value are constructed as normal.

<a id="ref-for-typedef-symbol①①"></a>

<a id="ref-for-integer-value④"></a>

[\<integer \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#integer-value) &#x26;&#x26; [\<symbol\>](#typedef-symbol)

<a id="ref-for-integer-value⑤"></a>

The [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) specifies a minimum length that all counter representations must reach.

<a id="ref-for-integer-value⑥"></a>

<a id="ref-for-grapheme-cluster"></a>

<a id="ref-for-initial-representation-for-the-counter-value"></a>

<a id="ref-for-generate-a-counter③"></a>

<a id="ref-for-use-a-negative-sign④"></a>

<a id="ref-for-descdef-counter-style-negative④"></a>

<a id="ref-for-typedef-symbol①②"></a>

Let <var>difference</var> be the provided [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) minus the number of [grapheme clusters](https://www.w3.org/TR/css-text-3/#grapheme-cluster) in the [initial representation for the counter value](#initial-representation-for-the-counter-value). <strong data-conversion-semantic="note">Note:</strong> (Note that, per the algorithm to [generate a counter representation](#generate-a-counter), this occurs before adding prefixes/suffixes/negatives.) If the counter value is negative and the counter style [uses a negative sign](#use-a-negative-sign), further reduce <var>difference</var> by the number of <a id="ref-for-grapheme-cluster①"></a>grapheme clusters in the counter style’s [negative](#descdef-counter-style-negative) descriptor’s [\<symbol\>](#typedef-symbol)(s).

<a id="ref-for-typedef-symbol①③"></a>

If <var>difference</var> is greater than zero, prepend <var>difference</var> copies of the specified [\<symbol\>](#typedef-symbol) to the representation.

<a id="ref-for-integer-value⑦"></a>

Negative [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) values are not allowed.

<a id="ref-for-descdef-counter-style-pad④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9431be5f"></a> The most common example of "fixed-width" numbering is zero-padded decimal numbering. If an author knows that the numbers used will be less than a thousand, for example, it can be zero-padded with a simple [pad: 3 "0";](#descdef-counter-style-pad) descriptor, ensuring that all of the representations are 3 digits wide.
>
> This will cause, for example, 1 to be represented as "001", 20 to be represented as "020", 300 to be represented as "300", 4000 to be represented as "4000", and -5 to be represented as "-05".

<a id="ref-for-descdef-counter-style-pad⑤"></a>

<a id="ref-for-grapheme-cluster②"></a>

<a id="ref-for-typedef-symbol①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [pad](#descdef-counter-style-pad) descriptor counts the number of [grapheme clusters](https://www.w3.org/TR/css-text-3/#grapheme-cluster) in the representation, but pads it with [\<symbol\>](#typedef-symbol)s. If the specified <a id="ref-for-descdef-counter-style-pad⑥"></a>pad <a id="ref-for-typedef-symbol①⑤"></a>\<symbol\> is multi-character, this will likely not have the desired effect. Unfortunately, there’s no way to use the number of <a id="ref-for-grapheme-cluster③"></a>grapheme clusters in the <a id="ref-for-descdef-counter-style-pad⑦"></a>pad <a id="ref-for-typedef-symbol①⑥"></a>\<symbol\> without violating useful constraints. It is recommended that authors only specify <a id="ref-for-typedef-symbol①⑦"></a>\<symbol\>s of a single <a id="ref-for-grapheme-cluster④"></a>grapheme cluster in the <a id="ref-for-descdef-counter-style-pad⑧"></a>pad descriptor.

<a id="ref-for-descdef-counter-style-fallback"></a>

### <a id="counter-style-fallback"></a>3.7.  Defining fallback: the [fallback](#descdef-counter-style-fallback) descriptor

| Field               | Definition                                                               |
|---------------------|--------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-counter-style-fallback"></a>fallback                                              |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-counter-style②⑥"></a>[@counter-style](#at-ruledef-counter-style)           |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-typedef-counter-style-name⑤"></a>[\<counter-style-name\>](#typedef-counter-style-name) |
| <strong>Initial:&#xA;      </strong> | decimal                                                                  |

<a id="ref-for-descdef-counter-style-fallback①"></a>

The [fallback](#descdef-counter-style-fallback) descriptor specifies a fallback counter style to be used when the current counter style can’t create a representation for a given counter value. For example, if a counter style defined with a range of 1-10 is asked to represent a counter value of 11, the counter value’s representation is instead constructed with the fallback counter style (or possibly the fallback style’s fallback style, if the fallback style can’t represent that value, etc.).

<a id="ref-for-descdef-counter-style-fallback②"></a>

<a id="ref-for-decimal⑥"></a>

If the value of the [fallback](#descdef-counter-style-fallback) descriptor isn’t the name of any defined counter style, the used value of the <a id="ref-for-descdef-counter-style-fallback③"></a>fallback descriptor is [decimal](#decimal) instead. Similarly, while following fallbacks to find a counter style that can render the given counter value, if a loop in the specified fallbacks is detected, the <a id="ref-for-decimal⑦"></a>decimal style must be used instead.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that it is not necessarily an error to specify fallback loops. For example, if an author desires a counter style with significantly different representations for even and odd counter values, they may find it easiest to define one style that can only represent odd values and one that can only represent even values, and specify each as the fallback for the other one. Though the fallback graph is circular, at no point do you encounter a loop while following these fallbacks - every counter value is represented by one or the other counter style.

<a id="ref-for-descdef-counter-style-symbols⑦"></a>

<a id="ref-for-descdef-counter-style-additive-symbols④"></a>

### <a id="counter-style-symbols"></a>3.8.  Marker characters: the [symbols](#descdef-counter-style-symbols) and [additive-symbols](#descdef-counter-style-additive-symbols) descriptors

| Field               | Definition                                                                                                                |
|---------------------|---------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-counter-style-symbols"></a>symbols                                                                                                |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-counter-style②⑦"></a>[@counter-style](#at-ruledef-counter-style)                                                            |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-one-plus"></a><a id="ref-for-typedef-symbol①⑧"></a>[\<symbol\>](#typedef-symbol)[+](https://www.w3.org/TR/css-values-4/#mult-one-plus) |
| <strong>Initial:&#xA;      </strong> | n/a                                                                                                                       |

| Field               | Definition                                                                                                                                                                                                                                                                                                                  |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-counter-style-additive-symbols"></a>additive-symbols                                                                                                                                                                                                                                                                                         |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-counter-style②⑧"></a>[@counter-style](#at-ruledef-counter-style)                                                                                                                                                                                                                                                              |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-comma①"></a><a id="ref-for-typedef-symbol①⑨"></a><a id="ref-for-comb-all①"></a><a id="ref-for-integer-value⑧"></a>\[ [\<integer \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#integer-value) [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) [\<symbol\>](#typedef-symbol) \][\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong>Initial:&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                         |

<a id="typedef-symbol"></a>

<a id="ref-for-string-value"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-typedef-image①"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-identifier-value②"></a>

```text
<symbol> = <string> | <image> | <custom-ident>
```
<a id="ref-for-descdef-counter-style-symbols⑧"></a>

<a id="ref-for-descdef-counter-style-additive-symbols⑤"></a>

<a id="ref-for-descdef-counter-style-system⑥"></a>

<a id="ref-for-at-ruledef-counter-style②⑨"></a>

<a id="ref-for-valdef-counter-style-system-cyclic④"></a>

<a id="ref-for-valdef-counter-style-system-numeric⑤"></a>

<a id="ref-for-valdef-counter-style-system-alphabetic⑤"></a>

<a id="ref-for-valdef-system-symbolic⑤"></a>

<a id="ref-for-valdef-counter-style-system-fixed③"></a>

<a id="ref-for-valdef-counter-style-system-additive⑤"></a>

<a id="ref-for-counter-style③"></a>

<a id="ref-for-at-rule"></a>

The [symbols](#descdef-counter-style-symbols) and [additive-symbols](#descdef-counter-style-additive-symbols) descriptors specify the symbols used by the marker-construction algorithm specified by the [system](#descdef-counter-style-system) descriptor. The [@counter-style](#at-ruledef-counter-style) rule must have a valid <a id="ref-for-descdef-counter-style-symbols⑨"></a>symbols descriptor if the counter system is [cyclic](#valdef-counter-style-system-cyclic), [numeric](#valdef-counter-style-system-numeric), [alphabetic](#valdef-counter-style-system-alphabetic), [symbolic](#valdef-system-symbolic), or [fixed](#valdef-counter-style-system-fixed), or a valid <a id="ref-for-descdef-counter-style-additive-symbols⑥"></a>additive-symbols descriptor if the counter system is [additive](#valdef-counter-style-system-additive); otherwise, the <a id="ref-for-at-ruledef-counter-style③⓪"></a>@counter-style does not define a [counter style](#counter-style) (but is still a valid [at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule)).

<a id="ref-for-descdef-counter-style-symbols①⓪"></a>

<a id="ref-for-at-ruledef-counter-style③①"></a>

<a id="ref-for-counter-style④"></a>

Some counter systems specify that the [symbols](#descdef-counter-style-symbols) descriptor must have at least two entries. If the counter style’s system is such, and the <a id="ref-for-descdef-counter-style-symbols①①"></a>symbols descriptor has only a single entry, the [@counter-style](#at-ruledef-counter-style) rule does not define a [counter style](#counter-style).

<a id="ref-for-descdef-counter-style-symbols①②"></a>

<a id="ref-for-descdef-counter-style-additive-symbols⑦"></a>

<a id="ref-for-counter-symbol③④"></a>

<a id="ref-for-additive-tuple②"></a>

Each entry in the [symbols](#descdef-counter-style-symbols) descriptor’s value defines a <a id="counter-symbol"></a>counter symbol, which is interpreted differently based on the counter style’s system. Each entry in the [additive-symbols](#descdef-counter-style-additive-symbols) descriptor’s value defines an <a id="additive-tuple"></a>additive tuple, which consists of a [counter symbol](#counter-symbol) and an integer weight. Each weight must be a non-negative integer, and the [additive tuples](#additive-tuple) must be specified in order of strictly descending weight; otherwise, the declaration is invalid and must be ignored.

<a id="ref-for-counter-symbol③⑤"></a>

<a id="ref-for-default-object-size"></a>

[Counter symbols](#counter-symbol) may be strings, images, or identifiers, and the three types can be mixed in a single descriptor. Counter representations are constructed by concatenating counter symbols together. Identifiers are rendered as strings containing the same characters. Images are rendered as inline replaced elements. The [default object size](https://www.w3.org/TR/css-images-3/#default-object-size) of an image <a id="ref-for-counter-symbol③⑥"></a>counter symbol is a 1em by 1em square.

<a id="ref-for-typedef-image②"></a>

<a id="ref-for-typedef-symbol②⓪"></a>

<a id="ref-for-funcdef-counter③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) syntax in [\<symbol\>](#typedef-symbol) is currently at-risk. No implementations have plans to implement it currently, and it complicates some usages of [counter()](https://www.w3.org/TR/css-lists-3/#funcdef-counter) in ways that haven’t been fully handled.

<a id="ref-for-descdef-counter-style-symbols①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If using identifiers rather than strings to define the symbols, be aware of the syntax of identifiers. In particular, ascii non-letters like "\*" are not identifiers, and so must be quoted in a string. Hex escapes, used in several of the counter styles defined in this specification, "eat" the following space (to allow a digit to follow a hex escape without ambiguity), so two spaces must be put after a hex escape to separate it from the following one, or else they’ll be considered adjacent, and part of the same identifier. For example, [symbols: &#x5C;660 &#x5C;661;](#descdef-counter-style-symbols) only defines a single symbol, consisting of the U+0660 and U+0661 characters, rather than the two that were intended; either quote the escapes in strings, like <a id="ref-for-descdef-counter-style-symbols①④"></a>symbols: "&#x5C;660" "&#x5C;661", or put two spaces between the escapes.

<a id="ref-for-descdef-counter-style-speak-as"></a>

### <a id="counter-style-speak-as"></a>3.9.  Speech Synthesis: the [speak-as](#descdef-counter-style-speak-as) descriptor

| Field               | Definition                                                                                                                                                                                                                                                                  |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-counter-style-speak-as"></a>speak-as                                                                                                                                                                                                                                                 |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-counter-style③②"></a>[@counter-style](#at-ruledef-counter-style)                                                                                                                                                                                                              |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-typedef-counter-style-name⑥"></a><a id="ref-for-comb-one①⓪"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) bullets <a id="ref-for-comb-one①①"></a>\| numbers <a id="ref-for-comb-one①②"></a>\| words <a id="ref-for-comb-one①③"></a>\| spell-out <a id="ref-for-comb-one①④"></a>\| [\<counter-style-name\>](#typedef-counter-style-name) |
| <strong>Initial:&#xA;      </strong> | auto                                                                                                                                                                                                                                                                        |

<a id="ref-for-descdef-counter-style-speak-as①"></a>

A counter style can be constructed with a meaning that is obvious visually, but impossible to meaningfully represent via a speech synthesizer or other non-visual means, or possible but nonsensical when naively read out loud. The [speak-as](#descdef-counter-style-speak-as) descriptor describes how to synthesize the spoken form of a counter formatted with the given counter style. Assistive technologies should use this spoken form when reading out the counter style, and may use the <a id="ref-for-descdef-counter-style-speak-as②"></a>speak-as value to inform transformations to outputs other than speech. Values have the following meanings:

<a id="valdef-counter-style-speak-as-auto"></a>auto

<a id="ref-for-valdef-counter-style-speak-as-numbers"></a>

<a id="ref-for-valdef-counter-style-speak-as-auto"></a>

<a id="ref-for-valdef-counter-style-system-extends⑥"></a>

<a id="ref-for-valdef-counter-style-speak-as-bullets"></a>

<a id="ref-for-valdef-counter-style-system-cyclic⑤"></a>

<a id="ref-for-valdef-counter-style-speak-as-spell-out"></a>

<a id="ref-for-valdef-counter-style-system-alphabetic⑥"></a>

<a id="ref-for-descdef-counter-style-system⑦"></a>

If the counter style’s [system](#descdef-counter-style-system) is [alphabetic](#valdef-counter-style-system-alphabetic), this value has the same effect as [spell-out](#valdef-counter-style-speak-as-spell-out). If the <a id="ref-for-descdef-counter-style-system⑧"></a>system is [cyclic](#valdef-counter-style-system-cyclic), this value has the same effect as [bullets](#valdef-counter-style-speak-as-bullets). If the <a id="ref-for-descdef-counter-style-system⑨"></a>system is [extends](#valdef-counter-style-system-extends), this value has the same effect as [auto](#valdef-counter-style-speak-as-auto) would have for the extended style. Otherwise, this value has the same effect as [numbers](#valdef-counter-style-speak-as-numbers).

<a id="valdef-counter-style-speak-as-bullets"></a>bullets

The UA speaks a UA-defined phrase or audio cue that represents an unordered list item being read out.

<a id="valdef-counter-style-speak-as-numbers"></a>numbers

<a id="ref-for-content-language"></a>

The counter’s numeric value is spoken as a number in the [content language](https://www.w3.org/TR/css-text-3/#content-language).

<a id="valdef-counter-style-speak-as-words"></a>words

<a id="ref-for-valdef-counter-style-speak-as-numbers①"></a>

<a id="ref-for-content-language①"></a>

<a id="ref-for-generate-a-counter④"></a>

[Generate a counter representation](#generate-a-counter) for the value as normal, then speak it as normal text in the [content language](https://www.w3.org/TR/css-text-3/#content-language). If the counter representation contains images, instead handle the value as for [numbers](#valdef-counter-style-speak-as-numbers).

<a id="valdef-counter-style-speak-as-spell-out"></a>spell-out

<a id="ref-for-valdef-counter-style-speak-as-numbers②"></a>

<a id="ref-for-content-language②"></a>

<a id="ref-for-generate-a-counter⑤"></a>

[Generate a counter representation](#generate-a-counter) for the value as normal, then spell it out letter-by-letter in the [content language](https://www.w3.org/TR/css-text-3/#content-language). If the UA does not know how to pronounce the symbols (or the counter representation contains images), it must instead handle the value as for [numbers](#valdef-counter-style-speak-as-numbers).

<a id="ref-for-lower-greek"></a>

<a id="ref-for-upper-latin"></a>

For example, [lower-greek](#lower-greek) in English would be read out as "alpha", "beta", "gamma", etc. Conversely, [upper-latin](#upper-latin) in French would be read out as (in phonetic notation) /a/, /be/, /se/, etc.

<a id="ref-for-typedef-counter-style-name⑦"></a>

<a id="valdef-counter-style-speak-as-counter-style-name"></a>[\<counter-style-name\>](#typedef-counter-style-name)

<a id="ref-for-descdef-counter-style-speak-as③"></a>

<a id="ref-for-valdef-counter-style-speak-as-auto①"></a>

<a id="ref-for-descdef-counter-style-fallback④"></a>

The counter’s value is instead spoken out in the specified style (similar to the behavior of the [fallback](#descdef-counter-style-fallback) descriptor when generating representations for a counter value). If the specified style does not exist, this value is treated as [auto](#valdef-counter-style-speak-as-auto). If a loop is detected when following [speak-as](#descdef-counter-style-speak-as) references, this value is treated as <a id="ref-for-valdef-counter-style-speak-as-auto②"></a>auto for the counter styles participating in the loop.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cce4b0db"></a> The ability to defer pronunciation to another counter style can help when the symbols being used aren’t actually letters. For example, here’s a possible definition of a circled-lower-latin counter-style, using some special Unicode characters:
>
> <a id="circled-lower-latin"></a>
>
> ```text
> @counter-style circled-lower-latin {
>   system: alphabetic;
>   speak-as: lower-latin;
>   symbols: ⓐ ⓑ ⓒ ⓓ ⓔ ⓕ ⓖ ⓗ ⓘ ⓙ ⓚ ⓛ ⓜ ⓝ ⓞ ⓟ ⓠ ⓡ ⓢ ⓣ ⓤ ⓥ ⓦ ⓧ ⓨ ⓩ;
>   suffix: " ";
> }
> ```
>
> <a id="ref-for-descdef-counter-style-system①⓪"></a>
>
> <a id="ref-for-valdef-counter-style-system-alphabetic⑦"></a>
>
> <a id="ref-for-descdef-counter-style-speak-as④"></a>
>
> <a id="ref-for-lower-latin"></a>
>
> Setting its [system](#descdef-counter-style-system) to [alphabetic](#valdef-counter-style-system-alphabetic) would normally make the UA try to read out the names of the characters, but in this case that might be something like "Circled Letter A", which is unlikely to make sense. Instead, explicitly setting [speak-as](#descdef-counter-style-speak-as) to [lower-latin](#lower-latin) ensures that they get read out as their corresponding latin letters, as intended.

<a id="ref-for-funcdef-symbols"></a>

## <a id="symbols-function"></a>4.  Defining Anonymous Counter Styles: the [symbols()](#funcdef-symbols) function

<a id="ref-for-funcdef-symbols①"></a>

<a id="ref-for-counter-style⑤"></a>

<a id="ref-for-at-ruledef-counter-style③③"></a>

The [symbols()](#funcdef-symbols) function allows a [counter style](#counter-style) to be defined inline in a property value, for when a style is used only once in a stylesheet and defining a full [@counter-style](#at-ruledef-counter-style) rule would be overkill. It does not provide the full feature-set of the <a id="ref-for-at-ruledef-counter-style③④"></a>@counter-style rule, but provides a sufficient subset to still be useful. The syntax of the <a id="ref-for-funcdef-symbols②"></a>symbols() rule is:

<a id="funcdef-symbols"></a>

<a id="ref-for-typedef-symbols-type"></a>

<a id="ref-for-mult-opt②"></a>

<a id="ref-for-string-value①"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-typedef-image③"></a>

<a id="ref-for-mult-one-plus①"></a>

<a id="typedef-symbols-type"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-comb-one①⑨"></a>

```text
symbols() = symbols( <symbols-type>? [ <string> | <image> ]+ )
<symbols-type> = cyclic | numeric | alphabetic | symbolic | fixed
```
<a id="ref-for-funcdef-symbols③"></a>

<a id="ref-for-descdef-counter-style-prefix③"></a>

<a id="ref-for-descdef-counter-style-suffix③"></a>

<a id="ref-for-descdef-counter-style-range③"></a>

<a id="ref-for-valdef-counter-style-range-auto①"></a>

<a id="ref-for-descdef-counter-style-fallback⑤"></a>

<a id="ref-for-decimal⑧"></a>

<a id="ref-for-descdef-counter-style-negative⑤"></a>

<a id="ref-for-descdef-counter-style-pad⑨"></a>

<a id="ref-for-descdef-counter-style-speak-as⑤"></a>

<a id="ref-for-valdef-counter-style-speak-as-auto③"></a>

<a id="ref-for-valdef-system-symbolic⑥"></a>

<a id="ref-for-string-value②"></a>

<a id="ref-for-typedef-image④"></a>

<a id="ref-for-descdef-counter-style-symbols①⑤"></a>

<a id="ref-for-valdef-counter-style-system-fixed④"></a>

<a id="ref-for-first-symbol-value③"></a>

The [symbols()](#funcdef-symbols) function defines an anonymous counter style with no name, a [prefix](#descdef-counter-style-prefix) of "" (empty string) and [suffix](#descdef-counter-style-suffix) of " " (U+0020 SPACE), a [range](#descdef-counter-style-range) of [auto](#valdef-counter-style-range-auto), a [fallback](#descdef-counter-style-fallback) of [decimal](#decimal), a [negative](#descdef-counter-style-negative) of "&#x5C;2D" ("-" hyphen-minus), a [pad](#descdef-counter-style-pad) of 0 "", and a [speak-as](#descdef-counter-style-speak-as) of [auto](#valdef-counter-style-speak-as-auto). The counter style’s algorithm is constructed by consulting the previous chapter using the provided system — or [symbolic](#valdef-system-symbolic) if the system was omitted — and the provided [\<string\>](https://www.w3.org/TR/css-values-4/#string-value)s and [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)s as the value of the [symbols](#descdef-counter-style-symbols) property. If the system is [fixed](#valdef-counter-style-system-fixed), the [first symbol value](#first-symbol-value) is 1.

<a id="ref-for-valdef-counter-style-system-alphabetic⑧"></a>

<a id="ref-for-valdef-counter-style-system-numeric⑥"></a>

<a id="ref-for-string-value③"></a>

<a id="ref-for-typedef-image⑤"></a>

If the system is [alphabetic](#valdef-counter-style-system-alphabetic) or [numeric](#valdef-counter-style-system-numeric), there must be at least two [\<string\>](https://www.w3.org/TR/css-values-4/#string-value)s or [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)s, or else the function is invalid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-62664520"></a> This code:
>
> ```text
> ol { list-style: symbols("*" "\2020" "\2021" "\A7"); }
> ```
>
> will produce lists that look like:
>
> ```text
> *   One
> †   Two
> ‡   Three
> §   Four
> **  Five
> ††  Six
> ‡‡  Seven
> ```
>
> On the other hand, specifying the system of counter, like so:
>
> ```text
> ol { list-style: symbols(cyclic "*" "\2020" "\2021" "\A7"); }
> ```
>
> will produce lists that look like:
>
> ```text
> *   One
> †   Two
> ‡   Three
> §   Four
> *   Five
> †   Six
> ‡   Seven
> ```
<a id="ref-for-funcdef-symbols④"></a>

<a id="ref-for-descdef-counter-style-symbols①⑥"></a>

<a id="ref-for-at-ruledef-counter-style③⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: the [symbols()](#funcdef-symbols) function only allows strings and images, while the [symbols](#descdef-counter-style-symbols) descriptor of a [@counter-style](#at-ruledef-counter-style) rule also allows identifiers.

<a id="ref-for-propdef-list-style-type①"></a>

<a id="ref-for-funcdef-counter④"></a>

<a id="ref-for-funcdef-counters②"></a>

## <a id="extending-css2"></a>5.  Extending [list-style-type](https://www.w3.org/TR/css-lists-3/#propdef-list-style-type), [counter()](https://www.w3.org/TR/css-lists-3/#funcdef-counter), and [counters()](https://www.w3.org/TR/css-lists-3/#funcdef-counters)

<a id="ref-for-propdef-list-style-type②"></a>

<a id="ref-for-funcdef-counter⑤"></a>

<a id="ref-for-funcdef-counters③"></a>

<a id="ref-for-typedef-counter-style"></a>

In CSS Level 2 [\[CSS21\]](#biblio-css21) the [list-style-type](https://www.w3.org/TR/css-lists-3/#propdef-list-style-type) property and the [counter()](https://www.w3.org/TR/css-lists-3/#funcdef-counter) and [counters()](https://www.w3.org/TR/css-lists-3/#funcdef-counters) notations accept various pre-defined keywords, each identifying a counter style. This module extends these features to take instead the [\<counter-style\>](#typedef-counter-style) type, defined below:

<a id="typedef-counter-style"></a>

<a id="ref-for-typedef-counter-style-name⑧"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-funcdef-symbols⑤"></a>

```text
<counter-style> = <counter-style-name> | <symbols()>
```
<a id="ref-for-typedef-counter-style-name⑨"></a>

<a id="ref-for-decimal⑨"></a>

<a id="ref-for-computed-value"></a>

If a [\<counter-style-name\>](#typedef-counter-style-name) is used that does not refer to any existing counter style, it must act identically to the [decimal](#decimal) counter style (but does not <em><a href="https://www.w3.org/TR/css-cascade-5/#computed-value">compute</a></em> to <a id="ref-for-decimal①⓪"></a>decimal).

<a id="ref-for-typedef-counter-style-name①⓪"></a>

<a id="ref-for-css-tree-scoped-reference"></a>

When used in these contexts, a [\<counter-style-name\>](#typedef-counter-style-name) is a [tree-scoped reference](https://drafts.csswg.org/css-scoping-1/#css-tree-scoped-reference).

## <a id="predefined-counters"></a>6.  Simple Predefined Counter Styles

<a id="ref-for-at-ruledef-counter-style③⑥"></a>

The following stylesheet uses the [@counter-style](#at-ruledef-counter-style) rule to redefine all of the counter styles defined in CSS 2 and CSS 2.1. This stylesheet is normative—UAs must include it in their UA stylesheet (or at least act as if these rules were defined at that level).

<a id="ref-for-decimal①①"></a>

<a id="ref-for-decimal-leading-zero"></a>

<a id="ref-for-valdef-counter-style-name-arabic-indic"></a>

<a id="ref-for-armenian"></a>

<a id="ref-for-valdef-counter-style-name-upper-armenian"></a>

<a id="ref-for-valdef-counter-style-name-lower-armenian"></a>

<a id="ref-for-valdef-counter-style-name-bengali"></a>

<a id="ref-for-valdef-counter-style-name-cambodian"></a>

<a id="ref-for-valdef-counter-style-name-khmer"></a>

<a id="ref-for-cjk-decimal"></a>

<a id="ref-for-valdef-counter-style-name-devanagari"></a>

<a id="ref-for-georgian"></a>

<a id="ref-for-valdef-counter-style-name-gujarati"></a>

<a id="ref-for-valdef-counter-style-name-gurmukhi"></a>

<a id="ref-for-hebrew"></a>

<a id="ref-for-valdef-counter-style-name-kannada"></a>

<a id="ref-for-valdef-counter-style-name-lao"></a>

<a id="ref-for-valdef-counter-style-name-malayalam"></a>

<a id="ref-for-valdef-counter-style-name-mongolian"></a>

<a id="ref-for-valdef-counter-style-name-myanmar"></a>

<a id="ref-for-valdef-counter-style-name-oriya"></a>

<a id="ref-for-valdef-counter-style-name-persian"></a>

<a id="ref-for-lower-roman"></a>

<a id="ref-for-upper-roman"></a>

<a id="ref-for-valdef-counter-style-name-tamil"></a>

<a id="ref-for-valdef-counter-style-name-telugu"></a>

<a id="ref-for-valdef-counter-style-name-thai"></a>

<a id="ref-for-valdef-counter-style-name-tibetan"></a>

### <a id="simple-numeric"></a>6.1.  Numeric: [decimal](#decimal), [decimal-leading-zero](#decimal-leading-zero), [arabic-indic](#valdef-counter-style-name-arabic-indic), [armenian](#armenian), [upper-armenian](#valdef-counter-style-name-upper-armenian), [lower-armenian](#valdef-counter-style-name-lower-armenian), [bengali](#valdef-counter-style-name-bengali), [cambodian](#valdef-counter-style-name-cambodian), [khmer](#valdef-counter-style-name-khmer), [cjk-decimal](#cjk-decimal), [devanagari](#valdef-counter-style-name-devanagari), [georgian](#georgian), [gujarati](#valdef-counter-style-name-gujarati), [gurmukhi](#valdef-counter-style-name-gurmukhi), [hebrew](#hebrew), [kannada](#valdef-counter-style-name-kannada), [lao](#valdef-counter-style-name-lao), [malayalam](#valdef-counter-style-name-malayalam), [mongolian](#valdef-counter-style-name-mongolian), [myanmar](#valdef-counter-style-name-myanmar), [oriya](#valdef-counter-style-name-oriya), [persian](#valdef-counter-style-name-persian), [lower-roman](#lower-roman), [upper-roman](#upper-roman), [tamil](#valdef-counter-style-name-tamil), [telugu](#valdef-counter-style-name-telugu), [thai](#valdef-counter-style-name-thai), [tibetan](#valdef-counter-style-name-tibetan)

<a id="decimal"></a>decimal  
Western decimal numbers (e.g., 1, 2, 3, ..., 98, 99, 100).

<a id="decimal-leading-zero"></a>decimal-leading-zero  
Decimal numbers padded by initial zeros (e.g., 01, 02, 03, ..., 98, 99, 100).

<a id="valdef-counter-style-name-arabic-indic"></a>arabic-indic  
Arabic-indic numbering (e.g., ١‎, ٢‎, ٣‎, ٤‎, ..., ٩٨‎, ٩٩‎, ١٠٠‎).

<a id="armenian"></a>armenian  
<a id="valdef-counter-style-name-upper-armenian"></a>upper-armenian  
Traditional uppercase Armenian numbering (e.g., Ա, Բ, Գ, ..., ՂԸ, ՂԹ, Ճ).

<a id="valdef-counter-style-name-lower-armenian"></a>lower-armenian  
Lowercase Armenian numbering (e.g., ա, բ, գ, ..., ղը, ղթ, ճ).

<a id="valdef-counter-style-name-bengali"></a>bengali  
Bengali numbering (e.g., ১, ২, ৩, ..., ৯৮, ৯৯, ১০০).

<a id="valdef-counter-style-name-cambodian"></a>cambodian  
<a id="valdef-counter-style-name-khmer"></a>khmer  
Cambodian/Khmer numbering (e.g., ១, ២, ៣, ..., ៩៨, ៩៩, ១០០).

<a id="cjk-decimal"></a>cjk-decimal  
Han decimal numbers (e.g., 一, 二, 三, ..., 九八, 九九, 一〇〇).

<a id="valdef-counter-style-name-devanagari"></a>devanagari  
devanagari numbering (e.g., १, २, ३, ..., ९८, ९९, १००).

<a id="georgian"></a>georgian  
Traditional Georgian numbering (e.g., ა, ბ, გ, ..., ჟჱ, ჟთ, რ).

<a id="valdef-counter-style-name-gujarati"></a>gujarati  
Gujarati numbering (e.g., ૧, ૨, ૩, ..., ૯૮, ૯૯, ૧૦૦).

<a id="valdef-counter-style-name-gurmukhi"></a>gurmukhi  
Gurmukhi numbering (e.g., ੧, ੨, ੩, ..., ੯੮, ੯੯, ੧੦੦).

<a id="hebrew"></a>hebrew  
Traditional Hebrew numbering (e.g., א‎, ב‎, ג‎, ..., צח‎, צט‎, ק‎).

<a id="valdef-counter-style-name-kannada"></a>kannada  
Kannada numbering (e.g., ೧, ೨, ೩, ..., ೯೮, ೯೯, ೧೦೦).

<a id="valdef-counter-style-name-lao"></a>lao  
Laotian numbering (e.g., ໑, ໒, ໓, ..., ໙໘, ໙໙, ໑໐໐).

<a id="valdef-counter-style-name-malayalam"></a>malayalam  
Malayalam numbering (e.g., ൧, ൨, ൩, ..., ൯൮, ൯൯, ൧൦൦).

<a id="valdef-counter-style-name-mongolian"></a>mongolian  
Mongolian numbering (e.g., ᠑, ᠒, ᠓, ..., ᠙᠘, ᠙᠙, ᠑᠐᠐).

<a id="valdef-counter-style-name-myanmar"></a>myanmar  
Myanmar (Burmese) numbering (e.g., ၁, ၂, ၃, ..., ၉၈, ၉၉, ၁၀၀).

<a id="valdef-counter-style-name-oriya"></a>oriya  
Oriya numbering (e.g., ୧, ୨, ୩, ..., ୯୮, ୯୯, ୧୦୦).

<a id="valdef-counter-style-name-persian"></a>persian  
Persian numbering (e.g., ۱, ۲, ۳, ۴, ..., ۹۸, ۹۹, ۱۰۰).

<a id="lower-roman"></a>lower-roman  
Lowercase ASCII Roman numerals (e.g., i, ii, iii, ..., xcviii, xcix, c).

<a id="upper-roman"></a>upper-roman  
Uppercase ASCII Roman numerals (e.g., I, II, III, ..., XCVIII, XCIX, C).

<a id="valdef-counter-style-name-tamil"></a>tamil  
Tamil numbering (e.g., ௧, ௨, ௩, ..., ௯௮, ௯௯, ௧௦௦).

<a id="valdef-counter-style-name-telugu"></a>telugu  
Telugu numbering (e.g., ౧, ౨, ౩, ..., ౯౮, ౯౯, ౧౦౦).

<a id="valdef-counter-style-name-thai"></a>thai  
Thai (Siamese) numbering (e.g., ๑, ๒, ๓, ..., ๙๘, ๙๙, ๑๐๐).

<a id="valdef-counter-style-name-tibetan"></a>tibetan  
Tibetan numbering (e.g., ༡, ༢, ༣, ..., ༩༨, ༩༩, ༡༠༠).

The following stylesheet fragment provides the normative definition of these predefined counter styles:

```text
@counter-style decimal {
  system: numeric;
  symbols: '0' '1' '2' '3' '4' '5' '6' '7' '8' '9';
}

@counter-style decimal-leading-zero {
  system: extends decimal;
  pad: 2 '0';
}

@counter-style arabic-indic {
  system: numeric;
  symbols: "\660" "\661" "\662" "\663" "\664" "\665" "\666" "\667" "\668" "\669";
  /* ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩ */
}

@counter-style armenian {
  system: additive;
  range: 1 9999;
  additive-symbols: 9000 \554, 8000 \553, 7000 \552, 6000 \551, 5000 \550, 4000 \54F, 3000 \54E, 2000 \54D, 1000 \54C, 900 \54B, 800 \54A, 700 \549, 600 \548, 500 \547, 400 \546, 300 \545, 200 \544, 100 \543, 90 \542, 80 \541, 70 \540, 60 \53F, 50 \53E, 40 \53D, 30 \53C, 20 \53B, 10 \53A, 9 \539, 8 \538, 7 \537, 6 \536, 5 \535, 4 \534, 3 \533, 2 \532, 1 \531;
  /* 9000 Ք, 8000 Փ, 7000 Ւ, 6000 Ց, 5000 Ր, 4000 Տ, 3000 Վ, 2000 Ս, 1000 Ռ, 900 Ջ, 800 Պ, 700 Չ, 600 Ո, 500 Շ, 400 Ն, 300 Յ, 200 Մ, 100 Ճ, 90 Ղ, 80 Ձ, 70 Հ, 60 Կ, 50 Ծ, 40 Խ, 30 Լ, 20 Ի, 10 Ժ, 9 Թ, 8 Ը, 7 Է, 6 Զ, 5 Ե, 4 Դ, 3 Գ, 2 Բ, 1 Ա */
}

@counter-style upper-armenian {
  system: extends armenian;
}

@counter-style lower-armenian {
  system: additive;
  range: 1 9999;
  additive-symbols: 9000 "\584", 8000 "\583", 7000 "\582", 6000 "\581", 5000 "\580", 4000 "\57F", 3000 "\57E", 2000 "\57D", 1000 "\57C", 900 "\57B", 800 "\57A", 700 "\579", 600 "\578", 500 "\577", 400 "\576", 300 "\575", 200 "\574", 100 "\573", 90 "\572", 80 "\571", 70 "\570", 60 "\56F", 50 "\56E", 40 "\56D", 30 "\56C", 20 "\56B", 10 "\56A", 9 "\569", 8 "\568", 7 "\567", 6 "\566", 5 "\565", 4 "\564", 3 "\563", 2 "\562", 1 "\561";
  /* 9000 ք, 8000 փ, 7000 ւ, 6000 ց, 5000 ր, 4000 տ, 3000 վ, 2000 ս, 1000 ռ, 900 ջ, 800 պ, 700 չ, 600 ո, 500 շ, 400 ն, 300 յ, 200 մ, 100 ճ, 90 ղ, 80 ձ, 70 հ, 60 կ, 50 ծ, 40 խ, 30 լ, 20 ի, 10 ժ, 9 թ, 8 ը, 7 է, 6 զ, 5 ե, 4 դ, 3 գ, 2 բ, 1 ա */
}

@counter-style bengali {
  system: numeric;
  symbols: "\9E6" "\9E7" "\9E8" "\9E9" "\9EA" "\9EB" "\9EC" "\9ED" "\9EE" "\9EF";
  /* ০ ১ ২ ৩ ৪ ৫ ৬ ৭ ৮ ৯ */
}

@counter-style cambodian {
  system: numeric;
  symbols: "\17E0" "\17E1" "\17E2" "\17E3" "\17E4" "\17E5" "\17E6" "\17E7" "\17E8" "\17E9";
  /* ០ ១ ២ ៣ ៤ ៥ ៦ ៧ ៨ ៩ */
}

@counter-style khmer {
  system: extends cambodian;
}

@counter-style cjk-decimal {
  system: numeric;
  range: 0 infinite;
  symbols: \3007  \4E00  \4E8C  \4E09  \56DB  \4E94  \516D  \4E03  \516B  \4E5D;
  /* 〇 一 二 三 四 五 六 七 八 九 */
  suffix: "\3001";
  /* "、" */
}

@counter-style devanagari {
  system: numeric;
  symbols: "\966" "\967" "\968" "\969" "\96A" "\96B" "\96C" "\96D" "\96E" "\96F";
  /* ० १ २ ३ ४ ५ ६ ७ ८ ९ */
}

@counter-style georgian {
  system: additive;
  range: 1 19999;
  additive-symbols: 10000 \10F5, 9000 \10F0, 8000 \10EF, 7000 \10F4, 6000 \10EE, 5000 \10ED, 4000 \10EC, 3000 \10EB, 2000 \10EA, 1000 \10E9, 900 \10E8, 800 \10E7, 700 \10E6, 600 \10E5, 500 \10E4, 400 \10F3, 300 \10E2, 200 \10E1, 100 \10E0, 90 \10DF, 80 \10DE, 70 \10DD, 60 \10F2, 50 \10DC, 40 \10DB, 30 \10DA, 20 \10D9, 10 \10D8, 9 \10D7, 8 \10F1, 7 \10D6, 6 \10D5, 5 \10D4, 4 \10D3, 3 \10D2, 2 \10D1, 1 \10D0;
  /* 10000 ჵ, 9000 ჰ, 8000 ჯ, 7000 ჴ, 6000 ხ, 5000 ჭ, 4000 წ, 3000 ძ, 2000 ც, 1000 ჩ, 900 შ, 800 ყ, 700 ღ, 600 ქ, 500 ფ, 400 ჳ, 300 ტ, 200 ს, 100 რ, 90 ჟ, 80 პ, 70 ო, 60 ჲ, 50 ნ, 40 მ, 30 ლ, 20 კ, 10 ი, 9 თ, 8 ჱ, 7 ზ, 6 ვ, 5 ე, 4 დ, 3 გ, 2 ბ, 1 ა */
}

@counter-style gujarati {
  system: numeric;
  symbols: "\AE6" "\AE7" "\AE8" "\AE9" "\AEA" "\AEB" "\AEC" "\AED" "\AEE" "\AEF";
  /* ૦ ૧ ૨ ૩ ૪ ૫ ૬ ૭ ૮ ૯ */
}

@counter-style gurmukhi {
  system: numeric;
  symbols: "\A66" "\A67" "\A68" "\A69" "\A6A" "\A6B" "\A6C" "\A6D" "\A6E" "\A6F";
  /* ੦ ੧ ੨ ੩ ੪ ੫ ੬ ੭ ੮ ੯ */
}

@counter-style hebrew {
  system: additive;
  range: 1 10999;
  additive-symbols: 10000 \5D9\5F3, 9000 \5D8\5F3, 8000 \5D7\5F3, 7000 \5D6\5F3, 6000 \5D5\5F3, 5000 \5D4\5F3, 4000 \5D3\5F3, 3000 \5D2\5F3, 2000 \5D1\5F3, 1000 \5D0\5F3, 400 \5EA, 300 \5E9, 200 \5E8, 100 \5E7, 90 \5E6, 80 \5E4, 70 \5E2, 60 \5E1, 50 \5E0, 40 \5DE, 30 \5DC, 20 \5DB, 19 \5D9\5D8, 18 \5D9\5D7, 17 \5D9\5D6, 16 \5D8\5D6, 15 \5D8\5D5, 10 \5D9, 9 \5D8, 8 \5D7, 7 \5D6, 6 \5D5, 5 \5D4, 4 \5D3, 3 \5D2, 2 \5D1, 1 \5D0;
  /* 10000 י׳, 9000 ט׳, 8000 ח׳, 7000 ז׳, 6000 ו׳, 5000 ה׳, 4000 ד׳, 3000 ג׳, 2000 ב׳, 1000 א׳, 400 ת, 300 ש, 200 ר, 100 ק, 90 צ, 80 פ, 70 ע, 60 ס, 50 נ, 40 מ, 30 ל, 20 כ, 19 יט, 18 יח, 17 יז, 16 טז, 15 טו, 10 י, 9 ט, 8 ח, 7 ז, 6 ו, 5 ה, 4 ד, 3 ג, 2 ב, 1 א */
  /* This system manually specifies the values for 19-15 to force the correct display of 15 and 16, which are commonly rewritten to avoid a close resemblance to the Tetragrammaton. */
  /* Implementations MAY choose to implement this manually to a higher range; see note below. */
}

@counter-style kannada {
  system: numeric;
  symbols: "\CE6" "\CE7" "\CE8" "\CE9" "\CEA" "\CEB" "\CEC" "\CED" "\CEE" "\CEF";
  /* ೦ ೧ ೨ ೩ ೪ ೫ ೬ ೭ ೮ ೯ */
}

@counter-style lao {
  system: numeric;
  symbols: "\ED0" "\ED1" "\ED2" "\ED3" "\ED4" "\ED5" "\ED6" "\ED7" "\ED8" "\ED9";
  /* ໐ ໑ ໒ ໓ ໔ ໕ ໖ ໗ ໘ ໙ */
}

@counter-style malayalam {
  system: numeric;
  symbols: "\D66" "\D67" "\D68" "\D69" "\D6A" "\D6B" "\D6C" "\D6D" "\D6E" "\D6F";
  /* ൦ ൧ ൨ ൩ ൪ ൫ ൬ ൭ ൮ ൯ */
}

@counter-style mongolian {
  system: numeric;
  symbols: "\1810" "\1811" "\1812" "\1813" "\1814" "\1815" "\1816" "\1817" "\1818" "\1819";
  /* ᠐ ᠑ ᠒ ᠓ ᠔ ᠕ ᠖ ᠗ ᠘ ᠙ */
}

@counter-style myanmar {
  system: numeric;
  symbols: "\1040" "\1041" "\1042" "\1043" "\1044" "\1045" "\1046" "\1047" "\1048" "\1049";
  /* ၀ ၁ ၂ ၃ ၄ ၅ ၆ ၇ ၈ ၉ */
}

@counter-style oriya {
  system: numeric;
  symbols: "\B66" "\B67" "\B68" "\B69" "\B6A" "\B6B" "\B6C" "\B6D" "\B6E" "\B6F";
  /* ୦ ୧ ୨ ୩ ୪ ୫ ୬ ୭ ୮ ୯ */
}

@counter-style persian {
  system: numeric;
  symbols: "\6F0" "\6F1" "\6F2" "\6F3" "\6F4" "\6F5" "\6F6" "\6F7" "\6F8" "\6F9";
  /* ۰ ۱ ۲ ۳ ۴ ۵ ۶ ۷ ۸ ۹ */
}

@counter-style lower-roman {
  system: additive;
  range: 1 3999;
  additive-symbols: 1000 m, 900 cm, 500 d, 400 cd, 100 c, 90 xc, 50 l, 40 xl, 10 x, 9 ix, 5 v, 4 iv, 1 i;
}

@counter-style upper-roman {
  system: additive;
  range: 1 3999;
  additive-symbols: 1000 M, 900 CM, 500 D, 400 CD, 100 C, 90 XC, 50 L, 40 XL, 10 X, 9 IX, 5 V, 4 IV, 1 I;
}

@counter-style tamil {
  system: numeric;
  symbols: "\BE6" "\BE7" "\BE8" "\BE9" "\BEA" "\BEB" "\BEC" "\BED" "\BEE" "\BEF";
  /* ௦ ௧ ௨ ௩ ௪ ௫ ௬ ௭ ௮ ௯ */
}

@counter-style telugu {
  system: numeric;
  symbols: "\C66" "\C67" "\C68" "\C69" "\C6A" "\C6B" "\C6C" "\C6D" "\C6E" "\C6F";
  /* ౦ ౧ ౨ ౩ ౪ ౫ ౬ ౭ ౮ ౯ */
}

@counter-style thai {
  system: numeric;
  symbols: "\E50" "\E51" "\E52" "\E53" "\E54" "\E55" "\E56" "\E57" "\E58" "\E59";
  /* ๐ ๑ ๒ ๓ ๔ ๕ ๖ ๗ ๘ ๙ */
}

@counter-style tibetan {
  system: numeric;
  symbols: "\F20" "\F21" "\F22" "\F23" "\F24" "\F25" "\F26" "\F27" "\F28" "\F29";
  /* ༠ ༡ ༢ ༣ ༤ ༥ ༦ ༧ ༨ ༩ */
}
```
<a id="ref-for-hebrew①"></a>

<a id="ref-for-at-ruledef-counter-style③⑦"></a>

<a id="ref-for-descdef-counter-style-range④"></a>

Implementations must implement [hebrew](#hebrew) at least to the range specified in the [@counter-style](#at-ruledef-counter-style) rule above, but may implement it to a higher range. If they do so, the corresponding [range](#descdef-counter-style-range) descriptor must reflect the implemented range.

<a id="ref-for-lower-alpha①"></a>

<a id="ref-for-lower-latin①"></a>

<a id="ref-for-upper-alpha④"></a>

<a id="ref-for-upper-latin①"></a>

<a id="ref-for-lower-greek①"></a>

<a id="ref-for-hiragana"></a>

<a id="ref-for-hiragana-iroha"></a>

<a id="ref-for-katakana"></a>

<a id="ref-for-katakana-iroha"></a>

### <a id="simple-alphabetic"></a>6.2.  Alphabetic: [lower-alpha](#lower-alpha), [lower-latin](#lower-latin), [upper-alpha](#upper-alpha), [upper-latin](#upper-latin), [lower-greek](#lower-greek), [hiragana](#hiragana), [hiragana-iroha](#hiragana-iroha), [katakana](#katakana), [katakana-iroha](#katakana-iroha)

<a id="lower-alpha"></a>lower-alpha  
<a id="lower-latin"></a>lower-latin  
Lowercase ASCII letters (e.g., a, b, c, ..., z, aa, ab).

<a id="upper-alpha"></a>upper-alpha  
<a id="upper-latin"></a>upper-latin  
Uppercase ASCII letters (e.g., A, B, C, ..., Z, AA, AB).

<a id="lower-greek"></a>lower-greek  
Lowercase classical Greek (e.g., α, β, γ, ..., ω, αα, αβ).

<a id="hiragana"></a>hiragana  
Dictionary-order hiragana lettering (e.g., あ, い, う, ..., ん, ああ, あい).

<a id="hiragana-iroha"></a>hiragana-iroha  
Iroha-order hiragana lettering (e.g., い, ろ, は, ..., す, いい, いろ).

<a id="katakana"></a>katakana  
Dictionary-order katakana lettering (e.g., ア, イ, ウ, ..., ン, アア, アイ).

<a id="katakana-iroha"></a>katakana-iroha  
Iroha-order katakana lettering (e.g., イ, ロ, ハ, ..., ス, イイ, イロ)

The following stylesheet fragment provides the normative definition of these predefined counter styles:

```text
@counter-style lower-alpha {
  system: alphabetic;
  symbols: a b c d e f g h i j k l m n o p q r s t u v w x y z;
}

@counter-style lower-latin {
  system: extends lower-alpha;
}

@counter-style upper-alpha {
  system: alphabetic;
  symbols: A B C D E F G H I J K L M N O P Q R S T U V W X Y Z;
}

@counter-style upper-latin {
  system: extends upper-alpha;
}

@counter-style lower-greek {
  system: alphabetic;
  symbols: "\3B1" "\3B2" "\3B3" "\3B4" "\3B5" "\3B6" "\3B7" "\3B8" "\3B9" "\3BA" "\3BB" "\3BC" "\3BD" "\3BE" "\3BF" "\3C0" "\3C1" "\3C3" "\3C4" "\3C5" "\3C6" "\3C7" "\3C8" "\3C9";
  /* α β γ δ ε ζ η θ ι κ λ μ ν ξ ο π ρ σ τ υ φ χ ψ ω */
}

@counter-style hiragana {
  system: alphabetic;
  symbols: "\3042" "\3044" "\3046" "\3048" "\304A" "\304B" "\304D" "\304F" "\3051" "\3053" "\3055" "\3057" "\3059" "\305B" "\305D" "\305F" "\3061" "\3064" "\3066" "\3068" "\306A" "\306B" "\306C" "\306D" "\306E" "\306F" "\3072" "\3075" "\3078" "\307B" "\307E" "\307F" "\3080" "\3081" "\3082" "\3084" "\3086" "\3088" "\3089" "\308A" "\308B" "\308C" "\308D" "\308F" "\3090" "\3091" "\3092" "\3093";
  /* あ い う え お か き く け こ さ し す せ そ た ち つ て と な に ぬ ね の は ひ ふ へ ほ ま み む め も や ゆ よ ら り る れ ろ わ ゐ ゑ を ん */
  suffix: "、";
}

@counter-style hiragana-iroha {
  system: alphabetic;
  symbols: "\3044" "\308D" "\306F" "\306B" "\307B" "\3078" "\3068" "\3061" "\308A" "\306C" "\308B" "\3092" "\308F" "\304B" "\3088" "\305F" "\308C" "\305D" "\3064" "\306D" "\306A" "\3089" "\3080" "\3046" "\3090" "\306E" "\304A" "\304F" "\3084" "\307E" "\3051" "\3075" "\3053" "\3048" "\3066" "\3042" "\3055" "\304D" "\3086" "\3081" "\307F" "\3057" "\3091" "\3072" "\3082" "\305B" "\3059";
  /* い ろ は に ほ へ と ち り ぬ る を わ か よ た れ そ つ ね な ら む う ゐ の お く や ま け ふ こ え て あ さ き ゆ め み し ゑ ひ も せ す */
  suffix: "、";
}

@counter-style katakana {
  system: alphabetic;
  symbols: "\30A2" "\30A4" "\30A6" "\30A8" "\30AA" "\30AB" "\30AD" "\30AF" "\30B1" "\30B3" "\30B5" "\30B7" "\30B9" "\30BB" "\30BD" "\30BF" "\30C1" "\30C4" "\30C6" "\30C8" "\30CA" "\30CB" "\30CC" "\30CD" "\30CE" "\30CF" "\30D2" "\30D5" "\30D8" "\30DB" "\30DE" "\30DF" "\30E0" "\30E1" "\30E2" "\30E4" "\30E6" "\30E8" "\30E9" "\30EA" "\30EB" "\30EC" "\30ED" "\30EF" "\30F0" "\30F1" "\30F2" "\30F3";
  /* ア イ ウ エ オ カ キ ク ケ コ サ シ ス セ ソ タ チ ツ テ ト ナ ニ ヌ ネ ノ ハ ヒ フ ヘ ホ マ ミ ム メ モ ヤ ユ ヨ ラ リ ル レ ロ ワ ヰ ヱ ヲ ン */
  suffix: "、";
}

@counter-style katakana-iroha {
  system: alphabetic;
  symbols: "\30A4" "\30ED" "\30CF" "\30CB" "\30DB" "\30D8" "\30C8" "\30C1" "\30EA" "\30CC" "\30EB" "\30F2" "\30EF" "\30AB" "\30E8" "\30BF" "\30EC" "\30BD" "\30C4" "\30CD" "\30CA" "\30E9" "\30E0" "\30A6" "\30F0" "\30CE" "\30AA" "\30AF" "\30E4" "\30DE" "\30B1" "\30D5" "\30B3" "\30A8" "\30C6" "\30A2" "\30B5" "\30AD" "\30E6" "\30E1" "\30DF" "\30B7" "\30F1" "\30D2" "\30E2" "\30BB" "\30B9";
  /* イ ロ ハ ニ ホ ヘ ト チ リ ヌ ル ヲ ワ カ ヨ タ レ ソ ツ ネ ナ ラ ム ウ ヰ ノ オ ク ヤ マ ケ フ コ エ テ ア サ キ ユ メ ミ シ ヱ ヒ モ セ ス */
  suffix: "、";
}
```
<a id="ref-for-disc②"></a>

<a id="ref-for-circle②"></a>

<a id="ref-for-square②"></a>

<a id="ref-for-disclosure-open②"></a>

<a id="ref-for-disclosure-closed②"></a>

### <a id="simple-symbolic"></a>6.3.  Symbolic: [disc](#disc), [circle](#circle), [square](#square), [disclosure-open](#disclosure-open), [disclosure-closed](#disclosure-closed)

<a id="disc"></a>disc  
A filled circle, similar to • U+2022 BULLET.

<a id="circle"></a>circle  
A hollow circle, similar to ◦ U+25E6 WHITE BULLET.

<a id="square"></a>square  
A filled square, similar to ▪ U+25AA BLACK SMALL SQUARE.

<a id="disclosure-open"></a>disclosure-open  
<a id="disclosure-closed"></a>disclosure-closed  
<a id="ref-for-the-details-element"></a>

Symbols appropriate for indicating an open or closed disclosure widget, such as the HTML <code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element">details</a></code> element.

The following stylesheet fragment provides the normative definition of these predefined counter styles:

```text
@counter-style disc {
  system: cyclic;
  symbols: \2022;
  /* • */
  suffix: " ";
}

@counter-style circle {
  system: cyclic;
  symbols: \25E6;
  /* ◦ */
  suffix: " ";
}

@counter-style square {
  system: cyclic;
  symbols: \25AA;
  /* ▪ */
  suffix: " ";
}

@counter-style disclosure-open {
  system: cyclic;
  suffix: " ";
  /* for symbols, see normative text below */
}

@counter-style disclosure-closed {
  system: cyclic;
  suffix: " ";
  /* for symbols, see normative text below */
}
```
<a id="ref-for-propdef-list-style-type③"></a>

When used in [list-style-type](https://www.w3.org/TR/css-lists-3/#propdef-list-style-type), a UA may instead render these styles using a UA-generated image or a UA-chosen font instead of rendering the specified character in the element’s own font. If using an image, it must look similar to the character, and must be sized to attractively fill a 1em by 1em square.

<a id="ref-for-disclosure-open③"></a>

<a id="ref-for-disclosure-closed③"></a>

<a id="ref-for-the-details-element①"></a>

<a id="ref-for-writing-mode"></a>

For the [disclosure-open](#disclosure-open) and [disclosure-closed](#disclosure-closed) counter styles, the marker must be an image or character suitable for indicating the open and closed states of a disclosure widget, such as HTML’s <code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element">details</a></code> element. If the image is directional, it must respond to the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the element, similar to the [bidi-sensitive images](https://drafts.csswg.org/css4-images/#bidi-images) feature of the Images 4 module. For example, the <a id="ref-for-disclosure-closed④"></a>disclosure-closed style might use the characters U+25B8 BLACK RIGHT-POINTING SMALL TRIANGLE (▸) and U+25C2 BLACK LEFT-POINTING SMALL TRIANGLE (◂), while the <a id="ref-for-disclosure-open④"></a>disclosure-open style might use the character U+25BE BLACK DOWN-POINTING SMALL TRIANGLE (▾).

<a id="ref-for-valdef-counter-style-name-cjk-earthly-branch"></a>

<a id="ref-for-valdef-counter-style-name-cjk-heavenly-stem"></a>

### <a id="simple-fixed"></a>6.4.  Fixed: [cjk-earthly-branch](#valdef-counter-style-name-cjk-earthly-branch), [cjk-heavenly-stem](#valdef-counter-style-name-cjk-heavenly-stem)

<a id="valdef-counter-style-name-cjk-earthly-branch"></a>cjk-earthly-branch  
Han "Earthly Branch" ordinals (e.g., 子, 丑, 寅, ..., 亥).

<a id="valdef-counter-style-name-cjk-heavenly-stem"></a>cjk-heavenly-stem  
Han "Heavenly Stem" ordinals (e.g., 甲, 乙, 丙, ..., 癸)

The following stylesheet fragment provides the normative definition of these predefined counter styles:

```text
@counter-style cjk-earthly-branch {
  system: fixed;
  symbols: "\5B50" "\4E11" "\5BC5" "\536F" "\8FB0" "\5DF3" "\5348" "\672A" "\7533" "\9149" "\620C" "\4EA5";
  /* 子 丑 寅 卯 辰 巳 午 未 申 酉 戌 亥 */
  suffix: "、";
}

@counter-style cjk-heavenly-stem {
  system: fixed;
  symbols: "\7532" "\4E59" "\4E19" "\4E01" "\620A" "\5DF1" "\5E9A" "\8F9B" "\58EC" "\7678";
  /* 甲 乙 丙 丁 戊 己 庚 辛 壬 癸 */
  suffix: "、";
}
```
## <a id="complex-predefined-counters"></a>7.  Complex Predefined Counter Styles

<a id="ref-for-at-ruledef-counter-style③⑧"></a>

While authors may define their own counter styles using the [@counter-style](#at-ruledef-counter-style) rule or rely on the set of predefined counter styles, a few counter styles are described by rules that are too complex to be captured by the predefined algorithms. These counter styles are described in this section.

<a id="ref-for-at-ruledef-counter-style③⑨"></a>

<a id="ref-for-valdef-counter-style-system-extends⑦"></a>

Some of the counter styles specified in this section have custom algorithms for generating counter values, but are otherwise identical to a counter style defined via the [@counter-style](#at-ruledef-counter-style) rule. For example, an author can reference one of these styles in an [extends](#valdef-counter-style-system-extends) system, reusing the algorithm but swapping out some of the other descriptors.

<a id="ref-for-descdef-counter-style-speak-as⑥"></a>

<a id="ref-for-valdef-counter-style-speak-as-numbers③"></a>

<a id="ref-for-use-a-negative-sign⑤"></a>

All of the counter styles defined in this section have a [spoken form](#descdef-counter-style-speak-as) of [numbers](#valdef-counter-style-speak-as-numbers), and [use a negative sign](#use-a-negative-sign).

### <a id="complex-cjk"></a>7.1.  Longhand East Asian Counter Styles

Chinese, Japanese, and Korean have counter styles which have a “longhand” nature, similar to “thirteen thousand one hundred and twenty-three” in English. Each has both formal and informal variants. The formal styles are typically used in financial and legal documents, as their characters are more difficult to alter into each other.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c1039050"></a> The following table shows examples of these styles, particularly some ways in which they differ.
>
> | Counter Style       | 0   | 1   | 2   | 3   | 10   | 11     | 99     | 100  | 101      | 6001     |
> |---------------------|-----|-----|-----|-----|------|--------|--------|------|----------|----------|
> | <strong><span><a id="ref-for-japanese-informal"></a></span><a href="#japanese-informal">japanese-informal</a> &#xA;       </strong> | 〇  | 一  | 二  | 三  | 十   | 十一   | 九十九 | 百   | 百一     | 六千一   |
> | <strong><span><a id="ref-for-japanese-formal"></a></span><a href="#japanese-formal">japanese-formal</a> &#xA;       </strong> | 零  | 壱  | 弐  | 参  | 壱拾 | 壱拾壱 | 九拾九 | 壱百 | 壱百壱   | 六阡壱   |
> | <strong><span><a id="ref-for-korean-hangul-formal"></a></span><a href="#korean-hangul-formal">korean-hangul-formal</a> &#xA;       </strong> | 영  | 일  | 이  | 삼  | 일십 | 일십일 | 구십구 | 일백 | 일백일   | 육천일   |
> | <strong><span><a id="ref-for-korean-hanja-informal"></a></span><a href="#korean-hanja-informal">korean-hanja-informal</a> &#xA;       </strong> | 零  | 一  | 二  | 三  | 十   | 十一   | 九十九 | 百   | 百一     | 六千一   |
> | <strong><span><a id="ref-for-korean-hanja-formal"></a></span><a href="#korean-hanja-formal">korean-hanja-formal</a> &#xA;       </strong> | 零  | 壹  | 貳  | 參  | 壹拾 | 壹拾壹 | 九拾九 | 壹百 | 壹百壹   | 六仟壹   |
> | <strong><span><a id="ref-for-simp-chinese-informal"></a></span><a href="#simp-chinese-informal">simp-chinese-informal</a> &#xA;       </strong> | 零  | 一  | 二  | 三  | 十   | 十一   | 九十九 | 一百 | 一百零一 | 六千零一 |
> | <strong><span><a id="ref-for-simp-chinese-formal"></a></span><a href="#simp-chinese-formal">simp-chinese-formal</a> &#xA;       </strong> | 零  | 壹  | 贰  | 叁  | 壹拾 | 壹拾壹 | 玖拾玖 | 壹佰 | 壹佰零壹 | 陆仟零壹 |
> | <strong><span><a id="ref-for-trad-chinese-informal"></a></span><a href="#trad-chinese-informal">trad-chinese-informal</a> &#xA;       </strong> | 零  | 一  | 二  | 三  | 十   | 十一   | 九十九 | 一百 | 一百零一 | 六千零一 |
> | <strong><span><a id="ref-for-trad-chinese-formal"></a></span><a href="#trad-chinese-formal">trad-chinese-formal</a> &#xA;       </strong> | 零  | 壹  | 貳  | 參  | 壹拾 | 壹拾壹 | 玖拾玖 | 壹佰 | 壹佰零壹 | 陸仟零壹 |

<a id="ref-for-cjk-decimal①"></a>

Because opinions differ on how best to represent numbers 10k or greater using the longhand CJK styles, all of the counter styles defined in this section are defined to have a range of -9999 to 9999, but implementations may support a larger range. Outside the implementation-supported range, the fallback is [cjk-decimal](#cjk-decimal).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Implementations are encouraged to research and implement counter representations beyond 10k and report back to the CSS Working Group with data when a generally-accepted answer is discovered. Some previous research on this topic is contained in an [earlier draft](https://www.w3.org/TR/2013/WD-css-counter-styles-3-20130718/#extended-cjk).

<a id="ref-for-japanese-informal①"></a>

<a id="ref-for-japanese-formal①"></a>

#### <a id="limited-japanese"></a>7.1.1.  Japanese: [japanese-informal](#japanese-informal) and [japanese-formal](#japanese-formal)

<a id="japanese-informal"></a>japanese-informal  
Informal Japanese Kanji numbering (e.g., 千百十一)

<a id="japanese-formal"></a>japanese-formal  
Formal Japanese Kanji numbering (e.g. 壱阡壱百壱拾壱)

```text
@counter-style japanese-informal {
  system: additive;
  range: -9999 9999;
  additive-symbols: 9000 \4E5D\5343, 8000 \516B\5343, 7000 \4E03\5343, 6000 \516D\5343, 5000 \4E94\5343, 4000 \56DB\5343, 3000 \4E09\5343, 2000 \4E8C\5343, 1000 \5343, 900 \4E5D\767E, 800 \516B\767E, 700 \4E03\767E, 600 \516D\767E, 500 \4E94\767E, 400 \56DB\767E, 300 \4E09\767E, 200 \4E8C\767E, 100 \767E, 90 \4E5D\5341, 80 \516B\5341, 70 \4E03\5341, 60 \516D\5341, 50 \4E94\5341, 40 \56DB\5341, 30 \4E09\5341, 20 \4E8C\5341, 10 \5341, 9 \4E5D, 8 \516B, 7 \4E03, 6 \516D, 5 \4E94, 4 \56DB, 3 \4E09, 2 \4E8C, 1 \4E00, 0 \3007;
  /* 9000 九千, 8000 八千, 7000 七千, 6000 六千, 5000 五千, 4000 四千, 3000 三千, 2000 二千, 1000 千, 900 九百, 800 八百, 700 七百, 600 六百, 500 五百, 400 四百, 300 三百, 200 二百, 100 百, 90 九十, 80 八十, 70 七十, 60 六十, 50 五十, 40 四十, 30 三十, 20 二十, 10 十, 9 九, 8 八, 7 七, 6 六, 5 五, 4 四, 3 三, 2 二, 1 一, 0 〇 */
  suffix: '\3001';
  /* 、 */
  negative: "\30DE\30A4\30CA\30B9";
  /* マイナス */
  fallback: cjk-decimal;
}

@counter-style japanese-formal {
  system: additive;
  range: -9999 9999;
  additive-symbols: 9000 \4E5D\9621, 8000 \516B\9621, 7000 \4E03\9621, 6000 \516D\9621, 5000 \4F0D\9621, 4000 \56DB\9621, 3000 \53C2\9621, 2000 \5F10\9621, 1000 \58F1\9621, 900 \4E5D\767E, 800 \516B\767E, 700 \4E03\767E, 600 \516D\767E, 500 \4F0D\767E, 400 \56DB\767E, 300 \53C2\767E, 200 \5F10\767E, 100 \58F1\767E, 90 \4E5D\62FE, 80 \516B\62FE, 70 \4E03\62FE, 60 \516D\62FE, 50 \4F0D\62FE, 40 \56DB\62FE, 30 \53C2\62FE, 20 \5F10\62FE, 10 \58F1\62FE, 9 \4E5D, 8 \516B, 7 \4E03, 6 \516D, 5 \4F0D, 4 \56DB, 3 \53C2, 2 \5F10, 1 \58F1, 0 \96F6;
  /* 9000 九阡, 8000 八阡, 7000 七阡, 6000 六阡, 5000 伍阡, 4000 四阡, 3000 参阡, 2000 弐阡, 1000 壱阡, 900 九百, 800 八百, 700 七百, 600 六百, 500 伍百, 400 四百, 300 参百, 200 弐百, 100 壱百, 90 九拾, 80 八拾, 70 七拾, 60 六拾, 50 伍拾, 40 四拾, 30 参拾, 20 弐拾, 10 壱拾, 9 九, 8 八, 7 七, 6 六, 5 伍, 4 四, 3 参, 2 弐, 1 壱, 0 零 */
  suffix: '\3001';
  /* 、 */
  negative: "\30DE\30A4\30CA\30B9";
  /* マイナス */
  fallback: cjk-decimal;
}
```
<a id="ref-for-korean-hangul-formal①"></a>

<a id="ref-for-korean-hanja-informal①"></a>

<a id="ref-for-korean-hanja-formal①"></a>

#### <a id="limited-korean"></a>7.1.2.  Korean: [korean-hangul-formal](#korean-hangul-formal), [korean-hanja-informal](#korean-hanja-informal), and [korean-hanja-formal](#korean-hanja-formal)

<a id="korean-hangul-formal"></a>korean-hangul-formal  
Korean Hangul numbering (e.g., 일천일백일십일)

<a id="korean-hanja-informal"></a>korean-hanja-informal  
Informal Korean Hanja numbering (e.g., 千百十一)

<a id="korean-hanja-formal"></a>korean-hanja-formal  
Formal Korean Han (Hanja) numbering (e.g., 壹仟壹百壹拾壹)

```text
@counter-style korean-hangul-formal {
  system: additive;
  range: -9999 9999;
  additive-symbols: 9000 \AD6C\CC9C, 8000 \D314\CC9C, 7000 \CE60\CC9C, 6000 \C721\CC9C, 5000 \C624\CC9C, 4000 \C0AC\CC9C, 3000 \C0BC\CC9C, 2000 \C774\CC9C, 1000 \C77C\CC9C, 900 \AD6C\BC31, 800 \D314\BC31, 700 \CE60\BC31, 600 \C721\BC31, 500 \C624\BC31, 400 \C0AC\BC31, 300 \C0BC\BC31, 200 \C774\BC31, 100 \C77C\BC31, 90 \AD6C\C2ED, 80 \D314\C2ED, 70 \CE60\C2ED, 60 \C721\C2ED, 50 \C624\C2ED, 40 \C0AC\C2ED, 30 \C0BC\C2ED, 20 \C774\C2ED, 10 \C77C\C2ED, 9 \AD6C, 8 \D314, 7 \CE60, 6 \C721, 5 \C624, 4 \C0AC, 3 \C0BC, 2 \C774, 1 \C77C, 0 \C601;
  /* 9000 구천, 8000 팔천, 7000 칠천, 6000 육천, 5000 오천, 4000 사천, 3000 삼천, 2000 이천, 1000 일천, 900 구백, 800 팔백, 700 칠백, 600 육백, 500 오백, 400 사백, 300 삼백, 200 이백, 100 일백, 90 구십, 80 팔십, 70 칠십, 60 육십, 50 오십, 40 사십, 30 삼십, 20 이십, 10 일십, 9 구, 8 팔, 7 칠, 6 육, 5 오, 4 사, 3 삼, 2 이, 1 일, 0 영 */
  suffix: ', ';
  negative: "\B9C8\C774\B108\C2A4  ";
  /* 마이너스 (followed by a space) */
}

@counter-style korean-hanja-informal {
  system: additive;
  range: -9999 9999;
  additive-symbols: 9000 \4E5D\5343, 8000 \516B\5343, 7000 \4E03\5343, 6000 \516D\5343, 5000 \4E94\5343, 4000 \56DB\5343, 3000 \4E09\5343, 2000 \4E8C\5343, 1000 \5343, 900 \4E5D\767E, 800 \516B\767E, 700 \4E03\767E, 600 \516D\767E, 500 \4E94\767E, 400 \56DB\767E, 300 \4E09\767E, 200 \4E8C\767E, 100 \767E, 90 \4E5D\5341, 80 \516B\5341, 70 \4E03\5341, 60 \516D\5341, 50 \4E94\5341, 40 \56DB\5341, 30 \4E09\5341, 20 \4E8C\5341, 10 \5341, 9 \4E5D, 8 \516B, 7 \4E03, 6 \516D, 5 \4E94, 4 \56DB, 3 \4E09, 2 \4E8C, 1 \4E00, 0 \96F6;
  /* 9000 九千, 8000 八千, 7000 七千, 6000 六千, 5000 五千, 4000 四千, 3000 三千, 2000 二千, 1000 千, 900 九百, 800 八百, 700 七百, 600 六百, 500 五百, 400 四百, 300 三百, 200 二百, 100 百, 90 九十, 80 八十, 70 七十, 60 六十, 50 五十, 40 四十, 30 三十, 20 二十, 10 十, 9 九, 8 八, 7 七, 6 六, 5 五, 4 四, 3 三, 2 二, 1 一, 0 零 */
  suffix: ', ';
  negative: "\B9C8\C774\B108\C2A4  ";
  /* 마이너스 (followed by a space) */
}

@counter-style korean-hanja-formal {
  system: additive;
  range: -9999 9999;
  additive-symbols: 9000 \4E5D\4EDF, 8000 \516B\4EDF, 7000 \4E03\4EDF, 6000 \516D\4EDF, 5000 \4E94\4EDF, 4000 \56DB\4EDF, 3000 \53C3\4EDF, 2000 \8CB3\4EDF, 1000 \58F9\4EDF, 900 \4E5D\767E, 800 \516B\767E, 700 \4E03\767E, 600 \516D\767E, 500 \4E94\767E, 400 \56DB\767E, 300 \53C3\767E, 200 \8CB3\767E, 100 \58F9\767E, 90 \4E5D\62FE, 80 \516B\62FE, 70 \4E03\62FE, 60 \516D\62FE, 50 \4E94\62FE, 40 \56DB\62FE, 30 \53C3\62FE, 20 \8CB3\62FE, 10 \58F9\62FE, 9 \4E5D, 8 \516B, 7 \4E03, 6 \516D, 5 \4E94, 4 \56DB, 3 \53C3, 2 \8CB3, 1 \58F9, 0 \96F6;
  /* 9000 九仟, 8000 八仟, 7000 七仟, 6000 六仟, 5000 五仟, 4000 四仟, 3000 參仟, 2000 貳仟, 1000 壹仟, 900 九百, 800 八百, 700 七百, 600 六百, 500 五百, 400 四百, 300 參百, 200 貳百, 100 壹百, 90 九拾, 80 八拾, 70 七拾, 60 六拾, 50 五拾, 40 四拾, 30 參拾, 20 貳拾, 10 壹拾, 9 九, 8 八, 7 七, 6 六, 5 五, 4 四, 3 參, 2 貳, 1 壹, 0 零 */
  suffix: ', ';
  negative: "\B9C8\C774\B108\C2A4  ";
  /* 마이너스 (followed by a space) */
}
```
<a id="ref-for-simp-chinese-informal①"></a>

<a id="ref-for-simp-chinese-formal①"></a>

<a id="ref-for-trad-chinese-informal①"></a>

<a id="ref-for-trad-chinese-formal①"></a>

#### <a id="limited-chinese"></a>7.1.3.  Chinese: [simp-chinese-informal](#simp-chinese-informal), [simp-chinese-formal](#simp-chinese-formal), [trad-chinese-informal](#trad-chinese-informal), and [trad-chinese-formal](#trad-chinese-formal)

<a id="simp-chinese-informal"></a>simp-chinese-informal  
Simplified Chinese informal numbering (e.g., 一千一百一十一)

<a id="simp-chinese-formal"></a>simp-chinese-formal  
Simplified Chinese formal numbering (e.g. 壹仟壹佰壹拾壹)

<a id="trad-chinese-informal"></a>trad-chinese-informal  
Traditional Chinese informal numbering (e.g., 一千一百一十一)

<a id="trad-chinese-formal"></a>trad-chinese-formal  
Traditional Chinese formal numbering (e.g., 壹仟壹佰壹拾壹)

<a id="cjk-ideographic"></a>cjk-ideographic  
<a id="ref-for-trad-chinese-informal②"></a>

This counter style is identical to [trad-chinese-informal](#trad-chinese-informal). (It exists for legacy reasons.)

The Chinese longhand styles are defined by almost identical algorithms (specified as a single algorithm here, with the differences called out when relevant), but use different sets of characters, as specified by the table following the algorithm.

1.  If the counter value is 0, the representation is the character for 0 specified for the given counter style. Skip the rest of this algorithm.
2.  Initially represent the counter value as a decimal number. For each digit that is not 0, append the appropriate digit marker to the digit. The ones digit has no marker.
3.  For the informal styles, if the counter value is between ten and nineteen, remove the tens digit (leave the digit marker).
4.  Drop any trailing zeros and collapse any remaining zeros into a single zero digit.
5.  Replace the digits 0-9 with the appropriate character for the given counter style. Return the resultant string as the representation of the counter value.

<a id="ref-for-descdef-counter-style-suffix④"></a>

<a id="ref-for-descdef-counter-style-fallback⑥"></a>

<a id="ref-for-cjk-decimal②"></a>

<a id="ref-for-descdef-counter-style-range⑤"></a>

<a id="ref-for-descdef-counter-style-negative⑥"></a>

For all of these counter styles, the [suffix](#descdef-counter-style-suffix) is "、" U+3001, the [fallback](#descdef-counter-style-fallback) is [cjk-decimal](#cjk-decimal), the [range](#descdef-counter-style-range) is -9999 9999, and the [negative](#descdef-counter-style-negative) value is given in the table of symbols for each style.

The following tables define the characters used in these styles:

**Table 12**

Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.

| Values | Codepoints / simp-chinese-informal | Codepoints / simp-chinese-formal | Codepoints / trad-chinese-informal | Codepoints / trad-chinese-formal |
| --- | --- | --- | --- | --- |
| Digit 0 | 零 U+96F6 | 零 U+96F6 | 零 U+96F6 | 零 U+96F6 |
| Digit 1 | 一 U+4E00 | 壹 U+58F9 | 一 U+4E00 | 壹 U+58F9 |
| Digit 2 | 二 U+4E8C | 贰 U+8D30 | 二 U+4E8C | 貳 U+8CB3 |
| Digit 3 | 三 U+4E09 | 叁 U+53C1 | 三 U+4E09 | 參 U+53C3 |
| Digit 4 | 四 U+56DB | 肆 U+8086 | 四 U+56DB | 肆 U+8086 |
| Digit 5 | 五 U+4E94 | 伍 U+4F0D | 五 U+4E94 | 伍 U+4F0D |
| Digit 6 | 六 U+516D | 陆 U+9646 | 六 U+516D | 陸 U+9678 |
| Digit 7 | 七 U+4E03 | 柒 U+67D2 | 七 U+4E03 | 柒 U+67D2 |
| Digit 8 | 八 U+516B | 捌 U+634C | 八 U+516B | 捌 U+634C |
| Digit 9 | 九 U+4E5D | 玖 U+7396 | 九 U+4E5D | 玖 U+7396 |
| Tens Digit Marker | 十 U+5341 | 拾 U+62FE | 十 U+5341 | 拾 U+62FE |
| Hundreds Digit Marker | 百 U+767E | 佰 U+4F70 | 百 U+767E | 佰 U+4F70 |
| Thousands Digit Marker | 千 U+5343 | 仟 U+4EDF | 千 U+5343 | 仟 U+4EDF |
| Negative Sign | 负 U+8D1F | 负 U+8D1F | 負 U+8CA0 | 負 U+8CA0 |

<a id="ref-for-simp-chinese-informal②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> For reference, here are the first 120 values for the [simp-chinese-informal](#simp-chinese-informal) style:
>
> ```text
>  1　　　　 一    41　　 四十一    81　　 八十一
>  2　　　　 二    42　　 四十二    82　　 八十二
>  3　　　　 三    43　　 四十三    83　　 八十三
>  4　　　　 四    44　　 四十四    84　　 八十四
>  5　　　　 五    45　　 四十五    85　　 八十五
>  6　　　　 六    46　　 四十六    86　　 八十六
>  7　　　　 七    47　　 四十七    87　　 八十七
>  8　　　　 八    48　　 四十八    88　　 八十八
>  9　　　　 九    49　　 四十九    89　　 八十九
> 10　　　　 十    50　　　 五十    90　　　 九十
> 11　　　 十一    51　　 五十一    91　　 九十一
> 12　　　 十二    52　　 五十二    92　　 九十二
> 13　　　 十三    53　　 五十三    93　　 九十三
> 14　　　 十四    54　　 五十四    94　　 九十四
> 15　　　 十五    55　　 五十五    95　　 九十五
> 16　　　 十六    56　　 五十六    96　　 九十六
> 17　　　 十七    57　　 五十七    97　　 九十七
> 18　　　 十八    58　　 五十八    98　　 九十八
> 19　　　 十九    59　　 五十九    99　　 九十九
> 20　　　 二十    60　　　 六十   100　　　 一百
> 21　　 二十一    61　　 六十一   101　 一百零一
> 22　　 二十二    62　　 六十二   102　 一百零二
> 23　　 二十三    63　　 六十三   103　 一百零三
> 24　　 二十四    64　　 六十四   104　 一百零四
> 25　　 二十五    65　　 六十五   105　 一百零五
> 26　　 二十六    66　　 六十六   106　 一百零六
> 27　　 二十七    67　　 六十七   107　 一百零七
> 28　　 二十八    68　　 六十八   108　 一百零八
> 29　　 二十九    69　　 六十九   109　 一百零九
> 30　　　 三十    70　　　 七十   110　 一百一十
> 31　　 三十一    71　　 七十一   111 一百一十一
> 32　　 三十二    72　　 七十二   112 一百一十二
> 33　　 三十三    73　　 七十三   113 一百一十三
> 34　　 三十四    74　　 七十四   114 一百一十四
> 35　　 三十五    75　　 七十五   115 一百一十五
> 36　　 三十六    76　　 七十六   116 一百一十六
> 37　　 三十七    77　　 七十七   117 一百一十七
> 38　　 三十八    78　　 七十八   118 一百一十八
> 39　　 三十九    79　　 七十九   119 一百一十九
> 40　　　 四十    80　　　 八十   120　 一百二十
> ```
<a id="ref-for-valdef-counter-style-name-ethiopic-numeric"></a>

### <a id="ethiopic-numeric-counter-style"></a>7.2.  Ethiopic Numeric Counter Style: [ethiopic-numeric](#valdef-counter-style-name-ethiopic-numeric)

The <a id="valdef-counter-style-name-ethiopic-numeric"></a>ethiopic-numeric counter style is defined for all positive non-zero numbers. The following algorithm converts decimal digits to ethiopic numbers:

1.  If the number is 1, return "፩" (U+1369).
2.  Split the number into groups of two digits, starting with the least significant decimal digit.
3.  Index each group sequentially, starting from the least significant as group number zero.
4.  If the group has the value zero, or if the group is the most significant one and has the value 1, or if the group has an odd index (as given in the previous step) and has the value 1, then remove the digits (but leave the group, so it still has a separator appended below).
5.  For each remaining digit, substitute the relevant ethiopic character from the list below.
    **Table 13**

Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.

| Tens / Values | Tens / Codepoints | Tens / Codepoints |
| --- | --- | --- |
| 10 | ፲ | U+1372 |
| 20 | ፳ | U+1373 |
| 30 | ፴ | U+1374 |
| 40 | ፵ | U+1375 |
| 50 | ፶ | U+1376 |
| 60 | ፷ | U+1377 |
| 70 | ፸ | U+1378 |
| 80 | ፹ | U+1379 |
| 90 | ፺ | U+137A |

| Units / Values | Units / Codepoints | Units / Codepoints |
| --- | --- | --- |
| 1 | ፩ | U+1369 |
| 2 | ፪ | U+136A |
| 3 | ፫ | U+136B |
| 4 | ፬ | U+136C |
| 5 | ፭ | U+136D |
| 6 | ፮ | U+136E |
| 7 | ፯ | U+136F |
| 8 | ፰ | U+1370 |
| 9 | ፱ | U+1371 |
6.  For each group with an odd index (as given in the second step), except groups which originally had a value of zero, append ፻ U+137B.
7.  For each group with an even index (as given in the second step), except the group with index 0, append ፼ U+137C.
8.  Concatenate the groups into one string, and return it.

<a id="ref-for-descdef-counter-style-range⑥"></a>

<a id="ref-for-descdef-counter-style-suffix⑤"></a>

For this system, the name is "ethiopic-numeric", the [range](#descdef-counter-style-range) is 1 infinite, the [suffix](#descdef-counter-style-suffix) is `"/ "` (U+002F SOLIDUS followed by a U+0020 SPACE), and the rest of the descriptors have their initial value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b515a2a9"></a> The decimal number 100, in ethiopic, is ፻ U+137B
>
> The decimal number 78010092, in ethiopic, is ፸፰፻፩፼፺፪ U+1378 U+1370 U+137B U+1369 U+137C U+137A U+136A.
>
> The decimal number 780100000092, in ethiopic, is ፸፰፻፩፼፼፺፪ U+1378 U+1370 U+137B U+1369 U+137C U+137C U+137A U+136A.

## <a id="additional-predefined"></a>8.  Additional “Ready-made” Counter Styles

<a id="ref-for-at-ruledef-counter-style④⓪"></a>

The Internationalization Working Group maintains a large list of ready-made [@counter-style](#at-ruledef-counter-style) rules for various world languages in their [Ready-made Counter Styles](https://www.w3.org/TR/predefined-counter-styles/) document. [\[predefined-counter-styles\]](#biblio-predefined-counter-styles)

These additional counter styles are not intended to be supported by user-agents by default, but can be used by users or authors copying them directly into style sheets.

## <a id="apis"></a>9. APIs

### <a id="extensions-to-cssrule-interface"></a>9.1.  Extensions to the `CSSRule` interface

The `CSSRule` interface is extended as follows:

<a id="ref-for-cssrule"></a>

<a id="ref-for-idl-unsigned-short"></a>

<a id="dom-cssrule-counter_style_rule"></a>

```text
partial interface CSSRule {
    const unsigned short COUNTER_STYLE_RULE = 11;
};
```
### <a id="the-csscounterstylerule-interface"></a>9.2.  The `CSSCounterStyleRule` interface

<a id="ref-for-csscounterstylerule"></a>

<a id="ref-for-at-ruledef-counter-style④①"></a>

The [CSSCounterStyleRule](#csscounterstylerule) interface represents a [@counter-style](#at-ruledef-counter-style) rule.

<a id="ref-for-Exposed"></a>

<a id="csscounterstylerule"></a>

<a id="ref-for-cssrule①"></a>

<a id="ref-for-cssomstring"></a>

<a id="ref-for-dom-csscounterstylerule-name"></a>

<a id="ref-for-cssomstring①"></a>

<a id="ref-for-dom-csscounterstylerule-system"></a>

<a id="ref-for-cssomstring②"></a>

<a id="ref-for-dom-csscounterstylerule-symbols"></a>

<a id="ref-for-cssomstring③"></a>

<a id="ref-for-dom-csscounterstylerule-additivesymbols"></a>

<a id="ref-for-cssomstring④"></a>

<a id="ref-for-dom-csscounterstylerule-negative"></a>

<a id="ref-for-cssomstring⑤"></a>

<a id="ref-for-dom-csscounterstylerule-prefix"></a>

<a id="ref-for-cssomstring⑥"></a>

<a id="ref-for-dom-csscounterstylerule-suffix"></a>

<a id="ref-for-cssomstring⑦"></a>

<a id="ref-for-dom-csscounterstylerule-range"></a>

<a id="ref-for-cssomstring⑧"></a>

<a id="ref-for-dom-csscounterstylerule-pad"></a>

<a id="ref-for-cssomstring⑨"></a>

<a id="ref-for-dom-csscounterstylerule-speakas"></a>

<a id="ref-for-cssomstring①⓪"></a>

<a id="ref-for-dom-csscounterstylerule-fallback"></a>

```text
[Exposed=Window]
interface CSSCounterStyleRule : CSSRule {
  attribute CSSOMString name;
  attribute CSSOMString system;
  attribute CSSOMString symbols;
  attribute CSSOMString additiveSymbols;
  attribute CSSOMString negative;
  attribute CSSOMString prefix;
  attribute CSSOMString suffix;
  attribute CSSOMString range;
  attribute CSSOMString pad;
  attribute CSSOMString speakAs;
  attribute CSSOMString fallback;
};
```
<a id="ref-for-cssomstring①①"></a>

<a id="dom-csscounterstylerule-name"></a>`name`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

<a id="ref-for-typedef-counter-style-name①①"></a>

The <var>name</var> attribute on getting must return a `CSSOMString` object that contains the serialization of the [\<counter-style-name\>](#typedef-counter-style-name) defined for the associated rule.

On setting the <var>name</var> attribute, run the following steps:

1.  <a id="ref-for-ascii-case-insensitive②"></a>

    <a id="ref-for-counter-style⑥"></a>

    If the value is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for any of the predefined [counter styles](#counter-style), lowercase it.

2.  <a id="ref-for-css-identifier"></a>

    If the value is not "decimal", "disc", or "none", replace the associated rule’s name with an [identifier](https://www.w3.org/TR/css-values-3/#css-identifier) equal to the value.

3.  Otherwise, do nothing.

<a id="ref-for-cssomstring①②"></a>

<a id="dom-csscounterstylerule-system"></a>`system`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

<a id="ref-for-cssomstring①③"></a>

<a id="dom-csscounterstylerule-symbols"></a>`symbols`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

<a id="ref-for-cssomstring①④"></a>

<a id="dom-csscounterstylerule-additivesymbols"></a>`additiveSymbols`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

<a id="ref-for-cssomstring①⑤"></a>

<a id="dom-csscounterstylerule-negative"></a>`negative`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

<a id="ref-for-cssomstring①⑥"></a>

<a id="dom-csscounterstylerule-prefix"></a>`prefix`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

<a id="ref-for-cssomstring①⑦"></a>

<a id="dom-csscounterstylerule-suffix"></a>`suffix`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

<a id="ref-for-cssomstring①⑧"></a>

<a id="dom-csscounterstylerule-range"></a>`range`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

<a id="ref-for-cssomstring①⑨"></a>

<a id="dom-csscounterstylerule-pad"></a>`pad`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

<a id="ref-for-cssomstring②⓪"></a>

<a id="dom-csscounterstylerule-speakas"></a>`speakAs`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

<a id="ref-for-cssomstring②①"></a>

<a id="dom-csscounterstylerule-fallback"></a>`fallback`, of type [CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)

The remaining attributes on getting must return a `CSSOMString` object that contains the serialization of the associated descriptor defined for the associated rule. If the descriptor was not specified in the associated rule, the attribute must return an empty string.

On setting, run the following steps:

1.  <a id="ref-for-parse-a-list-of-component-values"></a>

    [Parse a list of component values](https://www.w3.org/TR/css-syntax-3/#parse-a-list-of-component-values) from the value.

2.  <a id="ref-for-at-ruledef-counter-style④②"></a>

    <a id="ref-for-descdef-counter-style-symbols①⑦"></a>

    If the returned value is invalid according to the given descriptor’s grammar, or would cause the [@counter-style](#at-ruledef-counter-style) rule to become invalid, do nothing and abort these steps. (For example, some systems require the [symbols](#descdef-counter-style-symbols) descriptor to contain two values.)

3.  <a id="ref-for-dom-csscounterstylerule-system①"></a>

    <a id="ref-for-first-symbol-value④"></a>

    <a id="ref-for-valdef-counter-style-system-fixed⑤"></a>

    If the attribute being set is [system](#dom-csscounterstylerule-system), and the new value would change the algorithm used, do nothing and abort these steps. <strong data-conversion-semantic="note">Note:</strong> It’s okay to change an aspect of the algorithm, like the [first symbol value](#first-symbol-value) of a [fixed](#valdef-counter-style-system-fixed) system.

4.  Set the descriptor to the value.

## <a id="ua-stylesheet"></a>10.  Sample style sheet for HTML

This section is informative, not normative. HTML itself defines the styles that apply to its elements, and in some cases defers to the user agent’s discretion.

```text
details > summary {
  display: list-item;
  list-style: disclosure-closed inside;
}

details[open] > summary {
  list-style: disclosure-open inside;
}
```
## <a id="changes"></a> Changes

### <a id="changes-2017"></a> Changes since the December 2017 Candidate Recommendation

Significant changes since the [December 14 2017 Candidate Recommendation](https://www.w3.org/TR/2017/CR-css-counter-styles-3-20171214/):

- <a id="ref-for-valdef-list-style-type-none①"></a>

  <a id="ref-for-decimal①②"></a>

  <a id="ref-for-disc③"></a>

  <a id="ref-for-circle③"></a>

  <a id="ref-for-square③"></a>

  <a id="ref-for-disclosure-open⑤"></a>

  Made [none](https://www.w3.org/TR/css-lists-3/#valdef-list-style-type-none), [decimal](#decimal), [disc](#disc), [circle](#circle), [square](#square), [disclosure-open](#disclosure-open), disclosure-close non-overridable. ([Issue 3584](https://github.com/w3c/csswg-drafts/issues/3584))

- Clarified counter-style lookups in Shadow DOM. ([Issue 5693](https://github.com/w3c/csswg-drafts/issues/5693))

- <a id="ref-for-at-ruledef-counter-style④③"></a>

  Clarified what happens in various invalid [@counter-style](#at-ruledef-counter-style) situations. ([Issue 5698](https://github.com/w3c/csswg-drafts/issues/5698), [Issue 5717](https://github.com/w3c/csswg-drafts/issues/5717))

- Fixed divide-by-zero error in additive algorithm. ([Issue 5784](https://github.com/w3c/csswg-drafts/issues/5784))

- <a id="ref-for-square④"></a>

  Fixed [square](#square) symbol to not use an emoji symbol. ([Issue 6200](https://github.com/w3c/csswg-drafts/issues/6200))

- Allowed UAs to override the font choice of predefined symbolic counter styles. ([Issue 6201](https://github.com/w3c/csswg-drafts/issues/6201))

- <a id="ref-for-propdef-list-style-type④"></a>

  Restricted special rendering of predefined symbolic counter styles to usage as a list marker via [list-style-type](https://www.w3.org/TR/css-lists-3/#propdef-list-style-type). ([Issue 6201](https://github.com/w3c/csswg-drafts/issues/6201))

- <a id="ref-for-descdef-counter-style-speak-as⑦"></a>

  Clarified that [speak-as](#descdef-counter-style-speak-as) represents spoken output; it may be used for other AT. ([Issue 6040](https://github.com/w3c/csswg-drafts/issues/6040))

  > <a id="ref-for-descdef-counter-style-speak-as⑧"></a>
  >
  > <a id="ref-for-descdef-counter-style-speak-as⑨"></a>
  >
  > A counter style can be constructed with a meaning that is obvious visually, but impossible to meaningfully represent via a speech synthesizer or other non-visual means, or possible but nonsensical when naively read out <u>loud</u> . The [speak-as](#descdef-counter-style-speak-as) descriptor describes how to synthesize the spoken form of a counter formatted with the given counter style. <u>Assistive technologies should use this spoken form when reading out the counter style, and may use the [speak-as](#descdef-counter-style-speak-as) value to inform transformations to outputs other than speech.</u>

A [Disposition of Comments](https://drafts.csswg.org/css-counter-styles-3/issues-cr-2017) is available.

### <a id="changes-jun-2015"></a> Changes since the June 2015 Candidate Recommendation

Significant changes since the [June 11 2015 Candidate Recommendation](https://www.w3.org/TR/2015/CR-css-counter-styles-3-20150611/):

- <a id="ref-for-valdef-list-style-type-none②"></a>

  <a id="ref-for-disc④"></a>

  Exclude [none](https://www.w3.org/TR/css-lists-3/#valdef-list-style-type-none) and [disc](#disc) from being the name of a counter style.

- <a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>

  When setting CSSCounterStyle.name, take the string directly; don’t [parse](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) it as an ident.

- <a id="ref-for-content-language③"></a>

  Clarify that counter styles are read out in the element’s [content language](https://www.w3.org/TR/css-text-3/#content-language).

- <a id="ref-for-descdef-counter-style-additive-symbols⑧"></a>

  Clarified that [additive-symbols](#descdef-counter-style-additive-symbols) tuples must be of <em>strictly</em> decreasing weight.

- Specified that invalid values just invalidate the declaration, not the whole rule.

- <a id="ref-for-at-ruledef-counter-style④④"></a>

  <a id="ref-for-counter-style⑦"></a>

  [@counter-style](#at-ruledef-counter-style) rules that are invalid due to missing descriptors just fail to create a [counter style](#counter-style); they’re otherwise still valid rules.

- <a id="ref-for-css-bracketed-range-notation"></a>

  Changed syntax to use [CSS bracketed range notation](https://www.w3.org/TR/css-values-4/#css-bracketed-range-notation) to reflect the prose restrictions on negative values.

A [Disposition of Comments](https://drafts.csswg.org/css-counter-styles-3/issues-cr-20150611) is available.

### <a id="changes-feb-2015"></a> Changes since the Feb 2015 Candidate Recommendation

- <a id="ref-for-hebrew②"></a>

  <a id="ref-for-at-ruledef-counter-style④⑤"></a>

  Allowed UAs to extend the [hebrew](#hebrew) style past the spec-defined limits (since the current limits are mostly just an artifact of how annoying it is to go higher with the [@counter-style](#at-ruledef-counter-style)-based definition).

## <a id="acknowledgments"></a> Acknowledgments

The following people and documentation they wrote were very useful for defining the numbering systems: Alexander Savenkov, Arron Eicholz, Aryeh Gregor, Christopher Hoess, Daniel Yacob, Frank Tang, Jonathan Rosenne, Karl Ove Hufthammer, Musheg Arakelyan, Nariné Renard Karapetyan, Randall Bart, Richard Ishida, Simon Montagu (Mozilla, smontagu@smontagu.org)

Special thanks to Xidorn Quan for <em>extensive</em> reviews of all aspects of the spec, and also to Simon Sapin and Håkon Wium Lie for their review comments.

## <a id="priv-sec"></a> Privacy and Security Considerations

This specification introduces no new privacy or security considerations.

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

- [additive](#valdef-counter-style-system-additive), in §3.1.6
- [additive-symbols](#descdef-counter-style-additive-symbols), in §3.8
- [additiveSymbols](#dom-csscounterstylerule-additivesymbols), in §9.2
- [additive tuple](#additive-tuple), in §3.8
- [alphabetic](#valdef-counter-style-system-alphabetic), in §3.1.4
- [arabic-indic](#valdef-counter-style-name-arabic-indic), in §6.1
- [armenian](#armenian), in §6.1
- auto
  - [value for @counter-style/range](#valdef-counter-style-range-auto), in §3.5
  - [value for @counter-style/speak-as](#valdef-counter-style-speak-as-auto), in §3.9
- [bengali](#valdef-counter-style-name-bengali), in §6.1
- [box-corner](#box-corner), in §3.1.2
- [bullets](#valdef-counter-style-speak-as-bullets), in §3.9
- [cambodian](#valdef-counter-style-name-cambodian), in §6.1
- [circle](#circle), in §6.3
- [circled-lower-latin](#circled-lower-latin), in §3.9
- [cjk-decimal](#cjk-decimal), in §6.1
- [cjk-earthly-branch](#valdef-counter-style-name-cjk-earthly-branch), in §6.4
- [cjk-heavenly-stem](#valdef-counter-style-name-cjk-heavenly-stem), in §6.4
- [cjk-ideographic](#cjk-ideographic), in §7.1.3
- [\<counter-style\>](#typedef-counter-style), in §5
- [@counter-style](#at-ruledef-counter-style), in §3
- [counter style](#counter-style), in §2
- \<counter-style-name\>
  - [(type)](#typedef-counter-style-name), in §3
  - [value for @counter-style/speak-as](#valdef-counter-style-speak-as-counter-style-name), in §3.9
- [COUNTER_STYLE_RULE](#dom-cssrule-counter_style_rule), in §9.1
- [counter symbol](#counter-symbol), in §3.8
- [CSSCounterStyleRule](#csscounterstylerule), in §9.2
- [cyclic](#valdef-counter-style-system-cyclic), in §3.1.1
- [decimal](#decimal), in §6.1
- [decimal-leading-zero](#decimal-leading-zero), in §6.1
- [devanagari](#valdef-counter-style-name-devanagari), in §6.1
- [dice](#dice), in §3.1.6
- [disc](#disc), in §6.3
- [disclosure-closed](#disclosure-closed), in §6.3
- [disclosure-open](#disclosure-open), in §6.3
- [ethiopic-numeric](#valdef-counter-style-name-ethiopic-numeric), in §7.2
- [extends](#valdef-counter-style-system-extends), in §3.1.7
- fallback
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-fallback), in §9.2
  - [descriptor for @counter-style](#descdef-counter-style-fallback), in §3.7
- [first symbol value](#first-symbol-value), in §3.1.2
- [fixed](#valdef-counter-style-system-fixed), in §3.1.2
- [footnote](#footnote), in §3.1.3
- [generate a counter](#generate-a-counter), in §2
- [generate a counter representation](#generate-a-counter), in §2
- [georgian](#georgian), in §6.1
- [go](#go), in §3.1.4
- [gujarati](#valdef-counter-style-name-gujarati), in §6.1
- [gurmukhi](#valdef-counter-style-name-gurmukhi), in §6.1
- [hebrew](#hebrew), in §6.1
- [hiragana](#hiragana), in §6.2
- [hiragana-iroha](#hiragana-iroha), in §6.2
- [initial representation for the counter value](#initial-representation-for-the-counter-value), in §2
- [japanese-formal](#japanese-formal), in §7.1.1
- [japanese-informal](#japanese-informal), in §7.1.1
- [kannada](#valdef-counter-style-name-kannada), in §6.1
- [katakana](#katakana), in §6.2
- [katakana-iroha](#katakana-iroha), in §6.2
- [khmer](#valdef-counter-style-name-khmer), in §6.1
- [korean-hangul-formal](#korean-hangul-formal), in §7.1.2
- [korean-hanja-formal](#korean-hanja-formal), in §7.1.2
- [korean-hanja-informal](#korean-hanja-informal), in §7.1.2
- [lao](#valdef-counter-style-name-lao), in §6.1
- [lower-alpha](#lower-alpha), in §6.2
- [lower-armenian](#valdef-counter-style-name-lower-armenian), in §6.1
- [lower-greek](#lower-greek), in §6.2
- [lower-latin](#lower-latin), in §6.2
- [lower-roman](#lower-roman), in §6.1
- [malayalam](#valdef-counter-style-name-malayalam), in §6.1
- [mongolian](#valdef-counter-style-name-mongolian), in §6.1
- [myanmar](#valdef-counter-style-name-myanmar), in §6.1
- [name](#dom-csscounterstylerule-name), in §9.2
- negative
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-negative), in §9.2
  - [descriptor for @counter-style](#descdef-counter-style-negative), in §3.2
- [numbers](#valdef-counter-style-speak-as-numbers), in §3.9
- [numeric](#valdef-counter-style-system-numeric), in §3.1.5
- [oriya](#valdef-counter-style-name-oriya), in §6.1
- pad
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-pad), in §9.2
  - [descriptor for @counter-style](#descdef-counter-style-pad), in §3.6
- [persian](#valdef-counter-style-name-persian), in §6.1
- prefix
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-prefix), in §9.2
  - [descriptor for @counter-style](#descdef-counter-style-prefix), in §3.3
- range
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-range), in §9.2
  - [descriptor for @counter-style](#descdef-counter-style-range), in §3.5
- [simp-chinese-formal](#simp-chinese-formal), in §7.1.3
- [simp-chinese-informal](#simp-chinese-informal), in §7.1.3
- [speak-as](#descdef-counter-style-speak-as), in §3.9
- [speakAs](#dom-csscounterstylerule-speakas), in §9.2
- [spell-out](#valdef-counter-style-speak-as-spell-out), in §3.9
- [square](#square), in §6.3
- suffix
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-suffix), in §9.2
  - [descriptor for @counter-style](#descdef-counter-style-suffix), in §3.4
- [\<symbol\>](#typedef-symbol), in §3.8
- [symbolic](#valdef-system-symbolic), in §3.1.3
- symbols
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-symbols), in §9.2
  - [descriptor for @counter-style](#descdef-counter-style-symbols), in §3.8
- [symbols()](#funcdef-symbols), in §4
- [\<symbols-type\>](#typedef-symbols-type), in §4
- system
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-system), in §9.2
  - [descriptor for @counter-style](#descdef-counter-style-system), in §3.1
- [tamil](#valdef-counter-style-name-tamil), in §6.1
- [telugu](#valdef-counter-style-name-telugu), in §6.1
- [thai](#valdef-counter-style-name-thai), in §6.1
- [tibetan](#valdef-counter-style-name-tibetan), in §6.1
- [trad-chinese-formal](#trad-chinese-formal), in §7.1.3
- [trad-chinese-informal](#trad-chinese-informal), in §7.1.3
- [triangle](#triangle), in §3.1.1
- [trinary](#trinary), in §3.1.5
- [upper-alpha](#upper-alpha), in §6.2
- [upper-alpha-legal](#upper-alpha-legal), in §3.1.3
- [upper-armenian](#valdef-counter-style-name-upper-armenian), in §6.1
- [upper-latin](#upper-latin), in §6.2
- [upper-roman](#upper-roman), in §6.1
- [use a negative sign](#use-a-negative-sign), in §3.2
- [uses a negative sign](#use-a-negative-sign), in §3.2
- [words](#valdef-counter-style-speak-as-words), in §3.9

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-computed-value"></a>computed value
- \[css-content-3\] defines the following terms:
  - <a id="term-for-propdef-content"></a>content
- \[css-images-3\] defines the following terms:
  - <a id="term-for-typedef-image"></a>\<image\>
  - <a id="term-for-default-object-size"></a>default object size
- \[CSS-LISTS-3\] defines the following terms:
  - <a id="term-for-funcdef-counter"></a>counter()
  - <a id="term-for-funcdef-counters"></a>counters()
  - <a id="term-for-valdef-list-style-position-inside"></a>inside
  - <a id="term-for-propdef-list-style"></a>list-style
  - <a id="term-for-propdef-list-style-type"></a>list-style-type
  - <a id="term-for-valdef-list-style-type-none"></a>none
- \[css-pseudo-4\] defines the following terms:
  - <a id="term-for-selectordef-marker"></a>::marker
- \[css-scoping-1\] defines the following terms:
  - <a id="term-for-css-tree-scoped-name"></a>tree-scoped name
  - <a id="term-for-css-tree-scoped-reference"></a>tree-scoped reference
- \[css-syntax-3\] defines the following terms:
  - <a id="term-for-typedef-declaration-list"></a>\<declaration-list\>
  - <a id="term-for-at-rule"></a>at-rule
  - <a id="term-for-css-parse-something-according-to-a-css-grammar"></a>parse
  - <a id="term-for-parse-a-list-of-component-values"></a>parse a list of component values
- \[css-text-3\] defines the following terms:
  - <a id="term-for-content-language"></a>content language
  - <a id="term-for-grapheme-cluster"></a>grapheme cluster
- \[css-text-decor-4\] defines the following terms:
  - <a id="term-for-valdef-text-emphasis-skip-symbols"></a>symbols
- \[css-values-3\] defines the following terms:
  - <a id="term-for-css-identifier"></a>identifier
- \[css-values-4\] defines the following terms:
  - <a id="term-for-mult-comma"></a>\#
  - <a id="term-for-comb-all"></a>&#x26;&#x26;
  - <a id="term-for-mult-one-plus"></a>+
  - <a id="term-for-identifier-value"></a>\<custom-ident\>
  - <a id="term-for-integer-value"></a>\<integer\>
  - <a id="term-for-string-value"></a>\<string\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-css-bracketed-range-notation"></a>css bracketed range notation
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-mult-num"></a>{a}
  - <a id="term-for-comb-one"></a>\|
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-writing-mode"></a>writing mode
- \[cssom-1\] defines the following terms:
  - <a id="term-for-cssomstring"></a>CSSOMString
  - <a id="term-for-cssrule"></a>CSSRule
- \[HTML\] defines the following terms:
  - <a id="term-for-the-details-element"></a>details
- \[INFRA\] defines the following terms:
  - <a id="term-for-ascii-case-insensitive"></a>ascii case-insensitive
  - <a id="term-for-iteration-continue"></a>continue
- \[WebIDL\] defines the following terms:
  - <a id="term-for-Exposed"></a>Exposed
  - <a id="term-for-idl-unsigned-short"></a>unsigned short

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 8 June 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 17 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-lists-3"></a>\[CSS-LISTS-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://www.w3.org/TR/css-lists-3/). 17 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-lists-3&#x2F;](https://www.w3.org/TR/css-lists-3/)

<a id="biblio-css-scoping-1"></a>\[CSS-SCOPING-1\]  
Tab Atkins Jr.; Elika Etemad. [CSS Scoping Module Level 1](https://www.w3.org/TR/css-scoping-1/). 3 April 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scoping-1&#x2F;](https://www.w3.org/TR/css-scoping-1/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 16 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 22 April 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 6 May 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 15 July 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Simon Pieters; Glenn Adams. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 17 March 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-webidl"></a>\[WebIDL\]  
Boris Zbarsky. [Web IDL](https://heycam.github.io/webidl/). 15 December 2016. ED. URL: [https&#x3A;&#x2F;&#x2F;heycam&#x2E;github&#x2E;io&#x2F;webidl&#x2F;](https://heycam.github.io/webidl/)

### <a id="informative"></a>Informative References

<a id="biblio-css-content-3"></a>\[CSS-CONTENT-3\]  
Elika Etemad; Dave Cramer. [CSS Generated Content Module Level 3](https://www.w3.org/TR/css-content-3/). 2 August 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-content-3&#x2F;](https://www.w3.org/TR/css-content-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 31 December 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-predefined-counter-styles"></a>\[PREDEFINED-COUNTER-STYLES\]  
Richard Ishida. [Ready-made Counter Styles](https://www.w3.org/TR/predefined-counter-styles/). 9 June 2021. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;predefined-counter-styles&#x2F;](https://www.w3.org/TR/predefined-counter-styles/)

## <a id="property-index"></a>Property Index

No properties defined.

<a id="ref-for-at-ruledef-counter-style④⑥"></a>

### <a id="counter-style-descriptor-table"></a>[@counter-style](#at-ruledef-counter-style) Descriptors

| Name                | Value                                                                                                                     | Initial                                                                  |
|---------------------|---------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------|
| <strong><span><a id="ref-for-descdef-counter-style-additive-symbols⑨"></a></span><a href="#descdef-counter-style-additive-symbols">additive-symbols</a>&#xA;      </strong> | \[ \<integer \[0,∞\]\> &#x26;&#x26; \<symbol\> \]#                                                      | n/a                                                                      |
| <strong><span><a id="ref-for-descdef-counter-style-fallback⑦"></a></span><a href="#descdef-counter-style-fallback">fallback</a>&#xA;      </strong> | \<counter-style-name\>                                                                                                    | decimal                                                                  |
| <strong><span><a id="ref-for-descdef-counter-style-negative⑦"></a></span><a href="#descdef-counter-style-negative">negative</a>&#xA;      </strong> | \<symbol\> \<symbol\>?                                                                                                    | "&#x5C;2D" ("-" hyphen-minus)                                   |
| <strong><span><a id="ref-for-descdef-counter-style-pad①⓪"></a></span><a href="#descdef-counter-style-pad">pad</a>&#xA;      </strong> | \<integer \[0,∞\]\> &#x26;&#x26; \<symbol\>                                                             | 0 ""                                                                     |
| <strong><span><a id="ref-for-descdef-counter-style-prefix④"></a></span><a href="#descdef-counter-style-prefix">prefix</a>&#xA;      </strong> | \<symbol\>                                                                                                                | "" (the empty string)                                                    |
| <strong><span><a id="ref-for-descdef-counter-style-range⑦"></a></span><a href="#descdef-counter-style-range">range</a>&#xA;      </strong> | \[ \[ \<integer\> \| infinite \]{2} \]# \| auto                                                                           | auto                                                                     |
| <strong><span><a id="ref-for-descdef-counter-style-speak-as①⓪"></a></span><a href="#descdef-counter-style-speak-as">speak-as</a>&#xA;      </strong> | auto \| bullets \| numbers \| words \| spell-out \| \<counter-style-name\>                                                | auto                                                                     |
| <strong><span><a id="ref-for-descdef-counter-style-suffix⑥"></a></span><a href="#descdef-counter-style-suffix">suffix</a>&#xA;      </strong> | \<symbol\>                                                                                                                | "&#x5C;2E&#x5C;20" ("." full stop followed by a space) |
| <strong><span><a id="ref-for-descdef-counter-style-symbols①⑧"></a></span><a href="#descdef-counter-style-symbols">symbols</a>&#xA;      </strong> | \<symbol\>+                                                                                                               | n/a                                                                      |
| <strong><span><a id="ref-for-descdef-counter-style-system①①"></a></span><a href="#descdef-counter-style-system">system</a>&#xA;      </strong> | cyclic \| numeric \| alphabetic \| symbolic \| additive \| \[fixed \<integer\>?\] \| \[ extends \<counter-style-name\> \] | symbolic                                                                 |

## <a id="idl-index"></a>IDL Index

```text
partial interface CSSRule {
    const unsigned short COUNTER_STYLE_RULE = 11;
};

[Exposed=Window]
interface CSSCounterStyleRule : CSSRule {
  attribute CSSOMString name;
  attribute CSSOMString system;
  attribute CSSOMString symbols;
  attribute CSSOMString additiveSymbols;
  attribute CSSOMString negative;
  attribute CSSOMString prefix;
  attribute CSSOMString suffix;
  attribute CSSOMString range;
  attribute CSSOMString pad;
  attribute CSSOMString speakAs;
  attribute CSSOMString fallback;
};

```