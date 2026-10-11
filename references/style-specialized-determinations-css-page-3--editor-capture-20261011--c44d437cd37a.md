Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material copied from [CSS Paged Media Module Level 3](https://drafts.csswg.org/css-page-3/) under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Original copyright, liability, trademark and permissive document-license notice retained below.

# Provenance and boundary

Captured 2026-10-11. Full HTML SHA-256 `1b3a93e25d0bb730752af06a27529c72b274419d887d04df4d42ff02cc733890`; 407670 bytes. Bounded HTML SHA-256 `c44d437cd37a364cd50f50d4365373103e29003462053678ab5c82353e27b96b`; 35169 bytes. Source revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. Retained complete heading sections: `progression`, `first-pseudo`, each through the next equal-or-higher heading, plus source header/status including warnings. Other sections resolve through upstream links. This editor capture is work in progress; Surgeist policies are recorded separately and do not change the captured source.

Converted with the existing HTML preparation, Pandoc 3.1.11.1 and semantic filter.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Paged Media Module Level 3

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 3 March 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-page-3/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-page-3/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-page-3)

[Tracker](http://www.w3.org/Style/CSS/Tracker/products/13)

[Inline In Spec](https://drafts.csswg.org/css-page-3/#issues-index)

<strong>Editor:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple, formerly Mozilla)

<strong>Former Editors:</strong>

[Simon Sapin](http://exyr.org/about/) (Mozilla, formerly Kozea)

Melinda Grant (HP)

Håkon Wium Lie (Opera Software)

Jim Bigelow (Hewlett-Packard)

Jacob Refstrup (Hewlett-Packard)

Robert Stevahn (Hewlett-Packard)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-page-3/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-page” in the title, like this: “\[css-page\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-page%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

### <a id="progression"></a>3.3. Page Progression[](#progression)

CSS distinguishes between left pages and right pages on all documents, whether they are printed duplex or not. Each left page is followed by a right page and vice versa. Left and right pages can be styled differently with the [<code>&#58;left</code> and <code>&#58;right</code> pseudo-classes](https://drafts.csswg.org/css-page-3/#left-right-first).

Whether the first page of a document is a left page or a right page depends on the page progression of the document. The <a id="page-progression"></a><strong>page progression</strong> is the direction in which the printed pages of a document would be sequenced when laid out side-to-side. For example, English and horizontally-set Japanese typically progress from left to right, whereas Arabic and vertically-set Japanese pages typically progress from right to left.

The page progression direction is determined as follows:

- If text is laid out in horizontal lines, the <a id="ref-for-page-progression"></a>[page progression](#page-progression) is the same as the <a id="ref-for-inline-base-direction"></a>[inline base direction](https://drafts.csswg.org/css-writing-modes-4/#inline-base-direction).
- If text is laid out in vertical lines, the <a id="ref-for-page-progression①"></a>[page progression](#page-progression) is the same as the <a id="ref-for-block-flow-direction"></a>[block flow direction](https://drafts.csswg.org/css-writing-modes-4/#block-flow-direction).

If the UA supports the <a id="ref-for-propdef-direction"></a>[direction](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction) and <a id="ref-for-propdef-writing-mode"></a>[writing-mode](https://drafts.csswg.org/css-writing-modes-4/#propdef-writing-mode) properties from the CSS 3 Writing Modes Module [\[CSS-WRITING-MODES-3\]](https://drafts.csswg.org/css-page-3/#biblio-css-writing-modes-3), it must [determine](https://www.w3.org/TR/css3-writing-modes/#page-direction) the <a id="ref-for-page-progression②"></a>[page progression](#page-progression) is determined by the <a id="ref-for-principal-writing-mode"></a>[principal writing mode](https://drafts.csswg.org/css-writing-modes-4/#principal-writing-mode).

In documents with a left-to-right <a id="ref-for-page-progression③"></a>[page progression](#page-progression) the first page of the document is a right page, and vice versa. To explicitly force a document to begin printing on a left or right page, authors can specify a <a id="ref-for-propdef-break-before②"></a>[break-before](https://drafts.csswg.org/css-break-3/#propdef-break-before) value that that propagates a page break to the root. [\[CSS-BREAK-3\]](https://drafts.csswg.org/css-page-3/#biblio-css-break-3) The UA must suppress the first (empty) page(s) in this case (and the <code>&#58;first</code> pseudo-class matches the first printed page).

<a id="example-0b89bad0"></a>

<strong>Example:</strong>

[](#example-0b89bad0)

``` text
html { break-before: always }
```

For an HTML document with a left-to-right page progression, the above style rule will cause the first page of the document to print on a ':left' page

``` text
html { break-before: left }
```

For an HTML document, the above style rule will cause the first page of the document to print on a ':left' page, regardless of the page progression.

#### <a id="first-pseudo"></a>4.2.2.  First-page pseudo-class: <a id="ref-for-valdef-page-first"></a>[:first](#valdef-page-first)[](#first-pseudo)

The <a id="valdef-page-first"></a><strong>:first</strong> pseudo-class <a id="ref-for-match③"></a>[matches](https://drafts.csswg.org/css-page-3/#match) the first printed page of a document.

<a id="example-0c64c208"></a>

<strong>Example:</strong>

[](#example-0c64c208)

``` text
@page { margin: 2cm } /* All margins set to 2cm */

@page :first {
margin-top: 10cm /* Top margin on first page 10cm */
}
```
