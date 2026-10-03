Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Syntax and basic data types](https://www.w3.org/TR/2011/REC-CSS2-20110607/syndata.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Syntax and basic data types

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/syndata.html

Snapshot SHA-256: 02e71c159e14aae772a52b552b4fd8dbdcd4ea5716a0004bfff5cb8897ac8002

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 4 source tables are presented as readable Markdown tables or explicit labeled layouts: 2 complex-table layouts, 2 already-readable tables. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="q4.0"></a>

# 4 Syntax and basic data types

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<a id="syntax"></a>

## 4.1 Syntax

<a id="x0"></a>

This section describes a grammar (and forward-compatible parsing rules) common to any level of CSS (including CSS 2.1). Future updates of CSS will adhere to this core syntax, although they may add additional syntactic constraints.

These descriptions are normative. They are also complemented by the normative grammar rules presented in [Appendix G](css2--grammar.html--eeb984c469da.md).

In this specification, the expressions "immediately before" or "immediately after" mean with no intervening white space or comments.

<a id="tokenization"></a>

### 4.1.1 Tokenization

All levels of CSS — level 1, level 2, and any future levels — use the same core syntax. This allows UAs to parse (though not completely understand) style sheets written in levels of CSS that did not exist at the time the UAs were created. Designers can use this feature to create style sheets that work with older user agents, while also exercising the possibilities of the latest levels of CSS.

At the lexical level, CSS style sheets consist of a sequence of tokens. The list of tokens for CSS is as follows. The definitions use Lex-style regular expressions. Octal codes refer to ISO 10646 ([\[ISO10646\]](css2--refs.html--f208d881d0b7.md#ref-ISO10646)). As in Lex, in case of multiple matches, the longest match determines the token.

**Table 1**

Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.

| Token | Definition |
| --- | --- |
|  |  |
| IDENT | <code><var>{ident}</var></code> |
| ATKEYWORD | <code>@<var>{ident}</var></code> |
| STRING | <code><var>{string}</var></code> |
| BAD_STRING | <code><var>{badstring}</var></code> |
| BAD_URI | <code><var>{baduri}</var></code> |
| BAD_COMMENT | <code><var>{badcomment}</var></code> |
| HASH | <code>#<var>{name}</var></code> |
| NUMBER | <code><var>{num}</var></code> |
| PERCENTAGE | <code><var>{num}</var>%</code> |
| DIMENSION | <code><var>{num}{ident}</var></code> |
| URI | <code>url&#x5C;(<var>{w}{string}{w}</var>&#x5C;)<br>&#xA;&#x9;&#x9;\|url&#x5C;(<var>{w}</var>(&#x5B;!#$%&amp;&#x2A;-&#x5C;&#x5B;&#x5C;&#x5D;-&#x7E;&#x5D;\|<var>{nonascii}</var>\|<var>{escape}</var>)&#x2A;<var>{w}</var>&#x5C;)</code> |
| UNICODE-RANGE | `u\+[0-9a-f?]{1,6}(-[0-9a-f]{1,6})?` |
| CDO | `<!--` |
| CDC | `-->` |
| : | `:` |
| ; | `;` |
| { | `\{` |
| } | `\}` |
| ( | `\(` |
| ) | `\)` |
| \[ | `\[` |
| \] | `\]` |
| S | `[ \t\r\n\f]+` |
| COMMENT | `\/\*[^*]*\*+([^/*][^*]*\*+)*\/` |
| FUNCTION | <code><var>{ident}</var>&#x5C;(</code> |
| INCLUDES | `~=` |
| DASHMATCH | `\|=` |
| DELIM | <var>any other character not matched by&#xA;the above rules, and neither a single nor a double quote</var> |

The macros in curly braces ({}) above are defined as follows:

**Table 2**

Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.

| Macro | Definition |
| --- | --- |
|  |  |
| ident | <code>&#x5B;-&#x5D;?<var>{nmstart}</var><var>{nmchar}&#x2A;</var></code> |
| name | <code><var>{nmchar}+</var></code> |
| nmstart | <code>&#x5B;&#x5F;a-z&#x5D;\|<var>{nonascii}</var>\|<var>{escape}</var></code> |
| nonascii | `[^\0-\237]` |
| unicode | `\\[0-9a-f]{1,6}(\r\n\|[ \n\r\t\f])?` |
| escape | <code><var>{unicode}</var>\|&#x5C;&#x5C;&#x5B;^&#x5C;n&#x5C;r&#x5C;f0-9a-f&#x5D;</code> |
| nmchar | <code>&#x5B;&#x5F;a-z0-9-&#x5D;\|<var>{nonascii}</var>\|<var>{escape}</var></code> |
| num | `[0-9]+\|[0-9]*\.[0-9]+` |
| string | <code><var>{string1}</var>\|<var>{string2}</var></code> |
| string1 | <code>&#x5C;"(&#x5B;^&#x5C;n&#x5C;r&#x5C;f&#x5C;&#x5C;"&#x5D;\|&#x5C;&#x5C;{nl}\|<var>{escape}</var>)&#x2A;&#x5C;"</code> |
| string2 | <code>&#x5C;'(&#x5B;^&#x5C;n&#x5C;r&#x5C;f&#x5C;&#x5C;'&#x5D;\|&#x5C;&#x5C;{nl}\|<var>{escape}</var>)&#x2A;&#x5C;'</code> |
| badstring | <code><var>{badstring1}</var>\|<var>{badstring2}</var></code> |
| badstring1 | <code>&#x5C;"(&#x5B;^&#x5C;n&#x5C;r&#x5C;f&#x5C;&#x5C;"&#x5D;\|&#x5C;&#x5C;{nl}\|<var>{escape}</var>)&#x2A;&#x5C;&#x5C;?</code> |
| badstring2 | <code>&#x5C;'(&#x5B;^&#x5C;n&#x5C;r&#x5C;f&#x5C;&#x5C;'&#x5D;\|&#x5C;&#x5C;{nl}\|<var>{escape}</var>)&#x2A;&#x5C;&#x5C;?</code> |
| badcomment | <code><var>{badcomment1}</var>\|<var>{badcomment2}</var></code> |
| badcomment1 | `\/\*[^*]*\*+([^/*][^*]*\*+)*` |
| badcomment2 | `\/\*[^*]*(\*+[^/*][^*]*)*` |
| baduri | <code><var>{baduri1}</var>\|<var>{baduri2}</var>\|<var>{baduri3}</var></code> |
| baduri1 | <code>url&#x5C;(<var>{w}</var>(&#x5B;!#$%&amp;&#x2A;-&#x7E;&#x5D;\|<var>{nonascii}</var>\|<var>{escape}</var>)&#x2A;<var>{w}</var></code> |
| baduri2 | <code>url&#x5C;(<var>{w}</var><var>{string}</var><var>{w}</var></code> |
| baduri3 | <code>url&#x5C;(<var>{w}</var><var>{badstring}</var></code> |
| nl | `\n\|\r\n\|\r\|\f` |
| w | `[ \t\r\n\f]*` |

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> For example, the rule of the longest match means that "`red-->`" is tokenized as the IDENT "`red--`" followed by the DELIM "`>`", rather than as an IDENT followed by a CDC.

Below is the core syntax for CSS. The sections that follow describe how to use it. [Appendix G](css2--grammar.html--eeb984c469da.md) describes a more restrictive grammar that is closer to the CSS level 2 language. Parts of style sheets that can be parsed according to this grammar but not according to the grammar in Appendix G are among the parts that will be ignored according to the [rules for handling parsing errors](#parsing-errors).

```text

stylesheet  : [ CDO | CDC | S | statement ]*;
statement   : ruleset | at-rule;
at-rule     : ATKEYWORD S* any* [ block | ';' S* ];
block       : '{' S* [ any | block | ATKEYWORD S* | ';' S* ]* '}' S*;
ruleset     : selector? '{' S* declaration? [ ';' S* declaration? ]* '}' S*;
selector    : any+;
declaration : property S* ':' S* value;
property    : IDENT;
value       : [ any | block | ATKEYWORD S* ]+;
any         : [ IDENT | NUMBER | PERCENTAGE | DIMENSION | STRING
              | DELIM | URI | HASH | UNICODE-RANGE | INCLUDES
              | DASHMATCH | ':' | FUNCTION S* [any|unused]* ')'
              | '(' S* [any|unused]* ')' | '[' S* [any|unused]* ']'
              ] S*;
unused      : block | ATKEYWORD S* | ';' S* | CDO S* | CDC S*;
```
The "unused" production is not used in CSS and will not be used by any future extension. It is included here only to help with error handling. (See [4.2 "Rules for handling parsing errors."](#parsing-errors))

<a id="comment"></a>

COMMENT tokens do not occur in the grammar (to keep it readable), but any number of these tokens may appear anywhere outside other tokens. (Note, however, that a comment before or within the @charset rule disables the @charset.)

<a id="whitespace"></a>

The token S in the grammar above stands for white space. Only the characters "space" (U+0020), "tab" (U+0009), "line feed" (U+000A), "carriage return" (U+000D), and "form feed" (U+000C) can occur in white space. Other space-like characters, such as "em-space" (U+2003) and "ideographic space" (U+3000), are never part of white space.

The meaning of input that cannot be tokenized or parsed is undefined in CSS 2.1.

<a id="keywords"></a>

### 4.1.2 Keywords

Keywords have the form of [identifiers.](#value-def-identifier) Keywords must not be placed between quotes ("..." or '...'). Thus,

```text

red
```
is a keyword, but

```text

"red"
```
is not. (It is a [string](#strings).) Other illegal examples:

Illegal example(s):

```text

width: "auto";
border: "none";
background: "red";
```
<a id="vendor-keywords"></a>

#### 4.1.2.1 Vendor-specific extensions

In CSS, identifiers may begin with '`-`' (dash) or '`_`' (underscore). Keywords and [property names](#properties) beginning with `-`' or '`_`' are reserved for vendor-specific extensions. Such vendor-specific extensions should have one of the following formats:

```text

'-' + vendor identifier + '-' + meaningful name
'_' + vendor identifier + '-' + meaningful name
```
> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> For example, if XYZ organization added a property to describe the color of the border on the East side of the display, they might call it -xyz-border-east-color.
>
> Other known examples:
>
> ```text
> 
> -moz-box-sizing
> -moz-border-radius
> -wap-accesskey
> ```
An initial dash or underscore is guaranteed never to be used in a property or keyword by any current or future level of CSS. Thus typical CSS implementations may not recognize such properties and may ignore them according to the [rules for handling parsing errors](#parsing-errors). However, because the initial dash or underscore is part of the grammar, CSS 2.1 implementers should always be able to use a CSS-conforming parser, whether or not they support any vendor-specific extensions.

Authors should avoid vendor-specific extensions

<a id="vendor-keyword-history"></a>

#### 4.1.2.2 Informative Historical Notes

This section is informative.

At the time of writing, the following prefixes are known to exist:

| prefix                               | organization                            |
|--------------------------------------|-----------------------------------------|
| `-ms-`, `mso-` | Microsoft                               |
| `-moz-`                    | Mozilla                                 |
| `-o-`, `-xv-` | Opera Software                          |
| `-atsc-`                    | Advanced Television Standards Committee |
| `-wap-`                    | The WAP Forum                           |
| `-khtml-`                    | KDE                                     |
| `-webkit-`                    | Apple                                   |
| `prince-`                    | YesLogic                                |
| `-ah-`                    | Antenna House                           |
| `-hp-`                    | Hewlett Packard                         |
| `-ro-`                    | Real Objects                            |
| `-rim-`                    | Research In Motion                      |
| `-tc-`                    | TallComponents                          |

<a id="characters"></a>

### 4.1.3 Characters and case

The following rules always hold:

- <a id="x1"></a>

  All CSS syntax is case-insensitive within the ASCII range (i.e., \[a-z\] and \[A-Z\] are equivalent), except for parts that are not under the control of CSS. For example, the case-sensitivity of values of the HTML attributes "id" and "class", of font names, and of URIs lies outside the scope of this specification. Note in particular that element names are case-insensitive in HTML, but case-sensitive in XML.

- <a id="value-def-identifier"></a>

  In CSS, identifiers (including element names, classes, and IDs in [selectors](css2--selector.html--f66f7c932788.md)) can contain only the characters \[a-zA-Z0-9\] and ISO 10646 characters U+00A0 and higher, plus the hyphen (-) and the underscore (\_); they cannot start with a digit, two hyphens, or a hyphen followed by a digit. Identifiers can also contain escaped characters and any ISO 10646 character as a numeric code (see next item). <strong data-conversion-semantic="example">Example:</strong> For instance, the identifier "B&#x26;W?" may be written as "B&#x5C;&#x26;W&#x5C;?" or "B&#x5C;26 W&#x5C;3F".

  Note that Unicode is code-by-code equivalent to ISO 10646 (see [\[UNICODE\]](css2--refs.html--f208d881d0b7.md#ref-UNICODE) and [\[ISO10646\]](css2--refs.html--f208d881d0b7.md#ref-ISO10646)).

- <a id="escaped-characters"></a>

  In CSS 2.1, a backslash (&#x5C;) character can indicate one of three types of character escape. Inside a CSS comment, a backslash stands for itself, and if a backslash is immediately followed by the end of the style sheet, it also stands for itself (i.e., a DELIM token).

  First, inside a [string](#strings), a backslash followed by a newline is ignored (i.e., the string is deemed not to contain either the backslash or the newline). Outside a string, a backslash followed by a newline stands for itself (i.e., a DELIM followed by a newline).

  Second, it cancels the meaning of special CSS characters. Any character (except a hexadecimal digit, linefeed, carriage return, or form feed) can be escaped with a backslash to remove its special meaning. For example, `"\""` is a string consisting of one double quote. Style sheet preprocessors must not remove these backslashes from a style sheet since that would change the style sheet's meaning.

  Third, backslash escapes allow authors to refer to characters they cannot easily put in a document. In this case, the backslash is followed by at most six hexadecimal digits (0..9A..F), which stand for the ISO 10646 ([\[ISO10646\]](css2--refs.html--f208d881d0b7.md#ref-ISO10646)) character with that number, which must not be zero. (It is undefined in CSS 2.1 what happens if a style sheet <em>does</em> contain a character with Unicode codepoint zero.) If a character in the range \[0-9a-fA-F\] follows the hexadecimal number, the end of the number needs to be made clear. There are two ways to do that:

  1.  with a space (or other white space character): "&#x5C;26 B" ("&#x26;B"). In this case, user agents should treat a "CR/LF" pair (U+000D/U+000A) as a single white space character.
  2.  by providing exactly 6 hexadecimal digits: "&#x5C;000026B" ("&#x26;B")

  In fact, these two methods may be combined. Only one white space character is ignored after a hexadecimal escape. Note that this means that a "real" space after the escape sequence must be doubled.

  If the number is outside the range allowed by Unicode (e.g., "&#x5C;110000" is above the maximum 10FFFF allowed in current Unicode), the UA may replace the escape with the "replacement character" (U+FFFD). If the character is to be displayed, the UA should show a visible symbol, such as a "missing character" glyph (cf. [15.2,](css2--fonts.html--d52fc14f36c2.md#algorithm) point 5).

- <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: Backslash escapes are always considered to be part of an [identifier](#value-def-identifier) or a string (i.e., "&#x5C;7B" is not punctuation, even though "{" is, and "&#x5C;32" is allowed at the start of a class name, even though "2" is not).
  > The identifier "te&#x5C;st" is exactly the same identifier as "test".

<a id="statements"></a>

### 4.1.4 Statements

<a id="x5"></a>

<a id="x6"></a>

<a id="x7"></a>

A CSS style sheet, for any level of CSS, consists of a list of <em>statements</em> (see the [grammar](#tokenization) above). There are two kinds of statements: <em>at-rules</em> and <em>rule&#xA;sets.</em> There may be [white space](#whitespace) around the statements.

<a id="at-rules"></a>

### 4.1.5  At-rules

At-rules start with an at-keyword, an '@' character followed immediately by an [identifier](#value-def-identifier) (for example, '@import', '@page').

An at-rule consists of everything up to and including the next semicolon (;) or the next [block,](#block) whichever comes first.

<a id="x9"></a>

<a id="x10"></a>

CSS 2.1 user agents must [ignore](#ignore) any ['@import'](css2--cascade.html--c7aff33e6f0d.md#at-import) rule that occurs inside a [block](#block) or after any non-ignored statement other than an @charset or an @import rule.

Illegal example(s):

Assume, for example, that a CSS 2.1 parser encounters this style sheet:

```text

@import "subs.css";
h1 { color: blue }
@import "list.css";
```
<a id="x11"></a>

The second '@import' is illegal according to CSS 2.1. The CSS 2.1 parser [ignores](#ignore) the whole at-rule, effectively reducing the style sheet to:

```text

@import "subs.css";
h1 { color: blue }
```
Illegal example(s):

In the following example, the second '@import' rule is invalid, since it occurs inside a '@media' [block](#block).

```text

@import "subs.css";
@media print {
  @import "print-main.css";
  body { font-size: 10pt }
}
h1 {color: blue }
```
Instead, to achieve the effect of only importing a style sheet for 'print' media, use the @import rule with media syntax, e.g.:

```text

@import "subs.css";
@import "print-main.css" print;
@media print {
  body { font-size: 10pt }
}
h1 {color: blue }
```
<a id="block"></a>

### 4.1.6 Blocks

<a id="x12"></a>

<a id="x13"></a>

A <em>block</em> starts with a left curly brace ({) and ends with the matching right curly brace (}). In between there may be any tokens, except that parentheses (( )), brackets (\[ \]), and braces ({ }) must always occur in matching pairs and may be nested. Single (') and double quotes (") must also occur in matching pairs, and characters between them are parsed as a string. See [Tokenization](#tokenization) above for the definition of a string.

Illegal example(s):

Here is an example of a block. Note that the right brace between the double quotes does not match the opening brace of the block, and that the second single quote is an [escaped character](#escaped-characters), and thus does not match the first single quote:

```text

{ causta: "}" + ({7} * '\'') }
```
Note that the above rule is not valid CSS 2.1, but it is still a block as defined above.

<a id="rule-sets"></a>

### 4.1.7 Rule sets, declaration blocks, and selectors

A rule set (also called "rule") consists of a selector followed by a declaration block.

<a id="x14"></a>

A declaration block starts with a left curly brace ({) and ends with the matching right curly brace (}). In between there must be a list of zero or more semicolon-separated (;) declarations.

<a id="x15"></a>

<a id="x16"></a>

The <em>selector</em> (see also the section on [selectors](css2--selector.html--f66f7c932788.md)) consists of everything up to (but not including) the first left curly brace ({). A selector always goes together with a declaration block. When a user agent cannot parse the selector (i.e., it is not valid CSS 2.1), it must [ignore](#ignore) the selector and the following declaration block (if any) as well.

<a id="x17"></a>

CSS 2.1 gives a special meaning to the comma (,) in selectors. However, since it is not known if the comma may acquire other meanings in future updates of CSS, the whole statement should be [ignored](#ignore) if there is an error anywhere in the selector, even though the rest of the selector may look reasonable in CSS 2.1.

Illegal example(s):

<a id="x18"></a>

For example, since the "&#x26;" is not a valid token in a CSS 2.1 selector, a CSS 2.1 user agent must [ignore](#ignore) the whole second line, and not set the color of H3 to red:

```text

h1, h2 {color: green }
h3, h4 & h5 {color: red }
h6 {color: black }
```
> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Here is a more complex example. The first two pairs of curly braces are inside a string, and do not mark the end of the selector. This is a valid CSS 2.1 rule.
>
> ```text
> 
> p[example="public class foo\
> {\
>     private int x;\
> \
>     foo(int x) {\
>         this.x = x;\
>     }\
> \
> }"] { color: red }
> ```
<a id="declaration"></a>

<a id="properties"></a>

### 4.1.8 Declarations and properties

<a id="x19"></a>

<a id="x20"></a>

A declaration is either empty or consists of a property name, followed by a colon (:), followed by a property value. Around each of these there may be [white space](#whitespace).

Because of the way selectors work, multiple declarations for the same selector may be organized into semicolon (;) separated groups.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Thus, the following rules:
>
> ```text
> 
> h1 { font-weight: bold }
> h1 { font-size: 12px }
> h1 { line-height: 14px }
> h1 { font-family: Helvetica }
> h1 { font-variant: normal }
> h1 { font-style: normal }
> ```
>
> are equivalent to:
>
> ```text
> 
> h1 {
>   font-weight: bold;
>   font-size: 12px;
>   line-height: 14px;
>   font-family: Helvetica;
>   font-variant: normal;
>   font-style: normal
> }
> ```
A property name is an [identifier](#value-def-identifier). Any token may occur in the property value. Parentheses ("( )"), brackets ("\[ \]"), braces ("{ }"), single quotes ('), and double quotes (") must come in matching pairs, and semicolons not in strings must be [escaped](#escaped-characters). Parentheses, brackets, and braces may be nested. Inside the quotes, characters are parsed as a string.

<a id="x21"></a>

The syntax of values is specified separately for each property, but in any case, values are built from identifiers, strings, numbers, lengths, percentages, URIs, colors, etc.

<a id="x22"></a>

A user agent must [ignore](#ignore) a declaration with an invalid property name or an invalid value. Every CSS property has its own syntactic and semantic restrictions on the values it accepts.

Illegal example(s):

For example, assume a CSS 2.1 parser encounters this style sheet:

```text

h1 { color: red; font-style: 12pt }  /* Invalid value: 12pt */
p { color: blue;  font-vendor: any;  /* Invalid prop.: font-vendor */
    font-variant: small-caps }
em em { font-style: normal }
```
<a id="x23"></a>

The second declaration on the first line has an invalid value '12pt'. The second declaration on the second line contains an undefined property 'font-vendor'. The CSS 2.1 parser will [ignore](#ignore) these declarations, effectively reducing the style sheet to:

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> 
> h1 { color: red; }
> p { color: blue;  font-variant: small-caps }
> em em { font-style: normal }
> ```
<a id="comments"></a>

### 4.1.9 Comments

<a id="x24"></a>

Comments begin with the characters "/\*" and end with the characters "\*/". They may occur anywhere outside other tokens, and their contents have no influence on the rendering. Comments may not be nested.

CSS also allows the SGML comment delimiters ("\<!--" and "--\>") in certain places defined by the grammar, but they do not delimit CSS comments. They are permitted so that style rules appearing in an HTML source document (in the STYLE element) may be hidden from pre-HTML 3.2 user agents. See the HTML 4 specification ([\[HTML4\]](css2--refs.html--f208d881d0b7.md#ref-HTML4)) for more information.

<a id="parsing-errors"></a>

## 4.2 Rules for handling parsing errors

<a id="ignore"></a>

In some cases, user agents must ignore part of an illegal style sheet. This specification defines ignore to mean that the user agent parses the illegal part (in order to find its beginning and end), but otherwise acts as if it had not been there. CSS 2.1 reserves for future updates of CSS all property:value combinations and @-keywords that do not contain an identifier beginning with dash or underscore. Implementations must ignore such combinations (other than those introduced by future updates of CSS).

To ensure that new properties and new values for existing properties can be added in the future, user agents are required to obey the following rules when they encounter the following scenarios:

- <a id="x26"></a>

  <strong>Unknown properties.</strong> User agents must [ignore](#ignore) a [declaration](css2--syndata.html--02e71c159e14.md#declaration) with an unknown property. For example, if the style sheet is:

  ```text
  
  h1 { color: red; rotation: 70minutes }
  ```
  the user agent will treat this as if the style sheet had been

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > ```text
  > 
  > h1 { color: red }
  > ```
- <a id="x27"></a>

  <a id="illegalvalues"></a><strong>Illegal values.</strong> User agents must ignore a declaration with an illegal value. For example:

  ```text
  
  img { float: left }       /* correct CSS 2.1 */
  img { float: left here }  /* "here" is not a value of 'float' */
  img { background: "red" } /* keywords cannot be quoted */
  img { border-width: 3 }   /* a unit must be specified for length values */
  ```
  A CSS 2.1 parser would honor the first rule and [ignore](#ignore) the rest, as if the style sheet had been:

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > ```text
  > 
  > img { float: left }
  > img { }
  > img { }
  > img { }
  > ```
  A user agent conforming to a future CSS specification may accept one or more of the other rules as well.

- <strong>Malformed declarations.</strong> User agents must handle unexpected tokens encountered while parsing a declaration by reading until the end of the declaration, while observing the rules for matching pairs of (), \[\], {}, "", and '', and correctly handling escapes. For example, a malformed declaration may be missing a property name, colon (:), or property value. The following are all equivalent:
  > <strong data-conversion-semantic="example">Example</strong>
  >
  > ```text
  > 
  > p { color:green }
  > p { color:green; color }  /* malformed declaration missing ':', value */
  > p { color:red;   color; color:green }  /* same with expected recovery */
  > p { color:green; color: } /* malformed declaration missing value */
  > p { color:red;   color:; color:green } /* same with expected recovery */
  > p { color:green; color{;color:maroon} } /* unexpected tokens { } */
  > p { color:red;   color{;color:maroon}; color:green } /* same with recovery */
  > ```
- <strong>Malformed statements.</strong> User agents must handle unexpected tokens encountered while parsing a statement by reading until the end of the statement, while observing the rules for matching pairs of (), \[\], {}, "", and '', and correctly handling escapes. For example, a malformed statement may contain an unexpected closing brace or at-keyword. E.g., the following lines are all ignored:

  ```text
  
  p @here {color: red}     /* ruleset with unexpected at-keyword "@here" */
  @foo @bar;               /* at-rule with unexpected at-keyword "@bar" */
  }} {{ - }}               /* ruleset with unexpected right brace */
  ) ( {} ) p {color: red } /* ruleset with unexpected right parenthesis */
  ```
- <a id="x28"></a>

  <strong>At-rules with unknown at-keywords.</strong> User agents must [ignore](#ignore) an invalid at-keyword together with everything following it, up to the end of the block that contains the invalid at-keyword, or up to and including the next semicolon (;), or up to and including the next block ({...}), whichever comes first. For example, consider the following:

  ```text
  
  @three-dee {
    @background-lighting {
      azimuth: 30deg;
      elevation: 190deg;
    }
    h1 { color: red }
  }
  h1 { color: blue }
  ```
  <a id="x29"></a>

  <a id="x30"></a>

  The '@three-dee' at-rule is not part of CSS 2.1. Therefore, the whole at-rule (up to, and including, the third right curly brace) is [ignored.](#ignore) A CSS 2.1 user agent [ignores](#ignore) it, effectively reducing the style sheet to:

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > ```text
  > 
  > h1 { color: blue }
  > ```
  Something inside an at-rule that is ignored because it is invalid, such as an invalid declaration within an @media-rule, does not make the entire at-rule invalid.

- <a id="unexpected-eof"></a><strong>Unexpected end of style sheet.</strong>

  User agents must close all open constructs (for example: blocks, parentheses, brackets, rules, strings, and comments) at the end of the style sheet. For example:

  ```text
  
    @media screen {
      p:before { content: 'Hello
  ```
  would be treated the same as:

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > ```text
  > 
  >   @media screen {
  >     p:before { content: 'Hello'; }
  >   }
  > ```
  in a conformant UA.

- <strong>Unexpected end of string.</strong>

  User agents must close strings upon reaching the end of a line (i.e., before an unescaped line feed, carriage return or form feed character), but then drop the construct (declaration or rule) in which the string was found. For example:

  ```text
  
        p {
          color: green;
          font-family: 'Courier New Times
          color: red;
          color: green;
        }
  ```
  ...would be treated the same as:

  ```text
  
        p { color: green; color: green; }
  ```
  ...because the second declaration (from 'font-family' to the semicolon after 'color: red') is invalid and is dropped.

- See also [Rule sets, declaration blocks, and selectors](#rule-sets) for parsing rules for declaration blocks.

<a id="values"></a>

## 4.3 Values

<a id="numbers"></a>

### 4.3.1 Integers and real numbers

<a id="value-def-integer"></a>

<a id="value-def-number"></a>

Some value types may have integer values (denoted by \<integer\>) or real number values (denoted by \<number\>). Real numbers and integers are specified in decimal notation only. An \<integer\> consists of one or more digits "0" to "9". A \<number\> can either be an \<integer\>, or it can be zero or more digits followed by a dot (.) followed by one or more digits. Both integers and real numbers may be preceded by a "-" or "+" to indicate the sign. -0 is equivalent to 0 and is not a negative number.

Note that many properties that allow an integer or real number as a value actually restrict the value to some range, often to a non-negative value.

<a id="length-units"></a>

### 4.3.2 Lengths

Lengths refer to distance measurements.

<a id="value-def-length"></a>

The format of a length value (denoted by \<length\> in this specification) is a [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) (with or without a decimal point) immediately followed by a unit identifier (e.g., px, em, etc.). After a zero length, the unit identifier is optional.

Some properties allow negative length values, but this may complicate the formatting model and there may be implementation-specific limits. If a negative length value cannot be supported, it should be converted to the nearest value that can be supported.

If a negative length value is set on a property that does not allow negative length values, the declaration is ignored.

In cases where the [used](css2--cascade.html--c7aff33e6f0d.md#usedValue) length cannot be supported, user agents must approximate it in the [actual value.](css2--cascade.html--c7aff33e6f0d.md#actual-value)

<a id="absrel-units"></a>

<a id="x34"></a>

There are two types of length units: relative and absolute. <em>Relative length</em> units specify a length relative to another length property. Style sheets that use relative units can more easily scale from one output environment to another.

Relative units are:

- <strong>em</strong>: the ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size) of the relevant font
- <strong>ex</strong>: the 'x-height' of the relevant font

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1 { margin: 0.5em }      /* em */
> h1 { margin: 1ex }        /* ex */
> ```
<a id="em-width"></a>

The 'em' unit is equal to the computed value of the ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size) property of the element on which it is used. The exception is when 'em' occurs in the value of the 'font-size' property itself, in which case it refers to the font size of the parent element. It may be used for vertical or horizontal measurement. (This unit is also sometimes called the quad-width in typographic texts.)

<a id="ex"></a>

The 'ex' unit is defined by the element's first available font. The exception is when 'ex' occurs in the value of the ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size) property, in which case it refers to the 'ex' of the parent element.

The 'x-height' is so called because it is often equal to the height of the lowercase "x". However, an 'ex' is defined even for fonts that do not contain an "x".

The x-height of a font can be found in different ways. Some fonts contain reliable metrics for the x-height. If reliable font metrics are not available, UAs may determine the x-height from the height of a lowercase glyph. One possible heuristic is to look at how far the glyph for the lowercase "o" extends below the baseline, and subtract that value from the top of its bounding box. In the cases where it is impossible or impractical to determine the x-height, a value of 0.5em should be used.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The rule:
>
> ```text
> 
> h1 { line-height: 1.2em }
> ```
>
> means that the line height of "h1" elements will be 20% greater than the font size of the "h1" elements. On the other hand:
>
> ```text
> 
> h1 { font-size: 1.2em }
> ```
>
> means that the font-size of "h1" elements will be 20% greater than the font size inherited by "h1" elements.

When specified for the root of the [document tree](css2--conform.html--de58593b67d7.md#doctree) (e.g., "HTML" in HTML), 'em' and 'ex' refer to the property's [initial value](css2--about.html--c67ff594c990.md#initial-value).

Child elements do not inherit the relative values specified for their parent; they inherit the [computed values](css2--cascade.html--c7aff33e6f0d.md#computed-value).

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In the following rules, the computed ['text-indent'](css2--text.html--467a8857ae69.md#propdef-text-indent) value of "h1" elements will be 36px, not 45px, if "h1" is a child of the "body" element.
>
> ```text
> 
> body {
>   font-size: 12px;
>   text-indent: 3em;  /* i.e., 36px */
> }
> h1 { font-size: 15px }
> ```
<a id="x39"></a>

<em> Absolute&#xA;&#xA;length</em> units are fixed in relation to each other. They are mainly useful when the output environment is known. The absolute units consist of the physical units (in, cm, mm, pt, pc) and the px unit:

- <strong>in</strong>: inches — 1in is equal to 2.54cm.
- <strong>cm</strong>: centimeters
- <strong>mm</strong>: millimeters
- <strong>pt</strong>: points — the points used by CSS are equal to 1/72nd of 1in.
- <strong>pc</strong>: picas — 1pc is equal to 12pt.
- <strong>px</strong>: pixel units — 1px is equal to 0.75pt.

For a CSS device, these dimensions are either anchored (i) by relating the physical units to their physical measurements, or (ii) by relating the pixel unit to the <i>reference pixel</i>. For print media and similar high-resolution devices, the anchor unit should be one of the standard physical units (inches, centimeters, etc). For lower-resolution devices, and devices with unusual viewing distances, it is recommended instead that the anchor unit be the pixel unit. For such devices it is recommended that the pixel unit refer to the whole number of device pixels that best approximates the reference pixel.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that if the anchor unit is the pixel unit, the physical units might not match their physical measurements. Alternatively if the anchor unit is a physical unit, the pixel unit might not map to a whole number of device pixels.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that this definition of the pixel unit and the physical units differs from previous versions of CSS. In particular, in previous versions of CSS the pixel unit and the physical units were not related by a fixed ratio: the physical units were always tied to their physical measurements while the pixel unit would vary to most closely match the reference pixel. (This change was made because too much existing content relies on the assumption of 96dpi, and breaking that assumption breaks the content.)

<a id="x40"></a>

The <em>reference pixel</em> is the visual angle of one pixel on a device with a pixel density of 96dpi and a distance from the reader of an arm's length. For a nominal arm's length of 28 inches, the visual angle is therefore about 0.0213 degrees. For reading at arm's length, 1px thus corresponds to about 0.26 mm (1/96 inch).

The image below illustrates the effect of viewing distance on the size of a reference pixel: a reading distance of 71 cm (28 inches) results in a reference pixel of 0.26 mm, while a reading distance of 3.5 m (12 feet) results in a reference pixel of 1.3 mm.

<a id="img-pixel1"></a>

![Showing that pixels must become larger if the viewing distance increases](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/pixel1.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/pixel1-desc.html)

This second image illustrates the effect of a device's resolution on the pixel unit: an area of 1px by 1px is covered by a single dot in a low-resolution device (e.g. a typical computer display), while the same area is covered by 16 dots in a higher resolution device (such as a printer).

<a id="img-pixel2"></a>

![Showing that more device pixels (dots) are needed to cover a 1px by 1px area on a high-resolution device than on a low-res one](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/pixel2.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/pixel2-desc.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1 { margin: 0.5in }      /* inches  */
> h2 { line-height: 3cm }   /* centimeters */
> h3 { word-spacing: 4mm }  /* millimeters */
> h4 { font-size: 12pt }    /* points */
> h4 { font-size: 1pc }     /* picas */
> p  { font-size: 12px }    /* px */
> ```
<a id="percentage-units"></a>

### 4.3.3 Percentages

<a id="value-def-percentage"></a>

<a id="x43"></a>

The format of a percentage value (denoted by \<percentage\> in this specification) is a [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) immediately followed by '%'.

Percentage values are always relative to another value, for example a length. Each property that allows percentages also defines the value to which the percentage refers. The value may be that of another property for the same element, a property for an ancestor element, or a value of the formatting context (e.g., the width of a [containing block](css2--visuren.html--3f334c530cf4.md#containing-block)). When a percentage value is set for a property of the [root](css2--conform.html--de58593b67d7.md#root) element and the percentage is defined as referring to the inherited value of some property, the resultant value is the percentage times the [initial value](css2--about.html--c67ff594c990.md#initial-value) of that property.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Since child elements (generally) inherit the [computed values](css2--cascade.html--c7aff33e6f0d.md#computed-value) of their parent, in the following example, the children of the P element will inherit a value of 12px for ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height), not the percentage value (120%):
>
> ```text
> 
> p { font-size: 10px }
> p { line-height: 120% }  /* 120% of 'font-size' */
> ```
<a id="uri"></a>

### 4.3.4 URLs and URIs

<a id="value-def-uri"></a>

URI values (Uniform Resource Identifiers, see [\[RFC3986\]](css2--refs.html--f208d881d0b7.md#ref-RFC3986), which includes URLs, URNs, etc) in this specification are denoted by \<uri\>. The functional notation used to designate URIs in property values is "url()", as in:

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> body { background: url("http://www.example.com/pinkish.png") }
> ```
The format of a URI value is 'url(' followed by optional [white space](#whitespace) followed by an optional single quote (') or double quote (") character followed by the URI itself, followed by an optional single quote (') or double quote (") character followed by optional white space followed by ')'. The two quote characters must be the same.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> An example without quotes:
>
> ```text
> 
> li { list-style: url(http://www.example.com/redball.png) disc }
> ```
Some characters appearing in an unquoted URI, such as parentheses, white space characters, single quotes (') and double quotes ("), must be escaped with a backslash so that the resulting URI value is a URI token: '&#x5C;(', '&#x5C;)'.

Depending on the type of URI, it might also be possible to write the above characters as URI-escapes (where "(" = %28, ")" = %29, etc.) as described in [\[RFC3986\]](css2--refs.html--f208d881d0b7.md#ref-RFC3986).

> <strong data-conversion-semantic="note">Note</strong>
>
> <em>Note that COMMENT tokens cannot occur within other tokens:&#xA;thus, "url(/&#x2A;x&#x2A;/pic.png)" denotes the URI "/&#x2A;x&#x2A;/pic.png", not&#xA;"pic.png".</em>

In order to create modular style sheets that are not dependent on the absolute location of a resource, authors may use relative URIs. Relative URIs (as defined in [\[RFC3986\]](css2--refs.html--f208d881d0b7.md#ref-RFC3986)) are resolved to full URIs using a base URI. RFC 3986, section 5, defines the normative algorithm for this process. For CSS style sheets, the base URI is that of the style sheet, not that of the source document.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> For example, suppose the following rule:
>
> ```text
> 
> body { background: url("yellow") }
> ```
>
> is located in a style sheet designated by the URI:
>
> ```text
> http://www.example.org/style/basic.css
> ```
>
> The background of the source document's BODY will be tiled with whatever image is described by the resource designated by the URI
>
> ```text
> http://www.example.org/style/yellow
> ```
User agents may vary in how they handle invalid URIs or URIs that designate unavailable or inapplicable resources.

<a id="counter"></a>

### 4.3.5 Counters

<a id="value-def-counter"></a>

<a id="x46"></a>

Counters are denoted by case-sensitive identifiers (see the ['counter-increment'](css2--generate.html--37748b674cd2.md#propdef-counter-increment) and ['counter-reset'](css2--generate.html--37748b674cd2.md#propdef-counter-reset) properties). To refer to the value of a counter, the notation 'counter(\<identifier\>)' or 'counter(\<identifier\>, \<'list-style-type'\>)', with optional white space separating the tokens, is used. The default style is 'decimal'.

To refer to a sequence of nested counters of the same name, the notation is 'counters(\<identifier\>, \<string\>)' or 'counters(\<identifier\>, \<string\>, \<'list-style-type'\>)' with optional white space separating the tokens.

See ["Nested counters and scope"](css2--generate.html--37748b674cd2.md#scope) in the chapter on [generated content](css2--generate.html--37748b674cd2.md) for how user agents must determine the value or values of the counter. See the definition of counter values of the ['content'](css2--generate.html--37748b674cd2.md#propdef-content) property for how it must convert these values to a string.

In CSS 2.1, the values of counters can only be referred to from the ['content'](css2--generate.html--37748b674cd2.md#propdef-content) property. Note that 'none' is a possible \<'list-style-type'\>: 'counter(x, none)' yields an empty string.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Here is a style sheet that numbers paragraphs (p) for each chapter (h1). The paragraphs are numbered with roman numerals, followed by a period and a space:
>
> ```text
> 
> p {counter-increment: par-num}
> h1 {counter-reset: par-num}
> p:before {content: counter(par-num, upper-roman) ". "}
> ```
<a id="color-units"></a>

### 4.3.6 Colors

<a id="value-def-color"></a>

A \<color\> is either a keyword or a numerical RGB specification.

The list of color keywords is: aqua, black, blue, fuchsia, gray, green, lime, maroon, navy, olive, orange, purple, red, silver, teal, white, and yellow. These 17 colors have the following values:

<a id="TanteksColorDiagram20020613"></a>

maroon \#800000 red \#ff0000 orange \#ffA500 yellow \#ffff00 olive \#808000

purple \#800080 fuchsia \#ff00ff white \#ffffff lime \#00ff00 green \#008000

navy \#000080 blue \#0000ff aqua \#00ffff teal \#008080

black \#000000 silver \#c0c0c0 gray \#808080

In addition to these color keywords, users may specify keywords that correspond to the colors used by certain objects in the user's environment. Please consult the section on [system colors](css2--ui.html--ff1f72803ea0.md#system-colors) for more information.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> body {color: black; background: white }
> h1 { color: maroon }
> h2 { color: olive }
> ```
The RGB color model is used in numerical color specifications. These examples all specify the same color:

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> em { color: #f00 }              /* #rgb */
> em { color: #ff0000 }           /* #rrggbb */
> em { color: rgb(255,0,0) }      
> em { color: rgb(100%, 0%, 0%) } 
> ```
The format of an RGB value in hexadecimal notation is a '#' immediately followed by either three or six hexadecimal characters. The three-digit RGB notation (#rgb) is converted into six-digit form (#rrggbb) by replicating digits, not by adding zeros. For example, \#fb0 expands to \#ffbb00. This ensures that white (#ffffff) can be specified with the short notation (#fff) and removes any dependencies on the color depth of the display.

The format of an RGB value in the functional notation is 'rgb(' followed by a comma-separated list of three numerical values (either three integer values or three percentage values) followed by ')'. The integer value 255 corresponds to 100%, and to F or FF in the hexadecimal notation: rgb(255,255,255) = rgb(100%,100%,100%) = \#FFF. [White space](#whitespace) characters are allowed around the numerical values.

All RGB colors are specified in the sRGB color space (see [\[SRGB\]](css2--refs.html--f208d881d0b7.md#ref-SRGB)). User agents may vary in the fidelity with which they represent these colors, but using sRGB provides an unambiguous and objectively measurable definition of what the color should be, which can be related to international standards (see [\[COLORIMETRY\]](css2--refs.html--f208d881d0b7.md#ref-COLORIMETRY)).

[Conforming user agents](css2--conform.html--de58593b67d7.md#conformance) may limit their color-displaying efforts to performing a gamma-correction on them. sRGB specifies a display gamma of 2.2 under specified viewing conditions. User agents should adjust the colors given in CSS such that, in combination with an output device's "natural" display gamma, an effective display gamma of 2.2 is produced. Note that only colors specified in CSS are affected; e.g., images are expected to carry their own color information.

Values outside the device gamut should be clipped or mapped into the gamut when the gamut is known: the red, green, and blue values must be changed to fall within the range supported by the device. Users agents may perform higher quality mapping of colors from one gamut to another. For a typical CRT monitor, whose device gamut is the same as sRGB, the four rules below are equivalent:

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> em { color: rgb(255,0,0) }       /* integer range 0 - 255 */
> em { color: rgb(300,0,0) }       /* clipped to rgb(255,0,0) */
> em { color: rgb(255,-10,0) }     /* clipped to rgb(255,0,0) */
> em { color: rgb(110%, 0%, 0%) }  /* clipped to rgb(100%,0%,0%) */
> ```
Other devices, such as printers, have different gamuts than sRGB; some colors outside the 0..255 sRGB range will be representable (inside the device gamut), while other colors inside the 0..255 sRGB range will be outside the device gamut and will thus be mapped.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> Mapping or clipping of&#xA;color values should be done to the actual device gamut if known (which&#xA;may be larger or smaller than 0..255).</em>

<a id="strings"></a>

### 4.3.7 Strings

<a id="value-def-string"></a>

Strings can either be written with double quotes or with single quotes. Double quotes cannot occur inside double quotes, unless escaped (e.g., as '&#x5C;"' or as '&#x5C;22'). Analogously for single quotes (e.g., "&#x5C;'" or "&#x5C;27").

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> "this is a 'string'"
> "this is a \"string\""
> 'this is a "string"'
> 'this is a \'string\''
> ```
<a id="x49"></a>

A string cannot directly contain a newline. To include a newline in a string, use an escape representing the line feed character in ISO-10646 (U+000A), such as "&#x5C;A" or "&#x5C;00000a". This character represents the generic notion of "newline" in CSS. See the ['content'](css2--generate.html--37748b674cd2.md#propdef-content) property for an example.

It is possible to break strings over several lines, for aesthetic or other reasons, but in such a case the newline itself has to be escaped with a backslash (&#x5C;). For instance, the following two selectors are exactly the same:

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> a[title="a not s\
> o very long title"] {/*...*/}
> a[title="a not so very long title"] {/*...*/}
> ```
<a id="unsupported-values"></a>

### 4.3.8 Unsupported Values

If a UA does not support a particular value, it should <em>ignore</em> that value when parsing style sheets, as if that value was an [illegal value](#illegalvalues). For example:

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
>   h3 {
>     display: inline;
>     display: run-in;
>   }
> ```
A UA that supports the 'run-in' value for the 'display' property will accept the first display declaration and then "write over" that value with the second display declaration. A UA that does not support the 'run-in' value will process the first display declaration and ignore the second display declaration.

<a id="charset"></a>

## 4.4 CSS style sheet representation

<a id="x50"></a>

A CSS style sheet is a sequence of characters from the Universal Character Set (see [\[ISO10646\]](css2--refs.html--f208d881d0b7.md#ref-ISO10646)). For transmission and storage, these characters must be encoded by a character encoding that supports the set of characters available in US-ASCII (e.g., UTF-8, ISO 8859-x, SHIFT JIS, etc.). For a good introduction to character sets and character encodings, please consult the HTML 4 specification ([\[HTML4\]](css2--refs.html--f208d881d0b7.md#ref-HTML4), chapter 5). See also the XML 1.0 specification ([\[XML10\]](css2--refs.html--f208d881d0b7.md#ref-XML10), sections 2.2 and 4.3.3, and Appendix F).

When a style sheet is embedded in another document, such as in the STYLE element or "style" attribute of HTML, the style sheet shares the character encoding of the whole document.

<a id="x51"></a>

<a id="x52"></a>

When a style sheet resides in a separate file, user agents must observe the following priorities when determining a style sheet's character encoding (from highest priority to lowest):

1.  An HTTP "charset" parameter in a "Content-Type" field (or similar parameters in other protocols)

2.  <a id="x55"></a>

    <a id="x54"></a>

    BOM and/or @charset (see below)

3.  `<link charset="">` or other metadata from the linking mechanism (if any)

4.  charset of referring style sheet or document (if any)

5.  Assume UTF-8

<a id="x56"></a>

Authors using an @charset rule must place the rule at the very beginning of the style sheet, preceded by no characters. (If a byte order mark is appropriate for the encoding used, it may precede the @charset rule.)

<a id="x57"></a>

After "@charset", authors specify the name of a character encoding (in quotes). For example:

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> @charset "ISO-8859-1";
> ```
@charset must be written literally, i.e., the 10 characters '@charset "' (lowercase, no backslash escapes), followed by the encoding name, followed by '";'.

The name must be a charset name as described in the IANA registry. See [\[CHARSETS\]](css2--refs.html--f208d881d0b7.md#ref-CHARSETS) for a complete list of charsets. Authors should use the charset names marked as "preferred MIME name" in the IANA registry.

<a id="x58"></a>

User agents must support at least the UTF-8 encoding.

User agents must ignore any @charset rule not at the beginning of the style sheet. When user agents detect the character encoding using the BOM and/or the @charset rule, they should follow the following rules:

- Except as specified in these rules, all @charset rules are ignored.
- The encoding is detected based on the stream of bytes that begins the style sheet. The following table gives a set of possibilities for initial byte sequences (written in hexadecimal). The first row that matches the beginning of the style sheet gives the result of encoding detection based on the BOM and/or @charset rule. If no rows match, the encoding cannot be detected based on the BOM and/or @charset rule. The notation (...)\* refers to repetition for which the best match is the one that repeats as few times as possible. The bytes marked "XX" are those used to determine the name of the encoding, by treating them, in the order given, as a sequence of ASCII characters. Bytes marked "YY" are similar, but need to be transcoded into ASCII as noted. User agents may ignore entries in the table if they do not support any encodings relevant to the entry.
  | Initial Bytes                                                                                                                                                               | Result                                                                                                                      |
  |-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------|
  | EF BB BF 40 63 68 61 72 73 65 74 20 22 (XX)\* 22 3B                                                                                                                         | as specified                                                                                                                |
  | EF BB BF                                                                                                                                                                    | UTF-8                                                                                                                       |
  | 40 63 68 61 72 73 65 74 20 22 (XX)\* 22 3B                                                                                                                                  | as specified                                                                                                                |
  | FE FF 00 40 00 63 00 68 00 61 00 72 00 73 00 65 00 74 00 20 00 22 (00 XX)\* 00 22 00 3B                                                                                     | as specified (with BE endianness if not specified)                                                                          |
  | 00 40 00 63 00 68 00 61 00 72 00 73 00 65 00 74 00 20 00 22 (00 XX)\* 00 22 00 3B                                                                                           | as specified (with BE endianness if not specified)                                                                          |
  | FF FE 40 00 63 00 68 00 61 00 72 00 73 00 65 00 74 00 20 00 22 00 (XX 00)\* 22 00 3B 00                                                                                     | as specified (with LE endianness if not specified)                                                                          |
  | 40 00 63 00 68 00 61 00 72 00 73 00 65 00 74 00 20 00 22 00 (XX 00)\* 22 00 3B 00                                                                                           | as specified (with LE endianness if not specified)                                                                          |
  | 00 00 FE FF 00 00 00 40 00 00 00 63 00 00 00 68 00 00 00 61 00 00 00 72 00 00 00 73 00 00 00 65 00 00 00 74 00 00 00 20 00 00 00 22 (00 00 00 XX)\* 00 00 00 22 00 00 00 3B | as specified (with BE endianness if not specified)                                                                          |
  | 00 00 00 40 00 00 00 63 00 00 00 68 00 00 00 61 00 00 00 72 00 00 00 73 00 00 00 65 00 00 00 74 00 00 00 20 00 00 00 22 (00 00 00 XX)\* 00 00 00 22 00 00 00 3B             | as specified (with BE endianness if not specified)                                                                          |
  | 00 00 FF FE 00 00 40 00 00 00 63 00 00 00 68 00 00 00 61 00 00 00 72 00 00 00 73 00 00 00 65 00 00 00 74 00 00 00 20 00 00 00 22 00 (00 00 XX 00)\* 00 00 22 00 00 00 3B 00 | as specified (with 2143 endianness if not specified)                                                                        |
  | 00 00 40 00 00 00 63 00 00 00 68 00 00 00 61 00 00 00 72 00 00 00 73 00 00 00 65 00 00 00 74 00 00 00 20 00 00 00 22 00 (00 00 XX 00)\* 00 00 22 00 00 00 3B 00             | as specified (with 2143 endianness if not specified)                                                                        |
  | FE FF 00 00 00 40 00 00 00 63 00 00 00 68 00 00 00 61 00 00 00 72 00 00 00 73 00 00 00 65 00 00 00 74 00 00 00 20 00 00 00 22 00 00 (00 XX 00 00)\* 00 22 00 00 00 3B 00 00 | as specified (with 3412 endianness if not specified)                                                                        |
  | 00 40 00 00 00 63 00 00 00 68 00 00 00 61 00 00 00 72 00 00 00 73 00 00 00 65 00 00 00 74 00 00 00 20 00 00 00 22 00 00 (00 XX 00 00)\* 00 22 00 00 00 3B 00 00             | as specified (with 3412 endianness if not specified)                                                                        |
  | FF FE 00 00 40 00 00 00 63 00 00 00 68 00 00 00 61 00 00 00 72 00 00 00 73 00 00 00 65 00 00 00 74 00 00 00 20 00 00 00 22 00 00 00 (XX 00 00 00)\* 22 00 00 00 3B 00 00 00 | as specified (with LE endianness if not specified)                                                                          |
  | 40 00 00 00 63 00 00 00 68 00 00 00 61 00 00 00 72 00 00 00 73 00 00 00 65 00 00 00 74 00 00 00 20 00 00 00 22 00 00 00 (XX 00 00 00)\* 22 00 00 00 3B 00 00 00             | as specified (with LE endianness if not specified)                                                                          |
  | 00 00 FE FF                                                                                                                                                                 | UTF-32-BE                                                                                                                   |
  | FF FE 00 00                                                                                                                                                                 | UTF-32-LE                                                                                                                   |
  | 00 00 FF FE                                                                                                                                                                 | UTF-32-2143                                                                                                                 |
  | FE FF 00 00                                                                                                                                                                 | UTF-32-3412                                                                                                                 |
  | FE FF                                                                                                                                                                       | UTF-16-BE                                                                                                                   |
  | FF FE                                                                                                                                                                       | UTF-16-LE                                                                                                                   |
  | 7C 83 88 81 99 A2 85 A3 40 7F (YY)\* 7F 5E                                                                                                                                  | as specified, transcoded from EBCDIC to ASCII                                                                               |
  | AE 83 88 81 99 A2 85 A3 40 FC (YY)\* FC 5E                                                                                                                                  | as specified, transcoded from IBM1026 to ASCII                                                                              |
  | 00 63 68 61 72 73 65 74 20 22 (YY)\* 22 3B                                                                                                                                  | as specified, transcoded from GSM 03.38 to ASCII                                                                            |
  | analogous patterns                                                                                                                                                          | User agents may support additional, analogous, patterns if they support encodings that are not handled by the patterns here |
- If the encoding is detected based on one of the entries in the table above marked "as specified", the user agent ignores the style sheet if it does not parse an appropriate @charset rule at the beginning of the stream of characters resulting from decoding in the chosen @charset. This ensures that:
  - @charset rules should only function if they are in the encoding of the style sheet,
  - byte order marks are ignored only in encodings that support a byte order mark, and
  - encoding names cannot contain newlines.

User agents must ignore style sheets in unknown encodings.

<a id="escaping"></a>

### 4.4.1 Referring to characters not represented in a character encoding

A style sheet may have to refer to characters that cannot be represented in the current character encoding. These characters must be written as [escaped](#escaped-characters) references to ISO 10646 characters. These escapes serve the same purpose as numeric character references in HTML or XML documents (see [\[HTML4\]](css2--refs.html--f208d881d0b7.md#ref-HTML4), chapters 5 and 25).

The character escape mechanism should be used when only a few characters must be represented this way. If most of a style sheet requires escaping, authors should encode it with a more appropriate encoding (e.g., if the style sheet contains a lot of Greek characters, authors might use "ISO-8859-7" or "UTF-8").

Intermediate processors using a different character encoding may translate these escaped sequences into byte sequences of that encoding. Intermediate processors must not, on the other hand, alter escape sequences that cancel the special meaning of an ASCII character.

[Conforming user agents](css2--conform.html--de58593b67d7.md#conformance) must correctly map to ISO-10646 all characters in any character encodings that they recognize (or they must behave as if they did).

For example, a style sheet transmitted as ISO-8859-1 (Latin-1) cannot contain Greek letters directly: "κουρος" (Greek: "kouros") has to be written as "&#x5C;3BA&#x5C;3BF&#x5C;3C5&#x5C;3C1&#x5C;3BF&#x5C;3C2".

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong>&#xA;In HTML 4,&#xA;numeric character references are interpreted in "style" attribute&#xA;values but not in the content of the STYLE element. Because of this&#xA;asymmetry, we recommend that authors use the CSS character&#xA;escape mechanism rather than numeric character references&#xA;for both the "style" attribute and the STYLE element.&#xA;For example, we recommend:</em>
>
> ```text
> 
> <SPAN style="font-family: L\FC beck">...</SPAN>
> ```
>
> <em>rather than:</em>
>
> ```text
> 
> <SPAN style="font-family: L&#252;beck">...</SPAN>
> ```