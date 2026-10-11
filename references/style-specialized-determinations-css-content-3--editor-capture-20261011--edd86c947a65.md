Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material copied from [CSS Generated Content Module Level 3](https://drafts.csswg.org/css-content-3/) under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Original copyright, liability, trademark and permissive document-license notice retained below.

# Provenance and boundary

Captured 2026-10-11. Full HTML SHA-256 `9c640ae017c7f848bb11171b96cca9d6b3b01f78f33faafc9bb094607c833945`; 318215 bytes. Bounded HTML SHA-256 `edd86c947a653c278aad018c37116a4dd2173853890dd2ff22b64e77d54f6bc6`; 52521 bytes. Source revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. Retained complete heading sections: `target-counter`, `target-counters`, `target-text`, `content-function`, each through the next equal-or-higher heading, plus source header/status including warnings. Other sections resolve through upstream links. This editor capture is work in progress; Surgeist policies are recorded separately and do not change the captured source.

Converted with the existing HTML preparation, Pandoc 3.1.11.1 and semantic filter.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Generated Content Module Level 3

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 9 October 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-content-3/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-content-3/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2016/WD-css-content-3-20160602/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-content-3)

[Inline In Spec](https://drafts.csswg.org/css-content-3/#issues-index)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple, formerly Mozilla)

[Mike Bremford](mailto:mike@bfo.com) (BFO)

<strong>Former Editors:</strong>

[Dave Cramer](mailto:dauwhe@gmail.com) (Hachette Livre)

[Håkon Wium Lie](mailto:howcome@opera.com) (Opera Software)

[Ian Hickson](mailto:ian@hixie.ch) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-content-3/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-content/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-content” in the title, like this: “\[css-content\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-content%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This is a very rough draft, and is not ready for implementation.

The following features are at-risk, and may be dropped during the CR period:

- counters in <a id="ref-for-propdef-content"></a>[content](https://drafts.csswg.org/css-content-3/#propdef-content) alt text

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

### <a id="target-counter"></a>6.1.  The <a id="ref-for-funcdef-target-counter②"></a>[target-counter()](#funcdef-target-counter) function[](#target-counter)

<a id="funcdef-target-counter"></a><a id="ref-for-string-value①①"></a><a id="ref-for-comb-one②⑥"></a><a id="ref-for-url-value"></a><a id="ref-for-comb-comma"></a><a id="ref-for-identifier-value"></a><a id="ref-for-comb-comma①"></a><a id="ref-for-typedef-counter-style"></a><a id="ref-for-mult-opt①"></a>

``` text
target-counter() = target-counter( [ <string> | <url> ] , <custom-ident> , <counter-style>? )
```

The <a id="ref-for-funcdef-target-counter③"></a>[target-counter()](#funcdef-target-counter) function retrieves the value of the innermost counter with a given name. The required arguments are the url of the target and the name of the counter. An optional counter-style argument can be used to format the result.

These functions only take a fragment URL which points to a location in the current document. If there’s no fragment, if the ID referenced isn’t there, or if the URL points to an outside document, the user agent must treat that as an error.

<a id="issue-7443dd19"></a>

<strong>Issue:</strong>

[](#issue-7443dd19) what should error handling be?

<a id="issue-2b2bd98b"></a>

<strong>Issue:</strong>

[](#issue-2b2bd98b) restrict syntactically to local references for now.

<a id="example-a3b9f868"></a>

<strong>Example:</strong>

[](#example-a3b9f868) HTML:

``` text
…which will be discussed on page <a href="#chapter4_sec2"></a>.
```

CSS:

``` text
a::after { content: target-counter(attr(href url), page) }
```

Result:

``` text
…which will be discussed on page 137.
```

<a id="example-61dcfb19"></a>

<strong>Example:</strong>

[](#example-61dcfb19) Page numbers in tables of contents can be generated automatically:

HTML:

``` text
<nav>
  <ol>
   <li class="frontmatter"><a href="#pref_01">Preface</a></li>
   <li class="frontmatter"><a href="#intr_01">Introduction</a></li>
   <li class="bodymatter"><a href="#chap_01">Chapter One</a></li>
  </ol>
</nav>
```

CSS:

``` text
.frontmatter a::after { content: leader('.') target-counter(attr(href url), page, lower-roman) }
.bodymatter a::after { content: leader('.') target-counter(attr(href url), page, decimal) }
```

Result:

``` text
Preface.............vii
Introduction.........xi
Chapter One...........1
```

### <a id="target-counters"></a>6.2.  The <a id="ref-for-funcdef-target-counters②"></a>[target-counters()](#funcdef-target-counters) function[](#target-counters)

This functions fetches the value of all counters of a given name from the end of a link, and formats them by inserting a given string between the value of each nested counter.

<a id="funcdef-target-counters"></a><a id="ref-for-string-value①②"></a><a id="ref-for-comb-one②⑦"></a><a id="ref-for-url-value①"></a><a id="ref-for-comb-comma②"></a><a id="ref-for-identifier-value①"></a><a id="ref-for-comb-comma③"></a><a id="ref-for-string-value①③"></a><a id="ref-for-comb-comma④"></a><a id="ref-for-typedef-counter-style①"></a><a id="ref-for-mult-opt②"></a>

``` text
target-counters() = target-counters( [ <string> | <url> ] , <custom-ident> , <string> , <counter-style>? )
```

<a id="example-00974551"></a>

<strong>Example:</strong>

[](#example-00974551)

``` text
I have not found a compelling example for target-counters() yet.
```

<a id="issue-6e7203db"></a>

<strong>Issue:</strong>

[](#issue-6e7203db) found a compelling example, in CSS specs. Do something.

### <a id="target-text"></a>6.3.  The <a id="ref-for-target-text-function②"></a>[target-text()](#target-text-function) function[](#target-text)

The <a id="ref-for-target-text-function③"></a>[target-text()](#target-text-function) function retrieves the text value of the element referred to by the URL. An optional second argument specifies what content is retrieved, using the same values as the <a id="ref-for-propdef-string-set"></a>[string-set](https://drafts.csswg.org/css-content-3/#propdef-string-set) property above.

<a id="target-text-function"></a><a id="ref-for-string-value①④"></a><a id="ref-for-comb-one②⑧"></a><a id="ref-for-url-value②"></a><a id="ref-for-comb-comma⑤"></a><a id="ref-for-comb-one②⑨"></a><a id="ref-for-comb-one③⓪"></a><a id="ref-for-comb-one③①"></a><a id="ref-for-mult-opt③"></a>

``` text
target-text() = target-text( [ <string> | <url> ] , [ content | before | after | first-letter ]? )
```

<a id="issue-a82075c9"></a>

<strong>Issue:</strong>

[](#issue-a82075c9) A simpler syntax has been proposed by fantasai: <http://lists.w3.org/Archives/Public/www-style/2012Feb/0745.html>

<a id="example-d5ce9fa3"></a>

<strong>Example:</strong>

[](#example-d5ce9fa3)

``` text
…which will be discussed <a href="#chapter_h1_1">later</a>.

a::after { content: ", in the chapter entitled " target-text(attr(href url)) }
```

Result: …which will be discussed later, in the chapter entitled Loomings.

### <a id="content-function"></a>7.3.  The <a id="ref-for-funcdef-content②"></a>[content()](#funcdef-content) function[](#content-function)

<a id="funcdef-content"></a><a id="ref-for-comb-one③⑥"></a><a id="ref-for-comb-one③⑦"></a><a id="ref-for-comb-one③⑧"></a><a id="ref-for-comb-one③⑨"></a><a id="ref-for-mult-opt⑤"></a>

``` text
content() = content( [ text | before | after | first-letter | marker ]? )
```

<strong><a id="valdef-content-text"></a><strong>text</strong></strong>

The string value of the element. If no value is specified in <a id="ref-for-funcdef-content③"></a>[content()](#funcdef-content), it acts as if <a id="ref-for-valdef-content-text"></a>[text](#valdef-content-text) were specified.

<strong><a id="valdef-content-before"></a><strong>before</strong></strong>

The string value of the <a id="ref-for-selectordef-before③"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before) <a id="ref-for-pseudo-element⑧"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element).

<strong><a id="valdef-content-after"></a><strong>after</strong></strong>

The string value of the <a id="ref-for-selectordef-after④"></a>[::after](https://drafts.csswg.org/css-pseudo-4/#selectordef-after) <a id="ref-for-pseudo-element⑨"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element).

<strong><a id="valdef-content-first-letter"></a><strong>first-letter</strong></strong>

The first letter of the element, as defined for the <a id="ref-for-selectordef-first-letter"></a>[::first-letter](https://drafts.csswg.org/css-pseudo-4/#selectordef-first-letter) <a id="ref-for-pseudo-element①⓪"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element)

<strong><a id="valdef-content-marker"></a><strong>marker</strong></strong>

The string value of the <a id="ref-for-selectordef-marker②"></a>[::marker](https://drafts.csswg.org/css-pseudo-4/#selectordef-marker) <a id="ref-for-pseudo-element①①"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element).

<a id="example-e698ead2"></a>

<strong>Example:</strong>

[](#example-e698ead2) HTML:

``` text
<h1>Loomings</h1>
```

CSS:

``` text
h1::before { content: 'Chapter ' counter(chapter); }
h1 { string-set: header content(before) ':' content(text); }
h1::after { content: '.'; }
```

The value of the named string “header” will be “Chapter 1: Loomings”.

<a id="example-ba54cbec"></a>

<strong>Example:</strong>

[](#example-ba54cbec) HTML:

``` text
<section title="Loomings">
```

CSS:

``` text
section { string-set: header attr(title) }
```

The value of the “header” string will be “Loomings”.
