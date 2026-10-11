Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Selectors Level 5](https://drafts.csswg.org/selectors-5/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply. The capture’s full original notice and links remain below.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md).

# Source provenance

Title: Selectors Level 5

Source snapshot: https://drafts.csswg.org/selectors-5/

Retrieved: 2026-10-10. The captured page states Editor’s Draft, 18 August 2026. An undated editor URL can change; the retrieval date and exact byte hash identify this captured HTML.

Captured HTML SHA-256: 46c47dc5e45220c45944dc5cc6553b7f3ec5135d910b84a3faef50c7b4f3511a

Captured HTML revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Immutable upstream source: [Overview.bs](https://raw.githubusercontent.com/w3c/csswg-drafts/50337328484f7fcdd9ab6fe9ded6011815b1707e/selectors-5/Overview.bs), SHA-256 `bd9fd9abadfb552990fd435cdab0e89fca133273ff775b81b6aa6b5d5bf5d38b`. The source revision is inspected as reference material and is not executed.

Representation notes:

- Full format conversion of the captured HTML body, including status, metadata, bibliography, indexes, examples, test references, and legal notice. Script and style elements are omitted and were not executed.
- All source body IDs are retained, including those inside preformatted blocks, which are relocated immediately before those blocks. Fragment-only links stay local only when their exact captured target exists; other links remain upstream.
- All 0 tables have readable Markdown layouts. All tables have ordinary row/column layouts. Synthetic Field/Definition or Column N headings are non-normative. Source row headers remain bold; native HTML header/accessibility semantics are not expressible in GFM.
- Preformatted examples retain literal text. Single-line table examples use semantic inline code. Compiler-generated hyperlinks inside fenced code blocks are omitted while their IDs and visible code text are retained; other source links are preserved. Small semantic code, variable, emphasis, subscript, and superscript HTML remains where Markdown notation would alter text.
- Conversion uses Pandoc 3.1.11.1 with focused Python/lxml preparation and a semantic Lua filter; the parsed GFM output is checked against the captured HTML. The [conversion report](CONVERSION-REPORT.md) records provenance and check boundaries.
- This full edition is a selected planning source for unstarted selector execution and scope-cascade work. The [source-edition mapping](SOURCE-EDITIONS.md) distinguishes these planning selections from historical editions consumed by completed CSS work.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>Selectors Level 5

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 18 August 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/selectors-5/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/selectors-5/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/selectors-5)

[Inline In Spec](#issues-index)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Tab Atkins Jr.](http://xanthir.com/contact/) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/selectors-5/Overview.bs)

<strong>Delta Spec:</strong>

yes

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/selectors/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

<a id="ref-for-selector"></a>[Selectors](https://drafts.csswg.org/selectors-4/#selector) are patterns that match against elements in a tree, and as such form one of several technologies that can be used to select nodes in a document. Selectors have been optimized for use with HTML and XML, and are designed to be usable in performance-critical code. They are a core component of CSS (Cascading Style Sheets), which uses Selectors to bind style properties to elements in the document. Selectors Level 5 describes the selectors that already exist in [\[selectors-4\]](#biblio-selectors-4), and further introduces new selectors for CSS and other languages that may need them.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “selectors” in the title, like this: “\[selectors\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bselectors%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](#intro)
    1.  [1.1 Module Interactions](#placement)
2.  [2 Attribute Selectors](#attribute-selectors)
    1.  [2.1 Class Prefix Selectors](#class-prefix)
3.  [3 Location Pseudo-classes](#location)
    1.  [3.1 The Local Link Pseudo-class :local-link](#local-pseudo)
4.  [4 User Action Pseudo-classes](#useraction-pseudos)
    1.  [4.1 The Interest Pseudo-classes: :interest-source and :interest-target](#interest-pseudos)
5.  [5 The Input Pseudo-classes](#input-pseudos)
    1.  [5.1 Input Value-checking](#ui-validity)
        1.  [5.1.1 The Empty-Value Pseudo-class: :blank](#blank)
6.  [6 Time-dimensional Pseudo-classes](#time-pseudos)
    1.  [6.1 The Current-element Pseudo-class: :current](#the-current-pseudo)
    2.  [6.2 The Past-element Pseudo-class: :past](#the-past-pseudo)
    3.  [6.3 The Future-element Pseudo-class: :future](#the-future-pseudo)
7.  [7 Exposing custom state: the :state() pseudo-class](#custom-state)
8.  [8 Heading Structures: the heading pseudo-classes :heading, and :heading()](#headings)
9.  [9 Grid-Structural Selectors](#grid-structural-selectors)
    1.  [9.1 Column combinator (<code>&#124;&#124;</code>)](#the-column-combinator)
    2.  [9.2 :nth-col() pseudo-class](#the-nth-col-pseudo)
    3.  [9.3 :nth-last-col() pseudo-class](#the-nth-last-col-pseudo)
10. [10 Combinators](#combinators)
    1.  [10.1 Reference combinators <code>/ref/</code>](#idref-combinators)
11. [11 Grammar](#grammar)
12. [12 Changes](#changes)
    1.  [12.1 Changes Since the First Public Working Draft of 17 February 2026](#changes-20260415)
    2.  [12.2 Changes Since Level 4](#changes-level-4)
13. [13 Acknowledgements](#acknowledgements)
14. [ Privacy Considerations](#privacy)
15. [ Security Considerations](#security)
16. [ Conformance](#w3c-conformance)
    1.  [ Document conventions](#w3c-conventions)
    2.  [ Conformance classes](#w3c-conformance-classes)
    3.  [ Partial implementations](#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](#w3c-testing)
17. [ Index](#index)
    1.  [ Terms defined by this specification](#index-defined-here)
    2.  [ Terms defined by reference](#index-defined-elsewhere)
18. [ References](#references)
    1.  [ Normative References](#normative)
    2.  [ Non-Normative References](#informative)
19. [ Issues Index](#issues-index)

## <a id="intro"></a>1.  Introduction[](#intro)

<a id="issue-edc8dd61"></a>

<strong>Issue:</strong>

[](#issue-edc8dd61) <strong>This is a diff spec against [Selectors Level 4](https://www.w3.org/TR/selectors-4/).</strong>

### <a id="placement"></a>1.1.  Module Interactions[](#placement)

This module extends the set of selectors defined for CSS in [\[selectors-4\]](#biblio-selectors-4).

## <a id="attribute-selectors"></a>2.  Attribute Selectors[](#attribute-selectors)

### <a id="class-prefix"></a>2.1.  Class Prefix Selectors[](#class-prefix)

The <a id="class-prefix-selector"></a><strong>class prefix selector</strong> is a variant of the <a id="ref-for-class-selector"></a>[class selector](https://drafts.csswg.org/selectors-4/#class-selector). It consists of a full stop (. U+002E), immediately followed by an identifier that ends in one or more hyphens (- U+002D), immediately followed by an asterisk (\* U+002A), such as <code>&#46;foo-&#42;</code>.

A <a id="ref-for-class-prefix-selector"></a>[class prefix selector](#class-prefix-selector) represents an element which would match a <a id="ref-for-class-selector①"></a>[class selector](https://drafts.csswg.org/selectors-4/#class-selector) that started with the given prefix (the ident between the dot and asterisk). A class is considered to start with the given prefix if and only if it begins with that prefix, has at least one character beyond the prefix, and the first such character beyond the prefix is not also a hyphen.

<a id="example-5a086eb4"></a>

<strong>Example:</strong>

[](#example-5a086eb4) For example, an element with <code>class=&#34;foo-bar-baz&#34;</code> would match .foo-\* or .foo-bar-\*.

An element with <code>class=&#34;foo--bar&#34;</code>, such as is sometimes used in BEM-style naming, would match .foo--\*, but <strong>would not match</strong> .foo-\*. (The separator between those two units is --, not -.)

An element with <code>class=&#34;foo-bar-&#34;</code> (ending in a dash) would match .foo-\*, but <strong>would not match</strong> .foo-bar-\*. (That’s no longer a prefix, it’s the whole class).

<a id="example-d315b93d"></a>

<strong>Example:</strong>

[](#example-d315b93d) This syntax specifically matches dash-separated prefixes of a class; it does not match \*arbitrary\* prefixes. For example, .foo\* is simply an invalid selector (it’s a <a id="ref-for-class-selector②"></a>[class selector](https://drafts.csswg.org/selectors-4/#class-selector) followed by a <a id="ref-for-universal-selector"></a>[universal selector](https://drafts.csswg.org/selectors-4/#universal-selector), rather than being a <a id="ref-for-class-prefix-selector①"></a>[class prefix selector](#class-prefix-selector)), and will not match elements with <code>class=&#34;foobar&#34;</code>.

<strong>Note:</strong>

Why not?

There are two reasons we don’t allow arbitrary prefixes. For one, they’re easy to mess up; it’s pretty easy to \*accidentally\* have classes that start with a prefix that you didn’t intend to match. For example, if you use .foo\* and intend it to match <code>class=&#34;foo1&#34;</code>, <code>&#34;foo2&#34;</code>, etc, it would <em>also</em> match <code>class=&#34;footer&#34;</code>, which is probably unintentional since the "foo" isn’t a semantic unit of the class name. Limiting to dash prefixes makes this sort of confusion nearly impossible.

For two, it’s a lot more efficient in user agents. UAs use techniques like [Bloom Filters](https://en.wikipedia.org/wiki/Bloom_filter) to cheaply and quickly test whether a class might exist anywhere in a subtree, without having to examine every element individually. Arbitrary prefixes would require adding every prefix of every class into the filters, which would balloon their size for very little gain, or force UAs to bypass the filters entirely and check the prefix against every element manually. Limiting to dash prefixes means we add relatively few additional entries to the filters, and they’re a lot more likely to be useful since they automatically represent a semantic "unit" of the class name.

Note: This feature is still somewhat experimental. Other separators, like <code>&#95;</code>, might be added as possibilities in the future.

## <a id="location"></a>3.  Location Pseudo-classes[](#location)

### <a id="local-pseudo"></a>3.1.  The Local Link Pseudo-class <a id="ref-for-local-link-pseudo"></a>[:local-link](#local-link-pseudo)[](#local-pseudo)

The <a id="local-link-pseudo"></a><strong>:local-link</strong> pseudo-class allows authors to style [hyperlinks](https://drafts.csswg.org/selectors-4/#the-any-link-pseudo) based on the users current location within a site and to differentiate site-internal versus site-external links.

The (non-functional) <a id="ref-for-local-link-pseudo①"></a>[:local-link](#local-link-pseudo) pseudo-class represents an element that is the source anchor of a hyperlink whose target’s absolute URL matches the element’s own document URL. Any fragment identifiers are stripped before matching the document’s URL against the link’s URL; otherwise all portions of the URL are considered.

<a id="example-d2555a3c"></a>

<strong>Example:</strong>

[](#example-d2555a3c) For example, the following rule prevents links targeting the current page from being underlined when they are part of the navigation list:

``` text
nav :local-link { text-decoration: none; } 
```

As a functional pseudo-class, :local-link() can also accept a non-negative integer as its sole argument, which, if the document’s URL belongs to a hierarchical scheme, indicates the number of path levels to match:

- :local-link(0) represents a link element whose target is in the same origin as the document’s URL
- :local-link(1) represents a link element whose target has the same origin and first path segment
- :local-link(2) represents a link element whose target has the same origin, first, and second path segments
- etc.

<a id="example-a43b38c7"></a>

<strong>Example:</strong>

[](#example-a43b38c7) The following example styles all site-external links with a dashed underline.

``` text
:not(:local-link(0)) { text-decoration-style: dashed; } 
```

Path segments are portions of the URL’s path that are separated by forward slashes (/). If a segment is missing from the document’s URL, a pseudo-class requiring that segment to match does not match anything.

<a id="example-c9a76ca6"></a>

<strong>Example:</strong>

[](#example-c9a76ca6) So, given the links:

1.  <code>&#60;a href=&#34;http&#58;//www&#46;example&#46;com&#34;&#62;Home&#60;/a&#62;</code>
2.  <code>&#60;a href=&#34;http&#58;//www&#46;example&#46;com/2011&#34;&#62;2011&#60;/a&#62;</code>
3.  <code>&#60;a href=&#34;http&#58;//www&#46;example&#46;com/2011/03&#34;&#62;March&#60;/a&#62;</code>
4.  <code>&#60;a href=&#34;http&#58;//www&#46;example&#46;com/2011/03/&#34;&#62;March&#60;/a&#62;</code>
5.  <code>&#60;a href=&#34;http&#58;//www&#46;example&#46;com/2011/03/21&#34;&#62;21 March&#60;/a&#62;</code>
6.  <code>&#60;a href=&#34;https&#58;//www&#46;example&#46;com/2011/03/&#34;&#62;March&#60;/a&#62;</code>
7.  <code>&#60;a href=&#34;http&#58;//example&#46;com/2011/03/&#34;&#62;March&#60;/a&#62;</code>

and the styles:

1.  <code>a&#58;local-link {&#46;&#46;&#46;}</code>
2.  <code>a&#58;local-link(0) {&#46;&#46;&#46;}</code>
3.  <code>a&#58;local-link(1) {&#46;&#46;&#46;}</code>
4.  <code>a&#58;local-link(2) {&#46;&#46;&#46;}</code>
5.  <code>a&#58;local-link(3) {&#46;&#46;&#46;}</code>
6.  <code>a&#58;local-link(4) {&#46;&#46;&#46;}</code>

If the document’s URL is <code>http&#58;//www&#46;example&#46;com/2011/03/</code>:

1.  Link 1 would receive Style B
2.  Link 2 would receive Styles B and C
3.  Link 3 would receive Styles B, C, and D
4.  Link 4 would also receive Styles A, B, C, D, and E
5.  Link 5 would receive Styles B, C, and D
6.  Link 6 would remain unstyled
7.  Link 7 would remain unstyled
8.  Style F would not be applied to anything

The "origin" of the URL is defined by [RFC 6454, Section 4](http://tools.ietf.org/html/rfc6454#section-4). The username, password, query string, and fragment portions of the URL are not considered when matching against :local-link(<var>n</var>). If the document’s URL does not belong to a hierarchical scheme, the functional pseudo-class matches nothing.

<a id="issue-dd3371ba"></a>

<strong>Issue:</strong>

[](#issue-dd3371ba) It’s clear that, if the document URL has at least N segments, then :local-link(N) only matches links whose URL has at least N segments. (This lets you assign consistent semantics to :local-link so that, for example, :local-link(2) means a "within-repo" link on GitHub.) What about if the document url has less than N segments, and the link is same-page? Should "null segments" count as matching, or not?

## <a id="useraction-pseudos"></a>4.  User Action Pseudo-classes[](#useraction-pseudos)

### <a id="interest-pseudos"></a>4.1.  The Interest Pseudo-classes: <a id="ref-for-selectordef-interest-source"></a>[:interest-source](#selectordef-interest-source) and <a id="ref-for-selectordef-interest-target"></a>[:interest-target](#selectordef-interest-target)[](#interest-pseudos)

A common UI feature is the ability for the user to "show interest" in certain elements (an <a id="interest-source"></a><strong>interest source</strong>), and the UI to respond to that interest by showing additional information in another element (the <a id="interest-target"></a><strong>interest target</strong>), typically a popup. For example, hovering a username on a page might bring up an information card for that user; or long-pressing a link might bring up a preview of the link’s destination. Whenever this happens, the first element

Note: Exactly what elements are capable of being <a id="ref-for-interest-source"></a>[interest sources](#interest-source), how they’re linked to <a id="ref-for-interest-target"></a>[interest targets](#interest-target), and how "interest" is shown are all <a id="ref-for-host-language"></a>[host language](https://drafts.csswg.org/selectors-4/#host-language)-defined. In HTML, for example, this is done with the <u><code>interestfor</code></u> attribute to indicate an element capable of "showing interest" (being an <a id="ref-for-interest-source①"></a>interest source, and that attribute points to another element intended to show the additional information (the associated <a id="ref-for-interest-target①"></a>interest target) (usually a <code><a id="ref-for-attr-popover"></a>[popover](https://html&#46;spec&#46;whatwg&#46;org/multipage/popover&#46;html#attr-popover)</code> element).

The <a id="selectordef-interest-source"></a><strong>:interest-source</strong> pseudo-class applies to an <a id="ref-for-interest-source②"></a>[interest source](#interest-source) element that the user is currently "showing interest" in, and the <a id="selectordef-interest-target"></a><strong>:interest-target</strong> pseudo-class applies to the associated <a id="ref-for-interest-target②"></a>[interest target](#interest-target) of an element matching <a id="ref-for-selectordef-interest-source①"></a>[:interest-source](#selectordef-interest-source).

Note: <a id="ref-for-selectordef-interest-source②"></a>[:interest-source](#selectordef-interest-source) only matches an <a id="ref-for-interest-source③"></a>[interest source](#interest-source) the user is <em>currently</em> interested in, not any element that can <em>potentially</em> have interest shown in it. In HTML, for example, just having the <u><code>interestfor</code></u> attribute does not make an element match this pseudo-class; the user has to actually indicate interest in it too. Similarly, <a id="ref-for-selectordef-interest-target①"></a>[:interest-target](#selectordef-interest-target) only matches an <a id="ref-for-interest-target③"></a>[interest target](#interest-target) that is linked to an element the user is <em>currently</em> interested in, not any element pointed to by an <u><code>interestfor</code></u> attribute.

## <a id="input-pseudos"></a>5.  The Input Pseudo-classes[](#input-pseudos)

### <a id="ui-validity"></a>5.1.  Input Value-checking[](#ui-validity)

#### <a id="blank"></a>5.1.1.  The Empty-Value Pseudo-class: <a id="ref-for-blank-pseudo"></a>[:blank](#blank-pseudo)[](#blank)

The <a id="blank-pseudo"></a><strong>:blank</strong> pseudo-class applies to user-input elements whose input value is empty (consists of the empty string or otherwise null input).

Roughly speaking, if a human looked at a printout of the form and would say it’s blank, it matches

<strong>Note:</strong>

<a id="ref-for-blank-pseudo①"></a>[:blank](#blank-pseudo).

A rule of thumb for interpreting <a id="ref-for-blank-pseudo②"></a>[:blank](#blank-pseudo) on form controls is:

- If the control always submits, and would do so with an empty string, it matches <a id="ref-for-blank-pseudo③"></a>[:blank](#blank-pseudo). (Such as HTML’s <code>&#60;input type=text&#62;</code> when its value is empty.)

- If it sometimes submits, and is set to not submit, it matches <a id="ref-for-blank-pseudo④"></a>[:blank](#blank-pseudo). (Such as HTML’s <code>&#60;input type=checkbox&#62;</code> when not checked.)

- If it’s an “action button” (rather than a “toggle button” that represents a state) such as <code>&#60;button&#62;</code>, <code>&#60;input type=submit&#62;</code>, etc., it never matches <a id="ref-for-blank-pseudo⑤"></a>[:blank](#blank-pseudo).

Host languages can specify more precise rules for when form controls match <a id="ref-for-blank-pseudo⑥"></a>[:blank](#blank-pseudo).

## <a id="time-pseudos"></a>6.  Time-dimensional Pseudo-classes[](#time-pseudos)

These pseudo-classes classify elements with respect to the currently-displayed or active position in some timeline, such as during speech rendering of a document, or during the display of a video using WebVTT to render subtitles.

CSS does not define this timeline; the host language must do so. If there is no timeline defined for an element, these pseudo-classes must not match the element.

Note: Ancestors of a <a id="ref-for-current-pseudo"></a>[:current](#current-pseudo) element are also <a id="ref-for-current-pseudo①"></a>:current, but ancestors of a <a id="ref-for-past-pseudo"></a>[:past](#past-pseudo) or <a id="ref-for-future-pseudo"></a>[:future](#future-pseudo) element are not necessarily <a id="ref-for-past-pseudo①"></a>:past or <a id="ref-for-future-pseudo①"></a>:future as well. A given element matches at most one of <a id="ref-for-current-pseudo②"></a>:current, <a id="ref-for-past-pseudo②"></a>:past, or <a id="ref-for-future-pseudo②"></a>:future.

### <a id="the-current-pseudo"></a>6.1.  The Current-element Pseudo-class: <a id="ref-for-current-pseudo③"></a>[:current](#current-pseudo)[](#the-current-pseudo)

The <a id="current-pseudo"></a><strong>:current</strong> pseudo-class represents the element, or an ancestor of the element, that is currently being displayed.

Its alternate form <a id="selectordef-current"></a><strong>:current()</strong> takes a list of <a id="ref-for-compound"></a>[compound selectors](https://drafts.csswg.org/selectors-4/#compound) as its argument: it represents the <a id="ref-for-current-pseudo④"></a>[:current](#current-pseudo) element that matches the argument or, if that does not match, the innermost ancestor of the <a id="ref-for-current-pseudo⑤"></a>:current element that does. (If neither the <a id="ref-for-current-pseudo⑥"></a>:current element nor its ancestors match the argument, then the selector does not represent anything.)

<a id="example-34db2617"></a>

<strong>Example:</strong>

[](#example-34db2617) For example, the following rule will highlight whichever paragraph or list item is being read aloud in a speech rendering of the document:

``` text
:current(p, li, dt, dd) {
  background: yellow;
}
```

### <a id="the-past-pseudo"></a>6.2.  The Past-element Pseudo-class: <a id="ref-for-past-pseudo③"></a>[:past](#past-pseudo)[](#the-past-pseudo)

The <a id="past-pseudo"></a><strong>:past</strong> pseudo-class represents any element that is defined to occur entirely prior to a <a id="ref-for-current-pseudo⑦"></a>[:current](#current-pseudo) element. For example, the WebVTT spec defines the <a id="ref-for-past-pseudo④"></a>[:past](#past-pseudo) pseudo-class [relative to the current playback position of a media element](https://w3c.github.io/webvtt/#the-past-and-future-pseudo-classes). If a time-based order of elements is not defined by the document language, then this represents any element that is a (possibly indirect) previous sibling of a <a id="ref-for-current-pseudo⑧"></a>:current element.

### <a id="the-future-pseudo"></a>6.3.  The Future-element Pseudo-class: <a id="ref-for-future-pseudo③"></a>[:future](#future-pseudo)[](#the-future-pseudo)

The <a id="future-pseudo"></a><strong>:future</strong> pseudo-class represents any element that is defined to occur entirely after a <a id="ref-for-current-pseudo⑨"></a>[:current](#current-pseudo) element. For example, the WebVTT spec defines the <a id="ref-for-future-pseudo④"></a>[:future](#future-pseudo) pseudo-class [relative to the current playback position of a media element](https://w3c.github.io/webvtt/#the-past-and-future-pseudo-classes). If a time-based order of elements is not defined by the document language, then this represents any element that is a (possibly indirect) next sibling of a <a id="ref-for-current-pseudo①⓪"></a>:current element.

## <a id="custom-state"></a>7.  Exposing custom state: the <a id="ref-for-state-pseudo"></a>[:state()](#state-pseudo) pseudo-class[](#custom-state)

The <a id="state-pseudo"></a><strong>:state( <a id="ref-for-typedef-ident"></a>[\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident) )</strong> pseudo-class matches <a id="ref-for-custom-element"></a>[custom elements](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element) whose <a id="ref-for-states-set"></a>[states set](https://html.spec.whatwg.org/multipage/custom-elements.html#states-set) contains a string that <a id="ref-for-string-is"></a>[is](https://infra.spec.whatwg.org/#string-is) the selector’s argument’s value.

Tests

- [parse-state.html](https://wpt.fyi/results/css/selectors/parsing/parse-state.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-state.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-state.html)

Note: The "<a id="ref-for-string-is①"></a>[is](https://infra.spec.whatwg.org/#string-is)" matching behavior compares strings by codepoint; notably, it’s case-sensitive. So if "foo" is in the <a id="ref-for-states-set①"></a>[states set](https://html.spec.whatwg.org/multipage/custom-elements.html#states-set), :state(FOO) will <em>not</em> match.

The exact matching behavior of <a id="ref-for-state-pseudo①"></a>[:state()](#state-pseudo) pseudo-class is defined by the host language. For clarity, the concepts explaining this pseudo-class link to the HTML definition; see [HTML’s definition](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-state-pseudo-class) for more detail. Other host languages must define how this pseudo-class matches.

## <a id="headings"></a>8.  Heading Structures: the heading pseudo-classes <a id="ref-for-heading-pseudo"></a>[:heading](#heading-pseudo), and <a id="ref-for-heading-functional-pseudo"></a>[:heading()](#heading-functional-pseudo)[](#headings)

The (non-functional) <a id="heading-pseudo"></a><strong>:heading</strong> pseudo-class matches an element which has a <a id="ref-for-heading-level"></a>[heading level](https://html.spec.whatwg.org/multipage/sections.html#heading-level), with respect to the semantics defined by the document language (e.g. [\[HTML5\]](#biblio-html5)).

The <a id="ref-for-specificity"></a>[specificity](https://drafts.csswg.org/selectors-4/#specificity) of <a id="ref-for-heading-pseudo①"></a>[:heading](#heading-pseudo) is that of a class.

<a id="example-640da181"></a>

<strong>Example:</strong>

[](#example-640da181) For example, the following sheet contains a rule applying to all heading elements in the current page:

``` text
:heading { text-decoration: underline; }
```

As a functional pseudo-class, <a id="heading-functional-pseudo"></a><strong>:heading()</strong> notation represents elements that have a <a id="ref-for-heading-level①"></a>[heading level](https://html.spec.whatwg.org/multipage/sections.html#heading-level) among matching any of the provided <var>integer</var> values. The syntax is:

<a id="ref-for-typedef-level"></a><a id="ref-for-mult-comma"></a>

``` text
:heading() = :heading( <level># )
```

where <a id="typedef-level"></a><strong><code>&#60;level&#62;</code></strong> is a <a id="ref-for-typedef-number-token"></a>[\<number-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-number-token) with its type flag set to "integer".

The <a id="ref-for-specificity①"></a>[specificity](https://drafts.csswg.org/selectors-4/#specificity) of <a id="ref-for-heading-functional-pseudo①"></a>[:heading()](#heading-functional-pseudo) is that of a class.

<a id="example-8a5e281d"></a>

<strong>Example:</strong>

[](#example-8a5e281d) The following example styles headings with levels between 1 and 3 with a font-weight of 900, while headings with levels 6 onward with font-weight of 500, additionally heading levels 1 and 2 will be underlined, while 3 and beyond will have no text-decoration:

``` text
:heading(1, 2, 3) { font-weight: 900; }
```

``` text
:heading(6, 7, 8, 9) { font-weight: 500; }
```

``` text
:heading(1, 2) { text-decoration: underline; }
```

``` text
:heading(3, 4, 5, 6, 7, 8, 9) { text-decoration: none; }
```

Note: The <a id="ref-for-heading-level②"></a>[heading level](https://html.spec.whatwg.org/multipage/sections.html#heading-level) might be different from an element’s <a id="ref-for-type-selector"></a>[type selector](https://drafts.csswg.org/selectors-4/#type-selector). Thus, a selector h1:heading(3) matches any h1 tag which has an exposed heading level of 3.

Tests

- [heading.html](https://wpt.fyi/results/css/selectors/heading.html) [(live test)](http://wpt.live/css/selectors/heading.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/heading.html)
- [parse-heading.html](https://wpt.fyi/results/css/selectors/parsing/parse-heading.html) [(live test)](http://wpt.live/css/selectors/parsing/parse-heading.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/selectors/parsing/parse-heading.html)

## <a id="grid-structural-selectors"></a>9. <a id="table-pseudos"></a> Grid-Structural Selectors[](#grid-structural-selectors)

The double-association of a cell in a 2D grid (to its row and column) cannot be represented by parentage in a hierarchical markup language. Only one of those associations can be represented hierarchically: the other must be explicitly or implicitly defined in the document language semantics. In both HTML and DocBook, two of the most common hierarchical markup languages, the markup is row-primary (that is, the row associations are represented hierarchically); the columns must be implied. To be able to represent such implied column-based relationships, the <a id="ref-for-column-combinator"></a>[column combinator](#column-combinator) and the <a id="ref-for-nth-col-pseudo"></a>[:nth-col()](#nth-col-pseudo) and <a id="ref-for-nth-last-col-pseudo"></a>[:nth-last-col()](#nth-last-col-pseudo) pseudo-classes are defined. In a column-primary format, these pseudo-classes match against row associations instead.

### <a id="the-column-combinator"></a>9.1.  Column combinator (<code>&#124;&#124;</code>)[](#the-column-combinator)

The <a id="column-combinator"></a><strong>column combinator</strong>, which consists of two pipes (<a id="selectordef-column"></a><strong>\|\|</strong>) represents the relationship of a column element to a cell element belonging to the column it represents. Column membership is determined based on the semantics of the document language only: whether and how the elements are presented is not considered. If a cell element belongs to more than one column, it is represented by a selector indicating membership in any of those columns.

<a id="example-60f47339"></a>

<strong>Example:</strong>

[](#example-60f47339) The following example makes cells C, E, and G gray.

``` text
col.selected || td {
  background: gray;
  color: white;
  font-weight: bold;
}
```

``` text
<table>
  <col span="2">
  <col class="selected">
  <tr><td>A <td>B <td>C
  <tr><td colspan="2">D <td>E
  <tr><td>F <td colspan="2">G
</table>
```

### <a id="the-nth-col-pseudo"></a>9.2.  <a id="ref-for-nth-col-pseudo①"></a>[:nth-col()](#nth-col-pseudo) pseudo-class[](#the-nth-col-pseudo)

The <a id="nth-col-pseudo"></a><strong>:nth-col(<var>An+B</var>)</strong> pseudo-class notation represents a cell element belonging to a column that has <var>An+B</var>-1 columns <strong>before</strong> it, for any positive integer or zero value of <code>n</code>. Column membership is determined based on the semantics of the document language only: whether and how the elements are presented is not considered. If a cell element belongs to more than one column, it is represented by a selector indicating any of those columns.

The CSS Syntax Module [\[CSS3SYN\]](#biblio-css3syn) defines the [<var>An+B</var> notation](https://drafts.csswg.org/css-syntax/#anb).

### <a id="the-nth-last-col-pseudo"></a>9.3.  <a id="ref-for-nth-last-col-pseudo①"></a>[:nth-last-col()](#nth-last-col-pseudo) pseudo-class[](#the-nth-last-col-pseudo)

The <a id="nth-last-col-pseudo"></a><strong>:nth-last-col(<var>An+B</var>)</strong> pseudo-class notation represents a cell element belonging to a column that has <var>An+B</var>-1 columns <strong>after</strong> it, for any positive integer or zero value of <code>n</code>. Column membership is determined based on the semantics of the document language only: whether and how the elements are presented is not considered. If a cell element belongs to more than one column, it is represented by a selector indicating any of those columns.

The CSS Syntax Module [\[CSS3SYN\]](#biblio-css3syn) defines the [<var>An+B</var> notation](https://drafts.csswg.org/css-syntax/#anb).

## <a id="combinators"></a>10.  Combinators[](#combinators)

### <a id="idref-combinators"></a>10.1.  Reference combinators <code>/ref/</code>[](#idref-combinators)

The <a id="reference-combinator"></a><strong>reference combinator</strong> consists of two slashes with an intervening [CSS qualified name](http://www.w3.org/TR/css3-namespace/#css-qnames), and separates two <a id="ref-for-compound①"></a>[compound selectors](https://drafts.csswg.org/selectors-4/#compound), e.g. A /attr/ B. The element represented by the first <a id="ref-for-compound②"></a>compound selector explicitly references the element represented by the second <a id="ref-for-compound③"></a>compound selector. Unless the host language defines a different syntax for expressing this relationship, this relationship is considered to exist if the value of the specified attribute on the first element is an IDREF or an <a id="ref-for-id-selector"></a>[ID selector](https://drafts.csswg.org/selectors-4/#id-selector) referencing the second element.

Attribute matching for reference combinators follow the same rules as for [attribute selectors](http://www.w3.org/TR/css3-selectors/#attribute-selectors).

<a id="example-69e878ec"></a>

<strong>Example:</strong>

[](#example-69e878ec) The following example highlights an <a id="ref-for-the-input-element"></a>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element) element when its [<code>&#60;label&#62;</code>](http://www.w3.org/TR/html40/interact/forms.html#h-17.9) is focused or hovered-over:

``` text
label:is(:hover, :focus) /for/ input,       /* association by "for" attribute */
label:is(:hover, :focus):not([for]) input { /* association by containment */
  box-shadow: yellow 0 0 10px;
}
```

## <a id="grammar"></a>11. <a id="formal-syntax"></a> Grammar[](#grammar)

This module extends the CSS grammar of selectors with:

<a id="typedef-combinator"></a><a id="ref-for-comb-one"></a><a id="ref-for-comb-one①"></a><a id="ref-for-comb-one②"></a><a id="ref-for-comb-one③"></a>

``` text
<combinator> = '>' | '+' | '~' | [ '|' '|' ] | [ / <wq-name> / ]
```

- <a id="white-space"></a> [](#white-space) White space is forbidden:
  - Between the components of a <a id="ref-for-typedef-combinator"></a>[\<combinator\>](#typedef-combinator).

## <a id="changes"></a>12.  Changes[](#changes)

### <a id="changes-20260415"></a>12.1.  Changes Since the [First Public Working Draft](https://www.w3.org/TR/2026/WD-selectors-5-20260217/) of 17 February 2026[](#changes-20260415)

- Moved the column combinator to Selectors 5 [PR 13424](https://github.com/w3c/csswg-drafts/pull/13424)

### <a id="changes-level-4"></a>12.2.  Changes Since Level 4[](#changes-level-4)

Additions since [Level 4](https://www.w3.org/TR/selectors-4/):

- Reference combinators (deferred from an [earlier draft](https://www.w3.org/TR/2013/WD-selectors4-20130502/) of Selectors 4)
- The <a id="ref-for-local-link-pseudo②"></a>[:local-link](#local-link-pseudo) pseudo-class (deferred from an [earlier draft](https://www.w3.org/TR/2022/WD-selectors-4-20221111/) of Selectors 4) and its functional form (deferred from an [earlier draft](https://www.w3.org/TR/2013/WD-selectors4-20130502/) of Selectors 4)
- The <a id="ref-for-state-pseudo②"></a>[:state()](#state-pseudo) pseudo-class
- The <a id="ref-for-heading-pseudo②"></a>[:heading](#heading-pseudo) and <a id="ref-for-heading-functional-pseudo②"></a>[:heading()](#heading-functional-pseudo) pseudo-classes
- The <a id="ref-for-selectordef-interest-source③"></a>[:interest-source](#selectordef-interest-source) and <a id="ref-for-selectordef-interest-target②"></a>[:interest-target](#selectordef-interest-target) pseudo-classes
- The <a id="ref-for-blank-pseudo⑦"></a>[:blank](#blank-pseudo) pseudo-class (deferred from an [earlier draft](https://www.w3.org/TR/2022/WD-selectors-4-20221111/) of Selectors 4)
- The [grid-structural (column) selectors](#grid-structural-selectors) (deferred from an [earlier draft](https://www.w3.org/TR/2022/WD-selectors-4-20221111/) of Selectors 4)
- The [time-dimensional pseudo-classes](#time-pseudos) (deferred from an [earlier draft](https://www.w3.org/TR/2022/WD-selectors-4-20221111/) of Selectors 4)

## <a id="acknowledgements"></a>13.  Acknowledgements[](#acknowledgements)

The CSS working group would like to thank everyone who contributed to the [previous Selectors](https://www.w3.org/TR/selectors-4) specifications over the years, as those specifications formed the basis for this one. In particular, the working group would like to extend special thanks to the following for their specific contributions to Selectors Level 5: Joey Arhar.

## <a id="privacy"></a>Privacy Considerations[](#privacy)

Should be copied from Level 4 when appropriate.

## <a id="security"></a>Security Considerations[](#security)

Should be copied from Level 4 when appropriate.

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

- [\|\|](#selectordef-column), in § 9.1
- [:blank](#blank-pseudo), in § 5.1.1
- [class prefix selector](#class-prefix-selector), in § 2.1
- [column combinator](#column-combinator), in § 9.1
- [\<combinator\>](#typedef-combinator), in § 11
- [:current](#current-pseudo), in § 6.1
- [:current()](#selectordef-current), in § 6.1
- [:future](#future-pseudo), in § 6.3
- [:heading](#heading-pseudo), in § 8
- [:heading()](#heading-functional-pseudo), in § 8
- [:interest-source](#selectordef-interest-source), in § 4.1
- [interest source](#interest-source), in § 4.1
- [:interest-target](#selectordef-interest-target), in § 4.1
- [interest target](#interest-target), in § 4.1
- [\<level\>](#typedef-level), in § 8
- [:local-link](#local-link-pseudo), in § 3.1
- [:nth-col()](#nth-col-pseudo), in § 9.2
- [:nth-last-col()](#nth-last-col-pseudo), in § 9.3
- [:past](#past-pseudo), in § 6.2
- [reference combinator](#reference-combinator), in § 10.1
- [:state( \<ident\> )](#state-pseudo), in § 7

### <a id="index-defined-elsewhere"></a>Terms defined by reference[](#index-defined-elsewhere)

- \[CSS-VALUES-4\] defines the following terms:
  - <a id="68487d22"></a>\#
  - <a id="9213678e"></a>\<ident\>
  - <a id="6ec67710"></a>\|
- \[CSS3SYN\] defines the following terms:
  - <a id="92700be8"></a>\<number-token\>
- \[HTML\] defines the following terms:
  - <a id="b6fb7a51"></a>custom element
  - <a id="b87923a3"></a>heading level
  - <a id="d7d642a2"></a>input
  - <a id="f4da6587"></a>popover
  - <a id="141caf46"></a>states set
- \[INFRA\] defines the following terms:
  - <a id="7a87d819"></a>is
- \[SELECTORS-4\] defines the following terms:
  - <a id="223a826d"></a>class selector
  - <a id="24408a11"></a>compound selector
  - <a id="d4b111eb"></a>host language
  - <a id="679bc36f"></a>ID selector
  - <a id="33cbcfca"></a>selector
  - <a id="bb2056aa"></a>specificity
  - <a id="32c45571"></a>type selector
  - <a id="84540613"></a>universal selector

## <a id="references"></a>References[](#references)

### <a id="normative"></a>Normative References[](#normative)

<a id="biblio-css-values-4"></a><strong>\[CSS-VALUES-4\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: <https://drafts.csswg.org/css-values-4/>

<a id="biblio-css3syn"></a><strong>\[CSS3SYN\]</strong>

Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/). URL: <https://drafts.csswg.org/css-syntax/>

<a id="biblio-html"></a><strong>\[HTML\]</strong>

Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: <https://html.spec.whatwg.org/multipage/>

<a id="biblio-infra"></a><strong>\[INFRA\]</strong>

Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: <https://infra.spec.whatwg.org/>

<a id="biblio-rfc2119"></a><strong>\[RFC2119\]</strong>

S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: <https://datatracker.ietf.org/doc/html/rfc2119>

<a id="biblio-selectors-4"></a><strong>\[SELECTORS-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://drafts.csswg.org/selectors/). URL: <https://drafts.csswg.org/selectors/>

### <a id="informative"></a>Non-Normative References[](#informative)

<a id="biblio-html5"></a><strong>\[HTML5\]</strong>

Ian Hickson; et al. [HTML5](https://www.w3.org/TR/html5/). 27 March 2018. REC. URL: <https://www.w3.org/TR/html5/>

## <a id="issues-index"></a>Issues Index[](#issues-index)

<strong>Issue:</strong>

<strong>This is a diff spec against [Selectors Level 4](https://www.w3.org/TR/selectors-4/).</strong> [↵](#issue-edc8dd61)

It’s clear that, if the document URL has at least N segments, then

<strong>Issue:</strong>

:local-link(N) only matches links whose URL has at least N segments. (This lets you assign consistent semantics to :local-link so that, for example, :local-link(2) means a "within-repo" link on GitHub.) What about if the document url has less than N segments, and the link is same-page? Should "null segments" count as matching, or not? [↵](#issue-dd3371ba)

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
