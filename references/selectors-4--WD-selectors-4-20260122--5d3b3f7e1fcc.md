Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Selectors Level 4](https://www.w3.org/TR/2026/WD-selectors-4-20260122/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Selectors Level 4

Source snapshot: https://www.w3.org/TR/2026/WD-selectors-4-20260122/

Snapshot SHA-256: 5d3b3f7e1fcc562dcec1a01ad2c5651e9f0bac294b255a75352e9df202604e6d

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 1 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>Selectors Level 4

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-selector"></a>

[Selectors](#selector) are patterns that match against elements in a tree, and as such form one of several technologies that can be used to select nodes in a document. Selectors have been optimized for use with HTML and XML, and are designed to be usable in performance-critical code. They are a core component of CSS (Cascading Style Sheets), which uses Selectors to bind style properties to elements in the document. Selectors Level 4 describes the selectors that already exist in [\[SELECT\]](#biblio-select), and further introduces new selectors for CSS and other languages that may need them.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “selectors” in the title, like this: “\[selectors\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bselectors%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- the column combinator

- <a id="ref-for-tree-abiding"></a>

  <a id="ref-for-user-action-pseudo-class"></a>

  [user action pseudo-classes](#user-action-pseudo-class) applying to non-[tree-abiding pseudo-elements](https://www.w3.org/TR/css-pseudo-4/#tree-abiding)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="context"></a>1.  Introduction

<em>This section is not normative.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

<a id="ref-for-selector①"></a>

A [selector](#selector) is a boolean predicate that takes an element in a tree structure and tests whether the element matches the selector or not.

These expressions may be used for many things:

- directly on an element to test whether it matches some criteria, such as in the `element.matches()` function defined in [\[DOM\]](#biblio-dom)
- applied to an entire tree of elements to filter it into a set of elements that match the criteria, such as in the `document.querySelectorAll()` function defined in [\[DOM\]](#biblio-dom) or the selector of a CSS style rule.
- used "in reverse" to generate markup that would match a given selector, such as in [HAML](http://haml.info/) or [Emmet](https://en.wikipedia.org/wiki/Emmet_(software)).

Selectors Levels 1, 2, and 3 are defined as the subsets of selector functionality defined in the [CSS1](https://www.w3.org/TR/REC-CSS1), [CSS2.1](https://www.w3.org/TR/CSS21/), and [Selectors Level 3](https://www.w3.org/TR/css3-selectors/) specifications, respectively. This module defines Selectors Level 4.

### <a id="placement"></a>1.1. Module Interactions

Tests

Tests not needed for this section.

------------------------------------------------------------------------

This module replaces the definitions of and extends the set of selectors defined for CSS in [\[SELECT\]](#biblio-select) and [\[CSS21\]](#biblio-css21).

Pseudo-element selectors, which define abstract elements in a rendering tree, are not part of this specification: their generic syntax is described here, but, due to their close integration with the rendering model and irrelevance to other uses such as DOM queries, they will be defined in other modules.

## <a id="overview"></a>2.  Selectors Overview

<em>This section is non-normative, as it merely summarizes the
	following sections.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

A selector represents a structure. This structure can be used as a condition (e.g. in a CSS rule) that determines which elements a selector matches in the document tree, or as a flat description of the HTML or XML fragment corresponding to that structure.

Selectors may range from simple element names to rich contextual representations.

The following table summarizes the Selector syntax:

<a id="selector-examples"></a>

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Pattern

<strong>Column 2 (header cell):</strong>

Represents

<strong>Column 3 (header cell):</strong>

Section

<strong>Column 4 (header cell):</strong>

Level

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

`*`

<strong>Column 2 (data cell):</strong>

any element

<strong>Column 3 (data cell):</strong>

[§ 5.2 Universal selector](#the-universal-selector)

<strong>Column 4 (data cell):</strong>

2

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

`E`

<strong>Column 2 (data cell):</strong>

an element of type E

<strong>Column 3 (data cell):</strong>

[§ 5.1 Type (tag name) selector](#type-selectors)

<strong>Column 4 (data cell):</strong>

1

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

<code>E:not(<var>s1</var>, <var>s2</var>, …)</code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-compound"></a>

an E element that does not match either [compound selector](#compound) <var>s1</var> or <a id="ref-for-compound①"></a>compound selector <var>s2</var>

<strong>Column 3 (data cell):</strong>

[§ 4.3 The Negation (Matches-None) Pseudo-class: :not()](#negation)

<strong>Column 4 (data cell):</strong>

3/4

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

<code>E:is(<var>s1</var>, <var>s2</var>, …)</code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-compound②"></a>

an E element that matches [compound selector](#compound) <var>s1</var> and/or <a id="ref-for-compound③"></a>compound selector <var>s2</var>

<strong>Column 3 (data cell):</strong>

[§ 4.2 The Matches-Any Pseudo-class: :is()](#matches)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 6</strong>

<strong>Column 1 (data cell):</strong>

<code>E:where(<var>s1</var>, <var>s2</var>, …)</code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-compound④"></a>

an E element that matches [compound selector](#compound) <var>s1</var> and/or <a id="ref-for-compound⑤"></a>compound selector <var>s2</var> but contributes no specificity.

<strong>Column 3 (data cell):</strong>

[§ 4.4 The Specificity-adjustment Pseudo-class: :where()](#zero-matches)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 7</strong>

<strong>Column 1 (data cell):</strong>

<code>E:has(<var>rs1</var>, <var>rs2</var>, …)</code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-relative-selector-anchor-elements"></a>

<a id="ref-for-relative-selector"></a>

an E element, if there exists an element that matches either of the [relative selectors](#relative-selector) <var>rs1</var> or <var>rs2</var>, when evaluated with E as the [anchor elements](#relative-selector-anchor-elements)

<strong>Column 3 (data cell):</strong>

[§ 4.5 The Relational Pseudo-class: :has()](#relational)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 8</strong>

<strong>Column 1 (data cell):</strong>

`E.warning`

<strong>Column 2 (data cell):</strong>

an E element belonging to the class `warning` (the document language specifies how class is determined).

<strong>Column 3 (data cell):</strong>

[§ 6.6 Class selectors](#class-html)

<strong>Column 4 (data cell):</strong>

1

<strong>Row 9</strong>

<strong>Column 1 (data cell):</strong>

`E#myid`

<strong>Column 2 (data cell):</strong>

an E element with ID equal to `myid`.

<strong>Column 3 (data cell):</strong>

[§ 6.7 ID selectors](#id-selectors)

<strong>Column 4 (data cell):</strong>

1

<strong>Row 10</strong>

<strong>Column 1 (data cell):</strong>

`E[foo]`

<strong>Column 2 (data cell):</strong>

an E element with a `foo` attribute

<strong>Column 3 (data cell):</strong>

[§ 6.1 Attribute presence and value selectors](#attribute-representation)

<strong>Column 4 (data cell):</strong>

2

<strong>Row 11</strong>

<strong>Column 1 (data cell):</strong>

`E[foo="bar"]`

<strong>Column 2 (data cell):</strong>

an E element whose `foo` attribute value is exactly equal to `bar`

<strong>Column 3 (data cell):</strong>

[§ 6.1 Attribute presence and value selectors](#attribute-representation)

<strong>Column 4 (data cell):</strong>

2

<strong>Row 12</strong>

<strong>Column 1 (data cell):</strong>

`E[foo="bar" i]`

<strong>Column 2 (data cell):</strong>

an E element whose `foo` attribute value is exactly equal to any (ASCII-range) case-permutation of `bar`

<strong>Column 3 (data cell):</strong>

[§ 6.3 Case-sensitivity](#attribute-case)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 13</strong>

<strong>Column 1 (data cell):</strong>

`E[foo="bar" s]`

<strong>Column 2 (data cell):</strong>

<a id="ref-for-string-is"></a>

an E element whose `foo` attribute value is [identical to](https://infra.spec.whatwg.org/#string-is) `bar`

<strong>Column 3 (data cell):</strong>

[§ 6.3 Case-sensitivity](#attribute-case)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 14</strong>

<strong>Column 1 (data cell):</strong>

`E[foo~="bar"]`

<strong>Column 2 (data cell):</strong>

an E element whose `foo` attribute value is a list of whitespace-separated values, one of which is exactly equal to `bar`

<strong>Column 3 (data cell):</strong>

[§ 6.1 Attribute presence and value selectors](#attribute-representation)

<strong>Column 4 (data cell):</strong>

2

<strong>Row 15</strong>

<strong>Column 1 (data cell):</strong>

`E[foo^="bar"]`

<strong>Column 2 (data cell):</strong>

an E element whose `foo` attribute value begins exactly with the string `bar`

<strong>Column 3 (data cell):</strong>

[§ 6.2 Substring matching attribute selectors](#attribute-substrings)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 16</strong>

<strong>Column 1 (data cell):</strong>

`E[foo$="bar"]`

<strong>Column 2 (data cell):</strong>

an E element whose `foo` attribute value ends exactly with the string `bar`

<strong>Column 3 (data cell):</strong>

[§ 6.2 Substring matching attribute selectors](#attribute-substrings)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 17</strong>

<strong>Column 1 (data cell):</strong>

`E[foo*="bar"]`

<strong>Column 2 (data cell):</strong>

an E element whose `foo` attribute value contains the substring `bar`

<strong>Column 3 (data cell):</strong>

[§ 6.2 Substring matching attribute selectors](#attribute-substrings)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 18</strong>

<strong>Column 1 (data cell):</strong>

`E[foo|="en"]`

<strong>Column 2 (data cell):</strong>

an E element whose `foo` attribute value is a hyphen-separated list of values beginning with `en`

<strong>Column 3 (data cell):</strong>

[§ 6.1 Attribute presence and value selectors](#attribute-representation)

<strong>Column 4 (data cell):</strong>

2

<strong>Row 19</strong>

<strong>Column 1 (data cell):</strong>

`E:dir(ltr)`

<strong>Column 2 (data cell):</strong>

an element of type E with left-to-right directionality (the document language specifies how directionality is determined)

<strong>Column 3 (data cell):</strong>

[§ 7.1 The Directionality Pseudo-class: :dir()](#the-dir-pseudo)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 20</strong>

<strong>Column 1 (data cell):</strong>

`E:lang(zh, "*-hant")`

<strong>Column 2 (data cell):</strong>

an element of type E tagged as being either in Chinese (any dialect or writing system) or otherwise written with traditional Chinese characters

<strong>Column 3 (data cell):</strong>

[§ 7.2 The Language Pseudo-class: :lang()](#the-lang-pseudo)

<strong>Column 4 (data cell):</strong>

2/4

<strong>Row 21</strong>

<strong>Column 1 (data cell):</strong>

`E:any-link`

<strong>Column 2 (data cell):</strong>

an E element being the source anchor of a hyperlink

<strong>Column 3 (data cell):</strong>

[§ 8.1 The Hyperlink Pseudo-class: :any-link](#the-any-link-pseudo)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 22</strong>

<strong>Column 1 (data cell):</strong>

`E:link`

<strong>Column 2 (data cell):</strong>

an E element being the source anchor of a hyperlink of which the target is not yet visited

<strong>Column 3 (data cell):</strong>

[§ 8.2 The Link History Pseudo-classes: :link and :visited](#link)

<strong>Column 4 (data cell):</strong>

1

<strong>Row 23</strong>

<strong>Column 1 (data cell):</strong>

`E:visited`

<strong>Column 2 (data cell):</strong>

an E element being the source anchor of a hyperlink of which the target is already visited

<strong>Column 3 (data cell):</strong>

[§ 8.2 The Link History Pseudo-classes: :link and :visited](#link)

<strong>Column 4 (data cell):</strong>

1

<strong>Row 24</strong>

<strong>Column 1 (data cell):</strong>

`E:target`

<strong>Column 2 (data cell):</strong>

an E element being the target of the current URL

<strong>Column 3 (data cell):</strong>

[§ 8.3 The Target Pseudo-class: :target](#the-target-pseudo)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 25</strong>

<strong>Column 1 (data cell):</strong>

`E:scope`

<strong>Column 2 (data cell):</strong>

<a id="ref-for-scoping-root"></a>

an E element being a [scoping root](#scoping-root)

<strong>Column 3 (data cell):</strong>

[§ 8.4 The Reference Element Pseudo-class: :scope](#the-scope-pseudo)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 26</strong>

<strong>Column 1 (data cell):</strong>

`E:active`

<strong>Column 2 (data cell):</strong>

an E element that is in an activated state

<strong>Column 3 (data cell):</strong>

[§ 9.2 The Activation Pseudo-class: :active](#the-active-pseudo)

<strong>Column 4 (data cell):</strong>

1

<strong>Row 27</strong>

<strong>Column 1 (data cell):</strong>

`E:hover`

<strong>Column 2 (data cell):</strong>

an E element that is under the cursor, or that has a descendant under the cursor

<strong>Column 3 (data cell):</strong>

[§ 9.1 The Pointer Hover Pseudo-class: :hover](#the-hover-pseudo)

<strong>Column 4 (data cell):</strong>

2

<strong>Row 28</strong>

<strong>Column 1 (data cell):</strong>

`E:focus`

<strong>Column 2 (data cell):</strong>

an E element that has user input focus

<strong>Column 3 (data cell):</strong>

[§ 9.3 The Input Focus Pseudo-class: :focus](#the-focus-pseudo)

<strong>Column 4 (data cell):</strong>

2

<strong>Row 29</strong>

<strong>Column 1 (data cell):</strong>

`E:focus-within`

<strong>Column 2 (data cell):</strong>

an E element that has user input focus or contains an element that has input focus.

<strong>Column 3 (data cell):</strong>

[§ 9.5 The Focus Container Pseudo-class: :focus-within](#the-focus-within-pseudo)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 30</strong>

<strong>Column 1 (data cell):</strong>

`E:focus-visible`

<strong>Column 2 (data cell):</strong>

an E element that has user input focus, and the UA has determined that a focus ring or other indicator should be drawn for that element

<strong>Column 3 (data cell):</strong>

[§ 9.4 The Focus-Indicated Pseudo-class: :focus-visible](#the-focus-visible-pseudo)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 31</strong>

<strong>Column 1 (data cell):</strong>

`E:enabledE:disabled`

<strong>Column 2 (data cell):</strong>

a user interface element E that is enabled or disabled, respectively

<strong>Column 3 (data cell):</strong>

[§ 12.1.1 The :enabled and :disabled Pseudo-classes](#enableddisabled)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 32</strong>

<strong>Column 1 (data cell):</strong>

`E:read-write`  
`E:read-only`

<strong>Column 2 (data cell):</strong>

a user interface element E that is user alterable, or not

<strong>Column 3 (data cell):</strong>

[§ 12.1.2 The Mutability Pseudo-classes: :read-only and :read-write](#rw-pseudos)

<strong>Column 4 (data cell):</strong>

3-UI/4

<strong>Row 33</strong>

<strong>Column 1 (data cell):</strong>

`E:placeholder-shown`

<strong>Column 2 (data cell):</strong>

an input control currently showing placeholder text

<strong>Column 3 (data cell):</strong>

[§ 12.1.3 The Placeholder-shown Pseudo-class: :placeholder-shown](#placeholder)

<strong>Column 4 (data cell):</strong>

3-UI/4

<strong>Row 34</strong>

<strong>Column 1 (data cell):</strong>

`E:default`

<strong>Column 2 (data cell):</strong>

a user interface element E that is the default item in a group of related choices

<strong>Column 3 (data cell):</strong>

[§ 12.1.5 The Default-option Pseudo-class: :default](#the-default-pseudo)

<strong>Column 4 (data cell):</strong>

3-UI/4

<strong>Row 35</strong>

<strong>Column 1 (data cell):</strong>

`E:checked`  
`E:unchecked`  
`E:indeterminate`

<strong>Column 2 (data cell):</strong>

a user interface element E that is checked/selected (for instance a radio-button or checkbox), unchecked, or in an indeterminate state (neither checked nor unchecked)

<strong>Column 3 (data cell):</strong>

[§ 12.2 Input Value States](#input-value-states)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 36</strong>

<strong>Column 1 (data cell):</strong>

`E:valid`  
`E:invalid`

<strong>Column 2 (data cell):</strong>

a user-input element E that meets, or doesn’t, its data validity semantics

<strong>Column 3 (data cell):</strong>

[§ 12.3.1 The Validity Pseudo-classes: :valid and :invalid](#validity-pseudos)

<strong>Column 4 (data cell):</strong>

3-UI/4

<strong>Row 37</strong>

<strong>Column 1 (data cell):</strong>

`E:in-range`  
`E:out-of-range`

<strong>Column 2 (data cell):</strong>

a user-input element E whose value is in-range/out-of-range

<strong>Column 3 (data cell):</strong>

[§ 12.3.2 The Range Pseudo-classes: :in-range and :out-of-range](#range-pseudos)

<strong>Column 4 (data cell):</strong>

3-UI/4

<strong>Row 38</strong>

<strong>Column 1 (data cell):</strong>

`E:required`  
`E:optional`

<strong>Column 2 (data cell):</strong>

a user-input element E that requires/does not require input

<strong>Column 3 (data cell):</strong>

[§ 12.3.3 The Optionality Pseudo-classes: :required and :optional](#opt-pseudos)

<strong>Column 4 (data cell):</strong>

3-UI/4

<strong>Row 39</strong>

<strong>Column 1 (data cell):</strong>

`E:user-invalid`

<strong>Column 2 (data cell):</strong>

a user-altered user-input element E with incorrect input (invalid, out-of-range, omitted-but-required)

<strong>Column 3 (data cell):</strong>

[§ 12.3.4 The User-interaction Pseudo-classes: :user-valid and :user-invalid](#user-pseudos)

<strong>Column 4 (data cell):</strong>

4

<strong>Row 40</strong>

<strong>Column 1 (data cell):</strong>

`E:root`

<strong>Column 2 (data cell):</strong>

an E element, root of the document

<strong>Column 3 (data cell):</strong>

[§ 13.1 :root pseudo-class](#the-root-pseudo)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 41</strong>

<strong>Column 1 (data cell):</strong>

`E:empty`

<strong>Column 2 (data cell):</strong>

an E element that has no children (neither elements nor text) except perhaps white space

<strong>Column 3 (data cell):</strong>

[§ 13.2 :empty pseudo-class](#the-empty-pseudo)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 42</strong>

<strong>Column 1 (data cell):</strong>

<code>E:nth-child(<var>n</var> &#x5B;of <var>S</var>&#x5D;?)</code>

<strong>Column 2 (data cell):</strong>

an E element, the <var>n</var>-th child of its parent matching <var>S</var>

<strong>Column 3 (data cell):</strong>

[§ 13.3.1 :nth-child() pseudo-class](#the-nth-child-pseudo)

<strong>Column 4 (data cell):</strong>

3/4

<strong>Row 43</strong>

<strong>Column 1 (data cell):</strong>

<code>E:nth-last-child(<var>n</var> &#x5B;of <var>S</var>&#x5D;?)</code>

<strong>Column 2 (data cell):</strong>

an E element, the <var>n</var>-th child of its parent matching <var>S</var>, counting from the last one

<strong>Column 3 (data cell):</strong>

[§ 13.3.2 :nth-last-child() pseudo-class](#the-nth-last-child-pseudo)

<strong>Column 4 (data cell):</strong>

3/4

<strong>Row 44</strong>

<strong>Column 1 (data cell):</strong>

`E:first-child`

<strong>Column 2 (data cell):</strong>

an E element, first child of its parent

<strong>Column 3 (data cell):</strong>

[§ 13.3.3 :first-child pseudo-class](#the-first-child-pseudo)

<strong>Column 4 (data cell):</strong>

2

<strong>Row 45</strong>

<strong>Column 1 (data cell):</strong>

`E:last-child`

<strong>Column 2 (data cell):</strong>

an E element, last child of its parent

<strong>Column 3 (data cell):</strong>

[§ 13.3.4 :last-child pseudo-class](#the-last-child-pseudo)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 46</strong>

<strong>Column 1 (data cell):</strong>

`E:only-child`

<strong>Column 2 (data cell):</strong>

an E element, only child of its parent

<strong>Column 3 (data cell):</strong>

[§ 13.3.5 :only-child pseudo-class](#the-only-child-pseudo)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 47</strong>

<strong>Column 1 (data cell):</strong>

<code>E:nth-of-type(<var>n</var>)</code>

<strong>Column 2 (data cell):</strong>

an E element, the <var>n</var>-th sibling of its type

<strong>Column 3 (data cell):</strong>

[§ 13.4.1 :nth-of-type() pseudo-class](#the-nth-of-type-pseudo)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 48</strong>

<strong>Column 1 (data cell):</strong>

<code>E:nth-last-of-type(<var>n</var>)</code>

<strong>Column 2 (data cell):</strong>

an E element, the <var>n</var>-th sibling of its type, counting from the last one

<strong>Column 3 (data cell):</strong>

[§ 13.4.2 :nth-last-of-type() pseudo-class](#the-nth-last-of-type-pseudo)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 49</strong>

<strong>Column 1 (data cell):</strong>

`E:first-of-type`

<strong>Column 2 (data cell):</strong>

an E element, first sibling of its type

<strong>Column 3 (data cell):</strong>

[§ 13.4.3 :first-of-type pseudo-class](#the-first-of-type-pseudo)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 50</strong>

<strong>Column 1 (data cell):</strong>

`E:last-of-type`

<strong>Column 2 (data cell):</strong>

an E element, last sibling of its type

<strong>Column 3 (data cell):</strong>

[§ 13.4.4 :last-of-type pseudo-class](#the-last-of-type-pseudo)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 51</strong>

<strong>Column 1 (data cell):</strong>

`E:only-of-type`

<strong>Column 2 (data cell):</strong>

an E element, only sibling of its type

<strong>Column 3 (data cell):</strong>

[§ 13.4.5 :only-of-type pseudo-class](#the-only-of-type-pseudo)

<strong>Column 4 (data cell):</strong>

3

<strong>Row 52</strong>

<strong>Column 1 (data cell):</strong>

`E F`

<strong>Column 2 (data cell):</strong>

an F element descendant of an E element

<strong>Column 3 (data cell):</strong>

[§ 14.1 Descendant combinator ( )](#descendant-combinators)

<strong>Column 4 (data cell):</strong>

1

<strong>Row 53</strong>

<strong>Column 1 (data cell):</strong>

`E > F`

<strong>Column 2 (data cell):</strong>

an F element child of an E element

<strong>Column 3 (data cell):</strong>

[§ 14.2 Child combinator (\>)](#child-combinators)

<strong>Column 4 (data cell):</strong>

2

<strong>Row 54</strong>

<strong>Column 1 (data cell):</strong>

`E + F`

<strong>Column 2 (data cell):</strong>

an F element immediately preceded by an E element

<strong>Column 3 (data cell):</strong>

[§ 14.3 Next-sibling combinator (+)](#adjacent-sibling-combinators)

<strong>Column 4 (data cell):</strong>

2

<strong>Row 55</strong>

<strong>Column 1 (data cell):</strong>

`E ~ F`

<strong>Column 2 (data cell):</strong>

an F element preceded by an E element

<strong>Column 3 (data cell):</strong>

[§ 14.4 Subsequent-sibling combinator (~)](#general-sibling-combinators)

<strong>Column 4 (data cell):</strong>

3

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some Level 4 selectors (noted above as "3-UI") were introduced in [\[CSS3UI\]](#biblio-css3ui).

Tests

- css3-modsel-1.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-1.xml)
- css3-modsel-10.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-10.xml)
- css3-modsel-100.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-100.xml)
- css3-modsel-100b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-100b.xml)
- [css3-modsel-101.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-101.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-101.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-101.xml)
- [css3-modsel-101b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-101b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-101b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-101b.xml)
- [css3-modsel-102.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-102.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-102.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-102.xml)
- css3-modsel-102b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-102b.xml)
- [css3-modsel-103.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-103.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-103.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-103.xml)
- [css3-modsel-103b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-103b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-103b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-103b.xml)
- css3-modsel-104.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-104.xml)
- css3-modsel-104b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-104b.xml)
- [css3-modsel-105.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-105.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-105.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-105.xml)
- [css3-modsel-105b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-105b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-105b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-105b.xml)
- [css3-modsel-106.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-106.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-106.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-106.xml)
- [css3-modsel-106b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-106b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-106b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-106b.xml)
- css3-modsel-107.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-107.xml)
- css3-modsel-107b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-107b.xml)
- [css3-modsel-108.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-108.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-108.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-108.xml)
- [css3-modsel-108b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-108b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-108b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-108b.xml)
- [css3-modsel-109.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-109.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-109.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-109.xml)
- [css3-modsel-109b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-109b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-109b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-109b.xml)
- css3-modsel-11.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-11.xml)
- [css3-modsel-110.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-110.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-110.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-110.xml)
- [css3-modsel-110b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-110b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-110b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-110b.xml)
- css3-modsel-111.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-111.xml)
- css3-modsel-111b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-111b.xml)
- [css3-modsel-112.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-112.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-112.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-112.xml)
- [css3-modsel-112b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-112b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-112b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-112b.xml)
- css3-modsel-113.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-113.xml)
- css3-modsel-113b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-113b.xml)
- css3-modsel-114.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-114.xml)
- css3-modsel-114b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-114b.xml)
- [css3-modsel-115.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-115.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-115.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-115.xml)
- [css3-modsel-115b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-115b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-115b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-115b.xml)
- [css3-modsel-116.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-116.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-116.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-116.xml)
- [css3-modsel-116b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-116b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-116b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-116b.xml)
- [css3-modsel-117.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-117.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-117.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-117.xml)
- [css3-modsel-117b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-117b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-117b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-117b.xml)
- css3-modsel-118.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-118.xml)
- css3-modsel-119.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-119.xml)
- css3-modsel-120.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-120.xml)
- css3-modsel-121.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-121.xml)
- [css3-modsel-122.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-122.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-122.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-122.xml)
- css3-modsel-123.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-123.xml)
- css3-modsel-123b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-123b.xml)
- css3-modsel-124.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-124.xml)
- css3-modsel-124b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-124b.xml)
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
- css3-modsel-13.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-13.xml)
- css3-modsel-130.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-130.xml)
- css3-modsel-130b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-130b.xml)
- css3-modsel-131.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-131.xml)
- css3-modsel-131b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-131b.xml)
- css3-modsel-132.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-132.xml)
- css3-modsel-132b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-132b.xml)
- css3-modsel-133.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-133.xml)
- css3-modsel-133b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-133b.xml)
- [css3-modsel-134.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-134.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-134.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-134.xml)
- [css3-modsel-134b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-134b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-134b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-134b.xml)
- [css3-modsel-135.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-135.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-135.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-135.xml)
- [css3-modsel-135b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-135b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-135b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-135b.xml)
- [css3-modsel-136.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-136.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-136.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-136.xml)
- [css3-modsel-136b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-136b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-136b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-136b.xml)
- css3-modsel-137.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-137.xml)
- css3-modsel-137b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-137b.xml)
- css3-modsel-138.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-138.xml)
- css3-modsel-138b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-138b.xml)
- css3-modsel-139.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-139.xml)
- css3-modsel-139b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-139b.xml)
- css3-modsel-14.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-14.xml)
- css3-modsel-140.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-140.xml)
- css3-modsel-140b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-140b.xml)
- [css3-modsel-141.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-141.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-141.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-141.xml)
- [css3-modsel-141b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-141b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-141b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-141b.xml)
- [css3-modsel-142.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-142.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-142.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-142.xml)
- [css3-modsel-142b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-142b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-142b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-142b.xml)
- [css3-modsel-143.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-143.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-143.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-143.xml)
- [css3-modsel-143b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-143b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-143b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-143b.xml)
- css3-modsel-144.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-144.xml)
- css3-modsel-145a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-145a.xml)
- css3-modsel-145b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-145b.xml)
- css3-modsel-146a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-146a.xml)
- css3-modsel-146b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-146b.xml)
- css3-modsel-147a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-147a.xml)
- css3-modsel-147b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-147b.xml)
- [css3-modsel-148.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-148.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-148.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-148.xml)
- [css3-modsel-149.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-149.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-149.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-149.xml)
- [css3-modsel-149b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-149b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-149b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-149b.xml)
- [css3-modsel-14b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-14b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-14b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-14b.xml)
- css3-modsel-14c.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-14c.xml)
- css3-modsel-14d.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-14d.xml)
- css3-modsel-14e.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-14e.xml)
- css3-modsel-15.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-15.xml)
- css3-modsel-150.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-150.xml)
- [css3-modsel-151.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-151.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-151.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-151.xml)
- [css3-modsel-152.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-152.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-152.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-152.xml)
- css3-modsel-153.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-153.xml)
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
- css3-modsel-159.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-159.xml)
- [css3-modsel-15b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-15b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-15b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-15b.xml)
- css3-modsel-16.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-16.xml)
- [css3-modsel-160.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-160.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-160.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-160.xml)
- css3-modsel-161.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-161.xml)
- css3-modsel-166.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-166.xml)
- css3-modsel-166a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-166a.xml)
- css3-modsel-167.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-167.xml)
- css3-modsel-167a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-167a.xml)
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
- css3-modsel-171.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-171.xml)
- [css3-modsel-172a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-172a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-172a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-172a.xml)
- [css3-modsel-172b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-172b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-172b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-172b.xml)
- [css3-modsel-173a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-173a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-173a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-173a.xml)
- [css3-modsel-173b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-173b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-173b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-173b.xml)
- css3-modsel-174a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-174a.xml)
- css3-modsel-174b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-174b.xml)
- [css3-modsel-175a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-175a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-175a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-175a.xml)
- [css3-modsel-175b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-175b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-175b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-175b.xml)
- [css3-modsel-175c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-175c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-175c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-175c.xml)
- [css3-modsel-176.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-176.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-176.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-176.xml)
- css3-modsel-177a.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-177a.xml)
- [css3-modsel-177b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-177b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-177b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-177b.xml)
- [css3-modsel-178.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-178.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-178.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-178.xml)
- [css3-modsel-179.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-179.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-179.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-179.xml)
- css3-modsel-179a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-179a.xml)
- css3-modsel-18.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-18.xml)
- css3-modsel-180a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-180a.xml)
- css3-modsel-181.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-181.xml)
- css3-modsel-182.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-182.xml)
- css3-modsel-183.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-183.xml)
- [css3-modsel-184a.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184a.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184a.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184a.xml)
- [css3-modsel-184b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184b.xml)
- [css3-modsel-184c.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184c.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184c.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184c.xml)
- [css3-modsel-184d.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184d.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184d.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184d.xml)
- [css3-modsel-184e.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184e.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184e.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184e.xml)
- [css3-modsel-184f.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-184f.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-184f.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-184f.xml)
- css3-modsel-18a.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-18a.xml)
- css3-modsel-18b.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-18b.xml)
- css3-modsel-18c.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-18c.xml)
- css3-modsel-19.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-19.xml)
- css3-modsel-19b.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-19b.xml)
- css3-modsel-2.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-2.xml)
- css3-modsel-20.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-20.xml)
- css3-modsel-21.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-21.xml)
- css3-modsel-21b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-21b.xml)
- css3-modsel-21c.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-21c.xml)
- css3-modsel-22.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-22.xml)
- css3-modsel-25.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-25.xml)
- css3-modsel-27.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-27.xml)
- css3-modsel-27a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-27a.xml)
- css3-modsel-27b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-27b.xml)
- css3-modsel-28.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-28.xml)
- css3-modsel-28b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-28b.xml)
- css3-modsel-29.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-29.xml)
- css3-modsel-29b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-29b.xml)
- css3-modsel-3.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-3.xml)
- css3-modsel-30.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-30.xml)
- css3-modsel-31.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-31.xml)
- css3-modsel-32.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-32.xml)
- css3-modsel-33.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-33.xml)
- css3-modsel-34.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-34.xml)
- css3-modsel-35.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-35.xml)
- css3-modsel-36.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-36.xml)
- css3-modsel-37.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-37.xml)
- css3-modsel-38.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-38.xml)
- css3-modsel-39.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-39.xml)
- css3-modsel-39a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-39a.xml)
- css3-modsel-39b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-39b.xml)
- css3-modsel-39c.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-39c.xml)
- css3-modsel-3a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-3a.xml)
- css3-modsel-4.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-4.xml)
- css3-modsel-41.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-41.xml)
- css3-modsel-41a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-41a.xml)
- css3-modsel-42.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-42.xml)
- css3-modsel-42a.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-42a.xml)
- css3-modsel-43.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-43.xml)
- css3-modsel-43b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-43b.xml)
- css3-modsel-44.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-44.xml)
- css3-modsel-44b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-44b.xml)
- css3-modsel-44c.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-44c.xml)
- css3-modsel-44d.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-44d.xml)
- css3-modsel-45.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-45.xml)
- css3-modsel-45b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-45b.xml)
- css3-modsel-45c.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-45c.xml)
- css3-modsel-46.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-46.xml)
- css3-modsel-46b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-46b.xml)
- css3-modsel-47.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-47.xml)
- [css3-modsel-48.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-48.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-48.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-48.xml)
- [css3-modsel-49.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-49.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-49.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-49.xml)
- css3-modsel-5.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-5.xml)
- css3-modsel-50.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-50.xml)
- css3-modsel-51.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-51.xml)
- css3-modsel-52.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-52.xml)
- css3-modsel-53.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-53.xml)
- [css3-modsel-54.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-54.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-54.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-54.xml)
- [css3-modsel-55.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-55.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-55.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-55.xml)
- [css3-modsel-56.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-56.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-56.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-56.xml)
- css3-modsel-57.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-57.xml)
- css3-modsel-57b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-57b.xml)
- [css3-modsel-59.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-59.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-59.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-59.xml)
- css3-modsel-6.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-6.xml)
- [css3-modsel-60.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-60.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-60.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-60.xml)
- css3-modsel-61.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-61.xml)
- css3-modsel-62.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-62.xml)
- css3-modsel-63.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-63.xml)
- css3-modsel-64.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-64.xml)
- css3-modsel-65.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-65.xml)
- css3-modsel-66.xml (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-66.xml)
- css3-modsel-66b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-66b.xml)
- css3-modsel-67.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-67.xml)
- css3-modsel-7.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-7.xml)
- css3-modsel-70.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-70.xml)
- css3-modsel-72.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-72.xml)
- css3-modsel-72b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-72b.xml)
- css3-modsel-73.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-73.xml)
- css3-modsel-73b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-73b.xml)
- css3-modsel-74.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-74.xml)
- css3-modsel-74b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-74b.xml)
- css3-modsel-75.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-75.xml)
- css3-modsel-75b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-75b.xml)
- css3-modsel-76.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-76.xml)
- css3-modsel-76b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-76b.xml)
- css3-modsel-77.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-77.xml)
- css3-modsel-77b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-77b.xml)
- css3-modsel-78.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-78.xml)
- css3-modsel-78b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-78b.xml)
- css3-modsel-79.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-79.xml)
- [css3-modsel-7b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-7b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-7b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-7b.xml)
- css3-modsel-8.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-8.xml)
- css3-modsel-80.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-80.xml)
- css3-modsel-81.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-81.xml)
- css3-modsel-81b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-81b.xml)
- css3-modsel-82.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-82.xml)
- css3-modsel-82b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-82b.xml)
- [css3-modsel-83.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-83.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-83.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-83.xml)
- css3-modsel-86.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-86.xml)
- [css3-modsel-87.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-87.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-87.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-87.xml)
- [css3-modsel-87b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-87b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-87b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-87b.xml)
- css3-modsel-88.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-88.xml)
- css3-modsel-88b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-88b.xml)
- css3-modsel-89.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-89.xml)
- css3-modsel-9.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-9.xml)
- [css3-modsel-90.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-90.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-90.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-90.xml)
- [css3-modsel-90b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-90b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-90b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-90b.xml)
- css3-modsel-91.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-91.xml)
- css3-modsel-92.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-92.xml)
- css3-modsel-93.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-93.xml)
- css3-modsel-94.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-94.xml)
- css3-modsel-94b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-94b.xml)
- css3-modsel-95.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-95.xml)
- css3-modsel-96.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-96.xml)
- css3-modsel-96b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-96b.xml)
- css3-modsel-97.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-97.xml)
- css3-modsel-97b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-97b.xml)
- css3-modsel-98.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-98.xml)
- css3-modsel-98b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-98b.xml)
- [css3-modsel-99.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-99.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-99.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-99.xml)
- [css3-modsel-99b.xml](https://wpt.fyi/results/css/selectors/old-tests/css3-modsel-99b.xml) [(live test)](http://wpt.live/css/selectors/old-tests/css3-modsel-99b.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-99b.xml)
- css3-modsel-d1.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-d1.xml)
- css3-modsel-d1b.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-d1b.xml)
- css3-modsel-d2.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-d2.xml)
- css3-modsel-d3.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-d3.xml)
- css3-modsel-d4.xml (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/old-tests/css3-modsel-d4.xml)

