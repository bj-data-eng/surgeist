Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Grammar of CSS 2.1](https://www.w3.org/TR/2011/REC-CSS2-20110607/grammar.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Grammar of CSS 2.1

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/grammar.html

Snapshot SHA-256: eeb984c469daab242d4a7ba9be10e13a8457d96dacf472800ebc8e76fe893e28

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

<a id="q25.0"></a>

# Appendix G. Grammar of CSS 2.1

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

This appendix is non-normative.

The grammar below defines the syntax of CSS 2.1. It is in some sense, however, a superset of CSS 2.1 as this specification imposes additional semantic constraints not expressed in this grammar. A conforming UA must also adhere to the [forward-compatible parsing rules](css2--syndata.html--02e71c159e14.md#syntax), the selectors notation, the [property and value notation](css2--about.html--c67ff594c990.md#property-defs), and the unit notation. However, not all syntactically correct CSS can take effect, since the document language may impose restrictions that are not in CSS, e.g., HTML imposes restrictions on the possible values of the "class" attribute.

<a id="grammar"></a>

## G.1 Grammar

<a id="x0"></a>

The grammar below is LALR(1) (but note that most UA's should not use it directly, since it does not express the [parsing conventions](css2--syndata.html--02e71c159e14.md#parsing-errors), only the CSS 2.1 syntax). The format of the productions is optimized for human consumption and some shorthand notation beyond Yacc (see [\[YACC\]](css2--refs.html--f208d881d0b7.md#ref-YACC)) is used:

- <strong>&#x2A;</strong>: 0 or more
- <strong>+</strong>: 1 or more
- <strong>?</strong>: 0 or 1
- <strong>|</strong>: separates alternatives
- <strong>&#x5B; &#x5D;</strong>: grouping

The productions are:

<a id="x1"></a>

<a id="x2"></a>

```text

stylesheet
  : [ CHARSET_SYM STRING ';' ]?
    [S|CDO|CDC]* [ import [ CDO S* | CDC S* ]* ]*
    [ [ ruleset | media | page ] [ CDO S* | CDC S* ]* ]*
  ;
import
  : IMPORT_SYM S*
    [STRING|URI] S* media_list? ';' S*
  ;
media
  : MEDIA_SYM S* media_list '{' S* ruleset* '}' S*
  ;
media_list
  : medium [ COMMA S* medium]*
  ;
medium
  : IDENT S*
  ;
page
  : PAGE_SYM S* pseudo_page?
    '{' S* declaration? [ ';' S* declaration? ]* '}' S*
  ;
pseudo_page
  : ':' IDENT S*
  ;
operator
  : '/' S* | ',' S*
  ;
combinator
  : '+' S*
  | '>' S*
  ;
unary_operator
  : '-' | '+'
  ;
property
  : IDENT S*
  ;
ruleset
  : selector [ ',' S* selector ]*
    '{' S* declaration? [ ';' S* declaration? ]* '}' S*
  ;
selector
  : simple_selector [ combinator selector | S+ [ combinator? selector ]? ]?
  ;
simple_selector
  : element_name [ HASH | class | attrib | pseudo ]*
  | [ HASH | class | attrib | pseudo ]+
  ;
class
  : '.' IDENT
  ;
element_name
  : IDENT | '*'
  ;
attrib
  : '[' S* IDENT S* [ [ '=' | INCLUDES | DASHMATCH ] S*
    [ IDENT | STRING ] S* ]? ']'
  ;
pseudo
  : ':' [ IDENT | FUNCTION S* [IDENT S*]? ')' ]
  ;
declaration
  : property ':' S* expr prio?
  ;
prio
  : IMPORTANT_SYM S*
  ;
expr
  : term [ operator? term ]*
  ;
term
  : unary_operator?
    [ NUMBER S* | PERCENTAGE S* | LENGTH S* | EMS S* | EXS S* | ANGLE S* |
      TIME S* | FREQ S* ]
  | STRING S* | IDENT S* | URI S* | hexcolor | function
  ;
function
  : FUNCTION S* expr ')' S*
  ;
/*
 * There is a constraint on the color that it must
 * have either 3 or 6 hex-digits (i.e., [0-9a-fA-F])
 * after the "#"; e.g., "#000" is OK, but "#abcd" is not.
 */
hexcolor
  : HASH S*
  ;
```
<a id="scanner"></a>

## G.2 Lexical scanner

<a id="x3"></a>

The following is the tokenizer, written in Flex (see [\[FLEX\]](css2--refs.html--f208d881d0b7.md#ref-FLEX)) notation. The tokenizer is case-insensitive.

<a id="x4"></a>

<a id="x5"></a>

The "&#x5C;377" represents the highest character number that current versions of Flex can deal with (decimal 255). It should be read as "&#x5C;4177777" (decimal 1114111), which is the highest possible code point in Unicode/ISO-10646.

```text

%option case-insensitive

h		[0-9a-f]
nonascii	[\240-\377]
unicode		\\{h}{1,6}(\r\n|[ \t\r\n\f])?
escape		{unicode}|\\[^\r\n\f0-9a-f]
nmstart		[_a-z]|{nonascii}|{escape}
nmchar		[_a-z0-9-]|{nonascii}|{escape}
string1		\"([^\n\r\f\\"]|\\{nl}|{escape})*\"
string2		\'([^\n\r\f\\']|\\{nl}|{escape})*\'
badstring1      \"([^\n\r\f\\"]|\\{nl}|{escape})*\\?
badstring2      \'([^\n\r\f\\']|\\{nl}|{escape})*\\?
badcomment1     \/\*[^*]*\*+([^/*][^*]*\*+)*
badcomment2     \/\*[^*]*(\*+[^/*][^*]*)*
baduri1         url\({w}([!#$%&*-\[\]-~]|{nonascii}|{escape})*{w}
baduri2         url\({w}{string}{w}
baduri3         url\({w}{badstring}
comment		\/\*[^*]*\*+([^/*][^*]*\*+)*\/
ident		-?{nmstart}{nmchar}*
name		{nmchar}+
num		[0-9]+|[0-9]*"."[0-9]+
string		{string1}|{string2}
badstring       {badstring1}|{badstring2}
badcomment      {badcomment1}|{badcomment2}
baduri          {baduri1}|{baduri2}|{baduri3}
url		([!#$%&*-~]|{nonascii}|{escape})*
s		[ \t\r\n\f]+
w		{s}?
nl		\n|\r\n|\r|\f

A		a|\\0{0,4}(41|61)(\r\n|[ \t\r\n\f])?
C		c|\\0{0,4}(43|63)(\r\n|[ \t\r\n\f])?
D		d|\\0{0,4}(44|64)(\r\n|[ \t\r\n\f])?
E		e|\\0{0,4}(45|65)(\r\n|[ \t\r\n\f])?
G		g|\\0{0,4}(47|67)(\r\n|[ \t\r\n\f])?|\\g
H		h|\\0{0,4}(48|68)(\r\n|[ \t\r\n\f])?|\\h
I		i|\\0{0,4}(49|69)(\r\n|[ \t\r\n\f])?|\\i
K		k|\\0{0,4}(4b|6b)(\r\n|[ \t\r\n\f])?|\\k
L               l|\\0{0,4}(4c|6c)(\r\n|[ \t\r\n\f])?|\\l
M		m|\\0{0,4}(4d|6d)(\r\n|[ \t\r\n\f])?|\\m
N		n|\\0{0,4}(4e|6e)(\r\n|[ \t\r\n\f])?|\\n
O		o|\\0{0,4}(4f|6f)(\r\n|[ \t\r\n\f])?|\\o
P		p|\\0{0,4}(50|70)(\r\n|[ \t\r\n\f])?|\\p
R		r|\\0{0,4}(52|72)(\r\n|[ \t\r\n\f])?|\\r
S		s|\\0{0,4}(53|73)(\r\n|[ \t\r\n\f])?|\\s
T		t|\\0{0,4}(54|74)(\r\n|[ \t\r\n\f])?|\\t
U               u|\\0{0,4}(55|75)(\r\n|[ \t\r\n\f])?|\\u
X		x|\\0{0,4}(58|78)(\r\n|[ \t\r\n\f])?|\\x
Z		z|\\0{0,4}(5a|7a)(\r\n|[ \t\r\n\f])?|\\z

%%

{s}			{return S;}

\/\*[^*]*\*+([^/*][^*]*\*+)*\/		/* ignore comments */
{badcomment}                         /* unclosed comment at EOF */

"<!--"		{return CDO;}
"-->"			{return CDC;}
"~="			{return INCLUDES;}
"|="			{return DASHMATCH;}

{string}		{return STRING;}
{badstring}             {return BAD_STRING;}

{ident}			{return IDENT;}

"#"{name}		{return HASH;}

@{I}{M}{P}{O}{R}{T}	{return IMPORT_SYM;}
@{P}{A}{G}{E}		{return PAGE_SYM;}
@{M}{E}{D}{I}{A}	{return MEDIA_SYM;}
"@charset "		{return CHARSET_SYM;}

"!"({w}|{comment})*{I}{M}{P}{O}{R}{T}{A}{N}{T}	{return IMPORTANT_SYM;}

{num}{E}{M}		{return EMS;}
{num}{E}{X}		{return EXS;}
{num}{P}{X}		{return LENGTH;}
{num}{C}{M}		{return LENGTH;}
{num}{M}{M}		{return LENGTH;}
{num}{I}{N}		{return LENGTH;}
{num}{P}{T}		{return LENGTH;}
{num}{P}{C}		{return LENGTH;}
{num}{D}{E}{G}		{return ANGLE;}
{num}{R}{A}{D}		{return ANGLE;}
{num}{G}{R}{A}{D}	{return ANGLE;}
{num}{M}{S}		{return TIME;}
{num}{S}		{return TIME;}
{num}{H}{Z}		{return FREQ;}
{num}{K}{H}{Z}		{return FREQ;}
{num}{ident}		{return DIMENSION;}

{num}%			{return PERCENTAGE;}
{num}			{return NUMBER;}

"url("{w}{string}{w}")" {return URI;}
"url("{w}{url}{w}")"    {return URI;}
{baduri}                {return BAD_URI;}

{ident}"("		{return FUNCTION;}

.			{return *yytext;}
```
<a id="tokenizer-diffs"></a>

## G.3 Comparison of tokenization in CSS 2.1 and CSS1

There are some differences in the syntax specified in the CSS1 recommendation ([\[CSS1\]](css2--refs.html--f208d881d0b7.md#ref-CSS1)), and the one above. Most of these are due to new tokens in CSS2 that did not exist in CSS1. Others are because the grammar has been rewritten to be more readable. However, there are some incompatible changes, that were felt to be errors in the CSS1 syntax. They are explained below.

- CSS1 style sheets could only be in 1-byte-per-character encodings, such as ASCII and ISO-8859-1. CSS 2.1 has no such limitation. In practice, there was little difficulty in extrapolating the CSS1 tokenizer, and some UAs have accepted 2-byte encodings.
- CSS1 only allowed four hex-digits after the backslash (&#x5C;) to refer to Unicode characters, CSS2 [allows six](css2--syndata.html--02e71c159e14.md#escaped-characters). Furthermore, CSS2 allows a white space character to delimit the escape sequence. E.g., according to CSS1, the string "&#x5C;abcdef" has 3 letters (&#x5C;abcd, e, and f), according to CSS2 it has only one (&#x5C;abcdef).
- The tab character (ASCII 9) was not allowed in strings. However, since strings in CSS1 were only used for font names and for URLs, the only way this can lead to incompatibility between CSS1 and CSS2 is if a style sheet contains a font family that has a tab in its name.
- Similarly, newlines ([escaped with a backslash](css2--syndata.html--02e71c159e14.md#strings)) were not allowed in strings in CSS1.
- CSS2 parses a number immediately followed by an identifier as a DIMENSION token (i.e., an unknown unit), CSS1 parsed it as a number and an identifier. That means that in CSS1, the declaration 'font: 10pt/1.2serif' was correct, as was 'font: 10pt/12pt serif'; in CSS2, a space is required before "serif". (Some UAs accepted the first example, but not the second.)
- In CSS1, a class name could start with a digit (".55ft"), unless it was a dimension (".55in"). In CSS2, such classes are parsed as unknown dimensions (to allow for future additions of new units). To make ".55ft" a valid class, CSS2 requires the first digit to be escaped (".&#x5C;35 5ft")

<a id="q25.4"></a>

## G.4 Implementation note

The lexical scanner for the CSS core syntax in [section 4.1.1](css2--syndata.html--02e71c159e14.md#tokenization) can be implemented as a scanner without back-up. In Lex notation, that requires the addition of the following patterns (which do not change the returned tokens, only the efficiency of the scanner):

```text

{ident}/\\          return IDENT;
#{name}/\\          return HASH;
@{ident}/\\         return ATKEYWORD;
#/\\                return DELIM;
@/\\                return DELIM;
@/-                 return DELIM;
@/-\\               return DELIM;
-/\\                return DELIM;
-/-                 return DELIM;
\</!                return DELIM;
\</!-               return DELIM;
{num}{ident}/\\     return DIMENSION;
{num}/\\            return NUMBER;
{num}/-             return NUMBER;
{num}/-\\           return NUMBER;
[0-9]+/\.           return NUMBER;
u/\+                return IDENT;
u\+[0-9a-f?]{1,6}/- return UNICODE_RANGE;
```