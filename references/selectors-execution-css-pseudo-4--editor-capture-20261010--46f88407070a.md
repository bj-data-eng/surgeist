Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Pseudo-Elements Module Level 4](https://drafts.csswg.org/css-pseudo-4/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply. The capture’s full original notice and links remain below.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md).

# Source provenance

Title: CSS Pseudo-Elements Module Level 4

Source snapshot: https://drafts.csswg.org/css-pseudo-4/

Retrieved: 2026-10-10. The captured page states Editor’s Draft, 10 September 2026. An undated editor URL can change; the retrieval date and exact byte hash identify this captured HTML.

Captured HTML SHA-256: 46f88407070aa701eb2265a2e90d2e3890798504d5d661e5656976d8ca13e90c

Captured HTML revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Immutable upstream source: [Overview.bs](https://raw.githubusercontent.com/w3c/csswg-drafts/50337328484f7fcdd9ab6fe9ded6011815b1707e/css-pseudo-4/Overview.bs), SHA-256 `e63f43046719a9b3e0619bb0d4d4bb1637cabb4196200bc7f5e294367f652b18`. The source revision is inspected as reference material and is not executed.

Representation notes:

- Full format conversion of the captured HTML body, including status, metadata, bibliography, indexes, examples, test references, and legal notice. Script and style elements are omitted and were not executed.
- All source body IDs are retained, including those inside preformatted blocks, which are relocated immediately before those blocks. Fragment-only links stay local only when their exact captured target exists; other links remain upstream.
- All 0 tables have readable Markdown layouts. All tables have ordinary row/column layouts. Synthetic Field/Definition or Column N headings are non-normative. Source row headers remain bold; native HTML header/accessibility semantics are not expressible in GFM.
- The original passive inline SVG railroad diagram is retained as source HTML, including its coordinates and visible labels; no SVG scripts are executed.
- Preformatted examples retain literal text. Single-line table examples use semantic inline code. Compiler-generated hyperlinks inside fenced code blocks are omitted while their IDs and visible code text are retained; other source links are preserved. Small semantic code, variable, emphasis, subscript, and superscript HTML remains where Markdown notation would alter text.
- Conversion uses Pandoc 3.1.11.1 with focused Python/lxml preparation and a semantic Lua filter; the parsed GFM output is checked against the captured HTML. The [conversion report](CONVERSION-REPORT.md) records provenance and check boundaries.
- This full edition is a selected planning source for unstarted selector execution and scope-cascade work. The [source-edition mapping](SOURCE-EDITIONS.md) distinguishes these planning selections from historical editions consumed by completed CSS work.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Pseudo-Elements Module Level 4

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 10 September 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-pseudo-4/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-pseudo-4/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2020/WD-css-pseudo-4-20201231/>

<https://www.w3.org/TR/2019/WD-css-pseudo-4-20190225/>

<https://www.w3.org/TR/2016/WD-css-pseudo-4-20160607/>

<https://www.w3.org/TR/2015/WD-css-pseudo-4-20150115/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-pseudo-4)

[Inline In Spec](#issues-index)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Alan Stearns](mailto:stearns@adobe.com) (Adobe Systems Inc.)

<strong>Former Editor:</strong>

Daniel Glazman (Disruptive Innovations)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-pseudo-4/Overview.bs)

<strong>Issues List:</strong>

[Tracked in Editor’s Draft](https://drafts.csswg.org/css-pseudo/#issues-index)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-pseudo/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

This CSS module defines pseudo-elements, abstract elements that represent portions of the CSS render tree that can be selected and styled.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-pseudo” in the title, like this: “\[css-pseudo\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-pseudo%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

The following features are at-risk, and may be dropped during the CR period:

- the <a id="ref-for-selectordef-first-letter-prefix"></a>[::prefix](#selectordef-first-letter-prefix) and <a id="ref-for-selectordef-first-letter-suffix"></a>[::suffix](#selectordef-first-letter-suffix) sub-elements of <a id="ref-for-selectordef-first-letter"></a>[::first-letter](#selectordef-first-letter)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](#intro)
2.  [2 Typographic Pseudo-elements](#typographic-pseudos)
    1.  [2.1 First-Line Text: the ::first-line pseudo-element](#first-line-pseudo)
        1.  [2.1.1 Finding the First Formatted Line](#first-text-line)
        2.  [2.1.2 Styling the ::first-line Pseudo-element](#first-line-styling)
        3.  [2.1.3 Inheritance and the ::first-line Pseudo-element](#first-line-inheritance)
    2.  [2.2 First-Letter Text: ::first-letter pseudo-element and its ::prefix and ::suffix children](#first-letter-pseudo)
        1.  [2.2.1 First Letters and Associated Punctuation](#first-letter-pattern)
        2.  [2.2.2 Finding the First-Letter Text](#first-letter-application)
        3.  [2.2.3 Inheritance and Box Tree Structure of the First-Letter Pseudo-elements](#first-letter-tree)
        4.  [2.2.4 Styling the First-Letter Pseudo-elements](#first-letter-styling)
3.  [3 Highlight Pseudo-elements](#highlight-pseudos)
    1.  [3.1 Selecting Highlighted Content: the ::selection, ::search-text, ::target-text, ::spelling-error, ::grammar-error, and ::highlight() pseudo-elements](#highlight-selectors)
    2.  [3.2 Styling Highlights](#highlight-styling)
    3.  [3.3 Default UA Styles](#highlight-ua-styles)
        1.  [3.3.1 Paired Defaults](#paired-defaults)
    4.  [3.4 Area of a Highlight](#highlight-bounds)
    5.  [3.5 Cascading and Per-Element Highlight Styles](#highlight-cascade)
    6.  [3.6 Painting the Highlight](#highlight-painting)
        1.  [3.6.1 Backgrounds](#highlight-backgrounds)
        2.  [3.6.2 Shadows](#highlight-shadows)
        3.  [3.6.3 Text and Text Decorations](#highlight-text)
        4.  [3.6.4 Replaced Elements](#highlight-replaced)
    7.  [3.7 Security Considerations for Highlighting](#highlight-security)
4.  [4 Tree-Abiding Pseudo-elements](#treelike)
    1.  [4.1 Generated Content Pseudo-elements: ::before and ::after](#generated-content)
    2.  [4.2 List Markers: the ::marker pseudo-element](#marker-pseudo)
    3.  [4.3 Placeholder Input: the ::placeholder pseudo-element](#placeholder-pseudo)
5.  [5 Element-backed Pseudo-Elements](#element-like)
    1.  [5.1 File Selector Button: the ::file-selector-button pseudo-element](#file-selector-button-pseudo)
    2.  [5.2 Expandable contents of details element: the ::details-content pseudo-element](#details-content-pseudo)
6.  [6 Overlapping Pseudo-element Interactions](#interactions)
7.  [7 Additions to the CSS Object Model](#cssom)
    1.  [7.1 <code>CSSPseudoElement</code> Interface](#CSSPseudoElement-interface)
    2.  [7.2 <code>pseudo()</code> method of the <code>Element</code> interface](#window-interface)
    3.  [7.3 <code>getComputedStyle()</code>](#getComputedStyle)
8.  [8 Compatibility Syntax](#css2-compat)
9.  [ Security Considerations](#security)
10. [ Privacy Considerations](#privacy)
11. [ Changes](#changes)
12. [ Acknowledgements](#acknowledgements)
13. [ Conformance](#w3c-conformance)
    1.  [ Document conventions](#w3c-conventions)
    2.  [ Conformance classes](#w3c-conformance-classes)
    3.  [ Partial implementations](#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](#w3c-testing)
14. [ Index](#index)
    1.  [ Terms defined by this specification](#index-defined-here)
    2.  [ Terms defined by reference](#index-defined-elsewhere)
15. [ References](#references)
    1.  [ Normative References](#normative)
    2.  [ Non-Normative References](#informative)
16. [ IDL Index](#idl-index)
17. [ Issues Index](#issues-index)

## <a id="intro"></a>1. Introduction[](#intro)

<em>This section is informative.</em>

<a id="ref-for-pseudo-element"></a>[Pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) represent abstract elements of the document beyond those elements explicitly created by the document language. Since they are not restricted to fitting into the document tree, they can be used to select and style portions of the document that do not necessarily map to the document’s tree structure. For instance, the <a id="ref-for-selectordef-first-line"></a>[::first-line](#selectordef-first-line) pseudo-element can select content on the first formatted line of an element <em>after</em> text wrapping, allowing just that line to be styled differently from the rest of the paragraph.

Each pseudo-element is associated with an <a id="ref-for-originating-element"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) and has syntax of the form ::name-of-pseudo. This module defines the pseudo-elements that exist in CSS and how they can be styled. For more information on pseudo-elements in general, and on their syntax and interaction with other <a id="ref-for-selector"></a>[selectors](https://drafts.csswg.org/selectors-4/#selector), see [\[SELECTORS-4\]](#biblio-selectors-4).

Note: As a reminder, pseudo-elements cannot be chained together unless explicitly allowed. For example, ::marker::before is not allowed; but ::before::marker is.

Note: Level 4 is the first version of the CSS Pseudo specification under this name. It was branched from the Selectors 3 [\[SELECTORS-3\]](#biblio-selectors-3) specification, and thus started at level 4, with the remainder of Selectors 3 becoming Selectors 4 [\[SELECTORS-4\]](#biblio-selectors-4).

## <a id="typographic-pseudos"></a>2.  Typographic Pseudo-elements[](#typographic-pseudos)

### <a id="first-line-pseudo"></a>2.1.  First-Line Text: the <a id="ref-for-selectordef-first-line①"></a>[::first-line](#selectordef-first-line) pseudo-element[](#first-line-pseudo)

The <a id="selectordef-first-line"></a><strong>::first-line</strong> <a id="ref-for-pseudo-element①"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) represents the contents of the <a id="ref-for-first-formatted-line"></a>[first formatted line](#first-formatted-line) of its <a id="ref-for-originating-element①"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element).

Tests

- [first-line-marquee-crash.html](https://wpt.fyi/results/css/css-pseudo/crashtests/first-line-marquee-crash.html) [(live test)](http://wpt.live/css/css-pseudo/crashtests/first-line-marquee-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/crashtests/first-line-marquee-crash.html)
- [first-line-allowed-properties.html](https://wpt.fyi/results/css/css-pseudo/first-line-allowed-properties.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-allowed-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-allowed-properties.html)
- [first-line-and-marker.html](https://wpt.fyi/results/css/css-pseudo/first-line-and-marker.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-and-marker.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-and-marker.html)
- [first-line-and-placeholder.html](https://wpt.fyi/results/css/css-pseudo/first-line-and-placeholder.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-and-placeholder.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-and-placeholder.html)
- [first-line-change-inline-color-nested.html](https://wpt.fyi/results/css/css-pseudo/first-line-change-inline-color-nested.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-change-inline-color-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-change-inline-color-nested.html)
- [first-line-change-inline-color.html](https://wpt.fyi/results/css/css-pseudo/first-line-change-inline-color.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-change-inline-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-change-inline-color.html)
- [first-line-first-letter-insert-crash.html](https://wpt.fyi/results/css/css-pseudo/first-line-first-letter-insert-crash.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-first-letter-insert-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-first-letter-insert-crash.html)
- [first-line-float-mapped-attribute-crash.html](https://wpt.fyi/results/css/css-pseudo/first-line-float-mapped-attribute-crash.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-float-mapped-attribute-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-float-mapped-attribute-crash.html)
- [first-line-inherited-no-transition.html](https://wpt.fyi/results/css/css-pseudo/first-line-inherited-no-transition.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-inherited-no-transition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-inherited-no-transition.html)
- [first-line-inherited-transition-crash.html](https://wpt.fyi/results/css/css-pseudo/first-line-inherited-transition-crash.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-inherited-transition-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-inherited-transition-crash.html)
- [first-line-inherited-with-transition.html](https://wpt.fyi/results/css/css-pseudo/first-line-inherited-with-transition.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-inherited-with-transition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-inherited-with-transition.html)
- [first-line-input-image-crash.html](https://wpt.fyi/results/css/css-pseudo/first-line-input-image-crash.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-input-image-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-input-image-crash.html)
- [first-line-line-height-001.html](https://wpt.fyi/results/css/css-pseudo/first-line-line-height-001.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-line-height-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-line-height-001.html)
- [first-line-line-height-002.html](https://wpt.fyi/results/css/css-pseudo/first-line-line-height-002.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-line-height-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-line-height-002.html)
- [first-line-nested-gcs.html](https://wpt.fyi/results/css/css-pseudo/first-line-nested-gcs.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-nested-gcs.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-nested-gcs.html)
- [first-line-on-ancestor-block.html](https://wpt.fyi/results/css/css-pseudo/first-line-on-ancestor-block.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-on-ancestor-block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-on-ancestor-block.html)
- [first-line-opacity-001-ref.html](https://wpt.fyi/results/css/css-pseudo/first-line-opacity-001-ref.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-opacity-001-ref.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-opacity-001-ref.html)
- [first-line-opacity-001.html](https://wpt.fyi/results/css/css-pseudo/first-line-opacity-001.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-opacity-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-opacity-001.html)
- [first-line-replaced-001.html](https://wpt.fyi/results/css/css-pseudo/first-line-replaced-001.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-replaced-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-replaced-001.html)
- [first-line-with-before-after.html](https://wpt.fyi/results/css/css-pseudo/first-line-with-before-after.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-with-before-after.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-with-before-after.html)
- [first-line-with-inline-block-before.html](https://wpt.fyi/results/css/css-pseudo/first-line-with-inline-block-before.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-with-inline-block-before.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-with-inline-block-before.html)
- [first-line-with-inline-block.html](https://wpt.fyi/results/css/css-pseudo/first-line-with-inline-block.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-with-inline-block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-with-inline-block.html)
- [first-line-with-out-of-flow-and-nested-div.html](https://wpt.fyi/results/css/css-pseudo/first-line-with-out-of-flow-and-nested-div.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-with-out-of-flow-and-nested-div.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-with-out-of-flow-and-nested-div.html)
- [first-line-with-out-of-flow-and-nested-span.html](https://wpt.fyi/results/css/css-pseudo/first-line-with-out-of-flow-and-nested-span.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-with-out-of-flow-and-nested-span.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-with-out-of-flow-and-nested-span.html)
- [first-line-with-out-of-flow.html](https://wpt.fyi/results/css/css-pseudo/first-line-with-out-of-flow.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-with-out-of-flow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-with-out-of-flow.html)

<a id="example-fb83f35a"></a>

<strong>Example:</strong>

[](#example-fb83f35a) The rule below means “change the letters of the first line of every <code>p</code> element to uppercase”:

``` text
p::first-line { text-transform: uppercase }
```

The selector p::first-line does not match any real document element. It instead matches a pseudo-element that the user agent will automatically insert at the beginning of every <code>p</code> element.

Note: Note that the length of the first line depends on a number of factors, including the width of the page, the font size, etc.

<a id="first-line-example"></a>

<strong>Example:</strong>

[](#first-line-example) For example, given an ordinary HTML [\[HTML5\]](#biblio-html5) paragraph such as:

``` text
<P>This is a somewhat long HTML paragraph
that will be broken into several lines.
The first line will be styled
by the ‘::first-line’ pseudo-element.
The other lines will be treated
as ordinary lines in the paragraph.</P>
```

Depending on the width of the element, its lines might be broken as follows:

``` text
THIS IS A SOMEWHAT LONG HTML PARAGRAPH THAT
will be broken into several lines. The first
line will be styled by the ‘::first-line’
pseudo-element. The other lines will be
treated as ordinary lines in the paragraph.
```

or alternately as follows:

``` text
THIS IS A SOMEWHAT LONG
HTML paragraph that will
be broken into several
lines. The first line will
be styled by the
‘::first-line’ pseudo-
element. The other lines
will be treated as ordinary
lines in the paragraph.
```

#### <a id="first-text-line"></a>2.1.1.  Finding the First Formatted Line[](#first-text-line)

In CSS, the <a id="ref-for-selectordef-first-line②"></a>[::first-line](#selectordef-first-line) <a id="ref-for-pseudo-element②"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) can only have an effect when attached to a <a id="ref-for-block-container"></a>[block container](https://drafts.csswg.org/css-display-4/#block-container):

- The <a id="first-formatted-line"></a><strong>first formatted line</strong> of a <a id="ref-for-block-container①"></a>[block container](https://drafts.csswg.org/css-display-4/#block-container) that establishes an <a id="ref-for-inline-formatting-context"></a>[inline formatting context](https://drafts.csswg.org/css-display-4/#inline-formatting-context) represents the <a id="ref-for-inline-level"></a>[inline-level](https://drafts.csswg.org/css-display-4/#inline-level) content of its first <a id="ref-for-line-box"></a>[line box](https://drafts.csswg.org/css-inline-3/#line-box).

- The <a id="ref-for-first-formatted-line①"></a>[first formatted line](#first-formatted-line) of a <a id="ref-for-block-container②"></a>[block container](https://drafts.csswg.org/css-display-4/#block-container) or <a id="ref-for-multi-column-container"></a>[multi-column container](https://drafts.csswg.org/css-multicol-2/#multi-column-container) that contains <a id="ref-for-block-level"></a>[block-level](https://drafts.csswg.org/css-display-4/#block-level) content (and is not a <a id="ref-for-table-wrapper-box"></a>[table wrapper box](https://drafts.csswg.org/css-tables-3/#table-wrapper-box)) is the <a id="ref-for-first-formatted-line②"></a>first formatted line of its first <a id="ref-for-in-flow"></a>[in-flow](https://drafts.csswg.org/css-display-4/#in-flow) <a id="ref-for-block-level①"></a>block-level child. If no such line exists, it has no <a id="ref-for-first-formatted-line③"></a>first formatted line.

Note: The <a id="ref-for-first-formatted-line④"></a>[first formatted line](#first-formatted-line) can be an empty line. For example, the first line of the <code>p</code> in <code>&#60;p&#62;&#60;br&#62;First…</code> doesn’t contain any letters. Thus the word “First” is not on the first formatted line, and will not be affected by p::first-line.

Note: The first line of a <a id="ref-for-block-container③"></a>[block container](https://drafts.csswg.org/css-display-4/#block-container) that does not itself participate in a <a id="ref-for-block-formatting-context"></a>[block formatting context](https://drafts.csswg.org/css-display-4/#block-formatting-context) cannot be the first formatted line of an ancestor element. Thus, in <code>&#60;DIV&#62;&#60;P STYLE=&#34;display&#58; inline-block&#34;&#62;Hello&#60;BR&#62;Goodbye&#60;/P&#62; etcetera&#60;/DIV&#62;</code> the first formatted line of the <code>DIV</code> is not the line “Hello”, but rather the line that contains that entire inline block.

When a <a id="ref-for-first-formatted-line⑤"></a>[first formatted line](#first-formatted-line) is represented by multiple <a id="ref-for-selectordef-first-line③"></a>[::first-line](#selectordef-first-line) pseudo-elements, they are nested in the same order as their <a id="ref-for-originating-element②"></a>[originating elements](https://drafts.csswg.org/selectors-4/#originating-element). The <a id="ref-for-inline-level①"></a>[inline-level](https://drafts.csswg.org/css-display-4/#inline-level) contents of this line—​including its <a id="ref-for-root-inline-box"></a>[root inline box](https://drafts.csswg.org/css-inline-3/#root-inline-box) fragment—​are nested within the innermost <a id="ref-for-selectordef-first-line④"></a>::first-line <a id="ref-for-pseudo-element③"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element).

Tests

- [first-line-below-float.html](https://wpt.fyi/results/css/css-pseudo/first-line-below-float.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-below-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-below-float.html)
- [first-line-on-list-item-with-block-children.html](https://wpt.fyi/results/css/css-pseudo/first-line-on-list-item-with-block-children.html) [(live test)](http://wpt.live/css/css-pseudo/first-line-on-list-item-with-block-children.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-line-on-list-item-with-block-children.html)

<a id="example-786b8e79"></a>

<strong>Example:</strong>

[](#example-786b8e79) Consider the following markup:

``` text
<DIV>
  <P>First paragraph</P>
  <P>Second paragraph</P>
</DIV>
```

If we assume a <a id="ref-for-fictional-tag-sequence"></a>[fictional tag sequence](#fictional-tag-sequence) to represent the elements’ <a id="ref-for-selectordef-first-line⑤"></a>[::first-line](#selectordef-first-line) pseudo-elements, it would be something like:

``` text
<DIV>
  <P><DIV::first-line><P::first-line>First paragraph</P::first-line></DIV::first-line></P>
  <P><P::first-line>Second paragraph</P::first-line></P>
</DIV>
```

#### <a id="first-line-styling"></a>2.1.2.  Styling the <a id="ref-for-selectordef-first-line⑥"></a>[::first-line](#selectordef-first-line) Pseudo-element[](#first-line-styling)

The <a id="ref-for-selectordef-first-line⑦"></a>[::first-line](#selectordef-first-line) pseudo-element’s generated box behaves similar to that of an <a id="ref-for-inline-level-box"></a>[inline-level box](https://drafts.csswg.org/css-display-4/#inline-level-box), but with certain restrictions. The following CSS properties apply to a <a id="ref-for-selectordef-first-line⑧"></a>::first-line <a id="ref-for-pseudo-element④"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element):

- all font properties (see [\[CSS-FONTS-4\]](#biblio-css-fonts-4))
- the <a id="ref-for-propdef-color"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) and <a id="ref-for-propdef-opacity"></a>[opacity](https://drafts.csswg.org/css-color-4/#propdef-opacity) properties (see [\[CSS-COLOR-4\]](#biblio-css-color-4))
- all background properties (see [\[CSS-BACKGROUNDS-3\]](#biblio-css-backgrounds-3))
- any typesetting properties that apply to inline elements (see [\[CSS-TEXT-4\]](#biblio-css-text-4))
- all text decoration properties (see [\[CSS-TEXT-DECOR-4\]](#biblio-css-text-decor-4))
- the <a id="ref-for-propdef-ruby-position"></a>[ruby-position](https://drafts.csswg.org/css-ruby-1/#propdef-ruby-position) property (see [\[CSS-RUBY-1\]](#biblio-css-ruby-1))
- any inline layout properties that apply to inline elements (see [\[CSS-INLINE-3\]](#biblio-css-inline-3))
- any other properties defined to apply to <a id="ref-for-selectordef-first-line⑨"></a>[::first-line](#selectordef-first-line) by their respective specifications

User agents may apply other properties as well except for the following excluded properties:

- <a id="ref-for-propdef-writing-mode"></a>[writing-mode](https://drafts.csswg.org/css-writing-modes-4/#propdef-writing-mode)
- <a id="ref-for-propdef-direction"></a>[direction](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction)
- <a id="ref-for-propdef-text-orientation"></a>[text-orientation](https://drafts.csswg.org/css-writing-modes-4/#propdef-text-orientation)

Note: Setting <a id="ref-for-propdef-line-height"></a>[line-height](https://drafts.csswg.org/css2/#propdef-line-height) on <a id="ref-for-selectordef-first-line①⓪"></a>[::first-line](#selectordef-first-line) inherits to the fragment of the <a id="ref-for-root-inline-box①"></a>[root inline box](https://drafts.csswg.org/css-inline-3/#root-inline-box) that wraps the contents of the first line, and therefore can both increase and decrease the height of the first line box.

Tests

#### <a id="first-line-inheritance"></a>2.1.3.  Inheritance and the <a id="ref-for-selectordef-first-line①①"></a>[::first-line](#selectordef-first-line) Pseudo-element[](#first-line-inheritance)

During CSS <a id="ref-for-css-inheritance"></a>[inheritance](https://drafts.csswg.org/css-cascade-5/#css-inheritance), the <a id="ref-for-box-fragment"></a>[fragment](https://drafts.csswg.org/css-break-4/#box-fragment) of a child that occurs on the first line inherits any standard <a id="ref-for-inherited-property"></a>[inherited properties](https://drafts.csswg.org/css-cascade-5/#inherited-property)—​except the properties excluded above—​from the <a id="ref-for-selectordef-first-line①②"></a>[::first-line](#selectordef-first-line) <a id="ref-for-pseudo-element⑤"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element). For all other properties, including all <a id="ref-for-custom-property"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) [\[CSS-VARIABLES-1\]](#biblio-css-variables-1), inheritance is from the non-pseudo parent. (The portion of a child element that does not occur on the first line always inherits from the non-pseudo parent.)

<a id="example-62ec3857"></a>

<strong>Example:</strong>

[](#example-62ec3857) In the common case (of standard inherited CSS properties), <a id="ref-for-css-inheritance①"></a>[inheritance](https://drafts.csswg.org/css-cascade-5/#css-inheritance) into and from a <a id="ref-for-selectordef-first-line①③"></a>[::first-line](#selectordef-first-line) pseudo-element can be understood by writing out a <a id="fictional-tag-sequence"></a><strong>fictional tag sequence</strong> to represent <a id="ref-for-selectordef-first-line①④"></a>::first-line. Consider the [earlier example](#first-line-example); in case of the first rendering, the fictional tag sequence would be:

``` text
<P><p::first-line>This is a somewhat long HTML paragraph
that</p::first-line> will be broken into several lines.
The first line will be styled
by the ‘::first-line’ pseudo-element.
The other lines will be treated
as ordinary lines in the paragraph.</p>
```

And in the case of the second rendering:

``` text
<p><p::first-line>This is a somewhat long</p::first-line> HTML paragraph
that will be broken into several lines.
The first line will be styled
by the ‘::first-line’ pseudo-element.
The other lines will be treated
as ordinary lines in the paragraph.</p>
```

<a id="example-fce77ea8"></a>

<strong>Example:</strong>

[](#example-fce77ea8) If a pseudo-element breaks up a real element, the effect can often be described by a <a id="ref-for-fictional-tag-sequence①"></a>[fictional tag sequence](#fictional-tag-sequence) that closes and then re-opens the element. Suppose we mark up the earlier example with a <code>span</code> element encompassing the first sentence:

``` text
<p><span>This is a somewhat long HTML paragraph
that will be broken into several lines.</span>
The first line will be styled
by the ‘::first-line’ pseudo-element.
The other lines will be treated
as ordinary lines in the paragraph.</p>
```

The effect of the first rendering would be similar to the following <a id="ref-for-fictional-tag-sequence②"></a>[fictional tag sequence](#fictional-tag-sequence):

``` text
<p><p::first-line><span>This is a somewhat long HTML paragraph
that</span></p::first-line><span> will be broken into several lines.</span>
The first line will be styled
by the ‘::first-line’ pseudo-element.
The other lines will be treated
as ordinary lines in the paragraph.</p>
```

### <a id="first-letter-pseudo"></a>2.2.  First-Letter Text: <a id="ref-for-selectordef-first-letter①"></a>[::first-letter](#selectordef-first-letter) pseudo-element and its <a id="ref-for-selectordef-first-letter-prefix①"></a>[::prefix](#selectordef-first-letter-prefix) and <a id="ref-for-selectordef-first-letter-suffix①"></a>[::suffix](#selectordef-first-letter-suffix) children[](#first-letter-pseudo)

Tests

- [first-letter-001.html](https://wpt.fyi/results/css/css-pseudo/first-letter-001.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-001.html)
- [first-letter-002.html](https://wpt.fyi/results/css/css-pseudo/first-letter-002.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-002.html)
- [first-letter-003.html](https://wpt.fyi/results/css/css-pseudo/first-letter-003.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-003.html)
- [first-letter-004.html](https://wpt.fyi/results/css/css-pseudo/first-letter-004.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-004.html)
- [first-letter-005.html](https://wpt.fyi/results/css/css-pseudo/first-letter-005.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-005.html)
- [first-letter-allowed-properties.html](https://wpt.fyi/results/css/css-pseudo/first-letter-allowed-properties.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-allowed-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-allowed-properties.html)
- [first-letter-and-sibling-display-change.html](https://wpt.fyi/results/css/css-pseudo/first-letter-and-sibling-display-change.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-and-sibling-display-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-and-sibling-display-change.html)
- [first-letter-and-whitespace.html](https://wpt.fyi/results/css/css-pseudo/first-letter-and-whitespace.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-and-whitespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-and-whitespace.html)
- [first-letter-background-image-dynamic.html](https://wpt.fyi/results/css/css-pseudo/first-letter-background-image-dynamic.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-background-image-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-background-image-dynamic.html)
- [first-letter-background-image.html](https://wpt.fyi/results/css/css-pseudo/first-letter-background-image.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-background-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-background-image.html)
- [first-letter-bidi-pre-crash.html](https://wpt.fyi/results/css/css-pseudo/first-letter-bidi-pre-crash.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-bidi-pre-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-bidi-pre-crash.html)
- [first-letter-block-to-inline.html](https://wpt.fyi/results/css/css-pseudo/first-letter-block-to-inline.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-block-to-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-block-to-inline.html)
- [first-letter-crash.html](https://wpt.fyi/results/css/css-pseudo/first-letter-crash.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-crash.html)
- [first-letter-digraph.html](https://wpt.fyi/results/css/css-pseudo/first-letter-digraph.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-digraph.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-digraph.html)
- [first-letter-exclude-block-child-marker.html](https://wpt.fyi/results/css/css-pseudo/first-letter-exclude-block-child-marker.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-exclude-block-child-marker.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-exclude-block-child-marker.html)
- [first-letter-exclude-inline-child-marker.html](https://wpt.fyi/results/css/css-pseudo/first-letter-exclude-inline-child-marker.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-exclude-inline-child-marker.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-exclude-inline-child-marker.html)
- [first-letter-exclude-inline-marker.html](https://wpt.fyi/results/css/css-pseudo/first-letter-exclude-inline-marker.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-exclude-inline-marker.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-exclude-inline-marker.html)
- [first-letter-first-line-dynamic-001-crash.html](https://wpt.fyi/results/css/css-pseudo/first-letter-first-line-dynamic-001-crash.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-first-line-dynamic-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-first-line-dynamic-001-crash.html)
- [first-letter-hi-001.html](https://wpt.fyi/results/css/css-pseudo/first-letter-hi-001.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-hi-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-hi-001.html)
- [first-letter-hi-002.html](https://wpt.fyi/results/css/css-pseudo/first-letter-hi-002.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-hi-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-hi-002.html)
- [first-letter-list-item-dynamic-001.html](https://wpt.fyi/results/css/css-pseudo/first-letter-list-item-dynamic-001.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-list-item-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-list-item-dynamic-001.html)
- [first-letter-of-html-root-refcrash.html](https://wpt.fyi/results/css/css-pseudo/first-letter-of-html-root-refcrash.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-of-html-root-refcrash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-of-html-root-refcrash.html)
- [first-letter-opacity-001-ref.html](https://wpt.fyi/results/css/css-pseudo/first-letter-opacity-001-ref.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-opacity-001-ref.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-opacity-001-ref.html)
- [first-letter-opacity-001.html](https://wpt.fyi/results/css/css-pseudo/first-letter-opacity-001.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-opacity-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-opacity-001.html)
- [first-letter-opacity-float-001.html](https://wpt.fyi/results/css/css-pseudo/first-letter-opacity-float-001.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-opacity-float-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-opacity-float-001.html)
- [first-letter-punctuation-and-space.html](https://wpt.fyi/results/css/css-pseudo/first-letter-punctuation-and-space.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-punctuation-and-space.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-punctuation-and-space.html)
- [first-letter-punctuation-dynamic.html](https://wpt.fyi/results/css/css-pseudo/first-letter-punctuation-dynamic.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-punctuation-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-punctuation-dynamic.html)
- [first-letter-skip-empty-span-nested.html](https://wpt.fyi/results/css/css-pseudo/first-letter-skip-empty-span-nested.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-skip-empty-span-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-skip-empty-span-nested.html)
- [first-letter-skip-empty-span.html](https://wpt.fyi/results/css/css-pseudo/first-letter-skip-empty-span.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-skip-empty-span.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-skip-empty-span.html)
- [first-letter-skip-marker.html](https://wpt.fyi/results/css/css-pseudo/first-letter-skip-marker.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-skip-marker.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-skip-marker.html)
- [first-letter-text-and-display-change.html](https://wpt.fyi/results/css/css-pseudo/first-letter-text-and-display-change.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-text-and-display-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-text-and-display-change.html)
- [first-letter-width-2.html](https://wpt.fyi/results/css/css-pseudo/first-letter-width-2.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-width-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-width-2.html)
- [first-letter-width.html](https://wpt.fyi/results/css/css-pseudo/first-letter-width.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-width.html)
- [first-letter-with-before-after.html](https://wpt.fyi/results/css/css-pseudo/first-letter-with-before-after.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-with-before-after.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-with-before-after.html)
- [first-letter-with-preceding-new-line.html](https://wpt.fyi/results/css/css-pseudo/first-letter-with-preceding-new-line.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-with-preceding-new-line.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-with-preceding-new-line.html)
- [first-letter-with-quote.html](https://wpt.fyi/results/css/css-pseudo/first-letter-with-quote.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-with-quote.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-with-quote.html)
- [first-letter-with-span.html](https://wpt.fyi/results/css/css-pseudo/first-letter-with-span.html) [(live test)](http://wpt.live/css/css-pseudo/first-letter-with-span.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/first-letter-with-span.html)

![A drop-cap initial letter, including the opening quotation mark before it.](https://drafts.csswg.org/css-pseudo-4/images/first-letter2.gif)

The <a id="selectordef-first-letter"></a><strong>::first-letter</strong> <a id="ref-for-pseudo-element⑥"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) represents the first Letter, Number, or Symbol (Unicode category <code>L&#42;</code>, <code>N&#42;</code>, or <code>S&#42;</code>) <a id="ref-for-typographic-character-unit"></a>[typographic character unit](https://drafts.csswg.org/css-text-4/#typographic-character-unit) on the <a id="ref-for-first-formatted-line⑥"></a>[first formatted line](#first-formatted-line) of its <a id="ref-for-originating-element③"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) (the <a id="first-letter"></a><strong>first letter</strong>) as well as its associated punctuation. Collectively, this text is the <a id="first-letter-text"></a><strong>first-letter text</strong>. The <a id="ref-for-selectordef-first-letter②"></a>[::first-letter](#selectordef-first-letter) pseudo-element can be used to create “initial caps” and “drop caps”, which are common typographic effects.

<a id="example-2abd34c5"></a>

<strong>Example:</strong>

[](#example-2abd34c5) For example, the following rule creates a 2-line drop-letter on every paragraph following a level-2 header, using the <a id="ref-for-propdef-initial-letter"></a>[initial-letter](https://drafts.csswg.org/css-inline-3/#propdef-initial-letter) property defined in [\[CSS-INLINE-3\]](#biblio-css-inline-3):

``` text
h2 + p::first-letter { initial-letter: 3; }
```

Note: The <a id="ref-for-first-letter"></a>[first letter](#first-letter) may in fact be a digit, e.g., the “6” in “67 million dollars is a lot of money.”

To allow independent styling of the <a id="ref-for-first-letter①"></a>[first letter](#first-letter) itself and its adjacent punctuation, associated preceding punctuation is represented by the <a id="selectordef-first-letter-prefix"></a><strong>::prefix</strong> <a id="ref-for-sub-pseudo-element"></a>[sub-pseudo-element](https://drafts.csswg.org/selectors-4/#sub-pseudo-element) of the <a id="ref-for-selectordef-first-letter③"></a>[::first-letter](#selectordef-first-letter) <a id="ref-for-pseudo-element⑦"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) (::first-letter::prefix); and associated following punctuation is represented by the <a id="selectordef-first-letter-suffix"></a><strong>::suffix</strong> <a id="ref-for-sub-pseudo-element①"></a>sub-pseudo-element of the <a id="ref-for-selectordef-first-letter④"></a>::first-letter <a id="ref-for-pseudo-element⑧"></a>pseudo-element (::first-letter::suffix). See [§ 2.2.1 First Letters and Associated Punctuation](#first-letter-pattern), below.

#### <a id="first-letter-pattern"></a>2.2.1.  First Letters and Associated Punctuation[](#first-letter-pattern)

As explained in [CSS Text 3 § 1.4 Characters and Letters](https://drafts.csswg.org/css-text-3/#characters), a <a id="ref-for-typographic-character-unit①"></a>[typographic character unit](https://drafts.csswg.org/css-text-4/#typographic-character-unit) can include more than one Unicode codepoint. For example, combining characters must be kept with their base character. Also, languages may have additional rules about how to treat certain letter combinations. In Dutch, for example, if the letter combination "ij" appears at the beginning of an element, both letters should be considered within the <a id="ref-for-selectordef-first-letter⑤"></a>[::first-letter](#selectordef-first-letter) <a id="ref-for-pseudo-element⑨"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element). [\[UAX29\]](#biblio-uax29) When selecting the <a id="ref-for-first-letter②"></a>[first letter](#first-letter), the UA should tailor its definition of <a id="ref-for-typographic-character-unit②"></a>typographic character unit to reflect the first-letter traditions of the <a id="ref-for-selectordef-first-letter⑥"></a>::first-letter pseudo-element’s <em><a id="ref-for-containing-block"></a>[containing block](https://drafts.csswg.org/css-display-4/#containing-block)</em>’s <a id="ref-for-content-language"></a>[content language](https://drafts.csswg.org/css-text-4/#content-language).

Preceding and following punctuation must also be included as part of the <a id="ref-for-first-letter-text"></a>[first-letter text](#first-letter-text) in the <a id="ref-for-selectordef-first-letter⑦"></a>[::first-letter](#selectordef-first-letter) pseudo-element as follows:

- All punctuation—​i.e, characters that belong to the Punctuation (<code>P&#42;</code>) <a id="ref-for-unicode-general-category"></a>[Unicode general category](https://drafts.csswg.org/css-text-4/#unicode-general-category) [\[UAX44\]](#biblio-uax44)—​that precedes the <a id="ref-for-first-letter③"></a>[first letter](#first-letter), as well as any intervening typographic space—​characters belonging to the <code>Zs</code> Unicode general category [\[UAX44\]](#biblio-uax44) <em>other than</em> U+3000 IDEOGRAPHIC SPACE.
- Any punctuation other than opening punctuation and dashes—​i.e. characters that belong to the Punctuation (<code>P&#42;</code>) <a id="ref-for-unicode-general-category①"></a>[Unicode general category](https://drafts.csswg.org/css-text-4/#unicode-general-category), excluding Open Punctuation (<code>Ps</code>) and Dash Punctuation (<code>Pd</code>)—​that follows the <a id="ref-for-first-letter④"></a>[first letter](#first-letter), as well as any intervening typographic space—​characters belonging to the <code>Zs</code> Unicode general category [\[UAX44\]](#biblio-uax44) <em>other than</em> U+3000 IDEOGRAPHIC SPACE or a <a id="ref-for-word-separator"></a>[word separator](https://drafts.csswg.org/css-text-4/#word-separator).

Informally, the

<strong>Note:</strong>

<a id="ref-for-first-letter-text①"></a>[first-letter text](#first-letter-text)’s pattern here can be roughly (ignoring the exclusions from <code>Zs</code>) represented as a “regular expression” <code> (P (Zs&#124;P)&#42;)? (L&#124;N&#124;S) ((Zs&#124;P−(Ps&#124;Pd))&#42; (P−(Ps&#124;Pd))?</code>—​or, alternatively, <code> (&#91;P&#93; &#91;Zs P&#93;&#42;)? &#91;L N S&#93; (&#91;Zs &#91;P--&#91;Ps Pd&#93;&#93;&#93;&#42; &#91;P--&#91;Ps Pd&#93;&#93;)?</code>—​where the Unicode category abbreviation represents the set of all <a id="ref-for-typographic-character-unit③"></a>[typographic character units](https://drafts.csswg.org/css-text-4/#typographic-character-unit) belonging to that category.

<svg class="railroad-diagram" height="139" viewbox="0 0 775.0 139" width="775.0">
      <g transform="translate(.5 .5)">
       <g>
        <path d="M20 38v20m10 -20v20m-10 -10h20"></path>
       </g>
       <g>
        <path d="M40 48h0.0"></path>
        <path d="M255.5 48h0.0"></path>
        <path d="M40.0 48a10 10 0 0 0 10 -10v-8a10 10 0 0 1 10 -10"></path>
        <g>
         <path d="M60.0 20h175.5"></path>
        </g>
        <path d="M235.5 20a10 10 0 0 1 10 10v8a10 10 0 0 0 10 10"></path>
        <path d="M40.0 48h20"></path>
        <g>
         <path d="M60.0 48h0.0"></path>
         <path d="M235.5 48h0.0"></path>
         <g class="non-terminal">
          <path d="M60.0 48h0.0"></path>
          <path d="M88.5 48h0.0"></path>
          <rect height="22" width="28.5" x="60" y="37"></rect>
          <text x="74.25" y="52">P</text>
         </g>
         <path d="M88.5 48h10"></path>
         <g>
          <path d="M98.5 48h0.0"></path>
          <path d="M235.5 48h0.0"></path>
          <path d="M98.5 48a10 10 0 0 0 10 -10v0a10 10 0 0 1 10 -10"></path>
          <g>
           <path d="M118.5 28h97.0"></path>
          </g>
          <path d="M215.5 28a10 10 0 0 1 10 10v0a10 10 0 0 0 10 10"></path>
          <path d="M98.5 48h20"></path>
          <g>
           <path d="M118.5 48h0.0"></path>
           <path d="M215.5 48h0.0"></path>
           <path d="M118.5 48h10"></path>
           <g>
            <path d="M128.5 48h0.0"></path>
            <path d="M205.5 48h0.0"></path>
            <path d="M128.5 48h20"></path>
            <g class="non-terminal">
             <path d="M148.5 48h4.25"></path>
             <path d="M181.25 48h4.25"></path>
             <rect height="22" width="28.5" x="152.75" y="37"></rect>
             <text x="167" y="52">P</text>
            </g>
            <path d="M185.5 48h20"></path>
            <path d="M128.5 48a10 10 0 0 1 10 10v10a10 10 0 0 0 10 10"></path>
            <g class="non-terminal">
             <path d="M148.5 78h0.0"></path>
             <path d="M185.5 78h0.0"></path>
             <rect height="22" width="37" x="148.5" y="67"></rect>
             <text x="167" y="82">Zs</text>
            </g>
            <path d="M185.5 78a10 10 0 0 0 10 -10v-10a10 10 0 0 1 10 -10"></path>
           </g>
           <path d="M205.5 48h10"></path>
           <path d="M128.5 48a10 10 0 0 0 -10 10v29a10 10 0 0 0 10 10"></path>
           <g>
            <path d="M128.5 97h77.0"></path>
           </g>
           <path d="M205.5 97a10 10 0 0 0 10 -10v-29a10 10 0 0 0 -10 -10"></path>
          </g>
          <path d="M215.5 48h20"></path>
         </g>
        </g>
        <path d="M235.5 48h20"></path>
       </g>
       <g>
        <path d="M255.5 48h0.0"></path>
        <path d="M324.0 48h0.0"></path>
        <path d="M255.5 48h20"></path>
        <g class="non-terminal">
         <path d="M275.5 48h0.0"></path>
         <path d="M304.0 48h0.0"></path>
         <rect height="22" width="28.5" x="275.5" y="37"></rect>
         <text x="289.75" y="52">L</text>
        </g>
        <path d="M304.0 48h20"></path>
        <path d="M255.5 48a10 10 0 0 1 10 10v10a10 10 0 0 0 10 10"></path>
        <g class="non-terminal">
         <path d="M275.5 78h0.0"></path>
         <path d="M304.0 78h0.0"></path>
         <rect height="22" width="28.5" x="275.5" y="67"></rect>
         <text x="289.75" y="82">N</text>
        </g>
        <path d="M304.0 78a10 10 0 0 0 10 -10v-10a10 10 0 0 1 10 -10"></path>
        <path d="M255.5 48a10 10 0 0 1 10 10v40a10 10 0 0 0 10 10"></path>
        <g class="non-terminal">
         <path d="M275.5 108h0.0"></path>
         <path d="M304.0 108h0.0"></path>
         <rect height="22" width="28.5" x="275.5" y="97"></rect>
         <text x="289.75" y="112">S</text>
        </g>
        <path d="M304.0 108a10 10 0 0 0 10 -10v-40a10 10 0 0 1 10 -10"></path>
       </g>
       <g>
        <path d="M324.0 48h0.0"></path>
        <path d="M735.0 48h0.0"></path>
        <path d="M324.0 48a10 10 0 0 0 10 -10v-8a10 10 0 0 1 10 -10"></path>
        <g>
         <path d="M344.0 20h371.0"></path>
        </g>
        <path d="M715.0 20a10 10 0 0 1 10 10v8a10 10 0 0 0 10 10"></path>
        <path d="M324.0 48h20"></path>
        <g>
         <path d="M344.0 48h0.0"></path>
         <path d="M715.0 48h0.0"></path>
         <g class="non-terminal">
          <path d="M344.0 48h0.0"></path>
          <path d="M474.5 48h0.0"></path>
          <rect height="22" width="130.5" x="344" y="37"></rect>
          <text x="409.25" y="52">P − (Ps ∪ Pd)</text>
         </g>
         <path d="M474.5 48h10"></path>
         <g>
          <path d="M484.5 48h0.0"></path>
          <path d="M715.0 48h0.0"></path>
          <path d="M484.5 48a10 10 0 0 0 10 -10v0a10 10 0 0 1 10 -10"></path>
          <g>
           <path d="M504.5 28h190.5"></path>
          </g>
          <path d="M695.0 28a10 10 0 0 1 10 10v0a10 10 0 0 0 10 10"></path>
          <path d="M484.5 48h20"></path>
          <g>
           <path d="M504.5 48h0.0"></path>
           <path d="M695.0 48h0.0"></path>
           <path d="M504.5 48h10"></path>
           <g>
            <path d="M514.5 48h0.0"></path>
            <path d="M685.0 48h0.0"></path>
            <path d="M514.5 48h20"></path>
            <g class="non-terminal">
             <path d="M534.5 48h0.0"></path>
             <path d="M665.0 48h0.0"></path>
             <rect height="22" width="130.5" x="534.5" y="37"></rect>
             <text x="599.75" y="52">P − (Ps ∪ Pd)</text>
            </g>
            <path d="M665.0 48h20"></path>
            <path d="M514.5 48a10 10 0 0 1 10 10v10a10 10 0 0 0 10 10"></path>
            <g class="non-terminal">
             <path d="M534.5 78h46.75"></path>
             <path d="M618.25 78h46.75"></path>
             <rect height="22" width="37" x="581.25" y="67"></rect>
             <text x="599.75" y="82">Zs</text>
            </g>
            <path d="M665.0 78a10 10 0 0 0 10 -10v-10a10 10 0 0 1 10 -10"></path>
           </g>
           <path d="M685.0 48h10"></path>
           <path d="M514.5 48a10 10 0 0 0 -10 10v29a10 10 0 0 0 10 10"></path>
           <g>
            <path d="M514.5 97h170.5"></path>
           </g>
           <path d="M685.0 97a10 10 0 0 0 10 -10v-29a10 10 0 0 0 -10 -10"></path>
          </g>
          <path d="M695.0 48h20"></path>
         </g>
        </g>
        <path d="M715.0 48h20"></path>
       </g>
       <path d="M 735.0 48 h 20 m -10 -10 v 20 m 10 -20 v 20"></path>
      </g>
     </svg>

See [CSS Text 3 § 1.4 Characters and Letters](https://drafts.csswg.org/css-text-3/#characters) and [CSS Text 3 § E Characters and Properties](https://drafts.csswg.org/css-text-3/#character-properties) for more information on <a id="ref-for-typographic-character-unit④"></a>[typographic character units](https://drafts.csswg.org/css-text-4/#typographic-character-unit) and their Unicode properties. [\[CSS-TEXT-3\]](#biblio-css-text-3)

#### <a id="first-letter-application"></a>2.2.2.  Finding the First-Letter Text[](#first-letter-application)

As with <a id="ref-for-selectordef-first-line①⑤"></a>[::first-line](#selectordef-first-line), the <a id="ref-for-selectordef-first-letter⑧"></a>[::first-letter](#selectordef-first-letter) <a id="ref-for-pseudo-element①⓪"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) can only have an effect when attached to a <a id="ref-for-block-container④"></a>[block container](https://drafts.csswg.org/css-display-4/#block-container). Its <a id="ref-for-first-letter-text②"></a>[first-letter text](#first-letter-text) is the first such <a id="ref-for-inline-level②"></a>[inline-level content](https://drafts.csswg.org/css-display-4/#inline-level) participating in the <a id="ref-for-inline-formatting-context①"></a>[inline formatting context](https://drafts.csswg.org/css-display-4/#inline-formatting-context) of its <a id="ref-for-originating-element④"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element)’s <a id="ref-for-first-formatted-line⑦"></a>[first formatted line](#first-formatted-line), if it is not preceded by any other <a id="ref-for-in-flow①"></a>[in-flow](https://drafts.csswg.org/css-display-4/#in-flow) content (such as images or inline tables) on its line.

For this purpose, any <a id="ref-for-marker"></a>[marker boxes](https://drafts.csswg.org/css-lists-3/#marker) are ignored, as if they were <a id="ref-for-out-of-flow"></a>[out-of-flow](https://drafts.csswg.org/css-display-4/#out-of-flow). However, if an element has <a id="ref-for-in-flow②"></a>[in-flow](https://drafts.csswg.org/css-display-4/#in-flow) <a id="ref-for-selectordef-before"></a>[::before](#selectordef-before) or <a id="ref-for-selectordef-after"></a>[::after](#selectordef-after) content, the <a id="ref-for-first-letter-text③"></a>[first-letter text](#first-letter-text) is selected from the content of the element <em>including</em> that generated content.

<a id="example-179a5087"></a>[](#example-179a5087) Example: After the rule <code>p&#58;&#58;before {content&#58; &#34;Note&#58; &#34;}</code>, the selector p::first-letter matches the "N" of "Note".

If no qualifying text exists, then there is no <a id="ref-for-first-letter-text④"></a>[first-letter text](#first-letter-text) and no <a id="ref-for-selectordef-first-letter⑨"></a>[::first-letter](#selectordef-first-letter) pseudo-element.

Note: When the <a id="ref-for-first-formatted-line⑧"></a>[first formatted line](#first-formatted-line) is empty, <a id="ref-for-selectordef-first-letter①⓪"></a>[::first-letter](#selectordef-first-letter) will not match anything. For example, in this HTML fragment: <code>&#60;p&#62;&#60;br&#62;First&#46;&#46;&#46;</code> the first line doesn’t contain any letters, so <a id="ref-for-selectordef-first-letter①①"></a>::first-letter doesn’t match anything. In particular, it does not match the “F” of “First”, which is on the second line.

Note: As with <a id="ref-for-selectordef-first-line①⑥"></a>[::first-line](#selectordef-first-line), the <a id="ref-for-first-letter-text⑤"></a>[first-letter text](#first-letter-text) of a <a id="ref-for-block-container⑤"></a>[block container](https://drafts.csswg.org/css-display-4/#block-container) that does not participate in a <a id="ref-for-block-formatting-context①"></a>[block formatting context](https://drafts.csswg.org/css-display-4/#block-formatting-context) cannot be the <a id="ref-for-first-letter-text⑥"></a>first-letter text of an ancestor element. Thus, in <code>&#60;DIV&#62;&#60;P STYLE=&#34;display&#58; inline-block&#34;&#62;Hello&#60;BR&#62;Goodbye&#60;/P&#62; etcetera&#60;/DIV&#62;</code> the <a id="ref-for-first-letter⑤"></a>[first letter](#first-letter) of the <code>DIV</code> is not the letter “H”. In fact, the <code>DIV</code> doesn’t have a <a id="ref-for-first-letter⑥"></a>first letter.

Any portion of the <a id="ref-for-first-letter-text⑦"></a>[first-letter text](#first-letter-text) that is wrapped to the next line no longer forms part of the <a id="ref-for-selectordef-first-letter①②"></a>[::first-letter](#selectordef-first-letter) <a id="ref-for-pseudo-element①①"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element).

#### <a id="first-letter-tree"></a>2.2.3.  Inheritance and Box Tree Structure of the First-Letter Pseudo-elements[](#first-letter-tree)

The <a id="ref-for-selectordef-first-letter①③"></a>[::first-letter](#selectordef-first-letter) <a id="ref-for-pseudo-element①②"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) is wrapped immediately around the <a id="ref-for-first-letter-text⑧"></a>[first-letter text](#first-letter-text) it represents, even if that text is in a descendant. When a <a id="ref-for-first-letter-text⑨"></a>first-letter text is represented by multiple <a id="ref-for-selectordef-first-letter①④"></a>::first-letter pseudo-elements, they are nested in the same order as their <a id="ref-for-originating-element⑤"></a>[originating elements](https://drafts.csswg.org/selectors-4/#originating-element). <a id="ref-for-css-inheritance②"></a>[Inheritance](https://drafts.csswg.org/css-cascade-5/#css-inheritance) behaves accordingly.

<a id="example-fd33f595"></a>

<strong>Example:</strong>

[](#example-fd33f595) Consider the following markup:

``` text
<div>
  <p><span>The first few words</span>
  and the rest of the paragraph.
</div>
```

If we assume a <a id="ref-for-fictional-tag-sequence③"></a>[fictional tag sequence](#fictional-tag-sequence) to represent the elements’ <a id="ref-for-selectordef-first-letter①⑤"></a>[::first-letter](#selectordef-first-letter) pseudo-elements, it would be something like:

``` text
<div>
  <p><span><div::first-letter><p::first-letter>T</…></…>he first few words</span>
  and the rest of the paragraph.
</div>
```

If any ::first-letter::prefix or ::first-letter::suffix <a id="ref-for-pseudo-element①③"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) exist, they are nested within the innermost <a id="ref-for-selectordef-first-letter①⑥"></a>[::first-letter](#selectordef-first-letter), and otherwise interpreted similar to <a id="ref-for-selectordef-first-letter①⑦"></a>::first-letter itself.

<a id="example-af9bfeee"></a>

<strong>Example:</strong>

[](#example-af9bfeee) Consider the following markup:

``` text
<div>
  <p><span>“The first few words</span>
  and the rest of the quotation.
</div>
```

If we assume a <a id="ref-for-fictional-tag-sequence④"></a>[fictional tag sequence](#fictional-tag-sequence) to represent the elements’ <a id="ref-for-selectordef-first-letter①⑧"></a>[::first-letter](#selectordef-first-letter) pseudo-elements, it would be something like:

``` text
<div>
  <p><span><div::first-letter><p::first-letter><div::first-letter::prefix><p::first-letter::prefix>“</…></…>T</…></…>he first few words</span>
  and the rest of the paragraph.
</div>
```

If the characters that would form the <a id="ref-for-first-letter-text①⓪"></a>[first-letter text](#first-letter-text) are not all in the same element (as the <code>‘T</code> in <code>&#60;p&#62;‘&#60;em&#62;T&#46;&#46;&#46;</code>), the user agent may create the <a id="ref-for-selectordef-first-letter①⑨"></a>[::first-letter](#selectordef-first-letter) <a id="ref-for-pseudo-element①④"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) (and its <a id="ref-for-selectordef-first-letter-prefix②"></a>[::prefix](#selectordef-first-letter-prefix) or <a id="ref-for-selectordef-first-letter-suffix②"></a>[::suffix](#selectordef-first-letter-suffix) sub-elements, if any) from one of the elements, or all elements, or simply not create the pseudo-element(s). Additionally, if the <a id="ref-for-first-letter-text①①"></a>first-letter text is not at the start of the line (for example due to bidirectional reordering, or due to a <a id="ref-for-list-item"></a>[list item](https://drafts.csswg.org/css-lists-3/#list-item) <a id="ref-for-marker①"></a>[marker](https://drafts.csswg.org/css-lists-3/#marker) with <a id="ref-for-propdef-list-style-position"></a>[list-style-position: inside](https://drafts.csswg.org/css-lists-3/#propdef-list-style-position)), then the user agent is not required to create the pseudo-element(s).

A <a id="ref-for-selectordef-first-letter②⓪"></a>[::first-letter](#selectordef-first-letter) pseudo-element is contained within any <a id="ref-for-selectordef-first-line①⑦"></a>[::first-line](#selectordef-first-line) pseudo-elements, and thus inherits (potentially indirectly) from <a id="ref-for-selectordef-first-line①⑧"></a>::first-line, the same as any <a id="ref-for-inline-box"></a>[inline box](https://drafts.csswg.org/css-display-4/#inline-box) on the same line.

#### <a id="first-letter-styling"></a>2.2.4.  Styling the First-Letter Pseudo-elements[](#first-letter-styling)

In CSS a <a id="ref-for-selectordef-first-letter②①"></a>[::first-letter](#selectordef-first-letter) <a id="ref-for-pseudo-element①⑤"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) (and its <a id="ref-for-selectordef-first-letter-prefix③"></a>[::prefix](#selectordef-first-letter-prefix) and <a id="ref-for-selectordef-first-letter-suffix③"></a>[::suffix](#selectordef-first-letter-suffix) sub-elements) is similar to an <a id="ref-for-inline-box①"></a>[inline box](https://drafts.csswg.org/css-display-4/#inline-box). The following properties apply to <a id="ref-for-selectordef-first-letter②②"></a>::first-letter, ::first-letter::prefix, and ::first-letter::suffix pseudo-elements:

- all font properties (see [\[CSS-FONTS-4\]](#biblio-css-fonts-4))
- the <a id="ref-for-propdef-color①"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) and <a id="ref-for-propdef-opacity①"></a>[opacity](https://drafts.csswg.org/css-color-4/#propdef-opacity) properties (see [\[CSS-COLOR-4\]](#biblio-css-color-4))
- all background properties (see [\[CSS-BACKGROUNDS-4\]](#biblio-css-backgrounds-4))
- any typesetting properties that apply to inline elements (see [\[CSS-TEXT-4\]](#biblio-css-text-4))
- all text decoration properties (see [\[CSS-TEXT-DECOR-4\]](#biblio-css-text-decor-4))
- any inline layout properties that apply to inline elements (see [\[CSS-INLINE-3\]](#biblio-css-inline-3))
- margin and padding properties (see [\[CSS22\]](#biblio-css22))
- border properties and <a id="ref-for-propdef-box-shadow"></a>[box-shadow](https://drafts.csswg.org/css-backgrounds-3/#propdef-box-shadow) (see [\[CSS-BACKGROUNDS-4\]](#biblio-css-backgrounds-4))
- any other properties defined to apply to <a id="ref-for-selectordef-first-letter②③"></a>[::first-letter](#selectordef-first-letter) by their respective specifications

User agents may apply other properties as well. However, in no case may the application of such unlisted properties to <a id="ref-for-selectordef-first-letter②④"></a>[::first-letter](#selectordef-first-letter) change what <a id="ref-for-first-letter-text①②"></a>[first-letter text](#first-letter-text) is represented by that <a id="ref-for-selectordef-first-letter②⑤"></a>::first-letter.

Note: In previous levels of CSS, user agents were allowed to choose a line height, width, and height based on the shape of the letter, to approximate font sizes; and to take the glyph outline into account when performing layout. The possibility of such loosely-defined magic has been intentionally removed, as it proved to be a poor solution for the intended use case (drop caps and raised caps), yet caused interoperability problems. See <a id="ref-for-propdef-initial-letter①"></a>[initial-letter](https://drafts.csswg.org/css-inline-3/#propdef-initial-letter) in [\[CSS-INLINE-3\]](#biblio-css-inline-3) for explicitly handling drop caps and raised caps.

## <a id="highlight-pseudos"></a>3.  Highlight Pseudo-elements[](#highlight-pseudos)

### <a id="highlight-selectors"></a>3.1.  Selecting Highlighted Content: the <a id="ref-for-selectordef-selection"></a>[::selection](#selectordef-selection), <a id="ref-for-selectordef-search-text"></a>[::search-text](#selectordef-search-text), <a id="ref-for-selectordef-target-text"></a>[::target-text](#selectordef-target-text), <a id="ref-for-selectordef-spelling-error"></a>[::spelling-error](#selectordef-spelling-error), <a id="ref-for-selectordef-grammar-error"></a>[::grammar-error](#selectordef-grammar-error), and <a id="ref-for-selectordef-highlight-highlight-name-selector"></a>[::highlight()](#selectordef-highlight-highlight-name-selector) pseudo-elements[](#highlight-selectors)

Tests

- [grammar-spelling-errors-001.html](https://wpt.fyi/results/css/css-pseudo/grammar-spelling-errors-001.html) [(live test)](http://wpt.live/css/css-pseudo/grammar-spelling-errors-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/grammar-spelling-errors-001.html)
- [grammar-spelling-errors-002.html](https://wpt.fyi/results/css/css-pseudo/grammar-spelling-errors-002.html) [(live test)](http://wpt.live/css/css-pseudo/grammar-spelling-errors-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/grammar-spelling-errors-002.html)
- [cascade-highlight-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/cascade-highlight-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/cascade-highlight-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/cascade-highlight-001.html)
- [cascade-highlight-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/cascade-highlight-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/cascade-highlight-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/cascade-highlight-002.html)
- [cascade-highlight-004.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/cascade-highlight-004.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/cascade-highlight-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/cascade-highlight-004.html)
- [cascade-highlight-005.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/cascade-highlight-005.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/cascade-highlight-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/cascade-highlight-005.html)
- [highlight-cascade-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-cascade-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-cascade-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-cascade-001.html)
- [highlight-cascade-003.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-cascade-003.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-cascade-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-cascade-003.html)
- [highlight-cascade-004.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-cascade-004.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-cascade-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-cascade-004.html)
- [highlight-cascade-005.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-cascade-005.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-cascade-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-cascade-005.html)
- [highlight-cascade-006.xhtml](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-cascade-006.xhtml) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-cascade-006.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-cascade-006.xhtml)
- [highlight-cascade-007.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-cascade-007.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-cascade-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-cascade-007.html)
- [highlight-cascade-008.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-cascade-008.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-cascade-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-cascade-008.html)
- [highlight-cascade-009.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-cascade-009.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-cascade-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-cascade-009.html)
- [highlight-currentcolor-computed-inheritance.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-computed-inheritance.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-computed-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-computed-inheritance.html)
- [highlight-currentcolor-computed-visited.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-computed-visited.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-computed-visited.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-computed-visited.html)
- [highlight-currentcolor-computed.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-computed.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-computed.html)
- [highlight-currentcolor-painting-properties-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-properties-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-properties-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-properties-001.html)
- [highlight-currentcolor-painting-properties-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-properties-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-properties-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-properties-002.html)
- [highlight-currentcolor-painting-text-shadow-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-text-shadow-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-text-shadow-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-text-shadow-001.html)
- [highlight-currentcolor-painting-text-shadow-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-text-shadow-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-text-shadow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-painting-text-shadow-002.html)
- [highlight-currentcolor-root-explicit-default-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-explicit-default-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-explicit-default-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-explicit-default-001.html)
- [highlight-currentcolor-root-explicit-default-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-explicit-default-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-explicit-default-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-explicit-default-002.html)
- [highlight-currentcolor-root-implicit-default-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-implicit-default-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-implicit-default-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-implicit-default-001.html)
- [highlight-currentcolor-root-implicit-default-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-implicit-default-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-implicit-default-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-currentcolor-root-implicit-default-002.html)
- [highlight-pseudos-computed.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-pseudos-computed.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-pseudos-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-pseudos-computed.html)
- [highlight-pseudos-inheritance-computed-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-pseudos-inheritance-computed-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-pseudos-inheritance-computed-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-pseudos-inheritance-computed-001.html)
- [highlight-pseudos-visited-computed-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-pseudos-visited-computed-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-pseudos-visited-computed-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-pseudos-visited-computed-001.html)
- [highlight-custom-properties-dynamic-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-custom-properties-dynamic-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-custom-properties-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-custom-properties-dynamic-001.html)
- [highlight-painting-005-crash.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-005-crash.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-005-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-005-crash.html)
- [highlight-painting-currentcolor-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-001.html)
- [highlight-painting-currentcolor-001a.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-001a.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-001a.html)
- [highlight-painting-currentcolor-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-002.html)
- [highlight-painting-currentcolor-002a.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-002a.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-002a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-002a.html)
- [highlight-painting-currentcolor-002b.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-002b.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-002b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-002b.html)
- [highlight-painting-currentcolor-003.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-003.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-003.html)
- [highlight-painting-currentcolor-003a.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-003a.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-003a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-003a.html)
- [highlight-painting-currentcolor-003b.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-003b.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-003b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-003b.html)
- [highlight-painting-currentcolor-004.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-004.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-004.html)
- [highlight-painting-currentcolor-004a.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-004a.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-004a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-004a.html)
- [highlight-painting-currentcolor-004b.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-004b.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-004b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-004b.html)
- [highlight-painting-currentcolor-005.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-currentcolor-005.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-currentcolor-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-currentcolor-005.html)
- [highlight-painting-shadows-horizontal.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-shadows-horizontal.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-shadows-horizontal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-shadows-horizontal.html)
- [highlight-painting-shadows-vertical.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-shadows-vertical.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-shadows-vertical.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-shadows-vertical.html)
- [highlight-painting-soft-hyphens-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-soft-hyphens-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-soft-hyphens-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-soft-hyphens-001.html)
- [highlight-painting-soft-hyphens-002-crash.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-soft-hyphens-002-crash.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-soft-hyphens-002-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-soft-hyphens-002-crash.html)
- [highlight-styling-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-styling-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-styling-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-styling-001.html)
- [highlight-styling-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-styling-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-styling-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-styling-002.html)
- [highlight-pseudos.html](https://wpt.fyi/results/css/css-pseudo/parsing/highlight-pseudos.html) [(live test)](http://wpt.live/css/css-pseudo/parsing/highlight-pseudos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/parsing/highlight-pseudos.html)

The <a id="highlight-pseudo-element"></a><strong>highlight pseudo-elements</strong> represent portions of a document that have been given a particular status and are typically styled differently to indicate that status to the user. For example, selected portions of the document are typically highlighted (given alternate background and foreground colors, or a color wash) to indicate their selected status. The following <a id="ref-for-highlight-pseudo-element"></a>[highlight pseudo-elements](#highlight-pseudo-element) are defined:

<strong><a id="selectordef-selection"></a><strong>::selection</strong></strong>

The <a id="ref-for-selectordef-selection①"></a>[::selection](#selectordef-selection) <a id="ref-for-pseudo-element①⑥"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) represents the portion of a document that has been selected as the target or object of some possible future user-agent operation(s). It applies, for example, to selected text within an editable text field, which would be copied by a copy operation or replaced by a paste operation.

Tests

- active-selection-001-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-001-manual.html)
- active-selection-002-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-002-manual.html)
- active-selection-004-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-004-manual.html)
- [active-selection-011.html](https://wpt.fyi/results/css/css-pseudo/active-selection-011.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-011.html)
- [active-selection-012.html](https://wpt.fyi/results/css/css-pseudo/active-selection-012.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-012.html)
- [active-selection-014.html](https://wpt.fyi/results/css/css-pseudo/active-selection-014.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-014.html)
- [active-selection-016.html](https://wpt.fyi/results/css/css-pseudo/active-selection-016.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-016.html)
- [active-selection-018.html](https://wpt.fyi/results/css/css-pseudo/active-selection-018.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-018.html)
- [active-selection-025.html](https://wpt.fyi/results/css/css-pseudo/active-selection-025.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-025.html)
- [active-selection-027.html](https://wpt.fyi/results/css/css-pseudo/active-selection-027.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-027.html)
- [active-selection-056.html](https://wpt.fyi/results/css/css-pseudo/active-selection-056.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-056.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-056.html)
- [active-selection-057.html](https://wpt.fyi/results/css/css-pseudo/active-selection-057.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-057.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-057.html)
- [active-selection-063.html](https://wpt.fyi/results/css/css-pseudo/active-selection-063.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-063.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-063.html)
- [selection-background-color-001.html](https://wpt.fyi/results/css/css-pseudo/selection-background-color-001.html) [(live test)](http://wpt.live/css/css-pseudo/selection-background-color-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-background-color-001.html)
- [selection-link-001.html](https://wpt.fyi/results/css/css-pseudo/selection-link-001.html) [(live test)](http://wpt.live/css/css-pseudo/selection-link-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-link-001.html)
- [selection-link-002.html](https://wpt.fyi/results/css/css-pseudo/selection-link-002.html) [(live test)](http://wpt.live/css/css-pseudo/selection-link-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-link-002.html)
- [selection-link-003.html](https://wpt.fyi/results/css/css-pseudo/selection-link-003.html) [(live test)](http://wpt.live/css/css-pseudo/selection-link-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-link-003.html)
- [selection-over-highlight-001.html](https://wpt.fyi/results/css/css-pseudo/selection-over-highlight-001.html) [(live test)](http://wpt.live/css/css-pseudo/selection-over-highlight-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-over-highlight-001.html)
- [selection-universal-shadow-dom.html](https://wpt.fyi/results/css/css-pseudo/selection-universal-shadow-dom.html) [(live test)](http://wpt.live/css/css-pseudo/selection-universal-shadow-dom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-universal-shadow-dom.html)
- [selection-contenteditable-011.html](https://wpt.fyi/results/css/css-pseudo/selection-contenteditable-011.html) [(live test)](http://wpt.live/css/css-pseudo/selection-contenteditable-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-contenteditable-011.html)
- [selection-input-011.html](https://wpt.fyi/results/css/css-pseudo/selection-input-011.html) [(live test)](http://wpt.live/css/css-pseudo/selection-input-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-input-011.html)
- [selection-textarea-011.html](https://wpt.fyi/results/css/css-pseudo/selection-textarea-011.html) [(live test)](http://wpt.live/css/css-pseudo/selection-textarea-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-textarea-011.html)
- [svg-text-selection-fill-only.html](https://wpt.fyi/results/css/css-pseudo/svg-text-selection-fill-only.html) [(live test)](http://wpt.live/css/css-pseudo/svg-text-selection-fill-only.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/svg-text-selection-fill-only.html)
- [svg-text-selection-shadow.html](https://wpt.fyi/results/css/css-pseudo/svg-text-selection-shadow.html) [(live test)](http://wpt.live/css/css-pseudo/svg-text-selection-shadow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/svg-text-selection-shadow.html)
- [svg-text-selection-stroke-only.html](https://wpt.fyi/results/css/css-pseudo/svg-text-selection-stroke-only.html) [(live test)](http://wpt.live/css/css-pseudo/svg-text-selection-stroke-only.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/svg-text-selection-stroke-only.html)
- [svg-text-selection-transparent-background.html](https://wpt.fyi/results/css/css-pseudo/svg-text-selection-transparent-background.html) [(live test)](http://wpt.live/css/css-pseudo/svg-text-selection-transparent-background.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/svg-text-selection-transparent-background.html)
- [textpath-selection-011.html](https://wpt.fyi/results/css/css-pseudo/textpath-selection-011.html) [(live test)](http://wpt.live/css/css-pseudo/textpath-selection-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/textpath-selection-011.html)

<strong><a id="selectordef-search-text"></a><strong>::search-text</strong></strong>

The <a id="ref-for-selectordef-search-text①"></a>[::search-text](#selectordef-search-text) <a id="ref-for-pseudo-element①⑦"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) represents text identified by the user agent’s find-in-page feature. Since not all UAs style matched text in ways expressible with the <a id="ref-for-highlight-pseudo-element①"></a>[highlight pseudo-elements](#highlight-pseudo-element), this pseudo-element is optional to implement.

The <a id="ref-for-current-pseudo"></a>[:current](https://drafts.csswg.org/selectors-5/#current-pseudo) <a id="ref-for-pseudo-class"></a>[pseudo-class](https://drafts.csswg.org/selectors-4/#pseudo-class) (but not ::current()) may be combined with <a id="ref-for-selectordef-search-text②"></a>[::search-text](#selectordef-search-text) to represent the currently focused match instance. The <a id="ref-for-past-pseudo"></a>[:past](https://drafts.csswg.org/selectors-5/#past-pseudo) and <a id="ref-for-future-pseudo"></a>[:future](https://drafts.csswg.org/selectors-5/#future-pseudo) <a id="ref-for-x23"></a>[pseudo-classes](https://www.w3.org/TR/CSS2/selector.html#x23) are reserved for analogous use in the future. Any unsupported combination of these pseudo-classes with <a id="ref-for-selectordef-search-text③"></a>::search-text <em>must</em> be treated as invalid.

<strong><a id="selectordef-target-text"></a><strong>::target-text</strong></strong>

The <a id="ref-for-selectordef-target-text①"></a>[::target-text](#selectordef-target-text) <a id="ref-for-pseudo-element①⑧"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) represents text directly targeted by the document URL’s <a id="ref-for-concept-url-fragment"></a>[fragment](https://url.spec.whatwg.org/#concept-url-fragment), if any.

Note: When a <a id="ref-for-concept-url-fragment①"></a>[URL fragment](https://url.spec.whatwg.org/#concept-url-fragment) targets an element, the <a id="ref-for-target-pseudo"></a>[:target](https://drafts.csswg.org/selectors-4/#target-pseudo) pseudo-element can be used to select it, but <a id="ref-for-selectordef-target-text②"></a>[::target-text](#selectordef-target-text) does not match anything. It only matches text that is itself targeted by the \[<a id="ref-for-concept-url-fragment②"></a>fragment\].

Tests

- [target-text-001.html](https://wpt.fyi/results/css/css-pseudo/target-text-001.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-001.html)
- [target-text-002.html](https://wpt.fyi/results/css/css-pseudo/target-text-002.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-002.html)
- [target-text-003.html](https://wpt.fyi/results/css/css-pseudo/target-text-003.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-003.html)
- [target-text-004.html](https://wpt.fyi/results/css/css-pseudo/target-text-004.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-004.html)
- [target-text-005.html](https://wpt.fyi/results/css/css-pseudo/target-text-005.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-005.html)
- [target-text-006.html](https://wpt.fyi/results/css/css-pseudo/target-text-006.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-006.html)
- [target-text-007.html](https://wpt.fyi/results/css/css-pseudo/target-text-007.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-007.html)
- [target-text-008.html](https://wpt.fyi/results/css/css-pseudo/target-text-008.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-008.html)
- [target-text-009.html](https://wpt.fyi/results/css/css-pseudo/target-text-009.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-009.html)
- [target-text-010.html](https://wpt.fyi/results/css/css-pseudo/target-text-010.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-010.html)
- [target-text-dynamic-001.html](https://wpt.fyi/results/css/css-pseudo/target-text-dynamic-001.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-dynamic-001.html)
- [target-text-dynamic-002.html](https://wpt.fyi/results/css/css-pseudo/target-text-dynamic-002.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-dynamic-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-dynamic-002.html)
- [target-text-dynamic-003.html](https://wpt.fyi/results/css/css-pseudo/target-text-dynamic-003.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-dynamic-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-dynamic-003.html)
- [target-text-dynamic-004.html](https://wpt.fyi/results/css/css-pseudo/target-text-dynamic-004.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-dynamic-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-dynamic-004.html)
- [target-text-shadow-horizontal.html](https://wpt.fyi/results/css/css-pseudo/target-text-shadow-horizontal.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-shadow-horizontal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-shadow-horizontal.html)
- [target-text-shadow-vertical.html](https://wpt.fyi/results/css/css-pseudo/target-text-shadow-vertical.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-shadow-vertical.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-shadow-vertical.html)
- [target-text-text-decoration-001.html](https://wpt.fyi/results/css/css-pseudo/target-text-text-decoration-001.html) [(live test)](http://wpt.live/css/css-pseudo/target-text-text-decoration-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/target-text-text-decoration-001.html)

<strong><a id="selectordef-spelling-error"></a><strong>::spelling-error</strong></strong>

The <a id="ref-for-selectordef-spelling-error①"></a>[::spelling-error](#selectordef-spelling-error) <a id="ref-for-pseudo-element①⑨"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) represents a portion of text that has been flagged by the user agent as misspelled.

Tests

- [spelling-error-001.html](https://wpt.fyi/results/css/css-pseudo/spelling-error-001.html) [(live test)](http://wpt.live/css/css-pseudo/spelling-error-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/spelling-error-001.html)
- spelling-error-002-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/spelling-error-002-manual.html)
- spelling-error-003-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/spelling-error-003-manual.html)
- [spelling-error-004-crash.html](https://wpt.fyi/results/css/css-pseudo/spelling-error-004-crash.html) [(live test)](http://wpt.live/css/css-pseudo/spelling-error-004-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/spelling-error-004-crash.html)
- [spelling-error-005-crash.html](https://wpt.fyi/results/css/css-pseudo/spelling-error-005-crash.html) [(live test)](http://wpt.live/css/css-pseudo/spelling-error-005-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/spelling-error-005-crash.html)
- [spelling-error-006.html](https://wpt.fyi/results/css/css-pseudo/spelling-error-006.html) [(live test)](http://wpt.live/css/css-pseudo/spelling-error-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/spelling-error-006.html)

<strong><a id="selectordef-grammar-error"></a><strong>::grammar-error</strong></strong>

The <a id="ref-for-selectordef-grammar-error①"></a>[::grammar-error](#selectordef-grammar-error) <a id="ref-for-pseudo-element②⓪"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) represents a portion of text that has been flagged by the user agent as grammatically incorrect.

Tests

- [grammar-error-001.html](https://wpt.fyi/results/css/css-pseudo/grammar-error-001.html) [(live test)](http://wpt.live/css/css-pseudo/grammar-error-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/grammar-error-001.html)
- grammar-error-002-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/grammar-error-002-manual.html)
- grammar-error-003-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/grammar-error-003-manual.html)

<strong><a id="selectordef-highlight-highlight-name-selector"></a><strong>::highlight(<a id="ref-for-typedef-highlight-name-selector"></a>[\<highlight-name-selector\>](#typedef-highlight-name-selector))</strong></strong>

The <a id="ref-for-selectordef-highlight-highlight-name-selector①"></a>[::highlight()](#selectordef-highlight-highlight-name-selector) <a id="ref-for-functional-pseudo-element"></a>[functional pseudo-element](https://drafts.csswg.org/selectors-4/#functional-pseudo-element) represents the portion of the <a id="ref-for-originating-element⑥"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) associated with the <a id="ref-for-custom-highlight"></a>[custom highlights](https://drafts.csswg.org/css-highlight-api-1/#custom-highlight) selected by its argument:

<a id="typedef-highlight-name-selector"></a><a id="ref-for-comb-one"></a><a id="ref-for-identifier-value"></a>

``` text
<highlight-name-selector> = '*' | <custom-ident>
```

A <a id="ref-for-identifier-value①"></a>[\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) argument selects the single <a id="ref-for-custom-highlight①"></a>[custom highlight](https://drafts.csswg.org/css-highlight-api-1/#custom-highlight) whose <a id="ref-for-custom-highlight-name"></a>[custom highlight name](https://drafts.csswg.org/css-highlight-api-1/#custom-highlight-name) is that identifier. The <a id="ref-for-x"></a>[\*](https://drafts.csswg.org/selectors-3/#x) argument selects every <a id="ref-for-custom-highlight②"></a>custom highlight, regardless of its <a id="ref-for-custom-highlight-name①"></a>custom highlight name.

The specificity of a <a id="ref-for-selectordef-highlight-highlight-name-selector②"></a>[::highlight()](#selectordef-highlight-highlight-name-selector) selector with a <a id="ref-for-identifier-value②"></a>[\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) argument is equivalent to a <a id="ref-for-type-selector"></a>[type selector](https://drafts.csswg.org/selectors-4/#type-selector). The specificity of a <a id="ref-for-selectordef-highlight-highlight-name-selector③"></a>::highlight() selector with a <a id="ref-for-x①"></a>[\*](https://drafts.csswg.org/selectors-3/#x) argument is zero.

See [\[CSS-HIGHLIGHT-API-1\]](#biblio-css-highlight-api-1) for details.

The <a id="ref-for-highlight-pseudo-element②"></a>[highlight pseudo-elements](#highlight-pseudo-element) do not necessarily fit into the element tree, and can arbitrarily cross element boundaries without honoring its nesting structure.

### <a id="highlight-styling"></a>3.2.  Styling Highlights[](#highlight-styling)

The <a id="ref-for-highlight-pseudo-element③"></a>[highlight pseudo-elements](#highlight-pseudo-element) can only be styled by a limited set of properties that do not affect layout and can be applied performantly in a highly dynamic environment—​and additionally (to ensure interoperability) whose rendering within the [required area](#highlight-bounds) is not dependent on the exact (UA-determined) bounds of the <a id="ref-for-highlight-overlay"></a>[highlight overlay](#highlight-overlay).

The following properties apply to the <a id="ref-for-highlight-pseudo-element④"></a>[highlight pseudo-elements](#highlight-pseudo-element):

- <a id="ref-for-propdef-color②"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color)
  Tests

  - active-selection-001-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-001-manual.html)
  - [active-selection-011.html](https://wpt.fyi/results/css/css-pseudo/active-selection-011.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-011.html)
  - [active-selection-016.html](https://wpt.fyi/results/css/css-pseudo/active-selection-016.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-016.html)
  - [active-selection-018.html](https://wpt.fyi/results/css/css-pseudo/active-selection-018.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-018.html)
  - [highlight-styling-003.html](https://wpt.fyi/results/css/css-pseudo/highlight-styling-003.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-styling-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-styling-003.html)
  - [highlight-styling-004.html](https://wpt.fyi/results/css/css-pseudo/highlight-styling-004.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-styling-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-styling-004.html)
- <a id="ref-for-propdef-background-color"></a>[background-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color)
  Tests

  - active-selection-002-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-002-manual.html)
  - [active-selection-012.html](https://wpt.fyi/results/css/css-pseudo/active-selection-012.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-012.html)
  - [active-selection-031.html](https://wpt.fyi/results/css/css-pseudo/active-selection-031.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-031.html)
- <a id="ref-for-propdef-text-decoration"></a>[text-decoration](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration) and its associated properties (including <a id="ref-for-propdef-text-underline-position"></a>[text-underline-position](https://drafts.csswg.org/css-text-decor-4/#propdef-text-underline-position) and <a id="ref-for-propdef-text-underline-offset"></a>[text-underline-offset](https://drafts.csswg.org/css-text-decor-4/#propdef-text-underline-offset))
  Tests

  - active-selection-004-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-004-manual.html)
  - [active-selection-014.html](https://wpt.fyi/results/css/css-pseudo/active-selection-014.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-014.html)
  - [active-selection-021.html](https://wpt.fyi/results/css/css-pseudo/active-selection-021.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-021.html)
  - [grammar-error-001.html](https://wpt.fyi/results/css/css-pseudo/grammar-error-001.html) [(live test)](http://wpt.live/css/css-pseudo/grammar-error-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/grammar-error-001.html)
  - grammar-error-002-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/grammar-error-002-manual.html)
  - grammar-error-003-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/grammar-error-003-manual.html)
  - [spelling-error-001.html](https://wpt.fyi/results/css/css-pseudo/spelling-error-001.html) [(live test)](http://wpt.live/css/css-pseudo/spelling-error-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/spelling-error-001.html)
  - spelling-error-002-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/spelling-error-002-manual.html)
  - spelling-error-003-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/spelling-error-003-manual.html)
- <a id="ref-for-propdef-text-shadow"></a>[text-shadow](https://drafts.csswg.org/css-text-decor-4/#propdef-text-shadow)
  Tests

  - [marker-text-shadow.html](https://wpt.fyi/results/css/css-pseudo/marker-text-shadow.html) [(live test)](http://wpt.live/css/css-pseudo/marker-text-shadow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-text-shadow.html)
- <a id="ref-for-propdef-stroke-color"></a>[stroke-color](https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-color), <a id="ref-for-propdef-fill-color"></a>[fill-color](https://drafts.csswg.org/fill-stroke-3/#propdef-fill-color), and <a id="ref-for-propdef-stroke-width"></a>[stroke-width](https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-width)
  Tests

  - [highlight-styling-003.html](https://wpt.fyi/results/css/css-pseudo/highlight-styling-003.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-styling-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-styling-003.html)
  - [textpath-selection-011.html](https://wpt.fyi/results/css/css-pseudo/textpath-selection-011.html) [(live test)](http://wpt.live/css/css-pseudo/textpath-selection-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/textpath-selection-011.html)
- <a id="ref-for-custom-property①"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) [\[CSS-VARIABLES-1\]](#biblio-css-variables-1)
  Tests

  - [highlight-styling-005.html](https://wpt.fyi/results/css/css-pseudo/highlight-styling-005.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-styling-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-styling-005.html)

<a id="issue-6ca196f5"></a>

<strong>Issue:</strong>

[](#issue-6ca196f5) Are there any other properties that should be included here?

Note: Historically (and at the time of writing) only <a id="ref-for-propdef-color③"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) and <a id="ref-for-propdef-background-color①"></a>[background-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color) have been interoperably supported.

Note: The <a id="ref-for-propdef-color④"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) property sets the color of both the text and all line decorations (underline, overline, line-through) and emphasis marks (<a id="ref-for-propdef-text-emphasis"></a>[text-emphasis](https://drafts.csswg.org/css-text-decor-4/#propdef-text-emphasis)) applied to the text by the <a id="ref-for-originating-element⑦"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) and its ancestors and descendants.

For any properties not listed above, but which are required to resolve the values of applicable properties, their <a id="ref-for-computed-value"></a>[computed values](https://drafts.csswg.org/css-cascade-5/#computed-value) are copied from those of the <a id="ref-for-originating-element⑧"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element), ignoring any values specified on the <a id="ref-for-highlight-pseudo-element⑤"></a>[highlight pseudo-element](#highlight-pseudo-element) itself. For example:

- <a id="ref-for-propdef-forced-color-adjust"></a>[forced-color-adjust](https://drafts.csswg.org/css-color-adjust-1/#propdef-forced-color-adjust) (used in <a id="ref-for-forced-colors-mode"></a>[forced colors mode](https://drafts.csswg.org/css-color-adjust-1/#forced-colors-mode) to resolve colors) and <a id="ref-for-propdef-color-scheme"></a>[color-scheme](https://drafts.csswg.org/css-color-adjust-1/#propdef-color-scheme) (used to resolve <a id="ref-for-typedef-system-color"></a>[\<system-color\>](https://drafts.csswg.org/css-color-4/#typedef-system-color) values)
- <a id="ref-for-propdef-font-size"></a>[font-size](https://drafts.csswg.org/css-fonts-4/#propdef-font-size), <a id="ref-for-propdef-font-family"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family), etc. (used to resolve <a id="ref-for-em"></a>[em](https://drafts.csswg.org/css-values-4/#em) and other <a id="ref-for-font-relative-length"></a>[font-relative lengths](https://drafts.csswg.org/css-values-4/#font-relative-length)).
- <a id="ref-for-propdef-line-height①"></a>[line-height](https://drafts.csswg.org/css2/#propdef-line-height) (used to resolve <a id="ref-for-lh"></a>[lh](https://drafts.csswg.org/css-values-4/#lh) units)
- <a id="ref-for-custom-property②"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) [\[CSS-VARIABLES-1\]](#biblio-css-variables-1) (used in <a id="ref-for-funcdef-var"></a>[var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) substitutions)

If, for a given <a id="ref-for-highlight-pseudo-element⑥"></a>[highlight pseudo-element](#highlight-pseudo-element), there are colors specified in the <a id="ref-for-cascade-origin-author"></a>[author origin](https://drafts.csswg.org/css-cascade-5/#cascade-origin-author), those colors must be respected as specified; i.e. the UA must not apply any extra processing (such as using semi-transparent washes). However if there are no colors in the <a id="ref-for-cascade-origin-author①"></a>author origin, the UA may apply additional color processing.

Note: This is to ensure that color contrast is preserved across all user agents interpreting a given author stylesheet.

Vendor-prefixed properties such as <a id="ref-for-propdef--webkit-text-fill-color"></a>[-webkit-text-fill-color](https://compat.spec.whatwg.org/#propdef--webkit-text-fill-color) are not applicable to the <a id="ref-for-highlight-pseudo-element⑦"></a>[highlight pseudo-elements](#highlight-pseudo-element).

### <a id="highlight-ua-styles"></a>3.3.  Default UA Styles[](#highlight-ua-styles)

The following additions are recommended for the default UA stylesheet:

``` text
/* Represent default spelling/grammar error styling in an adjustable way */
:root::spelling-error { text-decoration: spelling-error; }
:root::grammar-error  { text-decoration: grammar-error; }
```

Some <a id="ref-for-highlight-pseudo-element⑧"></a>[highlight pseudo-elements](#highlight-pseudo-element) should have <a id="paired-default-highlight-colors"></a><strong>paired default highlight colors</strong>—​a default <a id="ref-for-propdef-color⑤"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) and <a id="ref-for-propdef-background-color②"></a>[background-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color) provided by the UA that are either used or overridden together, see [§ 3.3.1 Paired Defaults](#paired-defaults). For <a id="ref-for-selectordef-selection②"></a>[::selection](#selectordef-selection) they should correspond to <a id="ref-for-valdef-color-highlighttext"></a>[HighlightText](https://drafts.csswg.org/css-color-4/#valdef-color-highlighttext) and <a id="ref-for-valdef-color-highlight"></a>[Highlight](https://drafts.csswg.org/css-color-4/#valdef-color-highlight), while for <a id="ref-for-selectordef-target-text③"></a>[::target-text](#selectordef-target-text) they should correspond to <a id="ref-for-valdef-color-marktext"></a>[MarkText](https://drafts.csswg.org/css-color-4/#valdef-color-marktext) and <a id="ref-for-valdef-color-mark"></a>[Mark](https://drafts.csswg.org/css-color-4/#valdef-color-mark).

UAs may apply additional effects to enhance the presentation of highlighted content, for example dimming content other than the highlighted text or transitioning out a highlight style based on user interactions or timing. These are not controlled by CSS.

<a id="issue-c10df75c"></a>

<strong>Issue:</strong>

[](#issue-c10df75c) UA tweaks to the presentation of highlights in ways that <em>are</em> controlled by CSS are currently under discussion in [Issue 6853](https://github.com/w3c/csswg-drafts/issues/6853).

#### <a id="paired-defaults"></a>3.3.1.  Paired Defaults[](#paired-defaults)

For compatibility reasons, <a id="ref-for-paired-default-highlight-colors"></a>[paired default highlight colors](#paired-default-highlight-colors) must only be <a id="ref-for-used-value"></a>[used](https://drafts.csswg.org/css-cascade-5/#used-value) when neither <a id="ref-for-propdef-color⑥"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) nor <a id="ref-for-propdef-background-color③"></a>[background-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color) yield a <a id="ref-for-cascaded-value"></a>[cascaded value](https://drafts.csswg.org/css-cascade-5/#cascaded-value) from the <a id="ref-for-cascade-origin-author②"></a>[author origin](https://drafts.csswg.org/css-cascade-5/#cascade-origin-author) (or inherit their value from the author origin). When a highlight color is <a id="ref-for-valdef-all-revert"></a>[revert](https://drafts.csswg.org/css-cascade-5/#valdef-all-revert) or <a id="ref-for-valdef-all-revert-layer"></a>[revert-layer](https://drafts.csswg.org/css-cascade-5/#valdef-all-revert-layer), the origin <em>after</em> rolling back the cascade determines the <a id="ref-for-cascaded-value①"></a>cascaded value’s <a id="ref-for-origin"></a>[origin](https://drafts.csswg.org/css-cascade-6/#origin).

Note: Because this rule is for compatibility reasons, it does not apply to other similar properties like <a id="ref-for-propdef-fill-color①"></a>[fill-color](https://drafts.csswg.org/fill-stroke-3/#propdef-fill-color) or <a id="ref-for-propdef-stroke-color①"></a>[stroke-color](https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-color).

<a id="example-e46c5169"></a>

<strong>Example:</strong>

[](#example-e46c5169) For example, given the following markup:

``` text
<p>Highlight this <em>and this</em>.</p>
```

Any of the following rules would suppress the default <a id="ref-for-propdef-background-color④"></a>[background-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color) for <a id="ref-for-selectordef-selection③"></a>[::selection](#selectordef-selection) in the <code>&#60;em&#62;</code> element if given by the author:

``` text
em::selection { color: initial; }
em::selection { color: inherit; }
em::selection { color: unset; }
em::selection { color: green; }
p::selection { color: green; }
```

Tests

- [highlight-paired-cascade-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-paired-cascade-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-paired-cascade-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-paired-cascade-001.html)
- [highlight-paired-cascade-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-paired-cascade-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-paired-cascade-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-paired-cascade-002.html)
- [highlight-paired-cascade-003.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-paired-cascade-003.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-paired-cascade-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-paired-cascade-003.html)
- [highlight-paired-cascade-004.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-paired-cascade-004.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-paired-cascade-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-paired-cascade-004.html)
- [highlight-paired-cascade-005.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-paired-cascade-005.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-paired-cascade-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-paired-cascade-005.html)
- [highlight-paired-cascade-006.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-paired-cascade-006.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-paired-cascade-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-paired-cascade-006.html)

### <a id="highlight-bounds"></a>3.4.  Area of a Highlight[](#highlight-bounds)

For each type of highlighting (see [§ 3.1 Selecting Highlighted Content: the ::selection, ::search-text, ::target-text, ::spelling-error, ::grammar-error, and ::highlight() pseudo-elements](#highlight-selectors)) there exists a single <a id="highlight-overlay"></a><strong>highlight overlay</strong> for the entire document, the active portions of which are represented by the corresponding <a id="ref-for-highlight-pseudo-element⑨"></a>[highlight pseudo-element](#highlight-pseudo-element). Each box owns the piece of the overlay corresponding to any text or replaced content directly contained by the box.

- For text, the corresponding overlay must cover at least the entire em box and may extend further above/below the em box to the line box edges. Spacing between two characters may also be part of the overlay area, in which case it belongs to the innermost element that contains both characters and is selected when both characters are selected.
  Tests

  - [selection-intercharacter-011.html](https://wpt.fyi/results/css/css-pseudo/selection-intercharacter-011.html) [(live test)](http://wpt.live/css/css-pseudo/selection-intercharacter-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-intercharacter-011.html)
  - [selection-intercharacter-012.html](https://wpt.fyi/results/css/css-pseudo/selection-intercharacter-012.html) [(live test)](http://wpt.live/css/css-pseudo/selection-intercharacter-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-intercharacter-012.html)
- For replaced content, the associated overlay must cover at least the entire replaced object, and may extend outward to include the element’s entire content box.
  Tests

  - [active-selection-043.html](https://wpt.fyi/results/css/css-pseudo/active-selection-043.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-043.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-043.html)
- The overlay may also include other areas within the border-box of an element; in this case, those areas belong to the innermost such element that contains the area.
- For an <a id="ref-for-inline-level-box①"></a>[inline-level box](https://drafts.csswg.org/css-display-4/#inline-level-box), the overlay may extend outside its border edges in the <a id="ref-for-block-axis"></a>[block axis](https://drafts.csswg.org/css-writing-modes-4/#block-axis) as far as the edges of its <a id="ref-for-line-box①"></a>[line box](https://drafts.csswg.org/css-inline-3/#line-box).

<a id="issue-13b80e20"></a>

<strong>Issue:</strong>

[](#issue-13b80e20) See [F2F minutes](http://lists.w3.org/Archives/Public/www-style/2008Nov/0022.html), [dbaron’s message](http://lists.w3.org/Archives/Public/www-style/2008Oct/0268.html), [Daniel’s thread](http://lists.w3.org/Archives/Public/www-style/2010May/0247.html), [Gecko notes](http://lists.w3.org/Archives/Public/www-style/2010May/0261.html), [Opera notes](http://lists.w3.org/Archives/Public/www-style/2010May/0366.html), [Webkit notes](http://lists.w3.org/Archives/Public/www-style/2010May/0280.html)

<a id="issue-271a1b90"></a>

<strong>Issue:</strong>

[](#issue-271a1b90) Not sure if this is the correct way of describing the way things work.

### <a id="highlight-cascade"></a>3.5.  Cascading and Per-Element Highlight Styles[](#highlight-cascade)

Each element draws its own active portions of the <a id="ref-for-highlight-overlay①"></a>[highlight overlays](#highlight-overlay), which receives the styles specified by the corresponding <a id="ref-for-highlight-pseudo-element①⓪"></a>[highlight pseudo-element](#highlight-pseudo-element) styles for which that element is the <a id="ref-for-originating-element⑨"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element). When multiple styles conflict, the winning style is determined by the cascade.

When any supported property is not given a value by the cascade, or given a value of <a id="ref-for-valdef-all-inherit"></a>[inherit](https://drafts.csswg.org/css-cascade-5/#valdef-all-inherit) or <a id="ref-for-valdef-all-unset"></a>[unset](https://drafts.csswg.org/css-cascade-5/#valdef-all-unset), its <a id="ref-for-specified-value"></a>[specified value](https://drafts.csswg.org/css-cascade-5/#specified-value) is determined by inheritance from the corresponding <a id="ref-for-highlight-pseudo-element①①"></a>[highlight pseudo-element](#highlight-pseudo-element) of its <a id="ref-for-originating-element①⓪"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element)’s parent element. This occurs regardless of whether that property is an <a id="ref-for-inherited-property①"></a>[inherited property](https://drafts.csswg.org/css-cascade-5/#inherited-property).

Additionally, for <a id="ref-for-highlight-pseudo-element①②"></a>[highlight pseudo-elements](#highlight-pseudo-element) originating from the <a id="ref-for-root-element"></a>[root element](https://drafts.csswg.org/css-display-4/#root-element) the inherited value of <a id="ref-for-propdef-color⑦"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) is <a id="ref-for-valdef-color-currentcolor"></a>[currentColor](https://drafts.csswg.org/css-color-4/#valdef-color-currentcolor), not the <a id="ref-for-initial-value"></a>[initial value](https://drafts.csswg.org/css-cascade-5/#initial-value).

All <a id="ref-for-custom-property③"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) inherit from the <a id="ref-for-originating-element①①"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element), regardless of whether that property is a <a id="ref-for-custom-property④"></a>custom property that is registered to <a id="ref-for-css-inheritance③"></a>[inherit](https://drafts.csswg.org/css-cascade-5/#css-inheritance) or not.

Tests

- [active-selection-051.html](https://wpt.fyi/results/css/css-pseudo/active-selection-051.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-051.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-051.html)
- [active-selection-052.html](https://wpt.fyi/results/css/css-pseudo/active-selection-052.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-052.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-052.html)
- [active-selection-053.html](https://wpt.fyi/results/css/css-pseudo/active-selection-053.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-053.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-053.html)
- [active-selection-054.html](https://wpt.fyi/results/css/css-pseudo/active-selection-054.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-054.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-054.html)
- [cascade-highlight-004.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/cascade-highlight-004.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/cascade-highlight-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/cascade-highlight-004.html)
- [highlight-cascade-010.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-cascade-010.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-cascade-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-cascade-010.html)

<a id="example-c35bf49a"></a>

<strong>Example:</strong>

[](#example-c35bf49a) For example, if the following rules were applied:

``` text
p::selection      { color: yellow; background: green; }
p > em::selection { color: orange; }
em::selection     { color:    red; }
```

to the following markup:

``` text
<p>Highlight this <em>and this</em>.</p>
```

The selection highlight would be green throughout, with yellow text outside the <code>&#60;em&#62;</code> element and orange text inside it.

Tests

- [cascade-highlight-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/cascade-highlight-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/cascade-highlight-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/cascade-highlight-001.html)

Authors wanting multiple selections styles should use

<strong>Advisement:</strong>

<strong>:root::selection</strong> for their document-wide selection style, since this will allow clean overriding in descendants. <a id="ref-for-selectordef-selection④"></a>[::selection](#selectordef-selection) alone applies to every element in the tree, overriding the more specific styles of any ancestors.

<a id="example-97480f68"></a>

<strong>Example:</strong>

[](#example-97480f68) For example, if an author specified

``` text
::selection          { background: blue; }
p.warning::selection { background:  red; }
```

and the document included

``` text
<p class="warning">Some <strong>very important information</strong></p>
```

The highlight would be blue over “very important information” because the <code>&#60;strong&#62;</code> element´s <a id="ref-for-selectordef-selection⑤"></a>[::selection](#selectordef-selection) also matches the ::selection { background: blue; } rule. (Remember that <a id="ref-for-x②"></a>[\*](https://drafts.csswg.org/selectors-3/#x) is implied when a tag selector is missing.) The style rules that would give the intended behavior (red highlight within <code>p&#46;warning</code>, blue elsewhere) are

``` text
:root::selection     { background: blue; }
p.warning::selection { background:  red; }
```

Tests

- [cascade-highlight-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/cascade-highlight-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/cascade-highlight-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/cascade-highlight-002.html)

The following example demonstrates the inheritance of <a id="ref-for-custom-property⑤"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property).

<a id="example-6d7bbf04"></a>

<strong>Example:</strong>

[](#example-6d7bbf04) For example, if an author specified

``` text
:root {
   --background-color: green;
   --decoration-thickness: 10px;
   --decoration-color: purple;
 }
 ::selection {
   --decoration-thickness: 1px;
   --decoration-color: green;
 }
 div::selection {
   --decoration-color: blue;
   background-color: var(--background-color, red);
   text-decoration-line: underline;
   text-decoration-style: line;
   text-decoration-thickness: var(--decoration-thickness, 5px);
   text-decoration-color: var(--decoration-color, red);
 }
```

The universal <a id="ref-for-selectordef-selection⑥"></a>[::selection](#selectordef-selection) uses the user-agent’s default styling because it only defines custom properties, with no properties that influence the appearance. A div’s selection highlight would apply a green background to the selected content, with a 10px thick blue underline. Since --background-color and --decoration-thickness custom properties are not specified on the div::selection peudo-element, they are inherited from its originating <code><a id="ref-for-the-div-element"></a>[div](https://html&#46;spec&#46;whatwg&#46;org/multipage/grouping-content&#46;html#the-div-element)</code> element, which itself inherits the custom properties from the root. However since --decoration-color is specified on the div::selection itself, its value from the <a id="ref-for-originating-element①②"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) is not used.

Note: This behavior allows control of selection with custom properties in a way that is compatible with pre-existing implementations.

Tests

- [highlight-cascade-011.html](https://wpt.fyi/results/css/css-pseudo/highlight-cascade/highlight-cascade-011.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-cascade/highlight-cascade-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-cascade/highlight-cascade-011.html)

### <a id="highlight-painting"></a>3.6.  Painting the Highlight[](#highlight-painting)

Tests

- [active-selection-014.html](https://wpt.fyi/results/css/css-pseudo/active-selection-014.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-014.html)
- [active-selection-041.html](https://wpt.fyi/results/css/css-pseudo/active-selection-041.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-041.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-041.html)
- [active-selection-045.html](https://wpt.fyi/results/css/css-pseudo/active-selection-045.html) [(live test)](http://wpt.live/css/css-pseudo/active-selection-045.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/active-selection-045.html)
- [highlight-painting-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-001.html)
- [highlight-painting-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-002.html)
- [highlight-painting-003.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-003.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-003.html)
- [highlight-painting-004.html](https://wpt.fyi/results/css/css-pseudo/highlight-painting-004.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-painting-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-painting-004.html)

#### <a id="highlight-backgrounds"></a>3.6.1.  Backgrounds[](#highlight-backgrounds)

Each <a id="ref-for-highlight-pseudo-element①③"></a>[highlight pseudo-element](#highlight-pseudo-element) draws its background over the corresponding portion of the <a id="ref-for-highlight-overlay②"></a>[highlight overlay](#highlight-overlay), painting it immediately below any positioned descendants (i.e. just before step 8 in [CSS2.2§E.2](https://www.w3.org/TR/CSS22/zindex.html#painting-order)). The <a id="ref-for-selectordef-selection⑦"></a>[::selection](#selectordef-selection) overlay is drawn over the <a id="ref-for-selectordef-target-text④"></a>[::target-text](#selectordef-target-text) overlay which is drawn over the <a id="ref-for-selectordef-spelling-error②"></a>[::spelling-error](#selectordef-spelling-error) overlay which is drawn over the <a id="ref-for-selectordef-grammar-error②"></a>[::grammar-error](#selectordef-grammar-error) overlay which is drawn over the ::highlight overlays. The <a id="ref-for-selectordef-search-text④"></a>[::search-text](#selectordef-search-text) overlay is drawn directly over or below the :selection overlay depending on the UA, and drawn over all other overlays.

Tests

- [selection-overlay-and-grammar-001.html](https://wpt.fyi/results/css/css-pseudo/selection-overlay-and-grammar-001.html) [(live test)](http://wpt.live/css/css-pseudo/selection-overlay-and-grammar-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-overlay-and-grammar-001.html)
- [selection-overlay-and-spelling-001.html](https://wpt.fyi/results/css/css-pseudo/selection-overlay-and-spelling-001.html) [(live test)](http://wpt.live/css/css-pseudo/selection-overlay-and-spelling-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-overlay-and-spelling-001.html)
- [highlight-z-index-001.html](https://wpt.fyi/results/css/css-pseudo/highlight-z-index-001.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-z-index-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-z-index-001.html)
- [highlight-z-index-002.html](https://wpt.fyi/results/css/css-pseudo/highlight-z-index-002.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-z-index-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-z-index-002.html)
- [selection-background-painting-order.html](https://wpt.fyi/results/css/css-pseudo/selection-background-painting-order.html) [(live test)](http://wpt.live/css/css-pseudo/selection-background-painting-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-background-painting-order.html)

#### <a id="highlight-shadows"></a>3.6.2.  Shadows[](#highlight-shadows)

Any <a id="ref-for-propdef-text-shadow①"></a>[text-shadow](https://drafts.csswg.org/css-text-decor-4/#propdef-text-shadow) applying to a <a id="ref-for-highlight-pseudo-element①④"></a>[highlight pseudo-element](#highlight-pseudo-element) is drawn over its corresponding <a id="ref-for-highlight-overlay③"></a>[highlight overlay](#highlight-overlay) background. Such text shadows also stack over each other (and over any original <a id="ref-for-propdef-text-shadow②"></a>text-shadow applied to the text and its decorations, which continues to apply).

Note: Since each <a id="ref-for-highlight-overlay④"></a>[highlight overlay](#highlight-overlay) background is drawn over any shadows belonging to the layer(s) below, a <a id="ref-for-highlight-overlay⑤"></a>highlight overlay background can obscure lower-level shadows.

#### <a id="highlight-text"></a>3.6.3.  Text and Text Decorations[](#highlight-text)

A <a id="ref-for-highlight-pseudo-element①⑤"></a>[highlight pseudo-element](#highlight-pseudo-element) suppresses the normal drawing of any associated text, and the text decorations (other than shadows) that had been applied to that text. Instead the topmost active <a id="ref-for-highlight-overlay⑥"></a>[highlight overlay](#highlight-overlay) redraws that text (and those decorations) over all the <a id="ref-for-highlight-overlay⑦"></a>highlight overlay backgrounds using that highlight’s own <a id="ref-for-propdef-color⑧"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color).

Note: This means that unlike shadows, line decorations and emphasis marks won’t be obscured by any highlight overlay backgrounds that are drawn for the associated text.

For this purpose, <a id="ref-for-valdef-color-currentcolor①"></a>[currentColor](https://drafts.csswg.org/css-color-4/#valdef-color-currentcolor) on a <a id="ref-for-highlight-pseudo-element①⑥"></a>[highlight pseudo-element](#highlight-pseudo-element)’s <a id="ref-for-propdef-color⑨"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) property represents the <a id="ref-for-propdef-color①⓪"></a>color of the next <em>active</em> <a id="ref-for-highlight-pseudo-element①⑦"></a>highlight pseudo-element layer below, falling back finally to the colors that would otherwise have been used (those applied by the <a id="ref-for-originating-element①③"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) or an intervening <a id="ref-for-pseudo-element②①"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) such as <a id="ref-for-selectordef-first-line①⑨"></a>[::first-line](#selectordef-first-line) or <a id="ref-for-selectordef-first-letter②⑥"></a>[::first-letter](#selectordef-first-letter)).

Note: The element’s own text decorations (both [line decorations](https://www.w3.org/TR/css-text-decor/#line-decoration) and [emphasis marks](https://www.w3.org/TR/css-text-decor/#emphasis-marks)) are thus drawn in the pseudo-element’s own <a id="ref-for-propdef-color①①"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) when that is not <a id="ref-for-valdef-color-currentcolor②"></a>[currentColor](https://drafts.csswg.org/css-color-4/#valdef-color-currentcolor), regardless of their original color or fill specifications.

Any text decorations introduced by each <a id="ref-for-highlight-pseudo-element①⑧"></a>[highlight pseudo-element](#highlight-pseudo-element) are stacked in the same order as their backgrounds over the text’s original decorations and are all drawn, each decoration in its own color. The normal painting order applies, so as per [CSS2 Appendix E](https://www.w3.org/TR/CSS2/zindex.html#painting-order), all underlines are drawn below overlines which are drawn below the text which is drawn below any line-throughs. However, text decorations applied by <a id="ref-for-selectordef-selection⑧"></a>[::selection](#selectordef-selection) may instead all be drawn along with the text as a topmost set of layers above all other decorations.

<a id="example-92591876"></a>

<strong>Example:</strong>

[](#example-92591876) For example, assuming the original text has an underline and a strike-through, that <a id="ref-for-selectordef-selection⑨"></a>[::selection](#selectordef-selection) applies an underline and <a id="ref-for-selectordef-target-text⑤"></a>[::target-text](#selectordef-target-text) applies both overline and strike-through, the following are both conformant painting orders:

1.  original underline
2.  <a id="ref-for-selectordef-selection①⓪"></a>[::selection](#selectordef-selection) underline
3.  <a id="ref-for-selectordef-target-text⑥"></a>[::target-text](#selectordef-target-text) overline
4.  <a id="ref-for-selectordef-selection①①"></a>[::selection](#selectordef-selection)-colored text
5.  original strike-through
6.  <a id="ref-for-selectordef-target-text⑦"></a>[::target-text](#selectordef-target-text) strike-through

<!-- -->

1.  original underline
2.  <a id="ref-for-selectordef-target-text⑧"></a>[::target-text](#selectordef-target-text) overline
3.  original strike-through
4.  <a id="ref-for-selectordef-target-text⑨"></a>[::target-text](#selectordef-target-text) strike-through
5.  <a id="ref-for-selectordef-selection①②"></a>[::selection](#selectordef-selection) underline
6.  <a id="ref-for-selectordef-selection①③"></a>[::selection](#selectordef-selection)-colored text

[Line decorations](https://www.w3.org/TR/css-text-decor/#line-decoration) introduced by <a id="ref-for-highlight-pseudo-element①⑨"></a>[highlight pseudo-elements](#highlight-pseudo-element) apply only to the text associated with their <a id="ref-for-originating-element①④"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element), and are not propagated to descendants except via property <a id="ref-for-css-inheritance④"></a>[inheritance](https://drafts.csswg.org/css-cascade-5/#css-inheritance) (as described [above](#highlight-cascade)).

Note: Unlike the originating element’s own decorations, decorations declared on a highlight pseudo-element propagate to <a id="ref-for-out-of-flow①"></a>[out-of-flow elements](https://drafts.csswg.org/css-display-4/#out-of-flow) and <a id="ref-for-inline-block"></a>[inline blocks](https://drafts.csswg.org/css-display-4/#inline-block), with thickness and position varying between descendants.

Tests

- [selection-originating-underline-order.html](https://wpt.fyi/results/css/css-pseudo/selection-originating-underline-order.html) [(live test)](http://wpt.live/css/css-pseudo/selection-originating-underline-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-originating-underline-order.html)
- [selection-originating-decoration-color.html](https://wpt.fyi/results/css/css-pseudo/selection-originating-decoration-color.html) [(live test)](http://wpt.live/css/css-pseudo/selection-originating-decoration-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-originating-decoration-color.html)
- [selection-originating-strikethrough-order.html](https://wpt.fyi/results/css/css-pseudo/selection-originating-strikethrough-order.html) [(live test)](http://wpt.live/css/css-pseudo/selection-originating-strikethrough-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-originating-strikethrough-order.html)

#### <a id="highlight-replaced"></a>3.6.4.  Replaced Elements[](#highlight-replaced)

For non-replaced content, the UA must honor the <a id="ref-for-propdef-color①②"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) and <a id="ref-for-propdef-background-color⑤"></a>[background-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color) (including their alpha channels) as specified. However, for replaced content, the UA should create a semi-transparent wash to coat the content so that it can show through the selection. This wash should be of the specified <a id="ref-for-propdef-background-color⑥"></a>background-color if that is not <a id="ref-for-valdef-color-transparent"></a>[transparent](https://drafts.csswg.org/css-color-4/#valdef-color-transparent), else of the specified <a id="ref-for-propdef-color①③"></a>color; however the UA may adjust the alpha channel.

Tests

- [selection-paint-image.html](https://wpt.fyi/results/css/css-pseudo/selection-paint-image.html) [(live test)](http://wpt.live/css/css-pseudo/selection-paint-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/selection-paint-image.html)

### <a id="highlight-security"></a>3.7.  Security Considerations for Highlighting[](#highlight-security)

Because the styling of spelling and grammar errors can leak information about the contents of a user’s dictionary (which can include the user’s name and even includes the contents of their address book!) UAs that implement <a id="ref-for-selectordef-spelling-error③"></a>[::spelling-error](#selectordef-spelling-error) and <a id="ref-for-selectordef-grammar-error③"></a>[::grammar-error](#selectordef-grammar-error) must mitigate the following attacks:

- Pages attempting to directly read the styling of highlighted segments.
- Pages attempting to track performance impacts of styling highlighted segments via programatic modification of text fields.

## <a id="treelike"></a>4.  Tree-Abiding Pseudo-elements[](#treelike)

<a id="tree-abiding"></a><strong>Tree-abiding pseudo-elements</strong> always fit within the box tree. They <a id="ref-for-css-inheritance⑤"></a>[inherit](https://drafts.csswg.org/css-cascade-5/#css-inheritance) any inheritable properties from their <a id="ref-for-originating-element①⑤"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element); non-inheritable properties take their <a id="ref-for-initial-value①"></a>[initial values](https://drafts.csswg.org/css-cascade-5/#initial-value) as usual. [\[CSS-CASCADE-4\]](#biblio-css-cascade-4) A subset of these are the <a id="fully-styleable"></a><strong>fully styleable pseudo-elements</strong>; all properties that apply to a real element also apply to a <a id="ref-for-fully-styleable"></a>[fully styleable pseudo-element](#fully-styleable).

Tests

- [tree-abiding-pseudo-elements.html](https://wpt.fyi/results/css/css-pseudo/parsing/tree-abiding-pseudo-elements.html) [(live test)](http://wpt.live/css/css-pseudo/parsing/tree-abiding-pseudo-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/parsing/tree-abiding-pseudo-elements.html)

### <a id="generated-content"></a>4.1.  Generated Content Pseudo-elements: <a id="ref-for-selectordef-before①"></a>[::before](#selectordef-before) and <a id="ref-for-selectordef-after①"></a>[::after](#selectordef-after)[](#generated-content)

When their computed <a id="ref-for-propdef-content"></a>[content](https://drafts.csswg.org/css-content-3/#propdef-content) value is not <a id="ref-for-valdef-content-none"></a>[none](https://drafts.csswg.org/css-content-3/#valdef-content-none), these <a id="ref-for-pseudo-element②②"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) generate boxes as if they were immediate children of their <a id="ref-for-originating-element①⑥"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element), with content as specified by <a id="ref-for-propdef-content①"></a>content.

<strong><a id="selectordef-before"></a><strong>::before</strong></strong>

Represents a styleable child pseudo-element immediately before the <a id="ref-for-originating-element①⑦"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element)’s actual content.

<strong><a id="selectordef-after"></a><strong>::after</strong></strong>

Represents a styleable child pseudo-element immediately after the <a id="ref-for-originating-element①⑧"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element)’s actual content.

Tests

- [before-after-dynamic-custom-property-001.html](https://wpt.fyi/results/css/css-pseudo/before-after-dynamic-custom-property-001.html) [(live test)](http://wpt.live/css/css-pseudo/before-after-dynamic-custom-property-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/before-after-dynamic-custom-property-001.html)
- [before-as-flex-container.html](https://wpt.fyi/results/css/css-pseudo/before-as-flex-container.html) [(live test)](http://wpt.live/css/css-pseudo/before-as-flex-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/before-as-flex-container.html)
- [before-dynamic-display-none.html](https://wpt.fyi/results/css/css-pseudo/before-dynamic-display-none.html) [(live test)](http://wpt.live/css/css-pseudo/before-dynamic-display-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/before-dynamic-display-none.html)
- [before-in-display-none-thcrash.html](https://wpt.fyi/results/css/css-pseudo/before-in-display-none-thcrash.html) [(live test)](http://wpt.live/css/css-pseudo/before-in-display-none-thcrash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/before-in-display-none-thcrash.html)
- [before-preceding-whitespace-dynamic.html](https://wpt.fyi/results/css/css-pseudo/before-preceding-whitespace-dynamic.html) [(live test)](http://wpt.live/css/css-pseudo/before-preceding-whitespace-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/before-preceding-whitespace-dynamic.html)
- [focus-visible-originating-element.html](https://wpt.fyi/results/css/css-pseudo/focus-visible-originating-element.html) [(live test)](http://wpt.live/css/css-pseudo/focus-visible-originating-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/focus-visible-originating-element.html)

Both <a id="ref-for-selectordef-before②"></a>[::before](#selectordef-before) and <a id="ref-for-selectordef-after②"></a>[::after](#selectordef-after) are <a id="ref-for-fully-styleable①"></a>[fully styleable pseudo-elements](#fully-styleable): there is no restriction on what properties apply to them.

<a id="example-1db24d2c"></a>

<strong>Example:</strong>

[](#example-1db24d2c) For example, the following rule inserts the string “Note: ” before the content of every <code>&#60;p&#62;</code> element whose <code>class</code> attribute has the value <code>note</code>:

``` text
p.note::before { content: "Note: " }
```

Since the initial value of <a id="ref-for-propdef-display"></a>[display](https://drafts.csswg.org/css-display-4/#propdef-display) is <a id="ref-for-valdef-display-inline"></a>[inline](https://drafts.csswg.org/css-display-4/#valdef-display-inline), this will generate an inline box. Like other inline children of <code>&#60;p&#62;</code>, it will participate in <code>&#60;p&#62;</code>’s inline formatting context, potentially sharing a line with other content.

As with the content of regular elements, the generated content of <a id="ref-for-selectordef-before③"></a>[::before](#selectordef-before) and <a id="ref-for-selectordef-after③"></a>[:after](https://drafts.csswg.org/css2/#selectordef-after) pseudo-elements can form part of any <a id="ref-for-selectordef-first-line②⓪"></a>[::first-line](#selectordef-first-line) and <a id="ref-for-selectordef-first-letter②⑦"></a>[::first-letter](#selectordef-first-letter) pseudo-elements applied to its <a id="ref-for-originating-element①⑨"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element). Also as with regular child elements, the <a id="ref-for-selectordef-before④"></a>::before and <a id="ref-for-selectordef-after④"></a>[::after](#selectordef-after) pseudo-elements are suppressed when their parent, the <a id="ref-for-originating-element②⓪"></a>originating element, is <a id="ref-for-replaced-element"></a>[replaced](https://drafts.csswg.org/css-display-4/#replaced-element).

Tests

- [pseudo-replaced-elements.html](https://wpt.fyi/results/css/css-pseudo/pseudo-replaced-elements.html) [(live test)](http://wpt.live/css/css-pseudo/pseudo-replaced-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/pseudo-replaced-elements.html)

### <a id="marker-pseudo"></a>4.2.  List Markers: the <a id="ref-for-selectordef-marker"></a>[::marker](#selectordef-marker) pseudo-element[](#marker-pseudo)

The <a id="selectordef-marker"></a><strong>::marker</strong> <a id="ref-for-pseudo-element②③"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) represents the automatically generated <a id="ref-for-marker②"></a>[marker box](https://drafts.csswg.org/css-lists-3/#marker) of a <a id="ref-for-list-item①"></a>[list item](https://drafts.csswg.org/css-lists-3/#list-item). (See [\[CSS-DISPLAY-3\]](#biblio-css-display-3) and [\[CSS-LISTS-3\]](#biblio-css-lists-3).)

Tests

- [marker-and-other-pseudo-elements.html](https://wpt.fyi/results/css/css-pseudo/marker-and-other-pseudo-elements.html) [(live test)](http://wpt.live/css/css-pseudo/marker-and-other-pseudo-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-and-other-pseudo-elements.html)
- [marker-animate-002.html](https://wpt.fyi/results/css/css-pseudo/marker-animate-002.html) [(live test)](http://wpt.live/css/css-pseudo/marker-animate-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-animate-002.html)
- [marker-animate.html](https://wpt.fyi/results/css/css-pseudo/marker-animate.html) [(live test)](http://wpt.live/css/css-pseudo/marker-animate.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-animate.html)
- [marker-color.html](https://wpt.fyi/results/css/css-pseudo/marker-color.html) [(live test)](http://wpt.live/css/css-pseudo/marker-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-color.html)
- [marker-computed-content.html](https://wpt.fyi/results/css/css-pseudo/marker-computed-content.html) [(live test)](http://wpt.live/css/css-pseudo/marker-computed-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-computed-content.html)
- [marker-computed-size.html](https://wpt.fyi/results/css/css-pseudo/marker-computed-size.html) [(live test)](http://wpt.live/css/css-pseudo/marker-computed-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-computed-size.html)
- [marker-content-001.html](https://wpt.fyi/results/css/css-pseudo/marker-content-001.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-001.html)
- [marker-content-001b.html](https://wpt.fyi/results/css/css-pseudo/marker-content-001b.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-001b.html)
- [marker-content-001c.html](https://wpt.fyi/results/css/css-pseudo/marker-content-001c.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-001c.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-001c.html)
- [marker-content-002.html](https://wpt.fyi/results/css/css-pseudo/marker-content-002.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-002.html)
- [marker-content-003.html](https://wpt.fyi/results/css/css-pseudo/marker-content-003.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-003.html)
- [marker-content-003b.html](https://wpt.fyi/results/css/css-pseudo/marker-content-003b.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-003b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-003b.html)
- [marker-content-004.html](https://wpt.fyi/results/css/css-pseudo/marker-content-004.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-004.html)
- [marker-content-005.html](https://wpt.fyi/results/css/css-pseudo/marker-content-005.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-005.html)
- [marker-content-006.html](https://wpt.fyi/results/css/css-pseudo/marker-content-006.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-006.html)
- [marker-content-010.html](https://wpt.fyi/results/css/css-pseudo/marker-content-010.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-010.html)
- [marker-content-012.html](https://wpt.fyi/results/css/css-pseudo/marker-content-012.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-012.html)
- [marker-content-013.html](https://wpt.fyi/results/css/css-pseudo/marker-content-013.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-013.html)
- [marker-content-014.html](https://wpt.fyi/results/css/css-pseudo/marker-content-014.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-014.html)
- [marker-content-015.html](https://wpt.fyi/results/css/css-pseudo/marker-content-015.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-015.html)
- [marker-content-016.html](https://wpt.fyi/results/css/css-pseudo/marker-content-016.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-016.html)
- [marker-content-017.html](https://wpt.fyi/results/css/css-pseudo/marker-content-017.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-017.html)
- [marker-content-018.html](https://wpt.fyi/results/css/css-pseudo/marker-content-018.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-018.html)
- [marker-content-019.html](https://wpt.fyi/results/css/css-pseudo/marker-content-019.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-019.html)
- [marker-content-020.html](https://wpt.fyi/results/css/css-pseudo/marker-content-020.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-020.html)
- [marker-content-021.html](https://wpt.fyi/results/css/css-pseudo/marker-content-021.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-021.html)
- [marker-content-022.html](https://wpt.fyi/results/css/css-pseudo/marker-content-022.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-022.html)
- [marker-content-023.html](https://wpt.fyi/results/css/css-pseudo/marker-content-023.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-023.html)
- [marker-content-024.html](https://wpt.fyi/results/css/css-pseudo/marker-content-024.html) [(live test)](http://wpt.live/css/css-pseudo/marker-content-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-content-024.html)
- [marker-default-styles.html](https://wpt.fyi/results/css/css-pseudo/marker-default-styles.html) [(live test)](http://wpt.live/css/css-pseudo/marker-default-styles.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-default-styles.html)
- [marker-display-computed.html](https://wpt.fyi/results/css/css-pseudo/marker-display-computed.html) [(live test)](http://wpt.live/css/css-pseudo/marker-display-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-display-computed.html)
- [marker-display-dynamic-001.html](https://wpt.fyi/results/css/css-pseudo/marker-display-dynamic-001.html) [(live test)](http://wpt.live/css/css-pseudo/marker-display-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-display-dynamic-001.html)
- [marker-font-properties.html](https://wpt.fyi/results/css/css-pseudo/marker-font-properties.html) [(live test)](http://wpt.live/css/css-pseudo/marker-font-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-font-properties.html)
- [marker-font-variant-numeric-default-ref.html](https://wpt.fyi/results/css/css-pseudo/marker-font-variant-numeric-default-ref.html) [(live test)](http://wpt.live/css/css-pseudo/marker-font-variant-numeric-default-ref.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-font-variant-numeric-default-ref.html)
- [marker-font-variant-numeric-default.html](https://wpt.fyi/results/css/css-pseudo/marker-font-variant-numeric-default.html) [(live test)](http://wpt.live/css/css-pseudo/marker-font-variant-numeric-default.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-font-variant-numeric-default.html)
- [marker-font-variant-numeric-normal-ref.html](https://wpt.fyi/results/css/css-pseudo/marker-font-variant-numeric-normal-ref.html) [(live test)](http://wpt.live/css/css-pseudo/marker-font-variant-numeric-normal-ref.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-font-variant-numeric-normal-ref.html)
- [marker-font-variant-numeric-normal.html](https://wpt.fyi/results/css/css-pseudo/marker-font-variant-numeric-normal.html) [(live test)](http://wpt.live/css/css-pseudo/marker-font-variant-numeric-normal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-font-variant-numeric-normal.html)
- [marker-hit-testing.html](https://wpt.fyi/results/css/css-pseudo/marker-hit-testing.html) [(live test)](http://wpt.live/css/css-pseudo/marker-hit-testing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-hit-testing.html)
- [marker-hyphens.html](https://wpt.fyi/results/css/css-pseudo/marker-hyphens.html) [(live test)](http://wpt.live/css/css-pseudo/marker-hyphens.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-hyphens.html)
- [marker-inherit-line-height.html](https://wpt.fyi/results/css/css-pseudo/marker-inherit-line-height.html) [(live test)](http://wpt.live/css/css-pseudo/marker-inherit-line-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-inherit-line-height.html)
- [marker-inherit-values.html](https://wpt.fyi/results/css/css-pseudo/marker-inherit-values.html) [(live test)](http://wpt.live/css/css-pseudo/marker-inherit-values.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-inherit-values.html)
- [marker-intrinsic-contribution-001.html](https://wpt.fyi/results/css/css-pseudo/marker-intrinsic-contribution-001.html) [(live test)](http://wpt.live/css/css-pseudo/marker-intrinsic-contribution-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-intrinsic-contribution-001.html)
- [marker-intrinsic-contribution-002.html](https://wpt.fyi/results/css/css-pseudo/marker-intrinsic-contribution-002.html) [(live test)](http://wpt.live/css/css-pseudo/marker-intrinsic-contribution-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-intrinsic-contribution-002.html)
- [marker-letter-spacing.html](https://wpt.fyi/results/css/css-pseudo/marker-letter-spacing.html) [(live test)](http://wpt.live/css/css-pseudo/marker-letter-spacing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-letter-spacing.html)
- [marker-line-break.html](https://wpt.fyi/results/css/css-pseudo/marker-line-break.html) [(live test)](http://wpt.live/css/css-pseudo/marker-line-break.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-line-break.html)
- [marker-line-height.html](https://wpt.fyi/results/css/css-pseudo/marker-line-height.html) [(live test)](http://wpt.live/css/css-pseudo/marker-line-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-line-height.html)
- [marker-list-style-position.html](https://wpt.fyi/results/css/css-pseudo/marker-list-style-position.html) [(live test)](http://wpt.live/css/css-pseudo/marker-list-style-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-list-style-position.html)
- [marker-overflow-wrap.html](https://wpt.fyi/results/css/css-pseudo/marker-overflow-wrap.html) [(live test)](http://wpt.live/css/css-pseudo/marker-overflow-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-overflow-wrap.html)
- [marker-reverted-styles.html](https://wpt.fyi/results/css/css-pseudo/marker-reverted-styles.html) [(live test)](http://wpt.live/css/css-pseudo/marker-reverted-styles.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-reverted-styles.html)
- [marker-tab-size.html](https://wpt.fyi/results/css/css-pseudo/marker-tab-size.html) [(live test)](http://wpt.live/css/css-pseudo/marker-tab-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-tab-size.html)
- [marker-text-align-001.html](https://wpt.fyi/results/css/css-pseudo/marker-text-align-001.html) [(live test)](http://wpt.live/css/css-pseudo/marker-text-align-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-text-align-001.html)
- [marker-text-align-002.html](https://wpt.fyi/results/css/css-pseudo/marker-text-align-002.html) [(live test)](http://wpt.live/css/css-pseudo/marker-text-align-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-text-align-002.html)
- [marker-text-align-003.html](https://wpt.fyi/results/css/css-pseudo/marker-text-align-003.html) [(live test)](http://wpt.live/css/css-pseudo/marker-text-align-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-text-align-003.html)
- [marker-text-combine-upright.html](https://wpt.fyi/results/css/css-pseudo/marker-text-combine-upright.html) [(live test)](http://wpt.live/css/css-pseudo/marker-text-combine-upright.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-text-combine-upright.html)
- [marker-text-decoration-skip-ink.html](https://wpt.fyi/results/css/css-pseudo/marker-text-decoration-skip-ink.html) [(live test)](http://wpt.live/css/css-pseudo/marker-text-decoration-skip-ink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-text-decoration-skip-ink.html)
- [marker-text-emphasis.html](https://wpt.fyi/results/css/css-pseudo/marker-text-emphasis.html) [(live test)](http://wpt.live/css/css-pseudo/marker-text-emphasis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-text-emphasis.html)
- [marker-text-transform-default.html](https://wpt.fyi/results/css/css-pseudo/marker-text-transform-default.html) [(live test)](http://wpt.live/css/css-pseudo/marker-text-transform-default.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-text-transform-default.html)
- [marker-text-transform-dynamic.html](https://wpt.fyi/results/css/css-pseudo/marker-text-transform-dynamic.html) [(live test)](http://wpt.live/css/css-pseudo/marker-text-transform-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-text-transform-dynamic.html)
- [marker-text-transform-uppercase.html](https://wpt.fyi/results/css/css-pseudo/marker-text-transform-uppercase.html) [(live test)](http://wpt.live/css/css-pseudo/marker-text-transform-uppercase.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-text-transform-uppercase.html)
- [marker-unicode-bidi-default-ref.html](https://wpt.fyi/results/css/css-pseudo/marker-unicode-bidi-default-ref.html) [(live test)](http://wpt.live/css/css-pseudo/marker-unicode-bidi-default-ref.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-unicode-bidi-default-ref.html)
- [marker-unicode-bidi-default.html](https://wpt.fyi/results/css/css-pseudo/marker-unicode-bidi-default.html) [(live test)](http://wpt.live/css/css-pseudo/marker-unicode-bidi-default.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-unicode-bidi-default.html)
- [marker-unicode-bidi-normal-ref.html](https://wpt.fyi/results/css/css-pseudo/marker-unicode-bidi-normal-ref.html) [(live test)](http://wpt.live/css/css-pseudo/marker-unicode-bidi-normal-ref.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-unicode-bidi-normal-ref.html)
- [marker-unicode-bidi-normal.html](https://wpt.fyi/results/css/css-pseudo/marker-unicode-bidi-normal.html) [(live test)](http://wpt.live/css/css-pseudo/marker-unicode-bidi-normal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-unicode-bidi-normal.html)
- [marker-variable-computed-style.html](https://wpt.fyi/results/css/css-pseudo/marker-variable-computed-style.html) [(live test)](http://wpt.live/css/css-pseudo/marker-variable-computed-style.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-variable-computed-style.html)
- [marker-variable.html](https://wpt.fyi/results/css/css-pseudo/marker-variable.html) [(live test)](http://wpt.live/css/css-pseudo/marker-variable.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-variable.html)
- [marker-word-break.html](https://wpt.fyi/results/css/css-pseudo/marker-word-break.html) [(live test)](http://wpt.live/css/css-pseudo/marker-word-break.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-word-break.html)
- [marker-word-spacing.html](https://wpt.fyi/results/css/css-pseudo/marker-word-spacing.html) [(live test)](http://wpt.live/css/css-pseudo/marker-word-spacing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/marker-word-spacing.html)
- [outside-marker-paint-order.html](https://wpt.fyi/results/css/css-pseudo/outside-marker-paint-order.html) [(live test)](http://wpt.live/css/css-pseudo/outside-marker-paint-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/outside-marker-paint-order.html)
- [marker-supported-properties-in-animation.html](https://wpt.fyi/results/css/css-pseudo/parsing/marker-supported-properties-in-animation.html) [(live test)](http://wpt.live/css/css-pseudo/parsing/marker-supported-properties-in-animation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/parsing/marker-supported-properties-in-animation.html)
- [marker-supported-properties.html](https://wpt.fyi/results/css/css-pseudo/parsing/marker-supported-properties.html) [(live test)](http://wpt.live/css/css-pseudo/parsing/marker-supported-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/parsing/marker-supported-properties.html)

The contents of a <a id="ref-for-selectordef-marker①"></a>[::marker](#selectordef-marker) are ignored (not selected) by <a id="ref-for-selectordef-first-letter②⑧"></a>[::first-letter](#selectordef-first-letter).

<a id="issue-c80e7017"></a>

<strong>Issue:</strong>

[](#issue-c80e7017) Interaction of <a id="ref-for-selectordef-marker②"></a>[::marker](#selectordef-marker) and <a id="ref-for-selectordef-first-line②①"></a>[::first-line](#selectordef-first-line) is currently under discussion in [Issue 4506](https://github.com/w3c/csswg-drafts/issues/4506).

<a id="ref-for-selectordef-marker③"></a>[::marker](#selectordef-marker) is a <a id="ref-for-tree-abiding"></a>[tree-abiding pseudo-element](#tree-abiding), but is not <a id="ref-for-fully-styleable②"></a>[fully styleable](#fully-styleable); only a limited set of properties can be used on the <a id="ref-for-selectordef-marker④"></a>::marker pseudo-element. This list is defined in [CSS Lists 3 § 3.1.1 Properties Applying to ::marker](https://drafts.csswg.org/css-lists-3/#marker-properties).

The ::before::marker or ::after::marker selectors are valid and can be used to represent the <a id="ref-for-marker③"></a>[marker boxes](https://drafts.csswg.org/css-lists-3/#marker) of <a id="ref-for-selectordef-before⑤"></a>[::before](#selectordef-before) or <a id="ref-for-selectordef-after⑤"></a>[::after](#selectordef-after) pseudo-elements that happen to be <a id="ref-for-list-item②"></a>[list items](https://drafts.csswg.org/css-lists-3/#list-item). However ::marker::marker is invalid, and the computed value of <a id="ref-for-propdef-display①"></a>[display](https://drafts.csswg.org/css-display-4/#propdef-display) on <a id="ref-for-selectordef-marker⑤"></a>[::marker](#selectordef-marker) always loses any <a id="ref-for-valdef-display-list-item"></a>[list-item](https://drafts.csswg.org/css-display-4/#valdef-display-list-item) aspect.

<a id="issue-00545711"></a>

<strong>Issue:</strong>

[](#issue-00545711) Should <a id="ref-for-selectordef-marker⑥"></a>[::marker](#selectordef-marker) also have <a id="ref-for-selectordef-first-letter-prefix④"></a>[::prefix](#selectordef-first-letter-prefix) and <a id="ref-for-selectordef-first-letter-suffix④"></a>[::suffix](#selectordef-first-letter-suffix) sub-elements?

### <a id="placeholder-pseudo"></a>4.3.  Placeholder Input: the <a id="ref-for-selectordef-placeholder"></a>[::placeholder](#selectordef-placeholder) pseudo-element[](#placeholder-pseudo)

The <a id="selectordef-placeholder"></a><strong>::placeholder</strong> <a id="ref-for-pseudo-element②④"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) represents placeholder text in an input field: text that represents the input and provides a hint to the user on how to fill out the form. For example, a date-input field might have the placeholder text “YYYY/MM/DD” to clarify that numeric dates are to be entered in year-month-day order. It is a <a id="ref-for-tree-abiding①"></a>[tree-abiding pseudo-element](#tree-abiding).

Tests

- [placeholder-as-multicol-crash.html](https://wpt.fyi/results/css/css-pseudo/placeholder-as-multicol-crash.html) [(live test)](http://wpt.live/css/css-pseudo/placeholder-as-multicol-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/placeholder-as-multicol-crash.html)
- [placeholder-excluded-properties.html](https://wpt.fyi/results/css/css-pseudo/placeholder-excluded-properties.html) [(live test)](http://wpt.live/css/css-pseudo/placeholder-excluded-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/placeholder-excluded-properties.html)
- [placeholder-inherit.html](https://wpt.fyi/results/css/css-pseudo/placeholder-inherit.html) [(live test)](http://wpt.live/css/css-pseudo/placeholder-inherit.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/placeholder-inherit.html)
- [placeholder-input-dynamic-crash.html](https://wpt.fyi/results/css/css-pseudo/placeholder-input-dynamic-crash.html) [(live test)](http://wpt.live/css/css-pseudo/placeholder-input-dynamic-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/placeholder-input-dynamic-crash.html)
- [placeholder-input-number.html](https://wpt.fyi/results/css/css-pseudo/placeholder-input-number.html) [(live test)](http://wpt.live/css/css-pseudo/placeholder-input-number.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/placeholder-input-number.html)

<a id="example-60211a7d"></a>

<strong>Example:</strong>

[](#example-60211a7d) For example, according to the semantics of [\[HTML\]](#biblio-html) the <code><a id="ref-for-attr-input-placeholder"></a>[placeholder](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#attr-input-placeholder)</code> attribute on the <code><a id="ref-for-the-input-element"></a>[input](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#the-input-element)</code> and <code><a id="ref-for-the-textarea-element"></a>[textarea](https://html&#46;spec&#46;whatwg&#46;org/multipage/form-elements&#46;html#the-textarea-element)</code> elements provide placeholder text. The <a id="ref-for-selectordef-placeholder①"></a>[::placeholder](#selectordef-placeholder) <a id="ref-for-pseudo-element②⑤"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) represents such text when it is displayed.

Note: There also exists a <a id="ref-for-placeholder-shown-pseudo"></a>[:placeholder-shown](https://drafts.csswg.org/selectors-4/#placeholder-shown-pseudo) pseudo-<em>class</em>, which applies to (real) elements while they are showing placeholder text, and can be used to style such elements specially. <a id="ref-for-selectordef-placeholder②"></a>[::placeholder](#selectordef-placeholder) specifically represents the placeholder <em>text</em>, and is thus relatively limited in its abilities.

All properties that apply to the <a id="ref-for-selectordef-first-line②②"></a>[::first-line](#selectordef-first-line) pseudo-element also apply to the <a id="ref-for-selectordef-placeholder③"></a>[::placeholder](#selectordef-placeholder) pseudo-element, except those defined in [\[CSS-INLINE-3\]](#biblio-css-inline-3).

In interactive media, placeholder text is often hidden once the user has entered input; however this is not a requirement, and both the input value and the placeholder text may be visible simultaneously. The exact behavior is UA-defined. Note that in static media (such as print) placeholder text will be present even after the user has entered input.

<a id="issue-1631f2fa"></a>

<strong>Issue:</strong>

[](#issue-1631f2fa) Authors seem to want <a id="ref-for-propdef-text-align"></a>[text-align](https://drafts.csswg.org/css-text-3/#propdef-text-align) on the list of supported properties. See e.g. [comments here](http://css-tricks.com/almanac/selectors/p/placeholder/).

Note: It’s been requested that <a id="ref-for-selectordef-placeholder④"></a>[::placeholder](#selectordef-placeholder) also refer to a placeholder which has a corresponding element in the element tree. It’s not clear how this should work, but it may be worth doing. See [Issue 2417](https://github.com/w3c/csswg-drafts/issues/2517).

## <a id="element-like"></a>5. <a id="part-like"></a> Element-backed Pseudo-Elements[](#element-like)

The <a id="element-backed"></a><strong>element-backed pseudo-elements</strong>, interact with most CSS and other platform features as if they were real elements (and, in fact, often <em>are</em> real elements that are not otherwise selectable).

Element-backed pseudo-elements are always <a id="ref-for-tree-abiding②"></a>[tree-abiding](#tree-abiding). Unless otherwise specified, they are <a id="ref-for-fully-styleable③"></a>[fully styleable](#fully-styleable) and inherit from their <a id="ref-for-originating-element②①"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element), just like standard <a id="ref-for-tree-abiding③"></a>tree-abiding pseudo-elements; but they can be defined to inherit from another element instead. (For example, <a id="ref-for-selectordef-part"></a>[::part()](https://drafts.csswg.org/css-shadow-1/#selectordef-part) inherits from the parent of the element it represents in the shadow tree.)

All <a id="ref-for-x23①"></a>[pseudo-classes](https://www.w3.org/TR/CSS2/selector.html#x23) and <a id="ref-for-pseudo-element②⑥"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) are syntactically allowed after an <a id="ref-for-element-backed"></a>[element-backed pseudo-element](#element-backed) (such as x-button::part(label):hover or x-button::part(label)::before), just as if the pseudo-element were a <a id="ref-for-type-selector①"></a>[type selector](https://drafts.csswg.org/selectors-4/#type-selector); but some are disallowed from matching:

- The <a id="ref-for-structural-pseudo-classes"></a>[structural pseudo-classes](https://drafts.csswg.org/selectors-4/#structural-pseudo-classes), <a id="ref-for-has-pseudo"></a>[:has()](https://drafts.csswg.org/selectors-4/#has-pseudo) pseudo-class, <a id="ref-for-scope-pseudo"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo) pseudo-class, and <a id="ref-for-selectordef-host"></a>[:host](https://drafts.csswg.org/css-shadow-1/#selectordef-host)/<a id="ref-for-selectordef-host-function"></a>[:host()](https://drafts.csswg.org/css-shadow-1/#selectordef-host-function)/<a id="ref-for-selectordef-host-context"></a>[:host-context()](https://drafts.csswg.org/css-shadow-1/#selectordef-host-context) pseudo-classes never match.

- <a id="ref-for-selectordef-part①"></a>[::part()](https://drafts.csswg.org/css-shadow-1/#selectordef-part) never matches. (Other <a id="ref-for-element-backed①"></a>[element-backed pseudo-elements](#element-backed) can, however.)

An <a id="ref-for-element-backed②"></a>[element-backed pseudo-element](#element-backed) can define itself as representing a real element (possibly one not accessible in the current tree). If it does so, all pseudo-classes and pseudo-elements not otherwise disallowed (see above) match as they would on that real element. If it does not do so, it must define which pseudo-classes it matches and when; however, unless otherwise specified, any pseudo-classes allowed on <a id="ref-for-tree-abiding④"></a>[tree-abiding pseudo-elements](#tree-abiding) are always allowed on <a id="ref-for-element-backed③"></a>element-backed pseudo-elements.

### <a id="file-selector-button-pseudo"></a>5.1.  File Selector Button: the <a id="ref-for-selectordef-file-selector-button"></a>[::file-selector-button](#selectordef-file-selector-button) pseudo-element[](#file-selector-button-pseudo)

The <a id="selectordef-file-selector-button"></a><strong>::file-selector-button</strong> <a id="ref-for-pseudo-element②⑦"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) targets the \<button\> inside an \<input\> element with <code>type=file</code>, if the UA renders such a button.

Tests

- [file-selector-button-001.html](https://wpt.fyi/results/css/css-pseudo/file-selector-button-001.html) [(live test)](http://wpt.live/css/css-pseudo/file-selector-button-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/file-selector-button-001.html)
- [file-selector-button-after-part.html](https://wpt.fyi/results/css/css-pseudo/file-selector-button-after-part.html) [(live test)](http://wpt.live/css/css-pseudo/file-selector-button-after-part.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/file-selector-button-after-part.html)
- [file-selector-button-display-none-overflow-crash.html](https://wpt.fyi/results/css/css-pseudo/file-selector-button-display-none-overflow-crash.html) [(live test)](http://wpt.live/css/css-pseudo/file-selector-button-display-none-overflow-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/file-selector-button-display-none-overflow-crash.html)
- [file-selector-button-display-toggle-crash.html](https://wpt.fyi/results/css/css-pseudo/file-selector-button-display-toggle-crash.html) [(live test)](http://wpt.live/css/css-pseudo/file-selector-button-display-toggle-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/file-selector-button-display-toggle-crash.html)
- [file-selector-button-float.html](https://wpt.fyi/results/css/css-pseudo/file-selector-button-float.html) [(live test)](http://wpt.live/css/css-pseudo/file-selector-button-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/file-selector-button-float.html)
- [file-selector-button-inherit.html](https://wpt.fyi/results/css/css-pseudo/file-selector-button-inherit.html) [(live test)](http://wpt.live/css/css-pseudo/file-selector-button-inherit.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/file-selector-button-inherit.html)

There is no restriction on which properties apply to the <a id="ref-for-selectordef-file-selector-button①"></a>[::file-selector-button](#selectordef-file-selector-button) pseudo-element.

<a id="file-selector-button-example"></a>

<strong>Example:</strong>

[](#file-selector-button-example) For example, the following example should show a green border around the file selector button:

``` text
::file-selector-button { border: 3px solid green }
```

### <a id="details-content-pseudo"></a>5.2.  Expandable contents of details element: the <a id="ref-for-selectordef-details-content"></a>[::details-content](#selectordef-details-content) pseudo-element[](#details-content-pseudo)

The <a id="selectordef-details-content"></a><strong>::details-content</strong> <a id="ref-for-pseudo-element②⑧"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) targets the additional information in a <code><a id="ref-for-the-details-element"></a>[details](https://html&#46;spec&#46;whatwg&#46;org/multipage/interactive-elements&#46;html#the-details-element)</code> element that can be expanded or collapsed. It is an <a id="ref-for-element-backed④"></a>[element-backed pseudo-element](#element-backed).

Tests

details element

------------------------------------------------------------------------

There is no restriction on which properties apply to the <a id="ref-for-selectordef-details-content①"></a>[::details-content](#selectordef-details-content) pseudo-element.

<a id="details-content-example"></a>

<strong>Example:</strong>

[](#details-content-example) For example, the following example would animate the opacity of the additional information when the <code><a id="ref-for-the-details-element①"></a>[details](https://html&#46;spec&#46;whatwg&#46;org/multipage/interactive-elements&#46;html#the-details-element)</code> element opens:

``` text
details::details-content {
  opacity: 0;
  transition: content-visibility 300ms allow-discrete, opacity 300ms;
}

details[open]::details-content {
  opacity: 1;
}
```

Note: The default UA stylesheet styles a closed <a id="ref-for-selectordef-details-content②"></a>[::details-content](#selectordef-details-content) with <a id="ref-for-propdef-content-visibility"></a>[content-visibility: hidden](https://drafts.csswg.org/css-contain-2/#propdef-content-visibility), so the contents are still considered present for the purpose of find-in-page and similar features. This, however, only suppresses the display of the pseudo-elements’s <em>contents</em>; if margin, borders, etc are set on the pseudo-element, they’ll still be visible. Use \[open\]::details-content to apply these styles if that’s undesirable.

## <a id="interactions"></a>6.  Overlapping Pseudo-element Interactions[](#interactions)

<a id="overlapping-example"></a>

<strong>Example:</strong>

[](#overlapping-example) Recall that

- the contents of <a id="ref-for-selectordef-before⑥"></a>[::before](#selectordef-before) and <a id="ref-for-selectordef-after⑥"></a>[::after](#selectordef-after) are selected exactly as if they were normal elements in the document source tree
- the <a id="ref-for-selectordef-first-letter②⑨"></a>[::first-letter](#selectordef-first-letter) boundaries are tightly wrapped around the first letter text, and <a id="ref-for-selectordef-first-letter③⓪"></a>::first-letter is constrained to exist solely on the first formatted line.
- the <a id="ref-for-selectordef-first-line②③"></a>[::first-line](#selectordef-first-line) start is inserted just inside the containing block’s element boundary, and its end after the close of all content on the line

The following CSS and HTML example illustrates how overlapping <a id="ref-for-pseudo-element②⑨"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) interact:

``` text
<style>
p { color: red; font-size: 12pt }
p::first-letter { color: green; font-size: 200% }
p::first-line { color: blue }
</style>

<p>Some text that ends up on two lines</p>
```

The first letter of each P element will be green with a font size of ’24pt'. The rest of the first formatted line will be blue while the rest of the paragraph will be red.

Assuming that a line break will occur before the word "ends", the <a id="ref-for-fictional-tag-sequence⑤"></a>[fictional tag sequence](#fictional-tag-sequence) for this fragment might be:

``` text
<p>
  <p::first-line>
    <p::first-letter>
      S
    </p::first-letter>
    ome text that
  </p::first-line>
  ends up on two lines
</p>
```

## <a id="cssom"></a>7.  Additions to the CSS Object Model[](#cssom)

Tests

- [idlharness.html](https://wpt.fyi/results/css/css-pseudo/idlharness.html) [(live test)](http://wpt.live/css/css-pseudo/idlharness.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/idlharness.html)

### <a id="CSSPseudoElement-interface"></a>7.1.  <code><a id="ref-for-csspseudoelement"></a>[CSSPseudoElement](#csspseudoelement)</code> Interface[](#CSSPseudoElement-interface)

<a id="ref-for-Exposed"></a><a id="csspseudoelement"></a><a id="ref-for-cssomstring"></a><a id="ref-for-dom-csspseudoelement-type"></a><a id="ref-for-element"></a><a id="ref-for-dom-csspseudoelement-element"></a><a id="ref-for-element①"></a><a id="ref-for-csspseudoelement①"></a><a id="ref-for-dom-csspseudoelement-parent"></a><a id="ref-for-cssomstring①"></a><a id="ref-for-dom-csspseudoelement-selectortext"></a><a id="ref-for-csspseudoelement②"></a><a id="dom-csspseudoelement-pseudo"></a><a id="ref-for-cssomstring②"></a><a id="dom-csspseudoelement-pseudo-type-type"></a>

``` text
[Exposed=Window]
interface CSSPseudoElement {
    readonly attribute CSSOMString type;
    readonly attribute Element element;
    readonly attribute (Element or CSSPseudoElement) parent;
    readonly attribute CSSOMString selectorText;
    CSSPseudoElement? pseudo(CSSOMString type);
};
```

Tests

- [CSSPseudoElement-identity.html](https://wpt.fyi/results/css/css-pseudo/CSSPseudoElement-identity.html) [(live test)](http://wpt.live/css/css-pseudo/CSSPseudoElement-identity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/CSSPseudoElement-identity.html)

<a id="issue-22d457b7"></a>

<strong>Issue:</strong>

[](#issue-22d457b7) This interface is under design development, and this draft is looking for feedback more than implementation. The CSSWG would particularly appreciate hearing about use cases and problems.

The <a id="dom-csspseudoelement-type"></a><strong><code>type</code></strong> attribute is a string representing the type of the pseudo-element. This can be any standardized <a id="ref-for-tree-abiding⑤"></a>[tree-abiding pseudo-element](#tree-abiding) that is not an <a id="ref-for-element-backed⑤"></a>[element-backed pseudo-element](#element-backed), for example:

<strong><code>&#34;&#58;&#58;before&#34;</code></strong>

Represents the <a id="ref-for-selectordef-before⑦"></a>[::before](#selectordef-before) pseudo-element.

<strong><code>&#34;&#58;&#58;after&#34;</code></strong>

Represents the <a id="ref-for-selectordef-after⑦"></a>[::after](#selectordef-after) pseudo-element.

<strong><code>&#34;&#58;&#58;marker&#34;</code></strong>

Represents the <a id="ref-for-selectordef-marker⑦"></a>[::marker](#selectordef-marker) pseudo-element.

<strong><code>&#34;&#58;&#58;backdrop&#34;</code></strong>

Represents the <a id="ref-for-selectordef-backdrop"></a>[::backdrop](https://drafts.csswg.org/css-position-4/#selectordef-backdrop) pseudo-element.

<strong><code>&#34;&#58;&#58;view-transition&#34;</code></strong>

Represents the <a id="ref-for-selectordef-view-transition"></a>[::view-transition](https://drafts.csswg.org/css-view-transitions-2/#selectordef-view-transition) pseudo-element.

The <a id="dom-csspseudoelement-element"></a><strong><code>element</code></strong> attribute is the <a id="ref-for-ultimate-originating-element"></a>[ultimate originating element](https://drafts.csswg.org/selectors-4/#ultimate-originating-element) of the <a id="ref-for-pseudo-element③⓪"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element).

The <a id="dom-csspseudoelement-parent"></a><strong><code>parent</code></strong> attribute is the <a id="ref-for-originating-element②②"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) of the pseudo-element. For most pseudo-elements <code><a id="ref-for-dom-csspseudoelement-parent①"></a>[parent](#dom-csspseudoelement-parent)</code> and <code><a id="ref-for-dom-csspseudoelement-element①"></a>[element](#dom-csspseudoelement-element)</code> will return the same <code><a id="ref-for-element②"></a>[Element](https://dom&#46;spec&#46;whatwg&#46;org/#element)</code>; for <a id="ref-for-sub-pseudo-element②"></a>[sub-pseudo-elements](https://drafts.csswg.org/selectors-4/#sub-pseudo-element), <code><a id="ref-for-dom-csspseudoelement-parent②"></a>[parent](#dom-csspseudoelement-parent)</code> will return a <code><a id="ref-for-csspseudoelement③"></a>[CSSPseudoElement](#csspseudoelement)</code> while <code><a id="ref-for-dom-csspseudoelement-element②"></a>[element](#dom-csspseudoelement-element)</code> returns an <code><a id="ref-for-element③"></a>[Element](https://dom&#46;spec&#46;whatwg&#46;org/#element)</code>.

The <a id="dom-csspseudoelement-selectortext"></a><strong><code>selectorText</code></strong> attribute is a string representing the full, normalized selector text used to select the pseudo-element. This includes the pseudo-element name and any arguments, serialized in a form that can round-trip. For example, <code>&#34;&#58;&#58;after&#34;</code> for a <a id="ref-for-selectordef-after⑧"></a>[::after](#selectordef-after) pseudo-element, or <code>&#34;&#58;&#58;scroll-button(left)&#34;</code> for a ::scroll-button() pseudo-element with a <code>left</code> argument.

The <a id="dom-csspseudoelement-pseudo-type"></a><strong><code>pseudo(type)</code></strong> method returns the <code><a id="ref-for-csspseudoelement④"></a>[CSSPseudoElement](#csspseudoelement)</code> interface representing the <a id="ref-for-sub-pseudo-element③"></a>[sub-pseudo-element](https://drafts.csswg.org/selectors-4/#sub-pseudo-element) referenced in its argument, if such a <a id="ref-for-sub-pseudo-element④"></a>sub-pseudo-element could exist and would be valid, and null otherwise. See <code><a id="ref-for-dom-element-pseudo"></a>[pseudo()](#dom-element-pseudo)</code> below.

Note: This interface may be extended in the future to other pseudo-element types and/or to allow setting style information through a <code><a id="ref-for-cssstyledeclaration"></a>[CSSStyleDeclaration](https://drafts&#46;csswg&#46;org/cssom-1/#cssstyledeclaration)</code> <code>style</code> attribute. The current functionality is limited to that which is needed to support [\[web-animations-1\]](#biblio-web-animations-1).

### <a id="window-interface"></a>7.2.  <code><a id="ref-for-dom-element-pseudo①"></a>[pseudo()](#dom-element-pseudo)</code> method of the <code><a id="ref-for-element④"></a>[Element](https://dom&#46;spec&#46;whatwg&#46;org/#element)</code> interface[](#window-interface)

A new method is added to the <code><a id="ref-for-element⑤"></a>[Element](https://dom&#46;spec&#46;whatwg&#46;org/#element)</code> interface to retrieve <a id="ref-for-pseudo-element③①"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) created by a given element for a given type:

<a id="ref-for-element⑥"></a><a id="ref-for-csspseudoelement⑤"></a><a id="ref-for-dom-element-pseudo②"></a><a id="ref-for-cssomstring③"></a><a id="dom-element-pseudo-type-type"></a>

``` text
partial interface Element {
  CSSPseudoElement? pseudo(CSSOMString type);
};
```

The <a id="dom-element-pseudo"></a><strong><code>pseudo(CSSOMString type)</code></strong> method is used to retrieve the <code><a id="ref-for-csspseudoelement⑥"></a>[CSSPseudoElement](#csspseudoelement)</code> instance of the type matching <code><a id="ref-for-dom-element-pseudo-type-type"></a>[type](#dom-element-pseudo-type-type)</code> associated with the element. When it is called, execute the following steps:

1.  <a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>[Parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) the <code><a id="ref-for-dom-element-pseudo-type-type①"></a>[type](#dom-element-pseudo-type-type)</code> argument as a <a id="ref-for-typedef-pseudo-element-selector"></a>[\<pseudo-element-selector\>](https://drafts.csswg.org/selectors-4/#typedef-pseudo-element-selector), and let <var>type</var> be the result.

2.  If <var>type</var> is failure, return null.

3.  If <var>type</var> is an <a id="ref-for-element-backed⑥"></a>[element-backed pseudo-element](#element-backed), return null.

4.  If <var>type</var> is not a <a id="ref-for-tree-abiding⑥"></a>[tree-abiding pseudo-element](#tree-abiding), return null.

5.  Otherwise, return the <code><a id="ref-for-csspseudoelement⑦"></a>[CSSPseudoElement](#csspseudoelement)</code> object representing the pseudo-element that would match the selector <var>type</var> with <a id="ref-for-this"></a>[this](https://webidl.spec.whatwg.org/#this) as its <a id="ref-for-originating-element②③"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element).

Return values that represent the same <a id="ref-for-pseudo-element③②"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) on the same <a id="ref-for-originating-element②④"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) must be, insofar as observable, always the same <code><a id="ref-for-csspseudoelement⑧"></a>[CSSPseudoElement](#csspseudoelement)</code> object. (The UA may drop or regenerate the object for convenience or performance if this is not observable.)

Note: <code><a id="ref-for-csspseudoelement⑨"></a>[CSSPseudoElement](#csspseudoelement)</code> is returned even when this method is invoked on elements that can’t have the requested pseudo type (e.g. <a id="ref-for-selectordef-before⑧"></a>[::before](#selectordef-before) on <code><a id="ref-for-the-input-element①"></a>[input](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#the-input-element)</code>). See [resolution](https://github.com/w3c/csswg-drafts/issues/12159#issuecomment-3005238286) for more details. Discussion in [Issue 12158](https://github.com/w3c/csswg-drafts/issues/12158) will clarify how to determine that the requested pseudo doesn’t really "exist".

<a id="issue-302461e7"></a>

<strong>Issue:</strong>

[](#issue-302461e7) The identity, lifetime, and nullness of the return value (and potential error cases) of the <code><a id="ref-for-dom-csspseudoelement-pseudo"></a>[pseudo()](#dom-csspseudoelement-pseudo)</code> method is still under discussion. See [Issue 3607](https://github.com/w3c/csswg-drafts/issues/3607) and [Issue 3603](https://github.com/w3c/csswg-drafts/issues/3603).

### <a id="getComputedStyle"></a>7.3.  <code><a id="ref-for-dom-window-getcomputedstyle"></a>[getComputedStyle()](https://drafts&#46;csswg&#46;org/cssom-1/#dom-window-getcomputedstyle)</code>[](#getComputedStyle)

When the second parameter <var>pseudoElt</var> refers to a <a id="ref-for-highlight-pseudo-element②⓪"></a>[highlight pseudo-element](#highlight-pseudo-element), <code><a id="ref-for-dom-window-getcomputedstyle①"></a>[getComputedStyle()](https://drafts&#46;csswg&#46;org/cssom-1/#dom-window-getcomputedstyle)</code> returns styles as if that highlight is active and all other highlights are inactive. This avoids the potential ambiguity and privacy risks of returning a result that depends on the actual highlight state.

## <a id="css2-compat"></a>8.  Compatibility Syntax[](#css2-compat)

For compatibility with existing style sheets written against CSS Level 2 [\[CSS2\]](#biblio-css2), user agents must also accept the previous one-colon notation (<code>&#58;before</code>, <code>&#58;after</code>, <code>&#58;first-letter</code>, <code>&#58;first-line</code>) for the <a id="ref-for-selectordef-before⑨"></a>[::before](#selectordef-before), <a id="ref-for-selectordef-after⑨"></a>[::after](#selectordef-after), <a id="ref-for-selectordef-first-letter③①"></a>[::first-letter](#selectordef-first-letter), and <a id="ref-for-selectordef-first-line②④"></a>[::first-line](#selectordef-first-line) pseudo-elements.

## <a id="security"></a> Security Considerations[](#security)

For highlighting, see [§ 3.7 Security Considerations for Highlighting](#highlight-security).

## <a id="privacy"></a> Privacy Considerations[](#privacy)

No new privacy considerations have been raised against this module.

## <a id="changes"></a> Changes[](#changes)

Significant changes since the [30 December 2022 Working Draft](https://www.w3.org/TR/2022/WD-css-pseudo-4-20221230/) include:

- Defined <a id="ref-for-element-backed⑦"></a>[element-backed pseudo-element](#element-backed) and <a id="ref-for-fully-styleable④"></a>[fully styleable pseudo-element](#fully-styleable) as terms. ([Issue 10199](https://github.com/w3c/csswg-drafts/pull/10199), [Issue 10083](https://github.com/w3c/csswg-drafts/issues/10083))
- Reworked the property (and custom property) application and inheritance model for <a id="ref-for-highlight-pseudo-element②①"></a>[highlight pseudo-elements](#highlight-pseudo-element) (Issues [7591](https://github.com/w3c/csswg-drafts/issues/7591), [7580](https://github.com/w3c/csswg-drafts/issues/7580), [6641](https://github.com/w3c/csswg-drafts/issues/6641), [11317](https://github.com/w3c/csswg-drafts/issues/11317))
- Defined <a id="ref-for-selectordef-highlight-highlight-name-selector④"></a>[::highlight()](#selectordef-highlight-highlight-name-selector) and improved cross-referencing to [\[CSS-HIGHLIGHT-API-1\]](#biblio-css-highlight-api-1). (Issue [6755](https://github.com/w3c/csswg-drafts/issues/6755))
- Added <a id="ref-for-selectordef-search-text⑤"></a>[::search-text](#selectordef-search-text) as a new <a id="ref-for-highlight-pseudo-element②②"></a>[highlight pseudo-element](#highlight-pseudo-element), and allowed it to combine with <a id="ref-for-current-pseudo①"></a>[:current](https://drafts.csswg.org/selectors-5/#current-pseudo). (Issues [10212](https://github.com/w3c/csswg-drafts/issues/10212) [10213](https://github.com/w3c/csswg-drafts/issues/10213), and [10298](https://github.com/w3c/csswg-drafts/issues/10298))
- Added <a id="ref-for-selectordef-details-content③"></a>[::details-content](#selectordef-details-content) <a id="ref-for-pseudo-element③③"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) ([Issue 10379](https://github.com/w3c/csswg-drafts/pull/10379))
- Allow space and no-break space to be part of <a id="ref-for-selectordef-first-letter③②"></a>[::first-letter](#selectordef-first-letter) when occuring before the <a id="ref-for-first-letter⑦"></a>[first letter](#first-letter). ([Issue 9413](https://github.com/w3c/csswg-drafts/issues/9413))
- Renamed ::postfix to <a id="ref-for-selectordef-first-letter-suffix⑤"></a>[::suffix](#selectordef-first-letter-suffix). (Issue [Issue 11949](https://github.com/w3c/csswg-drafts/issues/11949))
- Excluded inline layout properties from <a id="ref-for-selectordef-placeholder⑤"></a>[::placeholder](#selectordef-placeholder). ([Issue 5379](https://github.com/w3c/csswg-drafts/issues/5379))
- Clarified that UAs must not tweak author-specified colors for <a id="ref-for-highlight-pseudo-element②③"></a>[highlight pseudo-elements](#highlight-pseudo-element). ([Issue 6853](https://github.com/w3c/csswg-drafts/issues/6853))
- Clarified that the inherited value of <a id="ref-for-propdef-color①④"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) is <a id="ref-for-valdef-color-currentcolor③"></a>[currentColor](https://drafts.csswg.org/css-color-4/#valdef-color-currentcolor), not the <a id="ref-for-initial-value②"></a>[initial value](https://drafts.csswg.org/css-cascade-5/#initial-value).

Significant changes since the [31 December 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-pseudo-4-20201231/) include:

- Added ::first-letter::prefix and ::first-letter::postfix <a id="ref-for-sub-pseudo-element⑤"></a>[sub-pseudo-elements](https://drafts.csswg.org/selectors-4/#sub-pseudo-element) to address punctuation associated with the <a id="ref-for-selectordef-first-letter③③"></a>[::first-letter](#selectordef-first-letter) <a id="ref-for-pseudo-element③④"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element). ([Issue 2040](https://github.com/w3c/csswg-drafts/issues/2040))
- Fine-tuned the <a id="ref-for-first-letter-text①③"></a>[first-letter text](#first-letter-text) pattern:
  - Excluded word-separating spaces from <a id="ref-for-first-letter-text①④"></a>[first-letter text](#first-letter-text). ([Issue 5830](https://github.com/w3c/csswg-drafts/issues/5830))
  - Excluded dashes and opening punctuation from following punctuation. ([Issue 5830](https://github.com/w3c/csswg-drafts/issues/5830))
  - Allowed symbols to represent the <a id="ref-for-first-letter⑧"></a>[first letter](#first-letter). ([Issue 5099](https://github.com/w3c/csswg-drafts/issues/5099))
  - Clarified that the <a id="ref-for-first-letter-text①⑤"></a>[first-letter text](#first-letter-text) is truncated by a line break. ([Issue 2254](https://github.com/w3c/csswg-drafts/issues/2254))
- Rewrote and clarified definition of <a id="ref-for-selectordef-first-letter③④"></a>[:first-letter](https://drafts.csswg.org/css2/#selectordef-first-letter), and improved examples and notes.
- Rewrote and clarified definition of <a id="ref-for-selectordef-first-line②⑤"></a>[::first-line](#selectordef-first-line), and improved examples.
- Defined application of <a id="ref-for-propdef-line-height②"></a>[line-height](https://drafts.csswg.org/css2/#propdef-line-height) to <a id="ref-for-selectordef-first-line②⑥"></a>[::first-line](#selectordef-first-line). ([Issue 2282](https://github.com/w3c/csswg-drafts/issues/2282))
- Defined box structure interaction of <a id="ref-for-selectordef-first-line②⑦"></a>[::first-line](#selectordef-first-line) and <a id="ref-for-selectordef-first-letter③⑤"></a>[::first-letter](#selectordef-first-letter).
- Defined hierarchy order of <a id="ref-for-selectordef-first-line②⑧"></a>[::first-line](#selectordef-first-line) and the <a id="ref-for-root-inline-box②"></a>[root inline box](https://drafts.csswg.org/css-inline-3/#root-inline-box). ([Issue 1384](https://github.com/w3c/csswg-drafts/issues/1384))
- Clarified [paired default](#paired-defaults) behavior of <a id="ref-for-highlight-pseudo-element②④"></a>[highlight pseudo-element](#highlight-pseudo-element) <a id="ref-for-propdef-color①⑤"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) and <a id="ref-for-propdef-background-color⑦"></a>[background-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color). ([Issue 6386](https://github.com/w3c/csswg-drafts/issues/6386), [Issue 6774](https://github.com/w3c/csswg-drafts/issues/6774), [Issue 6779](https://github.com/w3c/csswg-drafts/issues/6779), [Issue 7837](https://github.com/w3c/csswg-drafts/issues/7837))
- Defined application of <a id="ref-for-forced-colors-mode①"></a>[forced colors mode](https://drafts.csswg.org/css-color-adjust-1/#forced-colors-mode) to <a id="ref-for-highlight-pseudo-element②⑤"></a>[highlight pseudo-elements](#highlight-pseudo-element). ([Issue 7264](https://github.com/w3c/csswg-drafts/issues/7264))
- Clarified painting order, propagation, and application of <a id="ref-for-highlight-pseudo-element②⑥"></a>[highlight pseudo-element](#highlight-pseudo-element) text decorations. ([Issue 6022](https://github.com/w3c/csswg-drafts/issues/6022), [Issue 6829](https://github.com/w3c/csswg-drafts/issues/6829), [Issue 7101](https://github.com/w3c/csswg-drafts/issues/7101))
- Removed <a id="ref-for-propdef-caret-color"></a>[caret-color](https://drafts.csswg.org/css-ui-4/#propdef-caret-color) and <a id="ref-for-propdef-cursor"></a>[cursor](https://drafts.csswg.org/css-ui-4/#propdef-cursor) from applying to <a id="ref-for-highlight-pseudo-element②⑦"></a>[highlight pseudo-elements](#highlight-pseudo-element). ([Issue 4100](https://github.com/w3c/csswg-drafts/issues/4100))
- Clarified interaction of <a id="ref-for-highlight-pseudo-element②⑧"></a>[highlight pseudo-elements](#highlight-pseudo-element) and <code><a id="ref-for-dom-window-getcomputedstyle②"></a>[getComputedStyle()](https://drafts&#46;csswg&#46;org/cssom-1/#dom-window-getcomputedstyle)</code>. ([Issue 6818](https://github.com/w3c/csswg-drafts/issues/6818))
- Clarified application of <a id="ref-for-custom-property⑥"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) to <a id="ref-for-pseudo-element③⑤"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element). ([Issue 6264](https://github.com/w3c/csswg-drafts/issues/6264))
- 
- Extended the <code><a id="ref-for-csspseudoelement①⓪"></a>[CSSPseudoElement](#csspseudoelement)</code> interface with a <code><a id="ref-for-dom-csspseudoelement-parent③"></a>[parent](#dom-csspseudoelement-parent)</code> attribute and <code><a id="ref-for-dom-csspseudoelement-pseudo-type"></a>[pseudo()](#dom-csspseudoelement-pseudo-type)</code> method to address <a id="ref-for-sub-pseudo-element⑥"></a>[sub-pseudo-elements](https://drafts.csswg.org/selectors-4/#sub-pseudo-element). ([Issue 3836](https://github.com/w3c/csswg-drafts/issues/3836))

Significant changes since the [25 February 2019 Working Draft](https://www.w3.org/TR/2019/WD-css-pseudo-4-20190225/) include:

- Added <a id="ref-for-selectordef-target-text①⓪"></a>[::target-text](#selectordef-target-text) <a id="ref-for-pseudo-element③⑥"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element). ([Issue 5522](https://github.com/w3c/csswg-drafts/issues/5522))
- Removed ::inactive-selection. ([Issue 4579](https://github.com/w3c/csswg-drafts/issues/4579))
- Added ::file-selectors-button <a id="ref-for-pseudo-element③⑦"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element). ([Issue 5049](https://github.com/w3c/csswg-drafts/issues/5049))
- Adjusted rules for inheritance of properties from <a id="ref-for-selectordef-first-line②⑨"></a>[::first-line](#selectordef-first-line) to handle inheritable vs non-inheritable properties differently, specifically excluded applicability of properties that control <a id="ref-for-writing-mode"></a>[writing mode](https://drafts.csswg.org/css-writing-modes-4/#writing-mode) because they affect the cascade, and defined interaction with custom properties. ([Issue 1097](https://github.com/w3c/csswg-drafts/issues/1097))
- Added <a id="ref-for-propdef-ruby-position①"></a>[ruby-position](https://drafts.csswg.org/css-ruby-1/#propdef-ruby-position) to properties allowed on <a id="ref-for-selectordef-first-line③⓪"></a>[::first-line](#selectordef-first-line), analogous with <a id="ref-for-propdef-text-emphasis-position"></a>[text-emphasis-position](https://drafts.csswg.org/css-text-decor-4/#propdef-text-emphasis-position). ([Issue 2998](https://github.com/w3c/csswg-drafts/issues/2998))
- Included spaces between the first letter and its surrounding punctuation in <a id="ref-for-selectordef-first-letter③⑥"></a>[::first-letter](#selectordef-first-letter). ([Issue 5154](https://github.com/w3c/csswg-drafts/issues/5154))
- Defined interaction of <a id="ref-for-selectordef-marker⑧"></a>[::marker](#selectordef-marker) and <a id="ref-for-selectordef-first-letter③⑦"></a>[::first-letter](#selectordef-first-letter).
- Made ::before::marker and ::after::marker valid. ([Issue 1793](https://github.com/w3c/csswg-drafts/issues/1793))
- Moved the [list of properties](https://www.w3.org/TR/css-lists-3/#marker-properties) applying to <a id="ref-for-selectordef-marker⑨"></a>[::marker](#selectordef-marker) to [\[CSS-LISTS-3\]](#biblio-css-lists-3) (also adding <a id="ref-for-propdef-content②"></a>[content](https://drafts.csswg.org/css-content-3/#propdef-content), <a id="ref-for-propdef-unicode-bidi"></a>[unicode-bidi](https://drafts.csswg.org/css-writing-modes-3/#propdef-unicode-bidi), and <a id="ref-for-propdef-direction①"></a>[direction](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction)).
- Added <a id="ref-for-selectordef-marker①⓪"></a>[::marker](#selectordef-marker) to types allowed for <code><a id="ref-for-csspseudoelement①①"></a>[CSSPseudoElement](#csspseudoelement)</code> interface. ([Issue 3763](https://github.com/w3c/csswg-drafts/issues/3763))
- Clarified return value of <code><a id="ref-for-dom-csspseudoelement-pseudo①"></a>[pseudo()](#dom-csspseudoelement-pseudo)</code>. (Issues [3603](https://github.com/w3c/csswg-drafts/issues/3603) and [3607](https://github.com/w3c/csswg-drafts/issues/3607))
- Allowed highlight overlay of inline-level boxes to extend to the line box edges. (Issues [5395](https://github.com/w3c/csswg-drafts/issues/5395) and [4624](https://github.com/w3c/csswg-drafts/issues/4624))
- Clarified that <a id="ref-for-valdef-color-currentcolor④"></a>[currentColor](https://drafts.csswg.org/css-color-4/#valdef-color-currentcolor) on a <a id="ref-for-highlight-pseudo-element②⑨"></a>[highlight pseudo-element](#highlight-pseudo-element) uses the color that would be used if the text were not “highlighted”, even if that color was provided by a <a id="ref-for-pseudo-element③⑧"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) like <a id="ref-for-selectordef-first-line③①"></a>[::first-line](#selectordef-first-line). ([Issue 4625](https://github.com/w3c/csswg-drafts/issues/4625))
- Defined how <a id="ref-for-propdef-text-shadow③"></a>[text-shadow](https://drafts.csswg.org/css-text-decor-4/#propdef-text-shadow) interacts with <a id="ref-for-highlight-pseudo-element③⓪"></a>[highlight pseudo-elements](#highlight-pseudo-element). ([Issue 3932](https://github.com/w3c/csswg-drafts/issues/3932))
- Improved the [default UA style sheet](#highlight-ua-styles) rules applying to highlight pseudo-elements.

Changes since the [7 June 2016 Working Draft](https://www.w3.org/TR/2016/WD-css-pseudo-4-20160607/) include:

- Specified <a id="ref-for-valdef-text-decoration-line-spelling-error"></a>[spelling-error](https://drafts.csswg.org/css-text-decor-4/#valdef-text-decoration-line-spelling-error) and <a id="ref-for-valdef-text-decoration-line-grammar-error"></a>[grammar-error](https://drafts.csswg.org/css-text-decor-4/#valdef-text-decoration-line-grammar-error) with <a id="ref-for-selectordef-spelling-error④"></a>[::spelling-error](#selectordef-spelling-error) and <a id="ref-for-selectordef-grammar-error④"></a>[::grammar-error](#selectordef-grammar-error) in the UA stylesheet.
- Redefined value propagation between parent/child <a id="ref-for-highlight-pseudo-element③①"></a>[highlight pseudo-elements](#highlight-pseudo-element) to use inheritance rather than cascading. See discussion in and linked from [Issue 2474](https://github.com/w3c/csswg-drafts/issues/2474).
- Refined list of supported properties for <a id="ref-for-highlight-pseudo-element③②"></a>[highlight pseudo-elements](#highlight-pseudo-element), e.g. adding <a id="ref-for-propdef-stroke-color②"></a>[stroke-color](https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-color)/<a id="ref-for-propdef-fill-color②"></a>[fill-color](https://drafts.csswg.org/fill-stroke-3/#propdef-fill-color), removing <a id="ref-for-propdef-outline"></a>[outline](https://drafts.csswg.org/css-ui-4/#propdef-outline), etc.
- Clarified how text and text decoration colors are painted for <a id="ref-for-highlight-pseudo-element③③"></a>[highlight pseudo-elements](#highlight-pseudo-element).
- Added the <code><a id="ref-for-dom-csspseudoelement-element③"></a>[element](#dom-csspseudoelement-element)</code> attribute to the <code><a id="ref-for-csspseudoelement①②"></a>[CSSPseudoElement](#csspseudoelement)</code> interface.
- Changed the values of the <code><a id="ref-for-dom-csspseudoelement-type①"></a>[type](#dom-csspseudoelement-type)</code> attribute on the <code><a id="ref-for-csspseudoelement①③"></a>[CSSPseudoElement](#csspseudoelement)</code> interface to match the corresponding <a id="ref-for-pseudo-element③⑨"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element).
- Reduced supported types of <code><a id="ref-for-csspseudoelement①④"></a>[CSSPseudoElement](#csspseudoelement)</code> and removed unimplemented <code>style</code> attribute.
- Changed <code>window&#46;getPseudoElements(elem,type)</code> to <code>Element&#46;pseudo(type)</code>.
- Miscellaneous minor clarifications and fixes.

## <a id="acknowledgements"></a> Acknowledgements[](#acknowledgements)

The editors would like to specifically thank the following individuals for their contributions to this specification: Tab Atkins, Delan Azabani, David Baron, Oriol Brufau, Razvan Caliman, Stephen Chenney, Chris Coyier, Anders Grimsrud, Vincent Hardy, François Remy.

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

- [::after](#selectordef-after), in § 4.1
- [::before](#selectordef-before), in § 4.1
- [CSSPseudoElement](#csspseudoelement), in § 7.1
- [::details-content](#selectordef-details-content), in § 5.2
- [element](#dom-csspseudoelement-element), in § 7.1
- [element-backed](#element-backed), in § 5
- [element-backed pseudo-element](#element-backed), in § 5
- [fictional tag sequence](#fictional-tag-sequence), in § 2.1.3
- [::file-selector-button](#selectordef-file-selector-button), in § 5.1
- [first formatted line](#first-formatted-line), in § 2.1.1
- [::first-letter](#selectordef-first-letter), in § 2.2
- [first letter](#first-letter), in § 2.2
- [first-letter text](#first-letter-text), in § 2.2
- [::first-line](#selectordef-first-line), in § 2.1
- [fully styleable](#fully-styleable), in § 4
- [fully styleable pseudo-elements](#fully-styleable), in § 4
- [::grammar-error](#selectordef-grammar-error), in § 3.1
- [::highlight(\<highlight-name-selector\>)](#selectordef-highlight-highlight-name-selector), in § 3.1
- [\<highlight-name-selector\>](#typedef-highlight-name-selector), in § 3.1
- [highlight overlay](#highlight-overlay), in § 3.4
- [highlight pseudo-element](#highlight-pseudo-element), in § 3.1
- [::marker](#selectordef-marker), in § 4.2
- [paired default highlight colors](#paired-default-highlight-colors), in § 3.3
- [parent](#dom-csspseudoelement-parent), in § 7.1
- [::placeholder](#selectordef-placeholder), in § 4.3
- [::prefix](#selectordef-first-letter-prefix), in § 2.2
- pseudo(type)
  - [attribute for CSSPseudoElement](#dom-csspseudoelement-pseudo-type), in § 7.1
  - [method for CSSPseudoElement](#dom-csspseudoelement-pseudo), in § 7.1
  - [method for Element](#dom-element-pseudo), in § 7.2
- [::search-text](#selectordef-search-text), in § 3.1
- [::selection](#selectordef-selection), in § 3.1
- [selectorText](#dom-csspseudoelement-selectortext), in § 7.1
- [::spelling-error](#selectordef-spelling-error), in § 3.1
- [::suffix](#selectordef-first-letter-suffix), in § 2.2
- [::target-text](#selectordef-target-text), in § 3.1
- [tree-abiding](#tree-abiding), in § 4
- [Tree-abiding pseudo-elements](#tree-abiding), in § 4
- [type](#dom-csspseudoelement-type), in § 7.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference[](#index-defined-elsewhere)

- \[COMPAT\] defines the following terms:
  - <a id="139af2ab"></a>-webkit-text-fill-color
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="11fb4d5b"></a>background-color
  - <a id="6916104d"></a>box-shadow
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="c3bd6e1d"></a>box fragment
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="39e92270"></a>author origin
  - <a id="c665612f"></a>cascaded value
  - <a id="9cd25054"></a>computed value
  - <a id="21ec9802"></a>inherit (for CSS)
  - <a id="c388d253"></a>inherit (for all)
  - <a id="20fff200"></a>inheritance
  - <a id="2496f7a0"></a>inherited property
  - <a id="9ea6bed2"></a>initial value
  - <a id="bef88b2d"></a>revert
  - <a id="8e92a2a0"></a>revert-layer
  - <a id="1cf918c1"></a>specified value
  - <a id="9e004c39"></a>unset
  - <a id="a4eb3c57"></a>used value
- \[CSS-CASCADE-6\] defines the following terms:
  - <a id="6eacac06"></a>cascade origin
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="02c921d1"></a>\<system-color\>
  - <a id="73ea1d43"></a>color
  - <a id="cad72d2f"></a>currentcolor
  - <a id="21ff7f1a"></a>highlight
  - <a id="9c4a432b"></a>HighlightText
  - <a id="f94cfee7"></a>mark
  - <a id="533df3d1"></a>MarkText
  - <a id="91bfbe18"></a>opacity
  - <a id="313bb160"></a>transparent
- \[CSS-COLOR-ADJUST-1\] defines the following terms:
  - <a id="c373c5f8"></a>color-scheme
  - <a id="abb10c72"></a>forced colors mode
  - <a id="259a4df0"></a>forced-color-adjust
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="2306aaa1"></a>content-visibility
- \[CSS-CONTENT-3\] defines the following terms:
  - <a id="2467179e"></a>content
  - <a id="991145b0"></a>none
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="9f3d4f17"></a>block container
  - <a id="ca960d4f"></a>block formatting context
  - <a id="8379845e"></a>block-level
  - <a id="ea663a43"></a>containing block
  - <a id="e7f0dd6c"></a>display
  - <a id="99cabd32"></a>in-flow
  - <a id="c02f5d57"></a>inline
  - <a id="fc6c8cc8"></a>inline block
  - <a id="083ffe5f"></a>inline box
  - <a id="b8f126f1"></a>inline formatting context
  - <a id="d7fa3e1c"></a>inline-level
  - <a id="54d48ea2"></a>inline-level box
  - <a id="e77a37fc"></a>inline-level content
  - <a id="96626f55"></a>list-item
  - <a id="7ba1e996"></a>out-of-flow
  - <a id="a8485ff4"></a>replaced element
  - <a id="cc682c89"></a>root element
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="d65d17df"></a>font-family
  - <a id="fdf6efd5"></a>font-size
- \[CSS-HIGHLIGHT-API-1\] defines the following terms:
  - <a id="a45f1010"></a>custom highlight
  - <a id="392ffeb0"></a>custom highlight name
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="12cf38c9"></a>initial-letter
  - <a id="e34548f4"></a>line box
  - <a id="6f7798a0"></a>root inline box
- \[CSS-LISTS-3\] defines the following terms:
  - <a id="ac8c80fa"></a>list item
  - <a id="16d57f47"></a>list-style-position
  - <a id="4102a2ec"></a>marker
  - <a id="61656631"></a>marker box
- \[CSS-MULTICOL-2\] defines the following terms:
  - <a id="396d5070"></a>multi-column container
- \[CSS-POSITION-4\] defines the following terms:
  - <a id="b9a45e7f"></a>::backdrop
- \[CSS-RUBY-1\] defines the following terms:
  - <a id="dd21c139"></a>ruby-position
- \[CSS-SHADOW-1\] defines the following terms:
  - <a id="361a646a"></a>::part()
  - <a id="5561253e"></a>:host
  - <a id="21e38b43"></a>:host()
  - <a id="37487143"></a>:host-context()
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="76c4403d"></a>parse
- \[CSS-TABLES-3\] defines the following terms:
  - <a id="994d0e36"></a>table wrapper box
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="4356bfe3"></a>text-align
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="5879b11d"></a>content language
  - <a id="237b3136"></a>typographic character unit
  - <a id="be8bf61d"></a>Unicode General Category
  - <a id="86f91454"></a>word separator
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="d3129b15"></a>grammar-error
  - <a id="4462edae"></a>spelling-error
  - <a id="5bc3d0ec"></a>text-decoration
  - <a id="557f5d97"></a>text-emphasis
  - <a id="19119b4f"></a>text-emphasis-position
  - <a id="f338a9f0"></a>text-shadow
  - <a id="c9b2f77d"></a>text-underline-offset
  - <a id="d2181a0c"></a>text-underline-position
- \[CSS-UI-4\] defines the following terms:
  - <a id="8e0f26cc"></a>caret-color
  - <a id="647de2a6"></a>cursor
  - <a id="355da394"></a>outline
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="e274345c"></a>\<custom-ident\>
  - <a id="e8ff0bb4"></a>em
  - <a id="35d9f24e"></a>font-relative lengths
  - <a id="19203ecf"></a>lh
  - <a id="6ec67710"></a>\|
- \[CSS-VARIABLES-2\] defines the following terms:
  - <a id="198f1537"></a>custom property
  - <a id="dbba942a"></a>var()
- \[CSS-VIEW-TRANSITIONS-2\] defines the following terms:
  - <a id="ad981cd9"></a>::view-transition
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="5bc8bd12"></a>direction
  - <a id="982e17a3"></a>unicode-bidi
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="e1ab41b7"></a>block axis
  - <a id="3bd4bbe9"></a>text-orientation
  - <a id="a73617e0"></a>writing mode
  - <a id="385326d7"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="689ef56c"></a>:after
  - <a id="20d80a30"></a>:first-letter
  - <a id="ac54fbff"></a>line-height
  - <a id="85be5f20"></a>pseudo-classes
- \[CSSOM-1\] defines the following terms:
  - <a id="5fced98b"></a>CSSOMString
  - <a id="a82a20ef"></a>CSSStyleDeclaration
  - <a id="b203675a"></a>getComputedStyle(elt)
- \[DOM\] defines the following terms:
  - <a id="296f3551"></a>Element
- \[FILL-STROKE-3\] defines the following terms:
  - <a id="2c66542e"></a>fill-color
  - <a id="a19e06f8"></a>stroke-color
  - <a id="2bb037f1"></a>stroke-width
- \[HTML\] defines the following terms:
  - <a id="1748bbeb"></a>details
  - <a id="a7bbaf9f"></a>div
  - <a id="d7d642a2"></a>input
  - <a id="b9a05383"></a>placeholder
  - <a id="fc736137"></a>textarea
- \[SELECTORS-3\] defines the following terms:
  - <a id="53a193f1"></a>\*
- \[SELECTORS-4\] defines the following terms:
  - <a id="9b20a4a2"></a>:has()
  - <a id="a20bf8d2"></a>:placeholder-shown
  - <a id="dc55c3d9"></a>:scope
  - <a id="efe16f32"></a>:target
  - <a id="ad337600"></a>\<pseudo-element-selector\>
  - <a id="2e965368"></a>functional pseudo-element
  - <a id="82805a4e"></a>originating element
  - <a id="628c6566"></a>pseudo-class
  - <a id="42b00c6e"></a>pseudo-elements
  - <a id="33cbcfca"></a>selector
  - <a id="59d124b8"></a>structural pseudo-classes
  - <a id="f8abdd6d"></a>sub-pseudo-element
  - <a id="32c45571"></a>type selector
  - <a id="d99f286c"></a>ultimate originating element
- \[SELECTORS-5\] defines the following terms:
  - <a id="bc7f59fa"></a>:current
  - <a id="4eb2c150"></a>:future
  - <a id="34425ee5"></a>:past
- \[URL\] defines the following terms:
  - <a id="ed948033"></a>fragment
- \[WEBIDL\] defines the following terms:
  - <a id="889e932f"></a>Exposed
  - <a id="4013a022"></a>this

## <a id="references"></a>References[](#references)

### <a id="normative"></a>Normative References[](#normative)

<a id="biblio-compat"></a><strong>\[COMPAT\]</strong>

Mike Taylor. [Compatibility Standard](https://compat.spec.whatwg.org/). Living Standard. URL: <https://compat.spec.whatwg.org/>

<a id="biblio-css-backgrounds-3"></a><strong>\[CSS-BACKGROUNDS-3\]</strong>

Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://drafts.csswg.org/css-backgrounds/). URL: <https://drafts.csswg.org/css-backgrounds/>

<a id="biblio-css-break-4"></a><strong>\[CSS-BREAK-4\]</strong>

Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://drafts.csswg.org/css-break-4/). URL: <https://drafts.csswg.org/css-break-4/>

<a id="biblio-css-cascade-5"></a><strong>\[CSS-CASCADE-5\]</strong>

Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://drafts.csswg.org/css-cascade-5/). URL: <https://drafts.csswg.org/css-cascade-5/>

<a id="biblio-css-cascade-6"></a><strong>\[CSS-CASCADE-6\]</strong>

Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://drafts.csswg.org/css-cascade-6/). URL: <https://drafts.csswg.org/css-cascade-6/>

<a id="biblio-css-color-4"></a><strong>\[CSS-COLOR-4\]</strong>

Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://drafts.csswg.org/css-color-4/). URL: <https://drafts.csswg.org/css-color-4/>

<a id="biblio-css-color-adjust-1"></a><strong>\[CSS-COLOR-ADJUST-1\]</strong>

Elika Etemad; et al. [CSS Color Adjustment Module Level 1](https://drafts.csswg.org/css-color-adjust-1/). URL: <https://drafts.csswg.org/css-color-adjust-1/>

<a id="biblio-css-content-3"></a><strong>\[CSS-CONTENT-3\]</strong>

Elika Etemad; Mike Bremford. [CSS Generated Content Module Level 3](https://drafts.csswg.org/css-content-3/). URL: <https://drafts.csswg.org/css-content-3/>

<a id="biblio-css-display-4"></a><strong>\[CSS-DISPLAY-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://drafts.csswg.org/css-display-4/). URL: <https://drafts.csswg.org/css-display-4/>

<a id="biblio-css-fonts-4"></a><strong>\[CSS-FONTS-4\]</strong>

Chris Lilley. [CSS Fonts Module Level 4](https://drafts.csswg.org/css-fonts-4/). URL: <https://drafts.csswg.org/css-fonts-4/>

<a id="biblio-css-highlight-api-1"></a><strong>\[CSS-HIGHLIGHT-API-1\]</strong>

Florian Rivoal; Sanket Joshi; Megan Gardner. [CSS Custom Highlight API Module Level 1](https://drafts.csswg.org/css-highlight-api-1/). URL: <https://drafts.csswg.org/css-highlight-api-1/>

<a id="biblio-css-inline-3"></a><strong>\[CSS-INLINE-3\]</strong>

Elika Etemad. [CSS Inline Layout Module Level 3](https://drafts.csswg.org/css-inline-3/). URL: <https://drafts.csswg.org/css-inline-3/>

<a id="biblio-css-lists-3"></a><strong>\[CSS-LISTS-3\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://drafts.csswg.org/css-lists-3/). URL: <https://drafts.csswg.org/css-lists-3/>

<a id="biblio-css-multicol-2"></a><strong>\[CSS-MULTICOL-2\]</strong>

Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 2](https://drafts.csswg.org/css-multicol-2/). URL: <https://drafts.csswg.org/css-multicol-2/>

<a id="biblio-css-position-4"></a><strong>\[CSS-POSITION-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 4](https://drafts.csswg.org/css-position-4/). URL: <https://drafts.csswg.org/css-position-4/>

<a id="biblio-css-ruby-1"></a><strong>\[CSS-RUBY-1\]</strong>

Elika Etemad; et al. [CSS Ruby Annotation Layout Module Level 1](https://drafts.csswg.org/css-ruby-1/). URL: <https://drafts.csswg.org/css-ruby-1/>

<a id="biblio-css-shadow-1"></a><strong>\[CSS-SHADOW-1\]</strong>

[CSS Shadow Module Level 1](https://drafts.csswg.org/css-shadow-1/). Editor's Draft. URL: <https://drafts.csswg.org/css-shadow-1/>

<a id="biblio-css-syntax-3"></a><strong>\[CSS-SYNTAX-3\]</strong>

Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/). URL: <https://drafts.csswg.org/css-syntax/>

<a id="biblio-css-tables-3"></a><strong>\[CSS-TABLES-3\]</strong>

Keith Cirkel. [CSS Table Module Level 3](https://drafts.csswg.org/css-tables-3/). URL: <https://drafts.csswg.org/css-tables-3/>

<a id="biblio-css-text-3"></a><strong>\[CSS-TEXT-3\]</strong>

Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://drafts.csswg.org/css-text-3/). URL: <https://drafts.csswg.org/css-text-3/>

<a id="biblio-css-text-4"></a><strong>\[CSS-TEXT-4\]</strong>

Elika Etemad; et al. [CSS Text Module Level 4](https://drafts.csswg.org/css-text-4/). URL: <https://drafts.csswg.org/css-text-4/>

<a id="biblio-css-text-decor-4"></a><strong>\[CSS-TEXT-DECOR-4\]</strong>

Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://drafts.csswg.org/css-text-decor-4/). URL: <https://drafts.csswg.org/css-text-decor-4/>

<a id="biblio-css-ui-4"></a><strong>\[CSS-UI-4\]</strong>

Tab Atkins Jr.; Florian Rivoal. [CSS Basic User Interface Module Level 4](https://drafts.csswg.org/css-ui-4/). URL: <https://drafts.csswg.org/css-ui-4/>

<a id="biblio-css-values-4"></a><strong>\[CSS-VALUES-4\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: <https://drafts.csswg.org/css-values-4/>

<a id="biblio-css-variables-1"></a><strong>\[CSS-VARIABLES-1\]</strong>

Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://drafts.csswg.org/css-variables/). URL: <https://drafts.csswg.org/css-variables/>

<a id="biblio-css-variables-2"></a><strong>\[CSS-VARIABLES-2\]</strong>

[CSS Custom Properties for Cascading Variables Module Level 2](https://drafts.csswg.org/css-variables-2/). Editor's Draft. URL: <https://drafts.csswg.org/css-variables-2/>

<a id="biblio-css-view-transitions-2"></a><strong>\[CSS-VIEW-TRANSITIONS-2\]</strong>

Noam Rosenthal; et al. [CSS View Transitions Module Level 2](https://drafts.csswg.org/css-view-transitions-2/). URL: <https://drafts.csswg.org/css-view-transitions-2/>

<a id="biblio-css-writing-modes-3"></a><strong>\[CSS-WRITING-MODES-3\]</strong>

Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://drafts.csswg.org/css-writing-modes-3/). URL: <https://drafts.csswg.org/css-writing-modes-3/>

<a id="biblio-css-writing-modes-4"></a><strong>\[CSS-WRITING-MODES-4\]</strong>

Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://drafts.csswg.org/css-writing-modes-4/). URL: <https://drafts.csswg.org/css-writing-modes-4/>

<a id="biblio-css2"></a><strong>\[CSS2\]</strong>

Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://drafts.csswg.org/css2/). URL: <https://drafts.csswg.org/css2/>

<a id="biblio-cssom-1"></a><strong>\[CSSOM-1\]</strong>

Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://drafts.csswg.org/cssom/). URL: <https://drafts.csswg.org/cssom/>

<a id="biblio-dom"></a><strong>\[DOM\]</strong>

Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: <https://dom.spec.whatwg.org/>

<a id="biblio-fill-stroke-3"></a><strong>\[FILL-STROKE-3\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Fill and Stroke Module Level 3](https://drafts.csswg.org/fill-stroke/). URL: <https://drafts.csswg.org/fill-stroke/>

<a id="biblio-html"></a><strong>\[HTML\]</strong>

Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: <https://html.spec.whatwg.org/multipage/>

<a id="biblio-rfc2119"></a><strong>\[RFC2119\]</strong>

S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: <https://datatracker.ietf.org/doc/html/rfc2119>

<a id="biblio-selectors-3"></a><strong>\[SELECTORS-3\]</strong>

Tantek Çelik; et al. [Selectors Level 3](https://drafts.csswg.org/selectors-3/). URL: <https://drafts.csswg.org/selectors-3/>

<a id="biblio-selectors-4"></a><strong>\[SELECTORS-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://drafts.csswg.org/selectors/). URL: <https://drafts.csswg.org/selectors/>

<a id="biblio-selectors-5"></a><strong>\[SELECTORS-5\]</strong>

Elika Etemad; Tab Atkins Jr.. [Selectors Level 5](https://drafts.csswg.org/selectors-5/). URL: <https://drafts.csswg.org/selectors-5/>

<a id="biblio-uax44"></a><strong>\[UAX44\]</strong>

Ken Whistler. [Unicode Character Database](https://www.unicode.org/reports/tr44/tr44-38.html). 2 September 2026. Unicode Standard Annex \#44. URL: <https://www.unicode.org/reports/tr44/tr44-38.html>

<a id="biblio-url"></a><strong>\[URL\]</strong>

Anne van Kesteren. [URL Standard](https://url.spec.whatwg.org/). Living Standard. URL: <https://url.spec.whatwg.org/>

<a id="biblio-webidl"></a><strong>\[WEBIDL\]</strong>

Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: <https://webidl.spec.whatwg.org/>

### <a id="informative"></a>Non-Normative References[](#informative)

<a id="biblio-css-backgrounds-4"></a><strong>\[CSS-BACKGROUNDS-4\]</strong>

Bert Bos; Elika J. Etemad; Lea Verou. [CSS Backgrounds and Borders Module Level 4](https://drafts.csswg.org/css-backgrounds-4/). Proposal for a CSS module. URL: <https://drafts.csswg.org/css-backgrounds-4/>

<a id="biblio-css-cascade-4"></a><strong>\[CSS-CASCADE-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://drafts.csswg.org/css-cascade-4/). URL: <https://drafts.csswg.org/css-cascade-4/>

<a id="biblio-css-contain-2"></a><strong>\[CSS-CONTAIN-2\]</strong>

Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://drafts.csswg.org/css-contain-2/). URL: <https://drafts.csswg.org/css-contain-2/>

<a id="biblio-css-display-3"></a><strong>\[CSS-DISPLAY-3\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://drafts.csswg.org/css-display/). URL: <https://drafts.csswg.org/css-display/>

<a id="biblio-css22"></a><strong>\[CSS22\]</strong>

Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://drafts.csswg.org/css2/). URL: <https://drafts.csswg.org/css2/>

<a id="biblio-html5"></a><strong>\[HTML5\]</strong>

Ian Hickson; et al. [HTML5](https://www.w3.org/TR/html5/). 27 March 2018. REC. URL: <https://www.w3.org/TR/html5/>

<a id="biblio-uax29"></a><strong>\[UAX29\]</strong>

Josh Hadley. [Unicode Text Segmentation](https://www.unicode.org/reports/tr29/tr29-49.html). 1 September 2026. Unicode Standard Annex \#29. URL: <https://www.unicode.org/reports/tr29/tr29-49.html>

<a id="biblio-web-animations-1"></a><strong>\[WEB-ANIMATIONS-1\]</strong>

Brian Birtles; et al. [Web Animations](https://drafts.csswg.org/web-animations-1/). URL: <https://drafts.csswg.org/web-animations-1/>

## <a id="idl-index"></a>IDL Index[](#idl-index)

``` text
[Exposed=Window]
interface CSSPseudoElement {
    readonly attribute CSSOMString type;
    readonly attribute Element element;
    readonly attribute (Element or CSSPseudoElement) parent;
    readonly attribute CSSOMString selectorText;
    CSSPseudoElement? pseudo(CSSOMString type);
};

partial interface Element {
  CSSPseudoElement? pseudo(CSSOMString type);
};
```

## <a id="issues-index"></a>Issues Index[](#issues-index)

Are there any other properties that should be included here?

<strong>Issue:</strong>

[↵](#issue-6ca196f5)

UA tweaks to the presentation of highlights in ways that

<strong>Issue:</strong>

<em>are</em> controlled by CSS are currently under discussion in [Issue 6853](https://github.com/w3c/csswg-drafts/issues/6853). [↵](#issue-c10df75c)

See

<strong>Issue:</strong>

[F2F minutes](http://lists.w3.org/Archives/Public/www-style/2008Nov/0022.html), [dbaron’s message](http://lists.w3.org/Archives/Public/www-style/2008Oct/0268.html), [Daniel’s thread](http://lists.w3.org/Archives/Public/www-style/2010May/0247.html), [Gecko notes](http://lists.w3.org/Archives/Public/www-style/2010May/0261.html), [Opera notes](http://lists.w3.org/Archives/Public/www-style/2010May/0366.html), [Webkit notes](http://lists.w3.org/Archives/Public/www-style/2010May/0280.html) [↵](#issue-13b80e20)

Not sure if this is the correct way of describing the way things work.

<strong>Issue:</strong>

[↵](#issue-271a1b90)

Interaction of

<strong>Issue:</strong>

[::marker](#selectordef-marker) and [::first-line](#selectordef-first-line) is currently under discussion in [Issue 4506](https://github.com/w3c/csswg-drafts/issues/4506). [↵](#issue-c80e7017)

Should

<strong>Issue:</strong>

[::marker](#selectordef-marker) also have [::prefix](#selectordef-first-letter-prefix) and [::suffix](#selectordef-first-letter-suffix) sub-elements? [↵](#issue-00545711)

Authors seem to want

<strong>Issue:</strong>

[text-align](https://drafts.csswg.org/css-text-3/#propdef-text-align) on the list of supported properties. See e.g. [comments here](http://css-tricks.com/almanac/selectors/p/placeholder/). [↵](#issue-1631f2fa)

This interface is under design development, and this draft is looking for feedback more than implementation. The CSSWG would particularly appreciate hearing about use cases and problems.

<strong>Issue:</strong>

[↵](#issue-22d457b7)

The identity, lifetime, and nullness of the return value (and potential error cases) of the

<strong>Issue:</strong>

<code>[pseudo()](#dom-csspseudoelement-pseudo)</code> method is still under discussion. See [Issue 3607](https://github.com/w3c/csswg-drafts/issues/3607) and [Issue 3603](https://github.com/w3c/csswg-drafts/issues/3603). [↵](#issue-302461e7)

<strong>⚠</strong>MDN

[CSSPseudoElement/element](https://developer.mozilla.org/en-US/docs/Web/API/CSSPseudoElement/element)

In only one current engine.

Firefox🔰 75+SafariNoneChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[CSSPseudoElement/type](https://developer.mozilla.org/en-US/docs/Web/API/CSSPseudoElement/type)

In only one current engine.

Firefox🔰 75+SafariNoneChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[CSSPseudoElement](https://developer.mozilla.org/en-US/docs/Web/API/CSSPseudoElement)

In only one current engine.

Firefox🔰 75+SafariNoneChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[::after](https://developer.mozilla.org/en-US/docs/Web/CSS/::after)

In all current engines.

Firefox1.5+Safari4+Chrome1+

------------------------------------------------------------------------

Opera7+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

[::before](https://developer.mozilla.org/en-US/docs/Web/CSS/::before)

In all current engines.

Firefox1.5+Safari4+Chrome1+

------------------------------------------------------------------------

Opera7+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android4+iOS Safari3+Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[::file-selector-button](https://developer.mozilla.org/en-US/docs/Web/CSS/::file-selector-button)

In all current engines.

Firefox82+Safari14.1+Chrome89+

------------------------------------------------------------------------

Opera?Edge89+

------------------------------------------------------------------------

Edge (Legacy)NoneIENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari14.5+Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[::first-letter](https://developer.mozilla.org/en-US/docs/Web/CSS/::first-letter)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera7+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[::first-line](https://developer.mozilla.org/en-US/docs/Web/CSS/::first-line)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera7+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>⚠</strong>MDN

[::grammar-error](https://developer.mozilla.org/en-US/docs/Web/CSS/::grammar-error)

In no current engines.

FirefoxNoneSafariNoneChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[::marker](https://developer.mozilla.org/en-US/docs/Web/CSS/::marker)

Firefox68+SafariNoneChrome86+

------------------------------------------------------------------------

Opera?Edge86+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[::placeholder](https://developer.mozilla.org/en-US/docs/Web/CSS/::placeholder)

In all current engines.

Firefox51+Safari10.1+Chrome57+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)NoneIENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView57+Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[::selection](https://developer.mozilla.org/en-US/docs/Web/CSS/::selection)

In all current engines.

Firefox62+Safari1.1+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS SafariNoneChrome for Android?Android WebView37+Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[::spelling-error](https://developer.mozilla.org/en-US/docs/Web/CSS/::spelling-error)

In no current engines.

FirefoxNoneSafariNoneChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[::target-text](https://developer.mozilla.org/en-US/docs/Web/CSS/::target-text)

In only one current engine.

FirefoxNoneSafariNoneChrome89+

------------------------------------------------------------------------

Opera?Edge89+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?