Tests that do not relate to any section

- [eof-right-after-selector-crash.html](https://wpt.fyi/results/css/selectors/eof-right-after-selector-crash.html) [(live test)](http://wpt.live/css/selectors/eof-right-after-selector-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/eof-right-after-selector-crash.html)
- [eof-some-after-selector-crash.html](https://wpt.fyi/results/css/selectors/eof-some-after-selector-crash.html) [(live test)](http://wpt.live/css/selectors/eof-some-after-selector-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/eof-some-after-selector-crash.html)
- [hash-collision.html](https://wpt.fyi/results/css/selectors/hash-collision.html) [(live test)](http://wpt.live/css/selectors/hash-collision.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/hash-collision.html)
- [invalid-pseudos.html](https://wpt.fyi/results/css/selectors/parsing/invalid-pseudos.html) [(live test)](http://wpt.live/css/selectors/parsing/invalid-pseudos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/invalid-pseudos.html)
- [selector-after-font-family.html](https://wpt.fyi/results/css/selectors/selector-after-font-family.html) [(live test)](http://wpt.live/css/selectors/selector-after-font-family.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-after-font-family.html)

------------------------------------------------------------------------

## <a id="syntax"></a>3.  Selector Syntax and Structure

### <a id="structure"></a>3.1.  Structure and Terminology

<a id="ref-for-selector②"></a>

<a id="ref-for-simple"></a>

<a id="ref-for-compound⑥"></a>

<a id="ref-for-complex"></a>

<a id="ref-for-selector-list"></a>

A <a id="selector"></a>selector represents a particular pattern of element(s) in a tree structure. The term [selector](#selector) can refer to a [simple selector](#simple), [compound selector](#compound), [complex selector](#complex), or [selector list](#selector-list). The <a id="selector-subject"></a>subject of a selector is any element that selector is defined to be about; that is, any element <a id="match"></a>matching that <a id="ref-for-selector③"></a>selector.

<a id="ref-for-type-selector"></a>

<a id="ref-for-universal-selector"></a>

<a id="ref-for-attribute-selector"></a>

<a id="ref-for-class-selector"></a>

<a id="ref-for-id-selector"></a>

<a id="ref-for-pseudo-class"></a>

<a id="ref-for-simple①"></a>

<a id="ref-for-typedef-simple-selector"></a>

<a id="ref-for-match"></a>

<a id="ref-for-document-language"></a>

A <a id="simple"></a>simple selector is a single condition on an element. A [type selector](#type-selector), [universal selector](#universal-selector), [attribute selector](#attribute-selector), [class selector](#class-selector), [ID selector](#id-selector), or [pseudo-class](#pseudo-class) is a [simple selector](#simple). (It is represented by [\<simple-selector\>](#typedef-simple-selector) in the selectors [grammar](#grammar).) A given element is said to [match](#match) a <a id="ref-for-simple②"></a>simple selector when that <a id="ref-for-simple③"></a>simple selector, as defined in this specification and in accordance with the [document language](#document-language), accurately describes the element.

<a id="ref-for-simple④"></a>

<a id="ref-for-selector-combinator"></a>

<a id="ref-for-type-selector①"></a>

<a id="ref-for-universal-selector①"></a>

<a id="ref-for-compound⑦"></a>

<a id="ref-for-typedef-compound-selector"></a>

<a id="ref-for-match①"></a>

A <a id="compound"></a>compound selector is a sequence of [simple selectors](#simple) that are not separated by a [combinator](#selector-combinator), and represents a set of simultaneous conditions on a single element. If it contains a [type selector](#type-selector) or [universal selector](#universal-selector), that selector must come first in the sequence. Only one type selector or universal selector is allowed in the sequence. (A [compound selector](#compound) is represented by [\<compound-selector\>](#typedef-compound-selector) in the selectors [grammar](#grammar).) A given element is said to [match](#match) a <a id="ref-for-compound⑧"></a>compound selector when it matches all <a id="ref-for-simple⑤"></a>simple selectors in the <a id="ref-for-compound⑨"></a>compound selector.

<a id="ref-for-descendant-combinator"></a>

<a id="ref-for-simple⑥"></a>

<a id="ref-for-compound①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As whitespace represents the [descendant combinator](#descendant-combinator), no whitespace is allowed between the [simple selectors](#simple) in a [compound selector](#compound).

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-pseudo-class①"></a>

<a id="ref-for-compound①①"></a>

<a id="ref-for-pseudo-compound"></a>

<a id="ref-for-selector-combinator①"></a>

<a id="ref-for-typedef-pseudo-compound-selector"></a>

<a id="ref-for-match②"></a>

<a id="ref-for-originating-element"></a>

<a id="ref-for-universal-selector②"></a>

A <a id="pseudo-compound"></a>pseudo-compound selector is a [pseudo-element](#pseudo-element) selector, optionally followed by additional [pseudo-class](#pseudo-class) selectors, and optionally preceded by a [compound selector](#compound) or another [pseudo-compound selector](#pseudo-compound), without any [combinators](#selector-combinator). (A <a id="ref-for-pseudo-compound①"></a>pseudo-compound selector is represented by [\<pseudo-compound-selector\>](#typedef-pseudo-compound-selector) in the selectors [grammar](#grammar).) A <a id="ref-for-pseudo-element①"></a>pseudo-element [matches](#match) a <a id="ref-for-pseudo-compound②"></a>pseudo-compound selector when it has the specified pseudo-element name, matches the additional conditions represented by any <a id="ref-for-pseudo-class②"></a>pseudo-classes, and has an [originating element](#originating-element) represented by the adjacent preceding selector. If there is no adjacent preceding selector, the [universal selector](#universal-selector) is assumed. (For example, .foo ::before is equivalent to .foo \*::before, and distinct from .foo::before.)

<a id="ref-for-compound①②"></a>

<a id="ref-for-pseudo-compound③"></a>

<a id="ref-for-selectordef-before"></a>

<a id="ref-for-selectordef-marker"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9e91e72f"></a> For example, in .foo::before:hover, the .foo is a [compound selector](#compound), while the ::before:hover is a [pseudo-compound selector](#pseudo-compound). However, in .foo::before::marker, [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) and [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) are separate <a id="ref-for-pseudo-compound④"></a>pseudo-compound selectors.

<a id="ref-for-pseudo-compound⑤"></a>

<a id="ref-for-compound①③"></a>

<a id="ref-for-selector-combinator②"></a>

<a id="ref-for-originating-element①"></a>

<a id="ref-for-selectordef-child"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A [pseudo-compound selector](#pseudo-compound) <strong>is not</strong> a [compound selector](#compound), and can’t be used in places that expect a <a id="ref-for-compound①④"></a>compound selector only. <a id="ref-for-pseudo-compound⑥"></a>Pseudo-compound selectors act as if they carry a [combinator](#selector-combinator) with themselves, expressing their relationship with their [originating element](#originating-element), just as the [\>](#selectordef-child) combinator expresses a relationship with a parent element.

<a id="ref-for-compound①⑤"></a>

<a id="ref-for-descendant-combinator①"></a>

<a id="ref-for-child-combinator"></a>

<a id="ref-for-next-sibling-combinator"></a>

<a id="ref-for-subsequent-sibling-combinator"></a>

<a id="ref-for-match③"></a>

<a id="ref-for-selector-combinator③"></a>

A <a id="selector-combinator"></a>combinator is a condition of relationship between two elements represented by the [compound selectors](#compound) on either side. Combinators in Selectors Level 4 include: the [descendant combinator](#descendant-combinator) (white space), the [child combinator](#child-combinator) (U+003E, `>`), the [next-sibling combinator](#next-sibling-combinator) (U+002B, `+`), and the [subsequent-sibling combinator](#subsequent-sibling-combinator) (U+007E, `~`). Two given elements are said to [match](#match) a [combinator](#selector-combinator) when the condition of relationship between these elements is true.

<a id="ref-for-compound①⑥"></a>

<a id="ref-for-pseudo-compound⑦"></a>

<a id="ref-for-selector-combinator④"></a>

<a id="ref-for-typedef-complex-selector"></a>

<a id="ref-for-match④"></a>

<a id="ref-for-complex①"></a>

<a id="ref-for-pseudo-element②"></a>

<a id="ref-for-originating-element②"></a>

A <a id="complex"></a>complex selector is a sequence of one or more [compound selectors](#compound) and/or [pseudo-compound selectors](#pseudo-compound), with <a id="ref-for-compound①⑦"></a>compound selectors separated by [combinators](#selector-combinator). It represents a set of simultaneous conditions on a set of elements in the particular relationships described by its <a id="ref-for-selector-combinator⑤"></a>combinators. (Complex selectors are represented by [\<complex-selector\>](#typedef-complex-selector) in the selectors [grammar](#grammar).) A given element or pseudo-element is said to [match](#match) a [complex selector](#complex) when it matches the final <a id="ref-for-compound①⑧"></a>compound/<a id="ref-for-pseudo-compound⑧"></a>pseudo-compound selector in the sequence, and every preceding unit of the sequence also <a id="ref-for-match⑤"></a>matches an element or [pseudo-element](#pseudo-element), with the correct relationship between consecutive units as expressed by the combinators separating them (or, for <a id="ref-for-pseudo-compound⑨"></a>pseudo-compound selectors, the correct [originating element](#originating-element) relationship).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3e6d4159"></a> For example, .foo.bar matches an element with both "foo" and "bar" classes.
>
> <a id="ref-for-selectordef-child①"></a>
>
> .ancestor \> .foo.bar matches a subset of those elements: only those whose parent element (as indicated by the [\>](#selectordef-child) combinator) has the "ancestor" class.
>
> <a id="ref-for-selectordef-before①"></a>
>
> <a id="ref-for-originating-element③"></a>
>
> .foo.bar::before matches a [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) pseudo-element, whose [originating element](#originating-element) matches .foo.bar.

<a id="ref-for-simple⑦"></a>

<a id="ref-for-compound①⑨"></a>

<a id="ref-for-complex②"></a>

<a id="ref-for-list-of-simple-selectors"></a>

<a id="ref-for-selector-list①"></a>

<a id="ref-for-match⑥"></a>

<a id="ref-for-selector④"></a>

A <a id="list-of-simple-selectors"></a>list of simple/compound/complex selectors is a comma-separated list of [simple](#simple), [compound](#compound), or [complex selectors](#complex). This is also called just a <a id="selector-list"></a>selector list when the type is either unimportant or specified in the surrounding prose; if the type is important and unspecified, it defaults to meaning a [list of complex selectors](#list-of-simple-selectors). (See [§ 4.1 Selector Lists](#grouping) for additional information on [selector lists](#selector-list) and the various \<\*-selector-list\> productions in the [grammar](#grammar) for their formal syntax.) A given element is said to [match](#match) a <a id="ref-for-selector-list②"></a>selector list when it matches any (at least one) of the [selectors](#selector) in that <a id="ref-for-selector-list③"></a>selector list.

### <a id="data-model"></a>3.2.  Data Model

Selectors are evaluated against an element tree such as the DOM. [\[DOM\]](#biblio-dom) Within this specification, this may be referred to as the "document tree" or "source document".

Each element may have any of the following five aspects, which can be selected against, all of which are matched as strings:

- The element’s type (also known as its tag name).
- The element’s namespace.
- An ID.
- Classes (named groups) to which it belongs.
- Attributes, which are name-value pairs.

<a id="ref-for-lang-pseudo"></a>

<a id="ref-for-document-language①"></a>

<a id="ref-for-selectordef-first-line"></a>

<a id="ref-for-host-language"></a>

Many of the selectors depend on the semantics of the <a id="document-language"></a>document language (i.e. the language and semantics of the document tree) and/or the semantics of the <a id="host-language"></a>host language (i.e. the language that is using selectors syntax). For example, the [:lang()](#lang-pseudo) selector depends on the [document language](#document-language) (e.g. HTML) to define how an element is associated with a language. As a slightly different example, the [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) pseudo-element depends on the [host language](#host-language) (e.g. CSS) to define what a <a id="ref-for-selectordef-first-line①"></a>::first-line pseudo-element represents and what it can do.

#### <a id="featureless-elements"></a>3.2.1.  Featureless Elements

Tests

- [featureless-001.html](https://wpt.fyi/results/css/selectors/featureless-001.html) [(live test)](http://wpt.live/css/selectors/featureless-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/featureless-001.html)
- [featureless-002.html](https://wpt.fyi/results/css/selectors/featureless-002.html) [(live test)](http://wpt.live/css/selectors/featureless-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/featureless-002.html)
- [featureless-003.html](https://wpt.fyi/results/css/selectors/featureless-003.html) [(live test)](http://wpt.live/css/selectors/featureless-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/featureless-003.html)
- [featureless-004.html](https://wpt.fyi/results/css/selectors/featureless-004.html) [(live test)](http://wpt.live/css/selectors/featureless-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/featureless-004.html)
- [featureless-005.html](https://wpt.fyi/results/css/selectors/featureless-005.html) [(live test)](http://wpt.live/css/selectors/featureless-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/featureless-005.html)

<a id="ref-for-featureless"></a>

While individual elements may lack any of the above features, some elements are <a id="featureless"></a>featureless. A [featureless](#featureless) element does not match <em>any selector at all</em>, except:

- <a id="ref-for-simple⑧"></a>

  [simple selectors](#simple) it is explicitly defined to match

- <a id="ref-for-compound②⓪"></a>

  <a id="ref-for-simple⑨"></a>

  [compound selectors](#compound), if all contained [simple selectors](#simple) are allowed to match it

- <a id="ref-for-complex③"></a>

  <a id="ref-for-compound②①"></a>

  <a id="ref-for-selector-subject"></a>

  [complex selectors](#complex), if the [compound selector](#compound) targeting the [subject](#selector-subject) is allowed to match it

- <a id="ref-for-selector-list④"></a>

  [selector lists](#selector-list), if at least one selector in the list is allowed to match it

- <a id="ref-for-logical-combination-pseudo-classes"></a>

  [logical combination pseudo-classes](#logical-combination-pseudo-classes), if their argument selector is allowed to match it

- <a id="ref-for-has-pseudo"></a>

  <a id="ref-for-compound②②"></a>

  <a id="ref-for-simple①⓪"></a>

  the [:has()](#has-pseudo) pseudo-class, if and only if the [compound selector](#compound) it’s part of contains at least one <em>other</em> [simple selector](#simple) that’s allowed to match it.

<a id="ref-for-featureless①"></a>

If a selector would otherwise match a [featureless](#featureless) element, except for the existence of the default namespace [\[CSS-NAMESPACES-3\]](https://www.w3.org/TR/2026/WD-selectors-4-20260122/#biblio-css-namespaces-3) (because <a id="ref-for-featureless②"></a>featureless elements do not have a namespace unless otherwise defined), the default namespace does not prevent the match.

<a id="ref-for-element-shadow-host"></a>

<a id="ref-for-concept-shadow-tree"></a>

<a id="ref-for-featureless③"></a>

<a id="ref-for-pseudo-class③"></a>

<a id="ref-for-selectordef-host"></a>

<a id="ref-for-selectordef-host-context"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-351d0bfb"></a> For example, the [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) is [featureless](#featureless), and can’t be matched by <em>any</em> [pseudo-class](#pseudo-class) except for [:host](https://drafts.csswg.org/css-shadow-1/#selectordef-host) and [:host-context()](https://drafts.csswg.org/css-shadow-1/#selectordef-host-context) (or combinations including those, such as :is(:host, :root)).
>
> <a id="ref-for-element-shadow-host①"></a>
>
> Logical combinations like :not(.foo:host) will never match the host element (even if it doesn’t have a "foo" class), because not all of the simple selectors in .foo:host are allowed to match the [shadow host](https://dom.spec.whatwg.org/#element-shadow-host).
>
> <a id="ref-for-element-shadow-host②"></a>
>
> Similarly, :not(:host \> .foo) will never match the [shadow host](https://dom.spec.whatwg.org/#element-shadow-host), even tho the <a id="ref-for-element-shadow-host③"></a>shadow host is indeed \*not\* a descendant of itself and doesn’t have the "foo" class, because the subject of the complex selector argument (.foo) isn’t allowed to match the <a id="ref-for-element-shadow-host④"></a>shadow host.

<a id="ref-for-featureless④"></a>

<a id="ref-for-simple①①"></a>

<a id="ref-for-x"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a6be590b"></a> In general, you can’t match a [featureless](#featureless) element without explicitly using one of the [simple selectors](#simple) it’s allowed to match, to avoid accidentally selecting one of these elements (which are otherwise <em>intentionally</em> easy to not think about). For example, [\*](https://www.w3.org/TR/selectors-3/#x) will never match a <a id="ref-for-featureless⑤"></a>featureless element.
>
> <a id="ref-for-has-pseudo①"></a>
>
> <a id="ref-for-element-shadow-host⑤"></a>
>
> The rule for [:has()](#has-pseudo), above, works similarly. Even if a [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) contains a .foo descendant, :has(.foo) will not match it, because <em>the rest</em> of the compound selector (empty) doesn’t contain a simple selector that can match the host. You have to write :host:has(.foo) in order to match the host element.

### <a id="scoping"></a>3.3.  Scoped Selectors

Some host applications may choose to <a id="scoped-selector"></a>scope selectors to a particular subtree or fragment of the document, The root of the scoping subtree is called the <a id="scoping-root"></a>scoping root.

<a id="ref-for-scoped-selector"></a>

<a id="ref-for-scoping-root①"></a>

When a selector is [scoped](#scoped-selector), it matches an element only if the element is a descendant of the [scoping root](#scoping-root). (The rest of the selector can match unrestricted; it’s only the final matched elements that must be within the scope.)

<a id="ref-for-scoped-selector①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-32ec254f"></a> For example, the <code><a>querySelector()</a></code> method defined in [\[DOM\]](#biblio-dom) allows the author to evaluate a [scoped](#scoped-selector) selector relative to the element it’s called on.
>
> <a id="ref-for-the-a-element"></a>
>
> <a id="ref-for-the-a-element①"></a>
>
> A call like `widget.querySelector("a")` will thus only find <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element">a</a></code> elements inside of the `widget` element, ignoring any other <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element">a</a></code>s that might be scattered throughout the document.

### <a id="relative"></a>3.4.  Relative Selectors

<a id="ref-for-selector-combinator⑥"></a>

<a id="ref-for-relative-selector-anchor-elements①"></a>

<a id="ref-for-descendant-combinator②"></a>

Certain contexts may accept <a id="relative-selector"></a>relative selectors, which are a shorthand for selectors that represent elements relative to one or more <a id="relative-selector-anchor-elements"></a>relative selector anchor elements. Relative selectors begin with a [combinator](#selector-combinator), with a selector representing the [anchor element](#relative-selector-anchor-elements) implied at the start of the selector. (If no combinator is present, the [descendant combinator](#descendant-combinator) is implied.)

<a id="ref-for-typedef-relative-selector"></a>

<a id="ref-for-typedef-relative-selector-list"></a>

Relative selectors are represented by [\<relative-selector\>](#typedef-relative-selector) in the selectors [grammar](#grammar), and lists of them by [\<relative-selector-list\>](#typedef-relative-selector-list).

### <a id="pseudo-classes"></a>3.5.  Pseudo-classes

<a id="ref-for-simple①②"></a>

<a id="ref-for-pseudo-class④"></a>

<a id="pseudo-class"></a>Pseudo-classes are [simple selectors](#simple) that permit selection based on information that lies outside of the document tree or that can be awkward or impossible to express using the other simple selectors. They can also be dynamic, in the sense that an element can acquire or lose a pseudo-class while a user interacts with the document, without the document itself changing. [Pseudo-classes](#pseudo-class) do not appear in or modify the document source or document tree.

<a id="ref-for-pseudo-class⑤"></a>

<a id="ref-for-css-css-identifier"></a>

The syntax of a [pseudo-class](#pseudo-class) consists of a ":" (U+003A COLON) followed by the name of the <a id="ref-for-pseudo-class⑥"></a>pseudo-class as a CSS [identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier), and, in the case of a <a id="functional-pseudo-class"></a>functional pseudo-class, a pair of parentheses containing its arguments.

<a id="ref-for-valid-pseudo"></a>

<a id="ref-for-lang-pseudo①"></a>

<a id="ref-for-functional-pseudo-class"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-86ae2021"></a> For example, [:valid](#valid-pseudo) is a regular pseudo-class, and [:lang()](#lang-pseudo) is a [functional pseudo-class](#functional-pseudo-class).

<a id="ref-for-pseudo-class⑦"></a>

<a id="ref-for-ascii-case-insensitive"></a>

<a id="ref-for-whitespace"></a>

<a id="ref-for-functional-pseudo-class①"></a>

Like all CSS keywords, [pseudo-class](#pseudo-class) names are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive). No [white space](#whitespace) is allowed between the colon and the name of the <a id="ref-for-pseudo-class⑧"></a>pseudo-class, nor, as usual for CSS syntax, between a [functional pseudo-class](#functional-pseudo-class)’s name and its opening parenthesis (which thus form a CSS function token). Also as usual, <a id="ref-for-whitespace①"></a>white space is allowed around the arguments inside the parentheses of a functional pseudo-class unless otherwise specified.

<a id="ref-for-simple①③"></a>

<a id="ref-for-pseudo-class⑨"></a>

<a id="ref-for-compound②③"></a>

<a id="ref-for-type-selector②"></a>

<a id="ref-for-universal-selector③"></a>

Like other [simple selectors](#simple), [pseudo-classes](#pseudo-class) are allowed in all [compound selectors](#compound) contained in a selector, and must follow the [type selector](#type-selector) or [universal selector](#universal-selector), if present.

<a id="ref-for-pseudo-class①⓪"></a>

<a id="ref-for-compound②④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some [pseudo-classes](#pseudo-class) are mutually exclusive (such that a [compound selector](#compound) containing them, while valid, will never match anything), while others can apply simultaneously to the same element.

### <a id="pseudo-elements"></a>3.6. Pseudo-elements

Tests

- [x-pseudo-element.html](https://wpt.fyi/results/css/selectors/x-pseudo-element.html) [(live test)](http://wpt.live/css/selectors/x-pseudo-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/x-pseudo-element.html)

<a id="ref-for-pseudo-class①①"></a>

Similar to how certain [pseudo-classes](#pseudo-class) represent additional state information not directly present in the document tree, a <a id="pseudo-element"></a>pseudo-element represents an <em>element</em> not directly present in the document tree. They are used to create abstractions about the document tree beyond those provided by the document tree. For example, pseudo-elements can be used to select portions of the document that do not correspond to a document-language element (including such ranges as don’t align to element boundaries or fit within its tree structure); that represent content not in the document tree or in an alternate projection of the document tree; or that rely on information provided by styling, layout, user interaction, and other processes that are not reflected in the document tree.

<a id="ref-for-pseudo-element③"></a>

<a id="ref-for-selectordef-first-letter"></a>

<a id="ref-for-selectordef-first-line②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6916597f"></a> For instance, document languages do not offer mechanisms to access the first letter or first line of an element’s content, but there exist [pseudo-elements](#pseudo-element) ([::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) and [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line)) that allow those things to be styled. Notice especially that in the case of <a id="ref-for-selectordef-first-line③"></a>::first-line, which portion of content is represented by the pseudo-element depends on layout information that cannot be inferred from the document tree.
>
> <a id="ref-for-pseudo-element④"></a>
>
> <a id="ref-for-selectordef-before②"></a>
>
> <a id="ref-for-selectordef-after"></a>
>
> [Pseudo-elements](#pseudo-element) can also represent content that doesn’t exist in the source document at all, such as the [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) and [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudo-elements which allow additional content to be inserted before or after the contents of any element.

<a id="ref-for-pseudo-class①②"></a>

<a id="ref-for-pseudo-element⑤"></a>

<a id="ref-for-structural-pseudo-classes"></a>

<a id="ref-for-originating-element④"></a>

Like [pseudo-classes](#pseudo-class), [pseudo-elements](#pseudo-element) do not appear in or modify the document source or document tree. Accordingly, they also do not affect the interpretation of [structural pseudo-classes](#structural-pseudo-classes) or other selectors pertaining to their [originating element](#originating-element) or its tree.

The host language defines which pseudo-elements exist, their type, and their abilities. Pseudo-elements that exist in CSS are defined in [\[CSS21\]](#biblio-css21) (Level 2), [\[SELECT\]](#biblio-select) (Level 3), and [\[CSS-PSEUDO-4\]](#biblio-css-pseudo-4) (Level 4).

#### <a id="pseudo-element-syntax"></a>3.6.1.  Syntax

<a id="ref-for-pseudo-element⑥"></a>

<a id="ref-for-css-css-identifier①"></a>

<a id="ref-for-ascii-case-insensitive①"></a>

<a id="ref-for-whitespace②"></a>

The syntax of a [pseudo-element](#pseudo-element) is "::" (two U+003A COLON characters) followed by the name of the <a id="ref-for-pseudo-element⑦"></a>pseudo-element as an [identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier), and, in the case of a <a id="functional-pseudo-element"></a>functional pseudo-element, a pair of parentheses containing its arguments. <a id="ref-for-pseudo-element⑧"></a>Pseudo-element names are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive). No [white space](#whitespace) is allowed between the two colons, or between the colons and the name.

<a id="ref-for-selectordef-before③"></a>

<a id="ref-for-selectordef-after①"></a>

<a id="ref-for-selectordef-first-line④"></a>

<a id="ref-for-selectordef-first-letter①"></a>

<a id="ref-for-pseudo-element⑨"></a>

Because [CSS Level 1](https://www.w3.org/TR/CSS1) and [CSS Level 2](https://www.w3.org/TR/CSS2) conflated pseudo-elements and pseudo-classes by sharing a single-colon syntax for both, user agents must also accept the previous one-colon notation for the Level 1 &#x26; 2 pseudo-elements ([::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before), [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after), [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line), and [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter)). This compatibility notation is not allowed for any other [pseudo-elements](#pseudo-element). However, as this syntax is deprecated, authors should use the Level 3+ double-colon syntax for these <a id="ref-for-pseudo-element①⓪"></a>pseudo-elements.

<a id="ref-for-pseudo-element①①"></a>

<a id="ref-for-featureless⑥"></a>

[Pseudo-elements](#pseudo-element) are [featureless](#featureless), and so can’t be matched by any other selector.

#### <a id="pseudo-element-attachment"></a>3.6.2.  Binding to the Document Tree

<a id="ref-for-pseudo-element①②"></a>

<a id="ref-for-compound②⑤"></a>

<a id="ref-for-originating-element⑤"></a>

<a id="ref-for-universal-selector④"></a>

<a id="ref-for-x①"></a>

[Pseudo-elements](#pseudo-element) do not exist independently in the tree: they are always bound to another element on the page, called their <a id="originating-element"></a>originating element. Syntactically, a <a id="ref-for-pseudo-element①③"></a>pseudo-element immediately follows the [compound selector](#compound) representing its [originating element](#originating-element). If this <a id="ref-for-compound②⑥"></a>compound selector is omitted, it is assumed to be the [universal selector](#universal-selector) [\*](https://www.w3.org/TR/selectors-3/#x).

<a id="ref-for-valdef-lab-a"></a>

<a id="ref-for-originating-element⑥"></a>

<a id="ref-for-selectordef-before④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e6d9ca44"></a> For example, in the selector div a::before, the [a](https://www.w3.org/TR/css-color-5/#valdef-lab-a) elements matched by the selector are the [originating elements](#originating-element) for the [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) pseudo-elements attached to them.
>
> <a id="ref-for-selectordef-first-line⑤"></a>
>
> The selector [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) is equivalent to \*::first-line, which selects the <a id="ref-for-selectordef-first-line⑥"></a>::first-line pseudo-element on <em>every</em> element in the document.

<a id="ref-for-pseudo-element①④"></a>

<a id="ref-for-originating-element⑦"></a>

When a [pseudo-element](#pseudo-element) is encountered in a selector, the part of the selector before the <a id="ref-for-pseudo-element①⑤"></a>pseudo-element selects the [originating element](#originating-element) for the <a id="ref-for-pseudo-element①⑥"></a>pseudo-element; the part of the selector after it, if any, applies to the <a id="ref-for-pseudo-element①⑦"></a>pseudo-element itself. (See below.)

#### <a id="pseudo-element-states"></a>3.6.3.  Pseudo-classing Pseudo-elements

<a id="ref-for-pseudo-element①⑧"></a>

<a id="ref-for-pseudo-class①③"></a>

<a id="ref-for-logical-combination-pseudo-classes①"></a>

<a id="ref-for-user-action-pseudo-class①"></a>

<a id="ref-for-invalid-selector"></a>

Certain [pseudo-elements](#pseudo-element) may be immediately followed by any combination of certain [pseudo-classes](#pseudo-class), in which case the <a id="ref-for-pseudo-element①⑨"></a>pseudo-element is represented only when it is in the corresponding state. This specification allows any <a id="ref-for-pseudo-element②⓪"></a>pseudo-element to be followed by any combination of the [logical combination pseudo-classes](#logical-combination-pseudo-classes) and the [user action pseudo-classes](#user-action-pseudo-class). Other specifications may allow additional <a id="ref-for-pseudo-class①④"></a>pseudo-classes to be attached to particular <a id="ref-for-pseudo-element②①"></a>pseudo-elements. Combinations that are not explicitly allowed are [invalid selectors](#invalid-selector).

<a id="ref-for-logical-combination-pseudo-classes②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [logical combination pseudo-classes](#logical-combination-pseudo-classes) pass any restrictions on validity of selectors at their position to their arguments.

<a id="ref-for-hover-pseudo"></a>

<a id="ref-for-focus-pseudo"></a>

<a id="ref-for-selectordef-first-line⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-61c8ad8f"></a> For example, since the [:hover](#hover-pseudo) pseudo-class specifies that it can apply to any pseudo-element, ::first-line:hover will match when the first line is hovered. However, since neither [:focus](#focus-pseudo) nor [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) define that <a id="ref-for-focus-pseudo①"></a>:focus can apply to <a id="ref-for-selectordef-first-line⑧"></a>::first-line, the selector ::first-line:focus will never match anything.
>
> Notice that ::first-line:hover is very different from :hover::first-line, which matches the first line of any originating element that is hovered! For example, :hover::first-line also matches the first line of a paragraph when the second line of the paragraph is hovered, whereas ::first-line:hover only matches if the first line itself is hovered.

#### <a id="sub-pseudo-elements"></a>3.6.4.  Sub-pseudo-elements

<a id="ref-for-pseudo-element②②"></a>

<a id="ref-for-originating-element⑧"></a>

<a id="ref-for-selectordef-before⑤"></a>

<a id="ref-for-valdef-display-list-item"></a>

<a id="ref-for-display-type"></a>

<a id="ref-for-originating-pseudo-element"></a>

<a id="ref-for-sub-pseudo-element"></a>

Some [pseudo-elements](#pseudo-element) are able to be the [originating element](#originating-element) of other <a id="ref-for-pseudo-element②③"></a>pseudo-elements, which are defined as the <a id="sub-pseudo-element"></a>sub-pseudo-elements of this <a id="originating-pseudo-element"></a>originating pseudo-element. For example, when [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) is given a [list-item](https://www.w3.org/TR/css-display-4/#valdef-display-list-item) [display type](https://www.w3.org/TR/css-display-4/#display-type), it becomes the [originating pseudo-element](#originating-pseudo-element) of its ::before::marker [sub-pseudo-element](#sub-pseudo-element).

<a id="ref-for-pseudo-element②④"></a>

Where disambiguation is needed, the term <a id="ultimate-originating-element"></a>ultimate originating element refers to the real (non-pseudo) element from which a [pseudo-element](#pseudo-element) originates.

<a id="ref-for-sub-pseudo-element①"></a>

Unless the corresponding [sub-pseudo-element](#sub-pseudo-element) is explicitly defined to exist in another specification, pseudo-element selectors are not valid when compounded to another pseudo-element selector. So, for example, ::before::before is an invalid selector, but ::before::marker is valid (in implementations that support the ::before::marker <a id="ref-for-sub-pseudo-element②"></a>sub-pseudo-element).

#### <a id="pseudo-element-structure"></a>3.6.5.  Internal Structure

<a id="ref-for-pseudo-element②⑤"></a>

<a id="ref-for-selector-combinator⑦"></a>

Some [pseudo-elements](#pseudo-element) are defined to have internal structure. These <a id="ref-for-pseudo-element②⑥"></a>pseudo-elements may be followed by child/descendant combinators to express those relationships. Selectors containing [combinators](#selector-combinator) after the pseudo-element are otherwise invalid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3594b9ac"></a> For example, ::first-letter + span and ::first-letter em are invalid selectors. However, if a new ::shadow pseudo-element were defined to have internal structure, ::shadow \> p would be a valid selector.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A future specification may expand the capabilities of existing pseudo-elements, so some of these currently-invalid selectors (e.g. ::first-line :any-link) may become valid in the future.

<a id="ref-for-pseudo-element②⑦"></a>

<a id="ref-for-box-tree"></a>

The children of such [pseudo-elements](#pseudo-element) can simultaneously be children of other elements, too. However, at least in CSS, their rendering must be defined so as to maintain the tree-ness of the [box tree](https://www.w3.org/TR/css-display-4/#box-tree).

### <a id="case-sensitive"></a>3.7.  Characters and case sensitivity

Tests

- [case-insensitive-parent.html](https://wpt.fyi/results/css/selectors/case-insensitive-parent.html) [(live test)](http://wpt.live/css/selectors/case-insensitive-parent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/case-insensitive-parent.html)
- [quirks-mode-stylesheet-dynamic-add-001.html](https://wpt.fyi/results/css/selectors/invalidation/quirks-mode-stylesheet-dynamic-add-001.html) [(live test)](http://wpt.live/css/selectors/invalidation/quirks-mode-stylesheet-dynamic-add-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/quirks-mode-stylesheet-dynamic-add-001.html)
- [selectors-case-sensitive-001.html](https://wpt.fyi/results/css/selectors/selectors-case-sensitive-001.html) [(live test)](http://wpt.live/css/selectors/selectors-case-sensitive-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-case-sensitive-001.html)

<a id="ref-for-ascii-case-insensitive②"></a>

All Selectors syntax is [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) (i.e. \[a-z\] and \[A-Z\] are equivalent), except for the parts that are not under the control of Selectors: specifically, the case-sensitivity of document language element names, attribute names, and attribute values depends on the document language.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5ac03fca"></a> For example, [in HTML, element and attribute names are ASCII case-insensitive](https://html.spec.whatwg.org/multipage/semantics-other.html#selectors), but in XML, they are case-sensitive.

<a id="ref-for-language-range"></a>

<a id="ref-for-lang-pseudo②"></a>

Case sensitivity of namespace prefixes is defined in [\[CSS3NAMESPACE\]](#biblio-css3namespace). Case sensitivity of [language ranges](#language-range) is defined in the [:lang()](#lang-pseudo) section.

<a id="whitespace"></a>White space in Selectors consists of the code points SPACE (U+0020), TAB (U+0009), LINE FEED (U+000A), CARRIAGE RETURN (U+000D), and FORM FEED (U+000C). Other space-like code points, such as EM SPACE (U+2003) and IDEOGRAPHIC SPACE (U+3000), are never considered syntactic white space.

Code points in Selectors can be escaped with a backslash according to the same [escaping rules](https://www.w3.org/TR/CSS21/syndata.html#characters) as CSS. [\[CSS21\]](#biblio-css21) Note that escaping a code point “cancels out” any special meaning it may have in Selectors. For example, the selector \#foo\>a contains a combinator, but \#foo&#x5C;\>a instead selects an element with the id `foo>a`.

### <a id="namespaces"></a>3.8.  Declaring Namespace Prefixes

<a id="ref-for-at-ruledef-namespace"></a>

Certain selectors support namespace prefixes. The mechanism by which namespace prefixes are <a id="nsdecl"></a>declared should be specified by the language that uses Selectors. If the language does not specify a namespace prefix declaration mechanism, then no prefixes are declared. In CSS, namespace prefixes are declared with the [@namespace](https://drafts.csswg.org/css-namespaces-3/#at-ruledef-namespace) rule. [\[CSS3NAMESPACE\]](#biblio-css3namespace)

### <a id="invalid"></a>3.9.  Invalid Selectors and Error Handling

Tests

- [selectorText-dynamic-001.html](https://wpt.fyi/results/css/selectors/invalidation/selectorText-dynamic-001.html) [(live test)](http://wpt.live/css/selectors/invalidation/selectorText-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/selectorText-dynamic-001.html)
- [sheet-going-away-001.html](https://wpt.fyi/results/css/selectors/invalidation/sheet-going-away-001.html) [(live test)](http://wpt.live/css/selectors/invalidation/sheet-going-away-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/sheet-going-away-001.html)
- [sheet-going-away-002.html](https://wpt.fyi/results/css/selectors/invalidation/sheet-going-away-002.html) [(live test)](http://wpt.live/css/selectors/invalidation/sheet-going-away-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/sheet-going-away-002.html)

User agents must observe the rules for handling <a id="invalid-selector"></a>invalid selectors:

- a parsing error in a selector, e.g. an unrecognized token or a token which is not allowed at the current parsing point (see overall [§ 16 Grammar](#grammar) and per-selector syntax definitions), causes that selector to be invalid.

- a simple selector containing an [undeclared namespace prefix](#namespaces) is invalid

- a selector containing an invalid simple selector, an invalid combinator or an invalid token is invalid.

- a selector list containing an invalid selector is invalid.

- <a id="ref-for-compound②⑦"></a>

  an empty selector, i.e. one that contains no [compound selector](#compound), is invalid.

<a id="ref-for-invalid-selector①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Consistent with CSS’s forwards-compatible parsing principle, UAs <em>must</em> treat as [invalid](#invalid-selector) any pseudo-classes, pseudo-elements, combinators, or other syntactic constructs for which they have no usable level of support. See [Partial implementations](#w3c-partial).

<a id="ref-for-invalid-selector②"></a>

An [invalid selector](#invalid-selector) represents, and therefore matches, nothing.

### <a id="legacy-aliasing"></a>3.10.  Legacy Aliases

Some selectors have a <a id="legacy-selector-alias"></a>legacy selector alias. This is a name which, at parse time, is converted to the standard name (and thus does not appear anywhere in any object model representing the selector).

## <a id="logical-combination"></a>4.  Logical Combinations

Tests

- [is-where-pseudo-containing-hard-pseudo-and-never-matching.html](https://wpt.fyi/results/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo-and-never-matching.html) [(live test)](http://wpt.live/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo-and-never-matching.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo-and-never-matching.html)
- [is-where-pseudo-containing-hard-pseudo.html](https://wpt.fyi/results/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo.html) [(live test)](http://wpt.live/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/is-where-pseudo-containing-hard-pseudo.html)

<a id="ref-for-compound②⑧"></a>

<a id="ref-for-selector-list⑤"></a>

<a id="ref-for-matches-pseudo"></a>

<a id="ref-for-where-pseudo"></a>

<a id="ref-for-negation-pseudo"></a>

<a id="ref-for-logical-combination-pseudo-classes③"></a>

<a id="ref-for-pseudo-class①⑤"></a>

Selector logic can be manipulated by [compounding](#compound) (logical AND), [selector lists](#selector-list) (logical OR), and the <a id="logical-combination-pseudo-classes"></a>logical combination pseudo-classes [:is()](#matches-pseudo), [:where()](#where-pseudo), and [:not()](#negation-pseudo). The [logical combination pseudo-classes](#logical-combination-pseudo-classes) are allowed anywhere that any other [pseudo-classes](#pseudo-class) are allowed, but pass any restrictions to their arguments. (For example, if only <a id="ref-for-compound②⑨"></a>compound selectors are allowed, then only <a id="ref-for-compound③⓪"></a>compound selectors are valid within an <a id="ref-for-matches-pseudo①"></a>:is().)

<a id="ref-for-matches-pseudo②"></a>

<a id="ref-for-where-pseudo①"></a>

<a id="ref-for-pseudo-class①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since inside [:is()](#matches-pseudo) and [:where()](#where-pseudo) invalid arguments are dropped without invaliding the [pseudo-class](#pseudo-class) itself, selector arguments that are invalidated by contextual restrictions likewise do not invalidate the <a id="ref-for-matches-pseudo③"></a>:is() pseudo-class itself.

### <a id="grouping"></a>4.1.  Selector Lists

<a id="ref-for-selector-list⑥"></a>

A comma-separated list of selectors represents the union of all elements selected by each of the individual selectors in the [selector list](#selector-list). (A comma is U+002C.) For example, in CSS when several selectors share the same declarations, they may be grouped into a comma-separated list. White space may appear before and/or after the comma.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1930da7e"></a> CSS example: In this example, we condense three rules with identical declarations into one. Thus,
>
> ```text
> h1 { font-family: sans-serif }
> h2 { font-family: sans-serif }
> h3 { font-family: sans-serif }
> ```
>
> is equivalent to:
>
> ```text
> h1, h2, h3 { font-family: sans-serif }
> ```
<a id="ref-for-selector-list⑦"></a>

<strong>Warning</strong>: the equivalence is true in this example because all the selectors are valid selectors. If just one of these selectors were invalid, the entire [selector list](#selector-list) would be invalid. This would invalidate the rule for all three heading elements, whereas in the former case only one of the three individual heading rules would be invalidated.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fd0daa05"></a> Invalid CSS example:
>
> ```text
> h1 { font-family: sans-serif }
> h2..foo { font-family: sans-serif }
> h3 { font-family: sans-serif }
> ```
>
> is not equivalent to:
>
> ```text
> h1, h2..foo, h3 { font-family: sans-serif } 
> ```
>
> because the above selector (h1, h2..foo, h3) is entirely invalid and the entire style rule is dropped. (When the selectors are not grouped, only the rule for h2..foo is dropped.)

<a id="ref-for-matches-pseudo④"></a>

### <a id="matches"></a>4.2.  The Matches-Any Pseudo-class: [:is()](#matches-pseudo)

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

<a id="ref-for-typedef-forgiving-selector-list"></a>

The matches-any pseudo-class, <a id="matches-pseudo"></a>:is(), is a functional pseudo-class taking a [\<forgiving-selector-list\>](#typedef-forgiving-selector-list) as its sole argument.

If the argument, after parsing, is an empty list, the pseudo-class is valid but matches nothing. Otherwise, the pseudo-class matches any element that matches any of the selectors in the list.

<a id="ref-for-matches-pseudo⑤"></a>

<a id="ref-for-the-ol-element"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The specificity of the [:is()](#matches-pseudo) pseudo-class is replaced by the specificity of its most specific argument. Thus, a selector written with <a id="ref-for-matches-pseudo⑥"></a>:is() does not necessarily have equivalent specificity to the equivalent selector written without <a id="ref-for-matches-pseudo⑦"></a>:is() For example, if we have :is(ul, ol, .list) \> \[hidden\] and ul \> \[hidden\], ol \> \[hidden\], .list \> \[hidden\] a \[hidden\] child of an <code><a href="https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element">ol</a></code> matches the first selector with a specificity of (0,2,0) whereas it matches the second selector with a specificity of (0,1,1). See [§ 15 Calculating a selector’s specificity](#specificity-rules).

<a id="ref-for-matches-pseudo⑧"></a>

Pseudo-elements cannot be represented by the matches-any pseudo-class; they are not valid within [:is()](#matches-pseudo).

<a id="ref-for-compound③①"></a>

<a id="ref-for-selector-subject①"></a>

<a id="ref-for-matches-pseudo⑨"></a>

<a id="ref-for-universal-selector⑤"></a>

<a id="ref-for-type-selector③"></a>

Default namespace declarations do not affect the [compound selector](#compound) representing the [subject](#selector-subject) of any selector within a [:is()](#matches-pseudo) pseudo-class, unless that compound selector contains an explicit [universal selector](#universal-selector) or [type selector](#type-selector).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2cab03ed"></a> For example, the following selector matches any element that is being hovered or focused, regardless of its namespace. In particular, it is not limited to only matching elements in the default namespace that are being hovered or focused.
>
> ```text
> *|*:is(:hover, :focus) 
> ```
>
> <a id="ref-for-matches-pseudo①⓪"></a>
>
> The following selector, however, represents only hovered or focused elements that are in the default namespace, because it uses an explicit universal selector within the [:is()](#matches-pseudo) notation:
>
> ```text
> *|*:is(*:hover, *:focus) 
> ```
<a id="ref-for-legacy-selector-alias"></a>

<a id="ref-for-matches-pseudo①①"></a>

As previous drafts of this specification used the name <a id="selectordef-matches"></a>:matches() for this pseudo-class, UAs may additionally implement this obsolete name as a [legacy selector alias](#legacy-selector-alias) for [:is()](#matches-pseudo) if needed for backwards-compatibility.

<a id="ref-for-negation-pseudo①"></a>

### <a id="negation"></a>4.3.  The Negation (Matches-None) Pseudo-class: [:not()](#negation-pseudo)

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

<a id="ref-for-typedef-complex-real-selector-list"></a>

The negation pseudo-class, <a id="negation-pseudo"></a>:not(), is a functional pseudo-class taking a [\<complex-real-selector-list\>](#typedef-complex-real-selector-list) as an argument. It represents an element that is not represented by its argument.

<a id="ref-for-simple①④"></a>

<a id="ref-for-negation-pseudo②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In Selectors Level 3, only a single [simple selector](#simple) was allowed as the argument to [:not()](#negation-pseudo).

<a id="ref-for-negation-pseudo③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The specificity of the [:not()](#negation-pseudo) pseudo-class is replaced by the specificity of the most specific selector in its argument; thus it has the exact behavior of :not(:is(<var>argument</var>)). See [§ 15 Calculating a selector’s specificity](#specificity-rules).

<a id="ref-for-negation-pseudo④"></a>

Pseudo-elements cannot be represented by the negation pseudo-class; they are not valid within [:not()](#negation-pseudo).

<a id="ref-for-the-button-element"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2a176618"></a> For example, the following selector matches all [button](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element) elements in an HTML document that are not disabled.
>
> ```text
> button:not([DISABLED]) 
> ```
>
> The following selector represents all but FOO elements.
>
> ```text
> *:not(FOO)
> ```
>
> The following compound selector represents all HTML elements except links.
>
> ```text
> html|*:not(:link):not(:visited)
> ```
<a id="ref-for-matches-pseudo①②"></a>

<a id="ref-for-compound③②"></a>

<a id="ref-for-selector-subject②"></a>

<a id="ref-for-negation-pseudo⑤"></a>

<a id="ref-for-universal-selector⑥"></a>

<a id="ref-for-type-selector④"></a>

As with [:is()](#matches-pseudo), default namespace declarations do not affect the [compound selector](#compound) representing the [subject](#selector-subject) of any selector within a [:not()](#negation-pseudo) pseudo-class, unless that compound selector contains an explicit [universal selector](#universal-selector) or [type selector](#type-selector). (See <a id="ref-for-matches-pseudo①③"></a>:is() for examples.)

<a id="ref-for-negation-pseudo⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [:not()](#negation-pseudo) pseudo-class allows useless selectors to be written. For instance :not(\*\|\*), which represents no element at all, or div:not(span), which is equivalent to div but with a higher specificity.

<a id="ref-for-where-pseudo②"></a>

### <a id="zero-matches"></a>4.4.  The Specificity-adjustment Pseudo-class: [:where()](#where-pseudo)

Tests

- [where.html](https://wpt.fyi/results/css/selectors/invalidation/where.html) [(live test)](http://wpt.live/css/selectors/invalidation/where.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/where.html)
- [parse-where.html](https://wpt.fyi/results/css/selectors/parsing/parse-where.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-where.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-where.html)
- [pseudo-where-crash.html](https://wpt.fyi/results/css/selectors/pseudo-where-crash.html) [(live test)](http://wpt.live/css/selectors/pseudo-where-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/pseudo-where-crash.html)

<a id="ref-for-matches-pseudo①④"></a>

<a id="ref-for-where-pseudo③"></a>

<a id="ref-for-specificity"></a>

The Specificity-adjustment pseudo-class, <a id="where-pseudo"></a>:where(), is a functional pseudo-class with the same syntax and functionality as [:is()](#matches-pseudo). Unlike <a id="ref-for-matches-pseudo①⑤"></a>:is(), neither the [:where()](#where-pseudo) pseudo-class, nor any of its arguments, contribute to the [specificity](#specificity) of the selector—​its <a id="ref-for-specificity①"></a>specificity is always zero.

This is useful for introducing filters in a selector while keeping the associated style declarations easy to override.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7ca50acf"></a> Below is a common example where the specificity heuristic fails to match author expectations:
>
> ```text
> a:not(:hover) {
>   text-decoration: none;
> }
> 
> nav a {
>   /* Has no effect */
>   text-decoration: underline;
> }
> ```
>
> <a id="ref-for-where-pseudo④"></a>
>
> However, by using [:where()](#where-pseudo) the author can explicitly declare their intent:
>
> ```text
> a:where(:not(:hover)) {
>   text-decoration: none;
> }
> 
> nav a {
>   /* Works now! */
>   text-decoration: underline;
> }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Future levels of Selectors may introduce an additional argument to explicitly set the specificity of that instance of the pseudo-class.

<a id="ref-for-has-pseudo②"></a>

### <a id="relational"></a>4.5.  The Relational Pseudo-class: [:has()](#has-pseudo)

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
- [fullscreen-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/fullscreen-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/fullscreen-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/fullscreen-pseudo-class-in-has.html)
- [has-append-first-node.html](https://wpt.fyi/results/css/selectors/invalidation/has-append-first-node.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-append-first-node.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-append-first-node.html)
- [has-complexity.html](https://wpt.fyi/results/css/selectors/invalidation/has-complexity.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-complexity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-complexity.html)
- [has-css-nesting-shared.html](https://wpt.fyi/results/css/selectors/invalidation/has-css-nesting-shared.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-css-nesting-shared.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-css-nesting-shared.html)
- [has-in-adjacent-position.html](https://wpt.fyi/results/css/selectors/invalidation/has-in-adjacent-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-in-adjacent-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-in-adjacent-position.html)
- [has-in-ancestor-position.html](https://wpt.fyi/results/css/selectors/invalidation/has-in-ancestor-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/has-in-ancestor-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/has-in-ancestor-position.html)
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
- [is-pseudo-containing-complex-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/is-pseudo-containing-complex-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/is-pseudo-containing-complex-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/is-pseudo-containing-complex-in-has.html)
- [is-pseudo-containing-sibling-relationship-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/is-pseudo-containing-sibling-relationship-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/is-pseudo-containing-sibling-relationship-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/is-pseudo-containing-sibling-relationship-in-has.html)
- [lang-pseudo-class-in-has-document-element.html](https://wpt.fyi/results/css/selectors/invalidation/lang-pseudo-class-in-has-document-element.html) [(live test)](http://wpt.live/css/selectors/invalidation/lang-pseudo-class-in-has-document-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/lang-pseudo-class-in-has-document-element.html)
- [lang-pseudo-class-in-has-multiple-document-elements.html](https://wpt.fyi/results/css/selectors/invalidation/lang-pseudo-class-in-has-multiple-document-elements.html) [(live test)](http://wpt.live/css/selectors/invalidation/lang-pseudo-class-in-has-multiple-document-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/lang-pseudo-class-in-has-multiple-document-elements.html)
- [lang-pseudo-class-in-has-xhtml.xhtml](https://wpt.fyi/results/css/selectors/invalidation/lang-pseudo-class-in-has-xhtml.xhtml) [(live test)](http://wpt.live/css/selectors/invalidation/lang-pseudo-class-in-has-xhtml.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/lang-pseudo-class-in-has-xhtml.xhtml)
- [lang-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/lang-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/lang-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/lang-pseudo-class-in-has.html)
- [link-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/link-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/link-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/link-pseudo-class-in-has.html)
- [link-pseudo-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/link-pseudo-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/link-pseudo-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/link-pseudo-in-has.html)
- [location-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/location-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/location-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/location-pseudo-classes-in-has.html)
- [media-loading-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/media-loading-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/media-loading-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/media-loading-pseudo-classes-in-has.html)
- [media-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/media-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/media-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/media-pseudo-classes-in-has.html)
- [modal-pseudo-class-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/modal-pseudo-class-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/modal-pseudo-class-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/modal-pseudo-class-in-has.html)
- [negated-has-in-nonsubject-position.html](https://wpt.fyi/results/css/selectors/invalidation/negated-has-in-nonsubject-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-has-in-nonsubject-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-has-in-nonsubject-position.html)
- [not-pseudo-containing-complex-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/not-pseudo-containing-complex-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/not-pseudo-containing-complex-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/not-pseudo-containing-complex-in-has.html)
- [not-pseudo-containing-sibling-relationship-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/not-pseudo-containing-sibling-relationship-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/not-pseudo-containing-sibling-relationship-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/not-pseudo-containing-sibling-relationship-in-has.html)
- [state-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/state-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/state-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/state-in-has.html)
- [subject-has-invalidation-with-display-none-anchor-element.html](https://wpt.fyi/results/css/selectors/invalidation/subject-has-invalidation-with-display-none-anchor-element.html) [(live test)](http://wpt.live/css/selectors/invalidation/subject-has-invalidation-with-display-none-anchor-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/subject-has-invalidation-with-display-none-anchor-element.html)
- [target-pseudo-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/target-pseudo-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/target-pseudo-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/target-pseudo-in-has.html)
- [typed-child-indexed-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/typed-child-indexed-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/typed-child-indexed-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/typed-child-indexed-pseudo-classes-in-has.html)
- [user-action-pseudo-classes-in-has.html](https://wpt.fyi/results/css/selectors/invalidation/user-action-pseudo-classes-in-has.html) [(live test)](http://wpt.live/css/selectors/invalidation/user-action-pseudo-classes-in-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/user-action-pseudo-classes-in-has.html)
- [parse-has-disallow-nesting-has-inside-has.html](https://wpt.fyi/results/css/selectors/parsing/parse-has-disallow-nesting-has-inside-has.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-has-disallow-nesting-has-inside-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-has-disallow-nesting-has-inside-has.html)
- [parse-has-forgiving-selector.html](https://wpt.fyi/results/css/selectors/parsing/parse-has-forgiving-selector.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-has-forgiving-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-has-forgiving-selector.html)
- [parse-has.html](https://wpt.fyi/results/css/selectors/parsing/parse-has.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-has.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-has.html)

<a id="ref-for-typedef-relative-selector-list①"></a>

<a id="ref-for-relative-selector①"></a>

<a id="ref-for-relative-selector-anchor-elements②"></a>

The relational pseudo-class, <a id="has-pseudo"></a>:has(), is a functional pseudo-class taking a [\<relative-selector-list\>](#typedef-relative-selector-list) as an argument. It represents an element if any of the [relative selectors](#relative-selector) would match at least one element when [anchored against](#relative-selector-anchor-elements) this element.

<a id="ref-for-has-pseudo③"></a>

<a id="ref-for-pseudo-element②⑧"></a>

<a id="ref-for-has-allowed-pseudo-element"></a>

The [:has()](#has-pseudo) pseudo-class cannot be nested; <a id="ref-for-has-pseudo④"></a>:has() is not valid within <a id="ref-for-has-pseudo⑤"></a>:has(). Also, unless explicitly defined as a <a id="has-allowed-pseudo-element"></a>:has-allowed pseudo-element, [pseudo-elements](#pseudo-element) are not valid selectors within <a id="ref-for-has-pseudo⑥"></a>:has(). (This specification does not define any [:has-allowed pseudo-elements](#has-allowed-pseudo-element), but other specifications may do so.)

<a id="ref-for-has-pseudo⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Pseudo-elements are generally excluded from [:has()](#has-pseudo) because many of them exist conditionally, based on the styling of their ancestors, so allowing these to be queried by <a id="ref-for-has-pseudo⑧"></a>:has() would introduce cycles.

<a id="ref-for-has-pseudo⑨"></a>

<a id="ref-for-typedef-relative-selector-list②"></a>

<a id="ref-for-complex④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since [:has()](#has-pseudo) takes a [\<relative-selector-list\>](#typedef-relative-selector-list), its arguments are <em>inherently</em> [complex selectors](#complex) (because they start, perhaps implicitly, with a combinator). This means <a id="ref-for-has-pseudo①⓪"></a>:has() cannot be used in contexts that don’t allow complex selectors; its arguments will be guaranteed to be invalid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-61d52da1"></a> For example, the following selector matches only `<a>` elements that contain an `<img>` child:
>
> ```text
> a:has(> img)
> ```
>
> The following selector matches a `<dt>` element immediately followed by another `<dt>` element:
>
> ```text
> dt:has(+ dt)
> ```
>
> The following selector matches `<section>` elements that don’t contain any heading elements:
>
> ```text
> section:not(:has(h1, h2, h3, h4, h5, h6))
> ```
>
> Note that ordering matters in the above selector. Swapping the nesting of the two pseudo-classes, like:
>
> ```text
> section:has(:not(h1, h2, h3, h4, h5, h6))
> ```
>
> ...would result in matching any `<section>` element which contains anything that’s not a heading element.

## <a id="elemental-selectors"></a>5.  Elemental selectors

### <a id="type-selectors"></a>5.1.  Type (tag name) selector

A <a id="type-selector"></a>type selector is the name of a document language element type, and represents an instance of that element type in the document tree.

<a id="ref-for-the-h1-element"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-087f4a77"></a> For example, the selector h1 represents an [h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element in the document.

<a id="ref-for-type-selector⑤"></a>

<a id="ref-for-css-qualified-name"></a>

<a id="ref-for-css-css-identifier②"></a>

A [type selector](#type-selector) is written as a [CSS qualified name](https://www.w3.org/TR/css-namespaces-3/#css-qualified-name): an [identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier) with an optional namespace prefix. [\[CSS3NAMESPACE\]](#biblio-css3namespace) (See [§ 5.3 Namespaces in Elemental Selectors](#type-nmsp).)

### <a id="the-universal-selector"></a>5.2.  Universal selector 

Tests

- [parse-universal.html](https://wpt.fyi/results/css/selectors/parsing/parse-universal.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-universal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-universal.html)

<a id="ref-for-type-selector⑥"></a>

The <a id="universal-selector"></a>universal selector is a special [type selector](#type-selector), that represents an element of any element type.

<a id="ref-for-css-qualified-name①"></a>

<a id="ref-for-type-selector⑦"></a>

<a id="ref-for-universal-selector⑦"></a>

It is written as a [CSS qualified name](https://www.w3.org/TR/css-namespaces-3/#css-qualified-name) with an asterisk (`*` U+002A) as the local name. Like a [type selector](#type-selector), the [universal selector](#universal-selector) can be qualified by a namespace, restricting it to only elements belonging to that namespace, and is affected by a default namespace as defined in [§ 5.3 Namespaces in Elemental Selectors](#type-nmsp).

<a id="ref-for-featureless⑦"></a>

<a id="ref-for-universal-selector⑧"></a>

Unless an element is [featureless](#featureless), the presence of a [universal selector](#universal-selector) has no effect on whether the element matches the selector. (<a id="ref-for-featureless⑧"></a>Featureless elements do not match any selector, including the <a id="ref-for-universal-selector⑨"></a>universal selector.)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0f5d51ca"></a>
>
> - \*\[hreflang\|=en\] and \[hreflang\|=en\] are equivalent,
> - \*.warning and .warning are equivalent,
> - \*#myid and \#myid are equivalent.

<a id="ref-for-universal-selector①⓪"></a>

<a id="ref-for-type-selector⑧"></a>

<a id="ref-for-compound③③"></a>

<a id="ref-for-simple①⑤"></a>

The [universal selector](#universal-selector) follows the same syntax rules as other [type selectors](#type-selector): only one can appear per [compound selector](#compound), and it must be the first [simple selector](#simple) in the <a id="ref-for-compound③④"></a>compound selector.

<a id="ref-for-universal-selector①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In some cases, adding a [universal selector](#universal-selector) can make a selector easier to read, even though it has no effect on the matching behavior. For example, div :first-child and div:first-child are somewhat difficult to tell apart at a quick glance, but writing the former as div \*:first-child makes the difference obvious.

### <a id="type-nmsp"></a>5.3.  Namespaces in Elemental Selectors

<a id="ref-for-type-selector⑨"></a>

<a id="ref-for-universal-selector①②"></a>

<a id="ref-for-nsdecl"></a>

[Type selectors](#type-selector) and [universal selectors](#universal-selector) allow an optional namespace component: a namespace prefix that has been previously [declared](#nsdecl) may be prepended to the element name separated by the namespace separator “vertical bar” (`|` U+007C). (See, e.g., [\[XML-NAMES\]](#biblio-xml-names) for the use of namespaces in XML.) It has the following meaning in each form:

`ns|E`  
elements with name E in namespace ns

`*|E`  
elements with name E in any namespace, including those without a namespace

`|E`  
elements with name E without a namespace

`E`  
<a id="ref-for-nsdecl②"></a>

if no default namespace has been [declared](#nsdecl) for selectors, this is equivalent to \*\|E. Otherwise it is equivalent to ns\|E where ns is the default namespace.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-53e1ab95"></a> CSS examples:
>
> ```text
> @namespace foo url(http://www.example.com);
> foo|h1 { color: blue }  /* first rule */
> foo|* { color: yellow } /* second rule */
> |h1 { color: red }      /* ...*/
> *|h1 { color: green }
> h1 { color: green }
> ```
>
> <a id="ref-for-at-ruledef-namespace①"></a>
>
> <a id="ref-for-the-h1-element①"></a>
>
> The first rule (not counting the [@namespace](https://drafts.csswg.org/css-namespaces-3/#at-ruledef-namespace) at-rule) will match only [h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) elements in the "http://www.example.com" namespace.
>
> The second rule will match all elements in the "http://www.example.com" namespace.
>
> <a id="ref-for-the-h1-element②"></a>
>
> The third rule will match only [h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) elements with no namespace.
>
> <a id="ref-for-the-h1-element③"></a>
>
> The fourth rule will match [h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) elements in any namespace (including those without any namespace).
>
> The last rule is equivalent to the fourth rule because no default namespace has been defined.

<a id="ref-for-default-namespace"></a>

<a id="ref-for-compound③⑤"></a>

<a id="ref-for-type-selector①⓪"></a>

If a [default namespace](https://www.w3.org/TR/css-namespaces-3/#default-namespace) is declared, [compound selectors](#compound) without [type selectors](#type-selector) in them still only match elements in that default namespace.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-03281ceb"></a> For example, in the following style sheet:
>
> ```text
> @namespace url("http://example.com/foo");
> 
> .special { ... }
> ```
>
> The .special selector only matches elements in the "http://example.com/foo" namespace, even though no reference to the type name (which is paired with the namespace in the DOM) appeared.

<a id="ref-for-type-selector①①"></a>

<a id="ref-for-universal-selector①③"></a>

<a id="ref-for-nsdecl③"></a>

<a id="ref-for-invalid-selector③"></a>

A [type selector](#type-selector) or [universal selector](#universal-selector) containing a namespace prefix that has not been previously [declared](#nsdecl) is an [invalid selector](#invalid-selector).

<a id="ref-for-defined-pseudo"></a>

### <a id="the-defined-pseudo"></a>5.4.  The Defined Pseudo-class: [:defined](#defined-pseudo)

Tests

- [defined.html](https://wpt.fyi/results/css/selectors/invalidation/defined.html) [(live test)](http://wpt.live/css/selectors/invalidation/defined.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/defined.html)

<a id="ref-for-pseudo-class①⑦"></a>

In some host languages, elements can have a distinction between being “defined”/“constructed” or not. The <a id="defined-pseudo"></a>:defined [pseudo-class](#pseudo-class) matches elements that are fully defined, as dictated by the host language.

<a id="ref-for-defined-pseudo①"></a>

If the host language does not have this sort of distinction, all elements in it match [:defined](#defined-pseudo).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cd9dcdb2"></a> In HTML, all built-in elements are always considered to be defined, so the following example will always match:
>
> ```text
> p:defined { ... }
> ```
>
> <a id="ref-for-custom-element"></a>
>
> <a id="ref-for-element-definition"></a>
>
> <a id="ref-for-defined-pseudo②"></a>
>
> [Custom elements](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element), on the other hand, start out <em>un</em>defined, and only become defined when [properly registered](https://html.spec.whatwg.org/multipage/custom-elements.html#element-definition). This means the [:defined](#defined-pseudo) pseudo-class can be used to hide a custom element until it has been registered:
>
> ```text
> custom-element { visibility: hidden }
> ```
>
> ```text
> custom-element:defined { visibility: visible }
> ```
## <a id="attribute-selectors"></a>6.  Attribute selectors

Tests

- [style-attribute-selector.html](https://wpt.fyi/results/css/selectors/attribute-selectors/style-attribute-selector.html) [(live test)](http://wpt.live/css/selectors/attribute-selectors/style-attribute-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/attribute-selectors/style-attribute-selector.html)
- [attribute.html](https://wpt.fyi/results/css/selectors/invalidation/attribute.html) [(live test)](http://wpt.live/css/selectors/invalidation/attribute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/attribute.html)
- [class-id-attr.html](https://wpt.fyi/results/css/selectors/invalidation/class-id-attr.html) [(live test)](http://wpt.live/css/selectors/invalidation/class-id-attr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/class-id-attr.html)
- [missing-right-token.html](https://wpt.fyi/results/css/selectors/missing-right-token.html) [(live test)](http://wpt.live/css/selectors/missing-right-token.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/missing-right-token.html)
- [parse-attribute.html](https://wpt.fyi/results/css/selectors/parsing/parse-attribute.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-attribute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-attribute.html)
- [selectors-attr-many.html](https://wpt.fyi/results/css/selectors/selectors-attr-many.html) [(live test)](http://wpt.live/css/selectors/selectors-attr-many.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-attr-many.html)

Selectors allow the representation of an element’s attributes. When a selector is used as an expression to match against an element, an <a id="attribute-selector"></a>attribute selector must be considered to match an element if that element has an attribute that matches the attribute represented by the attribute selector.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-745ef775"></a>Add comma-separated syntax for [multiple-value matching](https://lists.w3.org/Archives/Public/www-style/2011Mar/0215.html)? e.g. \[rel ~= next, prev, up, first, last\]

### <a id="attribute-representation"></a>6.1.  Attribute presence and value selectors

CSS2 introduced four attribute selectors:

\[att\]  
Represents an element with the `att` attribute, whatever the value of the attribute.

\[att=val\]  
Represents an element with the `att` attribute whose value is exactly "val".

\[att~=val\]  
<a id="ref-for-whitespace③"></a>

Represents an element with the `att` attribute whose value is a [whitespace](#whitespace)-separated list of words, one of which is exactly "val". If "val" contains whitespace, it will never represent anything (since the words are <em>separated</em> by spaces). Also if "val" is the empty string, it will never represent anything.

\[att\|=val\]  
<a id="ref-for-lang-pseudo③"></a>

<a id="ref-for-the-a-element②"></a>

Represents an element with the `att` attribute, its value either being exactly "val" or beginning with "val" immediately followed by "-" (U+002D). This is primarily intended to allow language subcode matches (e.g., the `hreflang` attribute on the [a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) element in HTML) as described in BCP 47 ([\[BCP47\]](#biblio-bcp47)) or its successor. For `lang` (or `xml:lang`) language subcode matching, please see the [:lang()](#lang-pseudo) pseudo-class.

<a id="ref-for-typedef-ident-token"></a>

<a id="ref-for-typedef-string-token"></a>

Attribute values must be [\<ident-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-ident-token)s or [\<string-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-string-token)s. [\[CSS3SYN\]](#biblio-css3syn)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-52ee808b"></a> Examples:
>
> <a id="ref-for-the-h1-element④"></a>
>
> The following attribute selector represents an [h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element that carries the `title` attribute, whatever its value:
>
> ```text
> h1[title]
> ```
>
> <a id="ref-for-the-span-element"></a>
>
> In the following example, the selector represents a [span](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-span-element) element whose `class` attribute has exactly the value "example":
>
> ```text
> span[class="example"]
> ```
>
> <a id="ref-for-the-span-element①"></a>
>
> Multiple attribute selectors can be used to represent several attributes of an element, or several conditions on the same attribute. Here, the selector represents a [span](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-span-element) element whose `hello` attribute has exactly the value "Cleveland" and whose `goodbye` attribute has exactly the value "Columbus":
>
> ```text
> span[hello="Cleveland"][goodbye="Columbus"]
> ```
>
> <a id="ref-for-the-a-element③"></a>
>
> The following CSS rules illustrate the differences between "=" and "~=". The first selector would match, for example, an [a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) element with the value "copyright copyleft copyeditor" on a `rel` attribute. The second selector would only match an <a id="ref-for-the-a-element④"></a>a element with an `href` attribute having the exact value "http://www.w3.org/".
>
> ```text
> a[rel~="copyright"] { ... }
> a[href="http://www.w3.org/"] { ... }
> ```
>
> <a id="ref-for-the-a-element⑤"></a>
>
> The following selector represents an [a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) element whose `hreflang` attribute is exactly "fr".
>
> ```text
> a[hreflang=fr] 
> ```
>
> <a id="ref-for-the-a-element⑥"></a>
>
> The following selector represents an [a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) element for which the value of the `hreflang` attribute begins with "en", including "en", "en-US", and "en-scouse":
>
> ```text
> a[hreflang|="en"] 
> ```
>
> The following selectors represent a DIALOGUE element whenever it has one of two different values for an attribute `character`:
>
> ```text
> DIALOGUE[character=romeo]
> DIALOGUE[character=juliet]
> ```
### <a id="attribute-substrings"></a>6.2.  Substring matching attribute selectors

Three additional attribute selectors are provided for matching substrings in the value of an attribute:

\[att^=val\]  
Represents an element with the `att` attribute whose value begins with the prefix "val". If "val" is the empty string then the selector does not represent anything.

\[att\$=val\]  
Represents an element with the `att` attribute whose value ends with the suffix "val". If "val" is the empty string then the selector does not represent anything.

\[att\*=val\]  
Represents an element with the `att` attribute whose value contains at least one instance of the substring "val". If "val" is the empty string then the selector does not represent anything.

<a id="ref-for-typedef-ident-token①"></a>

<a id="ref-for-typedef-string-token①"></a>

Attribute values must be [\<ident-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-ident-token)s or [\<string-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-string-token)s.

<a id="ref-for-the-object-element"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d887e2c3"></a> Examples: The following selector represents an HTML [object](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-object-element) element, referencing an image:
>
> ```text
> object[type^="image/"] 
> ```
>
> <a id="ref-for-the-a-element⑦"></a>
>
> The following selector represents an HTML [a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) element with an `href` attribute whose value ends with ".html".
>
> ```text
> a[href$=".html"] 
> ```
>
> The following selector represents an HTML paragraph with a `title` attribute whose value contains the substring "hello"
>
> ```text
> p[title*="hello"] 
> ```
### <a id="attribute-case"></a>6.3.  Case-sensitivity

Tests

- [cssom.html](https://wpt.fyi/results/css/selectors/attribute-selectors/attribute-case/cssom.html) [(live test)](http://wpt.live/css/selectors/attribute-selectors/attribute-case/cssom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/attribute-selectors/attribute-case/cssom.html)
- [semantics.html](https://wpt.fyi/results/css/selectors/attribute-selectors/attribute-case/semantics.html) [(live test)](http://wpt.live/css/selectors/attribute-selectors/attribute-case/semantics.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/attribute-selectors/attribute-case/semantics.html)
- [syntax.html](https://wpt.fyi/results/css/selectors/attribute-selectors/attribute-case/syntax.html) [(live test)](http://wpt.live/css/selectors/attribute-selectors/attribute-case/syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/attribute-selectors/attribute-case/syntax.html)

By default case-sensitivity of attribute names and values in selectors depends on the document language.

<a id="ref-for-ascii-case-insensitive③"></a>

To match attribute values [ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive) regardless of document language rules, the attribute selector may include the identifier `i` before the closing bracket (`]`). When this flag is present, UAs must match the attribute’s value <a id="ref-for-ascii-case-insensitive④"></a>ASCII case-insensitively (i.e. \[a-z\] and \[A-Z\] are considered equivalent).

<a id="ref-for-string-is①"></a>

Alternately, the attribute selector may include the identifier `s` before the closing bracket (`]`); in this case the UA must match the value case-sensitively, with “[identical to](https://infra.spec.whatwg.org/#string-is)” semantics [\[INFRA\]](#biblio-infra), regardless of document language rules.

<a id="ref-for-ascii-case-insensitive⑤"></a>

Like the rest of Selectors syntax, the `i` and `s` identifiers themselves are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-348bad1b"></a> The following rule will style the `frame` attribute when it has a value of `hsides`, whether that value is represented as `hsides`, `HSIDES`, `hSides`, etc. even in an XML environment where attribute values are case-sensitive.
>
> ```text
> [frame=hsides i] { border-style: solid none; } 
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f596236d"></a> The following rule will style lists with `type="a"` attributes differently than `type="A"` even though HTML defines the `type` attribute to be case-insensitive.
>
> ```text
> [type="a" s] { list-style: lower-alpha; }
> [type="A" s] { list-style: upper-alpha; }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some document models normalize case-insensitive attribute values at parse time such that checking if a string is case-sensitive matching is impossible. Case-sensitive matching via `s` flags is only possible in systems that preserve the original case.

### <a id="attrnmsp"></a>6.4.  Attribute selectors and namespaces

Tests

- [selectors-namespace-001.xml](https://wpt.fyi/results/css/selectors/selectors-namespace-001.xml) [(live test)](http://wpt.live/css/selectors/selectors-namespace-001.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-namespace-001.xml)

<a id="ref-for-css-qualified-name②"></a>

<a id="ref-for-nsdecl④"></a>

The attribute name in an attribute selector is given as a [CSS qualified name](https://www.w3.org/TR/css-namespaces-3/#css-qualified-name): a namespace prefix that has been previously [declared](#nsdecl) may be prepended to the attribute name separated by the namespace separator "vertical bar" (`|`). In keeping with the Namespaces in the XML recommendation, default namespaces do not apply to attributes, therefore attribute selectors without a namespace component apply only to attributes that have no namespace (equivalent to \|attr). An asterisk may be used for the namespace prefix indicating that the selector is to match all attribute names without regard to the attribute’s namespace.

<a id="ref-for-nsdecl①"></a>

An attribute selector with an attribute name containing a namespace prefix that has not been previously [declared](#nsdecl) is an invalid selector.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a7ca2cb1"></a> CSS examples:
>
> ```text
> @namespace foo "http://www.example.com";
> [foo|att=val] { color: blue }
> [*|att] { color: yellow }
> [|att] { color: green }
> [att] { color: green }
> ```
>
> The first rule will match only elements with the attribute `att` in the "http://www.example.com" namespace with the value "val".
>
> The second rule will match only elements with the attribute `att` regardless of the namespace of the attribute (including no namespace).
>
> The last two rules are equivalent and will match only elements with the attribute `att` where the attribute is not in a namespace.

### <a id="def-values"></a>6.5.  Default attribute values in DTDs

Attribute selectors represent attribute values in the document tree. How that document tree is constructed is outside the scope of Selectors. In some document formats default attribute values can be defined in a DTD or elsewhere, but these can only be selected by attribute selectors if they appear in the document tree. Selectors should be designed so that they work whether or not the default values are included in the document tree.

For example, a XML UA may, but is <em>not</em> required to, read an “external subset” of the DTD, but <em>is</em> required to look for default attribute values in the document’s “internal subset”. (See, e.g., [\[XML10\]](#biblio-xml10) for definitions of these subsets.) Depending on the UA, a default attribute value defined in the external subset of the DTD might or might not appear in the document tree.

A UA that recognizes an XML namespace may, but is not required to use its knowledge of that namespace to treat default attribute values as if they were present in the document. (For example, an XHTML UA is not required to use its built-in knowledge of the XHTML DTD. See, e.g., [\[XML-NAMES\]](#biblio-xml-names) for details on namespaces in XML 1.0.)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Typically, implementations choose to ignore external subsets. This corresponds to the behavior of non-validating processors as defined by the XML specification.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ba3e8ad8"></a> Example:
>
> Consider an element `EXAMPLE` with an attribute `radix` that has a default value of `"decimal"`. The DTD fragment might be
>
> ```text
> <!ATTLIST EXAMPLE radix (decimal,octal) "decimal"> 
> ```
>
> If the style sheet contains the rules
>
> ```text
> EXAMPLE[radix=decimal] { /*... default property settings ...*/ }
> EXAMPLE[radix=octal]   { /*... other settings...*/ }
> ```
>
> the first rule might not match elements whose `radix` attribute is set by default, i.e. not set explicitly. To catch all cases, the attribute selector for the default value must be dropped:
>
> ```text
> EXAMPLE                { /*... default property settings ...*/ }
> EXAMPLE[radix=octal]   { /*... other settings...*/ }
> ```
>
> Here, because the selector EXAMPLE\[radix=octal\] is more specific than the type selector alone, the style declarations in the second rule will override those in the first for elements that have a `radix` attribute value of `"octal"`. Care has to be taken that all property declarations that are to apply only to the default case are overridden in the non-default cases' style rules.

### <a id="class-html"></a>6.6.  Class selectors

Tests

- [parse-class.html](https://wpt.fyi/results/css/selectors/parsing/parse-class.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-class.html)
- [xml-class-selector.xml](https://wpt.fyi/results/css/selectors/xml-class-selector.xml) [(live test)](http://wpt.live/css/selectors/xml-class-selector.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/xml-class-selector.xml)

The <a id="class-selector"></a>class selector is given as a full stop (. U+002E) immediately followed by an identifier. It represents an element belonging to the class identified by the identifier, as defined by the document language. For example, in [\[HTML5\]](#biblio-html5), [\[SVG11\]](#biblio-svg11), and [\[MATHML\]](#biblio-mathml) membership in a class is given by the `class` attribute: in these languages it is equivalent to the `~=` notation applied to the local `class` attribute (i.e. <code>&#x5B;class&#x7E;=<var>identifier</var>&#x5D;</code>).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f9c08b5b"></a> CSS examples:
>
> We can assign style information to all elements with `class~="pastoral"` as follows:
>
> ```text
> *.pastoral { color: green }  /* all elements with class~=pastoral */ 
> ```
>
> or just
>
> ```text
> .pastoral { color: green }  /* all elements with class~=pastoral */ 
> ```
>
> The following assigns style only to H1 elements with `class~="pastoral"`:
>
> ```text
> H1.pastoral { color: green }  /* H1 elements with class~=pastoral */ 
> ```
>
> Given these rules, the first `H1` instance below would not have green text, while the second would:
>
> ```text
> <H1>Not green</H1>
> <H1 class="pastoral">Very green</H1>
> ```
>
> <a id="ref-for-the-p-element"></a>
>
> <a id="ref-for-whitespace④"></a>
>
> The following rule matches any [P](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element whose `class` attribute has been assigned a list of [whitespace](#whitespace)-separated values that includes both `pastoral` and `marine`:
>
> ```text
> p.pastoral.marine { color: green } 
> ```
>
> This rule matches when
>
> ```text
> class="pastoral blue aqua
> 		marine"
> ```
>
> but does not match for
>
> ```text
> class="pastoral
> 		blue"
> ```
>
> .

<a id="ref-for-the-div-element"></a>

<a id="ref-for-the-span-element②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because CSS gives considerable power to the "class" attribute, authors could conceivably design their own "document language" based on elements with almost no associated presentation (such as [div](https://html.spec.whatwg.org/multipage/grouping-content.html#the-div-element) and [span](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-span-element) in HTML) and assigning style information through the "class" attribute. Authors should avoid this practice since the structural elements of a document language often have recognized and accepted meanings and author-defined classes may not.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If an element has multiple class attributes, their values must be concatenated with spaces between the values before searching for the class. As of this time the working group is not aware of any manner in which this situation can be reached, however, so this behavior is explicitly non-normative in this specification.

<a id="ref-for-concept-document-quirks"></a>

<a id="ref-for-ascii-case-insensitive⑥"></a>

<a id="ref-for-string-is②"></a>

When matching against a document which is in [quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks), class names must be matched [ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive); class selectors are otherwise case-sensitive, only matching class names they are [identical to](https://infra.spec.whatwg.org/#string-is). [\[INFRA\]](#biblio-infra)

### <a id="id-selectors"></a>6.7.  ID selectors

Tests

- [historical-xmlid.xht](https://wpt.fyi/results/css/selectors/historical-xmlid.xht) [(live test)](http://wpt.live/css/selectors/historical-xmlid.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/historical-xmlid.xht)
- [parse-id.html](https://wpt.fyi/results/css/selectors/parsing/parse-id.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-id.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-id.html)

Document languages may contain attributes that are declared to be of type ID. What makes attributes of type ID special is that no two such attributes can have the same value in a conformant document, regardless of the type of the elements that carry them; whatever the document language, an ID typed attribute can be used to uniquely identify its element. In HTML all ID attributes are named `id`; XML applications may name ID attributes differently, but the same restriction applies. Which attribute on an element is considered the “ID attribute” is defined by the document language.

An <a id="id-selector"></a>ID selector consists of a “number sign” (U+0023, `#`) immediately followed by the ID value, which must be a CSS [identifier](https://www.w3.org/TR/CSS21/syndata.html#value-def-identifier). An ID selector represents an element instance that has an identifier that matches the identifier in the ID selector. (It is possible in non-conforming documents for multiple elements to match a single ID selector.)

<a id="ref-for-the-h1-element⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3e8dc597"></a> Examples: The following ID selector represents an [h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element whose ID-typed attribute has the value "chapter1":
>
> ```text
> h1#chapter1 
> ```
>
> The following ID selector represents any element whose ID-typed attribute has the value "chapter1":
>
> ```text
> #chapter1 
> ```
>
> The following selector represents any element whose ID-typed attribute has the value "z98y".
>
> ```text
> *#z98y 
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In XML 1.0 [\[XML10\]](#biblio-xml10), the information about which attribute contains an element’s IDs is contained in a DTD or a schema. When parsing XML, UAs do not always read the DTD, and thus may not know what the ID of an element is (though a UA may have namespace-specific knowledge that allows it to determine which attribute is the ID attribute for that namespace). If a style sheet author knows or suspects that a UA may not know what the ID of an element is, they should use normal attribute selectors instead: \[name=p371\] instead of \#p371.

If an element has multiple ID attributes, all of them must be treated as IDs for that element for the purposes of the ID selector. Such a situation could be reached using mixtures of xml:id, DOM3 Core, XML DTDs, and namespace-specific knowledge.

<a id="ref-for-concept-document-quirks①"></a>

<a id="ref-for-ascii-case-insensitive⑦"></a>

<a id="ref-for-string-is③"></a>

When matching against a document which is in [quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks), IDs must be matched [ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive); ID selectors are otherwise case-sensitive, only matching IDs they are [identical to](https://infra.spec.whatwg.org/#string-is). [\[INFRA\]](#biblio-infra)

## <a id="linguistic-pseudos"></a>7.  Linguistic Pseudo-classes

<a id="ref-for-dir-pseudo"></a>

### <a id="the-dir-pseudo"></a>7.1.  The Directionality Pseudo-class: [:dir()](#dir-pseudo)

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

<a id="ref-for-document-language②"></a>

The <a id="dir-pseudo"></a>:dir() pseudo-class allows the author to write selectors that represent an element based on its directionality as determined by the [document language](#document-language). For example, [\[HTML5\]](#biblio-html5) defines [how to determine the directionality of an element](https://html.spec.whatwg.org/multipage/dom.html#the-directionality), based on a combination of the `dir` attribute, the surrounding text, and other factors. As another example, the `its:dir` and `dirRule` element of the Internationalization Tag Set [\[ITS20\]](#biblio-its20) are able to define the directionality of an element in [\[XML10\]](#biblio-xml10).

<a id="ref-for-dir-pseudo①"></a>

<a id="ref-for-propdef-direction"></a>

The [:dir()](#dir-pseudo) pseudo-class does not select based on stylistic states—for example, the CSS [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) property does not affect whether it matches.

<a id="ref-for-dir-pseudo②"></a>

The pseudo-class :dir(ltr) represents an element that has a directionality of left-to-right (`ltr`). The pseudo-class :dir(rtl) represents an element that has a directionality of right-to-left (`rtl`). The argument to [:dir()](#dir-pseudo) must be a single identifier, otherwise the selector is invalid. White space is optionally allowed between the identifier and the parentheses. Values other than `ltr` and `rtl` are not invalid, but do not match anything. (If a future markup spec defines other directionalities, then Selectors may be extended to allow corresponding values.)

The difference between :dir(C) and \[dir=C\] is that \[dir=C\] only performs a comparison against a given attribute on the element, while the :dir(C) pseudo-class uses the UAs knowledge of the document’s semantics to perform the comparison. For example, in HTML, the directionality of an element inherits so that a child without a `dir` attribute will have the same directionality as its closest ancestor with a valid `dir` attribute. As another example, in HTML, an element that matches \[dir=auto\] will match either :dir(ltr) or :dir(rtl) depending on the resolved directionality of the elements as determined by its contents. [\[HTML5\]](#biblio-html5)

<a id="ref-for-lang-pseudo④"></a>

### <a id="the-lang-pseudo"></a>7.2.  The Language Pseudo-class: [:lang()](#lang-pseudo)

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

<a id="ref-for-content-language"></a>

<a id="ref-for-language-range①"></a>

<a id="ref-for-lang-pseudo⑤"></a>

<a id="ref-for-typedef-ident"></a>

<a id="ref-for-string-value"></a>

If the document language specifies how the (human) [content language](https://www.w3.org/TR/css-text-4/#content-language) of an element is determined, it is possible to write selectors that represent an element based on its <a id="ref-for-content-language①"></a>content language. The <a id="lang-pseudo"></a>:lang() pseudo-class, which accepts a comma-separated list of one or more [language ranges](#language-range), represents an element whose <a id="ref-for-content-language②"></a>content language is one of the languages listed in its argument. Each <a id="language-range"></a>language range in [:lang()](#lang-pseudo) must be a valid CSS [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) or [\<string\>](https://www.w3.org/TR/css-values-4/#string-value). (Thus language ranges containing asterisks, for example, must be either correctly escaped or quoted as strings, e.g. :lang(&#x5C;\*-Latn) or :lang("\*-Latn").)

<a id="ref-for-content-language③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [content language](https://www.w3.org/TR/css-text-4/#content-language) of an element is defined by the document language.

<a id="ref-for-content-language④"></a>

<a id="ref-for-meta"></a>

For example, in HTML [\[HTML5\]](#biblio-html5), the [content language](https://www.w3.org/TR/css-text-4/#content-language) is determined by a combination of the `lang` attribute, information from [meta](https://html.spec.whatwg.org/multipage/semantics.html#meta) elements, and possibly also the protocol (e.g. from HTTP headers). XML languages can use the `xml:lang` attribute to indicate language information for an element. [\[XML10\]](#biblio-xml10)

<a id="ref-for-content-language⑤"></a>

<a id="ref-for-language-range②"></a>

The element’s [content language](https://www.w3.org/TR/css-text-4/#content-language) matches a [language range](#language-range) if its <a id="ref-for-content-language⑥"></a>content language, as represented in BCP 47 syntax, matches the given <a id="ref-for-language-range③"></a>language range in an <i>extended filtering</i> operation per [\[RFC4647\]](#biblio-rfc4647) Matching of Language Tags (section 3.3.2). Both the <a id="ref-for-content-language⑦"></a>content language and the <a id="ref-for-language-range④"></a>language range must be <i>canonicalized</i> and converted to <i>extlang form</i> as per section 4.5 of [\[RFC5646\]](https://www.w3.org/TR/2026/WD-selectors-4-20260122/#biblio-rfc5646) prior to the <i>extended filtering</i> operation. The matching is performed case-insensitively within the ASCII range.

<a id="ref-for-language-range⑤"></a>

The [language range](#language-range) does not need to be a valid language code to perform this comparison.

<a id="ref-for-language-range⑥"></a>

For this purpose, a wildcard [language range](#language-range) (`"*"`) does not match elements whose language is not tagged (e.g. `lang=""`), but does match elements whose language is tagged as undetermined (`lang=und`). A <a id="ref-for-language-range⑦"></a>language range consisting of an empty string (:lang("")) matches (only) elements whose language is not tagged.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is recommended that documents and protocols indicate language using codes from [\[BCP47\]](#biblio-bcp47) or its successor, and in the case of XML-based formats, by means of `xml:lang` attributes. [\[XML10\]](#biblio-xml10) See [“FAQ: Two-letter or three-letter language codes.”](https://www.w3.org/International/questions/qa-lang-2or3.html)

<a id="ref-for-the-q-element"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-91142d84"></a> Examples: The two following selectors represent an HTML document that is in Belgian French or German. The two next selectors represent [q](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-q-element) quotations in an arbitrary element in Belgian French or German.
>
> ```text
> html:lang(fr-be)
> html:lang(de)
> :lang(fr-be) > q
> :lang(de) > q
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: One difference between :lang(C) and the \|= operator is that the \|= operator only performs a comparison against a given attribute on the element, while the :lang(C) pseudo-class uses the UAs knowledge of the document’s semantics to perform the comparison.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c75c4e33"></a> In this HTML example, only the BODY matches \[lang\|=fr\] (because it has a LANG attribute) but both the BODY and the P match :lang(fr) (because both are in French). The P does not match the \[lang\|=fr\] because it does not have a LANG attribute.
>
> ```text
> <body lang=fr>
>   <p>Je suis français.</p>
> </body>
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c94037c6"></a> Another difference between :lang(C) and the \|= operator is that :lang(C) performs implicit wildcard matching.
>
> For example, :lang(de-DE) will match all of de-DE, de-DE-1996, de-Latn-DE, de-Latf-DE, de-Latn-DE-1996, whereas of those \[lang\|=de-DE\] will only match de-DE and de-DE-1996.
>
> To perform wildcard matching on the first subtag (the primary language), an asterisk must be used: \*-CH will match all of de-CH, it-CH, fr-CH, and rm-CH.
>
> To select against an element’s lang attribute value using this type of language range match, use both the attribute selector and language pseudo-class together, e.g. \[lang\]:lang(de-DE).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Wildcard language matching and comma-separated lists are new in Level 4.

## <a id="location"></a>8.  Location Pseudo-classes

<a id="ref-for-any-link-pseudo"></a>

### <a id="the-any-link-pseudo"></a>8.1.  The Hyperlink Pseudo-class: [:any-link](#any-link-pseudo)

Tests

- [any-link-attribute-removal.html](https://wpt.fyi/results/css/selectors/invalidation/any-link-attribute-removal.html) [(live test)](http://wpt.live/css/selectors/invalidation/any-link-attribute-removal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/any-link-attribute-removal.html)
- [any-link-pseudo.html](https://wpt.fyi/results/css/selectors/invalidation/any-link-pseudo.html) [(live test)](http://wpt.live/css/selectors/invalidation/any-link-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/any-link-pseudo.html)

<a id="ref-for-the-a-element⑧"></a>

<a id="ref-for-the-area-element"></a>

<a id="ref-for-attr-hyperlink-href"></a>

<a id="ref-for-link-pseudo"></a>

<a id="ref-for-visited-pseudo"></a>

The <a id="any-link-pseudo"></a>:any-link pseudo-class represents an element that acts as the source anchor of a hyperlink. For example, in [\[HTML5\]](#biblio-html5), any <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element">a</a></code> or <code><a href="https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element">area</a></code> elements with an <code><a href="https://html.spec.whatwg.org/multipage/links.html#attr-hyperlink-href">href</a></code> attribute are hyperlinks, and thus match `:any-link`. It matches an element if the element would match either [:link](#link-pseudo) or [:visited](#visited-pseudo), and is equivalent to :is(:link, :visited).

<a id="ref-for-link-pseudo①"></a>

<a id="ref-for-visited-pseudo①"></a>

### <a id="link"></a>8.2.  The Link History Pseudo-classes: [:link](#link-pseudo) and [:visited](#visited-pseudo)

Tests

- [caret-color-visited-inheritance.html](https://wpt.fyi/results/css/selectors/caret-color-visited-inheritance.html) [(live test)](http://wpt.live/css/selectors/caret-color-visited-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/caret-color-visited-inheritance.html)
- [text-emphasis-visited-inheritance.html](https://wpt.fyi/results/css/selectors/text-emphasis-visited-inheritance.html) [(live test)](http://wpt.live/css/selectors/text-emphasis-visited-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/text-emphasis-visited-inheritance.html)
- [text-fill-color-visited-inheritance.html](https://wpt.fyi/results/css/selectors/text-fill-color-visited-inheritance.html) [(live test)](http://wpt.live/css/selectors/text-fill-color-visited-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/text-fill-color-visited-inheritance.html)
- [text-stroke-color-visited-inheritance.html](https://wpt.fyi/results/css/selectors/text-stroke-color-visited-inheritance.html) [(live test)](http://wpt.live/css/selectors/text-stroke-color-visited-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/text-stroke-color-visited-inheritance.html)
- [visited-in-visited-compound.html](https://wpt.fyi/results/css/selectors/visited-in-visited-compound.html) [(live test)](http://wpt.live/css/selectors/visited-in-visited-compound.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/visited-in-visited-compound.html)
- [visited-inheritance.html](https://wpt.fyi/results/css/selectors/visited-inheritance.html) [(live test)](http://wpt.live/css/selectors/visited-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/visited-inheritance.html)
- [visited-nested.html](https://wpt.fyi/results/css/selectors/visited-nested.html) [(live test)](http://wpt.live/css/selectors/visited-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/visited-nested.html)
- [visited-part-crash.html](https://wpt.fyi/results/css/selectors/visited-part-crash.html) [(live test)](http://wpt.live/css/selectors/visited-part-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/visited-part-crash.html)

User agents commonly display unvisited [hyperlinks](#the-any-link-pseudo) differently from previously visited ones. Selectors provides the pseudo-classes <a id="link-pseudo"></a>:link and <a id="visited-pseudo"></a>:visited to distinguish them:

- <a id="ref-for-link-pseudo②"></a>

  The [:link](#link-pseudo) pseudo-class applies to links that have not yet been visited.

- <a id="ref-for-visited-pseudo②"></a>

  The [:visited](#visited-pseudo) pseudo-class applies once the link has been visited by the user.

The two states are mutually exclusive.

<a id="ref-for-link-pseudo③"></a>

After some amount of time, user agents may choose to return a visited link to the (unvisited) [:link](#link-pseudo) state.

<a id="ref-for-visited-pseudo③"></a>

The [:visited](#visited-pseudo) pseudo-class comes with obvious privacy implications—​letting random websites know what <em>other</em> websites you’ve visited can be problematic for a number of reasons—​and so user agents <em>must</em> preserve user privacy in their implementation of <a id="ref-for-visited-pseudo④"></a>:visited.

> <strong data-conversion-semantic="note">Note</strong>
>
> This specification intentionally does not specify exactly how to preserve user privacy in this regard, to allow for user agents to innovate in this space. The following methods are suggested, however:
>
> - <a id="ref-for-visited-pseudo⑤"></a>
>
>   <a id="ref-for-link-pseudo④"></a>
>
>   Have [:visited](#visited-pseudo) never match, so all links match [:link](#link-pseudo) instead.
>
> - <a id="ref-for-visited-pseudo⑥"></a>
>
>   Carefully track what history entries could have been observed by a given origin on their own, and only have links match [:visited](#visited-pseudo) if that visit would have been observable from the site’s origin. A possible specific approach for this is described in [Appendix C: Example Privacy-Preserving :visited Restrictions](#visited-privacy).
>
> - <a id="ref-for-visited-pseudo⑦"></a>
>
>   <a id="ref-for-dom-window-getcomputedstyle"></a>
>
>   <a id="ref-for-link-pseudo⑤"></a>
>
>   Allow links to match [:visited](#visited-pseudo) on any origin, but carefully restrict what styles they can apply and what information is returned by style-querying APIs like <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code>, to prevent sites from observing whether a link is styled with [:link](#link-pseudo) or <a id="ref-for-visited-pseudo⑧"></a>:visited. (This is documented at [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Privacy_and_the_:visited_selector), and was the historical approach browsers took, but is not perfect; there are several ways for a hostile page to still extract history information.)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b0f10727"></a> For example, the selector .footnote:visited would allow styling footnote links differently if they’ve been previously followed, allowing users of the page to know they might not need to click the footnote again.

<a id="ref-for-target-pseudo"></a>

### <a id="the-target-pseudo"></a>8.3.  The Target Pseudo-class: [:target](#target-pseudo)

<a id="ref-for-concept-url-fragment"></a>

In some document languages, the document’s URL can further point to specific elements <em>within</em> the document via the URL’s [fragment](https://url.spec.whatwg.org/#concept-url-fragment). The elements pointed to in this way are the target elements of the document.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b8c5336f"></a> In HTML the fragment points to the element in the page with the same ID. The url `https://example.com/index.html#section2`, for example, points to the element with `id="section2"` in the document at `https://example.com/index.html`.

The <a id="target-pseudo"></a>:target pseudo-class matches the document’s target elements. If the document’s URL has no fragment identifier, then the document has no target elements.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8a386e7c"></a> Example:
>
> ```text
> p.note:target 
> ```
>
> <a id="ref-for-the-p-element①"></a>
>
> This selector represents a <code><a href="https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element">p</a></code> element of class `note` that is the target element of the referring URL.

<a id="ref-for-target-pseudo①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-af403697"></a> CSS example: Here, the [:target](#target-pseudo) pseudo-class is used to make the target element red and place an image before it, if there is one:
>
> ```text
> :target { color : red }
> :target::before { content : url(target.png) }
> ```
<a id="ref-for-focus-within-pseudo"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification previously defined a :target-within pseudo-class, analogous to [:focus-within](#focus-within-pseudo). It was removed in favor of :has(:target), which should hopefully suffice to solve the same use-cases.

<a id="ref-for-scope-pseudo"></a>

### <a id="the-scope-pseudo"></a>8.4.  The Reference Element Pseudo-class: [:scope](#scope-pseudo)

Tests

- [scope-selector.html](https://wpt.fyi/results/css/selectors/scope-selector.html) [(live test)](http://wpt.live/css/selectors/scope-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/scope-selector.html)
- [scope-without-scoping.html](https://wpt.fyi/results/css/selectors/scope-without-scoping.html) [(live test)](http://wpt.live/css/selectors/scope-without-scoping.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/scope-without-scoping.html)

<a id="ref-for-scoping-root②"></a>

<a id="ref-for-documentfragment"></a>

In some contexts, selectors are matched with respect to one or more [scoping roots](#scoping-root), such as when calling the <code><a>querySelector()</a></code> method in [\[DOM\]](#biblio-dom). The <a id="scope-pseudo"></a>:scope pseudo-class represents this <a id="ref-for-scoping-root③"></a>scoping root, and may be either a true element or a virtual one (such as a <code><a href="https://dom.spec.whatwg.org/#documentfragment">DocumentFragment</a></code>).

<a id="ref-for-scoping-root④"></a>

<a id="ref-for-scope-pseudo①"></a>

<a id="ref-for-selectordef-host①"></a>

<a id="ref-for-concept-shadow-tree①"></a>

<a id="ref-for-root-pseudo"></a>

If there is no [scoping root](#scoping-root) then [:scope](#scope-pseudo) represents the root of the tree the element is in (equivalent to [:host](https://drafts.csswg.org/css-shadow-1/#selectordef-host) in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), or [:root](#root-pseudo) otherwise). Specifications intending for this pseudo-class to match specific elements rather than this tree root element must define their <a id="ref-for-scoping-root⑤"></a>scoping root(s).

<a id="ref-for-scoping-root⑥"></a>

<a id="ref-for-featureless⑨"></a>

<a id="ref-for-selector-subject③"></a>

A virtual [scoping root](#scoping-root) is some object representing the root of a document fragment, and can be used in selector patterns to represent other elements’ relationships to this <a id="ref-for-scoping-root⑦"></a>scoping root, acting as the parent of any root elements in the document fragment it represents. A virtual <a id="ref-for-scoping-root⑧"></a>scoping root is [featureless](#featureless) and cannot be the [subject of the selector](#selector-subject).

<a id="ref-for-documentfragment①"></a>

<a id="ref-for-dom-node-parentnode"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ef427647"></a> For example, if you have a <code><a href="https://dom.spec.whatwg.org/#documentfragment">DocumentFragment</a></code> `df`, then `df.querySelectorAll(":scope > .foo")` matches all the .foo elements that are "top-level" in the document fragment (those that have the document fragment as their <code><a href="https://dom.spec.whatwg.org/#dom-node-parentnode">parentNode</a></code>).
>
> <a id="ref-for-selector-subject④"></a>
>
> However, `df.querySelector(":scope")` will not match anything, as the document fragment itself can’t be the [subject of the selector](#selector-subject).

## <a id="useraction-pseudos"></a>9.  User Action Pseudo-classes

Interactive user interfaces sometimes change the rendering in response to user actions. Selectors provides several <a id="user-action-pseudo-class"></a>user action pseudo-classes for the selection of an element the user is acting on. (In non-interactive user agents, these pseudo-classes are valid, but never match any element.)

<a id="ref-for-user-action-pseudo-class②"></a>

The [user action pseudo-classes](#user-action-pseudo-class) are not mutually exclusive. An element can match several such pseudo-classes at the same time.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fc0139c4"></a> Examples:
>
> ```text
> a:hover   /* user hovers over the link */
> a:focus   /* user focuses the link     */
> 
> a:focus:hover
> /* user hovers over the link while it’s focused */
> ```
<a id="ref-for-user-action-pseudo-class③"></a>

<a id="ref-for-hover-pseudo①"></a>

<a id="ref-for-active-pseudo"></a>

<a id="ref-for-focus-within-pseudo①"></a>

<a id="ref-for-flat-tree"></a>

<a id="ref-for-document-top-layer"></a>

<a id="ref-for-concept-tree-root"></a>

Some [user action pseudo-classes](#user-action-pseudo-class), in addition to matching on the particular element with the property in question, match on that element’s ancestors as well: [:hover](#hover-pseudo), [:active](#active-pseudo), [:focus-within](#focus-within-pseudo). Specifically, if these match on a given element, they also match on the element’s [flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) ancestors, up to and including the first [top layer](https://www.w3.org/TR/css-position-4/#document-top-layer) element or the [root](https://dom.spec.whatwg.org/#concept-tree-root) element, whichever is encountered first.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The specifics of hit-testing, necessary to know when several of the pseudo-classes defined in this section apply, are not yet defined, but will be in the future.

<a id="ref-for-hover-pseudo②"></a>

### <a id="the-hover-pseudo"></a>9.1.  The Pointer Hover Pseudo-class: [:hover](#hover-pseudo)

Tests

- hover-001-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/hover-001-manual.html)
- [hover-002.html](https://wpt.fyi/results/css/selectors/hover-002.html) [(live test)](http://wpt.live/css/selectors/hover-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/hover-002.html)
- [remove-hovered-element.html](https://wpt.fyi/results/css/selectors/remove-hovered-element.html) [(live test)](http://wpt.live/css/selectors/remove-hovered-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/remove-hovered-element.html)

The <a id="hover-pseudo"></a>:hover pseudo-class applies while the user designates an element (or pseudo-element) with a pointing device, but does not necessarily activate it. For example, a visual user agent could apply this pseudo-class when the cursor (mouse pointer) hovers over a box generated by the element. Interactive user agents that cannot detect hovering due to hardware limitations (e.g., a pen device that does not detect hovering) are still conforming; the selector will simply never match in such a UA.

<a id="ref-for-hover-pseudo③"></a>

<a id="ref-for-flat-tree①"></a>

An element also matches [:hover](#hover-pseudo) if one of its descendants in the [flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) (including non-element nodes, such as text nodes) matches the above conditions.

<a id="ref-for-hover-pseudo④"></a>

<a id="ref-for-the-label-element"></a>

Document languages may define additional ways in which an element can match [:hover](#hover-pseudo). For example, [\[HTML5\]](#biblio-html5) defines a labeled control element as [matching `:hover`](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-hover) when its [label](https://html.spec.whatwg.org/multipage/forms.html#the-label-element) is hovered.

<a id="ref-for-hover-pseudo⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since the [:hover](#hover-pseudo) state can apply to an element because its child is designated by a pointing device, it is possible for <a id="ref-for-hover-pseudo⑥"></a>:hover to apply to an element that is not underneath the pointing device.

<a id="ref-for-active-pseudo①"></a>

### <a id="the-active-pseudo"></a>9.2.  The Activation Pseudo-class: [:active](#active-pseudo)

The <a id="active-pseudo"></a>:active pseudo-class applies while an element is being “activated” by the user, as defined by the host language; for example, while a hyperlink is being triggered.

<a id="ref-for-active-pseudo②"></a>

In addition, the [:active](#active-pseudo) pseudo-class applies while any generated box of <em>any</em> element (or pseudo-element) is being actively indicated by a pointing device (in the “down” state), e.g. between the time the user presses the primary mouse button and releases it, or while a finger is pressing on a touchscreen.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\[HTML5\]](#biblio-html5) defines [specific conditions for HTML elements to be activated](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-active) .

<a id="ref-for-active-pseudo③"></a>

<a id="ref-for-flat-tree②"></a>

An element also matches [:active](#active-pseudo) if one of its descendants in the [flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) (including non-element nodes, such as text nodes) matches the above conditions.

<a id="ref-for-focus-pseudo②"></a>

### <a id="the-focus-pseudo"></a>9.3.  The Input Focus Pseudo-class: [:focus](#focus-pseudo)

Tests

- [focus-display-none-001.html](https://wpt.fyi/results/css/selectors/focus-display-none-001.html) [(live test)](http://wpt.live/css/selectors/focus-display-none-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-display-none-001.html)
- [focus-in-focus-event-001.html](https://wpt.fyi/results/css/selectors/focus-in-focus-event-001.html) [(live test)](http://wpt.live/css/selectors/focus-in-focus-event-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-in-focus-event-001.html)
- [focus-in-focusin-event-001.html](https://wpt.fyi/results/css/selectors/focus-in-focusin-event-001.html) [(live test)](http://wpt.live/css/selectors/focus-in-focusin-event-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/focus-in-focusin-event-001.html)

The <a id="focus-pseudo"></a>:focus pseudo-class applies while an element (or pseudo-element) has the focus (accepts keyboard or other forms of input).

<a id="ref-for-focus-pseudo③"></a>

There may be document language or implementation specific limits on which elements can acquire [:focus](#focus-pseudo). For example, [\[HTML\]](#biblio-html) defines a list of [focusable areas](https://html.spec.whatwg.org/multipage/interaction.html#focusable-area).

<a id="ref-for-focus-pseudo④"></a>

<a id="ref-for-focus-within-pseudo②"></a>

Document languages may define additional ways in which an element can match [:focus](#focus-pseudo), except that the <a id="ref-for-focus-pseudo⑤"></a>:focus pseudo-class must not automatically propagate to the parent element—​see [:focus-within](#focus-within-pseudo) if matching on the parent is desired. (<a id="ref-for-focus-pseudo⑥"></a>:focus may still apply to the parent element if made to propagate due to other mechanisms, but not merely due to being the parent.)

<a id="ref-for-focus-pseudo⑦"></a>

<a id="ref-for-the-label-element①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-1416193e"></a> There’s a desire from authors to propagate [:focus](#focus-pseudo) from a form control to its associated <code><a href="https://html.spec.whatwg.org/multipage/forms.html#the-label-element">label</a></code> element; the main objection seems to be implementation difficulty. See [CSSWG issue (CSS)](https://github.com/w3c/csswg-drafts/issues/397) and [WHATWG issue (HTML)](https://github.com/whatwg/html/issues/1632).

<a id="ref-for-focus-visible-pseudo"></a>

### <a id="the-focus-visible-pseudo"></a>9.4. <a id="focus-ring-pseudo"></a><a id="the-focusring-pseudo"></a> The Focus-Indicated Pseudo-class: [:focus-visible](#focus-visible-pseudo)

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

<a id="ref-for-focus-pseudo⑧"></a>

<a id="ref-for-pseudo-class①⑧"></a>

While the [:focus](#focus-pseudo) [pseudo-class](#pseudo-class) always matches the currently-focused element, UAs only sometimes visibly <a id="indicate-focus"></a>indicate focus (such as by drawing a “focus ring”), instead using a variety of heuristics to visibly indicate the focus only when it would be most helpful to the user. The <a id="focus-visible-pseudo"></a>:focus-visible <a id="ref-for-pseudo-class①⑨"></a>pseudo-class matches a focused element (or pseudo-element) in these situations only, allowing authors to change the appearance of the focus indicator without changing <em>when</em> a focus indicator appears.

<a id="ref-for-focus-visible-pseudo①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c40d8011"></a> In this example, all focusable elements get a strong yellow outline on [:focus-visible](#focus-visible-pseudo), and links get both a yellow outline and a yellow background on <a id="ref-for-focus-visible-pseudo②"></a>:focus-visible. These styles are consistent throughout the page and are easily visible due to their bold styling, but do not appear unless the user is likely to need to understand where page focus is.
>
> ```text
> :root {
>   --focus-gold: #ffbf47;
> }
> 
> :focus-visible  {
>   outline: 3px solid var(--focus-gold);
> }
> 
> a:focus-visible {
>   background-color: var(--focus-gold);
> }
> ```
<a id="ref-for-indicate-focus"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-58f4807d"></a> User agents can choose their own heuristics for when to [indicate focus](#indicate-focus); however, the following (non-normative) suggestions can be used as a starting point for when to <a id="ref-for-indicate-focus①"></a>indicate focus on the currently focused element:
>
> - <a id="ref-for-indicate-focus②"></a>
>
>   If the user has expressed a preference (such as via a system preference or a browser setting) to always see a visible focus indicator, [indicate focus](#indicate-focus) regardless of any other factors. (Another option may be for the user agent to show its own focus indicator regardless of author styles.)
>
> - <a id="ref-for-the-input-element"></a>
>
>   <a id="ref-for-indicate-focus③"></a>
>
>   If the element which supports keyboard input (such as an <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code> element, or any other element that would triggers a virtual keyboard to be shown on focus if a physical keyboard were not present), [indicate focus](#indicate-focus).
>
> - <a id="ref-for-indicate-focus④"></a>
>
>   <a id="ref-for-focus-pseudo⑨"></a>
>
>   If the user interacts with the page via keyboard or some other non-pointing device, [indicate focus](#indicate-focus). (This means keyboard usage may change whether this pseudo-class matches even if it doesn’t affect [:focus](#focus-pseudo)).
>
> - <a id="ref-for-indicate-focus⑤"></a>
>
>   If the user interacts with the page via a pointing device (mouse, touchscreen, etc.) and the focused element does not support keyboard input, don’t [indicate focus](#indicate-focus).
>
> - <a id="ref-for-indicate-focus⑥"></a>
>
>   If the previously-focused element [indicated focus](#indicate-focus), and a script causes focus to move elsewhere, <a id="ref-for-indicate-focus⑦"></a>indicate focus on the newly focused element.
>
>   <a id="ref-for-indicate-focus⑧"></a>
>
>   Conversely, if the previously-focused element did not [indicate focus](#indicate-focus), and a script causes focus to move elsewhere, don’t <a id="ref-for-indicate-focus⑨"></a>indicate focus on the newly focused element.
>
> - <a id="ref-for-indicate-focus①⓪"></a>
>
>   If a newly-displayed element automatically gains focus (such as an action button in a freshly opened dialog), that element should [indicate focus](#indicate-focus).

<a id="ref-for-focus-visible-pseudo③"></a>

<a id="ref-for-focus-pseudo①⓪"></a>

User agents should also use [:focus-visible](#focus-visible-pseudo) to specify the default focus style, so that authors using <a id="ref-for-focus-visible-pseudo④"></a>:focus-visible will not also need to disable the default [:focus](#focus-pseudo) style.

<a id="ref-for-focus-within-pseudo③"></a>

### <a id="the-focus-within-pseudo"></a>9.5.  The Focus Container Pseudo-class: [:focus-within](#focus-within-pseudo)

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

<a id="ref-for-focus-pseudo①①"></a>

<a id="ref-for-flat-tree③"></a>

The <a id="focus-within-pseudo"></a>:focus-within pseudo-class applies to any element (or pseudo-element) for which the [:focus](#focus-pseudo) pseudo-class applies, as well as to an element (or pseudo-element) whose descendant in the [flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) (including non-element nodes, such as text nodes) matches the conditions for matching <a id="ref-for-focus-pseudo①②"></a>:focus.

## <a id="resource-pseudos"></a>10.  Resource State Pseudo-classes

The pseudo-classes in this section apply to elements that represent loaded resources, particularly images/videos, and allow authors to select them based on some quality of their state.

<a id="ref-for-selectordef-playing"></a>

<a id="ref-for-selectordef-paused"></a>

<a id="ref-for-selectordef-seeking"></a>

### <a id="video-state"></a>10.1.  Media Playback State: the [:playing](#selectordef-playing), [:paused](#selectordef-paused), and [:seeking](#selectordef-seeking) pseudo-classes

Tests

- [media-playback-state.html](https://wpt.fyi/results/css/selectors/media/media-playback-state.html) [(live test)](http://wpt.live/css/selectors/media/media-playback-state.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/media-playback-state.html)

The <a id="selectordef-playing"></a>:playing pseudo-class represents an element that is capable of being “played” or “paused”, when that element is “playing”. (This includes both when the element is explicitly playing, and when it’s temporarily stopped for some reason not connected to user intent, but will automatically resume when that reason is resolved, such as a “buffering” or “stalled” state.)

The <a id="selectordef-paused"></a>:paused pseudo-class represents an element that is capable of being “played” or “paused”, when that element is “paused” (i.e. <em>not</em> ”playing”). (This includes both an explicit “paused” state, and other non-playing states like “loaded, hasn’t been activated yet”, etc.)

<a id="ref-for-audio"></a>

<a id="ref-for-video"></a>

The <a id="selectordef-seeking"></a>:seeking pseudo-class represents an element that is capable of ”seeking” when that element is ”seeking”. (For the <code><a href="https://html.spec.whatwg.org/multipage/media.html#audio">audio</a></code> and <code><a href="https://html.spec.whatwg.org/multipage/media.html#video">video</a></code> elements of HTML, see [HTML § 4.8.11.9 Seeking](https://html.spec.whatwg.org/multipage/media.html#seeking).)

<a id="ref-for-selectordef-buffering"></a>

<a id="ref-for-selectordef-stalled"></a>

### <a id="media-loading-state"></a>10.2.  Media Loading State: the [:buffering](#selectordef-buffering) and [:stalled](#selectordef-stalled) pseudo-classes

Tests

- [media-loading-state.html](https://wpt.fyi/results/css/selectors/media/media-loading-state.html) [(live test)](http://wpt.live/css/selectors/media/media-loading-state.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/media-loading-state.html)
- [media-loading-state-timing.sub.html](https://wpt.fyi/results/css/selectors/media/media-loading-state-timing.sub.html) [(live test)](http://wpt.live/css/selectors/media/media-loading-state-timing.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/media-loading-state-timing.sub.html)
- [media-playback-state-timing.html](https://wpt.fyi/results/css/selectors/media/media-playback-state-timing.html) [(live test)](http://wpt.live/css/selectors/media/media-playback-state-timing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/media-playback-state-timing.html)

<a id="ref-for-media-data"></a>

<a id="ref-for-selectordef-buffering①"></a>

<a id="ref-for-selectordef-playing①"></a>

The <a id="selectordef-buffering"></a>:buffering pseudo-class represents an element that is capable of being “played” or “paused”, when that element cannot continue playing because it is actively attempting to obtain [media data](https://html.spec.whatwg.org/multipage/media.html#media-data) but has not yet obtained enough data to resume playback. (Note that the element is still considered to be “playing” when it is “buffering”. Whenever [:buffering](#selectordef-buffering) matches an element, [:playing](#selectordef-playing) also matches the element.)

<a id="ref-for-media-data①"></a>

<a id="ref-for-audio①"></a>

<a id="ref-for-video①"></a>

<a id="ref-for-stall-timeout"></a>

<a id="ref-for-selectordef-buffering②"></a>

<a id="ref-for-selectordef-stalled①"></a>

<a id="ref-for-selectordef-playing②"></a>

The <a id="selectordef-stalled"></a>:stalled pseudo-class represents an element when that element cannot continue playing because it is actively attempting to obtain [media data](https://html.spec.whatwg.org/multipage/media.html#media-data) but it has failed to receive any data for some amount of time. For the <code><a href="https://html.spec.whatwg.org/multipage/media.html#audio">audio</a></code> and <code><a href="https://html.spec.whatwg.org/multipage/media.html#video">video</a></code> elements of HTML, this amount of time is the [media element stall timeout](https://html.spec.whatwg.org/multipage/media.html#stall-timeout). [\[HTML\]](#biblio-html) (Note that, like with the [:buffering](#selectordef-buffering) pseudo-class, the element is still considered to be “playing” when it is “stalled”. Whenever [:stalled](#selectordef-stalled) matches an element, [:playing](#selectordef-playing) also matches the element.)

<a id="ref-for-selectordef-muted"></a>

<a id="ref-for-selectordef-volume-locked"></a>

### <a id="sound-state"></a>10.3.  Sound State: the [:muted](#selectordef-muted) and [:volume-locked](#selectordef-volume-locked) pseudo-classes

Tests

- [sound-state.html](https://wpt.fyi/results/css/selectors/media/sound-state.html) [(live test)](http://wpt.live/css/selectors/media/sound-state.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/media/sound-state.html)

<a id="ref-for-audio②"></a>

<a id="ref-for-video②"></a>

<a id="ref-for-concept-media-muted"></a>

The <a id="selectordef-muted"></a>:muted pseudo-class represents an element that is capable of making sound, but is currently “muted“ (forced silent). (For the <code><a href="https://html.spec.whatwg.org/multipage/media.html#audio">audio</a></code> and <code><a href="https://html.spec.whatwg.org/multipage/media.html#video">video</a></code> elements of HTML, see [muted](https://html.spec.whatwg.org/multipage/media.html#concept-media-muted). [\[HTML\]](#biblio-html))

<a id="ref-for-audio③"></a>

<a id="ref-for-video③"></a>

<a id="ref-for-effective-media-volume"></a>

The <a id="selectordef-volume-locked"></a>:volume-locked pseudo-class represents an element that is capable of making sound, and currently has its volume "locked" by the UA or the user, so the page author cannot change it. (For the <code><a href="https://html.spec.whatwg.org/multipage/media.html#audio">audio</a></code> and <code><a href="https://html.spec.whatwg.org/multipage/media.html#video">video</a></code> elements of HTML, see the algorithm for setting the element’s [effective media volume](https://html.spec.whatwg.org/multipage/media.html#effective-media-volume). [\[HTML\]](#biblio-html))

## <a id="display-state-pseudos"></a>11.  Element Display State Pseudo-classes

<a id="ref-for-selectordef-open"></a>

### <a id="open-state"></a>11.1.  Collapse State: the [:open](#selectordef-open) pseudo-class

Tests

- [open-pseudo.html](https://wpt.fyi/results/css/selectors/open-pseudo.html) [(live test)](http://wpt.live/css/selectors/open-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/open-pseudo.html)
- [details-open-pseudo-001.html](https://wpt.fyi/results/css/selectors/selectors-4/details-open-pseudo-001.html) [(live test)](http://wpt.live/css/selectors/selectors-4/details-open-pseudo-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/details-open-pseudo-001.html)
- [details-open-pseudo-002.html](https://wpt.fyi/results/css/selectors/selectors-4/details-open-pseudo-002.html) [(live test)](http://wpt.live/css/selectors/selectors-4/details-open-pseudo-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/details-open-pseudo-002.html)
- [details-open-pseudo-003.html](https://wpt.fyi/results/css/selectors/selectors-4/details-open-pseudo-003.html) [(live test)](http://wpt.live/css/selectors/selectors-4/details-open-pseudo-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-4/details-open-pseudo-003.html)

<!-- -->

- [input-element-pseudo-open.optional.html](https://wpt.fyi/results/css/css-pseudo/input-element-pseudo-open.optional.html) [(live test)](http://wpt.live/css/css-pseudo/input-element-pseudo-open.optional.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/input-element-pseudo-open.optional.html)

The <a id="selectordef-open"></a>:open pseudo-class represents an element that has both “open” and “closed” states, and which is currently in the “open” state.

<a id="ref-for-the-details-element"></a>

<a id="ref-for-the-select-element"></a>

<a id="ref-for-the-dialog-element"></a>

<a id="ref-for-the-input-element①"></a>

<a id="ref-for-the-dialog-element①"></a>

Exactly what “open” and “closed” mean is host-language specific, but exemplified by elements such as HTML’s <code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element">details</a></code>, <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element">select</a></code>, <code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element">dialog</a></code>, and <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code> elements, all of which can be toggled “open” to display more content (or any content at all, in the case of <code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element">dialog</a></code>).

<a id="ref-for-propdef-visibility"></a>

<a id="ref-for-propdef-display"></a>

<a id="ref-for-selectordef-open①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Being “open” or “closed” is a semantic state. An element not currently being displayed (for example, one that has [visibility: collapse](https://www.w3.org/TR/css-display-4/#propdef-visibility), or belongs to a [display: none](https://www.w3.org/TR/css-display-3/#propdef-display) subtree) can still be “open” and will match [:open](#selectordef-open).

<a id="ref-for-selectordef-open②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A ":closed" pseudo-class might be added in the future once the full set of things that support [:open](#selectordef-open) is known.

<a id="ref-for-selectordef-popover-open"></a>

### <a id="popover-open-state"></a>11.2.  Popover State: the [:popover-open](#selectordef-popover-open) pseudo-class

Tests

- [popover-open-with-has-sibling-selector.html](https://wpt.fyi/results/css/selectors/popover-open-with-has-sibling-selector.html) [(live test)](http://wpt.live/css/selectors/popover-open-with-has-sibling-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/popover-open-with-has-sibling-selector.html)

The <a id="selectordef-popover-open"></a>:popover-open pseudo-class represents an element that has both “popover-showing” and “popover-hidden” states and which is currently in the “popover-showing” state.

<a id="ref-for-selectordef-open③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is distinct from [:open](#selectordef-open) because an element can have element-specific open and closed states and also have separate popover-showing and popover-hidden states associated with being a popover.

<a id="ref-for-attr-popover"></a>

<a id="ref-for-popover-visibility-state"></a>

Exactly what “popover-showing” and “popover-hidden” states mean is host-language specific, but is exemplified by the presence of the HTML <code><a href="https://html.spec.whatwg.org/multipage/popover.html#attr-popover">popover</a></code> attribute and the associated [popover visibility state](https://html.spec.whatwg.org/multipage/popover.html#popover-visibility-state).

<a id="ref-for-selectordef-modal"></a>

### <a id="modal-state"></a>11.3.  Modal (Exclusive Interaction) State: the [:modal](#selectordef-modal) pseudo-class

Tests

- [modal-pseudo-class.html](https://wpt.fyi/results/css/selectors/modal-pseudo-class.html) [(live test)](http://wpt.live/css/selectors/modal-pseudo-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/modal-pseudo-class.html)

<a id="ref-for-selectordef-modal①"></a>

The <a id="selectordef-modal"></a>:modal pseudo-class represents an element which is in a state that excludes all interaction with elements outside it until it has been dismissed. Multiple elements can be [:modal](#selectordef-modal) simultaneously, with only one of them active (able to receive input).

<a id="ref-for-the-dialog-element②"></a>

<a id="ref-for-selectordef-modal②"></a>

<a id="ref-for-dom-dialog-showmodal"></a>

<a id="ref-for-selectordef-fullscreen"></a>

<a id="ref-for-dom-element-requestfullscreen"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-82b2b6f1"></a> For example, the <code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element">dialog</a></code> element is [:modal](#selectordef-modal) when opened with the <code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-showmodal">showModal()</a></code> API. Similarly, a [:fullscreen](#selectordef-fullscreen) element is also <a id="ref-for-selectordef-modal③"></a>:modal when opened with the <code><a href="https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen">requestFullscreen()</a></code> API, since this prevents interaction with the rest of the page.

<a id="ref-for-selectordef-fullscreen①"></a>

### <a id="fullscreen-state"></a>11.4.  Fullscreen Presentation State: the [:fullscreen](#selectordef-fullscreen) pseudo-class

The <a id="selectordef-fullscreen"></a>:fullscreen pseudo-class represents an element which is displayed in a mode that takes up most (usually all) of the screen, such as that defined by the Fullscreen API. [\[FULLSCREEN\]](#biblio-fullscreen)

<a id="ref-for-selectordef-picture-in-picture"></a>

### <a id="pip-state"></a>11.5.  Picture-in-Picture Presentation State: the [:picture-in-picture](#selectordef-picture-in-picture) pseudo-class

The <a id="selectordef-picture-in-picture"></a>:picture-in-picture pseudo-class represents an element which is displayed in a mode that takes up most (usually all) of the viewport, and whose viewport is confined to part of the screen while being displayed over other content, for example when using the Picture-in-Picture API. [\[picture-in-picture\]](#biblio-picture-in-picture)

## <a id="input-pseudos"></a>12.  The Input Pseudo-classes

<a id="ref-for-the-input-element②"></a>

The pseudo-classes in this section mostly apply to elements that take user input, such as HTML’s [input](https://html.spec.whatwg.org/multipage/input.html#the-input-element) element.

### <a id="input-states"></a>12.1.  Input Control States

<a id="ref-for-enabled-pseudo"></a>

<a id="ref-for-disabled-pseudo"></a>

#### <a id="enableddisabled"></a>12.1.1.  The [:enabled](#enabled-pseudo) and [:disabled](#disabled-pseudo) Pseudo-classes

Tests

- [enabled-disabled.html](https://wpt.fyi/results/css/selectors/invalidation/enabled-disabled.html) [(live test)](http://wpt.live/css/selectors/invalidation/enabled-disabled.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/enabled-disabled.html)
- [pseudo-enabled-disabled.html](https://wpt.fyi/results/css/selectors/pseudo-enabled-disabled.html) [(live test)](http://wpt.live/css/selectors/pseudo-enabled-disabled.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/pseudo-enabled-disabled.html)

The <a id="enabled-pseudo"></a>:enabled pseudo-class represents user interface elements that are in an enabled state; such elements must have a corresponding disabled state.

Conversely, the <a id="disabled-pseudo"></a>:disabled pseudo-class represents user interface elements that are in a disabled state; such elements must have a corresponding enabled state.

<a id="ref-for-enabled-pseudo①"></a>

<a id="ref-for-disabled-pseudo①"></a>

What constitutes an enabled state, a disabled state, and a user interface element is host-language-dependent. In a typical document most elements will be neither [:enabled](#enabled-pseudo) nor [:disabled](#disabled-pseudo). For example, [\[HTML5\]](#biblio-html5) defines [non-disabled interactive elements](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-enabled) to be <a id="ref-for-enabled-pseudo②"></a>:enabled, and any such elements that are [explicitly disabled](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-enabled) to be <a id="ref-for-disabled-pseudo②"></a>:disabled.

<a id="ref-for-enabled-pseudo③"></a>

<a id="ref-for-disabled-pseudo③"></a>

<a id="ref-for-propdef-display①"></a>

<a id="ref-for-propdef-visibility①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: CSS properties that might affect a user’s ability to interact with a given user interface element do not affect whether it matches [:enabled](#enabled-pseudo) or [:disabled](#disabled-pseudo); e.g., the [display](https://www.w3.org/TR/css-display-3/#propdef-display) and [visibility](https://www.w3.org/TR/css-display-4/#propdef-visibility) properties have no effect on the enabled/disabled state of an element.

<a id="ref-for-read-only-pseudo"></a>

<a id="ref-for-read-write-pseudo"></a>

#### <a id="rw-pseudos"></a>12.1.2.  The Mutability Pseudo-classes: [:read-only](#read-only-pseudo) and [:read-write](#read-write-pseudo)

Tests

- [selector-read-write-type-change-001.html](https://wpt.fyi/results/css/selectors/selector-read-write-type-change-001.html) [(live test)](http://wpt.live/css/selectors/selector-read-write-type-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-read-write-type-change-001.html)
- [selector-read-write-type-change-002.html](https://wpt.fyi/results/css/selectors/selector-read-write-type-change-002.html) [(live test)](http://wpt.live/css/selectors/selector-read-write-type-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-read-write-type-change-002.html)

An element matches <a id="read-write-pseudo"></a>:read-write if it is user-alterable, as defined by the document language. Otherwise, it is <a id="read-only-pseudo"></a>:read-only.

<a id="ref-for-read-write-pseudo①"></a>

For example, in [\[HTML5\]](#biblio-html5) a [non-disabled non-readonly `<input>` element](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-read-only) is [:read-write](#read-write-pseudo), as is any element with the `contenteditable` attribute set to the true state.

<a id="ref-for-placeholder-shown-pseudo"></a>

#### <a id="placeholder"></a>12.1.3.  The Placeholder-shown Pseudo-class: [:placeholder-shown](#placeholder-shown-pseudo)

Tests

- [placeholder-shown.html](https://wpt.fyi/results/css/selectors/invalidation/placeholder-shown.html) [(live test)](http://wpt.live/css/selectors/invalidation/placeholder-shown.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/placeholder-shown.html)
- [placeholder-shown.html](https://wpt.fyi/results/css/selectors/placeholder-shown.html) [(live test)](http://wpt.live/css/selectors/placeholder-shown.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/placeholder-shown.html)
- [selector-placeholder-shown-emptify-placeholder.html](https://wpt.fyi/results/css/selectors/selector-placeholder-shown-emptify-placeholder.html) [(live test)](http://wpt.live/css/selectors/selector-placeholder-shown-emptify-placeholder.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-placeholder-shown-emptify-placeholder.html)
- [selector-placeholder-shown-type-change-001.html](https://wpt.fyi/results/css/selectors/selector-placeholder-shown-type-change-001.html) [(live test)](http://wpt.live/css/selectors/selector-placeholder-shown-type-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-placeholder-shown-type-change-001.html)
- [selector-placeholder-shown-type-change-002.html](https://wpt.fyi/results/css/selectors/selector-placeholder-shown-type-change-002.html) [(live test)](http://wpt.live/css/selectors/selector-placeholder-shown-type-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-placeholder-shown-type-change-002.html)
- [selector-placeholder-shown-type-change-003.html](https://wpt.fyi/results/css/selectors/selector-placeholder-shown-type-change-003.html) [(live test)](http://wpt.live/css/selectors/selector-placeholder-shown-type-change-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-placeholder-shown-type-change-003.html)

Input elements can sometimes show placeholder text as a hint to the user on what to type in. See, for example, the `placeholder` attribute in [\[HTML5\]](#biblio-html5). The <a id="placeholder-shown-pseudo"></a>:placeholder-shown pseudo-class matches an input element that is showing such placeholder text, whether that text is given by an attribute or a real element, or is otherwise implied by the UA.

<a id="ref-for-attr-input-placeholder"></a>

<a id="ref-for-the-input-element③"></a>

<a id="ref-for-the-textarea-element"></a>

<a id="ref-for-placeholder-shown-pseudo①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-37912861"></a> For example, according to the semantics of [\[HTML\]](#biblio-html) the <code><a href="https://html.spec.whatwg.org/multipage/input.html#attr-input-placeholder">placeholder</a></code> attribute on the <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code> and <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element">textarea</a></code> elements provide placeholder text. The [:placeholder-shown](#placeholder-shown-pseudo) class thus applies whenever such placeholder text is shown.

<a id="ref-for-selectordef-autofill"></a>

#### <a id="autofill"></a>12.1.4.  The Automatic Input Pseudo-class: [:autofill](#selectordef-autofill)

The <a id="selectordef-autofill"></a>:autofill pseudo-class represents input elements that have been automatically filled by the user agent, and have not been subsequently altered by the user.

<a id="ref-for-default-pseudo"></a>

#### <a id="the-default-pseudo"></a>12.1.5.  The Default-option Pseudo-class: [:default](#default-pseudo)

The <a id="default-pseudo"></a>:default pseudo-class applies to the one or more UI elements that are the default among a set of similar elements. Typically applies to context menu items, buttons and select lists/menus.

<a id="ref-for-default-pseudo①"></a>

One example is the default submit button among a set of buttons. Another example is the default option from a popup menu. In a select-many group (such as for pizza toppings), multiple elements can match [:default](#default-pseudo). For example, [\[HTML5\]](#biblio-html5) defines that <a id="ref-for-default-pseudo②"></a>:default matches [the “default button” in a form, the initially-selected `<option>`(s) in a `<select>`, and a few other elements.](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-default)

### <a id="input-value-states"></a>12.2.  Input Value States

<a id="ref-for-checked-pseudo"></a>

<a id="ref-for-unchecked-pseudo"></a>

<a id="ref-for-indeterminate-pseudo"></a>

#### <a id="checked"></a>12.2.1. <a id="indeterminate"></a> The Selected-option Pseudo-classes: [:checked](#checked-pseudo), [:unchecked](#unchecked-pseudo), and [:indeterminate](#indeterminate-pseudo)

Radio and checkbox elements can be toggled by the user, and some menu items are “checked” when the user selects them. This state is reflected by the <a id="input-value-pseudo-classes"></a>input value pseudo-classes.

<a id="ref-for-checked-pseudo①"></a>

When such elements are toggled “on” the <a id="checked-pseudo"></a>:checked pseudo-class applies. For example, [\[HTML5\]](#biblio-html5) defines that [checked checkboxes, radio buttons, and selected `<option>` elements](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-checked) match [:checked](#checked-pseudo). Similarly, when such elements are toggled “off”, the <a id="unchecked-pseudo"></a>:unchecked pseudo-class applies.

<a id="ref-for-checked-pseudo②"></a>

<a id="ref-for-unchecked-pseudo①"></a>

<a id="ref-for-indeterminate-pseudo①"></a>

If an element that <em>could</em> match [:checked](#checked-pseudo) or [:unchecked](#unchecked-pseudo) is neither "on" nor "off", the <a id="indeterminate-pseudo"></a>:indeterminate pseudo-class applies. [:indeterminate](#indeterminate-pseudo) also matches elements which do not have a notion of being "checked", but whose "value" is still in an indeterminate state, such as a progress meter whose progress percentage is unknown.

These three pseudo-classes are mutually exclusive; an element can only match at most one of them at a time. (And might match none, if the concept of being "checked" doesn’t apply to them.) If the "indeterminate" concept is not mutually exclusive with the "checked"/"unchecked" concept in a host language (for example, in HTML a checkbox’s "indeterminate" state is a separate boolean from its "checked" state boolean), being indeterminate wins over being checked/unchecked for the purpose of matching these pseudo-classes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-33865490"></a> For example, in HTML:
>
> - <a id="ref-for-indeterminate-pseudo②"></a>
>
>   <a id="ref-for-dom-input-indeterminate"></a>
>
>   <a id="ref-for-checked-pseudo③"></a>
>
>   <a id="ref-for-unchecked-pseudo②"></a>
>
>   <a id="ref-for-dom-input-checked"></a>
>
>   Checkboxes match [:indeterminate](#indeterminate-pseudo) if their <code><a href="https://html.spec.whatwg.org/multipage/input.html#dom-input-indeterminate">indeterminate</a></code> property is true; otherwise, they match [:checked](#checked-pseudo) or [:unchecked](#unchecked-pseudo) depending on their <code><a href="https://html.spec.whatwg.org/multipage/input.html#dom-input-checked">checked</a></code> property.
>
> - <a id="ref-for-checked-pseudo④"></a>
>
>   <a id="ref-for-dom-input-checked①"></a>
>
>   <a id="ref-for-unchecked-pseudo③"></a>
>
>   <a id="ref-for-dom-input-checked②"></a>
>
>   <a id="ref-for-indeterminate-pseudo③"></a>
>
>   Radio buttons match [:checked](#checked-pseudo) if their <code><a href="https://html.spec.whatwg.org/multipage/input.html#dom-input-checked">checked</a></code> property is true. Otherwise, they match [:unchecked](#unchecked-pseudo) if they’re in a radio group that contains a <a id="ref-for-checked-pseudo⑤"></a>:checked radio button. Otherwise (their <code><a href="https://html.spec.whatwg.org/multipage/input.html#dom-input-checked">checked</a></code> property is false, but they’re alone or in a radio group with no checked radio button), they match [:indeterminate](#indeterminate-pseudo).
>
> - <a id="ref-for-the-option-element"></a>
>
>   <a id="ref-for-the-select-element①"></a>
>
>   <a id="ref-for-checked-pseudo⑥"></a>
>
>   <a id="ref-for-unchecked-pseudo④"></a>
>
>   <a id="ref-for-dom-option-selected"></a>
>
>   <a id="ref-for-indeterminate-pseudo④"></a>
>
>   <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-option-element">option</a></code> elements within a <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element">select</a></code> match [:checked](#checked-pseudo) or [:unchecked](#unchecked-pseudo) depending on their <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#dom-option-selected">selected</a></code> property. (They’re never [:indeterminate](#indeterminate-pseudo).)
>
> - <a id="ref-for-the-progress-element"></a>
>
>   <a id="ref-for-indeterminate-pseudo⑤"></a>
>
>   <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-progress-element">progress</a></code> elements match [:indeterminate](#indeterminate-pseudo) if they lack a `value` attribute

<a id="ref-for-input-value-pseudo-classes"></a>

While the [input value pseudo-classes](#input-value-pseudo-classes) are dynamic in nature, and can altered by user action, since they can also be based on the presence of semantic attributes in the document (such as the `selected` and `checked` attributes in [\[HTML5\]](#biblio-html5)), they apply to all media.

### <a id="ui-validity"></a>12.3.  Input Value-checking

<a id="ref-for-valid-pseudo①"></a>

<a id="ref-for-invalid-pseudo"></a>

#### <a id="validity-pseudos"></a>12.3.1.  The Validity Pseudo-classes: [:valid](#valid-pseudo) and [:invalid](#invalid-pseudo)

<a id="ref-for-valid-pseudo②"></a>

<a id="ref-for-invalid-pseudo①"></a>

An element is <a id="valid-pseudo"></a>:valid or <a id="invalid-pseudo"></a>:invalid when its contents or value is, respectively, valid or invalid with respect to data validity semantics defined by the document language (e.g. [\[XFORMS11\]](#biblio-xforms11) or [\[HTML5\]](#biblio-html5)). An element which lacks data validity semantics is neither [:valid](#valid-pseudo) nor [:invalid](#invalid-pseudo).

<a id="ref-for-valid-pseudo③"></a>

<a id="ref-for-invalid-pseudo②"></a>

<a id="ref-for-the-p-element②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There is a difference between an element which has no constraints, and thus would always be [:valid](#valid-pseudo), and one which has no data validity semantics at all, and thus is neither <a id="ref-for-valid-pseudo④"></a>:valid nor [:invalid](#invalid-pseudo). In HTML, for example, an `<input type="text">` element may have no constraints, but a [p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element has no validity semantics at all, and so it never matches either of these pseudo-classes.

<a id="ref-for-in-range-pseudo"></a>

<a id="ref-for-out-of-range-pseudo"></a>

#### <a id="range-pseudos"></a>12.3.2.  The Range Pseudo-classes: [:in-range](#in-range-pseudo) and [:out-of-range](#out-of-range-pseudo)

<a id="ref-for-in-range-pseudo①"></a>

<a id="ref-for-out-of-range-pseudo①"></a>

The <a id="in-range-pseudo"></a>:in-range and <a id="out-of-range-pseudo"></a>:out-of-range pseudo-classes apply only to elements that have range limitations. An element is [:in-range](#in-range-pseudo) or [:out-of-range](#out-of-range-pseudo) when the value that the element is bound to is in range or out of range with respect to its range limits as defined by the document language. An element that lacks data range limits or is not a form control is neither <a id="ref-for-in-range-pseudo②"></a>:in-range nor <a id="ref-for-out-of-range-pseudo②"></a>:out-of-range. E.g. a slider element with a value of 11 presented as a slider control that only represents the values from 1-10 is :out-of-range. Another example is a menu element with a value of "E" that happens to be presented in a popup menu that only has choices "A", "B" and "C".

<a id="ref-for-required-pseudo"></a>

<a id="ref-for-optional-pseudo"></a>

#### <a id="opt-pseudos"></a>12.3.3.  The Optionality Pseudo-classes: [:required](#required-pseudo) and [:optional](#optional-pseudo)

Tests

- [selector-required-type-change-001.html](https://wpt.fyi/results/css/selectors/selector-required-type-change-001.html) [(live test)](http://wpt.live/css/selectors/selector-required-type-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-required-type-change-001.html)
- [selector-required-type-change-002.html](https://wpt.fyi/results/css/selectors/selector-required-type-change-002.html) [(live test)](http://wpt.live/css/selectors/selector-required-type-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-required-type-change-002.html)
- [selector-required.html](https://wpt.fyi/results/css/selectors/selector-required.html) [(live test)](http://wpt.live/css/selectors/selector-required.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-required.html)

A form element is <a id="required-pseudo"></a>:required or <a id="optional-pseudo"></a>:optional if a value for it is, respectively, required or optional before the form it belongs to can be validly submitted. Elements that are not form elements are neither required nor optional.

<a id="ref-for-user-valid-pseudo"></a>

<a id="ref-for-user-invalid-pseudo"></a>

#### <a id="user-pseudos"></a>12.3.4.  The User-interaction Pseudo-classes: [:user-valid](#user-valid-pseudo) and [:user-invalid](#user-invalid-pseudo)

Tests

- [user-valid-user-invalid.html](https://wpt.fyi/results/css/selectors/invalidation/user-valid-user-invalid.html) [(live test)](http://wpt.live/css/selectors/invalidation/user-valid-user-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/user-valid-user-invalid.html)
- [user-invalid-form-submission-invalidation.html](https://wpt.fyi/results/css/selectors/user-invalid-form-submission-invalidation.html) [(live test)](http://wpt.live/css/selectors/user-invalid-form-submission-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/user-invalid-form-submission-invalidation.html)
- [user-invalid.html](https://wpt.fyi/results/css/selectors/user-invalid.html) [(live test)](http://wpt.live/css/selectors/user-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/user-invalid.html)
- [user-valid.html](https://wpt.fyi/results/css/selectors/user-valid.html) [(live test)](http://wpt.live/css/selectors/user-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/user-valid.html)
- [valid-invalid-form-fieldset.html](https://wpt.fyi/results/css/selectors/valid-invalid-form-fieldset.html) [(live test)](http://wpt.live/css/selectors/valid-invalid-form-fieldset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/valid-invalid-form-fieldset.html)

The <a id="user-invalid-pseudo"></a>:user-invalid and the <a id="user-valid-pseudo"></a>:user-valid pseudo-classes represent an element with incorrect or correct input, respectively, but only <em>after</em> the user has significantly interacted with it. Their purpose is to help the user identify mistakes in their input.

<a id="ref-for-user-invalid-pseudo①"></a>

<a id="ref-for-invalid-pseudo③"></a>

These pseudo-classes must at least match their respective elements after the user has attempted to submit the form and before the user has interacted again with the input element or reset the form. They may also match at other times, as would be appropriate for highlighting an error to the user. For example, [:user-invalid](#user-invalid-pseudo) could start matching an [:invalid](#invalid-pseudo) input element once the user has changed its value and focus has moved to another element; or stop matching only after the user has successfully corrected the input.

<a id="ref-for-user-invalid-pseudo②"></a>

<a id="ref-for-invalid-pseudo④"></a>

<a id="ref-for-user-valid-pseudo①"></a>

<a id="ref-for-valid-pseudo⑤"></a>

<a id="ref-for-selector-user-invalid"></a>

Host languages may define more precise matching rules or defer to platform conventions; otherwise the exact behavior is UA-defined. Regardless, the [:user-invalid](#user-invalid-pseudo) pseudo-class must only match [:invalid](#invalid-pseudo) elements; and the [:user-valid](#user-valid-pseudo) pseudo-class must only match [:valid](#valid-pseudo) elements. See the [HTML specification](https://html.spec.whatwg.org/multipage/semantics-other.html#selector-user-invalid) for the specific rules pertaining to HTML elements. [\[HTML\]](#biblio-html)

<a id="ref-for-the-input-element④"></a>

<a id="ref-for-invalid-pseudo⑤"></a>

<a id="ref-for-the-input-element⑤"></a>

<a id="ref-for-user-invalid-pseudo③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-81851d24"></a> For example, the <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code> element in the following document fragment would match [:invalid](#invalid-pseudo) as soon as the page is loaded (because the <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code>’s initial value violates its max constraint) but it won’t match [:user-invalid](#user-invalid-pseudo) until the user significantly interacts with the input field or attempts to submit the form it’s part of.
>
> ```text
> <form>
>   <label>
>     Volume:
>     <input name='vol' type=number min=0 max=10 value=11>
>   </label>
>   ...
> </form>
> ```
## <a id="structural-pseudos"></a>13.  Tree-Structural pseudo-classes

Tests

- [selector-structural-pseudo-root.html](https://wpt.fyi/results/css/selectors/selector-structural-pseudo-root.html) [(live test)](http://wpt.live/css/selectors/selector-structural-pseudo-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selector-structural-pseudo-root.html)

Selectors introduces the concept of <a id="structural-pseudo-classes"></a>structural pseudo-classes to permit selection based on extra information that lies in the document tree but cannot be represented by other simple selectors or combinators.

Standalone text and other non-element nodes are not counted when calculating the position of an element in the list of children of its parent. When calculating the position of an element in the list of children of its parent, the index numbering starts at 1.

<a id="ref-for-structural-pseudo-classes①"></a>

<a id="ref-for-pseudo-element②⑨"></a>

The [structural pseudo-classes](#structural-pseudo-classes) only apply to elements in the document tree; they must never match [pseudo-elements](#pseudo-element).

<a id="ref-for-root-pseudo①"></a>

### <a id="the-root-pseudo"></a>13.1.  [:root](#root-pseudo) pseudo-class

Tests

- [root-siblings.html](https://wpt.fyi/results/css/selectors/root-siblings.html) [(live test)](http://wpt.live/css/selectors/root-siblings.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/root-siblings.html)

The <a id="root-pseudo"></a>:root pseudo-class represents an element that is the root of the document.

<a id="ref-for-root-pseudo②"></a>

<a id="ref-for-document-element"></a>

<a id="ref-for-the-html-element"></a>

For example, in a DOM document, the [:root](#root-pseudo) pseudo-class matches the [document element](https://dom.spec.whatwg.org/#document-element). In HTML, this will be the <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-html-element">html</a></code> element (unless scripting has been used to modify the document).

<a id="ref-for-empty-pseudo"></a>

### <a id="the-empty-pseudo"></a>13.2.  [:empty](#empty-pseudo) pseudo-class

Tests

- [selectors-empty-001.xml](https://wpt.fyi/results/css/selectors/selectors-empty-001.xml) [(live test)](http://wpt.live/css/selectors/selectors-empty-001.xml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-empty-001.xml)

<a id="ref-for-white-space"></a>

The <a id="empty-pseudo"></a>:empty pseudo-class represents an element that has no children except, optionally, [document white space characters](https://www.w3.org/TR/css-text-4/#white-space). In terms of the document tree, only element nodes and content nodes (such as [\[DOM\]](#biblio-dom) text nodes, and entity references) whose data has a non-zero length must be considered as affecting emptiness; comments, processing instructions, and other nodes must not affect whether an element is considered empty or not.

<a id="ref-for-the-p-element③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c085b235"></a> Examples: p:empty is a valid representation of the <code><a href="https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element">p</a></code> elements in the following HTML fragment:
>
> ```text
> <p></p>
> <p>
> <p> </p>
> <p> <!-- comment --></p>
> ```
>
> div:empty is not a valid representation of the `<div>` elements in the following fragment:
>
> ```text
> <div>text</div>
> <div><p></p></div>
> <div>&nbsp;</div>
> <div><p>bla</p></div>
> <div>this is not <p>:empty</p></div>
> ```
<a id="ref-for-empty-pseudo①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In Level 2 and Level 3 of Selectors, [:empty](#empty-pseudo) did not match elements that contained only white space. This was changed so that that—​given white space is largely collapsible in HTML and is therefore used for source code formatting, and especially because elements with omitted end tags are likely to absorb such white space into their DOM text contents—​elements which authors perceive of as empty can be selected by this selector, as they expect.

### <a id="child-index"></a>13.3.  Child-indexed Pseudo-classes

Tests

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

<a id="ref-for-concept-tree-inclusive-sibling"></a>

The pseudo-classes defined in this section select elements based on their index amongst their [inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Selectors 3 described these selectors as selecting elements based on their index in the child list of their parents. (This description survives in the name of this very section, and the names of several of the pseudo-classes.) As there was no reason to exclude them from matching elements without parents, or with non-element parents, they have been rephrased to refer to an element’s relative index amongst its siblings.

<a id="ref-for-nth-child-pseudo"></a>

#### <a id="the-nth-child-pseudo"></a>13.3.1.  [:nth-child()](#nth-child-pseudo) pseudo-class

Tests

- [nth-of-invalid.html](https://wpt.fyi/results/css/selectors/nth-of-invalid.html) [(live test)](http://wpt.live/css/selectors/nth-of-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-of-invalid.html)
- [nth-child-of-attribute-crash.html](https://wpt.fyi/results/css/selectors/invalidation/crashtests/nth-child-of-attribute-crash.html) [(live test)](http://wpt.live/css/selectors/invalidation/crashtests/nth-child-of-attribute-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/crashtests/nth-child-of-attribute-crash.html)
- [nth-of-namespace-class-invalidation-crash.html](https://wpt.fyi/results/css/selectors/invalidation/nth-of-namespace-class-invalidation-crash.html) [(live test)](http://wpt.live/css/selectors/invalidation/nth-of-namespace-class-invalidation-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/nth-of-namespace-class-invalidation-crash.html)

<a id="ref-for-concept-tree-inclusive-sibling①"></a>

<a id="ref-for-selector-list⑧"></a>

<a id="ref-for-typedef-complex-real-selector-list①"></a>

The <a id="nth-child-pseudo"></a>:nth-child(<var>An+B</var> \[of <var>S</var>\]? ) pseudo-class notation represents elements that are among <var>An+B</var>th elements from the list composed of their [inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling) that match the [selector list](#selector-list) <var>S</var>, which is a [\<complex-real-selector-list\>](#typedef-complex-real-selector-list). If <var>S</var> is omitted, it defaults to \*\|\*.

The <var>An+B</var> notation and its interpretation are defined in [CSS Syntax 3 § 6 The An+B microsyntax](https://www.w3.org/TR/css-syntax-3/#anb-microsyntax); it represents any index <var>i</var> = <var>A</var><var>n</var> + <var>B</var> for any non-negative integer <var>n</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For these purposes, the list of elements is <b>1-indexed</b>; that is, the first child of an element has index 1, and will be matched by :nth-child(2n+1), because when `n=0` the expression evaluates to 1.

For example, this selector could address every other row in a table, and could be used to alternate the color of paragraph text in a cycle of four.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b8d691c9"></a> Examples:
>
> ```text
> :nth-child(even)   /* represents the 2nd, 4th, 6th, etc elements
> :nth-child(10n-1)  /* represents the 9th, 19th, 29th, etc elements */
> :nth-child(10n+9)  /* Same */
> :nth-child(10n+-1) /* Syntactically invalid, and would be ignored */
> ```
<a id="ref-for-nth-child-pseudo①"></a>

<a id="ref-for-complex⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The specificity of the [:nth-child()](#nth-child-pseudo) pseudo-class is the specificity of a single pseudo-class plus, if <var>S</var> is specified, the specificity of the most specific [complex selector](#complex) in <var>S</var>. See [§ 15 Calculating a selector’s specificity](#specificity-rules). Thus <var>S</var>:nth-child(<var>An+B</var>) and :nth-child(<var>An+B</var> of <var>S</var>) have the exact same specificity, although they do differ in behavior (see example below).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3c07f717"></a> By passing a selector argument, we can select the Nth element that matches that selector. For example, the following selector matches the first three “important” list items, denoted by the .important class:
>
> ```text
> :nth-child(-n+3 of li.important)
> ```
>
> Note that this is different from moving the selector outside of the function, like:
>
> ```text
> li.important:nth-child(-n+3)
> ```
>
> This selector instead just selects the first three children if they also happen to be "important" list items.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-48f80a26"></a> Here’s another example of using the selector argument, to ensure that zebra-striping a table works correctly.
>
> Normally, to zebra-stripe a table’s rows, an author would use CSS similar to the following:
>
> ```text
> tr {
>   background: white;
> }
> tr:nth-child(even) {
>   background: silver;
> }
> ```
>
> However, if some of the rows are hidden and not displayed, this can break up the pattern, causing multiple adjacent rows to have the same background color. Assuming that rows are hidden with the \[hidden\] attribute in HTML, the following CSS would zebra-stripe the table rows robustly, maintaining a proper alternating background regardless of which rows are hidden:
>
> ```text
> tr {
>   background: white;
> }
> tr:nth-child(even of :not([hidden])) {
>   background: silver;
> }
> ```
<a id="ref-for-nth-last-child-pseudo"></a>

#### <a id="the-nth-last-child-pseudo"></a>13.3.2.  [:nth-last-child()](#nth-last-child-pseudo) pseudo-class

<a id="ref-for-concept-tree-inclusive-sibling②"></a>

<a id="ref-for-selector-list⑨"></a>

<a id="ref-for-typedef-complex-real-selector-list②"></a>

The <a id="nth-last-child-pseudo"></a>:nth-last-child(<var>An+B</var> \[of <var>S</var>\]? ) pseudo-class notation represents elements that are among <var>An+B</var>th elements from the list composed of their [inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling) that match the [selector list](#selector-list) <var>S</var>, counting backwards from the end. <var>S</var> is [\<complex-real-selector-list\>](#typedef-complex-real-selector-list). If <var>S</var> is omitted, it defaults to \*\|\*.

<a id="ref-for-nth-last-child-pseudo①"></a>

<a id="ref-for-nth-child-pseudo②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The specificity of the [:nth-last-child()](#nth-last-child-pseudo) pseudo-class, like the [:nth-child()](#nth-child-pseudo) pseudo-class, combines the specificity of a regular pseudo-class with that of its selector argument <var>S</var>. See [§ 15 Calculating a selector’s specificity](#specificity-rules).

The CSS Syntax Module [\[CSS3SYN\]](#biblio-css3syn) defines the [<var>An+B</var> notation](https://drafts.csswg.org/css-syntax/#anb).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bee68ad4"></a> Examples:
>
> ```text
> tr:nth-last-child(-n+2)    /* represents the two last rows of an HTML table */
> 
> foo:nth-last-child(odd)    /* represents all odd foo elements in their parent element,
>                               counting from the last one */
> ```
<a id="ref-for-first-child-pseudo"></a>

#### <a id="the-first-child-pseudo"></a>13.3.3.  [:first-child](#first-child-pseudo) pseudo-class

Tests

- [first-child.html](https://wpt.fyi/results/css/selectors/first-child.html) [(live test)](http://wpt.live/css/selectors/first-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/first-child.html)
- [first-child-last-child.html](https://wpt.fyi/results/css/selectors/invalidation/first-child-last-child.html) [(live test)](http://wpt.live/css/selectors/invalidation/first-child-last-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/first-child-last-child.html)

<a id="ref-for-concept-tree-inclusive-sibling③"></a>

The <a id="first-child-pseudo"></a>:first-child pseudo-class represents an element that is first among its [inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling). Same as :nth-child(1).

<a id="ref-for-the-p-element④"></a>

<a id="ref-for-the-div-element①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-72934891"></a> Examples: The following selector represents a [p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element that is the first child of a [div](https://html.spec.whatwg.org/multipage/grouping-content.html#the-div-element) element:
>
> ```text
> div > p:first-child
> ```
>
> This selector can represent the `p` inside the `div` of the following fragment:
>
> ```text
> <p> The last P before the note.</p>
> <div class="note">
>    <p> The first P inside the note.</p>
> </div>
> ```
>
> but cannot represent the second `p` in the following fragment:
>
> ```text
> <p> The last P before the note.</p>
> <div class="note">
>    <h2> Note </h2>
>    <p> The first P inside the note.</p>
> </div>
> ```
>
> The following two selectors are usually equivalent:
>
> ```text
> * > a:first-child /* a is first child of any element */
> a:first-child /* Same (assuming a is not the root element) */
> ```
<a id="ref-for-last-child-pseudo"></a>

#### <a id="the-last-child-pseudo"></a>13.3.4.  [:last-child](#last-child-pseudo) pseudo-class

Tests

- [last-child.html](https://wpt.fyi/results/css/selectors/last-child.html) [(live test)](http://wpt.live/css/selectors/last-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/last-child.html)

<a id="ref-for-concept-tree-inclusive-sibling④"></a>

The <a id="last-child-pseudo"></a>:last-child pseudo-class represents an element that is last among its [inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling). Same as :nth-last-child(1).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-48137186"></a> Example: The following selector represents a list item `li` that is the last child of an ordered list `ol`.
>
> ```text
> ol > li:last-child
> ```
<a id="ref-for-only-child-pseudo"></a>

#### <a id="the-only-child-pseudo"></a>13.3.5.  [:only-child](#only-child-pseudo) pseudo-class

Tests

- [only-child.html](https://wpt.fyi/results/css/selectors/only-child.html) [(live test)](http://wpt.live/css/selectors/only-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/only-child.html)

The <a id="only-child-pseudo"></a>:only-child pseudo-class represents an element that has no siblings. Same as :first-child:last-child or :nth-child(1):nth-last-child(1), but with a lower specificity.

### <a id="typed-child-index"></a>13.4.  Typed Child-indexed Pseudo-classes

Tests

- [nth-of-type-namespace.html](https://wpt.fyi/results/css/selectors/nth-of-type-namespace.html) [(live test)](http://wpt.live/css/selectors/nth-of-type-namespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/nth-of-type-namespace.html)

The pseudo-classes in this section are similar to the [Child Index Pseudo-classes](#child-index), but they resolve based on an element’s index <strong>among elements of the same <a href="#type-selectors">type (tag name)</a></strong> in their sibling list.

<a id="ref-for-nth-of-type-pseudo"></a>

#### <a id="the-nth-of-type-pseudo"></a>13.4.1.  [:nth-of-type()](#nth-of-type-pseudo) pseudo-class

<a id="ref-for-type-selector①②"></a>

<a id="ref-for-the-img-element"></a>

<a id="ref-for-pseudo-class②⓪"></a>

The <a id="nth-of-type-pseudo"></a>:nth-of-type(<var>An+B</var>) pseudo-class notation represents the same elements that would be matched by :nth-child(\|An+B\| of <var>S</var>), where <var>S</var> is a [type selector](#type-selector) and namespace prefix matching the element in question. For example, when considering whether an HTML <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> element matches this [pseudo-class](#pseudo-class), the <var>S</var> in question is html\|img (assuming an appropriate `html` namespace is declared).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3d09bf4d"></a> CSS example: This allows an author to alternate the position of floated images:
>
> ```text
> img:nth-of-type(2n+1) { float: right; }
> img:nth-of-type(2n) { float: left; }
> ```
<a id="ref-for-nth-child-pseudo③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the type of the element is known ahead of time, this pseudo-class is equivalent to using [:nth-child()](#nth-child-pseudo) with a type selector. That is, img:nth-of-type(2) is equivalent to \*:nth-child(2 of img).

<a id="ref-for-nth-last-of-type-pseudo"></a>

#### <a id="the-nth-last-of-type-pseudo"></a>13.4.2.  [:nth-last-of-type()](#nth-last-of-type-pseudo) pseudo-class

Tests

- [last-of-type.html](https://wpt.fyi/results/css/selectors/last-of-type.html) [(live test)](http://wpt.live/css/selectors/last-of-type.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/last-of-type.html)

<a id="ref-for-type-selector①③"></a>

<a id="ref-for-the-img-element①"></a>

<a id="ref-for-pseudo-class②①"></a>

The <a id="nth-last-of-type-pseudo"></a>:nth-last-of-type(<var>An+B</var>) pseudo-class notation represents the same elements that would be matched by :nth-last-child(\|An+B\| of <var>S</var>), where <var>S</var> is a [type selector](#type-selector) and namespace prefix matching the element in question. For example, when considering whether an HTML <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> element matches this [pseudo-class](#pseudo-class), the <var>S</var> in question is html\|img (assuming an appropriate `html` namespace is declared).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b89418c8"></a> Example: To represent all `h2` children of an XHTML `body` except the first and last, one could use the following selector:
>
> ```text
> body > h2:nth-of-type(n+2):nth-last-of-type(n+2) 
> ```
>
> <a id="ref-for-negation-pseudo⑦"></a>
>
> In this case, one could also use [:not()](#negation-pseudo), although the selector ends up being just as long:
>
> ```text
> body > h2:not(:first-of-type):not(:last-of-type) 
> ```
<a id="ref-for-first-of-type-pseudo"></a>

#### <a id="the-first-of-type-pseudo"></a>13.4.3.  [:first-of-type](#first-of-type-pseudo) pseudo-class

Tests

- [first-of-type.html](https://wpt.fyi/results/css/selectors/first-of-type.html) [(live test)](http://wpt.live/css/selectors/first-of-type.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/first-of-type.html)
- [negated-always-matches-negated-first-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-always-matches-negated-first-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-always-matches-negated-first-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-always-matches-negated-first-of-type-when-ancestor-changes.html)
- [negated-first-of-type-in-nonsubject-position.html](https://wpt.fyi/results/css/selectors/invalidation/negated-first-of-type-in-nonsubject-position.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-first-of-type-in-nonsubject-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-first-of-type-in-nonsubject-position.html)
- [negated-is-always-matches-negated-first-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-is-always-matches-negated-first-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-is-always-matches-negated-first-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-is-always-matches-negated-first-of-type-when-ancestor-changes.html)
- [negated-is-never-matches-negated-first-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-is-never-matches-negated-first-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-is-never-matches-negated-first-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-is-never-matches-negated-first-of-type-when-ancestor-changes.html)
- [negated-negated-first-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-negated-first-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-negated-first-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-negated-first-of-type-when-ancestor-changes.html)
- [negated-never-matches-negated-first-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-never-matches-negated-first-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-never-matches-negated-first-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-never-matches-negated-first-of-type-when-ancestor-changes.html)
- [of-type-selectors.xhtml](https://wpt.fyi/results/css/selectors/of-type-selectors.xhtml) [(live test)](http://wpt.live/css/selectors/of-type-selectors.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/of-type-selectors.xhtml)

The <a id="first-of-type-pseudo"></a>:first-of-type pseudo-class represents the same element as :nth-of-type(1).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-16f73753"></a> Example: The following selector represents a definition title `dt` inside a definition list `dl`, this `dt` being the first of its type in the list of children of its parent element.
>
> ```text
> dl dt:first-of-type
> ```
>
> It is a valid description for the first two `dt` elements in the following example but not for the third one:
>
> ```text
> <dl>
>   <dt>gigogne</dt>
>   <dd>
>     <dl>
>       <dt>fusée</dt>
>       <dd>multistage rocket</dd>
>       <dt>table</dt>
>       <dd>nest of tables</dd>
>     </dl>
>   </dd>
> </dl>
> ```
<a id="ref-for-last-of-type-pseudo"></a>

#### <a id="the-last-of-type-pseudo"></a>13.4.4.  [:last-of-type](#last-of-type-pseudo) pseudo-class

Tests

- [negated-always-matches-negated-last-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-always-matches-negated-last-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-always-matches-negated-last-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-always-matches-negated-last-of-type-when-ancestor-changes.html)
- [negated-is-always-matches-negated-last-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-is-always-matches-negated-last-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-is-always-matches-negated-last-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-is-always-matches-negated-last-of-type-when-ancestor-changes.html)
- [negated-is-never-matches-negated-last-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-is-never-matches-negated-last-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-is-never-matches-negated-last-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-is-never-matches-negated-last-of-type-when-ancestor-changes.html)
- [negated-last-of-type-invalidation.html](https://wpt.fyi/results/css/selectors/invalidation/negated-last-of-type-invalidation.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-last-of-type-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-last-of-type-invalidation.html)
- [negated-negated-last-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-negated-last-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-negated-last-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-negated-last-of-type-when-ancestor-changes.html)
- [negated-never-matches-negated-last-of-type-when-ancestor-changes.html](https://wpt.fyi/results/css/selectors/invalidation/negated-never-matches-negated-last-of-type-when-ancestor-changes.html) [(live test)](http://wpt.live/css/selectors/invalidation/negated-never-matches-negated-last-of-type-when-ancestor-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/negated-never-matches-negated-last-of-type-when-ancestor-changes.html)

The <a id="last-of-type-pseudo"></a>:last-of-type pseudo-class represents the same element as :nth-last-of-type(1).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fee30007"></a> Example: The following selector represents the last data cell `td` of a table row `tr`.
>
> ```text
> tr > td:last-of-type
> ```
<a id="ref-for-only-of-type-pseudo"></a>

#### <a id="the-only-of-type-pseudo"></a>13.4.5.  [:only-of-type](#only-of-type-pseudo) pseudo-class

Tests

- [only-of-type.html](https://wpt.fyi/results/css/selectors/only-of-type.html) [(live test)](http://wpt.live/css/selectors/only-of-type.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/only-of-type.html)

The <a id="only-of-type-pseudo"></a>:only-of-type pseudo-class represents the same element as :first-of-type:last-of-type.

## <a id="combinators"></a>14.  Combinators

### <a id="descendant-combinators"></a>14.1.  Descendant combinator (`   `)

Tests

- [parse-descendant.html](https://wpt.fyi/results/css/selectors/parsing/parse-descendant.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-descendant.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-descendant.html)

<a id="ref-for-the-em-element"></a>

<a id="ref-for-the-h1-element⑥"></a>

At times, authors may want selectors to describe an element that is the descendant of another element in the document tree (e.g., "an [em](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-em-element) element that is contained within an [H1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element"). The <a id="descendant-combinator"></a>descendant combinator expresses such a relationship.

<a id="ref-for-compound③⑥"></a>

A descendant combinator is whitespace that separates two [compound selectors](#compound).

A selector of the form A B represents an element `B` that is an arbitrary descendant of some ancestor element `A`.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-13246ba1"></a> Examples: For example, consider the following selector:
>
> ```text
> h1 em
> ```
>
> <a id="ref-for-the-em-element①"></a>
>
> <a id="ref-for-the-h1-element⑦"></a>
>
> It represents an [em](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-em-element) element being the descendant of an [h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element. It is a correct and valid, but partial, description of the following fragment:
>
> ```text
> <h1>This <span class="myclass">headline
> is <em>very</em> important</span></h1>
> ```
>
> The following selector:
>
> ```text
> div * p
> ```
>
> <a id="ref-for-the-p-element⑤"></a>
>
> <a id="ref-for-the-div-element②"></a>
>
> represents a [p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element that is a grandchild or later descendant of a [div](https://html.spec.whatwg.org/multipage/grouping-content.html#the-div-element) element. Note the whitespace on either side of the "\*" is not part of the universal selector; the whitespace is a combinator indicating that the `div` must be the ancestor of some element, and that that element must be an ancestor of the `p`. The following selector, which combines descendant combinators and [attribute selectors](#attribute-selectors), represents an element that (1) has the `href` attribute set and (2) is inside a `p` that is itself inside a `div`:
>
> ```text
> div p *[href]
> ```
### <a id="child-combinators"></a>14.2.  Child combinator (`>`)

Tests

- [parse-child.html](https://wpt.fyi/results/css/selectors/parsing/parse-child.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-child.html)

<a id="ref-for-compound③⑦"></a>

A <a id="child-combinator"></a>child combinator describes a childhood relationship between two elements. A child combinator is made of the "greater-than sign" (U+003E, <a id="selectordef-child"></a>\>) code point and separates two [compound selectors](#compound).

<a id="ref-for-the-p-element⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-63e612ba"></a> Examples: The following selector represents a [p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element that is child of `body`:
>
> ```text
> body > p
> ```
>
> The following example combines descendant combinators and child combinators.
>
> ```text
> div ol>li p
> ```
>
> <a id="ref-for-the-p-element⑦"></a>
>
> <a id="ref-for-the-li-element"></a>
>
> <a id="ref-for-the-ol-element①"></a>
>
> It represents a [p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element that is a descendant of an [li](https://html.spec.whatwg.org/multipage/grouping-content.html#the-li-element) element; the <a id="ref-for-the-li-element①"></a>li element must be the child of an [ol](https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element) element; the <a id="ref-for-the-ol-element②"></a>ol element must be a descendant of a `div`. Notice that the optional white space around the "\>" combinator has been left out.

<a id="ref-for-first-child-pseudo①"></a>

For information on selecting the first child of an element, please see the section on the [:first-child](#first-child-pseudo) pseudo-class above.

### <a id="adjacent-sibling-combinators"></a>14.3.  Next-sibling combinator (`+`)

Tests

- [insert-sibling-001.html](https://wpt.fyi/results/css/selectors/invalidation/insert-sibling-001.html) [(live test)](http://wpt.live/css/selectors/invalidation/insert-sibling-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/insert-sibling-001.html)
- [insert-sibling-002.html](https://wpt.fyi/results/css/selectors/invalidation/insert-sibling-002.html) [(live test)](http://wpt.live/css/selectors/invalidation/insert-sibling-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/insert-sibling-002.html)
- [insert-sibling-003.html](https://wpt.fyi/results/css/selectors/invalidation/insert-sibling-003.html) [(live test)](http://wpt.live/css/selectors/invalidation/insert-sibling-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/insert-sibling-003.html)
- [insert-sibling-004.html](https://wpt.fyi/results/css/selectors/invalidation/insert-sibling-004.html) [(live test)](http://wpt.live/css/selectors/invalidation/insert-sibling-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/insert-sibling-004.html)
- [sibling.html](https://wpt.fyi/results/css/selectors/invalidation/sibling.html) [(live test)](http://wpt.live/css/selectors/invalidation/sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/invalidation/sibling.html)
- [parse-sibling.html](https://wpt.fyi/results/css/selectors/parsing/parse-sibling.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-sibling.html)

<a id="ref-for-compound③⑧"></a>

The <a id="next-sibling-combinator"></a>next-sibling combinator is made of the “plus sign” (U+002B, <a id="selectordef-adjacent"></a>+) code point that separates two [compound selectors](#compound). The elements represented by the two <a id="ref-for-compound③⑨"></a>compound selectors share the same parent in the document tree and the element represented by the first <a id="ref-for-compound④⓪"></a>compound selector immediately precedes the element represented by the second one. Non-element nodes (e.g. text between elements) are ignored when considering the adjacency of elements.

<a id="ref-for-the-p-element⑧"></a>

<a id="ref-for-dfn-math"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0f0597d9"></a> Examples: The following selector represents a [p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element) element immediately following a [math](https://www.w3.org/TR/mathml-core/#dfn-math) element:
>
> ```text
> math + p
> ```
>
> <a id="ref-for-the-h1-element⑧"></a>
>
> The following selector is conceptually similar to the one in the previous example, except that it adds an attribute selector — it adds a constraint to the [h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1-element) element, that it must have `class="opener"`:
>
> ```text
> h1.opener + h2
> ```
### <a id="general-sibling-combinators"></a>14.4.  Subsequent-sibling combinator (`~`)

<a id="ref-for-compound④①"></a>

The <a id="subsequent-sibling-combinator"></a>subsequent-sibling combinator is made of the "tilde" (U+007E, <a id="selectordef-sibling"></a>~) code point that separates two [compound selectors](#compound). The elements represented by the two <a id="ref-for-compound④②"></a>compound selectors share the same parent in the document tree and the element represented by the first compound selector precedes (not necessarily immediately) the element represented by the second one.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae614b62"></a>
>
> ```text
> h1 ~ pre
> ```
>
> <a id="ref-for-the-pre-element"></a>
>
> represents a [pre](https://html.spec.whatwg.org/multipage/grouping-content.html#the-pre-element) element following an `h1`. It is a correct and valid, but partial, description of:
>
> ```text
> <h1>Definition of the function a</h1>
> <p>Function a(x) has to be applied to all figures in the table.</p>
> <pre>function a(x) = 12x/13.5</pre>
> ```
## <a id="specificity-rules"></a>15.  Calculating a selector’s specificity

A selector’s <a id="specificity"></a>specificity is calculated for a given element as follows:

- count the number of ID selectors in the selector (= <var>A</var>)
- count the number of class selectors, attributes selectors, and pseudo-classes in the selector (= <var>B</var>)
- count the number of type selectors and pseudo-elements in the selector (= <var>C</var>)
- ignore the universal selector

<a id="ref-for-selector-list①⓪"></a>

If the selector is a [selector list](#selector-list), this number is calculated for each selector in the list. For a given matching process against the list, the specificity in effect is that of the most specific selector in the list that matches.

A few pseudo-classes provide “evaluation contexts” for other selectors, and so have their specificity defined specially:

- <a id="ref-for-selector-list①①"></a>

  <a id="ref-for-complex⑥"></a>

  <a id="ref-for-has-pseudo①①"></a>

  <a id="ref-for-negation-pseudo⑧"></a>

  <a id="ref-for-matches-pseudo①⑥"></a>

  The specificity of an [:is()](#matches-pseudo), [:not()](#negation-pseudo), or [:has()](#has-pseudo) pseudo-class is replaced by the specificity of the most specific [complex selector](#complex) in its [selector list](#selector-list) argument.

- <a id="ref-for-selector-list①②"></a>

  <a id="ref-for-complex⑦"></a>

  <a id="ref-for-nth-last-child-pseudo②"></a>

  <a id="ref-for-nth-child-pseudo④"></a>

  Analogously, the specificity of an [:nth-child()](#nth-child-pseudo) or [:nth-last-child()](#nth-last-child-pseudo) selector is the specificity of the pseudo-class itself (counting as one pseudo-class selector) plus the specificity of the most specific [complex selector](#complex) in its [selector list](#selector-list) argument (if any).

- <a id="ref-for-where-pseudo⑤"></a>

  The specificity of a [:where()](#where-pseudo) pseudo-class is replaced by zero.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-66faea1b"></a> For example:
>
> - :is(em, \#foo) has a specificity of (1,0,0)—​like an ID selector (\#foo)—​when matched against any of `<em>`, `<p id=foo>`, or `<em id=foo>`.
>
> - <a id="ref-for-where-pseudo⑥"></a>
>
>   .qux:where(em, \#foo#bar#baz) has a specificity of (0,1,0): only the .qux outside the [:where()](#where-pseudo) contributes to selector specificity.
>
> - :nth-child(even of li, .item) has a specificity of (0,2,0)—​like a class selector (.item) plus a pseudo-class—​when matched against any of `<li>`, `<ul class=item>`, or `<li class=item id=foo>`.
>
> - :not(em, strong#foo) has a specificity of (1,0,1)—​like a tag selector (strong) combined with an ID selector (\#foo)—​when matched against any element.

Specificities are compared by comparing the three components in order: the specificity with a larger <var>A</var> value is more specific; if the two <var>A</var> values are tied, then the specificity with a larger <var>B</var> value is more specific; if the two <var>B</var> values are also tied, then the specificity with a larger <var>C</var> value is more specific; if all the values are tied, the two specificities are equal.

Due to storage limitations, implementations may have limitations on the size of <var>A</var>, <var>B</var>, or <var>C</var>. If so, values higher than the limit must be clamped to that limit, and not overflow.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d97bd125"></a> Examples:
>
> ```text
> *               /* a=0 b=0 c=0 */
> LI              /* a=0 b=0 c=1 */
> UL LI           /* a=0 b=0 c=2 */
> UL OL+LI        /* a=0 b=0 c=3 */
> H1 + *[REL=up]  /* a=0 b=1 c=1 */
> UL OL LI.red    /* a=0 b=1 c=3 */
> LI.red.level    /* a=0 b=2 c=1 */
> #x34y           /* a=1 b=0 c=0 */
> #s12:not(FOO)   /* a=1 b=0 c=1 */
> .foo :is(.bar, #baz)
>                 /* a=1 b=1 c=0 */
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Repeated occurrences of the same simple selector are allowed and do increase specificity.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The specificity of the styles specified in an HTML `style` attribute [is described in CSS Style Attributes](https://www.w3.org/TR/css-style-attr/#interpret). [\[CSSSTYLEATTR\]](#biblio-cssstyleattr)

## <a id="grammar"></a>16. <a id="formal-syntax"></a> Grammar

Tests

- [selectors-attr-white-space-001.html](https://wpt.fyi/results/css/selectors/selectors-attr-white-space-001.html) [(live test)](http://wpt.live/css/selectors/selectors-attr-white-space-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/selectors-attr-white-space-001.html)

<a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>

Selectors are [parsed](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) according to the following grammar:

<a id="typedef-selector-list"></a>

<a id="ref-for-typedef-complex-selector-list"></a>

<a id="typedef-complex-selector-list"></a>

<a id="ref-for-typedef-complex-selector①"></a>

<a id="ref-for-mult-comma"></a>

<a id="typedef-complex-real-selector-list"></a>

<a id="ref-for-typedef-complex-real-selector"></a>

<a id="ref-for-mult-comma①"></a>

<a id="typedef-compound-selector-list"></a>

<a id="ref-for-typedef-compound-selector①"></a>

<a id="ref-for-mult-comma②"></a>

<a id="typedef-simple-selector-list"></a>

<a id="ref-for-typedef-simple-selector①"></a>

<a id="ref-for-mult-comma③"></a>

<a id="typedef-relative-selector-list"></a>

<a id="ref-for-typedef-relative-selector①"></a>

<a id="ref-for-mult-comma④"></a>

<a id="typedef-relative-real-selector-list"></a>

<a id="ref-for-typedef-relative-real-selector"></a>

<a id="ref-for-mult-comma⑤"></a>

<a id="typedef-complex-selector"></a>

<a id="ref-for-typedef-complex-selector-unit"></a>

<a id="ref-for-typedef-combinator"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-typedef-complex-selector-unit①"></a>

<a id="ref-for-mult-zero-plus"></a>

<a id="typedef-complex-selector-unit"></a>

<a id="ref-for-typedef-compound-selector②"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-typedef-pseudo-compound-selector①"></a>

<a id="ref-for-mult-zero-plus①"></a>

<a id="ref-for-mult-req"></a>

<a id="typedef-complex-real-selector"></a>

<a id="ref-for-typedef-compound-selector③"></a>

<a id="ref-for-typedef-combinator①"></a>

<a id="ref-for-mult-opt②"></a>

<a id="ref-for-typedef-compound-selector④"></a>

<a id="ref-for-mult-zero-plus②"></a>

<a id="typedef-relative-selector"></a>

<a id="ref-for-typedef-combinator②"></a>

<a id="ref-for-mult-opt③"></a>

<a id="ref-for-typedef-complex-selector②"></a>

<a id="typedef-relative-real-selector"></a>

<a id="ref-for-typedef-combinator③"></a>

<a id="ref-for-mult-opt④"></a>

<a id="ref-for-typedef-complex-real-selector①"></a>

<a id="typedef-compound-selector"></a>

<a id="ref-for-typedef-type-selector"></a>

<a id="ref-for-mult-opt⑤"></a>

<a id="ref-for-typedef-subclass-selector"></a>

<a id="ref-for-mult-zero-plus③"></a>

<a id="ref-for-mult-req①"></a>

<a id="typedef-pseudo-compound-selector"></a>

<a id="ref-for-typedef-pseudo-element-selector"></a>

<a id="ref-for-typedef-pseudo-class-selector"></a>

<a id="ref-for-mult-zero-plus④"></a>

<a id="typedef-simple-selector"></a>

<a id="ref-for-typedef-type-selector①"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-typedef-subclass-selector①"></a>

<a id="typedef-combinator"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-comb-one③"></a>

<a id="typedef-wq-name"></a>

<a id="ref-for-typedef-ns-prefix"></a>

<a id="ref-for-mult-opt⑥"></a>

<a id="ref-for-typedef-ident-token②"></a>

<a id="typedef-ns-prefix"></a>

<a id="ref-for-typedef-ident-token③"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-mult-opt⑦"></a>

<a id="typedef-type-selector"></a>

<a id="ref-for-typedef-wq-name"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-typedef-ns-prefix①"></a>

<a id="ref-for-mult-opt⑧"></a>

<a id="typedef-subclass-selector"></a>

<a id="ref-for-typedef-id-selector"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-typedef-class-selector"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-typedef-attribute-selector"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-typedef-pseudo-class-selector①"></a>

<a id="typedef-id-selector"></a>

<a id="ref-for-typedef-hash-token"></a>

<a id="typedef-class-selector"></a>

<a id="ref-for-typedef-ident-token④"></a>

<a id="typedef-attribute-selector"></a>

<a id="ref-for-typedef-wq-name①"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-typedef-wq-name②"></a>

<a id="ref-for-typedef-attr-matcher"></a>

<a id="ref-for-typedef-string-token②"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-typedef-ident-token⑤"></a>

<a id="ref-for-typedef-attr-modifier"></a>

<a id="ref-for-mult-opt⑨"></a>

<a id="typedef-attr-matcher"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-mult-opt①⓪"></a>

<a id="typedef-attr-modifier"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="typedef-pseudo-class-selector"></a>

<a id="ref-for-typedef-ident-token⑥"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-typedef-function-token"></a>

<a id="ref-for-typedef-any-value"></a>

<a id="typedef-pseudo-element-selector"></a>

<a id="ref-for-typedef-pseudo-class-selector②"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-typedef-legacy-pseudo-element-selector"></a>

<a id="typedef-legacy-pseudo-element-selector"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-comb-one②⓪"></a>

```text
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


<combinator> = '>' | '+' | '~' | [ '|' '|' ]

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

- <a id="white-space"></a> White space is forbidden:

  - <a id="ref-for-typedef-compound-selector⑤"></a>

    <a id="ref-for-typedef-complex-selector-unit②"></a>

    <a id="ref-for-typedef-type-selector②"></a>

    <a id="ref-for-typedef-subclass-selector②"></a>

    <a id="ref-for-typedef-pseudo-element-selector①"></a>

    <a id="ref-for-typedef-pseudo-class-selector③"></a>

    Between any of the top-level components of a [\<compound-selector\>](#typedef-compound-selector) or [\<complex-selector-unit\>](#typedef-complex-selector-unit) (that is, forbidden between the [\<type-selector\>](#typedef-type-selector) and [\<subclass-selector\>](#typedef-subclass-selector), or between the [\<pseudo-element-selector\>](#typedef-pseudo-element-selector) and [\<pseudo-class-selector\>](#typedef-pseudo-class-selector), etc).

  - <a id="ref-for-typedef-type-selector③"></a>

    <a id="ref-for-typedef-class-selector①"></a>

    Between <em>any</em> of the components of a [\<type-selector\>](#typedef-type-selector) or a [\<class-selector\>](#typedef-class-selector).

  - <a id="ref-for-typedef-ident-token⑦"></a>

    <a id="ref-for-typedef-function-token①"></a>

    <a id="ref-for-typedef-pseudo-element-selector②"></a>

    <a id="ref-for-typedef-pseudo-class-selector④"></a>

    Between the ':'s, or between the ':' and [\<ident-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-ident-token) or [\<function-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-function-token), of a [\<pseudo-element-selector\>](#typedef-pseudo-element-selector) or a [\<pseudo-class-selector\>](#typedef-pseudo-class-selector).

  - <a id="ref-for-typedef-wq-name③"></a>

    Between <em>any</em> of the components of a [\<wq-name\>](#typedef-wq-name).

  - <a id="ref-for-typedef-attr-matcher①"></a>

    Between the components of an [\<attr-matcher\>](#typedef-attr-matcher).

  - <a id="ref-for-typedef-compound-selector⑥"></a>

    <a id="ref-for-typedef-pseudo-compound-selector②"></a>

    <a id="ref-for-typedef-complex-selector-unit③"></a>

    Between the [\<compound-selector\>](#typedef-compound-selector) or [\<pseudo-compound-selector\>](#typedef-pseudo-compound-selector)s in a [\<complex-selector-unit\>](#typedef-complex-selector-unit)

  - <a id="ref-for-typedef-combinator④"></a>

    Between the components of a [\<combinator\>](#typedef-combinator).

  <a id="ref-for-typedef-complex-selector-unit④"></a>

  <a id="ref-for-typedef-combinator⑤"></a>

  Whitespace is <em>required</em> between two [\<complex-selector-unit\>](#typedef-complex-selector-unit)s if the [\<combinator\>](#typedef-combinator) between them is omitted. (This indicates the descendant combinator is being used.)

- <a id="ref-for-css-css-identifier③"></a>

  <a id="ref-for-typedef-hash-token①"></a>

  <a id="ref-for-typedef-id-selector①"></a>

  In [\<id-selector\>](#typedef-id-selector), the [\<hash-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-hash-token)’s value must be an [identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier).

- <a id="ref-for-selectordef-before⑥"></a>

  <a id="ref-for-typedef-legacy-pseudo-element-selector①"></a>

  <a id="ref-for-typedef-pseudo-class-selector⑤"></a>

  The [\<pseudo-class-selector\>](#typedef-pseudo-class-selector) production excludes the [\<legacy-pseudo-element-selector\>](#typedef-legacy-pseudo-element-selector) production. (That is, [:before](https://drafts.csswg.org/css2/#selectordef-before)/etc must never be parsed as a pseudo-class, even if doing so would cause the selector to become valid due to, for example, other simple selectors following it.)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A selector is also subject to a variety of more specific syntactic constraints, and adherence to the grammar above is necessary <em>but not sufficient</em> for the selector to be considered valid. See [§ 3.9 Invalid Selectors and Error Handling](#invalid) for additional rules for parsing selectors.

<a id="ref-for-typedef-pseudo-element-selector③"></a>

<a id="ref-for-typedef-compound-selector⑦"></a>

<a id="ref-for-typedef-complex-selector③"></a>

<a id="ref-for-typedef-pseudo-class-selector⑥"></a>

<a id="ref-for-user-action-pseudo-class④"></a>

<a id="ref-for-pseudo-element③⓪"></a>

<a id="ref-for-tree-abiding①"></a>

<a id="ref-for-selectordef-slotted"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In general, a [\<pseudo-element-selector\>](#typedef-pseudo-element-selector) is only valid if placed at the end of the last [\<compound-selector\>](#typedef-compound-selector) in a [\<complex-selector\>](#typedef-complex-selector). In some circumstances, however, it can be followed by more <a id="ref-for-typedef-pseudo-element-selector④"></a>\<pseudo-element-selector\>s or [\<pseudo-class-selector\>](#typedef-pseudo-class-selector)s; but these are specified on a case-by-case basis. (For example, the [user action pseudo-classes](#user-action-pseudo-class) are allowed after any [pseudo-element](#pseudo-element), and the [tree-abiding pseudo-elements](https://www.w3.org/TR/css-pseudo-4/#tree-abiding) are allowed after the [::slotted()](https://drafts.csswg.org/css-shadow-1/#selectordef-slotted) pseudo-element.)

<a id="ref-for-pseudo-element③①"></a>

<a id="ref-for-selectordef-before⑦"></a>

<a id="ref-for-selectordef-after②"></a>

<a id="ref-for-selectordef-first-line⑨"></a>

<a id="ref-for-selectordef-first-letter②"></a>

<a id="ref-for-typedef-pseudo-class-selector⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="single-colon-pseudos"></a> The four [Level 2](https://www.w3.org/TR/CSS2/selectors.html#pseudo-element-selectors) [pseudo-elements](#pseudo-element) ([::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before), [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after), [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line), and [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter)) may, for legacy reasons, be written with only a single ":" character at their front, making them resemble a [\<pseudo-class-selector\>](#typedef-pseudo-class-selector).

<a id="ref-for-typedef-forgiving-selector-list①"></a>

<a id="ref-for-typedef-forgiving-relative-selector-list"></a>

### <a id="forgiving-selector"></a>16.1.  [\<forgiving-selector-list\>](#typedef-forgiving-selector-list) and [\<forgiving-relative-selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-forgiving-relative-selector-list)

For legacy reasons, the general behavior of a selector list is that if any selector in the list fails to parse (because it uses new or UA-specific selector features, for instance), the entire selector list becomes invalid. This can make it hard to write CSS that uses new selectors and still works correctly in older user agents.

<a id="ref-for-typedef-forgiving-selector-list②"></a>

The <a id="typedef-forgiving-selector-list"></a>[\<forgiving-selector-list\>](#typedef-forgiving-selector-list) production instead parses each selector in the list individually, simply ignoring ones that fail to parse, so the remaining selectors can still be used.

<a id="ref-for-typedef-forgiving-selector-list③"></a>

<a id="ref-for-matches-pseudo①⑦"></a>

<a id="ref-for-where-pseudo⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Style rules still use the normal, unforgiving selector list behavior. [\<forgiving-selector-list\>](#typedef-forgiving-selector-list) is used in [:is()](#matches-pseudo) and [:where()](#where-pseudo) only. Although it does have some minor implications on specificity, wrapping a style rule’s selector in <a id="ref-for-matches-pseudo①⑧"></a>:is() effectively "upgrades" it to become forgiving, so long as it doesn’t contain any pseudo-elements (which aren’t valid in <a id="ref-for-matches-pseudo①⑨"></a>:is() or <a id="ref-for-where-pseudo⑧"></a>:where()).

<a id="ref-for-typedef-forgiving-selector-list④"></a>

<a id="ref-for-typedef-any-value①"></a>

<a id="ref-for-parse-as-a-forgiving-selector-list"></a>

Syntactically, [\<forgiving-selector-list\>](#typedef-forgiving-selector-list) is equivalent to <code><a href="https://www.w3.org/TR/css-syntax-3/#typedef-any-value">&lt;any-value&gt;</a>?</code>. It is then [parsed as a forgiving selector list](#parse-as-a-forgiving-selector-list) to obtain its actual value.

To <a id="parse-as-a-forgiving-selector-list"></a>parse as a forgiving selector list given an input <var>input</var>:

1.  <a id="ref-for-css-parse-a-comma-separated-list-according-to-a-css-grammar"></a>

    <a id="ref-for-typedef-complex-real-selector②"></a>

    [Parse a list](https://www.w3.org/TR/css-syntax-3/#css-parse-a-comma-separated-list-according-to-a-css-grammar) of [\<complex-real-selector\>](#typedef-complex-real-selector)s from <var>input</var>, and let <var>selector list</var> be the result.

2.  <a id="ref-for-invalid-selector④"></a>

    <a id="ref-for-typedef-selector-list"></a>

    Remove all failure items from <var>selector list</var>, and all items that are [invalid selectors](#invalid-selector), then return a [\<selector-list\>](#typedef-selector-list) representing the remaining items in <var>selector list</var>. (This might be empty.)

<a id="ref-for-typedef-forgiving-selector-list⑤"></a>

Any items in a [\<forgiving-selector-list\>](#typedef-forgiving-selector-list) that are invalid (whether explicitly, by using unknown selectors or syntax, or merely contextually, using known syntax but in an invalid context) must be treated as having zero specificity.

<a id="ref-for-typedef-forgiving-selector-list⑥"></a>

<a id="ref-for-matches-pseudo②⓪"></a>

<a id="ref-for-where-pseudo⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\<forgiving-selector-list\>](#typedef-forgiving-selector-list) is intentionally used only in [:is()](#matches-pseudo) and [:where()](#where-pseudo), not in any other selector that takes a selector argument.

## <a id="api-hooks"></a>17.  API Hooks

To aid in the writing of specs that use Selectors concepts, this section defines several API hooks that can be invoked by other specifications.

<a id="ref-for-match⑦"></a>

<a id="ref-for-invalid-selector⑤"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-55d7bd68"></a> Are these still necessary now that we have more rigorous definitions for [match](#match) and [invalid selector](#invalid-selector)? Nouns are a lot easier to coordinate across specification than predicates, and details like the exact order of elements returned from `querySelector` seem to make more sense being defined in the DOM specification than in Selectors.

### <a id="parse-selector"></a>17.1.  Parse A Selector

This section defines how to <a id="parse-a-selector"></a>parse a selector from a string <var>source</var>. It returns either a complex selector list, or failure.

1.  <a id="ref-for-invalid-selector⑥"></a>

    <a id="ref-for-typedef-selector-list①"></a>

    <a id="ref-for-css-parse-something-according-to-a-css-grammar①"></a>

    Let <var>selector</var> be the result of [parsing](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>source</var> as a [\<selector-list\>](#typedef-selector-list). If this returns failure, it’s an [invalid selector](#invalid-selector); return failure.

2.  <a id="ref-for-invalid-selector⑦"></a>

    If <var>selector</var> is an [invalid selector](#invalid-selector) for any other reason (such as, for example, containing an undeclared namespace prefix), return failure.

3.  Otherwise, return <var>selector</var>.

### <a id="parse-relative-selector"></a>17.2.  Parse A Relative Selector

This section defines how to <a id="parse-a-relative-selector"></a>parse a relative selector from a string <var>source</var>. It returns either a complex selector list, or failure.

1.  <a id="ref-for-invalid-selector⑧"></a>

    <a id="ref-for-typedef-relative-selector-list③"></a>

    <a id="ref-for-css-parse-something-according-to-a-css-grammar②"></a>

    Let <var>selector</var> be the result of [parsing](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>source</var> as a [\<relative-selector-list\>](#typedef-relative-selector-list). If this returns failure, it’s an [invalid selector](#invalid-selector); return failure.

2.  <a id="ref-for-invalid-selector⑨"></a>

    If <var>selector</var> is an [invalid selector](#invalid-selector) for any other reason (such as, for example, containing an undeclared namespace prefix), return failure.

3.  Otherwise, return <var>selector</var>.

### <a id="match-against-element"></a>17.3.  Match a Selector Against an Element

This section defines how to <a id="match-a-selector-against-an-element"></a>match a selector against an element.

APIs using this algorithm must provide a <var>selector</var> and an <var>element</var>.

Callers may optionally provide:

- <a id="ref-for-scope-pseudo②"></a>

  <a id="ref-for-scoping-root⑨"></a>

  one or more [scoping roots](#scoping-root), for resolving the [:scope](#scope-pseudo) pseudo-class against.

This algorithm returns either success or failure.

<a id="ref-for-complex⑧"></a>

<a id="ref-for-list-of-simple-selectors①"></a>

For each [complex selector](#complex) in the given <var>selector</var> (which is taken to be a [list of complex selectors](#list-of-simple-selectors)), match the complex selector against <var>element</var>, as described in the following paragraph. If the matching returns success for any complex selector, then the algorithm return success; otherwise it returns failure.

<a id="ref-for-compound④③"></a>

To <a id="match-a-complex-selector-against-an-element"></a>match a complex selector against an element, process it [compound selector](#compound) at a time, in right-to-left order. This process is defined recursively as follows:

- If any simple selectors in the rightmost compound selector does not match the element, return failure.

- Otherwise, if there is only one compound selector in the complex selector, return success.

- <a id="ref-for-selector-combinator⑧"></a>

  Otherwise, consider all possible elements that could be related to this element by the rightmost [combinator](#selector-combinator). If the operation of matching the selector consisting of this selector with the rightmost compound selector and rightmost combinator removed against any one of these elements returns success, then return success. Otherwise, return failure.

### <a id="match-against-pseudo-element"></a>17.4.  Match a Selector Against a Pseudo-element

This section defines how to <a id="match-a-selector-against-a-pseudo-element"></a>match a selector against a pseudo-element.

<a id="ref-for-match-a-selector-against-an-element"></a>

APIs using this algorithm must provide a <var>selector</var> and a <var>pseudo-element</var>. They may optionally provide the same things they may optionally provide to the algorithm to [match a selector against an element](#match-a-selector-against-an-element).

This algorithm returns success or failure.

<a id="ref-for-complex⑨"></a>

For each [complex selector](#complex) in the given <var>selector</var>, if both:

- <a id="ref-for-simple①⑥"></a>

  the rightmost [simple selector](#simple) in the complex selector matches <var>pseudo-element</var>, and

- <a id="ref-for-complex①⓪"></a>

  <a id="ref-for-match-a-complex-selector-against-an-element"></a>

  the result of running [match a complex selector against an element](#match-a-complex-selector-against-an-element) on the remainder of the [complex selector](#complex) (with just the rightmost simple selector of its rightmost complex selector removed), <var>pseudo-element</var>’s corresponding element, and any optional parameters provided to this algorithm returns success,

then return success.

Otherwise (that is, if this doesn’t happen for any of the complex selectors in <var>selector</var>), return failure.

### <a id="match-against-tree"></a>17.5.  Match a Selector Against a Tree

This section defines how to <a id="match-a-selector-against-a-tree"></a>match a selector against a tree.

<a id="ref-for-concept-tree"></a>

<a id="ref-for-concept-tree-root①"></a>

APIs using this algorithm must provide a selector, and one or more <var>root elements</var> indicating the [subtrees](https://dom.spec.whatwg.org/#concept-tree) that will be searched by the selector. All of the <var>root elements</var> must share the same [root](https://dom.spec.whatwg.org/#concept-tree-root), or else calling this algorithm is invalid.

They may optionally provide:

- <a id="ref-for-scoped-selector②"></a>

  <a id="ref-for-scoping-root①⓪"></a>

  One or more [scoping roots](#scoping-root) indicating the selector is [scoped](#scoped-selector).

- <a id="ref-for-pseudo-element③②"></a>

  A list of [pseudo-elements](#pseudo-element) that are allowed to show up in the match list. If not specified, this defaults to allowing all pseudo-elements.

  <a id="ref-for-tree-abiding②"></a>

  > <strong data-conversion-semantic="issue">Issue</strong>
  >
  > <a id="issue-15f3317f"></a> Only the [tree-abiding pseudo-elements](https://www.w3.org/TR/css-pseudo-4/#tree-abiding) are really handled in any way remotely like this.

This algorithm returns a (possibly empty) list of elements.

1.  <a id="ref-for-concept-shadow-including-tree-order"></a>

    Start with a list of <var>candidate elements</var>, which are the <var>root elements</var> and all of their descendant elements, sorted in [shadow-including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order), unless otherwise specified.

2.  <a id="ref-for-concept-tree-descendant"></a>

    <a id="ref-for-scoping-root①①"></a>

    If [scoping root](#scoping-root) were provided, then remove from the <var>candidate elements</var> any elements that are not [descendants](https://dom.spec.whatwg.org/#concept-tree-descendant) of at least one <a id="ref-for-scoping-root①②"></a>scoping root.

3.  Initialize the <var>selector match list</var> to empty.

4.  For each <var>element</var> in the set of <var>candidate elements</var>:
    1.  <a id="ref-for-match-a-selector-against-an-element①"></a>

        If the result of [match a selector against an element](#match-a-selector-against-an-element) for <var>element</var> and <var>selector</var> is success, add <var>element</var> to the <var>selector match list</var>.

    2.  <a id="ref-for-match-a-selector-against-a-pseudo-element"></a>

        For each possible pseudo-element associated with <var>element</var> that is one of the pseudo-elements allowed to show up in the match list, if the result of [match a selector against a pseudo-element](#match-a-selector-against-a-pseudo-element) for the pseudo-element and <var>selector</var> is success, add the pseudo-element to the <var>selector match list</var>.

        > <strong data-conversion-semantic="issue">Issue</strong>
        >
        > <a id="issue-b7f52036"></a> The relative position of pseudo-elements in <var>selector match list</var> is undefined. There’s not yet a context that exposes this information, but we need to decide on something eventually, before something <em>is</em> exposed.

## <a id="dom-mapping"></a> Appendix A: Guidance on Mapping Source Documents &#x26; Data to an Element Tree

<em>This section is informative.</em>

The element tree structure described by the DOM is powerful and useful, but generic enough to model pretty much any language that describes tree-based data (or even graph-based, with a suitable interpretation).

Some languages, like HTML, already have well-defined procedures for producing a DOM object from a resource. If a given language does not, such a procedure must be defined in order for Selectors to apply to documents in that language.

At minimum, the document language must define what maps to the DOM concept of an "element".

The primary one-to-many relationship between nodes—​parent/child in tree-based structures, element/neighbors in graph-based structures—​should be reflected as the child nodes of an element.

Other features of the element should be mapped to something that serves a similar purpose to the same feature in DOM:

type  
If the elements in the document language have some notion of "type" as a basic distinguisher between different groups of elements, it should be reflected as the "type" feature.

If this "type" can be separated into a "basic" name and a "namespace" that groups names into higher-level groups, the latter should be reflected as the "namespace" feature. Otherwise, the element shouldn’t have a "namespace" feature, and the entire name should be reflected as the "type" feature.

id  
If some aspect of the element functions as a unique identifier across the document, it should be mapped to the "id" feature.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While HTML only allows an element to have a single ID, this should not be taken as a general restriction. The important quality of an ID is that each ID should be associated with a single element; a single element can validly have multiple IDs.

classes and attributes  
Aspects of the element that are useful for identifying the element, but are not generally unique to elements within a document, should be mapped to the "class" or "attribute" features depending on if they’re something equivalent to a "label" (a string by itself) or a "property" (a name/value pair)

pseudo-classes and pseudo-elements  
If any elements match any pseudo-classes or have any pseudo-elements, that must be explicitly defined.

<a id="ref-for-has-pseudo①②"></a>

<a id="ref-for-matches-pseudo②①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-772ec278"></a> Some pseudo-classes are \*syntactical\*, like [:has()](#has-pseudo) and [:is()](#matches-pseudo), and thus should always work. Need to indicate that somewhere. Probably the structural pseudos always work whenever the child list is ordered.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-37a1ab6b"></a> For example, [JSONSelect](https://github.com/lloyd/JSONSelect) is a library that uses selectors to extract information from JSON documents.
>
> - The "elements" of the JSON document are each array, object, boolean, string, number, or null. The array and object elements have their contents as children.
>
> - Each element’s type is its JS type name: "array", "object", etc.
>
> - Children of an object have their key as a class.
>
> - <a id="ref-for-nth-child-pseudo⑤"></a>
>
>   <a id="ref-for-first-child-pseudo②"></a>
>
>   Children of an array match the [:first-child](#first-child-pseudo), [:nth-child()](#nth-child-pseudo), etc pseudo-classes.
>
> - <a id="ref-for-root-pseudo③"></a>
>
>   The root object matches [:root](#root-pseudo).
>
> - It additionally defines :val() and :contains() pseudo-classes, for matching boolean/number/string elements with a particular value or which contain a particular substring.
>
> This structure is sufficient to allow powerful, compact querying of JSON documents with selectors.

## <a id="compat"></a> Appendix B: Obsolete but Required `-webkit-` Parsing Quirks for Web Compat

Tests

- [webkit-pseudo-element.html](https://wpt.fyi/results/css/selectors/webkit-pseudo-element.html) [(live test)](http://wpt.live/css/selectors/webkit-pseudo-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/webkit-pseudo-element.html)

<em>This appendix is normative.</em>

Due to legacy Web-compat constraints, user agents expecting to parse Web documents must support the following features:

- <a id="ref-for-legacy-selector-alias①"></a>

  <a id="ref-for-selectordef-autofill①"></a>

  :-webkit-autofill must be treated as a [legacy selector alias](#legacy-selector-alias) of [:autofill](#selectordef-autofill).

- <a id="ref-for-pseudo-element③③"></a>

  <a id="ref-for-ascii-case-insensitive⑧"></a>

  All other [pseudo-elements](#pseudo-element) whose names begin with the string “-webkit-” (matched [ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive)) and that are not functional notations must be treated as valid at parse time. (That is, ::-webkit-asdf is valid at parse time, but ::-webkit-jkl() is not.) If they’re not otherwise recognized and supported, they must be treated as matching nothing, and are <a id="unknown--webkit--pseudo-elements"></a>unknown -webkit- pseudo-elements.

  <a id="ref-for-unknown--webkit--pseudo-elements"></a>

  [Unknown -webkit- pseudo-elements](#unknown--webkit--pseudo-elements) must be serialized in ASCII lowercase.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > What’s this quirk about?
  > Selectors have long had a behavior where a single unknown/invalid selector invalidates the entire selector list (rather than just invalidating the one complex selector it finds itself in). This is generally considered a legacy mistake by the WG, but can’t be fixed at this point, as too many stylesheets depend on this behavior, intentionally or not.
  >
  > <a id="ref-for-the-input-element⑥"></a>
  >
  > One aspect of this is that use of vendor-specific selectors invalidates the entire selector in other user agents that don’t recognize them, and takes the entire style rule down with it. This has been used intentionally in the past—​in the severely-not-recommended practice of hiding style rules from some browsers by making them invalid in every other browser—​and unintentionally, with people styling an element and also applying those styles to a vendor-specific pseudo-element (such as the various <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code>-related pseudos some browsers expose), not realizing that this hides the entire rule from other browsers.
  >
  > In addition to this more general reasoning, WebKit-derived user agents, such as Safari or Chrome, have an additional quirk related to their vendor-prefixed pseudo-elements, where any ::-webkit--prefixed selectors are considered valid at parse time. (This is probably a leftover quirk of an early CSS feature, since dropped, that intentionally treated all possible pseudo-elements as valid at parse time, in anticipation of a feature letting authors define their own pseudo-elements.)
  >
  > Similar to other legacy quirks, such as those documented in [\[QUIRKS\]](#biblio-quirks), this particular vendor-specific oddity has become common enough that other user agents are seeing sites breaking due to them depending on it, accidentally or not. As such, since the quirk is in practical terms <em>required</em> to render the modern web correctly, specifying it and requiring it for all user agents ensures that today’s web pages are more likely to be correctly rendered in user agents both current and future.
  >
  > As usual with quirks, however, webpages intentionally relying on this will be met with shaming and derision from members of the CSSWG, and all right-thinking web developers.

<a id="ref-for-visited-pseudo⑨"></a>

## <a id="visited-privacy"></a> Appendix C: Example Privacy-Preserving [:visited](#visited-pseudo) Restrictions

<a id="ref-for-visited-pseudo①⓪"></a>

<a id="ref-for-link-pseudo⑥"></a>

Previous attempts to protect user privacy in [:visited](#visited-pseudo) involved [complex restrictions and behaviors](https://developer.mozilla.org/en-US/docs/Web/CSS/Privacy_and_the_:visited_selector) to "lie" about whether the link match <a id="ref-for-visited-pseudo①①"></a>:visited or [:link](#link-pseudo), to reduce the chance that a hostile site could observe what unrelated sites a user had visited while still allowing <a id="ref-for-visited-pseudo①②"></a>:visited to work in all cases and help the user know what links they’d already clicked. This is ultimately an arms race that can’t be won; there are multiple documented ways to still extract a user’s browsing history even with these mitigations.

<a id="ref-for-visited-pseudo①③"></a>

This section describes an approach first developed and documented at [https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;explainers-by-googlers&#x2F;Partitioning-visited-links-history](https://github.com/explainers-by-googlers/Partitioning-visited-links-history), that partitions a user’s browsing history information, to allow [:visited](#visited-pseudo) to only match links corresponding to navigations that the site’s origin could have observed on its own. With this, <a id="ref-for-visited-pseudo①④"></a>:visited can be treated as a normal pseudo-class, without any of the complex mitigations described above, as it doesn’t expose any information not already theoretically available to the site, while still preserving as much of the <em>usefulness</em> of <a id="ref-for-visited-pseudo①⑤"></a>:visited as possible for the user.

1.  <a id="ref-for-ordered-set"></a>

    <a id="ref-for-tuple"></a>

    Let <var>visited history</var> be a [set](https://infra.spec.whatwg.org/#ordered-set) containing [tuples](https://infra.spec.whatwg.org/#tuple) of three pieces of information:

    - <a id="ref-for-concept-url"></a>

      a visited [URL](https://url.spec.whatwg.org/#concept-url)

    - <a id="ref-for-concept-origin"></a>

      an [origin](https://html.spec.whatwg.org/multipage/browsers.html#concept-origin) for the site that started a navigation

    - <a id="ref-for-site"></a>

      a [site](https://html.spec.whatwg.org/multipage/browsers.html#site) for the top-level site containing the frame that started the navigation. (This will often be the same as the previous, but can differ if the user clicks a link in a iframe, for example.)

2.  Whenever a navigation is triggered <em>from within a page</em>—​e.g., from the user clicking a link, or a script on the page initiating a navigation—​add an entry to <var>visited history</var> recording the navigation’s destination URL, the origin of the page containing the link or script, and the (schemeful) site of the top-level site containing that page (which might be the same site as the previous origin).

    <a id="ref-for-visited-pseudo①⑥"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This allows a site to see [:visited](#visited-pseudo) information for links that the user has clicked from anywhere in that site’s origin. In other words, any `A -> B` navigation where the site is A.

    Additionally, add an entry to <var>visited history</var> recording the destination’s URL, the <em>destination’s</em> origin, and the <em>destination’s</em> site. Do this only for navigations from top-level frames or iframes which are same-origin with their top-level frame.

    <a id="ref-for-visited-pseudo①⑦"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This allows for a site to see [:visited](#visited-pseudo) information about its own pages (which is already observable by the site) regardless of what site initiated the navigation to that page. In other words, any `A -> B` navigation where the site is B.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Notably, direct navigations triggered by the <em>user agent’s</em> UI, such as typing into the address bar, clicking on bookmarks, or dragging a link from another program into the page, <em>do not</em> add a <var>visited history</var> entry. These can, of course, still add to the browser’s record of visited sites that it uses for other purposes, such as suggesting URLs as the user types into the URL bar.

3.  <a id="ref-for-link-pseudo⑦"></a>

    <a id="ref-for-visited-pseudo①⑧"></a>

    When determining if a link element should match [:link](#link-pseudo) or [:visited](#visited-pseudo), only allow it to match <a id="ref-for-visited-pseudo①⑨"></a>:visited if the link’s destination, the origin of the page containing the link, and the top-level site containing the link match a tuple in <var>visited history</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> The inclusion of both page origin and top-level site prevents several possible privacy attacks, such as:
>
> - If history entries were <em>only</em> keyed by the starting site’s URL, a tracking site could be embedded in a hidden iframe on multiple sites which triggers a navigation to a unique URL for a user on the first visit, and then uses many such links on subsequent visits to see which one had been visited, effectively becoming a new "third-party cookie" identifying the user across the web. By keying the history entry with the top-level site, this information can’t be shared across different sites.
>
> - If history entires were <em>only</em> keyed by the top-level site’s URL, a hostile iframe, perhaps included in a page as part of an advertisement, could observe what sites were visited from the top-level site. By keying the history entry with the link’s own site, the top-level site’s information can’t "leak" into cross-origin iframes.

## <a id="changes"></a>18.  Changes

### <a id="changes-2022-11"></a>18.1.  Changes since the 11 November 2022 Working Draft

Significant changes since the [11 November 2022 Working Draft](https://www.w3.org/TR/2022/WD-selectors-4-20221111/):

- <a id="ref-for-read-write-pseudo②"></a>

  <a id="ref-for-has-pseudo①③"></a>

  Removed the at-risk status from [:read-write](#read-write-pseudo) and [:has()](#has-pseudo)

- <a id="ref-for-selectordef-popover-open①"></a>

  Added [:popover-open](#selectordef-popover-open) pseudo-class. ([Issue 8637](https://github.com/w3c/csswg-drafts/issues/8637))

- <a id="ref-for-has-pseudo①④"></a>

  <a id="ref-for-nth-child-pseudo⑥"></a>

  <a id="ref-for-nth-last-child-pseudo③"></a>

  Made [:has()](#has-pseudo) and the selector argument of [:nth-child()](#nth-child-pseudo)/[:nth-last-child()](#nth-last-child-pseudo) no longer forgiving. ([Issue 7676](https://github.com/w3c/csswg-drafts/issues/7676))

- Moved the legacy single-colon pseudo-element syntax into the grammar itself. ([Issue 8122](https://github.com/w3c/csswg-drafts/issues/8122))

- <a id="ref-for-local-link-pseudo"></a>

  Deferred the [:local-link](https://www.w3.org/TR/selectors-4/#local-link-pseudo) pseudo-class to Level 5. ([Issue 12799](https://github.com/w3c/csswg-drafts/issues/12799))

- <a id="ref-for-selectordef-interest-source"></a>

  <a id="ref-for-selectordef-interest-target"></a>

  Deferred the [:interest-source](https://drafts.csswg.org/selectors-5/#selectordef-interest-source) and [:interest-target](https://drafts.csswg.org/selectors-5/#selectordef-interest-target) pseudo-classes to Level 5. ([Issue 12799](https://github.com/w3c/csswg-drafts/issues/12799))

- <a id="ref-for-blank-pseudo"></a>

  Deferred the [:blank](https://drafts.csswg.org/selectors-5/#blank-pseudo) pseudo-class to Level 5. ([Issue 12799](https://github.com/w3c/csswg-drafts/issues/12799))

- Deferred the [grid-structural (column) selectors](https://www.w3.org/TR/2026/WD-selectors-4-20260122/selectors-5#grid-structural-selectors) to Level 5. ([Issue 12799](https://github.com/w3c/csswg-drafts/issues/12799))

- Deferred the [time-dimensional pseudo-classes](https://www.w3.org/TR/2026/WD-selectors-4-20260122/selectors-5#time-pseudos) to Level 5. ([Issue 12799](https://github.com/w3c/csswg-drafts/issues/12799))

### <a id="changes-2022-05"></a>18.2.  Changes since the 7 May 2022 Working Draft

Significant changes since the [7 May 2022 Working Draft](https://www.w3.org/TR/2022/WD-selectors-4-20220507/):

- <a id="ref-for-selectordef-open④"></a>

  Added [:open](#selectordef-open) pseudo-class. ([Issue 7319](https://github.com/w3c/csswg-drafts/issues/7319), [Issue 11039](https://github.com/w3c/csswg-drafts/issues/11039))

- <a id="ref-for-pseudo-element③④"></a>

  <a id="ref-for-has-pseudo①⑤"></a>

  Disallowed [pseudo-elements](#pseudo-element) from [:has()](#has-pseudo) unless explicitly allowed by the pseudo-element’s definition. ([Issue 7463](https://github.com/w3c/csswg-drafts/issues/7463))

- <a id="ref-for-has-pseudo①⑥"></a>

  Disallowed nesting of [:has()](#has-pseudo). ([Issue 7344](https://github.com/w3c/csswg-drafts/issues/7344))

- Defined matching of ::lang("") and of elements not tagged with a language. ([Issue 6915](https://github.com/w3c/csswg-drafts/issues/6915))

- Untangled the concepts of "scoped" and "relative" selectors completely. ([Issue 6399](https://github.com/w3c/csswg-drafts/issues/6399))

  - Removed "absolutize a selector" as well, and just defined relative selector matching in terms of the anchoring element.

- <a id="ref-for-nth-child-pseudo⑦"></a>

  Reverted compound selector limitation on [:nth-child()](#nth-child-pseudo). ([Issue 3760](https://github.com/w3c/csswg-drafts/issues/3760))

- <a id="ref-for-legacy-selector-alias②"></a>

  Defined :-webkit-autofill [legacy selector alias](#legacy-selector-alias). ([Issue 7474](https://github.com/w3c/csswg-drafts/issues/7474))

### <a id="changes-2018-11"></a>18.3.  Changes since the 21 November 2018 Working Draft

Significant changes since the [21 November 2018 Working Draft](https://www.w3.org/TR/2018/WD-selectors-4-20181121/):

- <a id="ref-for-has-pseudo①⑦"></a>

  Removed the Selector profiles, marked [:has()](#has-pseudo) as optional and at-risk instead. ([Issue 3925](https://github.com/w3c/csswg-drafts/issues/3925))

- <a id="ref-for-sub-pseudo-element③"></a>

  Added [§ 3.6.4 Sub-pseudo-elements](#sub-pseudo-elements) to define [sub-pseudo-elements](#sub-pseudo-element) and related terminology.

- <a id="ref-for-defined-pseudo③"></a>

  Added [:defined](#defined-pseudo). ([Issue 2258](https://github.com/w3c/csswg-drafts/issues/2258))

- <a id="ref-for-selectordef-modal④"></a>

  Added [:modal](#selectordef-modal). ([Issue 6965](https://github.com/w3c/csswg-drafts/issues/6965))

- <a id="ref-for-selectordef-picture-in-picture①"></a>

  <a id="ref-for-selectordef-fullscreen②"></a>

  Added [:fullscreen](#selectordef-fullscreen) and [:picture-in-picture](#selectordef-picture-in-picture). ([Issue 3796](https://github.com/w3c/csswg-drafts/issues/3796))

- <a id="ref-for-selectordef-stalled②"></a>

  <a id="ref-for-selectordef-buffering③"></a>

  <a id="ref-for-selectordef-seeking①"></a>

  Added [:seeking](#selectordef-seeking), [:buffering](#selectordef-buffering), and [:stalled](#selectordef-stalled) media playback state pseudo-classes. ([Issue 3821](https://github.com/w3c/csswg-drafts/issues/3821))

- <a id="ref-for-selectordef-volume-locked①"></a>

  <a id="ref-for-selectordef-muted①"></a>

  Added [:muted](#selectordef-muted) and [:volume-locked](#selectordef-volume-locked) sound state pseudo-classes. ([Issue 3821](https://github.com/w3c/csswg-drafts/issues/3821) and [Issue 3933](https://github.com/w3c/csswg-drafts/issues/3933))

- <a id="ref-for-selectordef-autofill②"></a>

  Added [:autofill](#selectordef-autofill). ([Issue 5775](https://github.com/w3c/csswg-drafts/issues/5775))

- <a id="ref-for-user-valid-pseudo②"></a>

  Added [:user-valid](#user-valid-pseudo). ([Discussion](https://lists.w3.org/Archives/Public/www-style/2015Sep/0111.html))

- <a id="ref-for-nth-last-child-pseudo④"></a>

  <a id="ref-for-nth-child-pseudo⑧"></a>

  <a id="ref-for-has-pseudo①⑧"></a>

  <a id="ref-for-where-pseudo①⓪"></a>

  <a id="ref-for-matches-pseudo②②"></a>

  Defined [:is()](#matches-pseudo), [:where()](#where-pseudo), [:has()](#has-pseudo), [:nth-child()](#nth-child-pseudo), and [:nth-last-child()](#nth-last-child-pseudo) to not be themselves invalidated when containing an invalid selector. ([Issue 3264](https://github.com/w3c/csswg-drafts/issues/3264))

- <a id="ref-for-compound④④"></a>

  <a id="ref-for-nth-last-child-pseudo⑤"></a>

  <a id="ref-for-nth-child-pseudo⑨"></a>

  Limited selectors in [:nth-child()](#nth-child-pseudo) and [:nth-last-child()](#nth-last-child-pseudo) to [compound selectors](#compound) for now. ([Issue 3760](https://github.com/w3c/csswg-drafts/issues/3760))

- Clarified case-sensitive string matching by referencing string identity as defined in [\[INFRA\]](#biblio-infra).

- <a id="ref-for-placeholder-shown-pseudo②"></a>

  Clarified that UA-provided placeholder text still triggers [:placeholder-shown](#placeholder-shown-pseudo).

- <a id="ref-for-focus-visible-pseudo⑤"></a>

  Rewrote [:focus-visible](#focus-visible-pseudo) definition for clarity.

- <a id="ref-for-typedef-combinator⑥"></a>

  <a id="ref-for-typedef-compound-selector⑧"></a>

  Switched reminder note in the grammar section to normative text describing the requirement of whitespace between [\<compound-selector\>](#typedef-compound-selector)s when a [\<combinator\>](#typedef-combinator) token is missing.

### <a id="changes-2018-02"></a>18.4.  Changes since the 2 February 2018 Working Draft

Significant changes since the [2 February 2018 Working Draft](https://www.w3.org/TR/2018/WD-selectors-4-20180202/):

- <a id="ref-for-where-pseudo①①"></a>

  Named the zero-specificity selector to [:where()](#where-pseudo). ([Issue 2143](https://github.com/w3c/csswg-drafts/issues/2143))

- <a id="ref-for-matches-pseudo②③"></a>

  <a id="ref-for-selectordef-matches"></a>

  Renamed [:matches()](#selectordef-matches) to [:is()](#matches-pseudo). ([Issue 3258](https://github.com/w3c/csswg-drafts/issues/3258))

- <a id="ref-for-empty-pseudo②"></a>

  Redefined [:empty](#empty-pseudo) to ignore white-space–only nodes. ([Issue 1967](https://github.com/w3c/csswg-drafts/issues/1967))

- <a id="ref-for-blank-pseudo①"></a>

  Redefined [:blank](https://drafts.csswg.org/selectors-5/#blank-pseudo) to represent empty user input, rather than empty elements. ([Issue 1283](https://github.com/w3c/csswg-drafts/issues/1283))

- <a id="ref-for-nth-child-pseudo①⓪"></a>

  <a id="ref-for-has-pseudo①⑨"></a>

  <a id="ref-for-matches-pseudo②④"></a>

  Changed the specificity of [:is()](#matches-pseudo), [:has()](#has-pseudo), and [:nth-child()](#nth-child-pseudo) to not depend on which selector argument matched. ([Issue 1027](https://github.com/w3c/csswg-drafts/issues/1027))

- Dropped the :drop() pseudo-classes since HTML dropped the related feature. ([Issue 2257](https://github.com/w3c/csswg-drafts/issues/2257))

- Added the case-sensitive flag `s` to the attribute selector. ([Issue 2101](https://github.com/w3c/csswg-drafts/issues/2101))

- <a id="ref-for-focus-visible-pseudo⑥"></a>

  Added further guidance on [:focus-visible](#focus-visible-pseudo).

- Added [Appendix B: Obsolete but Required -webkit- Parsing Quirks for Web Compat](#compat) defining ::-webkit- pseudo-element parsing quirk. ([Issue 3051](https://github.com/w3c/csswg-drafts/issues/3051))

- Rewrote grammar rules about where white space is allowed for clarity. (See [§ 16 Grammar](#grammar).)

### <a id="changes-2013"></a>18.5.  Changes since the 2 May 2013 Working Draft

Significant changes since the [2 May 2013 Working Draft](https://www.w3.org/TR/2013/WD-selectors4-20130502/) include:

- <a id="ref-for-selectordef-paused①"></a>

  <a id="ref-for-selectordef-playing③"></a>

  <a id="ref-for-focus-visible-pseudo⑦"></a>

  <a id="ref-for-focus-within-pseudo④"></a>

  <a id="ref-for-target-within-pseudo"></a>

  Added the [:target-within](https://www.w3.org/TR/selectors-4/#target-within-pseudo), [:focus-within](#focus-within-pseudo), [:focus-visible](#focus-visible-pseudo), [:playing](#selectordef-playing), and [:paused](#selectordef-paused) pseudo-classes.

- <a id="ref-for-selectordef-matches①"></a>

  Added a zero-specificity [:matches()](#selectordef-matches)-type pseudo-class, with name TBD.

- <a id="ref-for-has-pseudo②⓪"></a>

  Replaced subject indicator (!) feature with [:has()](#has-pseudo).

- Replaced the :nth-match() and :nth-last-match() selectors with :nth-child(… of <var>selector</var>) and :nth-last-child(… of <var>selector</var>).

- Changed the :active-drop-target, :valid-drop-target, :invalid-drop-target with :drop().

- Sketched out an empty-or-whitespace-only selector for discussion (See [open issue](https://github.com/w3c/csswg-drafts/issues/1967).)

- <a id="ref-for-user-invalid-pseudo④"></a>

  Renamed :user-error to [:user-invalid](#user-invalid-pseudo). (See [Discussion](https://www.w3.org/mid/CADhPm3v+WfeGQfBwwx8QBuiOjn2k38V_DcKW17Cm81VgZb1nbQ@mail.gmail.com))

- <a id="ref-for-selectordef-column"></a>

  <a id="ref-for-nth-last-col-pseudo"></a>

  <a id="ref-for-nth-col-pseudo"></a>

  Renamed :nth-column()/:nth-last-column() to [:nth-col()](https://www.w3.org/TR/selectors-4/#nth-col-pseudo)/[:nth-last-col()](https://www.w3.org/TR/selectors-4/#nth-last-col-pseudo) to avoid naming confusion with a potential [::column](https://www.w3.org/TR/css-multicol-2/#selectordef-column) pseudo-class.

- Changed the non-functional form of the :local-link pseudo-class to account for fragment URLs.

- Removed the functional form of the `:local-link()` pseudo-class and reference combinator for lack of interest.

- Rewrote selectors grammar using the CSS Value Definition Syntax.

- <a id="ref-for-scoped-selector③"></a>

  <a id="ref-for-relative-selector②"></a>

  Split out [relative selectors](#relative-selector) from [scoped selectors](#scoped-selector), as these are different concepts that can be independently invoked.

- <a id="ref-for-anb-production"></a>

  Moved definition of [\<An+B\>](https://www.w3.org/TR/css-syntax-3/#anb-production) microsyntax to CSS Syntax.

  > <strong data-conversion-semantic="issue">Issue</strong>
  >
  > <a id="issue-fb60ae39"></a> Semantic definition should probably move back here.

- Added new sections:
  - [§ 3.2 Data Model](#data-model)
    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-4ca3978c"></a> Need to define tree for XML.
  - [§ 17 API Hooks](#api-hooks)
    - <a id="ref-for-match-a-selector-against-a-tree"></a>

      Note that earlier versions of this section defined a section on <a id="evaluating-selectors"></a>evaluating a selector, but that section is no longer present. Specifications referencing that section should instead reference the algorithm to [match a selector against a tree](#match-a-selector-against-a-tree).

- <a id="ref-for-negation-pseudo⑨"></a>

  <a id="ref-for-selectordef-matches②"></a>

  Removed restriction on combinators within [:matches()](#selectordef-matches) and [:not()](#negation-pseudo); see [discussion](https://lists.w3.org/Archives/Public/www-style/2014Jan/0607.html).

- <a id="ref-for-selector-list①③"></a>

  <a id="ref-for-specificity②"></a>

  Defined [specificity](#specificity) of a [selector list](#selector-list). (Why?)

- <a id="ref-for-lang-pseudo⑥"></a>

  Required quotes around [:lang()](#lang-pseudo) values involving an asterisk; only language codes which happen to be CSS identifiers can be used unquoted.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The 1 February 2018 draft included an inadvertent commit of unfinished work; 2 February 2018 has reverted this commit (and fixed some links because why not).

### <a id="changes-2012"></a>18.6.  Changes since the 23 August 2012 Working Draft

Significant changes since the [23 August 2012 Working Draft](https://www.w3.org/TR/2012/WD-selectors4-20120823/) include:

- <a id="ref-for-placeholder-shown-pseudo③"></a>

  Added [:placeholder-shown](#placeholder-shown-pseudo) pseudo-classes.

- <a id="ref-for-negation-pseudo①⓪"></a>

  <a id="ref-for-selectordef-matches③"></a>

  Released some restrictions on [:matches()](#selectordef-matches) and [:not()](#negation-pseudo).

- Defined fast and complete Selectors profiles (now called “live” and “snapshot”).

- <a id="ref-for-selectordef-matches④"></a>

  <a id="ref-for-specificity③"></a>

  Improved definition of [specificity](#specificity) to better handle [:matches()](#selectordef-matches).

- Updated grammar.

- <a id="ref-for-anb-production①"></a>

  Cleaned up definition of [\<An+B\>](https://www.w3.org/TR/css-syntax-3/#anb-production) notation.

- Added definition of <i>scope-relative</i> selectors, changed <i>scope-constrained</i> to scope-filtered for less confusion with scope-contained.

- The :local-link() pseudo-class now ignores trailing slashes.

### <a id="changes-2011"></a>18.7.  Changes since the 29 September 2011 Working Draft

Significant changes since the [29 September 2011 Working Draft](https://www.w3.org/TR/2011/WD-selectors4-20110929/) include:

- Added language variant handling per RFC 4647.

- Added scoped selectors.

- <a id="ref-for-user-invalid-pseudo⑤"></a>

  Added :user-error (now called [:user-invalid](#user-invalid-pseudo)).

- Added :valid-drop-target.

- <a id="ref-for-column-combinator"></a>

  Changed [column combinator](https://www.w3.org/TR/selectors-4/#column-combinator) from double slash to double pipe.

### <a id="changes-level-3"></a>18.8.  Changes Since Level 3

Additions since [Level 3](https://www.w3.org/TR/selectors-3/):

- <a id="ref-for-negation-pseudo①①"></a>

  Extended [:not()](#negation-pseudo) to accept a selector list.

- <a id="ref-for-has-pseudo②①"></a>

  <a id="ref-for-where-pseudo①②"></a>

  <a id="ref-for-matches-pseudo②⑤"></a>

  Added [:is()](#matches-pseudo) and [:where()](#where-pseudo) and [:has()](#has-pseudo).

- <a id="ref-for-scope-pseudo③"></a>

  Added [:scope](#scope-pseudo).

- <a id="ref-for-any-link-pseudo①"></a>

  Added [:any-link](#any-link-pseudo).

- <a id="ref-for-focus-visible-pseudo⑧"></a>

  <a id="ref-for-focus-within-pseudo⑤"></a>

  <a id="ref-for-target-within-pseudo①"></a>

  Added [:target-within](https://www.w3.org/TR/selectors-4/#target-within-pseudo), [:focus-within](#focus-within-pseudo), and [:focus-visible](#focus-visible-pseudo).

- <a id="ref-for-dir-pseudo③"></a>

  Added [:dir()](#dir-pseudo).

- <a id="ref-for-lang-pseudo⑦"></a>

  Expanded [:lang()](#lang-pseudo) to accept wildcard matching and lists of language codes.

- Expanded :nth-child() to accept a selector list.

- <a id="ref-for-indeterminate-pseudo⑥"></a>

  Merged in input selectors from [CSS Basic User Interface Module Level 3](https://www.w3.org/TR/css-ui-3/) and added back [:indeterminate](#indeterminate-pseudo).

- <a id="ref-for-user-invalid-pseudo⑥"></a>

  Added [:user-invalid](#user-invalid-pseudo).

- Added case-insensitive / case-sensitive attribute-value matching flags.

## <a id="acknowledgements"></a>19.  Acknowledgements

The CSS working group would like to thank everyone who contributed to the [previous Selectors](https://www.w3.org/TR/css3-selectors) specifications over the years, as those specifications formed the basis for this one. In particular, the working group would like to extend special thanks to the following for their specific contributions to Selectors Level 4: L. David Baron, Andrew Fedoniouk, Daniel Glazman, Ian Hickson, Grey Hodge, Lachlan Hunt, Anne van Kesteren, Jason Cranford Teague, Lea Verou

## <a id="privacy"></a>Privacy Considerations

- <a id="ref-for-visited-pseudo②⓪"></a>

  The [:visited](#visited-pseudo) pseudo-class can expose information about which sites a user has previously visited, if the UA is not careful to screen from scripting any information that would reveal which elements match it.

- <a id="ref-for-selectordef-autofill③"></a>

  The [:autofill](#selectordef-autofill) pseudo-class can expose whether a user has interacted with this form before; however the same information can be derived by observing how quickly the form is filled out.

## <a id="security"></a>Security Considerations

The [Privacy Considerations](#privacy) could also be considered to affect Security.

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

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

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
- [evaluating a selector](#evaluating-selectors), in § 18.5
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

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-COLOR-5\] defines the following terms:
  - <a id="f3d71334"></a>a
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="2ccfe434"></a>display
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="2a8247fa"></a>box tree
  - <a id="af3737f8"></a>display type
  - <a id="c0b19878"></a>list-item
  - <a id="aceda213"></a>visibility
- \[CSS-MULTICOL-2\] defines the following terms:
  - <a id="b212b9cc"></a>::column
- \[CSS-POSITION-4\] defines the following terms:
  - <a id="e46d20c1"></a>top layer
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="70503bd6"></a>::after
  - <a id="7c6f51b7"></a>::before
  - <a id="63b59bd9"></a>::first-letter
  - <a id="4bda66a9"></a>::first-line
  - <a id="b6b63ba4"></a>::marker
  - <a id="00f795aa"></a>tree-abiding pseudo-elements
- \[CSS-SHADOW-1\] defines the following terms:
  - <a id="905d9a5a"></a>::slotted()
  - <a id="5561253e"></a>:host
  - <a id="37487143"></a>:host-context()
  - <a id="4cb3439f"></a>flat tree
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="219a15e3"></a>content language
  - <a id="f9fc4a3c"></a>document white space characters
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="81b3af3e"></a>!
  - <a id="c297b070"></a>\#
  - <a id="ef9f8297"></a>\*
  - <a id="dcecfc13"></a>\<ident\>
  - <a id="1d798932"></a>\<string\>
  - <a id="d4441b24"></a>?
  - <a id="ce7ea0de"></a>identifier
  - <a id="4eb9d37e"></a>\|
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
- \[CSS21\] defines the following terms:
  - <a id="74a27486"></a>:before
- \[CSS3NAMESPACE\] defines the following terms:
  - <a id="bf0e8186"></a>@namespace
  - <a id="8fba26d6"></a>CSS qualified name
  - <a id="c8858de1"></a>default namespace
- \[CSS3SYN\] defines the following terms:
  - <a id="b2862018"></a>\<an+b\>
  - <a id="4920620f"></a>\<any-value\>
  - <a id="a6331414"></a>\<function-token\>
  - <a id="7b8be35b"></a>\<hash-token\>
  - <a id="446c663e"></a>\<ident-token\>
  - <a id="17fe01a1"></a>\<string-token\>
  - <a id="67800454"></a>parse
  - <a id="cc7f03b8"></a>parse a list
- \[CSSOM-1\] defines the following terms:
  - <a id="bfb148e6"></a>getComputedStyle(elt)
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
  - <a id="91c016b5"></a>identical to
  - <a id="15e48c39"></a>set
  - <a id="0e8de730"></a>tuple
- \[MATHML-CORE\] defines the following terms:
  - <a id="cd257525"></a>math
- \[SELECT\] defines the following terms:
  - <a id="dfd67b05"></a>\*
- \[SELECTORS-4\] defines the following terms:
  - <a id="ea3ee634"></a>:local-link
  - <a id="0080ad69"></a>:nth-col()
  - <a id="29485ad2"></a>:nth-last-col()
  - <a id="f03eba48"></a>:target-within
  - <a id="877c1491"></a>\<forgiving-relative-selector-list\>
  - <a id="0a708963"></a>column combinator
- \[SELECTORS-5\] defines the following terms:
  - <a id="1bc79eaa"></a>:blank
  - <a id="ec885bc1"></a>:interest-source
  - <a id="9f5e7e1d"></a>:interest-target
- \[URL\] defines the following terms:
  - <a id="ed948033"></a>fragment
  - <a id="dcffbccd"></a>URL

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-bcp47"></a>\[BCP47\]  
A. Phillips, Ed.; M. Davis, Ed.. [Tags for Identifying Languages](https://www.rfc-editor.org/rfc/rfc5646). September 2009. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;rfc-editor&#x2E;org&#x2F;rfc&#x2F;rfc5646](https://www.rfc-editor.org/rfc/rfc5646)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 6 November 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-multicol-2"></a>\[CSS-MULTICOL-2\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 2](https://www.w3.org/TR/css-multicol-2/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-2&#x2F;](https://www.w3.org/TR/css-multicol-2/)

<a id="biblio-css-position-4"></a>\[CSS-POSITION-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 4](https://www.w3.org/TR/css-position-4/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-4&#x2F;](https://www.w3.org/TR/css-position-4/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-shadow-1"></a>\[CSS-SHADOW-1\]  
[CSS Shadow Module Level 1](https://drafts.csswg.org/css-shadow-1/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-shadow-1&#x2F;](https://drafts.csswg.org/css-shadow-1/)

<a id="biblio-css-text-4"></a>\[CSS-TEXT-4\]  
Elika Etemad; et al. [CSS Text Module Level 4](https://www.w3.org/TR/css-text-4/). 29 May 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-4&#x2F;](https://www.w3.org/TR/css-text-4/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3namespace"></a>\[CSS3NAMESPACE\]  
Elika Etemad. [CSS Namespaces Module Level 3](https://www.w3.org/TR/css-namespaces-3/). 20 March 2014. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-namespaces-3&#x2F;](https://www.w3.org/TR/css-namespaces-3/)

<a id="biblio-css3syn"></a>\[CSS3SYN\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-rfc4647"></a>\[RFC4647\]  
A. Phillips, Ed.; M. Davis, Ed.. [Matching of Language Tags](https://www.rfc-editor.org/rfc/rfc4647). September 2006. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;rfc-editor&#x2E;org&#x2F;rfc&#x2F;rfc4647](https://www.rfc-editor.org/rfc/rfc4647)

<a id="biblio-select"></a>\[SELECT\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-selectors-5"></a>\[SELECTORS-5\]  
[Selectors Level 5](https://drafts.csswg.org/selectors-5/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;selectors-5&#x2F;](https://drafts.csswg.org/selectors-5/)

<a id="biblio-url"></a>\[URL\]  
Anne van Kesteren. [URL Standard](https://url.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;url&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://url.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 13 January 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css3ui"></a>\[CSS3UI\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 21 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-cssstyleattr"></a>\[CSSSTYLEATTR\]  
Tantek Çelik; Elika Etemad. [CSS Style Attributes](https://www.w3.org/TR/css-style-attr/). 7 November 2013. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-style-attr&#x2F;](https://www.w3.org/TR/css-style-attr/)

<a id="biblio-fullscreen"></a>\[FULLSCREEN\]  
Philip Jägenstedt. [Fullscreen API Standard](https://fullscreen.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fullscreen&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fullscreen.spec.whatwg.org/)

<a id="biblio-html5"></a>\[HTML5\]  
Ian Hickson; et al. [HTML5](https://www.w3.org/TR/html5/). 27 March 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;html5&#x2F;](https://www.w3.org/TR/html5/)

<a id="biblio-its20"></a>\[ITS20\]  
David Filip; et al. [Internationalization Tag Set (ITS) Version 2.0](https://www.w3.org/TR/its20/). 29 October 2013. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;its20&#x2F;](https://www.w3.org/TR/its20/)

<a id="biblio-mathml"></a>\[MATHML\]  
Patrick D F Ion; Robert R Miner. [Mathematical Markup Language (MathML™) 1.01 Specification](https://www.w3.org/TR/REC-MathML/). 7 March 2023. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;REC-MathML&#x2F;](https://www.w3.org/TR/REC-MathML/)

<a id="biblio-mathml-core"></a>\[MATHML-CORE\]  
David Carlisle; Frédéric Wang. [MathML Core](https://www.w3.org/TR/mathml-core/). 24 June 2025. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mathml-core&#x2F;](https://www.w3.org/TR/mathml-core/)

<a id="biblio-picture-in-picture"></a>\[PICTURE-IN-PICTURE\]  
Francois Beaufort. [Picture-in-Picture](https://www.w3.org/TR/picture-in-picture/). 2 December 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;picture-in-picture&#x2F;](https://www.w3.org/TR/picture-in-picture/)

<a id="biblio-quirks"></a>\[QUIRKS\]  
Simon Pieters. [Quirks Mode Standard](https://quirks.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;quirks&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://quirks.spec.whatwg.org/)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

<a id="biblio-xforms11"></a>\[XFORMS11\]  
John Boyer. [XForms 1.1](https://www.w3.org/TR/xforms11/). 20 October 2009. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;xforms11&#x2F;](https://www.w3.org/TR/xforms11/)

<a id="biblio-xml-names"></a>\[XML-NAMES\]  
Tim Bray; et al. [Namespaces in XML 1.0 (Third Edition)](https://www.w3.org/TR/xml-names/). 8 December 2009. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;xml-names&#x2F;](https://www.w3.org/TR/xml-names/)

<a id="biblio-xml10"></a>\[XML10\]  
Tim Bray; et al. [Extensible Markup Language (XML) 1.0 (Fifth Edition)](https://www.w3.org/TR/xml/). 26 November 2008. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;xml&#x2F;](https://www.w3.org/TR/xml/)

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Add comma-separated syntax for [multiple-value matching](https://lists.w3.org/Archives/Public/www-style/2011Mar/0215.html)? e.g. \[rel ~= next, prev, up, first, last\] [↵](#issue-745ef775)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> There’s a desire from authors to propagate [:focus](#focus-pseudo) from a form control to its associated <code><a href="https://html.spec.whatwg.org/multipage/forms.html#the-label-element">label</a></code> element; the main objection seems to be implementation difficulty. See [CSSWG issue (CSS)](https://github.com/w3c/csswg-drafts/issues/397) and [WHATWG issue (HTML)](https://github.com/whatwg/html/issues/1632). [↵](#issue-1416193e)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Are these still necessary now that we have more rigorous definitions for [match](#match) and [invalid selector](#invalid-selector)? Nouns are a lot easier to coordinate across specification than predicates, and details like the exact order of elements returned from `querySelector` seem to make more sense being defined in the DOM specification than in Selectors. [↵](#issue-55d7bd68)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Only the [tree-abiding pseudo-elements](https://www.w3.org/TR/css-pseudo-4/#tree-abiding) are really handled in any way remotely like this. [↵](#issue-15f3317f)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The relative position of pseudo-elements in <var>selector match list</var> is undefined. There’s not yet a context that exposes this information, but we need to decide on something eventually, before something <em>is</em> exposed. [↵](#issue-b7f52036)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Some pseudo-classes are \*syntactical\*, like [:has()](#has-pseudo) and [:is()](#matches-pseudo), and thus should always work. Need to indicate that somewhere. Probably the structural pseudos always work whenever the child list is ordered. [↵](#issue-772ec278)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Semantic definition should probably move back here. [↵](#issue-fb60ae39)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Need to define tree for XML. [↵](#issue-4ca3978c)
