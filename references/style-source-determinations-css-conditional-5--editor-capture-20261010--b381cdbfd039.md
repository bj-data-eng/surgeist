Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Conditional Rules Module Level 5](https://drafts.csswg.org/css-conditional-5/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10. Source status: W3C Working Draft, 8 October 2026.

Full captured HTML SHA-256: `3b4a4b4ef336edeaca211cdb90481bfcf9cda1955c1cabce485b98e2fcb497e3` (494912 bytes). Conversion input SHA-256: `b381cdbfd039d5125c5bb7652f4d8706a3048e2505cb967cdccc5f5763dac975` (64490 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained scope: `container-type`, `size-container`, `container-lengths`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Conditional Rules Module Level 5

<a id="w3c-state"></a>[W3C Working Draft](https://www.w3.org/standards/types/#WD), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://www.w3.org/TR/2026/WD-css-conditional-5-20261008/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-conditional-5/>

<strong>Editor's Draft:</strong>

<https://drafts.csswg.org/css-conditional-5/>

<strong>History:</strong>

<https://www.w3.org/standards/history/css-conditional-5/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-conditional-5)

[Inline In Spec](https://drafts.csswg.org/css-conditional-5/#issues-index)

<strong>Editors:</strong>

[L. David Baron](https://dbaron.org/) ([Google](https://www.google.com/))

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Chris Lilley](https://svgees.us/) (W3C)

[Miriam E. Suzanne](https://www.miriamsuzanne.com/who/) (Invited Expert)

[Lea Verou](https://lea.verou.me/about) (Invited Expert)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-conditional-5/Overview.bs)

<strong>Delta Spec:</strong>

yes

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-conditional/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

### <a id="container-type"></a>5.1.  Creating Query Containers: the <a id="ref-for-propdef-container-type①"></a>[container-type](#propdef-container-type) property[](#container-type)

| Field                                                                                    | Definition                                                                                                                                                                                                                                            |
|------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-container-type"></a><strong>container-type</strong>                                                                                                                                                                                    |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | normal <a id="ref-for-comb-one⑨"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ \[ size <a id="ref-for-comb-one①⓪"></a>\| inline-size \] <a id="ref-for-comb-any"></a>[\|\|](https://www.w3.org/TR/css-values-4/#comb-any) scroll-state \] |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | normal                                                                                                                                                                                                                                                |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                   |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                    |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                   |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified keyword                                                                                                                                                                                                                                     |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                           |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | not animatable                                                                                                                                                                                                                                        |

The <a id="ref-for-propdef-container-type②"></a>[container-type](#propdef-container-type) property establishes the element as a <a id="ref-for-query-container⑤"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container) for certain types of queries. For size <a id="ref-for-container-query①"></a>[container queries](https://drafts.csswg.org/css-conditional-5/#container-query), which require certain types of containment, elements are explicitly made <a id="ref-for-query-container⑥"></a>query containers through this property. For other types of <a id="ref-for-query-container⑦"></a>query containers any element can be a <a id="ref-for-query-container⑧"></a>query container, such as for <a id="ref-for-container-style-query①"></a>[container style queries](https://drafts.csswg.org/css-conditional-5/#container-style-query).

Values have the following meanings:

<strong><a id="valdef-container-type-size"></a><strong>size</strong></strong>

Establishes a <a id="ref-for-query-container⑨"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container) for <a id="ref-for-container-size-query③"></a>[container size queries](#container-size-query) on both the <a id="ref-for-inline-axis"></a>[inline](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) and <a id="ref-for-block-axis"></a>[block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). Applies <a id="ref-for-style-containment"></a>[style containment](https://www.w3.org/TR/css-contain-2/#style-containment) and <a id="ref-for-size-containment"></a>[size containment](https://www.w3.org/TR/css-contain-2/#size-containment) to the <a id="ref-for-principal-box"></a>[principal box](https://www.w3.org/TR/css-display-4/#principal-box), and establishes an <a id="ref-for-independent-formatting-context"></a>[independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context).

<strong><a id="valdef-container-type-inline-size"></a><strong>inline-size</strong></strong>

Establishes a <a id="ref-for-query-container①⓪"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container) for <a id="ref-for-container-size-query④"></a>[container size queries](#container-size-query) on the container’s own <a id="ref-for-inline-axis①"></a>[inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). Applies <a id="ref-for-style-containment①"></a>[style containment](https://www.w3.org/TR/css-contain-2/#style-containment) and <a id="ref-for-inline-size-containment"></a>[inline-size containment](https://www.w3.org/TR/css-contain-3/#inline-size-containment) to the <a id="ref-for-principal-box①"></a>[principal box](https://www.w3.org/TR/css-display-4/#principal-box), and establishes an <a id="ref-for-independent-formatting-context①"></a>[independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context).

<strong><a id="valdef-container-type-scroll-state"></a><strong>scroll-state</strong></strong>

Establishes a <a id="ref-for-query-container①①"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container) for <a id="ref-for-container-scroll-state-query①"></a>[container scroll-state queries](https://drafts.csswg.org/css-conditional-5/#container-scroll-state-query). For queries that match the state of the scroller itself (such as scrollable), the element is only a <a id="ref-for-query-container①②"></a>query container on the element’s own <a id="ref-for-scrollable-axis"></a>[scrollable axises](https://drafts.csswg.org/css-overflow-3/#scrollable-axis).

Note: This means that a scrollable query, for example, could query against different containers for scroll-state(scrollable: block) and scroll-state(scrollable: inline) queries.

<strong><a id="valdef-container-type-normal"></a><strong>normal</strong></strong>

The element is not a <a id="ref-for-query-container①③"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container) for any <a id="ref-for-container-size-query⑤"></a>[container size queries](#container-size-query) or <a id="ref-for-container-scroll-state-query②"></a>[container scroll-state queries](https://drafts.csswg.org/css-conditional-5/#container-scroll-state-query), but remains a <a id="ref-for-query-container①④"></a>query container for <a id="ref-for-container-style-query②"></a>[container style queries](https://drafts.csswg.org/css-conditional-5/#container-style-query).

<a id="example-5708b886"></a>

<strong>Example:</strong>

[](#example-5708b886) For example, authors can create container-responsive typography, adjusting <a id="ref-for-propdef-font-size"></a>[font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size), <a id="ref-for-propdef-line-height"></a>[line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height), and other typographic concerns based on the size of a container:

``` text
aside, main {
  container-type: inline-size;
}

h2 { font-size: 1.2em; }

@container (width > 40em) {
  h2 { font-size: 1.5em; }
}
```

The 40em value used in the query condition is relative to the <a id="ref-for-computed-value"></a>[computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of <a id="ref-for-propdef-font-size①"></a>[font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) on the relevant <a id="ref-for-query-container①⑤"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container).

<a id="example-76326f8c"></a>

<strong>Example:</strong>

[](#example-76326f8c) Containers can also expose computed style values for querying. This can be useful for toggling behavior across multiple properties:

``` text
@container style(--cards: small) {
  article {
  border: thin solid silver;
  border-radius: 0.5em;
  padding: 1em;
  }
}
```

<a id="example-9f75dbe5"></a>

<strong>Example:</strong>

[](#example-9f75dbe5) Containers can also expose state that depends on scroll offset. This example styles a descendant of a sticky positioned element when it is stuck to the top edge:

``` text
#sticky {
  container-type: scroll-state;
  position: sticky;
}
@container scroll-state(stuck: top) {
  #sticky-child {
  background-color: lime;
  }
}
```

### <a id="size-container"></a>6.1.  Size Container Features[](#size-container)

A <a id="container-size-query"></a><strong>container size query</strong> allows querying the size of the <a id="ref-for-query-container③⓪"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-principal-box②"></a>[principal box](https://www.w3.org/TR/css-display-4/#principal-box). It is a boolean combination of individual <a id="size-features"></a><strong>size features</strong> (<a id="ref-for-typedef-size-feature①"></a>[\<size-feature\>](#typedef-size-feature)) that each query a single, specific dimensional feature of the <a id="ref-for-query-container③①"></a>query container. The syntax of a <a id="typedef-size-feature"></a><strong><a id="ref-for-typedef-size-feature②"></a>[\<size-feature\>](#typedef-size-feature)</strong> is the same as for a <a id="ref-for-media-feature①"></a>[media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature): a feature name, a comparator, and a value. [\[mediaqueries-5\]](https://drafts.csswg.org/css-conditional-5/#biblio-mediaqueries-5) The boolean syntax and logic combining <a id="ref-for-size-features"></a>[size features](#size-features) into a <a id="ref-for-container-size-query⑦"></a>[size query](#container-size-query) is the same as for <a id="ref-for-css-feature-queries"></a>[CSS feature queries](https://www.w3.org/TR/css-conditional-3/#css-feature-queries). (See <a id="ref-for-at-ruledef-supports⑤"></a>[&#64;supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports). [\[css-conditional-3\]](https://drafts.csswg.org/css-conditional-5/#biblio-css-conditional-3))

If the <a id="ref-for-query-container③②"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container) does not have a <a id="ref-for-principal-box③"></a>[principal box](https://www.w3.org/TR/css-display-4/#principal-box), or the principal box is not a <a id="ref-for-layout-containment-box"></a>[layout containment box](https://www.w3.org/TR/css-contain-2/#layout-containment-box), or the <a id="ref-for-query-container③③"></a>query container does not support <a id="ref-for-container-size-query⑧"></a>[container size queries](#container-size-query) on the relevant axes, then the result of evaluating the <a id="ref-for-size-features①"></a>[size feature](#size-features) is unknown.

<a id="ref-for-relative-length"></a>[Relative length](https://www.w3.org/TR/css-values-4/#relative-length) units (including <a id="ref-for-container-query-length①"></a>[container query length](#container-query-length) units) and <a id="ref-for-custom-property"></a>[custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) in <a id="ref-for-container-query①③"></a>[container query](https://drafts.csswg.org/css-conditional-5/#container-query) conditions are evaluated based on the <a id="ref-for-computed-value②"></a>[computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of the <a id="ref-for-query-container③④"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container).

Tree counting functions ([CSS Values 5 § 9 Tree Counting Functions: the sibling-count() and sibling-index() notations](https://www.w3.org/TR/css-values-5/#tree-counting)) are evaluated against the container element.

Note: This is different from the handling of relative units in <a id="ref-for-media-query②"></a>[media queries](https://www.w3.org/TR/mediaqueries-5/#media-query).

Note: If <a id="ref-for-custom-property①"></a>[custom property](https://www.w3.org/TR/css-variables-1/#custom-property) substitution results in an invalid value for the <a id="ref-for-size-features②"></a>[size feature](#size-features), it is handled the same as other invalid feature values, and the result of the <a id="ref-for-size-features③"></a>size feature is unknown.

<a id="example-031e4923"></a>

<strong>Example:</strong>

[](#example-031e4923) For example, <a id="ref-for-query-container③⑤"></a>[query containers](https://drafts.csswg.org/css-conditional-5/#query-container) with different font-sizes will evaluate <a id="ref-for-em"></a>[em](https://www.w3.org/TR/css-values-4/#em)-based queries relative to their own font sizes:

``` text
aside, main {
  container-type: inline-size;
}

aside { font-size: 16px; }
main { font-size: 24px; }

@container (width > 40em) {
  h2 { font-size: 1.5em; }
}
```

The 40em value used in the query condition is relative to the <a id="ref-for-computed-value③"></a>[computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of <a id="ref-for-propdef-font-size②"></a>[font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) on the relevant <a id="ref-for-query-container③⑥"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container):

- For any h2 inside aside, the query condition will be true above 640px.

- For any h2 inside main, the query condition will be true above 960px.

<a id="example-5df63e7c"></a>

<strong>Example:</strong>

[](#example-5df63e7c) Similarly, <a id="ref-for-query-container③⑦"></a>[query containers](https://drafts.csswg.org/css-conditional-5/#query-container) will evaluate <a id="ref-for-funcdef-var"></a>[var()](https://www.w3.org/TR/css-variables-1/#funcdef-var)-based queries relative to their own <a id="ref-for-computed-value④"></a>[computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the <a id="ref-for-custom-property②"></a>[custom property](https://www.w3.org/TR/css-variables-1/#custom-property):

``` text
aside, main {
  container-type: inline-size;
}

aside { --query: 300px; }
main { --query: 500px; }

@container (width > var(--query)) {
  h2 { font-size: 1.5em; }
}
```

The var(--query) value used in the query condition is substituted with the <a id="ref-for-computed-value⑤"></a>[computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the --query <a id="ref-for-custom-property③"></a>[custom property](https://www.w3.org/TR/css-variables-1/#custom-property) on the relevant <a id="ref-for-query-container③⑧"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container):

- For any h2 inside aside, the query condition will be true above 300px.

- For any h2 inside main, the query condition will be true above 500px.

#### <a id="width"></a>6.1.1.  Width: the <a id="ref-for-descdef-container-width"></a>[width](#descdef-container-width) feature[](#width)

| Field                   | Definition                                                                                                                   |
|-------------------------|------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-width"></a><strong>width</strong>                                                                   |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container⑤"></a>[&#64;container](https://drafts.csswg.org/css-conditional-5/#at-ruledef-container) |
| <strong>Value:</strong> | <a id="ref-for-length-value"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)                              |
| <strong>Type:</strong>  | range                                                                                                                        |

The <a id="ref-for-descdef-container-width①"></a>[width](#descdef-container-width) <a id="ref-for-container-feature④"></a>[container feature](https://drafts.csswg.org/css-conditional-5/#container-feature) queries the <a id="ref-for-width"></a>[width](https://www.w3.org/TR/css-sizing-3/#width) of the <a id="ref-for-query-container③⑨"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-content-box"></a>[content box](https://www.w3.org/TR/css-box-4/#content-box).

#### <a id="height"></a>6.1.2.  Height: the <a id="ref-for-descdef-container-height"></a>[height](#descdef-container-height) feature[](#height)

| Field                   | Definition                                                                                                                   |
|-------------------------|------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-height"></a><strong>height</strong>                                                                 |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container⑥"></a>[&#64;container](https://drafts.csswg.org/css-conditional-5/#at-ruledef-container) |
| <strong>Value:</strong> | <a id="ref-for-length-value①"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)                             |
| <strong>Type:</strong>  | range                                                                                                                        |

The <a id="ref-for-descdef-container-height①"></a>[height](#descdef-container-height) <a id="ref-for-container-feature⑤"></a>[container feature](https://drafts.csswg.org/css-conditional-5/#container-feature) queries the <a id="ref-for-height"></a>[height](https://www.w3.org/TR/css-sizing-3/#height) of the <a id="ref-for-query-container④⓪"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-content-box①"></a>[content box](https://www.w3.org/TR/css-box-4/#content-box).

#### <a id="inline-size"></a>6.1.3.  Inline-size: the <a id="ref-for-descdef-container-inline-size④"></a>[inline-size](#descdef-container-inline-size) feature[](#inline-size)

| Field                   | Definition                                                                                                                   |
|-------------------------|------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-inline-size"></a><strong>inline-size</strong>                                                       |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container⑦"></a>[&#64;container](https://drafts.csswg.org/css-conditional-5/#at-ruledef-container) |
| <strong>Value:</strong> | <a id="ref-for-length-value②"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)                             |
| <strong>Type:</strong>  | range                                                                                                                        |

The <a id="ref-for-descdef-container-inline-size⑤"></a>[inline-size](#descdef-container-inline-size) <a id="ref-for-container-feature⑥"></a>[container feature](https://drafts.csswg.org/css-conditional-5/#container-feature) queries the <a id="ref-for-size"></a>[size](https://www.w3.org/TR/css-sizing-3/#size) of the <a id="ref-for-query-container④①"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-content-box②"></a>[content box](https://www.w3.org/TR/css-box-4/#content-box) in the <a id="ref-for-query-container④②"></a>query container’s <a id="ref-for-inline-axis②"></a>[inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

#### <a id="block-size"></a>6.1.4.  Block-size: the <a id="ref-for-descdef-container-block-size"></a>[block-size](#descdef-container-block-size) feature[](#block-size)

| Field                   | Definition                                                                                                                   |
|-------------------------|------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-block-size"></a><strong>block-size</strong>                                                         |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container⑧"></a>[&#64;container](https://drafts.csswg.org/css-conditional-5/#at-ruledef-container) |
| <strong>Value:</strong> | <a id="ref-for-length-value③"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)                             |
| <strong>Type:</strong>  | range                                                                                                                        |

The <a id="ref-for-descdef-container-block-size①"></a>[block-size](#descdef-container-block-size) <a id="ref-for-container-feature⑦"></a>[container feature](https://drafts.csswg.org/css-conditional-5/#container-feature) queries the <a id="ref-for-size①"></a>[size](https://www.w3.org/TR/css-sizing-3/#size) of the <a id="ref-for-query-container④③"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-content-box③"></a>[content box](https://www.w3.org/TR/css-box-4/#content-box) in the <a id="ref-for-query-container④④"></a>query container’s <a id="ref-for-block-axis①"></a>[block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis).

#### <a id="aspect-ratio"></a>6.1.5.  Aspect-ratio: the <a id="ref-for-descdef-container-aspect-ratio"></a>[aspect-ratio](#descdef-container-aspect-ratio) feature[](#aspect-ratio)

| Field                   | Definition                                                                                                                   |
|-------------------------|------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-aspect-ratio"></a><strong>aspect-ratio</strong>                                                     |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container⑨"></a>[&#64;container](https://drafts.csswg.org/css-conditional-5/#at-ruledef-container) |
| <strong>Value:</strong> | <a id="ref-for-ratio-value"></a>[\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value)                                 |
| <strong>Type:</strong>  | range                                                                                                                        |

The <a id="ref-for-descdef-container-aspect-ratio①"></a>[aspect-ratio](#descdef-container-aspect-ratio) <a id="ref-for-container-feature⑧"></a>[container feature](https://drafts.csswg.org/css-conditional-5/#container-feature) is defined as the ratio of the value of the <a id="ref-for-descdef-container-width②"></a>[width](#descdef-container-width) <a id="ref-for-container-feature⑨"></a>container feature to the value of the <a id="ref-for-descdef-container-height②"></a>[height](#descdef-container-height) <a id="ref-for-container-feature①⓪"></a>container feature.

#### <a id="orientation"></a>6.1.6.  Orientation: the <a id="ref-for-descdef-container-orientation"></a>[orientation](#descdef-container-orientation) feature[](#orientation)

| Field                   | Definition                                                                                                                    |
|-------------------------|-------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-orientation"></a><strong>orientation</strong>                                                        |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container①⓪"></a>[&#64;container](https://drafts.csswg.org/css-conditional-5/#at-ruledef-container) |
| <strong>Value:</strong> | portrait <a id="ref-for-comb-one③③"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) landscape                          |
| <strong>Type:</strong>  | discrete                                                                                                                      |

<strong><a id="valdef-container-orientation-portrait"></a><strong>portrait</strong></strong>

The <a id="ref-for-descdef-container-orientation①"></a>[orientation](#descdef-container-orientation) <a id="ref-for-container-feature①①"></a>[container feature](https://drafts.csswg.org/css-conditional-5/#container-feature) is <a id="ref-for-valdef-container-orientation-portrait"></a>[portrait](#valdef-container-orientation-portrait) when the value of the <a id="ref-for-descdef-container-height③"></a>[height](#descdef-container-height) <a id="ref-for-container-feature①②"></a>container feature is greater than or equal to the value of the <a id="ref-for-descdef-container-width③"></a>[width](#descdef-container-width) <a id="ref-for-container-feature①③"></a>container feature.

<strong><a id="valdef-container-orientation-landscape"></a><strong>landscape</strong></strong>

Otherwise <a id="ref-for-descdef-container-orientation②"></a>[orientation](#descdef-container-orientation) is <a id="ref-for-valdef-container-orientation-landscape"></a>[landscape](#valdef-container-orientation-landscape).

## <a id="container-lengths"></a>7.  Container Relative Lengths: the cqw, cqh, cqi, cqb, cqmin, cqmax units[](#container-lengths)

<a id="container-query-length"></a><strong>Container query length units</strong> specify a length relative to the dimensions of a <a id="ref-for-query-container⑥⑤"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container). Style sheets that use <a id="ref-for-container-query-length②"></a>[container query length](#container-query-length) units can more easily move components from one <a id="ref-for-query-container⑥⑥"></a>query container to another.

The <a id="ref-for-container-query-length③"></a>[container query length](#container-query-length) units are:

Informative Summary of Container Units

| unit  | relative to                                                                                                                                                                                                                   |
|-------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| cqw   | 1% of a <a id="ref-for-query-container⑥⑦"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-width①"></a>[width](https://www.w3.org/TR/css-sizing-3/#width)                  |
| cqh   | 1% of a <a id="ref-for-query-container⑥⑧"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-height①"></a>[height](https://www.w3.org/TR/css-sizing-3/#height)               |
| cqi   | 1% of a <a id="ref-for-query-container⑥⑨"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-inline-size"></a>[inline size](https://www.w3.org/TR/css-sizing-3/#inline-size) |
| cqb   | 1% of a <a id="ref-for-query-container⑦⓪"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-block-size"></a>[block size](https://www.w3.org/TR/css-sizing-3/#block-size)    |
| cqmin | The smaller value of cqi or cqb                                                                                                                                                                                               |
| cqmax | The larger value of cqi or cqb                                                                                                                                                                                                |

For each element, <a id="ref-for-container-query-length④"></a>[container query length](#container-query-length) units are evaluated using the same rules as <a id="ref-for-container-size-query⑨"></a>[container size queries](#container-size-query) on the relevant axis (or axes) described by the unit. The <a id="ref-for-query-container⑦①"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container) for each axis is the nearest ancestor container that accepts <a id="ref-for-container-size-query①⓪"></a>container size queries on that axis. If no eligible <a id="ref-for-query-container⑦②"></a>query container is available, then use the <a id="ref-for-small-viewport-size"></a>[small viewport size](https://www.w3.org/TR/css-values-4/#small-viewport-size) for that axis.

Note: In some cases cqi and cqb units on the same element will evaluate in relation to different <a id="ref-for-query-container⑦③"></a>[query containers](https://drafts.csswg.org/css-conditional-5/#query-container). Similarly, cqmin and cqmax units represent the larger or smaller of the cqi and cqb units, even when those dimensions come from different <a id="ref-for-query-container⑦④"></a>query containers.

Child elements do not inherit the relative values as specified for their parent; they inherit the <a id="ref-for-computed-value①②"></a>[computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

<a id="example-a4252068"></a>

<strong>Example:</strong>

[](#example-a4252068) Authors can ensure that <a id="ref-for-container-query-length⑤"></a>[container query length](#container-query-length) units have an appropriate <a id="ref-for-query-container⑦⑤"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container) by applying them inside a <a id="ref-for-container-query①④"></a>[container query](https://drafts.csswg.org/css-conditional-5/#container-query) that relies on the same container-type. Custom fallback values can be defined outside the <a id="ref-for-container-query①⑤"></a>container query:

``` text
/* The fallback value does not rely on containment */
h2 { font-size: 1.2em; }

@container (inline-size >= 0px) {
  /* only applies when an inline-size container is available */
  h2 { font-size: calc(1.2em + 1cqi); }
}
```
