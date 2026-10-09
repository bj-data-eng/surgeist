Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/2022/CR-css-cascade-4-20220113/).

Original copyright notice: Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Cascading and Inheritance Level 4

Source snapshot: https://www.w3.org/TR/2022/CR-css-cascade-4-20220113/

Snapshot SHA-256: 7787bd75c1f7187fdfdfdf9e7b5b8391faeac0e7c80dd19461de8426931138e5

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 4 source tables are presented as readable Markdown tables or explicit labeled layouts: 4 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Cascading and Inheritance Level 4

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2022 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This CSS module describes how to collate style rules and assign values to all properties on all elements. By way of cascading and inheritance, values are propagated for all properties on all elements.

<a id="ref-for-valdef-all-revert"></a>

<a id="ref-for-typedef-supports-condition"></a>

<a id="ref-for-at-ruledef-import"></a>

New in this level are the [revert](#valdef-all-revert) keyword and [\<supports-condition\>](https://www.w3.org/TR/css3-conditional/#typedef-supports-condition) for the [@import](#at-ruledef-import) rule.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Snapshot</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Snapshot has received [wide review](https://www.w3.org/2021/Process-20211102/#dfn-wide-review), is intended to gather implementation experience, and has commitments from Working Group members to [royalty-free licensing](https://www.w3.org/Consortium/Patent-Policy/#sec-Requirements) for implementations. This document is intended to become a W3C Recommendation; it will remain a Candidate Recommendation at least until 13 March 2022 to gather additional feedback.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-cascade” in the title, like this: “\[css-cascade\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-cascade%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<a id="ref-for-css-property"></a>

<a id="ref-for-propdef-color"></a>

<a id="ref-for-descdef-font-face-font-size"></a>

<a id="ref-for-propdef-border-style"></a>

<a id="ref-for-typedef-color"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="ref-for-css-property-declarations"></a>

<a id="ref-for-valdef-color-red"></a>

<a id="ref-for-valdef-line-style-dotted"></a>

CSS defines a finite set of parameters, called <a id="css-property"></a>properties, that direct the rendering of a document. Each [property](#css-property) has a name (e.g., [color](https://www.w3.org/TR/css-color-4/#propdef-color), [font-size](https://www.w3.org/TR/css-fonts-5/#descdef-font-face-font-size), or [border-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-style)), a value space (e.g., [\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color), [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage), \[ solid \| dashed \| dotted \| … \]), and a defined behavior on the rendering of the document. Properties values are assigned to various parts of the document via [property declarations](https://www.w3.org/TR/css-syntax-3/#css-property-declarations), which assign the property a value (e.g. [red](https://www.w3.org/TR/css-color-4/#valdef-color-red), 12pt, [dotted](https://www.w3.org/TR/css-backgrounds-3/#valdef-line-style-dotted)) for the associated element or box.

<a id="ref-for-cascade"></a>

One of the fundamental design principles of CSS is [cascading](#cascade), which allows several style sheets to influence the presentation of a document. When different declarations try to set a value for the same element/property combination, the conflicts must somehow be resolved.

<a id="ref-for-inheritance"></a>

<a id="ref-for-initial-value"></a>

The opposite problem arises when no declarations try to set a the value for an element/property combination. In this case, a value is be found by way of [inheritance](#inheritance) or by looking at the property’s [initial value](#initial-value).

<a id="ref-for-cascade①④"></a>

<a id="ref-for-specified-value"></a>

The [cascading](#cascade) and [defaulting](#defaulting) process takes a set of declarations as input, and outputs a [specified value](#specified-value) for each property on each element.

The rules for finding the specified value for all properties on all elements in the document are described in this specification. The rules for finding the specified values in the page context and its margin boxes are described in [\[css-page-3\]](#biblio-css-page-3).

### <a id="placement"></a>1.1.  Module Interactions

<em>This section is normative.</em>

This module replaces and extends the rules for assigning property values, cascading, and inheritance defined in [\[CSS2\]](#biblio-css2) chapter 6.

<a id="ref-for-typedef-media-query"></a>

Other CSS modules may expand the definitions of some of the syntax and features defined here. For example, the Media Queries Level 4 specification, when combined with this module, expands the definition of the [\<media-query\>](https://www.w3.org/TR/mediaqueries-4/#typedef-media-query) value type as used in this specification.

<a id="ref-for-text-nodes"></a>

<a id="ref-for-elements"></a>

For the purpose of this specification, [text nodes](https://www.w3.org/TR/css-display-3/#text-nodes) are treated as [element](https://www.w3.org/TR/css-display-3/#elements) children of their associated element, and possess the full set of properties; since they cannot be targeted by selectors all of their computed values are assigned by [defaulting](#defaulting).

<a id="ref-for-at-ruledef-import①"></a>

## <a id="at-import"></a>2.  Importing Style Sheets: the [@import](#at-ruledef-import) rule

<a id="ref-for-at-ruledef-import②"></a>

The <a id="at-ruledef-import"></a>@import rule allows users to import style rules from other style sheets. If an [@import](#at-ruledef-import) rule refers to a valid stylesheet, user agents must treat the contents of the stylesheet as if they were written in place of the <a id="ref-for-at-ruledef-import③"></a>@import rule, with two exceptions:

- If a feature (such as the @namespace rule) <em>explicitly</em> defines that it only applies to a particular stylesheet, and not any imported ones, then it doesn’t apply to the imported stylesheet.

- <a id="ref-for-at-ruledef-import④"></a>

  If a feature relies on the relative ordering of two or more constructs in a stylesheet (such as the requirement that @namespace rules must not have any other rules other than [@import](#at-ruledef-import) preceding it), it only applies between constructs in the same stylesheet.

<a id="ref-for-at-ruledef-import⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d425da12"></a> For example, declarations in style rules from imported stylesheets interact with the cascade as if they were written literally into the stylesheet at the point of the [@import](#at-ruledef-import).

<a id="ref-for-at-ruledef-import⑥"></a>

<a id="ref-for-at-ruledef-charset"></a>

Any [@import](#at-ruledef-import) rules must precede all other valid at-rules and style rules in a style sheet (ignoring [@charset](https://www.w3.org/TR/css-syntax-3/#at-ruledef-charset)), or else the <a id="ref-for-at-ruledef-import⑦"></a>@import rule is invalid. The syntax of <a id="ref-for-at-ruledef-import⑧"></a>@import is:

<a id="ref-for-url-value"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-string-value"></a>

<a id="ref-for-typedef-supports-condition①"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-typedef-media-query-list"></a>

<a id="ref-for-mult-opt①"></a>

```text
@import [ <url> | <string> ]
        [ supports( [ <supports-condition> | <declaration> ] ) ]?
        <media-query-list>? ;
```
<a id="ref-for-url-value①"></a>

<a id="ref-for-string-value①"></a>

<a id="ref-for-typedef-supports-condition②"></a>

<a id="ref-for-typedef-media-query-list①"></a>

where the [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) or [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) gives the URL of the style sheet to be imported, and the optional \[[\<supports-condition\>](https://www.w3.org/TR/css3-conditional/#typedef-supports-condition)\|\<declaration\>\] and [\<media-query-list\>](https://www.w3.org/TR/mediaqueries-4/#typedef-media-query-list) (collectively, the <a id="import-conditions"></a>import conditions) state the conditions under which it applies.

<a id="ref-for-propdef-display"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-05c74581"></a> The following [conditional @import rule](#conditional-import) only loads the style sheet when the UA [supports](https://www.w3.org/TR/css-conditional-3/#support-definition) [display: flex](https://www.w3.org/TR/CSS2/visuren.html#propdef-display), and only applies the style sheet on a [handheld](https://www.w3.org/TR/CSS2/media.html#media-types) device with a [maximum viewport width](https://www.w3.org/TR/mediaqueries-4/#width) of 400px.
>
> ```text
> @import url("narrow.css") supports(display: flex) handheld and (max-width: 400px);
> ```
<a id="ref-for-string-value②"></a>

<a id="ref-for-url-value②"></a>

If a [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) is provided, it must be interpreted as a [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) with the same value.

<a id="ref-for-at-ruledef-import⑨"></a>

<a id="ref-for-funcdef-url"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-da2e49cb"></a> The following lines are equivalent in meaning and illustrate both [@import](#at-ruledef-import) syntaxes (one with [url()](https://www.w3.org/TR/css-values-4/#funcdef-url) and one with a bare string):
>
> ```css
> @import "mystyle.css";
> @import url("mystyle.css");
> ```
<a id="ref-for-at-ruledef-import①⓪"></a>

### <a id="conditional-import"></a>2.1.  Conditional [@import](#at-ruledef-import) Rules

<a id="ref-for-import-conditions"></a>

<a id="ref-for-valdef-media-all"></a>

<a id="ref-for-typedef-media-query-list②"></a>

<a id="ref-for-at-ruledef-media"></a>

<a id="ref-for-at-ruledef-supports"></a>

The [import conditions](#import-conditions) allow the import to be media– or feature-support–dependent. In the absence of any <a id="ref-for-import-conditions①"></a>import conditions, the import is unconditional. (Specifying [all](https://www.w3.org/TR/mediaqueries-4/#valdef-media-all) for the [\<media-query-list\>](https://www.w3.org/TR/mediaqueries-4/#typedef-media-query-list) has the same effect.) If the <a id="ref-for-import-conditions②"></a>import conditions do not match, the rules in the imported stylesheet do not apply, exactly as if the imported stylesheet were wrapped in [@media](https://www.w3.org/TR/css3-conditional/#at-ruledef-media) and/or [@supports](https://www.w3.org/TR/css3-conditional/#at-ruledef-supports) blocks with the given conditions.

<a id="ref-for-at-ruledef-import①①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5fa651f3"></a> The following rules illustrate how [@import](#at-ruledef-import) rules can be made media-dependent:
>
> ```css
> @import url("fineprint.css") print;
> @import url("bluish.css") projection, tv;
> @import url("narrow.css") handheld and (max-width: 400px);
> ```
<a id="ref-for-import-conditions③"></a>

<a id="ref-for-typedef-supports-condition③"></a>

User agents may therefore avoid fetching a conditional import as long as the [import conditions](#import-conditions) do not match. Additionally, if a [\<supports-condition\>](https://www.w3.org/TR/css3-conditional/#typedef-supports-condition) blocks the application of the imported style sheet, the UA <em>must not</em> fetch the style sheet (unless it is loaded through some other link) and <em>must</em> return null for the import rule’s CSSImportRule.styleSheet value (even if it is loaded through some other link).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c6c2c0ee"></a> The following rule illustrates how an author can provide fallback rules for legacy user agents without impacting network performance on newer user agents:
>
> ```css
> @import url("fallback-layout.css") supports(not (display: flex));
> @supports (display: flex) {
>   ...
> }
> ```
<a id="ref-for-import-conditions④"></a>

<a id="ref-for-typedef-media-query-list③"></a>

<a id="ref-for-media-query-list"></a>

<a id="ref-for-typedef-supports-condition④"></a>

<a id="ref-for-typedef-supports-decl"></a>

The [import conditions](#import-conditions) are given by [\<media-query-list\>](https://www.w3.org/TR/mediaqueries-4/#typedef-media-query-list), which is parsed and interpreted as a [media query list](https://www.w3.org/TR/mediaqueries-4/#media-query-list), and [\<supports-condition\>](https://www.w3.org/TR/css3-conditional/#typedef-supports-condition), is parsed and interpreted as a \[\[supports query\]\]. If a \<declaration\> is given in place of a <a id="ref-for-typedef-supports-condition⑤"></a>\<supports-condition\>, it must be interpreted as a [\<supports-decl\>](https://www.w3.org/TR/css3-conditional/#typedef-supports-decl) (i.e. the extra set of parentheses is implied) and treated as a <a id="ref-for-typedef-supports-condition⑥"></a>\<supports-condition\>.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-75806df4"></a> For example, the following two lines are equivalent:
>
> ```css
> @import "mystyle.css" supports(display: flex);
> @import "mystyle.css" supports((display: flex));
> ```
<a id="ref-for-import-conditions⑤"></a>

The evaluation and full syntax of the [import conditions](#import-conditions) are defined by the [Media Queries](https://www.w3.org/TR/mediaqueries/) [\[MEDIAQ\]](#biblio-mediaq) and [CSS Conditional Rules](https://www.w3.org/TR/css-conditional/) [\[CSS-CONDITIONAL-3\]](#biblio-css-conditional-3) specifications.

### <a id="import-processing"></a>2.2.  Processing Stylesheet Imports

When the same style sheet is imported or linked to a document in multiple places, user agents must process (or act as though they do) each link as though the link were to an independent style sheet.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This does not place any requirements on resource fetching, only how the style sheet is reflected in the CSSOM and used in specs such as this one. Assuming appropriate caching, it is perfectly appropriate for a UA to fetch a style sheet only once, even though it’s linked or imported multiple times.

<a id="ref-for-origin"></a>

The [cascade origin](#origin) of an imported style sheet is the <a id="ref-for-origin①"></a>cascade origin of the style sheet that imported it.

<a id="ref-for-environment-encoding"></a>

The [environment encoding](https://www.w3.org/TR/css-syntax-3/#environment-encoding) of an imported style sheet is the encoding of the style sheet that imported it. [\[css-syntax-3\]](#biblio-css-syntax-3)

<a id="ref-for-at-ruledef-import①②"></a>

To <a id="fetch-an-import"></a>fetch an @import, given an [@import](#at-ruledef-import) rule <var>rule</var>:

1.  <a id="ref-for-concept-css-rule-parent-css-style-sheet"></a>

    Let <var>parentStylesheet</var> be <var>rule</var>’s [parent CSS style sheet](https://www.w3.org/TR/cssom-1/#concept-css-rule-parent-css-style-sheet). [\[CSSOM\]](#biblio-cssom)

2.  <a id="ref-for-typedef-supports-condition⑦"></a>

    If <var>rule</var> has a [\<supports-condition\>](https://www.w3.org/TR/css3-conditional/#typedef-supports-condition), and that condition is not true, return.

3.  <a id="ref-for-concept-url-parser"></a>

    <a id="ref-for-concept-css-style-sheet-location"></a>

    Let <var>parsedUrl</var> be the result of the [URL parser](https://url.spec.whatwg.org/#concept-url-parser) steps with <var>rule</var>’s URL and <var>parentStylesheet</var>’s [location](https://www.w3.org/TR/cssom-1/#concept-css-style-sheet-location). If the algorithm returns an error, return. [\[CSSOM\]](#biblio-cssom)

4.  <a id="ref-for-fetch-a-style-resource"></a>

    <a id="ref-for-concept-response"></a>

    [Fetch a style resource](https://www.w3.org/TR/css-values-4/#fetch-a-style-resource) from <var>parsedUrl</var>, with stylesheet <var>parentStylesheet</var>, destination "style", CORS mode "no-cors", and processResponse being the following steps given [response](https://fetch.spec.whatwg.org/#concept-response) <var>response</var> and byte stream, null or failure <var>byteStream</var>:

    1.  If <var>maybeByteStream</var> is not a byte stream, return.

    2.  <a id="ref-for-concept-document-quirks"></a>

        <a id="ref-for-cors-same-origin"></a>

        If <var>parentStylesheet</var> is in [quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks) and <var>response</var> is [CORS-same-origin](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#cors-same-origin), let <var>content type</var> be `"text/css"`. Otherwise, let <var>content type</var> be the Content Type metadata of <var>response</var>.

    3.  If <var>content type</var> is not `"text/css"`, return.

    4.  <a id="ref-for-parse-a-stylesheet"></a>

        Let <var>importedStylesheet</var> be the result of [parsing](https://www.w3.org/TR/css-syntax-3/#parse-a-stylesheet) <var>byteStram</var> given <var>parsedUrl</var>.

    5.  <a id="ref-for-concept-css-style-sheet-origin-clean-flag"></a>

        Set <var>importedStylesheet</var>’s [origin-clean flag](https://www.w3.org/TR/cssom-1/#concept-css-style-sheet-origin-clean-flag) to <var>parentStylesheet</var>’s <a id="ref-for-concept-css-style-sheet-origin-clean-flag①"></a>origin-clean flag.

    6.  <a id="ref-for-cors-same-origin①"></a>

        <a id="ref-for-concept-css-style-sheet-origin-clean-flag②"></a>

        If <var>response</var> is not [CORS-same-origin](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#cors-same-origin), unset <var>importedStylesheet</var>’s [origin-clean flag](https://www.w3.org/TR/cssom-1/#concept-css-style-sheet-origin-clean-flag).

    7.  <a id="ref-for-dom-cssimportrule-stylesheet"></a>

        Set <var>rule</var>’s <code><a href="https://www.w3.org/TR/cssom-1/#dom-cssimportrule-stylesheet">styleSheet</a></code> to <var>importedStylesheet</var>.

### <a id="content-type"></a>2.3.  Content-Type of CSS Style Sheets

The processing of imported style sheets depends on the actual type of the linked resource:

- <a id="ref-for-content-type"></a>

  If the resource does not have [Content-Type metadata](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#content-type), the type is treated as `text/css`.

- <a id="ref-for-concept-document-quirks①"></a>

  <a id="ref-for-same-origin"></a>

  <a id="ref-for-concept-response①"></a>

  <a id="ref-for-concept-response-url"></a>

  If the host document is in [quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks), and the host document’s origin is [same origin](https://html.spec.whatwg.org/multipage/origin.html#same-origin) with the linked resource [response’s](https://fetch.spec.whatwg.org/#concept-response) [URL’s](https://fetch.spec.whatwg.org/#concept-response-url) origin, the type is treated as `text/css`.

- <a id="ref-for-content-type①"></a>

  Otherwise, the type is determined from its [Content-Type metadata](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#content-type).

If the linked resource’s type is `text/css`, it must be interpreted as a CSS style sheet. Otherwise, it must be interpreted as a network error.

## <a id="shorthand"></a>3.  Shorthand Properties

<a id="ref-for-shorthand-property"></a>

Some properties are <a id="shorthand-property"></a>shorthand properties, meaning that they allow authors to specify the values of several properties with a single property. A [shorthand property](#shorthand-property) sets all of its <a id="longhand"></a>longhand sub-properties, exactly as if expanded in place.

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-longhand"></a>

<a id="ref-for-initial-value①"></a>

When values are omitted from a [shorthand](#shorthand-property) form, unless otherwise defined, each “missing” [sub-property](#longhand) is assigned its [initial value](#initial-value).

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-longhand①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> This means that a [shorthand](#shorthand-property) property declaration always sets <em>all</em> of its [sub-properties](#longhand), even those that are not explicitly set. Carelessly used, this might result in inadvertently resetting some <a id="ref-for-longhand②"></a>sub-properties. Carefully used, a <a id="ref-for-shorthand-property③"></a>shorthand can guarantee a “blank slate” by resetting <a id="ref-for-longhand③"></a>sub-properties inadvertently cascaded from other sources.
>
> <a id="ref-for-propdef-background"></a>
>
> <a id="ref-for-propdef-background-color"></a>
>
> <a id="ref-for-propdef-background-image"></a>
>
> For example, writing [background: green](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) rather than [background-color: green](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color) ensures that the background color overrides any earlier declarations that might have set the background to an image with [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image).

<a id="ref-for-propdef-font"></a>

<a id="ref-for-shorthand-property④"></a>

<a id="ref-for-propdef-font-style"></a>

<a id="ref-for-propdef-font-variant"></a>

<a id="ref-for-propdef-font-weight"></a>

<a id="ref-for-descdef-font-face-font-size①"></a>

<a id="ref-for-propdef-line-height"></a>

<a id="ref-for-propdef-font-family"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8762997a"></a> For example, the CSS Level 1 [font](https://www.w3.org/TR/css-fonts-4/#propdef-font) property is a [shorthand](#shorthand-property) property for setting [font-style](https://www.w3.org/TR/css-fonts-4/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-4/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-4/#propdef-font-weight), [font-size](https://www.w3.org/TR/css-fonts-5/#descdef-font-face-font-size), [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height), and [font-family](https://www.w3.org/TR/css-fonts-4/#propdef-font-family) all at once. The multiple declarations of this example:
>
> ```css
> h1 {
>   font-weight: bold;
>   font-size: 12pt;
>   line-height: 14pt;
>   font-family: Helvetica;
>   font-variant: normal;
>   font-style: normal;
> }
> ```
>
> can therefore be rewritten as
>
> ```css
> h1 { font: bold 12pt/14pt Helvetica }
> ```
>
> <a id="ref-for-propdef-font①"></a>
>
> <a id="ref-for-longhand④"></a>
>
> As more [font](https://www.w3.org/TR/css-fonts-4/#propdef-font) [sub-properties](#longhand) are introduced into CSS, the shorthand declaration resets those to their initial values as well.

<a id="ref-for-shorthand-property⑤"></a>

<a id="ref-for-longhand⑤"></a>

In some cases, a [shorthand](#shorthand-property) might have different syntax or special keywords that don’t directly correspond to values of its [sub-properties](#longhand). (In such cases, the <a id="ref-for-shorthand-property⑥"></a>shorthand will explicitly define the expansion of its values.)

<a id="ref-for-longhand⑥"></a>

<a id="ref-for-propdef-border"></a>

<a id="ref-for-propdef-border-image"></a>

In other cases, a property might be a <a id="reset-only-sub-property"></a>reset-only sub-property of the shorthand: Like other [sub-properties](#longhand), it is reset to its initial value by the shorthand when unspecified, but the shorthand might not include syntax to set the <a id="ref-for-longhand⑦"></a>sub-property to any of its other values. For example, the [border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border) shorthand resets [border-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image) to its initial value of none, but has no syntax to set it to anything else. [\[css-backgrounds-3\]](#biblio-css-backgrounds-3)

<a id="ref-for-shorthand-property⑦"></a>

<a id="ref-for-longhand⑧"></a>

<a id="ref-for-reset-only-sub-property"></a>

If a [shorthand](#shorthand-property) is specified as one of the [CSS-wide keywords](https://www.w3.org/TR/css-values/#common-keywords) [\[css-values-3\]](#biblio-css-values-3), it sets all of its [sub-properties](#longhand) to that keyword, including any that are [reset-only sub-properties](#reset-only-sub-property). (Note that these keywords cannot be combined with other values in a single declaration, not even in a shorthand.)

<a id="ref-for-shorthand-property⑧"></a>

<a id="ref-for-longhand⑨"></a>

Declaring a [shorthand](#shorthand-property) property to be !important is equivalent to declaring all of its [sub-properties](#longhand) to be !important.

### <a id="aliasing"></a>3.1.  Property Aliasing

Properties sometimes change names after being supported for a while, such as vendor-prefixed properties being standardized. The original name still needs to be supported for compatibility reasons, but the new name is preferred. To accomplish this, CSS defines two different ways of “aliasing” old syntax to new syntax.

<a id="legacy-name-alias"></a>legacy name aliases  
When the old property’s value syntax is identical to that of the new property, the two names are aliased with an operation on par with case-mapping: at parse time, the old property is converted into the new property. This conversion also applies in the CSSOM, both for string arguments and property accessors: requests for the old property name transparently transfer to the new property name instead.

<a id="ref-for-legacy-name-alias"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5dd1d5a6"></a> For example, if old-name is a [legacy name alias](#legacy-name-alias) for new-name, `getComputedStyle(el).oldName` will return the computed style of the `newName` property, and `el.style.setPropertyValue("old-name", "value")` will set the new-name property to `"value"`.

<a id="legacy-shorthand"></a>legacy shorthands  
<a id="ref-for-legacy-shorthand"></a>

<a id="ref-for-shorthand-property⑨"></a>

When the old property has a distinct syntax from the new property, the two names are aliased using the [shorthand](#shorthand-property) mechanism. These shorthands are defined to be [legacy shorthands](#legacy-shorthand), and their use is <em>deprecated</em>. They otherwise behave exactly as regular shorthands, except that the CSSOM will not use them when serializing declarations. [\[CSSOM\]](#biblio-cssom)

<a id="ref-for-legacy-shorthand①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c7a1887e"></a> For example, the page-break-\* properties are [legacy shorthands](#legacy-shorthand) for the break-\* properties (see [CSS Fragmentation 3 § 3.4 Page Break Aliases: the page-break-before, page-break-after, and page-break-inside properties](https://www.w3.org/TR/css-break-3/#page-break-properties)).
> <a id="ref-for-propdef-page-break-before"></a>
>
> <a id="ref-for-propdef-break-before"></a>
>
> Setting [page-break-before: always](https://www.w3.org/TR/CSS2/page.html#propdef-page-break-before) expands to [break-before: page](https://www.w3.org/TR/css-break-3/#propdef-break-before) at parse time, like other shorthands do. Similarly, if <a id="ref-for-propdef-break-before①"></a>break-before: page is set, calling `getComputedStyle(el).pageBreakBefore` will return `"always"`. However, when serializing a style block (see [CSSOM 1 § 6.7.2 Serializing CSS Values](https://www.w3.org/TR/cssom-1/#serializing-css-values)), the <a id="ref-for-propdef-page-break-before①"></a>page-break-before property will never be chosen as the shorthand to serialize to, regardless of whether it or <a id="ref-for-propdef-break-before②"></a>break-before was specified; instead, <a id="ref-for-propdef-break-before③"></a>break-before will always be chosen.

<a id="ref-for-propdef-all"></a>

### <a id="all-shorthand"></a>3.2.  Resetting All Properties: the [all](#propdef-all) property

| Field               | Definition                                                                                                                                     |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-all"></a>all                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②"></a>initial [\|](https://www.w3.org/TR/css-values-4/#comb-one) inherit <a id="ref-for-comb-one③"></a>\| unset <a id="ref-for-comb-one④"></a>\| revert |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                    |

<a id="ref-for-propdef-all①"></a>

<a id="ref-for-shorthand-property①⓪"></a>

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-propdef-unicode-bidi"></a>

<a id="ref-for-custom-property"></a>

The [all](#propdef-all) property is a [shorthand](#shorthand-property) that resets <em>all</em> CSS properties except [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) and [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi). It only accepts the [CSS-wide keywords](https://www.w3.org/TR/css-values/#common-keywords). It does not reset [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) [\[css-variables-1\]](#biblio-css-variables-1).

<a id="ref-for-propdef-direction①"></a>

<a id="ref-for-propdef-unicode-bidi①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The excepted CSS properties [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) and [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi) are actually markup-level features, and [should not be set in the author’s style sheet](https://www.w3.org/TR/css-writing-modes-3/#text-direction). (They exist as CSS properties only to style document languages not supported by the UA.) Authors should use the appropriate markup, such as HTML’s `dir` attribute, instead. [\[css-writing-modes-3\]](#biblio-css-writing-modes-3)

<a id="ref-for-propdef-all②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dae18981"></a> For example, if an author specifies [all: initial](#propdef-all) on an element, it will block all inheritance and reset all properties, as if no rules appeared in the author, user, or user-agent levels of the cascade.
>
> <a id="ref-for-propdef-display①"></a>
>
> This can be useful for the root element of a "widget" included in a page, which does not wish to inherit the styles of the outer page. Note, however, that any "default" style applied to that element (such as, e.g. [display: block](https://www.w3.org/TR/CSS2/visuren.html#propdef-display) from the UA style sheet on block elements such as `<div>`) will also be blown away.

## <a id="value-stages"></a>4.  Value Processing

<a id="ref-for-flat-tree"></a>

Once a user agent has parsed a document and constructed a document tree, it must assign, to every element in the [flat tree](https://drafts.csswg.org/css-scoping-1/#flat-tree), and correspondingly to every box in the formatting structure, a value to every property that applies to the target media type.

The final value of a CSS property for a given element or box is the result of a multi-step calculation:

1.  <a id="ref-for-declared-value"></a>

    First, all the [declared values](#declared-value) applied to an element are collected, for each property on each element. There may be zero or many <a id="ref-for-declared-value①"></a>declared values applied to the element.

2.  <a id="ref-for-cascaded-value"></a>

    Cascading yields the [cascaded value](#cascaded-value). There is at most one <a id="ref-for-cascaded-value①"></a>cascaded value per property per element.

3.  <a id="ref-for-specified-value①"></a>

    Defaulting yields the [specified value](#specified-value). Every element has exactly one <a id="ref-for-specified-value②"></a>specified value per property.

4.  <a id="ref-for-computed-value"></a>

    Resolving value dependencies yields the [computed value](#computed-value). Every element has exactly one <a id="ref-for-computed-value①"></a>computed value per property.

5.  <a id="ref-for-used-value"></a>

    Formatting the document yields the [used value](#used-value). An element only has a <a id="ref-for-used-value①"></a>used value for a given property if that property applies to the element.

6.  <a id="ref-for-used-value②"></a>

    <a id="ref-for-actual-value"></a>

    Finally, the used value is transformed to the [actual value](#actual-value) based on constraints of the display environment. As with the [used value](#used-value), there may or may not be an <a id="ref-for-actual-value①"></a>actual value for a given property on an element.

<a id="ref-for-connected"></a>

<a id="ref-for-flat-tree①"></a>

<a id="ref-for-declared-value②"></a>

<a id="ref-for-cascaded-value②"></a>

<a id="ref-for-specified-value③"></a>

<a id="ref-for-computed-value②"></a>

<a id="ref-for-used-value③"></a>

<a id="ref-for-actual-value②"></a>

Elements that are not [connected](https://dom.spec.whatwg.org/#connected) or are not part of the document’s [flattened element tree](https://drafts.csswg.org/css-scoping-1/#flat-tree) do not participate in CSS value processing, and do not have [declared](#declared-value), [cascaded](#cascaded-value), [specified](#specified-value), [computed](#computed-value), [used](#used-value), or [actual](#actual-value) values, even if they potentially have style declarations assigned to them (for example, by a `style` attribute).

### <a id="declared"></a>4.1.  Declared Values

Each property declaration [applied to an element](#filtering) contributes a <a id="declared-value"></a>declared value for that property associated with the element. See [Filtering Declarations](#filtering) for details.

<a id="ref-for-cascade①"></a>

These values are then processed by the [cascade](#cascade) to choose a single “winning value”.

#### <a id="value-aliasing"></a>4.1.1.  Value Aliasing

<a id="ref-for-declared-value③"></a>

<a id="ref-for-vendor-prefix"></a>

Some property values have <a id="css-legacy-value-alias"></a>legacy value aliases: at parse time, the legacy syntax is converted into the new syntax, resulting in a [declared value](#declared-value) different from the parsed input. These aliases are typically used for handling legacy compatibility requirements, such as converting [vendor-prefixed](https://www.w3.org/TR/css-2021/#vendor-prefix) values to their standard equivalents.

### <a id="cascaded"></a>4.2.  Cascaded Values

<a id="ref-for-cascade①⑤"></a>

<a id="ref-for-declared-value④"></a>

<a id="ref-for-output-of-the-cascade"></a>

<a id="ref-for-cascaded-value③"></a>

The <a id="cascaded-value"></a>cascaded value represents the result of [the cascade](#cascade): it is the [declared value](#declared-value) that wins the cascade (is sorted first in the [output of the cascade](#output-of-the-cascade)). If the <a id="ref-for-output-of-the-cascade①"></a>output of the cascade is an empty list, there is no [cascaded value](#cascaded-value).

### <a id="specified"></a>4.3.  Specified Values

<a id="ref-for-cascaded-value④"></a>

<a id="ref-for-specified-value④"></a>

The <a id="specified-value"></a>specified value is the value of a given property that the style sheet authors intended for that element. It is the result of putting the [cascaded value](#cascaded-value) through the [defaulting](#defaulting) processes, guaranteeing that a [specified value](#specified-value) exists for every property on every element.

<a id="ref-for-specified-value⑤"></a>

<a id="ref-for-cascaded-value⑤"></a>

<a id="ref-for-css-wide-keywords"></a>

In many cases, the [specified value](#specified-value) is the [cascaded value](#cascaded-value). However, if there is no <a id="ref-for-cascaded-value⑥"></a>cascaded value at all, the <a id="ref-for-specified-value⑥"></a>specified value is [defaulted](#defaulting). The [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) are handled specially when they are the <a id="ref-for-cascaded-value⑦"></a>cascaded value of a property, setting the <a id="ref-for-specified-value⑦"></a>specified value as required by that keyword, see [§ 7.3 Explicit Defaulting](#defaulting-keywords).

### <a id="computed"></a>4.4.  Computed Values

<a id="ref-for-specified-value⑧"></a>

<a id="ref-for-inheritance①"></a>

The <a id="computed-value"></a>computed value is the result of resolving the [specified value](#specified-value) as defined in the “Computed Value” line of the property definition table, generally absolutizing it in preparation for [inheritance](#inheritance).

<a id="ref-for-computed-value③"></a>

<a id="ref-for-inheritance②"></a>

<a id="ref-for-dom-window-getcomputedstyle"></a>

<a id="ref-for-used-value④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [computed value](#computed-value) is the value that is transferred from parent to child during [inheritance](#inheritance). For historical reasons, it is not necessarily the value returned by the <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code> function, which sometimes returns [used values](#used-value). [\[CSSOM\]](#biblio-cssom) Furthermore, the <a id="ref-for-computed-value④"></a>computed value is an abstract data representation: their definitions reflect that data representation, not how that data is serialized. For example, serialization rules often allow omitting certain values which are implied during parsing; but those values are nonetheless part of the <a id="ref-for-computed-value⑤"></a>computed value.

<a id="ref-for-specified-value⑨"></a>

<a id="ref-for-valdef-color-red①"></a>

<a id="ref-for-valdef-justify-self-auto"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-29410c97"></a> A [specified value](#specified-value) can be either absolute (i.e., not relative to another value, as in [red](https://www.w3.org/TR/css-color-4/#valdef-color-red) or 2mm) or relative (i.e., relative to another value, as in [auto](https://www.w3.org/TR/css-align-3/#valdef-justify-self-auto), 2em). Computing a relative value generally absolutizes it:
>
> - <a id="ref-for-valdef-length-vw"></a>
>
>   <a id="ref-for-valdef-length-vh"></a>
>
>   <a id="ref-for-ex"></a>
>
>   <a id="ref-for-em"></a>
>
>   values with relative units ([em](https://www.w3.org/TR/css-values-4/#em), [ex](https://www.w3.org/TR/css-values-3/#ex), [vh](https://www.w3.org/TR/css-values-4/#valdef-length-vh), [vw](https://www.w3.org/TR/css-values-4/#valdef-length-vw)) must be made absolute by multiplying with the appropriate reference size
>
> - <a id="ref-for-valdef-font-weight-bolder"></a>
>
>   certain keywords (e.g., smaller, [bolder](https://www.w3.org/TR/css-fonts-4/#valdef-font-weight-bolder)) must be replaced according to their definitions
>
> - percentages on some properties must be multiplied by a reference value (defined by the property)
>
> - valid relative URLs must be resolved to become absolute.
>
> See examples (f), (g) and (h) in the [table below](#stages-examples).

<a id="ref-for-computed-value⑥"></a>

<a id="ref-for-specified-value①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In general, the [computed value](#computed-value) resolves the [specified value](#specified-value) as far as possible without laying out the document or performing other expensive or hard-to-parallelize operations, such as resolving network requests or retrieving values other than from the element and its parent.

<a id="ref-for-computed-value⑦"></a>

<a id="ref-for-apply"></a>

The [computed value](#computed-value) exists even when the property does not apply. However, some properties may change how they determine the <a id="ref-for-computed-value⑧"></a>computed value based on whether the property [applies to](#apply) the element.

### <a id="used"></a>4.5.  Used Values

<a id="ref-for-computed-value⑨"></a>

The <a id="used-value"></a>used value is the result of taking the [computed value](#computed-value) and completing any remaining calculations to make it the absolute theoretical value used in the formatting of the document.

<a id="ref-for-propdef-width"></a>

<a id="ref-for-computed-value①⓪"></a>

<a id="ref-for-valdef-justify-self-auto①"></a>

<a id="ref-for-used-value⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-aaa6b50b"></a> For example, a declaration of [width: auto](https://www.w3.org/TR/css-sizing-3/#propdef-width) can’t be resolved into a length without knowing the layout of the element’s ancestors, so the [computed value](#computed-value) is [auto](https://www.w3.org/TR/css-align-3/#valdef-justify-self-auto), while the [used value](#used-value) is an absolute length, such as 100px. [\[CSS2\]](#biblio-css2)

<a id="ref-for-propdef-break-before④"></a>

<a id="ref-for-valdef-justify-self-auto②"></a>

<a id="ref-for-valdef-break-before-page"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-aac2d669"></a> As another example, a `<div>` might have a computed [break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before) value of [auto](https://www.w3.org/TR/css-align-3/#valdef-justify-self-auto), but acquire a used <a id="ref-for-propdef-break-before⑤"></a>break-before value of [page](https://www.w3.org/TR/css-break-4/#valdef-break-before-page) by propagation from its first child. [\[css-break-3\]](#biblio-css-break-3)

<a id="ref-for-apply①"></a>

<a id="ref-for-used-value⑥"></a>

If a property does not [apply to](#apply) this element or box type then it has no [used value](#used-value) for that property.

<a id="ref-for-propdef-flex"></a>

<a id="ref-for-used-value⑦"></a>

<a id="ref-for-flex-item"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-19a1092f"></a> For example, the [flex](https://www.w3.org/TR/css-flexbox-1/#propdef-flex) property has no [used value](#used-value) on elements that aren’t [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item).

#### <a id="applies-to"></a>4.5.1.  Applicable Properties

If a property does not <a id="apply"></a>apply to an element or box type—as noted in its “Applies to” line—this means it does not directly take effect on that type of box or element.

<a id="ref-for-computed-value①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A property that does not apply can still have <em>indirect</em> formatting effects if its computed value affects the computation of other properties that do apply; and of course its [computed value](#computed-value), which always exists, can still inherit to descendants and take effect on them.

<a id="ref-for-propdef-writing-mode"></a>

<a id="ref-for-propdef-text-orientation"></a>

<a id="ref-for-ch"></a>

<a id="ref-for-length-value"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9b82d3cf"></a> Even though [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) do not apply to table rows (they do not affect how the table row or its children are laid out), setting them on such boxes will still affect the calculation of font relative units such as [ch](https://www.w3.org/TR/css-values-4/#ch), and thus possibly any property that takes a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value).

<a id="ref-for-propdef-text-transform"></a>

<a id="ref-for-the-p-element"></a>

<a id="ref-for-propdef-display②"></a>

<a id="ref-for-inline-box"></a>

<a id="ref-for-root-inline-box"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c1c9d619"></a> Setting [text-transform](https://www.w3.org/TR/css-text-3/#propdef-text-transform) on an HTML <code><a href="https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element">p</a></code> element (which is [display: block](https://www.w3.org/TR/CSS2/visuren.html#propdef-display) by default) will have an effect, even though <a id="ref-for-propdef-text-transform①"></a>text-transform only applies to [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box), because the property inherits into the paragraph’s anonymous [root inline box](https://www.w3.org/TR/css-inline-3/#root-inline-box) and applies to the text it contains.

<a id="ref-for-display-type"></a>

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-selectordef-before"></a>

<a id="ref-for-selectordef-after"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A property defined to apply to “all elements” applies to all elements and [display types](https://www.w3.org/TR/css-display-3/#display-type), but not necessarily to all [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) types, since pseudo-elements often have their own specific rendering models or other restrictions. The [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) and [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudo-elements, however, are defined to generate boxes almost exactly like normal elements and are therefore defined accept all properties that apply to “all elements”. See [\[CSS-PSEUDO-4\]](#biblio-css-pseudo-4) for more information about <a id="ref-for-pseudo-element①"></a>pseudo-elements.

### <a id="actual"></a>4.6.  Actual Values

<a id="ref-for-used-value⑧"></a>

<a id="ref-for-used-value⑨"></a>

<a id="ref-for-propdef-font-size-adjust"></a>

A [used value](#used-value) is in principle ready to be used, but a user agent may not be able to make use of the value in a given environment. For example, a user agent may only be able to render borders with integer pixel widths and may therefore have to approximate the [used](#used-value) width. Also, the font size of an element may need adjustment based on the availability of fonts or the value of the [font-size-adjust](https://www.w3.org/TR/css-fonts-5/#propdef-font-size-adjust) property. The <a id="actual-value"></a>actual value is the used value after any such adjustments have been made.

<a id="ref-for-propdef-page-break-after"></a>

<a id="ref-for-propdef-orphans"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: By probing the actual values of elements, much can be learned about how the document is laid out. However, not all information is recorded in the actual values. For example, the actual value of the [page-break-after](https://www.w3.org/TR/CSS2/page.html#propdef-page-break-after) property does not reflect whether there is a page break or not after the element. Similarly, the actual value of [orphans](https://www.w3.org/TR/css-break-3/#propdef-orphans) does not reflect how many orphan lines there is in a certain element. See examples (j) and (k) in the [table below](#stages-examples).

### <a id="stages-examples"></a>4.7.  Examples

|       | Property            | Winning declaration | Cascaded value | Specified value         | Computed value | Used value | Actual value |
|-------|---------------------|---------------------|----------------|-------------------------|----------------|------------|--------------|
| \(a\) | <strong><span><a id="ref-for-propdef-text-align"></a></span><a href="https://www.w3.org/TR/css-text-3/#propdef-text-align">text-align</a> &#xA;      </strong> | <code>text-align:&#x20;left</code>   | left           | left                    | left           | left       | left         |
| \(b\) | <strong><span><a id="ref-for-propdef-border-left-width"></a></span><span><a id="ref-for-propdef-border-bottom-width"></a></span><span><a id="ref-for-propdef-border-right-width"></a></span><span><a id="ref-for-propdef-border-top-width"></a></span><a href="https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width">border-top-width</a>, <a href="https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right-width">border-right-width</a>, <a href="https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-width">border-bottom-width</a>, <a href="https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-width">border-left-width</a> &#xA;      </strong> | <code>border-width:&#x20;inherit</code>   | inherit        | 4.2px                   | 4.2px          | 4.2px      | 4px          |
| \(c\) | <strong><span><a id="ref-for-propdef-width①"></a></span><a href="https://www.w3.org/TR/css-sizing-3/#propdef-width">width</a> &#xA;      </strong> | (none)              | (none)         | auto (initial value)    | auto           | 120px      | 120px        |
| \(d\) | <strong><span><a id="ref-for-propdef-list-style-position"></a></span><a href="https://www.w3.org/TR/css-lists-3/#propdef-list-style-position">list-style-position</a> &#xA;      </strong> | <code>list-style-position:&#x20;inherit</code>   | inherit        | inside                  | inside         | inside     | inside       |
| \(e\) | <strong><span><a id="ref-for-propdef-list-style-position①"></a></span><a href="https://www.w3.org/TR/css-lists-3/#propdef-list-style-position">list-style-position</a> &#xA;      </strong> | <code>list-style-position:&#x20;initial</code>   | initial        | outside (initial value) | outside        | outside    | outside      |
| \(f\) | <strong><span><a id="ref-for-descdef-font-face-font-size②"></a></span><a href="https://www.w3.org/TR/css-fonts-5/#descdef-font-face-font-size">font-size</a> &#xA;      </strong> | <code>font-size:&#x20;1.2em</code>   | 1.2em          | 1.2em                   | 14.1px         | 14.1px     | 14px         |
| \(g\) | <strong><span><a id="ref-for-propdef-width②"></a></span><a href="https://www.w3.org/TR/css-sizing-3/#propdef-width">width</a> &#xA;      </strong> | <code>width:&#x20;80%</code>   | 80%            | 80%                     | 80%            | 354.2px    | 354px        |
| \(h\) | <strong><span><a id="ref-for-propdef-width③"></a></span><a href="https://www.w3.org/TR/css-sizing-3/#propdef-width">width</a> &#xA;      </strong> | <code>width:&#x20;auto</code>   | auto           | auto                    | auto           | 134px      | 134px        |
| \(i\) | <strong><span><a id="ref-for-propdef-height"></a></span><a href="https://www.w3.org/TR/css-sizing-3/#propdef-height">height</a> &#xA;      </strong> | <code>height:&#x20;auto</code>   | auto           | auto                    | auto           | 176px      | 176px        |
| \(j\) | <strong><span><a id="ref-for-propdef-page-break-after①"></a></span><a href="https://www.w3.org/TR/CSS2/page.html#propdef-page-break-after">page-break-after</a> &#xA;      </strong> | (none)              | (none)         | auto (initial value)    | auto           | auto       | auto         |
| \(k\) | <strong><span><a id="ref-for-propdef-orphans①"></a></span><a href="https://www.w3.org/TR/css-break-3/#propdef-orphans">orphans</a> &#xA;      </strong> | <code>orphans:&#x20;3</code>   | 3              | 3                       | 3              | 3          | 3            |

Examples of CSS Value Computation

## <a id="filtering"></a>5.  Filtering

<a id="ref-for-declared-value⑤"></a>

In order to find the [declared values](#declared-value), implementations must first identify all declarations that apply to each element. A declaration applies to an element if:

- It belongs to a style sheet that currently applies to this document.
- It is not qualified by a conditional rule [\[CSS-CONDITIONAL-3\]](#biblio-css-conditional-3) with a false condition.
- It belongs to a style rule whose selector matches the element. [\[SELECT\]](#biblio-select) (Taking [scoping](https://www.w3.org/TR/selectors-4/#scoping) into account, if necessary.)
- It is syntactically valid: the declaration’s property is a known property name, and the declaration’s value matches the syntax for that property.

<a id="ref-for-declared-value⑥"></a>

<a id="ref-for-cascade②"></a>

The values of the declarations that apply form, for each property on each element, a list of [declared values](#declared-value). The next section, the [cascade](#cascade), prioritizes these lists.

## <a id="cascading"></a>6.  Cascading

<a id="ref-for-declared-value⑦"></a>

<a id="ref-for-cascaded-value⑧"></a>

The <a id="cascade"></a>cascade takes an unordered list of [declared values](#declared-value) for a given property on a given element, sorts them by their declaration’s precedence as determined below, and outputs a single [cascaded value](#cascaded-value).

### <a id="cascade-sort"></a>6.1.  Cascade Sorting Order

The cascade sorts declarations according to the following criteria, in descending order of priority:

<a id="cascade-origin"></a>Origin and Importance  
<a id="ref-for-important"></a>

<a id="ref-for-origin②"></a>

The [origin](#origin) of a declaration is based on where it comes from and its [importance](#important) is whether or not it is declared with !important (see [below](#importance)). The precedence of the various <a id="ref-for-origin③"></a>origins is, in descending order:

1.  Transition declarations [\[css-transitions-1\]](#biblio-css-transitions-1)

2.  <a id="ref-for-cascade-origin-ua"></a>

    <a id="ref-for-important①"></a>

    [Important](#important) [user agent](#cascade-origin-ua) declarations

3.  <a id="ref-for-cascade-origin-user"></a>

    <a id="ref-for-important②"></a>

    [Important](#important) [user](#cascade-origin-user) declarations

4.  <a id="ref-for-cascade-origin-author"></a>

    <a id="ref-for-important③"></a>

    [Important](#important) [author](#cascade-origin-author) declarations

5.  Animation declarations [\[css-animations-1\]](#biblio-css-animations-1)

6.  <a id="ref-for-cascade-origin-author①"></a>

    <a id="ref-for-normal"></a>

    [Normal](#normal) [author](#cascade-origin-author) declarations

7.  <a id="ref-for-cascade-origin-user①"></a>

    <a id="ref-for-normal①"></a>

    [Normal](#normal) [user](#cascade-origin-user) declarations

8.  <a id="ref-for-cascade-origin-ua①"></a>

    <a id="ref-for-normal②"></a>

    [Normal](#normal) [user agent](#cascade-origin-ua) declarations

<a id="ref-for-origin④"></a>

Declarations from [origins](#origin) earlier in this list win over declarations from later <a id="ref-for-origin⑤"></a>origins.

<a id="cascade-context"></a>Context  
<a id="ref-for-shadow-tree"></a>

<a id="ref-for-tree-context"></a>

A document language can provide for blending declarations sourced from different <a id="encapsulation-contexts"></a>encapsulation contexts, such as the nested [tree contexts](https://drafts.csswg.org/css-scoping-1/#tree-context) of [shadow trees](https://www.w3.org/TR/css-scoping-1/#shadow-tree) in the [\[DOM\]](#biblio-dom).

<a id="ref-for-encapsulation-contexts"></a>

<a id="ref-for-normal③"></a>

<a id="ref-for-important④"></a>

<a id="ref-for-tree-context①"></a>

<a id="ref-for-concept-shadow-including-tree-order"></a>

When comparing two declarations that are sourced from different [encapsulation contexts](#encapsulation-contexts), then for [normal](#normal) rules the declaration from the outer context wins, and for [important](#important) rules the declaration from the inner context wins. For this purpose, [\[DOM\]](#biblio-dom) [tree contexts](https://drafts.csswg.org/css-scoping-1/#tree-context) are considered to be nested in [shadow-including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order).

<a id="ref-for-normal④"></a>

<a id="ref-for-encapsulation-contexts①"></a>

<a id="ref-for-important⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This effectively means that [normal](#normal) declarations belonging to an [encapsulation context](#encapsulation-contexts) can set defaults that are easily overridden by the outer context, while [important](#important) declarations belonging to an <a id="ref-for-encapsulation-contexts②"></a>encapsulation context can enforce requirements that cannot be overridden by the outer context.

<a id="cascade-specificity"></a>Specificity  
The [Selectors module](https://www.w3.org/TR/selectors/#specificity) [\[SELECT\]](#biblio-select) describes how to compute the specificity of a selector. Each declaration has the same specificity as the style rule it appears in. For the purpose of this step, declarations that do not belong to a style rule (such as the [contents of a style attribute](https://www.w3.org/TR/css-style-attr/#interpret)) are considered to have a specificity higher than any selector. The declaration with the highest specificity wins.

<a id="cascade-order"></a>Order of Appearance  
The last declaration in document order wins. For this purpose:

- <a id="ref-for-at-ruledef-import①③"></a>

  Declarations from [imported style sheets](#at-ruledef-import) are ordered as if their style sheets were substituted in place of the <a id="ref-for-at-ruledef-import①④"></a>@import rule.

- Declarations from style sheets independently linked by the originating document are treated as if they were concatenated in linking order, as determined by the host document language.

- Declarations from style attributes are ordered according to the document order of the element the style attribute appears on, and are all placed after any style sheets.

<a id="ref-for-declared-value⑧"></a>

The <a id="output-of-the-cascade"></a>output of the cascade is a (potentially empty) sorted list of [declared values](#declared-value) for each property on each element.

### <a id="cascading-origins"></a>6.2.  Cascading Origins

<a id="ref-for-origin⑥"></a>

Each style rule has a <a id="origin"></a>cascade origin, which determines where it enters the cascade. CSS defines three core [origins](#origin):

<a id="cascade-origin-author"></a>Author Origin  
The author specifies style sheets for a source document according to the conventions of the document language. For instance, in HTML, style sheets may be included in the document or linked externally.

<a id="cascade-origin-user"></a>User Origin  
The user may be able to specify style information for a particular document. For example, the user may specify a file that contains a style sheet or the user agent may provide an interface that generates a user style sheet (or behaves as if it did).

<a id="cascade-origin-ua"></a>User-Agent Origin  
Conforming user agents must apply a default style sheet (or behave as if they did). A user agent’s default style sheet should present the elements of the document language in ways that satisfy general presentation expectations for the document language (e.g., for visual browsers, the EM element in HTML is presented using an italic font). See e.g. the [HTML user agent style sheet](https://html.spec.whatwg.org/multipage/rendering.html#the-css-user-agent-style-sheet-and-presentational-hints). [\[HTML\]](#biblio-html)

<a id="ref-for-origin⑦"></a>

Extensions to CSS define the following additional [origins](#origin):

<a id="cascade-origin-animation"></a>Animation Origin  
CSS Animations [\[css-animations-1\]](#biblio-css-animations-1) generate “virtual” rules representing their effects when running.

<a id="cascade-origin-transition"></a>Transition Origin  
Like CSS Animations, CSS Transitions [\[css-transitions-1\]](#biblio-css-transitions-1) generate “virtual” rules representing their effects when running.

### <a id="importance"></a>6.3.  Important Declarations: the !important annotation

<a id="ref-for-important⑥"></a>

CSS attempts to create a balance of power between author and user style sheets. By default, rules in an author’s style sheet override those in a user’s style sheet, which override those in the user-agent’s default style sheet. To balance this, a declaration can be marked [important](#important), which increases its weight in the cascade and inverts the order of precedence.

<a id="ref-for-important⑦"></a>

A declaration is <a id="important"></a>important if it has a !important annotation as defined by [\[css-syntax-3\]](#biblio-css-syntax-3), i.e. if the last two (non-whitespace, non-comment) tokens in its value are the delimiter token ! followed by the identifier token important. All other declarations are <a id="normal"></a>normal (non-[important](#important)).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-813b5c64"></a>
>
> ```css
> [hidden] { display: none !important; }
> ```
<a id="ref-for-important⑧"></a>

<a id="ref-for-normal⑤"></a>

<a id="ref-for-cascade-origin-user②"></a>

<a id="ref-for-cascade-origin-author②"></a>

An [important](#important) declaration takes precedence over a [normal](#normal) declaration. Author and user style sheets may contain <a id="ref-for-important⑨"></a>important declarations, with [user-origin](#cascade-origin-user) <a id="ref-for-important①⓪"></a>important declarations overriding [author-origin](#cascade-origin-author) <a id="ref-for-important①①"></a>important declarations. This CSS feature improves accessibility of documents by giving users with special requirements (large fonts, color combinations, etc.) control over presentation.

<a id="ref-for-important①②"></a>

[Important](#important) declarations from all origins take precedence over animations. This allows authors to override animated values in important cases. (Animated values normally override all other rules.) [\[css-animations-1\]](#biblio-css-animations-1)

<a id="ref-for-cascade-origin-ua②"></a>

<a id="ref-for-important①③"></a>

<a id="ref-for-cascade-origin-author③"></a>

<a id="ref-for-cascade-origin-user③"></a>

[User-agent style sheets](#cascade-origin-ua) may also contain [important](#important) declarations. These override all [author](#cascade-origin-author) and [user](#cascade-origin-user) declarations.

<a id="ref-for-shorthand-property①①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b189d121"></a> The first rule in the user’s style sheet in the following example contains an !important declaration, which overrides the corresponding declaration in the author’s style sheet. The declaration in the second rule will also win due to being marked !important. However, the third declaration in the user’s style sheet is not !important and will therefore lose to the second rule in the author’s style sheet (which happens to set style on a [shorthand](#shorthand-property) property). Also, the third author rule will lose to the second author rule since the second declaration is !important. This shows that !important declarations have a function also within author style sheets.
>
> ```css
> /* From the user’s style sheet */
> p { text-indent: 1em !important }
> p { font-style: italic !important }
> p { font-size: 18pt }
> 
> /* From the author’s style sheet */
> p { text-indent: 1.5em !important }
> p { font: normal 12pt sans-serif !important }
> p { font-size: 24pt }
> ```
>
> | Property            | Winning value                                                                                     |
> |---------------------|---------------------------------------------------------------------------------------------------|
> | <strong><span><a id="ref-for-propdef-text-indent"></a></span><a href="https://www.w3.org/TR/css-text-3/#propdef-text-indent">text-indent</a> &#xA;       </strong> | 1em                                                                                               |
> | <strong><span><a id="ref-for-propdef-font-style①"></a></span><a href="https://www.w3.org/TR/css-fonts-4/#propdef-font-style">font-style</a> &#xA;       </strong> | <a id="ref-for-valdef-font-style-italic"></a>[italic](https://www.w3.org/TR/css-fonts-4/#valdef-font-style-italic)          |
> | <strong><span><a id="ref-for-descdef-font-face-font-size③"></a></span><a href="https://www.w3.org/TR/css-fonts-5/#descdef-font-face-font-size">font-size</a> &#xA;       </strong> | 12pt                                                                                              |
> | <strong><span><a id="ref-for-propdef-font-family①"></a></span><a href="https://www.w3.org/TR/css-fonts-4/#propdef-font-family">font-family</a> &#xA;       </strong> | <a id="ref-for-valdef-font-family-sans-serif"></a>[sans-serif](https://www.w3.org/TR/css-fonts-4/#valdef-font-family-sans-serif) |

### <a id="preshint"></a>6.4.  Precedence of Non-CSS Presentational Hints

<a id="ref-for-the-s-element"></a>

<a id="ref-for-cascade-origin-ua③"></a>

<a id="ref-for-cascade-origin-user④"></a>

<a id="ref-for-cascade-origin-author④"></a>

<a id="ref-for-cascade③"></a>

<a id="ref-for-author-presentational-hint-origin"></a>

<a id="ref-for-origin⑧"></a>

<a id="ref-for-valdef-all-revert①"></a>

The UA may choose to honor presentational hints in a source document’s markup, for example the `bgcolor` attribute or <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-s-element">s</a></code> element in [\[HTML\]](#biblio-html). All document language-based styling must be translated to corresponding CSS rules and enter the cascade as rules in either the [UA-origin](#cascade-origin-ua) or a special-purpose <a id="author-presentational-hint-origin"></a>author presentational hint origin between the regular [user origin](#cascade-origin-user) and the [author origin](#cascade-origin-author). For the purpose of [cascading](#cascade) this [author presentational hint origin](#author-presentational-hint-origin) is treated as an independent [origin](#origin), but for the purpose of the [revert](#valdef-all-revert) keyword it is considered part of the <a id="ref-for-cascade-origin-author⑤"></a>author origin.

<a id="ref-for-cascade④"></a>

<a id="ref-for-cascade-origin-ua④"></a>

<a id="ref-for-cascade-origin-author⑥"></a>

A document language may define whether such a presentational hint enters the [cascade](#cascade) as [UA-origin](#cascade-origin-ua) or [author-origin](#cascade-origin-author); if so, the UA must behave accordingly. For example, [\[SVG11\]](#biblio-svg11) maps its presentation attributes into the <a id="ref-for-cascade-origin-author⑦"></a>author origin.

<a id="ref-for-cascade⑤"></a>

<a id="ref-for-cascade-origin-ua⑤"></a>

<a id="ref-for-cascade-origin-author⑧"></a>

<a id="ref-for-cascade-origin-user⑤"></a>

<a id="ref-for-author-presentational-hint-origin①"></a>

<a id="ref-for-important①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Presentational hints entering the [cascade](#cascade) as [UA-origin](#cascade-origin-ua) rules can be overridden by [author-origin](#cascade-origin-author) or [user-origin](#cascade-origin-user) styles. Presentational hints entering the cascade as [author presentational hint origin](#author-presentational-hint-origin) rules can be overridden by <a id="ref-for-cascade-origin-author⑨"></a>author-origin styles, but not by non-[important](#important) <a id="ref-for-cascade-origin-user⑥"></a>user-origin styles. Host languages should choose the appropriate origin for presentational hints with these considerations in mind.

## <a id="defaulting"></a>7.  Defaulting

<a id="ref-for-cascade⑥"></a>

<a id="ref-for-specified-value①①"></a>

<a id="ref-for-inherited-property"></a>

<a id="ref-for-inheritance③"></a>

<a id="ref-for-initial-value②"></a>

<a id="ref-for-valdef-all-inherit"></a>

<a id="ref-for-valdef-all-initial"></a>

When the [cascade](#cascade) does not result in a value, the [specified value](#specified-value) must be found some other way. [Inherited properties](#inherited-property) draw their defaults from their parent element through [inheritance](#inheritance); all other properties take their [initial value](#initial-value). Authors can explicitly request inheritance or initialization via the [inherit](#valdef-all-inherit) and [initial](#valdef-all-initial) keywords.

### <a id="initial-values"></a>7.1.  Initial Values

<a id="ref-for-inherited-property①"></a>

<a id="ref-for-cascade⑦"></a>

<a id="ref-for-specified-value①②"></a>

<a id="ref-for-initial-value③"></a>

Each property has an <a id="initial-value"></a>initial value, defined in the property’s definition table. If the property is not an [inherited property](#inherited-property), and the [cascade](#cascade) does not result in a value, then the [specified value](#specified-value) of the property is its [initial value](#initial-value).

### <a id="inheriting"></a>7.2.  Inheritance

<a id="ref-for-computed-value①②"></a>

<a id="ref-for-inherited-value"></a>

<a id="ref-for-initial-value④"></a>

<a id="inheritance"></a>Inheritance propagates property values from parent elements to their children. The <a id="inherited-value"></a>inherited value of a property on an element is the [computed value](#computed-value) of the property on the element’s parent element. For the root element, which has no parent element, the [inherited value](#inherited-value) is the [initial value](#initial-value) of the property.

<a id="ref-for-flat-tree②"></a>

<a id="ref-for-the-slot-element"></a>

<a id="ref-for-concept-light-tree"></a>

<a id="ref-for-pseudo-element②"></a>

For a [\[DOM\]](#biblio-dom) tree with shadows, inheritance operates on the [flattened element tree](https://drafts.csswg.org/css-scoping-1/#flat-tree). <strong data-conversion-semantic="note">Note:</strong> This means that slotted elements inherit from the <code><a href="https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element">slot</a></code> they’re assigned to, rather than directly from their [light tree](https://dom.spec.whatwg.org/#concept-light-tree) parent. [Pseudo-elements](https://www.w3.org/TR/selectors-4/#pseudo-element) inherit according to the fictional tag sequence described for each <a id="ref-for-pseudo-element③"></a>pseudo-element. [\[CSS-PSEUDO-4\]](#biblio-css-pseudo-4)

<a id="ref-for-cascade⑧"></a>

<a id="ref-for-inheritance④"></a>

Some properties are <a id="inherited-property"></a>inherited properties, as defined in their property definition table. This means that, unless the [cascade](#cascade) results in a value, the value will be determined by [inheritance](#inheritance).

<a id="ref-for-valdef-all-inherit①"></a>

A property can also be explicitly inherited. See the [inherit](#valdef-all-inherit) keyword.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Inheritance follows the document tree and is not intercepted by [anonymous boxes](https://www.w3.org/TR/CSS2/visuren.html#box-gen), or otherwise affected by manipulations of the box tree.

### <a id="defaulting-keywords"></a>7.3.  Explicit Defaulting

Several CSS-wide property values are defined below; declaring a property to have these values explicitly specifies a particular defaulting behavior. As specified in [CSS Values and Units](https://www.w3.org/TR/css-values/#common-keywords) [\[css-values-3\]](#biblio-css-values-3), all CSS properties can accept these values.

<a id="ref-for-valdef-all-initial①"></a>

#### <a id="initial"></a>7.3.1.  Resetting a Property: the [initial](#valdef-all-initial) keyword

<a id="ref-for-cascaded-value⑨"></a>

<a id="ref-for-specified-value①③"></a>

<a id="ref-for-initial-value⑤"></a>

If the [cascaded value](#cascaded-value) of a property is the <a id="valdef-all-initial"></a>initial keyword, the property’s [specified value](#specified-value) is its [initial value](#initial-value).

<a id="ref-for-valdef-all-inherit②"></a>

#### <a id="inherit"></a>7.3.2.  Explicit Inheritance: the [inherit](#valdef-all-inherit) keyword

<a id="ref-for-cascaded-value①⓪"></a>

<a id="ref-for-specified-value①④"></a>

<a id="ref-for-computed-value①③"></a>

<a id="ref-for-inherited-value①"></a>

If the [cascaded value](#cascaded-value) of a property is the <a id="valdef-all-inherit"></a>inherit keyword, the property’s [specified](#specified-value) and [computed values](#computed-value) are the [inherited value](#inherited-value).

<a id="ref-for-valdef-all-unset"></a>

#### <a id="inherit-initial"></a>7.3.3.  Erasing All Declarations: the [unset](#valdef-all-unset) keyword

<a id="ref-for-cascaded-value①①"></a>

<a id="ref-for-valdef-all-inherit③"></a>

<a id="ref-for-valdef-all-initial②"></a>

<a id="ref-for-declared-value⑨"></a>

<a id="ref-for-cascade⑨"></a>

<a id="ref-for-shorthand-property①②"></a>

If the [cascaded value](#cascaded-value) of a property is the <a id="valdef-all-unset"></a>unset keyword, then if it is an inherited property, this is treated as [inherit](#valdef-all-inherit), and if it is not, this is treated as [initial](#valdef-all-initial). This keyword effectively erases all [declared values](#declared-value) occurring earlier in the [cascade](#cascade), correctly inheriting or not as appropriate for the property (or all longhands of a [shorthand](#shorthand-property)).

<a id="ref-for-valdef-all-revert②"></a>

#### <a id="default"></a>7.3.4.  Rolling Back Cascade Origins: the [revert](#valdef-all-revert) keyword

<a id="ref-for-cascaded-value①②"></a>

<a id="ref-for-origin⑨"></a>

If the [cascaded value](#cascaded-value) of a property is the <a id="valdef-all-revert"></a>revert keyword, the behavior depends on the [cascade origin](#origin) to which the declaration belongs:

<a id="ref-for-cascade-origin-ua⑥"></a>

[user-agent origin](#cascade-origin-ua)

<a id="ref-for-valdef-all-unset①"></a>

Equivalent to [unset](#valdef-all-unset).

<a id="ref-for-cascade-origin-user⑦"></a>

[user origin](#cascade-origin-user)

<a id="ref-for-cascade-origin-user⑧"></a>

<a id="ref-for-cascade-origin-author①⓪"></a>

<a id="ref-for-specified-value①⑤"></a>

<a id="ref-for-cascaded-value①③"></a>

Rolls back the [cascaded value](#cascaded-value) to the user-agent level, so that the [specified value](#specified-value) is calculated as if no [author-origin](#cascade-origin-author) or [user-origin](#cascade-origin-user) rules were specified for this property on this element.

<a id="ref-for-cascade-origin-author①①"></a>

[author origin](#cascade-origin-author)

<a id="ref-for-origin①⓪"></a>

<a id="ref-for-valdef-all-revert③"></a>

<a id="ref-for-cascade-origin-author①②"></a>

<a id="ref-for-specified-value①⑥"></a>

<a id="ref-for-cascaded-value①④"></a>

Rolls back the [cascaded value](#cascaded-value) to the user level, so that the [specified value](#specified-value) is calculated as if no [author-origin](#cascade-origin-author) rules were specified for this property on this element. For the purpose of [revert](#valdef-all-revert), this origin includes the Animation [origin](#origin).

## <a id="changes"></a>8.  Changes

### <a id="changes-2021-10"></a>8.1.  Changes since the 15 Oct 2021 Working Draft

Non-trivial changes since the [15 October 2021 Working Draft](https://www.w3.org/TR/2021/WD-css-cascade-4-20211015/):

- Updated @import grammar for media queries and supports conditions

- Allowed functional notation parse-time aliases

- Defined fetching an @import, in terms of Fetch

- Added [§ 4.1.1 Value Aliasing](#value-aliasing) section. ([Issue 6193](https://github.com/w3c/csswg-drafts/issues/6193))

### <a id="changes-2018"></a>8.2.  Changes Since the 28 August 2018 Candidate Recommendation

Non-trivial changes since the [19 March 2021 Working Draft](https://www.w3.org/TR/2021/WD-css-cascade-4-20210319/) include:

- <a id="ref-for-encapsulation-contexts③"></a>

  <a id="ref-for-author-presentational-hint-origin②"></a>

  <a id="change-2021-preshint-origin"></a> Defined [author presentational hint origin](#author-presentational-hint-origin) to handle author-origin presentational hints, instead of relying on zero-specificity and source order, to correctly define their interaction with the [encapsulation context](#encapsulation-contexts) aspect of the cascade. ([Issue 66749](https://github.com/w3c/csswg-drafts/issues/6659))

Non-trivial changes since the [18 August 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-cascade-4-20200818/) include:

- <a id="ref-for-legacy-name-alias①"></a>

  <a id="change-2020-alias-subset"></a> Removed possibility of [legacy name aliases](#legacy-name-alias) to map subsets of the value space, since they are simple name aliases. ([Issue 4839](https://github.com/w3c/csswg-drafts/issues/4839))

- <a id="ref-for-apply②"></a>

  <a id="change-2020-applies-to"></a> Gave concept of [applies to](#apply) its own section and add some notes about implications. (Issues [1861](https://github.com/w3c/csswg-drafts/issues/1861) and [5565](https://github.com/w3c/csswg-drafts/issues/5565))

- <a id="ref-for-css-property①"></a>

  <a id="change-2020-properties"></a> Defined the term [property](#css-property). ([Issue 5633](https://github.com/w3c/csswg-drafts/issues/5633))

- <a id="change-2020-disconnected"></a> Defined value processing of elements that are not part of the tree. (Issue [1964](https://github.com/w3c/csswg-drafts/issues/1964) and [1548](https://github.com/w3c/csswg-drafts/issues/1548))

  > <a id="ref-for-connected①"></a>
  >
  > <a id="ref-for-flat-tree③"></a>
  >
  > <a id="ref-for-declared-value①⓪"></a>
  >
  > <a id="ref-for-cascaded-value①⑤"></a>
  >
  > <a id="ref-for-specified-value①⑦"></a>
  >
  > <a id="ref-for-computed-value①④"></a>
  >
  > <a id="ref-for-used-value①⓪"></a>
  >
  > <a id="ref-for-actual-value③"></a>
  >
  > <u>Elements that are not [connected](https://dom.spec.whatwg.org/#connected) or are not part of the document’s [flattened element tree](https://drafts.csswg.org/css-scoping-1/#flat-tree) do not participate in CSS value processing, and do not have [declared](#declared-value), [cascaded](#cascaded-value), [specified](#specified-value), [computed](#computed-value), [used](#used-value), or [actual](#actual-value) values, even if they potentially have style declarations assigned to them (for example, by a `style` attribute).</u>

- <a id="change-2020-origins"></a> Clarify origin comparison for quirks mode Content-Type assumptions in [§ 2.3 Content-Type of CSS Style Sheets](#content-type). ([Issue 4838](https://github.com/w3c/csswg-drafts/issues/4838))

Non-trivial changes since the [28 August 2018 Candidate Recommendation](https://www.w3.org/TR/2018/CR-css-cascade-4-20180828/) include:

- <a id="ref-for-cascade①⓪"></a>

  <a id="change-2018-context"></a> Added [context](#cascade-context) to the [cascade](#cascade) sort criteria to accommodate Shadow DOM. [\[DOM\]](#biblio-dom) ([Issue 5372](https://github.com/w3c/csswg-drafts/issues/5372))

- <a id="ref-for-flat-tree④"></a>

  <a id="ref-for-inheritance⑤"></a>

  <a id="ref-for-shadow-tree①"></a>

  <a id="change-2018-shadow-inherit"></a> Defined that, in consideration of [shadow trees](https://www.w3.org/TR/css-scoping-1/#shadow-tree), [inheritance](#inheritance) operates over the [flattened element tree](https://drafts.csswg.org/css-scoping-1/#flat-tree).

- <a id="ref-for-cascade①①"></a>

  <a id="change-2018-drop-scoped"></a> Removed scoping from the [cascade](#cascade) sort criteria, because it has not been implemented.

### <a id="changes-2016"></a>8.3.  Changes Since the 14 January 2016 Candidate Recommendation

Non-trivial changes since the [14 January 2016 Working Draft](https://www.w3.org/TR/2016/CR-css-cascade-4-20160114/) include:

- <a id="change-2016-alias"></a> Precisely defined the types of aliasing that CSS uses. ([Issue 866](https://github.com/w3c/csswg-drafts/issues/866)) See [§ 3.1 Property Aliasing](#aliasing).

- <a id="ref-for-valdef-all-revert④"></a>

  <a id="change-2016-revert"></a> Clarified that [revert](#valdef-all-revert) only affects the cascaded value, not the inherited value.

  > user origin  
  > <a id="ref-for-specified-value①⑧"></a>
  >
  > <a id="ref-for-cascaded-value①⑥"></a>
  >
  > Rolls back the ~~cascade~~ <u>[cascaded value](#cascaded-value) <u>to the user-agent level, so that the [specified value](#specified-value) is calculated as if no author-level or user-level rules were specified for this property <u>on this element</u> .</u></u>
  >
  > author origin  
  > <a id="ref-for-specified-value①⑨"></a>
  >
  > <a id="ref-for-cascaded-value①⑦"></a>
  >
  > Rolls back the ~~cascade~~ <u>[cascaded value](#cascaded-value)</u> to the user level, so that the [specified value](#specified-value) is calculated as if no author-level rules were specified for this property <u>on this element</u> .

- <a id="ref-for-propdef-all③"></a>

  <a id="ref-for-custom-property①"></a>

  <a id="change-2016-custom-all"></a> Clarified that [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) are not reset by the [all](#propdef-all) shorthand. ([2518](https://github.com/w3c/csswg-drafts/issues/2518))

  > <a id="ref-for-propdef-all④"></a>
  >
  > <a id="ref-for-shorthand-property①③"></a>
  >
  > <a id="ref-for-propdef-direction②"></a>
  >
  > <a id="ref-for-propdef-unicode-bidi②"></a>
  >
  > <a id="ref-for-custom-property②"></a>
  >
  > The [all](#propdef-all) property is a [shorthand](#shorthand-property) that resets <em>all</em> CSS properties except [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) and [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi). … <u>It does not reset [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) [\[css-variables-1\]](#biblio-css-variables-1).</u>

- <a id="change-2016-import"></a> Defined more precisely that imported stylesheets are interpreted separately from the importing stylesheet, in terms of ordering of rules, etc.

  > <a id="ref-for-at-ruledef-import①⑤"></a>
  >
  > If an [@import](#at-ruledef-import) rule refers to a valid stylesheet, user agents must treat the contents of the stylesheet as if they were written in place of the <a id="ref-for-at-ruledef-import①⑥"></a>@import rule <u>, with two exceptions:</u>
  >
  > - If a feature (such as the @namespace rule) <em>explicitly</em> defines that it only applies to a particular stylesheet, and not any imported ones, then it doesn’t apply to the imported stylesheet.
  >
  > - <a id="ref-for-at-ruledef-charset①"></a>
  >
  >   If a feature relies on the relative ordering of two or more constructs in a stylesheet (such as the requirement that [@charset](https://www.w3.org/TR/css-syntax-3/#at-ruledef-charset) must not have any other content preceding it), it only applies between constructs in the same stylesheet.

- <a id="ref-for-propdef-display③"></a>

  <a id="change-2016-text"></a> Specified that text nodes are considered children of their parent element, and receive styles via defaulting, as their properties are now observable distinct from their inline parent’s via [display: contents](https://www.w3.org/TR/CSS2/visuren.html#propdef-display) [\[css-display-3\]](#biblio-css-display-3).

  > <a id="ref-for-text-nodes①"></a>
  >
  > <a id="ref-for-elements①"></a>
  >
  > For the purpose of this specification, [text nodes](https://www.w3.org/TR/css-display-3/#text-nodes) are treated as [element](https://www.w3.org/TR/css-display-3/#elements) children of their associated element, and possess the full set of properties; since they cannot be targeted by selectors all of their computed values are assigned by [defaulting](#defaulting).

- <a id="change-2016-override"></a> Removed any mention of the obsolete “override” origin, originally defined by [DOM Level 2 Style](https://www.w3.org/TR/2000/REC-DOM-Level-2-Style-20001113/) and later abandoned. ([Issue 1385](https://github.com/w3c/csswg-drafts/issues/1385))

A [Disposition of Comments](https://drafts.csswg.org/css-cascade-3/issues-cr-2016) is available.

### <a id="changes-2015"></a>8.4.  Changes Since the 21 April 2015 Working Draft

Changes since the [21 April 2015 Working Draft](https://www.w3.org/TR/2015/WD-css-cascade-4-20150421/) include:

- <a id="ref-for-valdef-all-revert⑤"></a>

  Renamed default keyword to [revert](#valdef-all-revert).

- <a id="ref-for-funcdef-supports"></a>

  Allowed dropping duplicate parentheses in [supports()](https://www.w3.org/TR/css-conditional-5/#funcdef-supports) syntax when it only contains one declaration.

### <a id="additions-l3"></a>8.5.  Additions Since Level 3

The following features have been added since [Level 3](https://www.w3.org/TR/css-cascade-3/):

- <a id="ref-for-valdef-all-revert⑥"></a>

  Introduced [revert](#valdef-all-revert) keyword, for rolling back the cascade.

- <a id="ref-for-funcdef-supports①"></a>

  <a id="ref-for-at-ruledef-import①⑦"></a>

  Introduced [supports()](https://www.w3.org/TR/css-conditional-5/#funcdef-supports) syntax for supports-conditional [@import](#at-ruledef-import) rules.

- <a id="ref-for-encapsulation-contexts④"></a>

  <a id="ref-for-cascade①②"></a>

  Added [encapsulation context](#encapsulation-contexts) to the [cascade](#cascade) sort criteria to accommodate Shadow DOM. [\[DOM\]](#biblio-dom)

- Defined the property two aliasing mechanisms CSS uses to support legacy syntaxes. See [§ 3.1 Property Aliasing](#aliasing).

### <a id="changes-2"></a>8.6.  Additions Since Level 2

The following features have been added since [Level 2](https://www.w3.org/TR/CSS2/cascade.html):

- <a id="ref-for-propdef-all⑤"></a>

  The [all](#propdef-all) shorthand

- <a id="ref-for-valdef-all-initial③"></a>

  The [initial](#valdef-all-initial) keyword

- <a id="ref-for-valdef-all-unset②"></a>

  The [unset](#valdef-all-unset) keyword

- <a id="ref-for-cascade①③"></a>

  Incorporation of animations and transitions into the [cascade](#cascade).

## <a id="acknowledgments"></a>Acknowledgments

David Baron, Tantek Çelik, Simon Sapin, Noam Rosenthal, and Boris Zbarsky contributed to this specification.

## <a id="priv-sec"></a> Privacy and Security Considerations

- The cascade process does not distinguish between same-origin and cross-origin stylesheets, enabling the content of cross-origin stylesheets to be inferred from the computed styles they apply to a document.

- User preferences and UA defaults expressed via application of style rules are exposed by the cascade process, and can be inferred from the computed styles they apply to a document.

- <a id="ref-for-at-ruledef-import①⑧"></a>

  <a id="ref-for-cors-protocol"></a>

  The [@import](#at-ruledef-import) rule does not apply the [CORS protocol](https://fetch.spec.whatwg.org/#cors-protocol) to loading cross-origin stylesheets, instead allowing them to be freely imported and applied.

- <a id="ref-for-at-ruledef-import①⑨"></a>

  <a id="termref-for-content-type"></a>

  The [@import](#at-ruledef-import) rule assumes that resources without [`Content-Type` metadata](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#content-type) (or any same-origin file if the host document is in quirks mode) are `text/css`, potentially allowing arbitrary files to be imported into the page and interpreted as CSS, potentially allowing sensitive data to be inferred from the computed styles they apply to a document.

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

<b>Support:</b>Android Browser4.4.3+Baidu Browser7.12+Blackberry BrowserNoneChrome37+Chrome for Android96+Edge79+Firefox27+Firefox for Android95+IENoneIE MobileNoneKaiOS Browser2.5+Opera24+Opera MiniNoneOpera Mobile64+QQ Browser10.4+Safari9.1+Safari on iOS9.3+Samsung Internet4+UC Browser for Android12.12+

Source: [caniuse.com](https://caniuse.com/#feat=css-all) as of 2022-01-08

<b>Support:</b>Android Browser2.3+Baidu Browser7.12+Blackberry Browser7+Chrome4+Chrome for Android96+Edge12+Firefox19+Firefox for Android95+IENoneIE MobileNoneKaiOS Browser2.5+Opera15+Opera MiniNoneOpera Mobile64+QQ Browser10.4+Safari3.2+Safari on iOS4.0+Samsung Internet4+UC Browser for Android12.12+

Source: [caniuse.com](https://caniuse.com/#feat=css-initial-value) as of 2022-01-08

<b>Support:</b>Android Browser96+Baidu Browser7.12+Blackberry BrowserNoneChrome41+Chrome for Android96+Edge13+Firefox27+Firefox for Android95+IENoneIE MobileNoneKaiOS Browser2.5+Opera28+Opera MiniNoneOpera Mobile64+QQ Browser10.4+Safari9.1+Safari on iOS9.3+Samsung Internet4+UC Browser for Android12.12+

Source: [caniuse.com](https://caniuse.com/#feat=css-unset-value) as of 2022-01-08

<b>Support:</b>Android Browser96+Baidu BrowserNoneBlackberry BrowserNoneChrome84+Chrome for Android96+Edge84+Firefox67+Firefox for Android95+IENoneIE MobileNoneKaiOS BrowserNoneOpera73+Opera MiniNoneOpera Mobile64+QQ BrowserNoneSafari9.1+Safari on iOS9.3+Samsung Internet14.0+UC Browser for AndroidNone

Source: [caniuse.com](https://caniuse.com/#feat=css-revert-value) as of 2022-01-08

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [actual](#actual-value), in § 4.6
- [actual value](#actual-value), in § 4.6
- [all](#propdef-all), in § 3.2
- [Animation Origin](#cascade-origin-animation), in § 6.2
- [apply to](#apply), in § 4.5.1
- [author origin](#cascade-origin-author), in § 6.2
- [author-origin](#cascade-origin-author), in § 6.2
- [author presentational hint origin](#author-presentational-hint-origin), in § 6.4
- [author style sheet](#cascade-origin-author), in § 6.2
- [cascade](#cascade), in § 6
- [cascaded](#cascaded-value), in § 4.2
- [cascaded value](#cascaded-value), in § 4.2
- [cascade origin](#origin), in § 6.2
- [computed](#computed-value), in § 4.4
- [computed value](#computed-value), in § 4.4
- [context](#encapsulation-contexts), in § 6.1
- [declared](#declared-value), in § 4.1
- [declared value](#declared-value), in § 4.1
- [encapsulation contexts](#encapsulation-contexts), in § 6.1
- [fetch an @import](#fetch-an-import), in § 2.2
- [@import](#at-ruledef-import), in § 2
- [importance](#important), in § 6.3
- [important](#important), in § 6.3
- [import conditions](#import-conditions), in § 2
- inherit
  - [definition of](#inheritance), in § 7.2
  - [value for all](#valdef-all-inherit), in § 7.3.2
- [inheritance](#inheritance), in § 7.2
- [inherited property](#inherited-property), in § 7.2
- [inherited value](#inherited-value), in § 7.2
- [initial](#valdef-all-initial), in § 7.3.1
- [initial value](#initial-value), in § 7.1
- [legacy name alias](#legacy-name-alias), in § 3.1
- [legacy shorthand](#legacy-shorthand), in § 3.1
- [legacy value alias](#css-legacy-value-alias), in § 4.1.1
- [longhand](#longhand), in § 3
- [longhand property](#longhand), in § 3
- [normal](#normal), in § 6.3
- [origin](#origin), in § 6.2
- [output of the cascade](#output-of-the-cascade), in § 6.1
- [property](#css-property), in § 1
- [reset-only sub-property](#reset-only-sub-property), in § 3
- [revert](#valdef-all-revert), in § 7.3.4
- [shorthand](#shorthand-property), in § 3
- [shorthand property](#shorthand-property), in § 3
- [specified](#specified-value), in § 4.3
- [specified value](#specified-value), in § 4.3
- [sub-property](#longhand), in § 3
- [Transition Origin](#cascade-origin-transition), in § 6.2
- [UA origin](#cascade-origin-ua), in § 6.2
- [UA-origin](#cascade-origin-ua), in § 6.2
- [UA style sheet](#cascade-origin-ua), in § 6.2
- [unset](#valdef-all-unset), in § 7.3.3
- [used](#used-value), in § 4.5
- [used value](#used-value), in § 4.5
- [user-agent origin](#cascade-origin-ua), in § 6.2
- [user-agent style sheet](#cascade-origin-ua), in § 6.2
- [user origin](#cascade-origin-user), in § 6.2
- [user-origin](#cascade-origin-user), in § 6.2
- [user style sheet](#cascade-origin-user), in § 6.2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-2021\] defines the following terms:
  - <a id="term-for-vendor-prefix"></a>vendor-prefixed
- \[css-align-3\] defines the following terms:
  - <a id="term-for-valdef-justify-self-auto"></a>auto
- \[css-backgrounds-3\] defines the following terms:
  - <a id="term-for-propdef-background"></a>background
  - <a id="term-for-propdef-background-color"></a>background-color
  - <a id="term-for-propdef-background-image"></a>background-image
  - <a id="term-for-propdef-border"></a>border
  - <a id="term-for-propdef-border-bottom-width"></a>border-bottom-width
  - <a id="term-for-propdef-border-image"></a>border-image
  - <a id="term-for-propdef-border-left-width"></a>border-left-width
  - <a id="term-for-propdef-border-right-width"></a>border-right-width
  - <a id="term-for-propdef-border-style"></a>border-style
  - <a id="term-for-propdef-border-top-width"></a>border-top-width
  - <a id="term-for-valdef-line-style-dotted"></a>dotted
- \[css-break-3\] defines the following terms:
  - <a id="term-for-propdef-break-before"></a>break-before
  - <a id="term-for-propdef-orphans"></a>orphans
- \[css-break-4\] defines the following terms:
  - <a id="term-for-valdef-break-before-page"></a>page
- \[css-color-4\] defines the following terms:
  - <a id="term-for-typedef-color"></a>\<color\>
  - <a id="term-for-propdef-color"></a>color
  - <a id="term-for-valdef-color-red"></a>red
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="term-for-typedef-supports-condition"></a>\<supports-condition\>
  - <a id="term-for-typedef-supports-decl"></a>\<supports-decl\>
  - <a id="term-for-at-ruledef-media"></a>@media
  - <a id="term-for-at-ruledef-supports"></a>@supports
- \[css-conditional-5\] defines the following terms:
  - <a id="term-for-funcdef-supports"></a>supports()
- \[css-display-3\] defines the following terms:
  - <a id="term-for-display-type"></a>display type
  - <a id="term-for-elements"></a>element
  - <a id="term-for-inline-box"></a>inline box
  - <a id="term-for-text-nodes"></a>text node
- \[css-flexbox-1\] defines the following terms:
  - <a id="term-for-propdef-flex"></a>flex
  - <a id="term-for-flex-item"></a>flex item
- \[css-fonts-4\] defines the following terms:
  - <a id="term-for-valdef-font-weight-bolder"></a>bolder
  - <a id="term-for-propdef-font"></a>font
  - <a id="term-for-propdef-font-family"></a>font-family
  - <a id="term-for-propdef-font-style"></a>font-style
  - <a id="term-for-propdef-font-variant"></a>font-variant
  - <a id="term-for-propdef-font-weight"></a>font-weight
  - <a id="term-for-valdef-font-style-italic"></a>italic
  - <a id="term-for-valdef-font-family-sans-serif"></a>sans-serif
- \[css-fonts-5\] defines the following terms:
  - <a id="term-for-descdef-font-face-font-size"></a>font-size
  - <a id="term-for-propdef-font-size-adjust"></a>font-size-adjust
- \[css-inline-3\] defines the following terms:
  - <a id="term-for-root-inline-box"></a>root inline box
- \[css-lists-3\] defines the following terms:
  - <a id="term-for-propdef-list-style-position"></a>list-style-position
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="term-for-selectordef-after"></a>::after
  - <a id="term-for-selectordef-before"></a>::before
- \[css-scoping-1\] defines the following terms:
  - <a id="term-for-flat-tree"></a>flat tree
  - <a id="term-for-flat-tree①"></a>flattened element tree
  - <a id="term-for-shadow-tree"></a>shadow tree
  - <a id="term-for-tree-context"></a>tree context
- \[css-sizing-3\] defines the following terms:
  - <a id="term-for-propdef-height"></a>height
  - <a id="term-for-propdef-width"></a>width
- \[css-syntax-3\] defines the following terms:
  - <a id="term-for-at-ruledef-charset"></a>@charset
  - <a id="term-for-environment-encoding"></a>environment encoding
  - <a id="term-for-parse-a-stylesheet"></a>parse a stylesheet
  - <a id="term-for-css-property-declarations"></a>property declarations
- \[css-text-3\] defines the following terms:
  - <a id="term-for-propdef-text-align"></a>text-align
  - <a id="term-for-propdef-text-indent"></a>text-indent
  - <a id="term-for-propdef-text-transform"></a>text-transform
- \[css-values-3\] defines the following terms:
  - <a id="term-for-ex"></a>ex
- \[css-values-4\] defines the following terms:
  - <a id="term-for-typedef-length-percentage"></a>\<length-percentage\>
  - <a id="term-for-length-value"></a>\<length\>
  - <a id="term-for-string-value"></a>\<string\>
  - <a id="term-for-url-value"></a>\<url\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-ch"></a>ch
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-em"></a>em
  - <a id="term-for-fetch-a-style-resource"></a>fetch a style resource
  - <a id="term-for-funcdef-url"></a>url()
  - <a id="term-for-valdef-length-vh"></a>vh
  - <a id="term-for-valdef-length-vw"></a>vw
  - <a id="term-for-comb-one"></a>\|
- \[css-variables-1\] defines the following terms:
  - <a id="term-for-custom-property"></a>custom property
- \[css-writing-modes-3\] defines the following terms:
  - <a id="term-for-propdef-direction"></a>direction
  - <a id="term-for-propdef-unicode-bidi"></a>unicode-bidi
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-propdef-text-orientation"></a>text-orientation
  - <a id="term-for-propdef-writing-mode"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-propdef-line-height"></a>line-height
  - <a id="term-for-propdef-page-break-after"></a>page-break-after
  - <a id="term-for-propdef-page-break-before"></a>page-break-before
- \[CSSOM\] defines the following terms:
  - <a id="term-for-dom-window-getcomputedstyle"></a>getComputedStyle(elt)
  - <a id="term-for-concept-css-style-sheet-location"></a>location
  - <a id="term-for-concept-css-style-sheet-origin-clean-flag"></a>origin-clean flag
  - <a id="term-for-concept-css-rule-parent-css-style-sheet"></a>parent css style sheet
  - <a id="term-for-dom-cssimportrule-stylesheet"></a>styleSheet
- \[DOM\] defines the following terms:
  - <a id="term-for-connected"></a>connected
  - <a id="term-for-concept-light-tree"></a>light tree
  - <a id="term-for-concept-document-quirks"></a>quirks mode
  - <a id="term-for-concept-shadow-including-tree-order"></a>shadow-including tree order
- \[FETCH\] defines the following terms:
  - <a id="term-for-cors-protocol"></a>cors protocol
  - <a id="term-for-concept-response"></a>response
  - <a id="term-for-concept-response-url"></a>url
- \[HTML\] defines the following terms:
  - <a id="term-for-content-type"></a>content-type metadata
  - <a id="term-for-cors-same-origin"></a>cors-same-origin
  - <a id="term-for-the-p-element"></a>p
  - <a id="term-for-the-s-element"></a>s
  - <a id="term-for-same-origin"></a>same origin
  - <a id="term-for-the-slot-element"></a>slot
- \[MEDIAQ\] defines the following terms:
  - <a id="term-for-typedef-media-query-list"></a>\<media-query-list\>
  - <a id="term-for-typedef-media-query"></a>\<media-query\>
  - <a id="term-for-valdef-media-all"></a>all
  - <a id="term-for-media-query-list"></a>media query list
- \[selectors-4\] defines the following terms:
  - <a id="term-for-pseudo-element"></a>pseudo-element
- \[URL\] defines the following terms:
  - <a id="term-for-concept-url-parser"></a>url parser

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-2021"></a>\[CSS-2021\]  
Tab Atkins Jr.; Elika Etemad; Florian Rivoal. [CSS Snapshot 2021](https://www.w3.org/TR/css-2021/). 31 December 2021. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-2021&#x2F;](https://www.w3.org/TR/css-2021/)

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
Dean Jackson; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 26 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 15 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
David Baron; Elika Etemad; Chris Lilley. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 23 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-conditional-5"></a>\[CSS-CONDITIONAL-5\]  
David Baron; Elika Etemad; Chris Lilley. [CSS Conditional Rules Module Level 5](https://www.w3.org/TR/css-conditional-5/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-5&#x2F;](https://www.w3.org/TR/css-conditional-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 3 September 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-fonts-5"></a>\[CSS-FONTS-5\]  
Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 5](https://www.w3.org/TR/css-fonts-5/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-5&#x2F;](https://www.w3.org/TR/css-fonts-5/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 31 December 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-scoping-1"></a>\[CSS-SCOPING-1\]  
Tab Atkins Jr.; Elika Etemad. [CSS Scoping Module Level 1](https://www.w3.org/TR/css-scoping-1/). 3 April 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scoping-1&#x2F;](https://www.w3.org/TR/css-scoping-1/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 16 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 11 November 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-fetch"></a>\[FETCH\]  
Anne van Kesteren. [Fetch Standard](https://fetch.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fetch&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fetch.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-mediaq"></a>\[MEDIAQ\]  
Florian Rivoal; Tab Atkins Jr.. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 25 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-4&#x2F;](https://www.w3.org/TR/mediaqueries-4/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-select"></a>\[SELECT\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 21 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-url"></a>\[URL\]  
Anne van Kesteren. [URL Standard](https://url.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;url&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://url.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 24 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad; Steve Zilles. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 27 August 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-lists-3"></a>\[CSS-LISTS-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://www.w3.org/TR/css-lists-3/). 17 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-lists-3&#x2F;](https://www.w3.org/TR/css-lists-3/)

<a id="biblio-css-page-3"></a>\[CSS-PAGE-3\]  
Elika Etemad; Simon Sapin. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 18 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 22 April 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

## <a id="property-index"></a>Property Index

| Name                | Value                                 | Initial                   | Applies to                | Inh.                      | %ages                     | Anim­ation type            | Canonical order | Com­puted value            |
|---------------------|---------------------------------------|---------------------------|---------------------------|---------------------------|---------------------------|---------------------------|-----------------|---------------------------|
| <strong><span><a id="ref-for-propdef-all⑥"></a></span><a href="#propdef-all">all</a>&#xA;      </strong> | initial \| inherit \| unset \| revert | see individual properties | see individual properties | see individual properties | see individual properties | see individual properties | per grammar     | see individual properties |

