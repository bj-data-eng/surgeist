Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Shadow Module Level 1](https://raw.githubusercontent.com/w3c/csswg-drafts/d2ed7a98bb7e499f3b04e6688941bb1a9823d396/css-shadow-1/Overview.bs).

The selected CSSWG repository licenses this document by its contributors under the [W3C Software and Document License](../licenses/w3c/software-license-2023.txt); its [exact repository license declaration](../licenses/w3c/csswg-drafts/LICENSE.md) is retained. No source copyright year is supplied by that declaration.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Shadow Module Level 1

Source snapshot: https://raw.githubusercontent.com/w3c/csswg-drafts/d2ed7a98bb7e499f3b04e6688941bb1a9823d396/css-shadow-1/Overview.bs

Pinned source SHA-256: 0879121699895ad7670985665e6ee70bccd96c2a42e26cf99ca7fd56c44816e7

Generated intermediate HTML SHA-256: 1127eac36d371494872e86447cec44bda538e0eba08794694c647d328ac66a3d

Representation notes:
- Generated on 2026-10-03 from the exact pinned Bikeshed source with Bikeshed 7.1.3, then converted to Markdown. This is a generated rendering of that source, not an official publication or a captured historical rendering.
- The compiler used its bundled support-data manifest dated 2026-09-14 without updating it. External automatic link targets and generated bibliography descriptions come from that data; they do not establish historical versions of those external documents.
- Source headings, explicit anchors, normative prose, examples, metadata, and property-definition fields are retained. Compiler-inserted default property rows are omitted. Generated section numbers, cross-reference labels, and formatting are non-normative.
- 18 source test-reference lists are preserved as recorded, including hidden lists. Current test-index presence, links, and results are not asserted.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

<a id="ref-for-concept-shadow-tree④④"></a>

# CSS Shadow Module Level 1

## <a id="source-metadata"></a>Source metadata

Metadata copied from the pinned source. Editor’s Draft status and work status are those of the source, not a claim of publication or present-day status.



| Field            | Source value                                                                            |
|------------------|-----------------------------------------------------------------------------------------|
| Level            | 1                                                                                       |
| Shortname        | css-shadow                                                                              |
| Group            | CSSWG                                                                                   |
| Status           | ED                                                                                      |
| Work Status      | Exploring                                                                               |
| ED               | https://drafts.csswg.org/css-shadow-1/                                                  |
| Previous Version | https://www.w3.org/TR/2014/WD-css-scoping-1-20140403/                                   |
| Editor           | Tab Atkins Jr., Google, http://xanthir.com/contact/, w3cid 42199                        |
| Editor           | Fergal Daly, Google, fergal@chromium.org, w3cid 106713                                  |
| Abstract         | This module defines ways for CSS to interact with Shadow DOM and its scoping mechanism. |
| Ignored Terms    | inherit, slot, custom elements, stylesheets                                             |
| Ignored Vars     | root elements                                                                           |
| WPT Path Prefix  | css/css-scoping/                                                                        |
| WPT Display      | closed                                                                                  |



## <a id="intro"></a>1. Introduction

Shadow DOM allows authors to separate their page into "components", subtrees of markup whose details are only relevant to the component itself, not the outside page. This reduces the chance of a style meant for one part of the page accidentally over-applying and making a different part of the page look wrong. However, this styling barrier also makes it harder for a page to interact with its components when it actually <em>wants</em> to do so.

<a id="ref-for-selectordef-part"></a>

<a id="ref-for-concept-shadow-tree"></a>

<a id="ref-for-custom-property"></a>

This specification defines the [::part()](#selectordef-part) pseudo-element, which allows an author to style specific, purposely exposed elements in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) from the outside page’s context. In combination with [custom properties](https://drafts.csswg.org/css-variables-2/#custom-property), which let the outside page pass particular values (such as theme colors) into the component for it to do with as it will, these pseudo-elements allow components and the outside page to interact in safe, powerful ways, maintaining encapsulation without surrendering all control.

<a id="ref-for-selectordef-host"></a>

<a id="ref-for-functional-pseudo-class"></a>

<a id="ref-for-selectordef-host-function"></a>

<a id="ref-for-concept-shadow-tree①"></a>

<a id="ref-for-element-shadow-host"></a>

The [:host](#selectordef-host) pseudo-class and its [functional counterpart](https://drafts.csswg.org/selectors-4/#functional-pseudo-class) [:host()](#selectordef-host-function) match the [shadow tree’s](https://dom.spec.whatwg.org/#concept-shadow-tree) [shadow host](https://dom.spec.whatwg.org/#element-shadow-host).

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-selectordef-has-slotted"></a>

<a id="ref-for-pseudo-class"></a>

<a id="ref-for-concept-slot"></a>

<a id="ref-for-concept-shadow-tree②"></a>

Furthermore, the ::slotted [pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) and the related [:has-slotted](#selectordef-has-slotted) [pseudo-class](https://drafts.csswg.org/selectors-4/#pseudo-class) provide ways to interact with [slots](https://dom.spec.whatwg.org/#concept-slot) and their assigned [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree).

<strong>Source test references: General tests for shadow parts</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-shadow-parts/all-hosts.html`
- `css/css-shadow-parts/animation-part.html`
- `css/css-shadow-parts/chaining-invalid-selector.html`
- `css/css-shadow-parts/complex-matching.html`
- `css/css-shadow-parts/complex-non-matching.html`
- `css/css-shadow-parts/different-host.html`
- `css/css-shadow-parts/double-forward.html`
- `css/css-shadow-parts/grouping-with-checked.html`
- `css/css-shadow-parts/grouping-with-disabled.html`
- `css/css-shadow-parts/host-stylesheet.html`
- `css/css-shadow-parts/inner-host.html`
- `css/css-shadow-parts/interaction-with-nested-pseudo-class.html`
- `css/css-shadow-parts/interaction-with-placeholder.html`
- `css/css-shadow-parts/interaction-with-pseudo-elements.html`
- `css/css-shadow-parts/invalidation-complex-selector-forward.html`
- `css/css-shadow-parts/invalidation-complex-selector.html`
- `css/css-shadow-parts/invalidation-part-pseudo.html`
- `css/css-shadow-parts/multiple-parts.html`
- `css/css-shadow-parts/part-after-combinator-invalidation.html`
- `css/css-shadow-parts/part-mutation-pseudo.html`
- `css/css-shadow-parts/part-nested-pseudo.html`
- `css/css-shadow-parts/precedence-part-vs-part.html`
- `css/css-shadow-parts/pseudo-classes-after-part.html`
- `css/css-shadow-parts/pseudo-elements-after-part.html`
- `css/css-shadow-parts/serialization.html`
- `css/css-shadow-parts/simple-forward-shorthand.html`
- `css/css-shadow-parts/simple-forward.html`
- `css/css-shadow-parts/simple-important.html`
- `css/css-shadow-parts/simple-important-important.html`
- `css/css-shadow-parts/simple-important-inline.html`
- `css/css-shadow-parts/simple.html`
- `css/css-shadow-parts/simple-important.html`
- `css/css-shadow-parts/simple-inline.html`
- `css/css-shadow-parts/style-sharing.html`

<a id="ref-for-scoped-style-rules"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification was originally titled CSS Scoping Module Level 1. It has since been merged with CSS Shadow Parts Module Level 1 and renamed to CSS Shadow Module Level 1 to better reflect its comprehensive coverage of Shadow DOM features. The [scoped style rules](https://drafts.csswg.org/css-cascade-6/#scoped-style-rules) feature initially described in this module has been moved to [\[css-cascade-6\]](#biblio-css-cascade-6).

## <a id="default-element-styles"></a>2. Default Styles for Custom Elements

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> This section is <strong>experimental</strong>,&#xA;&#x9;and is under active discussion.&#xA;&#x9;Do not implement without consulting the CSSWG.</strong>

<a id="ref-for-custom-element"></a>

<a id="ref-for-concept-shadow-tree③"></a>

<a id="ref-for-type-selector"></a>

When defining [custom elements](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element), one often wants to set up "default" styles for them, akin to the user-agent styles that apply to built-in elements. This is, unfortunately, hard to do in vanilla CSS, due to issues of scoping and specificity—​the element in question might be used in [shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree), and thus is unreachable by any selector targeting it in the outermost document; and selectors, even low-specificity ones like simple [type selectors](https://drafts.csswg.org/selectors-4/#type-selector), can accidentally override author-level styles meant to target the element.

<a id="ref-for-concept-shadow-tree④"></a>

<a id="ref-for-cascade-origin-ua"></a>

To aid in this, this section defines a way to create a stylesheet of "default element styles" for a given element. This stylesheet applies across the entire document, in all [shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree), and the rules in it apply at the [user-agent origin](https://drafts.csswg.org/css-cascade-5/#cascade-origin-ua), so author-level rules automatically win.

<a id="ref-for-window"></a>

<a id="ref-for-concept-element-local-name"></a>

<a id="ref-for-css-stylesheet"></a>

<code><a href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#window">Window</a></code>s gain a private slot <a id="dom-window-defaultelementstylesmap-slot"></a>`[[defaultElementStylesMap]]` which is a map of [local names](https://dom.spec.whatwg.org/#concept-element-local-name) to [stylesheets](https://drafts.csswg.org/css-syntax-3/#css-stylesheet).

These stylesheets must apply to every document in the window. They must be interpreted as user agent stylesheets.

<a id="ref-for-concept-shadow-tree⑤"></a>

<a id="ref-for-cascade-origin-ua①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This implies, in particular, that they apply to all [shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) in every document, and that the declarations in them are from the [user-agent origin](https://drafts.csswg.org/css-cascade-5/#cascade-origin-ua).

<a id="ref-for-cascade"></a>

For the purpose of the [cascade](https://drafts.csswg.org/css-cascade-6/#cascade), these stylesheets are ordered after the user agent’s own stylesheets; their relative ordering doesn’t matter as it is not observable.

<a id="ref-for-complex"></a>

<a id="ref-for-compound"></a>

<a id="ref-for-type-selector①"></a>

<a id="ref-for-concept-element-local-name①"></a>

Within these stylesheets, [complex selectors](https://drafts.csswg.org/selectors-4/#complex) must be treated as invalid. Every [compound selector](https://drafts.csswg.org/selectors-4/#compound) must be treated as containing an additional [type selector](https://drafts.csswg.org/selectors-4/#type-selector) that selects elements with the [local name](https://dom.spec.whatwg.org/#concept-element-local-name) that the stylesheet is keyed with.

<a id="ref-for-at-rule"></a>

<a id="ref-for-at-font-face-rule"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f4e1dc6b"></a> Do we need to restrict the [at-rules](https://drafts.csswg.org/css-syntax-3/#at-rule) that can be used in these sheets? For example, do we allow an [@font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule)? I’m going to leave it as allowed unless/until I hear complaints.

<a id="ref-for-dom-window-defaultelementstylesmap-slot"></a>

This specification does not define how to add to, remove from, or generally manipulate the <code><a href="#dom-window-defaultelementstylesmap-slot">&#x5B;&#x5B;defaultElementStylesMap&#x5D;&#x5D;</a></code>. It is expected that other specifications, such as [\[DOM\]](#biblio-dom), will define ways to do so.

## <a id="shadow-dom"></a>3. Shadow Encapsulation

### <a id="shadow-gloss"></a>3.1. Informative Explanation of Shadow DOM

<em>The following is a non-normative explanation&#xA;&#x9;of several concepts normatively defined in the DOM Standard <a href="#biblio-dom" title="DOM Standard">&#x5B;DOM&#x5D;</a>,&#xA;&#x9;to aid in understanding what this spec defines&#xA;&#x9;without having to fully grok the DOM Standard.</em>

<a id="ref-for-concept-shadow-tree⑥"></a>

In addition to the qualities of an element tree defined in [Selectors Level 4 § data-model](https://drafts.csswg.org/selectors/#data-model), the DOM Standard adds several new concepts related to [shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree), several of which are relevant to CSS.

<a id="ref-for-concept-shadow-tree⑦"></a>

<a id="ref-for-concept-shadow-root"></a>

<a id="ref-for-concept-documentfragment-host"></a>

<a id="ref-for-element-shadow-host①"></a>

An element can host a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), which is a special kind of document fragment with a [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) (a non-element node) at its root. Children of the <a id="ref-for-concept-shadow-root①"></a>shadow root are ordinary elements and other nodes. The element hosting the <a id="ref-for-concept-shadow-tree⑧"></a>shadow tree is its [host](https://dom.spec.whatwg.org/#concept-documentfragment-host), or [shadow host](https://dom.spec.whatwg.org/#element-shadow-host).

<a id="ref-for-concept-shadow-tree⑨"></a>

<a id="ref-for-concept-tree-descendant"></a>

<a id="ref-for-element-shadow-host②"></a>

<a id="ref-for-descendant-combinator"></a>

<a id="ref-for-flat-tree"></a>

The elements in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) are not [descendants](https://dom.spec.whatwg.org/#concept-tree-descendant) of the [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) in general (including for the purposes of Selectors like the [descendant combinator](https://drafts.csswg.org/selectors-4/#descendant-combinator)). However, the <a id="ref-for-concept-shadow-tree①⓪"></a>shadow tree, when it exists, is used in the construction of the [flattened element tree](#flat-tree), which CSS uses for all purposes <em>after</em> Selectors (including inheritance and box construction).

<a id="ref-for-concept-shadow-tree①①"></a>

<a id="ref-for-element-shadow-host③"></a>

<a id="ref-for-concept-light-tree"></a>

<a id="ref-for-concept-slot①"></a>

<a id="ref-for-the-slot-element"></a>

Loosely, the [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) is treated as the [shadow host’s](https://dom.spec.whatwg.org/#element-shadow-host) contents instead of its normal [light tree](https://dom.spec.whatwg.org/#concept-light-tree) contents. However, some of its <a id="ref-for-concept-light-tree①"></a>light tree children can be "pulled into" the <a id="ref-for-concept-shadow-tree①②"></a>shadow tree by assigning them to [slots](https://dom.spec.whatwg.org/#concept-slot). This causes them to be treated as children of the <a id="ref-for-concept-slot②"></a>slot for CSS purposes. The <a id="ref-for-concept-slot③"></a>slots can then be assigned to <a id="ref-for-concept-slot④"></a>slots in deeper <a id="ref-for-concept-shadow-tree①③"></a>shadow trees; luckily, <a id="ref-for-concept-slot⑤"></a>slots themselves don’t generate boxes by default, so you don’t get an unpredictable cascade of <code><a href="https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element">slot</a></code> wrapper elements disrupting your CSS.

<a id="ref-for-concept-slot⑥"></a>

If nothing is explicitly assigned to a [slot](https://dom.spec.whatwg.org/#concept-slot), the <a id="ref-for-concept-slot⑦"></a>slot’s own children are instead assigned to it, as a sort of "default" contents.

### <a id="selectors"></a>3.2. Shadow DOM and Selectors

#### <a id="selectors-data-model"></a>3.2.1.  Matching Selectors Against Shadow Trees

<a id="ref-for-concept-shadow-tree①④"></a>

<a id="ref-for-element-shadow-host④"></a>

<a id="ref-for-concept-shadow-root②"></a>

When a selector is matched against a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), the selector match list is initially the [shadow host](https://dom.spec.whatwg.org/#element-shadow-host), followed by all children of the <a id="ref-for-concept-shadow-tree①⑤"></a>shadow tree’s [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) and their descendants, ordered by a pre-order traversal.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-b8d42d77"></a> Rewrite this against the newer call forms of the matching algorithms.

<a id="ref-for-concept-tree-descendant①"></a>

<a id="ref-for-concept-light-tree②"></a>

<a id="ref-for-concept-tree-child"></a>

<a id="ref-for-concept-shadow-tree①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Remember that the [descendants](https://dom.spec.whatwg.org/#concept-tree-descendant) of an element are based on the [light tree](https://dom.spec.whatwg.org/#concept-light-tree) [children](https://dom.spec.whatwg.org/#concept-tree-child) of the element, which does not include the [shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) of the element.

<a id="ref-for-match-a-selector-against-a-tree"></a>

<a id="ref-for-concept-tree-root"></a>

<a id="ref-for-tree-context"></a>

<a id="ref-for-concept-shadow-root③"></a>

When a selector is [matched against a tree](https://drafts.csswg.org/selectors-4/#match-a-selector-against-a-tree), its <a id="tree-context"></a>tree context is the [root](https://dom.spec.whatwg.org/#concept-tree-root) of the <var>root elements</var> passed to the algorithm. If the [tree context](#tree-context) is a [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root), that selector is being matched <a id="in-the-context-of-a-shadow-tree"></a>in the context of a shadow tree.

<a id="ref-for-concept-shadow-tree①⑦"></a>

<a id="ref-for-in-the-context-of-a-shadow-tree"></a>

<a id="ref-for-dom-parentnode-queryselector"></a>

<a id="ref-for-concept-shadow-root④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4f60723d"></a> For example, any selector in a stylesheet embedded in or linked from an element in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) is [in the context of a shadow tree](#in-the-context-of-a-shadow-tree). So is the argument to <code><a href="https://dom.spec.whatwg.org/#dom-parentnode-queryselector">querySelector()</a></code> when called from a [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root).

<a id="ref-for-tree-context①"></a>

Declarations inherit the [tree context](#tree-context) of the selector that was matched to apply them.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/css-scoping-shadow-assigned-node-with-before-after.html`
- `css/css-scoping/css-scoping-shadow-dynamic-remove-style-detached.html`
- `css/css-scoping/css-scoping-shadow-assigned-node-with-rules.html`
- `css/css-scoping/css-scoping-shadow-host-with-before-after.html`
- `css/css-scoping/css-scoping-shadow-invisible-slot.html`
- `css/css-scoping/css-scoping-shadow-root-hides-children.html`
- `css/css-scoping/css-scoping-shadow-slot-display-override.html`
- `css/css-scoping/css-scoping-shadow-slot-fallback.html`
- `css/css-scoping/css-scoping-shadow-slot.html`
- `css/css-scoping/css-scoping-shadow-slot-style.html`
- `css/css-scoping/css-scoping-shadow-with-outside-rules.html`
- `css/css-scoping/css-scoping-shadow-with-rules.html`
- `css/css-scoping/css-scoping-shadow-with-rules-no-style-leak.html`
- `css/css-scoping/keyframes-001.html`
- `css/css-scoping/keyframes-002.html`
- `css/css-scoping/shadow-assign-dynamic-001.html`
- `css/css-scoping/shadow-disabled-sheet-001.html`
- `css/css-scoping/shadow-fallback-dynamic-001.html`
- `css/css-scoping/shadow-fallback-dynamic-002.html`
- `css/css-scoping/shadow-fallback-dynamic-003.html`
- `css/css-scoping/shadow-fallback-dynamic-004.html`
- `css/css-scoping/shadow-fallback-dynamic-005.html`
- `css/css-scoping/shadow-host-with-before-after.html`
- `css/css-scoping/shadow-link-rel-stylesheet-no-style-leak.html`
- `css/css-scoping/shadow-multiple-links.html`
- `css/css-scoping/shadow-reassign-dynamic-001.html`
- `css/css-scoping/shadow-reassign-dynamic-002.html`
- `css/css-scoping/shadow-reassign-dynamic-004.html`
- `css/css-scoping/shadow-reassign-dynamic-005-crash.html`
- `css/css-scoping/shadow-reassign-dynamic-006.html`
- `css/css-scoping/shadow-root-insert-into-document.html`
- `css/css-scoping/slotted-text-with-flex.html`
- `css/css-scoping/whitespace-crash-001.html`

#### <a id="host-element-in-tree"></a>3.2.2.  Selecting Shadow Hosts from within a Shadow Tree

<a id="ref-for-element-shadow-host⑤"></a>

<a id="ref-for-concept-shadow-tree①⑧"></a>

<a id="ref-for-in-the-context-of-a-shadow-tree①"></a>

A [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) is outside of the [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) it hosts, and so would ordinarily be untargettable by any selectors evaluated [in the context of the shadow tree](#in-the-context-of-a-shadow-tree) (as selectors are limited to a single tree), but it is sometimes useful to be able to style it from inside the <a id="ref-for-concept-shadow-tree①⑨"></a>shadow tree context.

<a id="ref-for-element-shadow-host⑥"></a>

<a id="ref-for-concept-shadow-tree②⓪"></a>

<a id="ref-for-concept-shadow-root⑤"></a>

For the purpose of Selectors, a [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) also appears in its [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), with the contents of the <a id="ref-for-concept-shadow-tree②①"></a>shadow tree treated as its children. (In other words, the <a id="ref-for-element-shadow-host⑦"></a>shadow host is treated as replacing the [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) node.)

<a id="ref-for-concept-shadow-tree②②"></a>

<a id="ref-for-element-shadow-host⑧"></a>

<a id="ref-for-featureless"></a>

<a id="ref-for-selectordef-host①"></a>

<a id="ref-for-selectordef-host-function①"></a>

<a id="ref-for-selectordef-host-context"></a>

When considered within its own [shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree), the [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) is [featureless](https://drafts.csswg.org/selectors-4/#featureless). Only the [:host](#selectordef-host), [:host()](#selectordef-host-function), and [:host-context()](#selectordef-host-context) pseudo-classes are allowed to match it.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/host-descendant-003.html`
- `css/css-scoping/host-is-006.html`
- `css/css-scoping/host-is-featureless.html`
- `css/css-scoping/host-multiple-002.html`
- `css/css-scoping/host-multiple-003.html`
- `css/css-scoping/host-multiple-004.html`
- `css/css-scoping/host-multiple-005.html`
- `css/css-scoping/host-not-001.html`

> <strong data-conversion-semantic="note">Note</strong>
>
> Why is the shadow host so weird?
>
> <a id="ref-for-element-shadow-host⑨"></a>
>
> <a id="ref-for-concept-shadow-tree②③"></a>
>
> The [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) lives outside the [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), and its markup is in control of the page author, not the component author.
>
> <a id="ref-for-concept-shadow-tree②④"></a>
>
> <a id="ref-for-element-shadow-host①⓪"></a>
>
> It would not be very good if a component used a particular class name internally in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) stylesheet, and the page author using the component accidentally <em>also</em> used the same class name and put it on the [shadow host](https://dom.spec.whatwg.org/#element-shadow-host). Such a situation would result in accidental styling that is impossible for the component author to predict, and confusing for the page author to debug.
>
> <a id="ref-for-concept-shadow-tree②⑤"></a>
>
> <a id="ref-for-element-shadow-host①①"></a>
>
> <a id="ref-for-propdef-display"></a>
>
> <a id="ref-for-selectordef-host②"></a>
>
> However, there are still some reasonable use-cases for letting a stylesheet in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) style its [shadow host](https://dom.spec.whatwg.org/#element-shadow-host). (For example, the component might want to be laid out as a flexbox, requiring the <a id="ref-for-element-shadow-host①②"></a>shadow host to be set to [display: flex](https://drafts.csswg.org/css-display-4/#propdef-display).) So, to allow this situation but prevent accidental styling, the <a id="ref-for-element-shadow-host①③"></a>shadow host appears but is completely featureless and unselectable except through [:host](#selectordef-host) and its related functional forms, which make it very explicit when you’re trying to match against markup provided by the page author.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/host-with-default-namespace-001.html`
- `css/css-scoping/host-context-parsing.html`
- `css/css-scoping/host-defined.html`
- `css/css-scoping/host-is-001.html`
- `css/css-scoping/host-is-002.html`
- `css/css-scoping/host-is-003.html`
- `css/css-scoping/host-is-004.html`
- `css/css-scoping/host-is-005.html`
- `css/css-scoping/host-parsing.html`
- `css/css-scoping/scope-pseudo-in-shadow.html`

<a id="ref-for-selectordef-host③"></a>

<a id="ref-for-selectordef-host-function②"></a>

<a id="ref-for-selectordef-host-context①"></a>

#### <a id="host-selector"></a>3.2.3.  Selecting Into the Light: the [:host](#selectordef-host), [:host()](#selectordef-host-function), and [:host-context()](#selectordef-host-context) pseudo-classes

<a id="ref-for-in-the-context-of-a-shadow-tree②"></a>

<a id="ref-for-concept-shadow-tree②⑥"></a>

<a id="ref-for-element-shadow-host①④"></a>

The <a id="selectordef-host"></a>:host pseudo-class, when evaluated [in the context of a shadow tree](#in-the-context-of-a-shadow-tree), matches the [shadow tree’s](https://dom.spec.whatwg.org/#concept-shadow-tree) [shadow host](https://dom.spec.whatwg.org/#element-shadow-host). In any other context, it matches nothing.

The <a id="selectordef-host-function"></a>:host() function pseudo-class has the syntax:

<a id="ref-for-typedef-compound-selector"></a>

```text
:host( <compound-selector> )
```
<a id="ref-for-in-the-context-of-a-shadow-tree③"></a>

<a id="ref-for-concept-shadow-tree②⑦"></a>

<a id="ref-for-element-shadow-host①⑤"></a>

When evaluated [in the context of a shadow tree](#in-the-context-of-a-shadow-tree), it matches the [shadow tree’s](https://dom.spec.whatwg.org/#concept-shadow-tree) [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) if the <a id="ref-for-element-shadow-host①⑥"></a>shadow host, in its normal context, matches the selector argument. In any other context, it matches nothing.

<a id="ref-for-specificity"></a>

<a id="ref-for-selectordef-host④"></a>

<a id="ref-for-selectordef-host-function③"></a>

The [specificity](https://drafts.csswg.org/selectors-4/#specificity) of [:host](#selectordef-host) is that of a pseudo-class. The <a id="ref-for-specificity①"></a>specificity of [:host()](#selectordef-host-function) is that of a pseudo-class, plus the <a id="ref-for-specificity②"></a>specificity of its argument.

<a id="ref-for-matches-pseudo"></a>

<a id="ref-for-negation-pseudo"></a>

<a id="ref-for-selectordef-host⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is different from the specificity of similar pseudo-classes, like [:is()](https://drafts.csswg.org/selectors-4/#matches-pseudo) or [:not()](https://drafts.csswg.org/selectors-4/#negation-pseudo), which <em>only</em> take the specificity of their argument. This is because [:host](#selectordef-host) is affirmatively selecting an element all by itself, like a "normal" pseudo-class; it takes a selector argument for syntactic reasons (we can’t say that :host.foo matches but .foo doesn’t), but is otherwise identical to just using <a id="ref-for-selectordef-host⑥"></a>:host followed by a selector.

<a id="ref-for-concept-shadow-tree②⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-20a3843d"></a> For example, say you had a component with a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) like the following:
>
> ```text
> <x-foo class="foo">
>   <"shadow tree">
>     <div class="foo">...</div>
>   </>
> </x-foo>
> ```
>
> <a id="ref-for-concept-shadow-tree②⑨"></a>
>
> For a stylesheet within the [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree):
>
> - <a id="ref-for-selectordef-host⑦"></a>
>
>   [:host](#selectordef-host) matches the `<x-foo>` element.
>
> - x-foo matches nothing.
>
> - .foo matches only the `<div>` element.
>
> - .foo:host matches nothing
>
> - :host(.foo) matches the `<x-foo>` element.

<a id="ref-for-concept-shadow-tree③⓪"></a>

Ordinary, selectors within a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) can’t see elements outside the <a id="ref-for-concept-shadow-tree③①"></a>shadow tree at all. Sometimes, however, it’s useful to select an ancestor that lies somewhere outside the shadow tree, above it in the document.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-45cde0ef"></a> For example, a group of components can define a handful of color themes they know how to respond to. Page authors could opt into a particular theme by adding a specific class to the components, or higher up in the document.

<a id="ref-for-concept-shadow-tree③②"></a>

The <a id="selectordef-host-context"></a>:host-context() functional pseudo-class tests whether there is an ancestor, outside the [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), which matches a particular selector. Its syntax is:

<a id="ref-for-typedef-compound-selector①"></a>

```text
:host-context( <compound-selector> )
```
<a id="ref-for-in-the-context-of-a-shadow-tree④"></a>

<a id="ref-for-selectordef-host-context②"></a>

<a id="ref-for-element-shadow-host①⑦"></a>

<a id="ref-for-concept-shadow-including-ancestor"></a>

<a id="ref-for-typedef-compound-selector②"></a>

When evaluated [in the context of a shadow tree](#in-the-context-of-a-shadow-tree), the [:host-context()](#selectordef-host-context) pseudo-class matches the [shadow host](https://dom.spec.whatwg.org/#element-shadow-host), if the <a id="ref-for-element-shadow-host①⑧"></a>shadow host or one of its [shadow-including ancestors](https://dom.spec.whatwg.org/#concept-shadow-including-ancestor) matches the provided [\<compound-selector\>](https://drafts.csswg.org/selectors-4/#typedef-compound-selector). In any other context, it matches nothing.

<a id="ref-for-specificity③"></a>

<a id="ref-for-selectordef-host-context③"></a>

The [specificity](https://drafts.csswg.org/selectors-4/#specificity) of [:host-context()](#selectordef-host-context) is that of a pseudo-class, plus the <a id="ref-for-specificity④"></a>specificity of its argument.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that the selector pierces through shadow boundaries on the way up, looking for elements that match its argument, until it reaches the document root.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/css-scoping-shadow-host-functional-rule.html`
- `css/css-scoping/css-scoping-shadow-host-namespace.html`
- `css/css-scoping/css-scoping-shadow-host-rule.html`
- `css/css-scoping/host-context-specificity-001.html`
- `css/css-scoping/host-context-specificity-002.html`
- `css/css-scoping/host-context-specificity-003.html`
- `css/css-scoping/host-descendant-001.html`
- `css/css-scoping/host-descendant-002.html`
- `css/css-scoping/host-descendant-invalidation.html`
- `css/css-scoping/host-dom-001.html`
- `css/css-scoping/host-functional-descendant-invalidation.html`
- `css/css-scoping/host-has-internal-001.html`
- `css/css-scoping/host-has-internal-002.html`
- `css/css-scoping/host-has-internal-003.html`
- `css/css-scoping/host-has-internal-004.html`
- `css/css-scoping/host-in-host-context-selector.html`
- `css/css-scoping/host-in-host-selector.html`
- `css/css-scoping/host-multiple-001.html`
- `css/css-scoping/host-nested-001.html`
- `css/css-scoping/host-slotted-001.html`
- `css/css-scoping/host-specificity-002.html`
- `css/css-scoping/host-specificity-003.html`
- `css/css-scoping/host-specificity.html`
- `css/css-scoping/shadow-host-removal-invalidation.html`
- `css/css-scoping/shadow-at-import.html`
- `css/css-scoping/shadow-host-style-sharing.html`
- `css/css-scoping/shadow-link-rel-stylesheet.html`
- `css/css-scoping/shadow-shared-style-cache-001.html`
- `css/css-scoping/stylesheet-title-001.html`
- `css/css-scoping/stylesheet-title-002.html`

<a id="ref-for-selectordef-slotted"></a>

#### <a id="slotted-pseudo"></a>3.2.4.  Selecting Slot-Assigned Content: the [::slotted()](#selectordef-slotted) pseudo-element

<a id="ref-for-find-flattened-slotables"></a>

<a id="ref-for-concept-slot⑧"></a>

The <a id="selectordef-slotted"></a>::slotted() pseudo-element represents the elements [assigned, after flattening,](https://dom.spec.whatwg.org/#find-flattened-slotables) to a [slot](https://dom.spec.whatwg.org/#concept-slot). This pseudo-element only exists on <a id="ref-for-concept-slot⑨"></a>slots.

<a id="ref-for-selectordef-slotted①"></a>

The [::slotted()](#selectordef-slotted) pseudo-element is an <em>alias</em> for other elements in the tree, and does not generate any boxes itself.

<a id="ref-for-selectordef-slotted②"></a>

The grammar of the [::slotted()](#selectordef-slotted) pseudo-element is:

<a id="ref-for-typedef-compound-selector③"></a>

```text
::slotted( <compound-selector> )
```
<a id="ref-for-selectordef-slotted③"></a>

The [::slotted()](#selectordef-slotted) pseudo-element represents the elements that are:

- <a id="ref-for-find-flattened-slotables①"></a>

  <a id="ref-for-concept-slot①⓪"></a>

  [assigned, after flattening,](https://dom.spec.whatwg.org/#find-flattened-slotables) to the [slot](https://dom.spec.whatwg.org/#concept-slot) that is ::slotted’s originating element

- <a id="ref-for-match-a-selector-against-an-element"></a>

  <a id="ref-for-typedef-compound-selector④"></a>

  [matched](https://drafts.csswg.org/selectors-4/#match-a-selector-against-an-element) by its [\<compound-selector\>](https://drafts.csswg.org/selectors-4/#typedef-compound-selector) argument

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/slotted-has-001.html`
- `css/css-scoping/slotted-has-002.html`
- `css/css-scoping/slotted-has-003.html`
- `css/css-scoping/slotted-has-004.html`

<a id="ref-for-selectordef-slotted④"></a>

<a id="ref-for-tree-abiding"></a>

The [::slotted()](#selectordef-slotted) pseudo-element can be followed by a [tree-abiding pseudo-element](https://drafts.csswg.org/css-pseudo-4/#tree-abiding), like ::slotted()::before, representing the appropriate pseudo-element of the elements represented by the <a id="ref-for-selectordef-slotted⑤"></a>::slotted() pseudo-element.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/slotted-details-content.html`
- `css/css-scoping/slotted-file-selector-button.html`

<a id="ref-for-specificity⑤"></a>

<a id="ref-for-selectordef-slotted⑥"></a>

The [specificity](https://drafts.csswg.org/selectors-4/#specificity) of [::slotted()](#selectordef-slotted) is that of a pseudo-element, plus the <a id="ref-for-specificity⑥"></a>specificity of its argument.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7cc70c2d"></a> For example, say you had a component with both children and a shadow tree, like the following:
>
> ```text
> <x-foo>
>   <div id="one" slot="foo" class="foo">...</div>
>   <div id="two" slot="foo">...</div>
>   <div id="three" class="foo">
>     <div id="four" slot="foo">...</div>
>   </div>
>   <"shadow tree">
>     <div id="five">...</div>
>     <div id="six">...</div>
>     <slot name="foo"></slot>
>   </"shadow tree">
> </x-foo>
> ```
>
> <a id="ref-for-concept-shadow-tree③③"></a>
>
> <a id="ref-for-find-flattened-slotables②"></a>
>
> <a id="ref-for-the-slot-element①"></a>
>
> <a id="ref-for-concept-tree-child①"></a>
>
> <a id="ref-for-element-shadow-host①⑨"></a>
>
> <a id="ref-for-concept-slot①①"></a>
>
> For a stylesheet within the [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), a selector like ::slotted(\*) selects \#one and \#two only, as they’re the elements [assigned](https://dom.spec.whatwg.org/#find-flattened-slotables) to the sole <code><a href="https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element">slot</a></code> element. It will <em>not</em> select \#three (no `slot` attribute) nor \#four (only direct [children](https://dom.spec.whatwg.org/#concept-tree-child) of a [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) can be assigned to a [slot](https://dom.spec.whatwg.org/#concept-slot)).
>
> A selector like ::slotted(.foo), on the other hand, will only select \#one, as it matches .foo, but \#two doesn’t.
>
> <a id="ref-for-x"></a>
>
> <a id="ref-for-the-slot-element②"></a>
>
> <a id="ref-for-the-slot-element③"></a>
>
> <a id="ref-for-concept-slot①②"></a>
>
> <a id="ref-for-selectordef-slotted⑦"></a>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: Note that a selector like ::slotted(\*) is equivalent to \*::slotted(\*), where the [\*](https://drafts.csswg.org/selectors-3/#x) selects many more elements than just the <code><a href="https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element">slot</a></code> element. However, since only the <code><a href="https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element">slot</a></code> elements are [slots](https://dom.spec.whatwg.org/#concept-slot), they’re the only elements with a [::slotted()](#selectordef-slotted) pseudo-element as well.

<a id="ref-for-selectordef-slotted⑧"></a>

<a id="ref-for-concept-slot①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [::slotted()](#selectordef-slotted) can only represent the <em>elements</em> assigned to the [slot](https://dom.spec.whatwg.org/#concept-slot). <a id="ref-for-concept-slot①④"></a>Slots can also be assigned text nodes, which can’t be selected by <a id="ref-for-selectordef-slotted⑨"></a>::slotted(). The only way to style assigned text nodes is by styling the <a id="ref-for-concept-slot①⑤"></a>slot and relying on inheritance.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/css-scoping-shadow-slotted-nested.html`
- `css/css-scoping/css-scoping-shadow-slotted-rule.html`
- `css/css-scoping/reslot-text-inheritance.html`
- `css/css-scoping/slotted-invalidation.html`
- `css/css-scoping/slotted-link.html`
- `css/css-scoping/slotted-matches.html`
- `css/css-scoping/slotted-nested.html`
- `css/css-scoping/slotted-parsing.html`
- `css/css-scoping/slotted-placeholder.html`
- `css/css-scoping/slotted-slot.html`
- `css/css-scoping/slotted-specificity-002.html`
- `css/css-scoping/slotted-specificity.html`
- `css/css-scoping/slotted-with-pseudo-element.html`

<a id="ref-for-selectordef-has-slotted①"></a>

#### <a id="the-has-slotted-pseudo"></a>3.2.5.  Matching on the Presence of Slot-Assigned Nodes: the [:has-slotted](#selectordef-has-slotted) pseudo-class

<a id="ref-for-the-slot-element④"></a>

<a id="ref-for-find-flattened-slotables③"></a>

The <a id="selectordef-has-slotted"></a>:has-slotted pseudo-class matches <code><a href="https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element">slot</a></code> elements which have a non-empty list of [flattened slotted nodes](https://dom.spec.whatwg.org/#find-flattened-slotables).

<a id="ref-for-selectordef-has-slotted②"></a>

When [:has-slotted](#selectordef-has-slotted) matches a slot with fallback content, we can conclude that the fallback content is <em>not</em> being displayed.

<a id="ref-for-selectordef-has-slotted③"></a>

<a id="ref-for-dom-slot-assignednodes"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Even a single whitespace text node is sufficient to make [:has-slotted](#selectordef-has-slotted)' apply. This is by design, so that the behavior of this pseudo-class is consistent with the behavior of the <code><a href="https://html.spec.whatwg.org/multipage/scripting.html#dom-slot-assignednodes">assignedNodes()</a></code> method. A future version of this specification is expected to introduce a way to exclude this case from matching.

<a id="ref-for-selectordef-has-slotted④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is expected that a future version of this specification will introduce a functional :has-slotted() pseudo-class that allows more fine-grained matching by accepting a selector argument. [:has-slotted](#selectordef-has-slotted) is <em>not</em> an alias of :has-slotted(\*), as the latter would not match slotted text nodes, but <a id="ref-for-selectordef-has-slotted⑤"></a>:has-slotted does.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/has-slotted-001.html`
- `css/css-scoping/has-slotted-002.html`
- `css/css-scoping/has-slotted-003.html`
- `css/css-scoping/has-slotted-changing-001.html`
- `css/css-scoping/has-slotted-changing-002.html`
- `css/css-scoping/has-slotted-flattened-001.html`
- `css/css-scoping/has-slotted-flattened-002.html`
- `css/css-scoping/has-slotted-flattened-003.html`
- `css/css-scoping/has-slotted-flattened-004.html`
- `css/css-scoping/has-slotted-manual-assignment.html`
- `css/css-scoping/has-slotted-query-selector.html`

## <a id="shadow-cascading"></a>4. Shadow Trees and the Cascade

See [CSS Cascading 4 § 6.1 Cascade Sorting Order](https://drafts.csswg.org/css-cascade-4/#cascade-sort).

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/shadow-cascade-order-001.html`

### <a id="flattening"></a>4.1.  Flattening the DOM into an Element Tree

<a id="ref-for-concept-tree-child②"></a>

<a id="ref-for-flat-tree①"></a>

While Selectors operates on the DOM tree as the host language presents it, with separate trees that are unreachable via the standard parent/[child](https://dom.spec.whatwg.org/#concept-tree-child) relationship, the rest of CSS needs a single unified tree structure to work with. This is called the <a id="flat-tree"></a>flattened element tree (or [flat tree](#flat-tree)), and is constructed as follows:

1.  <a id="ref-for-concept-tree-root①"></a>

    Let <var>pending nodes</var> be a list of DOM nodes with associated parents, initially containing just the document’s [root](https://dom.spec.whatwg.org/#concept-tree-root) element with no associated parent.

2.  Repeatedly execute the following substeps until <var>pending nodes</var> is empty:

    1.  Pop the first element from <var>pending nodes</var>, and assign it to <var>pending node</var>.

    2.  <a id="ref-for-flat-tree②"></a>

        Insert <var>pending node</var> into the [flat tree](#flat-tree) as a child of its associated parent. (If it has no associated parent, it’s the document root—​just insert it into the <a id="ref-for-flat-tree③"></a>flat tree as its root.)

    3.  Perform one of the following, whichever is the first that matches:

        <a id="ref-for-element-shadow-host②⓪"></a>

        <var>pending node</var> is a [shadow host](https://dom.spec.whatwg.org/#element-shadow-host)

        <a id="ref-for-concept-shadow-tree③④"></a>

        <a id="ref-for-concept-shadow-root⑥"></a>

        Append the child nodes of the [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) of the [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) it hosts to <var>pending nodes</var>, with <var>pending node</var> as their associated parent.

        <a id="ref-for-concept-slot①⑥"></a>

        <var>pending node</var> is a [slot](https://dom.spec.whatwg.org/#concept-slot)

        <a id="ref-for-find-slotables"></a>

        [Find slottables](https://dom.spec.whatwg.org/#find-slotables) for <var>pending node</var>, and append them to <var>pending nodes</var>, with <var>pending node</var> as their associated parent.

        <a id="ref-for-concept-slotable"></a>

        <a id="ref-for-concept-tree-child③"></a>

        If no [slottables](https://dom.spec.whatwg.org/#concept-slotable) were found for <var>pending node</var>, instead append its [children](https://dom.spec.whatwg.org/#concept-tree-child) to <var>pending nodes</var>, with <var>pending node</var> as their associated parent.

        Otherwise,

        <a id="ref-for-concept-light-tree③"></a>

        Append the child nodes of <var>pending node</var>’s [light tree](https://dom.spec.whatwg.org/#concept-light-tree) to <var>pending nodes</var>, with <var>pending node</var> as their associated parent.

<a id="ref-for-flat-tree④"></a>

<a id="ref-for-element-shadow-host②①"></a>

<a id="ref-for-concept-shadow-tree③⑤"></a>

<a id="ref-for-concept-light-tree④"></a>

<a id="ref-for-concept-slot①⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In other words, the [flat tree](#flat-tree) is the top-level DOM tree, but [shadow hosts](https://dom.spec.whatwg.org/#element-shadow-host) are filled with their [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) children instead of their [light tree](https://dom.spec.whatwg.org/#concept-light-tree) children (and this proceeds recursively if the <a id="ref-for-concept-shadow-tree③⑥"></a>shadow tree contains any <a id="ref-for-element-shadow-host②②"></a>shadow hosts), and [slots](https://dom.spec.whatwg.org/#concept-slot) get filled with the nodes that are assigned to them (and this proceeds recursively if the <a id="ref-for-concept-slot①⑧"></a>slots are themselves assigned to a <a id="ref-for-concept-slot①⑨"></a>slot in a deeper <a id="ref-for-concept-shadow-tree③⑦"></a>shadow tree).

<a id="ref-for-selectordef-slotted①⓪"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-75772618"></a> A non-obvious result of this is that elements assigned to a slot inherit from that slot, not their light-tree parent or any deeper slots their slot gets assigned to. This means that text nodes are styled by the shadow tree of their parent, with nobody else capable of intervening in any way. Do we want an additional pseudo-element for targeting those text nodes so they can be styled at all slot-assignment levels, like normal elements can be? This implies it needs to work for text nodes in the light tree before they’re assigned downwards, so this can’t just be a [::slotted()](#selectordef-slotted) variant. Luckily, this is a long-standing request!

#### <a id="slots-in-shadow-tree"></a>4.1.1.  Slots and Slotted Elements in a Shadow Tree

<a id="ref-for-concept-slot②⓪"></a>

<a id="ref-for-propdef-display①"></a>

<a id="ref-for-cascade-origin-ua②"></a>

[Slots](https://dom.spec.whatwg.org/#concept-slot) must act as if they were assigned [display: contents](https://drafts.csswg.org/css-display-4/#propdef-display) via a rule in the [UA origin](https://drafts.csswg.org/css-cascade-5/#cascade-origin-ua). This must be possible to override via <a id="ref-for-propdef-display②"></a>display, so they <em>do</em> generate boxes if desired.

<a id="ref-for-concept-slot②①"></a>

<a id="ref-for-concept-light-tree⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A non-obvious result of assigning elements to [slots](https://dom.spec.whatwg.org/#concept-slot) is that they inherit from the <a id="ref-for-concept-slot②②"></a>slot they’re assigned to. Their original [light tree](https://dom.spec.whatwg.org/#concept-light-tree) parent, and any deeper <a id="ref-for-concept-slot②③"></a>slots that their <a id="ref-for-concept-slot②④"></a>slot gets assigned to, don’t affect inheritance.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/css-scoping-shadow-nested-slot-display-override.html`
- `css/css-scoping/shadow-reassign-dynamic-003.html`
- `css/css-scoping/slot-non-html-display-value.html`

### <a id="shadow-names"></a>4.2.  Name-Defining Constructs and Inheritance

<a id="ref-for-concept-shadow-tree③⑧"></a>

[Shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) are meant to be an encapsulation boundary, allowing independent authors to share code without accidentally polluting each other’s namespaces. For example, element IDs, which are generally meant to be unique within a document, can be validly used multiple times as long as each use is in a different <a id="ref-for-concept-shadow-tree③⑨"></a>shadow tree.

<a id="ref-for-at-rule①"></a>

<a id="ref-for-at-ruledef-keyframes"></a>

<a id="ref-for-at-font-face-rule①"></a>

<a id="ref-for-concept-shadow-tree④⓪"></a>

Similarly, several [at-rules](https://drafts.csswg.org/css-syntax-3/#at-rule) in CSS, such as [@keyframes](https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes) or [@font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule), define a name that later at-rules or properties can refer to them by. Like IDs, these names are globally exposed and unique within a document; also like IDs, this restriction is now loosened to being unique within a given [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree).

However, property inheritance can carry values from one tree to another, which complicates referencing the correct definition of a given name. Done naively, this can produce surprising and confusing results for authors. This section defines a set of concepts to use in defining and referencing "global" names in a way that respects encapsulation and doesn’t give surprising results.

<a id="ref-for-at-font-face-rule②"></a>

<a id="ref-for-descdef-font-face-font-family"></a>

<a id="ref-for-propdef-anchor-scope"></a>

<a id="ref-for-css-tree-scoped-name"></a>

<a id="ref-for-concept-tree-root②"></a>

<a id="ref-for-concept-element"></a>

If an at-rule or property defines a name that other CSS constructs can refer to it by, such as a [@font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule) [font-family](https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-family) name or an [anchor-scope](https://drafts.csswg.org/css-anchor-position-1/#propdef-anchor-scope) name, it must be defined as a <a id="css-tree-scoped-name"></a>tree-scoped name. [Tree-scoped names](#css-tree-scoped-name) are associated with the [root](https://dom.spec.whatwg.org/#concept-tree-root) of the [element](https://dom.spec.whatwg.org/#concept-element) hosting the stylesheet that the at-rule or property is declared in.

<a id="ref-for-css-tree-scoped-name①"></a>

<a id="ref-for-tree-scoped-name-loosely-matched"></a>

<a id="ref-for-css-tree-scoped-reference"></a>

<a id="ref-for-tree-scoped-name-strictly-matched"></a>

Additionally, [tree-scoped names](#css-tree-scoped-name) can be <a id="tree-scoped-name-loosely-matched"></a>loosely matched or <a id="tree-scoped-name-strictly-matched"></a>strictly matched, defaulting to [loosely matched](#tree-scoped-name-loosely-matched) unless otherwise specified. A <a id="ref-for-tree-scoped-name-loosely-matched①"></a>loosely matched <a id="ref-for-css-tree-scoped-name②"></a>tree-scoped name can be matched by [tree-scoped references](#css-tree-scoped-reference) (see below) in the same tree or descendant trees, while a [strictly matched](#tree-scoped-name-strictly-matched) <a id="ref-for-css-tree-scoped-name③"></a>tree-scoped name can only be matched by <a id="ref-for-css-tree-scoped-reference①"></a>tree-scoped references in the exact same tree.

<a id="ref-for-css-tree-scoped-name④"></a>

<a id="ref-for-propdef-font-family"></a>

<a id="ref-for-propdef-animation-name"></a>

<a id="ref-for-concept-node-tree"></a>

<a id="ref-for-concept-tree-root③"></a>

<a id="ref-for-concept-element①"></a>

<a id="ref-for-css-tree-scoped-reference②"></a>

Properties or descriptors that reference a [tree-scoped name](#css-tree-scoped-name), such as the [font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) or [animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name) properties, must define their value as a <a id="css-tree-scoped-reference"></a>tree-scoped reference. These references implicitly capture a [node tree](https://dom.spec.whatwg.org/#concept-node-tree) [root](https://dom.spec.whatwg.org/#concept-tree-root) along with their specified value: unless otherwise specified, the <a id="ref-for-concept-tree-root④"></a>root of the [element](https://dom.spec.whatwg.org/#concept-element) hosting the stylesheet that the property or descriptor is declared in. This <a id="ref-for-concept-tree-root⑤"></a>root reference stays with the [tree-scoped reference](#css-tree-scoped-reference) as it is inherited.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/svg-id-ref-001.html`

<a id="ref-for-css-tree-scoped-name⑤"></a>

<a id="ref-for-at-font-face-rule③"></a>

<a id="ref-for-css-tree-scoped-reference③"></a>

<a id="ref-for-concept-tree-root⑥"></a>

<a id="ref-for-concept-shadow-root⑦"></a>

<a id="ref-for-concept-documentfragment-host①"></a>

<a id="ref-for-concept-node-tree①"></a>

If a [tree-scoped name](#css-tree-scoped-name) is <a id="tree-scoped-name-global"></a>global (such as [@font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule) names), then when a [tree-scoped reference](#css-tree-scoped-reference) is dereferenced to find it, first search only the <a id="ref-for-css-tree-scoped-name⑥"></a>tree-scoped names associated with the same [root](https://dom.spec.whatwg.org/#concept-tree-root) as the <a id="ref-for-css-tree-scoped-reference④"></a>tree-scoped reference. If no relevant <a id="ref-for-css-tree-scoped-name⑦"></a>tree-scoped name is found, and the <a id="ref-for-concept-tree-root⑦"></a>root is a [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root), then repeat this search in the <a id="ref-for-concept-tree-root⑧"></a>root’s [host](https://dom.spec.whatwg.org/#concept-documentfragment-host)’s [node tree](https://dom.spec.whatwg.org/#concept-node-tree) (recursively). (In other words, global <a id="ref-for-css-tree-scoped-name⑧"></a>tree-scoped names “inherit” into descendant shadow trees, so long as they don’t define the same name themselves.)

<a id="ref-for-css-tree-scoped-name⑨"></a>

<a id="ref-for-propdef-anchor-name"></a>

<a id="ref-for-propdef-anchor-scope①"></a>

<a id="ref-for-css-tree-scoped-reference⑤"></a>

<a id="ref-for-tree-scoped-name-strictly-matched①"></a>

<a id="ref-for-tree-scoped-name-loosely-matched②"></a>

If a [tree-scoped name](#css-tree-scoped-name) is <a id="tree-scoped-name-local"></a>local to an element (such as [anchor-name](https://drafts.csswg.org/css-anchor-position-1/#propdef-anchor-name) or [anchor-scope](https://drafts.csswg.org/css-anchor-position-1/#propdef-anchor-scope) values), then whether a [tree-scoped reference](#css-tree-scoped-reference) matches the <a id="ref-for-css-tree-scoped-name①⓪"></a>tree-scoped name on a given element depends on whether the <a id="ref-for-css-tree-scoped-name①①"></a>tree-scoped name is [strictly](#tree-scoped-name-strictly-matched) or [loosely](#tree-scoped-name-loosely-matched) matched. A <a id="ref-for-tree-scoped-name-strictly-matched②"></a>strictly matched <a id="ref-for-css-tree-scoped-name①②"></a>tree-scoped name only matches if both names are associated with the same tree. A <a id="ref-for-tree-scoped-name-loosely-matched③"></a>loosely matched <a id="ref-for-css-tree-scoped-name①③"></a>tree-scoped name also matches if the <a id="ref-for-css-tree-scoped-name①④"></a>tree-scoped name is associated with an ancestor tree.

<a id="ref-for-css-tree-scoped-name①⑤"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-concept-tree-root⑨"></a>

If two [tree-scoped names](#css-tree-scoped-name) are directly compared (for example, when comparing [computed values](https://drafts.csswg.org/css-cascade-5/#computed-value)), they are considered to match only if their identifiers match, <em>and</em> their [root](https://dom.spec.whatwg.org/#concept-tree-root)s match exactly. (If one has a <a id="ref-for-concept-tree-root①⓪"></a>root that’s an ancestor of the other, for example, they <em>do not</em> match.)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3293692a"></a> TODO: Fix all the at-rules that define global names, and the properties that reference them, to use these concepts.
>
> Global names include:
>
> - <a id="ref-for-at-font-face-rule④"></a>
>
>   <a id="ref-for-propdef-font-family①"></a>
>
>   [@font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule), referenced by [font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family)
>
> - <a id="ref-for-at-ruledef-font-feature-values"></a>
>
>   <a id="ref-for-propdef-font-family②"></a>
>
>   [@font-feature-values](https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values), referenced by [font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family)
>
> - <a id="ref-for-at-ruledef-keyframes①"></a>
>
>   <a id="ref-for-propdef-animation-name①"></a>
>
>   [@keyframes](https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes), referenced by [animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name)
>
> - <a id="ref-for-at-ruledef-counter-style"></a>
>
>   <a id="ref-for-propdef-list-style-type"></a>
>
>   [@counter-style](https://drafts.csswg.org/css-counter-styles-3/#at-ruledef-counter-style), referenced by [list-style-type](https://drafts.csswg.org/css-lists-3/#propdef-list-style-type)
>
> - <a id="ref-for-at-ruledef-profile"></a>
>
>   <a id="ref-for-funcdef-color"></a>
>
>   [@color-profile](https://drafts.csswg.org/css-color-5/#at-ruledef-profile), referenced by the [color()](https://drafts.csswg.org/css-color-5/#funcdef-color) function
>
> - <a id="ref-for-at-ruledef-font-palette-values"></a>
>
>   <a id="ref-for-propdef-font-palette"></a>
>
>   [@font-palette-values](https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-palette-values), referenced by [font-palette](https://drafts.csswg.org/css-fonts-4/#propdef-font-palette)
>
> - others?

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cd0cbc57"></a>
>
> <a id="ref-for-concept-shadow-tree④①"></a>
>
> For example, given the following document (using the imaginary \<::shadow\>\</::shadow\> markup to indicate an element’s [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree)):
>
> ```text
> <p class=outer>Here's some text in the outer document's "foo" font.
> <style>
>   @font-face {
>     font-family: foo;
>     src: url(https://example.com/outer.woff);
>   }
>   body { font-family: foo; }
>   my-component::part(text) { font-family: foo; }
> </style>
> 
> <my-component>
>   <::shadow>
>     <p class=inner-default>I'm inheriting the outer document's font-family.
>     <p class=inner-styled>And I'm explicitly styled to be in the component's "foo" font.
>     <p class=part-styled part=text>
>       I'm explicitly styled by the outer document,
>       and get the outer document's "foo" font.
>     <style>
>       @font-face {
>         font-family: foo;
>         src: url(https://example.com/inner.woff);
>       }
>       .inner-styled { font-family: foo; }
>     </style>
>   </::shadow>
> </my-component>
> ```
>
> <a id="ref-for-at-font-face-rule⑤"></a>
>
> The .outer element references the outer [@font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule), using the "outer.woff" file.
>
> <a id="ref-for-propdef-font-family③"></a>
>
> <a id="ref-for-css-tree-scoped-reference⑥"></a>
>
> The .inner-default element inherits the [font-family: foo](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) value from the outer document, using the same [tree-scoped reference](#css-tree-scoped-reference) as .outer, and thus also uses the "outer.woff" font file.
>
> <a id="ref-for-propdef-font-family④"></a>
>
> <a id="ref-for-css-tree-scoped-reference⑦"></a>
>
> The .inner-style element, on the other hand, receives a [font-family: foo](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) from the stylesheet inside the shadow, and thus its [tree-scoped reference](#css-tree-scoped-reference) refers to the shadow’s @font-family, and it uses the "inner.woff" file.
>
> <a id="ref-for-css-tree-scoped-reference⑧"></a>
>
> The .part-styled element also receives its style from the outer document, tho by being directly set rather than by inheritance. Thus, its [tree-scoped reference](#css-tree-scoped-reference) also refer’s to the outer document, and it uses the "outer.woff" file.

<a id="ref-for-css-tree-scoped-name①⑥"></a>

<a id="ref-for-css-tree-scoped-reference⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6de7b535"></a> Here is a more complex example, showing three levels of trees, and illustrating precisely how [tree-scoped names](#css-tree-scoped-name) and [tree-scoped references](#css-tree-scoped-reference) inherit.
>
> ```text
> <style>
>   @font-face {
>     font-family: foo;
>     src: url(https://example.com/outer.woff);
>   }
>   body { font-family: foo; }
> </style>
> 
> <child-component>
>   <::shadow>
>     <style>
>       @font-face {
>         font-family: foo;
>         src: url(https://example.com/inner.woff);
>       }
>     </style>
> 
>     <grandchild-component>
>       <::shadow>
>         <p class=inner-default>
>           I'm inheriting the outer document's "foo" font.
>         </p>
>         <p class=inner-search>
>           And I can't find a local "foo" font,
>           so I'm searching further up the tree,
>           and find the shadow's "foo" font.
>         </p>
>         <style>
>         .inner-search { font-family: foo; }
>         </style>
>       </::shadow>
>     </grandchild-component>
>   </::shadow>
> </child-component>
> ```
>
> <a id="ref-for-propdef-font-family⑤"></a>
>
> <a id="ref-for-at-font-face-rule⑥"></a>
>
> Here, just as in the previous example, .inner-default is inheriting the [font-family: foo](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) declared in the outer document, and so it ends up referencing the outer document’s [@font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule), and is rendered with the "outer.woff" file.
>
> <a id="ref-for-at-font-face-rule⑦"></a>
>
> On the other hand, .inner-search receives its style from a stylesheet in `<grandchild-component>`’s shadow tree, so it attempts to find a [@font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule) defining a foo font in that tree. There is no such <a id="ref-for-at-font-face-rule⑧"></a>@font-face, so it starts walking up the shadow trees, finding an appropriate <a id="ref-for-at-font-face-rule⑨"></a>@font-face in `<child-component>`, so it’s rendered with the "inner.woff" file.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/font-face-001.html`
- `css/css-scoping/font-face-002.html`
- `css/css-scoping/font-face-003.html`
- `css/css-scoping/font-face-004.html`
- `css/css-scoping/font-face-005.html`
- `css/css-scoping/font-face-006.html`
- `css/css-scoping/font-face-007.html`
- `css/css-scoping/font-face-008.html`
- `css/css-scoping/font-face-009.html`
- `css/css-scoping/keyframes-003.html`
- `css/css-scoping/keyframes-004.html`
- `css/css-scoping/keyframes-005.html`
- `css/css-scoping/keyframes-006.html`
- `css/css-scoping/scoped-reference-animation-001.html`
- `css/css-scoping/scoped-reference-animation-002.html`

#### <a id="shadow-names-serialization"></a>4.2.1.  Serialized Tree-Scoped References

<a id="ref-for-css-tree-scoped-reference①⓪"></a>

<a id="ref-for-concept-tree-root①①"></a>

If a [tree-scoped reference](#css-tree-scoped-reference) is serialized, it serializes only its value; the associated [root](https://dom.spec.whatwg.org/#concept-tree-root) is lost.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ad083628"></a>
>
> <a id="ref-for-concept-shadow-tree④②"></a>
>
> This implies that \`el.style.foo = getComputedStyle(el).foo;\` is not necessarily a no-op, like it typically was before [shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) existed.
>
> <a id="ref-for-concept-shadow-tree④③"></a>
>
> For example, given the following document (using the imaginary \<::shadow\>\</::shadow\> markup to indicate an element’s [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree)):
>
> ```text
> <p class=outer>Here's some text in the outer document's "foo" font.
> <style>
>   @font-face {
>     font-family: foo;
>     src: url(foo.woff);
>   }
>   body { font-family: foo; }
> </style>
> 
> <my-component>
>   <::shadow>
>     <p class=inner-default>I'm inheriting the outer document's font-family.
>     <p class=inner-styled>And I'm explicitly styled to be in the component's "foo" font.
>     <style>
>       @font-face {
>         font-family: foo;
>         src: url(https://example.com/foo.woff);
>       }
>       .inner-styled { font-family: foo; }
>     </style>
>     <script>
>       const innerDefault = document.querySelector('.inner-default');
>       const innerStyled = document.querySelector('.inner-styled');
>       const defaultFont = getComputedStyle(innerDefault).fontFamily;
>       const styledFont = getComputedStyle(innerStyled).fontFamily;
> 
>       console.log(defaultFont == styledFont); // true!
>     </script>
>   </::shadow>
> </my-component>
> ```
>
> <a id="ref-for-at-font-face-rule①⓪"></a>
>
> <a id="ref-for-propdef-font-family⑥"></a>
>
> <a id="ref-for-css-tree-scoped-reference①①"></a>
>
> The `.outer` element is styled with the outer document’s "foo" [@font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule). The `.inner-default` element inherits [font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) from the outer document, meaning it inherits a [tree-scoped reference](#css-tree-scoped-reference) referencing that outer document, and so it’s in the same font as `.outer`.
>
> <a id="ref-for-css-tree-scoped-reference①②"></a>
>
> <a id="ref-for-at-font-face-rule①①"></a>
>
> Meanwhile, `.inner-styled` is explicitly styled from inside the shadow root, so it receives a fresh [tree-scoped reference](#css-tree-scoped-reference) referencing its shadow tree, and it is instead styled the shadow’s own "foo" [@font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule).
>
> <a id="ref-for-propdef-font-family⑦"></a>
>
> <a id="ref-for-concept-tree-root①②"></a>
>
> <a id="ref-for-css-tree-scoped-reference①③"></a>
>
> Despite that, the script running inside the component sees the two elements as having the same value for [font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family), because the [root](https://dom.spec.whatwg.org/#concept-tree-root)-reference part of a [tree-scoped reference](#css-tree-scoped-reference) is not preserved by serialization. If it were to set `innerDefault.style.fontFamily = defaultFont;` (thus setting the <a id="ref-for-propdef-font-family⑧"></a>font-family property of the element’s attribute stylesheet, which lives in the shadow tree), the `.inner-default` element would suddenly switch to the same font as `.inner-styled`!

<a id="ref-for-concept-tree-root①③"></a>

<a id="ref-for-css-tree-scoped-reference①④"></a>

<a id="ref-for-css-reify"></a>

<a id="ref-for-concept-node-tree②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\[css-typed-om-1\]](#biblio-css-typed-om-1) is expected to reflect the [root](https://dom.spec.whatwg.org/#concept-tree-root) reference of a [tree-scoped reference](#css-tree-scoped-reference) in its [reification](https://drafts.css-houdini.org/css-typed-om-1/#css-reify) rules for values, allowing authors to tell what [node tree](https://dom.spec.whatwg.org/#concept-node-tree) the reference is taking its values from, and allowing values to be transported across <a id="ref-for-concept-node-tree③"></a>node trees without changing their meaning.

## <a id="exposing"></a>5. Exposing a Shadow Element

Elements in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) may be exported for styling by stylesheets outside the tree using the part and exportparts attributes.

<a id="ref-for-ordered-set"></a>

Each element has a <a id="element-part-name-list"></a>part name list which is an [ordered set](https://infra.spec.whatwg.org/#ordered-set) of tokens.

<a id="ref-for-list"></a>

<a id="ref-for-tuple"></a>

<a id="ref-for-string"></a>

Each element has a <a id="element-forwarded-part-name-list"></a>forwarded part name list which is a [list](https://infra.spec.whatwg.org/#list) of [tuples](https://infra.spec.whatwg.org/#tuple) containing a [string](https://infra.spec.whatwg.org/#string) for the inner part being forwarded and a <a id="ref-for-string①"></a>string giving the name it will be exposed as.

<a id="ref-for-concept-shadow-root⑧"></a>

<a id="ref-for-string②"></a>

<a id="ref-for-ordered-set①"></a>

Each [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) can be thought of as having a <a id="shadow-root-part-element-map"></a>part element map with keys that are [strings](https://infra.spec.whatwg.org/#string) and values that are [ordered sets](https://infra.spec.whatwg.org/#ordered-set) of elements.

<a id="ref-for-shadow-root-part-element-map"></a>

The [part element map](#shadow-root-part-element-map) is described only as part of the algorithm for calculating style in this spec. It is not exposed via the DOM, as calculating it may be expensive and exposing it could allow access to elements inside closed shadow roots.

<a id="ref-for-shadow-root-part-element-map①"></a>

<a id="ref-for-element-part-name-list"></a>

<a id="ref-for-element-forwarded-part-name-list"></a>

[Part element maps](#shadow-root-part-element-map) are affected by the addition and removal of elements and changes to the [part name lists](#element-part-name-list) and [forwarded part name lists](#element-forwarded-part-name-list) of elements in the DOM.

<a id="ref-for-shadow-root-part-element-map②"></a>

To <a id="calculate-the-part-element-map"></a>calculate the [part element map](#shadow-root-part-element-map) of a shadow root, <var>outerRoot</var>:

1.  <a id="ref-for-concept-tree-descendant②"></a>

    For each [descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) <var>el</var> within <var>outerRoot</var>:

    1.  <a id="ref-for-element-part-name-list①"></a>

        <a id="ref-for-list-append"></a>

        <a id="ref-for-shadow-root-part-element-map③"></a>

        For each <var>name</var> in <var>el</var>’s [part name list](#element-part-name-list), [append](https://infra.spec.whatwg.org/#list-append) <var>el</var> to <var>outerRoot</var>’s [part element map](#shadow-root-part-element-map)\[<var>name</var>\].

    2.  <a id="ref-for-element-shadow-host②③"></a>

        If <var>el</var> is a [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) itself then let <var>innerRoot</var> be its shadow root.

    3.  <a id="ref-for-calculate-the-part-element-map"></a>

        <a id="ref-for-shadow-root-part-element-map④"></a>

        [Calculate](#calculate-the-part-element-map) <var>innerRoot</var>’s [part element map](#shadow-root-part-element-map).

    4.  <a id="ref-for-element-forwarded-part-name-list①"></a>

        For each <var>innerName</var>/<var>outerName</var> in <var>el</var>’s [forwarded part name list](#element-forwarded-part-name-list):

        1.  If <var>innerName</var> is an ident:

            1.  <a id="ref-for-shadow-root-part-element-map⑤"></a>

                Let <var>innerParts</var> be <var>innerRoot</var>’s [part element map](#shadow-root-part-element-map)\[<var>innerName</var>\]

            2.  <a id="ref-for-list-append①"></a>

                <a id="ref-for-shadow-root-part-element-map⑥"></a>

                [Append](https://infra.spec.whatwg.org/#list-append) the elements in <var>innerParts</var> to <var>outerRoot</var>’s [part element map](#shadow-root-part-element-map)\[<var>outerName</var>\]

        2.  If <var>innerName</var> is a pseudo-element name:

            1.  <a id="ref-for-list-append②"></a>

                <a id="ref-for-shadow-root-part-element-map⑦"></a>

                [Append](https://infra.spec.whatwg.org/#list-append) <var>innerRoot</var>’s pseudo-element(s) with that name to <var>outerRoot</var>’s [part element map](#shadow-root-part-element-map)\[<var>outerName</var>\].

### <a id="motivation"></a>5.1. Motivation

For custom elements to be fully useful and as capable as built-in elements it should be possible for parts of them to be styled from outside. Exactly what can be styled from outside should be controlled by the element author. Also, it should be possible for a custom element to present a stable "API" for styling. That is, the selector used to style a part of a custom element should not expose or require knowledge of the internal details of the element. The custom element author should be able to change the internal details of the element while leaving the selectors untouched.

The previous proposed method for styling inside the shadow tree, the \>\>\> combinator, turned out to be <em>too powerful</em> for its own good; it exposed too much of a component’s internal structure to scrutiny, defeating some of the encapsulation benefits that using Shadow DOM brings. For this, and other performance-related reasons, the \>\>\> combinator was eventually dropped.

<a id="ref-for-custom-property①"></a>

<a id="ref-for-element-shadow-host②④"></a>

This left us with using [custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) as the only way to style into a shadow tree: the component would advertise that it uses certain <a id="ref-for-custom-property②"></a>custom properties to style its internals, and the outer page could then set those properties as it wished on the [shadow host](https://dom.spec.whatwg.org/#element-shadow-host), letting inheritance push the values down to where they were needed. This works very well for many simple theming use-cases.

<a id="ref-for-custom-property③"></a>

<a id="ref-for-hover-pseudo"></a>

However, there are some cases where this falls down. If a component wishes to allow arbitrary styling of something in its shadow tree, the only way to do so is to define hundreds of [custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) (one per CSS property they wish to allow control of), which is obviously ridiculous for both usability and performance reasons. The situation is compounded if authors wish to style the component differently based on pseudo-classes like [:hover](https://drafts.csswg.org/selectors-4/#hover-pseudo); the component needs to duplicate the <a id="ref-for-custom-property④"></a>custom properties used for each pseudo-class (and each combination, like :hover:focus, resulting in a combinatorial explosion). This makes the usability and performance problems even worse.

<a id="ref-for-selectordef-part①"></a>

<a id="ref-for-custom-property⑤"></a>

We introduce [::part()](#selectordef-part) to handle this case much more elegantly and performantly. Rather than bundling everything into [custom property](https://drafts.csswg.org/css-variables-2/#custom-property) names, the functionality lives in selectors and style rule syntax, like it’s meant to. This is far more usable for both component authors and component users, should have much better performance, and allows for better encapsulation/API surface.

<a id="ref-for-selectordef-part②"></a>

<a id="ref-for-custom-property⑥"></a>

<a id="ref-for-shadow-root-part-element-map⑧"></a>

It’s important to note that [::part()](#selectordef-part) offers <em>absolutely zero new theoretical power</em>. It is not a rehash of the \>\>\> combinator, it is simply a more convenient and consistent syntax for something authors can already do with [custom properties](https://drafts.csswg.org/css-variables-2/#custom-property). By separating out the explicitly "published" parts of an element (the [part element map](#shadow-root-part-element-map)) from the sub-parts that it merely happens to contain, it also helps with encapsulation, as authors can use <a id="ref-for-selectordef-part③"></a>::part() without fear of accidental over-styling.

<a id="ref-for-element-attrdef-html-global-part"></a>

### <a id="part-attr"></a>5.2. Naming a Shadow Element: the <code><a href="#element-attrdef-html-global-part">part</a></code> attribute

<a id="ref-for-concept-shadow-tree④⑤"></a>

Any element in a shadow tree can have a <a id="element-attrdef-html-global-part"></a>`part` attribute. This is used to expose the element outside of the [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree).

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-shadow-parts/invalidation-change-part-name-forward.html`
- `css/css-shadow-parts/invalidation-change-part-name.html`

The part attribute is parsed as a space-separated list of tokens representing the part names of this element.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It’s okay to give a part multiple names. The "part name" should be considered similar to a class, not an id or tagname.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a2e8f894"></a>
>
> ```text
> <style>
>   c-e::part(textspan) { color: red; }
> </style>
> 
> <template id="c-e-template">
>   <span part="textspan">This text will be red</span>
> </template>
> <c-e></c-e>
> <script>
>   // Add template as custom element c-e
>   ...
> </script>
> ```
<a id="ref-for-element-attrdef-html-global-exportparts"></a>

### <a id="exportparts-attr"></a>5.3. Forwarding a Shadow Element: the <code><a href="#element-attrdef-html-global-exportparts">exportparts</a></code> attribute

<a id="ref-for-concept-shadow-tree④⑥"></a>

Any element in a shadow tree can have a <a id="element-attrdef-html-global-exportparts"></a>`exportparts` attribute. If the element is a shadow host, this is used to allow styling of parts from hosts inside the [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) by rules outside this the <a id="ref-for-concept-shadow-tree④⑦"></a>shadow tree (as if they were elements in the same tree as the host, named by a part attribute).

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-shadow-parts/both-part-and-exportparts.html`
- `css/css-shadow-parts/exportparts-different-scope.html`
- `css/css-shadow-parts/exportparts-layered.html`
- `css/css-shadow-parts/exportparts-multiple.html`
- `css/css-shadow-parts/invalidation-change-exportparts-forward.html`

The exportparts attribute is parsed as a comma-separated list of part mappings. Each part mapping is one of:

`innerIdent : outerIdent`  
<a id="ref-for-element-forwarded-part-name-list②"></a>

Adds `innerIdent`/`outerIdent` to el’s [forwarded part name list](#element-forwarded-part-name-list).

`ident`  
<a id="ref-for-element-forwarded-part-name-list③"></a>

Adds `ident`/`ident` to el’s [forwarded part name list](#element-forwarded-part-name-list).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is shorthand for `ident : ident`.

`::ident : outerIdent`  
<a id="ref-for-fully-styleable"></a>

<a id="ref-for-element-forwarded-part-name-list④"></a>

If `::ident` is the name of a [fully styleable pseudo-element](https://drafts.csswg.org/css-pseudo-4/#fully-styleable), adds `::ident`/`outerIdent` to el’s [forward part name list](#element-forwarded-part-name-list). Otherwise, does nothing.

anything else  
Ignored for error-recovery / future compatibility.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It’s okay to map a sub-part to several names.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8f835643"></a>
>
> ```text
> <style>
>   c-e::part(textspan) { color: red; }
> </style>
> 
> <template id="c-e-outer-template">
>   <c-e-inner exportparts="innerspan: textspan"></c-e-inner>
> </template>
> 
> <template id="c-e-inner-template">
>   <span part="innerspan">
>     This text will be red because the containing shadow
>     host forwards innerspan to the document as "textspan"
>     and the document style matches it.
>   </span>
>   <span part="textspan">
>     This text will not be red because textspan in the document style
>     cannot match against the part inside the inner custom element
>     if it is not forwarded.
>   </span>
> </template>
> 
> <c-e></c-e>
> <script>
>   // Add template as custom elements c-e-inner, c-e-outer
>   ...
> </script>
> ```
<a id="ref-for-fully-styleable①"></a>

<a id="ref-for-element-attrdef-html-global-exportparts①"></a>

<a id="ref-for-selectordef-part④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b11a0a88"></a> For example, a [fully styleable pseudo-element](https://drafts.csswg.org/css-pseudo-4/#fully-styleable) can be used in the <code><a href="#element-attrdef-html-global-exportparts">exportparts</a></code> attribute, to masquerade as a [::part()](#selectordef-part) for the component it’s in:
>
> ```text
> <template id=custom-element-template>
>   <p exportparts="::before : preceding-text, ::after : following-text">
>     Main text.
> </template>
> ```
>
> An element using that template can use a selector like x-component::part(preceding-text) to target the p::before pseudo-element in its shadow, so users of the component don’t need to know that the preceding text is implemented as a pseudo-element.

<a id="ref-for-selectordef-part⑤"></a>

### <a id="part"></a>5.4. Selecting a Shadow Element: the [::part()](#selectordef-part) pseudo-element

<a id="ref-for-element-attrdef-html-global-part①"></a>

The <a id="selectordef-part"></a>::part() pseudo-element allows you to select elements that have been exposed via a <code><a href="#element-attrdef-html-global-part">part</a></code> attribute. The syntax is:

<a id="ref-for-typedef-ident"></a>

<a id="ref-for-mult-one-plus"></a>

```text
::part() = ::part( <ident>+ )
```
<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-shadow-parts/host-part-001.html`
- `css/css-shadow-parts/host-part-002.html`
- `css/css-shadow-parts/host-part-003.html`
- `css/css-shadow-parts/host-part-nesting.html`
- `css/css-shadow-parts/multiple-scopes.html`

<a id="ref-for-selectordef-part⑥"></a>

<a id="ref-for-originating-element"></a>

<a id="ref-for-element-shadow-host②⑤"></a>

The [::part()](#selectordef-part) pseudo-element only matches anything when the [originating element](https://drafts.csswg.org/selectors-4/#originating-element) is a [shadow host](https://dom.spec.whatwg.org/#element-shadow-host).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c617aa59"></a> For example, if you have a custom button that contains a "label" element that is exposed for styling (via `part="label"`), you can select it with x-button::part(label).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-72ff019b"></a> Part names act similarly to classes: multiple elements can have the same part name, and a single element can have multiple part names.
>
> A tabstrip control might have multiple elements with `part="tab"`, all of which are selected by ::part(tab).
>
> If a single tab is active at a time, it can be specially indicated with `part="tab active"` and then selected by ::part(tab active) (or ::part(active tab), as order doesn’t matter).

<a id="ref-for-selectordef-part⑦"></a>

<a id="ref-for-fully-styleable②"></a>

<a id="ref-for-originating-element①"></a>

<a id="ref-for-concept-shadow-root⑨"></a>

<a id="ref-for-shadow-root-part-element-map⑨"></a>

<a id="ref-for-map-exists"></a>

<a id="ref-for-typedef-ident①"></a>

The [::part()](#selectordef-part) pseudo-element is a [fully styleable pseudo-element](https://drafts.csswg.org/css-pseudo-4/#fully-styleable). If the [originating element’s](https://drafts.csswg.org/selectors-4/#originating-element) [shadow root’s](https://dom.spec.whatwg.org/#concept-shadow-root) [part element map](#shadow-root-part-element-map) [contains](https://infra.spec.whatwg.org/#map-exists) the specified [\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident), <a id="ref-for-selectordef-part⑧"></a>::part() represents the elements keyed to that ident; if multiple idents are provided and the <a id="ref-for-shadow-root-part-element-map①⓪"></a>part element map contains them all, it represents the intersection of the elements keyed to each ident. Otherwise, it matches nothing.

<a id="ref-for-selectordef-part⑨"></a>

<a id="ref-for-originating-element②"></a>

[::part()](#selectordef-part) pseudo-elements inherit according to their position in the [originating element’s](https://drafts.csswg.org/selectors-4/#originating-element) shadow tree.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-11b18654"></a> For example, x-panel::part(confirm-button)::part(label) never matches anything. This is because doing so would expose more structural information than is intended.
>
> <a id="ref-for-shadow-root-part-element-map①①"></a>
>
> If the `<x-panel>`’s internal confirm button had used something like `part="label => confirm-label"` to forward the button’s internal parts up into the panel’s own [part element map](#shadow-root-part-element-map), then a selector like x-panel::part(confirm-label) would select just the one button’s label, ignoring any other labels.

<a id="ref-for-element"></a>

### <a id="idl"></a>5.5. Extensions to the <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> Interface

<a id="ref-for-element①"></a>

<a id="ref-for-SameObject"></a>

<a id="ref-for-PutForwards"></a>

<a id="ref-for-dom-domtokenlist-value"></a>

<a id="ref-for-domtokenlist"></a>

<a id="dom-element-part"></a>

```text
partial interface Element {
  [SameObject, PutForwards=value] readonly attribute DOMTokenList part;
};
```
<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-shadow-parts/idlharness.html`
- `css/css-shadow-parts/invalidation-change-part-name-idl-domtokenlist.html`
- `css/css-shadow-parts/invalidation-change-part-name-idl-setter.html`
- `css/css-shadow-parts/part-name-idl.html`

The part attribute’s getter must return a DOMTokenList object whose associated element is the context object and whose associated attribute’s local name is part. The token set of this particular DOMTokenList object are also known as the element’s parts.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-853df044"></a> Define this as a superglobal in the DOM spec. [\[w3c/csswg-drafts Issue \#3424\]](https://github.com/w3c/csswg-drafts/issues/3424)

### <a id="parsing"></a>5.6. Microsyntaxes for parsing

#### <a id="parsing-mapping"></a>5.6.1.  Rules for parsing part mappings

<a id="ref-for-tuple①"></a>

A <a id="valid-part-mapping"></a>valid part mapping is a [tuple](https://infra.spec.whatwg.org/#tuple) of tokens separated by a U+003A COLON character and any number of space characters before or after the U+003A COLON The tokens must not contain U+003A COLON or U+002C COMMA characters.

The rules for parsing a part mapping are as follows:

1.  Let <var>input</var> be the string being parsed.

2.  Let <var>position</var> be a pointer into <var>input</var>, initially pointing at the start of the string.

3.  <a id="ref-for-collect-a-sequence-of-code-points"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters

4.  <a id="ref-for-collect-a-sequence-of-code-points①"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are not space characters or U+003A COLON characters, and let <var>first token</var> be the result.

5.  If <var>first token</var> is empty then return error.

6.  <a id="ref-for-collect-a-sequence-of-code-points②"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters.

7.  <a id="ref-for-tuple②"></a>

    If the end of the <var>input</var> has been reached, return the [tuple](https://infra.spec.whatwg.org/#tuple) (<var>first token</var>, <var>first token</var>)

8.  If character at <var>position</var> is not a U+003A COLON character, return error.

9.  Consume the U+003A COLON character.

10. <a id="ref-for-collect-a-sequence-of-code-points③"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters.

11. <a id="ref-for-collect-a-sequence-of-code-points④"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are not space characters or U+003A COLON characters. and let <var>second token</var> be the result.

12. If <var>second token</var> is empty then return error.

13. <a id="ref-for-collect-a-sequence-of-code-points⑤"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters.

14. If <var>position</var> is not past the end of <var>input</var> then return error.

15. <a id="ref-for-tuple③"></a>

    Return the [tuple](https://infra.spec.whatwg.org/#tuple) (<var>first token</var>, <var>second token</var>).

#### <a id="parsing-mapping-list"></a>5.6.2.  Rules for parsing a list of part mappings

A <a id="valid-list-of-part-mappings"></a>valid list of part mappings is a number of valid part mappings separated by a U+002C COMMA character and any number of space characters before or after the U+002C COMMA

The rules for parsing a list of part mappings are as follow:

1.  Let <var>input</var> be the string being parsed.

2.  <a id="ref-for-split-on-commas"></a>

    [Split the string <var>input</var> on commas](https://infra.spec.whatwg.org/#split-on-commas). Let <var>unparsed mappings</var> be the resulting list of strings.

3.  <a id="ref-for-list①"></a>

    <a id="ref-for-tuple④"></a>

    Let <var>mappings</var> be an initially empty [list](https://infra.spec.whatwg.org/#list) of [tuples](https://infra.spec.whatwg.org/#tuple) of tokens. This <a id="ref-for-list②"></a>list will be the result of this algorithm.

4.  For each string <var>unparsed mapping</var> in <var>unparsed mappings</var>, run the following substeps:

    1.  If <var>unparsed mapping</var> is empty or contains only space characters, continue to the next iteration of the loop.

    2.  Let <var>mapping</var> be the result of parsing <var>unparsed mapping</var> using the rules for parsing part mappings.

    3.  If <var>mapping</var> is an error then continue to the next iteration of the loop. This allows clients to skip over new syntax that is not understood.

    4.  Append <var>mapping</var> to <var>mappings</var>.

## <a id="changes"></a>6. Changes

The following significant changes were made since the [3 April 2014 Working Draft](css-scoping-1-host-definitions--WD-css-scoping-1-20140403--0742453932fc.md).

- Defined that tree-scoped names inherit into descendant shadow trees

- Renamed ::content to ::slotted.

- Define the flattened tree.

- Generally reorg and rebase the Shadow DOM section on top of current DOM.

- Punt @scope and related things, and ::region and related things, to the next level of the draft.

- <a id="ref-for-selectordef-host⑧"></a>

  <a id="ref-for-selectordef-host-function④"></a>

  <a id="ref-for-selectordef-host-context④"></a>

  <a id="ref-for-selectordef-slotted①①"></a>

  Define the specificity of [:host](#selectordef-host), [:host()](#selectordef-host-function), [:host-context()](#selectordef-host-context), and [::slotted()](#selectordef-slotted)

- Remove the \>\>\> (previously called /deep/) combinator.

- <a id="ref-for-selectordef-slotted①②"></a>

  Define that tree-abiding pseudos are allowed after [::slotted()](#selectordef-slotted).

- <a id="ref-for-typedef-compound-selector-list"></a>

  Allow [\<compound-selector-list\>](https://drafts.csswg.org/selectors-4/#typedef-compound-selector-list) in all the pseudos.

- Define a way to create a stylesheet of default element styles for a given element.

- Make featureless elements match nothing.

- <a id="ref-for-in-the-context-of-a-shadow-tree⑤"></a>

  Define [in the context of a shadow tree](#in-the-context-of-a-shadow-tree).

- Merged [\[css-shadow-parts\]](#biblio-css-shadow-parts) and renamed the specification to CSS Shadow Module Level 1. ([Issue \#5809](https://github.com/w3c/csswg-drafts/issues/5809#issuecomment-910896765))

- <a id="ref-for-selectordef-part①⓪"></a>

  Added support for multiple names in [::part()](#selectordef-part)

- <a id="ref-for-element-forwarded-part-name-list⑤"></a>

  Renamed 'part name map' to [forwarded part name list](#element-forwarded-part-name-list)

- Restructured 'part element list' algorithm

- <a id="ref-for-selectordef-part①①"></a>

  <a id="ref-for-fully-styleable③"></a>

  Moved various [::part()](#selectordef-part) details to [fully styleable pseudo-element](https://drafts.csswg.org/css-pseudo-4/#fully-styleable)

- Added Web Platform Tests coverage

- Minor editorial improvements

## <a id="privacy"></a>7. Privacy Considerations

This specification introduces Shadow DOM and some shadow-piercing capabilities, but this does not introduce any privacy issues —​shadow DOM, as currently specified, is intentionally not a privacy boundary (and the parts of the UA that use shadow DOM and <em>do</em> have a privacy boundary implicitly rely on protections not yet specified, which protect them from the things defined in this specification).

## <a id="security"></a>8. Security Considerations

<a id="ref-for-concept-shadow-tree④⑧"></a>

This specification introduces Shadow DOM and some shadow-piercing capabilities, but this does not introduce any security issues —​shadow DOM, as currently specified, is intentionally not a security boundary, merely a convenience for page authors. Exposing [shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) to selectors in this way introduces no new security considerations. (And the parts of the UA that use shadow DOM and <em>do</em> have a security boundary implicitly rely on protections not yet specified, which protect them from the things defined in this specification).

<strong>Source test references (hidden in original rendering)</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-scoping/chrome-1492368-crash.html`

## <a id="references"></a>References

Generated bibliography: these reference descriptions and external auto-links were resolved using Bikeshed 7.1.3’s bundled data; they are not a historical capture of the linked specifications.

### <a id="normative"></a>Normative References

<a id="biblio-css-anchor-position-1"></a>\[CSS-ANCHOR-POSITION-1\]  
Tab Atkins Jr.; Elika Etemad; Ian Kilpatrick. [CSS Anchor Positioning Module Level 1](https://drafts.csswg.org/css-anchor-position-1/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-anchor-position-1&#x2F;](https://drafts.csswg.org/css-anchor-position-1/)

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://drafts.csswg.org/css-animations/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-animations&#x2F;](https://drafts.csswg.org/css-animations/)

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://drafts.csswg.org/css-cascade-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-cascade-4&#x2F;](https://drafts.csswg.org/css-cascade-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://drafts.csswg.org/css-cascade-5/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-cascade-5&#x2F;](https://drafts.csswg.org/css-cascade-5/)

<a id="biblio-css-cascade-6"></a>\[CSS-CASCADE-6\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://drafts.csswg.org/css-cascade-6/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-cascade-6&#x2F;](https://drafts.csswg.org/css-cascade-6/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; Una Kravets; Lea Verou. [CSS Color Module Level 5](https://drafts.csswg.org/css-color-5/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-color-5&#x2F;](https://drafts.csswg.org/css-color-5/)

<a id="biblio-css-counter-styles-3"></a>\[CSS-COUNTER-STYLES-3\]  
Tab Atkins Jr.. [CSS Counter Styles Level 3](https://drafts.csswg.org/css-counter-styles/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-counter-styles&#x2F;](https://drafts.csswg.org/css-counter-styles/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://drafts.csswg.org/css-display-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-display-4&#x2F;](https://drafts.csswg.org/css-display-4/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://drafts.csswg.org/css-fonts-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-fonts-4&#x2F;](https://drafts.csswg.org/css-fonts-4/)

<a id="biblio-css-fonts-5"></a>\[CSS-FONTS-5\]  
Chris Lilley. [CSS Fonts Module Level 5](https://drafts.csswg.org/css-fonts-5/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-fonts-5&#x2F;](https://drafts.csswg.org/css-fonts-5/)

<a id="biblio-css-lists-3"></a>\[CSS-LISTS-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://drafts.csswg.org/css-lists-3/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-lists-3&#x2F;](https://drafts.csswg.org/css-lists-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://drafts.csswg.org/css-pseudo-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-pseudo-4&#x2F;](https://drafts.csswg.org/css-pseudo-4/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-syntax&#x2F;](https://drafts.csswg.org/css-syntax/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-values-4&#x2F;](https://drafts.csswg.org/css-values-4/)

<a id="biblio-css-variables-2"></a>\[CSS-VARIABLES-2\]  
[CSS Custom Properties for Cascading Variables Module Level 2](https://drafts.csswg.org/css-variables-2/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-variables-2&#x2F;](https://drafts.csswg.org/css-variables-2/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://drafts.csswg.org/selectors/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;selectors&#x2F;](https://drafts.csswg.org/selectors/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Non-Normative References

<a id="biblio-css-shadow-parts"></a>\[CSS-SHADOW-PARTS\]  
Tab Atkins Jr.; Fergal Daly. [CSS Shadow Parts Module Level 1](https://drafts.csswg.org/css-shadow-parts/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-shadow-parts&#x2F;](https://drafts.csswg.org/css-shadow-parts/)

<a id="biblio-css-typed-om-1"></a>\[CSS-TYPED-OM-1\]  
Tab Atkins Jr.; François Remy. [CSS Typed OM Level 1](https://drafts.css-houdini.org/css-typed-om-1/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;css-houdini&#x2E;org&#x2F;css-typed-om-1&#x2F;](https://drafts.css-houdini.org/css-typed-om-1/)

<a id="biblio-selectors-3"></a>\[SELECTORS-3\]  
Tantek Çelik; et al. [Selectors Level 3](https://drafts.csswg.org/selectors-3/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;selectors-3&#x2F;](https://drafts.csswg.org/selectors-3/)

## <a id="source-linking-configuration"></a>Source linking configuration

Compiler configuration recorded in the pinned source; retained for provenance.

```text
spec:cascade-4; type:dfn; text: inherit
spec:css-color-5; type:function; text:color()
spec:css-pseudo-4; type:selector;
	text:::after
	text:::before
spec:css-fonts-4; type:property; text:font-family
spec:dom; type:dfn;
	text:child
	text:children
	text:descendant
	text:element; for:/
	text:find flattened slottables
	text:find slottables
	text:host
	text:root; for:tree
	text:shadow root; for:/
spec:html; type:element;
	text:style
spec:infra; type:dfn;
	text:string
	text:list
spec:selectors-4;
	type:selector; text::hover
	type:dfn;
		text:dynamic profile
		text:static profile
		text:type selector
```