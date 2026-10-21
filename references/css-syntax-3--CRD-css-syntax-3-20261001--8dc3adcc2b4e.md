Attribution and reformatting notice added for Surgeist on 2026-10-20

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Syntax Module Level 3](https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply. The capture’s full original notice and links remain below.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md#css-syntax-3-candidate-recommendation-draft--2026-10-20).

# Source provenance

Title: CSS Syntax Module Level 3

Source snapshot: https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/

Retrieved: 2026-10-20. The captured page states W3C Candidate Recommendation Draft, 1 October 2026. This is a dated W3C publication; the exact byte hash identifies the captured HTML.

Captured HTML SHA-256: 8dc3adcc2b4e8fd8fa73c0be85cec67f006874ab42010cdf4528af4aa83f2878

Captured HTML revision metadata: `4726b9562bc1a9679b4aacb68318e9b19c889fa8`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Representation notes:

- Full format conversion of the captured HTML body, including status, metadata, bibliography, indexes, examples, test references, and legal notice. Script and style elements are omitted and were not executed.
- All source body IDs are retained, including those inside preformatted blocks, which are relocated immediately before those blocks. Fragment-only links stay local only when their exact captured target exists; other links remain upstream.
- All 1 tables have readable Markdown layouts. All tables have ordinary row/column layouts. Synthetic Field/Definition or Column N headings are non-normative. Source row headers remain bold; native HTML header/accessibility semantics are not expressible in GFM.
- Preformatted examples retain literal text. Single-line table examples use semantic inline code. Compiler-generated hyperlinks inside fenced code blocks are omitted while their IDs and visible code text are retained; other source links are preserved. Small semantic code, variable, emphasis, subscript, and superscript HTML remains where Markdown notation would alter text.
- 28 inline SVG diagrams are retained as local passive SVG assets with their source geometry, exact-case names and visible text; railroad diagrams carry the capture’s railroad stylesheet with light-theme values resolved. Each image is followed by its diagram text. Supporting assets are not reference documents.
- Conversion uses Pandoc 3.1.11.1 with focused Python/lxml preparation and a semantic Lua filter; the parsed GFM output is checked against the captured HTML. The [conversion report](CONVERSION-REPORT.md#css-syntax-3-candidate-recommendation-draft--2026-10-20) records provenance and check boundaries.
- This edition is the selected CSS Syntax 3 reference, superseding the [24 December 2021 Candidate Recommendation Draft](css-syntax-3--CRD-css-syntax-3-20211224--413b27acb4e1.md). The [source-edition mapping](SOURCE-EDITIONS.md#css-syntax-3-edition) records the selection and the citations still being moved from the earlier edition.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Syntax Module Level 3

<a id="w3c-state"></a>[W3C Candidate Recommendation Draft](https://www.w3.org/standards/types/#CRD), 1 October 2026

More details about this document

<strong>This version:</strong>

<https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-syntax-3/>

<strong>Editor's Draft:</strong>

<https://drafts.csswg.org/css-syntax/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/>

<strong>History:</strong>

<https://www.w3.org/standards/history/css-syntax-3/>

<strong>Implementation Report:</strong>

<https://wpt.fyi/results/css/css-syntax?label=experimental&label=master&aligned>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-syntax-3)

<strong>Editors:</strong>

[Tab Atkins Jr.](http://xanthir.com/contact/) (Google)

[Simon Sapin](http://exyr.org/about/) (Mozilla)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-syntax-3/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-syntax/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

This module describes, in general terms, the basic structure and syntax of CSS stylesheets. It defines, in detail, the syntax and parsing of CSS - how to turn a stream of bytes into a meaningful stylesheet.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication. A list of current W3C publications and the latest revision of this technical report can be found in the [W3C standards and drafts index.](https://www.w3.org/TR/)</em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-syntax” in the title, like this: “\[css-syntax\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-syntax%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](#intro)
    1.  [1.1 Module interactions](#placement)
2.  [2 Description of CSS’s Syntax](#syntax-description)
    1.  [2.1 Escaping](#escaping)
    2.  [2.2 Error Handling](#error-handling)
3.  [3 Tokenizing and Parsing CSS](#tokenizing-and-parsing)
    1.  [3.1 Overview of the Parsing Model](#parsing-overview)
    2.  [3.2 The input byte stream](#input-byte-stream)
    3.  [3.3 Preprocessing the input stream](#input-preprocessing)
4.  [4 Tokenization](#tokenization)
    1.  [4.1 Token Railroad Diagrams](#token-diagrams)
    2.  [4.2 Definitions](#tokenizer-definitions)
    3.  [4.3 Tokenizer Algorithms](#tokenizer-algorithms)
        1.  [4.3.1 Consume a token](#consume-token)
        2.  [4.3.2 Consume comments](#consume-comment)
        3.  [4.3.3 Consume a numeric token](#consume-numeric-token)
        4.  [4.3.4 Consume an ident-like token](#consume-ident-like-token)
        5.  [4.3.5 Consume a string token](#consume-string-token)
        6.  [4.3.6 Consume a url token](#consume-url-token)
        7.  [4.3.7 Consume an escaped code point](#consume-escaped-code-point)
        8.  [4.3.8 Check if two code points are a valid escape](#starts-with-a-valid-escape)
        9.  [4.3.9 Check if three code points would start an ident sequence](#would-start-an-identifier)
        10. [4.3.10 Check if three code points would start a number](#starts-with-a-number)
        11. [4.3.11 Check if three code points would start a unicode-range](#starts-a-unicode-range)
        12. [4.3.12 Consume an ident sequence](#consume-name)
        13. [4.3.13 Consume a number](#consume-number)
        14. [4.3.14 Consume a unicode-range token](#consume-unicode-range-token)
        15. [4.3.15 Consume the remnants of a bad url](#consume-remnants-of-bad-url)
5.  [5 Parsing](#parsing)
    1.  [5.1 Parser Railroad Diagrams](#parser-diagrams)
    2.  [5.2 CSS Parsing Results](#css-tree)
    3.  [5.3 Token Streams](#parser-definitions)
    4.  [5.4 Parser Entry Points](#parser-entry-points)
        1.  [5.4.1 Parse something according to a CSS grammar](#parse-grammar)
        2.  [5.4.2 Parse a comma-separated list according to a CSS grammar](#parse-comma-list)
        3.  [5.4.3 Parse a stylesheet](#parse-stylesheet)
        4.  [5.4.4 Parse a stylesheet’s contents](#parse-stylesheet-contents)
        5.  [5.4.5 Parse a block’s contents](#parse-block-contents)
        6.  [5.4.6 Parse a rule](#parse-rule)
        7.  [5.4.7 Parse a declaration](#parse-declaration)
        8.  [5.4.8 Parse a component value](#parse-component-value)
        9.  [5.4.9 Parse a list of component values](#parse-list-of-component-values)
        10. [5.4.10 Parse a comma-separated list of component values](#parse-comma-separated-list-of-component-values)
    5.  [5.5 Parser Algorithms](#parser-algorithms)
        1.  [5.5.1 Consume a stylesheet’s contents](#consume-stylesheet-contents)
        2.  [5.5.2 Consume an at-rule](#consume-at-rule)
        3.  [5.5.3 Consume a qualified rule](#consume-qualified-rule)
        4.  [5.5.4 Consume a block](#consume-block)
        5.  [5.5.5 Consume a block’s contents](#consume-block-contents)
        6.  [5.5.6 Consume a declaration](#consume-declaration)
        7.  [5.5.7 Consume a list of component values](#consume-list-of-components)
        8.  [5.5.8 Consume a component value](#consume-component-value)
        9.  [5.5.9 Consume a simple block](#consume-simple-block)
        10. [5.5.10 Consume a function](#consume-function)
        11. [5.5.11 Consume a unicode-range value](#consume-unicode-range-value)
6.  [6 The <var>An+B</var> microsyntax](#anb-microsyntax)
    1.  [6.1 Informal Syntax Description](#anb-syntax)
    2.  [6.2 The <code>&#60;a-n-plus-b&#62;</code> type](#the-anb-type)
7.  [7 Defining Grammars for Rules and Other Values](#rule-defs)
    1.  [7.1 Defining Block Contents: the \<block-contents\>, \<declaration-list\>, \<qualified-rule-list\>, \<declaration-rule-list\>, and \<rule-list\> productions](#block-contents)
    2.  [7.2 Defining Arbitrary Contents: the \<declaration-value\> and \<any-value\> productions](#any-value)
8.  [8 CSS stylesheets](#css-stylesheets)
    1.  [8.1 Style rules](#style-rules)
    2.  [8.2 At-rules](#at-rules)
    3.  [8.3 The &#64;charset Rule](#charset-rule)
9.  [9 Serialization](#serialization)
    1.  [9.1 Serializing <var>\<a-n-plus-b\></var>](#serializing-anb)
10. [10 Privacy Considerations](#privacy)
11. [11 Security Considerations](#security)
12. [12 Changes](#changes)
    1.  [12.1 Changes from the 24 December 2021 Candidate Recommendation Draft](#changes-CR-20211224)
    2.  [12.2 Changes from the 16 August 2019 Candidate Recommendation](#changes-CR-20190716)
    3.  [12.3 Changes from the 20 February 2014 Candidate Recommendation](#changes-CR-20140220)
    4.  [12.4 Changes from the 5 November 2013 Last Call Working Draft](#changes-WD-20131105)
    5.  [12.5 Changes from the 19 September 2013 Working Draft](#changes-WD-20130919)
    6.  [12.6 Changes from CSS 2.1 and Selectors Level 3](#changes-css21)
13. [ Acknowledgments](#acknowledgments)
14. [ Conformance](#w3c-conformance)
    1.  [ Document conventions](#w3c-conventions)
    2.  [ Conformance classes](#w3c-conformance-classes)
    3.  [ Partial implementations](#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](#w3c-testing)
    5.  [ CR exit criteria](#w3c-cr-exit-criteria)
15. [ Index](#index)
    1.  [ Terms defined by this specification](#index-defined-here)
    2.  [ Terms defined by reference](#index-defined-elsewhere)
16. [ References](#references)
    1.  [ Normative References](#normative)
    2.  [ Non-Normative References](#informative)

## <a id="intro"></a>1.  Introduction[](#intro)

<em>This section is not normative.</em>

This module defines the abstract syntax and parsing of CSS stylesheets and other things which use CSS syntax (such as the HTML <code>style</code> attribute).

It defines algorithms for converting a stream of Unicode <a id="ref-for-code-point"></a>[code points](https://infra.spec.whatwg.org/#code-point) (in other words, text) into a stream of CSS tokens, and then further into CSS objects such as stylesheets, rules, and declarations.

### <a id="placement"></a>1.1.  Module interactions[](#placement)

This module defines the syntax and parsing of CSS stylesheets. It supersedes the lexical scanner and grammar defined in CSS 2.1.

## <a id="syntax-description"></a>2.  Description of CSS’s Syntax[](#syntax-description)

<em>This section is not normative.</em>

A CSS document is a series of <a id="ref-for-style-rule"></a>[style rules](#style-rule)—​which are <a id="ref-for-qualified-rule"></a>[qualified rules](#qualified-rule) that apply styles to elements in a document—​and <a id="ref-for-at-rule"></a>[at-rules](#at-rule)—​which define special processing rules or values for the CSS document.

A <a id="ref-for-qualified-rule①"></a>[qualified rule](#qualified-rule) starts with a prelude then has a {}-wrapped block containing a sequence of declarations. The meaning of the prelude varies based on the context that the rule appears in—​for <a id="ref-for-style-rule①"></a>[style rules](#style-rule), it’s a selector which specifies what elements the declarations will apply to. Each declaration has a name, followed by a colon and the declaration value. Declarations are separated by semicolons.

<a id="example-0ffd6a46"></a>

<strong>Example:</strong>

[](#example-0ffd6a46)

A typical rule might look something like this:

``` text
p > a {
  color: blue;
  text-decoration: underline;
}
```

In the above rule, "<code>p &#62; a</code>" is the selector, which, if the source document is HTML, selects any <code><a id="ref-for-the-a-element"></a>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element)</code> elements that are children of a <code><a id="ref-for-the-p-element"></a>[p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element)</code> element.

"<code>color&#58; blue</code>" is a declaration specifying that, for the elements that match the selector, their <a id="ref-for-propdef-color"></a>[color](https://www.w3.org/TR/css-color-4/#propdef-color) property should have the value <a id="ref-for-valdef-color-blue"></a>[blue](https://www.w3.org/TR/css-color-4/#valdef-color-blue). Similarly, their <a id="ref-for-propdef-text-decoration"></a>[text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration) property should have the value <a id="ref-for-valdef-text-decoration-line-underline"></a>[underline](https://www.w3.org/TR/css-text-decor-4/#valdef-text-decoration-line-underline).

<a id="ref-for-at-rule①"></a>[At-rules](#at-rule) are all different, but they have a basic structure in common. They start with an &#34;&#64;&#34; <a id="ref-for-code-point①"></a>[code point](https://infra.spec.whatwg.org/#code-point) followed by their name as a CSS keyword. Some <a id="ref-for-at-rule②"></a>at-rules are simple statements, with their name followed by more CSS values to specify their behavior, and finally ended by a semicolon. Others are blocks; they can have CSS values following their name, but they end with a {}-wrapped block, similar to a <a id="ref-for-qualified-rule②"></a>[qualified rule](#qualified-rule). Even the contents of these blocks are specific to the given <a id="ref-for-at-rule③"></a>at-rule: sometimes they contain a sequence of declarations, like a <a id="ref-for-qualified-rule③"></a>qualified rule; other times, they may contain additional blocks, or at-rules, or other structures altogether.

<a id="example-42888746"></a>

<strong>Example:</strong>

[](#example-42888746)

Here are several examples of <a id="ref-for-at-rule④"></a>[at-rules](#at-rule) that illustrate the varied syntax they may contain.

``` text
@import "my-styles.css";
```

The <a id="ref-for-at-ruledef-import"></a>[&#64;import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) <a id="ref-for-at-rule⑤"></a>[at-rule](#at-rule) is a simple statement. After its name, it takes a single string or <a id="ref-for-funcdef-url"></a>[url()](https://www.w3.org/TR/css-values-4/#funcdef-url) function to indicate the stylesheet that it should import.

``` text
@page :left {
  margin-left: 4cm;
  margin-right: 3cm;
}
```

The <a id="ref-for-at-ruledef-page"></a>[&#64;page](https://www.w3.org/TR/css-page-3/#at-ruledef-page) <a id="ref-for-at-rule⑥"></a>[at-rule](#at-rule) consists of an optional page selector (the <a id="ref-for-valdef-page-left"></a>[:left](https://www.w3.org/TR/css-page-3/#valdef-page-left) pseudoclass), followed by a block of properties that apply to the page when printed. In this way, it’s very similar to a normal style rule, except that its properties don’t apply to any "element", but rather the page itself.

``` text
@media print {
  body { font-size: 10pt }
}
```

The <a id="ref-for-at-ruledef-media"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) <a id="ref-for-at-rule⑦"></a>[at-rule](#at-rule) begins with a <a id="ref-for-media-type"></a>[media type](https://www.w3.org/TR/mediaqueries-5/#media-type) and a list of optional media queries. Its block contains entire rules, which are only applied when the <a id="ref-for-at-ruledef-media①"></a>&#64;medias conditions are fulfilled.

Property names and <a id="ref-for-at-rule⑧"></a>[at-rule](#at-rule) names are always <a id="ref-for-ident-sequence"></a>[ident sequences](#ident-sequence), which have to start with an <a id="ref-for-ident-start-code-point"></a>[ident-start code point](#ident-start-code-point), two hyphens, or a hyphen followed by an ident-start code point, and then can contain zero or more <a id="ref-for-ident-code-point"></a>[ident code points](#ident-code-point). You can include any <a id="ref-for-code-point②"></a>[code point](https://infra.spec.whatwg.org/#code-point) at all, even ones that CSS uses in its syntax, by <a id="ref-for-escape-codepoint"></a>[escaping](#escape-codepoint) it.

The syntax of selectors is defined in the [Selectors spec](https://www.w3.org/TR/selectors/). Similarly, the syntax of the wide variety of CSS values is defined in the [Values & Units spec](https://www.w3.org/TR/css3-values/). The special syntaxes of individual <a id="ref-for-at-rule⑨"></a>[at-rules](#at-rule) can be found in the specs that define them.

### <a id="escaping"></a>2.1.  Escaping[](#escaping)

<em>This section is not normative.</em>

Any Unicode <a id="ref-for-code-point③"></a>[code point](https://infra.spec.whatwg.org/#code-point) can be included in an <a id="ref-for-ident-sequence①"></a>[ident sequence](#ident-sequence) or quoted string by <a id="escape-codepoint"></a><strong>escaping</strong> it. CSS escape sequences start with a backslash (&#92;), and continue with:

- Any Unicode <a id="ref-for-code-point④"></a>[code point](https://infra.spec.whatwg.org/#code-point) that is not a <a id="ref-for-hex-digit"></a>[hex digits](#hex-digit) or a <a id="ref-for-newline"></a>[newline](#newline). The escape sequence is replaced by that <a id="ref-for-code-point⑤"></a>code point.

- Or one to six <a id="ref-for-hex-digit①"></a>[hex digits](#hex-digit), followed by an optional <a id="ref-for-whitespace"></a>[whitespace](#whitespace). The escape sequence is replaced by the Unicode <a id="ref-for-code-point⑥"></a>[code point](https://infra.spec.whatwg.org/#code-point) whose value is given by the hexadecimal digits. This optional whitespace allow hexadecimal escape sequences to be followed by "real" hex digits.

  <a id="example-cc2d6518"></a>

  <strong>Example:</strong>

  [](#example-cc2d6518) An <a id="ref-for-ident-sequence②"></a>[ident sequence](#ident-sequence) with the value "&B" could be written as \26 B or \000026B.

  A "real" space after the escape sequence must be doubled.

  <strong>Note:</strong>

### <a id="error-handling"></a>2.2.  Error Handling[](#error-handling)

<em>This section is not normative.</em>

When errors occur in CSS, the parser attempts to recover gracefully, throwing away only the minimum amount of content before returning to parsing as normal. This is because errors aren’t always mistakes—​new syntax looks like an error to an old parser, and it’s useful to be able to add new syntax to the language without worrying about stylesheets that include it being completely broken in older UAs.

The precise error-recovery behavior is detailed in the parser itself, but it’s simple enough that a short description is fairly accurate.

- At the "top level" of a stylesheet, an <a id="ref-for-typedef-at-keyword-token"></a>[\<at-keyword-token\>](#typedef-at-keyword-token) starts an at-rule. Anything else starts a qualified rule, and is included in the rule’s prelude. This may produce an invalid selector, but that’s not the concern of the CSS parser—​at worst, it means the selector will match nothing.
- Once an at-rule starts, nothing is invalid from the parser’s standpoint; it’s all part of the at-rule’s prelude. Encountering a <a id="ref-for-typedef-semicolon-token"></a>[\<semicolon-token\>](#typedef-semicolon-token) ends the at-rule immediately, while encountering an opening curly-brace <a id="ref-for-tokendef-open-curly"></a>[\<{-token\>](#tokendef-open-curly) starts the at-rule’s body. The at-rule seeks forward, matching blocks (content surrounded by (), {}, or \[\]) until it finds a closing curly-brace <a id="ref-for-tokendef-close-curly"></a>[\<}-token\>](#tokendef-close-curly) that isn’t matched by anything else or inside of another block. The contents of the at-rule are then interpreted according to the at-rule’s own grammar.
- Qualified rules work similarly, except that semicolons don’t end them; instead, they are just taken in as part of the rule’s prelude. When the first {} block is found, the contents are always interpreted as a list of declarations.
- When interpreting a list of declarations, unknown syntax at any point causes the parser to throw away whatever declaration it’s currently building, and seek forward until it finds a semicolon (or the end of the block). It then starts fresh, trying to parse a declaration again.
- <a id="autoclosing"></a> [](#autoclosing) If the stylesheet ends while any rule, declaration, function, string, etc. are still open, everything is automatically closed. This doesn’t make them invalid, though they may be incomplete and thus thrown away when they are verified against their grammar.
  <a id="example-84a0b91f"></a>
  <strong>Example:</strong>

  [](#example-84a0b91f) For example, the syntax of the <a id="ref-for-funcdef-transform-translatex"></a>[translateX()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translatex) function is:
  ``` text
  translateX( <translation-value> )
  ```

  However, the stylesheet may end with the function unclosed, like:

  ``` text
  .foo { transform: translate(50px
  ```

  The CSS parser parses this as a style rule containing one declaration, whose value is a function named "translate". This matches the above grammar, even though the ending token didn’t appear in the token stream, because by the time the parser is finished, the presence of the ending token is no longer possible to determine; all you have is the fact that there’s a block and a function.

After each construct (declaration, style rule, at-rule) is parsed, the user agent checks it against its expected grammar. If it does not match the grammar, it’s <a id="css-invalid"></a><strong>invalid</strong>, and gets <a id="css-ignored"></a><strong>ignored</strong> by the UA, which treats it as if it wasn’t there at all.

## <a id="tokenizing-and-parsing"></a>3.  Tokenizing and Parsing CSS[](#tokenizing-and-parsing)

User agents must use the parsing rules described in this specification to generate the [\[CSSOM\]](#biblio-cssom) trees from text/css resources. Together, these rules define what is referred to as the CSS parser.

This specification defines the parsing rules for CSS documents, whether they are syntactically correct or not. Certain points in the parsing algorithm are said to be <a id="parse-error"></a><strong>parse errors</strong>. The error handling for parse errors is well-defined: user agents must either act as described below when encountering such problems, or must abort processing at the first error that they encounter for which they do not wish to apply the rules described below.

Conformance checkers must report at least one parse error condition to the user if one or more parse error conditions exist in the document and must not report parse error conditions if none exist in the document. Conformance checkers may report more than one parse error condition if more than one parse error condition exists in the document. Conformance checkers are not required to recover from parse errors, but if they do, they must recover in the same way as user agents.

Tests

- [trailing-braces.html](https://wpt.fyi/results/css/css-syntax/trailing-braces.html) [(live test)](http://wpt.live/css/css-syntax/trailing-braces.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/trailing-braces.html)

### <a id="parsing-overview"></a>3.1.  Overview of the Parsing Model[](#parsing-overview)

The input to the CSS parsing process consists of a stream of Unicode <a id="ref-for-code-point⑦"></a>[code points](https://infra.spec.whatwg.org/#code-point), which is passed through a tokenization stage followed by a tree construction stage. The output is a CSSStyleSheet object.

Note: Implementations that do not support scripting do not have to actually create a CSSOM CSSStyleSheet object, but the CSSOM tree in such cases is still used as the model for the rest of the specification.

### <a id="input-byte-stream"></a>3.2.  The input byte stream[](#input-byte-stream)

When parsing a stylesheet, the stream of Unicode <a id="ref-for-code-point⑧"></a>[code points](https://infra.spec.whatwg.org/#code-point) that comprises the input to the tokenization stage might be initially seen by the user agent as a stream of bytes (typically coming over the network or from the local file system). If so, the user agent must decode these bytes into <a id="ref-for-code-point⑨"></a>code points according to a particular character encoding.

To <a id="css-decode-bytes"></a><strong>decode</strong> a <var>stylesheet</var>’s stream of bytes into a stream of <a id="ref-for-code-point①⓪"></a>[code points](https://infra.spec.whatwg.org/#code-point):

1.  <a id="ref-for-determine-the-fallback-encoding"></a>[Determine the fallback encoding](#determine-the-fallback-encoding) of <var>stylesheet</var>, and let <var>fallback</var> be the result.

2.  <a id="ref-for-decode"></a>[Decode](https://encoding.spec.whatwg.org/#decode) <var>stylesheet</var>’s stream of bytes with fallback encoding <var>fallback</var>, and return the result.

Note: The <a id="ref-for-decode①"></a>[decode](https://encoding.spec.whatwg.org/#decode) algorithm gives precedence to a byte order mark (BOM), and only uses the fallback when none is found.

To <a id="determine-the-fallback-encoding"></a><strong>determine the fallback encoding</strong> of a <var>stylesheet</var>:

1.  If HTTP or equivalent protocol provides an <var>encoding label</var> (e.g. via the charset parameter of the Content-Type header) for the <var>stylesheet</var>, <a id="ref-for-concept-encoding-get"></a>[get an encoding](https://encoding.spec.whatwg.org/#concept-encoding-get) from <var>encoding label</var>. If that does not return failure, return it.

2.  Otherwise, check <var>stylesheet</var>’s byte stream. If the first 1024 bytes of the stream begin with the hex sequence

    ``` text
    40 63 68 61 72 73 65 74 20 22 XX* 22 3B
    ```

    where each <code>XX</code> byte is a value between 0<sub>16</sub> and 21<sub>16</sub> inclusive or a value between 23<sub>16</sub> and 7F<sub>16</sub> inclusive, then <a id="ref-for-concept-encoding-get①"></a>[get an encoding](https://encoding.spec.whatwg.org/#concept-encoding-get) from a string formed out of the sequence of <code>XX</code> bytes, interpreted as <code>ASCII</code>.

    <strong>Note:</strong>

    What does that byte sequence mean?

    The byte sequence above, when decoded as ASCII, is the string "<code>@charset &#34;…&#34;;</code>", where the "…" is the sequence of bytes corresponding to the encoding’s label.

    If the return value was <code>utf-16be</code> or <code>utf-16le</code>, return <code>utf-8</code>; if it was anything else except failure, return it.

    <strong>Note:</strong>

    Why use utf-8 when the declaration says utf-16?

    The bytes of the encoding declaration spell out “<code>@charset &#34;…&#34;;</code>” in ASCII, but UTF-16 is not ASCII-compatible. Either you’ve typed in complete gibberish (like <code>䁣桡牳整•utf-16be∻</code>) to get the right bytes in the document, which we don’t want to encourage, or your document is actually in an ASCII-compatible encoding and your encoding declaration is lying.

    Either way, defaulting to UTF-8 is a decent answer.

    As well, this mimics the behavior of HTML’s <code>&#60;meta charset&#62;</code> attribute.

    Note: Note that the syntax of an encoding declaration <em>looks like</em> the syntax of an <a id="ref-for-at-rule①⓪"></a>[at-rule](#at-rule) named <a id="ref-for-at-ruledef-charset"></a>[&#64;charset](#at-ruledef-charset), but no such rule actually exists, and the rules for how you can write it are much more restrictive than they would normally be for recognizing such a rule. A number of things you can do in CSS that would produce a valid <a id="ref-for-at-ruledef-charset①"></a>&#64;charset rule (if one existed), such as using multiple spaces, comments, or single quotes, will cause the encoding declaration to not be recognized. This behavior keeps the encoding declaration as simple as possible, and thus maximizes the likelihood of it being implemented correctly.

3.  Otherwise, if an <a id="ref-for-environment-encoding"></a>[environment encoding](#environment-encoding) is provided by the referring document, return it.

4.  Otherwise, return <code>utf-8</code>.

<strong>Note:</strong>

Though UTF-8 is the default encoding for the web, and many newer web-based file formats assume or require UTF-8 encoding, CSS was created before it was clear which encoding would win, and thus can’t automatically assume the stylesheet is UTF-8.

Stylesheet authors <em>should</em> author their stylesheets in UTF-8, and ensure that either an HTTP header (or equivalent method) declares the encoding of the stylesheet to be UTF-8, or that the referring document declares its encoding to be UTF-8. (In HTML, this is done by adding a <code>&#60;meta charset=utf-8&#62;</code> element to the head of the document.)

If neither of these options are available, authors should begin the stylesheet with a UTF-8 BOM or the exact characters

``` text
@charset "utf-8";
```

Document languages that refer to CSS stylesheets that are decoded from bytes may define an <a id="environment-encoding"></a><strong>environment encoding</strong> for each such stylesheet, which is used as a fallback when other encoding hints are not available or can not be used.

The concept of <a id="ref-for-environment-encoding①"></a>[environment encoding](#environment-encoding) only exists for compatibility with legacy content. New formats and new linking mechanisms <strong>should not</strong> provide an <a id="ref-for-environment-encoding②"></a>environment encoding, so the stylesheet defaults to UTF-8 instead in the absence of more explicit information.

Note: [\[HTML\]](#biblio-html) defines [the environment encoding for <code>&#60;link rel=stylesheet&#62;</code>](https://html.spec.whatwg.org/multipage/links.html#link-type-stylesheet).

Note: [\[CSSOM\]](#biblio-cssom) defines [the environment encoding for <code>&#60;xml-stylesheet?&#62;</code>](https://drafts.csswg.org/cssom/#requirements-on-user-agents-implementing-the-xml-stylesheet-processing-instruction).

Note: [\[CSS-CASCADE-3\]](#biblio-css-cascade-3) defines <a id="ref-for-at-ruledef-import①"></a>[the environment encoding for <code>@import</code>](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import).

### <a id="input-preprocessing"></a>3.3.  Preprocessing the input stream[](#input-preprocessing)

The <a id="input-stream"></a><strong>input stream</strong> consists of the <a id="ref-for-css-filter-code-points"></a>[filtered code points](#css-filter-code-points) pushed into it as the input byte stream is decoded.

To <a id="css-filter-code-points"></a><strong>filter code points</strong> from a stream of (unfiltered) <a id="ref-for-code-point①①"></a>[code points](https://infra.spec.whatwg.org/#code-point) <var>input</var>:

- Replace any U+000D CARRIAGE RETURN (CR) <a id="ref-for-code-point①②"></a>[code points](https://infra.spec.whatwg.org/#code-point), U+000C FORM FEED (FF) <a id="ref-for-code-point①③"></a>code points, or pairs of U+000D CARRIAGE RETURN (CR) followed by U+000A LINE FEED (LF) in <var>input</var> by a single U+000A LINE FEED (LF) <a id="ref-for-code-point①④"></a>code point.

- Replace any U+0000 NULL or <a id="ref-for-surrogate"></a>[surrogate](https://infra.spec.whatwg.org/#surrogate) <a id="ref-for-code-point①⑤"></a>[code points](https://infra.spec.whatwg.org/#code-point) in <var>input</var> with U+FFFD REPLACEMENT CHARACTER (�).

  Note: The only way to produce a <a id="ref-for-surrogate①"></a>[surrogate](https://infra.spec.whatwg.org/#surrogate) <a id="ref-for-code-point①⑥"></a>[code point](https://infra.spec.whatwg.org/#code-point) in CSS content is by directly assigning a <code>DOMString</code> with one in it via an OM operation.

Tests

- [input-preprocessing.html](https://wpt.fyi/results/css/css-syntax/input-preprocessing.html) [(live test)](http://wpt.live/css/css-syntax/input-preprocessing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/input-preprocessing.html)

## <a id="tokenization"></a>4.  Tokenization[](#tokenization)

To <a id="css-tokenize"></a><strong>tokenize</strong> a stream of <a id="ref-for-code-point①⑦"></a>[code points](https://infra.spec.whatwg.org/#code-point) into a stream of CSS tokens <var>input</var>, repeatedly <a id="ref-for-tokenizer-consume-a-token"></a>[consume a token](#tokenizer-consume-a-token) from <var>input</var> until an <a id="ref-for-typedef-eof-token"></a>[\<EOF-token\>](#typedef-eof-token) is reached, pushing each of the returned tokens into a stream.

Note: Each call to the <a id="ref-for-tokenizer-consume-a-token①"></a>[consume a token](#tokenizer-consume-a-token) algorithm returns a single token, so it can also be used "on-demand" to tokenize a stream of <a id="ref-for-code-point①⑧"></a>[code points](https://infra.spec.whatwg.org/#code-point) <em>during</em> parsing, if so desired.

The output of tokenization step is a stream of zero or more of the following tokens: <a id="typedef-ident-token"></a><strong>\<ident-token\></strong>, <a id="typedef-function-token"></a><strong>\<function-token\></strong>, <a id="typedef-at-keyword-token"></a><strong>\<at-keyword-token\></strong>, <a id="typedef-hash-token"></a><strong>\<hash-token\></strong>, <a id="typedef-string-token"></a><strong>\<string-token\></strong>, <a id="typedef-bad-string-token"></a><strong>\<bad-string-token\></strong>, <a id="typedef-url-token"></a><strong>\<url-token\></strong>, <a id="typedef-bad-url-token"></a><strong>\<bad-url-token\></strong>, <a id="typedef-delim-token"></a><strong>\<delim-token\></strong>, <a id="typedef-number-token"></a><strong>\<number-token\></strong>, <a id="typedef-percentage-token"></a><strong>\<percentage-token\></strong>, <a id="typedef-dimension-token"></a><strong>\<dimension-token\></strong>, <a id="typedef-unicode-range-token"></a><strong>\<unicode-range-token\></strong>, <a id="typedef-whitespace-token"></a><strong>\<whitespace-token\></strong>, <a id="typedef-cdo-token"></a><strong>\<CDO-token\></strong>, <a id="typedef-cdc-token"></a><strong>\<CDC-token\></strong>, <a id="typedef-colon-token"></a><strong>\<colon-token\></strong>, <a id="typedef-semicolon-token"></a><strong>\<semicolon-token\></strong>, <a id="typedef-comma-token"></a><strong>\<comma-token\></strong>, <a id="tokendef-open-square"></a><strong>\<\[-token\></strong>, <a id="tokendef-close-square"></a><strong>\<\]-token\></strong>, <a id="tokendef-open-paren"></a><strong>\<(-token\></strong>, <a id="tokendef-close-paren"></a><strong>\<)-token\></strong>, <a id="tokendef-open-curly"></a><strong>\<{-token\></strong>, and <a id="tokendef-close-curly"></a><strong>\<}-token\></strong>.

- <a id="ref-for-typedef-ident-token"></a>[\<ident-token\>](#typedef-ident-token), <a id="ref-for-typedef-function-token"></a>[\<function-token\>](#typedef-function-token), <a id="ref-for-typedef-at-keyword-token①"></a>[\<at-keyword-token\>](#typedef-at-keyword-token), <a id="ref-for-typedef-hash-token"></a>[\<hash-token\>](#typedef-hash-token), <a id="ref-for-typedef-string-token"></a>[\<string-token\>](#typedef-string-token), and <a id="ref-for-typedef-url-token"></a>[\<url-token\>](#typedef-url-token) have a value composed of zero or more <a id="ref-for-code-point①⑨"></a>[code points](https://infra.spec.whatwg.org/#code-point). Additionally, hash tokens have a type flag set to either "id" or "unrestricted". The type flag defaults to "unrestricted" if not otherwise set.

- <a id="ref-for-typedef-delim-token"></a>[\<delim-token\>](#typedef-delim-token) has a value composed of a single <a id="ref-for-code-point②⓪"></a>[code point](https://infra.spec.whatwg.org/#code-point).

- <a id="ref-for-typedef-number-token"></a>[\<number-token\>](#typedef-number-token), <a id="ref-for-typedef-percentage-token"></a>[\<percentage-token\>](#typedef-percentage-token), and <a id="ref-for-typedef-dimension-token"></a>[\<dimension-token\>](#typedef-dimension-token) have a numeric value, and an optional sign character set to either "+" or "-" (or nothing).

  <a id="ref-for-typedef-number-token①"></a>[\<number-token\>](#typedef-number-token) and <a id="ref-for-typedef-dimension-token①"></a>[\<dimension-token\>](#typedef-dimension-token) additionally have a type flag set to either "integer" or "number". The type flag defaults to "integer" if not otherwise set. <a id="ref-for-typedef-dimension-token②"></a>\<dimension-token\> additionally have a unit composed of one or more <a id="ref-for-code-point②①"></a>[code points](https://infra.spec.whatwg.org/#code-point).

- <a id="ref-for-typedef-unicode-range-token"></a>[\<unicode-range-token\>](#typedef-unicode-range-token) has a starting and ending code point. It represents an inclusive range of codepoints (including both the start and end). If the ending code point is before the starting code point, it represents an empty range.

Note: The type flag of hash tokens is used in the Selectors syntax [\[SELECT\]](#biblio-select). Only hash tokens with the "id" type are valid [ID selectors](https://www.w3.org/TR/selectors/#id-selectors).

### <a id="token-diagrams"></a>4.1.  Token Railroad Diagrams[](#token-diagrams)

<em>This section is non-normative.</em>

This section presents an informative view of the tokenizer, in the form of railroad diagrams. Railroad diagrams are more compact than an explicit parser, but often easier to read than an regular expression.

These diagrams are <em>informative</em> and <em>incomplete</em>; they describe the grammar of "correct" tokens, but do not describe error-handling at all. They are provided solely to make it easier to get an intuitive grasp of the syntax of each token.

Diagrams with names such as <em>\<foo-token\></em> represent tokens. The rest are productions referred to by other diagrams.

<a id="comment-diagram"></a><strong>[](#comment-diagram)comment</strong>

![Source diagram 1](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-01.svg)

Diagram text: /\* anything but \* followed by / \*/

<a id="newline-diagram"></a><strong>[](#newline-diagram)newline</strong>

![Source diagram 2](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-02.svg)

Diagram text: \n \r\n \r \f

<a id="whitespace-diagram"></a><strong>[](#whitespace-diagram)whitespace</strong>

![Source diagram 3](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-03.svg)

Diagram text: space \t newline

<a id="hex-digit-diagram"></a><strong>[](#hex-digit-diagram)hex digit</strong>

![Source diagram 4](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-04.svg)

Diagram text: 0-9 a-f or A-F

<a id="escape-diagram"></a><strong>[](#escape-diagram)escape</strong>

![Source diagram 5](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-05.svg)

Diagram text: \\ not newline or hex digit hex digit 1-6 times whitespace

<a id="whitespace-token-diagram"></a><strong>[](#whitespace-token-diagram)<a id="ref-for-typedef-whitespace-token"></a>[\<whitespace-token\>](#typedef-whitespace-token)</strong>

![Source diagram 6](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-06.svg)

Diagram text: whitespace

<a id="ws&#42;-diagram"></a><strong>[](#ws*-diagram)ws\*</strong>

![Source diagram 7](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-07.svg)

Diagram text: \<whitespace-token\>

<a id="ident-token-diagram"></a><strong>[](#ident-token-diagram)<a id="ref-for-typedef-ident-token①"></a>[\<ident-token\>](#typedef-ident-token)</strong>

![Source diagram 8](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-08.svg)

Diagram text: -- - a-z A-Z \_ or non-ASCII escape a-z A-Z 0-9 \_ - or non-ASCII escape

<a id="function-token-diagram"></a><strong>[](#function-token-diagram)<a id="ref-for-typedef-function-token①"></a>[\<function-token\>](#typedef-function-token)</strong>

![Source diagram 9](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-09.svg)

Diagram text: \<ident-token\> (

<a id="at-keyword-token-diagram"></a><strong>[](#at-keyword-token-diagram)<a id="ref-for-typedef-at-keyword-token②"></a>[\<at-keyword-token\>](#typedef-at-keyword-token)</strong>

![Source diagram 10](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-10.svg)

Diagram text: &#64; \<ident-token\>

<a id="hash-token-diagram"></a><strong>[](#hash-token-diagram)<a id="ref-for-typedef-hash-token①"></a>[\<hash-token\>](#typedef-hash-token)</strong>

![Source diagram 11](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-11.svg)

Diagram text: \# a-z A-Z 0-9 \_ - or non-ASCII escape

<a id="string-token-diagram"></a><strong>[](#string-token-diagram)<a id="ref-for-typedef-string-token①"></a>[\<string-token\>](#typedef-string-token)</strong>

![Source diagram 12](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-12.svg)

Diagram text: " not " \\ or newline escape \\ newline " ' not ' \\ or newline escape \\ newline '

<a id="url-token-diagram"></a><strong>[](#url-token-diagram)<a id="ref-for-typedef-url-token①"></a>[\<url-token\>](#typedef-url-token)</strong>

![Source diagram 13](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-13.svg)

Diagram text: \<ident-token "url"\> ( ws\* not " ' ( ) \\ ws or non-printable escape ws\* )

<a id="number-token-diagram"></a><strong>[](#number-token-diagram)<a id="ref-for-typedef-number-token②"></a>[\<number-token\>](#typedef-number-token)</strong>

![Source diagram 14](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-14.svg)

Diagram text: + - digit . digit digit . digit e E + - digit

<a id="dimension-token-diagram"></a><strong>[](#dimension-token-diagram)<a id="ref-for-typedef-dimension-token③"></a>[\<dimension-token\>](#typedef-dimension-token)</strong>

![Source diagram 15](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-15.svg)

Diagram text: \<number-token\> \<ident-token\>

<a id="percentage-token-diagram"></a><strong>[](#percentage-token-diagram)<a id="ref-for-typedef-percentage-token①"></a>[\<percentage-token\>](#typedef-percentage-token)</strong>

![Source diagram 16](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-16.svg)

Diagram text: \<number-token\> %

<a id="CDO-token-diagram"></a><strong>[](#CDO-token-diagram)<a id="ref-for-typedef-cdo-token"></a>[\<CDO-token\>](#typedef-cdo-token)</strong>

![Source diagram 17](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-17.svg)

Diagram text: \<!--

<a id="CDC-token-diagram"></a><strong>[](#CDC-token-diagram)<a id="ref-for-typedef-cdc-token"></a>[\<CDC-token\>](#typedef-cdc-token)</strong>

![Source diagram 18](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-18.svg)

Diagram text: --\>

<a id="unicode-range-token-diagram"></a><strong>[](#unicode-range-token-diagram)<a id="ref-for-typedef-unicode-range-token①"></a>[\<unicode-range-token\>](#typedef-unicode-range-token)</strong>

![Source diagram 19](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-19.svg)

Diagram text: U u + hex digit 1-6 times hex digit 1-5 times ? 1 to (6-digits) times hex digit 1-6 times - hex digit 1-6 times

### <a id="tokenizer-definitions"></a>4.2.  Definitions[](#tokenizer-definitions)

This section defines several terms used during the tokenization phase.

<strong><a id="next-input-code-point"></a><strong>next input code point</strong></strong>

The first <a id="ref-for-code-point②②"></a>[code point](https://infra.spec.whatwg.org/#code-point) in the <a id="ref-for-input-stream"></a>[input stream](#input-stream) that has not yet been consumed.

<strong><a id="current-input-code-point"></a><strong>current input code point</strong></strong>

The last <a id="ref-for-code-point②③"></a>[code point](https://infra.spec.whatwg.org/#code-point) to have been consumed.

<strong><a id="reconsume-the-current-input-code-point"></a><strong>reconsume the current input code point</strong></strong>

Push the <a id="ref-for-current-input-code-point"></a>[current input code point](#current-input-code-point) back onto the front of the <a id="ref-for-input-stream①"></a>[input stream](#input-stream), so that the next time you are instructed to consume the <a id="ref-for-next-input-code-point"></a>[next input code point](#next-input-code-point), it will instead reconsume the <a id="ref-for-current-input-code-point①"></a>current input code point.

<strong><a id="eof-code-point"></a><strong>EOF code point</strong></strong>

A conceptual <a id="ref-for-code-point②④"></a>[code point](https://infra.spec.whatwg.org/#code-point) representing the end of the <a id="ref-for-input-stream②"></a>[input stream](#input-stream). Whenever the <a id="ref-for-input-stream③"></a>input stream is empty, the <a id="ref-for-next-input-code-point①"></a>[next input code point](#next-input-code-point) is always an EOF code point.

<strong><a id="digit"></a><strong>digit</strong></strong>

A <a id="ref-for-code-point②⑤"></a>[code point](https://infra.spec.whatwg.org/#code-point) between U+0030 DIGIT ZERO (0) and U+0039 DIGIT NINE (9) inclusive.

Tests

- [inclusive-ranges.html](https://wpt.fyi/results/css/css-syntax/inclusive-ranges.html) [(live test)](http://wpt.live/css/css-syntax/inclusive-ranges.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/inclusive-ranges.html)

<strong><a id="hex-digit"></a><strong>hex digit</strong></strong>

A <a id="ref-for-digit"></a>[digit](#digit), or a <a id="ref-for-code-point②⑥"></a>[code point](https://infra.spec.whatwg.org/#code-point) between U+0041 LATIN CAPITAL LETTER A (A) and U+0046 LATIN CAPITAL LETTER F (F) inclusive, or a <a id="ref-for-code-point②⑦"></a>code point between U+0061 LATIN SMALL LETTER A (a) and U+0066 LATIN SMALL LETTER F (f) inclusive.

<strong><a id="uppercase-letter"></a><strong>uppercase letter</strong></strong>

A <a id="ref-for-code-point②⑧"></a>[code point](https://infra.spec.whatwg.org/#code-point) between U+0041 LATIN CAPITAL LETTER A (A) and U+005A LATIN CAPITAL LETTER Z (Z) inclusive.

<strong><a id="lowercase-letter"></a><strong>lowercase letter</strong></strong>

A <a id="ref-for-code-point②⑨"></a>[code point](https://infra.spec.whatwg.org/#code-point) between U+0061 LATIN SMALL LETTER A (a) and U+007A LATIN SMALL LETTER Z (z) inclusive.

<strong><a id="letter"></a><strong>letter</strong></strong>

An <a id="ref-for-uppercase-letter"></a>[uppercase letter](#uppercase-letter) or a <a id="ref-for-lowercase-letter"></a>[lowercase letter](#lowercase-letter).

<strong><a id="non-ascii-ident-code-point"></a><strong>non-ASCII ident code point</strong></strong>

A <a id="ref-for-code-point③⓪"></a>[code point](https://infra.spec.whatwg.org/#code-point) whose value is any of:

- U+00B7

- between U+00C0 and U+00D6

- between U+00D8 and U+00F6

- between U+00F8 and U+037D

- between U+037F and U+1FFF

- U+200C

- U+200D

- U+203F

- U+2040

- between U+2070 and U+218F

- between U+2C00 and U+2FEF

- between U+3001 and U+D7FF

- between U+F900 and U+FDCF

- between U+FDF0 and U+FFFD

- greater than or equal to U+10000

All of these ranges are inclusive.

Tests

- [non-ascii-codepoints.html](https://wpt.fyi/results/css/css-syntax/non-ascii-codepoints.html) [(live test)](http://wpt.live/css/css-syntax/non-ascii-codepoints.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/non-ascii-codepoints.html)

<strong>Note:</strong>

Why these character, specifically?

This matches the list of non-ASCII codepoints allowed to be used in HTML <a id="ref-for-valid-custom-element-name"></a>[valid custom element names](https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name). It excludes a number of characters that appear as whitespace, or that can cause rendering or parsing issues in some tools, such as the direction override codepoints.

Note that this is a weaker set of restrictions than [UAX 31](https://unicode.org/reports/tr31/#Figure_Code_Point_Categories_for_Identifier_Parsing) recommends for identifiers (used by languages such as JavaScript to restrict their identifier syntax), allowing things such as starting an identifier with a combining character. Consistency with HTML custom element names (and thus, the ability to write selectors for all custom elements without having to use escapes) was considered valuable, and the set of characters restricted by HTML covers the "high value" restrictions well.

These restrictions do not avoid all possible confusing renderings; mixing characters from LTR and RTL scripts can still result in unexpected visual transposition in most text editors, for example. Source text can contain the restricted characters in non-ident contexts, as well: most of them are completely valid in strings, for example. Even when used in a way that creates invalid CSS, the parsing errors they cause might be limited to something unimportant, while their effect on rendering the source text in code review tools might be significant and/or malicious. For more details on these sorts of "source text attacks", see [this Rust-lang blog post](https://blog.rust-lang.org/2021/11/01/cve-2021-42574.html) [(archived)](https://web.archive.org/web/20220323175009/https://blog.rust-lang.org/2021/11/01/cve-2021-42574.html).

<strong><a id="ident-start-code-point"></a><strong><a id="identifier-start-code-point"></a><a id="name-start-code-point"></a>ident-start code point</strong></strong>

A <a id="ref-for-letter"></a>[letter](#letter), a <a id="ref-for-non-ascii-ident-code-point"></a>[non-ASCII ident code point](#non-ascii-ident-code-point), or U+005F LOW LINE (\_).

<strong><a id="ident-code-point"></a><strong><a id="identifier-code-point"></a><a id="name-code-point"></a>ident code point</strong></strong>

An <a id="ref-for-ident-start-code-point①"></a>[ident-start code point](#ident-start-code-point), a <a id="ref-for-digit①"></a>[digit](#digit), or U+002D HYPHEN-MINUS (-).

<strong><a id="non-printable-code-point"></a><strong>non-printable code point</strong></strong>

A <a id="ref-for-code-point③①"></a>[code point](https://infra.spec.whatwg.org/#code-point) between U+0000 NULL and U+0008 BACKSPACE inclusive, or U+000B LINE TABULATION, or a <a id="ref-for-code-point③②"></a>code point between U+000E SHIFT OUT and U+001F INFORMATION SEPARATOR ONE inclusive, or U+007F DELETE.

Tests

- [inclusive-ranges.html](https://wpt.fyi/results/css/css-syntax/inclusive-ranges.html) [(live test)](http://wpt.live/css/css-syntax/inclusive-ranges.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/inclusive-ranges.html)

<strong><a id="newline"></a><strong>newline</strong></strong>

U+000A LINE FEED. Note that U+000D CARRIAGE RETURN and U+000C FORM FEED are not included in this definition, as they are converted to U+000A LINE FEED during

<strong>Note:</strong>

[preprocessing](#input-preprocessing).

<strong><a id="whitespace"></a><strong>whitespace</strong></strong>

A <a id="ref-for-newline①"></a>[newline](#newline), U+0009 CHARACTER TABULATION, or U+0020 SPACE.

Tests

- [whitespace.html](https://wpt.fyi/results/css/css-syntax/whitespace.html) [(live test)](http://wpt.live/css/css-syntax/whitespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/whitespace.html)

<strong><a id="maximum-allowed-code-point"></a><strong>maximum allowed code point</strong></strong>

The greatest <a id="ref-for-code-point③③"></a>[code point](https://infra.spec.whatwg.org/#code-point) defined by Unicode: U+10FFFF.

<strong><a id="ident-sequence"></a><strong><a id="identifier"></a><a id="css-identifier"></a>ident sequence</strong></strong>

A sequence of <a id="ref-for-code-point③④"></a>[code points](https://infra.spec.whatwg.org/#code-point) that has the same syntax as an <a id="ref-for-typedef-ident-token②"></a>[\<ident-token\>](#typedef-ident-token).

Note: The part of an <a id="ref-for-typedef-at-keyword-token③"></a>[\<at-keyword-token\>](#typedef-at-keyword-token) after the &#34;&#64;&#34;, the part of a <a id="ref-for-typedef-hash-token②"></a>[\<hash-token\>](#typedef-hash-token) (with the "id" type flag) after the "#", the part of a <a id="ref-for-typedef-function-token②"></a>[\<function-token\>](#typedef-function-token) before the "(", and the unit of a <a id="ref-for-typedef-dimension-token④"></a>[\<dimension-token\>](#typedef-dimension-token) are all <a id="ref-for-ident-sequence③"></a>[ident sequences](#ident-sequence).

### <a id="tokenizer-algorithms"></a>4.3.  Tokenizer Algorithms[](#tokenizer-algorithms)

The algorithms defined in this section transform a stream of <a id="ref-for-code-point③⑤"></a>[code points](https://infra.spec.whatwg.org/#code-point) into a stream of tokens.

#### <a id="consume-token"></a>4.3.1.  Consume a token[](#consume-token)

This section describes how to <a id="tokenizer-consume-a-token"></a><strong>consume a token</strong> from a stream of <a id="ref-for-code-point③⑥"></a>[code points](https://infra.spec.whatwg.org/#code-point). It additionally takes an optional boolean <var>unicode ranges allowed</var>, defaulting to false. It will return a single token of any type.

<a id="ref-for-consume-comments"></a>[Consume comments](#consume-comments).

Consume the <a id="ref-for-next-input-code-point②"></a>[next input code point](#next-input-code-point).

<strong><a id="ref-for-whitespace①"></a>[whitespace](#whitespace)</strong>

Consume as much <a id="ref-for-whitespace②"></a>[whitespace](#whitespace) as possible. Return a <a id="ref-for-typedef-whitespace-token①"></a>[\<whitespace-token\>](#typedef-whitespace-token).

<strong>U+0022 QUOTATION MARK (")</strong>

<a id="ref-for-consume-a-string-token"></a>[Consume a string token](#consume-a-string-token) and return it.

<strong>U+0023 NUMBER SIGN (#)</strong>

If the <a id="ref-for-next-input-code-point③"></a>[next input code point](#next-input-code-point) is an <a id="ref-for-ident-code-point①"></a>[ident code point](#ident-code-point) or the <a id="ref-for-next-input-code-point④"></a>next two input code points <a id="ref-for-check-if-two-code-points-are-a-valid-escape"></a>[are a valid escape](#check-if-two-code-points-are-a-valid-escape), then:

1.  Create a <a id="ref-for-typedef-hash-token③"></a>[\<hash-token\>](#typedef-hash-token).
2.  If the <a id="ref-for-next-input-code-point⑤"></a>[next 3 input code points](#next-input-code-point) <a id="ref-for-check-if-three-code-points-would-start-an-ident-sequence"></a>[would start an ident sequence](#check-if-three-code-points-would-start-an-ident-sequence), set the <a id="ref-for-typedef-hash-token④"></a>[\<hash-token\>](#typedef-hash-token)’s type flag to "id".
3.  <a id="ref-for-consume-an-ident-sequence"></a>[Consume an ident sequence](#consume-an-ident-sequence), and set the <a id="ref-for-typedef-hash-token⑤"></a>[\<hash-token\>](#typedef-hash-token)’s value to the returned string.
4.  Return the <a id="ref-for-typedef-hash-token⑥"></a>[\<hash-token\>](#typedef-hash-token).

Otherwise, return a <a id="ref-for-typedef-delim-token①"></a>[\<delim-token\>](#typedef-delim-token) with its value set to the <a id="ref-for-current-input-code-point②"></a>[current input code point](#current-input-code-point).

<strong>U+0027 APOSTROPHE (')</strong>

<a id="ref-for-consume-a-string-token①"></a>[Consume a string token](#consume-a-string-token) and return it.

<strong>U+0028 LEFT PARENTHESIS (()</strong>

Return a <a id="ref-for-tokendef-open-paren"></a>[\<(-token\>](#tokendef-open-paren).

<strong>U+0029 RIGHT PARENTHESIS ())</strong>

Return a <a id="ref-for-tokendef-close-paren"></a>[\<)-token\>](#tokendef-close-paren).

<strong>U+002B PLUS SIGN (+)</strong>

If the input stream <a id="ref-for-check-if-three-code-points-would-start-a-number"></a>[starts with a number](#check-if-three-code-points-would-start-a-number), <a id="ref-for-reconsume-the-current-input-code-point"></a>[reconsume the current input code point](#reconsume-the-current-input-code-point), <a id="ref-for-consume-a-numeric-token"></a>[consume a numeric token](#consume-a-numeric-token), and return it.

Otherwise, return a <a id="ref-for-typedef-delim-token②"></a>[\<delim-token\>](#typedef-delim-token) with its value set to the <a id="ref-for-current-input-code-point③"></a>[current input code point](#current-input-code-point).

<strong>U+002C COMMA (,)</strong>

Return a <a id="ref-for-typedef-comma-token"></a>[\<comma-token\>](#typedef-comma-token).

<strong>U+002D HYPHEN-MINUS (-)</strong>

If the input stream <a id="ref-for-check-if-three-code-points-would-start-a-number①"></a>[starts with a number](#check-if-three-code-points-would-start-a-number), <a id="ref-for-reconsume-the-current-input-code-point①"></a>[reconsume the current input code point](#reconsume-the-current-input-code-point), <a id="ref-for-consume-a-numeric-token①"></a>[consume a numeric token](#consume-a-numeric-token), and return it.

Otherwise, if the <a id="ref-for-next-input-code-point⑥"></a>[next 2 input code points](#next-input-code-point) are U+002D HYPHEN-MINUS U+003E GREATER-THAN SIGN (-\>), consume them and return a <a id="ref-for-typedef-cdc-token①"></a>[\<CDC-token\>](#typedef-cdc-token).

Otherwise, if the input stream <a id="ref-for-check-if-three-code-points-would-start-an-ident-sequence①"></a>[starts with an ident sequence](#check-if-three-code-points-would-start-an-ident-sequence), <a id="ref-for-reconsume-the-current-input-code-point②"></a>[reconsume the current input code point](#reconsume-the-current-input-code-point), <a id="ref-for-consume-an-ident-like-token"></a>[consume an ident-like token](#consume-an-ident-like-token), and return it.

Otherwise, return a <a id="ref-for-typedef-delim-token③"></a>[\<delim-token\>](#typedef-delim-token) with its value set to the <a id="ref-for-current-input-code-point④"></a>[current input code point](#current-input-code-point).

<strong>U+002E FULL STOP (.)</strong>

If the input stream <a id="ref-for-check-if-three-code-points-would-start-a-number②"></a>[starts with a number](#check-if-three-code-points-would-start-a-number), <a id="ref-for-reconsume-the-current-input-code-point③"></a>[reconsume the current input code point](#reconsume-the-current-input-code-point), <a id="ref-for-consume-a-numeric-token②"></a>[consume a numeric token](#consume-a-numeric-token), and return it.

Otherwise, return a <a id="ref-for-typedef-delim-token④"></a>[\<delim-token\>](#typedef-delim-token) with its value set to the <a id="ref-for-current-input-code-point⑤"></a>[current input code point](#current-input-code-point).

<strong>U+003A COLON (:)</strong>

Return a <a id="ref-for-typedef-colon-token"></a>[\<colon-token\>](#typedef-colon-token).

<strong>U+003B SEMICOLON (;)</strong>

Return a <a id="ref-for-typedef-semicolon-token①"></a>[\<semicolon-token\>](#typedef-semicolon-token).

<strong>U+003C LESS-THAN SIGN (\<)</strong>

If the <a id="ref-for-next-input-code-point⑦"></a>[next 3 input code points](#next-input-code-point) are U+0021 EXCLAMATION MARK U+002D HYPHEN-MINUS U+002D HYPHEN-MINUS (!--), consume them and return a <a id="ref-for-typedef-cdo-token①"></a>[\<CDO-token\>](#typedef-cdo-token).

Otherwise, return a <a id="ref-for-typedef-delim-token⑤"></a>[\<delim-token\>](#typedef-delim-token) with its value set to the <a id="ref-for-current-input-code-point⑥"></a>[current input code point](#current-input-code-point).

<strong>U+0040 COMMERCIAL AT (&#64;)</strong>

If the <a id="ref-for-next-input-code-point⑧"></a>[next 3 input code points](#next-input-code-point) <a id="ref-for-check-if-three-code-points-would-start-an-ident-sequence②"></a>[would start an ident sequence](#check-if-three-code-points-would-start-an-ident-sequence), <a id="ref-for-consume-an-ident-sequence①"></a>[consume an ident sequence](#consume-an-ident-sequence), create an <a id="ref-for-typedef-at-keyword-token④"></a>[\<at-keyword-token\>](#typedef-at-keyword-token) with its value set to the returned value, and return it.

Otherwise, return a <a id="ref-for-typedef-delim-token⑥"></a>[\<delim-token\>](#typedef-delim-token) with its value set to the <a id="ref-for-current-input-code-point⑦"></a>[current input code point](#current-input-code-point).

<strong>U+005B LEFT SQUARE BRACKET (\[)</strong>

Return a <a id="ref-for-tokendef-open-square"></a>[\<\[-token\>](#tokendef-open-square).

<strong>U+005C REVERSE SOLIDUS (&#92;)</strong>

If the input stream <a id="ref-for-check-if-two-code-points-are-a-valid-escape①"></a>[starts with a valid escape](#check-if-two-code-points-are-a-valid-escape), <a id="ref-for-reconsume-the-current-input-code-point④"></a>[reconsume the current input code point](#reconsume-the-current-input-code-point), <a id="ref-for-consume-an-ident-like-token①"></a>[consume an ident-like token](#consume-an-ident-like-token), and return it.

Tests

- [url-whitespace-consumption.html](https://wpt.fyi/results/css/css-syntax/url-whitespace-consumption.html) [(live test)](http://wpt.live/css/css-syntax/url-whitespace-consumption.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/url-whitespace-consumption.html)

Otherwise, this is a <a id="ref-for-parse-error"></a>[parse error](#parse-error). Return a <a id="ref-for-typedef-delim-token⑦"></a>[\<delim-token\>](#typedef-delim-token) with its value set to the <a id="ref-for-current-input-code-point⑧"></a>[current input code point](#current-input-code-point).

<strong>U+005D RIGHT SQUARE BRACKET (\])</strong>

Return a <a id="ref-for-tokendef-close-square"></a>[\<\]-token\>](#tokendef-close-square).

<strong>U+007B LEFT CURLY BRACKET ({)</strong>

Return a <a id="ref-for-tokendef-open-curly③"></a>[\<{-token\>](#tokendef-open-curly).

<strong>U+007D RIGHT CURLY BRACKET (})</strong>

Return a <a id="ref-for-tokendef-close-curly①"></a>[\<}-token\>](#tokendef-close-curly).

<strong><a id="ref-for-digit②"></a>[digit](#digit)</strong>

<a id="ref-for-reconsume-the-current-input-code-point⑤"></a>[Reconsume the current input code point](#reconsume-the-current-input-code-point), <a id="ref-for-consume-a-numeric-token③"></a>[consume a numeric token](#consume-a-numeric-token), and return it.

<strong>U+0055 LATIN CAPITAL LETTER U (U)</strong>

<strong>u+0075 LATIN LOWERCASE LETTER U (u)</strong>

If <var>unicode ranges allowed</var> is true and the input stream <a id="ref-for-check-if-three-code-points-would-start-a-unicode-range"></a>[would start a unicode-range](#check-if-three-code-points-would-start-a-unicode-range), <a id="ref-for-reconsume-the-current-input-code-point⑥"></a>[reconsume the current input code point](#reconsume-the-current-input-code-point), <a id="ref-for-consume-a-unicode-range-token"></a>[consume a unicode-range token](#consume-a-unicode-range-token), and return it.

Otherwise, <a id="ref-for-reconsume-the-current-input-code-point⑦"></a>[reconsume the current input code point](#reconsume-the-current-input-code-point), <a id="ref-for-consume-an-ident-like-token②"></a>[consume an ident-like token](#consume-an-ident-like-token), and return it.

<strong><a id="ref-for-ident-start-code-point②"></a>[ident-start code point](#ident-start-code-point)</strong>

<a id="ref-for-reconsume-the-current-input-code-point⑧"></a>[Reconsume the current input code point](#reconsume-the-current-input-code-point), <a id="ref-for-consume-an-ident-like-token③"></a>[consume an ident-like token](#consume-an-ident-like-token), and return it.

<strong>EOF</strong>

Return an <a id="ref-for-typedef-eof-token①"></a>[\<EOF-token\>](#typedef-eof-token).

<strong>anything else</strong>

Return a <a id="ref-for-typedef-delim-token⑧"></a>[\<delim-token\>](#typedef-delim-token) with its value set to the <a id="ref-for-current-input-code-point⑨"></a>[current input code point](#current-input-code-point).

Tests

- [cdc-vs-ident-tokens.html](https://wpt.fyi/results/css/css-syntax/cdc-vs-ident-tokens.html) [(live test)](http://wpt.live/css/css-syntax/cdc-vs-ident-tokens.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/cdc-vs-ident-tokens.html)
- [escaped-eof.html](https://wpt.fyi/results/css/css-syntax/escaped-eof.html) [(live test)](http://wpt.live/css/css-syntax/escaped-eof.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/escaped-eof.html)

#### <a id="consume-comment"></a>4.3.2.  Consume comments[](#consume-comment)

This section describes how to <a id="consume-comments"></a><strong>consume comments</strong> from a stream of <a id="ref-for-code-point③⑦"></a>[code points](https://infra.spec.whatwg.org/#code-point). It returns nothing.

If the <a id="ref-for-next-input-code-point⑨"></a>[next two input code point](#next-input-code-point) are U+002F SOLIDUS (/) followed by a U+002A ASTERISK (\*), consume them and all following <a id="ref-for-code-point③⑧"></a>[code points](https://infra.spec.whatwg.org/#code-point) up to and including the first U+002A ASTERISK (\*) followed by a U+002F SOLIDUS (/), or up to an EOF code point. Return to the start of this step.

If the preceding paragraph ended by consuming an EOF code point, this is a <a id="ref-for-parse-error①"></a>[parse error](#parse-error).

Return nothing.

#### <a id="consume-numeric-token"></a>4.3.3.  Consume a numeric token[](#consume-numeric-token)

This section describes how to <a id="consume-a-numeric-token"></a><strong>consume a numeric token</strong> from a stream of <a id="ref-for-code-point③⑨"></a>[code points](https://infra.spec.whatwg.org/#code-point). It returns either a <a id="ref-for-typedef-number-token③"></a>[\<number-token\>](#typedef-number-token), <a id="ref-for-typedef-percentage-token②"></a>[\<percentage-token\>](#typedef-percentage-token), or <a id="ref-for-typedef-dimension-token⑤"></a>[\<dimension-token\>](#typedef-dimension-token).

<a id="ref-for-consume-a-number"></a>[Consume a number](#consume-a-number) and let <var>number</var> be the result.

If the <a id="ref-for-next-input-code-point①⓪"></a>[next 3 input code points](#next-input-code-point) <a id="ref-for-check-if-three-code-points-would-start-an-ident-sequence③"></a>[would start an ident sequence](#check-if-three-code-points-would-start-an-ident-sequence), then:

1.  Create a <a id="ref-for-typedef-dimension-token⑥"></a>[\<dimension-token\>](#typedef-dimension-token) with the same value, type flag, and sign character as <var>number</var>, and a unit set initially to the empty string.
2.  <a id="ref-for-consume-an-ident-sequence②"></a>[Consume an ident sequence](#consume-an-ident-sequence). Set the <a id="ref-for-typedef-dimension-token⑦"></a>[\<dimension-token\>](#typedef-dimension-token)’s unit to the returned value.
3.  Return the <a id="ref-for-typedef-dimension-token⑧"></a>[\<dimension-token\>](#typedef-dimension-token).

Otherwise, if the <a id="ref-for-next-input-code-point①①"></a>[next input code point](#next-input-code-point) is U+0025 PERCENTAGE SIGN (%), consume it. Create a <a id="ref-for-typedef-percentage-token③"></a>[\<percentage-token\>](#typedef-percentage-token) with the same value and sign character as <var>number</var>, and return it.

Otherwise, create a <a id="ref-for-typedef-number-token④"></a>[\<number-token\>](#typedef-number-token) with the same value, type flag, and sign character as <var>number</var>, and return it.

#### <a id="consume-ident-like-token"></a>4.3.4.  Consume an ident-like token[](#consume-ident-like-token)

This section describes how to <a id="consume-an-ident-like-token"></a><strong>consume an ident-like token</strong> from a stream of <a id="ref-for-code-point④⓪"></a>[code points](https://infra.spec.whatwg.org/#code-point). It returns an <a id="ref-for-typedef-ident-token③"></a>[\<ident-token\>](#typedef-ident-token), <a id="ref-for-typedef-function-token③"></a>[\<function-token\>](#typedef-function-token), <a id="ref-for-typedef-url-token②"></a>[\<url-token\>](#typedef-url-token), or <a id="ref-for-typedef-bad-url-token"></a>[\<bad-url-token\>](#typedef-bad-url-token).

<a id="ref-for-consume-an-ident-sequence③"></a>[Consume an ident sequence](#consume-an-ident-sequence), and let <var>string</var> be the result.

If <var>string</var>’s value is an <a id="ref-for-ascii-case-insensitive"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "url", and the <a id="ref-for-next-input-code-point①②"></a>[next input code point](#next-input-code-point) is U+0028 LEFT PARENTHESIS ((), consume it. While the <a id="ref-for-next-input-code-point①③"></a>next two input code points are <a id="ref-for-whitespace③"></a>[whitespace](#whitespace), consume the <a id="ref-for-next-input-code-point①④"></a>next input code point. If the <a id="ref-for-next-input-code-point①⑤"></a>next one or two input code points are U+0022 QUOTATION MARK ("), U+0027 APOSTROPHE ('), or <a id="ref-for-whitespace④"></a>whitespace followed by U+0022 QUOTATION MARK (") or U+0027 APOSTROPHE ('), then create a <a id="ref-for-typedef-function-token④"></a>[\<function-token\>](#typedef-function-token) with its value set to <var>string</var> and return it. Otherwise, <a id="ref-for-consume-a-url-token"></a>[consume a url token](#consume-a-url-token), and return it.

Otherwise, if the <a id="ref-for-next-input-code-point①⑥"></a>[next input code point](#next-input-code-point) is U+0028 LEFT PARENTHESIS ((), consume it. Create a <a id="ref-for-typedef-function-token⑤"></a>[\<function-token\>](#typedef-function-token) with its value set to <var>string</var> and return it.

Otherwise, create an <a id="ref-for-typedef-ident-token④"></a>[\<ident-token\>](#typedef-ident-token) with its value set to <var>string</var> and return it.

#### <a id="consume-string-token"></a>4.3.5.  Consume a string token[](#consume-string-token)

This section describes how to <a id="consume-a-string-token"></a><strong>consume a string token</strong> from a stream of <a id="ref-for-code-point④①"></a>[code points](https://infra.spec.whatwg.org/#code-point). It returns either a <a id="ref-for-typedef-string-token②"></a>[\<string-token\>](#typedef-string-token) or <a id="ref-for-typedef-bad-string-token"></a>[\<bad-string-token\>](#typedef-bad-string-token).

This algorithm may be called with an <var>ending code point</var>, which denotes the <a id="ref-for-code-point④②"></a>[code point](https://infra.spec.whatwg.org/#code-point) that ends the string. If an <var>ending code point</var> is not specified, the <a id="ref-for-current-input-code-point①⓪"></a>[current input code point](#current-input-code-point) is used.

Initially create a <a id="ref-for-typedef-string-token③"></a>[\<string-token\>](#typedef-string-token) with its value set to the empty string.

Repeatedly consume the <a id="ref-for-next-input-code-point①⑦"></a>[next input code point](#next-input-code-point) from the stream:

<strong><var>ending code point</var></strong>

Return the <a id="ref-for-typedef-string-token④"></a>[\<string-token\>](#typedef-string-token).

<strong>EOF</strong>

This is a <a id="ref-for-parse-error②"></a>[parse error](#parse-error). Return the <a id="ref-for-typedef-string-token⑤"></a>[\<string-token\>](#typedef-string-token).

<strong><a id="ref-for-newline②"></a>[newline](#newline)</strong>

This is a <a id="ref-for-parse-error③"></a>[parse error](#parse-error). <a id="ref-for-reconsume-the-current-input-code-point⑨"></a>[Reconsume the current input code point](#reconsume-the-current-input-code-point), create a <a id="ref-for-typedef-bad-string-token①"></a>[\<bad-string-token\>](#typedef-bad-string-token), and return it.

<strong>U+005C REVERSE SOLIDUS (&#92;)</strong>

If the <a id="ref-for-next-input-code-point①⑧"></a>[next input code point](#next-input-code-point) is EOF, do nothing.

Otherwise, if the <a id="ref-for-next-input-code-point①⑨"></a>[next input code point](#next-input-code-point) is a newline, consume it.

Otherwise, (the stream

<strong>Note:</strong>

<a id="ref-for-check-if-two-code-points-are-a-valid-escape②"></a>[starts with a valid escape](#check-if-two-code-points-are-a-valid-escape)) <a id="ref-for-consume-an-escaped-code-point"></a>[consume an escaped code point](#consume-an-escaped-code-point) and append the returned <a id="ref-for-code-point④③"></a>[code point](https://infra.spec.whatwg.org/#code-point) to the <a id="ref-for-typedef-string-token⑥"></a>[\<string-token\>](#typedef-string-token)’s value.

<strong>anything else</strong>

Append the <a id="ref-for-current-input-code-point①①"></a>[current input code point](#current-input-code-point) to the <a id="ref-for-typedef-string-token⑦"></a>[\<string-token\>](#typedef-string-token)’s value.

Tests

- [escaped-eof.html](https://wpt.fyi/results/css/css-syntax/escaped-eof.html) [(live test)](http://wpt.live/css/css-syntax/escaped-eof.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/escaped-eof.html)

#### <a id="consume-url-token"></a>4.3.6.  Consume a url token[](#consume-url-token)

This section describes how to <a id="consume-a-url-token"></a><strong>consume a url token</strong> from a stream of <a id="ref-for-code-point④④"></a>[code points](https://infra.spec.whatwg.org/#code-point). It returns either a <a id="ref-for-typedef-url-token③"></a>[\<url-token\>](#typedef-url-token) or a <a id="ref-for-typedef-bad-url-token①"></a>[\<bad-url-token\>](#typedef-bad-url-token).

Note: This algorithm assumes that the initial "url(" has already been consumed. This algorithm also assumes that it’s being called to consume an "unquoted" value, like url(foo). A quoted value, like url("foo"), is parsed as a <a id="ref-for-typedef-function-token⑥"></a>[\<function-token\>](#typedef-function-token). <a id="ref-for-consume-an-ident-like-token④"></a>[Consume an ident-like token](#consume-an-ident-like-token) automatically handles this distinction; this algorithm shouldn’t be called directly otherwise.

1.  Initially create a <a id="ref-for-typedef-url-token④"></a>[\<url-token\>](#typedef-url-token) with its value set to the empty string.
2.  Consume as much <a id="ref-for-whitespace⑤"></a>[whitespace](#whitespace) as possible.
3.  Repeatedly consume the <a id="ref-for-next-input-code-point②⓪"></a>[next input code point](#next-input-code-point) from the stream:
    <strong>U+0029 RIGHT PARENTHESIS ())</strong>

    Return the <a id="ref-for-typedef-url-token⑤"></a>[\<url-token\>](#typedef-url-token).
    <strong>EOF</strong>

    This is a <a id="ref-for-parse-error④"></a>[parse error](#parse-error). Return the <a id="ref-for-typedef-url-token⑥"></a>[\<url-token\>](#typedef-url-token).
    <strong><a id="ref-for-whitespace⑥"></a>[whitespace](#whitespace)</strong>

    Consume as much <a id="ref-for-whitespace⑦"></a>[whitespace](#whitespace) as possible. If the <a id="ref-for-next-input-code-point②①"></a>[next input code point](#next-input-code-point) is U+0029 RIGHT PARENTHESIS ()) or EOF, consume it and return the <a id="ref-for-typedef-url-token⑦"></a>[\<url-token\>](#typedef-url-token) (if EOF was encountered, this is a <a id="ref-for-parse-error⑤"></a>[parse error](#parse-error)); otherwise, <a id="ref-for-consume-the-remnants-of-a-bad-url"></a>[consume the remnants of a bad url](#consume-the-remnants-of-a-bad-url), create a <a id="ref-for-typedef-bad-url-token②"></a>[\<bad-url-token\>](#typedef-bad-url-token), and return it.
    <strong>U+0022 QUOTATION MARK (")</strong>

    <strong>U+0027 APOSTROPHE (')</strong>

    <strong>U+0028 LEFT PARENTHESIS (()</strong>

    <strong><a id="ref-for-non-printable-code-point"></a>[non-printable code point](#non-printable-code-point)</strong>

    This is a <a id="ref-for-parse-error⑥"></a>[parse error](#parse-error). <a id="ref-for-consume-the-remnants-of-a-bad-url①"></a>[Consume the remnants of a bad url](#consume-the-remnants-of-a-bad-url), create a <a id="ref-for-typedef-bad-url-token③"></a>[\<bad-url-token\>](#typedef-bad-url-token), and return it.
    <strong>U+005C REVERSE SOLIDUS (&#92;)</strong>

    If the stream <a id="ref-for-check-if-two-code-points-are-a-valid-escape③"></a>[starts with a valid escape](#check-if-two-code-points-are-a-valid-escape), <a id="ref-for-consume-an-escaped-code-point①"></a>[consume an escaped code point](#consume-an-escaped-code-point) and append the returned <a id="ref-for-code-point④⑤"></a>[code point](https://infra.spec.whatwg.org/#code-point) to the <a id="ref-for-typedef-url-token⑧"></a>[\<url-token\>](#typedef-url-token)’s value.
    Otherwise, this is a <a id="ref-for-parse-error⑦"></a>[parse error](#parse-error). <a id="ref-for-consume-the-remnants-of-a-bad-url②"></a>[Consume the remnants of a bad url](#consume-the-remnants-of-a-bad-url), create a <a id="ref-for-typedef-bad-url-token④"></a>[\<bad-url-token\>](#typedef-bad-url-token), and return it.

    <strong>anything else</strong>

    Append the <a id="ref-for-current-input-code-point①②"></a>[current input code point](#current-input-code-point) to the <a id="ref-for-typedef-url-token⑨"></a>[\<url-token\>](#typedef-url-token)’s value.

Tests

- [unclosed-url-at-eof.html](https://wpt.fyi/results/css/css-syntax/unclosed-url-at-eof.html) [(live test)](http://wpt.live/css/css-syntax/unclosed-url-at-eof.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/unclosed-url-at-eof.html)

#### <a id="consume-escaped-code-point"></a>4.3.7.  Consume an escaped code point[](#consume-escaped-code-point)

This section describes how to <a id="consume-an-escaped-code-point"></a><strong>consume an escaped code point</strong>. It assumes that the U+005C REVERSE SOLIDUS (&#92;) has already been consumed and that the next input code point has already been verified to be part of a valid escape. It will return a <a id="ref-for-code-point④⑥"></a>[code point](https://infra.spec.whatwg.org/#code-point).

Consume the <a id="ref-for-next-input-code-point②②"></a>[next input code point](#next-input-code-point).

<strong><a id="ref-for-hex-digit②"></a>[hex digit](#hex-digit)</strong>

Consume as many <a id="ref-for-hex-digit③"></a>[hex digits](#hex-digit) as possible, but no more than 5. Note that this means 1-6 hex digits have been consumed in total.

<strong>Note:</strong>

If the <a id="ref-for-next-input-code-point②③"></a>[next input code point](#next-input-code-point) is <a id="ref-for-whitespace⑧"></a>[whitespace](#whitespace), consume it as well. Interpret the <a id="ref-for-hex-digit④"></a>hex digits as a hexadecimal number. If this number is zero, or is for a <a id="ref-for-surrogate②"></a>[surrogate](https://infra.spec.whatwg.org/#surrogate), or is greater than the <a id="ref-for-maximum-allowed-code-point"></a>[maximum allowed code point](#maximum-allowed-code-point), return U+FFFD REPLACEMENT CHARACTER (�). Otherwise, return the <a id="ref-for-code-point④⑦"></a>[code point](https://infra.spec.whatwg.org/#code-point) with that value.

<strong>EOF</strong>

This is a <a id="ref-for-parse-error⑧"></a>[parse error](#parse-error). Return U+FFFD REPLACEMENT CHARACTER (�).

<strong>anything else</strong>

Return the <a id="ref-for-current-input-code-point①③"></a>[current input code point](#current-input-code-point).

Tests

- [escaped-eof.html](https://wpt.fyi/results/css/css-syntax/escaped-eof.html) [(live test)](http://wpt.live/css/css-syntax/escaped-eof.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/escaped-eof.html)

#### <a id="starts-with-a-valid-escape"></a>4.3.8.  Check if two code points are a valid escape[](#starts-with-a-valid-escape)

This section describes how to <a id="check-if-two-code-points-are-a-valid-escape"></a><strong>check if two code points are a valid escape</strong>. The algorithm described here can be called explicitly with two <a id="ref-for-code-point④⑧"></a>[code points](https://infra.spec.whatwg.org/#code-point), or can be called with the input stream itself. In the latter case, the two <a id="ref-for-code-point④⑨"></a>code points in question are the <a id="ref-for-current-input-code-point①④"></a>[current input code point](#current-input-code-point) and the <a id="ref-for-next-input-code-point②④"></a>[next input code point](#next-input-code-point), in that order.

Note: This algorithm will not consume any additional <a id="ref-for-code-point⑤⓪"></a>[code point](https://infra.spec.whatwg.org/#code-point).

If the first <a id="ref-for-code-point⑤①"></a>[code point](https://infra.spec.whatwg.org/#code-point) is not U+005C REVERSE SOLIDUS (&#92;), return false.

Otherwise, if the second <a id="ref-for-code-point⑤②"></a>[code point](https://infra.spec.whatwg.org/#code-point) is a <a id="ref-for-newline③"></a>[newline](#newline), return false.

Otherwise, return true.

#### <a id="would-start-an-identifier"></a>4.3.9.  Check if three code points would start an ident sequence[](#would-start-an-identifier)

This section describes how to <a id="check-if-three-code-points-would-start-an-ident-sequence"></a><strong><a id="check-if-three-code-points-would-start-an-identifier"></a>check if three code points would start an <a id="ref-for-ident-sequence④"></a>[ident sequence](#ident-sequence)</strong>. The algorithm described here can be called explicitly with three <a id="ref-for-code-point⑤③"></a>[code points](https://infra.spec.whatwg.org/#code-point), or can be called with the input stream itself. In the latter case, the three <a id="ref-for-code-point⑤④"></a>code points in question are the <a id="ref-for-current-input-code-point①⑤"></a>[current input code point](#current-input-code-point) and the <a id="ref-for-next-input-code-point②⑤"></a>[next two input code points](#next-input-code-point), in that order.

Note: This algorithm will not consume any additional <a id="ref-for-code-point⑤⑤"></a>[code points](https://infra.spec.whatwg.org/#code-point).

Look at the first <a id="ref-for-code-point⑤⑥"></a>[code point](https://infra.spec.whatwg.org/#code-point):

<strong>U+002D HYPHEN-MINUS</strong>

If the second <a id="ref-for-code-point⑤⑦"></a>[code point](https://infra.spec.whatwg.org/#code-point) is an <a id="ref-for-ident-start-code-point③"></a>[ident-start code point](#ident-start-code-point) or a U+002D HYPHEN-MINUS, or the second and third <a id="ref-for-code-point⑤⑧"></a>code points <a id="ref-for-check-if-two-code-points-are-a-valid-escape④"></a>[are a valid escape](#check-if-two-code-points-are-a-valid-escape), return true. Otherwise, return false.

<strong><a id="ref-for-ident-start-code-point④"></a>[ident-start code point](#ident-start-code-point)</strong>

Return true.

<strong>U+005C REVERSE SOLIDUS (&#92;)</strong>

If the first and second <a id="ref-for-code-point⑤⑨"></a>[code points](https://infra.spec.whatwg.org/#code-point) <a id="ref-for-check-if-two-code-points-are-a-valid-escape⑤"></a>[are a valid escape](#check-if-two-code-points-are-a-valid-escape), return true. Otherwise, return false.

<strong>anything else</strong>

Return false.

Tests

- [ident-three-code-points.html](https://wpt.fyi/results/css/css-syntax/ident-three-code-points.html) [(live test)](http://wpt.live/css/css-syntax/ident-three-code-points.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/ident-three-code-points.html)

#### <a id="starts-with-a-number"></a>4.3.10.  Check if three code points would start a number[](#starts-with-a-number)

This section describes how to <a id="check-if-three-code-points-would-start-a-number"></a><strong>check if three code points would start a number</strong>. The algorithm described here can be called explicitly with three <a id="ref-for-code-point⑥⓪"></a>[code points](https://infra.spec.whatwg.org/#code-point), or can be called with the input stream itself. In the latter case, the three <a id="ref-for-code-point⑥①"></a>code points in question are the <a id="ref-for-current-input-code-point①⑥"></a>[current input code point](#current-input-code-point) and the <a id="ref-for-next-input-code-point②⑥"></a>[next two input code points](#next-input-code-point), in that order.

Note: This algorithm will not consume any additional <a id="ref-for-code-point⑥②"></a>[code points](https://infra.spec.whatwg.org/#code-point).

Look at the first <a id="ref-for-code-point⑥③"></a>[code point](https://infra.spec.whatwg.org/#code-point):

<strong>U+002B PLUS SIGN (+)</strong>

<strong>U+002D HYPHEN-MINUS (-)</strong>

If the second <a id="ref-for-code-point⑥④"></a>[code point](https://infra.spec.whatwg.org/#code-point) is a <a id="ref-for-digit③"></a>[digit](#digit), return true.

Otherwise, if the second <a id="ref-for-code-point⑥⑤"></a>[code point](https://infra.spec.whatwg.org/#code-point) is a U+002E FULL STOP (.) and the third <a id="ref-for-code-point⑥⑥"></a>code point is a <a id="ref-for-digit④"></a>[digit](#digit), return true.

Otherwise, return false.

<strong>U+002E FULL STOP (.)</strong>

If the second <a id="ref-for-code-point⑥⑦"></a>[code point](https://infra.spec.whatwg.org/#code-point) is a <a id="ref-for-digit⑤"></a>[digit](#digit), return true. Otherwise, return false.

<strong><a id="ref-for-digit⑥"></a>[digit](#digit)</strong>

Return true.

<strong>anything else</strong>

Return false.

#### <a id="starts-a-unicode-range"></a>4.3.11.  Check if three code points would start a unicode-range[](#starts-a-unicode-range)

This section describes how to <a id="check-if-three-code-points-would-start-a-unicode-range"></a><strong>check if three code points would start a unicode-range</strong>. The algorithm described here can be called explicitly with three <a id="ref-for-code-point⑥⑧"></a>[code points](https://infra.spec.whatwg.org/#code-point), or can be called with the input stream itself. In the latter case, the three <a id="ref-for-code-point⑥⑨"></a>code points in question are the <a id="ref-for-current-input-code-point①⑦"></a>[current input code point](#current-input-code-point) and the <a id="ref-for-next-input-code-point②⑦"></a>[next two input code points](#next-input-code-point), in that order.

Note: This algorithm will not consume any additional <a id="ref-for-code-point⑦⓪"></a>[code points](https://infra.spec.whatwg.org/#code-point).

If all of the following are true:

- The first code point is either U+0055 LATIN CAPITAL LETTER U (U) or U+0075 LATIN SMALL LETTER U (u)

- The second code point is U+002B PLUS SIGN (+).

- The third code point is either U+003F QUESTION MARK (?) or a <a id="ref-for-hex-digit⑤"></a>[hex digit](#hex-digit)

then return true.

Otherwise return false.

#### <a id="consume-name"></a>4.3.12.  Consume an ident sequence[](#consume-name)

This section describes how to <a id="consume-an-ident-sequence"></a><strong><a id="consume-an-identifier"></a>consume an ident sequence</strong> from a stream of <a id="ref-for-code-point⑦①"></a>[code points](https://infra.spec.whatwg.org/#code-point). It returns a string containing the largest name that can be formed from adjacent <a id="ref-for-code-point⑦②"></a>code points in the stream, starting from the first.

Note: This algorithm does not do the verification of the first few <a id="ref-for-code-point⑦③"></a>[code points](https://infra.spec.whatwg.org/#code-point) that are necessary to ensure the returned <a id="ref-for-code-point⑦④"></a>code points would constitute an <a id="ref-for-typedef-ident-token⑤"></a>[\<ident-token\>](#typedef-ident-token). If that is the intended use, ensure that the stream <a id="ref-for-check-if-three-code-points-would-start-an-ident-sequence④"></a>[starts with an ident sequence](#check-if-three-code-points-would-start-an-ident-sequence) before calling this algorithm.

Let <var>result</var> initially be an empty string.

Repeatedly consume the <a id="ref-for-next-input-code-point②⑧"></a>[next input code point](#next-input-code-point) from the stream:

<strong><a id="ref-for-ident-code-point②"></a>[ident code point](#ident-code-point)</strong>

Append the <a id="ref-for-code-point⑦⑤"></a>[code point](https://infra.spec.whatwg.org/#code-point) to <var>result</var>.

<strong>the stream <a id="ref-for-check-if-two-code-points-are-a-valid-escape⑥"></a>[starts with a valid escape](#check-if-two-code-points-are-a-valid-escape)</strong>

<a id="ref-for-consume-an-escaped-code-point②"></a>[Consume an escaped code point](#consume-an-escaped-code-point). Append the returned <a id="ref-for-code-point⑦⑥"></a>[code point](https://infra.spec.whatwg.org/#code-point) to <var>result</var>.

<strong>anything else</strong>

<a id="ref-for-reconsume-the-current-input-code-point①⓪"></a>[Reconsume the current input code point](#reconsume-the-current-input-code-point). Return <var>result</var>.

#### <a id="consume-number"></a>4.3.13.  Consume a number[](#consume-number)

This section describes how to <a id="consume-a-number"></a><strong>consume a number</strong> from a stream of <a id="ref-for-code-point⑦⑦"></a>[code points](https://infra.spec.whatwg.org/#code-point). It returns a numeric <var>value</var>, a string <var>type</var> which is either "integer" or "number", and an optional <var>sign character</var> which is either "+", "-", or missing.

Note: This algorithm does not do the verification of the first few <a id="ref-for-code-point⑦⑧"></a>[code points](https://infra.spec.whatwg.org/#code-point) that are necessary to ensure a number can be obtained from the stream. Ensure that the stream <a id="ref-for-check-if-three-code-points-would-start-a-number③"></a>[starts with a number](#check-if-three-code-points-would-start-a-number) before calling this algorithm.

Execute the following steps in order:

1.  Let <var>type</var> be the string "integer". Let <var>number part</var> and <var>exponent part</var> be the empty string.

2.  If the <a id="ref-for-next-input-code-point②⑨"></a>[next input code point](#next-input-code-point) is U+002B PLUS SIGN (+) or U+002D HYPHEN-MINUS (-), consume it. Append it to <var>number part</var> and set <var>sign character</var> to it.

3.  While the <a id="ref-for-next-input-code-point③⓪"></a>[next input code point](#next-input-code-point) is a <a id="ref-for-digit⑦"></a>[digit](#digit), consume it and append it to <var>number part</var>.

4.  If the <a id="ref-for-next-input-code-point③①"></a>[next 2 input code points](#next-input-code-point) are U+002E FULL STOP (.) followed by a <a id="ref-for-digit⑧"></a>[digit](#digit), then:
    1.  Consume the <a id="ref-for-next-input-code-point③②"></a>[next input code point](#next-input-code-point) and append it to <var>number part</var>.
    2.  While the <a id="ref-for-next-input-code-point③③"></a>[next input code point](#next-input-code-point) is a <a id="ref-for-digit⑨"></a>[digit](#digit), consume it and append it to <var>number part</var>.
    3.  Set <var>type</var> to "number".

5.  If the <a id="ref-for-next-input-code-point③④"></a>[next 2 or 3 input code points](#next-input-code-point) are U+0045 LATIN CAPITAL LETTER E (E) or U+0065 LATIN SMALL LETTER E (e), optionally followed by U+002D HYPHEN-MINUS (-) or U+002B PLUS SIGN (+), followed by a <a id="ref-for-digit①⓪"></a>[digit](#digit), then:
    1.  Consume the <a id="ref-for-next-input-code-point③⑤"></a>[next input code point](#next-input-code-point).
    2.  If the <a id="ref-for-next-input-code-point③⑥"></a>[next input code point](#next-input-code-point) is "+" or "-", consume it and append it to <var>exponent part</var>.
    3.  While the <a id="ref-for-next-input-code-point③⑦"></a>[next input code point](#next-input-code-point) is a <a id="ref-for-digit①①"></a>[digit](#digit), consume it and append it to <var>exponent part</var>.
    4.  Set <var>type</var> to "number".

6.  Let <var>value</var> be the result of interpreting <var>number part</var> as a base-10 number.

    If <var>exponent part</var> is non-empty, interpret it as a base-10 integer, then raise 10 to the power of the result, multiply it by <var>value</var>, and set <var>value</var> to that result.

7.  Return <var>value</var>, <var>type</var>, and <var>sign character</var>.

Tests

- [decimal-points-in-numbers.html](https://wpt.fyi/results/css/css-syntax/decimal-points-in-numbers.html) [(live test)](http://wpt.live/css/css-syntax/decimal-points-in-numbers.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/decimal-points-in-numbers.html)

#### <a id="consume-unicode-range-token"></a>4.3.14.  Consume a unicode-range token[](#consume-unicode-range-token)

This section describes how to <a id="consume-a-unicode-range-token"></a><strong>consume a unicode-range token</strong> from a stream of <a id="ref-for-code-point⑦⑨"></a>[code points](https://infra.spec.whatwg.org/#code-point). It returns a <a id="ref-for-typedef-unicode-range-token②"></a>[\<unicode-range-token\>](#typedef-unicode-range-token).

Note: This algorithm does not do the verification of the first few code points that are necessary to ensure the returned code points would constitute an <a id="ref-for-typedef-unicode-range-token③"></a>[\<unicode-range-token\>](#typedef-unicode-range-token). Ensure that the stream <a id="ref-for-check-if-three-code-points-would-start-a-unicode-range①"></a>[would start a unicode-range](#check-if-three-code-points-would-start-a-unicode-range) before calling this algorithm.

Note: This token is not produced by the tokenizer under normal circumstances. This algorithm is only called during <a id="ref-for-consume-the-value-of-a-unicode-range-descriptor"></a>[consume the value of a unicode-range descriptor](#consume-the-value-of-a-unicode-range-descriptor), which itself is only called as a special case for parsing the <a id="ref-for-descdef-font-face-unicode-range"></a>[unicode-range](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-unicode-range) descriptor; this single invocation in the entire language is due to a bad syntax design in early CSS.

1.  Consume the <a id="ref-for-next-input-code-point③⑧"></a>[next two input code points](#next-input-code-point) and discard them.

2.  Consume as many <a id="ref-for-hex-digit⑥"></a>[hex digits](#hex-digit) as possible, but no more than 6. If less than 6 hex digits were consumed, consume as many U+003F QUESTION MARK (?) code points as possible, but no more than enough to make the total of hex digits and U+003F QUESTION MARK (?) code points equal to 6.

    Let <var>first segment</var> be the consumed code points.

3.  If <var>first segment</var> contains any question mark code points, then:

    1.  Replace the question marks in <var>first segment</var> with U+0030 DIGIT ZERO (0) <a id="ref-for-code-point⑧⓪"></a>[code points](https://infra.spec.whatwg.org/#code-point), and interpret the result as a hexadecimal number. Let this be <var>start of range</var>.

    2.  Replace the question marks in <var>first segment</var> with U+0046 LATIN CAPITAL LETTER F (F) <a id="ref-for-code-point⑧①"></a>[code points](https://infra.spec.whatwg.org/#code-point), and interpret the result as a hexadecimal number. Let this be <var>end of range</var>.

    3.  Return a new <a id="ref-for-typedef-unicode-range-token④"></a>[\<unicode-range-token\>](#typedef-unicode-range-token) starting at <var>start of range</var> and ending at <var>end of range</var>.

4.  Otherwise, interpret <var>first segment</var> as a hexadecimal number, and let the result be <var>start of range</var>.

5.  If the <a id="ref-for-next-input-code-point③⑨"></a>[next 2 input code points](#next-input-code-point) are U+002D HYPHEN-MINUS (-) followed by a <a id="ref-for-hex-digit⑦"></a>[hex digit](#hex-digit), then:

    1.  Consume the <a id="ref-for-next-input-code-point④⓪"></a>[next input code point](#next-input-code-point).

    2.  Consume as many <a id="ref-for-hex-digit⑧"></a>[hex digits](#hex-digit) as possible, but no more than 6. Interpret the consumed code points as a hexadecimal number. Let this be <var>end of range</var>.

    3.  Return a new <a id="ref-for-typedef-unicode-range-token⑤"></a>[\<unicode-range-token\>](#typedef-unicode-range-token) starting at <var>start of range</var> and ending at <var>end of range</var>.

6.  Otherwise, return a new <a id="ref-for-typedef-unicode-range-token⑥"></a>[\<unicode-range-token\>](#typedef-unicode-range-token) both starting and ending at <var>start of range</var>.

#### <a id="consume-remnants-of-bad-url"></a>4.3.15.  Consume the remnants of a bad url[](#consume-remnants-of-bad-url)

This section describes how to <a id="consume-the-remnants-of-a-bad-url"></a><strong>consume the remnants of a bad url</strong> from a stream of <a id="ref-for-code-point⑧②"></a>[code points](https://infra.spec.whatwg.org/#code-point), "cleaning up" after the tokenizer realizes that it’s in the middle of a <a id="ref-for-typedef-bad-url-token⑤"></a>[\<bad-url-token\>](#typedef-bad-url-token) rather than a <a id="ref-for-typedef-url-token①⓪"></a>[\<url-token\>](#typedef-url-token). It returns nothing; its sole use is to consume enough of the input stream to reach a recovery point where normal tokenizing can resume.

Repeatedly consume the <a id="ref-for-next-input-code-point④①"></a>[next input code point](#next-input-code-point) from the stream:

<strong>U+0029 RIGHT PARENTHESIS ())</strong>

<strong>EOF</strong>

Return.

<strong>the input stream <a id="ref-for-check-if-two-code-points-are-a-valid-escape⑦"></a>[starts with a valid escape](#check-if-two-code-points-are-a-valid-escape)</strong>

<a id="ref-for-consume-an-escaped-code-point③"></a>[Consume an escaped code point](#consume-an-escaped-code-point). This allows an escaped right parenthesis (&#34;&#92;)&#34;) to be encountered without ending the

<strong>Note:</strong>

<a id="ref-for-typedef-bad-url-token⑥"></a>[\<bad-url-token\>](#typedef-bad-url-token). This is otherwise identical to the "anything else" clause.

<strong>anything else</strong>

Do nothing.

## <a id="parsing"></a>5.  Parsing[](#parsing)

The CSS parser converts a <a id="ref-for-css-token-stream"></a>[token stream](#css-token-stream) (produced by the tokenization process, defined earlier in this spec) into one or more of several CSS constructs (depending on which parsing algorithm is invoked).

### <a id="parser-diagrams"></a>5.1.  Parser Railroad Diagrams[](#parser-diagrams)

<em>This section is non-normative.</em>

This section presents an informative view of the parser, in the form of railroad diagrams.

These diagrams are <em>informative</em> and <em>incomplete</em>; they describe the grammar of "correct" stylesheets, but do not describe error-handling at all. They are provided solely to make it easier to get an intuitive grasp of the syntax.

<a id="stylesheet-diagram"></a><strong>[](#stylesheet-diagram)Stylesheet</strong>

![Source diagram 20](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-20.svg)

Diagram text: \<whitespace-token\> \<CDC-token\> \<CDO-token\> Qualified rule At-rule

<a id="at-rule-diagram"></a><strong>[](#at-rule-diagram)At-rule</strong>

![Source diagram 21](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-21.svg)

Diagram text: \<at-keyword-token\> Component value {} block ;

<a id="qualified-rule-diagram"></a><strong>[](#qualified-rule-diagram)Qualified rule</strong>

![Source diagram 22](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-22.svg)

Diagram text: Component value {} block

<a id="declaration-list-diagram"></a><strong>[](#declaration-list-diagram){} block</strong>

![Source diagram 23](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-23.svg)

Diagram text: { ws\* Declaration ; At-rule Qualified rule ws\* ws\* }

<a id="declaration-diagram"></a><strong>[](#declaration-diagram)Declaration</strong>

![Source diagram 24](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-24.svg)

Diagram text: \<ident-token\> ws\* : Component value !important

<a id="!important-diagram"></a><strong>[](#!important-diagram)!important</strong>

![Source diagram 25](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-25.svg)

Diagram text: ! ws\* \<ident-token "important"\> ws\*

<a id="component-value-diagram"></a><strong>[](#component-value-diagram)Component value</strong>

![Source diagram 26](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-26.svg)

Diagram text: Preserved token Simple block Function block

<a id="simple-block-diagram"></a><strong>[](#simple-block-diagram)Simple block</strong>

![Source diagram 27](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-27.svg)

Diagram text: { Component value } ( Component value ) \[ Component value \]

<a id="function-block-diagram"></a><strong>[](#function-block-diagram)Function block</strong>

![Source diagram 28](assets/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e--diagram-28.svg)

Diagram text: \<function-token\> Component value )

### <a id="css-tree"></a>5.2.  CSS Parsing Results[](#css-tree)

The result of parsing can be any of the following (or lists of these):

<strong><a id="css-stylesheet"></a><strong>stylesheet</strong></strong>

A stylesheet has a list of <a id="ref-for-css-rule"></a>[rules](#css-rule).

<strong><a id="css-rule"></a><strong>rule</strong></strong>

A <a id="ref-for-css-rule①"></a>[rule](#css-rule) is either an <a id="ref-for-at-rule①①"></a>[at-rule](#at-rule) or a <a id="ref-for-qualified-rule④"></a>[qualified rule](#qualified-rule).

<strong><a id="ref-for-at-rule①②"></a>[at-rule](#at-rule)</strong>

An <a id="ref-for-at-rule①③"></a>[at-rule](#at-rule) has a name which is a <a id="ref-for-string"></a>[string](https://infra.spec.whatwg.org/#string), a prelude consisting of a list of <a id="ref-for-component-value"></a>[component values](#component-value). <a id="ref-for-block-at-rule"></a>[Block at-rules](#block-at-rule) (ending in a {}-block) will additionally have a list of <a id="ref-for-declaration"></a>[declarations](#declaration) and a list of child <a id="ref-for-css-rule②"></a>[rules](#css-rule). (<a id="ref-for-statement-at-rule"></a>[Statement at-rules](#statement-at-rule), ending in a semicolon, will not.)

<strong><a id="qualified-rule"></a><strong>qualified rule</strong></strong>

A qualified rule has a prelude consisting of a list of component values, a list of declarations, and a list of child rules.

Note: Most qualified rules will be style rules, where the prelude is a selector [\[selectors-4\]](#biblio-selectors-4) and its declarations are <a id="ref-for-css-property"></a>[properties](https://www.w3.org/TR/css-cascade-5/#css-property).

<strong><a id="declaration"></a><strong>declaration</strong></strong>

A declaration has a name which is a <a id="ref-for-string①"></a>[string](https://infra.spec.whatwg.org/#string), a value consisting of a list of <a id="ref-for-component-value①"></a>[component values](#component-value), and an <var>important</var> flag which is initially unset. It also has an optional <var>original text</var> which is a <a id="ref-for-string②"></a>string (captured for only a few declarations).

Declarations are further categorized as <a id="css-property-declarations"></a><strong>property declarations</strong> or <a id="css-descriptor-declarations"></a><strong>descriptor declarations</strong>, with the former setting CSS <a id="ref-for-css-property①"></a>[properties](https://www.w3.org/TR/css-cascade-5/#css-property) and appearing most often in <a id="ref-for-qualified-rule⑤"></a>[qualified rules](#qualified-rule) and the latter setting CSS <a id="ref-for-css-descriptor"></a>[descriptors](#css-descriptor), which appear only in <a id="ref-for-at-rule①④"></a>[at-rules](#at-rule). (This categorization does not occur at the Syntax level; instead, it is a product of where the declaration appears, and is defined by the respective specifications defining the given rule.)

<strong><a id="component-value"></a><strong>component value</strong></strong>

A component value is one of the <a id="ref-for-preserved-tokens"></a>[preserved tokens](#preserved-tokens), a <a id="ref-for-function"></a>[function](#function), or a <a id="ref-for-simple-block"></a>[simple block](#simple-block).

<strong><a id="preserved-tokens"></a><strong>preserved tokens</strong></strong>

Any token produced by the tokenizer except for <a id="ref-for-typedef-function-token⑦"></a>[\<function-token\>](#typedef-function-token)s, <a id="ref-for-tokendef-open-curly④"></a>[\<{-token\>](#tokendef-open-curly)s, <a id="ref-for-tokendef-open-paren①"></a>[\<(-token\>](#tokendef-open-paren)s, and <a id="ref-for-tokendef-open-square①"></a>[\<\[-token\>](#tokendef-open-square)s.

Note: The non-<a id="ref-for-preserved-tokens①"></a>[preserved tokens](#preserved-tokens) listed above are always consumed into higher-level objects, either functions or simple blocks, and so never appear in any parser output themselves.

Note: The tokens <a id="ref-for-tokendef-close-curly②"></a>[\<}-token\>](#tokendef-close-curly)s, <a id="ref-for-tokendef-close-paren①"></a>[\<)-token\>](#tokendef-close-paren)s, <a id="ref-for-tokendef-close-square①"></a>[\<\]-token\>](#tokendef-close-square), <a id="ref-for-typedef-bad-string-token②"></a>[\<bad-string-token\>](#typedef-bad-string-token), and <a id="ref-for-typedef-bad-url-token⑦"></a>[\<bad-url-token\>](#typedef-bad-url-token) are always parse errors, but they are preserved in the token stream by this specification to allow other specs, such as Media Queries, to define more fine-grained error-handling than just dropping an entire declaration or block.

<strong><a id="function"></a><strong>function</strong></strong>

A function has a name and a value consisting of a list of <a id="ref-for-component-value②"></a>[component values](#component-value).

<strong><a id="simple-block"></a><strong>simple block</strong></strong>

<strong><a id="curly-block"></a><strong>{}-block</strong></strong>

<strong><a id="square-block"></a><strong>\[\]-block</strong></strong>

<strong><a id="paren-block"></a><strong>()-block</strong></strong>

A simple block has an associated token (either a <a id="ref-for-tokendef-open-square②"></a>[\<\[-token\>](#tokendef-open-square), <a id="ref-for-tokendef-open-paren②"></a>[\<(-token\>](#tokendef-open-paren), or <a id="ref-for-tokendef-open-curly⑤"></a>[\<{-token\>](#tokendef-open-curly)) and a value consisting of a list of component values.

<a id="ref-for-curly-block"></a>[{}-block](#curly-block), <a id="ref-for-square-block"></a>[\[\]-block](#square-block), and <a id="ref-for-paren-block"></a>[()-block](#paren-block) refer specifically to a <a id="ref-for-simple-block①"></a>[simple block](#simple-block) with that corresponding associated token.

Tests

- [parse-invalid-input.html](https://wpt.fyi/results/css/css-syntax/parse-invalid-input.html) [(live test)](http://wpt.live/css/css-syntax/parse-invalid-input.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/parse-invalid-input.html)

### <a id="parser-definitions"></a>5.3.  Token Streams[](#parser-definitions)

A <a id="css-token-stream"></a><strong>token stream</strong> is a <a id="ref-for-struct"></a>[struct](https://infra.spec.whatwg.org/#struct) representing a stream of <a id="ref-for-token-stream-tokens"></a>[tokens](#token-stream-tokens) and/or <a id="ref-for-component-value③"></a>[component values](#component-value). It has the following <a id="ref-for-struct-item"></a>[items](https://infra.spec.whatwg.org/#struct-item):

<strong><a id="token-stream-tokens"></a><strong>tokens</strong></strong>

A <a id="ref-for-list"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-token-stream-tokens①"></a>[tokens](#token-stream-tokens) and/or <a id="ref-for-component-value④"></a>[component values](#component-value).

Note: This specification assumes, for simplicity, that the input stream has been fully tokenized before parsing begins. However, the parsing algorithms only use one token of "lookahead", so in practice tokenization and parsing can be done in lockstep.

<strong><a id="token-stream-index"></a><strong>index</strong></strong>

An index into the <a id="ref-for-token-stream-tokens②"></a>[tokens](#token-stream-tokens), representing the progress of parsing. It starts at 0 initially.

Note: Aside from <a id="ref-for-token-stream-mark"></a>[marking](#token-stream-mark), the <a id="ref-for-token-stream-index"></a>[index](#token-stream-index) never goes backwards. Thus the already-processed prefix of <a id="ref-for-token-stream-tokens③"></a>[tokens](#token-stream-tokens) can be eagerly discarded as it’s processed.

<strong><a id="token-stream-marked-indexes"></a><strong>marked indexes</strong></strong>

A <a id="ref-for-stack"></a>[stack](https://infra.spec.whatwg.org/#stack) of index values, representing points that the parser might return to. It starts empty initially.

CSS has a small number of places that require referencing the precise text that was parsed for a declaration’s value (not just what tokens were produced from that text). This is not explicitly described in the algorithmic structure here, but the <a id="ref-for-css-token-stream①"></a>[token stream](#css-token-stream) must, thus, have the ability to reproduce the original text of declarations on demand. See <a id="ref-for-consume-a-declaration"></a>[consume a declaration](#consume-a-declaration) for details on when this is required.

Several operations can be performed on a <a id="ref-for-css-token-stream②"></a>[token stream](#css-token-stream):

<strong><a id="token-stream-next-token"></a><strong>next token</strong></strong>

The item of <a id="ref-for-token-stream-tokens④"></a>[tokens](#token-stream-tokens) at <a id="ref-for-token-stream-index①"></a>[index](#token-stream-index).

If that index would be out-of-bounds past the end of the list, it’s instead an <a id="ref-for-typedef-eof-token②"></a>[\<eof-token\>](#typedef-eof-token).

<strong><a id="token-stream-empty"></a><strong>empty</strong></strong>

A token stream is <a id="ref-for-token-stream-empty"></a>[empty](#token-stream-empty) if the <a id="ref-for-token-stream-next-token"></a>[next token](#token-stream-next-token) is an <a id="ref-for-typedef-eof-token③"></a>[\<eof-token\>](#typedef-eof-token).

<strong><a id="token-stream-consume-a-token"></a><strong>consume a token</strong></strong>

Let <var>token</var> be the <a id="ref-for-token-stream-next-token①"></a>[next token](#token-stream-next-token). Increment <a id="ref-for-token-stream-index②"></a>[index](#token-stream-index), then return <var>token</var>.

<strong><a id="token-stream-discard-a-token"></a><strong>discard a token</strong></strong>

If the <a id="ref-for-css-token-stream③"></a>[token stream](#css-token-stream) is not <a id="ref-for-token-stream-empty①"></a>[empty](#token-stream-empty), increment <a id="ref-for-token-stream-index③"></a>[index](#token-stream-index).

<strong><a id="token-stream-mark"></a><strong>mark</strong></strong>

Append <a id="ref-for-token-stream-index④"></a>[index](#token-stream-index) to <a id="ref-for-token-stream-marked-indexes"></a>[marked indexes](#token-stream-marked-indexes).

<strong><a id="token-stream-restore-a-mark"></a><strong>restore a mark</strong></strong>

<a id="ref-for-stack-pop"></a>[Pop](https://infra.spec.whatwg.org/#stack-pop) from <a id="ref-for-token-stream-marked-indexes①"></a>[marked indexes](#token-stream-marked-indexes), and set <a id="ref-for-token-stream-index⑤"></a>[index](#token-stream-index) to the popped value.

<strong><a id="token-stream-discard-a-mark"></a><strong>discard a mark</strong></strong>

<a id="ref-for-stack-pop①"></a>[Pop](https://infra.spec.whatwg.org/#stack-pop) from <a id="ref-for-token-stream-marked-indexes②"></a>[marked indexes](#token-stream-marked-indexes), and do nothing with the popped value.

<strong><a id="token-stream-discard-whitespace"></a><strong>discard whitespace</strong></strong>

While the <a id="ref-for-token-stream-next-token②"></a>[next token](#token-stream-next-token) is a <a id="ref-for-typedef-whitespace-token②"></a>[\<whitespace-token\>](#typedef-whitespace-token), <a id="ref-for-token-stream-discard-a-token"></a>[discard a token](#token-stream-discard-a-token).

<strong><a id="token-stream-process"></a><strong>process</strong></strong>

To <a id="ref-for-token-stream-process"></a>[process](#token-stream-process), given a following list of token types and associated actions, perform the action associated with the <a id="ref-for-token-stream-next-token③"></a>[next token](#token-stream-next-token). Repeat until one of the actions returns something, then return that.

An <a id="typedef-eof-token"></a><strong><a id="ref-for-typedef-eof-token④"></a>[\<eof-token\>](#typedef-eof-token)</strong> is a conceptual token, not actually produced by the tokenizer, used to indicate that the <a id="ref-for-css-token-stream④"></a>[token stream](#css-token-stream) has been exhausted.

### <a id="parser-entry-points"></a>5.4.  Parser Entry Points[](#parser-entry-points)

The algorithms defined in this section produce high-level CSS objects from lists of CSS tokens.

The algorithms here operate on a token stream as input, but for convenience can also be invoked with a number of other value types.

To <a id="normalize-into-a-token-stream"></a><strong>normalize into a token stream</strong> a given <var>input</var>:

1.  If <var>input</var> is already a <a id="ref-for-css-token-stream⑤"></a>[token stream](#css-token-stream), return it.

2.  If <var>input</var> is a list of CSS <a id="ref-for-token-stream-tokens⑤"></a>[tokens](#token-stream-tokens) and/or <a id="ref-for-component-value⑤"></a>[component values](#component-value), create a new <a id="ref-for-css-token-stream⑥"></a>[token stream](#css-token-stream) with <var>input</var> as its <a id="ref-for-token-stream-tokens⑥"></a>tokens, and return it.

3.  If <var>input</var> is a <a id="ref-for-string③"></a>[string](https://infra.spec.whatwg.org/#string), then <a id="ref-for-css-filter-code-points①"></a>[filter code points](#css-filter-code-points) from <var>input</var>, <a id="ref-for-css-tokenize"></a>[tokenize](#css-tokenize) the result, and return the stream of tokens.

4.  Assert: Only the preceding types should be passed as <var>input</var>.

Note: Other specs can define additional entry points for their own purposes.

The following notes should probably be translated into normative text in the relevant specs, hooking this spec’s terms:

<strong>Note:</strong>

- "<a id="ref-for-parse-a-stylesheet"></a>[Parse a stylesheet](#parse-a-stylesheet)" is intended to be the normal parser entry point, for parsing stylesheets.
- "<a id="ref-for-parse-a-stylesheets-contents"></a>[Parse a stylesheet’s contents](#parse-a-stylesheets-contents)" is intended for use by the <code><a id="ref-for-dom-cssstylesheet-replace"></a>[CSSStyleSheet replace()](https://www.w3.org/TR/cssom-1/#dom-cssstylesheet-replace)</code> method, and similar, which parse text into the contents of an existing stylesheet.
- "<a id="ref-for-parse-a-rule"></a>[Parse a rule](#parse-a-rule)" is intended for use by the <code><a id="ref-for-dom-cssstylesheet-insertrule"></a>[CSSStyleSheet insertRule()](https://www.w3.org/TR/cssom-1/#dom-cssstylesheet-insertrule)</code> method, and similar, which parse text into a single rule. <code>CSSStyleSheet&#35;insertRule</code> method, and similar functions which might exist, which parse text into a single rule.
- "<a id="ref-for-parse-a-declaration"></a>[Parse a declaration](#parse-a-declaration)" is used in <a id="ref-for-at-ruledef-supports"></a>[&#64;supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) conditions. [\[CSS3-CONDITIONAL\]](#biblio-css3-conditional)
- "<a id="ref-for-parse-a-blocks-contents"></a>[Parse a block’s contents](#parse-a-blocks-contents)" is intended for parsing the contents of any block in CSS (including things like the style attribute), and APIs such as <code><a id="ref-for-dom-cssstyledeclaration-csstext"></a>[the CSSStyleDeclaration cssText attribute](https://www.w3.org/TR/cssom-1/#dom-cssstyledeclaration-csstext)</code>.
- "<a id="ref-for-parse-a-component-value"></a>[Parse a component value](#parse-a-component-value)" is for things that need to consume a single value, like the parsing rules for <a id="ref-for-funcdef-attr"></a>[attr()](https://www.w3.org/TR/css-values-5/#funcdef-attr).
- "<a id="ref-for-parse-a-list-of-component-values"></a>[Parse a list of component values](#parse-a-list-of-component-values)" is for the contents of presentational attributes, which parse text into a single declaration’s value, or for parsing a stand-alone selector [\[SELECT\]](#biblio-select) or list of Media Queries [\[MEDIAQ\]](#biblio-mediaq), as in [Selectors API](https://www.w3.org/TR/selectors-api/) or the <code>media</code> HTML attribute.

#### <a id="parse-grammar"></a>5.4.1.  Parse something according to a CSS grammar[](#parse-grammar)

It is often desirable to parse a string or token list to see if it matches some CSS grammar, and if it does, to destructure it according to the grammar. This section provides a generic hook for this kind of operation. It should be invoked like "parse <var>foo</var> as a CSS <a id="ref-for-typedef-color"></a>[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)", or similar.

This algorithm returns either failure, if the input does not match the provided grammar, or the result of parsing the input according to the grammar, which is an unspecified structure corresponding to the provided grammar specification. The return value must only be interacted with by specification prose, where the representation ambiguity is not problematic. If it is meant to be exposed outside of spec language, the spec using the result must explicitly translate it into a well-specified representation, such as, for example, by invoking a CSS serialization algorithm (like "serialize as a CSS <a id="ref-for-string-value"></a>[\<string\>](https://www.w3.org/TR/css-values-4/#string-value) value").

Note: This algorithm, and <a id="ref-for-css-parse-a-comma-separated-list-according-to-a-css-grammar"></a>[parse a comma-separated list according to a CSS grammar](#css-parse-a-comma-separated-list-according-to-a-css-grammar), are <em>usually</em> the only parsing algorithms other specs will want to call. The remaining parsing algorithms are meant mostly for [\[CSSOM\]](#biblio-cssom) and related "explicitly constructing CSS structures" cases. Consult the CSSWG for guidance first if you think you need to use one of the other algorithms.

To <a id="css-parse-something-according-to-a-css-grammar"></a><strong>parse something according to a CSS grammar</strong> (aka simply <a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>[parse](#css-parse-something-according-to-a-css-grammar)) given an <var>input</var> and a CSS <var>grammar</var> production:

1.  <a id="ref-for-normalize-into-a-token-stream"></a>[Normalize](#normalize-into-a-token-stream) <var>input</var>, and set <var>input</var> to the result.
2.  <a id="ref-for-parse-a-list-of-component-values①"></a>[Parse a list of component values](#parse-a-list-of-component-values) from <var>input</var>, and let <var>result</var> be the return value.
3.  Attempt to match <var>result</var> against <var>grammar</var>. If this is successful, return the matched result; otherwise, return failure.

#### <a id="parse-comma-list"></a>5.4.2.  Parse a comma-separated list according to a CSS grammar[](#parse-comma-list)

While one can definitely <a id="ref-for-css-parse-something-according-to-a-css-grammar①"></a>[parse](#css-parse-something-according-to-a-css-grammar) a value according to a grammar with commas in it, if <em>any</em> part of the value fails to parse, the entire thing doesn’t parse, and returns failure.

Sometimes that’s what’s desired (such as in list-valued CSS properties); other times, it’s better to let each comma-separated sub-part of the value parse separately, dealing with the parts that parse successfully one way, and the parts that fail to parse another way (typically ignoring them, such as in <code><a id="ref-for-attr-img-sizes"></a>[\<img sizes\>](https://html.spec.whatwg.org/multipage/embedded-content.html#attr-img-sizes)</code>).

This algorithm provides an easy hook to accomplish exactly that. It returns a list of values split by "top-level" commas, where each values is either failure (if it failed to parse) or the result of parsing (an unspecified structure, as described in the <a id="ref-for-css-parse-something-according-to-a-css-grammar②"></a>[parse](#css-parse-something-according-to-a-css-grammar) algorithm).

To <a id="css-parse-a-comma-separated-list-according-to-a-css-grammar"></a><strong>parse a comma-separated list according to a CSS grammar</strong> (aka <a id="ref-for-css-parse-a-comma-separated-list-according-to-a-css-grammar①"></a>[parse a list](#css-parse-a-comma-separated-list-according-to-a-css-grammar)) given an <var>input</var> and a CSS <var>grammar</var> production:

1.  <a id="ref-for-normalize-into-a-token-stream①"></a>[Normalize](#normalize-into-a-token-stream) <var>input</var>, and set <var>input</var> to the result.
2.  If <var>input</var> contains only <a id="ref-for-typedef-whitespace-token③"></a>[\<whitespace-token\>](#typedef-whitespace-token)s, return an empty <a id="ref-for-list①"></a>[list](https://infra.spec.whatwg.org/#list).
3.  <a id="ref-for-parse-a-comma-separated-list-of-component-values"></a>[Parse a comma-separated list of component values](#parse-a-comma-separated-list-of-component-values) from <var>input</var>, and let <var>list</var> be the return value.
4.  <a id="ref-for-list-iterate"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>item</var> of <var>list</var>, replace <var>item</var> with the result of <a id="ref-for-css-parse-something-according-to-a-css-grammar③"></a>[parsing](#css-parse-something-according-to-a-css-grammar) <var>item</var> with <var>grammar</var>.
5.  Return <var>list</var>.

#### <a id="parse-stylesheet"></a>5.4.3.  Parse a stylesheet[](#parse-stylesheet)

To <a id="parse-a-stylesheet"></a><strong>parse a stylesheet</strong> from an <var>input</var> given an optional <a id="ref-for-concept-url"></a>[url](https://url.spec.whatwg.org/#concept-url) <var>location</var>:

1.  If <var>input</var> is a byte stream for a stylesheet, <a id="ref-for-css-decode-bytes"></a>[decode bytes](#css-decode-bytes) from <var>input</var>, and set <var>input</var> to the result.
2.  <a id="ref-for-normalize-into-a-token-stream②"></a>[Normalize](#normalize-into-a-token-stream) <var>input</var>, and set <var>input</var> to the result.
3.  Create a new stylesheet, with its <a id="ref-for-concept-css-style-sheet-location"></a>[location](https://www.w3.org/TR/cssom-1/#concept-css-style-sheet-location) set to <var>location</var> (or null, if <var>location</var> was not passed).
4.  <a id="ref-for-consume-a-stylesheets-contents"></a>[Consume a stylesheet’s contents](#consume-a-stylesheets-contents) from <var>input</var>, and set the stylesheet’s rules to the result.
5.  Return the stylesheet.

#### <a id="parse-stylesheet-contents"></a>5.4.4. <a id="parse-list-of-rules"></a> Parse a stylesheet’s contents[](#parse-stylesheet-contents)

To <a id="parse-a-stylesheets-contents"></a><strong>parse a stylesheet’s contents</strong> from <var>input</var>:

1.  <a id="ref-for-normalize-into-a-token-stream③"></a>[Normalize](#normalize-into-a-token-stream) <var>input</var>, and set <var>input</var> to the result.
2.  <a id="ref-for-consume-a-stylesheets-contents①"></a>[Consume a stylesheet’s contents](#consume-a-stylesheets-contents) from <var>input</var>, and return the result.

#### <a id="parse-block-contents"></a>5.4.5. <a id="parse-list-of-declarations"></a> Parse a block’s contents[](#parse-block-contents)

To <a id="parse-a-blocks-contents"></a><strong>parse a block’s contents</strong> from <var>input</var>:

1.  <a id="ref-for-normalize-into-a-token-stream④"></a>[Normalize](#normalize-into-a-token-stream) <var>input</var>, and set <var>input</var> to the result.
2.  <a id="ref-for-consume-a-blocks-contents"></a>[Consume a block’s contents](#consume-a-blocks-contents) from <var>input</var>, and return the result.

#### <a id="parse-rule"></a>5.4.6.  Parse a rule[](#parse-rule)

To <a id="parse-a-rule"></a><strong>parse a rule</strong> from <var>input</var>:

1.  <a id="ref-for-normalize-into-a-token-stream⑤"></a>[Normalize](#normalize-into-a-token-stream) <var>input</var>, and set <var>input</var> to the result.

2.  <a id="ref-for-token-stream-discard-whitespace"></a>[Discard whitespace](#token-stream-discard-whitespace) from <var>input</var>.

3.  If the <a id="ref-for-token-stream-next-token④"></a>[next token](#token-stream-next-token) from <var>input</var> is an <a id="ref-for-typedef-eof-token⑤"></a>[\<EOF-token\>](#typedef-eof-token), return a syntax error.

    Otherwise, if the <a id="ref-for-token-stream-next-token⑤"></a>[next token](#token-stream-next-token) from <var>input</var> is an <a id="ref-for-typedef-at-keyword-token⑤"></a>[\<at-keyword-token\>](#typedef-at-keyword-token), <a id="ref-for-consume-an-at-rule"></a>[consume an at-rule](#consume-an-at-rule) from <var>input</var>, and let <var>rule</var> be the return value.

    Otherwise, <a id="ref-for-consume-a-qualified-rule"></a>[consume a qualified rule](#consume-a-qualified-rule) from <var>input</var> and let <var>rule</var> be the return value. If nothing or an <a id="ref-for-invalid-rule-error"></a>[invalid rule error](#invalid-rule-error) was returned, return a syntax error.

4.  <a id="ref-for-token-stream-discard-whitespace①"></a>[Discard whitespace](#token-stream-discard-whitespace) from <var>input</var>.

5.  If the <a id="ref-for-token-stream-next-token⑥"></a>[next token](#token-stream-next-token) from <var>input</var> is an <a id="ref-for-typedef-eof-token⑥"></a>[\<EOF-token\>](#typedef-eof-token), return <var>rule</var>. Otherwise, return a syntax error.

#### <a id="parse-declaration"></a>5.4.7.  Parse a declaration[](#parse-declaration)

Note: Unlike "<a id="ref-for-parse-a-list-of-declarations"></a>[Parse a list of declarations](https://www.w3.org/TR/css-syntax-3/#parse-a-list-of-declarations)", this parses only a declaration and not an at-rule.

To <a id="parse-a-declaration"></a><strong>parse a declaration</strong> from <var>input</var>:

1.  <a id="ref-for-normalize-into-a-token-stream⑥"></a>[Normalize](#normalize-into-a-token-stream) <var>input</var>, and set <var>input</var> to the result.
2.  <a id="ref-for-token-stream-discard-whitespace②"></a>[Discard whitespace](#token-stream-discard-whitespace) from <var>input</var>.
3.  <a id="ref-for-consume-a-declaration①"></a>[Consume a declaration](#consume-a-declaration) from <var>input</var>. If anything was returned, return it. Otherwise, return a syntax error.

#### <a id="parse-component-value"></a>5.4.8.  Parse a component value[](#parse-component-value)

To <a id="parse-a-component-value"></a><strong>parse a component value</strong> from <var>input</var>:

1.  <a id="ref-for-normalize-into-a-token-stream⑦"></a>[Normalize](#normalize-into-a-token-stream) <var>input</var>, and set <var>input</var> to the result.
2.  <a id="ref-for-token-stream-discard-whitespace③"></a>[Discard whitespace](#token-stream-discard-whitespace) from <var>input</var>.
3.  If <var>input</var> is <a id="ref-for-token-stream-empty②"></a>[empty](#token-stream-empty), return a syntax error.
4.  <a id="ref-for-consume-a-component-value"></a>[Consume a component value](#consume-a-component-value) from <var>input</var> and let <var>value</var> be the return value.
5.  <a id="ref-for-token-stream-discard-whitespace④"></a>[Discard whitespace](#token-stream-discard-whitespace) from <var>input</var>.
6.  If <var>input</var> is <a id="ref-for-token-stream-empty③"></a>[empty](#token-stream-empty), return <var>value</var>. Otherwise, return a syntax error.

#### <a id="parse-list-of-component-values"></a>5.4.9.  Parse a list of component values[](#parse-list-of-component-values)

To <a id="parse-a-list-of-component-values"></a><strong>parse a list of component values</strong> from <var>input</var>:

1.  <a id="ref-for-normalize-into-a-token-stream⑧"></a>[Normalize](#normalize-into-a-token-stream) <var>input</var>, and set <var>input</var> to the result.

2.  <a id="ref-for-consume-a-list-of-component-values"></a>[Consume a list of component values](#consume-a-list-of-component-values) from <var>input</var>, and return the result.

#### <a id="parse-comma-separated-list-of-component-values"></a>5.4.10.  Parse a comma-separated list of component values[](#parse-comma-separated-list-of-component-values)

To <a id="parse-a-comma-separated-list-of-component-values"></a><strong>parse a comma-separated list of component values</strong> from <var>input</var>:

1.  <a id="ref-for-normalize-into-a-token-stream⑨"></a>[Normalize](#normalize-into-a-token-stream) <var>input</var>, and set <var>input</var> to the result.

2.  Let <var>groups</var> be an empty <a id="ref-for-list②"></a>[list](https://infra.spec.whatwg.org/#list).

3.  While <var>input</var> is not <a id="ref-for-token-stream-empty④"></a>[empty](#token-stream-empty):

    1.  <a id="ref-for-consume-a-list-of-component-values①"></a>[Consume a list of component values](#consume-a-list-of-component-values) from <var>input</var>, with <a id="ref-for-typedef-comma-token①"></a>[\<comma-token\>](#typedef-comma-token) as the stop token, and append the result to <var>groups</var>.

    2.  <a id="ref-for-token-stream-discard-a-token①"></a>[Discard a token](#token-stream-discard-a-token) from <var>input</var>.

4.  Return <var>groups</var>.

### <a id="parser-algorithms"></a>5.5.  Parser Algorithms[](#parser-algorithms)

The following algorithms comprise the parser. They are called by the parser entry points above, and generally should not be called directly by other specifications.

Note that CSS parsing is case-sensitive, and checking the validity of constructs in a given context must be done <em>during</em> parsing in at least some circumstances. This specification intentionally does not specify <em>how</em> sufficient context should be passed around to enable validity-checking.

#### <a id="consume-stylesheet-contents"></a>5.5.1. <a id="consume-list-of-rules"></a> Consume a stylesheet’s contents[](#consume-stylesheet-contents)

To <a id="consume-a-stylesheets-contents"></a><strong>consume a stylesheet’s contents</strong> from a <a id="ref-for-css-token-stream⑦"></a>[token stream](#css-token-stream) <var>input</var>:

Let <var>rules</var> be an initially empty <a id="ref-for-list③"></a>[list](https://infra.spec.whatwg.org/#list) of rules.

<a id="ref-for-token-stream-process①"></a>[Process](#token-stream-process) <var>input</var>:

<strong><a id="ref-for-typedef-whitespace-token④"></a>[\<whitespace-token\>](#typedef-whitespace-token)</strong>

<a id="ref-for-token-stream-discard-a-token②"></a>[Discard a token](#token-stream-discard-a-token) from <var>input</var>.

<strong><a id="ref-for-typedef-eof-token⑦"></a>[\<EOF-token\>](#typedef-eof-token)</strong>

Return <var>rules</var>.

<strong><a id="ref-for-typedef-cdo-token②"></a>[\<CDO-token\>](#typedef-cdo-token)</strong>

<strong><a id="ref-for-typedef-cdc-token②"></a>[\<CDC-token\>](#typedef-cdc-token)</strong>

<a id="ref-for-token-stream-discard-a-token③"></a>[Discard a token](#token-stream-discard-a-token) from <var>input</var>.

<strong>Note:</strong>

What’s this for?

Back when CSS was first being introduced, the <code><a id="ref-for-the-style-element"></a>[style](https://html.spec.whatwg.org/multipage/semantics.html#the-style-element)</code> element was treated as an unknown element by older browsers. To avoid having its contents displayed in the page for these legacy browsers, it became common practice to wrap the stylesheet in an HTML comment, and newer browsers would simply ignore the HTML comment syntax. This requirement carries over to today, decades later.

The same practice was done for <code><a id="ref-for-script"></a>[script](https://html.spec.whatwg.org/multipage/scripting.html#script)</code> elements, where HTML comment syntax is treated as a line comment in JS (similar to <code>//</code>) for the same reason.

<strong><a id="ref-for-typedef-at-keyword-token⑥"></a>[\<at-keyword-token\>](#typedef-at-keyword-token)</strong>

<a id="ref-for-consume-an-at-rule①"></a>[Consume an at-rule](#consume-an-at-rule) from <var>input</var>. If anything is returned, append it to <var>rules</var>.

<strong>anything else</strong>

<a id="ref-for-consume-a-qualified-rule①"></a>[Consume a qualified rule](#consume-a-qualified-rule) from <var>input</var>. If a <a id="ref-for-css-rule③"></a>[rule](#css-rule) is returned, append it to <var>rules</var>.

#### <a id="consume-at-rule"></a>5.5.2.  Consume an at-rule[](#consume-at-rule)

To <a id="consume-an-at-rule"></a><strong>consume an at-rule</strong> from a <a id="ref-for-css-token-stream⑧"></a>[token stream](#css-token-stream) <var>input</var>, given an optional bool <var>nested</var> (default false):

Assert: The <a id="ref-for-token-stream-next-token⑦"></a>[next token](#token-stream-next-token) is an <a id="ref-for-typedef-at-keyword-token⑦"></a>[\<at-keyword-token\>](#typedef-at-keyword-token).

<a id="ref-for-token-stream-consume-a-token"></a>[Consume a token](#token-stream-consume-a-token) from <var>input</var>, and let <var>rule</var> be a new <a id="ref-for-at-rule①⑤"></a>[at-rule](#at-rule) with its name set to the returned token’s value, its prelude initially set to an empty <a id="ref-for-list④"></a>[list](https://infra.spec.whatwg.org/#list), and no declarations or child rules.

<a id="ref-for-token-stream-process②"></a>[Process](#token-stream-process) <var>input</var>:

<strong><a id="ref-for-typedef-semicolon-token②"></a>[\<semicolon-token\>](#typedef-semicolon-token)</strong>

<strong><a id="ref-for-typedef-eof-token⑧"></a>[\<EOF-token\>](#typedef-eof-token)</strong>

<a id="ref-for-token-stream-discard-a-token④"></a>[Discard a token](#token-stream-discard-a-token) from <var>input</var>. If <var>rule</var> is valid in the current context, return it; otherwise return nothing.

<strong><a id="ref-for-tokendef-close-curly③"></a>[\<}-token\>](#tokendef-close-curly)</strong>

If <var>nested</var> is true:

- If <var>rule</var> is valid in the current context, return it.

- Otherwise, return nothing.

Otherwise, <a id="ref-for-token-stream-consume-a-token①"></a>[consume a token](#token-stream-consume-a-token) and append the result to <var>rule</var>’s prelude.

<strong><a id="ref-for-tokendef-open-curly⑥"></a>[\<{-token\>](#tokendef-open-curly)</strong>

<a id="ref-for-consume-a-block"></a>[Consume a block](#consume-a-block) from <var>input</var>, and assign the result to <var>rule</var>’s child <a id="ref-for-css-rule④"></a>[rules](#css-rule).

Note: If the result contains <a id="ref-for-list⑤"></a>[lists](https://infra.spec.whatwg.org/#list) of <a id="ref-for-declaration①"></a>[declarations](#declaration), how they’re materialized in the CSSOM depends on the rule. Some turn them all into <a id="ref-for-nested-declarations-rule"></a>[nested declarations rules](https://www.w3.org/TR/css-nesting-1/#nested-declarations-rule), others will treat them all as declarations, and others will treat the first item differently from the rest.

If <var>rule</var> is valid in the current context, return it. Otherwise, return nothing.

<strong>anything else</strong>

<a id="ref-for-consume-a-component-value①"></a>[Consume a component value](#consume-a-component-value) from <var>input</var> and append the returned value to <var>rule</var>’s prelude.

#### <a id="consume-qualified-rule"></a>5.5.3.  Consume a qualified rule[](#consume-qualified-rule)

To <a id="consume-a-qualified-rule"></a><strong>consume a qualified rule</strong>, from a <a id="ref-for-css-token-stream⑨"></a>[token stream](#css-token-stream) <var>input</var>, given an optional <a id="ref-for-token-stream-tokens⑦"></a>[token](#token-stream-tokens) <var>stop token</var> and an optional bool <var>nested</var> (default false):

Let <var>rule</var> be a new <a id="ref-for-qualified-rule⑥"></a>[qualified rule](#qualified-rule) with its prelude, declarations, and child rules all initially set to empty <a id="ref-for-list⑥"></a>[lists](https://infra.spec.whatwg.org/#list).

<a id="ref-for-token-stream-process③"></a>[Process](#token-stream-process) <var>input</var>:

<strong><a id="ref-for-typedef-eof-token⑨"></a>[\<EOF-token\>](#typedef-eof-token)</strong>

<strong><var>stop token</var> (if passed)</strong>

This is a <a id="ref-for-parse-error⑨"></a>[parse error](#parse-error). Return nothing.

<strong><a id="ref-for-tokendef-close-curly④"></a>[\<}-token\>](#tokendef-close-curly)</strong>

This is a <a id="ref-for-parse-error①⓪"></a>[parse error](#parse-error). If <var>nested</var> is true, return nothing. Otherwise, <a id="ref-for-token-stream-consume-a-token②"></a>[consume a token](#token-stream-consume-a-token) and append the result to <var>rule</var>’s prelude.

<strong><a id="ref-for-tokendef-open-curly⑦"></a>[\<{-token\>](#tokendef-open-curly)</strong>

If the first two non-<a id="ref-for-typedef-whitespace-token⑤"></a>[\<whitespace-token\>](#typedef-whitespace-token) values of <var>rule</var>’s prelude are an <a id="ref-for-typedef-ident-token⑥"></a>[\<ident-token\>](#typedef-ident-token) whose value starts with "--" followed by a <a id="ref-for-typedef-colon-token①"></a>[\<colon-token\>](#typedef-colon-token), then:

- If <var>nested</var> is true, <a id="ref-for-consume-the-remnants-of-a-bad-declaration"></a>[consume the remnants of a bad declaration](#consume-the-remnants-of-a-bad-declaration) from <var>input</var>, with <var>nested</var> set to true, and return nothing.

- If <var>nested</var> is false, <a id="ref-for-consume-a-block①"></a>[consume a block](#consume-a-block) from <var>input</var>, and return nothing.

<strong>Note:</strong>

What’s this check for?

<a id="ref-for-declaration②"></a>[Declarations](#declaration) and <a id="ref-for-qualified-rule⑦"></a>[qualified rules](#qualified-rule) don’t <em>generally</em> overlap in their allowed syntax. No currently-defined CSS property allows {}-blocks in its value, so <code>foo&#58;bar {};</code> is definitely a rule, and <code>foo&#58; bar;</code> is definitely a property. Even if a future CSS property allows {}-blocks in its value, the allowed syntax is restricted to the {}-block being the whole value, such as <code>foo&#58; {...};</code>, which is guaranteed to not be a valid rule, since the : doesn’t have an ident or function following it to mark it as a pseudo-class.

This allows us to mix declarations and rules in the same context: we first try to parse something as a declaration, and if that doesn’t result in a valid declaration, we reparse it as a rule instead. An accidentally-invalid declaration will parse as a rule instead, but that’s fine: the parser will stop at the declaration’s ending semicolon and consider it an invalid rule. (Or in the case of a property containing a {}-block, will stop just \*before\* the semicolon, still considering it an invalid rule, and then the next attempt to parse something will throw out the lone semicolon as invalid.) So the total amount of tokens consumed is the same regardless.

<a id="ref-for-custom-property"></a>[Custom properties](https://www.w3.org/TR/css-variables-1/#custom-property), however, don’t have the CSSWG carefully vetting their syntax. Authors <em>can</em> write a custom property that takes a {}-block in its value, even combined with other things; if that custom property is then invalid (due to an invalidly-written <a id="ref-for-funcdef-var"></a>[var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function, for example), when it’s reparsed as a rule it will stop early, at the {}-block. The remaining tokens of the custom property’s value will then get parsed as a fresh construct, potentially causing unexpected declarations or rules to be created.

To avoid this (admittedly very niche) corner-case, we subtract the syntax of a <a id="ref-for-custom-property①"></a>[custom property](https://www.w3.org/TR/css-variables-1/#custom-property) from that of a <a id="ref-for-qualified-rule⑧"></a>[qualified rule](#qualified-rule); if you’re in a context that allows properties and rules to be mixed, and you somehow end up parsing a rule that looks like a <a id="ref-for-custom-property②"></a>custom property, you’ve messed up, and need to instead consume an entire custom property (all the way to the ending semicolon).

(If we’re in a context that doesn’t allow properties, we just throw away the rule if it looks like a custom property. This ensures that <code>--foo&#58;hover { color&#58; blue; }</code> is consistently invalid everywhere, without potentially consuming a ton of a stylesheet looking for the non-existent ending semicolon.)

Otherwise, <a id="ref-for-consume-a-block②"></a>[consume a block](#consume-a-block) from <var>input</var>, and let <var>child rules</var> be the result. If the first item of <var>child rules</var> is a <a id="ref-for-list⑦"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-declaration③"></a>[declarations](#declaration), remove it from <var>child rules</var> and assign it to <var>rule</var>’s declarations. If any remaining items of <var>child rules</var> are <a id="ref-for-list⑧"></a>lists of <a id="ref-for-declaration④"></a>declarations, replace them with <a id="ref-for-nested-declarations-rule①"></a>[nested declarations rules](https://www.w3.org/TR/css-nesting-1/#nested-declarations-rule) containing the <a id="ref-for-list⑨"></a>list as its sole child. Assign <var>child rules</var> to <var>rule</var>’s child <a id="ref-for-css-rule⑤"></a>[rules](#css-rule).

If <var>rule</var> is valid in the current context, return it; otherwise return an <a id="invalid-rule-error"></a><strong>invalid rule error</strong>.

Tests

- [invalid-nested-rules.html](https://wpt.fyi/results/css/css-syntax/invalid-nested-rules.html) [(live test)](http://wpt.live/css/css-syntax/invalid-nested-rules.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/invalid-nested-rules.html)

<strong>anything else</strong>

<a id="ref-for-consume-a-component-value②"></a>[Consume a component value](#consume-a-component-value) from <var>input</var> and append the result to <var>rule</var>’s prelude.

Tests

- [custom-property-rule-ambiguity.html](https://wpt.fyi/results/css/css-syntax/custom-property-rule-ambiguity.html) [(live test)](http://wpt.live/css/css-syntax/custom-property-rule-ambiguity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/custom-property-rule-ambiguity.html)
- [invalid-nested-rules.html](https://wpt.fyi/results/css/css-syntax/invalid-nested-rules.html) [(live test)](http://wpt.live/css/css-syntax/invalid-nested-rules.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/invalid-nested-rules.html)

#### <a id="consume-block"></a>5.5.4.  Consume a block[](#consume-block)

To <a id="consume-a-block"></a><strong>consume a block</strong>, from a <a id="ref-for-css-token-stream①⓪"></a>[token stream](#css-token-stream) <var>input</var>:

Assert: The <a id="ref-for-token-stream-next-token⑧"></a>[next token](#token-stream-next-token) is a <a id="ref-for-tokendef-open-curly⑧"></a>[\<{-token\>](#tokendef-open-curly).

<a id="ref-for-token-stream-discard-a-token⑤"></a>[Discard a token](#token-stream-discard-a-token) from <var>input</var>. <a id="ref-for-consume-a-blocks-contents①"></a>[Consume a block’s contents](#consume-a-blocks-contents) from <var>input</var> and let <var>rules</var> be the result. <a id="ref-for-token-stream-discard-a-token⑥"></a>Discard a token from <var>input</var>.

Return <var>rules</var>.

#### <a id="consume-block-contents"></a>5.5.5. <a id="consume-list-of-declarations"></a> Consume a block’s contents[](#consume-block-contents)

To <a id="consume-a-blocks-contents"></a><strong>consume a block’s contents</strong> from a <a id="ref-for-css-token-stream①①"></a>[token stream](#css-token-stream) <var>input</var>:

Note: This algorithm returns a <a id="ref-for-list①⓪"></a>[list](https://infra.spec.whatwg.org/#list), consisting of <a id="ref-for-css-rule⑥"></a>[rules](#css-rule) and <a id="ref-for-list①①"></a>lists of <a id="ref-for-declaration⑤"></a>[declarations](#declaration). Depending on the parent context, a <a id="ref-for-list①②"></a>list of <a id="ref-for-declaration⑥"></a>declarations might be materialized in the CSSOM as either a <code><a id="ref-for-cssstyledeclaration"></a>[CSSStyleDeclaration](https://www.w3.org/TR/cssom-1/#cssstyledeclaration)</code>, or as a <code><a id="ref-for-cssnesteddeclarations"></a>[CSSNestedDeclarations](https://www.w3.org/TR/css-nesting-1/#cssnesteddeclarations)</code> rule.

Let <var>rules</var> be an empty <a id="ref-for-list①③"></a>[list](https://infra.spec.whatwg.org/#list), containing either <a id="ref-for-css-rule⑦"></a>[rules](#css-rule) or <a id="ref-for-list①④"></a>lists of <a id="ref-for-declaration⑦"></a>[declarations](#declaration).

Let <var>decls</var> be an empty <a id="ref-for-list①⑤"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-declaration⑧"></a>[declarations](#declaration).

<a id="ref-for-token-stream-process④"></a>[Process](#token-stream-process) <var>input</var>:

<strong><a id="ref-for-typedef-whitespace-token⑥"></a>[\<whitespace-token\>](#typedef-whitespace-token)</strong>

<strong><a id="ref-for-typedef-semicolon-token③"></a>[\<semicolon-token\>](#typedef-semicolon-token)</strong>

<a id="ref-for-token-stream-discard-a-token⑦"></a>[Discard a token](#token-stream-discard-a-token) from <var>input</var>.

<strong><a id="ref-for-typedef-eof-token①⓪"></a>[\<EOF-token\>](#typedef-eof-token)</strong>

<strong><a id="ref-for-tokendef-close-curly⑤"></a>[\<}-token\>](#tokendef-close-curly)</strong>

Return <var>rules</var>.

<strong><a id="ref-for-typedef-at-keyword-token⑧"></a>[\<at-keyword-token\>](#typedef-at-keyword-token)</strong>

If <var>decls</var> is not empty, append it to <var>rules</var>, and set <var>decls</var> to a fresh empty <a id="ref-for-list①⑥"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-declaration⑨"></a>[declarations](#declaration).

<a id="ref-for-consume-an-at-rule②"></a>[Consume an at-rule](#consume-an-at-rule) from <var>input</var>, with <var>nested</var> set to true. If a <a id="ref-for-css-rule⑧"></a>[rule](#css-rule) was returned, append it to <var>rules</var>.

<strong>anything else</strong>

<a id="ref-for-token-stream-mark①"></a>[Mark](#token-stream-mark) <var>input</var>.

<a id="ref-for-consume-a-declaration②"></a>[Consume a declaration](#consume-a-declaration) from <var>input</var>, with <var>nested</var> set to true. If a <a id="ref-for-declaration①⓪"></a>[declaration](#declaration) was returned, append it to <var>decls</var>, and <a id="ref-for-token-stream-discard-a-mark"></a>[discard a mark](#token-stream-discard-a-mark) from <var>input</var>.

Otherwise, <a id="ref-for-token-stream-restore-a-mark"></a>[restore a mark](#token-stream-restore-a-mark) from <var>input</var>, then <a id="ref-for-consume-a-qualified-rule②"></a>[consume a qualified rule](#consume-a-qualified-rule) from <var>input</var>, with <var>nested</var> set to true, and <a id="ref-for-typedef-semicolon-token④"></a>[\<semicolon-token\>](#typedef-semicolon-token) as the <var>stop token</var>.

<strong>If nothing was returned</strong>

Do nothing

<strong>If an <a id="ref-for-invalid-rule-error①"></a>[invalid rule error](#invalid-rule-error) was returned</strong>

If <var>decls</var> is not empty, append <var>decls</var> to <var>rules</var>, and set <var>decls</var> to a fresh empty <a id="ref-for-list①⑦"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-declaration①①"></a>[declarations](#declaration). (Otherwise, do nothing.)

<strong>If a <a id="ref-for-css-rule⑨"></a>[rule](#css-rule) was returned</strong>

If <var>decls</var> is not empty, append <var>decls</var> to <var>rules</var>, and set <var>decls</var> to a fresh empty <a id="ref-for-list①⑧"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-declaration①②"></a>[declarations](#declaration). Append the <a id="ref-for-css-rule①⓪"></a>[rule](#css-rule) to <var>rules</var>.

<strong>Note:</strong>

Implementation note

This spec, as with many CSS specs, has been written to prioritize understandability over efficiency. A number of algorithms, notably the above "parse as a declaration, then parse as a rule" behavior can be fairly inefficient if implemented naively as described.

However, the behavior has been carefully written to allow "early exits" as much as possible. In particular, and roughly in order of when the exit can occur:

- If the first non-whitespace token isn’t an <a id="ref-for-typedef-ident-token⑦"></a>[\<ident-token\>](#typedef-ident-token) for a recognized property name (or a custom property name), you can immediately stop parsing as a declaration and reparse as a rule instead. If the <em>next</em> non-whitespace token isn’t a <a id="ref-for-typedef-colon-token②"></a>[\<colon-token\>](#typedef-colon-token), you can similarly immediately stop parsing as a declaration.

  (That is, font+ ... is guaranteed to not be a property, nor is not-a-prop-name: ....)

- If the first two non-whitespace tokens are a custom property name and a colon, it’s definitely a custom property and won’t ever produce a valid rule, so even if the custom property ends up invalid there’s no need to try and reparse as a rule.

  (That is, --foo:hover {...} is guaranteed to be a custom property, not a rule.)

- If the first three non-whitespace tokens are a valid property name, a colon, and anything other than a <a id="ref-for-tokendef-open-curly⑨"></a>[\<{-token\>](#tokendef-open-curly), and then while parsing the declaration’s value you encounter a <a id="ref-for-tokendef-open-curly①"></a>\<{-token\>, you can immediately stop parsing as a declaration and reparse as a rule instead.

  (That is, font:bar {... is guaranteed to be an invalid property.)

- If you see a recognized property name, a colon, and a {}-block, but the first non-whitespace tokens following that block isn’t either immediately the final semicolon, or the !important followed by the semicolon, you can immediately stop parsing as a declaration and reparse as a rule instead.

  (That is, <a id="ref-for-propdef-font"></a>[font: {} bar ...](https://www.w3.org/TR/css-fonts-4/#propdef-font) is guaranteed to be an invalid property; you don’t need to keep parsing until you hit a semicolon.)

Similarly, even tho the parsing requirements are specified to rely on checking the grammar of the declarations as you parse, a <em>generic</em> processor trying to implement a non-CSS language on top of the generic CSS <em>syntax</em> can still get away with just verifying that declarations start with an ident, a colon, and then either contain solely a {}-block or no {}-block at all. They’ll just spent a little more time on parsing than an implementation with grammar knowledge in cases like foo:hover ... {}, since they can’t early-exit on the first token.

Tests

- [at-rule-in-declaration-list.html](https://wpt.fyi/results/css/css-syntax/at-rule-in-declaration-list.html) [(live test)](http://wpt.live/css/css-syntax/at-rule-in-declaration-list.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/at-rule-in-declaration-list.html)

#### <a id="consume-declaration"></a>5.5.6.  Consume a declaration[](#consume-declaration)

To <a id="consume-a-declaration"></a><strong>consume a declaration</strong> from a <a id="ref-for-css-token-stream①②"></a>[token stream](#css-token-stream) <var>input</var>, given an optional bool <var>nested</var> (default false):

Let <var>decl</var> be a new <a id="ref-for-declaration①③"></a>[declaration](#declaration), with an initially empty name and a value set to an empty <a id="ref-for-list①⑨"></a>[list](https://infra.spec.whatwg.org/#list).

1.  If the <a id="ref-for-token-stream-next-token⑨"></a>[next token](#token-stream-next-token) is an <a id="ref-for-typedef-ident-token⑧"></a>[\<ident-token\>](#typedef-ident-token), <a id="ref-for-token-stream-consume-a-token③"></a>[consume a token](#token-stream-consume-a-token) from <var>input</var> and set <var>decl</var>'s name to the token’s value.

    Otherwise, <a id="ref-for-consume-the-remnants-of-a-bad-declaration①"></a>[consume the remnants of a bad declaration](#consume-the-remnants-of-a-bad-declaration) from <var>input</var>, with <var>nested</var>, and return nothing.

2.  <a id="ref-for-token-stream-discard-whitespace⑤"></a>[Discard whitespace](#token-stream-discard-whitespace) from <var>input</var>.

3.  If the <a id="ref-for-token-stream-next-token①⓪"></a>[next token](#token-stream-next-token) is a <a id="ref-for-typedef-colon-token③"></a>[\<colon-token\>](#typedef-colon-token), <a id="ref-for-token-stream-discard-a-token⑧"></a>[discard a token](#token-stream-discard-a-token) from <var>input</var>.

    Otherwise, <a id="ref-for-consume-the-remnants-of-a-bad-declaration②"></a>[consume the remnants of a bad declaration](#consume-the-remnants-of-a-bad-declaration) from <var>input</var>, with <var>nested</var>, and return nothing.

4.  <a id="ref-for-token-stream-discard-whitespace⑥"></a>[Discard whitespace](#token-stream-discard-whitespace) from <var>input</var>.

5.  <a id="ref-for-consume-a-list-of-component-values②"></a>[Consume a list of component values](#consume-a-list-of-component-values) from <var>input</var>, with <var>nested</var>, and with <a id="ref-for-typedef-semicolon-token⑤"></a>[\<semicolon-token\>](#typedef-semicolon-token) as the stop token, and set <var>decl</var>’s value to the result.

6.  If the last two non-<a id="ref-for-typedef-whitespace-token⑦"></a>[\<whitespace-token\>](#typedef-whitespace-token)s in <var>decl</var>’s value are a <a id="ref-for-typedef-delim-token⑨"></a>[\<delim-token\>](#typedef-delim-token) with the value "!" followed by an <a id="ref-for-typedef-ident-token⑨"></a>[\<ident-token\>](#typedef-ident-token) with a value that is an <a id="ref-for-ascii-case-insensitive①"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "important", remove them from <var>decl</var>’s value and set <var>decl</var>’s <var>important</var> flag.

7.  While the last item in <var>decl</var>’s value is a <a id="ref-for-typedef-whitespace-token⑧"></a>[\<whitespace-token\>](#typedef-whitespace-token), <a id="ref-for-list-remove"></a>[remove](https://infra.spec.whatwg.org/#list-remove) that token.

8.  If <var>decl</var>’s name is a <a id="ref-for-custom-property-name-string"></a>[custom property name string](https://www.w3.org/TR/css-typed-om-1/#custom-property-name-string), then set <var>decl</var>’s <var>original text</var> to the segment of the original source text string corresponding to the tokens of <var>decl</var>’s value.

    Otherwise, if <var>decl</var>’s value contains a top-level <a id="ref-for-simple-block②"></a>[simple block](#simple-block) with an associated token of <a id="ref-for-tokendef-open-curly②"></a>[\<{-token\>](#tokendef-open-curly), and also contains <em>any other</em> non-<a id="ref-for-typedef-whitespace-token⑨"></a>[\<whitespace-token\>](#typedef-whitespace-token) value, return nothing. (That is, a top-level {}-block is only allowed as the entire value of a non-custom property.)

    Otherwise, if <var>decl</var>’s name is an <a id="ref-for-ascii-case-insensitive②"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "unicode-range", <a id="ref-for-consume-the-value-of-a-unicode-range-descriptor①"></a>[consume the value of a unicode-range descriptor](#consume-the-value-of-a-unicode-range-descriptor) from the segment of the original source text string corresponding to the tokens returned by the <a id="ref-for-consume-a-list-of-component-values③"></a>[consume a list of component values](#consume-a-list-of-component-values) call, and replace <var>decl</var>’s value with the result.

    Tests

    - [var-with-blocks.html](https://wpt.fyi/results/css/css-syntax/var-with-blocks.html) [(live test)](http://wpt.live/css/css-syntax/var-with-blocks.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/var-with-blocks.html)

9.  If <var>decl</var> is valid in the current context, return it; otherwise return nothing.

To <a id="consume-the-remnants-of-a-bad-declaration"></a><strong>consume the remnants of a bad declaration</strong> from a <a id="ref-for-css-token-stream①③"></a>[token stream](#css-token-stream) <var>input</var>, given a bool <var>nested</var>:

<a id="ref-for-token-stream-process⑤"></a>[Process](#token-stream-process) <var>input</var>:

<strong><a id="ref-for-typedef-eof-token①①"></a>[\<eof-token\>](#typedef-eof-token)</strong>

<strong><a id="ref-for-typedef-semicolon-token⑥"></a>[\<semicolon-token\>](#typedef-semicolon-token)</strong>

<a id="ref-for-token-stream-discard-a-token⑨"></a>[Discard a token](#token-stream-discard-a-token) from <var>input</var>, and return nothing.

<strong><a id="ref-for-tokendef-close-curly⑥"></a>[\<}-token\>](#tokendef-close-curly)</strong>

If <var>nested</var> is true, return nothing. Otherwise, <a id="ref-for-token-stream-discard-a-token①⓪"></a>[discard a token](#token-stream-discard-a-token).

<strong>anything else</strong>

<a id="ref-for-consume-a-component-value③"></a>[Consume a component value](#consume-a-component-value) from <var>input</var>, and do nothing.

#### <a id="consume-list-of-components"></a>5.5.7.  Consume a list of component values[](#consume-list-of-components)

To <a id="consume-a-list-of-component-values"></a><strong>consume a list of component values</strong> from a <a id="ref-for-css-token-stream①④"></a>[token stream](#css-token-stream) <var>input</var>, given an optional <a id="ref-for-token-stream-tokens⑧"></a>[token](#token-stream-tokens) <var>stop token</var> and an optional boolean <var>nested</var> (default false):

Let <var>values</var> be an empty <a id="ref-for-list②⓪"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-component-value⑥"></a>[component values](#component-value).

<a id="ref-for-token-stream-process⑥"></a>[Process](#token-stream-process) <var>input</var>:

<strong><a id="ref-for-typedef-eof-token①②"></a>[\<eof-token\>](#typedef-eof-token)</strong>

<strong><var>stop token</var> (if passed)</strong>

Return <var>values</var>.

<strong><a id="ref-for-tokendef-close-curly⑦"></a>[\<}-token\>](#tokendef-close-curly)</strong>

If <var>nested</var> is true, return <var>values</var>.

Otherwise, this is a <a id="ref-for-parse-error①①"></a>[parse error](#parse-error). <a id="ref-for-token-stream-consume-a-token④"></a>[Consume a token](#token-stream-consume-a-token) from <var>input</var> and append the result to <var>values</var>.

<strong>anything else</strong>

<a id="ref-for-consume-a-component-value④"></a>[Consume a component value](#consume-a-component-value) from <var>input</var>, and append the result to <var>values</var>.

Tests

- [declarations-trim-whitespace.html](https://wpt.fyi/results/css/css-syntax/declarations-trim-whitespace.html) [(live test)](http://wpt.live/css/css-syntax/declarations-trim-whitespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/declarations-trim-whitespace.html)

#### <a id="consume-component-value"></a>5.5.8.  Consume a component value[](#consume-component-value)

To <a id="consume-a-component-value"></a><strong>consume a component value</strong> from a <a id="ref-for-css-token-stream①⑤"></a>[token stream](#css-token-stream) <var>input</var>:

<a id="ref-for-token-stream-process⑦"></a>[Process](#token-stream-process) <var>input</var>:

<strong><a id="ref-for-tokendef-open-curly①⓪"></a>[\<{-token\>](#tokendef-open-curly)</strong>

<strong><a id="ref-for-tokendef-open-square③"></a>[\<\[-token\>](#tokendef-open-square)</strong>

<strong><a id="ref-for-tokendef-open-paren③"></a>[\<(-token\>](#tokendef-open-paren)</strong>

<a id="ref-for-consume-a-simple-block"></a>[Consume a simple block](#consume-a-simple-block) from <var>input</var> and return the result.

<strong><a id="ref-for-typedef-function-token⑧"></a>[\<function-token\>](#typedef-function-token)</strong>

<a id="ref-for-consume-a-function"></a>[Consume a function](#consume-a-function) from <var>input</var> and return the result.

<strong>anything else</strong>

<a id="ref-for-token-stream-consume-a-token⑤"></a>[Consume a token](#token-stream-consume-a-token) from <var>input</var> and return the result.

#### <a id="consume-simple-block"></a>5.5.9.  Consume a simple block[](#consume-simple-block)

To <a id="consume-a-simple-block"></a><strong>consume a simple block</strong> from a <a id="ref-for-css-token-stream①⑥"></a>[token stream](#css-token-stream) <var>input</var>:

Assert: the <a id="ref-for-token-stream-next-token①①"></a>[next token](#token-stream-next-token) of <var>input</var> is <a id="ref-for-tokendef-open-curly①①"></a>[\<{-token\>](#tokendef-open-curly), <a id="ref-for-tokendef-open-square④"></a>[\<\[-token\>](#tokendef-open-square), or <a id="ref-for-tokendef-open-paren④"></a>[\<(-token\>](#tokendef-open-paren).

Let <var>ending token</var> be the mirror variant of the <a id="ref-for-token-stream-next-token①②"></a>[next token](#token-stream-next-token). (E.g. if it was called with <a id="ref-for-tokendef-open-square⑤"></a>[\<\[-token\>](#tokendef-open-square), the <var>ending token</var> is <a id="ref-for-tokendef-close-square②"></a>[\<\]-token\>](#tokendef-close-square).)

Let <var>block</var> be a new <a id="ref-for-simple-block③"></a>[simple block](#simple-block) with its associated token set to the <a id="ref-for-token-stream-next-token①③"></a>[next token](#token-stream-next-token) and with its value initially set to an empty <a id="ref-for-list②①"></a>[list](https://infra.spec.whatwg.org/#list).

<a id="ref-for-token-stream-discard-a-token①①"></a>[Discard a token](#token-stream-discard-a-token) from <var>input</var>.

<a id="ref-for-token-stream-process⑧"></a>[Process](#token-stream-process) <var>input</var>:

<strong><a id="ref-for-typedef-eof-token①③"></a>[\<eof-token\>](#typedef-eof-token)</strong>

<strong><var>ending token</var></strong>

<a id="ref-for-token-stream-discard-a-token①②"></a>[Discard a token](#token-stream-discard-a-token) from <var>input</var>. Return <var>block</var>.

<strong>anything else</strong>

<a id="ref-for-consume-a-component-value⑤"></a>[Consume a component value](#consume-a-component-value) from <var>input</var> and append the result to <var>block</var>’s value.

#### <a id="consume-function"></a>5.5.10.  Consume a function[](#consume-function)

To <a id="consume-a-function"></a><strong>consume a function</strong> from a <a id="ref-for-css-token-stream①⑦"></a>[token stream](#css-token-stream) <var>input</var>:

Assert: The <a id="ref-for-token-stream-next-token①④"></a>[next token](#token-stream-next-token) is a <a id="ref-for-typedef-function-token⑨"></a>[\<function-token\>](#typedef-function-token).

<a id="ref-for-token-stream-consume-a-token⑥"></a>[Consume a token](#token-stream-consume-a-token) from <var>input</var>, and let <var>function</var> be a new function with its name equal the returned token’s value, and a value set to an empty <a id="ref-for-list②②"></a>[list](https://infra.spec.whatwg.org/#list).

<a id="ref-for-token-stream-process⑨"></a>[Process](#token-stream-process) <var>input</var>:

<strong><a id="ref-for-typedef-eof-token①④"></a>[\<eof-token\>](#typedef-eof-token)</strong>

<strong><a id="ref-for-tokendef-close-paren②"></a>[\<)-token\>](#tokendef-close-paren)</strong>

<a id="ref-for-token-stream-discard-a-token①③"></a>[Discard a token](#token-stream-discard-a-token) from <var>input</var>. Return <var>function</var>.

<strong>anything else</strong>

<a id="ref-for-consume-a-component-value⑥"></a>[Consume a component value](#consume-a-component-value) from <var>input</var> and append the result to <var>function</var>’s value.

#### <a id="consume-unicode-range-value"></a>5.5.11.  Consume a <a id="ref-for-descdef-font-face-unicode-range①"></a>[unicode-range](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-unicode-range) value[](#consume-unicode-range-value)

To <a id="consume-the-value-of-a-unicode-range-descriptor"></a><strong>consume the value of a unicode-range descriptor</strong> from a string <var>input string</var>:

1.  Let <var>tokens</var> be the result of <a id="ref-for-css-tokenize①"></a>[tokenizing](#css-tokenize) <var>input string</var> with <var>unicode ranges allowed</var> set to true.

2.  <a id="ref-for-consume-a-list-of-component-values④"></a>[Consume a list of component values](#consume-a-list-of-component-values) from <var>tokens</var>, and return the result.

Note: The existence of this algorithm is due to a design mistake in early CSS. It should never be reproduced.

## <a id="anb-microsyntax"></a>6.  The <var>An+B</var> microsyntax[](#anb-microsyntax)

Several things in CSS, such as the <a id="ref-for-nth-child-pseudo"></a>[:nth-child()](https://www.w3.org/TR/selectors-4/#nth-child-pseudo) pseudoclass, need to indicate indexes in a list. The <var>An+B</var> microsyntax is useful for this, allowing an author to easily indicate single elements or all elements at regularly-spaced intervals in a list.

The <a id="anb"></a><strong>An+B</strong> notation defines an integer step (<var>A</var>) and offset (<var>B</var>), and represents the <var>An+B</var>th elements in a list, for every positive integer or zero value of <var>n</var>, with the first element in the list having index 1 (not 0).

For values of <var>A</var> and <var>B</var> greater than 0, this effectively divides the list into groups of <var>A</var> elements (the last group taking the remainder), and selecting the <var>B</var>th element of each group.

The <var>An+B</var> notation also accepts the even and odd keywords, which have the same meaning as 2n and 2n+1, respectively.

<a id="example-0b8828eb"></a>

<strong>Example:</strong>

[](#example-0b8828eb)

Examples:

``` text
2n+0   /* represents all of the even elements in the list */
even   /* same */
4n+1   /* represents the 1st, 5th, 9th, 13th, etc. elements in the list */
```

The values of <var>A</var> and <var>B</var> can be negative, but only the positive results of <var>An+B</var>, for <var>n</var> ≥ 0, are used.

<a id="example-34cd21ee"></a> [](#example-34cd21ee)

Example:

``` text
-1n+6   /* represents the first 6 elements of the list */
-4n+10  /* represents the 2nd, 6th, and 10th elements of the list */
```

If both <var>A</var> and <var>B</var> are 0, the pseudo-class represents no element in the list.

### <a id="anb-syntax"></a>6.1.  Informal Syntax Description[](#anb-syntax)

<em>This section is non-normative.</em>

When <var>A</var> is 0, the <var>An</var> part may be omitted (unless the <var>B</var> part is already omitted). When <var>An</var> is not included and <var>B</var> is non-negative, the <a id="ref-for-selectordef-adjacent"></a>[+](https://www.w3.org/TR/selectors-4/#selectordef-adjacent) sign before <var>B</var> (when allowed) may also be omitted. In this case the syntax simplifies to just <var>B</var>.

<a id="example-335ba750"></a>

<strong>Example:</strong>

[](#example-335ba750)

Examples:

``` text
0n+5   /* represents the 5th element in the list */
5      /* same */
```

When <var>A</var> is 1 or -1, the <code>1</code> may be omitted from the rule.

<a id="example-5d33d659"></a>

<strong>Example:</strong>

[](#example-5d33d659)

Examples:

The following notations are therefore equivalent:

``` text
1n+0   /* represents all elements in the list */
n+0    /* same */
n      /* same */
```

If <var>B</var> is 0, then every <var>A</var>th element is picked. In such a case, the <var>+B</var> (or <var>-B</var>) part may be omitted unless the <var>A</var> part is already omitted.

<a id="example-c62dcd18"></a>

<strong>Example:</strong>

[](#example-c62dcd18)

Examples:

``` text
2n+0   /* represents every even element in the list */
2n     /* same */
```

When B is negative, its minus sign replaces the <a id="ref-for-selectordef-adjacent①"></a>[+](https://www.w3.org/TR/selectors-4/#selectordef-adjacent) sign.

<a id="example-3d25855e"></a>

<strong>Example:</strong>

[](#example-3d25855e)

Valid example:

``` text
3n-6
```

Invalid example:

``` text
3n + -6
```

Whitespace is permitted on either side of the <a id="ref-for-selectordef-adjacent②"></a>[+](https://www.w3.org/TR/selectors-4/#selectordef-adjacent) or - that separates the <var>An</var> and <var>B</var> parts when both are present.

<a id="example-e61d78e8"></a>

<strong>Example:</strong>

[](#example-e61d78e8)

Valid Examples with white space:

``` text
3n + 1
+3n - 2
-n+ 6
+6
```

Invalid Examples with white space:

``` text
3 n
+ 2n
+ 2
```

### <a id="the-anb-type"></a>6.2.  The <code>&#60;a-n-plus-b&#62;</code> type[](#the-anb-type)

The <var>An+B</var> notation was originally defined using a slightly different tokenizer than the rest of CSS, resulting in a somewhat odd definition when expressed in terms of CSS tokens. This section describes how to recognize the <var>An+B</var> notation in terms of CSS tokens (thus defining the <var>\<a-n-plus-b\></var> type for CSS grammar purposes), and how to interpret the CSS tokens to obtain values for <var>A</var> and <var>B</var>.

Tests

- [anb-parsing.html](https://wpt.fyi/results/css/css-syntax/anb-parsing.html) [(live test)](http://wpt.live/css/css-syntax/anb-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/anb-parsing.html)
- [anb-serialization.html](https://wpt.fyi/results/css/css-syntax/anb-serialization.html) [(live test)](http://wpt.live/css/css-syntax/anb-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/anb-serialization.html)

The <var>\<a-n-plus-b\></var> type is defined (using the [Value Definition Syntax in the Values & Units spec](https://www.w3.org/TR/css3-values/#value-defs)) as:

<a id="anb-production"></a><a id="ref-for-comb-one"></a><a id="ref-for-comb-one①"></a><a id="ref-for-comb-one②"></a><a id="ref-for-comb-one③"></a><a id="ref-for-mult-opt"></a><a id="ref-for-comb-one④"></a><a id="ref-for-comb-one⑤"></a><a id="ref-for-comb-one⑥"></a><a id="ref-for-mult-opt①"></a><a id="ref-for-comb-one⑦"></a><a id="ref-for-comb-one⑧"></a><a id="ref-for-comb-one⑨"></a><a id="ref-for-mult-opt②"></a><a id="ref-for-comb-one①⓪"></a><a id="ref-for-comb-one①①"></a><a id="ref-for-comb-one①②"></a><a id="ref-for-mult-opt③"></a><a id="ref-for-comb-one①③"></a><a id="ref-for-comb-one①④"></a><a id="ref-for-comb-one①⑤"></a><a id="ref-for-comb-one①⑥"></a><a id="ref-for-mult-opt④"></a><a id="ref-for-comb-one①⑦"></a><a id="ref-for-comb-one①⑧"></a><a id="ref-for-comb-one①⑨"></a>

``` text
<a-n-plus-b> =
  odd | even |
  <integer> |

  <n-dimension> |
  '+'?† n |
  -n |

  <ndashdigit-dimension> |
  '+'?† <ndashdigit-ident> |
  <dashndashdigit-ident> |

  <n-dimension> <signed-integer> |
  '+'?† n <signed-integer> |
  -n <signed-integer> |

  <ndash-dimension> <signless-integer> |
  '+'?† n- <signless-integer> |
  -n- <signless-integer> |

  <n-dimension> ['+' | '-'] <signless-integer> |
  '+'?† n ['+' | '-'] <signless-integer> |
  -n ['+' | '-'] <signless-integer>
```

where:

- <a id="typedef-n-dimension"></a><strong><code>&#60;n-dimension&#62;</code></strong> is a <a id="ref-for-typedef-dimension-token⑨"></a>[\<dimension-token\>](#typedef-dimension-token) with its type flag set to "integer", and a unit that is an <a id="ref-for-ascii-case-insensitive③"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "n"
- <a id="typedef-ndash-dimension"></a><strong><code>&#60;ndash-dimension&#62;</code></strong> is a <a id="ref-for-typedef-dimension-token①⓪"></a>[\<dimension-token\>](#typedef-dimension-token) with its type flag set to "integer", and a unit that is an <a id="ref-for-ascii-case-insensitive④"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "n-"
- <a id="typedef-ndashdigit-dimension"></a><strong><code>&#60;ndashdigit-dimension&#62;</code></strong> is a <a id="ref-for-typedef-dimension-token①①"></a>[\<dimension-token\>](#typedef-dimension-token) with its type flag set to "integer", and a unit that is an <a id="ref-for-ascii-case-insensitive⑤"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "n-\*", where "\*" is a series of one or more <a id="ref-for-digit①②"></a>[digits](#digit)
- <a id="typedef-ndashdigit-ident"></a><strong><code>&#60;ndashdigit-ident&#62;</code></strong> is an <a id="ref-for-typedef-ident-token①⓪"></a>[\<ident-token\>](#typedef-ident-token) whose value is an <a id="ref-for-ascii-case-insensitive⑥"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "n-\*", where "\*" is a series of one or more <a id="ref-for-digit①③"></a>[digits](#digit)
- <a id="typedef-dashndashdigit-ident"></a><strong><code>&#60;dashndashdigit-ident&#62;</code></strong> is an <a id="ref-for-typedef-ident-token①①"></a>[\<ident-token\>](#typedef-ident-token) whose value is an <a id="ref-for-ascii-case-insensitive⑦"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "-n-\*", where "\*" is a series of one or more <a id="ref-for-digit①④"></a>[digits](#digit)
- <a id="typedef-integer"></a><strong><code>&#60;integer&#62;</code></strong> is a <a id="ref-for-typedef-number-token⑤"></a>[\<number-token\>](#typedef-number-token) with its type flag set to "integer"
- <a id="typedef-signed-integer"></a><strong><code>&#60;signed-integer&#62;</code></strong> is a <a id="ref-for-typedef-number-token⑥"></a>[\<number-token\>](#typedef-number-token) with its type flag set to "integer", and a sign character
- <a id="typedef-signless-integer"></a><strong><code>&#60;signless-integer&#62;</code></strong> is a <a id="ref-for-typedef-number-token⑦"></a>[\<number-token\>](#typedef-number-token) with its type flag set to "integer", and no sign character

<a id="anb-plus"></a> <sup>†</sup>: When a plus sign (+) precedes an ident starting with "n", as in the cases marked above, there must be no whitespace between the two tokens, or else the tokens do not match the above grammar. Whitespace is valid (and ignored) between any other two tokens.

The clauses of the production are interpreted as follows:

<strong>odd</strong>

<var>A</var> is 2, <var>B</var> is 1.

<strong>even</strong>

<var>A</var> is 2, <var>B</var> is 0.

<strong><code><var>\<integer\></var></code></strong>

<var>A</var> is 0, <var>B</var> is the integer’s value.

<strong><code><var>\<n-dimension\></var></code></strong>

<strong><code>'+'? n</code></strong>

<strong><code>-n</code></strong>

<var>A</var> is the dimension’s value, 1, or -1, respectively. <var>B</var> is 0.

<strong><code><var>\<ndashdigit-dimension\></var></code></strong>

<strong><code>'+'? <var>\<ndashdigit-ident\></var></code></strong>

<var>A</var> is the dimension’s value or 1, respectively. <var>B</var> is the dimension’s unit or ident’s value, respectively, with the first <a id="ref-for-code-point⑧③"></a>[code point](https://infra.spec.whatwg.org/#code-point) removed and the remainder interpreted as a base-10 number. B is negative.

<strong>Note:</strong>

<strong><code><var>\<dashndashdigit-ident\></var></code></strong>

<var>A</var> is -1. <var>B</var> is the ident’s value, with the first two <a id="ref-for-code-point⑧④"></a>[code points](https://infra.spec.whatwg.org/#code-point) removed and the remainder interpreted as a base-10 number. B is negative.

<strong>Note:</strong>

<strong><code><var>\<n-dimension\></var> <var>\<signed-integer\></var></code></strong>

<strong><code>'+'? n <var>\<signed-integer\></var></code></strong>

<strong><code>-n <var>\<signed-integer\></var></code></strong>

<var>A</var> is the dimension’s value, 1, or -1, respectively. <var>B</var> is the integer’s value.

<strong><code><var>\<ndash-dimension\></var> <var>\<signless-integer\></var></code></strong>

<strong><code>'+'? n- <var>\<signless-integer\></var></code></strong>

<strong><code>-n- <var>\<signless-integer\></var></code></strong>

<var>A</var> is the dimension’s value, 1, or -1, respectively. <var>B</var> is the negation of the integer’s value.

<strong><code><var>\<n-dimension\></var> \['+' \| '-'\] <var>\<signless-integer\></var></code></strong>

<strong><code>'+'? n \['+' \| '-'\] <var>\<signless-integer\></var></code></strong>

<strong><code>-n \['+' \| '-'\] <var>\<signless-integer\></var></code></strong>

<var>A</var> is the dimension’s value, 1, or -1, respectively. <var>B</var> is the integer’s value. If a <code>'-'</code> was provided between the two, <var>B</var> is instead the negation of the integer’s value.

## <a id="rule-defs"></a>7.  Defining Grammars for Rules and Other Values[](#rule-defs)

[CSS Values 4 § 2 Value Definition Syntax](https://www.w3.org/TR/css-values-4/#value-defs) defines how to specify a grammar for properties and other CSS syntactic constructions.

Tests

- [unclosed-constructs.html](https://wpt.fyi/results/css/css-syntax/unclosed-constructs.html) [(live test)](http://wpt.live/css/css-syntax/unclosed-constructs.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/unclosed-constructs.html)

### <a id="block-contents"></a>7.1.  Defining Block Contents: the <a id="ref-for-typedef-block-contents"></a>[\<block-contents\>](#typedef-block-contents), <a id="ref-for-typedef-declaration-list"></a>[\<declaration-list\>](#typedef-declaration-list), <a id="ref-for-typedef-qualified-rule-list"></a>[\<qualified-rule-list\>](#typedef-qualified-rule-list), <a id="ref-for-typedef-declaration-rule-list"></a>[\<declaration-rule-list\>](#typedef-declaration-rule-list), and <a id="ref-for-typedef-rule-list"></a>[\<rule-list\>](#typedef-rule-list) productions[](#block-contents)

The CSS parser is agnostic as to the contents of blocks—​they’re all <a id="ref-for-consume-a-blocks-contents②"></a>[parsed with the same algorithm](#consume-a-blocks-contents), and differentiate themselves solely by what constructs are valid.

When writing a rule grammar, <a id="typedef-block-contents"></a><strong>\<block-contents\></strong> represents this agnostic parsing. It must only be used as the sole value in a block, and represents that no restrictions are implicitly placed on what the block can contain.

Accompanying prose must define what is valid and invalid in this context. If any <a id="ref-for-declaration①④"></a>[declarations](#declaration) are valid, and are <a id="ref-for-css-property-declarations"></a>[property declarations](#css-property-declarations), it must define whether they interact with the cascade; if they do, it must define their specificity and how they use <code>!important</code>.

In many cases, however, a block can’t validly contain <em>any</em> constructs of a given type. To represent these cases more explicitly, the following productions may be used

- <a id="typedef-declaration-list"></a><strong>\<declaration-list\></strong>: only <a id="ref-for-declaration①⑤"></a>[declarations](#declaration) are allowed; <a id="ref-for-at-rule①⑥"></a>[at-rules](#at-rule) and <a id="ref-for-qualified-rule⑨"></a>[qualified rules](#qualified-rule) are automatically invalid.

- <a id="typedef-qualified-rule-list"></a><strong>\<qualified-rule-list\></strong>: only <a id="ref-for-qualified-rule①⓪"></a>[qualified rules](#qualified-rule) are allowed; <a id="ref-for-declaration①⑥"></a>[declarations](#declaration) and <a id="ref-for-at-rule①⑦"></a>[at-rules](#at-rule) are automatically invalid.

- <a id="typedef-at-rule-list"></a><strong>\<at-rule-list\></strong>: only <a id="ref-for-at-rule①⑧"></a>[at-rules](#at-rule) are allowed; <a id="ref-for-declaration①⑦"></a>[declarations](#declaration) and <a id="ref-for-qualified-rule①①"></a>[qualified rules](#qualified-rule) are automatically invalid.

- <a id="typedef-declaration-rule-list"></a><strong>\<declaration-rule-list\></strong>: <a id="ref-for-declaration①⑧"></a>[declarations](#declaration) and <a id="ref-for-at-rule①⑨"></a>[at-rules](#at-rule) are allowed; <a id="ref-for-qualified-rule①②"></a>[qualified rules](#qualified-rule) are automatically invalid.

- <a id="typedef-rule-list"></a><strong>\<rule-list\></strong>: <a id="ref-for-qualified-rule①③"></a>[qualified rules](#qualified-rule) and <a id="ref-for-at-rule②⓪"></a>[at-rules](#at-rule) are allowed; <a id="ref-for-declaration①⑨"></a>[declarations](#declaration) are automatically invalid.

All of these are exactly equivalent to <a id="ref-for-typedef-block-contents①"></a>[\<block-contents\>](#typedef-block-contents) in terms of parsing, but the accompanying prose only has to define validity for the categories that aren’t automatically invalid.

<a id="example-6dc7b979"></a>

<strong>Example:</strong>

[](#example-6dc7b979) Some examples of the various productions:

- A top-level <a id="ref-for-at-ruledef-media②"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) uses <a id="ref-for-typedef-rule-list①"></a>[\<rule-list\>](#typedef-rule-list) for its block, while a nested one [\[CSS-NESTING-1\]](#biblio-css-nesting-1) uses <a id="ref-for-typedef-block-contents②"></a>[\<block-contents\>](#typedef-block-contents).

- <a id="ref-for-style-rule②"></a>[Style rules](#style-rule) use <a id="ref-for-typedef-block-contents③"></a>[\<block-contents\>](#typedef-block-contents).

- <a id="ref-for-at-font-face-rule"></a>[&#64;font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) uses <a id="ref-for-typedef-declaration-list①"></a>[\<declaration-list\>](#typedef-declaration-list).

- <a id="ref-for-at-ruledef-page①"></a>[&#64;page](https://www.w3.org/TR/css-page-3/#at-ruledef-page) uses <a id="ref-for-typedef-declaration-rule-list①"></a>[\<declaration-rule-list\>](#typedef-declaration-rule-list).

- <a id="ref-for-at-ruledef-keyframes"></a>[&#64;keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes) uses <a id="ref-for-typedef-qualified-rule-list①"></a>[\<qualified-rule-list\>](#typedef-qualified-rule-list)

<a id="example-92c1a511"></a>

<strong>Example:</strong>

[](#example-92c1a511) For example, the grammar for <a id="ref-for-at-font-face-rule①"></a>[&#64;font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) can be written as:

<a id="ref-for-at-font-face-rule②"></a><a id="ref-for-typedef-declaration-list②"></a>

``` text
<@font-face> = @font-face { <declaration-list> }
```

and then accompanying prose defines the valid <a id="ref-for-css-descriptor①"></a>[descriptors](#css-descriptor) allowed in the block.

The grammar for <a id="ref-for-at-ruledef-keyframes①"></a>[&#64;keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes) can be written as:

<a id="ref-for-at-ruledef-keyframes②"></a><a id="ref-for-typedef-qualified-rule-list②"></a><a id="ref-for-typedef-keyframe-selector"></a><a id="ref-for-typedef-declaration-list③"></a>

``` text
<@keyframes> = @keyframes { <qualified-rule-list> }
<keyframe-rule> = <keyframe-selector> { <declaration-list> }
```

and then accompanying prose defines that only \<keyframe-rule\>s are allowed in <a id="ref-for-at-ruledef-keyframes③"></a>[&#64;keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes), and that \<keyframe-rule\>s accept all animatable CSS properties, plus the <a id="ref-for-propdef-animation-timing-function"></a>[animation-timing-function](https://www.w3.org/TR/css-animations-1/#propdef-animation-timing-function) property, but they do not interact with the cascade.

### <a id="any-value"></a>7.2.  Defining Arbitrary Contents: the <a id="ref-for-typedef-declaration-value"></a>[\<declaration-value\>](#typedef-declaration-value) and <a id="ref-for-typedef-any-value"></a>[\<any-value\>](#typedef-any-value) productions[](#any-value)

In some grammars, it is useful to accept any reasonable input in the grammar, and do more specific error-handling on the contents manually (rather than simply invalidating the construct, as grammar mismatches tend to do).

For example, <a id="ref-for-custom-property③"></a>[custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) allow any reasonable value, as they can contain arbitrary pieces of other CSS properties, or be used for things that aren’t part of existing CSS at all. For another example, the <a id="ref-for-typedef-general-enclosed"></a>[\<general-enclosed\>](https://www.w3.org/TR/mediaqueries-5/#typedef-general-enclosed) production in Media Queries defines the bounds of what future syntax MQs will allow, and uses special logic to deal with "unknown" values.

To aid in this, two additional productions are defined:

The <a id="typedef-declaration-value"></a><strong>\<declaration-value\></strong> production matches <em>any</em> sequence of one or more tokens, so long as the sequence does not contain <a id="ref-for-typedef-bad-string-token③"></a>[\<bad-string-token\>](#typedef-bad-string-token), <a id="ref-for-typedef-bad-url-token⑧"></a>[\<bad-url-token\>](#typedef-bad-url-token), unmatched <a id="ref-for-tokendef-close-paren③"></a>[\<)-token\>](#tokendef-close-paren), <a id="ref-for-tokendef-close-square③"></a>[\<\]-token\>](#tokendef-close-square), or <a id="ref-for-tokendef-close-curly⑧"></a>[\<}-token\>](#tokendef-close-curly), or top-level <a id="ref-for-typedef-semicolon-token⑦"></a>[\<semicolon-token\>](#typedef-semicolon-token) tokens or <a id="ref-for-typedef-delim-token①⓪"></a>[\<delim-token\>](#typedef-delim-token) tokens with a value of "!". It represents the entirety of what a valid declaration can have as its value.

The <a id="typedef-any-value"></a><strong>\<any-value\></strong> production is identical to <a id="ref-for-typedef-declaration-value①"></a>[\<declaration-value\>](#typedef-declaration-value), but also allows top-level <a id="ref-for-typedef-semicolon-token⑧"></a>[\<semicolon-token\>](#typedef-semicolon-token) tokens and <a id="ref-for-typedef-delim-token①①"></a>[\<delim-token\>](#typedef-delim-token) tokens with a value of "!". It represents the entirety of what valid CSS can be in any context.

## <a id="css-stylesheets"></a>8.  CSS stylesheets[](#css-stylesheets)

To <a id="parse-a-css-stylesheet"></a><strong>parse a CSS stylesheet</strong>, first <a id="ref-for-parse-a-stylesheet①"></a>[parse a stylesheet](#parse-a-stylesheet). Interpret all of the resulting top-level <a id="ref-for-qualified-rule①④"></a>[qualified rules](#qualified-rule) as <a id="ref-for-style-rule③"></a>[style rules](#style-rule), defined below.

If any style rule is <a id="ref-for-css-invalid"></a>[invalid](#css-invalid), or any at-rule is not recognized or is invalid according to its grammar or context, it’s a <a id="ref-for-parse-error①②"></a>[parse error](#parse-error). Discard that rule.

### <a id="style-rules"></a>8.1.  Style rules[](#style-rules)

A <a id="style-rule"></a><strong>style rule</strong> is a <a id="ref-for-qualified-rule①⑤"></a>[qualified rule](#qualified-rule) that associates a <a id="ref-for-selector-list"></a>[selector list](https://www.w3.org/TR/selectors-4/#selector-list) with a list of property declarations and possibly a list of nested rules. They are also called [rule sets](https://www.w3.org/TR/CSS2/syndata.html#rule-sets) in [\[CSS2\]](#biblio-css2). CSS Cascading and Inheritance [\[CSS-CASCADE-3\]](#biblio-css-cascade-3) defines how the declarations inside of style rules participate in the cascade.

The prelude of the qualified rule is <a id="ref-for-css-parse-something-according-to-a-css-grammar④"></a>[parsed](#css-parse-something-according-to-a-css-grammar) as a <a id="ref-for-typedef-selector-list"></a>[\<selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-selector-list). If this returns failure, the entire style rule is <a id="ref-for-css-invalid①"></a>[invalid](#css-invalid).

The content of the qualified rule’s block is parsed as a <a id="ref-for-typedef-declaration-list④"></a>[\<declaration-list\>](#typedef-declaration-list). Qualified rules in this block are also <a id="ref-for-style-rule④"></a>[style rules](#style-rule). Unless defined otherwise by another specification or a future level of this specification, at-rules in that list are <a id="ref-for-css-invalid②"></a>[invalid](#css-invalid) and must be ignored.

Note: [\[CSS-NESTING-1\]](#biblio-css-nesting-1) defines that <a id="ref-for-conditional-group-rule"></a>[conditional group rules](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) and some other <a id="ref-for-at-rule②①"></a>[at-rules](#at-rule) are allowed inside of <a id="ref-for-style-rule⑤"></a>[style rules](#style-rule).

Declarations for an unknown CSS property or whose value does not match the syntax defined by the property are <a id="ref-for-css-invalid③"></a>[invalid](#css-invalid) and must be ignored. The validity of the style rule’s contents have no effect on the validity of the style rule itself. Unless otherwise specified, property names are <a id="ref-for-ascii-case-insensitive⑧"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive).

Note: The names of Custom Properties [\[CSS-VARIABLES\]](#biblio-css-variables) are case-sensitive.

<a id="ref-for-qualified-rule①⑥"></a>[Qualified rules](#qualified-rule) at the top-level of a CSS stylesheet are <a id="ref-for-style-rule⑥"></a>[style rules](#style-rule). Qualified rules in other contexts may or may not be style rules, as defined by the context.

<a id="example-4acffe04"></a>

<strong>Example:</strong>

[](#example-4acffe04) For example, qualified rules inside <a id="ref-for-at-ruledef-media③"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) rules [\[CSS3-CONDITIONAL\]](#biblio-css3-conditional) are style rules, but qualified rules inside <a id="ref-for-at-ruledef-keyframes④"></a>[&#64;keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes) rules [\[CSS3-ANIMATIONS\]](#biblio-css3-animations) are not.

### <a id="at-rules"></a>8.2.  At-rules[](#at-rules)

An <a id="at-rule"></a><strong>at-rule</strong> is a rule that begins with an at-keyword, and can thus be distinguished from <a id="ref-for-style-rule⑦"></a>[style rules](#style-rule) in the same context.

<a id="ref-for-at-rule②②"></a>[At-rules](#at-rule) are used to:

- group and structure style rules and other at-rules such as in <a id="ref-for-conditional-group-rule①"></a>[conditional group rules](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule)

- declare style information that is not associated with a particular element, such as defining <a id="ref-for-counter-style"></a>[counter styles](https://www.w3.org/TR/css-counter-styles-3/#counter-style)

- manage syntactic constructs such as [imports](https://www.w3.org/TR/css-cascade-3/#at-import) and [namespaces](#biblio-css-namespaces-3) keyword mappings

- and serve other miscellaneous purposes not served by a <a id="ref-for-style-rule⑧"></a>[style rule](#style-rule)

At-rules take many forms, depending on the specific rule and its purpose, but broadly speaking there are two kinds: <a id="statement-at-rule"></a><strong>statement at-rules</strong> which are simpler constructs that end in a semicolon, and <a id="block-at-rule"></a><strong>block at-rules</strong> which end in a <a id="ref-for-curly-block①"></a>[{}-block](#curly-block) that can contain nested <a id="ref-for-qualified-rule①⑦"></a>[qualified rules](#qualified-rule), <a id="ref-for-at-rule②③"></a>[at-rules](#at-rule), or <a id="ref-for-declaration②⓪"></a>[declarations](#declaration).

<a id="ref-for-block-at-rule①"></a>[Block at-rules](#block-at-rule) will typically contain a collection of (generic or <a id="ref-for-at-rule②④"></a>[at-rule](#at-rule)–specific) <a id="ref-for-at-rule②⑤"></a>at-rules, <a id="ref-for-qualified-rule①⑧"></a>[qualified rules](#qualified-rule), and/or <a id="ref-for-css-descriptor-declarations"></a>[descriptor declarations](#css-descriptor-declarations) subject to limitations defined by the <a id="ref-for-at-rule②⑥"></a>at-rule. <a id="css-descriptor"></a><strong>Descriptors</strong> are similar to <a id="ref-for-css-property②"></a>[properties](https://www.w3.org/TR/css-cascade-5/#css-property) (and are declared with the same syntax) but are associated with a particular type of <a id="ref-for-at-rule②⑦"></a>at-rule rather than with elements and boxes in the tree.

### <a id="charset-rule"></a>8.3.  The <a id="ref-for-at-ruledef-charset②"></a>[&#64;charset](#at-ruledef-charset) Rule[](#charset-rule)

The algorithm used to <a id="ref-for-determine-the-fallback-encoding①"></a>[determine the fallback encoding](#determine-the-fallback-encoding) for a stylesheet looks for a specific byte sequence as the very first few bytes in the file, which has the syntactic form of an <a id="ref-for-at-rule②⑧"></a>[at-rule](#at-rule) named &#34;&#64;charset&#34;.

Tests

- [page-utf16-css-bomless-utf16.html](https://wpt.fyi/results/css/css-syntax/charset/page-utf16-css-bomless-utf16.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-utf16-css-bomless-utf16.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-utf16-css-bomless-utf16.html)
- [page-utf16-css-bomless-utf16be.html](https://wpt.fyi/results/css/css-syntax/charset/page-utf16-css-bomless-utf16be.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-utf16-css-bomless-utf16be.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-utf16-css-bomless-utf16be.html)
- [page-utf16-css-no-decl-ascii-only.html](https://wpt.fyi/results/css/css-syntax/charset/page-utf16-css-no-decl-ascii-only.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-utf16-css-no-decl-ascii-only.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-utf16-css-no-decl-ascii-only.html)
- [page-utf16-css-no-decl.html](https://wpt.fyi/results/css/css-syntax/charset/page-utf16-css-no-decl.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-utf16-css-no-decl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-utf16-css-no-decl.html)
- [page-windows-1251-charset-attribute-bogus.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-charset-attribute-bogus.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-charset-attribute-bogus.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-charset-attribute-bogus.html)
- [page-windows-1251-charset-attribute-empty.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-charset-attribute-empty.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-charset-attribute-empty.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-charset-attribute-empty.html)
- [page-windows-1251-css-at-charset-1250-charset-attribute-windows-1253.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-at-charset-1250-charset-attribute-windows-1253.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-at-charset-1250-charset-attribute-windows-1253.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-at-charset-1250-charset-attribute-windows-1253.html)
- [page-windows-1251-css-at-charset-bogus-charset-attribute-windows-1250.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-at-charset-bogus-charset-attribute-windows-1250.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-at-charset-bogus-charset-attribute-windows-1250.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-at-charset-bogus-charset-attribute-windows-1250.html)
- [page-windows-1251-css-at-charset-bogus.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-at-charset-bogus.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-at-charset-bogus.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-at-charset-bogus.html)
- [page-windows-1251-css-at-charset-utf16-ascii-only.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-at-charset-utf16-ascii-only.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-at-charset-utf16-ascii-only.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-at-charset-utf16-ascii-only.html)
- [page-windows-1251-css-at-charset-utf16.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-at-charset-utf16.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-at-charset-utf16.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-at-charset-utf16.html)
- [page-windows-1251-css-at-charset-utf16be.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-at-charset-utf16be.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-at-charset-utf16be.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-at-charset-utf16be.html)
- [page-windows-1251-css-at-charset-windows-1250-in-utf16.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-at-charset-windows-1250-in-utf16.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-at-charset-windows-1250-in-utf16.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-at-charset-windows-1250-in-utf16.html)
- [page-windows-1251-css-at-charset-windows-1250-in-utf16be.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-at-charset-windows-1250-in-utf16be.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-at-charset-windows-1250-in-utf16be.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-at-charset-windows-1250-in-utf16be.html)
- [page-windows-1251-css-http-bogus-at-charset-windows-1250.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-http-bogus-at-charset-windows-1250.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-http-bogus-at-charset-windows-1250.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-http-bogus-at-charset-windows-1250.html)
- [page-windows-1251-css-http-bogus.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-http-bogus.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-http-bogus.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-http-bogus.html)
- [page-windows-1251-css-http-windows-1250-at-charset-windows-1253.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-http-windows-1250-at-charset-windows-1253.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-http-windows-1250-at-charset-windows-1253.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-http-windows-1250-at-charset-windows-1253.html)
- [page-windows-1251-css-no-decl.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-no-decl.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-no-decl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-no-decl.html)
- [page-windows-1251-css-utf8-bom.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1251-css-utf8-bom.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1251-css-utf8-bom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1251-css-utf8-bom.html)
- [page-windows-1252-http-windows-1251-css-utf8-bom.html](https://wpt.fyi/results/css/css-syntax/charset/page-windows-1252-http-windows-1251-css-utf8-bom.html) [(live test)](http://wpt.live/css/css-syntax/charset/page-windows-1252-http-windows-1251-css-utf8-bom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/page-windows-1252-http-windows-1251-css-utf8-bom.html)
- [xml-stylesheet-page-windows-1251-charset-attribute-windows-1250.xhtml](https://wpt.fyi/results/css/css-syntax/charset/xml-stylesheet-page-windows-1251-charset-attribute-windows-1250.xhtml) [(live test)](http://wpt.live/css/css-syntax/charset/xml-stylesheet-page-windows-1251-charset-attribute-windows-1250.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset/xml-stylesheet-page-windows-1251-charset-attribute-windows-1250.xhtml)

However, there is no actual <a id="ref-for-at-rule②⑨"></a>[at-rule](#at-rule) named <a id="at-ruledef-charset"></a><strong>&#64;charset</strong>. When a stylesheet is actually parsed, any occurrences of an <a id="ref-for-at-ruledef-charset③"></a>[&#64;charset](#at-ruledef-charset) rule must be treated as an unrecognized rule, and thus dropped as invalid when the stylesheet is grammar-checked.

Note: In CSS 2.1, <a id="ref-for-at-ruledef-charset④"></a>[&#64;charset](#at-ruledef-charset) was a valid rule. Some legacy specs may still refer to a <a id="ref-for-at-ruledef-charset⑤"></a>&#64;charset rule, and explicitly talk about its presence in the stylesheet.

Tests

- [charset-is-not-a-rule.html](https://wpt.fyi/results/css/css-syntax/charset-is-not-a-rule.html) [(live test)](http://wpt.live/css/css-syntax/charset-is-not-a-rule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/charset-is-not-a-rule.html)

## <a id="serialization"></a>9.  Serialization[](#serialization)

The tokenizer described in this specification does not produce tokens for comments, or otherwise preserve them in any way. Implementations may preserve the contents of comments and their location in the token stream. If they do, this preserved information must have no effect on the parsing step.

This specification does not define how to serialize CSS in general, leaving that task to the [\[CSSOM\]](#biblio-cssom) and individual feature specifications. In particular, the serialization of comments and whitespace is not defined.

The only requirement for serialization is that it must "round-trip" with parsing, that is, parsing the stylesheet must produce the same data structures as parsing, serializing, and parsing again, except for consecutive <a id="ref-for-typedef-whitespace-token①⓪"></a>[\<whitespace-token\>](#typedef-whitespace-token)s, which may be collapsed into a single token.

Note: This exception can exist because CSS grammars always interpret any amount of whitespace as identical to a single space.

<a id="serialization-tables"></a>

<strong>Note:</strong>

[](#serialization-tables) To satisfy this requirement:

- A <a id="ref-for-typedef-delim-token①②"></a>[\<delim-token\>](#typedef-delim-token) containing U+005C REVERSE SOLIDUS (&#92;) must be serialized as U+005C REVERSE SOLIDUS followed by a <a id="ref-for-newline④"></a>[newline](#newline). (The tokenizer only ever emits such a token followed by a <a id="ref-for-typedef-whitespace-token①①"></a>[\<whitespace-token\>](#typedef-whitespace-token) that starts with a newline.)

- A <a id="ref-for-typedef-hash-token⑦"></a>[\<hash-token\>](#typedef-hash-token) with the "unrestricted" type flag may not need as much escaping as the same token with the "id" type flag.

- The unit of a <a id="ref-for-typedef-dimension-token①②"></a>[\<dimension-token\>](#typedef-dimension-token) may need escaping to disambiguate with scientific notation.

- For any consecutive pair of tokens, if the first token shows up in the row headings of the following table, and the second token shows up in the column headings, and there’s a ✗ in the cell denoted by the intersection of the chosen row and column, the pair of tokens must be serialized with a comment between them.

  If the tokenizer preserves comments, and there were comments originally between the token pair, the preserved comment(s) should be used; otherwise, an empty comment (<code>/&#42;&#42;/</code>) must be inserted. (Preserved comments may be reinserted even if the following tables don’t require a comment between two tokens.)

  Single characters in the row and column headings represent a <a id="ref-for-typedef-delim-token①③"></a>[\<delim-token\>](#typedef-delim-token) with that value, except for "<code>(</code>", which represents a <a id="ref-for-tokendef-open-paren⑤"></a>[(-token](#tokendef-open-paren).

| Column 1                    | Column 2               | Column 3                  | Column 4             | Column 5                 | Column 6           | Column 7                | Column 8                    | Column 9                   | Column 10            | Column 11          | Column 12           | Column 13          |
|-----------------------------|------------------------|---------------------------|----------------------|--------------------------|--------------------|-------------------------|-----------------------------|----------------------------|----------------------|--------------------|---------------------|--------------------|
|                             | <strong>ident</strong> | <strong>function</strong> | <strong>url</strong> | <strong>bad url</strong> | <strong>-</strong> | <strong>number</strong> | <strong>percentage</strong> | <strong>dimension</strong> | <strong>CDC</strong> | <strong>(</strong> | <strong>\*</strong> | <strong>%</strong> |
| <strong>ident</strong>      | ✗                      | ✗                         | ✗                    | ✗                        | ✗                  | ✗                       | ✗                           | ✗                          | ✗                    | ✗                  |                     |                    |
| <strong>at-keyword</strong> | ✗                      | ✗                         | ✗                    | ✗                        | ✗                  | ✗                       | ✗                           | ✗                          | ✗                    |                    |                     |                    |
| <strong>hash</strong>       | ✗                      | ✗                         | ✗                    | ✗                        | ✗                  | ✗                       | ✗                           | ✗                          | ✗                    |                    |                     |                    |
| <strong>dimension</strong>  | ✗                      | ✗                         | ✗                    | ✗                        | ✗                  | ✗                       | ✗                           | ✗                          | ✗                    |                    |                     |                    |
| <strong>\#</strong>         | ✗                      | ✗                         | ✗                    | ✗                        | ✗                  | ✗                       | ✗                           | ✗                          | ✗                    |                    |                     |                    |
| <strong>-</strong>          | ✗                      | ✗                         | ✗                    | ✗                        | ✗                  | ✗                       | ✗                           | ✗                          | ✗                    |                    |                     |                    |
| <strong>number</strong>     | ✗                      | ✗                         | ✗                    | ✗                        |                    | ✗                       | ✗                           | ✗                          | ✗                    |                    |                     | ✗                  |
| <strong>&#64;</strong>      | ✗                      | ✗                         | ✗                    | ✗                        | ✗                  |                         |                             |                            | ✗                    |                    |                     |                    |
| <strong>.</strong>          |                        |                           |                      |                          |                    | ✗                       | ✗                           | ✗                          |                      |                    |                     |                    |
| <strong>+</strong>          |                        |                           |                      |                          |                    | ✗                       | ✗                           | ✗                          |                      |                    |                     |                    |
| <strong>/</strong>          |                        |                           |                      |                          |                    |                         |                             |                            |                      |                    | ✗                   |                    |

Tests

- [serialize-consecutive-tokens.html](https://wpt.fyi/results/css/css-syntax/serialize-consecutive-tokens.html) [(live test)](http://wpt.live/css/css-syntax/serialize-consecutive-tokens.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/serialize-consecutive-tokens.html)
- [serialize-escape-identifiers.html](https://wpt.fyi/results/css/css-syntax/serialize-escape-identifiers.html) [(live test)](http://wpt.live/css/css-syntax/serialize-escape-identifiers.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-syntax/serialize-escape-identifiers.html)

### <a id="serializing-anb"></a>9.1.  Serializing <var>\<a-n-plus-b\></var>[](#serializing-anb)

To <a id="serialize-an-a-n-plus-b-value"></a><strong>serialize an <a id="ref-for-anb-production"></a>[\<a-n-plus-b\>](#anb-production) value</strong>, with integer values <var>A</var> and <var>B</var>:

1.  If <var>A</var> is zero, return the serialization of <var>B</var>.

2.  Otherwise, let <var>result</var> initially be an empty <a id="ref-for-string④"></a>[string](https://infra.spec.whatwg.org/#string).

3.  <strong><var>A</var> is <code>1</code></strong>

    Append "n" to <var>result</var>.

    <strong><var>A</var> is <code>-1</code></strong>

    Append "-n" to <var>result</var>.

    <strong><var>A</var> is non-zero</strong>

    Serialize <var>A</var> and append it to <var>result</var>, then append "n" to <var>result</var>.

4.  <strong><var>B</var> is greater than zero</strong>

    Append "+" to <var>result</var>, then append the serialization of <var>B</var> to <var>result</var>.

    <strong><var>B</var> is less than zero</strong>

    Append the serialization of <var>B</var> to <var>result</var>.

5.  Return <var>result</var>.

## <a id="privacy"></a>10.  Privacy Considerations[](#privacy)

This specification introduces no new privacy concerns.

## <a id="security"></a>11.  Security Considerations[](#security)

This specification improves security, in that CSS parsing is now unambiguously defined for all inputs.

Insofar as old parsers, such as whitelists/filters, parse differently from this specification, they are somewhat insecure, but the previous parsing specification left a lot of ambiguous corner cases which browsers interpreted differently, so those filters were potentially insecure already, and this specification does not worsen the situation.

## <a id="changes"></a>12.  Changes[](#changes)

<em>This section is non-normative.</em>

### <a id="changes-CR-20211224"></a>12.1.  Changes from the 24 December 2021 Candidate Recommendation Draft[](#changes-CR-20211224)

The following substantive changes were made:

- Rename \<an+b\> to \<a-n-plus-b\> ([9473](https://github.com/w3c/csswg-drafts/issues/9473))

- Remove duplicate token stream creations ([PR 11129](https://github.com/w3c/csswg-drafts/pull/11129))

- Consolidate valdef syntax definition

- Retire &#64;nest, introduce the 'nested declarations rule' ([10386](https://github.com/w3c/csswg-drafts/pull/10386))

- Change block-contents parsing to maintain relative order of interleaved declarations and rules. Add &#64;nest to Nesting to represent them ([8738](https://github.com/w3c/csswg-drafts/issues/8738))

- Blocks are only allowed as the entire value of a property ([9317](https://github.com/w3c/csswg-drafts/issues/9317))

- Don’t apply the hungry custom-prop-lookalike behavior if you’re not in a context that allows properties in the first place ([9336](https://github.com/w3c/csswg-drafts/issues/9336))

- Clarified that surrogates can only be produced by directly assigning a DOMString containing one, via a CSS Object Model operation ([PR 7997](https://github.com/w3c/csswg-drafts/pull/7997))

- The definition of <a id="ref-for-non-ascii-code-point"></a>[non-ASCII code point](https://www.w3.org/TR/css-syntax-3/#non-ascii-code-point) was restricted to omit some potentially troublesome codepoints. ([7129](https://github.com/w3c/csswg-drafts/issues/7129))

- Defined the <code>&#60;foo()&#62;</code> and <code>&#60;@foo&#62;</code> productions. ([5728](https://github.com/w3c/csswg-drafts/issues/5728))

- Allow nested style rules. ([7961](https://github.com/w3c/csswg-drafts/issues/7961#issuecomment-1514955984)).

- As part of allowing Nesting, significantly rewrote the entire parsing section. Notably, removed "parse a list of rules" and "parse a list of declarations" in favor of "parse a stylesheet’s contents" and "parse a block’s contents". Only additional normative change is that semicolons trigger slightly different error-recovery now in some contexts, so that parsing of things like <a id="ref-for-at-ruledef-media④"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) blocks is identical whether they’re nested or not. ([8834](https://github.com/w3c/csswg-drafts/issues/8834)).

- Removed the attempt at a <code>&#60;urange&#62;</code> production in terms of existing tokens, instead relying on a special re-parse from source specifically when you’re parsing the <a id="ref-for-descdef-font-face-unicode-range②"></a>[unicode-range](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-unicode-range) descriptor. ([8835](https://github.com/w3c/csswg-drafts/issues/8835))

- Since the above removed the main need to preserve a token’s "representation" (the original string it was parsed from), removed the rest of the references to "representation" in the An+B section and instead just recorded the few bits of information necessary for that (whether or not the number had an explicit sign).

- Explicitly noted that some uses (<a id="ref-for-custom-property④"></a>[custom properties](https://www.w3.org/TR/css-variables-1/#custom-property), the unicode-range descriptor) require access to the original string of a declaration’s entire value, and marked where that occurs in the parser.

- When <a id="ref-for-consume-a-blocks-contents③"></a>[consuming a block’s contents](#consume-a-blocks-contents), rather than grouping all declarations together, regardless of where they appear relative to other rules, declarations now maintain their relative position wrt surrounding rules. It’s up to individual rules to determine what they actually do with that information.

### <a id="changes-CR-20190716"></a>12.2.  Changes from the 16 August 2019 Candidate Recommendation[](#changes-CR-20190716)

The following substantive changes were made:

- Added a new [§ 5.4.2 Parse a comma-separated list according to a CSS grammar](#parse-comma-list) algorithm.

- Added a new "Parse a style block’s content" algorithm and a corresponding <a id="ref-for-typedef-style-block"></a>[\<style-block\>](https://www.w3.org/TR/css-syntax-3/#typedef-style-block) production, and defined that <a id="ref-for-style-rule⑨"></a>[style rules](#style-rule) use it.

- Aligned <a id="ref-for-parse-a-stylesheet②"></a>[parse a stylesheet](#parse-a-stylesheet) with the Fetch-related shenanigans. (See [commit](https://github.com/w3c/csswg-drafts/commit/64becb59fe678760608a6f2a8d6bfb3a334500c9#diff-9d409f899ee6e2cc8cd87356d0e2a0ff739c5aa9663d83f49c951c379a38f1f6).)

  > To <a id="ref-for-parse-a-stylesheet③"></a>[parse a stylesheet](#parse-a-stylesheet) from an <var>input</var> <u>given an optional <a id="ref-for-concept-url①"></a>[url](https://url.spec.whatwg.org/#concept-url) <var>location</var></u> :
  >
  > 1.  ...
  > 2.  Create a new stylesheet <u>, with its <a id="ref-for-concept-css-style-sheet-location①"></a>[location](https://www.w3.org/TR/cssom-1/#concept-css-style-sheet-location) set to <var>location</var> (or null, if <var>location</var> was not passed).</u>
  > 3.  ...

The following editorial changes were made:

- Added [§ 8.2 At-rules](#at-rules) to provide definitions for <a id="ref-for-at-rule③⓪"></a>[at-rules](#at-rule), <a id="ref-for-statement-at-rule①"></a>[statement at-rules](#statement-at-rule), <a id="ref-for-block-at-rule②"></a>[block at-rules](#block-at-rule), and <a id="ref-for-css-descriptor②"></a>[descriptors](#css-descriptor). ([5633](https://github.com/w3c/csswg-drafts/issues/5633))

- Improved the definition text for <a id="ref-for-declaration②①"></a>[declaration](#declaration), and added definitions for <a id="ref-for-css-property-declarations①"></a>[property declarations](#css-property-declarations) and <a id="ref-for-css-descriptor-declarations①"></a>[descriptor declarations](#css-descriptor-declarations).

- Switched to consistently refer to <a id="ref-for-ident-sequence⑤"></a>[ident sequence](#ident-sequence), rather than sometimes using the term “name”.

- Explicitly named several of the pre-tokenizing processes, and explicitly referred to them in the parsing entry points (rather than relying on a blanket "do X at the start of these algorithms" statement).

- Added more entries to the "put a comment between them" table, to properly handle the fact that idents can now start with <code>--</code>. ([6874](https://github.com/w3c/csswg-drafts/pull/6874))

### <a id="changes-CR-20140220"></a>12.3.  Changes from the 20 February 2014 Candidate Recommendation[](#changes-CR-20140220)

The following substantive changes were made:

- Removed <a id="ref-for-typedef-unicode-range-token⑦"></a>[\<unicode-range-token\>](#typedef-unicode-range-token), in favor of creating a <a id="ref-for-typedef-urange"></a>[\<urange\>](https://www.w3.org/TR/css-syntax-3/#typedef-urange) production.

- url() functions that contain a string are now parsed as normal <a id="ref-for-typedef-function-token①⓪"></a>[\<function-token\>](#typedef-function-token)s. url() functions that contain "raw" URLs are still specially parsed as <a id="ref-for-typedef-url-token①①"></a>[\<url-token\>](#typedef-url-token)s.

- Fixed a bug in the "Consume a URL token" algorithm, where it didn’t consume the quote character starting a string before attempting to consume the string.

- Fixed a bug in several of the parser algorithms related to the current/next token and things getting consumed early/late.

- Fix several bugs in the tokenization and parsing algorithms.

- Change the definition of ident-like tokens to allow "--" to start an ident. As part of this, rearrange the ordering of the clauses in the "-" step of <a id="ref-for-tokenizer-consume-a-token②"></a>[consume a token](#tokenizer-consume-a-token) so that <a id="ref-for-typedef-cdc-token③"></a>[\<CDC-token\>](#typedef-cdc-token)s are recognized as such instead of becoming a -- <a id="ref-for-typedef-ident-token①②"></a>[\<ident-token\>](#typedef-ident-token).

- Don’t serialize the digit in an <a id="ref-for-anb-production①"></a>[\<a-n-plus-b\>](#anb-production) when A is 1 or -1.

- Define all tokens to have a representation.

- Fixed minor bug in <a id="ref-for-check-if-two-code-points-are-a-valid-escape⑧"></a>[check if two code points are a valid escape](#check-if-two-code-points-are-a-valid-escape)—​a <code>&#92;</code> followed by an EOF is now correctly reported as <em>not</em> a valid escape. A final <code>&#92;</code> in a stylesheet now just emits itself as a <a id="ref-for-typedef-delim-token①④"></a>[\<delim-token\>](#typedef-delim-token).

- &#64;charset is no longer a valid CSS rule (there’s just an encoding declaration that <em>looks</em> like a rule named &#64;charset)

- Trimmed whitespace from the beginning/ending of a declaration’s value during parsing.

- Removed the Selectors-specific tokens, per WG resolution.

- Filtered <a id="ref-for-surrogate③"></a>[surrogates](https://infra.spec.whatwg.org/#surrogate) from the input stream, per WG resolution. Now the entire specification operates only on <a id="ref-for-scalar-value"></a>[scalar values](https://infra.spec.whatwg.org/#scalar-value).

The following editorial changes were made:

- The "Consume a string token" algorithm was changed to allow calling it without specifying an explicit ending token, so that it uses the current token instead. The three call-sites of the algorithm were changed to use that form.

- Minor editorial restructuring of algorithms.

- Added the <a id="ref-for-css-parse-something-according-to-a-css-grammar⑤"></a>[parse](#css-parse-something-according-to-a-css-grammar) and <a id="ref-for-parse-a-comma-separated-list-of-component-values①"></a>[parse a comma-separated list of component values](#parse-a-comma-separated-list-of-component-values) API entry points.

- Added the <a id="ref-for-typedef-declaration-value②"></a>[\<declaration-value\>](#typedef-declaration-value) and <a id="ref-for-typedef-any-value①"></a>[\<any-value\>](#typedef-any-value) productions.

- Removed "code point" and "surrogate code point" in favor of the identical definitions in the Infra Standard.

- Clarified on every range that they are inclusive.

- Added a column to the comment-insertion table to handle a number token appearing next to a "%" delim token.

[A Disposition of Comments is available.](https://github.com/w3c/csswg-drafts/milestone/5?closed=1)

### <a id="changes-WD-20131105"></a>12.4.  Changes from the 5 November 2013 Last Call Working Draft[](#changes-WD-20131105)

- The [Serialization](#serialization) section has been rewritten to make only the "round-trip" requirement normative, and move the details of how to achieve it into a note. Some corner cases in these details have been fixed.

- [\[ENCODING\]](#biblio-encoding) has been added to the list of normative references. It was already referenced in normative text before, just not listed as such.

- In the algorithm to <a id="ref-for-determine-the-fallback-encoding②"></a>[determine the fallback encoding](#determine-the-fallback-encoding) of a stylesheet, limit the <code>@charset</code> byte sequence to 1024 bytes. This aligns with what HTML does for <code>&#60;meta charset&#62;</code> and makes sure the size of the sequence is bounded. This only makes a difference with leading or trailing whitespace in the encoding label:

  ``` text
  @charset "   (lots of whitespace)   utf-8";
  ```

### <a id="changes-WD-20130919"></a>12.5.  Changes from the 19 September 2013 Working Draft[](#changes-WD-20130919)

- The concept of <a id="ref-for-environment-encoding③"></a>[environment encoding](#environment-encoding) was added. The behavior does not change, but some of the definitions should be moved to the relevant specs.

### <a id="changes-css21"></a>12.6.  Changes from CSS 2.1 and Selectors Level 3[](#changes-css21)

Note: The point of this spec is to match reality; changes from CSS2.1 are nearly always because CSS 2.1 specified something that doesn’t match actual browser behavior, or left something unspecified. If some detail doesn’t match browsers, please let me know as it’s almost certainly unintentional.

Changes in decoding from a byte stream:

- Only detect <a id="ref-for-at-ruledef-charset⑥"></a>[&#64;charset](#at-ruledef-charset) rules in ASCII-compatible byte patterns.
- Ignore <a id="ref-for-at-ruledef-charset⑦"></a>[&#64;charset](#at-ruledef-charset) rules that specify an ASCII-incompatible encoding, as that would cause the rule itself to not decode properly.
- Refer to [\[ENCODING\]](#biblio-encoding) rather than the IANA registry for character encodings.

Tokenization changes:

- Any U+0000 NULL <a id="ref-for-code-point⑧⑤"></a>[code point](https://infra.spec.whatwg.org/#code-point) in the CSS source is replaced with U+FFFD REPLACEMENT CHARACTER.

- Any hexadecimal escape sequence such as \0 that evaluates to zero produce U+FFFD REPLACEMENT CHARACTER rather than U+0000 NULL.

- The definition of <a id="ref-for-non-ascii-ident-code-point①"></a>[non-ASCII ident code point](#non-ascii-ident-code-point) was changed to be consistent with HTML’s <a id="ref-for-valid-custom-element-name①"></a>[valid custom element names](https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name).

- Tokenization does not emit COMMENT or BAD_COMMENT tokens anymore. BAD_COMMENT is now considered the same as a normal token (not an error). [Serialization](#serialization) is responsible for inserting comments as necessary between tokens that need to be separated, e.g. two consecutive <a id="ref-for-typedef-ident-token①③"></a>[\<ident-token\>](#typedef-ident-token)s.

- The <a id="ref-for-typedef-unicode-range-token⑧"></a>[\<unicode-range-token\>](#typedef-unicode-range-token) was removed, as it was low value and occasionally actively harmful. (u+a { font-weight: bold; } was an invalid selector, for example...)

  Instead, a <a id="ref-for-typedef-urange①"></a>[\<urange\>](https://www.w3.org/TR/css-syntax-3/#typedef-urange) production was added, based on token patterns. It is technically looser than what 2.1 allowed (any number of digits and ? characters), but not in any way that should impact its use in practice.

- Apply the [EOF error handling rule](https://www.w3.org/TR/CSS2/syndata.html#unexpected-eof) in the tokenizer and emit normal <a id="ref-for-typedef-string-token⑧"></a>[\<string-token\>](#typedef-string-token) and <a id="ref-for-typedef-url-token①②"></a>[\<url-token\>](#typedef-url-token) rather than BAD_STRING or BAD_URI on EOF.

- The BAD_URI token (now <a id="ref-for-typedef-bad-url-token⑨"></a>[\<bad-url-token\>](#typedef-bad-url-token)) is "self-contained". In other words, once the tokenizer realizes it’s in a <a id="ref-for-typedef-bad-url-token①⓪"></a>\<bad-url-token\> rather than a <a id="ref-for-typedef-url-token①③"></a>[\<url-token\>](#typedef-url-token), it just seeks forward to look for the closing ), ignoring everything else. This behavior is simpler than treating it like a <a id="ref-for-typedef-function-token①①"></a>[\<function-token\>](#typedef-function-token) and paying attention to opened blocks and such. Only WebKit exhibits this behavior, but it doesn’t appear that we’ve gotten any compat bugs from it.

- The <a id="ref-for-typedef-comma-token②"></a>[\<comma-token\>](#typedef-comma-token) has been added.

- <a id="ref-for-typedef-number-token⑧"></a>[\<number-token\>](#typedef-number-token), <a id="ref-for-typedef-percentage-token④"></a>[\<percentage-token\>](#typedef-percentage-token), and <a id="ref-for-typedef-dimension-token①③"></a>[\<dimension-token\>](#typedef-dimension-token) have been changed to include the preceding +/- sign as part of their value (rather than as a separate <a id="ref-for-typedef-delim-token①⑤"></a>[\<delim-token\>](#typedef-delim-token) that needs to be manually handled every time the token is mentioned in other specs). The only consequence of this is that comments can no longer be inserted between the sign and the number.

- Scientific notation is supported for numbers/percentages/dimensions to match SVG, per WG resolution.

- Hexadecimal escape for <a id="ref-for-surrogate④"></a>[surrogate](https://infra.spec.whatwg.org/#surrogate) now emit a replacement character rather than the surrogate. This allows implementations to safely use UTF-16 internally.

Parsing changes:

- Any list of declarations now also accepts at-rules, like <a id="ref-for-at-ruledef-page②"></a>[&#64;page](https://www.w3.org/TR/css-page-3/#at-ruledef-page), per WG resolution. This makes a difference in error handling even if no such at-rules are defined yet: an at-rule, valid or not, ends at a {} block without a <a id="ref-for-typedef-semicolon-token⑨"></a>[\<semicolon-token\>](#typedef-semicolon-token) and lets the next declaration begin.
- The handling of some miscellaneous "special" tokens (like an unmatched <a id="ref-for-tokendef-close-curly⑨"></a>[\<}-token\>](#tokendef-close-curly)) showing up in various places in the grammar has been specified with some reasonable behavior shown by at least one browser. Previously, stylesheets with those tokens in those places just didn’t match the stylesheet grammar at all, so their handling was totally undefined. Specifically:
  - \[\] blocks, () blocks and functions can now contain {} blocks, <a id="ref-for-typedef-at-keyword-token⑨"></a>[\<at-keyword-token\>](#typedef-at-keyword-token)s or <a id="ref-for-typedef-semicolon-token①⓪"></a>[\<semicolon-token\>](#typedef-semicolon-token)s
  - Qualified rule preludes can now contain semicolons
  - Qualified rule and at-rule preludes can now contain <a id="ref-for-typedef-at-keyword-token①⓪"></a>[\<at-keyword-token\>](#typedef-at-keyword-token)s

<var>An+B</var> changes from Selectors Level 3 [\[SELECT\]](#biblio-select):

- The <var>An+B</var> microsyntax has now been formally defined in terms of CSS tokens, rather than with a separate tokenizer. This has resulted in minor differences:
  - In some cases, minus signs or digits can be escaped (when they appear as part of the unit of a <a id="ref-for-typedef-dimension-token①④"></a>[\<dimension-token\>](#typedef-dimension-token) or <a id="ref-for-typedef-ident-token①④"></a>[\<ident-token\>](#typedef-ident-token)).

## <a id="acknowledgments"></a> Acknowledgments[](#acknowledgments)

Thanks for feedback and contributions from Anne van Kesteren, David Baron, Elika J. Etemad (fantasai), Henri Sivonen, Johannes Koch, 呂康豪 (Kang-Hao Lu), Marc O’Morain, Raffaello Giulietti, Simon Pieter, Tyler Karaszewski, and Zack Weinberg.

## <a id="w3c-conformance"></a> Conformance[](#w3c-conformance)

### <a id="w3c-conventions"></a> Document conventions[](#w3c-conventions)

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with <code>class=&#34;example&#34;</code>, like this:

<a id="w3c-example"></a>

<strong>Example:</strong>

[](#w3c-example)

This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with <code>class=&#34;note&#34;</code>, like this:

Note, this is an informative note.

<strong>Note:</strong>

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with <code>&#60;strong class=&#34;advisement&#34;&#62;</code>, like this: <strong>UAs MUST provide an accessible alternative.</strong>

<strong>Advisement:</strong>

Tests

Tests relating to the content of this specification may be documented in “Tests” blocks like this one. Any such block is non-normative.

------------------------------------------------------------------------

### <a id="w3c-conformance-classes"></a> Conformance classes[](#w3c-conformance-classes)

Conformance to this specification is defined for three conformance classes:

<strong>style sheet</strong>

A [CSS style sheet](https://www.w3.org/TR/CSS21/conform.html#style-sheet).

<strong>renderer</strong>

A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

<strong>authoring tool</strong>

A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="w3c-partial"></a> Partial implementations[](#w3c-partial)

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](https://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="w3c-conform-future-proofing"></a> Implementations of Unstable and Proprietary Features[](#w3c-conform-future-proofing)

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

### <a id="w3c-testing"></a> Non-experimental implementations[](#w3c-testing)

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at <https://www.w3.org/Style/CSS/Test/>. Questions should be directed to the [public-css-testsuite&#64;w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

### <a id="w3c-cr-exit-criteria"></a> CR exit criteria[](#w3c-cr-exit-criteria)

For this specification to be advanced to Proposed Recommendation, there must be at least two independent, interoperable implementations of each feature. Each feature may be implemented by a different set of products, there is no requirement that all features be implemented by a single product. For the purposes of this criterion, we define the following terms:

<strong>independent</strong>

each implementation must be developed by a different party and cannot share, reuse, or derive from code used by another qualifying implementation. Sections of code that have no bearing on the implementation of this specification are exempt from this requirement.

<strong>interoperable</strong>

passing the respective test case(s) in the official CSS test suite, or, if the implementation is not a Web browser, an equivalent test. Every relevant test in the test suite should have an equivalent test created if such a user agent (UA) is to be used to claim interoperability. In addition if such a UA is to be used to claim interoperability, then there must one or more additional UAs which can also pass those equivalent tests in the same way for the purpose of interoperability. The equivalent tests must be made publicly available for the purposes of peer review.

<strong>implementation</strong>

a user agent which:

1.  implements the specification.
2.  is available to the general public. The implementation may be a shipping product or other publicly available version (i.e., beta version, preview release, or "nightly build"). Non-shipping product releases must have implemented the feature(s) for a period of at least one month in order to demonstrate stability.
3.  is not experimental (i.e., a version specifically designed to pass the test suite and is not intended for normal usage going forward).

The specification will remain Candidate Recommendation for at least six months.

## <a id="index"></a>Index[](#index)

### <a id="index-defined-here"></a>Terms defined by this specification[](#index-defined-here)

- [An+B](#anb), in § 6
- [\<a-n-plus-b\>](#anb-production), in § 6.2
- [\<any-value\>](#typedef-any-value), in § 7.2
- [are a valid escape](#check-if-two-code-points-are-a-valid-escape), in § 4.3.8
- [\<at-keyword-token\>](#typedef-at-keyword-token), in § 4
- [at-rule](#at-rule), in § 8.2
- [\<at-rule-list\>](#typedef-at-rule-list), in § 7.1
- [\<bad-string-token\>](#typedef-bad-string-token), in § 4
- [\<bad-url-token\>](#typedef-bad-url-token), in § 4
- [()-block](#paren-block), in § 5.2
- [\[\]-block](#square-block), in § 5.2
- [{}-block](#curly-block), in § 5.2
- [block at-rule](#block-at-rule), in § 8.2
- [\<block-contents\>](#typedef-block-contents), in § 7.1
- [\<CDC-token\>](#typedef-cdc-token), in § 4
- [\<CDO-token\>](#typedef-cdo-token), in § 4
- [&#64;charset](#at-ruledef-charset), in § 8.3
- [check if three code points would start an ident sequence](#check-if-three-code-points-would-start-an-ident-sequence), in § 4.3.9
- [check if three code points would start a number](#check-if-three-code-points-would-start-a-number), in § 4.3.10
- [check if three code points would start a unicode-range](#check-if-three-code-points-would-start-a-unicode-range), in § 4.3.11
- [check if two code points are a valid escape](#check-if-two-code-points-are-a-valid-escape), in § 4.3.8
- [\<colon-token\>](#typedef-colon-token), in § 4
- [\<comma-token\>](#typedef-comma-token), in § 4
- [component value](#component-value), in § 5.2
- [consume a block](#consume-a-block), in § 5.5.4
- [consume a block’s contents](#consume-a-blocks-contents), in § 5.5.5
- [consume a component value](#consume-a-component-value), in § 5.5.8
- [consume a declaration](#consume-a-declaration), in § 5.5.6
- [consume a function](#consume-a-function), in § 5.5.10
- [consume a list of component values](#consume-a-list-of-component-values), in § 5.5.7
- [consume an at-rule](#consume-an-at-rule), in § 5.5.2
- [consume an escaped code point](#consume-an-escaped-code-point), in § 4.3.7
- [consume an ident-like token](#consume-an-ident-like-token), in § 4.3.4
- [consume an ident sequence](#consume-an-ident-sequence), in § 4.3.12
- [consume a number](#consume-a-number), in § 4.3.13
- [consume a numeric token](#consume-a-numeric-token), in § 4.3.3
- [consume a qualified rule](#consume-a-qualified-rule), in § 5.5.3
- [consume a simple block](#consume-a-simple-block), in § 5.5.9
- [consume a string token](#consume-a-string-token), in § 4.3.5
- [consume a stylesheet’s contents](#consume-a-stylesheets-contents), in § 5.5.1
- consume a token
  - [dfn for token stream](#token-stream-consume-a-token), in § 5.3
  - [dfn for tokenizer](#tokenizer-consume-a-token), in § 4.3.1
- [consume a unicode-range token](#consume-a-unicode-range-token), in § 4.3.14
- [consume a url token](#consume-a-url-token), in § 4.3.6
- [consume comments](#consume-comments), in § 4.3.2
- [consume the remnants of a bad declaration](#consume-the-remnants-of-a-bad-declaration), in § 5.5.6
- [consume the remnants of a bad url](#consume-the-remnants-of-a-bad-url), in § 4.3.15
- [consume the value of a unicode-range descriptor](#consume-the-value-of-a-unicode-range-descriptor), in § 5.5.11
- [CSS ident sequence](#ident-sequence), in § 4.2
- [current input code point](#current-input-code-point), in § 4.2
- [\<dashndashdigit-ident\>](#typedef-dashndashdigit-ident), in § 6.2
- [declaration](#declaration), in § 5.2
- [\<declaration-list\>](#typedef-declaration-list), in § 7.1
- [\<declaration-rule-list\>](#typedef-declaration-rule-list), in § 7.1
- [\<declaration-value\>](#typedef-declaration-value), in § 7.2
- [decode bytes](#css-decode-bytes), in § 3.2
- [\<delim-token\>](#typedef-delim-token), in § 4
- [descriptor](#css-descriptor), in § 8.2
- [descriptor declarations](#css-descriptor-declarations), in § 5.2
- [determine the fallback encoding](#determine-the-fallback-encoding), in § 3.2
- [digit](#digit), in § 4.2
- [\<dimension-token\>](#typedef-dimension-token), in § 4
- [discard a mark](#token-stream-discard-a-mark), in § 5.3
- [discard a token](#token-stream-discard-a-token), in § 5.3
- [discard whitespace](#token-stream-discard-whitespace), in § 5.3
- [empty](#token-stream-empty), in § 5.3
- [environment encoding](#environment-encoding), in § 3.2
- [EOF code point](#eof-code-point), in § 4.2
- [\<eof-token\>](#typedef-eof-token), in § 5.3
- [escaping](#escape-codepoint), in § 2.1
- [filter code points](#css-filter-code-points), in § 3.3
- [filtered code points](#css-filter-code-points), in § 3.3
- [function](#function), in § 5.2
- [\<function-token\>](#typedef-function-token), in § 4
- [\<hash-token\>](#typedef-hash-token), in § 4
- [hex digit](#hex-digit), in § 4.2
- [ident code point](#ident-code-point), in § 4.2
- [ident sequence](#ident-sequence), in § 4.2
- [ident-start code point](#ident-start-code-point), in § 4.2
- [\<ident-token\>](#typedef-ident-token), in § 4
- [ignored](#css-ignored), in § 2.2
- [index](#token-stream-index), in § 5.3
- [input stream](#input-stream), in § 3.3
- [\<integer\>](#typedef-integer), in § 6.2
- [invalid](#css-invalid), in § 2.2
- [invalid rule error](#invalid-rule-error), in § 5.5.3
- [letter](#letter), in § 4.2
- [lowercase letter](#lowercase-letter), in § 4.2
- [mark](#token-stream-mark), in § 5.3
- [marked indexes](#token-stream-marked-indexes), in § 5.3
- [maximum allowed code point](#maximum-allowed-code-point), in § 4.2
- [name-start code point](#ident-start-code-point), in § 4.2
- [\<ndashdigit-dimension\>](#typedef-ndashdigit-dimension), in § 6.2
- [\<ndashdigit-ident\>](#typedef-ndashdigit-ident), in § 6.2
- [\<ndash-dimension\>](#typedef-ndash-dimension), in § 6.2
- [\<n-dimension\>](#typedef-n-dimension), in § 6.2
- [newline](#newline), in § 4.2
- [next input code point](#next-input-code-point), in § 4.2
- [next token](#token-stream-next-token), in § 5.3
- [non-ASCII ident code point](#non-ascii-ident-code-point), in § 4.2
- [non-printable code point](#non-printable-code-point), in § 4.2
- [normalize](#normalize-into-a-token-stream), in § 5.4
- [normalize into a token stream](#normalize-into-a-token-stream), in § 5.4
- [\<number-token\>](#typedef-number-token), in § 4
- [parse](#css-parse-something-according-to-a-css-grammar), in § 5.4.1
- [parse a block’s contents](#parse-a-blocks-contents), in § 5.4.5
- [parse a comma-separated list according to a CSS grammar](#css-parse-a-comma-separated-list-according-to-a-css-grammar), in § 5.4.2
- [parse a comma-separated list of component values](#parse-a-comma-separated-list-of-component-values), in § 5.4.10
- [parse a component value](#parse-a-component-value), in § 5.4.8
- [parse a CSS stylesheet](#parse-a-css-stylesheet), in § 8
- [parse a declaration](#parse-a-declaration), in § 5.4.7
- [parse a list](#css-parse-a-comma-separated-list-according-to-a-css-grammar), in § 5.4.2
- [parse a list of component values](#parse-a-list-of-component-values), in § 5.4.9
- [parse a rule](#parse-a-rule), in § 5.4.6
- [parse a stylesheet](#parse-a-stylesheet), in § 5.4.3
- [parse a stylesheet’s contents](#parse-a-stylesheets-contents), in § 5.4.4
- [parse error](#parse-error), in § 3
- [parse something according to a CSS grammar](#css-parse-something-according-to-a-css-grammar), in § 5.4.1
- [\<percentage-token\>](#typedef-percentage-token), in § 4
- [preserved tokens](#preserved-tokens), in § 5.2
- [process](#token-stream-process), in § 5.3
- [property declarations](#css-property-declarations), in § 5.2
- [qualified rule](#qualified-rule), in § 5.2
- [\<qualified-rule-list\>](#typedef-qualified-rule-list), in § 7.1
- [reconsume the current input code point](#reconsume-the-current-input-code-point), in § 4.2
- [restore a mark](#token-stream-restore-a-mark), in § 5.3
- [rule](#css-rule), in § 5.2
- [\<rule-list\>](#typedef-rule-list), in § 7.1
- [\<semicolon-token\>](#typedef-semicolon-token), in § 4
- [serialize an \<a-n-plus-b\> value](#serialize-an-a-n-plus-b-value), in § 9.1
- [\<signed-integer\>](#typedef-signed-integer), in § 6.2
- [\<signless-integer\>](#typedef-signless-integer), in § 6.2
- [simple block](#simple-block), in § 5.2
- [starts with an ident sequence](#check-if-three-code-points-would-start-an-ident-sequence), in § 4.3.9
- [starts with a number](#check-if-three-code-points-would-start-a-number), in § 4.3.10
- [starts with a valid escape](#check-if-two-code-points-are-a-valid-escape), in § 4.3.8
- [start with an ident sequence](#check-if-three-code-points-would-start-an-ident-sequence), in § 4.3.9
- [start with a number](#check-if-three-code-points-would-start-a-number), in § 4.3.10
- [statement at-rule](#statement-at-rule), in § 8.2
- [\<string-token\>](#typedef-string-token), in § 4
- [style rule](#style-rule), in § 8.1
- [stylesheet](#css-stylesheet), in § 5.2
- [\<(-token\>](#tokendef-open-paren), in § 4
- [\<)-token\>](#tokendef-close-paren), in § 4
- [\<\[-token\>](#tokendef-open-square), in § 4
- [\<\]-token\>](#tokendef-close-square), in § 4
- [\<{-token\>](#tokendef-open-curly), in § 4
- [\<}-token\>](#tokendef-close-curly), in § 4
- [tokenization](#css-tokenize), in § 4
- [tokenize](#css-tokenize), in § 4
- [tokens](#token-stream-tokens), in § 5.3
- [token stream](#css-token-stream), in § 5.3
- [\<unicode-range-token\>](#typedef-unicode-range-token), in § 4
- [uppercase letter](#uppercase-letter), in § 4.2
- [\<url-token\>](#typedef-url-token), in § 4
- [whitespace](#whitespace), in § 4.2
- [\<whitespace-token\>](#typedef-whitespace-token), in § 4
- [would start an ident sequence](#check-if-three-code-points-would-start-an-ident-sequence), in § 4.3.9
- [would start a number](#check-if-three-code-points-would-start-a-number), in § 4.3.10
- [would start a unicode-range](#check-if-three-code-points-would-start-a-unicode-range), in § 4.3.11

### <a id="index-defined-elsewhere"></a>Terms defined by reference[](#index-defined-elsewhere)

- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="3eef835e"></a>&#64;import
  - <a id="81a52293"></a>property
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="a976c737"></a>blue
  - <a id="bcdf9b19"></a>color
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="d04b6986"></a>\<color\>
- \[CSS-COUNTER-STYLES-3\] defines the following terms:
  - <a id="7d714b13"></a>counter style
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="11a985a9"></a>font
  - <a id="e1ba292f"></a>unicode-range
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="b24ce65e"></a>&#64;font-face
- \[CSS-NESTING-1\] defines the following terms:
  - <a id="9e5e1294"></a>CSSNestedDeclarations
  - <a id="4dc96cc8"></a>nested declarations rule
- \[CSS-PAGE-3\] defines the following terms:
  - <a id="31e95f75"></a>:left
  - <a id="2c5a261b"></a>&#64;page
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="a015dd4d"></a>\<style-block\>
  - <a id="6524d004"></a>\<urange\>
  - <a id="38314b9f"></a>non-ASCII code point
  - <a id="69aefbfb"></a>parse a list of declarations
- \[CSS-TEXT-DECOR-3\] defines the following terms:
  - <a id="e9aa0c44"></a>text-decoration
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="840ef6c8"></a>underline
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="26ed428f"></a>translateX()
- \[CSS-TYPED-OM-1\] defines the following terms:
  - <a id="5f5ab972"></a>custom property name string
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="1d798932"></a>\<string\>
  - <a id="d4441b24"></a>?
  - <a id="3fa441fa"></a>url()
  - <a id="4eb9d37e"></a>\|
- \[CSS-VALUES-5\] defines the following terms:
  - <a id="a20cee51"></a>attr()
- \[CSS-VARIABLES\] defines the following terms:
  - <a id="5550667d"></a>custom property
  - <a id="3beec8c9"></a>var()
- \[CSS3-ANIMATIONS\] defines the following terms:
  - <a id="a73d7d2b"></a>\<keyframe-selector\>
  - <a id="7177b17d"></a>&#64;keyframes
  - <a id="79dcf364"></a>animation-timing-function
- \[CSS3-CONDITIONAL\] defines the following terms:
  - <a id="4397147f"></a>&#64;media
  - <a id="a5d6c9d2"></a>&#64;supports
  - <a id="f1a41224"></a>conditional group rule
- \[CSSOM\] defines the following terms:
  - <a id="2d22493a"></a>CSSStyleDeclaration
  - <a id="58164e11"></a>cssText
  - <a id="ea4973c0"></a>insertRule(rule)
  - <a id="d321b0e7"></a>location
  - <a id="eb28b1dd"></a>replace(text)
- \[ENCODING\] defines the following terms:
  - <a id="58b7ba18"></a>decode
  - <a id="4fdfba4e"></a>get an encoding
- \[HTML\] defines the following terms:
  - <a id="68bf3d31"></a>a
  - <a id="b80d76a2"></a>p
  - <a id="0cf3964f"></a>script
  - <a id="1d1029cb"></a>sizes
  - <a id="ba920583"></a>style
  - <a id="15c9cf79"></a>valid custom element name
- \[INFRA\] defines the following terms:
  - <a id="7f9469b5"></a>ASCII case-insensitive
  - <a id="915aff5e"></a>code point
  - <a id="16d07e10"></a>for each
  - <a id="c88f3887"></a>item
  - <a id="649608b9"></a>list
  - <a id="fca74142"></a>pop
  - <a id="99c988d6"></a>remove
  - <a id="ecf251b4"></a>scalar value
  - <a id="ceacaa1c"></a>stack
  - <a id="0698d556"></a>string
  - <a id="984221ca"></a>struct
  - <a id="a3fb968a"></a>surrogate
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="b9b36bfc"></a>\<general-enclosed\>
  - <a id="b5d46487"></a>media type
- \[SELECTORS-4\] defines the following terms:
  - <a id="3c7eec47"></a>+
  - <a id="3b74dc72"></a>:nth-child()
  - <a id="1fd14124"></a>\<selector-list\>
  - <a id="dc60071a"></a>selector list
- \[URL\] defines the following terms:
  - <a id="dcffbccd"></a>URL

## <a id="references"></a>References[](#references)

### <a id="normative"></a>Normative References[](#normative)

<a id="biblio-css-cascade-3"></a><strong>\[CSS-CASCADE-3\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://www.w3.org/TR/css-cascade-3/). 11 February 2021. REC. URL: <https://www.w3.org/TR/css-cascade-3/>

<a id="biblio-css-cascade-5"></a><strong>\[CSS-CASCADE-5\]</strong>

Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: <https://www.w3.org/TR/css-cascade-5/>

<a id="biblio-css-counter-styles-3"></a><strong>\[CSS-COUNTER-STYLES-3\]</strong>

Tab Atkins Jr.. [CSS Counter Styles Level 3](https://www.w3.org/TR/css-counter-styles-3/). 27 July 2021. CR. URL: <https://www.w3.org/TR/css-counter-styles-3/>

<a id="biblio-css-fonts-4"></a><strong>\[CSS-FONTS-4\]</strong>

Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 13 September 2026. WD. URL: <https://www.w3.org/TR/css-fonts-4/>

<a id="biblio-css-nesting-1"></a><strong>\[CSS-NESTING-1\]</strong>

Tab Atkins Jr.. [CSS Nesting Module Level 1](https://www.w3.org/TR/css-nesting-1/). 22 January 2026. WD. URL: <https://www.w3.org/TR/css-nesting-1/>

<a id="biblio-css-page-3"></a><strong>\[CSS-PAGE-3\]</strong>

Elika Etemad. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 14 September 2023. WD. URL: <https://www.w3.org/TR/css-page-3/>

<a id="biblio-css-syntax-3"></a><strong>\[CSS-SYNTAX-3\]</strong>

Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CRD. URL: <https://www.w3.org/TR/css-syntax-3/>

<a id="biblio-css-typed-om-1"></a><strong>\[CSS-TYPED-OM-1\]</strong>

Tab Atkins Jr.; François Remy. [CSS Typed OM Level 1](https://www.w3.org/TR/css-typed-om-1/). 21 March 2024. WD. URL: <https://www.w3.org/TR/css-typed-om-1/>

<a id="biblio-css-values-4"></a><strong>\[CSS-VALUES-4\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: <https://www.w3.org/TR/css-values-4/>

<a id="biblio-css-variables"></a><strong>\[CSS-VARIABLES\]</strong>

Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 16 June 2022. CR. URL: <https://www.w3.org/TR/css-variables-1/>

<a id="biblio-css3-conditional"></a><strong>\[CSS3-CONDITIONAL\]</strong>

Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 15 August 2024. CRD. URL: <https://www.w3.org/TR/css-conditional-3/>

<a id="biblio-cssom"></a><strong>\[CSSOM\]</strong>

Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: <https://www.w3.org/TR/cssom-1/>

<a id="biblio-encoding"></a><strong>\[ENCODING\]</strong>

Anne van Kesteren. [Encoding Standard](https://encoding.spec.whatwg.org/). Living Standard. URL: <https://encoding.spec.whatwg.org/>

<a id="biblio-html"></a><strong>\[HTML\]</strong>

Ian Hickson. [HTML](https://whatwg.org/html). Living Standard. URL: [http://whatwg.org/html](https://whatwg.org/html)

<a id="biblio-infra"></a><strong>\[INFRA\]</strong>

Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: <https://infra.spec.whatwg.org/>

<a id="biblio-rfc2119"></a><strong>\[RFC2119\]</strong>

S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: <https://datatracker.ietf.org/doc/html/rfc2119>

<a id="biblio-selectors-4"></a><strong>\[SELECTORS-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 22 January 2026. WD. URL: <https://www.w3.org/TR/selectors-4/>

<a id="biblio-url"></a><strong>\[URL\]</strong>

Anne van Kesteren. [URL Standard](https://url.spec.whatwg.org/). Living Standard. URL: <https://url.spec.whatwg.org/>

### <a id="informative"></a>Non-Normative References[](#informative)

<a id="biblio-css-color-4"></a><strong>\[CSS-COLOR-4\]</strong>

Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 26 September 2026. CRD. URL: <https://www.w3.org/TR/css-color-4/>

<a id="biblio-css-color-5"></a><strong>\[CSS-COLOR-5\]</strong>

Chris Lilley; Una Kravets; Lea Verou. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 13 September 2026. WD. URL: <https://www.w3.org/TR/css-color-5/>

<a id="biblio-css-fonts-5"></a><strong>\[CSS-FONTS-5\]</strong>

Chris Lilley. [CSS Fonts Module Level 5](https://www.w3.org/TR/css-fonts-5/). 13 September 2026. WD. URL: <https://www.w3.org/TR/css-fonts-5/>

<a id="biblio-css-namespaces-3"></a><strong>\[CSS-NAMESPACES-3\]</strong>

Elika Etemad. [CSS Namespaces Module Level 3](https://www.w3.org/TR/css-namespaces-3/). 20 March 2014. REC. URL: <https://www.w3.org/TR/css-namespaces-3/>

<a id="biblio-css-text-decor-3"></a><strong>\[CSS-TEXT-DECOR-3\]</strong>

Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 3](https://www.w3.org/TR/css-text-decor-3/). 5 May 2022. CRD. URL: <https://www.w3.org/TR/css-text-decor-3/>

<a id="biblio-css-text-decor-4"></a><strong>\[CSS-TEXT-DECOR-4\]</strong>

Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 4 May 2022. WD. URL: <https://www.w3.org/TR/css-text-decor-4/>

<a id="biblio-css-transforms-1"></a><strong>\[CSS-TRANSFORMS-1\]</strong>

Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: <https://www.w3.org/TR/css-transforms-1/>

<a id="biblio-css-values-5"></a><strong>\[CSS-VALUES-5\]</strong>

Tab Atkins Jr.; Elika Etemad; Miriam Suzanne. [CSS Values and Units Module Level 5](https://www.w3.org/TR/css-values-5/). 11 November 2024. WD. URL: <https://www.w3.org/TR/css-values-5/>

<a id="biblio-css2"></a><strong>\[CSS2\]</strong>

Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: <https://www.w3.org/TR/CSS2/>

<a id="biblio-css3-animations"></a><strong>\[CSS3-ANIMATIONS\]</strong>

David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: <https://www.w3.org/TR/css-animations-1/>

<a id="biblio-mediaq"></a><strong>\[MEDIAQ\]</strong>

Tab Atkins Jr.; Florian Rivoal. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 19 February 2026. CRD. URL: <https://www.w3.org/TR/mediaqueries-4/>

<a id="biblio-mediaqueries-5"></a><strong>\[MEDIAQUERIES-5\]</strong>

Tab Atkins Jr.; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 19 February 2026. WD. URL: <https://www.w3.org/TR/mediaqueries-5/>

<a id="biblio-select"></a><strong>\[SELECT\]</strong>

Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: <https://www.w3.org/TR/selectors-3/>
