Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Object Model (CSSOM) Module Level 1](https://drafts.csswg.org/cssom/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply. The capture’s full original notice and links remain below.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md#cssom-planning-reference-refresh--2026-10-09).

# Source provenance

Title: CSS Object Model (CSSOM) Module Level 1

Source snapshot: https://drafts.csswg.org/cssom/

Retrieved: 2026-10-09. The captured page states Editor’s Draft, 31 August 2026. An undated editor URL can change; the retrieval date and exact byte hash identify this captured HTML.

Captured HTML SHA-256: d42e145ec395103f592001e5683cbeb63084ea530b76ec2923ecbcf80c7e74f9

Captured HTML revision metadata: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Representation notes:

- Full format conversion of the captured HTML body, including status, metadata, bibliography, indexes, examples, test references, and legal notice. Script and style elements are omitted and were not executed.
- All source body IDs are retained, including those inside preformatted blocks, which are relocated immediately before those blocks. Fragment-only links stay local only when their exact captured target exists; other links remain upstream.
- All 3 tables have readable Markdown layouts. All tables have ordinary row/column layouts. Synthetic Field/Definition or Column N headings are non-normative. Source row headers remain bold; native HTML header/accessibility semantics are not expressible in GFM.
- Preformatted examples retain literal text. Single-line table examples use semantic inline code. Compiler-generated hyperlinks inside fenced code blocks are omitted while their IDs and visible code text are retained; other source links are preserved. Small semantic code, variable, emphasis, subscript, and superscript HTML remains where Markdown notation would alter text.
- Conversion uses Pandoc 3.1.11.1 with focused Python/lxml preparation and a semantic Lua filter; the parsed GFM output is checked against the captured HTML. The [conversion report](CONVERSION-REPORT.md#cssom-planning-reference-refresh--2026-10-09) records provenance and check boundaries.
- This full edition is the selected planning reference for the new CSSOM work. The [source-edition mapping](SOURCE-EDITIONS.md#cssom-planning-sources) distinguishes these planning selections from historical editions consumed by completed CSS work.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Object Model (CSSOM) Module Level 1

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 31 August 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/cssom/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/cssom-1/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2021/WD-cssom-1-20210826/>

<https://www.w3.org/TR/2016/WD-cssom-1-20160317/>

<https://www.w3.org/TR/2013/WD-cssom-20131205/>

<https://www.w3.org/TR/2011/WD-cssom-20110712/>

<https://www.w3.org/TR/2000/REC-DOM-Level-2-Style-20001113/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/cssom-1)

[Inline In Spec](#issues-index)

<strong>Editor:</strong>

[Emilio Cobos Álvarez](mailto:emilio@mozilla.com) (Mozilla)

<strong>Former Editors:</strong>

[Daniel Glazman](mailto:daniel.glazman@disruptive-innovations.com) ([Disruptive Innovations](http://disruptive-innovations.com/))

[Simon Pieters](mailto:simonp@opera.com) ([Opera Software AS](http://www.opera.com))

[Glenn Adams](http://www.w3.org/wiki/User:Gadams) ([Cox Communications, Inc.](http://www.cox.com)) [glenn.adams&#64;cos.com](mailto:glenn.adams@cos.com)

[Anne van Kesteren](https://annevankesteren.nl/) ([Opera Software ASA](http://www.opera.com)) [annevk&#64;annevk.nl](mailto:annevk@annevk.nl)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/cssom-1/Overview.bs)

<strong>Legacy issues list:</strong>

[Bugzilla](https://www.w3.org/Bugs/Public/buglist.cgi?product=CSS&component=CSSOM&resolution=---)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/cssom/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

CSSOM defines APIs (including generic parsing and serialization rules) for Media Queries, Selectors, and of course CSS itself.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “cssom” in the title, like this: “\[cssom\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcssom%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](#introduction)
2.  [2 Terminology](#terminology)
    1.  [2.1 Common Serializing Idioms](#common-serializing-idioms)
3.  [3 CSSOMString](#cssomstring-type)
4.  [4 Media Queries](#media-queries)
    1.  [4.1 Parsing Media Queries](#parsing-media-queries)
    2.  [4.2 Serializing Media Queries](#serializing-media-queries)
        1.  [4.2.1 Serializing Media Feature Values](#serializing-media-feature-values)
    3.  [4.3 Comparing Media Queries](#comparing-media-queries)
    4.  [4.4 The <code>MediaList</code> Interface](#the-medialist-interface)
5.  [5 Selectors](#selectors)
    1.  [5.1 Parsing Selectors](#parsing-selectors)
    2.  [5.2 Serializing Selectors](#serializing-selectors)
6.  [6 CSS](#css-object-model)
    1.  [6.1 CSS Style Sheets](#css-style-sheets)
        1.  [6.1.1 The <code>StyleSheet</code> Interface](#the-stylesheet-interface)
        2.  [6.1.2 The <code>CSSStyleSheet</code> Interface](#the-cssstylesheet-interface)
            1.  [6.1.2.1 Deprecated CSSStyleSheet members](#legacy-css-style-sheet-members)
    2.  [6.2 CSS Style Sheet Collections](#css-style-sheet-collections)
        1.  [6.2.1 The HTTP Default-Style Header](#the-http-default-style-header)
        2.  [6.2.2 The <code>StyleSheetList</code> Interface](#the-stylesheetlist-interface)
        3.  [6.2.3 Extensions to the <code>DocumentOrShadowRoot</code> Interface Mixin](#extensions-to-the-document-or-shadow-root-interface)
    3.  [6.3 Style Sheet Association](#style-sheet-association)
        1.  [6.3.1 Fetching CSS style sheets](#fetching-css-style-sheets)
        2.  [6.3.2 The <code>LinkStyle</code> Interface](#the-linkstyle-interface)
        3.  [6.3.3 Requirements on specifications](#requirements-on-specifications)
        4.  [6.3.4 Requirements on user agents Implementing the xml-stylesheet processing instruction](#requirements-on-user-agents-implementing-the-xml-stylesheet-processing-instruction)
        5.  [6.3.5 Requirements on user agents Implementing the HTTP Link Header](#requirements-on-user-agents-implementing-the-http-link-header)
    4.  [6.4 CSS Rules](#css-rules)
        1.  [6.4.1 The <code>CSSRuleList</code> Interface](#the-cssrulelist-interface)
        2.  [6.4.2 The <code>CSSRule</code> Interface](#the-cssrule-interface)
        3.  [6.4.3 The <code>CSSStyleRule</code> Interface](#the-cssstylerule-interface)
        4.  [6.4.4 The <code>CSSImportRule</code> Interface](#the-cssimportrule-interface)
        5.  [6.4.5 The <code>CSSGroupingRule</code> Interface](#the-cssgroupingrule-interface)
        6.  [6.4.6 The <code>CSSMediaRule</code> Interface](#the-cssmediarule-interface)
        7.  [6.4.7 The <code>CSSPageRule</code> Interface](#the-csspagerule-interface)
        8.  [6.4.8 The <code>CSSMarginRule</code> Interface](#the-cssmarginrule-interface)
        9.  [6.4.9 The <code>CSSNamespaceRule</code> Interface](#the-cssnamespacerule-interface)
    5.  [6.5 CSS Declarations](#css-declarations)
    6.  [6.6 CSS Declaration Blocks](#css-declaration-blocks)
        1.  [6.6.1 The <code>CSSStyleDeclaration</code> Interface](#the-cssstyledeclaration-interface)
    7.  [6.7 CSS Values](#css-values)
        1.  [6.7.1 Parsing CSS Values](#parsing-css-values)
        2.  [6.7.2 Serializing CSS Values](#serializing-css-values)
            1.  [6.7.2.1 Examples](#serializing-css-values-examples)
7.  [7 DOM Access to CSS Declaration Blocks](#dom-access-to-css-declaration-blocks)
    1.  [7.1 The <code>ElementCSSInlineStyle</code> Mixin](#the-elementcssinlinestyle-mixin)
    2.  [7.2 Extensions to the <code>Window</code> Interface](#extensions-to-the-window-interface)
8.  [8 Utility APIs](#utility-apis)
    1.  [8.1 The <code>CSS.escape()</code> Method](#the-css.escape()-method)
9.  [9 Resolved Values](#resolved-values)
10. [10 IANA Considerations](#iana-considerations)
    1.  [10.1 Default-Style](#default-style)
11. [11 Change History](#change-history)
    1.  [11.1 Changes From 17 March 2016](#changes-from-17-march-2016)
    2.  [11.2 Changes From 5 December 2013](#changes-from-5-december-2013)
    3.  [11.3 Changes From 12 July 2011 To 5 December 2013](#changes-from-12-july-2011-to-5-december-2013)
12. [12 Security Considerations](#sec)
13. [13 Privacy Considerations](#priv)
14. [14 Acknowledgments](#acknowledgments)
15. [ Conformance](#w3c-conformance)
    1.  [ Document conventions](#w3c-conventions)
    2.  [ Conformance classes](#w3c-conformance-classes)
    3.  [ Partial implementations](#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](#w3c-testing)
16. [ Index](#index)
    1.  [ Terms defined by this specification](#index-defined-here)
    2.  [ Terms defined by reference](#index-defined-elsewhere)
17. [ References](#references)
    1.  [ Normative References](#normative)
    2.  [ Non-Normative References](#informative)
18. [ IDL Index](#idl-index)
19. [ Issues Index](#issues-index)

## <a id="introduction"></a>1. Introduction[](#introduction)

This document formally specifies the core features of the CSS Object Model (CSSOM). Other documents in the CSSOM family of specifications as well as other CSS related specifications define extensions to these core features.

The core features of the CSSOM are oriented towards providing basic capabilities to author-defined scripts to permit access to and manipulation of style related state information and processes.

The features defined below are fundamentally based on prior specifications of the W3C DOM Working Group, primarily [\[DOM\]](#biblio-dom). The purposes of the present document are (1) to improve on that prior work by providing more technical specificity (so as to improve testability and interoperability), (2) to deprecate or remove certain less-widely implemented features no longer considered to be essential in this context, and (3) to newly specify certain extensions that have been or expected to be widely implemented.

Tests

Basic CSSOM tests

- [idlharness.html](https://wpt.fyi/results/css/cssom/idlharness.html) [(live test)](http://wpt.live/css/cssom/idlharness.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/idlharness.html)
- [invalid-pseudo-elements.html](https://wpt.fyi/results/css/cssom/invalid-pseudo-elements.html) [(live test)](http://wpt.live/css/cssom/invalid-pseudo-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/invalid-pseudo-elements.html)
- [historical.html](https://wpt.fyi/results/css/cssom/historical.html) [(live test)](http://wpt.live/css/cssom/historical.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/historical.html)
- [stylesheet-replacedata-dynamic.html](https://wpt.fyi/results/css/cssom/stylesheet-replacedata-dynamic.html) [(live test)](http://wpt.live/css/cssom/stylesheet-replacedata-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/stylesheet-replacedata-dynamic.html)
- [xml-stylesheet-pi-in-doctype.xhtml](https://wpt.fyi/results/css/cssom/xml-stylesheet-pi-in-doctype.xhtml) [(live test)](http://wpt.live/css/cssom/xml-stylesheet-pi-in-doctype.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/xml-stylesheet-pi-in-doctype.xhtml)

------------------------------------------------------------------------

## <a id="terminology"></a>2. Terminology[](#terminology)

This specification employs certain terminology from the following documents: DOM, HTML, CSS Syntax, Encoding, URL, Fetch, Associating Style Sheets with XML documents and XML. [\[DOM\]](#biblio-dom) [\[HTML\]](#biblio-html) [\[CSS3SYN\]](#biblio-css3syn) [\[ENCODING\]](#biblio-encoding) [\[URL\]](#biblio-url) [\[FETCH\]](#biblio-fetch) [\[XML-STYLESHEET\]](#biblio-xml-stylesheet) [\[XML\]](#biblio-xml)

When this specification talks about object <code><var>A</var></code> where <code><var>A</var></code> is actually an interface, it generally means an object implementing interface <code><var>A</var></code>.

The terms <a id="set"></a><strong>set</strong> and <a id="unset"></a><strong>unset</strong> to refer to the true and false values of binary flags or variables, respectively. These terms are also used as verbs in which case they refer to mutating some value to make it true or false, respectively.

The term <a id="supported-styling-language"></a><strong>supported styling language</strong> refers to CSS.

Note: If another styling language becomes supported in user agents, this specification is expected to be updated as necessary.

The term <a id="supported-css-property"></a><strong>supported CSS property</strong> refers to a CSS property that the user agent implements, including any vendor-prefixed properties, but excluding <a id="ref-for-custom-property"></a>[custom properties](https://drafts.csswg.org/css-variables-1/#custom-property). A <a id="ref-for-supported-css-property"></a>[supported CSS property](#supported-css-property) must be in its lowercase form for the purpose of comparisons in this specification.

In this specification the <a id="ref-for-sel-before"></a>[::before](https://drafts.csswg.org/selectors-3/#sel-before) and <a id="ref-for-sel-after"></a>[::after](https://drafts.csswg.org/selectors-3/#sel-after) pseudo-elements are assumed to exist for all elements even if no box is generated for them.

When a method or an attribute is said to call another method or attribute, the user agent must invoke its internal API for that attribute or method so that e.g. the author can’t change the behavior by overriding attributes or methods with custom properties or functions in ECMAScript.

Unless otherwise stated, string comparisons are done in a <a id="ref-for-dfn-case-sensitive"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) manner.

### <a id="common-serializing-idioms"></a>2.1. Common Serializing Idioms[](#common-serializing-idioms)

To <a id="escape-a-character"></a><strong>escape a character</strong> means to create a string of "<code>&#92;</code>" (U+005C), followed by the character.

To <a id="escape-a-character-as-code-point"></a><strong>escape a character as code point</strong> means to create a string of "<code>&#92;</code>" (U+005C), followed by the Unicode code point as the smallest possible number of hexadecimal digits in the range 0-9 a-f (U+0030 to U+0039 and U+0061 to U+0066) to represent the code point in base 16, followed by a single SPACE (U+0020).

To <a id="serialize-an-identifier"></a><strong>serialize an identifier</strong> means to create a string represented by the concatenation of, for each character of the identifier:

- If the character is NULL (U+0000), then the REPLACEMENT CHARACTER (U+FFFD).
- If the character is in the range \[\1-\1f\] (U+0001 to U+001F) or is U+007F, then the character <a id="ref-for-escape-a-character-as-code-point"></a>[escaped as code point](#escape-a-character-as-code-point).
- If the character is the first character and is in the range \[0-9\] (U+0030 to U+0039), then the character <a id="ref-for-escape-a-character-as-code-point①"></a>[escaped as code point](#escape-a-character-as-code-point).
- If the character is the second character and is in the range \[0-9\] (U+0030 to U+0039) and the first character is a "<code>-</code>" (U+002D), then the character <a id="ref-for-escape-a-character-as-code-point②"></a>[escaped as code point](#escape-a-character-as-code-point).
- If the character is the first character and is a "<code>-</code>" (U+002D), and there is no second character, then the <a id="ref-for-escape-a-character"></a>[escaped](#escape-a-character) character.
- If the character is not handled by one of the above rules and is greater than or equal to U+0080, is "<code>-</code>" (U+002D) or "<code>&#95;</code>" (U+005F), or is in one of the ranges \[0-9\] (U+0030 to U+0039), \[A-Z\] (U+0041 to U+005A), or \[a-z\] (U+0061 to U+007A), then the character itself.
- Otherwise, the <a id="ref-for-escape-a-character①"></a>[escaped](#escape-a-character) character.

To <a id="css-serialize-a-function"></a><strong>serialize a function</strong> <var>func</var>, returning a <a id="ref-for-string"></a>[string](https://infra.spec.whatwg.org/#string):

1.  Let <var>s</var> be an empty <a id="ref-for-string①"></a>[string](https://infra.spec.whatwg.org/#string).

2.  <a id="ref-for-serialize-an-identifier"></a>[Serialize an identifier](#serialize-an-identifier) from <var>func</var>’s name, ASCII lowercased, and append the result to <var>s</var>.

3.  Append "(" (U+0028) to <var>s</var>.

4.  Serialize <var>func</var>’s contents, either as specified by the definition of <var>func</var>, or in the shortest form possible (akin to the principles captured by <a id="ref-for-serialize-a-css-value"></a>[serialize a CSS value](#serialize-a-css-value)). Append the result to <var>s</var>.

5.  Append ")" (U+0029) to <var>s</var>.

6.  Return <var>s</var>.

To <a id="serialize-a-string"></a><strong>serialize a string</strong> means to create a string represented by '"' (U+0022), followed by the result of applying the rules below to each character of the given string, followed by '"' (U+0022):

- If the character is NULL (U+0000), then the REPLACEMENT CHARACTER (U+FFFD).
- If the character is in the range \[\1-\1f\] (U+0001 to U+001F) or is U+007F, the character <a id="ref-for-escape-a-character-as-code-point③"></a>[escaped as code point](#escape-a-character-as-code-point).
- If the character is '"' (U+0022) or "<code>&#92;</code>" (U+005C), the <a id="ref-for-escape-a-character②"></a>[escaped](#escape-a-character) character.
- Otherwise, the character itself.

Note: "<code>'</code>" (U+0027) is not escaped because strings are always serialized with '"' (U+0022).

To <a id="serialize-a-url"></a><strong>serialize a URL</strong> means to create a string represented by "<code>url(</code>", followed by the <a id="ref-for-serialize-a-string"></a>[serialization](#serialize-a-string) of the URL as a string, followed by "<code>)</code>".

To <a id="serialize-a-local"></a><strong>serialize a LOCAL</strong> means to create a string represented by "<code>local(</code>", followed by the <a id="ref-for-serialize-a-string①"></a>[serialization](#serialize-a-string) of the LOCAL as a string, followed by "<code>)</code>".

To <a id="serialize-a-comma-separated-list"></a><strong>serialize a comma-separated list</strong> concatenate all items of the list in list order while separating them by "<code>, </code>", i.e., COMMA (U+002C) followed by a single SPACE (U+0020).

To <a id="serialize-a-whitespace-separated-list"></a><strong>serialize a whitespace-separated list</strong> concatenate all items of the list in list order while separating them by "<code> </code>", i.e., a single SPACE (U+0020).

Note: When serializing a list according to the above rules, extraneous whitespace is not inserted prior to the first item or subsequent to the last item. Unless otherwise specified, an empty list is serialized as the empty string.

## <a id="cssomstring-type"></a>3. CSSOMString[](#cssomstring-type)

Most strings in CSSOM interfaces use the <a id="cssomstring"></a><strong><code>CSSOMString</code></strong> type. Each implementation chooses to define it as either <code><a id="ref-for-idl-USVString"></a>[USVString](https://webidl.spec.whatwg.org/#idl-USVString)</code> or <code><a id="ref-for-idl-DOMString"></a>[DOMString](https://webidl.spec.whatwg.org/#idl-DOMString)</code>:

``` text
typedef USVString CSSOMString;
```

Or, alternatively:

``` text
typedef DOMString CSSOMString;
```

The difference is only observable from web content when

<strong>Note:</strong>

<a id="ref-for-surrogate"></a>[surrogate](https://infra.spec.whatwg.org/#surrogate) code units are involved. <code><a id="ref-for-idl-DOMString①"></a>[DOMString](https://webidl.spec.whatwg.org/#idl-DOMString)</code> would preserve them, whereas <code><a id="ref-for-idl-USVString①"></a>[USVString](https://webidl.spec.whatwg.org/#idl-USVString)</code> would replace them with U+FFFD REPLACEMENT CHARACTER.

This choice effectively allows implementations to do this replacement, but does not require it.

Using <code><a id="ref-for-idl-USVString②"></a>[USVString](https://webidl.spec.whatwg.org/#idl-USVString)</code> enables an implementation to use UTF-8 internally to represent strings in memory. Since well-formed UTF-8 specifically disallows <a id="ref-for-surrogate①"></a>[surrogate](https://infra.spec.whatwg.org/#surrogate) code points, it effectively requires this replacement.

On the other hand, implementations that internally represent strings as 16-bit <a id="ref-for-code-unit"></a>[code units](https://infra.spec.whatwg.org/#code-unit) might prefer to avoid the cost of doing this replacement.

## <a id="media-queries"></a>4. Media Queries[](#media-queries)

<a id="ref-for-media-query"></a>[Media queries](https://drafts.csswg.org/mediaqueries-5/#media-query) are defined by [\[MEDIAQUERIES\]](#biblio-mediaqueries). This section defines various concepts around <a id="ref-for-media-query①"></a>media queries, including their API and serialization form.

### <a id="parsing-media-queries"></a>4.1. Parsing Media Queries[](#parsing-media-queries)

To <a id="parse-a-media-query-list"></a><strong>parse a media query list</strong> for a given string <var>s</var> into a <a id="ref-for-media-query-list"></a>[media query list](https://drafts.csswg.org/mediaqueries-5/#media-query-list) is defined in the Media Queries specification. Return the list of media queries that the algorithm defined there gives.

Note: A media query that ends up being "ignored" will turn into "<code>not all</code>".

To <a id="parse-a-media-query"></a><strong>parse a media query</strong> for a given string <var>s</var> means to follow the <a id="ref-for-parse-a-media-query-list"></a>[parse a media query list](#parse-a-media-query-list) steps and return null if more than one media query is returned or a media query if a single media query is returned.

Note: Again, a media query that ends up being "ignored" will turn into "<code>not all</code>".

### <a id="serializing-media-queries"></a>4.2. Serializing Media Queries[](#serializing-media-queries)

To <a id="serialize-a-media-query-list"></a><strong>serialize a media query list</strong> run these steps:

1.  If the <a id="ref-for-media-query-list①"></a>[media query list](https://drafts.csswg.org/mediaqueries-5/#media-query-list) is empty, then return the empty string.
2.  <a id="ref-for-serialize-a-media-query"></a>[Serialize](#serialize-a-media-query) each media query in the list of media queries, in the same order as they appear in the <a id="ref-for-media-query-list②"></a>[media query list](https://drafts.csswg.org/mediaqueries-5/#media-query-list), and then <a id="ref-for-serialize-a-comma-separated-list"></a>[serialize](#serialize-a-comma-separated-list) the list.

To <a id="serialize-a-media-query"></a><strong>serialize a media query</strong> let <var>s</var> be the empty string, run the steps below:

1.  If the <a id="ref-for-media-query②"></a>[media query](https://drafts.csswg.org/mediaqueries-5/#media-query) is negated append "<code>not</code>", followed by a single SPACE (U+0020), to <var>s</var>.
2.  Let <var>type</var> be the <a id="ref-for-serialize-an-identifier①"></a>[serialization as an identifier](#serialize-an-identifier) of the <a id="ref-for-media-type"></a>[media type](https://drafts.csswg.org/mediaqueries-5/#media-type) of the <a id="ref-for-media-query③"></a>[media query](https://drafts.csswg.org/mediaqueries-5/#media-query), <a id="ref-for-ascii-lowercase"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).
3.  If the <a id="ref-for-media-query④"></a>[media query](https://drafts.csswg.org/mediaqueries-5/#media-query) does not contain <a id="ref-for-media-feature"></a>[media features](https://drafts.csswg.org/mediaqueries-5/#media-feature) append <var>type</var>, to <var>s</var>, then return <var>s</var>.
4.  If <var>type</var> is not "<code>all</code>" or if the media query is negated append <var>type</var>, followed by a single SPACE (U+0020), followed by "<code>and</code>", followed by a single SPACE (U+0020), to <var>s</var>.
5.  Then, for each <a id="ref-for-media-feature①"></a>[media feature](https://drafts.csswg.org/mediaqueries-5/#media-feature):
    1.  Append a "<code>(</code>" (U+0028), followed by the <a id="ref-for-media-feature②"></a>[media feature](https://drafts.csswg.org/mediaqueries-5/#media-feature) name, <a id="ref-for-ascii-lowercase①"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase), to <var>s</var>.
    2.  If a value is given append a "<code>&#58;</code>" (U+003A), followed by a single SPACE (U+0020), followed by the <a id="ref-for-serialize-a-media-feature-value"></a>[serialized media feature value](#serialize-a-media-feature-value), to <var>s</var>.
    3.  Append a "<code>)</code>" (U+0029) to <var>s</var>.
    4.  If this is not the last <a id="ref-for-media-feature③"></a>[media feature](https://drafts.csswg.org/mediaqueries-5/#media-feature) append a single SPACE (U+0020), followed by "<code>and</code>", followed by a single SPACE (U+0020), to <var>s</var>.
6.  Return <var>s</var>.

<a id="example-0d89c7fd"></a>

<strong>Example:</strong>

[](#example-0d89c7fd) Here are some examples of input (first column) and output (second column):

| Input                                                                    | Output                                                                     |
|--------------------------------------------------------------------------|----------------------------------------------------------------------------|
| <code>not screen and (min-WIDTH&#58;5px) AND (max-width&#58;40px)</code> | <code>not screen and (min-width&#58; 5px) and (max-width&#58; 40px)</code> |
| <code>all and (color) and (color)</code>                                 | <code>(color) and (color)</code>                                           |

Tests

- [mediaquery-sort-dedup.html](https://wpt.fyi/results/css/cssom/mediaquery-sort-dedup.html) [(live test)](http://wpt.live/css/cssom/mediaquery-sort-dedup.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/mediaquery-sort-dedup.html)

#### <a id="serializing-media-feature-values"></a>4.2.1. Serializing Media Feature Values[](#serializing-media-feature-values)

<a id="issue-41128eee"></a>

<strong>Issue:</strong>

[](#issue-41128eee) This should probably be done in terms of mapping it to serializing CSS values as media features are defined in terms of CSS values after all.

To <a id="serialize-a-media-feature-value"></a><strong>serialize a media feature value</strong> named <var>v</var> locate <var>v</var> in the first column of the table below and use the serialization format described in the second column:

| Media Feature                                                                                                                                           | Serialization                                                                                                 |
|---------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------|
| <a id="ref-for-descdef-media-width"></a>[width](https://drafts.csswg.org/mediaqueries-5/#descdef-media-width)                                           | ...                                                                                                           |
| <a id="ref-for-descdef-media-height"></a>[height](https://drafts.csswg.org/mediaqueries-5/#descdef-media-height)                                        | ...                                                                                                           |
| <a id="ref-for-descdef-media-device-width"></a>[device-width](https://drafts.csswg.org/mediaqueries-5/#descdef-media-device-width)                      | ...                                                                                                           |
| <a id="ref-for-descdef-media-device-height"></a>[device-height](https://drafts.csswg.org/mediaqueries-5/#descdef-media-device-height)                   | ...                                                                                                           |
| <a id="ref-for-descdef-media-orientation"></a>[orientation](https://drafts.csswg.org/mediaqueries-5/#descdef-media-orientation)                         | If the value is portrait: "<code>portrait</code>". If the value is landscape: "<code>landscape</code>".       |
| <a id="ref-for-descdef-media-aspect-ratio"></a>[aspect-ratio](https://drafts.csswg.org/mediaqueries-5/#descdef-media-aspect-ratio)                      | ...                                                                                                           |
| <a id="ref-for-descdef-media-device-aspect-ratio"></a>[device-aspect-ratio](https://drafts.csswg.org/mediaqueries-5/#descdef-media-device-aspect-ratio) | ...                                                                                                           |
| <a id="ref-for-descdef-media-color"></a>[color](https://drafts.csswg.org/mediaqueries-5/#descdef-media-color)                                           | ...                                                                                                           |
| <a id="ref-for-descdef-media-color-index"></a>[color-index](https://drafts.csswg.org/mediaqueries-5/#descdef-media-color-index)                         | ...                                                                                                           |
| <a id="ref-for-descdef-media-monochrome"></a>[monochrome](https://drafts.csswg.org/mediaqueries-5/#descdef-media-monochrome)                            | ...                                                                                                           |
| <a id="ref-for-descdef-media-resolution"></a>[resolution](https://drafts.csswg.org/mediaqueries-5/#descdef-media-resolution)                            | ...                                                                                                           |
| <a id="ref-for-descdef-media-scan"></a>[scan](https://drafts.csswg.org/mediaqueries-5/#descdef-media-scan)                                              | If the value is progressive: "<code>progressive</code>". If the value is interlace: "<code>interlace</code>". |
| <a id="ref-for-descdef-media-grid"></a>[grid](https://drafts.csswg.org/mediaqueries-5/#descdef-media-grid)                                              | ...                                                                                                           |

Other specifications can extend this table and vendor-prefixed media features can have custom serialization formats as well.

### <a id="comparing-media-queries"></a>4.3. Comparing Media Queries[](#comparing-media-queries)

To <a id="compare-media-queries"></a><strong>compare media queries</strong> <var>m1</var> and <var>m2</var> means to <a id="ref-for-serialize-a-media-query①"></a>[serialize](#serialize-a-media-query) them both and return true if they are a <a id="ref-for-dfn-case-sensitive①"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match and false if they are not.

### <a id="the-medialist-interface"></a>4.4. The <code><a id="ref-for-medialist"></a>[MediaList](#medialist)</code> Interface[](#the-medialist-interface)

An object that implements the <code>MediaList</code> interface has an associated <a id="medialist-collection-of-media-queries"></a><strong>collection of media queries</strong>.

<a id="ref-for-Exposed"></a><a id="medialist"></a><a id="MediaList-stringification-behavior"></a><a id="ref-for-LegacyNullToEmptyString"></a><a id="ref-for-cssomstring"></a><a id="ref-for-dom-medialist-mediatext"></a><a id="ref-for-idl-unsigned-long"></a><a id="ref-for-dom-medialist-length"></a><a id="ref-for-cssomstring①"></a><a id="ref-for-dom-medialist-item"></a><a id="ref-for-idl-unsigned-long①"></a><a id="dom-medialist-item-index-index"></a><a id="ref-for-idl-undefined"></a><a id="ref-for-dom-medialist-appendmedium"></a><a id="ref-for-cssomstring②"></a><a id="dom-medialist-appendmedium-medium-medium"></a><a id="ref-for-idl-undefined①"></a><a id="ref-for-dom-medialist-deletemedium"></a><a id="ref-for-cssomstring③"></a><a id="dom-medialist-deletemedium-medium-medium"></a>

``` text
[Exposed=Window]
interface MediaList {
  stringifier attribute [LegacyNullToEmptyString] CSSOMString mediaText;
  readonly attribute unsigned long length;
  getter CSSOMString? item(unsigned long index);
  undefined appendMedium(CSSOMString medium);
  undefined deleteMedium(CSSOMString medium);
};
```

The object’s <a id="ref-for-dfn-supported-property-indices"></a>[supported property indices](http://heycam.github.io/webidl/#dfn-supported-property-indices) are the numbers in the range zero to one less than the number of media queries in the <a id="ref-for-medialist-collection-of-media-queries"></a>[collection of media queries](#medialist-collection-of-media-queries) represented by the collection. If there are no such media queries, then there are no <a id="ref-for-dfn-supported-property-indices①"></a>supported property indices.

Tests

- [medialist-dynamic-001.html](https://wpt.fyi/results/css/cssom/medialist-dynamic-001.html) [(live test)](http://wpt.live/css/cssom/medialist-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/medialist-dynamic-001.html)
- [medialist-interfaces-001.html](https://wpt.fyi/results/css/cssom/medialist-interfaces-001.html) [(live test)](http://wpt.live/css/cssom/medialist-interfaces-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/medialist-interfaces-001.html)
- [medialist-interfaces-002.html](https://wpt.fyi/results/css/cssom/medialist-interfaces-002.html) [(live test)](http://wpt.live/css/cssom/medialist-interfaces-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/medialist-interfaces-002.html)
- [medialist-interfaces-004.html](https://wpt.fyi/results/css/cssom/medialist-interfaces-004.html) [(live test)](http://wpt.live/css/cssom/medialist-interfaces-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/medialist-interfaces-004.html)
- [MediaList.html](https://wpt.fyi/results/css/cssom/MediaList.html) [(live test)](http://wpt.live/css/cssom/MediaList.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/MediaList.html)
- [MediaList2.xhtml](https://wpt.fyi/results/css/cssom/MediaList2.xhtml) [(live test)](http://wpt.live/css/cssom/MediaList2.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/MediaList2.xhtml)

To <a id="create-a-medialist-object"></a><strong>create a <code>MediaList</code> object</strong> with a string <var>text</var>, run the following steps:

1.  Create a new <code>MediaList</code> object.
2.  Set its <code><a id="ref-for-dom-medialist-mediatext①"></a>[mediaText](#dom-medialist-mediatext)</code> attribute to <var>text</var>.
3.  Return the newly created <code>MediaList</code> object.

The <a id="dom-medialist-mediatext"></a><strong><code>mediaText</code></strong> attribute, on getting, must return a <a id="ref-for-serialize-a-media-query-list"></a>[serialization](#serialize-a-media-query-list) of the <a id="ref-for-medialist-collection-of-media-queries①"></a>[collection of media queries](#medialist-collection-of-media-queries). Setting the <code><a id="ref-for-dom-medialist-mediatext②"></a>[mediaText](#dom-medialist-mediatext)</code> attribute must run these steps:

1.  Empty the <a id="ref-for-medialist-collection-of-media-queries②"></a>[collection of media queries](#medialist-collection-of-media-queries).
2.  If the given value is the empty string, then return.
3.  Append all the media queries as a result of <a id="ref-for-parse-a-media-query-list①"></a>[parsing](#parse-a-media-query-list) the given value to the <a id="ref-for-medialist-collection-of-media-queries③"></a>[collection of media queries](#medialist-collection-of-media-queries).

The <a id="dom-medialist-item"></a><strong><code>item(<var>index</var>)</code></strong> method must return a <a id="ref-for-serialize-a-media-query②"></a>[serialization](#serialize-a-media-query) of the media query in the <a id="ref-for-medialist-collection-of-media-queries④"></a>[collection of media queries](#medialist-collection-of-media-queries) given by <var>index</var>, or null, if <var>index</var> is greater than or equal to the number of media queries in the <a id="ref-for-medialist-collection-of-media-queries⑤"></a>collection of media queries.

The <a id="dom-medialist-length"></a><strong><code>length</code></strong> attribute must return the number of media queries in the <a id="ref-for-medialist-collection-of-media-queries⑥"></a>[collection of media queries](#medialist-collection-of-media-queries).

The <a id="dom-medialist-appendmedium"></a><strong><code>appendMedium(<var>medium</var>)</code></strong> method must run these steps:

1.  Let <var>m</var> be the result of <a id="ref-for-parse-a-media-query"></a>[parsing](#parse-a-media-query) the given value.
2.  If <var>m</var> is null, then return.
3.  If <a id="ref-for-compare-media-queries"></a>[comparing](#compare-media-queries) <var>m</var> with any of the media queries in the <a id="ref-for-medialist-collection-of-media-queries⑦"></a>[collection of media queries](#medialist-collection-of-media-queries) returns true, then return.
4.  Append <var>m</var> to the <a id="ref-for-medialist-collection-of-media-queries⑧"></a>[collection of media queries](#medialist-collection-of-media-queries).

The <a id="dom-medialist-deletemedium"></a><strong><code>deleteMedium(<var>medium</var>)</code></strong> method must run these steps:

1.  Let <var>m</var> be the result of <a id="ref-for-parse-a-media-query①"></a>[parsing](#parse-a-media-query) the given value.
2.  If <var>m</var> is null, then return.
3.  Remove any media query from the <a id="ref-for-medialist-collection-of-media-queries⑨"></a>[collection of media queries](#medialist-collection-of-media-queries) for which <a id="ref-for-compare-media-queries①"></a>[comparing](#compare-media-queries) the media query with <var>m</var> returns true. If nothing was removed, then <a id="ref-for-dfn-throw"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) a <code><a id="ref-for-notfounderror"></a>[NotFoundError](https://webidl.spec.whatwg.org/#notfounderror)</code> exception.

## <a id="selectors"></a>5. Selectors[](#selectors)

Selectors are defined in the Selectors specification. This section mainly defines how to serialize them.

### <a id="parsing-selectors"></a>5.1. Parsing Selectors[](#parsing-selectors)

To <a id="parse-a-group-of-selectors"></a><strong>parse a group of selectors</strong> means to parse the value using the <code>selectors&#95;group</code> production defined in the Selectors specification and return either a group of selectors if parsing did not fail or null if parsing did fail.

### <a id="serializing-selectors"></a>5.2. Serializing Selectors[](#serializing-selectors)

To <a id="serialize-a-group-of-selectors"></a><strong>serialize a group of selectors</strong> <a id="ref-for-serialize-a-selector"></a>[serialize](#serialize-a-selector) each selector in the group of selectors and then <a id="ref-for-serialize-a-comma-separated-list①"></a>[serialize](#serialize-a-comma-separated-list) a comma-separated list of these serializations.

To <a id="serialize-a-selector"></a><strong>serialize a selector</strong> let <var>s</var> be the empty string, run the steps below for each part of the chain of the selector, and finally return <var>s</var>:

1.  If there is only one <a id="ref-for-simple"></a>[simple selector](https://drafts.csswg.org/selectors-4/#simple) in the <a id="ref-for-compound"></a>[compound selectors](https://drafts.csswg.org/selectors-4/#compound) which is a <a id="ref-for-universal-selector"></a>[universal selector](https://drafts.csswg.org/selectors-4/#universal-selector), append the result of <a id="ref-for-serialize-a-simple-selector"></a>[serializing](#serialize-a-simple-selector) the <a id="ref-for-universal-selector①"></a>universal selector to <var>s</var>.
2.  Otherwise, for each <a id="ref-for-simple①"></a>[simple selector](https://drafts.csswg.org/selectors-4/#simple) in the <a id="ref-for-compound①"></a>[compound selectors](https://drafts.csswg.org/selectors-4/#compound) that is not a universal selector of which the <a id="ref-for-namespace-prefix"></a>[namespace prefix](https://drafts.csswg.org/css-namespaces-3/#namespace-prefix) maps to a namespace that is not the <a id="ref-for-default-namespace"></a>[default namespace](https://drafts.csswg.org/css-namespaces-3/#default-namespace) <a id="ref-for-serialize-a-simple-selector①"></a>[serialize](#serialize-a-simple-selector) the <a id="ref-for-simple②"></a>simple selector and append the result to <var>s</var>.
3.  If this is not the last part of the chain of the selector append a single SPACE (U+0020), followed by the combinator "<code>&#62;</code>", "<code>+</code>", "<code>&#126;</code>", "<code>&#62;&#62;</code>", "<code>&#124;&#124;</code>", as appropriate, followed by another single SPACE (U+0020) if the combinator was not whitespace, to <var>s</var>.
4.  If this is the last part of the chain of the selector and there is a pseudo-element, append "<code>&#58;&#58;</code>" followed by the name of the pseudo-element, to <var>s</var>.

Tests

- [selectorSerialize.html](https://wpt.fyi/results/css/cssom/selectorSerialize.html) [(live test)](http://wpt.live/css/cssom/selectorSerialize.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/selectorSerialize.html)
- [serialize-namespaced-type-selectors.html](https://wpt.fyi/results/css/cssom/serialize-namespaced-type-selectors.html) [(live test)](http://wpt.live/css/cssom/serialize-namespaced-type-selectors.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/serialize-namespaced-type-selectors.html)

To <a id="serialize-a-simple-selector"></a><strong>serialize a simple selector</strong> let <var>s</var> be the empty string, run the steps below, and finally return <var>s</var>:

<strong>type selector</strong>

<strong>universal selector</strong>

1.  If the <a id="ref-for-namespace-prefix①"></a>[namespace prefix](https://drafts.csswg.org/css-namespaces-3/#namespace-prefix) maps to a namespace that is not the <a id="ref-for-default-namespace①"></a>[default namespace](https://drafts.csswg.org/css-namespaces-3/#default-namespace) and is not the null namespace (not in a namespace) append the <a id="ref-for-serialize-an-identifier②"></a>[serialization](#serialize-an-identifier) of the <a id="ref-for-namespace-prefix②"></a>namespace prefix as an identifier, followed by a "<code>&#124;</code>" (U+007C) to <var>s</var>.
2.  If the <a id="ref-for-namespace-prefix③"></a>[namespace prefix](https://drafts.csswg.org/css-namespaces-3/#namespace-prefix) maps to a namespace that is the null namespace (not in a namespace) append "<code>&#124;</code>" (U+007C) to <var>s</var>.
3.  If this is a type selector append the <a id="ref-for-serialize-an-identifier③"></a>[serialization](#serialize-an-identifier) of the element name as an identifier to <var>s</var>.
4.  If this is a universal selector append "<code>&#42;</code>" (U+002A) to <var>s</var>.

<strong>attribute selector</strong>

1.  Append "<code>&#91;</code>" (U+005B) to <var>s</var>.
2.  If the <a id="ref-for-namespace-prefix④"></a>[namespace prefix](https://drafts.csswg.org/css-namespaces-3/#namespace-prefix) maps to a namespace that is not the null namespace (not in a namespace) append the <a id="ref-for-serialize-an-identifier④"></a>[serialization](#serialize-an-identifier) of the <a id="ref-for-namespace-prefix⑤"></a>namespace prefix as an identifier, followed by a "<code>&#124;</code>" (U+007C) to <var>s</var>.
3.  Append the <a id="ref-for-serialize-an-identifier⑤"></a>[serialization](#serialize-an-identifier) of the attribute name as an identifier to <var>s</var>.
4.  If there is an attribute value specified, append "<code>=</code>", "<code>&#126;=</code>", "<code>&#124;=</code>", "<code>^=</code>", "<code>$=</code>", or "<code>&#42;=</code>" as appropriate (depending on the type of attribute selector), followed by the <a id="ref-for-serialize-a-string②"></a>[serialization](#serialize-a-string) of the attribute value as a string, to <var>s</var>.
5.  If the attribute selector has the case-sensitivity flag present, append "<code> i</code>" (U+0020 U+0069) to <var>s</var>.
6.  Append "<code>&#93;</code>" (U+005D) to <var>s</var>.

<strong>class selector</strong>

Append a "<code>.</code>" (U+002E), followed by the <a id="ref-for-serialize-an-identifier⑥"></a>[serialization](#serialize-an-identifier) of the class name as an identifier to <var>s</var>.

<strong>ID selector</strong>

Append a "<code>&#35;</code>" (U+0023), followed by the <a id="ref-for-serialize-an-identifier⑦"></a>[serialization](#serialize-an-identifier) of the ID as an identifier to <var>s</var>.

<strong>pseudo-class</strong>

If the pseudo-class does not accept arguments append "<code>&#58;</code>" (U+003A), followed by the name of the pseudo-class, to <var>s</var>.

Otherwise, append "<code>&#58;</code>" (U+003A), followed by the name of the pseudo-class, followed by "<code>(</code>" (U+0028), followed by the value of the pseudo-class argument(s) determined as per below, followed by "<code>)</code>" (U+0029), to <var>s</var>.

<strong><code>&#58;lang()</code></strong>

The <a id="ref-for-serialize-a-comma-separated-list②"></a>[serialization of a comma-separated list](#serialize-a-comma-separated-list) of each argument’s <a id="ref-for-serialize-a-string③"></a>[serialization as a string](#serialize-a-string), preserving relative order.

<strong><code>&#58;nth-child()</code></strong>

<strong><code>&#58;nth-last-child()</code></strong>

<strong><code>&#58;nth-of-type()</code></strong>

<strong><code>&#58;nth-last-of-type()</code></strong>

The result of serializing the value using the rules to <a id="ref-for-serialize-an-a-n-plus-b-value"></a>[serialize an \<a-n-plus-b\> value](https://drafts.csswg.org/css-syntax-3/#serialize-an-a-n-plus-b-value).

<strong><code>&#58;not()</code></strong>

The result of serializing the value using the rules for <a id="ref-for-serialize-a-group-of-selectors"></a>[serializing a group of selectors](#serialize-a-group-of-selectors).

## <a id="css-object-model"></a>6. CSS[](#css-object-model)

### <a id="css-style-sheets"></a>6.1. CSS Style Sheets[](#css-style-sheets)

A <a id="css-style-sheet"></a><strong>CSS style sheet</strong> is an abstract concept that represents a style sheet as defined by the CSS specification. In the CSSOM a <a id="ref-for-css-style-sheet"></a>[CSS style sheet](#css-style-sheet) is represented as a <code><a id="ref-for-cssstylesheet"></a>[CSSStyleSheet](#cssstylesheet)</code> object.

<strong><a id="dom-cssstylesheet-cssstylesheet"></a><strong><code>CSSStyleSheet(<var>options</var>)</code></strong></strong>

When called, execute the steps to <a id="ref-for-create-a-constructed-cssstylesheet"></a>[create a constructed CSSStyleSheet](#create-a-constructed-cssstylesheet) given <var>options</var> and return the result.

<strong>To <a id="create-a-constructed-cssstylesheet"></a><strong>create a constructed <code><a id="ref-for-cssstylesheet①"></a>[CSSStyleSheet](#cssstylesheet)</code></strong></strong>

given <code><a id="ref-for-dictdef-cssstylesheetinit"></a>[CSSStyleSheetInit](#dictdef-cssstylesheetinit)</code> <var>options</var>, run these steps:

1.  Construct a new <code><a id="ref-for-cssstylesheet②"></a>[CSSStyleSheet](#cssstylesheet)</code> object <var>sheet</var>.
2.  Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-location"></a>[location](https://drafts.csswg.org/cssom-1/#concept-css-style-sheet-location) to the <u>base URL</u> of the <a id="ref-for-concept-document-window"></a>[associated Document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window) for the <a id="ref-for-current-global-object"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object).
3.  Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-stylesheet-base-url"></a>[stylesheet base URL](#concept-css-style-sheet-stylesheet-base-url) to the <code><a id="ref-for-dom-cssstylesheetinit-baseurl"></a>[baseURL](#dom-cssstylesheetinit-baseurl)</code> attribute value from <var>options</var>.
4.  Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-parent-css-style-sheet"></a>[parent CSS style sheet](#concept-css-style-sheet-parent-css-style-sheet) to null.
5.  Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-owner-node"></a>[owner node](#concept-css-style-sheet-owner-node) to null.
6.  Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-owner-css-rule"></a>[owner CSS rule](#concept-css-style-sheet-owner-css-rule) to null.
7.  Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-title"></a>[title](#concept-css-style-sheet-title) to the empty string.
8.  Unset <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-alternate-flag"></a>[alternate flag](#concept-css-style-sheet-alternate-flag).
9.  Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-origin-clean-flag"></a>[origin-clean flag](#concept-css-style-sheet-origin-clean-flag).
10. Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-constructed-flag"></a>[constructed flag](#concept-css-style-sheet-constructed-flag).
11. Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-constructor-document"></a>[Constructor document](#concept-css-style-sheet-constructor-document) to the <a id="ref-for-concept-document-window①"></a>[associated Document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window) for the <a id="ref-for-current-global-object①"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object).
12. If the <code><a id="ref-for-dom-cssstylesheetinit-media"></a>[media](#dom-cssstylesheetinit-media)</code> attribute of <var>options</var> is a string, <a id="ref-for-create-a-medialist-object"></a>[create a MediaList object](#create-a-medialist-object) from the string and assign it as <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-media"></a>[media](#concept-css-style-sheet-media). Otherwise, <a id="ref-for-serialize-a-media-query-list①"></a>[serialize a media query list](#serialize-a-media-query-list) from the attribute and then <a id="ref-for-create-a-medialist-object①"></a>create a MediaList object from the resulting string and set it as <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-media①"></a>media.
13. If the <code><a id="ref-for-dom-cssstylesheetinit-disabled"></a>[disabled](#dom-cssstylesheetinit-disabled)</code> attribute of <var>options</var> is true, set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-disabled-flag"></a>[disabled flag](#concept-css-style-sheet-disabled-flag).
14. Return <var>sheet</var>.

Tests

- [CSSStyleSheet-constructable-baseURL.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable-baseURL.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable-baseURL.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable-baseURL.html)
- [CSSStyleSheet-constructable-concat.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable-concat.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable-concat.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable-concat.html)
- [CSSStyleSheet-constructable-cssRules.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable-cssRules.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable-cssRules.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable-cssRules.html)
- [CSSStyleSheet-constructable-disabled-regular-sheet-insertion.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable-disabled-regular-sheet-insertion.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable-disabled-regular-sheet-insertion.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable-disabled-regular-sheet-insertion.html)
- [CSSStyleSheet-constructable-disallow-import.tentative.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable-disallow-import.tentative.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable-disallow-import.tentative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable-disallow-import.tentative.html)
- [CSSStyleSheet-constructable-duplicate.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable-duplicate.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable-duplicate.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable-duplicate.html)
- [CSSStyleSheet-constructable-insertRule-base-uri.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable-insertRule-base-uri.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable-insertRule-base-uri.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable-insertRule-base-uri.html)
- [CSSStyleSheet-constructable-invalidation.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable-invalidation.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable-invalidation.html)
- [CSSStyleSheet-constructable-replace-cssRules.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable-replace-cssRules.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable-replace-cssRules.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable-replace-cssRules.html)
- [CSSStyleSheet-constructable-replace-on-regular-sheet.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable-replace-on-regular-sheet.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable-replace-on-regular-sheet.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable-replace-on-regular-sheet.html)
- [CSSStyleSheet-constructable.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-constructable.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-constructable.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-constructable.html)
- [CSSStyleSheet-modify-after-removal.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-modify-after-removal.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-modify-after-removal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-modify-after-removal.html)
- [CSSStyleSheet-template-adoption.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet-template-adoption.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet-template-adoption.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet-template-adoption.html)
- [CSSStyleSheet.html](https://wpt.fyi/results/css/cssom/CSSStyleSheet.html) [(live test)](http://wpt.live/css/cssom/CSSStyleSheet.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleSheet.html)
- [style-sheet-interfaces-001.html](https://wpt.fyi/results/css/cssom/style-sheet-interfaces-001.html) [(live test)](http://wpt.live/css/cssom/style-sheet-interfaces-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/style-sheet-interfaces-001.html)
- [style-sheet-interfaces-002.html](https://wpt.fyi/results/css/cssom/style-sheet-interfaces-002.html) [(live test)](http://wpt.live/css/cssom/style-sheet-interfaces-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/style-sheet-interfaces-002.html)

A <a id="ref-for-css-style-sheet①"></a>[CSS style sheet](#css-style-sheet) has a number of associated state items:

<strong><a id="concept-css-style-sheet-type"></a><strong>type</strong></strong>

The literal string "<code>text/css</code>".

<strong><a id="concept-css-style-sheet-location"></a><strong>location</strong></strong>

Specified when created. The <a id="ref-for-absolute-url-string"></a>[absolute-URL string](https://url.spec.whatwg.org/#absolute-url-string) of the first request of the <a id="ref-for-css-style-sheet②"></a>[CSS style sheet](#css-style-sheet) or null if the <a id="ref-for-css-style-sheet③"></a>CSS style sheet was embedded. Does not change during the lifetime of the <a id="ref-for-css-style-sheet④"></a>CSS style sheet.

<strong><a id="concept-css-style-sheet-parent-css-style-sheet"></a><strong>parent CSS style sheet</strong></strong>

Specified when created. The <a id="ref-for-css-style-sheet⑤"></a>[CSS style sheet](#css-style-sheet) that is the parent of the <a id="ref-for-css-style-sheet⑥"></a>CSS style sheet or null if there is no associated parent.

<strong><a id="concept-css-style-sheet-owner-node"></a><strong>owner node</strong></strong>

Specified when created. The DOM node associated with the <a id="ref-for-css-style-sheet⑦"></a>[CSS style sheet](#css-style-sheet) or null if there is no associated DOM node.

<strong><a id="concept-css-style-sheet-owner-css-rule"></a><strong>owner CSS rule</strong></strong>

Specified when created. The <a id="ref-for-css-rule"></a>[CSS rule](#css-rule) in the <a id="ref-for-concept-css-style-sheet-parent-css-style-sheet①"></a>[parent CSS style sheet](#concept-css-style-sheet-parent-css-style-sheet) that caused the inclusion of the <a id="ref-for-css-style-sheet⑧"></a>[CSS style sheet](#css-style-sheet) or null if there is no associated rule.

<strong><a id="concept-css-style-sheet-media"></a><strong>media</strong></strong>

Specified when created. The <code><a id="ref-for-medialist①"></a>[MediaList](#medialist)</code> object associated with the <a id="ref-for-css-style-sheet⑨"></a>[CSS style sheet](#css-style-sheet).

If this property is specified to a string, the <a id="ref-for-concept-css-style-sheet-media②"></a>[media](#concept-css-style-sheet-media) must be set to the return value of invoking <a id="ref-for-create-a-medialist-object②"></a>[create a <code>MediaList</code> object](#create-a-medialist-object) steps for that string.

If this property is specified to an attribute of the <a id="ref-for-concept-css-style-sheet-owner-node①"></a>[owner node](#concept-css-style-sheet-owner-node), the <a id="ref-for-concept-css-style-sheet-media③"></a>[media](#concept-css-style-sheet-media) must be set to the return value of invoking <a id="ref-for-create-a-medialist-object③"></a>[create a <code>MediaList</code> object](#create-a-medialist-object) steps for the value of that attribute. Whenever the attribute is set, changed or removed, the <a id="ref-for-concept-css-style-sheet-media④"></a>media’s <code><a id="ref-for-dom-medialist-mediatext③"></a>[mediaText](#dom-medialist-mediatext)</code> attribute must be set to the new value of the attribute, or to null if the attribute is absent.

Note: Changing the <a id="ref-for-concept-css-style-sheet-media⑤"></a>[media](#concept-css-style-sheet-media)’s <code><a id="ref-for-dom-medialist-mediatext④"></a>[mediaText](#dom-medialist-mediatext)</code> attribute does not change the corresponding attribute on the <a id="ref-for-concept-css-style-sheet-owner-node②"></a>[owner node](#concept-css-style-sheet-owner-node).

Note: The <a id="ref-for-concept-css-style-sheet-owner-node③"></a>[owner node](#concept-css-style-sheet-owner-node) of a <a id="ref-for-css-style-sheet①⓪"></a>[CSS style sheet](#css-style-sheet), if non-null, is the node whose <a id="ref-for-associated-css-style-sheet"></a>[associated CSS style sheet](#associated-css-style-sheet) is the <a id="ref-for-css-style-sheet①①"></a>CSS style sheet in question, when the <a id="ref-for-css-style-sheet①②"></a>CSS style sheet is <a id="ref-for-add-a-css-style-sheet"></a>[added](#add-a-css-style-sheet).

<strong><a id="concept-css-style-sheet-title"></a><strong>title</strong></strong>

Specified when created. The title of the <a id="ref-for-css-style-sheet①③"></a>[CSS style sheet](#css-style-sheet), which can be the empty string.

<a id="example-a945fdba"></a>

<strong>Example:</strong>

[](#example-a945fdba) In the following, the <a id="ref-for-concept-css-style-sheet-title①"></a>[title](#concept-css-style-sheet-title) is non-empty for the first style sheet, but is empty for the second and third style sheets.

``` text
<style title="papaya whip">
  body { background: #ffefd5; }
</style>
```

``` text
<style title="">
  body { background: orange; }
</style>
```

``` text
<style>
  body { background: brown; }
</style>
```

If this property is specified to an attribute of the <a id="ref-for-concept-css-style-sheet-owner-node④"></a>[owner node](#concept-css-style-sheet-owner-node), the <a id="ref-for-concept-css-style-sheet-title②"></a>[title](#concept-css-style-sheet-title) must be set to the value of that attribute. Whenever the attribute is set, changed or removed, the <a id="ref-for-concept-css-style-sheet-title③"></a>title must be set to the new value of the attribute, or to the empty string if the attribute is absent.

Note: HTML only [specifies](https://html.spec.whatwg.org/#the-style-element:concept-css-style-sheet-title) <a id="ref-for-concept-css-style-sheet-title④"></a>[title](#concept-css-style-sheet-title) to be an attribute of the <a id="ref-for-concept-css-style-sheet-owner-node⑤"></a>[owner node](#concept-css-style-sheet-owner-node) if the node is in <a id="ref-for-in-a-document-tree"></a>[in a document tree](https://dom.spec.whatwg.org/#in-a-document-tree).

<strong><a id="concept-css-style-sheet-alternate-flag"></a><strong>alternate flag</strong></strong>

Specified when created. Either set or unset. Unset by default.

<a id="example-cd0d9c55"></a>

<strong>Example:</strong>

[](#example-cd0d9c55) The following <a id="ref-for-css-style-sheet①④"></a>[CSS style sheets](#css-style-sheet) have their <a id="ref-for-concept-css-style-sheet-alternate-flag①"></a>[alternate flag](#concept-css-style-sheet-alternate-flag) set:

``` text
<?xml-stylesheet alternate="yes" title="x" href="data:text/css,…"?>
```

``` text
<link rel="alternate stylesheet" title="x" href="data:text/css,…">
```

<strong><a id="concept-css-style-sheet-disabled-flag"></a><strong>disabled flag</strong></strong>

Either set or unset. Unset by default.

Note: Even when unset it does not necessarily mean that the <a id="ref-for-css-style-sheet①⑤"></a>[CSS style sheet](#css-style-sheet) is actually used for rendering.

<strong><a id="concept-css-style-sheet-css-rules"></a><strong>CSS rules</strong></strong>

The CSS rules associated with the <a id="ref-for-css-style-sheet①⑥"></a>[CSS style sheet](#css-style-sheet).

<strong><a id="concept-css-style-sheet-origin-clean-flag"></a><strong>origin-clean flag</strong></strong>

Specified when created. Either set or unset. If it is set, the API allows reading and modifying of the <a id="ref-for-concept-css-style-sheet-css-rules"></a>[CSS rules](#concept-css-style-sheet-css-rules).

<strong><a id="concept-css-style-sheet-constructed-flag"></a><strong>constructed flag</strong></strong>

Specified when created. Either set or unset. Unset by default. Signifies whether this stylesheet was created by invoking the IDL-defined constructor.

<strong><a id="concept-css-style-sheet-disallow-modification-flag"></a><strong>disallow modification flag</strong></strong>

Either set or unset. Unset by default. If set, modification of the stylesheet’s rules is not allowed.

<strong><a id="concept-css-style-sheet-constructor-document"></a><strong>constructor document</strong></strong>

Specified when created. The <code><a id="ref-for-document"></a>[Document](https://dom.spec.whatwg.org/#document)</code> a constructed stylesheet is associated with. Null by default. Only non-null for stylesheets that have <a id="ref-for-concept-css-style-sheet-constructed-flag①"></a>[constructed flag](#concept-css-style-sheet-constructed-flag) set.

<strong><a id="concept-css-style-sheet-stylesheet-base-url"></a><strong>stylesheet base URL</strong></strong>

The base URL to use when resolving relative URLs in the stylesheet. Null by default. Only non-null for stylesheets that have <a id="ref-for-concept-css-style-sheet-constructed-flag②"></a>[constructed flag](#concept-css-style-sheet-constructed-flag) set.

Tests

- [base-uri.html](https://wpt.fyi/results/css/cssom/base-uri.html) [(live test)](http://wpt.live/css/cssom/base-uri.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/base-uri.html)

#### <a id="the-stylesheet-interface"></a>6.1.1. The <code><a id="ref-for-stylesheet"></a>[StyleSheet](#stylesheet)</code> Interface[](#the-stylesheet-interface)

The <code><a id="ref-for-stylesheet①"></a>[StyleSheet](#stylesheet)</code> interface represents an abstract, base style sheet.

<a id="ref-for-Exposed①"></a><a id="stylesheet"></a><a id="ref-for-cssomstring④"></a><a id="ref-for-dom-stylesheet-type"></a><a id="ref-for-idl-USVString③"></a><a id="ref-for-dom-stylesheet-href"></a><a id="ref-for-element"></a><a id="ref-for-processinginstruction"></a><a id="ref-for-dom-stylesheet-ownernode"></a><a id="ref-for-cssstylesheet③"></a><a id="ref-for-dom-stylesheet-parentstylesheet"></a><a id="ref-for-idl-DOMString②"></a><a id="ref-for-dom-stylesheet-title"></a><a id="ref-for-SameObject"></a><a id="ref-for-PutForwards"></a><a id="ref-for-dom-medialist-mediatext⑤"></a><a id="ref-for-medialist②"></a><a id="ref-for-dom-stylesheet-media"></a><a id="ref-for-idl-boolean"></a><a id="ref-for-dom-stylesheet-disabled"></a>

``` text
[Exposed=Window]
interface StyleSheet {
  readonly attribute CSSOMString type;
  readonly attribute USVString? href;
  readonly attribute (Element or ProcessingInstruction)? ownerNode;
  readonly attribute CSSStyleSheet? parentStyleSheet;
  readonly attribute DOMString? title;
  [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
  attribute boolean disabled;
};
```

The <a id="dom-stylesheet-type"></a><strong><code>type</code></strong> attribute must return the <a id="ref-for-concept-css-style-sheet-type"></a>[type](#concept-css-style-sheet-type).

The <a id="dom-stylesheet-href"></a><strong><code>href</code></strong> attribute must return the <a id="ref-for-concept-css-style-sheet-location①"></a>[location](#concept-css-style-sheet-location).

The <a id="dom-stylesheet-ownernode"></a><strong><code>ownerNode</code></strong> attribute must return the <a id="ref-for-concept-css-style-sheet-owner-node⑥"></a>[owner node](#concept-css-style-sheet-owner-node).

The <a id="dom-stylesheet-parentstylesheet"></a><strong><code>parentStyleSheet</code></strong> attribute must return the <a id="ref-for-concept-css-style-sheet-parent-css-style-sheet②"></a>[parent CSS style sheet](#concept-css-style-sheet-parent-css-style-sheet).

The <a id="dom-stylesheet-title"></a><strong><code>title</code></strong> attribute must return the <a id="ref-for-concept-css-style-sheet-title⑤"></a>[title](#concept-css-style-sheet-title) or null if <a id="ref-for-concept-css-style-sheet-title⑥"></a>title is the empty string.

The <a id="dom-stylesheet-media"></a><strong><code>media</code></strong> attribute must return the <a id="ref-for-concept-css-style-sheet-media⑥"></a>[media](#concept-css-style-sheet-media).

The <a id="dom-stylesheet-disabled"></a><strong><code>disabled</code></strong> attribute, on getting, must return true if the <a id="ref-for-concept-css-style-sheet-disabled-flag①"></a>[disabled flag](#concept-css-style-sheet-disabled-flag) is set, or false otherwise. On setting, the <code><a id="ref-for-dom-stylesheet-disabled①"></a>[disabled](#dom-stylesheet-disabled)</code> attribute must set the <a id="ref-for-concept-css-style-sheet-disabled-flag②"></a>disabled flag if the new value is true, or unset the <a id="ref-for-concept-css-style-sheet-disabled-flag③"></a>disabled flag otherwise.

Tests

- [link-element-stylesheet-title.html](https://wpt.fyi/results/css/cssom/link-element-stylesheet-title.html) [(live test)](http://wpt.live/css/cssom/link-element-stylesheet-title.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/link-element-stylesheet-title.html)
- [stylesheet-same-origin.sub.html](https://wpt.fyi/results/css/cssom/stylesheet-same-origin.sub.html) [(live test)](http://wpt.live/css/cssom/stylesheet-same-origin.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/stylesheet-same-origin.sub.html)
- [stylesheet-title.html](https://wpt.fyi/results/css/cssom/stylesheet-title.html) [(live test)](http://wpt.live/css/cssom/stylesheet-title.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/stylesheet-title.html)

#### <a id="the-cssstylesheet-interface"></a>6.1.2. The <code><a id="ref-for-cssstylesheet④"></a>[CSSStyleSheet](#cssstylesheet)</code> Interface[](#the-cssstylesheet-interface)

The <code><a id="ref-for-cssstylesheet⑤"></a>[CSSStyleSheet](#cssstylesheet)</code> interface represents a <a id="ref-for-css-style-sheet①⑦"></a>[CSS style sheet](#css-style-sheet).

<a id="ref-for-Exposed②"></a><a id="cssstylesheet"></a><a id="ref-for-stylesheet②"></a><a id="ref-for-dom-cssstylesheet-cssstylesheet"></a><a id="ref-for-dictdef-cssstylesheetinit①"></a><a id="dom-cssstylesheet-cssstylesheet-options-options"></a><a id="ref-for-cssrule"></a><a id="ref-for-dom-cssstylesheet-ownerrule"></a><a id="ref-for-SameObject①"></a><a id="ref-for-cssrulelist"></a><a id="ref-for-dom-cssstylesheet-cssrules"></a><a id="ref-for-idl-unsigned-long②"></a><a id="ref-for-dom-cssstylesheet-insertrule"></a><a id="ref-for-cssomstring⑤"></a><a id="dom-cssstylesheet-insertrule-rule-index-rule"></a><a id="ref-for-idl-unsigned-long③"></a><a id="dom-cssstylesheet-insertrule-rule-index-index"></a><a id="ref-for-idl-undefined②"></a><a id="ref-for-dom-cssstylesheet-deleterule"></a><a id="ref-for-idl-unsigned-long④"></a><a id="dom-cssstylesheet-deleterule-index-index"></a><a id="ref-for-idl-promise"></a><a id="ref-for-cssstylesheet⑥"></a><a id="ref-for-dom-cssstylesheet-replace"></a><a id="ref-for-idl-USVString④"></a><a id="dom-cssstylesheet-replace-text-text"></a><a id="ref-for-idl-undefined③"></a><a id="ref-for-dom-cssstylesheet-replacesync"></a><a id="ref-for-idl-USVString⑤"></a><a id="dom-cssstylesheet-replacesync-text-text"></a><a id="dictdef-cssstylesheetinit"></a><a id="ref-for-idl-DOMString③"></a><a id="dom-cssstylesheetinit-baseurl"></a><a id="ref-for-medialist③"></a><a id="ref-for-idl-DOMString④"></a><a id="dom-cssstylesheetinit-media"></a><a id="ref-for-idl-boolean①"></a><a id="dom-cssstylesheetinit-disabled"></a>

``` text
[Exposed=Window]
interface CSSStyleSheet : StyleSheet {
  constructor(optional CSSStyleSheetInit options = {});

  readonly attribute CSSRule? ownerRule;
  [SameObject] readonly attribute CSSRuleList cssRules;
  unsigned long insertRule(CSSOMString rule, optional unsigned long index = 0);
  undefined deleteRule(unsigned long index);

  Promise<CSSStyleSheet> replace(USVString text);
  undefined replaceSync(USVString text);
};

dictionary CSSStyleSheetInit {
  DOMString? baseURL = null;
  (MediaList or DOMString) media = "";
  boolean disabled = false;
};
```

The <a id="dom-cssstylesheet-ownerrule"></a><strong><code>ownerRule</code></strong> attribute must return the <a id="ref-for-concept-css-style-sheet-owner-css-rule①"></a>[owner CSS rule](#concept-css-style-sheet-owner-css-rule). If a value other than null is ever returned, then that same value must always be returned on each get access.

The <a id="dom-cssstylesheet-cssrules"></a><strong><code>cssRules</code></strong> attribute must follow these steps:

1.  If the <a id="ref-for-concept-css-style-sheet-origin-clean-flag①"></a>[origin-clean flag](#concept-css-style-sheet-origin-clean-flag) is unset, <a id="ref-for-dfn-throw①"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) a <code><a id="ref-for-securityerror"></a>[SecurityError](https://webidl.spec.whatwg.org/#securityerror)</code> exception.

2.  Return a read-only, live <code><a id="ref-for-cssrulelist①"></a>[CSSRuleList](#cssrulelist)</code> object representing the <a id="ref-for-concept-css-style-sheet-css-rules①"></a>[CSS rules](#concept-css-style-sheet-css-rules).

    Note: Even though the returned <code><a id="ref-for-cssrulelist②"></a>[CSSRuleList](#cssrulelist)</code> object is read-only (from the perspective of client-authored script), it can nevertheless change over time due to its liveness status. For example, invoking the <code><a id="ref-for-dom-cssstylesheet-insertrule①"></a>[insertRule()](#dom-cssstylesheet-insertrule)</code> or <code><a id="ref-for-dom-cssstylesheet-deleterule①"></a>[deleteRule()](#dom-cssstylesheet-deleterule)</code> methods can result in mutations reflected in the returned object.

The <a id="dom-cssstylesheet-insertrule"></a><strong><code>insertRule(<var>rule</var>, <var>index</var>)</code></strong> method must run the following steps:

1.  If the <a id="ref-for-concept-css-style-sheet-origin-clean-flag②"></a>[origin-clean flag](#concept-css-style-sheet-origin-clean-flag) is unset, <a id="ref-for-dfn-throw②"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) a <code><a id="ref-for-securityerror①"></a>[SecurityError](https://webidl.spec.whatwg.org/#securityerror)</code> exception.
2.  If the <a id="ref-for-concept-css-style-sheet-disallow-modification-flag"></a>[disallow modification flag](#concept-css-style-sheet-disallow-modification-flag) is set, throw a <code><a id="ref-for-notallowederror"></a>[NotAllowedError](https://webidl.spec.whatwg.org/#notallowederror)</code> <code><a id="ref-for-idl-DOMException"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.
3.  Let <var>parsed rule</var> be the return value of invoking <a id="ref-for-parse-a-rule"></a>[parse a rule](https://drafts.csswg.org/css-syntax-3/#parse-a-rule) with <var>rule</var>.
4.  If <var>parsed rule</var> is a syntax error, throw a <code><a id="ref-for-syntaxerror"></a>[SyntaxError](https://webidl.spec.whatwg.org/#syntaxerror)</code> <code><a id="ref-for-idl-DOMException①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.
5.  If <var>parsed rule</var> is an <a id="ref-for-at-ruledef-import"></a>[&#64;import](https://drafts.csswg.org/css-cascade-6/#at-ruledef-import) rule, and the <a id="ref-for-concept-css-style-sheet-constructed-flag③"></a>[constructed flag](#concept-css-style-sheet-constructed-flag) is set, throw a <code><a id="ref-for-syntaxerror①"></a>[SyntaxError](https://webidl.spec.whatwg.org/#syntaxerror)</code> <code><a id="ref-for-idl-DOMException②"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.
6.  Return the result of invoking <a id="ref-for-insert-a-css-rule"></a>[insert a CSS rule](#insert-a-css-rule) <var>rule</var> in the <a id="ref-for-concept-css-style-sheet-css-rules②"></a>[CSS rules](#concept-css-style-sheet-css-rules) at <var>index</var>.

Tests

- [insert-dir-rule-crash.html](https://wpt.fyi/results/css/cssom/insert-dir-rule-crash.html) [(live test)](http://wpt.live/css/cssom/insert-dir-rule-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insert-dir-rule-crash.html)
- [insert-dir-rule-in-iframe-crash.html](https://wpt.fyi/results/css/cssom/insert-dir-rule-in-iframe-crash.html) [(live test)](http://wpt.live/css/cssom/insert-dir-rule-in-iframe-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insert-dir-rule-in-iframe-crash.html)
- [insert-invalid-where-rule-crash.html](https://wpt.fyi/results/css/cssom/insert-invalid-where-rule-crash.html) [(live test)](http://wpt.live/css/cssom/insert-invalid-where-rule-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insert-invalid-where-rule-crash.html)
- [insertRule-across-context.html](https://wpt.fyi/results/css/cssom/insertRule-across-context.html) [(live test)](http://wpt.live/css/cssom/insertRule-across-context.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insertRule-across-context.html)
- [insertRule-charset-no-index.html](https://wpt.fyi/results/css/cssom/insertRule-charset-no-index.html) [(live test)](http://wpt.live/css/cssom/insertRule-charset-no-index.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insertRule-charset-no-index.html)
- [insertRule-from-script.html](https://wpt.fyi/results/css/cssom/insertRule-from-script.html) [(live test)](http://wpt.live/css/cssom/insertRule-from-script.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insertRule-from-script.html)
- [insertRule-import-no-index.html](https://wpt.fyi/results/css/cssom/insertRule-import-no-index.html) [(live test)](http://wpt.live/css/cssom/insertRule-import-no-index.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insertRule-import-no-index.html)
- [insertRule-import-no-sheet-crash.html](https://wpt.fyi/results/css/cssom/insertRule-import-no-sheet-crash.html) [(live test)](http://wpt.live/css/cssom/insertRule-import-no-sheet-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insertRule-import-no-sheet-crash.html)
- [insertRule-import-trailing-garbage-crash.html](https://wpt.fyi/results/css/cssom/insertRule-import-trailing-garbage-crash.html) [(live test)](http://wpt.live/css/cssom/insertRule-import-trailing-garbage-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insertRule-import-trailing-garbage-crash.html)
- [insertRule-namespace-no-index.html](https://wpt.fyi/results/css/cssom/insertRule-namespace-no-index.html) [(live test)](http://wpt.live/css/cssom/insertRule-namespace-no-index.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insertRule-namespace-no-index.html)
- [insertRule-no-index.html](https://wpt.fyi/results/css/cssom/insertRule-no-index.html) [(live test)](http://wpt.live/css/cssom/insertRule-no-index.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insertRule-no-index.html)
- [insertRule-syntax-error-01.html](https://wpt.fyi/results/css/cssom/insertRule-syntax-error-01.html) [(live test)](http://wpt.live/css/cssom/insertRule-syntax-error-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/insertRule-syntax-error-01.html)

The <a id="dom-cssstylesheet-deleterule"></a><strong><code>deleteRule(<var>index</var>)</code></strong> method must run the following steps:

1.  If the <a id="ref-for-concept-css-style-sheet-origin-clean-flag③"></a>[origin-clean flag](#concept-css-style-sheet-origin-clean-flag) is unset, <a id="ref-for-dfn-throw③"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) a <code><a id="ref-for-securityerror②"></a>[SecurityError](https://webidl.spec.whatwg.org/#securityerror)</code> exception.
2.  If the <a id="ref-for-concept-css-style-sheet-disallow-modification-flag①"></a>[disallow modification flag](#concept-css-style-sheet-disallow-modification-flag) is set, throw a <code><a id="ref-for-notallowederror①"></a>[NotAllowedError](https://webidl.spec.whatwg.org/#notallowederror)</code> <code><a id="ref-for-idl-DOMException③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.
3.  <a id="ref-for-remove-a-css-rule"></a>[Remove a CSS rule](#remove-a-css-rule) in the <a id="ref-for-concept-css-style-sheet-css-rules③"></a>[CSS rules](#concept-css-style-sheet-css-rules) at <var>index</var>.

The <a id="dom-cssstylesheet-replace"></a><strong><code>replace(<a id="ref-for-concept-css-rule-text"></a>[text](#concept-css-rule-text))</code></strong> method must run the following steps:

1.  Let <var>promise</var> be a promise.
2.  If the <a id="ref-for-concept-css-style-sheet-constructed-flag④"></a>[constructed flag](#concept-css-style-sheet-constructed-flag) is not set, or the <a id="ref-for-concept-css-style-sheet-disallow-modification-flag②"></a>[disallow modification flag](#concept-css-style-sheet-disallow-modification-flag) is set, reject <var>promise</var> with a <code><a id="ref-for-notallowederror②"></a>[NotAllowedError](https://webidl.spec.whatwg.org/#notallowederror)</code> <code><a id="ref-for-idl-DOMException④"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> and return <var>promise</var>.
3.  Set the <a id="ref-for-concept-css-style-sheet-disallow-modification-flag③"></a>[disallow modification flag](#concept-css-style-sheet-disallow-modification-flag).
4.  <a id="ref-for-in-parallel"></a>[In parallel](https://html.spec.whatwg.org/multipage/infrastructure.html#in-parallel), do these steps:
    1.  Let <var>rules</var> be the result of running <a id="ref-for-parse-a-stylesheets-contents"></a>[parse a stylesheet’s contents](https://drafts.csswg.org/css-syntax-3/#parse-a-stylesheets-contents) from <var>text</var>.
    2.  If <var>rules</var> contains one or more <a id="ref-for-at-ruledef-import①"></a>[&#64;import](https://drafts.csswg.org/css-cascade-6/#at-ruledef-import) rules, <a id="ref-for-remove-a-css-rule①"></a>[remove those rules](#remove-a-css-rule) from <var>rules</var>.
    3.  Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-css-rules④"></a>[CSS rules](#concept-css-style-sheet-css-rules) to <var>rules</var>.
    4.  Unset <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-disallow-modification-flag④"></a>[disallow modification flag](#concept-css-style-sheet-disallow-modification-flag).
    5.  Resolve <var>promise</var> with <var>sheet</var>.
5.  Return <var>promise</var>.

The <a id="dom-cssstylesheet-replacesync"></a><strong><code>replaceSync(<a id="ref-for-concept-css-rule-text①"></a>[text](#concept-css-rule-text))</code></strong> method must run the steps to <a id="ref-for-synchronously-replace-the-rules-of-a-cssstylesheet"></a>[synchronously replace the rules of a CSSStyleSheet](#synchronously-replace-the-rules-of-a-cssstylesheet) on this <code><a id="ref-for-cssstylesheet⑦"></a>[CSSStyleSheet](#cssstylesheet)</code> given <var>text</var>.

To <a id="synchronously-replace-the-rules-of-a-cssstylesheet"></a><strong>synchronously replace the rules of a CSSStyleSheet</strong> on <var>sheet</var> given <var>text</var>, run these steps:

1.  If the <a id="ref-for-concept-css-style-sheet-constructed-flag⑤"></a>[constructed flag](#concept-css-style-sheet-constructed-flag) is not set, or the <a id="ref-for-concept-css-style-sheet-disallow-modification-flag⑤"></a>[disallow modification flag](#concept-css-style-sheet-disallow-modification-flag) is set, throw a <code><a id="ref-for-notallowederror③"></a>[NotAllowedError](https://webidl.spec.whatwg.org/#notallowederror)</code> <code><a id="ref-for-idl-DOMException⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.
2.  Let <var>rules</var> be the result of running <a id="ref-for-parse-a-stylesheets-contents①"></a>[parse a stylesheet’s contents](https://drafts.csswg.org/css-syntax-3/#parse-a-stylesheets-contents) from <var>text</var>.
3.  If <var>rules</var> contains one or more <a id="ref-for-at-ruledef-import②"></a>[&#64;import](https://drafts.csswg.org/css-cascade-6/#at-ruledef-import) rules, <a id="ref-for-remove-a-css-rule②"></a>[remove those rules](#remove-a-css-rule) from <var>rules</var>.
4.  Set <var>sheet</var>’s <a id="ref-for-concept-css-style-sheet-css-rules⑤"></a>[CSS rules](#concept-css-style-sheet-css-rules) to <var>rules</var>.

##### <a id="legacy-css-style-sheet-members"></a>6.1.2.1. Deprecated CSSStyleSheet members[](#legacy-css-style-sheet-members)

Note: These members are required for compatibility with existing sites.

<a id="ref-for-cssstylesheet⑧"></a><a id="ref-for-SameObject②"></a><a id="ref-for-cssrulelist③"></a><a id="ref-for-dom-cssstylesheet-rules"></a><a id="ref-for-idl-long"></a><a id="ref-for-dom-cssstylesheet-addrule"></a><a id="ref-for-idl-DOMString⑤"></a><a id="dom-cssstylesheet-addrule-selector-style-index-selector"></a><a id="ref-for-idl-DOMString⑥"></a><a id="dom-cssstylesheet-addrule-selector-style-index-style"></a><a id="ref-for-idl-unsigned-long⑤"></a><a id="dom-cssstylesheet-addrule-selector-style-index-index"></a><a id="ref-for-idl-undefined④"></a><a id="ref-for-dom-cssstylesheet-removerule"></a><a id="ref-for-idl-unsigned-long⑥"></a><a id="dom-cssstylesheet-removerule-index-index"></a>

``` text
partial interface CSSStyleSheet {
  [SameObject] readonly attribute CSSRuleList rules;
  long addRule(optional DOMString selector = "undefined", optional DOMString style = "undefined", optional unsigned long index);
  undefined removeRule(optional unsigned long index = 0);
};
```

The <a id="dom-cssstylesheet-rules"></a><strong><code>rules</code></strong> attribute must follow the same steps as <code><a id="ref-for-dom-cssstylesheet-cssrules①"></a>[cssRules](#dom-cssstylesheet-cssrules)</code>, and return the same object <code><a id="ref-for-dom-cssstylesheet-cssrules②"></a>[cssRules](#dom-cssstylesheet-cssrules)</code> would return.

The <a id="dom-cssstylesheet-removerule"></a><strong><code>removeRule(<var>index</var>)</code></strong> method must run the same steps as <code><a id="ref-for-dom-cssstylesheet-deleterule②"></a>[deleteRule()](#dom-cssstylesheet-deleterule)</code>.

The <a id="dom-cssstylesheet-addrule"></a><strong><code>addRule(<var>selector</var>, <var>block</var>, <var>optionalIndex</var>)</code></strong> method must run the following steps:

1.  Let <var>rule</var> be an empty string.
2.  Append <var>selector</var> to <var>rule</var>.
3.  Append <code>&#34; { &#34;</code> to <var>rule</var>.
4.  If <var>block</var> is not empty, append <var>block</var>, followed by a space, to <var>rule</var>.
5.  Append <code>&#34;}&#34;</code> to <var>rule</var>
6.  Let <var>index</var> be <var>optionalIndex</var> if provided, or the number of <a id="ref-for-concept-css-style-sheet-css-rules⑥"></a>[CSS rules](#concept-css-style-sheet-css-rules) in the stylesheet otherwise.
7.  Call <code><a id="ref-for-dom-cssstylesheet-insertrule②"></a>[insertRule()](#dom-cssstylesheet-insertrule)</code>, with <var>rule</var> and <var>index</var> as arguments.
8.  Return <code>-1</code>.

Authors should not use these members and should instead use and teach the standard

<strong>Advisement:</strong>

<code><a id="ref-for-cssstylesheet⑨"></a>[CSSStyleSheet](#cssstylesheet)</code> interface defined earlier, which is consistent with <code><a id="ref-for-cssgroupingrule"></a>[CSSGroupingRule](#cssgroupingrule)</code>.

Tests

- [removerule-invalidation-crash.html](https://wpt.fyi/results/css/cssom/removerule-invalidation-crash.html) [(live test)](http://wpt.live/css/cssom/removerule-invalidation-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/removerule-invalidation-crash.html)

### <a id="css-style-sheet-collections"></a>6.2. CSS Style Sheet Collections[](#css-style-sheet-collections)

Below various new concepts are defined that are associated with each <code><a id="ref-for-documentorshadowroot"></a>[DocumentOrShadowRoot](https://dom.spec.whatwg.org/#documentorshadowroot)</code> object.

Each <code><a id="ref-for-documentorshadowroot①"></a>[DocumentOrShadowRoot](https://dom.spec.whatwg.org/#documentorshadowroot)</code> has an associated list of zero or more <a id="ref-for-css-style-sheet①⑧"></a>[CSS style sheets](#css-style-sheet), named the <a id="documentorshadowroot-document-or-shadow-root-css-style-sheets"></a><strong>document or shadow root CSS style sheets</strong>. This is an ordered list that contains:

1.  Any <a id="ref-for-css-style-sheet①⑨"></a>[CSS style sheets](#css-style-sheet) created from HTTP <code>Link</code> headers, in header order
2.  Any <a id="ref-for-css-style-sheet②⓪"></a>[CSS style sheets](#css-style-sheet) associated with the <code><a id="ref-for-documentorshadowroot②"></a>[DocumentOrShadowRoot](https://dom.spec.whatwg.org/#documentorshadowroot)</code>, in <a id="ref-for-tree-order"></a>[tree order](https://html.spec.whatwg.org/multipage/infrastructure.html#tree-order)

Each <code><a id="ref-for-documentorshadowroot③"></a>[DocumentOrShadowRoot](https://dom.spec.whatwg.org/#documentorshadowroot)</code> has an associated list of zero or more <a id="ref-for-css-style-sheet②①"></a>[CSS style sheets](#css-style-sheet), named the <a id="documentorshadowroot-final-css-style-sheets"></a><strong>final CSS style sheets</strong>. This is an ordered list that contains:

1.  The <a id="ref-for-documentorshadowroot-document-or-shadow-root-css-style-sheets"></a>[document or shadow root CSS style sheets](#documentorshadowroot-document-or-shadow-root-css-style-sheets).
2.  The contents of <code><a id="ref-for-documentorshadowroot④"></a>[DocumentOrShadowRoot](https://dom.spec.whatwg.org/#documentorshadowroot)</code>’s <code><a id="ref-for-dom-documentorshadowroot-adoptedstylesheets"></a>[adoptedStyleSheets](#dom-documentorshadowroot-adoptedstylesheets)</code>' <a id="ref-for-observable-array-attribute-backing-list"></a>[backing list](https://webidl.spec.whatwg.org/#observable-array-attribute-backing-list), in array order.

To <a id="create-a-css-style-sheet"></a><strong>create a CSS style sheet</strong>, run these steps:

1.  Create a new <a id="ref-for-css-style-sheet②②"></a>[CSS style sheet](#css-style-sheet) object and set its properties as specified.

2.  Then run the <a id="ref-for-add-a-css-style-sheet①"></a>[add a CSS style sheet](#add-a-css-style-sheet) steps for the newly created <a id="ref-for-css-style-sheet②③"></a>[CSS style sheet](#css-style-sheet).

    If the

    <strong>Warning:</strong>

    <a id="ref-for-concept-css-style-sheet-origin-clean-flag④"></a>[origin-clean flag](#concept-css-style-sheet-origin-clean-flag) is unset, this can expose information from the user’s intranet.

To <a id="add-a-css-style-sheet"></a><strong>add a CSS style sheet</strong>, run these steps:

1.  Add the <a id="ref-for-css-style-sheet②④"></a>[CSS style sheet](#css-style-sheet) to the list of <a id="ref-for-documentorshadowroot-document-or-shadow-root-css-style-sheets①"></a>[document or shadow root CSS style sheets](#documentorshadowroot-document-or-shadow-root-css-style-sheets) at the appropriate location.

2.  If the <a id="ref-for-css-style-sheet②⑤"></a>[CSS style sheet](#css-style-sheet)’s <a id="ref-for-concept-css-style-sheet-owner-node⑦"></a>[owner node](#concept-css-style-sheet-owner-node) <a id="ref-for-contributes-a-script-blocking-style-sheet"></a>[contributes a script-blocking style sheet](https://html.spec.whatwg.org/multipage/semantics.html#contributes-a-script-blocking-style-sheet), then user agents must <a id="ref-for-list-append"></a>[append](https://infra.spec.whatwg.org/#list-append) the <a id="ref-for-concept-css-style-sheet-owner-node⑧"></a>owner node to its <a id="ref-for-concept-node-document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)’s <a id="ref-for-script-blocking-style-sheet-set"></a>[script-blocking style sheet set](https://html.spec.whatwg.org/multipage/semantics.html#script-blocking-style-sheet-set).

    The remainder of these steps deal with the

    <strong>Note:</strong>

    <a id="ref-for-concept-css-style-sheet-disabled-flag④"></a>[disabled flag](#concept-css-style-sheet-disabled-flag).

3.  If the <a id="ref-for-concept-css-style-sheet-disabled-flag⑤"></a>[disabled flag](#concept-css-style-sheet-disabled-flag) is set, then return.

4.  If the <a id="ref-for-concept-css-style-sheet-title⑦"></a>[title](#concept-css-style-sheet-title) is not the empty string, the <a id="ref-for-concept-css-style-sheet-alternate-flag②"></a>[alternate flag](#concept-css-style-sheet-alternate-flag) is unset, and <a id="ref-for-preferred-css-style-sheet-set-name"></a>[preferred CSS style sheet set name](#preferred-css-style-sheet-set-name) is the empty string <a id="ref-for-change-the-preferred-css-style-sheet-set-name"></a>[change the preferred CSS style sheet set name](#change-the-preferred-css-style-sheet-set-name) to the <a id="ref-for-concept-css-style-sheet-title⑧"></a>title.

5.  If any of the following is true, then unset the <a id="ref-for-concept-css-style-sheet-disabled-flag⑥"></a>[disabled flag](#concept-css-style-sheet-disabled-flag) and return:
    - The <a id="ref-for-concept-css-style-sheet-title⑨"></a>[title](#concept-css-style-sheet-title) is the empty string.
    - The <a id="ref-for-last-css-style-sheet-set-name"></a>[last CSS style sheet set name](#last-css-style-sheet-set-name) is null and the <a id="ref-for-concept-css-style-sheet-title①⓪"></a>[title](#concept-css-style-sheet-title) is a <a id="ref-for-dfn-case-sensitive②"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for the <a id="ref-for-preferred-css-style-sheet-set-name①"></a>[preferred CSS style sheet set name](#preferred-css-style-sheet-set-name).
    - The <a id="ref-for-concept-css-style-sheet-title①①"></a>[title](#concept-css-style-sheet-title) is a <a id="ref-for-dfn-case-sensitive③"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for the <a id="ref-for-last-css-style-sheet-set-name①"></a>[last CSS style sheet set name](#last-css-style-sheet-set-name).

6.  Set the <a id="ref-for-concept-css-style-sheet-disabled-flag⑦"></a>[disabled flag](#concept-css-style-sheet-disabled-flag).

Tests

- [preferred-stylesheet-order.html](https://wpt.fyi/results/css/cssom/preferred-stylesheet-order.html) [(live test)](http://wpt.live/css/cssom/preferred-stylesheet-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/preferred-stylesheet-order.html)
- [preferred-stylesheet-reversed-order.html](https://wpt.fyi/results/css/cssom/preferred-stylesheet-reversed-order.html) [(live test)](http://wpt.live/css/cssom/preferred-stylesheet-reversed-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/preferred-stylesheet-reversed-order.html)

To <a id="remove-a-css-style-sheet"></a><strong>remove a CSS style sheet</strong>, run these steps:

1.  Remove the <a id="ref-for-css-style-sheet②⑥"></a>[CSS style sheet](#css-style-sheet) from the list of <a id="ref-for-documentorshadowroot-document-or-shadow-root-css-style-sheets②"></a>[document or shadow root CSS style sheets](#documentorshadowroot-document-or-shadow-root-css-style-sheets).
2.  Set the <a id="ref-for-css-style-sheet②⑦"></a>[CSS style sheet](#css-style-sheet)’s <a id="ref-for-concept-css-style-sheet-parent-css-style-sheet③"></a>[parent CSS style sheet](#concept-css-style-sheet-parent-css-style-sheet), <a id="ref-for-concept-css-style-sheet-owner-node⑨"></a>[owner node](#concept-css-style-sheet-owner-node) and <a id="ref-for-concept-css-style-sheet-owner-css-rule②"></a>[owner CSS rule](#concept-css-style-sheet-owner-css-rule) to null.

Tests

- [delete-namespace-rule-when-child-rule-exists.html](https://wpt.fyi/results/css/cssom/delete-namespace-rule-when-child-rule-exists.html) [(live test)](http://wpt.live/css/cssom/delete-namespace-rule-when-child-rule-exists.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/delete-namespace-rule-when-child-rule-exists.html)

A <a id="persistent-css-style-sheet"></a><strong>persistent CSS style sheet</strong> is a <a id="ref-for-css-style-sheet②⑧"></a>[CSS style sheet](#css-style-sheet) from the <a id="ref-for-documentorshadowroot-document-or-shadow-root-css-style-sheets③"></a>[document or shadow root CSS style sheets](#documentorshadowroot-document-or-shadow-root-css-style-sheets) whose <a id="ref-for-concept-css-style-sheet-title①②"></a>[title](#concept-css-style-sheet-title) is the empty string and whose <a id="ref-for-concept-css-style-sheet-alternate-flag③"></a>[alternate flag](#concept-css-style-sheet-alternate-flag) is unset.

A <a id="css-style-sheet-set"></a><strong>CSS style sheet set</strong> is an ordered collection of one or more <a id="ref-for-css-style-sheet②⑨"></a>[CSS style sheets](#css-style-sheet) from the <a id="ref-for-documentorshadowroot-document-or-shadow-root-css-style-sheets④"></a>[document or shadow root CSS style sheets](#documentorshadowroot-document-or-shadow-root-css-style-sheets) which have an identical <a id="ref-for-concept-css-style-sheet-title①③"></a>[title](#concept-css-style-sheet-title) that is not the empty string.

A <a id="css-style-sheet-set-name"></a><strong>CSS style sheet set name</strong> is the <a id="ref-for-concept-css-style-sheet-title①④"></a>[title](#concept-css-style-sheet-title) the <a id="ref-for-css-style-sheet-set"></a>[CSS style sheet set](#css-style-sheet-set) has in common.

An <a id="enabled-css-style-sheet-set"></a><strong>enabled CSS style sheet set</strong> is a <a id="ref-for-css-style-sheet-set①"></a>[CSS style sheet set](#css-style-sheet-set) of which each <a id="ref-for-css-style-sheet③⓪"></a>[CSS style sheet](#css-style-sheet) has its <a id="ref-for-concept-css-style-sheet-disabled-flag⑧"></a>[disabled flag](#concept-css-style-sheet-disabled-flag) unset.

To <a id="enable-a-css-style-sheet-set"></a><strong>enable a CSS style sheet set</strong> with name <var>name</var>, run these steps:

1.  If <var>name</var> is the empty string, set the <a id="ref-for-concept-css-style-sheet-disabled-flag⑨"></a>[disabled flag](#concept-css-style-sheet-disabled-flag) for each <a id="ref-for-css-style-sheet③①"></a>[CSS style sheet](#css-style-sheet) that is in a <a id="ref-for-css-style-sheet-set②"></a>[CSS style sheet set](#css-style-sheet-set) and return.
2.  Unset the <a id="ref-for-concept-css-style-sheet-disabled-flag①⓪"></a>[disabled flag](#concept-css-style-sheet-disabled-flag) for each <a id="ref-for-css-style-sheet③②"></a>[CSS style sheet](#css-style-sheet) in a <a id="ref-for-css-style-sheet-set③"></a>[CSS style sheet set](#css-style-sheet-set) whose <a id="ref-for-css-style-sheet-set-name"></a>[CSS style sheet set name](#css-style-sheet-set-name) is a <a id="ref-for-dfn-case-sensitive④"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for <var>name</var> and set it for all other <a id="ref-for-css-style-sheet③③"></a>CSS style sheets in a <a id="ref-for-css-style-sheet-set④"></a>CSS style sheet set.

To <a id="select-a-css-style-sheet-set"></a><strong>select a CSS style sheet set</strong> with name <var>name</var>, run these steps:

1.  <a id="ref-for-enable-a-css-style-sheet-set"></a>[enable a CSS style sheet set](#enable-a-css-style-sheet-set) with name <var>name</var>.
2.  Set <a id="ref-for-last-css-style-sheet-set-name②"></a>[last CSS style sheet set name](#last-css-style-sheet-set-name) to <var>name</var>.

A <a id="last-css-style-sheet-set-name"></a><strong>last CSS style sheet set name</strong> is a concept to determine what <a id="ref-for-css-style-sheet-set⑤"></a>[CSS style sheet set](#css-style-sheet-set) was last <a id="ref-for-select-a-css-style-sheet-set"></a>[selected](#select-a-css-style-sheet-set). Initially its value is null.

A <a id="preferred-css-style-sheet-set-name"></a><strong>preferred CSS style sheet set name</strong> is a concept to determine which <a id="ref-for-css-style-sheet③④"></a>[CSS style sheets](#css-style-sheet) need to have their <a id="ref-for-concept-css-style-sheet-disabled-flag①①"></a>[disabled flag](#concept-css-style-sheet-disabled-flag) unset. Initially its value is the empty string.

To <a id="change-the-preferred-css-style-sheet-set-name"></a><strong>change the preferred CSS style sheet set name</strong> with name <var>name</var>, run these steps:

1.  Let <var>current</var> be the <a id="ref-for-preferred-css-style-sheet-set-name②"></a>[preferred CSS style sheet set name](#preferred-css-style-sheet-set-name).
2.  Set <a id="ref-for-preferred-css-style-sheet-set-name③"></a>[preferred CSS style sheet set name](#preferred-css-style-sheet-set-name) to <var>name</var>.
3.  If <var>name</var> is not a <a id="ref-for-dfn-case-sensitive⑤"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for <var>current</var> and <a id="ref-for-last-css-style-sheet-set-name③"></a>[last CSS style sheet set name](#last-css-style-sheet-set-name) is null <a id="ref-for-enable-a-css-style-sheet-set①"></a>[enable a CSS style sheet set](#enable-a-css-style-sheet-set) with name <var>name</var>.

#### <a id="the-http-default-style-header"></a>6.2.1. The HTTP Default-Style Header[](#the-http-default-style-header)

The HTTP <a id="ref-for-http-default-style"></a>[Default-Style](#http-default-style) header can be used to set the <a id="ref-for-preferred-css-style-sheet-set-name④"></a>[preferred CSS style sheet set name](#preferred-css-style-sheet-set-name) influencing which <a id="ref-for-css-style-sheet-set⑥"></a>[CSS style sheet set](#css-style-sheet-set) is (initially) the <a id="ref-for-enabled-css-style-sheet-set"></a>[enabled CSS style sheet set](#enabled-css-style-sheet-set).

For each HTTP <a id="ref-for-http-default-style①"></a>[Default-Style](#http-default-style) header, in header order, the user agent must <a id="ref-for-change-the-preferred-css-style-sheet-set-name①"></a>[change the preferred CSS style sheet set name](#change-the-preferred-css-style-sheet-set-name) with name being the value of the header.

#### <a id="the-stylesheetlist-interface"></a>6.2.2. The <code><a id="ref-for-stylesheetlist"></a>[StyleSheetList](#stylesheetlist)</code> Interface[](#the-stylesheetlist-interface)

The <code><a id="ref-for-stylesheetlist①"></a>[StyleSheetList](#stylesheetlist)</code> interface represents an ordered collection of <a id="ref-for-css-style-sheet③⑤"></a>[CSS style sheets](#css-style-sheet).

<a id="ref-for-Exposed③"></a><a id="stylesheetlist"></a><a id="ref-for-cssstylesheet①⓪"></a><a id="ref-for-dom-stylesheetlist-item"></a><a id="ref-for-idl-unsigned-long⑦"></a><a id="dom-stylesheetlist-item-index-index"></a><a id="ref-for-idl-unsigned-long⑧"></a><a id="ref-for-dom-stylesheetlist-length"></a>

``` text
[Exposed=Window]
interface StyleSheetList {
  getter CSSStyleSheet? item(unsigned long index);
  readonly attribute unsigned long length;
};
```

The object’s <a id="ref-for-dfn-supported-property-indices②"></a>[supported property indices](http://heycam.github.io/webidl/#dfn-supported-property-indices) are the numbers in the range zero to one less than the number of <a id="ref-for-css-style-sheet③⑥"></a>[CSS style sheets](#css-style-sheet) represented by the collection. If there are no such <a id="ref-for-css-style-sheet③⑦"></a>CSS style sheets, then there are no <a id="ref-for-dfn-supported-property-indices③"></a>supported property indices.

The <a id="dom-stylesheetlist-item"></a><strong><code>item(<var>index</var>)</code></strong> method must return the <var>index</var>th <a id="ref-for-css-style-sheet③⑧"></a>[CSS style sheet](#css-style-sheet) in the collection. If there is no <var>index</var>th object in the collection, then the method must return null.

The <a id="dom-stylesheetlist-length"></a><strong><code>length</code></strong> attribute must return the number of <a id="ref-for-css-style-sheet③⑨"></a>[CSS style sheets](#css-style-sheet) represented by the collection.

Tests

- [StyleSheetList-constructable-with-style-recalc.html](https://wpt.fyi/results/css/cssom/StyleSheetList-constructable-with-style-recalc.html) [(live test)](http://wpt.live/css/cssom/StyleSheetList-constructable-with-style-recalc.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/StyleSheetList-constructable-with-style-recalc.html)
- [StyleSheetList-constructable.html](https://wpt.fyi/results/css/cssom/StyleSheetList-constructable.html) [(live test)](http://wpt.live/css/cssom/StyleSheetList-constructable.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/StyleSheetList-constructable.html)
- [StyleSheetList.html](https://wpt.fyi/results/css/cssom/StyleSheetList.html) [(live test)](http://wpt.live/css/cssom/StyleSheetList.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/StyleSheetList.html)

#### <a id="extensions-to-the-document-or-shadow-root-interface"></a>6.2.3. Extensions to the <code><a id="ref-for-documentorshadowroot⑤"></a>[DocumentOrShadowRoot](https://dom.spec.whatwg.org/#documentorshadowroot)</code> Interface Mixin[](#extensions-to-the-document-or-shadow-root-interface)

<a id="ref-for-documentorshadowroot⑥"></a><a id="ref-for-SameObject③"></a><a id="ref-for-stylesheetlist②"></a><a id="ref-for-dom-documentorshadowroot-stylesheets"></a><a id="ref-for-idl-observable-array"></a><a id="ref-for-cssstylesheet①①"></a><a id="ref-for-dom-documentorshadowroot-adoptedstylesheets①"></a>

``` text
partial interface mixin DocumentOrShadowRoot {
  [SameObject] readonly attribute StyleSheetList styleSheets;
  attribute ObservableArray<CSSStyleSheet> adoptedStyleSheets;
};
```

The <a id="dom-documentorshadowroot-stylesheets"></a><strong><code>styleSheets</code></strong> attribute must return a <code><a id="ref-for-stylesheetlist③"></a>[StyleSheetList](#stylesheetlist)</code> collection representing the <a id="ref-for-documentorshadowroot-document-or-shadow-root-css-style-sheets⑤"></a>[document or shadow root CSS style sheets](#documentorshadowroot-document-or-shadow-root-css-style-sheets).

The <a id="ref-for-observable-array-attribute-set-an-indexed-value"></a>[set an indexed value](https://webidl.spec.whatwg.org/#observable-array-attribute-set-an-indexed-value) algorithm for <a id="dom-documentorshadowroot-adoptedstylesheets"></a><strong><code>adoptedStyleSheets</code></strong>, given <var>value</var> and <var>index</var>, is the following:

1.  If <var>value</var>’s <a id="ref-for-concept-css-style-sheet-constructed-flag⑥"></a>[constructed flag](#concept-css-style-sheet-constructed-flag) is not set, or its <a id="ref-for-concept-css-style-sheet-constructor-document①"></a>[constructor document](#concept-css-style-sheet-constructor-document) is not equal to this <code><a id="ref-for-documentorshadowroot⑦"></a>[DocumentOrShadowRoot](https://dom.spec.whatwg.org/#documentorshadowroot)</code>’s <a id="ref-for-concept-node-document①"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document), throw a "<code><a id="ref-for-notallowederror④"></a>[NotAllowedError](https://webidl.spec.whatwg.org/#notallowederror)</code>" <code><a id="ref-for-idl-DOMException⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

Tests

- [adoptedstylesheets-modify-array-and-sheet.html](https://wpt.fyi/results/css/cssom/adoptedstylesheets-modify-array-and-sheet.html) [(live test)](http://wpt.live/css/cssom/adoptedstylesheets-modify-array-and-sheet.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/adoptedstylesheets-modify-array-and-sheet.html)
- [adoptedstylesheets-observablearray.html](https://wpt.fyi/results/css/cssom/adoptedstylesheets-observablearray.html) [(live test)](http://wpt.live/css/cssom/adoptedstylesheets-observablearray.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/adoptedstylesheets-observablearray.html)
- [ttwf-cssom-doc-ext-load-count.html](https://wpt.fyi/results/css/cssom/ttwf-cssom-doc-ext-load-count.html) [(live test)](http://wpt.live/css/cssom/ttwf-cssom-doc-ext-load-count.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/ttwf-cssom-doc-ext-load-count.html)
- [ttwf-cssom-doc-ext-load-tree-order.html](https://wpt.fyi/results/css/cssom/ttwf-cssom-doc-ext-load-tree-order.html) [(live test)](http://wpt.live/css/cssom/ttwf-cssom-doc-ext-load-tree-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/ttwf-cssom-doc-ext-load-tree-order.html)
- [ttwf-cssom-document-extension.html](https://wpt.fyi/results/css/cssom/ttwf-cssom-document-extension.html) [(live test)](http://wpt.live/css/cssom/ttwf-cssom-document-extension.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/ttwf-cssom-document-extension.html)

### <a id="style-sheet-association"></a>6.3. Style Sheet Association[](#style-sheet-association)

This section defines the interface an <a id="ref-for-concept-css-style-sheet-owner-node①⓪"></a>[owner node](#concept-css-style-sheet-owner-node) of a <a id="ref-for-css-style-sheet④⓪"></a>[CSS style sheet](#css-style-sheet) has to implement and defines the requirements for <a id="ref-for-dt-xml-stylesheet"></a>[xml-stylesheet processing instructions](https://www.w3.org/TR/xml-stylesheet/#dt-xml-stylesheet) and HTTP <code>Link</code> headers when the link relation type is an <a id="ref-for-ascii-case-insensitive"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "<code>stylesheet</code>".

#### <a id="fetching-css-style-sheets"></a>6.3.1. Fetching CSS style sheets[](#fetching-css-style-sheets)

To <a id="fetch-a-css-style-sheet"></a><strong>fetch a CSS style sheet</strong> with parsed URL <var>parsed URL</var>, referrer <var>referrer</var>, document <var>document</var>, optionally a set of parameters <var>parameters</var> (used as input to creating a <a id="ref-for-concept-request"></a>[request](https://fetch.spec.whatwg.org/#concept-request)), and an algorithm for handling the response result <var>processTheResponse</var> that takes a response, follow these steps:

1.  Let <var>origin</var> be <var>document</var>’s <a id="ref-for-concept-origin"></a>[origin](https://html.spec.whatwg.org/multipage/browsers.html#concept-origin).
2.  Let <var>request</var> be a new <a id="ref-for-concept-request①"></a>[request](https://fetch.spec.whatwg.org/#concept-request), with the <a id="ref-for-concept-request-url"></a>[url](https://fetch.spec.whatwg.org/#concept-request-url) <var>parsed URL</var>, <a id="ref-for-concept-request-origin"></a>[origin](https://fetch.spec.whatwg.org/#concept-request-origin) <var>origin</var>, <a id="ref-for-concept-request-referrer"></a>[referrer](https://fetch.spec.whatwg.org/#concept-request-referrer) <var>referrer</var>, and if specified the set of parameters <var>parameters</var>.
3.  <a id="ref-for-concept-fetch"></a>[Fetch](https://fetch.spec.whatwg.org/#concept-fetch) <var>request</var>, with <var>processResponseEndOfBody</var>, given <var>response</var>, being the following steps:
    1.  If <var>response</var> is a <a id="ref-for-concept-network-error"></a>[network error](https://fetch.spec.whatwg.org/#concept-network-error), return.
    2.  If <var>document</var> is in <a id="ref-for-concept-document-quirks"></a>[quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks), <var>response</var> is <a id="ref-for-cors-same-origin"></a>[CORS-same-origin](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#cors-same-origin) and the <a id="ref-for-content-type"></a>[Content-Type metadata](https://html.spec.whatwg.org/multipage/infrastructure.html#content-type) of <var>response</var> is not a <a id="ref-for-supported-styling-language"></a>[supported styling language](#supported-styling-language) change the <a id="ref-for-content-type①"></a>Content-Type metadata of <var>response</var> to <code>text/css</code>.
    3.  If <var>response</var> is not in a <a id="ref-for-supported-styling-language①"></a>[supported styling language](#supported-styling-language), return.
    4.  Execute <var>processTheResponse</var> given <var>response</var>

#### <a id="the-linkstyle-interface"></a>6.3.2. The <code><a id="ref-for-linkstyle"></a>[LinkStyle](#linkstyle)</code> Interface[](#the-linkstyle-interface)

The <a id="associated-css-style-sheet"></a><strong>associated CSS style sheet</strong> of a node is the <a id="ref-for-css-style-sheet④①"></a>[CSS style sheet](#css-style-sheet) in the list of <a id="ref-for-documentorshadowroot-document-or-shadow-root-css-style-sheets⑥"></a>[document or shadow root CSS style sheets](#documentorshadowroot-document-or-shadow-root-css-style-sheets) of which the <a id="ref-for-concept-css-style-sheet-owner-node①①"></a>[owner node](#concept-css-style-sheet-owner-node) is said node. This node must also implement the <code><a id="ref-for-linkstyle①"></a>[LinkStyle](#linkstyle)</code> interface.

<a id="linkstyle"></a><a id="ref-for-cssstylesheet①②"></a><a id="ref-for-dom-linkstyle-sheet"></a>

``` text
interface mixin LinkStyle {
  readonly attribute CSSStyleSheet? sheet;
};
```

The <a id="dom-linkstyle-sheet"></a><strong><code>sheet</code></strong> attribute must return the <a id="ref-for-associated-css-style-sheet①"></a>[associated CSS style sheet](#associated-css-style-sheet) for the node or null if there is no <a id="ref-for-associated-css-style-sheet②"></a>associated CSS style sheet.

<a id="example-3accc107"></a>

<strong>Example:</strong>

[](#example-3accc107) In the following fragment, the first <code><a id="ref-for-the-style-element"></a>[style](https://html.spec.whatwg.org/multipage/semantics.html#the-style-element)</code> element has a <code><a id="ref-for-dom-linkstyle-sheet①"></a>[sheet](#dom-linkstyle-sheet)</code> attribute that returns a <code><a id="ref-for-stylesheet③"></a>[StyleSheet](#stylesheet)</code> object representing the style sheet, but for the second <code><a id="ref-for-the-style-element①"></a>[style](https://html.spec.whatwg.org/multipage/semantics.html#the-style-element)</code> element, the <code><a id="ref-for-dom-linkstyle-sheet②"></a>[sheet](#dom-linkstyle-sheet)</code> attribute returns null, assuming the user agent supports CSS (<code>text/css</code>), but does not support the (hypothetical) ExampleSheets (<code>text/example-sheets</code>).

``` text
<style type="text/css">
  body { background:lime }
</style>
```

``` text
<style type="text/example-sheets">
  $(body).background := lime
</style>
```

Note: Whether or not the node refers to a style sheet is defined by the specification that defines the semantics of said node.

Tests

- [HTMLLinkElement-disabled-001.html](https://wpt.fyi/results/css/cssom/HTMLLinkElement-disabled-001.html) [(live test)](http://wpt.live/css/cssom/HTMLLinkElement-disabled-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLLinkElement-disabled-001.html)
- [HTMLLinkElement-disabled-002.html](https://wpt.fyi/results/css/cssom/HTMLLinkElement-disabled-002.html) [(live test)](http://wpt.live/css/cssom/HTMLLinkElement-disabled-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLLinkElement-disabled-002.html)
- [HTMLLinkElement-disabled-003.html](https://wpt.fyi/results/css/cssom/HTMLLinkElement-disabled-003.html) [(live test)](http://wpt.live/css/cssom/HTMLLinkElement-disabled-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLLinkElement-disabled-003.html)
- [HTMLLinkElement-disabled-004.html](https://wpt.fyi/results/css/cssom/HTMLLinkElement-disabled-004.html) [(live test)](http://wpt.live/css/cssom/HTMLLinkElement-disabled-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLLinkElement-disabled-004.html)
- [HTMLLinkElement-disabled-005.html](https://wpt.fyi/results/css/cssom/HTMLLinkElement-disabled-005.html) [(live test)](http://wpt.live/css/cssom/HTMLLinkElement-disabled-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLLinkElement-disabled-005.html)
- [HTMLLinkElement-disabled-006.html](https://wpt.fyi/results/css/cssom/HTMLLinkElement-disabled-006.html) [(live test)](http://wpt.live/css/cssom/HTMLLinkElement-disabled-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLLinkElement-disabled-006.html)
- [HTMLLinkElement-disabled-007.html](https://wpt.fyi/results/css/cssom/HTMLLinkElement-disabled-007.html) [(live test)](http://wpt.live/css/cssom/HTMLLinkElement-disabled-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLLinkElement-disabled-007.html)
- [HTMLLinkElement-disabled-alternate.html](https://wpt.fyi/results/css/cssom/HTMLLinkElement-disabled-alternate.html) [(live test)](http://wpt.live/css/cssom/HTMLLinkElement-disabled-alternate.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLLinkElement-disabled-alternate.html)
- [HTMLLinkElement-load-event-002.html](https://wpt.fyi/results/css/cssom/HTMLLinkElement-load-event-002.html) [(live test)](http://wpt.live/css/cssom/HTMLLinkElement-load-event-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLLinkElement-load-event-002.html)
- [HTMLLinkElement-load-event.html](https://wpt.fyi/results/css/cssom/HTMLLinkElement-load-event.html) [(live test)](http://wpt.live/css/cssom/HTMLLinkElement-load-event.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLLinkElement-load-event.html)
- [HTMLStyleElement-load-event.html](https://wpt.fyi/results/css/cssom/HTMLStyleElement-load-event.html) [(live test)](http://wpt.live/css/cssom/HTMLStyleElement-load-event.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/HTMLStyleElement-load-event.html)

#### <a id="requirements-on-specifications"></a>6.3.3. Requirements on specifications[](#requirements-on-specifications)

Specifications introducing new ways of associating style sheets through the DOM should define which nodes implement the <code><a id="ref-for-linkstyle②"></a>[LinkStyle](#linkstyle)</code> interface. When doing so, they must also define when a <a id="ref-for-css-style-sheet④②"></a>[CSS style sheet](#css-style-sheet) is <a id="ref-for-create-a-css-style-sheet"></a>[created](#create-a-css-style-sheet).

#### <a id="requirements-on-user-agents-implementing-the-xml-stylesheet-processing-instruction"></a>6.3.4. Requirements on user agents Implementing the xml-stylesheet processing instruction[](#requirements-on-user-agents-implementing-the-xml-stylesheet-processing-instruction)

<a id="ref-for-processinginstruction①"></a><a id="ref-for-linkstyle③"></a>

``` text
ProcessingInstruction includes LinkStyle;
```

The <a id="prolog"></a><strong>prolog</strong> refers to <a id="ref-for-concept-node"></a>[nodes](https://dom.spec.whatwg.org/#concept-node) that are children of the <code><a id="ref-for-document①"></a>[Document](https://dom.spec.whatwg.org/#document)</code> and are not <a id="ref-for-concept-tree-following"></a>[following](https://dom.spec.whatwg.org/#concept-tree-following) the <code><a id="ref-for-element①"></a>[Element](https://dom.spec.whatwg.org/#element)</code> child of the <code><a id="ref-for-document②"></a>[Document](https://dom.spec.whatwg.org/#document)</code>, if any.

When a <code>ProcessingInstruction</code> <a id="ref-for-boundary-point-node"></a>[node](https://dom.spec.whatwg.org/#boundary-point-node) <var>node</var> becomes part of the <a id="ref-for-prolog"></a>[prolog](#prolog), is no longer part of the <a id="ref-for-prolog①"></a>prolog, or has its <a id="ref-for-concept-cd-data"></a>[data](https://dom.spec.whatwg.org/#concept-cd-data) changed, these steps must be run:

1.  If an instance of this algorithm is currently running for <var>node</var>, abort that instance, and stop the associated <a id="ref-for-concept-fetch①"></a>[fetching](https://fetch.spec.whatwg.org/#concept-fetch) if applicable.
2.  If <var>node</var> has an <a id="ref-for-associated-css-style-sheet③"></a>[associated CSS style sheet](#associated-css-style-sheet), <a id="ref-for-remove-a-css-style-sheet"></a>[remove](#remove-a-css-style-sheet) it.
3.  If <var>node</var> is not an <a id="ref-for-dt-xml-stylesheet①"></a>[xml-stylesheet processing instruction](https://www.w3.org/TR/xml-stylesheet/#dt-xml-stylesheet), then return.
4.  If <var>node</var> does not have an <code>href</code> <a id="ref-for-dt-pseudo-attribute"></a>[pseudo-attribute](https://www.w3.org/TR/xml-stylesheet/#dt-pseudo-attribute), then return.
5.  Let <var>title</var> be the value of the <code>title</code> <a id="ref-for-dt-pseudo-attribute①"></a>[pseudo-attribute](https://www.w3.org/TR/xml-stylesheet/#dt-pseudo-attribute) or the empty string if the <code>title</code> <a id="ref-for-dt-pseudo-attribute②"></a>pseudo-attribute is not specified.
6.  If there is an <code>alternate</code> <a id="ref-for-dt-pseudo-attribute③"></a>[pseudo-attribute](https://www.w3.org/TR/xml-stylesheet/#dt-pseudo-attribute) whose value is a <a id="ref-for-dfn-case-sensitive⑥"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for "<code>yes</code>" and <var>title</var> is the empty string, then return.
7.  If there is a <code>type</code> <a id="ref-for-dt-pseudo-attribute④"></a>[pseudo-attribute](https://www.w3.org/TR/xml-stylesheet/#dt-pseudo-attribute) whose value is not a <a id="ref-for-supported-styling-language②"></a>[supported styling language](#supported-styling-language) the user agent may return.
8.  Let <var>input URL</var> be the value specified by the <code>href</code> <a id="ref-for-dt-pseudo-attribute⑤"></a>[pseudo-attribute](https://www.w3.org/TR/xml-stylesheet/#dt-pseudo-attribute).
9.  Let <var>document</var> be <var>node</var>’s <a id="ref-for-concept-node-document②"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)
10. Let <var>base URL</var> be <var>document</var>’s <a id="ref-for-document-base-url"></a>[document base URL](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#document-base-url).
11. Let <var>referrer</var> be <var>document</var>’s <a id="ref-for-concept-document-url"></a>[address](https://dom.spec.whatwg.org/#concept-document-url).
12. Let <var>parsed URL</var> be the return value of invoking the <a id="ref-for-concept-url-parser"></a>[URL parser](https://url.spec.whatwg.org/#concept-url-parser) with the string <var>input URL</var> and the base URL <var>base URL</var>.
13. If <var>parsed URL</var> is failure, then return.
14. <a id="ref-for-fetch-a-css-style-sheet"></a>[Fetch a CSS style sheet](#fetch-a-css-style-sheet) with parsed URL <var>parsed URL</var>, referrer <var>referrer</var>, document <var>document</var>, and <var>processTheResponse</var> given <var>response</var> being the following steps:
    1.  <a id="ref-for-create-a-css-style-sheet①"></a>[Create a CSS style sheet](#create-a-css-style-sheet) with the following properties:

        <strong><a id="ref-for-concept-css-style-sheet-location②"></a>[location](#concept-css-style-sheet-location)</strong>

        The result of invoking the <a id="ref-for-concept-url-serializer"></a>[URL serializer](https://url.spec.whatwg.org/#concept-url-serializer) with <var>parsed URL</var>.
        <strong><a id="ref-for-concept-css-style-sheet-parent-css-style-sheet④"></a>[parent CSS style sheet](#concept-css-style-sheet-parent-css-style-sheet)</strong>

        null.
        <strong><a id="ref-for-concept-css-style-sheet-owner-node①②"></a>[owner node](#concept-css-style-sheet-owner-node)</strong>

        <var>node</var>.
        <strong><a id="ref-for-concept-css-style-sheet-owner-css-rule③"></a>[owner CSS rule](#concept-css-style-sheet-owner-css-rule)</strong>

        null.
        <strong><a id="ref-for-concept-css-style-sheet-media⑦"></a>[media](#concept-css-style-sheet-media)</strong>

        The value of the <code>media</code> <a id="ref-for-dt-pseudo-attribute⑥"></a>[pseudo-attribute](https://www.w3.org/TR/xml-stylesheet/#dt-pseudo-attribute) if any, or the empty string otherwise.
        <strong><a id="ref-for-concept-css-style-sheet-title①⑤"></a>[title](#concept-css-style-sheet-title)</strong>

        <var>title</var>.
        <strong><a id="ref-for-concept-css-style-sheet-alternate-flag④"></a>[alternate flag](#concept-css-style-sheet-alternate-flag)</strong>

        Set if the <code>alternate</code> <a id="ref-for-dt-pseudo-attribute⑦"></a>[pseudo-attribute](https://www.w3.org/TR/xml-stylesheet/#dt-pseudo-attribute) value is a <a id="ref-for-dfn-case-sensitive⑦"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for "<code>yes</code>", or unset otherwise.
        <strong><a id="ref-for-concept-css-style-sheet-origin-clean-flag⑤"></a>[origin-clean flag](#concept-css-style-sheet-origin-clean-flag)</strong>

        Set if <var>response</var> is <a id="ref-for-cors-same-origin①"></a>[CORS-same-origin](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#cors-same-origin), or unset otherwise.
        The CSS <a id="ref-for-environment-encoding"></a>[environment encoding](https://drafts.csswg.org/css-syntax-3/#environment-encoding) is the result of running the following steps:

        1.  If the element has a <code>charset</code> <a id="ref-for-dt-pseudo-attribute⑧"></a>[pseudo-attribute](https://www.w3.org/TR/xml-stylesheet/#dt-pseudo-attribute), <a id="ref-for-concept-encoding-get"></a>[get an encoding](https://encoding.spec.whatwg.org/#concept-encoding-get) from that pseudo-attribute’s value. If that succeeds, return the resulting encoding and abort these steps.
        2.  Otherwise, return the <a id="ref-for-concept-document-encoding"></a>[document’s character encoding](https://dom.spec.whatwg.org/#concept-document-encoding). [\[DOM\]](#biblio-dom)

#### <a id="requirements-on-user-agents-implementing-the-http-link-header"></a>6.3.5. Requirements on user agents Implementing the HTTP Link Header[](#requirements-on-user-agents-implementing-the-http-link-header)

For each HTTP <code>Link</code> header of which one of the link relation types is an <a id="ref-for-ascii-case-insensitive①"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "<code>stylesheet</code>" these steps must be run:

1.  Let <var>title</var> be the value of the first of all the <code>title</code> parameters. If there are no such parameters it is the empty string.

2.  If one of the (other) link relation types is an <a id="ref-for-ascii-case-insensitive②"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "<code>alternate</code>" and <var>title</var> is the empty string, then return.

3.  Let <var>input URL</var> be the value specified.

    <a id="issue-d4a93110"></a>

    <strong>Issue:</strong>

    [](#issue-d4a93110) Be more specific

4.  Let <var>base URL</var> be the document’s <a id="ref-for-document-base-url①"></a>[document base URL](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#document-base-url).

    <a id="issue-af048285"></a>

    <strong>Issue:</strong>

    [](#issue-af048285) Is there a document at this point?

5.  Let <var>referrer</var> be the document’s <a id="ref-for-concept-document-url①"></a>[address](https://dom.spec.whatwg.org/#concept-document-url).

6.  Let <var>parsed URL</var> be the return value of invoking the <a id="ref-for-concept-url-parser①"></a>[URL parser](https://url.spec.whatwg.org/#concept-url-parser) with the string <var>input URL</var> and the base URL <var>base URL</var>.

7.  If <var>parsed URL</var> is failure, then return.

8.  <a id="ref-for-fetch-a-css-style-sheet①"></a>[Fetch a CSS style sheet](#fetch-a-css-style-sheet) with parsed URL <var>parsed URL</var>, referrer <var>referrer</var>, document being the document, and <var>processTheResponse</var>, given <var>response</var>, being the following steps:

    <a id="issue-45012e41"></a>

    <strong>Issue:</strong>

    [](#issue-45012e41) What if the HTML parser hasn’t decided on quirks/non-quirks yet?

    1.  <a id="ref-for-create-a-css-style-sheet②"></a>[Create a CSS style sheet](#create-a-css-style-sheet) with the following properties:
        <strong><a id="ref-for-concept-css-style-sheet-location③"></a>[location](#concept-css-style-sheet-location)</strong>

        The result of invoking the <a id="ref-for-concept-url-serializer①"></a>[URL serializer](https://url.spec.whatwg.org/#concept-url-serializer) with <var>parsed URL</var>.
        <strong><a id="ref-for-concept-css-style-sheet-owner-node①③"></a>[owner node](#concept-css-style-sheet-owner-node)</strong>

        null.
        <strong><a id="ref-for-concept-css-style-sheet-parent-css-style-sheet⑤"></a>[parent CSS style sheet](#concept-css-style-sheet-parent-css-style-sheet)</strong>

        null.
        <strong><a id="ref-for-concept-css-style-sheet-owner-css-rule④"></a>[owner CSS rule](#concept-css-style-sheet-owner-css-rule)</strong>

        null.
        <strong><a id="ref-for-concept-css-style-sheet-media⑧"></a>[media](#concept-css-style-sheet-media)</strong>

        The value of the first <code>media</code> parameter.
        <strong><a id="ref-for-concept-css-style-sheet-title①⑥"></a>[title](#concept-css-style-sheet-title)</strong>

        <var>title</var>.
        <strong><a id="ref-for-concept-css-style-sheet-alternate-flag⑤"></a>[alternate flag](#concept-css-style-sheet-alternate-flag)</strong>

        Set if one of the specified link relation type for this HTTP <code>Link</code> header is an <a id="ref-for-ascii-case-insensitive③"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "<code>alternate</code>", or false otherwise.
        <strong><a id="ref-for-concept-css-style-sheet-origin-clean-flag⑥"></a>[origin-clean flag](#concept-css-style-sheet-origin-clean-flag)</strong>

        Set if <var>response</var> is <a id="ref-for-cors-same-origin②"></a>[CORS-same-origin](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#cors-same-origin), or unset otherwise.

A style sheet referenced by a HTTP <code>Link</code> header using the rules in this section is said to be <a id="ref-for-a-style-sheet-that-is-blocking-scripts"></a>[a style sheet that is blocking scripts](https://html.spec.whatwg.org/multipage/semantics.html#a-style-sheet-that-is-blocking-scripts) if the style sheet was enabled when created, and the user agent hasn’t given up on that particular style sheet yet. A user agent may give up on such a style sheet at any time.

### <a id="css-rules"></a>6.4. CSS Rules[](#css-rules)

A <a id="css-rule"></a><strong>CSS rule</strong> is an abstract concept that denotes a rule as defined by the CSS specification. A <a id="ref-for-css-rule①"></a>[CSS rule](#css-rule) is represented as an object that implements a subclass of the <code><a id="ref-for-cssrule①"></a>[CSSRule](#cssrule)</code> interface, and which has the following associated state items:

<strong><a id="concept-css-rule-type"></a><strong>type</strong></strong>

A non-negative integer associated with a particular type of rule. This item is initialized when a rule is created and cannot change.

<strong><a id="concept-css-rule-text"></a><strong>text</strong></strong>

A text representation of the rule suitable for direct use in a style sheet. This item is initialized when a rule is created and can be changed.

<strong><a id="concept-css-rule-parent-css-rule"></a><strong>parent CSS rule</strong></strong>

A reference to an enclosing <a id="ref-for-css-rule②"></a>[CSS rule](#css-rule) or null. If the rule has an enclosing rule when it is created, then this item is initialized to the enclosing rule; otherwise it is null. It can be changed to null.

<strong><a id="concept-css-rule-parent-css-style-sheet"></a><strong>parent CSS style sheet</strong></strong>

A reference to a parent <a id="ref-for-css-style-sheet④③"></a>[CSS style sheet](#css-style-sheet) or null. This item is initialized to reference an associated style sheet when the rule is created. It can be changed to null.

<strong><a id="concept-css-rule-child-css-rules"></a><strong>child CSS rules</strong></strong>

A list of child <a id="ref-for-css-rule③"></a>[CSS rules](#css-rule). The list can be mutated.

In addition to the above state, each <a id="ref-for-css-rule④"></a>[CSS rule](#css-rule) may be associated with other state in accordance with its <a id="ref-for-concept-css-rule-type"></a>[type](#concept-css-rule-type).

To <a id="parse-a-css-rule"></a><strong>parse a CSS rule</strong> from a string <var>string</var>, run the following steps:

1.  Let <var>rule</var> be the return value of invoking <a id="ref-for-parse-a-rule①"></a>[parse a rule](https://drafts.csswg.org/css-syntax-3/#parse-a-rule) with <var>string</var>.
2.  If <var>rule</var> is a syntax error, return <var>rule</var>.
3.  Let <var>parsed rule</var> be the result of parsing <var>rule</var> according to the appropriate CSS specifications, dropping parts that are said to be ignored. If the whole <var>rule</var> is dropped, return a syntax error.
4.  Return <var>parsed rule</var>.

To <a id="serialize-a-css-rule"></a><strong>serialize a CSS rule</strong>, perform one of the following in accordance with the <a id="ref-for-css-rule⑤"></a>[CSS rule](#css-rule)’s <a id="ref-for-concept-css-rule-type①"></a>[type](#concept-css-rule-type):

<strong><code><a id="ref-for-cssstylerule"></a>[CSSStyleRule](#cssstylerule)</code></strong>

Return the result of the following steps:

1.  Let <var>s</var> initially be the result of performing <a id="ref-for-serialize-a-group-of-selectors①"></a>[serialize a group of selectors](#serialize-a-group-of-selectors) on the rule’s associated selectors, followed by the string "<code> {</code>", i.e., a single SPACE (U+0020), followed by LEFT CURLY BRACKET (U+007B).
2.  Let <var>decls</var> be the result of performing <a id="ref-for-serialize-a-css-declaration-block"></a>[serialize a CSS declaration block](#serialize-a-css-declaration-block) on the rule’s associated declarations, or null if there are no such declarations.
3.  Let <var>rules</var> be the result of performing <a id="ref-for-serialize-a-css-rule"></a>[serialize a CSS rule](#serialize-a-css-rule) on each rule in the rule’s <u><code>cssRules</code></u> list, or null if there are no such rules.
4.  If <var>decls</var> and <var>rules</var> are both null, append " }" to <var>s</var> (i.e. a single SPACE (U+0020) followed by RIGHT CURLY BRACKET (U+007D)) and return <var>s</var>.
5.  If <var>rules</var> is null:
    1.  Append a single SPACE (U+0020) to <var>s</var>
    2.  Append <var>decls</var> to <var>s</var>
    3.  Append " }" to <var>s</var> (i.e. a single SPACE (U+0020) followed by RIGHT CURLY BRACKET (U+007D)).
    4.  Return <var>s</var>.
6.  Otherwise:
    1.  If <var>decls</var> is not null, prepend it to <var>rules</var>.
    2.  For each <var>rule</var> in <var>rules</var>:
        - If <var>rule</var> is the empty string, do nothing.
        - Otherwise:
          1.  Append a newline followed by two spaces to <var>s</var>.
          2.  Append <var>rule</var> to <var>s</var>.
    3.  Append a newline followed by RIGHT CURLY BRACKET (U+007D) to <var>s</var>.
    4.  Return <var>s</var>.

<strong><code><a id="ref-for-cssimportrule"></a>[CSSImportRule](#cssimportrule)</code></strong>

The result of concatenating the following:

1.  The string "<code>@import</code>" followed by a single SPACE (U+0020).
2.  The result of performing <a id="ref-for-serialize-a-url"></a>[serialize a URL](#serialize-a-url) on the rule’s location.
3.  If the rule’s associated media list is not empty, a single SPACE (U+0020) followed by the result of performing <a id="ref-for-serialize-a-media-query-list②"></a>[serialize a media query list](#serialize-a-media-query-list) on the media list.
4.  The string "<code>;</code>", i.e., SEMICOLON (U+003B).

<a id="example-844003e0"></a>

<strong>Example:</strong>

[](#example-844003e0)

``` text
@import url("import.css");
```

``` text
@import url("print.css") print;
```

<strong><code><a id="ref-for-cssmediarule"></a>[CSSMediaRule](https://drafts.csswg.org/css-conditional-3/#cssmediarule)</code></strong>

The result of concatenating the following:

1.  The string "<code>@media</code>", followed by a single SPACE (U+0020).
2.  The result of performing <a id="ref-for-serialize-a-media-query-list③"></a>[serialize a media query list](#serialize-a-media-query-list) on rule’s media query list.
3.  A single SPACE (U+0020), followed by the string "{", i.e., LEFT CURLY BRACKET (U+007B), followed by a newline.
4.  The result of performing <a id="ref-for-serialize-a-css-rule①"></a>[serialize a CSS rule](#serialize-a-css-rule) on each rule in the rule’s <code><a id="ref-for-dom-cssgroupingrule-cssrules"></a>[cssRules](#dom-cssgroupingrule-cssrules)</code> list, filtering out empty strings, indenting each item with two spaces, all joined with newline.
5.  A newline, followed by the string "}", i.e., RIGHT CURLY BRACKET (U+007D)

<strong><code><a id="ref-for-cssfontfacerule"></a>[CSSFontFaceRule](https://drafts.csswg.org/css-fonts-5/#cssfontfacerule)</code></strong>

The result of concatenating the following:

1.  The string "<code>@font-face {</code>".
2.  If the <a id="ref-for-descdef-font-face-font-family"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-family) descriptor is present:
    1.  A single SPACE (U+0020), followed by the string "<code>font-family&#58;</code>", followed by a single SPACE (U+0020).
    2.  The result of performing <a id="ref-for-serialize-a-string④"></a>[serialize a string](#serialize-a-string) on the rule’s font family name.
    3.  The string "<code>;</code>", i.e., SEMICOLON (U+003B).
3.  If the rule’s associated source list is not empty, follow these substeps:
    1.  A single SPACE (U+0020), followed by the string "<code>src&#58;</code>", followed by a single SPACE (U+0020).
    2.  The result of invoking <a id="ref-for-serialize-a-comma-separated-list③"></a>[serialize a comma-separated list](#serialize-a-comma-separated-list) on performing <a id="ref-for-serialize-a-url①"></a>[serialize a URL](#serialize-a-url) or <a id="ref-for-serialize-a-local"></a>[serialize a LOCAL](#serialize-a-local) for each source on the source list.
    3.  The string "<code>;</code>", i.e., SEMICOLON (U+003B).
4.  If rule’s associated <a id="ref-for-descdef-font-face-unicode-range"></a>[unicode-range](https://drafts.csswg.org/css-fonts-4/#descdef-font-face-unicode-range) descriptor is present, a single SPACE (U+0020), followed by the string "<code>unicode-range&#58;</code>", followed by a single SPACE (U+0020), followed by the result of performing serialize a <a id="ref-for-descdef-font-face-unicode-range①"></a>\<'unicode-range'\>, followed by the string "<code>;</code>", i.e., SEMICOLON (U+003B).
5.  If rule’s associated <a id="ref-for-propdef-font-variant"></a>[font-variant](https://drafts.csswg.org/css-fonts-4/#propdef-font-variant) descriptor is present, a single SPACE (U+0020), followed by the string "<code>font-variant&#58;</code>", followed by a single SPACE (U+0020), followed by the result of performing serialize a <a id="ref-for-propdef-font-variant①"></a>\<'font-variant'\>, followed by the string "<code>;</code>", i.e., SEMICOLON (U+003B).
6.  If rule’s associated <a id="ref-for-propdef-font-feature-settings"></a>[font-feature-settings](https://drafts.csswg.org/css-fonts-4/#propdef-font-feature-settings) descriptor is present, a single SPACE (U+0020), followed by the string "<code>font-feature-settings&#58;</code>", followed by a single SPACE (U+0020), followed by the result of performing serialize a <a id="ref-for-propdef-font-feature-settings①"></a>\<'font-feature-settings'\>, followed by the string "<code>;</code>", i.e., SEMICOLON (U+003B).
7.  If rule’s associated <a id="ref-for-propdef-font-stretch"></a>[font-stretch](https://drafts.csswg.org/css-fonts-4/#propdef-font-stretch) descriptor is present, a single SPACE (U+0020), followed by the string "<code>font-stretch&#58;</code>", followed by a single SPACE (U+0020), followed by the result of performing serialize a <a id="ref-for-propdef-font-stretch①"></a>\<'font-stretch'\>, followed by the string "<code>;</code>", i.e., SEMICOLON (U+003B).
8.  If rule’s associated <a id="ref-for-propdef-font-weight"></a>[font-weight](https://drafts.csswg.org/css-fonts-4/#propdef-font-weight) descriptor is present, a single SPACE (U+0020), followed by the string "<code>font-weight&#58;</code>", followed by a single SPACE (U+0020), followed by the result of performing serialize a <a id="ref-for-propdef-font-weight①"></a>\<'font-weight'\>, followed by the string "<code>;</code>", i.e., SEMICOLON (U+003B).
9.  If rule’s associated <a id="ref-for-propdef-font-style"></a>[font-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-style) descriptor is present, a single SPACE (U+0020), followed by the string "<code>font-style&#58;</code>", followed by a single SPACE (U+0020), followed by the result of performing serialize a <a id="ref-for-propdef-font-style①"></a>\<'font-style'\>, followed by the string "<code>;</code>", i.e., SEMICOLON (U+003B).
10. A single SPACE (U+0020), followed by the string "}", i.e., RIGHT CURLY BRACKET (U+007D).

<a id="issue-f92cf3b3"></a>

<strong>Issue:</strong>

[](#issue-f92cf3b3) Need to define how the <code><a id="ref-for-cssfontfacerule①"></a>[CSSFontFaceRule](https://drafts.csswg.org/css-fonts-5/#cssfontfacerule)</code> descriptors' values are serialized.

<strong><code><a id="ref-for-csspagerule"></a>[CSSPageRule](#csspagerule)</code></strong>

<a id="issue-be6dc86c"></a>

<strong>Issue:</strong>

[](#issue-be6dc86c) Need to define how <code><a id="ref-for-csspagerule①"></a>[CSSPageRule](#csspagerule)</code> is serialized.

<strong><code><a id="ref-for-cssnamespacerule"></a>[CSSNamespaceRule](#cssnamespacerule)</code></strong>

The literal string "<code>@namespace</code>", followed by a single SPACE (U+0020), followed by the <a id="ref-for-serialize-an-identifier⑧"></a>[serialization as an identifier](#serialize-an-identifier) of the <code><a id="ref-for-dom-cssnamespacerule-prefix"></a>[prefix](#dom-cssnamespacerule-prefix)</code> attribute (if any), followed by a single SPACE (U+0020) if there is a prefix, followed by the <a id="ref-for-serialize-a-url②"></a>[serialization as URL](#serialize-a-url) of the <code><a id="ref-for-dom-cssnamespacerule-namespaceuri"></a>[namespaceURI](#dom-cssnamespacerule-namespaceuri)</code> attribute, followed the character "<code>;</code>" (U+003B).

<strong><code><a id="ref-for-csskeyframesrule"></a>[CSSKeyframesRule](https://drafts.csswg.org/css-animations-1/#csskeyframesrule)</code></strong>

The result of concatenating the following:

1.  The literal string "<code>@keyframes</code>", followed by a single SPACE (U+0020).
2.  The serialization of the <code><a id="ref-for-dom-csskeyframesrule-name"></a>[name](https://drafts.csswg.org/css-animations-1/#dom-csskeyframesrule-name)</code> attribute. If the attribute is a CSS wide keyword, or the value default, or the value none, then it is <a id="ref-for-serialize-a-string⑤"></a>[serialized as a string](#serialize-a-string). Otherwise, it is <a id="ref-for-serialize-an-identifier⑨"></a>[serialized as an identifier](#serialize-an-identifier).
3.  The string "<code> { </code>", i.e., a single SPACE (U+0020), followed by LEFT CURLY BRACKET (U+007B), followed by a single SPACE (U+0020).
4.  The result of performing <a id="ref-for-serialize-a-css-rule②"></a>[serialize a CSS rule](#serialize-a-css-rule) on each rule in the rule’s <code><a id="ref-for-dom-csskeyframesrule-cssrules"></a>[cssRules](https://drafts.csswg.org/css-animations-1/#dom-csskeyframesrule-cssrules)</code> list, separated by a newline and indented by two spaces.
5.  A newline, followed by the string "}", i.e., RIGHT CURLY BRACKET (U+007D)

<strong><code><a id="ref-for-csskeyframerule"></a>[CSSKeyframeRule](https://drafts.csswg.org/css-animations-1/#csskeyframerule)</code></strong>

The result of concatenating the following:

1.  The <code><a id="ref-for-dom-csskeyframerule-keytext"></a>[keyText](https://drafts.csswg.org/css-animations-1/#dom-csskeyframerule-keytext)</code>.
2.  The string "<code> { </code>", i.e., a single SPACE (U+0020), followed by LEFT CURLY BRACKET (U+007B), followed by a single SPACE (U+0020).
3.  The result of performing <a id="ref-for-serialize-a-css-declaration-block①"></a>[serialize a CSS declaration block](#serialize-a-css-declaration-block) on the rule’s associated declarations.
4.  If the rule is associated with one or more declarations, the string "<code> </code>", i.e., a single SPACE (U+0020).
5.  The string "<code>}</code>", RIGHT CURLY BRACKET (U+007D).

<a id="issue-ca02cb3b"></a>

<strong>Issue:</strong>

[](#issue-ca02cb3b) The "indented by two spaces" bit matches browsers, but needs work, see [\#5494](https://github.com/w3c/csswg-drafts/issues/5494)

To <a id="insert-a-css-rule"></a><strong>insert a CSS rule</strong> <var>rule</var> in a CSS rule list <var>list</var> at index <var>index</var>, with a flag <var>nested</var>, follow these steps:

1.  Set <var>length</var> to the number of items in <var>list</var>.

2.  If <var>index</var> is greater than <var>length</var>, then <a id="ref-for-dfn-throw④"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) an <code><a id="ref-for-indexsizeerror"></a>[IndexSizeError](https://webidl.spec.whatwg.org/#indexsizeerror)</code> exception.

3.  Set <var>new rule</var> to the results of performing <a id="ref-for-parse-a-css-rule"></a>[parse a CSS rule](#parse-a-css-rule) on argument <var>rule</var>.

4.  If <var>new rule</var> is a syntax error, and <var>nested</var> is set, perform the following substeps:
    - Set <var>declarations</var> to the results of performing <a id="ref-for-parse-a-css-declaration-block"></a>[parse a CSS declaration block](#parse-a-css-declaration-block), on argument <var>rule</var>.
    - If <var>declarations</var> is empty, <a id="ref-for-dfn-throw⑤"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) a <code><a id="ref-for-syntaxerror②"></a>[SyntaxError](https://webidl.spec.whatwg.org/#syntaxerror)</code> exception.
    - Otherwise, set <var>new rule</var> to a new <a id="ref-for-nested-declarations-rule"></a>[nested declarations rule](https://drafts.csswg.org/css-nesting-1/#nested-declarations-rule) with <var>declarations</var> as it contents.

5.  If <var>new rule</var> is a syntax error, <a id="ref-for-dfn-throw⑥"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) a <code><a id="ref-for-syntaxerror③"></a>[SyntaxError](https://webidl.spec.whatwg.org/#syntaxerror)</code> exception.

6.  If <var>new rule</var> cannot be inserted into <var>list</var> at the zero-indexed position <var>index</var> due to constraints specified by CSS, then <a id="ref-for-dfn-throw⑦"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) a <code><a id="ref-for-hierarchyrequesterror"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code> exception. [\[CSS21\]](#biblio-css21)

    Note: For example, a CSS style sheet cannot contain an <code>@import</code> at-rule after a style rule.

7.  If <var>new rule</var> is an <code>@namespace</code> at-rule, and <var>list</var> contains anything other than <code>@import</code> at-rules, and <code>@namespace</code> at-rules, <a id="ref-for-dfn-throw⑧"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) an <code><a id="ref-for-invalidstateerror"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code> exception.

8.  Insert <var>new rule</var> into <var>list</var> at the zero-indexed position <var>index</var>.

9.  Return <var>index</var>.

Tests

- [serialize-media-rule.html](https://wpt.fyi/results/css/cssom/serialize-media-rule.html) [(live test)](http://wpt.live/css/cssom/serialize-media-rule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/serialize-media-rule.html)

To <a id="remove-a-css-rule"></a><strong>remove a CSS rule</strong> from a CSS rule list <var>list</var> at index <var>index</var>, follow these steps:

1.  Set <var>length</var> to the number of items in <var>list</var>.
2.  If <var>index</var> is greater than or equal to <var>length</var>, then <a id="ref-for-dfn-throw⑨"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) an <code><a id="ref-for-indexsizeerror①"></a>[IndexSizeError](https://webidl.spec.whatwg.org/#indexsizeerror)</code> exception.
3.  Set <var>old rule</var> to the <var>index</var>th item in <var>list</var>.
4.  If <var>old rule</var> is an <code>@namespace</code> at-rule, and <var>list</var> contains anything other than <code>@import</code> at-rules, and <code>@namespace</code> at-rules, <a id="ref-for-dfn-throw①⓪"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) an <code><a id="ref-for-invalidstateerror①"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code> exception.
5.  Remove rule <var>old rule</var> from <var>list</var> at the zero-indexed position <var>index</var>.
6.  Set <var>old rule</var>’s <a id="ref-for-concept-css-rule-parent-css-rule"></a>[parent CSS rule](#concept-css-rule-parent-css-rule) and <a id="ref-for-concept-css-rule-parent-css-style-sheet"></a>[parent CSS style sheet](#concept-css-rule-parent-css-style-sheet) to null.

#### <a id="the-cssrulelist-interface"></a>6.4.1. The <code><a id="ref-for-cssrulelist④"></a>[CSSRuleList](#cssrulelist)</code> Interface[](#the-cssrulelist-interface)

The <code><a id="ref-for-cssrulelist⑤"></a>[CSSRuleList](#cssrulelist)</code> interface represents an ordered collection of <a id="ref-for-concept-css-style-sheet-css-rules⑦"></a>[CSS rules](#concept-css-style-sheet-css-rules).

<a id="ref-for-Exposed④"></a><a id="cssrulelist"></a><a id="ref-for-cssrule②"></a><a id="ref-for-dom-cssrulelist-item"></a><a id="ref-for-idl-unsigned-long⑨"></a><a id="dom-cssrulelist-item-index-index"></a><a id="ref-for-idl-unsigned-long①⓪"></a><a id="ref-for-dom-cssrulelist-length"></a>

``` text
[Exposed=Window]
interface CSSRuleList {
  getter CSSRule? item(unsigned long index);
  readonly attribute unsigned long length;
};
```

Tests

- [CSSRuleList.html](https://wpt.fyi/results/css/cssom/CSSRuleList.html) [(live test)](http://wpt.live/css/cssom/CSSRuleList.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSRuleList.html)

The object’s <a id="ref-for-dfn-supported-property-indices④"></a>[supported property indices](http://heycam.github.io/webidl/#dfn-supported-property-indices) are the numbers in the range zero to one less than the number of <code><a id="ref-for-cssrule③"></a>[CSSRule](#cssrule)</code> objects represented by the collection. If there are no such <code><a id="ref-for-cssrule④"></a>[CSSRule](#cssrule)</code> objects, then there are no <a id="ref-for-dfn-supported-property-indices⑤"></a>supported property indices.

The <a id="dom-cssrulelist-item"></a><strong><code>item(<var>index</var>)</code></strong> method must return the <var>index</var>th <code><a id="ref-for-cssrule⑤"></a>[CSSRule](#cssrule)</code> object in the collection. If there is no <var>index</var>th object in the collection, then the method must return null.

The <a id="dom-cssrulelist-length"></a><strong><code>length</code></strong> attribute must return the number of <code><a id="ref-for-cssrule⑥"></a>[CSSRule](#cssrule)</code> objects represented by the collection.

#### <a id="the-cssrule-interface"></a>6.4.2. The <code><a id="ref-for-cssrule⑦"></a>[CSSRule](#cssrule)</code> Interface[](#the-cssrule-interface)

The <code><a id="ref-for-cssrule⑧"></a>[CSSRule](#cssrule)</code> interface represents an abstract, base <a id="ref-for-css-rule⑥"></a>[CSS rule](#css-rule). Each distinct CSS rule type is represented by a distinct interface that inherits from this interface.

<a id="ref-for-Exposed⑤"></a><a id="cssrule"></a><a id="ref-for-cssomstring⑥"></a><a id="ref-for-dom-cssrule-csstext"></a><a id="ref-for-cssrule⑨"></a><a id="ref-for-dom-cssrule-parentrule"></a><a id="ref-for-cssstylesheet①③"></a><a id="ref-for-dom-cssrule-parentstylesheet"></a><a id="ref-for-idl-unsigned-short"></a><a id="ref-for-dom-cssrule-type"></a><a id="ref-for-idl-unsigned-short①"></a><a id="dom-cssrule-style&#95;rule"></a><a id="ref-for-idl-unsigned-short②"></a><a id="dom-cssrule-charset&#95;rule"></a><a id="ref-for-idl-unsigned-short③"></a><a id="dom-cssrule-import&#95;rule"></a><a id="ref-for-idl-unsigned-short④"></a><a id="dom-cssrule-media&#95;rule"></a><a id="ref-for-idl-unsigned-short⑤"></a><a id="dom-cssrule-font&#95;face&#95;rule"></a><a id="ref-for-idl-unsigned-short⑥"></a><a id="dom-cssrule-page&#95;rule"></a><a id="ref-for-idl-unsigned-short⑦"></a><a id="dom-cssrule-margin&#95;rule"></a><a id="ref-for-idl-unsigned-short⑧"></a><a id="dom-cssrule-namespace&#95;rule"></a>

``` text
[Exposed=Window]
interface CSSRule {
  attribute CSSOMString cssText;
  readonly attribute CSSRule? parentRule;
  readonly attribute CSSStyleSheet? parentStyleSheet;

  // the following attribute and constants are historical
  readonly attribute unsigned short type;
  const unsigned short STYLE_RULE = 1;
  const unsigned short CHARSET_RULE = 2;
  const unsigned short IMPORT_RULE = 3;
  const unsigned short MEDIA_RULE = 4;
  const unsigned short FONT_FACE_RULE = 5;
  const unsigned short PAGE_RULE = 6;
  const unsigned short MARGIN_RULE = 9;
  const unsigned short NAMESPACE_RULE = 10;
};
```

The <a id="dom-cssrule-csstext"></a><strong><code>cssText</code></strong> attribute must return a <a id="ref-for-serialize-a-css-rule③"></a>[serialization](#serialize-a-css-rule) of the <a id="ref-for-css-rule⑦"></a>[CSS rule](#css-rule). On setting the <code><a id="ref-for-dom-cssrule-csstext①"></a>[cssText](#dom-cssrule-csstext)</code> attribute must do nothing.

Tests

- [css-style-reparse.html](https://wpt.fyi/results/css/cssom/css-style-reparse.html) [(live test)](http://wpt.live/css/cssom/css-style-reparse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/css-style-reparse.html)
- [cssom-cssText-serialize.html](https://wpt.fyi/results/css/cssom/cssom-cssText-serialize.html) [(live test)](http://wpt.live/css/cssom/cssom-cssText-serialize.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssom-cssText-serialize.html)
- [cssom-ruleTypeAndOrder.html](https://wpt.fyi/results/css/cssom/cssom-ruleTypeAndOrder.html) [(live test)](http://wpt.live/css/cssom/cssom-ruleTypeAndOrder.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssom-ruleTypeAndOrder.html)
- [declaration-block-all-crash.html](https://wpt.fyi/results/css/cssom/declaration-block-all-crash.html) [(live test)](http://wpt.live/css/cssom/declaration-block-all-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/declaration-block-all-crash.html)

The <a id="dom-cssrule-parentrule"></a><strong><code>parentRule</code></strong> attribute must return the <a id="ref-for-concept-css-rule-parent-css-rule①"></a>[parent CSS rule](#concept-css-rule-parent-css-rule).

Note: For example, <code>@media</code> can enclose a rule, in which case <code><a id="ref-for-dom-cssrule-parentrule①"></a>[parentRule](#dom-cssrule-parentrule)</code> would be non-null; in cases where there is no enclosing rule, <code><a id="ref-for-dom-cssrule-parentrule②"></a>[parentRule](#dom-cssrule-parentrule)</code> will be null.

The <a id="dom-cssrule-parentstylesheet"></a><strong><code>parentStyleSheet</code></strong> attribute must return the <a id="ref-for-concept-css-rule-parent-css-style-sheet①"></a>[parent CSS style sheet](#concept-css-rule-parent-css-style-sheet).

Note: The only circumstance where null is returned when a rule has been <a id="ref-for-remove-a-css-rule③"></a>[removed](#remove-a-css-rule).

Note: Removing a <code>Node</code> that implements the <code>LinkStyle</code> interface from a <code><a id="ref-for-document③"></a>[Document](https://dom.spec.whatwg.org/#document)</code> instance does not (by itself) cause the <code>CSSStyleSheet</code> referenced by a <code>CSSRule</code> to be unreachable.

The <a id="dom-cssrule-type"></a><strong><code>type</code></strong> attribute is deprecated. It must return an integer, as follows:

<strong>If the object is a <code><a id="ref-for-cssstylerule①"></a>[CSSStyleRule](#cssstylerule)</code></strong>

Return 1.

<strong>If the object is a <code><a id="ref-for-cssimportrule①"></a>[CSSImportRule](#cssimportrule)</code></strong>

Return 3.

<strong>If the object is a <code><a id="ref-for-cssmediarule①"></a>[CSSMediaRule](https://drafts.csswg.org/css-conditional-3/#cssmediarule)</code></strong>

Return 4.

<strong>If the object is a <code><a id="ref-for-cssfontfacerule②"></a>[CSSFontFaceRule](https://drafts.csswg.org/css-fonts-5/#cssfontfacerule)</code></strong>

Return 5.

<strong>If the object is a <code><a id="ref-for-csspagerule②"></a>[CSSPageRule](#csspagerule)</code></strong>

Return 6.

<strong>If the object is a <code><a id="ref-for-csskeyframesrule①"></a>[CSSKeyframesRule](https://drafts.csswg.org/css-animations-1/#csskeyframesrule)</code></strong>

Return 7.

<strong>If the object is a <code><a id="ref-for-csskeyframerule①"></a>[CSSKeyframeRule](https://drafts.csswg.org/css-animations-1/#csskeyframerule)</code></strong>

Return 8.

<strong>If the object is a <code><a id="ref-for-cssmarginrule"></a>[CSSMarginRule](#cssmarginrule)</code></strong>

Return 9.

<strong>If the object is a <code><a id="ref-for-cssnamespacerule①"></a>[CSSNamespaceRule](#cssnamespacerule)</code></strong>

Return 10.

<strong>If the object is a <code><a id="ref-for-csscounterstylerule"></a>[CSSCounterStyleRule](https://drafts.csswg.org/css-counter-styles-3/#csscounterstylerule)</code></strong>

Return 11.

<strong>If the object is a <code><a id="ref-for-csssupportsrule"></a>[CSSSupportsRule](https://drafts.csswg.org/css-conditional-3/#csssupportsrule)</code></strong>

Return 12.

<strong>If the object is a <code><a id="ref-for-om-fontfeaturevalues"></a>[CSSFontFeatureValuesRule](https://drafts.csswg.org/css-fonts-4/#om-fontfeaturevalues)</code></strong>

Return 14.

<strong>Otherwise</strong>

Return 0.

Note: The practice of using an integer enumeration and several constants to <em>identify</em> the integers is a legacy design practice that is no longer used in Web APIs. Instead, to tell what type of rule a given object is, it is recommended to check <code>rule.constructor.name</code>, which will return a string like <code>&#34;CSSStyleRule&#34;</code>. This enumeration is thus frozen in its current state, and no new new values will be added to reflect additional at-rules; all at-rules beyond the ones listed above will return 0.

#### <a id="the-cssstylerule-interface"></a>6.4.3. The <code><a id="ref-for-cssstylerule②"></a>[CSSStyleRule](#cssstylerule)</code> Interface[](#the-cssstylerule-interface)

The <code>CSSStyleRule</code> interface represents a style rule.

<a id="ref-for-Exposed⑥"></a><a id="cssstylerule"></a><a id="ref-for-cssgroupingrule①"></a><a id="ref-for-cssomstring⑦"></a><a id="ref-for-dom-cssstylerule-selectortext"></a><a id="ref-for-SameObject④"></a><a id="ref-for-PutForwards①"></a><a id="ref-for-cssstyleproperties"></a><a id="ref-for-dom-cssstylerule-style"></a>

``` text
[Exposed=Window]
interface CSSStyleRule : CSSGroupingRule {
  attribute CSSOMString selectorText;
  [SameObject, PutForwards=cssText] readonly attribute CSSStyleProperties style;
};
```

The <a id="dom-cssstylerule-selectortext"></a><strong><code>selectorText</code></strong> attribute, on getting, must return the result of <a id="ref-for-serialize-a-group-of-selectors②"></a>[serializing](#serialize-a-group-of-selectors) the rule’s associated <a id="ref-for-selector-list"></a>[selector list](https://drafts.csswg.org/selectors-4/#selector-list). On setting the <code><a id="ref-for-dom-cssstylerule-selectortext①"></a>[selectorText](#dom-cssstylerule-selectortext)</code> attribute these steps must be run:

1.  Run the <a id="ref-for-parse-a-group-of-selectors"></a>[parse a group of selectors](#parse-a-group-of-selectors) algorithm on the given value.
2.  If the algorithm returns a non-null value replace the associated <a id="ref-for-selector-list①"></a>[selector list](https://drafts.csswg.org/selectors-4/#selector-list) with the returned value.
3.  Otherwise, if the algorithm returns a null value, do nothing.

Tests

- [CSSStyleRule.html](https://wpt.fyi/results/css/cssom/CSSStyleRule.html) [(live test)](http://wpt.live/css/cssom/CSSStyleRule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleRule.html)
- [CSSStyleRule-set-selectorText.html](https://wpt.fyi/results/css/cssom/CSSStyleRule-set-selectorText.html) [(live test)](http://wpt.live/css/cssom/CSSStyleRule-set-selectorText.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleRule-set-selectorText.html)
- [CSSStyleRule-set-selectorText-namespace.html](https://wpt.fyi/results/css/cssom/CSSStyleRule-set-selectorText-namespace.html) [(live test)](http://wpt.live/css/cssom/CSSStyleRule-set-selectorText-namespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSStyleRule-set-selectorText-namespace.html)
- [selectorText-modification-restyle-001.html](https://wpt.fyi/results/css/cssom/selectorText-modification-restyle-001.html) [(live test)](http://wpt.live/css/cssom/selectorText-modification-restyle-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/selectorText-modification-restyle-001.html)
- [selectorText-modification-restyle-002.html](https://wpt.fyi/results/css/cssom/selectorText-modification-restyle-002.html) [(live test)](http://wpt.live/css/cssom/selectorText-modification-restyle-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/selectorText-modification-restyle-002.html)
- [set-selector-text-attachment.html](https://wpt.fyi/results/css/cssom/set-selector-text-attachment.html) [(live test)](http://wpt.live/css/cssom/set-selector-text-attachment.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/set-selector-text-attachment.html)

The <a id="dom-cssstylerule-style"></a><strong><code>style</code></strong> attribute must return a <code><a id="ref-for-cssstyleproperties①"></a>[CSSStyleProperties](#cssstyleproperties)</code> object for the style rule, with the following properties:

<strong><a id="ref-for-cssstyledeclaration-computed-flag"></a>[computed flag](#cssstyledeclaration-computed-flag)</strong>

Unset.

<strong><a id="ref-for-cssstyledeclaration-readonly-flag"></a>[readonly flag](#cssstyledeclaration-readonly-flag)</strong>

Unset.

<strong><a id="ref-for-cssstyledeclaration-declarations"></a>[declarations](#cssstyledeclaration-declarations)</strong>

The declared declarations in the rule, in <a id="ref-for-concept-declarations-specified-order"></a>[specified order](#concept-declarations-specified-order).

<strong><a id="ref-for-cssstyledeclaration-parent-css-rule"></a>[parent CSS rule](#cssstyledeclaration-parent-css-rule)</strong>

<a id="ref-for-this"></a>[this](https://webidl.spec.whatwg.org/#this).

<strong><a id="ref-for-cssstyledeclaration-owner-node"></a>[owner node](#cssstyledeclaration-owner-node)</strong>

Null.

The <a id="concept-declarations-specified-order"></a><strong>specified order</strong> for declarations is the same as specified, but with shorthand properties expanded into their longhand properties, in canonical order. If a property is specified more than once (after shorthand expansion), only the one with greatest cascading order must be represented, at the same relative position as it was specified. [\[CSS3CASCADE\]](#biblio-css3cascade)

#### <a id="the-cssimportrule-interface"></a>6.4.4. The <code><a id="ref-for-cssimportrule②"></a>[CSSImportRule](#cssimportrule)</code> Interface[](#the-cssimportrule-interface)

The <code>CSSImportRule</code> interface represents an <code>@import</code> at-rule.

<a id="ref-for-Exposed⑦"></a><a id="cssimportrule"></a><a id="ref-for-cssrule①⓪"></a><a id="ref-for-idl-USVString⑥"></a><a id="ref-for-dom-cssimportrule-href"></a><a id="ref-for-SameObject⑤"></a><a id="ref-for-PutForwards②"></a><a id="ref-for-dom-medialist-mediatext⑥"></a><a id="ref-for-medialist④"></a><a id="ref-for-dom-cssimportrule-media"></a><a id="ref-for-SameObject⑥"></a><a id="ref-for-cssstylesheet①④"></a><a id="ref-for-dom-cssimportrule-stylesheet"></a><a id="ref-for-cssomstring⑧"></a><a id="ref-for-dom-cssimportrule-layername"></a><a id="ref-for-cssomstring⑨"></a><a id="ref-for-dom-cssimportrule-supportstext"></a>

``` text
[Exposed=Window]
interface CSSImportRule : CSSRule {
  readonly attribute USVString href;
  [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
  [SameObject] readonly attribute CSSStyleSheet? styleSheet;
  readonly attribute CSSOMString? layerName;
  readonly attribute CSSOMString? supportsText;
};
```

The <a id="dom-cssimportrule-href"></a><strong><code>href</code></strong> attribute must return the <a id="ref-for-concept-url"></a>[URL](https://url.spec.whatwg.org/#concept-url) specified by the <code>@import</code> at-rule.

Note: To get the resolved <a id="ref-for-concept-url①"></a>[URL](https://url.spec.whatwg.org/#concept-url) use the <code><a id="ref-for-dom-stylesheet-href①"></a>[href](#dom-stylesheet-href)</code> attribute of the associated <a id="ref-for-css-style-sheet④④"></a>[CSS style sheet](#css-style-sheet).

The <a id="dom-cssimportrule-media"></a><strong><code>media</code></strong> attribute must return the value of the <code><a id="ref-for-dom-stylesheet-media①"></a>[media](#dom-stylesheet-media)</code> attribute of the associated <a id="ref-for-css-style-sheet④⑤"></a>[CSS style sheet](#css-style-sheet).

The <a id="dom-cssimportrule-stylesheet"></a><strong><code>styleSheet</code></strong> attribute must return the associated <a id="ref-for-css-style-sheet④⑥"></a>[CSS style sheet](#css-style-sheet), if any, or null otherwise.

The <a id="dom-cssimportrule-layername"></a><strong><code>layerName</code></strong> attribute must return the <a id="ref-for-layer-name"></a>[layer name](https://drafts.csswg.org/css-cascade-5/#layer-name) declared in the at-rule itself, or an empty string if the layer is anonymous, or null if the at-rule does not declare a layer.

The <a id="dom-cssimportrule-supportstext"></a><strong><code>supportsText</code></strong> attribute must return the <a id="ref-for-typedef-supports-condition"></a>[\<supports-condition\>](https://drafts.csswg.org/css-conditional-3/#typedef-supports-condition) declared in the at-rule itself, or null if the at-rule does not declare a supports condition.

Note: An <code>@import</code> at-rule might not have an associated <a id="ref-for-css-style-sheet④⑦"></a>[CSS style sheet](#css-style-sheet) (e.g., if it has a non-matching <code>supports()</code> condition).

Tests

- [cssimportrule.html](https://wpt.fyi/results/css/cssom/cssimportrule.html) [(live test)](http://wpt.live/css/cssom/cssimportrule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssimportrule.html)
- [cssimportrule-parent.html](https://wpt.fyi/results/css/cssom/cssimportrule-parent.html) [(live test)](http://wpt.live/css/cssom/cssimportrule-parent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssimportrule-parent.html)
- [cssimportrule-sheet-identity.html](https://wpt.fyi/results/css/cssom/cssimportrule-sheet-identity.html) [(live test)](http://wpt.live/css/cssom/cssimportrule-sheet-identity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssimportrule-sheet-identity.html)

#### <a id="the-cssgroupingrule-interface"></a>6.4.5. The <code><a id="ref-for-cssgroupingrule②"></a>[CSSGroupingRule](#cssgroupingrule)</code> Interface[](#the-cssgroupingrule-interface)

The <code>CSSGroupingRule</code> interface represents an at-rule that contains other rules nested inside itself.

<a id="ref-for-Exposed⑧"></a><a id="cssgroupingrule"></a><a id="ref-for-cssrule①①"></a><a id="ref-for-SameObject⑦"></a><a id="ref-for-cssrulelist⑥"></a><a id="ref-for-dom-cssgroupingrule-cssrules①"></a><a id="ref-for-idl-unsigned-long①①"></a><a id="ref-for-dom-cssgroupingrule-insertrule"></a><a id="ref-for-cssomstring①⓪"></a><a id="dom-cssgroupingrule-insertrule-rule-index-rule"></a><a id="ref-for-idl-unsigned-long①②"></a><a id="dom-cssgroupingrule-insertrule-rule-index-index"></a><a id="ref-for-idl-undefined⑤"></a><a id="ref-for-dom-cssgroupingrule-deleterule"></a><a id="ref-for-idl-unsigned-long①③"></a><a id="dom-cssgroupingrule-deleterule-index-index"></a>

``` text
[Exposed=Window]
interface CSSGroupingRule : CSSRule {
  [SameObject] readonly attribute CSSRuleList cssRules;
  unsigned long insertRule(CSSOMString rule, optional unsigned long index = 0);
  undefined deleteRule(unsigned long index);
};
```

The <a id="dom-cssgroupingrule-cssrules"></a><strong><code>cssRules</code></strong> attribute must return a <code>CSSRuleList</code> object for the <a id="ref-for-concept-css-rule-child-css-rules"></a>[child CSS rules](#concept-css-rule-child-css-rules).

The <a id="dom-cssgroupingrule-insertrule"></a><strong><code>insertRule(<var>rule</var>, <var>index</var>)</code></strong> method must return the result of invoking <a id="ref-for-insert-a-css-rule①"></a>[insert a CSS rule](#insert-a-css-rule) <var>rule</var> into the <a id="ref-for-concept-css-rule-child-css-rules①"></a>[child CSS rules](#concept-css-rule-child-css-rules) at <var>index</var>, with the <var>nested</var> flag set.

The <a id="dom-cssgroupingrule-deleterule"></a><strong><code>deleteRule(<var>index</var>)</code></strong> method must <a id="ref-for-remove-a-css-rule④"></a>[remove a CSS rule](#remove-a-css-rule) from the <a id="ref-for-concept-css-rule-child-css-rules②"></a>[child CSS rules](#concept-css-rule-child-css-rules) at <var>index</var>.

Tests

- [CSSGroupingRule-cssRules.html](https://wpt.fyi/results/css/cssom/CSSGroupingRule-cssRules.html) [(live test)](http://wpt.live/css/cssom/CSSGroupingRule-cssRules.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSGroupingRule-cssRules.html)
- [CSSGroupingRule-insertRule.html](https://wpt.fyi/results/css/cssom/CSSGroupingRule-insertRule.html) [(live test)](http://wpt.live/css/cssom/CSSGroupingRule-insertRule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSGroupingRule-insertRule.html)

#### <a id="the-cssmediarule-interface"></a>6.4.6. The <code><a id="ref-for-cssmediarule②"></a>[CSSMediaRule](https://drafts.csswg.org/css-conditional-3/#cssmediarule)</code> Interface[](#the-cssmediarule-interface)

The <code><a id="ref-for-cssmediarule③"></a>[CSSMediaRule](https://drafts.csswg.org/css-conditional-3/#cssmediarule)</code> interface is defined in CSS Conditional Rules. [\[CSS3-CONDITIONAL\]](#biblio-css3-conditional)

#### <a id="the-csspagerule-interface"></a>6.4.7. The <code><a id="ref-for-csspagerule③"></a>[CSSPageRule](#csspagerule)</code> Interface[](#the-csspagerule-interface)

The <code>CSSPageRule</code> interface represents an <code>@page</code> at-rule.

<a id="issue-ba9fab84"></a>

<strong>Issue:</strong>

[](#issue-ba9fab84) Need to define the rules for <a id="parse-a-list-of-css-page-selectors"></a><strong>parse a list of CSS page selectors</strong> and <a id="serialize-a-list-of-css-page-selectors"></a><strong>serialize a list of CSS page selectors</strong>.

<a id="ref-for-Exposed⑨"></a><a id="csspagedescriptors"></a><a id="ref-for-cssstyledeclaration"></a><a id="ref-for-LegacyNullToEmptyString①"></a><a id="ref-for-cssomstring①①"></a><a id="dom-csspagedescriptors-margin"></a><a id="ref-for-LegacyNullToEmptyString②"></a><a id="ref-for-cssomstring①②"></a><a id="dom-csspagedescriptors-margintop"></a><a id="ref-for-LegacyNullToEmptyString③"></a><a id="ref-for-cssomstring①③"></a><a id="dom-csspagedescriptors-marginright"></a><a id="ref-for-LegacyNullToEmptyString④"></a><a id="ref-for-cssomstring①④"></a><a id="dom-csspagedescriptors-marginbottom"></a><a id="ref-for-LegacyNullToEmptyString⑤"></a><a id="ref-for-cssomstring①⑤"></a><a id="dom-csspagedescriptors-marginleft"></a><a id="ref-for-LegacyNullToEmptyString⑥"></a><a id="ref-for-cssomstring①⑥"></a><a id="dom-csspagedescriptors-margin-top"></a><a id="ref-for-LegacyNullToEmptyString⑦"></a><a id="ref-for-cssomstring①⑦"></a><a id="dom-csspagedescriptors-margin-right"></a><a id="ref-for-LegacyNullToEmptyString⑧"></a><a id="ref-for-cssomstring①⑧"></a><a id="dom-csspagedescriptors-margin-bottom"></a><a id="ref-for-LegacyNullToEmptyString⑨"></a><a id="ref-for-cssomstring①⑨"></a><a id="dom-csspagedescriptors-margin-left"></a><a id="ref-for-LegacyNullToEmptyString①⓪"></a><a id="ref-for-cssomstring②⓪"></a><a id="dom-csspagedescriptors-size"></a><a id="ref-for-LegacyNullToEmptyString①①"></a><a id="ref-for-cssomstring②①"></a><a id="dom-csspagedescriptors-pageorientation"></a><a id="ref-for-LegacyNullToEmptyString①②"></a><a id="ref-for-cssomstring②②"></a><a id="dom-csspagedescriptors-page-orientation"></a><a id="ref-for-LegacyNullToEmptyString①③"></a><a id="ref-for-cssomstring②③"></a><a id="dom-csspagedescriptors-marks"></a><a id="ref-for-LegacyNullToEmptyString①④"></a><a id="ref-for-cssomstring②④"></a><a id="dom-csspagedescriptors-bleed"></a><a id="ref-for-Exposed①⓪"></a><a id="csspagerule"></a><a id="ref-for-cssgroupingrule③"></a><a id="ref-for-cssomstring②⑤"></a><a id="ref-for-dom-csspagerule-selectortext"></a><a id="ref-for-SameObject⑧"></a><a id="ref-for-PutForwards③"></a><a id="ref-for-csspagedescriptors"></a><a id="ref-for-dom-csspagerule-style"></a>

``` text
[Exposed=Window]
interface CSSPageDescriptors : CSSStyleDeclaration {
  attribute [LegacyNullToEmptyString] CSSOMString margin;
  attribute [LegacyNullToEmptyString] CSSOMString marginTop;
  attribute [LegacyNullToEmptyString] CSSOMString marginRight;
  attribute [LegacyNullToEmptyString] CSSOMString marginBottom;
  attribute [LegacyNullToEmptyString] CSSOMString marginLeft;
  attribute [LegacyNullToEmptyString] CSSOMString margin-top;
  attribute [LegacyNullToEmptyString] CSSOMString margin-right;
  attribute [LegacyNullToEmptyString] CSSOMString margin-bottom;
  attribute [LegacyNullToEmptyString] CSSOMString margin-left;
  attribute [LegacyNullToEmptyString] CSSOMString size;
  attribute [LegacyNullToEmptyString] CSSOMString pageOrientation;
  attribute [LegacyNullToEmptyString] CSSOMString page-orientation;
  attribute [LegacyNullToEmptyString] CSSOMString marks;
  attribute [LegacyNullToEmptyString] CSSOMString bleed;
};

[Exposed=Window]
interface CSSPageRule : CSSGroupingRule {
           attribute CSSOMString selectorText;
  [SameObject, PutForwards=cssText] readonly attribute CSSPageDescriptors style;
};
```

The <a id="dom-csspagerule-selectortext"></a><strong><code>selectorText</code></strong> attribute, on getting, must return the result of <a id="ref-for-serialize-a-list-of-css-page-selectors"></a>[serializing](#serialize-a-list-of-css-page-selectors) the associated <a id="ref-for-selector-list②"></a>[selector list](https://drafts.csswg.org/selectors-4/#selector-list). On setting the <code><a id="ref-for-dom-csspagerule-selectortext①"></a>[selectorText](#dom-csspagerule-selectortext)</code> attribute these steps must be run:

1.  Run the <a id="ref-for-parse-a-list-of-css-page-selectors"></a>[parse a list of CSS page selectors](#parse-a-list-of-css-page-selectors) algorithm on the given value.
2.  If the algorithm returns a non-null value replace the associated <a id="ref-for-selector-list③"></a>[selector list](https://drafts.csswg.org/selectors-4/#selector-list) with the returned value.
3.  Otherwise, if the algorithm returns a null value, do nothing.

The <a id="dom-csspagerule-style"></a><strong><code>style</code></strong> attribute must return a <code>CSSPageDescriptors</code> object for the <code>@page</code> at-rule, with the following properties:

<strong><a id="ref-for-cssstyledeclaration-computed-flag①"></a>[computed flag](#cssstyledeclaration-computed-flag)</strong>

Unset.

<strong><a id="ref-for-cssstyledeclaration-readonly-flag①"></a>[readonly flag](#cssstyledeclaration-readonly-flag)</strong>

Unset.

<strong><a id="ref-for-cssstyledeclaration-declarations①"></a>[declarations](#cssstyledeclaration-declarations)</strong>

The declared descriptors in the rule, in <a id="ref-for-concept-declarations-specified-order①"></a>[specified order](#concept-declarations-specified-order).

<strong><a id="ref-for-cssstyledeclaration-parent-css-rule①"></a>[parent CSS rule](#cssstyledeclaration-parent-css-rule)</strong>

<a id="ref-for-this①"></a>[this](https://webidl.spec.whatwg.org/#this).

<strong><a id="ref-for-cssstyledeclaration-owner-node①"></a>[owner node](#cssstyledeclaration-owner-node)</strong>

Null.

Tests

- [cssom-pagerule.html](https://wpt.fyi/results/css/cssom/cssom-pagerule.html) [(live test)](http://wpt.live/css/cssom/cssom-pagerule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssom-pagerule.html)

#### <a id="the-cssmarginrule-interface"></a>6.4.8. The <code><a id="ref-for-cssmarginrule①"></a>[CSSMarginRule](#cssmarginrule)</code> Interface[](#the-cssmarginrule-interface)

The <code>CSSMarginRule</code> interface represents a margin at-rule (e.g. <code>@top-left</code>) in an <code>@page</code> at-rule. [\[CSS3PAGE\]](#biblio-css3page)

<a id="ref-for-Exposed①①"></a><a id="cssmarginrule"></a><a id="ref-for-cssrule①②"></a><a id="ref-for-cssomstring②⑥"></a><a id="ref-for-dom-cssmarginrule-name"></a><a id="ref-for-SameObject⑨"></a><a id="ref-for-PutForwards④"></a><a id="ref-for-dom-cssmarginrule-style"></a>

``` text
[Exposed=Window]
interface CSSMarginRule : CSSRule {
  readonly attribute CSSOMString name;
  [SameObject, PutForwards=cssText] readonly attribute CSSMarginDescriptors style;
};
```

The <a id="dom-cssmarginrule-name"></a><strong><code>name</code></strong> attribute must return the name of the margin at-rule. The <code>@</code> character is not included in the name. [\[CSS3SYN\]](#biblio-css3syn)

The <a id="dom-cssmarginrule-style"></a><strong><code>style</code></strong> attribute must return a <code>CSSMarginDescriptors</code> object for the margin at-rule, with the following properties:

<strong><a id="ref-for-cssstyledeclaration-computed-flag②"></a>[computed flag](#cssstyledeclaration-computed-flag)</strong>

Unset.

<strong><a id="ref-for-cssstyledeclaration-readonly-flag②"></a>[readonly flag](#cssstyledeclaration-readonly-flag)</strong>

Unset.

<strong><a id="ref-for-cssstyledeclaration-declarations②"></a>[declarations](#cssstyledeclaration-declarations)</strong>

The declared declarations in the rule, in <a id="ref-for-concept-declarations-specified-order②"></a>[specified order](#concept-declarations-specified-order).

<strong><a id="ref-for-cssstyledeclaration-parent-css-rule②"></a>[parent CSS rule](#cssstyledeclaration-parent-css-rule)</strong>

<a id="ref-for-this②"></a>[this](https://webidl.spec.whatwg.org/#this).

<strong><a id="ref-for-cssstyledeclaration-owner-node②"></a>[owner node](#cssstyledeclaration-owner-node)</strong>

Null.

#### <a id="the-cssnamespacerule-interface"></a>6.4.9. The <code><a id="ref-for-cssnamespacerule②"></a>[CSSNamespaceRule](#cssnamespacerule)</code> Interface[](#the-cssnamespacerule-interface)

The <code>CSSNamespaceRule</code> interface represents an <code>@namespace</code> at-rule.

<a id="ref-for-Exposed①②"></a><a id="cssnamespacerule"></a><a id="ref-for-cssrule①③"></a><a id="ref-for-cssomstring②⑦"></a><a id="ref-for-dom-cssnamespacerule-namespaceuri①"></a><a id="ref-for-cssomstring②⑧"></a><a id="ref-for-dom-cssnamespacerule-prefix①"></a>

``` text
[Exposed=Window]
interface CSSNamespaceRule : CSSRule {
  readonly attribute CSSOMString namespaceURI;
  readonly attribute CSSOMString prefix;
};
```

The <a id="dom-cssnamespacerule-namespaceuri"></a><strong><code>namespaceURI</code></strong> attribute must return the namespace of the <code>@namespace</code> at-rule.

The <a id="dom-cssnamespacerule-prefix"></a><strong><code>prefix</code></strong> attribute must return the prefix of the <code>@namespace</code> at-rule or the empty string if there is no prefix.

Tests

- [at-namespace.html](https://wpt.fyi/results/css/cssom/at-namespace.html) [(live test)](http://wpt.live/css/cssom/at-namespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/at-namespace.html)
- [CSSNamespaceRule.html](https://wpt.fyi/results/css/cssom/CSSNamespaceRule.html) [(live test)](http://wpt.live/css/cssom/CSSNamespaceRule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSSNamespaceRule.html)

### <a id="css-declarations"></a>6.5. CSS Declarations[](#css-declarations)

A <a id="css-declaration"></a><strong>CSS declaration</strong> is an abstract concept that is not exposed as an object in the DOM. A <a id="ref-for-css-declaration"></a>[CSS declaration](#css-declaration) has the following associated properties:

<strong><a id="css-declaration-property-name"></a><strong>property name</strong></strong>

The property name of the declaration.

<strong><a id="css-declaration-value"></a><strong>value</strong></strong>

The value of the declaration represented as a list of component values.

<strong><a id="css-declaration-important-flag"></a><strong>important flag</strong></strong>

Either set or unset. Can be changed.

<strong><a id="css-declaration-case-sensitive-flag"></a><strong>case-sensitive flag</strong></strong>

Set if the <a id="ref-for-css-declaration-property-name"></a>[property name](#css-declaration-property-name) is defined to be case-sensitive according to its specification, otherwise unset.

### <a id="css-declaration-blocks"></a>6.6. CSS Declaration Blocks[](#css-declaration-blocks)

A <a id="css-declaration-block"></a><strong>CSS declaration block</strong> is an ordered collection of CSS properties with their associated values, also named <a id="ref-for-css-declaration①"></a>[CSS declarations](#css-declaration). In the DOM a <a id="ref-for-css-declaration-block"></a>[CSS declaration block](#css-declaration-block) is a <code>CSSStyleDeclaration</code> object. A <a id="ref-for-css-declaration-block①"></a>CSS declaration block has the following associated properties:

<strong><a id="cssstyledeclaration-computed-flag"></a><strong>computed flag</strong></strong>

Set if the object is a computed style declaration, rather than a specified style. Unless otherwise stated it is unset.

<strong><a id="cssstyledeclaration-readonly-flag"></a><strong>readonly flag</strong></strong>

Set if the object is not modifiable.

<strong><a id="cssstyledeclaration-declarations"></a><strong>declarations</strong></strong>

The <a id="ref-for-css-declaration②"></a>[CSS declarations](#css-declaration) associated with the object.

<strong><a id="cssstyledeclaration-parent-css-rule"></a><strong>parent CSS rule</strong></strong>

The <a id="ref-for-css-rule⑧"></a>[CSS rule](#css-rule) that the <a id="ref-for-css-declaration-block②"></a>[CSS declaration block](#css-declaration-block) is associated with, if any, or null otherwise.

<strong><a id="cssstyledeclaration-owner-node"></a><strong>owner node</strong></strong>

The <code><a id="ref-for-element②"></a>[Element](https://dom.spec.whatwg.org/#element)</code> that the <a id="ref-for-css-declaration-block③"></a>[CSS declaration block](#css-declaration-block) is associated with, if any, or null otherwise.

<strong><a id="cssstyledeclaration-updating-flag"></a><strong>updating flag</strong></strong>

Unset by default. Set when the <a id="ref-for-css-declaration-block④"></a>[CSS declaration block](#css-declaration-block) is updating the <a id="ref-for-cssstyledeclaration-owner-node③"></a>[owner node](#cssstyledeclaration-owner-node)’s <code>style</code> attribute.

To <a id="parse-a-css-declaration-block"></a><strong>parse a CSS declaration block</strong> from a string <var>string</var>, follow these steps:

1.  Let <var>declarations</var> be the returned declarations from invoking <a id="ref-for-parse-a-blocks-contents"></a>[parse a block’s contents](https://drafts.csswg.org/css-syntax-3/#parse-a-blocks-contents) with <var>string</var>.
2.  Let <var>parsed declarations</var> be a new empty list.
3.  For each item <var>declaration</var> in <var>declarations</var>, follow these substeps:
    1.  Let <var>parsed declaration</var> be the result of parsing <var>declaration</var> according to the appropriate CSS specifications, dropping parts that are said to be ignored. If the whole declaration is dropped, let <var>parsed declaration</var> be null.
    2.  If <var>parsed declaration</var> is not null, append it to <var>parsed declarations</var>.
4.  Return <var>parsed declarations</var>.

To <a id="serialize-a-css-declaration"></a><strong>serialize a CSS declaration</strong> with property name <var>property</var>, value <var>value</var> and optionally an <em>important</em> flag set, follow these steps:

1.  Let <var>s</var> be the empty string.
2.  Append <var>property</var> to <var>s</var>.
3.  Append "<code>&#58; </code>" (U+003A U+0020) to <var>s</var>.
4.  If <var>value</var> contains any non-whitespace characters, append <var>value</var> to <var>s</var>.
5.  If the <em>important</em> flag is set, append "<code> !important</code>" (U+0020 U+0021 U+0069 U+006D U+0070 U+006F U+0072 U+0074 U+0061 U+006E U+0074) to <var>s</var>.
6.  Append "<code>;</code>" (U+003B) to <var>s</var>.
7.  Return <var>s</var>.

Tests

- [serialization-CSSDeclaration-with-important.html](https://wpt.fyi/results/css/cssom/serialization-CSSDeclaration-with-important.html) [(live test)](http://wpt.live/css/cssom/serialization-CSSDeclaration-with-important.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/serialization-CSSDeclaration-with-important.html)

To <a id="serialize-a-css-declaration-block"></a><strong>serialize a CSS declaration block</strong> <var>declaration block</var>, run the following steps. It will return a <a id="ref-for-string②"></a>[string](https://infra.spec.whatwg.org/#string) representing the serialization of all the declarations in <var>declaration block</var>.

1.  Let <var>serialized props</var> initially be the empty <a id="ref-for-list"></a>[list](https://infra.spec.whatwg.org/#list).

2.  Let <var>already serialized</var> initially be the empty <a id="ref-for-set"></a>[set](#set).

3.  Let <var>decls</var> be <var>declaration block</var>’s <a id="ref-for-cssstyledeclaration-declarations③"></a>[declarations](#cssstyledeclaration-declarations).

4.  For each <a id="ref-for-css-declaration③"></a>[CSS declaration](#css-declaration) <var>decl</var> in <var>decls</var>:

    1.  Let <var>name</var> be <var>decl</var>’s <a id="ref-for-css-declaration-property-name①"></a>[property name](#css-declaration-property-name).

    2.  If <var>name</var> is in <var>already serialized</var>, <a id="ref-for-iteration-continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    3.  Attempt to <a id="ref-for-serialize-into-a-shorthand-form"></a>[serialize into a shorthand form](#serialize-into-a-shorthand-form) <var>name</var>, given <var>decls</var> and <var>already serialized</var>. If this returns a string, <a id="ref-for-list-append①"></a>[append](https://infra.spec.whatwg.org/#list-append) it to <var>serialized props</var> and <a id="ref-for-iteration-continue①"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    4.  Otherwise, <a id="ref-for-serialize-a-css-declaration"></a>[serialize a CSS declaration](#serialize-a-css-declaration) with property name <var>name</var>, value the result of <a id="ref-for-serialize-a-css-value①"></a>[serializing a CSS value](#serialize-a-css-value) from <var>decl</var>, and the <a id="ref-for-css-declaration-important-flag"></a>[important flag](#css-declaration-important-flag) set if <var>decl</var> has it set. <a id="ref-for-list-append②"></a>[Append](https://infra.spec.whatwg.org/#list-append) the result to <var>serialized props</var>.

    5.  <a id="ref-for-set-append"></a>[Append](https://infra.spec.whatwg.org/#set-append) <var>name</var> to <var>already serialized</var>.

5.  Return the result of <a id="ref-for-string-concatenate"></a>[concatenating](https://infra.spec.whatwg.org/#string-concatenate) <var>serialized props</var>, with separator "<code> </code>" (U+0020 SPACE).

To <a id="serialize-into-a-shorthand-form"></a><strong>serialize into a shorthand form</strong> a property <var>property</var>, given a <a id="ref-for-list①"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-css-declaration④"></a>[CSS declarations](#css-declaration) <var>decls</var> and a <a id="ref-for-set①"></a>[set](#set) of already-serialized declarations <var>already serialized</var>, run the following steps. It will return either a <a id="ref-for-string③"></a>[string](https://infra.spec.whatwg.org/#string) representing the serialization of a shorthand that <var>property</var> is part of, or <a id="ref-for-failure"></a>[failure](https://infra.spec.whatwg.org/#failure), and will mutate <var>already serialized</var>.

1.  Let <var>possible shorthands</var> be a list of shorthand declaration names that <var>property</var> is a longhand for, in <a id="ref-for-concept-shorthands-preferred-order"></a>[preferred shorthand order](#concept-shorthands-preferred-order).

2.  <u>For each</u> <var>shorthand</var> in <var>possible shorthands</var>:

    1.  Let <var>needed longhand names</var> be the names of the longhands that <var>shorthand</var> maps to.

    2.  If any of <var>needed longhand names</var> is not in <var>decls</var>, <a id="ref-for-iteration-continue②"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    3.  If any of <var>needed longhand names</var> is in <var>already serialized</var>, <a id="ref-for-iteration-continue③"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    4.  Let <var>used longhands</var> be the declarations from <var>decls</var> corresponding to the property names in <var>needed longhand names</var>.

    5.  If there is at least one declaration in <var>used longhands</var> with its <a id="ref-for-css-declaration-important-flag①"></a>[important flag](#css-declaration-important-flag) set, and at least one without it set, <a id="ref-for-iteration-continue④"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    6.  If any of the <a id="ref-for-intermixed-properties"></a>[intermixed properties](#intermixed-properties) in <var>decls</var> between <var>used longhands</var> belong to the same <a id="ref-for-logical-property-group"></a>[logical property group](https://drafts.csswg.org/css-logical-1/#logical-property-group) as a longhand in <var>used longhands</var> but have a different <a id="ref-for-mapping-logic"></a>[mapping logic](https://drafts.csswg.org/css-logical-1/#mapping-logic), and are not in <var>used longhands</var>, <a id="ref-for-iteration-continue⑤"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    7.  <a id="ref-for-serialize-a-css-value②"></a>[Serialize a CSS value](#serialize-a-css-value) with <var>used longhands</var>, and let <var>value</var> be the result.

    8.  If <var>value</var> is the empty string, <a id="ref-for-iteration-continue⑥"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    9.  <a id="ref-for-set-append①"></a>[Append](https://infra.spec.whatwg.org/#set-append) the property names of all the items in <var>used longhands</var> to <var>already serialized</var>.

    10. <a id="ref-for-serialize-a-css-declaration①"></a>[Serialize a CSS declaration](#serialize-a-css-declaration) with property name <var>shorthand</var>, value <var>value</var>, and the <a id="ref-for-css-declaration-important-flag②"></a>[important flag](#css-declaration-important-flag) set if all the <var>used longhands</var> have it set. Return the result.

3.  Return <a id="ref-for-failure①"></a>[failure](https://infra.spec.whatwg.org/#failure).

To get a list of <a id="intermixed-properties"></a><strong>intermixed properties</strong> in a list of declarations <var>decls</var> between a set of properties <var>props</var>:

1.  Find the first and last declarations in <var>decls</var> that belong to <var>props</var>.

2.  Return the slice of <var>decls</var> between (and including) those two declarations.

Note: The serialization of an empty CSS declaration block is the empty string.

Note: The serialization of a non-empty CSS declaration block does not include any surrounding whitespace, i.e., no whitespace appears before the first property name and no whitespace appears after the final semicolon delimiter that follows the last property value.

Tests

- [border-shorthand-serialization.html](https://wpt.fyi/results/css/cssom/border-shorthand-serialization.html) [(live test)](http://wpt.live/css/cssom/border-shorthand-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/border-shorthand-serialization.html)
- [css-style-attr-decl-block.html](https://wpt.fyi/results/css/cssom/css-style-attr-decl-block.html) [(live test)](http://wpt.live/css/cssom/css-style-attr-decl-block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/css-style-attr-decl-block.html)
- [font-family-serialization-001.html](https://wpt.fyi/results/css/cssom/font-family-serialization-001.html) [(live test)](http://wpt.live/css/cssom/font-family-serialization-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/font-family-serialization-001.html)
- [font-shorthand-serialization.html](https://wpt.fyi/results/css/cssom/font-shorthand-serialization.html) [(live test)](http://wpt.live/css/cssom/font-shorthand-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/font-shorthand-serialization.html)
- [font-variant-shorthand-serialization.html](https://wpt.fyi/results/css/cssom/font-variant-shorthand-serialization.html) [(live test)](http://wpt.live/css/cssom/font-variant-shorthand-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/font-variant-shorthand-serialization.html)
- [shorthand-serialization.html](https://wpt.fyi/results/css/cssom/shorthand-serialization.html) [(live test)](http://wpt.live/css/cssom/shorthand-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/shorthand-serialization.html)
- [shorthand-values.html](https://wpt.fyi/results/css/cssom/shorthand-values.html) [(live test)](http://wpt.live/css/cssom/shorthand-values.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/shorthand-values.html)

A <a id="ref-for-css-declaration-block⑤"></a>[CSS declaration block](#css-declaration-block) has these <a id="ref-for-concept-element-attributes-change-ext"></a>[attribute change steps](https://dom.spec.whatwg.org/#concept-element-attributes-change-ext) for its <a id="ref-for-cssstyledeclaration-owner-node④"></a>[owner node](#cssstyledeclaration-owner-node) with <var>localName</var>, <var>value</var>, and <var>namespace</var>:

1.  If the <a id="ref-for-cssstyledeclaration-computed-flag③"></a>[computed flag](#cssstyledeclaration-computed-flag) is set, then return.
2.  If the <a id="ref-for-cssstyledeclaration-updating-flag"></a>[updating flag](#cssstyledeclaration-updating-flag) is set, then return.
3.  If <var>localName</var> is not "<code>style</code>", or <var>namespace</var> is not null, then return.
4.  If <var>value</var> is null, empty the <a id="ref-for-cssstyledeclaration-declarations④"></a>[declarations](#cssstyledeclaration-declarations).
5.  Otherwise, let the <a id="ref-for-cssstyledeclaration-declarations⑤"></a>[declarations](#cssstyledeclaration-declarations) be the result of <a id="ref-for-parse-a-css-declaration-block①"></a>[parse a CSS declaration block](#parse-a-css-declaration-block) from a string <var>value</var>.

When a <a id="ref-for-css-declaration-block⑥"></a>[CSS declaration block](#css-declaration-block) object is created, then:

1.  Let <var>owner node</var> be the <a id="ref-for-cssstyledeclaration-owner-node⑤"></a>[owner node](#cssstyledeclaration-owner-node).
2.  If <var>owner node</var> is null, or the <a id="ref-for-cssstyledeclaration-computed-flag④"></a>[computed flag](#cssstyledeclaration-computed-flag) is set, then return.
3.  Let <var>value</var> be the result of <a id="ref-for-concept-element-attributes-get-by-namespace"></a>[getting an attribute](https://dom.spec.whatwg.org/#concept-element-attributes-get-by-namespace) given null, "<code>style</code>", and <var>owner node</var>.
4.  If <var>value</var> is not null, let the <a id="ref-for-cssstyledeclaration-declarations⑥"></a>[declarations](#cssstyledeclaration-declarations) be the result of <a id="ref-for-parse-a-css-declaration-block②"></a>[parse a CSS declaration block](#parse-a-css-declaration-block) from a string <var>value</var>.

To <a id="update-style-attribute-for"></a><strong>update style attribute for</strong> <var>declaration block</var> means to run the steps below:

1.  Assert: <var>declaration block</var>’s <a id="ref-for-cssstyledeclaration-computed-flag⑤"></a>[computed flag](#cssstyledeclaration-computed-flag) is unset.
2.  Let <var>owner node</var> be <var>declaration block</var>’s <a id="ref-for-cssstyledeclaration-owner-node⑥"></a>[owner node](#cssstyledeclaration-owner-node).
3.  If <var>owner node</var> is null, then return.
4.  Set <var>declaration block</var>’s <a id="ref-for-cssstyledeclaration-updating-flag①"></a>[updating flag](#cssstyledeclaration-updating-flag).
5.  <a id="ref-for-concept-element-attributes-set-value"></a>[Set an attribute value](https://dom.spec.whatwg.org/#concept-element-attributes-set-value) for <var>owner node</var> using "<code>style</code>" and the result of <a id="ref-for-serialize-a-css-declaration-block②"></a>[serializing](#serialize-a-css-declaration-block) <var>declaration block</var>.
6.  Unset <var>declaration block</var>’s <a id="ref-for-cssstyledeclaration-updating-flag②"></a>[updating flag](#cssstyledeclaration-updating-flag).

The <a id="concept-shorthands-preferred-order"></a><strong>preferred shorthand order</strong> of a list of shorthand properties <var>shorthands</var> is as follows:

1.  Order <var>shorthands</var> lexicographically.

2.  Remove all items in <var>shorthands</var> that are <a id="ref-for-legacy-shorthand"></a>[legacy shorthands](https://drafts.csswg.org/css-cascade-5/#legacy-shorthand) but do not begin with "<code>-</code>" (U+002D).

3.  Move all items in <var>shorthands</var> that begin with "<code>-webkit-</code>" (U+002D) last in the list, retaining their relative order.

4.  Move all items in <var>shorthands</var> that begin with "<code>-</code>" (U+002D) but do not begin with "<code>-webkit-</code>" last in the list, retaining their relative order.

5.  Order <var>shorthands</var> by the number of longhand properties that map to it, with the greatest number first,

retaining their relative order.

#### <a id="the-cssstyledeclaration-interface"></a>6.6.1. The <code><a id="ref-for-cssstyledeclaration①"></a>[CSSStyleDeclaration](#cssstyledeclaration)</code> Interface[](#the-cssstyledeclaration-interface)

The <code>CSSStyleDeclaration</code> interface represents a <a id="ref-for-css-declaration-block⑦"></a>[CSS declaration block](#css-declaration-block), including its underlying state, where this underlying state depends upon the source of the <code>CSSStyleDeclaration</code> instance.

<a id="ref-for-Exposed①③"></a><a id="cssstyledeclaration"></a><a id="ref-for-cereactions"></a><a id="ref-for-cssomstring②⑨"></a><a id="ref-for-dom-cssstyledeclaration-csstext"></a><a id="ref-for-idl-unsigned-long①④"></a><a id="ref-for-dom-cssstyledeclaration-length"></a><a id="ref-for-cssomstring③⓪"></a><a id="ref-for-dom-cssstyledeclaration-item"></a><a id="ref-for-idl-unsigned-long①⑤"></a><a id="dom-cssstyledeclaration-item-index-index"></a><a id="ref-for-cssomstring③①"></a><a id="ref-for-dom-cssstyledeclaration-getpropertyvalue"></a><a id="ref-for-cssomstring③②"></a><a id="dom-cssstyledeclaration-getpropertyvalue-property-property"></a><a id="ref-for-cssomstring③③"></a><a id="ref-for-dom-cssstyledeclaration-getpropertypriority"></a><a id="ref-for-cssomstring③④"></a><a id="dom-cssstyledeclaration-getpropertypriority-property-property"></a><a id="ref-for-cereactions①"></a><a id="ref-for-idl-undefined⑥"></a><a id="ref-for-dom-cssstyledeclaration-setproperty"></a><a id="ref-for-cssomstring③⑤"></a><a id="dom-cssstyledeclaration-setproperty-property-value-priority-property"></a><a id="ref-for-LegacyNullToEmptyString①⑤"></a><a id="ref-for-cssomstring③⑥"></a><a id="dom-cssstyledeclaration-setproperty-property-value-priority-value"></a><a id="ref-for-LegacyNullToEmptyString①⑥"></a><a id="ref-for-cssomstring③⑦"></a><a id="dom-cssstyledeclaration-setproperty-property-value-priority-priority"></a><a id="ref-for-cereactions②"></a><a id="ref-for-cssomstring③⑧"></a><a id="ref-for-dom-cssstyledeclaration-removeproperty"></a><a id="ref-for-cssomstring③⑨"></a><a id="dom-cssstyledeclaration-removeproperty-property-property"></a><a id="ref-for-cssrule①④"></a><a id="ref-for-dom-cssstyledeclaration-parentrule"></a><a id="ref-for-Exposed①④"></a><a id="cssstyleproperties"></a><a id="ref-for-cssstyledeclaration②"></a><a id="ref-for-cereactions③"></a><a id="ref-for-LegacyNullToEmptyString①⑦"></a><a id="ref-for-cssomstring④⓪"></a><a id="ref-for-dom-cssstyleproperties-cssfloat"></a>

``` text
[Exposed=Window]
interface CSSStyleDeclaration {
  [CEReactions] attribute CSSOMString cssText;
  readonly attribute unsigned long length;
  getter CSSOMString item(unsigned long index);
  CSSOMString getPropertyValue(CSSOMString property);
  CSSOMString getPropertyPriority(CSSOMString property);
  [CEReactions] undefined setProperty(CSSOMString property, [LegacyNullToEmptyString] CSSOMString value, optional [LegacyNullToEmptyString] CSSOMString priority = "");
  [CEReactions] CSSOMString removeProperty(CSSOMString property);
  readonly attribute CSSRule? parentRule;
};

[Exposed=Window]
interface CSSStyleProperties : CSSStyleDeclaration {
  [CEReactions] attribute [LegacyNullToEmptyString] CSSOMString cssFloat;
};
```

Tests

- [css-style-declaration-modifications.html](https://wpt.fyi/results/css/cssom/css-style-declaration-modifications.html) [(live test)](http://wpt.live/css/cssom/css-style-declaration-modifications.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/css-style-declaration-modifications.html)
- [cssom-cssstyledeclaration-set.html](https://wpt.fyi/results/css/cssom/cssom-cssstyledeclaration-set.html) [(live test)](http://wpt.live/css/cssom/cssom-cssstyledeclaration-set.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssom-cssstyledeclaration-set.html)
- [cssstyledeclaration-all-shorthand.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-all-shorthand.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-all-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-all-shorthand.html)
- [cssstyledeclaration-cssfontrule.tentative.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-cssfontrule.tentative.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-cssfontrule.tentative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-cssfontrule.tentative.html)
- [cssstyledeclaration-csstext-all-shorthand.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-csstext-all-shorthand.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-csstext-all-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-csstext-all-shorthand.html)
- [cssstyledeclaration-csstext-final-delimiter.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-csstext-final-delimiter.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-csstext-final-delimiter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-csstext-final-delimiter.html)
- [cssstyledeclaration-csstext-important.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-csstext-important.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-csstext-important.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-csstext-important.html)
- [cssstyledeclaration-csstext.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-csstext.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-csstext.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-csstext.html)
- [cssstyledeclaration-custom-properties.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-custom-properties.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-custom-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-custom-properties.html)
- [cssstyledeclaration-mutability.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-mutability.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-mutability.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-mutability.html)
- [cssstyledeclaration-mutationrecord-001.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-mutationrecord-001.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-mutationrecord-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-mutationrecord-001.html)
- [cssstyledeclaration-mutationrecord-002.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-mutationrecord-002.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-mutationrecord-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-mutationrecord-002.html)
- [cssstyledeclaration-mutationrecord-003.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-mutationrecord-003.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-mutationrecord-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-mutationrecord-003.html)
- [cssstyledeclaration-mutationrecord-004.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-mutationrecord-004.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-mutationrecord-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-mutationrecord-004.html)
- [cssstyledeclaration-mutationrecord-005.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-mutationrecord-005.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-mutationrecord-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-mutationrecord-005.html)
- [cssstyledeclaration-properties.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-properties.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-properties.html)
- [cssstyledeclaration-registered-custom-properties.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-registered-custom-properties.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-registered-custom-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-registered-custom-properties.html)
- [cssstyledeclaration-removeProperty-all.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-removeProperty-all.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-removeProperty-all.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-removeProperty-all.html)
- [cssstyledeclaration-setter-attr.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-setter-attr.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-setter-attr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-setter-attr.html)
- [cssstyledeclaration-setter-declarations.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-setter-declarations.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-setter-declarations.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-setter-declarations.html)
- [cssstyledeclaration-setter-form-controls.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-setter-form-controls.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-setter-form-controls.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-setter-form-controls.html)
- [cssstyledeclaration-setter-logical.html](https://wpt.fyi/results/css/cssom/cssstyledeclaration-setter-logical.html) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-setter-logical.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-setter-logical.html)
- [page-descriptors.html](https://wpt.fyi/results/css/cssom/page-descriptors.html) [(live test)](http://wpt.live/css/cssom/page-descriptors.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/page-descriptors.html)
- [property-accessors.html](https://wpt.fyi/results/css/cssom/property-accessors.html) [(live test)](http://wpt.live/css/cssom/property-accessors.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/property-accessors.html)

The object’s <a id="ref-for-dfn-supported-property-indices⑥"></a>[supported property indices](http://heycam.github.io/webidl/#dfn-supported-property-indices) are the numbers in the range zero to one less than the number of <a id="ref-for-css-declaration⑤"></a>[CSS declarations](#css-declaration) in the <a id="ref-for-cssstyledeclaration-declarations⑦"></a>[declarations](#cssstyledeclaration-declarations). If there are no such <a id="ref-for-css-declaration⑥"></a>CSS declarations, then there are no <a id="ref-for-dfn-supported-property-indices⑦"></a>supported property indices.

Getting the <a id="dom-cssstyledeclaration-csstext"></a><strong><code>cssText</code></strong> attribute must run these steps:

1.  If the <a id="ref-for-cssstyledeclaration-computed-flag⑥"></a>[computed flag](#cssstyledeclaration-computed-flag) is set, then return the empty string.

2.  Return the result of <a id="ref-for-serialize-a-css-declaration-block③"></a>[serializing](#serialize-a-css-declaration-block) the <a id="ref-for-cssstyledeclaration-declarations⑧"></a>[declarations](#cssstyledeclaration-declarations).

Setting the <code><a id="ref-for-dom-cssstyledeclaration-csstext①"></a>[cssText](#dom-cssstyledeclaration-csstext)</code> attribute must run these steps:

1.  If the <a id="ref-for-cssstyledeclaration-readonly-flag③"></a>[readonly flag](#cssstyledeclaration-readonly-flag) is set, then <a id="ref-for-dfn-throw①①"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) a <code><a id="ref-for-nomodificationallowederror"></a>[NoModificationAllowedError](https://webidl.spec.whatwg.org/#nomodificationallowederror)</code> exception.
2.  Empty the <a id="ref-for-cssstyledeclaration-declarations⑨"></a>[declarations](#cssstyledeclaration-declarations).
3.  <a id="ref-for-parse-a-css-declaration-block③"></a>[Parse](#parse-a-css-declaration-block) the given value and, if the return value is not the empty list, insert the items in the list into the <a id="ref-for-cssstyledeclaration-declarations①⓪"></a>[declarations](#cssstyledeclaration-declarations), in <a id="ref-for-concept-declarations-specified-order③"></a>[specified order](#concept-declarations-specified-order).
4.  <a id="ref-for-update-style-attribute-for"></a>[Update style attribute for](#update-style-attribute-for) the <a id="ref-for-css-declaration-block⑧"></a>[CSS declaration block](#css-declaration-block).

The <a id="dom-cssstyledeclaration-length"></a><strong><code>length</code></strong> attribute must return the number of <a id="ref-for-css-declaration⑦"></a>[CSS declarations](#css-declaration) in the <a id="ref-for-cssstyledeclaration-declarations①①"></a>[declarations](#cssstyledeclaration-declarations).

The <a id="dom-cssstyledeclaration-item"></a><strong><code>item(<var>index</var>)</code></strong> method must return the <a id="ref-for-css-declaration-property-name②"></a>[property name](#css-declaration-property-name) of the <a id="ref-for-css-declaration⑧"></a>[CSS declaration](#css-declaration) at position <var>index</var>. If there is no <var>index</var>th object in the collection, then the method must return the empty string.

The <a id="dom-cssstyledeclaration-getpropertyvalue"></a><strong><code>getPropertyValue(<var>property</var>)</code></strong> method must run these steps:

1.  If <var>property</var> is not a <a id="ref-for-custom-property①"></a>[custom property](https://drafts.csswg.org/css-variables-1/#custom-property), follow these substeps:
    1.  Let <var>property</var> be <var>property</var> <a id="ref-for-ascii-lowercase②"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).
    2.  If <var>property</var> is a shorthand property, then follow these substeps:
        1.  Let <var>list</var> be a new empty array.
        2.  For each longhand property <var>longhand</var> that <var>property</var> maps to, in canonical order, follow these substeps:
            1.  If <var>longhand</var> is a <a id="ref-for-dfn-case-sensitive⑧"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for a <a id="ref-for-css-declaration-property-name③"></a>[property name](#css-declaration-property-name) of a <a id="ref-for-css-declaration⑨"></a>[CSS declaration](#css-declaration) in the <a id="ref-for-cssstyledeclaration-declarations①②"></a>[declarations](#cssstyledeclaration-declarations), let <var>declaration</var> be that <a id="ref-for-css-declaration①⓪"></a>CSS declaration, or null otherwise.
            2.  If <var>declaration</var> is null, then return the empty string.
            3.  Append the <var>declaration</var> to <var>list</var>.
        3.  If <a id="ref-for-css-declaration-important-flag③"></a>[important flags](#css-declaration-important-flag) of all declarations in <var>list</var> are same, then return the <a id="ref-for-serialize-a-css-value③"></a>[serialization](#serialize-a-css-value) of <var>list</var>.
        4.  Return the empty string.
2.  If <var>property</var> is a <a id="ref-for-dfn-case-sensitive⑨"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for a <a id="ref-for-css-declaration-property-name④"></a>[property name](#css-declaration-property-name) of a <a id="ref-for-css-declaration①①"></a>[CSS declaration](#css-declaration) in the <a id="ref-for-cssstyledeclaration-declarations①③"></a>[declarations](#cssstyledeclaration-declarations), then return the result of invoking <a id="ref-for-serialize-a-css-value④"></a>[serialize a CSS value](#serialize-a-css-value) of that declaration.
3.  Return the empty string.

Tests

- [cssom-getPropertyValue-common-checks.html](https://wpt.fyi/results/css/cssom/cssom-getPropertyValue-common-checks.html) [(live test)](http://wpt.live/css/cssom/cssom-getPropertyValue-common-checks.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssom-getPropertyValue-common-checks.html)
- [serialize-all-longhands.html](https://wpt.fyi/results/css/cssom/serialize-all-longhands.html) [(live test)](http://wpt.live/css/cssom/serialize-all-longhands.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/serialize-all-longhands.html)

The <a id="dom-cssstyledeclaration-getpropertypriority"></a><strong><code>getPropertyPriority(<var>property</var>)</code></strong> method must run these steps:

1.  If <var>property</var> is not a <a id="ref-for-custom-property②"></a>[custom property](https://drafts.csswg.org/css-variables-1/#custom-property), follow these substeps:
    1.  Let <var>property</var> be <var>property</var> <a id="ref-for-ascii-lowercase③"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).
    2.  If <var>property</var> is a shorthand property, follow these substeps:
        1.  Let <var>list</var> be a new array.
        2.  For each longhand property <var>longhand</var> that <var>property</var> maps to, append the result of invoking <code><a id="ref-for-dom-cssstyledeclaration-getpropertypriority①"></a>[getPropertyPriority()](#dom-cssstyledeclaration-getpropertypriority)</code> with <var>longhand</var> as argument to <var>list</var>.
        3.  If all items in <var>list</var> are the string "<code>important</code>", then return the string "<code>important</code>".
2.  If <var>property</var> is a <a id="ref-for-dfn-case-sensitive①⓪"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for a <a id="ref-for-css-declaration-property-name⑤"></a>[property name](#css-declaration-property-name) of a <a id="ref-for-css-declaration①②"></a>[CSS declaration](#css-declaration) in the <a id="ref-for-cssstyledeclaration-declarations①④"></a>[declarations](#cssstyledeclaration-declarations) that has the <a id="ref-for-css-declaration-important-flag④"></a>[important flag](#css-declaration-important-flag) set, return the string "<code>important</code>".
3.  Return the empty string.

<a id="example-666534ab"></a>

<strong>Example:</strong>

[](#example-666534ab)E.g. for <code>background-color&#58;lime !IMPORTANT</code> the return value would be "<code>important</code>".

The <a id="dom-cssstyledeclaration-setproperty"></a><strong><code>setProperty(<var>property</var>, <var>value</var>, <var>priority</var>)</code></strong> method must run these steps:

1.  If the <a id="ref-for-cssstyledeclaration-readonly-flag④"></a>[readonly flag](#cssstyledeclaration-readonly-flag) is set, then <a id="ref-for-dfn-throw①②"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) a <code><a id="ref-for-nomodificationallowederror①"></a>[NoModificationAllowedError](https://webidl.spec.whatwg.org/#nomodificationallowederror)</code> exception.

2.  If <var>property</var> is not a <a id="ref-for-custom-property③"></a>[custom property](https://drafts.csswg.org/css-variables-1/#custom-property), follow these substeps:
    1.  Let <var>property</var> be <var>property</var> <a id="ref-for-ascii-lowercase④"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).
    2.  If <var>property</var> is not a <a id="ref-for-dfn-case-sensitive①①"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for a <a id="ref-for-supported-css-property①"></a>[supported CSS property](#supported-css-property), then return.

3.  If <var>value</var> is the empty string, invoke <code><a id="ref-for-dom-cssstyledeclaration-removeproperty①"></a>[removeProperty()](#dom-cssstyledeclaration-removeproperty)</code> with <var>property</var> as argument and return.

4.  If <var>priority</var> is not the empty string and is not an <a id="ref-for-ascii-case-insensitive④"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string "<code>important</code>", then return.

5.  Let <var>component value list</var> be the result of <a id="ref-for-parse-a-css-value"></a>[parsing](#parse-a-css-value) <var>value</var> for property <var>property</var>.

    Note: <var>value</var> can not include "<code>!important</code>".

6.  If <var>component value list</var> is null, then return.

7.  Let <var>updated</var> be false.

8.  If <var>property</var> is a shorthand property, then for each longhand property <var>longhand</var> that <var>property</var> maps to, in canonical order, follow these substeps:
    1.  Let <var>longhand result</var> be the result of <a id="ref-for-set-a-css-declaration"></a>[set the CSS declaration](#set-a-css-declaration) <var>longhand</var> with the appropriate value(s) from <var>component value list</var>, with the <em>important</em> flag set if <var>priority</var> is not the empty string, and unset otherwise, and with the list of declarations being the <a id="ref-for-cssstyledeclaration-declarations①⑤"></a>[declarations](#cssstyledeclaration-declarations).
    2.  If <var>longhand result</var> is true, let <var>updated</var> be true.

9.  Otherwise, let <var>updated</var> be the result of <a id="ref-for-set-a-css-declaration①"></a>[set the CSS declaration](#set-a-css-declaration) <var>property</var> with value <var>component value list</var>, with the <em>important</em> flag set if <var>priority</var> is not the empty string, and unset otherwise, and with the list of declarations being the <a id="ref-for-cssstyledeclaration-declarations①⑥"></a>[declarations](#cssstyledeclaration-declarations).

10. If <var>updated</var> is true, <a id="ref-for-update-style-attribute-for①"></a>[update style attribute for](#update-style-attribute-for) the <a id="ref-for-css-declaration-block⑨"></a>[CSS declaration block](#css-declaration-block).

Tests

- [cssom-setProperty-shorthand.html](https://wpt.fyi/results/css/cssom/cssom-setProperty-shorthand.html) [(live test)](http://wpt.live/css/cssom/cssom-setProperty-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssom-setProperty-shorthand.html)
- [MutationObserver-style.html](https://wpt.fyi/results/css/cssom/MutationObserver-style.html) [(live test)](http://wpt.live/css/cssom/MutationObserver-style.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/MutationObserver-style.html)
- [rule-restrictions.html](https://wpt.fyi/results/css/cssom/rule-restrictions.html) [(live test)](http://wpt.live/css/cssom/rule-restrictions.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/rule-restrictions.html)
- [setproperty-null-undefined.html](https://wpt.fyi/results/css/cssom/setproperty-null-undefined.html) [(live test)](http://wpt.live/css/cssom/setproperty-null-undefined.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/setproperty-null-undefined.html)

To <a id="set-a-css-declaration"></a><strong>set a CSS declaration</strong> <var>property</var> with a value <var>component value list</var> and optionally with an <em>important</em> flag set, in a list of declarations <var>declarations</var>, the user agent must ensure the following constraints hold after its steps:

- Exactly one <a id="ref-for-css-declaration①③"></a>[CSS declaration](#css-declaration) whose <a id="ref-for-css-declaration-property-name⑥"></a>[property name](#css-declaration-property-name) is a <a id="ref-for-dfn-case-sensitive①②"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match of <var>property</var> must exist in <var>declarations</var>. Such declaration is referenced as the <var>target declaration</var> below.
- The <var>target declaration</var> must have value being <var>component value list</var>, and <var>target declaration</var>’s <a id="ref-for-css-declaration-important-flag⑤"></a>[important flag](#css-declaration-important-flag) must be <a id="ref-for-set②"></a>[set](#set) if <em>important</em> flag is set, and <a id="ref-for-unset"></a>[unset](#unset) otherwise.
- Any <a id="ref-for-css-declaration①④"></a>[CSS declaration](#css-declaration) which is not the <var>target declaration</var> must not be changed, inserted, or removed from <var>declarations</var>.
- If there are <a id="ref-for-css-declaration①⑤"></a>[CSS declarations](#css-declaration) in <var>declarations</var> whose <a id="ref-for-css-declaration-property-name⑦"></a>[property name](#css-declaration-property-name) is in the same <a id="ref-for-logical-property-group①"></a>[logical property group](https://drafts.csswg.org/css-logical-1/#logical-property-group) as <var>property</var>, but has a different <a id="ref-for-mapping-logic①"></a>[mapping logic](https://drafts.csswg.org/css-logical-1/#mapping-logic), <var>target declaration</var> must be at an index after all of those <a id="ref-for-css-declaration①⑥"></a>CSS declarations.
- The steps must return true if the serialization of <var>declarations</var> was changed as result of the steps. It may return false otherwise.

<a id="issue-aaf411cc"></a>

<strong>Issue:</strong>

[](#issue-aaf411cc) Should we add something like "Any observable side effect must not be made outside <var>declarations</var>"? The current constraints sound like a hole for undefined behavior.

Note: The steps of <a id="ref-for-set-a-css-declaration②"></a>[set a CSS declaration](#set-a-css-declaration) are not defined in this level of CSSOM. User agents may use different algorithms as long as the constraints above hold.

<a id="example-a40690cb"></a>

<strong>Example:</strong>

[](#example-a40690cb) The simplest way to conform with the constraints would be to always remove any existing declaration matching <var>property</var>, and append the new declaration to the end. But based on implementation feedback, this approach would likely regress performance.

Another possible algorithm is:

1.  If <var>property</var> is a <a id="ref-for-dfn-case-sensitive①③"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for a <a id="ref-for-css-declaration-property-name⑧"></a>[property name](#css-declaration-property-name) of a <a id="ref-for-css-declaration①⑦"></a>[CSS declaration](#css-declaration) in <var>declarations</var>, follow these substeps:
    1.  Let <var>target declaration</var> be such <a id="ref-for-css-declaration①⑧"></a>[CSS declaration](#css-declaration).
    2.  Let <var>needs append</var> be false.
    3.  <a id="ref-for-list-iterate"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>declaration</var> in <var>declarations</var> after <var>target declaration</var>:
        1.  If <var>declaration</var>’s <a id="ref-for-css-declaration-property-name⑨"></a>[property name](#css-declaration-property-name) is not in the same <a id="ref-for-logical-property-group②"></a>[logical property group](https://drafts.csswg.org/css-logical-1/#logical-property-group) as <var>property</var>, then <a id="ref-for-iteration-continue⑦"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).
        2.  If <var>declaration</var>’ <a id="ref-for-css-declaration-property-name①⓪"></a>[property name](#css-declaration-property-name) has the same <a id="ref-for-mapping-logic②"></a>[mapping logic](https://drafts.csswg.org/css-logical-1/#mapping-logic) as <var>property</var>, then <a id="ref-for-iteration-continue⑧"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).
        3.  Let <var>needs append</var> be true.
        4.  <a id="ref-for-break"></a>[Break](https://drafts.csswg.org/css-break-4/#break).
    4.  If <var>needs append</var> is false, then:
        1.  Let <var>needs update</var> be false.
        2.  If <var>target declaration</var>’s <a id="ref-for-css-declaration-value"></a>[value](#css-declaration-value) is not equal to <var>component value list</var>, then let <var>needs update</var> be true.
        3.  If <var>target declaration</var>’s <a id="ref-for-css-declaration-important-flag⑥"></a>[important flag](#css-declaration-important-flag) is not equal to whether <em>important</em> flag is set, then let <var>needs update</var> be true.
        4.  If <var>needs update</var> is false, then return false.
        5.  Set <var>target declaration</var>’s <a id="ref-for-css-declaration-value①"></a>[value](#css-declaration-value) to <var>component value list</var>.
        6.  If <em>important</em> flag is set, then set <var>target declaration</var>’s <a id="ref-for-css-declaration-important-flag⑦"></a>[important flag](#css-declaration-important-flag), otherwise unset it.
        7.  Return true.
    5.  Otherwise, remove <var>target declaration</var> from <var>declarations</var>.
2.  Append a new <a id="ref-for-css-declaration①⑨"></a>[CSS declaration](#css-declaration) with <a id="ref-for-css-declaration-property-name①①"></a>[property name](#css-declaration-property-name) <var>property</var>, <a id="ref-for-css-declaration-value②"></a>[value](#css-declaration-value) <var>component value list</var>, and <a id="ref-for-css-declaration-important-flag⑧"></a>[important flag](#css-declaration-important-flag) set if <em>important</em> flag is set to <var>declarations</var>.
3.  Return true.

The <a id="dom-cssstyledeclaration-removeproperty"></a><strong><code>removeProperty(<var>property</var>)</code></strong> method must run these steps:

1.  If the <a id="ref-for-cssstyledeclaration-readonly-flag⑤"></a>[readonly flag](#cssstyledeclaration-readonly-flag) is set, then <a id="ref-for-dfn-throw①③"></a>[throw](http://heycam.github.io/webidl/#dfn-throw) a <code><a id="ref-for-nomodificationallowederror②"></a>[NoModificationAllowedError](https://webidl.spec.whatwg.org/#nomodificationallowederror)</code> exception.
2.  If <var>property</var> is not a <a id="ref-for-custom-property④"></a>[custom property](https://drafts.csswg.org/css-variables-1/#custom-property), let <var>property</var> be <var>property</var> <a id="ref-for-ascii-lowercase⑤"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).
3.  Let <var>value</var> be the return value of invoking <code><a id="ref-for-dom-cssstyledeclaration-getpropertyvalue①"></a>[getPropertyValue()](#dom-cssstyledeclaration-getpropertyvalue)</code> with <var>property</var> as argument.
4.  Let <var>removed</var> be false.
5.  If <var>property</var> is a shorthand property, for each longhand property <var>longhand</var> that <var>property</var> maps to:
    1.  If <var>longhand</var> is not a <a id="ref-for-css-declaration-property-name①②"></a>[property name](#css-declaration-property-name) of a <a id="ref-for-css-declaration②⓪"></a>[CSS declaration](#css-declaration) in the <a id="ref-for-cssstyledeclaration-declarations①⑦"></a>[declarations](#cssstyledeclaration-declarations), <a id="ref-for-iteration-continue⑨"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).
    2.  Remove that <a id="ref-for-css-declaration②①"></a>[CSS declaration](#css-declaration) and let <var>removed</var> be true.
6.  Otherwise, if <var>property</var> is a <a id="ref-for-dfn-case-sensitive①④"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive) match for a <a id="ref-for-css-declaration-property-name①③"></a>[property name](#css-declaration-property-name) of a <a id="ref-for-css-declaration②②"></a>[CSS declaration](#css-declaration) in the <a id="ref-for-cssstyledeclaration-declarations①⑧"></a>[declarations](#cssstyledeclaration-declarations), remove that <a id="ref-for-css-declaration②③"></a>CSS declaration and let <var>removed</var> be true.
7.  If <var>removed</var> is true, <a id="ref-for-update-style-attribute-for②"></a>[Update style attribute for](#update-style-attribute-for) the <a id="ref-for-css-declaration-block①⓪"></a>[CSS declaration block](#css-declaration-block).
8.  Return <var>value</var>.

The <a id="dom-cssstyledeclaration-parentrule"></a><strong><code>parentRule</code></strong> attribute must return the <a id="ref-for-cssstyledeclaration-parent-css-rule③"></a>[parent CSS rule](#cssstyledeclaration-parent-css-rule).

The <a id="dom-cssstyleproperties-cssfloat"></a><strong><code>cssFloat</code></strong> attribute, on getting, must return the result of invoking <code><a id="ref-for-dom-cssstyledeclaration-getpropertyvalue②"></a>[getPropertyValue()](#dom-cssstyledeclaration-getpropertyvalue)</code> with <code>float</code> as argument. On setting, the attribute must invoke <code><a id="ref-for-dom-cssstyledeclaration-setproperty①"></a>[setProperty()](#dom-cssstyledeclaration-setproperty)</code> with <code>float</code> as first argument, as second argument the given value, and no third argument. Any exceptions thrown must be re-thrown.

For each CSS property <var>property</var> that is a <a id="ref-for-supported-css-property②"></a>[supported CSS property](#supported-css-property), the following partial interface applies where <a id="dom-cssstyleproperties-camel-cased-attribute"></a><strong><code>camel-cased attribute</code></strong> is obtained by running the <a id="ref-for-css-property-to-idl-attribute"></a>[CSS property to IDL attribute](#css-property-to-idl-attribute) algorithm for <var>property</var>.

Tests

- [change-rule-with-layers-crash.html](https://wpt.fyi/results/css/cssom/change-rule-with-layers-crash.html) [(live test)](http://wpt.live/css/cssom/change-rule-with-layers-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/change-rule-with-layers-crash.html)
- [style-attr-update-across-documents.html](https://wpt.fyi/results/css/cssom/style-attr-update-across-documents.html) [(live test)](http://wpt.live/css/cssom/style-attr-update-across-documents.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/style-attr-update-across-documents.html)

<a id="ref-for-cssstyleproperties②"></a><a id="ref-for-cereactions④"></a><a id="ref-for-LegacyNullToEmptyString①⑧"></a><a id="ref-for-cssomstring④①"></a><a id="dom-cssstyleproperties-camel&#95;cased&#95;attribute"></a>

``` text
partial interface CSSStyleProperties {
  [CEReactions] attribute [LegacyNullToEmptyString] CSSOMString _camel_cased_attribute;
};
```

The <a id="ref-for-dom-cssstyleproperties-camel-cased-attribute"></a>[<var>camel-cased attribute</var>](#dom-cssstyleproperties-camel-cased-attribute) attribute, on getting, must return the result of invoking <code><a id="ref-for-dom-cssstyledeclaration-getpropertyvalue③"></a>[getPropertyValue()](#dom-cssstyledeclaration-getpropertyvalue)</code> with the argument being the result of running the <a id="ref-for-idl-attribute-to-css-property"></a>[IDL attribute to CSS property](#idl-attribute-to-css-property) algorithm for <var>camel-cased attribute</var>.

Setting the <a id="ref-for-dom-cssstyleproperties-camel-cased-attribute①"></a>[<var>camel-cased attribute</var>](#dom-cssstyleproperties-camel-cased-attribute) attribute must invoke <code><a id="ref-for-dom-cssstyledeclaration-setproperty②"></a>[setProperty()](#dom-cssstyledeclaration-setproperty)</code> with the first argument being the result of running the <a id="ref-for-idl-attribute-to-css-property①"></a>[IDL attribute to CSS property](#idl-attribute-to-css-property) algorithm for <var>camel-cased attribute</var>, as second argument the given value, and no third argument. Any exceptions thrown must be re-thrown.

<a id="example-5f92c57f"></a>

<strong>Example:</strong>

[](#example-5f92c57f)For example, for the <a id="ref-for-propdef-font-size"></a>[font-size](https://drafts.csswg.org/css-fonts-4/#propdef-font-size) property there would be a <code>fontSize</code> IDL attribute.

For each CSS property <var>property</var> that is a <a id="ref-for-supported-css-property③"></a>[supported CSS property](#supported-css-property) and that begins with the string <code>-webkit-</code>, the following partial interface applies where <var>webkit-cased attribute</var> is obtained by running the <a id="ref-for-css-property-to-idl-attribute①"></a>[CSS property to IDL attribute](#css-property-to-idl-attribute) algorithm for <var>property</var>, with the <em>lowercase first</em> flag set.

<a id="ref-for-cssstyleproperties③"></a><a id="ref-for-cereactions⑤"></a><a id="ref-for-LegacyNullToEmptyString①⑨"></a><a id="ref-for-cssomstring④②"></a><a id="dom-cssstyleproperties-webkit&#95;cased&#95;attribute"></a>

``` text
partial interface CSSStyleProperties {
  [CEReactions] attribute [LegacyNullToEmptyString] CSSOMString _webkit_cased_attribute;
};
```

The <a id="dom-cssstyleproperties-webkit-cased-attribute"></a><strong><code><var>webkit-cased attribute</var></code></strong> attribute, on getting, must return the result of invoking <code><a id="ref-for-dom-cssstyledeclaration-getpropertyvalue④"></a>[getPropertyValue()](#dom-cssstyledeclaration-getpropertyvalue)</code> with the argument being the result of running the <a id="ref-for-idl-attribute-to-css-property②"></a>[IDL attribute to CSS property](#idl-attribute-to-css-property) algorithm for <var>webkit-cased attribute</var>, with the <em>dash prefix</em> flag set.

Setting the <a id="ref-for-dom-cssstyledeclaration-webkit-cased-attribute"></a>[<var>webkit-cased attribute</var>](https://www.w3.org/TR/cssom-1/#dom-cssstyledeclaration-webkit-cased-attribute) attribute must invoke <code><a id="ref-for-dom-cssstyledeclaration-setproperty③"></a>[setProperty()](#dom-cssstyledeclaration-setproperty)</code> with the first argument being the result of running the <a id="ref-for-idl-attribute-to-css-property③"></a>[IDL attribute to CSS property](#idl-attribute-to-css-property) algorithm for <var>webkit-cased attribute</var>, with the <em>dash prefix</em> flag set, as second argument the given value, and no third argument. Any exceptions thrown must be re-thrown.

<a id="example-9b35d744"></a>

<strong>Example:</strong>

[](#example-9b35d744)For example, if the user agent supports the <a id="ref-for-propdef--webkit-transform"></a>[-webkit-transform](https://compat.spec.whatwg.org/#propdef--webkit-transform) property, there would be a <code>webkitTransform</code> IDL attribute. There would also be a <code>WebkitTransform</code> IDL attribute because of the rules for camel-cased attributes.

For each CSS property <var>property</var> that is a <a id="ref-for-supported-css-property④"></a>[supported CSS property](#supported-css-property), except for properties that have no "<code>-</code>" (U+002D) in the property name, the following partial interface applies where <var>dashed attribute</var> is <var>property</var>.

<a id="ref-for-cssstyleproperties④"></a><a id="ref-for-cereactions⑥"></a><a id="ref-for-LegacyNullToEmptyString②⓪"></a><a id="ref-for-cssomstring④③"></a><a id="dom-cssstyleproperties-dashed&#95;attribute"></a>

``` text
partial interface CSSStyleProperties {
  [CEReactions] attribute [LegacyNullToEmptyString] CSSOMString _dashed_attribute;
};
```

The <a id="dom-cssstyleproperties-dashed-attribute"></a><strong><code><var>dashed attribute</var></code></strong> attribute, on getting, must return the result of invoking <code><a id="ref-for-dom-cssstyledeclaration-getpropertyvalue⑤"></a>[getPropertyValue()](#dom-cssstyledeclaration-getpropertyvalue)</code> with the argument being <var>dashed attribute</var>.

Setting the <a id="ref-for-dom-cssstyleproperties-dashed-attribute"></a>[<var>dashed attribute</var>](#dom-cssstyleproperties-dashed-attribute) attribute must invoke <code><a id="ref-for-dom-cssstyledeclaration-setproperty④"></a>[setProperty()](#dom-cssstyledeclaration-setproperty)</code> with the first argument being <var>dashed attribute</var>, as second argument the given value, and no third argument. Any exceptions thrown must be re-thrown.

<a id="example-5db10c4f"></a>

<strong>Example:</strong>

[](#example-5db10c4f) For example, for the <a id="ref-for-propdef-font-size①"></a>[font-size](https://drafts.csswg.org/css-fonts-4/#propdef-font-size) property there would be a <code>font-size</code> IDL attribute. In JavaScript, the property can be accessed as follows, assuming <var>element</var> is an <a id="ref-for-html-elements"></a>[HTML element](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements):

``` text
element.style['font-size'];
```

The <a id="css-property-to-idl-attribute"></a><strong>CSS property to IDL attribute</strong> algorithm for <var>property</var>, optionally with a <em>lowercase first</em> flag set, is as follows:

1.  Let <var>output</var> be the empty string.
2.  Let <var>uppercase next</var> be unset.
3.  If the <em>lowercase first</em> flag is set, remove the first character from <var>property</var>.
4.  For each character <var>c</var> in <var>property</var>:
    1.  If <var>c</var> is "<code>-</code>" (U+002D), let <var>uppercase next</var> be set.
    2.  Otherwise, if <var>uppercase next</var> is set, let <var>uppercase next</var> be unset and append <var>c</var> <a id="ref-for-ascii-uppercase"></a>[converted to ASCII uppercase](https://infra.spec.whatwg.org/#ascii-uppercase) to <var>output</var>.
    3.  Otherwise, append <var>c</var> to <var>output</var>.
5.  Return <var>output</var>.

The <a id="idl-attribute-to-css-property"></a><strong>IDL attribute to CSS property</strong> algorithm for <var>attribute</var>, optionally with a <em>dash prefix</em> flag set, is as follows:

1.  Let <var>output</var> be the empty string.
2.  If the <em>dash prefix</em> flag is set, append "<code>-</code>" (U+002D) to <var>output</var>.
3.  For each character <var>c</var> in <var>attribute</var>:
    1.  If <var>c</var> is in the range U+0041 to U+005A (ASCII uppercase), append "<code>-</code>" (U+002D) followed by <var>c</var> <a id="ref-for-ascii-lowercase⑥"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase) to <var>output</var>.
    2.  Otherwise, append <var>c</var> to <var>output</var>.
4.  Return <var>output</var>.

### <a id="css-values"></a>6.7. CSS Values[](#css-values)

#### <a id="parsing-css-values"></a>6.7.1. Parsing CSS Values[](#parsing-css-values)

To <a id="parse-a-css-value"></a><strong>parse a CSS value</strong> <var>value</var> for a given <var>property</var> means to follow these steps:

1.  Let <var>list</var> be the value returned by invoking <a id="ref-for-parse-a-list-of-component-values"></a>[parse a list of component values](https://drafts.csswg.org/css-syntax-3/#parse-a-list-of-component-values) from <var>value</var>.
2.  Match <var>list</var> against the grammar for the property <var>property</var> in the CSS specification.
3.  If the above step failed, return null.
4.  Return <var>list</var>.

Note: "<code>!important</code>" declarations are not part of the property value space and will therefore cause <a id="ref-for-parse-a-css-value①"></a>[parse a CSS value](#parse-a-css-value) to return null.

#### <a id="serializing-css-values"></a>6.7.2. Serializing CSS Values[](#serializing-css-values)

To <a id="serialize-a-css-value"></a><strong>serialize a CSS value</strong> of a <a id="ref-for-css-declaration②④"></a>[CSS declaration](#css-declaration) <var>declaration</var> or a list of longhand <a id="ref-for-css-declaration②⑤"></a>CSS declarations <var>list</var>, follow these rules:

1.  If this algorithm is invoked with a <a id="ref-for-list②"></a>[list](https://infra.spec.whatwg.org/#list) <var>list</var>:

    1.  Let <var>shorthand</var> be the first shorthand property, in <a id="ref-for-concept-shorthands-preferred-order①"></a>[preferred shorthand order](#concept-shorthands-preferred-order), that exactly maps to all of the longhand properties in <var>list</var>.

    2.  If there is no such shorthand or <var>shorthand</var> cannot exactly represent the values of all the properties in <var>list</var>, return the empty string.

    3.  Otherwise, <a id="ref-for-serialize-a-css-value⑤"></a>[serialize a CSS value](#serialize-a-css-value) from a hypothetical declaration of the property <var>shorthand</var> with its value representing the combined values of the declarations in <var>list</var>.

2.  Represent the value of the <var>declaration</var> as a <a id="ref-for-list③"></a>[list](https://infra.spec.whatwg.org/#list) of CSS component values <var>components</var> that, when <a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>[parsed](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) according to the property’s grammar, would represent that value. Additionally, unless otherwise specified:

    - If certain component values can appear in any order without changing the meaning of the value (a pattern typically represented by a double bar <a id="ref-for-comb-any"></a>[\|\|](https://drafts.csswg.org/css-values-4/#comb-any) in the value syntax), reorder the component values to use the canonical order of component values as given in the property definition table.

    - If component values can be omitted or replaced with a shorter representation without changing the meaning of the value, omit/replace them.

    - If either of the above syntactic translations would be less backwards-compatible, do not perform them.

    Note: The rules described here outline the <em>general principles</em> of serialization. For legacy reasons, some properties serialize in a different manner, which is intentionally undefined here due to lack of resources. Please consult that property’s specification and/or your local reverse-engineer for details.

3.  Remove any <a id="ref-for-typedef-whitespace-token"></a>[\<whitespace-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-whitespace-token)s from <var>components</var>.

4.  Replace each component value in <var>components</var> with the result of invoking <a id="ref-for-serialize-a-css-component-value"></a>[serialize a CSS component value](#serialize-a-css-component-value).

5.  Join the items of <var>components</var> into a single string, inserting " " (U+0020 SPACE) between each pair of items unless the second item is a "," (U+002C COMMA) Return the result.

Tests

- [flex-serialization.html](https://wpt.fyi/results/css/cssom/flex-serialization.html) [(live test)](http://wpt.live/css/cssom/flex-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/flex-serialization.html)
- [overflow-serialization.html](https://wpt.fyi/results/css/cssom/overflow-serialization.html) [(live test)](http://wpt.live/css/cssom/overflow-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/overflow-serialization.html)
- [serialize-values.html](https://wpt.fyi/results/css/cssom/serialize-values.html) [(live test)](http://wpt.live/css/cssom/serialize-values.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/serialize-values.html)

To <a id="serialize-a-css-component-value"></a><strong>serialize a CSS component value</strong> depends on the component, as follows:

<strong><a id="ref-for-css-keyword"></a>[keyword](https://drafts.csswg.org/css-values-4/#css-keyword)</strong>

The keyword <a id="ref-for-ascii-lowercase⑦"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).

<strong><a id="ref-for-angle-value"></a>[\<angle\>](https://drafts.csswg.org/css-values-4/#angle-value)</strong>

The \<number\> component serialized as per \<number\> followed by the unit in canonical form as defined in its respective specification.

<a id="issue-87977b18"></a>

<strong>Issue:</strong>

[](#issue-87977b18) Probably should distinguish between declared and computed / resolved values.

<strong><a id="ref-for-typedef-color"></a>[\<color\>](https://drafts.csswg.org/css-color-5/#typedef-color)</strong>

If \<color\> is a component of a resolved value, see [CSS Color 4 §  15. Resolving \<color\> Values](https://drafts.csswg.org/css-color-4/#resolving-color-values).

If \<color\> is a component of a computed value, see [CSS Color 4 §  16. Serializing \<color\> Values](https://drafts.csswg.org/css-color-4/#serializing-color-values).

If <a id="ref-for-typedef-color①"></a>[\<color\>](https://drafts.csswg.org/css-color-5/#typedef-color) is a component of a declared value, then for sRGB values, see [CSS Color 4 § 15.1 Resolving sRGB values](https://drafts.csswg.org/css-color-4/#resolving-sRGB-values). For other color functions, see [CSS Color 4 §  15. Resolving \<color\> Values](https://drafts.csswg.org/css-color-4/#resolving-color-values).

<strong><a id="ref-for-typedef-color-alpha-value"></a>[\<alpha-value\>](https://drafts.csswg.org/css-color-4/#typedef-color-alpha-value)</strong>

See [CSS Color 4 § 16.1 Serializing alpha values](https://drafts.csswg.org/css-color-4/#serializing-alpha-values).

<strong><a id="ref-for-typedef-counter"></a>[\<counter\>](https://drafts.csswg.org/css-lists-3/#typedef-counter)</strong>

The return value of the following algorithm:

1.  Let <var>s</var> be the empty string.
2.  If \<counter\> has three CSS component values append the string "<code>counters(</code>" to <var>s</var>.
3.  If \<counter\> has two CSS component values append the string "<code>counter(</code>" to <var>s</var>.
4.  Let <var>list</var> be a list of CSS component values belonging to \<counter\>, omitting the last CSS component value if it is "decimal".
5.  Let each item in <var>list</var> be the result of invoking <a id="ref-for-serialize-a-css-component-value①"></a>[serialize a CSS component value](#serialize-a-css-component-value) on that item.
6.  Append the result of invoking <a id="ref-for-serialize-a-comma-separated-list④"></a>[serialize a comma-separated list](#serialize-a-comma-separated-list) on <var>list</var> to <var>s</var>.
7.  Append "<code>)</code>" (U+0029) to <var>s</var>.
8.  Return <var>s</var>.

<strong><a id="ref-for-frequency-value"></a>[\<frequency\>](https://drafts.csswg.org/css-values-4/#frequency-value)</strong>

The \<number\> component serialized as per \<number\> followed by the unit in its canonical form as defined in its respective specification.

<a id="issue-87977b18①"></a>

<strong>Issue:</strong>

[](#issue-87977b18%E2%91%A0) Probably should distinguish between declared and computed / resolved values.

<strong><a id="ref-for-value-def-identifier"></a>[\<identifier\>](https://drafts.csswg.org/css2/#value-def-identifier)</strong>

The identifier <a id="ref-for-serialize-an-identifier①⓪"></a>[serialized as an identifier](#serialize-an-identifier).

<strong><a id="ref-for-integer-value"></a>[\<integer\>](https://drafts.csswg.org/css-values-4/#integer-value)</strong>

A base-ten integer using digits 0-9 (U+0030 to U+0039) in the shortest form possible, preceded by "<code>-</code>" (U+002D) if it is negative.

<strong><a id="ref-for-length-value"></a>[\<length\>](https://drafts.csswg.org/css-values-4/#length-value)</strong>

The \<number\> component serialized as per \<number\> followed by the unit in its canonical form as defined in its respective specification.

<a id="issue-87977b18②"></a>

<strong>Issue:</strong>

[](#issue-87977b18%E2%91%A1) Probably should distinguish between declared and computed / resolved values.

<strong><a id="ref-for-number-value"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)</strong>

A base-ten number using digits 0-9 (U+0030 to U+0039) in the shortest form possible, using "<code>.</code>" to separate decimals (if any), rounding the value if necessary to not produce more than 6 decimals, preceded by "<code>-</code>" (U+002D) if it is negative.

Note: scientific notation is not used.

<strong><a id="ref-for-percentage-value"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value)</strong>

The \<number\> component serialized as per \<number\> followed by the literal string "<code>%</code>" (U+0025).

<strong><a id="ref-for-resolution-value"></a>[\<resolution\>](https://drafts.csswg.org/css-values-4/#resolution-value)</strong>

The resolution in dots per <a id="ref-for-px"></a>[CSS pixel](https://drafts.csswg.org/css-values-4/#px) serialized as per \<number\> followed by the literal string "<code>dppx</code>".

<strong><a id="ref-for-ratio-value"></a>[\<ratio\>](https://drafts.csswg.org/css-values-4/#ratio-value)</strong>

The numerator serialized as per \<number\> followed by the literal string "<code> / </code>", followed by the denominator serialized as per \<number\>.

<strong><a id="ref-for-value-def-shape"></a>[\<shape\>](https://drafts.csswg.org/css2/#value-def-shape)</strong>

The return value of the following algorithm:

1.  Let <var>s</var> be the string "<code>rect(</code>".
2.  Let <var>list</var> be a list of the CSS component values belonging to \<shape\>.
3.  Let each item in <var>list</var> be the result of invoking <a id="ref-for-serialize-a-css-component-value②"></a>[serialize a CSS component value](#serialize-a-css-component-value) of that item.
4.  Append the result of invoking <a id="ref-for-serialize-a-comma-separated-list⑤"></a>[serialize a comma-separated list](#serialize-a-comma-separated-list) on <var>list</var> to <var>s</var>.
5.  Append "<code>)</code>" (U+0029) to <var>s</var>.
6.  Return <var>s</var>.

<strong><a id="ref-for-string-value"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value)</strong>

<strong><a id="ref-for-font-family-name-value"></a>[\<font-family-name\>](https://drafts.csswg.org/css-fonts-4/#font-family-name-value)</strong>

The string <a id="ref-for-serialize-a-string⑥"></a>[serialized as a string](#serialize-a-string).

<strong><a id="ref-for-time-value"></a>[\<time\>](https://drafts.csswg.org/css-values-4/#time-value)</strong>

The time in seconds serialized as per \<number\> followed by the literal string "<code>s</code>".

<strong><a id="ref-for-url-value"></a>[\<url\>](https://drafts.csswg.org/css-values-4/#url-value)</strong>

The <a id="ref-for-absolute-url-string①"></a>[absolute-URL string](https://url.spec.whatwg.org/#absolute-url-string) <a id="ref-for-serialize-a-url③"></a>[serialized as URL](#serialize-a-url).

<a id="issue-c9f9e4e6"></a>

<strong>Issue:</strong>

[](#issue-c9f9e4e6) This should differentiate declared and computed <a id="ref-for-url-value①"></a>[\<url\>](https://drafts.csswg.org/css-values-4/#url-value) values, see [\#3195](https://github.com/w3c/csswg-drafts/issues/3195).

\<absolute-size\>, \<border-width\>, \<border-style\>, \<bottom\>, \<generic-font-family\>, \<generic-voice\>, \<left\>, \<margin-width\>, \<padding-width\>, \<relative-size\>, \<right\>, and \<top\>, are considered macros by this specification. They all represent instances of components outlined above.

<a id="issue-08e9ac89"></a>

<strong>Issue:</strong>

[](#issue-08e9ac89) One idea is that we can remove this section somewhere in the CSS3/CSS4 timeline by moving the above definitions to the drafts that define the CSS components.

Tests

- [serialize-custom-props.html](https://wpt.fyi/results/css/cssom/serialize-custom-props.html) [(live test)](http://wpt.live/css/cssom/serialize-custom-props.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/serialize-custom-props.html)

##### <a id="serializing-css-values-examples"></a>6.7.2.1. Examples[](#serializing-css-values-examples)

Here are some examples of before and after results on declared values. The before column could be what the author wrote in a style sheet, while the after column shows what querying the DOM would return.

<a id="example-9eae7eae"></a>

<strong>Example:</strong>

[](#example-9eae7eae)

| Before                                                      | After                                                                   |
|-------------------------------------------------------------|-------------------------------------------------------------------------|
| <code>background&#58; none</code>                           | <code>background&#58; rgba(0, 0, 0, 0)</code>                           |
| <code>outline&#58; none</code>                              | <code>outline&#58; invert</code>                                        |
| <code>border&#58; none</code>                               | <code>border&#58; medium</code>                                         |
| <code>list-style&#58; none</code>                           | <code>list-style&#58; disc</code>                                       |
| <code>margin&#58; 0 1px 1px 1px</code>                      | <code>margin&#58; 0px 1px 1px</code>                                    |
| <code>azimuth&#58; behind left</code>                       | <code>azimuth&#58; 220deg</code>                                        |
| <code>font-family&#58; a, 'b&#34;', serif</code>            | <code>font-family&#58; &#34;a&#34;, &#34;b&#92;&#34;&#34;, serif</code> |
| <code>content&#58; url('h)i') '&#92;&#91;&#92;&#93;'</code> | <code>content&#58; url(&#34;h)i&#34;) &#34;&#91;&#93;&#34;</code>       |
| <code>azimuth&#58; leftwards</code>                         | <code>azimuth&#58; leftwards</code>                                     |
| <code>color&#58; rgb(18, 52, 86)</code>                     | <code>color&#58; &#35;123456</code>                                     |
| <code>color&#58; rgba(000001, 0, 0, 1)</code>               | <code>color&#58; &#35;000000</code>                                     |

<a id="issue-3a42ec46"></a>

<strong>Issue:</strong>

[](#issue-3a42ec46) Some of these need to be updated per the new rules.

## <a id="dom-access-to-css-declaration-blocks"></a>7. DOM Access to CSS Declaration Blocks[](#dom-access-to-css-declaration-blocks)

### <a id="the-elementcssinlinestyle-mixin"></a>7.1. <a id="the-elementcssinlinestyle-interface"></a> The <code><a id="ref-for-elementcssinlinestyle"></a>[ElementCSSInlineStyle](#elementcssinlinestyle)</code> Mixin[](#the-elementcssinlinestyle-mixin)

The <code>ElementCSSInlineStyle</code> mixin provides access to inline style properties of an element.

<a id="elementcssinlinestyle"></a><a id="ref-for-SameObject①⓪"></a><a id="ref-for-PutForwards⑤"></a><a id="ref-for-cssstyleproperties⑤"></a><a id="ref-for-dom-elementcssinlinestyle-style"></a>

``` text
interface mixin ElementCSSInlineStyle {
  [SameObject, PutForwards=cssText] readonly attribute CSSStyleProperties style;
};
```

The <a id="dom-elementcssinlinestyle-style"></a><strong><code>style</code></strong> attribute must return a <code><a id="ref-for-cssstyleproperties⑥"></a>[CSSStyleProperties](#cssstyleproperties)</code> object whose <a id="ref-for-cssstyledeclaration-readonly-flag⑥"></a>[readonly flag](#cssstyledeclaration-readonly-flag) is unset, whose <a id="ref-for-cssstyledeclaration-parent-css-rule④"></a>[parent CSS rule](#cssstyledeclaration-parent-css-rule) is null, and whose <a id="ref-for-cssstyledeclaration-owner-node⑦"></a>[owner node](#cssstyledeclaration-owner-node) is <a id="ref-for-this③"></a>[this](https://webidl.spec.whatwg.org/#this).

If the user agent supports HTML, the following IDL applies: [\[HTML\]](#biblio-html)

<a id="ref-for-htmlelement"></a><a id="ref-for-elementcssinlinestyle①"></a>

``` text
HTMLElement includes ElementCSSInlineStyle;
```

If the user agent supports SVG, the following IDL applies: [\[SVG11\]](#biblio-svg11)

<a id="ref-for-InterfaceSVGElement"></a><a id="ref-for-elementcssinlinestyle②"></a>

``` text
SVGElement includes ElementCSSInlineStyle;
```

If the user agent supports MathML, the following IDL applies: [\[MathML-Core\]](#biblio-mathml-core)

<a id="ref-for-dom-mathmlelement"></a><a id="ref-for-elementcssinlinestyle③"></a>

``` text
MathMLElement includes ElementCSSInlineStyle;
```

Tests

- [css-style-attribute-modifications.html](https://wpt.fyi/results/css/cssom/css-style-attribute-modifications.html) [(live test)](http://wpt.live/css/cssom/css-style-attribute-modifications.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/css-style-attribute-modifications.html)
- [inline-style-001.html](https://wpt.fyi/results/css/cssom/inline-style-001.html) [(live test)](http://wpt.live/css/cssom/inline-style-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/inline-style-001.html)

### <a id="extensions-to-the-window-interface"></a>7.2. Extensions to the <code><a id="ref-for-window"></a>[Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window)</code> Interface[](#extensions-to-the-window-interface)

<a id="ref-for-window①"></a><a id="ref-for-NewObject"></a><a id="ref-for-cssstyleproperties⑦"></a><a id="ref-for-dom-window-getcomputedstyle"></a><a id="ref-for-element③"></a><a id="dom-window-getcomputedstyle-elt-pseudoelt-elt"></a><a id="ref-for-cssomstring④④"></a><a id="dom-window-getcomputedstyle-elt-pseudoelt-pseudoelt"></a>

``` text
partial interface Window {
  [NewObject] CSSStyleProperties getComputedStyle(Element elt, optional CSSOMString? pseudoElt);
};
```

The <a id="dom-window-getcomputedstyle"></a><strong><code>getComputedStyle(<var>elt</var>, <var>pseudoElt</var>)</code></strong> method must run these steps:

1.  Let <var>doc</var> be <var>elt</var>’s <a id="ref-for-concept-node-document③"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document).

2.  Let <var>obj</var> be <var>elt</var>.

3.  If <var>pseudoElt</var> is provided, is not the empty string, and starts with a colon, then:

    1.  <a id="ref-for-css-parse-something-according-to-a-css-grammar①"></a>[Parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>pseudoElt</var> as a <a id="ref-for-typedef-pseudo-element-selector"></a>[\<pseudo-element-selector\>](https://drafts.csswg.org/selectors-4/#typedef-pseudo-element-selector), and let <var>type</var> be the result.
    2.  If <var>type</var> is failure, or is a <a id="ref-for-selectordef-slotted"></a>[::slotted()](https://drafts.csswg.org/css-shadow-1/#selectordef-slotted) or <a id="ref-for-selectordef-part"></a>[::part()](https://drafts.csswg.org/css-shadow-1/#selectordef-part) pseudo-element, let <var>obj</var> be null.
    3.  Otherwise let <var>obj</var> be the given pseudo-element of <var>elt</var>.

    Note: CSS2 pseudo-elements should match both the double and single-colon versions. That is, both <code>&#58;before</code> and <code>&#58;&#58;before</code> should match above.

4.  Let <var>decls</var> be an empty list of <a id="ref-for-css-declaration②⑥"></a>[CSS declarations](#css-declaration).

5.  If <var>obj</var> is not null, and <var>elt</var> is <a id="ref-for-connected"></a>[connected](https://dom.spec.whatwg.org/#connected), part of the <a id="ref-for-flat-tree"></a>[flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree), and its <a id="ref-for-concept-shadow-including-root"></a>[shadow-including root](https://dom.spec.whatwg.org/#concept-shadow-including-root) has a <a id="ref-for-browsing-context"></a>[browsing context](https://html.spec.whatwg.org/multipage/browsers.html#browsing-context) which either doesn’t have a <u>browsing context container</u>, or whose <u>browsing context container</u> is <a id="ref-for-being-rendered"></a>[being rendered](https://html.spec.whatwg.org/multipage/rendering.html#being-rendered), set <var>decls</var> to a list of all longhand properties that are <a id="ref-for-supported-css-property⑤"></a>[supported CSS properties](#supported-css-property), in lexicographical order, with the value being the <a id="ref-for-resolved-value"></a>[resolved value](#resolved-value) computed for <var>obj</var> using the style rules associated with <var>doc</var>. Additionally, append to <var>decls</var> all the <a id="ref-for-custom-property⑤"></a>[custom properties](https://drafts.csswg.org/css-variables-1/#custom-property) whose <a id="ref-for-computed-value"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) for <var>obj</var> is not the <a id="ref-for-guaranteed-invalid-value"></a>[guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

    <a id="issue-28428295"></a>

    <strong>Issue:</strong>

    [](#issue-28428295) There are UAs that handle shorthands, and all UAs handle shorthands that used to be longhands like <a id="ref-for-propdef-overflow"></a>[overflow](https://drafts.csswg.org/css-overflow-3/#propdef-overflow), see [\#2529](https://github.com/w3c/csswg-drafts/issues/2529).

    <a id="issue-0b704bc1"></a>

    <strong>Issue:</strong>

    [](#issue-0b704bc1) Order of custom properties is currently undefined, see [\#4947](https://github.com/w3c/csswg-drafts/issues/4947).

6.  Return a live <code><a id="ref-for-cssstyleproperties⑧"></a>[CSSStyleProperties](#cssstyleproperties)</code> object with the following properties:
    <strong><a id="ref-for-cssstyledeclaration-computed-flag⑦"></a>[computed flag](#cssstyledeclaration-computed-flag)</strong>

    Set.
    <strong><a id="ref-for-cssstyledeclaration-readonly-flag⑦"></a>[readonly flag](#cssstyledeclaration-readonly-flag)</strong>

    Set.
    <strong><a id="ref-for-cssstyledeclaration-declarations①⑨"></a>[declarations](#cssstyledeclaration-declarations)</strong>

    <var>decls</var>.
    <strong><a id="ref-for-cssstyledeclaration-parent-css-rule⑤"></a>[parent CSS rule](#cssstyledeclaration-parent-css-rule)</strong>

    Null.
    <strong><a id="ref-for-cssstyledeclaration-owner-node⑧"></a>[owner node](#cssstyledeclaration-owner-node)</strong>

    <var>obj</var>.

The

<strong>Warning:</strong>

<code><a id="ref-for-dom-window-getcomputedstyle①"></a>[getComputedStyle()](#dom-window-getcomputedstyle)</code> method exposes information from <a id="ref-for-css-style-sheet④⑧"></a>[CSS style sheets](#css-style-sheet) with the <a id="ref-for-concept-css-style-sheet-origin-clean-flag⑦"></a>[origin-clean flag](#concept-css-style-sheet-origin-clean-flag) unset.

Tests

- [getComputedStyle-animations-replaced-into-ib-split.html](https://wpt.fyi/results/css/cssom/getComputedStyle-animations-replaced-into-ib-split.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-animations-replaced-into-ib-split.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-animations-replaced-into-ib-split.html)
- [getComputedStyle-detached-subtree.html](https://wpt.fyi/results/css/cssom/getComputedStyle-detached-subtree.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-detached-subtree.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-detached-subtree.html)
- [getComputedStyle-display-none-001.html](https://wpt.fyi/results/css/cssom/getComputedStyle-display-none-001.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-display-none-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-display-none-001.html)
- [getComputedStyle-display-none-002.html](https://wpt.fyi/results/css/cssom/getComputedStyle-display-none-002.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-display-none-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-display-none-002.html)
- [getComputedStyle-display-none-003.html](https://wpt.fyi/results/css/cssom/getComputedStyle-display-none-003.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-display-none-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-display-none-003.html)
- [getComputedStyle-dynamic-subdoc.html](https://wpt.fyi/results/css/cssom/getComputedStyle-dynamic-subdoc.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-dynamic-subdoc.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-dynamic-subdoc.html)
- [getComputedStyle-getter-v-properties.tentative.html](https://wpt.fyi/results/css/cssom/getComputedStyle-getter-v-properties.tentative.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-getter-v-properties.tentative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-getter-v-properties.tentative.html)
- [getComputedStyle-insets-absolute-crash.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-absolute-crash.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-absolute-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-absolute-crash.html)
- [getComputedStyle-insets-absolute-logical-crash.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-absolute-logical-crash.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-absolute-logical-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-absolute-logical-crash.html)
- [getComputedStyle-insets-absolute-roundtrip.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-absolute-roundtrip.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-absolute-roundtrip.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-absolute-roundtrip.html)
- [getComputedStyle-insets-absolute.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-absolute.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-absolute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-absolute.html)
- [getComputedStyle-insets-fixed.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-fixed.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-fixed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-fixed.html)
- [getComputedStyle-insets-grid.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-grid.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-grid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-grid.html)
- [getComputedStyle-insets-multicol-absolute-crash.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-multicol-absolute-crash.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-multicol-absolute-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-multicol-absolute-crash.html)
- [getComputedStyle-insets-nobox.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-nobox.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-nobox.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-nobox.html)
- [getComputedStyle-insets-relative.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-relative.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-relative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-relative.html)
- [getComputedStyle-insets-relpos-inline.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-relpos-inline.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-relpos-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-relpos-inline.html)
- [getComputedStyle-insets-static.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-static.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-static.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-static.html)
- [getComputedStyle-insets-sticky-container-for-abspos.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-sticky-container-for-abspos.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-sticky-container-for-abspos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-sticky-container-for-abspos.html)
- [getComputedStyle-insets-sticky.html](https://wpt.fyi/results/css/cssom/getComputedStyle-insets-sticky.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-insets-sticky.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-insets-sticky.html)
- [getComputedStyle-layout-dependent-removed-ib-sibling.html](https://wpt.fyi/results/css/cssom/getComputedStyle-layout-dependent-removed-ib-sibling.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-layout-dependent-removed-ib-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-layout-dependent-removed-ib-sibling.html)
- [getComputedStyle-layout-dependent-replaced-into-ib-split.html](https://wpt.fyi/results/css/cssom/getComputedStyle-layout-dependent-replaced-into-ib-split.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-layout-dependent-replaced-into-ib-split.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-layout-dependent-replaced-into-ib-split.html)
- [getComputedStyle-line-height.html](https://wpt.fyi/results/css/cssom/getComputedStyle-line-height.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-line-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-line-height.html)
- [getComputedStyle-logical-enumeration.html](https://wpt.fyi/results/css/cssom/getComputedStyle-logical-enumeration.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-logical-enumeration.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-logical-enumeration.html)
- [getComputedStyle-margins-roundtrip.html](https://wpt.fyi/results/css/cssom/getComputedStyle-margins-roundtrip.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-margins-roundtrip.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-margins-roundtrip.html)
- [getComputedStyle-property-order.html](https://wpt.fyi/results/css/cssom/getComputedStyle-property-order.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-property-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-property-order.html)
- [getComputedStyle-pseudo-checkmark.html](https://wpt.fyi/results/css/cssom/getComputedStyle-pseudo-checkmark.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-pseudo-checkmark.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-pseudo-checkmark.html)
- [getComputedStyle-pseudo-picker-icon.html](https://wpt.fyi/results/css/cssom/getComputedStyle-pseudo-picker-icon.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-pseudo-picker-icon.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-pseudo-picker-icon.html)
- [getComputedStyle-pseudo-with-argument.html](https://wpt.fyi/results/css/cssom/getComputedStyle-pseudo-with-argument.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-pseudo-with-argument.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-pseudo-with-argument.html)
- [getComputedStyle-pseudo.html](https://wpt.fyi/results/css/cssom/getComputedStyle-pseudo.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-pseudo.html)
- [getComputedStyle-resolved-colors.html](https://wpt.fyi/results/css/cssom/getComputedStyle-resolved-colors.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-resolved-colors.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-resolved-colors.html)
- [getComputedStyle-resolved-min-max-clamping.html](https://wpt.fyi/results/css/cssom/getComputedStyle-resolved-min-max-clamping.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-resolved-min-max-clamping.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-resolved-min-max-clamping.html)
- [getComputedStyle-resolved-min-size-auto.html](https://wpt.fyi/results/css/cssom/getComputedStyle-resolved-min-size-auto.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-resolved-min-size-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-resolved-min-size-auto.html)
- [getComputedStyle-special-chars-crash.html](https://wpt.fyi/results/css/cssom/getComputedStyle-special-chars-crash.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-special-chars-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-special-chars-crash.html)
- [getComputedStyle-sticky-pos-percent.html](https://wpt.fyi/results/css/cssom/getComputedStyle-sticky-pos-percent.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-sticky-pos-percent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-sticky-pos-percent.html)
- [getComputedStyle-width-scroll.tentative.html](https://wpt.fyi/results/css/cssom/getComputedStyle-width-scroll.tentative.html) [(live test)](http://wpt.live/css/cssom/getComputedStyle-width-scroll.tentative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/getComputedStyle-width-scroll.tentative.html)

## <a id="utility-apis"></a>8. Utility APIs[](#utility-apis)

### <a id="the-css.escape()-method"></a>8.1. The <code>CSS.escape()</code> Method[](#the-css.escape()-method)

The <code>CSS</code> namespace holds useful CSS-related functions that do not belong elsewhere.

<a id="ref-for-Exposed①⑤"></a><a id="namespacedef-css"></a><a id="ref-for-cssomstring④⑤"></a><a id="ref-for-dom-css-escape"></a><a id="ref-for-cssomstring④⑥"></a><a id="dom-css-escape-ident-ident"></a>

``` text
[Exposed=Window]
namespace CSS {
  CSSOMString escape(CSSOMString ident);
};
```

Tests

- [CSS-namespace-object-class-string.html](https://wpt.fyi/results/css/cssom/CSS-namespace-object-class-string.html) [(live test)](http://wpt.live/css/cssom/CSS-namespace-object-class-string.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/CSS-namespace-object-class-string.html)

<a id="issue-24739c22"></a>

<strong>Issue:</strong>

[](#issue-24739c22) This was previously specified as an IDL interface that only held static methods. Switching to an IDL namespace is \*nearly\* identical, so it’s expected that there won’t be any compat concerns. If any are discovered, please report so we can consider reverting this change.

The <a id="dom-css-escape"></a><strong><code>escape(<var>ident</var>)</code></strong> operation must return the result of invoking <a id="ref-for-serialize-an-identifier①①"></a>[serialize an identifier](#serialize-an-identifier) of <var>ident</var>.

<a id="example-c8dd1300"></a>

<strong>Example:</strong>

[](#example-c8dd1300) For example, to serialize a string for use as part of a selector, the <code><a id="ref-for-dom-css-escape①"></a>[escape()](#dom-css-escape)</code> method can be used:

``` text
var element = document.querySelector('#' + CSS.escape(id) + ' > img');
```

<a id="example-6c87326f"></a>

<strong>Example:</strong>

[](#example-6c87326f) The <code><a id="ref-for-dom-css-escape②"></a>[escape()](#dom-css-escape)</code> method can also be used for escaping strings, although it escapes characters that don’t strictly need to be escaped:

``` text
var element = document.querySelector('a[href="#' + CSS.escape(fragment) + '"]');
```

Specifications that define operations on the <code><a id="ref-for-namespacedef-css"></a>[CSS](#namespacedef-css)</code> namespace and want to store some state should store the state on the <a id="ref-for-current-global-object②"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object)’s <a id="ref-for-concept-document-window②"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

Tests

- [escape.html](https://wpt.fyi/results/css/cssom/escape.html) [(live test)](http://wpt.live/css/cssom/escape.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/escape.html)

## <a id="resolved-values"></a>9. Resolved Values[](#resolved-values)

<code><a id="ref-for-dom-window-getcomputedstyle②"></a>[getComputedStyle()](#dom-window-getcomputedstyle)</code> was historically defined to return the "computed value" of an element or pseudo-element. However, the concept of "computed value" changed between revisions of CSS while the implementation of <code><a id="ref-for-dom-window-getcomputedstyle③"></a>[getComputedStyle()](#dom-window-getcomputedstyle)</code> had to remain the same for compatibility with deployed scripts. To address this issue this specification introduces the concept of a <a id="resolved-value"></a><strong>resolved value</strong>.

The <a id="ref-for-resolved-value①"></a>[resolved value](#resolved-value) for a given longhand property can be determined as follows:

<strong><a id="ref-for-propdef-background-color"></a>[background-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color)</strong>

<strong><a id="ref-for-propdef-border-block-end-color"></a>[border-block-end-color](https://drafts.csswg.org/css-borders-4/#propdef-border-block-end-color)</strong>

<strong><a id="ref-for-propdef-border-block-start-color"></a>[border-block-start-color](https://drafts.csswg.org/css-borders-4/#propdef-border-block-start-color)</strong>

<strong><a id="ref-for-propdef-border-bottom-color"></a>[border-bottom-color](https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-color)</strong>

<strong><a id="ref-for-propdef-border-inline-end-color"></a>[border-inline-end-color](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end-color)</strong>

<strong><a id="ref-for-propdef-border-inline-start-color"></a>[border-inline-start-color](https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start-color)</strong>

<strong><a id="ref-for-propdef-border-left-color"></a>[border-left-color](https://drafts.csswg.org/css-borders-4/#propdef-border-left-color)</strong>

<strong><a id="ref-for-propdef-border-right-color"></a>[border-right-color](https://drafts.csswg.org/css-borders-4/#propdef-border-right-color)</strong>

<strong><a id="ref-for-propdef-border-top-color"></a>[border-top-color](https://drafts.csswg.org/css-borders-4/#propdef-border-top-color)</strong>

<strong><a id="ref-for-propdef-box-shadow"></a>[box-shadow](https://drafts.csswg.org/css-borders-4/#propdef-box-shadow)</strong>

<strong><a id="ref-for-propdef-caret-color"></a>[caret-color](https://drafts.csswg.org/css-ui-4/#propdef-caret-color)</strong>

<strong><a id="ref-for-propdef-color"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color)</strong>

<strong><a id="ref-for-propdef-outline-color"></a>[outline-color](https://drafts.csswg.org/css-ui-4/#propdef-outline-color)</strong>

<strong>A <a id="resolved-value-special-case-property-like-color"></a><strong>resolved value special case property like <a id="ref-for-propdef-color①"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color)</strong> defined in another specification</strong>

The <a id="ref-for-resolved-value②"></a>[resolved value](#resolved-value) is the <a id="ref-for-used-value"></a>[used value](https://drafts.csswg.org/css-cascade-5/#used-value).

<strong><a id="ref-for-propdef-line-height"></a>[line-height](https://drafts.csswg.org/css2/#propdef-line-height)</strong>

The <a id="ref-for-resolved-value③"></a>[resolved value](#resolved-value) is <a id="ref-for-valdef-line-height-normal"></a>[normal](https://drafts.csswg.org/css-inline-3/#valdef-line-height-normal) if the <a id="ref-for-computed-value①"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) is <a id="ref-for-valdef-line-height-normal①"></a>normal, or the <a id="ref-for-used-value①"></a>[used value](https://drafts.csswg.org/css-cascade-5/#used-value) otherwise.

<strong><a id="ref-for-propdef-block-size"></a>[block-size](https://drafts.csswg.org/css-logical-1/#propdef-block-size)</strong>

<strong><a id="ref-for-propdef-height"></a>[height](https://drafts.csswg.org/css-sizing-3/#propdef-height)</strong>

<strong><a id="ref-for-propdef-inline-size"></a>[inline-size](https://drafts.csswg.org/css-logical-1/#propdef-inline-size)</strong>

<strong><a id="ref-for-propdef-margin-block-end"></a>[margin-block-end](https://drafts.csswg.org/css-logical-1/#propdef-margin-block-end)</strong>

<strong><a id="ref-for-propdef-margin-block-start"></a>[margin-block-start](https://drafts.csswg.org/css-logical-1/#propdef-margin-block-start)</strong>

<strong><a id="ref-for-propdef-margin-bottom"></a>[margin-bottom](https://drafts.csswg.org/css-box-4/#propdef-margin-bottom)</strong>

<strong><a id="ref-for-propdef-margin-inline-end"></a>[margin-inline-end](https://drafts.csswg.org/css-logical-1/#propdef-margin-inline-end)</strong>

<strong><a id="ref-for-propdef-margin-inline-start"></a>[margin-inline-start](https://drafts.csswg.org/css-logical-1/#propdef-margin-inline-start)</strong>

<strong><a id="ref-for-propdef-margin-left"></a>[margin-left](https://drafts.csswg.org/css-box-4/#propdef-margin-left)</strong>

<strong><a id="ref-for-propdef-margin-right"></a>[margin-right](https://drafts.csswg.org/css-box-4/#propdef-margin-right)</strong>

<strong><a id="ref-for-propdef-margin-top"></a>[margin-top](https://drafts.csswg.org/css-box-4/#propdef-margin-top)</strong>

<strong><a id="ref-for-propdef-padding-block-end"></a>[padding-block-end](https://drafts.csswg.org/css-logical-1/#propdef-padding-block-end)</strong>

<strong><a id="ref-for-propdef-padding-block-start"></a>[padding-block-start](https://drafts.csswg.org/css-logical-1/#propdef-padding-block-start)</strong>

<strong><a id="ref-for-propdef-padding-bottom"></a>[padding-bottom](https://drafts.csswg.org/css-box-4/#propdef-padding-bottom)</strong>

<strong><a id="ref-for-propdef-padding-inline-end"></a>[padding-inline-end](https://drafts.csswg.org/css-logical-1/#propdef-padding-inline-end)</strong>

<strong><a id="ref-for-propdef-padding-inline-start"></a>[padding-inline-start](https://drafts.csswg.org/css-logical-1/#propdef-padding-inline-start)</strong>

<strong><a id="ref-for-propdef-padding-left"></a>[padding-left](https://drafts.csswg.org/css-box-4/#propdef-padding-left)</strong>

<strong><a id="ref-for-propdef-padding-right"></a>[padding-right](https://drafts.csswg.org/css-box-4/#propdef-padding-right)</strong>

<strong><a id="ref-for-propdef-padding-top"></a>[padding-top](https://drafts.csswg.org/css-box-4/#propdef-padding-top)</strong>

<strong><a id="ref-for-propdef-width"></a>[width](https://drafts.csswg.org/css-sizing-3/#propdef-width)</strong>

<strong>A <a id="resolved-value-special-case-property-like-height"></a><strong>resolved value special case property like <a id="ref-for-propdef-height①"></a>[height](https://drafts.csswg.org/css-sizing-3/#propdef-height)</strong> defined in another specification</strong>

If the property applies to the element or pseudo-element and the <a id="ref-for-resolved-value④"></a>[resolved value](#resolved-value) of the <a id="ref-for-propdef-display"></a>[display](https://drafts.csswg.org/css-display-4/#propdef-display) property is not <a id="ref-for-valdef-display-none"></a>[none](https://drafts.csswg.org/css-display-4/#valdef-display-none) or <a id="ref-for-valdef-display-contents"></a>[contents](https://drafts.csswg.org/css-display-4/#valdef-display-contents), then the <a id="ref-for-resolved-value⑤"></a>resolved value is the <a id="ref-for-used-value②"></a>[used value](https://drafts.csswg.org/css-cascade-5/#used-value). Otherwise the <a id="ref-for-resolved-value⑥"></a>resolved value is the <a id="ref-for-computed-value②"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value).

<strong><a id="ref-for-propdef-bottom"></a>[bottom](https://drafts.csswg.org/css-position-3/#propdef-bottom)</strong>

<strong><a id="ref-for-propdef-left"></a>[left](https://drafts.csswg.org/css-position-3/#propdef-left)</strong>

<strong><a id="ref-for-propdef-inset-block-end"></a>[inset-block-end](https://drafts.csswg.org/css-position-3/#propdef-inset-block-end)</strong>

<strong><a id="ref-for-propdef-inset-block-start"></a>[inset-block-start](https://drafts.csswg.org/css-position-3/#propdef-inset-block-start)</strong>

<strong><a id="ref-for-propdef-inset-inline-end"></a>[inset-inline-end](https://drafts.csswg.org/css-position-3/#propdef-inset-inline-end)</strong>

<strong><a id="ref-for-propdef-inset-inline-start"></a>[inset-inline-start](https://drafts.csswg.org/css-position-3/#propdef-inset-inline-start)</strong>

<strong><a id="ref-for-propdef-right"></a>[right](https://drafts.csswg.org/css-position-3/#propdef-right)</strong>

<strong><a id="ref-for-propdef-top"></a>[top](https://drafts.csswg.org/css-position-3/#propdef-top)</strong>

<strong>A <a id="resolved-value-special-case-property-like-top"></a><strong>resolved value special case property like <a id="ref-for-propdef-top①"></a>[top](https://drafts.csswg.org/css-position-3/#propdef-top)</strong> defined in another specification</strong>

If the property applies to a positioned element and the <a id="ref-for-resolved-value⑦"></a>[resolved value](#resolved-value) of the <a id="ref-for-propdef-display①"></a>[display](https://drafts.csswg.org/css-display-4/#propdef-display) property is not <a id="ref-for-valdef-display-none①"></a>[none](https://drafts.csswg.org/css-display-4/#valdef-display-none) or <a id="ref-for-valdef-display-contents①"></a>[contents](https://drafts.csswg.org/css-display-4/#valdef-display-contents), and the property is not over-constrained, then the <a id="ref-for-resolved-value⑧"></a>resolved value is the <a id="ref-for-used-value③"></a>[used value](https://drafts.csswg.org/css-cascade-5/#used-value). Otherwise the <a id="ref-for-resolved-value⑨"></a>resolved value is the <a id="ref-for-computed-value③"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value).

<strong>A <a id="resolved-value-special-case-property"></a><strong>resolved value special case property</strong> defined in another specification</strong>

As defined in the relevant specification.

<strong>Any other property</strong>

The <a id="ref-for-resolved-value①⓪"></a>[resolved value](#resolved-value) is the <a id="ref-for-computed-value④"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value).

Tests

- [computed-style-001.html](https://wpt.fyi/results/css/cssom/computed-style-001.html) [(live test)](http://wpt.live/css/cssom/computed-style-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/computed-style-001.html)
- [computed-style-002.html](https://wpt.fyi/results/css/cssom/computed-style-002.html) [(live test)](http://wpt.live/css/cssom/computed-style-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/computed-style-002.html)
- [computed-style-003.html](https://wpt.fyi/results/css/cssom/computed-style-003.html) [(live test)](http://wpt.live/css/cssom/computed-style-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/computed-style-003.html)
- [computed-style-004.html](https://wpt.fyi/results/css/cssom/computed-style-004.html) [(live test)](http://wpt.live/css/cssom/computed-style-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/computed-style-004.html)
- [computed-style-005.html](https://wpt.fyi/results/css/cssom/computed-style-005.html) [(live test)](http://wpt.live/css/cssom/computed-style-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/computed-style-005.html)
- [resolved-border-width.html](https://wpt.fyi/results/css/cssom/resolved-border-width.html) [(live test)](http://wpt.live/css/cssom/resolved-border-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/resolved-border-width.html)

## <a id="iana-considerations"></a>10. IANA Considerations[](#iana-considerations)

### <a id="default-style"></a>10.1. Default-Style[](#default-style)

This section describes a header field for registration in the Permanent Message Header Field Registry.

<strong>Header field name</strong>

<a id="http-default-style"></a><strong>Default-Style</strong>

<strong>Applicable protocol</strong>

http

<strong>Status</strong>

standard

<strong>Author/Change controller</strong>

W3C

<strong>Specification document(s)</strong>

This document is the relevant specification.

<strong>Related information</strong>

None.

## <a id="change-history"></a>11. Change History[](#change-history)

This section documents some of the changes between publications of this specification. This section is not exhaustive. Bug fixes and editorial changes are generally not listed.

### <a id="changes-from-17-march-2016"></a>11.1. Changes From 17 March 2016[](#changes-from-17-march-2016)

- Serialization of <a id="ref-for-resolution-value①"></a>[\<resolution\>](https://drafts.csswg.org/css-values-4/#resolution-value) is changed.

- <code>&#91;CEReactions&#93;</code> IDL extended attributes are added.

- Resolved value for logical properties are added.

- <code>getComputedStyle</code> for <a id="ref-for-propdef-display②"></a>[display](https://drafts.csswg.org/css-display-4/#propdef-display): <a id="ref-for-valdef-display-contents②"></a>[contents](https://drafts.csswg.org/css-display-4/#valdef-display-contents) is changed.

- <code>MediaList.item</code> now returns serialization.

- <code>MediaList.item</code> does not serialize shorthand if importance differs.

- Other specifications are allowed to specify resolved value.

- <code>index</code> argument in <code>insertRule</code> is now optional.

- <code>href</code> attribute of <code>Stylesheet</code> and <code>CSSImportRule</code> now uses <code>USVString</code>.

- <code>CSSOMString</code> is introduced.

- Serialization of <code>CSSMediaRule</code> and <code>CSSFontFaceRule</code> is added.

- <a id="ref-for-cssstyledeclaration-updating-flag③"></a>[Updating flag](#cssstyledeclaration-updating-flag) is added to CSS declaration block to avoid serialize-and-reparse on style attribute.

- Serialization of a declaration value is now properly defined.

- <code>getComputedStyle</code> now returns the style rules of the node’s document.

- A <code><a id="ref-for-exceptiondef-typeerror"></a>[TypeError](https://webidl.spec.whatwg.org/#exceptiondef-typeerror)</code> is thrown when the pseudo-element passed to <code>getComputedStyle</code> is unknown or <a id="ref-for-selectordef-slotted①"></a>[::slotted()](https://drafts.csswg.org/css-shadow-1/#selectordef-slotted).

- <code><a id="ref-for-namespacedef-css①"></a>[CSS](#namespacedef-css)</code> is switched from interface to namespace.

- <code>setPropertyValue</code> and <code>setPropertyPriority</code> are removed from <code><a id="ref-for-cssstyledeclaration③"></a>[CSSStyleDeclaration](#cssstyledeclaration)</code> due to lack of interest from implementations.

- The <code>styleSheets</code> IDL attribute is moved from <code><a id="ref-for-document④"></a>[Document](https://dom.spec.whatwg.org/#document)</code> to <code><a id="ref-for-documentorshadowroot⑧"></a>[DocumentOrShadowRoot](https://dom.spec.whatwg.org/#documentorshadowroot)</code>.

- LinkStyle.sheet now returns <code>CSSStyleSheet</code> instead of <code>StyleSheet</code>

- Deprecated CSSStyleSheet members are defined.

- The <code>CSSRule.type</code> attribute is deprecated.

- Serialization of <a id="ref-for-ratio-value①"></a>[\<ratio\>](https://drafts.csswg.org/css-values-4/#ratio-value) is added.

- <code>CSSStyleDeclaration.cssText</code> now returns the empty string for computed style.

- <a id="ref-for-custom-property⑥"></a>[Custom properties](https://drafts.csswg.org/css-variables-1/#custom-property) are included in <code>getComputedStyle</code>.

- MathML IDL is introduced.

- Serialization of <code><a id="ref-for-csskeyframesrule②"></a>[CSSKeyframesRule](https://drafts.csswg.org/css-animations-1/#csskeyframesrule)</code> and <code><a id="ref-for-csskeyframerule②"></a>[CSSKeyframeRule](https://drafts.csswg.org/css-animations-1/#csskeyframerule)</code> is added.

- Serialization of media query is changed.

- A shorthand is not serialized if there are longhands with other property group / mapping logic in between the longhands of that shorthand.

- <code><a id="ref-for-cssstylerule③"></a>[CSSStyleRule](#cssstylerule)</code> serialization is aware of nesting now.

- Constructable stylesheets is introduced.

### <a id="changes-from-5-december-2013"></a>11.2. Changes From 5 December 2013[](#changes-from-5-december-2013)

- API for alternative stylesheets is removed: <code>selectedStyleSheetSet</code>, <code>lastStyleSheetSet</code>, <code>preferredStyleSheetSet</code>, <code>styleSheetSets</code>, <code>enableStyleSheetsForSet()</code> on <code><a id="ref-for-document⑤"></a>[Document](https://dom.spec.whatwg.org/#document)</code>.

- The <code>pseudo()</code> method on <code><a id="ref-for-element④"></a>[Element](https://dom.spec.whatwg.org/#element)</code> and the <code>PseudoElement</code> interface is removed.

- The <code>cascadedStyle</code>, <code>defaultStyle</code>, <code>rawComputedStyle</code> and <code>usedStyle</code> IDL attributes on <code><a id="ref-for-element⑤"></a>[Element](https://dom.spec.whatwg.org/#element)</code> are removed.

- The <code><a id="ref-for-dom-cssrule-csstext②"></a>[cssText](#dom-cssrule-csstext)</code> IDL attribute’s setter on <code><a id="ref-for-cssrule①⑤"></a>[CSSRule](#cssrule)</code> is changed to do nothing.

- IDL attributes of the form <code>webkitFoo</code> (with lowercase <code>w</code>) on <code><a id="ref-for-cssstyledeclaration④"></a>[CSSStyleDeclaration](#cssstyledeclaration)</code> are added.

- <code><a id="ref-for-cssnamespacerule③"></a>[CSSNamespaceRule](#cssnamespacerule)</code> is changed back to readonly.

- Handling of <code>@charset</code> in <code><a id="ref-for-dom-cssstylesheet-insertrule③"></a>[insertRule()](#dom-cssstylesheet-insertrule)</code> is removed.

- <code>CSSCharsetRule</code> is removed again.

- Serialization of identifiers and strings is changed.

- Serialization of selectors now supports combinators "\>\>" and "\|\|" and the "i" flag in attribute selectors.

- Serialization of :lang() is changed.

- Serialization of <a id="ref-for-typedef-color②"></a>[\<color\>](https://drafts.csswg.org/css-color-5/#typedef-color) and <a id="ref-for-number-value①"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value) is changed.

- <code><a id="ref-for-dom-cssstyledeclaration-setproperty⑤"></a>[setProperty()](#dom-cssstyledeclaration-setproperty)</code> on <code><a id="ref-for-cssstyledeclaration⑤"></a>[CSSStyleDeclaration](#cssstyledeclaration)</code> is changed.

### <a id="changes-from-12-july-2011-to-5-december-2013"></a>11.3. Changes From 12 July 2011 To 5 December 2013[](#changes-from-12-july-2011-to-5-december-2013)

- Cross-origin stylesheets are not allowed to be read or changed.
- <code>CSSCharsetRule</code> is re-introduced.
- <code>CSSGroupingRule</code> and <code>CSSMarginRule</code> are introduced.
- <code>CSSNamespaceRule</code> is now mutable.
- <a id="ref-for-parse-a-css-declaration-block④"></a>[Parse](#parse-a-css-declaration-block) and <a id="ref-for-serialize-a-css-declaration-block④"></a>[serialize](#serialize-a-css-declaration-block) a CSS declaration block is now defined.
- Shorthands are now supported in <code><a id="ref-for-dom-cssstyledeclaration-setproperty⑥"></a>[setProperty()](#dom-cssstyledeclaration-setproperty)</code>, <code><a id="ref-for-dom-cssstyledeclaration-getpropertyvalue⑥"></a>[getPropertyValue()](#dom-cssstyledeclaration-getpropertyvalue)</code>, et al.
- <code>setPropertyValue</code> and <code>setPropertyPriority</code> are added to <code><a id="ref-for-cssstyledeclaration⑥"></a>[CSSStyleDeclaration](#cssstyledeclaration)</code>.
- The <code>style</code> and <code>media</code> attributes of various interfaces are annotated with the <code>&#91;PutForwards&#93;</code> WebIDL extended attribute.
- The <code>pseudo()</code> method on <code>Element</code> is introduced.
- The <code>PseudoElement</code> interface is introduced.
- The <code>cascadedStyle</code>, <code>rawComputedStyle</code> and <code>usedStyle</code> attributes on <code>Element</code> and <code>PseudoElement</code> are introduced.
- The <a id="ref-for-dom-css-escape③"></a>[CSS.escape()](#dom-css-escape) static method is introduced.

## <a id="sec"></a>12. Security Considerations[](#sec)

No new security considerations have been reported on this specification.

## <a id="priv"></a>13. Privacy Considerations[](#priv)

No new privacy considerations have been reported on this specification.

## <a id="acknowledgments"></a>14. Acknowledgments[](#acknowledgments)

The editors would like to thank Alexey Feldgendler, Benjamin Poulain, Björn Höhrmann, Boris Zbasky, Brian Kardell, Chris Dumez, Christian Krebs, Daniel Glazman, David Baron, Domenic Denicola, Dominique Hazael-Massieux, <em>fantasai</em>, Hallvord R. M. Steen, Ian Hickson, John Daggett, Lachlan Hunt, Mike Sherov, Myles C. Maxfield, Morten Stenshorne, Ms2ger, Nazım Can Altınova, Øyvind Stenhaug, Peter Sloetjes, Philip Jägenstedt, Philip Taylor, Richard Gibson, Robert O’Callahan, Simon Sapin, Sjoerd Visscher, Sylvain Galineau, Tarquin Wilton-Jones, Xidorn Quan, and Zack Weinberg for contributing to this specification.

Additional thanks to Ian Hickson for writing the initial version of the alternative style sheets API and canonicalization (now serialization) rules for CSS values.

Tests

- [cssstyledeclaration-csstext-setter.window.js](https://wpt.fyi/results/css/cssom/cssstyledeclaration-csstext-setter.window.js) [(live test)](http://wpt.live/css/cssom/cssstyledeclaration-csstext-setter.window.js) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom/cssstyledeclaration-csstext-setter.window.js)

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

A [CSS style sheet](http://www.w3.org/TR/CSS21/conform.html#style-sheet).

<strong>renderer</strong>

A [UA](http://www.w3.org/TR/CSS21/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

<strong>authoring tool</strong>

A [UA](http://www.w3.org/TR/CSS21/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="w3c-partial"></a> Partial implementations[](#w3c-partial)

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](http://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="w3c-conform-future-proofing"></a> Implementations of Unstable and Proprietary Features[](#w3c-conform-future-proofing)

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](http://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](http://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](http://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

### <a id="w3c-testing"></a> Non-experimental implementations[](#w3c-testing)

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at <http://www.w3.org/Style/CSS/Test/>. Questions should be directed to the [public-css-testsuite&#64;w3.org](http://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index[](#index)

### <a id="index-defined-here"></a>Terms defined by this specification[](#index-defined-here)

- [add a CSS style sheet](#add-a-css-style-sheet), in § 6.2
- [addRule()](#dom-cssstylesheet-addrule), in § 6.1.2.1
- [addRule(selector)](#dom-cssstylesheet-addrule), in § 6.1.2.1
- [addRule(selector, block, optionalIndex)](#dom-cssstylesheet-addrule), in § 6.1.2.1
- [addRule(selector, style)](#dom-cssstylesheet-addrule), in § 6.1.2.1
- [addRule(selector, style, index)](#dom-cssstylesheet-addrule), in § 6.1.2.1
- [adoptedStyleSheets](#dom-documentorshadowroot-adoptedstylesheets), in § 6.2.3
- [alternate flag](#concept-css-style-sheet-alternate-flag), in § 6.1
- [appendMedium(medium)](#dom-medialist-appendmedium), in § 4.4
- [associated CSS style sheet](#associated-css-style-sheet), in § 6.3.2
- [baseURL](#dom-cssstylesheetinit-baseurl), in § 6.1.2
- [bleed](#dom-csspagedescriptors-bleed), in § 6.4.7
- [camel-cased attribute](#dom-cssstyleproperties-camel-cased-attribute), in § 6.6.1
- [camel_cased_attribute](#dom-cssstyleproperties-camel_cased_attribute), in § 6.6.1
- [case-sensitive flag](#css-declaration-case-sensitive-flag), in § 6.5
- [change the preferred CSS style sheet set name](#change-the-preferred-css-style-sheet-set-name), in § 6.2
- [CHARSET_RULE](#dom-cssrule-charset_rule), in § 6.4.2
- [child CSS rules](#concept-css-rule-child-css-rules), in § 6.4
- [collection of media queries](#medialist-collection-of-media-queries), in § 4.4
- [compare media queries](#compare-media-queries), in § 4.3
- [computed flag](#cssstyledeclaration-computed-flag), in § 6.6
- [constructed flag](#concept-css-style-sheet-constructed-flag), in § 6.1
- [constructor()](#dom-cssstylesheet-cssstylesheet), in § 6.1
- [constructor document](#concept-css-style-sheet-constructor-document), in § 6.1
- [constructor(options)](#dom-cssstylesheet-cssstylesheet), in § 6.1
- [create a constructed CSSStyleSheet](#create-a-constructed-cssstylesheet), in § 6.1
- [create a CSS style sheet](#create-a-css-style-sheet), in § 6.2
- [create a MediaList object](#create-a-medialist-object), in § 4.4
- [CSS](#namespacedef-css), in § 8.1
- [CSS declaration](#css-declaration), in § 6.5
- [CSS declaration block](#css-declaration-block), in § 6.6
- [cssFloat](#dom-cssstyleproperties-cssfloat), in § 6.6.1
- [CSSGroupingRule](#cssgroupingrule), in § 6.4.5
- [CSSImportRule](#cssimportrule), in § 6.4.4
- [CSSMarginRule](#cssmarginrule), in § 6.4.8
- [CSSNamespaceRule](#cssnamespacerule), in § 6.4.9
- [CSSOMString](#cssomstring), in § 3
- [CSSPageDescriptors](#csspagedescriptors), in § 6.4.7
- [CSSPageRule](#csspagerule), in § 6.4.7
- [CSS property to IDL attribute](#css-property-to-idl-attribute), in § 6.6.1
- [CSS rule](#css-rule), in § 6.4
- [CSSRule](#cssrule), in § 6.4.2
- [CSSRuleList](#cssrulelist), in § 6.4.1
- [CSS rules](#concept-css-style-sheet-css-rules), in § 6.1
- cssRules
  - [attribute for CSSGroupingRule](#dom-cssgroupingrule-cssrules), in § 6.4.5
  - [attribute for CSSStyleSheet](#dom-cssstylesheet-cssrules), in § 6.1.2
- [CSSStyleDeclaration](#cssstyledeclaration), in § 6.6.1
- [CSSStyleProperties](#cssstyleproperties), in § 6.6.1
- [CSSStyleRule](#cssstylerule), in § 6.4.3
- [CSS style sheet](#css-style-sheet), in § 6.1
- [CSSStyleSheet](#cssstylesheet), in § 6.1.2
- [CSSStyleSheet()](#dom-cssstylesheet-cssstylesheet), in § 6.1
- [CSSStyleSheetInit](#dictdef-cssstylesheetinit), in § 6.1.2
- [CSSStyleSheet(options)](#dom-cssstylesheet-cssstylesheet), in § 6.1
- [CSS style sheet set](#css-style-sheet-set), in § 6.2
- [CSS style sheet set name](#css-style-sheet-set-name), in § 6.2
- cssText
  - [attribute for CSSRule](#dom-cssrule-csstext), in § 6.4.2
  - [attribute for CSSStyleDeclaration](#dom-cssstyledeclaration-csstext), in § 6.6.1
- [dashed attribute](#dom-cssstyleproperties-dashed-attribute), in § 6.6.1
- [dashed_attribute](#dom-cssstyleproperties-dashed_attribute), in § 6.6.1
- [declarations](#cssstyledeclaration-declarations), in § 6.6
- [deleteMedium(medium)](#dom-medialist-deletemedium), in § 4.4
- deleteRule(index)
  - [method for CSSGroupingRule](#dom-cssgroupingrule-deleterule), in § 6.4.5
  - [method for CSSStyleSheet](#dom-cssstylesheet-deleterule), in § 6.1.2
- disabled
  - [attribute for StyleSheet](#dom-stylesheet-disabled), in § 6.1.1
  - [dict-member for CSSStyleSheetInit](#dom-cssstylesheetinit-disabled), in § 6.1.2
- [disabled flag](#concept-css-style-sheet-disabled-flag), in § 6.1
- [disallow modification flag](#concept-css-style-sheet-disallow-modification-flag), in § 6.1
- [document or shadow root CSS style sheets](#documentorshadowroot-document-or-shadow-root-css-style-sheets), in § 6.2
- [ElementCSSInlineStyle](#elementcssinlinestyle), in § 7.1
- [enable a CSS style sheet set](#enable-a-css-style-sheet-set), in § 6.2
- [enabled CSS style sheet set](#enabled-css-style-sheet-set), in § 6.2
- [escape a character](#escape-a-character), in § 2.1
- [escape a character as code point](#escape-a-character-as-code-point), in § 2.1
- [escaped as code point](#escape-a-character-as-code-point), in § 2.1
- [escape(ident)](#dom-css-escape), in § 8.1
- [fetch a CSS style sheet](#fetch-a-css-style-sheet), in § 6.3.1
- [final CSS style sheets](#documentorshadowroot-final-css-style-sheets), in § 6.2
- [FONT_FACE_RULE](#dom-cssrule-font_face_rule), in § 6.4.2
- [getComputedStyle(elt)](#dom-window-getcomputedstyle), in § 7.2
- [getComputedStyle(elt, pseudoElt)](#dom-window-getcomputedstyle), in § 7.2
- [getPropertyPriority(property)](#dom-cssstyledeclaration-getpropertypriority), in § 6.6.1
- [getPropertyValue(property)](#dom-cssstyledeclaration-getpropertyvalue), in § 6.6.1
- href
  - [attribute for CSSImportRule](#dom-cssimportrule-href), in § 6.4.4
  - [attribute for StyleSheet](#dom-stylesheet-href), in § 6.1.1
- [http-default-style](#http-default-style), in § 10.1
- [IDL attribute to CSS property](#idl-attribute-to-css-property), in § 6.6.1
- [important flag](#css-declaration-important-flag), in § 6.5
- [IMPORT_RULE](#dom-cssrule-import_rule), in § 6.4.2
- [insert a CSS rule](#insert-a-css-rule), in § 6.4
- insertRule(rule)
  - [method for CSSGroupingRule](#dom-cssgroupingrule-insertrule), in § 6.4.5
  - [method for CSSStyleSheet](#dom-cssstylesheet-insertrule), in § 6.1.2
- insertRule(rule, index)
  - [method for CSSGroupingRule](#dom-cssgroupingrule-insertrule), in § 6.4.5
  - [method for CSSStyleSheet](#dom-cssstylesheet-insertrule), in § 6.1.2
- [intermixed properties](#intermixed-properties), in § 6.6
- item(index)
  - [method for CSSRuleList](#dom-cssrulelist-item), in § 6.4.1
  - [method for CSSStyleDeclaration](#dom-cssstyledeclaration-item), in § 6.6.1
  - [method for MediaList](#dom-medialist-item), in § 4.4
  - [method for StyleSheetList](#dom-stylesheetlist-item), in § 6.2.2
- [last CSS style sheet set name](#last-css-style-sheet-set-name), in § 6.2
- [layerName](#dom-cssimportrule-layername), in § 6.4.4
- length
  - [attribute for CSSRuleList](#dom-cssrulelist-length), in § 6.4.1
  - [attribute for CSSStyleDeclaration](#dom-cssstyledeclaration-length), in § 6.6.1
  - [attribute for MediaList](#dom-medialist-length), in § 4.4
  - [attribute for StyleSheetList](#dom-stylesheetlist-length), in § 6.2.2
- [LinkStyle](#linkstyle), in § 6.3.2
- [location](#concept-css-style-sheet-location), in § 6.1
- [margin](#dom-csspagedescriptors-margin), in § 6.4.7
- [margin-bottom](#dom-csspagedescriptors-margin-bottom), in § 6.4.7
- [marginBottom](#dom-csspagedescriptors-marginbottom), in § 6.4.7
- [margin-left](#dom-csspagedescriptors-margin-left), in § 6.4.7
- [marginLeft](#dom-csspagedescriptors-marginleft), in § 6.4.7
- [margin-right](#dom-csspagedescriptors-margin-right), in § 6.4.7
- [marginRight](#dom-csspagedescriptors-marginright), in § 6.4.7
- [MARGIN_RULE](#dom-cssrule-margin_rule), in § 6.4.2
- [margin-top](#dom-csspagedescriptors-margin-top), in § 6.4.7
- [marginTop](#dom-csspagedescriptors-margintop), in § 6.4.7
- [marks](#dom-csspagedescriptors-marks), in § 6.4.7
- media
  - [attribute for CSSImportRule](#dom-cssimportrule-media), in § 6.4.4
  - [attribute for StyleSheet](#dom-stylesheet-media), in § 6.1.1
  - [dfn for CSSStyleSheet](#concept-css-style-sheet-media), in § 6.1
  - [dict-member for CSSStyleSheetInit](#dom-cssstylesheetinit-media), in § 6.1.2
- [MediaList](#medialist), in § 4.4
- [MEDIA_RULE](#dom-cssrule-media_rule), in § 6.4.2
- [mediaText](#dom-medialist-mediatext), in § 4.4
- [name](#dom-cssmarginrule-name), in § 6.4.8
- [NAMESPACE_RULE](#dom-cssrule-namespace_rule), in § 6.4.2
- [namespaceURI](#dom-cssnamespacerule-namespaceuri), in § 6.4.9
- [origin-clean flag](#concept-css-style-sheet-origin-clean-flag), in § 6.1
- [owner CSS rule](#concept-css-style-sheet-owner-css-rule), in § 6.1
- owner node
  - [dfn for CSSStyleDeclaration](#cssstyledeclaration-owner-node), in § 6.6
  - [dfn for CSSStyleSheet](#concept-css-style-sheet-owner-node), in § 6.1
- [ownerNode](#dom-stylesheet-ownernode), in § 6.1.1
- [ownerRule](#dom-cssstylesheet-ownerrule), in § 6.1.2
- [page-orientation](#dom-csspagedescriptors-page-orientation), in § 6.4.7
- [pageOrientation](#dom-csspagedescriptors-pageorientation), in § 6.4.7
- [PAGE_RULE](#dom-cssrule-page_rule), in § 6.4.2
- parent CSS rule
  - [dfn for CSSRule](#concept-css-rule-parent-css-rule), in § 6.4
  - [dfn for CSSStyleDeclaration](#cssstyledeclaration-parent-css-rule), in § 6.6
- parent CSS style sheet
  - [dfn for CSSRule](#concept-css-rule-parent-css-style-sheet), in § 6.4
  - [dfn for CSSStyleSheet](#concept-css-style-sheet-parent-css-style-sheet), in § 6.1
- parentRule
  - [attribute for CSSRule](#dom-cssrule-parentrule), in § 6.4.2
  - [attribute for CSSStyleDeclaration](#dom-cssstyledeclaration-parentrule), in § 6.6.1
- parentStyleSheet
  - [attribute for CSSRule](#dom-cssrule-parentstylesheet), in § 6.4.2
  - [attribute for StyleSheet](#dom-stylesheet-parentstylesheet), in § 6.1.1
- [parse a CSS declaration block](#parse-a-css-declaration-block), in § 6.6
- [parse a CSS rule](#parse-a-css-rule), in § 6.4
- [parse a CSS value](#parse-a-css-value), in § 6.7.1
- [parse a group of selectors](#parse-a-group-of-selectors), in § 5.1
- [parse a list of CSS page selectors](#parse-a-list-of-css-page-selectors), in § 6.4.7
- [parse a media query](#parse-a-media-query), in § 4.1
- [parse a media query list](#parse-a-media-query-list), in § 4.1
- [persistent CSS style sheet](#persistent-css-style-sheet), in § 6.2
- [preferred CSS style sheet set name](#preferred-css-style-sheet-set-name), in § 6.2
- [preferred shorthand order](#concept-shorthands-preferred-order), in § 6.6
- [prefix](#dom-cssnamespacerule-prefix), in § 6.4.9
- [prolog](#prolog), in § 6.3.4
- [property name](#css-declaration-property-name), in § 6.5
- [readonly flag](#cssstyledeclaration-readonly-flag), in § 6.6
- [remove a CSS rule](#remove-a-css-rule), in § 6.4
- [remove a CSS style sheet](#remove-a-css-style-sheet), in § 6.2
- [removeProperty(property)](#dom-cssstyledeclaration-removeproperty), in § 6.6.1
- [removeRule()](#dom-cssstylesheet-removerule), in § 6.1.2.1
- [removeRule(index)](#dom-cssstylesheet-removerule), in § 6.1.2.1
- [replaceSync(text)](#dom-cssstylesheet-replacesync), in § 6.1.2
- [replace(text)](#dom-cssstylesheet-replace), in § 6.1.2
- [resolved value](#resolved-value), in § 9
- [resolved value special case property](#resolved-value-special-case-property), in § 9
- [resolved value special case property like color](#resolved-value-special-case-property-like-color), in § 9
- [resolved value special case property like height](#resolved-value-special-case-property-like-height), in § 9
- [resolved value special case property like top](#resolved-value-special-case-property-like-top), in § 9
- [rules](#dom-cssstylesheet-rules), in § 6.1.2.1
- [select a CSS style sheet set](#select-a-css-style-sheet-set), in § 6.2
- selectorText
  - [attribute for CSSPageRule](#dom-csspagerule-selectortext), in § 6.4.7
  - [attribute for CSSStyleRule](#dom-cssstylerule-selectortext), in § 6.4.3
- [serialize a comma-separated list](#serialize-a-comma-separated-list), in § 2.1
- [serialize a CSS component value](#serialize-a-css-component-value), in § 6.7.2
- [serialize a CSS declaration](#serialize-a-css-declaration), in § 6.6
- [serialize a CSS declaration block](#serialize-a-css-declaration-block), in § 6.6
- [serialize a CSS rule](#serialize-a-css-rule), in § 6.4
- [serialize a CSS value](#serialize-a-css-value), in § 6.7.2
- [serialize a function](#css-serialize-a-function), in § 2.1
- [serialize a group of selectors](#serialize-a-group-of-selectors), in § 5.2
- [serialize a list of CSS page selectors](#serialize-a-list-of-css-page-selectors), in § 6.4.7
- [serialize a LOCAL](#serialize-a-local), in § 2.1
- [serialize a media feature value](#serialize-a-media-feature-value), in § 4.2.1
- [serialize a media query](#serialize-a-media-query), in § 4.2
- [serialize a media query list](#serialize-a-media-query-list), in § 4.2
- [serialize an identifier](#serialize-an-identifier), in § 2.1
- [serialize as a string](#serialize-a-string), in § 2.1
- [serialize a selector](#serialize-a-selector), in § 5.2
- [serialize a simple selector](#serialize-a-simple-selector), in § 5.2
- [serialize a string](#serialize-a-string), in § 2.1
- [serialize a URL](#serialize-a-url), in § 2.1
- [serialize a whitespace-separated list](#serialize-a-whitespace-separated-list), in § 2.1
- [serialize into a shorthand form](#serialize-into-a-shorthand-form), in § 6.6
- [set](#set), in § 2
- [set a CSS declaration](#set-a-css-declaration), in § 6.6.1
- [setProperty(property, value)](#dom-cssstyledeclaration-setproperty), in § 6.6.1
- [setProperty(property, value, priority)](#dom-cssstyledeclaration-setproperty), in § 6.6.1
- [sheet](#dom-linkstyle-sheet), in § 6.3.2
- [size](#dom-csspagedescriptors-size), in § 6.4.7
- [specified order](#concept-declarations-specified-order), in § 6.4.3
- [stringification behavior](#MediaList-stringification-behavior), in § 4.4
- style
  - [attribute for CSSMarginRule](#dom-cssmarginrule-style), in § 6.4.8
  - [attribute for CSSPageRule](#dom-csspagerule-style), in § 6.4.7
  - [attribute for CSSStyleRule](#dom-cssstylerule-style), in § 6.4.3
  - [attribute for ElementCSSInlineStyle](#dom-elementcssinlinestyle-style), in § 7.1
- [STYLE_RULE](#dom-cssrule-style_rule), in § 6.4.2
- [StyleSheet](#stylesheet), in § 6.1.1
- [styleSheet](#dom-cssimportrule-stylesheet), in § 6.4.4
- [stylesheet base URL](#concept-css-style-sheet-stylesheet-base-url), in § 6.1
- [StyleSheetList](#stylesheetlist), in § 6.2.2
- [styleSheets](#dom-documentorshadowroot-stylesheets), in § 6.2.3
- [supported CSS property](#supported-css-property), in § 2
- [supported styling language](#supported-styling-language), in § 2
- [supportsText](#dom-cssimportrule-supportstext), in § 6.4.4
- [synchronously replace the rules of a CSSStyleSheet](#synchronously-replace-the-rules-of-a-cssstylesheet), in § 6.1.2
- [text](#concept-css-rule-text), in § 6.4
- title
  - [attribute for StyleSheet](#dom-stylesheet-title), in § 6.1.1
  - [dfn for CSSStyleSheet](#concept-css-style-sheet-title), in § 6.1
- type
  - [attribute for CSSRule](#dom-cssrule-type), in § 6.4.2
  - [attribute for StyleSheet](#dom-stylesheet-type), in § 6.1.1
  - [dfn for CSSRule](#concept-css-rule-type), in § 6.4
  - [dfn for CSSStyleSheet](#concept-css-style-sheet-type), in § 6.1
- [unset](#unset), in § 2
- [update style attribute for](#update-style-attribute-for), in § 6.6
- [updating flag](#cssstyledeclaration-updating-flag), in § 6.6
- [value](#css-declaration-value), in § 6.5
- [webkit-cased attribute](#dom-cssstyleproperties-webkit-cased-attribute), in § 6.6.1
- [webkit_cased_attribute](#dom-cssstyleproperties-webkit_cased_attribute), in § 6.6.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference[](#index-defined-elsewhere)

- \[\] defines the following terms:
  - <a id="cea6420e"></a>a style sheet that is blocking scripts
  - <a id="f8434dee"></a>being rendered
  - <a id="faf954d6"></a>browsing context
  - <a id="9e81ae64"></a>content-type metadata
  - <a id="5003dc6f"></a>contributes a script-blocking style sheet
  - <a id="6eb8a5d8"></a>document base url
  - <a id="74c04a49"></a>document url
  - <a id="a33db89a"></a>fetch
  - <a id="49a64d88"></a>html elements
  - <a id="003ea2bc"></a>long
  - <a id="eb1b1af3"></a>network error
  - <a id="5216e1a0"></a>node document
  - <a id="f78d5b5c"></a>origin
  - <a id="5f7259bd"></a>pseudo-attribute
  - <a id="fd11cdcd"></a>quirks mode
  - <a id="358f1dbd"></a>referrer
  - <a id="55213b5b"></a>request
  - <a id="dbbfc660"></a>script-blocking style sheet set
  - <a id="1d670c56"></a>supported property indices
  - <a id="1a4bc525"></a>throw
  - <a id="d1fe35a8"></a>tree order
  - <a id="dcffbccd"></a>URL
  - <a id="dc1cd39b"></a>url (for request)
  - <a id="ca3ca4ae"></a>URL parser
  - <a id="5442ea33"></a>URL serializer
  - <a id="6f0eb5e7"></a>xml-stylesheet processing instruction
- \[COMPAT\] defines the following terms:
  - <a id="e652d1b9"></a>-webkit-transform
- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="68164446"></a>CSSKeyframeRule
  - <a id="83120856"></a>CSSKeyframesRule
  - <a id="9af0059d"></a>cssRules
  - <a id="09633b18"></a>keyText
  - <a id="5c59509c"></a>name
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="11fb4d5b"></a>background-color
- \[CSS-BORDERS-4\] defines the following terms:
  - <a id="26b72b18"></a>border-block-end-color
  - <a id="7b5a0ddb"></a>border-block-start-color
  - <a id="7bb914ec"></a>border-bottom-color
  - <a id="60e9c55d"></a>border-inline-end-color
  - <a id="635a5b71"></a>border-inline-start-color
  - <a id="04aee887"></a>border-left-color
  - <a id="64e35b5b"></a>border-right-color
  - <a id="060efe1f"></a>border-top-color
  - <a id="b3527d2b"></a>box-shadow
- \[CSS-BOX-4\] defines the following terms:
  - <a id="cbfeec00"></a>margin-bottom
  - <a id="aaf6abed"></a>margin-left
  - <a id="ad065a5c"></a>margin-right
  - <a id="00140718"></a>margin-top
  - <a id="ca62982f"></a>padding-bottom
  - <a id="76207212"></a>padding-left
  - <a id="37c6bd4e"></a>padding-right
  - <a id="49a5f65f"></a>padding-top
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="ef21c66f"></a>break
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="9cd25054"></a>computed value
  - <a id="ed98cbd4"></a>layer name
  - <a id="1364cc84"></a>legacy shorthand
  - <a id="a4eb3c57"></a>used value
- \[CSS-CASCADE-6\] defines the following terms:
  - <a id="a803a89f"></a>&#64;import
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="79663d83"></a>\<alpha-value\>
  - <a id="73ea1d43"></a>color
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="1548047a"></a>\<color\>
- \[CSS-COUNTER-STYLES-3\] defines the following terms:
  - <a id="975ace19"></a>CSSCounterStyleRule
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="9ee4a8b2"></a>contents
  - <a id="e7f0dd6c"></a>display
  - <a id="55e8f5de"></a>none
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="cc103095"></a>\<font-family-name\>
  - <a id="6f1f6aac"></a>CSSFontFeatureValuesRule
  - <a id="465328f2"></a>font-family
  - <a id="00d808eb"></a>font-feature-settings
  - <a id="fdf6efd5"></a>font-size
  - <a id="e7cd6f5a"></a>font-stretch
  - <a id="d45afbab"></a>font-style
  - <a id="fb7153bd"></a>font-variant
  - <a id="a8fe91d7"></a>font-weight
  - <a id="dea91031"></a>unicode-range
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="3ca20be2"></a>CSSFontFaceRule
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="7522beee"></a>normal
- \[CSS-LISTS-3\] defines the following terms:
  - <a id="64fba2c3"></a>\<counter\>
- \[CSS-LOGICAL-1\] defines the following terms:
  - <a id="f0c46389"></a>block-size
  - <a id="0fffddd1"></a>inline-size
  - <a id="eb3aa672"></a>logical property group
  - <a id="8ca0e73e"></a>mapping logic
  - <a id="052c1740"></a>margin-block-end
  - <a id="12629b81"></a>margin-block-start
  - <a id="11c5a02b"></a>margin-inline-end
  - <a id="a6e20c61"></a>margin-inline-start
  - <a id="3bdfd276"></a>padding-block-end
  - <a id="5348b29d"></a>padding-block-start
  - <a id="8a0619b3"></a>padding-inline-end
  - <a id="002a383d"></a>padding-inline-start
- \[CSS-NAMESPACES-3\] defines the following terms:
  - <a id="de39d615"></a>default namespace
  - <a id="5ec1baf3"></a>namespace prefix
- \[CSS-NESTING-1\] defines the following terms:
  - <a id="9427c0d7"></a>nested declarations rule
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="86928bde"></a>overflow
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="a58a3cd8"></a>bottom
  - <a id="14b92907"></a>inset-block-end
  - <a id="ad49d317"></a>inset-block-start
  - <a id="8c3a081a"></a>inset-inline-end
  - <a id="e9c7fbb8"></a>inset-inline-start
  - <a id="dde41168"></a>left
  - <a id="b0b8d8c0"></a>right
  - <a id="e1483d91"></a>top
- \[CSS-SHADOW-1\] defines the following terms:
  - <a id="361a646a"></a>::part()
  - <a id="905d9a5a"></a>::slotted()
  - <a id="4cb3439f"></a>flat tree
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="68019d7a"></a>height
  - <a id="88643fe0"></a>width
- \[CSS-UI-4\] defines the following terms:
  - <a id="8e0f26cc"></a>caret-color
  - <a id="1ee7f091"></a>outline-color
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="3559c926"></a>\<angle\>
  - <a id="29a4aea4"></a>\<frequency\>
  - <a id="70c9f859"></a>\<integer\>
  - <a id="fb030e6c"></a>\<length\>
  - <a id="16310992"></a>\<number\>
  - <a id="15f5b381"></a>\<percentage\>
  - <a id="2de81520"></a>\<ratio\>
  - <a id="1dc65563"></a>\<resolution\>
  - <a id="977d3003"></a>\<string\>
  - <a id="03570f9e"></a>\<time\>
  - <a id="fa293cdd"></a>\<url\>
  - <a id="c8dc22e7"></a>keyword
  - <a id="9ddea951"></a>px
  - <a id="8a82fda1"></a>\|\|
- \[CSS-VARIABLES-1\] defines the following terms:
  - <a id="b5447e8b"></a>custom property
- \[CSS-VARIABLES-2\] defines the following terms:
  - <a id="eb7e3851"></a>guaranteed-invalid value
- \[CSS21\] defines the following terms:
  - <a id="fe1e6dc9"></a>\<identifier\>
  - <a id="3f47ca4a"></a>\<shape\>
  - <a id="ac54fbff"></a>line-height
- \[CSS3-CONDITIONAL\] defines the following terms:
  - <a id="12432dcb"></a>\<supports-condition\>
  - <a id="cad868ce"></a>CSSMediaRule
  - <a id="2e9d7788"></a>CSSSupportsRule
- \[CSS3SYN\] defines the following terms:
  - <a id="a942934d"></a>\<whitespace-token\>
  - <a id="7e4aa58b"></a>environment encoding
  - <a id="76c4403d"></a>parse
  - <a id="93665f4c"></a>parse a block's contents
  - <a id="569faf4d"></a>parse a list of component values
  - <a id="9bf5ff7b"></a>parse a rule
  - <a id="e92e094f"></a>parse a stylesheet's contents
  - <a id="ec82715c"></a>serialize an \<a-n-plus-b\> value
- \[CSSOM-1\] defines the following terms:
  - <a id="bbc31c9d"></a>location
  - <a id="aa155dab"></a>webkit-cased attribute
- \[DOM\] defines the following terms:
  - <a id="85394472"></a>Document
  - <a id="9ac2aadf"></a>DocumentOrShadowRoot
  - <a id="296f3551"></a>Element
  - <a id="c095490e"></a>ProcessingInstruction
  - <a id="d1a6c008"></a>attribute change steps
  - <a id="9638f92b"></a>connected
  - <a id="212ee1f7"></a>data
  - <a id="8d984e64"></a>encoding
  - <a id="6fba6744"></a>following
  - <a id="9bcac82c"></a>get an attribute by namespace and local name
  - <a id="a50a11ab"></a>in a document tree
  - <a id="d462b34f"></a>node
  - <a id="2ef14d2e"></a>nodes
  - <a id="8a3cbc07"></a>set an attribute value
  - <a id="ea2049db"></a>shadow-including root
- \[ENCODING\] defines the following terms:
  - <a id="4fdfba4e"></a>get an encoding
- \[HTML\] defines the following terms:
  - <a id="402ed79d"></a>CEReactions
  - <a id="b08d0bb2"></a>HTMLElement
  - <a id="5d7209e9"></a>Window
  - <a id="3349d69f"></a>associated Document
  - <a id="6a73137f"></a>CORS-same-origin
  - <a id="ce3d2bbb"></a>current global object
  - <a id="a72449dd"></a>in parallel
  - <a id="086e3aff"></a>origin
  - <a id="ba920583"></a>style
- \[I18N-GLOSSARY\] defines the following terms:
  - <a id="86c3e5ca"></a>case-sensitive
- \[INFRA\] defines the following terms:
  - <a id="53275e46"></a>append (for list)
  - <a id="a3b18719"></a>append (for set)
  - <a id="7f9469b5"></a>ASCII case-insensitive
  - <a id="6f2dfa22"></a>ASCII lowercase
  - <a id="a527e9ca"></a>ASCII uppercase
  - <a id="59912c93"></a>code unit
  - <a id="4a3bf5fb"></a>concatenate
  - <a id="f937b7b6"></a>continue
  - <a id="bf8d9616"></a>failure
  - <a id="16d07e10"></a>for each
  - <a id="649608b9"></a>list
  - <a id="0698d556"></a>string
  - <a id="a3fb968a"></a>surrogate
- \[MathML-Core\] defines the following terms:
  - <a id="57376736"></a>MathMLElement
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="dfe04a40"></a>aspect-ratio
  - <a id="344199b8"></a>color
  - <a id="0c6f4314"></a>color-index
  - <a id="e00f8df2"></a>device-aspect-ratio
  - <a id="46d08855"></a>device-height
  - <a id="ed56637c"></a>device-width
  - <a id="24cd7a9d"></a>grid
  - <a id="cb57cf48"></a>height
  - <a id="1fb62f09"></a>media feature
  - <a id="bd3d9ccd"></a>media query
  - <a id="cd31890e"></a>media query list
  - <a id="c93b716b"></a>media type
  - <a id="30efaeb2"></a>monochrome
  - <a id="8698c022"></a>orientation
  - <a id="42abe04d"></a>resolution
  - <a id="d1f57dfb"></a>scan
  - <a id="63e7492a"></a>width
- \[SELECTORS-3\] defines the following terms:
  - <a id="14048e52"></a>::after
  - <a id="bdad9fe9"></a>::before
- \[SELECTORS-4\] defines the following terms:
  - <a id="ad337600"></a>\<pseudo-element-selector\>
  - <a id="24408a11"></a>compound selector
  - <a id="069e89f6"></a>selector list
  - <a id="2fd70eda"></a>simple selector
  - <a id="84540613"></a>universal selector
- \[SVG2\] defines the following terms:
  - <a id="9c09cfc6"></a>SVGElement
- \[URL\] defines the following terms:
  - <a id="22ece843"></a>absolute-URL string
- \[WEBIDL\] defines the following terms:
  - <a id="dca2de17"></a>DOMException
  - <a id="8855a9aa"></a>DOMString
  - <a id="889e932f"></a>Exposed
  - <a id="4733bae2"></a>HierarchyRequestError
  - <a id="c9dac9f9"></a>IndexSizeError
  - <a id="797018a7"></a>InvalidStateError
  - <a id="2c4af168"></a>LegacyNullToEmptyString
  - <a id="c807e273"></a>NewObject
  - <a id="45999d1c"></a>NoModificationAllowedError
  - <a id="ba556545"></a>NotAllowedError
  - <a id="9eda9b58"></a>NotFoundError
  - <a id="fe15742e"></a>ObservableArray
  - <a id="bdbd19d1"></a>Promise
  - <a id="21ecf38f"></a>PutForwards
  - <a id="a5c91173"></a>SameObject
  - <a id="c3e881ef"></a>SecurityError
  - <a id="be2d2b4c"></a>SyntaxError
  - <a id="82ca3efc"></a>TypeError
  - <a id="b0d7f3c3"></a>USVString
  - <a id="811df97e"></a>backing list
  - <a id="5372cca8"></a>boolean
  - <a id="053b5fe6"></a>set an indexed value
  - <a id="4013a022"></a>this
  - <a id="5f90bbfb"></a>undefined
  - <a id="e97a9688"></a>unsigned long
  - <a id="450958f7"></a>unsigned short

## <a id="references"></a>References[](#references)

### <a id="normative"></a>Normative References[](#normative)

<a id="biblio-css-animations-1"></a><strong>\[CSS-ANIMATIONS-1\]</strong>

David Baron; et al. [CSS Animations Level 1](https://drafts.csswg.org/css-animations/). URL: <https://drafts.csswg.org/css-animations/>

<a id="biblio-css-backgrounds-3"></a><strong>\[CSS-BACKGROUNDS-3\]</strong>

Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://drafts.csswg.org/css-backgrounds/). URL: <https://drafts.csswg.org/css-backgrounds/>

<a id="biblio-css-borders-4"></a><strong>\[CSS-BORDERS-4\]</strong>

Elika Etemad; et al. [CSS Borders and Box Decorations Module Level 4](https://drafts.csswg.org/css-borders-4/). URL: <https://drafts.csswg.org/css-borders-4/>

<a id="biblio-css-box-4"></a><strong>\[CSS-BOX-4\]</strong>

Elika Etemad. [CSS Box Model Module Level 4](https://drafts.csswg.org/css-box-4/). URL: <https://drafts.csswg.org/css-box-4/>

<a id="biblio-css-cascade-5"></a><strong>\[CSS-CASCADE-5\]</strong>

Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://drafts.csswg.org/css-cascade-5/). URL: <https://drafts.csswg.org/css-cascade-5/>

<a id="biblio-css-cascade-6"></a><strong>\[CSS-CASCADE-6\]</strong>

Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://drafts.csswg.org/css-cascade-6/). URL: <https://drafts.csswg.org/css-cascade-6/>

<a id="biblio-css-color-4"></a><strong>\[CSS-COLOR-4\]</strong>

Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://drafts.csswg.org/css-color-4/). URL: <https://drafts.csswg.org/css-color-4/>

<a id="biblio-css-color-5"></a><strong>\[CSS-COLOR-5\]</strong>

Chris Lilley; Una Kravets; Lea Verou. [CSS Color Module Level 5](https://drafts.csswg.org/css-color-5/). URL: <https://drafts.csswg.org/css-color-5/>

<a id="biblio-css-counter-styles-3"></a><strong>\[CSS-COUNTER-STYLES-3\]</strong>

Tab Atkins Jr.. [CSS Counter Styles Level 3](https://drafts.csswg.org/css-counter-styles/). URL: <https://drafts.csswg.org/css-counter-styles/>

<a id="biblio-css-display-4"></a><strong>\[CSS-DISPLAY-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://drafts.csswg.org/css-display-4/). URL: <https://drafts.csswg.org/css-display-4/>

<a id="biblio-css-fonts-4"></a><strong>\[CSS-FONTS-4\]</strong>

Chris Lilley. [CSS Fonts Module Level 4](https://drafts.csswg.org/css-fonts-4/). URL: <https://drafts.csswg.org/css-fonts-4/>

<a id="biblio-css-fonts-5"></a><strong>\[CSS-FONTS-5\]</strong>

Chris Lilley. [CSS Fonts Module Level 5](https://drafts.csswg.org/css-fonts-5/). URL: <https://drafts.csswg.org/css-fonts-5/>

<a id="biblio-css-inline-3"></a><strong>\[CSS-INLINE-3\]</strong>

Elika Etemad. [CSS Inline Layout Module Level 3](https://drafts.csswg.org/css-inline-3/). URL: <https://drafts.csswg.org/css-inline-3/>

<a id="biblio-css-lists-3"></a><strong>\[CSS-LISTS-3\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://drafts.csswg.org/css-lists-3/). URL: <https://drafts.csswg.org/css-lists-3/>

<a id="biblio-css-logical-1"></a><strong>\[CSS-LOGICAL-1\]</strong>

Elika Etemad; Rossen Atanassov. [CSS Logical Properties and Values Module Level 1](https://drafts.csswg.org/css-logical-1/). URL: <https://drafts.csswg.org/css-logical-1/>

<a id="biblio-css-namespaces-3"></a><strong>\[CSS-NAMESPACES-3\]</strong>

Elika Etemad. [CSS Namespaces Module Level 3](https://drafts.csswg.org/css-namespaces/). URL: <https://drafts.csswg.org/css-namespaces/>

<a id="biblio-css-nesting-1"></a><strong>\[CSS-NESTING-1\]</strong>

Tab Atkins Jr.. [CSS Nesting Module Level 1](https://drafts.csswg.org/css-nesting/). URL: <https://drafts.csswg.org/css-nesting/>

<a id="biblio-css-overflow-3"></a><strong>\[CSS-OVERFLOW-3\]</strong>

Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://drafts.csswg.org/css-overflow-3/). URL: <https://drafts.csswg.org/css-overflow-3/>

<a id="biblio-css-position-3"></a><strong>\[CSS-POSITION-3\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://drafts.csswg.org/css-position-3/). URL: <https://drafts.csswg.org/css-position-3/>

<a id="biblio-css-shadow-1"></a><strong>\[CSS-SHADOW-1\]</strong>

[CSS Shadow Module Level 1](https://drafts.csswg.org/css-shadow-1/). Editor's Draft. URL: <https://drafts.csswg.org/css-shadow-1/>

<a id="biblio-css-sizing-3"></a><strong>\[CSS-SIZING-3\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://drafts.csswg.org/css-sizing-3/). URL: <https://drafts.csswg.org/css-sizing-3/>

<a id="biblio-css-ui-4"></a><strong>\[CSS-UI-4\]</strong>

Tab Atkins Jr.; Florian Rivoal. [CSS Basic User Interface Module Level 4](https://drafts.csswg.org/css-ui-4/). URL: <https://drafts.csswg.org/css-ui-4/>

<a id="biblio-css-values-4"></a><strong>\[CSS-VALUES-4\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: <https://drafts.csswg.org/css-values-4/>

<a id="biblio-css-variables-1"></a><strong>\[CSS-VARIABLES-1\]</strong>

Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://drafts.csswg.org/css-variables/). URL: <https://drafts.csswg.org/css-variables/>

<a id="biblio-css-variables-2"></a><strong>\[CSS-VARIABLES-2\]</strong>

[CSS Custom Properties for Cascading Variables Module Level 2](https://drafts.csswg.org/css-variables-2/). Editor's Draft. URL: <https://drafts.csswg.org/css-variables-2/>

<a id="biblio-css21"></a><strong>\[CSS21\]</strong>

Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://drafts.csswg.org/css2/). URL: <https://drafts.csswg.org/css2/>

<a id="biblio-css3-conditional"></a><strong>\[CSS3-CONDITIONAL\]</strong>

Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://drafts.csswg.org/css-conditional-3/). URL: <https://drafts.csswg.org/css-conditional-3/>

<a id="biblio-css3cascade"></a><strong>\[CSS3CASCADE\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://drafts.csswg.org/css-cascade-3/). URL: <https://drafts.csswg.org/css-cascade-3/>

<a id="biblio-css3page"></a><strong>\[CSS3PAGE\]</strong>

Elika Etemad. [CSS Paged Media Module Level 3](https://drafts.csswg.org/css-page-3/). URL: <https://drafts.csswg.org/css-page-3/>

<a id="biblio-css3syn"></a><strong>\[CSS3SYN\]</strong>

Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/). URL: <https://drafts.csswg.org/css-syntax/>

<a id="biblio-cssom-1"></a><strong>\[CSSOM-1\]</strong>

Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://drafts.csswg.org/cssom/). URL: <https://drafts.csswg.org/cssom/>

<a id="biblio-dom"></a><strong>\[DOM\]</strong>

Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: <https://dom.spec.whatwg.org/>

<a id="biblio-encoding"></a><strong>\[ENCODING\]</strong>

Anne van Kesteren. [Encoding Standard](https://encoding.spec.whatwg.org/). Living Standard. URL: <https://encoding.spec.whatwg.org/>

<a id="biblio-fetch"></a><strong>\[FETCH\]</strong>

Anne van Kesteren. [Fetch Standard](https://fetch.spec.whatwg.org/). Living Standard. URL: <https://fetch.spec.whatwg.org/>

<a id="biblio-html"></a><strong>\[HTML\]</strong>

Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: <https://html.spec.whatwg.org/multipage/>

<a id="biblio-i18n-glossary"></a><strong>\[I18N-GLOSSARY\]</strong>

Richard Ishida; Addison Phillips. [Internationalization Glossary](https://w3c.github.io/i18n-glossary/). URL: <https://w3c.github.io/i18n-glossary/>

<a id="biblio-infra"></a><strong>\[INFRA\]</strong>

Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: <https://infra.spec.whatwg.org/>

<a id="biblio-mathml-core"></a><strong>\[MathML-Core\]</strong>

David Carlisle; Frédéric Wang. [MathML Core](https://w3c.github.io/mathml-core/). URL: <https://w3c.github.io/mathml-core/>

<a id="biblio-mediaqueries"></a><strong>\[MEDIAQUERIES\]</strong>

Tab Atkins Jr.; Florian Rivoal. [Media Queries Level 4](https://drafts.csswg.org/mediaqueries-4/). URL: <https://drafts.csswg.org/mediaqueries-4/>

<a id="biblio-mediaqueries-5"></a><strong>\[MEDIAQUERIES-5\]</strong>

Tab Atkins Jr.; et al. [Media Queries Level 5](https://drafts.csswg.org/mediaqueries-5/). URL: <https://drafts.csswg.org/mediaqueries-5/>

<a id="biblio-rfc2119"></a><strong>\[RFC2119\]</strong>

S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: <https://datatracker.ietf.org/doc/html/rfc2119>

<a id="biblio-selectors-3"></a><strong>\[SELECTORS-3\]</strong>

Tantek Çelik; et al. [Selectors Level 3](https://drafts.csswg.org/selectors-3/). URL: <https://drafts.csswg.org/selectors-3/>

<a id="biblio-selectors-4"></a><strong>\[SELECTORS-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://drafts.csswg.org/selectors/). URL: <https://drafts.csswg.org/selectors/>

<a id="biblio-svg2"></a><strong>\[SVG2\]</strong>

Karl Dubost; et al. [Scalable Vector Graphics (SVG) 2](https://w3c.github.io/svgwg/svg2-draft/). URL: <https://w3c.github.io/svgwg/svg2-draft/>

<a id="biblio-url"></a><strong>\[URL\]</strong>

Anne van Kesteren. [URL Standard](https://url.spec.whatwg.org/). Living Standard. URL: <https://url.spec.whatwg.org/>

<a id="biblio-webidl"></a><strong>\[WEBIDL\]</strong>

Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: <https://webidl.spec.whatwg.org/>

<a id="biblio-xml"></a><strong>\[XML\]</strong>

Tim Bray; et al. [Extensible Markup Language (XML) 1.0 (Fifth Edition)](https://www.w3.org/TR/xml/). 26 November 2008. REC. URL: <https://www.w3.org/TR/xml/>

<a id="biblio-xml-stylesheet"></a><strong>\[XML-STYLESHEET\]</strong>

James Clark; Simon Pieters; Henry Thompson. [Associating Style Sheets with XML documents 1.0 (Second Edition)](https://www.w3.org/TR/xml-stylesheet/). 28 October 2010. REC. URL: <https://www.w3.org/TR/xml-stylesheet/>

### <a id="informative"></a>Non-Normative References[](#informative)

<a id="biblio-compat"></a><strong>\[COMPAT\]</strong>

Mike Taylor. [Compatibility Standard](https://compat.spec.whatwg.org/). Living Standard. URL: <https://compat.spec.whatwg.org/>

<a id="biblio-css-break-4"></a><strong>\[CSS-BREAK-4\]</strong>

Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://drafts.csswg.org/css-break-4/). URL: <https://drafts.csswg.org/css-break-4/>

<a id="biblio-svg11"></a><strong>\[SVG11\]</strong>

Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: <https://www.w3.org/TR/SVG11/>

## <a id="idl-index"></a>IDL Index[](#idl-index)

``` text
[Exposed=Window]
interface MediaList {
  stringifier attribute [LegacyNullToEmptyString] CSSOMString mediaText;
  readonly attribute unsigned long length;
  getter CSSOMString? item(unsigned long index);
  undefined appendMedium(CSSOMString medium);
  undefined deleteMedium(CSSOMString medium);
};

[Exposed=Window]
interface StyleSheet {
  readonly attribute CSSOMString type;
  readonly attribute USVString? href;
  readonly attribute (Element or ProcessingInstruction)? ownerNode;
  readonly attribute CSSStyleSheet? parentStyleSheet;
  readonly attribute DOMString? title;
  [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
  attribute boolean disabled;
};

[Exposed=Window]
interface CSSStyleSheet : StyleSheet {
  constructor(optional CSSStyleSheetInit options = {});

  readonly attribute CSSRule? ownerRule;
  [SameObject] readonly attribute CSSRuleList cssRules;
  unsigned long insertRule(CSSOMString rule, optional unsigned long index = 0);
  undefined deleteRule(unsigned long index);

  Promise<CSSStyleSheet> replace(USVString text);
  undefined replaceSync(USVString text);
};

dictionary CSSStyleSheetInit {
  DOMString? baseURL = null;
  (MediaList or DOMString) media = "";
  boolean disabled = false;
};

partial interface CSSStyleSheet {
  [SameObject] readonly attribute CSSRuleList rules;
  long addRule(optional DOMString selector = "undefined", optional DOMString style = "undefined", optional unsigned long index);
  undefined removeRule(optional unsigned long index = 0);
};

[Exposed=Window]
interface StyleSheetList {
  getter CSSStyleSheet? item(unsigned long index);
  readonly attribute unsigned long length;
};

partial interface mixin DocumentOrShadowRoot {
  [SameObject] readonly attribute StyleSheetList styleSheets;
  attribute ObservableArray<CSSStyleSheet> adoptedStyleSheets;
};

interface mixin LinkStyle {
  readonly attribute CSSStyleSheet? sheet;
};

ProcessingInstruction includes LinkStyle;
[Exposed=Window]
interface CSSRuleList {
  getter CSSRule? item(unsigned long index);
  readonly attribute unsigned long length;
};

[Exposed=Window]
interface CSSRule {
  attribute CSSOMString cssText;
  readonly attribute CSSRule? parentRule;
  readonly attribute CSSStyleSheet? parentStyleSheet;

  // the following attribute and constants are historical
  readonly attribute unsigned short type;
  const unsigned short STYLE_RULE = 1;
  const unsigned short CHARSET_RULE = 2;
  const unsigned short IMPORT_RULE = 3;
  const unsigned short MEDIA_RULE = 4;
  const unsigned short FONT_FACE_RULE = 5;
  const unsigned short PAGE_RULE = 6;
  const unsigned short MARGIN_RULE = 9;
  const unsigned short NAMESPACE_RULE = 10;
};

[Exposed=Window]
interface CSSStyleRule : CSSGroupingRule {
  attribute CSSOMString selectorText;
  [SameObject, PutForwards=cssText] readonly attribute CSSStyleProperties style;
};

[Exposed=Window]
interface CSSImportRule : CSSRule {
  readonly attribute USVString href;
  [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
  [SameObject] readonly attribute CSSStyleSheet? styleSheet;
  readonly attribute CSSOMString? layerName;
  readonly attribute CSSOMString? supportsText;
};

[Exposed=Window]
interface CSSGroupingRule : CSSRule {
  [SameObject] readonly attribute CSSRuleList cssRules;
  unsigned long insertRule(CSSOMString rule, optional unsigned long index = 0);
  undefined deleteRule(unsigned long index);
};

[Exposed=Window]
interface CSSPageDescriptors : CSSStyleDeclaration {
  attribute [LegacyNullToEmptyString] CSSOMString margin;
  attribute [LegacyNullToEmptyString] CSSOMString marginTop;
  attribute [LegacyNullToEmptyString] CSSOMString marginRight;
  attribute [LegacyNullToEmptyString] CSSOMString marginBottom;
  attribute [LegacyNullToEmptyString] CSSOMString marginLeft;
  attribute [LegacyNullToEmptyString] CSSOMString margin-top;
  attribute [LegacyNullToEmptyString] CSSOMString margin-right;
  attribute [LegacyNullToEmptyString] CSSOMString margin-bottom;
  attribute [LegacyNullToEmptyString] CSSOMString margin-left;
  attribute [LegacyNullToEmptyString] CSSOMString size;
  attribute [LegacyNullToEmptyString] CSSOMString pageOrientation;
  attribute [LegacyNullToEmptyString] CSSOMString page-orientation;
  attribute [LegacyNullToEmptyString] CSSOMString marks;
  attribute [LegacyNullToEmptyString] CSSOMString bleed;
};

[Exposed=Window]
interface CSSPageRule : CSSGroupingRule {
           attribute CSSOMString selectorText;
  [SameObject, PutForwards=cssText] readonly attribute CSSPageDescriptors style;
};

[Exposed=Window]
interface CSSMarginRule : CSSRule {
  readonly attribute CSSOMString name;
  [SameObject, PutForwards=cssText] readonly attribute CSSMarginDescriptors style;
};

[Exposed=Window]
interface CSSNamespaceRule : CSSRule {
  readonly attribute CSSOMString namespaceURI;
  readonly attribute CSSOMString prefix;
};

[Exposed=Window]
interface CSSStyleDeclaration {
  [CEReactions] attribute CSSOMString cssText;
  readonly attribute unsigned long length;
  getter CSSOMString item(unsigned long index);
  CSSOMString getPropertyValue(CSSOMString property);
  CSSOMString getPropertyPriority(CSSOMString property);
  [CEReactions] undefined setProperty(CSSOMString property, [LegacyNullToEmptyString] CSSOMString value, optional [LegacyNullToEmptyString] CSSOMString priority = "");
  [CEReactions] CSSOMString removeProperty(CSSOMString property);
  readonly attribute CSSRule? parentRule;
};

[Exposed=Window]
interface CSSStyleProperties : CSSStyleDeclaration {
  [CEReactions] attribute [LegacyNullToEmptyString] CSSOMString cssFloat;
};

interface mixin ElementCSSInlineStyle {
  [SameObject, PutForwards=cssText] readonly attribute CSSStyleProperties style;
};

HTMLElement includes ElementCSSInlineStyle;

SVGElement includes ElementCSSInlineStyle;

MathMLElement includes ElementCSSInlineStyle;

partial interface Window {
  [NewObject] CSSStyleProperties getComputedStyle(Element elt, optional CSSOMString? pseudoElt);
};

[Exposed=Window]
namespace CSS {
  CSSOMString escape(CSSOMString ident);
};
```

## <a id="issues-index"></a>Issues Index[](#issues-index)

This should probably be done in terms of mapping it to serializing CSS values as media features are defined in terms of CSS values after all.

<strong>Issue:</strong>

[↵](#issue-41128eee)

Be more specific

<strong>Issue:</strong>

[↵](#issue-d4a93110)

Is there a document at this point?

<strong>Issue:</strong>

[↵](#issue-af048285)

What if the HTML parser hasn’t decided on quirks/non-quirks yet?

<strong>Issue:</strong>

[↵](#issue-45012e41)

Need to define how the

<strong>Issue:</strong>

<code>[CSSFontFaceRule](https://drafts.csswg.org/css-fonts-5/#cssfontfacerule)</code> descriptors' values are serialized. [↵](#issue-f92cf3b3)

Need to define how

<strong>Issue:</strong>

<code>[CSSPageRule](#csspagerule)</code> is serialized. [↵](#issue-be6dc86c)

The "indented by two spaces" bit matches browsers, but needs work, see

<strong>Issue:</strong>

[\#5494](https://github.com/w3c/csswg-drafts/issues/5494) [↵](#issue-ca02cb3b)

Need to define the rules for

<strong>Issue:</strong>

parse a list of CSS page selectors and serialize a list of CSS page selectors. [↵](#issue-ba9fab84)

Should we add something like "Any observable side effect must not be made outside

<strong>Issue:</strong>

<var>declarations</var>"? The current constraints sound like a hole for undefined behavior. [↵](#issue-aaf411cc)

Probably should distinguish between declared and computed / resolved values.

<strong>Issue:</strong>

[↵](#issue-87977b18)

Probably should distinguish between declared and computed / resolved values.

<strong>Issue:</strong>

[↵](#issue-87977b18%E2%91%A0)

Probably should distinguish between declared and computed / resolved values.

<strong>Issue:</strong>

[↵](#issue-87977b18%E2%91%A1)

This should differentiate declared and computed

<strong>Issue:</strong>

[\<url\>](https://drafts.csswg.org/css-values-4/#url-value) values, see [\#3195](https://github.com/w3c/csswg-drafts/issues/3195). [↵](#issue-c9f9e4e6)

One idea is that we can remove this section somewhere in the CSS3/CSS4 timeline by moving the above definitions to the drafts that define the CSS components.

<strong>Issue:</strong>

[↵](#issue-08e9ac89)

Some of these need to be updated per the new rules.

<strong>Issue:</strong>

[↵](#issue-3a42ec46)

There are UAs that handle shorthands, and all UAs handle shorthands that used to be longhands like

<strong>Issue:</strong>

[overflow](https://drafts.csswg.org/css-overflow-3/#propdef-overflow), see [\#2529](https://github.com/w3c/csswg-drafts/issues/2529). [↵](#issue-28428295)

Order of custom properties is currently undefined, see

<strong>Issue:</strong>

[\#4947](https://github.com/w3c/csswg-drafts/issues/4947). [↵](#issue-0b704bc1)

This was previously specified as an IDL interface that only held static methods. Switching to an IDL namespace is \*nearly\* identical, so it’s expected that there won’t be any compat concerns. If any are discovered, please report so we can consider reverting this change.

<strong>Issue:</strong>

[↵](#issue-24739c22)

<strong>✔</strong>MDN

[CSS/escape_static](https://developer.mozilla.org/en-US/docs/Web/API/CSS/escape_static)

In all current engines.

Firefox31+Safari10.1+Chrome46+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSS](https://developer.mozilla.org/en-US/docs/Web/API/CSS)

In all current engines.

Firefox22+Safari9+Chrome28+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSGroupingRule/cssRules](https://developer.mozilla.org/en-US/docs/Web/API/CSSGroupingRule/cssRules)

In all current engines.

Firefox20+Safari3+Chrome45+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSGroupingRule/deleteRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSGroupingRule/deleteRule)

In all current engines.

Firefox20+Safari3+Chrome45+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSGroupingRule/insertRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSGroupingRule/insertRule)

In all current engines.

Firefox20+Safari3+Chrome45+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSGroupingRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSGroupingRule)

In all current engines.

Firefox20+Safari14.1+Chrome45+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSImportRule/href](https://developer.mozilla.org/en-US/docs/Web/API/CSSImportRule/href)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSImportRule/layerName](https://developer.mozilla.org/en-US/docs/Web/API/CSSImportRule/layerName)

In all current engines.

Firefox97+Safari15.4+Chrome99+

------------------------------------------------------------------------

Opera?Edge99+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSImportRule/media](https://developer.mozilla.org/en-US/docs/Web/API/CSSImportRule/media)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSImportRule/styleSheet](https://developer.mozilla.org/en-US/docs/Web/API/CSSImportRule/styleSheet)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>⚠</strong>MDN

[CSSImportRule/supportsText](https://developer.mozilla.org/en-US/docs/Web/API/CSSImportRule/supportsText)

In only one current engine.

Firefox114+SafariNoneChromeNone

------------------------------------------------------------------------

OperaNoneEdgeNone

------------------------------------------------------------------------

Edge (Legacy)NoneIENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera MobileNone

<strong>✔</strong>MDN

[CSSImportRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSImportRule)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSNamespaceRule/namespaceURI](https://developer.mozilla.org/en-US/docs/Web/API/CSSNamespaceRule/namespaceURI)

In all current engines.

Firefox59+Safari10.1+Chrome47+

------------------------------------------------------------------------

Opera36+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile36+

<strong>✔</strong>MDN

[CSSNamespaceRule/prefix](https://developer.mozilla.org/en-US/docs/Web/API/CSSNamespaceRule/prefix)

In all current engines.

Firefox59+Safari10.1+Chrome47+

------------------------------------------------------------------------

Opera36+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile36+

<strong>✔</strong>MDN

[CSSNamespaceRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSNamespaceRule)

In all current engines.

Firefox53+Safari10.1+Chrome47+

------------------------------------------------------------------------

Opera36+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile36+

<strong>✔</strong>MDN

[CSSPageRule/selectorText](https://developer.mozilla.org/en-US/docs/Web/API/CSSPageRule/selectorText)

In all current engines.

Firefox110+Safari3+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari1+Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSPageRule/style](https://developer.mozilla.org/en-US/docs/Web/API/CSSPageRule/style)

In all current engines.

Firefox19+Safari3+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari1+Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSPageRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSPageRule)

In all current engines.

Firefox19+Safari3+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari1+Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSRule/cssText](https://developer.mozilla.org/en-US/docs/Web/API/CSSRule/cssText)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSRule/parentRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSRule/parentRule)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSRule/parentStyleSheet](https://developer.mozilla.org/en-US/docs/Web/API/CSSRule/parentStyleSheet)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSRule)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSRuleList/item](https://developer.mozilla.org/en-US/docs/Web/API/CSSRuleList/item)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSRuleList/length](https://developer.mozilla.org/en-US/docs/Web/API/CSSRuleList/length)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSRuleList](https://developer.mozilla.org/en-US/docs/Web/API/CSSRuleList)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

MDN

[CSSStyleDeclaration/cssFloat](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration/cssFloat)

Firefox1–17Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleDeclaration/cssText](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration/cssText)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE5+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleDeclaration/getPropertyPriority](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration/getPropertyPriority)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleDeclaration/getPropertyValue](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration/getPropertyValue)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleDeclaration/item](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration/item)

In all current engines.

Firefox1+Safari6+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleDeclaration/length](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration/length)

In all current engines.

Firefox1+Safari6+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleDeclaration/parentRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration/parentRule)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSStyleDeclaration/removeProperty](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration/removeProperty)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleDeclaration/setProperty](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration/setProperty)

In all current engines.

Firefox1+Safari6+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[CSSStyleDeclaration](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE5+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleRule/selectorText](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleRule/selectorText)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleRule/style](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleRule/style)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleRule)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleSheet/CSSStyleSheet](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/CSSStyleSheet)

In all current engines.

Firefox101+Safari16.4+Chrome73+

------------------------------------------------------------------------

Opera53+Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet9.0+Opera Mobile47+

<strong>✔</strong>MDN

[CSSStyleSheet/cssRules](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/cssRules)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleSheet/deleteRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/deleteRule)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleSheet/insertRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/insertRule)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleSheet/ownerRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/ownerRule)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[CSSStyleSheet/replace](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/replace)

In all current engines.

Firefox101+Safari16.4+Chrome73+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSStyleSheet/replaceSync](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/replaceSync)

In all current engines.

Firefox101+Safari16.4+Chrome73+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSStyleSheet](https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[Document/adoptedStyleSheets](https://developer.mozilla.org/en-US/docs/Web/API/Document/adoptedStyleSheets)

In all current engines.

Firefox101+Safari16.4+Chrome73+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile50+

[ShadowRoot/adoptedStyleSheets](https://developer.mozilla.org/en-US/docs/Web/API/ShadowRoot/adoptedStyleSheets)

In all current engines.

Firefox101+Safari16.4+Chrome73+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile50+

<strong>✔</strong>MDN

[Document/styleSheets](https://developer.mozilla.org/en-US/docs/Web/API/Document/styleSheets)

In all current engines.

Firefox1+Safari4+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE4+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

[ShadowRoot/styleSheets](https://developer.mozilla.org/en-US/docs/Web/API/ShadowRoot/styleSheets)

In all current engines.

Firefox63+Safari12.1+Chrome53+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[HTMLElement/style](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/style)

In all current engines.

Firefox1+Safari3+Chrome1+

------------------------------------------------------------------------

Opera8+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE5.5+

------------------------------------------------------------------------

Firefox for Android?iOS Safari1+Chrome for Android?Android WebView?Samsung Internet?Opera Mobile10.1+

[SVGElement/style](https://developer.mozilla.org/en-US/docs/Web/API/SVGElement/style)

In all current engines.

Firefox1.5+Safari3+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari1+Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[HTMLLinkElement/sheet](https://developer.mozilla.org/en-US/docs/Web/API/HTMLLinkElement/sheet)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

[HTMLStyleElement/sheet](https://developer.mozilla.org/en-US/docs/Web/API/HTMLStyleElement/sheet)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

[ProcessingInstruction/sheet](https://developer.mozilla.org/en-US/docs/Web/API/ProcessingInstruction/sheet)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

[SVGStyleElement/sheet](https://developer.mozilla.org/en-US/docs/Web/API/SVGStyleElement/sheet)

In all current engines.

Firefox1.5+Safari16.4+Chrome38+

------------------------------------------------------------------------

Opera25+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile25+

<strong>✔</strong>MDN

[MediaList/appendMedium](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/appendMedium)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[MediaList/deleteMedium](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/deleteMedium)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[MediaList/item](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/item)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[MediaList/length](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/length)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[MediaList/mediaText](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/mediaText)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[MediaList](https://developer.mozilla.org/en-US/docs/Web/API/MediaList)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheet/disabled](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheet/disabled)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheet/href](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheet/href)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheet/media](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheet/media)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheet/ownerNode](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheet/ownerNode)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheet/parentStyleSheet](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheet/parentStyleSheet)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheet/title](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheet/title)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheet/type](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheet/type)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheet](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheet)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheetList/item](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheetList/item)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE4+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheetList/length](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheetList/length)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE4+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[StyleSheetList](https://developer.mozilla.org/en-US/docs/Web/API/StyleSheetList)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE4+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile12.1+

<strong>✔</strong>MDN

[Window/getComputedStyle](https://developer.mozilla.org/en-US/docs/Web/API/Window/getComputedStyle)

In all current engines.

Firefox1+Safari3+Chrome1+

------------------------------------------------------------------------

Opera7.2+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari1+Chrome for Android?Android WebView?Samsung Internet?Opera Mobile10.1+

<strong>⚠</strong>MDN

[Alternative_style_sheets](https://developer.mozilla.org/en-US/docs/Web/CSS/Alternative_style_sheets)

In only one current engine.

Firefox3+Safari?Chrome1–48

------------------------------------------------------------------------

OperaYesEdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IE8+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?
