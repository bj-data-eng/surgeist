Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material copied from [CSS Generated Content Module Level 3](https://drafts.csswg.org/css-content-3/) under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Original copyright, liability, trademark and permissive document-license notice retained below.

# Provenance and boundary

Captured 2026-10-11. Full HTML SHA-256 `9c640ae017c7f848bb11171b96cca9d6b3b01f78f33faafc9bb094607c833945`; 318215 bytes. Bounded HTML SHA-256 `b3028754056d71e57384be919b9adbdd0b0170628a193f9583e2e4041da0a7af`; 18978 bytes. Source revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. Retained complete sections: `bookmark-generation`, plus source header and status. Other sections resolve through original source links. Source status and open questions remain intact; Surgeist decisions are recorded separately.

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

## <a id="bookmark-generation"></a>8.  Bookmarks[](#bookmark-generation)

Some document formats, most notably PDF, allow the use of <a id="bookmarks"></a><strong>bookmarks</strong> as an aid to navigation. Bookmarks provide a list of links to document elements, as well as text to label the links and a level value. A bookmark has three properties: <a id="ref-for-propdef-bookmark-level"></a>[bookmark-level](#propdef-bookmark-level), <a id="ref-for-propdef-bookmark-label"></a>[bookmark-label](#propdef-bookmark-label), and <a id="ref-for-propdef-bookmark-state"></a>[bookmark-state](#propdef-bookmark-state).

When a user activates a bookmark, the user agent must bring that reference point to the user’s attention, exactly as if navigating to that element by fragment URL. This will also trigger matching the

<strong>Note:</strong>

<a id="ref-for-target-pseudo"></a>[:target](https://drafts.csswg.org/selectors-4/#target-pseudo) pseudo-class.

If an element has <a id="ref-for-style-containment①"></a>[style containment](https://drafts.csswg.org/css-contain-2/#style-containment), the <a id="ref-for-propdef-bookmark-level①"></a>[bookmark-level](#propdef-bookmark-level), <a id="ref-for-propdef-bookmark-label①"></a>[bookmark-label](#propdef-bookmark-label), and <a id="ref-for-propdef-bookmark-state①"></a>[bookmark-state](#propdef-bookmark-state) properties must have no effect on descendants of the element.

### <a id="bookmark-level"></a>8.1.  Setting a Bookmark: the <a id="ref-for-propdef-bookmark-level②"></a>[bookmark-level](#propdef-bookmark-level) property [](#bookmark-level)

The <a id="ref-for-propdef-bookmark-level③"></a>[bookmark-level](#propdef-bookmark-level) property determines if a bookmark is created, and at what level. If this property is absent, or has value <a id="ref-for-valdef-bookmark-level-none"></a>[none](#valdef-bookmark-level-none), no bookmark should be generated, regardless of the values of <a id="ref-for-propdef-bookmark-label②"></a>[bookmark-label](#propdef-bookmark-label) or <a id="ref-for-propdef-bookmark-state②"></a>[bookmark-state](#propdef-bookmark-state).

| Field                                                                                    | Definition                                                                                                                                                                                              |
|------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-bookmark-level"></a><strong>bookmark-level</strong>                                                                                                                                      |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | none <a id="ref-for-comb-one④⓪"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) <a id="ref-for-integer-value"></a>[\<integer \[1,∞\]\>](https://drafts.csswg.org/css-values-4/#integer-value) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | none                                                                                                                                                                                                    |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                     |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                      |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | N/A                                                                                                                                                                                                     |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | the keyword <a id="ref-for-valdef-bookmark-level-none①"></a>[none](#valdef-bookmark-level-none) or the specified integer                                                                                |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                             |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | by computed value type                                                                                                                                                                                  |

<strong><a id="valdef-bookmark-level-integer-1"></a><strong><a id="ref-for-integer-value①"></a>[\<integer \[1,∞\]\>](https://drafts.csswg.org/css-values-4/#integer-value)</strong></strong>

defines the level of the bookmark, with the top level being 1 (negative and zero values are invalid).

<strong><a id="valdef-bookmark-level-none"></a><strong>none</strong></strong>

no bookmark is generated.

<a id="example-dd8b2d49"></a>

<strong>Example:</strong>

[](#example-dd8b2d49)

``` text
section h1 { bookmark-level: 1; }
section section h1 { bookmark-level: 2; }
section section section h1 { bookmark-level: 3; }
```

Note: Bookmarks do not need to create a strict hierarchy of levels.

<a id="issue-de0f5f60"></a>

<strong>Issue:</strong>

[](#issue-de0f5f60) Should a bookmark be created for elements with <code>display&#58; none</code>?

### <a id="bookmark-label"></a>8.2.  Labelling a Bookmark: the <a id="ref-for-propdef-bookmark-label③"></a>[bookmark-label](#propdef-bookmark-label) property[](#bookmark-label)

| Field                                                                                    | Definition                                                                                                                 |
|------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-bookmark-label"></a><strong>bookmark-label</strong>                                                         |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | <a id="ref-for-typedef-content-list⑥"></a>[\<content-list\>](https://drafts.csswg.org/css-content-3/#typedef-content-list) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | content(text)                                                                                                              |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                        |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                         |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | N/A                                                                                                                        |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified value                                                                                                            |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                                   |

<strong><a id="valdef-bookmark-label-content-list"></a><strong><a id="ref-for-typedef-content-list⑦"></a>[\<content-list\>](https://drafts.csswg.org/css-content-3/#typedef-content-list)</strong></strong>

<a id="ref-for-typedef-content-list⑧"></a>[\<content-list\>](https://drafts.csswg.org/css-content-3/#typedef-content-list) is defined above, in the section on the <a id="ref-for-propdef-string-set⑤"></a>[string-set](https://drafts.csswg.org/css-content-3/#propdef-string-set) property. The value of <a id="ref-for-typedef-content-list⑨"></a>\<content-list\> becomes the text content of the bookmark label.

<a id="example-273659aa"></a>

<strong>Example:</strong>

[](#example-273659aa) HTML:

``` text
<h1>Loomings</h1>
```

CSS:

``` text
h1 {
bookmark-label: content(text);
bookmark-level: 1;
}
```

The bookmark label will be “Loomings”.

### <a id="bookmark-state"></a>8.3.  Initial Bookmark Toggle State: the <a id="ref-for-propdef-bookmark-state③"></a>[bookmark-state](#propdef-bookmark-state) property[](#bookmark-state)

The <a id="ref-for-propdef-bookmark-state④"></a>[bookmark-state](#propdef-bookmark-state) may be open or closed. The user must be able to toggle the bookmark state.

| Field                                                                                    | Definition                                                                                       |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-bookmark-state"></a><strong>bookmark-state</strong>                               |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | open <a id="ref-for-comb-one④①"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) closed |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | open                                                                                             |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | block-level elements                                                                             |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                               |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | N/A                                                                                              |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified keyword                                                                                |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                      |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                         |

<strong><a id="valdef-bookmark-state-open"></a><strong>open</strong></strong>

Subsequent bookmarks with <a id="ref-for-propdef-bookmark-level④"></a>[bookmark-level](#propdef-bookmark-level) greater than the given bookmark are displayed, until reaching another bookmark of the same level or lower. If one of subsequent bookmark is closed, apply the same test to determine if its subsequent bookmarks should be displayed.

<strong><a id="valdef-bookmark-state-closed"></a><strong>closed</strong></strong>

Subsequent bookmarks of bookmark-level greater than the given bookmark are not displayed, until reaching another bookmark of the same level or lower.

<a id="issue-be5a86f5"></a>

<strong>Issue:</strong>

[](#issue-be5a86f5) Is the initial bookmark state, or the bookmark state updated by the UA as appropriate?
