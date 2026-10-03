Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/2022/CR-css-variables-1-20220616/).

Original copyright notice: Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Custom Properties for Cascading Variables Module Level 1

Source snapshot: https://www.w3.org/TR/2022/CR-css-variables-1-20220616/

Snapshot SHA-256: 5ce7b2c106af576328f222dd622598f847881f2ba9a64d53d6681d27d07ea0db

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 1 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Custom Properties for Cascading Variables Module Level 1

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2022 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This module introduces cascading variables as a new primitive value type that is accepted by all CSS properties, and custom properties for defining them.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Snapshot</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Snapshot has received [wide review](https://www.w3.org/2021/Process-20211102/#dfn-wide-review), is intended to gather implementation experience, and has commitments from Working Group members to [royalty-free licensing](https://www.w3.org/Consortium/Patent-Policy/#sec-Requirements) for implementations. This document is intended to become a W3C Recommendation; it will remain a Candidate Recommendation at least until 16 August 2022 to gather additional feedback.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-variables” in the title, like this: “\[css-variables\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-variables%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

Large documents or applications (and even small ones) can contain quite a bit of CSS. Many of the values in the CSS file will be duplicate data; for example, a site may establish a color scheme and reuse three or four colors throughout the site. Altering this data can be difficult and error-prone, since it’s scattered throughout the CSS file (and possibly across multiple files), and may not be amenable to Find-and-Replace.

<a id="ref-for-custom-property"></a>

<a id="ref-for-funcdef-var"></a>

This module introduces a family of custom author-defined properties known collectively as [custom properties](#custom-property), which allow an author to assign arbitrary values to a property with an author-chosen name, and the [var()](#funcdef-var) function, which allow an author to then use those values in other properties elsewhere in the document. This makes it easier to read large files, as seemingly-arbitrary values now have informative names, and makes editing such files much easier and less error-prone, as one only has to change the value once, in the <a id="ref-for-custom-property①"></a>custom property, and the change will propagate to all uses of that variable automatically.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="defining-variables"></a>2.  Defining Custom Properties: the --\* family of properties

<a id="ref-for-custom-property②"></a>

<a id="ref-for-substitute-a-var"></a>

<a id="ref-for-funcdef-var①"></a>

This specification defines an open-ended set of properties called [custom properties](#custom-property), which, among other things, are used to define the [substitution value](#substitute-a-var) of [var()](#funcdef-var) functions.

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-"></a>--\*

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-typedef-declaration-value"></a>

[\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value)[?](https://www.w3.org/TR/css-values-4/#mult-opt)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-guaranteed-invalid-value"></a>

the [guaranteed-invalid value](#guaranteed-invalid-value)

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

all elements and all pseudo-elements (including those with restricted property lists)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

yes

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-guaranteed-invalid-value①"></a>

specified value with variables substituted, or the [guaranteed-invalid value](#guaranteed-invalid-value)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

User agents are expected to support this property on all media, including non-visual ones.

<a id="ref-for-typedef-dashed-ident"></a>

<a id="ref-for-css-css-identifier"></a>

<a id="ref-for-custom-property③"></a>

A <a id="custom-property"></a>custom property is any property whose name starts with two dashes (U+002D HYPHEN-MINUS), like --foo. The <a id="typedef-custom-property-name"></a>\<custom-property-name\> production corresponds to this: it’s defined as any [\<dashed-ident\>](https://www.w3.org/TR/css-values-4/#typedef-dashed-ident) (a valid [identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier) that starts with two dashes), except -- itself, which is reserved for future use by CSS. [Custom properties](#custom-property) are solely for use by authors and users; CSS will never give them a meaning beyond what is presented here.

Tests

- [variable-declaration-29.html](https://wpt.fyi/results/css/css-variables/variable-declaration-29.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-29.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-29.html)
- [variable-declaration-31.html](https://wpt.fyi/results/css/css-variables/variable-declaration-31.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-31.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-31.html)
- [variable-declaration-32.html](https://wpt.fyi/results/css/css-variables/variable-declaration-32.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-32.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-32.html)
- [variable-declaration-33.html](https://wpt.fyi/results/css/css-variables/variable-declaration-33.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-33.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-33.html)
- [variable-declaration-34.html](https://wpt.fyi/results/css/css-variables/variable-declaration-34.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-34.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-34.html)
- [variable-declaration-35.html](https://wpt.fyi/results/css/css-variables/variable-declaration-35.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-35.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-35.html)
- [variable-declaration-36.html](https://wpt.fyi/results/css/css-variables/variable-declaration-36.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-36.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-36.html)
- [variable-declaration-40.html](https://wpt.fyi/results/css/css-variables/variable-declaration-40.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-40.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-40.html)
- [variable-declaration-41.html](https://wpt.fyi/results/css/css-variables/variable-declaration-41.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-41.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-41.html)
- [variable-declaration-42.html](https://wpt.fyi/results/css/css-variables/variable-declaration-42.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-42.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-42.html)
- [variable-empty-name-reserved.html](https://wpt.fyi/results/css/css-variables/variable-empty-name-reserved.html) [(live test)](http://wpt.live/css/css-variables/variable-empty-name-reserved.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-empty-name-reserved.html)

<a id="ref-for-funcdef-var②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-81b6bcdd"></a> Custom properties define variables, referenced with the [var()](#funcdef-var) notation, which can be used for many purposes. For example, a page that consistently uses a small set of colors in its design can store the colors in custom properties and use them with variables:
>
> ```text
> :root {
>   --main-color: #06c;
>   --accent-color: #006;
> }
> /* The rest of the CSS file */
> #foo h1 {
>   color: var(--main-color);
> }
> ```
>
> The naming provides a mnemonic for the colors, prevents difficult-to-spot typos in the color codes, and if the theme colors are ever changed, focuses the change on one simple spot (the custom property value) rather than requiring many edits across all stylesheets in the webpage.

<a id="ref-for-ascii-case-insensitive"></a>

<a id="ref-for-string-is"></a>

Unlike other CSS properties, custom property names are <em>not</em> [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive). Instead, custom property names are only equal to each other if they are [identical to](https://infra.spec.whatwg.org/#string-is) each other.

Tests

- [css-vars-custom-property-case-sensitive-001.html](https://wpt.fyi/results/css/css-variables/css-vars-custom-property-case-sensitive-001.html) [(live test)](http://wpt.live/css/css-variables/css-vars-custom-property-case-sensitive-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/css-vars-custom-property-case-sensitive-001.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a71a0c0c"></a> While both --foo and --FOO are valid, they are distinct properties —using var(--foo) will refer to the first one, while using var(--FOO) will refer to the second.
>
> <a id="ref-for-string-is①"></a>
>
> Perhaps more surprisingly, --foó and --foo&#x301; are distinct properties. The first is spelled with U+00F3 (LATIN SMALL LETTER O WITH ACUTE) while the second is spelled with an ASCII "o" followed by U+0301 (COMBINING ACUTE ACCENT), and the "[identical to](https://infra.spec.whatwg.org/#string-is)" relation uses direct codepoint-by-codepoint comparison to determine if two strings are equal, to avoid the complexities and pitfalls of unicode normalization and locale-specific collation.

Operating systems, keyboards, or input methods sometimes encode visually-identical text using different codepoint sequences. Authors are advised to choose variable names that avoid potential confusion or to use escapes and other means to ensure that similar appearing sequences are identical. See Section 2.3 in [\[CHARMOD-NORM\]](#biblio-charmod-norm) for examples.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-120efb12"></a> Developers maintaining the following CSS might be confused why the test patch is red:
>
> ```text
> --fijord: red;
> --fĳord: green;
> --ﬁjord: blue;
> 
> .test {
>   background-color: var(--fijord);
> }
> ```
>
> The reason is that the first custom property uses the character sequence LATIN SMALL LETTER F + LATIN SMALL LETTER I + LATIN SMALL LETTER J; the second, identical-looking one uses the character sequence LATIN SMALL LETTER F + LATIN SMALL LIGATURE IJ while the third uses the character sequence LATIN SMALL LIGATURE FI + LATIN SMALL LETTER J.
>
> So the CSS contains three distinct custom properties, two of which are unused.

<a id="ref-for-propdef-all"></a>

Custom properties are <strong>not</strong> reset by the [all](https://www.w3.org/TR/css-cascade-5/#propdef-all) property. <strong data-conversion-semantic="note">Note:</strong> We may define a property in the future that resets all variables.

<a id="ref-for-css-wide-keywords①"></a>

The [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) can be used in custom properties, with the same meaning as in any another property.

Tests

- [variable-declaration-43.html](https://wpt.fyi/results/css/css-variables/variable-declaration-43.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-43.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-43.html)
- [variable-declaration-44.html](https://wpt.fyi/results/css/css-variables/variable-declaration-44.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-44.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-44.html)
- [variable-declaration-45.html](https://wpt.fyi/results/css/css-variables/variable-declaration-45.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-45.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-45.html)
- [variable-declaration-46.html](https://wpt.fyi/results/css/css-variables/variable-declaration-46.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-46.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-46.html)
- [variable-declaration-47.html](https://wpt.fyi/results/css/css-variables/variable-declaration-47.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-47.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-47.html)
- [variable-declaration-56.html](https://wpt.fyi/results/css/css-variables/variable-declaration-56.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-56.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-56.html)
- [variable-declaration-57.html](https://wpt.fyi/results/css/css-variables/variable-declaration-57.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-57.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-57.html)
- [variable-declaration-58.html](https://wpt.fyi/results/css/css-variables/variable-declaration-58.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-58.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-58.html)
- [variable-declaration-60.html](https://wpt.fyi/results/css/css-variables/variable-declaration-60.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-60.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-60.html)
- [variable-definition-keywords.html](https://wpt.fyi/results/css/css-variables/variable-definition-keywords.html) [(live test)](http://wpt.live/css/css-variables/variable-definition-keywords.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-definition-keywords.html)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That is, they’re interpreted at cascaded-value time as normal, and are not preserved as the custom property’s value, and thus are not substituted in by the corresponding variable.

<a id="ref-for-custom-property④"></a>

<a id="ref-for-funcdef-var③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While this module focuses on the use of [custom properties](#custom-property) with the [var()](#funcdef-var) function to create “variables”, they can also be used as actual custom properties, parsed by and acted on by script. It’s expected that the CSS Extensions spec [\[CSS-EXTENSIONS\]](#biblio-css-extensions) will expand on these use-cases and make them easier to do.

<a id="ref-for-at-ruledef-media"></a>

Custom properties are ordinary properties, so they can be declared on any element, are resolved with the normal inheritance and cascade rules, can be made conditional with [@media](https://www.w3.org/TR/css3-conditional/#at-ruledef-media) and other conditional rules, can be used in HTML’s `style` attribute, can be read or set using the CSSOM, etc.

Tests

- [css-vars-custom-property-inheritance.html](https://wpt.fyi/results/css/css-variables/css-vars-custom-property-inheritance.html) [(live test)](http://wpt.live/css/css-variables/css-vars-custom-property-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/css-vars-custom-property-inheritance.html)
- [variable-created-document.html](https://wpt.fyi/results/css/css-variables/variable-created-document.html) [(live test)](http://wpt.live/css/css-variables/variable-created-document.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-created-document.html)
- [variable-created-element.html](https://wpt.fyi/results/css/css-variables/variable-created-element.html) [(live test)](http://wpt.live/css/css-variables/variable-created-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-created-element.html)
- [variable-cssText.html](https://wpt.fyi/results/css/css-variables/variable-cssText.html) [(live test)](http://wpt.live/css/css-variables/variable-cssText.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-cssText.html)
- [variable-declaration-06.html](https://wpt.fyi/results/css/css-variables/variable-declaration-06.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-06.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-06.html)
- [variable-definition-cascading.html](https://wpt.fyi/results/css/css-variables/variable-definition-cascading.html) [(live test)](http://wpt.live/css/css-variables/variable-definition-cascading.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-definition-cascading.html)
- [variable-external-declaration-01.html](https://wpt.fyi/results/css/css-variables/variable-external-declaration-01.html) [(live test)](http://wpt.live/css/css-variables/variable-external-declaration-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-external-declaration-01.html)
- [variable-external-reference-01.html](https://wpt.fyi/results/css/css-variables/variable-external-reference-01.html) [(live test)](http://wpt.live/css/css-variables/variable-external-reference-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-external-reference-01.html)
- [variable-external-supports-01.html](https://wpt.fyi/results/css/css-variables/variable-external-supports-01.html) [(live test)](http://wpt.live/css/css-variables/variable-external-supports-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-external-supports-01.html)
- [variable-first-letter.html](https://wpt.fyi/results/css/css-variables/variable-first-letter.html) [(live test)](http://wpt.live/css/css-variables/variable-first-letter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-first-letter.html)
- [variable-first-line.html](https://wpt.fyi/results/css/css-variables/variable-first-line.html) [(live test)](http://wpt.live/css/css-variables/variable-first-line.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-first-line.html)
- [variable-pseudo-element.html](https://wpt.fyi/results/css/css-variables/variable-pseudo-element.html) [(live test)](http://wpt.live/css/css-variables/variable-pseudo-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-pseudo-element.html)
- [variable-reference-13.html](https://wpt.fyi/results/css/css-variables/variable-reference-13.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-13.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-13.html)
- [variable-reference-14.html](https://wpt.fyi/results/css/css-variables/variable-reference-14.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-14.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-14.html)
- [variable-reference-shorthands.html](https://wpt.fyi/results/css/css-variables/variable-reference-shorthands.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-shorthands.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-shorthands.html)
- [variable-reference-visited.html](https://wpt.fyi/results/css/css-variables/variable-reference-visited.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-visited.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-visited.html)

<a id="ref-for-custom-property⑤"></a>

<a id="ref-for-at-ruledef-keyframes"></a>

<a id="ref-for-funcdef-var④"></a>

Notably, they can even be transitioned or animated, but since the UA has no way to interpret their contents, they always use the "flips at 50%" behavior that is used for any other pair of values that can’t be intelligently interpolated. However, any [custom property](#custom-property) used in a [@keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes) rule becomes <a id="animation-tainted"></a>animation-tainted, which affects how it is treated when referred to via the [var()](#funcdef-var) function in an animation property.

Tests

- [variable-animation-from-to.html](https://wpt.fyi/results/css/css-variables/variable-animation-from-to.html) [(live test)](http://wpt.live/css/css-variables/variable-animation-from-to.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-animation-from-to.html)
- [variable-animation-over-transition.html](https://wpt.fyi/results/css/css-variables/variable-animation-over-transition.html) [(live test)](http://wpt.live/css/css-variables/variable-animation-over-transition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-animation-over-transition.html)
- [variable-animation-substitute-into-keyframe-shorthand.html](https://wpt.fyi/results/css/css-variables/variable-animation-substitute-into-keyframe-shorthand.html) [(live test)](http://wpt.live/css/css-variables/variable-animation-substitute-into-keyframe-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-animation-substitute-into-keyframe-shorthand.html)
- [variable-animation-substitute-into-keyframe-transform.html](https://wpt.fyi/results/css/css-variables/variable-animation-substitute-into-keyframe-transform.html) [(live test)](http://wpt.live/css/css-variables/variable-animation-substitute-into-keyframe-transform.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-animation-substitute-into-keyframe-transform.html)
- [variable-animation-substitute-into-keyframe.html](https://wpt.fyi/results/css/css-variables/variable-animation-substitute-into-keyframe.html) [(live test)](http://wpt.live/css/css-variables/variable-animation-substitute-into-keyframe.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-animation-substitute-into-keyframe.html)
- [variable-animation-substitute-within-keyframe-fallback.html](https://wpt.fyi/results/css/css-variables/variable-animation-substitute-within-keyframe-fallback.html) [(live test)](http://wpt.live/css/css-variables/variable-animation-substitute-within-keyframe-fallback.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-animation-substitute-within-keyframe-fallback.html)
- [variable-animation-substitute-within-keyframe-multiple.html](https://wpt.fyi/results/css/css-variables/variable-animation-substitute-within-keyframe-multiple.html) [(live test)](http://wpt.live/css/css-variables/variable-animation-substitute-within-keyframe-multiple.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-animation-substitute-within-keyframe-multiple.html)
- [variable-animation-substitute-within-keyframe.html](https://wpt.fyi/results/css/css-variables/variable-animation-substitute-within-keyframe.html) [(live test)](http://wpt.live/css/css-variables/variable-animation-substitute-within-keyframe.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-animation-substitute-within-keyframe.html)
- [variable-animation-to-only.html](https://wpt.fyi/results/css/css-variables/variable-animation-to-only.html) [(live test)](http://wpt.live/css/css-variables/variable-animation-to-only.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-animation-to-only.html)

<a id="ref-for-animation-tainted"></a>

[Animation-tainted](#animation-tainted) is "infectious": custom properties which reference <a id="ref-for-animation-tainted①"></a>animation-tainted properties also become <a id="ref-for-animation-tainted②"></a>animation-tainted.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d6247b0f"></a> This style rule:
>
> ```text
> :root {
>   --header-color: #06c;
> }
> ```
>
> <a id="ref-for-custom-property⑥"></a>
>
> <a id="ref-for-funcdef-var⑤"></a>
>
> declares a [custom property](#custom-property) named --header-color on the root element, and assigns to it the value "#06c". This property is then inherited to the elements in the rest of the document. Its value can be referenced with the [var()](#funcdef-var) function:
>
> ```text
> h1 { background-color: var(--header-color); }
> ```
>
> <a id="ref-for-propdef-background-color"></a>
>
> The preceding rule is equivalent to writing [background-color: \#06c;](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color), except that the variable name makes the origin of the color clearer, and if var(--header-color) is used on other elements in the document, all of the uses can be updated at once by changing the --header-color property on the root element.

<a id="ref-for-custom-property⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c3e4e8d9"></a> If a [custom property](#custom-property) is declared multiple times, the standard cascade rules help resolve it. Variables always draw from the computed value of the associated custom property on the same element:
>
> ```text
> :root { --color: blue; }
> div { --color: green; }
> #alert { --color: red; }
> * { color: var(--color); }
> 
> <p>I inherited blue from the root element!</p>
> <div>I got green set directly on me!</div>
> <div id='alert'>
>   While I got red set directly on me!
>   <p>I’m red too, because of inheritance!</p>
> </div>
> ```
<a id="ref-for-custom-property⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8f013eba"></a> A real-world example of [custom property](#custom-property) usage is easily separating out strings from where they’re used, to aid in maintenance of internationalization:
>
> ```css
> :root,
> :root:lang(en) {--external-link: "external link";}
> :root:lang(el) {--external-link: "εξωτερικός σύνδεσμος";}
> 
> a[href^="http"]::after {content: " (" var(--external-link) ")"}
> ```
>
> The variable declarations can even be kept in a separate file, to make maintaining the translations simpler.

### <a id="syntax"></a>2.1.  Custom Property Value Syntax

<a id="ref-for-custom-property⑨"></a>

<a id="ref-for-typedef-declaration-value①"></a>

<a id="ref-for-typedef-bad-string-token"></a>

<a id="ref-for-typedef-bad-url-token"></a>

<a id="ref-for-tokendef-close-paren"></a>

<a id="ref-for-tokendef-close-square"></a>

<a id="ref-for-tokendef-close-curly"></a>

<a id="ref-for-typedef-semicolon-token"></a>

<a id="ref-for-typedef-delim-token"></a>

The allowed syntax for [custom properties](#custom-property) is extremely permissive. The [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value) production matches <em>any</em> sequence of one or more tokens, so long as the sequence does not contain [\<bad-string-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-bad-string-token), [\<bad-url-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-bad-url-token), unmatched [\<)-token\>](https://www.w3.org/TR/css-syntax-3/#tokendef-close-paren), [\<\]-token\>](https://www.w3.org/TR/css-syntax-3/#tokendef-close-square), or [\<}-token\>](https://www.w3.org/TR/css-syntax-3/#tokendef-close-curly), or top-level [\<semicolon-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-semicolon-token) tokens or [\<delim-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-delim-token) tokens with a value of "!".

Tests

- [test_variable_legal_values.html](https://wpt.fyi/results/css/css-variables/test_variable_legal_values.html) [(live test)](http://wpt.live/css/css-variables/test_variable_legal_values.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/test_variable_legal_values.html)
- [variable-declaration-15.html](https://wpt.fyi/results/css/css-variables/variable-declaration-15.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-15.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-15.html)
- [variable-declaration-24.html](https://wpt.fyi/results/css/css-variables/variable-declaration-24.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-24.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-24.html)
- [variable-declaration-25.html](https://wpt.fyi/results/css/css-variables/variable-declaration-25.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-25.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-25.html)
- [variable-declaration-26.html](https://wpt.fyi/results/css/css-variables/variable-declaration-26.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-26.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-26.html)
- [variable-declaration-59.html](https://wpt.fyi/results/css/css-variables/variable-declaration-59.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-59.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-59.html)

<a id="ref-for-custom-property①⓪"></a>

<a id="ref-for-funcdef-var⑥"></a>

In addition, if the value of a [custom property](#custom-property) contains a [var()](#funcdef-var) reference, the <a id="ref-for-funcdef-var⑦"></a>var() reference must be valid according to the specified <a id="ref-for-funcdef-var⑧"></a>var() grammar. If not, the <a id="ref-for-custom-property①①"></a>custom property is invalid and must be ignored.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This definition, along with the general CSS syntax rules, implies that a custom property value never includes an unmatched quote or bracket, and so cannot have any effect on larger syntax constructs, like the enclosing style rule, when reserialized.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Custom properties can contain a trailing !important, but this is automatically removed from the property’s value by the CSS parser, and makes the custom property "important" in the CSS cascade. In other words, the prohibition on top-level "!" characters does not prevent !important from being used, as the !important is removed before syntax checking happens.

Tests

- [variable-declaration-20.html](https://wpt.fyi/results/css/css-variables/variable-declaration-20.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-20.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-20.html)
- [variable-declaration-23.html](https://wpt.fyi/results/css/css-variables/variable-declaration-23.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-23.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-23.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fb41560d"></a> For example, the following is a valid custom property:
>
> ```text
> --foo: if(x > 5) this.width = 10;
> ```
>
> While this value is obviously useless as a <em>variable</em>, as it would be invalid in any normal property, it might be read and acted on by JavaScript.

<a id="ref-for-funcdef-var⑨"></a>

<a id="ref-for-ascii-case-insensitive①"></a>

The values of custom properties, and the values of [var()](#funcdef-var) functions substituted into custom properties, are <em>case-sensitive</em>, and must be preserved in their original author-given casing. (Many CSS values are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive), which user agents can take advantage of by "canonicalizing" them into a single casing, but that isn’t allowed for custom properties.)

Tests

- [variable-declaration-38.html](https://wpt.fyi/results/css/css-variables/variable-declaration-38.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-38.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-38.html)
- [variable-declaration-39.html](https://wpt.fyi/results/css/css-variables/variable-declaration-39.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-39.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-39.html)

<a id="ref-for-funcdef-var①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Because custom properties can contain <em>anything</em>, there is no general way to know how to interpret what’s inside of them (until they’re substituted into a known property with [var()](#funcdef-var)). Rather than have them <em>partially</em> resolve in some cases but not others, they’re left completely unresolved; they’re a bare stream of [CSS tokens](https://www.w3.org/TR/css-syntax-3/#tokenization) interspersed with <a id="ref-for-funcdef-var①①"></a>var() functions.
>
> <a id="ref-for-propdef-background"></a>
>
> This has some knock-on implications. For example, relative URLs in CSS are resolved against the base URL of the stylesheet the value appears in. However, if a custom property like --my-image: url(foo.jpg); shows up in an `"/a/style.css"` stylesheet, it will not resolve into an absolute URL immediately; if that variable is later used in a <em>different</em> `"/b/style.css"` stylesheet like [background: var(--my-image);](https://www.w3.org/TR/css-backgrounds-3/#propdef-background), it will resolve <em>at that point</em> to `"/b/foo.jpg"`.

### <a id="guaranteed-invalid"></a>2.2.  Guaranteed-Invalid Values

<a id="ref-for-custom-property①②"></a>

<a id="ref-for-funcdef-var①②"></a>

<a id="ref-for-invalid-at-computed-value-time"></a>

The initial value of a [custom property](#custom-property) is a <a id="guaranteed-invalid-value"></a>guaranteed-invalid value. As defined in [§ 3 Using Cascading Variables: the var() notation](#using-variables), using [var()](#funcdef-var) to substitute a <a id="ref-for-custom-property①③"></a>custom property with this as its value makes the property referencing it [invalid at computed-value time](#invalid-at-computed-value-time).

<a id="ref-for-guaranteed-invalid-value②"></a>

<a id="ref-for-valdef-all-initial"></a>

This value serializes as the empty string, but actually writing an empty value into a custom property, like --foo: ;, is a valid (empty) value, not the [guaranteed-invalid value](#guaranteed-invalid-value). If, for whatever reason, one wants to manually reset a variable to the <a id="ref-for-guaranteed-invalid-value③"></a>guaranteed-invalid value, using the keyword [initial](https://www.w3.org/TR/css-cascade-5/#valdef-all-initial) will do this.

### <a id="cycles"></a>2.3.  Resolving Dependency Cycles

<a id="ref-for-custom-property①④"></a>

<a id="ref-for-funcdef-var①③"></a>

[Custom properties](#custom-property) are left almost entirely unevaluated, except that they allow and evaluate the [var()](#funcdef-var) function in their value. This can create cyclic dependencies where a custom property uses a <a id="ref-for-funcdef-var①④"></a>var() referring to itself, or two or more <a id="ref-for-custom-property①⑤"></a>custom properties each attempt to refer to each other.

<a id="ref-for-custom-property①⑥"></a>

<a id="ref-for-funcdef-var①⑤"></a>

For each element, create a directed dependency graph, containing nodes for each [custom property](#custom-property). If the value of a <a id="ref-for-custom-property①⑦"></a>custom property <var>prop</var> contains a [var()](#funcdef-var) function referring to the property <var>var</var> (including in the fallback argument of <a id="ref-for-funcdef-var①⑥"></a>var()), add an edge between <var>prop</var> and the <var>var</var>. <strong data-conversion-semantic="note">Note:</strong> Edges are possible from a custom property to itself.

<a id="ref-for-custom-property①⑧"></a>

<a id="ref-for-invalid-at-computed-value-time①"></a>

If there is a cycle in the dependency graph, all the [custom properties](#custom-property) in the cycle are [invalid at computed-value time](#invalid-at-computed-value-time).

Tests

- [variable-cycles.html](https://wpt.fyi/results/css/css-variables/variable-cycles.html) [(live test)](http://wpt.live/css/css-variables/variable-cycles.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-cycles.html)
- [variable-declaration-30.html](https://wpt.fyi/results/css/css-variables/variable-declaration-30.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-30.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-30.html)
- [variable-declaration-48.html](https://wpt.fyi/results/css/css-variables/variable-declaration-48.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-48.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-48.html)
- [variable-declaration-49.html](https://wpt.fyi/results/css/css-variables/variable-declaration-49.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-49.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-49.html)
- [variable-declaration-50.html](https://wpt.fyi/results/css/css-variables/variable-declaration-50.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-50.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-50.html)
- [variable-reference-39.html](https://wpt.fyi/results/css/css-variables/variable-reference-39.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-39.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-39.html)

<a id="ref-for-invalid-at-computed-value-time②"></a>

<a id="ref-for-propdef-font-size"></a>

<a id="ref-for-em"></a>

<a id="ref-for-guaranteed-invalid-value④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Defined properties that participate in a dependency cycle either end up with invalid variables in their value (becoming [invalid at computed-value time](#invalid-at-computed-value-time)), or define their own cyclic handling (like [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) using [em](https://www.w3.org/TR/css-values-4/#em) values). They do not compute to the [guaranteed-invalid value](#guaranteed-invalid-value) like custom properties do.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-75bf73fb"></a> This example shows a custom property safely using a variable:
>
> ```text
> :root {
>   --main-color: #c06;
>   --accent-background: linear-gradient(to top, var(--main-color), white);
> }
> ```
>
> The --accent-background property (along with any other properties that use var(--main-color)) will automatically update when the --main-color property is changed.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b8947a2b"></a> On the other hand, this example shows an invalid instance of variables depending on each other:
>
> ```text
> :root {
>   --one: calc(var(--two) + 20px);
>   --two: calc(var(--one) - 20px);
> }
> ```
>
> <a id="ref-for-invalid-at-computed-value-time③"></a>
>
> <a id="ref-for-guaranteed-invalid-value⑤"></a>
>
> Both --one and --two are now [invalid at computed-value time](#invalid-at-computed-value-time), and compute to the [guaranteed-invalid value](#guaranteed-invalid-value) rather than lengths.

<a id="ref-for-custom-property①⑨"></a>

<a id="ref-for-funcdef-var①⑦"></a>

It is important to note that [custom properties](#custom-property) resolve any [var()](#funcdef-var) functions in their values at computed-value time, which occurs <em>before</em> the value is inherited. In general, cyclic dependencies occur only when multiple custom properties on the same element refer to each other; custom properties defined on elements higher in the element tree can never cause a cyclic reference with properties defined on elements lower in the element tree.

Tests

- [variable-declaration-51.html](https://wpt.fyi/results/css/css-variables/variable-declaration-51.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-51.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-51.html)
- [variable-declaration-52.html](https://wpt.fyi/results/css/css-variables/variable-declaration-52.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-52.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-52.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-beef67cd"></a> For example, given the following structure, these custom properties are <strong>not</strong> cyclic, and all define valid variables:
>
> ```text
> <one><two><three /></two></one>
> <style>
> one   { --foo: 10px; }
> two   { --bar: calc(var(--foo) + 10px); }
> three { --foo: calc(var(--bar) + 10px); }
> </style>
> ```
>
> The \<one\> element defines a value for --foo. The \<two\> element inherits this value, and additionally assigns a value to --bar using the foo variable. Finally, the \<three\> element inherits the --bar value <em>after</em> variable substitution (in other words, it sees the value calc(10px + 10px)), and then redefines --foo in terms of that value. Since the value it inherited for --bar no longer contains a reference to the --foo property defined on \<one\>, defining --foo using the var(--bar) variable is not cyclic, and actually defines a value that will eventually (when referenced as a variable in a normal property) resolve to 30px.

<a id="ref-for-funcdef-var①⑧"></a>

## <a id="using-variables"></a>3.  Using Cascading Variables: the [var()](#funcdef-var) notation

<a id="ref-for-custom-property②⓪"></a>

<a id="ref-for-funcdef-var①⑨"></a>

The value of a [custom property](#custom-property) can be substituted into the value of another property with the [var()](#funcdef-var) function. The syntax of <a id="ref-for-funcdef-var②⓪"></a>var() is:

<a id="funcdef-var"></a>

<a id="ref-for-typedef-custom-property-name"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-declaration-value②"></a>

<a id="ref-for-mult-opt①"></a>

```text
var() = var( <custom-property-name> , <declaration-value>? )
```
Tests

- [variable-reference-07.html](https://wpt.fyi/results/css/css-variables/variable-reference-07.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-07.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-07.html)
- [variable-reference-08.html](https://wpt.fyi/results/css/css-variables/variable-reference-08.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-08.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-08.html)
- [variable-reference-09.html](https://wpt.fyi/results/css/css-variables/variable-reference-09.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-09.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-09.html)
- [variable-reference-10.html](https://wpt.fyi/results/css/css-variables/variable-reference-10.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-10.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-10.html)
- [variable-reference-17.html](https://wpt.fyi/results/css/css-variables/variable-reference-17.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-17.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-17.html)
- [variable-reference-20.html](https://wpt.fyi/results/css/css-variables/variable-reference-20.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-20.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-20.html)
- [variable-reference-21.html](https://wpt.fyi/results/css/css-variables/variable-reference-21.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-21.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-21.html)
- [variable-reference-22.html](https://wpt.fyi/results/css/css-variables/variable-reference-22.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-22.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-22.html)
- [variable-reference-23.html](https://wpt.fyi/results/css/css-variables/variable-reference-23.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-23.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-23.html)
- [variable-reference-24.html](https://wpt.fyi/results/css/css-variables/variable-reference-24.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-24.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-24.html)
- [variable-reference-25.html](https://wpt.fyi/results/css/css-variables/variable-reference-25.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-25.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-25.html)
- [variable-reference-28.html](https://wpt.fyi/results/css/css-variables/variable-reference-28.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-28.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-28.html)
- [variable-reference-29.html](https://wpt.fyi/results/css/css-variables/variable-reference-29.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-29.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-29.html)
- [variable-reference-31.html](https://wpt.fyi/results/css/css-variables/variable-reference-31.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-31.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-31.html)
- [variable-reference-32.html](https://wpt.fyi/results/css/css-variables/variable-reference-32.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-32.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-32.html)
- [variable-reference-33.html](https://wpt.fyi/results/css/css-variables/variable-reference-33.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-33.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-33.html)
- [variable-reference-34.html](https://wpt.fyi/results/css/css-variables/variable-reference-34.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-34.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-34.html)
- [variable-reference-35.html](https://wpt.fyi/results/css/css-variables/variable-reference-35.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-35.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-35.html)
- [variable-reference.html](https://wpt.fyi/results/css/css-variables/variable-reference.html) [(live test)](http://wpt.live/css/css-variables/variable-reference.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference.html)

@supports

- [variable-supports-01.html](https://wpt.fyi/results/css/css-variables/variable-supports-01.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-01.html)
- [variable-supports-02.html](https://wpt.fyi/results/css/css-variables/variable-supports-02.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-02.html)
- [variable-supports-03.html](https://wpt.fyi/results/css/css-variables/variable-supports-03.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-03.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-03.html)
- [variable-supports-04.html](https://wpt.fyi/results/css/css-variables/variable-supports-04.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-04.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-04.html)
- [variable-supports-05.html](https://wpt.fyi/results/css/css-variables/variable-supports-05.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-05.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-05.html)
- [variable-supports-06.html](https://wpt.fyi/results/css/css-variables/variable-supports-06.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-06.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-06.html)
- [variable-supports-07.html](https://wpt.fyi/results/css/css-variables/variable-supports-07.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-07.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-07.html)
- [variable-supports-08.html](https://wpt.fyi/results/css/css-variables/variable-supports-08.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-08.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-08.html)
- [variable-supports-09.html](https://wpt.fyi/results/css/css-variables/variable-supports-09.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-09.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-09.html)
- [variable-supports-10.html](https://wpt.fyi/results/css/css-variables/variable-supports-10.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-10.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-10.html)
- [variable-supports-11.html](https://wpt.fyi/results/css/css-variables/variable-supports-11.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-11.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-11.html)
- [variable-supports-12.html](https://wpt.fyi/results/css/css-variables/variable-supports-12.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-12.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-12.html)
- [variable-supports-13.html](https://wpt.fyi/results/css/css-variables/variable-supports-13.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-13.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-13.html)
- [variable-supports-14.html](https://wpt.fyi/results/css/css-variables/variable-supports-14.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-14.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-14.html)
- [variable-supports-15.html](https://wpt.fyi/results/css/css-variables/variable-supports-15.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-15.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-15.html)
- [variable-supports-16.html](https://wpt.fyi/results/css/css-variables/variable-supports-16.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-16.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-16.html)
- [variable-supports-17.html](https://wpt.fyi/results/css/css-variables/variable-supports-17.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-17.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-17.html)
- [variable-supports-18.html](https://wpt.fyi/results/css/css-variables/variable-supports-18.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-18.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-18.html)
- [variable-supports-19.html](https://wpt.fyi/results/css/css-variables/variable-supports-19.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-19.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-19.html)
- [variable-supports-20.html](https://wpt.fyi/results/css/css-variables/variable-supports-20.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-20.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-20.html)
- [variable-supports-21.html](https://wpt.fyi/results/css/css-variables/variable-supports-21.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-21.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-21.html)
- [variable-supports-22.html](https://wpt.fyi/results/css/css-variables/variable-supports-22.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-22.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-22.html)
- [variable-supports-23.html](https://wpt.fyi/results/css/css-variables/variable-supports-23.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-23.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-23.html)
- [variable-supports-24.html](https://wpt.fyi/results/css/css-variables/variable-supports-24.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-24.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-24.html)
- [variable-supports-25.html](https://wpt.fyi/results/css/css-variables/variable-supports-25.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-25.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-25.html)
- [variable-supports-26.html](https://wpt.fyi/results/css/css-variables/variable-supports-26.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-26.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-26.html)
- [variable-supports-27.html](https://wpt.fyi/results/css/css-variables/variable-supports-27.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-27.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-27.html)
- [variable-supports-28.html](https://wpt.fyi/results/css/css-variables/variable-supports-28.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-28.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-28.html)
- [variable-supports-29.html](https://wpt.fyi/results/css/css-variables/variable-supports-29.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-29.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-29.html)
- [variable-supports-30.html](https://wpt.fyi/results/css/css-variables/variable-supports-30.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-30.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-30.html)
- [variable-supports-31.html](https://wpt.fyi/results/css/css-variables/variable-supports-31.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-31.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-31.html)
- [variable-supports-32.html](https://wpt.fyi/results/css/css-variables/variable-supports-32.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-32.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-32.html)
- [variable-supports-33.html](https://wpt.fyi/results/css/css-variables/variable-supports-33.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-33.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-33.html)
- [variable-supports-34.html](https://wpt.fyi/results/css/css-variables/variable-supports-34.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-34.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-34.html)
- [variable-supports-35.html](https://wpt.fyi/results/css/css-variables/variable-supports-35.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-35.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-35.html)
- [variable-supports-36.html](https://wpt.fyi/results/css/css-variables/variable-supports-36.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-36.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-36.html)
- [variable-supports-37.html](https://wpt.fyi/results/css/css-variables/variable-supports-37.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-37.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-37.html)
- [variable-supports-38.html](https://wpt.fyi/results/css/css-variables/variable-supports-38.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-38.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-38.html)
- [variable-supports-39.html](https://wpt.fyi/results/css/css-variables/variable-supports-39.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-39.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-39.html)
- [variable-supports-40.html](https://wpt.fyi/results/css/css-variables/variable-supports-40.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-40.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-40.html)
- [variable-supports-41.html](https://wpt.fyi/results/css/css-variables/variable-supports-41.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-41.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-41.html)
- [variable-supports-42.html](https://wpt.fyi/results/css/css-variables/variable-supports-42.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-42.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-42.html)
- [variable-supports-43.html](https://wpt.fyi/results/css/css-variables/variable-supports-43.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-43.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-43.html)
- [variable-supports-44.html](https://wpt.fyi/results/css/css-variables/variable-supports-44.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-44.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-44.html)
- [variable-supports-45.html](https://wpt.fyi/results/css/css-variables/variable-supports-45.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-45.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-45.html)
- [variable-supports-46.html](https://wpt.fyi/results/css/css-variables/variable-supports-46.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-46.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-46.html)
- [variable-supports-47.html](https://wpt.fyi/results/css/css-variables/variable-supports-47.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-47.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-47.html)
- [variable-supports-48.html](https://wpt.fyi/results/css/css-variables/variable-supports-48.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-48.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-48.html)
- [variable-supports-49.html](https://wpt.fyi/results/css/css-variables/variable-supports-49.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-49.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-49.html)
- [variable-supports-50.html](https://wpt.fyi/results/css/css-variables/variable-supports-50.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-50.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-50.html)
- [variable-supports-51.html](https://wpt.fyi/results/css/css-variables/variable-supports-51.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-51.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-51.html)
- [variable-supports-52.html](https://wpt.fyi/results/css/css-variables/variable-supports-52.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-52.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-52.html)
- [variable-supports-53.html](https://wpt.fyi/results/css/css-variables/variable-supports-53.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-53.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-53.html)
- [variable-supports-54.html](https://wpt.fyi/results/css/css-variables/variable-supports-54.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-54.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-54.html)
- [variable-supports-55.html](https://wpt.fyi/results/css/css-variables/variable-supports-55.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-55.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-55.html)
- [variable-supports-56.html](https://wpt.fyi/results/css/css-variables/variable-supports-56.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-56.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-56.html)
- [variable-supports-57.html](https://wpt.fyi/results/css/css-variables/variable-supports-57.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-57.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-57.html)
- [variable-supports-58.html](https://wpt.fyi/results/css/css-variables/variable-supports-58.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-58.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-58.html)
- [variable-supports-59.html](https://wpt.fyi/results/css/css-variables/variable-supports-59.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-59.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-59.html)
- [variable-supports-60.html](https://wpt.fyi/results/css/css-variables/variable-supports-60.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-60.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-60.html)
- [variable-supports-61.html](https://wpt.fyi/results/css/css-variables/variable-supports-61.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-61.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-61.html)
- [variable-supports-62.html](https://wpt.fyi/results/css/css-variables/variable-supports-62.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-62.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-62.html)
- [variable-supports-63.html](https://wpt.fyi/results/css/css-variables/variable-supports-63.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-63.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-63.html)
- [variable-supports-64.html](https://wpt.fyi/results/css/css-variables/variable-supports-64.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-64.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-64.html)
- [variable-supports-65.html](https://wpt.fyi/results/css/css-variables/variable-supports-65.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-65.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-65.html)
- [variable-supports-66.html](https://wpt.fyi/results/css/css-variables/variable-supports-66.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-66.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-66.html)
- [variable-supports-67.html](https://wpt.fyi/results/css/css-variables/variable-supports-67.html) [(live test)](http://wpt.live/css/css-variables/variable-supports-67.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-supports-67.html)

------------------------------------------------------------------------

<a id="ref-for-comb-comma①"></a>

<a id="ref-for-funcdef-var②①"></a>

In an exception to the usual [comma elision rules](https://www.w3.org/TR/css-values-4/#comb-comma), which require commas to be omitted when they’re not separating values, a bare comma, with nothing following it, must be treated as valid in [var()](#funcdef-var), indicating an empty fallback value.

Tests

- [variable-declaration-07.html](https://wpt.fyi/results/css/css-variables/variable-declaration-07.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-07.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-07.html)
- [variable-declaration-37.html](https://wpt.fyi/results/css/css-variables/variable-declaration-37.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-37.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-37.html)
- [variable-reference-06.html](https://wpt.fyi/results/css/css-variables/variable-reference-06.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-06.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-06.html)
- [variable-reference-11.html](https://wpt.fyi/results/css/css-variables/variable-reference-11.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-11.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-11.html)
- [variable-reference-26.html](https://wpt.fyi/results/css/css-variables/variable-reference-26.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-26.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-26.html)
- [variable-reference-27.html](https://wpt.fyi/results/css/css-variables/variable-reference-27.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-27.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-27.html)

<a id="ref-for-funcdef-var②②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That is, var(--a,) is a valid function, specifying that if the --a custom property is invalid or missing, the [var()](#funcdef-var) should be replaced with nothing.

<a id="ref-for-funcdef-var②③"></a>

The [var()](#funcdef-var) function can be used in place of any part of a value in any property on an element. The <a id="ref-for-funcdef-var②④"></a>var() function can not be used as property names, selectors, or anything else besides property values. (Doing so usually produces invalid syntax, or else a value whose meaning has no connection to the variable.)

Tests

- [variable-external-font-face-01.html](https://wpt.fyi/results/css/css-variables/variable-external-font-face-01.html) [(live test)](http://wpt.live/css/css-variables/variable-external-font-face-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-external-font-face-01.html)
- [variable-font-face-01.html](https://wpt.fyi/results/css/css-variables/variable-font-face-01.html) [(live test)](http://wpt.live/css/css-variables/variable-font-face-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-font-face-01.html)
- [variable-font-face-02.html](https://wpt.fyi/results/css/css-variables/variable-font-face-02.html) [(live test)](http://wpt.live/css/css-variables/variable-font-face-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-font-face-02.html)

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
> <a id="ref-for-propdef-margin-top"></a>
>
> This is <em>not</em> equivalent to setting [margin-top: 20px;](https://www.w3.org/TR/css-box-4/#propdef-margin-top). Instead, the second declaration is simply thrown away as a syntax error for having an invalid property name.

<a id="ref-for-custom-property②①"></a>

<a id="ref-for-guaranteed-invalid-value⑥"></a>

The first argument to the function is the name of the [custom property](#custom-property) to be substituted. The second argument to the function, if provided, is a fallback value, which is used as the substitution value when the value of the referenced <a id="ref-for-custom-property②②"></a>custom property is the [guaranteed-invalid value](#guaranteed-invalid-value).

Tests

- [variable-declaration-08.html](https://wpt.fyi/results/css/css-variables/variable-declaration-08.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-08.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-08.html)
- [variable-declaration-09.html](https://wpt.fyi/results/css/css-variables/variable-declaration-09.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-09.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-09.html)
- [variable-declaration-10.html](https://wpt.fyi/results/css/css-variables/variable-declaration-10.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-10.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-10.html)
- [variable-declaration-11.html](https://wpt.fyi/results/css/css-variables/variable-declaration-11.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-11.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-11.html)
- [variable-declaration-12.html](https://wpt.fyi/results/css/css-variables/variable-declaration-12.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-12.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-12.html)
- [variable-declaration-13.html](https://wpt.fyi/results/css/css-variables/variable-declaration-13.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-13.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-13.html)
- [variable-declaration-22.html](https://wpt.fyi/results/css/css-variables/variable-declaration-22.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-22.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-22.html)

<a id="ref-for-custom-property②③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The syntax of the fallback, like that of [custom properties](#custom-property), allows commas. For example, var(--foo, red, blue) defines a fallback of red, blue; that is, anything between the first comma and the end of the function is considered a fallback value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-abd63bac"></a> The fallback value allows for some types of defensive coding. For example, an author may create a component intended to be included in a larger application, and use variables to style it so that it’s easy for the author of the larger application to theme the component to match the rest of the app.
>
> Without fallback, the app author must supply a value for every variable that your component uses. With fallback, the component author can supply defaults, so the app author only needs to supply values for the variables they wish to override.
>
> ```text
> /* In the component’s style: */
> .component .header {
>   color: var(--header-color, blue);
> }
> .component .text {
>   color: var(--text-color, black);
> }
> 
> /* In the larger application’s style: */
> .component {
>   --text-color: #080;
>   /* header-color isn’t set,
>      and so remains blue,
>      the fallback value */
> }
> ```
<a id="ref-for-funcdef-var②⑤"></a>

<a id="ref-for-substitute-a-var①"></a>

If a property contains one or more [var()](#funcdef-var) functions, and those functions are syntactically valid, the entire property’s grammar must be assumed to be valid at parse time. It is only syntax-checked at computed-value time, after <a id="ref-for-funcdef-var②⑥"></a>var() functions have been [substituted](#substitute-a-var).

Tests

- [variable-reference-18.html](https://wpt.fyi/results/css/css-variables/variable-reference-18.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-18.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-18.html)
- [variable-reference-19.html](https://wpt.fyi/results/css/css-variables/variable-reference-19.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-19.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-19.html)
- [variable-reference-30.html](https://wpt.fyi/results/css/css-variables/variable-reference-30.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-30.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-30.html)

To <a id="substitute-a-var"></a>substitute a var() in a property’s value:

1.  <a id="ref-for-not-animatable"></a>

    <a id="ref-for-animation-tainted③"></a>

    <a id="ref-for-funcdef-var②⑦"></a>

    <a id="ref-for-custom-property②④"></a>

    If the [custom property](#custom-property) named by the first argument to the [var()](#funcdef-var) function is [animation-tainted](#animation-tainted), and the <a id="ref-for-funcdef-var②⑧"></a>var() function is being used in a property that is [not animatable](https://www.w3.org/TR/web-animations-1/#not-animatable), treat the <a id="ref-for-custom-property②⑤"></a>custom property as having its initial value for the rest of this algorithm.

2.  <a id="ref-for-funcdef-var②⑨"></a>

    <a id="ref-for-custom-property②⑥"></a>

    If the value of the [custom property](#custom-property) named by the first argument to the [var()](#funcdef-var) function is anything but the initial value, replace the <a id="ref-for-funcdef-var③⓪"></a>var() function by the value of the corresponding <a id="ref-for-custom-property②⑦"></a>custom property.

3.  <a id="ref-for-substitute-a-var②"></a>

    <a id="ref-for-funcdef-var③①"></a>

    Otherwise, if the [var()](#funcdef-var) function has a fallback value as its second argument, replace the <a id="ref-for-funcdef-var③②"></a>var() function by the fallback value. If there are any <a id="ref-for-funcdef-var③③"></a>var() references in the fallback, [substitute](#substitute-a-var) them as well.

4.  <a id="ref-for-invalid-at-computed-value-time④"></a>

    <a id="ref-for-funcdef-var③④"></a>

    Otherwise, the property containing the [var()](#funcdef-var) function is [invalid at computed-value time](#invalid-at-computed-value-time).

    <a id="ref-for-invalid-at-computed-value-time⑤"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Other things can also make a property [invalid at computed-value time](#invalid-at-computed-value-time).

Tests

- [css-variable-change-style-001.html](https://wpt.fyi/results/css/css-variables/css-variable-change-style-001.html) [(live test)](http://wpt.live/css/css-variables/css-variable-change-style-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/css-variable-change-style-001.html)
- [css-variable-change-style-002.html](https://wpt.fyi/results/css/css-variables/css-variable-change-style-002.html) [(live test)](http://wpt.live/css/css-variables/css-variable-change-style-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/css-variable-change-style-002.html)
- [variable-declaration-01.html](https://wpt.fyi/results/css/css-variables/variable-declaration-01.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-01.html)
- [variable-declaration-02.html](https://wpt.fyi/results/css/css-variables/variable-declaration-02.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-02.html)
- [variable-declaration-03.html](https://wpt.fyi/results/css/css-variables/variable-declaration-03.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-03.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-03.html)
- [variable-declaration-04.html](https://wpt.fyi/results/css/css-variables/variable-declaration-04.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-04.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-04.html)
- [variable-declaration-05.html](https://wpt.fyi/results/css/css-variables/variable-declaration-05.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-05.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-05.html)
- [variable-generated-content-dynamic-001.html](https://wpt.fyi/results/css/css-variables/variable-generated-content-dynamic-001.html) [(live test)](http://wpt.live/css/css-variables/variable-generated-content-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-generated-content-dynamic-001.html)
- [variable-presentation-attribute.html](https://wpt.fyi/results/css/css-variables/variable-presentation-attribute.html) [(live test)](http://wpt.live/css/css-variables/variable-presentation-attribute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-presentation-attribute.html)
- [variable-reference-01.html](https://wpt.fyi/results/css/css-variables/variable-reference-01.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-01.html)
- [variable-reference-02.html](https://wpt.fyi/results/css/css-variables/variable-reference-02.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-02.html)
- [variable-reference-03.html](https://wpt.fyi/results/css/css-variables/variable-reference-03.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-03.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-03.html)
- [variable-reference-04.html](https://wpt.fyi/results/css/css-variables/variable-reference-04.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-04.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-04.html)
- [variable-reference-05.html](https://wpt.fyi/results/css/css-variables/variable-reference-05.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-05.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-05.html)
- [variable-reference-12.html](https://wpt.fyi/results/css/css-variables/variable-reference-12.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-12.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-12.html)
- [variable-reference-16.html](https://wpt.fyi/results/css/css-variables/variable-reference-16.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-16.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-16.html)
- [variable-reference-40.html](https://wpt.fyi/results/css/css-variables/variable-reference-40.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-40.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-40.html)
- [variable-reference-refresh.html](https://wpt.fyi/results/css/css-variables/variable-reference-refresh.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-refresh.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-refresh.html)
- [variable-substitution-background-properties.html](https://wpt.fyi/results/css/css-variables/variable-substitution-background-properties.html) [(live test)](http://wpt.live/css/css-variables/variable-substitution-background-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-substitution-background-properties.html)
- [variable-substitution-basic.html](https://wpt.fyi/results/css/css-variables/variable-substitution-basic.html) [(live test)](http://wpt.live/css/css-variables/variable-substitution-basic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-substitution-basic.html)
- [variable-substitution-filters.html](https://wpt.fyi/results/css/css-variables/variable-substitution-filters.html) [(live test)](http://wpt.live/css/css-variables/variable-substitution-filters.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-substitution-filters.html)
- [variable-substitution-replaced-size.html](https://wpt.fyi/results/css/css-variables/variable-substitution-replaced-size.html) [(live test)](http://wpt.live/css/css-variables/variable-substitution-replaced-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-substitution-replaced-size.html)
- [variable-substitution-shadow-properties.html](https://wpt.fyi/results/css/css-variables/variable-substitution-shadow-properties.html) [(live test)](http://wpt.live/css/css-variables/variable-substitution-shadow-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-substitution-shadow-properties.html)
- [variable-substitution-variable-declaration.html](https://wpt.fyi/results/css/css-variables/variable-substitution-variable-declaration.html) [(live test)](http://wpt.live/css/css-variables/variable-substitution-variable-declaration.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-substitution-variable-declaration.html)

CSSOM

- [variable-reference-cssom.html](https://wpt.fyi/results/css/css-variables/variable-reference-cssom.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-cssom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-cssom.html)

------------------------------------------------------------------------

<a id="ref-for-substitute-a-var③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that [var() substitution](#substitute-a-var) takes place at the level of CSS tokens [\[css-syntax-3\]](#biblio-css-syntax-3), not at a textual level; you can’t build up a single token where part of it is provided by a variable:
>
> ```text
> .foo {
>   --gap: 20;
>   margin-top: var(--gap)px;
> }
> ```
>
> <a id="ref-for-propdef-margin-top①"></a>
>
> <a id="ref-for-funcdef-calc"></a>
>
> This is <em>not</em> equivalent to setting [margin-top: 20px;](https://www.w3.org/TR/css-box-4/#propdef-margin-top) (a length). Instead, it’s equivalent to <a id="ref-for-propdef-margin-top②"></a>margin-top: 20 px; (a number followed by an ident), which is simply an invalid value for the <a id="ref-for-propdef-margin-top③"></a>margin-top property. Note, though, that [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) can be used to validly achieve the same thing, like so:
>
> ```text
> .foo {
>   --gap: 20;
>   margin-top: calc(var(--gap) * 1px);
> }
> ```
Tests

- [variable-declaration-14.html](https://wpt.fyi/results/css/css-variables/variable-declaration-14.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-14.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-14.html)
- [variable-declaration-53.html](https://wpt.fyi/results/css/css-variables/variable-declaration-53.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-53.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-53.html)
- [variable-declaration-54.html](https://wpt.fyi/results/css/css-variables/variable-declaration-54.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-54.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-54.html)
- [variable-declaration-55.html](https://wpt.fyi/results/css/css-variables/variable-declaration-55.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-55.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-55.html)
- [variable-reference-15.html](https://wpt.fyi/results/css/css-variables/variable-reference-15.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-15.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-15.html)
- [variable-reference-without-whitespace.html](https://wpt.fyi/results/css/css-variables/variable-reference-without-whitespace.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-without-whitespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-without-whitespace.html)

<a id="ref-for-funcdef-var③⑤"></a>

<a id="ref-for-substitute-a-var④"></a>

<a id="ref-for-invalid-at-computed-value-time⑥"></a>

[var()](#funcdef-var) functions are [substituted](#substitute-a-var) at computed-value time. If a declaration, once all <a id="ref-for-funcdef-var③⑥"></a>var() functions are substituted in, does not match its declared grammar, the declaration is [invalid at computed-value time](#invalid-at-computed-value-time).

Tests

- [variable-declaration-16.html](https://wpt.fyi/results/css/css-variables/variable-declaration-16.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-16.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-16.html)
- [variable-declaration-17.html](https://wpt.fyi/results/css/css-variables/variable-declaration-17.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-17.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-17.html)
- [variable-declaration-18.html](https://wpt.fyi/results/css/css-variables/variable-declaration-18.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-18.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-18.html)
- [variable-declaration-19.html](https://wpt.fyi/results/css/css-variables/variable-declaration-19.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-19.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-19.html)
- [variable-declaration-21.html](https://wpt.fyi/results/css/css-variables/variable-declaration-21.html) [(live test)](http://wpt.live/css/css-variables/variable-declaration-21.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-declaration-21.html)
- [variable-transitions-transition-property-all-before-value.html](https://wpt.fyi/results/css/css-variables/variable-transitions-transition-property-all-before-value.html) [(live test)](http://wpt.live/css/css-variables/variable-transitions-transition-property-all-before-value.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-transitions-transition-property-all-before-value.html)
- [variable-transitions-value-before-transition-property-all.html](https://wpt.fyi/results/css/css-variables/variable-transitions-value-before-transition-property-all.html) [(live test)](http://wpt.live/css/css-variables/variable-transitions-value-before-transition-property-all.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-transitions-value-before-transition-property-all.html)

<a id="ref-for-funcdef-var③⑦"></a>

<a id="ref-for-css-wide-keywords②"></a>

<a id="ref-for-specified-value"></a>

If a declaration, once all [var()](#funcdef-var) functions are substituted in, contains only a [CSS-wide keyword](https://www.w3.org/TR/css-values-4/#css-wide-keywords) (and possibly whitespace), its value is determined as if that keyword were its [specified value](https://www.w3.org/TR/css-cascade-5/#specified-value) all along.

Tests

- [wide-keyword-fallback.html](https://wpt.fyi/results/css/css-variables/wide-keyword-fallback.html) [(live test)](http://wpt.live/css/css-variables/wide-keyword-fallback.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/wide-keyword-fallback.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6ec8308f"></a> For example, the following usage is fine from a syntax standpoint, but results in nonsense when the variable is substituted in:
>
> ```text
> :root { --looks-valid: 20px; }
> p { background-color: var(--looks-valid); }
> ```
>
> <a id="ref-for-propdef-background-color①"></a>
>
> <a id="ref-for-valdef-color-transparent"></a>
>
> Since 20px is an invalid value for [background-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color), this instance of the property computes to [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent) (the initial value for <a id="ref-for-propdef-background-color②"></a>background-color) instead.
>
> <a id="ref-for-propdef-color"></a>
>
> If the property was one that’s inherited by default, such as [color](https://www.w3.org/TR/css-color-4/#propdef-color), it would compute to the inherited value rather than the initial value.

<a id="ref-for-funcdef-var③⑧"></a>

<a id="ref-for-css-wide-keywords③"></a>

<a id="ref-for-custom-property②⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ee865e1b"></a> While a [var()](#funcdef-var) function can’t get a [CSS-wide keyword](https://www.w3.org/TR/css-values-4/#css-wide-keywords) from the [custom property](#custom-property) itself—if you tried to specify that, like --foo: initial;, it would just trigger [explicit defaulting](https://www.w3.org/TR/css-cascade-4/#defaulting-keywords) for the custom property—it can have a <a id="ref-for-css-wide-keywords④"></a>CSS-wide keyword in its fallback:
>
> ```text
> p { color: var(--does-not-exist, initial); }
> ```
>
> <a id="ref-for-invalid-at-computed-value-time⑦"></a>
>
> <a id="ref-for-funcdef-var③⑨"></a>
>
> <a id="ref-for-valdef-all-initial①"></a>
>
> <a id="ref-for-propdef-color①"></a>
>
> In the above code, if the --does-not-exist property didn’t exist or is [invalid at computed-value time](#invalid-at-computed-value-time), the [var()](#funcdef-var) will instead substitute in the [initial](https://www.w3.org/TR/css-cascade-5/#valdef-all-initial) keyword, making the property behave as if it was originally [color: initial](https://www.w3.org/TR/css-color-4/#propdef-color). This will make it take on the document’s initial <a id="ref-for-propdef-color②"></a>color value, rather than defaulting to inheritance, as it would if there were no fallback.

### <a id="invalid-variables"></a>3.1.  Invalid Variables

<a id="ref-for-custom-property②⑨"></a>

<a id="ref-for-guaranteed-invalid-value⑦"></a>

<a id="ref-for-funcdef-var④⓪"></a>

<a id="ref-for-invalid-at-computed-value-time⑧"></a>

When a [custom property’s](#custom-property) value is the [guaranteed-invalid value](#guaranteed-invalid-value), [var()](#funcdef-var) functions cannot use it for substitution. Attempting to do so makes the declaration [invalid at computed-value time](#invalid-at-computed-value-time), unless a valid fallback is specified.

<a id="ref-for-funcdef-var④①"></a>

<a id="ref-for-custom-property③⓪"></a>

<a id="ref-for-guaranteed-invalid-value⑧"></a>

A declaration can be <a id="invalid-at-computed-value-time"></a>invalid at computed-value time if it contains a [var()](#funcdef-var) that references a [custom property](#custom-property) with the [guaranteed-invalid value](#guaranteed-invalid-value), as explained above, or if it uses a valid <a id="ref-for-custom-property③①"></a>custom property, but the property value, after substituting its <a id="ref-for-funcdef-var④②"></a>var() functions, is invalid. When this happens, the computed value is one of the following depending on the property’s type:

<a id="ref-for-custom-property③②"></a>

The property is a non-registered [custom property](#custom-property)

<a id="ref-for-universal-syntax-definition"></a>

<a id="ref-for-registered-custom-property"></a>

The property is a [registered custom property](https://drafts.css-houdini.org/css-properties-values-api-1/#registered-custom-property) with [universal syntax](https://drafts.css-houdini.org/css-properties-values-api-1/#universal-syntax-definition)

<a id="ref-for-guaranteed-invalid-value⑨"></a>

The computed value is the [guaranteed-invalid value](#guaranteed-invalid-value).

Otherwise

<a id="ref-for-valdef-all-unset"></a>

Either the property’s inherited value or its initial value depending on whether the property is inherited or not, respectively, as if the property’s value had been specified as the [unset](https://www.w3.org/TR/css-cascade-5/#valdef-all-unset) keyword.

Tests

- [variables-substitute-guaranteed-invalid.html](https://wpt.fyi/results/css/css-variables/variables-substitute-guaranteed-invalid.html) [(live test)](http://wpt.live/css/css-variables/variables-substitute-guaranteed-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variables-substitute-guaranteed-invalid.html)

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
> <a id="ref-for-propdef-background-color③"></a>
>
> <a id="ref-for-custom-property③③"></a>
>
> <a id="ref-for-funcdef-var④③"></a>
>
> the \<p\> elements will have transparent backgrounds (the initial value for [background-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color)), rather than red backgrounds. The same would happen if the [custom property](#custom-property) itself was unset, or contained an invalid [var()](#funcdef-var) function.
>
> <a id="ref-for-propdef-background-color④"></a>
>
> Note the difference between this and what happens if the author had just written [background-color: 20px](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color) directly in their stylesheet - that would be a normal syntax error, which would cause the rule to be discarded, so the <a id="ref-for-propdef-background-color⑤"></a>background-color: red rule would be used instead.

<a id="ref-for-invalid-at-computed-value-time⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [invalid at computed-value time](#invalid-at-computed-value-time) concept exists because variables can’t "fail early" like other syntax errors can, so by the time the user agent realizes a property value is invalid, it’s already thrown away the other cascaded values.

### <a id="variables-in-shorthands"></a>3.2.  Variables in Shorthand Properties

<a id="ref-for-funcdef-var④④"></a>

<a id="ref-for-shorthand-property"></a>

[var()](#funcdef-var) functions produce some complications when parsing [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) into their component longhands, and when serializing <a id="ref-for-shorthand-property①"></a>shorthand properties <em>from</em> their component longhands.

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-funcdef-var④⑤"></a>

<a id="ref-for-longhand"></a>

<a id="ref-for-substitute-a-var⑤"></a>

If a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) contains a [var()](#funcdef-var) function in its value, the [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) it’s associated with must instead be filled in with a special, unobservable-to-authors <a id="pending-substitution-value"></a>pending-substitution value that indicates the shorthand contains a variable, and thus the longhand’s value can’t be determined until variables are [substituted](#substitute-a-var).

<a id="ref-for-funcdef-var④⑥"></a>

This value must then be cascaded as normal, and at computed-value time, after [var()](#funcdef-var) functions are finally substituted in, the shorthand must be parsed and the longhands must be given their appropriate values at that point.

Tests

- [variable-reference-36.html](https://wpt.fyi/results/css/css-variables/variable-reference-36.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-36.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-36.html)
- [variable-reference-37.html](https://wpt.fyi/results/css/css-variables/variable-reference-37.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-37.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-37.html)
- [variable-reference-38.html](https://wpt.fyi/results/css/css-variables/variable-reference-38.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-38.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-38.html)
- [variable-substitution-shorthands.html](https://wpt.fyi/results/css/css-variables/variable-substitution-shorthands.html) [(live test)](http://wpt.live/css/css-variables/variable-substitution-shorthands.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-substitution-shorthands.html)
- [vars-background-shorthand-001.html](https://wpt.fyi/results/css/css-variables/vars-background-shorthand-001.html) [(live test)](http://wpt.live/css/css-variables/vars-background-shorthand-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/vars-background-shorthand-001.html)
- [vars-font-shorthand-001.html](https://wpt.fyi/results/css/css-variables/vars-font-shorthand-001.html) [(live test)](http://wpt.live/css/css-variables/vars-font-shorthand-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/vars-font-shorthand-001.html)

<a id="ref-for-funcdef-var④⑦"></a>

<a id="ref-for-longhand①"></a>

<a id="ref-for-cascade"></a>

<a id="ref-for-shorthand-property③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When a shorthand is written without a [var()](#funcdef-var), it is parsed and separated out into its component [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) at parse time; the longhands then participate in the [cascade](https://www.w3.org/TR/css-cascade-6/#cascade), with the [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) more or less discarded. When the shorthand contains a <a id="ref-for-funcdef-var④⑧"></a>var(), however, this can’t be done, as the <a id="ref-for-funcdef-var④⑨"></a>var() could be substituted with anything.

<a id="ref-for-pending-substitution-value"></a>

[Pending-substitution values](#pending-substitution-value) must be serialized as the empty string, if an API allows them to be observed.

Tests

- [variable-definition-border-shorthand-serialize.html](https://wpt.fyi/results/css/css-variables/variable-definition-border-shorthand-serialize.html) [(live test)](http://wpt.live/css/css-variables/variable-definition-border-shorthand-serialize.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-definition-border-shorthand-serialize.html)
- [vars-border-shorthand-serialize.html](https://wpt.fyi/results/css/css-variables/vars-border-shorthand-serialize.html) [(live test)](http://wpt.live/css/css-variables/vars-border-shorthand-serialize.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/vars-border-shorthand-serialize.html)

------------------------------------------------------------------------

<a id="ref-for-shorthand-property④"></a>

<a id="ref-for-longhand②"></a>

[Shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) are serialized by gathering the values of their component [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand), and synthesizing a value that will parse into the same set of values.

<a id="ref-for-longhand③"></a>

<a id="ref-for-shorthand-property⑤"></a>

<a id="ref-for-pending-substitution-value①"></a>

<a id="ref-for-funcdef-var⑤⓪"></a>

If all of the component [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) for a given [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) are [pending-substitution values](#pending-substitution-value) from the same original shorthand value, the <a id="ref-for-shorthand-property⑥"></a>shorthand property must serialize to that original ([var()](#funcdef-var)-containing) value.

<a id="ref-for-longhand④"></a>

<a id="ref-for-shorthand-property⑦"></a>

<a id="ref-for-pending-substitution-value②"></a>

<a id="ref-for-funcdef-var⑤①"></a>

Otherwise, if any of the component [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) for a given [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) are [pending-substitution values](#pending-substitution-value), or contain [var()](#funcdef-var) functions of their own that have not yet been substituted, the <a id="ref-for-shorthand-property⑧"></a>shorthand property must serialize to the empty string.

### <a id="long-variables"></a>3.3.  Safely Handling Overly-Long Variables

<a id="ref-for-funcdef-var⑤②"></a>

Naively implemented, [var()](#funcdef-var) functions can be used in a variation of the "billion laughs attack":

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

<a id="ref-for-funcdef-var⑤③"></a>

<a id="ref-for-invalid-at-computed-value-time①⓪"></a>

To avoid this sort of attack, UAs must impose a UA-defined limit on the allowed length of the token stream that a [var()](#funcdef-var) function expands into. If a <a id="ref-for-funcdef-var⑤④"></a>var() would expand into a longer token stream than this limit, it instead makes the property it’s expanding into [invalid at computed-value time](#invalid-at-computed-value-time).

Tests

- [variable-exponential-blowup.html](https://wpt.fyi/results/css/css-variables/variable-exponential-blowup.html) [(live test)](http://wpt.live/css/css-variables/variable-exponential-blowup.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-exponential-blowup.html)

This specification does not define what size limit should be imposed. However, since there are valid use-cases for custom properties that contain a kilobyte or more of text, it’s recommended that the limit be set relatively high.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The general principle that UAs are allowed to violate standards due to resource constraints is still generally true here; a UA might, separately, have limits on how long of a custom property they can support, or how large of an identifier they can support. This section calls out this attack specifically because of its long history, and the fact that it can be done without any of the pieces <em>seeming</em> to be too large on first inspection.

## <a id="apis"></a>4.  APIs

<a id="ref-for-custom-property③④"></a>

<a id="ref-for-cssstyledeclaration-declarations"></a>

<a id="ref-for-css-declaration-case-sensitive-flag"></a>

All [custom property](#custom-property) [declarations](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-declarations) have the [case-sensitive flag](https://www.w3.org/TR/cssom-1/#css-declaration-case-sensitive-flag) set.

Tests

- [variable-definition.html](https://wpt.fyi/results/css/css-variables/variable-definition.html) [(live test)](http://wpt.live/css/css-variables/variable-definition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-definition.html)
- [variable-invalidation.html](https://wpt.fyi/results/css/css-variables/variable-invalidation.html) [(live test)](http://wpt.live/css/css-variables/variable-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-invalidation.html)

<a id="ref-for-dom-cssstyledeclaration-getpropertyvalue"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Custom properties do not appear on a CSSStyleDeclaration object in camel-cased form, because their names may have both upper and lower case letters which indicate distinct custom properties. The sort of text transformation that automatic camel-casing performs is incompatible with this. They can still be accessed by their proper name via [getPropertyValue()](https://www.w3.org/TR/cssom-1/#dom-cssstyledeclaration-getpropertyvalue)/etc.

### <a id="serializing-custom-props"></a>4.1.  Serializing Custom Properties

Custom property names must be serialized as the exact code point sequence provided by the author, including not altering the case.

<a id="ref-for-ascii-case-insensitive②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For non-custom properties, property names are restricted to the ASCII range and are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive), so implementations typically serialize the name lowercased.

<a id="ref-for-custom-property③⑤"></a>

Specified values of [custom properties](#custom-property) must be serialized <em>exactly as specified by the author</em>. Simplifications that might occur in other properties, such as dropping comments, normalizing whitespace, reserializing numeric tokens from their value, etc., must not occur.

<a id="ref-for-custom-property③⑥"></a>

<a id="ref-for-funcdef-var⑤⑤"></a>

Computed values of [custom properties](#custom-property) must similarly be serialized <em>exactly as specified by the author</em>, save for the replacement of any [var()](#funcdef-var) functions.

Tests

- [variable-reference-shorthands-cssom.html](https://wpt.fyi/results/css/css-variables/variable-reference-shorthands-cssom.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-shorthands-cssom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-shorthands-cssom.html)
- [variable-reference-variable.html](https://wpt.fyi/results/css/css-variables/variable-reference-variable.html) [(live test)](http://wpt.live/css/css-variables/variable-reference-variable.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-variables/variable-reference-variable.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-55f61182"></a> For example, given the following properties:
>
> ```text
> --y: /* baz */;
> --x: /* foo */ var(--y) /* bar */;
> ```
>
> the serialization of the specified value of --x must be `"/* foo */ var(--y) /* bar */"`, while the serialization of the computed value of --x must be `"/* foo */ /* baz */ /* bar */"`.
>
> (Note that the leading whitespace on the value is automatically trimmed by the CSS parser; it’s not preserved here.)

> <strong data-conversion-semantic="note">Note</strong>
>
> This requirement exists because authors sometimes store non-CSS information in custom properties, and "normalizing" this information can change it in ways that break author code.
>
> For example, storing a UUID in a custom property, like --uuid: 12345678-12e3-8d9b-a456-426614174000, requires the UUID to be echoed back out as written when it’s accessed by script.
>
> This value is technically parsed by CSS as a series of adjacent numbers and dimensions. In particular, the segment "-12e3" parses as a number, equal to -12000. Reserializing it in that form, as required by CSSOM in other contexts, would fatally break the author’s use of the value.

## <a id="changes"></a>5.  Changes

### <a id="changes-20211111"></a>5.1.  Changes Since the 11 November 2021 CR Draft

- Clarified that custom properties apply all pseudo-elements (including those with restricted property lists)

- Added example to illustrate issues with combining characters, ligatures, etc

- Strengthened wording around similar-appearing variable names that use distinct codepoint sequences

- Clarified an example by using more visualy distinct languages as examples (English and Greek)

- Split Security and Privacy into separate sections

### <a id="changes-20151203"></a>5.2.  Changes Since the 03 December 2015 CR

- <a id="ref-for-typedef-declaration-value③"></a>

  Now that [\[css-syntax-3\]](#biblio-css-syntax-3) auto-trims whitespace from declaration values, made [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value) optional in the custom property grammar, so that empty variables are still allowed. ([Issue 774](https://github.com/w3c/csswg-drafts/issues/774#issuecomment-434571254))

- <a id="ref-for-funcdef-var⑤⑥"></a>

  Similarly, made empty fallbacks valid in [var()](#funcdef-var).

- The '-' property is reserved for future use by CSS.

- Added concept of "animation-tainted", to prevent non-animatable properties from using a variable to smuggle in some animatability.

- <a id="ref-for-guaranteed-invalid-value①⓪"></a>

  Defined the [guaranteed-invalid value](#guaranteed-invalid-value) to make the initial value of custom properties and the result of cycles or substitution failure more straightforward, and allow failure to propagate thru substitutions until finally intercepted by a fallback.

- <a id="ref-for-invalid-at-computed-value-time①①"></a>

  Defined that cycles trigger [invalid at computed-value time](#invalid-at-computed-value-time) behavior.

- Allowed variables to resolve to a CSS-wide keyword (only possible by providing it as a fallback).

- <a id="ref-for-registered-custom-property①"></a>

  <a id="ref-for-invalid-at-computed-value-time①②"></a>

  Clarified that [registered custom properties](https://drafts.css-houdini.org/css-properties-values-api-1/#registered-custom-property) act like non-custom properties when they’re [invalid at computed-value time](#invalid-at-computed-value-time).

- <a id="ref-for-funcdef-var⑤⑦"></a>

  Made longhands with [var()](#funcdef-var)s also trigger their shorthands to be unserializable, like longhands with pending-substitution values already did.

- Required UAs to defend against exponential substitution attacks.

- Defined how to serialize the <em>values</em> of custom properties (previously, only the property name’s serialization was specified).

### <a id="changes-20140506"></a>5.3.  Changes since the May 6 2014 Last Call Working Draft

- Serialization of longhands when shorthand uses a variable was defined.

- Link to DOM’s definition of "case-sensitive".

- <a id="ref-for-lang-pseudo"></a>

  Added example of using variables with [:lang()](https://www.w3.org/TR/selectors-4/#lang-pseudo) to do simple i18n.

- <a id="ref-for-funcdef-var⑤⑧"></a>

  Clarified that usage of [var()](#funcdef-var) in a custom property must be valid per the <a id="ref-for-funcdef-var⑤⑨"></a>var() grammar.

## <a id="acks"></a>6.  Acknowledgments

Many thanks to several people in the CSS Working Group for keeping the dream of variables alive over the years, particularly Daniel Glazman and David Hyatt. Thanks to multiple people on the mailing list for helping contribute to the development of this incarnation of variables, particularly Brian Kardell, David Baron, François Remy, Roland Steiner, and Shane Stephens.

## <a id="privacy"></a>7. Privacy Considerations

This specification defines a purely author-level mechanism for passing styling information around within a page they control. As such, there are no new privacy considerations.

## <a id="security"></a>8. Security Considerations

<a id="ref-for-funcdef-var⑥⓪"></a>

[§ 3.3 Safely Handling Overly-Long Variables](#long-variables) calls out a long-standing Denial-of-Service attack that can be mounted against "macro-expansion"-like mechanisms, such as the [var()](#funcdef-var) function, and mandates a defense against that attack.

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

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](http://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

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

- [--\*](#propdef-), in § 2
- [animation-tainted](#animation-tainted), in § 2
- [custom property](#custom-property), in § 2
- [\<custom-property-name\>](#typedef-custom-property-name), in § 2
- [guaranteed-invalid value](#guaranteed-invalid-value), in § 2.2
- [invalid at computed-value time](#invalid-at-computed-value-time), in § 3.1
- [pending-substitution value](#pending-substitution-value), in § 3.2
- [substitute](#substitute-a-var), in § 3
- [substitute a var()](#substitute-a-var), in § 3
- [var()](#funcdef-var), in § 3
- [var() substitution](#substitute-a-var), in § 3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-animations-1\] defines the following terms:
  - <a id="term-for-at-ruledef-keyframes"></a>@keyframes
- \[css-backgrounds-3\] defines the following terms:
  - <a id="term-for-propdef-background"></a>background
  - <a id="term-for-propdef-background-color"></a>background-color
- \[css-box-4\] defines the following terms:
  - <a id="term-for-propdef-margin-top"></a>margin-top
- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-propdef-all"></a>all
  - <a id="term-for-valdef-all-initial"></a>initial
  - <a id="term-for-longhand"></a>longhand property
  - <a id="term-for-shorthand-property"></a>shorthand
  - <a id="term-for-shorthand-property①"></a>shorthand property
  - <a id="term-for-specified-value"></a>specified value
  - <a id="term-for-valdef-all-unset"></a>unset
- \[css-cascade-6\] defines the following terms:
  - <a id="term-for-cascade"></a>cascade
- \[css-color-4\] defines the following terms:
  - <a id="term-for-propdef-color"></a>color
  - <a id="term-for-valdef-color-transparent"></a>transparent
- \[css-conditional-3\] defines the following terms:
  - <a id="term-for-at-ruledef-media"></a>@media
- \[css-fonts-4\] defines the following terms:
  - <a id="term-for-propdef-font-size"></a>font-size
- \[css-properties-values-api-1\] defines the following terms:
  - <a id="term-for-registered-custom-property"></a>registered custom property
  - <a id="term-for-universal-syntax-definition"></a>universal syntax definition
- \[css-syntax-3\] defines the following terms:
  - <a id="term-for-tokendef-close-paren"></a>\<)-token\>
  - <a id="term-for-tokendef-close-square"></a>\<\]-token\>
  - <a id="term-for-typedef-bad-string-token"></a>\<bad-string-token\>
  - <a id="term-for-typedef-bad-url-token"></a>\<bad-url-token\>
  - <a id="term-for-typedef-declaration-value"></a>\<declaration-value\>
  - <a id="term-for-typedef-delim-token"></a>\<delim-token\>
  - <a id="term-for-typedef-semicolon-token"></a>\<semicolon-token\>
  - <a id="term-for-tokendef-close-curly"></a>\<}-token\>
- \[css-values-4\] defines the following terms:
  - <a id="term-for-comb-comma"></a>,
  - <a id="term-for-typedef-dashed-ident"></a>\<dashed-ident\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-funcdef-calc"></a>calc()
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-em"></a>em
  - <a id="term-for-css-css-identifier"></a>identifier
- \[cssom-1\] defines the following terms:
  - <a id="term-for-css-declaration-case-sensitive-flag"></a>case-sensitive flag
  - <a id="term-for-cssstyledeclaration-declarations"></a>declarations
  - <a id="term-for-dom-cssstyledeclaration-getpropertyvalue"></a>getPropertyValue(property)
- \[INFRA\] defines the following terms:
  - <a id="term-for-ascii-case-insensitive"></a>ascii case-insensitive
  - <a id="term-for-string-is"></a>identical to
- \[selectors-4\] defines the following terms:
  - <a id="term-for-lang-pseudo"></a>:lang()
- \[web-animations-1\] defines the following terms:
  - <a id="term-for-not-animatable"></a>not animatable

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
Dean Jackson; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
David Baron; Elika Etemad; Chris Lilley. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-properties-values-api-1"></a>\[CSS-PROPERTIES-VALUES-API-1\]  
Tab Atkins Jr.; et al. [CSS Properties and Values API Level 1](https://www.w3.org/TR/css-properties-values-api-1/). 13 October 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-properties-values-api-1&#x2F;](https://www.w3.org/TR/css-properties-values-api-1/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 16 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 21 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 18 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

### <a id="informative"></a>Informative References

<a id="biblio-charmod-norm"></a>\[CHARMOD-NORM\]  
Addison Phillips; et al. [Character Model for the World Wide Web: String Matching](https://www.w3.org/TR/charmod-norm/). 11 August 2021. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;charmod-norm&#x2F;](https://www.w3.org/TR/charmod-norm/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 26 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-6"></a>\[CSS-CASCADE-6\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://www.w3.org/TR/css-cascade-6/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-6&#x2F;](https://www.w3.org/TR/css-cascade-6/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 15 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-extensions"></a>\[CSS-EXTENSIONS\]  
Tab Atkins Jr.. [CSS Extensions](https://drafts.csswg.org/css-extensions/). ED. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-extensions&#x2F;](https://drafts.csswg.org/css-extensions/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

## <a id="property-index"></a>Property Index

| Name | Value                  | Initial                      | Applies to                                                                            | Inh. | %ages | Anim­ation type | Canonical order | Com­puted value                                                              |
|------|------------------------|------------------------------|---------------------------------------------------------------------------------------|------|-------|----------------|-----------------|-----------------------------------------------------------------------------|
| --\* | \<declaration-value\>? | the guaranteed-invalid value | all elements and all pseudo-elements (including those with restricted property lists) | yes  | n/a   | discrete       | per grammar     | specified value with variables substituted, or the guaranteed-invalid value |
