Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Shadow Module Level 1](https://drafts.csswg.org/css-shadow-1/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply. The capture’s full original notice and links remain below.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md).

# Source provenance

Title: CSS Shadow Module Level 1

Source snapshot: https://drafts.csswg.org/css-shadow-1/

Retrieved: 2026-10-10. The captured page states Editor’s Draft, 28 April 2026. An undated editor URL can change; the retrieval date and exact byte hash identify this captured HTML.

Captured HTML SHA-256: b02c47a6b167d19ad65ae25eefe2a446475cc4fb5d60d3560e5996ac3be28e51

Captured HTML revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Immutable upstream source: [Overview.bs](https://raw.githubusercontent.com/w3c/csswg-drafts/50337328484f7fcdd9ab6fe9ded6011815b1707e/css-shadow-1/Overview.bs), SHA-256 `0879121699895ad7670985665e6ee70bccd96c2a42e26cf99ca7fd56c44816e7`. The source revision is inspected as reference material and is not executed.

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

# <a id="title"></a>CSS Shadow Module Level 1

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 28 April 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-shadow-1/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2014/WD-css-scoping-1-20140403/>

<strong>Issue Tracking:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-shadow-1)

[Inline In Spec](#issues-index)

<strong>Editors:</strong>

[Tab Atkins Jr.](http://xanthir.com/contact/) (Google)

[Fergal Daly](mailto:fergal@chromium.org) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-shadow-1/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

This module defines ways for CSS to interact with Shadow DOM and its scoping mechanism.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-shadow” in the title, like this: “\[css-shadow\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-shadow%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](#intro)
2.  [2 Default Styles for Custom Elements](#default-element-styles)
3.  [3 Shadow Encapsulation](#shadow-dom)
    1.  [3.1 Informative Explanation of Shadow DOM](#shadow-gloss)
    2.  [3.2 Shadow DOM and Selectors](#selectors)
        1.  [3.2.1 Matching Selectors Against Shadow Trees](#selectors-data-model)
        2.  [3.2.2 Selecting Shadow Hosts from within a Shadow Tree](#host-element-in-tree)
        3.  [3.2.3 Selecting Into the Light: the :host, :host(), and :host-context() pseudo-classes](#host-selector)
        4.  [3.2.4 Selecting Slot-Assigned Content: the ::slotted() pseudo-element](#slotted-pseudo)
        5.  [3.2.5 Matching on the Presence of Slot-Assigned Nodes: the :has-slotted pseudo-class](#the-has-slotted-pseudo)
4.  [4 Shadow Trees and the Cascade](#shadow-cascading)
    1.  [4.1 Flattening the DOM into an Element Tree](#flattening)
        1.  [4.1.1 Slots and Slotted Elements in a Shadow Tree](#slots-in-shadow-tree)
    2.  [4.2 Name-Defining Constructs and Inheritance](#shadow-names)
        1.  [4.2.1 Serialized Tree-Scoped References](#shadow-names-serialization)
5.  [5 Exposing a Shadow Element](#exposing)
    1.  [5.1 Motivation](#motivation)
    2.  [5.2 Naming a Shadow Element: the <code>part</code> attribute](#part-attr)
    3.  [5.3 Forwarding a Shadow Element: the <code>exportparts</code> attribute](#exportparts-attr)
    4.  [5.4 Selecting a Shadow Element: the ::part() pseudo-element](#part)
    5.  [5.5 Extensions to the <code>Element</code> Interface](#idl)
    6.  [5.6 Microsyntaxes for parsing](#parsing)
        1.  [5.6.1 Rules for parsing part mappings](#parsing-mapping)
        2.  [5.6.2 Rules for parsing a list of part mappings](#parsing-mapping-list)
6.  [6 Changes](#changes)
7.  [7 Privacy Considerations](#privacy)
8.  [8 Security Considerations](#security)
9.  [ Conformance](#w3c-conformance)
    1.  [ Document conventions](#w3c-conventions)
    2.  [ Conformance classes](#w3c-conformance-classes)
    3.  [ Partial implementations](#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](#w3c-testing)
10. [ Index](#index)
    1.  [ Terms defined by this specification](#index-defined-here)
    2.  [ Terms defined by reference](#index-defined-elsewhere)
11. [ References](#references)
    1.  [ Normative References](#normative)
    2.  [ Non-Normative References](#informative)
12. [ IDL Index](#idl-index)
13. [ Issues Index](#issues-index)

## <a id="intro"></a>1. Introduction[](#intro)

Shadow DOM allows authors to separate their page into "components", subtrees of markup whose details are only relevant to the component itself, not the outside page. This reduces the chance of a style meant for one part of the page accidentally over-applying and making a different part of the page look wrong. However, this styling barrier also makes it harder for a page to interact with its components when it actually <em>wants</em> to do so.

This specification defines the <a id="ref-for-selectordef-part"></a>[::part()](#selectordef-part) pseudo-element, which allows an author to style specific, purposely exposed elements in a <a id="ref-for-concept-shadow-tree"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) from the outside page’s context. In combination with <a id="ref-for-custom-property"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property), which let the outside page pass particular values (such as theme colors) into the component for it to do with as it will, these pseudo-elements allow components and the outside page to interact in safe, powerful ways, maintaining encapsulation without surrendering all control.

The <a id="ref-for-selectordef-host"></a>[:host](#selectordef-host) pseudo-class and its <a id="ref-for-functional-pseudo-class"></a>[functional counterpart](https://drafts.csswg.org/selectors-4/#functional-pseudo-class) <a id="ref-for-selectordef-host-function"></a>[:host()](#selectordef-host-function) match the <a id="ref-for-concept-shadow-tree①"></a>[shadow tree’s](https://dom.spec.whatwg.org/#concept-shadow-tree) <a id="ref-for-element-shadow-host"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host).

Furthermore, the ::slotted <a id="ref-for-pseudo-element"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) and the related <a id="ref-for-selectordef-has-slotted"></a>[:has-slotted](#selectordef-has-slotted) <a id="ref-for-pseudo-class"></a>[pseudo-class](https://drafts.csswg.org/selectors-4/#pseudo-class) provide ways to interact with <a id="ref-for-concept-slot"></a>[slots](https://dom.spec.whatwg.org/#concept-slot) and their assigned <a id="ref-for-concept-shadow-tree②"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree).

Tests

General tests for shadow parts

------------------------------------------------------------------------

Note: This specification was originally titled CSS Scoping Module Level 1. It has since been merged with CSS Shadow Parts Module Level 1 and renamed to CSS Shadow Module Level 1 to better reflect its comprehensive coverage of Shadow DOM features. The <a id="ref-for-scoped-style-rules"></a>[scoped style rules](https://drafts.csswg.org/css-cascade-6/#scoped-style-rules) feature initially described in this module has been moved to [\[css-cascade-6\]](#biblio-css-cascade-6).

## <a id="default-element-styles"></a>2. Default Styles for Custom Elements[](#default-element-styles)

<strong>This section is</strong>

<strong>Advisement:</strong>

<strong>experimental</strong>, and is under active discussion. Do not implement without consulting the CSSWG.

When defining <a id="ref-for-custom-element"></a>[custom elements](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element), one often wants to set up "default" styles for them, akin to the user-agent styles that apply to built-in elements. This is, unfortunately, hard to do in vanilla CSS, due to issues of scoping and specificity—​the element in question might be used in <a id="ref-for-concept-shadow-tree③"></a>[shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree), and thus is unreachable by any selector targeting it in the outermost document; and selectors, even low-specificity ones like simple <a id="ref-for-type-selector"></a>[type selectors](https://drafts.csswg.org/selectors-4/#type-selector), can accidentally override author-level styles meant to target the element.

To aid in this, this section defines a way to create a stylesheet of "default element styles" for a given element. This stylesheet applies across the entire document, in all <a id="ref-for-concept-shadow-tree④"></a>[shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree), and the rules in it apply at the <a id="ref-for-cascade-origin-ua"></a>[user-agent origin](https://drafts.csswg.org/css-cascade-5/#cascade-origin-ua), so author-level rules automatically win.

<code><a id="ref-for-window"></a>[Window](https://html&#46;spec&#46;whatwg&#46;org/multipage/nav-history-apis&#46;html#window)</code>s gain a private slot <a id="dom-window-defaultelementstylesmap-slot"></a><strong><code>&#91;&#91;defaultElementStylesMap&#93;&#93;</code></strong> which is a map of <a id="ref-for-concept-element-local-name"></a>[local names](https://dom.spec.whatwg.org/#concept-element-local-name) to <a id="ref-for-css-stylesheet"></a>[stylesheets](https://drafts.csswg.org/css-syntax-3/#css-stylesheet).

These stylesheets must apply to every document in the window. They must be interpreted as user agent stylesheets.

Note: This implies, in particular, that they apply to all <a id="ref-for-concept-shadow-tree⑤"></a>[shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) in every document, and that the declarations in them are from the <a id="ref-for-cascade-origin-ua①"></a>[user-agent origin](https://drafts.csswg.org/css-cascade-5/#cascade-origin-ua).

For the purpose of the <a id="ref-for-cascade"></a>[cascade](https://drafts.csswg.org/css-cascade-6/#cascade), these stylesheets are ordered after the user agent’s own stylesheets; their relative ordering doesn’t matter as it is not observable.

Within these stylesheets, <a id="ref-for-complex"></a>[complex selectors](https://drafts.csswg.org/selectors-4/#complex) must be treated as invalid. Every <a id="ref-for-compound"></a>[compound selector](https://drafts.csswg.org/selectors-4/#compound) must be treated as containing an additional <a id="ref-for-type-selector①"></a>[type selector](https://drafts.csswg.org/selectors-4/#type-selector) that selects elements with the <a id="ref-for-concept-element-local-name①"></a>[local name](https://dom.spec.whatwg.org/#concept-element-local-name) that the stylesheet is keyed with.

<a id="issue-f4e1dc6b"></a>

<strong>Issue:</strong>

[](#issue-f4e1dc6b) Do we need to restrict the <a id="ref-for-at-rule"></a>[at-rules](https://drafts.csswg.org/css-syntax-3/#at-rule) that can be used in these sheets? For example, do we allow an <a id="ref-for-at-font-face-rule"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule)? I’m going to leave it as allowed unless/until I hear complaints.

This specification does not define how to add to, remove from, or generally manipulate the <code><a id="ref-for-dom-window-defaultelementstylesmap-slot"></a>[\[\[defaultElementStylesMap\]\]](#dom-window-defaultelementstylesmap-slot)</code>. It is expected that other specifications, such as [\[DOM\]](#biblio-dom), will define ways to do so.

## <a id="shadow-dom"></a>3. Shadow Encapsulation[](#shadow-dom)

### <a id="shadow-gloss"></a>3.1. Informative Explanation of Shadow DOM[](#shadow-gloss)

<em>The following is a non-normative explanation of several concepts normatively defined in the DOM Standard [\[DOM\]](#biblio-dom), to aid in understanding what this spec defines without having to fully grok the DOM Standard.</em>

In addition to the qualities of an element tree defined in [Selectors Level 4 § data-model](https://drafts.csswg.org/selectors/#data-model), the DOM Standard adds several new concepts related to <a id="ref-for-concept-shadow-tree⑥"></a>[shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree), several of which are relevant to CSS.

An element can host a <a id="ref-for-concept-shadow-tree⑦"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), which is a special kind of document fragment with a <a id="ref-for-concept-shadow-root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) (a non-element node) at its root. Children of the <a id="ref-for-concept-shadow-root①"></a>shadow root are ordinary elements and other nodes. The element hosting the <a id="ref-for-concept-shadow-tree⑧"></a>shadow tree is its <a id="ref-for-concept-documentfragment-host"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host), or <a id="ref-for-element-shadow-host①"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host).

The elements in a <a id="ref-for-concept-shadow-tree⑨"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) are not <a id="ref-for-concept-tree-descendant"></a>[descendants](https://dom.spec.whatwg.org/#concept-tree-descendant) of the <a id="ref-for-element-shadow-host②"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) in general (including for the purposes of Selectors like the <a id="ref-for-descendant-combinator"></a>[descendant combinator](https://drafts.csswg.org/selectors-4/#descendant-combinator)). However, the <a id="ref-for-concept-shadow-tree①⓪"></a>shadow tree, when it exists, is used in the construction of the <a id="ref-for-flat-tree"></a>[flattened element tree](#flat-tree), which CSS uses for all purposes <em>after</em> Selectors (including inheritance and box construction).

Loosely, the <a id="ref-for-concept-shadow-tree①①"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) is treated as the <a id="ref-for-element-shadow-host③"></a>[shadow host’s](https://dom.spec.whatwg.org/#element-shadow-host) contents instead of its normal <a id="ref-for-concept-light-tree"></a>[light tree](https://dom.spec.whatwg.org/#concept-light-tree) contents. However, some of its <a id="ref-for-concept-light-tree①"></a>light tree children can be "pulled into" the <a id="ref-for-concept-shadow-tree①②"></a>shadow tree by assigning them to <a id="ref-for-concept-slot①"></a>[slots](https://dom.spec.whatwg.org/#concept-slot). This causes them to be treated as children of the <a id="ref-for-concept-slot②"></a>slot for CSS purposes. The <a id="ref-for-concept-slot③"></a>slots can then be assigned to <a id="ref-for-concept-slot④"></a>slots in deeper <a id="ref-for-concept-shadow-tree①③"></a>shadow trees; luckily, <a id="ref-for-concept-slot⑤"></a>slots themselves don’t generate boxes by default, so you don’t get an unpredictable cascade of <code><a id="ref-for-the-slot-element"></a>[slot](https://html&#46;spec&#46;whatwg&#46;org/multipage/scripting&#46;html#the-slot-element)</code> wrapper elements disrupting your CSS.

If nothing is explicitly assigned to a <a id="ref-for-concept-slot⑥"></a>[slot](https://dom.spec.whatwg.org/#concept-slot), the <a id="ref-for-concept-slot⑦"></a>slot’s own children are instead assigned to it, as a sort of "default" contents.

### <a id="selectors"></a>3.2. Shadow DOM and Selectors[](#selectors)

#### <a id="selectors-data-model"></a>3.2.1.  Matching Selectors Against Shadow Trees[](#selectors-data-model)

When a selector is matched against a <a id="ref-for-concept-shadow-tree①④"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), the selector match list is initially the <a id="ref-for-element-shadow-host④"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host), followed by all children of the <a id="ref-for-concept-shadow-tree①⑤"></a>shadow tree’s <a id="ref-for-concept-shadow-root②"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) and their descendants, ordered by a pre-order traversal.

<a id="issue-b8d42d77"></a>

<strong>Issue:</strong>

[](#issue-b8d42d77) Rewrite this against the newer call forms of the matching algorithms.

Note: Remember that the <a id="ref-for-concept-tree-descendant①"></a>[descendants](https://dom.spec.whatwg.org/#concept-tree-descendant) of an element are based on the <a id="ref-for-concept-light-tree②"></a>[light tree](https://dom.spec.whatwg.org/#concept-light-tree) <a id="ref-for-concept-tree-child"></a>[children](https://dom.spec.whatwg.org/#concept-tree-child) of the element, which does not include the <a id="ref-for-concept-shadow-tree①⑥"></a>[shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) of the element.

When a selector is <a id="ref-for-match-a-selector-against-a-tree"></a>[matched against a tree](https://drafts.csswg.org/selectors-4/#match-a-selector-against-a-tree), its <a id="tree-context"></a><strong>tree context</strong> is the <a id="ref-for-concept-tree-root"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) of the <var>root elements</var> passed to the algorithm. If the <a id="ref-for-tree-context"></a>[tree context](#tree-context) is a <a id="ref-for-concept-shadow-root③"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root), that selector is being matched <a id="in-the-context-of-a-shadow-tree"></a><strong>in the context of a shadow tree</strong>.

<a id="example-4f60723d"></a>

<strong>Example:</strong>

[](#example-4f60723d) For example, any selector in a stylesheet embedded in or linked from an element in a <a id="ref-for-concept-shadow-tree①⑦"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) is <a id="ref-for-in-the-context-of-a-shadow-tree"></a>[in the context of a shadow tree](#in-the-context-of-a-shadow-tree). So is the argument to <code><a id="ref-for-dom-parentnode-queryselector"></a>[querySelector()](https://dom&#46;spec&#46;whatwg&#46;org/#dom-parentnode-queryselector)</code> when called from a <a id="ref-for-concept-shadow-root④"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root).

Declarations inherit the <a id="ref-for-tree-context①"></a>[tree context](#tree-context) of the selector that was matched to apply them.

Tests

#### <a id="host-element-in-tree"></a>3.2.2.  Selecting Shadow Hosts from within a Shadow Tree[](#host-element-in-tree)

A <a id="ref-for-element-shadow-host⑤"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) is outside of the <a id="ref-for-concept-shadow-tree①⑧"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) it hosts, and so would ordinarily be untargettable by any selectors evaluated <a id="ref-for-in-the-context-of-a-shadow-tree①"></a>[in the context of the shadow tree](#in-the-context-of-a-shadow-tree) (as selectors are limited to a single tree), but it is sometimes useful to be able to style it from inside the <a id="ref-for-concept-shadow-tree①⑨"></a>shadow tree context.

For the purpose of Selectors, a <a id="ref-for-element-shadow-host⑥"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) also appears in its <a id="ref-for-concept-shadow-tree②⓪"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), with the contents of the <a id="ref-for-concept-shadow-tree②①"></a>shadow tree treated as its children. (In other words, the <a id="ref-for-element-shadow-host⑦"></a>shadow host is treated as replacing the <a id="ref-for-concept-shadow-root⑤"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) node.)

When considered within its own <a id="ref-for-concept-shadow-tree②②"></a>[shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree), the <a id="ref-for-element-shadow-host⑧"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) is <a id="ref-for-featureless"></a>[featureless](https://drafts.csswg.org/selectors-4/#featureless). Only the <a id="ref-for-selectordef-host①"></a>[:host](#selectordef-host), <a id="ref-for-selectordef-host-function①"></a>[:host()](#selectordef-host-function), and <a id="ref-for-selectordef-host-context"></a>[:host-context()](#selectordef-host-context) pseudo-classes are allowed to match it.

Tests

<strong>Note:</strong>

Why is the shadow host so weird?

The <a id="ref-for-element-shadow-host⑨"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) lives outside the <a id="ref-for-concept-shadow-tree②③"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), and its markup is in control of the page author, not the component author.

It would not be very good if a component used a particular class name internally in a <a id="ref-for-concept-shadow-tree②④"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) stylesheet, and the page author using the component accidentally <em>also</em> used the same class name and put it on the <a id="ref-for-element-shadow-host①⓪"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host). Such a situation would result in accidental styling that is impossible for the component author to predict, and confusing for the page author to debug.

However, there are still some reasonable use-cases for letting a stylesheet in a <a id="ref-for-concept-shadow-tree②⑤"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) style its <a id="ref-for-element-shadow-host①①"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host). (For example, the component might want to be laid out as a flexbox, requiring the <a id="ref-for-element-shadow-host①②"></a>shadow host to be set to <a id="ref-for-propdef-display"></a>[display: flex](https://drafts.csswg.org/css-display-4/#propdef-display).) So, to allow this situation but prevent accidental styling, the <a id="ref-for-element-shadow-host①③"></a>shadow host appears but is completely featureless and unselectable except through <a id="ref-for-selectordef-host②"></a>[:host](#selectordef-host) and its related functional forms, which make it very explicit when you’re trying to match against markup provided by the page author.

Tests

#### <a id="host-selector"></a>3.2.3.  Selecting Into the Light: the <a id="ref-for-selectordef-host③"></a>[:host](#selectordef-host), <a id="ref-for-selectordef-host-function②"></a>[:host()](#selectordef-host-function), and <a id="ref-for-selectordef-host-context①"></a>[:host-context()](#selectordef-host-context) pseudo-classes[](#host-selector)

The <a id="selectordef-host"></a><strong>:host</strong> pseudo-class, when evaluated <a id="ref-for-in-the-context-of-a-shadow-tree②"></a>[in the context of a shadow tree](#in-the-context-of-a-shadow-tree), matches the <a id="ref-for-concept-shadow-tree②⑥"></a>[shadow tree’s](https://dom.spec.whatwg.org/#concept-shadow-tree) <a id="ref-for-element-shadow-host①④"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host). In any other context, it matches nothing.

The <a id="selectordef-host-function"></a><strong>:host()</strong> function pseudo-class has the syntax:

<a id="ref-for-typedef-compound-selector"></a>

``` text
:host( <compound-selector> )
```

When evaluated <a id="ref-for-in-the-context-of-a-shadow-tree③"></a>[in the context of a shadow tree](#in-the-context-of-a-shadow-tree), it matches the <a id="ref-for-concept-shadow-tree②⑦"></a>[shadow tree’s](https://dom.spec.whatwg.org/#concept-shadow-tree) <a id="ref-for-element-shadow-host①⑤"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) if the <a id="ref-for-element-shadow-host①⑥"></a>shadow host, in its normal context, matches the selector argument. In any other context, it matches nothing.

The <a id="ref-for-specificity"></a>[specificity](https://drafts.csswg.org/selectors-4/#specificity) of <a id="ref-for-selectordef-host④"></a>[:host](#selectordef-host) is that of a pseudo-class. The <a id="ref-for-specificity①"></a>specificity of <a id="ref-for-selectordef-host-function③"></a>[:host()](#selectordef-host-function) is that of a pseudo-class, plus the <a id="ref-for-specificity②"></a>specificity of its argument.

Note: This is different from the specificity of similar pseudo-classes, like <a id="ref-for-matches-pseudo"></a>[:is()](https://drafts.csswg.org/selectors-4/#matches-pseudo) or <a id="ref-for-negation-pseudo"></a>[:not()](https://drafts.csswg.org/selectors-4/#negation-pseudo), which <em>only</em> take the specificity of their argument. This is because <a id="ref-for-selectordef-host⑤"></a>[:host](#selectordef-host) is affirmatively selecting an element all by itself, like a "normal" pseudo-class; it takes a selector argument for syntactic reasons (we can’t say that :host.foo matches but .foo doesn’t), but is otherwise identical to just using <a id="ref-for-selectordef-host⑥"></a>:host followed by a selector.

<a id="example-20a3843d"></a>

<strong>Example:</strong>

[](#example-20a3843d) For example, say you had a component with a <a id="ref-for-concept-shadow-tree②⑧"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) like the following:

``` text
<x-foo class="foo">
  <"shadow tree">
    <div class="foo">...</div>
  </>
</x-foo>
```

For a stylesheet within the <a id="ref-for-concept-shadow-tree②⑨"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree):

- <a id="ref-for-selectordef-host⑦"></a>[:host](#selectordef-host) matches the <code>&#60;x-foo&#62;</code> element.

- x-foo matches nothing.

- .foo matches only the <code>&#60;div&#62;</code> element.

- .foo:host matches nothing

- :host(.foo) matches the <code>&#60;x-foo&#62;</code> element.

Ordinary, selectors within a <a id="ref-for-concept-shadow-tree③⓪"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) can’t see elements outside the <a id="ref-for-concept-shadow-tree③①"></a>shadow tree at all. Sometimes, however, it’s useful to select an ancestor that lies somewhere outside the shadow tree, above it in the document.

<a id="example-45cde0ef"></a>

<strong>Example:</strong>

[](#example-45cde0ef) For example, a group of components can define a handful of color themes they know how to respond to. Page authors could opt into a particular theme by adding a specific class to the components, or higher up in the document.

The <a id="selectordef-host-context"></a><strong>:host-context()</strong> functional pseudo-class tests whether there is an ancestor, outside the <a id="ref-for-concept-shadow-tree③②"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), which matches a particular selector. Its syntax is:

<a id="ref-for-typedef-compound-selector①"></a>

``` text
:host-context( <compound-selector> )
```

When evaluated <a id="ref-for-in-the-context-of-a-shadow-tree④"></a>[in the context of a shadow tree](#in-the-context-of-a-shadow-tree), the <a id="ref-for-selectordef-host-context②"></a>[:host-context()](#selectordef-host-context) pseudo-class matches the <a id="ref-for-element-shadow-host①⑦"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host), if the <a id="ref-for-element-shadow-host①⑧"></a>shadow host or one of its <a id="ref-for-concept-shadow-including-ancestor"></a>[shadow-including ancestors](https://dom.spec.whatwg.org/#concept-shadow-including-ancestor) matches the provided <a id="ref-for-typedef-compound-selector②"></a>[\<compound-selector\>](https://drafts.csswg.org/selectors-4/#typedef-compound-selector). In any other context, it matches nothing.

The <a id="ref-for-specificity③"></a>[specificity](https://drafts.csswg.org/selectors-4/#specificity) of <a id="ref-for-selectordef-host-context③"></a>[:host-context()](#selectordef-host-context) is that of a pseudo-class, plus the <a id="ref-for-specificity④"></a>specificity of its argument.

Note: This means that the selector pierces through shadow boundaries on the way up, looking for elements that match its argument, until it reaches the document root.

Tests

#### <a id="slotted-pseudo"></a>3.2.4.  Selecting Slot-Assigned Content: the <a id="ref-for-selectordef-slotted"></a>[::slotted()](#selectordef-slotted) pseudo-element[](#slotted-pseudo)

The <a id="selectordef-slotted"></a><strong>::slotted()</strong> pseudo-element represents the elements <a id="ref-for-find-flattened-slotables"></a>[assigned, after flattening,](https://dom.spec.whatwg.org/#find-flattened-slotables) to a <a id="ref-for-concept-slot⑧"></a>[slot](https://dom.spec.whatwg.org/#concept-slot). This pseudo-element only exists on <a id="ref-for-concept-slot⑨"></a>slots.

The <a id="ref-for-selectordef-slotted①"></a>[::slotted()](#selectordef-slotted) pseudo-element is an <em>alias</em> for other elements in the tree, and does not generate any boxes itself.

The grammar of the <a id="ref-for-selectordef-slotted②"></a>[::slotted()](#selectordef-slotted) pseudo-element is:

<a id="ref-for-typedef-compound-selector③"></a>

``` text
::slotted( <compound-selector> )
```

The <a id="ref-for-selectordef-slotted③"></a>[::slotted()](#selectordef-slotted) pseudo-element represents the elements that are:

- <a id="ref-for-find-flattened-slotables①"></a>[assigned, after flattening,](https://dom.spec.whatwg.org/#find-flattened-slotables) to the <a id="ref-for-concept-slot①⓪"></a>[slot](https://dom.spec.whatwg.org/#concept-slot) that is ::slotted’s originating element

- <a id="ref-for-match-a-selector-against-an-element"></a>[matched](https://drafts.csswg.org/selectors-4/#match-a-selector-against-an-element) by its <a id="ref-for-typedef-compound-selector④"></a>[\<compound-selector\>](https://drafts.csswg.org/selectors-4/#typedef-compound-selector) argument

Tests

The <a id="ref-for-selectordef-slotted④"></a>[::slotted()](#selectordef-slotted) pseudo-element can be followed by a <a id="ref-for-tree-abiding"></a>[tree-abiding pseudo-element](https://drafts.csswg.org/css-pseudo-4/#tree-abiding), like ::slotted()::before, representing the appropriate pseudo-element of the elements represented by the <a id="ref-for-selectordef-slotted⑤"></a>::slotted() pseudo-element.

Tests

The <a id="ref-for-specificity⑤"></a>[specificity](https://drafts.csswg.org/selectors-4/#specificity) of <a id="ref-for-selectordef-slotted⑥"></a>[::slotted()](#selectordef-slotted) is that of a pseudo-element, plus the <a id="ref-for-specificity⑥"></a>specificity of its argument.

<a id="example-7cc70c2d"></a>

<strong>Example:</strong>

[](#example-7cc70c2d) For example, say you had a component with both children and a shadow tree, like the following:

``` text
<x-foo>
  <div id="one" slot="foo" class="foo">...</div>
  <div id="two" slot="foo">...</div>
  <div id="three" class="foo">
    <div id="four" slot="foo">...</div>
  </div>
  <"shadow tree">
    <div id="five">...</div>
    <div id="six">...</div>
    <slot name="foo"></slot>
  </"shadow tree">
</x-foo>
```

For a stylesheet within the <a id="ref-for-concept-shadow-tree③③"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), a selector like ::slotted(\*) selects \#one and \#two only, as they’re the elements <a id="ref-for-find-flattened-slotables②"></a>[assigned](https://dom.spec.whatwg.org/#find-flattened-slotables) to the sole <code><a id="ref-for-the-slot-element①"></a>[slot](https://html&#46;spec&#46;whatwg&#46;org/multipage/scripting&#46;html#the-slot-element)</code> element. It will <em>not</em> select \#three (no <code>slot</code> attribute) nor \#four (only direct <a id="ref-for-concept-tree-child①"></a>[children](https://dom.spec.whatwg.org/#concept-tree-child) of a <a id="ref-for-element-shadow-host①⑨"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) can be assigned to a <a id="ref-for-concept-slot①①"></a>[slot](https://dom.spec.whatwg.org/#concept-slot)).

A selector like ::slotted(.foo), on the other hand, will only select \#one, as it matches .foo, but \#two doesn’t.

Note: Note that a selector like ::slotted(\*) is equivalent to \*::slotted(\*), where the <a id="ref-for-x"></a>[\*](https://drafts.csswg.org/selectors-3/#x) selects many more elements than just the <code><a id="ref-for-the-slot-element②"></a>[slot](https://html&#46;spec&#46;whatwg&#46;org/multipage/scripting&#46;html#the-slot-element)</code> element. However, since only the <code><a id="ref-for-the-slot-element③"></a>[slot](https://html&#46;spec&#46;whatwg&#46;org/multipage/scripting&#46;html#the-slot-element)</code> elements are <a id="ref-for-concept-slot①②"></a>[slots](https://dom.spec.whatwg.org/#concept-slot), they’re the only elements with a <a id="ref-for-selectordef-slotted⑦"></a>[::slotted()](#selectordef-slotted) pseudo-element as well.

Note: <a id="ref-for-selectordef-slotted⑧"></a>[::slotted()](#selectordef-slotted) can only represent the <em>elements</em> assigned to the <a id="ref-for-concept-slot①③"></a>[slot](https://dom.spec.whatwg.org/#concept-slot). <a id="ref-for-concept-slot①④"></a>Slots can also be assigned text nodes, which can’t be selected by <a id="ref-for-selectordef-slotted⑨"></a>::slotted(). The only way to style assigned text nodes is by styling the <a id="ref-for-concept-slot①⑤"></a>slot and relying on inheritance.

Tests

#### <a id="the-has-slotted-pseudo"></a>3.2.5.  Matching on the Presence of Slot-Assigned Nodes: the <a id="ref-for-selectordef-has-slotted①"></a>[:has-slotted](#selectordef-has-slotted) pseudo-class[](#the-has-slotted-pseudo)

The <a id="selectordef-has-slotted"></a><strong>:has-slotted</strong> pseudo-class matches <code><a id="ref-for-the-slot-element④"></a>[slot](https://html&#46;spec&#46;whatwg&#46;org/multipage/scripting&#46;html#the-slot-element)</code> elements which have a non-empty list of <a id="ref-for-find-flattened-slotables③"></a>[flattened slotted nodes](https://dom.spec.whatwg.org/#find-flattened-slotables).

When <a id="ref-for-selectordef-has-slotted②"></a>[:has-slotted](#selectordef-has-slotted) matches a slot with fallback content, we can conclude that the fallback content is <em>not</em> being displayed.

Note: Even a single whitespace text node is sufficient to make <a id="ref-for-selectordef-has-slotted③"></a>[:has-slotted](#selectordef-has-slotted)' apply. This is by design, so that the behavior of this pseudo-class is consistent with the behavior of the <code><a id="ref-for-dom-slot-assignednodes"></a>[assignedNodes()](https://html&#46;spec&#46;whatwg&#46;org/multipage/scripting&#46;html#dom-slot-assignednodes)</code> method. A future version of this specification is expected to introduce a way to exclude this case from matching.

Note: It is expected that a future version of this specification will introduce a functional :has-slotted() pseudo-class that allows more fine-grained matching by accepting a selector argument. <a id="ref-for-selectordef-has-slotted④"></a>[:has-slotted](#selectordef-has-slotted) is <em>not</em> an alias of :has-slotted(\*), as the latter would not match slotted text nodes, but <a id="ref-for-selectordef-has-slotted⑤"></a>:has-slotted does.

Tests

## <a id="shadow-cascading"></a>4. Shadow Trees and the Cascade[](#shadow-cascading)

See [CSS Cascading 4 § 6.1 Cascade Sorting Order](https://drafts.csswg.org/css-cascade-4/#cascade-sort).

Tests

### <a id="flattening"></a>4.1.  Flattening the DOM into an Element Tree[](#flattening)

While Selectors operates on the DOM tree as the host language presents it, with separate trees that are unreachable via the standard parent/<a id="ref-for-concept-tree-child②"></a>[child](https://dom.spec.whatwg.org/#concept-tree-child) relationship, the rest of CSS needs a single unified tree structure to work with. This is called the <a id="flat-tree"></a><strong>flattened element tree</strong> (or <a id="ref-for-flat-tree①"></a>[flat tree](#flat-tree)), and is constructed as follows:

1.  Let <var>pending nodes</var> be a list of DOM nodes with associated parents, initially containing just the document’s <a id="ref-for-concept-tree-root①"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) element with no associated parent.

2.  Repeatedly execute the following substeps until <var>pending nodes</var> is empty:

    1.  Pop the first element from <var>pending nodes</var>, and assign it to <var>pending node</var>.

    2.  Insert <var>pending node</var> into the <a id="ref-for-flat-tree②"></a>[flat tree](#flat-tree) as a child of its associated parent. (If it has no associated parent, it’s the document root—​just insert it into the <a id="ref-for-flat-tree③"></a>flat tree as its root.)

    3.  Perform one of the following, whichever is the first that matches:

        <strong><var>pending node</var> is a <a id="ref-for-element-shadow-host②⓪"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host)</strong>

        Append the child nodes of the <a id="ref-for-concept-shadow-root⑥"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) of the <a id="ref-for-concept-shadow-tree③④"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) it hosts to <var>pending nodes</var>, with <var>pending node</var> as their associated parent.
        <strong><var>pending node</var> is a <a id="ref-for-concept-slot①⑥"></a>[slot](https://dom.spec.whatwg.org/#concept-slot)</strong>

        <a id="ref-for-find-slotables"></a>[Find slottables](https://dom.spec.whatwg.org/#find-slotables) for <var>pending node</var>, and append them to <var>pending nodes</var>, with <var>pending node</var> as their associated parent.
        If no <a id="ref-for-concept-slotable"></a>[slottables](https://dom.spec.whatwg.org/#concept-slotable) were found for <var>pending node</var>, instead append its <a id="ref-for-concept-tree-child③"></a>[children](https://dom.spec.whatwg.org/#concept-tree-child) to <var>pending nodes</var>, with <var>pending node</var> as their associated parent.

        <strong>Otherwise,</strong>

        Append the child nodes of <var>pending node</var>’s <a id="ref-for-concept-light-tree③"></a>[light tree](https://dom.spec.whatwg.org/#concept-light-tree) to <var>pending nodes</var>, with <var>pending node</var> as their associated parent.

Note: In other words, the <a id="ref-for-flat-tree④"></a>[flat tree](#flat-tree) is the top-level DOM tree, but <a id="ref-for-element-shadow-host②①"></a>[shadow hosts](https://dom.spec.whatwg.org/#element-shadow-host) are filled with their <a id="ref-for-concept-shadow-tree③⑤"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) children instead of their <a id="ref-for-concept-light-tree④"></a>[light tree](https://dom.spec.whatwg.org/#concept-light-tree) children (and this proceeds recursively if the <a id="ref-for-concept-shadow-tree③⑥"></a>shadow tree contains any <a id="ref-for-element-shadow-host②②"></a>shadow hosts), and <a id="ref-for-concept-slot①⑦"></a>[slots](https://dom.spec.whatwg.org/#concept-slot) get filled with the nodes that are assigned to them (and this proceeds recursively if the <a id="ref-for-concept-slot①⑧"></a>slots are themselves assigned to a <a id="ref-for-concept-slot①⑨"></a>slot in a deeper <a id="ref-for-concept-shadow-tree③⑦"></a>shadow tree).

<a id="issue-75772618"></a>

<strong>Issue:</strong>

[](#issue-75772618) A non-obvious result of this is that elements assigned to a slot inherit from that slot, not their light-tree parent or any deeper slots their slot gets assigned to. This means that text nodes are styled by the shadow tree of their parent, with nobody else capable of intervening in any way. Do we want an additional pseudo-element for targeting those text nodes so they can be styled at all slot-assignment levels, like normal elements can be? This implies it needs to work for text nodes in the light tree before they’re assigned downwards, so this can’t just be a <a id="ref-for-selectordef-slotted①⓪"></a>[::slotted()](#selectordef-slotted) variant. Luckily, this is a long-standing request!

#### <a id="slots-in-shadow-tree"></a>4.1.1.  Slots and Slotted Elements in a Shadow Tree[](#slots-in-shadow-tree)

<a id="ref-for-concept-slot②⓪"></a>[Slots](https://dom.spec.whatwg.org/#concept-slot) must act as if they were assigned <a id="ref-for-propdef-display①"></a>[display: contents](https://drafts.csswg.org/css-display-4/#propdef-display) via a rule in the <a id="ref-for-cascade-origin-ua②"></a>[UA origin](https://drafts.csswg.org/css-cascade-5/#cascade-origin-ua). This must be possible to override via <a id="ref-for-propdef-display②"></a>display, so they <em>do</em> generate boxes if desired.

Note: A non-obvious result of assigning elements to <a id="ref-for-concept-slot②①"></a>[slots](https://dom.spec.whatwg.org/#concept-slot) is that they inherit from the <a id="ref-for-concept-slot②②"></a>slot they’re assigned to. Their original <a id="ref-for-concept-light-tree⑤"></a>[light tree](https://dom.spec.whatwg.org/#concept-light-tree) parent, and any deeper <a id="ref-for-concept-slot②③"></a>slots that their <a id="ref-for-concept-slot②④"></a>slot gets assigned to, don’t affect inheritance.

Tests

### <a id="shadow-names"></a>4.2.  Name-Defining Constructs and Inheritance[](#shadow-names)

<a id="ref-for-concept-shadow-tree③⑧"></a>[Shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) are meant to be an encapsulation boundary, allowing independent authors to share code without accidentally polluting each other’s namespaces. For example, element IDs, which are generally meant to be unique within a document, can be validly used multiple times as long as each use is in a different <a id="ref-for-concept-shadow-tree③⑨"></a>shadow tree.

Similarly, several <a id="ref-for-at-rule①"></a>[at-rules](https://drafts.csswg.org/css-syntax-3/#at-rule) in CSS, such as <a id="ref-for-at-ruledef-keyframes"></a>[&#64;keyframes](https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes) or <a id="ref-for-at-font-face-rule①"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule), define a name that later at-rules or properties can refer to them by. Like IDs, these names are globally exposed and unique within a document; also like IDs, this restriction is now loosened to being unique within a given <a id="ref-for-concept-shadow-tree④⓪"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree).

However, property inheritance can carry values from one tree to another, which complicates referencing the correct definition of a given name. Done naively, this can produce surprising and confusing results for authors. This section defines a set of concepts to use in defining and referencing "global" names in a way that respects encapsulation and doesn’t give surprising results.

If an at-rule or property defines a name that other CSS constructs can refer to it by, such as a <a id="ref-for-at-font-face-rule②"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule) <a id="ref-for-descdef-font-face-font-family"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-family) name or an <a id="ref-for-propdef-anchor-scope"></a>[anchor-scope](https://drafts.csswg.org/css-anchor-position-1/#propdef-anchor-scope) name, it must be defined as a <a id="css-tree-scoped-name"></a><strong>tree-scoped name</strong>. <a id="ref-for-css-tree-scoped-name"></a>[Tree-scoped names](#css-tree-scoped-name) are associated with the <a id="ref-for-concept-tree-root②"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) of the <a id="ref-for-concept-element"></a>[element](https://dom.spec.whatwg.org/#concept-element) hosting the stylesheet that the at-rule or property is declared in.

Additionally, <a id="ref-for-css-tree-scoped-name①"></a>[tree-scoped names](#css-tree-scoped-name) can be <a id="tree-scoped-name-loosely-matched"></a><strong>loosely matched</strong> or <a id="tree-scoped-name-strictly-matched"></a><strong>strictly matched</strong>, defaulting to <a id="ref-for-tree-scoped-name-loosely-matched"></a>[loosely matched](#tree-scoped-name-loosely-matched) unless otherwise specified. A <a id="ref-for-tree-scoped-name-loosely-matched①"></a>loosely matched <a id="ref-for-css-tree-scoped-name②"></a>tree-scoped name can be matched by <a id="ref-for-css-tree-scoped-reference"></a>[tree-scoped references](#css-tree-scoped-reference) (see below) in the same tree or descendant trees, while a <a id="ref-for-tree-scoped-name-strictly-matched"></a>[strictly matched](#tree-scoped-name-strictly-matched) <a id="ref-for-css-tree-scoped-name③"></a>tree-scoped name can only be matched by <a id="ref-for-css-tree-scoped-reference①"></a>tree-scoped references in the exact same tree.

Properties or descriptors that reference a <a id="ref-for-css-tree-scoped-name④"></a>[tree-scoped name](#css-tree-scoped-name), such as the <a id="ref-for-propdef-font-family"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) or <a id="ref-for-propdef-animation-name"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name) properties, must define their value as a <a id="css-tree-scoped-reference"></a><strong>tree-scoped reference</strong>. These references implicitly capture a <a id="ref-for-concept-node-tree"></a>[node tree](https://dom.spec.whatwg.org/#concept-node-tree) <a id="ref-for-concept-tree-root③"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) along with their specified value: unless otherwise specified, the <a id="ref-for-concept-tree-root④"></a>root of the <a id="ref-for-concept-element①"></a>[element](https://dom.spec.whatwg.org/#concept-element) hosting the stylesheet that the property or descriptor is declared in. This <a id="ref-for-concept-tree-root⑤"></a>root reference stays with the <a id="ref-for-css-tree-scoped-reference②"></a>[tree-scoped reference](#css-tree-scoped-reference) as it is inherited.

Tests

If a <a id="ref-for-css-tree-scoped-name⑤"></a>[tree-scoped name](#css-tree-scoped-name) is <a id="tree-scoped-name-global"></a><strong>global</strong> (such as <a id="ref-for-at-font-face-rule③"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule) names), then when a <a id="ref-for-css-tree-scoped-reference③"></a>[tree-scoped reference](#css-tree-scoped-reference) is dereferenced to find it, first search only the <a id="ref-for-css-tree-scoped-name⑥"></a>tree-scoped names associated with the same <a id="ref-for-concept-tree-root⑥"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) as the <a id="ref-for-css-tree-scoped-reference④"></a>tree-scoped reference. If no relevant <a id="ref-for-css-tree-scoped-name⑦"></a>tree-scoped name is found, and the <a id="ref-for-concept-tree-root⑦"></a>root is a <a id="ref-for-concept-shadow-root⑦"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root), then repeat this search in the <a id="ref-for-concept-tree-root⑧"></a>root’s <a id="ref-for-concept-documentfragment-host①"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host)’s <a id="ref-for-concept-node-tree①"></a>[node tree](https://dom.spec.whatwg.org/#concept-node-tree) (recursively). (In other words, global <a id="ref-for-css-tree-scoped-name⑧"></a>tree-scoped names “inherit” into descendant shadow trees, so long as they don’t define the same name themselves.)

If a <a id="ref-for-css-tree-scoped-name⑨"></a>[tree-scoped name](#css-tree-scoped-name) is <a id="tree-scoped-name-local"></a><strong>local</strong> to an element (such as <a id="ref-for-propdef-anchor-name"></a>[anchor-name](https://drafts.csswg.org/css-anchor-position-1/#propdef-anchor-name) or <a id="ref-for-propdef-anchor-scope①"></a>[anchor-scope](https://drafts.csswg.org/css-anchor-position-1/#propdef-anchor-scope) values), then whether a <a id="ref-for-css-tree-scoped-reference⑤"></a>[tree-scoped reference](#css-tree-scoped-reference) matches the <a id="ref-for-css-tree-scoped-name①⓪"></a>tree-scoped name on a given element depends on whether the <a id="ref-for-css-tree-scoped-name①①"></a>tree-scoped name is <a id="ref-for-tree-scoped-name-strictly-matched①"></a>[strictly](#tree-scoped-name-strictly-matched) or <a id="ref-for-tree-scoped-name-loosely-matched②"></a>[loosely](#tree-scoped-name-loosely-matched) matched. A <a id="ref-for-tree-scoped-name-strictly-matched②"></a>strictly matched <a id="ref-for-css-tree-scoped-name①②"></a>tree-scoped name only matches if both names are associated with the same tree. A <a id="ref-for-tree-scoped-name-loosely-matched③"></a>loosely matched <a id="ref-for-css-tree-scoped-name①③"></a>tree-scoped name also matches if the <a id="ref-for-css-tree-scoped-name①④"></a>tree-scoped name is associated with an ancestor tree.

If two <a id="ref-for-css-tree-scoped-name①⑤"></a>[tree-scoped names](#css-tree-scoped-name) are directly compared (for example, when comparing <a id="ref-for-computed-value"></a>[computed values](https://drafts.csswg.org/css-cascade-5/#computed-value)), they are considered to match only if their identifiers match, <em>and</em> their <a id="ref-for-concept-tree-root⑨"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root)s match exactly. (If one has a <a id="ref-for-concept-tree-root①⓪"></a>root that’s an ancestor of the other, for example, they <em>do not</em> match.)

<a id="issue-3293692a"></a>

<strong>Issue:</strong>

[](#issue-3293692a) TODO: Fix all the at-rules that define global names, and the properties that reference them, to use these concepts.

Global names include:

- <a id="ref-for-at-font-face-rule④"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule), referenced by <a id="ref-for-propdef-font-family①"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family)

- <a id="ref-for-at-ruledef-font-feature-values"></a>[&#64;font-feature-values](https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values), referenced by <a id="ref-for-propdef-font-family②"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family)

- <a id="ref-for-at-ruledef-keyframes①"></a>[&#64;keyframes](https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes), referenced by <a id="ref-for-propdef-animation-name①"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name)

- <a id="ref-for-at-ruledef-counter-style"></a>[&#64;counter-style](https://drafts.csswg.org/css-counter-styles-3/#at-ruledef-counter-style), referenced by <a id="ref-for-propdef-list-style-type"></a>[list-style-type](https://drafts.csswg.org/css-lists-3/#propdef-list-style-type)

- <a id="ref-for-at-ruledef-profile"></a>[&#64;color-profile](https://drafts.csswg.org/css-color-5/#at-ruledef-profile), referenced by the <a id="ref-for-funcdef-color"></a>[color()](https://drafts.csswg.org/css-color-5/#funcdef-color) function

- <a id="ref-for-at-ruledef-font-palette-values"></a>[&#64;font-palette-values](https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-palette-values), referenced by <a id="ref-for-propdef-font-palette"></a>[font-palette](https://drafts.csswg.org/css-fonts-4/#propdef-font-palette)

- others?

<a id="example-cd0cbc57"></a>

<strong>Example:</strong>

[](#example-cd0cbc57)

For example, given the following document (using the imaginary \<::shadow\>\</::shadow\> markup to indicate an element’s <a id="ref-for-concept-shadow-tree④①"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree)):

``` text
<p class=outer>Here's some text in the outer document's "foo" font.
<style>
  @font-face {
    font-family: foo;
    src: url(https://example.com/outer.woff);
  }
  body { font-family: foo; }
  my-component::part(text) { font-family: foo; }
</style>

<my-component>
  <::shadow>
    <p class=inner-default>I'm inheriting the outer document's font-family.
    <p class=inner-styled>And I'm explicitly styled to be in the component's "foo" font.
    <p class=part-styled part=text>
      I'm explicitly styled by the outer document,
      and get the outer document's "foo" font.
    <style>
      @font-face {
        font-family: foo;
        src: url(https://example.com/inner.woff);
      }
      .inner-styled { font-family: foo; }
    </style>
  </::shadow>
</my-component>
```

The .outer element references the outer <a id="ref-for-at-font-face-rule⑤"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule), using the "outer.woff" file.

The .inner-default element inherits the <a id="ref-for-propdef-font-family③"></a>[font-family: foo](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) value from the outer document, using the same <a id="ref-for-css-tree-scoped-reference⑥"></a>[tree-scoped reference](#css-tree-scoped-reference) as .outer, and thus also uses the "outer.woff" font file.

The .inner-style element, on the other hand, receives a <a id="ref-for-propdef-font-family④"></a>[font-family: foo](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) from the stylesheet inside the shadow, and thus its <a id="ref-for-css-tree-scoped-reference⑦"></a>[tree-scoped reference](#css-tree-scoped-reference) refers to the shadow’s &#64;font-family, and it uses the "inner.woff" file.

The .part-styled element also receives its style from the outer document, tho by being directly set rather than by inheritance. Thus, its <a id="ref-for-css-tree-scoped-reference⑧"></a>[tree-scoped reference](#css-tree-scoped-reference) also refer’s to the outer document, and it uses the "outer.woff" file.

<a id="example-6de7b535"></a>

<strong>Example:</strong>

[](#example-6de7b535) Here is a more complex example, showing three levels of trees, and illustrating precisely how <a id="ref-for-css-tree-scoped-name①⑥"></a>[tree-scoped names](#css-tree-scoped-name) and <a id="ref-for-css-tree-scoped-reference⑨"></a>[tree-scoped references](#css-tree-scoped-reference) inherit.

``` text
<style>
  @font-face {
    font-family: foo;
    src: url(https://example.com/outer.woff);
  }
  body { font-family: foo; }
</style>

<child-component>
  <::shadow>
    <style>
      @font-face {
        font-family: foo;
        src: url(https://example.com/inner.woff);
      }
    </style>

    <grandchild-component>
      <::shadow>
        <p class=inner-default>
          I'm inheriting the outer document's "foo" font.
        </p>
        <p class=inner-search>
          And I can't find a local "foo" font,
          so I'm searching further up the tree,
          and find the shadow's "foo" font.
        </p>
        <style>
        .inner-search { font-family: foo; }
        </style>
      </::shadow>
    </grandchild-component>
  </::shadow>
</child-component>
```

Here, just as in the previous example, .inner-default is inheriting the <a id="ref-for-propdef-font-family⑤"></a>[font-family: foo](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) declared in the outer document, and so it ends up referencing the outer document’s <a id="ref-for-at-font-face-rule⑥"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule), and is rendered with the "outer.woff" file.

On the other hand, .inner-search receives its style from a stylesheet in <code>&#60;grandchild-component&#62;</code>’s shadow tree, so it attempts to find a <a id="ref-for-at-font-face-rule⑦"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule) defining a foo font in that tree. There is no such <a id="ref-for-at-font-face-rule⑧"></a>&#64;font-face, so it starts walking up the shadow trees, finding an appropriate <a id="ref-for-at-font-face-rule⑨"></a>&#64;font-face in <code>&#60;child-component&#62;</code>, so it’s rendered with the "inner.woff" file.

Tests

#### <a id="shadow-names-serialization"></a>4.2.1.  Serialized Tree-Scoped References[](#shadow-names-serialization)

If a <a id="ref-for-css-tree-scoped-reference①⓪"></a>[tree-scoped reference](#css-tree-scoped-reference) is serialized, it serializes only its value; the associated <a id="ref-for-concept-tree-root①①"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) is lost.

<a id="example-ad083628"></a>

<strong>Example:</strong>

[](#example-ad083628)

This implies that \`el.style.foo = getComputedStyle(el).foo;\` is not necessarily a no-op, like it typically was before <a id="ref-for-concept-shadow-tree④②"></a>[shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) existed.

For example, given the following document (using the imaginary \<::shadow\>\</::shadow\> markup to indicate an element’s <a id="ref-for-concept-shadow-tree④③"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree)):

``` text
<p class=outer>Here's some text in the outer document's "foo" font.
<style>
  @font-face {
    font-family: foo;
    src: url(foo.woff);
  }
  body { font-family: foo; }
</style>

<my-component>
  <::shadow>
    <p class=inner-default>I'm inheriting the outer document's font-family.
    <p class=inner-styled>And I'm explicitly styled to be in the component's "foo" font.
    <style>
      @font-face {
        font-family: foo;
        src: url(https://example.com/foo.woff);
      }
      .inner-styled { font-family: foo; }
    </style>
    <script>
      const innerDefault = document.querySelector('.inner-default');
      const innerStyled = document.querySelector('.inner-styled');
      const defaultFont = getComputedStyle(innerDefault).fontFamily;
      const styledFont = getComputedStyle(innerStyled).fontFamily;

      console.log(defaultFont == styledFont); // true!
    </script>
  </::shadow>
</my-component>
```

The <code>&#46;outer</code> element is styled with the outer document’s "foo" <a id="ref-for-at-font-face-rule①⓪"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule). The <code>&#46;inner-default</code> element inherits <a id="ref-for-propdef-font-family⑥"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) from the outer document, meaning it inherits a <a id="ref-for-css-tree-scoped-reference①①"></a>[tree-scoped reference](#css-tree-scoped-reference) referencing that outer document, and so it’s in the same font as <code>&#46;outer</code>.

Meanwhile, <code>&#46;inner-styled</code> is explicitly styled from inside the shadow root, so it receives a fresh <a id="ref-for-css-tree-scoped-reference①②"></a>[tree-scoped reference](#css-tree-scoped-reference) referencing its shadow tree, and it is instead styled the shadow’s own "foo" <a id="ref-for-at-font-face-rule①①"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule).

Despite that, the script running inside the component sees the two elements as having the same value for <a id="ref-for-propdef-font-family⑦"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family), because the <a id="ref-for-concept-tree-root①②"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root)-reference part of a <a id="ref-for-css-tree-scoped-reference①③"></a>[tree-scoped reference](#css-tree-scoped-reference) is not preserved by serialization. If it were to set <code>innerDefault&#46;style&#46;fontFamily = defaultFont;</code> (thus setting the <a id="ref-for-propdef-font-family⑧"></a>font-family property of the element’s attribute stylesheet, which lives in the shadow tree), the <code>&#46;inner-default</code> element would suddenly switch to the same font as <code>&#46;inner-styled</code>!

Note: The [\[css-typed-om-1\]](#biblio-css-typed-om-1) is expected to reflect the <a id="ref-for-concept-tree-root①③"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) reference of a <a id="ref-for-css-tree-scoped-reference①④"></a>[tree-scoped reference](#css-tree-scoped-reference) in its <a id="ref-for-css-reify"></a>[reification](https://drafts.css-houdini.org/css-typed-om-1/#css-reify) rules for values, allowing authors to tell what <a id="ref-for-concept-node-tree②"></a>[node tree](https://dom.spec.whatwg.org/#concept-node-tree) the reference is taking its values from, and allowing values to be transported across <a id="ref-for-concept-node-tree③"></a>node trees without changing their meaning.

## <a id="exposing"></a>5. Exposing a Shadow Element[](#exposing)

Elements in a <a id="ref-for-concept-shadow-tree④④"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) may be exported for styling by stylesheets outside the tree using the part and exportparts attributes.

Each element has a <a id="element-part-name-list"></a><strong>part name list</strong> which is an <a id="ref-for-ordered-set"></a>[ordered set](https://infra.spec.whatwg.org/#ordered-set) of tokens.

Each element has a <a id="element-forwarded-part-name-list"></a><strong>forwarded part name list</strong> which is a <a id="ref-for-list"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-tuple"></a>[tuples](https://infra.spec.whatwg.org/#tuple) containing a <a id="ref-for-string"></a>[string](https://infra.spec.whatwg.org/#string) for the inner part being forwarded and a <a id="ref-for-string①"></a>string giving the name it will be exposed as.

Each <a id="ref-for-concept-shadow-root⑧"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) can be thought of as having a <a id="shadow-root-part-element-map"></a><strong>part element map</strong> with keys that are <a id="ref-for-string②"></a>[strings](https://infra.spec.whatwg.org/#string) and values that are <a id="ref-for-ordered-set①"></a>[ordered sets](https://infra.spec.whatwg.org/#ordered-set) of elements.

The <a id="ref-for-shadow-root-part-element-map"></a>[part element map](#shadow-root-part-element-map) is described only as part of the algorithm for calculating style in this spec. It is not exposed via the DOM, as calculating it may be expensive and exposing it could allow access to elements inside closed shadow roots.

<a id="ref-for-shadow-root-part-element-map①"></a>[Part element maps](#shadow-root-part-element-map) are affected by the addition and removal of elements and changes to the <a id="ref-for-element-part-name-list"></a>[part name lists](#element-part-name-list) and <a id="ref-for-element-forwarded-part-name-list"></a>[forwarded part name lists](#element-forwarded-part-name-list) of elements in the DOM.

To <a id="calculate-the-part-element-map"></a><strong>calculate the <a id="ref-for-shadow-root-part-element-map②"></a>[part element map](#shadow-root-part-element-map)</strong> of a shadow root, <var>outerRoot</var>:

1.  For each <a id="ref-for-concept-tree-descendant②"></a>[descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) <var>el</var> within <var>outerRoot</var>:

    1.  For each <var>name</var> in <var>el</var>’s <a id="ref-for-element-part-name-list①"></a>[part name list](#element-part-name-list), <a id="ref-for-list-append"></a>[append](https://infra.spec.whatwg.org/#list-append) <var>el</var> to <var>outerRoot</var>’s <a id="ref-for-shadow-root-part-element-map③"></a>[part element map](#shadow-root-part-element-map)\[<var>name</var>\].

    2.  If <var>el</var> is a <a id="ref-for-element-shadow-host②③"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) itself then let <var>innerRoot</var> be its shadow root.

    3.  <a id="ref-for-calculate-the-part-element-map"></a>[Calculate](#calculate-the-part-element-map) <var>innerRoot</var>’s <a id="ref-for-shadow-root-part-element-map④"></a>[part element map](#shadow-root-part-element-map).

    4.  For each <var>innerName</var>/<var>outerName</var> in <var>el</var>’s <a id="ref-for-element-forwarded-part-name-list①"></a>[forwarded part name list](#element-forwarded-part-name-list):

        1.  If <var>innerName</var> is an ident:

            1.  Let <var>innerParts</var> be <var>innerRoot</var>’s <a id="ref-for-shadow-root-part-element-map⑤"></a>[part element map](#shadow-root-part-element-map)\[<var>innerName</var>\]

            2.  <a id="ref-for-list-append①"></a>[Append](https://infra.spec.whatwg.org/#list-append) the elements in <var>innerParts</var> to <var>outerRoot</var>’s <a id="ref-for-shadow-root-part-element-map⑥"></a>[part element map](#shadow-root-part-element-map)\[<var>outerName</var>\]

        2.  If <var>innerName</var> is a pseudo-element name:

            1.  <a id="ref-for-list-append②"></a>[Append](https://infra.spec.whatwg.org/#list-append) <var>innerRoot</var>’s pseudo-element(s) with that name to <var>outerRoot</var>’s <a id="ref-for-shadow-root-part-element-map⑦"></a>[part element map](#shadow-root-part-element-map)\[<var>outerName</var>\].

### <a id="motivation"></a>5.1. Motivation[](#motivation)

For custom elements to be fully useful and as capable as built-in elements it should be possible for parts of them to be styled from outside. Exactly what can be styled from outside should be controlled by the element author. Also, it should be possible for a custom element to present a stable "API" for styling. That is, the selector used to style a part of a custom element should not expose or require knowledge of the internal details of the element. The custom element author should be able to change the internal details of the element while leaving the selectors untouched.

The previous proposed method for styling inside the shadow tree, the \>\>\> combinator, turned out to be <em>too powerful</em> for its own good; it exposed too much of a component’s internal structure to scrutiny, defeating some of the encapsulation benefits that using Shadow DOM brings. For this, and other performance-related reasons, the \>\>\> combinator was eventually dropped.

This left us with using <a id="ref-for-custom-property①"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) as the only way to style into a shadow tree: the component would advertise that it uses certain <a id="ref-for-custom-property②"></a>custom properties to style its internals, and the outer page could then set those properties as it wished on the <a id="ref-for-element-shadow-host②④"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host), letting inheritance push the values down to where they were needed. This works very well for many simple theming use-cases.

However, there are some cases where this falls down. If a component wishes to allow arbitrary styling of something in its shadow tree, the only way to do so is to define hundreds of <a id="ref-for-custom-property③"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) (one per CSS property they wish to allow control of), which is obviously ridiculous for both usability and performance reasons. The situation is compounded if authors wish to style the component differently based on pseudo-classes like <a id="ref-for-hover-pseudo"></a>[:hover](https://drafts.csswg.org/selectors-4/#hover-pseudo); the component needs to duplicate the <a id="ref-for-custom-property④"></a>custom properties used for each pseudo-class (and each combination, like :hover:focus, resulting in a combinatorial explosion). This makes the usability and performance problems even worse.

We introduce <a id="ref-for-selectordef-part①"></a>[::part()](#selectordef-part) to handle this case much more elegantly and performantly. Rather than bundling everything into <a id="ref-for-custom-property⑤"></a>[custom property](https://drafts.csswg.org/css-variables-2/#custom-property) names, the functionality lives in selectors and style rule syntax, like it’s meant to. This is far more usable for both component authors and component users, should have much better performance, and allows for better encapsulation/API surface.

It’s important to note that <a id="ref-for-selectordef-part②"></a>[::part()](#selectordef-part) offers <em>absolutely zero new theoretical power</em>. It is not a rehash of the \>\>\> combinator, it is simply a more convenient and consistent syntax for something authors can already do with <a id="ref-for-custom-property⑥"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property). By separating out the explicitly "published" parts of an element (the <a id="ref-for-shadow-root-part-element-map⑧"></a>[part element map](#shadow-root-part-element-map)) from the sub-parts that it merely happens to contain, it also helps with encapsulation, as authors can use <a id="ref-for-selectordef-part③"></a>::part() without fear of accidental over-styling.

### <a id="part-attr"></a>5.2. Naming a Shadow Element: the <code><a id="ref-for-element-attrdef-html-global-part"></a>[part](#element-attrdef-html-global-part)</code> attribute[](#part-attr)

Any element in a shadow tree can have a <a id="element-attrdef-html-global-part"></a><strong><code>part</code></strong> attribute. This is used to expose the element outside of the <a id="ref-for-concept-shadow-tree④⑤"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree).

Tests

The part attribute is parsed as a space-separated list of tokens representing the part names of this element.

Note: It’s okay to give a part multiple names. The "part name" should be considered similar to a class, not an id or tagname.

<a id="example-a2e8f894"></a>

``` text
<style>
  c-e::part(textspan) { color: red; }
</style>

<template id="c-e-template">
  <span part="textspan">This text will be red</span>
</template>
<c-e></c-e>
<script>
  // Add template as custom element c-e
  ...
</script>
```

### <a id="exportparts-attr"></a>5.3. Forwarding a Shadow Element: the <code><a id="ref-for-element-attrdef-html-global-exportparts"></a>[exportparts](#element-attrdef-html-global-exportparts)</code> attribute[](#exportparts-attr)

Any element in a shadow tree can have a <a id="element-attrdef-html-global-exportparts"></a><strong><code>exportparts</code></strong> attribute. If the element is a shadow host, this is used to allow styling of parts from hosts inside the <a id="ref-for-concept-shadow-tree④⑥"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) by rules outside this the <a id="ref-for-concept-shadow-tree④⑦"></a>shadow tree (as if they were elements in the same tree as the host, named by a part attribute).

Tests

The exportparts attribute is parsed as a comma-separated list of part mappings. Each part mapping is one of:

<strong><code>innerIdent &#58; outerIdent</code></strong>

Adds <code>innerIdent</code>/<code>outerIdent</code> to el’s <a id="ref-for-element-forwarded-part-name-list②"></a>[forwarded part name list](#element-forwarded-part-name-list).

<strong><code>ident</code></strong>

Adds <code>ident</code>/<code>ident</code> to el’s <a id="ref-for-element-forwarded-part-name-list③"></a>[forwarded part name list](#element-forwarded-part-name-list).

Note: This is shorthand for <code>ident &#58; ident</code>.

<strong><code>&#58;&#58;ident &#58; outerIdent</code></strong>

If <code>&#58;&#58;ident</code> is the name of a <a id="ref-for-fully-styleable"></a>[fully styleable pseudo-element](https://drafts.csswg.org/css-pseudo-4/#fully-styleable), adds <code>&#58;&#58;ident</code>/<code>outerIdent</code> to el’s <a id="ref-for-element-forwarded-part-name-list④"></a>[forward part name list](#element-forwarded-part-name-list). Otherwise, does nothing.

<strong>anything else</strong>

Ignored for error-recovery / future compatibility.

Note: It’s okay to map a sub-part to several names.

<a id="example-8f835643"></a>

``` text
<style>
  c-e::part(textspan) { color: red; }
</style>

<template id="c-e-outer-template">
  <c-e-inner exportparts="innerspan: textspan"></c-e-inner>
</template>

<template id="c-e-inner-template">
  <span part="innerspan">
    This text will be red because the containing shadow
    host forwards innerspan to the document as "textspan"
    and the document style matches it.
  </span>
  <span part="textspan">
    This text will not be red because textspan in the document style
    cannot match against the part inside the inner custom element
    if it is not forwarded.
  </span>
</template>

<c-e></c-e>
<script>
  // Add template as custom elements c-e-inner, c-e-outer
  ...
</script>
```

<a id="example-b11a0a88"></a>

<strong>Example:</strong>

[](#example-b11a0a88) For example, a <a id="ref-for-fully-styleable①"></a>[fully styleable pseudo-element](https://drafts.csswg.org/css-pseudo-4/#fully-styleable) can be used in the <code><a id="ref-for-element-attrdef-html-global-exportparts①"></a>[exportparts](#element-attrdef-html-global-exportparts)</code> attribute, to masquerade as a <a id="ref-for-selectordef-part④"></a>[::part()](#selectordef-part) for the component it’s in:

``` text
<template id=custom-element-template>
  <p exportparts="::before : preceding-text, ::after : following-text">
    Main text.
</template>
```

An element using that template can use a selector like x-component::part(preceding-text) to target the p::before pseudo-element in its shadow, so users of the component don’t need to know that the preceding text is implemented as a pseudo-element.

### <a id="part"></a>5.4. Selecting a Shadow Element: the <a id="ref-for-selectordef-part⑤"></a>[::part()](#selectordef-part) pseudo-element[](#part)

The <a id="selectordef-part"></a><strong>::part()</strong> pseudo-element allows you to select elements that have been exposed via a <code><a id="ref-for-element-attrdef-html-global-part①"></a>[part](#element-attrdef-html-global-part)</code> attribute. The syntax is:

<a id="ref-for-typedef-ident"></a><a id="ref-for-mult-one-plus"></a>

``` text
::part() = ::part( <ident>+ )
```

Tests

The <a id="ref-for-selectordef-part⑥"></a>[::part()](#selectordef-part) pseudo-element only matches anything when the <a id="ref-for-originating-element"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) is a <a id="ref-for-element-shadow-host②⑤"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host).

<a id="example-c617aa59"></a>

<strong>Example:</strong>

[](#example-c617aa59) For example, if you have a custom button that contains a "label" element that is exposed for styling (via <code>part=&#34;label&#34;</code>), you can select it with x-button::part(label).

<a id="example-72ff019b"></a>

<strong>Example:</strong>

[](#example-72ff019b) Part names act similarly to classes: multiple elements can have the same part name, and a single element can have multiple part names.

A tabstrip control might have multiple elements with <code>part=&#34;tab&#34;</code>, all of which are selected by ::part(tab).

If a single tab is active at a time, it can be specially indicated with <code>part=&#34;tab active&#34;</code> and then selected by ::part(tab active) (or ::part(active tab), as order doesn’t matter).

The <a id="ref-for-selectordef-part⑦"></a>[::part()](#selectordef-part) pseudo-element is a <a id="ref-for-fully-styleable②"></a>[fully styleable pseudo-element](https://drafts.csswg.org/css-pseudo-4/#fully-styleable). If the <a id="ref-for-originating-element①"></a>[originating element’s](https://drafts.csswg.org/selectors-4/#originating-element) <a id="ref-for-concept-shadow-root⑨"></a>[shadow root’s](https://dom.spec.whatwg.org/#concept-shadow-root) <a id="ref-for-shadow-root-part-element-map⑨"></a>[part element map](#shadow-root-part-element-map) <a id="ref-for-map-exists"></a>[contains](https://infra.spec.whatwg.org/#map-exists) the specified <a id="ref-for-typedef-ident①"></a>[\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident), <a id="ref-for-selectordef-part⑧"></a>::part() represents the elements keyed to that ident; if multiple idents are provided and the <a id="ref-for-shadow-root-part-element-map①⓪"></a>part element map contains them all, it represents the intersection of the elements keyed to each ident. Otherwise, it matches nothing.

<a id="ref-for-selectordef-part⑨"></a>[::part()](#selectordef-part) pseudo-elements inherit according to their position in the <a id="ref-for-originating-element②"></a>[originating element’s](https://drafts.csswg.org/selectors-4/#originating-element) shadow tree.

<a id="example-11b18654"></a>

<strong>Example:</strong>

[](#example-11b18654) For example, x-panel::part(confirm-button)::part(label) never matches anything. This is because doing so would expose more structural information than is intended.

If the <code>&#60;x-panel&#62;</code>’s internal confirm button had used something like <code>part=&#34;label =&#62; confirm-label&#34;</code> to forward the button’s internal parts up into the panel’s own <a id="ref-for-shadow-root-part-element-map①①"></a>[part element map](#shadow-root-part-element-map), then a selector like x-panel::part(confirm-label) would select just the one button’s label, ignoring any other labels.

### <a id="idl"></a>5.5. Extensions to the <code><a id="ref-for-element"></a>[Element](https://dom&#46;spec&#46;whatwg&#46;org/#element)</code> Interface[](#idl)

<a id="ref-for-element①"></a><a id="ref-for-SameObject"></a><a id="ref-for-PutForwards"></a><a id="ref-for-dom-domtokenlist-value"></a><a id="ref-for-domtokenlist"></a><a id="dom-element-part"></a>

``` text
partial interface Element {
  [SameObject, PutForwards=value] readonly attribute DOMTokenList part;
};
```

Tests

The part attribute’s getter must return a DOMTokenList object whose associated element is the context object and whose associated attribute’s local name is part. The token set of this particular DOMTokenList object are also known as the element’s parts.

<a id="issue-853df044"></a>

<strong>Issue:</strong>

[](#issue-853df044) Define this as a superglobal in the DOM spec. [\[w3c/csswg-drafts Issue \#3424\]](https://github.com/w3c/csswg-drafts/issues/3424)

### <a id="parsing"></a>5.6. Microsyntaxes for parsing[](#parsing)

#### <a id="parsing-mapping"></a>5.6.1.  Rules for parsing part mappings[](#parsing-mapping)

A <a id="valid-part-mapping"></a><strong>valid part mapping</strong> is a <a id="ref-for-tuple①"></a>[tuple](https://infra.spec.whatwg.org/#tuple) of tokens separated by a U+003A COLON character and any number of space characters before or after the U+003A COLON The tokens must not contain U+003A COLON or U+002C COMMA characters.

The rules for parsing a part mapping are as follows:

1.  Let <var>input</var> be the string being parsed.

2.  Let <var>position</var> be a pointer into <var>input</var>, initially pointing at the start of the string.

3.  <a id="ref-for-collect-a-sequence-of-code-points"></a>[Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters

4.  <a id="ref-for-collect-a-sequence-of-code-points①"></a>[Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are not space characters or U+003A COLON characters, and let <var>first token</var> be the result.

5.  If <var>first token</var> is empty then return error.

6.  <a id="ref-for-collect-a-sequence-of-code-points②"></a>[Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters.

7.  If the end of the <var>input</var> has been reached, return the <a id="ref-for-tuple②"></a>[tuple](https://infra.spec.whatwg.org/#tuple) (<var>first token</var>, <var>first token</var>)

8.  If character at <var>position</var> is not a U+003A COLON character, return error.

9.  Consume the U+003A COLON character.

10. <a id="ref-for-collect-a-sequence-of-code-points③"></a>[Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters.

11. <a id="ref-for-collect-a-sequence-of-code-points④"></a>[Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are not space characters or U+003A COLON characters. and let <var>second token</var> be the result.

12. If <var>second token</var> is empty then return error.

13. <a id="ref-for-collect-a-sequence-of-code-points⑤"></a>[Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters.

14. If <var>position</var> is not past the end of <var>input</var> then return error.

15. Return the <a id="ref-for-tuple③"></a>[tuple](https://infra.spec.whatwg.org/#tuple) (<var>first token</var>, <var>second token</var>).

#### <a id="parsing-mapping-list"></a>5.6.2.  Rules for parsing a list of part mappings[](#parsing-mapping-list)

A <a id="valid-list-of-part-mappings"></a><strong>valid list of part mappings</strong> is a number of valid part mappings separated by a U+002C COMMA character and any number of space characters before or after the U+002C COMMA

The rules for parsing a list of part mappings are as follow:

1.  Let <var>input</var> be the string being parsed.

2.  <a id="ref-for-split-on-commas"></a>[Split the string <var>input</var> on commas](https://infra.spec.whatwg.org/#split-on-commas). Let <var>unparsed mappings</var> be the resulting list of strings.

3.  Let <var>mappings</var> be an initially empty <a id="ref-for-list①"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-tuple④"></a>[tuples](https://infra.spec.whatwg.org/#tuple) of tokens. This <a id="ref-for-list②"></a>list will be the result of this algorithm.

4.  For each string <var>unparsed mapping</var> in <var>unparsed mappings</var>, run the following substeps:

    1.  If <var>unparsed mapping</var> is empty or contains only space characters, continue to the next iteration of the loop.

    2.  Let <var>mapping</var> be the result of parsing <var>unparsed mapping</var> using the rules for parsing part mappings.

    3.  If <var>mapping</var> is an error then continue to the next iteration of the loop. This allows clients to skip over new syntax that is not understood.

    4.  Append <var>mapping</var> to <var>mappings</var>.

## <a id="changes"></a>6. Changes[](#changes)

The following significant changes were made since the [3 April 2014 Working Draft](https://www.w3.org/TR/2014/WD-css-scoping-1-20140403/).

- Defined that tree-scoped names inherit into descendant shadow trees

- Renamed ::content to ::slotted.

- Define the flattened tree.

- Generally reorg and rebase the Shadow DOM section on top of current DOM.

- Punt &#64;scope and related things, and ::region and related things, to the next level of the draft.

- Define the specificity of <a id="ref-for-selectordef-host⑧"></a>[:host](#selectordef-host), <a id="ref-for-selectordef-host-function④"></a>[:host()](#selectordef-host-function), <a id="ref-for-selectordef-host-context④"></a>[:host-context()](#selectordef-host-context), and <a id="ref-for-selectordef-slotted①①"></a>[::slotted()](#selectordef-slotted)

- Remove the \>\>\> (previously called /deep/) combinator.

- Define that tree-abiding pseudos are allowed after <a id="ref-for-selectordef-slotted①②"></a>[::slotted()](#selectordef-slotted).

- Allow <a id="ref-for-typedef-compound-selector-list"></a>[\<compound-selector-list\>](https://drafts.csswg.org/selectors-4/#typedef-compound-selector-list) in all the pseudos.

- Define a way to create a stylesheet of default element styles for a given element.

- Make featureless elements match nothing.

- Define <a id="ref-for-in-the-context-of-a-shadow-tree⑤"></a>[in the context of a shadow tree](#in-the-context-of-a-shadow-tree).

- Merged [\[css-shadow-parts\]](#biblio-css-shadow-parts) and renamed the specification to CSS Shadow Module Level 1. ([Issue \#5809](https://github.com/w3c/csswg-drafts/issues/5809#issuecomment-910896765))

- Added support for multiple names in <a id="ref-for-selectordef-part①⓪"></a>[::part()](#selectordef-part)

- Renamed 'part name map' to <a id="ref-for-element-forwarded-part-name-list⑤"></a>[forwarded part name list](#element-forwarded-part-name-list)

- Restructured 'part element list' algorithm

- Moved various <a id="ref-for-selectordef-part①①"></a>[::part()](#selectordef-part) details to <a id="ref-for-fully-styleable③"></a>[fully styleable pseudo-element](https://drafts.csswg.org/css-pseudo-4/#fully-styleable)

- Added Web Platform Tests coverage

- Minor editorial improvements

## <a id="privacy"></a>7. Privacy Considerations[](#privacy)

This specification introduces Shadow DOM and some shadow-piercing capabilities, but this does not introduce any privacy issues —​shadow DOM, as currently specified, is intentionally not a privacy boundary (and the parts of the UA that use shadow DOM and <em>do</em> have a privacy boundary implicitly rely on protections not yet specified, which protect them from the things defined in this specification).

## <a id="security"></a>8. Security Considerations[](#security)

This specification introduces Shadow DOM and some shadow-piercing capabilities, but this does not introduce any security issues —​shadow DOM, as currently specified, is intentionally not a security boundary, merely a convenience for page authors. Exposing <a id="ref-for-concept-shadow-tree④⑧"></a>[shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) to selectors in this way introduces no new security considerations. (And the parts of the UA that use shadow DOM and <em>do</em> have a security boundary implicitly rely on protections not yet specified, which protect them from the things defined in this specification).

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

- [calculate the part element map](#calculate-the-part-element-map), in § 5
- [\[\[defaultElementStylesMap\]\]](#dom-window-defaultelementstylesmap-slot), in § 2
- [exportparts](#element-attrdef-html-global-exportparts), in § 5.3
- [flattened element tree](#flat-tree), in § 4.1
- [flat tree](#flat-tree), in § 4.1
- [forwarded part name list](#element-forwarded-part-name-list), in § 5
- [global](#tree-scoped-name-global), in § 4.2
- [:has-slotted](#selectordef-has-slotted), in § 3.2.5
- [:host](#selectordef-host), in § 3.2.3
- [:host()](#selectordef-host-function), in § 3.2.3
- [:host-context()](#selectordef-host-context), in § 3.2.3
- [in the context of a shadow tree](#in-the-context-of-a-shadow-tree), in § 3.2.1
- [in the context of the shadow tree](#in-the-context-of-a-shadow-tree), in § 3.2.1
- [local](#tree-scoped-name-local), in § 4.2
- [loosely matched](#tree-scoped-name-loosely-matched), in § 4.2
- [loosely-matched](#tree-scoped-name-loosely-matched), in § 4.2
- [::part()](#selectordef-part), in § 5.4
- part
  - [attribute for Element](#dom-element-part), in § 5.5
  - [element-attr for html-global](#element-attrdef-html-global-part), in § 5.2
- [part element map](#shadow-root-part-element-map), in § 5
- [part name list](#element-part-name-list), in § 5
- [::slotted()](#selectordef-slotted), in § 3.2.4
- [strictly matched](#tree-scoped-name-strictly-matched), in § 4.2
- [strictly-matched](#tree-scoped-name-strictly-matched), in § 4.2
- [tree context](#tree-context), in § 3.2.1
- [tree-scoped name](#css-tree-scoped-name), in § 4.2
- [tree-scoped reference](#css-tree-scoped-reference), in § 4.2
- [valid list of part mappings](#valid-list-of-part-mappings), in § 5.6.2
- [valid part mapping](#valid-part-mapping), in § 5.6.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference[](#index-defined-elsewhere)

- \[CSS-ANCHOR-POSITION-1\] defines the following terms:
  - <a id="be2c21cf"></a>anchor-name
  - <a id="2284f85b"></a>anchor-scope
- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="fa19c68e"></a>&#64;keyframes
  - <a id="d984b6fe"></a>animation-name
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="9cd25054"></a>computed value
  - <a id="f45b0220"></a>UA origin
  - <a id="63fe4250"></a>user-agent origin
- \[CSS-CASCADE-6\] defines the following terms:
  - <a id="de6d76c2"></a>cascade
  - <a id="8a768cb6"></a>scoped style rules
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="94d3d62c"></a>&#64;color-profile
  - <a id="03eee7b4"></a>color()
- \[CSS-COUNTER-STYLES-3\] defines the following terms:
  - <a id="da946c0f"></a>&#64;counter-style
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="e7f0dd6c"></a>display
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="92befb0f"></a>&#64;font-feature-values
  - <a id="0a78ad53"></a>&#64;font-palette-values
  - <a id="d65d17df"></a>font-family
  - <a id="465328f2"></a>font-family (for &#64;font-face)
  - <a id="3a21f8b8"></a>font-palette
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="0497de86"></a>&#64;font-face
- \[CSS-LISTS-3\] defines the following terms:
  - <a id="de56fbfe"></a>list-style-type
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="3b733e03"></a>fully styleable pseudo-elements
  - <a id="16a52cab"></a>tree-abiding pseudo-elements
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="762610c7"></a>at-rule
  - <a id="11e722f0"></a>stylesheet
- \[CSS-TYPED-OM-1\] defines the following terms:
  - <a id="4b630c5b"></a>reification
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="bedd9d42"></a>+
  - <a id="9213678e"></a>\<ident\>
- \[CSS-VARIABLES-2\] defines the following terms:
  - <a id="198f1537"></a>custom property
- \[DOM\] defines the following terms:
  - <a id="141bbada"></a>DOMTokenList
  - <a id="296f3551"></a>Element
  - <a id="84f47c16"></a>child
  - <a id="92b2d7da"></a>children
  - <a id="da8b8e0e"></a>descendant
  - <a id="27d9b7ea"></a>element
  - <a id="71908cce"></a>find flattened slottables
  - <a id="2807a08c"></a>find slottables
  - <a id="5d233601"></a>host
  - <a id="cd1cb230"></a>light tree
  - <a id="83ca4a87"></a>local name
  - <a id="c7d8d91b"></a>node tree
  - <a id="31e1cce6"></a>querySelector(selectors)
  - <a id="f7960529"></a>root
  - <a id="58a8f724"></a>shadow host
  - <a id="3fcc582f"></a>shadow root
  - <a id="19f9e9df"></a>shadow tree
  - <a id="8f378588"></a>shadow-including ancestor
  - <a id="5b18ffe6"></a>slot
  - <a id="8e74149c"></a>slottable
  - <a id="ff74a37a"></a>value
- \[HTML\] defines the following terms:
  - <a id="5d7209e9"></a>Window
  - <a id="80e3bb55"></a>assignedNodes(options)
  - <a id="b6fb7a51"></a>custom element
  - <a id="3ad13207"></a>slot
- \[INFRA\] defines the following terms:
  - <a id="53275e46"></a>append
  - <a id="d24334b4"></a>collect a sequence of code points
  - <a id="a326add7"></a>contain
  - <a id="649608b9"></a>list
  - <a id="692595fe"></a>ordered set
  - <a id="b29084c6"></a>split a string on commas
  - <a id="0698d556"></a>string
  - <a id="0e8de730"></a>tuple
- \[SELECTORS-3\] defines the following terms:
  - <a id="53a193f1"></a>\*
- \[SELECTORS-4\] defines the following terms:
  - <a id="2fb9329a"></a>:hover
  - <a id="b4d48921"></a>:is()
  - <a id="1324ff49"></a>:not()
  - <a id="5a223d21"></a>\<compound-selector-list\>
  - <a id="32b9b591"></a>\<compound-selector\>
  - <a id="832ba741"></a>complex selector
  - <a id="24408a11"></a>compound selector
  - <a id="5f4f98ab"></a>descendant combinator
  - <a id="3a1154bf"></a>featureless
  - <a id="afde9b5e"></a>functional pseudo-class
  - <a id="c8f141cb"></a>match a selector against a tree
  - <a id="5e0d3b02"></a>match a selector against an element
  - <a id="82805a4e"></a>originating element
  - <a id="628c6566"></a>pseudo-class
  - <a id="42b00c6e"></a>pseudo-elements
  - <a id="bb2056aa"></a>specificity
  - <a id="32c45571"></a>type selector
- \[WEBIDL\] defines the following terms:
  - <a id="21ecf38f"></a>PutForwards
  - <a id="a5c91173"></a>SameObject

## <a id="references"></a>References[](#references)

### <a id="normative"></a>Normative References[](#normative)

<a id="biblio-css-anchor-position-1"></a><strong>\[CSS-ANCHOR-POSITION-1\]</strong>

Tab Atkins Jr.; Elika Etemad; Ian Kilpatrick. [CSS Anchor Positioning Module Level 1](https://drafts.csswg.org/css-anchor-position-1/). URL: <https://drafts.csswg.org/css-anchor-position-1/>

<a id="biblio-css-animations-1"></a><strong>\[CSS-ANIMATIONS-1\]</strong>

David Baron; et al. [CSS Animations Level 1](https://drafts.csswg.org/css-animations/). URL: <https://drafts.csswg.org/css-animations/>

<a id="biblio-css-cascade-4"></a><strong>\[CSS-CASCADE-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://drafts.csswg.org/css-cascade-4/). URL: <https://drafts.csswg.org/css-cascade-4/>

<a id="biblio-css-cascade-5"></a><strong>\[CSS-CASCADE-5\]</strong>

Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://drafts.csswg.org/css-cascade-5/). URL: <https://drafts.csswg.org/css-cascade-5/>

<a id="biblio-css-cascade-6"></a><strong>\[CSS-CASCADE-6\]</strong>

Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://drafts.csswg.org/css-cascade-6/). URL: <https://drafts.csswg.org/css-cascade-6/>

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

<a id="biblio-css-lists-3"></a><strong>\[CSS-LISTS-3\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://drafts.csswg.org/css-lists-3/). URL: <https://drafts.csswg.org/css-lists-3/>

<a id="biblio-css-pseudo-4"></a><strong>\[CSS-PSEUDO-4\]</strong>

Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://drafts.csswg.org/css-pseudo-4/). URL: <https://drafts.csswg.org/css-pseudo-4/>

<a id="biblio-css-syntax-3"></a><strong>\[CSS-SYNTAX-3\]</strong>

Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/). URL: <https://drafts.csswg.org/css-syntax/>

<a id="biblio-css-values-4"></a><strong>\[CSS-VALUES-4\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: <https://drafts.csswg.org/css-values-4/>

<a id="biblio-css-variables-2"></a><strong>\[CSS-VARIABLES-2\]</strong>

[CSS Custom Properties for Cascading Variables Module Level 2](https://drafts.csswg.org/css-variables-2/). Editor's Draft. URL: <https://drafts.csswg.org/css-variables-2/>

<a id="biblio-dom"></a><strong>\[DOM\]</strong>

Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: <https://dom.spec.whatwg.org/>

<a id="biblio-html"></a><strong>\[HTML\]</strong>

Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: <https://html.spec.whatwg.org/multipage/>

<a id="biblio-infra"></a><strong>\[INFRA\]</strong>

Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: <https://infra.spec.whatwg.org/>

<a id="biblio-rfc2119"></a><strong>\[RFC2119\]</strong>

S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: <https://datatracker.ietf.org/doc/html/rfc2119>

<a id="biblio-selectors-4"></a><strong>\[SELECTORS-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://drafts.csswg.org/selectors/). URL: <https://drafts.csswg.org/selectors/>

<a id="biblio-webidl"></a><strong>\[WEBIDL\]</strong>

Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: <https://webidl.spec.whatwg.org/>

### <a id="informative"></a>Non-Normative References[](#informative)

<a id="biblio-css-shadow-parts"></a><strong>\[CSS-SHADOW-PARTS\]</strong>

Tab Atkins Jr.; Fergal Daly. [CSS Shadow Parts Module Level 1](https://drafts.csswg.org/css-shadow-parts/). URL: <https://drafts.csswg.org/css-shadow-parts/>

<a id="biblio-css-typed-om-1"></a><strong>\[CSS-TYPED-OM-1\]</strong>

Tab Atkins Jr.; François Remy. [CSS Typed OM Level 1](https://drafts.css-houdini.org/css-typed-om-1/). URL: <https://drafts.css-houdini.org/css-typed-om-1/>

<a id="biblio-selectors-3"></a><strong>\[SELECTORS-3\]</strong>

Tantek Çelik; et al. [Selectors Level 3](https://drafts.csswg.org/selectors-3/). URL: <https://drafts.csswg.org/selectors-3/>

## <a id="idl-index"></a>IDL Index[](#idl-index)

``` text
partial interface Element {
  [SameObject, PutForwards=value] readonly attribute DOMTokenList part;
};
```

## <a id="issues-index"></a>Issues Index[](#issues-index)

Do we need to restrict the

<strong>Issue:</strong>

[at-rules](https://drafts.csswg.org/css-syntax-3/#at-rule) that can be used in these sheets? For example, do we allow an [&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule)? I’m going to leave it as allowed unless/until I hear complaints. [↵](#issue-f4e1dc6b)

Rewrite this against the newer call forms of the matching algorithms.

<strong>Issue:</strong>

[↵](#issue-b8d42d77)

A non-obvious result of this is that elements assigned to a slot inherit from that slot, not their light-tree parent or any deeper slots their slot gets assigned to. This means that text nodes are styled by the shadow tree of their parent, with nobody else capable of intervening in any way. Do we want an additional pseudo-element for targeting those text nodes so they can be styled at all slot-assignment levels, like normal elements can be? This implies it needs to work for text nodes in the light tree before they’re assigned downwards, so this can’t just be a

<strong>Issue:</strong>

[::slotted()](#selectordef-slotted) variant. Luckily, this is a long-standing request! [↵](#issue-75772618)

TODO: Fix all the at-rules that define global names, and the properties that reference them, to use these concepts.

<strong>Issue:</strong>

Global names include:

- [&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule), referenced by [font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family)

- [&#64;font-feature-values](https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values), referenced by [font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family)

- [&#64;keyframes](https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes), referenced by [animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name)

- [&#64;counter-style](https://drafts.csswg.org/css-counter-styles-3/#at-ruledef-counter-style), referenced by [list-style-type](https://drafts.csswg.org/css-lists-3/#propdef-list-style-type)

- [&#64;color-profile](https://drafts.csswg.org/css-color-5/#at-ruledef-profile), referenced by the [color()](https://drafts.csswg.org/css-color-5/#funcdef-color) function

- [&#64;font-palette-values](https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-palette-values), referenced by [font-palette](https://drafts.csswg.org/css-fonts-4/#propdef-font-palette)

- others?

[↵](#issue-3293692a)

Define this as a superglobal in the DOM spec.

<strong>Issue:</strong>

[\[w3c/csswg-drafts Issue \#3424\]](https://github.com/w3c/csswg-drafts/issues/3424) [↵](#issue-853df044)
