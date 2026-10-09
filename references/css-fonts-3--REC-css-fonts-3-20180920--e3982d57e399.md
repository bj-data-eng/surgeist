Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [CSS Fonts Module Level 3](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/).

Original copyright notice: Copyright © 2018 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Fonts Module Level 3

Source snapshot: https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/

Snapshot SHA-256: e3982d57e3992dfa7c0e56c1ff39977bd71d807731247decc794c61d2eacd63b

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# CSS Fonts Module Level 3

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2018 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](http://www.csail.mit.edu/), [ERCIM](http://www.ercim.eu/), [Keio](http://www.keio.ac.jp/), [Beihang](http://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [document use](https://www.w3.org/Consortium/Legal/copyright-documents) rules apply.

## <a id="abstract"></a>Abstract

This CSS3 module describes how font properties are specified and how font resources are loaded dynamically. The contents of this specification are a consolidation of content previously divided into [CSS3 Fonts](https://www.w3.org/TR/2002/WD-css3-fonts-20020802/) and [CSS3 Web Fonts](https://www.w3.org/TR/2002/WD-css3-webfonts-20020802/) modules. The description of font load events was moved into the [CSS Font Loading](https://drafts.csswg.org/css-font-loading-3/) module.

## <a id="status"></a>Status of this Document

<em>This section describes the status of this document at the time of its publication. Other documents may supersede this document. A list of current W3C publications and the latest revision of this technical report can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index</a> at https&#58;//www&#46;w3&#46;org/TR/.</em>

This document has been reviewed by W3C Members, by software developers, and by other W3C groups and interested parties, and is endorsed by the Director as a W3C Recommendation. It is a stable document and may be used as reference material or cited from another document. W3C's role in making the Recommendation is to draw attention to the specification and to promote its widespread deployment. This enhances the functionality and interoperability of the Web.

This document was produced by the [CSS Working Group](https://www.w3.org/Style/CSS/members) as a [W3C Recommendation.](https://www.w3.org/Consortium/Process/tr#RecsREC).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/#sec-Disclosure).

<a id="w3c_process_revision"></a>

This document is governed by the [1 February 2018 W3C Process Document](https://www.w3.org/2018/Process-20180201/).

A [test suite](https://test.csswg.org/harness/suite/css-fonts-3_dev/) and [implementation report](https://test.csswg.org/harness/results/css-fonts-3_dev/grouped/) are available.

## <a id="introduction"></a>1. Introduction

A font provides a resource containing the visual representation of characters [\[CHARMOD\]](#ref-CHARMOD)[\[UNICODE\]](#ref-UNICODE). At the simplest level it contains information that maps character codes to shapes (called glyphs) that represent these characters. Fonts sharing a common design style are commonly grouped into font families classified by a set of standard font properties. Within a family, the shape displayed for a given character can vary by stroke weight, slant or relative width, among others. An individual font face is described by a unique combination of these properties. For a given range of text, CSS font properties are used to select a font family and a specific font face within that family to be used when rendering that text. As a simple example, to use the bold form of Helvetica one could use:

```text
body {
    font-family: Helvetica;
    font-weight: bold;
}
```
Font resources may be installed locally on the system on which a user agent is running or downloadable. For local font resources descriptive information can be obtained directly from the font resource. For downloadable font resources (sometimes referred to as web fonts), the descriptive information is included with the reference to the font resource.

Families of fonts typically don't contain a single face for each possible variation of font properties. The CSS font selection mechanism describes how to match a given set of CSS font properties to a single font face.

## <a id="typography-background"></a>2. Typography Background

<em>This section is non-normative.</em>

Typographic traditions vary across the globe, so there is no unique way to classify all fonts across languages and cultures. For even common Latin letters, wide variations are possible:

![variations in glyphs for a single character](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/aaaaaa.png)

One character, many glyph variations

Differences in the anatomy of letterforms is one way to distinguish fonts. For Latin fonts, flourishes at the ends of a character's main strokes, or serifs, can distinguish a font from those without. Similar comparisons exist in non-Latin fonts between fonts with tapered strokes and those using primarily uniform strokes:

![serif vs. non-serifs](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/serifvssansserif.png)

Letterforms with and without serifs

![serif vs. non-serifs for japanese](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/minchovsgothic.png)

Similar groupings for Japanese typefaces

Fonts contain letterforms and the data needed to map characters to these letterforms. Often this may be a simple one-to-one mapping, but more complex mappings are also possible. The use of combining diacritic marks creates many variations for an underlying letterform:

![diacritic marks](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/aaaaaa-diacritics.png)

Variations with diacritic marks

A sequence of characters can be represented by a single glyph known as a ligature:

![example of a fi ligature](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/final-ligature.png)

Ligature example

Visual transformations based on textual context are often stylistic option in European languages. They are required to correctly render languages like [\[ARABIC-TYPO\]](#ref-ARABIC-TYPO), the lam and alef characters below <em>must</em> be combined when they exist in sequence:

![lam alef ligature](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/lamaleflig.png)

Required Arabic ligature

The relative complexity of these shaping transformations requires additional data within the font.

Sets of font faces with various stylistic variations are often grouped together into font families. In the simplest case a regular face is supplemented with bold and italic faces, but much more extensive groupings are possible. Variations in the thickness of letterform strokes, the <a id="weight"></a>weight, and the overall proportions of the letterform, the <a id="width"></a>width, are most common. In the example below, each letter uses a different font face within the Univers font family. The width used increases from top to bottom and the weight increases from left to right:

![various width and weight variations within a single family](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/weightwidthvariations.png)

Weight and width variations within a single font family

Creating fonts that support multiple scripts is a difficult task; designers need to understand the cultural traditions surrounding the use of type in different scripts and come up with letterforms that somehow share a common theme. Many languages often share a common script and each of these languages may have noticeable stylistic differences. For example, the Arabic script, when used for Persian and Urdu, exhibits significant and systematic differences in letterforms, as does Cyrillic when used with languages such as Serbian and Russian.

The [<em>character map</em>](#character-map) of a font defines the mapping of characters to glyphs for that font. If a document contains characters not supported by the [<em title="character map">character maps</em>](#character-map) of the fonts contained in a font family list, a user agent may use a [<em>system font fallback</em>](#system-font-fallback) procedure to locate an appropriate font that does. If no appropriate font can be found, some form of "missing glyph" character will be rendered by the user agent. System fallback can occur when the specified list of font families does not include a font that supports a given character.

Although the [<em>character map</em>](#character-map) of a font maps a given character to a glyph for that character, modern font technologies such as OpenType [\[OPENTYPE\]](#ref-OPENTYPE) and AAT (Apple Advanced Typography) [\[AAT-FEATURES\]](#ref-AAT-FEATURES) provide ways of mapping a character to different glyphs based upon feature settings. Fonts in these formats allow these features to be embedded in the font itself and controlled by applications. Common typographic features which can be specified this way include ligatures, swashes, contextual alternates, proportional and tabular figures, and automatic fractions, to list just a few. For a visual overview of OpenType features, see the [\[OPENTYPE-FONT-GUIDE\]](#ref-OPENTYPE-FONT-GUIDE).

## <a id="basic-font-props"></a>3. Basic Font Properties

The particular font face used to render a character is determined by the font family and other font properties that apply to a given element. This structure allows settings to be varied independent of each other.

### <a id="font-family-prop"></a>3.1. Font family: the [font-family](#propdef-font-family) property

|                 |                                                                                                    |
|-----------------|----------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-family"></a>font-family                                                                     |
| Value:          | \[ [<var>&lt;family-name&gt;</var>](#family-name-value) \| [<var>&lt;generic-family&gt;</var>](#generic-family-value) \] \# |
| Initial:        | depends on user agent                                                                              |
| Applies to:     | all elements                                                                                       |
| Inherited:      | yes                                                                                                |
| Percentages:    | N/A                                                                                                |
| Media:          | visual                                                                                             |
| Computed value: | as specified                                                                                       |
| Animatable:     | no                                                                                                 |

This property specifies a prioritized list of font family names or generic family names. A font family defines a set of faces that vary in weight, width or slope. CSS uses the combination of a family name with other style attributes to select an individual face. Using this selection mechanism, rather than selecting a face via the style name as is often done in design applications, allows some degree of regularity in textual display when fallback occurs.

> <strong data-conversion-semantic="note">Note</strong>
>
> Designers should note that the CSS definition of font attributes used for selection are explicitly not intended to define a font taxonomy. A type designer's idea of a family may often extend to a set of faces that vary along axes other than just the standard axes of weight, width and slope. A family may extend to include both a set of serif faces and a set of sans-serif faces or vary along axes that are unique to that family. The CSS font selection mechanism merely provides a way to determine the “closest” substitute when substitution is necessary.

Unlike other CSS properties, component values are a comma-separated list indicating alternatives. A user agent iterates through the list of family names until it matches an available font that contains a glyph for the character to be rendered. This allows for differences in available fonts across platforms and for differences in the range of characters supported by individual fonts.

A font family name only specifies a name given to a set of font faces, it does not specify an individual face. For example, given the availability of the fonts below, Futura would match but Futura Medium would not:

![family and face names](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/familyvsfacename.png)

Family and individual face names

Consider the example below:

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> body {
>     font-family: Helvetica, Verdana, sans-serif;
> }
> ```
>
> If Helvetica is available it will be used when rendering. If neither Helvetica nor Verdana is present, then the user-agent-defined sans serif font will be used.

There are two types of font family names:

<a id="family-name-value"></a><var>&lt;family-name&gt;</var>  
The name of a font family of choice such as Helvetica or Verdana in the previous example.

<a id="generic-family-value"></a><var>&lt;generic-family&gt;</var>  
The following generic family keywords are defined: ‘[`serif`](#serif)’, ‘[`sans-serif`](#sans-serif)’, ‘[`cursive`](#cursive)’, ‘[`fantasy`](#fantasy)’, and ‘[`monospace`](#monospace)’. These keywords can be used as a general fallback mechanism when an author's desired font choices are not available. As keywords, they must not be quoted. Authors are encouraged to append a generic font family as a last alternative for improved robustness.

Font family names other than generic families must either be given quoted as [strings,](https://www.w3.org/TR/CSS21/syndata.html#strings) or unquoted as a sequence of one or more [identifiers.](https://www.w3.org/TR/CSS21/syndata.html#value-def-identifier) This means most punctuation characters and digits at the start of each token must be escaped in unquoted font family names.

To illustrate this, the following declarations are invalid:

```text

font-family: Red/Black, sans-serif;
font-family: "Lucida" Grande, sans-serif;
font-family: Ahem!, sans-serif;
font-family: test@foo, sans-serif;
font-family: #POUND, sans-serif;
font-family: Hawaii 5-0, sans-serif;
```
If a sequence of identifiers is given as a font family name, the computed value is the name converted to a string by joining all the identifiers in the sequence by single spaces.

To avoid mistakes in escaping, it is recommended to quote font family names that contain white space, digits, or punctuation characters other than hyphens:

```text

body { font-family: "New Century Schoolbook", serif }

<BODY STYLE="font-family: '21st Century', fantasy">
```
Font family <em>names</em> that happen to be the same as keyword value (‘`inherit`’, ‘[`serif`](#serif)’, etc.) must be quoted to prevent confusion with the keywords with the same names. UAs must not consider these keywords as matching the [<var>&lt;family-name&gt;</var>](#family-name-value) type. This applies to any keyword across all of CSS.

The precise way a set of fonts are grouped into font families varies depending upon the platform font management API's. The Windows GDI API only allows four faces to be grouped into a family while the DirectWrite API and API's on OSX and other platforms support font families with a variety of weights, widths and slopes (see [Appendix A](#platform-props-to-css) for more details).

Some font formats allow fonts to carry multiple localizations of the family name. User agents must recognize and correctly match all of these names independent of the underlying platform localization, system API used or document encoding:

![examples of localized family names](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/localizedfamilynames.png)

Localized family names

The details of localized font family name matching and the corresponding issues of case sensitivity are described below in the [font matching](#font-family-casing) section.

#### <a id="generic-font-families"></a>3.1.1. Generic font families

All five generic font families must always result in at least one matched font face, for all CSS implementations. However, the generics may be composite faces (with different typefaces based on such things as the Unicode range of the character, the language of the containing element, user preferences and system settings, among others). They are also not guaranteed to always be different from each other.

User agents should provide reasonable default choices for the generic font families, which express the characteristics of each family as well as possible, within the limits allowed by the underlying technology. User agents are encouraged to allow users to select alternative choices for the generic fonts.

<a id="serif-def"></a>

<a id="serif"></a>

##### <a id="serif0"></a>serif

Serif fonts represent the formal text style for a script. This often means but is not limited to glyphs that have finishing strokes, flared or tapering ends, or have actual serifed endings (including slab serifs). Serif fonts are typically proportionately-spaced. They often display a greater variation between thick and thin strokes than fonts from the ‘[`sans-serif`](#sans-serif)’ generic font family. CSS uses the term ‘[`serif`](#serif)’ to apply to a font for any script, although other names may be more familiar for particular scripts, such as Mincho (Japanese), Sung or Song (Chinese), Batang (Korean). For Arabic, the Naskh style would correspond to ‘[`serif`](#serif)’ more due to its typographic role rather than its actual design style. Any font that is so described may be used to represent the generic ‘[`serif`](#serif)’ family.

![sample serif fonts](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/serifexamples.png)

Sample serif fonts

<a id="sans-serif-def"></a>

<a id="sans-serif"></a>

##### <a id="sans-serif0"></a> sans-serif

Glyphs in sans-serif fonts, as the term is used in CSS, are generally low contrast (vertical and horizontal stems have the close to the same thickness) and have stroke endings that are plain — without any flaring, cross stroke, or other ornamentation. Sans-serif fonts are typically proportionately-spaced. They often have little variation between thick and thin strokes, compared to fonts from the ‘[`serif`](#serif)’ family. CSS uses the term ‘[`sans-serif`](#sans-serif)’ to apply to a font for any script, although other names may be more familiar for particular scripts, such as Gothic (Japanese), Hei (Chinese), or Gulim (Korean). Any font that is so described may be used to represent the generic ‘[`sans-serif`](#sans-serif)’ family.

![sample sans-serif fonts](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/sansserifexamples.png)

Sample sans-serif fonts

<a id="cursive-def"></a>

<a id="cursive"></a>

##### <a id="cursive0"></a> cursive

Glyphs in cursive fonts generally use a more informal script style, and the result looks more like handwritten pen or brush writing than printed letterwork. CSS uses the term ‘[`cursive`](#cursive)’ to apply to a font for any script, although other names such as Chancery, Brush, Swing and Script are also used in font names.

![sample cursive fonts](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/cursiveexamples.png)

Sample cursive fonts

<a id="fantasy-def"></a>

<a id="fantasy"></a>

##### <a id="fantasy0"></a> fantasy

Fantasy fonts are primarily decorative or expressive fonts that contain decorative or expressive representations of characters. These do not include Pi or Picture fonts which do not represent actual characters.

![sample fantasy fonts](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/fantasyexamples.png)

Sample fantasy fonts

<a id="monospace-def"></a>

<a id="monospace"></a>

##### <a id="monospace0"></a> monospace

The sole criterion of a monospace font is that all glyphs have the same fixed width. This is often used to render samples of computer code.

![sample monospace fonts](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/monospaceexamples.png)

Sample monospace fonts

### <a id="font-weight-prop"></a>3.2. Font weight: the [font-weight](#propdef-font-weight) property

|                 |                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|-----------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-weight"></a>font-weight                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Value:          | [normal](#font-weight-normal-value) \| [bold](#bold) \| [bolder](#bolder) \| [lighter](#lighter) \| [100](#font-weight-numeric-values) \| [200](#font-weight-numeric-values) \| [300](#font-weight-numeric-values) \| [400](#font-weight-numeric-values) \| [500](#font-weight-numeric-values) \| [600](#font-weight-numeric-values) \| [700](#font-weight-numeric-values) \| [800](#font-weight-numeric-values) \| [900](#font-weight-numeric-values) |
| Initial:        | normal                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Applies to:     | all elements                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| Inherited:      | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| Percentages:    | N/A                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| Media:          | visual                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Computed value: | numeric weight value (see description)                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Animatable:     | as [font weight](https://drafts.csswg.org/css-transitions/#animtype-font-weight)                                                                                                                                                                                                                                                                                                                                                                       |

The [‘`font-weight`’](#propdef-font-weight) property specifies the weight of glyphs in the font, their degree of blackness or stroke thickness.

Values have the following meanings:

<a id="font-weight-numeric-values"></a>100 to 900  
These values form an ordered sequence, where each number indicates a weight that is at least as dark as its predecessor. These roughly correspond to the commonly used weight names below:

- 100 - Thin
- 200 - Extra Light (Ultra Light)
- 300 - Light
- 400 - Normal
- 500 - Medium
- 600 - Semi Bold (Demi Bold)
- 700 - Bold
- 800 - Extra Bold (Ultra Bold)
- 900 - Black (Heavy)

<a id="font-weight-normal-value"></a>normal  
Same as ‘`400`’.

<a id="bold"></a>bold  
Same as ‘`700`’.

<a id="bolder"></a>bolder  
Specifies a bolder weight than the inherited value.

<a id="lighter"></a>lighter  
Specifies a lighter weight than the inherited value.

Font formats that use a scale other than a nine-step scale should map their scale onto the CSS scale so that 400 roughly corresponds with a face that would be labeled as Regular, Book, Roman and 700 roughly matches a face that would be labeled as Bold. Or weights may be inferred from the style names, ones that correspond roughly with the scale above. The scale is relative, so a face with a larger weight value must never appear lighter. If style names are used to infer weights, care should be taken to handle variations in style names across locales.

Quite often there are only a few weights available for a particular font family. When a weight is specified for which no face exists, a face with a nearby weight is used. In general, bold weights map to faces with heavier weights and light weights map to faces with lighter weights (see the [font matching section below](#font-matching-algorithm) for a precise definition). The examples here illustrate which face is used for different weights, grey indicates a face for that weight does not exist so a face with a nearby weight is used:

![weight mappings for a family with 400, 700 and 900 weights](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/optimaweights.png)

Weight mappings for a font family with 400, 700 and 900 weight faces

![weight mappings for a family with 300, 600 weights](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/hiraginoweights.png)

Weight mappings for a font family with 300 and 600 weight faces

Although the practice is not well-loved by typographers, bold faces are often synthesized by user agents for faces that lack actual bold faces. For the purposes of style matching, these faces must be treated as if they exist within the family. Authors can explicitly avoid this behavior by using the ‘[`font-synthesis`](#propdef-font-synthesis)’ property.

Specified values of ‘[`bolder`](#bolder)’ and ‘[`lighter`](#lighter)’ indicate weights relative to the weight of the parent element. The computed weight is calculated based on the inherited [‘`font-weight`’](#propdef-font-weight) value using the chart below.

<a id="bolderlighter"></a>

| Inherited value | bolder | lighter |
|-----------------|--------|---------|
| 100             | 400    | 100     |
| 200             | 400    | 100     |
| 300             | 400    | 100     |
| 400             | 700    | 100     |
| 500             | 700    | 100     |
| 600             | 900    | 400     |
| 700             | 900    | 400     |
| 800             | 900    | 700     |
| 900             | 900    | 700     |

The table above is equivalent to selecting the next relative bolder or lighter face, given a font family containing normal and bold faces along with a thin and a heavy face. Authors who desire finer control over the exact weight values used for a given element may use numerical values instead of relative weights.

### <a id="font-stretch-prop"></a>3.3. Font width: the [font-stretch](#propdef-font-stretch) property

|                 |                                                                                                                                                                                                                                                                                                                           |
|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-stretch"></a>font-stretch                                                                                                                                                                                                                                                                                           |
| Value:          | [normal](#font-stretch-normal-value) \| [ultra-condensed](#ultra-condensed) \| [extra-condensed](#extra-condensed) \| [condensed](#condensed) \| [semi-condensed](#semi-condensed-) \| [semi-expanded](#semi-expanded) \| [expanded](#expanded) \| [extra-expanded](#extra-expanded) \| [ultra-expanded](#ultra-expanded) |
| Initial:        | normal                                                                                                                                                                                                                                                                                                                    |
| Applies to:     | all elements                                                                                                                                                                                                                                                                                                              |
| Inherited:      | yes                                                                                                                                                                                                                                                                                                                       |
| Percentages:    | N/A                                                                                                                                                                                                                                                                                                                       |
| Media:          | visual                                                                                                                                                                                                                                                                                                                    |
| Computed value: | as specified                                                                                                                                                                                                                                                                                                              |
| Animatable:     | as [font stretch](#font-stretch-animation)                                                                                                                                                                                                                                                                                |

The [‘`font-stretch`’](#propdef-font-stretch) property selects a normal, condensed, or expanded face from a font family. Absolute keyword values have the following ordering, from narrowest to widest:

- <a id="ultra-condensed"></a>ultra-condensed
- <a id="extra-condensed"></a>extra-condensed
- <a id="condensed"></a>condensed
- <a id="semi-condensed-"></a>semi-condensed
- <a id="font-stretch-normal-value"></a>normal
- <a id="semi-expanded"></a>semi-expanded
- <a id="expanded"></a>expanded
- <a id="extra-expanded"></a>extra-expanded
- <a id="ultra-expanded"></a>ultra-expanded

When a face does not exist for a given width, normal or condensed values map to a narrower face, otherwise a wider face. Conversely, expanded values map to a wider face, otherwise a narrower face. The figure below shows how the nine font-stretch property settings affect font selection for font family containing a variety of widths, grey indicates a width for which no face exists and a different width is substituted:

![width mappings for a family with condensed, normal and expanded faces](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/universwidths.png)

Width mappings for a font family with condensed, normal and expanded width faces

<a id="font-stretch-animation"></a>Animation of font stretch: Font stretch is interpolated in discrete steps. The interpolation happens as though the ordered values are equally spaced real numbers. The interpolation result is rounded to the nearest value, with values exactly halfway between two values rounded towards the later value in the list above.

### <a id="font-style-prop"></a>3.4. Font style: the [font-style](#propdef-font-style) property

|                 |                                                                                |
|-----------------|--------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-style"></a>font-style                                                  |
| Value:          | [normal](#font-style-normal-value) \| [italic](#italic) \| [oblique](#oblique) |
| Initial:        | normal                                                                         |
| Applies to:     | all elements                                                                   |
| Inherited:      | yes                                                                            |
| Percentages:    | N/A                                                                            |
| Media:          | visual                                                                         |
| Computed value: | as specified                                                                   |
| Animatable:     | no                                                                             |

The [‘`font-style`’](#propdef-font-style) property allows italic or oblique faces to be selected. Italic forms are generally cursive in nature while oblique faces are typically sloped versions of the regular face. Oblique faces can be simulated by artificially sloping the glyphs of the regular face. Compare the artificially sloped renderings of Palatino ‘`a`’ and Baskerville ‘`N`’ in grey with the actual italic versions:

![artificial sloping vs. real italics](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/realvsfakeitalics.png)

Artificial sloping versus real italics

Values have the following meanings:

<a id="font-style-normal-value"></a>normal  
selects a face that is classified as a normal face, one that is neither italic or obliqued

<a id="italic"></a>italic  
selects a font that is labeled as an italic face, or an oblique face if one is not

<a id="oblique"></a>oblique  
selects a font that is labeled as an oblique face, or an italic face if one is not

If no italic or oblique face is available, oblique faces can be synthesized by rendering non-obliqued faces with an artificial obliquing operation. The use of these artificially obliqued faces can be disabled using the ‘[`font-synthesis`](#propdef-font-synthesis)’ property. The details of the obliquing operation are not explicitly defined.

> <strong data-conversion-semantic="note">Note</strong>
>
> Authors should also be aware that synthesized approaches may not be suitable for scripts like Cyrillic, where italic forms are very different in shape. It is always better to use an actual italic font rather than rely on a synthetic version.

Many scripts lack the tradition of mixing a cursive form within text rendered with a normal face. Chinese, Japanese and Korean fonts almost always lack italic or oblique faces. Fonts that support a mixture of scripts will sometimes omit specific scripts such as Arabic from the set of glyphs supported in the italic face. User agents should be careful about making [<em>character map</em>](#character-map) assumptions across faces when implementing support for [<em>system font fallback</em>](#system-font-fallback).

### <a id="font-size-prop"></a>3.5. Font size: the [font-size](#propdef-font-size) property

|                 |                                                                                                                   |
|-----------------|-------------------------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-size"></a>font-size                                                                                      |
| Value:          | [<var>&lt;absolute-size&gt;</var>](#absolute-size-value) \| [<var>&lt;relative-size&gt;</var>](#relative-size-value) \| <var>&lt;length-percentage&gt;</var> |
| Initial:        | medium                                                                                                            |
| Applies to:     | all elements                                                                                                      |
| Inherited:      | yes                                                                                                               |
| Percentages:    | refer to parent element's font size                                                                               |
| Media:          | visual                                                                                                            |
| Computed value: | absolute length                                                                                                   |
| Animatable:     | as [length](https://drafts.csswg.org/css-transitions/#animtype-length)                                            |

This property indicates the desired height of glyphs from the font. For scalable fonts, the font-size is a scale factor applied to the EM unit of the font. (Note that certain glyphs may bleed outside their EM box.) For non-scalable fonts, the font-size is converted into absolute units and matched against the declared ‘[`font-size`](#propdef-font-size)’ of the font, using the same absolute coordinate space for both of the matched values. Values have the following meanings:

<a id="absolute-size-value"></a><var>&lt;absolute-size&gt;</var>

An [<var>&lt;absolute-size&gt;</var>](#absolute-size-value) keyword refers to an entry in a table of font sizes computed and kept by the user agent. Possible values are:

```text
 [ xx-small | x-small | small | medium | large | x-large |
     xx-large ]
```
<a id="relative-size-value"></a><var>&lt;relative-size&gt;</var>

A [<var>&lt;relative-size&gt;</var>](#relative-size-value) keyword is interpreted relative to the table of font sizes and the computed ‘[`font-size`](#propdef-font-size)’ of the parent element. Possible values are:

`  [ larger | smaller ] `

For example, if the parent element has a font size of ‘`medium`’, a value of ‘`larger`’ will make the font size of the current element be ‘`large`’. If the parent element's size is not close to a table entry, the user agent is free to interpolate between table entries or round off to the closest one. The user agent may have to extrapolate table values if the numerical value goes beyond the keywords.

<a id="length-percentage-size-value"></a>

[<var>&lt;length-percentage&gt;</var>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage)

A length value [\[CSS-VALUES\]](#ref-CSS-VALUES) specifies an absolute font size (independent of the user agent's font table). Negative lengths are invalid.

<a id="ref-for-em"></a>

A percentage value specifies an absolute font size relative to the parent element's font size. Use of percentage values, or values in [em](https://drafts.csswg.org/css-values-4/#em)s, leads to more robust and cascadable style sheets. Negative percentages are invalid.

The following table provides user agent guidelines for the absolute-size scaling factor and their mapping to HTML heading and absolute font-sizes. The ‘`medium`’ value is used as the reference middle value. The user agent may fine-tune these values for different fonts or different types of display devices.

| CSS absolute-size values | `xx-small` | `x-small` | `small` | `medium` | `large` | `x-large` | `xx-large` |     |
|--------------------------|-------------------|-------------------|-------------------|-------------------|-------------------|-------------------|-------------------|-----|
| scaling factor           | 3/5               | 3/4               | 8/9               | 1                 | 6/5               | 3/2               | 2/1               | 3/1 |
| HTML headings            | h6                |                   | h5                | h4                | h3                | h2                | h1                |     |
| HTML font sizes          | 1                 |                   | 2                 | 3                 | 4                 | 5                 | 6                 | 7   |

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note 1.</strong> To preserve readability, an UA
   applying these guidelines should nevertheless avoid creating font-size
   resulting in less than 9 device pixels per EM unit on a computer
   display.</em>

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note 2.</strong> In CSS1, the suggested scaling
   factor between adjacent indexes was 1.5 which user experience proved to be
   too large. In CSS2, the suggested scaling factor for computer screen
   between adjacent indexes was 1.2 which still created issues for the small
   sizes. The new scaling factor varies between each index to provide a
   better readability.</em>

The actual value of this property may differ from the computed value due a numerical value on ‘[`font-size-adjust`](#propdef-font-size-adjust)’ and the unavailability of certain font sizes.

Child elements inherit the computed [‘`font-size`’](#font-size-prop) value (otherwise, the effect of [‘`font-size-adjust`’](#font-size-adjust-prop) would compound).

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> p { font-size: 12pt; }
> blockquote { font-size: larger }
> em { font-size: 150% }
> em { font-size: 1.5em }
> ```
### <a id="font-size-adjust-prop"></a>3.6. Relative sizing: the [font-size-adjust](#propdef-font-size-adjust) property

|                 |                                                                                    |
|-----------------|------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-size-adjust"></a>font-size-adjust                                                |
| Value:          | [none](#font-size-adjust-none-value) \| [<var>&lt;number&gt;</var>](#aspect-ratio-value) |
| Initial:        | none                                                                               |
| Applies to:     | all elements                                                                       |
| Inherited:      | yes                                                                                |
| Percentages:    | N/A                                                                                |
| Media:          | visual                                                                             |
| Computed value: | as specified                                                                       |
| Animatable:     | as [number](https://drafts.csswg.org/css-transitions/#animtype-number)             |

<a id="aspect-value0"></a>

<a id="aspect-value"></a>

For any given font size, the apparent size and legibility of text varies across fonts. For scripts such as Latin or Cyrillic that distinguish between upper and lowercase letters, the relative height of lowercase letters compared to their uppercase counterparts is a determining factor of legibility. This is commonly referred to as the [aspect value](#aspect-value). Precisely defined, it is equal to the `x-height` of a font divided by the font size.

In situations where font fallback occurs, fallback fonts may not share the same aspect value as the desired font family and will thus appear less readable. The ‘[`font-size-adjust`](#propdef-font-size-adjust)’ property is a way to preserve the readability of text when font fallback occurs. It does this by adjusting the font-size so that the x-height is the same regardless of the font used.

> <strong data-conversion-semantic="example">Example</strong>
>
> The style defined below defines Verdana as the desired font family, but if Verdana is not available Futura or Times will be used.
>
> ```text
> p {
>     font-family: Verdana, Futura, Times;
> }
> 
> <p>Lorem ipsum dolor sit amet, ...</p>
> ```
>
> Verdana has a relatively high aspect value, lowercase letters are relatively tall compared to uppercase letters, so at small sizes text appears legible. Times has a lower aspect value and so if fallback occurs, the text will be less legible at small sizes than Verdana.

How text rendered in each of these fonts compares is shown below, the columns show text rendered in Verdana, Futura and Times. The same font-size value is used across cells within each row and red lines are included to show the differences in x-height. In the upper half each row is rendered in the same font-size value. The same is true for the lower half but in this half the ‘[`font-size-adjust`](#propdef-font-size-adjust)’ property is also set so that the actual font size is adjusted to preserve the x-height across each row. Note how small text remains relatively legible across each row in the lower half.

![text with and without 'font-size-adjust'](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/fontsizeadjust.png)

Text with and without the use of ‘[`font-size-adjust`](#propdef-font-size-adjust)’

This property allows authors to specify an [<i>aspect value</i>](#aspect-value) for an element that will effectively preserve the x-height of the first choice font, whether it is substituted or not. Values have the following meanings:

<a id="font-size-adjust-none-value"></a>none  
Do not preserve the font's x-height.

<a id="aspect-ratio-value"></a><var>&lt;number&gt;</var>  
Specifies the [<i>aspect value</i>](#aspect-value) used in the calculation below to calculate the adjusted font size:

```text
c  =  ( a / a' ) s
```
where:

```text
s  =  font-size value
a  =  aspect value as specified by the 'font-size-adjust' property
a' =  aspect value of actual font
c  =  adjusted font-size to use
```
Negative values are invalid.

This value applies to any font that is selected but in typical usage it should be based on the [<i>aspect
     value</i>](#aspect-value) of the first font in the font-family list. If this is specified accurately, the (a/a') term in the formula above is effectively 1 for the first font and no adjustment occurs. If the value is specified inaccurately, text rendered using the first font in the family list will display differently in older user agents that don't support ‘[`font-size-adjust`](#propdef-font-size-adjust)’.

The value of ‘[`font-size-adjust`](#propdef-font-size-adjust)’ affects the used value of ‘[`font-size`](#propdef-font-size)’ but does not affect the computed value. It affects the size of relative units that are based on font metrics of the [<i>first available
   font</i>](#first-available-font) such as `ex` and `ch` but does not affect the size of `em` units. Since numeric values of [‘`line-height`’](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) refer to the computed size of ‘[`font-size`](#propdef-font-size)’, ‘[`font-size-adjust`](#propdef-font-size-adjust)’ does not affect the used value of [‘`line-height`’](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height).

> <strong data-conversion-semantic="note">Note</strong>
>
> In CSS, authors often specify [‘`line-height`’](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) as a multiple of the ‘[`font-size`](#propdef-font-size)’. Since the ‘[`font-size-adjust`](#propdef-font-size-adjust)’ property affects the used value of ‘[`font-size`](#propdef-font-size)’, authors should take care setting the line height when ‘[`font-size-adjust`](#propdef-font-size-adjust)’ is used. Setting the line height too tightly can result in overlapping lines of text in this situation.

Authors can calculate the [<i>aspect
   value</i>](#aspect-value) for a given font by comparing spans with the same content but different ‘[`font-size-adjust`](#propdef-font-size-adjust)’ properties. If the same font-size is used, the spans will match when the ‘[`font-size-adjust`](#propdef-font-size-adjust)’ value is accurate for the given font.

> <strong data-conversion-semantic="example">Example</strong>
>
> Two spans with borders are used to determine the [<i>aspect value</i>](#aspect-value) of a font. The ‘[`font-size`](#propdef-font-size)’ is the same for both spans but the ‘[`font-size-adjust`](#propdef-font-size-adjust)’ property is specified only for the right span. Starting with a value of 0.5, the aspect value can be adjusted until the borders around the two letters line up.
>
> ```text
> p {
>     font-family: Futura;
>     font-size: 500px;
> }
> 
> span {
>     border: solid 1px red;
> }
> 
> .adjust {
>     font-size-adjust: 0.5;
> }
> 
> <p><span>b</span><span class="adjust">b</span></p>
> ```
>
> ![Futura with an \<i\>aspect value\</i\> of 0.5](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/beforefontsizeadjust.png)
>
> Futura with an [<i>aspect
     value</i>](#aspect-value) of 0.5
>
> The box on the right is a bit bigger than the one on the left, so the [<i>aspect value</i>](#aspect-value) of this font is something less than 0.5. Adjust the value until the boxes align.

### <a id="font-prop"></a>3.7. Shorthand font property: the [font](#propdef-font) property

|                 |                                                                                                                                                                                                                                                                                         |
|-----------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font"></a>font                                                                                                                                                                                                                                                                 |
| Value:          | \[ \[ <var>&lt;<a href="#propdef-font-style" title="font-style!!property">‘<code>font-style</code>’</a>&gt;</var> \|\| [<var>&lt;font-variant-css21&gt;</var>](#font-variant-css21-values) \|\| <var>&lt;<a href="#propdef-font-weight" title="font-weight!!property">‘<code>font-weight</code>’</a>&gt;</var> \|\| <var>&lt;<a href="#propdef-font-stretch" title="font-stretch!!property">‘<code>font-stretch</code>’</a>&gt;</var> \]? <var>&lt;‘<a href="#propdef-font-size"><code>font-size</code></a>’&gt;</var> \[ / <var>&lt;‘<code>line-height</code>’&gt;</var> \]? <var>&lt;<a href="#propdef-font-family" title="font-family!!property">‘<code>font-family</code>’</a>&gt;</var> \] \| caption \| icon \| menu \| message-box \| small-caption \| status-bar |
| Initial:        | see individual properties                                                                                                                                                                                                                                                               |
| Applies to:     | all elements                                                                                                                                                                                                                                                                            |
| Inherited:      | yes                                                                                                                                                                                                                                                                                     |
| Percentages:    | see individual properties                                                                                                                                                                                                                                                               |
| Media:          | visual                                                                                                                                                                                                                                                                                  |
| Computed value: | see individual properties                                                                                                                                                                                                                                                               |
| Animatable:     | see individual properties                                                                                                                                                                                                                                                               |

The [‘`font`’](#propdef-font) property is, except as described below, a shorthand property for setting [‘`font-style`’](#propdef-font-style), [‘`font-variant`’](#propdef-font-variant), [‘`font-weight`’](#propdef-font-weight), [‘`font-stretch`’](#propdef-font-stretch), [‘`font-size`’](#propdef-font-size), [‘`line-height`’](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height), [‘`font-family`’](#propdef-font-family) at the same place in the stylesheet. Values for the [‘`font-variant`’](#propdef-font-variant) property may also be included but only those supported in CSS 2.1, none of the [‘`font-variant`’](#propdef-font-variant) values added in this specification can be used in the [‘`font`’](#propdef-font) shorthand:

<a id="font-variant-css21-values"></a>

```text
<font-variant-css21> = [normal | small-caps]
```
The syntax of this property is based on a traditional typographical shorthand notation to set multiple properties related to fonts.

All subproperties of the ‘[`font`](#propdef-font)’ property are first reset to their initial values, including those listed above plus [‘`font-size-adjust`’](#propdef-font-size-adjust), [‘`font-kerning`’](#propdef-font-kerning), all subproperties of [‘`font-variant`’](#propdef-font-variant), and ‘`font-feature-settings`’, but <em>not</em> [‘`font-synthesis`’](#propdef-font-synthesis). Then, those properties that are given explicit values in the [‘`font`’](#propdef-font) shorthand are set to those values. For a definition of allowed and initial values, see the previously defined properties. For reasons of backwards compatibility, it is not possible to set [‘`font-size-adjust`’](#propdef-font-size-adjust) to anything other than its initial value using the [‘`font`’](#propdef-font) shorthand property; instead, use the individual property.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> p { font: 12pt/14pt sans-serif }
> p { font: 80% sans-serif }
> p { font: x-large/110% "new century schoolbook", serif }
> p { font: bold italic large Palatino, serif }
> p { font: normal small-caps 120%/120% fantasy }
> p { font: condensed oblique 12pt "Helvetica Neue", serif; }
> ```
>
> In the second rule, the font size percentage value (‘`80%`’) refers to the computed ‘[`font-size`](#propdef-font-size)’ of the parent element. In the third rule, the line height percentage (‘`110%`’) refers to the font size of the element itself.
>
> The first three rules do not specify the [‘`font-variant`’](#propdef-font-variant) and [‘`font-weight`’](#propdef-font-weight) explicitly, so these properties receive their initial values (‘`normal`’). Notice that the font family name "new century schoolbook", which contains spaces, is enclosed in quotes. The fourth rule sets the [‘`font-weight`’](#propdef-font-weight) to ‘[`bold`](#bold)’, the [‘`font-style`’](#propdef-font-style) to ‘[`italic`](#italic)’, and implicitly sets [‘`font-variant`’](#propdef-font-variant) to [‘`normal`’](#font-variant-normal-value).
>
> The fifth rule sets the [‘`font-variant`’](#propdef-font-variant) (‘[`small-caps`](#small-caps)’), the [‘`font-size`’](#propdef-font-size) (120% of the parent's font size), the [‘`line-height`’](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) (120% of the font size) and the [‘`font-family`’](#propdef-font-family) (‘[`fantasy`](#fantasy)’). It follows that the keyword ‘`normal`’ applies to the two remaining properties: [‘`font-style`’](#propdef-font-style) and [‘`font-weight`’](#propdef-font-weight).
>
> The sixth rule sets the [‘`font-style`’](#propdef-font-style), [‘`font-stretch`’](#propdef-font-stretch), [‘`font-size`’](#propdef-font-size), and [‘`font-family`’](#propdef-font-family), the other font properties being set to their initial values.

Since the [‘`font-stretch`’](#propdef-font-stretch) property was not defined in CSS 2.1, when using [‘`font-stretch`’](#propdef-font-stretch) values within ‘[`font`](#propdef-font)’ rules, authors should include a extra version compatible with older user agents:

```text
p {
  font: 80% sans-serif;   /* for older user agents */
  font: condensed 80% sans-serif;
}
```
The following values refer to system fonts:

<strong>caption</strong>  
The font used for captioned controls (e.g., buttons, drop-downs, etc.).

<strong>icon</strong>  
The font used to label icons.

<strong>menu</strong>  
The font used in menus (e.g., dropdown menus and menu lists).

<strong>message-box</strong>  
The font used in dialog boxes.

<strong>small-caption</strong>  
The font used for labeling small controls.

<strong>status-bar</strong>  
The font used in window status bars.

System fonts may only be set as a whole; that is, the font family, size, weight, style, etc. are all set at the same time. These values may then be altered individually if desired. If no font with the indicated characteristics exists on a given platform, the user agent should either intelligently substitute (e.g., a smaller version of the ‘`caption`’ font might be used for the ‘`small-caption`’ font), or substitute a user agent default font. As for regular fonts, if, for a system font, any of the individual properties are not part of the operating system's available user preferences, those properties should be set to their initial values.

That is why this property is "almost" a shorthand property: system fonts can only be specified with this property, not with [‘`font-family`’](#propdef-font-family) itself, so [‘`font`’](#propdef-font) allows authors to do more than the sum of its subproperties. However, the individual properties such as [‘`font-weight`’](#propdef-font-weight) are still given values taken from the system font, which can be independently varied.

Note that the keywords used for the system fonts listed above are only treated as keywords when they occur in the initial position, in other positions the same string is treated as part of the font family name:

```text
  font: menu;        /* use the font settings for system menus */
  font: large menu;  /* use a font family named "menu" */
```
> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> button { font: 300 italic 1.3em/1.7em "FB Armada", sans-serif }
> button p { font: menu }
> button p em { font-weight: bolder }
> ```
>
> If the font used for dropdown menus on a particular system happened to be, for example, 9-point Charcoal, with a weight of 600, then P elements that were descendants of BUTTON would be displayed as if this rule were in effect:
>
> ```text
> button p { font: 600 9pt Charcoal }
> ```
>
> Because the [‘`font`’](#propdef-font) shorthand resets to its initial value any property not explicitly given a value, this has the same effect as this declaration:
>
> ```text
> button p {
>   font-style: normal;
>   font-variant: normal;
>   font-weight: 600;
>   font-size: 9pt;
>   line-height: normal;
>   font-family: Charcoal
> }
> ```
### <a id="font-synthesis-prop"></a>3.8. Controlling synthetic faces: the [font-synthesis](#propdef-font-synthesis) property

|                 |                                   |
|-----------------|-----------------------------------|
| Name:           | <a id="propdef-font-synthesis"></a>font-synthesis |
| Value:          | none \| \[ weight \|\| style \]   |
| Initial:        | weight style                      |
| Applies to:     | all elements                      |
| Inherited:      | yes                               |
| Percentages:    | N/A                               |
| Media:          | visual                            |
| Computed value: | as specified                      |
| Animatable:     | no                                |

This property controls whether user agents are allowed to synthesize bold or oblique font faces when a font family lacks bold or italic faces. If ‘[`weight`](#weight)’ is not specified, user agents must not synthesize bold faces and if ‘`style`’ is not specified user agents must not synthesize italic faces. A value of ‘`none`’ disallows all synthetic faces.

> <strong data-conversion-semantic="example">Example</strong>
>
> The style rule below disables the use of synthetically obliqued Arabic:
>
> ```text
> *:lang(ar) { font-synthesis: none; }
> ```
## <a id="font-resources"></a>4. Font Resources

### <a id="font-face-rule"></a>4.1. The <a id="at-font-face-rule"></a>`@font-face` rule

The [`@font-face`](#at-font-face-rule) rule allows for linking to fonts that are automatically fetched and activated when needed. This allows authors to select a font that closely matches the design goals for a given page rather than limiting the font choice to a set of fonts available on a given platform. A set of font descriptors define the location of a font resource, either locally or externally, along with the style characteristics of an individual face. Multiple [`@font-face`](#at-font-face-rule) rules can be used to construct font families with a variety of faces. Using CSS font matching rules, a user agent can selectively download only those faces that are needed for a given piece of text.

The [`@font-face`](#at-font-face-rule) rule consists of the [`@font-face`](#at-font-face-rule) at-keyword followed by a block of descriptor declarations. In terms of the grammar, this specification defines the following productions:

<a id="font_face_rule"></a>

<a id="descriptor_declaration"></a>

```text
font_face_rule
  : FONT_FACE_SYM S* '{' S* descriptor_declaration? [ ';' S* descriptor_declaration? ]* '}' S*
  ;

descriptor_declaration
  : property ':' S* expr
  ;
```
The following new definitions are introduced:

```text
-    -|\\0{0,4}2d(\r\n|[ \t\r\n\f])?
F    f|\\0{0,4}(46|66)(\r\n|[ \t\r\n\f])?
```
The following new token is introduced:

<a id="font_face_sym"></a>

```text
@{F}{O}{N}{T}{-}{F}{A}{C}{E} {return FONT_FACE_SYM;}
```
Each [`@font-face`](#at-font-face-rule) rule specifies a value for every font descriptor, either implicitly or explicitly. Those not given explicit values in the rule take the initial value listed with each descriptor in this specification. These descriptors apply solely within the context of the [`@font-face`](#at-font-face-rule) rule in which they are defined, and do not apply to document language elements. There is no notion of which elements the descriptors apply to or whether the values are inherited by child elements. When a given descriptor occurs multiple times in a given [`@font-face`](#at-font-face-rule) rule, only the last descriptor declaration is used and all prior declarations for that descriptor are ignored.

> <strong data-conversion-semantic="example">Example</strong>
>
> To use a downloadable font called Gentium:
>
> ```text
> 
> @font-face {
>   font-family: Gentium;
>   src: url(http://example.com/fonts/Gentium.woff);
> }
> 
> p { font-family: Gentium, serif; }
> ```
>
> The user agent will download Gentium and use it when rendering text within paragraph elements. If for some reason the site serving the font is unavailable, the default serif font will be used.

A given set of [`@font-face`](#at-font-face-rule) rules define a set of fonts available for use within the documents that contain these rules. When font matching is done, fonts defined using these rules are considered before other available fonts on a system.

Downloaded fonts are only available to documents that reference them. The process of activating these fonts must not make them available to other applications or to documents that don't directly link to the same font. User agent implementers might consider it convenient to use downloaded fonts when rendering characters in other documents for which no other available font exists as part of the [<em>system font fallback</em>](#system-font-fallback) procedure. However, this would cause a security leak since the contents of one page would be able to affect other pages, something an attacker could use as an attack vector. These restrictions do not affect caching behavior, fonts are cached the same way other web resources are cached.

This at-rule follows the forward-compatible parsing rules of CSS. Like properties in a declaration block, declarations of any descriptors that are not supported by the user agent must be ignored. [`@font-face`](#at-font-face-rule) rules require a font-family and src descriptor; if either of these are missing, the [`@font-face`](#at-font-face-rule) rule must not be considered when performing the [font matching algorithm](#font-matching-algorithm).

In cases where user agents have limited platform resources or implement the ability to disable downloadable font resources, [`@font-face`](#at-font-face-rule) rules must simply be ignored; the behavior of individual descriptors as defined in this specification should not be altered.

### <a id="font-family-desc"></a>4.2. Font family: the [font-family](#descdef-font-family) descriptor

|                     |                                           |
|---------------------|-------------------------------------------|
| Name:               | <a id="descdef-font-family"></a>font-family            |
| Value:              | [<var>&lt;family-name&gt;</var>](#family-name-value) |
| <em>Initial:</em> | N/A                                       |

This descriptor defines the font family name that will be used in all CSS font family name matching. It is required for the [`@font-face`](#at-font-face-rule) rule to be valid. It overrides the font family names contained in the underlying font data. If the font family name is the same as a font family available in a given user's environment, it effectively hides the underlying font for documents that use the stylesheet. This permits a web author to freely choose font-family names without worrying about conflicts with font family names present in a given user's environment. Likewise, platform substitutions for a given font family name must not be used.

### <a id="src-desc"></a>4.3. Font reference: the [src](#descdef-src) descriptor

|                     |                                                                                             |
|---------------------|---------------------------------------------------------------------------------------------|
| Name:               | <a id="descdef-src"></a>src                                                                      |
| Value:              | \[ \<url\> \[format(\<string\> \#)\]? \| [<var>&lt;font-face-name&gt;</var>](#font-face-name-value) \] \# |
| <em>Initial:</em> | N/A                                                                                         |

This descriptor specifies the resource containing font data. It is required for the [`@font-face`](#at-font-face-rule) rule to be valid. Its value is a prioritized, comma-separated list of external references or locally-installed font face names. When a font is needed the user agent iterates over the set of references listed, using the first one it can successfully activate. Fonts containing invalid data or local font faces that are not found are ignored and the user agent loads the next font in the list.

As with other URLs in CSS, the URL may be relative, in which case it is resolved relative to the location of the style sheet containing the [`@font-face`](#at-font-face-rule) rule. In the case of SVG fonts, the URL points to an element within a document containing SVG font definitions. If the element reference is omitted, a reference to the first defined font is implied. Similarly, font container formats that can contain more than one font must load one and only one of the fonts for a given [`@font-face`](#at-font-face-rule) rule. Fragment identifiers are used to indicate which font to load; these use the PostScript name of the font as defined in [\[RFC8081\]](#ref-RFC8081). Conformant user agents must skip downloading a font resource if the fragment identifier is unknown or unsupported. For example, older user agents which do not support OpenType collections will skip to the next url in the list.

```text

src: url(fonts/simple.woff);       /* load simple.woff relative to stylesheet location */
src: url(/fonts/simple.woff);      /* load simple.woff from absolute location */
src: url(fonts/coll.otc#foo);      /* load font foo from collection coll.otc
src: url(fonts/coll.woff2#foo);    /* load font foo from woff2 collection coll.woff2
src: url(fonts.svg#simple);        /* load SVG font with id 'simple' */
```
External references consist of a URL, followed by an optional hint describing the format of the font resource referenced by that URL. The format hint contains a comma-separated list of format strings that denote well-known font formats. Conformant user agents must skip downloading a font resource if the format hints indicate only unsupported or unknown font formats. If no format hints are supplied, the user agent should download the font resource.

```text

/* load WOFF2 font if possible, otherwise WOFF, else use OpenType font */
@font-face {
  font-family: bodytext;
  src: url(ideal-sans-serif.woff2) format("woff2"),
       url(good-sans-serif.woff) format("woff"),
       url(basic-sans-serif.ttf) format("opentype");
}
```
Format strings defined by this specification:

<a id="fontformats"></a>

| String            | Font Format                                                                | Common extensions |
|-------------------|----------------------------------------------------------------------------|-------------------|
| `"woff"` | [WOFF 1.0 (Web Open Font Format)](https://www.w3.org/TR/WOFF/)             | .woff             |
| `"woff2"` | [WOFF 2.0 (Web Open Font Format)](https://www.w3.org/TR/WOFF2/)            | .woff2            |
| `"truetype"` | [TrueType](https://docs.microsoft.com/en-us/typography/opentype/spec/)     | .ttf              |
| `"opentype"` | [OpenType](https://docs.microsoft.com/en-us/typography/opentype/spec/)     | .ttf, .otf        |
| `"embedded-opentype"` | [Embedded OpenType](https://www.w3.org/Submission/2008/SUBM-EOT-20080305/) | .eot              |
| `"svg"` | [SVG Font](https://www.w3.org/TR/SVG11/fonts.html)                         | .svg, .svgz       |

Given the overlap in common usage between TrueType and OpenType [\[OPENTYPE\]](#ref-OPENTYPE), the format hints `"truetype"` and `"opentype"` must be considered as synonymous; a format hint of `"opentype"` does not imply that the font contains Postscript CFF style glyph data or that it contains OpenType layout information (see [Appendix A](#platform-props-to-css) for more background on this).

When authors would prefer to use a locally available copy of a given font and download it if it's not, `local()` can be used. The locally-installed <a id="font-face-name-value"></a><var>&lt;font-face-name&gt;</var> argument to `local()` is a format-specific string that uniquely identifies a single font face within a larger family. The syntax for a [<var>&lt;font-face-name&gt;</var>](#font-face-name-value) is a unique font face name enclosed by `"local("` and `")"`. The name can optionally be enclosed in quotes. If unquoted, the unquoted font family name processing conventions apply; the name must be a sequence of identifiers separated by [whitespace](https://www.w3.org/TR/css3-syntax/#whitespace) which is converted to a string by joining the identifiers together separated by a single space.

```text

/* regular face of Gentium */
@font-face {
  font-family: MyGentium;
  src: local(Gentium),    /* use locally available Gentium */
       url(Gentium.woff); /* otherwise, download it */
}
```
For OpenType and TrueType fonts, this string is used to match only the Postscript name or the full font name in the name table of locally available fonts. Which type of name is used varies by platform and font, so authors should include both of these names to assure proper matching across platforms. Platform substitutions for a given font name must not be used.

```text

/* bold face of Gentium */
@font-face {
  font-family: MyGentium;
  src: local(Gentium Bold),    /* full font name */
       local(Gentium-Bold),    /* Postscript name */
       url(GentiumBold.woff);  /* otherwise, download it */
  font-weight: bold;
}
```
Just as a [`@font-face`](#at-font-face-rule) rule specifies the characteristics of a single font within a family, the unique name used with `local()` specifies a single font, not an entire font family. Defined in terms of OpenType font data, the Postscript name is found in the font's [name table](https://docs.microsoft.com/en-us/typography/opentype/spec/name), in the name record with `nameID = 6` (see [\[OPENTYPE\]](#ref-OPENTYPE) for more details). The Postscript name is the commonly used key for all fonts on OSX and for Postscript CFF fonts under Windows. The full font name (`nameID = 4`) is used as a unique key for fonts with TrueType glyphs on Windows.

For OpenType fonts with multiple localizations of the full font name, the US English version is used (`language ID = 0x409` for Windows and `language ID = 0` for Macintosh) or the first localization when a US English full font name is not available (the OpenType specification recommends that [all fonts minimally include US English names](https://docs.microsoft.com/en-us/typography/opentype/spec/recom)). User agents that also match other full font names, e.g. matching the Dutch name when the current system locale is set to Dutch, are considered non-conformant. This is done not to prefer English but to avoid matching inconsistencies across font versions and OS localizations, since font style names (e.g. "Bold") are frequently localized into many languages and the set of localizations available varies widely across platform and font version. User agents that match a concatenation of family name (`nameID = 1`) with style name (`nameID = 2`) are considered non-conformant.

This also allows for referencing faces that belong to larger families that cannot otherwise be referenced.

> <strong data-conversion-semantic="example">Example</strong>
>
> Use a local font or reference an SVG font in another document:
>
> ```text
> 
> @font-face {
>   font-family: Headline;
>   src: local(Futura-Medium),
>        url(fonts.svg#MyGeometricModern) format("svg");
> }
> ```
>
> Create an alias for local Japanese fonts on different platforms:
>
> ```text
> 
> @font-face {
>   font-family: jpgothic;
>   src: local(HiraKakuPro-W3), local(Meiryo), local(IPAPGothic);
> }
> ```
>
> Reference a font face that cannot be matched within a larger family:
>
> ```text
> 
> @font-face {
>   font-family: Hoefler Text Ornaments;
>   /* has the same font properties as Hoefler Text Regular */
>   src: local(HoeflerText-Ornaments);
> }
> ```
>
> Since localized fullnames never match, a document with the header style rules below would always render using the default serif font, regardless whether a particular system locale parameter is set to Finnish or not:
>
> ```text
> 
> @font-face {
>   font-family: SectionHeader;
>   src: local("Arial Lihavoitu");  /* Finnish fullname for Arial Bold, should fail */
>   font-weight: bold;
> }
> 
> h2 { font-family: SectionHeader, serif; }
> ```
>
> A conformant user agent would never load the font ‘`gentium.eot`’ in the example below, since it is included in the first definition of the ‘[`src`](#descdef-src)’ descriptor which is overridden by the second definition in the same [`@font-face`](#at-font-face-rule) rule:
>
> ```text
> 
> @font-face {
>   font-family: MainText;
>   src: url(gentium.eot);                     /* for use with older user agents */
>   src: local("Gentium"), url(gentium.woff);  /* Overrides src definition */
> }
> ```
### <a id="font-prop-desc"></a>4.4. Font property descriptors: the [font-style](#descdef-font-style), [font-weight](#descdef-font-weight), [font-stretch](#descdef-font-stretch) descriptors

|                     |                               |
|---------------------|-------------------------------|
| Name:               | <a id="descdef-font-style"></a>font-style |
| Value:              | normal \| italic \| oblique   |
| <em>Initial:</em> | normal                        |

|                     |                                                                               |
|---------------------|-------------------------------------------------------------------------------|
| Name:               | <a id="descdef-font-weight"></a>font-weight                                                |
| Value:              | normal \| bold \| 100 \| 200 \| 300 \| 400 \| 500 \| 600 \| 700 \| 800 \| 900 |
| <em>Initial:</em> | normal                                                                        |

|                     |                                                                                                                                              |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| Name:               | <a id="descdef-font-stretch"></a>font-stretch                                                                                                              |
| Value:              | normal \| ultra-condensed \| extra-condensed \| condensed \| semi-condensed \| semi-expanded \| expanded \| extra-expanded \| ultra-expanded |
| <em>Initial:</em> | normal                                                                                                                                       |

These descriptors define the characteristics of a font face and are used in the process of matching styles to specific faces. For a font family defined with several [`@font-face`](#at-font-face-rule) rules, user agents can either download all faces in the family or use these descriptors to selectively download font faces that match actual styles used in document. The values for these descriptors are the same as those for the corresponding font properties except that relative keywords are not allowed, ‘[`bolder`](#bolder)’ and ‘[`lighter`](#lighter)’. If these descriptors are omitted, initial values are assumed.

The value for these font face style attributes is used in place of the style implied by the underlying font data. This allows authors to combine faces in flexible combinations, even in situations where the original font data was arranged differently. User agents that implement synthetic bolding and obliquing must only apply synthetic styling in cases where the font descriptors imply this is needed, rather than based on the style attributes implied by the font data.

> <strong data-conversion-semantic="example">Example</strong>
>
> The font descriptors defined in this section are used for selecting a font from within the set of fonts defined by [`@font-face`](#at-font-face-rule) rules for a given family.
>
> Consider a family containing a single, regular face:
>
> ```text
> 
> @font-face {
>   font-family: BaskervilleSimple;
>   src: url(baskerville-regular.woff);
> }
> ```
>
> Unstyled text would display using the regular face defined in the [`@font-face`](#at-font-face-rule) rule:
>
> ![regular face display](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/fiddlesticks-regular.png)
>
> However, italic text would display in most user agents using synthetically obliqued glyphs from the regular face, since a separate italic face is not defined:
>
> ![synthetic italics display](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/fiddlesticks-synitalics.png)
>
> Now consider a family for which an actual italic face is defined:
>
> ```text
> 
> @font-face {
>   font-family: BaskervilleFull;
>   src: url(baskerville-regular.woff);
> }
> 
> @font-face {
>   font-family: BaskervilleFull;
>   src: url(baskerville-italic.woff);
>   font-style: italic;
> }
> ```
>
> The second [`@font-face`](#at-font-face-rule) rule defines the font resource `baskerville-italic.woff` to have style attributes of normal weight, normal stretch and italic style. When displaying italic text, the user agent will use this font, since it's the closest match for italic text. Thus, the text will display using glyphs designed by a type designer rather than using synthetically obliqued glyphs from the regular face:
>
> ![real italics display](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/fiddlesticks-italics.png)
>
> See the section on [font matching](#font-style-matching) for more complete details of the process used to select a particular face within a font family.

### <a id="unicode-range-desc"></a>4.5. Character range: the [unicode-range](#descdef-unicode-range) descriptor

|                     |                                         |
|---------------------|-----------------------------------------|
| Name:               | <a id="descdef-unicode-range"></a>unicode-range        |
| Value:              | [<var>&lt;urange&gt;</var>](#urange-value) \# |
| <em>Initial:</em> | U+0-10FFFF                              |

This descriptor defines the set of Unicode [\[UNICODE\]](#ref-UNICODE) codepoints that may be supported by the font face for which it is declared. The descriptor value is a comma-delimited list of Unicode range ([<var>&lt;urange&gt;</var>](#urange-value)) values. The union of these ranges defines the set of codepoints that serves as a hint for user agents when deciding whether or not to download a font resource for a given text run.

Each <a id="urange-value"></a><var>&lt;urange&gt;</var> value is a [`UNICODE-RANGE`](https://www.w3.org/TR/CSS21/syndata.html#tokenization) token made up of a "U+" or "u+" prefix followed by a codepoint range in one of the three forms listed below. Ranges that do not fit one of the these forms are invalid and cause the declaration to be ignored.

single codepoint (e.g. U+416)  
a Unicode codepoint, represented as one to six hexadecimal digits

interval range (e.g. U+400-4ff)  
represented as two hyphen-separated Unicode codepoints indicating the inclusive start and end codepoints of a range

wildcard range (e.g. U+4??)  
defined by the set of codepoints implied when trailing ‘`?`’ characters signify any hexadeximal digit

Individual codepoints are written using hexadecimal values that correspond to [Unicode character codepoints](https://www.unicode.org/charts/) [\[UNICODE\]](#ref-UNICODE). Unicode codepoint values must be between 0 and 10FFFF inclusive. Digit values of codepoints are ASCII case-insensitive. For interval ranges, the start and end codepoints must be within the range noted above and the end codepoint must be greater than or equal to the start codepoint.

Wildcard ranges specified with ‘?’ that lack an initial digit (e.g. "U+???") are valid and equivalent to a wildcard range with an initial zero digit (e.g. "U+0???" = "U+0000-0FFF"). Wildcard ranges that extend beyond the range of Unicode codepoints are invalid. Because of this, the maximum number of trailing ‘`?`’ wildcard characters is five, even though the [`UNICODE-RANGE`](https://www.w3.org/TR/CSS21/syndata.html#tokenization) token accepts six.

Within the comma-delimited list of Unicode ranges in a ‘[`unicode-range`](#descdef-unicode-range)’ descriptor declaration, ranges may overlap. The union of these ranges defines the set of codepoints for which the corresponding font may be used. User agents must not download or use the font for codepoints outside this set. User agents may normalize the list of ranges into a list that is different but represents the same set of codepoints.

The associated font might not contain glyphs for the entire set of codepoints defined by the ‘[`unicode-range`](#descdef-unicode-range)’ descriptor. When the font is used, the <a id="effective-character-map"></a>effective character map is the intersection of the codepoints defined by ‘[`unicode-range`](#descdef-unicode-range)’ with the font's [<em>character map</em>](#character-map). This allows authors to define supported ranges in terms of broad ranges without worrying about the precise codepoint ranges supported by the underlying font.

### <a id="composite-fonts"></a>4.6. Using character ranges to define composite fonts

Multiple [`@font-face`](#at-font-face-rule) rules with different unicode ranges for the same family and style descriptor values can be used to create composite fonts that mix the glyphs from different fonts for different scripts. This can be used to combine fonts that only contain glyphs for a single script (e.g. Latin, Greek, Cyrillic) or it can be used by authors as a way of segmenting a font into fonts for commonly used characters and less frequently used characters. Since the user agent will only pull down the fonts it needs this helps reduce page bandwidth.

If the unicode ranges overlap for a set of [`@font-face`](#at-font-face-rule) rules with the same family and style descriptor values, the rules are ordered in the reverse order they were defined; the last rule defined is the first to be checked for a given character.

Example ranges for specific languages or characters:

unicode-range: U+A5;  
a single code point, the yen/yuan symbol

unicode-range: U+0-7F;  
code range for basic ASCII characters

unicode-range: U+590-5ff;  
code range for Hebrew characters

unicode-range: U+A5, U+4E00-9FFF, U+30??, U+FF00-FF9F;  
code range for Japanese kanji, hiragana and katakana characters plus yen/yuan symbol

> <strong data-conversion-semantic="example">Example</strong>
>
> The BBC provides news services in a wide variety of languages, many that are not well supported across all platforms. Using an [`@font-face`](#at-font-face-rule) rule, the BBC could provide a font for any of these languages, as it already does via a manual font download.
>
> ```text
> 
> @font-face {
>   font-family: BBCBengali;
>   src: url(fonts/BBCBengali.woff) format("woff");
>   unicode-range: U+00-FF, U+980-9FF;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> Technical documents often require a wide range of symbols. The STIX Fonts project is one project aimed at providing fonts to support a wide range of technical typesetting in a standardized way. The example below shows the use of a font that provides glyphs for many of the mathematical and technical symbol ranges within Unicode:
>
> ```text
> 
> @font-face {
>   font-family: STIXGeneral;
>   src: local(STIXGeneral), url(/stixfonts/STIXGeneral.otf);
>   unicode-range: U+000-49F, U+2000-27FF, U+2900-2BFF, U+1D400-1D7FF;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> This example shows how an author can override the glyphs used for Latin characters in a Japanese font with glyphs from a different font. The first rule specifies no range so it defaults to the entire range. The range specified in the second rule overlaps but takes precedence because it is defined later.
>
> ```text
> 
> @font-face {
>   font-family: JapaneseWithGentium;
>   src: local(MSMincho);
>   /* no range specified, defaults to entire range */
> }
> 
> @font-face {
>   font-family: JapaneseWithGentium;
>   src: url(../fonts/Gentium.woff);
>   unicode-range: U+0-2FF;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> Consider a family constructed to optimize bandwidth by separating out Latin, Japanese and other characters into different font files:
>
> ```text
> 
> /* fallback font - size: 4.5MB */
> @font-face {
>   font-family: DroidSans;
>   src: url(DroidSansFallback.woff);
>   /* no range specified, defaults to entire range */
> }
> 
> /* Japanese glyphs - size: 1.2MB */
> @font-face {
>   font-family: DroidSans;
>   src: url(DroidSansJapanese.woff);
>   unicode-range: U+3000-9FFF, U+ff??;
> }
> 
> /* Latin, Greek, Cyrillic along with some
>    punctuation and symbols - size: 190KB */
> @font-face {
>   font-family: DroidSans;
>   src: url(DroidSans.woff);
>   unicode-range: U+000-5FF, U+1e00-1fff, U+2000-2300;
> }
> ```
>
> For simple Latin text, only the font for Latin characters is downloaded:
>
> ```text
> 
> body { font-family: DroidSans; }
> 
> <p>This is that</p>
> ```
>
> In this case the user agent first checks the unicode-range for the font containing Latin characters (DroidSans.woff). Since all the characters above are in the range U+0-5FF, the user agent downloads the font and renders the text with that font.
>
> Next, consider text that makes use of an arrow character (⇨):
>
> ```text
> 
> <p>This &#x21e8; that<p>
> ```
>
> The user agent again first checks the unicode-range of the font containing Latin characters. Since U+2000-2300 includes the arrow code point (U+21E8), the user agent downloads the font. For this character however the Latin font does not have a matching glyph, so the effective unicode-range used for font matching excludes this code point. Next, the user agent evaluates the Japanese font. The unicode-range for the Japanese font, U+3000-9FFF and U+ff??, does not include U+21E8, so the user agent does not download the Japanese font. Next the fallback font is considered. The [`@font-face`](#at-font-face-rule) rule for the fallback font does not define unicode-range so its value defaults to the range of all Unicode code points. The fallback font is downloaded and used to render the arrow character.

### <a id="font-rend-desc"></a>4.7. Font features: the [font-feature-settings](#descdef-font-feature-settings) descriptor

|                     |                                                                                               |
|---------------------|-----------------------------------------------------------------------------------------------|
| Name:               | <a id="descdef-font-feature-settings"></a>font-feature-settings                                                      |
| Value:              | [normal](#font-feature-settings-normal-value) \| [<var>&lt;feature-tag-value&gt;</var>](#feature-tag-value) \# |
| <em>Initial:</em> | normal                                                                                        |

This descriptor defines initial settings that apply when the font defined by an [`@font-face`](#at-font-face-rule) rule is rendered. It does not affect font selection. Values are identical to those defined for the corresponding [‘`font-feature-settings`’](#propdef-font-feature-settings) property defined below except that the value ‘`inherit`’ is omitted. When multiple font feature descriptors or properties are used, the cumulative effect on text rendering is detailed in the section [Font Feature Resolution](#font-feature-resolution) below.

### <a id="font-face-loading"></a>4.8. Font loading guidelines

The [`@font-face`](#at-font-face-rule) rule is designed to allow lazy loading of font resources that are only downloaded when used within a document. A stylesheet can include [`@font-face`](#at-font-face-rule) rules for a library of fonts of which only a select set are used; user agents must only download those fonts that are referred to within the style rules applicable to a given page. User agents that download all fonts defined in [`@font-face`](#at-font-face-rule) rules without considering whether those fonts are in fact used within a page are considered non-conformant. In cases where a font might be downloaded in character fallback cases, user agents may download a font if it's contained within the computed value of [‘`font-family`’](#propdef-font-family) for a given text run.

```text

@font-face {
  font-family: GeometricModern;
  src: url(font.woff);
}

p {
  /* font will be downloaded for pages with p elements */
  font-family: GeometricModern, sans-serif;
}

h2 {
  /* font may be downloaded for pages with h2 elements, even if Futura is available locally */
  font-family: Futura, GeometricModern, sans-serif;
}
```
In cases where textual content is loaded before downloadable fonts are available, user agents may render text as it would be rendered if downloadable font resources are not available or they may render text transparently with fallback fonts to avoid a flash of text using a fallback font. In cases where the font download fails user agents must display text, simply leaving transparent text is considered non-conformant behavior. Authors are advised to use fallback fonts in their font lists that closely match the metrics of the downloadable fonts to avoid large page reflows where possible.

<a id="same-origin-restriction"></a>

<a id="allowing-cross-origin-font-loading"></a>

### <a id="font-fetching-requirements"></a>4.9. Font fetching requirements

For font loads, user agents must use the [potentially CORS-enabled fetch method](https://fetch.spec.whatwg.org/#http-cors-protocol) defined by the [\[FETCH\]](#ref-FETCH) specification for URL's defined within @font-face rules. When fetching, user agents must use "Anonymous" mode, set the referrer source to the stylesheet's URL and set the origin to the URL of the containing document.

> <strong data-conversion-semantic="note">Note</strong>
>
> The implications of this for authors are that fonts will typically not be loaded cross-origin unless authors specifically takes steps to permit cross-origin loads. Sites can explicitly allow cross-site loading of font data using the `Access-Control-Allow-Origin` HTTP header. For other schemes, no explicit mechanism to allow cross-origin loading, beyond what is permitted by the potentially CORS-enabled fetch method, is defined or required.

> <strong data-conversion-semantic="example">Example</strong>
>
> For the examples given below, assume that a document is located at `https://example.com/page.html` and all URL's link to valid font resources supported by the user agent. Fonts defined with the ‘[`src`](#descdef-src)’ descriptor values below will be loaded:
>
> ```text
> /* same origin (i.e. domain, scheme, port match document) */
> src: url(fonts/simple.woff);
> 
> /* data url's with no redirects are treated as same origin */
> src: url("data:application/font-woff;base64,...");
> 
> /* cross origin, different domain */
> /* Access-Control-Allow-Origin response header set to '*' */
> src: url(http://another.example.com/fonts/simple.woff);
> ```
>
> Fonts defined with the ‘[`src`](#descdef-src)’ descriptor values below will fail to load:
>
> ```text
> /* cross origin, different scheme */
> /* no Access-Control-xxx headers in response */
> src: url(https://example.com/fonts/simple.woff);
> 
> /* cross origin, different domain */
> /* no Access-Control-xxx headers in response */
> src: url(http://another.example.com/fonts/simple.woff);
> ```
## <a id="font-matching-algorithm"></a>5. Font Matching Algorithm

The algorithm below describes how fonts are associated with individual runs of text. For each character in the run a font family is chosen and a particular font face is selected containing a glyph for that character.

### <a id="font-family-casing"></a>5.1. Case sensitivity of font family names

As part of the font matching algorithm outlined below, user agents must match font family names used in style rules with actual font family names contained in fonts available in a given environment or with font family names defined in [`@font-face`](#at-font-face-rule) rules. User agents must match these names case insensitively, using the "Default Caseless Matching" algorithm outlined in the Unicode specification [\[UNICODE\]](#ref-UNICODE). This algorithm is detailed in section 3.13 entitled "Default Case Algorithms". Specifically, the algorithm must be applied without normalizing the strings involved and without applying any language-specific tailorings. The case folding method specified by this algorithm uses the case mappings with status field ‘`C`’ or ‘`F`’ in the CaseFolding.txt file of the Unicode Character Database [\[UNICODE\]](#ref-UNICODE).

> <strong data-conversion-semantic="note">Note</strong>
>
> For authors this means that font family names are matched case insensitively, whether those names exist in a platform font or in the [`@font-face`](#at-font-face-rule) rules contained in a stylesheet. Authors should take care to ensure that names use a character sequence consistent with the actual font family name, particularly when using combining characters such as diacritical marks. For example, a family name that contains a lowercase a (U+0061) followed by a combining ring (U+030A) will <strong>not</strong> match a name that looks identical but which uses the precomposed lowercase a-ring character (U+00E5) instead of the combining sequence.

> <strong data-conversion-semantic="note">Note</strong>
>
> Implementors should take care to verify that a given caseless string comparison implementation uses this precise algorithm and not assume that a given platform string matching routine follows it, as many of these have locale-specific behavior or use some level of string normalization [\[UAX15\]](#ref-UAX15).

### <a id="font-style-matching"></a>5.2. Matching font styles

The procedure for choosing a font for a given character in a run of text consists of iterating over the font families named by the [‘`font-family`’](#propdef-font-family) property, selecting a font face with the appropriate style based on other font properties and then determining whether a glyph exists for the given character. This is done using the <a id="character-map"></a>character map of the font, data which maps characters to the default glyph for that character. A font is considered to <a id="support"></a>support a given character if (1) the character is contained in the font's [<em>character map</em>](#character-map) and (2) if required by the containing script, shaping information is available for that character.

Some legacy fonts may include a given character in the [<em>character map</em>](#character-map) but lack the shaping information (e.g. [OpenType layout tables](https://www.microsoft.com/typography/otspec/ttochap1.htm) or [Graphite tables](https://scripts.sil.org/cms/scripts/page.php?site_id=projects&item_id=graphite_techAbout)) necessary for correctly rendering text runs containing that character.

Codepoint sequences consisting of a base character followed by a sequence of combining characters are treated slightly differently, see the section on [cluster matching](#cluster-matching) below.

For this procedure, the <a id="default-face"></a>default face for a given font family is defined to be the face that would be selected if all font style properties were set to their initial value.

<a id="fontmatchingalg"></a>

1.  Using the computed font property values for a given element, the user agent starts with the first family name specified by the [‘`font-family`’](#propdef-font-family) property.

2.  If the family name is a generic family keyword, the user agent looks up the appropriate font family name to be used. User agents may choose the generic font family to use based on the language of the containing element or the Unicode range of the character.

3.  For other family names, the user agent attempts to find the family name among fonts defined via [`@font-face`](#at-font-face-rule) rules and then among available system fonts, matching names with a [case-insensitive comparison](#font-family-casing) as outlined in the section above. On systems containing fonts with multiple localized font family names, user agents must match any of these names independent of the underlying system locale or platform API used. If the font resources defined for a given face in an [`@font-face`](#at-font-face-rule) rule are either not available or contain invalid font data, then the face should be treated as not present in the family. If no faces are present for a family defined via [`@font-face`](#at-font-face-rule) rules, the family should be treated as missing; matching a platform font with the same name must not occur in this case.

4.  If a font family match occurs, the user agent assembles the set of font faces in that family and then narrows the set to a single face using other font properties in the order given below. A group of faces defined via [`@font-face`](#at-font-face-rule) rules with identical font descriptor values but differing ‘[`unicode-range`](#descdef-unicode-range)’ values are considered to be a single <a id="composite-face"></a>composite face for this step:

    <a id="fontstylematchingalg"></a>

    1.  [‘`font-stretch`’](#propdef-font-stretch) is tried first. If the matching set contains faces with width values matching the [‘`font-stretch`’](#propdef-font-stretch) value, faces with other width values are removed from the matching set. If there is no face that exactly matches the width value the nearest width is used instead. If the value of [‘`font-stretch`’](#propdef-font-stretch) is [‘`normal`’](#font-stretch-normal-value) or one of the condensed values, narrower width values are checked first, then wider values. If the value of [‘`font-stretch`’](#propdef-font-stretch) is one of the expanded values, wider values are checked first, followed by narrower values. Once the closest matching width has been determined by this process, faces with other widths are removed from the matching set.
    2.  [‘`font-style`’](#propdef-font-style) is tried next. If the value of [‘`font-style`’](#propdef-font-style) is ‘[`italic`](#italic)’, italic faces are checked first, then oblique, then normal faces. If the value is ‘[`oblique`](#oblique)’, oblique faces are checked first, then italic faces and then normal faces. If the value is [‘`normal`’](#font-style-normal-value), normal faces are checked first, then oblique faces, then italic faces. Faces with other style values are excluded from the matching set. User agents are permitted to distinguish between italic and oblique faces within platform font families but this is not required, so all italic or oblique faces may be treated as italic faces. However, within font families defined via [`@font-face`](#at-font-face-rule) rules, italic and oblique faces must be distinguished using the value of the [‘`font-style`’](#descdef-font-style) descriptor. For families that lack any italic or oblique faces, user agents may create artificial oblique faces, if this is permitted by the value of the ‘[`font-synthesis`](#propdef-font-synthesis)’ property.
    3.  [‘`font-weight`’](#propdef-font-weight) is matched next, so it will always reduce the matching set to a single font face. If bolder/lighter relative weights are used, the effective weight is calculated based on the inherited weight value, as described in the definition of the [‘`font-weight`’](#propdef-font-weight) property. Given the desired weight and the weights of faces in the matching set after the steps above, if the desired weight is available that face matches. Otherwise, a weight is chosen using the rules below:
        - If the desired weight is less than 400, weights below the desired weight are checked in descending order followed by weights above the desired weight in ascending order until a match is found.
        - If the desired weight is greater than 500, weights above the desired weight are checked in ascending order followed by weights below the desired weight in descending order until a match is found.
        - If the desired weight is 400, 500 is checked first and then the rule for desired weights less than 400 is used.
        - If the desired weight is 500, 400 is checked first and then the rule for desired weights less than 400 is used.
    4.  [‘`font-size`’](#propdef-font-size) must be matched within a UA-dependent margin of tolerance. (Typically, sizes for scalable fonts are rounded to the nearest whole pixel, while the tolerance for bitmapped fonts could be as large as 20%.) Further computations, e.g., by ‘`em`’ values in other properties, are based on the [‘`font-size`’](#propdef-font-size) value that is used, not the one that is specified.

5.  If the matched face is defined via [`@font-face`](#at-font-face-rule) rules, user agents must use the procedure below to select a single font:

    1.  If the font resource has not been loaded and the range of characters defined by the ‘[`unicode-range`](#descdef-unicode-range)’ descriptor value includes the character in question, load the font.
    2.  After downloading, if the [<em>effective character map</em>](#effective-character-map) supports the character in question, select that font.

    When the matched face is a [<em>composite
     face</em>](#composite-face), user agents must use the procedure above on each of the faces in the [<em>composite face</em>](#composite-face) in reverse order of [`@font-face`](#at-font-face-rule) rule definition.

    While the download occurs, user agents may either wait until the font is downloaded or render once with substituted font metrics and render again once the font is downloaded.

6.  If no matching face exists or the matched face does not contain a glyph for the character to be rendered, the next family name is selected and the previous three steps repeated. Glyphs from other faces in the family are not considered. The only exception is that user agents may optionally substitute a synthetically obliqued version of the [<em>default face</em>](#default-face) if that face supports a given glyph and synthesis of these faces is permitted by the value of the ‘[`font-synthesis`](#propdef-font-synthesis)’ property. For example, a synthetic italic version of the regular face may be used if the italic face doesn't support glyphs for Arabic.

7.  If there are no more font families to be evaluated and no matching face has been found, then the user agent performs a <a id="system-font-fallback"></a>system font fallback procedure to find the best match for the character to be rendered. The result of this procedure may vary across user agents.

8.  If a particular character cannot be displayed using any font, the user agent should indicate by some means that a character is not being displayed, displaying either a symbolic representation of the missing glyph (e.g. using a [Last Resort Font](https://en.wikipedia.org/wiki/Last_resort_font)) or using the missing character glyph from a default font.

Optimizations of this process are allowed provided that an implementation behaves as if the algorithm had been followed exactly. Matching occurs in a well-defined order to ensure that the results are as consistent as possible across user agents, given an identical set of available fonts and rendering technology.

The <a id="first-available-font"></a>first available font, used for example in the definition of [<em>font-relative
   lengths</em>](https://www.w3.org/TR/css3-values/#font-relative-lengths) such as ‘`ex`’ and ‘`ch`’ or in the definition of the [‘`line-height`’](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) property, is defined to be the first available font that would match the U+0020 (space) character given font families in the ‘`font-family`’ list (or a user agent's default font if none are available).

### <a id="cluster-matching"></a>5.3. Cluster matching

When text contains characters such as combining marks, ideally the base character should be rendered using the same font as the mark, this assures proper placement of the mark. For this reason, the font matching algorithm for clusters is more specialized than the general case of matching a single character by itself. For sequences containing variation selectors, which indicate the precise glyph to be used for a given character, user agents always attempt [<em>system font
   fallback</em>](#system-font-fallback) to find the appropriate glyph before using the default glyph of the base character.

A sequence of codepoints containing combining mark or other modifiers is termed a grapheme cluster (see [\[CSS-TEXT-3\]](#ref-CSS-TEXT-3) and [\[UAX29\]](#ref-UAX29) for a more complete description). For a given cluster containing a base character, <em>b</em> and a sequence of combining characters <em>c1, c2…</em>, the entire cluster is matched using these steps:

1.  For each family in the font list, a face is chosen using the style selection rules defined in the previous section.
    1.  If all characters in the sequence <em>b + c1 + c2 …</em> are completely supported by the font, select this font for the sequence.
    2.  If a sequence of multiple codepoints is canonically equivalent to a single character and the font [<em title="support">supports</em>](#support) that character, select this font for the sequence and use the glyph associated with the canonically equiavlent character for the entire cluster.
2.  If no font was found in the font list in step 1:
    1.  If <em>c1</em> is a variation selector, system fallback must be used to find a font that [<em title="support">supports</em>](#support) the full sequence of <em>b + c1</em>. If no font on the system [<em title="support">supports</em>](#support) the full sequence, match the single character <em>b</em> using the normal procedure for matching single characters and ignore the variation selector. Note: a sequence with more than one variation selector must be treated as an encoding error and the trailing selectors must be ignored. [\[UNICODE\]](#ref-UNICODE)
    2.  Otherwise, the user agent may optionally use system font fallback to match a font that [<em title="support">supports</em>](#support) the entire cluster.
3.  If no font is found in step 2, use the matching sequence from step 1 to determine the longest sequence that is completely [<em title="support">supported</em>](#support) by a font in the font list and attempt to match the remaining combining characters separately using the rules for single characters.

### <a id="char-handling-issues"></a>5.4. Character handling issues

CSS font matching is always performed on text runs containing Unicode characters [\[UNICODE\]](#ref-UNICODE), so documents using legacy encodings are assumed to have been transcoded before matching fonts. For fonts containing [<em title="character map">character maps</em>](#character-map) for both legacy encodings and Unicode, the contents of the legacy encoding [<em>character map</em>](#character-map) must have no effect on the results of the font matching process.

The font matching process does not assume that text runs are in either normalized or denormalized form (see [\[CHARMOD-NORM\]](#ref-CHARMOD-NORM) for more details). Fonts may only support precomposed forms and not the decomposed sequence of base character plus combining marks. Authors should always tailor their choice of fonts to their content, including whether that content contains normalized or denormalized character streams.

If a given character is a Private-Use Area Unicode codepoint, user agents must only match font families named in the ‘`font-family`’ list that are not generic families. If none of the families named in the ‘`font-family`’ list contain a glyph for that codepoint, user agents must display some form of missing glyph symbol for that character rather than attempting [<em>system font fallback</em>](#system-font-fallback) for that codepoint. When matching the replacement character U+FFFD, user agents may skip the font matching process and immediately display some form of missing glyph symbol, they are not required to display the glyph from the font that would be selected by the font matching process.

In general, the fonts for a given family will all have the same or similar [<em title="character map">character
   maps</em>](#character-map). The process outlined here is designed to handle even font families containing faces with widely variant [<em title="character map">character maps</em>](#character-map). However, authors are cautioned that the use of such families can lead to unexpected results.

### <a id="font-matching-changes"></a>5.5. Font matching changes since CSS 2.1

The algorithm above is different from CSS 2.1 in a number of key places. These changes were made to better reflect actual font matching behavior across user agent implementations.

Differences compared to the font matching algorithm in CSS 2.1:

- The algorithm includes font-stretch matching.
- All possible font-style matching scenarios are delineated.
- Small-caps fonts are not matched as part of the font matching process, they are now handled via font features.
- Unicode variation selector matching is required.
- Cluster sequences are matched as a unit.

### <a id="font-matching-examples"></a>5.6. Font matching examples

> <strong data-conversion-semantic="example">Example</strong>
>
> It's useful to note that the CSS selector syntax may be used to create language-sensitive typography. For example, some Chinese and Japanese characters are unified to have the same Unicode code point, although the abstract glyphs are not the same in the two languages.
>
> ```text
> *:lang(ja) { font: 900 14pt/16pt "Heisei Mincho W9", serif; }
> *:lang(zh-Hant-TW) { font: 800 14pt/16.5pt "Li Sung", serif; }
> ```
>
> This selects any element that has the given language — Japanese or Traditional Chinese as used in Taiwan — and uses the appropriate font.

## <a id="font-rend-props"></a>6. Font Feature Properties

Modern font technologies support a variety of advanced typographic and language-specific font features. Using these features, a single font can provide glyphs for a wide range of ligatures, contextual and stylistic alternates, tabular and old-style figures, small capitals, automatic fractions, swashes, and alternates specific to a given language. To allow authors control over these font capabilities, the ‘`font-variant`’ property has been expanded for CSS3. It now functions as a shorthand for a set of properties that provide control over stylistic font features.

### <a id="glyph-selection-positioning"></a>6.1. Glyph selection and positioning

This section is non-normative

Simple fonts used for displaying Latin text use a very basic processing model. Fonts contain a [<em>character map</em>](#character-map) which maps each character to a glyph for that character. Glyphs for subsequent characters are simply placed one after the other along a run of text. Modern font formats such as OpenType and AAT (Apple Advanced Typography) use a richer processing model. The glyph for a given character can be chosen and positioned not just based on the codepoint of the character itself, but also on adjacent characters as well as the language, script, and features enabled for the text. Font features may be required for specific scripts, or recommended as enabled by default or they might be stylistic features meant to be used under author control. The point at which font selection and positioning happens in the overall order of text processing operations (such as text transformation, text orientation and text alignment) is described in [\[CSS-TEXT-3\]](#ref-CSS-TEXT-3), [§ Text Processing Order of Operations](https://drafts.csswg.org/css-text-3/#order).

For a good visual overview of these features, see the [\[OPENTYPE-FONT-GUIDE\]](#ref-OPENTYPE-FONT-GUIDE). For a detailed description of glyph processing for OpenType fonts, see [\[WINDOWS-GLYPH-PROC\]](#ref-WINDOWS-GLYPH-PROC).

Stylistic font features can be classified into two broad categories: ones that affect the harmonization of glyph shapes with the surrounding context, such as kerning and ligature features, and ones such as the small-caps, subscript/superscript and alternate features that affect shape selection.

The subproperties of [‘`font-variant`’](#propdef-font-variant) listed below are used to control these stylistic font features. They do not control features that are required for displaying certain scripts, such as the OpenType features used when displaying Arabic or Indic language text. They affect glyph selection and positioning, but do not affect font selection as described in the font matching section (except in cases required for compatibility with CSS 2.1).

To assure consistent behavior across user agents, the equivalent OpenType property settings are listed for individual properties and are normative. When using other font formats these should be used as a guideline to map CSS font feature property values to specific font features.

### <a id="language-specific-support"></a>6.2. Language-specific display

OpenType also supports language-specific glyph selection and positioning, so that text can be displayed correctly in cases where the language dictates a specific display behavior. Many languages share a common script, but the shape of certain letters can vary across those languages. For example, certain Cyrillic letters have different shapes in Russian text than in Bulgarian. In Latin text, it's common to render "fi" with an explicit fi-ligature that lacks a dot on the "i". However, in languages such as Turkish which uses both a dotted-i and a dotless-i, it's important to not use this ligature or use a specialized version that contains a dot over the "i". The example below shows language-specific variations based on stylistic traditions found in Spanish, Italian and French orthography:

![language specific forms, spanish](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/locl-1.png)

![language specific forms, italian](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/locl-2.png)

![language specific forms, french](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/locl-3.png)

If the content language of the element is known according to the rules of the [document language](https://www.w3.org/TR/CSS21/conform.html#doclanguage), user agents are required to infer the OpenType language system from the content language and use that when selecting and positioning glyphs using an OpenType font.

### <a id="font-kerning-prop"></a>6.3. Kerning: the [font-kerning](#propdef-font-kerning) property

|                 |                                                                                                              |
|-----------------|--------------------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-kerning"></a>font-kerning                                                                              |
| Value:          | [auto](#font-kerning-auto-value) \| [normal](#font-kerning-normal-value) \| [none](#font-kerning-none-value) |
| Initial:        | auto                                                                                                         |
| Applies to:     | all elements                                                                                                 |
| Inherited:      | yes                                                                                                          |
| Percentages:    | N/A                                                                                                          |
| Media:          | visual                                                                                                       |
| Computed value: | as specified                                                                                                 |
| Animatable:     | no                                                                                                           |

Kerning is the contextual adjustment of inter-glyph spacing. This property controls metric kerning, kerning that utilizes adjustment data contained in the font.

<a id="font-kerning-auto-value"></a>auto  
Specifies that kerning is applied at the discretion of the user agent

<a id="font-kerning-normal-value"></a>normal  
Specifies that kerning is applied

<a id="font-kerning-none-value"></a>none  
Specifies that kerning is not applied

For fonts that do not include kerning data this property will have no visible effect. When rendering with OpenType fonts, the [\[OPENTYPE\]](#ref-OPENTYPE) specification suggests that kerning be enabled by default. When kerning is enabled, the OpenType kern feature is enabled (for vertical text runs the vkrn feature is enabled instead). User agents must also support fonts that only support kerning via data contained in a kern font table, as detailed in the OpenType specification. If the ‘`letter-spacing`’ property is defined, kerning adjustments are considered part of the default spacing and letter spacing adjustments are made after kerning has been applied.

When set to ‘`auto`’, user agents can determine whether to apply kerning or not based on a number of factors: text size, script, or other factors that influence text processing speed. Authors who want proper kerning should use [‘`normal`’](#font-kerning-normal-value) to explicitly enable kerning. Likewise, some authors may prefer to disable kerning in situations where performance is more important than precise appearance. However, in well-designed modern implementations the use of kerning generally does not have a large impact on text rendering speed.

### <a id="font-variant-ligatures-prop"></a>6.4. Ligatures: the [font-variant-ligatures](#propdef-font-variant-ligatures) property

|                 |                                                                                                                                                                                                                                                                                                             |
|-----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-variant-ligatures"></a>font-variant-ligatures                                                                                                                                                                                                                                                                   |
| Value:          | [normal](#font-variant-ligatures-normal-value) \| [none](#font-variant-ligatures-none-value) \| \[ [<var>&lt;common-lig-values&gt;</var>](#common-lig-values) \|\| [<var>&lt;discretionary-lig-values&gt;</var>](#discretionary-lig-values) \|\| [<var>&lt;historical-lig-values&gt;</var>](#historical-lig-values) \|\| [<var>&lt;contextual-alt-values&gt;</var>](#contextual-alt-values) \] |
| Initial:        | normal                                                                                                                                                                                                                                                                                                      |
| Applies to:     | all elements                                                                                                                                                                                                                                                                                                |
| Inherited:      | yes                                                                                                                                                                                                                                                                                                         |
| Percentages:    | N/A                                                                                                                                                                                                                                                                                                         |
| Media:          | visual                                                                                                                                                                                                                                                                                                      |
| Computed value: | as specified                                                                                                                                                                                                                                                                                                |
| Animatable:     | no                                                                                                                                                                                                                                                                                                          |

Ligatures and contextual forms are ways of combining glyphs to produce more harmonized forms.

<a id="common-lig-values"></a>

```text
<common-lig-values>        = [ common-ligatures | no-common-ligatures ]
```
<a id="discretionary-lig-values"></a>

```text
<discretionary-lig-values> = [ discretionary-ligatures | no-discretionary-ligatures ]
```
<a id="historical-lig-values"></a>

```text
<historical-lig-values>    = [ historical-ligatures | no-historical-ligatures ]
```
<a id="contextual-alt-values"></a>

```text
<contextual-alt-values>    = [ contextual | no-contextual ]
```
Individual values have the following meanings:

<a id="font-variant-ligatures-normal-value"></a>normal  
A value of [‘`normal`’](#font-variant-ligatures-normal-value) specifies that common default features are enabled, [as described in detail in the next section](#font-feature-resolution). For OpenType fonts, common ligatures and contextual forms are on by default, discretionary and historical ligatures are not.

<a id="font-variant-ligatures-none-value"></a>none  
Specifies that all types of ligatures and contextual forms covered by this property are explicitly disabled. In situations where ligatures are not considered necessary, this may improve the speed of text rendering.

<a id="common-ligatures"></a>common-ligatures  
Enables display of common ligatures (OpenType features: liga, clig). For OpenType fonts, common ligatures are enabled by default.

![common ligature example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/liga.png)

<a id="no-common-ligatures"></a>no-common-ligatures  
Disables display of common ligatures (OpenType features: liga, clig).

<a id="discretionary-ligatures"></a>discretionary-ligatures  
Enables display of discretionary ligatures (OpenType feature: dlig). Which ligatures are discretionary or optional is decided by the type designer, so authors will need to refer to the documentation of a given font to understand which ligatures are considered discretionary.

![discretionary ligature example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/dlig.png)

<a id="no-discretionary-ligatures"></a>no-discretionary-ligatures  
Disables display of discretionary ligatures (OpenType feature: dlig).

<a id="historical-ligatures"></a>historical-ligatures  
Enables display of historical ligatures (OpenType feature: hlig).

![historical ligature example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/hlig.png)

<a id="no-historical-ligatures"></a>no-historical-ligatures  
Disables display of historical ligatures (OpenType feature: hlig).

<a id="contextual"></a>contextual  
Enables display of contextual alternates (OpenType feature: calt). Although not strictly a ligature feature, like ligatures this feature is commonly used to harmonize the shapes of glyphs with the surrounding context. For OpenType fonts, this feature is on by default.

![contextual alternate example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/calt.png)

<a id="no-contextual"></a>no-contextual  
Disables display of contextual alternates (OpenType feature: calt).

Required ligatures, needed for correctly rendering complex scripts, are not affected by the settings above, including ‘`none`’ (OpenType feature: rlig).

### <a id="font-variant-position-prop"></a>6.5. Subscript and superscript forms: the [font-variant-position](#propdef-font-variant-position) property

|                 |                                                                                 |
|-----------------|---------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-variant-position"></a>font-variant-position                                        |
| Value:          | [normal](#font-variant-position-normal-value) \| [sub](#sub) \| [super](#super) |
| Initial:        | normal                                                                          |
| Applies to:     | all elements                                                                    |
| Inherited:      | yes                                                                             |
| Percentages:    | N/A                                                                             |
| Media:          | visual                                                                          |
| Computed value: | as specified                                                                    |
| Animatable:     | no                                                                              |

This property is used to enable typographic subscript and superscript glyphs. These are alternate glyphs designed within the same em-box as default glyphs and are intended to be laid out on the same baseline as the default glyphs, with no resizing or repositioning of the baseline. They are explicitly designed to match the surrounding text and to be more readable without affecting the line height.

![comparison between real subscript glyphs and synthesized ones](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/realsubscripts.png)

Subscript glyphs (top) vs. typical synthesized subscripts (bottom)

Individual values have the following meanings:

<a id="font-variant-position-normal-value"></a>normal  
None of the features listed below are enabled.

<a id="sub"></a>sub  
Enables display of subscript variants (OpenType feature: [subs](#sub)).

<a id="super"></a>super  
Enables display of superscript variants (OpenType feature: sups).

Because of the semantic nature of subscripts and superscripts, when the value is either ‘[`sub`](#sub)’ or ‘[`super`](#super)’ for a given contiguous run of text, if a variant glyph is not available for all the characters in the run, simulated glyphs should be synthesized for all characters using reduced forms of the glyphs that would be used without this feature applied. This is done per run to avoid a mixture of variant glyphs and synthesized ones that would not align correctly. In the case of OpenType fonts that lack subscript or superscript glyphs for a given character, user agents <em>must</em> synthesize appropriate subscript and superscript glyphs.

![alternate superscripts vs. glyphs synthesized using superscript metrics](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/superscript-alt-synth.png)

Superscript alternate glyph (left), synthesized superscript glyphs (middle), and incorrect mixture of the two (right)

In situations where text decorations are only applied to runs of text containing superscript or subscript glyphs, the synthesized glyphs may be used, to avoid problems with the placement of decorations.

In the past, user agents have used font-size and vertical-align to simulate subscripts and superscripts for the `sub` and `sup` elements. To allow a backwards compatible way of defining subscripts and superscripts, it is recommended that authors use conditional rules [\[CSS3-CONDITIONAL\]](#ref-CSS3-CONDITIONAL) so that older user agents will still render subscripts and superscripts via the older mechanism.

Because `font-size: smaller` is often used for these elements, the effective scaling factor applied to subscript and superscript text varies depending upon the size. For larger text, the font size is often reduced by a third but for smaller text sizes, the reduction can be much less. This allows subscripts and superscripts to remain readable even within elements using small text sizes. User agents should consider this when deciding how to synthesize subscript and superscript glyphs.

> <strong data-conversion-semantic="note">Note</strong>
>
> The OpenType font format defines subscript and superscript metrics in the [OS/2 table](https://docs.microsoft.com/en-us/typography/opentype/spec/os2) [\[OPENTYPE\]](#ref-OPENTYPE) but these are not always accurate in practice and so cannot be relied upon when synthesizing subscript and superscript glyphs.

Authors should note that fonts typically only provide subscript and superscript glyphs for a subset of all characters supported by the font. For example, while subscript and superscript glyphs are often available for Latin numbers, glyphs for punctuation and letter characters are less frequently provided. The synthetic fallback rules defined for this property assure that subscripts and superscripts will always appear but the appearance may not match author expectations if the font used does not provide the appropriate alternate glyph for all characters contained in a subscript or superscript.

This property is not cumulative. Applying it to elements within a subscript or superscript won't nest the placement of a subscript or superscript glyph. Images contained within text runs where the value of this property is ‘[`sub`](#sub)’ or ‘[`super`](#super)’ will be drawn just as they would if the value was [‘`normal`’](#font-variant-position-normal-value).

Because of these limitations, ‘[`font-variant-position`](#propdef-font-variant-position)’ is not recommended for use in user agent stylesheets. Authors should use it in cases where subscripts or superscripts will only contain the narrow range of characters supported by the fonts specified.

> <strong data-conversion-semantic="note">Note</strong>
>
> The variant glyphs use the same baseline as the default glyphs would use. There is no shift in the placement along the baseline, so the use of variant glyphs doesn't affect the height of the inline box or alter the height of the linebox. This makes superscript and subscript variants ideal for situations where it's important that leading remain constant, such as in multi-column layout.

> <strong data-conversion-semantic="example">Example</strong>
>
> A typical user agent default style for the [sub](#sub) element:
>
> ```text
> sub {
>   vertical-align: sub;
>   font-size: smaller;
>   line-height: normal;
> }
> ```
>
> Using ‘[`font-variant-position`](#propdef-font-variant-position)’ to specify typographic subscripts in a way that will still show subscripts in older user agents:
>
> ```text
> @supports ( font-variant-position: sub ) {
> 
>   sub {
>     vertical-align: baseline;
>     font-size: 100%;
>     line-height: inherit;
>     font-variant-position: sub;
>   }
> 
> }
> ```
>
> User agents that support the ‘[`font-variant-position`](#propdef-font-variant-position)’ property will select a subscript variant glyph and render this without adjusting the baseline or font-size. Older user agents will ignore the ‘[`font-variant-position`](#propdef-font-variant-position)’ property definition and use the standard defaults for subscripts.

### <a id="font-variant-caps-prop"></a>6.6. Capitalization: the [font-variant-caps](#propdef-font-variant-caps) property

|                 |                                                                                                                                                                                                                                           |
|-----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-variant-caps"></a>font-variant-caps                                                                                                                                                                                                      |
| Value:          | [normal](#font-variant-caps-normal-value) \| [small-caps](#small-caps) \| [all-small-caps](#all-small-caps) \| [petite-caps](#petite-caps) \| [all-petite-caps](#all-petite-caps) \| [unicase](#unicase) \| [titling-caps](#titling-caps) |
| Initial:        | normal                                                                                                                                                                                                                                    |
| Applies to:     | all elements                                                                                                                                                                                                                              |
| Inherited:      | yes                                                                                                                                                                                                                                       |
| Percentages:    | N/A                                                                                                                                                                                                                                       |
| Media:          | visual                                                                                                                                                                                                                                    |
| Computed value: | as specified                                                                                                                                                                                                                              |
| Animatable:     | no                                                                                                                                                                                                                                        |

This property allows the selection of alternate glyphs used for small or petite capitals or for titling. These glyphs are specifically designed to blend well with the surrounding normal glyphs, to maintain the weight and readability which suffers when text is simply resized to fit this purpose.

Individual values have the following meanings:

<a id="font-variant-caps-normal-value"></a>normal  
None of the features listed below are enabled.

<a id="small-caps"></a>small-caps  
Enables display of small capitals (OpenType feature: smcp). Small-caps glyphs typically use the form of uppercase letters but are reduced to the size of lowercase letters.

![small-caps example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/smcp.png)

<a id="all-small-caps"></a>all-small-caps  
Enables display of small capitals for both upper and lowercase letters (OpenType features: c2sc, smcp).

<a id="petite-caps"></a>petite-caps  
Enables display of petite capitals (OpenType feature: pcap).

<a id="all-petite-caps"></a>all-petite-caps  
Enables display of petite capitals for both upper and lowercase letters (OpenType features: c2pc, pcap).

<a id="unicase"></a>unicase  
Enables display of mixture of small capitals for uppercase letters with normal lowercase letters (OpenType feature: unic).

<a id="titling-caps"></a>titling-caps  
Enables display of titling capitals (OpenType feature: titl). Uppercase letter glyphs are often designed for use with lowercase letters. When used in all uppercase titling sequences they can appear too strong. Titling capitals are designed specifically for this situation.

The availability of these glyphs is based on whether a given feature is defined or not in the feature list of the font. User agents can optionally decide this on a per-script basis but should explicitly not decide this on a per-character basis.

Some fonts may only support a subset or none of the features described for this property. For backwards compatibility with CSS 2.1, if ‘[`small-caps`](#small-caps)’ or ‘[`all-small-caps`](#all-small-caps)’ is specified but small-caps glyphs are not available for a given font, user agents should simulate a small-caps font, for example by taking a normal font and replacing the glyphs for lowercase letters with scaled versions of the glyphs for uppercase characters (replacing the glyphs for both upper and lowercase letters in the case of ‘[`all-small-caps`](#all-small-caps)’).

![synthetic vs. real small-caps](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/synthetic-vs-real-small-caps.png)

Synthetic vs. real small-caps

The ‘`font-feature-settings`’ property does not affect the decision of whether or not to use a simulated small-caps font.

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> #example1 { font-variant-caps: small-caps; }
> #example2 { font-variant-caps: small-caps; font-feature-settings: 'smcp' 0; }
> ```
>
> For fonts which don't support small caps, both \#example1 and \#example2 should be rendered with synthesized small caps. However, for fonts which do support small caps, \#example1 should be rendered with native small caps, while \#example2 should be rendered without any small-caps (native or synthesized).

To match the surrounding text, a font may provide alternate glyphs for caseless characters when these features are enabled but when a user agent simulates small capitals, it must not attempt to simulate alternates for codepoints which are considered caseless.

![caseless characters with small-caps, all-small-caps enabled](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/small-capitals-variations.png)

Caseless characters with small-caps, all-small-caps enabled

If either ‘[`petite-caps`](#petite-caps)’ or ‘[`all-petite-caps`](#all-petite-caps)’ is specified for a font that doesn't support these features, the property behaves as if ‘[`small-caps`](#small-caps)’ or ‘[`all-small-caps`](#all-small-caps)’, respectively, had been specified. If ‘[`unicase`](#unicase)’ is specified for a font that doesn't support that feature, the property behaves as if ‘[`small-caps`](#small-caps)’ was applied only to lowercased uppercase letters. If ‘[`titling-caps`](#titling-caps)’ is specified with a font that does not support this feature, this property has no visible effect. When simulated small capital glyphs are used, for scripts that lack uppercase and lowercase letters, ‘[`small-caps`](#small-caps)’, ‘[`all-small-caps`](#all-small-caps)’, ‘[`petite-caps`](#petite-caps)’, ‘[`all-petite-caps`](#all-petite-caps)’ and ‘[`unicase`](#unicase)’ have no visible effect.

When casing transforms are used to simulate small capitals, the casing transformations must match those used for the ‘`text-transform`’ property.

As a last resort, unscaled uppercase letter glyphs in a normal font may replace glyphs in a small-caps font so that the text appears in all uppercase letters.

![using all-small-caps in acronym-laden text](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/acronym-laden-text.png)

Using small capitals to improve readability in acronym-laden text

> <strong data-conversion-semantic="example">Example</strong>
>
> Quotes rendered italicised, with small-caps on the first line:
>
> ```text
> blockquote            { font-style: italic; }
> blockquote:first-line { font-variant: small-caps; }
> 
> <blockquote>I'll be honor-bound to slap them like a haddock.</blockquote>
> ```
### <a id="font-variant-numeric-prop"></a>6.7. Numerical formatting: the [font-variant-numeric](#propdef-font-variant-numeric) property

|                 |                                                                                                                                                                                                                                                                          |
|-----------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-variant-numeric"></a>font-variant-numeric                                                                                                                                                                                                                                  |
| Value:          | [normal](#font-variant-numeric-normal-value) \| \[ [<var>&lt;numeric-figure-values&gt;</var>](#numeric-figure-values) \|\| [<var>&lt;numeric-spacing-values&gt;</var>](#numeric-spacing-values) \|\| [<var>&lt;numeric-fraction-values&gt;</var>](#numeric-fraction-values) \|\| [ordinal](#ordinal) \|\| [slashed-zero](#slashed-zero) \] |
| Initial:        | normal                                                                                                                                                                                                                                                                   |
| Applies to:     | all elements                                                                                                                                                                                                                                                             |
| Inherited:      | yes                                                                                                                                                                                                                                                                      |
| Percentages:    | N/A                                                                                                                                                                                                                                                                      |
| Media:          | visual                                                                                                                                                                                                                                                                   |
| Computed value: | as specified                                                                                                                                                                                                                                                             |
| Animatable:     | no                                                                                                                                                                                                                                                                       |

Specifies control over numerical forms. The example below shows how some of these values can be combined to influence the rendering of tabular data with fonts that support these features. Within normal paragraph text, proportional numbers are used while tabular numbers are used so that columns of numbers line up properly:

![combining number styles](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/numberstyles.png)

Using number styles

Possible combinations:

<a id="numeric-figure-values"></a>

```text
<numeric-figure-values>   = [ lining-nums | oldstyle-nums ]
```
<a id="numeric-spacing-values"></a>

```text
<numeric-spacing-values>  = [ proportional-nums | tabular-nums ]
```
<a id="numeric-fraction-values"></a>

```text
<numeric-fraction-values> = [ diagonal-fractions | stacked-fractions ]
```
Individual values have the following meanings:

<a id="font-variant-numeric-normal-value"></a>normal  
None of the features listed below are enabled.

<a id="lining-nums"></a>lining-nums  
Enables display of lining numerals (OpenType feature: lnum).

<a id="oldstyle-nums"></a>oldstyle-nums  
Enables display of old-style numerals (OpenType feature: onum).

<a id="proportional-nums"></a>proportional-nums  
Enables display of proportional numerals (OpenType feature: pnum).

<a id="tabular-nums"></a>tabular-nums  
Enables display of tabular numerals (OpenType feature: tnum).

<a id="diagonal-fractions"></a>diagonal-fractions  
Enables display of lining diagonal fractions (OpenType feature: frac).

![diagonal fraction example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/frac.png)

<a id="stacked-fractions"></a>stacked-fractions  
Enables display of lining stacked fractions (OpenType feature: afrc).

![stacked fraction example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/afrc.png)

<a id="ordinal"></a>ordinal  
Enables display of letter forms used with ordinal numbers (OpenType feature: ordn).

![ordinals example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/ordinals.png)

<a id="slashed-zero"></a>slashed-zero  
Enables display of slashed zeros (OpenType feature: zero).

![slashed zero example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/zero.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ordinal-example"></a>
>
> In the case of ‘[`ordinal`](#ordinal)’, although ordinal forms are often the same as superscript forms, they are marked up differently.
>
> For superscripts, the variant property is only applied to the sub-element containing the superscript:
>
> ```text
> 
> sup { font-variant-position: super; }
> x<sup>2</sup>
> ```
>
> For ordinals, the variant property is applied to the entire ordinal number rather than just to the suffix (or to the containing paragraph):
>
> ```text
> 
> .ordinal { font-variant-numeric: ordinal; }
> <span class="ordinal">17th</span>
> ```
>
> In this case only the "th" will appear in ordinal form, the digits will remain unchanged. Depending upon the typographic traditions used in a given language, ordinal forms may differ from superscript forms. In Italian, for example, ordinal forms sometimes include an underline in the ordinal design.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="steak-marinade"></a>
>
> A simple flank steak marinade recipe, rendered with automatic fractions and old-style numerals:
>
> ```text
> .amount { font-variant-numeric: oldstyle-nums diagonal-fractions; }
> 
> <h4>Steak marinade:</h4>
> <ul>
>   <li><span class="amount">2</span> tbsp olive oil</li>
>   <li><span class="amount">1</span> tbsp lemon juice</li>
>   <li><span class="amount">1</span> tbsp soy sauce</li>
>   <li><span class="amount">1 1/2</span> tbsp dry minced onion</li>
>   <li><span class="amount">2 1/2</span> tsp italian seasoning</li>
>   <li>Salt &amp; pepper</li>
> </ul>
> 
> <p>Mix the meat with the marinade and let it sit covered in the refrigerator
> for a few hours or overnight.</p>
> ```
>
> Note that the fraction feature is only applied to values not the entire paragraph. Fonts often implement this feature using contextual rules based on the use of the slash (‘`/`’) character. As such, it's not suitable for use as a paragraph-level style.

### <a id="font-variant-east-asian-prop"></a>6.8. East Asian text rendering: the [font-variant-east-asian](#propdef-font-variant-east-asian) property

|                 |                                                                                                                                                                                    |
|-----------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-variant-east-asian"></a>font-variant-east-asian                                                                                                                                         |
| Value:          | [normal](#font-variant-east-asian-normal-value) \| \[ [<var>&lt;east-asian-variant-values&gt;</var>](#east-asian-variant-values) \|\| [<var>&lt;east-asian-width-values&gt;</var>](#east-asian-width-values) \|\| [ruby](#ruby) \] |
| Initial:        | normal                                                                                                                                                                             |
| Applies to:     | all elements                                                                                                                                                                       |
| Inherited:      | yes                                                                                                                                                                                |
| Percentages:    | N/A                                                                                                                                                                                |
| Media:          | visual                                                                                                                                                                             |
| Computed value: | as specified                                                                                                                                                                       |
| Animatable:     | no                                                                                                                                                                                 |

Allows control of glyph substitution and sizing in East Asian text.

<a id="east-asian-variant-values"></a>

```text
<east-asian-variant-values> = [ jis78 | jis83 | jis90 | jis04 | simplified | traditional ]
```
<a id="east-asian-width-values"></a>

```text
<east-asian-width-values>   = [ full-width | proportional-width ]
```
Individual values have the following meanings:

<a id="font-variant-east-asian-normal-value"></a>normal  
None of the features listed below are enabled.

<a id="jis78"></a>jis78  
Enables rendering of JIS78 forms (OpenType feature: jp78).

![JIS78 form example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/jp78.png)

<a id="jis83"></a>jis83  
Enables rendering of JIS83 forms (OpenType feature: jp83).

<a id="jis90"></a>jis90  
Enables rendering of JIS90 forms (OpenType feature: jp90).

<a id="jis04"></a>jis04  
Enables rendering of JIS2004 forms (OpenType feature: jp04).

The various JIS variants reflect the glyph forms defined in different Japanese national standards. Fonts generally include glyphs defined by the most recent national standard but it's sometimes necessary to use older variants, to match signage for example.

<a id="simplified"></a>simplified  
Enables rendering of simplified forms (OpenType feature: smpl).

<a id="traditional"></a>traditional  
Enables rendering of traditional forms (OpenType feature: trad).

The ‘[`simplified`](#simplified)’ and ‘[`traditional`](#traditional)’ values allow control over the glyph forms for characters which have been simplified over time but for which the older, traditional form is still used in some contexts. The exact set of characters and glyph forms will vary to some degree by context for which a given font was designed.

![tradtional form example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/trad.png)

<a id="full-width"></a>full-width  
Enables rendering of full-width variants (OpenType feature: fwid).

<a id="proportional-width"></a>proportional-width  
Enables rendering of proportionally-spaced variants (OpenType feature: pwid).

![proportionally spaced Japanese example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/pwid.png)

<a id="ruby"></a>ruby  
Enables display of ruby variant glyphs (OpenType feature: [ruby](#ruby)). Since ruby text is generally smaller than the associated body text, font designers can design special glyphs for use with ruby that are more readable than scaled down versions of the default glyphs. Only glyph selection is affected, there is no associated font scaling or other change that affects line layout. The red ruby text below is shown with default glyphs (top) and with ruby variant glyphs (bottom). Note the slight difference in stroke thickness.

![ruby variant example](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/rubyshinkansen.png)

### <a id="font-variant-prop"></a>6.9. Overall shorthand for font rendering: the [font-variant](#propdef-font-variant) property

|                 |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-variant"></a>font-variant                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| Value:          | [normal](#font-variant-normal-value) \| [none](#font-variant-none-value) \| \[ [<var>&lt;common-lig-values&gt;</var>](#common-lig-values) \|\| [<var>&lt;discretionary-lig-values&gt;</var>](#discretionary-lig-values) \|\| [<var>&lt;historical-lig-values&gt;</var>](#historical-lig-values) \|\| [<var>&lt;contextual-alt-values&gt;</var>](#contextual-alt-values) \|\| \[ [<i>small-caps</i>](#small-caps) \| [<i>all-small-caps</i>](#all-small-caps) \| [<i>petite-caps</i>](#petite-caps) \| [<i>all-petite-caps</i>](#all-petite-caps) \| [<i>unicase</i>](#unicase) \| [<i>titling-caps</i>](#titling-caps) \] \|\| [<var>&lt;numeric-figure-values&gt;</var>](#numeric-figure-values) \|\| [<var>&lt;numeric-spacing-values&gt;</var>](#numeric-spacing-values) \|\| [<var>&lt;numeric-fraction-values&gt;</var>](#numeric-fraction-values) \|\| [<i>ordinal</i>](#ordinal) \|\| [<i>slashed-zero</i>](#slashed-zero) \|\| [<var>&lt;east-asian-variant-values&gt;</var>](#east-asian-variant-values) \|\| [<var>&lt;east-asian-width-values&gt;</var>](#east-asian-width-values) \|\| [<i>ruby</i>](#ruby) \|\| \[ [<i>sub</i>](#sub) \| [<i>super</i>](#super) \] \] |
| Initial:        | normal                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| Applies to:     | all elements                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| Inherited:      | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| Percentages:    | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Media:          | visual                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| Computed value: | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Animatable:     | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |

The [‘`font-variant`’](#propdef-font-variant) property is a shorthand for all font-variant subproperties. The value <a id="font-variant-normal-value"></a>‘`normal`’ resets all subproperties of [‘`font-variant`’](#propdef-font-variant) to their inital value. The <a id="font-variant-none-value"></a>‘`none`’ value sets ‘[`font-variant-ligatures`](#propdef-font-variant-ligatures)’ to ‘`none`’ and resets all other font feature properties to their initial value. Like other shorthands, using [‘`font-variant`’](#propdef-font-variant) resets unspecified [‘`font-variant`’](#propdef-font-variant) subproperties to their initial values. It does not reset the values of [‘`font-feature-settings`’](#propdef-font-feature-settings).

### <a id="font-feature-settings-prop"></a>6.10. Low-level font feature settings control: the [font-feature-settings](#propdef-font-feature-settings) property

|                 |                                                                                               |
|-----------------|-----------------------------------------------------------------------------------------------|
| Name:           | <a id="propdef-font-feature-settings"></a>font-feature-settings                                                      |
| Value:          | [normal](#font-feature-settings-normal-value) \| [<var>&lt;feature-tag-value&gt;</var>](#feature-tag-value) \# |
| Initial:        | normal                                                                                        |
| Applies to:     | all elements                                                                                  |
| Inherited:      | yes                                                                                           |
| Percentages:    | N/A                                                                                           |
| Media:          | visual                                                                                        |
| Computed value: | as specified                                                                                  |
| Animatable:     | no                                                                                            |

This property provides low-level control over OpenType font features. It is intended as a way of providing access to font features that are not widely used but are needed for a particular use case.

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Authors should generally use [‘`font-variant`’](#propdef-font-variant) and its related subproperties whenever possible and only use this property for special cases where its use is the only way of accessing a particular infrequently used font feature.

```text
/* enable small caps and use second swash alternate */
font-feature-settings: "smcp", "swsh" 2;
```
A value of <a id="font-feature-settings-normal-value"></a>‘`normal`’ means that no change in glyph selection or positioning occurs due to this property.

Feature tag values have the following syntax:

<a id="feature-tag-value"></a>

```text
<feature-tag-value> = <string> [ <integer> | on | off ]?
```
The \<string\> is a case-sensitive OpenType feature tag. As specified in the OpenType specification [\[OPENTYPE\]](#ref-OPENTYPE), feature tags contain four ASCII characters. Tag strings longer or shorter than four characters, or containing characters outside the U+20–7E codepoint range are invalid. Feature tags need only match a feature tag defined in the font, so they are not limited to explicitly registered OpenType features. Fonts defining custom feature tags should follow the [tag name rules](https://www.microsoft.com/typography/otspec/featuretags.htm) defined in the OpenType specification [\[OPENTYPE-FEATURES\]](#ref-OPENTYPE-FEATURES).

Feature tags not present in the font are ignored; a user agent must not attempt to synthesize fallback behavior based on these feature tags. The one exception is that user agents may synthetically support the kern feature with fonts that contain kerning data in the form of a ‘`kern`’ table but lack kern feature support in the ‘`GPOS`’ table.

> <strong data-conversion-semantic="note">Note</strong>
>
> In general, authors should use the ‘[`font-kerning`](#propdef-font-kerning)’ property to explicitly enable or disable kerning since this property always affects fonts with either type of kerning data.

If present, a value indicates an index used for glyph selection. An \<integer\> value must be 0 or greater. A value of 0 indicates that the feature is disabled. For boolean features, a value of 1 enables the feature. For non-boolean features, a value of 1 or greater enables the feature and indicates the feature selection index. A value of ‘`on`’ is synonymous with 1 and ‘`off`’ is synonymous with 0. If the value is omitted, a value of 1 is assumed.

<a id="ffs-map"></a>The computed value of font-feature-settings is a map, so any duplicates in the specified value must not be preserved. If the same axis name appears more than once, the value associated with the last appearance supersedes any previous value for that axis.

```text

font-feature-settings: "dlig" 1;       /* dlig=1 enable discretionary ligatures */
font-feature-settings: "smcp" on;      /* smcp=1 enable small caps */
font-feature-settings: 'c2sc';         /* c2sc=1 enable caps to small caps */
font-feature-settings: "liga" off;     /* liga=0 no common ligatures */
font-feature-settings: "tnum", 'hist'; /* tnum=1, hist=1 enable tabular numbers and historical forms */
font-feature-settings: "tnum" "hist";  /* invalid, need a comma-delimited list */
font-feature-settings: "silly" off;    /* invalid, tag too long */
font-feature-settings: "PKRN";         /* PKRN=1 enable custom feature */
font-feature-settings: dlig;           /* invalid, tag must be a string */
```
When values greater than the range supported by the font are specified, the behavior is explicitly undefined. For boolean features, in general these will enable the feature. For non-boolean features, out of range values will in general be equivalent to a 0 value. However, in both cases the exact behavior will depend upon the way the font is designed (specifically, which type of lookup is used to define the feature).

Although specifically defined for OpenType feature tags, feature tags for other modern font formats that support font features may be added in the future. Where possible, features defined for other font formats should attempt to follow the pattern of registered OpenType tags.

> <strong data-conversion-semantic="example">Example</strong>
>
> The Japanese text below will be rendered with half-width kana characters:
>
> ```text
> 
> body { font-feature-settings: "hwid"; /* Half-width OpenType feature */ }
> 
> <p>毎日カレー食べてるのに、飽きない</p>
> ```
<a id="rendering-considerations"></a>

## <a id="font-feature-resolution"></a>7. Font Feature Resolution

As described in the previous section, font features can be enabled in a variety of ways, either via the use of [‘`font-variant`’](#propdef-font-variant) or [‘`font-feature-settings`’](#propdef-font-feature-settings) in a style rule or within an [`@font-face`](#at-font-face-rule) rule. The resolution order for the union of these settings is defined below. Features defined via CSS properties are applied on top of layout engine default features.

### <a id="default-features"></a>7.1. Default features

For OpenType fonts, user agents must enable the default features defined in the OpenType documentation for a given script and writing mode. Required ligatures, common ligatures and contextual forms must be enabled by default (OpenType features: rlig, liga, clig, calt), along with localized forms (OpenType feature: locl), and features required for proper display of composed characters and marks (OpenType features: ccmp, mark, mkmk). These features must always be enabled, even when the value of the [‘`font-variant`’](#propdef-font-variant) and [‘`font-feature-settings`’](#propdef-font-feature-settings) properties is ‘`normal`’. Individual features are only disabled when explicitly overridden by the author, as when ‘[`font-variant-ligatures`](#propdef-font-variant-ligatures)’ is set to ‘[`no-common-ligatures`](#no-common-ligatures)’. For handling complex scripts such as [Arabic](https://docs.microsoft.com/en-us/typography/script-development/arabic) [\[ARABIC-TYPO\]](#ref-ARABIC-TYPO), [Khmer](https://docs.microsoft.com/en-us/typography/script-development/khmer) or [Devanagari](https://docs.microsoft.com/en-us/typography/script-development/devanagari) additional features are required. For upright text within vertical text runs, vertical alternates (OpenType feature: vert) must be enabled.

### <a id="feature-precedence"></a>7.2. Feature precedence

General and <em>font specific</em> font feature property settings are resolved in the order below, in ascending order of precedence. This ordering is used to construct a combined list of font features that affect a given text run.

1.  Font features enabled by default, including features required for a given script.
2.  If the font is defined via an [`@font-face`](#at-font-face-rule) rule, the font features implied by the font-feature-settings descriptor in the [`@font-face`](#at-font-face-rule) rule.
3.  Font features implied by the value of the [‘`font-variant`’](#propdef-font-variant) property, the related [‘`font-variant`’](#propdef-font-variant) subproperties and any other CSS property that uses OpenType features (e.g. the ‘[`font-kerning`](#propdef-font-kerning)’ property).
4.  Feature settings determined by properties other than [‘`font-variant`’](#propdef-font-variant) or [‘`font-feature-settings`’](#propdef-font-feature-settings). For example, setting a non-default value for the ‘`letter-spacing`’ property disables common ligatures.
5.  Font features implied by the value of [‘`font-feature-settings`’](#propdef-font-feature-settings) property.

This ordering allows authors to set up a general set of defaults for fonts within their [`@font-face`](#at-font-face-rule) rules, then override them with property settings for specific elements. General property settings override the settings in [`@font-face`](#at-font-face-rule) rules and low-level font feature settings override [‘`font-variant`’](#propdef-font-variant) property settings.

For situations where the combined list of font feature settings contains more than one value for the same feature, the last value is used. When a font lacks support for a given underlying font feature, text is simply rendered as if that font feature was not enabled; font fallback does not occur and no attempt is made to synthesize the feature except where explicitly defined for specific properties.

### <a id="feature-precedence-examples"></a>7.3. Feature precedence examples

> <strong data-conversion-semantic="example">Example</strong>
>
> With the styles below, numbers are rendered proportionally when used within a paragraph but are shown in tabular form within tables of prices:
>
> ```text
> body {
>   font-variant-numeric: proportional-nums;
> }
> 
> table.prices td {
>   font-variant-numeric: tabular-nums;
> }
> ```
## <a id="object-model"></a>8. Object Model

The contents of [`@font-face`](#at-font-face-rule) rules can be accessed via the following extension to the CSS Object Model.

### <a id="om-fontface"></a>8.1. The [`CSSFontFaceRule`](#cssfontfacerule) interface

The <a id="cssfontfacerule"></a>CSSFontFaceRule interface represents a [`@font-face`](#at-font-face-rule) rule.

```text

interface CSSFontFaceRule : CSSRule {
    readonly attribute CSSStyleDeclaration style;
};
```
## <a id="platform-props-to-css"></a>Appendix A: Mapping platform font properties to CSS properties

<em>This appendix is included as background for some of the problems and
   situations that are described in other sections. It should be viewed as
   informative only.</em>

Font properties in CSS are designed to be independent of the underlying font formats used; they can be used to specify bitmap fonts, Type1 fonts, SVG fonts in addition to the common TrueType and OpenType fonts. But there are facets of the TrueType and OpenType formats that often cause confusion for authors and present challenges to implementers on different platforms.

Originally developed at Apple, TrueType \[\[TRUETYPE\]\] was designed as an outline font format for both screen and print. Microsoft joined Apple in developing the TrueType format and both platforms have supported TrueType fonts since then. Font data in the TrueType format consists of a set of tables distinguished with common four-letter tag names, each containing a specific type of data. For example, naming information, including copyright and license information, is stored in the ‘`name`’ table. The [<em>character map</em>](#character-map) (‘`cmap`’) table contains a mapping of character encodings to glyphs. Apple later added additional tables for supporting enhanced typographic functionality; these are now called Apple Advanced Typography, or AAT, fonts. Microsoft and Adobe developed a separate set of tables for advanced typography and called their format OpenType [\[OPENTYPE\]](#ref-OPENTYPE). The OpenType specification is standardized at ISO as the Open Font Format [\[OPEN-FONT-FORMAT\]](#ref-OPEN-FONT-FORMAT).

In many cases the font data used under Microsoft Windows or Linux is slightly different from the data used under Apple's Mac OS X because the TrueType format allowed for explicit variation across platforms. This includes font metrics, names and [<em>character
   map</em>](#character-map) data.

Specifically, font family name data is handled differently across platforms. For TrueType and OpenType fonts these names are contained in the ‘`name`’ table, in name records with name ID 1. Multiple names can be stored for different locales, but Microsoft recommends fonts always include at least a US English version of the name. On Windows, Microsoft made the decision for backwards compatibility to limit this family name to a maximum of four faces; for larger groupings the "preferred family" (name ID 16) or "WWS family" (name ID 21) can be used. Other platforms such as OSX don't have this limitation, so the family name is used to define all possible groupings.

Other name table data provides names used to uniquely identify a specific face within a family. The full font name (name ID 4) and the Postscript name (name ID 6) describe a single face uniquely. For example, the bold face of the Gill Sans family has a fullname of "Gill Sans Bold" and a Postscript name of "GillSans-Bold". There can be multiple localized versions of the fullname for a given face, but the Postscript name is always a unique name made from a limited set of ASCII characters.

On various platforms, different names are used to search for a font. For example, with the Windows GDI CreateIndirectFont API, either a family or fullname can be used to lookup a face, while on Mac OS X the CTFontCreateWithName API call is used to lookup a given face using the fullname and Postscript name. Under Linux, the fontconfig API allows fonts to be searched using any of these names. In situations where platform API's automatically substitute other font choices, it may be necessary to verify a returned font matches a given name.

The weight of a given face can be determined via the usWeightClass field of the OS/2 table or inferred from the style name (name ID 2). Likewise, the width can be determined via the usWidthClass of the OS/2 table or inferred from the style name. For historical reasons related to synthetic bolding at weights 200 or lower with the Windows GDI API, font designers have sometimes skewed values in the OS/2 table to avoid these weights.

Rendering complex scripts that use contextual shaping such as Thai, Arabic and Devanagari requires features present only in OpenType or AAT fonts. Currently, complex script rendering is supported on Windows and Linux using OpenType font features while both OpenType and AAT font features are used under Mac OS X.

## <a id="ch-ch-ch-changes"></a>Changes

### <a id="super-recent-changes"></a>Changes from the [14 August 2018 CSS Fonts 3 Proposed Recommendation](https://www.w3.org/TR/2018/PR-css-fonts-3-20180814/)

- Features mentioned in the changelog as having been moved to CSS Fonts 4, now link to the corresponding section in that specification
- Date and boilerplate updates for W3C Recommendation
- Unicode reference updated to latest version
- Updated this changes section

### <a id="recent-changes"></a> Changes from the [March 15 2018 CSS Fonts 3 Candidate Recommendation](https://www.w3.org/TR/2018/CR-css-fonts-3-20180315/)

- ‘`font-variant`’ descriptor moved to [CSS Fonts 4](https://drafts.csswg.org/css-fonts-4/#font-rend-desc) due to lack of implementations
- ‘`font-feature-values`’ at-rule moved to [CSS Fonts 4](https://drafts.csswg.org/css-fonts-4/#font-display-font-feature-values) due to lack of implementations
- clarified handling of unknown fragment identifiers
- linked to CSS Values &#x26; Units for definition of length-percentage

### <a id="less-recent-changes"></a> Changes from the [October 2013 CSS3 Fonts Candidate Recommendation](https://www.w3.org/TR/2013/CR-css-fonts-3-20131003/)

- ‘`font-language-override`’ property moved to [CSS Fonts 4](https://drafts.csswg.org/css-fonts-4/#font-language-override-prop)
- CSSFontFeatureValuesRule interface moved to [CSS Fonts 4](https://drafts.csswg.org/css-fonts-4/#om-fontfeaturevalues)
- CSSFontFaceRule interface reverted to the widely implemented one from DOM Level 2 style
- clarified that generic font families may be composite faces
- clarified that "first available font" is one that would match the U+0020 (space) character
- clarified how small-caps synthesis interacts with ‘`font-feature-settings`’
- all CSS keywords marked as invalid font family names
- clarified that ‘[`font-synthesis`](#propdef-font-synthesis)’ is not reset by the ‘[`font`](#propdef-font)’ shorthand.
- use the phrase "installed fonts" rather than "system fonts"
- clarified that malformed @font-face rules which lack font-family: or src: still show up in the DOM, but don't affect font selection
- clarified conventional ratio range for the relative sizes when they're not modifying an absolute keyword size
- clarified that for both font-variation-settings and font-feature-settings, the computed value is a map (and thus specified dupes are removed)
- added omitted ‘[`font-variant-position`](#propdef-font-variant-position)’ values to [‘`font-variant`’](#propdef-font-variant) shorthand
- made negative values for font-size-adjust invalid, along with negative percentage font-size values
- removed the requirement that user agents use OS/2 table subscript/superscript metrics
- added informative link to CSS Text order of operations
- added normative link to RFC 8081, the font top-level type
- minor editorial cleanups

## <a id="acknowledgments"></a>Acknowledgments

We'd like to thank Tal Leming, Jonathan Kew, Ken Lunde and Christopher Slye for all their help and feedback. John Hudson was kind enough to take the time to explain the subtleties of OpenType language tags and provided the example of character variant usage for displaying text on Byzantine seals. Ken Lunde and Eric Muller provided valuable feedback on CJK OpenType features and Unicode variation selectors. The idea for supporting font features by using [‘`font-variant`’](#propdef-font-variant) subproperties originated with Håkon Wium Lie, Adam Twardoch and Tal Leming. Elika Etemad supplied some of the initial design ideas for the `@font-feature-values` rule. Thanks also to House Industries for allowing the use of Ed Interlock in the discretionary ligatures example.

A special thanks to Robert Bringhurst for the sublime mind expansion that is <em>The Elements of Typographic Style</em>.

## <a id="conformance"></a> Conformance

### <a id="conventions"></a> Document Conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#ref-RFC2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

### <a id="conformance-classes"></a> Conformance Classes

Conformance to CSS Fonts Level 3 Module is defined for three conformance classes:

<a id="style-sheet"></a>style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS21/conform.html#style-sheet).

<a id="renderer"></a>renderer  
A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

<a id="authoring-tool"></a>authoring tool  
A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to CSS Fonts Level 3 Module if all of its declarations that use properties defined in this module have values that are valid according to the generic CSS grammar and the individual grammars of each property as given in this module.

A renderer is conformant to CSS Fonts Level 3 Module if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by CSS Fonts Level 3 Module by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to CSS Fonts Level 3 Module if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="partial"></a> Partial Implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](https://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

### <a id="experimental"></a> Experimental Implementations

To avoid clashes with future CSS features, the CSS2.1 specification reserves a [prefixed syntax](https://www.w3.org/TR/CSS21/syndata.html#vendor-keywords) for proprietary and experimental extensions to CSS.

Prior to a specification reaching the Candidate Recommendation stage in the W3C process, all implementations of a CSS feature are considered experimental. The CSS Working Group recommends that implementations use a vendor-prefixed syntax for such features, including those in W3C Working Drafts. This avoids incompatibilities with future changes in the draft.

### <a id="testing"></a> Non-Experimental Implementations

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group's website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="references"></a>References

### <a id="normative-references"></a>Normative References

<a id="ref-CSS-VALUES"></a>\[CSS-VALUES\]  
[Tab Atkins Jr.; Elika Etemad. CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values/) 29 September 2016. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values&#x2F;](https://www.w3.org/TR/css-values/)

<a id="ref-FETCH"></a>\[FETCH\]  
[Fetch](https://fetch.spec.whatwg.org/). WhatWG Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fetch&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fetch.spec.whatwg.org/)

<a id="ref-OPENTYPE"></a>\[OPENTYPE\]  
[OpenType specification.](http://www.microsoft.com/typography/otspec/default.htm) Microsoft. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;microsoft&#x2E;com&#x2F;typography&#x2F;otspec&#x2F;default&#x2E;htm](http://www.microsoft.com/typography/otspec/default.htm)

<a id="ref-OPENTYPE-FEATURES"></a>\[OPENTYPE-FEATURES\]  
[OpenType feature registry.](http://www.microsoft.com/typography/otspec/featurelist.htm) Microsoft. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;microsoft&#x2E;com&#x2F;typography&#x2F;otspec&#x2F;featurelist&#x2E;htm](http://www.microsoft.com/typography/otspec/featurelist.htm)

<a id="ref-RFC2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels.](http://www.ietf.org/rfc/rfc2119.txt) RFC 2119. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;ietf&#x2E;org&#x2F;rfc&#x2F;rfc2119&#x2E;txt](http://www.ietf.org/rfc/rfc2119.txt)

<a id="ref-RFC8081"></a>\[RFC8081\]  
C. Lilley. [The "font" Top-Level Media Type](https://tools.ietf.org/html/rfc8081). February 2017. Proposed Standard. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc8081](https://tools.ietf.org/html/rfc8081)

<a id="ref-UAX15"></a>\[UAX15\]  
Mark Davis; Ken Whistler. [Unicode Normalization Forms.](http://www.unicode.org/reports/tr15/) 31 August 2012. Unicode Standard Annex \#15. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr15&#x2F;](http://www.unicode.org/reports/tr15/)

<a id="ref-UNICODE"></a>\[UNICODE\]  
[The Unicode Standard](http://www.unicode.org/versions/latest/) URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;versions&#x2F;latest](http://www.unicode.org/versions/latest/)

### <a id="other-references"></a>Other References

<a id="ref-AAT-FEATURES"></a>\[AAT-FEATURES\]  
[Apple Advanced Typography font feature registry.](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html) Apple. URL: [https&#x3A;&#x2F;&#x2F;developer&#x2E;apple&#x2E;com&#x2F;fonts&#x2F;TrueType-Reference-Manual&#x2F;RM09&#x2F;AppendixF&#x2E;html](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html)

<a id="ref-ARABIC-TYPO"></a>\[ARABIC-TYPO\]  
Huda Smitshuijzen AbiFares. Arabic Typography: A Comprehensive Sourcebook. Saqi Books. 2001. ISBN 0-86356-347-3.

<a id="ref-CHARMOD"></a>\[CHARMOD\]  
Martin J. Dürst; et al. [Character Model for the World Wide Web 1.0: Fundamentals.](https://www.w3.org/TR/2005/REC-charmod-20050215/) 15 February 2005. W3C Recommendation. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2005&#x2F;REC-charmod-20050215&#x2F;](https://www.w3.org/TR/2005/REC-charmod-20050215/)

<a id="ref-CHARMOD-NORM"></a>\[CHARMOD-NORM\]  
Addison Phillips. [Character Model for the World Wide Web: String Matching.](https://www.w3.org/TR/2018/WD-charmod-norm-20180420/) 20 April 2018. W3C Working Draft. (Work in progress.) URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2018&#x2F;WD-charmod-norm-20180420&#x2F;](https://www.w3.org/TR/2018/WD-charmod-norm-20180420/)

<a id="ref-CSS-TEXT-3"></a>\[CSS-TEXT-3\]  
Elika J. Etemad / fantasai; Koji Ishii. [CSS Text Module Level 3.](https://www.w3.org/TR/2017/WD-css-text-3-20170822/) 22 August 2017. W3C Working Draft. (Work in progress.) URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2017&#x2F;WD-css-text-3-20170822&#x2F;](https://www.w3.org/TR/2017/WD-css-text-3-20170822/)

<a id="ref-CSS3-CONDITIONAL"></a>\[CSS3-CONDITIONAL\]  
L. David Baron. [CSS Conditional Rules Module Level 3.](https://www.w3.org/TR/2013/CR-css3-conditional-20130404/) 4 April 2013. W3C Candidate Recommendation. (Work in progress.) URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2013&#x2F;CR-css3-conditional-20130404&#x2F;](https://www.w3.org/TR/2013/CR-css3-conditional-20130404/)

<a id="ref-OPEN-FONT-FORMAT"></a>\[OPEN-FONT-FORMAT\]  
[Information technology — Coding of audio-visual objects — Part 22: Open Font Format.](http://standards.iso.org/ittf/PubliclyAvailableStandards/c052136_ISO_IEC_14496-22_2009%28E%29.zip) International Organization for Standardization. ISO/IEC 14496-22:2009. URL: [http&#x3A;&#x2F;&#x2F;standards&#x2E;iso&#x2E;org&#x2F;ittf&#x2F;PubliclyAvailableStandards&#x2F;c052136_ISO_IEC_14496-22_2009%28E%29&#x2E;zip](http://standards.iso.org/ittf/PubliclyAvailableStandards/c052136_ISO_IEC_14496-22_2009%28E%29.zip)

<a id="ref-OPENTYPE-FONT-GUIDE"></a>\[OPENTYPE-FONT-GUIDE\]  
[OpenType User Guide.](http://www.fontblog.de/wp-content/uploads/2015/11/FF_OTF_user_guide.pdf) FontShop International. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;fontblog&#x2E;de&#x2F;wp-content&#x2F;uploads&#x2F;2015&#x2F;11&#x2F;FF_OTF_user_guide&#x2E;pdf](http://www.fontblog.de/wp-content/uploads/2015/11/FF_OTF_user_guide.pdf)

<a id="ref-TRUETYPE"></a>\[TRUETYPE\]  
[TrueType™ Reference Manual](https://developer.apple.com/fonts/TrueType-Reference-Manual/). Apple. URL: [https&#x3A;&#x2F;&#x2F;developer&#x2E;apple&#x2E;com&#x2F;fonts&#x2F;TrueType-Reference-Manual&#x2F;](https://developer.apple.com/fonts/TrueType-Reference-Manual/)

<a id="ref-UAX29"></a>\[UAX29\]  
Mark Davis. [Unicode Text Segmentation.](http://www.unicode.org/reports/tr29/) 12 September 2012. Unicode Standard Annex \#29. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr29&#x2F;](http://www.unicode.org/reports/tr29/)

<a id="ref-WINDOWS-GLYPH-PROC"></a>\[WINDOWS-GLYPH-PROC\]  
John Hudson. [Windows Glyph Processing.](http://www.microsoft.com/typography/developers/opentype/default.htm) Microsoft Typogrraphy. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;microsoft&#x2E;com&#x2F;typography&#x2F;developers&#x2F;opentype&#x2F;default&#x2E;htm](http://www.microsoft.com/typography/developers/opentype/default.htm)

## <a id="index"></a>Index

- 100...900 weight values, [<strong>3.2.</strong>](#font-weight-numeric-values)
- <var>&lt;absolute-size&gt;</var>, [<strong>3.5.</strong>](#absolute-size-value)
- all-petite-caps, [<strong>6.6.</strong>](#all-petite-caps)
- all-small-caps, [<strong>6.6.</strong>](#all-small-caps)
- aspect value, [<strong>3.6.</strong>](#aspect-value0)
- authoring tool, [<strong>??</strong>](#authoring-tool)
- auto
  - font-kerning, [<strong>6.3.</strong>](#font-kerning-auto-value)
- bold, [<strong>3.2.</strong>](#bold)
- bolder, [<strong>3.2.</strong>](#bolder)
- character map, [<strong>5.2.</strong>](#character-map)
- common-ligatures, [<strong>6.4.</strong>](#common-ligatures)
- <var>&lt;common-lig-values&gt;</var>, [<strong>6.4.</strong>](#common-lig-values)
- composite face, [<strong>5.2.</strong>](#composite-face)
- condensed, [<strong>3.3.</strong>](#condensed)
- contextual, [<strong>6.4.</strong>](#contextual)
- <var>&lt;contextual-alt-values&gt;</var>, [<strong>6.4.</strong>](#contextual-alt-values)
- CSSFontFaceRule, [<strong>8.1.</strong>](#cssfontfacerule)
- cursive, definition of, [<strong>??</strong>](#cursive0)
- default face, [<strong>5.2.</strong>](#default-face)
- descriptor_declaration, [<strong>4.1.</strong>](#descriptor_declaration)
- diagonal-fractions, [<strong>6.7.</strong>](#diagonal-fractions)
- discretionary-ligatures, [<strong>6.4.</strong>](#discretionary-ligatures)
- <var>&lt;discretionary-lig-values&gt;</var>, [<strong>6.4.</strong>](#discretionary-lig-values)
- <var>&lt;east-asian-variant-values&gt;</var>, [<strong>6.8.</strong>](#east-asian-variant-values)
- <var>&lt;east-asian-width-values&gt;</var>, [<strong>6.8.</strong>](#east-asian-width-values)
- effective character map, [<strong>4.5.</strong>](#effective-character-map)
- expanded, [<strong>3.3.</strong>](#expanded)
- extra-condensed, [<strong>3.3.</strong>](#extra-condensed)
- extra-expanded, [<strong>3.3.</strong>](#extra-expanded)
- <var>&lt;family-name&gt;</var>, [<strong>3.1.</strong>](#family-name-value)
- fantasy, definition of, [<strong>??</strong>](#fantasy0)
- <var>&lt;feature-tag-value&gt;</var>, [<strong>6.10.</strong>](#feature-tag-value)
- first available font, [<strong>5.2.</strong>](#first-available-font)
- font, [<strong>3.7.</strong>](#propdef-font)
- `@font-face`, [<strong>4.1.</strong>](#at-font-face-rule)
- <var>&lt;font-face-name&gt;</var>, [<strong>4.3.</strong>](#font-face-name-value)
- font_face_rule, [<strong>4.1.</strong>](#font_face_rule)
- FONT_FACE_SYM, [<strong>4.1.</strong>](#font_face_sym)
- font-family
  - descriptor, [<strong>4.2.</strong>](#descdef-font-family)
  - property, [<strong>3.1.</strong>](#propdef-font-family)
- font-feature-settings
  - descriptor, [<strong>4.7.</strong>](#descdef-font-feature-settings)
  - property, [<strong>6.10.</strong>](#propdef-font-feature-settings)
- font-kerning, [<strong>6.3.</strong>](#propdef-font-kerning)
- font-size, [<strong>3.5.</strong>](#propdef-font-size)
- font-size-adjust, [<strong>3.6.</strong>](#propdef-font-size-adjust)
- font-stretch
  - descriptor, [<strong>4.4.</strong>](#descdef-font-stretch)
  - property, [<strong>3.3.</strong>](#propdef-font-stretch)
- font-style
  - descriptor, [<strong>4.4.</strong>](#descdef-font-style)
  - property, [<strong>3.4.</strong>](#propdef-font-style)
- font-synthesis, [<strong>3.8.</strong>](#propdef-font-synthesis)
- font-variant
  - property, [<strong>6.9.</strong>](#propdef-font-variant)
- font-variant-caps, [<strong>6.6.</strong>](#propdef-font-variant-caps)
- <var>&lt;font-variant-css21&gt;</var>, [<strong>3.7.</strong>](#font-variant-css21-values)
- font-variant-east-asian, [<strong>6.8.</strong>](#propdef-font-variant-east-asian)
- font-variant-ligatures, [<strong>6.4.</strong>](#propdef-font-variant-ligatures)
- font-variant-numeric, [<strong>6.7.</strong>](#propdef-font-variant-numeric)
- font-variant-position, [<strong>6.5.</strong>](#propdef-font-variant-position)
- font-weight
  - descriptor, [<strong>4.4.</strong>](#descdef-font-weight)
  - property, [<strong>3.2.</strong>](#propdef-font-weight)
- full-width, [<strong>6.8.</strong>](#full-width)
- <var>&lt;generic-family&gt;</var>, [<strong>3.1.</strong>](#generic-family-value)
- historical-ligatures, [<strong>6.4.</strong>](#historical-ligatures)
- <var>&lt;historical-lig-values&gt;</var>, [<strong>6.4.</strong>](#historical-lig-values)
- italic, [<strong>3.4.</strong>](#italic)
- jis04, [<strong>6.8.</strong>](#jis04)
- jis78, [<strong>6.8.</strong>](#jis78)
- jis83, [<strong>6.8.</strong>](#jis83)
- jis90, [<strong>6.8.</strong>](#jis90)
- lighter, [<strong>3.2.</strong>](#lighter)
- lining-nums, [<strong>6.7.</strong>](#lining-nums)
- monospace, definition of, [<strong>??</strong>](#monospace0)
- no-common-ligatures, [<strong>6.4.</strong>](#no-common-ligatures)
- no-contextual, [<strong>6.4.</strong>](#no-contextual)
- no-discretionary-ligatures, [<strong>6.4.</strong>](#no-discretionary-ligatures)
- no-historical-ligatures, [<strong>6.4.</strong>](#no-historical-ligatures)
- none
  - font-kerning, [<strong>6.3.</strong>](#font-kerning-none-value)
  - font-size-adjust, [<strong>3.6.</strong>](#font-size-adjust-none-value)
  - font-variant, [<strong>6.9.</strong>](#font-variant-none-value)
  - font-variant-ligatures, [<strong>6.4.</strong>](#font-variant-ligatures-none-value)
- normal
  - font-feature-settings, [<strong>6.10.</strong>](#font-feature-settings-normal-value)
  - font-kerning, [<strong>6.3.</strong>](#font-kerning-normal-value)
  - font-stretch, [<strong>3.3.</strong>](#font-stretch-normal-value)
  - font-style, [<strong>3.4.</strong>](#font-style-normal-value)
  - font-variant, [<strong>6.9.</strong>](#font-variant-normal-value)
  - font-variant-caps, [<strong>6.6.</strong>](#font-variant-caps-normal-value)
  - font-variant-east-asian, [<strong>6.8.</strong>](#font-variant-east-asian-normal-value)
  - font-variant-ligatures, [<strong>6.4.</strong>](#font-variant-ligatures-normal-value)
  - font-variant-numeric, [<strong>6.7.</strong>](#font-variant-numeric-normal-value)
  - font-variant-position, [<strong>6.5.</strong>](#font-variant-position-normal-value)
  - font-weight, [<strong>3.2.</strong>](#font-weight-normal-value)
- <var>&lt;number&gt;</var>, [<strong>3.6.</strong>](#aspect-ratio-value)
- <var>&lt;numeric-figure-values&gt;</var>, [<strong>6.7.</strong>](#numeric-figure-values)
- <var>&lt;numeric-fraction-values&gt;</var>, [<strong>6.7.</strong>](#numeric-fraction-values)
- <var>&lt;numeric-spacing-values&gt;</var>, [<strong>6.7.</strong>](#numeric-spacing-values)
- oblique, [<strong>3.4.</strong>](#oblique)
- oldstyle-nums, [<strong>6.7.</strong>](#oldstyle-nums)
- ordinal, [<strong>6.7.</strong>](#ordinal)
- petite-caps, [<strong>6.6.</strong>](#petite-caps)
- proportional-nums, [<strong>6.7.</strong>](#proportional-nums)
- proportional-width, [<strong>6.8.</strong>](#proportional-width)
- <var>&lt;relative-size&gt;</var>, [<strong>3.5.</strong>](#relative-size-value)
- renderer, [<strong>??</strong>](#renderer)
- ruby, [<strong>6.8.</strong>](#ruby)
- sans-serif, definition of, [<strong>??</strong>](#sans-serif0)
- semi-condensed, [<strong>3.3.</strong>](#semi-condensed-)
- semi-expanded, [<strong>3.3.</strong>](#semi-expanded)
- serif, definition of, [<strong>??</strong>](#serif0)
- simplified, [<strong>6.8.</strong>](#simplified)
- slashed-zero, [<strong>6.7.</strong>](#slashed-zero)
- small-caps, [<strong>6.6.</strong>](#small-caps)
- src, [<strong>4.3.</strong>](#descdef-src)
- stacked-fractions, [<strong>6.7.</strong>](#stacked-fractions)
- style sheet
  - as conformance class, [<strong>??</strong>](#style-sheet)
- sub, [<strong>6.5.</strong>](#sub)
- super, [<strong>6.5.</strong>](#super)
- support, [<strong>5.2.</strong>](#support)
- system font fallback, [<strong>5.2.</strong>](#system-font-fallback)
- tabular-nums, [<strong>6.7.</strong>](#tabular-nums)
- titling-caps, [<strong>6.6.</strong>](#titling-caps)
- traditional, [<strong>6.8.</strong>](#traditional)
- ultra-condensed, [<strong>3.3.</strong>](#ultra-condensed)
- ultra-expanded, [<strong>3.3.</strong>](#ultra-expanded)
- unicase, [<strong>6.6.</strong>](#unicase)
- unicode-range, [<strong>4.5.</strong>](#descdef-unicode-range)
- <var>&lt;urange&gt;</var>, [<strong>4.5.</strong>](#urange-value)
- weight, [<strong>2.</strong>](#weight)
- width, [<strong>2.</strong>](#width)

## <a id="property-index"></a>Property index

| Property                                                    | Values                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | Initial                   | Applies to   | Inh. | Percentages                         | Media  |
|-------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------|--------------|------|-------------------------------------|--------|
| [font](#propdef-font)                                       | \[ \[ \<‘font-style’\> \|\| \<font-variant-css21\> \|\| \<‘font-weight’\> \|\| \<‘font-stretch’\> \]? \<‘font-size’\> \[ / \<‘line-height’\> \]? \<‘font-family’\> \] \| caption \| icon \| menu \| message-box \| small-caption \| status-bar                                                                                                                                                                                                                                          | see individual properties | all elements | yes  | see individual properties           | visual |
| [font-family](#propdef-font-family)                         | \[ \<family-name\> \| \<generic-family\> \] \#                                                                                                                                                                                                                                                                                                                                                                                                                                          | depends on user agent     | all elements | yes  | N/A                                 | visual |
| [font-feature-settings](#propdef-font-feature-settings)     | normal \| \<feature-tag-value\> \#                                                                                                                                                                                                                                                                                                                                                                                                                                                      | normal                    | all elements | yes  | N/A                                 | visual |
| [font-kerning](#propdef-font-kerning)                       | auto \| normal \| none                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | auto                      | all elements | yes  | N/A                                 | visual |
| [font-size](#propdef-font-size)                             | \<absolute-size\> \| \<relative-size\> \| \<length-percentage\>                                                                                                                                                                                                                                                                                                                                                                                                                         | medium                    | all elements | yes  | refer to parent element's font size | visual |
| [font-size-adjust](#propdef-font-size-adjust)               | none \| \<number\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | none                      | all elements | yes  | N/A                                 | visual |
| [font-stretch](#propdef-font-stretch)                       | normal \| ultra-condensed \| extra-condensed \| condensed \| semi-condensed \| semi-expanded \| expanded \| extra-expanded \| ultra-expanded                                                                                                                                                                                                                                                                                                                                            | normal                    | all elements | yes  | N/A                                 | visual |
| [font-style](#propdef-font-style)                           | normal \| italic \| oblique                                                                                                                                                                                                                                                                                                                                                                                                                                                             | normal                    | all elements | yes  | N/A                                 | visual |
| [font-synthesis](#propdef-font-synthesis)                   | none \| \[ weight \|\| style \]                                                                                                                                                                                                                                                                                                                                                                                                                                                         | weight style              | all elements | yes  | N/A                                 | visual |
| [font-variant](#propdef-font-variant)                       | normal \| none \| \[ \<common-lig-values\> \|\| \<discretionary-lig-values\> \|\| \<historical-lig-values\> \|\| \<contextual-alt-values\> \|\| \[ small-caps \| all-small-caps \| petite-caps \| all-petite-caps \| unicase \| titling-caps \] \|\| \<numeric-figure-values\> \|\| \<numeric-spacing-values\> \|\| \<numeric-fraction-values\> \|\| ordinal \|\| slashed-zero \|\| \<east-asian-variant-values\> \|\| \<east-asian-width-values\> \|\| ruby \|\| \[ sub \| super \] \] | normal                    | all elements | yes  | see individual properties           | visual |
| [font-variant-caps](#propdef-font-variant-caps)             | normal \| small-caps \| all-small-caps \| petite-caps \| all-petite-caps \| unicase \| titling-caps                                                                                                                                                                                                                                                                                                                                                                                     | normal                    | all elements | yes  | N/A                                 | visual |
| [font-variant-east-asian](#propdef-font-variant-east-asian) | normal \| \[ \<east-asian-variant-values\> \|\| \<east-asian-width-values\> \|\| ruby \]                                                                                                                                                                                                                                                                                                                                                                                                | normal                    | all elements | yes  | N/A                                 | visual |
| [font-variant-ligatures](#propdef-font-variant-ligatures)   | normal \| none \| \[ \<common-lig-values\> \|\| \<discretionary-lig-values\> \|\| \<historical-lig-values\> \|\| \<contextual-alt-values\> \]                                                                                                                                                                                                                                                                                                                                           | normal                    | all elements | yes  | N/A                                 | visual |
| [font-variant-numeric](#propdef-font-variant-numeric)       | normal \| \[ \<numeric-figure-values\> \|\| \<numeric-spacing-values\> \|\| \<numeric-fraction-values\> \|\| ordinal \|\| slashed-zero \]                                                                                                                                                                                                                                                                                                                                               | normal                    | all elements | yes  | N/A                                 | visual |
| [font-variant-position](#propdef-font-variant-position)     | normal \| sub \| super                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | normal                    | all elements | yes  | N/A                                 | visual |
| [font-weight](#propdef-font-weight)                         | normal \| bold \| bolder \| lighter \| 100 \| 200 \| 300 \| 400 \| 500 \| 600 \| 700 \| 800 \| 900                                                                                                                                                                                                                                                                                                                                                                                      | normal                    | all elements | yes  | N/A                                 | visual |

| Descriptor                                              | Values                                                                                                                                       | Initial    |
|---------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------|------------|
| [font-family](#descdef-font-family)                     | \<family-name\>                                                                                                                              | N/A        |
| [font-feature-settings](#descdef-font-feature-settings) | normal \| \<feature-tag-value\> \#                                                                                                           | normal     |
| [font-stretch](#descdef-font-stretch)                   | normal \| ultra-condensed \| extra-condensed \| condensed \| semi-condensed \| semi-expanded \| expanded \| extra-expanded \| ultra-expanded | normal     |
| [font-style](#descdef-font-style)                       | normal \| italic \| oblique                                                                                                                  | normal     |
| [font-weight](#descdef-font-weight)                     | normal \| bold \| 100 \| 200 \| 300 \| 400 \| 500 \| 600 \| 700 \| 800 \| 900                                                                | normal     |
| [src](#descdef-src)                                     | \[ \<url\> \[format(\<string\> \#)\]? \| \<font-face-name\> \] \#                                                                            | N/A        |
| [unicode-range](#descdef-unicode-range)                 | \<urange\> \#                                                                                                                                | U+0-10FFFF |
