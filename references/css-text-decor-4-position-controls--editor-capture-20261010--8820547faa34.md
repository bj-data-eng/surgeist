Attribution and reformatting notice added for Surgeist on 2026-10-10

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added scope and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Text Decoration Module Level 4](https://drafts.csswg.org/css-text-decor-4/). Original copyright notice: Copyright © 2026 World Wide Web Consortium. The original legal notice and its links are retained below. License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt).

# Source provenance and bounded scope

Source: https://drafts.csswg.org/css-text-decor-4/

Retrieved: 2026-10-10. Source status: Editor’s Draft, 17 August 2026.

Original complete HTML SHA-256: `8820547faa34362e96dd4542039da8374f416634dc36f5e58f7a03346e7a4277` (391575 bytes). Declared page revision: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`. The date and byte hash identify this captured editor rendering; the live URL can change.

Scope retained: complete sections 2.4–2.9 (including all their subsections), plus document head/status/legal notice. All other document sections, bibliography, indexes, and appendices are excluded. Links to excluded fragments resolve to the authoritative upstream URL. This is a bounded excerpt, not a complete edition. The exact original HTML is retained as local acquisition evidence; the serialized bounded HTML SHA-256 is `8256acee092af7fd72e094a4d4b1fb34b36eded695ae515d3abe0d483463e0ca`.

Representation changes: HTML to GFM conversion using Pandoc 3.1.11.1 and the repository's [conversion tools](tools/html-to-markdown/README.md). Scripts and styles are omitted without execution. Retained IDs, ordinary prose, headings, links, literal examples, image descriptions, and table cells are verified against the bounded HTML. Spanning table values are expanded only to their applicable columns; added table headings and visible semantic labels are non-normative. This excerpt contains 7 tables.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Text Decoration Module Level 4

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 17 August 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-text-decor-4/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-text-decor-4/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2020/WD-css-text-decor-4-20200506/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-text-decor-4)

[Tracker](http://www.w3.org/Style/CSS/Tracker/products/10)

[Inline In Spec](https://drafts.csswg.org/css-text-decor-4/#issues-index)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Koji Ishii](mailto:kojiishi@gmail.com) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-text-decor-4/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

### <a id="text-decoration-thickness-property"></a>2.4. <a id="text-decoration-width-property"></a> Text Decoration Line Thickness: the <a id="ref-for-propdef-text-decoration-thickness"></a>[text-decoration-thickness](#propdef-text-decoration-thickness) property[](#text-decoration-thickness-property)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                                                                                                                                                           |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-text-decoration-thickness"></a><strong>text-decoration-thickness</strong>                                                                                                                                                                                                                                                                                                                                             |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | auto <a id="ref-for-comb-one⑦"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) from-font <a id="ref-for-comb-one⑧"></a>\| <a id="ref-for-typedef-length-percentage"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) <a id="ref-for-comb-one⑨"></a>\| <a id="ref-for-typedef-line-width"></a>[\<line-width\>](https://drafts.csswg.org/css-backgrounds-3/#typedef-line-width) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | auto                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                                                                                  |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | N/A                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | as specified, with <a id="ref-for-typedef-length-percentage①"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) values computed                                                                                                                                                                                                                                                          |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | by computed value                                                                                                                                                                                                                                                                                                                                                                                                                    |

This property, which is a <a id="ref-for-longhand⑤"></a>[sub-property](https://drafts.csswg.org/css-cascade-5/#longhand) of the <a id="ref-for-propdef-text-decoration⑤"></a>[text-decoration](#propdef-text-decoration) shorthand, sets the stroke thickness of underlines, overlines, and line-throughs specified on the element with <a id="ref-for-propdef-text-decoration-line③"></a>[text-decoration-line](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-line), and affects all decorations originating from this element even if descendant boxes specify a different thickness.

Values have the following meanings:

<strong><a id="valdef-text-decoration-thickness-auto"></a><strong>auto</strong></strong>

The UA chooses an appropriate thickness for text decoration lines; see below.

<strong><a id="valdef-text-decoration-thickness-from-font"></a><strong>from-font</strong></strong>

If the <a id="ref-for-first-available-font"></a>[first available font](https://drafts.csswg.org/css-fonts-4/#first-available-font) has metrics indicating a preferred underline width, use that width, otherwise behaves as <a id="ref-for-valdef-text-decoration-thickness-auto"></a>[auto](#valdef-text-decoration-thickness-auto).

<strong><a id="valdef-text-decoration-thickness-length-percentage"></a><strong><a id="ref-for-typedef-length-percentage②"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage)</strong></strong>

<strong><a id="valdef-text-decoration-thickness-line-width"></a><strong><a id="ref-for-typedef-line-width①"></a>[\<line-width\>](https://drafts.csswg.org/css-backgrounds-3/#typedef-line-width)</strong></strong>

A length value specifies the thickness of text decoration lines as a fixed length.

Note: A length will inherit as a fixed value, and will not scale with the font.

A percentage value specifies the thickness of text decoration lines as a percentage of the used <a id="ref-for-propdef-font-size"></a>[font-size](https://drafts.csswg.org/css-fonts-4/#propdef-font-size).

Note: A percentage will inherit as a relative value, and will therefore scale with changes in the font as it inherits.

The UA should round the actual value to the nearest integer device pixel, and ensure it is at least one device pixel.

#### <a id="text-decoration-thickness"></a>2.4.1.  Automatic Thickness of Text Decoration Lines[](#text-decoration-thickness)

Some font formats (such as OpenType) can offer information about the appropriate thickness of a line decoration. The UA should use such font-based information when choosing <a id="ref-for-valdef-text-decoration-thickness-auto①"></a>[auto](#valdef-text-decoration-thickness-auto) line thicknesses wherever appropriate.

### <a id="line-position"></a>2.5.  Determining the Position and Thickness of Line Decorations[](#line-position)

<a id="issue-d1c311e6"></a>

<strong>Issue:</strong>

[](#issue-d1c311e6) This section is copied over from early drafts of Text Decoration Level 3. It is still under review, and needs integration with <a id="ref-for-propdef-text-underline-offset"></a>[text-underline-offset](#propdef-text-underline-offset) and <a id="ref-for-propdef-text-decoration-thickness①"></a>[text-decoration-thickness](#propdef-text-decoration-thickness).

Since line decorations can span elements with varying font sizes and vertical alignments, the best position for a line decoration is not necessarily the ideal position dictated by the <a id="ref-for-decorating-box④"></a>[decorating box](https://drafts.csswg.org/css-text-decor-4/#decorating-box). Instead, it’s calculated, per line, from all text decorated by the <a id="ref-for-decorating-box⑤"></a>decorating box on that line, the <a id="considered-text"></a><strong>considered text</strong>. However, descendants of the <a id="ref-for-decorating-box⑥"></a>decorating box that are skipped due to <a id="ref-for-propdef-text-decoration-skip"></a>[text-decoration-skip](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-skip), descendant inlines with <a id="ref-for-propdef-text-decoration-skip①"></a>text-decoration-skip: ink, and any descendants that do not participate in the <a id="ref-for-decorating-box⑦"></a>decorating box’s inline formatting context are excluded from the set of <a id="ref-for-considered-text"></a>[considered text](#considered-text).

The line decoration positions are then calculated per line as follows (treating <a id="ref-for-over"></a>[over](https://drafts.csswg.org/css-writing-modes-4/#over)-positioned underlines as <a id="ref-for-over①"></a>over lines and <a id="ref-for-under"></a>[under](https://drafts.csswg.org/css-writing-modes-4/#under)-positioned overlines as <a id="ref-for-under①"></a>under lines):

<strong><a id="ref-for-over②"></a>[over](https://drafts.csswg.org/css-writing-modes-4/#over) lines</strong>

Align the line decoration with respect to the highest <a id="ref-for-over③"></a>[over](https://drafts.csswg.org/css-writing-modes-4/#over) EM-box edge of the <a id="ref-for-considered-text①"></a>[considered text](#considered-text).

<strong>alphabetic underlines</strong>

The alphabetic underline position is calculated by taking the ideal offset (from the alphabetic baseline) of each run of <a id="ref-for-considered-text②"></a>[considered text](#considered-text), averaging those, and then using the lowest alphabetic baseline to actually position the line. (Alphabetic baselines can differ between <a id="ref-for-valdef-vertical-align-baseline"></a>[baseline](https://drafts.csswg.org/css2/#valdef-vertical-align-baseline)-aligned boxes if the dominant baseline is non-alphabetic.) To prevent superscripts and subscripts from throwing this position off-kilter, an inline with a non-initial computed <a id="ref-for-propdef-vertical-align"></a>[vertical-align](https://drafts.csswg.org/css-inline-3/#propdef-vertical-align) is treated as having the ideal underline position of its parent.

<strong>non-alphabetic <a id="ref-for-under②"></a>[under](https://drafts.csswg.org/css-writing-modes-4/#under) lines</strong>

Position the line decoration with respect to the lowest <a id="ref-for-under③"></a>[under](https://drafts.csswg.org/css-writing-modes-4/#under) EM-box edge of the <a id="ref-for-considered-text③"></a>[considered text](#considered-text).

<strong>line-throughs</strong>

Line-throughs essentially use the same sort of averaging as for alphabetic underlines, but recompute the position when drawing across a descendant with a different computed <a id="ref-for-propdef-font-size①"></a>[font-size](https://drafts.csswg.org/css-fonts-4/#propdef-font-size). (This ensures that the text remains effectively “crossed out” despite any font size changes.) For each run of <a id="ref-for-considered-text④"></a>[considered text](#considered-text) with the same <a id="ref-for-propdef-font-size②"></a>font-size, compute an ideal position averaged from its font metrics. To prevent superscripts and subscripts from throwing this position off-kilter, an inline with a non-initial computed <a id="ref-for-propdef-vertical-align①"></a>[vertical-align](https://drafts.csswg.org/css-inline-3/#propdef-vertical-align) is treated as having the ideal underline position of its parent. Position the portion of the line across each decorated fragment at that position.

<a id="issue-3dd812b9"></a>

<strong>Issue:</strong>

[](#issue-3dd812b9) For simplicity, line-throughs should draw over each element at that element’s preferred/averaged position. This can produce some undesirable jumpiness, but there doesn’t appear to be any way to avoid that which is correct in all instances, and all attempts are worryingly complex. What position should line-throughs adopt over elements that have a different font-size, but no <a id="ref-for-considered-text⑤"></a>[considered text](#considered-text)?

CSS does not define the thickness of line decorations. In determining the thickness of text decoration lines, user agents may consider the font sizes, faces, and weights of descendants to provide an appropriately averaged thickness.

<a id="example-081ed4f4"></a>

<strong>Example:</strong>

[](#example-081ed4f4) The following figure shows the averaging for underline:

<img src="https://drafts.csswg.org/css-text-decor-4/images/underline-averaging.gif" alt="In the first rendering of the underlined text &#x27;1st a&#x27;&#10;&#9;&#9;&#9;&#9;&#9;&#9;&#9;&#9; with &#x27;st&#x27; as a superscript, both the &#x27;1st&#x27; and the &#x27;a&#x27;&#10;&#9;&#9;&#9;&#9;&#9;&#9;&#9;&#9; are rendered in a small font. In the second rendering,&#10;&#9;&#9;&#9;&#9;&#9;&#9;&#9;&#9; the &#x27;a&#x27; is rendered in a larger font. In the third, both&#10;&#9;&#9;&#9;&#9;&#9;&#9;&#9;&#9; &#x27;1st&#x27; and &#x27;a&#x27; are large.">

In the three fragments of underlined text, the underline is drawn consecutively lower and thicker as the ratio of large text to small text increases.

Using the same example, a line-through would in the second fragment, instead of averaging the two font sizes, split the line-through into two segments:

![](https://drafts.csswg.org/css-text-decor-4/images/linethrough-averaging.gif)

In both cases, however, the superscript, due to the vertical-alignment shift, has no effect on the position of the line.

### <a id="text-decoration-property"></a>2.6.  Text Decoration Shorthand: the <a id="ref-for-propdef-text-decoration⑥"></a>[text-decoration](#propdef-text-decoration) property[](#text-decoration-property)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
|------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-text-decoration"></a><strong>text-decoration</strong>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | <a id="ref-for-propdef-text-decoration-line④"></a>[\<'text-decoration-line'\>](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-line) <a id="ref-for-comb-any③"></a>[\|\|](https://drafts.csswg.org/css-values-4/#comb-any) <a id="ref-for-propdef-text-decoration-thickness②"></a>[\<'text-decoration-thickness'\>](#propdef-text-decoration-thickness) <a id="ref-for-comb-any④"></a>\|\| <a id="ref-for-propdef-text-decoration-style①"></a>[\<'text-decoration-style'\>](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-style) <a id="ref-for-comb-any⑤"></a>\|\| <a id="ref-for-propdef-text-decoration-color①"></a>[\<'text-decoration-color'\>](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-color) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |

This property is a shorthand for setting <a id="ref-for-propdef-text-decoration-line⑤"></a>[text-decoration-line](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-line), <a id="ref-for-propdef-text-decoration-thickness③"></a>[text-decoration-thickness](#propdef-text-decoration-thickness), <a id="ref-for-propdef-text-decoration-style②"></a>[text-decoration-style](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-style), and <a id="ref-for-propdef-text-decoration-color②"></a>[text-decoration-color](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-color) in one declaration. Omitted values are set to their initial values.

<a id="example-4a6e8734"></a>

<strong>Example:</strong>

[](#example-4a6e8734) The following example underlines unvisited links with a solid blue underline in CSS1 and CSS2 UAs and a navy dotted underline in CSS3 UAs.

``` text
:link {
  color: blue;
  text-decoration: underline;
  text-decoration: navy dotted underline; /* Ignored in CSS1/CSS2 UAs */
}
```

Note: The shorthand purposefully omits the <a id="ref-for-propdef-text-underline-position②"></a>[text-underline-position](#propdef-text-underline-position) property, which is a language/writing-system–dependent setting that keys off the content, so that it can cascade and inherit independently from the (uninherited) stylistic settings of the <a id="ref-for-propdef-text-decoration⑦"></a>[text-decoration](#propdef-text-decoration) shorthand.

### <a id="text-underline-position-property"></a>2.7.  Text Underline Position: the <a id="ref-for-propdef-text-underline-position③"></a>[text-underline-position](#propdef-text-underline-position) property[](#text-underline-position-property)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                 |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-text-underline-position"></a><strong>text-underline-position</strong>                                                                                                                                                                                                       |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | auto <a id="ref-for-comb-one①⓪"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) \[ from-font <a id="ref-for-comb-one①①"></a>\| under \] <a id="ref-for-comb-any⑥"></a>[\|\|](https://drafts.csswg.org/css-values-4/#comb-any) \[ left <a id="ref-for-comb-one①②"></a>\| right \] |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | auto                                                                                                                                                                                                                                                                                       |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                        |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | yes                                                                                                                                                                                                                                                                                        |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                                                        |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified keyword(s)                                                                                                                                                                                                                                                                       |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                                                                                                                                                                                                   |

This property, which is <em>not</em> a <a id="ref-for-longhand⑥"></a>[sub-property](https://drafts.csswg.org/css-cascade-5/#longhand) of the <a id="ref-for-propdef-text-decoration⑧"></a>[text-decoration](#propdef-text-decoration) shorthand, sets the position of an underline with respect to the text, and defines its <a id="ref-for-underline-zero-position"></a>[zero position](#underline-zero-position) for further adjustment by <a id="ref-for-propdef-text-underline-offset①"></a>[text-underline-offset](#propdef-text-underline-offset). It affects all decorations originating from this element, even if descendant boxes specify a different position. It does not affect underlines specified by ancestor elements.

<a id="example-7303f01e"></a>

<strong>Example:</strong>

[](#example-7303f01e) The following example styles modern Chinese, Japanese, and Korean texts with the appropriate underline positions in both horizontal and vertical text:

``` text
:root:lang(ja), [lang|=ja], :root:lang(ko), [lang|=ko] { text-underline-position: under right; }
:root:lang(zh), [lang|=zh] { text-underline-position: under left; }
```

If <a id="ref-for-underline-left"></a>[left](#underline-left) or <a id="ref-for-underline-right"></a>[right](#underline-right) is specified alone, <a id="ref-for-underline-auto"></a>[auto](#underline-auto) is also implied. Values have the following meanings:

<strong><a id="underline-auto"></a><strong>auto</strong></strong>

The user agent may use any algorithm to determine the underline’s position; however it must be placed at or under the alphabetic baseline.

Note: It is suggested that the default underline position be close to the alphabetic baseline, unless that would either cross subscripted (or otherwise lowered) text or draw over glyphs from Asian scripts such as Han or Tibetan for which an alphabetic underline is too high: in such cases, shifting the underline lower or aligning to the em box edge as described for <a id="ref-for-underline-under"></a>[under](#underline-under) may be more appropriate.

<img src="https://drafts.csswg.org/css-text-decor-4/images/underline-position-alphabetic.png" alt="In a typical Latin font, the underline is positioned slightly&#10;&#9;&#9;&#9;&#9;          below the alphabetic baseline, leaving a gap between the line&#10;&#9;&#9;&#9;&#9;          and the bottom of most Latin letters, but crossing through&#10;&#9;&#9;&#9;&#9;          descenders such as the stem of a &#x27;p&#x27;.">

A typical “alphabetic” underline is positioned just below the alphabetic baseline

<strong><a id="valdef-text-underline-position-from-font"></a><strong>from-font</strong></strong>

If the <a id="ref-for-first-available-font①"></a>[first available font](https://drafts.csswg.org/css-fonts-4/#first-available-font) has metrics indicating a preferred underline offset, use that offset, otherwise behaves as <a id="ref-for-valdef-text-underline-offset-auto"></a>[auto](#valdef-text-underline-offset-auto).

<strong><a id="underline-under"></a><strong>under</strong></strong>

The underline is positioned <a id="ref-for-under④"></a>[under](https://drafts.csswg.org/css-writing-modes-4/#under) the element’s text content. In this case the underline usually does not cross the descenders. (This is sometimes called “accounting” underline.) This value can be combined with <a id="ref-for-underline-left①"></a>[left](#underline-left) or <a id="ref-for-underline-right①"></a>[right](#underline-right) if a particular side is preferred in vertical <a id="ref-for-typographic-mode"></a>[typographic modes](https://drafts.csswg.org/css-writing-modes-4/#typographic-mode).

<img src="https://drafts.csswg.org/css-text-decor-4/images/underline-position-under.png" alt="In a typical Latin font, the underline is far enough&#10;&#9;&#9;&#9;           below the text that it does not cross the bottom of a &#x27;g&#x27;.">

<a id="ref-for-propdef-text-underline-position④"></a>[text-underline-position: under](#propdef-text-underline-position)

<a id="example-95685877"></a>

<strong>Example:</strong>

[](#example-95685877) Because <a id="ref-for-propdef-text-underline-position⑤"></a>[text-underline-position](#propdef-text-underline-position) inherits, and is not reset by the <a id="ref-for-propdef-text-decoration⑨"></a>[text-decoration](#propdef-text-decoration) shorthand, the following example switches the document to use <a id="ref-for-underline-under①"></a>[under](#underline-under) underlining, which can be more appropriate for writing systems with long, complicated descenders. It is also often useful for mathematical or chemical texts that use many subscripts.

``` text
:root { text-underline-position: under; }
```

Note: The under value does not guarantee that the underline will not conflict with glyphs, as some fonts have descenders or diacritics that extend below the font’s descent metrics.

<strong><a id="underline-left"></a><strong>left</strong></strong>

In vertical <a id="ref-for-typographic-mode①"></a>[typographic modes](https://drafts.csswg.org/css-writing-modes-4/#typographic-mode), the underline is aligned as for <a id="ref-for-underline-under②"></a>[under](#underline-under), except it is always aligned to the left edge of the text. If this causes the underline to be drawn on the "over" side of the text, then an overline also switches sides and is drawn on the "under" side.

<strong><a id="underline-right"></a><strong>right</strong></strong>

In vertical <a id="ref-for-typographic-mode②"></a>[typographic modes](https://drafts.csswg.org/css-writing-modes-4/#typographic-mode), the underline is aligned as for <a id="ref-for-underline-under③"></a>[under](#underline-under), except it is always aligned to the right edge of the text. If this causes the underline to be drawn on the "over" side of the text, then an overline also switches sides and is drawn on the "under" side.

<a id="fig-text-underline-position"></a>

| Field                                                                                                                                                                                                      | Definition                                                                                                                                                                                                    |
|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <img src="https://drafts.csswg.org/css-text-decor-4/images/underline-position-left.png" alt="In mixed Japanese-Latin vertical text, &#x27;text-underline-position: left&#x27;&#10;&#9;&#9;&#9;&#9;&#9;          places the underline on the left side of the text."> | <img src="https://drafts.csswg.org/css-text-decor-4/images/underline-position-right.png" alt="In mixed Japanese-Latin vertical text, &#x27;text-underline-position: right&#x27;&#10;&#9;&#9;&#9;&#9;&#9;          places the underline on the right side of the text."> |
| <a id="ref-for-underline-left②"></a>[left](#underline-left)                                                                                                                                                | <a id="ref-for-underline-right②"></a>[right](#underline-right)                                                                                                                                                |

In vertical <a id="ref-for-typographic-mode③"></a>[typographic modes](https://drafts.csswg.org/css-writing-modes-4/#typographic-mode), the <a id="ref-for-propdef-text-underline-position⑥"></a>[text-underline-position](#propdef-text-underline-position) values <a id="ref-for-underline-left③"></a>[left](#underline-left) and <a id="ref-for-underline-right③"></a>[right](#underline-right) allow placing the underline on either side of the text. (In horizontal <a id="ref-for-typographic-mode④"></a>typographic modes, both values are treated as <a id="ref-for-underline-auto①"></a>[auto](#underline-auto).)

### <a id="underline-offset"></a>2.8.  Text Underline Offset: the <a id="ref-for-propdef-text-underline-offset②"></a>[text-underline-offset](#propdef-text-underline-offset) property[](#underline-offset)

| Field                                                                                    | Definition                                                                                                                                                                                                                         |
|------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-text-underline-offset"></a><strong>text-underline-offset</strong>                                                                                                                                                   |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | auto <a id="ref-for-comb-one①③"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) <a id="ref-for-typedef-length-percentage③"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | auto                                                                                                                                                                                                                               |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | yes                                                                                                                                                                                                                                |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | N/A                                                                                                                                                                                                                                |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | as specified, with <a id="ref-for-typedef-length-percentage④"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) values computed                                                        |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                        |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | by computed value                                                                                                                                                                                                                  |

This property, which is <em>not</em> a <a id="ref-for-longhand⑦"></a>[sub-property](https://drafts.csswg.org/css-cascade-5/#longhand) of the <a id="ref-for-propdef-text-decoration①⓪"></a>[text-decoration](#propdef-text-decoration) shorthand, sets the offset of underlines from their <a id="ref-for-underline-zero-position①"></a>[zero position](#underline-zero-position). Positive offsets represent distances outward from the text; negative offsets inward. It affects all decorations originating from this element, even if descendant boxes specify a different position. It does not affect underlines specified by ancestor elements.

Values have the following meanings:

<strong><a id="valdef-text-underline-offset-auto"></a><strong>auto</strong></strong>

The UA chooses an appropriate offset for underlines.

However, this offset must be zero if the computed value of <a id="ref-for-propdef-text-underline-position⑦"></a>[text-underline-position](#propdef-text-underline-position) is <a id="ref-for-valdef-text-underline-position-from-font"></a>[from-font](#valdef-text-underline-position-from-font) and the UA was able to extract an appropriate metric to use from the font.

<strong><a id="valdef-text-underline-offset-length-percentage"></a><strong><a id="ref-for-typedef-length-percentage⑤"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage)</strong></strong>

A length value specifies the offset of underlines as a fixed length.

Note: A length will inherit as a fixed value, and will not scale with the font.

A percentage value specifies the offset of underlines as a percentage of the used <a id="ref-for-propdef-font-size③"></a>[font-size](https://drafts.csswg.org/css-fonts-4/#propdef-font-size).

Note: A percentage will inherit as a relative value, and will therefore scale with changes in the font as it inherits.

When the value of the <a id="ref-for-propdef-text-decoration-line⑥"></a>[text-decoration-line](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-line) property is either <a id="ref-for-valdef-text-decoration-line-spelling-error①"></a>[spelling-error](https://drafts.csswg.org/css-text-decor-4/#valdef-text-decoration-line-spelling-error) or <a id="ref-for-valdef-text-decoration-line-grammar-error①"></a>[grammar-error](https://drafts.csswg.org/css-text-decor-4/#valdef-text-decoration-line-grammar-error), the UA must ignore the value of <a id="ref-for-propdef-text-underline-position⑧"></a>[text-underline-position](#propdef-text-underline-position).

#### <a id="line-offset-zero"></a>2.8.1.  Underline Offset Origin (Zero Position)[](#line-offset-zero)

The <a id="underline-zero-position"></a><strong>zero position</strong> of the underline depends on the value of <a id="ref-for-propdef-text-underline-position⑨"></a>[text-underline-position](#propdef-text-underline-position) as detailed below.

Interaction of <a id="ref-for-propdef-text-underline-position①⓪"></a>[text-underline-position](#propdef-text-underline-position) and <a id="ref-for-propdef-text-underline-offset③"></a>[text-underline-offset](#propdef-text-underline-offset)

| <a id="ref-for-propdef-text-underline-position①①"></a>[text-underline-position](#propdef-text-underline-position)    | Zero Position                                                               | Positive Direction                                                                      |
|----------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------|-----------------------------------------------------------------------------------------|
| <a id="ref-for-underline-auto②"></a>[auto](#underline-auto)                                                          | alphabetic baseline                                                         | <a id="ref-for-under⑤"></a>[under](https://drafts.csswg.org/css-writing-modes-4/#under) |
| <a id="ref-for-valdef-text-underline-position-from-font①"></a>[from-font](#valdef-text-underline-position-from-font) | position specified by the font metrics, falling back to alphabetic baseline | <a id="ref-for-under⑥"></a>[under](https://drafts.csswg.org/css-writing-modes-4/#under) |
| <a id="ref-for-underline-under④"></a>[under](#underline-under)                                                       | text-under edge                                                             | <a id="ref-for-under⑦"></a>[under](https://drafts.csswg.org/css-writing-modes-4/#under) |
| <a id="ref-for-underline-left④"></a>[left](#underline-left)                                                          | text-under (left) edge                                                      | <a id="ref-for-under⑧"></a>[under](https://drafts.csswg.org/css-writing-modes-4/#under) |
| <a id="ref-for-underline-right④"></a>[right](#underline-right)                                                       | text-over (right) edge                                                      | <a id="ref-for-over④"></a>[over](https://drafts.csswg.org/css-writing-modes-4/#over)    |

The underline is aligned to the outside of the specified position (extending its thickness in the positive direction only).

Any automatic adjustments made to accommodate descendant content are maintained; the <a id="ref-for-propdef-text-underline-offset④"></a>[text-underline-offset](#propdef-text-underline-offset) is in addition to those.

#### <a id="line-auto-offset"></a>2.8.2.  Using Font Metrics for Automatic Positioning[](#line-auto-offset)

Some font formats (such as OpenType) can offer information about the appropriate position of a line decoration. The UA should use such font-based information in its choice of <a id="ref-for-valdef-text-underline-offset-auto①"></a>[auto](#valdef-text-underline-offset-auto) offset wherever appropriate, and must use such information when <a id="ref-for-valdef-text-underline-position-from-font②"></a>[from-font](#valdef-text-underline-position-from-font) is specified for <a id="ref-for-propdef-text-underline-position①②"></a>[text-underline-position](#propdef-text-underline-position).

Note: Typically, OpenType font metrics give the position of an alphabetic underline; in some cases (especially in CJK fonts), it gives the position of a under left underline. (In this case, the font’s underline metrics typically touch the bottom edge of the em box). The UA may but is not required to correct for incorrect font metrics.

### <a id="text-line-constancy"></a>2.9.  Text Decoration Line Uniformity[](#text-line-constancy)

The exact position and thickness of line decorations depends on the values of <a id="ref-for-propdef-text-underline-position①③"></a>[text-underline-position](#propdef-text-underline-position), <a id="ref-for-propdef-text-underline-offset⑤"></a>[text-underline-offset](#propdef-text-underline-offset), and <a id="ref-for-propdef-text-decoration-thickness④"></a>[text-decoration-thickness](#propdef-text-decoration-thickness) as defined above, and is otherwise UA-defined. However, for underlines and overlines the UA must use a single thickness and position on each line for the decorations deriving from a single <a id="ref-for-decorating-box⑧"></a>[decorating box](https://drafts.csswg.org/css-text-decor-4/#decorating-box).

![A single underline drawn under varying font sizes and vertical positions must be a single line.](https://drafts.csswg.org/css-text-decor-4/images/underline-single.png) vs. ![Drawing multiple line segments, each with the position and thickness appropriate to the decorated text, is incorrect.](https://drafts.csswg.org/css-text-decor-4/images/underline-broken.png)

Correct and incorrect rendering of <code>&#60;u&#62;A&#60;sup&#62;B&#60;/sup&#62;&#60;big&#62;C&#60;/big&#62;D&#60;/u&#62;</code>

Note, since line decorations can span elements with varying font sizes and vertical alignments, the best position for a line decoration is not necessarily the ideal position dictated by the

<strong>Note:</strong>

<a id="ref-for-decorating-box⑨"></a>[decorating box](https://drafts.csswg.org/css-text-decor-4/#decorating-box). For example, an overline positioned to a small font will effectively become a line-through if the element contains text in a significantly larger font-size. Even for underlines, if the text is not aligned to the alphabetic baseline (for example, in vertical typesetting styles, text is aligned by its central baseline by default [\[CSS-WRITING-MODES-4\]](https://drafts.csswg.org/css-text-decor-4/#biblio-css-writing-modes-4)) an underline will cut through descendant text of a larger font-size. UA consideration of descendant content will therefore result in better typography.

![](https://drafts.csswg.org/css-text-decor-4/images/leftline-cross.png) ![](https://drafts.csswg.org/css-text-decor-4/images/leftline-under.png)

Due to the central baseline alignment of vertical text, a left-side underline on small vertical text will cut through the text of a child with a larger font size. The underline is not allowed to be broken, but adjusting its position further to the left properly accommodates all of the underlined text.

UAs <em>must</em> adjust line positions to match the shifted metrics of <a id="ref-for-decorating-box①⓪"></a>[decorating boxes](https://drafts.csswg.org/css-text-decor-4/#decorating-box) shifted with <a id="ref-for-propdef-vertical-align②"></a>[vertical-align](https://drafts.csswg.org/css-inline-3/#propdef-vertical-align) values other than <a id="ref-for-valdef-vertical-align-baseline①"></a>[baseline](https://drafts.csswg.org/css2/#valdef-vertical-align-baseline) [\[CSS2\]](https://drafts.csswg.org/css-text-decor-4/#biblio-css2) or subscripted/superscripted via <a id="ref-for-propdef-font-variant-position"></a>[font-variant-position](https://drafts.csswg.org/css-fonts-4/#propdef-font-variant-position) [\[CSS-FONTS-3\]](https://drafts.csswg.org/css-text-decor-4/#biblio-css-fonts-3), but <em>must not</em> adjust the line position or thickness in response to descendants of a <a id="ref-for-decorating-box①①"></a>decorating box that are so styled (even though it <em>may</em> adjust the position to accommodate descendants that are not so styled, such as those merely typeset in a different font size as noted above). This allows superscripts and subscripts to be properly decorated (underlined, struck through, etc.) but prevents them from distorting or breaking the positioning of such decorations on their ancestors.

<img src="https://drafts.csswg.org/css-text-decor-4/images/underline-superscript.png" alt="An underline for just the superscript &#x27;st&#x27; in &#x27;1st&#x27; is drawn just below the superscript,&#10;&#9;&#9;          whereas an underline for the entire text is drawn at the appropriate position for full-size text.">

Example of underline applied to superscripted text vs. underline applied to text containing a superscript

#### <a id="text-decoration-inset-property"></a>2.9.1.  Trimming, Expanding, and Shifting Text Decoration Lines: the <a id="ref-for-propdef-text-decoration-inset"></a>[text-decoration-inset](#propdef-text-decoration-inset) property[](#text-decoration-inset-property)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                |
|------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-text-decoration-inset"></a><strong>text-decoration-inset</strong>                                                                                                                                                                                                                                                                                                                                                                          |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | <a id="ref-for-typedef-length-percentage⑥"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage)<a id="ref-for-mult-num-range"></a>[{1,2}](https://drafts.csswg.org/css-values-4/#mult-num-range) <a id="ref-for-comb-one①④"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) auto                                                                                                                       |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | 0                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | Depending on the value of <a id="ref-for-propdef-box-decoration-break"></a>[box-decoration-break](https://drafts.csswg.org/css-break-4/#propdef-box-decoration-break), either refer to the inline size of the <a id="ref-for-decorating-box①②"></a>[decorating box](https://drafts.csswg.org/css-text-decor-4/#decorating-box) or of each individual <a id="ref-for-box-fragment①"></a>[box fragment](https://drafts.csswg.org/css-break-4/#box-fragment) |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified keyword or absolute length                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | by computed value                                                                                                                                                                                                                                                                                                                                                                                                                                         |

This property adjusts the start and end endpoints of line decorations. Positive values move an endpoint inward, trimming the decoration; negative values move it outward, extending the decoration.

It controls all text decoration lines drawn by this <a id="ref-for-decorating-box①③"></a>[decorating box](https://drafts.csswg.org/css-text-decor-4/#decorating-box), but not any text decoration lines drawn by its ancestors. If two component values are given, the first applies to the <a id="ref-for-css-start"></a>[start](https://drafts.csswg.org/css-writing-modes-4/#css-start) and the second to the <a id="ref-for-css-end"></a>[end](https://drafts.csswg.org/css-writing-modes-4/#css-end). Values have the following meanings:

<strong><a id="valdef-text-decoration-inset-length-percentage"></a><strong><a id="ref-for-typedef-length-percentage⑦"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage)</strong></strong>

Adjusts the start or end endpoint of the affected line decorations. Positive values move the endpoint inward (trimming), negative values move it outward (extending).

Percentage values either refer to the total inline size of the <a id="ref-for-decorating-box①④"></a>[decorating box](https://drafts.csswg.org/css-text-decor-4/#decorating-box), if <a id="ref-for-propdef-box-decoration-break①"></a>[box-decoration-break](https://drafts.csswg.org/css-break-4/#propdef-box-decoration-break) is set to <a id="ref-for-valdef-box-decoration-break-slice"></a>[slice](https://drafts.csswg.org/css-break-4/#valdef-box-decoration-break-slice), or to the inline size of each individual <a id="ref-for-box-fragment②"></a>[box fragment](https://drafts.csswg.org/css-break-4/#box-fragment), if it is set to <a id="ref-for-valdef-box-decoration-break-clone"></a>[clone](https://drafts.csswg.org/css-break-4/#valdef-box-decoration-break-clone).

<a id="example-f99f1c9e"></a>

<strong>Example:</strong>

[](#example-f99f1c9e)

The following example offsets an extra thick underline 1em endwards with respect to the text

``` text
h1 {
  text-decoration: underline 0.3em rgb(36 148 187 / 0.25);
  text-decoration-inset: 1em -1em;
}
```

<a id="example-962d633a"></a>

<strong>Example:</strong>

[](#example-962d633a)

In this example, the underline is inset 10% of the inline size of the <a id="ref-for-decorating-box①⑤"></a>[decorating box](https://drafts.csswg.org/css-text-decor-4/#decorating-box) from the start and end of it.

``` text
h1 {
  text-decoration: underline 0.2em rgb(216 148 23);
  text-decoration-inset: 10%;
}
```

This results in the orange underline being inset by 10 percent from the start and end of the headline.

![Sample rendering of the underline inset example](https://drafts.csswg.org/css-text-decor-4/images/underline-inset-percentage.png)

<a id="example-cc759271"></a>

<strong>Example:</strong>

[](#example-cc759271)

In this example, the underline is inset 10% of the inline size of each <a id="ref-for-box-fragment③"></a>[box fragment](https://drafts.csswg.org/css-break-4/#box-fragment) from the end of that fragment.

``` text
h1 {
  text-decoration: underline 0.2em rgb(216 148 23);
  text-decoration-inset: 0 10%;
  box-decoration-break: clone;
}
```

This results in the orange underline of each line being inset by 10 percent from the end.

![Sample rendering of the underline inset example](https://drafts.csswg.org/css-text-decor-4/images/underline-inset-percentage-clone.png)

<strong><a id="valdef-text-decoration-inset-auto"></a><strong>auto</strong></strong>

The UA chooses an inset amount that ensures that if two identical underlined elements appear side-by-side they do not appear to have a single underline. (This is important in Chinese, where underlining is a form of punctuation.)

![An underline below a series of Chinese characters has a gap between two adjacent underlining elements.](https://drafts.csswg.org/css-text-decor-4/images/decoration-skip-inset.png)

<a id="ref-for-propdef-text-decoration-inset①"></a>[text-decoration-inset: auto](#propdef-text-decoration-inset) for <code>&#60;u&#62;石井&#60;/u&#62;&#60;u&#62;艾俐俐&#60;/u&#62;</code>

The adjustment of the text decoration’s endpoints is subject to <a id="ref-for-propdef-box-decoration-break②"></a>[box-decoration-break](https://drafts.csswg.org/css-break-4/#propdef-box-decoration-break):

- for <a id="ref-for-valdef-box-decoration-break-slice①"></a>[slice](https://drafts.csswg.org/css-break-4/#valdef-box-decoration-break-slice) (the default) endpoint adjustment is only applied to the <a id="ref-for-css-start①"></a>[start](https://drafts.csswg.org/css-writing-modes-4/#css-start) edge of the first fragment and the <a id="ref-for-css-end①"></a>[end](https://drafts.csswg.org/css-writing-modes-4/#css-end) edge of the last fragment, and may accumulate to other fragments if the amount of the inset is more than the length of the fragment.

- for <a id="ref-for-valdef-box-decoration-break-clone①"></a>[clone](https://drafts.csswg.org/css-break-4/#valdef-box-decoration-break-clone) endpoint adjustment is applied to each fragment independently.
