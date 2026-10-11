Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Selectors Level 4](https://drafts.csswg.org/selectors-4/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply. The capture’s full original notice and links remain below.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md).

# Source provenance

Title: Selectors Level 4

Source snapshot: https://drafts.csswg.org/selectors-4/

Retrieved: 2026-10-10. The captured page states Editor’s Draft, 18 September 2026. An undated editor URL can change; the retrieval date and exact byte hash identify this captured HTML.

Captured HTML SHA-256: 7dddba84e895d869585244e571127f5a8bc4beac61c5cebd16809e71cee789f6

Captured HTML revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Immutable upstream source: [Overview.bs](https://raw.githubusercontent.com/w3c/csswg-drafts/50337328484f7fcdd9ab6fe9ded6011815b1707e/selectors-4/Overview.bs), SHA-256 `0db7958a1744d95f7998f2451684e2cf270670091c8cd14a7681ac53cf068098`. The source revision is inspected as reference material and is not executed.

Representation notes:

- Full format conversion of the captured HTML body, including status, metadata, bibliography, indexes, examples, test references, and legal notice. Script and style elements are omitted and were not executed.
- All source body IDs are retained, including those inside preformatted blocks, which are relocated immediately before those blocks. Fragment-only links stay local only when their exact captured target exists; other links remain upstream.
- All 1 tables have readable Markdown layouts. All tables have ordinary row/column layouts. Synthetic Field/Definition or Column N headings are non-normative. Source row headers remain bold; native HTML header/accessibility semantics are not expressible in GFM.
- Preformatted examples retain literal text. Single-line table examples use semantic inline code. Compiler-generated hyperlinks inside fenced code blocks are omitted while their IDs and visible code text are retained; other source links are preserved. Small semantic code, variable, emphasis, subscript, and superscript HTML remains where Markdown notation would alter text.
- Conversion uses Pandoc 3.1.11.1 with focused Python/lxml preparation and a semantic Lua filter; the parsed GFM output is checked against the captured HTML. The [conversion report](CONVERSION-REPORT.md) records provenance and check boundaries.
- This full edition is a selected planning source for unstarted selector execution and scope-cascade work. The [source-edition mapping](SOURCE-EDITIONS.md) distinguishes these planning selections from historical editions consumed by completed CSS work.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>Selectors Level 4

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 18 September 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/selectors/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/selectors-4/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2026/WD-selectors-4-20260122/>

<https://www.w3.org/TR/2022/WD-selectors-4-20221111/>

<https://www.w3.org/TR/2022/WD-selectors-4-20220507/>

<https://www.w3.org/TR/2018/WD-selectors-4-20181121/>

<https://www.w3.org/TR/2018/WD-selectors-4-20180202/>

<https://www.w3.org/TR/2018/WD-selectors-4-20180201/>

<https://www.w3.org/TR/2013/WD-selectors4-20130502/>

<https://www.w3.org/TR/2012/WD-selectors4-20120823/>

<https://www.w3.org/TR/2011/WD-selectors4-20110929/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/selectors-4)

[Inline In Spec](#issues-index)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Tab Atkins Jr.](http://xanthir.com/contact/) (Google)

<strong>Former Editors:</strong>

[Tantek Çelik](http://www.tantek.com)

Daniel Glazman

Ian Hickson

Peter Linss

John Williams

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/selectors-4/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/selectors/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

<a id="ref-for-selector"></a>[Selectors](#selector) are patterns that match against elements in a tree, and as such form one of several technologies that can be used to select nodes in a document. Selectors have been optimized for use with HTML and XML, and are designed to be usable in performance-critical code. They are a core component of CSS (Cascading Style Sheets), which uses Selectors to bind style properties to elements in the document. Selectors Level 4 describes the selectors that already exist in [\[SELECT\]](#biblio-select), and further introduces new selectors for CSS and other languages that may need them.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “selectors” in the title, like this: “\[selectors\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bselectors%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-user-action-pseudo-class"></a>[user action pseudo-classes](#user-action-pseudo-class) applying to non-<a id="ref-for-tree-abiding"></a>[tree-abiding pseudo-elements](https://drafts.csswg.org/css-pseudo-4/#tree-abiding)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](#context)
    1.  [1.1 Module Interactions](#placement)
2.  [2 Selectors Overview](#overview)
3.  [3 Selector Syntax and Structure](#syntax)
    1.  [3.1 Structure and Terminology](#structure)
    2.  [3.2 Data Model](#data-model)
        1.  [3.2.1 Featureless Elements](#featureless-elements)
    3.  [3.3 Scoped Selectors](#scoping)
    4.  [3.4 Relative Selectors](#relative)
    5.  [3.5 Pseudo-classes](#pseudo-classes)
    6.  [3.6 Pseudo-elements](#pseudo-elements)
        1.  [3.6.1 Syntax](#pseudo-element-syntax)
        2.  [3.6.2 Binding to the Document Tree](#pseudo-element-attachment)
        3.  [3.6.3 Pseudo-classing Pseudo-elements](#pseudo-element-states)
        4.  [3.6.4 Sub-pseudo-elements](#sub-pseudo-elements)
        5.  [3.6.5 Internal Structure](#pseudo-element-structure)
    7.  [3.7 Characters and case sensitivity](#case-sensitive)
    8.  [3.8 Declaring Namespace Prefixes](#namespaces)
    9.  [3.9 Invalid Selectors and Error Handling](#invalid)
    10. [3.10 Legacy Aliases](#legacy-aliasing)
4.  [4 Logical Combinations](#logical-combination)
    1.  [4.1 Selector Lists](#grouping)
    2.  [4.2 The Matches-Any Pseudo-class: :is()](#matches)
    3.  [4.3 The Negation (Matches-None) Pseudo-class: :not()](#negation)
    4.  [4.4 The Specificity-adjustment Pseudo-class: :where()](#zero-matches)
    5.  [4.5 The Relational Pseudo-class: :has()](#relational)
5.  [5 Elemental selectors](#elemental-selectors)
    1.  [5.1 Type (tag name) selector](#type-selectors)
    2.  [5.2 Universal selector ](#the-universal-selector)
    3.  [5.3 Namespaces in Elemental Selectors](#type-nmsp)
    4.  [5.4 The Defined Pseudo-class: :defined](#the-defined-pseudo)
6.  [6 Attribute selectors](#attribute-selectors)
    1.  [6.1 Attribute presence and value selectors](#attribute-representation)
    2.  [6.2 Substring matching attribute selectors](#attribute-substrings)
    3.  [6.3 Case-sensitivity](#attribute-case)
    4.  [6.4 Attribute selectors and namespaces](#attrnmsp)
    5.  [6.5 Default attribute values in DTDs](#def-values)
    6.  [6.6 Class selectors](#class-html)
    7.  [6.7 ID selectors](#id-selectors)
7.  [7 Linguistic Pseudo-classes](#linguistic-pseudos)
    1.  [7.1 The Directionality Pseudo-class: :dir()](#the-dir-pseudo)
    2.  [7.2 The Language Pseudo-class: :lang()](#the-lang-pseudo)
8.  [8 Location Pseudo-classes](#location)
    1.  [8.1 The Hyperlink Pseudo-class: :any-link](#the-any-link-pseudo)
    2.  [8.2 The Link History Pseudo-classes: :link and :visited](#link)
    3.  [8.3 The Target Pseudo-class: :target](#the-target-pseudo)
    4.  [8.4 The Reference Element Pseudo-class: :scope](#the-scope-pseudo)
9.  [9 User Action Pseudo-classes](#useraction-pseudos)
    1.  [9.1 The Pointer Hover Pseudo-class: :hover](#the-hover-pseudo)
    2.  [9.2 The Activation Pseudo-class: :active](#the-active-pseudo)
    3.  [9.3 The Input Focus Pseudo-class: :focus](#the-focus-pseudo)
    4.  [9.4 The Focus-Indicated Pseudo-class: :focus-visible](#the-focus-visible-pseudo)
    5.  [9.5 The Focus Container Pseudo-class: :focus-within](#the-focus-within-pseudo)
10. [10 Resource State Pseudo-classes](#resource-pseudos)
    1.  [10.1 Media Playback State: the :playing, :paused, and :seeking pseudo-classes](#video-state)
    2.  [10.2 Media Loading State: the :buffering and :stalled pseudo-classes](#media-loading-state)
    3.  [10.3 Sound State: the :muted and :volume-locked pseudo-classes](#sound-state)
11. [11 Element Display State Pseudo-classes](#display-state-pseudos)
    1.  [11.1 Collapse State: the :open pseudo-class](#open-state)
    2.  [11.2 Popover State: the :popover-open pseudo-class](#popover-open-state)
    3.  [11.3 Modal (Exclusive Interaction) State: the :modal pseudo-class](#modal-state)
    4.  [11.4 Fullscreen Presentation State: the :fullscreen pseudo-class](#fullscreen-state)
    5.  [11.5 Picture-in-Picture Presentation State: the :picture-in-picture pseudo-class](#pip-state)
12. [12 The Input Pseudo-classes](#input-pseudos)
    1.  [12.1 Input Control States](#input-states)
        1.  [12.1.1 The :enabled and :disabled Pseudo-classes](#enableddisabled)
        2.  [12.1.2 The Mutability Pseudo-classes: :read-only and :read-write](#rw-pseudos)
        3.  [12.1.3 The Placeholder-shown Pseudo-class: :placeholder-shown](#placeholder)
        4.  [12.1.4 The Automatic Input Pseudo-class: :autofill](#autofill)
        5.  [12.1.5 The Default-option Pseudo-class: :default](#the-default-pseudo)
    2.  [12.2 Input Value States](#input-value-states)
        1.  [12.2.1 The Selected-option Pseudo-classes: :checked, :unchecked, and :indeterminate](#checked)
    3.  [12.3 Input Value-checking](#ui-validity)
        1.  [12.3.1 The Validity Pseudo-classes: :valid and :invalid](#validity-pseudos)
        2.  [12.3.2 The Range Pseudo-classes: :in-range and :out-of-range](#range-pseudos)
        3.  [12.3.3 The Optionality Pseudo-classes: :required and :optional](#opt-pseudos)
        4.  [12.3.4 The User-interaction Pseudo-classes: :user-valid and :user-invalid](#user-pseudos)
13. [13 Tree-Structural pseudo-classes](#structural-pseudos)
    1.  [13.1 :root pseudo-class](#the-root-pseudo)
    2.  [13.2 :empty pseudo-class](#the-empty-pseudo)
    3.  [13.3 Child-indexed Pseudo-classes](#child-index)
        1.  [13.3.1 :nth-child() pseudo-class](#the-nth-child-pseudo)
        2.  [13.3.2 :nth-last-child() pseudo-class](#the-nth-last-child-pseudo)
        3.  [13.3.3 :first-child pseudo-class](#the-first-child-pseudo)
        4.  [13.3.4 :last-child pseudo-class](#the-last-child-pseudo)
        5.  [13.3.5 :only-child pseudo-class](#the-only-child-pseudo)
    4.  [13.4 Typed Child-indexed Pseudo-classes](#typed-child-index)
        1.  [13.4.1 :nth-of-type() pseudo-class](#the-nth-of-type-pseudo)
        2.  [13.4.2 :nth-last-of-type() pseudo-class](#the-nth-last-of-type-pseudo)
        3.  [13.4.3 :first-of-type pseudo-class](#the-first-of-type-pseudo)
        4.  [13.4.4 :last-of-type pseudo-class](#the-last-of-type-pseudo)
        5.  [13.4.5 :only-of-type pseudo-class](#the-only-of-type-pseudo)
14. [14 Combinators](#combinators)
    1.  [14.1 Descendant combinator (<code> </code>)](#descendant-combinators)
    2.  [14.2 Child combinator (<code>&#62;</code>)](#child-combinators)
    3.  [14.3 Next-sibling combinator (<code>+</code>)](#adjacent-sibling-combinators)
    4.  [14.4 Subsequent-sibling combinator (<code>&#126;</code>)](#general-sibling-combinators)
15. [15 Calculating a selector’s specificity](#specificity-rules)
16. [16 Grammar](#grammar)
    1.  [16.1 \<forgiving-selector-list\>](#forgiving-selector)
17. [17 API Hooks](#api-hooks)
    1.  [17.1 Parse A Selector](#parse-selector)
    2.  [17.2 Parse A Relative Selector](#parse-relative-selector)
    3.  [17.3 Match a Selector Against an Element](#match-against-element)
    4.  [17.4 Match a Selector Against a Pseudo-element](#match-against-pseudo-element)
    5.  [17.5 Match a Selector Against a Tree](#match-against-tree)
18. [ Appendix A: Guidance on Mapping Source Documents & Data to an Element Tree](#dom-mapping)
19. [ Appendix B: Obsolete but Required <code>-webkit-</code> Parsing Quirks for Web Compat](#compat)
20. [ Appendix C: Example Privacy-Preserving :visited Restrictions](#visited-privacy)
21. [18 Changes](#changes)
    1.  [18.1 Changes since the 22 January 2026 Working Draft ](#changes-2026-01)
    2.  [18.2 Changes since the 11 November 2022 Working Draft](#changes-2022-11)
    3.  [18.3 Changes since the 7 May 2022 Working Draft](#changes-2022-05)
    4.  [18.4 Changes since the 21 November 2018 Working Draft](#changes-2018-11)
    5.  [18.5 Changes since the 2 February 2018 Working Draft](#changes-2018-02)
    6.  [18.6 Changes since the 2 May 2013 Working Draft](#changes-2013)
    7.  [18.7 Changes since the 23 August 2012 Working Draft](#changes-2012)
    8.  [18.8 Changes since the 29 September 2011 Working Draft](#changes-2011)
    9.  [18.9 Changes Since Level 3](#changes-level-3)
        1.  [18.9.1 Conditional Logic](#changes-conditional)
        2.  [18.9.2 Interaction, Input and Forms](#changes-forms)
        3.  [18.9.3 Hyperlinking Support](#changes-hyperlinking)
        4.  [18.9.4 Clearer Expression of Authorial Intent](#changes-authorial)
        5.  [18.9.5 Media State](#changes-media)
        6.  [18.9.6 Internationalization Improvements](#changes-i18n)
22. [19 Acknowledgements](#acknowledgements)
23. [ Privacy Considerations](#privacy)
24. [ Security Considerations](#security)
25. [ Conformance](#w3c-conformance)
    1.  [ Document conventions](#w3c-conventions)
    2.  [ Conformance classes](#w3c-conformance-classes)
    3.  [ Partial implementations](#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](#w3c-testing)
26. [ Index](#index)
    1.  [ Terms defined by this specification](#index-defined-here)
    2.  [ Terms defined by reference](#index-defined-elsewhere)
27. [ References](#references)
    1.  [ Normative References](#normative)
    2.  [ Non-Normative References](#informative)
28. [ Issues Index](#issues-index)

## <a id="context"></a>1.  Introduction[](#context)

<em>This section is not normative.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

A <a id="ref-for-selector①"></a>[selector](#selector) is a boolean predicate that takes an element in a tree structure and tests whether the element matches the selector or not.

These expressions may be used for many things:

- directly on an element to test whether it matches some criteria, such as in the <code>element&#46;matches()</code> function defined in [\[DOM\]](#biblio-dom)
- applied to an entire tree of elements to filter it into a set of elements that match the criteria, such as in the <code>document&#46;querySelectorAll()</code> function defined in [\[DOM\]](#biblio-dom) or the selector of a CSS style rule.
- used "in reverse" to generate markup that would match a given selector, such as in [HAML](http://haml.info/) or [Emmet](https://en.wikipedia.org/wiki/Emmet_(software)) .

Selectors Levels 1, 2, and 3 are defined as the subsets of selector functionality defined in the [CSS1](https://www.w3.org/TR/REC-CSS1), [CSS2.1](https://www.w3.org/TR/CSS21/), and [Selectors Level 3](https://www.w3.org/TR/css3-selectors/) specifications, respectively. This module defines Selectors Level 4.

### <a id="placement"></a>1.1. Module Interactions[](#placement)

Tests

Tests not needed for this section.

------------------------------------------------------------------------

This module replaces the definitions of and extends the set of selectors defined for CSS in [\[SELECT\]](#biblio-select) and [\[CSS21\]](#biblio-css21).

Pseudo-element selectors, which define abstract elements in a rendering tree, are not part of this specification: their generic syntax is described here, but, due to their close integration with the rendering model and irrelevance to other uses such as DOM queries, they will be defined in other modules.

## <a id="overview"></a>2.  Selectors Overview[](#overview)

<em>This section is non-normative, as it merely summarizes the following sections.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

A selector represents a structure. This structure can be used as a condition (e.g. in a CSS rule) that determines which elements a selector matches in the document tree, or as a flat description of the HTML or XML fragment corresponding to that structure.

Selectors may range from simple element names to rich contextual representations.

The following table summarizes the Selector syntax:

<a id="selector-examples"></a>

<table>
<thead>
<tr class="header">
<th>Pattern</th>
<th>Represents</th>
<th>Section</th>
<th>Level</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td><code>&#42;</code></td>
<td>any element</td>
<td><a href="#the-universal-selector">§ 5.2 Universal selector</a></td>
<td>2</td>
</tr>
<tr class="even">
<td><code>E</code></td>
<td>an element of type E</td>
<td><a href="#type-selectors">§ 5.1 Type (tag name) selector</a></td>
<td>1</td>
</tr>
</tbody>
<tbody>
<tr class="odd">
<td><code>E:not(<var>s1</var>, <var>s2</var>, …)</code></td>
<td>an E element that does not match either <a id="ref-for-compound"></a><a href="#compound">compound selector</a> <var>s1</var> or <a id="ref-for-compound①"></a>compound selector <var>s2</var></td>
<td><a href="#negation">§ 4.3 The Negation (Matches-None) Pseudo-class: :not()</a></td>
<td>3/4</td>
</tr>
<tr class="even">
<td><code>E:is(<var>s1</var>, <var>s2</var>, …)</code></td>
<td>an E element that matches <a id="ref-for-compound②"></a><a href="#compound">compound selector</a> <var>s1</var> and/or <a id="ref-for-compound③"></a>compound selector <var>s2</var></td>
<td><a href="#matches">§ 4.2 The Matches-Any Pseudo-class: :is()</a></td>
<td>4</td>
</tr>
<tr class="odd">
<td><code>E:where(<var>s1</var>, <var>s2</var>, …)</code></td>
<td>an E element that matches <a id="ref-for-compound④"></a><a href="#compound">compound selector</a> <var>s1</var> and/or <a id="ref-for-compound⑤"></a>compound selector <var>s2</var> but contributes no specificity.</td>
<td><a href="#zero-matches">§ 4.4 The Specificity-adjustment Pseudo-class: :where()</a></td>
<td>4</td>
</tr>
<tr class="even">
<td><code>E:has(<var>rs1</var>, <var>rs2</var>, …)</code></td>
<td>an E element, if there exists an element that matches either of the <a id="ref-for-relative-selector"></a><a href="#relative-selector">relative selectors</a> <var>rs1</var> or <var>rs2</var>, when evaluated with E as the <a id="ref-for-relative-selector-anchor-elements"></a><a href="#relative-selector-anchor-elements">anchor elements</a></td>
<td><a href="#relational">§ 4.5 The Relational Pseudo-class: :has()</a></td>
<td>4</td>
</tr>
</tbody>
<tbody>
<tr class="odd">
<td><code>E&#46;warning</code></td>
<td>an E element belonging to the class <code>warning</code> (the document language specifies how class is determined).</td>
<td><a href="#class-html">§ 6.6 Class selectors</a></td>
<td>1</td>
</tr>
<tr class="even">
<td><code>E&#35;myid</code></td>
<td>an E element with ID equal to <code>myid</code>.</td>
<td><a href="#id-selectors">§ 6.7 ID selectors</a></td>
<td>1</td>
</tr>
<tr class="odd">
<td><code>E&#91;foo&#93;</code></td>
<td>an E element with a <code>foo</code> attribute</td>
<td><a href="#attribute-representation">§ 6.1 Attribute presence and value selectors</a></td>
<td>2</td>
</tr>
<tr class="even">
<td><code>E&#91;foo=&#34;bar&#34;&#93;</code></td>
<td>an E element whose <code>foo</code> attribute value is exactly equal to <code>bar</code></td>
<td><a href="#attribute-representation">§ 6.1 Attribute presence and value selectors</a></td>
<td>2</td>
</tr>
<tr class="odd">
<td><code>E&#91;foo=&#34;bar&#34; i&#93;</code></td>
<td>an E element whose <code>foo</code> attribute value is exactly equal to any (ASCII-range) case-permutation of <code>bar</code></td>
<td><a href="#attribute-case">§ 6.3 Case-sensitivity</a></td>
<td>4</td>
</tr>
<tr class="even">
<td><code>E&#91;foo=&#34;bar&#34; s&#93;</code></td>
<td>an E element whose <code>foo</code> attribute value is <a id="ref-for-is"></a><a href="https://infra.spec.whatwg.org/#is">identical to</a> <code>bar</code></td>
<td><a href="#attribute-case">§ 6.3 Case-sensitivity</a></td>
<td>4</td>
</tr>
<tr class="odd">
<td><code>E&#91;foo&#126;=&#34;bar&#34;&#93;</code></td>
<td>an E element whose <code>foo</code> attribute value is a list of whitespace-separated values, one of which is exactly equal to <code>bar</code></td>
<td><a href="#attribute-representation">§ 6.1 Attribute presence and value selectors</a></td>
<td>2</td>
</tr>
<tr class="even">
<td><code>E&#91;foo^=&#34;bar&#34;&#93;</code></td>
<td>an E element whose <code>foo</code> attribute value begins exactly with the string <code>bar</code></td>
<td><a href="#attribute-substrings">§ 6.2 Substring matching attribute selectors</a></td>
<td>3</td>
</tr>
<tr class="odd">
<td><code>E&#91;foo$=&#34;bar&#34;&#93;</code></td>
<td>an E element whose <code>foo</code> attribute value ends exactly with the string <code>bar</code></td>
<td><a href="#attribute-substrings">§ 6.2 Substring matching attribute selectors</a></td>
<td>3</td>
</tr>
<tr class="even">
<td><code>E&#91;foo&#42;=&#34;bar&#34;&#93;</code></td>
<td>an E element whose <code>foo</code> attribute value contains the substring <code>bar</code></td>
<td><a href="#attribute-substrings">§ 6.2 Substring matching attribute selectors</a></td>
<td>3</td>
</tr>
<tr class="odd">
<td><code>E&#91;foo&#124;=&#34;en&#34;&#93;</code></td>
<td>an E element whose <code>foo</code> attribute value is a hyphen-separated list of values beginning with <code>en</code></td>
<td><a href="#attribute-representation">§ 6.1 Attribute presence and value selectors</a></td>
<td>2</td>
</tr>
</tbody>
<tbody>
<tr class="odd">
<td><code>E&#58;dir(ltr)</code></td>
<td>an element of type E with left-to-right directionality (the document language specifies how directionality is determined)</td>
<td><a href="#the-dir-pseudo">§ 7.1 The Directionality Pseudo-class: :dir()</a></td>
<td>4</td>
</tr>
<tr class="even">
<td><code>E&#58;lang(sr, &#34;&#42;-Cyrl&#34;)</code></td>
<td>an element of type E tagged as being in Serbian, (which typically uses Cyrillic, but can be written with Latin characters) but also anything written with Cyrillic characters</td>
<td><a href="#the-lang-pseudo">§ 7.2 The Language Pseudo-class: :lang()</a></td>
<td>2/4</td>
</tr>
</tbody>
<tbody>
<tr class="odd">
<td><code>E&#58;any-link</code></td>
<td>an E element being the source anchor of a hyperlink</td>
<td><a href="#the-any-link-pseudo">§ 8.1 The Hyperlink Pseudo-class: :any-link</a></td>
<td>4</td>
</tr>
<tr class="even">
<td><code>E&#58;link</code></td>
<td>an E element being the source anchor of a hyperlink of which the target is not yet visited</td>
<td><a href="#link">§ 8.2 The Link History Pseudo-classes: :link and :visited</a></td>
<td>1</td>
</tr>
<tr class="odd">
<td><code>E&#58;visited</code></td>
<td>an E element being the source anchor of a hyperlink of which the target is already visited</td>
<td><a href="#link">§ 8.2 The Link History Pseudo-classes: :link and :visited</a></td>
<td>1</td>
</tr>
<tr class="even">
<td><code>E&#58;target</code></td>
<td>an E element being the target of the current URL</td>
<td><a href="#the-target-pseudo">§ 8.3 The Target Pseudo-class: :target</a></td>
<td>3</td>
</tr>
<tr class="odd">
<td><code>E&#58;scope</code></td>
<td>an E element being a <a id="ref-for-scoping-root"></a><a href="#scoping-root">scoping root</a></td>
<td><a href="#the-scope-pseudo">§ 8.4 The Reference Element Pseudo-class: :scope</a></td>
<td>4</td>
</tr>
</tbody>
<tbody>
<tr class="odd">
<td><code>E&#58;active</code></td>
<td>an E element that is in an activated state</td>
<td><a href="#the-active-pseudo">§ 9.2 The Activation Pseudo-class: :active</a></td>
<td>1</td>
</tr>
<tr class="even">
<td><code>E&#58;hover</code></td>
<td>an E element that is under the cursor, or that has a descendant under the cursor</td>
<td><a href="#the-hover-pseudo">§ 9.1 The Pointer Hover Pseudo-class: :hover</a></td>
<td>2</td>
</tr>
<tr class="odd">
<td><code>E&#58;focus</code></td>
<td>an E element that has user input focus</td>
<td><a href="#the-focus-pseudo">§ 9.3 The Input Focus Pseudo-class: :focus</a></td>
<td>2</td>
</tr>
<tr class="even">
<td><code>E&#58;focus-within</code></td>
<td>an E element that has user input focus or contains an element that has input focus.</td>
<td><a href="#the-focus-within-pseudo">§ 9.5 The Focus Container Pseudo-class: :focus-within</a></td>
<td>4</td>
</tr>
<tr class="odd">
<td><code>E&#58;focus-visible</code></td>
<td>an E element that has user input focus, and the UA has determined that a focus ring or other indicator should be drawn for that element</td>
<td><a href="#the-focus-visible-pseudo">§ 9.4 The Focus-Indicated Pseudo-class: :focus-visible</a></td>
<td>4</td>
</tr>
</tbody>
<tbody>
<tr class="odd">
<td><code>E&#58;enabled<br />E&#58;disabled</code></td>
<td>a user interface element E that is enabled or disabled, respectively</td>
<td><a href="#enableddisabled">§ 12.1.1 The :enabled and :disabled Pseudo-classes</a></td>
<td>3</td>
</tr>
<tr class="even">
<td><code>E&#58;read-write</code><br /><code>E&#58;read-only</code></td>
<td>a user interface element E that is user alterable, or not</td>
<td><a href="#rw-pseudos">§ 12.1.2 The Mutability Pseudo-classes: :read-only and :read-write</a></td>
<td>3-UI/4</td>
</tr>
<tr class="odd">
<td><code>E&#58;placeholder-shown</code></td>
<td>an input control currently showing placeholder text</td>
<td><a href="#placeholder">§ 12.1.3 The Placeholder-shown Pseudo-class: :placeholder-shown</a></td>
<td>3-UI/4</td>
</tr>
<tr class="even">
<td><code>E&#58;default</code></td>
<td>a user interface element E that is the default item in a group of related choices</td>
<td><a href="#the-default-pseudo">§ 12.1.5 The Default-option Pseudo-class: :default</a></td>
<td>3-UI/4</td>
</tr>
<tr class="odd">
<td><code>E&#58;checked</code><br /><code>E&#58;unchecked</code><br /><code>E&#58;indeterminate</code></td>
<td>a user interface element E that is checked/selected (for instance a radio-button or checkbox), unchecked, or in an indeterminate state (neither checked nor unchecked)</td>
<td><a href="#input-value-states">§ 12.2 Input Value States</a></td>
<td>3</td>
</tr>
<tr class="even">
<td><code>E&#58;valid</code><br /><code>E&#58;invalid</code></td>
<td>a user-input element E that meets, or doesn’t, its data validity semantics</td>
<td><a href="#validity-pseudos">§ 12.3.1 The Validity Pseudo-classes: :valid and :invalid</a></td>
<td>3-UI/4</td>
</tr>
<tr class="odd">
<td><code>E&#58;in-range</code><br /><code>E&#58;out-of-range</code></td>
<td>a user-input element E whose value is in-range/out-of-range</td>
<td><a href="#range-pseudos">§ 12.3.2 The Range Pseudo-classes: :in-range and :out-of-range</a></td>
<td>3-UI/4</td>
</tr>
<tr class="even">
<td><code>E&#58;required</code><br /><code>E&#58;optional</code></td>
<td>a user-input element E that requires/does not require input</td>
<td><a href="#opt-pseudos">§ 12.3.3 The Optionality Pseudo-classes: :required and :optional</a></td>
<td>3-UI/4</td>
</tr>
<tr class="odd">
<td><code>E&#58;user-invalid</code></td>
<td>a user-altered user-input element E with incorrect input (invalid, out-of-range, omitted-but-required)</td>
<td><a href="#user-pseudos">§ 12.3.4 The User-interaction Pseudo-classes: :user-valid and :user-invalid</a></td>
<td>4</td>
</tr>
</tbody>
<tbody>
<tr class="odd">
<td><code>E&#58;root</code></td>
<td>an E element, root of the document</td>
<td><a href="#the-root-pseudo">§ 13.1 :root pseudo-class</a></td>
<td>3</td>
</tr>
<tr class="even">
<td><code>E&#58;empty</code></td>
<td>an E element that has no children (neither elements nor text) except perhaps white space</td>
<td><a href="#the-empty-pseudo">§ 13.2 :empty pseudo-class</a></td>
<td>3</td>
</tr>
<tr class="odd">
<td><code>E:nth-child(<var>n</var> [of <var>S</var>]?)</code></td>
<td>an E element, the <var>n</var>-th child of its parent matching <var>S</var></td>
<td><a href="#the-nth-child-pseudo">§ 13.3.1 :nth-child() pseudo-class</a></td>
<td>3/4</td>
</tr>
<tr class="even">
<td><code>E:nth-last-child(<var>n</var> [of <var>S</var>]?)</code></td>
<td>an E element, the <var>n</var>-th child of its parent matching <var>S</var>, counting from the last one</td>
<td><a href="#the-nth-last-child-pseudo">§ 13.3.2 :nth-last-child() pseudo-class</a></td>
<td>3/4</td>
</tr>
<tr class="odd">
<td><code>E&#58;first-child</code></td>
<td>an E element, first child of its parent</td>
<td><a href="#the-first-child-pseudo">§ 13.3.3 :first-child pseudo-class</a></td>
<td>2</td>
</tr>
<tr class="even">
<td><code>E&#58;last-child</code></td>
<td>an E element, last child of its parent</td>
<td><a href="#the-last-child-pseudo">§ 13.3.4 :last-child pseudo-class</a></td>
<td>3</td>
</tr>
<tr class="odd">
<td><code>E&#58;only-child</code></td>
<td>an E element, only child of its parent</td>
<td><a href="#the-only-child-pseudo">§ 13.3.5 :only-child pseudo-class</a></td>
<td>3</td>
</tr>
<tr class="even">
<td><code>E:nth-of-type(<var>n</var>)</code></td>
<td>an E element, the <var>n</var>-th sibling of its type</td>
<td><a href="#the-nth-of-type-pseudo">§ 13.4.1 :nth-of-type() pseudo-class</a></td>
<td>3</td>
</tr>
<tr class="odd">
<td><code>E:nth-last-of-type(<var>n</var>)</code></td>
<td>an E element, the <var>n</var>-th sibling of its type, counting from the last one</td>
<td><a href="#the-nth-last-of-type-pseudo">§ 13.4.2 :nth-last-of-type() pseudo-class</a></td>
<td>3</td>
</tr>
<tr class="even">
<td><code>E&#58;first-of-type</code></td>
<td>an E element, first sibling of its type</td>
<td><a href="#the-first-of-type-pseudo">§ 13.4.3 :first-of-type pseudo-class</a></td>
<td>3</td>
</tr>
<tr class="odd">
<td><code>E&#58;last-of-type</code></td>
<td>an E element, last sibling of its type</td>
<td><a href="#the-last-of-type-pseudo">§ 13.4.4 :last-of-type pseudo-class</a></td>
<td>3</td>
</tr>
<tr class="even">
<td><code>E&#58;only-of-type</code></td>
<td>an E element, only sibling of its type</td>
<td><a href="#the-only-of-type-pseudo">§ 13.4.5 :only-of-type pseudo-class</a></td>
<td>3</td>
</tr>
</tbody>
<tbody>
<tr class="odd">
<td><code>E F</code></td>
<td>an F element descendant of an E element</td>
<td><a href="#descendant-combinators">§ 14.1 Descendant combinator ( )</a></td>
<td>1</td>
</tr>
<tr class="even">
<td><code>E &#62; F</code></td>
<td>an F element child of an E element</td>
<td><a href="#child-combinators">§ 14.2 Child combinator (&gt;)</a></td>
<td>2</td>
</tr>
<tr class="odd">
<td><code>E + F</code></td>
<td>an F element immediately preceded by an E element</td>
<td><a href="#adjacent-sibling-combinators">§ 14.3 Next-sibling combinator (+)</a></td>
<td>2</td>
</tr>
<tr class="even">
<td><code>E &#126; F</code></td>
<td>an F element preceded by an E element</td>
<td><a href="#general-sibling-combinators">§ 14.4 Subsequent-sibling combinator (~)</a></td>
<td>3</td>
</tr>
</tbody>
</table>

Note: Some Level 4 selectors (noted above as "3-UI") were introduced in [\[CSS3UI\]](#biblio-css3ui).

Tests

- [css3-modsel-1.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-1.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-1.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-1.xml)
- [css3-modsel-10.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-10.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-10.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-10.xml)
- [css3-modsel-100.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-100.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-100.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-100.xml)
- [css3-modsel-100b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-100b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-100b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-100b.xml)
- [css3-modsel-101.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-101.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-101.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-101.xml)
- [css3-modsel-101b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-101b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-101b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-101b.xml)
- [css3-modsel-102.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-102.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-102.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-102.xml)
- [css3-modsel-102b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-102b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-102b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-102b.xml)
- [css3-modsel-103.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-103.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-103.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-103.xml)
- [css3-modsel-103b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-103b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-103b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-103b.xml)
- [css3-modsel-104.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-104.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-104.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-104.xml)
- [css3-modsel-104b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-104b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-104b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-104b.xml)
- [css3-modsel-105.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-105.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-105.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-105.xml)
- [css3-modsel-105b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-105b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-105b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-105b.xml)
- [css3-modsel-106.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-106.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-106.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-106.xml)
- [css3-modsel-106b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-106b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-106b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-106b.xml)
- [css3-modsel-107.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-107.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-107.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-107.xml)
- [css3-modsel-107b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-107b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-107b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-107b.xml)
- [css3-modsel-108.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-108.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-108.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-108.xml)
- [css3-modsel-108b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-108b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-108b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-108b.xml)
- [css3-modsel-109.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-109.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-109.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-109.xml)
- [css3-modsel-109b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-109b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-109b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-109b.xml)
- [css3-modsel-11.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-11.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-11.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-11.xml)
- [css3-modsel-110.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-110.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-110.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-110.xml)
- [css3-modsel-110b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-110b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-110b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-110b.xml)
- [css3-modsel-111.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-111.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-111.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-111.xml)
- [css3-modsel-111b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-111b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-111b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-111b.xml)
- [css3-modsel-112.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-112.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-112.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-112.xml)
- [css3-modsel-112b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-112b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-112b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-112b.xml)
- [css3-modsel-113.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-113.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-113.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-113.xml)
- [css3-modsel-113b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-113b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-113b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-113b.xml)
- [css3-modsel-114.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-114.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-114.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-114.xml)
- [css3-modsel-114b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-114b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-114b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-114b.xml)
- [css3-modsel-115.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-115.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-115.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-115.xml)
- [css3-modsel-115b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-115b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-115b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-115b.xml)
- [css3-modsel-116.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-116.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-116.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-116.xml)
- [css3-modsel-116b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-116b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-116b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-116b.xml)
- [css3-modsel-117.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-117.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-117.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-117.xml)
- [css3-modsel-117b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-117b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-117b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-117b.xml)
- [css3-modsel-118.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-118.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-118.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-118.xml)
- [css3-modsel-119.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-119.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-119.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-119.xml)
- [css3-modsel-120.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-120.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-120.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-120.xml)
- [css3-modsel-121.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-121.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-121.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-121.xml)
- [css3-modsel-122.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-122.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-122.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-122.xml)
- [css3-modsel-123.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-123.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-123.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-123.xml)
- [css3-modsel-123b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-123b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-123b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-123b.xml)
- [css3-modsel-124.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-124.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-124.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-124.xml)
- [css3-modsel-124b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-124b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-124b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-124b.xml)
- [css3-modsel-125.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-125.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-125.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-125.xml)
- [css3-modsel-125b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-125b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-125b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-125b.xml)
- [css3-modsel-126.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-126.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-126.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-126.xml)
- [css3-modsel-126b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-126b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-126b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-126b.xml)
- [css3-modsel-127.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-127.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-127.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-127.xml)
- [css3-modsel-127b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-127b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-127b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-127b.xml)
- [css3-modsel-128.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-128.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-128.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-128.xml)
- [css3-modsel-128b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-128b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-128b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-128b.xml)
- [css3-modsel-129.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-129.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-129.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-129.xml)
- [css3-modsel-129b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-129b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-129b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-129b.xml)
- [css3-modsel-13.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-13.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-13.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-13.xml)
- [css3-modsel-130.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-130.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-130.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-130.xml)
- [css3-modsel-130b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-130b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-130b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-130b.xml)
- [css3-modsel-131.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-131.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-131.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-131.xml)
- [css3-modsel-131b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-131b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-131b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-131b.xml)
- [css3-modsel-132.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-132.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-132.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-132.xml)
- [css3-modsel-132b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-132b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-132b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-132b.xml)
- [css3-modsel-133.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-133.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-133.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-133.xml)
- [css3-modsel-133b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-133b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-133b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-133b.xml)
- [css3-modsel-134.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-134.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-134.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-134.xml)
- [css3-modsel-134b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-134b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-134b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-134b.xml)
- [css3-modsel-135.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-135.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-135.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-135.xml)
- [css3-modsel-135b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-135b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-135b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-135b.xml)
- [css3-modsel-136.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-136.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-136.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-136.xml)
- [css3-modsel-136b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-136b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-136b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-136b.xml)
- [css3-modsel-137.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-137.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-137.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-137.xml)
- [css3-modsel-137b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-137b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-137b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-137b.xml)
- [css3-modsel-138.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-138.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-138.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-138.xml)
- [css3-modsel-138b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-138b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-138b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-138b.xml)
- [css3-modsel-139.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-139.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-139.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-139.xml)
- [css3-modsel-139b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-139b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-139b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-139b.xml)
- [css3-modsel-14.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-14.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-14.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-14.xml)
- [css3-modsel-140.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-140.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-140.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-140.xml)
- [css3-modsel-140b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-140b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-140b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-140b.xml)
- [css3-modsel-141.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-141.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-141.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-141.xml)
- [css3-modsel-141b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-141b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-141b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-141b.xml)
- [css3-modsel-142.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-142.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-142.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-142.xml)
- [css3-modsel-142b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-142b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-142b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-142b.xml)
- [css3-modsel-143.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-143.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-143.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-143.xml)
- [css3-modsel-143b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-143b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-143b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-143b.xml)
- [css3-modsel-144.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-144.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-144.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-144.xml)
- [css3-modsel-145a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-145a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-145a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-145a.xml)
- [css3-modsel-145b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-145b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-145b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-145b.xml)
- [css3-modsel-146a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-146a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-146a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-146a.xml)
- [css3-modsel-146b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-146b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-146b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-146b.xml)
- [css3-modsel-147a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-147a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-147a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-147a.xml)
- [css3-modsel-147b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-147b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-147b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-147b.xml)
- [css3-modsel-148.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-148.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-148.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-148.xml)
- [css3-modsel-149.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-149.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-149.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-149.xml)
- [css3-modsel-149b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-149b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-149b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-149b.xml)
- [css3-modsel-14b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-14b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-14b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-14b.xml)
- [css3-modsel-14c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-14c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-14c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-14c.xml)
- [css3-modsel-14d.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-14d.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-14d.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-14d.xml)
- [css3-modsel-14e.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-14e.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-14e.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-14e.xml)
- [css3-modsel-15.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-15.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-15.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-15.xml)
- [css3-modsel-150.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-150.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-150.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-150.xml)
- [css3-modsel-151.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-151.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-151.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-151.xml)
- [css3-modsel-152.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-152.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-152.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-152.xml)
- [css3-modsel-153.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-153.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-153.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-153.xml)
- [css3-modsel-154.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-154.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-154.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-154.xml)
- [css3-modsel-155.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-155.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-155.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-155.xml)
- [css3-modsel-155a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-155a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-155a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-155a.xml)
- [css3-modsel-155b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-155b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-155b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-155b.xml)
- [css3-modsel-155c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-155c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-155c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-155c.xml)
- [css3-modsel-155d.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-155d.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-155d.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-155d.xml)
- [css3-modsel-156.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-156.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-156.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-156.xml)
- [css3-modsel-156b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-156b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-156b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-156b.xml)
- [css3-modsel-156c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-156c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-156c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-156c.xml)
- [css3-modsel-157.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-157.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-157.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-157.xml)
- [css3-modsel-158.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-158.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-158.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-158.xml)
- [css3-modsel-159.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-159.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-159.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-159.xml)
- [css3-modsel-15b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-15b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-15b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-15b.xml)
- css3-modsel-16.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-16.xml)
- [css3-modsel-160.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-160.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-160.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-160.xml)
- [css3-modsel-161.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-161.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-161.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-161.xml)
- [css3-modsel-166.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-166.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-166.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-166.xml)
- [css3-modsel-166a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-166a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-166a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-166a.xml)
- [css3-modsel-167.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-167.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-167.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-167.xml)
- [css3-modsel-167a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-167a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-167a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-167a.xml)
- [css3-modsel-168.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-168.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-168.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-168.xml)
- [css3-modsel-168a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-168a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-168a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-168a.xml)
- [css3-modsel-169.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-169.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-169.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-169.xml)
- [css3-modsel-169a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-169a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-169a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-169a.xml)
- css3-modsel-17.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-17.xml)
- [css3-modsel-170.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-170.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-170.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-170.xml)
- [css3-modsel-170a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-170a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-170a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-170a.xml)
- [css3-modsel-170b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-170b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-170b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-170b.xml)
- [css3-modsel-170c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-170c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-170c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-170c.xml)
- [css3-modsel-170d.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-170d.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-170d.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-170d.xml)
- [css3-modsel-171.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-171.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-171.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-171.xml)
- [css3-modsel-172a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-172a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-172a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-172a.xml)
- [css3-modsel-172b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-172b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-172b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-172b.xml)
- [css3-modsel-173a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-173a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-173a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-173a.xml)
- [css3-modsel-173b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-173b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-173b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-173b.xml)
- [css3-modsel-174a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-174a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-174a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-174a.xml)
- [css3-modsel-174b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-174b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-174b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-174b.xml)
- [css3-modsel-175a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-175a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-175a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-175a.xml)
- [css3-modsel-175b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-175b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-175b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-175b.xml)
- [css3-modsel-175c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-175c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-175c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-175c.xml)
- [css3-modsel-176.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-176.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-176.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-176.xml)
- [css3-modsel-177a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-177a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-177a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-177a.xml)
- [css3-modsel-177b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-177b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-177b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-177b.xml)
- [css3-modsel-178.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-178.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-178.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-178.xml)
- [css3-modsel-179.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-179.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-179.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-179.xml)
- [css3-modsel-179a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-179a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-179a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-179a.xml)
- [css3-modsel-18.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-18.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-18.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-18.xml)
- [css3-modsel-180a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-180a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-180a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-180a.xml)
- [css3-modsel-181.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-181.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-181.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-181.xml)
- [css3-modsel-182.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-182.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-182.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-182.xml)
- [css3-modsel-183.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-183.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-183.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-183.xml)
- [css3-modsel-184a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184a.xml)
- [css3-modsel-184b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184b.xml)
- [css3-modsel-184c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184c.xml)
- [css3-modsel-184d.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184d.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184d.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184d.xml)
- [css3-modsel-184e.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184e.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184e.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184e.xml)
- [css3-modsel-184f.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184f.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184f.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184f.xml)
- [css3-modsel-18a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-18a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-18a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-18a.xml)
- [css3-modsel-18b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-18b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-18b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-18b.xml)
- [css3-modsel-18c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-18c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-18c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-18c.xml)
- [css3-modsel-19.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-19.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-19.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-19.xml)
- [css3-modsel-19b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-19b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-19b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-19b.xml)
- [css3-modsel-2.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-2.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-2.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-2.xml)
- [css3-modsel-20.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-20.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-20.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-20.xml)
- [css3-modsel-21.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-21.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-21.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-21.xml)
- [css3-modsel-21b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-21b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-21b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-21b.xml)
- [css3-modsel-21c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-21c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-21c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-21c.xml)
- [css3-modsel-22.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-22.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-22.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-22.xml)
- [css3-modsel-25.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-25.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-25.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-25.xml)
- [css3-modsel-27.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-27.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-27.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-27.xml)
- css3-modsel-27a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-27a.xml)
- [css3-modsel-27b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-27b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-27b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-27b.xml)
- [css3-modsel-28.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-28.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-28.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-28.xml)
- [css3-modsel-28b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-28b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-28b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-28b.xml)
- [css3-modsel-29.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-29.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-29.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-29.xml)
- [css3-modsel-29b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-29b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-29b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-29b.xml)
- [css3-modsel-3.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-3.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-3.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-3.xml)
- [css3-modsel-30.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-30.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-30.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-30.xml)
- [css3-modsel-31.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-31.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-31.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-31.xml)
- [css3-modsel-32.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-32.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-32.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-32.xml)
- [css3-modsel-33.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-33.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-33.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-33.xml)
- [css3-modsel-34.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-34.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-34.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-34.xml)
- [css3-modsel-35.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-35.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-35.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-35.xml)
- [css3-modsel-36.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-36.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-36.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-36.xml)
- [css3-modsel-37.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-37.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-37.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-37.xml)
- [css3-modsel-38.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-38.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-38.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-38.xml)
- [css3-modsel-39.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-39.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-39.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-39.xml)
- [css3-modsel-39a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-39a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-39a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-39a.xml)
- [css3-modsel-39b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-39b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-39b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-39b.xml)
- [css3-modsel-39c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-39c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-39c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-39c.xml)
- [css3-modsel-3a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-3a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-3a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-3a.xml)
- [css3-modsel-4.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-4.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-4.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-4.xml)
- [css3-modsel-41.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-41.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-41.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-41.xml)
- [css3-modsel-41a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-41a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-41a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-41a.xml)
- [css3-modsel-42.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-42.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-42.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-42.xml)
- [css3-modsel-42a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-42a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-42a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-42a.xml)
- [css3-modsel-43.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-43.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-43.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-43.xml)
- [css3-modsel-43b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-43b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-43b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-43b.xml)
- [css3-modsel-44.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-44.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-44.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-44.xml)
- [css3-modsel-44b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-44b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-44b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-44b.xml)
- [css3-modsel-44c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-44c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-44c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-44c.xml)
- [css3-modsel-44d.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-44d.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-44d.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-44d.xml)
- [css3-modsel-45.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-45.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-45.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-45.xml)
- [css3-modsel-45b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-45b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-45b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-45b.xml)
- [css3-modsel-45c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-45c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-45c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-45c.xml)
- [css3-modsel-46.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-46.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-46.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-46.xml)
- [css3-modsel-46b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-46b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-46b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-46b.xml)
- [css3-modsel-47.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-47.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-47.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-47.xml)
- [css3-modsel-48.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-48.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-48.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-48.xml)
- [css3-modsel-49.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-49.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-49.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-49.xml)
- [css3-modsel-5.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-5.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-5.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-5.xml)
- [css3-modsel-50.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-50.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-50.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-50.xml)
- [css3-modsel-51.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-51.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-51.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-51.xml)
- [css3-modsel-52.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-52.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-52.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-52.xml)
- [css3-modsel-53.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-53.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-53.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-53.xml)
- [css3-modsel-54.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-54.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-54.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-54.xml)
- [css3-modsel-55.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-55.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-55.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-55.xml)
- [css3-modsel-56.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-56.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-56.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-56.xml)
- [css3-modsel-57.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-57.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-57.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-57.xml)
- [css3-modsel-57b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-57b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-57b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-57b.xml)
- [css3-modsel-59.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-59.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-59.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-59.xml)
- [css3-modsel-6.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-6.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-6.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-6.xml)
- [css3-modsel-60.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-60.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-60.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-60.xml)
- css3-modsel-61.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-61.xml)
- css3-modsel-62.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-62.xml)
- [css3-modsel-63.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-63.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-63.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-63.xml)
- [css3-modsel-64.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-64.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-64.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-64.xml)
- [css3-modsel-65.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-65.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-65.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-65.xml)
- [css3-modsel-66.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-66.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-66.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-66.xml)
- [css3-modsel-66b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-66b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-66b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-66b.xml)
- [css3-modsel-67.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-67.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-67.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-67.xml)
- [css3-modsel-7.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-7.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-7.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-7.xml)
- [css3-modsel-70.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-70.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-70.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-70.xml)
- [css3-modsel-72.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-72.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-72.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-72.xml)
- [css3-modsel-72b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-72b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-72b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-72b.xml)
- [css3-modsel-73.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-73.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-73.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-73.xml)
- [css3-modsel-73b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-73b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-73b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-73b.xml)
- [css3-modsel-74.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-74.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-74.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-74.xml)
- [css3-modsel-74b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-74b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-74b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-74b.xml)
- [css3-modsel-75.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-75.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-75.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-75.xml)
- [css3-modsel-75b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-75b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-75b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-75b.xml)
- [css3-modsel-76.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-76.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-76.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-76.xml)
- [css3-modsel-76b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-76b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-76b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-76b.xml)
- [css3-modsel-77.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-77.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-77.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-77.xml)
- [css3-modsel-77b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-77b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-77b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-77b.xml)
- [css3-modsel-78.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-78.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-78.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-78.xml)
- [css3-modsel-78b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-78b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-78b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-78b.xml)
- [css3-modsel-79.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-79.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-79.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-79.xml)
- [css3-modsel-7b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-7b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-7b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-7b.xml)
- [css3-modsel-8.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-8.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-8.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-8.xml)
- [css3-modsel-80.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-80.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-80.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-80.xml)
- [css3-modsel-81.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-81.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-81.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-81.xml)
- [css3-modsel-81b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-81b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-81b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-81b.xml)
- [css3-modsel-82.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-82.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-82.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-82.xml)
- [css3-modsel-82b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-82b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-82b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-82b.xml)
- [css3-modsel-83.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-83.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-83.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-83.xml)
- [css3-modsel-86.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-86.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-86.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-86.xml)
- [css3-modsel-87.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-87.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-87.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-87.xml)
- [css3-modsel-87b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-87b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-87b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-87b.xml)
- [css3-modsel-88.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-88.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-88.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-88.xml)
- [css3-modsel-88b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-88b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-88b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-88b.xml)
- [css3-modsel-89.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-89.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-89.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-89.xml)
- [css3-modsel-9.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-9.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-9.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-9.xml)
- [css3-modsel-90.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-90.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-90.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-90.xml)
- [css3-modsel-90b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-90b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-90b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-90b.xml)
- [css3-modsel-91.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-91.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-91.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-91.xml)
- [css3-modsel-92.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-92.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-92.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-92.xml)
- [css3-modsel-93.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-93.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-93.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-93.xml)
- [css3-modsel-94.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-94.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-94.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-94.xml)
- [css3-modsel-94b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-94b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-94b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-94b.xml)
- [css3-modsel-95.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-95.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-95.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-95.xml)
- [css3-modsel-96.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-96.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-96.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-96.xml)
- [css3-modsel-96b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-96b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-96b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-96b.xml)
- [css3-modsel-97.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-97.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-97.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-97.xml)
- [css3-modsel-97b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-97b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-97b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-97b.xml)
- [css3-modsel-98.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-98.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-98.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-98.xml)
- [css3-modsel-98b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-98b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-98b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-98b.xml)
- [css3-modsel-99.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-99.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-99.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-99.xml)
- [css3-modsel-99b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-99b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-99b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-99b.xml)
- [css3-modsel-d1.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-d1.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-d1.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-d1.xml)
- [css3-modsel-d1b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-d1b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-d1b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-d1b.xml)
- [css3-modsel-d2.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-d2.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-d2.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-d2.xml)
- [css3-modsel-d3.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-d3.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-d3.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-d3.xml)
- [css3-modsel-d4.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-d4.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-d4.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-d4.xml)

Tests that do not relate to any section

- [eof-right-after-selector-crash.html](https://wpt.fyi/results/css/selectors/eof-right-after-selector-crash.html) [(live test)](http://wpt.live/css/selectors/eof-right-after-selector-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/eof-right-after-selector-crash.html)
- [eof-some-after-selector-crash.html](https://wpt.fyi/results/css/selectors/eof-some-after-selector-crash.html) [(live test)](http://wpt.live/css/selectors/eof-some-after-selector-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/eof-some-after-selector-crash.html)
- [hash-collision.html](https://wpt.fyi/results/css/selectors/hash-collision.html) [(live test)](http://wpt.live/css/selectors/hash-collision.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/hash-collision.html)
- [invalid-pseudos.html](https://wpt.fyi/results/css/selectors/parsing/invalid-pseudos.html) [(live test)](http://wpt.live/css/selectors/parsing/invalid-pseudos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/invalid-pseudos.html)
- [selector-after-font-family.html](https://wpt.fyi/results/css/selectors/selector-after-font-family.html) [(live test)](http://wpt.live/css/selectors/selector-after-font-family.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-after-font-family.html)

------------------------------------------------------------------------

## <a id="syntax"></a>3.  Selector Syntax and Structure[](#syntax)

### <a id="structure"></a>3.1.  Structure and Terminology[](#structure)

A <a id="selector"></a><strong>selector</strong> represents a particular pattern of element(s) in a tree structure. The term <a id="ref-for-selector②"></a>[selector](#selector) can refer to a <a id="ref-for-simple"></a>[simple selector](#simple), <a id="ref-for-compound⑥"></a>[compound selector](#compound), <a id="ref-for-complex"></a>[complex selector](#complex), or <a id="ref-for-selector-list"></a>[selector list](#selector-list). The <a id="selector-subject"></a><strong>subject of a selector</strong> is any element that selector is defined to be about; that is, any element <a id="match"></a><strong>matching</strong> that <a id="ref-for-selector③"></a>selector.

A <a id="simple"></a><strong>simple selector</strong> is a single condition on an element. A <a id="ref-for-type-selector"></a>[type selector](#type-selector), <a id="ref-for-universal-selector"></a>[universal selector](#universal-selector), <a id="ref-for-attribute-selector"></a>[attribute selector](#attribute-selector), <a id="ref-for-class-selector"></a>[class selector](#class-selector), <a id="ref-for-id-selector"></a>[ID selector](#id-selector), or <a id="ref-for-pseudo-class"></a>[pseudo-class](#pseudo-class) is a <a id="ref-for-simple①"></a>[simple selector](#simple). (It is represented by <a id="ref-for-typedef-simple-selector"></a>[\<simple-selector\>](#typedef-simple-selector) in the selectors [grammar](#grammar).) A given element is said to <a id="ref-for-match"></a>[match](#match) a <a id="ref-for-simple②"></a>simple selector when that <a id="ref-for-simple③"></a>simple selector, as defined in this specification and in accordance with the <a id="ref-for-document-language"></a>[document language](#document-language), accurately describes the element.

A <a id="compound"></a><strong>compound selector</strong> is a sequence of <a id="ref-for-simple④"></a>[simple selectors](#simple) that are not separated by a <a id="ref-for-selector-combinator"></a>[combinator](#selector-combinator), and represents a set of simultaneous conditions on a single element. If it contains a <a id="ref-for-type-selector①"></a>[type selector](#type-selector) or <a id="ref-for-universal-selector①"></a>[universal selector](#universal-selector), that selector must come first in the sequence. Only one type selector or universal selector is allowed in the sequence. (A <a id="ref-for-compound⑦"></a>[compound selector](#compound) is represented by <a id="ref-for-typedef-compound-selector"></a>[\<compound-selector\>](#typedef-compound-selector) in the selectors [grammar](#grammar).) A given element is said to <a id="ref-for-match①"></a>[match](#match) a <a id="ref-for-compound⑧"></a>compound selector when it matches all <a id="ref-for-simple⑤"></a>simple selectors in the <a id="ref-for-compound⑨"></a>compound selector.

Note: As whitespace represents the <a id="ref-for-descendant-combinator"></a>[descendant combinator](#descendant-combinator), no whitespace is allowed between the <a id="ref-for-simple⑥"></a>[simple selectors](#simple) in a <a id="ref-for-compound①⓪"></a>[compound selector](#compound).

A <a id="pseudo-compound"></a><strong>pseudo-compound selector</strong> is a <a id="ref-for-pseudo-element"></a>[pseudo-element](#pseudo-element) selector, optionally followed by additional <a id="ref-for-pseudo-class①"></a>[pseudo-class](#pseudo-class) selectors, and optionally preceded by a <a id="ref-for-compound①①"></a>[compound selector](#compound) or another <a id="ref-for-pseudo-compound"></a>[pseudo-compound selector](#pseudo-compound), without any <a id="ref-for-selector-combinator①"></a>[combinators](#selector-combinator). (A <a id="ref-for-pseudo-compound①"></a>pseudo-compound selector is represented by <a id="ref-for-typedef-pseudo-compound-selector"></a>[\<pseudo-compound-selector\>](#typedef-pseudo-compound-selector) in the selectors [grammar](#grammar).) A <a id="ref-for-pseudo-element①"></a>pseudo-element <a id="ref-for-match②"></a>[matches](#match) a <a id="ref-for-pseudo-compound②"></a>pseudo-compound selector when it has the specified pseudo-element name, matches the additional conditions represented by any <a id="ref-for-pseudo-class②"></a>pseudo-classes, and has an <a id="ref-for-originating-element"></a>[originating element](#originating-element) represented by the adjacent preceding selector. If there is no adjacent preceding selector, the <a id="ref-for-universal-selector②"></a>[universal selector](#universal-selector) is assumed. (For example, .foo ::before is equivalent to .foo \*::before, and distinct from .foo::before.)

<a id="example-9e91e72f"></a>

<strong>Example:</strong>

[](#example-9e91e72f) For example, in .foo::before:hover, the .foo is a <a id="ref-for-compound①②"></a>[compound selector](#compound), while the ::before:hover is a <a id="ref-for-pseudo-compound③"></a>[pseudo-compound selector](#pseudo-compound). However, in .foo::before::marker, <a id="ref-for-selectordef-before"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before) and <a id="ref-for-selectordef-marker"></a>[::marker](https://drafts.csswg.org/css-pseudo-4/#selectordef-marker) are separate <a id="ref-for-pseudo-compound④"></a>pseudo-compound selectors.

Note: A <a id="ref-for-pseudo-compound⑤"></a>[pseudo-compound selector](#pseudo-compound) <strong>is not</strong> a <a id="ref-for-compound①③"></a>[compound selector](#compound), and can’t be used in places that expect a <a id="ref-for-compound①④"></a>compound selector only. <a id="ref-for-pseudo-compound⑥"></a>Pseudo-compound selectors act as if they carry a <a id="ref-for-selector-combinator②"></a>[combinator](#selector-combinator) with themselves, expressing their relationship with their <a id="ref-for-originating-element①"></a>[originating element](#originating-element), just as the <a id="ref-for-selectordef-child"></a>[\>](#selectordef-child) combinator expresses a relationship with a parent element.

A <a id="selector-combinator"></a><strong>combinator</strong> is a condition of relationship between two elements represented by the <a id="ref-for-compound①⑤"></a>[compound selectors](#compound) on either side. Combinators in Selectors Level 4 include: the <a id="ref-for-descendant-combinator①"></a>[descendant combinator](#descendant-combinator) (white space), the <a id="ref-for-child-combinator"></a>[child combinator](#child-combinator) (U+003E, <code>&#62;</code>), the <a id="ref-for-next-sibling-combinator"></a>[next-sibling combinator](#next-sibling-combinator) (U+002B, <code>+</code>), and the <a id="ref-for-subsequent-sibling-combinator"></a>[subsequent-sibling combinator](#subsequent-sibling-combinator) (U+007E, <code>&#126;</code>). Two given elements are said to <a id="ref-for-match③"></a>[match](#match) a <a id="ref-for-selector-combinator③"></a>[combinator](#selector-combinator) when the condition of relationship between these elements is true.

A <a id="complex"></a><strong>complex selector</strong> is a sequence of one or more <a id="ref-for-compound①⑥"></a>[compound selectors](#compound) and/or <a id="ref-for-pseudo-compound⑦"></a>[pseudo-compound selectors](#pseudo-compound), with <a id="ref-for-compound①⑦"></a>compound selectors separated by <a id="ref-for-selector-combinator④"></a>[combinators](#selector-combinator). It represents a set of simultaneous conditions on a set of elements in the particular relationships described by its <a id="ref-for-selector-combinator⑤"></a>combinators. (Complex selectors are represented by <a id="ref-for-typedef-complex-selector"></a>[\<complex-selector\>](#typedef-complex-selector) in the selectors [grammar](#grammar).) A given element or pseudo-element is said to <a id="ref-for-match④"></a>[match](#match) a <a id="ref-for-complex①"></a>[complex selector](#complex) when it matches the final <a id="ref-for-compound①⑧"></a>compound/<a id="ref-for-pseudo-compound⑧"></a>pseudo-compound selector in the sequence, and every preceding unit of the sequence also <a id="ref-for-match⑤"></a>matches an element or <a id="ref-for-pseudo-element②"></a>[pseudo-element](#pseudo-element), with the correct relationship between consecutive units as expressed by the combinators separating them (or, for <a id="ref-for-pseudo-compound⑨"></a>pseudo-compound selectors, the correct <a id="ref-for-originating-element②"></a>[originating element](#originating-element) relationship).

<a id="example-3e6d4159"></a>

<strong>Example:</strong>

[](#example-3e6d4159) For example, .foo.bar matches an element with both "foo" and "bar" classes.

.ancestor \> .foo.bar matches a subset of those elements: only those whose parent element (as indicated by the <a id="ref-for-selectordef-child①"></a>[\>](#selectordef-child) combinator) has the "ancestor" class.

.foo.bar::before matches a <a id="ref-for-selectordef-before①"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before) pseudo-element, whose <a id="ref-for-originating-element③"></a>[originating element](#originating-element) matches .foo.bar.

A <a id="list-of-simple-selectors"></a><strong>list of simple/compound/complex selectors</strong> is a comma-separated list of <a id="ref-for-simple⑦"></a>[simple](#simple), <a id="ref-for-compound①⑨"></a>[compound](#compound), or <a id="ref-for-complex②"></a>[complex selectors](#complex). This is also called just a <a id="selector-list"></a><strong>selector list</strong> when the type is either unimportant or specified in the surrounding prose; if the type is important and unspecified, it defaults to meaning a <a id="ref-for-list-of-simple-selectors"></a>[list of complex selectors](#list-of-simple-selectors). (See [§ 4.1 Selector Lists](#grouping) for additional information on <a id="ref-for-selector-list①"></a>[selector lists](#selector-list) and the various \<\*-selector-list\> productions in the [grammar](#grammar) for their formal syntax.) A given element is said to <a id="ref-for-match⑥"></a>[match](#match) a <a id="ref-for-selector-list②"></a>selector list when it matches any (at least one) of the <a id="ref-for-selector④"></a>[selectors](#selector) in that <a id="ref-for-selector-list③"></a>selector list.

### <a id="data-model"></a>3.2.  Data Model[](#data-model)

Selectors are evaluated against an element tree such as the DOM. [\[DOM\]](#biblio-dom) Within this specification, this may be referred to as the "document tree" or "source document".

Each element may have any of the following five aspects, which can be selected against, all of which are matched as strings:

- The element’s type (also known as its tag name).
- The element’s namespace.
- An ID.
- Classes (named groups) to which it belongs.
- Attributes, which are name-value pairs.

Many of the selectors depend on the semantics of the <a id="document-language"></a><strong>document language</strong> (i.e. the language and semantics of the document tree) and/or the semantics of the <a id="host-language"></a><strong>host language</strong> (i.e. the language that is using selectors syntax). For example, the <a id="ref-for-lang-pseudo"></a>[:lang()](#lang-pseudo) selector depends on the <a id="ref-for-document-language①"></a>[document language](#document-language) (e.g. HTML) to define how an element is associated with a language. As a slightly different example, the <a id="ref-for-selectordef-first-line"></a>[::first-line](https://drafts.csswg.org/css-pseudo-4/#selectordef-first-line) pseudo-element depends on the <a id="ref-for-host-language"></a>[host language](#host-language) (e.g. CSS) to define what a <a id="ref-for-selectordef-first-line①"></a>::first-line pseudo-element represents and what it can do.

#### <a id="featureless-elements"></a>3.2.1.  Featureless Elements[](#featureless-elements)

Tests

- [featureless-001.html](https://wpt.fyi/results/css/selectors/featureless-001.html) [(live test)](http://wpt.live/css/selectors/featureless-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/featureless-001.html)
- [featureless-002.html](https://wpt.fyi/results/css/selectors/featureless-002.html) [(live test)](http://wpt.live/css/selectors/featureless-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/featureless-002.html)
- [featureless-003.html](https://wpt.fyi/results/css/selectors/featureless-003.html) [(live test)](http://wpt.live/css/selectors/featureless-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/featureless-003.html)
- [featureless-004.html](https://wpt.fyi/results/css/selectors/featureless-004.html) [(live test)](http://wpt.live/css/selectors/featureless-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/featureless-004.html)
- [featureless-005.html](https://wpt.fyi/results/css/selectors/featureless-005.html) [(live test)](http://wpt.live/css/selectors/featureless-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/featureless-005.html)

While individual elements may lack any of the above features, some elements are <a id="featureless"></a><strong>featureless</strong>. A <a id="ref-for-featureless"></a>[featureless](#featureless) element does not match <em>any selector at all</em>, except:

- <a id="ref-for-simple⑧"></a>[simple selectors](#simple) it is explicitly defined to match

- <a id="ref-for-compound②⓪"></a>[compound selectors](#compound), if all contained <a id="ref-for-simple⑨"></a>[simple selectors](#simple) are allowed to match it

- <a id="ref-for-complex③"></a>[complex selectors](#complex), if the <a id="ref-for-compound②①"></a>[compound selector](#compound) targeting the <a id="ref-for-selector-subject"></a>[subject](#selector-subject) is allowed to match it

- <a id="ref-for-selector-list④"></a>[selector lists](#selector-list), if at least one selector in the list is allowed to match it

- <a id="ref-for-logical-combination-pseudo-classes"></a>[logical combination pseudo-classes](#logical-combination-pseudo-classes), if their argument selector is allowed to match it

- the <a id="ref-for-has-pseudo"></a>[:has()](#has-pseudo) pseudo-class, if and only if the <a id="ref-for-compound②②"></a>[compound selector](#compound) it’s part of contains at least one <em>other</em> <a id="ref-for-simple①⓪"></a>[simple selector](#simple) that’s allowed to match it.

If a selector would otherwise match a <a id="ref-for-featureless①"></a>[featureless](#featureless) element, except for the existence of the default namespace [\[CSS-NAMESPACES-3\]](https://drafts.csswg.org/selectors-4/#biblio-css-namespaces-3) (because <a id="ref-for-featureless②"></a>featureless elements do not have a namespace unless otherwise defined), the default namespace does not prevent the match.

<a id="example-351d0bfb"></a>

<strong>Example:</strong>

[](#example-351d0bfb) For example, the <a id="ref-for-element-shadow-host"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) in a <a id="ref-for-concept-shadow-tree"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) is <a id="ref-for-featureless③"></a>[featureless](#featureless), and can’t be matched by <em>any</em> <a id="ref-for-pseudo-class③"></a>[pseudo-class](#pseudo-class) except for <a id="ref-for-selectordef-host"></a>[:host](https://drafts.csswg.org/css-shadow-1/#selectordef-host) and <a id="ref-for-selectordef-host-context"></a>[:host-context()](https://drafts.csswg.org/css-shadow-1/#selectordef-host-context) (or combinations including those, such as :is(:host, :root)).

Logical combinations like :not(.foo:host) will never match the host element (even if it doesn’t have a "foo" class), because not all of the simple selectors in .foo:host are allowed to match the <a id="ref-for-element-shadow-host①"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host).

Similarly, :not(:host \> .foo) will never match the <a id="ref-for-element-shadow-host②"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host), even tho the <a id="ref-for-element-shadow-host③"></a>shadow host is indeed \*not\* a descendant of itself and doesn’t have the "foo" class, because the subject of the complex selector argument (.foo) isn’t allowed to match the <a id="ref-for-element-shadow-host④"></a>shadow host.

<a id="example-a6be590b"></a>

<strong>Example:</strong>

[](#example-a6be590b) In general, you can’t match a <a id="ref-for-featureless④"></a>[featureless](#featureless) element without explicitly using one of the <a id="ref-for-simple①①"></a>[simple selectors](#simple) it’s allowed to match, to avoid accidentally selecting one of these elements (which are otherwise <em>intentionally</em> easy to not think about). For example, <a id="ref-for-x"></a>[\*](https://drafts.csswg.org/selectors-3/#x) will never match a <a id="ref-for-featureless⑤"></a>featureless element.

The rule for <a id="ref-for-has-pseudo①"></a>[:has()](#has-pseudo), above, works similarly. Even if a <a id="ref-for-element-shadow-host⑤"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) contains a .foo descendant, :has(.foo) will not match it, because <em>the rest</em> of the compound selector (empty) doesn’t contain a simple selector that can match the host. You have to write :host:has(.foo) in order to match the host element.

### <a id="scoping"></a>3.3.  Scoped Selectors[](#scoping)

Some host applications may choose to <a id="scoped-selector"></a><strong>scope</strong> selectors to a particular subtree or fragment of the document, The root of the scoping subtree is called the <a id="scoping-root"></a><strong>scoping root</strong>.

When a selector is <a id="ref-for-scoped-selector"></a>[scoped](#scoped-selector), it matches an element only if the element is a descendant of the <a id="ref-for-scoping-root①"></a>[scoping root](#scoping-root). (The rest of the selector can match unrestricted; it’s only the final matched elements that must be within the scope.)

<a id="example-32ec254f"></a>

<strong>Example:</strong>

[](#example-32ec254f) For example, the <code>querySelector()</code> method defined in [\[DOM\]](#biblio-dom) allows the author to evaluate a <a id="ref-for-scoped-selector①"></a>[scoped](#scoped-selector) selector relative to the element it’s called on.

A call like <code>widget&#46;querySelector(&#34;a&#34;)</code> will thus only find <code><a id="ref-for-the-a-element"></a>[a](https://html&#46;spec&#46;whatwg&#46;org/multipage/text-level-semantics&#46;html#the-a-element)</code> elements inside of the <code>widget</code> element, ignoring any other <code><a id="ref-for-the-a-element①"></a>[a](https://html&#46;spec&#46;whatwg&#46;org/multipage/text-level-semantics&#46;html#the-a-element)</code>s that might be scattered throughout the document.

### <a id="relative"></a>3.4.  Relative Selectors[](#relative)

Certain contexts may accept <a id="relative-selector"></a><strong>relative selectors</strong>, which are a shorthand for selectors that represent elements relative to one or more <a id="relative-selector-anchor-elements"></a><strong>relative selector anchor elements</strong>. Relative selectors begin with a <a id="ref-for-selector-combinator⑥"></a>[combinator](#selector-combinator), with a selector representing the <a id="ref-for-relative-selector-anchor-elements①"></a>[anchor element](#relative-selector-anchor-elements) implied at the start of the selector. (If no combinator is present, the <a id="ref-for-descendant-combinator②"></a>[descendant combinator](#descendant-combinator) is implied.)

Relative selectors are represented by <a id="ref-for-typedef-relative-selector"></a>[\<relative-selector\>](#typedef-relative-selector) in the selectors [grammar](#grammar), and lists of them by <a id="ref-for-typedef-relative-selector-list"></a>[\<relative-selector-list\>](#typedef-relative-selector-list).

### <a id="pseudo-classes"></a>3.5.  Pseudo-classes[](#pseudo-classes)

<a id="pseudo-class"></a><strong>Pseudo-classes</strong> are <a id="ref-for-simple①②"></a>[simple selectors](#simple) that permit selection based on information that lies outside of the document tree or that can be awkward or impossible to express using the other simple selectors. They can also be dynamic, in the sense that an element can acquire or lose a pseudo-class while a user interacts with the document, without the document itself changing. <a id="ref-for-pseudo-class④"></a>[Pseudo-classes](#pseudo-class) do not appear in or modify the document source or document tree.

The syntax of a <a id="ref-for-pseudo-class⑤"></a>[pseudo-class](#pseudo-class) consists of a ":" (U+003A COLON) followed by the name of the <a id="ref-for-pseudo-class⑥"></a>pseudo-class as a CSS <a id="ref-for-css-css-identifier"></a>[identifier](https://drafts.csswg.org/css-values-4/#css-css-identifier), and, in the case of a <a id="functional-pseudo-class"></a><strong>functional pseudo-class</strong>, a pair of parentheses containing its arguments.

<a id="example-86ae2021"></a>

<strong>Example:</strong>

[](#example-86ae2021) For example, <a id="ref-for-valid-pseudo"></a>[:valid](#valid-pseudo) is a regular pseudo-class, and <a id="ref-for-lang-pseudo①"></a>[:lang()](#lang-pseudo) is a <a id="ref-for-functional-pseudo-class"></a>[functional pseudo-class](#functional-pseudo-class).

Like all CSS keywords, <a id="ref-for-pseudo-class⑦"></a>[pseudo-class](#pseudo-class) names are <a id="ref-for-ascii-case-insensitive"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive). No <a id="ref-for-whitespace"></a>[white space](#whitespace) is allowed between the colon and the name of the <a id="ref-for-pseudo-class⑧"></a>pseudo-class, nor, as usual for CSS syntax, between a <a id="ref-for-functional-pseudo-class①"></a>[functional pseudo-class](#functional-pseudo-class)’s name and its opening parenthesis (which thus form a CSS function token). Also as usual, <a id="ref-for-whitespace①"></a>white space is allowed around the arguments inside the parentheses of a functional pseudo-class unless otherwise specified.

Like other <a id="ref-for-simple①③"></a>[simple selectors](#simple), <a id="ref-for-pseudo-class⑨"></a>[pseudo-classes](#pseudo-class) are allowed in all <a id="ref-for-compound②③"></a>[compound selectors](#compound) contained in a selector, and must follow the <a id="ref-for-type-selector②"></a>[type selector](#type-selector) or <a id="ref-for-universal-selector③"></a>[universal selector](#universal-selector), if present.

Note: Some <a id="ref-for-pseudo-class①⓪"></a>[pseudo-classes](#pseudo-class) are mutually exclusive (such that a <a id="ref-for-compound②④"></a>[compound selector](#compound) containing them, while valid, will never match anything), while others can apply simultaneously to the same element.

### <a id="pseudo-elements"></a>3.6. Pseudo-elements[](#pseudo-elements)

Tests

- [x-pseudo-element.html](https://wpt.fyi/results/css/selectors/x-pseudo-element.html) [(live test)](http://wpt.live/css/selectors/x-pseudo-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/x-pseudo-element.html)

Similar to how certain <a id="ref-for-pseudo-class①①"></a>[pseudo-classes](#pseudo-class) represent additional state information not directly present in the document tree, a <a id="pseudo-element"></a><strong>pseudo-element</strong> represents an <em>element</em> not directly present in the document tree. They are used to create abstractions about the document tree beyond those provided by the document tree. For example, pseudo-elements can be used to select portions of the document that do not correspond to a document-language element (including such ranges as don’t align to element boundaries or fit within its tree structure); that represent content not in the document tree or in an alternate projection of the document tree; or that rely on information provided by styling, layout, user interaction, and other processes that are not reflected in the document tree.

<a id="example-6916597f"></a>

<strong>Example:</strong>

[](#example-6916597f) For instance, document languages do not offer mechanisms to access the first letter or first line of an element’s content, but there exist <a id="ref-for-pseudo-element③"></a>[pseudo-elements](#pseudo-element) (<a id="ref-for-selectordef-first-letter"></a>[::first-letter](https://drafts.csswg.org/css-pseudo-4/#selectordef-first-letter) and <a id="ref-for-selectordef-first-line②"></a>[::first-line](https://drafts.csswg.org/css-pseudo-4/#selectordef-first-line)) that allow those things to be styled. Notice especially that in the case of <a id="ref-for-selectordef-first-line③"></a>::first-line, which portion of content is represented by the pseudo-element depends on layout information that cannot be inferred from the document tree.

<a id="ref-for-pseudo-element④"></a>[Pseudo-elements](#pseudo-element) can also represent content that doesn’t exist in the source document at all, such as the <a id="ref-for-selectordef-before②"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before) and <a id="ref-for-selectordef-after"></a>[::after](https://drafts.csswg.org/css-pseudo-4/#selectordef-after) pseudo-elements which allow additional content to be inserted before or after the contents of any element.

Like <a id="ref-for-pseudo-class①②"></a>[pseudo-classes](#pseudo-class), <a id="ref-for-pseudo-element⑤"></a>[pseudo-elements](#pseudo-element) do not appear in or modify the document source or document tree. Accordingly, they also do not affect the interpretation of <a id="ref-for-structural-pseudo-classes"></a>[structural pseudo-classes](#structural-pseudo-classes) or other selectors pertaining to their <a id="ref-for-originating-element④"></a>[originating element](#originating-element) or its tree.

The host language defines which pseudo-elements exist, their type, and their abilities. Pseudo-elements that exist in CSS are defined in [\[CSS21\]](#biblio-css21) (Level 2), [\[SELECT\]](#biblio-select) (Level 3), and [\[CSS-PSEUDO-4\]](#biblio-css-pseudo-4) (Level 4).

#### <a id="pseudo-element-syntax"></a>3.6.1.  Syntax[](#pseudo-element-syntax)

The syntax of a <a id="ref-for-pseudo-element⑥"></a>[pseudo-element](#pseudo-element) is "::" (two U+003A COLON characters) followed by the name of the <a id="ref-for-pseudo-element⑦"></a>pseudo-element as an <a id="ref-for-css-css-identifier①"></a>[identifier](https://drafts.csswg.org/css-values-4/#css-css-identifier), and, in the case of a <a id="functional-pseudo-element"></a><strong>functional pseudo-element</strong>, a pair of parentheses containing its arguments. <a id="ref-for-pseudo-element⑧"></a>Pseudo-element names are <a id="ref-for-ascii-case-insensitive①"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive). No <a id="ref-for-whitespace②"></a>[white space](#whitespace) is allowed between the two colons, or between the colons and the name.

Because [CSS Level 1](https://www.w3.org/TR/CSS1) and [CSS Level 2](https://www.w3.org/TR/CSS2) conflated pseudo-elements and pseudo-classes by sharing a single-colon syntax for both, user agents must also accept the previous one-colon notation for the Level 1 & 2 pseudo-elements (<a id="ref-for-selectordef-before③"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before), <a id="ref-for-selectordef-after①"></a>[::after](https://drafts.csswg.org/css-pseudo-4/#selectordef-after), <a id="ref-for-selectordef-first-line④"></a>[::first-line](https://drafts.csswg.org/css-pseudo-4/#selectordef-first-line), and <a id="ref-for-selectordef-first-letter①"></a>[::first-letter](https://drafts.csswg.org/css-pseudo-4/#selectordef-first-letter)). This compatibility notation is not allowed for any other <a id="ref-for-pseudo-element⑨"></a>[pseudo-elements](#pseudo-element). However, as this syntax is deprecated, authors should use the Level 3+ double-colon syntax for these <a id="ref-for-pseudo-element①⓪"></a>pseudo-elements.

<a id="ref-for-pseudo-element①①"></a>[Pseudo-elements](#pseudo-element) are <a id="ref-for-featureless⑥"></a>[featureless](#featureless), and so can’t be matched by any other selector.

#### <a id="pseudo-element-attachment"></a>3.6.2.  Binding to the Document Tree[](#pseudo-element-attachment)

<a id="ref-for-pseudo-element①②"></a>[Pseudo-elements](#pseudo-element) do not exist independently in the tree: they are always bound to another element on the page, called their <a id="originating-element"></a><strong>originating element</strong>. Syntactically, a <a id="ref-for-pseudo-element①③"></a>pseudo-element immediately follows the <a id="ref-for-compound②⑤"></a>[compound selector](#compound) representing its <a id="ref-for-originating-element⑤"></a>[originating element](#originating-element). If this <a id="ref-for-compound②⑥"></a>compound selector is omitted, it is assumed to be the <a id="ref-for-universal-selector④"></a>[universal selector](#universal-selector) <a id="ref-for-x①"></a>[\*](https://drafts.csswg.org/selectors-3/#x).

<a id="example-e6d9ca44"></a>

<strong>Example:</strong>

[](#example-e6d9ca44) For example, in the selector div a::before, the <code><a id="ref-for-the-a-element②"></a>[a](https://html&#46;spec&#46;whatwg&#46;org/multipage/text-level-semantics&#46;html#the-a-element)</code> elements matched by the selector are the <a id="ref-for-originating-element⑥"></a>[originating elements](#originating-element) for the <a id="ref-for-selectordef-before④"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before) pseudo-elements attached to them.

The selector <a id="ref-for-selectordef-first-line⑤"></a>[::first-line](https://drafts.csswg.org/css-pseudo-4/#selectordef-first-line) is equivalent to \*::first-line, which selects the <a id="ref-for-selectordef-first-line⑥"></a>::first-line pseudo-element on <em>every</em> element in the document.

When a <a id="ref-for-pseudo-element①④"></a>[pseudo-element](#pseudo-element) is encountered in a selector, the part of the selector before the <a id="ref-for-pseudo-element①⑤"></a>pseudo-element selects the <a id="ref-for-originating-element⑦"></a>[originating element](#originating-element) for the <a id="ref-for-pseudo-element①⑥"></a>pseudo-element; the part of the selector after it, if any, applies to the <a id="ref-for-pseudo-element①⑦"></a>pseudo-element itself. (See below.)

#### <a id="pseudo-element-states"></a>3.6.3.  Pseudo-classing Pseudo-elements[](#pseudo-element-states)

Certain <a id="ref-for-pseudo-element①⑧"></a>[pseudo-elements](#pseudo-element) may be immediately followed by any combination of certain <a id="ref-for-pseudo-class①③"></a>[pseudo-classes](#pseudo-class), in which case the <a id="ref-for-pseudo-element①⑨"></a>pseudo-element is represented only when it is in the corresponding state. This specification allows any <a id="ref-for-pseudo-element②⓪"></a>pseudo-element to be followed by any combination of the <a id="ref-for-logical-combination-pseudo-classes①"></a>[logical combination pseudo-classes](#logical-combination-pseudo-classes) and the <a id="ref-for-user-action-pseudo-class①"></a>[user action pseudo-classes](#user-action-pseudo-class). Other specifications may allow additional <a id="ref-for-pseudo-class①④"></a>pseudo-classes to be attached to particular <a id="ref-for-pseudo-element②①"></a>pseudo-elements. Combinations that are not explicitly allowed are <a id="ref-for-invalid-selector"></a>[invalid selectors](#invalid-selector).

Note: The <a id="ref-for-logical-combination-pseudo-classes②"></a>[logical combination pseudo-classes](#logical-combination-pseudo-classes) pass any restrictions on validity of selectors at their position to their arguments.

<a id="example-61c8ad8f"></a>

<strong>Example:</strong>

[](#example-61c8ad8f) For example, since the <a id="ref-for-hover-pseudo"></a>[:hover](#hover-pseudo) pseudo-class specifies that it can apply to any pseudo-element, ::first-line:hover will match when the first line is hovered. However, since neither <a id="ref-for-focus-pseudo"></a>[:focus](#focus-pseudo) nor <a id="ref-for-selectordef-first-line⑦"></a>[::first-line](https://drafts.csswg.org/css-pseudo-4/#selectordef-first-line) define that <a id="ref-for-focus-pseudo①"></a>:focus can apply to <a id="ref-for-selectordef-first-line⑧"></a>::first-line, the selector ::first-line:focus will never match anything.

Notice that ::first-line:hover is very different from :hover::first-line, which matches the first line of any originating element that is hovered! For example, :hover::first-line also matches the first line of a paragraph when the second line of the paragraph is hovered, whereas ::first-line:hover only matches if the first line itself is hovered.

#### <a id="sub-pseudo-elements"></a>3.6.4.  Sub-pseudo-elements[](#sub-pseudo-elements)

Some <a id="ref-for-pseudo-element②②"></a>[pseudo-elements](#pseudo-element) are able to be the <a id="ref-for-originating-element⑧"></a>[originating element](#originating-element) of other <a id="ref-for-pseudo-element②③"></a>pseudo-elements, which are defined as the <a id="sub-pseudo-element"></a><strong>sub-pseudo-elements</strong> of this <a id="originating-pseudo-element"></a><strong>originating pseudo-element</strong>. For example, when <a id="ref-for-selectordef-before⑤"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before) is given a <a id="ref-for-valdef-display-list-item"></a>[list-item](https://drafts.csswg.org/css-display-4/#valdef-display-list-item) <a id="ref-for-display-type"></a>[display type](https://drafts.csswg.org/css-display-4/#display-type), it becomes the <a id="ref-for-originating-pseudo-element"></a>[originating pseudo-element](#originating-pseudo-element) of its ::before::marker <a id="ref-for-sub-pseudo-element"></a>[sub-pseudo-element](#sub-pseudo-element).

Where disambiguation is needed, the term <a id="ultimate-originating-element"></a><strong>ultimate originating element</strong> refers to the real (non-pseudo) element from which a <a id="ref-for-pseudo-element②④"></a>[pseudo-element](#pseudo-element) originates.

Unless the corresponding <a id="ref-for-sub-pseudo-element①"></a>[sub-pseudo-element](#sub-pseudo-element) is explicitly defined to exist in another specification, pseudo-element selectors are not valid when compounded to another pseudo-element selector. So, for example, ::before::before is an invalid selector, but ::before::marker is valid (in implementations that support the ::before::marker <a id="ref-for-sub-pseudo-element②"></a>sub-pseudo-element).

#### <a id="pseudo-element-structure"></a>3.6.5.  Internal Structure[](#pseudo-element-structure)

Some <a id="ref-for-pseudo-element②⑤"></a>[pseudo-elements](#pseudo-element) are defined to have internal structure. These <a id="ref-for-pseudo-element②⑥"></a>pseudo-elements may be followed by child/descendant combinators to express those relationships. Selectors containing <a id="ref-for-selector-combinator⑦"></a>[combinators](#selector-combinator) after the pseudo-element are otherwise invalid.

<a id="example-3594b9ac"></a>

<strong>Example:</strong>

[](#example-3594b9ac) For example, ::first-letter + span and ::first-letter em are invalid selectors. However, if a new ::shadow pseudo-element were defined to have internal structure, ::shadow \> p would be a valid selector.

Note: A future specification may expand the capabilities of existing pseudo-elements, so some of these currently-invalid selectors (e.g. ::first-line :any-link) may become valid in the future.

The children of such <a id="ref-for-pseudo-element②⑦"></a>[pseudo-elements](#pseudo-element) can simultaneously be children of other elements, too. However, at least in CSS, their rendering must be defined so as to maintain the tree-ness of the <a id="ref-for-box-tree"></a>[box tree](https://drafts.csswg.org/css-display-4/#box-tree).

### <a id="case-sensitive"></a>3.7.  Characters and case sensitivity[](#case-sensitive)

Tests

- [quirks-mode-stylesheet-dynamic-add-001.html](https://wpt.fyi/results/css/selectors/invalidation/quirks-mode-stylesheet-dynamic-add-001.html) [(live test)](http://wpt.live/css/selectors/invalidation/quirks-mode-stylesheet-dynamic-add-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/quirks-mode-stylesheet-dynamic-add-001.html)
- [selectors-case-sensitive-001.html](https://wpt.fyi/results/css/selectors/selectors-case-sensitive-001.html) [(live test)](http://wpt.live/css/selectors/selectors-case-sensitive-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-case-sensitive-001.html)
- [value-case-sensitivity-svg.html](https://wpt.fyi/results/css/selectors/attribute-selectors/attribute-case/value-case-sensitivity-svg.html) [(live test)](http://wpt.live/css/selectors/attribute-selectors/attribute-case/value-case-sensitivity-svg.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/attribute-selectors/attribute-case/value-case-sensitivity-svg.html)

All syntax defined by the Selectors spec is <a id="ref-for-ascii-case-insensitive②"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) (i.e. \[a-z\] and \[A-Z\] are equivalent), as usual for CSS syntax—​this includes pseudo-class and pseudo-element names, and keywords used in various places, such as the <a id="ref-for-valdef-attribute-selector-s"></a>[s](#valdef-attribute-selector-s) or <a id="ref-for-valdef-attribute-selector-i"></a>[i](#valdef-attribute-selector-i) keyword in attribute selectors.

When matching Selectors to names and values defined by the document, by default this is done using the <a id="ref-for-string-is"></a>[identical to](https://infra.spec.whatwg.org/#string-is) operation (aka "case sensitive"), unless otherwise defined by the host language or another relevant specification. This includes element names, class names, IDs, attribute names, and attribute values.

<a id="example-97fadc2b"></a>

<strong>Example:</strong>

[](#example-97fadc2b) For example, HTML has some mildly complex rules about matching certain names on HTML elements, which are <em>nearly</em> equivalent to being case-insensitive. See [HTML § 4.16.2 Case-sensitivity of selectors](https://html.spec.whatwg.org/multipage/semantics-other.html#case-sensitivity-of-selectors) for details—​this affects element names, attribute names, and <em>some</em> attribute values.

(The HTML parser automatically ASCII-lowercases a number of these values, so the rules it defines are <em>usually</em> equivalent to being <a id="ref-for-ascii-case-insensitive③"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive)), but it is possible to produce attributes via scripting that retain their non-lowercase casing, and as a result cannot be matched by Selectors due to these rules.)

Case sensitivity of namespace prefixes is defined in [\[CSS3NAMESPACE\]](#biblio-css3namespace). Case sensitivity of <a id="ref-for-language-range"></a>[language ranges](#language-range) is defined in the <a id="ref-for-lang-pseudo②"></a>[:lang()](#lang-pseudo) section.

<a id="whitespace"></a><strong>White space</strong> in Selectors consists of the code points SPACE (U+0020), TAB (U+0009), LINE FEED (U+000A), CARRIAGE RETURN (U+000D), and FORM FEED (U+000C). Other space-like code points, such as EM SPACE (U+2003) and IDEOGRAPHIC SPACE (U+3000), are never considered syntactic white space.

Code points in Selectors can be escaped with a backslash according to the same [escaping rules](https://www.w3.org/TR/CSS21/syndata.html#characters) as CSS. [\[CSS21\]](#biblio-css21) Note that escaping a code point “cancels out” any special meaning it may have in Selectors. For example, the selector \#foo\>a contains a combinator, but \#foo&#92;&#62;a instead selects an element with the id <code>foo&#62;a</code>.

### <a id="namespaces"></a>3.8.  Declaring Namespace Prefixes[](#namespaces)

Certain selectors support namespace prefixes. The mechanism by which namespace prefixes are <a id="nsdecl"></a><strong>declared</strong> should be specified by the language that uses Selectors. If the language does not specify a namespace prefix declaration mechanism, then no prefixes are declared. In CSS, namespace prefixes are declared with the <a id="ref-for-at-ruledef-namespace"></a>[&#64;namespace](https://drafts.csswg.org/css-namespaces-3/#at-ruledef-namespace) rule. [\[CSS3NAMESPACE\]](#biblio-css3namespace)

### <a id="invalid"></a>3.9.  Invalid Selectors and Error Handling[](#invalid)

Tests

- [selectorText-dynamic-001.html](https://wpt.fyi/results/css/selectors/invalidation/selectorText-dynamic-001.html) [(live test)](http://wpt.live/css/selectors/invalidation/selectorText-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/selectorText-dynamic-001.html)
- [sheet-going-away-001.html](https://wpt.fyi/results/css/selectors/invalidation/sheet-going-away-001.html) [(live test)](http://wpt.live/css/selectors/invalidation/sheet-going-away-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/sheet-going-away-001.html)
- [sheet-going-away-002.html](https://wpt.fyi/results/css/selectors/invalidation/sheet-going-away-002.html) [(live test)](http://wpt.live/css/selectors/invalidation/sheet-going-away-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/sheet-going-away-002.html)

User agents must observe the rules for handling <a id="invalid-selector"></a><strong>invalid selectors</strong>:

- a parsing error in a selector, e.g. an unrecognized token or a token which is not allowed at the current parsing point (see overall [§ 16 Grammar](#grammar) and per-selector syntax definitions), causes that selector to be invalid.
- a simple selector containing an [undeclared namespace prefix](#namespaces) is invalid
- a selector containing an invalid simple selector, an invalid combinator or an invalid token is invalid.
- a selector list containing an invalid selector is invalid.
- an empty selector, i.e. one that contains no <a id="ref-for-compound②⑦"></a>[compound selector](#compound), is invalid.

Note: Consistent with CSS’s forwards-compatible parsing principle, UAs <em>must</em> treat as <a id="ref-for-invalid-selector①"></a>[invalid](#invalid-selector) any pseudo-classes, pseudo-elements, combinators, or other syntactic constructs for which they have no usable level of support. See [Partial implementations](#w3c-partial).

An <a id="ref-for-invalid-selector②"></a>[invalid selector](#invalid-selector) represents, and therefore matches, nothing.

### <a id="legacy-aliasing"></a>3.10.  Legacy Aliases[](#legacy-aliasing)

Some selectors have a <a id="legacy-selector-alias"></a><strong>legacy selector alias</strong>. This is a name which, at parse time, is converted to the standard name (and thus does not appear anywhere in any object model representing the selector).

## <a id="logical-combination"></a>4.  Logical Combinations[](#logical-combination)

Tests

- [is-where-pseudo-containing-hard-pseudo-and-never-matching.html](https://wpt.fyi/results/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo-and-never-matching.html) [(live test)](http://wpt.live/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo-and-never-matching.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo-and-never-matching.html)
- [is-where-pseudo-containing-hard-pseudo.html](https://wpt.fyi/results/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo.html) [(live test)](http://wpt.live/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo.html)

Selector logic can be manipulated by <a id="ref-for-compound②⑧"></a>[compounding](#compound) (logical AND), <a id="ref-for-selector-list⑤"></a>[selector lists](#selector-list) (logical OR), and the <a id="logical-combination-pseudo-classes"></a><strong>logical combination pseudo-classes</strong> <a id="ref-for-matches-pseudo"></a>[:is()](#matches-pseudo), <a id="ref-for-where-pseudo"></a>[:where()](#where-pseudo), and <a id="ref-for-negation-pseudo"></a>[:not()](#negation-pseudo). The <a id="ref-for-logical-combination-pseudo-classes③"></a>[logical combination pseudo-classes](#logical-combination-pseudo-classes) are allowed anywhere that any other <a id="ref-for-pseudo-class①⑤"></a>[pseudo-classes](#pseudo-class) are allowed, but pass any restrictions to their arguments. (For example, if only <a id="ref-for-compound②⑨"></a>compound selectors are allowed, then only <a id="ref-for-compound③⓪"></a>compound selectors are valid within an <a id="ref-for-matches-pseudo①"></a>:is().)

Note: Since inside <a id="ref-for-matches-pseudo②"></a>[:is()](#matches-pseudo) and <a id="ref-for-where-pseudo①"></a>[:where()](#where-pseudo) invalid arguments are dropped without invaliding the <a id="ref-for-pseudo-class①⑥"></a>[pseudo-class](#pseudo-class) itself, selector arguments that are invalidated by contextual restrictions likewise do not invalidate the <a id="ref-for-matches-pseudo③"></a>:is() pseudo-class itself.

### <a id="grouping"></a>4.1.  Selector Lists[](#grouping)

A comma-separated list of selectors represents the union of all elements selected by each of the individual selectors in the <a id="ref-for-selector-list⑥"></a>[selector list](#selector-list). (A comma is U+002C.) For example, in CSS when several selectors share the same declarations, they may be grouped into a comma-separated list. White space may appear before and/or after the comma.

<a id="example-1930da7e"></a>

<strong>Example:</strong>

[](#example-1930da7e) CSS example: In this example, we condense three rules with identical declarations into one. Thus,

``` text
h1 { font-family: sans-serif }
h2 { font-family: sans-serif }
h3 { font-family: sans-serif }
```

is equivalent to:

``` text
h1, h2, h3 { font-family: sans-serif }
```

<strong>Warning</strong>: the equivalence is true in this example because all the selectors are valid selectors. If just one of these selectors were invalid, the entire <a id="ref-for-selector-list⑦"></a>[selector list](#selector-list) would be invalid. This would invalidate the rule for all three heading elements, whereas in the former case only one of the three individual heading rules would be invalidated.

<a id="example-fd0daa05"></a>

<strong>Example:</strong>

[](#example-fd0daa05) Invalid CSS example:

``` text
h1 { font-family: sans-serif }
h2..foo { font-family: sans-serif }
h3 { font-family: sans-serif }
```

is not equivalent to:

``` text
h1, h2..foo, h3 { font-family: sans-serif } 
```

because the above selector (h1, h2..foo, h3) is entirely invalid and the entire style rule is dropped. (When the selectors are not grouped, only the rule for h2..foo is dropped.)

### <a id="matches"></a>4.2.  The Matches-Any Pseudo-class: <a id="ref-for-matches-pseudo④"></a>[:is()](#matches-pseudo)[](#matches)

Tests

- [is.html](https://wpt.fyi/results/css/selectors/invalidation/is.html) [(live test)](http://wpt.live/css/selectors/invalidation/is.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/is.html)
- [is-default-ns-001.html](https://wpt.fyi/results/css/selectors/is-default-ns-001.html) [(live test)](http://wpt.live/css/selectors/is-default-ns-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-default-ns-001.html)
- [is-default-ns-002.html](https://wpt.fyi/results/css/selectors/is-default-ns-002.html) [(live test)](http://wpt.live/css/selectors/is-default-ns-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-default-ns-002.html)
- [is-default-ns-003.html](https://wpt.fyi/results/css/selectors/is-default-ns-003.html) [(live test)](http://wpt.live/css/selectors/is-default-ns-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-default-ns-003.html)
- [is-default-ns-002.html](https://wpt.fyi/results/css/selectors/is-default-ns-002.html) [(live test)](http://wpt.live/css/selectors/is-default-ns-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-default-ns-002.html)
- [is-default-ns-003.html](https://wpt.fyi/results/css/selectors/is-default-ns-003.html) [(live test)](http://wpt.live/css/selectors/is-default-ns-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-default-ns-003.html)
- [is-nested.html](https://wpt.fyi/results/css/selectors/is-nested.html) [(live test)](http://wpt.live/css/selectors/is-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-nested.html)
- [is-specificity-shadow.html](https://wpt.fyi/results/css/selectors/is-specificity-shadow.html) [(live test)](http://wpt.live/css/selectors/is-specificity-shadow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-specificity-shadow.html)
- [is-specificity.html](https://wpt.fyi/results/css/selectors/is-specificity.html) [(live test)](http://wpt.live/css/selectors/is-specificity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-specificity.html)
- [is-where-basic.html](https://wpt.fyi/results/css/selectors/is-where-basic.html) [(live test)](http://wpt.live/css/selectors/is-where-basic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-where-basic.html)
- [is-where-error-crash.html](https://wpt.fyi/results/css/selectors/is-where-error-crash.html) [(live test)](http://wpt.live/css/selectors/is-where-error-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-where-error-crash.html)
- [is-where-error-recovery.html](https://wpt.fyi/results/css/selectors/is-where-error-recovery.html) [(live test)](http://wpt.live/css/selectors/is-where-error-recovery.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-where-error-recovery.html)
- [is-where-not.html](https://wpt.fyi/results/css/selectors/is-where-not.html) [(live test)](http://wpt.live/css/selectors/is-where-not.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-where-not.html)
- [is-where-pseudo-classes.html](https://wpt.fyi/results/css/selectors/is-where-pseudo-classes.html) [(live test)](http://wpt.live/css/selectors/is-where-pseudo-classes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-where-pseudo-classes.html)
- [is-where-pseudo-elements.html](https://wpt.fyi/results/css/selectors/is-where-pseudo-elements.html) [(live test)](http://wpt.live/css/selectors/is-where-pseudo-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-where-pseudo-elements.html)
- [is-where-shadow.html](https://wpt.fyi/results/css/selectors/is-where-shadow.html) [(live test)](http://wpt.live/css/selectors/is-where-shadow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-where-shadow.html)
- [is-where-visited.html](https://wpt.fyi/results/css/selectors/is-where-visited.html) [(live test)](http://wpt.live/css/selectors/is-where-visited.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/is-where-visited.html)
- [parse-is-where.html](https://wpt.fyi/results/css/selectors/parsing/parse-is-where.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-is-where.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-is-where.html)
- [parse-is.html](https://wpt.fyi/results/css/selectors/parsing/parse-is.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-is.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-is.html)
- [query-is.html](https://wpt.fyi/results/css/selectors/query/query-is.html) [(live test)](http://wpt.live/css/selectors/query/query-is.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/query/query-is.html)

The matches-any pseudo-class, <a id="matches-pseudo"></a><strong>:is()</strong>, is a functional pseudo-class taking a <a id="ref-for-typedef-forgiving-selector-list"></a>[\<forgiving-selector-list\>](#typedef-forgiving-selector-list) as its sole argument.

If the argument, after parsing, is an empty list, the pseudo-class is valid but matches nothing. Otherwise, the pseudo-class matches any element that matches any of the selectors in the list.

Note: The specificity of the <a id="ref-for-matches-pseudo⑤"></a>[:is()](#matches-pseudo) pseudo-class is replaced by the specificity of its most specific argument. Thus, a selector written with <a id="ref-for-matches-pseudo⑥"></a>:is() does not necessarily have equivalent specificity to the equivalent selector written without <a id="ref-for-matches-pseudo⑦"></a>:is() For example, if we have :is(ul, ol, .list) \> \[hidden\] and ul \> \[hidden\], ol \> \[hidden\], .list \> \[hidden\] a \[hidden\] child of an <code><a id="ref-for-the-ol-element"></a>[ol](https://html&#46;spec&#46;whatwg&#46;org/multipage/grouping-content&#46;html#the-ol-element)</code> matches the first selector with a specificity of (0,2,0) whereas it matches the second selector with a specificity of (0,1,1). See [§ 15 Calculating a selector’s specificity](#specificity-rules).

Pseudo-elements cannot be represented by the matches-any pseudo-class; they are not valid within <a id="ref-for-matches-pseudo⑧"></a>[:is()](#matches-pseudo).

Default namespace declarations do not affect the <a id="ref-for-compound③①"></a>[compound selector](#compound) representing the <a id="ref-for-selector-subject①"></a>[subject](#selector-subject) of any selector within a <a id="ref-for-matches-pseudo⑨"></a>[:is()](#matches-pseudo) pseudo-class, unless that compound selector contains an explicit <a id="ref-for-universal-selector⑤"></a>[universal selector](#universal-selector) or <a id="ref-for-type-selector③"></a>[type selector](#type-selector).

<a id="example-2cab03ed"></a>

<strong>Example:</strong>

[](#example-2cab03ed) For example, the following selector matches any element that is being hovered or focused, regardless of its namespace. In particular, it is not limited to only matching elements in the default namespace that are being hovered or focused.

``` text
*|*:is(:hover, :focus) 
```

The following selector, however, represents only hovered or focused elements that are in the default namespace, because it uses an explicit universal selector within the <a id="ref-for-matches-pseudo①⓪"></a>[:is()](#matches-pseudo) notation:

``` text
*|*:is(*:hover, *:focus) 
```

As previous drafts of this specification used the name <a id="selectordef-matches"></a><strong>:matches()</strong> for this pseudo-class, UAs may additionally implement this obsolete name as a <a id="ref-for-legacy-selector-alias"></a>[legacy selector alias](#legacy-selector-alias) for <a id="ref-for-matches-pseudo①①"></a>[:is()](#matches-pseudo) if needed for backwards-compatibility.

### <a id="negation"></a>4.3.  The Negation (Matches-None) Pseudo-class: <a id="ref-for-negation-pseudo①"></a>[:not()](#negation-pseudo)[](#negation)

Tests

- [not-001.html](https://wpt.fyi/results/css/selectors/invalidation/not-001.html) [(live test)](http://wpt.live/css/selectors/invalidation/not-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/not-001.html)
- [not-002.html](https://wpt.fyi/results/css/selectors/invalidation/not-002.html) [(live test)](http://wpt.live/css/selectors/invalidation/not-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/not-002.html)
- [not-complex.html](https://wpt.fyi/results/css/selectors/not-complex.html) [(live test)](http://wpt.live/css/selectors/not-complex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/not-complex.html)
- [not-default-ns-001.html](https://wpt.fyi/results/css/selectors/not-default-ns-001.html) [(live test)](http://wpt.live/css/selectors/not-default-ns-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/not-default-ns-001.html)
- [not-default-ns-002.html](https://wpt.fyi/results/css/selectors/not-default-ns-002.html) [(live test)](http://wpt.live/css/selectors/not-default-ns-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/not-default-ns-002.html)
- [not-default-ns-003.html](https://wpt.fyi/results/css/selectors/not-default-ns-003.html) [(live test)](http://wpt.live/css/selectors/not-default-ns-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/not-default-ns-003.html)
- [not-links.html](https://wpt.fyi/results/css/selectors/not-links.html) [(live test)](http://wpt.live/css/selectors/not-links.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/not-links.html)
- [not-specificity.html](https://wpt.fyi/results/css/selectors/not-specificity.html) [(live test)](http://wpt.live/css/selectors/not-specificity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/not-specificity.html)
- [parse-not.html](https://wpt.fyi/results/css/selectors/parsing/parse-not.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-not.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-not.html)
- [query-where.html](https://wpt.fyi/results/css/selectors/query/query-where.html) [(live test)](http://wpt.live/css/selectors/query/query-where.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/query/query-where.html)

The negation pseudo-class, <a id="negation-pseudo"></a><strong>:not()</strong>, is a functional pseudo-class taking a <a id="ref-for-typedef-complex-real-selector-list"></a>[\<complex-real-selector-list\>](#typedef-complex-real-selector-list) as an argument. It represents an element that is not represented by its argument.

Note: In Selectors Level 3, only a single <a id="ref-for-simple①④"></a>[simple selector](#simple) was allowed as the argument to <a id="ref-for-negation-pseudo②"></a>[:not()](#negation-pseudo).

Note: The specificity of the <a id="ref-for-negation-pseudo③"></a>[:not()](#negation-pseudo) pseudo-class is replaced by the specificity of the most specific selector in its argument; thus it has the exact behavior of :not(:is(<var>argument</var>)). See [§ 15 Calculating a selector’s specificity](#specificity-rules).

Pseudo-elements cannot be represented by the negation pseudo-class; they are not valid within <a id="ref-for-negation-pseudo④"></a>[:not()](#negation-pseudo).

<a id="example-2a176618"></a>

<strong>Example:</strong>

[](#example-2a176618) For example, the following selector matches all <a id="ref-for-the-button-element"></a>[button](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element) elements in an HTML document that are not disabled.

``` text
button:not([DISABLED]) 
```

The following selector represents all but FOO elements.

``` text
*:not(FOO)
```

The following compound selector represents all HTML elements except links.

``` text
html|*:not(:link):not(:visited)
```

As with <a id="ref-for-matches-pseudo①②"></a>[:is()](#matches-pseudo), default namespace declarations do not affect the <a id="ref-for-compound③②"></a>[compound selector](#compound) representing the <a id="ref-for-selector-subject②"></a>[subject](#selector-subject) of any selector within a <a id="ref-for-negation-pseudo⑤"></a>[:not()](#negation-pseudo) pseudo-class, unless that compound selector contains an explicit <a id="ref-for-universal-selector⑥"></a>[universal selector](#universal-selector) or <a id="ref-for-type-selector④"></a>[type selector](#type-selector). (See <a id="ref-for-matches-pseudo①③"></a>:is() for examples.)

Note: The <a id="ref-for-negation-pseudo⑥"></a>[:not()](#negation-pseudo) pseudo-class allows useless selectors to be written. For instance :not(\*\|\*), which represents no element at all, or div:not(span), which is equivalent to div but with a higher specificity.

### <a id="zero-matches"></a>4.4.  The Specificity-adjustment Pseudo-class: <a id="ref-for-where-pseudo②"></a>[:where()](#where-pseudo)[](#zero-matches)

Tests

- [where.html](https://wpt.fyi/results/css/selectors/invalidation/where.html) [(live test)](http://wpt.live/css/selectors/invalidation/where.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/where.html)
- [parse-where.html](https://wpt.fyi/results/css/selectors/parsing/parse-where.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-where.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-where.html)
- [pseudo-where-crash.html](https://wpt.fyi/results/css/selectors/pseudo-where-crash.html) [(live test)](http://wpt.live/css/selectors/pseudo-where-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/pseudo-where-crash.html)

The Specificity-adjustment pseudo-class, <a id="where-pseudo"></a><strong>:where()</strong>, is a functional pseudo-class with the same syntax and functionality as <a id="ref-for-matches-pseudo①④"></a>[:is()](#matches-pseudo). Unlike <a id="ref-for-matches-pseudo①⑤"></a>:is(), neither the <a id="ref-for-where-pseudo③"></a>[:where()](#where-pseudo) pseudo-class, nor any of its arguments, contribute to the <a id="ref-for-specificity"></a>[specificity](#specificity) of the selector—​its <a id="ref-for-specificity①"></a>specificity is always zero.

This is useful for introducing filters in a selector while keeping the associated style declarations easy to override.

<a id="example-7ca50acf"></a>

<strong>Example:</strong>

[](#example-7ca50acf) Below is a common example where the specificity heuristic fails to match author expectations:

``` text
a:not(:hover) {
  text-decoration: none;
}

nav a {
  /* Has no effect */
  text-decoration: underline;
}
```

However, by using <a id="ref-for-where-pseudo④"></a>[:where()](#where-pseudo) the author can explicitly declare their intent:

``` text
a:where(:not(:hover)) {
  text-decoration: none;
}

nav a {
  /* Works now! */
  text-decoration: underline;
}
```

Note: Future levels of Selectors may introduce an additional argument to explicitly set the specificity of that instance of the pseudo-class.

### <a id="relational"></a>4.5.  The Relational Pseudo-class: <a id="ref-for-has-pseudo②"></a>[:has()](#has-pseudo)[](#relational)

Tests

- [has-argument-with-explicit-scope.html](https://wpt.fyi/results/css/selectors/has-argument-with-explicit-scope.html) [(live test)](http://wpt.live/css/selectors/has-argument-with-explicit-scope.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-argument-with-explicit-scope.html)
- [has-basic.html](https://wpt.fyi/results/css/selectors/has-basic.html) [(live test)](http://wpt.live/css/selectors/has-basic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-basic.html)
- [has-display-none-checked.html](https://wpt.fyi/results/css/selectors/has-display-none-checked.html) [(live test)](http://wpt.live/css/selectors/has-display-none-checked.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-display-none-checked.html)
- [has-focus-display-change.html](https://wpt.fyi/results/css/selectors/has-focus-display-change.html) [(live test)](http://wpt.live/css/selectors/has-focus-display-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-focus-display-change.html)
- [has-matches-to-uninserted-elements.html](https://wpt.fyi/results/css/selectors/has-matches-to-uninserted-elements.html) [(live test)](http://wpt.live/css/selectors/has-matches-to-uninserted-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-matches-to-uninserted-elements.html)
- [has-nth-of-crash.html](https://wpt.fyi/results/css/selectors/has-nth-of-crash.html) [(live test)](http://wpt.live/css/selectors/has-nth-of-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-nth-of-crash.html)
- [has-relative-argument.html](https://wpt.fyi/results/css/selectors/has-relative-argument.html) [(live test)](http://wpt.live/css/selectors/has-relative-argument.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-relative-argument.html)
- [has-sibling-chrome-crash.html](https://wpt.fyi/results/css/selectors/has-sibling-chrome-crash.html) [(live test)](http://wpt.live/css/selectors/has-sibling-chrome-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-sibling-chrome-crash.html)
- [has-specificity.html](https://wpt.fyi/results/css/selectors/has-specificity.html) [(live test)](http://wpt.live/css/selectors/has-specificity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-specificity.html)
- [has-style-sharing-001.html](https://wpt.fyi/results/css/selectors/has-style-sharing-001.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-001.html)
- [has-style-sharing-002.html](https://wpt.fyi/results/css/selectors/has-style-sharing-002.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-002.html)
- [has-style-sharing-003.html](https://wpt.fyi/results/css/selectors/has-style-sharing-003.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-003.html)
- [has-style-sharing-004.html](https://wpt.fyi/results/css/selectors/has-style-sharing-004.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-004.html)
- [has-style-sharing-005.html](https://wpt.fyi/results/css/selectors/has-style-sharing-005.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-005.html)
- [has-style-sharing-006.html](https://wpt.fyi/results/css/selectors/has-style-sharing-006.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-006.html)
- [has-style-sharing-007.html](https://wpt.fyi/results/css/selectors/has-style-sharing-007.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-007.html)
- [has-style-sharing-pseudo-001.html](https://wpt.fyi/results/css/selectors/has-style-sharing-pseudo-001.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-pseudo-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-pseudo-001.html)
- [has-style-sharing-pseudo-002.html](https://wpt.fyi/results/css/selectors/has-style-sharing-pseudo-002.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-pseudo-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-pseudo-002.html)
- [has-style-sharing-pseudo-003.html](https://wpt.fyi/results/css/selectors/has-style-sharing-pseudo-003.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-pseudo-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-pseudo-003.html)
- [has-style-sharing-pseudo-004.html](https://wpt.fyi/results/css/selectors/has-style-sharing-pseudo-004.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-pseudo-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-pseudo-004.html)
- [has-style-sharing-pseudo-005.html](https://wpt.fyi/results/css/selectors/has-style-sharing-pseudo-005.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-pseudo-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-pseudo-005.html)
- [has-style-sharing-pseudo-006.html](https://wpt.fyi/results/css/selectors/has-style-sharing-pseudo-006.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-pseudo-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-pseudo-006.html)
- [has-style-sharing-pseudo-007.html](https://wpt.fyi/results/css/selectors/has-style-sharing-pseudo-007.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-pseudo-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-pseudo-007.html)
- [has-style-sharing-pseudo-008.html](https://wpt.fyi/results/css/selectors/has-style-sharing-pseudo-008.html) [(live test)](http://wpt.live/css/selectors/has-style-sharing-pseudo-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-style-sharing-pseudo-008.html)
- [has-visited.html](https://wpt.fyi/results/css/selectors/has-visited.html) [(live test)](http://wpt.live/css/selectors/has-visited.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/has-visited.html)
- [attribute-or-elemental-selectors-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/attribute-or-elemental-selectors-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/attribute-or-elemental-selectors-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/attribute-or-elemental-selectors-in-has.html)
- [child-indexed-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/child-indexed-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/child-indexed-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/child-indexed-pseudo-classes-in-has.html)
- [has-pseudoclass-only-crash.html](https://wpt.fyi/results/css/selectors/invalidation/crashtests/has-pseudoclass-only-crash.html) [(live test)](http://wpt.live/css/selectors/invalidation/crashtests/has-pseudoclass-only-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/crashtests/has-pseudoclass-only-crash.html)
- [defined-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/defined-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/defined-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/defined-in-has.html)
- [dir-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/dir-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/dir-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/dir-pseudo-class-in-has.html)
- [empty-pseudo-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/empty-pseudo-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/empty-pseudo-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/empty-pseudo-in-has.html)
- [empty-pseudo-in-has-display-none.html](https://wpt.fyi/results/css/selectors/invalidation/empty-pseudo-in-has-display-none.html) [(live test)](http://wpt.live/css/selectors/invalidation/empty-pseudo-in-has-display-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/empty-pseudo-in-has-display-none.html)
- [fullscreen-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/fullscreen-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/fullscreen-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/fullscreen-pseudo-class-in-has.html)
- [has-append-first-node.html](https://wpt.fyi/results/css/selectors/invalidation/has-append-first-node.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-append-first-node.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-append-first-node.html)
- [has-complexity.html](https://wpt.fyi/results/css/selectors/invalidation/has-complexity.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-complexity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-complexity.html)
- [has-css-nesting-shared.html](https://wpt.fyi/results/css/selectors/invalidation/has-css-nesting-shared.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-css-nesting-shared.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-css-nesting-shared.html)
- [has-in-adjacent-position.html](https://wpt.fyi/results/css/selectors/invalidation/has-in-adjacent-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-in-adjacent-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-in-adjacent-position.html)
- [has-in-ancestor-position.html](https://wpt.fyi/results/css/selectors/invalidation/has-in-ancestor-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-in-ancestor-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-in-ancestor-position.html)
- [has-in-is-non-subject-compound.html](https://wpt.fyi/results/css/selectors/invalidation/has-in-is-non-subject-compound.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-in-is-non-subject-compound.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-in-is-non-subject-compound.html)
- [has-in-parent-position.html](https://wpt.fyi/results/css/selectors/invalidation/has-in-parent-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-in-parent-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-in-parent-position.html)
- [has-in-sibling-position.html](https://wpt.fyi/results/css/selectors/invalidation/has-in-sibling-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-in-sibling-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-in-sibling-position.html)
- [has-invalidation-after-removing-non-first-element.html](https://wpt.fyi/results/css/selectors/invalidation/has-invalidation-after-removing-non-first-element.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-invalidation-after-removing-non-first-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-invalidation-after-removing-non-first-element.html)
- [has-invalidation-first-in-sibling-chain.html](https://wpt.fyi/results/css/selectors/invalidation/has-invalidation-first-in-sibling-chain.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-invalidation-first-in-sibling-chain.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-invalidation-first-in-sibling-chain.html)
- [has-invalidation-for-wiping-an-element.html](https://wpt.fyi/results/css/selectors/invalidation/has-invalidation-for-wiping-an-element.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-invalidation-for-wiping-an-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-invalidation-for-wiping-an-element.html)
- [has-nested-pseudo-001-crash.html](https://wpt.fyi/results/css/selectors/invalidation/has-nested-pseudo-001-crash.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-nested-pseudo-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-nested-pseudo-001-crash.html)
- [has-nested-pseudo-002-crash.html](https://wpt.fyi/results/css/selectors/invalidation/has-nested-pseudo-002-crash.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-nested-pseudo-002-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-nested-pseudo-002-crash.html)
- [has-nested-pseudo-003-crash.html](https://wpt.fyi/results/css/selectors/invalidation/has-nested-pseudo-003-crash.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-nested-pseudo-003-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-nested-pseudo-003-crash.html)
- [has-pseudo-element.html](https://wpt.fyi/results/css/selectors/invalidation/has-pseudo-element.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-pseudo-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-pseudo-element.html)
- [has-pseudoclass-only.html](https://wpt.fyi/results/css/selectors/invalidation/has-pseudoclass-only.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-pseudoclass-only.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-pseudoclass-only.html)
- [has-pseudo-element-subject-child-mutation.html](https://wpt.fyi/results/css/selectors/invalidation/has-pseudo-element-subject-child-mutation.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-pseudo-element-subject-child-mutation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-pseudo-element-subject-child-mutation.html)
- [has-sibling-insertion-removal.html](https://wpt.fyi/results/css/selectors/invalidation/has-sibling-insertion-removal.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-sibling-insertion-removal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-sibling-insertion-removal.html)
- [has-sibling.html](https://wpt.fyi/results/css/selectors/invalidation/has-sibling.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-sibling.html)
- [has-side-effect.html](https://wpt.fyi/results/css/selectors/invalidation/has-side-effect.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-side-effect.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-side-effect.html)
- [has-unstyled.html](https://wpt.fyi/results/css/selectors/invalidation/has-unstyled.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-unstyled.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-unstyled.html)
- [has-with-is-child-combinator.html](https://wpt.fyi/results/css/selectors/invalidation/has-with-is-child-combinator.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-with-is-child-combinator.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-with-is-child-combinator.html)
- [has-with-nesting-parent-containing-complex.html](https://wpt.fyi/results/css/selectors/invalidation/has-with-nesting-parent-containing-complex.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-with-nesting-parent-containing-complex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-with-nesting-parent-containing-complex.html)
- [has-with-nesting-parent-containing-hover.html](https://wpt.fyi/results/css/selectors/invalidation/has-with-nesting-parent-containing-hover.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-with-nesting-parent-containing-hover.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-with-nesting-parent-containing-hover.html)
- [has-with-not.html](https://wpt.fyi/results/css/selectors/invalidation/has-with-not.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-with-not.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-with-not.html)
- [has-with-nth-child-sibling-remove.html](https://wpt.fyi/results/css/selectors/invalidation/has-with-nth-child-sibling-remove.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-with-nth-child-sibling-remove.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-with-nth-child-sibling-remove.html)
- [has-with-nth-child.html](https://wpt.fyi/results/css/selectors/invalidation/has-with-nth-child.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-with-nth-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-with-nth-child.html)
- [has-with-pseudo-class.html](https://wpt.fyi/results/css/selectors/invalidation/has-with-pseudo-class.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-with-pseudo-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-with-pseudo-class.html)
- [host-context-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/host-context-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/host-context-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/host-context-pseudo-class-in-has.html)
- [host-has-shadow-tree-element-at-nonsubject-position.html](https://wpt.fyi/results/css/selectors/invalidation/host-has-shadow-tree-element-at-nonsubject-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/host-has-shadow-tree-element-at-nonsubject-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/host-has-shadow-tree-element-at-nonsubject-position.html)
- [host-has-shadow-tree-element-at-subject-position.html](https://wpt.fyi/results/css/selectors/invalidation/host-has-shadow-tree-element-at-subject-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/host-has-shadow-tree-element-at-subject-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/host-has-shadow-tree-element-at-subject-position.html)
- [host-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/host-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/host-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/host-pseudo-class-in-has.html)
- [input-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/input-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/input-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/input-pseudo-classes-in-has.html)
- [input-in-range-in-has-with-readonly.html](https://wpt.fyi/results/css/selectors/invalidation/input-in-range-in-has-with-readonly.html) [(live test)](http://wpt.live/css/selectors/invalidation/input-in-range-in-has-with-readonly.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/input-in-range-in-has-with-readonly.html)
- [is-pseudo-containing-complex-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/is-pseudo-containing-complex-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/is-pseudo-containing-complex-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/is-pseudo-containing-complex-in-has.html)
- [is-pseudo-containing-sibling-relationship-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/is-pseudo-containing-sibling-relationship-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/is-pseudo-containing-sibling-relationship-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/is-pseudo-containing-sibling-relationship-in-has.html)
- [lang-pseudo-class-in-has-document-element.html](https://wpt.fyi/results/css/selectors/invalidation/lang-pseudo-class-in-has-document-element.html) [(live test)](http://wpt.live/css/selectors/invalidation/lang-pseudo-class-in-has-document-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/lang-pseudo-class-in-has-document-element.html)
- [lang-pseudo-class-in-has-multiple-document-elements.html](https://wpt.fyi/results/css/selectors/invalidation/lang-pseudo-class-in-has-multiple-document-elements.html) [(live test)](http://wpt.live/css/selectors/invalidation/lang-pseudo-class-in-has-multiple-document-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/lang-pseudo-class-in-has-multiple-document-elements.html)
- [lang-pseudo-class-in-has-xhtml.xhtml](https://wpt.fyi/results/css/selectors/invalidation/lang-pseudo-class-in-has-xhtml.xhtml) [(live test)](http://wpt.live/css/selectors/invalidation/lang-pseudo-class-in-has-xhtml.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/lang-pseudo-class-in-has-xhtml.xhtml)
- [lang-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/lang-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/lang-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/lang-pseudo-class-in-has.html)
- [link-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/link-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/link-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/link-pseudo-class-in-has.html)
- [link-pseudo-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/link-pseudo-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/link-pseudo-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/link-pseudo-in-has.html)
- [location-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/location-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/location-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/location-pseudo-classes-in-has.html)
- [media-loading-pseudo-classes-in-has.sub.html](https://wpt.fyi/results/css/selectors/invalidation/media-loading-pseudo-classes-in-has.sub.html) [(live test)](http://wpt.live/css/selectors/invalidation/media-loading-pseudo-classes-in-has.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/media-loading-pseudo-classes-in-has.sub.html)
- [media-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/media-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/media-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/media-pseudo-classes-in-has.html)
- [modal-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/modal-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/modal-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/modal-pseudo-class-in-has.html)
- [negated-has-in-nonsubject-position.html](https://wpt.fyi/results/css/selectors/invalidation/negated-has-in-nonsubject-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-has-in-nonsubject-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-has-in-nonsubject-position.html)
- [not-pseudo-containing-complex-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/not-pseudo-containing-complex-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/not-pseudo-containing-complex-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/not-pseudo-containing-complex-in-has.html)
- [not-pseudo-containing-sibling-relationship-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/not-pseudo-containing-sibling-relationship-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/not-pseudo-containing-sibling-relationship-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/not-pseudo-containing-sibling-relationship-in-has.html)
- [open-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/open-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/open-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/open-pseudo-class-in-has.html)
- [state-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/state-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/state-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/state-in-has.html)
- [subject-has-invalidation-with-display-none-anchor-element.html](https://wpt.fyi/results/css/selectors/invalidation/subject-has-invalidation-with-display-none-anchor-element.html) [(live test)](http://wpt.live/css/selectors/invalidation/subject-has-invalidation-with-display-none-anchor-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/subject-has-invalidation-with-display-none-anchor-element.html)
- [target-pseudo-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/target-pseudo-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/target-pseudo-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/target-pseudo-in-has.html)
- [typed-child-indexed-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/typed-child-indexed-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/typed-child-indexed-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/typed-child-indexed-pseudo-classes-in-has.html)
- [user-action-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/user-action-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/user-action-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/user-action-pseudo-classes-in-has.html)
- [parse-has-disallow-nesting-has-inside-has.html](https://wpt.fyi/results/css/selectors/parsing/parse-has-disallow-nesting-has-inside-has.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-has-disallow-nesting-has-inside-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-has-disallow-nesting-has-inside-has.html)
- [parse-has-forgiving-selector.html](https://wpt.fyi/results/css/selectors/parsing/parse-has-forgiving-selector.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-has-forgiving-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-has-forgiving-selector.html)
- [parse-has.html](https://wpt.fyi/results/css/selectors/parsing/parse-has.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-has.html)

The relational pseudo-class, <a id="has-pseudo"></a><strong>:has()</strong>, is a functional pseudo-class taking a <a id="ref-for-typedef-relative-selector-list①"></a>[\<relative-selector-list\>](#typedef-relative-selector-list) as an argument. It represents an element if any of the <a id="ref-for-relative-selector①"></a>[relative selectors](#relative-selector) would match at least one element when <a id="ref-for-relative-selector-anchor-elements②"></a>[anchored against](#relative-selector-anchor-elements) this element.

The <a id="ref-for-has-pseudo③"></a>[:has()](#has-pseudo) pseudo-class cannot be nested; <a id="ref-for-has-pseudo④"></a>:has() is not valid within <a id="ref-for-has-pseudo⑤"></a>:has(). Also, unless explicitly defined as a <a id="has-allowed-pseudo-element"></a><strong>:has-allowed pseudo-element</strong>, <a id="ref-for-pseudo-element②⑧"></a>[pseudo-elements](#pseudo-element) are not valid selectors within <a id="ref-for-has-pseudo⑥"></a>:has(). (This specification does not define any <a id="ref-for-has-allowed-pseudo-element"></a>[:has-allowed pseudo-elements](#has-allowed-pseudo-element), but other specifications may do so.)

Note: Pseudo-elements are generally excluded from <a id="ref-for-has-pseudo⑦"></a>[:has()](#has-pseudo) because many of them exist conditionally, based on the styling of their ancestors, so allowing these to be queried by <a id="ref-for-has-pseudo⑧"></a>:has() would introduce cycles.

Note: Since <a id="ref-for-has-pseudo⑨"></a>[:has()](#has-pseudo) takes a <a id="ref-for-typedef-relative-selector-list②"></a>[\<relative-selector-list\>](#typedef-relative-selector-list), its arguments are <em>inherently</em> <a id="ref-for-complex④"></a>[complex selectors](#complex) (because they start, perhaps implicitly, with a combinator). This means <a id="ref-for-has-pseudo①⓪"></a>:has() cannot be used in contexts that don’t allow complex selectors; its arguments will be guaranteed to be invalid.

<a id="example-61d52da1"></a>

<strong>Example:</strong>

[](#example-61d52da1) For example, the following selector matches only <code>&#60;a&#62;</code> elements that contain an <code>&#60;img&#62;</code> child:

``` text
a:has(> img)
```

The following selector matches a <code>&#60;dt&#62;</code> element immediately followed by another <code>&#60;dt&#62;</code> element:

``` text
dt:has(+ dt)
```

The following selector matches <code>&#60;section&#62;</code> elements that don’t contain any heading elements:

``` text
section:not(:has(h1, h2, h3, h4, h5, h6))
```

Note that ordering matters in the above selector. Swapping the nesting of the two pseudo-classes, like:

``` text
section:has(:not(h1, h2, h3, h4, h5, h6))
```

...would result in matching any <code>&#60;section&#62;</code> element which contains anything that’s not a heading element.

## <a id="elemental-selectors"></a>5.  Elemental selectors[](#elemental-selectors)

### <a id="type-selectors"></a>5.1.  Type (tag name) selector[](#type-selectors)

A <a id="type-selector"></a><strong>type selector</strong> is the name of a document language element type, and represents an instance of that element type in the document tree.

<a id="example-087f4a77"></a>

<strong>Example:</strong>

[](#example-087f4a77) For example, the selector h1 represents an <a id="ref-for-the-h1-element"></a>[h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element in the document.

A <a id="ref-for-type-selector⑤"></a>[type selector](#type-selector) is written as a <a id="ref-for-css-qualified-name"></a>[CSS qualified name](https://drafts.csswg.org/css-namespaces-3/#css-qualified-name): an <a id="ref-for-css-css-identifier②"></a>[identifier](https://drafts.csswg.org/css-values-4/#css-css-identifier) with an optional namespace prefix. [\[CSS3NAMESPACE\]](#biblio-css3namespace) (See [§ 5.3 Namespaces in Elemental Selectors](#type-nmsp).)

### <a id="the-universal-selector"></a>5.2.  Universal selector [](#the-universal-selector)

Tests

- [parse-universal.html](https://wpt.fyi/results/css/selectors/parsing/parse-universal.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-universal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-universal.html)

The <a id="universal-selector"></a><strong>universal selector</strong> is a special <a id="ref-for-type-selector⑥"></a>[type selector](#type-selector), that represents an element of any element type.

It is written as a <a id="ref-for-css-qualified-name①"></a>[CSS qualified name](https://drafts.csswg.org/css-namespaces-3/#css-qualified-name) with an asterisk (<code>&#42;</code> U+002A) as the local name. Like a <a id="ref-for-type-selector⑦"></a>[type selector](#type-selector), the <a id="ref-for-universal-selector⑦"></a>[universal selector](#universal-selector) can be qualified by a namespace, restricting it to only elements belonging to that namespace, and is affected by a default namespace as defined in [§ 5.3 Namespaces in Elemental Selectors](#type-nmsp).

Unless an element is <a id="ref-for-featureless⑦"></a>[featureless](#featureless), the presence of a <a id="ref-for-universal-selector⑧"></a>[universal selector](#universal-selector) has no effect on whether the element matches the selector. (<a id="ref-for-featureless⑧"></a>Featureless elements do not match any selector, including the <a id="ref-for-universal-selector⑨"></a>universal selector.)

<a id="example-0f5d51ca"></a>

<strong>Example:</strong>

[](#example-0f5d51ca)

- \*\[hreflang\|=en\] and \[hreflang\|=en\] are equivalent,
- \*.warning and .warning are equivalent,
- \*#myid and \#myid are equivalent.

The <a id="ref-for-universal-selector①⓪"></a>[universal selector](#universal-selector) follows the same syntax rules as other <a id="ref-for-type-selector⑧"></a>[type selectors](#type-selector): only one can appear per <a id="ref-for-compound③③"></a>[compound selector](#compound), and it must be the first <a id="ref-for-simple①⑤"></a>[simple selector](#simple) in the <a id="ref-for-compound③④"></a>compound selector.

Note: In some cases, adding a <a id="ref-for-universal-selector①①"></a>[universal selector](#universal-selector) can make a selector easier to read, even though it has no effect on the matching behavior. For example, div :first-child and div:first-child are somewhat difficult to tell apart at a quick glance, but writing the former as div \*:first-child makes the difference obvious.

### <a id="type-nmsp"></a>5.3.  Namespaces in Elemental Selectors[](#type-nmsp)

<a id="ref-for-type-selector⑨"></a>[Type selectors](#type-selector) and <a id="ref-for-universal-selector①②"></a>[universal selectors](#universal-selector) allow an optional namespace component: a namespace prefix that has been previously <a id="ref-for-nsdecl"></a>[declared](#nsdecl) may be prepended to the element name separated by the namespace separator “vertical bar” (<code>&#124;</code> U+007C). (See, e.g., [\[XML-NAMES\]](#biblio-xml-names) for the use of namespaces in XML.) It has the following meaning in each form:

<strong><code>ns&#124;E</code></strong>

elements with name E in namespace ns

<strong><code>&#42;&#124;E</code></strong>

elements with name E in any namespace, including those without a namespace

<strong><code>&#124;E</code></strong>

elements with name E without a namespace

<strong><code>E</code></strong>

if no default namespace has been <a id="ref-for-nsdecl②"></a>[declared](#nsdecl) for selectors, this is equivalent to \*\|E. Otherwise it is equivalent to ns\|E where ns is the default namespace.

<a id="example-53e1ab95"></a>

<strong>Example:</strong>

[](#example-53e1ab95) CSS examples:

``` text
@namespace foo url(http://www.example.com);
foo|h1 { color: blue }  /* first rule */
foo|* { color: yellow } /* second rule */
|h1 { color: red }      /* ...*/
*|h1 { color: green }
h1 { color: green }
```

The first rule (not counting the <a id="ref-for-at-ruledef-namespace①"></a>[&#64;namespace](https://drafts.csswg.org/css-namespaces-3/#at-ruledef-namespace) at-rule) will match only <a id="ref-for-the-h1-element①"></a>[h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) elements in the "http&#58;//www&#46;example&#46;com" namespace.

The second rule will match all elements in the "http&#58;//www&#46;example&#46;com" namespace.

The third rule will match only <a id="ref-for-the-h1-element②"></a>[h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) elements with no namespace.

The fourth rule will match <a id="ref-for-the-h1-element③"></a>[h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) elements in any namespace (including those without any namespace).

The last rule is equivalent to the fourth rule because no default namespace has been defined.

If a <a id="ref-for-default-namespace"></a>[default namespace](https://drafts.csswg.org/css-namespaces-3/#default-namespace) is declared, <a id="ref-for-compound③⑤"></a>[compound selectors](#compound) without <a id="ref-for-type-selector①⓪"></a>[type selectors](#type-selector) in them still only match elements in that default namespace.

<a id="example-03281ceb"></a>

<strong>Example:</strong>

[](#example-03281ceb) For example, in the following style sheet:

``` text
@namespace url("http://example.com/foo");

.special { ... }
```

The .special selector only matches elements in the "http&#58;//example&#46;com/foo" namespace, even though no reference to the type name (which is paired with the namespace in the DOM) appeared.

A <a id="ref-for-type-selector①①"></a>[type selector](#type-selector) or <a id="ref-for-universal-selector①③"></a>[universal selector](#universal-selector) containing a namespace prefix that has not been previously <a id="ref-for-nsdecl③"></a>[declared](#nsdecl) is an <a id="ref-for-invalid-selector③"></a>[invalid selector](#invalid-selector).

### <a id="the-defined-pseudo"></a>5.4.  The Defined Pseudo-class: <a id="ref-for-defined-pseudo"></a>[:defined](#defined-pseudo)[](#the-defined-pseudo)

Tests

- [defined.html](https://wpt.fyi/results/css/selectors/invalidation/defined.html) [(live test)](http://wpt.live/css/selectors/invalidation/defined.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/defined.html)

In some host languages, elements can have a distinction between being “defined”/“constructed” or not. The <a id="defined-pseudo"></a><strong>:defined</strong> <a id="ref-for-pseudo-class①⑦"></a>[pseudo-class](#pseudo-class) matches elements that are fully defined, as dictated by the host language.

If the host language does not have this sort of distinction, all elements in it match <a id="ref-for-defined-pseudo①"></a>[:defined](#defined-pseudo).

<a id="example-cd9dcdb2"></a>

<strong>Example:</strong>

[](#example-cd9dcdb2) In HTML, all built-in elements are always considered to be defined, so the following example will always match:

``` text
p:defined { ... }
```

<a id="ref-for-custom-element"></a>[Custom elements](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element), on the other hand, start out <em>un</em>defined, and only become defined when <a id="ref-for-element-definition"></a>[properly registered](https://html.spec.whatwg.org/multipage/custom-elements.html#element-definition). This means the <a id="ref-for-defined-pseudo②"></a>[:defined](#defined-pseudo) pseudo-class can be used to hide a custom element until it has been registered:

``` text
custom-element { visibility: hidden }
```

``` text
custom-element:defined { visibility: visible }
```

## <a id="attribute-selectors"></a>6.  Attribute selectors[](#attribute-selectors)

Tests

- [style-attribute-selector.html](https://wpt.fyi/results/css/selectors/attribute-selectors/style-attribute-selector.html) [(live test)](http://wpt.live/css/selectors/attribute-selectors/style-attribute-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/attribute-selectors/style-attribute-selector.html)
- [attribute.html](https://wpt.fyi/results/css/selectors/invalidation/attribute.html) [(live test)](http://wpt.live/css/selectors/invalidation/attribute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/attribute.html)
- [class-id-attr.html](https://wpt.fyi/results/css/selectors/invalidation/class-id-attr.html) [(live test)](http://wpt.live/css/selectors/invalidation/class-id-attr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/class-id-attr.html)
- [missing-right-token.html](https://wpt.fyi/results/css/selectors/missing-right-token.html) [(live test)](http://wpt.live/css/selectors/missing-right-token.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/missing-right-token.html)
- [parse-attribute.html](https://wpt.fyi/results/css/selectors/parsing/parse-attribute.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-attribute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-attribute.html)
- [selectors-attr-many.html](https://wpt.fyi/results/css/selectors/selectors-attr-many.html) [(live test)](http://wpt.live/css/selectors/selectors-attr-many.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-attr-many.html)
- [selectors-attr-many-2.html](https://wpt.fyi/results/css/selectors/selectors-attr-many-2.html) [(live test)](http://wpt.live/css/selectors/selectors-attr-many-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-attr-many-2.html)

Selectors allow the representation of an element’s attributes. When a selector is used as an expression to match against an element, an <a id="attribute-selector"></a><strong>attribute selector</strong> must be considered to match an element if that element has an attribute that matches the attribute represented by the attribute selector.

<a id="issue-745ef775"></a>

<strong>Issue:</strong>

[](#issue-745ef775)Add comma-separated syntax for [multiple-value matching](http://lists.w3.org/Archives/Public/www-style/2011Mar/0215.html)? e.g. \[rel ~= next, prev, up, first, last\]

### <a id="attribute-representation"></a>6.1.  Attribute presence and value selectors[](#attribute-representation)

CSS2 introduced four attribute selectors:

<strong>\[att\]</strong>

Represents an element with the <code>att</code> attribute, whatever the value of the attribute.

<strong>\[att=val\]</strong>

Represents an element with the <code>att</code> attribute whose value is exactly "val".

<strong>\[att~=val\]</strong>

Represents an element with the <code>att</code> attribute whose value is a <a id="ref-for-whitespace③"></a>[whitespace](#whitespace)-separated list of words, one of which is exactly "val". If "val" contains whitespace, it will never represent anything (since the words are <em>separated</em> by spaces). Also if "val" is the empty string, it will never represent anything.

<strong>\[att\|=val\]</strong>

Represents an element with the <code>att</code> attribute, its value either being exactly "val" or beginning with "val" immediately followed by "-" (U+002D). This is primarily intended to allow language subcode matches (e.g., the <code>hreflang</code> attribute on the <a id="ref-for-the-a-element③"></a>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) element in HTML) as described in BCP 47 ([\[BCP47\]](#biblio-bcp47)). For <code>lang</code> (or <code>xml&#58;lang</code>) language subcode matching, please see the <a id="ref-for-lang-pseudo③"></a>[:lang()](#lang-pseudo) pseudo-class.

Attribute values must be <a id="ref-for-typedef-ident-token"></a>[\<ident-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-ident-token)s or <a id="ref-for-typedef-string-token"></a>[\<string-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-string-token)s. [\[CSS3SYN\]](#biblio-css3syn)

<a id="example-52ee808b"></a>

<strong>Example:</strong>

[](#example-52ee808b) Examples:

The following attribute selector represents an <a id="ref-for-the-h1-element④"></a>[h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element that carries the <code>title</code> attribute, whatever its value:

``` text
h1[title]
```

In the following example, the selector represents a <a id="ref-for-the-span-element"></a>[span](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-span-element) element whose <code>class</code> attribute has exactly the value "example":

``` text
span[class="example"]
```

Multiple attribute selectors can be used to represent several attributes of an element, or several conditions on the same attribute. Here, the selector represents a <a id="ref-for-the-span-element①"></a>[span](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-span-element) element whose <code>hello</code> attribute has exactly the value "Cleveland" and whose <code>goodbye</code> attribute has exactly the value "Columbus":

``` text
span[hello="Cleveland"][goodbye="Columbus"]
```

The following CSS rules illustrate the differences between "=" and "~=". The first selector would match, for example, an <a id="ref-for-the-a-element④"></a>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) element with the value "copyright copyleft copyeditor" on a <code>rel</code> attribute. The second selector would only match an <a id="ref-for-the-a-element⑤"></a>a element with an <code>href</code> attribute having the exact value "http&#58;//www&#46;w3&#46;org/".

``` text
a[rel~="copyright"] { ... }
a[href="http://www.w3.org/"] { ... }
```

The following selector represents an <a id="ref-for-the-a-element⑥"></a>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) element whose <code>hreflang</code> attribute is exactly "fr".

``` text
a[hreflang=fr] 
```

The following selector represents an <a id="ref-for-the-a-element⑦"></a>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) element for which the value of the <code>hreflang</code> attribute begins with "en", including "en", "en-US", and "en-scouse":

``` text
a[hreflang|="en"] 
```

The following selectors represent a DIALOGUE element whenever it has one of two different values for an attribute <code>character</code>:

``` text
DIALOGUE[character=romeo]
DIALOGUE[character=juliet]
```

### <a id="attribute-substrings"></a>6.2.  Substring matching attribute selectors[](#attribute-substrings)

Three additional attribute selectors are provided for matching substrings in the value of an attribute:

<strong>\[att^=val\]</strong>

Represents an element with the <code>att</code> attribute whose value begins with the prefix "val". If "val" is the empty string then the selector does not represent anything.

<strong>\[att\$=val\]</strong>

Represents an element with the <code>att</code> attribute whose value ends with the suffix "val". If "val" is the empty string then the selector does not represent anything.

<strong>\[att\*=val\]</strong>

Represents an element with the <code>att</code> attribute whose value contains at least one instance of the substring "val". If "val" is the empty string then the selector does not represent anything.

Attribute values must be <a id="ref-for-typedef-ident-token①"></a>[\<ident-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-ident-token)s or <a id="ref-for-typedef-string-token①"></a>[\<string-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-string-token)s.

<a id="example-d887e2c3"></a>

<strong>Example:</strong>

[](#example-d887e2c3) Examples: The following selector represents an HTML <a id="ref-for-the-object-element"></a>[object](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-object-element) element, referencing an image:

``` text
object[type^="image/"] 
```

The following selector represents an HTML <a id="ref-for-the-a-element⑧"></a>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) element with an <code>href</code> attribute whose value ends with ".html".

``` text
a[href$=".html"] 
```

The following selector represents an HTML paragraph with a <code>title</code> attribute whose value contains the substring "hello"

``` text
p[title*="hello"] 
```

### <a id="attribute-case"></a>6.3.  Case-sensitivity[](#attribute-case)

Tests

- [cssom.html](https://wpt.fyi/results/css/selectors/attribute-selectors/attribute-case/cssom.html) [(live test)](http://wpt.live/css/selectors/attribute-selectors/attribute-case/cssom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/attribute-selectors/attribute-case/cssom.html)
- [semantics.html](https://wpt.fyi/results/css/selectors/attribute-selectors/attribute-case/semantics.html) [(live test)](http://wpt.live/css/selectors/attribute-selectors/attribute-case/semantics.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/attribute-selectors/attribute-case/semantics.html)
- [syntax.html](https://wpt.fyi/results/css/selectors/attribute-selectors/attribute-case/syntax.html) [(live test)](http://wpt.live/css/selectors/attribute-selectors/attribute-case/syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/attribute-selectors/attribute-case/syntax.html)

By default case-sensitivity of attribute names and values in selectors depends on the document language.

To match attribute values <a id="ref-for-ascii-case-insensitive④"></a>[ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive) regardless of document language rules, the attribute selector may include the identifier <a id="valdef-attribute-selector-i"></a><strong>i</strong> before the closing bracket (<code>&#93;</code>). When this flag is present, UAs must match the attribute’s value <a id="ref-for-ascii-case-insensitive⑤"></a>ASCII case-insensitively (i.e. \[a-z\] and \[A-Z\] are considered equivalent).

<a id="example-eb510622"></a>

<strong>Example:</strong>

[](#example-eb510622) For example, ASCII case insensitivity allows <code>green</code> to match <code>GREEN</code>. However, <code>grün</code> would not match <code>GRÜN</code>.

Alternately, the attribute selector may include the identifier <a id="valdef-attribute-selector-s"></a><strong>s</strong> before the closing bracket (<code>&#93;</code>); in this case the UA must match the value case-sensitively, with “<a id="ref-for-is①"></a>[identical to](https://infra.spec.whatwg.org/#is)” semantics [\[INFRA\]](#biblio-infra), regardless of document language rules.

Like the rest of Selectors syntax, the <code>i</code> and <code>s</code> identifiers themselves are <a id="ref-for-ascii-case-insensitive⑥"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive).

<a id="example-348bad1b"></a>

<strong>Example:</strong>

[](#example-348bad1b) The following rule will style the <code>frame</code> attribute when it has a value of <code>hsides</code>, whether that value is represented as <code>hsides</code>, <code>HSIDES</code>, <code>hSides</code>, etc. even in an XML environment where attribute values are case-sensitive.

``` text
[frame=hsides i] { border-style: solid none; } 
```

<a id="example-f596236d"></a>

<strong>Example:</strong>

[](#example-f596236d) The following rule will style lists with <code>type=&#34;a&#34;</code> attributes differently than <code>type=&#34;A&#34;</code> even though HTML defines the <code>type</code> attribute to be case-insensitive.

``` text
[type="a" s] { list-style: lower-alpha; }
[type="A" s] { list-style: upper-alpha; }
```

Note: Some document models normalize case-insensitive attribute values at parse time such that checking if a string is case-sensitive matching is impossible. Case-sensitive matching via <code>s</code> flags is only possible in systems that preserve the original case.

### <a id="attrnmsp"></a>6.4.  Attribute selectors and namespaces[](#attrnmsp)

Tests

- [selectors-namespace-001.xml](https://wpt.fyi/results/css/selectors/selectors-namespace-001.xml) [(live test)](http://wpt.live/css/selectors/selectors-namespace-001.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-namespace-001.xml)

The attribute name in an attribute selector is given as a <a id="ref-for-css-qualified-name②"></a>[CSS qualified name](https://drafts.csswg.org/css-namespaces-3/#css-qualified-name): a namespace prefix that has been previously <a id="ref-for-nsdecl④"></a>[declared](#nsdecl) may be prepended to the attribute name separated by the namespace separator "vertical bar" (<code>&#124;</code>). In keeping with the Namespaces in the XML recommendation, default namespaces do not apply to attributes, therefore attribute selectors without a namespace component apply only to attributes that have no namespace (equivalent to \|attr). An asterisk may be used for the namespace prefix indicating that the selector is to match all attribute names without regard to the attribute’s namespace.

An attribute selector with an attribute name containing a namespace prefix that has not been previously <a id="ref-for-nsdecl①"></a>[declared](#nsdecl) is an invalid selector.

<a id="example-a7ca2cb1"></a>

<strong>Example:</strong>

[](#example-a7ca2cb1) CSS examples:

``` text
@namespace foo "http://www.example.com";
[foo|att=val] { color: blue }
[*|att] { color: yellow }
[|att] { color: green }
[att] { color: green }
```

The first rule will match only elements with the attribute <code>att</code> in the "http&#58;//www&#46;example&#46;com" namespace with the value "val".

The second rule will match only elements with the attribute <code>att</code> regardless of the namespace of the attribute (including no namespace).

The last two rules are equivalent and will match only elements with the attribute <code>att</code> where the attribute is not in a namespace.

### <a id="def-values"></a>6.5.  Default attribute values in DTDs[](#def-values)

Attribute selectors represent attribute values in the document tree. How that document tree is constructed is outside the scope of Selectors. In some document formats default attribute values can be defined in a DTD or elsewhere, but these can only be selected by attribute selectors if they appear in the document tree. Selectors should be designed so that they work whether or not the default values are included in the document tree.

For example, a XML UA may, but is <em>not</em> required to, read an “external subset” of the DTD, but <em>is</em> required to look for default attribute values in the document’s “internal subset”. (See, e.g., [\[XML10\]](#biblio-xml10) for definitions of these subsets.) Depending on the UA, a default attribute value defined in the external subset of the DTD might or might not appear in the document tree.

A UA that recognizes an XML namespace may, but is not required to use its knowledge of that namespace to treat default attribute values as if they were present in the document. (For example, an XHTML UA is not required to use its built-in knowledge of the XHTML DTD. See, e.g., [\[XML-NAMES\]](#biblio-xml-names) for details on namespaces in XML 1.0.)

Note: Typically, implementations choose to ignore external subsets. This corresponds to the behavior of non-validating processors as defined by the XML specification.

<a id="example-ba3e8ad8"></a> [](#example-ba3e8ad8) Example:

Consider an element <code>EXAMPLE</code> with an attribute <code>radix</code> that has a default value of <code>&#34;decimal&#34;</code>. The DTD fragment might be

``` text
<!ATTLIST EXAMPLE radix (decimal,octal) "decimal"> 
```

If the style sheet contains the rules

``` text
EXAMPLE[radix=decimal] { /*... default property settings ...*/ }
EXAMPLE[radix=octal]   { /*... other settings...*/ }
```

the first rule might not match elements whose <code>radix</code> attribute is set by default, i.e. not set explicitly. To catch all cases, the attribute selector for the default value must be dropped:

``` text
EXAMPLE                { /*... default property settings ...*/ }
EXAMPLE[radix=octal]   { /*... other settings...*/ }
```

Here, because the selector EXAMPLE\[radix=octal\] is more specific than the type selector alone, the style declarations in the second rule will override those in the first for elements that have a <code>radix</code> attribute value of <code>&#34;octal&#34;</code>. Care has to be taken that all property declarations that are to apply only to the default case are overridden in the non-default cases' style rules.

### <a id="class-html"></a>6.6.  Class selectors[](#class-html)

Tests

- [parse-class.html](https://wpt.fyi/results/css/selectors/parsing/parse-class.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-class.html)
- [xml-class-selector.xml](https://wpt.fyi/results/css/selectors/xml-class-selector.xml) [(live test)](http://wpt.live/css/selectors/xml-class-selector.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/xml-class-selector.xml)

The <a id="class-selector"></a><strong>class selector</strong> is given as a full stop (. U+002E) immediately followed by an identifier. It represents an element belonging to the class identified by the identifier, as defined by the document language. For example, in [\[HTML\]](#biblio-html), [\[SVG11\]](#biblio-svg11), and [\[MATHML\]](#biblio-mathml) membership in a class is given by the <code>class</code> attribute: in these languages it is equivalent to the <code>&#126;=</code> notation applied to the local <code>class</code> attribute (i.e. <code>\[class~=<var>identifier</var>\]</code>).

<a id="example-f9c08b5b"></a>

<strong>Example:</strong>

[](#example-f9c08b5b) CSS examples:

We can assign style information to all elements with <code>class&#126;=&#34;pastoral&#34;</code> as follows:

``` text
*.pastoral { color: green }  /* all elements with class~=pastoral */ 
```

or just

``` text
.pastoral { color: green }  /* all elements with class~=pastoral */ 
```

The following assigns style only to H1 elements with <code>class&#126;=&#34;pastoral&#34;</code>:

``` text
H1.pastoral { color: green }  /* H1 elements with class~=pastoral */ 
```

Given these rules, the first <code>H1</code> instance below would not have green text, while the second would:

``` text
<H1>Not green</H1>
<H1 class="pastoral">Very green</H1>
```

The following rule matches any <a id="ref-for-the-p-element"></a>[P](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element whose <code>class</code> attribute has been assigned a list of <a id="ref-for-whitespace④"></a>[whitespace](#whitespace)-separated values that includes both <code>pastoral</code> and <code>marine</code>:

``` text
p.pastoral.marine { color: green } 
```

This rule matches when <code>class=&#34;pastoral blue aqua marine&#34;</code> but does not match for <code>class=&#34;pastoral blue&#34;</code>.

Note: Because CSS gives considerable power to the "class" attribute, authors could conceivably design their own "document language" based on elements with almost no associated presentation (such as <a id="ref-for-the-div-element"></a>[div](https://html.spec.whatwg.org/multipage/grouping-content.html#the-div-element) and <a id="ref-for-the-span-element②"></a>[span](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-span-element) in HTML) and assigning style information through the "class" attribute. Authors should avoid this practice since the structural elements of a document language often have recognized and accepted meanings and author-defined classes may not.

Note: If an element has multiple class attributes, their values must be concatenated with spaces between the values before searching for the class. As of this time the working group is not aware of any manner in which this situation can be reached, however, so this behavior is explicitly non-normative in this specification.

When matching against a document which is in <a id="ref-for-concept-document-quirks"></a>[quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks), class names must be matched <a id="ref-for-ascii-case-insensitive⑦"></a>[ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive); class selectors are otherwise case-sensitive, only matching class names they are <a id="ref-for-is②"></a>[identical to](https://infra.spec.whatwg.org/#is). [\[INFRA\]](#biblio-infra)

### <a id="id-selectors"></a>6.7.  ID selectors[](#id-selectors)

Tests

- [historical-xmlid.xht](https://wpt.fyi/results/css/selectors/historical-xmlid.xht) [(live test)](http://wpt.live/css/selectors/historical-xmlid.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/historical-xmlid.xht)
- [parse-id.html](https://wpt.fyi/results/css/selectors/parsing/parse-id.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-id.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-id.html)

Document languages may contain attributes that are declared to be of type ID. What makes attributes of type ID special is that no two such attributes can have the same value in a conformant document, regardless of the type of the elements that carry them; whatever the document language, an ID typed attribute can be used to uniquely identify its element. In HTML all ID attributes are named <code>id</code>; XML applications may name ID attributes differently, but the same restriction applies. Which attribute on an element is considered the “ID attribute” is defined by the document language.

An <a id="id-selector"></a><strong>ID selector</strong> consists of a “number sign” (U+0023, <code>&#35;</code>) immediately followed by the ID value, which must be a CSS [identifier](https://www.w3.org/TR/CSS21/syndata.html#value-def-identifier). An ID selector represents an element instance that has an identifier that matches the identifier in the ID selector. (It is possible in non-conforming documents for multiple elements to match a single ID selector.)

<a id="example-3e8dc597"></a>

<strong>Example:</strong>

[](#example-3e8dc597) Examples: The following ID selector represents an <a id="ref-for-the-h1-element⑤"></a>[h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element whose ID-typed attribute has the value "chapter1":

``` text
h1#chapter1 
```

The following ID selector represents any element whose ID-typed attribute has the value "chapter1":

``` text
#chapter1 
```

The following selector represents any element whose ID-typed attribute has the value "z98y".

``` text
*#z98y 
```

Note: In XML 1.0 [\[XML10\]](#biblio-xml10), the information about which attribute contains an element’s IDs is contained in a DTD or a schema. When parsing XML, UAs do not always read the DTD, and thus may not know what the ID of an element is (though a UA may have namespace-specific knowledge that allows it to determine which attribute is the ID attribute for that namespace). If a style sheet author knows or suspects that a UA may not know what the ID of an element is, they should use normal attribute selectors instead: \[name=p371\] instead of \#p371.

If an element has multiple ID attributes, all of them must be treated as IDs for that element for the purposes of the ID selector. Such a situation could be reached using mixtures of xml:id, DOM3 Core, XML DTDs, and namespace-specific knowledge.

When matching against a document which is in <a id="ref-for-concept-document-quirks①"></a>[quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks), IDs must be matched <a id="ref-for-ascii-case-insensitive⑧"></a>[ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive); ID selectors are otherwise case-sensitive, only matching IDs they are <a id="ref-for-is③"></a>[identical to](https://infra.spec.whatwg.org/#is). [\[INFRA\]](#biblio-infra)

## <a id="linguistic-pseudos"></a>7.  Linguistic Pseudo-classes[](#linguistic-pseudos)

### <a id="the-dir-pseudo"></a>7.1.  The Directionality Pseudo-class: <a id="ref-for-dir-pseudo"></a>[:dir()](#dir-pseudo)[](#the-dir-pseudo)

Tests

- [dir-pseudo-in-has.html](https://wpt.fyi/results/css/selectors/dir-pseudo-in-has.html) [(live test)](http://wpt.live/css/selectors/dir-pseudo-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-pseudo-in-has.html)
- [dir-pseudo-on-bdi-element.html](https://wpt.fyi/results/css/selectors/dir-pseudo-on-bdi-element.html) [(live test)](http://wpt.live/css/selectors/dir-pseudo-on-bdi-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-pseudo-on-bdi-element.html)
- [dir-pseudo-on-input-element.html](https://wpt.fyi/results/css/selectors/dir-pseudo-on-input-element.html) [(live test)](http://wpt.live/css/selectors/dir-pseudo-on-input-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-pseudo-on-input-element.html)
- [dir-pseudo-update-document-element.html](https://wpt.fyi/results/css/selectors/dir-pseudo-update-document-element.html) [(live test)](http://wpt.live/css/selectors/dir-pseudo-update-document-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-pseudo-update-document-element.html)
- [dir-selector-auto-direction-change-001.html](https://wpt.fyi/results/css/selectors/dir-selector-auto-direction-change-001.html) [(live test)](http://wpt.live/css/selectors/dir-selector-auto-direction-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-auto-direction-change-001.html)
- [dir-selector-auto.html](https://wpt.fyi/results/css/selectors/dir-selector-auto.html) [(live test)](http://wpt.live/css/selectors/dir-selector-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-auto.html)
- [dir-selector-change-001.html](https://wpt.fyi/results/css/selectors/dir-selector-change-001.html) [(live test)](http://wpt.live/css/selectors/dir-selector-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-change-001.html)
- [dir-selector-change-002.html](https://wpt.fyi/results/css/selectors/dir-selector-change-002.html) [(live test)](http://wpt.live/css/selectors/dir-selector-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-change-002.html)
- [dir-selector-change-003.html](https://wpt.fyi/results/css/selectors/dir-selector-change-003.html) [(live test)](http://wpt.live/css/selectors/dir-selector-change-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-change-003.html)
- [dir-selector-change-004.html](https://wpt.fyi/results/css/selectors/dir-selector-change-004.html) [(live test)](http://wpt.live/css/selectors/dir-selector-change-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-change-004.html)
- [dir-selector-ltr-001.html](https://wpt.fyi/results/css/selectors/dir-selector-ltr-001.html) [(live test)](http://wpt.live/css/selectors/dir-selector-ltr-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-ltr-001.html)
- [dir-selector-ltr-002.html](https://wpt.fyi/results/css/selectors/dir-selector-ltr-002.html) [(live test)](http://wpt.live/css/selectors/dir-selector-ltr-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-ltr-002.html)
- [dir-selector-ltr-003.html](https://wpt.fyi/results/css/selectors/dir-selector-ltr-003.html) [(live test)](http://wpt.live/css/selectors/dir-selector-ltr-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-ltr-003.html)
- [dir-selector-querySelector.html](https://wpt.fyi/results/css/selectors/dir-selector-querySelector.html) [(live test)](http://wpt.live/css/selectors/dir-selector-querySelector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-querySelector.html)
- [dir-selector-rtl-001.html](https://wpt.fyi/results/css/selectors/dir-selector-rtl-001.html) [(live test)](http://wpt.live/css/selectors/dir-selector-rtl-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-rtl-001.html)
- [dir-selector-white-space-001.html](https://wpt.fyi/results/css/selectors/dir-selector-white-space-001.html) [(live test)](http://wpt.live/css/selectors/dir-selector-white-space-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-selector-white-space-001.html)
- [dir-style-01a.html](https://wpt.fyi/results/css/selectors/dir-style-01a.html) [(live test)](http://wpt.live/css/selectors/dir-style-01a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-style-01a.html)
- [dir-style-01b.html](https://wpt.fyi/results/css/selectors/dir-style-01b.html) [(live test)](http://wpt.live/css/selectors/dir-style-01b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-style-01b.html)
- [dir-style-02a.html](https://wpt.fyi/results/css/selectors/dir-style-02a.html) [(live test)](http://wpt.live/css/selectors/dir-style-02a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-style-02a.html)
- [dir-style-02b.html](https://wpt.fyi/results/css/selectors/dir-style-02b.html) [(live test)](http://wpt.live/css/selectors/dir-style-02b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-style-02b.html)
- [dir-style-03a.html](https://wpt.fyi/results/css/selectors/dir-style-03a.html) [(live test)](http://wpt.live/css/selectors/dir-style-03a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-style-03a.html)
- [dir-style-03b.html](https://wpt.fyi/results/css/selectors/dir-style-03b.html) [(live test)](http://wpt.live/css/selectors/dir-style-03b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-style-03b.html)
- [dir-style-04.html](https://wpt.fyi/results/css/selectors/dir-style-04.html) [(live test)](http://wpt.live/css/selectors/dir-style-04.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/dir-style-04.html)
- [part-dir.html](https://wpt.fyi/results/css/selectors/invalidation/part-dir.html) [(live test)](http://wpt.live/css/selectors/invalidation/part-dir.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/part-dir.html)

The <a id="dir-pseudo"></a><strong>:dir()</strong> pseudo-class allows the author to write selectors that represent an element based on its directionality as determined by the <a id="ref-for-document-language②"></a>[document language](#document-language). For example, [\[HTML\]](#biblio-html) defines [how to determine the directionality of an element](https://html.spec.whatwg.org/multipage/dom.html#the-directionality), based on a combination of the <code>dir</code> attribute, the surrounding text, and other factors. As another example, the <code>its&#58;dir</code> and <code>dirRule</code> element of the Internationalization Tag Set [\[ITS20\]](#biblio-its20) are able to define the directionality of an element in [\[XML10\]](#biblio-xml10).

The <a id="ref-for-dir-pseudo①"></a>[:dir()](#dir-pseudo) pseudo-class does not select based on stylistic states—for example, the CSS <a id="ref-for-propdef-direction"></a>[direction](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction) property does not affect whether it matches.

The pseudo-class :dir(ltr) represents an element that has a directionality of left-to-right (<code>ltr</code>). The pseudo-class :dir(rtl) represents an element that has a directionality of right-to-left (<code>rtl</code>). The argument to <a id="ref-for-dir-pseudo②"></a>[:dir()](#dir-pseudo) must be a single identifier, otherwise the selector is invalid. White space is optionally allowed between the identifier and the parentheses. Values other than <code>ltr</code> and <code>rtl</code> are not invalid, but do not match anything. (If a future markup spec defines other directionalities, then Selectors may be extended to allow corresponding values.)

The difference between :dir(C) and \[dir=C\] is that \[dir=C\] only performs a comparison against a given attribute on the element, while the :dir(C) pseudo-class uses the UAs knowledge of the document’s semantics to perform the comparison. For example, in HTML, the directionality of an element inherits so that a child without a <code>dir</code> attribute will have the same directionality as its closest ancestor with a valid <code>dir</code> attribute. As another example, in HTML, an element that matches \[dir=auto\] will match either :dir(ltr) or :dir(rtl) depending on the resolved directionality of the elements as determined by its contents. [\[HTML\]](#biblio-html)

### <a id="the-lang-pseudo"></a>7.2.  The Language Pseudo-class: <a id="ref-for-lang-pseudo④"></a>[:lang()](#lang-pseudo)[](#the-lang-pseudo)

Tests

- [css3-selectors-lang-001.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-001.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-001.html)
- [css3-selectors-lang-002.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-002.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-002.html)
- [css3-selectors-lang-004.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-004.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-004.html)
- [css3-selectors-lang-005.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-005.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-005.html)
- [css3-selectors-lang-006.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-006.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-006.html)
- [css3-selectors-lang-007.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-007.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-007.html)
- [css3-selectors-lang-008.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-008.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-008.html)
- [css3-selectors-lang-009.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-009.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-009.html)
- [css3-selectors-lang-010.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-010.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-010.html)
- [css3-selectors-lang-011.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-011.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-011.html)
- [css3-selectors-lang-012.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-012.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-012.html)
- [css3-selectors-lang-014.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-014.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-014.html)
- [css3-selectors-lang-015.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-015.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-015.html)
- [css3-selectors-lang-016.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-016.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-016.html)
- [css3-selectors-lang-021.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-021.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-021.html)
- [css3-selectors-lang-022.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-022.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-022.html)
- [css3-selectors-lang-024.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-024.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-024.html)
- [css3-selectors-lang-025.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-025.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-025.html)
- [css3-selectors-lang-026.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-026.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-026.html)
- [css3-selectors-lang-027.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-027.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-027.html)
- [css3-selectors-lang-028.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-028.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-028.html)
- [css3-selectors-lang-029.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-029.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-029.html)
- [css3-selectors-lang-030.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-030.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-030.html)
- [css3-selectors-lang-031.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-031.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-031.html)
- [css3-selectors-lang-032.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-032.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-032.html)
- [css3-selectors-lang-034.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-034.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-034.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-034.html)
- [css3-selectors-lang-035.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-035.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-035.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-035.html)
- [css3-selectors-lang-036.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-036.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-036.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-036.html)
- [css3-selectors-lang-041.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-041.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-041.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-041.html)
- [css3-selectors-lang-042.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-042.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-042.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-042.html)
- [css3-selectors-lang-044.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-044.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-044.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-044.html)
- [css3-selectors-lang-045.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-045.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-045.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-045.html)
- [css3-selectors-lang-046.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-046.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-046.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-046.html)
- [css3-selectors-lang-047.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-047.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-047.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-047.html)
- [css3-selectors-lang-048.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-048.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-048.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-048.html)
- [css3-selectors-lang-049.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-049.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-049.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-049.html)
- [css3-selectors-lang-050.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-050.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-050.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-050.html)
- [css3-selectors-lang-051.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-051.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-051.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-051.html)
- [css3-selectors-lang-052.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-052.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-052.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-052.html)
- [css3-selectors-lang-054.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-054.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-054.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-054.html)
- [css3-selectors-lang-055.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-055.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-055.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-055.html)
- [css3-selectors-lang-056.html](https://wpt.fyi/results/css/selectors/i18n/css3-selectors-lang-056.html) [(live test)](http://wpt.live/css/selectors/i18n/css3-selectors-lang-056.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/css3-selectors-lang-056.html)
- [lang-pseudo-class-across-shadow-boundaries.html](https://wpt.fyi/results/css/selectors/i18n/lang-pseudo-class-across-shadow-boundaries.html) [(live test)](http://wpt.live/css/selectors/i18n/lang-pseudo-class-across-shadow-boundaries.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/lang-pseudo-class-across-shadow-boundaries.html)
- [lang-pseudo-class-disconnected.html](https://wpt.fyi/results/css/selectors/i18n/lang-pseudo-class-disconnected.html) [(live test)](http://wpt.live/css/selectors/i18n/lang-pseudo-class-disconnected.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/lang-pseudo-class-disconnected.html)
- [lang-pseudo-class-empty-attribute.xhtml](https://wpt.fyi/results/css/selectors/i18n/lang-pseudo-class-empty-attribute.xhtml) [(live test)](http://wpt.live/css/selectors/i18n/lang-pseudo-class-empty-attribute.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/i18n/lang-pseudo-class-empty-attribute.xhtml)
- [part-lang.html](https://wpt.fyi/results/css/selectors/invalidation/part-lang.html) [(live test)](http://wpt.live/css/selectors/invalidation/part-lang.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/part-lang.html)
- [lang-000.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-000.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-000.html)
- [lang-001.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-001.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-001.html)
- [lang-002.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-002.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-002.html)
- [lang-003.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-003.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-003.html)
- [lang-004.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-004.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-004.html)
- [lang-005.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-005.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-005.html)
- [lang-006.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-006.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-006.html)
- [lang-007.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-007.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-007.html)
- [lang-008.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-008.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-008.html)
- [lang-009.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-009.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-009.html)
- [lang-010.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-010.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-010.html)
- [lang-011.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-011.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-011.html)
- [lang-012.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-012.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-012.html)
- [lang-013.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-013.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-013.html)
- [lang-014.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-014.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-014.html)
- [lang-015.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-015.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-015.html)
- [lang-016.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-016.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-016.html)
- [lang-017.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-017.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-017.html)
- [lang-018.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-018.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-018.html)
- [lang-019.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-019.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-019.html)
- [lang-020.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-020.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-020.html)
- [lang-021.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-021.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-021.html)
- [lang-022.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-022.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-022.html)
- [lang-023.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-023.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-023.html)
- [lang-024.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-024.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-024.html)
- [lang-025.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-025.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-025.html)
- [lang-singleton-subtag-matching.html](https://wpt.fyi/results/css/selectors/selectors-4/lang-singleton-subtag-matching.html) [(live test)](http://wpt.live/css/selectors/selectors-4/lang-singleton-subtag-matching.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/lang-singleton-subtag-matching.html)

If the document language specifies how the (human) <a id="ref-for-content-language"></a>[content language](https://drafts.csswg.org/css-text-4/#content-language) of an element is determined, it is possible to write selectors that represent an element based on its <a id="ref-for-content-language①"></a>content language. The <a id="lang-pseudo"></a><strong>:lang()</strong> pseudo-class, which accepts a comma-separated list of one or more <a id="ref-for-language-range①"></a>[language ranges](#language-range), represents an element whose <a id="ref-for-content-language②"></a>content language is one of the languages listed in its argument. Each <a id="language-range"></a><strong>language range</strong> in <a id="ref-for-lang-pseudo⑤"></a>[:lang()](#lang-pseudo) is an [extended language range](https://www.rfc-editor.org/rfc/rfc4647.html#section-2.2), as defined in BCP 47, and must be a valid CSS <a id="ref-for-typedef-ident"></a>[\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident) or <a id="ref-for-string-value"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value). (Thus language ranges containing asterisks, for example, must be either correctly escaped or quoted as strings, e.g. :lang(&#92;&#42;-Latn) or :lang("\*-Latn").)

Note: The <a id="ref-for-content-language③"></a>[content language](https://drafts.csswg.org/css-text-4/#content-language) of an element is defined by the document language.

For example, in HTML [\[HTML\]](#biblio-html), the <a id="ref-for-content-language④"></a>[content language](https://drafts.csswg.org/css-text-4/#content-language) is determined by a combination of the <code>lang</code> attribute, information from <a id="ref-for-meta"></a>[meta](https://html.spec.whatwg.org/multipage/semantics.html#meta) elements, and possibly also the protocol (e.g. from HTTP headers). XML languages can use the <code>xml&#58;lang</code> attribute to indicate language information for an element. [\[XML10\]](#biblio-xml10)

The element’s <a id="ref-for-content-language⑤"></a>[content language](https://drafts.csswg.org/css-text-4/#content-language) matches a <a id="ref-for-language-range②"></a>[language range](#language-range) if its <a id="ref-for-content-language⑥"></a>content language, as represented in BCP 47 syntax, matches the given <a id="ref-for-language-range③"></a>language range in an <em>extended filtering</em> operation per [\[RFC4647\]](#biblio-rfc4647) Matching of Language Tags (section 3.3.2). The <a id="ref-for-content-language⑦"></a>content language and the <a id="ref-for-language-range④"></a>language range must be <em>canonicalized</em> and converted to <em>extlang form</em> as per section 4.5 of [\[RFC5646\]](#biblio-rfc5646) prior to the <em>extended filtering</em> operation; language tags or ranges that are not [valid](https://www.w3.org/TR/i18n-glossary/#dfn-valid) do not match anything. The matching is performed <a id="ref-for-ascii-case-insensitive⑨"></a>[ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive).

The <a id="ref-for-language-range⑤"></a>[language range](#language-range) must be an extended language range according to BCP47. Language ranges that are not [well-formed](https://www.w3.org/TR/i18n-glossary/#dfn-well-formed) language tags or which would not be a well-formed language tag if an initial wildcard character "\*" were replaced with a valid subtag, do not match anything.

<a id="example-7409fe3d"></a>

<strong>Example:</strong>

[](#example-7409fe3d) For example,

``` text
:lang(åå)
```

would not match, because it containins non-ASCII characters so is ill-formed.

On the other hand,

``` text
:lang(qq)
```

could match, even though qq is not a registered language code.

For this purpose, a wildcard <a id="ref-for-language-range⑥"></a>[language range](#language-range) (<code>&#34;&#42;&#34;</code>) does not match elements whose language is not tagged (e.g. <code>lang=&#34;&#34;</code>), but does match elements whose language is tagged as undetermined (<code>lang=und</code>). A <a id="ref-for-language-range⑦"></a>language range consisting of an empty string (:lang("")) matches (only) elements whose language is not tagged.

Note: It is recommended that documents and protocols indicate language using language tags from [\[BCP47\]](#biblio-bcp47), and in the case of XML-based formats, by means of <code>xml&#58;lang</code> attributes. [\[XML10\]](#biblio-xml10) See [“Choosing a Language Tag”](https://www.w3.org/International/questions/qa-choosing-language-tags)

<a id="example-91142d84"></a>

<strong>Example:</strong>

[](#example-91142d84) Examples: The two following selectors represent an HTML document that is in Belgian French or German. The two next selectors represent <a id="ref-for-the-q-element"></a>[q](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-q-element) quotations in an arbitrary element in Belgian French or German.

``` text
html:lang(fr-be)
html:lang(de)
:lang(fr-be) > q
:lang(de) > q
```

Note: One difference between :lang(C) and the \|= operator is that the \|= operator only performs a comparison against a given attribute on the element, while the :lang(C) pseudo-class uses the UAs knowledge of the document’s semantics to perform the comparison.

<a id="example-c75c4e33"></a>

<strong>Example:</strong>

[](#example-c75c4e33) In this HTML example, only the BODY matches \[lang\|=fr\] (because it has a LANG attribute) but both the BODY and the P match :lang(fr) (because both are in French). The P does not match the \[lang\|=fr\] because it does not have a LANG attribute.

``` text
<body lang=fr>
  <p>Je suis français.</p>
</body>
```

<a id="example-c94037c6"></a>

<strong>Example:</strong>

[](#example-c94037c6) Another difference between :lang(C) and the \|= operator is that :lang(C) performs implicit wildcard matching.

For example, :lang(de-DE) will match all of de-DE, de-DE-1996, de-Latn-DE, de-Latf-DE, de-Latn-DE-1996, whereas of those \[lang\|=de-DE\] will only match de-DE and de-DE-1996.

To perform wildcard matching on the first subtag (the primary language), an asterisk must be used: \*-CH will match all of de-CH, it-CH, fr-CH, and rm-CH.

To select against an element’s lang attribute value using this type of language range match, use both the attribute selector and language pseudo-class together, e.g. \[lang\]:lang(de-DE).

Note: Wildcard language matching and comma-separated lists are new in Level 4.

## <a id="location"></a>8.  Location Pseudo-classes[](#location)

### <a id="the-any-link-pseudo"></a>8.1.  The Hyperlink Pseudo-class: <a id="ref-for-any-link-pseudo"></a>[:any-link](#any-link-pseudo)[](#the-any-link-pseudo)

Tests

- [any-link-attribute-removal.html](https://wpt.fyi/results/css/selectors/invalidation/any-link-attribute-removal.html) [(live test)](http://wpt.live/css/selectors/invalidation/any-link-attribute-removal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/any-link-attribute-removal.html)
- [any-link-pseudo.html](https://wpt.fyi/results/css/selectors/invalidation/any-link-pseudo.html) [(live test)](http://wpt.live/css/selectors/invalidation/any-link-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/any-link-pseudo.html)

The <a id="any-link-pseudo"></a><strong>:any-link</strong> pseudo-class represents an element that acts as the source anchor of a hyperlink. For example, in [\[HTML\]](#biblio-html), any <code><a id="ref-for-the-a-element⑨"></a>[a](https://html&#46;spec&#46;whatwg&#46;org/multipage/text-level-semantics&#46;html#the-a-element)</code> or <code><a id="ref-for-the-area-element"></a>[area](https://html&#46;spec&#46;whatwg&#46;org/multipage/image-maps&#46;html#the-area-element)</code> elements with an <code><a id="ref-for-attr-hyperlink-href"></a>[href](https://html&#46;spec&#46;whatwg&#46;org/multipage/links&#46;html#attr-hyperlink-href)</code> attribute are hyperlinks, and thus match <code>&#58;any-link</code>. It matches an element if the element would match either <a id="ref-for-link-pseudo"></a>[:link](#link-pseudo) or <a id="ref-for-visited-pseudo"></a>[:visited](#visited-pseudo), and is equivalent to :is(:link, :visited).

### <a id="link"></a>8.2.  The Link History Pseudo-classes: <a id="ref-for-link-pseudo①"></a>[:link](#link-pseudo) and <a id="ref-for-visited-pseudo①"></a>[:visited](#visited-pseudo)[](#link)

Tests

- [caret-color-visited-inheritance.html](https://wpt.fyi/results/css/selectors/caret-color-visited-inheritance.html) [(live test)](http://wpt.live/css/selectors/caret-color-visited-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/caret-color-visited-inheritance.html)
- [text-emphasis-visited-inheritance.html](https://wpt.fyi/results/css/selectors/text-emphasis-visited-inheritance.html) [(live test)](http://wpt.live/css/selectors/text-emphasis-visited-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/text-emphasis-visited-inheritance.html)
- [text-fill-color-visited-inheritance.html](https://wpt.fyi/results/css/selectors/text-fill-color-visited-inheritance.html) [(live test)](http://wpt.live/css/selectors/text-fill-color-visited-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/text-fill-color-visited-inheritance.html)
- [text-stroke-color-visited-inheritance.html](https://wpt.fyi/results/css/selectors/text-stroke-color-visited-inheritance.html) [(live test)](http://wpt.live/css/selectors/text-stroke-color-visited-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/text-stroke-color-visited-inheritance.html)
- [visited-in-visited-compound.html](https://wpt.fyi/results/css/selectors/visited-in-visited-compound.html) [(live test)](http://wpt.live/css/selectors/visited-in-visited-compound.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/visited-in-visited-compound.html)
- [visited-inheritance.html](https://wpt.fyi/results/css/selectors/visited-inheritance.html) [(live test)](http://wpt.live/css/selectors/visited-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/visited-inheritance.html)
- [visited-nested.html](https://wpt.fyi/results/css/selectors/visited-nested.html) [(live test)](http://wpt.live/css/selectors/visited-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/visited-nested.html)
- [visited-part-crash.html](https://wpt.fyi/results/css/selectors/visited-part-crash.html) [(live test)](http://wpt.live/css/selectors/visited-part-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/visited-part-crash.html)

User agents commonly display unvisited [hyperlinks](#the-any-link-pseudo) differently from previously visited ones. Selectors provides the pseudo-classes <a id="link-pseudo"></a><strong>:link</strong> and <a id="visited-pseudo"></a><strong>:visited</strong> to distinguish them:

- The <a id="ref-for-link-pseudo②"></a>[:link](#link-pseudo) pseudo-class applies to links that have not yet been visited.
- The <a id="ref-for-visited-pseudo②"></a>[:visited](#visited-pseudo) pseudo-class applies once the link has been visited by the user.

The two states are mutually exclusive.

After some amount of time, user agents may choose to return a visited link to the (unvisited) <a id="ref-for-link-pseudo③"></a>[:link](#link-pseudo) state.

The <a id="ref-for-visited-pseudo③"></a>[:visited](#visited-pseudo) pseudo-class comes with obvious privacy implications—​letting random websites know what <em>other</em> websites you’ve visited can be problematic for a number of reasons—​and so user agents <em>must</em> preserve user privacy in their implementation of <a id="ref-for-visited-pseudo④"></a>:visited.

This specification intentionally does not specify exactly how to preserve user privacy in this regard, to allow for user agents to innovate in this space. The following methods are suggested, however:

<strong>Note:</strong>

- Have <a id="ref-for-visited-pseudo⑤"></a>[:visited](#visited-pseudo) never match, so all links match <a id="ref-for-link-pseudo④"></a>[:link](#link-pseudo) instead.

- Carefully track what history entries could have been observed by a given origin on their own, and only have links match <a id="ref-for-visited-pseudo⑥"></a>[:visited](#visited-pseudo) if that visit would have been observable from the site’s origin. A possible specific approach for this is described in [Appendix C: Example Privacy-Preserving :visited Restrictions](#visited-privacy).

- Allow links to match <a id="ref-for-visited-pseudo⑦"></a>[:visited](#visited-pseudo) on any origin, but carefully restrict what styles they can apply and what information is returned by style-querying APIs like <code><a id="ref-for-dom-window-getcomputedstyle"></a>[getComputedStyle()](https://drafts&#46;csswg&#46;org/cssom-1/#dom-window-getcomputedstyle)</code>, to prevent sites from observing whether a link is styled with <a id="ref-for-link-pseudo⑤"></a>[:link](#link-pseudo) or <a id="ref-for-visited-pseudo⑧"></a>:visited. (This is documented at [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Privacy_and_the_:visited_selector), and was the historical approach browsers took, but is not perfect; there are several ways for a hostile page to still extract history information.)

<a id="example-b0f10727"></a>

<strong>Example:</strong>

[](#example-b0f10727) For example, the selector .footnote:visited would allow styling footnote links differently if they’ve been previously followed, allowing users of the page to know they might not need to click the footnote again.

### <a id="the-target-pseudo"></a>8.3.  The Target Pseudo-class: <a id="ref-for-target-pseudo"></a>[:target](#target-pseudo)[](#the-target-pseudo)

In some document languages, the document’s URL can further point to specific elements <em>within</em> the document via the URL’s <a id="ref-for-concept-url-fragment"></a>[fragment](https://url.spec.whatwg.org/#concept-url-fragment). The elements pointed to in this way are the target elements of the document.

<a id="example-b8c5336f"></a>

<strong>Example:</strong>

[](#example-b8c5336f) In HTML the fragment points to the element in the page with the same ID. The url <code>https&#58;//example&#46;com/index&#46;html&#35;section2</code>, for example, points to the element with <code>id=&#34;section2&#34;</code> in the document at <code>https&#58;//example&#46;com/index&#46;html</code>.

The <a id="target-pseudo"></a><strong>:target</strong> pseudo-class matches the document’s target elements. If the document’s URL has no fragment identifier, then the document has no target elements.

<a id="example-8a386e7c"></a> [](#example-8a386e7c) Example:

``` text
p.note:target 
```

This selector represents a <code><a id="ref-for-the-p-element①"></a>[p](https://html&#46;spec&#46;whatwg&#46;org/multipage/grouping-content&#46;html#the-p-element)</code> element of class <code>note</code> that is the target element of the referring URL.

<a id="example-af403697"></a>

<strong>Example:</strong>

[](#example-af403697) CSS example: Here, the <a id="ref-for-target-pseudo①"></a>[:target](#target-pseudo) pseudo-class is used to make the target element red and place an image before it, if there is one:

``` text
:target { color : red }
:target::before { content : url(target.png) }
```

Note: This specification previously defined a :target-within pseudo-class, analogous to <a id="ref-for-focus-within-pseudo"></a>[:focus-within](#focus-within-pseudo). It was removed in favor of :has(:target), which should hopefully suffice to solve the same use-cases.

### <a id="the-scope-pseudo"></a>8.4.  The Reference Element Pseudo-class: <a id="ref-for-scope-pseudo"></a>[:scope](#scope-pseudo)[](#the-scope-pseudo)

Tests

- [scope-selector.html](https://wpt.fyi/results/css/selectors/scope-selector.html) [(live test)](http://wpt.live/css/selectors/scope-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/scope-selector.html)
- [scope-without-scoping.html](https://wpt.fyi/results/css/selectors/scope-without-scoping.html) [(live test)](http://wpt.live/css/selectors/scope-without-scoping.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/scope-without-scoping.html)

In some contexts, selectors are matched with respect to one or more <a id="ref-for-scoping-root②"></a>[scoping roots](#scoping-root), such as when calling the <code>querySelector()</code> method in [\[DOM\]](#biblio-dom). The <a id="scope-pseudo"></a><strong>:scope</strong> pseudo-class represents this <a id="ref-for-scoping-root③"></a>scoping root, and may be either a true element or a virtual one (such as a <code><a id="ref-for-documentfragment"></a>[DocumentFragment](https://dom&#46;spec&#46;whatwg&#46;org/#documentfragment)</code>).

If there is no <a id="ref-for-scoping-root④"></a>[scoping root](#scoping-root) then <a id="ref-for-scope-pseudo①"></a>[:scope](#scope-pseudo) represents the root of the tree the element is in (equivalent to <a id="ref-for-selectordef-host①"></a>[:host](https://drafts.csswg.org/css-shadow-1/#selectordef-host) in a <a id="ref-for-concept-shadow-tree①"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), or <a id="ref-for-root-pseudo"></a>[:root](#root-pseudo) otherwise). Specifications intending for this pseudo-class to match specific elements rather than this tree root element must define their <a id="ref-for-scoping-root⑤"></a>scoping root(s).

A virtual <a id="ref-for-scoping-root⑥"></a>[scoping root](#scoping-root) is some object representing the root of a document fragment, and can be used in selector patterns to represent other elements’ relationships to this <a id="ref-for-scoping-root⑦"></a>scoping root, acting as the parent of any root elements in the document fragment it represents. A virtual <a id="ref-for-scoping-root⑧"></a>scoping root is <a id="ref-for-featureless⑨"></a>[featureless](#featureless) and cannot be the <a id="ref-for-selector-subject③"></a>[subject of the selector](#selector-subject).

<a id="example-ef427647"></a>

<strong>Example:</strong>

[](#example-ef427647) For example, if you have a <code><a id="ref-for-documentfragment①"></a>[DocumentFragment](https://dom&#46;spec&#46;whatwg&#46;org/#documentfragment)</code> <code>df</code>, then <code>df&#46;querySelectorAll(&#34;&#58;scope &#62; &#46;foo&#34;)</code> matches all the .foo elements that are "top-level" in the document fragment (those that have the document fragment as their <code><a id="ref-for-dom-node-parentnode"></a>[parentNode](https://dom&#46;spec&#46;whatwg&#46;org/#dom-node-parentnode)</code>).

However, <code>df&#46;querySelector(&#34;&#58;scope&#34;)</code> will not match anything, as the document fragment itself can’t be the <a id="ref-for-selector-subject④"></a>[subject of the selector](#selector-subject).

## <a id="useraction-pseudos"></a>9.  User Action Pseudo-classes[](#useraction-pseudos)

Interactive user interfaces sometimes change the rendering in response to user actions. Selectors provides several <a id="user-action-pseudo-class"></a><strong>user action pseudo-classes</strong> for the selection of an element the user is acting on. (In non-interactive user agents, these pseudo-classes are valid, but never match any element.)

The <a id="ref-for-user-action-pseudo-class②"></a>[user action pseudo-classes](#user-action-pseudo-class) are not mutually exclusive. An element can match several such pseudo-classes at the same time.

<a id="example-b4125c5c"></a>

<strong>Example:</strong>

[](#example-b4125c5c) Examples:

``` text
a:hover   /* user hovers over the link */
a:focus   /* user focuses the link     */

a:focus:hover
/* user hovers over the link while it's focused */
```

Some <a id="ref-for-user-action-pseudo-class③"></a>[user action pseudo-classes](#user-action-pseudo-class), in addition to matching on the particular element with the property in question, match on that element’s ancestors as well: <a id="ref-for-hover-pseudo①"></a>[:hover](#hover-pseudo), <a id="ref-for-active-pseudo"></a>[:active](#active-pseudo), <a id="ref-for-focus-within-pseudo①"></a>[:focus-within](#focus-within-pseudo). Specifically, if these match on a given element, they also match on the element’s <a id="ref-for-flat-tree"></a>[flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) ancestors, up to and including the first <a id="ref-for-document-top-layer"></a>[top layer](https://drafts.csswg.org/css-position-4/#document-top-layer) element or the <a id="ref-for-concept-tree-root"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) element, whichever is encountered first.

Note: The specifics of hit-testing, necessary to know when several of the pseudo-classes defined in this section apply, are not yet defined, but will be in the future.

Tests

- [active-display-none-001.html](https://wpt.fyi/results/css/selectors/active-display-none-001.html) [(live test)](http://wpt.live/css/selectors/active-display-none-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/active-display-none-001.html)
- [active-observable.html](https://wpt.fyi/results/css/selectors/active-observable.html) [(live test)](http://wpt.live/css/selectors/active-observable.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/active-observable.html)
- [active-toplayer-001.html](https://wpt.fyi/results/css/selectors/active-toplayer-001.html) [(live test)](http://wpt.live/css/selectors/active-toplayer-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/active-toplayer-001.html)
- [active-after-relayouts.html](https://wpt.fyi/results/css/selectors/active-after-relayouts.html) [(live test)](http://wpt.live/css/selectors/active-after-relayouts.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/active-after-relayouts.html)
- [focus-within-toplayer-001.html](https://wpt.fyi/results/css/selectors/focus-within-toplayer-001.html) [(live test)](http://wpt.live/css/selectors/focus-within-toplayer-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-toplayer-001.html)
- [hover-toplayer-001.html](https://wpt.fyi/results/css/selectors/hover-toplayer-001.html) [(live test)](http://wpt.live/css/selectors/hover-toplayer-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/hover-toplayer-001.html)
- [toplayer-transition-001.html](https://wpt.fyi/results/css/selectors/toplayer-transition-001.html) [(live test)](http://wpt.live/css/selectors/toplayer-transition-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/toplayer-transition-001.html)
- [user-action-pseudo-top-layer.html](https://wpt.fyi/results/css/selectors/user-action-pseudo-top-layer.html) [(live test)](http://wpt.live/css/selectors/user-action-pseudo-top-layer.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/user-action-pseudo-top-layer.html)

### <a id="the-hover-pseudo"></a>9.1.  The Pointer Hover Pseudo-class: <a id="ref-for-hover-pseudo②"></a>[:hover](#hover-pseudo)[](#the-hover-pseudo)

Tests

- hover-001-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/hover-001-manual.html)
- [hover-002.html](https://wpt.fyi/results/css/selectors/hover-002.html) [(live test)](http://wpt.live/css/selectors/hover-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/hover-002.html)
- [remove-hovered-element.html](https://wpt.fyi/results/css/selectors/remove-hovered-element.html) [(live test)](http://wpt.live/css/selectors/remove-hovered-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/remove-hovered-element.html)

The <a id="hover-pseudo"></a><strong>:hover</strong> pseudo-class applies while the user designates an element (or pseudo-element) with a pointing device, but does not necessarily activate it. For example, a visual user agent could apply this pseudo-class when the cursor (mouse pointer) hovers over a box generated by the element. Interactive user agents that cannot detect hovering due to hardware limitations (e.g., a pen device that does not detect hovering) are still conforming; the selector will simply never match in such a UA.

An element also matches <a id="ref-for-hover-pseudo③"></a>[:hover](#hover-pseudo) if one of its descendants in the <a id="ref-for-flat-tree①"></a>[flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) (including non-element nodes, such as text nodes) matches the above conditions.

Document languages may define additional ways in which an element can match <a id="ref-for-hover-pseudo④"></a>[:hover](#hover-pseudo). For example, [\[HTML\]](#biblio-html) defines a labeled control element as [matching <code>&#58;hover</code>](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-hover) when its <a id="ref-for-the-label-element"></a>[label](https://html.spec.whatwg.org/multipage/forms.html#the-label-element) is hovered.

Note: Since the <a id="ref-for-hover-pseudo⑤"></a>[:hover](#hover-pseudo) state can apply to an element because its child is designated by a pointing device, it is possible for <a id="ref-for-hover-pseudo⑥"></a>:hover to apply to an element that is not underneath the pointing device.

### <a id="the-active-pseudo"></a>9.2.  The Activation Pseudo-class: <a id="ref-for-active-pseudo①"></a>[:active](#active-pseudo)[](#the-active-pseudo)

The <a id="active-pseudo"></a><strong>:active</strong> pseudo-class applies while an element is being “activated” by the user, as defined by the host language; for example, while a hyperlink is being triggered.

In addition, the <a id="ref-for-active-pseudo②"></a>[:active](#active-pseudo) pseudo-class applies while any generated box of <em>any</em> element (or pseudo-element) is being actively indicated by a pointing device (in the “down” state), e.g. between the time the user presses the primary mouse button and releases it, or while a finger is pressing on a touchscreen.

Note: [\[HTML\]](#biblio-html) defines [specific conditions for HTML elements to be activated](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-active) .

An element also matches <a id="ref-for-active-pseudo③"></a>[:active](#active-pseudo) if one of its descendants in the <a id="ref-for-flat-tree②"></a>[flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) (including non-element nodes, such as text nodes) matches the above conditions.

### <a id="the-focus-pseudo"></a>9.3.  The Input Focus Pseudo-class: <a id="ref-for-focus-pseudo②"></a>[:focus](#focus-pseudo)[](#the-focus-pseudo)

Tests

- [focus-display-none-001.html](https://wpt.fyi/results/css/selectors/focus-display-none-001.html) [(live test)](http://wpt.live/css/selectors/focus-display-none-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-display-none-001.html)
- [focus-in-focus-event-001.html](https://wpt.fyi/results/css/selectors/focus-in-focus-event-001.html) [(live test)](http://wpt.live/css/selectors/focus-in-focus-event-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-in-focus-event-001.html)
- [focus-in-focusin-event-001.html](https://wpt.fyi/results/css/selectors/focus-in-focusin-event-001.html) [(live test)](http://wpt.live/css/selectors/focus-in-focusin-event-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-in-focusin-event-001.html)

The <a id="focus-pseudo"></a><strong>:focus</strong> pseudo-class applies while an element (or pseudo-element) has the focus (accepts keyboard or other forms of input).

There may be document language or implementation specific limits on which elements can acquire <a id="ref-for-focus-pseudo③"></a>[:focus](#focus-pseudo). For example, [\[HTML\]](#biblio-html) defines a list of [focusable areas](https://html.spec.whatwg.org/multipage/interaction.html#focusable-area).

Document languages may define additional ways in which an element can match <a id="ref-for-focus-pseudo④"></a>[:focus](#focus-pseudo), except that the <a id="ref-for-focus-pseudo⑤"></a>:focus pseudo-class must not automatically propagate to the parent element—​see <a id="ref-for-focus-within-pseudo②"></a>[:focus-within](#focus-within-pseudo) if matching on the parent is desired. (<a id="ref-for-focus-pseudo⑥"></a>:focus may still apply to the parent element if made to propagate due to other mechanisms, but not merely due to being the parent.)

<a id="issue-1416193e"></a>

<strong>Issue:</strong>

[](#issue-1416193e) There’s a desire from authors to propagate <a id="ref-for-focus-pseudo⑦"></a>[:focus](#focus-pseudo) from a form control to its associated <code><a id="ref-for-the-label-element①"></a>[label](https://html&#46;spec&#46;whatwg&#46;org/multipage/forms&#46;html#the-label-element)</code> element; the main objection seems to be implementation difficulty. See [CSSWG issue (CSS)](https://github.com/w3c/csswg-drafts/issues/397) and [WHATWG issue (HTML)](https://github.com/whatwg/html/issues/1632).

### <a id="the-focus-visible-pseudo"></a>9.4. <a id="focus-ring-pseudo"></a><a id="the-focusring-pseudo"></a> The Focus-Indicated Pseudo-class: <a id="ref-for-focus-visible-pseudo"></a>[:focus-visible](#focus-visible-pseudo)[](#the-focus-visible-pseudo)

Tests

- [focus-visible-001.html](https://wpt.fyi/results/css/selectors/focus-visible-001.html) [(live test)](http://wpt.live/css/selectors/focus-visible-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-001.html)
- [focus-visible-002.html](https://wpt.fyi/results/css/selectors/focus-visible-002.html) [(live test)](http://wpt.live/css/selectors/focus-visible-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-002.html)
- [focus-visible-003.html](https://wpt.fyi/results/css/selectors/focus-visible-003.html) [(live test)](http://wpt.live/css/selectors/focus-visible-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-003.html)
- [focus-visible-004.html](https://wpt.fyi/results/css/selectors/focus-visible-004.html) [(live test)](http://wpt.live/css/selectors/focus-visible-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-004.html)
- [focus-visible-005.html](https://wpt.fyi/results/css/selectors/focus-visible-005.html) [(live test)](http://wpt.live/css/selectors/focus-visible-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-005.html)
- [focus-visible-006.html](https://wpt.fyi/results/css/selectors/focus-visible-006.html) [(live test)](http://wpt.live/css/selectors/focus-visible-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-006.html)
- [focus-visible-007.html](https://wpt.fyi/results/css/selectors/focus-visible-007.html) [(live test)](http://wpt.live/css/selectors/focus-visible-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-007.html)
- [focus-visible-008.html](https://wpt.fyi/results/css/selectors/focus-visible-008.html) [(live test)](http://wpt.live/css/selectors/focus-visible-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-008.html)
- [focus-visible-009.html](https://wpt.fyi/results/css/selectors/focus-visible-009.html) [(live test)](http://wpt.live/css/selectors/focus-visible-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-009.html)
- [focus-visible-010.html](https://wpt.fyi/results/css/selectors/focus-visible-010.html) [(live test)](http://wpt.live/css/selectors/focus-visible-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-010.html)
- [focus-visible-011.html](https://wpt.fyi/results/css/selectors/focus-visible-011.html) [(live test)](http://wpt.live/css/selectors/focus-visible-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-011.html)
- [focus-visible-012.html](https://wpt.fyi/results/css/selectors/focus-visible-012.html) [(live test)](http://wpt.live/css/selectors/focus-visible-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-012.html)
- [focus-visible-013.html](https://wpt.fyi/results/css/selectors/focus-visible-013.html) [(live test)](http://wpt.live/css/selectors/focus-visible-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-013.html)
- [focus-visible-014.html](https://wpt.fyi/results/css/selectors/focus-visible-014.html) [(live test)](http://wpt.live/css/selectors/focus-visible-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-014.html)
- [focus-visible-015.html](https://wpt.fyi/results/css/selectors/focus-visible-015.html) [(live test)](http://wpt.live/css/selectors/focus-visible-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-015.html)
- [focus-visible-016.html](https://wpt.fyi/results/css/selectors/focus-visible-016.html) [(live test)](http://wpt.live/css/selectors/focus-visible-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-016.html)
- [focus-visible-017-2.html](https://wpt.fyi/results/css/selectors/focus-visible-017-2.html) [(live test)](http://wpt.live/css/selectors/focus-visible-017-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-017-2.html)
- [focus-visible-017.html](https://wpt.fyi/results/css/selectors/focus-visible-017.html) [(live test)](http://wpt.live/css/selectors/focus-visible-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-017.html)
- [focus-visible-018-2.html](https://wpt.fyi/results/css/selectors/focus-visible-018-2.html) [(live test)](http://wpt.live/css/selectors/focus-visible-018-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-018-2.html)
- [focus-visible-018.html](https://wpt.fyi/results/css/selectors/focus-visible-018.html) [(live test)](http://wpt.live/css/selectors/focus-visible-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-018.html)
- [focus-visible-019.html](https://wpt.fyi/results/css/selectors/focus-visible-019.html) [(live test)](http://wpt.live/css/selectors/focus-visible-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-019.html)
- [focus-visible-020.html](https://wpt.fyi/results/css/selectors/focus-visible-020.html) [(live test)](http://wpt.live/css/selectors/focus-visible-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-020.html)
- [focus-visible-021.html](https://wpt.fyi/results/css/selectors/focus-visible-021.html) [(live test)](http://wpt.live/css/selectors/focus-visible-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-021.html)
- [focus-visible-023.html](https://wpt.fyi/results/css/selectors/focus-visible-023.html) [(live test)](http://wpt.live/css/selectors/focus-visible-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-023.html)
- [focus-visible-024.html](https://wpt.fyi/results/css/selectors/focus-visible-024.html) [(live test)](http://wpt.live/css/selectors/focus-visible-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-024.html)
- [focus-visible-025.html](https://wpt.fyi/results/css/selectors/focus-visible-025.html) [(live test)](http://wpt.live/css/selectors/focus-visible-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-025.html)
- [focus-visible-026.html](https://wpt.fyi/results/css/selectors/focus-visible-026.html) [(live test)](http://wpt.live/css/selectors/focus-visible-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-026.html)
- [focus-visible-027.html](https://wpt.fyi/results/css/selectors/focus-visible-027.html) [(live test)](http://wpt.live/css/selectors/focus-visible-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-027.html)
- [focus-visible-028.html](https://wpt.fyi/results/css/selectors/focus-visible-028.html) [(live test)](http://wpt.live/css/selectors/focus-visible-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-028.html)
- [focus-visible-script-focus-001.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-001.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-001.html)
- [focus-visible-script-focus-004.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-004.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-004.html)
- [focus-visible-script-focus-005.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-005.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-005.html)
- [focus-visible-script-focus-008-b.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-008-b.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-008-b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-008-b.html)
- [focus-visible-script-focus-008.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-008.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-008.html)
- [focus-visible-script-focus-009.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-009.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-009.html)
- [focus-visible-script-focus-010.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-010.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-010.html)
- [focus-visible-script-focus-011.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-011.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-011.html)
- [focus-visible-script-focus-012.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-012.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-012.html)
- [focus-visible-script-focus-013.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-013.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-013.html)
- [focus-visible-script-focus-014.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-014.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-014.html)
- [focus-visible-script-focus-015.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-015.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-015.html)
- [focus-visible-script-focus-018.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-018.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-018.html)
- [focus-visible-script-focus-019.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-019.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-019.html)
- [focus-visible-script-focus-020.html](https://wpt.fyi/results/css/selectors/focus-visible-script-focus-020.html) [(live test)](http://wpt.live/css/selectors/focus-visible-script-focus-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-visible-script-focus-020.html)
- [parse-focus-visible.html](https://wpt.fyi/results/css/selectors/parsing/parse-focus-visible.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-focus-visible.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-focus-visible.html)

While the <a id="ref-for-focus-pseudo⑧"></a>[:focus](#focus-pseudo) <a id="ref-for-pseudo-class①⑧"></a>[pseudo-class](#pseudo-class) always matches the currently-focused element, UAs only sometimes visibly <a id="indicate-focus"></a><strong>indicate focus</strong> (such as by drawing a “focus ring”), instead using a variety of heuristics to visibly indicate the focus only when it would be most helpful to the user. The <a id="focus-visible-pseudo"></a><strong>:focus-visible</strong> <a id="ref-for-pseudo-class①⑨"></a>pseudo-class matches a focused element (or pseudo-element) in these situations only, allowing authors to change the appearance of the focus indicator without changing <em>when</em> a focus indicator appears.

<a id="example-c40d8011"></a>

<strong>Example:</strong>

[](#example-c40d8011) In this example, all focusable elements get a strong yellow outline on <a id="ref-for-focus-visible-pseudo①"></a>[:focus-visible](#focus-visible-pseudo), and links get both a yellow outline and a yellow background on <a id="ref-for-focus-visible-pseudo②"></a>:focus-visible. These styles are consistent throughout the page and are easily visible due to their bold styling, but do not appear unless the user is likely to need to understand where page focus is.

``` text
:root {
  --focus-gold: #ffbf47;
}

:focus-visible  {
  outline: 3px solid var(--focus-gold);
}

a:focus-visible {
  background-color: var(--focus-gold);
}
```

<a id="example-ea93cf5c"></a>

<strong>Example:</strong>

[](#example-ea93cf5c) User agents can choose their own heuristics for when to <a id="ref-for-indicate-focus"></a>[indicate focus](#indicate-focus); however, the following (non-normative) suggestions can be used as a starting point for when to <a id="ref-for-indicate-focus①"></a>indicate focus on the currently focused element:

- If the user has expressed a preference (such as via a system preference or a browser setting) to always see a visible focus indicator, <a id="ref-for-indicate-focus②"></a>[indicate focus](#indicate-focus) regardless of any other factors. (Another option may be for the user agent to show its own focus indicator regardless of author styles.)

- If the element supports keyboard input (such as an <code><a id="ref-for-the-input-element"></a>[input](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#the-input-element)</code> element, or any other element that would triggers a virtual keyboard to be shown on focus if a physical keyboard were not present), <a id="ref-for-indicate-focus③"></a>[indicate focus](#indicate-focus).

- If the user interacts with the page via keyboard or some other non-pointing device, <a id="ref-for-indicate-focus④"></a>[indicate focus](#indicate-focus). (This means keyboard usage may change whether this pseudo-class matches even if it doesn’t affect <a id="ref-for-focus-pseudo⑨"></a>[:focus](#focus-pseudo)).

- If the user interacts with the page via a pointing device (mouse, touchscreen, etc.) and the focused element does not support keyboard input, don’t <a id="ref-for-indicate-focus⑤"></a>[indicate focus](#indicate-focus).

- If the previously-focused element <a id="ref-for-indicate-focus⑥"></a>[indicated focus](#indicate-focus), and a script causes focus to move elsewhere, <a id="ref-for-indicate-focus⑦"></a>indicate focus on the newly focused element.

  Conversely, if the previously-focused element did not <a id="ref-for-indicate-focus⑧"></a>[indicate focus](#indicate-focus), and a script causes focus to move elsewhere, don’t <a id="ref-for-indicate-focus⑨"></a>indicate focus on the newly focused element.

- If a newly-displayed element automatically gains focus (such as an action button in a freshly opened dialog), that element should <a id="ref-for-indicate-focus①⓪"></a>[indicate focus](#indicate-focus).

User agents should also use <a id="ref-for-focus-visible-pseudo③"></a>[:focus-visible](#focus-visible-pseudo) to specify the default focus style, so that authors using <a id="ref-for-focus-visible-pseudo④"></a>:focus-visible will not also need to disable the default <a id="ref-for-focus-pseudo①⓪"></a>[:focus](#focus-pseudo) style.

### <a id="the-focus-within-pseudo"></a>9.5.  The Focus Container Pseudo-class: <a id="ref-for-focus-within-pseudo③"></a>[:focus-within](#focus-within-pseudo)[](#the-focus-within-pseudo)

Tests

- [focus-within-001.html](https://wpt.fyi/results/css/selectors/focus-within-001.html) [(live test)](http://wpt.live/css/selectors/focus-within-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-001.html)
- [focus-within-002.html](https://wpt.fyi/results/css/selectors/focus-within-002.html) [(live test)](http://wpt.live/css/selectors/focus-within-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-002.html)
- [focus-within-003.html](https://wpt.fyi/results/css/selectors/focus-within-003.html) [(live test)](http://wpt.live/css/selectors/focus-within-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-003.html)
- [focus-within-004.html](https://wpt.fyi/results/css/selectors/focus-within-004.html) [(live test)](http://wpt.live/css/selectors/focus-within-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-004.html)
- [focus-within-005.html](https://wpt.fyi/results/css/selectors/focus-within-005.html) [(live test)](http://wpt.live/css/selectors/focus-within-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-005.html)
- [focus-within-006.html](https://wpt.fyi/results/css/selectors/focus-within-006.html) [(live test)](http://wpt.live/css/selectors/focus-within-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-006.html)
- [focus-within-007.html](https://wpt.fyi/results/css/selectors/focus-within-007.html) [(live test)](http://wpt.live/css/selectors/focus-within-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-007.html)
- [focus-within-008.html](https://wpt.fyi/results/css/selectors/focus-within-008.html) [(live test)](http://wpt.live/css/selectors/focus-within-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-008.html)
- [focus-within-009.html](https://wpt.fyi/results/css/selectors/focus-within-009.html) [(live test)](http://wpt.live/css/selectors/focus-within-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-009.html)
- [focus-within-010.html](https://wpt.fyi/results/css/selectors/focus-within-010.html) [(live test)](http://wpt.live/css/selectors/focus-within-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-010.html)
- [focus-within-011.html](https://wpt.fyi/results/css/selectors/focus-within-011.html) [(live test)](http://wpt.live/css/selectors/focus-within-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-011.html)
- [focus-within-012.html](https://wpt.fyi/results/css/selectors/focus-within-012.html) [(live test)](http://wpt.live/css/selectors/focus-within-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-012.html)
- [focus-within-013.html](https://wpt.fyi/results/css/selectors/focus-within-013.html) [(live test)](http://wpt.live/css/selectors/focus-within-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-013.html)
- [focus-within-display-none-001.html](https://wpt.fyi/results/css/selectors/focus-within-display-none-001.html) [(live test)](http://wpt.live/css/selectors/focus-within-display-none-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-display-none-001.html)
- [focus-within-focus-move.html](https://wpt.fyi/results/css/selectors/focus-within-focus-move.html) [(live test)](http://wpt.live/css/selectors/focus-within-focus-move.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-focus-move.html)
- [focus-within-removal.html](https://wpt.fyi/results/css/selectors/focus-within-removal.html) [(live test)](http://wpt.live/css/selectors/focus-within-removal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-removal.html)
- [focus-within-shadow-001.html](https://wpt.fyi/results/css/selectors/focus-within-shadow-001.html) [(live test)](http://wpt.live/css/selectors/focus-within-shadow-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-shadow-001.html)
- [focus-within-shadow-002.html](https://wpt.fyi/results/css/selectors/focus-within-shadow-002.html) [(live test)](http://wpt.live/css/selectors/focus-within-shadow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-shadow-002.html)
- [focus-within-shadow-003.html](https://wpt.fyi/results/css/selectors/focus-within-shadow-003.html) [(live test)](http://wpt.live/css/selectors/focus-within-shadow-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-shadow-003.html)
- [focus-within-shadow-004.html](https://wpt.fyi/results/css/selectors/focus-within-shadow-004.html) [(live test)](http://wpt.live/css/selectors/focus-within-shadow-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-shadow-004.html)
- [focus-within-shadow-005.html](https://wpt.fyi/results/css/selectors/focus-within-shadow-005.html) [(live test)](http://wpt.live/css/selectors/focus-within-shadow-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-shadow-005.html)
- [focus-within-shadow-006.html](https://wpt.fyi/results/css/selectors/focus-within-shadow-006.html) [(live test)](http://wpt.live/css/selectors/focus-within-shadow-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-within-shadow-006.html)

The <a id="focus-within-pseudo"></a><strong>:focus-within</strong> pseudo-class applies to any element (or pseudo-element) for which the <a id="ref-for-focus-pseudo①①"></a>[:focus](#focus-pseudo) pseudo-class applies, as well as to an element (or pseudo-element) whose descendant in the <a id="ref-for-flat-tree③"></a>[flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) (including non-element nodes, such as text nodes) matches the conditions for matching <a id="ref-for-focus-pseudo①②"></a>:focus.

## <a id="resource-pseudos"></a>10.  Resource State Pseudo-classes[](#resource-pseudos)

The pseudo-classes in this section apply to elements that represent loaded resources, particularly images/videos, and allow authors to select them based on some quality of their state.

### <a id="video-state"></a>10.1.  Media Playback State: the <a id="ref-for-selectordef-playing"></a>[:playing](#selectordef-playing), <a id="ref-for-selectordef-paused"></a>[:paused](#selectordef-paused), and <a id="ref-for-selectordef-seeking"></a>[:seeking](#selectordef-seeking) pseudo-classes[](#video-state)

Tests

- [media-playback-state.html](https://wpt.fyi/results/css/selectors/media/media-playback-state.html) [(live test)](http://wpt.live/css/selectors/media/media-playback-state.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/media-playback-state.html)
- [media-loading-state.sub.html](https://wpt.fyi/results/css/selectors/media/media-loading-state.sub.html) [(live test)](http://wpt.live/css/selectors/media/media-loading-state.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/media-loading-state.sub.html)
- [media-playing-paused-style-invalidation.html](https://wpt.fyi/results/css/selectors/media/media-playing-paused-style-invalidation.html) [(live test)](http://wpt.live/css/selectors/media/media-playing-paused-style-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/media-playing-paused-style-invalidation.html)

The <a id="selectordef-playing"></a><strong>:playing</strong> pseudo-class represents an element that is capable of being “played” or “paused”, when that element is “playing”. (This includes both when the element is explicitly playing, and when it’s temporarily stopped for some reason not connected to user intent, but will automatically resume when that reason is resolved, such as a “buffering” or “stalled” state.)

The <a id="selectordef-paused"></a><strong>:paused</strong> pseudo-class represents an element that is capable of being “played” or “paused”, when that element is “paused” (i.e. <em>not</em> ”playing”). (This includes both an explicit “paused” state, and other non-playing states like “loaded, hasn’t been activated yet”, etc.)

The <a id="selectordef-seeking"></a><strong>:seeking</strong> pseudo-class represents an element that is capable of ”seeking” when that element is ”seeking”. (For the <code><a id="ref-for-audio"></a>[audio](https://html&#46;spec&#46;whatwg&#46;org/multipage/media&#46;html#audio)</code> and <code><a id="ref-for-video"></a>[video](https://html&#46;spec&#46;whatwg&#46;org/multipage/media&#46;html#video)</code> elements of HTML, see [HTML § 4.8.11.9 Seeking](https://html.spec.whatwg.org/multipage/media.html#seeking).)

### <a id="media-loading-state"></a>10.2.  Media Loading State: the <a id="ref-for-selectordef-buffering"></a>[:buffering](#selectordef-buffering) and <a id="ref-for-selectordef-stalled"></a>[:stalled](#selectordef-stalled) pseudo-classes[](#media-loading-state)

Tests

- [media-loading-state-timing.sub.html](https://wpt.fyi/results/css/selectors/media/media-loading-state-timing.sub.html) [(live test)](http://wpt.live/css/selectors/media/media-loading-state-timing.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/media-loading-state-timing.sub.html)
- [media-playback-state-timing.html](https://wpt.fyi/results/css/selectors/media/media-playback-state-timing.html) [(live test)](http://wpt.live/css/selectors/media/media-playback-state-timing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/media-playback-state-timing.html)

The <a id="selectordef-buffering"></a><strong>:buffering</strong> pseudo-class represents an element that is capable of being “played” or “paused”, when that element cannot continue playing because it is actively attempting to obtain <a id="ref-for-media-data"></a>[media data](https://html.spec.whatwg.org/multipage/media.html#media-data) but has not yet obtained enough data to resume playback. (Note that the element is still considered to be “playing” when it is “buffering”. Whenever <a id="ref-for-selectordef-buffering①"></a>[:buffering](#selectordef-buffering) matches an element, <a id="ref-for-selectordef-playing①"></a>[:playing](#selectordef-playing) also matches the element.)

The <a id="selectordef-stalled"></a><strong>:stalled</strong> pseudo-class represents an element when that element cannot continue playing because it is actively attempting to obtain <a id="ref-for-media-data①"></a>[media data](https://html.spec.whatwg.org/multipage/media.html#media-data) but it has failed to receive any data for some amount of time. For the <code><a id="ref-for-audio①"></a>[audio](https://html&#46;spec&#46;whatwg&#46;org/multipage/media&#46;html#audio)</code> and <code><a id="ref-for-video①"></a>[video](https://html&#46;spec&#46;whatwg&#46;org/multipage/media&#46;html#video)</code> elements of HTML, this amount of time is the <a id="ref-for-stall-timeout"></a>[media element stall timeout](https://html.spec.whatwg.org/multipage/media.html#stall-timeout). [\[HTML\]](#biblio-html) (Note that, like with the <a id="ref-for-selectordef-buffering②"></a>[:buffering](#selectordef-buffering) pseudo-class, the element is still considered to be “playing” when it is “stalled”. Whenever <a id="ref-for-selectordef-stalled①"></a>[:stalled](#selectordef-stalled) matches an element, <a id="ref-for-selectordef-playing②"></a>[:playing](#selectordef-playing) also matches the element.)

### <a id="sound-state"></a>10.3.  Sound State: the <a id="ref-for-selectordef-muted"></a>[:muted](#selectordef-muted) and <a id="ref-for-selectordef-volume-locked"></a>[:volume-locked](#selectordef-volume-locked) pseudo-classes[](#sound-state)

Tests

- [sound-state.html](https://wpt.fyi/results/css/selectors/media/sound-state.html) [(live test)](http://wpt.live/css/selectors/media/sound-state.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/sound-state.html)

The <a id="selectordef-muted"></a><strong>:muted</strong> pseudo-class represents an element that is capable of making sound, but is currently “muted“ (forced silent). (For the <code><a id="ref-for-audio②"></a>[audio](https://html&#46;spec&#46;whatwg&#46;org/multipage/media&#46;html#audio)</code> and <code><a id="ref-for-video②"></a>[video](https://html&#46;spec&#46;whatwg&#46;org/multipage/media&#46;html#video)</code> elements of HTML, see <a id="ref-for-concept-media-muted"></a>[muted](https://html.spec.whatwg.org/multipage/media.html#concept-media-muted). [\[HTML\]](#biblio-html))

The <a id="selectordef-volume-locked"></a><strong>:volume-locked</strong> pseudo-class represents an element that is capable of making sound, and currently has its volume "locked" by the UA or the user, so the page author cannot change it. (For the <code><a id="ref-for-audio③"></a>[audio](https://html&#46;spec&#46;whatwg&#46;org/multipage/media&#46;html#audio)</code> and <code><a id="ref-for-video③"></a>[video](https://html&#46;spec&#46;whatwg&#46;org/multipage/media&#46;html#video)</code> elements of HTML, see the algorithm for setting the element’s <a id="ref-for-effective-media-volume"></a>[effective media volume](https://html.spec.whatwg.org/multipage/media.html#effective-media-volume). [\[HTML\]](#biblio-html))

## <a id="display-state-pseudos"></a>11.  Element Display State Pseudo-classes[](#display-state-pseudos)

### <a id="open-state"></a>11.1.  Collapse State: the <a id="ref-for-selectordef-open"></a>[:open](#selectordef-open) pseudo-class[](#open-state)

Tests

- [open-pseudo.html](https://wpt.fyi/results/css/selectors/open-pseudo.html) [(live test)](http://wpt.live/css/selectors/open-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/open-pseudo.html)
- [details-open-pseudo-001.html](https://wpt.fyi/results/css/selectors/selectors-4/details-open-pseudo-001.html) [(live test)](http://wpt.live/css/selectors/selectors-4/details-open-pseudo-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/details-open-pseudo-001.html)
- [details-open-pseudo-002.html](https://wpt.fyi/results/css/selectors/selectors-4/details-open-pseudo-002.html) [(live test)](http://wpt.live/css/selectors/selectors-4/details-open-pseudo-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/details-open-pseudo-002.html)
- [details-open-pseudo-003.html](https://wpt.fyi/results/css/selectors/selectors-4/details-open-pseudo-003.html) [(live test)](http://wpt.live/css/selectors/selectors-4/details-open-pseudo-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/details-open-pseudo-003.html)

<!-- -->

- [input-element-pseudo-open.optional.html](https://wpt.fyi/results/css/css-pseudo/input-element-pseudo-open.optional.html) [(live test)](http://wpt.live/css/css-pseudo/input-element-pseudo-open.optional.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/input-element-pseudo-open.optional.html)

The <a id="selectordef-open"></a><strong>:open</strong> pseudo-class represents an element that has both “open” and “closed” states, and which is currently in the “open” state.

Exactly what “open” and “closed” mean is host-language specific, but exemplified by elements such as HTML’s <code><a id="ref-for-the-details-element"></a>[details](https://html&#46;spec&#46;whatwg&#46;org/multipage/interactive-elements&#46;html#the-details-element)</code>, <code><a id="ref-for-the-select-element"></a>[select](https://html&#46;spec&#46;whatwg&#46;org/multipage/form-elements&#46;html#the-select-element)</code>, <code><a id="ref-for-the-dialog-element"></a>[dialog](https://html&#46;spec&#46;whatwg&#46;org/multipage/interactive-elements&#46;html#the-dialog-element)</code>, and <code><a id="ref-for-the-input-element①"></a>[input](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#the-input-element)</code> elements, all of which can be toggled “open” to display more content (or any content at all, in the case of <code><a id="ref-for-the-dialog-element①"></a>[dialog](https://html&#46;spec&#46;whatwg&#46;org/multipage/interactive-elements&#46;html#the-dialog-element)</code>).

Note: Being “open” or “closed” is a semantic state. An element not currently being displayed (for example, one that has <a id="ref-for-propdef-visibility"></a>[visibility: collapse](https://drafts.csswg.org/css-display-4/#propdef-visibility), or belongs to a <a id="ref-for-propdef-display"></a>[display: none](https://drafts.csswg.org/css-display-3/#propdef-display) subtree) can still be “open” and will match <a id="ref-for-selectordef-open①"></a>[:open](#selectordef-open).

Note: A ":closed" pseudo-class might be added in the future once the full set of things that support <a id="ref-for-selectordef-open②"></a>[:open](#selectordef-open) is known.

### <a id="popover-open-state"></a>11.2.  Popover State: the <a id="ref-for-selectordef-popover-open"></a>[:popover-open](#selectordef-popover-open) pseudo-class[](#popover-open-state)

Tests

- [popover-open-with-has-sibling-selector.html](https://wpt.fyi/results/css/selectors/popover-open-with-has-sibling-selector.html) [(live test)](http://wpt.live/css/selectors/popover-open-with-has-sibling-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/popover-open-with-has-sibling-selector.html)

The <a id="selectordef-popover-open"></a><strong>:popover-open</strong> pseudo-class represents an element that has both “popover-showing” and “popover-hidden” states and which is currently in the “popover-showing” state.

Note: This is distinct from <a id="ref-for-selectordef-open③"></a>[:open](#selectordef-open) because an element can have element-specific open and closed states and also have separate popover-showing and popover-hidden states associated with being a popover.

Exactly what “popover-showing” and “popover-hidden” states mean is host-language specific, but is exemplified by the presence of the HTML <code><a id="ref-for-attr-popover"></a>[popover](https://html&#46;spec&#46;whatwg&#46;org/multipage/popover&#46;html#attr-popover)</code> attribute and the associated <a id="ref-for-popover-visibility-state"></a>[popover visibility state](https://html.spec.whatwg.org/multipage/popover.html#popover-visibility-state).

### <a id="modal-state"></a>11.3.  Modal (Exclusive Interaction) State: the <a id="ref-for-selectordef-modal"></a>[:modal](#selectordef-modal) pseudo-class[](#modal-state)

Tests

- [modal-pseudo-class.html](https://wpt.fyi/results/css/selectors/modal-pseudo-class.html) [(live test)](http://wpt.live/css/selectors/modal-pseudo-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/modal-pseudo-class.html)

The <a id="selectordef-modal"></a><strong>:modal</strong> pseudo-class represents an element which is in a state that excludes all interaction with elements outside it until it has been dismissed. Multiple elements can be <a id="ref-for-selectordef-modal①"></a>[:modal](#selectordef-modal) simultaneously, with only one of them active (able to receive input).

<a id="example-82b2b6f1"></a>

<strong>Example:</strong>

[](#example-82b2b6f1) For example, the <code><a id="ref-for-the-dialog-element②"></a>[dialog](https://html&#46;spec&#46;whatwg&#46;org/multipage/interactive-elements&#46;html#the-dialog-element)</code> element is <a id="ref-for-selectordef-modal②"></a>[:modal](#selectordef-modal) when opened with the <code><a id="ref-for-dom-dialog-showmodal"></a>[showModal()](https://html&#46;spec&#46;whatwg&#46;org/multipage/interactive-elements&#46;html#dom-dialog-showmodal)</code> API. Similarly, a <a id="ref-for-selectordef-fullscreen"></a>[:fullscreen](#selectordef-fullscreen) element is also <a id="ref-for-selectordef-modal③"></a>:modal when opened with the <code><a id="ref-for-dom-element-requestfullscreen"></a>[requestFullscreen()](https://fullscreen&#46;spec&#46;whatwg&#46;org/#dom-element-requestfullscreen)</code> API, since this prevents interaction with the rest of the page.

### <a id="fullscreen-state"></a>11.4.  Fullscreen Presentation State: the <a id="ref-for-selectordef-fullscreen①"></a>[:fullscreen](#selectordef-fullscreen) pseudo-class[](#fullscreen-state)

The <a id="selectordef-fullscreen"></a><strong>:fullscreen</strong> pseudo-class represents an element which is displayed in a mode that takes up most (usually all) of the screen, such as that defined by the Fullscreen API. [\[FULLSCREEN\]](#biblio-fullscreen)

### <a id="pip-state"></a>11.5.  Picture-in-Picture Presentation State: the <a id="ref-for-selectordef-picture-in-picture"></a>[:picture-in-picture](#selectordef-picture-in-picture) pseudo-class[](#pip-state)

The <a id="selectordef-picture-in-picture"></a><strong>:picture-in-picture</strong> pseudo-class represents an element whose contents are being displayed in a “picture-in-picture” mode (i.e. displayed in a window confined to part of the screen and overlaying other content) rather than embedded in the document. For example, this pseudo-class can be used to style a <code><a id="ref-for-video④"></a>[video](https://html&#46;spec&#46;whatwg&#46;org/multipage/media&#46;html#video)</code> element’s box while its contents are being played in an overlaid window using the Picture-in-Picture API. [\[picture-in-picture\]](#biblio-picture-in-picture)

## <a id="input-pseudos"></a>12.  The Input Pseudo-classes[](#input-pseudos)

The pseudo-classes in this section mostly apply to elements that take user input, such as HTML’s <a id="ref-for-the-input-element②"></a>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element) element.

### <a id="input-states"></a>12.1.  Input Control States[](#input-states)

#### <a id="enableddisabled"></a>12.1.1.  The <a id="ref-for-enabled-pseudo"></a>[:enabled](#enabled-pseudo) and <a id="ref-for-disabled-pseudo"></a>[:disabled](#disabled-pseudo) Pseudo-classes[](#enableddisabled)

Tests

- [enabled-disabled.html](https://wpt.fyi/results/css/selectors/invalidation/enabled-disabled.html) [(live test)](http://wpt.live/css/selectors/invalidation/enabled-disabled.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/enabled-disabled.html)
- [option-disabled-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/option-disabled-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/option-disabled-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/option-disabled-when-ancestor-changes.html)
- [pseudo-enabled-disabled.html](https://wpt.fyi/results/css/selectors/pseudo-enabled-disabled.html) [(live test)](http://wpt.live/css/selectors/pseudo-enabled-disabled.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/pseudo-enabled-disabled.html)

The <a id="enabled-pseudo"></a><strong>:enabled</strong> pseudo-class represents user interface elements that are in an enabled state; such elements must have a corresponding disabled state.

Conversely, the <a id="disabled-pseudo"></a><strong>:disabled</strong> pseudo-class represents user interface elements that are in a disabled state; such elements must have a corresponding enabled state.

What constitutes an enabled state, a disabled state, and a user interface element is host-language-dependent. In a typical document most elements will be neither <a id="ref-for-enabled-pseudo①"></a>[:enabled](#enabled-pseudo) nor <a id="ref-for-disabled-pseudo①"></a>[:disabled](#disabled-pseudo). For example, [\[HTML\]](#biblio-html) defines [non-disabled interactive elements](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-enabled) to be <a id="ref-for-enabled-pseudo②"></a>:enabled, and any such elements that are [explicitly disabled](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-enabled) to be <a id="ref-for-disabled-pseudo②"></a>:disabled.

Note: CSS properties that might affect a user’s ability to interact with a given user interface element do not affect whether it matches <a id="ref-for-enabled-pseudo③"></a>[:enabled](#enabled-pseudo) or <a id="ref-for-disabled-pseudo③"></a>[:disabled](#disabled-pseudo); e.g., the <a id="ref-for-propdef-display①"></a>[display](https://drafts.csswg.org/css-display-3/#propdef-display) and <a id="ref-for-propdef-visibility①"></a>[visibility](https://drafts.csswg.org/css-display-4/#propdef-visibility) properties have no effect on the enabled/disabled state of an element.

#### <a id="rw-pseudos"></a>12.1.2.  The Mutability Pseudo-classes: <a id="ref-for-read-only-pseudo"></a>[:read-only](#read-only-pseudo) and <a id="ref-for-read-write-pseudo"></a>[:read-write](#read-write-pseudo)[](#rw-pseudos)

Tests

- [selector-read-write-type-change-001.html](https://wpt.fyi/results/css/selectors/selector-read-write-type-change-001.html) [(live test)](http://wpt.live/css/selectors/selector-read-write-type-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-read-write-type-change-001.html)
- [selector-read-write-type-change-002.html](https://wpt.fyi/results/css/selectors/selector-read-write-type-change-002.html) [(live test)](http://wpt.live/css/selectors/selector-read-write-type-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-read-write-type-change-002.html)

An element matches <a id="read-write-pseudo"></a><strong>:read-write</strong> if it is user-alterable, as defined by the document language. Otherwise, it is <a id="read-only-pseudo"></a><strong>:read-only</strong>.

For example, in [\[HTML\]](#biblio-html) a [non-disabled non-readonly <code>&#60;input&#62;</code> element](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-read-only) is <a id="ref-for-read-write-pseudo①"></a>[:read-write](#read-write-pseudo), as is any element with the <code>contenteditable</code> attribute set to the true state.

#### <a id="placeholder"></a>12.1.3.  The Placeholder-shown Pseudo-class: <a id="ref-for-placeholder-shown-pseudo"></a>[:placeholder-shown](#placeholder-shown-pseudo)[](#placeholder)

Tests

- [placeholder-shown.html](https://wpt.fyi/results/css/selectors/invalidation/placeholder-shown.html) [(live test)](http://wpt.live/css/selectors/invalidation/placeholder-shown.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/placeholder-shown.html)
- [placeholder-shown.html](https://wpt.fyi/results/css/selectors/placeholder-shown.html) [(live test)](http://wpt.live/css/selectors/placeholder-shown.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/placeholder-shown.html)
- [selector-placeholder-shown-emptify-placeholder.html](https://wpt.fyi/results/css/selectors/selector-placeholder-shown-emptify-placeholder.html) [(live test)](http://wpt.live/css/selectors/selector-placeholder-shown-emptify-placeholder.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-placeholder-shown-emptify-placeholder.html)
- [selector-placeholder-shown-type-change-001.html](https://wpt.fyi/results/css/selectors/selector-placeholder-shown-type-change-001.html) [(live test)](http://wpt.live/css/selectors/selector-placeholder-shown-type-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-placeholder-shown-type-change-001.html)
- [selector-placeholder-shown-type-change-002.html](https://wpt.fyi/results/css/selectors/selector-placeholder-shown-type-change-002.html) [(live test)](http://wpt.live/css/selectors/selector-placeholder-shown-type-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-placeholder-shown-type-change-002.html)
- [selector-placeholder-shown-type-change-003.html](https://wpt.fyi/results/css/selectors/selector-placeholder-shown-type-change-003.html) [(live test)](http://wpt.live/css/selectors/selector-placeholder-shown-type-change-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-placeholder-shown-type-change-003.html)

Input elements can sometimes show placeholder text as a hint to the user on what to type in. See, for example, the <code>placeholder</code> attribute in [\[HTML\]](#biblio-html). The <a id="placeholder-shown-pseudo"></a><strong>:placeholder-shown</strong> pseudo-class matches an input element that is showing such placeholder text, whether that text is given by an attribute or a real element, or is otherwise implied by the UA.

<a id="example-37912861"></a>

<strong>Example:</strong>

[](#example-37912861) For example, according to the semantics of [\[HTML\]](#biblio-html) the <code><a id="ref-for-attr-input-placeholder"></a>[placeholder](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#attr-input-placeholder)</code> attribute on the <code><a id="ref-for-the-input-element③"></a>[input](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#the-input-element)</code> and <code><a id="ref-for-the-textarea-element"></a>[textarea](https://html&#46;spec&#46;whatwg&#46;org/multipage/form-elements&#46;html#the-textarea-element)</code> elements provide placeholder text. The <a id="ref-for-placeholder-shown-pseudo①"></a>[:placeholder-shown](#placeholder-shown-pseudo) class thus applies whenever such placeholder text is shown.

#### <a id="autofill"></a>12.1.4.  The Automatic Input Pseudo-class: <a id="ref-for-selectordef-autofill"></a>[:autofill](#selectordef-autofill)[](#autofill)

The <a id="selectordef-autofill"></a><strong>:autofill</strong> pseudo-class represents input elements that have been automatically filled by the user agent, and have not been subsequently altered by the user.

#### <a id="the-default-pseudo"></a>12.1.5.  The Default-option Pseudo-class: <a id="ref-for-default-pseudo"></a>[:default](#default-pseudo)[](#the-default-pseudo)

The <a id="default-pseudo"></a><strong>:default</strong> pseudo-class applies to the one or more UI elements that are the default among a set of similar elements. Typically applies to context menu items, buttons and select lists/menus.

One example is the default submit button among a set of buttons. Another example is the default option from a popup menu. In a select-many group (such as for pizza toppings), multiple elements can match <a id="ref-for-default-pseudo①"></a>[:default](#default-pseudo). For example, [\[HTML\]](#biblio-html) defines that <a id="ref-for-default-pseudo②"></a>:default matches [the “default button” in a form, the initially-selected <code>&#60;option&#62;</code>(s) in a <code>&#60;select&#62;</code>, and a few other elements.](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-default)

### <a id="input-value-states"></a>12.2.  Input Value States[](#input-value-states)

#### <a id="checked"></a>12.2.1. <a id="indeterminate"></a> The Selected-option Pseudo-classes: <a id="ref-for-checked-pseudo"></a>[:checked](#checked-pseudo), <a id="ref-for-unchecked-pseudo"></a>[:unchecked](#unchecked-pseudo), and <a id="ref-for-indeterminate-pseudo"></a>[:indeterminate](#indeterminate-pseudo)[](#checked)

Radio and checkbox elements can be toggled by the user, and some menu items are “checked” when the user selects them. This state is reflected by the <a id="input-value-pseudo-classes"></a><strong>input value pseudo-classes</strong>.

When such elements are toggled “on” the <a id="checked-pseudo"></a><strong>:checked</strong> pseudo-class applies. For example, [\[HTML\]](#biblio-html) defines that [checked checkboxes, radio buttons, and selected <code>&#60;option&#62;</code> elements](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-checked) match <a id="ref-for-checked-pseudo①"></a>[:checked](#checked-pseudo). Similarly, when such elements are toggled “off”, the <a id="unchecked-pseudo"></a><strong>:unchecked</strong> pseudo-class applies.

If an element that <em>could</em> match <a id="ref-for-checked-pseudo②"></a>[:checked](#checked-pseudo) or <a id="ref-for-unchecked-pseudo①"></a>[:unchecked](#unchecked-pseudo) is neither "on" nor "off", the <a id="indeterminate-pseudo"></a><strong>:indeterminate</strong> pseudo-class applies. <a id="ref-for-indeterminate-pseudo①"></a>[:indeterminate](#indeterminate-pseudo) also matches elements which do not have a notion of being "checked", but whose "value" is still in an indeterminate state, such as a progress meter whose progress percentage is unknown.

These three pseudo-classes are mutually exclusive; an element can only match at most one of them at a time. (And might match none, if the concept of being "checked" doesn’t apply to them.) If the "indeterminate" concept is not mutually exclusive with the "checked"/"unchecked" concept in a host language (for example, in HTML a checkbox’s "indeterminate" state is a separate boolean from its "checked" state boolean), being indeterminate wins over being checked/unchecked for the purpose of matching these pseudo-classes.

<a id="example-33865490"></a>

<strong>Example:</strong>

[](#example-33865490) For example, in HTML:

- Checkboxes match <a id="ref-for-indeterminate-pseudo②"></a>[:indeterminate](#indeterminate-pseudo) if their <code><a id="ref-for-dom-input-indeterminate"></a>[indeterminate](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#dom-input-indeterminate)</code> property is true; otherwise, they match <a id="ref-for-checked-pseudo③"></a>[:checked](#checked-pseudo) or <a id="ref-for-unchecked-pseudo②"></a>[:unchecked](#unchecked-pseudo) depending on their <code><a id="ref-for-dom-input-checked"></a>[checked](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#dom-input-checked)</code> property.

- Radio buttons match <a id="ref-for-checked-pseudo④"></a>[:checked](#checked-pseudo) if their <code><a id="ref-for-dom-input-checked①"></a>[checked](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#dom-input-checked)</code> property is true. Otherwise, they match <a id="ref-for-unchecked-pseudo③"></a>[:unchecked](#unchecked-pseudo) if they’re in a radio group that contains a <a id="ref-for-checked-pseudo⑤"></a>:checked radio button. Otherwise (their <code><a id="ref-for-dom-input-checked②"></a>[checked](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#dom-input-checked)</code> property is false, but they’re alone or in a radio group with no checked radio button), they match <a id="ref-for-indeterminate-pseudo③"></a>[:indeterminate](#indeterminate-pseudo).

- <code><a id="ref-for-the-option-element"></a>[option](https://html&#46;spec&#46;whatwg&#46;org/multipage/form-elements&#46;html#the-option-element)</code> elements within a <code><a id="ref-for-the-select-element①"></a>[select](https://html&#46;spec&#46;whatwg&#46;org/multipage/form-elements&#46;html#the-select-element)</code> match <a id="ref-for-checked-pseudo⑥"></a>[:checked](#checked-pseudo) or <a id="ref-for-unchecked-pseudo④"></a>[:unchecked](#unchecked-pseudo) depending on their <code><a id="ref-for-dom-option-selected"></a>[selected](https://html&#46;spec&#46;whatwg&#46;org/multipage/form-elements&#46;html#dom-option-selected)</code> property. (They’re never <a id="ref-for-indeterminate-pseudo④"></a>[:indeterminate](#indeterminate-pseudo).)

- <code><a id="ref-for-the-progress-element"></a>[progress](https://html&#46;spec&#46;whatwg&#46;org/multipage/form-elements&#46;html#the-progress-element)</code> elements match <a id="ref-for-indeterminate-pseudo⑤"></a>[:indeterminate](#indeterminate-pseudo) if they lack a <code>value</code> attribute

While the <a id="ref-for-input-value-pseudo-classes"></a>[input value pseudo-classes](#input-value-pseudo-classes) are dynamic in nature, and can altered by user action, since they can also be based on the presence of semantic attributes in the document (such as the <code>selected</code> and <code>checked</code> attributes in [\[HTML\]](#biblio-html)), they apply to all media.

### <a id="ui-validity"></a>12.3.  Input Value-checking[](#ui-validity)

#### <a id="validity-pseudos"></a>12.3.1.  The Validity Pseudo-classes: <a id="ref-for-valid-pseudo①"></a>[:valid](#valid-pseudo) and <a id="ref-for-invalid-pseudo"></a>[:invalid](#invalid-pseudo)[](#validity-pseudos)

An element is <a id="valid-pseudo"></a><strong>:valid</strong> or <a id="invalid-pseudo"></a><strong>:invalid</strong> when its contents or value is, respectively, valid or invalid with respect to data validity semantics defined by the document language (e.g. [\[XFORMS11\]](#biblio-xforms11) or [\[HTML\]](#biblio-html)). An element which lacks data validity semantics is neither <a id="ref-for-valid-pseudo②"></a>[:valid](#valid-pseudo) nor <a id="ref-for-invalid-pseudo①"></a>[:invalid](#invalid-pseudo).

Note: There is a difference between an element which has no constraints, and thus would always be <a id="ref-for-valid-pseudo③"></a>[:valid](#valid-pseudo), and one which has no data validity semantics at all, and thus is neither <a id="ref-for-valid-pseudo④"></a>:valid nor <a id="ref-for-invalid-pseudo②"></a>[:invalid](#invalid-pseudo). In HTML, for example, an <code>&#60;input type=&#34;text&#34;&#62;</code> element may have no constraints, but a <a id="ref-for-the-p-element②"></a>[p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element has no validity semantics at all, and so it never matches either of these pseudo-classes.

Tests

- [input-in-range-in-has-with-readonly.html](https://wpt.fyi/results/css/selectors/invalidation/input-in-range-in-has-with-readonly.html) [(live test)](http://wpt.live/css/selectors/invalidation/input-in-range-in-has-with-readonly.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/input-in-range-in-has-with-readonly.html)
- [media-loading-pseudo-classes-in-has.sub.html](https://wpt.fyi/results/css/selectors/invalidation/media-loading-pseudo-classes-in-has.sub.html) [(live test)](http://wpt.live/css/selectors/invalidation/media-loading-pseudo-classes-in-has.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/media-loading-pseudo-classes-in-has.sub.html)

#### <a id="range-pseudos"></a>12.3.2.  The Range Pseudo-classes: <a id="ref-for-in-range-pseudo"></a>[:in-range](#in-range-pseudo) and <a id="ref-for-out-of-range-pseudo"></a>[:out-of-range](#out-of-range-pseudo)[](#range-pseudos)

The <a id="in-range-pseudo"></a><strong>:in-range</strong> and <a id="out-of-range-pseudo"></a><strong>:out-of-range</strong> pseudo-classes apply only to elements that have range limitations. An element is <a id="ref-for-in-range-pseudo①"></a>[:in-range](#in-range-pseudo) or <a id="ref-for-out-of-range-pseudo①"></a>[:out-of-range](#out-of-range-pseudo) when the value that the element is bound to is in range or out of range with respect to its range limits as defined by the document language. An element that lacks data range limits or is not a form control is neither <a id="ref-for-in-range-pseudo②"></a>:in-range nor <a id="ref-for-out-of-range-pseudo②"></a>:out-of-range. E.g. a slider element with a value of 11 presented as a slider control that only represents the values from 1-10 is :out-of-range. Another example is a menu element with a value of "E" that happens to be presented in a popup menu that only has choices "A", "B" and "C".

#### <a id="opt-pseudos"></a>12.3.3.  The Optionality Pseudo-classes: <a id="ref-for-required-pseudo"></a>[:required](#required-pseudo) and <a id="ref-for-optional-pseudo"></a>[:optional](#optional-pseudo)[](#opt-pseudos)

Tests

- [selector-required-type-change-001.html](https://wpt.fyi/results/css/selectors/selector-required-type-change-001.html) [(live test)](http://wpt.live/css/selectors/selector-required-type-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-required-type-change-001.html)
- [selector-required-type-change-002.html](https://wpt.fyi/results/css/selectors/selector-required-type-change-002.html) [(live test)](http://wpt.live/css/selectors/selector-required-type-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-required-type-change-002.html)
- [selector-required.html](https://wpt.fyi/results/css/selectors/selector-required.html) [(live test)](http://wpt.live/css/selectors/selector-required.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-required.html)

A form element is <a id="required-pseudo"></a><strong>:required</strong> or <a id="optional-pseudo"></a><strong>:optional</strong> if a value for it is, respectively, required or optional before the form it belongs to can be validly submitted. Elements that are not form elements are neither required nor optional.

#### <a id="user-pseudos"></a>12.3.4.  The User-interaction Pseudo-classes: <a id="ref-for-user-valid-pseudo"></a>[:user-valid](#user-valid-pseudo) and <a id="ref-for-user-invalid-pseudo"></a>[:user-invalid](#user-invalid-pseudo)[](#user-pseudos)

Tests

- [user-valid-user-invalid.html](https://wpt.fyi/results/css/selectors/invalidation/user-valid-user-invalid.html) [(live test)](http://wpt.live/css/selectors/invalidation/user-valid-user-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/user-valid-user-invalid.html)
- [user-invalid-form-submission-invalidation.html](https://wpt.fyi/results/css/selectors/user-invalid-form-submission-invalidation.html) [(live test)](http://wpt.live/css/selectors/user-invalid-form-submission-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/user-invalid-form-submission-invalidation.html)
- [user-invalid.html](https://wpt.fyi/results/css/selectors/user-invalid.html) [(live test)](http://wpt.live/css/selectors/user-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/user-invalid.html)
- [user-valid.html](https://wpt.fyi/results/css/selectors/user-valid.html) [(live test)](http://wpt.live/css/selectors/user-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/user-valid.html)
- [valid-invalid-form-fieldset.html](https://wpt.fyi/results/css/selectors/valid-invalid-form-fieldset.html) [(live test)](http://wpt.live/css/selectors/valid-invalid-form-fieldset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/valid-invalid-form-fieldset.html)

The <a id="user-invalid-pseudo"></a><strong>:user-invalid</strong> and the <a id="user-valid-pseudo"></a><strong>:user-valid</strong> pseudo-classes represent an element with incorrect or correct input, respectively, but only <em>after</em> the user has significantly interacted with it. Their purpose is to help the user identify mistakes in their input.

These pseudo-classes must at least match their respective elements after the user has attempted to submit the form and before the user has interacted again with the input element or reset the form. They may also match at other times, as would be appropriate for highlighting an error to the user. For example, <a id="ref-for-user-invalid-pseudo①"></a>[:user-invalid](#user-invalid-pseudo) could start matching an <a id="ref-for-invalid-pseudo③"></a>[:invalid](#invalid-pseudo) input element once the user has changed its value and focus has moved to another element; or stop matching only after the user has successfully corrected the input.

Host languages may define more precise matching rules or defer to platform conventions; otherwise the exact behavior is UA-defined. Regardless, the <a id="ref-for-user-invalid-pseudo②"></a>[:user-invalid](#user-invalid-pseudo) pseudo-class must only match <a id="ref-for-invalid-pseudo④"></a>[:invalid](#invalid-pseudo) elements; and the <a id="ref-for-user-valid-pseudo①"></a>[:user-valid](#user-valid-pseudo) pseudo-class must only match <a id="ref-for-valid-pseudo⑤"></a>[:valid](#valid-pseudo) elements. See the <a id="ref-for-selector-user-invalid"></a>[HTML specification](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-user-invalid) for the specific rules pertaining to HTML elements. [\[HTML\]](#biblio-html)

<a id="example-81851d24"></a>

<strong>Example:</strong>

[](#example-81851d24) For example, the <code><a id="ref-for-the-input-element④"></a>[input](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#the-input-element)</code> element in the following document fragment would match <a id="ref-for-invalid-pseudo⑤"></a>[:invalid](#invalid-pseudo) as soon as the page is loaded (because the <code><a id="ref-for-the-input-element⑤"></a>[input](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#the-input-element)</code>’s initial value violates its max constraint) but it won’t match <a id="ref-for-user-invalid-pseudo③"></a>[:user-invalid](#user-invalid-pseudo) until the user significantly interacts with the input field or attempts to submit the form it’s part of.

``` text
<form>
  <label>
    Volume:
    <input name='vol' type=number min=0 max=10 value=11>
  </label>
  ...
</form>
```

## <a id="structural-pseudos"></a>13.  Tree-Structural pseudo-classes[](#structural-pseudos)

Tests

- [selector-structural-pseudo-root.html](https://wpt.fyi/results/css/selectors/selector-structural-pseudo-root.html) [(live test)](http://wpt.live/css/selectors/selector-structural-pseudo-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-structural-pseudo-root.html)

Selectors introduces the concept of <a id="structural-pseudo-classes"></a><strong>structural pseudo-classes</strong> to permit selection based on extra information that lies in the document tree but cannot be represented by other simple selectors or combinators.

Standalone text and other non-element nodes are not counted when calculating the position of an element in the list of children of its parent. When calculating the position of an element in the list of children of its parent, the index numbering starts at 1.

The <a id="ref-for-structural-pseudo-classes①"></a>[structural pseudo-classes](#structural-pseudo-classes) only apply to elements in the document tree; they must never match <a id="ref-for-pseudo-element②⑨"></a>[pseudo-elements](#pseudo-element).

### <a id="the-root-pseudo"></a>13.1.  <a id="ref-for-root-pseudo①"></a>[:root](#root-pseudo) pseudo-class[](#the-root-pseudo)

Tests

- [root-siblings.html](https://wpt.fyi/results/css/selectors/root-siblings.html) [(live test)](http://wpt.live/css/selectors/root-siblings.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/root-siblings.html)

The <a id="root-pseudo"></a><strong>:root</strong> pseudo-class represents an element that is the root of the document.

For example, in a DOM document, the <a id="ref-for-root-pseudo②"></a>[:root](#root-pseudo) pseudo-class matches the <a id="ref-for-document-element"></a>[document element](https://dom.spec.whatwg.org/#document-element). In HTML, this will be the <code><a id="ref-for-the-html-element"></a>[html](https://html&#46;spec&#46;whatwg&#46;org/multipage/semantics&#46;html#the-html-element)</code> element (unless scripting has been used to modify the document).

### <a id="the-empty-pseudo"></a>13.2.  <a id="ref-for-empty-pseudo"></a>[:empty](#empty-pseudo) pseudo-class[](#the-empty-pseudo)

Tests

- [selectors-empty-001.xml](https://wpt.fyi/results/css/selectors/selectors-empty-001.xml) [(live test)](http://wpt.live/css/selectors/selectors-empty-001.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-empty-001.xml)
- [empty-pseudo-in-has-display-none.html](https://wpt.fyi/results/css/selectors/invalidation/empty-pseudo-in-has-display-none.html) [(live test)](http://wpt.live/css/selectors/invalidation/empty-pseudo-in-has-display-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/empty-pseudo-in-has-display-none.html)

The <a id="empty-pseudo"></a><strong>:empty</strong> pseudo-class represents an element that has no children except, optionally, <a id="ref-for-white-space"></a>[document white space characters](https://drafts.csswg.org/css-text-4/#white-space). In terms of the document tree, only element nodes and content nodes (such as [\[DOM\]](#biblio-dom) text nodes, and entity references) whose data has a non-zero length must be considered as affecting emptiness; comments, processing instructions, and other nodes must not affect whether an element is considered empty or not.

<a id="example-c085b235"></a>

<strong>Example:</strong>

[](#example-c085b235) Examples: p:empty is a valid representation of the <code><a id="ref-for-the-p-element③"></a>[p](https://html&#46;spec&#46;whatwg&#46;org/multipage/grouping-content&#46;html#the-p-element)</code> elements in the following HTML fragment:

``` text
<p></p>
<p>
<p> </p>
<p> <!-- comment --></p>
```

div:empty is not a valid representation of the <code>&#60;div&#62;</code> elements in the following fragment:

``` text
<div>text</div>
<div><p></p></div>
<div>&nbsp;</div>
<div><p>bla</p></div>
<div>this is not <p>:empty</p></div>
```

Note: In Level 2 and Level 3 of Selectors, <a id="ref-for-empty-pseudo①"></a>[:empty](#empty-pseudo) did not match elements that contained only white space. This was changed so that that—​given white space is largely collapsible in HTML and is therefore used for source code formatting, and especially because elements with omitted end tags are likely to absorb such white space into their DOM text contents—​elements which authors perceive of as empty can be selected by this selector, as they expect.

### <a id="child-index"></a>13.3.  Child-indexed Pseudo-classes[](#child-index)

Tests

- [child-indexed-during-parse.html](https://wpt.fyi/results/css/selectors/child-indexed-during-parse.html) [(live test)](http://wpt.live/css/selectors/child-indexed-during-parse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/child-indexed-during-parse.html)
- [child-indexed-no-parent.html](https://wpt.fyi/results/css/selectors/child-indexed-no-parent.html) [(live test)](http://wpt.live/css/selectors/child-indexed-no-parent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/child-indexed-no-parent.html)
- [child-indexed-pseudo-class.html](https://wpt.fyi/results/css/selectors/child-indexed-pseudo-class.html) [(live test)](http://wpt.live/css/selectors/child-indexed-pseudo-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/child-indexed-pseudo-class.html)
- [negated-nth-child-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-nth-child-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-nth-child-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-nth-child-when-ancestor-changes.html)
- [negated-nth-last-child-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-nth-last-child-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-nth-last-child-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-nth-last-child-when-ancestor-changes.html)
- [nth-child-containing-ancestor.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-containing-ancestor.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-containing-ancestor.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-containing-ancestor.html)
- [nth-child-in-shadow-root.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-in-shadow-root.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-in-shadow-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-in-shadow-root.html)
- [nth-child-of-attr-largedom.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-attr-largedom.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-attr-largedom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-attr-largedom.html)
- [nth-child-of-attr.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-attr.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-attr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-attr.html)
- [nth-child-of-class-prefix.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-class-prefix.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-class-prefix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-class-prefix.html)
- [nth-child-of-class.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-class.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-class.html)
- [nth-child-of-has.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-has.html)
- [nth-child-of-id-prefix.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-id-prefix.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-id-prefix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-id-prefix.html)
- [nth-child-of-ids.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-ids.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-ids.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-ids.html)
- [nth-child-of-in-ancestor.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-in-ancestor.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-in-ancestor.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-in-ancestor.html)
- [nth-child-of-in-is.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-in-is.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-in-is.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-in-is.html)
- [nth-child-of-in-shadow-root.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-in-shadow-root.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-in-shadow-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-in-shadow-root.html)
- [nth-child-of-is.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-is.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-is.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-is.html)
- [nth-child-of-pseudo-class.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-pseudo-class.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-pseudo-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-pseudo-class.html)
- [nth-child-of-sibling.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-of-sibling.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-of-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-of-sibling.html)
- [nth-child-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-when-ancestor-changes.html)
- [nth-child-when-sibling-changes.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-when-sibling-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-when-sibling-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-when-sibling-changes.html)
- [nth-child-whole-subtree.html](https://wpt.fyi/results/css/selectors/invalidation/nth-child-whole-subtree.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-child-whole-subtree.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-child-whole-subtree.html)
- [nth-last-child-containing-ancestor.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-containing-ancestor.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-containing-ancestor.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-containing-ancestor.html)
- [nth-last-child-in-shadow-root.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-in-shadow-root.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-in-shadow-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-in-shadow-root.html)
- [nth-last-child-of-attr.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-attr.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-attr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-attr.html)
- [nth-last-child-of-class-prefix.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-class-prefix.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-class-prefix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-class-prefix.html)
- [nth-last-child-of-class.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-class.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-class.html)
- [nth-last-child-of-has.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-has.html)
- [nth-last-child-of-id-prefix.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-id-prefix.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-id-prefix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-id-prefix.html)
- [nth-last-child-of-ids.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-ids.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-ids.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-ids.html)
- [nth-last-child-of-in-ancestor.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-in-ancestor.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-in-ancestor.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-in-ancestor.html)
- [nth-last-child-of-in-is.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-in-is.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-in-is.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-in-is.html)
- [nth-last-child-of-in-shadow-root.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-in-shadow-root.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-in-shadow-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-in-shadow-root.html)
- [nth-last-child-of-is.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-is.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-is.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-is.html)
- [nth-last-child-of-pseudo-class.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-pseudo-class.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-pseudo-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-pseudo-class.html)
- [nth-last-child-of-sibling.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-of-sibling.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-of-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-of-sibling.html)
- [nth-last-child-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-when-ancestor-changes.html)
- [nth-last-child-when-sibling-changes.html](https://wpt.fyi/results/css/selectors/invalidation/nth-last-child-when-sibling-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-last-child-when-sibling-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-last-child-when-sibling-changes.html)
- [nth-child-and-nth-last-child.html](https://wpt.fyi/results/css/selectors/nth-child-and-nth-last-child.html) [(live test)](http://wpt.live/css/selectors/nth-child-and-nth-last-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-and-nth-last-child.html)
- [nth-child-large-anplusb-clamp.html](https://wpt.fyi/results/css/selectors/nth-child-large-anplusb-clamp.html) [(live test)](http://wpt.live/css/selectors/nth-child-large-anplusb-clamp.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-large-anplusb-clamp.html)
- [nth-child-of-attribute.html](https://wpt.fyi/results/css/selectors/nth-child-of-attribute.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-attribute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-attribute.html)
- [nth-child-of-classname-002.html](https://wpt.fyi/results/css/selectors/nth-child-of-classname-002.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-classname-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-classname-002.html)
- [nth-child-of-classname.html](https://wpt.fyi/results/css/selectors/nth-child-of-classname.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-classname.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-classname.html)
- [nth-child-of-complex-selector-many-children-2.html](https://wpt.fyi/results/css/selectors/nth-child-of-complex-selector-many-children-2.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-complex-selector-many-children-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-complex-selector-many-children-2.html)
- [nth-child-of-complex-selector-many-children.html](https://wpt.fyi/results/css/selectors/nth-child-of-complex-selector-many-children.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-complex-selector-many-children.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-complex-selector-many-children.html)
- [nth-child-of-complex-selector.html](https://wpt.fyi/results/css/selectors/nth-child-of-complex-selector.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-complex-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-complex-selector.html)
- [nth-child-of-compound-selector.html](https://wpt.fyi/results/css/selectors/nth-child-of-compound-selector.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-compound-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-compound-selector.html)
- [nth-child-of-has.html](https://wpt.fyi/results/css/selectors/nth-child-of-has.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-has.html)
- [nth-child-of-nesting.html](https://wpt.fyi/results/css/selectors/nth-child-of-nesting.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-nesting.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-nesting.html)
- [nth-child-of-no-space-after-of.html](https://wpt.fyi/results/css/selectors/nth-child-of-no-space-after-of.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-no-space-after-of.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-no-space-after-of.html)
- [nth-child-of-not.html](https://wpt.fyi/results/css/selectors/nth-child-of-not.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-not.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-not.html)
- [nth-child-of-nth-child.html](https://wpt.fyi/results/css/selectors/nth-child-of-nth-child.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-nth-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-nth-child.html)
- [nth-child-of-pseudo.html](https://wpt.fyi/results/css/selectors/nth-child-of-pseudo.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-pseudo.html)
- [nth-child-of-tagname.html](https://wpt.fyi/results/css/selectors/nth-child-of-tagname.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-tagname.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-tagname.html)
- [nth-child-of-universal-selector.html](https://wpt.fyi/results/css/selectors/nth-child-of-universal-selector.html) [(live test)](http://wpt.live/css/selectors/nth-child-of-universal-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-of-universal-selector.html)
- [nth-child-specificity-1.html](https://wpt.fyi/results/css/selectors/nth-child-specificity-1.html) [(live test)](http://wpt.live/css/selectors/nth-child-specificity-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-specificity-1.html)
- [nth-child-specificity-2.html](https://wpt.fyi/results/css/selectors/nth-child-specificity-2.html) [(live test)](http://wpt.live/css/selectors/nth-child-specificity-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-specificity-2.html)
- [nth-child-specificity-3.html](https://wpt.fyi/results/css/selectors/nth-child-specificity-3.html) [(live test)](http://wpt.live/css/selectors/nth-child-specificity-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-specificity-3.html)
- [nth-child-specificity-4.html](https://wpt.fyi/results/css/selectors/nth-child-specificity-4.html) [(live test)](http://wpt.live/css/selectors/nth-child-specificity-4.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-specificity-4.html)
- [nth-child-spurious-brace-crash.html](https://wpt.fyi/results/css/selectors/nth-child-spurious-brace-crash.html) [(live test)](http://wpt.live/css/selectors/nth-child-spurious-brace-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-child-spurious-brace-crash.html)
- [nth-last-child-invalid.html](https://wpt.fyi/results/css/selectors/nth-last-child-invalid.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-invalid.html)
- [nth-last-child-of-classname.html](https://wpt.fyi/results/css/selectors/nth-last-child-of-classname.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-of-classname.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-of-classname.html)
- [nth-last-child-of-complex-selector.html](https://wpt.fyi/results/css/selectors/nth-last-child-of-complex-selector.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-of-complex-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-of-complex-selector.html)
- [nth-last-child-of-compound-selector.html](https://wpt.fyi/results/css/selectors/nth-last-child-of-compound-selector.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-of-compound-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-of-compound-selector.html)
- [nth-last-child-of-nesting.html](https://wpt.fyi/results/css/selectors/nth-last-child-of-nesting.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-of-nesting.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-of-nesting.html)
- [nth-last-child-of-no-space-after-of.html](https://wpt.fyi/results/css/selectors/nth-last-child-of-no-space-after-of.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-of-no-space-after-of.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-of-no-space-after-of.html)
- [nth-last-child-of-style-sharing-1.html](https://wpt.fyi/results/css/selectors/nth-last-child-of-style-sharing-1.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-of-style-sharing-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-of-style-sharing-1.html)
- [nth-last-child-of-style-sharing-2.html](https://wpt.fyi/results/css/selectors/nth-last-child-of-style-sharing-2.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-of-style-sharing-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-of-style-sharing-2.html)
- [nth-last-child-of-tagname.html](https://wpt.fyi/results/css/selectors/nth-last-child-of-tagname.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-of-tagname.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-of-tagname.html)
- [nth-last-child-specificity-1.html](https://wpt.fyi/results/css/selectors/nth-last-child-specificity-1.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-specificity-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-specificity-1.html)
- [nth-last-child-specificity-2.html](https://wpt.fyi/results/css/selectors/nth-last-child-specificity-2.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-specificity-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-specificity-2.html)
- [nth-last-child-specificity-3.html](https://wpt.fyi/results/css/selectors/nth-last-child-specificity-3.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-specificity-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-specificity-3.html)
- [nth-last-child-specificity-4.html](https://wpt.fyi/results/css/selectors/nth-last-child-specificity-4.html) [(live test)](http://wpt.live/css/selectors/nth-last-child-specificity-4.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-last-child-specificity-4.html)
- [sharing-in-svg-use.html](https://wpt.fyi/results/css/selectors/sharing-in-svg-use.html) [(live test)](http://wpt.live/css/selectors/sharing-in-svg-use.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/sharing-in-svg-use.html)
- [parse-anplusb.html](https://wpt.fyi/results/css/selectors/parsing/parse-anplusb.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-anplusb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-anplusb.html)

The pseudo-classes defined in this section select elements based on their index amongst their <a id="ref-for-concept-tree-inclusive-sibling"></a>[inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling).

Note: Selectors 3 described these selectors as selecting elements based on their index in the child list of their parents. (This description survives in the name of this very section, and the names of several of the pseudo-classes.) As there was no reason to exclude them from matching elements without parents, or with non-element parents, they have been rephrased to refer to an element’s relative index amongst its siblings.

#### <a id="the-nth-child-pseudo"></a>13.3.1.  <a id="ref-for-nth-child-pseudo"></a>[:nth-child()](#nth-child-pseudo) pseudo-class[](#the-nth-child-pseudo)

Tests

- [nth-of-invalid.html](https://wpt.fyi/results/css/selectors/nth-of-invalid.html) [(live test)](http://wpt.live/css/selectors/nth-of-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-of-invalid.html)
- [nth-of-selector-before.html](https://wpt.fyi/results/css/selectors/nth-of-selector-before.html) [(live test)](http://wpt.live/css/selectors/nth-of-selector-before.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-of-selector-before.html)
- [nth-child-of-attribute-crash.html](https://wpt.fyi/results/css/selectors/invalidation/crashtests/nth-child-of-attribute-crash.html) [(live test)](http://wpt.live/css/selectors/invalidation/crashtests/nth-child-of-attribute-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/crashtests/nth-child-of-attribute-crash.html)
- [nth-of-namespace-class-invalidation-crash.html](https://wpt.fyi/results/css/selectors/invalidation/nth-of-namespace-class-invalidation-crash.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-of-namespace-class-invalidation-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-of-namespace-class-invalidation-crash.html)

The <a id="nth-child-pseudo"></a><strong>:nth-child(<var>An+B</var> \[of <var>S</var>\]? )</strong> pseudo-class notation represents elements that are among <var>An+B</var>th elements from the list composed of their <a id="ref-for-concept-tree-inclusive-sibling①"></a>[inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling) that match the <a id="ref-for-selector-list⑧"></a>[selector list](#selector-list) <var>S</var>, which is a <a id="ref-for-typedef-complex-real-selector-list①"></a>[\<complex-real-selector-list\>](#typedef-complex-real-selector-list). If <var>S</var> is omitted, it defaults to \*\|\*.

The <var>An+B</var> notation and its interpretation are defined in [CSS Syntax 3 § 6 The An+B microsyntax](https://drafts.csswg.org/css-syntax-3/#anb-microsyntax); it represents any index <var>i</var> = <var>A</var><var>n</var> + <var>B</var> for any non-negative integer <var>n</var>.

Note: For these purposes, the list of elements is <strong>1-indexed</strong>; that is, the first child of an element has index 1, and will be matched by :nth-child(2n+1), because when <code>n=0</code> the expression evaluates to 1.

For example, this selector could address every other row in a table, and could be used to alternate the color of paragraph text in a cycle of four.

<a id="example-b8d691c9"></a>

<strong>Example:</strong>

[](#example-b8d691c9) Examples:

``` text
:nth-child(even)   /* represents the 2nd, 4th, 6th, etc elements
:nth-child(10n-1)  /* represents the 9th, 19th, 29th, etc elements */
:nth-child(10n+9)  /* Same */
:nth-child(10n+-1) /* Syntactically invalid, and would be ignored */
```

Note: The specificity of the <a id="ref-for-nth-child-pseudo①"></a>[:nth-child()](#nth-child-pseudo) pseudo-class is the specificity of a single pseudo-class plus, if <var>S</var> is specified, the specificity of the most specific <a id="ref-for-complex⑤"></a>[complex selector](#complex) in <var>S</var>. See [§ 15 Calculating a selector’s specificity](#specificity-rules). Thus <var>S</var>:nth-child(<var>An+B</var>) and :nth-child(<var>An+B</var> of <var>S</var>) have the exact same specificity, although they do differ in behavior (see example below).

<a id="example-3c07f717"></a>

<strong>Example:</strong>

[](#example-3c07f717) By passing a selector argument, we can select the Nth element that matches that selector. For example, the following selector matches the first three “important” list items, denoted by the .important class:

``` text
:nth-child(-n+3 of li.important)
```

Note that this is different from moving the selector outside of the function, like:

``` text
li.important:nth-child(-n+3)
```

This selector instead just selects the first three children if they also happen to be "important" list items.

<a id="example-48f80a26"></a>

<strong>Example:</strong>

[](#example-48f80a26) Here’s another example of using the selector argument, to ensure that zebra-striping a table works correctly.

Normally, to zebra-stripe a table’s rows, an author would use CSS similar to the following:

``` text
tr {
  background: white;
}
tr:nth-child(even) {
  background: silver;
}
```

However, if some of the rows are hidden and not displayed, this can break up the pattern, causing multiple adjacent rows to have the same background color. Assuming that rows are hidden with the \[hidden\] attribute in HTML, the following CSS would zebra-stripe the table rows robustly, maintaining a proper alternating background regardless of which rows are hidden:

``` text
tr {
  background: white;
}
tr:nth-child(even of :not([hidden])) {
  background: silver;
}
```

#### <a id="the-nth-last-child-pseudo"></a>13.3.2.  <a id="ref-for-nth-last-child-pseudo"></a>[:nth-last-child()](#nth-last-child-pseudo) pseudo-class[](#the-nth-last-child-pseudo)

The <a id="nth-last-child-pseudo"></a><strong>:nth-last-child(<var>An+B</var> \[of <var>S</var>\]? )</strong> pseudo-class notation represents elements that are among <var>An+B</var>th elements from the list composed of their <a id="ref-for-concept-tree-inclusive-sibling②"></a>[inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling) that match the <a id="ref-for-selector-list⑨"></a>[selector list](#selector-list) <var>S</var>, counting backwards from the end. <var>S</var> is <a id="ref-for-typedef-complex-real-selector-list②"></a>[\<complex-real-selector-list\>](#typedef-complex-real-selector-list). If <var>S</var> is omitted, it defaults to \*\|\*.

Note: The specificity of the <a id="ref-for-nth-last-child-pseudo①"></a>[:nth-last-child()](#nth-last-child-pseudo) pseudo-class, like the <a id="ref-for-nth-child-pseudo②"></a>[:nth-child()](#nth-child-pseudo) pseudo-class, combines the specificity of a regular pseudo-class with that of its selector argument <var>S</var>. See [§ 15 Calculating a selector’s specificity](#specificity-rules).

The CSS Syntax Module [\[CSS3SYN\]](#biblio-css3syn) defines the [<var>An+B</var> notation](https://drafts.csswg.org/css-syntax/#anb).

<a id="example-bee68ad4"></a>

<strong>Example:</strong>

[](#example-bee68ad4) Examples:

``` text
tr:nth-last-child(-n+2)    /* represents the two last rows of an HTML table */

foo:nth-last-child(odd)    /* represents all odd foo elements in their parent element,
                              counting from the last one */
```

#### <a id="the-first-child-pseudo"></a>13.3.3.  <a id="ref-for-first-child-pseudo"></a>[:first-child](#first-child-pseudo) pseudo-class[](#the-first-child-pseudo)

Tests

- [first-child.html](https://wpt.fyi/results/css/selectors/first-child.html) [(live test)](http://wpt.live/css/selectors/first-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/first-child.html)
- [first-child-last-child.html](https://wpt.fyi/results/css/selectors/invalidation/first-child-last-child.html) [(live test)](http://wpt.live/css/selectors/invalidation/first-child-last-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/first-child-last-child.html)

The <a id="first-child-pseudo"></a><strong>:first-child</strong> pseudo-class represents an element that is first among its <a id="ref-for-concept-tree-inclusive-sibling③"></a>[inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling). Same as :nth-child(1).

<a id="example-72934891"></a>

<strong>Example:</strong>

[](#example-72934891) Examples: The following selector represents a <a id="ref-for-the-p-element④"></a>[p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element that is the first child of a <a id="ref-for-the-div-element①"></a>[div](https://html.spec.whatwg.org/multipage/grouping-content.html#the-div-element) element:

``` text
div > p:first-child
```

This selector can represent the <code>p</code> inside the <code>div</code> of the following fragment:

``` text
<p> The last P before the note.</p>
<div class="note">
   <p> The first P inside the note.</p>
</div>
```

but cannot represent the second <code>p</code> in the following fragment:

``` text
<p> The last P before the note.</p>
<div class="note">
   <h2> Note </h2>
   <p> The first P inside the note.</p>
</div>
```

The following two selectors are usually equivalent:

``` text
* > a:first-child /* a is first child of any element */
a:first-child /* Same (assuming a is not the root element) */
```

#### <a id="the-last-child-pseudo"></a>13.3.4.  <a id="ref-for-last-child-pseudo"></a>[:last-child](#last-child-pseudo) pseudo-class[](#the-last-child-pseudo)

Tests

- [child-indexed-during-parse.html](https://wpt.fyi/results/css/selectors/child-indexed-during-parse.html) [(live test)](http://wpt.live/css/selectors/child-indexed-during-parse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/child-indexed-during-parse.html)
- [last-child.html](https://wpt.fyi/results/css/selectors/last-child.html) [(live test)](http://wpt.live/css/selectors/last-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/last-child.html)

The <a id="last-child-pseudo"></a><strong>:last-child</strong> pseudo-class represents an element that is last among its <a id="ref-for-concept-tree-inclusive-sibling④"></a>[inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling). Same as :nth-last-child(1).

<a id="example-48137186"></a> [](#example-48137186) Example: The following selector represents a list item <code>li</code> that is the last child of an ordered list <code>ol</code>.

``` text
ol > li:last-child
```

#### <a id="the-only-child-pseudo"></a>13.3.5.  <a id="ref-for-only-child-pseudo"></a>[:only-child](#only-child-pseudo) pseudo-class[](#the-only-child-pseudo)

Tests

- [child-indexed-during-parse.html](https://wpt.fyi/results/css/selectors/child-indexed-during-parse.html) [(live test)](http://wpt.live/css/selectors/child-indexed-during-parse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/child-indexed-during-parse.html)
- [only-child.html](https://wpt.fyi/results/css/selectors/only-child.html) [(live test)](http://wpt.live/css/selectors/only-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/only-child.html)

The <a id="only-child-pseudo"></a><strong>:only-child</strong> pseudo-class represents an element that has no siblings. Same as :first-child:last-child or :nth-child(1):nth-last-child(1), but with a lower specificity.

### <a id="typed-child-index"></a>13.4.  Typed Child-indexed Pseudo-classes[](#typed-child-index)

Tests

- [nth-of-type-namespace.html](https://wpt.fyi/results/css/selectors/nth-of-type-namespace.html) [(live test)](http://wpt.live/css/selectors/nth-of-type-namespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-of-type-namespace.html)

The pseudo-classes in this section are similar to the [Child Index Pseudo-classes](#child-index), but they resolve based on an element’s index <strong>among elements of the same [type (tag name)](#type-selectors)</strong> in their sibling list.

#### <a id="the-nth-of-type-pseudo"></a>13.4.1.  <a id="ref-for-nth-of-type-pseudo"></a>[:nth-of-type()](#nth-of-type-pseudo) pseudo-class[](#the-nth-of-type-pseudo)

The <a id="nth-of-type-pseudo"></a><strong>:nth-of-type(<var>An+B</var>)</strong> pseudo-class notation represents the same elements that would be matched by :nth-child(\|An+B\| of <var>S</var>), where <var>S</var> is a <a id="ref-for-type-selector①②"></a>[type selector](#type-selector) and namespace prefix matching the element in question. For example, when considering whether an HTML <code><a id="ref-for-the-img-element"></a>[img](https://html&#46;spec&#46;whatwg&#46;org/multipage/embedded-content&#46;html#the-img-element)</code> element matches this <a id="ref-for-pseudo-class②⓪"></a>[pseudo-class](#pseudo-class), the <var>S</var> in question is html\|img (assuming an appropriate <code>html</code> namespace is declared).

<a id="example-3d09bf4d"></a>

<strong>Example:</strong>

[](#example-3d09bf4d) CSS example: This allows an author to alternate the position of floated images:

``` text
img:nth-of-type(2n+1) { float: right; }
img:nth-of-type(2n) { float: left; }
```

Note: If the type of the element is known ahead of time, this pseudo-class is equivalent to using <a id="ref-for-nth-child-pseudo③"></a>[:nth-child()](#nth-child-pseudo) with a type selector. That is, img:nth-of-type(2) is equivalent to \*:nth-child(2 of img).

#### <a id="the-nth-last-of-type-pseudo"></a>13.4.2.  <a id="ref-for-nth-last-of-type-pseudo"></a>[:nth-last-of-type()](#nth-last-of-type-pseudo) pseudo-class[](#the-nth-last-of-type-pseudo)

Tests

- [child-indexed-during-parse.html](https://wpt.fyi/results/css/selectors/child-indexed-during-parse.html) [(live test)](http://wpt.live/css/selectors/child-indexed-during-parse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/child-indexed-during-parse.html)
- [last-of-type.html](https://wpt.fyi/results/css/selectors/last-of-type.html) [(live test)](http://wpt.live/css/selectors/last-of-type.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/last-of-type.html)

The <a id="nth-last-of-type-pseudo"></a><strong>:nth-last-of-type(<var>An+B</var>)</strong> pseudo-class notation represents the same elements that would be matched by :nth-last-child(\|An+B\| of <var>S</var>), where <var>S</var> is a <a id="ref-for-type-selector①③"></a>[type selector](#type-selector) and namespace prefix matching the element in question. For example, when considering whether an HTML <code><a id="ref-for-the-img-element①"></a>[img](https://html&#46;spec&#46;whatwg&#46;org/multipage/embedded-content&#46;html#the-img-element)</code> element matches this <a id="ref-for-pseudo-class②①"></a>[pseudo-class](#pseudo-class), the <var>S</var> in question is html\|img (assuming an appropriate <code>html</code> namespace is declared).

<a id="example-b89418c8"></a> [](#example-b89418c8) Example: To represent all <code>h2</code> children of an XHTML <code>body</code> except the first and last, one could use the following selector:

``` text
body > h2:nth-of-type(n+2):nth-last-of-type(n+2) 
```

In this case, one could also use <a id="ref-for-negation-pseudo⑦"></a>[:not()](#negation-pseudo), although the selector ends up being just as long:

``` text
body > h2:not(:first-of-type):not(:last-of-type) 
```

#### <a id="the-first-of-type-pseudo"></a>13.4.3.  <a id="ref-for-first-of-type-pseudo"></a>[:first-of-type](#first-of-type-pseudo) pseudo-class[](#the-first-of-type-pseudo)

Tests

- [first-of-type.html](https://wpt.fyi/results/css/selectors/first-of-type.html) [(live test)](http://wpt.live/css/selectors/first-of-type.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/first-of-type.html)
- [negated-always-matches-negated-first-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-always-matches-negated-first-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-always-matches-negated-first-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-always-matches-negated-first-of-type-when-ancestor-changes.html)
- [negated-first-of-type-in-nonsubject-position.html](https://wpt.fyi/results/css/selectors/invalidation/negated-first-of-type-in-nonsubject-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-first-of-type-in-nonsubject-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-first-of-type-in-nonsubject-position.html)
- [negated-is-always-matches-negated-first-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-is-always-matches-negated-first-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-is-always-matches-negated-first-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-is-always-matches-negated-first-of-type-when-ancestor-changes.html)
- [negated-is-never-matches-negated-first-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-is-never-matches-negated-first-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-is-never-matches-negated-first-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-is-never-matches-negated-first-of-type-when-ancestor-changes.html)
- [negated-negated-first-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-negated-first-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-negated-first-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-negated-first-of-type-when-ancestor-changes.html)
- [negated-never-matches-negated-first-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-never-matches-negated-first-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-never-matches-negated-first-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-never-matches-negated-first-of-type-when-ancestor-changes.html)
- [of-type-selectors.xhtml](https://wpt.fyi/results/css/selectors/of-type-selectors.xhtml) [(live test)](http://wpt.live/css/selectors/of-type-selectors.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/of-type-selectors.xhtml)

The <a id="first-of-type-pseudo"></a><strong>:first-of-type</strong> pseudo-class represents the same element as :nth-of-type(1).

<a id="example-16f73753"></a> [](#example-16f73753) Example: The following selector represents a definition title <code>dt</code> inside a definition list <code>dl</code>, this <code>dt</code> being the first of its type in the list of children of its parent element.

``` text
dl dt:first-of-type
```

It is a valid description for the first two <code>dt</code> elements in the following example but not for the third one:

``` text
<dl>
  <dt>gigogne</dt>
  <dd>
    <dl>
      <dt>fusée</dt>
      <dd>multistage rocket</dd>
      <dt>table</dt>
      <dd>nest of tables</dd>
    </dl>
  </dd>
</dl>
```

#### <a id="the-last-of-type-pseudo"></a>13.4.4.  <a id="ref-for-last-of-type-pseudo"></a>[:last-of-type](#last-of-type-pseudo) pseudo-class[](#the-last-of-type-pseudo)

Tests

- [child-indexed-during-parse.html](https://wpt.fyi/results/css/selectors/child-indexed-during-parse.html) [(live test)](http://wpt.live/css/selectors/child-indexed-during-parse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/child-indexed-during-parse.html)
- [negated-always-matches-negated-last-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-always-matches-negated-last-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-always-matches-negated-last-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-always-matches-negated-last-of-type-when-ancestor-changes.html)
- [negated-is-always-matches-negated-last-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-is-always-matches-negated-last-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-is-always-matches-negated-last-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-is-always-matches-negated-last-of-type-when-ancestor-changes.html)
- [negated-is-never-matches-negated-last-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-is-never-matches-negated-last-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-is-never-matches-negated-last-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-is-never-matches-negated-last-of-type-when-ancestor-changes.html)
- [negated-last-of-type-invalidation.html](https://wpt.fyi/results/css/selectors/invalidation/negated-last-of-type-invalidation.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-last-of-type-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-last-of-type-invalidation.html)
- [negated-negated-last-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-negated-last-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-negated-last-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-negated-last-of-type-when-ancestor-changes.html)
- [negated-never-matches-negated-last-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-never-matches-negated-last-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-never-matches-negated-last-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-never-matches-negated-last-of-type-when-ancestor-changes.html)

The <a id="last-of-type-pseudo"></a><strong>:last-of-type</strong> pseudo-class represents the same element as :nth-last-of-type(1).

<a id="example-fee30007"></a> [](#example-fee30007) Example: The following selector represents the last data cell <code>td</code> of a table row <code>tr</code>.

``` text
tr > td:last-of-type
```

#### <a id="the-only-of-type-pseudo"></a>13.4.5.  <a id="ref-for-only-of-type-pseudo"></a>[:only-of-type](#only-of-type-pseudo) pseudo-class[](#the-only-of-type-pseudo)

Tests

- [child-indexed-during-parse.html](https://wpt.fyi/results/css/selectors/child-indexed-during-parse.html) [(live test)](http://wpt.live/css/selectors/child-indexed-during-parse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/child-indexed-during-parse.html)
- [only-of-type.html](https://wpt.fyi/results/css/selectors/only-of-type.html) [(live test)](http://wpt.live/css/selectors/only-of-type.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/only-of-type.html)

The <a id="only-of-type-pseudo"></a><strong>:only-of-type</strong> pseudo-class represents the same element as :first-of-type:last-of-type.

## <a id="combinators"></a>14.  Combinators[](#combinators)

### <a id="descendant-combinators"></a>14.1.  Descendant combinator (<code> </code>)[](#descendant-combinators)

Tests

- [parse-descendant.html](https://wpt.fyi/results/css/selectors/parsing/parse-descendant.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-descendant.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-descendant.html)

At times, authors may want selectors to describe an element that is the descendant of another element in the document tree (e.g., "an <a id="ref-for-the-em-element"></a>[em](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-em-element) element that is contained within an <a id="ref-for-the-h1-element⑥"></a>[H1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element"). The <a id="descendant-combinator"></a><strong>descendant combinator</strong> expresses such a relationship.

A descendant combinator is whitespace that separates two <a id="ref-for-compound③⑥"></a>[compound selectors](#compound).

A selector of the form A B represents an element <code>B</code> that is an arbitrary descendant of some ancestor element <code>A</code>.

<a id="example-13246ba1"></a>

<strong>Example:</strong>

[](#example-13246ba1) Examples: For example, consider the following selector:

``` text
h1 em
```

It represents an <a id="ref-for-the-em-element①"></a>[em](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-em-element) element being the descendant of an <a id="ref-for-the-h1-element⑦"></a>[h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element. It is a correct and valid, but partial, description of the following fragment:

``` text
<h1>This <span class="myclass">headline
is <em>very</em> important</span></h1>
```

The following selector:

``` text
div * p
```

represents a <a id="ref-for-the-p-element⑤"></a>[p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element that is a grandchild or later descendant of a <a id="ref-for-the-div-element②"></a>[div](https://html.spec.whatwg.org/multipage/grouping-content.html#the-div-element) element. Note the whitespace on either side of the "\*" is not part of the universal selector; the whitespace is a combinator indicating that the <code>div</code> must be the ancestor of some element, and that that element must be an ancestor of the <code>p</code>. The following selector, which combines descendant combinators and [attribute selectors](#attribute-selectors), represents an element that (1) has the <code>href</code> attribute set and (2) is inside a <code>p</code> that is itself inside a <code>div</code>:

``` text
div p *[href]
```

### <a id="child-combinators"></a>14.2.  Child combinator (<code>&#62;</code>)[](#child-combinators)

Tests

- [parse-child.html](https://wpt.fyi/results/css/selectors/parsing/parse-child.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-child.html)

A <a id="child-combinator"></a><strong>child combinator</strong> describes a childhood relationship between two elements. A child combinator is made of the "greater-than sign" (U+003E, <a id="selectordef-child"></a><strong>\></strong>) code point and separates two <a id="ref-for-compound③⑦"></a>[compound selectors](#compound).

<a id="example-63e612ba"></a>

<strong>Example:</strong>

[](#example-63e612ba) Examples: The following selector represents a <a id="ref-for-the-p-element⑥"></a>[p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element that is child of <code>body</code>:

``` text
body > p
```

The following example combines descendant combinators and child combinators.

``` text
div ol>li p
```

It represents a <a id="ref-for-the-p-element⑦"></a>[p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element that is a descendant of an <a id="ref-for-the-li-element"></a>[li](https://html.spec.whatwg.org/multipage/grouping-content.html#the-li-element) element; the <a id="ref-for-the-li-element①"></a>li element must be the child of an <a id="ref-for-the-ol-element①"></a>[ol](https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element) element; the <a id="ref-for-the-ol-element②"></a>ol element must be a descendant of a <code>div</code>. Notice that the optional white space around the "\>" combinator has been left out.

For information on selecting the first child of an element, please see the section on the <a id="ref-for-first-child-pseudo①"></a>[:first-child](#first-child-pseudo) pseudo-class above.

### <a id="adjacent-sibling-combinators"></a>14.3.  Next-sibling combinator (<code>+</code>)[](#adjacent-sibling-combinators)

Tests

- [insert-sibling-001.html](https://wpt.fyi/results/css/selectors/invalidation/insert-sibling-001.html) [(live test)](http://wpt.live/css/selectors/invalidation/insert-sibling-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/insert-sibling-001.html)
- [insert-sibling-002.html](https://wpt.fyi/results/css/selectors/invalidation/insert-sibling-002.html) [(live test)](http://wpt.live/css/selectors/invalidation/insert-sibling-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/insert-sibling-002.html)
- [insert-sibling-003.html](https://wpt.fyi/results/css/selectors/invalidation/insert-sibling-003.html) [(live test)](http://wpt.live/css/selectors/invalidation/insert-sibling-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/insert-sibling-003.html)
- [insert-sibling-004.html](https://wpt.fyi/results/css/selectors/invalidation/insert-sibling-004.html) [(live test)](http://wpt.live/css/selectors/invalidation/insert-sibling-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/insert-sibling-004.html)
- [sibling.html](https://wpt.fyi/results/css/selectors/invalidation/sibling.html) [(live test)](http://wpt.live/css/selectors/invalidation/sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/sibling.html)
- [parse-sibling.html](https://wpt.fyi/results/css/selectors/parsing/parse-sibling.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-sibling.html)

The <a id="next-sibling-combinator"></a><strong>next-sibling combinator</strong> is made of the “plus sign” (U+002B, <a id="selectordef-adjacent"></a><strong>+</strong>) code point that separates two <a id="ref-for-compound③⑧"></a>[compound selectors](#compound). The elements represented by the two <a id="ref-for-compound③⑨"></a>compound selectors share the same parent in the document tree and the element represented by the first <a id="ref-for-compound④⓪"></a>compound selector immediately precedes the element represented by the second one. Non-element nodes (e.g. text between elements) are ignored when considering the adjacency of elements.

<a id="example-0f0597d9"></a>

<strong>Example:</strong>

[](#example-0f0597d9) Examples: The following selector represents a <a id="ref-for-the-p-element⑧"></a>[p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element immediately following a <a id="ref-for-dfn-math"></a>[math](https://w3c.github.io/mathml-core/#dfn-math) element:

``` text
math + p
```

The following selector is conceptually similar to the one in the previous example, except that it adds an attribute selector — it adds a constraint to the <a id="ref-for-the-h1-element⑧"></a>[h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element, that it must have <code>class=&#34;opener&#34;</code>:

``` text
h1.opener + h2
```

### <a id="general-sibling-combinators"></a>14.4.  Subsequent-sibling combinator (<code>&#126;</code>)[](#general-sibling-combinators)

The <a id="subsequent-sibling-combinator"></a><strong>subsequent-sibling combinator</strong> is made of the "tilde" (U+007E, <a id="selectordef-sibling"></a><strong>~</strong>) code point that separates two <a id="ref-for-compound④①"></a>[compound selectors](#compound). The elements represented by the two <a id="ref-for-compound④②"></a>compound selectors share the same parent in the document tree and the element represented by the first compound selector precedes (not necessarily immediately) the element represented by the second one.

<a id="example-ae614b62"></a>

<strong>Example:</strong>

[](#example-ae614b62)

``` text
h1 ~ pre
```

represents a <a id="ref-for-the-pre-element"></a>[pre](https://html.spec.whatwg.org/multipage/grouping-content.html#the-pre-element) element following an <code>h1</code>. It is a correct and valid, but partial, description of:

``` text
<h1>Definition of the function a</h1>
<p>Function a(x) has to be applied to all figures in the table.</p>
<pre>function a(x) = 12x/13.5</pre>
```

## <a id="specificity-rules"></a>15.  Calculating a selector’s specificity[](#specificity-rules)

A selector’s <a id="specificity"></a><strong>specificity</strong> is calculated for a given element as follows:

- count the number of ID selectors in the selector (= <var>A</var>)
- count the number of class selectors, attributes selectors, and pseudo-classes in the selector (= <var>B</var>)
- count the number of type selectors and pseudo-elements in the selector (= <var>C</var>)
- ignore the universal selector

If the selector is a <a id="ref-for-selector-list①⓪"></a>[selector list](#selector-list), this number is calculated for each selector in the list. For a given matching process against the list, the specificity in effect is that of the most specific selector in the list that matches.

A few pseudo-classes provide “evaluation contexts” for other selectors, and so have their specificity defined specially:

- The specificity of an <a id="ref-for-matches-pseudo①⑥"></a>[:is()](#matches-pseudo), <a id="ref-for-negation-pseudo⑧"></a>[:not()](#negation-pseudo), or <a id="ref-for-has-pseudo①①"></a>[:has()](#has-pseudo) pseudo-class is replaced by the specificity of the most specific <a id="ref-for-complex⑥"></a>[complex selector](#complex) in its <a id="ref-for-selector-list①①"></a>[selector list](#selector-list) argument.
- Analogously, the specificity of an <a id="ref-for-nth-child-pseudo④"></a>[:nth-child()](#nth-child-pseudo) or <a id="ref-for-nth-last-child-pseudo②"></a>[:nth-last-child()](#nth-last-child-pseudo) selector is the specificity of the pseudo-class itself (counting as one pseudo-class selector) plus the specificity of the most specific <a id="ref-for-complex⑦"></a>[complex selector](#complex) in its <a id="ref-for-selector-list①②"></a>[selector list](#selector-list) argument (if any).
- The specificity of a <a id="ref-for-where-pseudo⑤"></a>[:where()](#where-pseudo) pseudo-class is replaced by zero.

<a id="example-66faea1b"></a>

<strong>Example:</strong>

[](#example-66faea1b) For example:

- :is(em, \#foo) has a specificity of (1,0,0)—​like an ID selector (\#foo)—​when matched against any of <code>&#60;em&#62;</code>, <code>&#60;p id=foo&#62;</code>, or <code>&#60;em id=foo&#62;</code>.
- .qux:where(em, \#foo#bar#baz) has a specificity of (0,1,0): only the .qux outside the <a id="ref-for-where-pseudo⑥"></a>[:where()](#where-pseudo) contributes to selector specificity.
- :nth-child(even of li, .item) has a specificity of (0,2,0)—​like a class selector (.item) plus a pseudo-class—​when matched against any of <code>&#60;li&#62;</code>, <code>&#60;ul class=item&#62;</code>, or <code>&#60;li class=item id=foo&#62;</code>.
- :not(em, strong#foo) has a specificity of (1,0,1)—​like a tag selector (strong) combined with an ID selector (\#foo)—​when matched against any element.

Specificities are compared by comparing the three components in order: the specificity with a larger <var>A</var> value is more specific; if the two <var>A</var> values are tied, then the specificity with a larger <var>B</var> value is more specific; if the two <var>B</var> values are also tied, then the specificity with a larger <var>C</var> value is more specific; if all the values are tied, the two specificities are equal.

Due to storage limitations, implementations may have limitations on the size of <var>A</var>, <var>B</var>, or <var>C</var>. If so, values higher than the limit must be clamped to that limit, and not overflow.

<a id="example-d97bd125"></a>

<strong>Example:</strong>

[](#example-d97bd125) Examples:

``` text
*               /* a=0 b=0 c=0 */
LI              /* a=0 b=0 c=1 */
UL LI           /* a=0 b=0 c=2 */
UL OL+LI        /* a=0 b=0 c=3 */
H1 + *[REL=up]  /* a=0 b=1 c=1 */
UL OL LI.red    /* a=0 b=1 c=3 */
LI.red.level    /* a=0 b=2 c=1 */
#x34y           /* a=1 b=0 c=0 */
#s12:not(FOO)   /* a=1 b=0 c=1 */
.foo :is(.bar, #baz)
                /* a=1 b=1 c=0 */
```

Note: Repeated occurrences of the same simple selector are allowed and do increase specificity.

Note: The specificity of the styles specified in an HTML <code>style</code> attribute [is described in CSS Style Attributes](https://www.w3.org/TR/css-style-attr/#interpret). [\[CSSSTYLEATTR\]](#biblio-cssstyleattr)

## <a id="grammar"></a>16. <a id="formal-syntax"></a> Grammar[](#grammar)

Tests

- [selectors-attr-white-space-001.html](https://wpt.fyi/results/css/selectors/selectors-attr-white-space-001.html) [(live test)](http://wpt.live/css/selectors/selectors-attr-white-space-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-attr-white-space-001.html)

Selectors are <a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>[parsed](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) according to the following grammar:

<a id="typedef-selector-list"></a><a id="ref-for-typedef-complex-selector-list"></a><a id="typedef-complex-selector-list"></a><a id="ref-for-typedef-complex-selector①"></a><a id="ref-for-mult-comma"></a><a id="typedef-complex-real-selector-list"></a><a id="ref-for-typedef-complex-real-selector"></a><a id="ref-for-mult-comma①"></a><a id="typedef-compound-selector-list"></a><a id="ref-for-typedef-compound-selector①"></a><a id="ref-for-mult-comma②"></a><a id="typedef-simple-selector-list"></a><a id="ref-for-typedef-simple-selector①"></a><a id="ref-for-mult-comma③"></a><a id="typedef-relative-selector-list"></a><a id="ref-for-typedef-relative-selector①"></a><a id="ref-for-mult-comma④"></a><a id="typedef-relative-real-selector-list"></a><a id="ref-for-typedef-relative-real-selector"></a><a id="ref-for-mult-comma⑤"></a><a id="typedef-complex-selector"></a><a id="ref-for-typedef-complex-selector-unit"></a><a id="ref-for-typedef-combinator"></a><a id="ref-for-mult-opt"></a><a id="ref-for-typedef-complex-selector-unit①"></a><a id="ref-for-mult-zero-plus"></a><a id="typedef-complex-selector-unit"></a><a id="ref-for-typedef-compound-selector②"></a><a id="ref-for-mult-opt①"></a><a id="ref-for-typedef-pseudo-compound-selector①"></a><a id="ref-for-mult-zero-plus①"></a><a id="ref-for-mult-req"></a><a id="typedef-complex-real-selector"></a><a id="ref-for-typedef-compound-selector③"></a><a id="ref-for-typedef-combinator①"></a><a id="ref-for-mult-opt②"></a><a id="ref-for-typedef-compound-selector④"></a><a id="ref-for-mult-zero-plus②"></a><a id="typedef-relative-selector"></a><a id="ref-for-typedef-combinator②"></a><a id="ref-for-mult-opt③"></a><a id="ref-for-typedef-complex-selector②"></a><a id="typedef-relative-real-selector"></a><a id="ref-for-typedef-combinator③"></a><a id="ref-for-mult-opt④"></a><a id="ref-for-typedef-complex-real-selector①"></a><a id="typedef-compound-selector"></a><a id="ref-for-typedef-type-selector"></a><a id="ref-for-mult-opt⑤"></a><a id="ref-for-typedef-subclass-selector"></a><a id="ref-for-mult-zero-plus③"></a><a id="ref-for-mult-req①"></a><a id="typedef-pseudo-compound-selector"></a><a id="ref-for-typedef-pseudo-element-selector"></a><a id="ref-for-typedef-pseudo-class-selector"></a><a id="ref-for-mult-zero-plus④"></a><a id="typedef-simple-selector"></a><a id="ref-for-typedef-type-selector①"></a><a id="ref-for-comb-one"></a><a id="ref-for-typedef-subclass-selector①"></a><a id="typedef-combinator"></a><a id="ref-for-comb-one①"></a><a id="ref-for-comb-one②"></a><a id="typedef-wq-name"></a><a id="ref-for-typedef-ns-prefix"></a><a id="ref-for-mult-opt⑥"></a><a id="ref-for-typedef-ident-token②"></a><a id="typedef-ns-prefix"></a><a id="ref-for-typedef-ident-token③"></a><a id="ref-for-comb-one③"></a><a id="ref-for-mult-opt⑦"></a><a id="typedef-type-selector"></a><a id="ref-for-typedef-wq-name"></a><a id="ref-for-comb-one④"></a><a id="ref-for-typedef-ns-prefix①"></a><a id="ref-for-mult-opt⑧"></a><a id="typedef-subclass-selector"></a><a id="ref-for-typedef-id-selector"></a><a id="ref-for-comb-one⑤"></a><a id="ref-for-typedef-class-selector"></a><a id="ref-for-comb-one⑥"></a><a id="ref-for-typedef-attribute-selector"></a><a id="ref-for-comb-one⑦"></a><a id="ref-for-typedef-pseudo-class-selector①"></a><a id="typedef-id-selector"></a><a id="ref-for-typedef-hash-token"></a><a id="typedef-class-selector"></a><a id="ref-for-typedef-ident-token④"></a><a id="typedef-attribute-selector"></a><a id="ref-for-typedef-wq-name①"></a><a id="ref-for-comb-one⑧"></a><a id="ref-for-typedef-wq-name②"></a><a id="ref-for-typedef-attr-matcher"></a><a id="ref-for-typedef-string-token②"></a><a id="ref-for-comb-one⑨"></a><a id="ref-for-typedef-ident-token⑤"></a><a id="ref-for-typedef-attr-modifier"></a><a id="ref-for-mult-opt⑨"></a><a id="typedef-attr-matcher"></a><a id="ref-for-comb-one①⓪"></a><a id="ref-for-comb-one①①"></a><a id="ref-for-comb-one①②"></a><a id="ref-for-comb-one①③"></a><a id="ref-for-mult-opt①⓪"></a><a id="typedef-attr-modifier"></a><a id="ref-for-comb-one①④"></a><a id="typedef-pseudo-class-selector"></a><a id="ref-for-typedef-ident-token⑥"></a><a id="ref-for-comb-one①⑤"></a><a id="ref-for-typedef-function-token"></a><a id="ref-for-typedef-any-value"></a><a id="typedef-pseudo-element-selector"></a><a id="ref-for-typedef-pseudo-class-selector②"></a><a id="ref-for-comb-one①⑥"></a><a id="ref-for-typedef-legacy-pseudo-element-selector"></a><a id="typedef-legacy-pseudo-element-selector"></a><a id="ref-for-comb-one①⑦"></a><a id="ref-for-comb-one①⑧"></a><a id="ref-for-comb-one①⑨"></a>

``` text
<selector-list> = <complex-selector-list>

<complex-selector-list> = <complex-selector>#
<complex-real-selector-list> = <complex-real-selector>#

<compound-selector-list> = <compound-selector>#

<simple-selector-list> = <simple-selector>#

<relative-selector-list> = <relative-selector>#
<relative-real-selector-list> = <relative-real-selector>#


<complex-selector> = <complex-selector-unit> [ <combinator>? <complex-selector-unit> ]*
<complex-selector-unit> = [ <compound-selector>? <pseudo-compound-selector>* ]!
<complex-real-selector> = <compound-selector> [ <combinator>? <compound-selector> ]*

<relative-selector> = <combinator>? <complex-selector>
<relative-real-selector> = <combinator>? <complex-real-selector>

<compound-selector> = [ <type-selector>? <subclass-selector>* ]!
<pseudo-compound-selector> =  <pseudo-element-selector> <pseudo-class-selector>*

<simple-selector> = <type-selector> | <subclass-selector>


<combinator> = '>' | '+' | '~'

<wq-name> = <ns-prefix>? <ident-token>
<ns-prefix> = [ <ident-token> | '*' ]? '|'

<type-selector> = <wq-name> | <ns-prefix>? '*'

<subclass-selector> = <id-selector> | <class-selector> |
                      <attribute-selector> | <pseudo-class-selector>

<id-selector> = <hash-token>

<class-selector> = '.' <ident-token>

<attribute-selector> = '[' <wq-name> ']' |
    '[' <wq-name> <attr-matcher> [ <string-token> | <ident-token> ] <attr-modifier>? ']'
<attr-matcher> = [ '~' | '|' | '^' | '$' | '*' ]? '='
<attr-modifier> = i | s

<pseudo-class-selector> = : <ident-token> |
                          : <function-token> <any-value> )

<pseudo-element-selector> = : <pseudo-class-selector> | <legacy-pseudo-element-selector>
<legacy-pseudo-element-selector> =  : [before | after | first-line | first-letter]
```

In interpreting the above grammar, the following rules apply:

- <a id="white-space"></a> [](#white-space) White space is forbidden:

  - Between any of the top-level components of a <a id="ref-for-typedef-compound-selector⑤"></a>[\<compound-selector\>](#typedef-compound-selector) or <a id="ref-for-typedef-complex-selector-unit②"></a>[\<complex-selector-unit\>](#typedef-complex-selector-unit) (that is, forbidden between the <a id="ref-for-typedef-type-selector②"></a>[\<type-selector\>](#typedef-type-selector) and <a id="ref-for-typedef-subclass-selector②"></a>[\<subclass-selector\>](#typedef-subclass-selector), or between the <a id="ref-for-typedef-pseudo-element-selector①"></a>[\<pseudo-element-selector\>](#typedef-pseudo-element-selector) and <a id="ref-for-typedef-pseudo-class-selector③"></a>[\<pseudo-class-selector\>](#typedef-pseudo-class-selector), etc).

  - Between <em>any</em> of the components of a <a id="ref-for-typedef-type-selector③"></a>[\<type-selector\>](#typedef-type-selector) or a <a id="ref-for-typedef-class-selector①"></a>[\<class-selector\>](#typedef-class-selector).

  - Between the ':'s, or between the ':' and <a id="ref-for-typedef-ident-token⑦"></a>[\<ident-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-ident-token) or <a id="ref-for-typedef-function-token①"></a>[\<function-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-function-token), of a <a id="ref-for-typedef-pseudo-element-selector②"></a>[\<pseudo-element-selector\>](#typedef-pseudo-element-selector) or a <a id="ref-for-typedef-pseudo-class-selector④"></a>[\<pseudo-class-selector\>](#typedef-pseudo-class-selector).

  - Between <em>any</em> of the components of a <a id="ref-for-typedef-wq-name③"></a>[\<wq-name\>](#typedef-wq-name).

  - Between the components of an <a id="ref-for-typedef-attr-matcher①"></a>[\<attr-matcher\>](#typedef-attr-matcher).

  - Between the <a id="ref-for-typedef-compound-selector⑥"></a>[\<compound-selector\>](#typedef-compound-selector) or <a id="ref-for-typedef-pseudo-compound-selector②"></a>[\<pseudo-compound-selector\>](#typedef-pseudo-compound-selector)s in a <a id="ref-for-typedef-complex-selector-unit③"></a>[\<complex-selector-unit\>](#typedef-complex-selector-unit)

  Whitespace is <em>required</em> between two <a id="ref-for-typedef-complex-selector-unit④"></a>[\<complex-selector-unit\>](#typedef-complex-selector-unit)s if the <a id="ref-for-typedef-combinator④"></a>[\<combinator\>](#typedef-combinator) between them is omitted. (This indicates the descendant combinator is being used.)

- In <a id="ref-for-typedef-id-selector①"></a>[\<id-selector\>](#typedef-id-selector), the <a id="ref-for-typedef-hash-token①"></a>[\<hash-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-hash-token)’s value must be an <a id="ref-for-css-css-identifier③"></a>[identifier](https://drafts.csswg.org/css-values-4/#css-css-identifier).

- The <a id="ref-for-typedef-pseudo-class-selector⑤"></a>[\<pseudo-class-selector\>](#typedef-pseudo-class-selector) production excludes the <a id="ref-for-typedef-legacy-pseudo-element-selector①"></a>[\<legacy-pseudo-element-selector\>](#typedef-legacy-pseudo-element-selector) production. (That is, <a id="ref-for-selectordef-before⑥"></a>[:before](https://drafts.csswg.org/css2/#selectordef-before)/etc must never be parsed as a pseudo-class, even if doing so would cause the selector to become valid due to, for example, other simple selectors following it.)

Note: A selector is also subject to a variety of more specific syntactic constraints, and adherence to the grammar above is necessary <em>but not sufficient</em> for the selector to be considered valid. See [§ 3.9 Invalid Selectors and Error Handling](#invalid) for additional rules for parsing selectors.

Note: In general, a <a id="ref-for-typedef-pseudo-element-selector③"></a>[\<pseudo-element-selector\>](#typedef-pseudo-element-selector) is only valid if placed at the end of the last <a id="ref-for-typedef-compound-selector⑦"></a>[\<compound-selector\>](#typedef-compound-selector) in a <a id="ref-for-typedef-complex-selector③"></a>[\<complex-selector\>](#typedef-complex-selector). In some circumstances, however, it can be followed by more <a id="ref-for-typedef-pseudo-element-selector④"></a>\<pseudo-element-selector\>s or <a id="ref-for-typedef-pseudo-class-selector⑥"></a>[\<pseudo-class-selector\>](#typedef-pseudo-class-selector)s; but these are specified on a case-by-case basis. (For example, the <a id="ref-for-user-action-pseudo-class④"></a>[user action pseudo-classes](#user-action-pseudo-class) are allowed after any <a id="ref-for-pseudo-element③⓪"></a>[pseudo-element](#pseudo-element), and the <a id="ref-for-tree-abiding①"></a>[tree-abiding pseudo-elements](https://drafts.csswg.org/css-pseudo-4/#tree-abiding) are allowed after the <a id="ref-for-selectordef-slotted"></a>[::slotted()](https://drafts.csswg.org/css-shadow-1/#selectordef-slotted) pseudo-element.)

<a id="single-colon-pseudos"></a>

<strong>Note:</strong>

[](#single-colon-pseudos) The four [Level 2](https://www.w3.org/TR/CSS2/selectors.html#pseudo-element-selectors) <a id="ref-for-pseudo-element③①"></a>[pseudo-elements](#pseudo-element) (<a id="ref-for-selectordef-before⑦"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before), <a id="ref-for-selectordef-after②"></a>[::after](https://drafts.csswg.org/css-pseudo-4/#selectordef-after), <a id="ref-for-selectordef-first-line⑨"></a>[::first-line](https://drafts.csswg.org/css-pseudo-4/#selectordef-first-line), and <a id="ref-for-selectordef-first-letter②"></a>[::first-letter](https://drafts.csswg.org/css-pseudo-4/#selectordef-first-letter)) may, for legacy reasons, be written with only a single ":" character at their front, making them resemble a <a id="ref-for-typedef-pseudo-class-selector⑦"></a>[\<pseudo-class-selector\>](#typedef-pseudo-class-selector).

### <a id="forgiving-selector"></a>16.1.  <a id="ref-for-typedef-forgiving-selector-list①"></a>[\<forgiving-selector-list\>](#typedef-forgiving-selector-list)[](#forgiving-selector)

For legacy reasons, the general behavior of a selector list is that if any selector in the list fails to parse (because it uses new or UA-specific selector features, for instance), the entire selector list becomes invalid. This can make it hard to write CSS that uses new selectors and still works correctly in older user agents.

The <a id="typedef-forgiving-selector-list"></a><strong><a id="ref-for-typedef-forgiving-selector-list②"></a>[\<forgiving-selector-list\>](#typedef-forgiving-selector-list)</strong> production instead parses each selector in the list individually, simply ignoring ones that fail to parse, so the remaining selectors can still be used.

Note: Style rules still use the normal, unforgiving selector list behavior. <a id="ref-for-typedef-forgiving-selector-list③"></a>[\<forgiving-selector-list\>](#typedef-forgiving-selector-list) is used in <a id="ref-for-matches-pseudo①⑦"></a>[:is()](#matches-pseudo) and <a id="ref-for-where-pseudo⑦"></a>[:where()](#where-pseudo) only. Although it does have some minor implications on specificity, wrapping a style rule’s selector in <a id="ref-for-matches-pseudo①⑧"></a>:is() effectively "upgrades" it to become forgiving, so long as it doesn’t contain any pseudo-elements (which aren’t valid in <a id="ref-for-matches-pseudo①⑨"></a>:is() or <a id="ref-for-where-pseudo⑧"></a>:where()).

Syntactically, <a id="ref-for-typedef-forgiving-selector-list④"></a>[\<forgiving-selector-list\>](#typedef-forgiving-selector-list) is equivalent to <code><a id="ref-for-typedef-any-value①"></a>[\<any-value\>](https://drafts&#46;csswg&#46;org/css-syntax-3/#typedef-any-value)?</code>. It is then <a id="ref-for-parse-as-a-forgiving-selector-list"></a>[parsed as a forgiving selector list](#parse-as-a-forgiving-selector-list) to obtain its actual value.

To <a id="parse-as-a-forgiving-selector-list"></a><strong>parse as a forgiving selector list</strong> given an input <var>input</var>:

1.  <a id="ref-for-css-parse-a-comma-separated-list-according-to-a-css-grammar"></a>[Parse a list](https://drafts.csswg.org/css-syntax-3/#css-parse-a-comma-separated-list-according-to-a-css-grammar) of <a id="ref-for-typedef-complex-real-selector②"></a>[\<complex-real-selector\>](#typedef-complex-real-selector)s from <var>input</var>, and let <var>selector list</var> be the result.

2.  Remove all failure items from <var>selector list</var>, and all items that are <a id="ref-for-invalid-selector④"></a>[invalid selectors](#invalid-selector), then return a <a id="ref-for-typedef-selector-list"></a>[\<selector-list\>](#typedef-selector-list) representing the remaining items in <var>selector list</var>. (This might be empty.)

Any items in a <a id="ref-for-typedef-forgiving-selector-list⑤"></a>[\<forgiving-selector-list\>](#typedef-forgiving-selector-list) that are invalid (whether explicitly, by using unknown selectors or syntax, or merely contextually, using known syntax but in an invalid context) must be treated as having zero specificity.

Note: <a id="ref-for-typedef-forgiving-selector-list⑥"></a>[\<forgiving-selector-list\>](#typedef-forgiving-selector-list) is intentionally used only in <a id="ref-for-matches-pseudo②⓪"></a>[:is()](#matches-pseudo) and <a id="ref-for-where-pseudo⑨"></a>[:where()](#where-pseudo), not in any other selector that takes a selector argument.

## <a id="api-hooks"></a>17.  API Hooks[](#api-hooks)

To aid in the writing of specs that use Selectors concepts, this section defines several API hooks that can be invoked by other specifications.

<a id="issue-55d7bd68"></a>

<strong>Issue:</strong>

[](#issue-55d7bd68) Are these still necessary now that we have more rigorous definitions for <a id="ref-for-match⑦"></a>[match](#match) and <a id="ref-for-invalid-selector⑤"></a>[invalid selector](#invalid-selector)? Nouns are a lot easier to coordinate across specification than predicates, and details like the exact order of elements returned from <code>querySelector</code> seem to make more sense being defined in the DOM specification than in Selectors.

### <a id="parse-selector"></a>17.1.  Parse A Selector[](#parse-selector)

This section defines how to <a id="parse-a-selector"></a><strong>parse a selector</strong> from a string <var>source</var>. It returns either a complex selector list, or failure.

1.  Let <var>selector</var> be the result of <a id="ref-for-css-parse-something-according-to-a-css-grammar①"></a>[parsing](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>source</var> as a <a id="ref-for-typedef-selector-list①"></a>[\<selector-list\>](#typedef-selector-list). If this returns failure, it’s an <a id="ref-for-invalid-selector⑥"></a>[invalid selector](#invalid-selector); return failure.
2.  If <var>selector</var> is an <a id="ref-for-invalid-selector⑦"></a>[invalid selector](#invalid-selector) for any other reason (such as, for example, containing an undeclared namespace prefix), return failure.
3.  Otherwise, return <var>selector</var>.

### <a id="parse-relative-selector"></a>17.2.  Parse A Relative Selector[](#parse-relative-selector)

This section defines how to <a id="parse-a-relative-selector"></a><strong>parse a relative selector</strong> from a string <var>source</var>. It returns either a complex selector list, or failure.

1.  Let <var>selector</var> be the result of <a id="ref-for-css-parse-something-according-to-a-css-grammar②"></a>[parsing](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>source</var> as a <a id="ref-for-typedef-relative-selector-list③"></a>[\<relative-selector-list\>](#typedef-relative-selector-list). If this returns failure, it’s an <a id="ref-for-invalid-selector⑧"></a>[invalid selector](#invalid-selector); return failure.
2.  If <var>selector</var> is an <a id="ref-for-invalid-selector⑨"></a>[invalid selector](#invalid-selector) for any other reason (such as, for example, containing an undeclared namespace prefix), return failure.
3.  Otherwise, return <var>selector</var>.

### <a id="match-against-element"></a>17.3.  Match a Selector Against an Element[](#match-against-element)

This section defines how to <a id="match-a-selector-against-an-element"></a><strong>match a selector against an element</strong>.

APIs using this algorithm must provide a <var>selector</var> and an <var>element</var>.

Callers may optionally provide:

- one or more <a id="ref-for-scoping-root⑨"></a>[scoping roots](#scoping-root), for resolving the <a id="ref-for-scope-pseudo②"></a>[:scope](#scope-pseudo) pseudo-class against.

This algorithm returns either success or failure.

For each <a id="ref-for-complex⑧"></a>[complex selector](#complex) in the given <var>selector</var> (which is taken to be a <a id="ref-for-list-of-simple-selectors①"></a>[list of complex selectors](#list-of-simple-selectors)), match the complex selector against <var>element</var>, as described in the following paragraph. If the matching returns success for any complex selector, then the algorithm return success; otherwise it returns failure.

To <a id="match-a-complex-selector-against-an-element"></a><strong>match a complex selector against an element</strong>, process it <a id="ref-for-compound④③"></a>[compound selector](#compound) at a time, in right-to-left order. This process is defined recursively as follows:

- If any simple selectors in the rightmost compound selector does not match the element, return failure.
- Otherwise, if there is only one compound selector in the complex selector, return success.
- Otherwise, consider all possible elements that could be related to this element by the rightmost <a id="ref-for-selector-combinator⑧"></a>[combinator](#selector-combinator). If the operation of matching the selector consisting of this selector with the rightmost compound selector and rightmost combinator removed against any one of these elements returns success, then return success. Otherwise, return failure.

### <a id="match-against-pseudo-element"></a>17.4.  Match a Selector Against a Pseudo-element[](#match-against-pseudo-element)

This section defines how to <a id="match-a-selector-against-a-pseudo-element"></a><strong>match a selector against a pseudo-element</strong>.

APIs using this algorithm must provide a <var>selector</var> and a <var>pseudo-element</var>. They may optionally provide the same things they may optionally provide to the algorithm to <a id="ref-for-match-a-selector-against-an-element"></a>[match a selector against an element](#match-a-selector-against-an-element).

This algorithm returns success or failure.

For each <a id="ref-for-complex⑨"></a>[complex selector](#complex) in the given <var>selector</var>, if both:

- the rightmost <a id="ref-for-simple①⑥"></a>[simple selector](#simple) in the complex selector matches <var>pseudo-element</var>, and
- the result of running <a id="ref-for-match-a-complex-selector-against-an-element"></a>[match a complex selector against an element](#match-a-complex-selector-against-an-element) on the remainder of the <a id="ref-for-complex①⓪"></a>[complex selector](#complex) (with just the rightmost simple selector of its rightmost complex selector removed), <var>pseudo-element</var>’s corresponding element, and any optional parameters provided to this algorithm returns success,

then return success.

Otherwise (that is, if this doesn’t happen for any of the complex selectors in <var>selector</var>), return failure.

### <a id="match-against-tree"></a>17.5.  Match a Selector Against a Tree[](#match-against-tree)

This section defines how to <a id="match-a-selector-against-a-tree"></a><strong>match a selector against a tree</strong>.

APIs using this algorithm must provide a selector, and one or more <var>root elements</var> indicating the <a id="ref-for-concept-tree"></a>[subtrees](https://dom.spec.whatwg.org/#concept-tree) that will be searched by the selector. All of the <var>root elements</var> must share the same <a id="ref-for-concept-tree-root①"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root), or else calling this algorithm is invalid.

They may optionally provide:

- One or more <a id="ref-for-scoping-root①⓪"></a>[scoping roots](#scoping-root) indicating the selector is <a id="ref-for-scoped-selector②"></a>[scoped](#scoped-selector).

- A list of <a id="ref-for-pseudo-element③②"></a>[pseudo-elements](#pseudo-element) that are allowed to show up in the match list. If not specified, this defaults to allowing all pseudo-elements.

  <a id="issue-15f3317f"></a>

  <strong>Issue:</strong>

  [](#issue-15f3317f) Only the <a id="ref-for-tree-abiding②"></a>[tree-abiding pseudo-elements](https://drafts.csswg.org/css-pseudo-4/#tree-abiding) are really handled in any way remotely like this.

This algorithm returns a (possibly empty) list of elements.

1.  Start with a list of <var>candidate elements</var>, which are the <var>root elements</var> and all of their descendant elements, sorted in <a id="ref-for-concept-shadow-including-tree-order"></a>[shadow-including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order), unless otherwise specified.
2.  If <a id="ref-for-scoping-root①①"></a>[scoping root](#scoping-root) were provided, then remove from the <var>candidate elements</var> any elements that are not <a id="ref-for-concept-tree-descendant"></a>[descendants](https://dom.spec.whatwg.org/#concept-tree-descendant) of at least one <a id="ref-for-scoping-root①②"></a>scoping root.
3.  Initialize the <var>selector match list</var> to empty.
4.  For each <var>element</var> in the set of <var>candidate elements</var>:
    1.  If the result of <a id="ref-for-match-a-selector-against-an-element①"></a>[match a selector against an element](#match-a-selector-against-an-element) for <var>element</var> and <var>selector</var> is success, add <var>element</var> to the <var>selector match list</var>.

    2.  For each possible pseudo-element associated with <var>element</var> that is one of the pseudo-elements allowed to show up in the match list, if the result of <a id="ref-for-match-a-selector-against-a-pseudo-element"></a>[match a selector against a pseudo-element](#match-a-selector-against-a-pseudo-element) for the pseudo-element and <var>selector</var> is success, add the pseudo-element to the <var>selector match list</var>.

        <a id="issue-b7f52036"></a>

        <strong>Issue:</strong>

        [](#issue-b7f52036) The relative position of pseudo-elements in <var>selector match list</var> is undefined. There’s not yet a context that exposes this information, but we need to decide on something eventually, before something <em>is</em> exposed.

## <a id="dom-mapping"></a> Appendix A: Guidance on Mapping Source Documents & Data to an Element Tree[](#dom-mapping)

<em>This section is informative.</em>

The element tree structure described by the DOM is powerful and useful, but generic enough to model pretty much any language that describes tree-based data (or even graph-based, with a suitable interpretation).

Some languages, like HTML, already have well-defined procedures for producing a DOM object from a resource. If a given language does not, such a procedure must be defined in order for Selectors to apply to documents in that language.

At minimum, the document language must define what maps to the DOM concept of an "element".

The primary one-to-many relationship between nodes—​parent/child in tree-based structures, element/neighbors in graph-based structures—​should be reflected as the child nodes of an element.

Other features of the element should be mapped to something that serves a similar purpose to the same feature in DOM:

<strong>type</strong>

If the elements in the document language have some notion of "type" as a basic distinguisher between different groups of elements, it should be reflected as the "type" feature.

If this "type" can be separated into a "basic" name and a "namespace" that groups names into higher-level groups, the latter should be reflected as the "namespace" feature. Otherwise, the element shouldn’t have a "namespace" feature, and the entire name should be reflected as the "type" feature.

<strong>id</strong>

If some aspect of the element functions as a unique identifier across the document, it should be mapped to the "id" feature.

Note: While HTML only allows an element to have a single ID, this should not be taken as a general restriction. The important quality of an ID is that each ID should be associated with a single element; a single element can validly have multiple IDs.

<strong>classes and attributes</strong>

Aspects of the element that are useful for identifying the element, but are not generally unique to elements within a document, should be mapped to the "class" or "attribute" features depending on if they’re something equivalent to a "label" (a string by itself) or a "property" (a name/value pair)

<strong>pseudo-classes and pseudo-elements</strong>

If any elements match any pseudo-classes or have any pseudo-elements, that must be explicitly defined.

<a id="issue-772ec278"></a>

<strong>Issue:</strong>

[](#issue-772ec278) Some pseudo-classes are \*syntactical\*, like <a id="ref-for-has-pseudo①②"></a>[:has()](#has-pseudo) and <a id="ref-for-matches-pseudo②①"></a>[:is()](#matches-pseudo), and thus should always work. Need to indicate that somewhere. Probably the structural pseudos always work whenever the child list is ordered.

In addition to the above, the document language must define how to match strings originating from the document, if it differs from Selector’s default; see [§ 3.7 Characters and case sensitivity](#case-sensitive) for details.

<a id="example-37a1ab6b"></a>

<strong>Example:</strong>

[](#example-37a1ab6b) For example, [JSONSelect](https://github.com/lloyd/JSONSelect) is a library that uses selectors to extract information from JSON documents.

- The "elements" of the JSON document are each array, object, boolean, string, number, or null. The array and object elements have their contents as children.
- Each element’s type is its JS type name: "array", "object", etc.
- Children of an object have their key as a class.
- Children of an array match the <a id="ref-for-first-child-pseudo②"></a>[:first-child](#first-child-pseudo), <a id="ref-for-nth-child-pseudo⑤"></a>[:nth-child()](#nth-child-pseudo), etc pseudo-classes.
- The root object matches <a id="ref-for-root-pseudo③"></a>[:root](#root-pseudo).
- It additionally defines :val() and :contains() pseudo-classes, for matching boolean/number/string elements with a particular value or which contain a particular substring.

This structure is sufficient to allow powerful, compact querying of JSON documents with selectors.

## <a id="compat"></a> Appendix B: Obsolete but Required <code>-webkit-</code> Parsing Quirks for Web Compat[](#compat)

Tests

- [webkit-pseudo-element.html](https://wpt.fyi/results/css/selectors/webkit-pseudo-element.html) [(live test)](http://wpt.live/css/selectors/webkit-pseudo-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/webkit-pseudo-element.html)

<em>This appendix is normative.</em>

Due to legacy Web-compat constraints, user agents expecting to parse Web documents must support the following features:

- :-webkit-autofill must be treated as a <a id="ref-for-legacy-selector-alias①"></a>[legacy selector alias](#legacy-selector-alias) of <a id="ref-for-selectordef-autofill①"></a>[:autofill](#selectordef-autofill).

- All other <a id="ref-for-pseudo-element③③"></a>[pseudo-elements](#pseudo-element) whose names begin with the string “-webkit-” (matched <a id="ref-for-ascii-case-insensitive①⓪"></a>[ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive)) and that are not functional notations must be treated as valid at parse time. (That is, ::-webkit-asdf is valid at parse time, but ::-webkit-jkl() is not.) If they’re not otherwise recognized and supported, they must be treated as matching nothing, and are <a id="unknown--webkit--pseudo-elements"></a><strong>unknown -webkit- pseudo-elements</strong>.

  <a id="ref-for-unknown--webkit--pseudo-elements"></a>[Unknown -webkit- pseudo-elements](#unknown--webkit--pseudo-elements) must be serialized in ASCII lowercase.

  <strong>Note:</strong>

  What’s this quirk about?

  Selectors have long had a behavior where a single unknown/invalid selector invalidates the entire selector list (rather than just invalidating the one complex selector it finds itself in). This is generally considered a legacy mistake by the WG, but can’t be fixed at this point, as too many stylesheets depend on this behavior, intentionally or not.

  One aspect of this is that use of vendor-specific selectors invalidates the entire selector in other user agents that don’t recognize them, and takes the entire style rule down with it. This has been used intentionally in the past—​in the severely-not-recommended practice of hiding style rules from some browsers by making them invalid in every other browser—​and unintentionally, with people styling an element and also applying those styles to a vendor-specific pseudo-element (such as the various <code><a id="ref-for-the-input-element⑥"></a>[input](https://html&#46;spec&#46;whatwg&#46;org/multipage/input&#46;html#the-input-element)</code>-related pseudos some browsers expose), not realizing that this hides the entire rule from other browsers.

  In addition to this more general reasoning, WebKit-derived user agents, such as Safari or Chrome, have an additional quirk related to their vendor-prefixed pseudo-elements, where any ::-webkit--prefixed selectors are considered valid at parse time. (This is probably a leftover quirk of an early CSS feature, since dropped, that intentionally treated all possible pseudo-elements as valid at parse time, in anticipation of a feature letting authors define their own pseudo-elements.)

  Similar to other legacy quirks, such as those documented in [\[QUIRKS\]](#biblio-quirks), this particular vendor-specific oddity has become common enough that other user agents are seeing sites breaking due to them depending on it, accidentally or not. As such, since the quirk is in practical terms <em>required</em> to render the modern web correctly, specifying it and requiring it for all user agents ensures that today’s web pages are more likely to be correctly rendered in user agents both current and future.

  As usual with quirks, however, webpages intentionally relying on this will be met with shaming and derision from members of the CSSWG, and all right-thinking web developers.

## <a id="visited-privacy"></a> Appendix C: Example Privacy-Preserving <a id="ref-for-visited-pseudo⑨"></a>[:visited](#visited-pseudo) Restrictions[](#visited-privacy)

Previous attempts to protect user privacy in <a id="ref-for-visited-pseudo①⓪"></a>[:visited](#visited-pseudo) involved [complex restrictions and behaviors](https://developer.mozilla.org/en-US/docs/Web/CSS/Privacy_and_the_:visited_selector) to "lie" about whether the link match <a id="ref-for-visited-pseudo①①"></a>:visited or <a id="ref-for-link-pseudo⑥"></a>[:link](#link-pseudo), to reduce the chance that a hostile site could observe what unrelated sites a user had visited while still allowing <a id="ref-for-visited-pseudo①②"></a>:visited to work in all cases and help the user know what links they’d already clicked. This is ultimately an arms race that can’t be won; there are multiple documented ways to still extract a user’s browsing history even with these mitigations.

This section describes an approach first developed and documented at <https://github.com/explainers-by-googlers/Partitioning-visited-links-history>, that partitions a user’s browsing history information, to allow <a id="ref-for-visited-pseudo①③"></a>[:visited](#visited-pseudo) to only match links corresponding to navigations that the site’s origin could have observed on its own. With this, <a id="ref-for-visited-pseudo①④"></a>:visited can be treated as a normal pseudo-class, without any of the complex mitigations described above, as it doesn’t expose any information not already theoretically available to the site, while still preserving as much of the <em>usefulness</em> of <a id="ref-for-visited-pseudo①⑤"></a>:visited as possible for the user.

1.  Let <var>visited history</var> be a <a id="ref-for-ordered-set"></a>[set](https://infra.spec.whatwg.org/#ordered-set) containing <a id="ref-for-tuple"></a>[tuples](https://infra.spec.whatwg.org/#tuple) of three pieces of information:

    - a visited <a id="ref-for-concept-url"></a>[URL](https://url.spec.whatwg.org/#concept-url)

    - an <a id="ref-for-concept-origin"></a>[origin](https://html.spec.whatwg.org/multipage/browsers.html#concept-origin) for the site that started a navigation

    - a <a id="ref-for-site"></a>[site](https://html.spec.whatwg.org/multipage/browsers.html#site) for the top-level site containing the frame that started the navigation. (This will often be the same as the previous, but can differ if the user clicks a link in a iframe, for example.)

2.  Whenever a navigation is triggered <em>from within a page</em>—​e.g., from the user clicking a link, or a script on the page initiating a navigation—​add an entry to <var>visited history</var> recording the navigation’s destination URL, the origin of the page containing the link or script, and the (schemeful) site of the top-level site containing that page (which might be the same site as the previous origin).

    Note: This allows a site to see <a id="ref-for-visited-pseudo①⑥"></a>[:visited](#visited-pseudo) information for links that the user has clicked from anywhere in that site’s origin. In other words, any <code>A -&#62; B</code> navigation where the site is A.

    Additionally, add an entry to <var>visited history</var> recording the destination’s URL, the <em>destination’s</em> origin, and the <em>destination’s</em> site. Do this only for navigations from top-level frames or iframes which are same-origin with their top-level frame.

    Note: This allows for a site to see <a id="ref-for-visited-pseudo①⑦"></a>[:visited](#visited-pseudo) information about its own pages (which is already observable by the site) regardless of what site initiated the navigation to that page. In other words, any <code>A -&#62; B</code> navigation where the site is B.

    Note: Notably, direct navigations triggered by the <em>user agent’s</em> UI, such as typing into the address bar, clicking on bookmarks, or dragging a link from another program into the page, <em>do not</em> add a <var>visited history</var> entry. These can, of course, still add to the browser’s record of visited sites that it uses for other purposes, such as suggesting URLs as the user types into the URL bar.

3.  When determining if a link element should match <a id="ref-for-link-pseudo⑦"></a>[:link](#link-pseudo) or <a id="ref-for-visited-pseudo①⑧"></a>[:visited](#visited-pseudo), only allow it to match <a id="ref-for-visited-pseudo①⑨"></a>:visited if the link’s destination, the origin of the page containing the link, and the top-level site containing the link match a tuple in <var>visited history</var>.

The inclusion of both page origin and top-level site prevents several possible privacy attacks, such as:

<strong>Note:</strong>

- If history entries were <em>only</em> keyed by the starting site’s URL, a tracking site could be embedded in a hidden iframe on multiple sites which triggers a navigation to a unique URL for a user on the first visit, and then uses many such links on subsequent visits to see which one had been visited, effectively becoming a new "third-party cookie" identifying the user across the web. By keying the history entry with the top-level site, this information can’t be shared across different sites.

- If history entires were <em>only</em> keyed by the top-level site’s URL, a hostile iframe, perhaps included in a page as part of an advertisement, could observe what sites were visited from the top-level site. By keying the history entry with the link’s own site, the top-level site’s information can’t "leak" into cross-origin iframes.

## <a id="changes"></a>18.  Changes[](#changes)

### <a id="changes-2026-01"></a>18.1.  Changes since the 22 January 2026 Working Draft [](#changes-2026-01)

Significant changes since the [22 January 2026 Working Draft](https://www.w3.org/TR/2026/WD-selectors-4-20260122/):

- Updated HTML5 references to point to HTML LS instead

- Fixed explanation of HTML’s case-matching rules. Defined the default matching of document-supplied strings. ([Issue 2259](https://github.com/w3c/csswg-drafts/issues/2259))

- Used infra terminology for ascii case insensitivity ( [Issue13427](https://github.com/w3c/csswg-drafts/pull/13427))

- Responded to review feedback on BCP 47 language codes ([Issue 13647](https://github.com/w3c/csswg-drafts/issues/13647))

- Added example showing the difference between general case-insensitivity and strict ASCII case insensitivity ([Issue 13645](https://github.com/w3c/csswg-drafts/issues/13645))

- Defined language range as BCP 47 extended language range ([Issue 13646](https://github.com/w3c/csswg-drafts/issues/13646))

- Changed introductory :lang example from (zh, "\*-hant") to (sr, "\*-Cyrl") ([Issue 13644](https://github.com/w3c/csswg-drafts/issues/13644))

- Moved the column combinator to Selectors 5

- Clarified that language codes must be well-formed to match ([Issue 8720](https://github.com/w3c/csswg-drafts/issues/8720))

- Reworded constraints on language ranges ([Issue 8720](https://github.com/w3c/csswg-drafts/issues/8720))

- Clarified that :picture-in-picture applies to the embedder element, not the PiP display ([Issue 13720](https://github.com/w3c/csswg-drafts/issues/13720))

- Renamed \<an+b\> to \<a-n-plus-b\> ([Issue 9473](https://github.com/w3c/csswg-drafts/issues/9473))

- Added CanIUse panels ([Issue 1193](https://github.com/w3c/csswg-drafts/issues/1193))

- Forbade non-valid tags/ranges from ever matching ([Issue 3022](https://github.com/w3c/csswg-drafts/issues/3022#issuecomment-4568519381))

### <a id="changes-2022-11"></a>18.2.  Changes since the 11 November 2022 Working Draft[](#changes-2022-11)

Significant changes since the [11 November 2022 Working Draft](https://www.w3.org/TR/2022/WD-selectors-4-20221111/):

- Defined language range as BCP 47 extended language range ([Issue 13646](https://github.com/w3c/csswg-drafts/issues/13646))

- Removed the at-risk status from <a id="ref-for-read-write-pseudo②"></a>[:read-write](#read-write-pseudo) and <a id="ref-for-has-pseudo①③"></a>[:has()](#has-pseudo)

- Added <a id="ref-for-selectordef-popover-open①"></a>[:popover-open](#selectordef-popover-open) pseudo-class. ([Issue 8637](https://github.com/w3c/csswg-drafts/issues/8637))

- Made <a id="ref-for-has-pseudo①④"></a>[:has()](#has-pseudo) and the selector argument of <a id="ref-for-nth-child-pseudo⑥"></a>[:nth-child()](#nth-child-pseudo)/<a id="ref-for-nth-last-child-pseudo③"></a>[:nth-last-child()](#nth-last-child-pseudo) no longer forgiving. ([Issue 7676](https://github.com/w3c/csswg-drafts/issues/7676))

- Moved the legacy single-colon pseudo-element syntax into the grammar itself. ([Issue 8122](https://github.com/w3c/csswg-drafts/issues/8122))

- Deferred the <a id="ref-for-local-link-pseudo"></a>[:local-link](https://drafts.csswg.org/selectors-5/#local-link-pseudo) pseudo-class to Level 5. ([Issue 12799](https://github.com/w3c/csswg-drafts/issues/12799))

- Deferred the <a id="ref-for-selectordef-interest-source"></a>[:interest-source](https://drafts.csswg.org/selectors-5/#selectordef-interest-source) and <a id="ref-for-selectordef-interest-target"></a>[:interest-target](https://drafts.csswg.org/selectors-5/#selectordef-interest-target) pseudo-classes to Level 5. ([Issue 12799](https://github.com/w3c/csswg-drafts/issues/12799))

- Deferred the <a id="ref-for-blank-pseudo"></a>[:blank](https://drafts.csswg.org/selectors-5/#blank-pseudo) pseudo-class to Level 5. ([Issue 12799](https://github.com/w3c/csswg-drafts/issues/12799))

- Deferred the [grid-structural (column) selectors](https://drafts.csswg.org/selectors-4/selectors-5#grid-structural-selectors) to Level 5. ([Issue 12799](https://github.com/w3c/csswg-drafts/issues/12799))

- Deferred the [time-dimensional pseudo-classes](https://drafts.csswg.org/selectors-4/selectors-5#time-pseudos) to Level 5. ([Issue 12799](https://github.com/w3c/csswg-drafts/issues/12799))

### <a id="changes-2022-05"></a>18.3.  Changes since the 7 May 2022 Working Draft[](#changes-2022-05)

Significant changes since the [7 May 2022 Working Draft](https://www.w3.org/TR/2022/WD-selectors-4-20220507/):

- Added <a id="ref-for-selectordef-open④"></a>[:open](#selectordef-open) pseudo-class. ([Issue 7319](https://github.com/w3c/csswg-drafts/issues/7319), [Issue 11039](https://github.com/w3c/csswg-drafts/issues/11039))

- Disallowed <a id="ref-for-pseudo-element③④"></a>[pseudo-elements](#pseudo-element) from <a id="ref-for-has-pseudo①⑤"></a>[:has()](#has-pseudo) unless explicitly allowed by the pseudo-element’s definition. ([Issue 7463](https://github.com/w3c/csswg-drafts/issues/7463))

- Disallowed nesting of <a id="ref-for-has-pseudo①⑥"></a>[:has()](#has-pseudo). ([Issue 7344](https://github.com/w3c/csswg-drafts/issues/7344))

- Defined matching of ::lang("") and of elements not tagged with a language. ([Issue 6915](https://github.com/w3c/csswg-drafts/issues/6915))

- Untangled the concepts of "scoped" and "relative" selectors completely. ([Issue 6399](https://github.com/w3c/csswg-drafts/issues/6399))

  - Removed "absolutize a selector" as well, and just defined relative selector matching in terms of the anchoring element.

- Reverted compound selector limitation on <a id="ref-for-nth-child-pseudo⑦"></a>[:nth-child()](#nth-child-pseudo). ([Issue 3760](https://github.com/w3c/csswg-drafts/issues/3760))

- Defined :-webkit-autofill <a id="ref-for-legacy-selector-alias②"></a>[legacy selector alias](#legacy-selector-alias). ([Issue 7474](https://github.com/w3c/csswg-drafts/issues/7474))

### <a id="changes-2018-11"></a>18.4.  Changes since the 21 November 2018 Working Draft[](#changes-2018-11)

Significant changes since the [21 November 2018 Working Draft](https://www.w3.org/TR/2018/WD-selectors-4-20181121/):

- Removed the Selector profiles, marked <a id="ref-for-has-pseudo①⑦"></a>[:has()](#has-pseudo) as optional and at-risk instead. ([Issue 3925](https://github.com/w3c/csswg-drafts/issues/3925))
- Added [§ 3.6.4 Sub-pseudo-elements](#sub-pseudo-elements) to define <a id="ref-for-sub-pseudo-element③"></a>[sub-pseudo-elements](#sub-pseudo-element) and related terminology.
- Added <a id="ref-for-defined-pseudo③"></a>[:defined](#defined-pseudo). ([Issue 2258](https://github.com/w3c/csswg-drafts/issues/2258))
- Added <a id="ref-for-selectordef-modal④"></a>[:modal](#selectordef-modal). ([Issue 6965](https://github.com/w3c/csswg-drafts/issues/6965))
- Added <a id="ref-for-selectordef-fullscreen②"></a>[:fullscreen](#selectordef-fullscreen) and <a id="ref-for-selectordef-picture-in-picture①"></a>[:picture-in-picture](#selectordef-picture-in-picture). ([Issue 3796](https://github.com/w3c/csswg-drafts/issues/3796))
- Added <a id="ref-for-selectordef-seeking①"></a>[:seeking](#selectordef-seeking), <a id="ref-for-selectordef-buffering③"></a>[:buffering](#selectordef-buffering), and <a id="ref-for-selectordef-stalled②"></a>[:stalled](#selectordef-stalled) media playback state pseudo-classes. ([Issue 3821](https://github.com/w3c/csswg-drafts/issues/3821))
- Added <a id="ref-for-selectordef-muted①"></a>[:muted](#selectordef-muted) and <a id="ref-for-selectordef-volume-locked①"></a>[:volume-locked](#selectordef-volume-locked) sound state pseudo-classes. ([Issue 3821](https://github.com/w3c/csswg-drafts/issues/3821) and [Issue 3933](https://github.com/w3c/csswg-drafts/issues/3933))
- Added <a id="ref-for-selectordef-autofill②"></a>[:autofill](#selectordef-autofill). ([Issue 5775](https://github.com/w3c/csswg-drafts/issues/5775))
- Added <a id="ref-for-user-valid-pseudo②"></a>[:user-valid](#user-valid-pseudo). ([Discussion](https://lists.w3.org/Archives/Public/www-style/2015Sep/0111.html))
- Defined <a id="ref-for-matches-pseudo②②"></a>[:is()](#matches-pseudo), <a id="ref-for-where-pseudo①⓪"></a>[:where()](#where-pseudo), <a id="ref-for-has-pseudo①⑧"></a>[:has()](#has-pseudo), <a id="ref-for-nth-child-pseudo⑧"></a>[:nth-child()](#nth-child-pseudo), and <a id="ref-for-nth-last-child-pseudo④"></a>[:nth-last-child()](#nth-last-child-pseudo) to not be themselves invalidated when containing an invalid selector. ([Issue 3264](https://github.com/w3c/csswg-drafts/issues/3264))
- Limited selectors in <a id="ref-for-nth-child-pseudo⑨"></a>[:nth-child()](#nth-child-pseudo) and <a id="ref-for-nth-last-child-pseudo⑤"></a>[:nth-last-child()](#nth-last-child-pseudo) to <a id="ref-for-compound④④"></a>[compound selectors](#compound) for now. ([Issue 3760](https://github.com/w3c/csswg-drafts/issues/3760))
- Clarified case-sensitive string matching by referencing string identity as defined in [\[INFRA\]](#biblio-infra).
- Clarified that UA-provided placeholder text still triggers <a id="ref-for-placeholder-shown-pseudo②"></a>[:placeholder-shown](#placeholder-shown-pseudo).
- Rewrote <a id="ref-for-focus-visible-pseudo⑤"></a>[:focus-visible](#focus-visible-pseudo) definition for clarity.
- Switched reminder note in the grammar section to normative text describing the requirement of whitespace between <a id="ref-for-typedef-compound-selector⑧"></a>[\<compound-selector\>](#typedef-compound-selector)s when a <a id="ref-for-typedef-combinator⑤"></a>[\<combinator\>](#typedef-combinator) token is missing.

### <a id="changes-2018-02"></a>18.5.  Changes since the 2 February 2018 Working Draft[](#changes-2018-02)

Significant changes since the [2 February 2018 Working Draft](https://www.w3.org/TR/2018/WD-selectors-4-20180202/):

- Named the zero-specificity selector to <a id="ref-for-where-pseudo①①"></a>[:where()](#where-pseudo). ([Issue 2143](https://github.com/w3c/csswg-drafts/issues/2143))
- Renamed <a id="ref-for-selectordef-matches"></a>[:matches()](#selectordef-matches) to <a id="ref-for-matches-pseudo②③"></a>[:is()](#matches-pseudo). ([Issue 3258](https://github.com/w3c/csswg-drafts/issues/3258))
- Redefined <a id="ref-for-empty-pseudo②"></a>[:empty](#empty-pseudo) to ignore white-space–only nodes. ([Issue 1967](https://github.com/w3c/csswg-drafts/issues/1967))
- Redefined <a id="ref-for-blank-pseudo①"></a>[:blank](https://drafts.csswg.org/selectors-5/#blank-pseudo) to represent empty user input, rather than empty elements. ([Issue 1283](https://github.com/w3c/csswg-drafts/issues/1283))
- Changed the specificity of <a id="ref-for-matches-pseudo②④"></a>[:is()](#matches-pseudo), <a id="ref-for-has-pseudo①⑨"></a>[:has()](#has-pseudo), and <a id="ref-for-nth-child-pseudo①⓪"></a>[:nth-child()](#nth-child-pseudo) to not depend on which selector argument matched. ([Issue 1027](https://github.com/w3c/csswg-drafts/issues/1027))
- Dropped the :drop() pseudo-classes since HTML dropped the related feature. ([Issue 2257](https://github.com/w3c/csswg-drafts/issues/2257))
- Added the case-sensitive flag <code>s</code> to the attribute selector. ([Issue 2101](https://github.com/w3c/csswg-drafts/issues/2101))
- Added further guidance on <a id="ref-for-focus-visible-pseudo⑥"></a>[:focus-visible](#focus-visible-pseudo).
- Added [Appendix B: Obsolete but Required -webkit- Parsing Quirks for Web Compat](#compat) defining ::-webkit- pseudo-element parsing quirk. ([Issue 3051](https://github.com/w3c/csswg-drafts/issues/3051))
- Rewrote grammar rules about where white space is allowed for clarity. (See [§ 16 Grammar](#grammar).)

### <a id="changes-2013"></a>18.6.  Changes since the 2 May 2013 Working Draft[](#changes-2013)

Significant changes since the [2 May 2013 Working Draft](https://www.w3.org/TR/2013/WD-selectors4-20130502/) include:

- Added the :target-within, <a id="ref-for-focus-within-pseudo④"></a>[:focus-within](#focus-within-pseudo), <a id="ref-for-focus-visible-pseudo⑦"></a>[:focus-visible](#focus-visible-pseudo), <a id="ref-for-selectordef-playing③"></a>[:playing](#selectordef-playing), and <a id="ref-for-selectordef-paused①"></a>[:paused](#selectordef-paused) pseudo-classes.

- Added a zero-specificity <a id="ref-for-selectordef-matches①"></a>[:matches()](#selectordef-matches)-type pseudo-class, with name TBD.

- Replaced subject indicator (!) feature with <a id="ref-for-has-pseudo②⓪"></a>[:has()](#has-pseudo).

- Replaced the :nth-match() and :nth-last-match() selectors with :nth-child(… of <var>selector</var>) and :nth-last-child(… of <var>selector</var>).

- Changed the :active-drop-target, :valid-drop-target, :invalid-drop-target with :drop().

- Sketched out an empty-or-whitespace-only selector for discussion (See [open issue](https://github.com/w3c/csswg-drafts/issues/1967).)

- Renamed :user-error to <a id="ref-for-user-invalid-pseudo④"></a>[:user-invalid](#user-invalid-pseudo). (See [Discussion](https://www.w3.org/mid/CADhPm3v+WfeGQfBwwx8QBuiOjn2k38V_DcKW17Cm81VgZb1nbQ@mail.gmail.com))

- Renamed :nth-column()/:nth-last-column() to <a id="ref-for-nth-col-pseudo"></a>[:nth-col()](https://drafts.csswg.org/selectors-5/#nth-col-pseudo)/<a id="ref-for-nth-last-col-pseudo"></a>[:nth-last-col()](https://drafts.csswg.org/selectors-5/#nth-last-col-pseudo) to avoid naming confusion with a potential <a id="ref-for-selectordef-column"></a>[::column](https://drafts.csswg.org/css-multicol-2/#selectordef-column) pseudo-class.

- Changed the non-functional form of the :local-link pseudo-class to account for fragment URLs.

- Removed the functional form of the <code>&#58;local-link()</code> pseudo-class and reference combinator for lack of interest.

- Rewrote selectors grammar using the CSS Value Definition Syntax.

- Split out <a id="ref-for-relative-selector②"></a>[relative selectors](#relative-selector) from <a id="ref-for-scoped-selector③"></a>[scoped selectors](#scoped-selector), as these are different concepts that can be independently invoked.

- Moved definition of <a id="ref-for-anb-production"></a>[\<a-n-plus-b\>](https://drafts.csswg.org/css-syntax-3/#anb-production) microsyntax to CSS Syntax.

  <a id="issue-fb60ae39"></a>

  <strong>Issue:</strong>

  [](#issue-fb60ae39) Semantic definition should probably move back here.

- Added new sections:
  - [§ 3.2 Data Model](#data-model)

    <a id="issue-4ca3978c"></a>

    <strong>Issue:</strong>

    [](#issue-4ca3978c) Need to define tree for XML.

  - [§ 17 API Hooks](#api-hooks)
    - Note that earlier versions of this section defined a section on <a id="evaluating-selectors"></a><strong>evaluating a selector</strong>, but that section is no longer present. Specifications referencing that section should instead reference the algorithm to <a id="ref-for-match-a-selector-against-a-tree"></a>[match a selector against a tree](#match-a-selector-against-a-tree).

- Removed restriction on combinators within <a id="ref-for-selectordef-matches②"></a>[:matches()](#selectordef-matches) and <a id="ref-for-negation-pseudo⑨"></a>[:not()](#negation-pseudo); see [discussion](https://lists.w3.org/Archives/Public/www-style/2014Jan/0607.html).

- Defined <a id="ref-for-specificity②"></a>[specificity](#specificity) of a <a id="ref-for-selector-list①③"></a>[selector list](#selector-list). (Why?)

- Required quotes around <a id="ref-for-lang-pseudo⑥"></a>[:lang()](#lang-pseudo) values involving an asterisk; only language codes which happen to be CSS identifiers can be used unquoted.

Note: The 1 February 2018 draft included an inadvertent commit of unfinished work; 2 February 2018 has reverted this commit (and fixed some links because why not).

### <a id="changes-2012"></a>18.7.  Changes since the 23 August 2012 Working Draft[](#changes-2012)

Significant changes since the [23 August 2012 Working Draft](https://www.w3.org/TR/2012/WD-selectors4-20120823/) include:

- Added <a id="ref-for-placeholder-shown-pseudo③"></a>[:placeholder-shown](#placeholder-shown-pseudo) pseudo-classes.
- Released some restrictions on <a id="ref-for-selectordef-matches③"></a>[:matches()](#selectordef-matches) and <a id="ref-for-negation-pseudo①⓪"></a>[:not()](#negation-pseudo).
- Defined fast and complete Selectors profiles (now called “live” and “snapshot”).
- Improved definition of <a id="ref-for-specificity③"></a>[specificity](#specificity) to better handle <a id="ref-for-selectordef-matches④"></a>[:matches()](#selectordef-matches).
- Updated grammar.
- Cleaned up definition of <a id="ref-for-anb-production①"></a>[\<a-n-plus-b\>](https://drafts.csswg.org/css-syntax-3/#anb-production) notation.
- Added definition of <em>scope-relative</em> selectors, changed <em>scope-constrained</em> to scope-filtered for less confusion with scope-contained.
- The :local-link() pseudo-class now ignores trailing slashes.

### <a id="changes-2011"></a>18.8.  Changes since the 29 September 2011 Working Draft[](#changes-2011)

Significant changes since the [29 September 2011 Working Draft](https://www.w3.org/TR/2011/WD-selectors4-20110929/) include:

- Added language variant handling per RFC 4647.
- Added scoped selectors.
- Added :user-error (now called <a id="ref-for-user-invalid-pseudo⑤"></a>[:user-invalid](#user-invalid-pseudo)).
- Added :valid-drop-target.
- Changed <a id="ref-for-column-combinator"></a>[column combinator](https://drafts.csswg.org/selectors-5/#column-combinator) from double slash to double pipe.

### <a id="changes-level-3"></a>18.9.  Changes Since Level 3[](#changes-level-3)

Additions since [Selectors Level 3](https://www.w3.org/TR/selectors-3/):

#### <a id="changes-conditional"></a>18.9.1. Conditional Logic[](#changes-conditional)

- The matches-any [:is](#matches) pseudo-class, formerly called :matches(), has been added to allow stylesheet authors to group functionally related elements, such as any element that is either hovered or focussed.
- The specificity-adjusting [:where()](#zero-matches) pseudo-class has been added to construct zero-specificity filters, which are easy to override, replacing the many ugly hacks previously required to simulate this.

#### <a id="changes-forms"></a>18.9.2. Interaction, Input and Forms[](#changes-forms)

- Added the [:modal](#modal-state) pseudo-class, to select elements which exclude interaction with all other elements until dismissed.
- Added [:open](#open-state) and [:popover-open](#popover-open-state) pseudo-classes, primarily but not exclusively to use with HTML forms.

<em>The following selectors were previously in [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://drafts.csswg.org/css-ui-3/) and have been moved to the main Selectors 4 specification for ease of reference.</em>

- Added the <a id="ref-for-valid-pseudo⑥"></a>[:valid](#valid-pseudo) and <a id="ref-for-invalid-pseudo⑥"></a>[:invalid](#invalid-pseudo) pseudo-classes.
- Added the <a id="ref-for-user-valid-pseudo③"></a>[:user-valid](#user-valid-pseudo) and <a id="ref-for-user-invalid-pseudo⑥"></a>[:user-invalid](#user-invalid-pseudo) pseudo-classes. These only match after significant user interaction.
- Added the <a id="ref-for-required-pseudo①"></a>[:required](#required-pseudo) and <a id="ref-for-optional-pseudo①"></a>[:optional](#optional-pseudo) optionality pseudo-classes
- Added the <a id="ref-for-enabled-pseudo④"></a>[:enabled](#enabled-pseudo) and <a id="ref-for-disabled-pseudo④"></a>[:disabled](#disabled-pseudo) pseudo-classes.
- Added the <a id="ref-for-read-only-pseudo①"></a>[:read-only](#read-only-pseudo) and <a id="ref-for-read-write-pseudo③"></a>[:read-write](#read-write-pseudo) mutability pseudo-classes.
- Added the automatic input [:autofill](#autofill) pseudo-class.
- Added the selected-option [:checked](#checked), <a id="ref-for-unchecked-pseudo⑤"></a>[:unchecked](#unchecked-pseudo) and <a id="ref-for-indeterminate-pseudo⑥"></a>[:indeterminate](#indeterminate-pseudo) pseudo-classes.

#### <a id="changes-hyperlinking"></a>18.9.3. Hyperlinking Support[](#changes-hyperlinking)

- Added [:any-link](#the-any-link-pseudo) to match any element that acts as the source anchor of a hyperlink.

#### <a id="changes-authorial"></a>18.9.4. Clearer Expression of Authorial Intent[](#changes-authorial)

- Expanded the sibling-counting [:nth-child()](#the-nth-child-pseudo) and negation [:not()](#negation) pseudo-classes to accept a selector list.

#### <a id="changes-media"></a>18.9.5. Media State[](#changes-media)

- Added the :playing, :paused and :seeking [media playback state](#video-state) pseudo classes, to enable selecting on the dynamic state of video elements, functionality which previously required JavaScript.
- Added the <a id="ref-for-selectordef-buffering④"></a>[:buffering](#selectordef-buffering) and <a id="ref-for-selectordef-stalled③"></a>[:stalled](#selectordef-stalled) [media loading state](#media-loading-state) pseudo-classes, again providing functionality that previously needed JavaScript,
- Added the [:fullscreen](#fullscreen-state) and [:picture-in-picture](#pip-state) pseudo-classes.
- Added the <a id="ref-for-selectordef-muted②"></a>[:muted](#selectordef-muted) and <a id="ref-for-selectordef-volume-locked②"></a>[:volume-locked](#selectordef-volume-locked) pseudo-classes for sound-producing elements.

#### <a id="changes-i18n"></a>18.9.6. Internationalization Improvements[](#changes-i18n)

- Expanded the content language [:lang()](#the-lang-pseudo) pseudo-class to accept wildcard matching, and comma-separated lists of language codes, for greater flexibility styling related languages.
- Added the content language writing direction [:dir()](#the-dir-pseudo) pseudo-class for convenient styling of left-to-right and right-to-left content.
- Clarified case-insensitive string matching.
- General improvements to closely align with BCP 47.

## <a id="acknowledgements"></a>19.  Acknowledgements[](#acknowledgements)

The CSS working group would like to thank everyone who contributed to the [previous Selectors](https://www.w3.org/TR/css3-selectors) specifications over the years, as those specifications formed the basis for this one. In particular, the working group would like to extend special thanks to the following for their specific contributions to Selectors Level 4: L. David Baron, Andrew Fedoniouk, Daniel Glazman, Ian Hickson, Grey Hodge, Lachlan Hunt, Anne van Kesteren, Jason Cranford Teague, Lea Verou

## <a id="privacy"></a>Privacy Considerations[](#privacy)

- The <a id="ref-for-visited-pseudo②⓪"></a>[:visited](#visited-pseudo) pseudo-class can expose information about which sites a user has previously visited, if the UA is not careful to screen from scripting any information that would reveal which elements match it.
- The <a id="ref-for-selectordef-autofill③"></a>[:autofill](#selectordef-autofill) pseudo-class can expose whether a user has interacted with this form before; however the same information can be derived by observing how quickly the form is filled out.

## <a id="security"></a>Security Considerations[](#security)

The [Privacy Considerations](#privacy) could also be considered to affect Security.

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

- [+](#selectordef-adjacent), in § 14.3
- [\>](#selectordef-child), in § 14.2
- [~](#selectordef-sibling), in § 14.4
- [:active](#active-pseudo), in § 9.2
- [anchor element](#relative-selector-anchor-elements), in § 3.4
- [:any-link](#any-link-pseudo), in § 8.1
- [\<attribute-selector\>](#typedef-attribute-selector), in § 16
- [attribute selector](#attribute-selector), in § 6
- [\<attr-matcher\>](#typedef-attr-matcher), in § 16
- [\<attr-modifier\>](#typedef-attr-modifier), in § 16
- [:autofill](#selectordef-autofill), in § 12.1.4
- [:buffering](#selectordef-buffering), in § 10.2
- [:checked](#checked-pseudo), in § 12.2.1
- [child combinator](#child-combinator), in § 14.2
- [\<class-selector\>](#typedef-class-selector), in § 16
- [class selector](#class-selector), in § 6.6
- [\<combinator\>](#typedef-combinator), in § 16
- [combinator](#selector-combinator), in § 3.1
- [\<complex-real-selector\>](#typedef-complex-real-selector), in § 16
- [\<complex-real-selector-list\>](#typedef-complex-real-selector-list), in § 16
- [\<complex-selector\>](#typedef-complex-selector), in § 16
- [complex selector](#complex), in § 3.1
- [\<complex-selector-list\>](#typedef-complex-selector-list), in § 16
- [\<complex-selector-unit\>](#typedef-complex-selector-unit), in § 16
- [\<compound-selector\>](#typedef-compound-selector), in § 16
- [compound selector](#compound), in § 3.1
- [\<compound-selector-list\>](#typedef-compound-selector-list), in § 16
- [declared](#nsdecl), in § 3.8
- [:default](#default-pseudo), in § 12.1.5
- [:defined](#defined-pseudo), in § 5.4
- [descendant combinator](#descendant-combinator), in § 14.1
- [:dir()](#dir-pseudo), in § 7.1
- [:disabled](#disabled-pseudo), in § 12.1.1
- [document language](#document-language), in § 3.2
- [:empty](#empty-pseudo), in § 13.2
- [:enabled](#enabled-pseudo), in § 12.1.1
- [evaluating a selector](#evaluating-selectors), in § 18.6
- [featureless](#featureless), in § 3.2.1
- [:first-child](#first-child-pseudo), in § 13.3.3
- [:first-of-type](#first-of-type-pseudo), in § 13.4.3
- [:focus](#focus-pseudo), in § 9.3
- [:focus-visible](#focus-visible-pseudo), in § 9.4
- [:focus-within](#focus-within-pseudo), in § 9.5
- [\<forgiving-selector-list\>](#typedef-forgiving-selector-list), in § 16.1
- [:fullscreen](#selectordef-fullscreen), in § 11.4
- [functional pseudo-class](#functional-pseudo-class), in § 3.5
- [functional pseudo-element](#functional-pseudo-element), in § 3.6.1
- [:has()](#has-pseudo), in § 4.5
- [:has-allowed pseudo-element](#has-allowed-pseudo-element), in § 4.5
- [host language](#host-language), in § 3.2
- [:hover](#hover-pseudo), in § 9.1
- [i](#valdef-attribute-selector-i), in § 6.3
- [\<id-selector\>](#typedef-id-selector), in § 16
- [ID selector](#id-selector), in § 6.7
- [:indeterminate](#indeterminate-pseudo), in § 12.2.1
- [indicate focus](#indicate-focus), in § 9.4
- [input value pseudo-classes](#input-value-pseudo-classes), in § 12.2.1
- [:in-range](#in-range-pseudo), in § 12.3.2
- [:invalid](#invalid-pseudo), in § 12.3.1
- [invalid](#invalid-selector), in § 3.9
- [invalid selector](#invalid-selector), in § 3.9
- [:is()](#matches-pseudo), in § 4.2
- [:lang()](#lang-pseudo), in § 7.2
- [language range](#language-range), in § 7.2
- [:last-child](#last-child-pseudo), in § 13.3.4
- [:last-of-type](#last-of-type-pseudo), in § 13.4.4
- [\<legacy-pseudo-element-selector\>](#typedef-legacy-pseudo-element-selector), in § 16
- [legacy selector alias](#legacy-selector-alias), in § 3.10
- [:link](#link-pseudo), in § 8.2
- [list of complex selectors](#list-of-simple-selectors), in § 3.1
- [list of compound selectors](#list-of-simple-selectors), in § 3.1
- [list of selectors](#selector-list), in § 3.1
- [list of simple selectors](#list-of-simple-selectors), in § 3.1
- [logical combination pseudo-classes](#logical-combination-pseudo-classes), in § 4
- [match](#match), in § 3.1
- [match a complex selector against an element](#match-a-complex-selector-against-an-element), in § 17.3
- [match a selector against an element](#match-a-selector-against-an-element), in § 17.3
- [match a selector against a pseudo-element](#match-a-selector-against-a-pseudo-element), in § 17.4
- [match a selector against a tree](#match-a-selector-against-a-tree), in § 17.5
- [:matches()](#selectordef-matches), in § 4.2
- [:modal](#selectordef-modal), in § 11.3
- [:muted](#selectordef-muted), in § 10.3
- [next-sibling combinator](#next-sibling-combinator), in § 14.3
- [:not()](#negation-pseudo), in § 4.3
- [\<ns-prefix\>](#typedef-ns-prefix), in § 16
- [:nth-child()](#nth-child-pseudo), in § 13.3.1
- [:nth-last-child()](#nth-last-child-pseudo), in § 13.3.2
- [:nth-last-of-type()](#nth-last-of-type-pseudo), in § 13.4.2
- [:nth-of-type()](#nth-of-type-pseudo), in § 13.4.1
- [:only-child](#only-child-pseudo), in § 13.3.5
- [:only-of-type](#only-of-type-pseudo), in § 13.4.5
- [:open](#selectordef-open), in § 11.1
- [:optional](#optional-pseudo), in § 12.3.3
- [originating element](#originating-element), in § 3.6.2
- [originating pseudo-element](#originating-pseudo-element), in § 3.6.4
- [:out-of-range](#out-of-range-pseudo), in § 12.3.2
- [parse a relative selector](#parse-a-relative-selector), in § 17.2
- [parse as a forgiving selector list](#parse-as-a-forgiving-selector-list), in § 16.1
- [parse a selector](#parse-a-selector), in § 17.1
- [:paused](#selectordef-paused), in § 10.1
- [:picture-in-picture](#selectordef-picture-in-picture), in § 11.5
- [:placeholder-shown](#placeholder-shown-pseudo), in § 12.1.3
- [:playing](#selectordef-playing), in § 10.1
- [:popover-open](#selectordef-popover-open), in § 11.2
- [pseudo-class](#pseudo-class), in § 3.5
- [\<pseudo-class-selector\>](#typedef-pseudo-class-selector), in § 16
- [\<pseudo-compound-selector\>](#typedef-pseudo-compound-selector), in § 16
- [pseudo-compound selector](#pseudo-compound), in § 3.1
- [pseudo-elements](#pseudo-element), in § 3.6
- [\<pseudo-element-selector\>](#typedef-pseudo-element-selector), in § 16
- [:read-only](#read-only-pseudo), in § 12.1.2
- [:read-write](#read-write-pseudo), in § 12.1.2
- [relative](#relative-selector), in § 3.4
- [\<relative-real-selector\>](#typedef-relative-real-selector), in § 16
- [\<relative-real-selector-list\>](#typedef-relative-real-selector-list), in § 16
- [\<relative-selector\>](#typedef-relative-selector), in § 16
- [relative selector](#relative-selector), in § 3.4
- [relative selector anchor elements](#relative-selector-anchor-elements), in § 3.4
- [\<relative-selector-list\>](#typedef-relative-selector-list), in § 16
- [:required](#required-pseudo), in § 12.3.3
- [:root](#root-pseudo), in § 13.1
- [s](#valdef-attribute-selector-s), in § 6.3
- [:scope](#scope-pseudo), in § 8.4
- [scope](#scoped-selector), in § 3.3
- [scoped selector](#scoped-selector), in § 3.3
- [scoping root](#scoping-root), in § 3.3
- [:seeking](#selectordef-seeking), in § 10.1
- [selector](#selector), in § 3.1
- [\<selector-list\>](#typedef-selector-list), in § 16
- [selector list](#selector-list), in § 3.1
- [\<simple-selector\>](#typedef-simple-selector), in § 16
- [simple selector](#simple), in § 3.1
- [\<simple-selector-list\>](#typedef-simple-selector-list), in § 16
- [specificity](#specificity), in § 15
- [:stalled](#selectordef-stalled), in § 10.2
- [structural pseudo-classes](#structural-pseudo-classes), in § 13
- [\<subclass-selector\>](#typedef-subclass-selector), in § 16
- [subject](#selector-subject), in § 3.1
- [subject of a selector](#selector-subject), in § 3.1
- [subject of the selector](#selector-subject), in § 3.1
- [sub-pseudo-element](#sub-pseudo-element), in § 3.6.4
- [subsequent-sibling combinator](#subsequent-sibling-combinator), in § 14.4
- [:target](#target-pseudo), in § 8.3
- [\<type-selector\>](#typedef-type-selector), in § 16
- [type selector](#type-selector), in § 5.1
- [ultimate originating element](#ultimate-originating-element), in § 3.6.4
- [:unchecked](#unchecked-pseudo), in § 12.2.1
- [universal selector](#universal-selector), in § 5.2
- [unknown -webkit- pseudo-elements](#unknown--webkit--pseudo-elements), in § Unnumbered section
- [user action pseudo-class](#user-action-pseudo-class), in § 9
- [:user-invalid](#user-invalid-pseudo), in § 12.3.4
- [:user-valid](#user-valid-pseudo), in § 12.3.4
- [:valid](#valid-pseudo), in § 12.3.1
- [:visited](#visited-pseudo), in § 8.2
- [:volume-locked](#selectordef-volume-locked), in § 10.3
- [:where()](#where-pseudo), in § 4.4
- [White space](#whitespace), in § 3.7
- [\<wq-name\>](#typedef-wq-name), in § 16

### <a id="index-defined-elsewhere"></a>Terms defined by reference[](#index-defined-elsewhere)

- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="0e9419b9"></a>display
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="df19338c"></a>box tree
  - <a id="09cd65e4"></a>display type
  - <a id="96626f55"></a>list-item
  - <a id="12f8fb07"></a>visibility
- \[CSS-MULTICOL-2\] defines the following terms:
  - <a id="e515eec3"></a>::column
- \[CSS-POSITION-4\] defines the following terms:
  - <a id="8f59bb58"></a>top layer
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="fd0c9e35"></a>::after
  - <a id="5869f885"></a>::before
  - <a id="a1addf6c"></a>::first-letter
  - <a id="75bfa627"></a>::first-line
  - <a id="5875aeb9"></a>::marker
  - <a id="16a52cab"></a>tree-abiding pseudo-elements
- \[CSS-SHADOW-1\] defines the following terms:
  - <a id="905d9a5a"></a>::slotted()
  - <a id="5561253e"></a>:host
  - <a id="37487143"></a>:host-context()
  - <a id="4cb3439f"></a>flat tree
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="5879b11d"></a>content language
  - <a id="a963f9ba"></a>document white space characters
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="08eba280"></a>!
  - <a id="68487d22"></a>\#
  - <a id="68150e76"></a>\*
  - <a id="9213678e"></a>\<ident\>
  - <a id="977d3003"></a>\<string\>
  - <a id="537cf076"></a>?
  - <a id="9dc6ce48"></a>identifier
  - <a id="6ec67710"></a>\|
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="5bc8bd12"></a>direction
- \[CSS21\] defines the following terms:
  - <a id="74a27486"></a>:before
- \[CSS3NAMESPACE\] defines the following terms:
  - <a id="bf0e8186"></a>&#64;namespace
  - <a id="ee301461"></a>CSS qualified name
  - <a id="de39d615"></a>default namespace
- \[CSS3SYN\] defines the following terms:
  - <a id="680e429b"></a>\<a-n-plus-b\>
  - <a id="8716d04f"></a>\<any-value\>
  - <a id="af50aa62"></a>\<function-token\>
  - <a id="0190963b"></a>\<hash-token\>
  - <a id="07308da5"></a>\<ident-token\>
  - <a id="6158d44a"></a>\<string-token\>
  - <a id="76c4403d"></a>parse
  - <a id="ca0abe12"></a>parse a list
- \[CSSOM-1\] defines the following terms:
  - <a id="b203675a"></a>getComputedStyle(elt)
- \[DOM\] defines the following terms:
  - <a id="49a2843d"></a>DocumentFragment
  - <a id="da8b8e0e"></a>descendant
  - <a id="2f0ba72c"></a>document element
  - <a id="8bc6d766"></a>inclusive sibling
  - <a id="6f0f52dc"></a>parentNode
  - <a id="fd11cdcd"></a>quirks mode
  - <a id="f7960529"></a>root
  - <a id="58a8f724"></a>shadow host
  - <a id="19f9e9df"></a>shadow tree
  - <a id="fefa5851"></a>shadow-including tree order
  - <a id="e6cc3311"></a>tree
- \[FULLSCREEN\] defines the following terms:
  - <a id="bcdb6841"></a>requestFullscreen()
- \[HTML\] defines the following terms:
  - <a id="d056461c"></a>:user-invalid
  - <a id="68bf3d31"></a>a
  - <a id="a1a807e5"></a>area
  - <a id="2c82d279"></a>audio
  - <a id="bfff6250"></a>button
  - <a id="dfc2dd10"></a>checked
  - <a id="b6fb7a51"></a>custom element
  - <a id="1748bbeb"></a>details
  - <a id="1812300f"></a>dialog
  - <a id="a7bbaf9f"></a>div
  - <a id="28c097f8"></a>effective media volume
  - <a id="da60253e"></a>element definition
  - <a id="7992f81d"></a>em
  - <a id="7a348049"></a>h1
  - <a id="ca0c892d"></a>href
  - <a id="c3dd181e"></a>html
  - <a id="f0811ff8"></a>img
  - <a id="24c3d56b"></a>indeterminate
  - <a id="d7d642a2"></a>input
  - <a id="84f243ef"></a>label
  - <a id="7486f2c8"></a>li
  - <a id="0027e642"></a>media data
  - <a id="b2198f14"></a>media element stall timeout
  - <a id="64b89595"></a>meta
  - <a id="655fdd86"></a>muted
  - <a id="99b5cef6"></a>object
  - <a id="f29863d8"></a>ol
  - <a id="aee00462"></a>option
  - <a id="086e3aff"></a>origin
  - <a id="b80d76a2"></a>p
  - <a id="b9a05383"></a>placeholder
  - <a id="f4da6587"></a>popover
  - <a id="83b82cf5"></a>popover visibility state
  - <a id="22a0e026"></a>pre
  - <a id="b90b2ad5"></a>progress
  - <a id="9dc3e09f"></a>q
  - <a id="85188fb3"></a>select
  - <a id="cd8eeff2"></a>selected
  - <a id="9d861566"></a>showModal()
  - <a id="fb9bb722"></a>site
  - <a id="b32b852a"></a>span
  - <a id="fc736137"></a>textarea
  - <a id="aa7bbf63"></a>video
- \[INFRA\] defines the following terms:
  - <a id="7f9469b5"></a>ASCII case-insensitive
  - <a id="b6f2095c"></a>identical to
  - <a id="91c016b5"></a>identical to (for string)
  - <a id="15e48c39"></a>set
  - <a id="0e8de730"></a>tuple
- \[MATHML-CORE\] defines the following terms:
  - <a id="5f4f7d1c"></a>math
- \[SELECT\] defines the following terms:
  - <a id="53a193f1"></a>\*
- \[SELECTORS-5\] defines the following terms:
  - <a id="1bc79eaa"></a>:blank
  - <a id="ec885bc1"></a>:interest-source
  - <a id="9f5e7e1d"></a>:interest-target
  - <a id="f55ffdf2"></a>:local-link
  - <a id="ec34ede6"></a>:nth-col()
  - <a id="68ded3db"></a>:nth-last-col()
  - <a id="343e92e5"></a>column combinator
- \[URL\] defines the following terms:
  - <a id="ed948033"></a>fragment
  - <a id="dcffbccd"></a>URL

## <a id="references"></a>References[](#references)

### <a id="normative"></a>Normative References[](#normative)

<a id="biblio-css-display-4"></a><strong>\[CSS-DISPLAY-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://drafts.csswg.org/css-display-4/). URL: <https://drafts.csswg.org/css-display-4/>

<a id="biblio-css-position-4"></a><strong>\[CSS-POSITION-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 4](https://drafts.csswg.org/css-position-4/). URL: <https://drafts.csswg.org/css-position-4/>

<a id="biblio-css-pseudo-4"></a><strong>\[CSS-PSEUDO-4\]</strong>

Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://drafts.csswg.org/css-pseudo-4/). URL: <https://drafts.csswg.org/css-pseudo-4/>

<a id="biblio-css-shadow-1"></a><strong>\[CSS-SHADOW-1\]</strong>

[CSS Shadow Module Level 1](https://drafts.csswg.org/css-shadow-1/). Editor's Draft. URL: <https://drafts.csswg.org/css-shadow-1/>

<a id="biblio-css-text-4"></a><strong>\[CSS-TEXT-4\]</strong>

Elika Etemad; et al. [CSS Text Module Level 4](https://drafts.csswg.org/css-text-4/). URL: <https://drafts.csswg.org/css-text-4/>

<a id="biblio-css-values-4"></a><strong>\[CSS-VALUES-4\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: <https://drafts.csswg.org/css-values-4/>

<a id="biblio-css-writing-modes-3"></a><strong>\[CSS-WRITING-MODES-3\]</strong>

Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://drafts.csswg.org/css-writing-modes-3/). URL: <https://drafts.csswg.org/css-writing-modes-3/>

<a id="biblio-css21"></a><strong>\[CSS21\]</strong>

Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://drafts.csswg.org/css2/). URL: <https://drafts.csswg.org/css2/>

<a id="biblio-css3namespace"></a><strong>\[CSS3NAMESPACE\]</strong>

Elika Etemad. [CSS Namespaces Module Level 3](https://drafts.csswg.org/css-namespaces/). URL: <https://drafts.csswg.org/css-namespaces/>

<a id="biblio-css3syn"></a><strong>\[CSS3SYN\]</strong>

Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/). URL: <https://drafts.csswg.org/css-syntax/>

<a id="biblio-dom"></a><strong>\[DOM\]</strong>

Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: <https://dom.spec.whatwg.org/>

<a id="biblio-html"></a><strong>\[HTML\]</strong>

Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: <https://html.spec.whatwg.org/multipage/>

<a id="biblio-infra"></a><strong>\[INFRA\]</strong>

Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: <https://infra.spec.whatwg.org/>

<a id="biblio-rfc2119"></a><strong>\[RFC2119\]</strong>

S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: <https://datatracker.ietf.org/doc/html/rfc2119>

<a id="biblio-rfc4647"></a><strong>\[RFC4647\]</strong>

A. Phillips, Ed.; M. Davis, Ed.. [Matching of Language Tags](https://www.rfc-editor.org/info/rfc4647/). September 2006. Best Current Practice. URL: <https://www.rfc-editor.org/info/rfc4647/>

<a id="biblio-rfc5646"></a><strong>\[RFC5646\]</strong>

A. Phillips, Ed.; M. Davis, Ed.. [Tags for Identifying Languages](https://www.rfc-editor.org/info/rfc5646/). September 2009. Best Current Practice. URL: <https://www.rfc-editor.org/info/rfc5646/>

<a id="biblio-select"></a><strong>\[SELECT\]</strong>

Tantek Çelik; et al. [Selectors Level 3](https://drafts.csswg.org/selectors-3/). URL: <https://drafts.csswg.org/selectors-3/>

<a id="biblio-url"></a><strong>\[URL\]</strong>

Anne van Kesteren. [URL Standard](https://url.spec.whatwg.org/). Living Standard. URL: <https://url.spec.whatwg.org/>

### <a id="informative"></a>Non-Normative References[](#informative)

<a id="biblio-bcp47"></a><strong>\[BCP47\]</strong>

Addison Phillips; Mark Davis. [Tags for Identifying Languages](https://www.rfc-editor.org/info/bcp47). September 2009. Best Current Practice. URL: <https://www.rfc-editor.org/info/bcp47>

<a id="biblio-css-display-3"></a><strong>\[CSS-DISPLAY-3\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://drafts.csswg.org/css-display/). URL: <https://drafts.csswg.org/css-display/>

<a id="biblio-css-multicol-2"></a><strong>\[CSS-MULTICOL-2\]</strong>

Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 2](https://drafts.csswg.org/css-multicol-2/). URL: <https://drafts.csswg.org/css-multicol-2/>

<a id="biblio-css3ui"></a><strong>\[CSS3UI\]</strong>

Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://drafts.csswg.org/css-ui-3/). URL: <https://drafts.csswg.org/css-ui-3/>

<a id="biblio-cssom-1"></a><strong>\[CSSOM-1\]</strong>

Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://drafts.csswg.org/cssom/). URL: <https://drafts.csswg.org/cssom/>

<a id="biblio-cssstyleattr"></a><strong>\[CSSSTYLEATTR\]</strong>

Tantek Çelik; Elika Etemad. [CSS Style Attributes](https://drafts.csswg.org/css-style-attr/). URL: <https://drafts.csswg.org/css-style-attr/>

<a id="biblio-fullscreen"></a><strong>\[FULLSCREEN\]</strong>

Philip Jägenstedt. [Fullscreen API Standard](https://fullscreen.spec.whatwg.org/). Living Standard. URL: <https://fullscreen.spec.whatwg.org/>

<a id="biblio-its20"></a><strong>\[ITS20\]</strong>

David Filip; et al. [Internationalization Tag Set (ITS) Version 2.0](https://www.w3.org/TR/its20/). 29 October 2013. REC. URL: <https://www.w3.org/TR/its20/>

<a id="biblio-mathml"></a><strong>\[MATHML\]</strong>

Patrick D F Ion; Robert R Miner. [Mathematical Markup Language (MathML™) 1.01 Specification](https://www.w3.org/TR/REC-MathML/). 7 March 2023. REC. URL: <https://www.w3.org/TR/REC-MathML/>

<a id="biblio-mathml-core"></a><strong>\[MATHML-CORE\]</strong>

David Carlisle; Frédéric Wang. [MathML Core](https://w3c.github.io/mathml-core/). URL: <https://w3c.github.io/mathml-core/>

<a id="biblio-picture-in-picture"></a><strong>\[PICTURE-IN-PICTURE\]</strong>

Francois Beaufort; Simon Farre. [Picture-in-Picture](https://w3c.github.io/picture-in-picture/). URL: <https://w3c.github.io/picture-in-picture/>

<a id="biblio-quirks"></a><strong>\[QUIRKS\]</strong>

Simon Pieters. [Quirks Mode Standard](https://quirks.spec.whatwg.org/). Living Standard. URL: <https://quirks.spec.whatwg.org/>

<a id="biblio-selectors-5"></a><strong>\[SELECTORS-5\]</strong>

Elika Etemad; Tab Atkins Jr.. [Selectors Level 5](https://drafts.csswg.org/selectors-5/). URL: <https://drafts.csswg.org/selectors-5/>

<a id="biblio-svg11"></a><strong>\[SVG11\]</strong>

Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: <https://www.w3.org/TR/SVG11/>

<a id="biblio-xforms11"></a><strong>\[XFORMS11\]</strong>

John Boyer. [XForms 1.1](https://www.w3.org/TR/xforms11/). 20 October 2009. REC. URL: <https://www.w3.org/TR/xforms11/>

<a id="biblio-xml-names"></a><strong>\[XML-NAMES\]</strong>

Tim Bray; et al. [Namespaces in XML 1.0 (Third Edition)](https://www.w3.org/TR/xml-names/). 8 December 2009. REC. URL: <https://www.w3.org/TR/xml-names/>

<a id="biblio-xml10"></a><strong>\[XML10\]</strong>

Tim Bray; et al. [Extensible Markup Language (XML) 1.0 (Fifth Edition)](https://www.w3.org/TR/xml/). 26 November 2008. REC. URL: <https://www.w3.org/TR/xml/>

## <a id="issues-index"></a>Issues Index[](#issues-index)

Add comma-separated syntax for

<strong>Issue:</strong>

[multiple-value matching](http://lists.w3.org/Archives/Public/www-style/2011Mar/0215.html)? e.g. \[rel ~= next, prev, up, first, last\] [↵](#issue-745ef775)

There’s a desire from authors to propagate

<strong>Issue:</strong>

[:focus](#focus-pseudo) from a form control to its associated <code>[label](https://html&#46;spec&#46;whatwg&#46;org/multipage/forms&#46;html#the-label-element)</code> element; the main objection seems to be implementation difficulty. See [CSSWG issue (CSS)](https://github.com/w3c/csswg-drafts/issues/397) and [WHATWG issue (HTML)](https://github.com/whatwg/html/issues/1632). [↵](#issue-1416193e)

Are these still necessary now that we have more rigorous definitions for

<strong>Issue:</strong>

[match](#match) and [invalid selector](#invalid-selector)? Nouns are a lot easier to coordinate across specification than predicates, and details like the exact order of elements returned from <code>querySelector</code> seem to make more sense being defined in the DOM specification than in Selectors. [↵](#issue-55d7bd68)

Only the

<strong>Issue:</strong>

[tree-abiding pseudo-elements](https://drafts.csswg.org/css-pseudo-4/#tree-abiding) are really handled in any way remotely like this. [↵](#issue-15f3317f)

The relative position of pseudo-elements in

<strong>Issue:</strong>

<var>selector match list</var> is undefined. There’s not yet a context that exposes this information, but we need to decide on something eventually, before something <em>is</em> exposed. [↵](#issue-b7f52036)

Some pseudo-classes are \*syntactical\*, like

<strong>Issue:</strong>

[:has()](#has-pseudo) and [:is()](#matches-pseudo), and thus should always work. Need to indicate that somewhere. Probably the structural pseudos always work whenever the child list is ordered. [↵](#issue-772ec278)

Semantic definition should probably move back here.

<strong>Issue:</strong>

[↵](#issue-fb60ae39)

Need to define tree for XML.

<strong>Issue:</strong>

[↵](#issue-4ca3978c)

<strong>✔</strong>MDN

[:active](https://developer.mozilla.org/en-US/docs/Web/CSS/:active)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE4+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[Adjacent_sibling_combinator](https://developer.mozilla.org/en-US/docs/Web/CSS/Adjacent_sibling_combinator)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE7+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:any-link](https://developer.mozilla.org/en-US/docs/Web/CSS/:any-link)

In all current engines.

Firefox50+Safari9+Chrome65+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari9+Chrome for Android?Android WebView65+Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[Attribute_selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/Attribute_selectors)

In all current engines.

Firefox1+Safari3+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE7+

------------------------------------------------------------------------

Firefox for Android?iOS Safari1+Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[:blank](https://developer.mozilla.org/en-US/docs/Web/CSS/:blank)

In no current engines.

FirefoxNoneSafariNoneChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:checked](https://developer.mozilla.org/en-US/docs/Web/CSS/:checked)

In all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android4+iOS Safari?Chrome for Android18+Android WebView2+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[Child_combinator](https://developer.mozilla.org/en-US/docs/Web/CSS/Child_combinator)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera4+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE7+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[Class_selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/Class_selectors)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE3+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>⚠</strong>MDN

[Column_combinator](https://developer.mozilla.org/en-US/docs/Web/CSS/Column_combinator)

In no current engines.

FirefoxNoneSafariNoneChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:default](https://developer.mozilla.org/en-US/docs/Web/CSS/:default)

In all current engines.

Firefox4+Safari5+Chrome10+

------------------------------------------------------------------------

Opera10+Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari5+Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[Descendant_combinator](https://developer.mozilla.org/en-US/docs/Web/CSS/Descendant_combinator)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE3+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile10.1+

MDN

[:dir](https://developer.mozilla.org/en-US/docs/Web/CSS/:dir)

Firefox49+Safari16.4+ChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:disabled](https://developer.mozilla.org/en-US/docs/Web/CSS/:disabled)

In all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView2+Samsung Internet?Opera Mobile10.1+

[:enabled](https://developer.mozilla.org/en-US/docs/Web/CSS/:enabled)

In all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView2+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:empty](https://developer.mozilla.org/en-US/docs/Web/CSS/:empty)

In all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView2+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:first-child](https://developer.mozilla.org/en-US/docs/Web/CSS/:first-child)

In all current engines.

Firefox3+Safari3.1+Chrome4+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE7+

------------------------------------------------------------------------

Firefox for Android?iOS Safari4+Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:first-of-type](https://developer.mozilla.org/en-US/docs/Web/CSS/:first-of-type)

In all current engines.

Firefox3.5+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView2+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:focus-visible](https://developer.mozilla.org/en-US/docs/Web/CSS/:focus-visible)

In all current engines.

Firefox85+Safari15.4+Chrome86+

------------------------------------------------------------------------

Opera?Edge86+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:focus-within](https://developer.mozilla.org/en-US/docs/Web/CSS/:focus-within)

In all current engines.

Firefox52+Safari10.1+Chrome60+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:focus](https://developer.mozilla.org/en-US/docs/Web/CSS/:focus)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera7+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE8+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile10.1+

<strong>⚠</strong>MDN

[:future](https://developer.mozilla.org/en-US/docs/Web/CSS/:future)

In only one current engine.

FirefoxNoneSafari7+ChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[General_sibling_combinator](https://developer.mozilla.org/en-US/docs/Web/CSS/General_sibling_combinator)

In all current engines.

Firefox1+Safari3+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE7+

------------------------------------------------------------------------

Firefox for Android?iOS Safari1+Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:has](https://developer.mozilla.org/en-US/docs/Web/CSS/:has)

In all current engines.

Firefox🔰 103+Safari15.4+Chrome105+

------------------------------------------------------------------------

Opera?Edge105+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:hover](https://developer.mozilla.org/en-US/docs/Web/CSS/:hover)

In all current engines.

Firefox1+Safari2+Chrome1+

------------------------------------------------------------------------

Opera4+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE4+

------------------------------------------------------------------------

Firefox for Android?iOS Safari1+Chrome for Android?Android WebView?Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[ID_selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/ID_selectors)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE3+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:in-range](https://developer.mozilla.org/en-US/docs/Web/CSS/:in-range)

In all current engines.

Firefox29+Safari5.1+Chrome10+

------------------------------------------------------------------------

Opera11+Edge79+

------------------------------------------------------------------------

Edge (Legacy)13+IENone

------------------------------------------------------------------------

Firefox for Android16+iOS Safari?Chrome for Android?Android WebView2.2+Samsung Internet1.0+Opera Mobile11+

<strong>✔</strong>MDN

[:indeterminate](https://developer.mozilla.org/en-US/docs/Web/CSS/:indeterminate)

In all current engines.

Firefox2+Safari3+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE10+

------------------------------------------------------------------------

Firefox for Android?iOS Safari1+Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:invalid](https://developer.mozilla.org/en-US/docs/Web/CSS/:invalid)

In all current engines.

Firefox4+Safari5+Chrome10+

------------------------------------------------------------------------

Opera10+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE10+

------------------------------------------------------------------------

Firefox for Android?iOS Safari5+Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

[:valid](https://developer.mozilla.org/en-US/docs/Web/CSS/:valid)

In all current engines.

Firefox4+Safari5+Chrome10+

------------------------------------------------------------------------

Opera10+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE10+

------------------------------------------------------------------------

Firefox for Android?iOS Safari5+Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:is](https://developer.mozilla.org/en-US/docs/Web/CSS/:is)

In all current engines.

Firefox78+Safari14+Chrome88+

------------------------------------------------------------------------

Opera?Edge88+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari14+Chrome for Android?Android WebView88+Samsung Internet15.0+Opera Mobile?

<strong>✔</strong>MDN

[:lang](https://developer.mozilla.org/en-US/docs/Web/CSS/:lang)

In all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera8+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE8+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:last-child](https://developer.mozilla.org/en-US/docs/Web/CSS/:last-child)

In all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:last-of-type](https://developer.mozilla.org/en-US/docs/Web/CSS/:last-of-type)

In all current engines.

Firefox3.5+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView2+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:link](https://developer.mozilla.org/en-US/docs/Web/CSS/:link)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE3+

------------------------------------------------------------------------

Firefox for Android?iOS Safari3.2+Chrome for Android?Android WebView1.5+Samsung Internet?Opera Mobile?

[:visited](https://developer.mozilla.org/en-US/docs/Web/CSS/:visited)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE4+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[Selector_list](https://developer.mozilla.org/en-US/docs/Web/CSS/Selector_list)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE3+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:modal](https://developer.mozilla.org/en-US/docs/Web/CSS/:modal)

In all current engines.

Firefox103+Safari15.6+Chrome105+

------------------------------------------------------------------------

Opera?Edge105+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:not](https://developer.mozilla.org/en-US/docs/Web/CSS/:not)

In all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView2+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:nth-child](https://developer.mozilla.org/en-US/docs/Web/CSS/:nth-child)

In all current engines.

Firefox3.5+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:nth-last-child](https://developer.mozilla.org/en-US/docs/Web/CSS/:nth-last-child)

In all current engines.

Firefox3.5+Safari3.1+Chrome4+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:nth-last-of-type](https://developer.mozilla.org/en-US/docs/Web/CSS/:nth-last-of-type)

In all current engines.

Firefox3.5+Safari3.1+Chrome4+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView2+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:nth-of-type](https://developer.mozilla.org/en-US/docs/Web/CSS/:nth-of-type)

In all current engines.

Firefox3.5+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView2+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:only-child](https://developer.mozilla.org/en-US/docs/Web/CSS/:only-child)

In all current engines.

Firefox1.5+Safari3.1+Chrome2+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:only-of-type](https://developer.mozilla.org/en-US/docs/Web/CSS/:only-of-type)

In all current engines.

Firefox3.5+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView2+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:optional](https://developer.mozilla.org/en-US/docs/Web/CSS/:optional)

In all current engines.

Firefox4+Safari5+Chrome10+

------------------------------------------------------------------------

Opera10+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE10+

------------------------------------------------------------------------

Firefox for Android?iOS Safari5+Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

[:required](https://developer.mozilla.org/en-US/docs/Web/CSS/:required)

In all current engines.

Firefox4+Safari5+Chrome10+

------------------------------------------------------------------------

Opera10+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE10+

------------------------------------------------------------------------

Firefox for Android?iOS Safari5+Chrome for Android?Android WebView4.4.3+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:out-of-range](https://developer.mozilla.org/en-US/docs/Web/CSS/:out-of-range)

In all current engines.

Firefox29+Safari5.1+Chrome10+

------------------------------------------------------------------------

Opera11+Edge79+

------------------------------------------------------------------------

Edge (Legacy)13+IENone

------------------------------------------------------------------------

Firefox for Android16+iOS Safari?Chrome for Android?Android WebView2.2+Samsung Internet?Opera Mobile11+

<strong>⚠</strong>MDN

[:past](https://developer.mozilla.org/en-US/docs/Web/CSS/:past)

In only one current engine.

FirefoxNoneSafari7+ChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[:paused](https://developer.mozilla.org/en-US/docs/Web/CSS/:paused)

In only one current engine.

FirefoxNoneSafari15.4+ChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[:picture-in-picture](https://developer.mozilla.org/en-US/docs/Web/CSS/:picture-in-picture)

In only one current engine.

FirefoxNoneSafariNoneChrome110+

------------------------------------------------------------------------

Opera?Edge110+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:placeholder-shown](https://developer.mozilla.org/en-US/docs/Web/CSS/:placeholder-shown)

In all current engines.

Firefox51+Safari9+Chrome47+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[:playing](https://developer.mozilla.org/en-US/docs/Web/CSS/:playing)

In only one current engine.

FirefoxNoneSafari15.4+ChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:read-only](https://developer.mozilla.org/en-US/docs/Web/CSS/:read-only)

In all current engines.

Firefox78+Safari4+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)13+IENone

------------------------------------------------------------------------

Firefox for AndroidNoneiOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

[:read-write](https://developer.mozilla.org/en-US/docs/Web/CSS/:read-write)

In all current engines.

Firefox78+Safari4+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)13+IENone

------------------------------------------------------------------------

Firefox for AndroidNoneiOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[:root](https://developer.mozilla.org/en-US/docs/Web/CSS/:root)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:scope](https://developer.mozilla.org/en-US/docs/Web/CSS/:scope)

In all current engines.

Firefox32+Safari7+Chrome27+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[:target-within](https://developer.mozilla.org/en-US/docs/Web/CSS/:target-within)

In no current engines.

FirefoxNoneSafariNoneChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:target](https://developer.mozilla.org/en-US/docs/Web/CSS/:target)

In all current engines.

Firefox1+Safari1.3+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE9+

------------------------------------------------------------------------

Firefox for Android?iOS Safari2+Chrome for Android?Android WebView2+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[Type_selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/Type_selectors)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE3+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

<strong>✔</strong>MDN

[Universal_selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/Universal_selectors)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE7+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+

MDN

[:user-invalid](https://developer.mozilla.org/en-US/docs/Web/CSS/:user-invalid)

Firefox88+Safari16.5+ChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[:user-valid](https://developer.mozilla.org/en-US/docs/Web/CSS/:user-valid)

Firefox88+Safari16.5+ChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[:where](https://developer.mozilla.org/en-US/docs/Web/CSS/:where)

In all current engines.

Firefox78+Safari14+Chrome88+

------------------------------------------------------------------------

Opera?Edge88+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

CanIUse

<strong>Support:</strong>Android Browser154+Baidu Browser13.52+Blackberry BrowserNoneChrome88+Chrome for Android154+Edge88+Firefox78+Firefox for Android157+IENoneIE MobileNoneKaiOS Browser3.0+Opera75+Opera MiniNoneOpera Mobile80+QQ BrowserNoneSafari14+Safari on iOS14.0+Samsung Internet15.0+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-matches-pseudo) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu Browser13.52+Blackberry BrowserNoneChrome88+Chrome for Android154+Edge88+Firefox84+Firefox for Android157+IENoneIE MobileNoneKaiOS Browser3.0+Opera75+Opera MiniNoneOpera Mobile80+QQ BrowserNoneSafari9+Safari on iOS9.0+Samsung Internet15.0+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-not-sel-list) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu BrowserNoneBlackberry BrowserNoneChrome105+Chrome for Android154+Edge105+Firefox121+Firefox for Android157+IENoneIE MobileNoneKaiOS BrowserNoneOpera91+Opera MiniNoneOpera Mobile80+QQ BrowserNoneSafari15.4+Safari on iOS15.4+Samsung Internet20+UC Browser for AndroidNone

Source: [caniuse.com](https://caniuse.com/#feat=css-has) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu Browser13.52+Blackberry BrowserNoneChrome49+Chrome for Android154+Edge79+Firefox47+Firefox for Android157+IENoneIE MobileNoneKaiOS Browser2.5+Opera36+Opera MiniNoneOpera Mobile80+QQ Browser14.9+Safari9+Safari on iOS9.0+Samsung Internet5.0+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-case-insensitive) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu BrowserNoneBlackberry BrowserNoneChrome120+Chrome for Android154+Edge120+Firefox49+Firefox for Android157+IENoneIE MobileNoneKaiOS Browser3.0+Opera106+Opera MiniNoneOpera Mobile80+QQ BrowserNoneSafari16.4+Safari on iOS16.4+Samsung Internet25+UC Browser for AndroidNone

Source: [caniuse.com](https://caniuse.com/#feat=css-dir-pseudo) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu Browser13.52+Chrome65+Chrome for Android154+Edge79+Firefox50+Firefox for Android157+IENoneIE MobileNoneKaiOS Browser3.0+Opera52+Opera MiniNoneOpera Mobile80+QQ Browser14.9+Safari9+Safari on iOS9.0+Samsung Internet9.2+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-any-link) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu Browser13.52+Blackberry BrowserNoneChrome60+Chrome for Android154+Edge79+Firefox52+Firefox for Android157+IENoneIE MobileNoneKaiOS Browser3.0+Opera47+Opera MiniNoneOpera Mobile80+QQ Browser14.9+Safari10.1+Safari on iOS10.3+Samsung Internet8.2+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-focus-within) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu Browser13.52+Blackberry BrowserNoneChrome47+Chrome for Android154+Edge79+Firefox51+Firefox for Android157+IENoneIE MobileNoneKaiOS Browser3.0+Opera34+Opera MiniNoneOpera Mobile80+QQ Browser14.9+Safari9+Safari on iOS9.0+Samsung Internet5.0+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-placeholder-shown) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu Browser13.52+Blackberry Browser (limited)10+Chrome51+Chrome for Android154+Edge79+Firefox4+Firefox for Android157+IENoneIE MobileNoneKaiOS Browser2.5+Opera38+Opera Mini (limited)AllOpera Mobile80+QQ Browser14.9+Safari10.1+Safari on iOS10.3+Samsung Internet5.0+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-default-pseudo) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu Browser13.52+Blackberry Browser (limited)10+Chrome39+Chrome for Android154+Edge79+Firefox51+Firefox for Android157+IE (limited)9+IE Mobile (limited)10+KaiOS Browser3.0+Opera26+Opera Mini (limited)AllOpera Mobile80+QQ Browser14.9+Safari10.1+Safari on iOS10.3+Samsung Internet4+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-indeterminate-pseudo) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu Browser13.52+Blackberry Browser (limited)10+Chrome53+Chrome for Android154+Edge79+Firefox50+Firefox for Android157+IENoneIE MobileNoneKaiOS Browser3.0+Opera40+Opera Mini (limited)AllOpera Mobile80+QQ Browser14.9+Safari10.1+Safari on iOS10.3+Samsung Internet5.0+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-in-out-of-range) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser2.3+Baidu Browser13.52+Blackberry Browser7+Chrome15+Chrome for Android154+Edge12+Firefox4+Firefox for Android157+IE10+IE Mobile10+KaiOS Browser2.5+Opera15+Opera Mini (limited)AllOpera Mobile80+QQ Browser14.9+Safari5+Safari on iOS5.0+Samsung Internet4+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-optional-pseudo) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android Browser154+Baidu BrowserNoneBlackberry BrowserNoneChrome111+Chrome for Android154+Edge111+Firefox113+Firefox for Android157+IENoneIE MobileNoneKaiOS BrowserNoneOpera98+Opera MiniNoneOpera Mobile80+QQ BrowserNoneSafari9+Safari on iOS9.0+Samsung Internet22+UC Browser for AndroidNone

Source: [caniuse.com](https://caniuse.com/#feat=css-nth-child-of) as of 2026-10-06

CanIUse

<strong>Support:</strong>Android BrowserNoneBaidu BrowserNoneBlackberry BrowserNoneChromeNoneChrome for AndroidNoneEdgeNoneFirefoxNoneFirefox for AndroidNoneIENoneIE MobileNoneKaiOS BrowserNoneOperaNoneOpera MiniNoneOpera MobileNoneQQ BrowserNoneSafariNoneSafari on iOSNoneSamsung InternetNoneUC Browser for AndroidNone

Source: [caniuse.com](https://caniuse.com/#feat=css-descendant-gtgt) as of 2026-10-06
