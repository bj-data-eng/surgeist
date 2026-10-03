Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Errata in REC-CSS2-20110607](https://www.w3.org/Style/css2-updates/REC-CSS2-20110607-errata.html).

Original copyright notice: Copyright © 2011–2012 W3C®

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Errata in REC-CSS2-20110607

Source snapshot: https://www.w3.org/Style/css2-updates/REC-CSS2-20110607-errata.html

Snapshot SHA-256: 36b37eee597352e0e077a839eb2e9a02de55f89e9916fd385276678a71fe0428

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 6 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

[![W3C](https://www.w3.org/Icons/w3c_home.gif)](https://www.w3.org/)

# Errata in REC-CSS2-20110607

This document:  
[http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;css2-updates&#x2F;REC-CSS2-20110607-errata&#x2E;html](css2--REC-CSS2-20110607-errata.html--36b37eee5973.md)

Last revised:  
\$Date: 2017/08/03 08:11:39 \$

This document records known errors in the document [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2011&#x2F;REC-CSS2-20110607](css2--REC-CSS2-20110607--1e43327015ed.md)

These errata have the same status as a <em>Working Draft.</em>

- <a id="s.6.2.1"></a>

  \[2011-10-12\] In [“6.2.1 The 'inherit' value,”](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) change

  > Each property may also have a cascaded value of 'inherit', which means that, for a given element, the property takes ~~the same specified value as the property for~~ <u>as specified value the computed value of</u> the element's parent.

  (See [CSS WG minutes 2011-10-12.](http://lists.w3.org/Archives/Public/www-style/2011Oct/0482.html))

- <a id="s.6.1.1"></a>

  \[2011-10-12\] In [“6.1.1 Specified values,”](css2--cascade.html--c7aff33e6f0d.md#specified-value) add this clarification:

  > 1.  If the cascade results in a value use it. <u>Except that, if the value is 'inherit', the specified value is defined in [“The 'inherit' value”](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) below</u>

  (See [CSS WG minutes 2011-10-12.](http://lists.w3.org/Archives/Public/www-style/2011Oct/0482.html))

- <a id="s.8.3.1c"></a>

  \[2012-04-04\] In [“8.3.1 Collapsing margins,”](css2--box.html--8875bbcdefe6.md#collapsing-margins) add a new item as follows:

  > Adjoining vertical margins collapse, except:
  >
  > - Margins of the root element's box do not collapse.
  > - If the top and bottom margins of an element with clearance are adjoining, its margins collapse with the adjoining margins of following siblings but that resulting margin does not collapse with the bottom margin of the parent block.
  > - <u>If the top margin of a box with non-zero computed 'min-height' and 'auto' computed 'height' collapses with the bottom margin of its last in-flow child, then the child's bottom margin does not collapse with the parent's bottom margin.</u>

  (See [CSS WG minutes 2012-04-04.](http://lists.w3.org/Archives/Public/www-style/2012Apr/0101.html))

- <a id="s.10.7"></a>

  \[2012-04-11\] In [“10.7 Minimum and maximum heights: 'min-height' and 'max-height',”](css2--visudet.html--12e8bc0e6b7c.md#min-max-heights) clarify the note as follows:

  > ~~These steps do not affect the real computed values of the above properties. The change of used 'height' has no effect on margin collapsing except as specifically required by rules for 'min-height' or 'max-height' in "Collapsing margins" (8.3.1).~~
  >
  > <u>These steps do not affect the real computed value of 'height'. Consequently, for example, they do not affect margin collapsing, which depends on the computed value.</u>

  (See [CSS WG minutes 2012-04-11.](http://lists.w3.org/Archives/Public/www-style/2012Apr/0276.html))

- <a id="s.8.3.1d"></a>

  \[2012-04-11\] In [“8.3.1 Collapsing margins,”](css2--box.html--8875bbcdefe6.md#collapsing-margins) clarify 7th bullet in the 2nd note:

  > ~~The bottom margin of an in-flow block box with a 'height' of 'auto' and a 'min-height' of zero collapses with its last in-flow block-level child's bottom margin if the box has no bottom padding and no bottom border and the child's bottom margin does not collapse with a top margin that has clearance.~~
  >
  > <u>The bottom margin of an in-flow block box with a 'height' of 'auto' collapses with its last in-flow block-level child's bottom margin, if:</u>
  >
  > - <u>the box has no bottom padding, and</u>
  > - <u>the box has no bottom border, and</u>
  > - <u>the child's bottom margin neither collapses with a top margin that has clearance, nor (if the box's min-height is non-zero) with the box's top margin.</u>

  (See [CSS WG minutes 2012-04-11.](http://lists.w3.org/Archives/Public/www-style/2012Apr/0276.html))

- <a id="s.15.3a"></a>

  \[2012-05-02\] In [“15.3 Font family: the 'font-family' property,”](css2--fonts.html--d52fc14f36c2.md#font-family-prop) clarify that using the font name “inherit” <em>without</em> quotes is an error:

  > ~~Font family <em>names</em> that happen to be the same as a keyword value ('inherit', 'serif', 'sans-serif', 'monospace', 'fantasy', and 'cursive') must be quoted to prevent confusion with the keywords with the same names. The keywords 'initial' and 'default' are reserved for future use and must also be quoted when used as font names. UAs must not consider these keywords as matching the '\<family-name\>' type.~~
  >
  > <u>Unquoted font family <em>names</em> that happen to be the same as the keyword values 'inherit', 'default' and 'initial' or the generic font keywords ('serif', 'sans-serif', 'monospace', 'fantasy', and 'cursive') do not match the '\<family-name\>' type. These names must be quoted to prevent confusion with the keywords with the same names. Note that 'font-family: Times, inherit' is therefore an invalid declaration, because 'inherit' in that position can neither be a valid keyword nor a valid font family name.</u>

  (See [CSS WG minutes 2012-05-02.](http://lists.w3.org/Archives/Public/www-style/2012May/0063.html))

- <a id="s.4.3.1"></a>

  \[2012-05-02\] Spaces and comments are not allowed between the sign and the digits of a \<number\>, \<length\> or \<percentage\>. In [“4.3.1 Integers and real numbers,”](css2--syndata.html--02e71c159e14.md#numbers) insert “immediately” as follows:

  > Both integers and real numbers may <u>immediately</u> be preceded by a "-" or "+" to indicate the sign.

  In [“4.1.1 Tokenization,”](css2--syndata.html--02e71c159e14.md#tokenization) allow "+" or "-" at the start of the {num} macro:

  > <strong>Table 1 — structured row/cell transcription</strong>
  >
  > <strong>Row 1</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > num
  > <strong>Column 2 (data cell):</strong>
  >
  > `[-+]?[0-9]+|[-+]?[0-9]*\.[0-9]+`

  (Note that this changes the definition of three tokens, NUMBER, DIMENSION and PERCENTAGE, and thus the tokenization of CSS, but it does not change the language generated by the grammar as a whole.)

  No change is required in 4.3.2 (\<length\>) or 4.3.3 (\<percentage\>), because they refer to 4.3.1.

  (See [CSS WG minutes 2012-05-02.](http://lists.w3.org/Archives/Public/www-style/2012May/0063.html))

- <a id="s.10.1a"></a>

  \[2012-08-01\] In [10.1 “Definition of "containing block,"”](css2--visudet.html--12e8bc0e6b7c.md#containing-block-details) change:

  > 1.  \[…\]
  > 2.  For other elements, if the element's position is 'relative' or 'static', the containing block is formed by the content edge of the nearest <u>ancestor box that is a</u> [block container](css2--visuren.html--3f334c530cf4.md#block-boxes) ~~ancestor box~~ <u>or which establishes a formatting context</u>.

  (See [CSS WG minutes 2012-08-01.](http://lists.w3.org/Archives/Public/www-style/2012Aug/0087.html))

- <a id="s.9.4"></a>

  \[2012-08-01\] In [9.4 “Normal flow,”](css2--visuren.html--3f334c530cf4.md#normal-flow) replace as follows:

  > Boxes in the normal flow belong to a formatting context, which <u>in CSS 2</u> may be <u>table,</u> block or inline~~, but not both simultaneously~~. <u>In future levels of CSS, other types of formatting context will be introduced.</u> [Block-level](https://www.w3.org/TR/2011/REC-CSS2-20110607/#block-level) boxes participate in a [block formatting](https://www.w3.org/TR/2011/REC-CSS2-20110607/#block-formatting) context. [Inline-level boxes](https://www.w3.org/TR/2011/REC-CSS2-20110607/#inline-level) participate in an [inline formatting](https://www.w3.org/TR/2011/REC-CSS2-20110607/#inline-formatting) context. <u>Table formatting contexts are described in the [chapter on tables.](css2--tables.html--201812dd6e3c.md)</u>

  (See [CSS WG minutes 2012-08-01.](http://lists.w3.org/Archives/Public/www-style/2012Aug/0087.html))

- <a id="s.9.4.2"></a>

  \[2012-08-01\] In [9.4.2 “Inline formatting contexts,”](css2--visuren.html--3f334c530cf4.md#inline-formatting) add this sentence:

  > <u>An inline formatting context is established by a block container box that contains no block-level boxes.</u> In an inline formatting context, boxes are laid out horizontally, one after the other, beginning at the top of a containing block.

  (See [CSS WG minutes 2012-08-01.](http://lists.w3.org/Archives/Public/www-style/2012Aug/0087.html))

- <a id="s.17.4a"></a>

  \[2012-08-01\] In [17.4 “Tables in the visual formatting model,”](css2--tables.html--201812dd6e3c.md#model) replace as follows:

  > The table wrapper box is a 'block' box if the table is block-level, and an 'inline-block' box if the table is inline-level. The table wrapper box establishes a block formatting context<u>, and the table box establishes a table formatting context</u>.

  (See [CSS WG minutes 2012-08-01.](http://lists.w3.org/Archives/Public/www-style/2012Aug/0087.html))

- <a id="s.17.5"></a>

  \[2012-08-01\] In [17.5 “Visual layout of table contents,”](css2--tables.html--201812dd6e3c.md#table-layout) replace as follows:

  > Internal table elements generate rectangular [boxes](css2--box.html--8875bbcdefe6.md#box-dimensions) ~~with~~ <u>which participate in the table formatting context established by the table box. These boxes have</u> content and borders~~.~~ <u>and</u> cells have padding as well. Internal table elements do not have margins.

  (See [CSS WG minutes 2012-08-01.](http://lists.w3.org/Archives/Public/www-style/2012Aug/0087.html))

- <a id="s.11.1.1a"></a>

  \[2012-08-01\] In [11.1.1 “Overflow: the 'overflow' property,”](css2--visufx.html--2bc674cb7ab0.md#overflow) change:

  > <em>Applies to:</em> block containers <u>and boxes that establish a formatting context</u>

  (See [CSS WG minutes 2012-08-01.](http://lists.w3.org/Archives/Public/www-style/2012Aug/0087.html))

- <a id="s.4.1.1a"></a>

  \[2013-04-29\] The letters u, r and l of the URI token may be written as escapes. In [4.1.1 “Tokenization,”](css2--syndata.html--02e71c159e14.md#tokenization) change in the first table:

  > <strong>Table 2 — structured row/cell transcription</strong>
  >
  > <strong>Row 1</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > URI
  > <strong>Column 2 (data cell):</strong>
  >
  > <code><del>url</del><ins>{U}{R}{L}</ins>&#x5C;(<var>{w}{string}{w}</var>&#x5C;)<br>&#xA;&#x9;  |<del>url</del><ins>{U}{R}{L}</ins>&#x5C;(<var>{w}</var>(&#x5B;!#$%&amp;&#x2A;-&#x5C;&#x5B;&#x5C;&#x5D;-&#x7E;&#x5D;|<var>{nonascii}</var>|<var>{escape}</var>)&#x2A;<var>{w}</var>&#x5C;)</code>

  and in the second table:

  > <strong>Table 3 — structured row/cell transcription</strong>
  >
  > <strong>Row 1</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > baduri1
  > <strong>Column 2 (data cell):</strong>
  >
  > <code><del>url</del><ins>{U}{R}{L}</ins>&#x5C;(<var>{w}</var>(&#x5B;!#$%&amp;&#x2A;-&#x7E;&#x5D;|<var>{nonascii}</var>|<var>{escape}</var>)&#x2A;<var>{w}</var></code>
  > <strong>Row 2</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > baduri2
  > <strong>Column 2 (data cell):</strong>
  >
  > <code><del>url</del><ins>{U}{R}{L}</ins>&#x5C;(<var>{w}</var><var>{string}</var><var>{w}</var></code>
  > <strong>Row 3</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > baduri3
  > <strong>Column 2 (data cell):</strong>
  >
  > <code><del>url</del><ins>{U}{R}{L}</ins>&#x5C;(<var>{w}</var><var>{badstring}</var></code>

  And add to the second table:

  > <strong>Table 4 — structured row/cell transcription</strong>
  >
  > <strong>Row 1</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > <u>L</u>
  > <strong>Column 2 (data cell):</strong>
  >
  > <u>`l|\\0{0,4}(4c|6c)(\r\n|[ \t\r\n\f])?|\\l`</u>
  > <strong>Row 2</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > <u>R</u>
  > <strong>Column 2 (data cell):</strong>
  >
  > <u>`r|\\0{0,4}(52|72)(\r\n|[ \t\r\n\f])?|\\r`</u>
  > <strong>Row 3</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > <u>U</u>
  > <strong>Column 2 (data cell):</strong>
  >
  > <u>`u|\\0{0,4}(55|75)(\r\n|[ \t\r\n\f])?|\\u`</u>

  (See [CSS WG minutes 2013-01-30.](http://lists.w3.org/Archives/Public/www-style/2013Jan/0616.html))

- <a id="s.G.2d"></a>

  \[2013-04-29\] The letters u, r and l of the URI token may be written as escapes (see the [previous errata](https://www.w3.org/TR/2011/REC-CSS2-20110607/#s.4.1.1)). In [G.2 “Lexical scanner,”](css2--grammar.html--eeb984c469da.md#scanner) change:

  > ```text
  > 
  > baduri1         url{U}{R}{L}\({w}([!#$%&*-\[\]-~]|{nonascii}|{escape})*{w}
  > baduri2         url{U}{R}{L}\({w}{string}{w}
  > baduri3         url{U}{R}{L}\({w}{badstring}
  > ```
  and

  > ```text
  > 
  > "url("{U}{R}{L}"("{w}{string}{w}")" {return URI;}
  > "url("{U}{R}{L}"("{w}{url}{w}")"    {return URI;}
  > ```
  (See [CSS WG minutes 2013-01-30.](http://lists.w3.org/Archives/Public/www-style/2013Jan/0616.html))

- <a id="s.4.1.1b"></a>

  \[2013-04-29\] Unicode control characters between U+0080 and U+009F can be used in identifiers and in URI tokens. (Previously such characters made the style sheet invalid.) In [4.1.1 “Tokenization,”](css2--syndata.html--02e71c159e14.md#tokenization) change:

  > |          |                                                                       |
  > |----------|-----------------------------------------------------------------------|
  > | nonascii | \[^&#x5C;0-~~&#x5C;237~~<u>&#x5C;177</u>\] |

  (See [CSS WG minutes 2013-01-30.](http://lists.w3.org/Archives/Public/www-style/2013Jan/0616.html))

- <a id="s.4.1.3d"></a>

  \[2013-04-29\] Unicode control characters between U+0080 and U+009F can be used in identifiers and in URI tokens. (Previously such characters made the style sheet invalid.) In [4.1.3 “Characters and case,”](css2--syndata.html--02e71c159e14.md#characters) change:

  > - In CSS, identifiers (including element names, classes, and IDs in [selectors](css2--selector.html--f66f7c932788.md)) can contain only the characters \[a-zA-Z0-9\] and ISO 10646 characters ~~U+00A0~~ <u>U+0080</u> and higher, plus the hyphen (-) and the underscore (\_);

  (See [CSS WG minutes 2013-01-30.](http://lists.w3.org/Archives/Public/www-style/2013Jan/0616.html))

- <a id="s.4.4"></a>

  \[2013-05-02\] When a CSS file is known to be in a UTF-based character encoding, based on out-of-band information, and the file starts with a BOM, then the BOM determines which of the UTF-based encodings is used, overriding the out-of-band information. In [4.4 “CSS style sheet representation,”](css2--syndata.html--02e71c159e14.md#charset) insert:

  > <u>If rule 1 above (an HTTP "charset" parameter or similar) yields a character encoding and it is one of UTF-8, UTF-16 or UTF-32, then a BOM, if any, at the start of the file overrides that character encoding, as follows:</u>
  >
  > | <u>First bytes (hexadecimal)</u> | <u>Resulting encoding</u>    |
  > |----------------------------------|------------------------------|
  > | <u>00 00 FE FF</u>               | <u>UTF-32, big-endian</u>    |
  > | <u>FF FE 00 00</u>               | <u>UTF-32, little-endian</u> |
  > | <u>FE FF</u>                     | <u>UTF-16, big-endian</u>    |
  > | <u>FF FE</u>                     | <u>UTF-16, little-endian</u> |
  > | <u>EF BB BF</u>                  | <u>UTF-8</u>                 |
  >
  > <u>If rule 1 yields a character encoding of UTF-16BE, UTF-16LE, UTF-32BE or UTF-32LE, then it is an error if the file starts with a BOM. A CSS UA must recover by ignoring the specified encoding and using the table above.</u>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > <u>Note that the fact that a BOM at the start of a file is an error in UTF-16BE, UTF-16LE, UTF-32BE or UTF-32LE is specified by [\[UNICODE\]](css2--refs.html--f208d881d0b7.md#ref-UNICODE).</u>

  (See [CSS WG minutes 2012-10-24.](http://lists.w3.org/Archives/Public/www-style/2012Oct/0727.html))

- <a id="s.11.1.1b"></a>

  \[2013-07-15\] In [11.1.1 “Overflow: the 'overflow' property,”](css2--visufx.html--2bc674cb7ab0.md#overflow) change the definition of 'scroll' and 'auto':

  > <strong>scroll</strong>  
  > This value indicates that the content is clipped and that if the user agent uses a scrolling mechanism that is visible on the screen (such as a scroll bar or a panner), that mechanism should be displayed for a box whether or not any of its content is clipped. This avoids any problem with scrollbars appearing and disappearing in a dynamic environment. When this value is specified and the target medium is 'print', overflowing content may be printed. <u>When used on [table boxes,](https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#table-box) this value has the same meaning as 'visible'.</u>
  >
  > <strong>auto</strong>  
  > The behavior of the 'auto' value is user agent-dependent, but should cause a scrolling mechanism to be provided for overflowing boxes. <u>When used on [table boxes,](https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#table-box) this value has the same meaning as 'visible'.</u>

  (See [CSS WG minutes 2012-08-08.](http://lists.w3.org/Archives/Public/www-style/2012Aug/0298.html))

- <a id="s.15.3b"></a>

  \[2013-07-15\] In [15.3 Font family: the 'font-family' property](css2--fonts.html--d52fc14f36c2.md#font-family-prop) the grammar is missing a pair of brackets:

  > Value: \[\[ \<family-name\> \| \<generic-family\> \] \[, <u>\[</u> \<family-name\>\| \<generic-family\> \] <u>\]</u>\* \] \| inherit

  (See [CSS WG minutes 2012-05-03.](http://lists.w3.org/Archives/Public/www-style/2012May/0887.html))

- <a id="s.G.1a"></a>

  \[2013-07-15\] Spaces and comments are not allowed between the sign and the digits of a \<number\>, \<length\> or \<percentage\>. In [“G.2 Lexical scanner,”](css2--grammar.html--eeb984c469da.md#scanner) change the {num} macro as follows:

  > ```text
  > num		[-+]?[0-9]+|[-+]?[0-9]*"."[0-9]+
  > ```
  In [“G.1 Grammar,”](css2--grammar.html--eeb984c469da.md#grammar) remove unary_operator from the grammar:

  > ```text
  > 
  > unary_operator
  >   : '-' | '+'
  >   ;
  > ```
  and

  > ```text
  > 
  > term
  >   : unary_operator?
  >     [ NUMBER S* | PERCENTAGE S* | LENGTH S* | EMS S* | EXS S* | ANGLE S* |
  >       TIME S* | FREQ S* ]
  >   | STRING S* | IDENT S* | URI S* | hexcolor | function
  >   ;
  > ```
  (See [CSS WG minutes 2012-05-02.](http://lists.w3.org/Archives/Public/www-style/2012May/0063.html))

- <a id="s.10.5a"></a>

  \[2013-07-18\] A percentage on 'height', even if not used, can be inherited. In [“10.5 Content height: the 'height' property,”](css2--visudet.html--12e8bc0e6b7c.md#the-height-property) change the “computed value” line as follows:

  > |                       |                                                                                                                                                                       |
  > |-----------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
  > | <em>Computed&#xA0;value:</em>   | the percentage or 'auto' ~~(see prose under [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage))~~ <u>(as specified)</u> |

  and in the definition of \<percentage\>:

  > <strong>&lt;percentage&gt;</strong>  
  > Specifies a percentage height. The percentage is calculated with respect to the height of the generated box's [containing block](css2--visuren.html--3f334c530cf4.md#containing-block). If the height of the containing block is not specified explicitly (i.e., it depends on content height), and this element is not absolutely positioned, ~~the value computes to 'auto'~~ <u>the used height is calculated as if 'auto' was specified</u>.

  (See [CSS WG minutes 2013-05-08.](http://lists.w3.org/Archives/Public/www-style/2013May/0201.html))

- <a id="s.4.1.1c"></a>

  \[2013-09-09\] In [“4.1.1 Tokenization,”](css2--syndata.html--02e71c159e14.md#tokenization) make the UNICODE-RANGE token more precise:

  > <strong>Table 8 — structured row/cell transcription</strong>
  >
  > <strong>Row 1</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > UNICODE-RANGE
  > <strong>Column 2 (data cell):</strong>
  >
  > ```text
  > u\+[0-9a-f?]{1,6}(-[0-9a-f]{1,6})?
  > 	  u\+[?]{1,6}|
  > 	   u\+[0-9a-f]{1}[?]{0,5}|
  > 	   u\+[0-9a-f]{2}[?]{0,4}|
  > 	   u\+[0-9a-f]{3}[?]{0,3}|
  > 	   u\+[0-9a-f]{4}[?]{0,2}|
  > 	   u\+[0-9a-f]{5}[?]{0,1}|
  > 	   u\+[0-9a-f]{6}|
  > 	   u\+[0-9a-f]{1,6}-[0-9a-f]{1,6}
  > ```
  E.g., “U+A?5” previously was a single UNICODE-RANGE token (although a semantically meaningless one), now this is two tokens: “U+A?” (meaning the 16-character range U+A0-AF) and the number “5”.

  (See [CSS WG minutes 2013-09-03.](https://www.w3.org/mid/5227C33B.9010408@inkedblade.net))

- <a id="s.9.2a"></a>

  \[2012-09-19\] Modify [“9.2 Controlling box generation”](css2--visuren.html--3f334c530cf4.md#box-gen) and [“9.2.1 Block-level elements and block boxes”](css2--visuren.html--3f334c530cf4.md#anonymous-block-level) as follows:

  > <strong>Controlling box generation</strong>
  >
  > The following sections describe the types of boxes that may be generated in CSS 2.1. A box's type affects, in part, its behavior in the visual formatting model. The 'display' property, described below, specifies a box's type.
  >
  > <u>Certain values of the ''display' property cause an element of the source document to generate a principal box that contains descendant boxes and generated content and is also the box involved in any positioning scheme. Some elements may generate additional boxes in addition to the principal box: 'list-item' elements. These additional boxes are placed with respect to the principal box.</u>
  >
  > <strong>9.2.1 Block-level elements and block boxes</strong>
  >
  > ~~Block-level elements are those elements of the source document that are formatted visually as blocks (e.g., paragraphs). The following values of the 'display' property make an element block-level: 'block', 'list-item', and 'table'.~~
  >
  > <u>Block-level elements – those elements of the source document that are formatted visually as blocks (e.g., paragraphs) – are elements which generate a block-level principal box. Values of the 'display' property that make an element block-level include: 'block', 'list-item', and 'table'. Block-level boxes are boxes that participate in a block formatting context.</u>
  >
  > ~~Block-level boxes are boxes that participate in a block formatting context. Each block-level element generates a principal block-level box that contains descendant boxes and generated content and is also the box involved in any positioning scheme. Some block-level elements may generate additional boxes in addition to the principal box: 'list-item' elements. These additional boxes are placed with respect to the principal box.~~
  >
  > ~~Except for table boxes, which are described in a later chapter, and replaced elements,~~ <u>In CSS 2,</u> a block-level box is also a block container box <u>unless it is a table box or the principal box of a replaced element</u>. A block container box either contains only block-level boxes or establishes an inline formatting context and thus contains only inline-level boxes. <u>An element whose principal box is a block container box is a block container element.</u> Values of the 'display' property which make a non-replaced element generate a block container include 'block', 'list-item' and 'inline-block'. Not all block container boxes are block-level boxes: non-replaced inline blocks and non-replaced table cells are block containers but <u>are</u> not block-level ~~boxes~~. Block-level boxes that are also block containers are called block boxes.
  >
  > The three terms "block-level box," "block container box," and "block box" are sometimes abbreviated as "block" where unambiguous.

- <a id="s.9.2.4a"></a>

  \[2012-09-19\] Modify [“9.2.4 The 'display' property”](css2--visuren.html--3f334c530cf4.md#display-prop) as follows:

  > <strong>block</strong>  
  > This value causes an element to generate a <u>principal</u> block box.
  >
  > <strong>inline-block</strong>  
  > This value causes an element to generate ~~an~~ <u>a principal</u> inline-level block container. <u>(</u>The inside of an inline-block is formatted as a block box, and the element itself is formatted as an atomic inline-level box.<u>)</u>

- <a id="s.17.4b"></a>

  \[2012-09-19\] Modify [“17.4 Tables in the visual formatting model”](css2--tables.html--201812dd6e3c.md#model) as follows:

  > In both cases, the table generates a principal block <u>container</u> box called the table wrapper box that contains the table box itself and any caption boxes (in document order). The table box is a block-level box that contains the table's internal table boxes. The caption boxes are <u>principal</u> block-level boxes that retain their own content, padding, margin, and border areas, and are rendered as normal block boxes inside the table wrapper box. Whether the caption boxes are placed before or after the table box is decided by the 'caption-side' property, as described below.
  >
  > The table wrapper box is ~~a 'block' box if the table is block-level~~ <u>block-level for 'display: table'</u>, and ~~an 'inline-block' box if the table is inline-level~~ <u>inline-level for 'display: inline-table'</u>. The table wrapper box establishes a block formatting context, and the table box establishes a table formatting context. The table box (not the table wrapper box) is used when doing baseline vertical alignment for an 'inline-table'. The width of the table wrapper box is the border-edge width of the table box inside it, as described by section 17.5.2. Percentages on 'width' and 'height' on the table are relative to the table wrapper box's containing block, not the table wrapper box itself.

- <a id="s.4.1.1d"></a>

  \[2012-09-19\] For compatibility with SVG, modify the definition of macro num in [“4.1.1 Tokenization”](css2--syndata.html--02e71c159e14.md#tokenization) as follows:

  > <strong>Table 9 — structured row/cell transcription</strong>
  >
  > <strong>Row 1</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > ~~num~~
  > <strong>Column 2 (data cell):</strong>
  >
  > ~~`[-+]?[0-9]+|[-+]?[0-9]*\.[0-9]+`~~
  > <strong>Row 2</strong>
  >
  > <strong>Column 1 (data cell):</strong>
  >
  > <u>num</u>
  > <strong>Column 2 (data cell):</strong>
  >
  > <u>`[+-]?([0-9]+|[0-9]*\.[0-9]+)(e[+-]?[0-9]+)?`</u>

- <a id="s.14.2"></a>

  \[2014-07-16\] The background of the canvas cannot be taken from an element that is suppressed with 'display: none'. On the other hand, if the element is merely invisible ('visibility: hidden'), its background can still be used for the canvas. In other words, if the root element has 'display: none', the background of the canvas is transparent. In the case of (X)HTML documents, if the root element has 'background: transparent' and the \<body\> element has 'display: none', the background of the canvas is likewise transparent. Change the text in [“14.2 The background”](css2--colors.html--5784063d2778.md#background) as follows:

  > \[…\] Such backgrounds must also be anchored at the same point as they would be if they were painted only for the root element.
  >
  > <u>However, if no boxes are generated for the element whose background would be used for the background of the canvas, then the canvas background is transparent. (In CSS 2, that is the case when the element or an ancestor has 'display: none'.)</u>
  >
  > <u>Note that, if the element has 'visibility: hidden' but not 'display: none', boxes <em>are</em> generated for it and its background <em>is</em> used for the canvas.</u>

- <a id="s.9.9.1"></a>

  \[2015-07-01\] An element with 'position: fixed' <em>always</em> establishes a new stacking context. (This is different from 'position: absolute', where 'z-index' determines if the element establishes a stacking context or not.)

  Change the definition of 'auto' in [“9.9.1 Specifying the stack level: the 'z-index' property”](css2--visuren.html--3f334c530cf4.md#z-index) as follows:

  > <strong>auto</strong>  
  > The stack level of the generated box in the current stacking context is 0. ~~The box does not establish a new stacking context unless it is the root element.~~ <u>If the box has 'position: fixed' or if it is the root, it also establishes a new stacking context.</u>

- <a id="s.4.1.1e"></a>

  \[2015-09-05\] Malformed declarations are handled differently when the start of the malformed declaration conforms to the syntax of an at-rule. In that case, parsing resumes not at the next semicolon or at the closing curly brace of the enclosing block, but immediately after that at-rule. This is expressed by adding the at-rule to the core syntax for rulesets, as shown below.

  In [“4.1.1 Tokenization”,](css2--syndata.html--02e71c159e14.md#tokenization) change the production for ruleset as follows:

  > ```text
  > 
  > ruleset     : selector? '{' S* declaration? [ ';' S* declaration? ]* '}' S*;
  > ruleset     : selector? '{' S* declaration-list '}' S*;
  > declaration-list: declaration [ ';' S* declaration-list ]?
  >               | at-rule declaration-list
  >               | /* empty */;
  > ```
  In [“4.1.7 Rule sets, declaration blocks, and selectors”,](css2--syndata.html--02e71c159e14.md#rule-sets) change the second paragraph as follows:

  > A declaration block starts with a left curly brace ({) and ends with the matching right curly brace (}). In between there must be a list of zero or more ~~semicolon-separated (;) declarations~~ <u>declarations and at-rules. Declarations must end with a semicolon (;) unless they are last in the list.</u>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > <u>Note: CSS level 2 has no at-rules that may appear inside rule sets, but such at-rules may be defined in future levels.</u>

  In [“4.2 Rules for handling parsing errors”,](css2--syndata.html--02e71c159e14.md#parsing-errors) change the rule for malformed declarations as follows:

  > - <strong>Malformed declarations.</strong> User agents must handle unexpected tokens encountered while parsing a declaration by reading until the end of the declaration, while observing the rules for matching pairs of (), \[\], {}, "", and '', and correctly handling escapes. For example, a malformed declaration may be missing a property name, colon (:), or property value.
  >
  >   <u>When the UA expects the start of a declaration or at-rule (i.e., an IDENT token or an ATKEYWORD token) but finds an unexpected token instead, that token is considered to be the first token of a malformed declaration. I.e., the rule for malformed declarations, rather than malformed statements is used to determine which tokens to ignore in that case.</u>
  >
  >   The following are all equivalent:
  >
  >   > <strong data-conversion-semantic="example">Example</strong>
  >   >
  >   > ```text
  >   > 
  >   > p { color:green }
  >   > p { @foo { bar: baz } color:green }  /* unknown at-rule */
  >   > p { color:green; color }  /* malformed declaration missing ':', value */
  >   > p { color:red;   color; color:green }  /* same with expected recovery */
  >   > p { color:green; color: } /* malformed declaration missing value */
  >   > p { color:red;   color:; color:green } /* same with expected recovery */
  >   > p { color:green; color{;color:maroon} } /* unexpected tokens { } */
  >   > 	 p { color:red;   color{;color:maroon}; color:green } /* same with recovery */
  >   > ```
  And, finally, in [“13.2 Page boxes: the @page rule&#x26;rdquo,](css2--page.html--984664616bba.md#page-box) remove the following text, which is now redundant:

  > ~~The rules for handling malformed declarations, malformed statements, and invalid at-rules inside @page are as defined in [section 4.2,](css2--syndata.html--02e71c159e14.md#parsing-errors) with the following addition: when the UA expects the start of a declaration or at-rule (i.e., an IDENT token or an ATKEYWORD token) but finds an unexpected token instead, that token is considered to be the first token of a malformed declaration. I.e., the rule for malformed declarations, rather than malformed statements is used to determine which tokens to ignore in that case.~~

- <a id="s.10.8.1"></a>

  \[2015-04-01\] In [“10.8.1 Leading and half-leading”](css2--visudet.html--12e8bc0e6b7c.md#leading), replace the last paragraph as follows:

  > ~~The baseline of an 'inline-block' is the baseline of its last line box in the normal flow, unless it has either no in-flow line boxes or if its 'overflow' property has a computed value other than 'visible', in which case the baseline is the bottom margin edge.~~
  >
  > <u>The baseline of an 'inline-block’ whose ‘overflow’ property has a computed value of ‘visible’ is the baseline of its last line box in the normal flow, unless it has no in-flow line boxes, in which case the baseline is the bottom margin edge. The baseline of an inline-block whose ‘overflow’ property has a computed value not equal to ‘visible’ is the higher of either its bottom margin edge or the baseline of its last line box in the normal flow, unless it has no in-flow line boxes, in which case its baseline is the baseline is the bottom margin edge.</u>

  This avoids an undesirable visual difference between 'overflow: visible' and 'overflow: auto', in particular in the case when there is no visible scrolling mechanism, beause the height is sufficient.

[CSS Working Group](https://www.w3.org/Style/CSS/members)  
Contact: [Bert Bos](https://www.w3.org/People/Bos/)  
[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2011–2012 [W3C<sup>®</sup>](https://www.w3.org/)
