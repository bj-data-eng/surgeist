Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [CSS Style Attributes](https://www.w3.org/TR/2013/REC-css-style-attr-20131107/).

Original copyright notice: Copyright © 2013 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Style Attributes

Source snapshot: https://www.w3.org/TR/2013/REC-css-style-attr-20131107/

Snapshot SHA-256: 711c9a43978ff186225d21d7a7bff580037bc032ba70a7ba33c4fab16e396746

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# CSS Style Attributes

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2013 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](http://www.csail.mit.edu/), [ERCIM](http://www.ercim.eu/), [Keio](http://www.keio.ac.jp/), [Beihang](http://ev.buaa.edu.cn/)), All Rights Reserved. W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [document use](https://www.w3.org/Consortium/Legal/copyright-documents) rules apply.

## <a id="abstract"></a>Abstract

Markup languages such as HTML [\[HTML401\]](#HTML401) and SVG [\[SVG11\]](#SVG11) provide a style attribute on most elements, to hold inline style information that applies to those elements. This draft describes the syntax and interpretation of the CSS fragment that can be used in such style attributes.

## <a id="status"></a>Status of this document

This section describes the status of this document at the time of its publication. Other documents may supersede this document. A list of current W3C publications and the latest revision of this technical report can be found in the [W3C technical reports index](https://www.w3.org/TR/) at http://www.w3.org/TR/.

This document has been reviewed by W3C Members, by software developers, and by other W3C groups and interested parties, and is endorsed by the Director as a W3C Recommendation. It is a stable document and may be used as reference material or cited from another document. W3C's role in making the Recommendation is to draw attention to the specification and to promote its widespread deployment. This enhances the functionality and interoperability of the Web.

Please see the Working Group's [implementation report](http://test.csswg.org/suites/css-style-attr/nightly-unstable/report/results.html). One test is not passed, although this is due to bugs in browser implementation of xml:base and attribute (non)ordering, not the style attribute itself. The equivalent test for HTML (including the xml serialisation of HTML5) is passed by multiple implementations.

No changes to this document have been made since the previous version.

The ([archived](http://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-style-attr%5D%20PUT%20SUBJECT%20HERE) (see [instructions](https://www.w3.org/Mail/Request)) is preferred for discussion of this specification. When sending e-mail, please put the text “css-style-attr” in the subject, preferably like this: “\[css-style-attr\] <em>…summary of comment…</em>”

This document was produced by the [CSS Working Group](https://www.w3.org/Style/CSS/members) (part of the [Style Activity](https://www.w3.org/Style/)).

This document was produced by a group operating under the [5 February 2004 W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20040205/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/#sec-Disclosure).

## <a id="contents"></a>Table of Contents

## <a id="intro"></a>1. Introduction

Some document formats have a <a id="style-attribute"></a>style attribute to permit the author to directly apply style information to specific elements in documents. If a document format defines a style attribute (whether named ‘`style`’ or something else) and the attribute accepts CSS as its value, then this specification defines that <a id="style-attribute0"></a>style attribute’s syntax and interpretation.

> <strong data-conversion-semantic="example">Example</strong>
>
> The following example shows the use of the `style` attribute in HTML [\[HTML401\]](#HTML401):
>
> ```text
> <p style="color: #090; line-height: 1.2">...</p>
> ```
## <a id="conformance"></a>2. Conformance

A document or implementation cannot conform to CSS Style Attributes alone, but can claim conformance to CSS Style Attributes if it satisfies the conformance requirements in this specification when implementing CSS together with style attribute handling as defined in a document language that has one or more CSS style attributes.

Conformance to CSS Style Attributes is defined for two classes:

<a id="document"></a>document  
A document represented in a document language that defines a style attribute for one or more of its elements.

<a id="interpreter"></a>interpreter  
Someone or something that interprets the semantics of a document and its associated style information. (Most CSS [user agents](https://www.w3.org/TR/CSS21/conform.html#user-agent) fall under this category.)

The conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification. All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#RFC2119)

Examples in this specification are introduced with the words "for example" or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> This is an example of an informative example.

Informative notes begin with the word "Note" and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

## <a id="syntax"></a>3. Syntax and Parsing

The value of the style attribute must match the syntax of the contents of a CSS [declaration block](https://www.w3.org/TR/CSS21/syndata.html#rule-sets) (excluding the delimiting braces), whose formal grammar is given below in the terms and conventions of the [CSS core grammar](https://www.w3.org/TR/CSS21/syndata.html#syntax):

```text


declaration-list

  : S* declaration? [ ';' S* declaration? ]*

  ;

```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note that following the CSS2.1 convention, comment tokens are not shown in the rule above.

The interpreter must parse the style attribute's value using the same forward-compatible parsing rules that apply to parsing declaration block contents in a normal CSS style sheet. See [chapter 4 of the CSS2.1 specification](https://www.w3.org/TR/CSS21/syndata.html) for details. [\[CSS21\]](#CSS21)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that because there is no open brace delimiting the declaration list in the CSS style attribute syntax, a close brace (`}`) in the style attribute's value does not terminate the style data: it is merely an invalid token.

## <a id="interpret"></a>4. Cascading and Interpretation

The declarations in a style attribute apply to the element to which the attribute belongs. In the cascade, these declarations are considered to have author origin and a specificity higher than any selector. CSS2.1 [defines](https://www.w3.org/TR/CSS21/cascade.html#specificity) how style sheets and style attributes are cascaded together. [\[CSS21\]](#CSS21) Relative URLs in the style data must be resolved relative to the style attribute's element (or to the document if per-element resolution is not defined) when the attribute's value is parsed.

Aside from the differences in cascading, the declarations in a style attribute must be interpreted exactly as if they were given in a CSS style rule that applies to the element.

The CSS Working Group strongly recommends that document languages do not allow multiple CSS style attributes on a single element. If a document language allows multiple CSS style attributes, each must be parsed independently and treated as a separate style rule, the ordering of which should be defined by the document language, else is undefined.

## <a id="ack"></a>5. Acknowledgments

Thanks to feedback from Daniel Glazman, Ian Hickson, Eric A. Meyer, Björn Höhrmann.

## <a id="references"></a>6. References

### <a id="normative-references"></a>Normative references

<a id="CSS21"></a>\[CSS21\]

Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification.](css2--REC-CSS2-20110607--1e43327015ed.md) 7 June 2011. W3C Recommendation. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2011&#x2F;REC-CSS2-20110607](css2--REC-CSS2-20110607--1e43327015ed.md)

<a id="RFC2119"></a>\[RFC2119\]

S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels.](http://www.ietf.org/rfc/rfc2119.txt) Internet RFC 2119. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;ietf&#x2E;org&#x2F;rfc&#x2F;rfc2119&#x2E;txt](http://www.ietf.org/rfc/rfc2119.txt)

### <a id="informative-references"></a>Informative references

<a id="HTML401"></a>\[HTML401\]

Dave Raggett; Arnaud Le Hors; Ian Jacobs. [HTML 4.01 Specification.](https://www.w3.org/TR/1999/REC-html401-19991224) 24 December 1999. W3C Recommendation. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;1999&#x2F;REC-html401-19991224](https://www.w3.org/TR/1999/REC-html401-19991224)

<a id="SVG11"></a>\[SVG11\]

Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition).](https://www.w3.org/TR/2011/REC-SVG11-20110816/) 16 August 2011. W3C Recommendation. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2011&#x2F;REC-SVG11-20110816&#x2F;](https://www.w3.org/TR/2011/REC-SVG11-20110816/)
