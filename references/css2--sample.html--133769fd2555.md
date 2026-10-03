Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Default style sheet for HTML 4](https://www.w3.org/TR/2011/REC-CSS2-20110607/sample.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Default style sheet for HTML 4

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/sample.html

Snapshot SHA-256: 133769fd2555638382dda99739cea7b853bd683e46bc0ce82b7bbfbfe399d90d

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.

---

<a id="q22.0"></a>

# Appendix D. Default style sheet for HTML 4

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<em>This appendix is informative, not normative.</em>

This style sheet describes the typical formatting of all HTML 4 ([\[HTML4\]](css2--refs.html--f208d881d0b7.md#ref-HTML4)) elements based on extensive research into current UA practice. Developers are encouraged to use it as a default style sheet in their implementations.

The full presentation of some HTML elements cannot be expressed in CSS 2.1, including [replaced](css2--conform.html--de58593b67d7.md#replaced-element) elements ("img", "object"), scripting elements ("script", "applet"), form control elements, and frame elements.

For other elements, the legacy presentation can be described in CSS but the solution removes the element. For example, the FONT element can be replaced by attaching CSS declarations to other elements (e.g., DIV). Likewise, legacy presentation of presentational attributes (e.g., the "border" attribute on TABLE) can be described in CSS, but the markup in the source document must be changed.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="bidi"></a>
>
> ```text
> 
> html, address,
> blockquote,
> body, dd, div,
> dl, dt, fieldset, form,
> frame, frameset,
> h1, h2, h3, h4,
> h5, h6, noframes,
> ol, p, ul, center,
> dir, hr, menu, pre   { display: block; unicode-bidi: embed }
> li              { display: list-item }
> head            { display: none }
> table           { display: table }
> tr              { display: table-row }
> thead           { display: table-header-group }
> tbody           { display: table-row-group }
> tfoot           { display: table-footer-group }
> col             { display: table-column }
> colgroup        { display: table-column-group }
> td, th          { display: table-cell }
> caption         { display: table-caption }
> th              { font-weight: bolder; text-align: center }
> caption         { text-align: center }
> body            { margin: 8px }
> h1              { font-size: 2em; margin: .67em 0 }
> h2              { font-size: 1.5em; margin: .75em 0 }
> h3              { font-size: 1.17em; margin: .83em 0 }
> h4, p,
> blockquote, ul,
> fieldset, form,
> ol, dl, dir,
> menu            { margin: 1.12em 0 }
> h5              { font-size: .83em; margin: 1.5em 0 }
> h6              { font-size: .75em; margin: 1.67em 0 }
> h1, h2, h3, h4,
> h5, h6, b,
> strong          { font-weight: bolder }
> blockquote      { margin-left: 40px; margin-right: 40px }
> i, cite, em,
> var, address    { font-style: italic }
> pre, tt, code,
> kbd, samp       { font-family: monospace }
> pre             { white-space: pre }
> button, textarea,
> input, select   { display: inline-block }
> big             { font-size: 1.17em }
> small, sub, sup { font-size: .83em }
> sub             { vertical-align: sub }
> sup             { vertical-align: super }
> table           { border-spacing: 2px; }
> thead, tbody,
> tfoot           { vertical-align: middle }
> td, th, tr      { vertical-align: inherit }
> s, strike, del  { text-decoration: line-through }
> hr              { border: 1px inset }
> ol, ul, dir,
> menu, dd        { margin-left: 40px }
> ol              { list-style-type: decimal }
> ol ul, ul ol,
> ul ul, ol ol    { margin-top: 0; margin-bottom: 0 }
> u, ins          { text-decoration: underline }
> br:before       { content: "\A"; white-space: pre-line }
> center          { text-align: center }
> :link, :visited { text-decoration: underline }
> :focus          { outline: thin dotted invert }
> 
> /* Begin bidirectionality settings (do not change) */
> BDO[DIR="ltr"]  { direction: ltr; unicode-bidi: bidi-override }
> BDO[DIR="rtl"]  { direction: rtl; unicode-bidi: bidi-override }
> 
> *[DIR="ltr"]    { direction: ltr; unicode-bidi: embed }
> *[DIR="rtl"]    { direction: rtl; unicode-bidi: embed }
> 
> @media print {
>   h1            { page-break-before: always }
>   h1, h2, h3,
>   h4, h5, h6    { page-break-after: avoid }
>   ul, ol, dl    { page-break-before: avoid }
> }
> 
> 
> ```