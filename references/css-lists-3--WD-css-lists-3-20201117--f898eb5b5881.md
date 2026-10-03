Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Lists and Counters Module Level 3](https://www.w3.org/TR/2020/WD-css-lists-3-20201117/).

Original copyright notice: Copyright © 2020 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Lists and Counters Module Level 3

Source snapshot: https://www.w3.org/TR/2020/WD-css-lists-3-20201117/

Snapshot SHA-256: f898eb5b58818862a598d23d225da2ca3a58396b7bb32c9515c2cc8e31470ccf

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 10 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Lists and Counters Module Level 3

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2020 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This module contains CSS features related to list counters: styling them, positioning them, and manipulating their value.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	Other documents may supersede this document.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/Style/CSS/) as a <strong>Working Draft</strong>. Publication as a Working Draft does not imply endorsement by the W3C Membership.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-lists” in the title, like this: “\[css-lists\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/public-css-archive/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-lists%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [15 September 2020 W3C Process Document](https://www.w3.org/2020/Process-20200915/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20170801/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20170801/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20170801/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<a id="ref-for-selectordef-marker"></a>

<a id="ref-for-valdef-counter-increment-list-item"></a>

<a id="ref-for-display-type"></a>

This specification defines the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element, the [list-item](#valdef-counter-increment-list-item) [display type](https://www.w3.org/TR/css-display-3/#display-type) that generates markers, and several properties controlling the placement and styling of markers.

<a id="ref-for-counter"></a>

It also defines [counters](#counter), which are special numerical objects often used to generate the default contents of markers.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f930c7d5"></a> For instance, the following example illustrates how markers can be used to add parentheses around each numbered list item:
>
> ```text
> <style>
> li::marker { content: "(" counter(list-item, lower-roman) ")"; }
> li { display: list-item; }
> </style>
> <ol>
>   <li>This is the first item.
>   <li>This is the second item.
>   <li>This is the third item.
> </ol>
> ```
>
> It should produce something like this:
>
> ```text
>   (i) This is the first item.
>  (ii) This is the second item.
> (iii) This is the third item.
> ```
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: Note that this example is far more verbose than is usually needed in HTML, as the UA default style sheet takes care of most of the necessary styling.

With descendant selectors and child selectors, it’s possible to specify different marker types depending on the depth of embedded lists.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="declaring-a-list-item"></a>2.  Declaring a List Item

<a id="ref-for-propdef-display"></a>

<a id="ref-for-valdef-display-list-item"></a>

<a id="ref-for-list-item"></a>

<a id="ref-for-selectordef-marker①"></a>

<a id="ref-for-valdef-counter-increment-list-item①"></a>

<a id="ref-for-counter①"></a>

A <a id="list-item"></a>list item is any element with its [display](https://www.w3.org/TR/css-display-3/#propdef-display) property set to [list-item](https://www.w3.org/TR/css-display-3/#valdef-display-list-item). [List items](#list-item) generate [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-elements; no other elements do. Additionally, <a id="ref-for-list-item①"></a>list items automatically increment an implied [list-item](#valdef-counter-increment-list-item) [counter](#counter) (see [§ 4.6 The Implicit list-item Counter](#list-item-counter)).

## <a id="markers"></a>3.  Markers

<a id="ref-for-list-item②"></a>

<a id="ref-for-display-type①"></a>

<a id="ref-for-marker"></a>

<a id="ref-for-propdef-list-style-type"></a>

<a id="ref-for-propdef-list-style-image"></a>

<a id="ref-for-selectordef-marker②"></a>

The defining feature of the [list item](#list-item) [display type](https://www.w3.org/TR/css-display-3/#display-type) is its <a id="marker"></a>marker, a symbol or ordinal that helps denote the beginning of each <a id="ref-for-list-item③"></a>list item in a list. In the CSS layout model, <a id="ref-for-list-item④"></a>list item [markers](#marker) are represented by a <a id="ref-for-marker①"></a>marker box associated with each <a id="ref-for-list-item⑤"></a>list item. The contents of this <a id="ref-for-marker②"></a>marker can be controlled with the [list-style-type](#propdef-list-style-type) and [list-style-image](#propdef-list-style-image) properties on the <a id="ref-for-list-item⑥"></a>list item and by assigning properties to its [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element.

<a id="ref-for-selectordef-marker③"></a>

### <a id="marker-pseudo"></a>3.1.  The [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) Pseudo-Element

<a id="ref-for-marker③"></a>

<a id="ref-for-selectordef-marker④"></a>

<a id="ref-for-list-item⑦"></a>

<a id="ref-for-selectordef-before"></a>

The [marker box](#marker) is generated by the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element of a [list item](#list-item) as the <a id="ref-for-list-item⑧"></a>list item’s first child, before the [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) pseudo-element (if it exists on the element). It is filled with content as defined in [§ 3.2 Generating Marker Contents](#content-property).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-062e4ca0"></a> In this example, markers are used to number paragraphs that are designated as "notes":
>
> ```text
> <style>
> p { margin-left: 12 em; }
> p.note {
>   display: list-item;
>   counter-increment: note-counter;
> }
> p.note::marker {
>   content: "Note " counter(note-counter) ":";
> }
> </style>
> <p>This is the first paragraph in this document.
> <p class="note">This is a very short document.
> <p>This is the end.
> ```
>
> It should render something like this:
>
> ```text
>         This is the first paragraph
>         in this document.
> 
> Note 1: This is a very short
>         document.
> 
>         This is the end.
> ```
<a id="ref-for-selectordef-marker⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-203ca64c"></a> By using the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element, a list’s markers can be styled independently from the text of the list item itself:
>
> ```text
> <style>
> p { margin-left: 8em } /* Make space for counters */
> li { list-style-type: lower-roman; }
> li::marker { color: blue; font-weight:bold; }
> </style>
> <p>This is a long preceding paragraph ...
> <ol>
>   <li>This is the first item.
>   <li>This is the second item.
>   <li>This is the third item.
> </ol>
> <p>This is a long following paragraph ...
> ```
>
> The preceding document should render something like this:
>
> ```text
>        This is a long preceding
>        paragraph ...
> 
>   i.   This is the first item.
>  ii.   This is the second item.
> iii.   This is the third item.
> 
>        This is a long following
>        paragraph ...
> ```
>
> Previously the only way to style a marker was through inheritance; one had to put the desired marker styling on the list item, and then revert that on a wrapper element around the list item’s actual contents.

<a id="ref-for-marker④"></a>

<a id="ref-for-list-item⑨"></a>

<a id="ref-for-selectordef-marker⑥"></a>

<a id="ref-for-propdef-content"></a>

<a id="ref-for-valdef-content-none"></a>

[Marker boxes](#marker) only exist for [list items](#list-item): on any other element, the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element’s [content](https://www.w3.org/TR/css-content-3/#propdef-content) property must compute to [none](https://www.w3.org/TR/css-content-3/#valdef-content-none), which suppresses its creation.

<a id="ref-for-selectordef-marker⑦"></a>

#### <a id="marker-properties"></a>3.1.1.  Properties Applying to [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker)

<a id="ref-for-selectordef-marker⑧"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-marker⑤"></a>

All properties can be set on a [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element and will have a [computed value](https://www.w3.org/TR/css-cascade-4/#computed-value); however, only the following CSS properties actually apply to a [marker box](#marker):

- <a id="ref-for-propdef-direction"></a>

  <a id="ref-for-propdef-unicode-bidi"></a>

  <a id="ref-for-propdef-text-combine-upright"></a>

  the [text-combine-upright](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-combine-upright), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), and [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) properties (see [\[CSS-WRITING-MODES-3\]](#biblio-css-writing-modes-3))

- <a id="ref-for-propdef-content①"></a>

  the [content](https://www.w3.org/TR/css-content-3/#propdef-content) property (see [§ 3.2 Generating Marker Contents](#content-property), below)

- all animation and transition properties (see [\[CSS-ANIMATIONS-1\]](#biblio-css-animations-1) and [\[CSS-TRANSITIONS-1\]](#biblio-css-transitions-1))

> <strong data-conversion-semantic="note">Note</strong>
>
> It is expected that future specifications will extend this list of properties; however at the moment outside marker box layout is not fully defined, so only these properties are allowed.

<a id="ref-for-marker⑥"></a>

<a id="ref-for-cascade-origin-author"></a>

<a id="ref-for-cascade"></a>

<a id="ref-for-cascade-origin-ua"></a>

<a id="ref-for-selectordef-marker⑨"></a>

Other properties must not have an effect on the [marker box](#marker) when set in the [author origin](https://www.w3.org/TR/css-cascade-4/#cascade-origin-author) of the [cascade](https://www.w3.org/TR/css-cascade-4/#cascade). UAs may either treat such properties as not applying, or enforce their value by setting a [user-agent origin](https://www.w3.org/TR/css-cascade-4/#cascade-origin-ua) !important rule. However, inheritable properties that apply to text can be set on the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element: these will inherit to and take effect on its text contents.

<a id="ref-for-selectordef-marker①⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6b44c445"></a> Examples of properties that apply to text, and therefore to the contents of [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) when declared on <a id="ref-for-selectordef-marker①①"></a>::marker:
>
> - <a id="ref-for-propdef-letter-spacing"></a>
>
>   <a id="ref-for-propdef-text-transform"></a>
>
>   <a id="ref-for-propdef-white-space"></a>
>
>   [white-space](https://www.w3.org/TR/css-text-3/#propdef-white-space), [text-transform](https://www.w3.org/TR/css-text-3/#propdef-text-transform), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing) (see [\[CSS-TEXT-3\]](#biblio-css-text-3))
>
> - all font properties (see [\[CSS-FONTS-3\]](#biblio-css-fonts-3) and its successors)
>
> - <a id="ref-for-propdef-color"></a>
>
>   the [color](https://www.w3.org/TR/css-color-4/#propdef-color) property (see [\[CSS-COLOR-3\]](#biblio-css-color-3))

UAs must add the following rule to their default style sheet:

```text
::marker, ::before::marker, ::after::marker {
  unicode-bidi: isolate;
  font-variant-numeric: tabular-nums;
  white-space: pre;
  text-transform: none;
}
```
<a id="ref-for-selectordef-marker①②"></a>

<a id="ref-for-marker⑦"></a>

<a id="ref-for-selectordef-before①"></a>

<a id="ref-for-selectordef-after"></a>

<a id="ref-for-compound"></a>

<a id="ref-for-originating-element"></a>

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-selector"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element can represent the [marker box](#marker) of a [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) or [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudo-element, the [compound selector](https://www.w3.org/TR/selectors-4/#compound) <a id="ref-for-selectordef-marker①③"></a>::marker, which expands to \*::marker [\[SELECTORS-4\]](#biblio-selectors-4), will not select these markers—an [originating element](https://www.w3.org/TR/selectors-4/#originating-element) that is a [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) needs to be explicitly specified in the [selector](https://www.w3.org/TR/selectors-4/#selector), e.g. ::before::marker.

<a id="ref-for-propdef-white-space①"></a>

<a id="ref-for-propdef-text-space-collapse"></a>

<a id="ref-for-propdef-text-space-trim"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-04da870c"></a> [white-space: pre](https://www.w3.org/TR/css-text-3/#propdef-white-space) doesn’t have quite the right behavior; [text-space-collapse: preserve-spaces](https://drafts.csswg.org/css-text-4/#propdef-text-space-collapse) + [text-space-trim: discard-after](https://drafts.csswg.org/css-text-4/#propdef-text-space-trim) might be closer to what’s needed here. See discussion in [Issue 4448](https://github.com/w3c/csswg-drafts/issues/4448) and [Issue 4891](https://github.com/w3c/csswg-drafts/issues/4891).

### <a id="content-property"></a>3.2.  Generating Marker Contents

<a id="ref-for-marker⑧"></a>

The contents of a [marker box](#marker) are determined by the first of these conditions that is true:

<a id="ref-for-valdef-content-normal"></a>

<a id="ref-for-selectordef-marker①④"></a>

<a id="ref-for-propdef-content②"></a>

[content](https://www.w3.org/TR/css-content-3/#propdef-content) on the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) itself is not [normal](https://www.w3.org/TR/css-content-3/#valdef-content-normal)

<a id="ref-for-selectordef-before②"></a>

<a id="ref-for-propdef-content③"></a>

<a id="ref-for-marker⑨"></a>

The contents of the [marker box](#marker) are determined as defined by the [content](https://www.w3.org/TR/css-content-3/#propdef-content) property, exactly as for [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before).

<a id="ref-for-marker-image"></a>

<a id="ref-for-originating-element①"></a>

<a id="ref-for-propdef-list-style-image①"></a>

[list-style-image](#propdef-list-style-image) on the [originating element](https://www.w3.org/TR/selectors-4/#originating-element) defines a [marker image](#marker-image)

<a id="ref-for-text-run"></a>

<a id="ref-for-marker-image①"></a>

<a id="ref-for-replaced-element"></a>

<a id="ref-for-inline"></a>

<a id="ref-for-anonymous"></a>

<a id="ref-for-marker①⓪"></a>

The '[marker box](#marker) contains an [anonymous](https://www.w3.org/TR/css-display-3/#anonymous) [inline](https://www.w3.org/TR/css-display-3/#inline) [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) representing the specified [marker image](#marker-image), followed by a [text run](https://www.w3.org/TR/css-display-3/#text-run) consisting of a single space (U+0020 SPACE).

<a id="ref-for-marker-string"></a>

<a id="ref-for-originating-element②"></a>

<a id="ref-for-propdef-list-style-type①"></a>

[list-style-type](#propdef-list-style-type) on the [originating element](https://www.w3.org/TR/selectors-4/#originating-element) defines a [marker string](#marker-string)

<a id="ref-for-marker-string①"></a>

<a id="ref-for-text-run①"></a>

<a id="ref-for-marker①①"></a>

The [marker box](#marker) contains a [text run](https://www.w3.org/TR/css-display-3/#text-run) consisting of the specified [marker string](#marker-string).

otherwise

<a id="ref-for-selectordef-marker①⑤"></a>

<a id="ref-for-marker①②"></a>

The [marker box](#marker) has no contents and [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) does not generate a box.

<a id="ref-for-forced-line-break"></a>

Additionally, the UA may transform into spaces or discard any preserved [forced line breaks](https://www.w3.org/TR/css-text-3/#forced-line-break).

<a id="ref-for-propdef-list-style-image②"></a>

### <a id="image-markers"></a>3.3.  Image Markers: the [list-style-image](#propdef-list-style-image) property

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-list-style-image"></a>list-style-image

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one"></a>

<a id="ref-for-typedef-image"></a>

[\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) [\|](https://www.w3.org/TR/css-values-4/#comb-one) none

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-list-item①⓪"></a>

[list items](#list-item)

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

<a id="ref-for-typedef-image①"></a>

<a id="ref-for-valdef-list-style-image-none"></a>

the keyword [none](#valdef-list-style-image-none)or the computed [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<a id="ref-for-list-item①①"></a>

<a id="ref-for-marker①③"></a>

<a id="ref-for-propdef-content④"></a>

<a id="ref-for-valdef-content-normal①"></a>

Specifies the <a id="marker-image"></a>marker image, which is used to fill the [list item’s](#list-item) [marker](#marker) when its [content](https://www.w3.org/TR/css-content-3/#propdef-content) is [normal](https://www.w3.org/TR/css-content-3/#valdef-content-normal). The values are as follows:

<a id="ref-for-typedef-image②"></a>

<a id="valdef-list-style-image-image"></a>[\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)

<a id="ref-for-marker-image②"></a>

<a id="ref-for-invalid-image"></a>

<a id="ref-for-typedef-image③"></a>

If the [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) represents a [valid image](https://www.w3.org/TR/css-images-4/#invalid-image), specifies the element’s [marker image](#marker-image) as the <a id="ref-for-typedef-image④"></a>\<image\>. Otherwise, the element has no <a id="ref-for-marker-image③"></a>marker image.

<a id="valdef-list-style-image-none"></a>none

<a id="ref-for-marker-image④"></a>

The element has no [marker image](#marker-image).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f0b1fc19"></a> The following example sets the marker at the beginning of each list item to be the image "ellipse.png".
>
> ```text
> li { list-style-image: url("http://www.example.com/ellipse.png") }
> ```
<a id="ref-for-propdef-list-style-type②"></a>

### <a id="text-markers"></a>3.4.  Text-based Markers: the [list-style-type](#propdef-list-style-type) property

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-list-style-type"></a>list-style-type

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-string-value"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-typedef-counter-style"></a>

[\<counter-style\>](https://www.w3.org/TR/css-counter-styles-3/#typedef-counter-style) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<string\>](https://www.w3.org/TR/css-values-3/#string-value) <a id="ref-for-comb-one②"></a>\| none

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

disc

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-list-item①②"></a>

[list items](#list-item)

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

specified value

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<a id="ref-for-list-item①③"></a>

<a id="ref-for-marker①④"></a>

<a id="ref-for-propdef-content⑤"></a>

<a id="ref-for-valdef-content-normal②"></a>

<a id="ref-for-marker-image⑤"></a>

Specifies the <a id="marker-string"></a>marker string, which is used to fill the [list item](#list-item)’s [marker](#marker) when its [content](https://www.w3.org/TR/css-content-3/#propdef-content) value is [normal](https://www.w3.org/TR/css-content-3/#valdef-content-normal) and there is no [marker image](#marker-image). The values are as follows:

<a id="ref-for-typedef-counter-style①"></a>

<a id="valdef-list-style-type-counter-style"></a>[\<counter-style\>](https://www.w3.org/TR/css-counter-styles-3/#typedef-counter-style)

<a id="ref-for-typedef-counter-style②"></a>

<a id="ref-for-valdef-counter-increment-list-item②"></a>

<a id="ref-for-marker-string②"></a>

Specifies the element’s [marker string](#marker-string) as the value of the [list-item](#valdef-counter-increment-list-item) counter represented using the specified [\<counter-style\>](https://www.w3.org/TR/css-counter-styles-3/#typedef-counter-style).

<a id="ref-for-marker-string③"></a>

<a id="ref-for-generate-a-counter"></a>

<a id="ref-for-valdef-counter-increment-list-item③"></a>

<a id="ref-for-typedef-counter-style③"></a>

<a id="ref-for-descdef-counter-style-prefix"></a>

<a id="ref-for-descdef-counter-style-suffix"></a>

<a id="ref-for-decimal"></a>

Specifically, the [marker string](#marker-string) is the result of [generating a counter representation](https://www.w3.org/TR/css-counter-styles-3/#generate-a-counter) of the [list-item](#valdef-counter-increment-list-item) counter value using the specified [\<counter-style\>](https://www.w3.org/TR/css-counter-styles-3/#typedef-counter-style), prefixed by the [prefix](https://www.w3.org/TR/css-counter-styles-3/#descdef-counter-style-prefix) of the <a id="ref-for-typedef-counter-style④"></a>\<counter-style\>, and followed by the [suffix](https://www.w3.org/TR/css-counter-styles-3/#descdef-counter-style-suffix) of the <a id="ref-for-typedef-counter-style⑤"></a>\<counter-style\>. If the specified <a id="ref-for-typedef-counter-style⑥"></a>\<counter-style\> does not exist, [decimal](https://www.w3.org/TR/css-counter-styles-3/#decimal) is assumed.

<a id="ref-for-string-value①"></a>

<a id="valdef-list-style-type-string"></a>[\<string\>](https://www.w3.org/TR/css-values-3/#string-value)

<a id="ref-for-string-value②"></a>

<a id="ref-for-marker-string④"></a>

The element’s [marker string](#marker-string) is the specified [\<string\>](https://www.w3.org/TR/css-values-3/#string-value).

<a id="valdef-list-style-type-none"></a>none

<a id="ref-for-marker-string⑤"></a>

The element has no [marker string](#marker-string).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6484e67f"></a> The following examples illustrate how to set markers to various values:
>
> ```text
> ul { list-style-type: "★"; }
> /* Sets the marker to a "star" character */
> 
> p.note {
>   display: list-item;
>   list-style-type: "Note: ";
>   list-style-position: inside;
> }
> /* Gives note paragraphs a marker consisting of the string "Note: " */
> 
> ol { list-style-type: upper-roman; }
> /* Sets all ordered lists to use the upper-roman counter-style
>    (defined in the Counter Styles specification [[CSS-COUNTER-STYLES]]) */
> 
> ul { list-style-type: symbols(cyclic '○' '●'); }
> /* Sets all unordered list items to alternate between empty and
>    filled circles for their markers. */
> 
> ul { list-style-type: none; }
> /* Suppresses the marker entirely, unless list-style-image is specified
>    with a valid image. */
> ```
<a id="ref-for-propdef-list-style-position"></a>

### <a id="list-style-position-property"></a>3.5.  Positioning Markers: The [list-style-position](#propdef-list-style-position) property

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-list-style-position"></a>list-style-position

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one③"></a>

inside [\|](https://www.w3.org/TR/css-values-4/#comb-one) outside

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

outside

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-list-item①④"></a>

[list items](#list-item)

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

keyword, but see prose

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<a id="ref-for-selectordef-marker①⑥"></a>

<a id="ref-for-list-item①⑤"></a>

This property dictates whether the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) is rendered inline, or positioned just outside of the [list item](#list-item). The values are as follows:

<a id="valdef-list-style-position-inside"></a>inside  
<a id="ref-for-list-item①⑥"></a>

<a id="ref-for-selectordef-marker①⑦"></a>

No special effect. (The [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) is an inline element at the start of the [list item’s](#list-item) contents.)

<a id="list-style-position-outside"></a>outside  
<a id="ref-for-valdef-overflow-visible"></a>

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-propdef-marker-side"></a>

<a id="ref-for-writing-mode"></a>

<a id="ref-for-inline-start"></a>

<a id="ref-for-principal-box"></a>

<a id="ref-for-block-container"></a>

<a id="ref-for-list-item①⑦"></a>

If the [list item](#list-item) is a [block container](https://www.w3.org/TR/css-display-3/#block-container): the marker box is a <a id="ref-for-block-container①"></a>block container and is placed outside the [principal block box](https://www.w3.org/TR/css-display-3/#principal-box); however, the position of the list-item marker adjacent to floats is undefined. CSS does not specify the precise location of the marker box or its position in the painting order, but does require that it be placed on the [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) side of the box, using the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the box indicated by [marker-side](#propdef-marker-side). The marker box is fixed with respect to the principal block box’s border and does not scroll with the principal box’s content. A UA may hide the marker if the element’s [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is other than [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible). (This allowance may change in the future.) The size or contents of the marker box may affect the height of the principal block box and/or the height of its first line box, and in some cases may cause the creation of a new line box; this interaction is also not defined.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-ffa70ac6"></a> This is handwavey nonsense from CSS2, and needs a real definition.

<a id="ref-for-list-item①⑧"></a>

<a id="ref-for-inline-box"></a>

<a id="ref-for-valdef-list-style-position-inside"></a>

If the [list item](#list-item) is an [inline box](https://www.w3.org/TR/css-display-3/#inline-box): this value is equivalent to [inside](#valdef-list-style-position-inside).

<a id="ref-for-list-style-position-outside"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-bf311ada"></a> Alternatively, [outside](#list-style-position-outside) could lay out the marker as a previous sibling of the principal inline box.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b050971d"></a> For example:
>
> ```text
> <style>
>   ul.compact { list-style: inside; }
>   ul         { list-style: outside; }
> </style>
> <ul class=compact>
>   <li>first "inside" list item comes first</li>
>   <li>second "inside" list item comes first</li>
> </ul>
> <hr>
> <ul>
>   <li>first "outside" list item comes first</li>
>   <li>second "outside" list item comes first</li>
> </ul>
> ```
>
> The above example may be formatted as:
>
> ```text
>   * first "inside" list
>   item comes first
>   * second "inside" list
>   item comes second
> 
> ========================
> 
> * first "outside" list
>   item comes first
> * second "outside" list
>   item comes second
> ```
<a id="ref-for-propdef-list-style"></a>

### <a id="list-style-property"></a>3.6.  Styling Markers: the [list-style](#propdef-list-style) shorthand property

<strong>Table 4 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-list-style"></a>list-style

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-list-style-type③"></a>

<a id="ref-for-propdef-list-style-image③"></a>

<a id="ref-for-comb-any"></a>

<a id="ref-for-propdef-list-style-position①"></a>

[\<'list-style-position'\>](#propdef-list-style-position) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'list-style-image'\>](#propdef-list-style-image) <a id="ref-for-comb-any①"></a>\|\| [\<'list-style-type'\>](#propdef-list-style-type)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-list-item①⑨"></a>

[list items](#list-item)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-propdef-list-style①"></a>

<a id="ref-for-propdef-list-style-type④"></a>

<a id="ref-for-propdef-list-style-image④"></a>

<a id="ref-for-propdef-list-style-position②"></a>

The [list-style](#propdef-list-style) property is a shorthand notation for setting the three properties [list-style-type](#propdef-list-style-type), [list-style-image](#propdef-list-style-image), and [list-style-position](#propdef-list-style-position) at the same place in the style sheet.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7bda667c"></a> For example:
>
> ```text
> ul { list-style: upper-roman inside }  /* Any UL */
> ul ul { list-style: circle outside } /* Any UL child of a UL */
> ```
<a id="ref-for-propdef-list-style-image⑤"></a>

<a id="ref-for-propdef-list-style-type⑤"></a>

Using a value of none in the shorthand is potentially ambiguous, as none is a valid value for both [list-style-image](#propdef-list-style-image) and [list-style-type](#propdef-list-style-type). To resolve this ambiguity, a value of none in the shorthand must be applied to whichever of the two properties aren’t otherwise set by the shorthand.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4ba742b8"></a>
>
> ```text
> list-style: none disc;
> /* Sets the image to "none" and the type to "disc". */
> 
> list-style: none url(bullet.png);
> /* Sets the image to "url(bullet.png)" and the type to "none". */
> 
> list-style: none;
> /* Sets both image and type to "none". */
> 
> list-style: none disc url(bullet.png);
> /* Syntax error */
> ```
<a id="ref-for-typedef-counter-style⑦"></a>

<a id="ref-for-propdef-list-style-type⑥"></a>

<a id="ref-for-identifier-value"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<counter-style\>](https://www.w3.org/TR/css-counter-styles-3/#typedef-counter-style) values of [list-style-type](#propdef-list-style-type) can also create grammatical ambiguities. As such values are ultimately [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) values, the parsing rules in [\[CSS-VALUES-3\]](#biblio-css-values-3) apply.

<a id="ref-for-propdef-list-style②"></a>

<a id="ref-for-the-li-element"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4d590fca"></a> Although authors may specify [list-style](#propdef-list-style) information directly on list item elements (e.g., <code><a href="https://html.spec.whatwg.org/multipage/grouping-content.html#the-li-element">li</a></code> in HTML), they should do so with care. Consider the following rules:
>
> ```text
> ol.alpha li { list-style: lower-alpha; }
> ul li       { list-style: disc; }
> ```
>
> <a id="ref-for-the-ul-element"></a>
>
> <a id="ref-for-the-ol-element"></a>
>
> <a id="ref-for-the-ul-element①"></a>
>
> The above won’t work as expected. If you nest a <code><a href="https://html.spec.whatwg.org/multipage/grouping-content.html#the-ul-element">ul</a></code> into an [ol class=alpha](https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element), the first rule’s specificity will make the <code><a href="https://html.spec.whatwg.org/multipage/grouping-content.html#the-ul-element">ul</a></code>’s list items use the lower-alpha style.
>
> ```text
> ol.alpha > li { list-style: lower-alpha; }
> ul > li       { list-style: disc; }
> ```
>
> These work as intended.
>
> ```text
> ol.alpha { list-style: lower-alpha; }
> ul       { list-style: disc; }
> ```
>
> <a id="ref-for-propdef-list-style③"></a>
>
> These are even better, since inheritance will transfer the [list-style](#propdef-list-style) value to the list items.

<a id="ref-for-propdef-marker-side①"></a>

### <a id="marker-side"></a>3.7.  The [marker-side](#propdef-marker-side) property

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-marker-side"></a>marker-side

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one④"></a>

match-self [\|](https://www.w3.org/TR/css-values-4/#comb-one) match-parent

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

match-self

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-list-item②⓪"></a>

[list items](#list-item)

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

specified keyword

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<a id="ref-for-propdef-marker-side②"></a>

<a id="ref-for-list-style-position-outside①"></a>

<a id="ref-for-marker①⑤"></a>

<a id="ref-for-originating-element③"></a>

The [marker-side](#propdef-marker-side) property specifies whether an [outside](#list-style-position-outside) [marker box](#marker) is positioned based on the directionality of the list item itself (i.e. its [originating element](https://www.w3.org/TR/selectors-4/#originating-element)) or the directionality of the list container (i.e. the <a id="ref-for-originating-element④"></a>originating element’s parent). In the first case, the position of the marker can vary across items in the same list, based on the directionality assigned to each list item individually; in the second case they will all align on the same side, as determined by the directionality assigned to the list as a whole.

<a id="valdef-marker-side-match-self"></a>match-self  
<a id="ref-for-originating-element⑤"></a>

<a id="ref-for-selectordef-marker①⑧"></a>

<a id="ref-for-marker①⑥"></a>

The [marker box](#marker) is positioned using the directionality of the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker)’s [originating element](https://www.w3.org/TR/selectors-4/#originating-element).

<a id="valdef-marker-side-match-parent"></a>match-parent  
<a id="ref-for-originating-element⑥"></a>

<a id="ref-for-selectordef-marker①⑨"></a>

<a id="ref-for-marker①⑦"></a>

The [marker box](#marker) is positioned using the directionality of the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker)’s [originating element’s](https://www.w3.org/TR/selectors-4/#originating-element) parent element.

<a id="ref-for-selectordef-marker②⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-305896a3"></a> By default, elements or [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-elements position themselves according to their list item’s directionality. However, if the list item is grouped with several other list items which may have different directionality (for example, multiple \<li\>s with different "dir" attributes in an \<ol\> in HTML), it is sometimes more useful to have all the markers line up on one side, so the author can specify a single "gutter" on that side and be assured that all the markers will lie in that gutter and be visible.
>
> <a id="ref-for-propdef-marker-side③"></a>
>
> Both of the following example renderings are generated from the following HTML, with the only difference being the value of [marker-side](#propdef-marker-side) on the list:
>
> ```text
> <ul>
>   <li>english one
>   <li dir=rtl>OWT WERBEH
>   <li>english three
>   <li dir=rtl>RUOF WERBEH
> </ul>
> ```
>
> <strong>Table 6 — structured row/cell transcription</strong>
>
> <strong>Row 1</strong>
>
> <strong>Column 1 (header cell):</strong>
>
> <a id="ref-for-valdef-marker-side-match-self"></a>
>
> [match-self](#valdef-marker-side-match-self)
>
> <strong>Column 2 (header cell):</strong>
>
> <a id="ref-for-valdef-marker-side-match-parent"></a>
>
> [match-parent](#valdef-marker-side-match-parent)
>
> <strong>Row 2</strong>
>
> <strong>Column 1 (data cell):</strong>
>
> ```text
> * english one
>      OWT WERBEH *
> * english three
>     RUOF WERBEH *
> ```
>
> <strong>Column 2 (data cell):</strong>
>
> ```text
> * english one
> *    OWT WERBEH
> * english three
> *   RUOF WERBEH
> ```
<a id="ref-for-propdef-direction①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-91830b42"></a> For this order punctuation inside the marker correctly, it would also need to take the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) value of the parent. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;4202&#x3E;](https://github.com/w3c/csswg-drafts/issues/4202)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-69d20ea1"></a> There are issues open on [renaming the keywords](https://github.com/w3c/csswg-drafts/issues/5308) and on [merging with list-style-position](https://github.com/w3c/csswg-drafts/issues/4209).

## <a id="auto-numbering"></a>4.  Automatic Numbering With Counters

<a id="ref-for-counter②"></a>

<a id="ref-for-propdef-counter-increment"></a>

<a id="ref-for-propdef-counter-set"></a>

<a id="ref-for-propdef-counter-reset"></a>

<a id="ref-for-funcdef-counter"></a>

<a id="ref-for-funcdef-counters"></a>

<a id="ref-for-functional-notation"></a>

A <a id="counter"></a>counter is a special numeric tracker used, among other things, to automatically number list items in CSS. Every element has a collection of zero or more counters, which are inherited through the document tree in a way similar to inherited property values. [Counters](#counter) have a <a id="css-counter-name"></a>name and <a id="css-counter-creator"></a>creator, which identify the counter, and an integer <a id="css-counter-value"></a>value. They are created and manipulated with the <a id="counter-properties"></a>counter properties [counter-increment](#propdef-counter-increment), [counter-set](#propdef-counter-set) and [counter-reset](#propdef-counter-reset), and used with the [counter()](#funcdef-counter) and [counters()](#funcdef-counters) [functional notations](https://www.w3.org/TR/css-values-4/#functional-notation).

<a id="ref-for-typedef-counter-name"></a>

<a id="ref-for-identifier-value①"></a>

<a id="ref-for-typedef-counter-name①"></a>

<a id="ref-for-valdef-counter-reset-none"></a>

<a id="ref-for-css-invalid"></a>

Counters are referred to in CSS syntax using the <a id="typedef-counter-name"></a>[\<counter-name\>](#typedef-counter-name) type, which represents their name as a [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value). A [\<counter-name\>](#typedef-counter-name) name cannot match the keyword [none](#valdef-counter-reset-none); such an identifier is [invalid](https://www.w3.org/TR/css-syntax-3/#css-invalid) as a <a id="ref-for-typedef-counter-name②"></a>\<counter-name\>.

<a id="ref-for-counter③"></a>

Resolving [counter](#counter) values on a given element is a multi-step process:

1.  <a id="ref-for-inherit-counters"></a>

    Existing counters are [inherited](#inherit-counters) from previous elements.

2.  <a id="ref-for-instantiate-counter"></a>

    <a id="ref-for-propdef-counter-reset①"></a>

    New counters are [instantiated](#instantiate-counter) ([counter-reset](#propdef-counter-reset)).

3.  <a id="ref-for-propdef-counter-increment①"></a>

    Counter values are incremented ([counter-increment](#propdef-counter-increment)).

4.  <a id="ref-for-propdef-counter-set①"></a>

    Counter values are explicitly set ([counter-set](#propdef-counter-set)).

5.  <a id="ref-for-funcdef-counter①"></a>

    <a id="ref-for-funcdef-counters①"></a>

    Counter values are used ([counter()](#funcdef-counter)/[counters()](#funcdef-counters)).

UAs may have implementation-specific limits on the maximum or minimum value of a counter. If a counter reset, set, or increment would push the value outside of that range, the value must be clamped to that range.

<a id="ref-for-propdef-counter-reset②"></a>

### <a id="counter-reset"></a>4.1.  Creating Counters: the [counter-reset](#propdef-counter-reset) property

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-counter-reset"></a>counter-reset

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-mult-one-plus"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-integer-value"></a>

<a id="ref-for-typedef-counter-name③"></a>

\[ [\<counter-name\>](#typedef-counter-name) [\<integer\>](https://www.w3.org/TR/css-values-3/#integer-value)[?](https://www.w3.org/TR/css-values-4/#mult-opt) \][+](https://www.w3.org/TR/css-values-4/#mult-one-plus) [\|](https://www.w3.org/TR/css-values-4/#comb-one) none

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-counter-reset-none①"></a>

the keyword [none](#valdef-counter-reset-none) or a list, each item an identifier paired with an integer

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

User Agents are expected to support this property on all media, including non-visual ones.

<a id="ref-for-propdef-counter-reset③"></a>

<a id="ref-for-instantiate-counter①"></a>

<a id="ref-for-counter④"></a>

The [counter-reset](#propdef-counter-reset) property [instantiates](#instantiate-counter) new [counters](#counter) on an element and sets them to the specified integer values. Its values are defined as follows:

<a id="valdef-counter-reset-none"></a>none

This element does not create any new counters.

<a id="ref-for-integer-value①"></a>

<a id="ref-for-typedef-counter-name④"></a>

<a id="valdef-counter-reset-counter-name-integer"></a>[\<counter-name\>](#typedef-counter-name) [\<integer\>](https://www.w3.org/TR/css-values-3/#integer-value)?

<a id="ref-for-integer-value②"></a>

<a id="ref-for-typedef-counter-name⑤"></a>

<a id="ref-for-instantiate-counter②"></a>

[Instantiates](#instantiate-counter) a counter of the given [\<counter-name\>](#typedef-counter-name) with a starting value of the given [\<integer\>](https://www.w3.org/TR/css-values-3/#integer-value), defaulting to 0.

<a id="ref-for-cascade①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2636b37c"></a> Note that counter properties follow the [cascading](https://www.w3.org/TR/css-cascade-4/#cascade) rules as normal. Thus, due to cascading, the following style sheet:
>
> ```text
> h1 { counter-reset: section -1 }
> h1 { counter-reset: imagenum 99 }
> ```
>
> will only reset imagenum. To reset both counters, they have to be specified together:
>
> ```text
> H1 { counter-reset: section -1 imagenum 99 }
> ```
>
> <a id="ref-for-propdef-counter-set②"></a>
>
> <a id="ref-for-propdef-counter-increment②"></a>
>
> The same principles apply to the [counter-set](#propdef-counter-set) and [counter-increment](#propdef-counter-increment) properties. See [\[css-cascade-4\]](#biblio-css-cascade-4).

<a id="ref-for-typedef-counter-name⑥"></a>

If multiple instances of the same [\<counter-name\>](#typedef-counter-name) occur in the property value, only the last one is honored.

<a id="ref-for-propdef-counter-increment③"></a>

<a id="ref-for-propdef-counter-set③"></a>

### <a id="increment-set"></a>4.2.  Manipulating Counter Values: the [counter-increment](#propdef-counter-increment) and [counter-set](#propdef-counter-set) properties

<strong>Table 8 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-counter-increment"></a>counter-increment

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-mult-one-plus①"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-integer-value③"></a>

<a id="ref-for-typedef-counter-name⑦"></a>

\[ [\<counter-name\>](#typedef-counter-name) [\<integer\>](https://www.w3.org/TR/css-values-3/#integer-value)[?](https://www.w3.org/TR/css-values-4/#mult-opt) \][+](https://www.w3.org/TR/css-values-4/#mult-one-plus) [\|](https://www.w3.org/TR/css-values-4/#comb-one) none

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-counter-reset-none②"></a>

the keyword [none](#valdef-counter-reset-none) or a list, each item an identifier paired with an integer

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

User Agents are expected to support this property on all media, including non-visual ones.

<strong>Table 9 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-counter-set"></a>counter-set

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-mult-one-plus②"></a>

<a id="ref-for-mult-opt②"></a>

<a id="ref-for-integer-value④"></a>

<a id="ref-for-typedef-counter-name⑧"></a>

\[ [\<counter-name\>](#typedef-counter-name) [\<integer\>](https://www.w3.org/TR/css-values-3/#integer-value)[?](https://www.w3.org/TR/css-values-4/#mult-opt) \][+](https://www.w3.org/TR/css-values-4/#mult-one-plus) [\|](https://www.w3.org/TR/css-values-4/#comb-one) none

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-counter-reset-none③"></a>

the keyword [none](#valdef-counter-reset-none) or a list, each item an identifier paired with an integer

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

User Agents are expected to support this property on all media, including non-visual ones.

<a id="ref-for-propdef-counter-increment④"></a>

<a id="ref-for-propdef-counter-set④"></a>

<a id="ref-for-counter⑤"></a>

<a id="ref-for-instantiate-counter③"></a>

The [counter-increment](#propdef-counter-increment) and [counter-set](#propdef-counter-set) properties manipulate the value of existing [counters](#counter). They only [instantiate](#instantiate-counter) new counters if there is no counter of the given name on the element yet. Their values are defined as follows:

<a id="valdef-counter-set-counter-increment-none"></a>none

This element does not alter the value of any counters.

<a id="ref-for-integer-value⑤"></a>

<a id="ref-for-typedef-counter-name⑨"></a>

<a id="valdef-counter-set-counter-increment-counter-name-integer"></a>[\<counter-name\>](#typedef-counter-name) [\<integer\>](https://www.w3.org/TR/css-values-3/#integer-value)?

<a id="ref-for-integer-value⑥"></a>

<a id="ref-for-propdef-counter-increment⑤"></a>

<a id="ref-for-propdef-counter-set⑤"></a>

Sets (for [counter-set](#propdef-counter-set)) or increments (for [counter-increment](#propdef-counter-increment)) the value of the named counter on the element to/by the specified [\<integer\>](https://www.w3.org/TR/css-values-3/#integer-value). If the <a id="ref-for-integer-value⑦"></a>\<integer\> is omitted, it defaults to 1 (for <a id="ref-for-propdef-counter-increment⑥"></a>counter-increment) or 0 (for <a id="ref-for-propdef-counter-set⑥"></a>counter-set).

<a id="ref-for-instantiate-counter④"></a>

If there is not currently a counter of the given name on the element, the element [instantiates](#instantiate-counter) a new counter of the given name with a starting value of 0 before setting or incrementing its value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-838dca3e"></a>
>
> This example shows a way to number chapters and sections with "Chapter 1", "1.1", "1.2", etc.
>
> ```text
> h1::before {
>     content: "Chapter " counter(chapter) ". ";
>     counter-increment: chapter;  /* Add 1 to chapter */
>     counter-reset: section;      /* Set section to 0 */
> }
> h2::before {
>     content: counter(chapter) "." counter(section) " ";
>     counter-increment: section;
> }
> ```
<a id="ref-for-typedef-counter-name①⓪"></a>

If multiple instances of the same [\<counter-name\>](#typedef-counter-name) occur in the property value, they are all processed, in order. Thus increments will compound, but only the last set value will take effect.

### <a id="nested-counters"></a>4.3.  Nested Counters and Scope

<a id="ref-for-instantiate-counter⑤"></a>

<a id="ref-for-counter⑥"></a>

<a id="ref-for-funcdef-counter②"></a>

<a id="ref-for-innermost"></a>

<a id="ref-for-funcdef-counters②"></a>

Counters are “self-nesting”; [instantiating](#instantiate-counter) a new counter on an element which inherited an identically-named [counter](#counter) from its parent creates a new counter of the same name, nested inside the existing counter. This is important for situations like lists in HTML, where lists can be nested inside lists to arbitrary depth: it would be impossible to define uniquely named counters for each level. The [counter()](#funcdef-counter) function only retrieves the [innermost](#innermost) counter of a given name on the element, whereas the [counters()](#funcdef-counters) function uses all counters of a given name that contain the element.

<a id="ref-for-counter⑦"></a>

<a id="ref-for-instantiate-counter⑥"></a>

<a id="ref-for-propdef-counter-reset④"></a>

The <a id="counter-scope"></a>scope of a [counter](#counter) therefore starts at the first element in the document that [instantiates](#instantiate-counter) that counter and includes the element’s descendants and its following siblings with their descendants. However, it does not include any elements in the scope of a counter with the same name created by a [counter-reset](#propdef-counter-reset) on a later sibling of the element, allowing such explicit counter instantiations to obscure those earlier siblings.

See [§ 4.4 Creating and Inheriting Counters](#creating-counters) for the exact rules governing the scope of counters and their values.

<a id="ref-for-propdef-list-style④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="counter-nesting-example"></a> The following code numbers nested list items. The result is very similar to that of setting display:list-item and [list-style: inside](#propdef-list-style) on the LI element:
>
> ```text
> ol { counter-reset: item }
> li { display: block }
> li::before { content: counter(item) ". "; counter-increment: item }
> ```
>
> <a id="ref-for-the-ol-element①"></a>
>
> In this example, an [ol](https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element) will create a counter, and all children of the <a id="ref-for-the-ol-element②"></a>ol will refer to that counter.
>
> If we denote the <var>n</var><sup>th</sup> instance of the item counter by item<sub><var>n</var></sub>, then the following HTML fragment will use the indicated counters.
>
> `<ol>` item<sub>0</sub> is created, set to 0
>
> `<li>` item<sub>0</sub> is incremented to 1
>
> `<li>` item<sub>0</sub> is incremented to 2
>
> `<ol>` item<sub>1</sub> is created, set to 0, nested in item<sub>0</sub>
>
> `<li>` item<sub>1</sub> is incremented to 1
>
> `<li>` item<sub>1</sub> is incremented to 2
>
> `<li>` item<sub>1</sub> is incremented to 3
>
> `<ol>` item<sub>2</sub> is created, set to 0, nested in item<sub>1</sub>
>
> `<li>` item<sub>2</sub> is incremented to 1
>
> `</ol>`
>
> `<li>` item<sub>1</sub> is incremented to 4
>
> `<ol>` item<sub>3</sub> is created, set to 0, nested in item<sub>1</sub>
>
> `<li>` item<sub>3</sub> is incremented to 1
>
> `</ol>`
>
> `<li>` item<sub>1</sub> is incremented to 5
>
> `</ol>`
>
> `<li>` item<sub>0</sub> is incremented to 3
>
> `<li>` item<sub>0</sub> is incremented to 4
>
> `</ol>`
>
> `<ol>` item<sub>4</sub> is created, set to 0
>
> `<li>` item<sub>4</sub> is incremented to 1
>
> `<li>` item<sub>4</sub> is incremented to 2
>
> `</ol>`

### <a id="creating-counters"></a>4.4.  Creating and Inheriting Counters

<a id="ref-for-counter⑧"></a>

<a id="ref-for-counter-scope"></a>

<a id="ref-for-ordered-set"></a>

<a id="ref-for-tuple"></a>

<a id="ref-for-string"></a>

<a id="ref-for-css-counter-name"></a>

<a id="ref-for-css-counter-creator"></a>

<a id="ref-for-css-counter-value"></a>

Each element or pseudo-element in a document has a (possibly empty) set of [counters](#counter) in the [scope](#counter-scope) of that element, either through inheritance from another element or through instantiation on the element directly. These counters are represented as a <a id="css-counters-set"></a>CSS counters set, which is a [set](https://infra.spec.whatwg.org/#ordered-set) whose values are each a [tuple](https://infra.spec.whatwg.org/#tuple) of: a [string](https://infra.spec.whatwg.org/#string) (representing a counter’s [name](#css-counter-name)), an element (representing the counter’s [creator](#css-counter-creator)), and an integer (representing the counter’s [value](#css-counter-value)). The latest <a id="ref-for-counter⑨"></a>counter of a given name in that set represents the <a id="innermost"></a>innermost counter of that name.

#### <a id="inheriting-counters"></a>4.4.1.  Inheriting Counters

<a id="ref-for-inherit-counters①"></a>

<a id="ref-for-concept-tree-order"></a>

An element [inherits](#inherit-counters) its initial set of counters from its parent and preceding <em>sibling</em>. It then takes the values for those counters from the values of the matching counters on its preceding <em>element in <a href="https://dom.spec.whatwg.org/#concept-tree-order">tree order</a></em> (which might be its parent, its preceding sibling, or a descendant of its previous sibling). To <a id="inherit-counters"></a>inherit counters into an <var>element</var>:

1.  <a id="ref-for-concept-tree-root"></a>

    <a id="ref-for-css-counters-set"></a>

    If <var>element</var> is the [root](https://dom.spec.whatwg.org/#concept-tree-root) of its document tree, the element has an initially-empty [CSS counters set](#css-counters-set). Return.

2.  <a id="ref-for-css-counters-set①"></a>

    Let <var>element counters</var>, representing <var>element</var>’s own [CSS counters set](#css-counters-set), be a copy of the <a id="ref-for-css-counters-set②"></a>CSS counters set of <var>element</var>’s parent element.

3.  <a id="ref-for-css-counters-set③"></a>

    Let <var>sibling counters</var> be the [CSS counters set](#css-counters-set) of <var>element</var>’s preceding sibling (if it has one), or an empty <a id="ref-for-css-counters-set④"></a>CSS counters set otherwise.

    <a id="ref-for-map-iterate"></a>

    <a id="ref-for-css-counter-name①"></a>

    [For each](https://infra.spec.whatwg.org/#map-iterate) <var>counter</var> of <var>sibling counters</var>, if <var>element counters</var> does not already contain a counter with the same [name](#css-counter-name), append a copy of <var>counter</var> to <var>element counters</var>.

4.  <a id="ref-for-css-counters-set⑤"></a>

    <a id="ref-for-concept-tree-order①"></a>

    Let <var>value source</var> be the [CSS counters set](#css-counters-set) of the element immediately preceding <var>element</var> in [tree order](https://dom.spec.whatwg.org/#concept-tree-order).

    <a id="ref-for-map-iterate①"></a>

    <a id="ref-for-list-contain"></a>

    <a id="ref-for-counter①⓪"></a>

    <a id="ref-for-css-counter-name②"></a>

    <a id="ref-for-css-counter-creator①"></a>

    <a id="ref-for-css-counter-value①"></a>

    [For each](https://infra.spec.whatwg.org/#map-iterate) <var>source counter</var> of <var>value source</var>, if <var>element counters</var> [contains](https://infra.spec.whatwg.org/#list-contain) a [counter](#counter) with the same [name](#css-counter-name) and [creator](#css-counter-creator), then set the the [value](#css-counter-value) of that counter to <var>source counter</var>’s <a id="ref-for-css-counter-value②"></a>value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="counter-inheritance-example"></a> Take the following code as an example:
>
> ```text
> <ul style='counter-reset: example 0;'>
>   <li id='foo' style='counter-increment: example;'>
>     foo
>     <div id='bar' style='counter-increment: example;'>bar</div>
>   </li>
>   <li id='baz'>
>     baz
>   </li>
> </ul>
> ```
>
> <a id="ref-for-concept-tree-order②"></a>
>
> Recall that [tree order](https://dom.spec.whatwg.org/#concept-tree-order) turns a document tree into an ordered list, where an element comes before its children, and its children come before its next sibling. In other words, for a language like HTML, it’s the order in which the parser encounters start tags as it reads the document.
>
> <a id="ref-for-the-ul-element②"></a>
>
> <a id="ref-for-the-ul-element③"></a>
>
> <a id="ref-for-concept-tree-order③"></a>
>
> In here, the <code><a href="https://html.spec.whatwg.org/multipage/grouping-content.html#the-ul-element">ul</a></code> element establishes a new counter named example, and sets its value to 0. The <b>#foo</b> element, being the first child of the <code><a href="https://html.spec.whatwg.org/multipage/grouping-content.html#the-ul-element">ul</a></code>, inherits this counter. Its parent is also its immediately preceding element in [tree order](https://dom.spec.whatwg.org/#concept-tree-order), so it inherits the value 0 with it, and then immediately increments the value to 1.
>
> The same happens with the <b>#bar</b> element. It inherits the example counter from <b>#foo</b>, and inherits the value 1 from it as well and increments it to 2.
>
> <a id="ref-for-concept-tree-order④"></a>
>
> However, the <b>#baz</b> element is a bit different. It inherits the example counter from the <b>#foo</b> element, its previous sibling. However, rather than inheriting the value 1 from <b>#foo</b> along with the counter, in inherits the value 2 from <b>#bar</b>, the previous element in [tree order](https://dom.spec.whatwg.org/#concept-tree-order).
>
> This behavior allows a single counter to be used throughout a document, continuously incrementing, without the author having to worry about the nested structure of their document.

<a id="ref-for-inheritance"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Counter inheritance, like regular CSS [inheritance](https://www.w3.org/TR/css-cascade-4/#inheritance), operates on the “flattened element tree” in the context of the [\[DOM\]](#biblio-dom).

#### <a id="instantiating-counters"></a>4.4.2.  Instantiating Counters

<a id="ref-for-counter①①"></a>

<a id="ref-for-instantiate-counter⑦"></a>

<a id="ref-for-propdef-counter-reset⑤"></a>

<a id="ref-for-propdef-counter-increment⑦"></a>

<a id="ref-for-propdef-counter-set⑦"></a>

<a id="ref-for-funcdef-counter③"></a>

<a id="ref-for-funcdef-counters③"></a>

[Counters](#counter) are [instantiated](#instantiate-counter) when named in [counter-reset](#propdef-counter-reset), and also when not otherwise present if named in [counter-increment](#propdef-counter-increment), [counter-set](#propdef-counter-set), or the [counter()](#funcdef-counter) or [counters()](#funcdef-counters) notations. (Newly <a id="ref-for-instantiate-counter⑧"></a>instantiated <a id="ref-for-counter①②"></a>counters replace identically-named <a id="ref-for-counter①③"></a>counters originating from previous siblings, but are added in addition to identically-named <a id="ref-for-counter①④"></a>counters originating from ancestor elements, see [§ 4.3 Nested Counters and Scope](#nested-counters).) To <a id="instantiate-counter"></a>instantiate a counter of a given <var>name</var> on an <var>element</var> with a starting <var>value</var>:

1.  <a id="ref-for-css-counters-set⑥"></a>

    Let <var>counters</var> be <var>element</var>’s [CSS counters set](#css-counters-set).

2.  <a id="ref-for-counter①⑤"></a>

    <a id="ref-for-list-remove"></a>

    Let <var>innermost counter</var> be the last [counter](#counter) in <var>counters</var> with the name <var>name</var>. If <var>innermost counter</var>’s originating element is <var>element</var> or a previous sibling of <var>element</var>, [remove](https://infra.spec.whatwg.org/#list-remove) <var>innermost counter</var> from <var>counters</var>.

3.  <a id="ref-for-set-append"></a>

    <a id="ref-for-counter①⑥"></a>

    [Append](https://infra.spec.whatwg.org/#set-append) a new [counter](#counter) to <var>counters</var> with name <var>name</var>, originating element <var>element</var>, and initial value <var>value</var>

### <a id="counters-without-boxes"></a>4.5.  Counters in elements that do not generate boxes

<a id="ref-for-propdef-display①"></a>

<a id="ref-for-valdef-display-none"></a>

<a id="ref-for-propdef-content⑥"></a>

<a id="ref-for-valdef-content-none①"></a>

An element that does not generate a box (for example, an element with [display](https://www.w3.org/TR/css-display-3/#propdef-display) set to [none](https://www.w3.org/TR/css-display-3/#valdef-display-none), or a pseudo-element with [content](https://www.w3.org/TR/css-content-3/#propdef-content) set to [none](https://www.w3.org/TR/css-content-3/#valdef-content-none)) cannot set, reset, or increment a counter. The counter properties are still valid on such an element, but they must have no effect.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bdf0aac0"></a> For example, with the following style sheet, H2s with class "secret" do not increment count2.
>
> ```text
> h2 { counter-increment: count2; }
> h2.secret { display: none; }
> ```
<a id="ref-for-propdef-visibility"></a>

<a id="ref-for-valdef-visibility-hidden"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Other methods of “hiding” elements, such as setting [visibility](https://www.w3.org/TR/CSS2/visufx.html#propdef-visibility) to [hidden](https://drafts.csswg.org/css2/#valdef-visibility-hidden), still cause the element to generate a box, and so are not excepted here.

<a id="ref-for-replaced-element①"></a>

<a id="ref-for-the-option-element"></a>

<a id="ref-for-elementdef-rect"></a>

Whether a [replaced element’s](https://www.w3.org/TR/css-display-3/#replaced-element) descendants (such as HTML <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-option-element">option</a></code>, or SVG <code><a href="https://www.w3.org/TR/SVG2/shapes.html#elementdef-rect">rect</a></code>) can set, reset, or increment a counter is undefined.

<a id="ref-for-replaced-element②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The behavior on [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) descendants is currently undefined due to a lack of interoperability across implementations.

<a id="ref-for-valdef-counter-increment-list-item④"></a>

### <a id="list-item-counter"></a>4.6.  The Implicit [list-item](#valdef-counter-increment-list-item) Counter

<a id="ref-for-counter①⑦"></a>

<a id="ref-for-list-item②①"></a>

<a id="ref-for-marker-string⑥"></a>

<a id="ref-for-propdef-list-style-type⑦"></a>

In addition to any explicitly defined [counters](#counter) that authors write in their styles, [list items](#list-item) automatically increment a special <a id="valdef-counter-increment-list-item"></a>list-item <a id="ref-for-counter①⑧"></a>counter, which is used when generating the default [marker string](#marker-string) on <a id="ref-for-list-item②②"></a>list items (see [list-style-type](#propdef-list-style-type)).

<a id="ref-for-propdef-counter-increment⑧"></a>

<a id="ref-for-valdef-counter-increment-list-item⑤"></a>

<a id="ref-for-list-item②③"></a>

<a id="ref-for-counter①⑨"></a>

<a id="ref-for-instantiate-counter⑨"></a>

<a id="ref-for-specified-value"></a>

<a id="ref-for-computed-value①"></a>

Specifically, unless the [counter-increment](#propdef-counter-increment) property explicitly specifies a different increment for the [list-item](#valdef-counter-increment-list-item) counter, it must be incremented by 1 on every [list item](#list-item), at the same time that [counters](#counter) are normally incremented (exactly as if the <a id="ref-for-list-item②④"></a>list item had list-item 1 appended to their <a id="ref-for-propdef-counter-increment⑨"></a>counter-increment value, including side-effects such as possibly [instantiating](#instantiate-counter) a new <a id="ref-for-counter②⓪"></a>counter, etc). This does not affect the [specified](https://www.w3.org/TR/css-cascade-4/#specified-value) or [computed values](https://www.w3.org/TR/css-cascade-4/#computed-value) of <a id="ref-for-propdef-counter-increment①⓪"></a>counter-increment.

<a id="ref-for-list-item②⑤"></a>

<a id="ref-for-valdef-counter-increment-list-item⑥"></a>

<a id="ref-for-propdef-list-style-type⑧"></a>

<a id="ref-for-propdef-counter-increment①①"></a>

<a id="ref-for-valdef-counter-reset-none④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-60f85b38"></a> Because each [list item](#list-item) automatically increments the [list-item](#valdef-counter-increment-list-item) counter by 1, consecutive <a id="ref-for-list-item②⑥"></a>list items with a numeric [list-style-type](#propdef-list-style-type) will be consecutively numbered by default—even if the author sets [counter-increment](#propdef-counter-increment) to another value such as <a id="ref-for-propdef-counter-increment①②"></a>counter-increment: itemnumber or even [none](https://drafts.csswg.org/css2/#valdef-counter-reset-none). This protects the automatic <a id="ref-for-valdef-counter-increment-list-item⑦"></a>list-item counter from inadvertently being overridden by declarations intended to address other counters.
>
> <a id="ref-for-valdef-counter-increment-list-item⑧"></a>
>
> <a id="ref-for-list-item②⑦"></a>
>
> <a id="ref-for-propdef-counter-increment①③"></a>
>
> However, since the automatic [list-item](#valdef-counter-increment-list-item) increment <em>does not</em> happen if the [list item’s](#list-item) [counter-increment](#propdef-counter-increment) explicitly mentions the <a id="ref-for-valdef-counter-increment-list-item⑨"></a>list-item counter, li { counter-increment: list-item 2; } will increment <a id="ref-for-valdef-counter-increment-list-item①⓪"></a>list-item by 2 as specified, not by 3 as would happen if list-item 1 were unconditionally appended.
>
> <a id="ref-for-valdef-counter-increment-list-item①①"></a>
>
> <a id="ref-for-propdef-counter-increment①④"></a>
>
> This also allows to turn off the automatic [list-item](#valdef-counter-increment-list-item) counter increment, by overriding it explicitly, e.g. [counter-increment: list-item 0;](#propdef-counter-increment).

<a id="ref-for-valdef-counter-increment-list-item①②"></a>

<a id="ref-for-counter②①"></a>

<a id="ref-for-list-item②⑧"></a>

In all other respects, the [list-item](#valdef-counter-increment-list-item) [counter](#counter) behaves like any other <a id="ref-for-counter②②"></a>counter and can be used and manipulated by authors to adjust [list item](#list-item) styling or for other purposes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0003035e"></a>
>
> In the following example, the list is modified to count by twos:
>
> ```text
> ol.evens > li { counter-increment: list-item 2; }
> ```
>
> A three-item list would be rendered as
>
> ```text
> 2. First Item
> 4. Second Item
> 6. Third Item
> ```
<a id="ref-for-valdef-counter-increment-list-item①③"></a>

UAs and host languages should ensure that the [list-item](#valdef-counter-increment-list-item) counter values by default reflect the underlying numeric value dictated by host language semantics when setting up list item styling in their UA style sheet and presentational hint style mappings. See, e.g. [Appendix A: Sample Style Sheet For HTML](#ua-stylesheet).

<a id="ref-for-propdef-content⑦"></a>

<a id="ref-for-valdef-counter-increment-list-item①④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-598c7e61"></a> In the following example, the [content](https://www.w3.org/TR/css-content-3/#propdef-content) property is used to create tiered numbering that hooks into the [list-item](#valdef-counter-increment-list-item) counter, and thus respects any numbering changes inflicted through HTML:
>
> ```text
> ol > li::marker { content: counters(list-item,'.') '.'; }
> ```
>
> Nested lists using this rule would be rendered like
>
> ```text
> 1. First top-level item
> 5. Second top-level item, value=5
>    5.3. First second-level item, list start=3
>    5.4. Second second-level item, list start=3
>         5.4.4. First third-level item in reversed list
>         5.4.3. Second third-level item in reversed list
>         5.4.2. Third third-level item in reversed list
>         5.4.1. Fourth third-level item in reversed list
>    5.5. Third second-level item, list start=3
> 6. Third top-level item
> ```
>
> given markup such as
>
> ```text
> <ol>
>   <li>First top-level item
>   <li value=5>Second top-level item, value=5
>     <ol start=3>
>       <li>First second-level item, list start=3
>       <li>Second second-level item, list start=3
>         <ol reversed>
>           <li>First third-level item in reversed list
>           <li>Second third-level item in reversed list
>           <li>Third third-level item in reversed list
>           <li>Fourth third-level item in reversed list
>         </ol>
>     </ol>
>   <li>Third second-level item, list start=3
>   <li>Third top-level item
> </ol>
> ```
<a id="ref-for-funcdef-counter④"></a>

<a id="ref-for-funcdef-counters④"></a>

### <a id="counter-functions"></a>4.7.  Outputting Counters: the [counter()](#funcdef-counter) and [counters()](#funcdef-counters) functions

<a id="ref-for-counter②③"></a>

<a id="ref-for-funcdef-counter⑤"></a>

<a id="ref-for-funcdef-counters⑤"></a>

<a id="ref-for-used-value"></a>

[Counters](#counter) have no visible effect by themselves, but their values can be used with the [counter()](#funcdef-counter) and [counters()](#funcdef-counters) functions, whose [used values](https://www.w3.org/TR/css-cascade-4/#used-value) represent counter values as strings or images. They are defined as follows:

<a id="typedef-counter"></a>

<a id="ref-for-typedef-counter"></a>

<a id="ref-for-funcdef-counter⑥"></a>

<a id="ref-for-funcdef-counters⑥"></a>

<a id="funcdef-counter"></a>

<a id="ref-for-typedef-counter-name①①"></a>

<a id="ref-for-typedef-counter-style⑧"></a>

<a id="funcdef-counters"></a>

<a id="ref-for-typedef-counter-name①②"></a>

<a id="ref-for-string-value③"></a>

<a id="ref-for-typedef-counter-style⑨"></a>

```text
<counter> = <counter()> | <counters()>
counter()  =  counter( <counter-name>, <counter-style>? )
counters() = counters( <counter-name>, <string>, <counter-style>? )
```
<a id="ref-for-typedef-counter-style①⓪"></a>

<a id="ref-for-counter-style"></a>

<a id="ref-for-generate-a-counter①"></a>

where [\<counter-style\>](https://www.w3.org/TR/css-counter-styles-3/#typedef-counter-style) specifies the [counter style](https://www.w3.org/TR/css-counter-styles-3/#counter-style) for [generating a representation](https://www.w3.org/TR/css-counter-styles-3/#generate-a-counter) of the named counter(s) as defined in [\[css-counter-styles-3\]](#biblio-css-counter-styles-3) and

counter()  
<a id="ref-for-typedef-counter-style①①"></a>

<a id="ref-for-counter-style①"></a>

<a id="ref-for-typedef-counter-name①③"></a>

<a id="ref-for-css-counters-set⑦"></a>

<a id="ref-for-counter②④"></a>

<a id="ref-for-innermost①"></a>

Represents the value of the [innermost](#innermost) [counter](#counter) in the element’s [CSS counters set](#css-counters-set) named [\<counter-name\>](#typedef-counter-name) using the [counter style](https://www.w3.org/TR/css-counter-styles-3/#counter-style) named [\<counter-style\>](https://www.w3.org/TR/css-counter-styles-3/#typedef-counter-style).

counters()  
<a id="ref-for-string-value④"></a>

<a id="ref-for-innermost②"></a>

<a id="ref-for-typedef-counter-style①②"></a>

<a id="ref-for-counter-style②"></a>

<a id="ref-for-typedef-counter-name①④"></a>

<a id="ref-for-css-counters-set⑧"></a>

<a id="ref-for-counter②⑤"></a>

Represents the values of all the [counters](#counter) in the element’s [CSS counters set](#css-counters-set) named [\<counter-name\>](#typedef-counter-name) using the [counter style](https://www.w3.org/TR/css-counter-styles-3/#counter-style) named [\<counter-style\>](https://www.w3.org/TR/css-counter-styles-3/#typedef-counter-style), sorted in outermost-first to [innermost](#innermost)-last order and joined by the specified [\<string\>](https://www.w3.org/TR/css-values-3/#string-value).

<a id="ref-for-typedef-counter-style①③"></a>

<a id="ref-for-decimal①"></a>

In both cases, if the [\<counter-style\>](https://www.w3.org/TR/css-counter-styles-3/#typedef-counter-style) argument is omitted it defaults to [decimal](https://www.w3.org/TR/css-counter-styles-3/#decimal).

<a id="ref-for-counter②⑥"></a>

<a id="ref-for-typedef-counter-name①⑤"></a>

<a id="ref-for-funcdef-counter⑦"></a>

<a id="ref-for-funcdef-counters⑦"></a>

<a id="ref-for-instantiate-counter①⓪"></a>

If no [counter](#counter) named [\<counter-name\>](#typedef-counter-name) exists on an element where [counter()](#funcdef-counter) or [counters()](#funcdef-counters) is used, one is first [instantiated](#instantiate-counter) with a starting value of 0.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a273a37d"></a>
>
> ```text
> H1::before        { content: counter(chno, upper-latin) ". " }
> /* Generates headings like "A. A History of Discontent" */
> 
> H2::before        { content: counter(section, upper-roman) " - " }
> /* Generates headings like "II - The Discontent Part" */
> 
> BLOCKQUOTE::after { content: " [" counter(bq, decimal) "]" }
> /* Generates blockquotes that end like "... [3]" */
> 
> DIV.note::before  { content: counter(notecntr, disc) " " }
> /* Simply generates a bullet before every div.note */
> 
> P::before         { content: counter(p, none) }
> /* inserts nothing */
> ```
<a id="ref-for-funcdef-counters⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e8a9db02"></a> The following example shows a simple use of the [counters()](#funcdef-counters) function:
>
> ```text
> <ul>
>   <li>one</li>
>   <li>two
>     <ul>
>       <li>nested one</li>
>       <li>nested two</li>
>     </ul>
>   </li>
>   <li>three</li>
> </ul>
> <style>
> li::marker { content: '(' counters(list-item,'.') ') '; }
> </style>
> ```
>
> The preceding document should render something like this:
>
> ```text
> (1) one
> (2) two
>    (2.1) nested one
>    (2.2) nested two
> (3) three
> ```
<a id="ref-for-funcdef-counters⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b1a4542d"></a> Because counters inherit to siblings as well, they can be used to number headings and subheadings, which aren’t nested within each other. Unfortunately, this prevents the use of [counters()](#funcdef-counters) as counters from siblings don’t nest, but one can create multiple counters and manually concatenate them instead:
>
> ```text
> <h1>First H1</h1>
> ...
> <h2>First H2 in H1</h2>
> ...
> <h2>Second H2 in H1</h2>
> ...
> <h3>First H3 in H2</h3>
> ...
> <h1>Second H1</h1>
> ...
> <h2>First H2 in H1</h2>
> ...
> <style>
> body { counter-reset: h1 h2 h3; }
> h1   { counter-increment: h1; counter-reset: h2 h3;}
> h2   { counter-increment: h2; counter-reset:    h3; }
> h3   { counter-increment: h3; }
> h1::before { content: counter(h1,upper-alpha) ' '; }
> h2::before { content: counter(h1,upper-alpha) '.'
>                       counter(h2,decimal) ' '; }
> h3::before { content: counter(h1,upper-alpha) '.'
>                       counter(h2,decimal) '.'
>                       counter(h3,lower-roman) ' '; }
> </style>
> ```
>
> The preceding document should render something like this:
>
> ```text
> A First H1
> ...
> A.1 First H2 in H1
> ...
> A.2 Second H2 in H1
> ...
> A.2.i First H3 in H2
> ...
> B Second H1
> ...
> B.1 First H2 in H1
> ...
> ```
<a id="ref-for-propdef-order"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4b0c2d48"></a> Counters are sometimes useful for things other than printing markers. In general, they provide the ability to number elements in sequence, which can be useful for other properties to reference. For example, using [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) to put an element between two other specific elements currently requires you to explicitly put <a id="ref-for-propdef-order①"></a>order on every element before and/or after the desired insertion point. If you can set the <a id="ref-for-propdef-order②"></a>order value of everything to a counter, tho, you can more easily insert an element into an arbitrary spot between two others.
>
> Other use-cases involve nested or sibling elements with transforms that are meant to be slightly different from each other. Today you have to use a preprocessor to do this in a reasonable way, but a counter would make it work well in "plain" CSS.
>
> <a id="ref-for-custom-property"></a>
>
> <a id="ref-for-funcdef-calc"></a>
>
> (You can built up successive values in the nested case today by using [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) and stacking up nested [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc)s, but this is a \*little bit\* clumsy, and doesn’t work for siblings.)
>
> <a id="ref-for-typedef-counter-name①⑥"></a>
>
> Suggestion is to add a counter-value([\<counter-name\>](#typedef-counter-name)) function, which returns the value of the named counter as an integer, rather than returning a string.
>
> See [Issue 1026](https://github.com/w3c/csswg-drafts/issues/1026).

## <a id="ua-stylesheet"></a> Appendix A: Sample Style Sheet For HTML

<em>This section is informative, not normative.
	The <a href="#biblio-html">&#x5B;HTML&#x5D;</a> <a href="https://html.spec.whatwg.org/multipage/rendering.html#lists">Rendering</a> chapter
	defines the normative default properties that apply to HTML lists;
	this sample style sheet is provided to illustrate the CSS features
	using familiar markup conventions.</em>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-33887700"></a> Discussion of how to support `ol[reversed]` list numbering in CSS is ongoing. See, e.g. [Issue 4181](https://github.com/w3c/csswg-drafts/issues/4181).

```text
/* Set up list items */
li {
  display: list-item; /* implies 'counter-increment: list-item' */
}

/* Set up ol and ul so that they scope the list-item counter */
ol, ul {
  counter-reset: list-item;
}

/* Default list style types for lists */
ol { list-style-type: decimal; }
ul { list-style-type: toggle(disc, circle, square); }

/* The type attribute on ol and ul elements */
ul[type="disc"]   { list-style-type: disc;   }
ul[type="circle"] { list-style-type: circle; }
ul[type="square"] { list-style-type: square; }
ol[type="1"] { list-style-type: decimal;     }
ol[type="a"] { list-style-type: lower-alpha; }
ol[type="A"] { list-style-type: upper-alpha; }
ol[type="i"] { list-style-type: lower-roman; }
ol[type="I"] { list-style-type: upper-roman; }

/* The start attribute on ol elements */
ol[start] {
  counter-reset: list-item calc(attr(start integer, 1) - 1);
}

/* The value attribute on li elements */
li[value] {
  counter-set: list-item attr(value integer, 1);
}


/* Box Model Rules */
ol, ul {
  display: block;
  margin-block: 1em;
  marker-side: match-parent;
  padding-inline-start: 40px;
}
ol ol, ol ul, ul ul, ul ol {
  margin-block: 0;
}

li {
  text-align: match-parent;
}
```
## <a id="acknowledgments"></a> Acknowledgments

This specification is made possible by input from Aharon Lanin, Arron Eicholz, Brad Kemper, David Baron, Emilio Cobos Álvarez, Mats Palmgren, Oriol Brufau, Simon Sapin, Xidorn Quan

## <a id="changes"></a>Changes

This section documents the changes since previous publications.

### <a id="changes-20200709"></a>Changes since the 9 July 2020 WD

- <a id="ref-for-selectordef-marker②①"></a>

  Clarified properties that apply to [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) boxes vs. to the contents of <a id="ref-for-selectordef-marker②②"></a>::marker boxes. ([Issue 4568](https://github.com/w3c/csswg-drafts/issues/4568))

- <a id="ref-for-selectordef-marker②③"></a>

  <a id="ref-for-propdef-text-transform①"></a>

  Added [text-transform: none](https://www.w3.org/TR/css-text-3/#propdef-text-transform) to the UA default style sheet for [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker). ([Issue 4206](https://github.com/w3c/csswg-drafts/issues/4206))

- Changed counter inheritance to take from the parent first, and only take from the sibling if it’s a new counter. ([Issue 5477](https://github.com/w3c/csswg-drafts/issues/5477))

### <a id="changes-20190817"></a>Changes since the 17 August 2019 WD

- <a id="ref-for-outer-display-type"></a>

  <a id="ref-for-block-container②"></a>

  <a id="ref-for-list-style-position-outside②"></a>

  Specified that [outside](#list-style-position-outside) list markers [block containers](https://www.w3.org/TR/css-display-3/#block-container). (Their [outer display type](https://www.w3.org/TR/css-display-3/#outer-display-type) remains undefined.)

- <a id="ref-for-propdef-white-space②"></a>

  <a id="ref-for-selectordef-marker②④"></a>

  Stole list of properties applying to [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) from [\[CSS-PSEUDO-4\]](#biblio-css-pseudo-4) and added animations, transitions, and [white-space](https://www.w3.org/TR/css-text-3/#propdef-white-space).

- <a id="ref-for-selectordef-marker②⑤"></a>

  <a id="ref-for-propdef-white-space③"></a>

  Added [white-space: pre](https://www.w3.org/TR/css-text-3/#propdef-white-space) to UA default style sheet for [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker). ([Issue 4448](https://github.com/w3c/csswg-drafts/issues/4448)) Note, however, that the exact white space processing behavior of marker boxes is still being worked out.

### <a id="changes-20190425"></a>Changes since the 25 April 2019 WD

- Rewrote the [§ 4 Automatic Numbering With Counters](#auto-numbering) section for better precision, editorial clarity, and synchronization with CSS2.

### <a id="changes-20140320"></a>Changes since the 20 March 2014 WD

- <a id="ref-for-identifier-value②"></a>

  Use [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) consistently for counter names.

- <a id="ref-for-propdef-position"></a>

  Dropped [position: marker](https://www.w3.org/TR/css-position-3/#propdef-position) (marker positioning is now mostly undefined, as in CSS2).

- Completely rewrote chapter on markers to tighten it up, align with current expectations, and make editorial improvements.

- <a id="ref-for-valdef-counter-increment-list-item①⑤"></a>

  Pulled the [list-item](#valdef-counter-increment-list-item) counter definition into its own section, added examples, and made some clarifications.

- <a id="ref-for-propdef-marker-side④"></a>

  Renamed values of [marker-side](#propdef-marker-side) to match conventions from box/text alignment.

- <a id="ref-for-propdef-counter-increment①⑤"></a>

  <a id="ref-for-propdef-counter-set⑧"></a>

  Defined that [counter-set](#propdef-counter-set) is applied after [counter-increment](#propdef-counter-increment) rather than before. ([Issue 3810](https://github.com/w3c/csswg-drafts/issues/3810))

- <a id="ref-for-propdef-list-style-type⑨"></a>

  <a id="ref-for-propdef-list-style⑤"></a>

  Established the canonical order of [list-style](#propdef-list-style) serialization to put [\<'list-style-type'\>](#propdef-list-style-type) last. ([Issue 2624](https://github.com/w3c/csswg-drafts/issues/2624))

### <a id="changes-from-css2"></a>Changes From CSS Level 2

As described in the introduction section, there are significant changes in this module when compared to CSS2.1.

1.  <a id="ref-for-selectordef-marker②⑥"></a>

    The [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element has been introduced to allow styling of the list marker directly.

2.  <a id="ref-for-typedef-counter-style①④"></a>

    <a id="ref-for-string-value⑤"></a>

    <a id="ref-for-propdef-list-style-type①⓪"></a>

    [list-style-type](#propdef-list-style-type) now accepts a [\<string\>](https://www.w3.org/TR/css-values-3/#string-value) as well as the extended [\<counter-style\>](https://www.w3.org/TR/css-counter-styles-3/#typedef-counter-style) values from [\[css-counter-styles-3\]](#biblio-css-counter-styles-3)..

3.  <a id="ref-for-valdef-counter-increment-list-item①⑥"></a>

    The [list-item](#valdef-counter-increment-list-item) predefined counter identifier has been introduced.

4.  <a id="ref-for-propdef-counter-set⑨"></a>

    The [counter-set](#propdef-counter-set) property has been added.

5.  <a id="ref-for-list-item②⑨"></a>

    <a id="ref-for-inline-level"></a>

    Allowed for [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) [list items](#list-item), as introduced in [\[CSS-DISPLAY-3\]](#biblio-css-display-3).

## <a id="conformance"></a> Conformance

### <a id="conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae2b6bc0"></a>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong> UAs MUST provide an accessible alternative. </strong>

### <a id="conformance-classes"></a> Conformance classes

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

### <a id="partial"></a> Partial implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](https://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

### <a id="testing"></a> Non-experimental implementations

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [\<counter\>](#typedef-counter), in §4.7
- [counter](#counter), in §4
- [counter()](#funcdef-counter), in §4.7
- [counter-increment](#propdef-counter-increment), in §4.2
- [\<counter-name\>](#typedef-counter-name), in §4
- \<counter-name\> \<integer\>?
  - [value for counter-reset](#valdef-counter-reset-counter-name-integer), in §4.1
  - [value for counter-set counter-increment](#valdef-counter-set-counter-increment-counter-name-integer), in §4.2
- [counter properties](#counter-properties), in §4
- [counter-reset](#propdef-counter-reset), in §4.1
- [counters()](#funcdef-counters), in §4.7
- [counter scope](#counter-scope), in §4.3
- [counter-set](#propdef-counter-set), in §4.2
- [\<counter-style\>](#valdef-list-style-type-counter-style), in §3.4
- [creator](#css-counter-creator), in §4
- [CSS counters set](#css-counters-set), in §4.4
- [\<image\>](#valdef-list-style-image-image), in §3.3
- [inherit counters](#inherit-counters), in §4.4.1
- [innermost](#innermost), in §4.4
- [inside](#valdef-list-style-position-inside), in §3.5
- [instantiate](#instantiate-counter), in §4.4.2
- [instantiate counter](#instantiate-counter), in §4.4.2
- [list item](#list-item), in §2
- [list-item](#valdef-counter-increment-list-item), in §4.6
- [list-style](#propdef-list-style), in §3.6
- [list-style-image](#propdef-list-style-image), in §3.3
- [list-style-position](#propdef-list-style-position), in §3.5
- [list-style-type](#propdef-list-style-type), in §3.4
- [marker](#marker), in §3
- [marker box](#marker), in §3
- [marker image](#marker-image), in §3.3
- [marker-side](#propdef-marker-side), in §3.7
- [marker string](#marker-string), in §3.4
- [match-parent](#valdef-marker-side-match-parent), in §3.7
- [match-self](#valdef-marker-side-match-self), in §3.7
- [name](#css-counter-name), in §4
- none
  - [value for counter-reset](#valdef-counter-reset-none), in §4.1
  - [value for counter-set counter-increment](#valdef-counter-set-counter-increment-none), in §4.2
  - [value for list-style-image](#valdef-list-style-image-none), in §3.3
  - [value for list-style-type](#valdef-list-style-type-none), in §3.4
- [outside](#list-style-position-outside), in §3.5
- [scope](#counter-scope), in §4.3
- [\<string\>](#valdef-list-style-type-string), in §3.4
- [value](#css-counter-value), in §4

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-cascade-4\] defines the following terms:
  - <a id="term-for-cascade-origin-author"></a>author origin
  - <a id="term-for-cascade"></a>cascade
  - <a id="term-for-computed-value"></a>computed value
  - <a id="term-for-inheritance"></a>inheritance
  - <a id="term-for-specified-value"></a>specified value
  - <a id="term-for-used-value"></a>used value
  - <a id="term-for-cascade-origin-ua"></a>user-agent origin
- \[css-color-4\] defines the following terms:
  - <a id="term-for-propdef-color"></a>color
- \[css-content-3\] defines the following terms:
  - <a id="term-for-propdef-content"></a>content
  - <a id="term-for-valdef-content-none"></a>none
  - <a id="term-for-valdef-content-normal"></a>normal
- \[css-counter-styles-3\] defines the following terms:
  - <a id="term-for-typedef-counter-style"></a>\<counter-style\>
  - <a id="term-for-counter-style"></a>counter style
  - <a id="term-for-decimal"></a>decimal
  - <a id="term-for-generate-a-counter"></a>generate a counter representation
  - <a id="term-for-descdef-counter-style-prefix"></a>prefix
  - <a id="term-for-descdef-counter-style-suffix"></a>suffix
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="term-for-anonymous"></a>anonymous
  - <a id="term-for-block-container"></a>block container
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-display-type"></a>display type
  - <a id="term-for-inline"></a>inline
  - <a id="term-for-inline-box"></a>inline box
  - <a id="term-for-inline-level"></a>inline-level
  - <a id="term-for-valdef-display-list-item"></a>list-item
  - <a id="term-for-valdef-display-none"></a>none
  - <a id="term-for-outer-display-type"></a>outer display type
  - <a id="term-for-principal-box"></a>principal box
  - <a id="term-for-replaced-element"></a>replaced element
  - <a id="term-for-text-run"></a>text run
- \[css-flexbox-1\] defines the following terms:
  - <a id="term-for-propdef-order"></a>order
- \[css-images-3\] defines the following terms:
  - <a id="term-for-typedef-image"></a>\<image\>
- \[css-images-4\] defines the following terms:
  - <a id="term-for-invalid-image"></a>valid image
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-propdef-overflow"></a>overflow
  - <a id="term-for-valdef-overflow-visible"></a>visible
- \[css-position-3\] defines the following terms:
  - <a id="term-for-propdef-position"></a>position
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="term-for-selectordef-after"></a>::after
  - <a id="term-for-selectordef-before"></a>::before
  - <a id="term-for-selectordef-marker"></a>::marker
- \[css-syntax-3\] defines the following terms:
  - <a id="term-for-css-invalid"></a>invalid
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="term-for-forced-line-break"></a>forced line break
  - <a id="term-for-propdef-letter-spacing"></a>letter-spacing
  - <a id="term-for-propdef-text-transform"></a>text-transform
  - <a id="term-for-propdef-white-space"></a>white-space
- \[css-text-4\] defines the following terms:
  - <a id="term-for-propdef-text-space-collapse"></a>text-space-collapse
  - <a id="term-for-propdef-text-space-trim"></a>text-space-trim
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="term-for-integer-value"></a>\<integer\>
  - <a id="term-for-string-value"></a>\<string\>
- \[css-values-4\] defines the following terms:
  - <a id="term-for-mult-one-plus"></a>+
  - <a id="term-for-identifier-value"></a>\<custom-ident\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-funcdef-calc"></a>calc()
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-functional-notation"></a>functional notation
  - <a id="term-for-comb-one"></a>\|
  - <a id="term-for-comb-any"></a>\|\|
- \[css-variables-1\] defines the following terms:
  - <a id="term-for-custom-property"></a>custom property
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="term-for-propdef-direction"></a>direction
  - <a id="term-for-propdef-unicode-bidi"></a>unicode-bidi
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-inline-start"></a>inline-start
  - <a id="term-for-propdef-text-combine-upright"></a>text-combine-upright
  - <a id="term-for-writing-mode"></a>writing mode
- \[CSS2\] defines the following terms:
  - <a id="term-for-valdef-visibility-hidden"></a>hidden
  - <a id="term-for-valdef-counter-reset-none"></a>none
  - <a id="term-for-propdef-visibility"></a>visibility
- \[DOM\] defines the following terms:
  - <a id="term-for-concept-tree-root"></a>root
  - <a id="term-for-concept-tree-order"></a>tree order
- \[HTML\] defines the following terms:
  - <a id="term-for-the-li-element"></a>li
  - <a id="term-for-the-ol-element"></a>ol
  - <a id="term-for-the-option-element"></a>option
  - <a id="term-for-the-ul-element"></a>ul
- \[INFRA\] defines the following terms:
  - <a id="term-for-set-append"></a>append
  - <a id="term-for-list-contain"></a>contain
  - <a id="term-for-map-iterate"></a>for each
  - <a id="term-for-list-remove"></a>remove
  - <a id="term-for-ordered-set"></a>set
  - <a id="term-for-string"></a>string
  - <a id="term-for-tuple"></a>tuple
- \[SELECTORS-4\] defines the following terms:
  - <a id="term-for-compound"></a>compound selector
  - <a id="term-for-originating-element"></a>originating element
  - <a id="term-for-pseudo-element"></a>pseudo-element
  - <a id="term-for-selector"></a>selector
- \[SVG2\] defines the following terms:
  - <a id="term-for-elementdef-rect"></a>rect

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 18 August 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-content-3"></a>\[CSS-CONTENT-3\]  
Elika Etemad; Dave Cramer. [CSS Generated Content Module Level 3](https://www.w3.org/TR/css-content-3/). 2 August 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-content-3&#x2F;](https://www.w3.org/TR/css-content-3/)

<a id="biblio-css-counter-styles-3"></a>\[CSS-COUNTER-STYLES-3\]  
Tab Atkins Jr.. [CSS Counter Styles Level 3](https://www.w3.org/TR/css-counter-styles-3/). 14 December 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-counter-styles-3&#x2F;](https://www.w3.org/TR/css-counter-styles-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 19 May 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 10 October 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Image Values and Replaced Content Module Level 4](https://www.w3.org/TR/css-images-4/). 13 April 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 3 June 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; et al. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 19 May 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 25 February 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 16 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 29 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-text-4"></a>\[CSS-TEXT-4\]  
Elika Etemad; et al. [CSS Text Module Level 4](https://www.w3.org/TR/css-text-4/). 13 November 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-4&#x2F;](https://www.w3.org/TR/css-text-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 31 January 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 3 December 2015. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 21 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

### <a id="informative"></a>Informative References

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
Dean Jackson; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-color-3"></a>\[CSS-COLOR-3\]  
Tantek Çelik; Chris Lilley; David Baron. [CSS Color Module Level 3](https://www.w3.org/TR/css-color-3/). 19 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-3&#x2F;](https://www.w3.org/TR/css-color-3/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 5 November 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-fonts-3"></a>\[CSS-FONTS-3\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

## <a id="property-index"></a>Property Index

<strong>Table 10 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell; scope col):</strong>

Name

<strong>Column 2 (header cell; scope col):</strong>

Value

<strong>Column 3 (header cell; scope col):</strong>

Initial

<strong>Column 4 (header cell; scope col):</strong>

Applies to

<strong>Column 5 (header cell; scope col):</strong>

Inh.

<strong>Column 6 (header cell; scope col):</strong>

%ages

<strong>Column 7 (header cell; scope col):</strong>

Anim­ation type

<strong>Column 8 (header cell; scope col):</strong>

Canonical order

<strong>Column 9 (header cell; scope col):</strong>

Com­puted value

<strong>Row 2</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-counter-increment①⑥"></a>

[counter-increment](#propdef-counter-increment)

<strong>Column 2 (data cell):</strong>

\[ \<counter-name\> \<integer\>? \]+ \| none

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword none or a list, each item an identifier paired with an integer

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-counter-reset⑥"></a>

[counter-reset](#propdef-counter-reset)

<strong>Column 2 (data cell):</strong>

\[ \<counter-name\> \<integer\>? \]+ \| none

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword none or a list, each item an identifier paired with an integer

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-counter-set①⓪"></a>

[counter-set](#propdef-counter-set)

<strong>Column 2 (data cell):</strong>

\[ \<counter-name\> \<integer\>? \]+ \| none

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword none or a list, each item an identifier paired with an integer

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-list-style⑥"></a>

[list-style](#propdef-list-style)

<strong>Column 2 (data cell):</strong>

\<'list-style-position'\> \|\| \<'list-style-image'\> \|\| \<'list-style-type'\>

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

list items

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-list-style-image⑥"></a>

[list-style-image](#propdef-list-style-image)

<strong>Column 2 (data cell):</strong>

\<image\> \| none

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

list items

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword noneor the computed \<image\>

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-list-style-position③"></a>

[list-style-position](#propdef-list-style-position)

<strong>Column 2 (data cell):</strong>

inside \| outside

<strong>Column 3 (data cell):</strong>

outside

<strong>Column 4 (data cell):</strong>

list items

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

keyword, but see prose

<strong>Row 8</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-list-style-type①①"></a>

[list-style-type](#propdef-list-style-type)

<strong>Column 2 (data cell):</strong>

\<counter-style\> \| \<string\> \| none

<strong>Column 3 (data cell):</strong>

disc

<strong>Column 4 (data cell):</strong>

list items

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified value

<strong>Row 9</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-marker-side⑤"></a>

[marker-side](#propdef-marker-side)

<strong>Column 2 (data cell):</strong>

match-self \| match-parent

<strong>Column 3 (data cell):</strong>

match-self

<strong>Column 4 (data cell):</strong>

list items

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [white-space: pre](https://www.w3.org/TR/css-text-3/#propdef-white-space) doesn’t have quite the right behavior; [text-space-collapse: preserve-spaces](https://drafts.csswg.org/css-text-4/#propdef-text-space-collapse) + [text-space-trim: discard-after](https://drafts.csswg.org/css-text-4/#propdef-text-space-trim) might be closer to what’s needed here. See discussion in [Issue 4448](https://github.com/w3c/csswg-drafts/issues/4448) and [Issue 4891](https://github.com/w3c/csswg-drafts/issues/4891). [↵](#issue-04da870c)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This is handwavey nonsense from CSS2, and needs a real definition. [↵](#issue-ffa70ac6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Alternatively, [outside](#list-style-position-outside) could lay out the marker as a previous sibling of the principal inline box. [↵](#issue-bf311ada)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> For this order punctuation inside the marker correctly, it would also need to take the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) value of the parent. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;4202&#x3E;](https://github.com/w3c/csswg-drafts/issues/4202) [↵](#issue-91830b42)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> There are issues open on [renaming the keywords](https://github.com/w3c/csswg-drafts/issues/5308) and on [merging with list-style-position](https://github.com/w3c/csswg-drafts/issues/4209). [↵](#issue-69d20ea1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Counters are sometimes useful for things other than printing markers. In general, they provide the ability to number elements in sequence, which can be useful for other properties to reference. For example, using [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) to put an element between two other specific elements currently requires you to explicitly put order on every element before and/or after the desired insertion point. If you can set the order value of everything to a counter, tho, you can more easily insert an element into an arbitrary spot between two others.
>
> Other use-cases involve nested or sibling elements with transforms that are meant to be slightly different from each other. Today you have to use a preprocessor to do this in a reasonable way, but a counter would make it work well in "plain" CSS.
>
> (You can built up successive values in the nested case today by using [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) and stacking up nested [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc)s, but this is a \*little bit\* clumsy, and doesn’t work for siblings.)
>
> Suggestion is to add a counter-value([\<counter-name\>](#typedef-counter-name)) function, which returns the value of the named counter as an integer, rather than returning a string.
>
> See [Issue 1026](https://github.com/w3c/csswg-drafts/issues/1026).
>
> [↵](#issue-4b0c2d48)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Discussion of how to support `ol[reversed]` list numbering in CSS is ongoing. See, e.g. [Issue 4181](https://github.com/w3c/csswg-drafts/issues/4181). [↵](#issue-33887700)
