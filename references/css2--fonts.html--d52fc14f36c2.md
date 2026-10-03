Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Fonts](https://www.w3.org/TR/2011/REC-CSS2-20110607/fonts.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Fonts

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/fonts.html

Snapshot SHA-256: d52fc14f36c26c6afad471e298a8e7d574d86030bfe3b97dcfb46e89a6d1d1b1

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.

---

<a id="q15.0"></a>

# 15 Fonts

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<a id="fonts-intro"></a>

## 15.1 Introduction

Setting font properties will be among the most common uses of style sheets. Unfortunately, there exists no well-defined and universally accepted taxonomy for classifying fonts, and terms that apply to one font family may not be appropriate for others. E.g., 'italic' is commonly used to label slanted text, but slanted text may also be labeled as being <em>Oblique, Slanted, Incline, Cursive</em> or <em>Kursiv</em>. Therefore it is not a simple problem to map typical font selection properties to a specific font.

<a id="algorithm"></a>

## 15.2 Font matching algorithm

Because there is no accepted, universal taxonomy of font properties, matching of properties to font faces must be done carefully. The properties are matched in a well-defined order to insure that the results of this matching process are as consistent as possible across UAs (assuming that the same library of font faces is presented to each of them).

1.  The User Agent makes (or accesses) a database of relevant CSS 2.1 properties of all the fonts of which the UA is aware. If there are two fonts with exactly the same properties, the user agent selects one of them.
2.  At a given element and for each character in that element, the UA assembles the font properties applicable to that element. Using the complete set of properties, the UA uses the 'font-family' property to choose a tentative font family. The remaining properties are tested against the family according to the matching criteria described with each property. If there are matches for all the remaining properties, then that is the matching font face for the given element or character.
3.  If there is no matching font face within the 'font-family' being processed by step 2, and if there is a next alternative 'font-family' in the font set, then repeat step 2 with the next alternative 'font-family'.
4.  If there is a matching font face, but it does not contain a glyph for the current character, and if there is a next alternative 'font-family' in the font sets, then repeat step 2 with the next alternative 'font-family'.
5.  If there is no font within the family selected in 2, then use a UA-dependent default 'font-family' and repeat step 2, using the best match that can be obtained within the default font. If a particular character cannot be displayed using this font, then the UA may use other means to determine a suitable font for that character. The UA should map each character for which it has no suitable font to a visible symbol chosen by the UA, preferably a "missing character" glyph from one of the font faces available to the UA.

(The above algorithm can be optimized to avoid having to revisit the CSS 2.1 properties for each character.)

The per-property matching rules from (2) above are as follows:

1.  ['font-style'](css2--fonts.html--d52fc14f36c2.md#propdef-font-style) is tried first. 'Italic' will be satisfied if there is either a face in the UA's font database labeled with the CSS keyword 'italic' (preferred) or 'oblique'. Otherwise the values must be matched exactly or font-style will fail.
2.  ['font-variant'](css2--fonts.html--d52fc14f36c2.md#propdef-font-variant) is tried next. 'Small-caps' matches (1) a font labeled as 'small-caps', (2) a font in which the small caps are synthesized, or (3) a font where all lowercase letters are replaced by upper case letters. A small-caps font may be synthesized by electronically scaling uppercase letters from a normal font. 'normal' matches a font's normal (non-small-caps) variant. A font cannot fail to have a normal variant. A font that is only available as small-caps shall be selectable as either a 'normal' face or a 'small-caps' face.
3.  ['font-weight'](css2--fonts.html--d52fc14f36c2.md#propdef-font-weight) is matched next, it will never fail. (See 'font-weight' below.)
4.  ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size) must be matched within a UA-dependent margin of tolerance. (Typically, sizes for scalable fonts are rounded to the nearest whole pixel, while the tolerance for bitmapped fonts could be as large as 20%.) Further computations, e.g., by 'em' values in other properties, are based on the computed value of 'font-size'.

<a id="font-family-prop"></a>

## 15.3 Font family: the ['font-family'](css2--fonts.html--d52fc14f36c2.md#propdef-font-family) property

<a id="propdef-font-family"></a>

<strong>'font-family'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
|-----------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[\[ [\<family-name\>](css2--fonts.html--d52fc14f36c2.md#value-def-family-name) \| [\<generic-family\>](css2--fonts.html--d52fc14f36c2.md#value-def-generic-family) \] \[, [\<family-name\>](css2--fonts.html--d52fc14f36c2.md#value-def-family-name)\| [\<generic-family\>](css2--fonts.html--d52fc14f36c2.md#value-def-generic-family)\]\* \] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | depends on user agent                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <em>Computed value:</em>   | as specified                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |

The property value is a prioritized list of font family names and/or [generic family names.](#generic-font-families) Unlike most other CSS properties, component values are separated by a comma to indicate that they are alternatives:

```text

body { font-family: Gill, Helvetica, sans-serif }
```
Although many fonts provide the "missing character" glyph, typically an open box, as its name implies this should not be considered a match for characters that cannot be found in the font. (It should, however, be considered a match for U+FFFD, the "missing character" character's code point).

There are two types of font family names:

<a id="value-def-family-name"></a>

\<family-name\>

The name of a font family of choice. In the last example, "Gill" and "Helvetica" are font families.

<a id="value-def-generic-family"></a>

\<generic-family\>

In the example above, the last value is a generic family name. The following generic families are defined:

- 'serif' (e.g., Times)
- 'sans-serif' (e.g., Helvetica)
- 'cursive' (e.g., Zapf-Chancery)
- 'fantasy' (e.g., Western)
- 'monospace' (e.g., Courier)

Style sheet designers are encouraged to offer a generic font family as a last alternative. Generic font family names are keywords and must NOT be quoted.

Font family names must either be given quoted as [strings,](css2--syndata.html--02e71c159e14.md#strings) or unquoted as a sequence of one or more [identifiers.](css2--syndata.html--02e71c159e14.md#value-def-identifier) This means most punctuation characters and digits at the start of each token must be escaped in unquoted font family names.

For example, the following declarations are invalid:

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
Font family <em>names</em> that happen to be the same as a keyword value ('inherit', 'serif', 'sans-serif', 'monospace', 'fantasy', and 'cursive') must be quoted to prevent confusion with the keywords with the same names. The keywords 'initial' and 'default' are reserved for future use and must also be quoted when used as font names. UAs must not consider these keywords as matching the '\<family-name\>' type.

<a id="generic-font-families"></a>

### 15.3.1 Generic font families

Generic font families are a fallback mechanism, a means of preserving some of the style sheet author's intent in the worst case when none of the specified fonts can be selected. For optimum typographic control, particular named fonts should be used in style sheets.

<a id="defined-to-exist"></a>

All five generic font families are defined to exist in all CSS implementations (they need not necessarily map to five distinct actual fonts). User agents should provide reasonable default choices for the generic font families, which express the characteristics of each family as well as possible within the limits allowed by the underlying technology.

User agents are encouraged to allow users to select alternative choices for the generic fonts.

<a id="serif-def"></a>

#### 15.3.1.1 serif

Glyphs of serif fonts, as the term is used in CSS, tend to have finishing strokes, flared or tapering ends, or have actual serifed endings (including slab serifs). Serif fonts are typically proportionately-spaced. They often display a greater variation between thick and thin strokes than fonts from the 'sans-serif' generic font family. CSS uses the term 'serif' to apply to a font for any script, although other names may be more familiar for particular scripts, such as Mincho (Japanese), Sung or Song (Chinese), Totum or Kodig (Korean). Any font that is so described may be used to represent the generic 'serif' family.

Examples of fonts that fit this description include:

|                |                                                                                                        |
|----------------|--------------------------------------------------------------------------------------------------------|
| Latin fonts    | Times New Roman, Bodoni, Garamond, Minion Web, ITC Stone Serif, MS Georgia, Bitstream Cyberbit         |
| Greek fonts    | Bitstream Cyberbit                                                                                     |
| Cyrillic fonts | Adobe Minion Cyrillic, Excelsior Cyrillic Upright, Monotype Albion 70, Bitstream Cyberbit, ER Bukinist |
| Hebrew fonts   | New Peninim, Raanana, Bitstream Cyberbit                                                               |
| Japanese fonts | Ryumin Light-KL, Kyokasho ICA, Futo Min A101                                                           |
| Arabic fonts   | Bitstream Cyberbit                                                                                     |
| Cherokee fonts | Lo Cicero Cherokee                                                                                     |

<a id="sans-serif-def"></a>

#### 15.3.1.2  sans-serif

Glyphs in sans-serif fonts, as the term is used in CSS, tend to have stroke endings that are plain -- with little or no flaring, cross stroke, or other ornamentation. Sans-serif fonts are typically proportionately-spaced. They often have little variation between thick and thin strokes, compared to fonts from the 'serif' family. CSS uses the term 'sans-serif' to apply to a font for any script, although other names may be more familiar for particular scripts, such as Gothic (Japanese), Kai (Chinese), or Pathang (Korean). Any font that is so described may be used to represent the generic 'sans-serif' family.

Examples of fonts that fit this description include:

|                |                                                                                                                                     |
|----------------|-------------------------------------------------------------------------------------------------------------------------------------|
| Latin fonts    | MS Trebuchet, ITC Avant Garde Gothic, MS Arial, MS Verdana, Univers, Futura, ITC Stone Sans, Gill Sans, Akzidenz Grotesk, Helvetica |
| Greek fonts    | Attika, Typiko New Era, MS Tahoma, Monotype Gill Sans 571, Helvetica Greek                                                          |
| Cyrillic fonts | Helvetica Cyrillic, ER Univers, Lucida Sans Unicode, Bastion                                                                        |
| Hebrew fonts   | Arial Hebrew, MS Tahoma                                                                                                             |
| Japanese fonts | Shin Go, Heisei Kaku Gothic W5                                                                                                      |
| Arabic fonts   | MS Tahoma                                                                                                                           |

<a id="cursive-def"></a>

#### 15.3.1.3  cursive

Glyphs in cursive fonts, as the term is used in CSS, generally have either joining strokes or other cursive characteristics beyond those of italic typefaces. The glyphs are partially or completely connected, and the result looks more like handwritten pen or brush writing than printed letterwork. Fonts for some scripts, such as Arabic, are almost always cursive. CSS uses the term 'cursive' to apply to a font for any script, although other names such as Chancery, Brush, Swing and Script are also used in font names.

Examples of fonts that fit this description include:

|                |                                                                                   |
|----------------|-----------------------------------------------------------------------------------|
| Latin fonts    | Caflisch Script, Adobe Poetica, Sanvito, Ex Ponto, Snell Roundhand, Zapf-Chancery |
| Cyrillic fonts | ER Architekt                                                                      |
| Hebrew fonts   | Corsiva                                                                           |
| Arabic fonts   | DecoType Naskh, Monotype Urdu 507                                                 |

<a id="fantasy-def"></a>

#### 15.3.1.4  fantasy

Fantasy fonts, as used in CSS, are primarily decorative while still containing representations of characters (as opposed to Pi or Picture fonts, which do not represent characters). Examples include:

|             |                                                           |
|-------------|-----------------------------------------------------------|
| Latin fonts | Alpha Geometrique, Critter, Cottonwood, FB Reactor, Studz |

<a id="monospace-def"></a>

#### 15.3.1.5  monospace

The sole criterion of a monospace font is that all glyphs have the same fixed width. (This can make some scripts, such as Arabic, look most peculiar.) The effect is similar to a manual typewriter, and is often used to set samples of computer code.

Examples of fonts which fit this description include:

|                |                                                 |
|----------------|-------------------------------------------------|
| Latin fonts    | Courier, MS Courier New, Prestige, Everson Mono |
| Greek Fonts    | MS Courier New, Everson Mono                    |
| Cyrillic fonts | ER Kurier, Everson Mono                         |
| Japanese fonts | Osaka Monospaced                                |
| Cherokee fonts | Everson Mono                                    |

<a id="font-styling"></a>

## 15.4 Font styling: the ['font-style'](css2--fonts.html--d52fc14f36c2.md#propdef-font-style) property

<a id="propdef-font-style"></a>

<strong>'font-style'</strong>

|                       |                                                                                                                       |
|-----------------------|-----------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | normal \| italic \| oblique \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | normal                                                                                                                |
| <em>Applies to:</em>   | all elements                                                                                                          |
| <em>Inherited:</em>   | yes                                                                                                                   |
| <em>Percentages:</em>   | N/A                                                                                                                   |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                  |
| <em>Computed value:</em>   | as specified                                                                                                          |

The 'font-style' property selects between normal (sometimes referred to as "roman" or "upright"), italic and oblique faces within a font family.

A value of 'normal' selects a font that is classified as 'normal' in the UA's font database, while 'oblique' selects a font that is labeled 'oblique'. A value of 'italic' selects a font that is labeled 'italic', or, if that is not available, one labeled 'oblique'.

The font that is labeled 'oblique' in the UA's font database may actually have been generated by electronically slanting a normal font.

Fonts with Oblique, Slanted or Incline in their names will typically be labeled 'oblique' in the UA's font database. Fonts with <em>Italic, Cursive</em> or <em>Kursiv</em> in their names will typically be labeled 'italic'.

```text

h1, h2, h3 { font-style: italic }
h1 em { font-style: normal }
```
In the example above, emphasized text within 'H1' will appear in a normal face.

<a id="small-caps"></a>

## 15.5 Small-caps: the ['font-variant'](css2--fonts.html--d52fc14f36c2.md#propdef-font-variant) property

<a id="propdef-font-variant"></a>

<strong>'font-variant'</strong>

|                       |                                                                                                                |
|-----------------------|----------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | normal \| small-caps \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | normal                                                                                                         |
| <em>Applies to:</em>   | all elements                                                                                                   |
| <em>Inherited:</em>   | yes                                                                                                            |
| <em>Percentages:</em>   | N/A                                                                                                            |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                           |
| <em>Computed value:</em>   | as specified                                                                                                   |

Another type of variation within a font family is the small-caps. In a small-caps font the lower case letters look similar to the uppercase ones, but in a smaller size and with slightly different proportions. The 'font-variant' property selects that font.

A value of 'normal' selects a font that is not a small-caps font, 'small-caps' selects a small-caps font. It is acceptable (but not required) in CSS 2.1 if the small-caps font is a created by taking a normal font and replacing the lower case letters by scaled uppercase characters. As a last resort, uppercase letters will be used as replacement for a small-caps font.

The following example results in an 'H3' element in small-caps, with any emphasized words in oblique, and any emphasized words within an 'H3' oblique small-caps:

```text

h3 { font-variant: small-caps }
em { font-style: oblique }
```
There may be other variants in the font family as well, such as fonts with old-style numerals, small-caps numerals, condensed or expanded letters, etc. CSS 2.1 has no properties that select those.

<em>Note:</em> insofar as this property causes text to be transformed to uppercase, the same considerations as for ['text-transform'](css2--text.html--467a8857ae69.md#propdef-text-transform) apply.

<a id="font-boldness"></a>

## 15.6 Font boldness: the ['font-weight'](css2--fonts.html--d52fc14f36c2.md#propdef-font-weight) property

<a id="propdef-font-weight"></a>

<strong>'font-weight'</strong>

|                       |                                                                                                                                                                                              |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | normal \| bold \| bolder \| lighter \| 100 \| 200 \| 300 \| 400 \| 500 \| 600 \| 700 \| 800 \| 900 \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | normal                                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                 |
| <em>Inherited:</em>   | yes                                                                                                                                                                                          |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                          |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                         |
| <em>Computed value:</em>   | see text                                                                                                                                                                                     |

The 'font-weight' property selects the weight of the font. The values '100' to '900' form an ordered sequence, where each number indicates a weight that is at least as dark as its predecessor. The keyword 'normal' is synonymous with '400', and 'bold' is synonymous with '700'. Keywords other than 'normal' and 'bold' have been shown to be often confused with font names and a numerical scale was therefore chosen for the 9-value list.

```text

p { font-weight: normal }   /* 400 */
h1 { font-weight: 700 }     /* bold */
```
The 'bolder' and 'lighter' values select font weights that are relative to the weight inherited from the parent:

```text

strong { font-weight: bolder }
```
Fonts (the font data) typically have one or more properties whose values are names that are descriptive of the "weight" of a font. There is no accepted, universal meaning to these weight names. Their primary role is to distinguish faces of differing darkness within a single font family. Usage across font families is quite variant; for example, a font that one might think of as being bold might be described as being <em>Regular, Roman, Book, Medium, Semi-</em> or <em>DemiBold,
Bold,</em> or <em>Black,</em> depending on how black the "normal" face of the font is within the design. Because there is no standard usage of names, the weight property values in CSS 2.1 are given on a numerical scale in which the value '400' (or 'normal') corresponds to the "normal" text face for that family. The weight name associated with that face will typically be <em>Book, Regular, Roman, Normal</em> or sometimes <em>Medium</em>.

The association of other weights within a family to the numerical weight values is intended only to preserve the ordering of darkness within that family. However, the following heuristics tell how the assignment is done in this case:

- If the font family already uses a numerical scale with nine values (like e.g., <em>OpenType</em> does), the font weights should be mapped directly.
- If there is both a face labeled <em>Medium</em> and one labeled <em>Book, Regular, Roman</em> or <em>Normal,</em> then the <em>Medium</em> is normally assigned to the '500'.
- The font labeled "Bold" will often correspond to the weight value '700'.

Once the font family's weights are mapped onto the CSS scale, missing weights are selected as follows:

- If the desired weight is less than 400, weights below the desired weight are checked in descending order followed by weights above the desired weight in ascending order until a match is found.
- If the desired weight is greater than 500, weights above desired weight are checked in ascending order followed by weights below the desired weight in descending order until a match is found.
- If the desired weight is 400, 500 is checked first and then the rule for desired weights less than 400 is used.
- If the desired weight is 500, 400 is checked first and then the rule for desired weights less than 400 is used.

The following two examples show typical mappings.

Assume four weights in the "Rattlesnake" family, from lightest to darkest: <em>Regular, Medium, Bold, Heavy.</em>

| Available faces       | Assignments | Filling the holes |
|-----------------------|-------------|-------------------|
| "Rattlesnake Regular" | 400         | 100, 200, 300     |
| "Rattlesnake Medium"  | 500         |                   |
| "Rattlesnake Bold"    | 700         | 600               |
| "Rattlesnake Heavy"   | 800         | 900               |

First example of font-weight mapping

Assume six weights in the "Ice Prawn" family: <em>Book, Medium, Bold, Heavy, Black,
ExtraBlack.</em> Note that in this instance the user agent has decided <em>not</em> to assign a numeric value to "Ice Prawn ExtraBlack".

| Available faces        | Assignments | Filling the holes |
|------------------------|-------------|-------------------|
| "Ice Prawn Book"       | 400         | 100, 200, 300     |
| "Ice Prawn Medium"     | 500         |                   |
| "Ice Prawn Bold"       | 700         | 600               |
| "Ice Prawn Heavy"      | 800         |                   |
| "Ice Prawn Black"      | 900         |                   |
| "Ice Prawn ExtraBlack" | (none)      |                   |

Second example of font-weight mapping

Values of 'bolder' and 'lighter' indicate values relative to the weight of the parent element. Based on the inherited weight value, the weight used is calculated using the chart below. Child elements inherit the calculated weight, not a value of 'bolder' or 'lighter'.

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

The meaning of 'bolder' and 'lighter'

The table above is equivalent to selecting the next relative bolder or lighter face, given a font family containing normal and bold faces along with a thin and a heavy face. Authors who desire finer control over the exact weight values used for a given element should use numerical values instead of relative weights.

There is no guarantee that there will be a darker face for each of the 'font-weight' values; for example, some fonts may have only a normal and a bold face, while others may have eight face weights. There is no guarantee on how a UA will map font faces within a family to weight values. The only guarantee is that a face of a given value will be no less dark than the faces of lighter values.

<a id="font-size-props"></a>

## 15.7 Font size: the ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size) property

<a id="propdef-font-size"></a>

<strong>'font-size'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<absolute-size\>](css2--fonts.html--d52fc14f36c2.md#value-def-absolute-size) \| [\<relative-size\>](css2--fonts.html--d52fc14f36c2.md#value-def-relative-size) \| [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | medium                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <em>Percentages:</em>   | refer to inherited font size                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <em>Computed value:</em>   | absolute length                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |

The font size corresponds to the em square, a concept used in typography. Note that certain glyphs may bleed outside their em squares. Values have the following meanings:

<a id="value-def-absolute-size"></a>

<b><a>&lt;absolute-size&gt;</a></b>

An \<absolute-size\> keyword is an index to a table of font sizes computed and kept by the UA. Possible values are:

\[ xx-small \| x-small \| small \| medium \| large \| x-large \| xx-large \]

The following table provides user agent guidelines for the absolute-size mapping to HTML heading and absolute font-sizes. The 'medium' value is the user's preferred font size and is used as the reference middle value.

<a id="AutoNumber2"></a>

|                          |          |         |       |        |       |         |          |     |
|--------------------------|----------|---------|-------|--------|-------|---------|----------|-----|
| CSS absolute-size values | xx-small | x-small | small | medium | large | x-large | xx-large |     |
| HTML font sizes          | 1        |         | 2     | 3      | 4     | 5       | 6        | 7   |

Implementors should build a table of scaling factors for absolute-size keywords relative to the 'medium' font size and the particular device and its characteristics (e.g., the resolution of the device).

Different media may need different scaling factors. Also, the UA should take the quality and availability of fonts into account when computing the table. The table may be different from one font family to another.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note 1.</strong> To preserve readability, a UA applying
these guidelines should nevertheless avoid creating font-size resulting
 in less than 9 pixels per EM unit on a computer display.</em>

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note 2.</strong> In CSS1, the suggested
scaling factor between adjacent indexes was 1.5, which user experience
proved to be too large. In CSS2, the suggested scaling factor for a
computer screen between adjacent indexes was 1.2, which still created
issues for the small sizes. Implementation experience has demonstrated
that a fixed ratio between adjacent absolute-size keywords is
problematic, and this specification does <em>not</em> recommend such a
fixed ratio.</em>

<a id="value-def-relative-size"></a>

<b><a>&lt;relative-size&gt;</a></b>

A \<relative-size\> keyword is interpreted relative to the table of font sizes and the font size of the parent element. Possible values are: \[ larger \| smaller \]. For example, if the parent element has a font size of 'medium', a value of 'larger' will make the font size of the current element be 'large'. If the parent element's size is not close to a table entry, the UA is free to interpolate between table entries or round off to the closest one. The UA may have to extrapolate table values if the numerical value goes beyond the keywords.

Length and percentage values should not take the font size table into account when calculating the font size of the element.

Negative values are not allowed.

On all other properties, 'em' and 'ex' length values refer to the computed font size of the current element. On the 'font-size' property, these length units refer to the computed font size of the parent element.

Note that an application may reinterpret an explicit size, depending on the context. E.g., inside a VR scene a font may get a different size because of perspective distortion.

Examples:

```text

p { font-size: 16px; }
@media print {
	p { font-size: 12pt; }
}
blockquote { font-size: larger }
em { font-size: 150% }
em { font-size: 1.5em }
```
<a id="font-shorthand"></a>

## 15.8 Shorthand font property: the ['font'](css2--fonts.html--d52fc14f36c2.md#propdef-font) property

<a id="propdef-font"></a>

<strong>'font'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[ \[ [\<'font-style'\>](css2--fonts.html--d52fc14f36c2.md#propdef-font-style) \|\| [\<'font-variant'\>](css2--fonts.html--d52fc14f36c2.md#propdef-font-variant) \|\| [\<'font-weight'\>](css2--fonts.html--d52fc14f36c2.md#propdef-font-weight) \]? [\<'font-size'\>](css2--fonts.html--d52fc14f36c2.md#propdef-font-size) \[ / [\<'line-height'\>](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height) \]? [\<'font-family'\>](css2--fonts.html--d52fc14f36c2.md#propdef-font-family) \] \| caption \| icon \| menu \| message-box \| small-caption \| status-bar \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <em>Percentages:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |

The ['font'](css2--fonts.html--d52fc14f36c2.md#propdef-font) property is, except as described [below](#almost), a shorthand property for setting ['font-style'](css2--fonts.html--d52fc14f36c2.md#propdef-font-style), ['font-variant'](css2--fonts.html--d52fc14f36c2.md#propdef-font-variant), ['font-weight'](css2--fonts.html--d52fc14f36c2.md#propdef-font-weight), ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size), ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height) and ['font-family'](css2--fonts.html--d52fc14f36c2.md#propdef-font-family) at the same place in the style sheet. The syntax of this property is based on a traditional typographical shorthand notation to set multiple properties related to fonts.

All font-related properties are first reset to their initial values, including those listed in the preceding paragraph. Then, those properties that are given explicit values in the ['font'](css2--fonts.html--d52fc14f36c2.md#propdef-font) shorthand are set to those values. For a definition of allowed and initial values, see the previously defined properties.

```text

p { font: 12px/14px sans-serif }
p { font: 80% sans-serif }
p { font: x-large/110% "New Century Schoolbook", serif }
p { font: bold italic large Palatino, serif }
p { font: normal small-caps 120%/120% fantasy }
```
In the second rule, the font size percentage value ('80%') refers to the font size of the parent element. In the third rule, the line height percentage refers to the font size of the element itself.

In the first three rules above, the 'font-style', 'font-variant' and 'font-weight' are not explicitly mentioned, which means they are all three set to their initial value ('normal'). The fourth rule sets the 'font-weight' to 'bold', the 'font-style' to 'italic' and implicitly sets 'font-variant' to 'normal'.

The fifth rule sets the 'font-variant' ('small-caps'), the 'font-size' (120% of the parent's font), the 'line-height' (120% times the font size) and the 'font-family' ('fantasy'). It follows that the keyword 'normal' applies to the two remaining properties: 'font-style' and 'font-weight'.

<a id="x11"></a>

The following values refer to system fonts:

caption  
The font used for captioned controls (e.g., buttons, drop-downs, etc.).

icon  
The font used to label icons.

menu  
The font used in menus (e.g., dropdown menus and menu lists).

message-box  
The font used in dialog boxes.

small-caption  
The font used for labeling small controls.

status-bar  
The font used in window status bars.

System fonts may only be set as a whole; that is, the font family, size, weight, style, etc. are all set at the same time. These values may then be altered individually if desired. If no font with the indicated characteristics exists on a given platform, the user agent should either intelligently substitute (e.g., a smaller version of the 'caption' font might be used for the 'small-caption' font), or substitute a user agent default font. As for regular fonts, if, for a system font, any of the individual properties are not part of the operating system's available user preferences, those properties should be set to their initial values.

<a id="almost"></a>That is why this property is "almost" a shorthand property: system fonts can only be specified with this property, not with ['font-family'](css2--fonts.html--d52fc14f36c2.md#propdef-font-family) itself, so ['font'](css2--fonts.html--d52fc14f36c2.md#propdef-font) allows authors to do more than the sum of its subproperties. However, the individual properties such as ['font-weight'](css2--fonts.html--d52fc14f36c2.md#propdef-font-weight) are still given values taken from the system font, which can be independently varied.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> button { font: 300 italic 1.3em/1.7em "FB Armada", sans-serif }
> button p { font: menu }
> button p em { font-weight: bolder }
> ```
>
> If the font used for dropdown menus on a particular system happened to be, for example, 9-point Charcoal, with a weight of 600, then P elements that were descendants of BUTTON would be displayed as if this rule were in effect:
>
> ```text
> 
> button p { font: 600 9px Charcoal }
> ```
>
> Because the ['font'](css2--fonts.html--d52fc14f36c2.md#propdef-font) shorthand property resets any property not explicitly given a value to its initial value, this has the same effect as this declaration:
>
> ```text
> 
> button p {
>   font-family: Charcoal;
>   font-style: normal;
>   font-variant: normal;
>   font-weight: 600;
>   font-size: 9px;
>   line-height: normal;
> }
> ```