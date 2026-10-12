# XML: bounded pseudo-attribute grammar closure

This bounded software-support reference supplies the XML grammar dependencies of
the [complete XML-stylesheet pseudo-attribute operation](xml-stylesheet-pseudo-attributes--REC-xml-stylesheet-20101028--a0e1a5198a21.md#pseudo-attributes).
It preserves exact grammar ranges and complete selected definitions, including
the complete EBNF notation section. This is not a replacement technical
specification or a complete XML parser. The original English Recommendation
remains authoritative.

Original: [Extensible Markup Language (XML) 1.0 (Fifth Edition)](https://www.w3.org/TR/2008/REC-xml-20081126/). Complete original SHA-256:
`f58a4b9e1e5b8bac13fb55c4e5c7c8c5e2c176aa45ee68e2f1ab2498511927ec`. The dated original and retained latest alias were byte-identical.
The original itself records the 7 February 2013 in-place RFC-link correction;
this hash identifies those actual retained bytes.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document
includes material copied from or derived from [Extensible Markup Language (XML) 1.0 (Fifth Edition)](https://www.w3.org/TR/2008/REC-xml-20081126/).

This required derivative notice under the complete local
[W3C Document License 2015](../licenses/w3c/document-license-2015.txt)
supplements the original ©2008 notice retained below.
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

Original title/editors/notice raw 39–53; complete original STATUS raw 59–95; Character Range table plus complete following encoding paragraph and discouraged-character note raw 306–320; S context, complete table and complete following note raw 322–331; Name context raw 331–357 and exactly the three complete NameStartChar/NameChar/Name rows from the table on raw 358; complete CharRef production, Legal Character constraint and numeric interpretation raw 1141–1150; complete §6 Notation raw 1724–1780.

## Original edition notice and status

<a id="title"></a>

### Extensible Markup Language (XML) 1.0 (Fifth Edition)

<a id="w3c-doctype"></a>

### W3C Recommendation 26 November 2008

<a id="notice-20130207"></a>

<strong>Note:</strong> On 7 February 2013, this specification was modified in place to replace broken links to RFC4646 and RFC4647.

<dl><dt>This version:</dt><dd>
			<a href="https://www.w3.org/TR/2008/REC-xml-20081126/">http://www.w3.org/TR/2008/REC-xml-20081126/</a>
		</dd><dt>Latest version:</dt><dd>
			<a href="https://www.w3.org/TR/xml/">http://www.w3.org/TR/xml/</a>
		</dd><dt>Previous versions:</dt><dd>
   <a href="https://www.w3.org/TR/2008/PER-xml-20080205/">http://www.w3.org/TR/2008/PER-xml-20080205/</a>
  <br>
   <a href="https://www.w3.org/TR/2006/REC-xml-20060816/">http://www.w3.org/TR/2006/REC-xml-20060816/</a>
		</dd><dt>Editors:</dt><dd>Tim Bray, Textuality and Netscape <a href="mailto:tbray@textuality.com">&lt;tbray@textuality.com&gt;</a></dd><dd>Jean Paoli, Microsoft <a href="mailto:jeanpa@microsoft.com">&lt;jeanpa@microsoft.com&gt;</a></dd><dd>C. M. Sperberg-McQueen, W3C <a href="mailto:cmsmcq@w3.org">&lt;cmsmcq@w3.org&gt;</a></dd><dd>Eve Maler, Sun Microsystems, Inc. <a href="mailto:elm@east.sun.com">&lt;eve.maler@east.sun.com&gt;</a></dd><dd>François Yergeau</dd></dl>

Please refer to the <a href="https://www.w3.org/XML/xml-V10-5e-errata"><strong>errata</strong></a> for this document, which may
      include some normative corrections.

The <a href="https://www.w3.org/XML/xml-V10-4e-errata">previous errata</a> for this document, are also available.

See also <a href="https://www.w3.org/2003/03/Translations/byTechnology?technology=xml"><strong>translations</strong></a>.

This document is also available in these non-normative formats: <a href="https://www.w3.org/TR/2008/REC-xml-20081126/REC-xml-20081126.xml">XML</a> and <a href="https://www.w3.org/TR/2008/REC-xml-20081126/REC-xml-20081126-review.html">XHTML with color-coded revision indicators</a>.

<a href="https://www.w3.org/Consortium/Legal/ipr-notice#Copyright">Copyright</a> © 2008 <a href="https://www.w3.org/"><acronym title="World Wide Web Consortium">W3C</acronym></a><sup>®</sup> (<a href="http://www.csail.mit.edu/"><acronym title="Massachusetts Institute of Technology">MIT</acronym></a>, <a href="http://www.ercim.org/"><acronym title="European Research Consortium for Informatics and Mathematics">ERCIM</acronym></a>, <a href="http://www.keio.ac.jp/">Keio</a>), All Rights Reserved. W3C <a href="https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer">liability</a>, <a href="https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks">trademark</a> and <a href="https://www.w3.org/Consortium/Legal/copyright-documents">document use</a> rules apply.

<a id="status"></a>

### Status of this Document

<em>This section describes the status of this document at the time of its publication.
				Other documents may supersede this document. A list of current W3C publications and the
				latest revision of this technical report can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index</a> at
				http://www.w3.org/TR/.</em>

This document specifies a syntax created by subsetting an existing, widely
				used international text processing standard (Standard Generalized Markup Language,
				ISO 8879:1986(E) as amended and corrected) for use on the World Wide Web.
				It is a product of the <a href="https://www.w3.org/XML/Core/">XML Core Working Group</a>
				as part of the <a href="https://www.w3.org/XML/Activity">XML Activity</a>.
				The English version of this specification is the only normative version. However,
				for translations of this document, see <a href="https://www.w3.org/2003/03/Translations/byTechnology?technology=xml">http://www.w3.org/2003/03/Translations/byTechnology?technology=xml</a>.

This document is a <a href="https://www.w3.org/2005/10/Process-20051014/tr.html#q74">W3C Recommendation</a>. This fifth edition is <em>not</em> a new version of XML. As a convenience to readers,
				it incorporates the changes dictated by the accumulated errata (available at
				<a href="https://www.w3.org/XML/xml-V10-4e-errata">http://www.w3.org/XML/xml-V10-4e-errata</a>) to the <a href="https://www.w3.org/TR/2006/REC-xml-20060816/">Fourth
				Edition of XML 1.0, dated 16 August 2006</a>.  In particular, erratum <a href="https://www.w3.org/XML/xml-V10-4e-errata#E09">[E09]</a>
				relaxes the restrictions on element and attribute names, thereby providing in XML 1.0 the major end user benefit
				currently achievable only by using XML
1.1.  As a consequence, many possible
 documents which were not well-formed according to previous editions of this
 specification are now well-formed, and previously invalid documents
using the newly-allowed name characters in, for example, ID
attributes, are now valid.

This edition supersedes the previous <a href="https://www.w3.org/TR/2006/REC-xml-20060816/">W3C Recommendation
				of 16 August 2006</a>.

Please report errors in this document to
the public <a href="mailto:xml-editor@w3.org">xml-editor@w3.org</a> mail list; public
				<a href="http://lists.w3.org/Archives/Public/xml-editor/">archives</a> are available. For the convenience of readers,
				an <a href="https://www.w3.org/TR/2008/REC-xml-20081126/REC-xml-20081126-review.html">XHTML version with color-coded revision indicators</a> is
				also provided; this version highlights each change due to an erratum published in the
				<a href="https://www.w3.org/XML/xml-V10-4e-errata">errata
list</a> for the previous edition, together with a link to the particular
				erratum in that list. Most of the
errata in the list provide a rationale for the change.  The errata
list for this fifth edition is available at <a href="https://www.w3.org/XML/xml-V10-5e-errata">http://www.w3.org/XML/xml-V10-5e-errata</a>.

An implementation report is available at <a href="https://www.w3.org/XML/2008/01/xml10-5e-implementation.html">http://www.w3.org/XML/2008/01/xml10-5e-implementation.html</a>.
				A <a href="https://www.w3.org/XML/Test/">Test Suite</a> is maintained to help assessing conformance to this specification.

This document has been reviewed by W3C Members, by software developers, and by other W3C groups and interested parties, and is endorsed by the Director as a W3C Recommendation. It is a stable document and may be used as reference material or cited from another document. W3C's role in making the Recommendation is to draw attention to the specification and to promote its widespread deployment. This enhances the functionality and interoperability of the Web.

W3C maintains a <a href="https://www.w3.org/2002/08/xmlcore-IPR-statements" rel="disclosure">public list of
				any patent disclosures</a> made in connection with the deliverables of
				the group; that page also includes instructions for disclosing a patent.
				An individual who has actual knowledge of a patent which the individual
				believes contains <a href="https://www.w3.org/Consortium/Patent-Policy-20040205/#def-essential">Essential
				Claim(s)</a> must disclose the information in accordance with
				<a href="https://www.w3.org/Consortium/Patent-Policy-20040205/#sec-Disclosure">section 6 of the W3C Patent Policy</a>.

## Character Range and original contextual note

<a id="char32"></a>

### Character Range

<table class="scrap" summary="Scrap"><tbody><tr valign="baseline"><td><a id="NT-Char"></a>[2]   </td><td><code>Char</code></td><td>   ::=   </td><td><code>#x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]</code></td><td><i>/* any Unicode character, excluding the surrogate blocks, FFFE, and FFFF. */</i></td></tr></tbody></table>

The mechanism for encoding character code points into bit patterns may
vary from entity to entity. All XML processors <em class="rfc2119" title="Keyword in RFC 2119 context">MUST</em> accept the UTF-8 and UTF-16
encodings of Unicode <span><a href="https://www.w3.org/TR/2008/REC-xml-20081126/#Unicode">[Unicode]</a></span>;
the mechanisms for signaling which of the two is in use,
or for bringing other encodings into play, are discussed later, in <a href="https://www.w3.org/TR/2008/REC-xml-20081126/#charencoding"><b>4.3.3 Character Encoding in Entities</b></a>.

<b>Note:</b>

Document authors are encouraged to avoid
"compatibility characters", as defined
in section <span>2.3</span> of <a href="https://www.w3.org/TR/2008/REC-xml-20081126/#Unicode">[Unicode]</a>. The characters defined in the following ranges are also
discouraged. They are either control characters or permanently undefined Unicode
characters:

<pre>[#x7F-#x84], [#x86-#x9F], [#xFDD0-#xFD<span>E</span>F],
[#x1FFFE-#x1FFFF], [#x2FFFE-#x2FFFF], [#x3FFFE-#x3FFFF],
[#x4FFFE-#x4FFFF], [#x5FFFE-#x5FFFF], [#x6FFFE-#x6FFFF],
[#x7FFFE-#x7FFFF], [#x8FFFE-#x8FFFF], [#x9FFFE-#x9FFFF],
[#xAFFFE-#xAFFFF], [#xBFFFE-#xBFFFF], [#xCFFFE-#xCFFFF],
[#xDFFFE-#xDFFFF], [#xEFFFE-#xEFFFF], [#xFFFFE-#xFFFFF],
[#x10FFFE-#x10FFFF].</pre>

## White Space and original contextual note

<a href="#NT-S">S</a> (white space) consists of one or more space (#x20)
characters, carriage returns, line feeds, or tabs.

<a id="white"></a>

### White Space

<table class="scrap" summary="Scrap"><tbody><tr valign="baseline"><td><a id="NT-S"></a>[3]   </td><td><code>S</code></td><td>   ::=   </td><td><code>(#x20 | #x9 | #xD | #xA)+</code></td></tr></tbody></table>

<b>Note:</b>

The presence of #xD in the above production is
	maintained purely for backward compatibility with the
	<a href="https://www.w3.org/TR/1998/REC-xml-19980210">First Edition</a>.
	As explained in <a href="https://www.w3.org/TR/2008/REC-xml-20081126/#sec-line-ends"><b>2.11 End-of-Line Handling</b></a>,
	all #xD characters literally present in an XML document
	are either removed or replaced by #xA characters before
	any other processing is done. The only way to get a #xD character to match this production is to
  use a character reference in an entity value literal.

## Name context and three selected complete production rows

An <a href="https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Nmtoken">Nmtoken</a> (name token) is any mixture of name
characters.

[<a id="dt-name" title="Name">Definition</a>: A <a href="#NT-Name">Name</a> is an <a href="https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Nmtoken">Nmtoken</a> with a restricted set of initial characters.]  Disallowed initial characters for <a href="#NT-Name">Names</a> include digits, diacritics, the full stop and the hyphen.

Names beginning with the string "<code>xml</code>",
or with any string which would match <code>(('X'|'x') ('M'|'m') ('L'|'l'))</code>,
are reserved for standardization in this or future versions of this specification.

<b>Note:</b>

The
Namespaces in XML Recommendation <a href="https://www.w3.org/TR/2008/REC-xml-20081126/#xml-names">[XML Names]</a> assigns a meaning
to names containing colon characters. Therefore, authors should not use the
colon in XML names except for namespace purposes, but XML processors must
accept the colon as a name character.

The first character of a <a href="#NT-Name">Name</a> <em class="rfc2119" title="Keyword in RFC 2119 context">MUST</em> be a <a href="#NT-NameStartChar">NameStartChar</a>, and any
					other characters <em class="rfc2119" title="Keyword in RFC 2119 context">MUST</em> be <a href="#NT-NameChar">NameChars</a>; this mechanism is used to
					prevent names from beginning with European (ASCII) digits or with
					basic combining characters. Almost all characters are permitted in
					names, except those which either are or reasonably could be used as
					delimiters. The intention is to be inclusive rather than exclusive,
					so that writing systems not yet encoded in Unicode can be used in
					XML names. See <a href="https://www.w3.org/TR/2008/REC-xml-20081126/#sec-suggested-names"><b>J Suggestions for XML Names</b></a> for suggestions on the creation of
					names.

Document authors are encouraged to use names which are
					meaningful words or combinations of words in natural languages, and
					to avoid symbolic or white space characters in names. Note that
					COLON, HYPHEN-MINUS, FULL STOP (period), LOW LINE (underscore), and
					MIDDLE DOT are explicitly permitted.

The ASCII symbols and punctuation marks, along with a fairly
					large group of Unicode symbol characters, are excluded from names
					because they are more useful as delimiters in contexts where XML
					names are used outside XML documents; providing this group gives
					those contexts hard guarantees about what <em>cannot</em> be part of
					an XML name. The character #x037E, GREEK QUESTION MARK, is excluded
					because when normalized it becomes a semicolon, which could change
					the meaning of entity references.

<a id="d0e804"></a>

### Names and Tokens

<table class="scrap" summary="Scrap"><tbody><tr valign="baseline"><td><a id="NT-NameStartChar"></a>[4]   </td><td><code>NameStartChar</code></td><td>   ::=   </td><td><code>":" | [A-Z] | "_" | [a-z] | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]</code></td></tr></tbody><tbody><tr valign="baseline"><td><a id="NT-NameChar"></a>[4a]   </td><td><code>NameChar</code></td><td>   ::=   </td><td><code><a href="#NT-NameStartChar">NameStartChar</a> | "-" | "." | [0-9] | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]</code></td></tr></tbody><tbody><tr valign="baseline"><td><a id="NT-Name"></a>[5]   </td><td><code>Name</code></td><td>   ::=   </td><td><code><a href="#NT-NameStartChar">NameStartChar</a> (<a href="#NT-NameChar">NameChar</a>)*</code></td></tr></tbody></table>

## Character Reference and Legal Character constraint

<a id="d0e3895"></a>

### Character Reference

<table class="scrap" summary="Scrap"><tbody><tr valign="baseline"><td><a id="NT-CharRef"></a>[66]   </td><td><code>CharRef</code></td><td>   ::=   </td><td><code>'&amp;#' [0-9]+ ';' </code></td></tr><tr valign="baseline"><td></td><td></td><td></td><td><code>| '&amp;#x' [0-9a-fA-F]+ ';'</code></td><td><a href="#wf-Legalchar">[WFC: Legal Character]</a></td></tr></tbody></table>

<a id="wf-Legalchar"></a><b>Well-formedness constraint: Legal Character</b>

Characters referred
to using character references <em class="rfc2119" title="Keyword in RFC 2119 context">MUST</em> match the production for <a href="#NT-Char">Char</a>.

If the character reference begins with "
						<code>&amp;#x</code>
					",
the digits and letters up to the terminating <code>;</code> provide a hexadecimal
representation of the character's code point in ISO/IEC 10646. If it begins
just with "
						<code>&amp;#</code>
					", the digits up to the terminating <code>;</code>
provide a decimal representation of the character's code point.

## Complete original §6 Notation

<a id="sec-notation"></a>

### 6 Notation

The formal grammar of XML is given in this specification using a simple
Extended Backus-Naur Form (EBNF) notation. Each rule in the grammar defines
one symbol, in the form

<pre>symbol ::= expression</pre>

Symbols are written with an initial capital letter if they are the
start symbol of a regular language, otherwise with an initial lowercase letter.
Literal strings are quoted.

Within the expression on the right-hand side of a rule, the following expressions
are used to match strings of one or more characters:

<dl><dt class="label">
							<code>#xN</code>
						</dt><dd><p>where <code>N</code> is a hexadecimal integer, the expression matches the character
whose number
(code point) in ISO/IEC 10646 is <code>N</code>. The number of leading zeros in the <code>#xN</code>
form is insignificant.</p></dd><dt class="label">
							<code>[a-zA-Z]</code>, <code>[#xN-#xN]</code>
						</dt><dd><p>matches any <a href="#NT-Char">Char</a> with a value in the range(s) indicated (inclusive).</p></dd><dt class="label">
							<code>[abc]</code>, <code>[#xN#xN#xN]</code>
						</dt><dd><p>matches any <a href="#NT-Char">Char</a> with a value among the characters
enumerated. Enumerations and ranges can be mixed in one set of brackets.</p></dd><dt class="label">
							<code>[^a-z]</code>, <code>[^#xN-#xN]</code>
						</dt><dd><p>matches any <a href="#NT-Char">Char</a> with a value <em>outside</em> the range
indicated.</p></dd><dt class="label">
							<code>[^abc]</code>, <code>[^#xN#xN#xN]</code>
						</dt><dd><p>matches any <a href="#NT-Char">Char</a> with a value not among the characters given. Enumerations
and ranges of forbidden values can be mixed in one set of brackets.</p></dd><dt class="label">
							<code>"string"</code>
						</dt><dd><p>matches a literal string <a title="match" href="https://www.w3.org/TR/2008/REC-xml-20081126/#dt-match">matching</a> that
given inside the double quotes.</p></dd><dt class="label">
							<code>'string'</code>
						</dt><dd><p>matches a literal string <a title="match" href="https://www.w3.org/TR/2008/REC-xml-20081126/#dt-match">matching</a> that
given inside the single quotes.</p></dd></dl>

These symbols may be combined to match more complex patterns as follows,
where <code>A</code> and <code>B</code> represent simple expressions:

<dl><dt class="label">(<code>expression</code>)</dt><dd><p>
								<code>expression</code> is treated as a unit and may be combined as described
in this list.</p></dd><dt class="label">
							<code>A?</code>
						</dt><dd><p>matches <code>A</code> or nothing; optional <code>A</code>.</p></dd><dt class="label">
							<code>A B</code>
						</dt><dd><p>matches <code>A</code> followed by <code>B</code>. This
operator has higher precedence than alternation; thus <code>A B | C D</code>
is identical to <code>(A B) | (C D)</code>.</p></dd><dt class="label">
							<code>A | B</code>
						</dt><dd><p>matches <code>A</code> or <code>B</code>.</p></dd><dt class="label">
							<code>A - B</code>
						</dt><dd><p>matches any string that matches <code>A</code> but does not match <code>B</code>.</p></dd><dt class="label">
							<code>A+</code>
						</dt><dd><p>matches one or more occurrences of <code>A</code>. Concatenation
has higher precedence than alternation; thus <code>A+ | B+</code> is identical
to <code>(A+) | (B+)</code>.</p></dd><dt class="label">
							<code>A*</code>
						</dt><dd><p>matches zero or more occurrences of <code>A</code>. Concatenation
has higher precedence than alternation; thus <code>A* | B*</code> is identical
to <code>(A*) | (B*)</code>.</p></dd></dl>

Other notations used in the productions are:

<dl><dt class="label">
							<code>/* ... */</code>
						</dt><dd><p>comment.</p></dd><dt class="label">
							<code>[ wfc: ... ]</code>
						</dt><dd><p>well-formedness constraint; this identifies by name a constraint on <a title="Well-Formed" href="https://www.w3.org/TR/2008/REC-xml-20081126/#dt-wellformed">well-formed</a> documents associated with a production.</p></dd><dt class="label">
							<code>[ vc: ... ]</code>
						</dt><dd><p>validity constraint; this identifies by name a constraint on <a title="Validity" href="https://www.w3.org/TR/2008/REC-xml-20081126/#dt-valid">valid</a>
documents associated with a production.</p></dd></dl>

## Selection boundary

XML §6 defines negated character classes over Char; they do not admit arbitrary
Unicode or UTF-16 code units. Permitted-but-discouraged characters in the original
note are not prohibited by Char. NameStartChar has its own upper bound, different
from Char's. The three selected Names and Tokens rows are complete; Names,
Nmtoken and Nmtokens productions are not reproduced.

The original S note describes preprocessing an XML document. The delegated
operation receives an existing string, so this reference does not import that
document preprocessing into PI pseudo-attribute parsing. The original name
reservation and namespace guidance remain source context; they are not additional
Name-production restrictions. No XML document/DTD processing, external entity
resolution or general attribute-value normalization is selected.

Errata links and original status remain visible. No pending erratum or
normalization advice has been silently substituted for the selected productions.
The [edition mapping](SOURCE-EDITIONS.md#dom-delegated-xml-pseudo-attribute-grammar)
records the bounded caller and source identity; reference retention is not
implementation acceptance.
