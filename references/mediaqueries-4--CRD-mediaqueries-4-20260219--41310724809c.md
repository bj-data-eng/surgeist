Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Media Queries Level 4](https://www.w3.org/TR/2026/CRD-mediaqueries-4-20260219/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Media Queries Level 4

Source snapshot: https://www.w3.org/TR/2026/CRD-mediaqueries-4-20260219/

Snapshot SHA-256: 41310724809c30304b674442d35e09ec59b0b7b74ba6104bb1d90334953522e0

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 4 inline SVG diagrams are retained as local passive SVG assets, with original geometry and visible source diagram text. Supporting assets are not reference documents.
- The 24 source tables are presented as readable Markdown tables or explicit labeled layouts: 23 ordinary table conversions, 1 complex-table layout. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>Media Queries Level 4

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-media-query"></a>

[Media Queries](#media-query) allow authors to test and query values or features of the user agent or display device, independent of the document being rendered. They are used in the CSS @media rule to conditionally apply styles to a document, and in various other contexts and languages, such as HTML and JavaScript.

Media Queries Level 4 describes the mechanism and syntax of media queries, media types, and media features. It extends and supersedes the features defined in Media Queries Level 3.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “mediaqueries” in the title, like this: “\[mediaqueries\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bmediaqueries%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

There is currently no preliminary interoperability or implementation report.

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

<a id="ref-for-media-type"></a>

In 1997, HTML4 [\[HTML401\]](#biblio-html401) defined a mechanism to support media-dependent style sheets, tailored for different [media types](#media-type). For example, a document may use different style sheets for screen and for print. In HTML, this can be written as:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f12a7f15"></a>
>
> ```text
> <link rel="stylesheet" type="text/css" media="screen" href="style.css">
> <link rel="stylesheet" type="text/css" media="print" href="print.css">
> ```
<a id="ref-for-at-ruledef-media"></a>

<a id="ref-for-at-ruledef-import"></a>

CSS adapted and extended this functionality with its [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) and [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) rules, adding the ability to query the value of individual features:

<a id="ref-for-media-type①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-23ee34e2"></a> Inside a CSS style sheet, one can declare that sections apply to certain [media types](#media-type):
>
> ```text
> @media screen {
>   * { font-family: sans-serif }
> }
> ```
>
> Similarly, stylesheets can be conditionally imported based on media queries:
>
> ```text
> @import "print-styles.css" print;
> ```
<a id="ref-for-media-query①"></a>

[Media queries](#media-query) can be used with HTML, XHTML, XML [\[xml-stylesheet\]](#biblio-xml-stylesheet) and the @import and @media rules of CSS.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b1c63ddf"></a> Here is the same example written in HTML, XHTML, XML, @import and @media:
>
> ```text
> <link media="screen and (color), projection and (color)"
>       rel="stylesheet" href="example.css">
> 
> <link media="screen and (color), projection and (color)"
>       rel="stylesheet" href="example.css" />
> 
> <?xml-stylesheet media="screen and (color), projection and (color)"
>                  rel="stylesheet" href="example.css" ?>
> 
> @import url(example.css) screen and (color), projection and (color);
> 
> @media screen and (color), projection and (color) { … }
> ```
### <a id="placement"></a>1.1.  Module interactions

This module replaces and extends the Media Queries, Media Type and Media Features defined in [\[CSS2\]](#biblio-css2) sections 7 and in [\[MEDIAQUERIES-3\]](#biblio-mediaqueries-3).

### <a id="values"></a>1.2.  Values

<a id="ref-for-integer-value"></a>

<a id="ref-for-number-value"></a>

<a id="ref-for-resolution-value"></a>

Value types not defined in this specification, such as [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value), [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) or [\<resolution\>](https://www.w3.org/TR/css-values-4/#resolution-value), are defined in [\[CSS-VALUES-3\]](#biblio-css-values-3). Other CSS modules may expand the definitions of these value types.

### <a id="units"></a>1.3.  Units

The units used in media queries are the same as in other parts of CSS, as defined in [\[CSS-VALUES-3\]](#biblio-css-values-3). For example, the pixel unit represents CSS pixels and not physical pixels.

<a id="ref-for-relative-length"></a>

<a id="ref-for-initial-value"></a>

<a id="ref-for-em"></a>

<a id="ref-for-initial-value①"></a>

<a id="ref-for-propdef-font-size"></a>

[Relative length](https://www.w3.org/TR/css-values-3/#relative-length) units in media queries are based on the [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value), which means that units are never based on results of declarations. <strong data-conversion-semantic="note">Note:</strong> For example, in HTML, the [em](https://www.w3.org/TR/css-values-3/#em) unit is relative to the [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value) of [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size), defined by the user agent or the user’s preferences, not any styling on the page.

Tests

- [relative-units-001.html](https://wpt.fyi/results/css/mediaqueries/relative-units-001.html) [(live test)](http://wpt.live/css/mediaqueries/relative-units-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/relative-units-001.html)
- [relative-units-002.html](https://wpt.fyi/results/css/mediaqueries/relative-units-002.html) [(live test)](http://wpt.live/css/mediaqueries/relative-units-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/relative-units-002.html)
- [relative-units-003.html](https://wpt.fyi/results/css/mediaqueries/relative-units-003.html) [(live test)](http://wpt.live/css/mediaqueries/relative-units-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/relative-units-003.html)
- [relative-units-004.html](https://wpt.fyi/results/css/mediaqueries/relative-units-004.html) [(live test)](http://wpt.live/css/mediaqueries/relative-units-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/relative-units-004.html)
- [relative-units-005.html](https://wpt.fyi/results/css/mediaqueries/relative-units-005.html) [(live test)](http://wpt.live/css/mediaqueries/relative-units-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/relative-units-005.html)

## <a id="media"></a>2.  Media Queries

<a id="ref-for-media-query②"></a>

A <a id="media-query"></a>media query is a method of testing certain aspects of the user agent or device that the document is being displayed in. [Media queries](#media-query) are (almost) always independent of the contents of the document, its styling, or any other internal aspect; they’re only dependent on “external” information unless another feature explicitly specifies that it affects the resolution of Media Queries.

<a id="ref-for-media-query③"></a>

<a id="ref-for-media-query-modifier"></a>

<a id="ref-for-media-type②"></a>

<a id="ref-for-media-feature"></a>

The syntax of a [media query](#media-query) consists of an optional [media query modifier](#media-query-modifier), an optional [media type](#media-type), and zero or more [media features](#media-feature):

![Source diagram 1](assets/mediaqueries-4--CRD-mediaqueries-4-20260219--41310724809c--diagram-01.svg)

Diagram text: media condition only not media type and media condition

<a id="ref-for-media-query④"></a>

A [media query](#media-query) is a logical expression that is either true or false. A media query is true if:

- <a id="ref-for-media-type③"></a>

  the [media type](#media-type), if specified, matches the media type of the device where the user agent is running, and

- <a id="ref-for-media-condition"></a>

  the [media condition](#media-condition) is true.

Statements regarding media queries in this section assume the [syntax section](#mq-syntax) is followed. Media queries that do not conform to the syntax are discussed in [§ 3.2 Error Handling](#error-handling). I.e. the syntax takes precedence over requirements in this section.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-724d1b7b"></a> Here is a simple example written in HTML:
>
> ```text
> <link rel="stylesheet" media="screen and (color)" href="example.css" />
> ```
>
> <a id="ref-for-valdef-media-screen"></a>
>
> This example expresses that a certain style sheet (`example.css`) applies to devices of a certain media type ([screen](#valdef-media-screen)) with certain feature (it must be a color screen).
>
> Here is the same media query written in an @import-rule in CSS:
>
> ```text
> @import url(example.css) screen and (color);
> ```
<a id="ref-for-media-query⑤"></a>

User agents must re-evaluate [media queries](#media-query) in response to changes in the user environment that they’re aware of, for example if the device is tiled from landscape to portrait orientation, and change the behavior of any constructs dependent on those <a id="ref-for-media-query⑥"></a>media queries accordingly.

Unless another feature explicitly specifies that it affects the resolution of Media Queries, it is never necessary to apply a style sheet in order to evaluate expressions.

Tests

- media-queries-001.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/media-queries-001.xht)
- media-queries-002.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/media-queries-002.xht)
- media-queries-003.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/media-queries-003.xht)
- [mq-calc-001.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-001.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-001.html)
- [mq-calc-002.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-002.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-002.html)
- [mq-calc-003.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-003.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-003.html)
- [mq-calc-004.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-004.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-004.html)
- [mq-calc-005.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-005.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-005.html)
- [mq-calc-006.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-006.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-006.html)
- [mq-calc-007.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-007.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-007.html)
- [mq-calc-008.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-008.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-008.html)
- [mq-calc-resolution.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-resolution.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-resolution.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-resolution.html)
- [mq-calc-sign-function-001.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-sign-function-001.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-sign-function-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-sign-function-001.html)
- [mq-calc-sign-function-002.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-sign-function-002.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-sign-function-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-sign-function-002.html)
- [mq-calc-sign-function-003.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-sign-function-003.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-sign-function-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-sign-function-003.html)
- [mq-calc-sign-function-004.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-sign-function-004.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-sign-function-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-sign-function-004.html)
- [mq-calc-sign-function-005.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-sign-function-005.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-sign-function-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-sign-function-005.html)
- [mq-calc-sign-function-006.html](https://wpt.fyi/results/css/mediaqueries/mq-calc-sign-function-006.html) [(live test)](http://wpt.live/css/mediaqueries/mq-calc-sign-function-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-calc-sign-function-006.html)
- [mq-dynamic-empty-children.html](https://wpt.fyi/results/css/mediaqueries/mq-dynamic-empty-children.html) [(live test)](http://wpt.live/css/mediaqueries/mq-dynamic-empty-children.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-dynamic-empty-children.html)
- [test_media_queries.html](https://wpt.fyi/results/css/mediaqueries/test_media_queries.html) [(live test)](http://wpt.live/css/mediaqueries/test_media_queries.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/test_media_queries.html)

### <a id="mq-list"></a>2.1.  Combining Media Queries

<a id="ref-for-media-query⑦"></a>

Several [media queries](#media-query) can be combined into a comma-separated <a id="media-query-list"></a>media query list.

![Source diagram 2](assets/mediaqueries-4--CRD-mediaqueries-4-20260219--41310724809c--diagram-02.svg)

Diagram text: media query ,

<a id="ref-for-media-query-list"></a>

<a id="ref-for-media-query⑧"></a>

A [media query list](#media-query-list) is true if <em>any</em> of its component [media queries](#media-query) are true, and false only if <em>all</em> of its component <a id="ref-for-media-query⑨"></a>media queries are false.

<a id="ref-for-media-query-list①"></a>

<a id="ref-for-media-type④"></a>

<a id="ref-for-valdef-media-screen①"></a>

<a id="ref-for-valdef-media-projection"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-75a76747"></a> For example, the following [media query list](#media-query-list) is true if either the [media type](#media-type) is [screen](#valdef-media-screen) and it’s a color device, <strong>or</strong> the <a id="ref-for-media-type⑤"></a>media type is [projection](#valdef-media-projection) and it’s a color device:
>
> ```text
> @media screen and (color), projection and (color) { … }
> ```
<a id="ref-for-media-query-list②"></a>

An empty [media query list](#media-query-list) evaluates to true.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6f06ee45"></a> For example, these are equivalent:
>
> ```text
> @media all { … }
> @media { … }
> ```
### <a id="mq-prefix"></a>2.2.  Media Query Modifiers

<a id="ref-for-media-query①⓪"></a>

A [media query](#media-query) may optionally be prefixed by a single <a id="media-query-modifier"></a>media query modifier, which is a single keyword which alters the meaning of the following <a id="ref-for-media-query①①"></a>media query.

<a id="ref-for-valdef-media-not"></a>

#### <a id="mq-not"></a>2.2.1.  Negating a Media Query: the [not](#valdef-media-not) keyword

<a id="ref-for-media-query①②"></a>

<a id="ref-for-valdef-media-not①"></a>

An individual [media query](#media-query) can have its result negated by prefixing it with the keyword <a id="valdef-media-not"></a>not. If the <a id="ref-for-media-query①③"></a>media query would normally evaluate to true, prefixing it with [not](#valdef-media-not) makes it evaluate to false, and vice versa.

<a id="ref-for-media-type⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b5a260cd"></a> For example, the following will apply to everything except color-capable screens. Note that the entire media query is negated, not just the [media type](#media-type).
>
> ```text
> <link rel="stylesheet" media="not screen and (color)" href="example.css" />
> ```
<a id="ref-for-valdef-media-only"></a>

#### <a id="mq-only"></a>2.2.2.  Hiding a Media Query From Legacy user agents: the [only](#valdef-media-only) keyword

<a id="ref-for-media-query①④"></a>

<a id="ref-for-media-type⑦"></a>

<a id="ref-for-media-feature①"></a>

<a id="ref-for-valdef-media-screen②"></a>

The concept of [media queries](#media-query) originates from HTML4 [\[HTML401\]](#biblio-html401). That specification only defined [media types](#media-type), but had a forward-compatible syntax that accommodated the addition of future concepts like [media features](#media-feature): it would consume the characters of a <a id="ref-for-media-query①⑤"></a>media query up to the first non-alphanumeric character, and interpret that as a <a id="ref-for-media-type⑧"></a>media type, ignoring the rest. For example, the <a id="ref-for-media-query①⑥"></a>media query screen and (color) would be truncated to just [screen](#valdef-media-screen).

<a id="ref-for-media-feature②"></a>

<a id="ref-for-media-query①⑦"></a>

<a id="ref-for-media-type⑨"></a>

Unfortunately, this means that legacy user agents using this error-handling behavior will ignore any [media features](#media-feature) in a [media query](#media-query), even if they’re far more important than the [media type](#media-type) in the query. This can result in styles accidentally being applied in inappropriate situations.

<a id="ref-for-media-query①⑧"></a>

<a id="ref-for-valdef-media-only①"></a>

<a id="ref-for-media-type①⓪"></a>

To hide these [media queries](#media-query) from legacy user agents, the <a id="ref-for-media-query①⑨"></a>media query can be prefixed with the keyword <a id="valdef-media-only"></a>only. The [only](#valdef-media-only) keyword <strong>has no effect</strong> on the <a id="ref-for-media-query②⓪"></a>media query’s result, but will cause the <a id="ref-for-media-query②①"></a>media query to be parsed by legacy user agents as specifying the unknown [media type](#media-type) “only”, and thus be ignored.

<a id="ref-for-valdef-media-screen③"></a>

<a id="ref-for-media-type①①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-88774f15"></a> In this example, the stylesheet specified by the `<link>` element will not be used by legacy user agents, even if they would normally match the [screen](#valdef-media-screen) [media type](#media-type).
>
> ```text
> <link rel="stylesheet" media="only screen and (color)" href="example.css" />
> ```
<a id="ref-for-valdef-media-only②"></a>

<a id="ref-for-media-type①②"></a>

<a id="ref-for-media-query②②"></a>

<a id="ref-for-media-feature③"></a>

<a id="ref-for-media-query-modifier①"></a>

<a id="ref-for-valdef-media-not②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that the [only](#valdef-media-only) keyword can only be used before a [media type](#media-type). A [media query](#media-query) consisting only of [media features](#media-feature), or one with another [media query modifier](#media-query-modifier) like [not](#valdef-media-not), will be treated as false by legacy user agents automatically.

<a id="ref-for-valdef-media-only③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: At the time of publishing this specification, such legacy user agents are extremely rare, and so using the [only](#valdef-media-only) modifier is rarely, if ever, necessary.

### <a id="media-types"></a>2.3.  Media Types

<a id="ref-for-media-type①③"></a>

A <a id="media-type"></a>media type is a broad category of user-agent devices on which a document may be displayed. The original set of [media types](#media-type) were defined in HTML4, for the `media` attribute on `<link>` elements.

<a id="ref-for-media-type①④"></a>

<a id="ref-for-valdef-media-screen④"></a>

<a id="ref-for-valdef-media-handheld"></a>

<a id="ref-for-valdef-media-tty"></a>

<a id="ref-for-valdef-media-tv"></a>

<a id="ref-for-media-feature④"></a>

<a id="ref-for-descdef-media-grid"></a>

<a id="ref-for-descdef-media-scan"></a>

Unfortunately, [media types](#media-type) have proven insufficient as a way of discriminating between devices with different styling needs. Some categories which were originally quite distinct, such as [screen](#valdef-media-screen) and [handheld](#valdef-media-handheld), have blended significantly in the years since their invention. Others, such as [tty](#valdef-media-tty) or [tv](#valdef-media-tv), expose useful differences from the norm of a full-featured computer monitor, and so are potentially useful to target with different styling, but the definition of <a id="ref-for-media-type①⑤"></a>media types as mutually exclusive makes it difficult to use them in a reasonable manner; instead, their exclusive aspects are better expressed as [media features](#media-feature) such as [grid](#descdef-media-grid) or [scan](#descdef-media-scan).

<a id="ref-for-media-type①⑥"></a>

<a id="ref-for-media-query②③"></a>

As such, the following [media types](#media-type) are defined for use in [media queries](#media-query):

<a id="valdef-media-all"></a>all  
Matches all devices.

<a id="valdef-media-print"></a>print  
Matches printers, and devices intended to reproduce a printed display, such as a web browser showing a document in “Print Preview”.

<a id="valdef-media-screen"></a>screen  
<a id="ref-for-valdef-media-print"></a>

Matches all devices that aren’t matched by [print](#valdef-media-print).

<a id="ref-for-media-type①⑦"></a>

<a id="ref-for-media-feature⑤"></a>

In addition, the following <strong>deprecated</strong> [media types](#media-type) are defined. Authors must not use these <a id="ref-for-media-type①⑧"></a>media types; instead, it is recommended that they select appropriate [media features](#media-feature) that better represent the aspect of the device that they are attempting to style against.

<a id="ref-for-media-type①⑨"></a>

User agents must recognize the following [media types](#media-type) as valid, but must make them match nothing.

- <a id="valdef-media-tty"></a>tty
- <a id="valdef-media-tv"></a>tv
- <a id="valdef-media-projection"></a>projection
- <a id="valdef-media-handheld"></a>handheld
- <a id="valdef-media-braille"></a>braille
- <a id="valdef-media-embossed"></a>embossed
- <a id="valdef-media-aural"></a>aural
- <a id="valdef-media-speech"></a>speech

<a id="ref-for-media-feature⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is expected that all of the media types will also be deprecated in time, as appropriate [media features](#media-feature) are defined which capture their important differences.

Tests

- [mq-deprecated-001.html](https://wpt.fyi/results/css/mediaqueries/mq-deprecated-001.html) [(live test)](http://wpt.live/css/mediaqueries/mq-deprecated-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-deprecated-001.html)

### <a id="mq-features"></a>2.4.  Media Features

<a id="ref-for-media-type②⓪"></a>

A <a id="media-feature"></a>media feature is a more fine-grained test than [media types](#media-type), testing a single, specific feature of the user agent or display device.

<a id="ref-for-media-feature⑦"></a>

Syntactically, [media features](#media-feature) resemble CSS properties: they consist of a feature name, a colon, and a value to test for. They may also be written in boolean form as just a feature name, or in range form with a comparison operator.

![Source diagram 3](assets/mediaqueries-4--CRD-mediaqueries-4-20260219--41310724809c--diagram-03.svg)

Diagram text: ( feature name : feature value feature name range form (see below) )

There are, however, several important differences between properties and media features:

- Properties are used to give information about how to present a document. Media features are used to describe requirements of the output device.

- Media features are always wrapped in parentheses and combined with the and or or keywords, like (color) and (min-width: 600px), rather than being separated with semicolons.

- <a id="ref-for-media-feature⑧"></a>

  <a id="ref-for-descdef-media-color"></a>

  <a id="ref-for-boolean-context"></a>

  A media feature may be given with <em>only</em> its name (omitting the colon and value) to evaluate the feature in a [boolean context](#boolean-context). This is a convenient shorthand for features that have a reasonable value representing 0 or “none”. For example, (color) is true if the [color](#descdef-media-color) [media feature](#media-feature) is non-zero.

- <a id="ref-for-range-context"></a>

  <a id="ref-for-media-feature⑨"></a>

  [Media features](#media-feature) with “range” type can be written in a [range context](#range-context), which uses standard mathematical comparison operators rather than a colon, or have their feature names [prefixed with “min-” or “max-”](#mq-min-max).

- <a id="ref-for-media-feature①⓪"></a>

  Properties sometimes accept complex values, e.g., calculations that involve several other values. [Media features](#media-feature) only accept single values: one keyword, one number, etc.

<a id="ref-for-media-feature①①"></a>

If a [media feature](#media-feature) references a concept which does not exist on the device where the UA is running (for example, speech UAs do not have a concept of “width”), the <a id="ref-for-media-feature①②"></a>media feature must always evaluate to false.

<a id="ref-for-valdef-media-speech"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b561f02b"></a> The media feature device-aspect-ratio only applies to visual devices. On an [speech](#valdef-media-speech) device, expressions involving device-aspect-ratio will therefore always be false:
>
> ```text
> <link media="speech and (device-aspect-ratio: 16/9)"
>       rel="stylesheet" href="example.css">
> ```
#### <a id="mq-ranges"></a>2.4.1.  Media Feature Types: “range” and “discrete”

Every media feature defines its “type” as either “range” or “discrete” in its definition table.

<a id="ref-for-descdef-media-pointer"></a>

“Discrete” media features, like [pointer](#descdef-media-pointer) take their values from a set. The values may be keywords or boolean numbers (0 and 1), but the common factor is that there’s no intrinsic “order” to them—​none of the values are “less than” or “greater than” each other.

<a id="ref-for-descdef-media-width"></a>

“Range” media features like [width](#descdef-media-width), on the other hand, take their values from a range. Any two values can be compared to see which is lesser and which is greater.

<a id="ref-for-media-feature①③"></a>

<a id="ref-for-range-context①"></a>

The only significant difference between the two types is that “range” [media features](#media-feature) can be evaluated in a [range context](#range-context) and accept “min-” and “max-” prefixes on their name. Doing either of these changes the meaning of the feature—​rather than the <a id="ref-for-media-feature①④"></a>media feature being true when the feature exactly matches the given value, it matches when the feature is greater than/less than/equal to the given value.

<a id="ref-for-media-feature①⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-92fc9072"></a> A (width \>= 600px) [media feature](#media-feature) is true when the viewport’s width is 600px <em>or more</em>.
>
> On the other hand, (width: 600px) by itself is only true when the viewport’s width is <em>exactly</em> 600px. If it’s less or greater than 600px, it’ll be false.

#### <a id="mq-boolean-context"></a>2.4.2.  Evaluating Media Features in a Boolean Context

<a id="ref-for-media-feature①⑥"></a>

While [media features](#media-feature) normally have a syntax similar to CSS properties, they can also be written more simply as just the feature name, like (color).

<a id="ref-for-media-feature①⑦"></a>

<a id="ref-for-typedef-dimension"></a>

<a id="ref-for-valdef-media-update-none"></a>

When written like this, the [media feature](#media-feature) is evaluated in a <a id="boolean-context"></a>boolean context. If the feature would be true for any value <em>other than</em> the number 0, a [\<dimension\>](https://www.w3.org/TR/css-values-3/#typedef-dimension) with the value 0, or the keyword [none](#valdef-media-update-none), the <a id="ref-for-media-feature①⑧"></a>media feature evaluates to true. Otherwise, it evaluates to false.

<a id="ref-for-media-feature①⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f4e46a51"></a> Some [media features](#media-feature) are designed to be written like this.
>
> <a id="ref-for-descdef-media-update"></a>
>
> For example, [update](#descdef-media-update) is typically written as (update) to test if any kind of updating is available, or not (update) to check for the opposite.
>
> It can still be given an explicit value as well, with (update: fast) or (update: slow) equal to (update), and (update: none) equal to not (update).

<a id="ref-for-media-feature②⓪"></a>

<a id="ref-for-descdef-media-width①"></a>

<a id="ref-for-boolean-context①"></a>

<a id="ref-for-descdef-media-color①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b4366ae2"></a> Some numeric [media features](#media-feature), like [width](#descdef-media-width), are rarely if ever useful to evaluate in a [boolean context](#boolean-context), as their values are almost always greater than zero. Others, like [color](#descdef-media-color), have meaningful zero values: (color) is identical to (color \> 0), indicating that the device is capable of displaying color at all.

<a id="ref-for-media-feature②①"></a>

<a id="ref-for-boolean-context②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ec07be25"></a> Only some of the [media features](#media-feature) that accept keywords are meaningful in a [boolean context](#boolean-context).
>
> <a id="ref-for-descdef-media-pointer①"></a>
>
> <a id="ref-for-valdef-media-pointer-none"></a>
>
> For example, (pointer) is useful, as [pointer](#descdef-media-pointer) has a [none](#valdef-media-pointer-none) value to indicate there’s no pointing device at all on the device.
>
> <a id="ref-for-descdef-media-color-gamut"></a>
>
> <a id="ref-for-boolean-context③"></a>
>
> Similarly, not (color-gamut) can be useful to detect a very low-quality screen, as such a device won’t match any of the [color-gamut](#descdef-media-color-gamut) keywords; even tho <a id="ref-for-descdef-media-color-gamut①"></a>color-gamut lacks a none keyword, it’ll still be false in a [boolean context](#boolean-context) because none of its values match.
>
> On the other hand, (scan) is just always true or always false (depending on whether it applies at all to the device), as, if it applies at all, the device is guaranteed to match at least one of its values.

#### <a id="mq-range-context"></a>2.4.3.  Evaluating Media Features in a Range Context

<a id="ref-for-media-feature②②"></a>

[Media features](#media-feature) with a “range” type can be alternately written in a <a id="range-context"></a>range context that takes advantage of the fact that their values are ordered, using ordinary mathematical comparison operators:

![Source diagram 4](assets/mediaqueries-4--CRD-mediaqueries-4-20260219--41310724809c--diagram-04.svg)

Diagram text: ( feature name/value \> \<= \< = \>= feature value/name value \< \<= feature name \< \<= value value \> \>= feature name \> \>= value )

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This syntax is new to Level 4 of Mediaqueries, and thus is not as widely supported at the moment as the min-/max- prefixes.

The basic form, consisting of a feature name, a comparison operator, and a value, returns true if the relationship is true.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fc928617"></a> For example, (height \> 600px) (or (600px \< height)) returns true if the viewport height is greater than 600px.

The remaining forms, with the feature name nested between two value comparisons, returns true if both comparisons are true.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ef30a3ee"></a> For example, (400px \< width \< 1000px) returns true if the viewport width is between 400px and 1000px (but not equal to either).

Some media features with a "range" type are said to be <a id="false-in-the-negative-range"></a>false in the negative range. This means that negative values are valid and must be parsed, and that querying whether the media feature is equal to, less than, or less or equal than any such negative value must evaluate to false. Querying whether the media feature is greater, or greater or equal, than a negative value evaluates to true if the relationship is true.

<a id="ref-for-descdef-media-resolution"></a>

<a id="ref-for-descdef-media-width②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If negative values had been rejected at parse time instead, they would be treated as unknown based on the error handling rules. However, in reality, whether a device’s [resolution](#descdef-media-resolution) is -300dpi is not unknown, it is known to be false. Similarly, for any visual device, the [width](#descdef-media-width) of the targeted display area is known to be greater than -200px The above rule reflects that, making intuition match what UAs do.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d3333f79"></a> The following examples result in a green background on all visual devices:
>
> ```text
> @media not (width <= -100px) {
>   body { background: green; }
> }
> ```
>
> ```text
> @media (height > -100px) {
>   body { background: green; }
> }
> ```
>
> ```text
> @media not (resolution: -300dpi) {
>   body { background: green; }
> }
> ```
<a id="ref-for-media-query②④"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> This is a behavior change compared to Media Queries Level 3 [\[MEDIAQUERIES-3\]](#biblio-mediaqueries-3), where negative values on these properties caused a syntax error. In level 3, syntax errors—including forbidden values—resulted in the entire [media query](#media-query) being false, rather than the unknown treatment defined in this level. Implementations updating from level 3 should make sure to change the handling of negative values for the relevant properties when they add support for the richer syntax defined in [§ 2.5 Combining Media Features](#media-conditions), to avoid introducing unintended semantics.

Tests

- [mq-negative-range-001.html](https://wpt.fyi/results/css/mediaqueries/mq-negative-range-001.html) [(live test)](http://wpt.live/css/mediaqueries/mq-negative-range-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-negative-range-001.html)
- [mq-negative-range-002.html](https://wpt.fyi/results/css/mediaqueries/mq-negative-range-002.html) [(live test)](http://wpt.live/css/mediaqueries/mq-negative-range-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-negative-range-002.html)

#### <a id="mq-min-max"></a>2.4.4.  Using “min-” and “max-” Prefixes On Range Features

<a id="ref-for-media-feature②③"></a>

Rather than evaluating a “range” type [media feature](#media-feature) in a range context, as described above, the feature may be written as a normal <a id="ref-for-media-feature②④"></a>media feature, but with a “min-” or “max-” prefix on the feature name.

<a id="ref-for-range-context②"></a>

This is equivalent to evaluating the feature in a [range context](#range-context), as follows:

- Using a “min-” prefix on a feature name is equivalent to using the “\>=” operator. For example, (min-height: 600px) is equivalent to (height \>= 600px).
- Using a “max-” prefix on a feature name is equivalent to using the “\<=” operator. For example, (max-width: 40em) is equivalent to (width \<= 40em).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: because “min-” and “max-” both equate to range comparisons that <strong>include</strong> the value, they may be limiting in certain situations.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-69e2181d"></a> For instance, authors trying to define different styles based on a breakpoint in the viewport width using “min-” and “max-” would generally offset the values they’re comparing, to ensure that both queries don’t evaluate to true simultaneously. Assuming the breakpoint is at 320px, authors would conceptually use:
>
> ```text
> @media (max-width: 320px) { /* styles for viewports <= 320px */ }
> @media (min-width: 321px) { /* styles for viewports >= 321px */ }
> ```
>
> While this ensures that the two sets of styles don’t apply simultaneously when the viewport width is 320px, it does not take into account the possibility of fractional viewport sizes which can occur as a result of non-integer pixel densities (e.g. on high-dpi displays or as a result of zooming/scaling). Any viewport widths that fall between 320px and 321px will result in none of the styles being applied.
>
> One approach to work around this problem is to increase the precision of the values used for the comparison. Using the example above, changing the second comparison value to 320.01px significantly reduces the chance that a viewport width on a device would fall between the cracks.
>
> ```text
> @media (max-width: 320px) { /* styles for viewports <= 320px */ }
> @media (min-width: 320.01px) { /* styles for viewports >= 320.01px */ }
> ```
>
> <a id="ref-for-range-context③"></a>
>
> However, in these situations, [range context](#range-context) queries (which are not limited to “\>=” and “\<=” comparisons) offer a more appropriate solution:
>
> ```text
> @media (width <= 320px) { /* styles for viewports <= 320px */ }
> @media (width > 320px) { /* styles for viewports > 320px */ }
> ```
<a id="ref-for-media-feature②⑤"></a>

“Discrete” type properties do not accept “min-” or “max-” prefixes. Adding such a prefix to a “discrete” type [media feature](#media-feature) simply results in an unknown feature name.

<a id="ref-for-descdef-media-grid①"></a>

<a id="ref-for-media-feature②⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d539f038"></a> For example, (min-grid: 1) is invalid, because [grid](#descdef-media-grid) is a “discrete” [media feature](#media-feature), and so doesn’t accept the prefixes. (Even though the <a id="ref-for-descdef-media-grid②"></a>grid <a id="ref-for-media-feature②⑦"></a>media feature appears to be numeric, as it accepts the values 0 and 1.)

<a id="ref-for-media-feature②⑧"></a>

<a id="ref-for-boolean-context④"></a>

Attempting to evaluate a min/max prefixed [media feature](#media-feature) in a [boolean context](#boolean-context) is invalid and a syntax error.

### <a id="media-conditions"></a>2.5. Combining Media Features

<a id="ref-for-media-feature②⑨"></a>

Multiple [media features](#media-feature) can be combined together into a <a id="media-condition"></a>media condition using full boolean algebra (not, and, or).

- <a id="ref-for-valdef-media-not③"></a>

  Any media feature can be negated by placing [not](#valdef-media-not) before it. For example, not (color) inverts the meaning of (color)—​since (color) matches a device with any kind of color display, not (color) matches a device <em>without</em> any kind of color display.

- Two or more media features can be chained together, such that the query is only true if <em>all</em> of the media features are true, by placing and between them. For example, (width \< 600px) and (height \< 600px) only matches devices whose screens are smaller than 600px wide in both dimensions.

- Alternately, two or more media features can be chained together, such that the query is true if <em>any</em> of the media features are true, by placing or between them. For example, (update: slow) or (hover: none) matches if the device is slow to update the screen (such as an e-reader) <em>or</em> the primary pointing device has no hover capability, perhaps indicating that one should use a layout that displays more information rather than compactly hiding it until the user hovers.

- <a id="ref-for-media-condition①"></a>

  [Media conditions](#media-condition) can be grouped by wrapping them in parentheses () which can then be nested within a condition the same as a single media query. For example, (not (color)) or (hover) is true on devices that are monochrome and/or that have hover capabilities. If one instead wanted to query for a device that was monochrome and <em>didn’t</em> have hover capabilities, it must instead be written as not ((color) or (hover)) (or, equivalently, as (not (color)) and (not (hover))).

<a id="ref-for-valdef-media-not④"></a>

It is <em>invalid</em> to mix and and or and [not](#valdef-media-not) at the same “level” of a media query. For example, (color) and (pointer) or (hover) is illegal, as it’s unclear what was meant. Instead, parentheses can be used to group things using a particular joining keyword, yielding either (color) and ((pointer) or (hover)) or ((color) and (pointer)) or (hover). These two have very different meanings: if only (hover) is true, the first one evaluates to false but the second evaluates to true.

Tests

- [negation-001.html](https://wpt.fyi/results/css/mediaqueries/negation-001.html) [(live test)](http://wpt.live/css/mediaqueries/negation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/negation-001.html)
- [negation-002.html](https://wpt.fyi/results/css/mediaqueries/negation-002.html) [(live test)](http://wpt.live/css/mediaqueries/negation-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/negation-002.html)

## <a id="mq-syntax"></a>3.  Syntax

Informal descriptions of the media query syntax appear in the prose and railroad diagrams in previous sections. The formal media query syntax is described in this section, with the rule/property grammar syntax defined in [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3) and [\[CSS-VALUES-3\]](#biblio-css-values-3).

<a id="ref-for-parse-a-comma-separated-list-of-component-values"></a>

<a id="ref-for-typedef-media-query"></a>

To parse a <a id="typedef-media-query-list"></a>\<media-query-list\> production, [parse a comma-separated list of component values](https://www.w3.org/TR/css-syntax-3/#parse-a-comma-separated-list-of-component-values), then parse each entry in the returned list as a [\<media-query\>](#typedef-media-query). Its value is the list of <a id="ref-for-typedef-media-query①"></a>\<media-query\>s so produced.

<a id="ref-for-typedef-media-query-list"></a>

<a id="ref-for-media-query-list③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This explicit definition of [\<media-query-list\>](#typedef-media-query-list) parsing is necessary to make the error-recovery behavior of [media query lists](#media-query-list) well-defined.

<a id="ref-for-typedef-media-query-list①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This definition of [\<media-query-list\>](#typedef-media-query-list) parsing intentionally accepts an empty list.

<a id="ref-for-ascii-case-insensitive"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As per [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3), tokens are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive).

<a id="typedef-media-query"></a>

<a id="ref-for-typedef-media-condition"></a>

<a id="ref-for-typedef-media-type"></a>

<a id="ref-for-typedef-media-condition-without-or"></a>

<a id="typedef-media-type"></a>

<a id="ref-for-typedef-ident"></a>

<a id="typedef-media-condition"></a>

<a id="ref-for-typedef-media-not"></a>

<a id="ref-for-typedef-media-in-parens"></a>

<a id="ref-for-typedef-media-and"></a>

<a id="ref-for-typedef-media-or"></a>

<a id="typedef-media-condition-without-or"></a>

<a id="ref-for-typedef-media-not①"></a>

<a id="ref-for-typedef-media-in-parens①"></a>

<a id="ref-for-typedef-media-and①"></a>

<a id="typedef-media-not"></a>

<a id="ref-for-typedef-media-in-parens②"></a>

<a id="typedef-media-and"></a>

<a id="ref-for-typedef-media-in-parens③"></a>

<a id="typedef-media-or"></a>

<a id="ref-for-typedef-media-in-parens④"></a>

<a id="typedef-media-in-parens"></a>

<a id="ref-for-typedef-media-condition①"></a>

<a id="ref-for-typedef-media-feature"></a>

<a id="ref-for-typedef-general-enclosed"></a>

<a id="typedef-media-feature"></a>

<a id="ref-for-typedef-mf-plain"></a>

<a id="ref-for-typedef-mf-boolean"></a>

<a id="ref-for-typedef-mf-range"></a>

<a id="typedef-mf-plain"></a>

<a id="ref-for-typedef-mf-name"></a>

<a id="ref-for-typedef-mf-value"></a>

<a id="typedef-mf-boolean"></a>

<a id="ref-for-typedef-mf-name①"></a>

<a id="typedef-mf-range"></a>

<a id="ref-for-typedef-mf-name②"></a>

<a id="ref-for-typedef-mf-comparison"></a>

<a id="ref-for-typedef-mf-value①"></a>

<a id="ref-for-typedef-mf-value②"></a>

<a id="ref-for-typedef-mf-comparison①"></a>

<a id="ref-for-typedef-mf-name③"></a>

<a id="ref-for-typedef-mf-value③"></a>

<a id="ref-for-typedef-mf-lt"></a>

<a id="ref-for-typedef-mf-name④"></a>

<a id="ref-for-typedef-mf-lt①"></a>

<a id="ref-for-typedef-mf-value④"></a>

<a id="ref-for-typedef-mf-value⑤"></a>

<a id="ref-for-typedef-mf-gt"></a>

<a id="ref-for-typedef-mf-name⑤"></a>

<a id="ref-for-typedef-mf-gt①"></a>

<a id="ref-for-typedef-mf-value⑥"></a>

<a id="typedef-mf-name"></a>

<a id="ref-for-typedef-ident①"></a>

<a id="typedef-mf-value"></a>

<a id="ref-for-number-value①"></a>

<a id="ref-for-typedef-dimension①"></a>

<a id="ref-for-typedef-ident②"></a>

<a id="ref-for-ratio-value"></a>

<a id="typedef-mf-lt"></a>

<a id="typedef-mf-gt"></a>

<a id="typedef-mf-eq"></a>

<a id="typedef-mf-comparison"></a>

<a id="ref-for-typedef-mf-lt②"></a>

<a id="ref-for-typedef-mf-gt②"></a>

<a id="ref-for-typedef-mf-eq"></a>

<a id="typedef-general-enclosed"></a>

<a id="ref-for-typedef-function-token"></a>

<a id="ref-for-typedef-any-value"></a>

<a id="ref-for-typedef-any-value①"></a>

```text
<media-query> = <media-condition>
             | [ not | only ]? <media-type> [ and <media-condition-without-or> ]?
<media-type> = <ident>

<media-condition> = <media-not> | <media-in-parens> [ <media-and>* | <media-or>* ]
<media-condition-without-or> = <media-not> | <media-in-parens> <media-and>*
<media-not> = not <media-in-parens>
<media-and> = and <media-in-parens>
<media-or> = or <media-in-parens>
<media-in-parens> = ( <media-condition> ) | ( <media-feature> ) | <general-enclosed>

<media-feature> = [ <mf-plain> | <mf-boolean> | <mf-range> ]
<mf-plain> = <mf-name> : <mf-value>
<mf-boolean> = <mf-name>
<mf-range> = <mf-name> <mf-comparison> <mf-value>
           | <mf-value> <mf-comparison> <mf-name>
           | <mf-value> <mf-lt> <mf-name> <mf-lt> <mf-value>
           | <mf-value> <mf-gt> <mf-name> <mf-gt> <mf-value>
<mf-name> = <ident>
<mf-value> = <number> | <dimension> | <ident> | <ratio>
<mf-lt> = '<' '='?
<mf-gt> = '>' '='?
<mf-eq> = '='
<mf-comparison> = <mf-lt> | <mf-gt> | <mf-eq>

<general-enclosed> = [ <function-token> <any-value>? ) ] | [ ( <any-value>? ) ]
```
<a id="ref-for-typedef-media-type①"></a>

<a id="ref-for-valdef-media-only④"></a>

<a id="ref-for-valdef-media-not⑤"></a>

The [\<media-type\>](#typedef-media-type) production does not include the keywords [only](#valdef-media-only), [not](#valdef-media-not), and, or, and layer.

<a id="ref-for-cascade-layers"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The exclusion of layer is because it would otherwise be ambiguous when when used in the `@import url(…) layer;` syntax for the sake of [cascade layers](https://www.w3.org/TR/css-cascade-5/#cascade-layers). See [\[CSS-CASCADE-5\]](#biblio-css-cascade-5).

<a id="ref-for-typedef-delim-token"></a>

No whitespace is allowed between the “\<” or “\>” [\<delim-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-delim-token)s and the following “=” <a id="ref-for-typedef-delim-token①"></a>\<delim-token\>, if it’s present.

<a id="ref-for-valdef-media-not⑥"></a>

<a id="ref-for-typedef-function-token①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Whitespace is required between a [not](#valdef-media-not), and, or or keyword and the following ( character, because without it that would instead parse as a [\<function-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-function-token). This is not made explicitly invalid because it’s already covered by the above grammar. It’s fine to have whitespace between a ) and a following keyword, however.

<a id="ref-for-typedef-media-in-parens⑤"></a>

<a id="ref-for-typedef-general-enclosed①"></a>

<a id="ref-for-typedef-general-enclosed②"></a>

When parsing the [\<media-in-parens\>](#typedef-media-in-parens) production, the [\<general-enclosed\>](#typedef-general-enclosed) branch must only be chosen if the input does not match either of the preceding branches. <strong data-conversion-semantic="note">Note:</strong> [\<general-enclosed\>](#typedef-general-enclosed) exists to allow for future expansion of the grammar in a reasonably compatible way.

Tests

- [match-media-parsing.html](https://wpt.fyi/results/css/mediaqueries/match-media-parsing.html) [(live test)](http://wpt.live/css/mediaqueries/match-media-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/match-media-parsing.html)
- [mq-case-insensitive-001.html](https://wpt.fyi/results/css/mediaqueries/mq-case-insensitive-001.html) [(live test)](http://wpt.live/css/mediaqueries/mq-case-insensitive-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-case-insensitive-001.html)
- [mq-range-001.html](https://wpt.fyi/results/css/mediaqueries/mq-range-001.html) [(live test)](http://wpt.live/css/mediaqueries/mq-range-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-range-001.html)
- [mq-unknown-feature-custom-property.html](https://wpt.fyi/results/css/mediaqueries/mq-unknown-feature-custom-property.html) [(live test)](http://wpt.live/css/mediaqueries/mq-unknown-feature-custom-property.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-unknown-feature-custom-property.html)

### <a id="evaluating"></a>3.1. Evaluating Media Queries

<a id="ref-for-typedef-media-condition②"></a>

<a id="ref-for-typedef-media-condition-without-or①"></a>

Each of the major subexpression of [\<media-condition\>](#typedef-media-condition) or [\<media-condition-without-or\>](#typedef-media-condition-without-or) is associated with a boolean result, as follows:

<a id="ref-for-typedef-media-condition③"></a>

[\<media-condition\>](#typedef-media-condition)

<a id="ref-for-typedef-media-condition-without-or②"></a>

[\<media-condition-without-or\>](#typedef-media-condition-without-or)

The result is the result of the child subexpression.

<a id="ref-for-typedef-media-in-parens⑥"></a>

[\<media-in-parens\>](#typedef-media-in-parens)

The result is the result of the child term.

<a id="ref-for-typedef-media-not②"></a>

[\<media-not\>](#typedef-media-not)

<a id="ref-for-typedef-media-in-parens⑦"></a>

The result is the negation of the [\<media-in-parens\>](#typedef-media-in-parens) term. The negation of unknown is unknown.

<a id="ref-for-typedef-media-and②"></a>

<a id="ref-for-typedef-media-in-parens⑧"></a>

[\<media-in-parens\>](#typedef-media-in-parens) [\<media-and\>](#typedef-media-and)\*

<a id="ref-for-typedef-media-and③"></a>

<a id="ref-for-typedef-media-in-parens⑨"></a>

The result is true if the [\<media-in-parens\>](#typedef-media-in-parens) child term and all of the <a id="ref-for-typedef-media-in-parens①⓪"></a>\<media-in-parens\> children of the [\<media-and\>](#typedef-media-and) child terms are true, false if at least one of these <a id="ref-for-typedef-media-in-parens①①"></a>\<media-in-parens\> terms are false, and unknown otherwise.

<a id="ref-for-typedef-media-or①"></a>

<a id="ref-for-typedef-media-in-parens①②"></a>

[\<media-in-parens\>](#typedef-media-in-parens) [\<media-or\>](#typedef-media-or)\*

<a id="ref-for-typedef-media-or②"></a>

<a id="ref-for-typedef-media-in-parens①③"></a>

The result is false if the [\<media-in-parens\>](#typedef-media-in-parens) child term and all of the <a id="ref-for-typedef-media-in-parens①④"></a>\<media-in-parens\> children of the [\<media-or\>](#typedef-media-or) child terms are false, true if at least one of these <a id="ref-for-typedef-media-in-parens①⑤"></a>\<media-in-parens\> terms are true, and unknown otherwise.

<a id="ref-for-typedef-general-enclosed③"></a>

[\<general-enclosed\>](#typedef-general-enclosed)

The result is unknown.

<a id="ref-for-typedef-general-enclosed④"></a>

<a id="ref-for-typedef-media-condition④"></a>

Authors must not use [\<general-enclosed\>](#typedef-general-enclosed) in their stylesheets. <strong data-conversion-semantic="note">Note:</strong> It exists only for future-compatibility, so that new syntax additions do not invalidate too much of a [\<media-condition\>](#typedef-media-condition) in older user agents.

<a id="ref-for-typedef-media-feature①"></a>

[\<media-feature\>](#typedef-media-feature)

The result is the result of evaluating the specified media feature.

If the result of any of the above productions is used in any context that expects a two-valued boolean, “unknown” must be converted to “false”.

<a id="ref-for-media-query②⑤"></a>

<a id="ref-for-at-ruledef-media①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that, for example, when a [media query](#media-query) is used in a [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) rule, if it resolves to “unknown” it’s treated as “false” and fails to match.

> <strong data-conversion-semantic="note">Note</strong>
>
> Media Queries use a three-value logic where terms can be “true”, “false”, or “unknown”. Specifically, it uses the [Kleene 3-valued logic](https://en.wikipedia.org/wiki/Three-valued_logic#Kleene_and_Priest_logics). In this logic, “unknown” means “either true or false, but we’re not sure which yet”.
>
> In general, an unknown value showing up in a formula will cause the formula to be unknown as well, as substituting “true” for the unknown will give the formula a different result than substituting “false”. The only way to eliminate an unknown value is to use it in a formula that will give the same result whether the unknown is replaced with a true or false value. This occurs when you have “false AND unknown” (evaluates to false regardless) and “true OR unknown” (evaluates to true regardless).
>
> <a id="ref-for-typedef-general-enclosed⑤"></a>
>
> <a id="ref-for-media-query②⑥"></a>
>
> This logic was adopted because [\<general-enclosed\>](#typedef-general-enclosed) needs to be assigned a truth value. In standard boolean logic, the only reasonable value is “false”, but this means that not unknown(function) is true, which can be confusing and unwanted. Kleene’s 3-valued logic ensures that unknown things will prevent a [media query](#media-query) from matching, unless their value is irrelevant to the final result.

### <a id="error-handling"></a>3.2.  Error Handling

A media query that does not match the grammar in the previous section must be replaced by not all during parsing.

<a id="ref-for-media-query-list④"></a>

<a id="ref-for-media-query②⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that a grammar mismatch does <strong>not</strong> wipe out an entire [media query list](#media-query-list), just the problematic [media query](#media-query). The parsing behavior defined above automatically recovers at the next top-level comma.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bc6a6750"></a>
>
> ```text
> @media (example, all,), speech { /* only applicable to speech devices */ }
> @media &test, speech           { /* only applicable to speech devices */ }
> ```
>
> <a id="ref-for-media-query-list⑤"></a>
>
> <a id="ref-for-valdef-media-speech①"></a>
>
> Both of the above [media query lists](#media-query-list) are turned into not all, speech during parsing, which has the same truth value as just [speech](#valdef-media-speech).
>
> <a id="ref-for-media-query②⑧"></a>
>
> Note that error-recovery only happens at the top-level of a [media query](#media-query); anything inside of an invalid parenthesized block will just get turned into not all as a group. For example:
>
> ```text
> @media (example, speech { /* rules for speech devices */ }
> ```
>
> <a id="ref-for-media-query②⑨"></a>
>
> Because the parenthesized block is unclosed, it will contain the entire rest of the stylesheet from that point (unless it happens to encounter an unmatched “)” character somewhere in the stylesheet), and turn the entire thing into a not all [media query](#media-query).

<a id="ref-for-typedef-media-type②"></a>

An unknown [\<media-type\>](#typedef-media-type) must be treated as not matching.

<a id="ref-for-media-type②①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2b46833d"></a> For example, the media query unknown is false, as unknown is an unknown [media type](#media-type).
>
> <a id="ref-for-valdef-media-not⑦"></a>
>
> But not unknown is true, as the [not](#valdef-media-not) negates the false media type.

<a id="ref-for-typedef-media-type③"></a>

<a id="ref-for-media-type②②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-45dbfecd"></a> Remember that some keywords aren’t allowed as [\<media-type\>](#typedef-media-type)s and cause parsing to fail entirely: the media query or and (color) is turned into not all during parsing, rather than just treating the or as an unknown [media type](#media-type).

<a id="ref-for-typedef-mf-name⑥"></a>

<a id="ref-for-typedef-mf-value⑦"></a>

<a id="ref-for-media-feature③⓪"></a>

<a id="ref-for-typedef-media-query②"></a>

An unknown [\<mf-name\>](#typedef-mf-name) or [\<mf-value\>](#typedef-mf-value), or a feature value which does not match the value syntax for that [media feature](#media-feature), results in the value “unknown”. A [\<media-query\>](#typedef-media-query) whose value is “unknown” must be replaced with not all.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d129d027"></a>
>
> ```text
> <link media="screen and (max-weight: 3kg) and (color), (color)"rel="stylesheet" href="example.css" />
> ```
>
> <a id="ref-for-media-feature③①"></a>
>
> <a id="ref-for-media-query-list⑥"></a>
>
> As max-weight is an unknown [media feature](#media-feature), this [media query list](#media-query-list) is turned into not all, (color), which is equivalent to just (color).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a1cc014d"></a>
>
> ```text
> @media (min-orientation: portrait) { … }
> ```
>
> <a id="ref-for-descdef-media-orientation"></a>
>
> <a id="ref-for-media-feature③②"></a>
>
> The [orientation](#descdef-media-orientation) feature does not accept prefixes, so this is considered an unknown [media feature](#media-feature), and turned into not all.

<a id="ref-for-descdef-media-color②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-951a2717"></a> The media query (color:20example) specifies an unknown value for the [color](#descdef-media-color) media feature and is therefore turned into not all.

<a id="ref-for-media-query③⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that [media queries](#media-query) are also subject to the parsing rules of the host language. For example, take the following CSS snippet:
>
> ```text
> @media test;,all { body { background: lime } }
> ```
>
> <a id="ref-for-at-ruledef-media②"></a>
>
> <a id="ref-for-media-query③①"></a>
>
> The media query test;,all is, parsed by itself, equivalent to not all, all, which is always true. However, CSS’s parsing rules cause the [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) rule, and thus the [media query](#media-query), to end at the semicolon. The remainder of the text is treated as a style rule with an invalid selector and contents.

Tests

- [duplicate-media-stylesheet-crash.html](https://wpt.fyi/results/css/mediaqueries/duplicate-media-stylesheet-crash.html) [(live test)](http://wpt.live/css/mediaqueries/duplicate-media-stylesheet-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/duplicate-media-stylesheet-crash.html)
- [mq-invalid-media-type-001.html](https://wpt.fyi/results/css/mediaqueries/mq-invalid-media-type-001.html) [(live test)](http://wpt.live/css/mediaqueries/mq-invalid-media-type-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-invalid-media-type-001.html)
- [mq-invalid-media-type-002.html](https://wpt.fyi/results/css/mediaqueries/mq-invalid-media-type-002.html) [(live test)](http://wpt.live/css/mediaqueries/mq-invalid-media-type-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-invalid-media-type-002.html)
- [mq-invalid-media-type-003.html](https://wpt.fyi/results/css/mediaqueries/mq-invalid-media-type-003.html) [(live test)](http://wpt.live/css/mediaqueries/mq-invalid-media-type-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-invalid-media-type-003.html)
- [mq-invalid-media-type-004.html](https://wpt.fyi/results/css/mediaqueries/mq-invalid-media-type-004.html) [(live test)](http://wpt.live/css/mediaqueries/mq-invalid-media-type-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-invalid-media-type-004.html)
- [mq-invalid-media-type-005.html](https://wpt.fyi/results/css/mediaqueries/mq-invalid-media-type-005.html) [(live test)](http://wpt.live/css/mediaqueries/mq-invalid-media-type-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-invalid-media-type-005.html)
- [mq-invalid-media-type-006.html](https://wpt.fyi/results/css/mediaqueries/mq-invalid-media-type-006.html) [(live test)](http://wpt.live/css/mediaqueries/mq-invalid-media-type-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-invalid-media-type-006.html)
- [mq-invalid-media-type-layer-001.html](https://wpt.fyi/results/css/mediaqueries/mq-invalid-media-type-layer-001.html) [(live test)](http://wpt.live/css/mediaqueries/mq-invalid-media-type-layer-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-invalid-media-type-layer-001.html)
- [mq-invalid-media-type-layer-002.html](https://wpt.fyi/results/css/mediaqueries/mq-invalid-media-type-layer-002.html) [(live test)](http://wpt.live/css/mediaqueries/mq-invalid-media-type-layer-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-invalid-media-type-layer-002.html)

## <a id="mf-dimensions"></a>4.  Viewport/Page Dimensions Media Features

<a id="ref-for-descdef-media-width③"></a>

### <a id="width"></a>4.1.  Width: the [width](#descdef-media-width) feature

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-width"></a>width                                                               |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media③"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-length-value"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)      |
| <strong>Type:&#xA;      </strong> | range                                                                                  |

<a id="ref-for-descdef-media-width④"></a>

<a id="ref-for-continuous-media"></a>

<a id="ref-for-paged-media"></a>

The [width](#descdef-media-width) media feature describes the width of the targeted display area of the output device. For [continuous media](#continuous-media), this is the width of the viewport (as described by CSS2, section 9.1.1 [\[CSS2\]](#biblio-css2)) including the size of a rendered scroll bar (if any). For [paged media](#paged-media), this is the width of the page box (as described by CSS2, section 13.2 \[CSS2\]).

<a id="ref-for-length-value①"></a>

[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)s are interpreted according to [§ 1.3 Units](#units).

<a id="ref-for-descdef-media-width⑤"></a>

<a id="ref-for-false-in-the-negative-range"></a>

[width](#descdef-media-width) is [false in the negative range](#false-in-the-negative-range).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3ef57809"></a> For example, this media query expresses that the style sheet is used on printed output wider than 25cm:
>
> ```text
> <link rel="stylesheet" media="print and (min-width: 25cm)" href="http://…" />
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-462c5899"></a> This media query expresses that the style sheet is used on devices with viewport (the part of the screen/paper where the document is rendered) widths between 400 and 700 pixels:
>
> ```text
> @media (400px <= width <= 700px) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fab3f079"></a> This media query expresses that style sheet is used if the width of the viewport is greater than 20em.
>
> ```text
> @media (min-width: 20em) { … }
> ```
>
> <a id="ref-for-em①"></a>
>
> <a id="ref-for-initial-value②"></a>
>
> <a id="ref-for-propdef-font-size①"></a>
>
> The [em](https://www.w3.org/TR/css-values-3/#em) value is relative to the [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value) of [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size).

Tests

- [media-query-matches-in-iframe.html](https://wpt.fyi/results/css/mediaqueries/media-query-matches-in-iframe.html) [(live test)](http://wpt.live/css/mediaqueries/media-query-matches-in-iframe.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/media-query-matches-in-iframe.html)
- [min-width-001.xht](https://wpt.fyi/results/css/mediaqueries/min-width-001.xht) [(live test)](http://wpt.live/css/mediaqueries/min-width-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/min-width-001.xht)
- [min-width-tables-001.html](https://wpt.fyi/results/css/mediaqueries/min-width-tables-001.html) [(live test)](http://wpt.live/css/mediaqueries/min-width-tables-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/min-width-tables-001.html)
- [viewport-script-dynamic.html](https://wpt.fyi/results/css/mediaqueries/viewport-script-dynamic.html) [(live test)](http://wpt.live/css/mediaqueries/viewport-script-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/viewport-script-dynamic.html)

<a id="ref-for-descdef-media-height"></a>

### <a id="height"></a>4.2.  Height: the [height](#descdef-media-height) feature

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-height"></a>height                                                              |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media④"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-length-value②"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)      |
| <strong>Type:&#xA;      </strong> | range                                                                                  |

<a id="ref-for-descdef-media-height①"></a>

<a id="ref-for-continuous-media①"></a>

<a id="ref-for-paged-media①"></a>

The [height](#descdef-media-height) media feature describes the height of the targeted display area of the output device. For [continuous media](#continuous-media), this is the height of the viewport including the size of a rendered scroll bar (if any). For [paged media](#paged-media), this is the height of the page box.

<a id="ref-for-length-value③"></a>

[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)s are interpreted according to [§ 1.3 Units](#units).

<a id="ref-for-descdef-media-height②"></a>

<a id="ref-for-false-in-the-negative-range①"></a>

[height](#descdef-media-height) is [false in the negative range](#false-in-the-negative-range).

<a id="ref-for-descdef-media-aspect-ratio"></a>

### <a id="aspect-ratio"></a>4.3.  Aspect-Ratio: the [aspect-ratio](#descdef-media-aspect-ratio) feature

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-aspect-ratio"></a>aspect-ratio                                                        |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media⑤"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-ratio-value①"></a>[\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value)        |
| <strong>Type:&#xA;      </strong> | range                                                                                  |

<a id="ref-for-descdef-media-aspect-ratio①"></a>

<a id="ref-for-descdef-media-width⑥"></a>

<a id="ref-for-descdef-media-height③"></a>

The [aspect-ratio](#descdef-media-aspect-ratio) media feature is defined as the ratio of the value of the [width](#descdef-media-width) media feature to the value of the [height](#descdef-media-height) media feature.

Tests

- [aspect-ratio-006.html](https://wpt.fyi/results/css/mediaqueries/aspect-ratio-006.html) [(live test)](http://wpt.live/css/mediaqueries/aspect-ratio-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/aspect-ratio-006.html)
- [aspect-ratio-005.html](https://wpt.fyi/results/css/mediaqueries/aspect-ratio-005.html) [(live test)](http://wpt.live/css/mediaqueries/aspect-ratio-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/aspect-ratio-005.html)
- [aspect-ratio-004.html](https://wpt.fyi/results/css/mediaqueries/aspect-ratio-004.html) [(live test)](http://wpt.live/css/mediaqueries/aspect-ratio-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/aspect-ratio-004.html)
- [aspect-ratio-003.html](https://wpt.fyi/results/css/mediaqueries/aspect-ratio-003.html) [(live test)](http://wpt.live/css/mediaqueries/aspect-ratio-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/aspect-ratio-003.html)
- [aspect-ratio-002.html](https://wpt.fyi/results/css/mediaqueries/aspect-ratio-002.html) [(live test)](http://wpt.live/css/mediaqueries/aspect-ratio-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/aspect-ratio-002.html)
- [aspect-ratio-001.html](https://wpt.fyi/results/css/mediaqueries/aspect-ratio-001.html) [(live test)](http://wpt.live/css/mediaqueries/aspect-ratio-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/aspect-ratio-001.html)
- [aspect-ratio-serialization.html](https://wpt.fyi/results/css/mediaqueries/aspect-ratio-serialization.html) [(live test)](http://wpt.live/css/mediaqueries/aspect-ratio-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/aspect-ratio-serialization.html)

<a id="ref-for-descdef-media-orientation①"></a>

### <a id="orientation"></a>4.4.  Orientation: the [orientation](#descdef-media-orientation) feature

| Field               | Definition                                                                               |
|---------------------|------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-orientation"></a>orientation                                                           |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media⑥"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)   |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one"></a>portrait [\|](https://www.w3.org/TR/css-values-3/#comb-one) landscape |
| <strong>Type:&#xA;      </strong> | discrete                                                                                 |

<a id="valdef-media-orientation-portrait"></a>portrait  
<a id="ref-for-descdef-media-width⑦"></a>

<a id="ref-for-descdef-media-height④"></a>

<a id="ref-for-valdef-media-orientation-portrait"></a>

<a id="ref-for-descdef-media-orientation②"></a>

The [orientation](#descdef-media-orientation) media feature is [portrait](#valdef-media-orientation-portrait) when the value of the [height](#descdef-media-height) media feature is greater than or equal to the value of the [width](#descdef-media-width) media feature.

<a id="valdef-media-orientation-landscape"></a>landscape  
<a id="ref-for-valdef-media-orientation-landscape"></a>

<a id="ref-for-descdef-media-orientation③"></a>

Otherwise [orientation](#descdef-media-orientation) is [landscape](#valdef-media-orientation-landscape).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a79558f0"></a> The following media query tests for “portrait” orientation, like a phone held upright.
>
> ```text
> @media (orientation: portrait) { … }
> ```
## <a id="mf-display-quality"></a>5.  Display Quality Media Features

<a id="ref-for-descdef-media-resolution①"></a>

### <a id="resolution"></a>5.1.  Display Resolution: the [resolution](#descdef-media-resolution) feature

| Field               | Definition                                                                                                                                                               |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-resolution"></a>resolution                                                                                                                                            |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media⑦"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)                                                                                   |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one①"></a><a id="ref-for-resolution-value①"></a>[\<resolution\>](https://www.w3.org/TR/css-values-4/#resolution-value) [\|](https://www.w3.org/TR/css-values-3/#comb-one) infinite |
| <strong>Type:&#xA;      </strong> | range                                                                                                                                                                    |

<a id="ref-for-descdef-media-resolution②"></a>

<a id="ref-for-page-zoom"></a>

<a id="ref-for-scale-factor"></a>

The [resolution](#descdef-media-resolution) media feature describes the resolution of the output device, i.e. the density of the pixels, taking into account the [page zoom](https://www.w3.org/TR/cssom-view-1/#page-zoom) but assuming a [scale factor](https://www.w3.org/TR/cssom-view-1/#scale-factor) of 1.0.

<a id="ref-for-descdef-media-resolution③"></a>

<a id="ref-for-false-in-the-negative-range②"></a>

The [resolution](#descdef-media-resolution) media feature is [false in the negative range](#false-in-the-negative-range)

<a id="ref-for-descdef-media-resolution④"></a>

When querying media with non-square pixels, [resolution](#descdef-media-resolution) queries the density in the vertical dimension.

For printers, this corresponds to the screening resolution (the resolution for printing dots of arbitrary color). Printers might have a different resolution for grayscale printing.

<a id="ref-for-range-context④"></a>

<a id="ref-for-valdef-media-resolution-infinite"></a>

<a id="ref-for-resolution-value②"></a>

For output mediums that have no physical constraints on resolution (such as outputting to vector graphics), this feature must match the <a id="valdef-media-resolution-infinite"></a>infinite value. For the purpose of evaluating this media feature in the [range context](#range-context), [infinite](#valdef-media-resolution-infinite) must be treated as larger than any possible [\<resolution\>](https://www.w3.org/TR/css-values-4/#resolution-value). (That is, a query like (resolution \> 1000dpi) will be true for an <a id="ref-for-valdef-media-resolution-infinite①"></a>infinite media.)

<a id="ref-for-px"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4141fdad"></a> This media query simply detects “high-resolution” screens (those with a hardware pixel to CSS [px](https://www.w3.org/TR/css-values-3/#px) ratio of at least 2):
>
> ```text
> @media (resolution >= 2dppx)
> ```
<a id="ref-for-in"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-618117ac"></a> For example, this media query expresses that a style sheet is used on devices with resolution greater than 300 dots per CSS [in](https://www.w3.org/TR/css-values-3/#in):
>
> ```text
> @media print and (min-resolution: 300dpi) { … }
> ```
>
> <a id="ref-for-cm"></a>
>
> This media query is equivalent, but uses the CSS [cm](https://www.w3.org/TR/css-values-3/#cm) unit:
>
> ```text
> @media print and (min-resolution: 118dpcm) { … }
> ```
<a id="ref-for-resolution-value③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> [\<resolution\>](https://www.w3.org/TR/css-values-4/#resolution-value) does not refer to the number of device pixels per physical length unit, but the number of device pixels per css unit. This mapping is done by the user agent, so it is always known to the user agent.
>
> If the user agent either has no knowledge of the geometry of physical pixels, or knows about the geometry physical pixels and they are (close enough to) square, it would not map a different number of device pixels per css pixels along each axis, and the would therefore be no difference between the vertical and horizontal resolution.
>
> Otherwise, if the UA chooses to map a different number along each axis, this would be to respond to physical pixels not being square either. How the UA comes to this knowledge is out of scope, but having enough information to take this decision, it can invert the mapping should the device be rotated 90 degrees.

<a id="ref-for-descdef-media-scan①"></a>

### <a id="scan"></a>5.2.  Display Type: the [scan](#descdef-media-scan) feature

| Field               | Definition                                                                                  |
|---------------------|---------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-scan"></a>scan                                                                     |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media⑧"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)      |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one②"></a>interlace [\|](https://www.w3.org/TR/css-values-3/#comb-one) progressive |
| <strong>Type:&#xA;      </strong> | discrete                                                                                    |

<a id="ref-for-descdef-media-scan②"></a>

The [scan](#descdef-media-scan) media feature describes the scanning process of some output devices.

<a id="valdef-media-scan-interlace"></a>interlace  
CRT and some types of plasma TV screens used “interlaced” rendering, where video frames alternated between specifying only the “even” lines on the screen and only the “odd” lines, exploiting various automatic mental image-correction abilities to produce smooth motion. This allowed them to simulate a higher FPS broadcast at half the bandwidth cost.

When displaying on interlaced screens, authors should avoid very fast movement across the screen to avoid “combing”, and should ensure that details on the screen are wider than 1px to avoid [“twitter”](https://en.wikipedia.org/wiki/Interlaced_video#Interline_twitter).

<a id="valdef-media-scan-progressive"></a>progressive  
A screen using “progressive” rendering displays each screen fully, and needs no special treatment.

Most modern screens, and all computer screens, use progressive rendering.

<a id="ref-for-descdef-media-scan③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3db7de6d"></a> For example, the “feet” of letters in serif fonts are very small features that can provoke “twitter” on interlaced devices. The [scan](#descdef-media-scan) media feature can be used to detect this, and use an alternative font with less chance of “twitter”:
>
> ```text
> @media (scan: interlace) { body { font-family: sans-serif; } }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: At the time of writing, all known implementations match `scan: progressive` rather than `scan: interlace`.

<a id="ref-for-descdef-media-grid③"></a>

### <a id="grid"></a>5.3.  Detecting Console Displays: the [grid](#descdef-media-grid) feature

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-grid"></a>grid                                                                |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media⑨"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-typedef-mq-boolean"></a>[\<mq-boolean\>](#typedef-mq-boolean)                               |
| <strong>Type:&#xA;      </strong> | discrete                                                                               |

<a id="typedef-mq-boolean"></a>

<a id="ref-for-integer-value①"></a>

```text
<mq-boolean> = <integer [0,1]>
```
<a id="ref-for-descdef-media-grid④"></a>

The [grid](#descdef-media-grid) media feature is used to query whether the output device is grid or bitmap. If the output device is grid-based (e.g., a “tty” terminal, or a phone display with only one fixed font), the value will be 1. Otherwise, the value will be 0.

<a id="ref-for-typedef-mq-boolean①"></a>

<a id="ref-for-integer-value②"></a>

<a id="ref-for-typedef-mq-boolean②"></a>

The [\<mq-boolean\>](#typedef-mq-boolean) value type is an [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) with the value 0 or 1. Any other integer value is invalid. <strong data-conversion-semantic="note">Note:</strong> Note that -0 is always equivalent to 0 in CSS, and so is also accepted as a valid [\<mq-boolean\>](#typedef-mq-boolean) value.

<a id="ref-for-typedef-mq-boolean③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<mq-boolean\>](#typedef-mq-boolean) type exists only for legacy purposes. If this feature were being designed today, it would instead use proper named keywords for its values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-07e10c7e"></a> Here is an example that detects a narrow console screen:
>
> ```text
> @media (grid) and (max-width: 15em) { … }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: At the time of writing, all known implementations match `grid: 0` rather than `grid: 1`.

<a id="ref-for-descdef-media-update①"></a>

### <a id="update"></a>5.4.  Display Update Frequency: the [update](#descdef-media-update) feature

| Field               | Definition                                                                                                 |
|---------------------|------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-update"></a>update                                                                                  |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media①⓪"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)                     |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one③"></a>none [\|](https://www.w3.org/TR/css-values-3/#comb-one) slow <a id="ref-for-comb-one④"></a>\| fast |
| <strong>Type:&#xA;      </strong> | discrete                                                                                                   |

<a id="ref-for-descdef-media-update②"></a>

The [update](#descdef-media-update) media feature is used to query the ability of the output device to modify the appearance of content once it has been rendered. It accepts the following values:

<a id="valdef-media-update-none"></a>none  
Once it has been rendered, the layout can no longer be updated. Example: documents printed on paper.

<a id="valdef-media-update-slow"></a>slow  
The layout may change dynamically according to the usual rules of CSS, but the output device is not able to render or display changes quickly enough for them to be perceived as a smooth animation. Example: E-ink screens or severely under-powered devices.

<a id="valdef-media-update-fast"></a>fast  
The layout may change dynamically according to the usual rules of CSS, and the output device is not unusually constrained in speed, so regularly-updating things like CSS Animations can be used. Example: computer screens.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-57cc8bd6"></a> For example, if a page styles its links to only add underlines on hover, it may want to always display underlines when printed:
>
> ```text
> @media (update) {
>   a { text-decoration: none; }
>   a:hover, a:focus { text-decoration: underline; }
> }
> /* In non-updating UAs, the links get their default underline at all times. */
> ```
Tests

- [update-media-feature.html](https://wpt.fyi/results/css/mediaqueries/update-media-feature.html) [(live test)](http://wpt.live/css/mediaqueries/update-media-feature.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/update-media-feature.html)

<a id="ref-for-descdef-media-overflow-block"></a>

### <a id="mf-overflow-block"></a>5.5.  Block-Axis Overflow: the [overflow-block](#descdef-media-overflow-block) feature

| Field               | Definition                                                                                                    |
|---------------------|---------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-overflow-block"></a>overflow-block                                                                             |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media①①"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)                        |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one⑤"></a>none [\|](https://www.w3.org/TR/css-values-3/#comb-one) scroll <a id="ref-for-comb-one⑥"></a>\| paged |
| <strong>Type:&#xA;      </strong> | discrete                                                                                                      |

<a id="ref-for-descdef-media-overflow-block①"></a>

<a id="ref-for-block-axis"></a>

The [overflow-block](#descdef-media-overflow-block) media feature describes the behavior of the device when content overflows the initial containing block in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis).

<a id="valdef-media-overflow-block-none"></a>none  
<a id="ref-for-block-axis①"></a>

There is no affordance for overflow in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis); any overflowing content is simply not displayed. Examples: billboards

<a id="valdef-media-overflow-block-scroll"></a>scroll  
<a id="ref-for-block-axis②"></a>

Overflowing content in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) is exposed by allowing users to scroll to it. Examples: computer screens

<a id="valdef-media-overflow-block-paged"></a>paged  
<a id="ref-for-block-axis③"></a>

Content is broken up into discrete pages; content that overflows one page in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) is displayed on the following page. Examples: printers, ebook readers

<a id="ref-for-valdef-media-overflow-block-none"></a>

<a id="ref-for-valdef-media-overflow-block-scroll"></a>

<a id="ref-for-valdef-media-overflow-block-paged"></a>

Media that match [none](#valdef-media-overflow-block-none) or [scroll](#valdef-media-overflow-block-scroll) are said to be <a id="continuous-media"></a>continuous media, while those that match [paged](#valdef-media-overflow-block-paged) are said to be <a id="paged-media"></a>paged media

<a id="ref-for-continuous-media②"></a>

<a id="ref-for-paged-media②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Additional values for this media feature may be added in the future to describe classes of user agents with a hybrid behavior combining aspects of [continuous](#continuous-media) and [paged media](#paged-media). For example, the Presto layout engine (now discontinued) shipped with a semi-paginated presentation-mode behavior similar to continuous except that it honored forced page breaks. Not knowing of any currently-shipping user agent with this type of behavior, the Working Group has decided not to add such a value in this level to avoid mischaracterizing any such user agent. Anyone implementing a user agent not adequately described by any of the values specified above is encouraged to contact the Working Group so that extensions to this media feature may be considered.

Tests

- [overflow-media-features.html](https://wpt.fyi/results/css/mediaqueries/overflow-media-features.html) [(live test)](http://wpt.live/css/mediaqueries/overflow-media-features.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/overflow-media-features.html)

<a id="ref-for-descdef-media-overflow-inline"></a>

### <a id="mf-overflow-inline"></a>5.6.  Inline-Axis Overflow: the [overflow-inline](#descdef-media-overflow-inline) feature

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-overflow-inline"></a>overflow-inline                                                     |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media①②"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one⑦"></a>none [\|](https://www.w3.org/TR/css-values-3/#comb-one) scroll      |
| <strong>Type:&#xA;      </strong> | discrete                                                                               |

<a id="ref-for-descdef-media-overflow-inline①"></a>

<a id="ref-for-inline-axis"></a>

The [overflow-inline](#descdef-media-overflow-inline) media feature describes the behavior of the device when content overflows the initial containing block in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

<a id="valdef-media-overflow-inline-none"></a>none  
<a id="ref-for-inline-axis①"></a>

There is no affordance for overflow in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis); any overflowing content is simply not displayed.

<a id="valdef-media-overflow-inline-scroll"></a>scroll  
<a id="ref-for-inline-axis②"></a>

Overflowing content in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) is exposed by allowing users to scroll to it.

<a id="ref-for-valdef-media-overflow-block-paged①"></a>

<a id="ref-for-descdef-media-overflow-inline②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There are no known implementations of paged overflow of inline-overflowing content, and the very concept doesn’t seem to make much sense, so there is intentionally no [paged](#valdef-media-overflow-block-paged) value for [overflow-inline](#descdef-media-overflow-inline).

## <a id="mf-colors"></a>6.  Color Media Features

<a id="ref-for-descdef-media-color③"></a>

### <a id="color"></a>6.1.  Color Depth: the [color](#descdef-media-color) feature

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-color"></a>color                                                               |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media①③"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-integer-value③"></a>[\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value)    |
| <strong>Type:&#xA;      </strong> | range                                                                                  |

<a id="ref-for-descdef-media-color④"></a>

The [color](#descdef-media-color) media feature describes the number of bits per color component of the output device. If the device is not a color device, the value is zero.

<a id="ref-for-descdef-media-color⑤"></a>

<a id="ref-for-false-in-the-negative-range③"></a>

[color](#descdef-media-color) is [false in the negative range](#false-in-the-negative-range).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8d42e196"></a> For example, these two media queries express that a style sheet applies to all color devices:
>
> ```text
> @media (color) { … }
> @media (min-color: 1) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f33339f9"></a> This media query expresses that a style sheet applies to color devices with at least 8 bits per color component:
>
> ```text
> @media (color >= 8) { … }
> ```
If different color components are represented by different number of bits, the smallest number is used.

<a id="ref-for-descdef-media-color⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d1c7ceef"></a> For instance, if an 8-bit color system represents the red component with 3 bits, the green component with 3 bits, and the blue component with 2 bits, the [color](#descdef-media-color) media feature will have a value of 2.

In a device with indexed colors, the minimum number of bits per color component in the lookup table is used.

<a id="ref-for-descdef-media-color-gamut②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The described functionality is only able to describe color capabilities at a superficial level. [color-gamut](#descdef-media-color-gamut), is generally more relevant to authors’ needs. If further functionality is required, RFC2879 [\[RFC2879\]](#biblio-rfc2879) provides more specific media features which may be supported at a later stage.

<a id="ref-for-descdef-media-color-index"></a>

### <a id="color-index"></a>6.2.  Paletted Color Screens: the [color-index](#descdef-media-color-index) feature

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-color-index"></a>color-index                                                         |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media①④"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-integer-value④"></a>[\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value)    |
| <strong>Type:&#xA;      </strong> | range                                                                                  |

<a id="ref-for-descdef-media-color-index①"></a>

The [color-index](#descdef-media-color-index) media feature describes the number of entries in the color lookup table of the output device. If the device does not use a color lookup table, the value is zero.

<a id="ref-for-descdef-media-color-index②"></a>

<a id="ref-for-false-in-the-negative-range④"></a>

[color-index](#descdef-media-color-index) is [false in the negative range](#false-in-the-negative-range).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d0764fe1"></a> For example, here are two ways to express that a style sheet applies to all color index devices:
>
> ```text
> @media (color-index) { … }
> @media (color-index >= 1) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0fec94ea"></a> This media query expresses that a style sheet applies to a color index device with 256 or more entries:
>
> ```text
> <?xml-stylesheet media="(min-color-index: 256)"
>   href="http://www.example.com/…" ?>
> ```
<a id="ref-for-descdef-media-monochrome"></a>

### <a id="monochrome"></a>6.3.  Monochrome Screens: the [monochrome](#descdef-media-monochrome) feature

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-monochrome"></a>monochrome                                                          |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media①⑤"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-integer-value⑤"></a>[\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value)    |
| <strong>Type:&#xA;      </strong> | range                                                                                  |

<a id="ref-for-descdef-media-monochrome①"></a>

The [monochrome](#descdef-media-monochrome) media feature describes the number of bits per pixel in a monochrome frame buffer. If the device is not a monochrome device, the output device value will be 0.

<a id="ref-for-descdef-media-monochrome②"></a>

<a id="ref-for-false-in-the-negative-range⑤"></a>

[monochrome](#descdef-media-monochrome) is [false in the negative range](#false-in-the-negative-range).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fa837be6"></a> For example, this is how to express that a style sheet applies to all monochrome devices:
>
> ```text
> @media (monochrome) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-83e2c204"></a> Express that a style sheet applies to monochrome devices with more than 2 bits per pixels:
>
> ```text
> @media (monochrome >= 2) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-60bef5a3"></a> Express that there is one style sheet for color pages and another for monochrome:
>
> ```text
> <link rel="stylesheet" media="print and (color)" href="http://…" />
> <link rel="stylesheet" media="print and (monochrome)" href="http://…" />
> ```
<a id="ref-for-descdef-media-color-gamut③"></a>

### <a id="color-gamut"></a>6.4.  Color Display Quality: the [color-gamut](#descdef-media-color-gamut) feature

| Field               | Definition                                                                                                  |
|---------------------|-------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-color-gamut"></a>color-gamut                                                                              |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media①⑥"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)                      |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one⑧"></a>srgb [\|](https://www.w3.org/TR/css-values-3/#comb-one) p3 <a id="ref-for-comb-one⑨"></a>\| rec2020 |
| <strong>Type:&#xA;      </strong> | discrete                                                                                                    |

<a id="ref-for-descdef-media-color-gamut④"></a>

The [color-gamut](#descdef-media-color-gamut) media feature describes the approximate range of colors that are supported by the UA and output device. That is, if the UA receives content with colors in the specified space it can cause the output device to render the appropriate color, or something appropriately close enough.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The query uses approximate ranges for a few reasons. Firstly, there are a lot of differences in display hardware. For example, a device might claim to support "Rec. 2020", but actually renders a significantly lower range of the full gamut. Secondly, there are a lot of different color ranges that different devices support, and enumerating them all would be tedious. In most cases the author does not need to know the exact capabilities of the display, just whether it is better than sRGB, or significantly better than sRGB. That way they can serve appropriate images, tagged with color profiles, to the user.

<a id="valdef-media-color-gamut-srgb"></a>srgb  
The UA and output device can support approximately the sRGB gamut or more.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is expected that the vast majority of color displays will be able to return true to a query of this type.

<a id="valdef-media-color-gamut-p3"></a>p3  
The UA and output device can support approximately the gamut specified by the Display P3 [\[Display-P3\]](#biblio-display-p3) Color Space or more.

<a id="ref-for-valdef-media-color-gamut-p3"></a>

<a id="ref-for-valdef-media-color-gamut-srgb"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [p3](#valdef-media-color-gamut-p3) gamut is larger than and includes the [srgb](#valdef-media-color-gamut-srgb) gamut.

<a id="valdef-media-color-gamut-rec2020"></a>rec2020  
The UA and output device can support approximately the gamut specified by the ITU-R Recommendation BT.2020 Color Space or more.

<a id="ref-for-valdef-media-color-gamut-rec2020"></a>

<a id="ref-for-valdef-media-color-gamut-p3①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [rec2020](#valdef-media-color-gamut-rec2020) gamut is larger than and includes the [p3](#valdef-media-color-gamut-p3) gamut.

The following table lists the primary colors of these color spaces in terms of their color space chromaticity coordinates, as defined in [\[COLORIMETRY\]](#biblio-colorimetry).

**Table 15**

Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.

| Color Space | White Point / x<sub>W</sub> | White Point / y<sub>W</sub> |
| --- | --- | --- |
| srgb | 0.3127 | 0.3290 |
| p3 | 0.3127 | 0.3290 |
| rec2020 | 0.3127 | 0.3290 |

| Color Space | Primaries / Red / x<sub>R</sub> | Primaries / Red / y<sub>R</sub> |
| --- | --- | --- |
| srgb | 0.640 | 0.330 |
| p3 | 0.680 | 0.320 |
| rec2020 | 0.708 | 0.292 |

| Color Space | Primaries / Green / x<sub>G</sub> | Primaries / Green / y<sub>G</sub> |
| --- | --- | --- |
| srgb | 0.300 | 0.600 |
| p3 | 0.265 | 0.690 |
| rec2020 | 0.170 | 0.797 |

| Color Space | Primaries / Blue / x<sub>B</sub> | Primaries / Blue / y<sub>B</sub> |
| --- | --- | --- |
| srgb | 0.150 | 0.060 |
| p3 | 0.150 | 0.060 |
| rec2020 | 0.131 | 0.046 |

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The table above does not contains enough information to fully describe the color spaces, but is sufficient to determine whether an output device approximately covers their respective gamuts. See [\[SRGB\]](#biblio-srgb) for more information on sRGB, [\[Display-P3\]](#biblio-display-p3) for more information on Display P3, and [\[ITU-R-BT-2020-2\]](#biblio-itu-r-bt-2020-2) for more information on ITU-R Recommendation BT.2020.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-602686e6"></a> For example, this media query applies when the display supports colors in the range of Display P3:
>
> ```css
> @media (color-gamut: p3) { … }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: An output device can return true for multiple values of this media feature, if its full output gamut is large enough, or one gamut is a subset of another supported gamut. As a result, this feature is best used in an "ascending" fashion—​set a base value when (color-gamut: srgb) is true, then override it if (color-gamut: p3) is true, etc.

<a id="ref-for-valdef-media-color-gamut-srgb①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some output devices, such as monochrome displays, cannot support even the [srgb](#valdef-media-color-gamut-srgb) gamut. To test for these devices, you can use this feature in a negated boolean-context fashion: not (color-gamut).

Tests

- [mq-gamut-001.html](https://wpt.fyi/results/css/mediaqueries/mq-gamut-001.html) [(live test)](http://wpt.live/css/mediaqueries/mq-gamut-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-gamut-001.html)
- [mq-gamut-002.html](https://wpt.fyi/results/css/mediaqueries/mq-gamut-002.html) [(live test)](http://wpt.live/css/mediaqueries/mq-gamut-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-gamut-002.html)
- [mq-gamut-003.html](https://wpt.fyi/results/css/mediaqueries/mq-gamut-003.html) [(live test)](http://wpt.live/css/mediaqueries/mq-gamut-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-gamut-003.html)
- [mq-gamut-004.html](https://wpt.fyi/results/css/mediaqueries/mq-gamut-004.html) [(live test)](http://wpt.live/css/mediaqueries/mq-gamut-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-gamut-004.html)
- [mq-gamut-005.html](https://wpt.fyi/results/css/mediaqueries/mq-gamut-005.html) [(live test)](http://wpt.live/css/mediaqueries/mq-gamut-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/mq-gamut-005.html)

## <a id="mf-interaction"></a>7.  Interaction Media Features

The “interaction” media features reflect various aspects of how the user interacts with the page.

<a id="ref-for-descdef-media-pointer②"></a>

<a id="ref-for-descdef-media-hover"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Typical examples of devices matching combinations of [pointer](#descdef-media-pointer) and [hover](#descdef-media-hover):
>
> |                     | <a id="ref-for-descdef-media-pointer③"></a>[pointer: none](#descdef-media-pointer)          | <a id="ref-for-descdef-media-pointer④"></a>[pointer: coarse](#descdef-media-pointer) | <a id="ref-for-descdef-media-pointer⑤"></a>[pointer: fine](#descdef-media-pointer)                                  |
> |---------------------|---------------------------------------------------------------------|--------------------------------------------------------------|---------------------------------------------------------------------------------------------|
> | <strong><span><a id="ref-for-descdef-media-hover①"></a></span><a href="#descdef-media-hover">hover: none</a> &#xA;       </strong> | keyboard-only controls, sequential/spatial (d-pad) focus navigation | smartphones, touch screens                                   | basic stylus digitizers (Cintiq, Wacom, etc)                                                |
> | <strong><span><a id="ref-for-descdef-media-hover②"></a></span><a href="#descdef-media-hover">hover: hover</a> &#xA;       </strong> |                                                                     | Nintendo Wii controller, Kinect                              | mouse, touch pad, advanced stylus digitizers (Surface, Samsung Note, Wacom Intuos Pro, etc) |

<a id="ref-for-descdef-media-pointer⑥"></a>

<a id="ref-for-descdef-media-hover③"></a>

<a id="ref-for-descdef-media-any-pointer"></a>

<a id="ref-for-descdef-media-any-hover"></a>

The [pointer](#descdef-media-pointer) and [hover](#descdef-media-hover) features relate to the characteristics of the “primary” pointing device, while [any-pointer](#descdef-media-any-pointer) and [any-hover](#descdef-media-any-hover) can be used to query the properties of all potentially available pointing devices.

<a id="ref-for-descdef-media-pointer⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While this specification does not define how user agents should decide what the “primary” pointing device is, the expectation is that user agents should make this determination by combining knowledge about the device/environment they are running on, the number and type of pointing devices available, and a notion of which of these is generally and/or currently being used. In situations where the primary input mechanism for a device is not a pointing device, but there is a secondary – and less frequently used – input that is a pointing devices, the user agent may decide to treat the non-pointing device as the primary (resulting in [pointer: none](#descdef-media-pointer)). user agents may also decide to dynamically change what type of pointing device is deemed to be primary, in response to changes in the user environment or in the way the user is interacting with the UA.

<a id="ref-for-descdef-media-pointer⑧"></a>

<a id="ref-for-descdef-media-hover④"></a>

<a id="ref-for-descdef-media-any-pointer①"></a>

<a id="ref-for-descdef-media-any-hover①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [pointer](#descdef-media-pointer), [hover](#descdef-media-hover), [any-pointer](#descdef-media-any-pointer) and [any-hover](#descdef-media-any-hover) features only relate to the characteristics, or the complete absence, of pointing devices, and can not be used to detect the presence of non-pointing device input mechanisms such as keyboards. Authors should take into account the potential presence of non-pointing device inputs, regardless of which values are matched when querying these features.

<a id="ref-for-descdef-media-pointer⑨"></a>

<a id="ref-for-descdef-media-hover⑤"></a>

<a id="ref-for-descdef-media-any-pointer②"></a>

<a id="ref-for-descdef-media-any-hover②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> While [pointer](#descdef-media-pointer) and [hover](#descdef-media-hover) can be used to design the main style and interaction mode of the page to suit the primary input mechanism (based on the characteristics, or complete absence, of the primary pointing device), authors should strongly consider using [any-pointer](#descdef-media-any-pointer) and [any-hover](#descdef-media-any-hover) to take into account all possible types of pointing devices that have been detected.

<a id="ref-for-descdef-media-pointer①⓪"></a>

### <a id="pointer"></a>7.1.  Pointing Device Quality: the [pointer](#descdef-media-pointer) feature

| Field               | Definition                                                                                                   |
|---------------------|--------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-pointer"></a>pointer                                                                                   |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media①⑦"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)                       |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one①⓪"></a>none [\|](https://www.w3.org/TR/css-values-3/#comb-one) coarse <a id="ref-for-comb-one①①"></a>\| fine |
| <strong>Type:&#xA;      </strong> | discrete                                                                                                     |

<a id="ref-for-descdef-media-pointer①①"></a>

<a id="ref-for-descdef-media-any-pointer③"></a>

The [pointer](#descdef-media-pointer) media feature is used to query the presence and accuracy of a pointing device such as a mouse. If multiple pointing devices are present, the <a id="ref-for-descdef-media-pointer①②"></a>pointer media feature must reflect the characteristics of the “primary” pointing device, as determined by the user agent. (To query the capabilities of <em>any</em> available pointing devices, see the [any-pointer](#descdef-media-any-pointer) media feature.)

<a id="valdef-media-pointer-none"></a>none  
The primary input mechanism of the device does not include a pointing device.

<a id="valdef-media-pointer-coarse"></a>coarse  
The primary input mechanism of the device includes a pointing device of limited accuracy. Examples include touchscreens and motion-detection sensors (like the Kinect peripheral for the Xbox.)

<a id="valdef-media-pointer-fine"></a>fine  
The primary input mechanism of the device includes an accurate pointing device. Examples include mice, touchpads, and drawing styluses.

<a id="ref-for-valdef-media-pointer-coarse"></a>

<a id="ref-for-valdef-media-pointer-fine"></a>

Both [coarse](#valdef-media-pointer-coarse) and [fine](#valdef-media-pointer-fine) indicate the presence of a pointing device, but differ in accuracy. A pointing device with which it would be difficult or impossible to reliably pick one of several small adjacent targets at a zoom factor of 1 would qualify as <a id="ref-for-valdef-media-pointer-coarse①"></a>coarse. Changing the zoom level does not affect the value of this media feature.

<a id="ref-for-valdef-media-pointer-coarse②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As the UA may provide the user with the ability to zoom, or as secondary pointing devices may have a different accuracy, the user may be able to perform accurate clicks even if the value of this media feature is [coarse](#valdef-media-pointer-coarse). This media feature does not indicate that the user will never be able to click accurately, only that it is inconvenient for them to do so. Authors are expected to react to a value of <a id="ref-for-valdef-media-pointer-coarse③"></a>coarse by designing pages that do not rely on accurate clicking to be operated.

<a id="ref-for-valdef-media-pointer-fine①"></a>

<a id="ref-for-valdef-media-pointer-coarse④"></a>

<a id="ref-for-valdef-media-pointer-none①"></a>

<a id="ref-for-descdef-media-any-pointer④"></a>

For accessibility reasons, even on devices whose pointing device can be described as [fine](#valdef-media-pointer-fine), the UA may give a value of [coarse](#valdef-media-pointer-coarse) or [none](#valdef-media-pointer-none) to this media query, to indicate that the user has difficulties manipulating the pointing device accurately or at all. In addition, even if the primary pointing device has <a id="ref-for-valdef-media-pointer-fine②"></a>fine pointing accuracy, there may be additional <a id="ref-for-valdef-media-pointer-coarse⑤"></a>coarse pointing devices available to the user. Authors may wish to query the [any-pointer](#descdef-media-any-pointer) media feature to take these other <a id="ref-for-valdef-media-pointer-coarse⑥"></a>coarse potential pointing devices into account.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b703d20d"></a>
>
> ```text
> /* Make radio buttons and check boxes larger if we have an inaccurate primary pointing device */
> @media (pointer: coarse) {
>   input[type="checkbox"], input[type="radio"] {
>     min-width: 30px;
>     min-height: 40px;
>     background: transparent;
>   }
> }
> ```
<a id="ref-for-descdef-media-hover⑥"></a>

### <a id="hover"></a>7.2.  Hover Capability: the [hover](#descdef-media-hover) feature

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-hover"></a>hover                                                               |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media①⑧"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one①②"></a>none [\|](https://www.w3.org/TR/css-values-3/#comb-one) hover       |
| <strong>Type:&#xA;      </strong> | discrete                                                                               |

<a id="ref-for-descdef-media-hover⑦"></a>

<a id="ref-for-descdef-media-any-hover③"></a>

The [hover](#descdef-media-hover) media feature is used to query the user’s ability to hover over elements on the page with the primary pointing device. If a device has multiple pointing devices, the <a id="ref-for-descdef-media-hover⑧"></a>hover media feature must reflect the characteristics of the “primary” pointing device, as determined by the user agent. (To query the capabilities of <em>any</em> available pointing devices, see the [any-hover](#descdef-media-any-hover) media feature.)

<a id="valdef-media-hover-none"></a>none  
Indicates that the primary pointing device can’t hover, or that there is no pointing device. Examples include touchscreens and screens that use a basic drawing stylus.

<a id="ref-for-descdef-media-hover⑨"></a>

Pointing devices that can hover, but for which doing so is inconvenient and not part of the normal way they are used, also match this value. For example, a touchscreen where a long press is treated as hovering would match [hover: none](#descdef-media-hover).

<a id="valdef-media-hover-hover"></a>hover  
Indicates that the primary pointing device can easily hover over parts of the page. Examples include mice and devices that physically point at the screen, like the Nintendo Wii controller.

<a id="ref-for-descdef-media-hover①⓪"></a>

<a id="ref-for-media-feature③③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-847fb239"></a> For example, on a touch screen device that can also be controlled by an optional mouse, the [hover](#descdef-media-hover) [media feature](#media-feature) should match <a id="ref-for-descdef-media-hover①①"></a>hover: none, as the primary pointing device (the touch screen) does not allow the user to hover.
>
> <a id="ref-for-hover-pseudo"></a>
>
> <a id="ref-for-pseudo-class"></a>
>
> <a id="ref-for-descdef-media-hover①②"></a>
>
> However, despite this, the optional mouse does allow users to hover. Authors should therefore be careful not to assume that the [:hover](https://www.w3.org/TR/selectors-4/#hover-pseudo) [pseudo-class](https://www.w3.org/TR/selectors-4/#pseudo-class) will never match on a device where [hover: none](#descdef-media-hover) is true, but they should design layouts that do not depend on hovering to be fully usable.

<a id="ref-for-descdef-media-hover①③"></a>

For accessibility reasons, even on devices that do support hovering, the UA may give a value of [hover: none](#descdef-media-hover) to this media query, to opt into layouts that work well without hovering. Note that even if the primary input mechanism has <a id="ref-for-descdef-media-hover①④"></a>hover: hover capability, there may be additional input mechanisms available to the user that do not provide hover capabilities.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-46b96c8f"></a>
>
> ```text
> /* Only use a hover-activated drop down menu on devices that can conveniently hover. */
> @media (hover) {
>   .menu > li        {display:inline-block;}
>   .menu ul          {display:none; position:absolute;}
>   .menu li:hover ul {display:block; list-style:none; padding:0;}
>   /* ... */
> }
> ```
<a id="ref-for-descdef-media-any-pointer⑤"></a>

<a id="ref-for-descdef-media-any-hover④"></a>

### <a id="@media/any-input"></a>7.3.  All Available Interaction Capabilities: the [any-pointer](#descdef-media-any-pointer) and [any-hover](#descdef-media-any-hover) features

| Field               | Definition                                                                                                   |
|---------------------|--------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-any-pointer"></a>any-pointer                                                                               |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media①⑨"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)                       |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one①③"></a>none [\|](https://www.w3.org/TR/css-values-3/#comb-one) coarse <a id="ref-for-comb-one①④"></a>\| fine |
| <strong>Type:&#xA;      </strong> | discrete                                                                                                     |

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-any-hover"></a>any-hover                                                           |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media②⓪"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one①⑤"></a>none [\|](https://www.w3.org/TR/css-values-3/#comb-one) hover       |
| <strong>Type:&#xA;      </strong> | discrete                                                                               |

<a id="ref-for-descdef-media-any-pointer⑥"></a>

<a id="ref-for-descdef-media-any-hover⑤"></a>

<a id="ref-for-descdef-media-pointer①③"></a>

<a id="ref-for-descdef-media-hover①⑤"></a>

The [any-pointer](#descdef-media-any-pointer) and [any-hover](#descdef-media-any-hover) media features are identical to the [pointer](#descdef-media-pointer) and [hover](#descdef-media-hover) media features, but they correspond to the union of capabilities of all the pointing devices available to the user. In the case of <a id="ref-for-descdef-media-any-pointer⑦"></a>any-pointer, more than one of the values can match, if different pointing devices have different characteristics.

<a id="ref-for-descdef-media-any-pointer⑧"></a>

<a id="ref-for-descdef-media-any-hover⑥"></a>

<a id="ref-for-valdef-media-pointer-none②"></a>

[any-pointer](#descdef-media-any-pointer) and [any-hover](#descdef-media-any-hover) must only match none if <em>all</em> of the pointing devices would match [none](#valdef-media-pointer-none) for the corresponding query, or there are no pointing devices at all.

<a id="ref-for-descdef-media-any-pointer⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> [any-pointer](#descdef-media-any-pointer) is used to query the presence and accuracy of pointing devices. It does not take into account any additional non-pointing device inputs, and can not be used to test for the presence of other input mechanisms, such as d-pads or keyboard-only controls, that don’t move an on-screen pointer. <a id="ref-for-descdef-media-any-pointer①⓪"></a>any-pointer: none will only evaluate to true if there are no pointing devices at all present.

<a id="ref-for-descdef-media-any-pointer①①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-43cafa84"></a> On a traditional desktop environment with a mouse and keyboard, [any-pointer: none](#descdef-media-any-pointer) will be false (due to the presence of the mouse), even though a non-pointer input (the keyboard) is also present.

<a id="ref-for-descdef-media-any-hover⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> [any-hover: none](#descdef-media-any-hover) will only evaluate to true if there are no pointing devices, or if all the pointing devices present lack hover capabilities. As such, it should be understood as a query to test if any hover-capable pointing devices are present, rather than whether or not any of the pointing devices is hover-incapable. The latter scenario can currently not be determined using <a id="ref-for-descdef-media-any-hover⑧"></a>any-hover or any other interaction media feature. Additionally, it does not take into account any non-pointing device inputs, such as d-pads or keyboard-only controls, which by their very nature are also not hover-capable.

<a id="ref-for-descdef-media-any-hover⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b5550b4b"></a> On a touch-enabled laptop with a mouse and a touchscreen, [any-hover: none](#descdef-media-any-hover) will evaluate to false (due to the presence of the hover-capable mouse), even though a non-hover-capable pointing device (the touchscreen) is also present. It is currently not possible to provide different styles for cases where different pointing devices have different hover capabilities.

<a id="ref-for-descdef-media-any-hover①⓪"></a>

<a id="ref-for-descdef-media-any-pointer①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Designing a page that relies on hovering or accurate pointing only because [any-hover](#descdef-media-any-hover) or [any-pointer](#descdef-media-any-pointer) indicate that at least one of the available input mechanisms has these capabilities is likely to result in a poor experience. However, authors may use this information to inform their decision about the style and functionality they wish to provide based on any additional pointing devices that are available to the user.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-45cc4e7d"></a> A number of smart TVs come with a way to control an on-screen cursor, but it is often fairly basic controller which is difficult to operate accurately.
>
> <a id="ref-for-valdef-media-pointer-coarse⑦"></a>
>
> <a id="ref-for-descdef-media-pointer①④"></a>
>
> <a id="ref-for-descdef-media-any-pointer①③"></a>
>
> A browser in such a smart TV would have [coarse](#valdef-media-pointer-coarse) as the value of both [pointer](#descdef-media-pointer) and [any-pointer](#descdef-media-any-pointer), allowing authors to provide a layout with large and easy to reach click targets.
>
> <a id="ref-for-descdef-media-pointer①⑤"></a>
>
> <a id="ref-for-valdef-media-pointer-coarse⑧"></a>
>
> <a id="ref-for-descdef-media-any-pointer①④"></a>
>
> <a id="ref-for-valdef-media-pointer-fine③"></a>
>
> The user may also have paired a Bluetooth mouse with the TV, and occasionally use it for extra convenience, but this mouse is not the main way the TV is operated. [pointer](#descdef-media-pointer) still matches [coarse](#valdef-media-pointer-coarse), while [any-pointer](#descdef-media-any-pointer) now both matches <a id="ref-for-valdef-media-pointer-coarse⑨"></a>coarse and [fine](#valdef-media-pointer-fine).
>
> Switching to small click targets based on the fact that (any-pointer: fine) is now true would not be appropriate. It would not only surprise the user by providing an experience out of line with what they expect on a TV, but may also be quite inconvenient: the mouse, not being the primary way to control the TV, may be out of reach, hidden under one of the cushions on the sofa...
>
> By contrast, consider scrolling on the same TV. Scrollbars are difficult to manipulate without an accurate pointing device. Having prepared an alternative way to indicate that there is more content to be seen based on (pointer: coarse) being true, an author may want to still show the scrollbars in addition if (any-pointer: fine) is true, or to hide them altogether to reduce visual clutter if (any-pointer: fine) is false.

## <a id="mf-deprecated"></a> Appendix A: Deprecated Media Features

<a id="ref-for-media-feature③④"></a>

The following [media features](#media-feature) are <strong>deprecated</strong>. They are kept for backward compatibility, but are not appropriate for newly written style sheets. Authors must not use them. User agents must support them as specified.

<a id="ref-for-descdef-media-width⑧"></a>

<a id="ref-for-descdef-media-height⑤"></a>

<a id="ref-for-descdef-media-aspect-ratio②"></a>

<a id="ref-for-media-feature③⑤"></a>

<a id="ref-for-descdef-media-device-width"></a>

<a id="ref-for-descdef-media-device-height"></a>

<a id="ref-for-descdef-media-device-aspect-ratio"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> To query for the size of the viewport (or the page box on page media), the [width](#descdef-media-width), [height](#descdef-media-height) and [aspect-ratio](#descdef-media-aspect-ratio) [media features](#media-feature) should be used, rather than [device-width](#descdef-media-device-width), [device-height](#descdef-media-device-height) and [device-aspect-ratio](#descdef-media-device-aspect-ratio), which refer to the physical size of the device regardless of how much space is available for the document being laid out. The device-\* <a id="ref-for-media-feature③⑥"></a>media features are also sometimes used as a proxy to detect mobile devices. Instead, authors should use <a id="ref-for-media-feature③⑦"></a>media features that better represent the aspect of the device that they are attempting to style against.

### <a id="device-width"></a> device-width

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-device-width"></a>device-width                                                        |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media②①"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-length-value④"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)      |
| <strong>Type:&#xA;      </strong> | range                                                                                  |

<a id="ref-for-descdef-media-device-width①"></a>

<a id="ref-for-continuous-media③"></a>

<a id="ref-for-web-exposed-screen-area"></a>

<a id="ref-for-paged-media③"></a>

The [device-width](#descdef-media-device-width) media feature describes the width of the rendering surface of the output device. For [continuous media](#continuous-media), this is the width of the [Web-exposed screen area](https://www.w3.org/TR/cssom-view-1/#web-exposed-screen-area). For [paged media](#paged-media), this is the width of the page sheet size.

<a id="ref-for-descdef-media-device-width②"></a>

<a id="ref-for-false-in-the-negative-range⑥"></a>

[device-width](#descdef-media-device-width) is [false in the negative range](#false-in-the-negative-range).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4b4397c6"></a>
>
> ```text
> @media (device-width < 800px) { … }
> ```
>
> <a id="ref-for-px①"></a>
>
> In the example above, the style sheet will apply only to screens less than 800px in length. The [px](https://www.w3.org/TR/css-values-3/#px) unit is of the logical kind, as described in the [Units](#units) section.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If a device can be used in multiple orientations, such as portrait and landscape, the device-\* media features reflect the current orientation.

### <a id="device-height"></a> device-height

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-device-height"></a>device-height                                                       |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media②②"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-length-value⑤"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)      |
| <strong>Type:&#xA;      </strong> | range                                                                                  |

<a id="ref-for-descdef-media-device-height①"></a>

<a id="ref-for-continuous-media④"></a>

<a id="ref-for-web-exposed-screen-area①"></a>

<a id="ref-for-paged-media④"></a>

The [device-height](#descdef-media-device-height) media feature describes the height of the rendering surface of the output device. For [continuous media](#continuous-media), this is the height of the [Web-exposed screen area](https://www.w3.org/TR/cssom-view-1/#web-exposed-screen-area). For [paged media](#paged-media), this is the height of the page sheet size.

<a id="ref-for-descdef-media-device-height②"></a>

<a id="ref-for-false-in-the-negative-range⑦"></a>

[device-height](#descdef-media-device-height) is [false in the negative range](#false-in-the-negative-range).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c05b0263"></a>
>
> ```text
> <link rel="stylesheet" media="(device-height > 600px)" />
> ```
>
> <a id="ref-for-px②"></a>
>
> In the example above, the style sheet will apply only to screens taller than 600 vertical pixels. Note that the definition of the [px](https://www.w3.org/TR/css-values-3/#px) unit is the same as in other parts of CSS.

### <a id="device-aspect-ratio"></a> device-aspect-ratio

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-media-device-aspect-ratio"></a>device-aspect-ratio                                                 |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-media②③"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-ratio-value②"></a>[\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value)        |
| <strong>Type:&#xA;      </strong> | range                                                                                  |

<a id="ref-for-descdef-media-device-aspect-ratio①"></a>

<a id="ref-for-descdef-media-device-width③"></a>

<a id="ref-for-descdef-media-device-height③"></a>

The [device-aspect-ratio](#descdef-media-device-aspect-ratio) media feature is defined as the ratio of the value of the [device-width](#descdef-media-device-width) media feature to the value of the [device-height](#descdef-media-device-height) media feature.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b60c3a77"></a> For example, if a screen device with square pixels has 1280 horizontal pixels and 720 vertical pixels (commonly referred to as “16:9”), the following media queries will all match the device:
>
> ```text
> @media (device-aspect-ratio: 16/9) { … }
> @media (device-aspect-ratio: 32/18) { … }
> @media (device-aspect-ratio: 1280/720) { … }
> @media (device-aspect-ratio: 2560/1440) { … }
> ```
Tests

- [device-aspect-ratio-002.html](https://wpt.fyi/results/css/mediaqueries/device-aspect-ratio-002.html) [(live test)](http://wpt.live/css/mediaqueries/device-aspect-ratio-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/device-aspect-ratio-002.html)
- [device-aspect-ratio-003.html](https://wpt.fyi/results/css/mediaqueries/device-aspect-ratio-003.html) [(live test)](http://wpt.live/css/mediaqueries/device-aspect-ratio-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/device-aspect-ratio-003.html)
- [device-aspect-ratio-004.html](https://wpt.fyi/results/css/mediaqueries/device-aspect-ratio-004.html) [(live test)](http://wpt.live/css/mediaqueries/device-aspect-ratio-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/device-aspect-ratio-004.html)
- [device-aspect-ratio-006.html](https://wpt.fyi/results/css/mediaqueries/device-aspect-ratio-006.html) [(live test)](http://wpt.live/css/mediaqueries/device-aspect-ratio-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/mediaqueries/device-aspect-ratio-006.html)

## <a id="changes"></a> Changes

### <a id="changes-2021-12"></a> Changes since the 25 December 2021 Candidate Recommendation Draft

The following changes were made to this specification since the [25 December 2021 Candidate Recommendation Draft](https://www.w3.org/TR/2020/CR-mediaqueries-4-20200721/):

- Establish a normative reference for [\[Display-P3\]](#biblio-display-p3)

- <a id="ref-for-cascade-layers①"></a>

  Disallow use of layer as a media type, rather than merely treat it as an unknown one, for compatibility with [cascade layers](https://www.w3.org/TR/css-cascade-5/#cascade-layers).

### <a id="changes-2020-7"></a> Changes since the 21 July 2020 Candidate Recommendation

The following changes were made to this specification since the [21 July 2020 Candidate Recommendation](https://www.w3.org/TR/2020/CR-mediaqueries-4-20200721/):

- <a id="ref-for-typedef-general-enclosed⑥"></a>

  Allow empty functions in [\<general-enclosed\>](#typedef-general-enclosed) (see [Issue 6803](https://github.com/w3c/csswg-drafts/issues/6803)).

- Editorial tweak to how the grammar is defined (see [Issue 6806](https://github.com/w3c/csswg-drafts/issues/6806)).

### <a id="changes-2017-9"></a> Changes since the 5 September 2017 Candidate Recommendation

The following changes were made to this specification since the [5 September 2017 Candidate Recommendation](https://www.w3.org/TR/2017/CR-mediaqueries-4-20170905/):

- <a id="ref-for-valdef-media-screen⑤"></a>

  <a id="ref-for-valdef-media-speech②"></a>

  Deprecate the [speech](#valdef-media-speech) media type. As media types are exclusive, it cannot be about screen readers, which as their name indicates, work based on a screen rendition, and therefore match the [screen](#valdef-media-screen) media type. It could have been about pure-audio UAs, but no such implementation is known.

- Add note referencing the syntax spec to remind that token parsing is ascii case insensitive

- Fix a bug in the grammar that accidentally allowed forms like (width 500px), without any comparison

- <a id="ref-for-ratio-value③"></a>

  Delegate the definition of [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) to [\[CSS-VALUES-4\]](#biblio-css-values-4), as it is now used by more than just mediaqueries.

  <a id="ref-for-ratio-value④"></a>

  <a id="ref-for-integer-value⑥"></a>

  <a id="ref-for-integer-value⑦"></a>

  <a id="ref-for-ratio-value⑤"></a>

  <a id="ref-for-number-value②"></a>

  <a id="ref-for-number-value③"></a>

  <a id="ref-for-mult-opt"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: [\[CSS-VALUES-4\]](#biblio-css-values-4) has expanded the definition from <code><a href="https://www.w3.org/TR/css-values-4/#ratio-value">&lt;ratio&gt;</a> = <a href="https://www.w3.org/TR/css-values-4/#integer-value">&lt;integer&gt;</a> / <span>&lt;integer&gt;</span></code> to <code><a href="https://www.w3.org/TR/css-values-4/#ratio-value">&lt;ratio&gt;</a> = <a href="https://www.w3.org/TR/css-values-4/#number-value">&lt;number &#x5B;0,∞&#x5D;&gt;</a> &#x5B; / <span>&lt;number &#x5B;0,∞&#x5D;&gt;</span> &#x5D;<a href="https://www.w3.org/TR/css-values-3/#mult-opt">?</a></code>

- Various editorial tweaks, phrasing improvements, and clarifications.

- <a id="ref-for-paged-media⑤"></a>

  <a id="ref-for-continuous-media⑤"></a>

  Add definitions for the terms [continuous media](#continuous-media) and [paged media](#paged-media).

- <a id="ref-for-descdef-media-overflow-block②"></a>

  Dropped the `optional-paged` value of [overflow-block](#descdef-media-overflow-block) due to a lack of current UAs having the behavior that it described.

- <a id="ref-for-descdef-media-update③"></a>

  Mark [update](#descdef-media-update) at risk.

### <a id="changes-2017"></a> Changes since the 19 May 2017 Working Draft

The following changes were made to this specification since the [19 May 2017 Working Draft](https://www.w3.org/TR/2017/WD-mediaqueries-4-20170519/) :

- <a id="ref-for-false-in-the-negative-range⑧"></a>

  Changed range media features to be [false in the negative range](#false-in-the-negative-range) instead of failing to parse negative values.

- <a id="ref-for-descdef-media-color-gamut⑤"></a>

  Included enough information about the color spaces needed by [color-gamut](#descdef-media-color-gamut) directly into the specification.

- <a id="ref-for-descdef-media-any-pointer①⑤"></a>

  <a id="ref-for-descdef-media-any-hover①①"></a>

  <a id="ref-for-descdef-media-pointer①⑥"></a>

  <a id="ref-for-descdef-media-hover①⑥"></a>

  Marked [hover](#descdef-media-hover), [pointer](#descdef-media-pointer), [any-hover](#descdef-media-any-hover), and [any-pointer](#descdef-media-any-pointer) as no longer at-risk.

### <a id="changes-2012"></a> Changes Since Media Queries Level 3

The following changes were made to this specification since the [19 June 2012 Recommendation of Media Queries Level 3](https://www.w3.org/TR/css3-mediaqueries/):

- Large editorial rewrite and reorganization of the document.

- <a id="ref-for-media-feature③⑧"></a>

  [Boolean-context](#mq-boolean-context) [media features](#media-feature) are now additionally false if they would be true for the keyword none.

- <a id="ref-for-media-feature③⑨"></a>

  [Media features](#media-feature) with numeric values can now be written in a [range context](#mq-range-context).

- <a id="ref-for-descdef-media-overflow-inline③"></a>

  <a id="ref-for-descdef-media-overflow-block③"></a>

  <a id="ref-for-descdef-media-color-gamut⑥"></a>

  <a id="ref-for-descdef-media-update④"></a>

  <a id="ref-for-descdef-media-any-hover①②"></a>

  <a id="ref-for-descdef-media-hover①⑦"></a>

  <a id="ref-for-descdef-media-any-pointer①⑥"></a>

  <a id="ref-for-descdef-media-pointer①⑦"></a>

  The [pointer](#descdef-media-pointer), [any-pointer](#descdef-media-any-pointer), [hover](#descdef-media-hover), [any-hover](#descdef-media-any-hover), [update](#descdef-media-update), [color-gamut](#descdef-media-color-gamut), [overflow-block](#descdef-media-overflow-block), and [overflow-inline](#descdef-media-overflow-inline) media features were added.

- <a id="ref-for-valdef-media-not⑧"></a>

  <a id="ref-for-valdef-media-only⑤"></a>

  or, and, [only](#valdef-media-only) and [not](#valdef-media-not) are disallowed from being recognized as media types, even invalid ones. (They’ll trigger a syntax error instead.)

- <a id="ref-for-valdef-media-all"></a>

  <a id="ref-for-valdef-media-speech③"></a>

  <a id="ref-for-valdef-media-print①"></a>

  <a id="ref-for-valdef-media-screen⑥"></a>

  All media types except for [screen](#valdef-media-screen), [print](#valdef-media-print), [speech](#valdef-media-speech), and [all](#valdef-media-all) are deprecated.

- <a id="ref-for-web-exposed-screen-area②"></a>

  <a id="ref-for-descdef-media-device-aspect-ratio②"></a>

  <a id="ref-for-descdef-media-device-height④"></a>

  <a id="ref-for-descdef-media-device-width④"></a>

  Deprecated [device-width](#descdef-media-device-width), [device-height](#descdef-media-device-height), [device-aspect-ratio](#descdef-media-device-aspect-ratio), and made them refer to the [Web-exposed screen area](https://www.w3.org/TR/cssom-view-1/#web-exposed-screen-area) instead of the screen for privacy and security reasons.

- Mediaqueries may depend on the evaluation of style sheets in some cases

## <a id="acknowledgments"></a> Acknowledgments

This specification is the product of the W3C Working Group on Cascading Style Sheets.

Comments from Amelia Bellamy-Royds, Andreas Lind, Andres Galante, Arve Bersvendsen, Björn Höhrmann, Chris Lilley, Chris Rebert, Christian Biesinger, Christoph Päper, Dean Jackson, Elika J. Etemad (fantasai), Emilio Cobos Álvarez, François Remy, Frédéric Wang, Greg Whitworth, Ian Pouncey, James Craig, Jinfeng Ma, Kivi Shapiro, L. David Baron, Masataka Yakura, Melinda Grant, Michael\[tm\] Smith, Nicholas C. Zakas Patrick H. Lauke, Philipp Hoschka, Rick Byers, Rijk van Geijtenbeek, Roger Gimson, Sam Sneddon, Sigurd Lerstad, Simon Kissane, Simon Pieters, Steven Pemberton, Susan Lesch, Tantek Çelik, Thomas Wisniewski, Vi Nguyen, Xidorn Quan, Yves Lafon, and 張俊芝 improved this specification.

## <a id="security"></a>Security Considerations

No new security considerations have been reported on this specification.

## <a id="privacy"></a>Privacy Considerations

Media Queries enable CSS to query various aspects of the page’s environment, including things that can be difficult or impossible to find via scripting. This is potentially a privacy hazard, allowing enhanced fingerprinting of a user, but the risk is generally low. At minimum, the same information should be <em>inferrable</em> via scripting by examining the user agent string. However, UA string spoofing does not affect Media Queries, making this a somewhat more robust detection technique.

That said, the information granted by Media Queries is relatively coarse, and does not contribute much entropy in this regard.

<a id="ref-for-descdef-media-device-width⑤"></a>

<a id="ref-for-descdef-media-device-height⑤"></a>

<a id="ref-for-descdef-media-device-aspect-ratio③"></a>

A few legacy Media Features ([device-width](#descdef-media-device-width), [device-height](#descdef-media-device-height), and [device-aspect-ratio](#descdef-media-device-aspect-ratio)) expose information about the environment in which the UA is running without any clear benefit to doing so. They are retained for compatibility reasons, but for the sake of privacy and security, UAs have been allowed to report inaccurate information.

The [TAG](https://www.w3.org/2001/tag/) has developed a [self-review questionnaire](https://www.w3.org/TR/security-privacy-questionnaire/) to help editors and Working Groups evaluate the risks introduced by their specifications. Answers are provided below.

Does this specification deal with personally-identifiable information?  
No.

Does this specification deal with high-value data?  
No.

Does this specification introduce new state for an origin that persists across browsing sessions?  
No.

Does this specification expose persistent, cross-origin state to the web?  
No.

Does this specification expose any other data to an origin that it doesn’t currently have access to?  
No.

Does this specification enable new script execution/loading mechanisms?  
No.

Does this specification allow an origin access to a user’s location?  
No.

Does this specification allow an origin access to sensors on a user’s device?  
No.

Does this specification allow an origin access to aspects of a user’s local computing environment?  
Yes, as described in the prose above this questionnaire.

Does this specification allow an origin access to other devices?  
No.

Does this specification allow an origin some measure of control over a user agent’s native UI?  
No.

Does this specification expose temporary identifiers to the web?  
No.

Does this specification distinguish between behavior in first-party and third-party contexts?  
No.

How should this specification work in the context of a user agent’s "incognito" mode?  
No difference in behavior is needed.

Does this specification persist data to a user’s local device?  
No.

Does this specification have a "Security Considerations" and "Privacy Considerations" section?  
Yes, this is the section you are currently reading.

Does this specification allow downgrading default security characteristics?  
No.

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

- [all](#valdef-media-all), in § 2.3
- [any-hover](#descdef-media-any-hover), in § 7.3
- [any-pointer](#descdef-media-any-pointer), in § 7.3
- [aspect-ratio](#descdef-media-aspect-ratio), in § 4.3
- [aural](#valdef-media-aural), in § 2.3
- [boolean context](#boolean-context), in § 2.4.2
- [braille](#valdef-media-braille), in § 2.3
- [coarse](#valdef-media-pointer-coarse), in § 7.1
- [color](#descdef-media-color), in § 6.1
- [color-gamut](#descdef-media-color-gamut), in § 6.4
- [color-index](#descdef-media-color-index), in § 6.2
- [continuous media](#continuous-media), in § 5.5
- [device-aspect-ratio](#descdef-media-device-aspect-ratio), in § Unnumbered section
- [device-height](#descdef-media-device-height), in § Unnumbered section
- [device-width](#descdef-media-device-width), in § Unnumbered section
- [embossed](#valdef-media-embossed), in § 2.3
- [false in the negative range](#false-in-the-negative-range), in § 2.4.3
- [fast](#valdef-media-update-fast), in § 5.4
- [fine](#valdef-media-pointer-fine), in § 7.1
- [\<general-enclosed\>](#typedef-general-enclosed), in § 3
- [grid](#descdef-media-grid), in § 5.3
- [handheld](#valdef-media-handheld), in § 2.3
- [height](#descdef-media-height), in § 4.2
- hover
  - [descriptor for @media](#descdef-media-hover), in § 7.2
  - [value for @media/hover](#valdef-media-hover-hover), in § 7.2
- [infinite](#valdef-media-resolution-infinite), in § 5.1
- [interlace](#valdef-media-scan-interlace), in § 5.2
- [landscape](#valdef-media-orientation-landscape), in § 4.4
- [\<media-and\>](#typedef-media-and), in § 3
- [\<media-condition\>](#typedef-media-condition), in § 3
- [media condition](#media-condition), in § 2.5
- [\<media-condition-without-or\>](#typedef-media-condition-without-or), in § 3
- [\<media-feature\>](#typedef-media-feature), in § 3
- [media feature](#media-feature), in § 2.4
- [\<media-in-parens\>](#typedef-media-in-parens), in § 3
- [\<media-not\>](#typedef-media-not), in § 3
- [\<media-or\>](#typedef-media-or), in § 3
- [\<media-query\>](#typedef-media-query), in § 3
- [media query](#media-query), in § 2
- [\<media-query-list\>](#typedef-media-query-list), in § 3
- [media query list](#media-query-list), in § 2.1
- [media query modifier](#media-query-modifier), in § 2.2
- [\<media-type\>](#typedef-media-type), in § 3
- [media type](#media-type), in § 2.3
- [\<mf-boolean\>](#typedef-mf-boolean), in § 3
- [\<mf-comparison\>](#typedef-mf-comparison), in § 3
- [\<mf-eq\>](#typedef-mf-eq), in § 3
- [\<mf-gt\>](#typedef-mf-gt), in § 3
- [\<mf-lt\>](#typedef-mf-lt), in § 3
- [\<mf-name\>](#typedef-mf-name), in § 3
- [\<mf-plain\>](#typedef-mf-plain), in § 3
- [\<mf-range\>](#typedef-mf-range), in § 3
- [\<mf-value\>](#typedef-mf-value), in § 3
- [monochrome](#descdef-media-monochrome), in § 6.3
- [\<mq-boolean\>](#typedef-mq-boolean), in § 5.3
- none
  - [value for @media/hover](#valdef-media-hover-none), in § 7.2
  - [value for @media/overflow-block](#valdef-media-overflow-block-none), in § 5.5
  - [value for @media/overflow-inline](#valdef-media-overflow-inline-none), in § 5.6
  - [value for @media/pointer](#valdef-media-pointer-none), in § 7.1
  - [value for @media/update](#valdef-media-update-none), in § 5.4
- [not](#valdef-media-not), in § 2.2.1
- [only](#valdef-media-only), in § 2.2.2
- [orientation](#descdef-media-orientation), in § 4.4
- [overflow-block](#descdef-media-overflow-block), in § 5.5
- [overflow-inline](#descdef-media-overflow-inline), in § 5.6
- [p3](#valdef-media-color-gamut-p3), in § 6.4
- [paged](#valdef-media-overflow-block-paged), in § 5.5
- [paged media](#paged-media), in § 5.5
- [pointer](#descdef-media-pointer), in § 7.1
- [portrait](#valdef-media-orientation-portrait), in § 4.4
- [print](#valdef-media-print), in § 2.3
- [progressive](#valdef-media-scan-progressive), in § 5.2
- [projection](#valdef-media-projection), in § 2.3
- [range context](#range-context), in § 2.4.3
- [rec2020](#valdef-media-color-gamut-rec2020), in § 6.4
- [resolution](#descdef-media-resolution), in § 5.1
- [scan](#descdef-media-scan), in § 5.2
- [screen](#valdef-media-screen), in § 2.3
- scroll
  - [value for @media/overflow-block](#valdef-media-overflow-block-scroll), in § 5.5
  - [value for @media/overflow-inline](#valdef-media-overflow-inline-scroll), in § 5.6
- [slow](#valdef-media-update-slow), in § 5.4
- [speech](#valdef-media-speech), in § 2.3
- [srgb](#valdef-media-color-gamut-srgb), in § 6.4
- [tty](#valdef-media-tty), in § 2.3
- [tv](#valdef-media-tv), in § 2.3
- [update](#descdef-media-update), in § 5.4
- [width](#descdef-media-width), in § 4.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="3eef835e"></a>@import
  - <a id="5b7444b2"></a>cascade layers
  - <a id="6b448e93"></a>initial value
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="4397147f"></a>@media
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="297dfe3a"></a>font-size
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="4920620f"></a>\<any-value\>
  - <a id="f9309bf7"></a>\<delim-token\>
  - <a id="a6331414"></a>\<function-token\>
  - <a id="073ee1d9"></a>parse a comma-separated list of component values
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="8aa385d4"></a>\<dimension\>
  - <a id="630b8044"></a>\<ident\>
  - <a id="569c7f9f"></a>?
  - <a id="8df3bbd5"></a>cm
  - <a id="62b901e0"></a>em
  - <a id="8c377f49"></a>in
  - <a id="11e45893"></a>px
  - <a id="3952e8de"></a>relative length
  - <a id="a0a83db1"></a>\|
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="d73c993d"></a>\<integer\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="ee68e69a"></a>\<ratio\>
  - <a id="9108b09d"></a>\<resolution\>
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="a6eb24bb"></a>inline axis
- \[CSSOM-VIEW-1\] defines the following terms:
  - <a id="0496c9ed"></a>page zoom
  - <a id="66946fda"></a>scale factor
  - <a id="793a0b0d"></a>web-exposed screen area
- \[INFRA\] defines the following terms:
  - <a id="7f9469b5"></a>ASCII case-insensitive
- \[SELECTORS-4\] defines the following terms:
  - <a id="b725b7bc"></a>:hover
  - <a id="f6cdcdf7"></a>pseudo-class

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-colorimetry"></a>\[COLORIMETRY\]  
[Colorimetry, Fourth Edition. CIE 015:2018](http://www.cie.co.at/publications/colorimetry-4th-edition). 2018. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;cie&#x2E;co&#x2E;at&#x2F;publications&#x2F;colorimetry-4th-edition](http://www.cie.co.at/publications/colorimetry-4th-edition)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 15 August 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-cssom-view-1"></a>\[CSSOM-VIEW-1\]  
Simon Fraser; Emilio Cobos Álvarez. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 16 September 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-mediaqueries-3"></a>\[MEDIAQUERIES-3\]  
Florian Rivoal. [Media Queries Level 3](https://www.w3.org/TR/mediaqueries-3/). 21 May 2024. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-3&#x2F;](https://www.w3.org/TR/mediaqueries-3/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 1 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-display-p3"></a>\[Display-P3\]  
A; et al. [Display P3](https://www.color.org/chardata/rgb/DisplayP3.xalter). 2022-02. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;color&#x2E;org&#x2F;chardata&#x2F;rgb&#x2F;DisplayP3&#x2E;xalter](https://www.color.org/chardata/rgb/DisplayP3.xalter)

<a id="biblio-html401"></a>\[HTML401\]  
Dave Raggett; Arnaud Le Hors; Ian Jacobs. [HTML 4.01 Specification](https://www.w3.org/TR/html401/). 27 March 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;html401&#x2F;](https://www.w3.org/TR/html401/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-itu-r-bt-2020-2"></a>\[ITU-R-BT-2020-2\]  
[Parameter values for ultra-high definition television systems for production and international programme exchange](https://www.itu.int/rec/R-REC-BT.2020/en). October 2015. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;itu&#x2E;int&#x2F;rec&#x2F;R-REC-BT&#x2E;2020&#x2F;en](https://www.itu.int/rec/R-REC-BT.2020/en)

<a id="biblio-rfc2879"></a>\[RFC2879\]  
G. Klyne; L. McIntyre. [Content Feature Schema for Internet Fax (V2)](https://www.rfc-editor.org/rfc/rfc2879). August 2000. Proposed Standard. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;rfc-editor&#x2E;org&#x2F;rfc&#x2F;rfc2879](https://www.rfc-editor.org/rfc/rfc2879)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 22 January 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-srgb"></a>\[SRGB\]  
[Multimedia systems and equipment - Colour measurement and management - Part 2-1: Colour management - Default RGB colour space - sRGB](https://webstore.iec.ch/publication/6169). URL: [https&#x3A;&#x2F;&#x2F;webstore&#x2E;iec&#x2E;ch&#x2F;publication&#x2F;6169](https://webstore.iec.ch/publication/6169)

<a id="biblio-xml-stylesheet"></a>\[XML-STYLESHEET\]  
James Clark; Simon Pieters; Henry Thompson. [Associating Style Sheets with XML documents 1.0 (Second Edition)](https://www.w3.org/TR/xml-stylesheet/). 28 October 2010. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;xml-stylesheet&#x2F;](https://www.w3.org/TR/xml-stylesheet/)

## <a id="property-index"></a>Property Index

No properties defined.

<a id="ref-for-at-ruledef-media②④"></a>

### <a id="media-descriptor-table"></a>[@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) Descriptors

| Name                | Value                      | Initial | Type     |
|---------------------|----------------------------|---------|----------|
| <strong><span><a id="ref-for-descdef-media-any-hover①③"></a></span><a href="#descdef-media-any-hover">any-hover</a>&#xA;      </strong> | none \| hover              |         | discrete |
| <strong><span><a id="ref-for-descdef-media-any-pointer①⑦"></a></span><a href="#descdef-media-any-pointer">any-pointer</a>&#xA;      </strong> | none \| coarse \| fine     |         | discrete |
| <strong><span><a id="ref-for-descdef-media-aspect-ratio③"></a></span><a href="#descdef-media-aspect-ratio">aspect-ratio</a>&#xA;      </strong> | \<ratio\>                  |         | range    |
| <strong><span><a id="ref-for-descdef-media-color⑦"></a></span><a href="#descdef-media-color">color</a>&#xA;      </strong> | \<integer\>                |         | range    |
| <strong><span><a id="ref-for-descdef-media-color-gamut⑦"></a></span><a href="#descdef-media-color-gamut">color-gamut</a>&#xA;      </strong> | srgb \| p3 \| rec2020      |         | discrete |
| <strong><span><a id="ref-for-descdef-media-color-index③"></a></span><a href="#descdef-media-color-index">color-index</a>&#xA;      </strong> | \<integer\>                |         | range    |
| <strong><span><a id="ref-for-descdef-media-device-aspect-ratio④"></a></span><a href="#descdef-media-device-aspect-ratio">device-aspect-ratio</a>&#xA;      </strong> | \<ratio\>                  |         | range    |
| <strong><span><a id="ref-for-descdef-media-device-height⑥"></a></span><a href="#descdef-media-device-height">device-height</a>&#xA;      </strong> | \<length\>                 |         | range    |
| <strong><span><a id="ref-for-descdef-media-device-width⑥"></a></span><a href="#descdef-media-device-width">device-width</a>&#xA;      </strong> | \<length\>                 |         | range    |
| <strong><span><a id="ref-for-descdef-media-grid⑤"></a></span><a href="#descdef-media-grid">grid</a>&#xA;      </strong> | \<mq-boolean\>             |         | discrete |
| <strong><span><a id="ref-for-descdef-media-height⑥"></a></span><a href="#descdef-media-height">height</a>&#xA;      </strong> | \<length\>                 |         | range    |
| <strong><span><a id="ref-for-descdef-media-hover①⑧"></a></span><a href="#descdef-media-hover">hover</a>&#xA;      </strong> | none \| hover              |         | discrete |
| <strong><span><a id="ref-for-descdef-media-monochrome③"></a></span><a href="#descdef-media-monochrome">monochrome</a>&#xA;      </strong> | \<integer\>                |         | range    |
| <strong><span><a id="ref-for-descdef-media-orientation④"></a></span><a href="#descdef-media-orientation">orientation</a>&#xA;      </strong> | portrait \| landscape      |         | discrete |
| <strong><span><a id="ref-for-descdef-media-overflow-block④"></a></span><a href="#descdef-media-overflow-block">overflow-block</a>&#xA;      </strong> | none \| scroll \| paged    |         | discrete |
| <strong><span><a id="ref-for-descdef-media-overflow-inline④"></a></span><a href="#descdef-media-overflow-inline">overflow-inline</a>&#xA;      </strong> | none \| scroll             |         | discrete |
| <strong><span><a id="ref-for-descdef-media-pointer①⑧"></a></span><a href="#descdef-media-pointer">pointer</a>&#xA;      </strong> | none \| coarse \| fine     |         | discrete |
| <strong><span><a id="ref-for-descdef-media-resolution⑤"></a></span><a href="#descdef-media-resolution">resolution</a>&#xA;      </strong> | \<resolution\> \| infinite |         | range    |
| <strong><span><a id="ref-for-descdef-media-scan④"></a></span><a href="#descdef-media-scan">scan</a>&#xA;      </strong> | interlace \| progressive   |         | discrete |
| <strong><span><a id="ref-for-descdef-media-update⑤"></a></span><a href="#descdef-media-update">update</a>&#xA;      </strong> | none \| slow \| fast       |         | discrete |
| <strong><span><a id="ref-for-descdef-media-width⑨"></a></span><a href="#descdef-media-width">width</a>&#xA;      </strong> | \<length\>                 |         | range    |

