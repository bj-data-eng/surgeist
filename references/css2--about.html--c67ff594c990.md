Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [About the CSS 2.1 Specification](https://www.w3.org/TR/2011/REC-CSS2-20110607/about.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: About the CSS 2.1 Specification

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/about.html

Snapshot SHA-256: c67ff594c99037d087680ac676d7dca89e85e34bfd13ebb0fdee47a15703d5d3

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

<a id="q1.0"></a>

# 1 About the CSS 2.1 Specification

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<a id="css2.1-v-css2"></a>

## 1.1 CSS 2.1 vs CSS 2

The CSS community has gained significant experience with the CSS2 specification since it became a recommendation in 1998. Errors in the CSS2 specification have subsequently been corrected via the publication of various errata, but there has not yet been an opportunity for the specification to be changed based on experience gained.

While many of these issues will be addressed by the upcoming CSS3 specifications, the current state of affairs hinders the implementation and interoperability of CSS2. The CSS 2.1 specification attempts to address this situation by:

- Maintaining compatibility with those portions of CSS2 that are widely accepted and implemented.
- Incorporating all published CSS2 errata.
- Where implementations overwhelmingly differ from the CSS2 specification, modifying the specification to be in accordance with generally accepted practice.
- Removing CSS2 features which, by virtue of not having been implemented, have been rejected by the CSS community. CSS 2.1 aims to reflect what CSS features are reasonably widely implemented for HTML and XML languages in general (rather than <em>only</em> for a particular XML language, or <em>only</em> for HTML).
- Removing CSS2 features that will be obsoleted by CSS3, thus encouraging adoption of the proposed CSS3 features in their place.
- Adding a (very) small number of [new property values,](css2--changes.html--830f199c671e.md#new) when implementation experience has shown that they are needed for implementing CSS2.

Thus, while it is not the case that a CSS2 style sheet is necessarily forwards-compatible with CSS 2.1, it is the case that a style sheet restricting itself to CSS 2.1 features is more likely to find a compliant user agent today and to preserve forwards compatibility in the future. While breaking forward compatibility is not desirable, we believe the advantages to the revisions in CSS 2.1 are worthwhile.

CSS 2.1 is derived from and is intended to replace CSS2. Some parts of CSS2 are unchanged in CSS 2.1, some parts have been altered, and some parts removed. The removed portions may be used in a future CSS3 specification. Future specs should refer to CSS 2.1 (unless they need features from CSS2 which have been dropped in CSS 2.1, and then they should only reference CSS2 for those features, or preferably reference such feature(s) in the respective CSS3 Module that includes those feature(s)).

<a id="reading"></a>

## 1.2 Reading the specification

This section is non-normative.

This specification has been written with two types of readers in mind: CSS authors and CSS implementors. We hope the specification will provide authors with the tools they need to write efficient, attractive, and accessible documents, without overexposing them to CSS's implementation details. Implementors, however, should find all they need to build [conforming user agents](css2--conform.html--de58593b67d7.md#conformance). The specification begins with a general presentation of CSS and becomes more and more technical and specific towards the end. For quick access to information, a general table of contents, specific tables of contents at the beginning of each section, and an index provide easy navigation, in both the electronic and printed versions.

The specification has been written with two modes of presentation in mind: electronic and printed. Although the two presentations will no doubt be similar, readers will find some differences. For example, links will not work in the printed version (obviously), and page numbers will not appear in the electronic version. In case of a discrepancy, the electronic version is considered the authoritative version of the document.

<a id="organization"></a>

## 1.3 How the specification is organized

This section is non-normative.

The specification is organized into the following sections:

<strong>Section 2: An introduction to CSS 2.1</strong>  
The introduction includes a brief tutorial on CSS 2.1 and a discussion of design principles behind CSS 2.1.

<strong>Sections 3 - 18: CSS 2.1 reference manual.</strong>  
The bulk of the reference manual consists of the CSS 2.1 language reference. This reference defines what may go into a CSS 2.1 style sheet (syntax, properties, property values) and how user agents must interpret these style sheets in order to claim [conformance](css2--conform.html--de58593b67d7.md#conformance).

<strong>Appendixes:</strong>  
Appendixes contain information about [aural properties](css2--aural.html--2915ca92d45a.md) (non-normative), [a sample style sheet for HTML 4](css2--sample.html--133769fd2555.md), [changes from CSS2](css2--changes.html--830f199c671e.md), [the grammar of CSS 2.1](css2--grammar.html--eeb984c469da.md), a list of normative and informative [references](css2--refs.html--f208d881d0b7.md), and two indexes: one for [properties](css2--propidx.html--503a73555ead.md) and one [general index](css2--indexlist.html--e51f2555f471.md).

<a id="conventions"></a>

## 1.4 Conventions

<a id="doc-language"></a>

### 1.4.1 [Document language](css2--conform.html--de58593b67d7.md#doclanguage) elements and attributes

- CSS property and pseudo-class names are delimited by single quotes.
- CSS values are delimited by single quotes.
- Document language attribute names are in lowercase letters and delimited by double quotes.

<a id="property-defs"></a>

### 1.4.2 CSS property definitions

Each CSS property definition begins with a summary of key information that resembles the following:

<a id="propdef-property-name"></a>

<strong>'property-name'</strong>

|                       |                                            |
|-----------------------|--------------------------------------------|
| <em>Value:</em>   | legal values &#x26; syntax        |
| <em>Initial:</em>   | initial value                              |
| <em>Applies to:</em>   | elements this property applies to          |
| <em>Inherited:</em>   | whether the property is inherited          |
| <em>Percentages:</em>   | how percentage values are interpreted      |
| <em>Media:</em>   | which media groups the property applies to |
| <em>Computed value:</em>   | how to compute the computed value          |

<a id="value-defs"></a>

#### 1.4.2.1 Value

This part specifies the set of valid values for the property whose name is ['property-name'](css2--about.html--c67ff594c990.md#propdef-property-name). A property value can have one or more components. Component value types are designated in several ways:

1.  <a id="syndata.html#keywords"></a>

    keyword values (e.g., auto, disc, etc.)

2.  basic data types, which appear between "\<" and "\>" (e.g., \<length\>, \<percentage\>, etc.). In the electronic version of the document, each instance of a basic data type links to its definition.

3.  types that have the same range of values as a property bearing the same name (e.g., \<'border-width'\> \<'background-attachment'\>, etc.). In this case, the type name is the property name (complete with quotes) between "\<" and "\>" (e.g., \<'border-width'\>). Such a type does <strong>not</strong> include the value 'inherit'. In the electronic version of the document, each instance of this type of non-terminal links to the corresponding property definition.

4.  non-terminals that do not share the same name as a property. In this case, the non-terminal name appears between "\<" and "\>", as in \<border-width\>. Notice the distinction between \<border-width\> and \<'border-width'\>; the latter is defined in terms of the former. The definition of a non-terminal is located near its first appearance in the specification. In the electronic version of the document, each instance of this type of value links to the corresponding value definition.

Other words in these definitions are keywords that must appear literally, without quotes (e.g., red). The slash (/) and the comma (,) must also appear literally.

Component values may be arranged into property values as follows:

- Several juxtaposed words mean that all of them must occur, in the given order.
- A bar (\|) separates two or more alternatives: exactly one of them must occur.
- A double bar (\|\|) separates two or more options: one or more of them must occur, in any order.
- A double ampersand (&#x26;&#x26;) separates two or more components, all of which must occur, in any order.
- Brackets (\[ \]) are for grouping.

Juxtaposition is stronger than the double ampersand, the double ampersand is stronger than the double bar, and the double bar is stronger than the bar. Thus, the following lines are equivalent:

```text

    a b   |   c ||   d &&   e f
  [ a b ] | [ c || [ d && [ e f ]]]
```
Every type, keyword, or bracketed group may be followed by one of the following modifiers:

- An asterisk (\*) indicates that the preceding type, word, or group occurs zero or more times.
- A plus (+) indicates that the preceding type, word, or group occurs one or more times.
- A question mark (?) indicates that the preceding type, word, or group is optional.
- A pair of numbers in curly braces ({A,B}) indicates that the preceding type, word, or group occurs at least A and at most B times.

The following examples illustrate different value types:

> <em>Value:</em> N \| NW \| NE  
> <em>Value:</em> \[ \<length\> \| thick \| thin \]{1,4}  
> <em>Value:</em> \[\<family-name\> , \]\* \<family-name\>  
> <em>Value:</em> \<uri\>? \<color\> \[ / \<color\> \]?  
> <em>Value:</em> \<uri\> \|\| \<color\>  
> <em>Value:</em> inset? &#x26;&#x26; \[ \<length\>{2,4} &#x26;&#x26; \<color\>? \]

Component values are specified in terms of tokens, as described in [Appendix G.2](css2--grammar.html--eeb984c469da.md#scanner). As the grammar allows spaces between tokens in the components of the `expr` production, spaces may appear between tokens in property values.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In many cases, spaces will in fact be <em>required</em> between tokens in order to distinguish them from each other. For example, the value '1em2em' would be parsed as a single `DIMEN` token with the number '1' and the identifier 'em2em', which is an invalid unit. In this case, a space would be required before the '2' to get this parsed as the two lengths '1em' and '2em'.

<a id="initial-value"></a>

#### 1.4.2.2 Initial

This part specifies the property's initial value. Please consult the section on [the cascade](css2--cascade.html--c7aff33e6f0d.md) for information about the interaction between style sheet-specified, inherited, and initial property values.

<a id="applies-to"></a>

#### 1.4.2.3 Applies to

This part lists the elements to which the property applies. All elements are considered to have all properties, but some properties have no rendering effect on some types of elements. For example, the ['clear'](css2--visuren.html--3f334c530cf4.md#propdef-clear) property only affects block-level elements.

<a id="inherited-prop"></a>

#### 1.4.2.4 Inherited

This part indicates whether the value of the property is inherited from an ancestor element. Please consult the section on [the cascade](css2--cascade.html--c7aff33e6f0d.md) for information about the interaction between style sheet-specified, inherited, and initial property values.

<a id="percentage-wrt"></a>

#### 1.4.2.5 Percentage values

This part indicates how percentages should be interpreted, if they occur in the value of the property. If "N/A" appears here, it means that the property does not accept percentages in its values.

<a id="media-applies"></a>

#### 1.4.2.6 Media groups

This part indicates the [media groups](css2--media.html--3a324a170379.md#media-groups) to which the property applies. Information about media groups is non-normative.

<a id="computed-defs"></a>

#### 1.4.2.7 Computed value

This part describes the computed value for the property. See the section on [computed values](css2--cascade.html--c7aff33e6f0d.md#computed-value) for how this definition is used.

<a id="shorthand"></a>

### 1.4.3 Shorthand properties

<a id="x1"></a>

Some properties are shorthand properties, meaning that they allow authors to specify the values of several properties with a single property.

For instance, the ['font'](css2--fonts.html--d52fc14f36c2.md#propdef-font) property is a shorthand property for setting ['font-style'](css2--fonts.html--d52fc14f36c2.md#propdef-font-style), ['font-variant'](css2--fonts.html--d52fc14f36c2.md#propdef-font-variant), ['font-weight'](css2--fonts.html--d52fc14f36c2.md#propdef-font-weight), ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size), ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height), and ['font-family'](css2--fonts.html--d52fc14f36c2.md#propdef-font-family) all at once.

When values are omitted from a shorthand form, each "missing" property is assigned its initial value (see the section on [the cascade](css2--cascade.html--c7aff33e6f0d.md)).

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The multiple style rules of this example:
>
> ```text
> 
> h1 { 
>   font-weight: bold; 
>   font-size: 12pt;
>   line-height: 14pt; 
>   font-family: Helvetica; 
>   font-variant: normal;
>   font-style: normal;
> }
> ```
>
> may be rewritten with a single shorthand property:
>
> ```text
> 
> h1 { font: bold 12pt/14pt Helvetica }
> ```
>
> In this example, ['font-variant'](css2--fonts.html--d52fc14f36c2.md#propdef-font-variant), and ['font-style'](css2--fonts.html--d52fc14f36c2.md#propdef-font-style) take their initial values.

<a id="notes-and-examples"></a>

### 1.4.4 Notes and examples

All examples that illustrate illegal usage are clearly marked as "ILLEGAL EXAMPLE".

HTML examples lacking DOCTYPE declarations are SGML Text Entities conforming to the HTML 4.01 Strict DTD [\[HTML4\]](css2--refs.html--f208d881d0b7.md#ref-HTML4). Other HTML examples conform to the DTDs given in the examples.

All notes are informative only.

Examples and notes are [marked within the source HTML](css2--conform.html--de58593b67d7.md#defs) for the specification and CSS user agents will render them specially.

<a id="images-and-longdesc"></a>

### 1.4.5 Images and long descriptions

Most images in the electronic version of this specification are accompanied by "long descriptions" of what they represent. A link to the long description is denoted by a "\[D\]" after the image.

Images and long descriptions are informative only.

<a id="acknowledgements"></a>

## 1.5 Acknowledgments

This section is non-normative.

CSS 2.1 is based on CSS2. See the [acknowledgments section of CSS2](https://www.w3.org/TR/2008/REC-CSS2-20080411/about.html#q15) for the people that contributed to CSS2.

We would like to thank the following people who, through their input and feedback on the www-style mailing list, have helped us with the creation of this specification: Andrew Clover, Bernd Mielke, C. Bottelier, Christian Roth, Christoph Päper, Claus Färber, Coises, Craig Saila, Darren Ferguson, Dylan Schiemann, Etan Wexler, George Lund, James Craig, Jan Eirik Olufsen, Jan Roland Eriksson, Joris Huizer, Joshua Prowse, Kai Lahmann, Kevin Smith, Lachlan Cannon, Lars Knoll, Lauri Raittila, Mark Gallagher, Michael Day, Peter Sheerin, Rijk van Geijtenbeek, Robin Berjon, Scott Montgomery, Shelby Moore, Stuart Ballard, Tom Gilder, Vadim Plessky, Peter Moulder, Anton Prowse, G�rard Talbot, Ingo Chao, Bruno Fassino, Justin Rogers, Boris Zbarsky, Garrett Smith, Zack Weinberg, Bjoern Hoehrmann, and the Open eBook Publication Structure Working Group Editors. We would also like to thank Gary Schnabl, Glenn Adams and Susan Lesch who helped proofread earlier versions of this document.

In addition, we would like to extend special thanks to Elika J. Etemad, Ada Chan and Boris Zbarsky who have contributed significant time to CSS 2.1, and to Kimberly Blessing for help with the editing.

Many thanks also to the following people for their help with the test suite: Robert Stam, Aharon Lanin, Alan Gresley, Alan Harder, Alexander Dawson, Arron Eicholz, Bernd Mielke, Bert Bos, Boris Zbarsky, Bruno Fassino, Daniel Schattenkirchner, David Hammond, David Hyatt, Eira Monstad, Elika J. Etemad, G�rard Talbot, Gabriele Romanato, Germain Garand, Hilbrand Edskes, Ian Hickson, James Hopkins, Justin Boss, L. David Baron, Lachlan Hunt, Magne Andersson, Marc Pacheco, Mark McKenzie-Bell, Matt Bradley, Melinda Grant, Michael Turnwall, Ray Kiddy, Richard Ishida, Robert O'Callahan, Simon Montagu, Tom Clancy, Vasil Dinkov, … and all the contributors to the CSS1 test suite.

Working Group members active during the development of this specification: C�sar Acebal (Universidad de Oviedo), Tab Atkins Jr. (Google, Inc.), L. David Baron (Mozilla Foundation), Bert Bos (W3C/ERCIM), Tantek �elik (W3C Invited Experts), Cathy Chan (Nokia), Giorgi Chavchanidze (Opera Software), John Daggett (Mozilla Foundation), Beth Dakin (Apple, Inc.), Arron Eicholz (Microsoft Corp.), Elika J. Etemad (W3C Invited Experts), Simon Fraser (Apple, Inc.), Sylvain Galineau (Microsoft Corp.), Daniel Glazman (Disruptive Innovations), Molly Holzschlag (Opera Software), David Hyatt (Apple, Inc.), Richard Ishida (W3C/ERCIM), John Jansen (Microsoft Corp.), Brad Kemper (W3C Invited Experts), H�kon Wium Lie (Opera Software), Chris Lilley (W3C/ERCIM), Peter Linss (HP), Markus Mielke (Microsoft Corp.), Alex Mogilevsky (Microsoft Corp.), David Singer (Apple Inc.), Anne van Kesteren (Opera Software), Steve Zilles (Adobe Systems Inc.), Ian Hickson (Google, Inc.), Melinda Grant (HP), �yvind Stenhaug (Opera Software), and Paul Nelson (Microsoft Corp.).
