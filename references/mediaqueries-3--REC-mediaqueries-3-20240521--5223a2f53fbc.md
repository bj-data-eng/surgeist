Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Media Queries Level 3](https://www.w3.org/TR/2024/REC-mediaqueries-3-20240521/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Media Queries Level 3

Source snapshot: https://www.w3.org/TR/2024/REC-mediaqueries-3-20240521/

Snapshot SHA-256: 5223a2f53fbcd9b85ee78c81d7db9ea9508624eec403241944c3324d7fdf851e

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>Media Queries Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

HTML and CSS currently support media-dependent style sheets tailored for different <em>media types</em>. For example, a document may use sans-serif fonts when displayed on a screen and serif fonts when printed. ‘`screen`’ and ‘`print`’ are two media types that have been defined. <em>Media queries</em> extend the functionality of media types by allowing more precise labeling of style sheets.

A media query consists of a media type and zero or more expressions that check for the conditions of particular <em>media features</em>. Among the media features that can be used in media queries are ‘`width`’, ‘`height`’, and ‘`color`’. By using media queries, presentations can be tailored to a specific range of output devices without changing the content itself.

## <a id="status"></a>Status of this Document

<em>This section describes the status of this document at the time of its publication. A list of current W3C publications and the latest revision of this technical report can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index</a> at https&#58;//www&#46;w3&#46;org/TR/.</em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a Recommendation using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes). It includes [proposed corrections](https://www.w3.org/2023/Process-20231103/#proposed-corrections).

A W3C Recommendation is a specification that, after extensive consensus-building, is endorsed by W3C and its Members, and has commitments from Working Group members to [royalty-free licensing](https://www.w3.org/Consortium/Patent-Policy/#sec-Requirements) for implementations.

W3C recommends the wide deployment of this specification as a standard for the Web.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “mediaqueries-3” in the title, like this: “\[mediaqueries-3\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bmediaqueries-3%5D%20PUT%20SUBJECT%20HERE). Comments are due by 21 July 2024.

Future updates to this Recommendation may incorporate [new features](https://www.w3.org/2023/Process-20231103/#allow-new-features).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/2023/Process-20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/#sec-Disclosure).

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

[Proposed corrections](https://www.w3.org/2021/Process-20211102/#proposed-corrections) are marked in the document.

## <a id="background"></a>1. Background

(This section is not normative.)

HTML4 [\[HTML401\]](#HTML401) and CSS2 [\[CSS21\]](#CSS21) currently support media-dependent style sheets tailored for different media types. For example, a document may use different style sheets for screen and print. In HTML4, this can be written as:

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> <link rel="stylesheet" type="text/css" media="screen" href="sans-serif.css">
> <link rel="stylesheet" type="text/css" media="print" href="serif.css">
> ```
Inside a CSS style sheet, one can declare that sections apply to certain media types:

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> @media screen {
>   * { font-family: sans-serif }
> }
> ```
The ‘`print`’ and ‘`screen`’ media types are defined in HTML4. The complete list of media types in HTML4 is: ‘`aural`’, ‘`braille`’, ‘`handheld`’, ‘`print`’, ‘`projection`’, ‘`screen`’, ‘`tty`’, ‘`tv`’. CSS2 defines the same list, deprecates ‘`aural`’ and adds ‘`embossed`’ and ‘`speech`’. Also, ‘`all`’ is used to indicate that the style sheet applies to all media types.

Media-specific style sheets are supported by several user agents. The most commonly used feature is to distinguish between ‘`screen`’ and ‘`print`’.

There have been requests for ways to describe in more detail what type of output devices a style sheet applies to. Fortunately HTML4 foresaw these requests and defined a forward-compatible syntax for media types. Here is a quote from [HTML4, section 6.13](https://www.w3.org/TR/1999/REC-html401-19991224/types.html#h-6.13):

> Future versions of HTML may introduce new values and may allow parameterized values. To facilitate the introduction of these extensions, conforming user agents must be able to parse the [`media`](https://www.w3.org/TR/1999/REC-html401-19991224/present/styles.html#adef-media) attribute value as follows:
>
> 1.  The value is a comma-separated list of entries. For example,
>
>     ```text
>     media="screen, 3d-glasses, print and resolution > 90dpi"
>     ```
>
>     is mapped to:
>
>     ```text
>     "screen"
>     "3d-glasses"
>     "print and resolution > 90dpi"
>     ```
>
> 2.  Each entry is truncated just before the first character that isn't a US ASCII letter \[a-zA-Z\] (Unicode decimal 65-90, 97-122), digit \[0-9\] (Unicode hex 30-39), or hyphen (45). In the example, this gives:
>
>     ```text
>     "screen"
>     "3d-glasses"
>     "print"
>     ```
Media queries, as described in this specification, build on the mechanism outlined in HTML4. The syntax of media queries fit into the media type syntax reserved in HTML4. The `media` attribute of HTML4 also exists in XHTML and generic XML. The same syntax can also be used inside in the ‘`@media`’ and ‘`@import`’ rules of CSS.

However, the parsing rules for media queries are incompatible with those of HTML4 so that they are consistent with those of media queries used in CSS.

> <strong data-conversion-semantic="note">Note</strong>
>
> Newer versions of HTML [\[HTML\]](#biblio-html) reference the Media Queries specification directly and thus updates the rules for HTML.

## <a id="media0"></a>2. Media Queries

A media query consists of a media type and zero or more <a id="expressions"></a>expressions that check for the conditions of particular <a id="media-features"></a>media features.

Statements regarding media queries in this section assume the [syntax section](#syntax) is followed. Media queries that do not conform to the syntax are discussed in the [error handling section](#error-handling). I.e. the syntax takes precedence over requirements in this section.

> <strong data-conversion-semantic="example">Example</strong>
>
> Here is a simple example written in HTML:
>
> ```text
> <link rel="stylesheet" media="screen and (color)" href="example.css" />
> ```
>
> This example expresses that a certain style sheet (`example.css`) applies to devices of a certain media type (‘`screen`’) with certain feature (it must be a color screen).

> <strong data-conversion-semantic="example">Example</strong>
>
> Here the same media query written in an @import-rule in CSS:
>
> ```text
> @import url(color.css) screen and (color);
> ```
A media query is a logical expression that is either true or false. A media query is true if the media type of the media query matches the media type of the device where the user agent is running (as defined in the "Applies to" line), and all expressions in the media query are true.

A shorthand syntax is offered for media queries that apply to all media types; the keyword ‘`all`’ can be left out (along with the trailing ‘`and`’). I.e. if the media type is not explicitly given it is ‘`all`’.

> <strong data-conversion-semantic="example">Example</strong>
>
> I.e. these are identical:
>
> ```text
> @media all and (min-width:500px) { … }
> @media (min-width:500px) { … }
> ```
>
> As are these:
>
> ```text
> @media (orientation: portrait) { … }
> @media all and (orientation: portrait) { … }
> ```
Several media queries can be combined in a media query list. A comma-separated list of media queries. If one or more of the media queries in the comma-separated list are true, the whole list is true, and otherwise false. In the media queries syntax, the comma expresses a logical OR, while the ‘`and`’ keyword expresses a logical AND.

> <strong data-conversion-semantic="example">Example</strong>
>
> Here is an example of several media queries in a comma-separated list using the an @media-rule in CSS:
>
> ```text
> @media screen and (color), projection and (color) { … }
> ```
If the media query list is empty (i.e. the declaration is the empty string or consists solely of whitespace) it evaluates to true.

> <strong data-conversion-semantic="example">Example</strong>
>
> I.e. these are equivalent:
>
> ```text
> @media all { … }
> @media { … }
> ```
The logical NOT can be expressed through the ‘`not`’ keyword. The presence of the keyword ‘`not`’ at the beginning of the media query negates the result. I.e., if the media query had been true without the ‘`not`’ keyword it will become false, and vice versa. User agents that only support media types (as described in HTML4) will not recognize the ‘`not`’ keyword and the associated style sheet is therefore not applied.

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> <link rel="stylesheet" media="not screen and (color)" href="example.css" />
> ```
The keyword ‘`only`’ can also be used to hide style sheets from older user agents. User agents must process media queries starting with ‘`only`’ as if the ‘`only`’ keyword was not present.

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> <link rel="stylesheet" media="only screen and (color)" href="example.css" />
> ```
The media queries syntax can be used with HTML, XHTML, XML [\[XMLSTYLE\]](#XMLSTYLE) and the @import and @media rules of CSS.

> <strong data-conversion-semantic="example">Example</strong>
>
> Here is the same example written in HTML, XHTML, XML, @import and @media:
>
> ```text
> <link media="screen and (color), projection and (color)" rel="stylesheet" href="example.css">
> ```
>
> ```text
> <link media="screen and (color), projection and (color)" rel="stylesheet" href="example.css" />
> ```
>
> ```text
> <?xml-stylesheet media="screen and (color), projection and (color)" rel="stylesheet" href="example.css" ?>
> ```
>
> ```text
> @import url(example.css) screen and (color), projection and (color);
> ```
>
> ```text
> @media screen and (color), projection and (color) { … }
> ```
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > The [\[XMLSTYLE\]](#XMLSTYLE) specification has not yet been updated to use media queries in the `media` pseudo-attribute.

If a media feature does not apply to the device where the UA is running, expressions involving the media feature will be false.

> <strong data-conversion-semantic="example">Example</strong>
>
> The media feature ‘`device-aspect-ratio`’ only applies to visual devices. On an aural device, expressions involving ‘`device-aspect-ratio`’ will therefore always be false:
>
> ```text
> <link rel="stylesheet" media="aural and (device-aspect-ratio: 16/9)" href="example.css" />
> ```
Expressions will always be false if the unit of measurement does not apply to the device.

> <strong data-conversion-semantic="example">Example</strong>
>
> The ‘`px`’ unit does not apply to ‘`speech`’ devices so the following media query is always false:
>
> ```text
> <link rel="stylesheet" media="speech and (min-device-width: 800px)" href="example.css" />
> ```
>
> Note that the media queries in this example would have been true if the keyword ‘`not`’ had been added to the beginning of the media query.

To avoid circular dependencies, unless another feature explicitly specifies that it affects the resolution of Media Queries, it is not necessary to apply the style sheet in order to evaluate expressions. For example, the aspect ratio of a printed document may be influenced by a style sheet, but expressions involving ‘`device-aspect-ratio`’ will be based on the default aspect ratio of the user agent.

> <strong data-conversion-semantic="note">Note</strong>
>
> User agents are expected, but not required, to re-evaluate and re-layout the page in response to changes in the user environment, for example if the device is tilted from landscape to portrait mode.

## <a id="syntax"></a>3. Syntax

The media query syntax is described in terms of the [CSS2 grammar](https://www.w3.org/TR/CSS21/grammar.html). As such, rules not defined here are defined in CSS2. The `media_query_list` production defined below replaces the `media_list` production from CSS2. [\[CSS21\]](#CSS21)

```text
media_query_list
 : S* [media_query [ ',' S* media_query ]* ]?
 ;
media_query
 : [ONLY | NOT]? S* media_type S* [ AND S* expression ]*
 | expression [ AND S* expression ]*
 ;
media_type
 : IDENT
 ;
expression
 : '(' S* media_feature S* [ ':' S* expr ]? ')' S*
 ;
media_feature
 : IDENT
 ;
```
COMMENT tokens, as defined by CSS2, do not occur in the grammar (to keep it readable), but any number of these tokens may appear anywhere between other tokens. [\[CSS21\]](#CSS21)

The following new definitions are introduced:

```text
L  l|\\0{0,4}(4c|6c)(\r\n|[ \t\r\n\f])?|\\l
Y  y|\\0{0,4}(59|79)(\r\n|[ \t\r\n\f])?|\\y
```
The following new tokens are introduced:

```text
{O}{N}{L}{Y}      {return ONLY;}
{N}{O}{T}         {return NOT;}
{A}{N}{D}         {return AND;}
{num}{D}{P}{I}    {return RESOLUTION;}
{num}{D}{P}{C}{M} {return RESOLUTION;}
```
`RESOLUTION` is to be added to the CSS2 `term` production.

CSS style sheets are generally [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive), and this is also the case for media queries.

In addition to conforming to the syntax, each media query needs to use media types and media features according to their respective specification in order to be considered conforming.

> <strong data-conversion-semantic="example">Example</strong>
>
> Only the first media query is conforming in the example below because the "example" media type does not exist.
>
> ```text
> @media all { body { background:lime } }
> @media example { body { background:red } }
> ```
### <a id="error-handling"></a>3.1. Error Handling

For media queries that are not conforming user agents need to follow the rules described in this section.

<a id="c2"></a> Proposed Correction 2: Require that ‘`layer`’ is also to not be treated as an unknown media type, but as a syntax errors when used in place of media types.

This change was introduced as a result of [issue 7225](https://github.com/w3c/csswg-drafts/issues/7225).

This change has tests

[Tests for this change](https://github.com/web-platform-tests/wpt/pull/33940) have been added to WPT. The results can be viewed at [wpt.fyi](https://wpt.fyi/results/css/mediaqueries?q=mq-invalid-media-type-layer).

- <strong>Unknown media types.</strong> Unknown media types evaluate to false. Effectively, they are treated identically to known media types that do not match the media type of the device. However, an exception is made for media types <u>‘`layer`’,</u> ‘`not`’, ‘`and`’, ‘`only`’, and ‘`or`’. Even though they do match the IDENT production, they must not be treated as unknown media types, but rather trigger the malformed query clause.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: The exclusion of ‘`layer`’ is because it would otherwise be ambiguous when used in the `@import url(…) layer;` syntax for the sake of [cascade layers](https://www.w3.org/TR/css-cascade-5/#layering). See [\[CSS-CASCADE-5\]](#biblio-css-cascade-5).

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > The media query "`unknown`" will evaluate to false, unless `unknown` is actually a supported media type. Similarly, "`not unknown`" will evaluate to true.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > The following is a malformed media query because it uses ‘`only`’ and ‘`or`’ as media types.
  >
  > ```text
  > @media only and or { … }
  > ```
  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Unknown media types are distinct from media types that do not actually match the IDENT production. Those fall under the malformed media query clause.

- <strong>Unknown media features.</strong> User agents are to represent a media query as "`not all`" when one of the specified media features is not known.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > ```text
  > <link rel="stylesheet" media="screen and (max-weight: 3kg) and (color), (color)" href="example.css" />
  > ```
  >
  > In this example, the first media query will be represented as "`not all`" and evaluate to false and the second media query is evaluated as if the first had not been specified, effectively.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > ```text
  > @media (min-orientation:portrait) { … }
  > ```
  >
  > Is represented as "`not all`" because the ‘`orientation`’ feature does not accept the ‘`min-`’ prefix.

- <strong>Unknown media feature values.</strong> As with unknown media features, user agents are to represent a media query as "

  ```text
  not
       all
  ```
  " when one of the specified media feature values is not known.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > The media query `(color:20example)` specifies an unknown value for the ‘`color`’ media feature and is therefore represented as "`not all`".

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > This media query is represented as "`not all`" because negative lengths are not allowed for the ‘`width`’ media feature:
  >
  > ```text
  > @media (min-width: -100px) { … }
  > ```
- <strong>Malformed media query.</strong> User agents are to handle unexpected tokens encountered while parsing a media query by reading until the end of the media query, while observing [the rules for matching pairs](https://www.w3.org/TR/CSS21/syndata.html#block) of (), \[\], {}, "", and '', and correctly handling escapes. Media queries with unexpected tokens are represented as "`not all`". [\[CSS21\]](#CSS21)

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > ```text
  > @media (example, all,), speech { /* only applicable to speech devices */ }
  > @media &test, screen           { /* only applicable to screen devices */ }
  > ```
  > <strong data-conversion-semantic="example">Example</strong>
  >
  > The following is an malformed media query because having no space between ‘`and`’ and the expression is not allowed. (That is reserved for the functional notation syntax.)
  >
  > ```text
  > @media all and(color) { … }
  > ```
  Media queries are expected to follow the error handling rules of the host language as well.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > ```text
  > @media test;,all { body { background:lime } }
  > ```
  >
  > … will not apply because the semicolon terminates the `@media` rule in CSS.

## <a id="media1"></a>4. Media features

Syntactically, media features resemble CSS properties: they have names and accept certain values. There are, however, several important differences between properties and media features:

- Properties are used in <em>declarations</em> to give information about how to present a document. Media features are used in <em>expressions</em> to describe requirements of the output device.
- Most media features accept optional ‘`min-`’ or ‘`max-`’ prefixes to express "greater or equal to" and "smaller or equal to" constraints. This syntax is used to avoid "\<" and "\>" characters which may conflict with HTML and XML. Those media features that accept prefixes will most often be used with prefixes, but can also be used alone.
- Properties always require a value to form a declaration. Media features, on the other hand, can also be used without a value. For a media feature <var>feature</var>, <code>(<var>feature</var>)</code> will evaluate to true if <code>(<var>feature</var>:<var>x</var>)</code> will evaluate to true for a value <var>x</var> other than zero or zero followed by a unit identifier (i.e., other than `0`, `0px`, `0em`, etc.). Media features that are prefixed by min/max cannot be used without a value. When a media feature prefixed with min/max is used without a value it makes the media query malformed.
- Properties may accept more complex values, e.g., calculations that involve several other values. Media features only accept single values: one keyword, one number, or a number with a unit identifier. (The only exceptions are the ‘`aspect-ratio`’ and ‘`device-aspect-ratio`’ media features.)

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, the ‘`color`’ media feature can form expressions without a value (‘`(color)`’), or with a value (‘`(min-color: 1)`’).

> <strong data-conversion-semantic="note">Note</strong>
>
> This specification defines media features usable with visual and tactile devices. Similarly, media features can be defined for aural media types.

### <a id="width"></a>4.1. width

Value: \<length\>  
Applies to: visual and tactile media types  
Accepts min/max prefixes: yes  

The ‘`width`’ media feature describes the width of the targeted display area of the output device. For continuous media, this is the width of the viewport (as described by CSS2, section 9.1.1 [\[CSS21\]](#CSS21)) including the size of a rendered scroll bar (if any). For paged media, this is the width of the page box (as described by CSS2, section 13.2 [\[CSS21\]](#CSS21)).

A specified \<length\> cannot be negative.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, this media query expresses that the style sheet is usable on printed output wider than 25cm:
>
> ```text
> <link rel="stylesheet" media="print and (min-width: 25cm)" href="http://…" />
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> This media query expresses that the style sheet is usable on devices with viewport (the part of the screen/paper where the document is rendered) widths between 400 and 700 pixels:
>
> ```text
> @media screen and (min-width: 400px) and (max-width: 700px) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> This media query expresses that style sheet is usable on screen and handheld devices if the width of the viewport is greater than 20em.
>
> ```text
> @media handheld and (min-width: 20em),
>   screen and (min-width: 20em) { … }
> ```
>
> The ‘`em`’ value is relative to the initial value of ‘font-size’.

### <a id="height"></a>4.2. height

Value: \<length\>  
Applies to: visual and tactile media types  
Accepts min/max prefixes: yes  

The ‘`height`’ media feature describes the height of the targeted display area of the output device. For continuous media, this is the height of the viewport including the size of a rendered scroll bar (if any). For paged media, this is the height of the page box.

A specified \<length\> cannot be negative.

### <a id="device-width"></a>4.3. device-width

Value: \<length\>  
Applies to: visual and tactile media types  
Accepts min/max prefixes: yes  

The ‘`device-width`’ media feature describes the width of the rendering surface of the output device. For continuous media, this is the width of the screen. For paged media, this is the width of the page sheet size.

A specified \<length\> cannot be negative.

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> @media screen and (device-width: 800px) { … }
> ```
>
> In the example above, the style sheet will apply only to screens that currently displays exactly 800 horizontal pixels. The ‘`px`’ unit is of the logical kind, as described in the [Units](#units) section.

### <a id="device-height"></a>4.4. device-height

Value: \<length\>  
Applies to: visual and tactile media types  
Accepts min/max prefixes: yes  

The ‘`device-height`’ media feature describes the height of the rendering surface of the output device. For continuous media, this is the height of the screen. For paged media, this is the height of the page sheet size.

A specified \<length\> cannot be negative.

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> <link rel="stylesheet" media="screen and (device-height: 600px)" />
> ```
>
> In the example above, the style sheet will apply only to screens that have exactly 600 vertical pixels. Note that the definition of the ‘`px`’ unit is the same as in other parts of CSS.

### <a id="orientation"></a>4.5. orientation

Value: portrait \| landscape  
Applies to: bitmap media types  
Accepts min/max prefixes: no  

The ‘`orientation`’ media feature is ‘`portrait`’ when the value of the ‘`height`’ media feature is greater than or equal to the value of the ‘`width`’ media feature. Otherwise ‘`orientation`’ is ‘`landscape`’.

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> @media all and (orientation:portrait) { … }
> @media all and (orientation:landscape) { … }
> ```
### <a id="aspect-ratio"></a>4.6. aspect-ratio

Value: \<ratio\>  
Applies to: bitmap media types  
Accepts min/max prefixes: yes  

The ‘`aspect-ratio`’ media feature is defined as the ratio of the value of the ‘`width`’ media feature to the value of the ‘`height`’ media feature.

### <a id="device-aspect-ratio"></a>4.7. device-aspect-ratio

Value: \<ratio\>  
Applies to: bitmap media types  
Accepts min/max prefixes: yes  

The ‘`device-aspect-ratio`’ media feature is defined as the ratio of the value of the ‘`device-width`’ media feature to the value of the ‘`device-height`’ media feature.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, if a screen device with square pixels has 1280 horizontal pixels and 720 vertical pixels (commonly referred to as "16:9"), the following Media Queries will all match the device:
>
> ```text
> @media screen and (device-aspect-ratio: 16/9) { … }
> @media screen and (device-aspect-ratio: 32/18) { … }
> @media screen and (device-aspect-ratio: 1280/720) { … }
> @media screen and (device-aspect-ratio: 2560/1440) { … }
> ```
### <a id="color"></a>4.8. color

Value: \<integer\>  
Applies to: visual media types  
Accept min/max prefixes: yes  

The ‘`color`’ media feature describes the number of bits per color component of the output device. If the device is not a color device, the value is zero.

A specified \<integer\> cannot be negative.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, these two media queries express that a style sheet applies to all color devices:
>
> ```text
> @media all and (color) { … }
> @media all and (min-color: 1) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> This media query expresses that a style sheet applies to color devices with 2 or more bits per color component:
>
> ```text
> @media all and (min-color: 2) { … }
> ```
If different color components are represented by different number of bits, the smallest number is used.

> <strong data-conversion-semantic="example">Example</strong>
>
> For instance, if an 8-bit color system represents the red component with 3 bits, the green component with 3 bits and the blue component with 2 bits, the ‘`color`’ media feature will have a value of 2.

In a device with indexed colors, the minimum number of bits per color component in the lookup table is used.

> <strong data-conversion-semantic="note">Note</strong>
>
> The described functionality is only able to describe color capabilities at a superficial level. If further functionality is required, RFC2531 [\[RFC2531\]](#RFC2531) provides more specific media features which may be supported at a later stage.

### <a id="color-index"></a>4.9. color-index

Value: \<integer\>  
Applies to: visual media types  
Accepts min/max prefixes: yes  

The ‘`color-index`’ media feature describes the number of entries in the color lookup table of the output device. If the device does not use a color lookup table, the value is zero.

A specified \<integer\> cannot be negative.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, here are two ways to express that a style sheet applies to all color index devices:
>
> ```text
> @media all and (color-index) { … }
> @media all and (min-color-index: 1) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> This media query expresses that a style sheet applies to a color index device with 256 or more entries:
>
> ```text
> <?xml-stylesheet media="all and (min-color-index: 256)"
>   href="http://www.example.com/…" ?>
> ```
### <a id="monochrome"></a>4.10. monochrome

Value: \<integer\>  
Applies to: visual media types  
Accepts min/max prefixes: yes  

The ‘`monochrome`’ media feature describes the number of bits per pixel in a monochrome frame buffer. If the device is not a monochrome device, the output device value will be 0.

A specified \<integer\> cannot be negative.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, here are two ways to express that a style sheet applies to all monochrome devices:
>
> ```text
> @media all and (monochrome) { … }
> @media all and (min-monochrome: 1) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> Express that a style sheet applies to monochrome devices with more than 2 bits per pixels:
>
> ```text
> @media all and (min-monochrome: 2) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> Express that there is one style sheet for color pages and another for monochrome:
>
> ```text
> <link rel="stylesheet" media="print and (color)" href="http://…" />
> <link rel="stylesheet" media="print and (monochrome)" href="http://…" />
> ```
### <a id="resolution"></a>4.11. resolution

Value: \<resolution\>  
Applies to: bitmap media types  
Accepts min/max prefixes: yes  

The ‘`resolution`’ media feature describes the resolution of the output device, i.e. the density of the pixels. When querying devices with non-square pixels, in ‘`min-resolution`’ queries the least-dense dimension must be compared to the specified value and in ‘`max-resolution`’ queries the most-dense dimensions must be compared instead. A ‘`resolution`’ (without a "min-" or "max-" prefix) query never matches a device with non-square pixels.

For printers, this corresponds to the screening resolution (the resolution for printing dots of arbitrary color).

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, this media query expresses that a style sheet is usable on devices with resolution greater than 300 dots per inch:
>
> ```text
> @media print and (min-resolution: 300dpi) { … }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> This media query expresses that a style sheet is usable on devices with resolution greater than 118 dots per centimeter:
>
> ```text
> @media print and (min-resolution: 118dpcm) { … }
> ```
### <a id="scan"></a>4.12. scan

Value: progressive \| interlace  
Applies to: "tv" media types  
Accepts min/max prefixes: no  

The ‘`scan`’ media feature describes the scanning process of "tv" output devices.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, this media query expresses that a style sheet is usable on tv devices with progressive scanning:
>
> ```text
> @media tv and (scan: progressive) { … }
> ```
### <a id="grid"></a>4.13. grid

Value: \<integer\>  
Applies to: visual and tactile media types  
Accepts min/max prefixes: no  

The ‘`grid`’ media feature is used to query whether the output device is grid or bitmap. If the output device is grid-based (e.g., a "tty" terminal, or a phone display with only one fixed font), the value will be 1. Otherwise, the value will be 0.

Only 0 and 1 are valid values. (This includes -0.) Thus everything else creates a malformed media query.

> <strong data-conversion-semantic="example">Example</strong>
>
> Here are two examples:
>
> ```text
> @media handheld and (grid) and (max-width: 15em) { … }
> @media handheld and (grid) and (max-device-height: 7em) { … }
> ```
## <a id="values"></a>5. Values

This specification also introduces two new values.

The \<ratio\> value is a positive (not zero or negative) \<integer\> followed by optional whitespace, followed by a solidus (‘`/`’), followed by optional whitespace, followed by a positive \<integer\>.

The \<resolution\> value is a positive \<number\> immediately followed by a unit identifier (‘`dpi`’ or ‘`dpcm`’).

Whitespace, \<integer\>, \<number\> and other values used by this specification are the same as in other parts of CSS, normatively defined by CSS 2.1. [\[CSS21\]](#CSS21)

## <a id="units"></a>6. Units

The units used in media queries are the same as in other parts of CSS. For example, the pixel unit represents CSS pixels and not physical pixels.

Relative units in media queries are based on the initial value, which means that units are never based on results of declarations. For example, in HTML, the ‘`em`’ unit is relative to the initial value of ‘`font-size`’.

### <a id="resolution0"></a>6.1. Resolution

The ‘`dpi`’ and ‘`dpcm`’ units describe the resolution of an output device, i.e., the density of device pixels. Resolution unit identifiers are:

dpi  
dots per CSS ‘`inch`’

dpcm  
dots per CSS ‘`centimeter`’

In this specification, these units are only used in the ‘`resolution`’ media feature.

## <a id="changes"></a>7. Changes

### <a id="changes-2022"></a>7.1. Changes Since the 05 April 2022 Recommendation

An earlier Proposed Correction was normatively incorporated into the Recommendation:

- <a id="c1"></a> Former “Proposed Correction 1” in [Section 3.1](#error-handling): Clarify that the keywords ‘`not`’, ‘`and`’, ‘`only`’, and ‘`or`’ should not be treated as unknown media types, but as syntax errors when used in place of media types.

  > Unknown media types evaluate to false. Effectively, they are treated identically to known media types that do not match the media type of the device. <u>However, an exception is made for media types ‘`not`’, ‘`and`’, ‘`only`’, and ‘`or`’. Even though they do match the IDENT production, they must not be treated as unknown media types, but rather trigger the malformed query clause.</u>

  The reasoning for this change can be found in [the minutes of the 2013-05-30 CSS WG teleconference](https://lists.w3.org/Archives/Public/www-style/2013May/0783.html) and in the emails referenced therefrom.

  [Tests for this change](https://github.com/web-platform-tests/wpt/commit/ea1821d4bd24ed1e859db03571cca8e783dbf957) have been added to WPT. The results can be viewed at [wpt.fyi](https://wpt.fyi/results/css/mediaqueries?q=mq-invalid-media-type-0).

A Proposed Correction was introduced:

- [Proposed Correction 2](#c2) in [Section 3.1](#error-handling): Require that ‘`layer`’ is also to not be treated as an unknown media type, but as a syntax errors when used in place of media types.

### <a id="changes-2012"></a>7.2. Changes Since the 19 June 2012 Recommendation

Proposed Corrections were introduced:

- [Proposed Correction 1](#c1) in [Section 3.1](#error-handling): Clarify that the keywords ‘`not`’, ‘`and`’, ‘`only`’, and ‘`or`’ should not be treated as unknown media types, but as syntax errors when used in place of media types.

A handful of editorial and markup corrections were also made:

- [Section 2](#media0): Dropped a redundant attribute in an example.

  > ```text
  > <link rel="stylesheet" media="screen and (color), projection and (color)" rel="stylesheet" href="example.css">
  > ```
  >
  > ```text
  > <link rel="stylesheet" media="screen and (color), projection and (color)" rel="stylesheet" href="example.css" />
  > ```
- [Section 2](#media0): Adjusted a sentence to make it easier for other specifications to extend this one.

  > To avoid circular dependencies, ~~it is never~~<u>unless another feature explicitly specifies that it affects the resolution of Media Queries, it is not</u> necessary to apply the style sheet in order to evaluate expressions.

- [Section 3](#syntax): Used a more precise term to characterize the syntax of css, in a descriptive (rather than prescriptive) sentence.

  > CSS style sheets are generally ~~case-insensitive~~ [<u>ASCII case-insensitive</u>](https://infra.spec.whatwg.org/#ascii-case-insensitive), and this is also the case for media queries.

  The veracity of this claim is validated by a [test](https://wpt.fyi/results/css/mediaqueries/mq-case-insensitive-001.html).

- [Section 4.13](#grid): Corrected a syntax error in an example.

  > ```text
  > @media handheld and (grid) and (device-max-heightmax-device-height: 7em) { … }
  > ```
- Bibliographical references have been updated to point to the latest versions.

- Various links throughout the specification were updated from http to https.

### <a id="changes-2010"></a>7.3. Changes Since the 27 July 2010 Candidate Recommendation

The following changes were made to this specification since the [27 July 2010 Candidate Recommendation](https://www.w3.org/TR/2010/CR-css3-mediaqueries-20100727/):

- [Section 4.11](#resolution): Clarified the meaning of resolution in the case of printers, for which the meaning of dots was ambiguous.

  > <u>For printers, this corresponds to the screening resolution (the resolution for printing dots of arbitrary color).</u>

- [Section 6.1](#units): Made it explicit that the ‘`inch`’ and ‘`cm`’ mentioned are the CSS units, not the physical ones.

  > dpi  
  > dots per <u>CSS ‘`inch`’</u>~~inch~~
  >
  > dpcm  
  > dots per <u>CSS ‘`centimeter`’</u>~~cm~~

- [Section 4.1](#width): Adjust mistaken non normative wording to match correct normative wording from [Section 6](#units).

  > The ‘`em`’ value is relative to the ~~font size of the root element~~<u>initial value of ‘font-size’.</u>.

- [Section 6](#units): Clarify that units are never based on the results of declarations.

  > Relative units in media queries are based on the initial value<u>, which means that units are never based on results of declarations</u>. For example, in HTML, the ‘`em`’ unit is relative to the initial value of ‘`font-size`’.

## <a id="acknowledgments"></a>Acknowledgments

This specification is the product of the W3C Working Group on Cascading Style Sheets.

Comments from Björn Höhrmann, Christoph Päper, Chris Lilley, Simon Pieters, Rijk van Geijtenbeek, Sigurd Lerstad, Arve Bersvendsen, Susan Lesch, Philipp Hoschka, Roger Gimson, Steven Pemberton, Simon Kissane, Melinda Grant, and L. David Baron improved this specification.

## <a id="privacy"></a>Privacy Considerations

No new privacy considerations have been reported on this specification.

## <a id="security"></a>Security Considerations

No new security considerations have been reported on this specification.

## <a id="references"></a>References

### <a id="normative-references"></a>Normative references

<a id="CSS21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification.](css2--REC-CSS2-20110607--1e43327015ed.md) 7 June 2011. W3C Recommendation. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2011&#x2F;REC-CSS2-20110607](css2--REC-CSS2-20110607--1e43327015ed.md)

### <a id="other-references"></a>Other references

<a id="HTML401"></a>\[HTML401\]  
Dave Raggett; Arnaud Le Hors; Ian Jacobs. [HTML 4.01 Specification.](https://www.w3.org/TR/2018/SPSD-html401-20180327/) 24 December 1999, superseded 27 March 2018. W3C Recommendation. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2018&#x2F;SPSD-html401-20180327&#x2F;](https://www.w3.org/TR/2018/SPSD-html401-20180327/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="RFC2531"></a>\[RFC2531\]  
G. Klyne; L. McIntyre. [Content Feature Schema for Internet Fax.](http://www.ietf.org/rfc/rfc2531.txt) March 1999. Internet RFC 2531. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;ietf&#x2E;org&#x2F;rfc&#x2F;rfc2531&#x2E;txt](http://www.ietf.org/rfc/rfc2531.txt)

<a id="XMLSTYLE"></a>\[XMLSTYLE\]  
James Clark; Simon Pieters; Henry S. Thompson [Associating Style Sheets with XML documents 1.0 (Second Edition)](https://www.w3.org/TR/2010/REC-xml-stylesheet-20101028/) 28 October 2010. W3C Recommendation. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2010&#x2F;REC-xml-stylesheet-20101028&#x2F;](https://www.w3.org/TR/2010/REC-xml-stylesheet-20101028/)

<a id="biblio-css-cascade-5"></a><u>\[CSS-CASCADE-5\]</u>  
<u>Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)</u>
