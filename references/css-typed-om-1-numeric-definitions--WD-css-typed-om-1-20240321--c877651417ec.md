Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Typed OM Level 1](https://www.w3.org/TR/2024/WD-css-typed-om-1-20240321/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Typed OM Level 1

Source snapshot: https://www.w3.org/TR/2024/WD-css-typed-om-1-20240321/

Snapshot SHA-256: c877651417ec547297d6c8b583f2b9b2d69179c3551894bed368111a44100b76

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Typed OM Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

Converting CSSOM value strings into meaningfully typed JavaScript representations and back can incur a significant performance overhead. This specification exposes CSS values as typed JavaScript objects, to make manipulating them both easier and more performant.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/css-houdini-drafts/issues) (preferred), including the spec code “css-typed-om” in the title, like this: “\[css-typed-om\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-houdini-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-typed-om%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/2023/Process-20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1. Introduction

CSS stylesheets are parsed into abstract UA-internal data structures, the <a id="css-internal-representation"></a>internal representations of CSS, which various specification algorithms manipulate.

<a id="ref-for-css-internal-representation"></a>

<a id="ref-for-css-internal-representation①"></a>

[Internal representations](#css-internal-representation) can’t be directly manipulated, as they are implementation-dependent; UAs have to agree on how to <em>interpret</em> the [internal representations](#css-internal-representation), but the representations themselves are purposely left undefined so that UAs can store and manipulate CSS in whatever way is most efficient for them.

<a id="ref-for-css-internal-representation②"></a>

<a id="ref-for-css-internal-representation③"></a>

<a id="ref-for-css-internal-representation④"></a>

Previously, the only way to read or write to the [internal representations](#css-internal-representation) was via strings—​stylesheets or the CSSOM allowed authors to send strings to the UA, which were parsed into [internal representations](#css-internal-representation), and the CSSOM allowed authors to request that the UA serialize their [internal representations](#css-internal-representation) back into strings.

<a id="ref-for-css-internal-representation⑤"></a>

<a id="ref-for-css-internal-representation⑥"></a>

This specification introduces a new way to interact with [internal representations](#css-internal-representation), by representing them with specialized JS objects that can be manipulated and understood more easily and more reliably than string parsing/concatenation. This new approach is both easier for authors (for example, numeric values are reflected with actual JS numbers, and have unit-aware mathematical operations defined for them) and in many cases are more performant, as values can be directly manipulated and then cheaply translated back into [internal representations](#css-internal-representation) without having to build and then parse strings of CSS.

<a id="ref-for-cssstylevalue"></a>

## <a id="stylevalue-objects"></a>2. <code><a href="#cssstylevalue">CSSStyleValue</a></code> objects

<a id="ref-for-Exposed"></a>

<a id="cssstylevalue"></a>

<a id="CSSStyleValue-stringification-behavior"></a>

<a id="ref-for-Exposed①"></a>

<a id="ref-for-cssstylevalue①"></a>

<a id="ref-for-dom-cssstylevalue-parse"></a>

<a id="ref-for-idl-USVString"></a>

<a id="dom-cssstylevalue-parse-property-csstext-property"></a>

<a id="ref-for-idl-USVString①"></a>

<a id="dom-cssstylevalue-parse-property-csstext-csstext"></a>

<a id="ref-for-Exposed②"></a>

<a id="ref-for-idl-sequence"></a>

<a id="ref-for-cssstylevalue②"></a>

<a id="ref-for-dom-cssstylevalue-parseall"></a>

<a id="ref-for-idl-USVString②"></a>

<a id="dom-cssstylevalue-parseall-property-csstext-property"></a>

<a id="ref-for-idl-USVString③"></a>

<a id="dom-cssstylevalue-parseall-property-csstext-csstext"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSStyleValue {
    stringifier;
    [Exposed=Window] static CSSStyleValue parse(USVString property, USVString cssText);
    [Exposed=Window] static sequence<CSSStyleValue> parseAll(USVString property, USVString cssText);
};
```
<a id="ref-for-cssstylevalue③"></a>

<code><a href="#cssstylevalue">CSSStyleValue</a></code> objects are the base class of all CSS values accessible via the Typed OM API.

<a id="ref-for-CSSStyleValue-stringification-behavior"></a>

<a id="ref-for-cssstylevalue④"></a>

The [stringification behavior](#CSSStyleValue-stringification-behavior) of <code><a href="#cssstylevalue">CSSStyleValue</a></code> objects is defined in [§ 6 CSSStyleValue Serialization](#stylevalue-serialization).

<a id="ref-for-parse-a-cssstylevalue"></a>

The <a id="dom-cssstylevalue-parse"></a><code>parse(<var>property</var>, <var>cssText</var>)</code> method, when invoked, must [parse a CSSStyleValue](#parse-a-cssstylevalue) with property <var>property</var>, cssText <var>cssText</var>, and parseMultiple set to false, and return the result.

<a id="ref-for-parse-a-cssstylevalue①"></a>

The <a id="dom-cssstylevalue-parseall"></a><code>parseAll(<var>property</var>, <var>cssText</var>)</code> method, when invoked, must [parse a CSSStyleValue](#parse-a-cssstylevalue) with property <var>property</var>, cssText <var>cssText</var>, and parseMultiple set to true, and return the result.

<a id="ref-for-string"></a>

<a id="ref-for-string①"></a>

To <a id="parse-a-cssstylevalue"></a>parse a CSSStyleValue given a [string](https://infra.spec.whatwg.org/#string) <var>property</var>, a [string](https://infra.spec.whatwg.org/#string) <var>cssText</var>, and a <var>parseMultiple</var> flag, run these steps:

1.  <a id="ref-for-custom-property-name-string"></a>

    <a id="ref-for-ascii-lowercase"></a>

    If <var>property</var> is not a [custom property name string](#custom-property-name-string), set <var>property</var> to <var>property</var> [ASCII lowercased](https://infra.spec.whatwg.org/#ascii-lowercase).

2.  <a id="ref-for-valid-css-property"></a>

    <a id="ref-for-dfn-throw"></a>

    <a id="ref-for-exceptiondef-typeerror"></a>

    If <var>property</var> is not a [valid CSS property](#valid-css-property), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>

    <a id="ref-for-dfn-throw①"></a>

    <a id="ref-for-exceptiondef-typeerror①"></a>

    Attempt to [parse](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>cssText</var> according to <var>property</var>’s grammar. If this fails, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>. Otherwise, let <var>whole value</var> be the parsed result.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > The behavior of custom properties are different when modified via JavaScript than when defined in style sheets.
    > When a custom property is defined with an invalid syntax in a style sheet, then the value is recorded as "unset", to avoid having to reparse every style sheet when a custom property is registered.
    >
    > <a id="ref-for-exceptiondef-typeerror②"></a>
    >
    > Conversely, when a custom property is modified via the JavaScript API, any parse errors are propagated to the progamming environment via a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>. This allows more immediate feedback of errors to developers.

4.  <a id="ref-for-subdivide-into-iterations"></a>

    [Subdivide into iterations](#subdivide-into-iterations) <var>whole value</var>, according to <var>property</var>, and let <var>values</var> be the result.

5.  <a id="ref-for-list-iterate"></a>

    <a id="ref-for-css-reify"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>value</var> in <var>values</var>, replace it with the result of [reifying](#css-reify) <var>value</var> for <var>property</var>.

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-03265e97"></a> Define the global.

6.  If <var>parseMultiple</var> is false, return <var>values</var>\[0\]. Otherwise, return <var>values</var>.

To <a id="subdivide-into-iterations"></a>subdivide into iterations a CSS value <var>whole value</var> for a property <var>property</var>, execute the following steps:

1.  <a id="ref-for-single-valued-properties"></a>

    <a id="ref-for-list"></a>

    If <var>property</var> is a [single-valued property](#single-valued-properties), return a [list](https://infra.spec.whatwg.org/#list) containing <var>whole value</var>.

2.  <a id="ref-for-list①"></a>

    Otherwise, divide <var>whole value</var> into individual iterations, as appropriate for <var>property</var>, and return a [list](https://infra.spec.whatwg.org/#list) containing the iterations in order.

<a id="ref-for-list-valued-properties"></a>

<a id="ref-for-propdef-counter-reset"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c8d43bc8"></a> How to divide a [list-valued property](#list-valued-properties) into iterations is intentionally undefined and hand-wavey at the moment. <em>Generally</em>, you just split it on top-level commas (corresponding to a top-level `<foo>#` term in the grammar), but some legacy properties (such as [counter-reset](https://www.w3.org/TR/css-lists-3/#propdef-counter-reset)) don’t separate their iterations with commas.
>
> It’s expected to be rigorously defined in the future, but at the moment is explicitly a "you know what we mean" thing.

<a id="ref-for-cssstylevalue⑤"></a>

### <a id="direct-cssstylevalue"></a>2.1. Direct <code><a href="#cssstylevalue">CSSStyleValue</a></code> Objects

<a id="ref-for-cssstylevalue⑥"></a>

<a id="ref-for-cssstylevalue⑦"></a>

Values that can’t yet be directly supported by a more specialized <code><a href="#cssstylevalue">CSSStyleValue</a></code> subclass are instead represented as <code><a href="#cssstylevalue">CSSStyleValue</a></code> objects.

<a id="ref-for-cssstylevalue⑧"></a>

<a id="ref-for-dom-cssstylevalue-associatedproperty-slot"></a>

<a id="ref-for-css-internal-representation⑦"></a>

<a id="ref-for-css-internal-representation⑧"></a>

<a id="ref-for-css-reify①"></a>

<a id="ref-for-css-internal-representation⑨"></a>

Each <code><a href="#cssstylevalue">CSSStyleValue</a></code> object is associated with a particular CSS property, via its <code><a href="#dom-cssstylevalue-associatedproperty-slot">&#x5B;&#x5B;associatedProperty&#x5D;&#x5D;</a></code> internal slot, and a particular, immutable, [internal representation](#css-internal-representation). These objects are said to "represent" the particular [internal representation](#css-internal-representation) they were [reified](#css-reify) from, such that if they are set back into a stylesheet for the same property, they reproduce an equivalent [internal representation](#css-internal-representation).

<a id="ref-for-cssstylevalue⑨"></a>

<a id="ref-for-cssstylevalue①⓪"></a>

<a id="ref-for-string②"></a>

These <code><a href="#cssstylevalue">CSSStyleValue</a></code> objects are only considered valid for the property that they were parsed for. This is enforced by <code><a href="#cssstylevalue">CSSStyleValue</a></code> objects having a <a id="dom-cssstylevalue-associatedproperty-slot"></a>`[[associatedProperty]]` internal slot, which is either `null` (the default) or a [string](https://infra.spec.whatwg.org/#string) specifying a property name.

<a id="ref-for-stylepropertymap"></a>

<a id="ref-for-dom-stylepropertymap-set"></a>

<a id="ref-for-dom-stylepropertymap-append"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This slot is checked by <code><a href="#stylepropertymap">StylePropertyMap</a></code>.<code><a href="#dom-stylepropertymap-set">set()</a></code>/<code><a href="#dom-stylepropertymap-append">append()</a></code>

<a id="ref-for-stylepropertymap①"></a>

## <a id="the-stylepropertymap"></a>3. The <code><a href="#stylepropertymap">StylePropertyMap</a></code>

<a id="ref-for-Exposed③"></a>

<a id="stylepropertymapreadonly"></a>

<a id="ref-for-idl-USVString④"></a>

<a id="ref-for-idl-sequence①"></a>

<a id="ref-for-cssstylevalue①①"></a>

<a id="ref-for-idl-undefined"></a>

<a id="ref-for-cssstylevalue①②"></a>

<a id="ref-for-dom-stylepropertymapreadonly-get"></a>

<a id="ref-for-idl-USVString⑤"></a>

<a id="dom-stylepropertymapreadonly-get-property-property"></a>

<a id="ref-for-idl-sequence②"></a>

<a id="ref-for-cssstylevalue①③"></a>

<a id="ref-for-dom-stylepropertymapreadonly-getall"></a>

<a id="ref-for-idl-USVString⑥"></a>

<a id="dom-stylepropertymapreadonly-getall-property-property"></a>

<a id="ref-for-idl-boolean"></a>

<a id="ref-for-dom-stylepropertymapreadonly-has"></a>

<a id="ref-for-idl-USVString⑦"></a>

<a id="dom-stylepropertymapreadonly-has-property-property"></a>

<a id="ref-for-idl-unsigned-long"></a>

<a id="ref-for-dom-stylepropertymapreadonly-size"></a>

<a id="ref-for-Exposed④"></a>

<a id="stylepropertymap"></a>

<a id="ref-for-stylepropertymapreadonly"></a>

<a id="ref-for-idl-undefined①"></a>

<a id="ref-for-dom-stylepropertymap-set①"></a>

<a id="ref-for-idl-USVString⑧"></a>

<a id="dom-stylepropertymap-set-property-values-property"></a>

<a id="ref-for-cssstylevalue①④"></a>

<a id="ref-for-idl-USVString⑨"></a>

<a id="dom-stylepropertymap-set-property-values-values"></a>

<a id="ref-for-idl-undefined②"></a>

<a id="ref-for-dom-stylepropertymap-append①"></a>

<a id="ref-for-idl-USVString①⓪"></a>

<a id="dom-stylepropertymap-append-property-values-property"></a>

<a id="ref-for-cssstylevalue①⑤"></a>

<a id="ref-for-idl-USVString①①"></a>

<a id="dom-stylepropertymap-append-property-values-values"></a>

<a id="ref-for-idl-undefined③"></a>

<a id="ref-for-dom-stylepropertymap-delete"></a>

<a id="ref-for-idl-USVString①②"></a>

<a id="dom-stylepropertymap-delete-property-property"></a>

<a id="ref-for-idl-undefined④"></a>

<a id="ref-for-dom-stylepropertymap-clear"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface StylePropertyMapReadOnly {
    iterable<USVString, sequence<CSSStyleValue>>;
    (undefined or CSSStyleValue) get(USVString property);
    sequence<CSSStyleValue> getAll(USVString property);
    boolean has(USVString property);
    readonly attribute unsigned long size;
};

[Exposed=Window]
interface StylePropertyMap : StylePropertyMapReadOnly {
    undefined set(USVString property, (CSSStyleValue or USVString)... values);
    undefined append(USVString property, (CSSStyleValue or USVString)... values);
    undefined delete(USVString property);
    undefined clear();
};
```
<a id="ref-for-stylepropertymap②"></a>

<a id="ref-for-css-declaration-block"></a>

<a id="ref-for-css-declaration-block①"></a>

<a id="ref-for-cssstyledeclaration"></a>

<code><a href="#stylepropertymap">StylePropertyMap</a></code> is an alternate way to represent a [CSS declaration block](https://www.w3.org/TR/cssom-1/#css-declaration-block) as an object (when fetched via the [\[cssom\]](#biblio-cssom), [CSS declaration blocks](https://www.w3.org/TR/cssom-1/#css-declaration-block) are instead represented as <code><a href="https://www.w3.org/TR/cssom-1/#cssstyledeclaration">CSSStyleDeclaration</a></code> objects.)

<a id="ref-for-stylepropertymapreadonly①"></a>

<a id="ref-for-ordered-map"></a>

<a id="ref-for-css-declaration-block②"></a>

<a id="ref-for-cssstyledeclaration-declarations"></a>

A <code><a href="#stylepropertymapreadonly">StylePropertyMapReadOnly</a></code> object has a <a id="dom-stylepropertymapreadonly-declarations-slot"></a>`[[declarations]]` internal slot, which is a [map](https://infra.spec.whatwg.org/#ordered-map) reflecting the [CSS declaration block](https://www.w3.org/TR/cssom-1/#css-declaration-block)'s [declarations](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-declarations).

<a id="ref-for-cssstyledeclaration-declarations①"></a>

<a id="ref-for-ordered-map①"></a>

<a id="ref-for-string③"></a>

<a id="ref-for-css-internal-representation①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [declarations](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-declarations) are not yet defined using [\[infra\]](#biblio-infra) terminology, but for the purpose of this spec it’s assumed to be a [map](https://infra.spec.whatwg.org/#ordered-map) whose keys are [strings](https://infra.spec.whatwg.org/#string) (representing property names) and whose values are [internal representations](#css-internal-representation) for those properties.

<a id="ref-for-dom-stylepropertymapreadonly-declarations-slot"></a>

Unless otherwise stated, the initial ordering of the <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot is based on the key of each entry:

1.  <a id="ref-for-custom-property"></a>

    <a id="ref-for-ascii-lowercase①"></a>

    Standardized properties (not [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) or vendor-prefixed properties), [ASCII lowercased](https://infra.spec.whatwg.org/#ascii-lowercase) and then sorted in increasing code-point order.

2.  <a id="ref-for-ascii-lowercase②"></a>

    Vendor-prefixed/experimental properties (those whose name starts with a single dash), [ASCII lowercased](https://infra.spec.whatwg.org/#ascii-lowercase) and then sorted in increasing code-point order.

3.  <a id="ref-for-custom-property①"></a>

    [Custom properties](https://www.w3.org/TR/css-variables-1/#custom-property), sorted in increasing code-point order. (These are never lower-cased; they are preserved exactly as written.)

<a id="ref-for-dfn-value-pairs-to-iterate-over"></a>

<a id="ref-for-stylepropertymapreadonly②"></a>

The [value pairs to iterate over](https://webidl.spec.whatwg.org/#dfn-value-pairs-to-iterate-over) for a <code><a href="#stylepropertymapreadonly">StylePropertyMapReadOnly</a></code> object <var>this</var> are obtained as follows:

1.  <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot①"></a>

    Let <var>declarations</var> be <var>this</var>’s <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> slot.

2.  <a id="ref-for-list②"></a>

    Let <var>value pairs</var> be an empty [list](https://infra.spec.whatwg.org/#list).

3.  <a id="ref-for-map-iterate"></a>

    [For each](https://infra.spec.whatwg.org/#map-iterate) <var>prop</var> → <var>value</var> in <var>declarations</var>:

    1.  <a id="ref-for-subdivide-into-iterations①"></a>

        Let <var>iterations</var> be the result of [dividing into iterations](#subdivide-into-iterations) <var>value</var>.

    2.  <a id="ref-for-css-reify②"></a>

        <a id="ref-for-list-item"></a>

        [Reify](#css-reify) each [item](https://infra.spec.whatwg.org/#list-item) of <var>iterations</var>, and let <var>objects</var> be the result.

    3.  Append <var>prop</var>/<var>objects</var> to <var>value pairs</var>.

4.  Return <var>value pairs</var>.

<a id="ref-for-propdef-background-image"></a>

<a id="ref-for-propdef-animation"></a>

<a id="ref-for-propdef-counter-reset①"></a>

<a id="ref-for-propdef-color"></a>

Some CSS properties are <a id="list-valued-properties"></a>list-valued properties, such as [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) or [animation](https://www.w3.org/TR/css-animations-1/#propdef-animation); their value is a list of parallel grammar terms, almost always comma-separated (the only exceptions are certain legacy properties like [counter-reset](https://www.w3.org/TR/css-lists-3/#propdef-counter-reset)), indicating multiple distinct "values" interpreted in the same way. Other properties, such as [color](https://www.w3.org/TR/css-color-4/#propdef-color), are <a id="single-valued-properties"></a>single-valued properties; they take only a single (possibly complex) value.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-75ed902d"></a>[w3c/css-houdini-drafts/644](https://github.com/w3c/css-houdini-drafts/issues/644)[\[css-typed-om\]Define precisely which properties are list-valued and which aren't, probably in an appendix.](https://github.com/w3c/css-houdini-drafts/issues/644)

<a id="ref-for-single-valued-properties①"></a>

<a id="ref-for-list-valued-properties①"></a>

<a id="ref-for-single-valued-properties②"></a>

<a id="ref-for-list-valued-properties②"></a>

<a id="ref-for-stylepropertymap③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> There are multiple examples of CSS properties that have transitioned from being [single-valued](#single-valued-properties) to [list-valued](#list-valued-properties). To ensure that code written at a time when a property was [single-valued](#single-valued-properties) does not break when it becomes [list-valued](#list-valued-properties) in the future, the <code><a href="#stylepropertymap">StylePropertyMap</a></code> is a <b>multi-map</b>; it stores <em>list</em> of values for each key, but allows you to interact with it as if there was only a single value for each key as well.
>
> <a id="ref-for-stylepropertymap④"></a>
>
> <a id="ref-for-propdef-background-image①"></a>
>
> This means that multiple values for a single property in a <code><a href="#stylepropertymap">StylePropertyMap</a></code> do not represent multiple successive definition of that property’s value; instead, they represent multiple comma-separated sub-values in a single property value, like each "layer" in a [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) property.

<a id="ref-for-stylepropertymap⑤"></a>

The <a id="dom-stylepropertymapreadonly-get"></a><code>get(<var>property</var>)</code> method, when called on a <code><a href="#stylepropertymap">StylePropertyMap</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-custom-property-name-string①"></a>

    <a id="ref-for-ascii-lowercase③"></a>

    If <var>property</var> is not a [custom property name string](#custom-property-name-string), set <var>property</var> to <var>property</var> [ASCII lowercased](https://infra.spec.whatwg.org/#ascii-lowercase).

2.  <a id="ref-for-valid-css-property①"></a>

    <a id="ref-for-dfn-throw②"></a>

    <a id="ref-for-exceptiondef-typeerror③"></a>

    If <var>property</var> is not a [valid CSS property](#valid-css-property), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot②"></a>

    Let <var>props</var> be the value of <var>this</var>’s <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot.

4.  <a id="ref-for-map-exists"></a>

    <a id="ref-for-subdivide-into-iterations②"></a>

    <a id="ref-for-css-reify③"></a>

    If <var>props</var>\[<var>property</var>\] [exists](https://infra.spec.whatwg.org/#map-exists), [subdivide into iterations](#subdivide-into-iterations) <var>props</var>\[<var>property</var>\], then [reify](#css-reify) the first item of the result and return it.

    Otherwise, return `undefined`.

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-03265e97①"></a> Define the global.

<a id="ref-for-stylepropertymap⑥"></a>

The <a id="dom-stylepropertymapreadonly-getall"></a><code>getAll(<var>property</var>)</code> method, when called on a <code><a href="#stylepropertymap">StylePropertyMap</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-custom-property-name-string②"></a>

    <a id="ref-for-ascii-lowercase④"></a>

    If <var>property</var> is not a [custom property name string](#custom-property-name-string), set <var>property</var> to <var>property</var> [ASCII lowercased](https://infra.spec.whatwg.org/#ascii-lowercase).

2.  <a id="ref-for-valid-css-property②"></a>

    <a id="ref-for-dfn-throw③"></a>

    <a id="ref-for-exceptiondef-typeerror④"></a>

    If <var>property</var> is not a [valid CSS property](#valid-css-property), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot③"></a>

    Let <var>props</var> be the value of <var>this</var>’s <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot.

4.  <a id="ref-for-map-exists①"></a>

    <a id="ref-for-subdivide-into-iterations③"></a>

    <a id="ref-for-css-reify④"></a>

    <a id="ref-for-list-item①"></a>

    If <var>props</var>\[<var>property</var>\] [exists](https://infra.spec.whatwg.org/#map-exists), [subdivide into iterations](#subdivide-into-iterations) <var>props</var>\[<var>property</var>\], then [reify](#css-reify) each [item](https://infra.spec.whatwg.org/#list-item) of the result, and return the list.

    <a id="ref-for-list③"></a>

    Otherwise, return an empty [list](https://infra.spec.whatwg.org/#list).

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-03265e97②"></a> Define the global.

<a id="ref-for-stylepropertymap⑦"></a>

The <a id="dom-stylepropertymapreadonly-has"></a><code>has(<var>property</var>)</code> method, when called on a <code><a href="#stylepropertymap">StylePropertyMap</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-custom-property-name-string③"></a>

    <a id="ref-for-ascii-lowercase⑤"></a>

    If <var>property</var> is not a [custom property name string](#custom-property-name-string), set <var>property</var> to <var>property</var> [ASCII lowercased](https://infra.spec.whatwg.org/#ascii-lowercase).

2.  <a id="ref-for-valid-css-property③"></a>

    <a id="ref-for-dfn-throw④"></a>

    <a id="ref-for-exceptiondef-typeerror⑤"></a>

    If <var>property</var> is not a [valid CSS property](#valid-css-property), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot④"></a>

    Let <var>props</var> be the value of <var>this</var>’s <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot.

4.  <a id="ref-for-map-exists②"></a>

    If <var>props</var>\[<var>property</var>\] [exists](https://infra.spec.whatwg.org/#map-exists), return `true`. Otherwise, return `false`.

<a id="ref-for-stylepropertymap⑧"></a>

The <a id="dom-stylepropertymapreadonly-size"></a>`size` attribute, on getting from a <code><a href="#stylepropertymap">StylePropertyMap</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-map-size"></a>

    <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot⑤"></a>

    Return the [size](https://infra.spec.whatwg.org/#map-size) of the value of <var>this</var>’s <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot.

<a id="ref-for-stylepropertymap⑨"></a>

The <a id="dom-stylepropertymap-set"></a><code>set(<var>property</var>, ...<var>values</var>)</code> method, when called on a <code><a href="#stylepropertymap">StylePropertyMap</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-custom-property-name-string④"></a>

    <a id="ref-for-ascii-lowercase⑥"></a>

    If <var>property</var> is not a [custom property name string](#custom-property-name-string), set <var>property</var> to <var>property</var> [ASCII lowercased](https://infra.spec.whatwg.org/#ascii-lowercase).

2.  <a id="ref-for-valid-css-property④"></a>

    <a id="ref-for-dfn-throw⑤"></a>

    <a id="ref-for-exceptiondef-typeerror⑥"></a>

    If <var>property</var> is not a [valid CSS property](#valid-css-property), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-single-valued-properties③"></a>

    <a id="ref-for-list-item②"></a>

    <a id="ref-for-dfn-throw⑥"></a>

    <a id="ref-for-exceptiondef-typeerror⑦"></a>

    If <var>property</var> is a [single-valued property](#single-valued-properties) and <var>values</var> has more than one [item](https://infra.spec.whatwg.org/#list-item), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

4.  <a id="ref-for-list-item③"></a>

    <a id="ref-for-dom-cssstylevalue-associatedproperty-slot①"></a>

    <a id="ref-for-dfn-throw⑦"></a>

    <a id="ref-for-exceptiondef-typeerror⑧"></a>

    If any of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var> have a non-null <code><a href="#dom-cssstylevalue-associatedproperty-slot">&#x5B;&#x5B;associatedProperty&#x5D;&#x5D;</a></code> internal slot, and that slot’s value is anything other than <var>property</var>, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

5.  <a id="ref-for-list-size"></a>

    <a id="ref-for-list-item④"></a>

    <a id="ref-for-cssunparsedvalue"></a>

    <a id="ref-for-cssvariablereferencevalue"></a>

    <a id="ref-for-dfn-throw⑧"></a>

    <a id="ref-for-exceptiondef-typeerror⑨"></a>

    If the [size](https://infra.spec.whatwg.org/#list-size) of <var>values</var> is two or more, and one or more of the [items](https://infra.spec.whatwg.org/#list-item) are a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> or <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> object, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

    <a id="ref-for-funcdef-var"></a>

    <a id="ref-for-funcdef-var①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Having 2+ values implies that you’re setting multiple items of a list-valued property, but the presence of a [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function in the string-based OM disables all syntax parsing, including splitting into individual iterations (because there might be more commas inside of the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) value, so you can’t tell how many items are actually going to show up). This step’s restriction preserves the same semantics in the Typed OM.

6.  <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot⑥"></a>

    Let <var>props</var> be the value of <var>this</var>’s <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot.

7.  <a id="ref-for-map-exists③"></a>

    <a id="ref-for-map-remove"></a>

    If <var>props</var>\[<var>property</var>\] [exists](https://infra.spec.whatwg.org/#map-exists), [remove](https://infra.spec.whatwg.org/#map-remove) it.

8.  <a id="ref-for-list④"></a>

    Let <var>values to set</var> be an empty [list](https://infra.spec.whatwg.org/#list).

9.  <a id="ref-for-create-an-internal-representation"></a>

    For each <var>value</var> in <var>values</var>, [create an internal representation](#create-an-internal-representation) for <var>property</var> and <var>value</var>, and append the result to <var>values to set</var>.

10. Set <var>props</var>\[<var>property</var>\] to <var>values to set</var>.

<a id="ref-for-ordered-map②"></a>

<a id="ref-for-shorthand-property"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The property is deleted then added back so that it gets put at the end of the [ordered map](https://infra.spec.whatwg.org/#ordered-map), which gives the expected behavior in the face of [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property).

<a id="ref-for-stylepropertymap①⓪"></a>

The <a id="dom-stylepropertymap-append"></a><code>append(<var>property</var>, ...<var>values</var>)</code> method, when called on a <code><a href="#stylepropertymap">StylePropertyMap</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-custom-property-name-string⑤"></a>

    <a id="ref-for-ascii-lowercase⑦"></a>

    If <var>property</var> is not a [custom property name string](#custom-property-name-string), set <var>property</var> to <var>property</var> [ASCII lowercased](https://infra.spec.whatwg.org/#ascii-lowercase).

2.  <a id="ref-for-valid-css-property⑤"></a>

    <a id="ref-for-dfn-throw⑨"></a>

    <a id="ref-for-exceptiondef-typeerror①⓪"></a>

    If <var>property</var> is not a [valid CSS property](#valid-css-property), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-list-valued-properties③"></a>

    <a id="ref-for-dfn-throw①⓪"></a>

    <a id="ref-for-exceptiondef-typeerror①①"></a>

    If <var>property</var> is not a [list-valued property](#list-valued-properties), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

4.  <a id="ref-for-list-item⑤"></a>

    <a id="ref-for-dom-cssstylevalue-associatedproperty-slot②"></a>

    <a id="ref-for-dfn-throw①①"></a>

    <a id="ref-for-exceptiondef-typeerror①②"></a>

    If any of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var> have a non-null <code><a href="#dom-cssstylevalue-associatedproperty-slot">&#x5B;&#x5B;associatedProperty&#x5D;&#x5D;</a></code> internal slot, and that slot’s value is anything other than <var>property</var>, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

5.  <a id="ref-for-list-item⑥"></a>

    <a id="ref-for-cssunparsedvalue①"></a>

    <a id="ref-for-cssvariablereferencevalue①"></a>

    <a id="ref-for-dfn-throw①②"></a>

    <a id="ref-for-exceptiondef-typeerror①③"></a>

    If any of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var> are a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> or <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> object, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

    <a id="ref-for-funcdef-var②"></a>

    <a id="ref-for-funcdef-var③"></a>

    <a id="ref-for-component-value"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: When a property is set via string-based APIs, the presence of [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) in a property prevents the entire thing from being interpreted. In other words, everything <em>besides</em> the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) is a plain [component value](https://www.w3.org/TR/css-syntax-3/#component-value), not a meaningful type. This step’s restriction preserves the same semantics in the Typed OM.

6.  <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot⑦"></a>

    Let <var>props</var> be the value of <var>this</var>’s <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot.

7.  <a id="ref-for-map-exists④"></a>

    <a id="ref-for-map-set"></a>

    <a id="ref-for-list⑤"></a>

    If <var>props</var>\[<var>property</var>\] does not [exist](https://infra.spec.whatwg.org/#map-exists), [set](https://infra.spec.whatwg.org/#map-set) <var>props</var>\[<var>property</var>\] to an empty [list](https://infra.spec.whatwg.org/#list).

8.  <a id="ref-for-funcdef-var④"></a>

    <a id="ref-for-dfn-throw①③"></a>

    <a id="ref-for-exceptiondef-typeerror①④"></a>

    If <var>props</var>\[<var>property</var>\] contains a [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) reference, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

9.  <a id="ref-for-list⑥"></a>

    Let <var>temp values</var> be an empty [list](https://infra.spec.whatwg.org/#list).

10. <a id="ref-for-create-an-internal-representation①"></a>

    <a id="ref-for-list-append"></a>

    For each <var>value</var> in <var>values</var>, [create an internal representation](#create-an-internal-representation) with <var>property</var> and <var>value</var>, and [append](https://infra.spec.whatwg.org/#list-append) the returned value to <var>temp values</var>.

11. <a id="ref-for-list-append①"></a>

    [Append](https://infra.spec.whatwg.org/#list-append) the entries of <var>temp values</var> to <var>props</var>\[<var>property</var>\].

<a id="ref-for-stylepropertymap①①"></a>

The <a id="dom-stylepropertymap-delete"></a><code>delete(<var>property</var>)</code> method, when called on a <code><a href="#stylepropertymap">StylePropertyMap</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-custom-property-name-string⑥"></a>

    <a id="ref-for-ascii-lowercase⑧"></a>

    If <var>property</var> is not a [custom property name string](#custom-property-name-string), set <var>property</var> to <var>property</var> [ASCII lowercased](https://infra.spec.whatwg.org/#ascii-lowercase).

2.  <a id="ref-for-valid-css-property⑥"></a>

    <a id="ref-for-dfn-throw①④"></a>

    <a id="ref-for-exceptiondef-typeerror①⑤"></a>

    If <var>property</var> is not a [valid CSS property](#valid-css-property), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot⑧"></a>

    <a id="ref-for-map-exists⑤"></a>

    <a id="ref-for-map-remove①"></a>

    If <var>this</var>’s <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot [contains](https://infra.spec.whatwg.org/#map-exists) <var>property</var>, [remove](https://infra.spec.whatwg.org/#map-remove) it.

<a id="ref-for-stylepropertymap①②"></a>

The <a id="dom-stylepropertymap-clear"></a>`clear()` method, when called on a <code><a href="#stylepropertymap">StylePropertyMap</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-map-remove②"></a>

    <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot⑨"></a>

    [Remove](https://infra.spec.whatwg.org/#map-remove) all of the declarations in <var>this</var>’s <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot.

<a id="ref-for-string④"></a>

<a id="ref-for-string⑤"></a>

<a id="ref-for-cssstylevalue①⑥"></a>

To <a id="create-an-internal-representation"></a>create an internal representation, given a [string](https://infra.spec.whatwg.org/#string) <var>property</var> and a [string](https://infra.spec.whatwg.org/#string) or <code><a href="#cssstylevalue">CSSStyleValue</a></code> <var>value</var>:

<a id="ref-for-cssstylevalue①⑦"></a>

If <var>value</var> is a direct <code><a href="#cssstylevalue">CSSStyleValue</a></code>,

Return <var>value</var>’s associated value.

<a id="ref-for-cssstylevalue①⑧"></a>

If <var>value</var> is a <code><a href="#cssstylevalue">CSSStyleValue</a></code> subclass,

<a id="ref-for-cssstylevalue-match-a-grammar"></a>

<a id="ref-for-dfn-throw①⑤"></a>

<a id="ref-for-exceptiondef-typeerror①⑥"></a>

If <var>value</var> does not [match the grammar](#cssstylevalue-match-a-grammar) of a list-valued property iteration of <var>property</var>, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

<a id="ref-for-cssunitvalue"></a>

<a id="ref-for-cssmathsum"></a>

<a id="ref-for-dom-cssmathsum-values"></a>

If any component of <var>property</var>’s CSS grammar has a limited numeric range, and the corresponding part of <var>value</var> is a <code><a href="#cssunitvalue">CSSUnitValue</a></code> that is outside of that range, replace that value with the result of wrapping it in a fresh <code><a href="#cssmathsum">CSSMathSum</a></code> whose <code><a href="#dom-cssmathsum-values">values</a></code> internal slot contains only that part of <var>value</var>.

Return the <var>value</var>.

<a id="ref-for-idl-USVString①③"></a>

If <var>value</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-USVString">USVString</a></code>,

<a id="ref-for-parse-a-cssstylevalue②"></a>

[Parse a CSSStyleValue](#parse-a-cssstylevalue) with property <var>property</var>, cssText <var>value</var>, and parseMultiple set to `false`, and return the result.

<a id="ref-for-exceptiondef-typeerror①⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This can throw a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code> instead.

<a id="ref-for-cssstylevalue①⑨"></a>

CSS properties express their valid inputs with grammars, which are written with the assumption of being matched against strings parsed into CSS tokens, as defined in [CSS Syntax 3 § 4 Tokenization](https://www.w3.org/TR/css-syntax-3/#tokenization). <code><a href="#cssstylevalue">CSSStyleValue</a></code> objects can also be matched against these grammars, however.

<a id="ref-for-cssstylevalue②⓪"></a>

A <code><a href="#cssstylevalue">CSSStyleValue</a></code> is said to <a id="cssstylevalue-match-a-grammar"></a>match a grammar based on the following rules:

- <a id="ref-for-csskeywordvalue"></a>

  <a id="ref-for-typedef-ident"></a>

  <a id="ref-for-dom-csskeywordvalue-value"></a>

  A <code><a href="#csskeywordvalue">CSSKeywordValue</a></code> matches an [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) specified in a grammar if its <code><a href="#dom-csskeywordvalue-value">value</a></code> internal slot matches the identifier.

  <a id="ref-for-typedef-ident①"></a>

  <a id="ref-for-valdef-top-auto"></a>

  <a id="ref-for-valdef-top-auto①"></a>

  <a id="ref-for-propdef-width"></a>

  If case-folding rules are in effect normally for that [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) (such as [Auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto) matching the keyword [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto) specified in the grammar for [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)), they apply to this comparison as well.

- <a id="ref-for-csstransformvalue"></a>

  <a id="ref-for-typedef-transform-list"></a>

  A <code><a href="#csstransformvalue">CSSTransformValue</a></code> matches [\<transform-list\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-list).

- <a id="ref-for-cssnumericvalue"></a>

  <a id="ref-for-cssnumericvalue-match"></a>

  A <code><a href="#cssnumericvalue">CSSNumericValue</a></code> matches what its type [matches](#cssnumericvalue-match).

- <a id="ref-for-cssunparsedvalue②"></a>

  A <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> matches any grammar.

- <a id="ref-for-cssstylevalue②①"></a>

  <a id="ref-for-dom-cssstylevalue-associatedproperty-slot③"></a>

  <a id="ref-for-dom-cssstylevalue-associatedproperty-slot④"></a>

  A direct <code><a href="#cssstylevalue">CSSStyleValue</a></code> object (not a subclass) with a non-null <code><a href="#dom-cssstylevalue-associatedproperty-slot">&#x5B;&#x5B;associatedProperty&#x5D;&#x5D;</a></code> slot matches the grammar of the property specified in its <code><a href="#dom-cssstylevalue-associatedproperty-slot">&#x5B;&#x5B;associatedProperty&#x5D;&#x5D;</a></code> slot, regardless of what it is.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As the ability to create more complex values in Typed OM increases, this section will become more complex.

<a id="ref-for-string⑥"></a>

<a id="ref-for-typedef-custom-property-name"></a>

<a id="ref-for-string⑦"></a>

<a id="ref-for-css-css-identifier"></a>

A [string](https://infra.spec.whatwg.org/#string) is a <a id="custom-property-name-string"></a>custom property name string if it starts with two dashes (U+002D HYPHEN-MINUS), like `--foo`. (This corresponds to the [\<custom-property-name\>](https://www.w3.org/TR/css-variables-1/#typedef-custom-property-name) production, but applies to [strings](https://infra.spec.whatwg.org/#string), rather than [identifiers](https://www.w3.org/TR/css-values-4/#css-css-identifier); it can be used without invoking the CSS parser.)

<a id="ref-for-string⑧"></a>

<a id="ref-for-custom-property-name-string⑦"></a>

A [string](https://infra.spec.whatwg.org/#string) is a <a id="valid-css-property"></a>valid CSS property if it is a [custom property name string](#custom-property-name-string), or is a CSS property name recognized by the user agent.

<a id="ref-for-stylepropertymapreadonly③"></a>

### <a id="computed-stylepropertymapreadonly-objects"></a>3.1. Computed <code><a href="#stylepropertymapreadonly">StylePropertyMapReadOnly</a></code> objects

<a id="ref-for-element"></a>

<a id="ref-for-SameObject"></a>

<a id="ref-for-stylepropertymapreadonly④"></a>

<a id="ref-for-dom-element-computedstylemap"></a>

```text
partial interface Element {
    [SameObject] StylePropertyMapReadOnly computedStyleMap();
};
```
<a id="ref-for-computed-value"></a>

<a id="ref-for-element①"></a>

<a id="ref-for-dom-element-computedstylemap①"></a>

<a id="computed-stylepropertymap"></a>Computed StylePropertyMap objects represent the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>, and are accessed by calling the <code><a href="#dom-element-computedstylemap">computedStyleMap()</a></code> method.

<a id="ref-for-element②"></a>

<a id="ref-for-dom-element-computedstylemap②"></a>

Every <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> has a <a id="dom-element-computedstylemapcache-slot"></a>`[[computedStyleMapCache]]` internal slot, initially set to `null`, which caches the result of the <code><a href="#dom-element-computedstylemap">computedStyleMap()</a></code> method when it is first called.

<a id="ref-for-element③"></a>

The <a id="dom-element-computedstylemap"></a>`computedStyleMap()` method must, when called on an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> <var>this</var>, perform the following steps:

1.  <a id="ref-for-dom-element-computedstylemapcache-slot"></a>

    <a id="ref-for-stylepropertymapreadonly⑤"></a>

    <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot①⓪"></a>

    <a id="ref-for-computed-value①"></a>

    <a id="ref-for-custom-property②"></a>

    <a id="ref-for-custom-property③"></a>

    If <var>this</var>’s <code><a href="#dom-element-computedstylemapcache-slot">&#x5B;&#x5B;computedStyleMapCache&#x5D;&#x5D;</a></code> internal slot is set to `null`, set its value to a new <code><a href="#stylepropertymapreadonly">StylePropertyMapReadOnly</a></code> object, whose <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot are the name and [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of every longhand CSS property supported by the User Agent, every registered [custom property](https://www.w3.org/TR/css-variables-1/#custom-property), and every non-registered [custom property](https://www.w3.org/TR/css-variables-1/#custom-property) which is not set to its initial value on <var>this</var>, in the standard order.

    <a id="ref-for-computed-value②"></a>

    <a id="ref-for-dom-stylepropertymapreadonly-declarations-slot①①"></a>

    The [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) in the <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> of this object must remain up-to-date, changing as style resolution changes the properties on <var>this</var> and how they’re computed.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: In practice, since the values are "hidden" behind a `.get()` method call, UAs can delay computing anything until a given property is actually requested.

2.  <a id="ref-for-dom-element-computedstylemapcache-slot①"></a>

    Return <var>this</var>’s <code><a href="#dom-element-computedstylemapcache-slot">&#x5B;&#x5B;computedStyleMapCache&#x5D;&#x5D;</a></code> internal slot.

<a id="ref-for-dom-window-getcomputedstyle"></a>

<a id="ref-for-concept-css-style-sheet-origin-clean-flag"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: like <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">Window.getComputedStyle()</a></code>, this method can expose information from stylesheets with the [origin-clean flag](https://www.w3.org/TR/cssom-1/#concept-css-style-sheet-origin-clean-flag) unset.

<a id="ref-for-stylepropertymapreadonly⑥"></a>

<a id="ref-for-computed-value③"></a>

<a id="ref-for-resolved-value"></a>

<a id="ref-for-dom-window-getcomputedstyle①"></a>

<a id="ref-for-dom-window-getcomputedstyle②"></a>

<a id="ref-for-propdef-width①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The <code><a href="#stylepropertymapreadonly">StylePropertyMapReadOnly</a></code> returned by this method represents the <em>actual</em> [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value), not the [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) concept used by <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">Window.getComputedStyle()</a></code>. It can thus return different values than <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">Window.getComputedStyle()</a></code> for some properties (such as [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)).

<a id="ref-for-dom-window-getcomputedstyle③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Per [WG resolution](https://github.com/w3c/css-houdini-drafts/issues/350#issuecomment-294690156), pseudo-element styles are intended to be obtainable by adding this method to the new <code><a>PseudoElement</a></code> interface (rather than using a `pseudoElt` argument like <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">Window.getComputedStyle()</a></code> does).

<a id="ref-for-stylepropertymap①③"></a>

### <a id="declared-stylepropertymap-objects"></a>3.2. Declared &#x26; Inline <code><a href="#stylepropertymap">StylePropertyMap</a></code> objects

<a id="ref-for-cssstylerule"></a>

<a id="ref-for-SameObject①"></a>

<a id="ref-for-stylepropertymap①④"></a>

<a id="ref-for-dom-cssstylerule-stylemap"></a>

<a id="ref-for-elementcssinlinestyle"></a>

<a id="ref-for-SameObject②"></a>

<a id="ref-for-stylepropertymap①⑤"></a>

<a id="ref-for-dom-elementcssinlinestyle-attributestylemap"></a>

```text
partial interface CSSStyleRule {
    [SameObject] readonly attribute StylePropertyMap styleMap;
};

partial interface mixin ElementCSSInlineStyle {
    [SameObject] readonly attribute StylePropertyMap attributeStyleMap;
};
```
<a id="ref-for-cssstylerule①"></a>

<a id="ref-for-elementcssinlinestyle①"></a>

<a id="ref-for-htmlelement"></a>

<a id="declared-stylepropertymap"></a>Declared StylePropertyMap objects represent style property-value pairs embedded in a style rule or inline style, and are accessed via the <a id="dom-cssstylerule-stylemap"></a>`styleMap` attribute of <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> objects, or the <a id="dom-elementcssinlinestyle-attributestylemap"></a>`attributeStyleMap` attribute of objects implementing the <code><a href="https://www.w3.org/TR/cssom-1/#elementcssinlinestyle">ElementCSSInlineStyle</a></code> interface mixin (such as <code><a href="https://html.spec.whatwg.org/multipage/dom.html#htmlelement">HTMLElement</a></code>s).

<a id="ref-for-dom-stylepropertymapreadonly-declarations-slot①②"></a>

<a id="ref-for-declared-stylepropertymap"></a>

<a id="ref-for-cssstylerule②"></a>

<a id="ref-for-cssstylerule③"></a>

When constructed, the <code><a href="#dom-stylepropertymapreadonly-declarations-slot">&#x5B;&#x5B;declarations&#x5D;&#x5D;</a></code> internal slot for [declared StylePropertyMap](#declared-stylepropertymap) objects is initialized to contain an entry for each property with a valid value inside the <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> or inline style that the object represents, in the same order as the <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> or inline style.

<a id="ref-for-cssstylevalue②②"></a>

## <a id="stylevalue-subclasses"></a>4. <code><a href="#cssstylevalue">CSSStyleValue</a></code> subclasses

<a id="ref-for-cssunparsedvalue③"></a>

### <a id="unparsedvalue-objects"></a>4.1. <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> objects

<a id="ref-for-Exposed⑤"></a>

<a id="cssunparsedvalue"></a>

<a id="ref-for-cssstylevalue②③"></a>

<a id="dom-cssunparsedvalue-cssunparsedvalue"></a>

<a id="ref-for-idl-sequence③"></a>

<a id="ref-for-typedefdef-cssunparsedsegment"></a>

<a id="dom-cssunparsedvalue-cssunparsedvalue-members-members"></a>

<a id="ref-for-typedefdef-cssunparsedsegment①"></a>

<a id="ref-for-idl-unsigned-long①"></a>

<a id="ref-for-dom-cssunparsedvalue-length"></a>

<a id="ref-for-typedefdef-cssunparsedsegment②"></a>

<a id="ref-for-idl-unsigned-long②"></a>

<a id="dom-cssunparsedvalue-__getter__-index-index"></a>

<a id="ref-for-typedefdef-cssunparsedsegment③"></a>

<a id="ref-for-idl-unsigned-long③"></a>

<a id="dom-cssunparsedvalue-__setter__-index-val-index"></a>

<a id="ref-for-typedefdef-cssunparsedsegment④"></a>

<a id="dom-cssunparsedvalue-__setter__-index-val-val"></a>

<a id="ref-for-idl-USVString①④"></a>

<a id="ref-for-cssvariablereferencevalue②"></a>

<a id="typedefdef-cssunparsedsegment"></a>

<a id="ref-for-Exposed⑥"></a>

<a id="cssvariablereferencevalue"></a>

<a id="ref-for-dom-cssvariablereferencevalue-cssvariablereferencevalue"></a>

<a id="ref-for-idl-USVString①⑤"></a>

<a id="dom-cssvariablereferencevalue-cssvariablereferencevalue-variable-fallback-variable"></a>

<a id="ref-for-cssunparsedvalue④"></a>

<a id="dom-cssvariablereferencevalue-cssvariablereferencevalue-variable-fallback-fallback"></a>

<a id="ref-for-idl-USVString①⑥"></a>

<a id="ref-for-dom-cssvariablereferencevalue-variable"></a>

<a id="ref-for-cssunparsedvalue⑤"></a>

<a id="dom-cssvariablereferencevalue-fallback"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSUnparsedValue : CSSStyleValue {
    constructor(sequence<CSSUnparsedSegment> members);
    iterable<CSSUnparsedSegment>;
    readonly attribute unsigned long length;
    getter CSSUnparsedSegment (unsigned long index);
    setter CSSUnparsedSegment (unsigned long index, CSSUnparsedSegment val);
};

typedef (USVString or CSSVariableReferenceValue) CSSUnparsedSegment;

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSVariableReferenceValue {
    constructor(USVString variable, optional CSSUnparsedValue? fallback = null);
    attribute USVString variable;
    readonly attribute CSSUnparsedValue? fallback;
};
```
<a id="ref-for-cssunparsedvalue⑥"></a>

<code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> objects represent property values that reference custom properties. They are comprised of a list of string fragments and variable references.

<a id="ref-for-list⑦"></a>

<a id="ref-for-idl-USVString①⑦"></a>

<a id="ref-for-cssvariablereferencevalue③"></a>

They have a <a id="dom-cssunparsedvalue-tokens-slot"></a>`[[tokens]]` internal slot, which is a [list](https://infra.spec.whatwg.org/#list) of <code><a href="https://webidl.spec.whatwg.org/#idl-USVString">USVString</a></code>s and <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> objects. This list is the object’s values to iterate over.

<a id="ref-for-list-size①"></a>

<a id="ref-for-dom-cssunparsedvalue-tokens-slot"></a>

The <a id="dom-cssunparsedvalue-length"></a>`length` attribute returns the [size](https://infra.spec.whatwg.org/#list-size) of the <code><a href="#dom-cssunparsedvalue-tokens-slot">&#x5B;&#x5B;tokens&#x5D;&#x5D;</a></code> internal slot.

<a id="ref-for-dfn-supported-property-indices"></a>

<a id="ref-for-cssunparsedvalue⑦"></a>

<a id="ref-for-list-size②"></a>

<a id="ref-for-dom-cssunparsedvalue-tokens-slot①"></a>

The [supported property indexes](https://webidl.spec.whatwg.org/#dfn-supported-property-indices) of a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> <var>this</var> are the integers greater than or equal to 0, and less than the [size](https://infra.spec.whatwg.org/#list-size) of <var>this</var>’s <code><a href="#dom-cssunparsedvalue-tokens-slot">&#x5B;&#x5B;tokens&#x5D;&#x5D;</a></code> internal slot.

<a id="ref-for-dfn-determine-the-value-of-an-indexed-property"></a>

<a id="ref-for-cssunparsedvalue⑧"></a>

<a id="ref-for-dom-cssunparsedvalue-tokens-slot②"></a>

To [determine the value of an indexed property](https://webidl.spec.whatwg.org/#dfn-determine-the-value-of-an-indexed-property) of a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> <var>this</var> and an index <var>n</var>, let <var>tokens</var> be <var>this</var>’s <code><a href="#dom-cssunparsedvalue-tokens-slot">&#x5B;&#x5B;tokens&#x5D;&#x5D;</a></code> internal slot, and return <var>tokens</var>\[<var>n</var>\].

<a id="ref-for-dfn-set-the-value-of-an-existing-indexed-property"></a>

<a id="ref-for-cssunparsedvalue⑨"></a>

<a id="ref-for-dom-cssunparsedvalue-tokens-slot③"></a>

To [set the value of an existing indexed property](https://webidl.spec.whatwg.org/#dfn-set-the-value-of-an-existing-indexed-property) of a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> <var>this</var>, an index <var>n</var>, and a value <var>new value</var>, let <var>tokens</var> be <var>this</var>’s <code><a href="#dom-cssunparsedvalue-tokens-slot">&#x5B;&#x5B;tokens&#x5D;&#x5D;</a></code> internal slot, and set <var>tokens</var>\[<var>n</var>\] to <var>new value</var>.

<a id="ref-for-dfn-set-the-value-of-a-new-indexed-property"></a>

<a id="ref-for-cssunparsedvalue①⓪"></a>

<a id="ref-for-dom-cssunparsedvalue-tokens-slot④"></a>

<a id="ref-for-list-size③"></a>

<a id="ref-for-dfn-throw①⑥"></a>

<a id="ref-for-exceptiondef-rangeerror"></a>

<a id="ref-for-list-append②"></a>

To [set the value of a new indexed property](https://webidl.spec.whatwg.org/#dfn-set-the-value-of-a-new-indexed-property) of a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> <var>this</var>, an index <var>n</var>, and a value <var>new value</var>, let <var>tokens</var> be <var>this</var>’s <code><a href="#dom-cssunparsedvalue-tokens-slot">&#x5B;&#x5B;tokens&#x5D;&#x5D;</a></code> internal slot. If <var>n</var> is not equal to the [size](https://infra.spec.whatwg.org/#list-size) of <var>tokens</var>, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-rangeerror">RangeError</a></code>. Otherwise, [append](https://infra.spec.whatwg.org/#list-append) <var>new value</var> to <var>tokens</var>.

<a id="ref-for-cssvariablereferencevalue④"></a>

<a id="ref-for-dom-cssvariablereferencevalue-variable①"></a>

The getter for the <a id="dom-cssvariablereferencevalue-variable"></a>`variable` attribute of a <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> <var>this</var> must return its <code><a href="#dom-cssvariablereferencevalue-variable">variable</a></code> internal slot.

<a id="ref-for-dom-cssvariablereferencevalue-variable②"></a>

<a id="ref-for-cssvariablereferencevalue⑤"></a>

The <code><a href="#dom-cssvariablereferencevalue-variable">variable</a></code> attribute of a <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> <var>this</var> must, on setting a variable <var>variable</var>, perform the following steps:

1.  <a id="ref-for-custom-property-name-string⑧"></a>

    <a id="ref-for-dfn-throw①⑦"></a>

    <a id="ref-for-exceptiondef-typeerror①⑧"></a>

    If <var>variable</var> is not a [custom property name string](#custom-property-name-string), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-dom-cssvariablereferencevalue-variable③"></a>

    Otherwise, set <var>this</var>’s <code><a href="#dom-cssvariablereferencevalue-variable">variable</a></code> internal slot to <var>variable</var>.

The <a id="dom-cssvariablereferencevalue-cssvariablereferencevalue"></a><code>CSSVariableReferenceValue(<var>variable</var>, <var>fallback</var>)</code> constructor must, when called, perform the following steps:

1.  <a id="ref-for-custom-property-name-string⑨"></a>

    <a id="ref-for-dfn-throw①⑧"></a>

    <a id="ref-for-exceptiondef-typeerror①⑨"></a>

    If <var>variable</var> is not a [custom property name string](#custom-property-name-string), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-cssvariablereferencevalue⑥"></a>

    <a id="ref-for-dom-cssvariablereferencevalue-variable④"></a>

    <a id="ref-for-dom-cssvariablereferencevalue-fallback"></a>

    Return a new <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> with its <code><a href="#dom-cssvariablereferencevalue-variable">variable</a></code> internal slot set to <var>variable</var> and its <code><a href="#dom-cssvariablereferencevalue-fallback">fallback</a></code> internal slot set to <var>fallback</var>.

<a id="ref-for-csskeywordvalue①"></a>

### <a id="keywordvalue-objects"></a>4.2. <code><a href="#csskeywordvalue">CSSKeywordValue</a></code> objects

<a id="ref-for-csskeywordvalue②"></a>

<a id="ref-for-css-css-identifier①"></a>

<code><a href="#csskeywordvalue">CSSKeywordValue</a></code> objects represent CSS keywords and other [idents](https://www.w3.org/TR/css-values-4/#css-css-identifier).

<a id="ref-for-Exposed⑦"></a>

<a id="csskeywordvalue"></a>

<a id="ref-for-cssstylevalue②④"></a>

<a id="ref-for-dom-csskeywordvalue-csskeywordvalue"></a>

<a id="ref-for-idl-USVString①⑧"></a>

<a id="dom-csskeywordvalue-csskeywordvalue-value-value"></a>

<a id="ref-for-idl-USVString①⑨"></a>

<a id="ref-for-dom-csskeywordvalue-value①"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSKeywordValue : CSSStyleValue {
    constructor(USVString value);
    attribute USVString value;
};
```
The <a id="dom-csskeywordvalue-csskeywordvalue"></a><code>CSSKeywordValue(<var>value</var>)</code> constructor must, when called, perform the following steps:

1.  <a id="ref-for-dfn-throw①⑨"></a>

    <a id="ref-for-exceptiondef-typeerror②⓪"></a>

    If <var>value</var> is an empty string, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-csskeywordvalue③"></a>

    <a id="ref-for-dom-csskeywordvalue-value②"></a>

    Otherwise, return a new <code><a href="#csskeywordvalue">CSSKeywordValue</a></code> with its <code><a href="#dom-csskeywordvalue-value">value</a></code> internal slot set to <var>value</var>.

<a id="ref-for-csskeywordvalue④"></a>

<a id="ref-for-idl-USVString②⓪"></a>

Any place that accepts a <code><a href="#csskeywordvalue">CSSKeywordValue</a></code> also accepts a raw <code><a href="https://webidl.spec.whatwg.org/#idl-USVString">USVString</a></code>, by using the following typedef and algorithm:

<a id="ref-for-idl-DOMString"></a>

<a id="ref-for-csskeywordvalue⑤"></a>

<a id="typedefdef-csskeywordish"></a>

```text
typedef (DOMString or CSSKeywordValue) CSSKeywordish;
```
To <a id="rectify-a-keywordish-value"></a>rectify a keywordish value <var>val</var>, perform the following steps:

1.  <a id="ref-for-csskeywordvalue⑥"></a>

    If <var>val</var> is a <code><a href="#csskeywordvalue">CSSKeywordValue</a></code>, return <var>val</var>.

2.  <a id="ref-for-idl-DOMString①"></a>

    <a id="ref-for-csskeywordvalue⑦"></a>

    <a id="ref-for-dom-csskeywordvalue-value③"></a>

    If <var>val</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-DOMString">DOMString</a></code>, return a new <code><a href="#csskeywordvalue">CSSKeywordValue</a></code> with its <code><a href="#dom-csskeywordvalue-value">value</a></code> internal slot set to <var>val</var>.

<a id="ref-for-csskeywordvalue⑧"></a>

The <a id="dom-csskeywordvalue-value"></a>`value` attribute of a <code><a href="#csskeywordvalue">CSSKeywordValue</a></code> <var>this</var> must, on setting a value <var>value</var>, perform the following steps:

1.  <a id="ref-for-dfn-throw②⓪"></a>

    <a id="ref-for-exceptiondef-typeerror②①"></a>

    If <var>value</var> is an empty string, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-dom-csskeywordvalue-value④"></a>

    Otherwise, set <var>this</var>’s <code><a href="#dom-csskeywordvalue-value">value</a></code> internal slot, to <var>value</var>.

### <a id="numeric-objects"></a>4.3. Numeric Values:

<a id="ref-for-cssnumericvalue①"></a>

<a id="ref-for-number-value"></a>

<a id="ref-for-percentage-value"></a>

<a id="ref-for-typedef-dimension"></a>

<a id="ref-for-cssnumericvalue②"></a>

<code><a href="#cssnumericvalue">CSSNumericValue</a></code> objects represent CSS values that are numeric in nature ([\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s, [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s, [\<dimension\>](https://www.w3.org/TR/css-values-4/#typedef-dimension)s). There are two interfaces that inherit from <code><a href="#cssnumericvalue">CSSNumericValue</a></code>:

- <a id="ref-for-cssunitvalue①"></a>

  <code><a href="#cssunitvalue">CSSUnitValue</a></code> objects represent values that contain a single unit type (for example "42px").

- <a id="ref-for-cssmathvalue"></a>

  <code><a href="#cssmathvalue">CSSMathValue</a></code> objects represent math expressions, which can contain more than one value/unit (for example "calc(56em + 10%)").

<a id="ref-for-cssnumericvalue③"></a>

<a id="ref-for-cssnumericvalue④"></a>

<a id="ref-for-declared-stylepropertymap①"></a>

<code><a href="#cssnumericvalue">CSSNumericValue</a></code> objects are not range-restricted. Any valid numeric value can be represented by a <code><a href="#cssnumericvalue">CSSNumericValue</a></code>, and that value will not be clamped, rounded, or rejected when set on a [declared StylePropertyMap](#declared-stylepropertymap). Instead, clamping and/or rounding will occur during computation of style.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dfe33355"></a> The following code is valid
>
> ```js
> myElement.attributeStyleMap.set("opacity", CSS.number(3));
> myElement.attributeStyleMap.set("z-index", CSS.number(15.4));
> 
> console.log(myElement.attributeStyleMap.get("opacity").value); // 3
> console.log(myElement.attributeStyleMap.get("z-index").value); // 15.4
> 
> var computedStyle = myElement.computedStyleMap();
> var opacity = computedStyle.get("opacity");
> var zIndex = computedStyle.get("z-index");
> ```
>
> <a id="ref-for-propdef-opacity"></a>
>
> <a id="ref-for-propdef-z-index"></a>
>
> After execution, the value of `opacity` is `1` ([opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) is range-restricted), and the value of `zIndex` is `15` ([z-index](https://www.w3.org/TR/CSS21/visuren.html#propdef-z-index) is rounded to an integer value).

<a id="ref-for-cssunparsedvalue①①"></a>

<a id="ref-for-csskeywordvalue⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: "Numeric values" which incorporate variable references will instead be represented as <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> objects, and keywords as <code><a href="#csskeywordvalue">CSSKeywordValue</a></code> objects.

<a id="ref-for-cssnumericvalue⑤"></a>

<a id="ref-for-idl-double"></a>

Any place that accepts a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> also accepts a raw <code><a href="https://webidl.spec.whatwg.org/#idl-double">double</a></code>, by using the following typedef and algorithm:

<a id="ref-for-idl-double①"></a>

<a id="ref-for-cssnumericvalue⑥"></a>

<a id="typedefdef-cssnumberish"></a>

```text
typedef (double or CSSNumericValue) CSSNumberish;
```
To <a id="rectify-a-numberish-value"></a>rectify a numberish value <var>num</var>, optionally to a given unit <var>unit</var> (defaulting to "number"), perform the following steps:

1.  <a id="ref-for-cssnumericvalue⑦"></a>

    If <var>num</var> is a <code><a href="#cssnumericvalue">CSSNumericValue</a></code>, return <var>num</var>.

2.  <a id="ref-for-idl-double②"></a>

    <a id="ref-for-cssunitvalue②"></a>

    <a id="ref-for-dom-cssunitvalue-value"></a>

    <a id="ref-for-dom-cssunitvalue-unit"></a>

    If <var>num</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-double">double</a></code>, return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> with its <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot set to <var>num</var> and its <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot set to <var>unit</var>.

<a id="ref-for-cssnumericvalue⑧"></a>

#### <a id="numeric-value"></a>4.3.1. Common Numeric Operations, and the <code><a href="#cssnumericvalue">CSSNumericValue</a></code> Superclass

<a id="ref-for-number-value①"></a>

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-typedef-dimension①"></a>

<a id="ref-for-cssnumericvalue⑨"></a>

All numeric CSS values ([\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s, [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s, and [\<dimension\>](https://www.w3.org/TR/css-values-4/#typedef-dimension)s) are represented by subclasses of the <code><a href="#cssnumericvalue">CSSNumericValue</a></code> interface.

<a id="enumdef-cssnumericbasetype"></a>

<a id="dom-cssnumericbasetype-length"></a>

<a id="dom-cssnumericbasetype-angle"></a>

<a id="dom-cssnumericbasetype-time"></a>

<a id="dom-cssnumericbasetype-frequency"></a>

<a id="dom-cssnumericbasetype-resolution"></a>

<a id="dom-cssnumericbasetype-flex"></a>

<a id="dom-cssnumericbasetype-percent"></a>

<a id="dictdef-cssnumerictype"></a>

<a id="ref-for-idl-long"></a>

<a id="dom-cssnumerictype-length"></a>

<a id="ref-for-idl-long①"></a>

<a id="dom-cssnumerictype-angle"></a>

<a id="ref-for-idl-long②"></a>

<a id="dom-cssnumerictype-time"></a>

<a id="ref-for-idl-long③"></a>

<a id="dom-cssnumerictype-frequency"></a>

<a id="ref-for-idl-long④"></a>

<a id="dom-cssnumerictype-resolution"></a>

<a id="ref-for-idl-long⑤"></a>

<a id="dom-cssnumerictype-flex"></a>

<a id="ref-for-idl-long⑥"></a>

<a id="dom-cssnumerictype-percent"></a>

<a id="ref-for-enumdef-cssnumericbasetype"></a>

<a id="dom-cssnumerictype-percenthint"></a>

<a id="ref-for-Exposed⑧"></a>

<a id="cssnumericvalue"></a>

<a id="ref-for-cssstylevalue②⑤"></a>

<a id="ref-for-cssnumericvalue①⓪"></a>

<a id="ref-for-dom-cssnumericvalue-add"></a>

<a id="ref-for-typedefdef-cssnumberish"></a>

<a id="dom-cssnumericvalue-add-values-values"></a>

<a id="ref-for-cssnumericvalue①①"></a>

<a id="ref-for-dom-cssnumericvalue-sub"></a>

<a id="ref-for-typedefdef-cssnumberish①"></a>

<a id="dom-cssnumericvalue-sub-values-values"></a>

<a id="ref-for-cssnumericvalue①②"></a>

<a id="ref-for-dom-cssnumericvalue-mul"></a>

<a id="ref-for-typedefdef-cssnumberish②"></a>

<a id="dom-cssnumericvalue-mul-values-values"></a>

<a id="ref-for-cssnumericvalue①③"></a>

<a id="ref-for-dom-cssnumericvalue-div"></a>

<a id="ref-for-typedefdef-cssnumberish③"></a>

<a id="dom-cssnumericvalue-div-values-values"></a>

<a id="ref-for-cssnumericvalue①④"></a>

<a id="ref-for-dom-cssnumericvalue-min"></a>

<a id="ref-for-typedefdef-cssnumberish④"></a>

<a id="dom-cssnumericvalue-min-values-values"></a>

<a id="ref-for-cssnumericvalue①⑤"></a>

<a id="ref-for-dom-cssnumericvalue-max"></a>

<a id="ref-for-typedefdef-cssnumberish⑤"></a>

<a id="dom-cssnumericvalue-max-values-values"></a>

<a id="ref-for-idl-boolean①"></a>

<a id="ref-for-dom-cssnumericvalue-equals"></a>

<a id="ref-for-typedefdef-cssnumberish⑥"></a>

<a id="dom-cssnumericvalue-equals-value-value"></a>

<a id="ref-for-cssunitvalue③"></a>

<a id="ref-for-dom-cssnumericvalue-to"></a>

<a id="ref-for-idl-USVString②①"></a>

<a id="dom-cssnumericvalue-to-unit-unit"></a>

<a id="ref-for-cssmathsum①"></a>

<a id="ref-for-dom-cssnumericvalue-tosum"></a>

<a id="ref-for-idl-USVString②②"></a>

<a id="dom-cssnumericvalue-tosum-units-units"></a>

<a id="ref-for-dictdef-cssnumerictype"></a>

<a id="ref-for-dom-cssnumericvalue-type"></a>

<a id="ref-for-Exposed⑨"></a>

<a id="ref-for-cssnumericvalue①⑥"></a>

<a id="ref-for-dom-cssnumericvalue-parse"></a>

<a id="ref-for-idl-USVString②③"></a>

<a id="dom-cssnumericvalue-parse-csstext-csstext"></a>

```text
enum CSSNumericBaseType {
    "length",
    "angle",
    "time",
    "frequency",
    "resolution",
    "flex",
    "percent",
};

dictionary CSSNumericType {
    long length;
    long angle;
    long time;
    long frequency;
    long resolution;
    long flex;
    long percent;
    CSSNumericBaseType percentHint;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSNumericValue : CSSStyleValue {
    CSSNumericValue add(CSSNumberish... values);
    CSSNumericValue sub(CSSNumberish... values);
    CSSNumericValue mul(CSSNumberish... values);
    CSSNumericValue div(CSSNumberish... values);
    CSSNumericValue min(CSSNumberish... values);
    CSSNumericValue max(CSSNumberish... values);

    boolean equals(CSSNumberish... value);

    CSSUnitValue to(USVString unit);
    CSSMathSum toSum(USVString... units);
    CSSNumericType type();

    [Exposed=Window] static CSSNumericValue parse(USVString cssText);
};
```
<a id="ref-for-cssnumericvalue①⑦"></a>

The methods on the <code><a href="#cssnumericvalue">CSSNumericValue</a></code> superclass represent operations that all numeric values can perform.

The following are the arithmetic operations you can perform on dimensions:

<a id="ref-for-cssnumericvalue①⑧"></a>

The <a id="dom-cssnumericvalue-add"></a><code>add(...<var>values</var>)</code> method, when called on a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-list-item⑦"></a>

    <a id="ref-for-rectify-a-numberish-value"></a>

    <a id="ref-for-list-item⑧"></a>

    Replace each [item](https://infra.spec.whatwg.org/#list-item) of <var>values</var> with the result of [rectifying a numberish value](#rectify-a-numberish-value) for the [item](https://infra.spec.whatwg.org/#list-item).

2.  <a id="ref-for-cssmathsum②"></a>

    <a id="ref-for-list-prepend"></a>

    <a id="ref-for-list-item⑨"></a>

    <a id="ref-for-dom-cssmathsum-values①"></a>

    If <var>this</var> is a <code><a href="#cssmathsum">CSSMathSum</a></code> object, [prepend](https://infra.spec.whatwg.org/#list-prepend) the [items](https://infra.spec.whatwg.org/#list-item) in <var>this</var>’s <code><a href="#dom-cssmathsum-values">values</a></code> internal slot to <var>values</var>. Otherwise, prepend <var>this</var> to <var>values</var>.

3.  <a id="ref-for-list-item①⓪"></a>

    <a id="ref-for-cssunitvalue④"></a>

    <a id="ref-for-dom-cssunitvalue-unit①"></a>

    <a id="ref-for-cssunitvalue⑤"></a>

    <a id="ref-for-dom-cssunitvalue-unit②"></a>

    <a id="ref-for-dom-cssunitvalue-unit③"></a>

    <a id="ref-for-dom-cssunitvalue-value①"></a>

    <a id="ref-for-dom-cssunitvalue-value②"></a>

    <a id="ref-for-list-item①①"></a>

    If all of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var> are <code><a href="#cssunitvalue">CSSUnitValue</a></code>s and have the same <code><a href="#dom-cssunitvalue-unit">unit</a></code>, return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> whose <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot is set to <var>this</var>’s <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot, and <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to the sum of the <code><a href="#dom-cssunitvalue-value">value</a></code> internal slots of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var>. This addition must be done "left to right" - if <var>values</var> is « 1, 2, 3, 4 », the result must be (((1 + 2) + 3) + 4). (This detail is necessary to ensure interoperability in the presence of floating-point arithmetic.)

4.  <a id="ref-for-cssnumericvalue-add-two-types"></a>

    <a id="ref-for-cssnumericvalue-type"></a>

    <a id="ref-for-list-item①②"></a>

    <a id="ref-for-dfn-throw②①"></a>

    <a id="ref-for-exceptiondef-typeerror②②"></a>

    Let <var>type</var> be the result of [adding](#cssnumericvalue-add-two-types) the [types](#cssnumericvalue-type) of every [item](https://infra.spec.whatwg.org/#list-item) in <var>values</var>. If <var>type</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

5.  <a id="ref-for-cssmathsum③"></a>

    <a id="ref-for-dom-cssmathsum-values②"></a>

    Return a new <code><a href="#cssmathsum">CSSMathSum</a></code> object whose <code><a href="#dom-cssmathsum-values">values</a></code> internal slot is set to <var>values</var>.

<a id="ref-for-cssnumericvalue①⑨"></a>

The <a id="dom-cssnumericvalue-sub"></a><code>sub(...<var>values</var>)</code> method, when called on a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-list-item①③"></a>

    <a id="ref-for-rectify-a-numberish-value①"></a>

    <a id="ref-for-list-item①④"></a>

    <a id="ref-for-cssmath-negate-a-cssnumericvalue"></a>

    Replace each [item](https://infra.spec.whatwg.org/#list-item) of <var>values</var> with the result of [rectifying a numberish value](#rectify-a-numberish-value) for the [item](https://infra.spec.whatwg.org/#list-item), then [negating](#cssmath-negate-a-cssnumericvalue) the value.

2.  <a id="ref-for-dom-cssnumericvalue-add①"></a>

    Return the result of calling the <code><a href="#dom-cssnumericvalue-add">add()</a></code> internal algorithm with <var>this</var> and <var>values</var>.

<a id="ref-for-cssnumericvalue②⓪"></a>

To <a id="cssmath-negate-a-cssnumericvalue"></a>negate a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>:

1.  <a id="ref-for-cssmathnegate"></a>

    <a id="ref-for-dom-cssmathnegate-value"></a>

    If <var>this</var> is a <code><a href="#cssmathnegate">CSSMathNegate</a></code> object, return <var>this</var>’s <code><a href="#dom-cssmathnegate-value">value</a></code> internal slot.

2.  <a id="ref-for-cssunitvalue⑥"></a>

    <a id="ref-for-cssunitvalue⑦"></a>

    <a id="ref-for-dom-cssunitvalue-unit④"></a>

    <a id="ref-for-dom-cssunitvalue-value③"></a>

    If <var>this</var> is a <code><a href="#cssunitvalue">CSSUnitValue</a></code> object, return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> with the same <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot as <var>this</var>, and a <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot set to the negation of <var>this</var>’s.

3.  <a id="ref-for-cssmathnegate①"></a>

    <a id="ref-for-dom-cssmathnegate-value①"></a>

    Otherwise, return a new <code><a href="#cssmathnegate">CSSMathNegate</a></code> object whose <code><a href="#dom-cssmathnegate-value">value</a></code> internal slot is set to <var>this</var>.

<a id="ref-for-cssnumericvalue②①"></a>

The <a id="dom-cssnumericvalue-mul"></a><code>mul(...<var>values</var>)</code> method, when called on a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-list-item①⑤"></a>

    <a id="ref-for-rectify-a-numberish-value②"></a>

    <a id="ref-for-list-item①⑥"></a>

    Replace each [item](https://infra.spec.whatwg.org/#list-item) of <var>values</var> with the result of [rectifying a numberish value](#rectify-a-numberish-value) for the [item](https://infra.spec.whatwg.org/#list-item).

2.  <a id="ref-for-cssmathproduct"></a>

    <a id="ref-for-list-prepend①"></a>

    <a id="ref-for-list-item①⑦"></a>

    <a id="ref-for-dom-cssmathproduct-values"></a>

    If <var>this</var> is a <code><a href="#cssmathproduct">CSSMathProduct</a></code> object, [prepend](https://infra.spec.whatwg.org/#list-prepend) the [items](https://infra.spec.whatwg.org/#list-item) in <var>this</var>’s <code><a href="#dom-cssmathproduct-values">values</a></code> internal slot to <var>values</var>. Otherwise, prepend <var>this</var> to <var>values</var>.

3.  <a id="ref-for-list-item①⑧"></a>

    <a id="ref-for-cssunitvalue⑧"></a>

    <a id="ref-for-dom-cssunitvalue-unit⑤"></a>

    <a id="ref-for-cssunitvalue⑨"></a>

    <a id="ref-for-dom-cssunitvalue-unit⑥"></a>

    <a id="ref-for-dom-cssunitvalue-value④"></a>

    <a id="ref-for-dom-cssunitvalue-value⑤"></a>

    <a id="ref-for-list-item①⑨"></a>

    If all of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var> are <code><a href="#cssunitvalue">CSSUnitValue</a></code>s with <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot set to "number", return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> whose <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot is set to "number", and <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to the product of the <code><a href="#dom-cssunitvalue-value">value</a></code> internal slots of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var>.

    This multiplication must be done "left to right" - if <var>values</var> is « 1, 2, 3, 4 », the result must be (((1 × 2) × 3) × 4). (This detail is necessary to ensure interoperability in the presence of floating-point arithmetic.)

4.  <a id="ref-for-list-item②⓪"></a>

    <a id="ref-for-cssunitvalue①⓪"></a>

    <a id="ref-for-dom-cssunitvalue-unit⑦"></a>

    <a id="ref-for-cssunitvalue①①"></a>

    <a id="ref-for-dom-cssunitvalue-unit⑧"></a>

    <a id="ref-for-dom-cssunitvalue-value⑥"></a>

    <a id="ref-for-dom-cssunitvalue-value⑦"></a>

    <a id="ref-for-list-item②①"></a>

    If all of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var> are <code><a href="#cssunitvalue">CSSUnitValue</a></code>s with <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot set to "number" except one which is set to <var>unit</var>, return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> whose <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot is set to <var>unit</var>, and <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to the product of the <code><a href="#dom-cssunitvalue-value">value</a></code> internal slots of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var>.

    This multiplication must be done "left to right" - if <var>values</var> is « 1, 2, 3, 4 », the result must be (((1 × 2) × 3) × 4).

5.  <a id="ref-for-cssnumericvalue-multiply-two-types"></a>

    <a id="ref-for-cssnumericvalue-type①"></a>

    <a id="ref-for-list-item②②"></a>

    <a id="ref-for-dfn-throw②②"></a>

    <a id="ref-for-exceptiondef-typeerror②③"></a>

    Let <var>type</var> be the result of [multiplying](#cssnumericvalue-multiply-two-types) the [types](#cssnumericvalue-type) of every [item](https://infra.spec.whatwg.org/#list-item) in <var>values</var>. If <var>type</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

6.  <a id="ref-for-cssmathproduct①"></a>

    <a id="ref-for-dom-cssmathproduct-values①"></a>

    Return a new <code><a href="#cssmathproduct">CSSMathProduct</a></code> object whose <code><a href="#dom-cssmathproduct-values">values</a></code> internal slot is set to <var>values</var>.

<a id="ref-for-cssnumericvalue②②"></a>

The <a id="dom-cssnumericvalue-div"></a><code>div(...<var>values</var>)</code> method, when called on a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-list-item②③"></a>

    <a id="ref-for-rectify-a-numberish-value③"></a>

    <a id="ref-for-list-item②④"></a>

    <a id="ref-for-cssmath-invert-a-cssnumericvalue"></a>

    Replace each [item](https://infra.spec.whatwg.org/#list-item) of <var>values</var> with the result of [rectifying a numberish value](#rectify-a-numberish-value) for the [item](https://infra.spec.whatwg.org/#list-item), then [inverting](#cssmath-invert-a-cssnumericvalue) the value.

2.  <a id="ref-for-dom-cssnumericvalue-mul①"></a>

    Return the result of calling the <code><a href="#dom-cssnumericvalue-mul">mul()</a></code> internal algorithm with <var>this</var> and <var>values</var>.

<a id="ref-for-cssnumericvalue②③"></a>

To <a id="cssmath-invert-a-cssnumericvalue"></a>invert a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>:

1.  <a id="ref-for-cssmathinvert"></a>

    <a id="ref-for-dom-cssmathinvert-value"></a>

    If <var>this</var> is a <code><a href="#cssmathinvert">CSSMathInvert</a></code> object, return <var>this</var>’s <code><a href="#dom-cssmathinvert-value">value</a></code> internal slot.

2.  <a id="ref-for-cssunitvalue①②"></a>

    <a id="ref-for-dom-cssunitvalue-unit⑨"></a>

    If <var>this</var> is a <code><a href="#cssunitvalue">CSSUnitValue</a></code> object with <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot set to "number":

    1.  <a id="ref-for-dom-cssunitvalue-value⑧"></a>

        <a id="ref-for-dfn-throw②③"></a>

        <a id="ref-for-exceptiondef-rangeerror①"></a>

        If <var>this</var>’s <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to 0 or -0, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-rangeerror">RangeError</a></code>.

    2.  <a id="ref-for-cssunitvalue①③"></a>

        <a id="ref-for-dom-cssunitvalue-unit①⓪"></a>

        <a id="ref-for-dom-cssunitvalue-value⑨"></a>

        Else return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> with the <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot set to "number", and a <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot set to 1 divided by <var>this</var>’s {CSSUnitValue/value}} internal slot.

3.  <a id="ref-for-cssmathinvert①"></a>

    <a id="ref-for-dom-cssmathinvert-value①"></a>

    Otherwise, return a new <code><a href="#cssmathinvert">CSSMathInvert</a></code> object whose <code><a href="#dom-cssmathinvert-value">value</a></code> internal slot is set to <var>this</var>.

<a id="ref-for-cssnumericvalue②④"></a>

The <a id="dom-cssnumericvalue-min"></a><code>min(...<var>values</var>)</code> method, when called on a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-list-item②⑤"></a>

    <a id="ref-for-rectify-a-numberish-value④"></a>

    <a id="ref-for-list-item②⑥"></a>

    Replace each [item](https://infra.spec.whatwg.org/#list-item) of <var>values</var> with the result of [rectifying a numberish value](#rectify-a-numberish-value) for the [item](https://infra.spec.whatwg.org/#list-item).

2.  <a id="ref-for-cssmathmin"></a>

    <a id="ref-for-list-prepend②"></a>

    <a id="ref-for-list-item②⑦"></a>

    <a id="ref-for-dom-cssmathmin-values"></a>

    If <var>this</var> is a <code><a href="#cssmathmin">CSSMathMin</a></code> object, [prepend](https://infra.spec.whatwg.org/#list-prepend) the [items](https://infra.spec.whatwg.org/#list-item) in <var>this</var>’s <code><a href="#dom-cssmathmin-values">values</a></code> internal slot to <var>values</var>. Otherwise, prepend <var>this</var> to <var>values</var>.

3.  <a id="ref-for-list-item②⑧"></a>

    <a id="ref-for-cssunitvalue①④"></a>

    <a id="ref-for-dom-cssunitvalue-unit①①"></a>

    <a id="ref-for-cssunitvalue①⑤"></a>

    <a id="ref-for-dom-cssunitvalue-unit①②"></a>

    <a id="ref-for-dom-cssunitvalue-unit①③"></a>

    <a id="ref-for-dom-cssunitvalue-value①⓪"></a>

    <a id="ref-for-dom-cssunitvalue-value①①"></a>

    <a id="ref-for-list-item②⑨"></a>

    If all of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var> are <code><a href="#cssunitvalue">CSSUnitValue</a></code>s and have the same <code><a href="#dom-cssunitvalue-unit">unit</a></code>, return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> whose <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot is set to <var>this</var>’s <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot, and <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to the minimum of the <code><a href="#dom-cssunitvalue-value">value</a></code> internal slots of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var>.

4.  <a id="ref-for-cssnumericvalue-add-two-types①"></a>

    <a id="ref-for-cssnumericvalue-type②"></a>

    <a id="ref-for-list-item③⓪"></a>

    <a id="ref-for-dfn-throw②④"></a>

    <a id="ref-for-exceptiondef-typeerror②④"></a>

    Let <var>type</var> be the result of [adding](#cssnumericvalue-add-two-types) the [types](#cssnumericvalue-type) of every [item](https://infra.spec.whatwg.org/#list-item) in <var>values</var>. If <var>type</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

5.  <a id="ref-for-cssmathmin①"></a>

    <a id="ref-for-dom-cssmathmin-values①"></a>

    Return a new <code><a href="#cssmathmin">CSSMathMin</a></code> object whose <code><a href="#dom-cssmathmin-values">values</a></code> internal slot is set to <var>values</var>.

<a id="ref-for-cssnumericvalue②⑤"></a>

The <a id="dom-cssnumericvalue-max"></a><code>max(...<var>values</var>)</code> method, when called on a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-list-item③①"></a>

    <a id="ref-for-rectify-a-numberish-value⑤"></a>

    <a id="ref-for-list-item③②"></a>

    Replace each [item](https://infra.spec.whatwg.org/#list-item) of <var>values</var> with the result of [rectifying a numberish value](#rectify-a-numberish-value) for the [item](https://infra.spec.whatwg.org/#list-item).

2.  <a id="ref-for-cssmathmax"></a>

    <a id="ref-for-list-prepend③"></a>

    <a id="ref-for-list-item③③"></a>

    <a id="ref-for-dom-cssmathmax-values"></a>

    If <var>this</var> is a <code><a href="#cssmathmax">CSSMathMax</a></code> object, [prepend](https://infra.spec.whatwg.org/#list-prepend) the [items](https://infra.spec.whatwg.org/#list-item) in <var>this</var>’s <code><a href="#dom-cssmathmax-values">values</a></code> internal slot to <var>values</var>. Otherwise, prepend <var>this</var> to <var>values</var>.

3.  <a id="ref-for-list-item③④"></a>

    <a id="ref-for-cssunitvalue①⑥"></a>

    <a id="ref-for-dom-cssunitvalue-unit①④"></a>

    <a id="ref-for-cssunitvalue①⑦"></a>

    <a id="ref-for-dom-cssunitvalue-unit①⑤"></a>

    <a id="ref-for-dom-cssunitvalue-unit①⑥"></a>

    <a id="ref-for-dom-cssunitvalue-value①②"></a>

    <a id="ref-for-dom-cssunitvalue-value①③"></a>

    <a id="ref-for-list-item③⑤"></a>

    If all of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var> are <code><a href="#cssunitvalue">CSSUnitValue</a></code>s and have the same <code><a href="#dom-cssunitvalue-unit">unit</a></code>, return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> whose <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot is set to <var>this</var>’s <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot, and <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to the maximum of the <code><a href="#dom-cssunitvalue-value">value</a></code> internal slots of the [items](https://infra.spec.whatwg.org/#list-item) in <var>values</var>.

4.  <a id="ref-for-cssnumericvalue-add-two-types②"></a>

    <a id="ref-for-cssnumericvalue-type③"></a>

    <a id="ref-for-list-item③⑥"></a>

    <a id="ref-for-dfn-throw②⑤"></a>

    <a id="ref-for-exceptiondef-typeerror②⑤"></a>

    Let <var>type</var> be the result of [adding](#cssnumericvalue-add-two-types) the [types](#cssnumericvalue-type) of every [item](https://infra.spec.whatwg.org/#list-item) in <var>values</var>. If <var>type</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

5.  <a id="ref-for-cssmathmax①"></a>

    <a id="ref-for-dom-cssmathmax-values①"></a>

    Return a new <code><a href="#cssmathmax">CSSMathMax</a></code> object whose <code><a href="#dom-cssmathmax-values">values</a></code> internal slot is set to <var>values</var>.

<a id="ref-for-cssnumericvalue②⑥"></a>

The <a id="dom-cssnumericvalue-equals"></a><code>equals(...<var>values</var>)</code> method, when called on a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>, must perform the following steps:

1.  <a id="ref-for-list-item③⑦"></a>

    <a id="ref-for-rectify-a-numberish-value⑥"></a>

    <a id="ref-for-list-item③⑧"></a>

    Replace each [item](https://infra.spec.whatwg.org/#list-item) of <var>values</var> with the result of [rectifying a numberish value](#rectify-a-numberish-value) for the [item](https://infra.spec.whatwg.org/#list-item).

2.  <a id="ref-for-list-item③⑨"></a>

    <a id="ref-for-list-item④⓪"></a>

    <a id="ref-for-equal-numeric-value"></a>

    For each [item](https://infra.spec.whatwg.org/#list-item) in <var>values</var>, if the [item](https://infra.spec.whatwg.org/#list-item) is not an [equal numeric value](#equal-numeric-value) to <var>this</var>, return `false`.

3.  Return `true`.

> <strong data-conversion-semantic="note">Note</strong>
>
> This notion of equality is purposely fairly exacting; all the values must be the exact same type and value, in the same order. For example, `CSSMathSum(CSS.px(1), CSS.px(2))` is <em>not</em> equal to `CSSMathSum(CSS.px(2), CSS.px(1))`.
>
> This precise notion is used because it allows structural equality to be tested for very quickly; if we were to use a slower and more forgiving notion of equality, such as allowing the arguments to match in any order, we’d probably want to go all the way and perform other simplifications, like considering 96px to be equal to 1in; this looser notion of equality might be added in the future.

<a id="ref-for-cssnumericvalue②⑦"></a>

To determine whether two <code><a href="#cssnumericvalue">CSSNumericValue</a></code>s <var>value1</var> and <var>value2</var> are <a id="equal-numeric-value"></a>equal numeric values, perform the following steps:

1.  If <var>value1</var> and <var>value2</var> are not members of the same interface, return `false`.

2.  <a id="ref-for-cssunitvalue①⑧"></a>

    <a id="ref-for-dom-cssunitvalue-unit①⑦"></a>

    <a id="ref-for-dom-cssunitvalue-value①④"></a>

    If <var>value1</var> and <var>value2</var> are both <code><a href="#cssunitvalue">CSSUnitValue</a></code>s, return `true` if they have equal <code><a href="#dom-cssunitvalue-unit">unit</a></code> and <code><a href="#dom-cssunitvalue-value">value</a></code> internal slots, or `false` otherwise.

3.  <a id="ref-for-cssmathsum④"></a>

    <a id="ref-for-cssmathproduct②"></a>

    <a id="ref-for-cssmathmin②"></a>

    <a id="ref-for-cssmathmax②"></a>

    If <var>value1</var> and <var>value2</var> are both <code><a href="#cssmathsum">CSSMathSum</a></code>s, <code><a href="#cssmathproduct">CSSMathProduct</a></code>s, <code><a href="#cssmathmin">CSSMathMin</a></code>s, or <code><a href="#cssmathmax">CSSMathMax</a></code>s:

    1.  <a id="ref-for-dom-cssmathsum-values③"></a>

        <a id="ref-for-dom-cssmathsum-values④"></a>

        <a id="ref-for-list-size④"></a>

        If <var>value1</var>’s <code><a href="#dom-cssmathsum-values">values</a></code> and <var>value2</var>s <code><a href="#dom-cssmathsum-values">values</a></code> internal slots have different [sizes](https://infra.spec.whatwg.org/#list-size), return `false`.

    2.  <a id="ref-for-list-item④①"></a>

        <a id="ref-for-dom-cssmathsum-values⑤"></a>

        <a id="ref-for-equal-numeric-value①"></a>

        <a id="ref-for-list-item④②"></a>

        <a id="ref-for-dom-cssmathsum-values⑥"></a>

        If any [item](https://infra.spec.whatwg.org/#list-item) in <var>value1</var>’s <code><a href="#dom-cssmathsum-values">values</a></code> internal slot is not an [equal numeric value](#equal-numeric-value) to the [item](https://infra.spec.whatwg.org/#list-item) in <var>value2</var>’s <code><a href="#dom-cssmathsum-values">values</a></code> internal slot at the same index, return `false`.

    3.  Return `true`.

4.  <a id="ref-for-cssmathnegate②"></a>

    <a id="ref-for-cssmathinvert②"></a>

    Assert: <var>value1</var> and <var>value2</var> are both <code><a href="#cssmathnegate">CSSMathNegate</a></code>s or <code><a href="#cssmathinvert">CSSMathInvert</a></code>s.

5.  <a id="ref-for-dom-cssmathnegate-value②"></a>

    <a id="ref-for-dom-cssmathnegate-value③"></a>

    <a id="ref-for-equal-numeric-value②"></a>

    Return whether <var>value1</var>’s <code><a href="#dom-cssmathnegate-value">value</a></code> and <var>value2</var>’s <code><a href="#dom-cssmathnegate-value">value</a></code> are [equal numeric values](#equal-numeric-value).

<a id="ref-for-cssnumericvalue②⑧"></a>

The <a id="dom-cssnumericvalue-to"></a><code>to(<var>unit</var>)</code> method converts an existing <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var> into another one with the specified <var>unit</var>, if possible. When called, it must perform the following steps:

1.  <a id="ref-for-cssnumericvalue-create-a-type"></a>

    <a id="ref-for-dfn-throw②⑥"></a>

    <a id="ref-for-syntaxerror"></a>

    Let <var>type</var> be the result of [creating a type](#cssnumericvalue-create-a-type) from <var>unit</var>. If <var>type</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code>.

2.  <a id="ref-for-create-a-sum-value"></a>

    <a id="ref-for-dfn-throw②⑦"></a>

    <a id="ref-for-exceptiondef-typeerror②⑥"></a>

    Let <var>sum</var> be the result of [creating a sum value](#create-a-sum-value) from <var>this</var>. If <var>sum</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-list-item④③"></a>

    <a id="ref-for-dfn-throw②⑧"></a>

    <a id="ref-for-exceptiondef-typeerror②⑦"></a>

    <a id="ref-for-create-a-cssunitvalue-from-a-sum-value-item"></a>

    <a id="ref-for-list-item④④"></a>

    <a id="ref-for-convert-a-cssunitvalue"></a>

    <a id="ref-for-dfn-throw②⑨"></a>

    <a id="ref-for-exceptiondef-typeerror②⑧"></a>

    If <var>sum</var> has more than one [item](https://infra.spec.whatwg.org/#list-item), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>. Otherwise, let <var>item</var> be the result of [creating a CSSUnitValue](#create-a-cssunitvalue-from-a-sum-value-item) from the sole [item](https://infra.spec.whatwg.org/#list-item) in <var>sum</var>, then [converting](#convert-a-cssunitvalue) it to <var>unit</var>. If <var>item</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

4.  Return <var>item</var>.

When asked to <a id="create-a-cssunitvalue-from-a-sum-value-item"></a>create a CSSUnitValue from a sum value item <var>item</var>, perform the following steps:

1.  <a id="ref-for-map-entry"></a>

    <a id="ref-for-sum-value-unit-map"></a>

    If <var>item</var> has more than one [entry](https://infra.spec.whatwg.org/#map-entry) in its [unit map](#sum-value-unit-map), return failure.

2.  <a id="ref-for-map-entry①"></a>

    <a id="ref-for-sum-value-unit-map①"></a>

    <a id="ref-for-cssunitvalue①⑨"></a>

    <a id="ref-for-dom-cssunitvalue-unit①⑧"></a>

    <a id="ref-for-dom-cssunitvalue-value①⑤"></a>

    <a id="ref-for-sum-value-value"></a>

    If <var>item</var> has no [entries](https://infra.spec.whatwg.org/#map-entry) in its [unit map](#sum-value-unit-map), return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> whose <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot is set to "number", and whose <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to <var>item</var>’s [value](#sum-value-value).

3.  <a id="ref-for-map-entry②"></a>

    <a id="ref-for-sum-value-unit-map②"></a>

    <a id="ref-for-map-entry③"></a>

    <a id="ref-for-map-value"></a>

    Otherwise, <var>item</var> has a single [entry](https://infra.spec.whatwg.org/#map-entry) in its [unit map](#sum-value-unit-map). If that [entry’s](https://infra.spec.whatwg.org/#map-entry) [value](https://infra.spec.whatwg.org/#map-value) is anything other than `1`, return failure.

4.  <a id="ref-for-cssunitvalue②⓪"></a>

    <a id="ref-for-dom-cssunitvalue-unit①⑨"></a>

    <a id="ref-for-map-entry④"></a>

    <a id="ref-for-map-key"></a>

    <a id="ref-for-dom-cssunitvalue-value①⑥"></a>

    <a id="ref-for-sum-value-value①"></a>

    Otherwise, return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> whose <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot is set to that [entry’s](https://infra.spec.whatwg.org/#map-entry) [key](https://infra.spec.whatwg.org/#map-key), and whose <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to <var>item</var>’s [value](#sum-value-value).

<a id="ref-for-cssnumericvalue②⑨"></a>

<a id="ref-for-cssmathsum⑤"></a>

<a id="ref-for-cssunitvalue②①"></a>

<a id="ref-for-dom-cssnumericvalue-to①"></a>

<a id="ref-for-cssunitvalue②②"></a>

The <a id="dom-cssnumericvalue-tosum"></a><code>toSum(...<var>units</var>)</code> method converts an existing <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var> into a <code><a href="#cssmathsum">CSSMathSum</a></code> of only <code><a href="#cssunitvalue">CSSUnitValue</a></code>s with the specified units, if possible. (It’s like <code><a href="#dom-cssnumericvalue-to">to()</a></code>, but allows the result to have multiple units in it.) If called without any units, it just simplifies <var>this</var> into a minimal sum of <code><a href="#cssunitvalue">CSSUnitValue</a></code>s.

When called, it must perform the following steps:

1.  <a id="ref-for-list-iterate①"></a>

    <a id="ref-for-cssnumericvalue-create-a-type①"></a>

    <a id="ref-for-dfn-throw③⓪"></a>

    <a id="ref-for-syntaxerror①"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>unit</var> in <var>units</var>, if the result of [creating a type](#cssnumericvalue-create-a-type) from <var>unit</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code>.

2.  <a id="ref-for-create-a-sum-value①"></a>

    <a id="ref-for-dfn-throw③①"></a>

    <a id="ref-for-exceptiondef-typeerror②⑨"></a>

    Let <var>sum</var> be the result of [creating a sum value](#create-a-sum-value) from <var>this</var>. If <var>sum</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-create-a-cssunitvalue-from-a-sum-value-item①"></a>

    <a id="ref-for-list-iterate②"></a>

    <a id="ref-for-list-item④⑤"></a>

    <a id="ref-for-list-item④⑥"></a>

    <a id="ref-for-dfn-throw③②"></a>

    <a id="ref-for-exceptiondef-typeerror③⓪"></a>

    Let <var>values</var> be the result of [creating a CSSUnitValue](#create-a-cssunitvalue-from-a-sum-value-item) [for each](https://infra.spec.whatwg.org/#list-iterate) [item](https://infra.spec.whatwg.org/#list-item) in <var>sum</var>. If any [item](https://infra.spec.whatwg.org/#list-item) of <var>values</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

4.  <a id="ref-for-list-empty"></a>

    <a id="ref-for-code-point"></a>

    <a id="ref-for-dom-cssunitvalue-unit②⓪"></a>

    <a id="ref-for-list-item④⑦"></a>

    <a id="ref-for-cssmathsum⑥"></a>

    <a id="ref-for-dom-cssmathsum-values⑦"></a>

    If <var>units</var> is [empty](https://infra.spec.whatwg.org/#list-empty), sort <var>values</var> in [code point](https://infra.spec.whatwg.org/#code-point) order according to the <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot of its [items](https://infra.spec.whatwg.org/#list-item), then return a new <code><a href="#cssmathsum">CSSMathSum</a></code> object whose <code><a href="#dom-cssmathsum-values">values</a></code> internal slot is set to <var>values</var>.

5.  <a id="ref-for-list⑧"></a>

    <a id="ref-for-list-iterate③"></a>

    Otherwise, let <var>result</var> initially be an empty [list](https://infra.spec.whatwg.org/#list). [For each](https://infra.spec.whatwg.org/#list-iterate) <var>unit</var> in <var>units</var>:

    1.  <a id="ref-for-cssunitvalue②③"></a>

        <a id="ref-for-dom-cssunitvalue-unit②①"></a>

        <a id="ref-for-dom-cssunitvalue-value①⑦"></a>

        Let <var>temp</var> initially be a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> whose <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot is set to <var>unit</var> and whose <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to `0`.

    2.  <a id="ref-for-list-iterate④"></a>

        [For each](https://infra.spec.whatwg.org/#list-iterate) <var>value</var> in <var>values</var>:

        1.  <a id="ref-for-dom-cssunitvalue-unit②②"></a>

            Let <var>value unit</var> be <var>value</var>’s <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot.

        2.  <a id="ref-for-compatible-units"></a>

            If <var>value unit</var> is a [compatible unit](https://www.w3.org/TR/css-values-4/#compatible-units) with <var>unit</var>, then:

            1.  <a id="ref-for-convert-a-cssunitvalue①"></a>

                [Convert](#convert-a-cssunitvalue) <var>value</var> to <var>unit</var>.

            2.  <a id="ref-for-dom-cssunitvalue-value①⑧"></a>

                <a id="ref-for-dom-cssunitvalue-value①⑨"></a>

                Increment <var>temp</var>’s <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot by the value of <var>value</var>’s <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot.

            3.  <a id="ref-for-list-remove"></a>

                [Remove](https://infra.spec.whatwg.org/#list-remove) <var>value</var> from <var>values</var>.

    3.  <a id="ref-for-list-append③"></a>

        [Append](https://infra.spec.whatwg.org/#list-append) <var>temp</var> to <var>result</var>.

6.  <a id="ref-for-list-empty①"></a>

    <a id="ref-for-dfn-throw③③"></a>

    <a id="ref-for-exceptiondef-typeerror③①"></a>

    If <var>values</var> is not [empty](https://infra.spec.whatwg.org/#list-empty), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>. <strong data-conversion-semantic="note">Note:</strong> <var>this</var> had units that you didn’t ask for.

7.  <a id="ref-for-cssmathsum⑦"></a>

    <a id="ref-for-dom-cssmathsum-values⑧"></a>

    Return a new <code><a href="#cssmathsum">CSSMathSum</a></code> object whose <code><a href="#dom-cssmathsum-values">values</a></code> internal slot is set to <var>result</var>.

<a id="ref-for-cssnumericvalue-type④"></a>

The <a id="dom-cssnumericvalue-type"></a>`type()` method returns a representation of the [type](#cssnumericvalue-type) of <var>this</var>.

When called, it must perform the following steps:

1.  <a id="ref-for-dictdef-cssnumerictype①"></a>

    Let <var>result</var> be a new <code><a href="#dictdef-cssnumerictype">CSSNumericType</a></code>.

2.  <a id="ref-for-cssnumericvalue-type⑤"></a>

    For each <var>baseType</var> → <var>power</var> in the [type](#cssnumericvalue-type) of <var>this</var>,

    1.  If <var>power</var> is not 0, set <var>result</var>\[<var>baseType</var>\] to <var>power</var>.

3.  <a id="ref-for-cssnumericvalue-percent-hint"></a>

    If the [percent hint](#cssnumericvalue-percent-hint) of <var>this</var> is not null,

    1.  <a id="ref-for-dom-cssnumerictype-percenthint"></a>

        <a id="ref-for-cssnumericvalue-percent-hint①"></a>

        Set <code><a href="#dom-cssnumerictype-percenthint">percentHint</a></code> to the [percent hint](#cssnumericvalue-percent-hint) of <var>this</var>.

4.  Return <var>result</var>.

<a id="ref-for-cssnumericvalue③⓪"></a>

<a id="ref-for-cssnumericvalue③①"></a>

<a id="ref-for-cssnumericvalue-sum-value"></a>

A <a id="cssnumericvalue-sum-value"></a>sum value is an abstract representation of a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> as a sum of numbers with (possibly complex) units. Not all <code><a href="#cssnumericvalue">CSSNumericValue</a></code>s can be expressed as a [sum value](#cssnumericvalue-sum-value).

<a id="ref-for-cssnumericvalue-sum-value①"></a>

<a id="ref-for-list⑨"></a>

<a id="ref-for-tuple"></a>

<a id="ref-for-ordered-map③"></a>

A [sum value](#cssnumericvalue-sum-value) is a [list](https://infra.spec.whatwg.org/#list). Each entry in the list is a [tuple](https://infra.spec.whatwg.org/#tuple) of a <a id="sum-value-value"></a>value, which is a number, and a <a id="sum-value-unit-map"></a>unit map, which is a [map](https://infra.spec.whatwg.org/#ordered-map) of units (strings) to powers (integers).

<a id="ref-for-cssnumericvalue-sum-value②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-18256559"></a> Here are a few examples of CSS values, and their equivalent [sum values](#cssnumericvalue-sum-value):
>
> - 1px becomes `«(1, «["px" → 1]»)»`
>
> - <a id="ref-for-in"></a>
>
>   <a id="ref-for-px"></a>
>
>   <a id="ref-for-compatible-units①"></a>
>
>   <a id="ref-for-px①"></a>
>
>   <a id="ref-for-canonical-unit"></a>
>
>   calc(1px + 1in) becomes `«(97, «["px" → 1]»)»` (because [in](https://www.w3.org/TR/css-values-4/#in) and [px](https://www.w3.org/TR/css-values-4/#px) are [compatible units](https://www.w3.org/TR/css-values-4/#compatible-units), and [px](https://www.w3.org/TR/css-values-4/#px) is the [canonical unit](https://www.w3.org/TR/css-values-4/#canonical-unit) for them)
>
> - calc(1px + 2em) becomes `«(1, «["px" → 1]»), (2, «["em" → 1]»)»`
>
> - <a id="ref-for-cssnumericvalue-type⑥"></a>
>
>   calc(1px + 2%) becomes `«(1, «["px" → 1]»), (2, «["percent" → 1]»)»` (percentages are allowed to add to other units, but aren’t resolved into another unit, like they are in a [type](#cssnumericvalue-type))
>
> - calc(1px \* 2em) becomes `«(2, «["em" → 1, "px" → 1]»)»`
>
> - <a id="ref-for-cssnumericvalue-sum-value③"></a>
>
>   calc(1px + 1deg) can’t be represented as a [sum value](#cssnumericvalue-sum-value) because it’s an invalid computation
>
> - calc(1px \* 2deg) becomes `«(2, «["deg" → 1, "px" → 1]»)»`

<a id="ref-for-cssnumericvalue③②"></a>

To <a id="create-a-sum-value"></a>create a sum value from a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>, the steps differ based on <var>this</var>’s class:

<a id="ref-for-cssunitvalue②④"></a>

<code><a href="#cssunitvalue">CSSUnitValue</a></code>

1.  <a id="ref-for-dom-cssunitvalue-unit②③"></a>

    <a id="ref-for-dom-cssunitvalue-value②⓪"></a>

    Let <var>unit</var> be the value of <var>this</var>’s <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot, and <var>value</var> be the value of <var>this</var>’s <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot.

2.  <a id="ref-for-compatible-units②"></a>

    <a id="ref-for-canonical-unit①"></a>

    <a id="ref-for-canonical-unit②"></a>

    <a id="ref-for-canonical-unit③"></a>

    If <var>unit</var> is a member of a set of [compatible units](https://www.w3.org/TR/css-values-4/#compatible-units), and is not the set’s [canonical unit](https://www.w3.org/TR/css-values-4/#canonical-unit), multiply <var>value</var> by the conversion ratio between <var>unit</var> and the [canonical unit](https://www.w3.org/TR/css-values-4/#canonical-unit), and change <var>unit</var> to the [canonical unit](https://www.w3.org/TR/css-values-4/#canonical-unit).

3.  If <var>unit</var> is `"number"`, return «(<var>value</var>, «\[ \]»)».

4.  Otherwise, return <code>«(<var>value</var>, «&#x5B;<var>unit</var> → 1&#x5D;»)»</code>.

<a id="ref-for-cssmathsum⑧"></a>

<code><a href="#cssmathsum">CSSMathSum</a></code>

1.  <a id="ref-for-list①⓪"></a>

    Let <var>values</var> initially be an empty [list](https://infra.spec.whatwg.org/#list).

2.  <a id="ref-for-list-iterate⑤"></a>

    <a id="ref-for-dom-cssmathsum-values⑨"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>item</var> in <var>this</var>’s <code><a href="#dom-cssmathsum-values">values</a></code> internal slot:

    1.  <a id="ref-for-create-a-sum-value②"></a>

        Let <var>value</var> be the result of [creating a sum value](#create-a-sum-value) from <var>item</var>. If <var>value</var> is failure, return failure.

    2.  <a id="ref-for-list-iterate⑥"></a>

        [For each](https://infra.spec.whatwg.org/#list-iterate) <var>subvalue</var> of <var>value</var>:

        1.  <a id="ref-for-list-item④⑧"></a>

            <a id="ref-for-sum-value-unit-map③"></a>

            <a id="ref-for-list-item④⑨"></a>

            <a id="ref-for-sum-value-value②"></a>

            <a id="ref-for-sum-value-value③"></a>

            If <var>values</var> already contains an [item](https://infra.spec.whatwg.org/#list-item) with the same [unit map](#sum-value-unit-map) as <var>subvalue</var>, increment that [item](https://infra.spec.whatwg.org/#list-item)’s [value](#sum-value-value) by the [value](#sum-value-value) of <var>subvalue</var>.

        2.  <a id="ref-for-list-append④"></a>

            Otherwise, [append](https://infra.spec.whatwg.org/#list-append) <var>subvalue</var> to <var>values</var>.

3.  <a id="ref-for-create-a-type-from-a-unit-map"></a>

    <a id="ref-for-sum-value-unit-map④"></a>

    <a id="ref-for-list-item⑤⓪"></a>

    <a id="ref-for-cssnumericvalue-add-two-types③"></a>

    [Create a type](#create-a-type-from-a-unit-map) from the [unit map](#sum-value-unit-map) of each [item](https://infra.spec.whatwg.org/#list-item) of <var>values</var>, and [add](#cssnumericvalue-add-two-types) all the types together. If the result is failure, return failure.

4.  Return <var>values</var>.

<a id="ref-for-cssmathnegate③"></a>

<code><a href="#cssmathnegate">CSSMathNegate</a></code>

1.  <a id="ref-for-create-a-sum-value③"></a>

    <a id="ref-for-dom-cssmathnegate-value④"></a>

    Let <var>values</var> be the result of [creating a sum value](#create-a-sum-value) from <var>this</var>’s <code><a href="#dom-cssmathnegate-value">value</a></code> internal slot.

2.  If <var>values</var> is failure, return failure.

3.  <a id="ref-for-sum-value-value④"></a>

    <a id="ref-for-list-item⑤①"></a>

    Negate the [value](#sum-value-value) of each [item](https://infra.spec.whatwg.org/#list-item) of <var>values</var>.

4.  Return <var>values</var>.

<a id="ref-for-cssmathproduct③"></a>

<code><a href="#cssmathproduct">CSSMathProduct</a></code>

1.  <a id="ref-for-cssnumericvalue-sum-value④"></a>

    Let <var>values</var> initially be the [sum value](#cssnumericvalue-sum-value) «(1, «\[ \]»)». (I.e. what you’d get from 1.)

2.  <a id="ref-for-list-iterate⑦"></a>

    <a id="ref-for-dom-cssmathproduct-values②"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>item</var> in <var>this</var>’s <code><a href="#dom-cssmathproduct-values">values</a></code> internal slot:

    1.  <a id="ref-for-create-a-sum-value④"></a>

        <a id="ref-for-list①①"></a>

        Let <var>new values</var> be the result of [creating a sum value](#create-a-sum-value) from <var>item</var>. Let <var>temp</var> initially be an empty [list](https://infra.spec.whatwg.org/#list).

    2.  If <var>new values</var> is failure, return failure.

    3.  <a id="ref-for-list-iterate⑧"></a>

        [For each](https://infra.spec.whatwg.org/#list-iterate) <var>item1</var> in <var>values</var>:

        1.  <a id="ref-for-list-iterate⑨"></a>

            [For each](https://infra.spec.whatwg.org/#list-iterate) <var>item2</var> in <var>new values</var>:

            1.  <a id="ref-for-tuple①"></a>

                <a id="ref-for-sum-value-value⑤"></a>

                <a id="ref-for-sum-value-value⑥"></a>

                <a id="ref-for-sum-value-unit-map⑤"></a>

                <a id="ref-for-product-of-two-unit-maps"></a>

                <a id="ref-for-sum-value-unit-map⑥"></a>

                <a id="ref-for-map-entry⑤"></a>

                Let <var>item</var> be a [tuple](https://infra.spec.whatwg.org/#tuple) with its [value](#sum-value-value) set to the product of the [values](#sum-value-value) of <var>item1</var> and <var>item2</var>, and its [unit map](#sum-value-unit-map) set to the [product](#product-of-two-unit-maps) of the [unit maps](#sum-value-unit-map) of <var>item1</var> and <var>item2</var>, with all [entries](https://infra.spec.whatwg.org/#map-entry) with a zero value removed.

            2.  Append <var>item</var> to <var>temp</var>.

    4.  Set <var>values</var> to <var>temp</var>.

3.  Return <var>values</var>.

<a id="ref-for-cssmathinvert③"></a>

<code><a href="#cssmathinvert">CSSMathInvert</a></code>

1.  <a id="ref-for-create-a-sum-value⑤"></a>

    <a id="ref-for-dom-cssmathinvert-value②"></a>

    Let <var>values</var> be the result of [creating a sum value](#create-a-sum-value) from <var>this</var>’s <code><a href="#dom-cssmathinvert-value">value</a></code> internal slot.

2.  If <var>values</var> is failure, return failure.

3.  If the length of values is more than one, return failure.

4.  <a id="ref-for-sum-value-value⑦"></a>

    <a id="ref-for-list-item⑤②"></a>

    <a id="ref-for-map-value①"></a>

    <a id="ref-for-map-entry⑥"></a>

    <a id="ref-for-sum-value-unit-map⑦"></a>

    Invert (find the reciprocal of) the [value](#sum-value-value) of the [item](https://infra.spec.whatwg.org/#list-item) in <var>values</var>, and negate the [value](https://infra.spec.whatwg.org/#map-value) of each [entry](https://infra.spec.whatwg.org/#map-entry) in its [unit map](#sum-value-unit-map).

5.  Return <var>values</var>.

<a id="ref-for-cssmathmin③"></a>

<code><a href="#cssmathmin">CSSMathMin</a></code>

1.  <a id="ref-for-create-a-sum-value⑥"></a>

    <a id="ref-for-list-iterate①⓪"></a>

    <a id="ref-for-list-item⑤③"></a>

    <a id="ref-for-dom-cssmathmin-values②"></a>

    Let <var>args</var> be the result of [creating a sum value](#create-a-sum-value) [for each](https://infra.spec.whatwg.org/#list-iterate) [item](https://infra.spec.whatwg.org/#list-item) in <var>this</var>’s <code><a href="#dom-cssmathmin-values">values</a></code> internal slot.

2.  <a id="ref-for-list-item⑤④"></a>

    If any [item](https://infra.spec.whatwg.org/#list-item) of <var>args</var> is failure, or has a length greater than one, return failure.

3.  <a id="ref-for-sum-value-unit-map⑧"></a>

    <a id="ref-for-list-item⑤⑤"></a>

    If not all of the [unit maps](#sum-value-unit-map) among the [items](https://infra.spec.whatwg.org/#list-item) of <var>args</var> are identical, return failure.

4.  <a id="ref-for-list-item⑤⑥"></a>

    <a id="ref-for-list-item⑤⑦"></a>

    <a id="ref-for-sum-value-value⑧"></a>

    Return the [item](https://infra.spec.whatwg.org/#list-item) of <var>args</var> whose sole [item](https://infra.spec.whatwg.org/#list-item) has the smallest [value](#sum-value-value).

<a id="ref-for-cssmathmax③"></a>

<code><a href="#cssmathmax">CSSMathMax</a></code>

1.  <a id="ref-for-create-a-sum-value⑦"></a>

    <a id="ref-for-list-iterate①①"></a>

    <a id="ref-for-list-item⑤⑧"></a>

    <a id="ref-for-dom-cssmathmax-values②"></a>

    Let <var>args</var> be the result of [creating a sum value](#create-a-sum-value) [for each](https://infra.spec.whatwg.org/#list-iterate) [item](https://infra.spec.whatwg.org/#list-item) in <var>this</var>’s <code><a href="#dom-cssmathmax-values">values</a></code> internal slot.

2.  <a id="ref-for-list-item⑤⑨"></a>

    If any [item](https://infra.spec.whatwg.org/#list-item) of <var>args</var> is failure, or has a length greater than one, return failure.

3.  <a id="ref-for-sum-value-unit-map⑨"></a>

    <a id="ref-for-list-item⑥⓪"></a>

    If not all of the [unit maps](#sum-value-unit-map) among the [items](https://infra.spec.whatwg.org/#list-item) of <var>args</var> are identical, return failure.

4.  <a id="ref-for-list-item⑥①"></a>

    <a id="ref-for-list-item⑥②"></a>

    <a id="ref-for-sum-value-value⑨"></a>

    Return the [item](https://infra.spec.whatwg.org/#list-item) of <var>args</var> whose sole [item](https://infra.spec.whatwg.org/#list-item) has the largest [value](#sum-value-value).

To <a id="create-a-type-from-a-unit-map"></a>create a type from a unit map <var>unit map</var>:

1.  <a id="ref-for-list①②"></a>

    Let <var>types</var> be an initially empty [list](https://infra.spec.whatwg.org/#list).

2.  <a id="ref-for-map-iterate①"></a>

    [For each](https://infra.spec.whatwg.org/#map-iterate) <var>unit</var> → <var>power</var> in <var>unit map</var>:

    1.  <a id="ref-for-cssnumericvalue-create-a-type②"></a>

        Let <var>type</var> be the result of [creating a type](#cssnumericvalue-create-a-type) from <var>unit</var>.

    2.  <a id="ref-for-map-value②"></a>

        Set <var>type</var>’s sole [value](https://infra.spec.whatwg.org/#map-value) to <var>power</var>.

    3.  <a id="ref-for-list-append⑤"></a>

        [Append](https://infra.spec.whatwg.org/#list-append) <var>type</var> to <var>types</var>.

3.  <a id="ref-for-cssnumericvalue-multiply-two-types①"></a>

    <a id="ref-for-list-item⑥③"></a>

    Return the result of [multiplying](#cssnumericvalue-multiply-two-types) all the [items](https://infra.spec.whatwg.org/#list-item) of <var>types</var>.

The <a id="product-of-two-unit-maps"></a>product of two unit maps <var>units1</var> and <var>units2</var> is the result given by the following steps:

1.  Let <var>result</var> be a copy of <var>units1</var>.

2.  <a id="ref-for-map-iterate②"></a>

    [For each](https://infra.spec.whatwg.org/#map-iterate) <var>unit</var> → <var>power</var> in <var>units2</var>:

    1.  <a id="ref-for-map-exists⑥"></a>

        If <var>result</var>\[<var>unit</var>\] [exists](https://infra.spec.whatwg.org/#map-exists), increment <var>result</var>\[<var>unit</var>\] by <var>power</var>.

    2.  Otherwise, set <var>result</var>\[<var>unit</var>\] to <var>power</var>.

3.  Return <var>result</var>.

<a id="ref-for-dom-cssnumericvalue-parse①"></a>

<a id="ref-for-cssnumericvalue③③"></a>

<a id="ref-for-cssnumericvalue③④"></a>

<a id="ref-for-cssnumericvalue③⑤"></a>

The <code><a href="#dom-cssnumericvalue-parse">parse()</a></code> method allows a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> to be constructed directly from a string containing CSS. Note that this is a static method, existing directly on the <code><a href="#cssnumericvalue">CSSNumericValue</a></code> interface object, rather than on <code><a href="#cssnumericvalue">CSSNumericValue</a></code> instances.

The <a id="dom-cssnumericvalue-parse"></a><code>parse(<var>cssText</var>)</code> method, when called, must perform the following steps:

1.  <a id="ref-for-parse-a-component-value"></a>

    <a id="ref-for-dfn-throw③④"></a>

    <a id="ref-for-syntaxerror②"></a>

    [Parse a component value](https://www.w3.org/TR/css-syntax-3/#parse-a-component-value) from <var>cssText</var> and let <var>result</var> be the result. If <var>result</var> is a syntax error, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code> and abort this algorithm.

2.  <a id="ref-for-typedef-number-token"></a>

    <a id="ref-for-typedef-percentage-token"></a>

    <a id="ref-for-typedef-dimension-token"></a>

    <a id="ref-for-math-function"></a>

    <a id="ref-for-dfn-throw③⑤"></a>

    <a id="ref-for-syntaxerror③"></a>

    If <var>result</var> is not a [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token), [\<percentage-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-percentage-token), [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token), or a [math function](https://www.w3.org/TR/css-values-4/#math-function), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code> and abort this algorithm.

3.  <a id="ref-for-typedef-dimension-token①"></a>

    <a id="ref-for-cssnumericvalue-create-a-type③"></a>

    <a id="ref-for-dfn-throw③⑥"></a>

    <a id="ref-for-syntaxerror④"></a>

    If <var>result</var> is a [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token) and [creating a type](#cssnumericvalue-create-a-type) from <var>result</var>’s unit returns failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code> and abort this algorithm.

4.  <a id="ref-for-reify-a-numeric-value"></a>

    [Reify a numeric value](#reify-a-numeric-value) <var>result</var>, and return the result.

#### <a id="numeric-typing"></a>4.3.2. Numeric Value Typing

<a id="ref-for-cssnumericvalue③⑥"></a>

<a id="ref-for-ordered-map④"></a>

<a id="ref-for-cssnumericvalue-base-type"></a>

<a id="ref-for-length-value"></a>

<a id="ref-for-cssnumericvalue-percent-hint②"></a>

<a id="ref-for-cssnumericvalue-base-type①"></a>

Each <code><a href="#cssnumericvalue">CSSNumericValue</a></code> has an associated <a id="cssnumericvalue-type"></a>type, which is a [map](https://infra.spec.whatwg.org/#ordered-map) of [base types](#cssnumericvalue-base-type) to integers (denoting the exponent of each type, so a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)<sup>2</sup>, such as from calc(1px \* 1em), is «\[ "length" → 2 \]»), and an associated [percent hint](#cssnumericvalue-percent-hint) (indicating that the type actually holds a percentage, but that percentage will eventually resolve to the hinted [base type](#cssnumericvalue-base-type), and so has been replaced with it in the type).

<a id="ref-for-cssnumericvalue-type⑦"></a>

<a id="ref-for-cssnumericvalue-base-type②"></a>

<a id="ref-for-cssnumericvalue-base-type③"></a>

The <a id="cssnumericvalue-base-type"></a>base types are "length", "angle", "time", "frequency", "resolution", "flex", and "percent". The ordering of a [type](#cssnumericvalue-type)’s entries always matches this [base type](#cssnumericvalue-base-type) ordering. The <a id="cssnumericvalue-percent-hint"></a>percent hint is either null or a [base type](#cssnumericvalue-base-type) other than "percent".

<a id="ref-for-cssnumericvalue-base-type④"></a>

<a id="ref-for-math-function①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As new unit types are added to CSS, they’ll be added to this list of [base types](#cssnumericvalue-base-type), and to the CSS [math functions](https://www.w3.org/TR/css-values-4/#math-function).

To <a id="cssnumericvalue-create-a-type"></a>create a type from a string <var>unit</var>, follow the appropriate branch of the following:

<var>unit</var> is "number"

Return «\[ \]» (empty map)

<var>unit</var> is "percent"

Return «\[ "percent" → 1 \]»

<a id="ref-for-length-value①"></a>

<var>unit</var> is a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) unit

Return «\[ "length" → 1 \]»

<a id="ref-for-angle-value"></a>

<var>unit</var> is an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) unit

Return «\[ "angle" → 1 \]»

<a id="ref-for-time-value"></a>

<var>unit</var> is a [\<time\>](https://www.w3.org/TR/css-values-4/#time-value) unit

Return «\[ "time" → 1 \]»

<a id="ref-for-frequency-value"></a>

<var>unit</var> is a [\<frequency\>](https://www.w3.org/TR/css-values-4/#frequency-value) unit

Return «\[ "frequency" → 1 \]»

<a id="ref-for-resolution-value"></a>

<var>unit</var> is a [\<resolution\>](https://www.w3.org/TR/css-values-4/#resolution-value) unit

Return «\[ "resolution" → 1 \]»

<a id="ref-for-typedef-flex"></a>

<var>unit</var> is a [\<flex\>](https://www.w3.org/TR/css-grid-2/#typedef-flex) unit

Return «\[ "flex" → 1 \]»

anything else

Return failure.

<a id="ref-for-cssnumericvalue-percent-hint③"></a>

In all cases, the associated [percent hint](#cssnumericvalue-percent-hint) is null.

To <a id="cssnumericvalue-add-two-types"></a>add two types <var>type1</var> and <var>type2</var>, perform the following steps:

1.  <a id="ref-for-cssnumericvalue-type⑧"></a>

    <a id="ref-for-ordered-map⑤"></a>

    <a id="ref-for-cssnumericvalue-percent-hint④"></a>

    Replace <var>type1</var> with a fresh copy of <var>type1</var>, and <var>type2</var> with a fresh copy of <var>type2</var>. Let <var>finalType</var> be a new [type](#cssnumericvalue-type) with an initially empty [ordered map](https://infra.spec.whatwg.org/#ordered-map) and an initially null [percent hint](#cssnumericvalue-percent-hint).

2.  <a id="ref-for-cssnumericvalue-percent-hint⑤"></a>

    If both <var>type1</var> and <var>type2</var> have non-null [percent hints](#cssnumericvalue-percent-hint) with different values

    The types can’t be added. Return failure.

    <a id="ref-for-cssnumericvalue-percent-hint⑥"></a>

    If <var>type1</var> has a non-null [percent hint](#cssnumericvalue-percent-hint) <var>hint</var> and <var>type2</var> doesn’t

    <a id="ref-for-apply-the-percent-hint"></a>

    [Apply the percent hint](#apply-the-percent-hint) <var>hint</var> to <var>type2</var>.

    <a id="ref-for-cssnumericvalue-percent-hint⑦"></a>

    Vice versa if <var>type2</var> has a non-null [percent hint](#cssnumericvalue-percent-hint) and <var>type1</var> doesn’t.

    Otherwise

    Continue to the next step.

3.  <a id="ref-for-map-exists⑦"></a>

    <a id="ref-for-map-entry⑦"></a>

    If all the [entries](https://infra.spec.whatwg.org/#map-entry) of <var>type1</var> with non-zero values are [contained](https://infra.spec.whatwg.org/#map-exists) in <var>type2</var> with the same value, and vice-versa

    <a id="ref-for-map-entry⑧"></a>

    <a id="ref-for-map-entry⑨"></a>

    <a id="ref-for-map-exists⑧"></a>

    <a id="ref-for-cssnumericvalue-percent-hint⑧"></a>

    <a id="ref-for-cssnumericvalue-percent-hint⑨"></a>

    Copy all of <var>type1</var>’s [entries](https://infra.spec.whatwg.org/#map-entry) to <var>finalType</var>, and then copy all of <var>type2</var>’s [entries](https://infra.spec.whatwg.org/#map-entry) to <var>finalType</var> that <var>finalType</var> doesn’t already [contain](https://infra.spec.whatwg.org/#map-exists). Set <var>finalType</var>’s [percent hint](#cssnumericvalue-percent-hint) to <var>type1</var>’s [percent hint](#cssnumericvalue-percent-hint). Return <var>finalType</var>.

    <a id="ref-for-map-exists①⓪"></a>

    <a id="ref-for-map-exists⑨"></a>

    If <var>type1</var> and/or <var>type2</var> [contain](https://infra.spec.whatwg.org/#map-exists) "percent" with a non-zero value, and <var>type1</var> and/or <var>type2</var> [contain](https://infra.spec.whatwg.org/#map-exists) a key <em>other than</em> "percent" with a non-zero value

    <a id="ref-for-cssnumericvalue-base-type⑤"></a>

    For each [base type](#cssnumericvalue-base-type) other than "percent" <var>hint</var>:

    1.  <a id="ref-for-apply-the-percent-hint①"></a>

        Provisionally [apply the percent hint](#apply-the-percent-hint) <var>hint</var> to both <var>type1</var> and <var>type2</var>.

    2.  <a id="ref-for-map-entry①⓪"></a>

        <a id="ref-for-map-exists①①"></a>

        <a id="ref-for-map-entry①①"></a>

        <a id="ref-for-map-entry①②"></a>

        <a id="ref-for-map-exists①②"></a>

        <a id="ref-for-cssnumericvalue-percent-hint①⓪"></a>

        If, afterwards, all the [entries](https://infra.spec.whatwg.org/#map-entry) of <var>type1</var> with non-zero values are [contained](https://infra.spec.whatwg.org/#map-exists) in <var>type2</var> with the same value, and vice versa, then copy all of <var>type1</var>’s [entries](https://infra.spec.whatwg.org/#map-entry) to <var>finalType</var>, and then copy all of <var>type2</var>’s [entries](https://infra.spec.whatwg.org/#map-entry) to <var>finalType</var> that <var>finalType</var> doesn’t already [contain](https://infra.spec.whatwg.org/#map-exists). Set <var>finalType</var>’s [percent hint](#cssnumericvalue-percent-hint) to <var>hint</var>. Return <var>finalType</var>.

    3.  Otherwise, revert <var>type1</var> and <var>type2</var> to their state at the start of this loop.

    If the loop finishes without returning <var>finalType</var>, then the types can’t be added. Return failure.

    <a id="ref-for-map-getting-the-values"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: You can shortcut this in some cases by just checking the sum of all the [values](https://infra.spec.whatwg.org/#map-getting-the-values) of <var>type1</var> vs <var>type2</var>. If the sums are different, the types can’t be added.

    Otherwise

    The types can’t be added. Return failure.

To <a id="apply-the-percent-hint"></a>apply the percent hint <var>hint</var> to a <var>type</var>, perform the following steps:

1.  <a id="ref-for-map-exists①③"></a>

    If <var>type</var> doesn’t [contain](https://infra.spec.whatwg.org/#map-exists) <var>hint</var>, set <var>type</var>\[<var>hint</var>\] to 0.

2.  <a id="ref-for-map-exists①④"></a>

    If <var>type</var> [contains](https://infra.spec.whatwg.org/#map-exists) "percent", add <var>type</var>\["percent"\] to <var>type</var>\[<var>hint</var>\], then set <var>type</var>\["percent"\] to 0.

3.  <a id="ref-for-cssnumericvalue-percent-hint①①"></a>

    Set <var>type</var>’s [percent hint](#cssnumericvalue-percent-hint) to <var>hint</var>.

To <a id="cssnumericvalue-multiply-two-types"></a>multiply two types <var>type1</var> and <var>type2</var>, perform the following steps:

1.  <a id="ref-for-cssnumericvalue-type⑨"></a>

    <a id="ref-for-ordered-map⑥"></a>

    <a id="ref-for-cssnumericvalue-percent-hint①②"></a>

    Replace <var>type1</var> with a fresh copy of <var>type1</var>, and <var>type2</var> with a fresh copy of <var>type2</var>. Let <var>finalType</var> be a new [type](#cssnumericvalue-type) with an initially empty [ordered map](https://infra.spec.whatwg.org/#ordered-map) and an initially null [percent hint](#cssnumericvalue-percent-hint).

2.  <a id="ref-for-cssnumericvalue-percent-hint①③"></a>

    If both <var>type1</var> and <var>type2</var> have non-null [percent hints](#cssnumericvalue-percent-hint) with different values, the types can’t be multiplied. Return failure.

3.  <a id="ref-for-cssnumericvalue-percent-hint①④"></a>

    <a id="ref-for-apply-the-percent-hint②"></a>

    If <var>type1</var> has a non-null [percent hint](#cssnumericvalue-percent-hint) <var>hint</var> and <var>type2</var> doesn’t, [apply the percent hint](#apply-the-percent-hint) <var>hint</var> to <var>type2</var>.

    <a id="ref-for-cssnumericvalue-percent-hint①⑤"></a>

    Vice versa if <var>type2</var> has a non-null [percent hint](#cssnumericvalue-percent-hint) and <var>type1</var> doesn’t.

4.  <a id="ref-for-map-entry①③"></a>

    <a id="ref-for-map-iterate③"></a>

    Copy all of <var>type1</var>’s [entries](https://infra.spec.whatwg.org/#map-entry) to <var>finalType</var>, then [for each](https://infra.spec.whatwg.org/#map-iterate) <var>baseType</var> → <var>power</var> of <var>type2</var>:

    1.  <a id="ref-for-map-exists①⑤"></a>

        If <var>finalType</var>\[<var>baseType</var>\] [exists](https://infra.spec.whatwg.org/#map-exists), increment its value by <var>power</var>.

    2.  Otherwise, set <var>finalType</var>\[<var>baseType</var>\] to <var>power</var>.

    <a id="ref-for-cssnumericvalue-percent-hint①⑥"></a>

    <a id="ref-for-cssnumericvalue-percent-hint①⑦"></a>

    Set <var>finalType</var>’s [percent hint](#cssnumericvalue-percent-hint) to <var>type1</var>’s [percent hint](#cssnumericvalue-percent-hint).

5.  Return <var>finalType</var>.

To <a id="cssnumericvalue-invert-a-type"></a>invert a type <var>type</var>, perform the following steps:

1.  <a id="ref-for-cssnumericvalue-type①⓪"></a>

    <a id="ref-for-ordered-map⑦"></a>

    <a id="ref-for-cssnumericvalue-percent-hint①⑧"></a>

    Let <var>result</var> be a new [type](#cssnumericvalue-type) with an initially empty [ordered map](https://infra.spec.whatwg.org/#ordered-map) and a [percent hint](#cssnumericvalue-percent-hint) matching that of <var>type</var>.

2.  <a id="ref-for-map-iterate④"></a>

    [For each](https://infra.spec.whatwg.org/#map-iterate) <var>unit</var> → <var>exponent</var> of <var>type</var>, set <var>result</var>\[<var>unit</var>\] to (-1 \* <var>exponent</var>).

3.  Return <var>result</var>.

<a id="ref-for-cssnumericvalue-type①①"></a>

A [type](#cssnumericvalue-type) is said to <a id="cssnumericvalue-match"></a>match a CSS production in some circumstances:

- <a id="ref-for-cssnumericvalue-type①②"></a>

  <a id="ref-for-length-value②"></a>

  <a id="ref-for-map-entry①④"></a>

  <a id="ref-for-angle-value①"></a>

  <a id="ref-for-time-value①"></a>

  <a id="ref-for-frequency-value①"></a>

  <a id="ref-for-resolution-value①"></a>

  <a id="ref-for-typedef-flex①"></a>

  A [type](#cssnumericvalue-type) matches [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) if its only non-zero [entry](https://infra.spec.whatwg.org/#map-entry) is «\[ "length" → 1 \]». Similarly for [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), [\<time\>](https://www.w3.org/TR/css-values-4/#time-value), [\<frequency\>](https://www.w3.org/TR/css-values-4/#frequency-value), [\<resolution\>](https://www.w3.org/TR/css-values-4/#resolution-value), and [\<flex\>](https://www.w3.org/TR/css-grid-2/#typedef-flex).

  <a id="ref-for-percentage-value②"></a>

  <a id="ref-for-cssnumericvalue-type①③"></a>

  <a id="ref-for-cssnumericvalue-percent-hint①⑨"></a>

  <a id="ref-for-cssnumericvalue-match①"></a>

  If the context in which the value is used does not allow [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) values, then the [type](#cssnumericvalue-type) must additionally have a null [percent hint](#cssnumericvalue-percent-hint) to be considered [matching](#cssnumericvalue-match).

- <a id="ref-for-cssnumericvalue-type①④"></a>

  <a id="ref-for-percentage-value③"></a>

  <a id="ref-for-map-entry①⑤"></a>

  A [type](#cssnumericvalue-type) matches [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) if its only non-zero [entry](https://infra.spec.whatwg.org/#map-entry) is «\[ "percent" → 1 \]».

- <a id="ref-for-cssnumericvalue-type①⑤"></a>

  <a id="ref-for-typedef-length-percentage"></a>

  <a id="ref-for-map-entry①⑥"></a>

  <a id="ref-for-typedef-angle-percentage"></a>

  <a id="ref-for-typedef-time-percentage"></a>

  A [type](#cssnumericvalue-type) matches [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) if its only non-zero [entry](https://infra.spec.whatwg.org/#map-entry) is either «\[ "length" → 1 \]» or «\[ "percent" → 1 \]». Same for [\<angle-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-angle-percentage), [\<time-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-time-percentage), etc.

- <a id="ref-for-cssnumericvalue-type①⑥"></a>

  <a id="ref-for-number-value②"></a>

  <a id="ref-for-map-entry①⑦"></a>

  A [type](#cssnumericvalue-type) matches [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) if it has no non-zero [entries](https://infra.spec.whatwg.org/#map-entry).

  <a id="ref-for-percentage-value④"></a>

  <a id="ref-for-cssnumericvalue-type①⑦"></a>

  <a id="ref-for-cssnumericvalue-percent-hint②⓪"></a>

  <a id="ref-for-cssnumericvalue-match②"></a>

  If the context in which the value is used does not allow [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) values, then the [type](#cssnumericvalue-type) must additionally have a null [percent hint](#cssnumericvalue-percent-hint) to be considered [matching](#cssnumericvalue-match).

<a id="ref-for-cssnumericvalue-type①⑧"></a>

<a id="ref-for-cssnumericvalue-percent-hint②①"></a>

<a id="ref-for-cssnumericvalue-add-two-types④"></a>

<a id="ref-for-cssnumericvalue-add-two-types⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Types](#cssnumericvalue-type) form a semi-group under both addition and a monoid under multiplication (with the multiplicative identity being «\[ \]» with a null [percent hint](#cssnumericvalue-percent-hint)), meaning that they’re associative and commutative. Thus the spec can, for example, [add](#cssnumericvalue-add-two-types) an unbounded number of types together unambiguously, rather than having to manually [add](#cssnumericvalue-add-two-types) them pair-wise.

<a id="ref-for-cssunitvalue②⑤"></a>

#### <a id="simple-numeric"></a>4.3.3. Value + Unit: <code><a href="#cssunitvalue">CSSUnitValue</a></code> objects

<a id="ref-for-cssunitvalue②⑥"></a>

Numeric values that can be expressed as a single unit (or a naked number or percentage) are represented as <code><a href="#cssunitvalue">CSSUnitValue</a></code>s.

<a id="ref-for-cssunitvalue②⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e128c925"></a> For example, the value 5px in a stylesheet will be represented by a <code><a href="#cssunitvalue">CSSUnitValue</a></code> with its `value` attribute set to `5` and its `unit` attribute set to `"px"`.
>
> <a id="ref-for-cssunitvalue②⑧"></a>
>
> Similarly, the value 10 in a stylesheet will be represented by a <code><a href="#cssunitvalue">CSSUnitValue</a></code> with its `value` attribute set to `10` and its `unit` attribute set to `"number"`.

<a id="ref-for-Exposed①⓪"></a>

<a id="cssunitvalue"></a>

<a id="ref-for-cssnumericvalue③⑦"></a>

<a id="ref-for-dom-cssunitvalue-cssunitvalue"></a>

<a id="ref-for-idl-double③"></a>

<a id="dom-cssunitvalue-cssunitvalue-value-unit-value"></a>

<a id="ref-for-idl-USVString②④"></a>

<a id="dom-cssunitvalue-cssunitvalue-value-unit-unit"></a>

<a id="ref-for-idl-double④"></a>

<a id="dom-cssunitvalue-value"></a>

<a id="ref-for-idl-USVString②⑤"></a>

<a id="dom-cssunitvalue-unit"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSUnitValue : CSSNumericValue {
    constructor(double value, USVString unit);
    attribute double value;
    readonly attribute USVString unit;
};
```
The <a id="dom-cssunitvalue-cssunitvalue"></a><code>CSSUnitValue(<var>value</var>, <var>unit</var>)</code> constructor must, when called, perform the following steps:

1.  <a id="ref-for-cssnumericvalue-create-a-type④"></a>

    <a id="ref-for-dfn-throw③⑦"></a>

    <a id="ref-for-exceptiondef-typeerror③②"></a>

    If [creating a type](#cssnumericvalue-create-a-type) from <var>unit</var> returns failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code> and abort this algorithm.

2.  <a id="ref-for-cssunitvalue②⑨"></a>

    <a id="ref-for-dom-cssunitvalue-value②①"></a>

    <a id="ref-for-dom-cssunitvalue-unit②④"></a>

    Return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> with its <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot set to <var>value</var> and its <code><a href="#dom-cssunitvalue-unit">unit</a></code> set to <var>unit</var>.

<a id="ref-for-cssnumericvalue-type①⑨"></a>

<a id="ref-for-cssunitvalue③⓪"></a>

<a id="ref-for-cssnumericvalue-create-a-type⑤"></a>

<a id="ref-for-dom-cssunitvalue-unit②⑤"></a>

The <a id="type-of-a-cssunitvalue"></a>[type](#cssnumericvalue-type) of a <code><a href="#cssunitvalue">CSSUnitValue</a></code> is the result of [creating a type](#cssnumericvalue-create-a-type) from its <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot.

<a id="ref-for-cssunitvalue③①"></a>

<a id="ref-for-dom-cssunitvalue-value②②"></a>

<a id="ref-for-dom-cssunitvalue-unit②⑥"></a>

To <a id="create-a-cssunitvalue-from-a-pair"></a>create a CSSUnitValue from a pair (<var>num</var>, <var>unit</var>), return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> object with its <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot set to <var>num</var>, and its <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot set to <var>unit</var>.

<a id="ref-for-create-a-cssunitvalue-from-a-pair"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c9504d03"></a> For example, creating a [new unit value](#create-a-cssunitvalue-from-a-pair) from `(5, "px")` creates an object equivalent to `new CSSUnitValue(5, "px")`.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is a spec-internal algorithm, meant simply to make it easier to create unit values in algorithms when needed.

To <a id="convert-a-cssunitvalue"></a>convert a CSSUnitValue <var>this</var> to a unit <var>unit</var>, perform the following steps:

1.  <a id="ref-for-dom-cssunitvalue-unit②⑦"></a>

    <a id="ref-for-dom-cssunitvalue-value②③"></a>

    Let <var>old unit</var> be the value of <var>this</var>’s <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot, and <var>old value</var> be the value of <var>this</var>’s <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot.

2.  <a id="ref-for-compatible-units③"></a>

    If <var>old unit</var> and <var>unit</var> are not [compatible units](https://www.w3.org/TR/css-values-4/#compatible-units), return failure.

3.  <a id="ref-for-cssunitvalue③②"></a>

    <a id="ref-for-dom-cssunitvalue-unit②⑧"></a>

    <a id="ref-for-dom-cssunitvalue-value②④"></a>

    Return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> whose <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot is set to <var>unit</var>, and whose <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to <var>old value</var> multiplied by the conversation ratio between <var>old unit</var> and <var>unit</var>.

<a id="ref-for-cssmathvalue①"></a>

#### <a id="complex-numeric"></a>4.3.4. Complex Numeric Values: <code><a href="#cssmathvalue">CSSMathValue</a></code> objects

<a id="ref-for-cssmathvalue②"></a>

<a id="ref-for-cssunitvalue③③"></a>

<a id="ref-for-funcdef-calc"></a>

<a id="ref-for-funcdef-min"></a>

<a id="ref-for-funcdef-max"></a>

Numeric values that are more complicated than a single value+unit are represented by a tree of <code><a href="#cssmathvalue">CSSMathValue</a></code> subclasses, eventually terminating in <code><a href="#cssunitvalue">CSSUnitValue</a></code> objects at the leaf nodes. The [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc), [min()](https://www.w3.org/TR/css-values-4/#funcdef-min), and [max()](https://www.w3.org/TR/css-values-4/#funcdef-max) functions in CSS are represented in this way.

<a id="ref-for-cssmathsum⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5e437640"></a> For example, the CSS value calc(1em + 5px) will be represented by a <code><a href="#cssmathsum">CSSMathSum</a></code> like `CSSMathSum(CSS.em(1), CSS.px(5))`.
>
> A more complex expression, like calc(1em + 5px \* 2), will be represented by a nested structure like `CSSMathSum(CSS.em(1), CSSMathProduct(CSS.px(5), 2))`.

<a id="ref-for-Exposed①①"></a>

<a id="cssmathvalue"></a>

<a id="ref-for-cssnumericvalue③⑧"></a>

<a id="ref-for-enumdef-cssmathoperator"></a>

<a id="ref-for-dom-cssmathvalue-operator"></a>

<a id="ref-for-Exposed①②"></a>

<a id="cssmathsum"></a>

<a id="ref-for-cssmathvalue③"></a>

<a id="ref-for-dom-cssmathsum-cssmathsum"></a>

<a id="ref-for-typedefdef-cssnumberish⑦"></a>

<a id="dom-cssmathsum-cssmathsum-args-args"></a>

<a id="ref-for-cssnumericarray"></a>

<a id="dom-cssmathsum-values"></a>

<a id="ref-for-Exposed①③"></a>

<a id="cssmathproduct"></a>

<a id="ref-for-cssmathvalue④"></a>

<a id="ref-for-dom-cssmathproduct-cssmathproduct"></a>

<a id="ref-for-typedefdef-cssnumberish⑧"></a>

<a id="dom-cssmathproduct-cssmathproduct-args-args"></a>

<a id="ref-for-cssnumericarray①"></a>

<a id="dom-cssmathproduct-values"></a>

<a id="ref-for-Exposed①④"></a>

<a id="cssmathnegate"></a>

<a id="ref-for-cssmathvalue⑤"></a>

<a id="ref-for-dom-cssmathnegate-cssmathnegate"></a>

<a id="ref-for-typedefdef-cssnumberish⑨"></a>

<a id="dom-cssmathnegate-cssmathnegate-arg-arg"></a>

<a id="ref-for-cssnumericvalue③⑨"></a>

<a id="dom-cssmathnegate-value"></a>

<a id="ref-for-Exposed①⑤"></a>

<a id="cssmathinvert"></a>

<a id="ref-for-cssmathvalue⑥"></a>

<a id="ref-for-dom-cssmathinvert-cssmathinvert"></a>

<a id="ref-for-typedefdef-cssnumberish①⓪"></a>

<a id="dom-cssmathinvert-cssmathinvert-arg-arg"></a>

<a id="ref-for-cssnumericvalue④⓪"></a>

<a id="dom-cssmathinvert-value"></a>

<a id="ref-for-Exposed①⑥"></a>

<a id="cssmathmin"></a>

<a id="ref-for-cssmathvalue⑦"></a>

<a id="ref-for-dom-cssmathmin-cssmathmin"></a>

<a id="ref-for-typedefdef-cssnumberish①①"></a>

<a id="dom-cssmathmin-cssmathmin-args-args"></a>

<a id="ref-for-cssnumericarray②"></a>

<a id="dom-cssmathmin-values"></a>

<a id="ref-for-Exposed①⑦"></a>

<a id="cssmathmax"></a>

<a id="ref-for-cssmathvalue⑧"></a>

<a id="ref-for-dom-cssmathmax-cssmathmax"></a>

<a id="ref-for-typedefdef-cssnumberish①②"></a>

<a id="dom-cssmathmax-cssmathmax-args-args"></a>

<a id="ref-for-cssnumericarray③"></a>

<a id="dom-cssmathmax-values"></a>

<a id="ref-for-Exposed①⑧"></a>

<a id="cssmathclamp"></a>

<a id="ref-for-cssmathvalue⑨"></a>

<a id="ref-for-dom-cssmathclamp-cssmathclamp"></a>

<a id="ref-for-typedefdef-cssnumberish①③"></a>

<a id="dom-cssmathclamp-cssmathclamp-lower-value-upper-lower"></a>

<a id="ref-for-typedefdef-cssnumberish①④"></a>

<a id="dom-cssmathclamp-cssmathclamp-lower-value-upper-value"></a>

<a id="ref-for-typedefdef-cssnumberish①⑤"></a>

<a id="dom-cssmathclamp-cssmathclamp-lower-value-upper-upper"></a>

<a id="ref-for-cssnumericvalue④①"></a>

<a id="dom-cssmathclamp-lower"></a>

<a id="ref-for-cssnumericvalue④②"></a>

<a id="dom-cssmathclamp-value"></a>

<a id="ref-for-cssnumericvalue④③"></a>

<a id="dom-cssmathclamp-upper"></a>

<a id="ref-for-Exposed①⑨"></a>

<a id="cssnumericarray"></a>

<a id="ref-for-cssnumericvalue④④"></a>

<a id="ref-for-idl-unsigned-long④"></a>

<a id="ref-for-dom-cssnumericarray-length"></a>

<a id="ref-for-cssnumericvalue④⑤"></a>

<a id="ref-for-idl-unsigned-long⑤"></a>

<a id="dom-cssnumericarray-__getter__-index-index"></a>

<a id="enumdef-cssmathoperator"></a>

<a id="dom-cssmathoperator-sum"></a>

<a id="dom-cssmathoperator-product"></a>

<a id="dom-cssmathoperator-negate"></a>

<a id="dom-cssmathoperator-invert"></a>

<a id="dom-cssmathoperator-min"></a>

<a id="dom-cssmathoperator-max"></a>

<a id="dom-cssmathoperator-clamp"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathValue : CSSNumericValue {
    readonly attribute CSSMathOperator operator;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathSum : CSSMathValue {
    constructor(CSSNumberish... args);
    readonly attribute CSSNumericArray values;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathProduct : CSSMathValue {
    constructor(CSSNumberish... args);
    readonly attribute CSSNumericArray values;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathNegate : CSSMathValue {
    constructor(CSSNumberish arg);
    readonly attribute CSSNumericValue value;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathInvert : CSSMathValue {
    constructor(CSSNumberish arg);
    readonly attribute CSSNumericValue value;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathMin : CSSMathValue {
    constructor(CSSNumberish... args);
    readonly attribute CSSNumericArray values;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathMax : CSSMathValue {
    constructor(CSSNumberish... args);
    readonly attribute CSSNumericArray values;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathClamp : CSSMathValue {
    constructor(CSSNumberish lower, CSSNumberish value, CSSNumberish upper);
    readonly attribute CSSNumericValue lower;
    readonly attribute CSSNumericValue value;
    readonly attribute CSSNumericValue upper;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSNumericArray {
    iterable<CSSNumericValue>;
    readonly attribute unsigned long length;
    getter CSSNumericValue (unsigned long index);
};

enum CSSMathOperator {
    "sum",
    "product",
    "negate",
    "invert",
    "min",
    "max",
    "clamp",
};
```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: CSSMathValue, being a pure superclass, cannot be directly constructed. It exists solely to host the common attributes of all the "math" operations.

<a id="ref-for-cssmathvalue①⓪"></a>

The <a id="dom-cssmathvalue-operator"></a>`operator` attribute of a <code><a href="#cssmathvalue">CSSMathValue</a></code> <var>this</var> must, on getting, return the following string, depending on the interface of <var>this</var>:

<a id="ref-for-cssmathsum①⓪"></a>

<code><a href="#cssmathsum">CSSMathSum</a></code>

`"sum"`

<a id="ref-for-cssmathproduct④"></a>

<code><a href="#cssmathproduct">CSSMathProduct</a></code>

`"product"`

<a id="ref-for-cssmathmin④"></a>

<code><a href="#cssmathmin">CSSMathMin</a></code>

`"min"`

<a id="ref-for-cssmathmax④"></a>

<code><a href="#cssmathmax">CSSMathMax</a></code>

`"max"`

<a id="ref-for-cssmathclamp"></a>

<code><a href="#cssmathclamp">CSSMathClamp</a></code>

`"clamp"`

<a id="ref-for-cssmathnegate④"></a>

<code><a href="#cssmathnegate">CSSMathNegate</a></code>

`"negate"`

<a id="ref-for-cssmathinvert④"></a>

<code><a href="#cssmathinvert">CSSMathInvert</a></code>

`"invert"`

<a id="ref-for-enumdef-cssmathoperator①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These are all instances of the <code><a href="#enumdef-cssmathoperator">CSSMathOperator</a></code> enum.

The <a id="dom-cssmathsum-cssmathsum"></a><code>CSSMathSum(...<var>args</var>)</code> constructor must, when called, perform the following steps:

1.  <a id="ref-for-list-item⑥④"></a>

    <a id="ref-for-rectify-a-numberish-value⑦"></a>

    <a id="ref-for-list-item⑥⑤"></a>

    Replace each [item](https://infra.spec.whatwg.org/#list-item) of <var>args</var> with the result of [rectifying a numberish value](#rectify-a-numberish-value) for the [item](https://infra.spec.whatwg.org/#list-item).

2.  <a id="ref-for-list-is-empty"></a>

    <a id="ref-for-dfn-throw③⑧"></a>

    <a id="ref-for-syntaxerror⑤"></a>

    If <var>args</var> [is empty](https://infra.spec.whatwg.org/#list-is-empty), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code>.

3.  <a id="ref-for-cssnumericvalue-add-two-types⑥"></a>

    <a id="ref-for-cssnumericvalue-type②⓪"></a>

    <a id="ref-for-list-item⑥⑥"></a>

    <a id="ref-for-dfn-throw③⑨"></a>

    <a id="ref-for-exceptiondef-typeerror③③"></a>

    Let <var>type</var> be the result of [adding](#cssnumericvalue-add-two-types) the [types](#cssnumericvalue-type) of all the [items](https://infra.spec.whatwg.org/#list-item) of <var>args</var>. If <var>type</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

4.  <a id="ref-for-cssmathsum①①"></a>

    <a id="ref-for-dom-cssmathsum-values①⓪"></a>

    Return a new <code><a href="#cssmathsum">CSSMathSum</a></code> whose <code><a href="#dom-cssmathsum-values">values</a></code> internal slot is set to <var>args</var>.

<a id="ref-for-cssmathmin⑤"></a>

<a id="ref-for-cssmathmax⑤"></a>

The <a id="dom-cssmathmin-cssmathmin"></a><code>CSSMathMin(...<var>args</var>)</code> and <a id="dom-cssmathmax-cssmathmax"></a><code>CSSMathMax(...<var>args</var>)</code> constructors are defined identically to the above, except that in the last step they return a new <code><a href="#cssmathmin">CSSMathMin</a></code> or <code><a href="#cssmathmax">CSSMathMax</a></code> object, respectively.

<a id="ref-for-cssnumericvalue-multiply-two-types②"></a>

<a id="ref-for-cssnumericvalue-add-two-types⑦"></a>

<a id="ref-for-cssmathproduct⑤"></a>

The <a id="dom-cssmathproduct-cssmathproduct"></a><code>CSSMathProduct(...<var>args</var>)</code> constructor is defined identically to the above, except that in step 3 it [multiplies](#cssnumericvalue-multiply-two-types) the types instead of [adding](#cssnumericvalue-add-two-types), and in the last step it returns a <code><a href="#cssmathproduct">CSSMathProduct</a></code>.

The <a id="dom-cssmathclamp-cssmathclamp"></a><code>CSSMathClamp(<var>lower</var>, <var>value</var>, <var>upper</var>)</code> constructor must, when called, perform the following steps:

1.  <a id="ref-for-rectify-a-numberish-value⑧"></a>

    Replace <var>lower</var>, <var>value</var>, and <var>upper</var> with the result of [rectifying a numberish value](#rectify-a-numberish-value) for each.

2.  <a id="ref-for-cssnumericvalue-add-two-types⑧"></a>

    <a id="ref-for-cssnumericvalue-type②①"></a>

    <a id="ref-for-dfn-throw④⓪"></a>

    <a id="ref-for-exceptiondef-typeerror③④"></a>

    Let <var>type</var> be the result of [adding](#cssnumericvalue-add-two-types) the [types](#cssnumericvalue-type) of <var>lower</var>, <var>value</var>, and <var>upper</var>. If <var>type</var> is failure, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-cssmathclamp①"></a>

    <a id="ref-for-dom-cssmathclamp-lower"></a>

    <a id="ref-for-dom-cssmathclamp-value"></a>

    <a id="ref-for-dom-cssmathclamp-upper"></a>

    Return a new <code><a href="#cssmathclamp">CSSMathClamp</a></code> whose <code><a href="#dom-cssmathclamp-lower">lower</a></code>, <code><a href="#dom-cssmathclamp-value">value</a></code>, and <code><a href="#dom-cssmathclamp-upper">upper</a></code> internal slots are set to <var>lower</var>, <var>value</var>, and <var>upper</var>, respectively.

The <a id="dom-cssmathnegate-cssmathnegate"></a><code>CSSMathNegate(<var>arg</var>)</code> constructor must, when called, perform the following steps:

1.  <a id="ref-for-rectify-a-numberish-value⑨"></a>

    Replace <var>arg</var> with the result of [rectifying a numberish value](#rectify-a-numberish-value) for <var>arg</var>.

2.  <a id="ref-for-cssmathnegate⑤"></a>

    <a id="ref-for-dom-cssmathnegate-value⑤"></a>

    Return a new <code><a href="#cssmathnegate">CSSMathNegate</a></code> whose <code><a href="#dom-cssmathnegate-value">value</a></code> internal slot is set to <var>arg</var>.

<a id="ref-for-cssmathinvert⑤"></a>

The <a id="dom-cssmathinvert-cssmathinvert"></a><code>CSSMathInvert(<var>arg</var>)</code> constructor is defined identically to the above, except that in the last step it returns a new <code><a href="#cssmathinvert">CSSMathInvert</a></code> object.

<a id="ref-for-cssnumericvalue-type②②"></a>

The <a id="type-of-a-cssmathvalue"></a>[type](#cssnumericvalue-type) of a CSSMathValue depends on its class:

<a id="ref-for-cssmathsum①②"></a>

<code><a href="#cssmathsum">CSSMathSum</a></code>

<a id="ref-for-cssmathmin⑥"></a>

<code><a href="#cssmathmin">CSSMathMin</a></code>

<a id="ref-for-cssmathmax⑥"></a>

<code><a href="#cssmathmax">CSSMathMax</a></code>

<a id="ref-for-cssnumericvalue-type②③"></a>

<a id="ref-for-cssnumericvalue-add-two-types⑨"></a>

<a id="ref-for-cssnumericvalue-type②④"></a>

<a id="ref-for-list-item⑥⑦"></a>

<a id="ref-for-dom-cssmathsum-values①①"></a>

The [type](#cssnumericvalue-type) is the result of [adding](#cssnumericvalue-add-two-types) the [types](#cssnumericvalue-type) of each of the [items](https://infra.spec.whatwg.org/#list-item) in its <code><a href="#dom-cssmathsum-values">values</a></code> internal slot.

<a id="ref-for-cssmathclamp②"></a>

<code><a href="#cssmathclamp">CSSMathClamp</a></code>

<a id="ref-for-cssnumericvalue-type②⑤"></a>

<a id="ref-for-cssnumericvalue-add-two-types①⓪"></a>

<a id="ref-for-cssnumericvalue-type②⑥"></a>

<a id="ref-for-dom-cssmathclamp-lower①"></a>

<a id="ref-for-dom-cssmathclamp-value①"></a>

<a id="ref-for-dom-cssmathclamp-upper①"></a>

The [type](#cssnumericvalue-type) is the result of [adding](#cssnumericvalue-add-two-types) the [types](#cssnumericvalue-type) of the <code><a href="#dom-cssmathclamp-lower">lower</a></code>, <code><a href="#dom-cssmathclamp-value">value</a></code>, and <code><a href="#dom-cssmathclamp-upper">upper</a></code> internal slots.

<a id="ref-for-cssmathproduct⑥"></a>

<code><a href="#cssmathproduct">CSSMathProduct</a></code>

<a id="ref-for-cssnumericvalue-type②⑦"></a>

<a id="ref-for-cssnumericvalue-multiply-two-types③"></a>

<a id="ref-for-cssnumericvalue-type②⑧"></a>

<a id="ref-for-list-item⑥⑧"></a>

<a id="ref-for-dom-cssmathproduct-values③"></a>

The [type](#cssnumericvalue-type) is the result of [multiplying](#cssnumericvalue-multiply-two-types) the [types](#cssnumericvalue-type) of each of the [items](https://infra.spec.whatwg.org/#list-item) in its <code><a href="#dom-cssmathproduct-values">values</a></code> internal slot.

<a id="ref-for-cssmathnegate⑥"></a>

<code><a href="#cssmathnegate">CSSMathNegate</a></code>

<a id="ref-for-cssnumericvalue-type②⑨"></a>

<a id="ref-for-cssnumericvalue-type③⓪"></a>

<a id="ref-for-dom-cssmathnegate-value⑥"></a>

The [type](#cssnumericvalue-type) is the same as the [type](#cssnumericvalue-type) of its <code><a href="#dom-cssmathnegate-value">value</a></code> internal slot.

<a id="ref-for-cssmathinvert⑥"></a>

<code><a href="#cssmathinvert">CSSMathInvert</a></code>

<a id="ref-for-cssnumericvalue-type③①"></a>

<a id="ref-for-cssnumericvalue-type③②"></a>

<a id="ref-for-dom-cssmathinvert-value③"></a>

<a id="ref-for-map-getting-the-values①"></a>

The [type](#cssnumericvalue-type) is the same as the [type](#cssnumericvalue-type) of its <code><a href="#dom-cssmathinvert-value">value</a></code> internal slot, but with all [values](https://infra.spec.whatwg.org/#map-getting-the-values) negated.

<a id="ref-for-cssnumericarray④"></a>

<a id="ref-for-cssnumericvalue④⑥"></a>

<a id="ref-for-cssnumericarray⑤"></a>

The <a id="dom-cssnumericarray-length"></a>`length` attribute of <code><a href="#cssnumericarray">CSSNumericArray</a></code> indicates how many <code><a href="#cssnumericvalue">CSSNumericValue</a></code>s are contained within the <code><a href="#cssnumericarray">CSSNumericArray</a></code>.

<a id="ref-for-dfn-indexed-property-getter"></a>

<a id="ref-for-cssnumericarray⑥"></a>

<a id="ref-for-cssnumericvalue④⑦"></a>

The <a id="cssnumericarray-indexed-property-getter"></a>[indexed property getter](https://webidl.spec.whatwg.org/#dfn-indexed-property-getter) of <code><a href="#cssnumericarray">CSSNumericArray</a></code> retrieves the <code><a href="#cssnumericvalue">CSSNumericValue</a></code> at the provided index.

#### <a id="numeric-factory"></a>4.3.5. Numeric Factory Functions

The following factory functions can be used to create new numeric values much less verbosely than using the constructors directly.

<a id="ref-for-namespacedef-css"></a>

<a id="ref-for-cssunitvalue③④"></a>

<a id="dom-css-number"></a>

<a id="ref-for-idl-double⑤"></a>

<a id="dom-css-number-value-value"></a>

<a id="ref-for-cssunitvalue③⑤"></a>

<a id="dom-css-percent"></a>

<a id="ref-for-idl-double⑥"></a>

<a id="dom-css-percent-value-value"></a>

<a id="ref-for-cssunitvalue③⑥"></a>

<a id="dom-css-cap"></a>

<a id="ref-for-idl-double⑦"></a>

<a id="dom-css-cap-value-value"></a>

<a id="ref-for-cssunitvalue③⑦"></a>

<a id="dom-css-ch"></a>

<a id="ref-for-idl-double⑧"></a>

<a id="dom-css-ch-value-value"></a>

<a id="ref-for-cssunitvalue③⑧"></a>

<a id="dom-css-em"></a>

<a id="ref-for-idl-double⑨"></a>

<a id="dom-css-em-value-value"></a>

<a id="ref-for-cssunitvalue③⑨"></a>

<a id="dom-css-ex"></a>

<a id="ref-for-idl-double①⓪"></a>

<a id="dom-css-ex-value-value"></a>

<a id="ref-for-cssunitvalue④⓪"></a>

<a id="dom-css-ic"></a>

<a id="ref-for-idl-double①①"></a>

<a id="dom-css-ic-value-value"></a>

<a id="ref-for-cssunitvalue④①"></a>

<a id="dom-css-lh"></a>

<a id="ref-for-idl-double①②"></a>

<a id="dom-css-lh-value-value"></a>

<a id="ref-for-cssunitvalue④②"></a>

<a id="dom-css-rcap"></a>

<a id="ref-for-idl-double①③"></a>

<a id="dom-css-rcap-value-value"></a>

<a id="ref-for-cssunitvalue④③"></a>

<a id="dom-css-rch"></a>

<a id="ref-for-idl-double①④"></a>

<a id="dom-css-rch-value-value"></a>

<a id="ref-for-cssunitvalue④④"></a>

<a id="dom-css-rem"></a>

<a id="ref-for-idl-double①⑤"></a>

<a id="dom-css-rem-value-value"></a>

<a id="ref-for-cssunitvalue④⑤"></a>

<a id="dom-css-rex"></a>

<a id="ref-for-idl-double①⑥"></a>

<a id="dom-css-rex-value-value"></a>

<a id="ref-for-cssunitvalue④⑥"></a>

<a id="dom-css-ric"></a>

<a id="ref-for-idl-double①⑦"></a>

<a id="dom-css-ric-value-value"></a>

<a id="ref-for-cssunitvalue④⑦"></a>

<a id="dom-css-rlh"></a>

<a id="ref-for-idl-double①⑧"></a>

<a id="dom-css-rlh-value-value"></a>

<a id="ref-for-cssunitvalue④⑧"></a>

<a id="dom-css-vw"></a>

<a id="ref-for-idl-double①⑨"></a>

<a id="dom-css-vw-value-value"></a>

<a id="ref-for-cssunitvalue④⑨"></a>

<a id="dom-css-vh"></a>

<a id="ref-for-idl-double②⓪"></a>

<a id="dom-css-vh-value-value"></a>

<a id="ref-for-cssunitvalue⑤⓪"></a>

<a id="dom-css-vi"></a>

<a id="ref-for-idl-double②①"></a>

<a id="dom-css-vi-value-value"></a>

<a id="ref-for-cssunitvalue⑤①"></a>

<a id="dom-css-vb"></a>

<a id="ref-for-idl-double②②"></a>

<a id="dom-css-vb-value-value"></a>

<a id="ref-for-cssunitvalue⑤②"></a>

<a id="dom-css-vmin"></a>

<a id="ref-for-idl-double②③"></a>

<a id="dom-css-vmin-value-value"></a>

<a id="ref-for-cssunitvalue⑤③"></a>

<a id="dom-css-vmax"></a>

<a id="ref-for-idl-double②④"></a>

<a id="dom-css-vmax-value-value"></a>

<a id="ref-for-cssunitvalue⑤④"></a>

<a id="dom-css-svw"></a>

<a id="ref-for-idl-double②⑤"></a>

<a id="dom-css-svw-value-value"></a>

<a id="ref-for-cssunitvalue⑤⑤"></a>

<a id="dom-css-svh"></a>

<a id="ref-for-idl-double②⑥"></a>

<a id="dom-css-svh-value-value"></a>

<a id="ref-for-cssunitvalue⑤⑥"></a>

<a id="dom-css-svi"></a>

<a id="ref-for-idl-double②⑦"></a>

<a id="dom-css-svi-value-value"></a>

<a id="ref-for-cssunitvalue⑤⑦"></a>

<a id="dom-css-svb"></a>

<a id="ref-for-idl-double②⑧"></a>

<a id="dom-css-svb-value-value"></a>

<a id="ref-for-cssunitvalue⑤⑧"></a>

<a id="dom-css-svmin"></a>

<a id="ref-for-idl-double②⑨"></a>

<a id="dom-css-svmin-value-value"></a>

<a id="ref-for-cssunitvalue⑤⑨"></a>

<a id="dom-css-svmax"></a>

<a id="ref-for-idl-double③⓪"></a>

<a id="dom-css-svmax-value-value"></a>

<a id="ref-for-cssunitvalue⑥⓪"></a>

<a id="dom-css-lvw"></a>

<a id="ref-for-idl-double③①"></a>

<a id="dom-css-lvw-value-value"></a>

<a id="ref-for-cssunitvalue⑥①"></a>

<a id="dom-css-lvh"></a>

<a id="ref-for-idl-double③②"></a>

<a id="dom-css-lvh-value-value"></a>

<a id="ref-for-cssunitvalue⑥②"></a>

<a id="dom-css-lvi"></a>

<a id="ref-for-idl-double③③"></a>

<a id="dom-css-lvi-value-value"></a>

<a id="ref-for-cssunitvalue⑥③"></a>

<a id="dom-css-lvb"></a>

<a id="ref-for-idl-double③④"></a>

<a id="dom-css-lvb-value-value"></a>

<a id="ref-for-cssunitvalue⑥④"></a>

<a id="dom-css-lvmin"></a>

<a id="ref-for-idl-double③⑤"></a>

<a id="dom-css-lvmin-value-value"></a>

<a id="ref-for-cssunitvalue⑥⑤"></a>

<a id="dom-css-lvmax"></a>

<a id="ref-for-idl-double③⑥"></a>

<a id="dom-css-lvmax-value-value"></a>

<a id="ref-for-cssunitvalue⑥⑥"></a>

<a id="dom-css-dvw"></a>

<a id="ref-for-idl-double③⑦"></a>

<a id="dom-css-dvw-value-value"></a>

<a id="ref-for-cssunitvalue⑥⑦"></a>

<a id="dom-css-dvh"></a>

<a id="ref-for-idl-double③⑧"></a>

<a id="dom-css-dvh-value-value"></a>

<a id="ref-for-cssunitvalue⑥⑧"></a>

<a id="dom-css-dvi"></a>

<a id="ref-for-idl-double③⑨"></a>

<a id="dom-css-dvi-value-value"></a>

<a id="ref-for-cssunitvalue⑥⑨"></a>

<a id="dom-css-dvb"></a>

<a id="ref-for-idl-double④⓪"></a>

<a id="dom-css-dvb-value-value"></a>

<a id="ref-for-cssunitvalue⑦⓪"></a>

<a id="dom-css-dvmin"></a>

<a id="ref-for-idl-double④①"></a>

<a id="dom-css-dvmin-value-value"></a>

<a id="ref-for-cssunitvalue⑦①"></a>

<a id="dom-css-dvmax"></a>

<a id="ref-for-idl-double④②"></a>

<a id="dom-css-dvmax-value-value"></a>

<a id="ref-for-cssunitvalue⑦②"></a>

<a id="dom-css-cqw"></a>

<a id="ref-for-idl-double④③"></a>

<a id="dom-css-cqw-value-value"></a>

<a id="ref-for-cssunitvalue⑦③"></a>

<a id="dom-css-cqh"></a>

<a id="ref-for-idl-double④④"></a>

<a id="dom-css-cqh-value-value"></a>

<a id="ref-for-cssunitvalue⑦④"></a>

<a id="dom-css-cqi"></a>

<a id="ref-for-idl-double④⑤"></a>

<a id="dom-css-cqi-value-value"></a>

<a id="ref-for-cssunitvalue⑦⑤"></a>

<a id="dom-css-cqb"></a>

<a id="ref-for-idl-double④⑥"></a>

<a id="dom-css-cqb-value-value"></a>

<a id="ref-for-cssunitvalue⑦⑥"></a>

<a id="dom-css-cqmin"></a>

<a id="ref-for-idl-double④⑦"></a>

<a id="dom-css-cqmin-value-value"></a>

<a id="ref-for-cssunitvalue⑦⑦"></a>

<a id="dom-css-cqmax"></a>

<a id="ref-for-idl-double④⑧"></a>

<a id="dom-css-cqmax-value-value"></a>

<a id="ref-for-cssunitvalue⑦⑧"></a>

<a id="dom-css-cm"></a>

<a id="ref-for-idl-double④⑨"></a>

<a id="dom-css-cm-value-value"></a>

<a id="ref-for-cssunitvalue⑦⑨"></a>

<a id="dom-css-mm"></a>

<a id="ref-for-idl-double⑤⓪"></a>

<a id="dom-css-mm-value-value"></a>

<a id="ref-for-cssunitvalue⑧⓪"></a>

<a id="dom-css-q"></a>

<a id="ref-for-idl-double⑤①"></a>

<a id="dom-css-q-value-value"></a>

<a id="ref-for-cssunitvalue⑧①"></a>

<a id="dom-css-in"></a>

<a id="ref-for-idl-double⑤②"></a>

<a id="dom-css-in-value-value"></a>

<a id="ref-for-cssunitvalue⑧②"></a>

<a id="dom-css-pt"></a>

<a id="ref-for-idl-double⑤③"></a>

<a id="dom-css-pt-value-value"></a>

<a id="ref-for-cssunitvalue⑧③"></a>

<a id="dom-css-pc"></a>

<a id="ref-for-idl-double⑤④"></a>

<a id="dom-css-pc-value-value"></a>

<a id="ref-for-cssunitvalue⑧④"></a>

<a id="dom-css-px"></a>

<a id="ref-for-idl-double⑤⑤"></a>

<a id="dom-css-px-value-value"></a>

<a id="ref-for-cssunitvalue⑧⑤"></a>

<a id="dom-css-deg"></a>

<a id="ref-for-idl-double⑤⑥"></a>

<a id="dom-css-deg-value-value"></a>

<a id="ref-for-cssunitvalue⑧⑥"></a>

<a id="dom-css-grad"></a>

<a id="ref-for-idl-double⑤⑦"></a>

<a id="dom-css-grad-value-value"></a>

<a id="ref-for-cssunitvalue⑧⑦"></a>

<a id="dom-css-rad"></a>

<a id="ref-for-idl-double⑤⑧"></a>

<a id="dom-css-rad-value-value"></a>

<a id="ref-for-cssunitvalue⑧⑧"></a>

<a id="dom-css-turn"></a>

<a id="ref-for-idl-double⑤⑨"></a>

<a id="dom-css-turn-value-value"></a>

<a id="ref-for-cssunitvalue⑧⑨"></a>

<a id="dom-css-s"></a>

<a id="ref-for-idl-double⑥⓪"></a>

<a id="dom-css-s-value-value"></a>

<a id="ref-for-cssunitvalue⑨⓪"></a>

<a id="dom-css-ms"></a>

<a id="ref-for-idl-double⑥①"></a>

<a id="dom-css-ms-value-value"></a>

<a id="ref-for-cssunitvalue⑨①"></a>

<a id="dom-css-hz"></a>

<a id="ref-for-idl-double⑥②"></a>

<a id="dom-css-hz-value-value"></a>

<a id="ref-for-cssunitvalue⑨②"></a>

<a id="dom-css-khz"></a>

<a id="ref-for-idl-double⑥③"></a>

<a id="dom-css-khz-value-value"></a>

<a id="ref-for-cssunitvalue⑨③"></a>

<a id="dom-css-dpi"></a>

<a id="ref-for-idl-double⑥④"></a>

<a id="dom-css-dpi-value-value"></a>

<a id="ref-for-cssunitvalue⑨④"></a>

<a id="dom-css-dpcm"></a>

<a id="ref-for-idl-double⑥⑤"></a>

<a id="dom-css-dpcm-value-value"></a>

<a id="ref-for-cssunitvalue⑨⑤"></a>

<a id="dom-css-dppx"></a>

<a id="ref-for-idl-double⑥⑥"></a>

<a id="dom-css-dppx-value-value"></a>

<a id="ref-for-cssunitvalue⑨⑥"></a>

<a id="dom-css-fr"></a>

<a id="ref-for-idl-double⑥⑦"></a>

<a id="dom-css-fr-value-value"></a>

```text
partial namespace CSS {
    CSSUnitValue number(double value);
    CSSUnitValue percent(double value);

    // <length>
    CSSUnitValue cap(double value);
    CSSUnitValue ch(double value);
    CSSUnitValue em(double value);
    CSSUnitValue ex(double value);
    CSSUnitValue ic(double value);
    CSSUnitValue lh(double value);
    CSSUnitValue rcap(double value);
    CSSUnitValue rch(double value);
    CSSUnitValue rem(double value);
    CSSUnitValue rex(double value);
    CSSUnitValue ric(double value);
    CSSUnitValue rlh(double value);
    CSSUnitValue vw(double value);
    CSSUnitValue vh(double value);
    CSSUnitValue vi(double value);
    CSSUnitValue vb(double value);
    CSSUnitValue vmin(double value);
    CSSUnitValue vmax(double value);
    CSSUnitValue svw(double value);
    CSSUnitValue svh(double value);
    CSSUnitValue svi(double value);
    CSSUnitValue svb(double value);
    CSSUnitValue svmin(double value);
    CSSUnitValue svmax(double value);
    CSSUnitValue lvw(double value);
    CSSUnitValue lvh(double value);
    CSSUnitValue lvi(double value);
    CSSUnitValue lvb(double value);
    CSSUnitValue lvmin(double value);
    CSSUnitValue lvmax(double value);
    CSSUnitValue dvw(double value);
    CSSUnitValue dvh(double value);
    CSSUnitValue dvi(double value);
    CSSUnitValue dvb(double value);
    CSSUnitValue dvmin(double value);
    CSSUnitValue dvmax(double value);
    CSSUnitValue cqw(double value);
    CSSUnitValue cqh(double value);
    CSSUnitValue cqi(double value);
    CSSUnitValue cqb(double value);
    CSSUnitValue cqmin(double value);
    CSSUnitValue cqmax(double value);
    CSSUnitValue cm(double value);
    CSSUnitValue mm(double value);
    CSSUnitValue Q(double value);
    CSSUnitValue in(double value);
    CSSUnitValue pt(double value);
    CSSUnitValue pc(double value);
    CSSUnitValue px(double value);

    // <angle>
    CSSUnitValue deg(double value);
    CSSUnitValue grad(double value);
    CSSUnitValue rad(double value);
    CSSUnitValue turn(double value);

    // <time>
    CSSUnitValue s(double value);
    CSSUnitValue ms(double value);

    // <frequency>
    CSSUnitValue Hz(double value);
    CSSUnitValue kHz(double value);

    // <resolution>
    CSSUnitValue dpi(double value);
    CSSUnitValue dpcm(double value);
    CSSUnitValue dppx(double value);

    // <flex>
    CSSUnitValue fr(double value);
};
```
<a id="ref-for-cssunitvalue⑨⑦"></a>

<a id="ref-for-dom-cssunitvalue-value②⑤"></a>

<a id="ref-for-dom-cssunitvalue-unit②⑨"></a>

All of the above methods must, when called with a double <var>value</var>, return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> whose <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot is set to <var>value</var> and whose <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot is set to the name of the method as defined here.

<a id="ref-for-cssunitvalue⑨⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The unit used does not depend on the <em>current</em> name of the function, if it’s stored in another variable; `let foo = CSS.px; let val = foo(5);` does not return a `{value: 5, unit: "foo"}` <code><a href="#cssunitvalue">CSSUnitValue</a></code>. The above talk about names is just a shorthand to avoid defining the unit individually for all ~60 functions.

<a id="ref-for-dictdef-cssnumerictype②"></a>

The above list of methods reflects the set of CSS’s valid predefined units at one particular point in time. It will be updated over time, but might be out-of-date at any given moment. If an implementation supports additional CSS units that do not have a corresponding method in the above list, but that do correspond to one of the existing <code><a href="#dictdef-cssnumerictype">CSSNumericType</a></code> values, it must additionally support such a method, named after the unit in its defined canonical casing, using the generic behavior defined above.

<a id="ref-for-dictdef-cssnumerictype③"></a>

If an implementation supports units that do <em>not</em> correspond to one of the existing <code><a href="#dictdef-cssnumerictype">CSSNumericType</a></code> values, it must not support those units in the APIs defined in this specification; it should request the units and their types be added explicitly to this specification, as the appropriate type name is not implicit from the unit.

If an implementation does not support a given unit, it must not implement its corresponding method from the list above.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d3ba8244"></a> For example, the CSS Speech spec [\[CSS-SPEECH-1\]](#biblio-css-speech-1) defines two additional units, the decibel dB and semitone st. No current browser implementation supports these or has plans to, so they’re not included in the above list, but if an implementation <em>does</em> support the Speec spec, it must also expose `CSS.dB()` and `CSS.st()` methods.

<a id="ref-for-csstransformvalue①"></a>

### <a id="transformvalue-objects"></a>4.4. <code><a href="#csstransformvalue">CSSTransformValue</a></code> objects

<a id="ref-for-csstransformvalue②"></a>

<a id="ref-for-typedef-transform-list①"></a>

<a id="ref-for-propdef-transform"></a>

<a id="ref-for-csstransformcomponent"></a>

<a id="ref-for-typedef-transform-function"></a>

<code><a href="#csstransformvalue">CSSTransformValue</a></code> objects represent [\<transform-list\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-list) values, used by the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property. They "contain" one or more <code><a href="#csstransformcomponent">CSSTransformComponent</a></code>s, which represent individual [\<transform-function\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-function) values.

<a id="ref-for-Exposed②⓪"></a>

<a id="csstransformvalue"></a>

<a id="ref-for-cssstylevalue②⑥"></a>

<a id="ref-for-dom-csstransformvalue-csstransformvalue"></a>

<a id="ref-for-idl-sequence④"></a>

<a id="ref-for-csstransformcomponent①"></a>

<a id="dom-csstransformvalue-csstransformvalue-transforms-transforms"></a>

<a id="ref-for-csstransformcomponent②"></a>

<a id="ref-for-idl-unsigned-long⑥"></a>

<a id="ref-for-dom-csstransformvalue-length"></a>

<a id="ref-for-csstransformcomponent③"></a>

<a id="ref-for-idl-unsigned-long⑦"></a>

<a id="dom-csstransformvalue-__getter__-index-index"></a>

<a id="ref-for-csstransformcomponent④"></a>

<a id="ref-for-idl-unsigned-long⑧"></a>

<a id="dom-csstransformvalue-__setter__-index-val-index"></a>

<a id="ref-for-csstransformcomponent⑤"></a>

<a id="dom-csstransformvalue-__setter__-index-val-val"></a>

<a id="ref-for-idl-boolean②"></a>

<a id="ref-for-dom-csstransformvalue-is2d"></a>

<a id="ref-for-dommatrix"></a>

<a id="ref-for-dom-csstransformvalue-tomatrix"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSTransformValue : CSSStyleValue {
    constructor(sequence<CSSTransformComponent> transforms);
    iterable<CSSTransformComponent>;
    readonly attribute unsigned long length;
    getter CSSTransformComponent (unsigned long index);
    setter CSSTransformComponent (unsigned long index, CSSTransformComponent val);

    readonly attribute boolean is2D;
    DOMMatrix toMatrix();
};
```
<a id="ref-for-csstransformvalue③"></a>

<a id="ref-for-list①③"></a>

<a id="ref-for-csstransformcomponent⑥"></a>

A <code><a href="#csstransformvalue">CSSTransformValue</a></code>’s values to iterate over is a [list](https://infra.spec.whatwg.org/#list) of <code><a href="#csstransformcomponent">CSSTransformComponent</a></code>s.

The <a id="dom-csstransformvalue-csstransformvalue"></a><code>CSSTransformValue(<var>transforms</var>)</code> constructor must, when called, perform the following steps:

1.  <a id="ref-for-list-is-empty①"></a>

    <a id="ref-for-dfn-throw④①"></a>

    <a id="ref-for-exceptiondef-typeerror③⑤"></a>

    If <var>transforms</var> [is empty](https://infra.spec.whatwg.org/#list-is-empty), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-csstransformvalue④"></a>

    Return a new <code><a href="#csstransformvalue">CSSTransformValue</a></code> whose values to iterate over is <var>transforms</var>.

<a id="ref-for-csstransformvalue⑤"></a>

<a id="ref-for-list-iterate①②"></a>

<a id="ref-for-dom-csstransformcomponent-is2d"></a>

The <a id="dom-csstransformvalue-is2d"></a>`is2D` attribute of a <code><a href="#csstransformvalue">CSSTransformValue</a></code> <var>this</var> must, on getting, return `true` if, [for each](https://infra.spec.whatwg.org/#list-iterate) <var>func</var> in <var>this</var>’s values to iterate over, the <var>func</var>’s <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> attribute would return `true`; otherwise, the attribute returns `false`.

<a id="ref-for-csstransformvalue⑥"></a>

The <a id="dom-csstransformvalue-tomatrix"></a>`toMatrix()` method of a <code><a href="#csstransformvalue">CSSTransformValue</a></code> <var>this</var> must, when called, perform the following steps:

1.  <a id="ref-for-dommatrix①"></a>

    <a id="ref-for-dom-dommatrixreadonly-is2d"></a>

    Let <var>matrix</var> be a new <code><a href="https://www.w3.org/TR/geometry-1/#dommatrix">DOMMatrix</a></code>, initialized to the identity matrix, with its <code><a href="https://www.w3.org/TR/geometry-1/#dom-dommatrixreadonly-is2d">is2D</a></code> internal slot set to `true`.

2.  <a id="ref-for-list-iterate①③"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>func</var> in <var>this</var>’s values to iterate over:

    1.  <a id="ref-for-dommatrix②"></a>

        <a id="ref-for-dom-csstransformcomponent-tomatrix"></a>

        Let <var>funcMatrix</var> be the <code><a href="https://www.w3.org/TR/geometry-1/#dommatrix">DOMMatrix</a></code> returned by calling <code><a href="#dom-csstransformcomponent-tomatrix">toMatrix()</a></code> on <var>func</var>.

    2.  Set <var>matrix</var> to the result of multiplying <var>matrix</var> and the matrix represented by <var>funcMatrix</var>.

3.  Return <var>matrix</var>.

<a id="ref-for-csstransformvalue⑦"></a>

The <a id="dom-csstransformvalue-length"></a>`length` attribute indicates how many transform components are contained within the <code><a href="#csstransformvalue">CSSTransformValue</a></code>.

<a id="ref-for-list①④"></a>

<a id="ref-for-csstransformcomponent⑦"></a>

They have a <a id="dom-csstransformvalue-values-slot"></a>`[[values]]` internal slot, which is a [list](https://infra.spec.whatwg.org/#list) of <code><a href="#csstransformcomponent">CSSTransformComponent</a></code> objects. This list is the object’s values to iterate over.

<a id="ref-for-dfn-supported-property-indices①"></a>

<a id="ref-for-csstransformvalue⑧"></a>

<a id="ref-for-list-size⑤"></a>

<a id="ref-for-dom-csstransformvalue-values-slot"></a>

The [supported property indexes](https://webidl.spec.whatwg.org/#dfn-supported-property-indices) of a <code><a href="#csstransformvalue">CSSTransformValue</a></code> <var>this</var> are the integers greater than or equal to 0, and less than the [size](https://infra.spec.whatwg.org/#list-size) of <var>this</var>’s <code><a href="#dom-csstransformvalue-values-slot">&#x5B;&#x5B;values&#x5D;&#x5D;</a></code> internal slot.

<a id="ref-for-dfn-determine-the-value-of-an-indexed-property①"></a>

<a id="ref-for-csstransformvalue⑨"></a>

<a id="ref-for-dom-csstransformvalue-values-slot①"></a>

To [determine the value of an indexed property](https://webidl.spec.whatwg.org/#dfn-determine-the-value-of-an-indexed-property) of a <code><a href="#csstransformvalue">CSSTransformValue</a></code> <var>this</var> and an index <var>n</var>, let <var>values</var> be <var>this</var>’s <code><a href="#dom-csstransformvalue-values-slot">&#x5B;&#x5B;values&#x5D;&#x5D;</a></code> internal slot, and return <var>values</var>\[<var>n</var>\].

<a id="ref-for-dfn-set-the-value-of-an-existing-indexed-property①"></a>

<a id="ref-for-csstransformvalue①⓪"></a>

<a id="ref-for-dom-csstransformvalue-values-slot②"></a>

To [set the value of an existing indexed property](https://webidl.spec.whatwg.org/#dfn-set-the-value-of-an-existing-indexed-property) of a <code><a href="#csstransformvalue">CSSTransformValue</a></code> <var>this</var>, an index <var>n</var>, and a value <var>new value</var>, let <var>values</var> be <var>this</var>’s <code><a href="#dom-csstransformvalue-values-slot">&#x5B;&#x5B;values&#x5D;&#x5D;</a></code> internal slot, and set <var>values</var>\[<var>n</var>\] to <var>new value</var>.

<a id="ref-for-dfn-set-the-value-of-a-new-indexed-property①"></a>

<a id="ref-for-csstransformvalue①①"></a>

<a id="ref-for-dom-csstransformvalue-values-slot③"></a>

<a id="ref-for-list-size⑥"></a>

<a id="ref-for-dfn-throw④②"></a>

<a id="ref-for-exceptiondef-rangeerror②"></a>

<a id="ref-for-list-append⑥"></a>

To [set the value of a new indexed property](https://webidl.spec.whatwg.org/#dfn-set-the-value-of-a-new-indexed-property) of a <code><a href="#csstransformvalue">CSSTransformValue</a></code> <var>this</var>, an index <var>n</var>, and a value <var>new value</var>, let <var>values</var> be <var>this</var>’s <code><a href="#dom-csstransformvalue-values-slot">&#x5B;&#x5B;values&#x5D;&#x5D;</a></code> internal slot. If <var>n</var> is not equal to the [size](https://infra.spec.whatwg.org/#list-size) of <var>values</var>, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-rangeerror">RangeError</a></code>. Otherwise, [append](https://infra.spec.whatwg.org/#list-append) <var>new value</var> to <var>values</var>.

<a id="ref-for-cssnumericvalue④⑧"></a>

<a id="ref-for-typedefdef-csskeywordish"></a>

<a id="typedefdef-cssperspectivevalue"></a>

<a id="ref-for-Exposed②①"></a>

<a id="csstransformcomponent"></a>

<a id="CSSTransformComponent-stringification-behavior"></a>

<a id="ref-for-idl-boolean③"></a>

<a id="ref-for-dom-csstransformcomponent-is2d①"></a>

<a id="ref-for-dommatrix③"></a>

<a id="ref-for-dom-csstransformcomponent-tomatrix①"></a>

<a id="ref-for-Exposed②②"></a>

<a id="csstranslate"></a>

<a id="ref-for-csstransformcomponent⑧"></a>

<a id="ref-for-dom-csstranslate-csstranslate"></a>

<a id="ref-for-cssnumericvalue④⑨"></a>

<a id="dom-csstranslate-csstranslate-x-y-z-x"></a>

<a id="ref-for-cssnumericvalue⑤⓪"></a>

<a id="dom-csstranslate-csstranslate-x-y-z-y"></a>

<a id="ref-for-cssnumericvalue⑤①"></a>

<a id="dom-csstranslate-csstranslate-x-y-z-z"></a>

<a id="ref-for-cssnumericvalue⑤②"></a>

<a id="dom-csstranslate-x"></a>

<a id="ref-for-cssnumericvalue⑤③"></a>

<a id="dom-csstranslate-y"></a>

<a id="ref-for-cssnumericvalue⑤④"></a>

<a id="dom-csstranslate-z"></a>

<a id="ref-for-Exposed②③"></a>

<a id="cssrotate"></a>

<a id="ref-for-csstransformcomponent⑨"></a>

<a id="ref-for-dom-cssrotate-cssrotate"></a>

<a id="ref-for-cssnumericvalue⑤⑤"></a>

<a id="dom-cssrotate-cssrotate-angle-angle"></a>

<a id="ref-for-dom-cssrotate-cssrotate-x-y-z-angle"></a>

<a id="ref-for-typedefdef-cssnumberish①⑥"></a>

<a id="dom-cssrotate-cssrotate-x-y-z-angle-x"></a>

<a id="ref-for-typedefdef-cssnumberish①⑦"></a>

<a id="dom-cssrotate-cssrotate-x-y-z-angle-y"></a>

<a id="ref-for-typedefdef-cssnumberish①⑧"></a>

<a id="dom-cssrotate-cssrotate-x-y-z-angle-z"></a>

<a id="ref-for-cssnumericvalue⑤⑥"></a>

<a id="dom-cssrotate-cssrotate-x-y-z-angle-angle"></a>

<a id="ref-for-typedefdef-cssnumberish①⑨"></a>

<a id="ref-for-dom-cssrotate-x"></a>

<a id="ref-for-typedefdef-cssnumberish②⓪"></a>

<a id="ref-for-dom-cssrotate-y"></a>

<a id="ref-for-typedefdef-cssnumberish②①"></a>

<a id="ref-for-dom-cssrotate-z"></a>

<a id="ref-for-cssnumericvalue⑤⑦"></a>

<a id="dom-cssrotate-angle"></a>

<a id="ref-for-Exposed②④"></a>

<a id="cssscale"></a>

<a id="ref-for-csstransformcomponent①⓪"></a>

<a id="ref-for-dom-cssscale-cssscale"></a>

<a id="ref-for-typedefdef-cssnumberish②②"></a>

<a id="dom-cssscale-cssscale-x-y-z-x"></a>

<a id="ref-for-typedefdef-cssnumberish②③"></a>

<a id="dom-cssscale-cssscale-x-y-z-y"></a>

<a id="ref-for-typedefdef-cssnumberish②④"></a>

<a id="dom-cssscale-cssscale-x-y-z-z"></a>

<a id="ref-for-typedefdef-cssnumberish②⑤"></a>

<a id="ref-for-dom-cssscale-x"></a>

<a id="ref-for-typedefdef-cssnumberish②⑥"></a>

<a id="ref-for-dom-cssscale-y"></a>

<a id="ref-for-typedefdef-cssnumberish②⑦"></a>

<a id="ref-for-dom-cssscale-z"></a>

<a id="ref-for-Exposed②⑤"></a>

<a id="cssskew"></a>

<a id="ref-for-csstransformcomponent①①"></a>

<a id="ref-for-dom-cssskew-cssskew"></a>

<a id="ref-for-cssnumericvalue⑤⑧"></a>

<a id="dom-cssskew-cssskew-ax-ay-ax"></a>

<a id="ref-for-cssnumericvalue⑤⑨"></a>

<a id="dom-cssskew-cssskew-ax-ay-ay"></a>

<a id="ref-for-cssnumericvalue⑥⓪"></a>

<a id="dom-cssskew-ax"></a>

<a id="ref-for-cssnumericvalue⑥①"></a>

<a id="dom-cssskew-ay"></a>

<a id="ref-for-Exposed②⑥"></a>

<a id="cssskewx"></a>

<a id="ref-for-csstransformcomponent①②"></a>

<a id="ref-for-dom-cssskewx-cssskewx"></a>

<a id="ref-for-cssnumericvalue⑥②"></a>

<a id="dom-cssskewx-cssskewx-ax-ax"></a>

<a id="ref-for-cssnumericvalue⑥③"></a>

<a id="dom-cssskewx-ax"></a>

<a id="ref-for-Exposed②⑦"></a>

<a id="cssskewy"></a>

<a id="ref-for-csstransformcomponent①③"></a>

<a id="ref-for-dom-cssskewy-cssskewy"></a>

<a id="ref-for-cssnumericvalue⑥④"></a>

<a id="dom-cssskewy-cssskewy-ay-ay"></a>

<a id="ref-for-cssnumericvalue⑥⑤"></a>

<a id="dom-cssskewy-ay"></a>

<a id="ref-for-Exposed②⑧"></a>

<a id="cssperspective"></a>

<a id="ref-for-csstransformcomponent①④"></a>

<a id="ref-for-dom-cssperspective-cssperspective"></a>

<a id="ref-for-typedefdef-cssperspectivevalue"></a>

<a id="dom-cssperspective-cssperspective-length-length"></a>

<a id="ref-for-typedefdef-cssperspectivevalue①"></a>

<a id="dom-cssperspective-length"></a>

<a id="ref-for-Exposed②⑨"></a>

<a id="cssmatrixcomponent"></a>

<a id="ref-for-csstransformcomponent①⑤"></a>

<a id="ref-for-dom-cssmatrixcomponent-cssmatrixcomponent"></a>

<a id="ref-for-dommatrixreadonly"></a>

<a id="dom-cssmatrixcomponent-cssmatrixcomponent-matrix-options-matrix"></a>

<a id="ref-for-dictdef-cssmatrixcomponentoptions"></a>

<a id="dom-cssmatrixcomponent-cssmatrixcomponent-matrix-options-options"></a>

<a id="ref-for-dommatrix④"></a>

<a id="dom-cssmatrixcomponent-matrix"></a>

<a id="dictdef-cssmatrixcomponentoptions"></a>

<a id="ref-for-idl-boolean④"></a>

<a id="dom-cssmatrixcomponentoptions-is2d"></a>

```text
typedef (CSSNumericValue or CSSKeywordish) CSSPerspectiveValue;

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSTransformComponent {
    stringifier;
    attribute boolean is2D;
    DOMMatrix toMatrix();
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSTranslate : CSSTransformComponent {
    constructor(CSSNumericValue x, CSSNumericValue y, optional CSSNumericValue z);
    attribute CSSNumericValue x;
    attribute CSSNumericValue y;
    attribute CSSNumericValue z;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSRotate : CSSTransformComponent {
    constructor(CSSNumericValue angle);
    constructor(CSSNumberish x, CSSNumberish y, CSSNumberish z, CSSNumericValue angle);
    attribute CSSNumberish x;
    attribute CSSNumberish y;
    attribute CSSNumberish z;
    attribute CSSNumericValue angle;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSScale : CSSTransformComponent {
    constructor(CSSNumberish x, CSSNumberish y, optional CSSNumberish z);
    attribute CSSNumberish x;
    attribute CSSNumberish y;
    attribute CSSNumberish z;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSSkew : CSSTransformComponent {
    constructor(CSSNumericValue ax, CSSNumericValue ay);
    attribute CSSNumericValue ax;
    attribute CSSNumericValue ay;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSSkewX : CSSTransformComponent {
    constructor(CSSNumericValue ax);
    attribute CSSNumericValue ax;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSSkewY : CSSTransformComponent {
    constructor(CSSNumericValue ay);
    attribute CSSNumericValue ay;
};

/* Note that skew(x,y) is *not* the same as skewX(x) skewY(y),
   thus the separate interfaces for all three. */

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSPerspective : CSSTransformComponent {
    constructor(CSSPerspectiveValue length);
    attribute CSSPerspectiveValue length;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMatrixComponent : CSSTransformComponent {
    constructor(DOMMatrixReadOnly matrix, optional CSSMatrixComponentOptions options = {});
    attribute DOMMatrix matrix;
};

dictionary CSSMatrixComponentOptions {
    boolean is2D;
};
```
<a id="ref-for-dom-csstranslate-z"></a>

The <a id="dom-csstransformcomponent-is2d"></a>`is2D` attribute indicates whether the transform is 2D or 3D. When it’s `true`, the attributes of the transform that are relevant to 3D transforms (such as the <code><a href="#dom-csstranslate-z">CSSTranslate.z</a></code> attribute) simply have no effect on the transform they represent.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This affects the serialization of the object, and concepts such as the object’s "equivalent 4x4 matrix".

<a id="ref-for-dom-csstransformcomponent-is2d②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> Design Considerations
>
> For legacy reasons, 2D and 3D transforms are distinct, even if they have identical effects; a translateZ(0px) has observable effects on a page, even tho it’s defined to be an identity transform, as the UA activates some 3D-based optimizations for the element.
>
> There were several possible ways to reflect this—​nullable 3D-related attributes, separate 2D and 3D interfaces, etc—​but we chose the current design (an author-flippable switch that dictates the behavior) because it allows authors to, in most circumstances, operate on transforms without having to care whether they’re 2D or 3D, but also prevents "accidentally" flipping a 2D transform into becoming 3D.

<a id="ref-for-csstransformcomponent①⑥"></a>

The <a id="dom-csstransformcomponent-tomatrix"></a>`toMatrix()` method of a <code><a href="#csstransformcomponent">CSSTransformComponent</a></code> <var>this</var> must, when called, perform the following steps:

1.  <a id="ref-for-dommatrix⑤"></a>

    <a id="ref-for-dom-dommatrixreadonly-is2d①"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d③"></a>

    Let <var>matrix</var> be a new <code><a href="https://www.w3.org/TR/geometry-1/#dommatrix">DOMMatrix</a></code> object, initialized to <var>this</var>’s equivalent 4x4 transform matrix, as defined in [CSS Transforms 1 §  14. Mathematical Description of Transform Functions](https://www.w3.org/TR/css-transforms-1/#mathematical-description), and with its <code><a href="https://www.w3.org/TR/geometry-1/#dom-dommatrixreadonly-is2d">is2D</a></code> internal slot set to the same value as <var>this</var>’s <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot.

    <a id="ref-for-dom-csstransformcomponent-is2d④"></a>

    <a id="ref-for-csstransformcomponent①⑦"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Recall that the <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> flag affects what transform, and thus what equivalent matrix, a <code><a href="#csstransformcomponent">CSSTransformComponent</a></code> represents.

    <a id="ref-for-px②"></a>

    <a id="ref-for-length-value③"></a>

    <a id="ref-for-compatible-units④"></a>

    <a id="ref-for-px③"></a>

    <a id="ref-for-relative-length"></a>

    <a id="ref-for-percentage"></a>

    <a id="ref-for-dfn-throw④③"></a>

    <a id="ref-for-exceptiondef-typeerror③⑥"></a>

    As the entries of such a matrix are defined relative to the [px](https://www.w3.org/TR/css-values-4/#px) unit, if any [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)s in <var>this</var> involved in generating the matrix are not [compatible units](https://www.w3.org/TR/css-values-4/#compatible-units) with [px](https://www.w3.org/TR/css-values-4/#px) (such as [relative lengths](https://www.w3.org/TR/css-values-4/#relative-length) or [percentages](https://www.w3.org/TR/css-values-4/#percentage)), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  Return <var>matrix</var>.

The <a id="dom-csstranslate-csstranslate"></a><code>CSSTranslate(<var>x</var>, <var>y</var>, <var>z</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-cssnumericvalue-match③"></a>

    <a id="ref-for-typedef-length-percentage①"></a>

    <a id="ref-for-dfn-throw④④"></a>

    <a id="ref-for-exceptiondef-typeerror③⑦"></a>

    If <var>x</var> or <var>y</var> don’t [match](#cssnumericvalue-match) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-cssnumericvalue-match④"></a>

    <a id="ref-for-length-value④"></a>

    <a id="ref-for-dfn-throw④⑤"></a>

    <a id="ref-for-exceptiondef-typeerror③⑧"></a>

    If <var>z</var> was passed, but doesn’t [match](#cssnumericvalue-match) [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-csstranslate"></a>

    <a id="ref-for-dom-csstranslate-x"></a>

    <a id="ref-for-dom-csstranslate-y"></a>

    Let <var>this</var> be a new <code><a href="#csstranslate">CSSTranslate</a></code> object, with its <code><a href="#dom-csstranslate-x">x</a></code> and <code><a href="#dom-csstranslate-y">y</a></code> internal slots set to <var>x</var> and <var>y</var>.

4.  <a id="ref-for-dom-csstranslate-z①"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d⑤"></a>

    If <var>z</var> was passed, set <var>this</var>’s <code><a href="#dom-csstranslate-z">z</a></code> internal slot to <var>z</var>, and set <var>this</var>’s <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot to `false`.

5.  <a id="ref-for-dom-csstranslate-z②"></a>

    <a id="ref-for-create-a-cssunitvalue-from-a-pair①"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d⑥"></a>

    If <var>z</var> was not passed, set <var>this</var>’s <code><a href="#dom-csstranslate-z">z</a></code> internal slot to a [new unit value](#create-a-cssunitvalue-from-a-pair) of `(0, "px")`, and set <var>this</var>’s <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot to `true`.

6.  Return <var>this</var>.

The <a id="dom-cssrotate-cssrotate"></a><code>CSSRotate(<var>angle</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-cssnumericvalue-match⑤"></a>

    <a id="ref-for-angle-value②"></a>

    <a id="ref-for-dfn-throw④⑥"></a>

    <a id="ref-for-exceptiondef-typeerror③⑨"></a>

    If <var>angle</var> doesn’t [match](#cssnumericvalue-match) [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-cssrotate"></a>

    <a id="ref-for-dom-cssrotate-angle"></a>

    <a id="ref-for-dom-cssrotate-x①"></a>

    <a id="ref-for-dom-cssrotate-y①"></a>

    <a id="ref-for-create-a-cssunitvalue-from-a-pair②"></a>

    <a id="ref-for-dom-cssrotate-z①"></a>

    <a id="ref-for-create-a-cssunitvalue-from-a-pair③"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d⑦"></a>

    Return a new <code><a href="#cssrotate">CSSRotate</a></code> with its <code><a href="#dom-cssrotate-angle">angle</a></code> internal slot set to <var>angle</var>, its <code><a href="#dom-cssrotate-x">x</a></code> and <code><a href="#dom-cssrotate-y">y</a></code> internal slots set to [new unit values](#create-a-cssunitvalue-from-a-pair) of `(0, "number")`, its <code><a href="#dom-cssrotate-z">z</a></code> internal slot set to a [new unit value](#create-a-cssunitvalue-from-a-pair) of `(1, "number")`, and its <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot set to `true`.

The <a id="dom-cssrotate-cssrotate-x-y-z-angle"></a><code>CSSRotate(<var>x</var>, <var>y</var>, <var>z</var>, <var>angle</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-cssnumericvalue-match⑥"></a>

    <a id="ref-for-angle-value③"></a>

    <a id="ref-for-dfn-throw④⑦"></a>

    <a id="ref-for-exceptiondef-typeerror④⓪"></a>

    If <var>angle</var> doesn’t [match](#cssnumericvalue-match) [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-rectify-a-numberish-value①⓪"></a>

    Let <var>x</var>, <var>y</var>, and <var>z</var> be replaced by the result of [rectifying a numberish value](#rectify-a-numberish-value).

3.  <a id="ref-for-cssnumericvalue-match⑦"></a>

    <a id="ref-for-number-value③"></a>

    <a id="ref-for-dfn-throw④⑧"></a>

    <a id="ref-for-exceptiondef-typeerror④①"></a>

    If <var>x</var>, <var>y</var>, or <var>z</var> don’t [match](#cssnumericvalue-match) [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

4.  <a id="ref-for-cssrotate①"></a>

    <a id="ref-for-dom-cssrotate-angle①"></a>

    <a id="ref-for-dom-cssrotate-x②"></a>

    <a id="ref-for-dom-cssrotate-y②"></a>

    <a id="ref-for-dom-cssrotate-z②"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d⑧"></a>

    Return a new <code><a href="#cssrotate">CSSRotate</a></code> with its <code><a href="#dom-cssrotate-angle">angle</a></code> internal slot set to <var>angle</var>, its <code><a href="#dom-cssrotate-x">x</a></code>, <code><a href="#dom-cssrotate-y">y</a></code>, <code><a href="#dom-cssrotate-z">z</a></code> internal slots set to <var>x</var>, <var>y</var>, and <var>z</var>, and its <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot set to `false`.

<a id="ref-for-rectify-a-numberish-value①①"></a>

The <a id="dom-cssrotate-x"></a>`x`, <a id="dom-cssrotate-y"></a>`y`, and <a id="dom-cssrotate-z"></a>`z` attributes must, on setting to a new value <var>val</var>, [rectify a numberish value](#rectify-a-numberish-value) from <var>val</var> and set the corresponding internal slot to the result of that.

The <a id="dom-cssscale-cssscale"></a><code>CSSScale(<var>x</var>, <var>y</var>, <var>z</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-rectify-a-numberish-value①②"></a>

    Let <var>x</var>, <var>y</var>, and <var>z</var> (if passed) be replaced by the result of [rectifying a numberish value](#rectify-a-numberish-value).

2.  <a id="ref-for-cssnumericvalue-match⑧"></a>

    <a id="ref-for-number-value④"></a>

    <a id="ref-for-dfn-throw④⑨"></a>

    <a id="ref-for-exceptiondef-typeerror④②"></a>

    If <var>x</var>, <var>y</var>, or <var>z</var> (if passed) don’t [match](#cssnumericvalue-match) [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-cssscale"></a>

    <a id="ref-for-dom-cssscale-x①"></a>

    <a id="ref-for-dom-cssscale-y①"></a>

    Let <var>this</var> be a new <code><a href="#cssscale">CSSScale</a></code> object, with its <code><a href="#dom-cssscale-x">x</a></code> and <code><a href="#dom-cssscale-y">y</a></code> internal slots set to <var>x</var> and <var>y</var>.

4.  <a id="ref-for-dom-cssscale-z①"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d⑨"></a>

    If <var>z</var> was passed, set <var>this</var>’s <code><a href="#dom-cssscale-z">z</a></code> internal slot to <var>z</var>, and set <var>this</var>’s <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot to `false`.

5.  <a id="ref-for-dom-cssscale-z②"></a>

    <a id="ref-for-create-a-cssunitvalue-from-a-pair④"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d①⓪"></a>

    If <var>z</var> was not passed, set <var>this</var>’s <code><a href="#dom-cssscale-z">z</a></code> internal slot to a [new unit value](#create-a-cssunitvalue-from-a-pair) of `(1, "number")`, and set <var>this</var>’s <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot to `true`.

6.  Return <var>this</var>.

<a id="ref-for-rectify-a-numberish-value①③"></a>

The <a id="dom-cssscale-x"></a>`x`, <a id="dom-cssscale-y"></a>`y`, and <a id="dom-cssscale-z"></a>`z` attributes must, on setting to a new value <var>val</var>, [rectify a numberish value](#rectify-a-numberish-value) from <var>val</var> and set the corresponding internal slot to the result of that.

The <a id="dom-cssskew-cssskew"></a><code>CSSSkew(<var>ax</var>, <var>ay</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-cssnumericvalue-match⑨"></a>

    <a id="ref-for-angle-value④"></a>

    <a id="ref-for-dfn-throw⑤⓪"></a>

    <a id="ref-for-exceptiondef-typeerror④③"></a>

    If <var>ax</var> or <var>ay</var> do not [match](#cssnumericvalue-match) [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-cssskew"></a>

    <a id="ref-for-dom-cssskew-ax"></a>

    <a id="ref-for-dom-cssskew-ay"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d①①"></a>

    Return a new <code><a href="#cssskew">CSSSkew</a></code> object with its <code><a href="#dom-cssskew-ax">ax</a></code> and <code><a href="#dom-cssskew-ay">ay</a></code> internal slots set to <var>ax</var> and <var>ay</var>, and its <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot set to `true`.

The <a id="dom-cssskewx-cssskewx"></a><code>CSSSkewX(<var>ax</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-cssnumericvalue-match①⓪"></a>

    <a id="ref-for-angle-value⑤"></a>

    <a id="ref-for-dfn-throw⑤①"></a>

    <a id="ref-for-exceptiondef-typeerror④④"></a>

    If <var>ax</var> does not [match](#cssnumericvalue-match) [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-cssskewx"></a>

    <a id="ref-for-dom-cssskewx-ax"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d①②"></a>

    Return a new <code><a href="#cssskewx">CSSSkewX</a></code> object with its <code><a href="#dom-cssskewx-ax">ax</a></code> internal slot set to <var>ax</var>, and its <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot set to `true`.

The <a id="dom-cssskewy-cssskewy"></a><code>CSSSkewY(<var>ay</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-cssnumericvalue-match①①"></a>

    <a id="ref-for-angle-value⑥"></a>

    <a id="ref-for-dfn-throw⑤②"></a>

    <a id="ref-for-exceptiondef-typeerror④⑤"></a>

    If <var>ay</var> does not [match](#cssnumericvalue-match) [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-cssskewy"></a>

    <a id="ref-for-dom-cssskewy-ay"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d①③"></a>

    Return a new <code><a href="#cssskewy">CSSSkewY</a></code> object with its <code><a href="#dom-cssskewy-ay">ay</a></code> internal slot set to <var>ay</var>, and its <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot set to `true`.

<a id="ref-for-cssskew①"></a>

<a id="ref-for-cssskewx①"></a>

<a id="ref-for-cssskewy①"></a>

The <a id="dom-cssskew-is2d"></a>`is2D` attribute of a <code><a href="#cssskew">CSSSkew</a></code>, <code><a href="#cssskewx">CSSSkewX</a></code>, or <code><a href="#cssskewy">CSSSkewY</a></code> object must, on setting, do nothing.

<a id="ref-for-funcdef-transform-skew"></a>

<a id="ref-for-funcdef-transform-skewx"></a>

<a id="ref-for-funcdef-transform-skewy"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [skew()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-skew), [skewX()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-skewx), and [skewY()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-skewy) functions always represent 2D transforms.

The <a id="dom-cssperspective-cssperspective"></a><code>CSSPerspective(<var>length</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-cssnumericvalue⑥⑥"></a>

    If <var>length</var> is a <code><a href="#cssnumericvalue">CSSNumericValue</a></code>:

    1.  <a id="ref-for-cssnumericvalue-match①②"></a>

        <a id="ref-for-length-value⑤"></a>

        <a id="ref-for-dfn-throw⑤③"></a>

        <a id="ref-for-exceptiondef-typeerror④⑥"></a>

        If <var>length</var> does not [match](#cssnumericvalue-match) [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

2.  <a id="ref-for-cssnumericvalue⑥⑦"></a>

    Otherwise (that is, if <var>length</var> is not a <code><a href="#cssnumericvalue">CSSNumericValue</a></code>):

    1.  <a id="ref-for-rectify-a-keywordish-value"></a>

        [Rectify a keywordish value](#rectify-a-keywordish-value) from <var>length</var>, then set <var>length</var> to the result’s value.

    2.  <a id="ref-for-ascii-case-insensitive"></a>

        <a id="ref-for-valdef-perspective-func-none"></a>

        <a id="ref-for-dfn-throw⑤④"></a>

        <a id="ref-for-exceptiondef-typeerror④⑦"></a>

        If <var>length</var> does not represent a value that is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the keyword [none](https://www.w3.org/TR/css-transforms-2/#valdef-perspective-func-none), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

3.  <a id="ref-for-cssperspective"></a>

    <a id="ref-for-dom-cssperspective-length"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d①④"></a>

    Return a new <code><a href="#cssperspective">CSSPerspective</a></code> object with its <code><a href="#dom-cssperspective-length">length</a></code> internal slot set to <var>length</var>, and its <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot set to `false`.

<a id="ref-for-cssperspective①"></a>

The <a id="dom-cssperspective-is2d"></a>`is2D` attribute of a <code><a href="#cssperspective">CSSPerspective</a></code> object must, on setting, do nothing.

<a id="ref-for-funcdef-perspective"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [perspective()](https://www.w3.org/TR/css-transforms-2/#funcdef-perspective) functions always represent 3D transforms.

The <a id="dom-cssmatrixcomponent-cssmatrixcomponent"></a><code>CSSMatrixComponent(<var>matrix</var>, <var>options</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-cssmatrixcomponent"></a>

    <a id="ref-for-dom-cssmatrixcomponent-matrix"></a>

    Let <var>this</var> be a new <code><a href="#cssmatrixcomponent">CSSMatrixComponent</a></code> object with its <code><a href="#dom-cssmatrixcomponent-matrix">matrix</a></code> internal slot set to <var>matrix</var>.

2.  <a id="ref-for-dom-cssmatrixcomponentoptions-is2d"></a>

    <a id="ref-for-dom-csstransformvalue-is2d①"></a>

    If <var>options</var> was passed and has a <code><a href="#dom-cssmatrixcomponentoptions-is2d">is2D</a></code> field, set <var>this</var>’s <code><a href="#dom-csstransformvalue-is2d">is2D</a></code> internal slot to the value of that field.

3.  <a id="ref-for-dom-csstransformvalue-is2d②"></a>

    <a id="ref-for-dom-dommatrixreadonly-is2d②"></a>

    Otherwise, set <var>this</var>’s <code><a href="#dom-csstransformvalue-is2d">is2D</a></code> internal slot to the value of <var>matrix</var>’s <code><a href="https://www.w3.org/TR/geometry-1/#dom-dommatrixreadonly-is2d">is2D</a></code> internal slot.

4.  Return <var>this</var>.

<a id="ref-for-csstransformcomponent①⑧"></a>

<a id="ref-for-csstranslate①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Each <code><a href="#csstransformcomponent">CSSTransformComponent</a></code> can correspond to one of a number of underlying transform functions. For example, a <code><a href="#csstranslate">CSSTranslate</a></code> with an x value of 10px and y &#x26; z values of 0px could represent any of the following:
>
> - translate(10px)
>
> - translate(10px, 0)
>
> - translateX(10px)
>
> - translate3d(10px, 0, 0)
>
> <a id="ref-for-dom-csstransformcomponent-is2d①⑤"></a>
>
> When stringified, however, it will always print out either translate(10px, 0px) or translate3d(10px, 0px, 0px), depending on whether its <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot is `true` or `false`, respectively.

<a id="ref-for-cssimagevalue"></a>

### <a id="imagevalue-objects"></a>4.5. <code><a href="#cssimagevalue">CSSImageValue</a></code> objects

<a id="ref-for-Exposed③⓪"></a>

<a id="cssimagevalue"></a>

<a id="ref-for-cssstylevalue②⑦"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSImageValue : CSSStyleValue {
};
```
<a id="ref-for-cssimagevalue①"></a>

<a id="ref-for-typedef-image"></a>

<a id="ref-for-propdef-background-image②"></a>

<a id="ref-for-propdef-list-style-image"></a>

<a id="ref-for-propdef-border-image-source"></a>

<code><a href="#cssimagevalue">CSSImageValue</a></code> objects represent values for properties that take [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) productions, for example [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image), [list-style-image](https://www.w3.org/TR/css-lists-3/#propdef-list-style-image), and [border-image-source](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-source).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This object is intentionally opaque, and exposes no details of what kind of image it contains, or any aspect of the image. This is because having <em>something</em> to represent images is necessary for Custom Paint, but there are sufficient complexities in getting URL-handling and loading specified firmly that it’s not realistically possible to specify in the timeline of this specification. This will be expanded on in future levels.

<a id="ref-for-cssimagevalue②"></a>

<a id="ref-for-typedef-image①"></a>

<a id="ref-for-funcdef-url"></a>

<a id="ref-for-funcdef-image"></a>

If a <code><a href="#cssimagevalue">CSSImageValue</a></code> object represents an [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) that involves a URL (such as [url()](https://www.w3.org/TR/css-values-4/#funcdef-url) or [image()](https://www.w3.org/TR/css-images-4/#funcdef-image)), the handling of such values is identical to how CSS currently handles them. In particular, resolving relative URLs or fragment URLs has the same behavior as in normal CSS.

<a id="ref-for-the-style-element"></a>

<a id="ref-for-attr-style"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e5256ca1"></a> For example, relative URLs are resolved against the URL of the stylesheet they’re within (or the document’s URL, if they’re specified in a <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-style-element">style</a></code> element or <code><a href="https://html.spec.whatwg.org/multipage/dom.html#attr-style">style</a></code> attribute). This resolution doesn’t happen eagerly at parse-time, but at some currently-unspecified point during value computation.
>
> <a id="ref-for-propdef-background-image③"></a>
>
> Thus, if an element’s style is set to [background-image: url(foo);](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image), and that specified value is extracted via the Typed OM and then set on an element in a different document, both the source and destination elements will resolve the URL differently, as they provide different base URLs.
>
> <a id="ref-for-computed-value④"></a>
>
> <a id="ref-for-dom-element-computedstylemap③"></a>
>
> On the other hand, if the extracted value was a [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) (from <code><a href="#dom-element-computedstylemap">computedStyleMap()</a></code>), then it would already be resolved to an absolute URL, and thus would act identically no matter where you later set it to. (Unless it was a fragment URL, which CSS treats differently and never fully resolves, so it always resolves against the current document.)

<a id="ref-for-csscolorvalue"></a>

### <a id="colorvalue-objects"></a>4.6. <code><a href="#csscolorvalue">CSSColorValue</a></code> objects

<a id="ref-for-csscolorvalue①"></a>

<a id="ref-for-typedef-color"></a>

<code><a href="#csscolorvalue">CSSColorValue</a></code> objects represent [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) values. It is an abstract superclass, with the subclasses representing individual CSS color functions.

<a id="ref-for-Exposed③①"></a>

<a id="csscolorvalue"></a>

<a id="ref-for-cssstylevalue②⑧"></a>

<a id="ref-for-Exposed③②"></a>

<a id="ref-for-csscolorvalue②"></a>

<a id="ref-for-cssstylevalue②⑨"></a>

<a id="ref-for-dom-csscolorvalue-parse"></a>

<a id="ref-for-idl-USVString②⑥"></a>

<a id="dom-csscolorvalue-parse-csstext-csstext"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSColorValue : CSSStyleValue {
    [Exposed=Window] static (CSSColorValue or CSSStyleValue) parse(USVString cssText);
};
```
The <a id="dom-csscolorvalue-parse"></a><code>parse(<var>cssText</var>)</code> method, when called, must perform the following steps:

1.  <a id="ref-for-css-parse-something-according-to-a-css-grammar①"></a>

    <a id="ref-for-typedef-color①"></a>

    <a id="ref-for-dfn-throw⑤⑤"></a>

    <a id="ref-for-syntaxerror⑥"></a>

    [Parse](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>cssText</var> as a [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) and let <var>result</var> be the result. If <var>result</var> is a syntax error, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code> and abort this algorithm.

2.  <a id="ref-for-reify-a-color-value"></a>

    [Reify a color value](#reify-a-color-value) from <var>result</var>, and return the result.

===============

<a id="ref-for-csscolorvalue③"></a>

Several IDL types are defined to be used in <code><a href="#csscolorvalue">CSSColorValue</a></code>s:

<a id="ref-for-typedefdef-cssnumberish②⑧"></a>

<a id="ref-for-typedefdef-csskeywordish①"></a>

<a id="typedefdef-csscolorrgbcomp"></a>

<a id="ref-for-typedefdef-cssnumberish②⑨"></a>

<a id="ref-for-typedefdef-csskeywordish②"></a>

<a id="typedefdef-csscolorpercent"></a>

<a id="ref-for-typedefdef-cssnumberish③⓪"></a>

<a id="ref-for-typedefdef-csskeywordish③"></a>

<a id="typedefdef-csscolornumber"></a>

<a id="ref-for-typedefdef-cssnumberish③①"></a>

<a id="ref-for-typedefdef-csskeywordish④"></a>

<a id="typedefdef-csscolorangle"></a>

```text
typedef (CSSNumberish or CSSKeywordish) CSSColorRGBComp;
typedef (CSSNumberish or CSSKeywordish) CSSColorPercent;
typedef (CSSNumberish or CSSKeywordish) CSSColorNumber;
typedef (CSSNumberish or CSSKeywordish) CSSColorAngle;
```
<a id="ref-for-number-value⑤"></a>

<a id="ref-for-percentage-value⑤"></a>

<a id="ref-for-percentage-value⑥"></a>

<a id="ref-for-number-value⑥"></a>

<a id="ref-for-angle-value⑦"></a>

All of these types are the same in terms of type signature, but they represent distinct values: `CSSColorRGBComp` represents a value that is, canonically, either a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), or the keyword none; `CSSColorPercent` represents a value that is, canonically, either a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) or the keyword none; `CSSColorNumber` represents a value that is, canonically, either a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) or the keyword none; `CSSColorAngle` represents a value that is, canonically, either an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) or the keyword none.

<a id="ref-for-idl-double⑥⑧"></a>

<a id="ref-for-cssnumericvalue⑥⑧"></a>

Their corresponding rectification algorithms also all have distinct behaviors for translating a <code><a href="https://webidl.spec.whatwg.org/#idl-double">double</a></code> value into a <code><a href="#cssnumericvalue">CSSNumericValue</a></code>.

To <a id="rectify-a-csscolorrgbcomp"></a>rectify a CSSColorRGBComp <var>val</var>:

1.  <a id="ref-for-idl-double⑥⑨"></a>

    <a id="ref-for-create-a-cssunitvalue-from-a-pair⑤"></a>

    If <var>val</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-double">double</a></code>, replace it with a [new unit value](#create-a-cssunitvalue-from-a-pair) from (<var>val</var>\*100, "percent").

2.  <a id="ref-for-idl-DOMString②"></a>

    <a id="ref-for-rectify-a-keywordish-value①"></a>

    If <var>val</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-DOMString">DOMString</a></code>, replace it with the result of [rectifying a keywordish value](#rectify-a-keywordish-value) from <var>val</var>.

3.  <a id="ref-for-cssnumericvalue⑥⑨"></a>

    <a id="ref-for-cssnumericvalue-match①③"></a>

    <a id="ref-for-number-value⑦"></a>

    <a id="ref-for-percentage-value⑦"></a>

    If <var>val</var> is a <code><a href="#cssnumericvalue">CSSNumericValue</a></code>, and it [matches](#cssnumericvalue-match) [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), return <var>val</var>.

4.  <a id="ref-for-csskeywordvalue①⓪"></a>

    <a id="ref-for-dom-csskeywordvalue-value⑤"></a>

    <a id="ref-for-ascii-case-insensitive①"></a>

    If <var>val</var> is a <code><a href="#csskeywordvalue">CSSKeywordValue</a></code>, and its <code><a href="#dom-csskeywordvalue-value">value</a></code> internal slot is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for `"none"`, return <var>val</var>.

5.  <a id="ref-for-syntaxerror⑦"></a>

    Throw a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code>.

To <a id="rectify-a-csscolorpercent"></a>rectify a CSSColorPercent <var>val</var>:

1.  <a id="ref-for-idl-double⑦⓪"></a>

    <a id="ref-for-create-a-cssunitvalue-from-a-pair⑥"></a>

    If <var>val</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-double">double</a></code>, replace it with a [new unit value](#create-a-cssunitvalue-from-a-pair) from (<var>val</var>\*100, "percent").

2.  <a id="ref-for-idl-DOMString③"></a>

    <a id="ref-for-rectify-a-keywordish-value②"></a>

    If <var>val</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-DOMString">DOMString</a></code>, replace it with the result of [rectifying a keywordish value](#rectify-a-keywordish-value) from <var>val</var>.

3.  <a id="ref-for-cssnumericvalue⑦⓪"></a>

    <a id="ref-for-cssnumericvalue-match①④"></a>

    <a id="ref-for-percentage-value⑧"></a>

    If <var>val</var> is a <code><a href="#cssnumericvalue">CSSNumericValue</a></code>, and it [matches](#cssnumericvalue-match) [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), return <var>val</var>.

4.  <a id="ref-for-csskeywordvalue①①"></a>

    <a id="ref-for-dom-csskeywordvalue-value⑥"></a>

    <a id="ref-for-ascii-case-insensitive②"></a>

    If <var>val</var> is a <code><a href="#csskeywordvalue">CSSKeywordValue</a></code>, and its <code><a href="#dom-csskeywordvalue-value">value</a></code> internal slot is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for `"none"`, return <var>val</var>.

5.  <a id="ref-for-syntaxerror⑧"></a>

    Throw a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code>.

To <a id="rectify-a-csscolornumber"></a>rectify a CSSColorNumber <var>val</var>:

1.  <a id="ref-for-idl-double⑦①"></a>

    <a id="ref-for-create-a-cssunitvalue-from-a-pair⑦"></a>

    If <var>val</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-double">double</a></code>, replace it with a [new unit value](#create-a-cssunitvalue-from-a-pair) from (<var>val</var>, "number").

2.  <a id="ref-for-idl-DOMString④"></a>

    <a id="ref-for-rectify-a-keywordish-value③"></a>

    If <var>val</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-DOMString">DOMString</a></code>, replace it with the result of [rectifying a keywordish value](#rectify-a-keywordish-value) from <var>val</var>.

3.  <a id="ref-for-cssnumericvalue⑦①"></a>

    <a id="ref-for-cssnumericvalue-match①⑤"></a>

    <a id="ref-for-number-value⑧"></a>

    If <var>val</var> is a <code><a href="#cssnumericvalue">CSSNumericValue</a></code>, and it [matches](#cssnumericvalue-match) [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), return <var>val</var>.

4.  <a id="ref-for-csskeywordvalue①②"></a>

    <a id="ref-for-dom-csskeywordvalue-value⑦"></a>

    <a id="ref-for-ascii-case-insensitive③"></a>

    If <var>val</var> is a <code><a href="#csskeywordvalue">CSSKeywordValue</a></code>, and its <code><a href="#dom-csskeywordvalue-value">value</a></code> internal slot is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for `"none"`, return <var>val</var>.

5.  <a id="ref-for-syntaxerror⑨"></a>

    Throw a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code>.

To <a id="rectify-a-csscolorangle"></a>rectify a CSSColorAngle <var>val</var>:

1.  <a id="ref-for-idl-double⑦②"></a>

    <a id="ref-for-create-a-cssunitvalue-from-a-pair⑧"></a>

    If <var>val</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-double">double</a></code>, replace it with a [new unit value](#create-a-cssunitvalue-from-a-pair) from (<var>val</var>, "deg").

2.  <a id="ref-for-idl-DOMString⑤"></a>

    <a id="ref-for-rectify-a-keywordish-value④"></a>

    If <var>val</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-DOMString">DOMString</a></code>, replace it with the result of [rectifying a keywordish value](#rectify-a-keywordish-value) from <var>val</var>.

3.  <a id="ref-for-cssnumericvalue⑦②"></a>

    <a id="ref-for-cssnumericvalue-match①⑥"></a>

    <a id="ref-for-angle-value⑧"></a>

    If <var>val</var> is a <code><a href="#cssnumericvalue">CSSNumericValue</a></code>, and it [matches](#cssnumericvalue-match) [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), return <var>val</var>.

4.  <a id="ref-for-csskeywordvalue①③"></a>

    <a id="ref-for-dom-csskeywordvalue-value⑧"></a>

    <a id="ref-for-ascii-case-insensitive④"></a>

    If <var>val</var> is a <code><a href="#csskeywordvalue">CSSKeywordValue</a></code>, and its <code><a href="#dom-csskeywordvalue-value">value</a></code> internal slot is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for `"none"`, return <var>val</var>.

5.  <a id="ref-for-exceptiondef-typeerror④⑧"></a>

    Throw a <code><a href="https://webidl.spec.whatwg.org/#exceptiondef-typeerror">TypeError</a></code>.

===============

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-0e2e471e"></a> TODO add stringifiers

<a id="ref-for-Exposed③③"></a>

<a id="cssrgb"></a>

<a id="ref-for-csscolorvalue④"></a>

<a id="dom-cssrgb-cssrgb"></a>

<a id="ref-for-typedefdef-csscolorrgbcomp"></a>

<a id="dom-cssrgb-cssrgb-r-g-b-alpha-r"></a>

<a id="ref-for-typedefdef-csscolorrgbcomp①"></a>

<a id="dom-cssrgb-cssrgb-r-g-b-alpha-g"></a>

<a id="ref-for-typedefdef-csscolorrgbcomp②"></a>

<a id="dom-cssrgb-cssrgb-r-g-b-alpha-b"></a>

<a id="ref-for-typedefdef-csscolorpercent"></a>

<a id="dom-cssrgb-cssrgb-r-g-b-alpha-alpha"></a>

<a id="ref-for-typedefdef-csscolorrgbcomp③"></a>

<a id="ref-for-dom-cssrgb-r"></a>

<a id="ref-for-typedefdef-csscolorrgbcomp④"></a>

<a id="ref-for-dom-cssrgb-g"></a>

<a id="ref-for-typedefdef-csscolorrgbcomp⑤"></a>

<a id="ref-for-dom-cssrgb-b"></a>

<a id="ref-for-typedefdef-csscolorpercent①"></a>

<a id="ref-for-dom-cssrgb-alpha"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSRGB : CSSColorValue {
    constructor(CSSColorRGBComp r, CSSColorRGBComp g, CSSColorRGBComp b, optional CSSColorPercent alpha = 1);
    attribute CSSColorRGBComp r;
    attribute CSSColorRGBComp g;
    attribute CSSColorRGBComp b;
    attribute CSSColorPercent alpha;
};
```
<a id="ref-for-cssrgb"></a>

<a id="ref-for-funcdef-rgb"></a>

<a id="ref-for-funcdef-rgba"></a>

The <code><a href="#cssrgb">CSSRGB</a></code> class represents the CSS [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb)/[rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba) functions.

The <a id="dom-cssrgb-cssrgb-r-g-b-optional-alpha"></a><code>CSSRGB(<var>r</var>, <var>g</var>, <var>b</var>, optional <var>alpha</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-rectify-a-csscolorrgbcomp"></a>

    <a id="ref-for-rectify-a-csscolorpercent"></a>

    Let <var>r</var>, <var>g</var>, <var>b</var> be replaced by the result of [rectifying a CSSColorRGBComp](#rectify-a-csscolorrgbcomp) from each of them. Let <var>alpha</var> be replaced by the result of [rectifying a CSSColorPercent](#rectify-a-csscolorpercent) from it.

2.  <a id="ref-for-cssrgb①"></a>

    <a id="ref-for-dom-cssrgb-r①"></a>

    <a id="ref-for-dom-cssrgb-g①"></a>

    <a id="ref-for-dom-cssrgb-b①"></a>

    <a id="ref-for-dom-cssrgb-alpha①"></a>

    Return a new <code><a href="#cssrgb">CSSRGB</a></code> with its <code><a href="#dom-cssrgb-r">r</a></code>, <code><a href="#dom-cssrgb-g">g</a></code>, <code><a href="#dom-cssrgb-b">b</a></code>, and <code><a href="#dom-cssrgb-alpha">alpha</a></code> internal slots set to <var>r</var>, <var>g</var>, <var>b</var>, and <var>alpha</var>.

<a id="ref-for-cssrgb②"></a>

<a id="ref-for-rectify-a-csscolorrgbcomp①"></a>

The <a id="dom-cssrgb-r"></a>`r`, <a id="dom-cssrgb-g"></a>`g`, and <a id="dom-cssrgb-b"></a>`b` attributes of a <code><a href="#cssrgb">CSSRGB</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorRGBComp](#rectify-a-csscolorrgbcomp) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-cssrgb③"></a>

<a id="ref-for-rectify-a-csscolorpercent①"></a>

The <a id="dom-cssrgb-alpha"></a>`alpha` attribute of a <code><a href="#cssrgb">CSSRGB</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorPercent](#rectify-a-csscolorpercent) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-Exposed③④"></a>

<a id="csshsl"></a>

<a id="ref-for-csscolorvalue⑤"></a>

<a id="dom-csshsl-csshsl"></a>

<a id="ref-for-typedefdef-csscolorangle"></a>

<a id="dom-csshsl-csshsl-h-s-l-alpha-h"></a>

<a id="ref-for-typedefdef-csscolorpercent②"></a>

<a id="dom-csshsl-csshsl-h-s-l-alpha-s"></a>

<a id="ref-for-typedefdef-csscolorpercent③"></a>

<a id="dom-csshsl-csshsl-h-s-l-alpha-l"></a>

<a id="ref-for-typedefdef-csscolorpercent④"></a>

<a id="dom-csshsl-csshsl-h-s-l-alpha-alpha"></a>

<a id="ref-for-typedefdef-csscolorangle①"></a>

<a id="ref-for-dom-csshsl-h"></a>

<a id="ref-for-typedefdef-csscolorpercent⑤"></a>

<a id="ref-for-dom-csshsl-s"></a>

<a id="ref-for-typedefdef-csscolorpercent⑥"></a>

<a id="ref-for-dom-csshsl-l"></a>

<a id="ref-for-typedefdef-csscolorpercent⑦"></a>

<a id="ref-for-dom-csshsl-alpha"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSHSL : CSSColorValue {
    constructor(CSSColorAngle h, CSSColorPercent s, CSSColorPercent l, optional CSSColorPercent alpha = 1);
    attribute CSSColorAngle h;
    attribute CSSColorPercent s;
    attribute CSSColorPercent l;
    attribute CSSColorPercent alpha;
};
```
<a id="ref-for-csshsl"></a>

<a id="ref-for-funcdef-hsl"></a>

<a id="ref-for-funcdef-hsla"></a>

The <code><a href="#csshsl">CSSHSL</a></code> class represents the CSS [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl)/[hsla()](https://www.w3.org/TR/css-color-4/#funcdef-hsla) functions.

The <a id="dom-csshsl-csshsl-h-s-l-optional-alpha"></a><code>CSSHSL(<var>h</var>, <var>s</var>, <var>l</var>, optional <var>alpha</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-rectify-a-csscolorangle"></a>

    <a id="ref-for-rectify-a-csscolorpercent②"></a>

    Let <var>h</var> be replaced by the result of [rectifying a CSSColorAngle](#rectify-a-csscolorangle) from it. Let <var>s</var>, <var>l</var>, and <var>alpha</var> be replaced by the result of [rectifying a CSSColorPercent](#rectify-a-csscolorpercent) from each of them.

2.  <a id="ref-for-csshsl①"></a>

    <a id="ref-for-dom-csshsl-h①"></a>

    <a id="ref-for-dom-csshsl-s①"></a>

    <a id="ref-for-dom-csshsl-l①"></a>

    <a id="ref-for-dom-csshsl-alpha①"></a>

    Return a new <code><a href="#csshsl">CSSHSL</a></code> with its <code><a href="#dom-csshsl-h">h</a></code>, <code><a href="#dom-csshsl-s">s</a></code>, <code><a href="#dom-csshsl-l">l</a></code>, and <code><a href="#dom-csshsl-alpha">alpha</a></code> internal slots set to <var>h</var>, <var>s</var>, <var>l</var>, and <var>alpha</var>.

<a id="ref-for-csshsl②"></a>

<a id="ref-for-rectify-a-csscolorangle①"></a>

The <a id="dom-csshsl-h"></a>`h` attribute of a <code><a href="#csshsl">CSSHSL</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorAngle](#rectify-a-csscolorangle) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-csshsl③"></a>

<a id="ref-for-rectify-a-csscolorpercent③"></a>

The <a id="dom-csshsl-s"></a>`s`, <a id="dom-csshsl-l"></a>`l`, and <a id="dom-csshsl-alpha"></a>`alpha` attributes of a <code><a href="#csshsl">CSSHSL</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorPercent](#rectify-a-csscolorpercent) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-Exposed③⑤"></a>

<a id="csshwb"></a>

<a id="ref-for-csscolorvalue⑥"></a>

<a id="dom-csshwb-csshwb"></a>

<a id="ref-for-cssnumericvalue⑦③"></a>

<a id="dom-csshwb-csshwb-h-w-b-alpha-h"></a>

<a id="ref-for-typedefdef-cssnumberish③②"></a>

<a id="dom-csshwb-csshwb-h-w-b-alpha-w"></a>

<a id="ref-for-typedefdef-cssnumberish③③"></a>

<a id="dom-csshwb-csshwb-h-w-b-alpha-b"></a>

<a id="ref-for-typedefdef-cssnumberish③④"></a>

<a id="dom-csshwb-csshwb-h-w-b-alpha-alpha"></a>

<a id="ref-for-cssnumericvalue⑦④"></a>

<a id="ref-for-dom-csshwb-h"></a>

<a id="ref-for-typedefdef-cssnumberish③⑤"></a>

<a id="ref-for-dom-csshwb-w"></a>

<a id="ref-for-typedefdef-cssnumberish③⑥"></a>

<a id="ref-for-dom-csshwb-b"></a>

<a id="ref-for-typedefdef-cssnumberish③⑦"></a>

<a id="ref-for-dom-csshwb-alpha"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSHWB : CSSColorValue {
    constructor(CSSNumericValue h, CSSNumberish w, CSSNumberish b, optional CSSNumberish alpha = 1);
    attribute CSSNumericValue h;
    attribute CSSNumberish w;
    attribute CSSNumberish b;
    attribute CSSNumberish alpha;
};
```
<a id="ref-for-csshwb"></a>

<a id="ref-for-funcdef-hwb"></a>

The <code><a href="#csshwb">CSSHWB</a></code> class represents the CSS [hwb()](https://www.w3.org/TR/css-color-5/#funcdef-hwb) function.

The <a id="dom-csshwb-csshwb-h-w-b-optional-alpha"></a><code>CSSHWB(<var>h</var>, <var>w</var>, <var>b</var>, optional <var>alpha</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-rectify-a-csscolorangle②"></a>

    <a id="ref-for-rectify-a-csscolorpercent④"></a>

    Let <var>h</var> be replaced by the result of [rectifying a CSSColorAngle](#rectify-a-csscolorangle) from it. Let <var>w</var>, <var>b</var>, and <var>alpha</var> be replaced by the result of [rectifying a CSSColorPercent](#rectify-a-csscolorpercent) from each of them.

2.  <a id="ref-for-csshwb①"></a>

    <a id="ref-for-dom-csshwb-h①"></a>

    <a id="ref-for-dom-csshwb-w①"></a>

    <a id="ref-for-dom-csshwb-b①"></a>

    <a id="ref-for-dom-csshwb-alpha①"></a>

    Return a new <code><a href="#csshwb">CSSHWB</a></code> with its <code><a href="#dom-csshwb-h">h</a></code>, <code><a href="#dom-csshwb-w">w</a></code>, <code><a href="#dom-csshwb-b">b</a></code>, and <code><a href="#dom-csshwb-alpha">alpha</a></code> internal slots set to <var>h</var>, <var>w</var>, <var>b</var>, and <var>alpha</var>.

<a id="ref-for-csshwb②"></a>

<a id="ref-for-rectify-a-csscolorangle③"></a>

The <a id="dom-csshwb-h"></a>`h` attribute of a <code><a href="#csshwb">CSSHWB</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorAngle](#rectify-a-csscolorangle) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-csshwb③"></a>

<a id="ref-for-rectify-a-csscolorpercent⑤"></a>

The <a id="dom-csshwb-w"></a>`w`, <a id="dom-csshwb-b"></a>`b`, and <a id="dom-csshwb-alpha"></a>`alpha` attributes of a <code><a href="#csshwb">CSSHWB</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorPercent](#rectify-a-csscolorpercent) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-Exposed③⑥"></a>

<a id="csslab"></a>

<a id="ref-for-csscolorvalue⑦"></a>

<a id="dom-csslab-csslab"></a>

<a id="ref-for-typedefdef-csscolorpercent⑧"></a>

<a id="dom-csslab-csslab-l-a-b-alpha-l"></a>

<a id="ref-for-typedefdef-csscolornumber"></a>

<a id="dom-csslab-csslab-l-a-b-alpha-a"></a>

<a id="ref-for-typedefdef-csscolornumber①"></a>

<a id="dom-csslab-csslab-l-a-b-alpha-b"></a>

<a id="ref-for-typedefdef-csscolorpercent⑨"></a>

<a id="dom-csslab-csslab-l-a-b-alpha-alpha"></a>

<a id="ref-for-typedefdef-csscolorpercent①⓪"></a>

<a id="ref-for-dom-csslab-l"></a>

<a id="ref-for-typedefdef-csscolornumber②"></a>

<a id="ref-for-dom-csslab-a"></a>

<a id="ref-for-typedefdef-csscolornumber③"></a>

<a id="ref-for-dom-csslab-b"></a>

<a id="ref-for-typedefdef-csscolorpercent①①"></a>

<a id="ref-for-dom-csslab-alpha"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSLab : CSSColorValue {
    constructor(CSSColorPercent l, CSSColorNumber a, CSSColorNumber b, optional CSSColorPercent alpha = 1);
    attribute CSSColorPercent l;
    attribute CSSColorNumber a;
    attribute CSSColorNumber b;
    attribute CSSColorPercent alpha;
};
```
<a id="ref-for-csslab"></a>

<a id="ref-for-funcdef-lab"></a>

The <code><a href="#csslab">CSSLab</a></code> class represents the CSS [lab()](https://www.w3.org/TR/css-color-5/#funcdef-lab) function.

The <a id="dom-csslab-csslab-l-a-b-optional-alpha"></a><code>CSSLab(<var>l</var>, <var>a</var>, <var>b</var>, optional <var>alpha</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-rectify-a-csscolornumber"></a>

    <a id="ref-for-rectify-a-csscolorpercent⑥"></a>

    Let <var>a</var> and <var>b</var> be replaced by the result of [rectifying a CSSColorNumber](#rectify-a-csscolornumber) from each of them. Let <var>l</var> and <var>alpha</var> be replaced by the result of [rectifying a CSSColorPercent](#rectify-a-csscolorpercent) from each of them.

2.  <a id="ref-for-csslab①"></a>

    <a id="ref-for-dom-csslab-l①"></a>

    <a id="ref-for-dom-csslab-a①"></a>

    <a id="ref-for-dom-csslab-b①"></a>

    <a id="ref-for-dom-csslab-alpha①"></a>

    Return a new <code><a href="#csslab">CSSLab</a></code> with its <code><a href="#dom-csslab-l">l</a></code>, <code><a href="#dom-csslab-a">a</a></code>, <code><a href="#dom-csslab-b">b</a></code>, and <code><a href="#dom-csslab-alpha">alpha</a></code> internal slots set to <var>l</var>, <var>a</var>, <var>b</var>, and <var>alpha</var>.

<a id="ref-for-csslab②"></a>

<a id="ref-for-rectify-a-csscolorpercent⑦"></a>

The <a id="dom-csslab-l"></a>`l` and <a id="dom-csslab-alpha"></a>`alpha` attributes of a <code><a href="#csslab">CSSLab</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorPercent](#rectify-a-csscolorpercent) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-csslab③"></a>

<a id="ref-for-rectify-a-csscolornumber①"></a>

The <a id="dom-csslab-a"></a>`a`, and <a id="dom-csslab-b"></a>`b` attributes of a <code><a href="#csslab">CSSLab</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorNumber](#rectify-a-csscolornumber) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-Exposed③⑦"></a>

<a id="csslch"></a>

<a id="ref-for-csscolorvalue⑧"></a>

<a id="dom-csslch-csslch"></a>

<a id="ref-for-typedefdef-csscolorpercent①②"></a>

<a id="dom-csslch-csslch-l-c-h-alpha-l"></a>

<a id="ref-for-typedefdef-csscolorpercent①③"></a>

<a id="dom-csslch-csslch-l-c-h-alpha-c"></a>

<a id="ref-for-typedefdef-csscolorangle②"></a>

<a id="dom-csslch-csslch-l-c-h-alpha-h"></a>

<a id="ref-for-typedefdef-csscolorpercent①④"></a>

<a id="dom-csslch-csslch-l-c-h-alpha-alpha"></a>

<a id="ref-for-typedefdef-csscolorpercent①⑤"></a>

<a id="ref-for-dom-csslch-l"></a>

<a id="ref-for-typedefdef-csscolorpercent①⑥"></a>

<a id="ref-for-dom-csslch-c"></a>

<a id="ref-for-typedefdef-csscolorangle③"></a>

<a id="ref-for-dom-csslch-h"></a>

<a id="ref-for-typedefdef-csscolorpercent①⑦"></a>

<a id="ref-for-dom-csslch-alpha"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSLCH : CSSColorValue {
    constructor(CSSColorPercent l, CSSColorPercent c, CSSColorAngle h, optional CSSColorPercent alpha = 1);
    attribute CSSColorPercent l;
    attribute CSSColorPercent c;
    attribute CSSColorAngle h;
    attribute CSSColorPercent alpha;
};
```
<a id="ref-for-csslch"></a>

<a id="ref-for-funcdef-lch"></a>

The <code><a href="#csslch">CSSLCH</a></code> class represents the CSS [lch()](https://www.w3.org/TR/css-color-5/#funcdef-lch) function.

The <a id="dom-csslch-csslch-l-c-h-optional-alpha"></a><code>CSSLCH(<var>l</var>, <var>c</var>, <var>h</var>, optional <var>alpha</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-rectify-a-csscolorangle④"></a>

    <a id="ref-for-rectify-a-csscolorpercent⑧"></a>

    Let <var>h</var> be replaced by the result of [rectifying a CSSColorAngle](#rectify-a-csscolorangle) from it. Let <var>l</var>, <var>c</var>, and <var>alpha</var> be replaced by the result of [rectifying a CSSColorPercent](#rectify-a-csscolorpercent) from each of them.

2.  <a id="ref-for-csslch①"></a>

    <a id="ref-for-dom-csslch-l①"></a>

    <a id="ref-for-dom-csslch-c①"></a>

    <a id="ref-for-dom-csslch-h①"></a>

    <a id="ref-for-dom-csslch-alpha①"></a>

    Return a new <code><a href="#csslch">CSSLCH</a></code> with its <code><a href="#dom-csslch-l">l</a></code>, <code><a href="#dom-csslch-c">c</a></code>, <code><a href="#dom-csslch-h">h</a></code>, and <code><a href="#dom-csslch-alpha">alpha</a></code> internal slots set to <var>l</var>, <var>c</var>, <var>h</var>, and <var>alpha</var>.

<a id="ref-for-csslch②"></a>

<a id="ref-for-rectify-a-csscolorangle⑤"></a>

The <a id="dom-csslch-h"></a>`h` attribute of a <code><a href="#csslch">CSSLCH</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorAngle](#rectify-a-csscolorangle) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-csslch③"></a>

<a id="ref-for-rectify-a-csscolorpercent⑨"></a>

The <a id="dom-csslch-l"></a>`l`, <a id="dom-csslch-c"></a>`c`, and <a id="dom-csslch-alpha"></a>`alpha` attributes of a <code><a href="#csslch">CSSLCH</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorPercent](#rectify-a-csscolorpercent) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-Exposed③⑧"></a>

<a id="cssoklab"></a>

<a id="ref-for-csscolorvalue⑨"></a>

<a id="dom-cssoklab-cssoklab"></a>

<a id="ref-for-typedefdef-csscolorpercent①⑧"></a>

<a id="dom-cssoklab-cssoklab-l-a-b-alpha-l"></a>

<a id="ref-for-typedefdef-csscolornumber④"></a>

<a id="dom-cssoklab-cssoklab-l-a-b-alpha-a"></a>

<a id="ref-for-typedefdef-csscolornumber⑤"></a>

<a id="dom-cssoklab-cssoklab-l-a-b-alpha-b"></a>

<a id="ref-for-typedefdef-csscolorpercent①⑨"></a>

<a id="dom-cssoklab-cssoklab-l-a-b-alpha-alpha"></a>

<a id="ref-for-typedefdef-csscolorpercent②⓪"></a>

<a id="ref-for-dom-cssoklab-l"></a>

<a id="ref-for-typedefdef-csscolornumber⑥"></a>

<a id="ref-for-dom-cssoklab-a"></a>

<a id="ref-for-typedefdef-csscolornumber⑦"></a>

<a id="ref-for-dom-cssoklab-b"></a>

<a id="ref-for-typedefdef-csscolorpercent②①"></a>

<a id="ref-for-dom-cssoklab-alpha"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSOKLab : CSSColorValue {
    constructor(CSSColorPercent l, CSSColorNumber a, CSSColorNumber b, optional CSSColorPercent alpha = 1);
    attribute CSSColorPercent l;
    attribute CSSColorNumber a;
    attribute CSSColorNumber b;
    attribute CSSColorPercent alpha;
};
```
<a id="ref-for-cssoklab"></a>

<a id="ref-for-funcdef-oklab"></a>

The <code><a href="#cssoklab">CSSOKLab</a></code> class represents the CSS [oklab()](https://www.w3.org/TR/css-color-5/#funcdef-oklab) function.

The <a id="dom-cssoklab-cssoklab-l-a-b-optional-alpha"></a><code>CSSOKLab(<var>l</var>, <var>a</var>, <var>b</var>, optional <var>alpha</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-rectify-a-csscolornumber②"></a>

    <a id="ref-for-rectify-a-csscolorpercent①⓪"></a>

    Let <var>a</var> and <var>b</var> be replaced by the result of [rectifying a CSSColorNumber](#rectify-a-csscolornumber) from each of them. Let <var>l</var> and <var>alpha</var> be replaced by the result of [rectifying a CSSColorPercent](#rectify-a-csscolorpercent) from each of them.

2.  <a id="ref-for-cssoklab①"></a>

    <a id="ref-for-dom-cssoklab-l①"></a>

    <a id="ref-for-dom-cssoklab-a①"></a>

    <a id="ref-for-dom-cssoklab-b①"></a>

    <a id="ref-for-dom-cssoklab-alpha①"></a>

    Return a new <code><a href="#cssoklab">CSSOKLab</a></code> with its <code><a href="#dom-cssoklab-l">l</a></code>, <code><a href="#dom-cssoklab-a">a</a></code>, <code><a href="#dom-cssoklab-b">b</a></code>, and <code><a href="#dom-cssoklab-alpha">alpha</a></code> internal slots set to <var>l</var>, <var>a</var>, <var>b</var>, and <var>alpha</var>.

<a id="ref-for-cssoklab②"></a>

<a id="ref-for-rectify-a-csscolorpercent①①"></a>

The <a id="dom-cssoklab-l"></a>`l` and <a id="dom-cssoklab-alpha"></a>`alpha` attributes of a <code><a href="#cssoklab">CSSOKLab</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorPercent](#rectify-a-csscolorpercent) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-cssoklab③"></a>

<a id="ref-for-rectify-a-csscolornumber③"></a>

The <a id="dom-cssoklab-a"></a>`a`, and <a id="dom-cssoklab-b"></a>`b` attributes of a <code><a href="#cssoklab">CSSOKLab</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorNumber](#rectify-a-csscolornumber) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-Exposed③⑨"></a>

<a id="cssoklch"></a>

<a id="ref-for-csscolorvalue①⓪"></a>

<a id="dom-cssoklch-cssoklch"></a>

<a id="ref-for-typedefdef-csscolorpercent②②"></a>

<a id="dom-cssoklch-cssoklch-l-c-h-alpha-l"></a>

<a id="ref-for-typedefdef-csscolorpercent②③"></a>

<a id="dom-cssoklch-cssoklch-l-c-h-alpha-c"></a>

<a id="ref-for-typedefdef-csscolorangle④"></a>

<a id="dom-cssoklch-cssoklch-l-c-h-alpha-h"></a>

<a id="ref-for-typedefdef-csscolorpercent②④"></a>

<a id="dom-cssoklch-cssoklch-l-c-h-alpha-alpha"></a>

<a id="ref-for-typedefdef-csscolorpercent②⑤"></a>

<a id="ref-for-dom-cssoklch-l"></a>

<a id="ref-for-typedefdef-csscolorpercent②⑥"></a>

<a id="ref-for-dom-cssoklch-c"></a>

<a id="ref-for-typedefdef-csscolorangle⑤"></a>

<a id="ref-for-dom-cssoklch-h"></a>

<a id="ref-for-typedefdef-csscolorpercent②⑦"></a>

<a id="ref-for-dom-cssoklch-alpha"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSOKLCH : CSSColorValue {
    constructor(CSSColorPercent l, CSSColorPercent c, CSSColorAngle h, optional CSSColorPercent alpha = 1);
    attribute CSSColorPercent l;
    attribute CSSColorPercent c;
    attribute CSSColorAngle h;
    attribute CSSColorPercent alpha;
};
```
<a id="ref-for-cssoklch"></a>

<a id="ref-for-funcdef-lch①"></a>

The <code><a href="#cssoklch">CSSOKLCH</a></code> class represents the CSS [lch()](https://www.w3.org/TR/css-color-5/#funcdef-lch) function.

The <a id="dom-cssoklch-cssoklch-l-c-h-optional-alpha"></a><code>CSSOKLCH(<var>l</var>, <var>c</var>, <var>h</var>, optional <var>alpha</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-rectify-a-csscolorangle⑥"></a>

    <a id="ref-for-rectify-a-csscolorpercent①②"></a>

    Let <var>h</var> be replaced by the result of [rectifying a CSSColorAngle](#rectify-a-csscolorangle) from it. Let <var>l</var>, <var>c</var>, and <var>alpha</var> be replaced by the result of [rectifying a CSSColorPercent](#rectify-a-csscolorpercent) from each of them.

2.  <a id="ref-for-cssoklch①"></a>

    <a id="ref-for-dom-cssoklch-l①"></a>

    <a id="ref-for-dom-cssoklch-c①"></a>

    <a id="ref-for-dom-cssoklch-h①"></a>

    <a id="ref-for-dom-cssoklch-alpha①"></a>

    Return a new <code><a href="#cssoklch">CSSOKLCH</a></code> with its <code><a href="#dom-cssoklch-l">l</a></code>, <code><a href="#dom-cssoklch-c">c</a></code>, <code><a href="#dom-cssoklch-h">h</a></code>, and <code><a href="#dom-cssoklch-alpha">alpha</a></code> internal slots set to <var>l</var>, <var>c</var>, <var>h</var>, and <var>alpha</var>.

<a id="ref-for-cssoklch②"></a>

<a id="ref-for-rectify-a-csscolorangle⑦"></a>

The <a id="dom-cssoklch-h"></a>`h` attribute of a <code><a href="#cssoklch">CSSOKLCH</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorAngle](#rectify-a-csscolorangle) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-cssoklch③"></a>

<a id="ref-for-rectify-a-csscolorpercent①③"></a>

The <a id="dom-cssoklch-l"></a>`l`, <a id="dom-cssoklch-c"></a>`c`, and <a id="dom-cssoklch-alpha"></a>`alpha` attributes of a <code><a href="#cssoklch">CSSOKLCH</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorPercent](#rectify-a-csscolorpercent) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-Exposed④⓪"></a>

<a id="csscolor"></a>

<a id="ref-for-csscolorvalue①①"></a>

<a id="dom-csscolor-csscolor"></a>

<a id="ref-for-typedefdef-csskeywordish⑤"></a>

<a id="dom-csscolor-csscolor-colorspace-channels-alpha-colorspace"></a>

<a id="ref-for-idl-sequence⑤"></a>

<a id="ref-for-typedefdef-csscolorpercent②⑧"></a>

<a id="dom-csscolor-csscolor-colorspace-channels-alpha-channels"></a>

<a id="ref-for-typedefdef-cssnumberish③⑧"></a>

<a id="dom-csscolor-csscolor-colorspace-channels-alpha-alpha"></a>

<a id="ref-for-typedefdef-csskeywordish⑥"></a>

<a id="ref-for-dom-csscolor-colorspace"></a>

<a id="ref-for-idl-observable-array"></a>

<a id="ref-for-typedefdef-csscolorpercent②⑨"></a>

<a id="dom-csscolor-channels"></a>

<a id="ref-for-typedefdef-cssnumberish③⑨"></a>

<a id="ref-for-dom-csscolor-alpha"></a>

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSColor : CSSColorValue {
    constructor(CSSKeywordish colorSpace, sequence<CSSColorPercent> channels, optional CSSNumberish alpha = 1);
    attribute CSSKeywordish colorSpace;
    attribute ObservableArray<CSSColorPercent> channels;
    attribute CSSNumberish alpha;
};
```
<a id="ref-for-csscolor"></a>

<a id="ref-for-funcdef-color"></a>

The <code><a href="#csscolor">CSSColor</a></code> class represents the CSS [color()](https://www.w3.org/TR/css-color-4/#funcdef-color) function.

The <a id="dom-csscolor-csscolor-colorspace-channels-optional-alpha"></a><code>CSSColor(<var>colorSpace</var>, <var>channels</var>, optional <var>alpha</var>)</code> constructor must, when invoked, perform the following steps:

1.  <a id="ref-for-rectify-a-keywordish-value⑤"></a>

    <a id="ref-for-rectify-a-csscolorpercent①④"></a>

    <a id="ref-for-rectify-a-csscolorpercent①⑤"></a>

    Let <var>colorSpace</var> be replaced by the result of [rectifying a keywordish value](#rectify-a-keywordish-value) from it. Let each item in <var>channels</var> be replaced by the result of [rectifying a CSSColorPercent](#rectify-a-csscolorpercent) from the item. Let <var>alpha</var> be replaced by the result of [rectifying a CSSColorPercent](#rectify-a-csscolorpercent) from it.

2.  <a id="ref-for-csscolor①"></a>

    <a id="ref-for-dom-csscolor-colorspace①"></a>

    <a id="ref-for-dom-csscolor-channels"></a>

    <a id="ref-for-dom-csscolor-alpha①"></a>

    Return a new <code><a href="#csscolor">CSSColor</a></code> with its <code><a href="#dom-csscolor-colorspace">colorSpace</a></code>, <code><a href="#dom-csscolor-channels">channels</a></code>, and <code><a href="#dom-csscolor-alpha">alpha</a></code> internal slots set to <var>colorSpace</var>, <var>channels</var>, and <var>alpha</var>.

<a id="ref-for-csscolor②"></a>

<a id="ref-for-rectify-a-keywordish-value⑥"></a>

The <a id="dom-csscolor-colorspace"></a>`colorSpace` attribute of a <code><a href="#csscolor">CSSColor</a></code> value must, on setting to a new value <var>val</var>, [rectify a keywordish value](#rectify-a-keywordish-value) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-csscolor③"></a>

<a id="ref-for-rectify-a-csscolorpercent①⑥"></a>

The <a id="dom-csscolor-alpha"></a>`alpha` attribute of a <code><a href="#csscolor">CSSColor</a></code> value must, on setting to a new value <var>val</var>, [rectify a CSSColorPercent](#rectify-a-csscolorpercent) from <var>val</var> and set the corresponding internal slot to the result of that.

<a id="ref-for-observable-array-attribute-set-an-indexed-value"></a>

<a id="ref-for-csscolor④"></a>

<a id="ref-for-dom-csscolor-channels①"></a>

To [set an indexed value](https://webidl.spec.whatwg.org/#observable-array-attribute-set-an-indexed-value) <var>val</var> at index <var>i</var> for a <code><a href="#csscolor">CSSColor</a></code> value’s <code><a href="#dom-csscolor-channels">channels</a></code> attribute:

1.  <a id="ref-for-rectify-a-csscolorpercent①⑦"></a>

    Replace <var>val</var> with the result of [rectifying a CSSColorPercent](#rectify-a-csscolorpercent) from <var>val</var>.

2.  <a id="ref-for-dom-csscolor-channels②"></a>

    <a id="ref-for-observable-array-attribute-backing-list"></a>

    Set the <var>i</var>th value in <code><a href="#dom-csscolor-channels">channels</a></code>'s [backing list](https://webidl.spec.whatwg.org/#observable-array-attribute-backing-list) to <var>val</var>.

<a id="ref-for-observable-array-attribute-delete-an-indexed-value"></a>

<a id="ref-for-csscolor⑤"></a>

<a id="ref-for-dom-csscolor-channels③"></a>

To [delete an indexed value](https://webidl.spec.whatwg.org/#observable-array-attribute-delete-an-indexed-value) <var>val</var> at index <var>i</var> for a <code><a href="#csscolor">CSSColor</a></code> value’s <code><a href="#dom-csscolor-channels">channels</a></code> attribute:

1.  <a id="ref-for-list-remove①"></a>

    <a id="ref-for-dom-csscolor-channels④"></a>

    <a id="ref-for-observable-array-attribute-backing-list①"></a>

    [Remove](https://infra.spec.whatwg.org/#list-remove) the <var>i</var>th value from <code><a href="#dom-csscolor-channels">channels</a></code>'s [backing list](https://webidl.spec.whatwg.org/#observable-array-attribute-backing-list).

<a id="ref-for-cssstylevalue③⓪"></a>

## <a id="reify-stylevalue"></a>5. <code><a href="#cssstylevalue">CSSStyleValue</a></code> Reification

<a id="ref-for-css-internal-representation①①"></a>

This section describes how Typed OM objects are constructed from [internal representations](#css-internal-representation), a process called <a id="css-reify"></a>reification.

<a id="ref-for-css-reify⑤"></a>

Some general principles apply to all [reification](#css-reify), and so aren’t stated in each individual instance:

- <a id="ref-for-css-internal-representation①②"></a>

  <a id="ref-for-list-valued-properties④"></a>

  <a id="ref-for-stylepropertymap①⑥"></a>

  <a id="ref-for-dom-stylepropertymapreadonly-getall①"></a>

  If an [internal representation](#css-internal-representation) is from a [list-valued](#list-valued-properties) property, this list defines how to reify a single iteration of the property; multiple iterations are reflected by returning multiple values from <code><a href="#stylepropertymap">StylePropertyMap</a></code>.<code><a href="#dom-stylepropertymapreadonly-getall">getAll()</a></code>.

- <a id="ref-for-css-internal-representation①③"></a>

  <a id="ref-for-funcdef-var⑤"></a>

  <a id="ref-for-reify-a-list-of-component-values"></a>

  If an [internal representation](#css-internal-representation) contains a [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) reference, then it is reified by [reifying a list of component values](#reify-a-list-of-component-values), regardless of what property it is for.

### <a id="reify-property"></a>5.1. Property-specific Rules

<a id="ref-for-css-reify⑥"></a>

The following list defines the [reification](#css-reify) behavior for every single property in CSS, for both specified and computed values.

<a id="ref-for-custom-property④"></a>

unregistered [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property)

<a id="ref-for-reify-a-list-of-component-values①"></a>

For both specified and computed values, [reify a list of component values](#reify-a-list-of-component-values) from the value, and return the result.

<a id="ref-for-custom-property⑤"></a>

registered [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property)

Reified as described by [CSS Properties and Values API 1 § 6.2 CSSStyleValue Reification](https://www.w3.org/TR/css-properties-values-api-1/#css-style-value-reification).

<a id="ref-for-propdef-align-content"></a>

[align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content)

<a id="ref-for-propdef-align-items"></a>

[align-items](https://www.w3.org/TR/css-align-3/#propdef-align-items)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier"></a>

    If the value is normal or stretch, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-an-identifier①"></a>

    If the value is baseline or first baseline,, [reify an identifier](#reify-an-identifier) "baseline" and return the result.

3.  <a id="ref-for-typedef-self-position"></a>

    <a id="ref-for-typedef-overflow-position"></a>

    <a id="ref-for-reify-an-identifier②"></a>

    If the value is a [\<self-position\>](https://www.w3.org/TR/css-align-3/#typedef-self-position) with no [\<overflow-position\>](https://www.w3.org/TR/css-align-3/#typedef-overflow-position), [reify an identifier](#reify-an-identifier) from the value and return the result.

4.  <a id="ref-for-cssstylevalue③①"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-align-self"></a>

[align-self](https://www.w3.org/TR/css-align-3/#propdef-align-self)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier③"></a>

    If the value is auto, normal, or stretch, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-an-identifier④"></a>

    If the value is baseline or first baseline,, [reify an identifier](#reify-an-identifier) "baseline" and return the result.

3.  <a id="ref-for-typedef-self-position①"></a>

    <a id="ref-for-typedef-overflow-position①"></a>

    <a id="ref-for-reify-an-identifier⑤"></a>

    If the value is a [\<self-position\>](https://www.w3.org/TR/css-align-3/#typedef-self-position) with no [\<overflow-position\>](https://www.w3.org/TR/css-align-3/#typedef-overflow-position), [reify an identifier](#reify-an-identifier) from the value and return the result.

4.  <a id="ref-for-cssstylevalue③②"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-alignment-baseline"></a>

[alignment-baseline](https://www.w3.org/TR/css-inline-3/#propdef-alignment-baseline)

<a id="ref-for-reify-an-identifier⑥"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-all"></a>

[all](https://www.w3.org/TR/css-cascade-5/#propdef-all)

<a id="ref-for-reify-an-identifier⑦"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-animation-composition"></a>

[animation-composition](https://www.w3.org/TR/css-animations-2/#propdef-animation-composition)

<a id="ref-for-reify-an-identifier⑧"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-appearance"></a>

[appearance](https://www.w3.org/TR/css-ui-4/#propdef-appearance)

<a id="ref-for-reify-an-identifier⑨"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-azimuth"></a>

[azimuth](https://www.w3.org/TR/CSS21/aural.html#propdef-azimuth)

For specified values:  
1.  <a id="ref-for-angle-value⑨"></a>

    <a id="ref-for-reify-a-numeric-value①"></a>

    If the value is an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

2.  <a id="ref-for-reify-an-identifier①⓪"></a>

    If the value is a single keyword, [reify an identifier](#reify-an-identifier) from the value and return the result.

3.  <a id="ref-for-cssstylevalue③③"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

For computed values:  
<a id="ref-for-reify-a-numeric-value②"></a>

[Reify a numeric value](#reify-a-numeric-value) from the angle and return the result.

<a id="ref-for-propdef-backdrop-filter"></a>

[backdrop-filter](https://drafts.fxtf.org/filter-effects-2/#propdef-backdrop-filter)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier①①"></a>

    If the value is none, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-cssstylevalue③④"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-backface-visibility"></a>

[backface-visibility](https://www.w3.org/TR/css-transforms-2/#propdef-backface-visibility)

<a id="ref-for-reify-an-identifier①②"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-background"></a>

[background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background)

<a id="ref-for-cssstylevalue③⑤"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-background-attachment"></a>

[background-attachment](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-attachment)

<a id="ref-for-reify-an-identifier①③"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-background-blend-mode"></a>

[background-blend-mode](https://www.w3.org/TR/compositing-1/#propdef-background-blend-mode)

<a id="ref-for-reify-an-identifier①④"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-background-clip"></a>

[background-clip](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-clip)

<a id="ref-for-reify-an-identifier①⑤"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-background-color"></a>

[background-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color)

<a id="ref-for-cssstylevalue③⑥"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-background-image④"></a>

[background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier①⑥"></a>

    If the value is none, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-funcdef-url①"></a>

    If the value is a [url()](https://www.w3.org/TR/css-values-4/#funcdef-url) function, reify a url from the value and return the result.

3.  Otherwise, reify an image from the value and return the result.

<a id="ref-for-propdef-background-position"></a>

[background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position)

For both specified and computed values, reify a position from the value and return the result.

<a id="ref-for-propdef-background-repeat"></a>

[background-repeat](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-repeat)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier①⑦"></a>

    If the value is a single keyword, or the same keyword repeated twice, [reify an identifier](#reify-an-identifier) from the keyword and return the result.

2.  <a id="ref-for-reify-an-identifier①⑧"></a>

    If the value is repeat no-repeat, [reify an identifier](#reify-an-identifier) "repeat-x" and return the result.

3.  <a id="ref-for-reify-an-identifier①⑨"></a>

    If the value is no-repeat repeat, [reify an identifier](#reify-an-identifier) "repeat-y" and return the result.

4.  <a id="ref-for-cssstylevalue③⑦"></a>

    Otherwise, reify to a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-baseline-shift"></a>

[baseline-shift](https://www.w3.org/TR/css-inline-3/#propdef-baseline-shift)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier②⓪"></a>

    If the value is sub or super, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value③"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-block-size"></a>

[block-size](https://www.w3.org/TR/css-logical-1/#propdef-block-size)

<a id="ref-for-propdef-width②"></a>

Same as for [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)

<a id="ref-for-propdef-block-step"></a>

[block-step](https://www.w3.org/TR/css-rhythm-1/#propdef-block-step)

<a id="ref-for-cssstylevalue③⑧"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-block-step-align"></a>

[block-step-align](https://www.w3.org/TR/css-rhythm-1/#propdef-block-step-align)

<a id="ref-for-reify-an-identifier②①"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-block-step-insert"></a>

[block-step-insert](https://www.w3.org/TR/css-rhythm-1/#propdef-block-step-insert)

<a id="ref-for-reify-an-identifier②②"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-block-step-round"></a>

[block-step-round](https://www.w3.org/TR/css-rhythm-1/#propdef-block-step-round)

<a id="ref-for-reify-an-identifier②③"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-block-step-size"></a>

[block-step-size](https://www.w3.org/TR/css-rhythm-1/#propdef-block-step-size)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier②④"></a>

    If the value is none, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value④"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-bookmark-label"></a>

[bookmark-label](https://www.w3.org/TR/css-content-3/#propdef-bookmark-label)

<a id="ref-for-cssstylevalue③⑨"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-bookmark-level"></a>

[bookmark-level](https://www.w3.org/TR/css-content-3/#propdef-bookmark-level)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier②⑤"></a>

    If the value is none, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value⑤"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-bookmark-state"></a>

[bookmark-state](https://www.w3.org/TR/css-content-3/#propdef-bookmark-state)

<a id="ref-for-reify-an-identifier②⑥"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-border"></a>

[border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border)

<a id="ref-for-cssstylevalue④⓪"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-border-block"></a>

[border-block](https://drafts.csswg.org/css-borders-4/#propdef-border-block)

<a id="ref-for-propdef-border-block-start"></a>

Same as [border-block-start](https://drafts.csswg.org/css-borders-4/#propdef-border-block-start)

<a id="ref-for-propdef-border-block-color"></a>

[border-block-color](https://drafts.csswg.org/css-borders-4/#propdef-border-block-color)

<a id="ref-for-cssstylevalue④①"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-border-block-end"></a>

[border-block-end](https://drafts.csswg.org/css-borders-4/#propdef-border-block-end)

<a id="ref-for-propdef-border-block-start①"></a>

Same as [border-block-start](https://drafts.csswg.org/css-borders-4/#propdef-border-block-start)

<a id="ref-for-propdef-border-block-end-color"></a>

[border-block-end-color](https://drafts.csswg.org/css-borders-4/#propdef-border-block-end-color)

<a id="ref-for-propdef-border-top-color"></a>

Same as [border-top-color](https://drafts.csswg.org/css-borders-4/#propdef-border-top-color)

<a id="ref-for-propdef-border-block-end-style"></a>

[border-block-end-style](https://drafts.csswg.org/css-borders-4/#propdef-border-block-end-style)

<a id="ref-for-propdef-border-top-style"></a>

Same as [border-top-style](https://drafts.csswg.org/css-borders-4/#propdef-border-top-style)

<a id="ref-for-propdef-border-block-end-width"></a>

[border-block-end-width](https://drafts.csswg.org/css-borders-4/#propdef-border-block-end-width)

<a id="ref-for-propdef-border-top-width"></a>

Same as [border-top-width](https://drafts.csswg.org/css-borders-4/#propdef-border-top-width)

<a id="ref-for-propdef-border-block-start②"></a>

[border-block-start](https://drafts.csswg.org/css-borders-4/#propdef-border-block-start)

<a id="ref-for-propdef-border-top"></a>

Same as [border-top](https://drafts.csswg.org/css-borders-4/#propdef-border-top)

<a id="ref-for-propdef-border-block-start-color"></a>

[border-block-start-color](https://drafts.csswg.org/css-borders-4/#propdef-border-block-start-color)

<a id="ref-for-propdef-border-top-color①"></a>

Same as [border-top-color](https://drafts.csswg.org/css-borders-4/#propdef-border-top-color)

<a id="ref-for-propdef-border-block-start-style"></a>

[border-block-start-style](https://drafts.csswg.org/css-borders-4/#propdef-border-block-start-style)

<a id="ref-for-propdef-border-top-style①"></a>

Same as [border-top-style](https://drafts.csswg.org/css-borders-4/#propdef-border-top-style)

<a id="ref-for-propdef-border-block-start-width"></a>

[border-block-start-width](https://drafts.csswg.org/css-borders-4/#propdef-border-block-start-width)

<a id="ref-for-propdef-border-top-width①"></a>

Same as [border-top-width](https://drafts.csswg.org/css-borders-4/#propdef-border-top-width)

<a id="ref-for-propdef-border-block-style"></a>

[border-block-style](https://drafts.csswg.org/css-borders-4/#propdef-border-block-style)

<a id="ref-for-cssstylevalue④②"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-border-block-width"></a>

[border-block-width](https://drafts.csswg.org/css-borders-4/#propdef-border-block-width)

<a id="ref-for-cssstylevalue④③"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-border-bottom"></a>

[border-bottom](https://drafts.csswg.org/css-borders-4/#propdef-border-bottom)

<a id="ref-for-propdef-border-top①"></a>

Same as [border-top](https://drafts.csswg.org/css-borders-4/#propdef-border-top)

<a id="ref-for-propdef-border-bottom-color"></a>

[border-bottom-color](https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-color)

<a id="ref-for-propdef-border-top-color②"></a>

Same as [border-top-color](https://drafts.csswg.org/css-borders-4/#propdef-border-top-color)

<a id="ref-for-propdef-border-bottom-style"></a>

[border-bottom-style](https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-style)

<a id="ref-for-propdef-border-top-style②"></a>

Same as [border-top-style](https://drafts.csswg.org/css-borders-4/#propdef-border-top-style)

<a id="ref-for-propdef-border-bottom-width"></a>

[border-bottom-width](https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-width)

<a id="ref-for-propdef-border-top-width②"></a>

Same as [border-top-width](https://drafts.csswg.org/css-borders-4/#propdef-border-top-width)

<a id="ref-for-propdef-border-boundary"></a>

[border-boundary](https://www.w3.org/TR/css-round-display-1/#propdef-border-boundary)

<a id="ref-for-reify-an-identifier②⑦"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-border-collapse"></a>

[border-collapse](https://www.w3.org/TR/css-tables-3/#propdef-border-collapse)

<a id="ref-for-reify-an-identifier②⑧"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-border-color"></a>

[border-color](https://drafts.csswg.org/css-borders-4/#propdef-border-color)

<a id="ref-for-cssstylevalue④④"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-border-inline"></a>

[border-inline](https://drafts.csswg.org/css-borders-4/#propdef-border-inline)

<a id="ref-for-propdef-border-inline-color"></a>

[border-inline-color](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-color)

<a id="ref-for-propdef-border-inline-end"></a>

[border-inline-end](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end)

<a id="ref-for-propdef-border-inline-end-color"></a>

[border-inline-end-color](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end-color)

<a id="ref-for-propdef-border-inline-end-style"></a>

[border-inline-end-style](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end-style)

<a id="ref-for-propdef-border-inline-end-width"></a>

[border-inline-end-width](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end-width)

<a id="ref-for-propdef-border-inline-start"></a>

[border-inline-start](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start)

<a id="ref-for-propdef-border-inline-start-color"></a>

[border-inline-start-color](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start-color)

<a id="ref-for-propdef-border-inline-start-style"></a>

[border-inline-start-style](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start-style)

<a id="ref-for-propdef-border-inline-start-width"></a>

[border-inline-start-width](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start-width)

<a id="ref-for-propdef-border-inline-style"></a>

[border-inline-style](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-style)

<a id="ref-for-propdef-border-inline-width"></a>

[border-inline-width](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-width)

<a id="ref-for-propdef-border-left"></a>

[border-left](https://drafts.csswg.org/css-borders-4/#propdef-border-left)

<a id="ref-for-propdef-border-top②"></a>

Same as [border-top](https://drafts.csswg.org/css-borders-4/#propdef-border-top)

<a id="ref-for-propdef-border-left-color"></a>

[border-left-color](https://drafts.csswg.org/css-borders-4/#propdef-border-left-color)

<a id="ref-for-propdef-border-top-color③"></a>

Same as [border-top-color](https://drafts.csswg.org/css-borders-4/#propdef-border-top-color)

<a id="ref-for-propdef-border-left-style"></a>

[border-left-style](https://drafts.csswg.org/css-borders-4/#propdef-border-left-style)

<a id="ref-for-propdef-border-top-style③"></a>

Same as [border-top-style](https://drafts.csswg.org/css-borders-4/#propdef-border-top-style)

<a id="ref-for-propdef-border-left-width"></a>

[border-left-width](https://drafts.csswg.org/css-borders-4/#propdef-border-left-width)

<a id="ref-for-propdef-border-top-width③"></a>

Same as [border-top-width](https://drafts.csswg.org/css-borders-4/#propdef-border-top-width)

<a id="ref-for-propdef-border-radius"></a>

[border-radius](https://drafts.csswg.org/css-borders-4/#propdef-border-radius)

<a id="ref-for-cssstylevalue④⑤"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-border-right"></a>

[border-right](https://drafts.csswg.org/css-borders-4/#propdef-border-right)

<a id="ref-for-propdef-border-top③"></a>

Same as [border-top](https://drafts.csswg.org/css-borders-4/#propdef-border-top)

<a id="ref-for-propdef-border-right-color"></a>

[border-right-color](https://drafts.csswg.org/css-borders-4/#propdef-border-right-color)

<a id="ref-for-propdef-border-top-color④"></a>

Same as [border-top-color](https://drafts.csswg.org/css-borders-4/#propdef-border-top-color)

<a id="ref-for-propdef-border-right-style"></a>

[border-right-style](https://drafts.csswg.org/css-borders-4/#propdef-border-right-style)

<a id="ref-for-propdef-border-top-style④"></a>

Same as [border-top-style](https://drafts.csswg.org/css-borders-4/#propdef-border-top-style)

<a id="ref-for-propdef-border-right-width"></a>

[border-right-width](https://drafts.csswg.org/css-borders-4/#propdef-border-right-width)

<a id="ref-for-propdef-border-top-width④"></a>

Same as [border-top-width](https://drafts.csswg.org/css-borders-4/#propdef-border-top-width)

<a id="ref-for-propdef-border-spacing"></a>

[border-spacing](https://www.w3.org/TR/CSS21/tables.html#propdef-border-spacing)

<a id="ref-for-propdef-border-style"></a>

[border-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-style)

<a id="ref-for-propdef-border-top④"></a>

[border-top](https://drafts.csswg.org/css-borders-4/#propdef-border-top)

<a id="ref-for-cssstylevalue④⑥"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-border-top-color⑤"></a>

[border-top-color](https://drafts.csswg.org/css-borders-4/#propdef-border-top-color)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier②⑨"></a>

    If the value is currentcolor, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-cssstylevalue④⑦"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-border-top-style⑤"></a>

[border-top-style](https://drafts.csswg.org/css-borders-4/#propdef-border-top-style)

<a id="ref-for-reify-an-identifier③⓪"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-border-top-width⑤"></a>

[border-top-width](https://drafts.csswg.org/css-borders-4/#propdef-border-top-width)

For both specified and computed values:

1.  <a id="ref-for-length-value⑥"></a>

    <a id="ref-for-reify-a-numeric-value⑥"></a>

    If the value is a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

2.  <a id="ref-for-reify-an-identifier③①"></a>

    Otherwise, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-border-width"></a>

[border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width)

<a id="ref-for-cssstylevalue④⑧"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-bottom"></a>

[bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier③②"></a>

    If the value is auto, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value⑦"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-box-decoration-break"></a>

[box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break)

<a id="ref-for-propdef-box-sizing"></a>

[box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing)

<a id="ref-for-reify-an-identifier③③"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-box-snap"></a>

[box-snap](https://www.w3.org/TR/css-line-grid-1/#propdef-box-snap)

<a id="ref-for-propdef-break-after"></a>

[break-after](https://www.w3.org/TR/css-break-3/#propdef-break-after)

<a id="ref-for-propdef-break-before"></a>

[break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before)

<a id="ref-for-propdef-break-inside"></a>

[break-inside](https://www.w3.org/TR/css-break-3/#propdef-break-inside)

<a id="ref-for-propdef-caption-side"></a>

[caption-side](https://www.w3.org/TR/css-tables-3/#propdef-caption-side)

<a id="ref-for-propdef-caret"></a>

[caret](https://www.w3.org/TR/css-ui-4/#propdef-caret)

<a id="ref-for-propdef-caret-color"></a>

[caret-color](https://www.w3.org/TR/css-ui-4/#propdef-caret-color)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier③④"></a>

    If the value is currentcolor, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-cssstylevalue④⑨"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-caret-shape"></a>

[caret-shape](https://www.w3.org/TR/css-ui-4/#propdef-caret-shape)

<a id="ref-for-propdef-clear"></a>

[clear](https://drafts.csswg.org/css2/#propdef-clear)

<a id="ref-for-reify-an-identifier③⑤"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-clip"></a>

[clip](https://www.w3.org/TR/css-masking-1/#propdef-clip)

<a id="ref-for-propdef-clip-path"></a>

[clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path)

<a id="ref-for-propdef-clip-rule"></a>

[clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule)

<a id="ref-for-propdef-color①"></a>

[color](https://www.w3.org/TR/css-color-4/#propdef-color)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier③⑥"></a>

    If the value is currentcolor, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-cssstylevalue⑤⓪"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-color-adjust"></a>

[color-adjust](https://www.w3.org/TR/css-color-adjust-1/#propdef-color-adjust)

<a id="ref-for-ColorInterpolationProperty"></a>

[color-interpolation](https://www.w3.org/TR/SVG2/painting.html#ColorInterpolationProperty)

<a id="ref-for-ColorRenderingProperty"></a>

[color-rendering](https://www.w3.org/TR/SVG2/painting.html#ColorRenderingProperty)

<a id="ref-for-propdef-column-gap"></a>

[column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap)

<a id="ref-for-propdef-column-span"></a>

[column-span](https://www.w3.org/TR/css-multicol-1/#propdef-column-span)

<a id="ref-for-propdef-contain"></a>

[contain](https://www.w3.org/TR/css-contain-2/#propdef-contain)

<a id="ref-for-propdef-content"></a>

[content](https://www.w3.org/TR/css-content-3/#propdef-content)

<a id="ref-for-propdef-continue"></a>

[continue](https://www.w3.org/TR/css-overflow-4/#propdef-continue)

<a id="ref-for-propdef-copy-into"></a>

[copy-into](https://drafts.csswg.org/css-gcpm-4/#propdef-copy-into)

<a id="ref-for-propdef-counter-increment"></a>

[counter-increment](https://www.w3.org/TR/css-lists-3/#propdef-counter-increment)

<a id="ref-for-propdef-counter-reset②"></a>

[counter-reset](https://www.w3.org/TR/css-lists-3/#propdef-counter-reset)

<a id="ref-for-propdef-counter-set"></a>

[counter-set](https://www.w3.org/TR/css-lists-3/#propdef-counter-set)

<a id="ref-for-propdef-cue"></a>

[cue](https://www.w3.org/TR/css-speech-1/#propdef-cue)

<a id="ref-for-propdef-cue-after"></a>

[cue-after](https://www.w3.org/TR/css-speech-1/#propdef-cue-after)

<a id="ref-for-propdef-cue-before"></a>

[cue-before](https://www.w3.org/TR/css-speech-1/#propdef-cue-before)

<a id="ref-for-propdef-cursor"></a>

[cursor](https://www.w3.org/TR/css-ui-4/#propdef-cursor)

<a id="ref-for-CxProperty"></a>

[cx](https://www.w3.org/TR/SVG2/geometry.html#CxProperty)

<a id="ref-for-CyProperty"></a>

[cy](https://www.w3.org/TR/SVG2/geometry.html#CyProperty)

<a id="ref-for-DProperty"></a>

[d](https://svgwg.org/svg2-draft/paths.html#DProperty)

<a id="ref-for-propdef-direction"></a>

[direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction)

<a id="ref-for-reify-an-identifier③⑦"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-display"></a>

[display](https://www.w3.org/TR/css-display-3/#propdef-display)

<a id="ref-for-reify-an-identifier③⑧"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-dominant-baseline"></a>

[dominant-baseline](https://www.w3.org/TR/css-inline-3/#propdef-dominant-baseline)

<a id="ref-for-propdef-elevation"></a>

[elevation](https://www.w3.org/TR/CSS21/aural.html#propdef-elevation)

<a id="ref-for-propdef-empty-cells"></a>

[empty-cells](https://www.w3.org/TR/css-tables-3/#propdef-empty-cells)

<a id="ref-for-reify-an-identifier③⑨"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-FillProperty"></a>

[fill](https://www.w3.org/TR/SVG2/painting.html#FillProperty)

<a id="ref-for-propdef-fill-break"></a>

[fill-break](https://www.w3.org/TR/fill-stroke-3/#propdef-fill-break)

<a id="ref-for-propdef-fill-color"></a>

[fill-color](https://www.w3.org/TR/fill-stroke-3/#propdef-fill-color)

<a id="ref-for-propdef-fill-image"></a>

[fill-image](https://www.w3.org/TR/fill-stroke-3/#propdef-fill-image)

<a id="ref-for-FillOpacityProperty"></a>

[fill-opacity](https://www.w3.org/TR/SVG2/painting.html#FillOpacityProperty)

<a id="ref-for-propdef-fill-origin"></a>

[fill-origin](https://www.w3.org/TR/fill-stroke-3/#propdef-fill-origin)

<a id="ref-for-propdef-fill-position"></a>

[fill-position](https://www.w3.org/TR/fill-stroke-3/#propdef-fill-position)

<a id="ref-for-propdef-fill-repeat"></a>

[fill-repeat](https://www.w3.org/TR/fill-stroke-3/#propdef-fill-repeat)

<a id="ref-for-FillRuleProperty"></a>

[fill-rule](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty)

<a id="ref-for-propdef-fill-size"></a>

[fill-size](https://www.w3.org/TR/fill-stroke-3/#propdef-fill-size)

'filter-margin-top, filter-margin-right, filter-margin-bottom, filter-margin-left'

<a id="ref-for-propdef-flex"></a>

[flex](https://www.w3.org/TR/css-flexbox-1/#propdef-flex)

<a id="ref-for-propdef-flex-basis"></a>

[flex-basis](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-basis)

<a id="ref-for-propdef-flex-direction"></a>

[flex-direction](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-direction)

<a id="ref-for-propdef-flex-flow"></a>

[flex-flow](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-flow)

<a id="ref-for-propdef-flex-grow"></a>

[flex-grow](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-grow)

<a id="ref-for-propdef-flex-shrink"></a>

[flex-shrink](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-shrink)

<a id="ref-for-propdef-flex-wrap"></a>

[flex-wrap](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-wrap)

<a id="ref-for-propdef-float"></a>

[float](https://drafts.csswg.org/css2/#propdef-float)

<a id="ref-for-reify-an-identifier④⓪"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-float-defer"></a>

[float-defer](https://www.w3.org/TR/css-page-floats-3/#propdef-float-defer)

<a id="ref-for-propdef-font"></a>

[font](https://www.w3.org/TR/css-fonts-4/#propdef-font)

<a id="ref-for-cssstylevalue⑤①"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-font-family"></a>

[font-family](https://www.w3.org/TR/css-fonts-4/#propdef-font-family)

<a id="ref-for-cssstylevalue⑤②"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-font-language-override"></a>

[font-language-override](https://www.w3.org/TR/css-fonts-4/#propdef-font-language-override)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier④①"></a>

    If the value is normal, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-cssstylevalue⑤③"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-font-optical-sizing"></a>

[font-optical-sizing](https://www.w3.org/TR/css-fonts-4/#propdef-font-optical-sizing)

<a id="ref-for-reify-an-identifier④②"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-font-palette"></a>

[font-palette](https://www.w3.org/TR/css-fonts-4/#propdef-font-palette)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier④③"></a>

    If the value is normal, light or dark, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-cssstylevalue⑤④"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-font-size"></a>

[font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size)

For both specified and computed values:

1.  <a id="ref-for-value-def-absolute-size"></a>

    <a id="ref-for-value-def-relative-size"></a>

    <a id="ref-for-reify-an-identifier④④"></a>

    If the value is an [\<absolute-size\>](https://drafts.csswg.org/css2/#value-def-absolute-size) or [\<relative-size\>](https://drafts.csswg.org/css2/#value-def-relative-size), [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value⑧"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-font-size-adjust"></a>

[font-size-adjust](https://www.w3.org/TR/css-fonts-5/#propdef-font-size-adjust)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier④⑤"></a>

    If the value is none, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value⑨"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-font-stretch"></a>

[font-stretch](https://www.w3.org/TR/css-fonts-4/#propdef-font-stretch)

For both specified and computed values:

1.  <a id="ref-for-percentage-value⑨"></a>

    <a id="ref-for-reify-a-numeric-value①⓪"></a>

    If the value is a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

2.  <a id="ref-for-reify-an-identifier④⑥"></a>

    Otherwise, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-font-style"></a>

[font-style](https://www.w3.org/TR/css-fonts-4/#propdef-font-style)

<a id="ref-for-reify-an-identifier④⑦"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-font-synthesis"></a>

[font-synthesis](https://www.w3.org/TR/css-fonts-4/#propdef-font-synthesis)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier④⑧"></a>

    If the value is none, weight, style or small-caps, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-cssstylevalue⑤⑤"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-font-variant"></a>

[font-variant](https://www.w3.org/TR/css-fonts-4/#propdef-font-variant)

<a id="ref-for-cssstylevalue⑤⑥"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-font-variant-alternates"></a>

[font-variant-alternates](https://www.w3.org/TR/css-fonts-4/#propdef-font-variant-alternates)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier④⑨"></a>

    If the value is none or historical-forms, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-cssstylevalue⑤⑦"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-font-variant-emoji"></a>

[font-variant-emoji](https://www.w3.org/TR/css-fonts-4/#propdef-font-variant-emoji)

<a id="ref-for-reify-an-identifier⑤⓪"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-font-variation-settings"></a>

[font-variation-settings](https://www.w3.org/TR/css-fonts-4/#propdef-font-variation-settings)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑤①"></a>

    If the value is normal, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-cssstylevalue⑤⑧"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-font-weight"></a>

[font-weight](https://www.w3.org/TR/css-fonts-4/#propdef-font-weight)

For both specified and computed values:

1.  <a id="ref-for-number-value⑨"></a>

    <a id="ref-for-reify-a-numeric-value①①"></a>

    If the value is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

2.  <a id="ref-for-reify-an-identifier⑤②"></a>

    Otherwise, [reify an identifier](#reify-an-identifier) from the value

<a id="ref-for-propdef-gap"></a>

[gap](https://www.w3.org/TR/css-align-3/#propdef-gap)

globalcompositeoperation

<a id="ref-for-propdef-glyph-orientation-vertical"></a>

[glyph-orientation-vertical](https://www.w3.org/TR/css-writing-modes-4/#propdef-glyph-orientation-vertical)

<a id="ref-for-propdef-grid"></a>

[grid](https://www.w3.org/TR/css-grid-2/#propdef-grid)

<a id="ref-for-propdef-grid-area"></a>

[grid-area](https://www.w3.org/TR/css-grid-2/#propdef-grid-area)

<a id="ref-for-propdef-grid-auto-columns"></a>

[grid-auto-columns](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-columns)

<a id="ref-for-propdef-grid-auto-flow"></a>

[grid-auto-flow](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-flow)

<a id="ref-for-propdef-grid-auto-rows"></a>

[grid-auto-rows](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-rows)

<a id="ref-for-propdef-grid-column"></a>

[grid-column](https://www.w3.org/TR/css-grid-2/#propdef-grid-column)

<a id="ref-for-propdef-grid-column-end"></a>

[grid-column-end](https://www.w3.org/TR/css-grid-2/#propdef-grid-column-end)

<a id="ref-for-propdef-grid-column-gap"></a>

[grid-column-gap](https://www.w3.org/TR/css-align-3/#propdef-grid-column-gap)

<a id="ref-for-propdef-grid-column-start"></a>

[grid-column-start](https://www.w3.org/TR/css-grid-2/#propdef-grid-column-start)

<a id="ref-for-propdef-grid-gap"></a>

[grid-gap](https://www.w3.org/TR/css-align-3/#propdef-grid-gap)

<a id="ref-for-propdef-grid-row"></a>

[grid-row](https://www.w3.org/TR/css-grid-2/#propdef-grid-row)

<a id="ref-for-propdef-grid-row-end"></a>

[grid-row-end](https://www.w3.org/TR/css-grid-2/#propdef-grid-row-end)

<a id="ref-for-propdef-grid-row-gap"></a>

[grid-row-gap](https://www.w3.org/TR/css-align-3/#propdef-grid-row-gap)

<a id="ref-for-propdef-grid-row-start"></a>

[grid-row-start](https://www.w3.org/TR/css-grid-2/#propdef-grid-row-start)

<a id="ref-for-propdef-grid-template"></a>

[grid-template](https://www.w3.org/TR/css-grid-2/#propdef-grid-template)

<a id="ref-for-propdef-grid-template-areas"></a>

[grid-template-areas](https://www.w3.org/TR/css-grid-2/#propdef-grid-template-areas)

<a id="ref-for-propdef-grid-template-columns"></a>

[grid-template-columns](https://www.w3.org/TR/css-grid-2/#propdef-grid-template-columns)

<a id="ref-for-propdef-grid-template-rows"></a>

[grid-template-rows](https://www.w3.org/TR/css-grid-2/#propdef-grid-template-rows)

<a id="ref-for-propdef-height"></a>

[height](https://www.w3.org/TR/css-sizing-3/#propdef-height)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑤③"></a>

    If the value is auto, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-length-value⑦"></a>

    <a id="ref-for-percentage-value①⓪"></a>

    <a id="ref-for-reify-a-numeric-value①②"></a>

    If the value is a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-image-rendering"></a>

[image-rendering](https://www.w3.org/TR/css-images-3/#propdef-image-rendering)

<a id="ref-for-propdef-image-resolution"></a>

[image-resolution](https://www.w3.org/TR/css-images-4/#propdef-image-resolution)

<a id="ref-for-propdef-initial-letter"></a>

[initial-letter](https://www.w3.org/TR/css-inline-3/#propdef-initial-letter)

<a id="ref-for-propdef-initial-letter-align"></a>

[initial-letter-align](https://www.w3.org/TR/css-inline-3/#propdef-initial-letter-align)

<a id="ref-for-propdef-initial-letter-wrap"></a>

[initial-letter-wrap](https://www.w3.org/TR/css-inline-3/#propdef-initial-letter-wrap)

<a id="ref-for-propdef-inline-size"></a>

[inline-size](https://www.w3.org/TR/css-logical-1/#propdef-inline-size)

<a id="ref-for-propdef-inset"></a>

[inset](https://www.w3.org/TR/css-position-3/#propdef-inset)

<a id="ref-for-propdef-inset-block"></a>

[inset-block](https://www.w3.org/TR/css-position-3/#propdef-inset-block)

<a id="ref-for-propdef-inset-block-end"></a>

[inset-block-end](https://www.w3.org/TR/css-position-3/#propdef-inset-block-end)

<a id="ref-for-propdef-inset-block-start"></a>

[inset-block-start](https://www.w3.org/TR/css-position-3/#propdef-inset-block-start)

<a id="ref-for-propdef-inset-inline"></a>

[inset-inline](https://www.w3.org/TR/css-position-3/#propdef-inset-inline)

<a id="ref-for-propdef-inset-inline-end"></a>

[inset-inline-end](https://www.w3.org/TR/css-position-3/#propdef-inset-inline-end)

<a id="ref-for-propdef-inset-inline-start"></a>

[inset-inline-start](https://www.w3.org/TR/css-position-3/#propdef-inset-inline-start)

<a id="ref-for-propdef-isolation"></a>

[isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation)

<a id="ref-for-propdef-justify-content"></a>

[justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content)

<a id="ref-for-propdef-justify-items"></a>

[justify-items](https://www.w3.org/TR/css-align-3/#propdef-justify-items)

<a id="ref-for-propdef-justify-self"></a>

[justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self)

<a id="ref-for-propdef-left"></a>

[left](https://www.w3.org/TR/css-position-3/#propdef-left)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑤④"></a>

    If the value is auto, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value①③"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-letter-spacing"></a>

[letter-spacing](https://www.w3.org/TR/css-text-4/#propdef-letter-spacing)

<a id="ref-for-propdef-line-grid"></a>

[line-grid](https://www.w3.org/TR/css-line-grid-1/#propdef-line-grid)

<a id="ref-for-propdef-line-height"></a>

[line-height](https://drafts.csswg.org/css2/#propdef-line-height)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑤⑤"></a>

    If the value is normal, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value①④"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-line-height-step"></a>

[line-height-step](https://www.w3.org/TR/css-rhythm-1/#propdef-line-height-step)

<a id="ref-for-propdef-line-snap"></a>

[line-snap](https://www.w3.org/TR/css-line-grid-1/#propdef-line-snap)

<a id="ref-for-propdef-list-style"></a>

[list-style](https://www.w3.org/TR/css-lists-3/#propdef-list-style)

<a id="ref-for-propdef-list-style-image①"></a>

[list-style-image](https://www.w3.org/TR/css-lists-3/#propdef-list-style-image)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑤⑥"></a>

    If the value is none, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-funcdef-url②"></a>

    If the value is a [url()](https://www.w3.org/TR/css-values-4/#funcdef-url) function, reify a url from the value and return the result.

3.  Otherwise, reify an image from the value and return the result.

<a id="ref-for-propdef-list-style-position"></a>

[list-style-position](https://www.w3.org/TR/css-lists-3/#propdef-list-style-position)

<a id="ref-for-reify-an-identifier⑤⑦"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-list-style-type"></a>

[list-style-type](https://www.w3.org/TR/css-lists-3/#propdef-list-style-type)

<a id="ref-for-propdef-margin"></a>

[margin](https://www.w3.org/TR/css-box-4/#propdef-margin)

<a id="ref-for-cssstylevalue⑤⑨"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-margin-block"></a>

[margin-block](https://www.w3.org/TR/css-logical-1/#propdef-margin-block)

<a id="ref-for-propdef-margin-block-end"></a>

[margin-block-end](https://www.w3.org/TR/css-logical-1/#propdef-margin-block-end)

<a id="ref-for-propdef-margin-block-start"></a>

[margin-block-start](https://www.w3.org/TR/css-logical-1/#propdef-margin-block-start)

<a id="ref-for-propdef-margin-bottom"></a>

[margin-bottom](https://www.w3.org/TR/css-box-4/#propdef-margin-bottom)

<a id="ref-for-propdef-margin-top"></a>

Same as [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top)

<a id="ref-for-propdef-margin-inline"></a>

[margin-inline](https://www.w3.org/TR/css-logical-1/#propdef-margin-inline)

<a id="ref-for-propdef-margin-inline-end"></a>

[margin-inline-end](https://www.w3.org/TR/css-logical-1/#propdef-margin-inline-end)

<a id="ref-for-propdef-margin-inline-start"></a>

[margin-inline-start](https://www.w3.org/TR/css-logical-1/#propdef-margin-inline-start)

<a id="ref-for-propdef-margin-left"></a>

[margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left)

<a id="ref-for-propdef-margin-top①"></a>

Same as [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top)

<a id="ref-for-propdef-margin-right"></a>

[margin-right](https://www.w3.org/TR/css-box-4/#propdef-margin-right)

<a id="ref-for-propdef-margin-top②"></a>

Same as [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top)

<a id="ref-for-propdef-margin-top③"></a>

[margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑤⑧"></a>

    If the value is auto, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value①⑤"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-MarkerProperty"></a>

[marker](https://www.w3.org/TR/SVG2/painting.html#MarkerProperty)

<a id="ref-for-MarkerEndProperty"></a>

[marker-end](https://www.w3.org/TR/SVG2/painting.html#MarkerEndProperty)

<a id="ref-for-MarkerMidProperty"></a>

[marker-mid](https://www.w3.org/TR/SVG2/painting.html#MarkerMidProperty)

<a id="ref-for-propdef-marker-side"></a>

[marker-side](https://www.w3.org/TR/css-lists-3/#propdef-marker-side)

<a id="ref-for-MarkerStartProperty"></a>

[marker-start](https://www.w3.org/TR/SVG2/painting.html#MarkerStartProperty)

<a id="ref-for-propdef-mask"></a>

[mask](https://www.w3.org/TR/css-masking-1/#propdef-mask)

<a id="ref-for-propdef-mask-border"></a>

[mask-border](https://www.w3.org/TR/css-masking-1/#propdef-mask-border)

<a id="ref-for-propdef-mask-border-mode"></a>

[mask-border-mode](https://www.w3.org/TR/css-masking-1/#propdef-mask-border-mode)

<a id="ref-for-propdef-mask-border-outset"></a>

[mask-border-outset](https://www.w3.org/TR/css-masking-1/#propdef-mask-border-outset)

<a id="ref-for-propdef-mask-border-repeat"></a>

[mask-border-repeat](https://www.w3.org/TR/css-masking-1/#propdef-mask-border-repeat)

<a id="ref-for-propdef-mask-border-slice"></a>

[mask-border-slice](https://www.w3.org/TR/css-masking-1/#propdef-mask-border-slice)

<a id="ref-for-propdef-mask-border-source"></a>

[mask-border-source](https://www.w3.org/TR/css-masking-1/#propdef-mask-border-source)

<a id="ref-for-propdef-mask-border-width"></a>

[mask-border-width](https://www.w3.org/TR/css-masking-1/#propdef-mask-border-width)

<a id="ref-for-propdef-mask-clip"></a>

[mask-clip](https://www.w3.org/TR/css-masking-1/#propdef-mask-clip)

<a id="ref-for-propdef-mask-composite"></a>

[mask-composite](https://www.w3.org/TR/css-masking-1/#propdef-mask-composite)

<a id="ref-for-propdef-mask-image"></a>

[mask-image](https://www.w3.org/TR/css-masking-1/#propdef-mask-image)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑤⑨"></a>

    If the value is none, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  Otherwise, reify an image from the value and return the result.

<a id="ref-for-propdef-mask-mode"></a>

[mask-mode](https://www.w3.org/TR/css-masking-1/#propdef-mask-mode)

<a id="ref-for-propdef-mask-origin"></a>

[mask-origin](https://www.w3.org/TR/css-masking-1/#propdef-mask-origin)

<a id="ref-for-propdef-mask-position"></a>

[mask-position](https://www.w3.org/TR/css-masking-1/#propdef-mask-position)

<a id="ref-for-propdef-mask-repeat"></a>

[mask-repeat](https://www.w3.org/TR/css-masking-1/#propdef-mask-repeat)

<a id="ref-for-propdef-mask-size"></a>

[mask-size](https://www.w3.org/TR/css-masking-1/#propdef-mask-size)

<a id="ref-for-propdef-mask-type"></a>

[mask-type](https://www.w3.org/TR/css-masking-1/#propdef-mask-type)

<a id="ref-for-propdef-max-block-size"></a>

[max-block-size](https://www.w3.org/TR/css-logical-1/#propdef-max-block-size)

<a id="ref-for-propdef-max-height"></a>

[max-height](https://www.w3.org/TR/css-sizing-3/#propdef-max-height)

<a id="ref-for-propdef-max-inline-size"></a>

[max-inline-size](https://www.w3.org/TR/css-logical-1/#propdef-max-inline-size)

<a id="ref-for-propdef-max-lines"></a>

[max-lines](https://www.w3.org/TR/css-overflow-4/#propdef-max-lines)

<a id="ref-for-propdef-max-width"></a>

[max-width](https://www.w3.org/TR/css-sizing-3/#propdef-max-width)

<a id="ref-for-propdef-min-block-size"></a>

[min-block-size](https://www.w3.org/TR/css-logical-1/#propdef-min-block-size)

<a id="ref-for-propdef-min-height"></a>

[min-height](https://www.w3.org/TR/css-sizing-3/#propdef-min-height)

<a id="ref-for-propdef-min-inline-size"></a>

[min-inline-size](https://www.w3.org/TR/css-logical-1/#propdef-min-inline-size)

<a id="ref-for-propdef-min-width"></a>

[min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width)

<a id="ref-for-propdef-mix-blend-mode"></a>

[mix-blend-mode](https://www.w3.org/TR/compositing-1/#propdef-mix-blend-mode)

<a id="ref-for-propdef-nav-down"></a>

[nav-down](https://www.w3.org/TR/css-ui-4/#propdef-nav-down)

<a id="ref-for-propdef-nav-left"></a>

[nav-left](https://www.w3.org/TR/css-ui-4/#propdef-nav-left)

<a id="ref-for-propdef-nav-right"></a>

[nav-right](https://www.w3.org/TR/css-ui-4/#propdef-nav-right)

<a id="ref-for-propdef-nav-up"></a>

[nav-up](https://www.w3.org/TR/css-ui-4/#propdef-nav-up)

<a id="ref-for-propdef-object-fit"></a>

[object-fit](https://www.w3.org/TR/css-images-4/#propdef-object-fit)

<a id="ref-for-propdef-offset"></a>

[offset](https://www.w3.org/TR/motion-1/#propdef-offset)

<a id="ref-for-propdef-offset-anchor"></a>

[offset-anchor](https://www.w3.org/TR/motion-1/#propdef-offset-anchor)

<a id="ref-for-propdef-offset-distance"></a>

[offset-distance](https://www.w3.org/TR/motion-1/#propdef-offset-distance)

<a id="ref-for-propdef-offset-path"></a>

[offset-path](https://www.w3.org/TR/motion-1/#propdef-offset-path)

<a id="ref-for-propdef-offset-position"></a>

[offset-position](https://www.w3.org/TR/motion-1/#propdef-offset-position)

<a id="ref-for-propdef-offset-rotate"></a>

[offset-rotate](https://www.w3.org/TR/motion-1/#propdef-offset-rotate)

<a id="ref-for-propdef-opacity①"></a>

[opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity)

<a id="ref-for-reify-a-numeric-value①⑥"></a>

For both specified and computed values, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-order"></a>

[order](https://www.w3.org/TR/css-display-3/#propdef-order)

<a id="ref-for-propdef-orphans"></a>

[orphans](https://www.w3.org/TR/css-break-3/#propdef-orphans)

<a id="ref-for-propdef-outline"></a>

[outline](https://www.w3.org/TR/css-ui-4/#propdef-outline)

<a id="ref-for-propdef-outline-color"></a>

[outline-color](https://www.w3.org/TR/css-ui-4/#propdef-outline-color)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑥⓪"></a>

    If the value is currentcolor, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-cssstylevalue⑥⓪"></a>

    Otherwise, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-outline-offset"></a>

[outline-offset](https://www.w3.org/TR/css-ui-4/#propdef-outline-offset)

<a id="ref-for-propdef-outline-style"></a>

[outline-style](https://www.w3.org/TR/css-ui-4/#propdef-outline-style)

<a id="ref-for-reify-an-identifier⑥①"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-outline-width"></a>

[outline-width](https://www.w3.org/TR/css-ui-4/#propdef-outline-width)

<a id="ref-for-propdef-overflow"></a>

[overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow)

<a id="ref-for-cssstylevalue⑥①"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-overflow-anchor"></a>

[overflow-anchor](https://www.w3.org/TR/css-scroll-anchoring-1/#propdef-overflow-anchor)

<a id="ref-for-reify-an-identifier⑥②"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-overflow-x"></a>

[overflow-x](https://www.w3.org/TR/css-overflow-3/#propdef-overflow-x)

<a id="ref-for-reify-an-identifier⑥③"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-overflow-y"></a>

[overflow-y](https://www.w3.org/TR/css-overflow-3/#propdef-overflow-y)

<a id="ref-for-reify-an-identifier⑥④"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-padding"></a>

[padding](https://www.w3.org/TR/css-box-4/#propdef-padding)

<a id="ref-for-cssstylevalue⑥②"></a>

For both specified and computed values, reify as a <code><a href="#cssstylevalue">CSSStyleValue</a></code> and return the result.

<a id="ref-for-propdef-padding-block"></a>

[padding-block](https://www.w3.org/TR/css-logical-1/#propdef-padding-block)

<a id="ref-for-propdef-padding-block-end"></a>

[padding-block-end](https://www.w3.org/TR/css-logical-1/#propdef-padding-block-end)

<a id="ref-for-propdef-padding-block-start"></a>

[padding-block-start](https://www.w3.org/TR/css-logical-1/#propdef-padding-block-start)

<a id="ref-for-propdef-padding-bottom"></a>

[padding-bottom](https://www.w3.org/TR/css-box-4/#propdef-padding-bottom)

<a id="ref-for-propdef-padding-top"></a>

Same as [padding-top](https://www.w3.org/TR/css-box-4/#propdef-padding-top)

<a id="ref-for-propdef-padding-inline"></a>

[padding-inline](https://www.w3.org/TR/css-logical-1/#propdef-padding-inline)

<a id="ref-for-propdef-padding-inline-end"></a>

[padding-inline-end](https://www.w3.org/TR/css-logical-1/#propdef-padding-inline-end)

<a id="ref-for-propdef-padding-inline-start"></a>

[padding-inline-start](https://www.w3.org/TR/css-logical-1/#propdef-padding-inline-start)

<a id="ref-for-propdef-padding-left"></a>

[padding-left](https://www.w3.org/TR/css-box-4/#propdef-padding-left)

<a id="ref-for-propdef-padding-top①"></a>

Same as [padding-top](https://www.w3.org/TR/css-box-4/#propdef-padding-top)

<a id="ref-for-propdef-padding-right"></a>

[padding-right](https://www.w3.org/TR/css-box-4/#propdef-padding-right)

<a id="ref-for-propdef-padding-top②"></a>

Same as [padding-top](https://www.w3.org/TR/css-box-4/#propdef-padding-top)

<a id="ref-for-propdef-padding-top③"></a>

[padding-top](https://www.w3.org/TR/css-box-4/#propdef-padding-top)

<a id="ref-for-reify-a-numeric-value①⑦"></a>

For both specified and computed values, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-page"></a>

[page](https://www.w3.org/TR/css-page-3/#propdef-page)

<a id="ref-for-propdef-page-break-after"></a>

[page-break-after](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-after)

<a id="ref-for-propdef-page-break-before"></a>

[page-break-before](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-before)

<a id="ref-for-propdef-page-break-inside"></a>

[page-break-inside](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-inside)

<a id="ref-for-PaintOrderProperty"></a>

[paint-order](https://www.w3.org/TR/SVG2/painting.html#PaintOrderProperty)

<a id="ref-for-propdef-pause"></a>

[pause](https://www.w3.org/TR/css-speech-1/#propdef-pause)

<a id="ref-for-propdef-pause-after"></a>

[pause-after](https://www.w3.org/TR/css-speech-1/#propdef-pause-after)

<a id="ref-for-propdef-pause-before"></a>

[pause-before](https://www.w3.org/TR/css-speech-1/#propdef-pause-before)

<a id="ref-for-propdef-perspective"></a>

[perspective](https://www.w3.org/TR/css-transforms-2/#propdef-perspective)

<a id="ref-for-propdef-perspective-origin"></a>

[perspective-origin](https://www.w3.org/TR/css-transforms-2/#propdef-perspective-origin)

<a id="ref-for-propdef-pitch"></a>

[pitch](https://www.w3.org/TR/CSS21/aural.html#propdef-pitch)

<a id="ref-for-propdef-pitch-range"></a>

[pitch-range](https://www.w3.org/TR/CSS21/aural.html#propdef-pitch-range)

<a id="ref-for-propdef-place-content"></a>

[place-content](https://www.w3.org/TR/css-align-3/#propdef-place-content)

<a id="ref-for-propdef-place-items"></a>

[place-items](https://www.w3.org/TR/css-align-3/#propdef-place-items)

<a id="ref-for-propdef-place-self"></a>

[place-self](https://www.w3.org/TR/css-align-3/#propdef-place-self)

<a id="ref-for-propdef-play-during"></a>

[play-during](https://www.w3.org/TR/CSS21/aural.html#propdef-play-during)

<a id="ref-for-propdef-pointer-events"></a>

[pointer-events](https://drafts.csswg.org/css-ui-4/#propdef-pointer-events)

<a id="ref-for-propdef-position"></a>

[position](https://www.w3.org/TR/css-position-3/#propdef-position)

<a id="ref-for-reify-an-identifier⑥⑤"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

presentation-level

<a id="ref-for-propdef-quotes"></a>

[quotes](https://www.w3.org/TR/css-content-3/#propdef-quotes)

<a id="ref-for-RProperty"></a>

[r](https://www.w3.org/TR/SVG2/geometry.html#RProperty)

<a id="ref-for-propdef-region-fragment"></a>

[region-fragment](https://www.w3.org/TR/css-regions-1/#propdef-region-fragment)

<a id="ref-for-propdef-resize"></a>

[resize](https://www.w3.org/TR/css-ui-4/#propdef-resize)

<a id="ref-for-reify-an-identifier⑥⑥"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-rest"></a>

[rest](https://www.w3.org/TR/css-speech-1/#propdef-rest)

<a id="ref-for-propdef-rest-after"></a>

[rest-after](https://www.w3.org/TR/css-speech-1/#propdef-rest-after)

<a id="ref-for-propdef-rest-before"></a>

[rest-before](https://www.w3.org/TR/css-speech-1/#propdef-rest-before)

<a id="ref-for-propdef-richness"></a>

[richness](https://www.w3.org/TR/CSS21/aural.html#propdef-richness)

<a id="ref-for-propdef-right"></a>

[right](https://www.w3.org/TR/css-position-3/#propdef-right)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑥⑦"></a>

    If the value is auto, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value①⑧"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-rotate"></a>

[rotate](https://www.w3.org/TR/css-transforms-2/#propdef-rotate)

<a id="ref-for-propdef-row-gap"></a>

[row-gap](https://www.w3.org/TR/css-align-3/#propdef-row-gap)

<a id="ref-for-propdef-ruby-align"></a>

[ruby-align](https://www.w3.org/TR/css-ruby-1/#propdef-ruby-align)

<a id="ref-for-propdef-ruby-merge"></a>

[ruby-merge](https://www.w3.org/TR/css-ruby-1/#propdef-ruby-merge)

<a id="ref-for-propdef-ruby-position"></a>

[ruby-position](https://www.w3.org/TR/css-ruby-1/#propdef-ruby-position)

<a id="ref-for-RxProperty"></a>

[rx](https://www.w3.org/TR/SVG2/geometry.html#RxProperty)

<a id="ref-for-RyProperty"></a>

[ry](https://www.w3.org/TR/SVG2/geometry.html#RyProperty)

<a id="ref-for-propdef-scale"></a>

[scale](https://www.w3.org/TR/css-transforms-2/#propdef-scale)

<a id="ref-for-propdef-scroll-behavior"></a>

[scroll-behavior](https://www.w3.org/TR/css-overflow-3/#propdef-scroll-behavior)

<a id="ref-for-propdef-scroll-margin"></a>

[scroll-margin](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin)

<a id="ref-for-propdef-scroll-margin-block"></a>

[scroll-margin-block](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin-block)

<a id="ref-for-propdef-scroll-margin-block-end"></a>

[scroll-margin-block-end](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin-block-end)

<a id="ref-for-propdef-scroll-margin-block-start"></a>

[scroll-margin-block-start](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin-block-start)

<a id="ref-for-propdef-scroll-margin-bottom"></a>

[scroll-margin-bottom](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin-bottom)

<a id="ref-for-propdef-scroll-margin-inline"></a>

[scroll-margin-inline](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin-inline)

<a id="ref-for-propdef-scroll-margin-inline-end"></a>

[scroll-margin-inline-end](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin-inline-end)

<a id="ref-for-propdef-scroll-margin-inline-start"></a>

[scroll-margin-inline-start](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin-inline-start)

<a id="ref-for-propdef-scroll-margin-left"></a>

[scroll-margin-left](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin-left)

<a id="ref-for-propdef-scroll-margin-right"></a>

[scroll-margin-right](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin-right)

<a id="ref-for-propdef-scroll-margin-top"></a>

[scroll-margin-top](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin-top)

<a id="ref-for-propdef-scroll-padding"></a>

[scroll-padding](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding)

<a id="ref-for-propdef-scroll-padding-block"></a>

[scroll-padding-block](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding-block)

<a id="ref-for-propdef-scroll-padding-block-end"></a>

[scroll-padding-block-end](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding-block-end)

<a id="ref-for-propdef-scroll-padding-block-start"></a>

[scroll-padding-block-start](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding-block-start)

<a id="ref-for-propdef-scroll-padding-bottom"></a>

[scroll-padding-bottom](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding-bottom)

<a id="ref-for-propdef-scroll-padding-inline"></a>

[scroll-padding-inline](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding-inline)

<a id="ref-for-propdef-scroll-padding-inline-end"></a>

[scroll-padding-inline-end](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding-inline-end)

<a id="ref-for-propdef-scroll-padding-inline-start"></a>

[scroll-padding-inline-start](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding-inline-start)

<a id="ref-for-propdef-scroll-padding-left"></a>

[scroll-padding-left](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding-left)

<a id="ref-for-propdef-scroll-padding-right"></a>

[scroll-padding-right](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding-right)

<a id="ref-for-propdef-scroll-padding-top"></a>

[scroll-padding-top](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding-top)

<a id="ref-for-propdef-scroll-snap-align"></a>

[scroll-snap-align](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-snap-align)

<a id="ref-for-propdef-scroll-snap-stop"></a>

[scroll-snap-stop](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-snap-stop)

<a id="ref-for-propdef-scroll-snap-type"></a>

[scroll-snap-type](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-snap-type)

<a id="ref-for-propdef-scrollbar-gutter"></a>

[scrollbar-gutter](https://www.w3.org/TR/css-overflow-3/#propdef-scrollbar-gutter)

<a id="ref-for-propdef-shape-inside"></a>

[shape-inside](https://drafts.csswg.org/css-shapes-2/#propdef-shape-inside)

<a id="ref-for-propdef-shape-margin"></a>

[shape-margin](https://www.w3.org/TR/css-shapes-1/#propdef-shape-margin)

<a id="ref-for-propdef-shape-padding"></a>

[shape-padding](https://drafts.csswg.org/css-shapes-2/#propdef-shape-padding)

<a id="ref-for-ShapeRenderingProperty"></a>

[shape-rendering](https://www.w3.org/TR/SVG2/painting.html#ShapeRenderingProperty)

<a id="ref-for-ShapesubtractProperty"></a>

[shape-subtract](https://www.w3.org/TR/SVG2/text.html#ShapesubtractProperty)

<a id="ref-for-propdef-speak"></a>

[speak](https://www.w3.org/TR/css-speech-1/#propdef-speak)

<a id="ref-for-propdef-speak-as"></a>

[speak-as](https://www.w3.org/TR/css-speech-1/#propdef-speak-as)

<a id="ref-for-propdef-speak-header"></a>

[speak-header](https://www.w3.org/TR/CSS21/aural.html#propdef-speak-header)

<a id="ref-for-propdef-speak-numeral"></a>

[speak-numeral](https://www.w3.org/TR/CSS21/aural.html#propdef-speak-numeral)

<a id="ref-for-propdef-speak-punctuation"></a>

[speak-punctuation](https://www.w3.org/TR/CSS21/aural.html#propdef-speak-punctuation)

<a id="ref-for-propdef-speech-rate"></a>

[speech-rate](https://www.w3.org/TR/CSS21/aural.html#propdef-speech-rate)

<a id="ref-for-StopColorProperty"></a>

[stop-color](https://www.w3.org/TR/SVG2/pservers.html#StopColorProperty)

<a id="ref-for-StopOpacityProperty"></a>

[stop-opacity](https://www.w3.org/TR/SVG2/pservers.html#StopOpacityProperty)

<a id="ref-for-propdef-stress"></a>

[stress](https://www.w3.org/TR/CSS21/aural.html#propdef-stress)

<a id="ref-for-StrokeProperty"></a>

[stroke](https://www.w3.org/TR/SVG2/painting.html#StrokeProperty)

<a id="ref-for-propdef-stroke-align"></a>

[stroke-align](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke-align)

<a id="ref-for-propdef-stroke-break"></a>

[stroke-break](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke-break)

<a id="ref-for-propdef-stroke-color"></a>

[stroke-color](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke-color)

<a id="ref-for-propdef-stroke-dash-corner"></a>

[stroke-dash-corner](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke-dash-corner)

<a id="ref-for-propdef-stroke-dash-justify"></a>

[stroke-dash-justify](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke-dash-justify)

<a id="ref-for-StrokeDasharrayProperty"></a>

[stroke-dasharray](https://www.w3.org/TR/SVG2/painting.html#StrokeDasharrayProperty)

<a id="ref-for-StrokeDashoffsetProperty"></a>

[stroke-dashoffset](https://www.w3.org/TR/SVG2/painting.html#StrokeDashoffsetProperty)

<a id="ref-for-propdef-stroke-image"></a>

[stroke-image](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke-image)

<a id="ref-for-StrokeLinecapProperty"></a>

[stroke-linecap](https://www.w3.org/TR/SVG2/painting.html#StrokeLinecapProperty)

<a id="ref-for-StrokeLinejoinProperty"></a>

[stroke-linejoin](https://www.w3.org/TR/SVG2/painting.html#StrokeLinejoinProperty)

<a id="ref-for-StrokeMiterlimitProperty"></a>

[stroke-miterlimit](https://www.w3.org/TR/SVG2/painting.html#StrokeMiterlimitProperty)

<a id="ref-for-StrokeOpacityProperty"></a>

[stroke-opacity](https://www.w3.org/TR/SVG2/painting.html#StrokeOpacityProperty)

<a id="ref-for-propdef-stroke-origin"></a>

[stroke-origin](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke-origin)

<a id="ref-for-propdef-stroke-position"></a>

[stroke-position](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke-position)

<a id="ref-for-propdef-stroke-repeat"></a>

[stroke-repeat](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke-repeat)

<a id="ref-for-propdef-stroke-size"></a>

[stroke-size](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke-size)

<a id="ref-for-StrokeWidthProperty"></a>

[stroke-width](https://www.w3.org/TR/SVG2/painting.html#StrokeWidthProperty)

<a id="ref-for-propdef-table-layout"></a>

[table-layout](https://www.w3.org/TR/css-tables-3/#propdef-table-layout)

<a id="ref-for-propdef-text-align"></a>

[text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align)

<a id="ref-for-reify-an-identifier⑥⑧"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-TextAnchorProperty"></a>

[text-anchor](https://www.w3.org/TR/SVG2/text.html#TextAnchorProperty)

<a id="ref-for-propdef-text-combine-upright"></a>

[text-combine-upright](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-combine-upright)

<a id="ref-for-propdef-text-decoration"></a>

[text-decoration](https://www.w3.org/TR/css-text-decor-4/#propdef-text-decoration)

<a id="ref-for-TextDecorationFillProperty"></a>

[text-decoration-fill](https://www.w3.org/TR/SVG2/text.html#TextDecorationFillProperty)

<a id="ref-for-propdef-text-decoration-skip"></a>

[text-decoration-skip](https://www.w3.org/TR/css-text-decor-4/#propdef-text-decoration-skip)

<a id="ref-for-propdef-text-decoration-skip-ink"></a>

[text-decoration-skip-ink](https://www.w3.org/TR/css-text-decor-4/#propdef-text-decoration-skip-ink)

<a id="ref-for-TextDecorationStrokeProperty"></a>

[text-decoration-stroke](https://www.w3.org/TR/SVG2/text.html#TextDecorationStrokeProperty)

<a id="ref-for-propdef-text-decoration-thickness"></a>

[text-decoration-thickness](https://www.w3.org/TR/css-text-decor-4/#propdef-text-decoration-thickness)

<a id="ref-for-propdef-text-emphasis-skip"></a>

[text-emphasis-skip](https://www.w3.org/TR/css-text-decor-4/#propdef-text-emphasis-skip)

<a id="ref-for-propdef-text-indent"></a>

[text-indent](https://www.w3.org/TR/css-text-4/#propdef-text-indent)

<a id="ref-for-propdef-text-orientation"></a>

[text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation)

<a id="ref-for-propdef-text-overflow"></a>

[text-overflow](https://www.w3.org/TR/css-overflow-3/#propdef-text-overflow)

<a id="ref-for-TextRenderingProperty"></a>

[text-rendering](https://www.w3.org/TR/SVG2/painting.html#TextRenderingProperty)

<a id="ref-for-propdef-text-size-adjust"></a>

[text-size-adjust](https://drafts.csswg.org/css-size-adjust-1/#propdef-text-size-adjust)

<a id="ref-for-propdef-text-transform"></a>

[text-transform](https://www.w3.org/TR/css-text-4/#propdef-text-transform)

<a id="ref-for-reify-an-identifier⑥⑨"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-text-underline-offset"></a>

[text-underline-offset](https://www.w3.org/TR/css-text-decor-4/#propdef-text-underline-offset)

<a id="ref-for-propdef-top"></a>

[top](https://www.w3.org/TR/css-position-3/#propdef-top)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑦⓪"></a>

    If the value is auto, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value①⑨"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-transform-style"></a>

[transform-style](https://www.w3.org/TR/css-transforms-2/#propdef-transform-style)

<a id="ref-for-propdef-transition"></a>

[transition](https://www.w3.org/TR/css-transitions-1/#propdef-transition)

<a id="ref-for-propdef-transition-delay"></a>

[transition-delay](https://www.w3.org/TR/css-transitions-1/#propdef-transition-delay)

<a id="ref-for-propdef-transition-duration"></a>

[transition-duration](https://www.w3.org/TR/css-transitions-1/#propdef-transition-duration)

<a id="ref-for-propdef-transition-property"></a>

[transition-property](https://www.w3.org/TR/css-transitions-1/#propdef-transition-property)

<a id="ref-for-propdef-transition-timing-function"></a>

[transition-timing-function](https://www.w3.org/TR/css-transitions-1/#propdef-transition-timing-function)

<a id="ref-for-propdef-translate"></a>

[translate](https://www.w3.org/TR/css-transforms-2/#propdef-translate)

<a id="ref-for-propdef-unicode-bidi"></a>

[unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi)

<a id="ref-for-propdef-user-select"></a>

[user-select](https://www.w3.org/TR/css-ui-4/#propdef-user-select)

<a id="ref-for-VectorEffectProperty"></a>

[vector-effect](https://www.w3.org/TR/SVG2/coords.html#VectorEffectProperty)

<a id="ref-for-propdef-vertical-align"></a>

[vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑦①"></a>

    If the value is baseline, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-reify-a-numeric-value②⓪"></a>

    Otherwise, [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-visibility"></a>

[visibility](https://www.w3.org/TR/css-display-3/#propdef-visibility)

<a id="ref-for-reify-an-identifier⑦②"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-voice-balance"></a>

[voice-balance](https://www.w3.org/TR/css-speech-1/#propdef-voice-balance)

<a id="ref-for-propdef-voice-duration"></a>

[voice-duration](https://www.w3.org/TR/css-speech-1/#propdef-voice-duration)

<a id="ref-for-propdef-voice-family"></a>

[voice-family](https://www.w3.org/TR/css-speech-1/#propdef-voice-family)

<a id="ref-for-propdef-voice-pitch"></a>

[voice-pitch](https://www.w3.org/TR/css-speech-1/#propdef-voice-pitch)

<a id="ref-for-propdef-voice-range"></a>

[voice-range](https://www.w3.org/TR/css-speech-1/#propdef-voice-range)

<a id="ref-for-propdef-voice-rate"></a>

[voice-rate](https://www.w3.org/TR/css-speech-1/#propdef-voice-rate)

<a id="ref-for-propdef-voice-stress"></a>

[voice-stress](https://www.w3.org/TR/css-speech-1/#propdef-voice-stress)

<a id="ref-for-propdef-voice-volume"></a>

[voice-volume](https://www.w3.org/TR/css-speech-1/#propdef-voice-volume)

<a id="ref-for-propdef-volume"></a>

[volume](https://www.w3.org/TR/CSS21/aural.html#propdef-volume)

<a id="ref-for-propdef-white-space"></a>

[white-space](https://www.w3.org/TR/css-text-4/#propdef-white-space)

<a id="ref-for-reify-an-identifier⑦③"></a>

For both specified and computed values, [reify an identifier](#reify-an-identifier) from the value and return the result.

<a id="ref-for-propdef-widows"></a>

[widows](https://www.w3.org/TR/css-break-4/#propdef-widows)

<a id="ref-for-propdef-width③"></a>

[width](https://www.w3.org/TR/css-sizing-3/#propdef-width)

For both specified and computed values:

1.  <a id="ref-for-reify-an-identifier⑦④"></a>

    If the value is auto, [reify an identifier](#reify-an-identifier) from the value and return the result.

2.  <a id="ref-for-length-value⑧"></a>

    <a id="ref-for-percentage-value①①"></a>

    <a id="ref-for-reify-a-numeric-value②①"></a>

    If the value is a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), [reify a numeric value](#reify-a-numeric-value) from the value and return the result.

<a id="ref-for-propdef-will-change"></a>

[will-change](https://www.w3.org/TR/css-will-change-1/#propdef-will-change)

<a id="ref-for-propdef-word-spacing"></a>

[word-spacing](https://www.w3.org/TR/css-text-4/#propdef-word-spacing)

<a id="ref-for-propdef-writing-mode"></a>

[writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

<a id="ref-for-XProperty"></a>

[x](https://www.w3.org/TR/SVG2/geometry.html#XProperty)

<a id="ref-for-YProperty"></a>

[y](https://www.w3.org/TR/SVG2/geometry.html#YProperty)

<a id="ref-for-propdef-z-index①"></a>

[z-index](https://www.w3.org/TR/CSS21/visuren.html#propdef-z-index)

### <a id="reify-failure"></a>5.2. Unrepresentable Values

<a id="ref-for-css-internal-representation①④"></a>

<a id="ref-for-css-reify⑦"></a>

<a id="ref-for-cssstylevalue⑥③"></a>

<a id="ref-for-reify-as-a-cssstylevalue"></a>

Not all [internal representations](#css-internal-representation) are simple enough to be [reified](#css-reify) with the current set of <code><a href="#cssstylevalue">CSSStyleValue</a></code> subclasses. When this is the case, the property is [reified as a CSSStyleValue](#reify-as-a-cssstylevalue) for a particular property, ensuring that it can be used as a value for that property, and nothing else.

To <a id="reify-as-a-cssstylevalue"></a>reify as a CSSStyleValue a <var>value</var> for a <var>property</var>:

1.  <a id="ref-for-cssstylevalue⑥④"></a>

    <a id="ref-for-dom-cssstylevalue-associatedproperty-slot⑤"></a>

    Return a new <code><a href="#cssstylevalue">CSSStyleValue</a></code> object representing <var>value</var> whose <code><a href="#dom-cssstylevalue-associatedproperty-slot">&#x5B;&#x5B;associatedProperty&#x5D;&#x5D;</a></code> internal slot is set to <var>property</var>.

<a id="ref-for-funcdef-var⑥"></a>

### <a id="reify-tokens"></a>5.3. Raw CSS tokens: properties with [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) references

<a id="ref-for-funcdef-var⑦"></a>

<a id="ref-for-list①⑤"></a>

<a id="ref-for-component-value①"></a>

<a id="ref-for-cssunparsedvalue①②"></a>

Regardless of what the property’s grammar is otherwise, a property value with an un-substituted [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) reference is represented as a [list](https://infra.spec.whatwg.org/#list) of [component values](https://www.w3.org/TR/css-syntax-3/#component-value), which becomes a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> in the Typed OM.

To <a id="reify-a-list-of-component-values"></a>reify a list of component values from a <var>list</var>:

1.  <a id="ref-for-funcdef-var⑧"></a>

    <a id="ref-for-cssvariablereferencevalue⑦"></a>

    Replace all [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) references in <var>list</var> with <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> objects, as described in [§ 5.4 var() References](#reify-var).

2.  <a id="ref-for-component-value②"></a>

    Replace each remaining maximal subsequence of [component values](https://www.w3.org/TR/css-syntax-3/#component-value) in <var>list</var> with a single string of their concatenated serializations.

3.  <a id="ref-for-cssunparsedvalue①③"></a>

    <a id="ref-for-dom-cssunparsedvalue-tokens-slot⑤"></a>

    Return a new <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> whose <code><a href="#dom-cssunparsedvalue-tokens-slot">&#x5B;&#x5B;tokens&#x5D;&#x5D;</a></code> slot is set to <var>list</var>.

<a id="ref-for-cssunparsedvalue①④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5e9f60fb"></a> The string "calc(42px + var(--foo, 15em) + var(--bar, var(--far) + 15px))" is converted into a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> that contains a sequence with:
>
> - the string "calc(42px + "
>
> - <a id="ref-for-cssvariablereferencevalue⑧"></a>
>
>   a <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> with:
>
>   - <a id="ref-for-dom-cssvariablereferencevalue-variable⑤"></a>
>
>     <code><a href="#dom-cssvariablereferencevalue-variable">variable</a></code> "--foo"
>
>   - <a id="ref-for-dom-cssvariablereferencevalue-fallback①"></a>
>
>     <a id="ref-for-cssunparsedvalue①⑤"></a>
>
>     <code><a href="#dom-cssvariablereferencevalue-fallback">fallback</a></code> a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> with a single-valued sequence containing " 15em"
>
> - the string " + "
>
> - <a id="ref-for-cssvariablereferencevalue⑨"></a>
>
>   a <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> with:
>
>   - <a id="ref-for-dom-cssvariablereferencevalue-variable⑥"></a>
>
>     <code><a href="#dom-cssvariablereferencevalue-variable">variable</a></code> "--bar"
>
>   - <a id="ref-for-dom-cssvariablereferencevalue-fallback②"></a>
>
>     <a id="ref-for-cssunparsedvalue①⑥"></a>
>
>     <code><a href="#dom-cssvariablereferencevalue-fallback">fallback</a></code> a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> with a sequence containing:
>
>     - the string " "
>
>     - <a id="ref-for-cssvariablereferencevalue①⓪"></a>
>
>       a <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> with
>
>       - <a id="ref-for-dom-cssvariablereferencevalue-variable⑦"></a>
>
>         <code><a href="#dom-cssvariablereferencevalue-variable">variable</a></code> "--far"
>
>       - <a id="ref-for-dom-cssvariablereferencevalue-fallback③"></a>
>
>         <code><a href="#dom-cssvariablereferencevalue-fallback">fallback</a></code> null
>
>     - the string " + 15px"
>
> - the string ")"

<a id="ref-for-funcdef-var⑨"></a>

### <a id="reify-var"></a>5.4. [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) References

<a id="ref-for-funcdef-var①⓪"></a>

<a id="ref-for-cssvariablereferencevalue①①"></a>

[var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) references become <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code>s in the Typed OM.

<a id="ref-for-funcdef-var①①"></a>

To <a id="reify-a-var-reference"></a>reify a [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) reference <var>var</var>:

1.  <a id="ref-for-cssvariablereferencevalue①②"></a>

    Let <var>object</var> be a new <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code>.

2.  <a id="ref-for-dom-cssvariablereferencevalue-variable⑧"></a>

    <a id="ref-for-identifier-value"></a>

    Set <var>object</var>’s <code><a href="#dom-cssvariablereferencevalue-variable">variable</a></code> internal slot to the serialization of the [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) providing the variable name.

3.  <a id="ref-for-dom-cssvariablereferencevalue-fallback④"></a>

    <a id="ref-for-reify-a-list-of-component-values②"></a>

    If <var>var</var> has a fallback value, set <var>object</var>’s <code><a href="#dom-cssvariablereferencevalue-fallback">fallback</a></code> internal slot to the result of [reifying the fallback’s component values](#reify-a-list-of-component-values). Otherwise, set it to `null`.

4.  Return <var>object</var>.

<a id="ref-for-css-css-identifier②"></a>

### <a id="reify-ident"></a>5.5. [Identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier) Values

<a id="ref-for-css-css-identifier③"></a>

<a id="ref-for-csskeywordvalue①④"></a>

CSS [identifiers](https://www.w3.org/TR/css-values-4/#css-css-identifier) become <code><a href="#csskeywordvalue">CSSKeywordValue</a></code>s in the Typed OM.

<a id="ref-for-css-css-identifier④"></a>

To <a id="reify-an-identifier"></a>reify an [identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier) <var>ident</var>:

1.  <a id="ref-for-csskeywordvalue①⑤"></a>

    <a id="ref-for-dom-csskeywordvalue-value⑨"></a>

    Return a new <code><a href="#csskeywordvalue">CSSKeywordValue</a></code> with its <code><a href="#dom-csskeywordvalue-value">value</a></code> internal slot set to the serialization of <var>ident</var>.

<a id="ref-for-number-value①⓪"></a>

<a id="ref-for-percentage-value①②"></a>

<a id="ref-for-typedef-dimension②"></a>

### <a id="reify-numeric"></a>5.6. [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), and [\<dimension\>](https://www.w3.org/TR/css-values-4/#typedef-dimension) values

<a id="ref-for-number-value①①"></a>

<a id="ref-for-percentage-value①③"></a>

<a id="ref-for-typedef-dimension③"></a>

<a id="ref-for-cssnumericvalue⑦⑤"></a>

CSS [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), and [\<dimension\>](https://www.w3.org/TR/css-values-4/#typedef-dimension) values become <code><a href="#cssnumericvalue">CSSNumericValue</a></code>s in the Typed OM.

To <a id="reify-a-numeric-value"></a>reify a numeric value <var>num</var>:

1.  <a id="ref-for-math-function②"></a>

    <a id="ref-for-reify-a-math-expression"></a>

    If <var>num</var> is a [math function](https://www.w3.org/TR/css-values-4/#math-function), [reify a math expression](#reify-a-math-expression) from <var>num</var> and return the result.

2.  <a id="ref-for-typedef-dimension④"></a>

    <a id="ref-for-cssunitvalue⑨⑨"></a>

    <a id="ref-for-dom-cssunitvalue-value②⑥"></a>

    <a id="ref-for-dom-cssunitvalue-unit③⓪"></a>

    If <var>num</var> is the unitless value 0 and <var>num</var> is a [\<dimension\>](https://www.w3.org/TR/css-values-4/#typedef-dimension), return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> with its <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot set to 0, and its <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot set to "px".

3.  <a id="ref-for-cssunitvalue①⓪⓪"></a>

    <a id="ref-for-dom-cssunitvalue-value②⑦"></a>

    <a id="ref-for-dom-cssunitvalue-unit③①"></a>

    <a id="ref-for-number-value①②"></a>

    <a id="ref-for-percentage-value①④"></a>

    <a id="ref-for-typedef-dimension⑤"></a>

    Return a new <code><a href="#cssunitvalue">CSSUnitValue</a></code> with its <code><a href="#dom-cssunitvalue-value">value</a></code> internal slot set to the numeric value of <var>num</var>, and its <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slot set to "number" if <var>num</var> is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), "percent" if <var>num</var> is a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), and <var>num</var>’s unit if <var>num</var> is a [\<dimension\>](https://www.w3.org/TR/css-values-4/#typedef-dimension).

    <a id="ref-for-css-reify⑧"></a>

    <a id="ref-for-canonical-unit④"></a>

    If the value being [reified](#css-reify) is a computed value, the unit used must be the appropriate [canonical unit](https://www.w3.org/TR/css-values-4/#canonical-unit) for the value’s type, with the numeric value scaled accordingly.

    <a id="ref-for-px④"></a>

    <a id="ref-for-canonical-unit⑤"></a>

    > <strong data-conversion-semantic="example">Example</strong>
    >
    > <a id="example-b5c7e2e8"></a> For example, if an element has `style="width: 1in;"`, `el.attributeStyleMap.get('width')` will return `CSS.in(1)`, but `el.computedStyleMap.get('width')` will return `CSS.px(96)`, as [px](https://www.w3.org/TR/css-values-4/#px) is the [canonical unit](https://www.w3.org/TR/css-values-4/#canonical-unit) for absolute lengths.

To <a id="reify-a-math-expression"></a>reify a math expression <var>num</var>:

1.  <a id="ref-for-funcdef-min①"></a>

    <a id="ref-for-funcdef-max①"></a>

    If <var>num</var> is a [min()](https://www.w3.org/TR/css-values-4/#funcdef-min) or [max()](https://www.w3.org/TR/css-values-4/#funcdef-max) expression:

    1.  <a id="ref-for-reify-a-math-expression①"></a>

        <a id="ref-for-funcdef-calc①"></a>

        Let <var>values</var> be the result of [reifying](#reify-a-math-expression) the arguments to the expression, treating each argument as if it were the contents of a [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) expression.

    2.  <a id="ref-for-cssmathmin⑦"></a>

        <a id="ref-for-cssmathmax⑦"></a>

        <a id="ref-for-dom-cssmathmin-values③"></a>

        Return a new <code><a href="#cssmathmin">CSSMathMin</a></code> or <code><a href="#cssmathmax">CSSMathMax</a></code> object, respectively, with its <code><a href="#dom-cssmathmin-values">values</a></code> internal slot set to <var>values</var>.

2.  <a id="ref-for-funcdef-calc②"></a>

    Assert: Otherwise, <var>num</var> is a [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc).

3.  Turn <var>num</var>’s argument into an expression tree using standard PEMDAS precedence rules, with the following exceptions/clarification:

    - Treat subtraction as instead being addition, with the RHS argument instead wrapped in a special "negate" node.

    - Treat division as instead being multiplication, with the RHS argument instead wrapped in a special "invert" node.

    - Addition and multiplication are N-ary; each node can have any number of arguments.

    - If an expression has only a single value in it, and no operation, treat it as an addition node with the single argument.

4.  Recursively transform the expression tree into objects, as follows:

    addition node  
    <a id="ref-for-cssmathsum①③"></a>

    <a id="ref-for-dom-cssmathsum-values①②"></a>

    becomes a new <code><a href="#cssmathsum">CSSMathSum</a></code> object, with its <code><a href="#dom-cssmathsum-values">values</a></code> internal slot set to its list of arguments

    multiplication node  
    <a id="ref-for-cssmathproduct⑦"></a>

    <a id="ref-for-dom-cssmathproduct-values④"></a>

    becomes a new <code><a href="#cssmathproduct">CSSMathProduct</a></code> object, with its <code><a href="#dom-cssmathproduct-values">values</a></code> internal slot set to its list of arguments

    negate node  
    <a id="ref-for-cssmathnegate⑦"></a>

    <a id="ref-for-dom-cssmathnegate-value⑦"></a>

    becomes a new <code><a href="#cssmathnegate">CSSMathNegate</a></code> object, with its <code><a href="#dom-cssmathnegate-value">value</a></code> internal slot set to its argument

    invert node  
    <a id="ref-for-cssmathinvert⑦"></a>

    <a id="ref-for-dom-cssmathinvert-value④"></a>

    becomes a new <code><a href="#cssmathinvert">CSSMathInvert</a></code> object, with its <code><a href="#dom-cssmathinvert-value">value</a></code> internal slot set to its argument

    leaf node  
    <a id="ref-for-css-reify⑨"></a>

    [reified](#css-reify) as appropriate

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-585bdcaa"></a> For example, calc(1px - 2 \* 3em) produces the structure:
>
> ```text
> CSSMathSum(
>     CSS.px(1),
>     CSSMathNegate(
>         CSSMathProduct(
>             2,
>             CSS.em(3)
>         )
>     )
> )
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-41a5f864"></a> Note that addition and multiplication are N-ary, so calc(1px + 2px + 3px) produces the structure:
>
> ```text
> CSSMathSum(
>     CSS.px(1),
>     CSS.px(2),
>     CSS.px(3)
> )
> ```
>
> but calc(calc(1px + 2px) + 3px) produces the structure:
>
> ```text
> CSSMathSum(
>     CSSMathSum(
>         CSS.px(1),
>         CSS.px(2)
>     ),
>     CSS.px(3)
> )
> ```
<a id="ref-for-em"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The value computation process may transform different units into identical ones, simplifying the resulting expression. For example, calc(1px + 2em) as a specified value results in a `CSSMathSum(CSS.px(1), CSS.em(2))`, but as a computed value will give `CSS.px(33)` or similar (depending on the value of an [em](https://www.w3.org/TR/css-values-4/#em) in that context).

<a id="ref-for-typedef-color②"></a>

### <a id="reify-color"></a>5.7. [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) Values

<a id="ref-for-typedef-color③"></a>

<a id="ref-for-csscolorvalue①②"></a>

<a id="ref-for-cssstylevalue⑥⑤"></a>

CSS [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) values become either <code><a href="#csscolorvalue">CSSColorValue</a></code>s (if they can be resolved to an absolute color) or generic <code><a href="#cssstylevalue">CSSStyleValue</a></code>s (otherwise).

To <a id="reify-a-color-value"></a>reify a color value <var>val</var>:

1.  <a id="ref-for-typedef-hex-color"></a>

    <a id="ref-for-funcdef-rgb①"></a>

    <a id="ref-for-funcdef-rgba①"></a>

    <a id="ref-for-cssrgb④"></a>

    <a id="ref-for-dom-cssrgb-r②"></a>

    <a id="ref-for-dom-cssrgb-g②"></a>

    <a id="ref-for-dom-cssrgb-b②"></a>

    <a id="ref-for-dom-cssrgb-alpha②"></a>

    <a id="ref-for-css-reify①⓪"></a>

    If <var>val</var> is a [\<hex-color\>](https://www.w3.org/TR/css-color-4/#typedef-hex-color), an [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb) function, or an [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba) function, then return a new <code><a href="#cssrgb">CSSRGB</a></code> object with its <code><a href="#dom-cssrgb-r">r</a></code>, <code><a href="#dom-cssrgb-g">g</a></code>, <code><a href="#dom-cssrgb-b">b</a></code>, and <code><a href="#dom-cssrgb-alpha">alpha</a></code> internal slots set to the [reification](#css-reify) of its red, green, blue, and alpha components, respectively.

2.  <a id="ref-for-funcdef-hsl①"></a>

    <a id="ref-for-funcdef-hsla①"></a>

    <a id="ref-for-csshsl④"></a>

    <a id="ref-for-dom-csshsl-h②"></a>

    <a id="ref-for-dom-csshsl-s②"></a>

    <a id="ref-for-dom-csshsl-l②"></a>

    <a id="ref-for-dom-csshsl-alpha②"></a>

    <a id="ref-for-css-reify①①"></a>

    If <var>val</var> is an [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) or [hsla()](https://www.w3.org/TR/css-color-4/#funcdef-hsla) function, then return a new <code><a href="#csshsl">CSSHSL</a></code> object with its <code><a href="#dom-csshsl-h">h</a></code>, <code><a href="#dom-csshsl-s">s</a></code>, <code><a href="#dom-csshsl-l">l</a></code>, and <code><a href="#dom-csshsl-alpha">alpha</a></code> internal slots set to the [reification](#css-reify) of its hue angle, saturation, lightness, and alpha components, respectively.

3.  <a id="ref-for-funcdef-hwb①"></a>

    <a id="ref-for-csshwb④"></a>

    <a id="ref-for-dom-csshwb-h②"></a>

    <a id="ref-for-dom-csshwb-w②"></a>

    <a id="ref-for-dom-csshwb-b②"></a>

    <a id="ref-for-dom-csshwb-alpha②"></a>

    <a id="ref-for-css-reify①②"></a>

    If <var>val</var> is an [hwb()](https://www.w3.org/TR/css-color-5/#funcdef-hwb) function, then return a new <code><a href="#csshwb">CSSHWB</a></code> object with its <code><a href="#dom-csshwb-h">h</a></code>, <code><a href="#dom-csshwb-w">w</a></code>, <code><a href="#dom-csshwb-b">b</a></code>, and <code><a href="#dom-csshwb-alpha">alpha</a></code> internal slots set to the [reification](#css-reify) of its hue angle, whiteness, blackness, and alpha components, respectively.

4.  <a id="ref-for-funcdef-lch②"></a>

    <a id="ref-for-csslch④"></a>

    <a id="ref-for-dom-csslch-l②"></a>

    <a id="ref-for-dom-csslch-c②"></a>

    <a id="ref-for-dom-csslch-h②"></a>

    <a id="ref-for-dom-csslch-alpha②"></a>

    <a id="ref-for-css-reify①③"></a>

    If <var>val</var> is an [lch()](https://www.w3.org/TR/css-color-5/#funcdef-lch) function, then return a new <code><a href="#csslch">CSSLCH</a></code> object with its <code><a href="#dom-csslch-l">l</a></code>, <code><a href="#dom-csslch-c">c</a></code>, <code><a href="#dom-csslch-h">h</a></code>, and <code><a href="#dom-csslch-alpha">alpha</a></code> internal slots set to the [reification](#css-reify) of its lightness, chroma, hue angle, and alpha components, respectively.

5.  <a id="ref-for-funcdef-lab①"></a>

    <a id="ref-for-csslab④"></a>

    <a id="ref-for-dom-csslab-l②"></a>

    <a id="ref-for-dom-csslab-a②"></a>

    <a id="ref-for-dom-csslab-b②"></a>

    <a id="ref-for-dom-csslab-alpha②"></a>

    <a id="ref-for-css-reify①④"></a>

    If <var>val</var> is an [lab()](https://www.w3.org/TR/css-color-5/#funcdef-lab) function, then return a new <code><a href="#csslab">CSSLab</a></code> object with its <code><a href="#dom-csslab-l">l</a></code>, <code><a href="#dom-csslab-a">a</a></code>, <code><a href="#dom-csslab-b">b</a></code>, and <code><a href="#dom-csslab-alpha">alpha</a></code> internal slots set to the [reification](#css-reify) of its lightness, a, b, and alpha components, respectively.

6.  <a id="ref-for-funcdef-color①"></a>

    <a id="ref-for-csscolor⑥"></a>

    <a id="ref-for-dom-csscolor-colorspace②"></a>

    <a id="ref-for-reify-an-identifier⑦⑤"></a>

    <a id="ref-for-dom-csscolor-channels⑤"></a>

    <a id="ref-for-observable-array-attribute-backing-list②"></a>

    <a id="ref-for-css-reify①⑤"></a>

    <a id="ref-for-dom-csscolor-alpha②"></a>

    <a id="ref-for-css-reify①⑥"></a>

    If <var>val</var> is a [color()](https://www.w3.org/TR/css-color-4/#funcdef-color) function, then return a new <code><a href="#csscolor">CSSColor</a></code> object with its <code><a href="#dom-csscolor-colorspace">colorSpace</a></code> internal slot set to the result of [reifying an identifier](#reify-an-identifier) from <var>val</var>’s color space, its <code><a href="#dom-csscolor-channels">channels</a></code> internal slot’s [backing list](https://webidl.spec.whatwg.org/#observable-array-attribute-backing-list) set to the result of [reifying](#css-reify) <var>val</var>’s list of non-alpha components, and <code><a href="#dom-csscolor-alpha">alpha</a></code> internal slot set to the result of [reifying](#css-reify) <var>val</var>’s alpha component.

7.  <a id="ref-for-typedef-named-color"></a>

    <a id="ref-for-valdef-color-transparent"></a>

    <a id="ref-for-cssrgb⑤"></a>

    <a id="ref-for-dom-cssrgb-r③"></a>

    <a id="ref-for-dom-cssrgb-g③"></a>

    <a id="ref-for-dom-cssrgb-b③"></a>

    <a id="ref-for-dom-cssrgb-alpha③"></a>

    <a id="ref-for-css-reify①⑦"></a>

    If <var>val</var> is a [\<named-color\>](https://www.w3.org/TR/css-color-4/#typedef-named-color) or the keyword [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent), then return a new <code><a href="#cssrgb">CSSRGB</a></code> object with its <code><a href="#dom-cssrgb-r">r</a></code>, <code><a href="#dom-cssrgb-g">g</a></code>, <code><a href="#dom-cssrgb-b">b</a></code>, and <code><a href="#dom-cssrgb-alpha">alpha</a></code> internal slots set to the [reification](#css-reify) of its red, green, blue, and alpha components, respectively.

8.  <a id="ref-for-reify-an-identifier⑦⑥"></a>

    If <var>val</var> is any other color keyword, return the result of [reifying an identifier](#reify-an-identifier) from <var>val</var>.

<a id="ref-for-typedef-transform-list②"></a>

<a id="ref-for-typedef-transform-function①"></a>

### <a id="reify-transformvalue"></a>5.8. [\<transform-list\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-list) and [\<transform-function\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-function) Values

<a id="ref-for-typedef-transform-list③"></a>

<a id="ref-for-csstransformvalue①②"></a>

<a id="ref-for-typedef-transform-function②"></a>

<a id="ref-for-csstransformcomponent①⑨"></a>

CSS [\<transform-list\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-list) values become <code><a href="#csstransformvalue">CSSTransformValue</a></code>s in the Typed OM, while CSS [\<transform-function\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-function) values become <code><a href="#csstransformcomponent">CSSTransformComponent</a></code>s.

<a id="ref-for-typedef-transform-list④"></a>

To <a id="reify-a-transform-list"></a>reify a [\<transform-list\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-list) <var>list</var>:

1.  <a id="ref-for-csstransformvalue①③"></a>

    <a id="ref-for-reify-a-transform-function"></a>

    Return a new <code><a href="#csstransformvalue">CSSTransformValue</a></code> whose values to iterate over are the result of mapping the [reify a \<transform-function\>](#reify-a-transform-function) algorithm over <var>list</var>.

<a id="ref-for-typedef-transform-function③"></a>

To <a id="reify-a-transform-function"></a>reify a [\<transform-function\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-function) <var>func</var>, perform the appropriate set of steps below, based on <var>func</var>:

<a id="ref-for-funcdef-transform-matrix"></a>

[matrix()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-matrix)

<a id="ref-for-funcdef-matrix3d"></a>

[matrix3d()](https://www.w3.org/TR/css-transforms-2/#funcdef-matrix3d)

1.  <a id="ref-for-cssmatrixcomponent①"></a>

    <a id="ref-for-dom-cssmatrixcomponent-matrix①"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d①⑥"></a>

    <a id="ref-for-funcdef-transform-matrix①"></a>

    Return a new <code><a href="#cssmatrixcomponent">CSSMatrixComponent</a></code> object, whose <code><a href="#dom-cssmatrixcomponent-matrix">matrix</a></code> internal slot is set to a 4x4 matrix representing the same information as <var>func</var>, and whose <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot is `true` if <var>func</var> is [matrix()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-matrix), and `false` otherwise.

<a id="ref-for-funcdef-transform-translate"></a>

[translate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translate)

<a id="ref-for-funcdef-transform-translatex"></a>

[translateX()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translatex)

<a id="ref-for-funcdef-transform-translatey"></a>

[translateY()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translatey)

<a id="ref-for-funcdef-translate3d"></a>

[translate3d()](https://www.w3.org/TR/css-transforms-2/#funcdef-translate3d)

<a id="ref-for-funcdef-translatez"></a>

[translateZ()](https://www.w3.org/TR/css-transforms-2/#funcdef-translatez)

1.  <a id="ref-for-csstranslate②"></a>

    <a id="ref-for-dom-csstranslate-x①"></a>

    <a id="ref-for-dom-csstranslate-y①"></a>

    <a id="ref-for-dom-csstranslate-z③"></a>

    <a id="ref-for-reify-a-numeric-value②②"></a>

    <a id="ref-for-reify-a-numeric-value②③"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d①⑦"></a>

    <a id="ref-for-funcdef-transform-translate①"></a>

    <a id="ref-for-funcdef-transform-translatex①"></a>

    <a id="ref-for-funcdef-transform-translatey①"></a>

    Return a new <code><a href="#csstranslate">CSSTranslate</a></code> object, whose <code><a href="#dom-csstranslate-x">x</a></code>, <code><a href="#dom-csstranslate-y">y</a></code>, and <code><a href="#dom-csstranslate-z">z</a></code> internal slots are set to the [reification](#reify-a-numeric-value) of the specified x/y/z offsets, or the [reification](#reify-a-numeric-value) of 0px if not specified in <var>func</var>, and whose <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot is `true` if <var>func</var> is [translate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translate), [translateX()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translatex), or [translateY()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translatey), and `false` otherwise.

<a id="ref-for-funcdef-scale"></a>

[scale()](https://www.w3.org/TR/css-transforms-2/#funcdef-scale)

<a id="ref-for-funcdef-scalex"></a>

[scaleX()](https://www.w3.org/TR/css-transforms-2/#funcdef-scalex)

<a id="ref-for-funcdef-scaley"></a>

[scaleY()](https://www.w3.org/TR/css-transforms-2/#funcdef-scaley)

<a id="ref-for-funcdef-scale3d"></a>

[scale3d()](https://www.w3.org/TR/css-transforms-2/#funcdef-scale3d)

<a id="ref-for-funcdef-scalez"></a>

[scaleZ()](https://www.w3.org/TR/css-transforms-2/#funcdef-scalez)

1.  <a id="ref-for-cssscale①"></a>

    <a id="ref-for-dom-cssscale-x②"></a>

    <a id="ref-for-dom-cssscale-y②"></a>

    <a id="ref-for-dom-cssscale-z③"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d①⑧"></a>

    <a id="ref-for-funcdef-scale①"></a>

    <a id="ref-for-funcdef-scalex①"></a>

    <a id="ref-for-funcdef-scaley①"></a>

    Return a new <code><a href="#cssscale">CSSScale</a></code> object, whose <code><a href="#dom-cssscale-x">x</a></code>, <code><a href="#dom-cssscale-y">y</a></code>, and <code><a href="#dom-cssscale-z">z</a></code> internal slots are set to the specified x/y/z scales, or to 1 if not specified in <var>func</var> and whose <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot is `true` if <var>func</var> is [scale()](https://www.w3.org/TR/css-transforms-2/#funcdef-scale), [scaleX()](https://www.w3.org/TR/css-transforms-2/#funcdef-scalex), or [scaleY()](https://www.w3.org/TR/css-transforms-2/#funcdef-scaley), and `false` otherwise.

<a id="ref-for-funcdef-transform-rotate"></a>

[rotate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-rotate)

<a id="ref-for-funcdef-rotate3d"></a>

[rotate3d()](https://www.w3.org/TR/css-transforms-2/#funcdef-rotate3d)

<a id="ref-for-funcdef-rotatex"></a>

[rotateX()](https://www.w3.org/TR/css-transforms-2/#funcdef-rotatex)

<a id="ref-for-funcdef-rotatey"></a>

[rotateY()](https://www.w3.org/TR/css-transforms-2/#funcdef-rotatey)

<a id="ref-for-funcdef-rotatez"></a>

[rotateZ()](https://www.w3.org/TR/css-transforms-2/#funcdef-rotatez)

1.  <a id="ref-for-cssrotate②"></a>

    <a id="ref-for-dom-cssrotate-angle②"></a>

    <a id="ref-for-reify-a-numeric-value②④"></a>

    <a id="ref-for-dom-cssrotate-x③"></a>

    <a id="ref-for-dom-cssrotate-y③"></a>

    <a id="ref-for-dom-cssrotate-z③"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d①⑨"></a>

    <a id="ref-for-funcdef-transform-rotate①"></a>

    Return a new <code><a href="#cssrotate">CSSRotate</a></code> object, whose <code><a href="#dom-cssrotate-angle">angle</a></code> internal slot is set to the [reification](#reify-a-numeric-value) of the specified angle, and whose <code><a href="#dom-cssrotate-x">x</a></code>, <code><a href="#dom-cssrotate-y">y</a></code>, and <code><a href="#dom-cssrotate-z">z</a></code> internal slots are set to the specified rotation axis coordinates, or the implicit axis coordinates if not specified in <var>func</var> and whose <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot is `true` if <var>func</var> is [rotate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-rotate), and `false` otherwise.

<a id="ref-for-funcdef-transform-skew①"></a>

[skew()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-skew)

1.  <a id="ref-for-cssskew②"></a>

    <a id="ref-for-dom-cssskew-ax①"></a>

    <a id="ref-for-dom-cssskew-ay①"></a>

    <a id="ref-for-reify-a-numeric-value②⑤"></a>

    <a id="ref-for-reify-a-numeric-value②⑥"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d②⓪"></a>

    Return a new <code><a href="#cssskew">CSSSkew</a></code> object, whose <code><a href="#dom-cssskew-ax">ax</a></code> and <code><a href="#dom-cssskew-ay">ay</a></code> internal slots are set to the [reification](#reify-a-numeric-value) of the specified x and y angles, or the [reification](#reify-a-numeric-value) of 0deg if not specified in <var>func</var>, and whose <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot is `true`.

<a id="ref-for-funcdef-transform-skewx①"></a>

[skewX()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-skewx)

1.  <a id="ref-for-cssskewx②"></a>

    <a id="ref-for-dom-cssskewx-ax①"></a>

    <a id="ref-for-reify-a-numeric-value②⑦"></a>

    <a id="ref-for-reify-a-numeric-value②⑧"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d②①"></a>

    Return a new <code><a href="#cssskewx">CSSSkewX</a></code> object, whose <code><a href="#dom-cssskewx-ax">ax</a></code> internal slot is set to the [reification](#reify-a-numeric-value) of the specified x angle, or the [reification](#reify-a-numeric-value) of 0deg if not specified in <var>func</var>, and whose <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot is `true`.

<a id="ref-for-funcdef-transform-skewy①"></a>

[skewY()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-skewy)

1.  <a id="ref-for-cssskewy②"></a>

    <a id="ref-for-dom-cssskewy-ay①"></a>

    <a id="ref-for-reify-a-numeric-value②⑨"></a>

    <a id="ref-for-reify-a-numeric-value③⓪"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d②②"></a>

    Return a new <code><a href="#cssskewy">CSSSkewY</a></code> object, whose <code><a href="#dom-cssskewy-ay">ay</a></code> internal slot is set to the [reification](#reify-a-numeric-value) of the specified y angle, or the [reification](#reify-a-numeric-value) of 0deg if not specified in <var>func</var>, and whose <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot is `true`.

<a id="ref-for-funcdef-perspective①"></a>

[perspective()](https://www.w3.org/TR/css-transforms-2/#funcdef-perspective)

1.  <a id="ref-for-cssperspective②"></a>

    <a id="ref-for-dom-cssperspective-length①"></a>

    <a id="ref-for-reify-a-numeric-value③①"></a>

    <a id="ref-for-reify-an-identifier⑦⑦"></a>

    <a id="ref-for-valdef-perspective-func-none①"></a>

    <a id="ref-for-dom-csstransformcomponent-is2d②③"></a>

    Return a new <code><a href="#cssperspective">CSSPerspective</a></code> object, whose <code><a href="#dom-cssperspective-length">length</a></code> internal slot is set to the reification of the specified length (see [reify a numeric value](#reify-a-numeric-value) if it is a length, and [reify an identifier](#reify-an-identifier) if it is the keyword [none](https://www.w3.org/TR/css-transforms-2/#valdef-perspective-func-none)) and whose <code><a href="#dom-csstransformcomponent-is2d">is2D</a></code> internal slot is `false`.

<a id="ref-for-cssstylevalue⑥⑥"></a>

## <a id="stylevalue-serialization"></a>6. <code><a href="#cssstylevalue">CSSStyleValue</a></code> Serialization

<a id="ref-for-cssstylevalue⑥⑦"></a>

The way that a <code><a href="#cssstylevalue">CSSStyleValue</a></code> serializes is dependent on how the value was constructed.

if the value was constructed from a USVString  
the serialization is the USVString from which the value was constructed.

otherwise, if the value was constructed using an IDL constructor  
the serialization is specified in the sections below.

otherwise, if the value was extracted from the CSSOM  
the serialization is specified in [§ 6.7 Serialization from CSSOM Values](#cssom-serialization) below.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5d6cbad2"></a>
>
> For example:
>
> ```javascript
> var length1 = CSSNumericValue.parse("42.0px");
> length1.toString(); // "42.0px"
> 
> var length2 = CSS.px(42.0);
> length2.toString(); // "42px";
> 
> element.style.width = "42.0px";
> var length3 = element.attributeStyleMap.get('width');
> length3.toString(); // "42px";
> ```
<a id="ref-for-cssunparsedvalue①⑦"></a>

### <a id="unparsedvalue-serialization"></a>6.1. <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> Serialization

<a id="ref-for-cssunparsedvalue①⑧"></a>

To <a id="serialize-a-cssunparsedvalue"></a>serialize a <code><a href="#cssunparsedvalue">CSSUnparsedValue</a></code> <var>this</var>:

1.  <a id="ref-for-string⑨"></a>

    Let <var>s</var> initially be the empty [string](https://infra.spec.whatwg.org/#string).

2.  <a id="ref-for-list-iterate①④"></a>

    <a id="ref-for-dom-cssunparsedvalue-tokens-slot⑥"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>item</var> in <var>this</var>’s <code><a href="#dom-cssunparsedvalue-tokens-slot">&#x5B;&#x5B;tokens&#x5D;&#x5D;</a></code> internal slot:

    1.  <a id="ref-for-idl-USVString②⑦"></a>

        If <var>item</var> is a <code><a href="https://webidl.spec.whatwg.org/#idl-USVString">USVString</a></code>, append it to <var>s</var>.

    2.  <a id="ref-for-cssvariablereferencevalue①③"></a>

        Otherwise, <var>item</var> is a <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code>. Serialize it, then append the result to <var>s</var>.

3.  Return <var>s</var>.

<a id="ref-for-cssvariablereferencevalue①④"></a>

To <a id="serialize-a-cssvariablereferencevalue"></a>serialize a <code><a href="#cssvariablereferencevalue">CSSVariableReferenceValue</a></code> <var>this</var>:

1.  Let <var>s</var> initally be "var(".

2.  <a id="ref-for-dom-cssvariablereferencevalue-variable⑨"></a>

    Append <var>this</var>’s <code><a href="#dom-cssvariablereferencevalue-variable">variable</a></code> internal slot to <var>s</var>.

3.  <a id="ref-for-dom-cssvariablereferencevalue-fallback⑤"></a>

    <a id="ref-for-dom-cssvariablereferencevalue-fallback⑥"></a>

    If <var>this</var>’s <code><a href="#dom-cssvariablereferencevalue-fallback">fallback</a></code> internal slot is not `null`, append ", " to <var>s</var>, then serialize the <code><a href="#dom-cssvariablereferencevalue-fallback">fallback</a></code> internal slot and append it to <var>s</var>.

4.  Append ")" to <var>s</var> and return <var>s</var>.

<a id="ref-for-csskeywordvalue①⑥"></a>

### <a id="keywordvalue-serialization"></a>6.2. <code><a href="#csskeywordvalue">CSSKeywordValue</a></code> Serialization

<a id="ref-for-csskeywordvalue①⑦"></a>

To <a id="serialize-a-csskeywordvalue"></a>serialize a <code><a href="#csskeywordvalue">CSSKeywordValue</a></code> <var>this</var>:

1.  <a id="ref-for-dom-csskeywordvalue-value①⓪"></a>

    Return <var>this</var>’s <code><a href="#dom-csskeywordvalue-value">value</a></code> internal slot.

<a id="ref-for-cssnumericvalue⑦⑥"></a>

### <a id="numericvalue-serialization"></a>6.3. <code><a href="#cssnumericvalue">CSSNumericValue</a></code> Serialization

<a id="ref-for-cssnumericvalue⑦⑦"></a>

To <a id="serialize-a-cssnumericvalue"></a>serialize a <code><a href="#cssnumericvalue">CSSNumericValue</a></code> <var>this</var>, given an optional <var>minimum</var>, a numeric value, and optional <var>maximum</var>, a numeric value:

1.  <a id="ref-for-cssunitvalue①⓪①"></a>

    <a id="ref-for-serialize-a-cssunitvalue"></a>

    If <var>this</var> is a <code><a href="#cssunitvalue">CSSUnitValue</a></code>, [serialize a CSSUnitValue](#serialize-a-cssunitvalue) from <var>this</var>, passing <var>minimum</var> and <var>maximum</var>. Return the result.

2.  <a id="ref-for-serialize-a-cssmathvalue"></a>

    Otherwise, [serialize a CSSMathValue](#serialize-a-cssmathvalue) from <var>this</var>, and return the result.

<a id="ref-for-cssunitvalue①⓪②"></a>

### <a id="unitvalue-serialization"></a>6.4. <code><a href="#cssunitvalue">CSSUnitValue</a></code> Serialization

<a id="ref-for-cssunitvalue①⓪③"></a>

To <a id="serialize-a-cssunitvalue"></a>serialize a <code><a href="#cssunitvalue">CSSUnitValue</a></code> <var>this</var>, with optional arguments <var>minimum</var>, a numeric value, and <var>maximum</var>, a numeric value:

1.  <a id="ref-for-dom-cssunitvalue-value②⑧"></a>

    <a id="ref-for-dom-cssunitvalue-unit③②"></a>

    Let <var>value</var> and <var>unit</var> be <var>this</var>‘s <code><a href="#dom-cssunitvalue-value">value</a></code> and <code><a href="#dom-cssunitvalue-unit">unit</a></code> internal slots.

2.  <a id="ref-for-number-value①③"></a>

    Set <var>s</var> to the result of serializing a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) from <var>value</var>, per [CSSOM § 6.7.2 Serializing CSS Values](https://www.w3.org/TR/cssom-1/#serializing-css-values).

3.  If <var>unit</var> is:

    "number"  
    Do nothing.

    "percent"  
    Append "%" to <var>s</var>.

    anything else  
    Append <var>unit</var> to <var>s</var>.

4.  If <var>minimum</var> was passed and <var>this</var> is less than <var>minimum</var>, or if <var>maximum</var> was passed and <var>this</var> is greater than <var>maximum</var>, or either <var>minimum</var> and/or <var>maximum</var> were passed and the relative size of <var>this</var> and <var>minimum</var>/<var>maximum</var> can’t be determined with the available information at this time, prepend "calc(" to <var>s</var>, then append ")" to <var>s</var>.

5.  Return <var>s</var>.

<a id="ref-for-cssmathvalue①①"></a>

### <a id="calc-serialization"></a>6.5. <code><a href="#cssmathvalue">CSSMathValue</a></code> Serialization

<a id="ref-for-cssmathvalue①②"></a>

To <a id="serialize-a-cssmathvalue"></a>serialize a <code><a href="#cssmathvalue">CSSMathValue</a></code> <var>this</var>, with optional arguments <var>nested</var>, a boolean (defaulting to false if unspecified), <var>paren-less</var>, a boolean (defaulting to false if unspecified), perform the following steps.

1.  <a id="ref-for-string①⓪"></a>

    Let <var>s</var> initially be the empty [string](https://infra.spec.whatwg.org/#string).

2.  <a id="ref-for-cssmathmin⑧"></a>

    <a id="ref-for-cssmathmax⑧"></a>

    If <var>this</var> is a <code><a href="#cssmathmin">CSSMathMin</a></code> or <code><a href="#cssmathmax">CSSMathMax</a></code>:

    1.  Append "min(" or "max(" to <var>s</var>, as appropriate.

    2.  <a id="ref-for-list-iterate①⑤"></a>

        <a id="ref-for-dom-cssmathmin-values④"></a>

        [For each](https://infra.spec.whatwg.org/#list-iterate) <var>arg</var> in <var>this</var>’s <code><a href="#dom-cssmathmin-values">values</a></code> internal slot, serialize <var>arg</var> with <var>nested</var> and <var>paren-less</var> both true, and append the result to <var>s</var>, appending a ", " between successive values.

    3.  Append ")" to <var>s</var> and return <var>s</var>.

3.  <a id="ref-for-cssmathsum①④"></a>

    Otherwise, if <var>this</var> is a <code><a href="#cssmathsum">CSSMathSum</a></code>:

    1.  If <var>paren-less</var> is true, continue to the next step; otherwise, if <var>nested</var> is true, append "(" to <var>s</var>; otherwise, append "calc(" to <var>s</var>.

    2.  <a id="ref-for-list-item⑥⑨"></a>

        <a id="ref-for-dom-cssmathsum-values①③"></a>

        Serialize the first [item](https://infra.spec.whatwg.org/#list-item) in <var>this</var>’s <code><a href="#dom-cssmathsum-values">values</a></code> internal slot with <var>nested</var> set to true, and append the result to <var>s</var>.

    3.  <a id="ref-for-list-iterate①⑥"></a>

        <a id="ref-for-dom-cssmathsum-values①④"></a>

        [For each](https://infra.spec.whatwg.org/#list-iterate) <var>arg</var> in <var>this</var>’s <code><a href="#dom-cssmathsum-values">values</a></code> internal slot beyond the first:

        1.  <a id="ref-for-cssmathnegate⑧"></a>

            <a id="ref-for-dom-cssmathnegate-value⑧"></a>

            If <var>arg</var> is a <code><a href="#cssmathnegate">CSSMathNegate</a></code>, append " - " to <var>s</var>, then serialize <var>arg</var>’s <code><a href="#dom-cssmathnegate-value">value</a></code> internal slot with <var>nested</var> set to true, and append the result to <var>s</var>.

        2.  Otherwise, append " + " to <var>s</var>, then serialize <var>arg</var> with <var>nested</var> set to true, and append the result to <var>s</var>.

    4.  If <var>paren-less</var> is false, append ")" to <var>s</var>,

    5.  Return <var>s</var>.

4.  <a id="ref-for-cssmathnegate⑨"></a>

    Otherwise, if <var>this</var> is a <code><a href="#cssmathnegate">CSSMathNegate</a></code>:

    1.  If <var>paren-less</var> is true, continue to the next step; otherwise, if <var>nested</var> is true, append "(" to <var>s</var>; otherwise, append "calc(" to <var>s</var>.

    2.  Append "-" to <var>s</var>.

    3.  <a id="ref-for-dom-cssmathnegate-value⑨"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssmathnegate-value">value</a></code> internal slot with <var>nested</var> set to true, and append the result to <var>s</var>.

    4.  If <var>paren-less</var> is false, append ")" to <var>s</var>,

    5.  Return <var>s</var>.

5.  <a id="ref-for-cssmathproduct⑧"></a>

    Otherwise, if <var>this</var> is a <code><a href="#cssmathproduct">CSSMathProduct</a></code>:

    1.  If <var>paren-less</var> is true, continue to the next step; otherwise, if <var>nested</var> is true, append "(" to <var>s</var>; otherwise, append "calc(" to <var>s</var>.

    2.  <a id="ref-for-list-item⑦⓪"></a>

        <a id="ref-for-dom-cssmathproduct-values⑤"></a>

        Serialize the first [item](https://infra.spec.whatwg.org/#list-item) in <var>this</var>’s <code><a href="#dom-cssmathproduct-values">values</a></code> internal slot with <var>nested</var> set to true, and append the result to <var>s</var>.

    3.  <a id="ref-for-list-iterate①⑦"></a>

        <a id="ref-for-dom-cssmathproduct-values⑥"></a>

        [For each](https://infra.spec.whatwg.org/#list-iterate) <var>arg</var> in <var>this</var>’s <code><a href="#dom-cssmathproduct-values">values</a></code> internal slot beyond the first:

        1.  <a id="ref-for-cssmathinvert⑧"></a>

            <a id="ref-for-dom-cssmathinvert-value⑤"></a>

            If <var>arg</var> is a <code><a href="#cssmathinvert">CSSMathInvert</a></code>, append " / " to <var>s</var>, then serialize <var>arg</var>’s <code><a href="#dom-cssmathinvert-value">value</a></code> internal slot with <var>nested</var> set to true, and append the result to <var>s</var>.

        2.  Otherwise, append " \* " to <var>s</var>, then serialize <var>arg</var> with <var>nested</var> set to true, and append the result to <var>s</var>.

    4.  If <var>paren-less</var> is false, append ")" to <var>s</var>,

    5.  Return <var>s</var>.

6.  <a id="ref-for-cssmathinvert⑨"></a>

    Otherwise, if <var>this</var> is a <code><a href="#cssmathinvert">CSSMathInvert</a></code>:

    1.  If <var>paren-less</var> is true, continue to the next step; otherwise, if <var>nested</var> is true, append "(" to <var>s</var>; otherwise, append "calc(" to <var>s</var>.

    2.  Append "1 / " to <var>s</var>.

    3.  <a id="ref-for-dom-cssmathinvert-value⑥"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssmathinvert-value">value</a></code> internal slot with <var>nested</var> set to true, and append the result to <var>s</var>.

    4.  If <var>paren-less</var> is false, append ")" to <var>s</var>,

    5.  Return <var>s</var>.

<a id="ref-for-csstransformvalue①④"></a>

<a id="ref-for-csstransformcomponent②⓪"></a>

### <a id="transformvalue-serialization"></a>6.6. <code><a href="#csstransformvalue">CSSTransformValue</a></code> and <code><a href="#csstransformcomponent">CSSTransformComponent</a></code> Serialization

<a id="ref-for-csstransformvalue①⑤"></a>

To <a id="serialize-a-csstransformvalue"></a>serialize a <code><a href="#csstransformvalue">CSSTransformValue</a></code> <var>this</var>:

1.  <a id="ref-for-list-item⑦①"></a>

    Return the result of serializing each [item](https://infra.spec.whatwg.org/#list-item) in <var>this</var>’s values to iterate over, then concatenating them separated by " ".

<a id="ref-for-csstranslate③"></a>

To <a id="serialize-a-csstranslate"></a>serialize a <code><a href="#csstranslate">CSSTranslate</a></code> <var>this</var>:

1.  <a id="ref-for-string①①"></a>

    Let <var>s</var> initially be the empty [string](https://infra.spec.whatwg.org/#string).

2.  <a id="ref-for-dom-csstransformvalue-is2d③"></a>

    If <var>this</var>’s <code><a href="#dom-csstransformvalue-is2d">is2D</a></code> internal slot is `false`:

    1.  Append "translate3d(" to <var>s</var>.

    2.  <a id="ref-for-dom-csstranslate-x②"></a>

        Serialize <var>this</var>’s <code><a href="#dom-csstranslate-x">x</a></code> internal slot, and append it to <var>s</var>.

    3.  Append ", " to <var>s</var>.

    4.  <a id="ref-for-dom-csstranslate-y②"></a>

        Serialize <var>this</var>’s <code><a href="#dom-csstranslate-y">y</a></code> internal slot, and append it to <var>s</var>.

    5.  Append ", " to <var>s</var>.

    6.  <a id="ref-for-dom-csstranslate-z④"></a>

        Serialize <var>this</var>’s <code><a href="#dom-csstranslate-z">z</a></code> internal slot, and append it to <var>s</var>.

    7.  Append ")" to <var>s</var>, and return <var>s</var>.

3.  Otherwise:

    1.  Append "translate(" to <var>s</var>.

    2.  <a id="ref-for-dom-csstranslate-x③"></a>

        Serialize <var>this</var>’s <code><a href="#dom-csstranslate-x">x</a></code> internal slot, and append it to <var>s</var>.

    3.  Append ", " to <var>s</var>.

    4.  <a id="ref-for-dom-csstranslate-y③"></a>

        Serialize <var>this</var>’s <code><a href="#dom-csstranslate-y">y</a></code> internal slot, and append it to <var>s</var>.

    5.  Append ")" to <var>s</var>, and return <var>s</var>.

<a id="ref-for-cssrotate③"></a>

To <a id="serialize-a-cssrotate"></a>serialize a <code><a href="#cssrotate">CSSRotate</a></code> <var>this</var>:

1.  <a id="ref-for-string①②"></a>

    Let <var>s</var> initially be the empty [string](https://infra.spec.whatwg.org/#string).

2.  <a id="ref-for-dom-csstransformvalue-is2d④"></a>

    If <var>this</var>’s <code><a href="#dom-csstransformvalue-is2d">is2D</a></code> internal slot is `false`:

    1.  Append "rotate3d(" to <var>s</var>.

    2.  <a id="ref-for-dom-cssrotate-x④"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssrotate-x">x</a></code> internal slot, and append it to <var>s</var>.

    3.  Append ", " to <var>s</var>.

    4.  <a id="ref-for-dom-cssrotate-y④"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssrotate-y">y</a></code> internal slot, and append it to <var>s</var>.

    5.  Append ", " to <var>s</var>.

    6.  <a id="ref-for-dom-cssrotate-z④"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssrotate-z">z</a></code> internal slot, and append it to <var>s</var>.

    7.  Append "," to <var>s</var>.

    8.  <a id="ref-for-dom-cssrotate-angle③"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssrotate-angle">angle</a></code> internal slot, and append it to <var>s</var>.

    9.  Append ")" to <var>s</var>, and return <var>s</var>.

3.  Otherwise:

    1.  Append "rotate(" to <var>s</var>.

    2.  <a id="ref-for-dom-cssrotate-angle④"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssrotate-angle">angle</a></code> internal slot, and append it to <var>s</var>.

    3.  Append ")" to <var>s</var>, and return <var>s</var>.

<a id="ref-for-cssscale②"></a>

To <a id="serialize-a-cssscale"></a>serialize a <code><a href="#cssscale">CSSScale</a></code> <var>this</var>:

1.  <a id="ref-for-string①③"></a>

    Let <var>s</var> initially be the empty [string](https://infra.spec.whatwg.org/#string).

2.  <a id="ref-for-dom-csstransformvalue-is2d⑤"></a>

    If <var>this</var>’s <code><a href="#dom-csstransformvalue-is2d">is2D</a></code> internal slot is `false`:

    1.  Append "scale3d(" to <var>s</var>.

    2.  <a id="ref-for-dom-cssscale-x③"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssscale-x">x</a></code> internal slot, and append it to <var>s</var>.

    3.  Append ", " to <var>s</var>.

    4.  <a id="ref-for-dom-cssscale-y③"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssscale-y">y</a></code> internal slot, and append it to <var>s</var>.

    5.  Append ", " to <var>s</var>.

    6.  <a id="ref-for-dom-cssscale-z④"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssscale-z">z</a></code> internal slot, and append it to <var>s</var>.

    7.  Append ")" to <var>s</var>, and return <var>s</var>.

3.  Otherwise:

    1.  Append "scale(" to <var>s</var>.

    2.  <a id="ref-for-dom-cssscale-x④"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssscale-x">x</a></code> internal slot, and append it to <var>s</var>.

    3.  <a id="ref-for-dom-cssscale-x⑤"></a>

        <a id="ref-for-dom-cssscale-y④"></a>

        <a id="ref-for-equal-numeric-value③"></a>

        If <var>this</var>’s <code><a href="#dom-cssscale-x">x</a></code> and <code><a href="#dom-cssscale-y">y</a></code> internal slots are [equal numeric values](#equal-numeric-value), append ")" to <var>s</var> and return <var>s</var>.

    4.  Otherwise, append ", " to <var>s</var>.

    5.  <a id="ref-for-dom-cssscale-y⑤"></a>

        Serialize <var>this</var>’s <code><a href="#dom-cssscale-y">y</a></code> internal slot, and append it to <var>s</var>.

    6.  Append ")" to <var>s</var>, and return <var>s</var>.

<a id="ref-for-cssskew③"></a>

To <a id="serialize-a-cssskew"></a>serialize a <code><a href="#cssskew">CSSSkew</a></code> <var>this</var>:

1.  Let <var>s</var> initially be "skew(".

2.  <a id="ref-for-dom-cssskew-ax②"></a>

    Serialize <var>this</var>’s <code><a href="#dom-cssskew-ax">ax</a></code> internal slot, and append it to <var>s</var>.

3.  <a id="ref-for-dom-cssskew-ay②"></a>

    <a id="ref-for-cssunitvalue①⓪④"></a>

    <a id="ref-for-dom-cssunitvalue-value②⑨"></a>

    If <var>this</var>’s <code><a href="#dom-cssskew-ay">ay</a></code> internal slot is a <code><a href="#cssunitvalue">CSSUnitValue</a></code> with a <code><a href="#dom-cssunitvalue-value">value</a></code> of `0`, then append ")" to <var>s</var> and return <var>s</var>.

4.  Otherwise, append ", " to <var>s</var>.

5.  <a id="ref-for-dom-cssskew-ay③"></a>

    Serialize <var>this</var>’s <code><a href="#dom-cssskew-ay">ay</a></code> internal slot, and append it to <var>s</var>.

6.  Append ")" to <var>s</var>, and return <var>s</var>.

<a id="ref-for-cssskewx③"></a>

To <a id="serialize-a-cssskewx"></a>serialize a <code><a href="#cssskewx">CSSSkewX</a></code> <var>this</var>:

1.  Let <var>s</var> initially be "skewX(".

2.  <a id="ref-for-dom-cssskewx-ax②"></a>

    Serialize <var>this</var>’s <code><a href="#dom-cssskewx-ax">ax</a></code> internal slot, and append it to <var>s</var>.

3.  Append ")" to <var>s</var>, and return <var>s</var>.

<a id="ref-for-cssskewy③"></a>

To <a id="serialize-a-cssskewy"></a>serialize a <code><a href="#cssskewy">CSSSkewY</a></code> <var>this</var>:

1.  Let <var>s</var> initially be "skewY(".

2.  <a id="ref-for-dom-cssskewy-ay②"></a>

    Serialize <var>this</var>’s <code><a href="#dom-cssskewy-ay">ay</a></code> internal slot, and append it to <var>s</var>.

3.  Append ")" to <var>s</var>, and return <var>s</var>.

<a id="ref-for-cssperspective③"></a>

To <a id="serialize-a-cssperspective"></a>serialize a <code><a href="#cssperspective">CSSPerspective</a></code> <var>this</var>:

1.  Let <var>s</var> initially be "perspective(".

2.  <a id="ref-for-dom-cssperspective-length②"></a>

    Serialize <var>this</var>’s <code><a href="#dom-cssperspective-length">length</a></code> internal slot, with a <var>minimum</var> of 0px, and append it to <var>s</var>.

3.  Append ")" to <var>s</var>, and return <var>s</var>.

<a id="ref-for-cssmatrixcomponent②"></a>

To <a id="serialize-a-cssmatrixcomponent"></a>serialize a <code><a href="#cssmatrixcomponent">CSSMatrixComponent</a></code> <var>this</var>:

1.  <a id="ref-for-dommatrixreadonly-stringification-behavior"></a>

    <a id="ref-for-dom-cssmatrixcomponent-matrix②"></a>

    Return the [serialization](https://www.w3.org/TR/geometry-1/#dommatrixreadonly-stringification-behavior) of <var>this</var>’s <code><a href="#dom-cssmatrixcomponent-matrix">matrix</a></code> internal slot.

### <a id="cssom-serialization"></a>6.7. Serialization from CSSOM Values

<a id="ref-for-cssstylevalue⑥⑧"></a>

<code><a href="#cssstylevalue">CSSStyleValue</a></code> objects produced by the user agent from values in the CSSOM, rather than directly constructed by the author, are serialized according to the following rules, depending on the property they came from:

<a id="ref-for-propdef-background-color①"></a>

[background-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color)

1.  <a id="ref-for-valdef-color-currentcolor"></a>

    If the value is the [currentcolor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) keyword, return "currentcolor".

2.  <a id="ref-for-typedef-color④"></a>

    Otherwise, return the result of serializing the [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) value.

<a id="ref-for-propdef-border-color①"></a>

[border-color](https://drafts.csswg.org/css-borders-4/#propdef-border-color)

1.  <a id="ref-for-valdef-color-currentcolor①"></a>

    If the value is the [currentcolor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) keyword, return "currentcolor".

2.  <a id="ref-for-typedef-color⑤"></a>

    Otherwise, return the result of serializing the [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) value.

<a id="ref-for-propdef-border-image"></a>

[border-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image)

1.  <a id="ref-for-list①⑥"></a>

    Let <var>values</var> initially be the empty [list](https://infra.spec.whatwg.org/#list).

2.  <a id="ref-for-propdef-border-image-source①"></a>

    <a id="ref-for-propdef-border-image-source②"></a>

    If [border-image-source](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-source) is not none, serialize [border-image-source](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-source) and append it to <var>values</var>.

3.  <a id="ref-for-propdef-border-image-slice"></a>

    <a id="ref-for-border-image-slice-fill"></a>

    <a id="ref-for-propdef-border-image-slice①"></a>

    If [border-image-slice](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-slice) does not specify 100% for all sides and omits the [fill](https://www.w3.org/TR/css-backgrounds-3/#border-image-slice-fill) keyword, serialize [border-image-slice](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-slice) and append it to <var>values</var>.

4.  <a id="ref-for-propdef-border-image-width"></a>

    <a id="ref-for-propdef-border-image-width①"></a>

    If [border-image-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-width) does not specify 1 for all sides, append "/ " (U+002F FORWARD SLASH followed by U+0020 SPACE) to the result of serializing [border-image-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-width) and append it to <var>values</var>.

5.  <a id="ref-for-propdef-border-image-outset"></a>

    If [border-image-outset](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-outset) does not specify 0 for all sides:

    1.  <a id="ref-for-propdef-border-image-width②"></a>

        If the previous [border-image-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-width) step did not append anything to <var>values</var>, let <var>prefix</var> be "// " (two U+002F FORWARD SLASH characters followed by U+0020 SPACE); otherwise let <var>prefix</var> be "/ " (U+002F FORWARD SLASH followed by U+0020 SPACE)

    2.  <a id="ref-for-propdef-border-image-outset①"></a>

        Append <var>prefix</var> to the result of serializing [border-image-outset](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-outset) and append it to <var>values</var>.

6.  <a id="ref-for-propdef-border-image-repeat"></a>

    <a id="ref-for-valdef-border-image-repeat-stretch"></a>

    <a id="ref-for-propdef-border-image-repeat①"></a>

    If [border-image-repeat](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-repeat) is not [stretch](https://www.w3.org/TR/css-backgrounds-3/#valdef-border-image-repeat-stretch) in both axises, serialize [border-image-repeat](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-repeat) and append it to <var>values</var>.

7.  <a id="ref-for-list-empty②"></a>

    If <var>values</var> is [empty](https://infra.spec.whatwg.org/#list-empty), append "none" to <var>values</var>.

8.  Return the result of concatenating all the items in <var>values</var>, separated by " " (U+0020 SPACE).

<a id="ref-for-propdef-bottom①"></a>

[bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom)

1.  <a id="ref-for-valdef-top-auto②"></a>

    If the value is the [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto) keyword, return "auto".

2.  <a id="ref-for-length-value⑨"></a>

    <a id="ref-for-length-value①⓪"></a>

    If the value is of type [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), return the result of serializing the [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) value.

3.  <a id="ref-for-percentage-value①⑤"></a>

    Otherwise, return the result of serializing the [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) value.

<a id="ref-for-propdef-color②"></a>

[color](https://www.w3.org/TR/css-color-4/#propdef-color)

1.  <a id="ref-for-valdef-color-currentcolor②"></a>

    If the value is the [currentcolor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) keyword, return "currentcolor".

2.  <a id="ref-for-typedef-color⑥"></a>

    Otherwise, return the result of serializing the [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) value.

<a id="ref-for-propdef-left①"></a>

[left](https://www.w3.org/TR/css-position-3/#propdef-left)

1.  <a id="ref-for-valdef-top-auto③"></a>

    If the value is the [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto) keyword, return "auto".

2.  <a id="ref-for-length-value①①"></a>

    <a id="ref-for-length-value①②"></a>

    If the value is of type [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), return the result of serializing the [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) value.

3.  <a id="ref-for-percentage-value①⑥"></a>

    Otherwise, return the result of serializing the [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) value.

<a id="ref-for-propdef-opacity②"></a>

[opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity)

1.  <a id="ref-for-number-value①④"></a>

    <a id="ref-for-number-value①⑤"></a>

    If the value is of type [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), return the result of serializing the [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) value.

2.  <a id="ref-for-percentage-value①⑦"></a>

    Otherwise, return the result of serializing the [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) value.

<a id="ref-for-propdef-right①"></a>

[right](https://www.w3.org/TR/css-position-3/#propdef-right)

1.  <a id="ref-for-valdef-top-auto④"></a>

    If the value is the [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto) keyword, return "auto".

2.  <a id="ref-for-length-value①③"></a>

    <a id="ref-for-length-value①④"></a>

    If the value is of type [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), return the result of serializing the [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) value.

3.  <a id="ref-for-percentage-value①⑧"></a>

    Otherwise, return the result of serializing the [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) value.

<a id="ref-for-propdef-top①"></a>

[top](https://www.w3.org/TR/css-position-3/#propdef-top)

1.  <a id="ref-for-valdef-top-auto⑤"></a>

    If the value is the [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto) keyword, return "auto".

2.  <a id="ref-for-length-value①⑤"></a>

    <a id="ref-for-length-value①⑥"></a>

    If the value is of type [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), return the result of serializing the [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) value.

3.  <a id="ref-for-percentage-value①⑨"></a>

    Otherwise, return the result of serializing the [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) value.

## <a id="security-considerations"></a>7. Security Considerations

There are no known security issues introduced by these features.

## <a id="privacy-considerations"></a>8. Privacy Considerations

There are no known privacy issues introduced by these features.

## <a id="changes"></a>9. Changes

### <a id="changes-20180410"></a>9.1. Changes since the [10 April 2018 Working Draft](https://www.w3.org/TR/2018/WD-css-typed-om-1-20180410/)

- Fixed the type match algorithm to refer to the percent hint more abstractly.

- Clarified that "invert a type" needs to preserve the percent hint.

- Added missing font units to CSS numeric factory. ([\#1107](https://github.com/w3c/css-houdini-drafts/pull/1107))

- Specified that the list of unit shorthand methods must be reduced or expanded to match the implementation’s support.

- Used undefined union value for "StylePropertyMapReadOnly.get()". ([\#1087](https://github.com/w3c/css-houdini-drafts/pull/1087))

- Removed .to() and the CSSColorValue.colorSpace, as conversion isn’t really in Typed OM’s remit. ([\#1070](https://github.com/w3c/css-houdini-drafts/issues/1070))

- Added min/max to "serialize a CSSUnitValue", and pass it when serializing the argument of CSSPerspective. ([\#1069](https://github.com/w3c/css-houdini-drafts/issues/1069))

- Changed reifying from "a CSSStyleValue" to "an identifier". ([\#1068](https://github.com/w3c/css-houdini-drafts/pull/1068))

- Added factory functions for new viewport/container units. ([\#1067](https://github.com/w3c/css-houdini-drafts/pull/1067))

- Added type checking to CSSNumericValue.parse. ([\#1065](https://github.com/w3c/css-houdini-drafts/pull/1065))

- Removed CSSDeviceCMYK, add CSSOKLab and CSSOKLCH, made all the color classes accept the "none" keyword.

- Defined how to reify a color value.

- Allowed "new unit value" to link to "create a CSSUnitValue from a pair" since that phrasing is used everywhere already.

- Added support for perspective(none) to CSSPerspective. ([\#1053](https://github.com/w3c/css-houdini-drafts/pull/1053))

- Fixed accidental clash of CSSClampValue.min/max with CSSNumericValue.min/max. ([\#855](https://github.com/w3c/css-houdini-drafts/issues/855))

- Simplified the Abstract

- Removed CSSGray and associated spec text, since Color 4 <em>dropped</em> gray() some time ago. ([\#1027](https://github.com/w3c/css-houdini-drafts/issues/1027))

- Moved .colorSpace up to the CSSColorValue superclass. Swapped the .to\*() color-conversion functions for a generic .to(colorSpace) method. ([\#1036](https://github.com/w3c/css-houdini-drafts/issues/1036))

- Added device-cmyk() support.

- Fix the OM for CSSColor to match recent simplifications.

- Specified which color function each CSSColorValue subclass represents, and added CSSGray.

- Added missing parentheses for union. ([\#1016](https://github.com/w3c/css-houdini-drafts/pull/1016))

- Added some explanatory text about the makeup of a "type".

- Aligned with Web IDL specification. ([\#965](https://github.com/w3c/css-houdini-drafts/pull/965) and [\#1006](https://github.com/w3c/css-houdini-drafts/pull/1006) )

- Fixed algorithm nesting.

- Added default dictionary value, required by update to [WebIDL](https://github.com/whatwg/webidl/pull/750). ([\#936](https://github.com/w3c/css-houdini-drafts/pull/936))

- Switched term from "underlying value" to :internal representation", as Web Animations already uses "underlying value" for something different.

- Clarified that reifying as a CSSSTyleValue requires a property.

- Defined reification behavior for registered custom properties ([\#886](https://github.com/w3c/css-houdini-drafts/pull/886))

- Used partial interface mixin ElementCSSInlineStyle ([\#853](https://github.com/w3c/css-houdini-drafts/pull/853))

- Used the correct name from WebIDL for indexed property getter.

- Exported a number of terms for use in other specifications

- Droped CSSPositionValue from this level.

- Added CSSMathClamp for the "clamp()"" function.

- Made the "match a grammar" algorithm a little more precise.

- Reifying computed numeric values uses the canonical unit. ([\#725](https://github.com/w3c/css-houdini-drafts/issues/725))

- Added note about new unit types. ([\#734](https://github.com/w3c/css-houdini-drafts/issues/734))

- Used correct unit "percent", not "percentage" ([\#761](https://github.com/w3c/css-houdini-drafts/issues/761)).

- Added note about custom property parsing

- Moved Naina Raisinghani to Former Editor

- Added an 'invert a type' operation.

- Exported several numeric-type terms, so CSS Values &#x26; Units can refer to them.

- Moved Shane Stephens to Former Editor

## <a id="w3c-conformance"></a> Conformance

### <a id="w3c-conventions"></a> Document conventions

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

- a
  - [attribute for CSSLab](#dom-csslab-a), in § 4.6
  - [attribute for CSSOKLab](#dom-cssoklab-a), in § 4.6
- [add](#cssnumericvalue-add-two-types), in § 4.3.2
- [add()](#dom-cssnumericvalue-add), in § 4.3.1
- [addition](#cssnumericvalue-add-two-types), in § 4.3.2
- [add two types](#cssnumericvalue-add-two-types), in § 4.3.2
- [add(...values)](#dom-cssnumericvalue-add), in § 4.3.1
- alpha
  - [attribute for CSSColor](#dom-csscolor-alpha), in § 4.6
  - [attribute for CSSHSL](#dom-csshsl-alpha), in § 4.6
  - [attribute for CSSHWB](#dom-csshwb-alpha), in § 4.6
  - [attribute for CSSLCH](#dom-csslch-alpha), in § 4.6
  - [attribute for CSSLab](#dom-csslab-alpha), in § 4.6
  - [attribute for CSSOKLCH](#dom-cssoklch-alpha), in § 4.6
  - [attribute for CSSOKLab](#dom-cssoklab-alpha), in § 4.6
  - [attribute for CSSRGB](#dom-cssrgb-alpha), in § 4.6
- ["angle"](#dom-cssnumericbasetype-angle), in § 4.3.1
- angle
  - [attribute for CSSRotate](#dom-cssrotate-angle), in § 4.4
  - [dict-member for CSSNumericType](#dom-cssnumerictype-angle), in § 4.3.1
- [append(property)](#dom-stylepropertymap-append), in § 3
- [append(property, ...values)](#dom-stylepropertymap-append), in § 3
- [apply the percent hint](#apply-the-percent-hint), in § 4.3.2
- [\[\[associatedProperty\]\]](#dom-cssstylevalue-associatedproperty-slot), in § 2.1
- [attributeStyleMap](#dom-elementcssinlinestyle-attributestylemap), in § 3.2
- ax
  - [attribute for CSSSkew](#dom-cssskew-ax), in § 4.4
  - [attribute for CSSSkewX](#dom-cssskewx-ax), in § 4.4
- ay
  - [attribute for CSSSkew](#dom-cssskew-ay), in § 4.4
  - [attribute for CSSSkewY](#dom-cssskewy-ay), in § 4.4
- b
  - [attribute for CSSHWB](#dom-csshwb-b), in § 4.6
  - [attribute for CSSLab](#dom-csslab-b), in § 4.6
  - [attribute for CSSOKLab](#dom-cssoklab-b), in § 4.6
  - [attribute for CSSRGB](#dom-cssrgb-b), in § 4.6
- [base type](#cssnumericvalue-base-type), in § 4.3.2
- c
  - [attribute for CSSLCH](#dom-csslch-c), in § 4.6
  - [attribute for CSSOKLCH](#dom-cssoklch-c), in § 4.6
- [cap(value)](#dom-css-cap), in § 4.3.5
- [channels](#dom-csscolor-channels), in § 4.6
- [ch(value)](#dom-css-ch), in § 4.3.5
- ["clamp"](#dom-cssmathoperator-clamp), in § 4.3.4
- [clear()](#dom-stylepropertymap-clear), in § 3
- [cm(value)](#dom-css-cm), in § 4.3.5
- [colorSpace](#dom-csscolor-colorspace), in § 4.6
- [computedStyleMap()](#dom-element-computedstylemap), in § 3.1
- [\[\[computedStyleMapCache\]\]](#dom-element-computedstylemapcache-slot), in § 3.1
- [Computed StylePropertyMap](#computed-stylepropertymap), in § 3.1
- constructor()
  - [constructor for CSSMathMax](#dom-cssmathmax-cssmathmax), in § 4.3.4
  - [constructor for CSSMathMin](#dom-cssmathmin-cssmathmin), in § 4.3.4
  - [constructor for CSSMathProduct](#dom-cssmathproduct-cssmathproduct), in § 4.3.4
  - [constructor for CSSMathSum](#dom-cssmathsum-cssmathsum), in § 4.3.4
- [constructor(angle)](#dom-cssrotate-cssrotate), in § 4.4
- constructor(arg)
  - [constructor for CSSMathInvert](#dom-cssmathinvert-cssmathinvert), in § 4.3.4
  - [constructor for CSSMathNegate](#dom-cssmathnegate-cssmathnegate), in § 4.3.4
- constructor(...args)
  - [constructor for CSSMathMax](#dom-cssmathmax-cssmathmax), in § 4.3.4
  - [constructor for CSSMathMin](#dom-cssmathmin-cssmathmin), in § 4.3.4
  - [constructor for CSSMathProduct](#dom-cssmathproduct-cssmathproduct), in § 4.3.4
  - [constructor for CSSMathSum](#dom-cssmathsum-cssmathsum), in § 4.3.4
- [constructor(ax)](#dom-cssskewx-cssskewx), in § 4.4
- [constructor(ax, ay)](#dom-cssskew-cssskew), in § 4.4
- [constructor(ay)](#dom-cssskewy-cssskewy), in § 4.4
- [constructor(colorSpace, channels)](#dom-csscolor-csscolor), in § 4.6
- [constructor(colorSpace, channels, alpha)](#dom-csscolor-csscolor), in § 4.6
- [constructor(h, s, l)](#dom-csshsl-csshsl), in § 4.6
- [constructor(h, s, l, alpha)](#dom-csshsl-csshsl), in § 4.6
- [constructor(h, w, b)](#dom-csshwb-csshwb), in § 4.6
- [constructor(h, w, b, alpha)](#dom-csshwb-csshwb), in § 4.6
- constructor(l, a, b)
  - [constructor for CSSLab](#dom-csslab-csslab), in § 4.6
  - [constructor for CSSOKLab](#dom-cssoklab-cssoklab), in § 4.6
- constructor(l, a, b, alpha)
  - [constructor for CSSLab](#dom-csslab-csslab), in § 4.6
  - [constructor for CSSOKLab](#dom-cssoklab-cssoklab), in § 4.6
- constructor(l, c, h)
  - [constructor for CSSLCH](#dom-csslch-csslch), in § 4.6
  - [constructor for CSSOKLCH](#dom-cssoklch-cssoklch), in § 4.6
- constructor(l, c, h, alpha)
  - [constructor for CSSLCH](#dom-csslch-csslch), in § 4.6
  - [constructor for CSSOKLCH](#dom-cssoklch-cssoklch), in § 4.6
- [constructor(length)](#dom-cssperspective-cssperspective), in § 4.4
- [constructor(lower, value, upper)](#dom-cssmathclamp-cssmathclamp), in § 4.3.4
- [constructor(matrix)](#dom-cssmatrixcomponent-cssmatrixcomponent), in § 4.4
- [constructor(matrix, options)](#dom-cssmatrixcomponent-cssmatrixcomponent), in § 4.4
- [constructor(members)](#dom-cssunparsedvalue-cssunparsedvalue), in § 4.1
- [constructor(r, g, b)](#dom-cssrgb-cssrgb), in § 4.6
- [constructor(r, g, b, alpha)](#dom-cssrgb-cssrgb), in § 4.6
- [constructor(transforms)](#dom-csstransformvalue-csstransformvalue), in § 4.4
- [constructor(value)](#dom-csskeywordvalue-csskeywordvalue), in § 4.2
- [constructor(value, unit)](#dom-cssunitvalue-cssunitvalue), in § 4.3.3
- [constructor(variable)](#dom-cssvariablereferencevalue-cssvariablereferencevalue), in § 4.1
- [constructor(variable, fallback)](#dom-cssvariablereferencevalue-cssvariablereferencevalue), in § 4.1
- constructor(x, y)
  - [constructor for CSSScale](#dom-cssscale-cssscale), in § 4.4
  - [constructor for CSSTranslate](#dom-csstranslate-csstranslate), in § 4.4
- constructor(x, y, z)
  - [constructor for CSSScale](#dom-cssscale-cssscale), in § 4.4
  - [constructor for CSSTranslate](#dom-csstranslate-csstranslate), in § 4.4
- [constructor(x, y, z, angle)](#dom-cssrotate-cssrotate-x-y-z-angle), in § 4.4
- [convert a CSSUnitValue](#convert-a-cssunitvalue), in § 4.3.3
- [cqb(value)](#dom-css-cqb), in § 4.3.5
- [cqh(value)](#dom-css-cqh), in § 4.3.5
- [cqi(value)](#dom-css-cqi), in § 4.3.5
- [cqmax(value)](#dom-css-cqmax), in § 4.3.5
- [cqmin(value)](#dom-css-cqmin), in § 4.3.5
- [cqw(value)](#dom-css-cqw), in § 4.3.5
- [create a CSSUnitValue from a pair](#create-a-cssunitvalue-from-a-pair), in § 4.3.3
- [create a CSSUnitValue from a sum value item](#create-a-cssunitvalue-from-a-sum-value-item), in § 4.3.1
- [create an internal representation](#create-an-internal-representation), in § 3
- [create a sum value](#create-a-sum-value), in § 4.3.1
- [create a type](#cssnumericvalue-create-a-type), in § 4.3.2
- [create a type from a unit map](#create-a-type-from-a-unit-map), in § 4.3.1
- [creating a sum value](#create-a-sum-value), in § 4.3.1
- [creating a type](#cssnumericvalue-create-a-type), in § 4.3.2
- [CSSColor](#csscolor), in § 4.6
- [CSSColorAngle](#typedefdef-csscolorangle), in § 4.6
- [CSSColor(colorSpace, channels)](#dom-csscolor-csscolor), in § 4.6
- [CSSColor(colorSpace, channels, alpha)](#dom-csscolor-csscolor), in § 4.6
- [CSSColor(colorSpace, channels, optional alpha)](#dom-csscolor-csscolor-colorspace-channels-optional-alpha), in § 4.6
- [CSSColorNumber](#typedefdef-csscolornumber), in § 4.6
- [CSSColorPercent](#typedefdef-csscolorpercent), in § 4.6
- [CSSColorRGBComp](#typedefdef-csscolorrgbcomp), in § 4.6
- [CSSColorValue](#csscolorvalue), in § 4.6
- [CSSHSL](#csshsl), in § 4.6
- [CSSHSL(h, s, l)](#dom-csshsl-csshsl), in § 4.6
- [CSSHSL(h, s, l, alpha)](#dom-csshsl-csshsl), in § 4.6
- [CSSHSL(h, s, l, optional alpha)](#dom-csshsl-csshsl-h-s-l-optional-alpha), in § 4.6
- [CSSHWB](#csshwb), in § 4.6
- [CSSHWB(h, w, b)](#dom-csshwb-csshwb), in § 4.6
- [CSSHWB(h, w, b, alpha)](#dom-csshwb-csshwb), in § 4.6
- [CSSHWB(h, w, b, optional alpha)](#dom-csshwb-csshwb-h-w-b-optional-alpha), in § 4.6
- [CSSImageValue](#cssimagevalue), in § 4.5
- [CSSKeywordish](#typedefdef-csskeywordish), in § 4.2
- [CSSKeywordValue](#csskeywordvalue), in § 4.2
- [CSSKeywordValue(value)](#dom-csskeywordvalue-csskeywordvalue), in § 4.2
- [CSSLab](#csslab), in § 4.6
- [CSSLab(l, a, b)](#dom-csslab-csslab), in § 4.6
- [CSSLab(l, a, b, alpha)](#dom-csslab-csslab), in § 4.6
- [CSSLab(l, a, b, optional alpha)](#dom-csslab-csslab-l-a-b-optional-alpha), in § 4.6
- [CSSLCH](#csslch), in § 4.6
- [CSSLCH(l, c, h)](#dom-csslch-csslch), in § 4.6
- [CSSLCH(l, c, h, alpha)](#dom-csslch-csslch), in § 4.6
- [CSSLCH(l, c, h, optional alpha)](#dom-csslch-csslch-l-c-h-optional-alpha), in § 4.6
- [CSSMathClamp](#cssmathclamp), in § 4.3.4
- [CSSMathClamp(lower, value, upper)](#dom-cssmathclamp-cssmathclamp), in § 4.3.4
- [CSSMathInvert](#cssmathinvert), in § 4.3.4
- [CSSMathInvert(arg)](#dom-cssmathinvert-cssmathinvert), in § 4.3.4
- [CSSMathMax](#cssmathmax), in § 4.3.4
- [CSSMathMax()](#dom-cssmathmax-cssmathmax), in § 4.3.4
- [CSSMathMax(...args)](#dom-cssmathmax-cssmathmax), in § 4.3.4
- [CSSMathMin](#cssmathmin), in § 4.3.4
- [CSSMathMin()](#dom-cssmathmin-cssmathmin), in § 4.3.4
- [CSSMathMin(...args)](#dom-cssmathmin-cssmathmin), in § 4.3.4
- [CSSMathNegate](#cssmathnegate), in § 4.3.4
- [CSSMathNegate(arg)](#dom-cssmathnegate-cssmathnegate), in § 4.3.4
- [CSSMathOperator](#enumdef-cssmathoperator), in § 4.3.4
- [CSSMathProduct](#cssmathproduct), in § 4.3.4
- [CSSMathProduct()](#dom-cssmathproduct-cssmathproduct), in § 4.3.4
- [CSSMathProduct(...args)](#dom-cssmathproduct-cssmathproduct), in § 4.3.4
- [CSSMathSum](#cssmathsum), in § 4.3.4
- [CSSMathSum()](#dom-cssmathsum-cssmathsum), in § 4.3.4
- [CSSMathSum(...args)](#dom-cssmathsum-cssmathsum), in § 4.3.4
- [CSSMathValue](#cssmathvalue), in § 4.3.4
- [CSSMatrixComponent](#cssmatrixcomponent), in § 4.4
- [CSSMatrixComponent(matrix)](#dom-cssmatrixcomponent-cssmatrixcomponent), in § 4.4
- [CSSMatrixComponent(matrix, options)](#dom-cssmatrixcomponent-cssmatrixcomponent), in § 4.4
- [CSSMatrixComponentOptions](#dictdef-cssmatrixcomponentoptions), in § 4.4
- [CSSNumberish](#typedefdef-cssnumberish), in § 4.3
- [CSSNumericArray](#cssnumericarray), in § 4.3.4
- [CSSNumericBaseType](#enumdef-cssnumericbasetype), in § 4.3.1
- [CSSNumericType](#dictdef-cssnumerictype), in § 4.3.1
- [CSSNumericValue](#cssnumericvalue), in § 4.3.1
- [CSSOKLab](#cssoklab), in § 4.6
- [CSSOKLab(l, a, b)](#dom-cssoklab-cssoklab), in § 4.6
- [CSSOKLab(l, a, b, alpha)](#dom-cssoklab-cssoklab), in § 4.6
- [CSSOKLab(l, a, b, optional alpha)](#dom-cssoklab-cssoklab-l-a-b-optional-alpha), in § 4.6
- [CSSOKLCH](#cssoklch), in § 4.6
- [CSSOKLCH(l, c, h)](#dom-cssoklch-cssoklch), in § 4.6
- [CSSOKLCH(l, c, h, alpha)](#dom-cssoklch-cssoklch), in § 4.6
- [CSSOKLCH(l, c, h, optional alpha)](#dom-cssoklch-cssoklch-l-c-h-optional-alpha), in § 4.6
- [CSSPerspective](#cssperspective), in § 4.4
- [CSSPerspective(length)](#dom-cssperspective-cssperspective), in § 4.4
- [CSSPerspectiveValue](#typedefdef-cssperspectivevalue), in § 4.4
- [CSSRGB](#cssrgb), in § 4.6
- [CSSRGB(r, g, b)](#dom-cssrgb-cssrgb), in § 4.6
- [CSSRGB(r, g, b, alpha)](#dom-cssrgb-cssrgb), in § 4.6
- [CSSRGB(r, g, b, optional alpha)](#dom-cssrgb-cssrgb-r-g-b-optional-alpha), in § 4.6
- [CSSRotate](#cssrotate), in § 4.4
- [CSSRotate(angle)](#dom-cssrotate-cssrotate), in § 4.4
- [CSSRotate(x, y, z, angle)](#dom-cssrotate-cssrotate-x-y-z-angle), in § 4.4
- [CSSScale](#cssscale), in § 4.4
- [CSSScale(x, y)](#dom-cssscale-cssscale), in § 4.4
- [CSSScale(x, y, z)](#dom-cssscale-cssscale), in § 4.4
- [CSSSkew](#cssskew), in § 4.4
- [CSSSkew(ax, ay)](#dom-cssskew-cssskew), in § 4.4
- [CSSSkewX](#cssskewx), in § 4.4
- [CSSSkewX(ax)](#dom-cssskewx-cssskewx), in § 4.4
- [CSSSkewY](#cssskewy), in § 4.4
- [CSSSkewY(ay)](#dom-cssskewy-cssskewy), in § 4.4
- [CSSStyleValue](#cssstylevalue), in § 2
- [CSSTransformComponent](#csstransformcomponent), in § 4.4
- [CSSTransformValue](#csstransformvalue), in § 4.4
- [CSSTransformValue(transforms)](#dom-csstransformvalue-csstransformvalue), in § 4.4
- [CSSTranslate](#csstranslate), in § 4.4
- [CSSTranslate(x, y)](#dom-csstranslate-csstranslate), in § 4.4
- [CSSTranslate(x, y, z)](#dom-csstranslate-csstranslate), in § 4.4
- [CSSUnitValue](#cssunitvalue), in § 4.3.3
- [CSSUnitValue(value, unit)](#dom-cssunitvalue-cssunitvalue), in § 4.3.3
- [CSSUnparsedSegment](#typedefdef-cssunparsedsegment), in § 4.1
- [CSSUnparsedValue](#cssunparsedvalue), in § 4.1
- [CSSUnparsedValue(members)](#dom-cssunparsedvalue-cssunparsedvalue), in § 4.1
- [CSSVariableReferenceValue](#cssvariablereferencevalue), in § 4.1
- [CSSVariableReferenceValue(variable)](#dom-cssvariablereferencevalue-cssvariablereferencevalue), in § 4.1
- [CSSVariableReferenceValue(variable, fallback)](#dom-cssvariablereferencevalue-cssvariablereferencevalue), in § 4.1
- [custom property name string](#custom-property-name-string), in § 3
- [\[\[declarations\]\]](#dom-stylepropertymapreadonly-declarations-slot), in § 3
- [Declared StylePropertyMap](#declared-stylepropertymap), in § 3.2
- [deg(value)](#dom-css-deg), in § 4.3.5
- [delete(property)](#dom-stylepropertymap-delete), in § 3
- [div()](#dom-cssnumericvalue-div), in § 4.3.1
- [div(...values)](#dom-cssnumericvalue-div), in § 4.3.1
- [dpcm(value)](#dom-css-dpcm), in § 4.3.5
- [dpi(value)](#dom-css-dpi), in § 4.3.5
- [dppx(value)](#dom-css-dppx), in § 4.3.5
- [dvb(value)](#dom-css-dvb), in § 4.3.5
- [dvh(value)](#dom-css-dvh), in § 4.3.5
- [dvi(value)](#dom-css-dvi), in § 4.3.5
- [dvmax(value)](#dom-css-dvmax), in § 4.3.5
- [dvmin(value)](#dom-css-dvmin), in § 4.3.5
- [dvw(value)](#dom-css-dvw), in § 4.3.5
- [em(value)](#dom-css-em), in § 4.3.5
- [equal numeric value](#equal-numeric-value), in § 4.3.1
- [equals()](#dom-cssnumericvalue-equals), in § 4.3.1
- [equals(...value)](#dom-cssnumericvalue-equals), in § 4.3.1
- [equals(...values)](#dom-cssnumericvalue-equals), in § 4.3.1
- [ex(value)](#dom-css-ex), in § 4.3.5
- [fallback](#dom-cssvariablereferencevalue-fallback), in § 4.1
- ["flex"](#dom-cssnumericbasetype-flex), in § 4.3.1
- [flex](#dom-cssnumerictype-flex), in § 4.3.1
- ["frequency"](#dom-cssnumericbasetype-frequency), in § 4.3.1
- [frequency](#dom-cssnumerictype-frequency), in § 4.3.1
- [fr(value)](#dom-css-fr), in § 4.3.5
- [g](#dom-cssrgb-g), in § 4.6
- [getAll(property)](#dom-stylepropertymapreadonly-getall), in § 3
- [get(property)](#dom-stylepropertymapreadonly-get), in § 3
- [grad(value)](#dom-css-grad), in § 4.3.5
- h
  - [attribute for CSSHSL](#dom-csshsl-h), in § 4.6
  - [attribute for CSSHWB](#dom-csshwb-h), in § 4.6
  - [attribute for CSSLCH](#dom-csslch-h), in § 4.6
  - [attribute for CSSOKLCH](#dom-cssoklch-h), in § 4.6
- [has(property)](#dom-stylepropertymapreadonly-has), in § 3
- [Hz(value)](#dom-css-hz), in § 4.3.5
- [ic(value)](#dom-css-ic), in § 4.3.5
- [indexed property getter](#cssnumericarray-indexed-property-getter), in § 4.3.4
- [internal representation](#css-internal-representation), in § 1
- [in(value)](#dom-css-in), in § 4.3.5
- ["invert"](#dom-cssmathoperator-invert), in § 4.3.4
- [invert](#cssmath-invert-a-cssnumericvalue), in § 4.3.1
- [invert a CSSNumericValue](#cssmath-invert-a-cssnumericvalue), in § 4.3.1
- [invert a type](#cssnumericvalue-invert-a-type), in § 4.3.2
- is2D
  - [attribute for CSSPerspective](#dom-cssperspective-is2d), in § 4.4
  - [attribute for CSSSkew, CSSSkewX, CSSSkewY](#dom-cssskew-is2d), in § 4.4
  - [attribute for CSSTransformComponent](#dom-csstransformcomponent-is2d), in § 4.4
  - [attribute for CSSTransformValue](#dom-csstransformvalue-is2d), in § 4.4
  - [dict-member for CSSMatrixComponentOptions](#dom-cssmatrixcomponentoptions-is2d), in § 4.4
- [kHz(value)](#dom-css-khz), in § 4.3.5
- l
  - [attribute for CSSHSL](#dom-csshsl-l), in § 4.6
  - [attribute for CSSLCH](#dom-csslch-l), in § 4.6
  - [attribute for CSSLab](#dom-csslab-l), in § 4.6
  - [attribute for CSSOKLCH](#dom-cssoklch-l), in § 4.6
  - [attribute for CSSOKLab](#dom-cssoklab-l), in § 4.6
- ["length"](#dom-cssnumericbasetype-length), in § 4.3.1
- length
  - [attribute for CSSNumericArray](#dom-cssnumericarray-length), in § 4.3.4
  - [attribute for CSSPerspective](#dom-cssperspective-length), in § 4.4
  - [attribute for CSSTransformValue](#dom-csstransformvalue-length), in § 4.4
  - [attribute for CSSUnparsedValue](#dom-cssunparsedvalue-length), in § 4.1
  - [dict-member for CSSNumericType](#dom-cssnumerictype-length), in § 4.3.1
- [lh(value)](#dom-css-lh), in § 4.3.5
- [list-valued](#list-valued-properties), in § 3
- [list-valued properties](#list-valued-properties), in § 3
- [lower](#dom-cssmathclamp-lower), in § 4.3.4
- [lvb(value)](#dom-css-lvb), in § 4.3.5
- [lvh(value)](#dom-css-lvh), in § 4.3.5
- [lvi(value)](#dom-css-lvi), in § 4.3.5
- [lvmax(value)](#dom-css-lvmax), in § 4.3.5
- [lvmin(value)](#dom-css-lvmin), in § 4.3.5
- [lvw(value)](#dom-css-lvw), in § 4.3.5
- [match](#cssnumericvalue-match), in § 4.3.2
- [match a grammar](#cssstylevalue-match-a-grammar), in § 3
- [match the grammar](#cssstylevalue-match-a-grammar), in § 3
- [matrix](#dom-cssmatrixcomponent-matrix), in § 4.4
- ["max"](#dom-cssmathoperator-max), in § 4.3.4
- [max()](#dom-cssnumericvalue-max), in § 4.3.1
- [max(...values)](#dom-cssnumericvalue-max), in § 4.3.1
- ["min"](#dom-cssmathoperator-min), in § 4.3.4
- [min()](#dom-cssnumericvalue-min), in § 4.3.1
- [min(...values)](#dom-cssnumericvalue-min), in § 4.3.1
- [mm(value)](#dom-css-mm), in § 4.3.5
- [ms(value)](#dom-css-ms), in § 4.3.5
- [mul()](#dom-cssnumericvalue-mul), in § 4.3.1
- [multiplication](#cssnumericvalue-multiply-two-types), in § 4.3.2
- [multiply](#cssnumericvalue-multiply-two-types), in § 4.3.2
- [multiply two types](#cssnumericvalue-multiply-two-types), in § 4.3.2
- [mul(...values)](#dom-cssnumericvalue-mul), in § 4.3.1
- ["negate"](#dom-cssmathoperator-negate), in § 4.3.4
- [negate](#cssmath-negate-a-cssnumericvalue), in § 4.3.1
- [negate a CSSNumericValue](#cssmath-negate-a-cssnumericvalue), in § 4.3.1
- [new unit value](#create-a-cssunitvalue-from-a-pair), in § 4.3.3
- [number(value)](#dom-css-number), in § 4.3.5
- [operator](#dom-cssmathvalue-operator), in § 4.3.4
- [parse a CSSStyleValue](#parse-a-cssstylevalue), in § 2
- [parseAll(property, cssText)](#dom-cssstylevalue-parseall), in § 2
- parse(cssText)
  - [method for CSSColorValue](#dom-csscolorvalue-parse), in § 4.6
  - [method for CSSNumericValue](#dom-cssnumericvalue-parse), in § 4.3.1
- [parse(property, cssText)](#dom-cssstylevalue-parse), in § 2
- [pc(value)](#dom-css-pc), in § 4.3.5
- ["percent"](#dom-cssnumericbasetype-percent), in § 4.3.1
- [percent](#dom-cssnumerictype-percent), in § 4.3.1
- [percent hint](#cssnumericvalue-percent-hint), in § 4.3.2
- [percentHint](#dom-cssnumerictype-percenthint), in § 4.3.1
- [percent(value)](#dom-css-percent), in § 4.3.5
- ["product"](#dom-cssmathoperator-product), in § 4.3.4
- [product of two unit maps](#product-of-two-unit-maps), in § 4.3.1
- [pt(value)](#dom-css-pt), in § 4.3.5
- [px(value)](#dom-css-px), in § 4.3.5
- [Q(value)](#dom-css-q), in § 4.3.5
- [r](#dom-cssrgb-r), in § 4.6
- [rad(value)](#dom-css-rad), in § 4.3.5
- [rcap(value)](#dom-css-rcap), in § 4.3.5
- [rch(value)](#dom-css-rch), in § 4.3.5
- [rectify a CSSColorAngle](#rectify-a-csscolorangle), in § 4.6
- [rectify a CSSColorNumber](#rectify-a-csscolornumber), in § 4.6
- [rectify a CSSColorPercent](#rectify-a-csscolorpercent), in § 4.6
- [rectify a CSSColorRGBComp](#rectify-a-csscolorrgbcomp), in § 4.6
- [rectify a keywordish value](#rectify-a-keywordish-value), in § 4.2
- [rectify a numberish value](#rectify-a-numberish-value), in § 4.3
- [reification](#css-reify), in § 5
- [reified as a CSSStyleValue](#reify-as-a-cssstylevalue), in § 5.2
- [reify](#css-reify), in § 5
- [reify a color value](#reify-a-color-value), in § 5.7
- [reify a list of component values](#reify-a-list-of-component-values), in § 5.3
- [reify a math expression](#reify-a-math-expression), in § 5.6
- [reify an identifier](#reify-an-identifier), in § 5.5
- [reify a numeric value](#reify-a-numeric-value), in § 5.6
- [reify as a CSSStyleValue](#reify-as-a-cssstylevalue), in § 5.2
- [reify a \<transform-function\>](#reify-a-transform-function), in § 5.8
- [reify a \<transform-list\>](#reify-a-transform-list), in § 5.8
- [reify a var() reference](#reify-a-var-reference), in § 5.4
- [rem(value)](#dom-css-rem), in § 4.3.5
- ["resolution"](#dom-cssnumericbasetype-resolution), in § 4.3.1
- [resolution](#dom-cssnumerictype-resolution), in § 4.3.1
- [rex(value)](#dom-css-rex), in § 4.3.5
- [ric(value)](#dom-css-ric), in § 4.3.5
- [rlh(value)](#dom-css-rlh), in § 4.3.5
- [s](#dom-csshsl-s), in § 4.6
- [serialize a CSSKeywordValue](#serialize-a-csskeywordvalue), in § 6.2
- [serialize a CSSMathValue](#serialize-a-cssmathvalue), in § 6.5
- [serialize a CSSMatrixComponent](#serialize-a-cssmatrixcomponent), in § 6.6
- [serialize a CSSNumericValue](#serialize-a-cssnumericvalue), in § 6.3
- [serialize a CSSPerspective](#serialize-a-cssperspective), in § 6.6
- [serialize a CSSRotate](#serialize-a-cssrotate), in § 6.6
- [serialize a CSSScale](#serialize-a-cssscale), in § 6.6
- [serialize a CSSSkew](#serialize-a-cssskew), in § 6.6
- [serialize a CSSSkewX](#serialize-a-cssskewx), in § 6.6
- [serialize a CSSSkewY](#serialize-a-cssskewy), in § 6.6
- [serialize a CSSTransformValue](#serialize-a-csstransformvalue), in § 6.6
- [serialize a CSSTranslate](#serialize-a-csstranslate), in § 6.6
- [serialize a CSSUnitValue](#serialize-a-cssunitvalue), in § 6.4
- [serialize a CSSUnparsedValue](#serialize-a-cssunparsedvalue), in § 6.1
- [serialize a CSSVariableReferenceValue](#serialize-a-cssvariablereferencevalue), in § 6.1
- [set(property)](#dom-stylepropertymap-set), in § 3
- [set(property, ...values)](#dom-stylepropertymap-set), in § 3
- [single-valued](#single-valued-properties), in § 3
- [single-valued properties](#single-valued-properties), in § 3
- [size](#dom-stylepropertymapreadonly-size), in § 3
- stringification behavior
  - [dfn for CSSStyleValue](#CSSStyleValue-stringification-behavior), in § 2
  - [dfn for CSSTransformComponent](#CSSTransformComponent-stringification-behavior), in § 4.4
- [styleMap](#dom-cssstylerule-stylemap), in § 3.2
- [StylePropertyMap](#stylepropertymap), in § 3
- [StylePropertyMapReadOnly](#stylepropertymapreadonly), in § 3
- [sub()](#dom-cssnumericvalue-sub), in § 4.3.1
- [subdivide into iterations](#subdivide-into-iterations), in § 2
- [sub(...values)](#dom-cssnumericvalue-sub), in § 4.3.1
- ["sum"](#dom-cssmathoperator-sum), in § 4.3.4
- [sum value](#cssnumericvalue-sum-value), in § 4.3.1
- [s(value)](#dom-css-s), in § 4.3.5
- [svb(value)](#dom-css-svb), in § 4.3.5
- [svh(value)](#dom-css-svh), in § 4.3.5
- [svi(value)](#dom-css-svi), in § 4.3.5
- [svmax(value)](#dom-css-svmax), in § 4.3.5
- [svmin(value)](#dom-css-svmin), in § 4.3.5
- [svw(value)](#dom-css-svw), in § 4.3.5
- ["time"](#dom-cssnumericbasetype-time), in § 4.3.1
- [time](#dom-cssnumerictype-time), in § 4.3.1
- [\[\[tokens\]\]](#dom-cssunparsedvalue-tokens-slot), in § 4.1
- toMatrix()
  - [method for CSSTransformComponent](#dom-csstransformcomponent-tomatrix), in § 4.4
  - [method for CSSTransformValue](#dom-csstransformvalue-tomatrix), in § 4.4
- [toSum()](#dom-cssnumericvalue-tosum), in § 4.3.1
- [toSum(...units)](#dom-cssnumericvalue-tosum), in § 4.3.1
- [to(unit)](#dom-cssnumericvalue-to), in § 4.3.1
- [turn(value)](#dom-css-turn), in § 4.3.5
- [type](#cssnumericvalue-type), in § 4.3.2
- [type()](#dom-cssnumericvalue-type), in § 4.3.1
- [type of a CSSMathValue](#type-of-a-cssmathvalue), in § 4.3.4
- [type of a CSSUnitValue](#type-of-a-cssunitvalue), in § 4.3.3
- [unit](#dom-cssunitvalue-unit), in § 4.3.3
- [unit map](#sum-value-unit-map), in § 4.3.1
- [upper](#dom-cssmathclamp-upper), in § 4.3.4
- [valid CSS property](#valid-css-property), in § 3
- value
  - [attribute for CSSKeywordValue](#dom-csskeywordvalue-value), in § 4.2
  - [attribute for CSSMathClamp](#dom-cssmathclamp-value), in § 4.3.4
  - [attribute for CSSMathInvert](#dom-cssmathinvert-value), in § 4.3.4
  - [attribute for CSSMathNegate](#dom-cssmathnegate-value), in § 4.3.4
  - [attribute for CSSUnitValue](#dom-cssunitvalue-value), in § 4.3.3
  - [dfn for sum value](#sum-value-value), in § 4.3.1
- [\[\[values\]\]](#dom-csstransformvalue-values-slot), in § 4.4
- values
  - [attribute for CSSMathMax](#dom-cssmathmax-values), in § 4.3.4
  - [attribute for CSSMathMin](#dom-cssmathmin-values), in § 4.3.4
  - [attribute for CSSMathProduct](#dom-cssmathproduct-values), in § 4.3.4
  - [attribute for CSSMathSum](#dom-cssmathsum-values), in § 4.3.4
- [variable](#dom-cssvariablereferencevalue-variable), in § 4.1
- [vb(value)](#dom-css-vb), in § 4.3.5
- [vh(value)](#dom-css-vh), in § 4.3.5
- [vi(value)](#dom-css-vi), in § 4.3.5
- [vmax(value)](#dom-css-vmax), in § 4.3.5
- [vmin(value)](#dom-css-vmin), in § 4.3.5
- [vw(value)](#dom-css-vw), in § 4.3.5
- [w](#dom-csshwb-w), in § 4.6
- x
  - [attribute for CSSRotate](#dom-cssrotate-x), in § 4.4
  - [attribute for CSSScale](#dom-cssscale-x), in § 4.4
  - [attribute for CSSTranslate](#dom-csstranslate-x), in § 4.4
- y
  - [attribute for CSSRotate](#dom-cssrotate-y), in § 4.4
  - [attribute for CSSScale](#dom-cssscale-y), in § 4.4
  - [attribute for CSSTranslate](#dom-csstranslate-y), in § 4.4
- z
  - [attribute for CSSRotate](#dom-cssrotate-z), in § 4.4
  - [attribute for CSSScale](#dom-cssscale-z), in § 4.4
  - [attribute for CSSTranslate](#dom-csstranslate-z), in § 4.4

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[COMPOSITING-1\] defines the following terms:
  - <a id="a55291b2"></a>background-blend-mode
  - <a id="ecfe8e64"></a>isolation
  - <a id="3249d67d"></a>mix-blend-mode
- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="ec1b298e"></a>\<overflow-position\>
  - <a id="652dcdea"></a>\<self-position\>
  - <a id="fde954ee"></a>align-content
  - <a id="e6b1dd7a"></a>align-items
  - <a id="9d95c839"></a>align-self
  - <a id="b7d152c3"></a>column-gap
  - <a id="b4d83355"></a>gap
  - <a id="f2c0557b"></a>grid-column-gap
  - <a id="6f9416f0"></a>grid-gap
  - <a id="8eaafef8"></a>grid-row-gap
  - <a id="de6bd31b"></a>justify-content
  - <a id="e4237559"></a>justify-items
  - <a id="80d2b689"></a>justify-self
  - <a id="0ef4bcd3"></a>place-content
  - <a id="64ff600f"></a>place-items
  - <a id="866c18ed"></a>place-self
  - <a id="2b373266"></a>row-gap
- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="1e12dba3"></a>animation
- \[CSS-ANIMATIONS-2\] defines the following terms:
  - <a id="419c34ee"></a>animation-composition
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="db6870d5"></a>background
  - <a id="8218676f"></a>background-attachment
  - <a id="a5c1f433"></a>background-clip
  - <a id="2754893b"></a>background-color
  - <a id="5ced56d0"></a>background-image
  - <a id="f2249e38"></a>background-position
  - <a id="e316431d"></a>background-repeat
  - <a id="e1674793"></a>border
  - <a id="bb65d94f"></a>border-image
  - <a id="b288fc8b"></a>border-image-outset
  - <a id="4ab170b5"></a>border-image-repeat
  - <a id="e75eda74"></a>border-image-slice
  - <a id="8fc9a97c"></a>border-image-source
  - <a id="9c014d15"></a>border-image-width
  - <a id="b6bb4b13"></a>border-style
  - <a id="064303ba"></a>border-width
  - <a id="3a23cc5f"></a>fill
  - <a id="8670a53a"></a>stretch
- \[CSS-BORDERS-4\] defines the following terms:
  - <a id="45a96a37"></a>border-block
  - <a id="6b65248d"></a>border-block-color
  - <a id="5d70435a"></a>border-block-end
  - <a id="26b72b18"></a>border-block-end-color
  - <a id="ceced8d4"></a>border-block-end-style
  - <a id="50391fa8"></a>border-block-end-width
  - <a id="f793e65e"></a>border-block-start
  - <a id="7b5a0ddb"></a>border-block-start-color
  - <a id="58cdf4fb"></a>border-block-start-style
  - <a id="91481de2"></a>border-block-start-width
  - <a id="ff338f6c"></a>border-block-style
  - <a id="8b01f835"></a>border-block-width
  - <a id="e697420b"></a>border-bottom
  - <a id="7bb914ec"></a>border-bottom-color
  - <a id="68fbbafd"></a>border-bottom-style
  - <a id="2ee001de"></a>border-bottom-width
  - <a id="29da74f9"></a>border-color
  - <a id="59de49b2"></a>border-inline
  - <a id="eeeb1518"></a>border-inline-color
  - <a id="125de889"></a>border-inline-end
  - <a id="60e9c55d"></a>border-inline-end-color
  - <a id="99cf77dd"></a>border-inline-end-style
  - <a id="8f60c8e7"></a>border-inline-end-width
  - <a id="121d9bdb"></a>border-inline-start
  - <a id="635a5b71"></a>border-inline-start-color
  - <a id="f2b8b95d"></a>border-inline-start-style
  - <a id="883360ae"></a>border-inline-start-width
  - <a id="ac2531a4"></a>border-inline-style
  - <a id="be8e9f0a"></a>border-inline-width
  - <a id="6daf8b2a"></a>border-left
  - <a id="04aee887"></a>border-left-color
  - <a id="63522c37"></a>border-left-style
  - <a id="bafd8e4b"></a>border-left-width
  - <a id="f24b672a"></a>border-radius
  - <a id="7d1d6278"></a>border-right
  - <a id="64e35b5b"></a>border-right-color
  - <a id="d0d8dec3"></a>border-right-style
  - <a id="c65dc97d"></a>border-right-width
  - <a id="8b0ff70b"></a>border-top
  - <a id="060efe1f"></a>border-top-color
  - <a id="58e9447e"></a>border-top-style
  - <a id="5900ea04"></a>border-top-width
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
  - <a id="51ee3396"></a>break-after
  - <a id="eb306f02"></a>break-before
  - <a id="8ae583f9"></a>break-inside
  - <a id="4f75e4ec"></a>orphans
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="a0542bba"></a>box-decoration-break
  - <a id="54f91f87"></a>widows
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="d3b48763"></a>all
  - <a id="8c8e51b4"></a>computed value
  - <a id="980ac56a"></a>shorthand property
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="cee404c5"></a>\<hex-color\>
  - <a id="8fad2d26"></a>\<named-color\>
  - <a id="bcdf9b19"></a>color
  - <a id="990ff186"></a>color()
  - <a id="a42c65ac"></a>currentcolor
  - <a id="5bd3632a"></a>hsl()
  - <a id="239640fd"></a>hsla()
  - <a id="3b7558dc"></a>opacity
  - <a id="daabf294"></a>rgb()
  - <a id="f3226176"></a>rgba()
  - <a id="96e27c16"></a>transparent
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="d04b6986"></a>\<color\>
  - <a id="f7f2e8bc"></a>hwb()
  - <a id="f1cce64a"></a>lab()
  - <a id="9b2c9faa"></a>lch()
  - <a id="deb378dd"></a>oklab()
- \[CSS-COLOR-ADJUST-1\] defines the following terms:
  - <a id="cf4eb8bb"></a>color-adjust
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="5dfeee7f"></a>contain
- \[CSS-CONTENT-3\] defines the following terms:
  - <a id="b246dfdc"></a>bookmark-label
  - <a id="7e7c0df8"></a>bookmark-level
  - <a id="cc800151"></a>bookmark-state
  - <a id="f3e8378c"></a>content
  - <a id="4ea7a8bb"></a>quotes
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="2ccfe434"></a>display
  - <a id="be21fe64"></a>order
  - <a id="d3ff9a69"></a>visibility
- \[CSS-FLEXBOX-1\] defines the following terms:
  - <a id="855cee06"></a>flex
  - <a id="50463bf6"></a>flex-basis
  - <a id="720fa00b"></a>flex-direction
  - <a id="546f7867"></a>flex-flow
  - <a id="85c23929"></a>flex-grow
  - <a id="02afbcf3"></a>flex-shrink
  - <a id="b70fab87"></a>flex-wrap
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="11a985a9"></a>font
  - <a id="7066562d"></a>font-family
  - <a id="9ce65b41"></a>font-language-override
  - <a id="a3b980a1"></a>font-optical-sizing
  - <a id="bd7559da"></a>font-palette
  - <a id="297dfe3a"></a>font-size
  - <a id="22947a28"></a>font-stretch
  - <a id="4f77d9f4"></a>font-style
  - <a id="7c64b027"></a>font-synthesis
  - <a id="009e8b9e"></a>font-variant
  - <a id="c3ac7f3a"></a>font-variant-alternates
  - <a id="f91916bc"></a>font-variant-emoji
  - <a id="0dc1a481"></a>font-variation-settings
  - <a id="73ccab19"></a>font-weight
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="ddd65350"></a>font-size-adjust
- \[CSS-GCPM-4\] defines the following terms:
  - <a id="4dfb5ba2"></a>copy-into
- \[CSS-GRID-2\] defines the following terms:
  - <a id="b1382fd7"></a>\<flex\>
  - <a id="e0422758"></a>grid
  - <a id="a9bf8584"></a>grid-area
  - <a id="228fb890"></a>grid-auto-columns
  - <a id="4ab05c66"></a>grid-auto-flow
  - <a id="6cc286fe"></a>grid-auto-rows
  - <a id="cb9c8621"></a>grid-column
  - <a id="0234daef"></a>grid-column-end
  - <a id="bc83f681"></a>grid-column-start
  - <a id="1de3963f"></a>grid-row
  - <a id="ea903a6a"></a>grid-row-end
  - <a id="de87c4d2"></a>grid-row-start
  - <a id="fda04115"></a>grid-template
  - <a id="a9860e91"></a>grid-template-areas
  - <a id="354cf3be"></a>grid-template-columns
  - <a id="25d528b6"></a>grid-template-rows
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="35bf32f2"></a>\<image\>
  - <a id="ac3e8a3d"></a>image-rendering
- \[CSS-IMAGES-4\] defines the following terms:
  - <a id="d7b0e86f"></a>image()
  - <a id="5663abb5"></a>image-resolution
  - <a id="99492242"></a>object-fit
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="6818bc7b"></a>alignment-baseline
  - <a id="ca30f467"></a>baseline-shift
  - <a id="e06c6241"></a>dominant-baseline
  - <a id="ca8d6b4a"></a>initial-letter
  - <a id="ea2306e1"></a>initial-letter-align
  - <a id="d319ada2"></a>initial-letter-wrap
  - <a id="2d8be2d9"></a>vertical-align
- \[CSS-LINE-GRID-1\] defines the following terms:
  - <a id="3ad3a385"></a>box-snap
  - <a id="a234626f"></a>line-grid
  - <a id="7e17ec34"></a>line-snap
- \[CSS-LISTS-3\] defines the following terms:
  - <a id="5630055e"></a>counter-increment
  - <a id="a9b8d58a"></a>counter-reset
  - <a id="3d394266"></a>counter-set
  - <a id="428c86a7"></a>list-style
  - <a id="237f9b49"></a>list-style-image
  - <a id="9585951c"></a>list-style-position
  - <a id="d065f190"></a>list-style-type
  - <a id="6ddcb142"></a>marker-side
- \[CSS-LOGICAL-1\] defines the following terms:
  - <a id="16a98720"></a>block-size
  - <a id="c4179341"></a>inline-size
  - <a id="eb515e3f"></a>margin-block
  - <a id="35f86b2a"></a>margin-block-end
  - <a id="7de802de"></a>margin-block-start
  - <a id="528bfadc"></a>margin-inline
  - <a id="a65d2b9d"></a>margin-inline-end
  - <a id="4e03920f"></a>margin-inline-start
  - <a id="02ef6a8c"></a>max-block-size
  - <a id="859f22e7"></a>max-inline-size
  - <a id="5010c349"></a>min-block-size
  - <a id="eaabd001"></a>min-inline-size
  - <a id="e35b2567"></a>padding-block
  - <a id="8e805bb2"></a>padding-block-end
  - <a id="e6333553"></a>padding-block-start
  - <a id="1e12703c"></a>padding-inline
  - <a id="2158dcb1"></a>padding-inline-end
  - <a id="55e4e010"></a>padding-inline-start
- \[CSS-MASKING-1\] defines the following terms:
  - <a id="e97d95b6"></a>clip
  - <a id="e2b06daa"></a>clip-path
  - <a id="1d4aea19"></a>clip-rule
  - <a id="5b43edf4"></a>mask
  - <a id="f75064ae"></a>mask-border
  - <a id="cbb11538"></a>mask-border-mode
  - <a id="e0d0a9a1"></a>mask-border-outset
  - <a id="9f556bb8"></a>mask-border-repeat
  - <a id="342c3f9f"></a>mask-border-slice
  - <a id="58dc6deb"></a>mask-border-source
  - <a id="60330cc8"></a>mask-border-width
  - <a id="9b097444"></a>mask-clip
  - <a id="d6d735dd"></a>mask-composite
  - <a id="2bb33078"></a>mask-image
  - <a id="6d1b3616"></a>mask-mode
  - <a id="eb3829e8"></a>mask-origin
  - <a id="c81a9470"></a>mask-position
  - <a id="d02d7e70"></a>mask-repeat
  - <a id="2da8691d"></a>mask-size
  - <a id="2695d5c6"></a>mask-type
- \[CSS-MULTICOL-1\] defines the following terms:
  - <a id="68c0aa81"></a>column-span
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="add377f4"></a>overflow
  - <a id="4ed1ab05"></a>overflow-x
  - <a id="e55d9f25"></a>overflow-y
  - <a id="3c986031"></a>scroll-behavior
  - <a id="91e8e72e"></a>scrollbar-gutter
  - <a id="a11024f3"></a>text-overflow
- \[CSS-OVERFLOW-4\] defines the following terms:
  - <a id="42ff6ed0"></a>continue
  - <a id="072f08da"></a>max-lines
- \[CSS-PAGE-3\] defines the following terms:
  - <a id="ce1025d0"></a>page
- \[CSS-PAGE-FLOATS-3\] defines the following terms:
  - <a id="761881c0"></a>float-defer
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="22a281b0"></a>auto
  - <a id="f411d42d"></a>bottom
  - <a id="8ddd23f9"></a>inset
  - <a id="9c1a22ed"></a>inset-block
  - <a id="ea28adea"></a>inset-block-end
  - <a id="5ec8aa2f"></a>inset-block-start
  - <a id="f05bf0e7"></a>inset-inline
  - <a id="ed598d3e"></a>inset-inline-end
  - <a id="ba8efa07"></a>inset-inline-start
  - <a id="ebcbc56d"></a>left
  - <a id="b8c34db8"></a>position
  - <a id="a5bae6ee"></a>right
  - <a id="f99d4ae2"></a>top
- \[CSS-REGIONS-1\] defines the following terms:
  - <a id="8ddccf89"></a>region-fragment
- \[CSS-RHYTHM-1\] defines the following terms:
  - <a id="ba99bbeb"></a>block-step
  - <a id="041b09d6"></a>block-step-align
  - <a id="aa05a223"></a>block-step-insert
  - <a id="1392b196"></a>block-step-round
  - <a id="03e2765f"></a>block-step-size
  - <a id="c834f1b3"></a>line-height-step
- \[CSS-ROUND-DISPLAY-1\] defines the following terms:
  - <a id="ed833f96"></a>border-boundary
- \[CSS-RUBY-1\] defines the following terms:
  - <a id="9f9cf6a5"></a>ruby-align
  - <a id="8090fb33"></a>ruby-merge
  - <a id="7a3159ad"></a>ruby-position
- \[CSS-SCROLL-ANCHORING-1\] defines the following terms:
  - <a id="9bc7612e"></a>overflow-anchor
- \[CSS-SCROLL-SNAP-1\] defines the following terms:
  - <a id="f205ddd4"></a>scroll-margin
  - <a id="d7afc778"></a>scroll-margin-block
  - <a id="0e5870f5"></a>scroll-margin-block-end
  - <a id="63595a2f"></a>scroll-margin-block-start
  - <a id="398ec5b7"></a>scroll-margin-bottom
  - <a id="2d566e9a"></a>scroll-margin-inline
  - <a id="d275a56c"></a>scroll-margin-inline-end
  - <a id="0828978f"></a>scroll-margin-inline-start
  - <a id="e5f2cc0a"></a>scroll-margin-left
  - <a id="d9d64e60"></a>scroll-margin-right
  - <a id="53835cb5"></a>scroll-margin-top
  - <a id="2f6cb60c"></a>scroll-padding
  - <a id="3dafe5c5"></a>scroll-padding-block
  - <a id="daa41a9c"></a>scroll-padding-block-end
  - <a id="05fe311b"></a>scroll-padding-block-start
  - <a id="69de0809"></a>scroll-padding-bottom
  - <a id="fc3aac96"></a>scroll-padding-inline
  - <a id="1bbf66ab"></a>scroll-padding-inline-end
  - <a id="bb8433a9"></a>scroll-padding-inline-start
  - <a id="2dc829a3"></a>scroll-padding-left
  - <a id="97f37d97"></a>scroll-padding-right
  - <a id="f468d014"></a>scroll-padding-top
  - <a id="5da025cc"></a>scroll-snap-align
  - <a id="0bcbabf1"></a>scroll-snap-stop
  - <a id="b8bb50c6"></a>scroll-snap-type
- \[CSS-SHAPES-1\] defines the following terms:
  - <a id="730a72b3"></a>shape-margin
- \[CSS-SHAPES-2\] defines the following terms:
  - <a id="ad2bb926"></a>shape-inside
  - <a id="4985f121"></a>shape-padding
- \[CSS-SIZE-ADJUST-1\] defines the following terms:
  - <a id="194f46b6"></a>text-size-adjust
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="54a1fea8"></a>box-sizing
  - <a id="5ad01cca"></a>height
  - <a id="2d68423f"></a>max-height
  - <a id="f4066072"></a>max-width
  - <a id="f12a84ab"></a>min-height
  - <a id="4ed06a81"></a>min-width
  - <a id="49731d1d"></a>width
- \[CSS-SPEECH-1\] defines the following terms:
  - <a id="138b31a3"></a>cue
  - <a id="3aacb0e5"></a>cue-after
  - <a id="189f9375"></a>cue-before
  - <a id="6c35f2ea"></a>pause
  - <a id="b8e70a54"></a>pause-after
  - <a id="23ff8128"></a>pause-before
  - <a id="b801b3f8"></a>rest
  - <a id="854cec6d"></a>rest-after
  - <a id="b20fd490"></a>rest-before
  - <a id="decb9188"></a>speak
  - <a id="86ac0f3d"></a>speak-as
  - <a id="1345b2fa"></a>voice-balance
  - <a id="7cdc3f2c"></a>voice-duration
  - <a id="ffb9cdc9"></a>voice-family
  - <a id="c6973623"></a>voice-pitch
  - <a id="9fb76693"></a>voice-range
  - <a id="d8fe6496"></a>voice-rate
  - <a id="29ad3311"></a>voice-stress
  - <a id="dda34368"></a>voice-volume
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="87de393a"></a>\<dimension-token\>
  - <a id="eebbfe3d"></a>\<number-token\>
  - <a id="8a73a2e3"></a>\<percentage-token\>
  - <a id="267b6766"></a>component value
  - <a id="67800454"></a>parse
  - <a id="fa8e4976"></a>parse a component value
- \[CSS-TABLES-3\] defines the following terms:
  - <a id="3f5a69c4"></a>border-collapse
  - <a id="917a1d3a"></a>caption-side
  - <a id="cf8f5e26"></a>empty-cells
  - <a id="f5fd32dd"></a>table-layout
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="36e5f32e"></a>text-align
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="02bd7a8b"></a>letter-spacing
  - <a id="1e5ca230"></a>text-indent
  - <a id="9d1f52f6"></a>text-transform
  - <a id="b093a29f"></a>white-space
  - <a id="a22cd86a"></a>word-spacing
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="4d38e4c5"></a>text-decoration
  - <a id="6b5fc94b"></a>text-decoration-skip
  - <a id="f88f3382"></a>text-decoration-skip-ink
  - <a id="bfc0b3a6"></a>text-decoration-thickness
  - <a id="90ea8c21"></a>text-emphasis-skip
  - <a id="6e8b8579"></a>text-underline-offset
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="77a042da"></a>\<transform-function\>
  - <a id="76b6d2eb"></a>\<transform-list\>
  - <a id="c2647302"></a>matrix()
  - <a id="6441b868"></a>rotate()
  - <a id="2160fcb3"></a>skew()
  - <a id="aaf764c1"></a>skewx()
  - <a id="0b52fa91"></a>skewy()
  - <a id="e7c6bf78"></a>transform
  - <a id="72bd2c25"></a>translate()
  - <a id="26ed428f"></a>translatex()
  - <a id="71159d37"></a>translatey()
- \[CSS-TRANSFORMS-2\] defines the following terms:
  - <a id="61e0d2e8"></a>backface-visibility
  - <a id="6be55714"></a>matrix3d()
  - <a id="0d0e57ea"></a>none
  - <a id="a8c69c2f"></a>perspective
  - <a id="3564358a"></a>perspective()
  - <a id="50dee79d"></a>perspective-origin
  - <a id="c7dad71d"></a>rotate
  - <a id="5cd8b624"></a>rotate3d()
  - <a id="483b78c4"></a>rotatex()
  - <a id="2d92ac7f"></a>rotatey()
  - <a id="45dee326"></a>rotatez()
  - <a id="75f8cbbb"></a>scale
  - <a id="cde5421d"></a>scale()
  - <a id="c1de87f1"></a>scale3d()
  - <a id="0f6cc9ce"></a>scalex()
  - <a id="f62ca22b"></a>scaley()
  - <a id="69a12b7f"></a>scalez()
  - <a id="98511d95"></a>transform-style
  - <a id="a0605489"></a>translate
  - <a id="78c7f5ad"></a>translate3d()
  - <a id="b7425531"></a>translatez()
- \[CSS-TRANSITIONS-1\] defines the following terms:
  - <a id="d3706df0"></a>transition
  - <a id="6ea7b710"></a>transition-delay
  - <a id="dce74f57"></a>transition-duration
  - <a id="6c957606"></a>transition-property
  - <a id="ed00ecee"></a>transition-timing-function
- \[CSS-UI-4\] defines the following terms:
  - <a id="727bd17b"></a>appearance
  - <a id="b1082c33"></a>caret
  - <a id="8378d695"></a>caret-color
  - <a id="5be9a59d"></a>caret-shape
  - <a id="5b085f76"></a>cursor
  - <a id="fb5642f9"></a>nav-down
  - <a id="0bcc030a"></a>nav-left
  - <a id="bf6e698b"></a>nav-right
  - <a id="776bbe9e"></a>nav-up
  - <a id="4e2aade2"></a>outline
  - <a id="6ed19243"></a>outline-color
  - <a id="d96b668e"></a>outline-offset
  - <a id="1173c7a3"></a>outline-style
  - <a id="61948517"></a>outline-width
  - <a id="73bc6606"></a>pointer-events
  - <a id="c034f069"></a>resize
  - <a id="47c02b73"></a>user-select
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="cf951e6c"></a>\<angle-percentage\>
  - <a id="d7e1d67b"></a>\<angle\>
  - <a id="a0144f62"></a>\<custom-ident\>
  - <a id="51ba8407"></a>\<dimension\>
  - <a id="c2268b97"></a>\<frequency\>
  - <a id="dcecfc13"></a>\<ident\>
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="9108b09d"></a>\<resolution\>
  - <a id="834ea28c"></a>\<time-percentage\>
  - <a id="7aaa7c88"></a>\<time\>
  - <a id="14d3255d"></a>calc()
  - <a id="4dbf81d6"></a>canonical unit
  - <a id="efabd40a"></a>compatible units
  - <a id="eefce2af"></a>em
  - <a id="d52686a1"></a>ident
  - <a id="ce7ea0de"></a>identifier
  - <a id="b2919015"></a>in
  - <a id="3db7b9e0"></a>math function
  - <a id="582b973a"></a>max()
  - <a id="23559e62"></a>min()
  - <a id="14182cd3"></a>percentage
  - <a id="20730c34"></a>px
  - <a id="5d6143d2"></a>relative length
  - <a id="3fa441fa"></a>url()
- \[CSS-VARIABLES-1\] defines the following terms:
  - <a id="310b3140"></a>\<custom-property-name\>
  - <a id="5550667d"></a>custom property
  - <a id="3beec8c9"></a>var()
- \[CSS-WILL-CHANGE-1\] defines the following terms:
  - <a id="81ec7485"></a>will-change
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
  - <a id="23cd6e75"></a>unicode-bidi
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="ad11923b"></a>glyph-orientation-vertical
  - <a id="9fc17679"></a>text-combine-upright
  - <a id="8664e85f"></a>text-orientation
  - <a id="37bb38a0"></a>writing-mode
- \[CSS21\] defines the following terms:
  - <a id="36504895"></a>azimuth
  - <a id="fe0abd77"></a>border-spacing
  - <a id="f1ecd537"></a>elevation
  - <a id="6d1040c8"></a>page-break-after
  - <a id="beca1e45"></a>page-break-before
  - <a id="7ee8e6fb"></a>page-break-inside
  - <a id="90b2accf"></a>pitch
  - <a id="a9069881"></a>pitch-range
  - <a id="e2b8e6b7"></a>play-during
  - <a id="fdaa092f"></a>richness
  - <a id="7ca33cdf"></a>speak-header
  - <a id="d6711e02"></a>speak-numeral
  - <a id="24bdba2c"></a>speak-punctuation
  - <a id="b8f14ba7"></a>speech-rate
  - <a id="78404507"></a>stress
  - <a id="22308f61"></a>volume
  - <a id="5455396f"></a>z-index
- \[CSS22\] defines the following terms:
  - <a id="4962ecad"></a>\<absolute-size\>
  - <a id="d686638a"></a>\<relative-size\>
  - <a id="9436e460"></a>clear
  - <a id="0570259e"></a>float
  - <a id="ac54fbff"></a>line-height
- \[CSSOM\] defines the following terms:
  - <a id="839bfab4"></a>CSS
  - <a id="2d22493a"></a>CSSStyleDeclaration
  - <a id="d293a05b"></a>CSSStyleRule
  - <a id="77eac6ac"></a>ElementCSSInlineStyle
  - <a id="ff26bbb9"></a>css declaration block
  - <a id="9852f862"></a>declarations
  - <a id="bfb148e6"></a>getComputedStyle(elt)
  - <a id="0c5a4350"></a>origin-clean flag
  - <a id="fc19454a"></a>resolved value
- \[DOM\] defines the following terms:
  - <a id="296f3551"></a>Element
- \[FILL-STROKE-3\] defines the following terms:
  - <a id="e754076c"></a>fill-break
  - <a id="b7cb01f3"></a>fill-color
  - <a id="11cf96f1"></a>fill-image
  - <a id="a6d6ba71"></a>fill-origin
  - <a id="98f51657"></a>fill-position
  - <a id="290e0dcc"></a>fill-repeat
  - <a id="b05d167f"></a>fill-size
  - <a id="0427150e"></a>stroke-align
  - <a id="66a0ea15"></a>stroke-break
  - <a id="cd67c18a"></a>stroke-color
  - <a id="515f6d67"></a>stroke-dash-corner
  - <a id="2ac9bf35"></a>stroke-dash-justify
  - <a id="436d2642"></a>stroke-image
  - <a id="a10505f9"></a>stroke-origin
  - <a id="262922c6"></a>stroke-position
  - <a id="62f7dae0"></a>stroke-repeat
  - <a id="854825c4"></a>stroke-size
- \[FILTER-EFFECTS-2\] defines the following terms:
  - <a id="54fa0afb"></a>backdrop-filter
- \[GEOMETRY-1\] defines the following terms:
  - <a id="fc6d6516"></a>DOMMatrix
  - <a id="8c648dcc"></a>DOMMatrixReadOnly
  - <a id="2bd0d107"></a>is2D
  - <a id="b50b957e"></a>stringification behavior
- \[HTML\] defines the following terms:
  - <a id="b08d0bb2"></a>HTMLElement
  - <a id="ba920583"></a>style
  - <a id="63a7a8ed"></a>style (for html-global)
- \[INFRA\] defines the following terms:
  - <a id="53275e46"></a>append
  - <a id="7f9469b5"></a>ascii case-insensitive
  - <a id="6f2dfa22"></a>ascii lowercase
  - <a id="915aff5e"></a>code point
  - <a id="a326add7"></a>contain
  - <a id="03afaf9c"></a>empty
  - <a id="8958b003"></a>entry
  - <a id="1243a891"></a>exist
  - <a id="16d07e10"></a>for each (for list)
  - <a id="45209803"></a>for each (for map)
  - <a id="6b815fdd"></a>is empty
  - <a id="5afbefcd"></a>item
  - <a id="d14a6f26"></a>key
  - <a id="649608b9"></a>list
  - <a id="3fca5a9e"></a>map
  - <a id="84b454ff"></a>ordered map
  - <a id="e2fc6023"></a>prepend
  - <a id="99c988d6"></a>remove (for list)
  - <a id="7d4424b2"></a>remove (for map)
  - <a id="0e6b2056"></a>set
  - <a id="0204d188"></a>size (for list)
  - <a id="653d0848"></a>size (for map)
  - <a id="0698d556"></a>string
  - <a id="0e8de730"></a>tuple
  - <a id="802b0fdd"></a>value
  - <a id="12d6b9a8"></a>values
- \[MOTION-1\] defines the following terms:
  - <a id="baf46cd8"></a>offset
  - <a id="4c57d46c"></a>offset-anchor
  - <a id="58cdffc6"></a>offset-distance
  - <a id="129692cd"></a>offset-path
  - <a id="0d4a490a"></a>offset-position
  - <a id="400cc450"></a>offset-rotate
- \[SVG2\] defines the following terms:
  - <a id="53053ffa"></a>color-interpolation
  - <a id="95054108"></a>color-rendering
  - <a id="49db0a8f"></a>cx
  - <a id="9054ecfa"></a>cy
  - <a id="5a1d84b9"></a>d
  - <a id="9fb26e00"></a>fill
  - <a id="4bf21ac7"></a>fill-opacity
  - <a id="a086f470"></a>fill-rule
  - <a id="de226194"></a>marker
  - <a id="838e5b64"></a>marker-end
  - <a id="3e655cf8"></a>marker-mid
  - <a id="8f035173"></a>marker-start
  - <a id="ef95ffba"></a>paint-order
  - <a id="bbd6f3c4"></a>r
  - <a id="5ea5f82b"></a>rx
  - <a id="0c2e0cb9"></a>ry
  - <a id="cfdad187"></a>shape-rendering
  - <a id="ee49973c"></a>shape-subtract
  - <a id="c6539ab4"></a>stop-color
  - <a id="6b5848eb"></a>stop-opacity
  - <a id="29ec1ba6"></a>stroke
  - <a id="73c7b281"></a>stroke-dasharray
  - <a id="1d0ba9a1"></a>stroke-dashoffset
  - <a id="acee373b"></a>stroke-linecap
  - <a id="acec364b"></a>stroke-linejoin
  - <a id="c5091c6b"></a>stroke-miterlimit
  - <a id="e643c18e"></a>stroke-opacity
  - <a id="02b0f49f"></a>stroke-width
  - <a id="0e38b48b"></a>text-anchor
  - <a id="510cef3b"></a>text-decoration-fill
  - <a id="b6029199"></a>text-decoration-stroke
  - <a id="6bf37c0a"></a>text-rendering
  - <a id="4a1865db"></a>vector-effect
  - <a id="63933d2b"></a>x
  - <a id="05627c5c"></a>y
- \[WEBIDL\] defines the following terms:
  - <a id="8855a9aa"></a>DOMString
  - <a id="889e932f"></a>Exposed
  - <a id="fe15742e"></a>ObservableArray
  - <a id="ec878a66"></a>RangeError
  - <a id="a5c91173"></a>SameObject
  - <a id="be2d2b4c"></a>SyntaxError
  - <a id="82ca3efc"></a>TypeError
  - <a id="b0d7f3c3"></a>USVString
  - <a id="811df97e"></a>backing list
  - <a id="5372cca8"></a>boolean
  - <a id="f9b8107a"></a>delete an indexed value
  - <a id="ac51ff4d"></a>determine the value of an indexed property
  - <a id="8c800cdf"></a>double
  - <a id="57cf55df"></a>indexed property getter
  - <a id="f8de33a3"></a>long
  - <a id="9cce47fd"></a>sequence
  - <a id="053b5fe6"></a>set an indexed value
  - <a id="83768b54"></a>set the value of a new indexed property
  - <a id="e030a144"></a>set the value of an existing indexed property
  - <a id="e031fa01"></a>supported property indices
  - <a id="b4cfa5ce"></a>throw
  - <a id="5f90bbfb"></a>undefined
  - <a id="e97a9688"></a>unsigned long
  - <a id="42a03705"></a>value pairs to iterate over

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-compositing-1"></a>\[COMPOSITING-1\]  
Rik Cabanier; Nikos Andronikos. [Compositing and Blending Level 1](https://www.w3.org/TR/compositing-1/). 13 January 2015. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;compositing-1&#x2F;](https://www.w3.org/TR/compositing-1/)

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-animations-2"></a>\[CSS-ANIMATIONS-2\]  
David Baron; Brian Birtles. [CSS Animations Level 2](https://www.w3.org/TR/css-animations-2/). 2 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-2&#x2F;](https://www.w3.org/TR/css-animations-2/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 19 December 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-borders-4"></a>\[CSS-BORDERS-4\]  
[CSS Borders and Box Decorations Module Level 4](https://drafts.csswg.org/css-borders-4/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-borders-4&#x2F;](https://drafts.csswg.org/css-borders-4/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 3 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 1 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 28 June 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-color-adjust-1"></a>\[CSS-COLOR-ADJUST-1\]  
Elika Etemad; et al. [CSS Color Adjustment Module Level 1](https://www.w3.org/TR/css-color-adjust-1/). 14 June 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-adjust-1&#x2F;](https://www.w3.org/TR/css-color-adjust-1/)

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-content-3"></a>\[CSS-CONTENT-3\]  
Elika Etemad; Dave Cramer. [CSS Generated Content Module Level 3](https://www.w3.org/TR/css-content-3/). 2 August 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-content-3&#x2F;](https://www.w3.org/TR/css-content-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 1 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-fonts-5"></a>\[CSS-FONTS-5\]  
Chris Lilley. [CSS Fonts Module Level 5](https://www.w3.org/TR/css-fonts-5/). 6 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-5&#x2F;](https://www.w3.org/TR/css-fonts-5/)

<a id="biblio-css-gcpm-4"></a>\[CSS-GCPM-4\]  
[CSS Generated Content for Paged Media Module Level 4](https://drafts.csswg.org/css-gcpm-4/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-gcpm-4&#x2F;](https://drafts.csswg.org/css-gcpm-4/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 1 April 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-line-grid-1"></a>\[CSS-LINE-GRID-1\]  
Elika Etemad; Koji Ishii; Alan Stearns. [CSS Line Grid Module Level 1](https://www.w3.org/TR/css-line-grid-1/). 16 September 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-line-grid-1&#x2F;](https://www.w3.org/TR/css-line-grid-1/)

<a id="biblio-css-lists-3"></a>\[CSS-LISTS-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://www.w3.org/TR/css-lists-3/). 17 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-lists-3&#x2F;](https://www.w3.org/TR/css-lists-3/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Rossen Atanassov; Elika Etemad. [CSS Logical Properties and Values Level 1](https://www.w3.org/TR/css-logical-1/). 27 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-multicol-1"></a>\[CSS-MULTICOL-1\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 12 October 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-overflow-4"></a>\[CSS-OVERFLOW-4\]  
David Baron; Florian Rivoal; Elika Etemad. [CSS Overflow Module Level 4](https://www.w3.org/TR/css-overflow-4/). 21 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-4&#x2F;](https://www.w3.org/TR/css-overflow-4/)

<a id="biblio-css-page-3"></a>\[CSS-PAGE-3\]  
Elika Etemad. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 14 September 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css-page-floats-3"></a>\[CSS-PAGE-FLOATS-3\]  
Johannes Wilm. [CSS Page Floats](https://www.w3.org/TR/css-page-floats-3/). 15 September 2015. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-floats-3&#x2F;](https://www.w3.org/TR/css-page-floats-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 3 April 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-properties-values-api-1"></a>\[CSS-PROPERTIES-VALUES-API-1\]  
Tab Atkins Jr.; et al. [CSS Properties and Values API Level 1](https://www.w3.org/TR/css-properties-values-api-1/). 13 October 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-properties-values-api-1&#x2F;](https://www.w3.org/TR/css-properties-values-api-1/)

<a id="biblio-css-regions-1"></a>\[CSS-REGIONS-1\]  
Rossen Atanassov; Alan Stearns. [CSS Regions Module Level 1](https://www.w3.org/TR/css-regions-1/). 9 October 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-regions-1&#x2F;](https://www.w3.org/TR/css-regions-1/)

<a id="biblio-css-rhythm-1"></a>\[CSS-RHYTHM-1\]  
Koji Ishii; Elika Etemad. [CSS Rhythmic Sizing](https://www.w3.org/TR/css-rhythm-1/). 2 March 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-rhythm-1&#x2F;](https://www.w3.org/TR/css-rhythm-1/)

<a id="biblio-css-round-display-1"></a>\[CSS-ROUND-DISPLAY-1\]  
Jihye Hong. [CSS Round Display Level 1](https://www.w3.org/TR/css-round-display-1/). 22 December 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-round-display-1&#x2F;](https://www.w3.org/TR/css-round-display-1/)

<a id="biblio-css-ruby-1"></a>\[CSS-RUBY-1\]  
Elika Etemad; et al. [CSS Ruby Annotation Layout Module Level 1](https://www.w3.org/TR/css-ruby-1/). 31 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ruby-1&#x2F;](https://www.w3.org/TR/css-ruby-1/)

<a id="biblio-css-scroll-anchoring-1"></a>\[CSS-SCROLL-ANCHORING-1\]  
Tab Atkins Jr.. [CSS Scroll Anchoring Module Level 1](https://www.w3.org/TR/css-scroll-anchoring-1/). 11 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scroll-anchoring-1&#x2F;](https://www.w3.org/TR/css-scroll-anchoring-1/)

<a id="biblio-css-scroll-snap-1"></a>\[CSS-SCROLL-SNAP-1\]  
Matt Rakow; et al. [CSS Scroll Snap Module Level 1](https://www.w3.org/TR/css-scroll-snap-1/). 11 March 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scroll-snap-1&#x2F;](https://www.w3.org/TR/css-scroll-snap-1/)

<a id="biblio-css-shapes-1"></a>\[CSS-SHAPES-1\]  
Rossen Atanassov; Alan Stearns. [CSS Shapes Module Level 1](https://www.w3.org/TR/css-shapes-1/). 15 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shapes-1&#x2F;](https://www.w3.org/TR/css-shapes-1/)

<a id="biblio-css-shapes-2"></a>\[CSS-SHAPES-2\]  
[CSS Shapes Module Level 2](https://drafts.csswg.org/css-shapes-2/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-shapes-2&#x2F;](https://drafts.csswg.org/css-shapes-2/)

<a id="biblio-css-size-adjust-1"></a>\[CSS-SIZE-ADJUST-1\]  
[CSS Mobile Text Size Adjustment Module Level 1](https://drafts.csswg.org/css-size-adjust-1/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-size-adjust-1&#x2F;](https://drafts.csswg.org/css-size-adjust-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-speech-1"></a>\[CSS-SPEECH-1\]  
Léonie Watson; Elika Etemad. [CSS Speech Module Level 1](https://www.w3.org/TR/css-speech-1/). 14 February 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-speech-1&#x2F;](https://www.w3.org/TR/css-speech-1/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-tables-3"></a>\[CSS-TABLES-3\]  
François Remy; Greg Whitworth; David Baron. [CSS Table Module Level 3](https://www.w3.org/TR/css-tables-3/). 27 July 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-tables-3&#x2F;](https://www.w3.org/TR/css-tables-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 3 September 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-text-4"></a>\[CSS-TEXT-4\]  
Elika Etemad; et al. [CSS Text Module Level 4](https://www.w3.org/TR/css-text-4/). 20 October 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-4&#x2F;](https://www.w3.org/TR/css-text-4/)

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 4 May 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-transforms-2"></a>\[CSS-TRANSFORMS-2\]  
Tab Atkins Jr.; et al. [CSS Transforms Module Level 2](https://www.w3.org/TR/css-transforms-2/). 9 November 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-2&#x2F;](https://www.w3.org/TR/css-transforms-2/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 18 December 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 16 June 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-css-will-change-1"></a>\[CSS-WILL-CHANGE-1\]  
Tab Atkins Jr.. [CSS Will Change Module Level 1](https://www.w3.org/TR/css-will-change-1/). 5 May 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-will-change-1&#x2F;](https://www.w3.org/TR/css-will-change-1/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css22"></a>\[CSS22\]  
Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://www.w3.org/TR/CSS22/). 12 April 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS22&#x2F;](https://www.w3.org/TR/CSS22/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-fill-stroke-3"></a>\[FILL-STROKE-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Fill and Stroke Module Level 3](https://www.w3.org/TR/fill-stroke-3/). 13 April 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;fill-stroke-3&#x2F;](https://www.w3.org/TR/fill-stroke-3/)

<a id="biblio-filter-effects-2"></a>\[FILTER-EFFECTS-2\]  
[Filter Effects Module Level 2](https://drafts.fxtf.org/filter-effects-2/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;fxtf&#x2E;org&#x2F;filter-effects-2&#x2F;](https://drafts.fxtf.org/filter-effects-2/)

<a id="biblio-geometry-1"></a>\[GEOMETRY-1\]  
Simon Pieters; Chris Harrelson. [Geometry Interfaces Module Level 1](https://www.w3.org/TR/geometry-1/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;geometry-1&#x2F;](https://www.w3.org/TR/geometry-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-motion-1"></a>\[MOTION-1\]  
Dirk Schulze; et al. [Motion Path Module Level 1](https://www.w3.org/TR/motion-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;motion-1&#x2F;](https://www.w3.org/TR/motion-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

## <a id="idl-index"></a>IDL Index

```text
[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSStyleValue {
    stringifier;
    [Exposed=Window] static CSSStyleValue parse(USVString property, USVString cssText);
    [Exposed=Window] static sequence<CSSStyleValue> parseAll(USVString property, USVString cssText);
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface StylePropertyMapReadOnly {
    iterable<USVString, sequence<CSSStyleValue>>;
    (undefined or CSSStyleValue) get(USVString property);
    sequence<CSSStyleValue> getAll(USVString property);
    boolean has(USVString property);
    readonly attribute unsigned long size;
};

[Exposed=Window]
interface StylePropertyMap : StylePropertyMapReadOnly {
    undefined set(USVString property, (CSSStyleValue or USVString)... values);
    undefined append(USVString property, (CSSStyleValue or USVString)... values);
    undefined delete(USVString property);
    undefined clear();
};

partial interface Element {
    [SameObject] StylePropertyMapReadOnly computedStyleMap();
};

partial interface CSSStyleRule {
    [SameObject] readonly attribute StylePropertyMap styleMap;
};

partial interface mixin ElementCSSInlineStyle {
    [SameObject] readonly attribute StylePropertyMap attributeStyleMap;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSUnparsedValue : CSSStyleValue {
    constructor(sequence<CSSUnparsedSegment> members);
    iterable<CSSUnparsedSegment>;
    readonly attribute unsigned long length;
    getter CSSUnparsedSegment (unsigned long index);
    setter CSSUnparsedSegment (unsigned long index, CSSUnparsedSegment val);
};

typedef (USVString or CSSVariableReferenceValue) CSSUnparsedSegment;

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSVariableReferenceValue {
    constructor(USVString variable, optional CSSUnparsedValue? fallback = null);
    attribute USVString variable;
    readonly attribute CSSUnparsedValue? fallback;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSKeywordValue : CSSStyleValue {
    constructor(USVString value);
    attribute USVString value;
};

typedef (DOMString or CSSKeywordValue) CSSKeywordish;

typedef (double or CSSNumericValue) CSSNumberish;

enum CSSNumericBaseType {
    "length",
    "angle",
    "time",
    "frequency",
    "resolution",
    "flex",
    "percent",
};

dictionary CSSNumericType {
    long length;
    long angle;
    long time;
    long frequency;
    long resolution;
    long flex;
    long percent;
    CSSNumericBaseType percentHint;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSNumericValue : CSSStyleValue {
    CSSNumericValue add(CSSNumberish... values);
    CSSNumericValue sub(CSSNumberish... values);
    CSSNumericValue mul(CSSNumberish... values);
    CSSNumericValue div(CSSNumberish... values);
    CSSNumericValue min(CSSNumberish... values);
    CSSNumericValue max(CSSNumberish... values);

    boolean equals(CSSNumberish... value);

    CSSUnitValue to(USVString unit);
    CSSMathSum toSum(USVString... units);
    CSSNumericType type();

    [Exposed=Window] static CSSNumericValue parse(USVString cssText);
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSUnitValue : CSSNumericValue {
    constructor(double value, USVString unit);
    attribute double value;
    readonly attribute USVString unit;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathValue : CSSNumericValue {
    readonly attribute CSSMathOperator operator;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathSum : CSSMathValue {
    constructor(CSSNumberish... args);
    readonly attribute CSSNumericArray values;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathProduct : CSSMathValue {
    constructor(CSSNumberish... args);
    readonly attribute CSSNumericArray values;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathNegate : CSSMathValue {
    constructor(CSSNumberish arg);
    readonly attribute CSSNumericValue value;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathInvert : CSSMathValue {
    constructor(CSSNumberish arg);
    readonly attribute CSSNumericValue value;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathMin : CSSMathValue {
    constructor(CSSNumberish... args);
    readonly attribute CSSNumericArray values;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathMax : CSSMathValue {
    constructor(CSSNumberish... args);
    readonly attribute CSSNumericArray values;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMathClamp : CSSMathValue {
    constructor(CSSNumberish lower, CSSNumberish value, CSSNumberish upper);
    readonly attribute CSSNumericValue lower;
    readonly attribute CSSNumericValue value;
    readonly attribute CSSNumericValue upper;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSNumericArray {
    iterable<CSSNumericValue>;
    readonly attribute unsigned long length;
    getter CSSNumericValue (unsigned long index);
};

enum CSSMathOperator {
    "sum",
    "product",
    "negate",
    "invert",
    "min",
    "max",
    "clamp",
};

partial namespace CSS {
    CSSUnitValue number(double value);
    CSSUnitValue percent(double value);

    // <length>
    CSSUnitValue cap(double value);
    CSSUnitValue ch(double value);
    CSSUnitValue em(double value);
    CSSUnitValue ex(double value);
    CSSUnitValue ic(double value);
    CSSUnitValue lh(double value);
    CSSUnitValue rcap(double value);
    CSSUnitValue rch(double value);
    CSSUnitValue rem(double value);
    CSSUnitValue rex(double value);
    CSSUnitValue ric(double value);
    CSSUnitValue rlh(double value);
    CSSUnitValue vw(double value);
    CSSUnitValue vh(double value);
    CSSUnitValue vi(double value);
    CSSUnitValue vb(double value);
    CSSUnitValue vmin(double value);
    CSSUnitValue vmax(double value);
    CSSUnitValue svw(double value);
    CSSUnitValue svh(double value);
    CSSUnitValue svi(double value);
    CSSUnitValue svb(double value);
    CSSUnitValue svmin(double value);
    CSSUnitValue svmax(double value);
    CSSUnitValue lvw(double value);
    CSSUnitValue lvh(double value);
    CSSUnitValue lvi(double value);
    CSSUnitValue lvb(double value);
    CSSUnitValue lvmin(double value);
    CSSUnitValue lvmax(double value);
    CSSUnitValue dvw(double value);
    CSSUnitValue dvh(double value);
    CSSUnitValue dvi(double value);
    CSSUnitValue dvb(double value);
    CSSUnitValue dvmin(double value);
    CSSUnitValue dvmax(double value);
    CSSUnitValue cqw(double value);
    CSSUnitValue cqh(double value);
    CSSUnitValue cqi(double value);
    CSSUnitValue cqb(double value);
    CSSUnitValue cqmin(double value);
    CSSUnitValue cqmax(double value);
    CSSUnitValue cm(double value);
    CSSUnitValue mm(double value);
    CSSUnitValue Q(double value);
    CSSUnitValue in(double value);
    CSSUnitValue pt(double value);
    CSSUnitValue pc(double value);
    CSSUnitValue px(double value);

    // <angle>
    CSSUnitValue deg(double value);
    CSSUnitValue grad(double value);
    CSSUnitValue rad(double value);
    CSSUnitValue turn(double value);

    // <time>
    CSSUnitValue s(double value);
    CSSUnitValue ms(double value);

    // <frequency>
    CSSUnitValue Hz(double value);
    CSSUnitValue kHz(double value);

    // <resolution>
    CSSUnitValue dpi(double value);
    CSSUnitValue dpcm(double value);
    CSSUnitValue dppx(double value);

    // <flex>
    CSSUnitValue fr(double value);
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSTransformValue : CSSStyleValue {
    constructor(sequence<CSSTransformComponent> transforms);
    iterable<CSSTransformComponent>;
    readonly attribute unsigned long length;
    getter CSSTransformComponent (unsigned long index);
    setter CSSTransformComponent (unsigned long index, CSSTransformComponent val);

    readonly attribute boolean is2D;
    DOMMatrix toMatrix();
};

typedef (CSSNumericValue or CSSKeywordish) CSSPerspectiveValue;

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSTransformComponent {
    stringifier;
    attribute boolean is2D;
    DOMMatrix toMatrix();
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSTranslate : CSSTransformComponent {
    constructor(CSSNumericValue x, CSSNumericValue y, optional CSSNumericValue z);
    attribute CSSNumericValue x;
    attribute CSSNumericValue y;
    attribute CSSNumericValue z;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSRotate : CSSTransformComponent {
    constructor(CSSNumericValue angle);
    constructor(CSSNumberish x, CSSNumberish y, CSSNumberish z, CSSNumericValue angle);
    attribute CSSNumberish x;
    attribute CSSNumberish y;
    attribute CSSNumberish z;
    attribute CSSNumericValue angle;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSScale : CSSTransformComponent {
    constructor(CSSNumberish x, CSSNumberish y, optional CSSNumberish z);
    attribute CSSNumberish x;
    attribute CSSNumberish y;
    attribute CSSNumberish z;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSSkew : CSSTransformComponent {
    constructor(CSSNumericValue ax, CSSNumericValue ay);
    attribute CSSNumericValue ax;
    attribute CSSNumericValue ay;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSSkewX : CSSTransformComponent {
    constructor(CSSNumericValue ax);
    attribute CSSNumericValue ax;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSSkewY : CSSTransformComponent {
    constructor(CSSNumericValue ay);
    attribute CSSNumericValue ay;
};

/* Note that skew(x,y) is *not* the same as skewX(x) skewY(y),
   thus the separate interfaces for all three. */

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSPerspective : CSSTransformComponent {
    constructor(CSSPerspectiveValue length);
    attribute CSSPerspectiveValue length;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSMatrixComponent : CSSTransformComponent {
    constructor(DOMMatrixReadOnly matrix, optional CSSMatrixComponentOptions options = {});
    attribute DOMMatrix matrix;
};

dictionary CSSMatrixComponentOptions {
    boolean is2D;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSImageValue : CSSStyleValue {
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSColorValue : CSSStyleValue {
    [Exposed=Window] static (CSSColorValue or CSSStyleValue) parse(USVString cssText);
};

typedef (CSSNumberish or CSSKeywordish) CSSColorRGBComp;
typedef (CSSNumberish or CSSKeywordish) CSSColorPercent;
typedef (CSSNumberish or CSSKeywordish) CSSColorNumber;
typedef (CSSNumberish or CSSKeywordish) CSSColorAngle;

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSRGB : CSSColorValue {
    constructor(CSSColorRGBComp r, CSSColorRGBComp g, CSSColorRGBComp b, optional CSSColorPercent alpha = 1);
    attribute CSSColorRGBComp r;
    attribute CSSColorRGBComp g;
    attribute CSSColorRGBComp b;
    attribute CSSColorPercent alpha;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSHSL : CSSColorValue {
    constructor(CSSColorAngle h, CSSColorPercent s, CSSColorPercent l, optional CSSColorPercent alpha = 1);
    attribute CSSColorAngle h;
    attribute CSSColorPercent s;
    attribute CSSColorPercent l;
    attribute CSSColorPercent alpha;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSHWB : CSSColorValue {
    constructor(CSSNumericValue h, CSSNumberish w, CSSNumberish b, optional CSSNumberish alpha = 1);
    attribute CSSNumericValue h;
    attribute CSSNumberish w;
    attribute CSSNumberish b;
    attribute CSSNumberish alpha;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSLab : CSSColorValue {
    constructor(CSSColorPercent l, CSSColorNumber a, CSSColorNumber b, optional CSSColorPercent alpha = 1);
    attribute CSSColorPercent l;
    attribute CSSColorNumber a;
    attribute CSSColorNumber b;
    attribute CSSColorPercent alpha;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSLCH : CSSColorValue {
    constructor(CSSColorPercent l, CSSColorPercent c, CSSColorAngle h, optional CSSColorPercent alpha = 1);
    attribute CSSColorPercent l;
    attribute CSSColorPercent c;
    attribute CSSColorAngle h;
    attribute CSSColorPercent alpha;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSOKLab : CSSColorValue {
    constructor(CSSColorPercent l, CSSColorNumber a, CSSColorNumber b, optional CSSColorPercent alpha = 1);
    attribute CSSColorPercent l;
    attribute CSSColorNumber a;
    attribute CSSColorNumber b;
    attribute CSSColorPercent alpha;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSOKLCH : CSSColorValue {
    constructor(CSSColorPercent l, CSSColorPercent c, CSSColorAngle h, optional CSSColorPercent alpha = 1);
    attribute CSSColorPercent l;
    attribute CSSColorPercent c;
    attribute CSSColorAngle h;
    attribute CSSColorPercent alpha;
};

[Exposed=(Window, Worker, PaintWorklet, LayoutWorklet)]
interface CSSColor : CSSColorValue {
    constructor(CSSKeywordish colorSpace, sequence<CSSColorPercent> channels, optional CSSNumberish alpha = 1);
    attribute CSSKeywordish colorSpace;
    attribute ObservableArray<CSSColorPercent> channels;
    attribute CSSNumberish alpha;
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define the global. [↵](#issue-03265e97)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> How to divide a [list-valued property](#list-valued-properties) into iterations is intentionally undefined and hand-wavey at the moment. <em>Generally</em>, you just split it on top-level commas (corresponding to a top-level `<foo>#` term in the grammar), but some legacy properties (such as [counter-reset](https://www.w3.org/TR/css-lists-3/#propdef-counter-reset)) don’t separate their iterations with commas.
>
> It’s expected to be rigorously defined in the future, but at the moment is explicitly a "you know what we mean" thing.
>
> [↵](#issue-c8d43bc8)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/css-houdini-drafts/644](https://github.com/w3c/css-houdini-drafts/issues/644)[\[css-typed-om\]Define precisely which properties are list-valued and which aren't, probably in an appendix.](https://github.com/w3c/css-houdini-drafts/issues/644) [↵](#issue-75ed902d)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define the global. [↵](#issue-03265e97%E2%91%A0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define the global. [↵](#issue-03265e97%E2%91%A1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> TODO add stringifiers [↵](#issue-0e2e471e)

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome105+

------------------------------------------------------------------------

Opera?Edge105+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome105+

------------------------------------------------------------------------

Opera?Edge105+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome105+

------------------------------------------------------------------------

Opera?Edge105+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome105+

------------------------------------------------------------------------

Opera?Edge105+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome105+

------------------------------------------------------------------------

Opera?Edge105+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome105+

------------------------------------------------------------------------

Opera?Edge105+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome108+

------------------------------------------------------------------------

Opera?Edge108+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSS/factory_functions_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/factory_functions_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSImageValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSImageValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSKeywordValue/CSSKeywordValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSKeywordValue/CSSKeywordValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSKeywordValue/value](https://developer.mozilla.org/en-US/docs/Web/API/CSSKeywordValue/value)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSKeywordValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSKeywordValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathInvert/CSSMathInvert](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathInvert/CSSMathInvert)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathInvert/value](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathInvert/value)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathInvert](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathInvert)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<b title="This feature is in less than two current engines.">⚠</b>MDN

[CSSMathMax/CSSMathMax](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathMax/CSSMathMax)

In only one current engine.

FirefoxNoneSafariNoneChrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathMax/values](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathMax/values)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathMax](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathMax)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<b title="This feature is in less than two current engines.">⚠</b>MDN

[CSSMathMin/CSSMathMin](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathMin/CSSMathMin)

In only one current engine.

FirefoxNoneSafariNoneChrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathMin/values](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathMin/values)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathMin](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathMin)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathNegate/CSSMathNegate](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathNegate/CSSMathNegate)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathNegate/value](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathNegate/value)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathNegate](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathNegate)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<b title="This feature is in less than two current engines.">⚠</b>MDN

[CSSMathProduct/CSSMathProduct](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathProduct/CSSMathProduct)

In only one current engine.

FirefoxNoneSafariNoneChrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathProduct/values](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathProduct/values)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathProduct](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathProduct)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<b title="This feature is in less than two current engines.">⚠</b>MDN

[CSSMathSum/CSSMathSum](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathSum/CSSMathSum)

In only one current engine.

FirefoxNoneSafariNoneChrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathSum/values](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathSum/values)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathSum](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathSum)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathValue/operator](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathValue/operator)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMathValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSMathValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMatrixComponent/CSSMatrixComponent](https://developer.mozilla.org/en-US/docs/Web/API/CSSMatrixComponent/CSSMatrixComponent)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMatrixComponent/matrix](https://developer.mozilla.org/en-US/docs/Web/API/CSSMatrixComponent/matrix)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSMatrixComponent](https://developer.mozilla.org/en-US/docs/Web/API/CSSMatrixComponent)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericArray/length](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericArray/length)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericArray](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericArray)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/add](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/add)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/div](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/div)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/equals](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/equals)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/max](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/max)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/min](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/min)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/mul](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/mul)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/parse_static](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/parse_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/sub](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/sub)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/to](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/to)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/toSum](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/toSum)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue/type](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue/type)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSNumericValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSNumericValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSPerspective/CSSPerspective](https://developer.mozilla.org/en-US/docs/Web/API/CSSPerspective/CSSPerspective)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSPerspective/length](https://developer.mozilla.org/en-US/docs/Web/API/CSSPerspective/length)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSPerspective](https://developer.mozilla.org/en-US/docs/Web/API/CSSPerspective)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSRotate/CSSRotate](https://developer.mozilla.org/en-US/docs/Web/API/CSSRotate/CSSRotate)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSRotate/CSSRotate](https://developer.mozilla.org/en-US/docs/Web/API/CSSRotate/CSSRotate)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSRotate/angle](https://developer.mozilla.org/en-US/docs/Web/API/CSSRotate/angle)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSRotate/x](https://developer.mozilla.org/en-US/docs/Web/API/CSSRotate/x)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSRotate/y](https://developer.mozilla.org/en-US/docs/Web/API/CSSRotate/y)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSRotate/z](https://developer.mozilla.org/en-US/docs/Web/API/CSSRotate/z)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSRotate](https://developer.mozilla.org/en-US/docs/Web/API/CSSRotate)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSScale/CSSScale](https://developer.mozilla.org/en-US/docs/Web/API/CSSScale/CSSScale)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSScale/x](https://developer.mozilla.org/en-US/docs/Web/API/CSSScale/x)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSScale/y](https://developer.mozilla.org/en-US/docs/Web/API/CSSScale/y)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSScale/z](https://developer.mozilla.org/en-US/docs/Web/API/CSSScale/z)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSScale](https://developer.mozilla.org/en-US/docs/Web/API/CSSScale)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSSkew/CSSSkew](https://developer.mozilla.org/en-US/docs/Web/API/CSSSkew/CSSSkew)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSSkew/ax](https://developer.mozilla.org/en-US/docs/Web/API/CSSSkew/ax)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSSkew/ay](https://developer.mozilla.org/en-US/docs/Web/API/CSSSkew/ay)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSSkew](https://developer.mozilla.org/en-US/docs/Web/API/CSSSkew)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSSkewX/CSSSkewX](https://developer.mozilla.org/en-US/docs/Web/API/CSSSkewX/CSSSkewX)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSSkewX/ax](https://developer.mozilla.org/en-US/docs/Web/API/CSSSkewX/ax)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSSkewX](https://developer.mozilla.org/en-US/docs/Web/API/CSSSkewX)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSSkewY/CSSSkewY](https://developer.mozilla.org/en-US/docs/Web/API/CSSSkewY/CSSSkewY)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSSkewY/ay](https://developer.mozilla.org/en-US/docs/Web/API/CSSSkewY/ay)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSSkewY](https://developer.mozilla.org/en-US/docs/Web/API/CSSSkewY)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSStyleRule/styleMap](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleRule/styleMap)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSStyleValue/parse_static](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleValue/parse_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSStyleValue/parseAll_static](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleValue/parseAll_static)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSStyleValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTransformComponent/is2D](https://developer.mozilla.org/en-US/docs/Web/API/CSSTransformComponent/is2D)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTransformComponent/toMatrix](https://developer.mozilla.org/en-US/docs/Web/API/CSSTransformComponent/toMatrix)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTransformComponent/toString](https://developer.mozilla.org/en-US/docs/Web/API/CSSTransformComponent/toString)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTransformComponent](https://developer.mozilla.org/en-US/docs/Web/API/CSSTransformComponent)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTransformValue/CSSTransformValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSTransformValue/CSSTransformValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTransformValue/is2D](https://developer.mozilla.org/en-US/docs/Web/API/CSSTransformValue/is2D)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTransformValue/length](https://developer.mozilla.org/en-US/docs/Web/API/CSSTransformValue/length)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTransformValue/toMatrix](https://developer.mozilla.org/en-US/docs/Web/API/CSSTransformValue/toMatrix)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTransformValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSTransformValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTranslate/CSSTranslate](https://developer.mozilla.org/en-US/docs/Web/API/CSSTranslate/CSSTranslate)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTranslate/x](https://developer.mozilla.org/en-US/docs/Web/API/CSSTranslate/x)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTranslate/y](https://developer.mozilla.org/en-US/docs/Web/API/CSSTranslate/y)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTranslate/z](https://developer.mozilla.org/en-US/docs/Web/API/CSSTranslate/z)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSTranslate](https://developer.mozilla.org/en-US/docs/Web/API/CSSTranslate)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSUnitValue/CSSUnitValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSUnitValue/CSSUnitValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSUnitValue/unit](https://developer.mozilla.org/en-US/docs/Web/API/CSSUnitValue/unit)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSUnitValue/value](https://developer.mozilla.org/en-US/docs/Web/API/CSSUnitValue/value)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSUnitValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSUnitValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSUnparsedValue/CSSUnparsedValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSUnparsedValue/CSSUnparsedValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSUnparsedValue/length](https://developer.mozilla.org/en-US/docs/Web/API/CSSUnparsedValue/length)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSUnparsedValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSUnparsedValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSVariableReferenceValue/CSSVariableReferenceValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSVariableReferenceValue/CSSVariableReferenceValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSVariableReferenceValue/fallback](https://developer.mozilla.org/en-US/docs/Web/API/CSSVariableReferenceValue/fallback)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSVariableReferenceValue/variable](https://developer.mozilla.org/en-US/docs/Web/API/CSSVariableReferenceValue/variable)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[CSSVariableReferenceValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSVariableReferenceValue)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[Element/computedStyleMap](https://developer.mozilla.org/en-US/docs/Web/API/Element/computedStyleMap)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[StylePropertyMap/append](https://developer.mozilla.org/en-US/docs/Web/API/StylePropertyMap/append)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[StylePropertyMap/clear](https://developer.mozilla.org/en-US/docs/Web/API/StylePropertyMap/clear)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[StylePropertyMap/delete](https://developer.mozilla.org/en-US/docs/Web/API/StylePropertyMap/delete)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[StylePropertyMap/set](https://developer.mozilla.org/en-US/docs/Web/API/StylePropertyMap/set)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[StylePropertyMap](https://developer.mozilla.org/en-US/docs/Web/API/StylePropertyMap)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

[StylePropertyMapReadOnly](https://developer.mozilla.org/en-US/docs/Web/API/StylePropertyMapReadOnly)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[StylePropertyMapReadOnly/get](https://developer.mozilla.org/en-US/docs/Web/API/StylePropertyMapReadOnly/get)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[StylePropertyMapReadOnly/getAll](https://developer.mozilla.org/en-US/docs/Web/API/StylePropertyMapReadOnly/getAll)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[StylePropertyMapReadOnly/has](https://developer.mozilla.org/en-US/docs/Web/API/StylePropertyMapReadOnly/has)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[StylePropertyMapReadOnly/size](https://developer.mozilla.org/en-US/docs/Web/API/StylePropertyMapReadOnly/size)

FirefoxNoneSafari16.4+Chrome66+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?
