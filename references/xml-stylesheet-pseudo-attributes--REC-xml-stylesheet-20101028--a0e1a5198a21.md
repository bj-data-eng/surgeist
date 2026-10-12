# XML-stylesheet: complete pseudo-attribute parsing rules

This bounded software-support reference retains the entire delegated §3 parsing
operation. The parsing result is a complete set or an error, not a partial
recovery map. This is not a replacement technical specification, an XML document
parser or stylesheet loading support. The original English Recommendation
remains authoritative.

Original: [Associating Style Sheets with XML documents 1.0 (Second Edition)](https://www.w3.org/TR/2010/REC-xml-stylesheet-20101028/). Complete original SHA-256:
`a0e1a5198a21c2aa4b623fc35b3c890d6e51c6c14ee126aed71cf3b93157515a`. The dated original and retained latest alias were byte-identical.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document
includes material copied from or derived from [Associating Style Sheets with XML documents 1.0 (Second Edition)](https://www.w3.org/TR/2010/REC-xml-stylesheet-20101028/).

This required derivative notice under the complete local
[W3C Document License 2015](../licenses/w3c/document-license-2015.txt)
supplements the original ©2010 notice retained below.
The complete [historical 2002 document-use terms](../licenses/w3c/document-license-2002.txt)
remain available. The [official 5 February 2015 update](https://www.w3.org/news/2015/w3c-updates-general-document-license/)
extends implementation-support derivative permission to existing publications.
This bounded software/support conversion is not published as a replacement
technical specification. Original disclaimer, trademark and status terms
remain; no endorsement or project software-license change is asserted.
[Local attribution](../licenses/w3c/NOTICE.md#dom-delegated-xml-pseudo-attribute-grammar)
records both original sources.

Changes: bounded selection; Markdown headings/paragraph presentation; semantic
HTML tables, lists and definition lists retained; source anchors retained with
explicit closing tags; source-relative and excluded-fragment links made dated
absolute; specifically selected delegated XML grammar links made local; inline
layout styles removed; line/inter-element whitespace normalized. Original prose,
grammar operators/ranges, table rows/cells, conditions and list order are
unchanged. No source script or linked resource is executed. Raw locators below
are one-based LF source lines, not runtime PI source indices.

Original title/editors/notice raw 35–43; complete original STATUS raw 45–88; whole §3 raw 164–196.

## Original edition notice and status

<a id="title"></a>

### Associating Style Sheets with XML documents 1.0 (Second Edition)

<a id="w3c-doctype"></a>

### W3C Recommendation 28 October 2010

<dl><dt>This version:</dt><dd>
     <a href="https://www.w3.org/TR/2010/REC-xml-stylesheet-20101028">http://www.w3.org/TR/2010/REC-xml-stylesheet-20101028</a>
  </dd><dt>Latest version:</dt><dd>
     <a href="https://www.w3.org/TR/xml-stylesheet/">http://www.w3.org/TR/xml-stylesheet</a>
    </dd><dt>Previous versions:</dt><dd>
     <a href="https://www.w3.org/TR/2010/PER-xml-stylesheet-20100909">http://www.w3.org/TR/2010/PER-xml-stylesheet-20100909</a>
    </dd><dt>Editors:</dt><dd>James Clark (First Edition) <a href="mailto:jjc@jclark.com">&lt;jjc@jclark.com&gt;</a></dd><dd>Simon Pieters (Second Edition), Opera Software <a href="mailto:simonp@opera.com">&lt;simonp@opera.com&gt;</a></dd><dd>Henry S. Thompson (Second Edition), University of Edinburgh <a href="mailto:ht@inf.ed.ac.uk">&lt;ht@inf.ed.ac.uk&gt;</a></dd></dl>

Please refer to the <a href="https://www.w3.org/1999/06/REC-xml-stylesheet-19990629/errata"><strong>errata</strong></a> for this document, which may
      include normative corrections.

See also <a href="http://www.w3.org/2003/03/Translations/byTechnology?technology=xml-stylesheet"><strong>translations</strong></a>.

This document is also available in these non-normative formats: <a href="https://www.w3.org/TR/2010/REC-xml-stylesheet-20101028/xml-stylesheet.xml">XML</a>.

<a href="https://www.w3.org/Consortium/Legal/ipr-notice#Copyright">Copyright</a> © 2010 <a href="https://www.w3.org/"><acronym title="World Wide Web Consortium">W3C</acronym></a><sup>®</sup> (<a href="http://www.csail.mit.edu/"><acronym title="Massachusetts Institute of Technology">MIT</acronym></a>, <a href="http://www.ercim.eu/"><acronym title="European Research Consortium for Informatics and Mathematics">ERCIM</acronym></a>, <a href="http://www.keio.ac.jp/">Keio</a>), All Rights Reserved. W3C <a href="https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer">liability</a>, <a href="https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks">trademark</a> and <a href="https://www.w3.org/Consortium/Legal/copyright-documents">document use</a> rules apply.

<a id="status"></a>

### Status of This Document

<em>
This section describes the status of this document at the time
of its publication. Other documents may supersede this document.
A list of current W3C publications and the latest revision of
this technical report can be found in the
<a href="https://www.w3.org/TR/">W3C technical reports index</a>
at http://www.w3.org/TR/.
</em>

This document is a product of the
<a href="https://www.w3.org/XML/Core/">XML Core Working Group</a>
as part of the
<a href="https://www.w3.org/XML/Activity.html">W3C XML Activity</a>.
The English version of this specification is the only normative version.
However, for translations of this document, see
<a href="https://www.w3.org/2003/03/Translations/byTechnology?technology=xml-stylesheet">
http://www.w3.org/2003/03/Translations/byTechnology?technology=xml-stylesheet</a>.

This second edition incorporates all known errata as of the publication date,
clarifies several areas left unspecified in the earlier edition, and has been
restructured to allow other specifications to reuse the
rules for parsing pseudo-attributes from a string.  A more detailed description
of the changes from the first edition is in <a href="https://www.w3.org/TR/2010/REC-xml-stylesheet-20101028/#changes"><b>B Changes since the first edition</b></a>.
This edition supersedes the previous
<a href="https://www.w3.org/1999/06/REC-xml-stylesheet-19990629/">edition of 29
June 1999</a>.

Comments on this document may be sent to
<a href="mailto:www-xml-stylesheet-comments@w3.org">www-xml-stylesheet-comments@w3.org</a>;
public
<a href="http://lists.w3.org/Archives/Public/www-xml-stylesheet-comments/">archives</a>
are available.  The errata list for this document is available at <a href="https://www.w3.org/1999/06/REC-xml-stylesheet-19990629/errata">http://www.w3.org/1999/06/REC-xml-stylesheet-19990629/errata</a>

This document has been reviewed by W3C Members, by software developers, and by other W3C groups and interested parties, and is endorsed by the Director as a W3C Recommendation. It is a stable document and may be used as reference material or cited from another document. W3C's role in making the Recommendation is to draw attention to the specification and to promote its widespread deployment. This enhances the functionality and interoperability of the Web.

W3C maintains a
<a href="https://www.w3.org/2002/08/xmlcore-IPR-statements">public
list of any patent disclosures</a>
made in connection with the deliverables of
the group; that page also includes instructions for disclosing a patent.
An individual who has actual knowledge of a patent which the individual
believes contains
<a href="https://www.w3.org/Consortium/Patent-Policy-20040205/#def-essential">Essential
Claim(s)</a>
must disclose the information in accordance with
<a href="https://www.w3.org/Consortium/Patent-Policy-20040205/#sec-Disclosure">section
6 of the W3C Patent Policy</a>.

## Complete original §3

<a id="pseudo-attributes"></a>

### 3 Pseudo-attributes

[<a id="dt-parsing" title="rules for parsing pseudo-attributes from a string">Definition</a>: The <b>rules for parsing pseudo-attributes from a string</b> are given in this section.]

[<a id="dt-parsing-result" title="parsing result">Definition</a>: The <b>parsing result</b> is either a set of <a title="pseudo-attribute" href="#dt-pseudo-attribute">pseudo-attributes</a> or an error.]

If the given string is not matched by the <a href="#NT-PseudoAtts">PseudoAtts</a> production, the <a title="parsing result" href="#dt-parsing-result">parsing result</a> is an error.

<a id="d0e323"></a>

### Productions for strings with pseudo-attributes

<table class="scrap" summary="Scrap"><tbody><tr valign="baseline"><td><a id="NT-PseudoAtts"></a>[1a]   </td><td><code>PseudoAtts</code></td><td>   ::=   </td><td><code>
                     <a href="#NT-PseudoAtt">PseudoAtt</a>? (<a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#NT-S">S</a> <a href="#NT-PseudoAtt">PseudoAtt</a>)* <a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#NT-S">S</a>?</code></td></tr><tr valign="baseline"><td><a id="NT-PseudoAtt"></a>[2]   </td><td><code>PseudoAtt</code></td><td>   ::=   </td><td><code><a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#NT-Name">Name</a> <a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#NT-S">S</a>? "=" <a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#NT-S">S</a>? <a href="#NT-PseudoAttValue">PseudoAttValue</a>
                  </code></td></tr><tr valign="baseline"><td><a id="NT-PseudoAttValue"></a>[3]   </td><td><code>PseudoAttValue</code></td><td>   ::=   </td><td><code>('"' ([^"&lt;&amp;] | <a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#NT-CharRef">CharRef</a> | <a href="#NT-PredefEntityRef">PredefEntityRef</a>)* '"' | "'"
([^'&lt;&amp;] | <a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#NT-CharRef">CharRef</a> | <a href="#NT-PredefEntityRef">PredefEntityRef</a>)* "'")</code></td></tr><tr valign="baseline"><td><a id="NT-PredefEntityRef"></a>[4]   </td><td><code>PredefEntityRef</code></td><td>   ::=   </td><td><code>"&amp;amp;" | "&amp;lt;" | "&amp;gt;" | "&amp;quot;" |
"&amp;apos;"</code></td></tr></tbody></table>

[<a id="dt-pseudo-attribute" title="pseudo-attribute">Definition</a>: Each
string matched by <a href="#NT-PseudoAtt">PseudoAtt</a> in the <a href="#NT-PseudoAtts">PseudoAtts</a> production
represents a <b>pseudo-attribute</b>.] A
<a title="pseudo-attribute" href="#dt-pseudo-attribute">pseudo-attribute</a> has a
<a title="name" href="#dt-name">name</a> and a <a title="value" href="#dt-value">value</a>.

[<a id="dt-name" title="name">Definition</a>: The
string matched by
<a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#NT-Name">Name</a> in the <a href="#NT-PseudoAtt">PseudoAtt</a>
production constitutes the <b>name</b> of the corresponding <a title="pseudo-attribute" href="#dt-pseudo-attribute">pseudo-attribute</a>.]

[<a id="dt-value" title="value">Definition</a>: The string matched by
<a href="#NT-PseudoAttValue">PseudoAttValue</a> in the <a href="#NT-PseudoAtt">PseudoAtt</a>
production —
with the
three modifications specified below — constitutes the <b>value</b>
of the corresponding <a title="pseudo-attribute" href="#dt-pseudo-attribute">pseudo-attribute</a>.]

<ol class="enumar"><li><p>Each <a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#NT-CharRef">CharRef</a> is replaced with the character it represents according to
XML <a href="https://www.w3.org/TR/2010/REC-xml-stylesheet-20101028/#xml">[XML]</a>.</p></li><li><p>Each <a href="#NT-PredefEntityRef">PredefEntityRef</a> in <a href="#NT-PseudoAttValue">PseudoAttValue</a> is replaced with U+0026 (&amp;) if it is
"<code>&amp;amp;</code>", U+003C (&lt;) if it is
"<code>&amp;lt;</code>", U+003E (&gt;) if it is
"<code>&amp;gt;</code>", U+0022 (") if it is
"<code>&amp;quot;</code>" and U+0027 (') if it is
"<code>&amp;apos;</code>".</p></li><li><p>The
 first and last character (the start and end quotes) are removed.</p></li></ol>

The <a title="parsing result" href="#dt-parsing-result">parsing result</a> is an error if the XML <a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#wf-Legalchar">Legal Character well-formedness contraint</a> is violated for any <a href="xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md#NT-CharRef">CharRef</a>. <a href="https://www.w3.org/TR/2010/REC-xml-stylesheet-20101028/#xml">[XML]</a>

The <a title="parsing result" href="#dt-parsing-result">parsing result</a> is an error if there are more than one <a title="pseudo-attribute" href="#dt-pseudo-attribute">pseudo-attribute</a> with the same <a title="name" href="#dt-name">name</a>.

If
the <a title="parsing result" href="#dt-parsing-result">parsing result</a> is not an error, then it is the set of
<a title="pseudo-attribute" href="#dt-pseudo-attribute">pseudo-attributes</a> represented when the given string is matched by the
<a href="#NT-PseudoAtts">PseudoAtts</a> production.

## Selection boundary

The [XML grammar closure](xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md)
supplies Char, S, NameStartChar/NameChar/Name, CharRef/Legal Character and complete
notation semantics. Grammar matching is whole-string, names are exact, empty
sets and quoted empty values are permitted, and errors discard the result.
The delegated value modifications do not introduce XML attribute normalization,
case folding or Unicode normalization. XML-stylesheet §4 loading/prolog behavior
and general XML entity/DTD machinery are outside this selection.

The existing [DOM PI initialization and update operation](dom-foundation--editor-capture-20261010--08dff0a4827f.md#update-attributes-from-data)
owns the caller's map clearing and error behavior separately. Reference retention
does not certify that parser construction or PI mutation implements it. The
[edition mapping](SOURCE-EDITIONS.md#dom-delegated-xml-pseudo-attribute-grammar)
records the delegated source scope without a new HTML edition.
