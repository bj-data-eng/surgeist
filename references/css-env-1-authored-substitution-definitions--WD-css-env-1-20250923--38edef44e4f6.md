Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Environment Variables Module Level 1](https://www.w3.org/TR/2025/WD-css-env-1-20250923/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Environment Variables Module Level 1

Source snapshot: https://www.w3.org/TR/2025/WD-css-env-1-20250923/

Snapshot SHA-256: 38edef44e4f698039cebdfca3614dd3aec0ed15e1d61ded3383b88796d4c10ba

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 4 source tables are presented as readable Markdown tables or explicit labeled layouts: 4 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Environment Variables Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-css-environment-variable"></a>

<a id="ref-for-funcdef-env"></a>

<a id="ref-for-funcdef-var"></a>

This specification defines the concept of [environment variables](#css-environment-variable) and the [env()](#funcdef-env) function, which work similarly to custom properties and the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function, but are defined globally for a document. These can be defined either by the user agent, providing values that can be used on the page based on information the UA has special access to, or provided by the author for "global" variables that are guaranteed to be the same no matter where in the document they’re used.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>First Public Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a First Public Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-env” in the title, like this: “\[css-env\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-env%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1. Introduction

<a id="ref-for-custom-property"></a>

<a id="ref-for-funcdef-var①"></a>

The [\[css-variables-1\]](#biblio-css-variables-1) specification defined the concept of "cascading variables", author-defined variables created from the value of [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property), capable of being substituted into arbitrary other properties via the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function.

<a id="ref-for-css-environment-variable①"></a>

<a id="ref-for-custom-property①"></a>

<a id="ref-for-funcdef-env①"></a>

<a id="ref-for-funcdef-var②"></a>

This specification defines a related, but simpler, concept of [environment variables](#css-environment-variable). Unlike "cascading variables", which can change throughout the page as their corresponding [custom property](https://www.w3.org/TR/css-variables-1/#custom-property) takes on different values, an <a id="ref-for-css-environment-variable②"></a>environment variable is "global" to a particular document—​its value is the same everywhere. The [env()](#funcdef-env) function can then be used to substitute the value into arbitrary locations, similar to the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function.

These "global" variables have both benefits and downsides versus cascading variables:

- <a id="ref-for-css-environment-variable③"></a>

  <a id="ref-for-custom-property②"></a>

  Many variables aren’t meant to change over the course of a page; they set up themes, or are helpers for particular numerical values. Using [environment variables](#css-environment-variable) instead of [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) to define these communicates the proper intent, which is good both for the author of the document (particularly when multiple people are collaborating on a single document), and for the user agent, as it can store these variables in a more optimal way.

- <a id="ref-for-css-environment-variable④"></a>

  <a id="ref-for-at-ruledef-media"></a>

  <a id="ref-for-funcdef-var③"></a>

  Because [environment variables](#css-environment-variable) don’t depend on the value of anything drawn from a particular element, they can be used in places where there is no obvious element to draw from, such as in [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) rules, where the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function would not be valid.

- <a id="ref-for-funcdef-env②"></a>

  <a id="ref-for-funcdef-var④"></a>

  Information from the user agent itself, such as the margin of the viewport to avoid laying out in by default (for example, to avoid overlapping a "notch" in the screen), can be retrieved via [env()](#funcdef-env), whereas the element-specific nature of [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) was not an appropriate place to pipe that information in.

<a id="ref-for-css-environment-variable⑤"></a>

Most [environment variables](#css-environment-variable) will have a single value at a time. Some, however, are "indexed", representing multiple values at once, such as the sizes and positions of several distinct panes of content in the viewport-segment-\* variables. To refer to these indexed variables, one or more integers must be provided alongside the variable name, like viewport-segment-width 1 0, to select a single value from the list or grid of possibilities, similar to selecting one element from a list in a traditional programming language with a syntax like `values[0]`.

## <a id="environment"></a>2. Environment Variables

<a id="ref-for-typedef-declaration-value"></a>

<a id="ref-for-custom-property③"></a>

<a id="ref-for-css-environment-variable⑥"></a>

<a id="ref-for-typedef-custom-property-name"></a>

A CSS <a id="css-environment-variable"></a>environment variable is a name associated with a [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value) (a sequence of zero more CSS tokens, with almost no restrictions on what tokens can exist), similar to a [custom property](https://www.w3.org/TR/css-variables-1/#custom-property). [Environment variables](#css-environment-variable) can be defined by the user agent, or by the user. (In the latter case, the names are [\<custom-property-name\>](https://www.w3.org/TR/css-variables-1/#typedef-custom-property-name)s, and start with \`--\` per standard for custom identifiers.)

<a id="ref-for-css-environment-variable⑦"></a>

<a id="ref-for-document"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c17a17b4"></a> Is the set of UA-defined [environment variables](#css-environment-variable) visible to script? If so, define an API on <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> to expose them.

<a id="ref-for-css-environment-variable⑧"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-ed0904e4"></a> Define how authors can add [environment variables](#css-environment-variable), preferably both via JS and via CSS. Note that mixing CSS rules and JS-defined stuff can easily get messy, as demonstrated by CSSFontFaceRule vs FontFace...

<a id="ref-for-css-environment-variable⑨"></a>

The following UA-defined [environment variables](#css-environment-variable) are officially defined and must be supported. Additional UA-defined <a id="ref-for-css-environment-variable①⓪"></a>environment variables <em>must not</em> be supported unless/until they are added to this list.

### <a id="safe-area-insets"></a>2.1. Safe area inset variables

| Name                                      | Value                                                                             | Number of dimensions |
|-------------------------------------------|-----------------------------------------------------------------------------------|----------------------|
| <a id="valdef-env-safe-area-inset-top"></a>safe-area-inset-top    | <a id="ref-for-length-value"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 0 (scalar)           |
| <a id="valdef-env-safe-area-inset-right"></a>safe-area-inset-right  | <a id="ref-for-length-value①"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 0 (scalar)           |
| <a id="valdef-env-safe-area-inset-bottom"></a>safe-area-inset-bottom | <a id="ref-for-length-value②"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 0 (scalar)           |
| <a id="valdef-env-safe-area-inset-left"></a>safe-area-inset-left   | <a id="ref-for-length-value③"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 0 (scalar)           |

<a id="ref-for-css-environment-variable①①"></a>

The safe area insets are four [environment variables](#css-environment-variable) that define a rectangle by its top, right, bottom, and left insets from the edge of the viewport. For rectangular displays, these must all be zero, but for nonrectangular displays they must form a rectangle, chosen by the user agent, such that all content inside the rectangle is visible, and such that reducing any of the insets would cause some content inside of the rectangle to be invisible due to the nonrectangular nature of the display. This allows authors to limit the layout of essential content to the space inside of the safe area rectangle.

### <a id="safe-area-max-insets"></a>2.2. Safe area maximum inset variables

| Name                                          | Value                                                                             | Number of dimensions |
|-----------------------------------------------|-----------------------------------------------------------------------------------|----------------------|
| <a id="valdef-env-safe-area-max-inset-top"></a>safe-area-max-inset-top    | <a id="ref-for-length-value④"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 0 (scalar)           |
| <a id="valdef-env-safe-area-max-inset-right"></a>safe-area-max-inset-right  | <a id="ref-for-length-value⑤"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 0 (scalar)           |
| <a id="valdef-env-safe-area-max-inset-bottom"></a>safe-area-max-inset-bottom | <a id="ref-for-length-value⑥"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 0 (scalar)           |
| <a id="valdef-env-safe-area-max-inset-left"></a>safe-area-max-inset-left   | <a id="ref-for-length-value⑦"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 0 (scalar)           |

<a id="ref-for-css-environment-variable①②"></a>

<a id="ref-for-layout-viewport"></a>

<a id="ref-for-large-viewport-size"></a>

The safe area maximum insets are four [environment variables](#css-environment-variable) that are tied to the [safe area inset variables](#safe-area-insets). Unlike the safe area inset variables which are dynamic values, the safe area maximum insets are static values that represent the maximum value of their dynamic counterpart when dynamic UA interfaces are retracted, making the [layout viewport](https://www.w3.org/TR/cssom-view-1/#layout-viewport) size the [large viewport size](https://www.w3.org/TR/css-values-4/#large-viewport-size).

### <a id="viewport-segments"></a>2.3. Viewport segment variables

| Name                                       | Value                                                                             | Number of dimensions |
|--------------------------------------------|-----------------------------------------------------------------------------------|----------------------|
| <a id="valdef-env-viewport-segment-width"></a>viewport-segment-width  | <a id="ref-for-length-value⑧"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 2                    |
| <a id="valdef-env-viewport-segment-height"></a>viewport-segment-height | <a id="ref-for-length-value⑨"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 2                    |
| <a id="valdef-env-viewport-segment-top"></a>viewport-segment-top    | <a id="ref-for-length-value①⓪"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 2                    |
| <a id="valdef-env-viewport-segment-left"></a>viewport-segment-left   | <a id="ref-for-length-value①①"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 2                    |
| <a id="valdef-env-viewport-segment-bottom"></a>viewport-segment-bottom | <a id="ref-for-length-value①②"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 2                    |
| <a id="valdef-env-viewport-segment-right"></a>viewport-segment-right  | <a id="ref-for-length-value①③"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) | 2                    |

<a id="ref-for-css-environment-variable①③"></a>

The viewport segments are [environment variables](#css-environment-variable) that define the position and dimensions of a logically separate region of the viewport. Viewport segments are created when the viewport is split by one or more hardware features (such as a fold or a hinge between separate displays) that act as a divider; segments are the regions of the viewport that can be treated as logically distinct by the author.

<a id="ref-for-css-environment-variable①④"></a>

The viewport segment [environment variables](#css-environment-variable) have two dimensions, which represent the x and y position, respectively, in the two dimensional grid created by the hardware features separating the segments. Segments along the left edge have x position 0, those in the next column to the right have x position 1, etc. Similarly, segments along the top edge have y position 0, etc.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In certain hardware configurations, the separator itself may occupy logical space within the viewport. The dimensions of the separator can be computed by calculating the area between the position of the viewport segments.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-87d68a03"></a> When the viewport is split into two side-by-side segments, the viewport segment on the left would have indices (0, 0). Its width would be represented as env(viewport-segment-width 0 0, 300px). The viewport segment on the right would have indices (1, 0). Similarly, for a viewport split into two vertical segments, the viewport segment on the top would have indices (0, 0) and the one on the bottom (0, 1).

These variables are only defined when there are at least two such segments. Viewport units should be used instead when there is no hardware feature splitting the viewport, otherwise content will not display as intended when viewed on a device with multiple segments.

### <a id="text-zoom"></a>2.4. Preferred Text Zoom

| Name                                    | Value                                                                             | Number of dimensions |
|-----------------------------------------|-----------------------------------------------------------------------------------|----------------------|
| <a id="valdef-env-preferred-text-scale"></a>preferred-text-scale | <a id="ref-for-number-value"></a>[\<number\>](https://www.w3.org/TR/css-values-4/#number-value) | 0 (scalar)           |

<a id="ref-for-valdef-env-preferred-text-scale"></a>

<a id="ref-for-css-environment-variable①⑤"></a>

<a id="ref-for-propdef-text-size-adjust"></a>

The [preferred-text-scale](#valdef-env-preferred-text-scale) [environment variable](#css-environment-variable) represents the user’s preferred text scale factor; aka, the adjustment they make to the "default" font size of the OS and/or user agent. (On devices where [text-size-adjust](https://drafts.csswg.org/css-size-adjust-1/#propdef-text-size-adjust) has an effect, this is the scale factor applied by <a id="ref-for-propdef-text-size-adjust①"></a>text-size-adjust: auto.)

For example, if text-size-adjust:auto would cause text sizes to double, then env(preferred-text-scale) would resolve to 2.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The pem unit represents this same information; 1em is exactly equivalent to calc(1em \* env(preferred-text-scale)). When directly sizing things, bsem is just a more convenient length to use.

<a id="ref-for-css-environment-variable①⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-301ba781"></a> This [environment variable](#css-environment-variable) requires care to be used correctly. By default, text scaling is applied automatically; using env(preferred-text-scale) or pem would result in the scale being <em>double</em>-applied, making text or UI elements too large.
>
> Typically, authors should either:
>
> - <a id="ref-for-propdef-text-size-adjust②"></a>
>
>   set [text-size-adjust: calc(100% \* env(preferred-text-scale));](https://drafts.csswg.org/css-size-adjust-1/#propdef-text-size-adjust), to ensure that all the text in the page is automatically scaled to the user’s preference, and when necessary scale non-text sizes by the scale factor as well.
>
> - <a id="ref-for-css-environment-variable①⑦"></a>
>
>   or set text-size-adjust:none, and then consistently use this [environment variable](#css-environment-variable) and/or the pem unit to scale relevant text and UI to the user’s preference.

<a id="ref-for-funcdef-env③"></a>

## <a id="env-function"></a>3. Using Environment Variables: the [env()](#funcdef-env) notation

<a id="ref-for-css-environment-variable①⑧"></a>

<a id="ref-for-funcdef-env④"></a>

In order to substitute the value of an [environment variable](#css-environment-variable) into a CSS context, use the [env()](#funcdef-env) function:

<a id="funcdef-env"></a>

<a id="ref-for-identifier-value"></a>

<a id="ref-for-integer-value"></a>

<a id="ref-for-mult-zero-plus"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-declaration-value①"></a>

<a id="ref-for-mult-opt"></a>

```text
env() = env( <custom-ident> <integer [0,∞]>*, <declaration-value>? )
```
<a id="ref-for-funcdef-env⑤"></a>

<a id="ref-for-at-rule"></a>

The [env()](#funcdef-env) function can be used in place of any part of a value in any property on any element, or any part of a value in any descriptor on any [at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule), and in several other places where CSS values are allowed.

<a id="ref-for-funcdef-env⑥"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9ee89738"></a> Define the full set of places [env()](#funcdef-env) can be used.
>
> - Should be able to replace any subset of MQ syntax, for example.
>
> - Should be able to replace selectors, maybe?
>
> - Should it work on a rule level, so you can insert arbitrary stuff into a rule, like reusing a block of declarations?

<a id="ref-for-funcdef-env⑦"></a>

<a id="ref-for-css-environment-variable①⑨"></a>

The first argument to [env()](#funcdef-env) provides the name of an [environment variable](#css-environment-variable) to be substituted. Following the first argument are integers that represent indices into the dimensions of the <a id="ref-for-css-environment-variable②⓪"></a>environment variable, if the provided name represents an array-like <a id="ref-for-css-environment-variable②①"></a>environment variable. The argument after the comma, if provided, is a fallback value, which is used as the substitution value when the referenced <a id="ref-for-css-environment-variable②②"></a>environment variable does not exist.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The syntax of the fallback, like that of custom properties, allows commas. For example, env(foo, red, blue) defines a fallback of red, blue; that is, anything between the first comma and the end of the function is considered a fallback value.

<a id="ref-for-funcdef-env⑧"></a>

<a id="ref-for-substitute-an-env"></a>

If a property contains one or more [env()](#funcdef-env) functions, and those functions are syntactically valid, the entire property’s grammar must be assumed to be valid at parse time. It is only syntax-checked at computed-time, after <a id="ref-for-funcdef-env⑨"></a>env() functions have been [substituted](#substitute-an-env).

<a id="ref-for-funcdef-env①⓪"></a>

<a id="ref-for-substitute-an-env①"></a>

If a descriptor contains one or more [env()](#funcdef-env) functions, and those functions are syntactically valid, the entire declaration’s grammar must be assumed to be valid at parse time. It is only syntax-checked after <a id="ref-for-funcdef-env①①"></a>env() functions have been [substituted](#substitute-an-env).

To <a id="substitute-an-env"></a>substitute an env() in a property or descriptor:

1.  <a id="ref-for-funcdef-env①②"></a>

    <a id="ref-for-css-environment-variable②③"></a>

    If the name provided by the first argument of the [env()](#funcdef-env) function is a recognized [environment variable](#css-environment-variable) name, the number of supplied integers matches the number of dimensions of the <a id="ref-for-css-environment-variable②④"></a>environment variable referenced by that name, and values of the indices correspond to a known sub-value, replace the <a id="ref-for-funcdef-env①③"></a>env() function by the value of the named <a id="ref-for-css-environment-variable②⑤"></a>environment variable.

2.  <a id="ref-for-funcdef-env①④"></a>

    <a id="ref-for-substitute-an-env②"></a>

    Otherwise, if the [env()](#funcdef-env) function has a fallback value as its second argument, replace the <a id="ref-for-funcdef-env①⑤"></a>env() function by the fallback value. If there are any <a id="ref-for-funcdef-env①⑥"></a>env() references in the fallback, [substitute](#substitute-an-env) them as well.

3.  <a id="ref-for-funcdef-env①⑦"></a>

    <a id="ref-for-invalid-at-computed-value-time"></a>

    Otherwise, the property or descriptor containing the [env()](#funcdef-env) function is [invalid at computed-value time](https://www.w3.org/TR/css-variables-1/#invalid-at-computed-value-time).

<a id="ref-for-funcdef-var⑤"></a>

<a id="ref-for-funcdef-env①⑧"></a>

<a id="ref-for-custom-property④"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-42f9a2dd"></a> Define when substitution happens. It has to be before [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) substitution. Alternately, should [env()](#funcdef-env) substitution happen at parse time, so unknown variable names cause it to fail syntax checking? There’s no particular reason to have it happen at computed-value time, like <a id="ref-for-funcdef-var⑥"></a>var() does—​that was to ensure that [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) could inherit their value down before they were picked up by a <a id="ref-for-funcdef-var⑦"></a>var().

<a id="ref-for-funcdef-env①⑨"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9806265e"></a> When I figure out where else [env()](#funcdef-env) can go, define how/when it substitutes.

### <a id="env-in-shorthands"></a>3.1. Environment Variables in Shorthand Properties

<a id="ref-for-funcdef-env②⓪"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-66b99b40"></a> If [env()](#funcdef-env) substitution happens during parsing, then this is unnecessary.

<a id="ref-for-funcdef-env②①"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-funcdef-var⑧"></a>

The [env()](#funcdef-env) function causes the same difficulties with [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) as the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function does. When an <a id="ref-for-funcdef-env②②"></a>env() is used in a <a id="ref-for-shorthand-property①"></a>shorthand property, then, it has the same effects as defined in [CSS Variables 1 § 3.2 Variables in Shorthand Properties](https://www.w3.org/TR/css-variables-1/#variables-in-shorthands).

## <a id="priv"></a>4.  Privacy Considerations

<a id="ref-for-css-environment-variable②⑥"></a>

The [environment variables](#css-environment-variable) defined by this specification are <em>potentially</em> privacy-sensitive, since they represent additional information potentially not already avaialble to the page. In particular, they potentially represent a fingerprinting vector, by exposing additional information about the device a user is viewing the page with.

<a id="ref-for-css-environment-variable②⑦"></a>

So far, the [environment variables](#css-environment-variable) defined by this specifcation have been reviewed and deemed acceptable to expose by the CSSWG.

## <a id="sec"></a>5.  Security Considerations

This specification provides read-only access to some new types of information about the device.

<a id="ref-for-css-environment-variable②⑧"></a>

The [environment variables](#css-environment-variable) defined by this specification do not expose any security-sensitive information.

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

- [env()](#funcdef-env), in § 3
- [environment variable](#css-environment-variable), in § 2
- [preferred-text-scale](#valdef-env-preferred-text-scale), in § 2.4
- [safe-area-inset-bottom](#valdef-env-safe-area-inset-bottom), in § 2.1
- [safe-area-inset-left](#valdef-env-safe-area-inset-left), in § 2.1
- [safe-area-inset-right](#valdef-env-safe-area-inset-right), in § 2.1
- [safe-area-inset-top](#valdef-env-safe-area-inset-top), in § 2.1
- [safe-area-max-inset-bottom](#valdef-env-safe-area-max-inset-bottom), in § 2.2
- [safe-area-max-inset-left](#valdef-env-safe-area-max-inset-left), in § 2.2
- [safe-area-max-inset-right](#valdef-env-safe-area-max-inset-right), in § 2.2
- [safe-area-max-inset-top](#valdef-env-safe-area-max-inset-top), in § 2.2
- [substitute](#substitute-an-env), in § 3
- [substitute an env()](#substitute-an-env), in § 3
- [viewport-segment-bottom](#valdef-env-viewport-segment-bottom), in § 2.3
- [viewport-segment-height](#valdef-env-viewport-segment-height), in § 2.3
- [viewport-segment-left](#valdef-env-viewport-segment-left), in § 2.3
- [viewport-segment-right](#valdef-env-viewport-segment-right), in § 2.3
- [viewport-segment-top](#valdef-env-viewport-segment-top), in § 2.3
- [viewport-segment-width](#valdef-env-viewport-segment-width), in § 2.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="980ac56a"></a>shorthand property
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="4397147f"></a>@media
- \[CSS-SIZE-ADJUST-1\] defines the following terms:
  - <a id="194f46b6"></a>text-size-adjust
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="04853566"></a>\<declaration-value\>
  - <a id="b29fecf5"></a>at-rule
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="ef9f8297"></a>\*
  - <a id="8cd4f032"></a>,
  - <a id="a0144f62"></a>\<custom-ident\>
  - <a id="d73c993d"></a>\<integer\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="d4441b24"></a>?
  - <a id="2ec4d9a2"></a>large viewport size
- \[CSS-VARIABLES-1\] defines the following terms:
  - <a id="310b3140"></a>\<custom-property-name\>
  - <a id="5550667d"></a>custom property
  - <a id="81bc970d"></a>invalid at computed-value time
  - <a id="3beec8c9"></a>var()
- \[CSSOM-VIEW-1\] defines the following terms:
  - <a id="bc23ba5b"></a>layout viewport
- \[DOM\] defines the following terms:
  - <a id="85394472"></a>Document

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 15 August 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-size-adjust-1"></a>\[CSS-SIZE-ADJUST-1\]  
[CSS Mobile Text Size Adjustment Module Level 1](https://drafts.csswg.org/css-size-adjust-1/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-size-adjust-1&#x2F;](https://drafts.csswg.org/css-size-adjust-1/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 16 June 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-cssom-view-1"></a>\[CSSOM-VIEW-1\]  
Simon Fraser; Emilio Cobos Álvarez. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 16 September 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Is the set of UA-defined [environment variables](#css-environment-variable) visible to script? If so, define an API on <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> to expose them. [↵](#issue-c17a17b4)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define how authors can add [environment variables](#css-environment-variable), preferably both via JS and via CSS. Note that mixing CSS rules and JS-defined stuff can easily get messy, as demonstrated by CSSFontFaceRule vs FontFace... [↵](#issue-ed0904e4)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define the full set of places [env()](#funcdef-env) can be used.
>
> - Should be able to replace any subset of MQ syntax, for example.
>
> - Should be able to replace selectors, maybe?
>
> - Should it work on a rule level, so you can insert arbitrary stuff into a rule, like reusing a block of declarations?
>
> [↵](#issue-9ee89738)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define when substitution happens. It has to be before [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) substitution. Alternately, should [env()](#funcdef-env) substitution happen at parse time, so unknown variable names cause it to fail syntax checking? There’s no particular reason to have it happen at computed-value time, like var() does—​that was to ensure that [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) could inherit their value down before they were picked up by a var(). [↵](#issue-42f9a2dd)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> When I figure out where else [env()](#funcdef-env) can go, define how/when it substitutes. [↵](#issue-9806265e)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> If [env()](#funcdef-env) substitution happens during parsing, then this is unnecessary. [↵](#issue-66b99b40)
