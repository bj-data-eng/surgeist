Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Changes](https://www.w3.org/TR/2011/REC-CSS2-20110607/changes.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Changes

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/changes.html

Snapshot SHA-256: 830f199c671eea9ff2626fed1161bc4b5cbc6eb536be1f6b663cbf6fea13b589

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="q21.0"></a>

# Appendix C. Changes

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<em>This appendix is informative, not normative.</em>

CSS 2.1 is an updated revision of CSS2. The changes between the CSS2 specification (see [\[CSS2\]](css2--refs.html--f208d881d0b7.md#ref-CSS2)) and this specification fall into five groups: [known errors,](#known-errors) typographical errors, [clarifications,](#clarifications) [changes](#changes) and [additions.](#new) Typographical errors are not listed here.

In addition, this chapter lists the [errata (part 1](#errata) and [part 2)](#errata2) that were subsequently applied to CSS 2.1 since it became a Candidate Recommendation in July 2007.

This chapter is not a complete list of changes. Minor editorial changes and most changes to examples are also not listed here.

<a id="new"></a>

## C.1 Additional property values

<a id="q21.2"></a>

### <a id="a4.3.6"></a>C.1.1 [Section 4.3.6 Colors](css2--syndata.html--02e71c159e14.md#color-units)

New color value: 'orange'

<a id="q21.3"></a>

### <a id="a9.2.4"></a>C.1.2 [Section 9.2.4 The 'display' property](css2--visuren.html--3f334c530cf4.md#propdef-display)

New 'display' value: 'inline-block'

<a id="q21.4"></a>

### <a id="a12.2"></a>C.1.3 [Section 12.2 The 'content' property](css2--generate.html--37748b674cd2.md#content)

New 'content' values 'none' and 'normal'. (The values 'none' and 'normal' are equivalent in CSS 2.1, but may have different functions in CSS3.)

<a id="q21.5"></a>

### <a id="a16.6"></a>C.1.4 [Section 16.6 White space: the 'white-space' property](css2--text.html--467a8857ae69.md#propdef-white-space)

New 'white-space' values: 'pre-wrap' and 'pre-line'

<a id="q21.6"></a>

### <a id="a18.1"></a>C.1.5 [Section 18.1 Cursors: the 'cursor' property](css2--ui.html--ff1f72803ea0.md#propdef-cursor)

New 'cursor' value: 'progress'

<a id="changes"></a>

## C.2 Changes

<a id="q21.8"></a>

### <a id="c1.1"></a>C.2.1 [Section 1.1 CSS 2.1 vs CSS 2](css2--about.html--c67ff594c990.md#css2.1-v-css2)

This new section is added to explain the motivation for CSS2.1 and its relation to CSS2.

<a id="q21.9"></a>

### <a id="c1.2"></a>C.2.2 [Section 1.2 Reading the specification](css2--about.html--c67ff594c990.md#reading)

This section (formerly Section 1.1) has been marked non-normative.

<a id="q21.10"></a>

### <a id="c1.3"></a>C.2.3 [Section 1.3 How the specification is organized](css2--about.html--c67ff594c990.md#organization)

This section (formerly Section 1.2) has been marked non-normative.

<a id="q21.11"></a>

### <a id="c1.4.2.1"></a>C.2.4 [Section 1.4.2.1 Value](css2--about.html--c67ff594c990.md#value-defs)

This section (formerly unnumbered under 1.3.2) notes that value types are specified in terms of tokens and that spaces may appear between tokens in values. A note explains that spaces are required between some tokens.

<a id="q21.12"></a>

### <a id="c1.4.2.6"></a>C.2.5 [Section 1.4.2.6 Media groups](css2--about.html--c67ff594c990.md#media-applies)

This section (formerly unnumbered under 1.3.2) now declares the Media line in property definitions to be non-normative.

<a id="q21.13"></a>

### <a id="c1.4.2.7"></a>C.2.6 [Section 1.4.2.7 Computed value](css2--about.html--c67ff594c990.md#computed-defs)

A new line is added to each property definition specifying what the computed values are for the property. (This defines what level of computation is done to a property value before inheritance and before certain other calculations.)

<a id="q21.14"></a>

### <a id="c1.4.4"></a>C.2.7 [Section 1.4.4 Notes and examples](css2--about.html--c67ff594c990.md#notes-and-examples)

This section (formerly 1.3.4) now specifies that HTML examples lacking DOCTYPE declarations are SGML Text Entities conforming to the HTML 4.01 Strict DTD \[HTML4\]. The markup for many examples has been reformulated to either include a DOCTYPE or conform to this definition.

<a id="q21.15"></a>

### <a id="c1.5"></a>C.2.8 [Section 1.5 Acknowledgments](css2--about.html--c67ff594c990.md#acknowledgements)

This section (formerly 1.4) has been updated to reflect contributions to CSS2.1 and has been marked non-normative.

<a id="q21.16"></a>

### <a id="c3.2"></a>C.2.9 [Section 3.2 Conformance](css2--conform.html--de58593b67d7.md#conformance)

Support for user style sheets is now required (in most cases), rather than just recommended.

Support for turning of author style sheets is now required.

Application of CSS properties to form controls is explicitly undefined. Authors are recommended to treat form control styling capabilities in UAs as experimental.

<a id="q21.17"></a>

### <a id="c3.3"></a>C.2.10 [Section 3.3 Error Conditions](css2--conform.html--de58593b67d7.md#errors)

This section changed to say that error handling <em>is</em> specified in most cases.

<a id="q21.18"></a>

### <a id="c4.1.1"></a>C.2.11 [Section 4.1.1 Tokenization](css2--syndata.html--02e71c159e14.md#tokenization)

Added INVALID token and rules for its definition.

An optional hyphen, "-", is now allowed at the beginning of an "ident" for vendor extensions. (See section 4.1.2.1)

<a id="underscore"></a>The underscore character ("\_") is allowed in identifiers. The definitions of the lexical macros "nmstart" and "nmchar" now include it. See also section 4.1.2.1 (Vendor extensions).

The "escape" macro has been modified to allow the escaping of any character except newlines, form feeds, and hex digits (to avoid conflict with Unicode escapes).

Modified "string1" and "string2" macros by defining allowed characters through excluding disallowed characters. This allows invisible ASCII characters to be included in a string.

<a id="q21.19"></a>

### <a id="c4.1.3"></a>C.2.12 [Section 4.1.3 Characters and case](css2--syndata.html--02e71c159e14.md#characters)

Updated prose about identifiers (second bullet point) to match changes in the tokenization (above).

Excluded null (0x0) character from CSS numerical escapes and indicate that it is undefined in CSS2.1 what happens if such a character is encountered.

Allowed the use of U+FFFD as a replacement for characters outside the range allowed by Unicode.

CSS is no longer case-insensitive, but case-sensitive with exceptions. Changed "All CSS style sheets are case-insensitive, except for parts that are not under the control of CSS" to "All CSS syntax is case-insensitive within the ASCII range (i.e., \[a-z\] and \[A-Z\] are equivalent), except for parts that are not under the control of CSS." See also the change to case-sensitivity of counters in [4.3.5.](#c4.3.5)

<a id="q21.20"></a>

### <a id="c4.2"></a>C.2.13 [Section 4.2 Rules for handling parsing errors](css2--syndata.html--02e71c159e14.md#parsing-errors)

Defined parsing in the cases of Malformed Declarations, Unexpected End of Stylesheet, and Unexpected End of String.

<a id="q21.21"></a>

### <a id="c4.3"></a>C.2.14 [Section 4.3 Values](css2--syndata.html--02e71c159e14.md#values)

Sections 4.3.7 (Angles), 4.3.8 (Times), and 4.3.9 (Frequencies) have been moved to the informative Appendix A.

<a id="q21.22"></a>

### <a id="c4.3.2"></a>C.2.15 [Section 4.3.2 Lengths](css2--syndata.html--02e71c159e14.md#length-units)

Added a paragraph on heuristics for finding the x-height of a font.

<a id="q21.23"></a>

### <a id="c4.3.4"></a>C.2.16 [Section 4.3.4 URLs and URIs](css2--syndata.html--02e71c159e14.md#uri)

Updated URI references to RFC3986.

<a id="q21.24"></a>

### <a id="c4.3.5"></a>C.2.17 [Section 4.3.5 Counters](css2--syndata.html--02e71c159e14.md#counter)

Changed "Counters are denoted by identifiers" to "Counters are denoted by <u>case-sensitive</u> identifiers" (see also the change to case-sensitivity in [4.1.3](#c4.1.3)).

<a id="q21.25"></a>

### <a id="c4.3.6"></a>C.2.18 [Section 4.3.6 Colors](css2--syndata.html--02e71c159e14.md#color-units)

Defined the numeric values corresponding to color keywords instead of referencing HTML4 for those values.

UAs are now allowed to intelligently map colors outside the gamut into the gamut instead of simply clipping them into the range of the gamut.

<a id="q21.26"></a>

### <a id="c4.3.8"></a>C.2.19 [Section 4.3.8 Unsupported Values](css2--syndata.html--02e71c159e14.md#unsupported-values)

Added this section to recommend that unsupported properties and values be ignored as if they were invalid.

<a id="q21.27"></a>

### <a id="c4.4"></a>C.2.20 [Section 4.4 CSS style sheet representation](css2--syndata.html--02e71c159e14.md#charset)

Changed character encoding detection rule 2 to include a BOM and referred to additional rules below.

Added rule 4 to provide for use of the referring style sheet or document's character encoding.

Added rule 5 to require falling back to UTF-8.

Removed the restriction on using @charset in embedded style sheets.

Allowed a BOM to precede the @charset rule.

Added requirement that @charset rule must be a literal '@charset"...";', not a CSS-syntax equivalent.

Added requirement to support for UTF-8 at minimum.

Specified that any @charset rule not at the beginning of the style sheet must be ignored.

Removed note on theoretical problem with @charset problem and precisely defined rules for character encoding detection based on @charset and/or BOM.

Specified that UAs must ignore style sheets in unknown encodings.

<a id="q21.28"></a>

### <a id="c5.8.1"></a>C.2.21 [Section 5.8.1 Matching attributes and attribute values](css2--selector.html--f66f7c932788.md#matching-attrs)

BCP 47 replaces RFC 1766.

<a id="q21.29"></a>

### <a id="c5.8.3"></a>C.2.22 [Section 5.8.3 Class selectors](css2--selector.html--f66f7c932788.md#class-html)

Class selectors are allowed for other formats than HTML.

Added a note about matching classes in formats with multiple class attributes per element. The behavior is non-normative, because, at the time of writing, there exist no such formats.

<a id="q21.30"></a>

### <a id="c5.9"></a>C.2.23 [Section 5.9 ID selectors](css2--selector.html--f66f7c932788.md#id-selectors)

Specified how to match elements with two or more ID attributes.

<a id="q21.31"></a>

### <a id="c5.10"></a>C.2.24 [Section 5.10 Pseudo-elements and pseudo-classes](css2--selector.html--f66f7c932788.md#pseudo-elements)

Removed exception for HTML UAs that allowed them (and only them) to ignore ':first-letter' and ':first-line'.

<a id="q21.32"></a>

### <a id="c5.11.2"></a>C.2.25 [Section 5.11.2 The link pseudo-classes: :link and :visited](css2--selector.html--f66f7c932788.md#link-pseudo-classes)

UAs may return a :visited link to :link status at some point. (This was previously a note, but is now normative.)

Added a note about privacy concerns with link pseudo classes and allowed UAs to treat :visited as :link.

<a id="q21.33"></a>

### <a id="c5.11.4"></a>C.2.26 [Section 5.11.4 The language pseudo-class: :lang](css2--selector.html--f66f7c932788.md#lang)

The identifier C in ':lang(C)' need not be a valid language code, but it must not be empty.

<a id="q21.34"></a>

### <a id="c5.12.1"></a>C.2.27 [Section 5.12.1 The :first-line pseudo-element](css2--selector.html--f66f7c932788.md#first-line-pseudo)

':first-line' also applies to inline blocks, table captions and table cells. Added a definition of "first formatted line" to make the rules about which line is the first line more precise.

UAs are no longer forbidden from applying more properties than the given list.

<a id="q21.35"></a>

### <a id="c5.12.2"></a>C.2.28 [Section 5.12.2 The :first-letter pseudo-element](css2--selector.html--f66f7c932788.md#first-letter)

More precise definition of first letter. Added rules for cases where the first letter is in an inline block or table cell. Added rules for cases when preceding punctuation is in a different element from the first letter itself.

UAs may apply other properties to first letters than the given list.

Unicode character classes Pi and Pf added to the definition of punctuation.

<a id="q21.36"></a>

### <a id="c6.1"></a>C.2.29 [Section 6.1 Specified, computed, and actual values](css2--cascade.html--c7aff33e6f0d.md#value-stages)

Redefined "computed value" and created the concept of "used value" so that inheritance can be performed without laying out the document. This change has the effect of allowing (requiring) percentages to be inherited as percentages and affects many other layout calculations throughout the spec.

Since computed value of a property can now also be a percentage. In particular, the following properties now inherit the percentage if the specified value is a percentage:

- background-position
- bottom, left, right, top
- height, width
- margin-bottom, margin-left, margin-right, margin-top,
- min-height, min-width
- padding-bottom, padding-left, padding-right, padding-top
- text-indent

Note that only 'text-indent' inherits by default, the others only inherit if the 'inherit' keyword is specified.

<a id="q21.37"></a>

### <a id="c6.4.1"></a>C.2.30 [Section 6.4.1 Cascading order](css2--cascade.html--c7aff33e6f0d.md#cascading-order)

Changed suggestion that user be able to turn off author styles to a requirement.

<a id="q21.38"></a>

### <a id="c6.4.3"></a>C.2.31 [Section 6.4.3 Calculating a selector's specificity](css2--cascade.html--c7aff33e6f0d.md#specificity)

The "style" attribute now has a higher specificity than any style rule.

Pseudo-elements are now counted with elements in calculating a a selector's specificity.

<a id="q21.39"></a>

### <a id="c6.4.4"></a>C.2.32 [Section 6.4.4 Precedence of non-CSS presentational hints](css2--cascade.html--c7aff33e6f0d.md#preshint)

"Non-CSS presentational hints" no longer exist, with the exception of a small set of attributes in HTML.

<a id="q21.40"></a>

### <a id="c7.3"></a>C.2.33 [Section 7.3 Recognized Media Types](css2--media.html--3a324a170379.md#media-types)

Added 'speech' media type.

Marked "Media" field in property descriptions informative.

<a id="q21.41"></a>

### <a id="c7.3.1"></a>C.2.34 [Section 7.3.1 Media Groups](css2--media.html--3a324a170379.md#media-types)

Marked this section informative.

Added sound to 'handheld' in media type/media group table.

Changed 'tactile' to be both 'static' and 'interactive'.

<a id="q21.42"></a>

### <a id="c8.3"></a>C.2.35 [Section 8.3 Margin properties](css2--box.html--8875bbcdefe6.md#margin-properties)

If the containing block's width depends on an element with percentage margins, then the resulting layout is undefined in CSS 2.1.

<a id="q21.43"></a>

### <a id="c8.3.1"></a>C.2.36 [Section 8.3.1 Collapsing margins](css2--box.html--8875bbcdefe6.md#collapsing-margins)

In the definition of "collapsing margins", added "non-empty content" and "clearance" to the parenthetical list of things that prevent consecutive margins from being adjoining.

Vertical margins of elements with 'overflow' other than 'visible' no longer collapse with their in-flow children.

Defined how margins collapse through an element with adjoining top and bottom margins.

Added that margins of the root element's box do not collapse.

More rigorously [defined "adjoining" for margin collapsing](css2--box.html--8875bbcdefe6.md#what-is-adjoining).

Sixth bullet, second sub-bullet: to find the position of the top border edge, assume the element has a <em>bottom</em> (rather than top) border.

Margins of relatively positioned elements <em>do</em> sometimes collapse.

<a id="q21.44"></a>

### <a id="c8.4"></a>C.2.37 [Section 8.4 Padding properties](css2--box.html--8875bbcdefe6.md#padding-properties)

If the containing block's width depends on an element with percentage padding, then the resulting layout is undefined in CSS 2.1.

<a id="q21.45"></a>

### <a id="c8.5.2"></a>C.2.38 [Section 8.5.2 Border color](css2--box.html--8875bbcdefe6.md#border-color-properties)

'transparent' can now be specified independently for each border side, on par with \<color\>.

<a id="q21.46"></a>

### <a id="c8.5.3"></a>C.2.39 [Section 8.5.3 Border style](css2--box.html--8875bbcdefe6.md#border-style-properties)

3D border styles ('groove', 'ridge', 'inset', 'outset') now depend on the corresponding border-color rather than on 'color'.

<a id="q21.47"></a>

### <a id="c8.6"></a>C.2.40 [Section 8.6 The box model for inline elements in bidirectional context](css2--box.html--8875bbcdefe6.md#bidi-box-model)

Added this new section to specify layout of inline boxes when affected by bidi.

<a id="q21.48"></a>

### <a id="c9.1.2"></a>C.2.41 [Section 9.1.2 Containing blocks](css2--visuren.html--3f334c530cf4.md#containing-block)

Removed paragraphs about the initial containing block, as this is now defined differently. (See [changes to section 10.1](#initial-containing-block).)

<a id="q21.49"></a>

### <a id="c9.2.1.1"></a>C.2.42 [Section 9.2.1.1 Anonymous block boxes](css2--visuren.html--3f334c530cf4.md#anonymous-block-level)

Added a paragraph to define formatting when an inline box contains a block box.

Specified what property values are applied to anonymous boxes.

<a id="q21.50"></a>

### <a id="c9.2.2.1"></a>C.2.43 [Section 9.2.2.1 Anonymous inline boxes](css2--visuren.html--3f334c530cf4.md#anonymous)

Specified that collapsed white space does not generate anonymous inline boxes.

<a id="q21.51"></a>

### <a id="c9.2.3"></a>C.2.44 [Section 9.2.3 Run-in boxes](css2--visuren.html--3f334c530cf4.md#run-in)

Changed run-in rules so that a) run-ins that contain blocks become blocks b) run-ins can only run into sibling blocks and c) run-ins cannot run into other run-ins.

<a id="q21.52"></a>

### <a id="c9.2.4"></a>C.2.45 [Section 9.2.4 The 'display' property](css2--visuren.html--3f334c530cf4.md#display-prop)

The 'marker' and 'compact' values of the 'display' property are not part of CSS 2.1. Text relating to these values has been removed throughout the specification.

Defined the computed value of 'display' as the specified value except for positioned and floating elements and for the root element. The computed value of 'display' for these elements is defined in [section 9.7](css2--visuren.html--3f334c530cf4.md#dis-pos-flo) and is slightly different from the definition in CSS2.

Conforming HTML UAs are no longer allowed to ignore the 'display' property.

<a id="q21.53"></a>

### <a id="c9.3.1"></a>C.2.46 [Section 9.3.1 Choosing a positioning scheme](css2--visuren.html--3f334c530cf4.md#choose-position)

The 'position' property now applies to all elements, including generated content.

The effect of relative positioning on table captions and internal table elements is undefined in CSS 2.1.

For fixed positioning, introduced a conflict between this section and section 10.1 rule 3. See [howcome](http://lists.w3.org/Archives/Member/w3c-css-wg/2003AprJun/0172.html) \[member-only\] for rationale.

Forbid UAs from paginating the content of fixed boxes.

UAs are allowed to treat all values of 'position' as 'static' on the root element.

<a id="q21.54"></a>

### <a id="c9.3.2"></a>C.2.47 [Section 9.3.2 Box offsets](css2--visuren.html--3f334c530cf4.md#position-props)

Defined computed values of 'top', 'right', 'bottom', 'left' based on the value of 'position'.

Percentage offsets are no longer undefined for containing blocks without an explicit height.

<a id="q21.55"></a>

### <a id="c9.4.1"></a>C.2.48 [Section 9.4.1 Block formatting contexts](css2--visuren.html--3f334c530cf4.md#block-formatting)

Specified that floats, absolutely positioned elements, inline-blocks, table-cells, table-captions, and elements with 'overflow' other than 'visible' establish new block formatting contexts.

In the paragraph about the position of a box's outer edge with respect to its containing block, except boxes that establish a new block formatting context, as they may become narrower due to floats.

<a id="q21.56"></a>

### <a id="c9.4.2"></a>C.2.49 [Section 9.4.2 Inline formatting context](css2--visuren.html--3f334c530cf4.md#inline-formatting)

Specified that the effect of 'justify' on the content of a line box does not affect the contents of inline-table and inline-block boxes.

Empty line boxes are now required to be treated as zero-height and ignored in margin collapsing.

<a id="q21.57"></a>

### <a id="c9.4.3"></a>C.2.50 [Section 9.4.3 Relative positioning](css2--visuren.html--3f334c530cf4.md#relative-positioning)

Added several paragraphs and an example to explain exactly what the computed values of relatively-positioned offsets are, how they affect each other, and what happens when the positioning is overconstrained. (These were not previously defined.)

<a id="q21.58"></a>

### <a id="c9.5"></a>C.2.51 [Section 9.5 Floats](css2--visuren.html--3f334c530cf4.md#floats)

Floats are no longer required to have an explicit width.

Floats outside of line boxes no longer align to the bottom of the preceding block box; it is implied that they are initially aligned with their non-floated position.

Specified that "If a shortened line box is too small to contain any further content, then it is shifted downward until either it fits or there are no more floats present."

Specified that the border box of a table, block-level replaced element, or element in the normal flow that establishes a new block formatting context must not overlap any floats in the same block formatting context.

<a id="q21.59"></a>

### <a id="c9.5.1"></a>C.2.52 [Section 9.5.1 Positioning the float](css2--visuren.html--3f334c530cf4.md#float-position)

The 'float' property now also applies to :before/:after and generated content.

UAs are now allowed to treat all values of float as 'none' on the root element.

Added to rule 4 prose to define the position of a float when it occurs between two collapsing margins.

<a id="q21.60"></a>

### <a id="c9.5.2"></a>C.2.53 [Section 9.5.2 Controlling flow next to floats](css2--visuren.html--3f334c530cf4.md#flow-control)

Defined <em>clearance</em> to precisely detail the 'clear' property's effect on margin collapsing and the block's cleared position.

Added note to explain effect of 'clear' on inline elements since CSS1 (but not CSS2 or CSS 2.1) allows 'clear' on inline elements.

<a id="q21.61"></a>

### <a id="c9.7"></a>C.2.54 [Section 9.7 Relationships between 'display', 'position', and 'float'](css2--visuren.html--3f334c530cf4.md#dis-pos-flo)

Changed rules to convert 'display' not always to 'block', but to an appropriate block-level display value as given by a mapping table.

Added rule 4 to convert root element's 'display' value according to the mapping.

<a id="q21.62"></a>

### <a id="c9.9"></a>C.2.55 [Section 9.9 Layered presentation](css2--visuren.html--3f334c530cf4.md#layers)

Specified that the background and borders of an element that forms a stacking context are behind all of its descendants, altered stacking context prose to be more precise, and added a normative [Appendix E: Elaborate description of Stacking Contexts](css2--zindex.html--b486aef70d4d.md) to be even more precise about the position of borders, backgrounds, and content on the z-axis.

<a id="q21.63"></a>

### <a id="c9.10"></a>C.2.56 [Section 9.10 Text direction](css2--visuren.html--3f334c530cf4.md#direction)

Conforming UAs are now allowed to not support bidirectional text; in this case they must ignore the 'direction' and 'unicode-bidi' properties. However since applying bidi can have an effect even when a document does not contain right-to-left characters, UAs that do support bidi are no longer permitted to not apply the algorithm just because the document lacks right-to-left characters.

Added a paragraph to define precisely how the Unicode bidirectional algorithm applies to text in the CSS formatting model and how the CSS 'direction' property on blocks maps into the algorithm.

Conforming HTML UAs are no longer exempt from supporting 'direction' and 'unicode-bidi'.

<a id="q21.64"></a>

### <a id="c10"></a>C.2.57 [Chapter 10 Visual formatting model details](css2--visudet.html--12e8bc0e6b7c.md)

Updated prose to use the terms "specified", "computed" and "used" as appropriate when referencing values. This affects many calculations in this section. (See [changes to section 6.1](#c6.1).)

<a id="q21.65"></a>

### <a id="c10.1"></a>C.2.58 [Section 10.1 Definition of "containing block"](css2--visudet.html--12e8bc0e6b7c.md#containing-block-details)

<a id="initial-containing-block"></a>In rule 1, defined the initial containing block as the viewport for continuous media and the page area for paged media. (It was previously undefined.)

In rule 2, defined the page area as the containing block for fixed positioned elements in paged media.

In rule 4.1, when the containing block of an absolutely-positioned element is formed by an inline-level element, it is now formed by that element's padding edges, not its content edges.

In rule 4, changed the containing block for absolutely positioned elements with only statically positioned elements from the root's content box to the initial containing block.

Specified the positioning and breaking behavior of absolutely-positioned elements in paged media.

<a id="q21.66"></a>

### <a id="c10.2"></a>C.2.59 [Section 10.2 Content width](css2--visudet.html--12e8bc0e6b7c.md#the-width-property)

Declared that if the containing block's width depends on an element's percentage width, then the resulting layout is undefined in CSS 2.1.

<a id="q21.67"></a>

### <a id="c10.3"></a>C.2.60 [Section 10.3 Calculating widths and margins](css2--visudet.html--12e8bc0e6b7c.md#Computing_widths_and_margins)

The computed values of 'left' and 'right' for are now defined in [section 9.3.2](css2--visuren.html--3f334c530cf4.md#position-props). The value 'auto' does not always compute to zero.

Added sections 10.3.9 and 10.3.10 to define calculations for inline blocks.

<a id="q21.68"></a>

### <a id="c10.3.2"></a>C.2.61 [Section 10.3.2 Inline, replaced elements](css2--visudet.html--12e8bc0e6b7c.md#inline-replaced-width)

The sizing algorithm for replaced elements now takes into account and attempts to preserve the replaced content's intrinsic ratio. Sizing of replaced elements with percentage intrinsic sizes and without intrinsic sizes is now also defined.

The effect of percentage intrinsic widths is now undefined for CSS level�2, rather than ignored.

<a id="q21.69"></a>

### <a id="c10.3.3"></a>C.2.62 [Section 10.3.3 Block-level, non-replaced elements in normal flow](css2--visudet.html--12e8bc0e6b7c.md#blockwidth)

Specified that a computed total of the width, padding, and borders that is greater than the containing block width causes auto margins to be treated as zero in the rest of the rules. This avoids 'auto' margins being negative on the start edge.

<a id="q21.70"></a>

### <a id="c10.3.4"></a>C.2.63 [Section 10.3.4 Block-level, replaced elements in normal flow](css2--visudet.html--12e8bc0e6b7c.md#block-replaced-width)

Applied changes to section 10.3.2 and section 10.3.3 to block-level replaced elements in normal flow by referring to the calculations in those sections.

<a id="q21.71"></a>

### <a id="c10.3.5"></a>C.2.64 [Section 10.3.5 Floating, non-replaced elements](css2--visudet.html--12e8bc0e6b7c.md#float-width)

Defined computations for 'auto' width floats as shrink-to-fit. (Floats were previously required to have fixed widths.)

<a id="q21.72"></a>

### <a id="c10.3.6"></a>C.2.65 [Section 10.3.6 Floating, replaced elements](css2--visudet.html--12e8bc0e6b7c.md#float-replaced-width)

Applied changes to section 10.3.2 to this section by referencing it for 'auto' width calculations.

<a id="q21.73"></a>

### <a id="c10.3.7"></a>C.2.66 [Section 10.3.7 Absolutely positioned, non-replaced elements](css2--visudet.html--12e8bc0e6b7c.md#abs-non-replaced-width)

Defined the static position of an element more precisely.

Rewrote constraint rules.

The 'direction' property of the containing block of the static position determines which side is clamped to the static position, not the 'direction' property of the containing block of the absolutely positioned element.

<a id="q21.74"></a>

### <a id="c10.3.8"></a>C.2.67 [Section 10.3.8 Absolutely positioned, replaced elements](css2--visudet.html--12e8bc0e6b7c.md#abs-replaced-width)

In rule 1, applied sizing rules from section 10.3.2.

In rule 2 (formerly rules 2 and 3), referred to new definition of 'static position' in section 10.3.7.

Also in rule 2, the 'direction' property of the containing block of the static position determines which side is clamped to the static position, not the 'direction' property of the containing block of the absolutely positioned element.

In rule 4 (formerly rule 5), prevented 'auto' left and right margins in resulting in a negative margin on the start edge.

<a id="q21.75"></a>

### <a id="c10.4"></a>C.2.68 [Section 10.4 Minimum and maximum widths](css2--visudet.html--12e8bc0e6b7c.md#min-max-widths)

Specified that if the containing block's width is negative, the used value of a percentage min/max width is zero.

Specified that if the min/max width is specified in percentages and the containing block's width depends on this element's width, then the resulting layout is undefined in CSS 2.1.

The UA is no longer allowed to select an arbitrary minimum width.

The used width of replaced elements with an intrinsic ratio and both 'width' and 'height' specified as 'auto' is now calculated according to a table designed to preserve the intrinsic ratio as much as possible within the given constraints.

<a id="q21.76"></a>

### <a id="c10.5"></a>C.2.69 [Section 10.5 Content height](css2--visudet.html--12e8bc0e6b7c.md#the-height-property)

Removed mention of 'line-height' for inline elements since their content box height no longer depends on 'line-height'.

Percentage heights on absolutely-positioned elements are no longer treated as 'auto' when the containing block's height is not explicitly specified. Added a note to explain why this is possible.

Specified that a percentage height on the root element is relative to the initial containing block.

<a id="q21.77"></a>

### <a id="c10.6"></a>C.2.70 [Section 10.6 Calculating heights and margins](css2--visudet.html--12e8bc0e6b7c.md#Computing_heights_and_margins)

The computed values of 'top' and 'bottom' for are now defined in [section 9.3.2](css2--visuren.html--3f334c530cf4.md#position-props). The value 'auto' does not always compute to zero.

Added section 10.6.6 to cover cases that are no longer covered under the previous sections.

Added section 10.6.7 to define 'auto' heights for block formatting context roots. (Unlike other block boxes, the height of these boxes increases to accommodate any normal-flow descendant floats.)

<a id="q21.78"></a>

### <a id="c10.6.1"></a>C.2.71 [Section 10.6.1 Inline, non-replaced elements](css2--visudet.html--12e8bc0e6b7c.md#inline-non-replaced)

The height of an inline box is no longer given by the 'line-height' property and is now undefined. This section now suggests that the height of the box can be based on the font.

<a id="q21.79"></a>

### <a id="c10.6.2"></a>C.2.72 [Section 10.6.2 Inline replaced elements, block-level replaced elements in normal flow, 'inline-block' replaced elements in normal flow and floating replaced elements](css2--visudet.html--12e8bc0e6b7c.md#inline-replaced-height)

The sizing algorithm for replaced elements now takes into account and attempts to preserve the replaced content's intrinsic ratio. Sizing of replaced elements with percentage intrinsic sizes and without intrinsic sizes is now also defined.

Specified that for inline elements, the margin box is used when calculating the height of the line box.

<a id="q21.80"></a>

### <a id="c10.6.3"></a>C.2.73 [Section 10.6.3 Block-level non-replaced elements in normal flow when 'overflow' computes to 'visible'](css2--visudet.html--12e8bc0e6b7c.md#normal-block)

This section now only applies to elements whose 'overflow' value computes to 'visible'; elements with other values of 'overflow' are discussed in the new section 10.6.7 ('Auto' heights for block formatting context roots).

<a id="q21.81"></a>

### <a id="c10.6.4"></a>C.2.74 [Section 10.6.4 Absolutely positioned, non-replaced elements](css2--visudet.html--12e8bc0e6b7c.md#abs-non-replaced-height)

Defined the static position of an element more precisely.

Rewrote constraint rules.

<a id="q21.82"></a>

### <a id="c10.6.5"></a>C.2.75 [Section 10.6.5 Absolutely positioned, replaced elements](css2--visudet.html--12e8bc0e6b7c.md#abs-replaced-height)

In rule 1, applied sizing rules from section 10.6.2.

<a id="q21.83"></a>

### <a id="c10.7"></a>C.2.76 [Section 10.7 Minimum and maximum heights](css2--visudet.html--12e8bc0e6b7c.md#min-max-heights)

Percentage min/max heights on absolutely-positioned elements are no longer treated as '0'/'none' when the containing block's height is not explicitly specified. However if the containing block's width depends on an element's percentage width, then the resulting layout is undefined in CSS 2.1.

The used width of replaced elements with an intrinsic ratio and both 'width' and 'height' specified as 'auto' is now calculated according to a table designed to preserve the intrinsic ratio as much as possible within the given constraints.

<a id="q21.84"></a>

### <a id="c10.8"></a>C.2.77 [Section 10.8 Line height calculations](css2--visudet.html--12e8bc0e6b7c.md#line-height)

Added rule 4 to specify that the height of the line box must be at least as much as that specified by the 'line-height' property on the this block.

<a id="q21.85"></a>

### <a id="c10.8.1"></a>C.2.78 [Section 10.8.1 Leading and half-leading](css2--visudet.html--12e8bc0e6b7c.md#leading)

UAs are no longer permitted to clip content to the line box, and are instead asked to render overlapping boxes in document order.

'line-height' set on a block no longer specifies the minimal height of each inline box; instead it specifies the minimal height of each line box. The exact effect of this requirement is expressed in terms of struts; it is affected by vertical-alignment.

Adjusted text to reflect that the content box height of an inline is no longer dictated by the 'line-height' property.

Since the content box is now defined by the font and not by the line-height, 'text-top' and 'text-bottom' refer to the content area instead of the font.

Defined 'top' and 'bottom' alignment in terms of aligned subtrees to take into account any protruding descendants.

Defined the baseline of inline tables and inline blocks.

<a id="q21.86"></a>

### <a id="c11.1"></a>C.2.79 [Section 11.1 Overflow and clipping](css2--visufx.html--2bc674cb7ab0.md#overflow-clipping)

Specified that 'overflow' clips to the padding edge.

<a id="q21.87"></a>

### <a id="c11.1.1"></a>C.2.80 [Section 11.1.1 Overflow](css2--visufx.html--2bc674cb7ab0.md#overflow)

'projection' media are no longer permitted to print overflowing content for 'overflow: scroll'. 'Print' media now <em>may</em>, as opposed to <em>should</em>.

UAs are now required to apply the 'overflow' property set on the root element to the viewport. Additionally, HTML UAs must use the 'overflow' property on the HTML BODY element instead if the root element's 'overflow' value is 'visible'.

Specified placement of scrollbar in the box model.

The width of any scrollbars is no longer included in the width of the containing block. (And consequently, all text in section�10.3 that subtracts the scrollbar width from the containing block width has been removed.)

<a id="q21.88"></a>

### <a id="c11.1.2"></a>C.2.81 [Section 11.1.2 Clipping: the 'clip' property](css2--visufx.html--2bc674cb7ab0.md#clipping)

The 'clip' property now applies only to absolutely positioned elements. Furthermore, it applies to those elements even when their 'overflow' is 'visible'.

The default value of 'clip', 'auto', now indicates no clipping rather than clipping to the element's border box.

Values of "rect()" should be separated by commas. UAs are required to support this syntax, but may also support a space-separated syntax since CSS2 was not clear about this.

While CSS2 specified that values of "rect()" give offsets from the respective sides of the box, current implementations interpret values with respect to the top and left edges for <em>all</em> four values (top, right, bottom, and left). This is now the specified interpretation.

<a id="q21.89"></a>

### <a id="c11.2"></a>C.2.82 [Section 11.2 Visibility](css2--visufx.html--2bc674cb7ab0.md#visibility)

The 'visibility' property is now defined to inherit, and descendant elements can override an ancestor's hidden visibility.

<a id="q21.90"></a>

### <a id="c12"></a>C.2.83 [Chapter 12 Generated content, automatic numbering, and lists](css2--generate.html--37748b674cd2.md)

Moved all discussion of aural rendering to Appendix A.

<a id="q21.91"></a>

### <a id="c12.1"></a>C.2.84 [Section 12.1 The :before and :after pseudo-elements](css2--generate.html--37748b674cd2.md#before-after-content)

Removed restrictions on which properties and property values are allowed on ':before' and ':after' pseudo-elements.

<a id="q21.92"></a>

### <a id="c12.2"></a>C.2.85 [Section 12.2 The 'content' property](css2--generate.html--37748b674cd2.md#content)

The initial value of 'content' is now 'normal', not the empty string.

The 'content' property now distinguishes between the empty string, which creates an empty box; and 'normal'/'none', which create no box at all. (There is no distinction between 'normal' and 'none' in level 2.)

A UA is now allowed to report a URI that fails to download.

Removed recommendation to authors to put rules with media-sensitive 'content' properties inside '@media'.

Whether '&#x5C;A' escapes in generated content create line breaks is now subject to the 'white-space' property.

The former section 12.3 on interaction between ':before', ':after' and elements with 'display: compact' or 'display: run-in' has been removed. (The interaction is already fully defined, because generated content consists of boxes in the tree, no different from other boxes.)

<a id="q21.93"></a>

### <a id="c12.3.2"></a>C.2.86 [Section 12.3.2 Inserting quotes with the 'content' property](css2--generate.html--37748b674cd2.md#quotes-insert)

Specified that extra 'close-quote's and 'no-close-quote's (those without a matching 'open-quote' or 'no-open-quote') are not rendered, and that neither 'close-quote' nor 'no-close-quote' cause the quoting depth to be negative.

<a id="q21.94"></a>

### <a id="c12.4"></a>C.2.87 [Section 12.4 Automatic counters and numbering](css2--generate.html--37748b674cd2.md#counters)

Defined what a rule with duplicate counters, such as 'counter-reset: section 2 section', means.

<a id="q21.95"></a>

### <a id="c12.4.1"></a>C.2.88 [Section 12.4.1 Nested counters and scope](css2--generate.html--37748b674cd2.md#scope)

The scope of a counter no longer defaults to the whole document, but starts at the first element that uses the counter. (This affects counters that are used without a prior 'counter-reset' to set the scope explicitly.)

<a id="q21.96"></a>

### <a id="c12.5"></a>C.2.89 [Section 12.5 Lists](css2--generate.html--37748b674cd2.md#lists)

Removed text in section 12.5 (formerly 12.6) relating to the 'marker' display value.

Removed the 'marker-offset' property (and thus former section 12.6.1).

<a id="q21.97"></a>

### <a id="c12.5.1"></a>C.2.90 [Section 12.5.1 Lists](css2--generate.html--37748b674cd2.md#list-style)

The list styles 'hebrew', 'armenian', 'georgian', 'cjk-ideographic', 'hiragana', 'katakana', 'hiragana-iroha' and 'katakana-iroha' have been removed due to lack of implementation experience. (They are expected to return in the CSS3 Lists module.)

Removed the sentence that said that an unknown value for 'list-style-type' should cause the value 'decimal' to be used instead. Instead, normal parsing rules apply and cause the rule to be ignored.

The size of list style markers without an intrinsic size is now defined.

<a id="q21.98"></a>

### <a id="c13.1"></a>C.2.91 [Chapter 13 Paged media](css2--page.html--984664616bba.md)

The 'size', 'marks', and 'page' properties are not part of CSS 2.1.

<a id="q21.99"></a>

### <a id="c13.2.2"></a>C.2.92 [Section 13.2.2 Page selectors](css2--page.html--984664616bba.md#page-selectors)

The requirement for UA's to honor different declarations for :left, :right, and :first pages has been softened to simplify implementations: the page area of the :first page may be used for :left and :right pages as well.

<a id="q21.100"></a>

### <a id="c13.3.1"></a>C.2.93 [Section 13.3.1 Page break properties](css2--page.html--984664616bba.md#page-break-props)

UAs are now only required to apply the page break properties to block-level elements in the normal flow of the root element, not to other blocks.However, UAs are now permitted to apply these properties to elements other than block-level elements.

Defined treatment of margins, borders, and padding when a page break splits a box.

The 'page-break-inside' property no longer inherits.

<a id="q21.101"></a>

### <a id="c13.3.3"></a>C.2.94 [Section 13.3.3 Allowed page breaks](css2--page.html--984664616bba.md#allowed-page-breaks)

The 'page-break-inside' property of all ancestors is checked for page-breaking restrictions, not just that of the breakpoint's parent.

When dropping restrictions to find a page breaking opportunity, rule A is dropped together with B and D rather than together with C.

Removed restriction on breaking within absolutely positioned boxes.

<a id="q21.102"></a>

### <a id="c14.2.1"></a>C.2.95 [Section 14.2.1 Background properties](css2--colors.html--5784063d2778.md#background-properties)

For 'background-position', the restriction that keywords cannot be combined with percentage or length values is removed. I.e., a value like: '25% top' is now allowed. Also, 'background-position' now applies to all elements, not just to block-level and replaced elements.

User agents are no longer allowed to treat a value of 'fixed' for 'background-attachment' as 'scroll'. Instead they must ignore all such declarations as if 'fixed' were an invalid value.

The size of background images without an intrinsic size is now defined.

<a id="q21.103"></a>

### <a id="c14.3"></a>C.2.96 Section 14.3 Gamma correction

The contents of this section is now a non-normative note.

<a id="q21.104"></a>

### <a id="c15"></a>C.2.97 [Chapter 15 Fonts](css2--fonts.html--d52fc14f36c2.md)

The 'font-stretch' and 'font-size-adjust' properties have been removed in CSS 2.1.

Font descriptors, the '@font-face' declaration, and all associated parts of the font matching algorithm have been removed in CSS 2.1.

<a id="q21.105"></a>

### <a id="c15.2"></a>C.2.98 [Section 15.2 Font matching algorithm](css2--fonts.html--d52fc14f36c2.md#algorithm)

In this section (previously 15.5), in step 5 (previously 8) of the font matching algorithm, the UA is now allowed to use multiple default fallback fonts to find a glyph for a given character.

In the per-property rule 2, specified that if there is only a small-caps font in a given family, then that font will be selected by 'normal'.

<a id="q21.106"></a>

### <a id="c15.3"></a>C.2.99 [Section 15.2.2 Font family](css2--fonts.html--d52fc14f36c2.md#font-family-prop)

The "missing character" glyph is no longer considered a match for the last font in a font set, but is now considered a match for U+FFFD.

Certain punctuation characters when appearing in unquoted font family names are now required to be escaped.

<a id="q21.107"></a>

### <a id="c15.5"></a>C.2.100 [Section 15.5 Small-caps](css2--fonts.html--d52fc14f36c2.md#small-caps)

The 'font-variant' property's effect is no longer restricted to bicameral scripts.

<a id="q21.108"></a>

### <a id="c15.6"></a>C.2.101 [Section 15.6 Font boldness](css2--fonts.html--d52fc14f36c2.md#font-boldness)

The computed value of 'font-weight' has been defined more precisely such that the 'bolder' and 'lighter' values have an appropriate effect when inheriting through elements with different font-families.

<a id="q21.109"></a>

### <a id="c15.7"></a>C.2.102 [Section 15.7 Font size](css2--fonts.html--d52fc14f36c2.md#font-size-props)

Removed suggestion of 1.2 fixed ratio between keyword font sizes in favor of notes recommending a variable ratio and a smallest font-size no less than 9 pixels per EM unit.

Added table mapping CSS font-size keywords to HTML font size numbers.

<a id="q21.110"></a>

### <a id="c16"></a>C.2.103 [Chapter 16 Text](css2--text.html--467a8857ae69.md)

The 'text-shadow' property is not in CSS 2.1.

<a id="q21.111"></a>

### <a id="c16.2"></a>C.2.104 [Section 16.2 Alignment](css2--text.html--467a8857ae69.md#alignment-prop)

The initial value of 'text-align' is no longer UA-defined but a nameless value that acts as 'left' if 'direction' is 'ltr', 'right' if 'direction' is 'rtl'.

The \<string\> value for 'text-align' is not part of CSS 2.1.

For 'text-align', specified that 'justify' is treated as the initial value when computed value of 'white-space' is 'pre' or 'pre-line'.

<a id="q21.112"></a>

### <a id="c16.3.1"></a>C.2.105 [Section 16.3.1 Underlining, over lining, striking, and blinking](css2--text.html--467a8857ae69.md#lining-striking-props)

More precisely defined what boxes are affected by text decorations specified on a given element.

Specified that underlines, overlines, and line-throughs apply only to text.

Specified that an underline, overline, or line-through applied across a line must be at a constant vertical position and with a constant thickness across the entire line.

Specified how text decorations are affected by relative positioning on descendants.

User agents are now allowed to recognize the 'blink' value but not blink, whereas before they were required to ignore the 'blink' value if they chose not to support blinking text.

Added text to allow older UAs to conform to this section if they follow CSS2's 'text-decoration' requirements but not the additional requirements in CSS2.1.

<a id="q21.113"></a>

### <a id="c16.4"></a>C.2.106 [Section 16.4 Letter and word spacing](css2--text.html--467a8857ae69.md#spacing-props)

Support for the various values of 'letter-spacing' and 'word-spacing' is no longer optional.

Specified that word spacing affects each space, non-breaking space, and ideographic space left in the text after white space processing rules have been applied.

<a id="q21.114"></a>

### <a id="c16.5"></a>C.2.107 [Section 16.5 Capitalization](css2--text.html--467a8857ae69.md#caps-prop)

UAs are no longer allowed to not transform characters for which there is an appropriate transformation but which are outside of Latin-1.

<a id="q21.115"></a>

### <a id="c16.6"></a>C.2.108 [Section 16.6 White space](css2--text.html--467a8857ae69.md#white-space-prop)

The 'white-space' property now applies to all elements, not just to block-level elements.

"&#x5C;A" in generated content no longer forces a break for 'normal' and 'nowrap' values of 'white-space'.

Specified that the CSS white space processing model assumes all newlines have been normalized to line feeds.

Added [section 16.6.1](css2--text.html--467a8857ae69.md#white-space-model) to precisely define white space handling.

Added [section 16.6.3](css2--text.html--467a8857ae69.md#ctrlchars) to specify handling of control and combining characters.

<a id="q21.116"></a>

### <a id="c17"></a>C.2.109 [Chapter 17 Tables](css2--tables.html--201812dd6e3c.md)

Moved all discussion of aural rendering and related properties to Appendix A.

Updated prose to use the terms "specified", "computed" and "used" as appropriate when referencing values. (See [changes to section 6.1](#c6.1).)

<a id="q21.117"></a>

### <a id="c17.2"></a>C.2.110 [Section 17.2 The CSS table model](css2--tables.html--201812dd6e3c.md#table-display)

Defined handling of multiple 'table-header-group' and 'table-footer-group' elements.

UAs are no longer allowed to ignore the table display values on arbitrary HTML elements, only on HTML table elements.

<a id="q21.118"></a>

### <a id="c17.2.1"></a>C.2.111 [Section 17.2.1 Anonymous table objects](css2--tables.html--201812dd6e3c.md#anonymous-boxes)

Changed rules so that internal table elements without an enclosing 'table' or 'inline-table' box generate an anonymous 'inline-table' rather than an anonymous 'table' when inside a "display: inline" parent element.

The anonymous table object rules now treat anonymous boxes as equal to elements' boxes. Replaced several instances of the term "element" with "box", removed several instances of "(in the document tree)" and clarified that anonymous boxes generated in earlier rules are part of the input to later rules. Also replaced the term "object" with "box", as is used throughout the rest of the specification.

HTML UAs are no longer exempt from the anonymous box generation rules.

<a id="q21.119"></a>

### <a id="c17.4"></a>C.2.112 [Section 17.4 Tables in the visual formatting model](css2--tables.html--201812dd6e3c.md#model)

The relationship of the caption box, table box, and outer anonymous table box has been changed as follows:

- The margins of the table box now apply to the outer (anonymous) table box that encloses both the table box and the caption(s), not to the inner table box.
- The width of the anonymous box is now equal to the border-box width of the table box inside it instead of adapting to the widths and positions of both the table box and its captions.

<a id="q21.120"></a>

### <a id="c17.4.1"></a>C.2.113 [Section 17.4.1 Caption position and alignment](css2--tables.html--201812dd6e3c.md#caption-position)

The 'left' and 'right' values on 'caption-side' have been removed.

<a id="q21.121"></a>

### <a id="c17.5"></a>C.2.114 [Section 17.5 Visual layout of table contents](css2--tables.html--201812dd6e3c.md#table-layout)

Changed rule 5 in grid layout rules to allow overlapping of table cells instead of leaving skipping a gap in the grid to avoid overlap.

<a id="q21.122"></a>

### <a id="c17.5.1"></a>C.2.115 [Section 17.5.1 Table layers and transparency](css2--tables.html--201812dd6e3c.md#table-layers)

In point 6, changed 'These "empty" cells are transparent' to:

> If the value of their 'empty-cells' property is 'hide' these "empty" cells are transparent through the cell, row, row group, column, and column group backgrounds, letting the table background show through.

<a id="q21.123"></a>

### <a id="c17.5.2.1"></a>C.2.116 [Section 17.5.2.1 Fixed table layout](css2--tables.html--201812dd6e3c.md#fixed-table-layout)

Specified that in fixed table layout, extra columns in rows after the first must not be rendered.

<a id="q21.124"></a>

### <a id="c17.5.2.2"></a>C.2.117 [Section 17.5.2.2 Automatic table layout](css2--tables.html--201812dd6e3c.md#auto-table-layout)

Restricted inputs to the table layout algorithm for 'table-layout: auto', whether or not the algorithm described in this section is used, to the width of the containing block and the content of, and any CSS properties set on, the table and any of its descendants.

Added rule 4 to include the column group's width in the algorithm for determining column widths.

<a id="q21.125"></a>

### <a id="c17.5.3"></a>C.2.118 [Section 17.5.3 Table height algorithms](css2--tables.html--201812dd6e3c.md#height-layout)

The 'height' property on tables is now treated as a minimum height; the UA no longer has the option of using 'height' to constrain the size of the table to be smaller than its contents.

The baseline of a cell is now defined much more precisely.

Defined the baseline of a row with no baseline-aligned cells.

<a id="q21.126"></a>

### <a id="c17.5.4"></a>C.2.119 [Section 17.5.4 Horizontal alignment in a column](css2--tables.html--201812dd6e3c.md#column-alignment)

The \<string\> value for 'text-align' is not part of CSS 2.1.

<a id="q21.127"></a>

### <a id="c17.6"></a>C.2.120 [Section 17.6 Borders](css2--tables.html--201812dd6e3c.md#borders)

Several popular browsers assume an initial value for 'border-collapse' of 'separate' rather than 'collapse' or exhibit behavior that is close to that value, even if they do not actually implement the CSS table model. 'Separate' is now the initial value.

<a id="q21.128"></a>

### <a id="c17.6.1"></a>C.2.121 [Section 17.6.1 The separated borders model](css2--tables.html--201812dd6e3c.md#separated-borders)

Specified the effect of padding on the table element.

Specified which parts of the table are included in the width measurement.

<a id="q21.129"></a>

### <a id="c17.6.1.1"></a>C.2.122 [Section 17.6.1.1 Borders and Backgrounds around empty cells](css2--tables.html--201812dd6e3c.md#empty-cells)

Refined definition of "empty" when used as a condition for the 'empty-cells' property so that it is not triggered when the cell includes any child elements, even if they are empty.

The 'empty-cells' property now hides both borders and backgrounds, not just borders.

Changed behavior of a row when it collapses due to 'empty-cells': it is no longer treated as "display: none". Instead it is given zero height and its associated border-spacing is eliminated.

<a id="q21.130"></a>

### <a id="c17.6.2"></a>C.2.123 [Section 17.6.2 The collapsing border model](css2--tables.html--201812dd6e3c.md#collapsing-borders)

The outer half of the table borders no longer lie in the margin area. Specified which part of the table is considered the border are in the collapsed borders model and how its width is calculated. The edges of the box in which the table background is painted is, however left explicitly undefined.

<a id="q21.131"></a>

### <a id="c17.6.2.1"></a>C.2.124 [Section 17.6.2.1 Border conflict resolution](css2--tables.html--201812dd6e3c.md#border-conflict-resolution)

Defined in rule 4 what happens when two elements of the same type conflict and their borders have the same width and style.

<a id="q21.132"></a>

### <a id="c18.1"></a>C.2.125 [Section 18.1 Cursors: the 'cursor' property](css2--ui.html--ff1f72803ea0.md#cursor-props)

The size of cursors without an intrinsic size is now defined.

<a id="q21.133"></a>

### <a id="c18.4"></a>C.2.126 [Section 18.4 Dynamic outlines](css2--ui.html--ff1f72803ea0.md#dynamic-outlines)

Position of outline with respect to the border edge is now only suggested, not required.

Conformant UAs are now allowed to ignore the 'invert' value. In such UAs the initial value of 'outline-color' is the value of the 'color' property.

<a id="q21.134"></a>

### <a id="s-12"></a>C.2.127 [Chapter 12 Generated content, automatic numbering, and lists](css2--generate.html--37748b674cd2.md)

The 'marker' value for 'display' does not exist in CSS 2.1

<a id="q21.135"></a>

### <a id="cA"></a>C.2.128 [Appendix A. Aural style sheets](css2--aural.html--2915ca92d45a.md)

Chapter 19 on aural style sheets has become appendix A and is not normative in CSS 2.1. Related units (deg, grad, rad, ms, s, Hz, kHz) are also moved to this appendix, as is the 'speak-header' property from the "tables" chapter and other notes on aural table rendering. The 'aural' media type is deprecated in favor of the new 'speech' media type.

<a id="q21.136"></a>

### <a id="cA.5"></a>C.2.129 [Appendix A Section 5 Pause properties](css2--aural.html--2915ca92d45a.md#pause-props)

Changed the initial value of 'pause-before' and 'pause-after' to be 0 instead of UA-defined.

A note has been added to this section (formerly 19.4) about the change in position and behavior of pauses in CSS3 Speech compared to this appendix.

<a id="q21.137"></a>

### <a id="cA.6"></a>C.2.130 [Appendix A Section 6 Cue properties](css2--aural.html--2915ca92d45a.md#cue-props)

This section (formerly Section 19.5) now specifies the placement of cues and pauses with respect to the :before and :after pseudo-elements.

<a id="q21.138"></a>

### <a id="cA.7"></a>C.2.131 [Appendix A Section 7 Mixing properties](css2--aural.html--2915ca92d45a.md#mixing-props)

The keywords 'mix' and 'repeat' may now appear in either order.

<a id="q21.139"></a>

### <a id="cB"></a>C.2.132 [Appendix B Bibliography](css2--refs.html--f208d881d0b7.md)

Various references in Appendix B (formerly Appendix E) have been updated as appropriate.

Switched \[CSS1\] from Normative to Informative.

Updated URI reference from \[RFC1808\] and the draft-fielding-uri-syntax-01.txt to \[RFC3986\].

Updated HTTP reference from \[RFC2068\] to \[RFC2616\].

Removed normative references to \[IANA\] and \[ICC32\].

Added normative references to \[ICC42\], \[RFC3986\], \[RFC2070\], \[UAAG10\].

Added informative references to CSS2, CSS3 Color, CSS3 Lists, Selectors, CSS3 Speech, DOM 3 Core, MathML 2, P3P, RFC1630, SVG 1.1, XHTML 1, XML ID, and XML Namespaces.

Removed informative references to \[ISO10179\] (DSSSL), \[INFINIFONT\], \[ISO9899\] (C), \[MONOTYPE\], \[NEGOT\], \[OPENTYPE\], \[PANOSE\], \[PANOSE2\], \[POSTSCRIPT\], \[RFC1866\] (HTML 2), \[RFC1942\] (HTML Tables), \[TRUETYPEGX\], \[W3CStyle\].

Updated language tags references from \[RFC1766\] to \[BCP47\].

<a id="q21.140"></a>

### <a id="other"></a>C.2.133 Other

The former informative appendix C, "Implementation and performance notes for fonts," is left out of CSS 2.1.

<a id="known-errors"></a>

## C.3 Errors

<a id="q21.142"></a>

### <a id="x-shorthand-inherit"></a>C.3.1 Shorthand properties

Shorthand properties take a list of subproperty values <em>or</em> the value 'inherit'. One cannot mix 'inherit' with other subproperty values as it would not be possible to specify the subproperty to which 'inherit' applied. The definitions of a number of shorthand properties did not enforce this rule: 'border-top', 'border-right', 'border-bottom', 'border-left', 'border', 'background', 'font', 'list-style', 'cue', and 'outline'.

<a id="q21.143"></a>

### <a id="x-applies-table"></a>C.3.2 Applies to

The "applies to" line of many property definitions has been made more accurate by excluding or including table display types where appropriate.

<a id="q21.144"></a>

### <a id="x4.1.1"></a>C.3.3 [Section 4.1.1](css2--syndata.html--02e71c159e14.md#tokenization) (and [G2](css2--grammar.html--eeb984c469da.md#grammar))

DELIM should not have included single or double quote. Refer also to section 4.1.6 on strings, which must have matching single or double quotes around them.

Removed "A-Z" from the "nmchar" token: as CSS is case insensitive anyway, it was redundant.

Corrected "unicode" macro to treat CRLF as a single character.

Corrected "block" production to allow white space between declarations.

In the rule for "any" (in the core syntax), corrected "FUNCTION" to "FUNCTION any\* ')'".

<a id="q21.145"></a>

### <a id="x4.1.3"></a>C.3.4 [Section 4.1.3 Characters and case](css2--syndata.html--02e71c159e14.md#characters)

Corrected third paragraph to say that an '@import' rule can only be preceded by an '@charset' rule or other '@import' rules.

<a id="q21.146"></a>

### <a id="x4.3"></a>C.3.5 [Section 4.3 (Double sign problem)](css2--syndata.html--02e71c159e14.md#values)

Several values described in subsections of this section incorrectly allowed two "+" or "-" signs at their beginnings.

<a id="q21.147"></a>

### <a id="x4.3.2"></a>C.3.6 [Section 4.3.2 Lengths](css2--syndata.html--02e71c159e14.md#length-units)

Fixed double sign error in definition of \<length\>. (\<number\> already has a sign.)

Corrected the suggested reference pixel to be based on a <u>96 dpi</u> device, not 90 dpi. The visual angle is thus about <u>0.0213 degrees</u> instead of 0.0227, and a pixel at arm's length is about <u>0.26 mm</u> instead of 0.28

Corrected last sentence to refer to a unsupported <em>used</em> length, not an unsupported specified length.

<a id="q21.148"></a>

### <a id="x4.3.3"></a>C.3.7 [Section 4.3.3 Percentages](css2--syndata.html--02e71c159e14.md#percentage-units)

Fixed double sign error in definition of \<percentage\>. (\<number\> already has a sign.)

<a id="q21.149"></a>

### <a id="x4.3.4"></a>C.3.8 [Section 4.3.4 URLs and URIs](css2--syndata.html--02e71c159e14.md#uri)

Defined escaping requirements in terms of the URI token so that no escaping requirements are missing from the prose.

Included invalid URIs in last paragraph about URI error handling.

<a id="q21.150"></a>

### <a id="x4.3.5"></a>C.3.9 [Section 4.3.5 Counters](css2--syndata.html--02e71c159e14.md#counter)

Corrected syntax of counter() and counters() notation to allow white space between tokens.

<a id="q21.151"></a>

### <a id="x4.3.6"></a>C.3.10 [Section 4.3.6 Colors](css2--syndata.html--02e71c159e14.md#color-units)

Deleted the comments about range restriction after the following examples:

```text

em { color: rgb(255,0,0) }
em { color: rgb(100%, 0%, 0%) }
```
<a id="q21.152"></a>

### <a id="x4.3.7"></a>C.3.11 [Section 4.3.7 Strings](css2--syndata.html--02e71c159e14.md#strings)

(Formerly section 4.3.10) Corrected text to allow all forms of Unicode escapes for U+000A, not just the "&#x5C;A" form, for including newlines in strings.

<a id="q21.153"></a>

### <a id="x5.10"></a>C.3.12 [Section 5.10 Pseudo-elements and pseudo-classes](css2--selector.html--f66f7c932788.md#pseudo-elements)

In the second bullet, added that the ':lang()' pseudo-class can also be deduced from the document in some cases.

<a id="q21.154"></a>

### <a id="x6.4"></a>C.3.13 [Section 6.4 The cascade](css2--cascade.html--c7aff33e6f0d.md#cascade)

Removed paragraph about imported style sheets being overridden by rules in the importing style sheet: imported style rules follow the cascade as specified in [6.4.1 Cascading order](css2--cascade.html--c7aff33e6f0d.md#cascading-order), exactly as if they were inserted in place of the @import rule.

<a id="q21.155"></a>

### <a id="x8.1"></a>C.3.14 [Section 8.1 Box Dimensions](css2--box.html--8875bbcdefe6.md#box-dimensions)

The definition of "content edge" has been changed to depend on 'width' and 'height' rather than directly on 'rendered content'.

From the definition of "padding edge", deleted the sentence "The padding edge of a box defines the edges of the containing block established by the box." For information about containing blocks, consult [Section 10.1](css2--visudet.html--12e8bc0e6b7c.md#containing-block-details).

<a id="q21.156"></a>

### <a id="x8.2"></a>C.3.15 [Section 8.2 Example of margins, padding, and borders](css2--box.html--8875bbcdefe6.md#mpb-examples)

The colors in the example HTML did not match the colors in the image.

<a id="q21.157"></a>

### <a id="x8.5.4"></a>C.3.16 [Section 8.5.4 Border shorthand properties](css2--box.html--8875bbcdefe6.md#border-shorthand-properties)

Changed various border shorthands' syntax definitions to use the \<border-width\>, \<border-style\> and \<'border-top-color'\> value types as appropriate.

<a id="q21.158"></a>

### <a id="x9.2.1"></a>C.3.17 [Section 9.2.1 Block-level elements and block boxes](css2--visuren.html--3f334c530cf4.md#containing-block)

Excepted table elements from second paragraph about principal block boxes and their contents.

Corrected sentence to say "either only block boxes or only inline boxes" instead of "only block boxes".

<a id="q21.159"></a>

### <a id="x9.3.1"></a>C.3.18 [Section 9.3.1 Choosing a positioning scheme](css2--visuren.html--3f334c530cf4.md#choose-position)

In the definition of "position: static", added 'right' and 'bottom' to the sentence saying that 'top' and 'left' do not apply.

<a id="q21.160"></a>

### <a id="x9.3.2"></a>C.3.19 [Section 9.3.2 Box offsets](css2--visuren.html--3f334c530cf4.md#position-props)

The properties 'top', 'right', 'bottom', and 'left', incorrectly referred to offsets with respect to a box's content edge. The proper edge is the margin edge. Thus, for 'top', the description now reads: "This property specifies how far a box's top margin edge is offset below the top edge of the box's containing block."

Corrected text under property definitions to say that for relatively-positioned elements, 'top', 'right', 'bottom', and 'left' define the offset from the box's position in the normal flow, not from the edges of the containing block. (The previous definition conflicted with that was further down; since that text is now redundant, it has been removed.)

<a id="q21.161"></a>

### <a id="x9.4.1"></a>C.3.20 [Section 9.4.1 Block formatting contexts](css2--visuren.html--3f334c530cf4.md#block-formatting)

In paragraph about relationship of a box's outer edges to its containing block's edges, corrected parenthetical to say that <em>line boxes</em>, not the <em>content area</em>, may shrink due to floats.

<a id="q21.162"></a>

### <a id="x9.4.2"></a>C.3.21 [Section 9.4.2 Inline formatting context](css2--visuren.html--3f334c530cf4.md#inline-formatting)

Added "and the presence of floats" to "The width of a line box is determined by a containing block".

<a id="q21.163"></a>

### <a id="x9.4.3"></a>C.3.22 [Section 9.4.3 Relative positioning](css2--visuren.html--3f334c530cf4.md#relative-positioning)

In the first paragraph, added "or floated" to the phrase "laid out according to the normal flow" as floated elements can be relatively positioned but are not part of the normal flow.

<a id="q21.164"></a>

### <a id="x9.5"></a>C.3.23 [Section 9.5 Floats](css2--visuren.html--3f334c530cf4.md#floats)

Corrected sentence about not enough horizontal room for the float to say that it is shifted downward until either it fits <em>or there
are no more floats present</em>.

<a id="q21.165"></a>

### <a id="x9.5.1"></a>C.3.24 [Section 9.5.1 Positioning the float](css2--visuren.html--3f334c530cf4.md#float-position)

Correct "Applies to" line and prose to say that the 'float' property can be set for any element but only applies to elements that are not absolutely positioned.

<a id="q21.166"></a>

### <a id="x9.5.2"></a>C.3.25 [Section 9.5.2 Controlling flow next to floats](css2--visuren.html--3f334c530cf4.md#flow-control)

Removed sentence saying that 'clear' may only be specified for block-level elements: it can be specified for any element, it only <em>applies to</em> block-level elements.

<a id="q21.167"></a>

### <a id="x9.6"></a>C.3.26 [Section 9.6 Absolute positioning](css2--visuren.html--3f334c530cf4.md#absolute-positioning)

Corrected sentence that said absolutely positioned boxes establish a new containing block for absolutely positioned descendants to except fixed positioned descendants.

<a id="q21.168"></a>

### <a id="x9.7"></a>C.3.27 [Section 9.7 Relationships between 'display', 'position', and 'float'](css2--visuren.html--3f334c530cf4.md#dis-pos-flo)

In rule 1, corrected "user agents must ignore 'position' and 'float" to "'position' and 'float' do not apply".

<a id="q21.169"></a>

### <a id="x9.10"></a>C.3.28 [Section 9.10 Text direction](css2--visuren.html--3f334c530cf4.md#direction)

Corrected note about 'direction' on table column elements to say that "columns are not the ancestors of the cells in the document tree" rather than saying "columns do not exist in the document tree".

Added table cells, table captions, and inline blocks alongside block-level elements in description of 'bidi-override' value. Also corrected the prose to handle anonymous child blocks.

Updated mention of Unicode's embedding limit from 15 to 61.

<a id="q21.170"></a>

### <a id="x10.1"></a>C.3.29 [Section 10.1 Definition of "containing block"](css2--visudet.html--12e8bc0e6b7c.md#containing-block-details)

Included table cells (and inline blocks) together with block-level elements in rule 2 defining the containing block of non-absolutely-positioned elements.

<a id="q21.171"></a>

### <a id="x10.3.3"></a>C.3.30 [Section 10.3.3 Block-level, non-replaced elements in normal flow](css2--visudet.html--12e8bc0e6b7c.md#blockwidth)

In the last sentence of the paragraph following the equation ("If the value of 'direction' is 'ltr', this happens to 'margin-left' instead") substituted 'rtl' for 'ltr'.

<a id="q21.172"></a>

### <a id="x10.4"></a>C.3.31 [Section 10.4 Minimum and maximum widths](css2--visudet.html--12e8bc0e6b7c.md#min-max-widths)

The initial value for 'min-width' is now '0' rather than UA-dependent.

Corrected "applies to" exception for both 'min-width' and 'max-width' from "table elements" to "table rows and row groups".

Specified that negative values for 'min-width' and 'max-width' are illegal.

<a id="q21.173"></a>

### <a id="x10.6.3"></a>C.3.32 [Section 10.6.3 Block-level non-replaced elements in normal flow when 'overflow' computes to 'visible'](css2--visudet.html--12e8bc0e6b7c.md#normal-block)

Added that 'auto' height also depends on whether the element has padding or borders, as these influence margin-collapsing behavior.

Added text to correctly account for margin collapsing behavior.

<a id="q21.174"></a>

### <a id="x10.7"></a>C.3.33 [Section 10.7 Minimum and maximum heights](css2--visudet.html--12e8bc0e6b7c.md#min-max-heights)

Corrected "applies to" exception for both 'min-width' and 'max-width' from "table elements" to "table columns and column groups".

Specified that negative values for 'min-height' and 'max-height' are illegal.

<a id="q21.175"></a>

### <a id="x11.1.1"></a>C.3.34 [Section 11.1.1 Overflow](css2--visufx.html--2bc674cb7ab0.md#overflow)

Corrected "applies to" line for 'overflow' from "block-level and replaced elements" to "non-replaced block-level elements, table cells, and inline-block elements".

The example of a DIV element containing a BLOCKQUOTE containing another DIV was not rendered correctly. The first style rule applied to both DIVs, so the second DIV box should have been rendered with a red border as well. The second DIV has now been changed to a CITE, which does not have a red border.

<a id="q21.176"></a>

### <a id="x11.1.2"></a>C.3.35 [Section 11.1.2 Clipping: the 'clip' property](css2--visufx.html--2bc674cb7ab0.md#clipping)

Corrected "rect (\<top\> \<right\> \<bottom\> \<left\>)" to "rect(\<top\>, \<right\>, \<bottom\>, \<left\>)".

<a id="q21.177"></a>

### <a id="x11.2"></a>C.3.36 [Section 11.2 Visibility](css2--visufx.html--2bc674cb7ab0.md#overflow-clipping)

Corrected initial value of 'visibility' to 'visible'.

<a id="q21.178"></a>

### <a id="x12.4.2"></a>C.3.37 [Section 12.4.2 Counter styles](css2--generate.html--37748b674cd2.md#counter-styles)

The example used the style 'hebrew', which does not exist in CSS level�2. Changed to 'lower-greek'.

<a id="q21.179"></a>

### <a id="x12.6.2"></a>C.3.38 [Section 12.6.2 Lists](css2--generate.html--37748b674cd2.md#list-style)

Under the 'list-style' property, the example:

```text

ul > ul { list-style: circle outside } /* Any UL child of a UL */
```
could never match valid HTML markup (since a UL element cannot be a child of another UL element). An LI has been inserted in between.

<a id="q21.180"></a>

### <a id="x14.2"></a>C.3.39 [Section 14.2 The background](css2--colors.html--5784063d2778.md#background)

Second sentence: "In terms of the box model, 'background' refers to the background of the content and the padding areas" now also mentions the border area. (See also [errata to section 8.1](#x8.1) above.) Thus:

> In terms of the box model, "background" refers to the background of the content, padding and border areas.

<a id="q21.181"></a>

### <a id="x14.2.1"></a>C.3.40 [Section 14.2.1 Background properties](css2--colors.html--5784063d2778.md#background-properties)

Under 'background-image', defined the image tile size used when the background image has intrinsic sizes specified in percentages or no intrinsic size.

Under 'background-repeat', the sentence "All tiling covers the content and padding areas \[...\]" has been corrected to

> "All tiling covers the content, padding <u>and border</u> areas \[...\]".

Under 'background-attachment', the value 'scroll' is defined to scroll with the "containing block" rather than with the "document". Also the sentence "Even if the image is fixed \[...\] background or padding area of the element" has been corrected to

> Even if the image is fixed, it is still only visible when it is in the background, padding <u>or border</u> area of the element.

<a id="q21.182"></a>

### <a id="x15.2"></a>C.3.41 [Section 15.2 Font matching algorithm](css2--fonts.html--d52fc14f36c2.md#algorithm)

In bullet 2, changed "the UA uses the 'font-family' descriptor" to "the UA uses the 'font-family' property".

<a id="q21.183"></a>

### <a id="x15.7"></a>C.3.42 [Section 15.7 Font size](css2--fonts.html--d52fc14f36c2.md#font-size-props)

The statement "Negative values are not allowed" for 'font-size' now applies to percentages as well as lengths.

<a id="q21.184"></a>

### <a id="x16.1"></a>C.3.43 [Section 16.1 Indentation](css2--text.html--467a8857ae69.md#indentation-prop)

Corrected 'text-indent' to apply to table cells (and inline blocks) as well as block-level elements.

<a id="q21.185"></a>

### <a id="x16.2"></a>C.3.44 [Section 16.2 Alignment](css2--text.html--467a8857ae69.md#alignment-prop)

Corrected 'text-align' to apply to table cells (and inline blocks) as well as block-level elements.

Changed prose about the effect of 'justify' to be less correct.

Corrected the note to say that justification is also dependent on the script, not just the language, of the text.

<a id="q21.186"></a>

### <a id="x17.2"></a>C.3.45 [Section 17.2 The CSS table model](css2--tables.html--201812dd6e3c.md#table-display)

In the definition of <strong>table-header-group</strong>, changed "footer" to "header" in "Print user agents may repeat footer rows on each page spanned by a table."

<a id="q21.187"></a>

### <a id="x17.2.1"></a>C.3.46 [Section 17.2.1 Anonymous table objects](css2--tables.html--201812dd6e3c.md#anonymous-boxes)

Added 'table-header-group' and 'table-footer-group' alongside mentions of 'table-row-group' where missing.

Corrected 'caption' to 'table-caption'.

Added missing rule (#3) for 'table-column' boxes.

Added 'table-caption' and 'table-column-group' to list of boxes requiring a 'table' or 'inline-table' parent in rule 4.

Added rules 5 and 6 to generate 'table-row' boxes where necessary for children of 'table'/'inline-table' and 'table-row-group'/'table-header-group'/'table-footer-group' boxes.

<a id="q21.188"></a>

### <a id="x17.4"></a>C.3.47 [Section 17.4 Tables in the visual formatting model](css2--tables.html--201812dd6e3c.md#model)

Specified handling of multiple caption boxes.

Specified that the anonymous outer table box is a 'block' box if the table is block-level and an 'inline-block' box if the table is inline-level but that the anonymous outer table box cannot accept run-ins.

<a id="q21.189"></a>

### <a id="x17.5"></a>C.3.48 [Section 17.5 Visual layout of table contents](css2--tables.html--201812dd6e3c.md#table-layout)

Correct text that said all internal table elements have padding; change to say that of these only table cells have padding.

The following note:

> <strong>Note.</strong> Table cells may be relatively and absolutely positioned, but this is not recommended: positioning and floating remove a box from the flow, affecting table alignment.

has been amended as follows:

> <strong>Note.</strong> Table cells may be positioned, but this is not recommended: absolute and fixed positioning, as well as floating, remove a box from the flow, affecting table size.

<a id="q21.190"></a>

### <a id="x17.5.1"></a>C.3.49 [Section 17.5.1 Table layers and transparency](css2--tables.html--201812dd6e3c.md#table-layers)

The rows and columns only cover the whole table in the collapsed borders model, not in the separated borders model.

The points 2, 3, 4 and 5 have been corrected to define the area covered by rows, columns, row groups and column groups and thus the positioning and painting of backgrounds on those elements.

Specify the handling of "missing cells".

<a id="q21.191"></a>

### <a id="x17.6.1"></a>C.3.50 [Section 17.6.1 The separated borders model](css2--tables.html--201812dd6e3c.md#separated-borders)

In the image, changed "cell-spacing" to "border-spacing".

<a id="q21.192"></a>

### <a id="x18.2"></a>C.3.51 [Section 18.2 System Colors](css2--ui.html--ff1f72803ea0.md#system-colors)

For the 'ButtonHighlight' value, changed the description from "Dark shadow" to "Highlight color".

<a id="q21.193"></a>

### <a id="xE.2"></a>C.3.52 [Section E.2 Painting order](css2--zindex.html--b486aef70d4d.md#painting-order)

Changed "but any descendants which actually create a new stacking context" to "but any <u>positioned descendants and</u> descendants which actually create a new stacking context" (3 times).

This change also occurred once in [section 9.5 (Floats)](css2--visuren.html--3f334c530cf4.md#floats) and once in section [section 9.9 (Layered presentation).](css2--visuren.html--3f334c530cf4.md#layers)

<a id="clarifications"></a>

## C.4 Clarifications

<a id="q21.195"></a>

### <a id="r2.1"></a>C.4.1 [Section 2.1 A brief CSS 2.1 tutorial for HTML](css2--intro.html--d40812139784.md#html-tutorial)

This section has been marked non-normative.

<a id="q21.196"></a>

### <a id="r2.2"></a>C.4.2 [Section 2.2 A brief CSS 2.1 tutorial for XML](css2--intro.html--d40812139784.md#xml-tutorial)

This section has been marked non-normative.

Added a statement about case-sensitivity of selectors for XML.

The specification for the [XML style sheet PI](https://www.w3.org/1999/06/REC-xml-stylesheet-19990629/) was written after CSS2 was finalized. The first line of the full XML example should not have been be

```text
<?XML:stylesheet type="text/css"
href="bach.css"?>
```
, but

```text
<?xml-stylesheet type="text/css" href="bach.css"?>
```
<a id="q21.197"></a>

### <a id="r2.3"></a>C.4.3 [Section 2.3 The CSS 2.1 processing model](css2--intro.html--d40812139784.md#processing-model)

This section has been marked non-normative.

<a id="q21.198"></a>

### <a id="r3.1"></a>C.4.4 [Section 3.1 Definitions](css2--conform.html--de58593b67d7.md#defs)

Added a note to clarify that the deprecated/non-deprecated status of a feature is distinct from its normative/non-normative status.

Under 'document language' clarified that CSS only describes the presentation of a document language, and has no effect on its semantics.

Changed definition of 'replaced element' to "an element whose content is outside the scope of the CSS formatting model" and added further clarifying text. This clarifies that e.g., SVG images embedded in an XML document are also considered replaced elements, not just those linked in from an outside file. Also changed definition of 'rendered content' to be consistent with this clarification.

Added under "Intrinsic dimension" that raster images without reliable resolution information are assumed to have a size of 1 px unit per image source pixel.

Added definition for 'ignore'.

Added definition for 'HTML user agent'.

Added definition for 'property'.

<a id="q21.199"></a>

### <a id="r4.1"></a>C.4.5 [Section 4.1 Syntax](css2--syndata.html--02e71c159e14.md#syntax)

Moved definitions of "immediately before" and "immediately after" forward so they apply to the whole Syntax section.

Added sections 4.1.2.1 and 4.1.2.2 to defined vendor-specific extensions.

<a id="q21.200"></a>

### <a id="r4.1.1"></a>C.4.6 [Section 4.1.1 Tokenization](css2--syndata.html--02e71c159e14.md#tokenization)

Clarified that input that cannot be parsed according to the core syntax is ignored according to the rules for handling parsing errors.

Clarified that input that cannot be tokenized or parsed has no meaning in CSS2.1.

<a id="q21.201"></a>

### <a id="r4.1.3"></a>C.4.7 [Section 4.1.3 Characters and case](css2--syndata.html--02e71c159e14.md#characters)

Clarified that when a CRLF pair terminates an escape sequence, the pair is treated as a single white space character as [corrected](#x4.1.1) in the tokenization rules.

Replaced "\[a-z0-9\]" by "\[a-zA-Z0-9\]" as an extra reminder that CSS identifiers are case-insensitive.

<a id="q21.202"></a>

### <a id="r4.1.7"></a>C.4.8 [Section 4.1.7 Rule sets, declaration blocks, and selectors](css2--syndata.html--02e71c159e14.md#declaration)

Replaced the term "{}-block" with "declaration block".

<a id="q21.203"></a>

### <a id="r4.2"></a>C.4.9 [Section 4.2 Rules for handling parsing errors](css2--syndata.html--02e71c159e14.md#parsing-errors)

Clarified that all property:value combinations and @-keywords that do not contain an identifier beginning with dash or underscore are reserved by CSS for future use.

Clarified that when something inside an at-rule is ignored because it is invalid, this does not make the entire at-rule invalid.

Referenced section 4.1.7 for parsing invalid bits inside declaration blocks.

<a id="q21.204"></a>

### <a id="r4.3.1"></a>C.4.10 [Section 4.3.1 Integers and real numbers](css2--syndata.html--02e71c159e14.md#numbers)

Clarified that '-0' is equivalent to '0' and is not a negative number.

<a id="q21.205"></a>

### <a id="r4.3.2"></a>C.4.11 [Section 4.3.2 Lengths](css2--syndata.html--02e71c159e14.md#length-units)

Clarified that negative length values on properties that do not allow them cause the declaration to be ignored.

<a id="q21.206"></a>

### <a id="r4.3.4"></a>C.4.12 [Section 4.3.4 URLs and URIs](css2--syndata.html--02e71c159e14.md#uri)

Reduced unnecessary discussion of what a URI is.

<a id="q21.207"></a>

### <a id="r5.1"></a>C.4.13 [Section 5.1 Pattern matching](css2--selector.html--f66f7c932788.md#pattern-matching)

Added note about terminology change ("simple selector") between CSS2 and CSS3.

<a id="q21.208"></a>

### <a id="r5.7"></a>C.4.14 [Section 5.7 Adjacent sibling selectors](css2--selector.html--f66f7c932788.md#adjacent-selectors)

Clarified that text nodes and comments do not affect whether a sibling selector matches.

<a id="q21.209"></a>

### <a id="r5.8.1"></a>C.4.15 [Section 5.8.1 Matching attributes and attribute values](css2--selector.html--f66f7c932788.md#matching-attrs)

Clarified ~= and \|= by using the definitions from the Selectors module.

<a id="q21.210"></a>

### <a id="r5.8.2"></a>C.4.16 [Section 5.8.2 Default attribute values in DTDs](css2--selector.html--f66f7c932788.md#default-attrs)

Clarified that rules about default attribute values are the same, whether the default is specified in a DTD or by other means.

<a id="q21.211"></a>

### <a id="r5.9"></a>C.4.17 [Section 5.9 ID selectors](css2--selector.html--f66f7c932788.md#id-selectors)

Added a note that it depends on the document format which attributes are ID attributes.

<a id="q21.212"></a>

### <a id="r5.11.3"></a>C.4.18 [Section 5.11.3 The dynamic pseudo-classes: :hover, :active, and :focus](css2--selector.html--f66f7c932788.md#dynamic-pseudo-classes)

Clarified that CSS 2.1 does <em>not</em> define if the parent of an element that matches ':active' or ':hover' itself also matches ':active' or ':hover'.

Added note that, in CSS1, ':active' only applies to links.

<a id="q21.213"></a>

### <a id="r5.11.4"></a>C.4.19 [Section 5.11.4 The language pseudo-class: :lang](css2--selector.html--f66f7c932788.md#lang)

Added a note to show the differences between ':lang(xx)' and '\[lang=xx\]'.

<a id="q21.214"></a>

### <a id="r5.12.2"></a>C.4.20 [Section 5.12.2 The :first-letter pseudo-element](css2--selector.html--f66f7c932788.md#first-letter)

Clarified that digits can also be first letter.

<a id="q21.215"></a>

### <a id="r6.2"></a>C.4.21 [Section 6.2 Inheritance](css2--cascade.html--c7aff33e6f0d.md#inheritance)

Clarified that computed values are inherited (not specified values) and that they become the specified value on the inheritor.

Removed discussion of "default" styles for a document.

<a id="q21.216"></a>

### <a id="r6.2.1"></a>C.4.22 [Section 6.2.1 The 'inherit' value](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit)

Clarify that 'inherit' can be used on properties that are not normally inherited and that when set on the root element, it has the effect of assigning the property's initial value.

<a id="q21.217"></a>

### <a id="r6.3"></a>C.4.23 [Section 6.3 The @import rule](css2--cascade.html--c7aff33e6f0d.md#at-import)

Except @charset from the statement that @imports must precede all other rules.

<a id="q21.218"></a>

### <a id="r6.4"></a>C.4.24 [Section 6.4 The Cascade](css2--cascade.html--c7aff33e6f0d.md#cascade)

Obfuscated note about system settings and UA limitations.

<a id="q21.219"></a>

### <a id="r6.4.1"></a>C.4.25 [Section 6.4.1 Cascading order](css2--cascade.html--c7aff33e6f0d.md#cascading-order)

Various editorial changes to clarify sort order.

<a id="q21.220"></a>

### <a id="r6.4.3"></a>C.4.26 [Section 6.4.3 Calculating a selector's specificity](css2--cascade.html--c7aff33e6f0d.md#specificity)

Added a note:

> The specificity is based only on the form of the selector. In particular, a selector of the form "`[id=p33]`" is counted as an attribute selector (a=0, b=1, c=0), even if the `id` attribute is defined as an "ID" in the source document's DTD.

<a id="q21.221"></a>

### <a id="r7.2.1"></a>C.4.27 [Section 7.2.1 The @media rule](css2--media.html--3a324a170379.md#at-media-rule)

Clarify that Style rules outside of @media rules apply to the same media types that the style sheet itself applies to.

<a id="q21.222"></a>

### <a id="r7.3"></a>C.4.28 [Section 7.3 Recognized media types](css2--media.html--3a324a170379.md#media-types)

Added text to clarify that media types are mutually exclusive, but a UA can render simultaneously to canvases with different media types.

<a id="q21.223"></a>

### <a id="r7.3.1"></a>C.4.29 [Section 7.3.1 Media groups](css2--media.html--3a324a170379.md#media-groups)

Split "aural" media group into "audio" and "speech".

<a id="q21.224"></a>

### <a id="r8.1"></a>C.4.30 [Section 8.1 Box dimensions](css2--box.html--8875bbcdefe6.md#box-dimensions)

- The terms "content box", "padding box", "border box", and "margin box" have been defined.

- Border backgrounds are not specified by border properties. Changed the last paragraph of 8.1 to:

  > The background style of the content, padding, and border areas of a box is specified by the ['background'](css2--colors.html--5784063d2778.md#propdef-background) property of the generating element. Margin backgrounds are always transparent.

- Removed definition of "box width" and "height".

<a id="q21.225"></a>

### <a id="r8.3"></a>C.4.31 [Section 8.3 Margin properties](css2--box.html--8875bbcdefe6.md#margin-properties)

Added a sentence to note that vertical margins have no effect on non-replaced inline elements.

<a id="q21.226"></a>

### <a id="r8.3.1"></a>C.4.32 [Section 8.3.1 Collapsing margins](css2--box.html--8875bbcdefe6.md#collapsing-margins)

Changed "absolute maximum" to "maximum of the absolute values" in sentence about negative margins collapsing.

Added this clarifying note to the first bullet of the explanation of vertical collapsing of margins:

> <strong>Note.</strong> Adjoining boxes may be generated by elements that are not related as siblings or ancestors.

Emphasized that floating elements' margins do not collapse even between a float and its in-flow children.

Emphasized that absolutely positioned elements' margins do not collapse even between the positioned element and its in-flow children.

<a id="q21.227"></a>

### <a id="r8.5.3"></a>C.4.33 [Section 8.5.3 Border style](css2--box.html--8875bbcdefe6.md#border-style-properties)

Changed description of 'none' value to not imply that all four border widths are set to zero.

<a id="q21.228"></a>

### <a id="r9.1.1"></a>C.4.34 [Section 9.1.1 The viewport](css2--visuren.html--3f334c530cf4.md#viewport)

Changed the sentence "When the viewport is smaller than the ..., the user agent should offer a scrolling mechanism" to use "area of the canvas on which the document is rendered" instead of "document's initial containing block".

<a id="q21.229"></a>

### <a id="r9.2.4"></a>C.4.35 [Section 9.2.4 The 'display' property](css2--visuren.html--3f334c530cf4.md#display-prop)

Clarified that 'display: none' also applies to non-visual media.

<a id="q21.230"></a>

### <a id="r9.3.1"></a>C.4.36 [Section 9.3.1 Choosing a positioning scheme](css2--visuren.html--3f334c530cf4.md#choose-position)

Clarified that the margins of fixed positioned boxes do not collapse with any other margins.

Clarified that in print media fixed boxes are rendered on every page.

<a id="q21.231"></a>

### <a id="r9.3.2"></a>C.4.37 [Section 9.3.2 Box offsets](css2--visuren.html--3f334c530cf4.md#position-props)

Clarified that negative lengths and percentages are allowed as values of 'top', 'right', 'bottom', and 'left'.

Added "For replaced elements, the effect of this value depends only on the intrinsic dimensions of the replaced content. See the sections on the width and height of absolutely positioned, replaced elements for details." to the definition of 'auto' because that's not what chapter 10 says at all.

<a id="q21.232"></a>

### <a id="r9.4.2"></a>C.4.38 [Section 9.4.2 Inline formatting context](css2--visuren.html--3f334c530cf4.md#inline-formatting)

Clarified that 'justify' stretches "spaces and words in inline boxes"; previous text simply said that it stretches "inline boxes".

The statement "When an inline box is split, margins, borders, and padding have no visual effect where the split occurs." has been generalized. Margins, borders, and padding have no visual effect where one or more splits occur.

Clarified that an inline box that exceeds the width of a line box and cannot be split therefore overflows the line box.

Removed sentence about formatting of margins, borders, and padding for split inline boxes not being fully defined when affected by bidi as that situation is now defined in [section 8.6](css2--box.html--8875bbcdefe6.md#bidi-box-model).

<a id="q21.233"></a>

### <a id="r9.4.3"></a>C.4.39 [Section 9.4.3 Relative positioning](css2--visuren.html--3f334c530cf4.md#relative-positioning)

Clarified that although relative positioning normally does not directly affect layout, it may affect layout indirectly through the creation of scrollbars.

Relatively positioned boxes do not always establish new containing blocks. Changed the second paragraph to refer to the section on containing blocks accordingly.

The paragraph about dynamic movement and superscripting has been shifted into a non-normative note.

<a id="q21.234"></a>

### <a id="r9.5"></a>C.4.40 [Section 9.5 Floats](css2--visuren.html--3f334c530cf4.md#floats)

Clarified that line boxes are shortened to make room for the <em>margin box</em> of the float.

Added some text to clarify what "Any content in the current line before a floated box is reflowed in the first available line on the other side of the float" means.

Clarified floats' position in the stacking order.

<a id="q21.235"></a>

### <a id="r9.5.1"></a>C.4.41 [Section 9.5.1 Positioning the float](css2--visuren.html--3f334c530cf4.md#float-position)

Clarified that the elements referenced in the float behavior rules are in the same block formatting context as the float.

<a id="q21.236"></a>

### <a id="r9.5.2"></a>C.4.42 [Section 9.5.2 Controlling flow next to floats](css2--visuren.html--3f334c530cf4.md#flow-control)

Clarified that the effects of 'clear' do not consider floats in other block formatting contexts.

<a id="q21.237"></a>

### <a id="r9.8"></a>C.4.43 [Section 9.8 Comparison of normal flow, floats, and absolute positioning](css2--visuren.html--3f334c530cf4.md#comparison)

Added a note to clarify that the images in this section are not drawn to scale and are illustrations, not reference renderings.

<a id="q21.238"></a>

### <a id="r10.1"></a>C.4.44 [Section 10.1 Definition of "containing block"](css2--visudet.html--12e8bc0e6b7c.md#containing-block-details)

Noted that a containing block formed by inline elements may wind up with a negative containing block width.

<a id="q21.239"></a>

### <a id="r10.2"></a>C.4.45 [Section 10.2 Content width](css2--visudet.html--12e8bc0e6b7c.md#the-width-property)

In the definition of \<length\> values for the 'width' property, changed "Specifies a fixed width" to "Specifies the width of the content area using a length unit".

<a id="q21.240"></a>

### <a id="r10.3.3"></a>C.4.46 [Section 10.3.3 Block-level, non-replaced elements in normal flow](css2--visudet.html--12e8bc0e6b7c.md#blockwidth)

Clarified that setting both left and right margins to 'auto' horizontally centers the element within its containing block.

<a id="q21.241"></a>

### <a id="r10.3.8"></a>C.4.47 [Section 10.3.8 Absolutely positioning, replaced elements](css2--visudet.html--12e8bc0e6b7c.md#abs-replaced-width)

Clarified which part of the text of section 10.3.7 is re-used.

<a id="q21.242"></a>

### <a id="r10.4"></a>C.4.48 [Section 10.4 Minimum and maximum widths](css2--visudet.html--12e8bc0e6b7c.md#min-max-widths)

Clarified that 'min-width' and 'max-width' do not affect the computed values of any properties. (They only affect the used value.)

<a id="q21.243"></a>

### <a id="r10.6.1"></a>C.4.49 [Section 10.6 Calculating heights and margins](css2--visudet.html--12e8bc0e6b7c.md#Computing_heights_and_margins)

Clarified that these rules apply to the root element just as to any other element.

<a id="q21.244"></a>

### <a id="r10.7"></a>C.4.50 [Section 10.7 Minimum and maximum heights](css2--visudet.html--12e8bc0e6b7c.md#min-max-heights)

Clarified that 'min-width' and 'max-width' do not affect the computed values of any properties. (They only affect the used value.)

<a id="q21.245"></a>

### <a id="r10.8"></a>C.4.51 [Section 10.8 Line height calculations](css2--visudet.html--12e8bc0e6b7c.md#line-height)

Removed clarifying note about line height being taller than tallest single inline box due to vertical alignment.

<a id="q21.246"></a>

### <a id="r10.8.1"></a>C.4.52 [Section 10.8.1 Leading and half-leading](css2--visudet.html--12e8bc0e6b7c.md#leading)

Removed "slightly" from the note "Values of this property have ~~slightly~~ different meanings in the context of tables."

<a id="q21.247"></a>

### <a id="r11.1"></a>C.4.53 [Section 11.1 Overflow and clipping](css2--visufx.html--2bc674cb7ab0.md#overflow-clipping)

Clarified when absolute positioning and negative margins cause overflow.

Added 'text-indent' to the list of things that can cause overflow.

Removed mention of 'clip' since it no longer affects most elements; mentioned that the 'overflow' property also specifies whether a scrolling mechanism is provided to access clipped content.

<a id="q21.248"></a>

### <a id="r11.1.1"></a>C.4.54 [Section 11.1.1 Overflow](css2--visufx.html--2bc674cb7ab0.md#overflow)

Clarified that descendant elements whose containing block is the viewport or an ancestor of the element are not affected by overflow clipping.

Removed unnecessary mentions of the 'clip' property from the 'hidden' value definition.

<a id="q21.249"></a>

### <a id="r11.1.2"></a>C.4.55 [Section 11.1.2 Clipping](css2--visufx.html--2bc674cb7ab0.md#clipping)

Changed "portion of an element's rendered content" to "portion of an element's border box" since clipping also affects the element's backgrounds and borders.

Clarified what parts of the element are affected by clipping.

Clarified that clipped content does not cause overflow.

Clarified that arguments of clip() can be separated by spaces or by commas, <em>but not a combination.</em>

<a id="q21.250"></a>

### <a id="r11.2"></a>C.4.56 [Section 11.2 Visibility](css2--visufx.html--2bc674cb7ab0.md#overflow-clipping)

Clarified that descendants of a 'visibility: hidden' element will be visible if they have 'visibility: visible'.

<a id="q21.251"></a>

### <a id="r12.1"></a>C.4.57 [Section 12.1 The :before and :after pseudo-elements](css2--generate.html--37748b674cd2.md#before-after-content)

Clarified that :before and :after pseudo-elements interact with other boxes as if they were real elements just inside their associated element.

Noted that the interaction of :before and :after with replaced elements is left undefined for now.

<a id="q21.252"></a>

### <a id="r12.2"></a>C.4.58 [Section 12.2 The 'content' property](css2--generate.html--37748b674cd2.md#content)

Clarified which counters are used for counter() and counters() in case there are multiple counters of the same name.

<a id="q21.253"></a>

### <a id="r12.3.2"></a>C.4.59 [Section 12.3.2 Inserting quotes with the 'content' property](css2--generate.html--37748b674cd2.md#quotes-insert)

Removed note about common typographic practices when quotes in different languages are mixed.

<a id="q21.254"></a>

### <a id="r12.4"></a>C.4.60 [Section 12.4 Automatic counters and numbering](css2--generate.html--37748b674cd2.md#counters)

In the "self-nesting" behavior of counters, clarified that merely using a counter in a child element does not create a new instance of it: only resetting it does.

Clarified that the scope of a counter does not include any elements in the scope of a counter with the same name created by a 'counter-reset' on a later sibling or a later 'counter-reset' on the same element.

Removed sentence about scope of 'counter-increment' without prior 'counter-reset' as that is now defined (differently) under "12.4.1 Nested counters and scope."

<a id="q21.255"></a>

### <a id="r12.4.3"></a>C.4.61 [Section 12.4.3 Counters in elements with 'display: none'](css2--generate.html--37748b674cd2.md#undisplayed-counters)

Clarified that pseudo-elements that generate no boxes also do not increment counters.

<a id="q21.256"></a>

### <a id="r14.2"></a>C.4.62 [Section 14.2 The background](css2--colors.html--5784063d2778.md#background)

Clarified that the root background image, although painted over the entire canvas, is anchored as if painted only for the root element, and that the root's background is only painted once.

Clarified rules for propagation of background settings on HTML's \<body\> element to the root.

Added statement about z-index of backgrounds for elements that form a stacking context and referred to z-index property for details.

Added this note after the first paragraph after 'background-attachment':

> Note that there is only <em>one</em> viewport per document. I.e., even if an element has a scrolling mechanism (see 'overflow'), a 'fixed' background does not move with it.

Definition of 'background-position' has been rewritten as normative rules rather than just examples.

Stated that the tiling and positioning of background images for inline elements is undefined in CSS2.1.

<a id="q21.257"></a>

### <a id="r15.1"></a>C.4.63 [Section 15.1 Fonts Introduction](css2--fonts.html--d52fc14f36c2.md#fonts-intro)

Drastically shortened introduction.

<a id="q21.258"></a>

### <a id="r15.2"></a>C.4.64 [Section 15.2 Font matching algorithm](css2--fonts.html--d52fc14f36c2.md#algorithm)

In the per-property rule 2, clarified that 'normal' matches the non-small-caps variant (if there is one).

<a id="q21.259"></a>

### <a id="r15.3"></a>C.4.65 [Section 15.2.2 Font family](css2--fonts.html--d52fc14f36c2.md#font-family-prop)

Removed discussion of font-matching algorithm. (It is already covered in the [font-matching algorithm's own section](css2--fonts.html--d52fc14f36c2.md#algorithm).

Clarified that quoted strings that are the same as a keyword value must be treated as font family names and not as the keyword value (which must be unquoted).

<a id="q21.260"></a>

### <a id="r15.3.1"></a>C.4.66 [Section 15.3.1 Generic font families](css2--fonts.html--d52fc14f36c2.md#generic-font-families)

This section, previously section 15.2.6, has been moved but no other change was made.

<a id="q21.261"></a>

### <a id="r15.4"></a>C.4.67 [Section 15.4 Font styling](css2--fonts.html--d52fc14f36c2.md#font-styling)

The text for this section (formerly part of 15.2.3) has been reverted to its CSS1 format.

<a id="q21.262"></a>

### <a id="r15.5"></a>C.4.68 [Section 15.5 Small-caps](css2--fonts.html--d52fc14f36c2.md#small-caps)

The text for this section (formerly part of 15.2.3) has been reverted to its CSS1 format.

Clarified that CSS2.1 cannot select font variants besides small-caps.

Clarified that when "font-variant: small-caps" results in the substitution of full-caps, the behavior is the same as for text-transform.

<a id="q21.263"></a>

### <a id="r15.6"></a>C.4.69 [Section 15.6 Font boldness](css2--fonts.html--d52fc14f36c2.md#font-boldness)

The text for this section (formerly part of 15.2.3) has been reverted to its CSS1 format. Also, discussion of font-weight from other parts of the Fonts chapter has been aggregated under this section.

Removed statement that says "User agents must map names to values in a way that preserves visual order; a face mapped to a value must not be lighter than faces mapped to lower values." This is otherwise implied by "The only guarantee is that a face of a given value will be no less dark than the faces of lighter values."

<a id="q21.264"></a>

### <a id="r15.7"></a>C.4.70 [Section 15.7 Font size](css2--fonts.html--d52fc14f36c2.md#font-size-props)

Clarified relationship of font size to em squares.

Added a totally irrelevant note about font sizes virtual reality scenes.

<a id="q21.265"></a>

### <a id="r16.1"></a>C.4.71 [Section 16.1 Indentation](css2--text.html--467a8857ae69.md#indentation-prop)

Clarified that text overflowing due to text-indent is affected by the 'overflow' property.

Added a note about text-indents inheriting behavior and suggesting 'text-indent: 0' on inline-blocks.

<a id="q21.266"></a>

### <a id="r16.2"></a>C.4.72 [Section 16.2 Alignment](css2--text.html--467a8857ae69.md#alignment-prop)

Changed "double justify" to "justify" under "left, right, center, and justify".

<a id="q21.267"></a>

### <a id="r16.3.1"></a>C.4.73 [Section 16.3.1 Underlining, over lining, striking, and blinking](css2--text.html--467a8857ae69.md#lining-striking-props)

Added an example to illustrate how underlining affects descendant boxes.

<a id="q21.268"></a>

### <a id="r16.5"></a>C.4.74 [Section 16.5 Capitalization](css2--text.html--467a8857ae69.md#caps-prop)

Switched language reference from RFC2070 to BCP47.

<a id="q21.269"></a>

### <a id="r16.6"></a>C.4.75 [Section 16.6 White space](css2--text.html--467a8857ae69.md#white-space-prop)

Added [section 16.6.1](css2--text.html--467a8857ae69.md#egbidiwscollapse) as an example to illustrate the interaction of white space collapsing and bidi.

<a id="q21.270"></a>

### <a id="r17.1"></a>C.4.76 [Section 17.1 Introduction to tables](css2--tables.html--201812dd6e3c.md#tables-intro)

Expanded introduction to include a brief discussion of the two table layout models. Mentioned that the automatic table algorithm is not fully defined in CSS 2.1 but that some implementations have achieved relatively close interoperability.

<a id="q21.271"></a>

### <a id="r17.2"></a>C.4.77 [Section 17.2 The CSS table model](css2--tables.html--201812dd6e3c.md#table-display)

Clarify that all table captions must be rendered if more than one exists.

Specified that replaced elements with table display values are treated as table elements in table layout.

<a id="q21.272"></a>

### <a id="r17.2.1"></a>C.4.78 [Section 17.2.1 Anonymous table objects](css2--tables.html--201812dd6e3c.md#anonymous-boxes)

Moved the first bullet text to the prose before the list of generation rules as it is a general statement of what the rules are supposed to accomplish.

<a id="q21.273"></a>

### <a id="r17.4"></a>C.4.79 [Section 17.4 Tables in the visual formatting model](css2--tables.html--201812dd6e3c.md#model)

Clarified that "display: table" elements behave as block-level elements and "display: inline-table" elements behave as inline-level elements and not the other way around.

Clarified that 'table-caption' boxes behave as normal block boxes within the outer anonymous table box.

Clarified that percentage 'width' and 'height' on the table box is relative to the anonymous box's containing block, not the anonymous box itself.

Clarified that the 'position', 'float', 'top', 'right', 'bottom', and 'left' values on the table box are used on the anonymous outer box instead of the table box and that the table box itself uses the initial values of those properties.

<a id="q21.274"></a>

### <a id="r17.5"></a>C.4.80 [Section 17.5 Visual layout of table contents](css2--tables.html--201812dd6e3c.md#table-layout)

To remove ambiguity about the position of extent of internal table boxes, the following paragraph was added after point 6:

> the edges of the rows, columns, row groups and column groups in the [collapsing borders model](css2--tables.html--201812dd6e3c.md#collapsing-borders) coincide with the hypothetical grid lines on which the borders of the cells are centered. (And thus, in this model, the rows together exactly cover the table, leaving no gaps; ditto for the columns.) In the [separated borders model,](css2--tables.html--201812dd6e3c.md#separated-borders) the edges coincide with the [border edges](css2--box.html--8875bbcdefe6.md#border-edge) of cells. (And thus, in this model, there may be gaps between the rows and columns, corresponding to the ['border-spacing'](css2--tables.html--201812dd6e3c.md#propdef-border-spacing) property.)

Changed warning note about positioning of table cells to be more precise about the possibly unintended effects.

<a id="q21.275"></a>

### <a id="r17.5.1"></a>C.4.81 [Section 17.5.1 Table layers and transparency](css2--tables.html--201812dd6e3c.md#table-layers)

At the end of the section added the following paragraph:

> Note that if the table has 'border-collapse: separate', the background of the area given by the 'border-spacing' property is always the background of the table element. See 17.6.1

<a id="q21.276"></a>

### <a id="r17.5.2"></a>C.4.82 [Section 17.5.2 Table width algorithms](css2--tables.html--201812dd6e3c.md#width-layout)

Added a paragraph to clarify the interaction of the table width algorithms with the rules in [section 10.3](css2--visudet.html--12e8bc0e6b7c.md#Computing_widths_and_margins) (Calculating widths and margins).

<a id="q21.277"></a>

### <a id="r17.5.2.1"></a>C.4.83 [Section 17.5.2.1 Fixed table layout](css2--tables.html--201812dd6e3c.md#fixed-table-layout)

Explicitly mentioned that the fixed table layout algorithm may be used with the algorithm of [section 10.3.3](css2--visudet.html--12e8bc0e6b7c.md#blockwidth) when 'table-layout' is 'fixed' but 'width' is 'auto'.

<a id="q21.278"></a>

### <a id="r17.5.2.2"></a>C.4.84 [Section 17.5.2.2 Automatic table layout](css2--tables.html--201812dd6e3c.md#auto-table-layout)

Clarified that UAs can use other algorithms besides the one in this section <em>even if</em> it results in different behavior. Also marked the rest of the section non-normative in accordance with that statement.

<a id="q21.279"></a>

### <a id="r17.5.4"></a>C.4.85 [Section 17.5.4 Horizontal alignment in a column](css2--tables.html--201812dd6e3c.md#column-alignment)

Changed "The horizontal alignment of a cell's content within a cell box is specified with the 'text-align' property" to "The horizontal alignment of a cell's <u>inline content</u> within a cell box <u>can be</u> specified with the 'text-align' property."

<a id="q21.280"></a>

### <a id="r17.5.5"></a>C.4.86 [Section 17.5.5 Dynamic row and column effects](css2--tables.html--201812dd6e3c.md#dynamic-effects)

Clarified that not affecting layout means that 'visibility: collapse' causes the part of row- and column-spanning cells that span into the collapsed row to be clipped.

<a id="q21.281"></a>

### <a id="r17.6.1"></a>C.4.87 [Section 17.6.1 The separated borders model](css2--tables.html--201812dd6e3c.md#separated-borders)

Added a note explaining that 'border-spacing' can be used as a substitute for the non-standard 'framespacing' attribute on frameset elements (which are out-of-scope for CSS2.1).

Added clarification about backgrounds: the sentence "This space is filled with the background of the table element" was replaced by:

> In this space, the row, column, row group, and column group backgrounds are invisible, allowing the table background to show through.

<a id="q21.282"></a>

### <a id="r17.6.2"></a>C.4.88 [Section 17.6.2 The collapsing borders model](css2--tables.html--201812dd6e3c.md#collapsing-borders)

In the sentence after the question, added "and padding-left<sub>i</sub> and padding-right<sub>i</sub> refer to the left (resp., right) padding of cell i."

<a id="q21.283"></a>

### <a id="r18.2"></a>C.4.89 [Section 18.2 System Colors](css2--ui.html--ff1f72803ea0.md#system-colors)

Noted that system colors are deprecated in CSS3.

<a id="q21.284"></a>

### <a id="r18.4"></a>C.4.90 [Section 18.4 Dynamic outlines](css2--ui.html--ff1f72803ea0.md#dynamic-outlines)

Clarified that outlines do not cause overflow.

Clarified that outlines are only fully connected "if possible".

<a id="q21.285"></a>

### <a id="r18.4.1"></a>C.4.91 [Section 18.4.1 Outlines and the focus](css2--ui.html--ff1f72803ea0.md#outline-focus)

Clarify that changing outlines in response to focus should not cause a document to reflow.

<a id="q21.286"></a>

### <a id="rD"></a>C.4.92 [Appendix D Default style sheet for HTML 4](css2--sample.html--133769fd2555.md)

Added paragraph clarifying that some presentational markup in HTML can be replaced with CSS, but it requires different markup.

<a id="errata"></a>

## C.5 Errata since the Candidate Recommendation of July 2007

Errata to CSS 2.1 since [CR version of July 19, 2007.](https://www.w3.org/TR/2007/CR-CSS21-20070719)

<a id="q21.288"></a>

### <a id="s.1.4.2.1"></a>C.5.1 [Section 1.4.2.1 Value](https://www.w3.org/TR/2007/CR-CSS21-20070719/about.html#value-defs)

\[2009-04-15\] The notation “&#x26;&#x26;” may be used in syntax definitions in future CSS specifications.

<a id="q21.289"></a>

### <a id="s.2.3"></a>C.5.2 [Section 2.3 The CSS 2.1 processing model](https://www.w3.org/TR/2007/CR-CSS21-20070719/intro.html#processing-model)

\[2008-08-19\] The first part of the section is not normative.

<a id="q21.290"></a>

### <a id="s.3.1"></a>C.5.3 [Section 3.1 Definitions](https://www.w3.org/TR/2007/CR-CSS21-20070719/conform.html#defs)

\[2007-11-14\] Append “<u>For raster images without reliable resolution information, a size of 1 px unit per image source pixel must be assumed.</u>” to the definition of “intrinsic dimensions.”

<a id="q21.291"></a>

### <a id="s.4.1.1"></a>C.5.4 [Section 4.1.1 Tokenization](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#tokenization)

\[2007-09-27\] Remove “DELIM?” from the grammar rule

```text
declaration : DELIM? property S* ':' S* value;
```
The DELIM was allowed there so that unofficial properties could start with a dash (-), but the dash was already allowed because of the definition of “IDENT.”

\[2009-02-02\] Change ~~U~~ to <u>u</u> in token UNICODE-RANGE. (It means the same, but seems to avoid confusion.)

\[2009-02-02\] Clarify where comments are allowed:

> COMMENT tokens do not occur in the grammar (to keep it readable), but any number of these tokens may appear anywhere ~~between~~ <u>outside</u> other tokens. <u>(Note, however, that a comment before or within the @charset rule disables the @charset.)</u>

<a id="q21.292"></a>

### <a id="s.4.1.2.2"></a>C.5.5 [Section 4.1.2.2 Informative Historical Notes](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#vendor-keyword-history)

\[2008-12-09\] Other known vendor prefixes are: -xv-, -ah-, prince-, -webkit-, and -khtml-.

<a id="q21.293"></a>

### <a id="s.4.1.3"></a>C.5.6 [Section 4.1.3 Characters and case](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#characters)

\[2007-11-14\] In the second bullet, change “~~\[a-z0-9\]~~” to “<u>\[a-zA-Z0-9\]</u>”; in the third bullet, change “~~\[0-9a-f\]~~” to “<u>\[0-9a-fA-F\]</u>”.

Although the preceding bullet already says that CSS is case-insensitive, the explicit mention of upper and lower case letters helps avoid mistakes.

<a id="q21.294"></a>

### <a id="s.4.1.3a"></a>C.5.7 [Section 4.1.3 Characters and case](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#characters)

\[2008-03-05\] CSS is now case-sensitive, except for certain parts:

> All CSS syntax is case-insensitive <u>within the ASCII range (i.e., \[a-z\] and \[A-Z\] are equivalent)</u>, except for parts that are not under the control of CSS.

<a id="q21.295"></a>

### <a id="s.4.1.3b"></a>C.5.8 [Section 4.1.3 Characters and case](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#characters)

\[2008-12-02\] The pair “\*/” ends a comment, even if preceded by a backslash. Change this sentence in the third bullet:

> <u>Except within CSS comments,</u> any character (except a hexadecimal digit) can be escaped with a backslash to remove its special meaning.

<a id="q21.296"></a>

### <a id="s.4.1.3c"></a>C.5.9 [Section 4.1.3 Characters and case](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#characters)

\[2009-04-15\] Text added to match the grammar:

> \[…\] any character (except a hexadecimal digit <u>, linefeed, carriage return or form feed</u>) can be escaped \[…\]

<a id="q21.297"></a>

### <a id="s.4.1.5"></a>C.5.10 [Section 4.1.5 At-rules](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#at-rules)

\[2009-04-15\] Clarified that unknown statements are ignored when looking for @import:

> CSS 2.1 user agents must ignore any '@import' rule that occurs inside a block or after any ~~valid~~ <u>non-ignored</u> statement other than an @charset or an @import rule.

<a id="q21.298"></a>

### <a id="s.4.1.7"></a>C.5.11 [Section 4.1.7 Rule sets, declaration blocks, and selectors](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#rule-sets)

\[2008-11-26\] More precise statement of what is ignored:

> When a user agent cannot parse the selector (i.e., it is not valid CSS 2.1), it must ignore the <u>selector and the following</u> declaration block <u>(if any)</u> as well.

<a id="q21.299"></a>

### <a id="s.4.2"></a>C.5.12 [Section 4.2 Rules for handling parsing errors](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#parsing-errors)

\[2009-04-15\] Added error recovery rule for unexpected tokens at the top level:

> <strong>Malformed statements.</strong> User agents must handle unexpected tokens encountered while parsing a statement by reading until the end of the statement, while observing the rules for matching pairs of (), \[\], {}, "", and '', and correctly handling escapes. For example, a malformed statement may contain an unexpected closing brace or at-keyword. E.g., the following lines are all ignored:
>
> ```text
> 
> p @here {color: red}     /* ruleset with unexpected at-keyword "@here" */
> @foo @bar;               /* at-rule with unexpected at-keyword "@bar" */
> }} {{ - }}               /* ruleset with unexpected right brace */
> ) [ {} ] p {color: red } /* ruleset with unexpected right parenthesis */
> ```
<a id="q21.300"></a>

### <a id="s.4.2a"></a>C.5.13 [Section 4.2 Rules for handling parsing errors](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#parsing-errors)

\[2008-11-26\] Change “or block” as follows:

> User agents must ignore an invalid at-keyword together with everything following it, up to and including the next semicolon (;), ~~or block ({...})~~ <u>the next block ({...}), or the end of the block (}) that contains the invalid at-keyword</u>, whichever comes first.

<a id="q21.301"></a>

### <a id="s.4.3.2"></a>C.5.14 [Section 4.3.2 Lengths](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#length-units)

\[2008-08-19\] Add recommendation about size of px:

> \[…\] the user agent should rescale pixel values. <u>It is recommended that the pixel unit refer to the whole number of device pixels that best approximates the reference pixel.</u>

<a id="q21.302"></a>

### <a id="s.4.3.5"></a>C.5.15 [Section 4.3.5 Counters](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#counter)

\[2008-03-05\] Insert “case-sensitive” in “Counters are denoted by <u>case-sensitive</u> identifiers”.

<a id="q21.303"></a>

### <a id="s.5.8.1"></a>C.5.16 [Section 5.8.1 Matching attributes and attribute values](https://www.w3.org/TR/2007/CR-CSS21-20070719/selector.html#matching-attrs)

\[2008-04-07\] Clarified ~= and \|= by using the definitions from the [Selectors](https://www.w3.org/TR/2005/WD-css3-selectors-20051215/#attribute-substrings) module.

\[2008-11-03\] Clarified that \[foo~=""\] (i.e., with an empty value) will not match anything.

<a id="q21.304"></a>

### <a id="s.5.8.2"></a>C.5.17 [Section 5.8.2 Default attribute values in DTDs](https://www.w3.org/TR/2007/CR-CSS21-20070719/selector.html#default-attrs)

\[2007-11-14\] Replace “~~tag~~ selector” by “<u>type</u> selector”.

<a id="q21.305"></a>

### <a id="s.5.11.4"></a>C.5.18 [Section 5.11.4 The language pseudo-class: :lang](https://www.w3.org/TR/2007/CR-CSS21-20070719/selector.html#lang)

\[2009-04-15\] The language code is case-insensitive.

<a id="q21.306"></a>

### <a id="s.5.12.3"></a>C.5.19 [Section 5.12.3 The :before and :after pseudo-elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/selector.html#before-and-after)

\[2008-11-03\] Clarified text:

> When the :first-letter and :first-line pseudo-elements are ~~combined with~~ <u>applied to an element having content generated using</u> :before and :after, they apply to the first letter or line of the element including the ~~inserted text~~ <u>generated content</u>.

<a id="q21.307"></a>

### <a id="s.6.3"></a>C.5.20 [Section 6.3 The @import rule](https://www.w3.org/TR/2007/CR-CSS21-20070719/cascade.html#at-import)

\[2008-08-19\] Add “In CSS 2.1” and “See the section on parsing for when user agents must ignore @import rules” to

> <u>In CSS 2.1,</u> any @import rules must precede all other rules (except the @charset rule, if present). <u>See the section on parsing for when user agents must ignore @import rules.</u>

<a id="q21.308"></a>

### <a id="s.6.3a"></a>C.5.21 [Section 6.3 The @import rule](https://www.w3.org/TR/2007/CR-CSS21-20070719/cascade.html#at-import)

\[2008-11-26\] Define what it means to import a style sheet twice and how the media list is matched. Add at the end:

> In the absence of any media types, the import is unconditional. Specifying 'all' for the medium has the same effect. <u>The import only takes effect if the target medium matches the media list.</u>
>
> <u>A target medium matches a media list if one of the items in the media list is the target medium or 'all'.</u>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > <u>Note that Media Queries \[MEDIAQ\] extends the syntax of media lists and the definition of matching.</u>
>
> <u>When the same style sheet is imported or linked to a document in multiple places, user agents must process (or act as though they do) each link as though the link were to a separate style sheet.</u>

<a id="q21.309"></a>

### <a id="s.6.4.1"></a>C.5.22 [Section 6.4.1 Cascading order](https://www.w3.org/TR/2007/CR-CSS21-20070719/cascade.html#cascading-order)

\[2007-11-22\] Spelling error: “prece~~n~~dence”.

<a id="q21.310"></a>

### <a id="s.6.4.1a"></a>C.5.23 [Section 6.4.1 Cascading order](https://www.w3.org/TR/2007/CR-CSS21-20070719/cascade.html#cascading-order)

\[2008-11-26\] Define the meaning of a media list:

> Find all declarations that apply to the element and property in question, for the target media type. Declarations apply if the associated selector matches the element in question <u>and the target medium matches the media list on all @media rules containing the declaration and on all links on the path through which the style sheet was reached.</u>

<a id="q21.311"></a>

### <a id="s.7.2.1"></a>C.5.24 [Section 7.2.1 The @media rule](https://www.w3.org/TR/2007/CR-CSS21-20070719/media.html#at-media-rule)

\[2008-12-02\] The rules for parsing unknown statements inside @media blocks were ambiguous. Change the first sentence as follows:

> An @media rule specifies the target media types (separated by commas) of a set of ~~rules~~ <u>statements</u> (delimited by curly braces). <u>Invalid statements must be ignored per 4.1.7 "Rule sets, declaration blocks, and selectors" and 4.2 "Rules for handling parsing errors."</u>

Also make it explicit that CSS level 2 (unlike higher levels) has no nested @-rules. Add at the end of the section: “<u>At-rules inside @media are invalid in CSS 2.1.</u>”

<a id="q21.312"></a>

### <a id="s.8.3.1"></a>C.5.25 [Section 8.3.1 Collapsing margins](https://www.w3.org/TR/2007/CR-CSS21-20070719/box.html#collapsing-margins)

\[2008-08-18\] In bullet 6, sub-bullet 2, the position of the top border edge is determined by assuming the element has a non-zero <em>bottom</em> (not: top) border.

<a id="q21.313"></a>

### <a id="s.8.3.1a"></a>C.5.26 [Section 8.3.1 Collapsing margins](https://www.w3.org/TR/2007/CR-CSS21-20070719/box.html#collapsing-margins)

\[2009-02-02\] Rephrased the rule for adjoining margins so that the 'min-height' and 'max-height' of an element have no influence over whether the element's bottom margin is adjoining to its last child's bottom margin.

<a id="q21.314"></a>

### <a id="s.8.3.1b"></a>C.5.27 [Section 8.3.1 Collapsing margins](https://www.w3.org/TR/2007/CR-CSS21-20070719/box.html#collapsing-margins)

\[2008-12-02\] Not only elements with 'overflow' other than 'visible', but <em>all</em> block formatting contexts avoid collapsing their margins with their children. Change the third bullet as follows:

> - Vertical margins of elements ~~with 'overflow' other than 'visible'~~ <u>that establish new block formatting contexts (such as floats and elements with 'overflow' other than 'visible')</u> do not collapse with their in-flow children.

<a id="q21.315"></a>

### <a id="s.9.2.2"></a>C.5.28 [Section 9.2.2 Inline-level elements and inline boxes](https://www.w3.org/TR/2007/CR-CSS21-20070719/visuren.html#inline-boxes)

\[2008-12-02\] Added missing 'inline-block' in: “Several values of the 'display' property make an element inline: 'inline', 'inline-table', <u>'inline-block'</u> and 'run-in' (part of the time; see run-in boxes).”

<a id="q21.316"></a>

### <a id="s.9.2.4"></a>C.5.29 [Section 9.2.4 The 'display' property](https://www.w3.org/TR/2007/CR-CSS21-20070719/visuren.html#display-prop)

\[2008-04-07\] Clarified that 'display: none' also applies to non-visual media.

<a id="q21.317"></a>

### <a id="s.9.3.2"></a>C.5.30 [Section 9.3.2 Box offsets: 'top', 'right', 'bottom', 'left'](https://www.w3.org/TR/2007/CR-CSS21-20070719/visuren.html#position-props)

\[2008-08-19\] Remove true but confusing note (occurs 4×):

> ~~Note: For absolutely positioned elements whose containing block is based on a block-level element, this property is an offset from the padding edge of that element.~~

<a id="q21.318"></a>

### <a id="s.9.5"></a>C.5.31 [Section 9.5 Floats](https://www.w3.org/TR/2007/CR-CSS21-20070719/visuren.html#floats)

\[2008-08-19\] Positioned descendants of a float are in the stacking context of the float's parent. Add “positioned elements and” to

> \[…\] except that any <u>positioned elements and</u> elements that actually create new stacking contexts take part in the float's parent's stacking context.

Same change in [Section 9.9 Layered presentation](https://www.w3.org/TR/2007/CR-CSS21-20070719/visuren.html#layers):

> \[…\] except that any <u>positioned elements and</u> any elements that actually create new stacking contexts take part in the parent stacking context.”

<a id="q21.319"></a>

### <a id="s.9.5a"></a>C.5.32 [Section 9.5 Floats](https://www.w3.org/TR/2007/CR-CSS21-20070719/visuren.html#floats)

\[2008-12-02\] Remove “'s” that may be misinterpreted: “the float's parent~~'s~~ stacking context.”

<a id="q21.320"></a>

### <a id="s.9.5.2"></a>C.5.33 [Section 9.5.2 Controlling flow next to floats: the 'clear' property](https://www.w3.org/TR/2007/CR-CSS21-20070719/visuren.html#flow-control)

\[2009-02-02\] Add an example of negative clearance after the first note.

<a id="q21.321"></a>

### <a id="s.9.6.1"></a>C.5.34 [Section 9.6.1 Fixed positioning](https://www.w3.org/TR/2007/CR-CSS21-20070719/visuren.html#fixed-positioning)

\[2008-11-03\] Added:

> <u>Boxes with fixed position that are larger than the page box are clipped. Parts of the fixed position box that are not visible in the initial containing block will not print.</u>

<a id="q21.322"></a>

### <a id="s.9.9.1"></a>C.5.35 [Section 9.9.1 Specifying the stack level: the 'z-index' property](https://www.w3.org/TR/2007/CR-CSS21-20070719/visuren.html#z-index)

\[2008-12-02\] The list of stacking levels is ambiguous: relatively positioned elements could fall under items 3/4/5 or under item 6. Meant is item 6, so exclude them from 3/4/5 as follows:

> 1.  the background and borders of the element forming the stacking context.
> 2.  the stacking contexts of descendants with negative stack levels.
> 3.  a stacking level containing in-flow non-inline-level <u>non-positioned</u> descendants.
> 4.  a stacking level for <u>non-positioned</u> floats and their contents.
> 5.  a stacking level for in-flow inline-level <u>non-positioned</u> descendants.
> 6.  a stacking level for positioned descendants with 'z-index: auto', and any descendant stacking contexts with 'z-index: 0'.
> 7.  the stacking contexts of descendants with positive stack levels.

<a id="q21.323"></a>

### <a id="s.10.1"></a>C.5.36 [Section 10.1 Definition of "containing block"](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#containing-block-details)

\[2009-02-02\] Rephrase first bullet point to make easier to read:

> ~~The containing block in which the root element lives is a rectangle with the dimensions of the viewport, anchored at the canvas origin for continuous media, and the page area for paged media. This containing block is called the initial containing block.~~

> <u>The containing block in which the root element lives is a rectangle called the initial containing block. For continuous media, it has the dimensions of the viewport and is anchored at the canvas origin; it is the page area for paged media.</u>

<a id="q21.324"></a>

### <a id="s.10.3"></a>C.5.37 [Section 10.3 Calculating widths and margins](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#Computing_widths_and_margins)

\[2009-04-15\] The values of 'left' and 'right' are only determined by section 9.4.3 in the case of relatively positioned elements:

> For Points 1-6 and 9-10, the values of 'left' and 'right' ~~used for layout~~ <u>in the case of relatively positioned elements</u> are determined by the rules in [section 9.4.3.](css2--visuren.html--3f334c530cf4.md#relative-positioning)

<a id="q21.325"></a>

### <a id="s.10.3.1"></a>C.5.38 [Section 10.3.1 Inline, non-replaced elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#inline-width)

\[2009-04-15\] The only case in which 'left' or 'right' can be 'auto' is when the element is statically positioned. In that case 'left' and 'right are ignored and there is thus no need to determine a used value:

> A computed value of 'auto' for ~~'left', 'right',~~ 'margin-left' or 'margin-right' becomes a used value of '0'.

<a id="q21.326"></a>

### <a id="s.10.3.2"></a>C.5.39 [Section 10.3.2 Inline, replaced elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#inline-replaced-width)

\[2007-11-14\] Add the following paragraph:

> <u>Otherwise, if 'width' has a computed value of 'auto', and the element has an intrinsic width, then that intrinsic width is the used value of 'width'.</u>

just before the paragraph beginning “Otherwise, if 'width' has a computed value of 'auto', but none of the conditions above are met, \[…\]”.

<a id="q21.327"></a>

### <a id="s.10.3.2a"></a>C.5.40 [Section 10.3.2 Inline, replaced elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#inline-replaced-width)

\[2008-03-05\] Change the last paragraph as follows:

> If it does, ~~then a percentage intrinsic width on that element cannot be resolved and the element is assumed to have no intrinsic width~~ <u>then the resulting layout is undefined in CSS2.1</u>.

<a id="q21.328"></a>

### <a id="s.10.3.3"></a>C.5.41 [Section 10.3.3 Block-level, non-replaced elements in normal flow](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#blockwidth)

\[2008-03-05\] Scrollbar widths are no longer included in the containing block width. Remove scrollbar width from:

> 'margin-left' + 'border-left-width' + 'padding-left' + 'width' + 'padding-right' + 'border-right-width' + 'margin-right' ~~+ scrollbar width (if any)~~ = width of containing block

and from:

> If 'width' is not 'auto' and 'border-left-width' + 'padding-left' + 'width' + 'padding-right' + 'border-right-width' ~~+ scrollbar width (if any)~~ \[...\]

and remove the paragraph:

> ~~The "scrollbar width" value is only relevant if the user agent uses a scrollbar as its scrolling mechanism. See the definition of the 'overflow' property.~~

<a id="q21.329"></a>

### <a id="s.10.3.7"></a>C.5.42 [Section 10.3.7 Absolutely positioned, non-replaced elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#abs-non-replaced-width)

\[2008-03-05\] Scrollbar widths are no longer included in the containing block width. Remove scrollbar width from:

> 'left' + 'margin-left' + 'border-left-width' + 'padding-left' + 'width' + 'padding-right' + 'border-right-width' + 'margin-right' + 'right' ~~+ scrollbar width (if any)~~ = width of containing block

and remove the paragraph:

> ~~The "scrollbar width" value is only relevant if the user agent uses a scrollbar as its scrolling mechanism. See the definition of the 'overflow' property.~~

<a id="q21.330"></a>

### <a id="s.10.3.7a"></a>C.5.43 [Section 10.3.7 Absolutely positioned, non-replaced elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#abs-non-replaced-width)

\[2008-03-05\] Add the following definition.

\[2008-08-19\] Add the following note to that definition.

> <u>The static-position containing block is the containing block of a hypothetical box that would have been the first box of the element if its specified 'position' property had been 'static' and its 'float' had been 'none'. (Note that due to the rules in section 9.7 this hypothetical calculation might require also assuming a different computed value for 'display'.)</u>

And change which 'direction' property is used as follows (two occurrences):

> \[...\] if the 'direction' property of the <u>element establishing the static-position</u> containing block is \[...\]

<a id="q21.331"></a>

### <a id="s.10.3.8"></a>C.5.44 [Section 10.3.8 Absolutely positioned, replaced elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#abs-replaced-width)

\[2008-03-05\] Change bullet 2 as follows:

> \[...\] if <u>the</u> 'direction' <u>property</u> of the <u>element establishing the static-position</u> containing block is \[...\]

<a id="q21.332"></a>

### <a id="s.10.3.8a"></a>C.5.45 [Section 10.3.8 Absolutely positioned, replaced elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#abs-replaced-width)

\[2008-03-05\] Clarification. Replace

> ~~This situation is similar to the previous one, except that the element has an intrinsic width. The sequence of substitutions is now:~~

by

> <u>In this case, section 10.3.7 applies up through and including the constraint equation, but the rest of section 10.3.7 is replaced by the following rules:</u>

<a id="q21.333"></a>

### <a id="s.10.3.8c"></a>C.5.46 [Section 10.3.8 Absolutely positioned, replaced elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#abs-replaced-width)

\[2008-04-07\] Clarified that margins are <em>not</em> calculated as for inline elements.

<a id="q21.334"></a>

### <a id="s.10.5"></a>C.5.47 [Section 10.5 Content height: the 'height' property](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#the-height-property)

Under “\<percentage\>,” add the same note as under “\<percentage\>,” in section 10.2 (“Content width: the 'width' property”).

<a id="q21.335"></a>

### <a id="s.10.6.2"></a>C.5.48 [Section 10.6.2 Inline replaced elements \[…\]](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#inline-replaced-height)

\[2007-11-14\] Add the following paragraph:

> <u>Otherwise, if 'height' has a computed value of 'auto', and the element has an intrinsic height, then that intrinsic height is the used value of 'height'.</u>

just before the paragraph beginning “Otherwise, if 'height' has a computed value of 'auto', but none of the conditions above are met \[…\]”.

<a id="q21.336"></a>

### <a id="s.10.6.4"></a>C.5.49 [Section 10.6.4 Absolutely positioned, non-replaced elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#abs-non-replaced-height)

\[2008-11-26\] The static position is determined considering <em>neither</em> float <em>nor</em> clear. Add this:

> \[…\] and its specified 'float' had been 'none' <u>and 'clear' had been 'none'</u>.

<a id="q21.337"></a>

### <a id="s.10.6.5"></a>C.5.50 [Section 10.6.5 Absolutely positioned, replaced elements](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#abs-replaced-height)

\[2008-04-07\] Clarified that margins are <em>not</em> calculated as for inline elements.

<a id="q21.338"></a>

### <a id="s.10.8.1"></a>C.5.51 [Section 10.8.1 Leading and half-leading](https://www.w3.org/TR/2007/CR-CSS21-20070719/visudet.html#leading)

\[2007-11-14\] In the Note under 'vertical-align', remove “slightly” from “Values of this property have ~~slightly~~ different meanings in the context of tables.”

<a id="q21.339"></a>

### <a id="s.11.1.1"></a>C.5.52 [Section 11.1.1 Overflow: the 'overflow' property](https://www.w3.org/TR/2007/CR-CSS21-20070719/visufx.html#overflow)

\[2008-03-05\] Scrollbar widths are no longer included in the containing block width. Replace

> ~~The space taken up by the scrollbars affects the computation of the dimensions in the rendering model.~~

by

> <u>Any space taken up by the scrollbars should be taken out of (subtracted from the dimensions of) the containing block formed by the element with the scrollbars.</u>

\[2008-11-03\] 'Overflow' on BODY is special not only in HTML but also in XHTML. Change the sentence “~~HTML UAs must instead apply the 'overflow' property from the BODY element to the viewport, if the value on the HTML element is 'visible'.~~” to:

> <u>When the root element is an HTML "HTML" element or an XHTML "html" element, and that element has an HTML "BODY" element or an XHTML "body" element as a child, user agents must instead apply the 'overflow' property from the first such child element to the viewport, if the value on the root element is 'visible'.</u>

<a id="q21.340"></a>

### <a id="s.11.1.2"></a>C.5.53 [Section 11.1.2 Clipping: the 'clip' property](https://www.w3.org/TR/2007/CR-CSS21-20070719/visufx.html#clipping)

\[2008-03-05\] Insert “(but not a combination)” in “User agents must support separation with commas, but may also support separation without commas <u>(but not a combination)</u>”.

<a id="q21.341"></a>

### <a id="s.12.2"></a>C.5.54 [Section 12.2 The 'content' property](https://www.w3.org/TR/2007/CR-CSS21-20070719/generate.html#content)

\[2009-04-15\] (And also in section 12.4:) certain keywords, in particular 'none', 'inherit' and 'initial' (the latter being reserved for future use) cannot be used as names for counters.

<a id="q21.342"></a>

### <a id="s.12.4.2"></a>C.5.55 [Section 12.4.2 Counter styles](https://www.w3.org/TR/2007/CR-CSS21-20070719/generate.html#counter-styles)

\[2008-03-05\] Error in example. Replace hebrew by lower-greek:

```text
BLOCKQUOTE:after { content: " [" counter(bq, hebrew lower-greek) "]" }
```
<a id="q21.343"></a>

### <a id="s.12.5"></a>C.5.56 [Section 12.5 Lists](https://www.w3.org/TR/2007/CR-CSS21-20070719/generate.html#lists)

\[2008-12-01\] Change “in” to “with respect to” in

> The list properties describe basic visual formatting of lists: they allow style sheets to specify the marker type (image, glyph, or number), and the marker position ~~in~~ <u>with respect to</u> the principal box (outside it or within it before content).

because the marker is, as the rest of the sentence itself makes clear, not necessarily <em>in</em> the principal box.

<a id="q21.344"></a>

### <a id="s.12.5.1"></a>C.5.57 [Section 12.5.1 Lists: the 'list-style-type', 'list-style-image', 'list-style-position', and 'list-style' properties](https://www.w3.org/TR/2007/CR-CSS21-20070719/generate.html#list-style)

\[2008-04-07\] The size of list style markers without an intrinsic size is now defined.

<a id="q21.345"></a>

### <a id="s.12.5.1a"></a>C.5.58 [Section 12.5.1 Lists: the 'list-style-type', 'list-style-image', 'list-style-position', and 'list-style' properties](https://www.w3.org/TR/2007/CR-CSS21-20070719/generate.html#list-style)

\[2008-12-01\] CSS 2.1 does not specify the position of the list item marker, but <em>does</em> require it to be on the left or right of the content. Also, the marker is not affected by 'overflow', but <em>may</em> influence the height of the principal box. Add to the definition of 'outside':

> <u>… but does require that for list items whose 'direction' property is 'ltr' the marker box be on the left side of the content and for elements whose 'direction' property is 'rtl' the marker box be on the right side of the content. 'Overflow' on the element does not clip the marker box. The marker box is fixed with respect to the principal block box's border and does not scroll with the principal block box's content. The size or contents of the marker box may affect the height of the principal block box and/or the height of its first line box, and in some cases may cause the creation of a new line box. Note: This interaction may be more precisely defined in a future level of CSS.</u>

<a id="q21.346"></a>

### <a id="s.12.5.1b"></a>C.5.59 [Section 12.5.1 Lists: the 'list-style-type', 'list-style-image', 'list-style-position', and 'list-style' properties](https://www.w3.org/TR/2007/CR-CSS21-20070719/generate.html#list-style)

\[2009-04-015\] Meaning of 'none' for 'list-style' was only defined by an example.

<a id="q21.347"></a>

### <a id="s.13.2"></a>C.5.60 [Section 13.2 Page boxes: the @page rule](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#page-box)

\[2008-08-19\] Add rules for drawing canvas to:

> - The page area. The page area includes the boxes laid out on that page. The edges of the first page area establish the rectangle that is the initial containing block of the document. <u>The canvas background is painted within and covers the page area.</u>
> - The margin area, which surrounds the page area. <u>The page margin area is transparent.</u>

<a id="q21.348"></a>

### <a id="s.13.2.1.1"></a>C.5.61 [Section 13.2.1.1 Rendering page boxes that do not fit a target sheet](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#oversized-page-box)

\[2009-02-02\]

Remove sections 13.2.1.1 and 13.2.1.2. (The described situations cannot occur in CSS 2.1, because CSS 2.1 does not have a 'size' property.)

<a id="q21.349"></a>

### <a id="s.13.2.3"></a>C.5.62 [Section 13.2.3 Content outside the page box](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#outside-page-box)

\[2008-11-03\] Clarified what locations are inconvenient for printing:

> When formatting content in the page model, some content may end up outside the <u>current</u> page box. For example, an element whose 'white-space' property has the value 'pre' may generate a box that is wider than the page box. <u>As another example,</u> when boxes are positioned absolutely <u>or relatively</u>, they may end up in “inconvenient” locations. For example, images may be placed on the edge of the page box or 100,000 meters below the page box.

<a id="q21.350"></a>

### <a id="s.13.3.1"></a>C.5.63 [Section 13.3.1 Page break properties: 'page-break-before', 'page-break-after', 'page-break-inside'](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#page-break-props)

\[2008-04-30\] The 'page-break-inside' property no longer inherits.

<a id="q21.351"></a>

### <a id="s.13.3.1a"></a>C.5.64 [Section 13.3.1 Page break properties: 'page-break-before', 'page-break-after', 'page-break-inside'](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#page-break-props)

\[2008-12-01\] UAs <em>may</em> apply 'page-break-before', 'page-break-after' and 'page-break-inside' to other elements than block-level ones.

<a id="q21.352"></a>

### <a id="s.13.3.2"></a>C.5.65 [Section 13.3.2 Breaks inside elements: 'orphans', 'widows'](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#break-inside)

\[2009-02-02\] “Paragraph” is not a defined term. Change ~~of a paragraph~~ to <u>in a block element</u> (twice).

<a id="q21.353"></a>

### <a id="s.13.3.2a"></a>C.5.66 [Section 13.3.2 Breaks inside elements: 'orphans', 'widows'](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#break-inside)

\[2009-04-15\] 'Widows' and 'orphans' only accept positive values.

<a id="q21.354"></a>

### <a id="s.13.3.3"></a>C.5.67 [Section 13.3.3 Allowed page breaks](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#allowed-page-breaks)

\[2008-04-30\] The 'page-break-inside' property of all ancestors is checked for page-breaking restrictions, not just that of the breakpoint's parent.

<a id="q21.355"></a>

### <a id="s.13.3.3a"></a>C.5.68 [Section 13.3.3 Allowed page breaks](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#allowed-page-breaks)

\[2009-02-02\] Remove possible confusion:

> Rule D: In addition, breaking at (2) is allowed only if the 'page-break-inside' property of <u>the element and</u> all <u>its</u> ancestors is 'auto'.

<a id="q21.356"></a>

### <a id="s.13.3.3b"></a>C.5.69 [Section 13.3.3 Allowed page breaks](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#allowed-page-breaks)

\[2009-02-02\] Top margins do <em>not</em> disappear at a page break that is forced by a 'page-break-after' or 'page-break-before'. Correct the first bullet to:

> When an <u>unforced</u> page break occurs here, the used values of the relevant 'margin-top' and 'margin-bottom' properties are set to '0'. <u>When a forced page break occurs here, the used value of the relevant 'margin-bottom' property is set to '0'; the relevant 'margin-top' used value may either be set to '0' or retained.</u>

And add the following note:

> <u>Note: It is expected that CSS3 will specify that the relevant 'margin-top' applies (i.e., is not set to '0') after a forced page break.</u>

<a id="q21.357"></a>

### <a id="s.13.3.5"></a>C.5.70 [Section 13.3.5 "Best" page breaks](https://www.w3.org/TR/2007/CR-CSS21-20070719/page.html#best-page-breaks)

\[2009-02-02\] Remove the advice to user agents to avoid breaking inside elements with borders, inside tables or inside floating elements; add the advice to avoid breaking inside replaced elements.

<a id="q21.358"></a>

### <a id="s.14.2"></a>C.5.71 [Section 14.2 The background](https://www.w3.org/TR/2007/CR-CSS21-20070719/colors.html#background)

\[2008-11-03\] The 'background' property is special on BODY not only in HTML�but also in XHTML.

<a id="q21.359"></a>

### <a id="s.14.2a"></a>C.5.72 [Section 14.2 The background](https://www.w3.org/TR/2007/CR-CSS21-20070719/colors.html#background)

\[2009-04-15\] The whole 'background' property is used for the canvas, not just the color and the image:

> For documents whose root element is an HTML "HTML" element or an XHTML "html" element that has computed values of 'transparent' for 'background-color' and 'none' for 'background-image', user agents must instead use the computed value of ~~those~~ <u>the background</u> properties from that element's first HTML "BODY" element or XHTML "body" element child \[…\]

<a id="q21.360"></a>

### <a id="s.14.2.1a"></a>C.5.73 [Section 14.2.1 Background properties: 'background-color', 'background-image', 'background-repeat', 'background-attachment', 'background-position', and 'background'](https://www.w3.org/TR/2007/CR-CSS21-20070719/colors.html#background-properties)

\[2008-04-07\] The size of background images without an intrinsic size is now defined.

<a id="q21.361"></a>

### <a id="s.15.6"></a>C.5.74 [Section 15.6 Font boldness: the 'font-weight' property](https://www.w3.org/TR/2007/CR-CSS21-20070719/fonts.html#font-boldness)

\[2008-11-26\] Remove incorrect text:

> - 'bolder' selects the next weight that is assigned to a font that is darker than the inherited one. ~~If there is no such weight, it simply results in the next darker numerical value (and the font remains unchanged), unless the inherited value was '900' in which case the resulting weight is also '900'.~~
> - 'lighter' is similar, but works in the opposite direction: it selects the next lighter keyword with a different font from the inherited one, ~~unless there is no such font, in which case it selects the next lighter numerical value (and keeps the font unchanged)~~.

and:

> ~~The computed value of "font-weight" is either:~~
>
> - ~~one of the legal number values, or~~
> - ~~one of the legal number values combined with one or more of the relative values (bolder or lighter). This type of computed values is necessary to use when the font in question does not have all weight variations that are needed.~~

And instead add this note:

> <u>Note: A set of nested elements that mix 'bolder' and 'lighter' will give unpredictable results depending on the UA, OS, and font availability. This behavior will be more precisely defined in CSS3.</u>

<a id="q21.362"></a>

### <a id="s.16.6"></a>C.5.75 [Section 16.6 Whitespace: the 'white-space' property](https://www.w3.org/TR/2007/CR-CSS21-20070719/text.html#white-space-prop)

\[2008-08-19\] Remove rules about generated text from:

> The following examples show what whitespace behavior is expected from the PRE and P elements, the “nowrap” attribute in HTML, ~~and in generated content~~.
>
> ```text
> pre        { white-space: pre }
> p          { white-space: normal }
> td[nowrap] { white-space: nowrap }
> :before,:after { white-space: pre-line }
> ```
<a id="q21.363"></a>

### <a id="s.16.6.1"></a>C.5.76 [Section 16.6.1 The 'white-space' processing model](https://www.w3.org/TR/2007/CR-CSS21-20070719/text.html#white-space-model)

\[2009-02-02\] Collapsing of white space does not remove any line breaking opportunities. Add the following clarification:

> Then, the entire block is rendered. Inlines are laid out, taking bidi reordering into account, and wrapping as specified by the 'white-space' property. <u>When wrapping, line breaking opportunities are determined based on the text prior to the white space collapsing steps above.</u>

<a id="q21.364"></a>

### <a id="s.17.2.1"></a>C.5.77 [Section 17.2.1 Anonymous table objects](https://www.w3.org/TR/2007/CR-CSS21-20070719/tables.html#anonymous-boxes)

\[2007-11-14\] Spelling error: “boxes~~s~~”.

<a id="q21.365"></a>

### <a id="s.17.2.1a"></a>C.5.78 [Section 17.2.1 Anonymous table objects](https://www.w3.org/TR/2007/CR-CSS21-20070719/tables.html#anonymous-boxes)

\[2008-10-13\] Added new rule after bullet 4:

> <u>5. If a child T of a 'table', 'inline-table', 'table-row-group', 'table-header-group', 'table-footer-group', or 'table-row' box is an anonymous inline box that contains only white space, then it is treated as if it has 'display: none'.</u>

<a id="q21.366"></a>

### <a id="s.17.4"></a>C.5.79 [Section 17.4 Tables in the visual formatting model](https://www.w3.org/TR/2007/CR-CSS21-20070719/tables.html#model)

\[2009-02-02\] The anonymous block containing the table and its caption establishes a block formatting context:

> The anonymous box is a 'block' box if the table is block-level, and an 'inline-block' box if the table is inline-level ~~except that this block is never considered as a block for 'run-in' interaction, and that~~ <u>The anonymous box establishes a block formatting context.</u> The table box (not the anonymous box) is used when doing baseline vertical alignment for an 'inline-table'.

The diagram now shows the caption's margins <em>inside</em> the anonymous box.

<a id="q21.367"></a>

### <a id="s.17.5.4a"></a>C.5.80 [Section 17.5.4 Horizontal alignment in a column](https://www.w3.org/TR/2007/CR-CSS21-20070719/tables.html#column-alignment)

\[2008-04-07\] Clarification:

> The horizontal alignment of a cell's <u>inline</u> content within a cell box ~~is~~ <u>can be</u> specified ~~with the 'text-align' property~~ <u>by the value of the 'text-align' property on the cell</u>.

<a id="q21.368"></a>

### <a id="s.18.1"></a>C.5.81 [Section 18.1 Cursors: the 'cursor' property](https://www.w3.org/TR/2007/CR-CSS21-20070719/ui.html#cursor-props)

\[2008-04-07\] The size of cursors without an intrinsic size is now defined.

<a id="q21.369"></a>

### <a id="s.B.2"></a>C.5.82 [Section B.2 Informative references](https://www.w3.org/TR/2007/CR-CSS21-20070719/refs.html#q2)

\[2007-11-14\] Spelling error: change “~~?~~lik” to “<u>Ç</u>elik” (2×).

<a id="q21.370"></a>

### <a id="s.D"></a>C.5.83 [Appendix D. Default style sheet for HTML 4](https://www.w3.org/TR/2007/CR-CSS21-20070719/sample.html)

\[2008-08-19\] Replace

```text
br:before       { content: "\A" }
:before, :after { white-space: pre-line }
```
with

```text
br:before       { content: "\A"; white-space: pre-line }
```
<a id="q21.371"></a>

### <a id="s.Da"></a>C.5.84 [Appendix D. Default style sheet for HTML 4](https://www.w3.org/TR/2007/CR-CSS21-20070719/sample.html)

\[2008-08-19\] Add tr to:

```text
td, th, tr      { vertical-align: inherit }
```
<a id="q21.372"></a>

### <a id="s.E.2"></a>C.5.85 [Section E.2 Painting order](https://www.w3.org/TR/2007/CR-CSS21-20070719/zindex.html#painting-order)

\[2007-11-14\] Replace “but any descendants which actually create a new stacking context” by “but any <u>positioned descendants and</u> descendants which actually create a new stacking context”.

<a id="q21.373"></a>

### <a id="s.G"></a>C.5.86 [Appendix G. Grammar of CSS 2.1](https://www.w3.org/TR/2007/CR-CSS21-20070719/grammar.html)

\[2007-09-27\] Change the last “S” in the grammar rule for “combinator” to “S+”:

```text
combinator
  : PLUS S*
  | GREATER S*
  | S+
```
and remove the rule

```text
{s}+\/\*[^*]*\*+([^/*][^*]*\*+)*\/  {unput(' '); /*replace by space*/}
```
in the tokenizer. The resulting language is the same, but the grammar is easier to read and relies less on specific notations of Flex.

<a id="q21.374"></a>

### <a id="s.G.1"></a>C.5.87 [Section G.1 Grammar](https://www.w3.org/TR/2007/CR-CSS21-20070719/grammar.html#grammar)

\[2007-09-27\] Changes to remove ambiguity with respect to the S token and avoid nullable non-terminals.

<a id="q21.375"></a>

### <a id="s.G.2"></a>C.5.88 [Section G.2 Lexical scanner](https://www.w3.org/TR/2007/CR-CSS21-20070719/grammar.html#scanner)

\[2007-09-27\] Change the tokenizer rule

```text
@{C}{H}{A}{R}{S}{E}{T}	{return CHARSET_SYM;}
```
to

```text
"@charset "  {return CHARSET_SYM;}
```
The @charset must be in lowercase and must have a space after it (as defined in section  [4.4 CSS style sheet representation](https://www.w3.org/TR/2007/CR-CSS21-20070719/syndata.html#charset)).

<a id="q21.376"></a>

### <a id="s.G.2a"></a>C.5.89 [Section G.2 Lexical scanner](https://www.w3.org/TR/2007/CR-CSS21-20070719/grammar.html#scanner)

\[2008-03-05\] Change the tokenizer rules

```text
"url("{w}{string}{w}")" {return URI;}
"url("{w}{url}{w}")"    {return URI;}
```
to

```text
{U}{R}{L}"("{w}{string}{w}")"	{return URI;}
{U}{R}{L}"("{w}{url}{w}")"	{return URI;}
```
<a id="q21.377"></a>

### <a id="s.G.2b"></a>C.5.90 [Section G.2 Lexical scanner](https://www.w3.org/TR/2007/CR-CSS21-20070719/grammar.html#scanner)

\[2008-04-07\] The definition of the macro “O” is wrong. The letters O and o can be written with hexadecimal escapes as “`\4f`” and “`\6f`” respectively (<em>not</em> as “`\51`” and “`\71`”). The macro should therefore be

```text
O		o|\\0{0,4}(4f|6f)(\r\n|[ \t\r\n\f])?|\\o
```
<a id="q21.378"></a>

### <a id="s.G.2c"></a>C.5.91 [Section G.2 Lexical scanner](https://www.w3.org/TR/2007/CR-CSS21-20070719/grammar.html#scanner)

“The two occurrences of "&#x5C;377"…”: There is in fact only one occurrence.

<a id="q21.379"></a>

### <a id="s.I"></a>C.5.92 [Appendix I. Index](https://www.w3.org/TR/2007/CR-CSS21-20070719/indexlist.html)

Add a TITLE attribute to all links and which is equal to the lemma.

<a id="errata2"></a>

## C.6 Errata since the Candidate Recommendation of April 2009

These are the errata for [CSS level 2 revision 1, CR version of 23 April 2009.](https://www.w3.org/TR/2009/CR-CSS2-20090423) These corrections have the status of a draft.

<a id="q21.381"></a>

### <a id="s.4.2b"></a>C.6.1 [Section 4.2 Rules for handling parsing errors](https://www.w3.org/TR/2009/CR-CSS2-20090423/syndata.html#parsing-errors)

\[2009-08-06\] Clarified the rules for ignoring invalid at-keywords:

> <strong>Invalid at-keywords.</strong> User agents must ignore an invalid at-keyword together with everything following it, <u>up to the end of the block that contains the invalid at-keyword, or</u> up to and including the next semicolon (;), <u>or up to and including</u> the next block ({...}), ~~or the end of the block (}) that contains the invalid at-keyword,~~ whichever comes first.

<a id="q21.382"></a>

### <a id="s.13.3.3c"></a>C.6.2 [Section 13.3.3 Allowed page breaks](https://www.w3.org/TR/2009/CR-CSS2-20090423/page.html#allowed-page-breaks)

\[2009-08-06\] Page breaks are also allowed when there is a gap after the last content of a block. Added the following to the first list:

> 3\. Between the content edge of a block box and the outer edges of its child content (margin edges of block-level children or line box edges for inline-level children) if there is a (non-zero) gap between them.

<a id="q21.383"></a>

### <a id="s.15.3"></a>C.6.3 [Section 15.3 Font family: the 'font-family' property](https://www.w3.org/TR/2009/CR-CSS2-20090423/fonts.html#font-family-prop)

\[2009-08-31\] The list of keywords in “(e.g., 'initial', 'inherit', 'default', 'serif', 'sans-serif', 'monospace', 'fantasy', and 'cursive')” isn't an example, but is in fact the complete and normative list.

<a id="q21.384"></a>

### <a id="s.15.3.1.1"></a>C.6.4 [Section 15.3.1.1 serif](https://www.w3.org/TR/2009/CR-CSS2-20090423/fonts.html#serif-def)

\[2009-08-31\] Spelling errors in font names. The correct names are “Excelsior Cyrillic Upright” and “ER Bukinist.”

<a id="q21.385"></a>

### <a id="s.15.7"></a>C.6.5 [Section 15.7 Font size: the 'font-size' property](https://www.w3.org/TR/2009/CR-CSS2-20090423/fonts.html#font-size-props)

\[2009-08-31\] The two notes “Note: implementation experience has demonstrated…” and “Note 2. In CSS1, the suggested scaling factor… say essentially the same thing. They are replaced by a single note:

> <strong>Note 2.</strong> In CSS1, the suggested scaling factor between adjacent indexes was 1.5, which user experience proved to be too large. In CSS2, the suggested scaling factor for a computer screen between adjacent indexes was 1.2, which still created issues for the small sizes. Implementation experience has demonstrated that a fixed ratio between adjacent absolute-size keywords is problematic, and this specification does <em>not</em> recommend such a fixed ratio.

<a id="q21.386"></a>

### <a id="s.17.5.2.1"></a>C.6.6 [Section 17.5.2.1 Fixed table layout](https://www.w3.org/TR/2009/CR-CSS2-20090423/tables.html#fixed-table-layout)

\[2009-05-20\] UAs <em>may</em> render extra columns if there are unexpected columns in later rows of a 'fixed' table layout. In that case, the width of the columns and of the table is undefined.

<a id="q21.387"></a>

### <a id="s.17.5.3"></a>C.6.7 [Section 17.5.3 Table height layout](https://www.w3.org/TR/2009/CR-CSS2-20090423/tables.html#height-layout)

\[2009-08-06\] Replaced “~~Percentage heights on table cells, table rows, and table row groups compute to 'auto'~~ by

> <u>CSS 2.1 does not define how the height of table cells and table rows is calculated when their height is specified using percentage values. CSS 2.1 does not define the meaning of ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) on row groups.</u>

<a id="q21.388"></a>

### <a id="s.Ga"></a>C.6.8 [Appendix G. Grammar of CSS 2.1](https://www.w3.org/TR/2009/CR-CSS2-20090423/grammar.html)

\[2009-08-06\] Removed ambiguities from the grammar. (The ambiguities only affected spaces and were harmless.)

<a id="errata3"></a>

## C.7 Errata since the Candidate Recommendation of September 2009

These are the errata for [CSS level 2 revision 1, CR version of 8 September 2009.](https://www.w3.org/TR/2009/CR-CSS2-20090908) These corrections have the status of a draft.

<a id="q21.390"></a>

### <a id="t.1"></a>C.7.1 [Section 1.4.2.1 Value](https://www.w3.org/TR/2009/CR-CSS2-20090908/about.html#value-defs)

\[2010-08-06\] (Also in various other sections throughout the specification.) Distinguished all cases where the word <em>value</em> referred to a whole property value from where it referred to only part of such a value (such as a component in a comma-separated list). The former is now <em>property value,</em> the latter <em>component value.</em>

<a id="q21.391"></a>

### <a id="t.3.1"></a>C.7.2 [Section 3.1 Definitions](https://www.w3.org/TR/2009/CR-CSS2-20090908/conform.html#defs)

\[2010-04-19\] Add a clarification to the definition of replaced element:

> <u>The content of replaced elements is not considered in the CSS rendering model.</u>

(Previously, the definition only said that the content was “outside the scope of CSS.”)

<a id="q21.392"></a>

### <a id="t.4.1.1"></a>C.7.3 [Section 4.1.1 Tokenization](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#tokenization)

\[2010-04-19\] The definition of “identifier” in 4.1.3 (2nd bullet) and in the grammar were contradictory w.r.t. whether no-break space (U+00A0) was allowed in identifiers or not. Change the text in 4.1.3 to allow no-break space: “characters ~~U+00A1~~ <u>U+00A0</u> and higher.”

Also, change the macro “nonascii” in the token definition from “~~\[^&#x5C;0-&#x5C;177\]~~” to <u>\[^&#x5C;0-&#x5C;237\]</u>”. (When CSS was first written, Unicode didn't have code points U+0080 to U+009F, i.e., &#x5C;200-&#x5C;237 in octal.)

<a id="q21.393"></a>

### <a id="t.4.1.1a"></a>C.7.4 [Section 4.1.1 Tokenization](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#tokenization)

\[2010-09-29\] The tokenizer has been modified so that it can be implemented as a state machine without back-up (e.g., with Lex). This changes the meaning of an input of the form “url(…(…)…)”, i.e., input that starts like a URI token but then contains a parenthesis (which is not allowed in a URI token). Previously, such input was re-parsed to yield a FUNCTION token followed by other things; now it yields a BAD_URI token. Given that CSS has never used a FUNCTION token of the form “url(” this should not affect any existing CSS style sheets.

A non-normative section has been added to appendix G with an explanation of how to make a tokenizer without back-up.

<a id="q21.394"></a>

### <a id="t.4.1.1b"></a>C.7.5 [Section 4.1.1 Tokenization](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#tokenization)

\[2010-09-29\] The definition of the URI token was ambiguous: it allowed a backslash to be either parsed on its own or as part of an escape. A backslash in a URI token must always be interpreted as part of an escape.

<a id="q21.395"></a>

### <a id="t.4.1.1c"></a>C.7.6 [Section 4.1.1 Tokenization](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#tokenization)

\[2010-09-29\] Error handling for illegal tokens (braces, at-keywords, and SGML comment tokens) inside parenthesized expressions was not well defined. Change the production for “any” as follows

> ```text
> 
> any         : [ IDENT | NUMBER | PERCENTAGE | DIMENSION | STRING
>               | DELIM | URI | HASH | UNICODE-RANGE | INCLUDES
>               | DASHMATCH | ':' | FUNCTION S* [any|unsused]* ')'
>               | '(' S* [any|unused]* ')' | '[' S* [any|unused]* ']'
>               ] S*;
> unused      : block | ATKEYWORD S* | ';' S* | CDO S* | CDC S*;
> ```
and add the following explanation:

> <u>The "unused" production is not used in CSS and will not be used by any future extension. It is included here only to help with error handling. (See 4.2 "Rules for handling parsing errors.")</u>

<a id="q21.396"></a>

### <a id="t.4.1.2.2"></a>C.7.7 [Section 4.1.2.2 Informative Historical Notes](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#vendor-keyword-history)

\[2010-04-19\] Add “-tc-” to the list of existing vendor prefixes.

<a id="q21.397"></a>

### <a id="t.4.1.3"></a>C.7.8 [Section 4.1.3 Characters and case](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#characters)

\[2010-08-06\] The handling of a backslash before a newline or at the end of a file is no longer undefined: it is parsed as a DELIM.

<a id="q21.398"></a>

### <a id="t.4.1.3a"></a>C.7.9 [Section 4.1.3 Characters and case](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#characters)

\[2010-08-06\] Make text and formal grammar the same:

> In CSS, identifiers \[…\]; they cannot start with a digit, <u>two hyphens,</u> or a hyphen followed by a digit.

<a id="q21.399"></a>

### <a id="t.4.1.8"></a>C.7.10 [Section 4.1.8 Declarations and properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#declaration)

\[2010-05-12\] Remove “2.1” from

> Every CSS ~~2.1~~ property has its own syntactic and semantic restrictions

<a id="q21.400"></a>

### <a id="t.4.2"></a>C.7.11 [Section 4.2 Rules for handling parsing errors](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#parsing-errors)

\[2010-07-07\] Clarify that the fifth bullet only applies to at-rules. (At-keywords in other constructs are already handled in the preceding bullets.)

> - <strong><del>Invalid at-keywords</del> <ins>At-rules with
unknown at-keywords</ins>.</strong> User agents must ignore…

<a id="q21.401"></a>

### <a id="t.4.3.2"></a>C.7.12 [Section 4.3.2 Lengths](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#length-units)

\[2010-04-19\] Make explicit that 'ex', when used in the 'font-size' property, refers to the <em>parent
element's</em> 'ex' (just as 'em' refers to the parent's 'em' in that case.)

<a id="q21.402"></a>

### <a id="t.4.3.2a"></a>C.7.13 [Section 4.3.2 Lengths](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#length-units)

\[2010-10-28\] A UA must now <em>either</em> display absolute lengths (cm, in, pt, etc.) at their real size <em>or</em> make px align with device pixel boundaries near the 0.0213 degrees viewing angle, but not both. In either case, 3px must equal 4pt.

(Until now, authors could use absolute lengths for physical sizes and px for aligning to device pixels, but couldn't know the number of pt in a px, except in combination with Media Queries. Authors can no longer choose between absolute or device-related units, but can use px and pt interchangeably. This should only affect relatively low-resolution devices: above 300 dots per inch, the maximum error is about 16%.)

<a id="q21.403"></a>

### <a id="t.4.3.4a"></a>C.7.14 [Section 4.3.4 URLs and URIs](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#uri)

\[2010-05-12\] Commas do not have to be escaped in \<uri\> tokens:

> Some characters appearing in an unquoted URI, such as parentheses, ~~commas,~~ white space characters, single quotes (') and double quotes ("), must be escaped

<a id="q21.404"></a>

### <a id="t.4.3.4"></a>C.7.15 [Section 4.3.4 URLs and URIs](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#uri)

\[2010-04-21\] Describe in English what was only expressed through the grammar:

> <u>Note. Since URIs may contain characters that would otherwise be used as delimiters in CSS, the entire URI value must be treated as a single unit by the tokenizer and normal tokenization behavior does not apply within a URI value. Therefore comments are not allowed within a URI value.</u>

<a id="q21.405"></a>

### <a id="t.5.8.2"></a>C.7.16 [Section 5.8.2 Default attribute values in DTDs](https://www.w3.org/TR/2009/CR-CSS2-20090908/selector.html#default-attrs)

\[2010-09-29\] Clarify what is meant by “is not required”:

> More precisely, a UA <u>may, but</u> is <em>not</em> required to<u>,</u> read an "external subset" of the DTD but <em>is</em> required to look for default attribute values in the document's "internal subset." (See \[XML10\] for definitions of these subsets.) <u>Depending on the UA, a default attribute value defined in the external subset of the DTD might or might not appear in the document tree.</u>
>
> A UA that recognizes an XML namespace \[XMLNAMESPACES\] <u>may, but</u> is not required to<u>,</u> use its knowledge of that namespace to treat default attribute values as if they were present in the document. (E.g., an XHTML UA is not required to use its built-in knowledge of the XHTML DTD.)

and:

> the first rule ~~will~~ <u>might</u> not match elements whose "notation" attribute is set by default, i.e., not set explicitly. To catch all cases, the attribute selector for the default value must be dropped:

<a id="q21.406"></a>

### <a id="t.5.11.4"></a>C.7.17 [Section 5.11.4 The language pseudo-class: :lang](https://www.w3.org/TR/2009/CR-CSS2-20090908/selector.html#lang)

\[2010-08-06\] The argument of ':lang()' is only case-insensitive for characters in ASCII.

<a id="q21.407"></a>

### <a id="t.5.12"></a>C.7.18 [Section 5.12 Pseudo-elements](https://www.w3.org/TR/2009/CR-CSS2-20090908/selector.html#pseudo-element-selectors)

\[2010-08-06\] Clarify that pseudo-elements behave like elements for the aspects not explicitly mentioned:

> <u>Pseudo-elements behave just like real elements in CSS with the exceptions described below and [elsewhere.](https://www.w3.org/TR/2009/CR-CSS2-20090908/generate.html#before-after-content)</u>

<a id="q21.408"></a>

### <a id="t.5.12.1"></a>C.7.19 [Section 5.12.1 The :first-line pseudo-element](https://www.w3.org/TR/2009/CR-CSS2-20090908/selector.html#first-line-pseudo)

\[2010-08-24\] More consistent use of “block” and “block-level.” Change:

> The :first-line pseudo-element can only be attached to a ~~block-level element, inline-block, table-caption or a table-cell~~ <u>block container element</u>.

<a id="q21.409"></a>

### <a id="t.5.12.2"></a>C.7.20 [Section 5.12.2 The :first-letter pseudo-element](https://www.w3.org/TR/2009/CR-CSS2-20090908/selector.html#first-letter)

\[2010-08-24\] More consistent use of “block” and “block-level.” Change:

> The :first-letter pseudo-element applies to ~~block, list-item, table-cell, table-caption and inline-block elements~~ <u>block container elements</u>.

<a id="q21.410"></a>

### <a id="t.6.2"></a>C.7.21 [Section 6.2 Inheritance](https://www.w3.org/TR/2009/CR-CSS2-20090908/cascade.html#inheritance)

\[2010-08-06\] Add a note that, because it follows the document tree, inheritance is not intercepted by anonymous boxes

<a id="q21.411"></a>

### <a id="t.6.4.4"></a>C.7.22 [Section 6.4.4 Precedence of non-CSS presentational hints](https://www.w3.org/TR/2009/CR-CSS2-20090908/cascade.html#preshint)

\[2010-10-05\] Give other languages than HTML (such as SVG) the possibility to define certain attributes as “presentational attributes”:

> For other languages, all document language-based styling ~~should be handled in the user agent style sheet~~ <u>must be translated to the corresponding CSS and either enter the cascade at the user agent level or, as with HTML presentational hints, be treated as author level rules with a specificity of zero placed at the start of the author style sheet</u>.

<a id="q21.412"></a>

### <a id="t.7.3"></a>C.7.23 [Section 7.3 Recognized media types](https://www.w3.org/TR/2009/CR-CSS2-20090908/media.html#media-types)

\[2010-09-08\] Clarify what is ignored. Change:

> @media and @import rules with unknown media types <u>(that are nonetheless valid identifiers)</u> are treated as if the unknown media types are not present. <u>If an @media/@import rule contains a malformed media type (not an identifier) then the statement is invalid.</u>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > <u><em><strong>Note:</strong> Media Queries supercedes
this error handling.</em></u>

<a id="q21.413"></a>

### <a id="t.8.3.1"></a>C.7.24 [Section 8.3.1 Collapsing margins](https://www.w3.org/TR/2009/CR-CSS2-20090908/box.html#collapsing-margins)

\[2010-05-12\] Simplify/clarify text:

> An element that has ~~had~~ clearance ~~applied to it~~ never collapses

and:

> When an element's own margins collapse, and that element has ~~had~~ clearance ~~applied to it~~

<a id="q21.414"></a>

### <a id="t.8.3.1a"></a>C.7.25 [Section 8.3.1 Collapsing margins](https://www.w3.org/TR/2009/CR-CSS2-20090908/box.html#collapsing-margins)

\[2010-08-24\] More consistent use of “block box” vs “block-level element.” Include table captions in the set of block-level elements. See also [changes to 9.2.1](#t.9.2.1) and to [9.2.1.1.](#t.9.2.1.1a)

> Two or more adjoining vertical margins of block<u>-level</u> boxes in the normal flow collapse.

and

> The top margin of an in-flow ~~block-level element~~ <u>block box</u> is adjoining to its first in-flow block-level child's top margin

and

> The bottom margin of an in-flow ~~block-level element~~ <u>block box</u> with a 'height' of 'auto'

<a id="q21.415"></a>

### <a id="t.9.2.1"></a>C.7.26 [Section 9.2.1 Block-level elements and block boxes](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#anonymous-block-level)

\[2010-08-24\] Define the term “block-level element” more precisely. Also define auxiliary terms “block container box” and “block box”:

More consistent use of block box vs block-level element in section 9.2.1.1. See also [changes to section 8.3.1](#t.8.3.1a) and [9.4.](#t.9.4)

<a id="q21.416"></a>

### <a id="t.9.2.1.1"></a>C.7.27 [Section 9.2.1.1 Anonymous block boxes](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#anonymous-block-level)

\[2010-05-12\] The example has invalid HTML mark-up. Change it to use P and SPAN elements instead of BODY and P.

\[2010-08-06\] Also clarify that “block box” only refers to boxes in the same flow.

<a id="q21.417"></a>

### <a id="t.9.2.1.1a"></a>C.7.28 [Section 9.2.1.1 Anonymous block boxes](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#anonymous-block-level)

\[2010-09-29\] Percentage values that refer to dimensions of parent boxes ignore any intervening anonymous boxes. Add this paragraph:

> <u>Anonymous block boxes are ignored when resolving percentage values that would refer to it: the closest non-anonymous ancestor box is used instead. For example, if the child of the anonymous block box inside the DIV above needs to know the height of its containing block to resolve a percentage height, then it will use the height of the containing block formed by the DIV, not of the anonymous block box.</u>

<a id="q21.418"></a>

### <a id="t.9.2.1.1b"></a>C.7.29 [Section 9.2.1.1 Anonymous block boxes](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#anonymous-block-level)

\[2010-09-29\] Clarify the wording:

> When an inline box contains an in-flow block box \[…\] When such an inline box is affected by relative positioning, the relative positioning also affects the block-level box <u>contained in the block box</u>.

<a id="q21.419"></a>

### <a id="t.9.2.1.1c"></a>C.7.30 [Section 9.2.1.1 Anonymous block boxes](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#anonymous-block-level)

\[2010-10-13\] Clarify that an inline box that is broken around a block-level box is always broken into <em>two</em> pieces, even if one or both are empty:

> When an inline box contains an in-flow block-level box, the inline box (and its inline ancestors within the same line box) are broken around the block-level box<u>, dividing the inline box into two pieces, even if either side is empty.</u>.

<a id="q21.420"></a>

### <a id="t.9.2.2"></a>C.7.31 [Section 9.2.2 Inline-level elements and inline boxes](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#anonymous-block-level)

\[2010-08-24\] Better define the term “inline-level element/box” and define the auxiliary terms “inline box” and “atomic inline-level box.”

<a id="q21.421"></a>

### <a id="t.9.2.3a"></a>C.7.32 [Section 9.2.3 Run-in boxes](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#run-in)

\[2010-04-19\] Make the definition of 'run-in' more precise:

> ~~A run-in box behaves as follows:~~
>
> 1.  ~~If the run-in box contains a block box, the run-in box becomes a block box.~~
> 2.  ~~If a sibling block box (that does not float and is not absolutely positioned) follows the run-in box, the run-in box becomes the first inline box of the block box. A run-in cannot run in to a block that already starts with a run-in or that itself is a run-in.~~
> 3.  ~~Otherwise, the run-in box becomes a block box.~~
>
> <u>A run-in element (or pseudo-element) <var>A</var> behaves as follows:</u>
>
> 1.  <u>If <var>A</var> has any children that inhibit run-in behavior (see below), then <var>A</var> is rendered as if it had 'display: block'.</u>
> 2.  <u>Let <var>B</var> be the first of <var>A</var>'s following siblings that is neither floating nor absolutely positioned nor has 'display: none'. If <var>B</var> exists and has a specified value for 'display' of 'block' or 'list-item' and is not replaced, then <var>A</var> is rendered as an 'inline' element at the start of <var>B</var>'s principal box. Note: <var>A</var> is rendered before <var>B</var>'s ':before' pseudo-element, if any. See 12.1.</u>
> 3.  <u>Otherwise, <var>A</var> is rendered as if it had 'display: block'.</u>
>
> <u>In the above, "siblings" and "children" include both normal elements and :before/:after pseudo-elements.</u>
>
> <u>An element or pseudo-element <var>C</var> inhibits run-in behavior if one of the following is true. (Note that the definition is recursive.)</u>
>
> 1.  <u><var>C</var> is not floating and not absolutely positioned and the computed value of its 'display' is one of 'block', 'list-item', 'table' or 'run-in'.</u>
> 2.  <u><var>C</var> has a computed value for 'display' of 'inline' and it has one or more children that inhibit run-in behavior. (Where "children" includes both normal elements and :before/:after pseudo-elements.)</u>

It remains undefined how 'run-in' and ':first-line' interact:

> <u>It is undefined in CSS 2.1 if a run-in inherits from a ':first-line' pseudo-element.</u>

<a id="q21.422"></a>

### <a id="t.9.2.4"></a>C.7.33 [Section 9.2.4 The 'display' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#display-prop)

\[2010-08-06\] Use the same terminology as in chapter 12:

> list-item  
> This value causes an element (e.g., LI in HTML) to generate a principal block box and a ~~list-item inline~~ <u>marker</u> box.

<a id="q21.423"></a>

### <a id="t.9.2.4a"></a>C.7.34 [Section 9.2.4 The 'display' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#display-prop)

\[2010-08-24\] More consistent use of “inline-level.”

> inline-block  
> This value causes an element to generate ~~a block box, which itself is flowed as a single inline box, similar to a replaced element~~ <u>an inline-level block container</u>. The inside of an inline-block is formatted as a block box, and the element itself is formatted as an ~~inline replaced element~~ <u>an atomic inline-level box</u>.

<a id="q21.424"></a>

### <a id="t.9.3"></a>C.7.35 [Section 9.3 Positioning schemes](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#positioning-scheme)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.”

> 1.  Normal flow. In CSS 2.1, normal flow includes block formatting of block<u>-level</u> boxes, inline formatting of inline<u>-level</u> boxes, relative positioning of block<u>-level</u> ~~or~~ <u>and</u> inline<u>-level</u> boxes, and ~~positioning~~ <u>formatting</u> of run-in boxes.

<a id="q21.425"></a>

### <a id="t.9.4"></a>C.7.36 [Section 9.4 Normal flow](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#normal-flow)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.”

> Boxes in the normal flow belong to a formatting context, which may be block or inline, but not both simultaneously. Block<u>-level</u> boxes participate in a block formatting context. Inline<u>-level</u> boxes participate in an inline formatting context.

In 9.4.1:

> Floats, absolutely positioned elements, ~~inline-blocks, table-cells, table-captions, and elements with 'overflow' other than 'visible' (except when that value has been propagated to the viewport) establish new block formatting contexts~~ <u>block containers (such as inline-blocks, table-cells, and table-captions) that are not block boxes, and block boxes with 'overflow' other than 'visible'</u>.
>
> In a block formatting context, boxes are laid out one after the other, vertically, beginning at the top of a containing block. The vertical distance between two sibling boxes is determined by the 'margin' properties. Vertical margins between adjacent block<u>-level</u> boxes in a block formatting context collapse.

In 9.4.2:

> \[…\] When several inline<u>-level</u> boxes cannot fit horizontally within a single line box, they are distributed among two or more vertically-stacked line boxes.
>
> When the total width of the inline<u>-level</u> boxes on a line \[…\]is less than the width of the line box containing them, their horizontal distribution within the line box is determined by the 'text-align' property. If that property has the value 'justify', the user agent may stretch spaces and words in inline boxes (~~except for~~ <u>but not</u> inline-table and inline-block boxes) as well.

<a id="q21.426"></a>

### <a id="t.9.3.2"></a>C.7.37 [Section 9.3.2 Box offsets: 'top', 'right', 'bottom', 'left'](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#position-props)

\[2010-07-19\] If 'top', 'right', 'bottom' or 'left' is specified as 'auto', the <em>used</em> value rather than the <em>computed</em> value is set to the negative of the opposite side. For all four, change:

> |                     |                                                                                                                                                                                                                                             |
> |---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
> | <em>Computed value:</em> | ~~for 'position:relative', see section Relative Positioning.~~ For 'position:static', 'auto'. Otherwise: if specified as a length, the corresponding absolute length; if specified as a percentage, the specified value; otherwise, 'auto'. |

And in section 9.4.3:

> \[…\] Since boxes are not split or stretched as a result of 'left' or 'right', the ~~computed~~ <u>used</u> values are always: left = -right.
>
> If both 'left' and 'right' are 'auto' (their initial values), the ~~computed~~ <u>used</u> values are '0' (i.e., the boxes stay in their original position).
>
> If 'left' is 'auto', its ~~computed~~ <u>used</u> value is minus the value of 'right' (i.e., the boxes move to the left by the value of 'right').
>
> If 'right' is specified as 'auto', its ~~computed~~ <u>used</u> value is minus the value of 'left'.
>
> \[…\] Since boxes are not split or stretched as a result of 'top' or 'bottom', the ~~computed~~ <u>used</u> values are always: top = -bottom. If both are 'auto', their ~~computed~~ <u>used</u> values are both '0'. If one of them is 'auto', it becomes the negative of the other. If neither is 'auto', 'bottom' is ignored (i.e., the ~~computed~~ <u>used</u> value of 'bottom' will be minus the value of 'top').

<a id="q21.427"></a>

### <a id="t.9.5"></a>C.7.38 [Section 9.5 Floats](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#floats)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.”

> \[…\] In other words, if inline<u>-level</u> boxes are placed on the line before a left float is encountered that fits in the remaining line box space, the left float is placed on that line, aligned with the top of the line box, and then the inline<u>-level</u> boxes already on the line are moved accordingly to the right of the float (the right being the other side of the left float) and vice versa for rtl and right floats.

In 9.5.2:

> Values have the following meanings when applied to non-floating block<u>-level</u> boxes:

<a id="q21.428"></a>

### <a id="t.9.5a"></a>C.7.39 [Section 9.5 Floats](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#floats)

\[2010-10-25\] Define exactly what it means for a line box to be <em>next to</em> a float:

> \[…\] However, line boxes created next to the float are shortened to make room for the margin box of the float.
>
> <u>A line box is next to a float when there exists a vertical position that satisfies all of these four conditions: (a) at or below the top of the line box, (b) at or above the bottom of the line box, (c) below the top margin edge of the float, and (d) above the bottom margin edge of the float.</u>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > <u>Note: this means that floats with zero height or negative height do not move line boxes.</u>

<a id="q21.429"></a>

### <a id="t.9.5.2"></a>C.7.40 [Section 9.5.2 Controlling flow next to floats: the 'clear' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#flow-control)

\[2010-05-12\] Clarify that 'clear' only introduces clearance above an element if necessary; and that clearance may have zero height.

<a id="q21.430"></a>

### <a id="t.9.5.2a"></a>C.7.41 [Section 9.5.2 Controlling flow next to floats: the 'clear' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#flow-control)

\[2010-10-13\] Added an example of calculating clearance from two collapsing margins M1 and M2 and the height H of a float.

<a id="q21.431"></a>

### <a id="t.9.5.2b"></a>C.7.42 [Section 9.5.2 Controlling flow next to floats: the 'clear' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#flow-control)

\[2010-10-13\] Clarify the language:

> Computing the clearance of an element on which 'clear' is set is done by first determining the hypothetical position of the element's top border edge within its parent block. ~~This position is determined after the top margin of the element has been collapsed with previous adjacent margins (including the top margin of the parent block).~~ <u>This position where the actual top border edge would have been if the element had a non-zero top border and its 'clear' property had been 'none'.</u>
>
> If this hypothetical position of the element's top border edge is not past the relevant floats, then clearance ~~must be~~ <u>is introduced, and margins collapse according to the rules in 8.3.1.</u>
>
> <u>Then the amount of clearance is</u> set to the greater of:
>
> 1.  The amount necessary to place the border edge of the block even with the bottom outer edge of the lowest float that is to be cleared.
>
> 2.  ~~The amount necessary to make the sum of the following equal to the distance to which these margins collapsed when the hypothetical position was calculated:~~
>
>     - ~~the margins collapsing above the clearance~~
>     - ~~the clearance itself~~
>     - ~~if the block's own margins collapse together: the block's top margin~~
>     - ~~if the block's own margins do not collapse together: the margins collapsing below the clearance~~
>
>     <u>The amount necessary to place the top border edge of the block at its hypothetical position.</u>

<a id="q21.432"></a>

### <a id="t.9.5.2c"></a>C.7.43 [Section 9.5.2 Controlling flow next to floats: the 'clear' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#flow-control)

\[2010-10-13\] Correction: The hypothetical position is determined by assuming the box has a non-zero <em>bottom</em> border (see section 8.3.1):

> This position is where the actual top border edge would have been if the element had a non-zero ~~top~~ <u>bottom</u> border and its 'clear' property had been 'none'.

<a id="q21.433"></a>

### <a id="t.14.2.1"></a>C.7.44 [Section 14.2.1 Background properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/colors.html#background-properties)

\[2010-04-19\] 'Fixed' backgrounds in paged media are positioned relative to the page box (and thus repeat on every page, just like 'fixed' elements). The position of fixed backgrounds in paged media was previously undefined.

<a id="q21.434"></a>

### <a id="t.9.9.1"></a>C.7.45 [Section 9.9.1 Specifying the stack level: the 'z-index' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#z-index)

\[2010-07-07\] Some ambiguities in the description of stacking contexts are fixed and the description is clearly marked as non-normative. (Appendix E holds the normative description.)

<a id="q21.435"></a>

### <a id="t.9.10"></a>C.7.46 [Section 9.10 Text direction: the 'direction' and 'unicode-bidi' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#direction)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.”

> User agents that support bidirectional text must apply the Unicode bidirectional algorithm to every sequence of inline<u>-level</u> boxes uninterrupted by a forced line break or block boundary. This sequence forms the "paragraph" unit in the bidirectional algorithm. The paragraph embedding level is set according to the value of the 'direction' property of the containing block rather than by the heuristic given in steps P2 and P3 of the Unicode algorithm.
>
> \[…\]
>
> For the 'direction' property to affect reordering in inline~~-level~~ elements, the 'unicode-bidi' property's value must be 'embed' or 'override'.
>
> \[…\]
>
> <strong>normal</strong>  
> The element does not open an additional level of embedding with respect to the bidirectional algorithm. For inline~~-level~~ elements, implicit reordering works across element boundaries.
>
> <strong>embed</strong>  
> If the element is inline~~-level~~, this value opens an additional level of embedding with respect to the bidirectional algorithm. The direction of this embedding level is given by the 'direction' property. Inside the element, reordering is done implicitly. This corresponds to adding a LRE (U+202A; for 'direction: ltr') or RLE (U+202B; for 'direction: rtl') at the start of the element and a PDF (U+202C) at the end of the element.
>
> <strong>bidi-override</strong>  
> For inline~~-level~~ elements this creates an override. For ~~block-level, table-cell, table-caption, or inline-block~~ <u>block container</u> elements this creates an override for inline-level descendants not within another block container element. This means that inside the element, reordering is strictly in sequence according to the 'direction' property; the implicit part of the bidirectional algorithm is ignored. This corresponds to adding a LRO (U+202D; for 'direction: ltr') or RLO (U+202E; for 'direction: rtl') at the start of the element or at the start of each anonymous child block box, if any, and a PDF (U+202C) at the end of the element.
>
> The final order of characters in each ~~block-level element~~ <u>block container</u> is \[…\]

<a id="q21.436"></a>

### <a id="t.9.10a"></a>C.7.47 [Section 9.10 Text direction: the 'direction' and 'unicode-bidi' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#direction)

\[2010-10-05\] Add a reference to [bidi class B](http://www.unicode.org/reports/tr9/#Bidirectional_Character_Types) in Unicode TR 9 to clarify what a “forced break” is in the context of the Unicode bidi algorithm:

> \[…\] inline-level boxes uninterrupted by a forced ~~line~~ <u>([bidi class B](http://www.unicode.org/reports/tr9/#Bidirectional_Character_Types))</u> break or block boundary

<a id="q21.437"></a>

### <a id="t.9.10b"></a>C.7.48 [Section 9.10 Text direction: the 'direction' and 'unicode-bidi' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#direction)

\[2010-10-25\] clarify “non-textual entities”:

> In this process, ~~non-textual entities such as images~~ <u>replaced elements with 'display: inline' (and replaced elements with 'display: run-in', when they generate inline-level boxes)</u> are treated as neutral characters, unless their 'unicode-bidi' property has a value other than 'normal', in which case they are treated as strong characters in the 'direction' specified for the element. <u>All other atomic inline-level boxes are treated as neutral characters always.</u>

<a id="q21.438"></a>

### <a id="t.10.1"></a>C.7.49 [Section 10.1 Definition of "containing block"](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#containing-block-details)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.”

> 1.  \[…\]
> 2.  For other elements, if the element's position is 'relative' or 'static', the containing block is formed by the content edge of the nearest ~~block-level, table cell or inline-block~~ <u>block container</u> ancestor box.
> 3.  \[…\]
> 4.  \[…\]
>     1.  In the case that the ancestor is ~~inline-level~~ <u>an inline box</u>, the containing block depends on the 'direction' property of the ancestor:

<a id="q21.439"></a>

### <a id="t.10.2"></a>C.7.50 [Section 10.2 Content width: the 'width' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#the-width-property)

\[2010-05-12\] The computed value of 'width' doesn't depend on whether the property applies or not:

> Computed value: the percentage or 'auto' as specified or the absolute length; ~~'auto' if the property does not apply~~

<a id="q21.440"></a>

### <a id="t.10.2a"></a>C.7.51 [Section 10.2 Content width: the 'width' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#the-width-property)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.”

> This property specifies the content width of boxes ~~generated by block-level and replaced elements~~.
>
> This property does not apply to non-replaced inline~~-level~~ elements.

<a id="q21.441"></a>

### <a id="t.10.2b"></a>C.7.52 [Section 10.2 Content width: the 'width' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#the-width-property)

\[2010-10-05\] Remove unclear and redundant sentence:

> ~~The width of a replaced element's box is intrinsic and may be scaled by the user agent if the value of this property is different than 'auto'.~~

<a id="q21.442"></a>

### <a id="t.10.5"></a>C.7.53 [Section 10.5 Content height: the 'height' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#the-height-property)

\[2010-05-12\] The computed value of 'height' doesn't depend on whether the property applies or not:

> Computed value: the percentage or 'auto' (see prose under \<percentage\>) or the absolute length; ~~'auto' if the property does not apply~~

<a id="q21.443"></a>

### <a id="t.10.5a"></a>C.7.54 [Section 10.5 Content height: the 'height' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#the-height-property)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.”

> This property specifies the content height of boxes ~~generated by block-level, inline-block and replaced elements~~.
>
> This property does not apply to non-replaced inline~~-level~~ elements. See the section on computing heights and margins for non-replaced inline elements for the rules used instead.

<a id="q21.444"></a>

### <a id="t.10.6.7"></a>C.7.55 [Section 10.6.7 'Auto' heights for block formatting context roots](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#root-height)

\[2010-08-06\] Clarify “bottom” and “preceding”:

> In certain cases (see ~~the preceding sections~~ <u>e.g., sections 10.6.4 and 10.6.6</u>), the height of an element that establishes a block formatting context is computed as follows:
>
> \[…\]
>
> In addition, if the element has any floating descendants whose bottom margin edge is below the ~~bottom~~ <u>the element's bottom content edge</u>, then the height is increased to include those edges. Only floats that are children of the element itself or of descendants in the normal flow are taken into account, e.g., floats inside absolutely positioned descendants or other floats are not.

<a id="q21.445"></a>

### <a id="t.10.7"></a>C.7.56 [Section 10.7 Minimum and maximum heights: 'min-height' and 'max-height'](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#propdef-min-height)

\[2010-10-26\] The effect of 'min-height' and 'max-height' on table cells is still undefined in CSS:

> <u>In CSS 2.1, the effect of 'min-height' and 'max-height' on tables, inline tables, table cells, table rows, and row groups is undefined.</u>

<a id="q21.446"></a>

### <a id="t.10.8"></a>C.7.57 [Section 10.8 Line height calculations: the 'line-height' and 'vertical-align' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#line-height)

\[2010-06-02\] Clarifications to the calculation of the line boxes and the minimum line height ("strut"). Item 2 in the bulleted list is expanded and items 3 and 4 are merged, as follows:

> 1.  The height of each inline box in the line box is calculated (see "Calculating heights and margins" and the 'line-height' property).
> 2.  The inline boxes are aligned vertically according to their 'vertical-align' property. <u>In case they are aligned 'top' or 'bottom', they must be aligned so as to minimize the line box height. If such boxes are tall enough, there are multiple solutions and CSS 2.1 does not define the position of the line box's baseline (i.e., the position of the strut, see below).</u>
> 3.  The line box height is the distance between the uppermost box top and the lowermost box bottom. <u>(This includes the strut, as explained under 'line-height' below.)</u>
> 4.  ~~If the resulting height is smaller than the minimal height of line boxes for this block, as specified by the 'line-height' property, the height is increased to be that minimal height.~~

Furthermore, in 10.8.1, after the definition of “strut,” clarify that the font determines the initial baseline:

> <u>The height and depth of the font above and below the baseline are assumed to be metrics that are contained in the font. (For more details, see CSS level 3.)</u>

<a id="q21.447"></a>

### <a id="t.10.8a"></a>C.7.58 [Section 10.8 Line height calculations: the 'line-height' and 'vertical-align' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#line-height)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.”

> As described in the section on inline formatting contexts, user agents flow inline<u>-level</u> boxes into a vertical stack of line boxes. The height of a line box is determined as follows:
>
> 1.  The height of each inline<u>-level</u> box in the line box is calculated (see "Calculating heights and margins" and the 'line-height' property).
> 2.  The inline<u>-level</u> boxes are aligned vertically according to their 'vertical-align' property.

In 10.8.1:

> On a ~~block-level, table-cell, table-caption or inline-block~~ <u>block container</u> element whose content is composed of inline-level elements, 'line-height' specifies the <em>minimal</em> height of line boxes within the element. \[…\]
>
> On an inline~~-level~~ element, 'line-height' specifies the height that is used in the calculation of the line box height \[…\]

After the definition of 'vertical-align':

> The following values only have meaning with respect to a parent inline~~-level~~ element, or to the strut of a parent ~~block-level, table-cell, table-caption or inline-block~~ <u>block container</u> element.

<a id="q21.448"></a>

### <a id="t.10.8.1"></a>C.7.59 [Section 10.8.1 Leading and half-leading](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#leading)

\[2010-07-19\] Clarify text:

> On a block-level, table-cell, table-caption or inline-block element whose content is composed of inline-level elements, 'line-height' specifies the minimal height of line boxes within the element. The minimum height consists of a minimum height above the ~~block's~~ baseline and a minimum depth below it, exactly as if each line box starts with a zero-width inline box with the ~~block's~~ <u>element's</u> font and line height properties. ~~(what TEX calls a "strut").~~ <u>We call that imaginary box a "strut." (The name is inspired by TeX.).</u>

<a id="q21.449"></a>

### <a id="t.10.8.1a"></a>C.7.60 [Section 10.8.1 Leading and half-leading](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#leading)

\[2010-08-20\] Remove text that talks about the “content area” of an inline box and about “center vertically” and instead make it more explicit how leading is added to a glyph: leading is added above and below a hypothetical box around each glyph that represents the (normal or ideal) height of a line of text in that font, as given in the font metrics.

Add a note referring to 10.6.1 (which defines that the content area is undefined) and explaining that the exact position of backgrounds and borders relative to the line box is undefined.

Also add a note about how to find the relevant metrics in OpenType and TrueType fonts.

<a id="q21.450"></a>

### <a id="t.10.8.1b"></a>C.7.61 [Section 10.8.1 Leading and half-leading](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#leading)

\[2010-08-20\] Clarify some imprecise terms:

> When an element contains text that is rendered in more than one font, user agents may determine the <u>'normal'</u> 'line-height' value according to the largest font size.
>
> Generally, when there is only one value of 'line-height' for all inline boxes in a ~~paragraph~~ <u>block container box</u> (and no ~~tall images~~ <u>replaced elements, inline-block elements, etc.</u>), the above will ensure that baselines of successive lines are exactly 'line-height' apart. This is important when columns of text in different fonts have to be aligned, for example in a table.

<a id="q21.451"></a>

### <a id="t.11.1"></a>C.7.62 [Section 11.1 Overflow and clipping](https://www.w3.org/TR/2009/CR-CSS2-20090908/visufx.html#overflow-clipping)

\[2010-10-25\] Clarify which ancestors are meant:

> - A descendant box is [positioned absolutely](css2--visuren.html--3f334c530cf4.md#absolute-positioning), partly outside the box. Such boxes are not always clipped by the overflow property on their ancestors; <u>specifically, they are not clipped by the overflow of any ancestor between themselves and their containing block</u>

<a id="q21.452"></a>

### <a id="t.11.1.1"></a>C.7.63 [Section 11.1.1 Overflow: the 'overflow' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visufx.html#overflow)

\[2010-08-06\] The phrase “containing block” in the example doesn't refer to the technical term [“containing block”](https://www.w3.org/TR/2009/CR-CSS2-20090908/visudet.html#containing-block-details) but simply to the containing box. Change “containing ~~block~~” to “containing <u>div</u>.”

<a id="q21.453"></a>

### <a id="t.11.1.1a"></a>C.7.64 [Section 11.1.1 Overflow: the 'overflow' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visufx.html#overflow)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.”

> This property specifies whether content of a ~~block-level~~ <u>block container</u> element is clipped when it overflows the element's box.

<a id="q21.454"></a>

### <a id="t.11.1.1b"></a>C.7.65 [Section 11.1.1 Overflow: the 'overflow' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visufx.html#overflow)

\[2010-10-25\] Add missing inline-table:

> |                     |                                                                                                |
> |---------------------|------------------------------------------------------------------------------------------------|
> | <em>Applies to:</em> | non-replaced block-level elements, table cells, <u>inline-table,</u> and inline-block elements |

<a id="q21.455"></a>

### <a id="t.11.1.2"></a>C.7.66 [Section 11.1.2 Clipping: the 'clip' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visufx.html#propdef-clip)

\[2010-10-25\] The computed value of 'auto' is 'auto' also when 'auto' is specified inside 'rect()':

> |                     |                                                                                                                                                                                                                                                                   |
> |---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
> | <em>Computed value:</em> | ~~For rectangle values, a rectangle consisting of four computed lengths; otherwise, as specified~~ <u>'auto' if specified as 'auto', otherwise a rectangle with four values, each of which is 'auto' if specified as 'auto' and the computed length otherwise</u> |

And:

> \<top\>, \<right\>, \<bottom\>, and \<left\> may either have a \<length\> value or 'auto'. Negative lengths are permitted. The value 'auto' means that a given edge of the clipping region will be the same as the edge of the element's generated border box (i.e., 'auto' means the same as '0' for \<top\> and \<left\> (in left-to-right text, \<right\> in right-to-left text), the same as the ~~computed~~ <u>used</u> value of the height plus the sum of vertical padding and border widths for \<bottom\>, and the same as the ~~computed~~ <u>used</u> value of the width plus the sum of the horizontal padding and border widths for \<right\> (in left-to-right text, \<left\> in right-to-left text), such that four 'auto' values result in the clipping region being the same as the element's border box).

<a id="q21.456"></a>

### <a id="t.12.5"></a>C.7.67 [Section 12.5 Lists](https://www.w3.org/TR/2009/CR-CSS2-20090908/generate.html#lists)

\[2010-10-05\] Improve wording: the marker box of a list item isn't “optional,” it is sometimes absent. Change:

> CSS 2.1 offers basic visual formatting of lists. An element with 'display: list-item' generates a principal box for the element's content ~~and an optional marker box~~ <u>and, depending on the values of 'list-style-type' and 'list-style-image', possibly also a marker box</u> as a visual indication that the element is a list item.

<a id="q21.457"></a>

### <a id="t.12.5.1"></a>C.7.68 [Section 12.5.1 Lists: the 'list-style-type', 'list-style-image', 'list-style-position', and 'list-style' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/generate.html#list-style)

\[2010-07-14\] Because of persistent incompatibilites between implementations, the constraints on the position of 'outside' markers are relaxed in the presence of floats. This will be fixed in a future specification.

<a id="q21.458"></a>

### <a id="t.12.5.1a"></a>C.7.69 [Section 12.5.1 Lists: the 'list-style-type', 'list-style-image', 'list-style-position', and 'list-style' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/generate.html#list-style)

\[2010-08-06\] The 'armenian' list-style-type refers to <em>uppercase</em> Armenian numbering.

<a id="q21.459"></a>

### <a id="t.12.5.1b"></a>C.7.70 [Section 12.5.1 Lists: the 'list-style-type', 'list-style-image', 'list-style-position', and 'list-style' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/generate.html#list-style)

\[2010-08-06\] Define the order of 'inside' marker boxes and ':before' pseudo-elements:

> <strong>inside</strong>  
> The marker box is <u>placed as</u> the first inline box in the principal block box, ~~after which the element's content flows~~ <u>before the element's content and before any :before pseudo-elements</u>.

<a id="q21.460"></a>

### <a id="t.12.5.1c"></a>C.7.71 [Section 12.5.1 Lists: the 'list-style-type', 'list-style-image', 'list-style-position', and 'list-style' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/generate.html#list-style)

\[2010-08-06\] CSS 2.1 does not specify the precise location of an 'outside' marker box, <em>including</em> its z-order. Append:

> CSS 2.1 does not specify the precise location of the marker box <u>or its position in the painting order</u>

<a id="q21.461"></a>

### <a id="t.12.5.1d"></a>C.7.72 [Section 12.5.1 Lists: the 'list-style-type', 'list-style-image', 'list-style-position', and 'list-style' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/generate.html#list-style)

\[2010-11-25\] Because of historical ambiguity, CSS level 2 does not yet require the marker to be visible when 'list-style-position' is 'outside' and 'overflow' is other than 'visible'. Insert in the definition of 'outside':

> <u>In CSS 2.1, a UA may hide the marker if the element's ['overflow'](css2--visufx.html--2bc674cb7ab0.md#propdef-overflow) is other than 'visible'. (This is expected to change in the future.)</u>

<a id="q21.462"></a>

### <a id="t.13.2"></a>C.7.73 [Section 13.2 Page boxes: the @page rule](https://www.w3.org/TR/2009/CR-CSS2-20090908/page.html#page-box)

\[2010-07-07\] The @page rule can contain not just declarations but also other @-rules. (There aren't any such nested @-rules defined in level 2, but there are in level 3.)

> An @page rule consists of the keyword "@page", followed by an optional page selector, followed by a block ~~of declarations~~ <u>containing declarations and at-rules</u>.
>
> <u>Note: CSS level 2 has no at-rules that may appear inside @page, but such at-rules are expected to be defined in level 3.</u>

And add just above section 13.2.1:

> <u>The rules for handling malformed declarations, malformed statements, and invalid at-rules inside @page are as defined in section 4.2, with the following addition: when the UA expects the start of a declaration or at-rule (i.e., an IDENT token or an ATKEYWORD token) but finds an unexpected token instead, that token is considered to be the first token of a malformed declaration. I.e., the rule for malformed declarations, rather than malformed statements is used to determine which tokens to ignore in that case.</u>

<a id="q21.463"></a>

### <a id="t.13.2.2"></a>C.7.74 [Section 13.2.2 Page selectors: selecting left, right, and first pages](https://www.w3.org/TR/2009/CR-CSS2-20090908/page.html#page-selectors)

\[2010-10-25\] Whether the first page of a document is :left or :right depends on the major writing direction. Give an example of <em>how:</em>

> All pages are automatically classified by user agents into either the :left or :right pseudo-class. <u>Whether the first page of a document is :left or :right depends on the major writing direction of the root element. For example, the first page of a document with a left-to-right major writing direction would be a :right page, and the first page of a document with a right-to-left major writing direction would be a :left page. To explicitly force a document to begin printing on a left or right page, authors can insert a page break before the first generated box.</u>

And in 13.3.1:

> ~~Whether the first page of a document is :left or :right depends on the major writing direction of the document.~~

<a id="q21.464"></a>

### <a id="t.13.3.2"></a>C.7.75 [Section 13.3.2 Breaks inside elements: 'orphans', 'widows'](https://www.w3.org/TR/2009/CR-CSS2-20090908/page.html#break-inside)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.” Change for both 'orphans' and 'widows':

> |                     |                                                 |
> |---------------------|-------------------------------------------------|
> | <em>Applies to:</em> | ~~block-level~~ <u>block container</u> elements |

And change:

> The 'orphans' property specifies the minimum number of lines in a block ~~element~~ <u>container</u> that must be left at the bottom of a page. The 'widows' property specifies the minimum number of lines in a block ~~element~~ <u>container</u> that must be left at the top of a page. Examples of how they are used to control page breaks are given below.

<a id="q21.465"></a>

### <a id="t.13.3.3"></a>C.7.76 [Section 13.3.3 Allowed page breaks](https://www.w3.org/TR/2009/CR-CSS2-20090908/page.html#allowed-page-breaks)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.” Change:

> 1.  In the vertical margin between block<u>-level</u> boxes. \[…\]
> 2.  Between line boxes inside a block <u>container</u> box.
> 3.  Between the content edge of a block <u>container</u> box and the outer edges of its child content \[…\]

<a id="q21.466"></a>

### <a id="t.15.3"></a>C.7.77 [Section 15.3 Font family: the 'font-family' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/fonts.html#font-family-prop)

\[2010-07-19\] The specification was ambiguous as to whether parentheses, brackets and braces in font names must <em>always</em> be escaped, or only when needed to conform to the syntax for declarations. Because of that, and because of the many bugs in implementations, <em>all</em> font names must now either be quoted, or be escaped so as to consist of only identifiers.

<a id="q21.467"></a>

### <a id="t.15.3.1"></a>C.7.78 [Section 15.3.1 Generic font families](https://www.w3.org/TR/2009/CR-CSS2-20090908/fonts.html#generic-font-families)

\[2010-08-26\] Make it clearer that CSS does not try to define what fonts are serif or sans-serif:

> <strong>15.3.1.1 serif</strong>
>
> Glyphs of serif fonts, as the term is used in CSS, <u>tend to</u> have finishing strokes, flared or tapering ends, or have actual serifed endings (including slab serifs). \[&#x26;hellip\]
>
> <strong>15.3.1.2 sans-serif</strong>
>
> Glyphs in sans-serif fonts, as the term is used in CSS, <u>tend to</u> have stroke endings that are plain – ~~without any~~ <u>with little or no</u> flaring, cross stroke, or other ornamentation. \[…\]

<a id="q21.468"></a>

### <a id="t.15.6"></a>C.7.79 [Section 15.6 Font boldness: the 'font-weight' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/fonts.html#font-boldness)

\[2010-04-19\] The meaning of the keywords 'bolder' and 'lighter' no longer depends on both the inherited weight and the actually used font, but only on the inherited weight.

<a id="q21.469"></a>

### <a id="t.15.6q"></a>C.7.80 [Section 15.6 Font boldness: the 'font-weight' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/fonts.html#font-boldness)

\[2010-10-13\] Clarify the algorithm for mapping CSS font weight values to the actual weights of a font and make it normative:

> The association of other weights within a family to the numerical weight values is intended only to preserve the ordering of darkness within that family. However, the following heuristics tell how the assignment is done in typical cases:
>
> - If the font family already uses a numerical scale with nine values (like e.g., <em>OpenType</em> does), the font weights should be mapped directly.
> - If there is both a face labeled <em>Medium</em> and one labeled <em>Book, Regular, Roman</em> or <em>Normal,</em> then the <em>Medium</em> is normally assigned to the '500'.
> - The font labeled "Bold" will often correspond to the weight value '700'.
>
> <u>Once the font family's weights are mapped onto the CSS scale, missing weights are selected as follows:</u>
>
> - ~~If there are fewer then 9 weights in the family, the default algorithm for filling the "holes" is as follows. If '500' is unassigned, it will be assigned the same font as '400'. If any of the values '600', '700', '800' or '900' remains unassigned, they are assigned to the same face as the next darker assigned keyword, if any, or the next lighter one otherwise. If any of '300', '200' or '100' remains unassigned, it is assigned to the next lighter assigned keyword, if any, or the next darker otherwise.~~
> - <u>If the desired weight is less than 400, weights below the desired weight are checked in descending order followed by weights above the desired weight in ascending order until a match is found.</u>
> - <u>If the desired weight is greater than 500, weights above desired weight are checked in ascending order followed by weights below the desired weight in descending order until a match is found.</u>
> - <u>If the desired weight is 400, 500 is checked first and then the rule for desired weights less than 400 is used.</u>
> - <u>If the desired weight is 500, 400 is checked first and then the rule for desired weights less than 400 is used.</u>

<a id="q21.470"></a>

### <a id="t.15.7"></a>C.7.81 [Section 15.7 Font size: the 'font-size' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/fonts.html#font-size-props)

\[2010-08-06\] Changed “Percentages: refer to ~~parent element's~~ font size” to “Percentages: refer to <u>inherited</u> font size” so that it uses the same terminology as [Section 4.3.3.](https://www.w3.org/TR/2009/CR-CSS2-20090908/syndata.html#percentage-units)

<a id="q21.471"></a>

### <a id="t.16.1"></a>C.7.82 [Section 16.1 Indentation: the 'text-indent' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#indentation-prop)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.” Change:

> |                     |                                                                                 |
> |---------------------|---------------------------------------------------------------------------------|
> | <em>Applies to:</em> | ~~block-level elements, table cells and inline blocks~~ <u>block containers</u> |
>
> \[…\]
>
> This property specifies the indentation of the first line of text in a block <u>container</u>.

<a id="q21.472"></a>

### <a id="t.16.1a"></a>C.7.83 [Section 16.1 Indentation: the 'text-indent' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#indentation-prop)

\[2010-10-25\] Clarify that the “first line” of the “first box,” etc., is the same as the “first formatted line” of chapter 5:

> <u>'Text-indent' only affects a line if it is the first formatted line of an element. For example, the first line of an anonymous block box is only affected if it is the first child of its parent element.</u>

<a id="q21.473"></a>

### <a id="t.16.2"></a>C.7.84 [Section 16.2 Alignment: the 'text-align' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#alignment-prop)

\[2010-07-19\] The value 'pre-line' of 'white-space' does <em>not</em> inhibit justification. (Only lines that end with an explicit newline aren't justified, as is the case for any value of 'white-space'.) But, 'pre-wrap' <em>does</em> inhibit justification. Replace

> ~~If the computed value of text-align is 'justify' while the computed value of white-space is 'pre' or 'pre-line', the actual value of text-align is set to the initial value.~~

with

> <u>If an element has a computed value for 'white-space' of 'pre' or 'pre-wrap', then neither the glyphs of that element's text content nor its white space may be altered for the purpose of justification.</u>

<a id="q21.474"></a>

### <a id="t.16.2a"></a>C.7.85 [Section 16.2 Alignment: the 'text-align' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#alignment-prop)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.” Change:

> |                     |                                                                                 |
> |---------------------|---------------------------------------------------------------------------------|
> | <em>Applies to:</em> | ~~block-level elements, table cells and inline blocks~~ <u>block containers</u> |
>
> This property describes how inline<u>-level</u> content of a block <u>container</u> is aligned.

And:

> \[…\] In the case of 'left', 'right' and 'center', this property specifies how the inline<u>-level</u> boxes within each line box align with respect to the line box's left and right sides; alignment is not with respect to the viewport. In the case of 'justify', this property specifies that the inline<u>-level</u> boxes are to be made flush with both sides of the block <u>container</u> if possible, by expanding or contracting the contents of inline boxes, else aligned as for the initial value.

<a id="q21.475"></a>

### <a id="t.16.3.1"></a>C.7.86 [Section 16.3.1 Underlining, overlining, striking, and blinking: the 'text-decoration' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#lining-striking-props)

\[2010-08-24\] Clarify that 'text-decoration' does not propagate to inline-table and inline-block elements. Change:

> This property describes decorations that are added to the text of an element using the element's color. When specified on ~~an inline element, it affects all the boxes generated by that element; for all other elements, the decorations are propagated to an anonymous inline box that wraps all the in-flow inline children of the element, and to any block-level in-flow descendants. It is not, however, further propagated to floating and absolutely positioned descendants, nor to the contents of 'inline-table' and 'inline-block' descendants.~~ <u>or propagated to an inline element, it affects all the boxes generated by that element, and is further propagated to any in-flow block-level boxes that split the inline (see section 9.2.1.1). For block containers that establish an inline formatting context, the decorations are propagated to an anonymous inline element that wraps all the in-flow inline-level children of the block container. For all other elements it is propagated to any in-flow children. Note that text decorations are not propagated to floating and absolutely positioned descendants, nor to the contents of atomic inline-level descendants such as inline blocks and inline tables.</u>

and:

> ~~If an element contains no text, user agents must refrain from rendering these text decorations on the element. For example, images will not be underlined.~~ <u>User agents must not render these text decorations on content that is not text. For example, images and inline blocks must not be underlined.</u>

<a id="q21.476"></a>

### <a id="t.16.3.1a"></a>C.7.87 [Section 16.3.1 Underlining, overlining, striking, and blinking: the 'text-decoration' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#lining-striking-props)

\[2010-10-05\] CSS 2.1 does not specify if a text decoration that is specified on a transparent element ('visibility: hidden') is itself transparent, or only transparent where the text is transparent. Add this note:

> > <strong data-conversion-semantic="note">Note</strong>
> >
> > <u><em><strong>Note.</strong> If an element E has both 'visibility:
hidden' and 'text-decoration: underline', the underline is invisible
(although any decoration of E's parent <strong>is</strong> visible.) 
However, CSS 2.1 does not specify if the underline is visible or
invisible in E's children:</em></u>
> >
> > ```text
> > 
> > <span style="visibility: hidden; text-decoration: underline">
> >  <span style="visibility: visible">
> >   underlined or not?
> >  </span>
> > </span>
> > ```
> >
> > <u><em>This is expected to be specified in level 3 of
CSS.</em></u>

<a id="q21.477"></a>

### <a id="t.16.4"></a>C.7.88 [Section 16.4 Letter and word spacing: the 'letter-spacing' and 'word-spacing' properties](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#spacing-props)

\[2010-04-19\] Word spacing does not affect fixed-width spaces. Change:

> Word spacing affects each space (U+0020)~~,~~ <u>and</u> non-breaking space (U+00A0) ~~and ideographic space (U+3000)~~, left in the text after the white space processing rules have been applied. <u>The effect of the property on other word-separator characters is undefined. However general punctuation, characters with zero advance width (such as the zero with space U+200B) and fixed-width spaces (such as U+3000 and U+2000 through U+200A) are not affected.</u>

<a id="q21.478"></a>

### <a id="t.16.6"></a>C.7.89 [Section 16.6 White space: the 'white-space' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#white-space-prop)

\[2010-10-25\] If the document language specifies how newlines are represented, those newlines must be passed to the CSS UA as line feed (LF) characters. If the document language does not define how newlines are expressed (e.g., if text is inserted with the 'content' property), the CSS UA must treat CR, and CRLF as if they were LF:

> Newlines in the source can be represented by a carriage return (U+000D), a linefeed (U+000A) or both (U+000D U+000A) or by some other mechanism that identifies the beginning and end of document segments, such as the SGML RECORD-START and RECORD-END tokens. The CSS 'white-space' processing model assumes all newlines have been normalized to line feeds. <u>UAs that recognize other newline representations must apply the white space processing rules as if this normalization has taken place. If no newline rules are specified for the document language, each carriage return (U+000D) and CRLF sequence (U+000D U+000A) in the document text is treated as single line feed character. This default normalization rule also applies to generated content.</u>
>
> \[…\]
>
> 1.  Each tab (U+0009), ~~carriage return (U+000D),~~ or space (U+0020) character surrounding a linefeed (U+000A) character is removed if 'white-space' is set to 'normal', 'nowrap', or 'pre-line'.

<a id="q21.479"></a>

### <a id="t.16.6.1"></a>C.7.90 [Section 16.6.1 The 'white-space' processing model](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#white-space-model)

\[2010-08-06\] The sentence that absolutely positioned elements do not create line breaking opportunities is normative, not informative.

<a id="q21.480"></a>

### <a id="t.16.6.1a"></a>C.7.91 [Section 16.6.1 The 'white-space' processing model](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#white-space-model)

\[2010-08-06\] The first paragraph is moved to 9.2.2.1. Also, as is clear from the latter section, the “should” is a “must”:

> Any text that is directly contained inside a block <u>container</u> element (not inside an inline element) ~~should~~ <u>must</u> be treated as an anonymous inline element.

<a id="q21.481"></a>

### <a id="t.16.6.1b"></a>C.7.92 [Section 16.6.1 The 'white-space' processing model](https://www.w3.org/TR/2009/CR-CSS2-20090908/text.html#white-space-model)

\[2010-08-24\] More consistent use of “inline-level” and “block-level.” Change:

> Then, the ~~entire block is rendered~~ <u>block container's inlines are laid out</u>.

<a id="q21.482"></a>

### <a id="t.17.2"></a>C.7.93 [Section 17.2 The CSS table model](https://www.w3.org/TR/2009/CR-CSS2-20090908/tables.html#table-display)

\[2010-08-04\] Clarify that the term “row group” includes header groups and footer groups as well:

> Thus, the table model consists of tables, captions, rows, row groups <u>(including header groups and footer groups)</u>, columns, column groups, and cells.

<a id="q21.483"></a>

### <a id="t.17.2.1"></a>C.7.94 [Section 17.2.1 Anonymous table objects](https://www.w3.org/TR/2009/CR-CSS2-20090908/tables.html#anonymous-boxes)

\[2010-08-24\] XML and HTML5, unlike SGML, do not automatically remove insignificant white space. Change the rules for generating anonymous table elements to suppress most white space between elements, rather than consider it the content of an anonymous table cell.

<a id="q21.484"></a>

### <a id="t.17.2.1a"></a>C.7.95 [Section 17.2.1 Anonymous table objects](https://www.w3.org/TR/2009/CR-CSS2-20090908/tables.html#anonymous-boxes)

\[2010-08-24\] The static position of absolutely positioned elements <em>between</em> table cells or rows was not very useful. Define that the static position of such an element is found not just as if the element had 'position: static', but also <em>had 'display: inline' and zero width and height.</em>

<a id="q21.485"></a>

### <a id="t.17.4"></a>C.7.96 [Section 17.4 Tables in the visual formatting model](https://www.w3.org/TR/2009/CR-CSS2-20090908/tables.html#model)

\[2010-04-19\] The caption of the image still describes the image as it was in the previous version. Change:

> Diagram of a table with a caption above it~~; the top margin of the caption is collapsed with the top margin of the table~~.

<a id="q21.486"></a>

### <a id="t.17.4a"></a>C.7.97 [Section 17.4 Tables in the visual formatting model](https://www.w3.org/TR/2009/CR-CSS2-20090908/tables.html#model)

\[2010-10-13\] Clarify which of the two boxes generated by a table element is the principal box:

> In both cases, the table ~~box~~ generates ~~an anonymous box~~ <u>a principal block box called the table wrapper box</u> that contains the table box itself and any caption boxes (in document order). <u>The table box is a block-level box that contains the table's internal table boxes.</u> The caption boxes are block-level boxes that retain their own content, padding, margin, and border areas, and are rendered as normal ~~blocks~~ <u>block boxes</u> inside the ~~anonymous~~ <u>table wrapper</u> box. Whether the caption boxes are placed before or after the table box is decided by the 'caption-side' property, as described below.
>
> The ~~anonymous~~ <u>table wrapper</u> box is a 'block' box if the table is block-level, and an 'inline-block' box if the table is inline-level. The ~~anonymous~~ <u>table wrapper</u> box establishes a block formatting context. The table box (not the ~~anonymous~~ <u>table wrapper</u> box) is used when doing baseline vertical alignment for an 'inline-table'. The width of the ~~anonymous~~ <u>table wrapper</u> box is the border-edge width of the table box inside it, as described by section 17.5.2. Percentages on 'width' and 'height' on the table are relative to the ~~anonymous~~ <u>table wrapper</u> box's containing block, not the ~~anonymous~~ <u>table wrapper</u> box itself.
>
> The computed values of properties 'position', 'float', 'margin-\*', 'top', 'right', 'bottom', and 'left' on the table box are used on the ~~anonymous~~ <u>table wrapper</u> box instead of the table box. The table box uses the initial values for those properties.

<a id="q21.487"></a>

### <a id="t.17.5.2.2"></a>C.7.98 [Section 17.5.2.2 Automatic table layout](https://www.w3.org/TR/2009/CR-CSS2-20090908/tables.html#auto-table-layout)

\[2010-10-25\] The width of the table caption contributes to the width of the table if 'table-layout' is 'auto':

> This gives a maximum and minimum width for each column.
>
> <u>The caption width minimum (CAPMIN) is determined by calculating for each caption the minimum caption outer width as the MCW of a hypothetical table cell that contains the caption formatted as "display: block". The greatest of the minimum caption outer widths is CAPMIN.</u>
>
> Column <u>and caption</u> widths influence the final table width as follows:
>
> 1.  If the 'table' or 'inline-table' element's 'width' property has a computed value (W) other than 'auto', the ~~property's value as used for layout~~ <u>used width</u> is the greater of W<u>, CAPMIN,</u> and the minimum width required by all the columns plus cell spacing or borders (MIN). If ~~W~~ <u>the used width</u>is greater than MIN, the extra width should be distributed over the columns.
> 2.  If the 'table' or 'inline-table' element has 'width: auto', the ~~table width used for layout~~ <u>used width</u> is the greater of the table's containing block width<u>, CAPMIN,</u> and MIN. However, if <u>either CAPMIN or</u> the maximum width required by the columns plus cell spacing or borders (MAX) is less than that of the containing block, use ~~MAX~~ <u>max(MAX, CAPMIN)</u>.

<a id="q21.488"></a>

### <a id="t.17.5.3"></a>C.7.99 [Section 17.5.3 Table height algorithms](https://www.w3.org/TR/2009/CR-CSS2-20090908/tables.html#height-layout)

\[2010-07-15\] Clarify that the height of a table row can be influenced by 'vertical-align' and 'height', but the content box of the table cell is not affected.

> \[…\] it is the maximum of the row's specified 'height', <u>the specified 'height' of each cell in the row,</u> and the minimum height (MIN) required by the cells

and

> In CSS 2.1, the height of a cell box is the ~~maximum of the table cell's 'height' property and the minimum height required by the content (MIN).~~ <u>minimum height required by the content. The table cell's 'height' property can influence the height of the row, but it does not increase the height of the cell box.</u> ~~A value of 'auto' for 'height' implies that the value MIN will be used for layout.~~

<a id="q21.489"></a>

### <a id="t.17.5.4"></a>C.7.100 [Section 17.5.4 Horizontal alignment in a column](https://www.w3.org/TR/2009/CR-CSS2-20090908/tables.html#column-alignment)

\[2010-08-24\] More consistent use of “inline-level.” Change:

> The horizontal alignment of ~~a cell's inline content~~ <u>inline-level content</u> within a cell box

<a id="q21.490"></a>

### <a id="t.B.2"></a>C.7.101 [Section B.2 Informative references](https://www.w3.org/TR/2009/CR-CSS2-20090908/refs.html)

\[2010-08-06\] BCP 47 replaces RFC 3066.

<a id="q21.491"></a>

### <a id="t.D"></a>C.7.102 [Section D. Default style sheet for HTML 4](https://www.w3.org/TR/2009/CR-CSS2-20090908/sample.html)

\[2010-10-05\] HTML defines that HTML's block elements represent a Unicode embedding even if they are displayed inline by means of a style sheet. The default style sheet for HTML didn't yet express that. Add:

> ```text
> 
> html, address,
> blockquote,
> body, dd, div,
> dl, dt, fieldset, form,
> frame, frameset,
> h1, h2, h3, h4,
> h5, h6, noframes,
> ol, p, ul, center,
> dir, hr, menu, pre   { display: block; unicode-bidi: embed }
> ```
<a id="q21.492"></a>

### <a id="t.E.2"></a>C.7.103 [Section E.2 Painting order](https://www.w3.org/TR/2009/CR-CSS2-20090908/zindex.html#painting-order)

\[2010-07-07\] Clarification:

> The ~~stacking order for~~ <u>painting order for the descendants of</u> an element generating a stacking context (see the 'z-index' property) is: \[…\]

<a id="q21.493"></a>

### <a id="t.G"></a>C.7.104 [Appendix G Grammar of CSS 2.1](https://www.w3.org/TR/2009/CR-CSS2-20090908/grammar.html)

\[2010-10-25\] The appendix is not normative.

<a id="q21.494"></a>

## <a id="errata4"></a>C.8 Changes since the working draft of 7 December 2010

<a id="q21.495"></a>

### <a id="u.8.3.1"></a>C.8.1 [8.3.1 Collapsing margins](https://www.w3.org/TR/2010/WD-CSS2-20101207/box.html#collapsing-margins)

The section is completely rewritten to make the normative text shorter and clearer.

<a id="q21.496"></a>

### <a id="u.10.8.1"></a>C.8.2 [10.8.1 Leading and half-leading](https://www.w3.org/TR/2010/WD-CSS2-20101207/visudet.html#leading)

The remark about equal line spacing is made more precise and put in green, to make it clearer that it is a note:

> > <strong data-conversion-semantic="note">Note</strong>
> >
> > <em><del>Generally,</del>
<ins><strong>Note.</strong></ins> when there is only one value of
'line-height' for all inline boxes in a block container box <ins>and
they are all in the same font</ins> (and <ins>there are</ins> no
replaced elements, inline-block elements, etc.), the above will ensure
that baselines of successive lines are exactly 'line-height'
apart. This is important when columns of text in different fonts have
to be aligned, for example in a table.</em>

<a id="q21.497"></a>

### <a id="u.10.3"></a>C.8.3 [10.3 Calculating widths and margins](https://www.w3.org/TR/2010/WD-CSS2-20101207/visudet.html#Computing_widths_and_margins)

Added a note that the width calculation only yields a tentative value, still to be compared to 'min-width' and 'max-width'

> > <strong data-conversion-semantic="note">Note</strong>
> >
> > <u><em><strong>Note.</strong> The used value of
'width' calculated below is a tentative value, and may have to be
calculated multiple times, depending on 'min-width' and 'max-width',
see the section Minimum and maximum widths below.</em></u>

A similar note is added to section 10.6 about calculating heights.

<a id="q21.498"></a>

### <a id="u.14.3"></a>C.8.4 14.3 Gamma correction

The section on gamma correction was removed. It existed only to help implementations on certain operating systems of the 1990s.

<a id="q21.499"></a>

### <a id="u.11.1.2"></a>C.8.5 [11.1.2 Clipping: the 'clip' property](https://www.w3.org/TR/2010/WD-CSS2-20101207/visufx.html#clipping)

The 2nd and 4th offsets of the clip rectangle are offsets from the left edge of the element. The 'direction' property no longer has an influence.

<a id="q21.500"></a>

### <a id="u.9.4.2"></a>C.8.6 [9.4.2 Inline formatting contexts](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#inline-formatting)

The words "line feed" were a typing error. The intended words are "forced line break."

(The sentence was subsequently changed further as a result of [another issue.](#u.9.4.2a))

<a id="q21.501"></a>

### <a id="u.10.3.2"></a>C.8.7 [10.3.2 Inline, replaced elements](https://www.w3.org/TR/2010/WD-CSS2-20101207/visudet.html#inline-replaced-width)

No image formats were found that allow an intrinsic size to be expressed as a percentage. The relevant definitions are removed:

> ~~Percentage intrinsic widths are first evaluated with respect to the containing block's width, if that width does not itself depend on the replaced element's width. If it does, then the resulting layout is undefined in CSS 2.1.~~

Similarly in [10.6.2](https://www.w3.org/TR/2010/WD-CSS2-20101207/visudet.html#inline-replaced-height):

> ~~Percentage intrinsic heights are evaluated with respect to the containing block's height, if that height is specified explicitly, or if the replaced element is absolutely positioned. If neither of these conditions is met, then percentage values on such replaced elements cannot be resolved and such elements are assumed to have no intrinsic height.~~

And in [12.5.1](https://www.w3.org/TR/2010/WD-CSS2-20101207/generate.html#list-style):

> ~~2. If the image's intrinsic width or height is given as a percentage, then that percentage is resolved against 1em.~~

<a id="q21.502"></a>

### <a id="u.10.1"></a>C.8.8 [10.1 Definition of "containing block"](https://www.w3.org/TR/2010/WD-CSS2-20101207/visudet.html#containing-block-details)

In CSS 2.1, it is undefined what the containing block of an absolutely positioned element is, if its nearest positioned ancestor is inline and split over multiple lines:

> 4\. If the element has 'position: absolute' \[&#x26;hellip\] following way;
>
> 1.  In the case that the ancestor is an ~~inline box~~ <u>inline-level element</u>, the containing block ~~depends on the ['direction'](css2--visuren.html--3f334c530cf4.md#propdef-direction) property of the ancestor:~~ <u>is the bounding box around the padding boxes of the first and the last inline boxes generated for that element. In CSS 2.1, if the inline element is split across multiple lines, the containing block is undefined.</u>
>     1.  ~~If the ['direction'](css2--visuren.html--3f334c530cf4.md#propdef-direction) is 'ltr', the top and left of the containing block are the top and left padding edges of the first box generated by the ancestor, and the bottom and right are the bottom and right padding edges of the last box of the ancestor.~~
>     2.  ~~If the ['direction'](css2--visuren.html--3f334c530cf4.md#propdef-direction) is 'rtl', the top and right are the top and right padding edges of the first box generated by the ancestor, and the bottom and left are the bottom and left padding edges of the last box of the ancestor.~~
>
>     > <strong data-conversion-semantic="note">Note</strong>
>     >
>     > ~~Note: This may cause the containing block's width to be negative.~~

<a id="q21.503"></a>

### <a id="u.13.2.2"></a>C.8.9 [13.2.2 Page selectors: selecting left, right, and first pages](https://www.w3.org/TR/2010/WD-CSS2-20101207/page.html#page-selectors)

CSS 2.1 does not define if ':first' applies to the first page or the first non-blank page:

> <u>If a forced break occurs before the first generated box, it is undefined in CSS 2.1 whether ':first' applies to the blank page before the break or to the page after it.</u>

<a id="q21.504"></a>

### <a id="u.8.3.1a"></a>C.8.10 [8.3.1 Collapsing margins](https://www.w3.org/TR/2010/WD-CSS2-20101207/box.html#collapsing-margins)

Added a note with a link to 9.4.2, which defines types of line boxes that exist but do not interfere with collapsing margins.

> - no line boxes, no clearance, no padding and no border separate them <u><strong data-conversion-semantic="note">Note:</strong> (Note that [certain zero-height line boxes](css2--visuren.html--3f334c530cf4.md#phantom-line-box) (see [9.4.2](css2--visuren.html--3f334c530cf4.md#inline-formatting)) are ignored for this purpose.)</u>

<a id="q21.505"></a>

### <a id="u.10.8"></a>C.8.11 [10.8 Line height calculations: the 'line-height' and 'vertical-align' properties](https://www.w3.org/TR/2010/WD-CSS2-20101207/visudet.html#line-height)

The definition of which height is used for the different kinds of inline-level boxes is made explicit, rather than linked:

> 1.  The height of each inline-level box in the line box is calculated. <u>For replaced elements, inline-block elements, and inline-table elements, this is the height of their margin box; for inline boxes, this is their 'line-height'.</u> (See ["Calculating heights and margins"](css2--visudet.html--12e8bc0e6b7c.md#Computing_heights_and_margins) and the ~~'line-height' property~~ <u>[height of inline boxes](css2--visudet.html--12e8bc0e6b7c.md#inline-box-height) in ["Leading and half-leading"](css2--visudet.html--12e8bc0e6b7c.md#leading)</u>.)

The part of the definition that was in [10.6.2](css2--visudet.html--12e8bc0e6b7c.md#inline-replaced-height) is removed:

> ~~For 'inline' and 'inline-block' elements, the margin box is used when calculating the height of the line box.~~

<a id="q21.506"></a>

### <a id="u.10.8.1a"></a>C.8.12 [10.8.1 Leading and half-leading](https://www.w3.org/TR/2010/WD-CSS2-20101207/visudet.html#leading)

Inserted the following before the definitions of the keywords of 'vertical-align' to define precisely which box is aligned:

> <u>In the following definitions, for inline non-replaced elements, the box used for alignment is the box whose height is the 'line-height' (containing the box's glyphs and the half-leading on each side, see [above](css2--visudet.html--12e8bc0e6b7c.md#inline-box-height)). For all other elements, the box used for alignment is the margin box.</u>

Also, to make sure there always is a box whose height is 'line-height', a phrase earlier in the same section was removed:

> User agent must align the glyphs in a non-replaced inline box to each other by their relevant baselines~~, and to nested inline boxes according to 'vertical-align'~~.

And another modified:

> The height of the inline box ~~is then the smallest such that it encloses all glyphs and their leading, as well as all nested inline boxes.~~ <u>encloses all glyphs and their half-leading on each side and is thus exactly 'line-height'. Boxes of child elements do not influence this height.</u>

<a id="q21.507"></a>

### <a id="u.10.6.1a"></a>C.8.13 [10.6.1 Inline, non-replaced elements](https://www.w3.org/TR/2010/WD-CSS2-20101207/visudet.html#inline-non-replaced)

Improve language:

> The vertical padding, border and margin of an inline, non-replaced box start at the top and bottom of the content area, ~~not~~ <u>and has nothing to do with</u> the 'line-height'. But only the 'line-height' is used when calculating the height of the line box.

<a id="q21.508"></a>

### <a id="u.9.5.1"></a>C.8.14 [9.5.1 Positioning the float: the 'float' property](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#float-position)

A left float must not only not overlap a right float, but must also not be completely to the right of it.

> 3\. The right [outer edge](css2--box.html--8875bbcdefe6.md#outer-edge) of a left-floating box may not be to the right of the left [outer edge](css2--box.html--8875bbcdefe6.md#outer-edge) of any right-floating box that is ~~to the right of~~ <u>next to</u> it. Analogous rules hold for right-floating elements.

<a id="q21.509"></a>

### <a id="u.9.2.1.1"></a>C.8.15 [9.2.1.1 Anonymous block boxes](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#anonymous-block-level)

An error in the description of the example:

> The resulting boxes would be ~~an anonymous block box around~~ <u>a block box representing</u> the BODY, containing an anonymous block box around C1, the SPAN block box, and another anonymous block box around C2.

<a id="q21.510"></a>

### <a id="u.5.12.1"></a>C.8.16 [5.12.1 The :first-line pseudo-element](https://www.w3.org/TR/2010/WD-CSS2-20101207/selector.html#first-line-pseudo)

UAs are not required to support 'vertical-align' on '::first-line'.

> The following properties apply to a :first-line pseudo-element: font properties, color property, background properties, 'word-spacing', 'letter-spacing', 'text-decoration', ~~'vertical-align',~~ 'text-transform', <u>and</u> 'line-height'. UAs may apply other properties as well.

<a id="q21.511"></a>

### <a id="u.16.6"></a>C.8.17 [16.6 White space: the 'white-space' property](https://www.w3.org/TR/2010/WD-CSS2-20101207/text.html#white-space-prop)

CSS 2.1 does not define whether the Line Separator character in Unicode and other forced line break characters (other than LF) cause a line break. (Level 3 will probably define this in detail.)

> pre  
> This value prevents user agents from collapsing sequences of white space. Lines are only broken at ~~newlines in the source, or at occurrences of "&#x5C;A" in generated content~~ <u>preserved newline characters</u>.

and

> pre-wrap  
> This value prevents user agents from collapsing sequences of white space. Lines are broken at ~~newlines in the source, at occurrences of "&#x5C;A" in generated content,~~ <u>preserved newline characters,</u> and as necessary to fill line boxes.
>
> pre-line  
> This value directs user agents to collapse sequences of white space. Lines are broken at ~~newlines in the source, at occurrences of "&#x5C;A" in generated content,~~ <u>preserved newline characters,</u> and as necessary to fill line boxes.

and add this paragraph:

> <u>UAs must recognize line feeds (U+000A) as newline characters. UAs may additionally treat other forced break characters as newline characters per UAX14.</u>

<a id="q21.512"></a>

### <a id="u.12.5.1"></a>C.8.18 [12.5.1 Lists: the 'list-style-type', 'list-style-image', 'list-style-position', and 'list-style' properties](https://www.w3.org/TR/2010/WD-CSS2-20101207/generate.html#list-style)

CSS 2.1 omits to define how the implicit counters of 'list-item' are reset and incremented. This will be specified in level 3.

> <u>CSS 2.1 does not define how the list numbering is reset and incremented. This is expected to be defined in the CSS List Module [\[CSS3LIST\]](css2--refs.html--f208d881d0b7.md#ref-CSS3LIST).</u>

<a id="q21.513"></a>

### <a id="u.9.7"></a>C.8.19 [9.7 Relationships between 'display', 'position', and 'float'](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#dis-pos-flo)

Some UAs treat 'display: list-item' on the root element as 'block'. Allow that behavior for now:

> 4\. Otherwise, if the element is the root element, 'display' is set according to the table below<u>, except that it is undefined in CSS 2.1 whether a specified value of 'list-item' becomes a computed value of 'block' or 'list-item'</u>.

<a id="q21.514"></a>

### <a id="u.9.4.2a"></a>C.8.20 [9.4.2 Inline formatting contexts](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#inline-formatting)

Empty line boxes aren't generated at all, rather than just ignored for margin collapsing. But their virtual position must still be calculated if they contain empty inlines with absolutely positioned or floating descendants:

> <u>Line boxes are created as needed to hold inline-level content within an inline formatting context.</u> Line boxes that contain no text, no [preserved white space,](css2--text.html--467a8857ae69.md#white-space-prop) no inline elements with non-zero margins, padding, or borders, and no other [in-flow](css2--visuren.html--3f334c530cf4.md#positioning-scheme) content (such as images, inline blocks or inline tables), and do not end with a ~~line feed~~ <u>preserved newline</u> must be treated as zero-height line boxes <u>for the purposes of determining the positions of any elements inside of them, and treated as not existing for any other purpose.</u> ~~For the purposes of margin collapsing, this line box must be ignored.~~

<a id="q21.515"></a>

### <a id="u.4.1.9"></a>C.8.21 [4.1.9 Comments](https://www.w3.org/TR/2010/WD-CSS2-20101207/syndata.html#comments)

Use same phrasing for comment tokens as in [section 4.1.1:](css2--syndata.html--02e71c159e14.md#comments)

> They may occur anywhere ~~between~~ <u>outside other</u> tokens

<a id="q21.516"></a>

### <a id="u.12.5.1a"></a>C.8.22 [12.5.1 Lists: the 'list-style-type', 'list-style-image', 'list-style-position', and 'list-style' properties](https://www.w3.org/TR/2010/WD-CSS2-20101207/syndata.html#comments)

The size computation of list marker images without an intrinsic size is modified to be consistent with how image sizes are computed in other places, using 1em for the available width and 1:1 for the default aspect ratio:

> 1.  ~~If the image has an intrinsic width or height, then that intrinsic width/height becomes the image's used width/height.~~ <u>If the image has a intrinsic width and height, the used width and height are the intrinsic width and height.</u>
> 2.  ~~If the image has no intrinsic ratio and a ratio cannot be calculated from its width and height, then its intrinsic ratio is assumed to be 1:1.~~ <u>Otherwise, if the image has an intrinsic ratio and either an intrinsic width or an intrinsic height, the used width/height is the same as the provided intrinsic width/height, and the used value of the missing dimension is calculated from the provided dimension and the ratio.</u>
> 3.  ~~If the image has a width but no height, its height is calculated from the intrinsic ratio.~~ <u>Otherwise, if the image has an intrinsic ratio, the used width is 1em and the used height is calculated from this width and the intrinsic ratio. If this would produce a height larger than 1em, then the used height is instead set to 1em and the used width is calculated from this height and the intrinsic ratio.</u>
> 4.  ~~If the image's height cannot be resolved from the rules above, then the image's height is assumed to be 1em.~~ <u>Otherwise, the image's used width is its intrinsic width if it has one, or else 1em. The image's used height is its intrinsic height if it has one, or else 1em.</u>
> 5.  ~~If the image has no intrinsic width, then its width is calculated from the resolved height and the intrinsic ratio.~~

<a id="q21.517"></a>

### <a id="u.9.5.1a"></a>C.8.23 [9.5.1 Positioning the float: the 'float' property](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#float-position)

Because of lack of sufficient implementations, the top of a floating box is allowed to be above the top of earlier boxes in certain difficult cases. Add after the numbered list:

> <u>But in CSS 2.1, if, within the block formatting context, there is an in-flow negative vertical margin such that the float's position is above the position it would be at were all such negative margins set to zero, the position of the float is undefined.</u>

<a id="q21.518"></a>

### <a id="u.9.3"></a>C.8.24 [9.3 Positioning schemes](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#positioning-scheme)

Add formal definitions of the terms “out of flow,” “in-flow” and “flow of an element”:

> <u>An element is called out of flow if it is floated, absolutely positioned, or is the root element. An element is called in-flow if it is not out-of-flow. The flow of an element <var>A</var> is the set consisting of <var>A</var> and all in-flow elements whose nearest out-of-flow ancestor is <var>A</var>.</u>

<a id="q21.519"></a>

### <a id="u.9.10"></a>C.8.25 [9.10 Text direction: the 'direction' and 'unicode-bidi' properties](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#direction)

The list of features affected by 'direction' is not meant to be exclusive:

> This property specifies the base writing direction of blocks and the direction of embeddings and overrides (see ['unicode-bidi'](css2--visuren.html--3f334c530cf4.md#propdef-unicode-bidi)) for the Unicode bidirectional algorithm. In addition, it specifies <u>such things as</u> the direction of [table](css2--tables.html--201812dd6e3c.md) column layout, the direction of horizontal [overflow](css2--visufx.html--2bc674cb7ab0.md#overflow), the position of an incomplete last line in a block in case of 'text-align: justify'.

<a id="q21.520"></a>

### <a id="u.16.3.1"></a>C.8.26 [16.3.1 Underlining, overlining, striking, and blinking: the 'text-decoration' property](https://www.w3.org/TR/2010/WD-CSS2-20101207/text.html#lining-striking-props)

Whether the effect of 'text-decoration' propagates into tables may be the subject of a separate property in level 3:

> \[…\] When specified on or propagated to an inline element, it affects all the boxes generated by that element, and is further propagated to any in-flow block-level boxes that split the inline (see [section 9.2.1.1](css2--visuren.html--3f334c530cf4.md#anonymous-block-level)). <u>But, in CSS 2.1, it is undefined whether the decoration propagates into block-level tables.</u>

<a id="q21.521"></a>

### <a id="u.16.3.1a"></a>C.8.27 [16.3.1 Underlining, overlining, striking, and blinking: the 'text-decoration' property](https://www.w3.org/TR/2010/WD-CSS2-20101207/text.html#lining-striking-props)

Clarify that the text for 'inset' and 'outset' only talks about how the border styles look (and not, e.g., about which style takes priority):

> <strong>&#x2A;inset</strong>  
> In the separated borders model, the border makes the entire box look as though it were embedded in the canvas. In the collapsing border model, <u>drawn the</u> same as 'ridge'.
>
> <strong>&#x2A;outset</strong>  
> In the separated borders model, the border makes the entire box look as though it were coming out of the canvas. In the collapsing border model, <u>drawn the</u> same as 'groove'.

<a id="q21.522"></a>

### <a id="u.10.4"></a>C.8.28 [10.4 Minimum and maximum widths: 'min-width' and 'max-width'](https://www.w3.org/TR/2010/WD-CSS2-20101207/visudet.html#min-max-widths)

Added:

> <u>In CSS 2.1, the effect of 'min-width' and 'max-width' on tables, inline tables, table cells, table columns, and column groups is undefined.</u>

<a id="q21.523"></a>

### <a id="u.9.3.2"></a>C.8.29 [9.3.2 Box offsets: 'top', 'right', 'bottom', 'left'](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#position-props)

'Top', right', 'bottom' and 'left' are always computed, independent of the value of other properties:

> <strong>'top'</strong>  
> |                       |                                                                                                                                                                                                                                                                                                |
> |-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
> | <em>Value:</em>   | [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| auto \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
> | <em>Initial:</em>   | auto                                                                                                                                                                                                                                                                                           |
> | <em>Applies to:</em>   | positioned elements                                                                                                                                                                                                                                                                            |
> | <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                             |
> | <em>Percentages:</em>   | refer to height of containing block                                                                                                                                                                                                                                                            |
> | <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                           |
> | <em>Computed value:</em>   | ~~for 'position:static', 'auto'. Otherwise:~~ if specified as a length, the corresponding absolute length; if specified as a percentage, the specified value; otherwise, 'auto'.                                                                                                               |

Analogously for 'right', 'bottom' and 'left'.

<a id="q21.524"></a>

### <a id="u.9.2.1.1a"></a>C.8.30 [9.2.1.1 Anonymous block boxes](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#anonymous-block-level)

Clarify that the two parts of an inline that is split by a block are on opposite sides of the block:

> When an inline box contains an in-flow block-level box, the inline box (and its inline ancestors within the same line box) are broken around the block-level box (and any block-level siblings that are consecutive or separated only by collapsible whitespace and/or out-of-flow elements), ~~dividing~~ <u>splitting</u> the inline box into two pieces (even if either side is empty)<u>, one on each side of the block-level box(es)</u>.

<a id="q21.525"></a>

### <a id="u.9.2.1.1b"></a>C.8.31 [17.4 Tables in the visual formatting model](https://www.w3.org/TR/2010/WD-CSS2-20101207/tables.html#model)

More precise rule for which properties apply to the table box and which to the table wrapper box:

> The computed values of properties 'position', 'float', 'margin-\*', 'top', 'right', 'bottom', and 'left' on the table ~~box~~ <u>element</u> are used on the table wrapper box ~~instead of~~ <u>and not</u> the table box~~. The table box uses the initial values for those properties.~~ <u>; all other values of non-inheritable properties are used on the table box and not the table wrapper box. (Where the table element's values are not used on the table and table wrapper boxes, the initial values are used instead.)</u>

<a id="q21.526"></a>

### <a id="u.11.1.2a"></a>C.8.32 [11.1.2 Clipping: the 'clip' property](https://www.w3.org/TR/2010/WD-CSS2-20101207/visufx.html#clipping)

Some text and arrows were added to the example to make it easier to see where the four offsets of the clip rectangle are applied:

> <a id="img-clip"></a>
>
> ![Two clipping regions](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/clip.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/clip-desc.html)

<a id="q21.527"></a>

### <a id="u.13.2"></a>C.8.33 [13.2 Page boxes: the @page rule](https://www.w3.org/TR/2010/WD-CSS2-20101207/page.html#page-box)

The definition of the @page rule didn't mention explicitly (except with examples) that white space is allowed:

> An @page rule consists of the keyword "@page", followed by an optional page selector, followed by a block containing declarations and at-rules. <u>Comments and white space are allowed, but optional, between the @page token and the page selector and between the page selector and the block.</u>

<a id="q21.528"></a>

### <a id="u.4.1.1"></a>C.8.34 [4.1.1 Tokenization](https://www.w3.org/TR/2010/WD-CSS2-20101207/syndata.html#tokenization)

Added an example to illustrate what is meant by “the longest match” in the tokenizer:

> > <strong data-conversion-semantic="example">Example</strong>
> >
> > Example(s):
> >
> > <u>For example, the rule of the longest match means that "`red-->`" is tokenized as the IDENT "`red--`" followed by the DELIM "`>`", rather than as an IDENT followed by a CDC.</u>

<a id="q21.529"></a>

### <a id="u.4.2"></a>C.8.35 [4.2 Rules for handling parsing errors](https://www.w3.org/TR/2010/WD-CSS2-20101207/syndata.html#parsing-errors)

Clarify that “end of line” means an end of line <em>character,</em> i.e., the end of file is not an end of line:

> User agents must close strings upon reaching the end of a line <u>(i.e., before an unescaped line feed, carriage return or form feed character)</u>, but then drop the construct (declaration or rule) in which the string was found.

<a id="q21.530"></a>

### <a id="u.3.1"></a>C.8.36 [3.1 Definitions](https://www.w3.org/TR/2010/WD-CSS2-20101207/conform.html#defs)

There may soon be a newer version of HTML then HTML4:

> An HTML user agent is one that supports <u>one or more of</u> the HTML ~~2.x, HTML 3.x, or HTML 4.x~~ specifications. A user agent that supports XHTML \[XHTML\], but not HTML ~~(as listed in the previous sentence)~~ is not considered an HTML user agent for the purpose of conformance with this specification.

<a id="q21.531"></a>

### <a id="u.4.3.4"></a>C.8.37 [4.3.4 URLs and URIs](https://www.w3.org/TR/2010/WD-CSS2-20101207/syndata.html#uri)

Make the note about parsing URLs shorter and clearer:

> > <strong data-conversion-semantic="note">Note</strong>
> >
> > <em>Note that COMMENT tokens cannot occur within other tokens:
thus, "url(/&#x2A;x&#x2A;/pic.png)" denotes the URI "/&#x2A;x&#x2A;/pic.png", not
"pic.png".</em>

<a id="q21.532"></a>

### <a id="u.9.5"></a>C.8.38 [9.5 Floats](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#floats)

Clarify the note:

> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: this means that floats with zero <u>outer</u> height or negative <u>outer</u> height do not shorten line boxes.

<a id="q21.533"></a>

### <a id="u.11.1.1"></a>C.8.39 [11.1.1 Overflow: the 'overflow' property](https://www.w3.org/TR/2010/WD-CSS2-20101207/visufx.html#overflow)

Shorten the “applies to” line:

> <strong>'overflow'</strong>  
> |                       |                                                                                                                               |
> |-----------------------|-------------------------------------------------------------------------------------------------------------------------------|
> | <em>Value:</em>   | visible \| hidden \| scroll \| auto \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
> | <em>Initial:</em>   | visible                                                                                                                       |
> | <em>Applies to:</em>   | ~~non-replaced block-level elements, table cells, inline-table, and inline-block elements~~ <u>block containers</u>           |
> | <em>Inherited:</em>   | no                                                                                                                            |
> | <em>Percentages:</em>   | N/A                                                                                                                           |
> | <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                          |
> | <em>Computed value:</em>   | as specified                                                                                                                  |

<a id="q21.534"></a>

### <a id="u.9.2.1.1c"></a>C.8.40 [9.2.1.1 Anonymous block boxes](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#anonymous-block-level)

Clarify how block-level elements inside inline elements are affected by relative positioning:

> When such an inline box is affected by relative positioning, ~~the relative positioning~~ <u>any resulting translation</u> also affects the block-level box contained in the inline box.

<a id="q21.535"></a>

### <a id="u.16.2"></a>C.8.41 [16.2 Alignment: the 'text-align' property](https://www.w3.org/TR/2010/WD-CSS2-20101207/text.html#alignment-prop)

Text is justified within the line box, which may be narrower than the block box:

> In the case of 'justify', this property specifies that the inline-level boxes are to be made flush with both sides of the ~~block container~~ <u>line box</u> if possible, \[…\]

<a id="q21.536"></a>

### <a id="u.9.5a"></a>C.8.42 [9.5 Floats](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#floats)

Only the current and later line boxes can be shortened by a float. Earlier line boxes, if the float ends up next to them, will overlap the float instead:

> Since a float is not in the flow, non-positioned block boxes created before and after the float box flow vertically as if the float did not exist. However, <u>the current and subsequent</u> line boxes created next to the float are shortened to make room for the margin box of the float.

<a id="q21.537"></a>

### <a id="u.9.4.2b"></a>C.8.43 [9.4.2 Inline formatting contexts](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#inline-formatting)

A float may cause a gap between line boxes:

> \[…\] Thus, a paragraph is a vertical stack of line boxes. Line boxes are stacked with no vertical separation <u>(except as specified elsewhere)</u> and they never overlap.

<a id="q21.538"></a>

### <a id="u.5.12"></a>C.8.44 [5.12 Pseudo-elements](https://www.w3.org/TR/2010/WD-CSS2-20101207/selector.html#pseudo-element-selectors)

Added a note to make it explicit that CSS 2.1 does not define ':first-line' and ':first-letter' completely:

> > <strong data-conversion-semantic="note">Note</strong>
> >
> > <u><em>Note that the sections below do not define
the exact rendering of ':first-line' and ':first-letter' in all
cases. A future level of CSS may define them more
precisely.</em></u>

<a id="q21.539"></a>

### <a id="u.9.5b"></a>C.8.45 [9.5 Floats](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#floats)

Clarify that “overlap a float” means overlap the <em>margin box</em> of the float:

> The border box of a table, a block-level replaced element, or an element in the normal flow that establishes a new [block formatting context](css2--visuren.html--3f334c530cf4.md#block-formatting) (such as an element with 'overflow' other than 'visible') must not overlap <u>the margin box of</u> any floats in the same block formatting context as the element itself.

<a id="q21.540"></a>

### <a id="u.9.5c"></a>C.8.46 [9.5 Floats](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#floats)

A line box next to a float is not shortened if it already doesn't overlap the float:

> \[…\] However, the current and subsequent line boxes created next to the float are shortened <u>as necessary</u> to make room for the margin box of the float.

<a id="q21.541"></a>

### <a id="u.14.2.1"></a>C.8.47 [14.2.1 Background properties: 'background-color', 'background-image', 'background-repeat', 'background-attachment', 'background-position', and 'background'](https://www.w3.org/TR/2010/WD-CSS2-20101207/colors.html#background-properties)

Because of insufficient implementations of background images with an intrinsic ratio but no intrinsic size, add this note:

> <u>However, the position is undefined in CSS 2.1 if the image has an intrinsic ratio, but no intrinsic size.</u>

<a id="q21.542"></a>

### <a id="u.9.2.4"></a>C.8.48 [9.2.4 The 'display' property](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#display-prop)

Because some aspects of 'run-in' (most notably if and how 'clear' should apply to run-in elements when they are inline) are still under discussion, 'run-in' has been reclassified as a level 3 feature.

Change in section 9.2.4:

> |                     |                                                                                                                                                                                                                                                                                                                                            |
> |---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
> | <em>Value:</em> | inline \| block \| list-item \| ~~run-in \|~~ inline-block \| table \| inline-table \| inline \| block \| list-item \| run-in \| inline-block \| table \| inline-table \| table-row-group \| table-header-group \| table-footer-group \| table-row \| table-column-group \| table-column \| table-cell \| table-caption \| none \| inherit |

and

> ~~run-in~~  
> ~~This value creates either block or inline boxes, depending on context. Properties apply to run-in boxes based on their final status (inline-level or block-level).~~

Remove 'run-in' from [section 9.2.1 “Block-level elements and block boxes”](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#block-boxes):

> \[…\] The following values of the 'display' property make an element block-level: 'block', 'list-item', ~~and 'run-in' (part of the time; see run-in boxes),~~ and 'table'.

Remove 'run-in' from [section 9.2.2 “Inline-level elements and inline boxes”](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#inline-boxes):

> \[…\] The following values of the 'display' property make an element inline-level: 'inline', 'inline-table', <u>and</u> 'inline-block' ~~and 'run-in' (part of the time; see run-in boxes)~~. \[…\]
>
> \[…\] A non-replaced element with a 'display' value of 'inline' generates an inline box. ~~An element with a 'display' value of 'run-in' can also generate an inline box; see run-in boxes.~~

Replace [section 9.2.3 “Run-in boxes”](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#run-in) by this:

> <strong>9.2.3 Run-in boxes</strong>
>
> \[This section exists so that the section numbers are the same as in previous drafts. 'Display: run-in' is now defined in CSS level 3 (see [CSS basic box model](https://www.w3.org/TR/css3-box)).\]

Remove 'run-in' from [section 9.3 “Positioning schemes”](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#positioning-scheme):

> 1.  Normal flow. In CSS 2.1, normal flow includes block formatting of block-level boxes, inline formatting of inline-level boxes, <u>and</u> relative positioning of block-level and inline-level boxes~~, and formatting of run-in boxes~~.

Remove 'run-in' from [section 9.5.2 “Controlling flow next to floats: the 'clear' property”](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#flow-control):

> ~~For run-in boxes, this property applies to the final block box to which the run-in box belongs.~~

Remove 'run-in' from [section 9.7 “Relationships between 'display', 'position', and 'float'”](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#dis-pos-flo):

> | Specified value                                                                                                                                                   | Computed value    |
> |-------------------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------|
> | inline-table                                                                                                                                                      | table             |
> | inline, ~~run-in,~~ table-row-group, table-column, table-column-group, table-header-group, table-footer-group, table-row, table-cell, table-caption, inline-block | block             |
> | others                                                                                                                                                            | same as specified |

Remove 'run-in' from [section 9.10 “Text direction: the 'direction' and 'unicode-bidi' properties”](https://www.w3.org/TR/2010/WD-CSS2-20101207/visuren.html#direction):

> The final order of characters in each block container is the same as if the bidi control codes had been added as described above, markup had been stripped, and the resulting character sequence had been passed to an implementation of the Unicode bidirectional algorithm for plain text that produced the same line-breaks as the styled text. In this process, replaced elements with 'display: inline' ~~(and replaced elements with 'display: run-in', when they generate inline-level boxes)~~ are treated as neutral characters, unless their 'unicode-bidi' property has a value other than 'normal', in which case they are treated as strong characters in the 'direction' specified for the element. All other atomic inline-level boxes are treated as neutral characters always.

Remove 'run-in' from [section E.1 “Definitions”](https://www.w3.org/TR/2010/WD-CSS2-20101207/zindex.html#stacking-defs):

> Tree Order  
> Preorder depth-first traversal of the rendering tree, in logical (not visual) order for bidirectional content, after taking into account properties that move boxes around ~~such as the 'run-in' value of 'display'~~.

Remove 'run-in' from [section 12.1 “The :before and :after pseudo-elements”](https://www.w3.org/TR/2010/WD-CSS2-20101207/generate.html#before-after-content):

> The :before and :after pseudo-elements interact with other boxes~~, such as run-in boxes,~~ as if they were real elements inserted just inside their associated element.

and also from the subsequent example.

<a id="q21.543"></a>

### <a id="u.6.1.2"></a>C.8.49 [6.1.2 Computed values](https://www.w3.org/TR/2010/WD-CSS2-20101207/cascade.html#computed-value)

Clarify that the keyword 'inherit' means that the specified value is the inherited value. The value is <em>not</em> the keyword itself.

> ~~When the specified value is not 'inherit',~~ the computed value of a property is determined as specified by the Computed Value line in the definition of the property. See the section on [inheritance](css2--cascade.html--c7aff33e6f0d.md#inheritance) for the definition of computed values when the specified value is 'inherit'.

And in [6.2.1:](https://www.w3.org/TR/2010/WD-CSS2-20101207/cascade.html#value-def-inherit)

> Each property may also have a ~~specified~~ <u>cascaded</u> value of 'inherit', which means that, for a given element, the property takes the same ~~computed~~ <u>specified</u> value as the property for the element's parent. The 'inherit' value can be used to ~~strengthen inherited~~ <u>enforce inheritance of</u> values, and it can also be used on properties that are not normally inherited.

<a id="q21.544"></a>

### <a id="u.10.3.2a"></a>C.8.50 [10.3.2 Inline, replaced elements](https://www.w3.org/TR/2010/WD-CSS2-20101207/visudet.html#inline-replaced-width)

Because of lack of implementations, the width of a replaced element with an intrinsic ratio but neither intrinsic with nor intrinsic height is left undefined:

> If 'height' and 'width' both have computed values of 'auto' and the element has an intrinsic ratio but no intrinsic height or width, ~~and~~ <u>then the used value of 'width' is undefined in CSS 2.1. However, it is suggested that, if</u> the containing block's width does not itself depend on the replaced element's width, then the used value of 'width' is calculated from the constraint equation used for block-level, non-replaced elements in normal flow.

<a id="q21.545"></a>

### <a id="t.9.5.2d"></a>C.8.51 [Section 9.5.2 Controlling flow next to floats: the 'clear' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#flow-control)

Because of lack of implementations, also allow 'clear' to work in a different way for now:

> Computing the clearance of an element on which 'clear' is set is done by first determining the hypothetical position of the element's top border edge ~~within its parent block~~. This position is where the actual top border edge would have been if the element had a non-zero bottom border and its 'clear' property had been 'none'.
>
> If this hypothetical position of the element's top border edge is not past the relevant floats, then clearance is introduced, and margins collapse according to the rules in 8.3.1.
>
> Then the amount of clearance is set to the greater of:
>
> 1.  The amount necessary to place the border edge of the block even with the bottom outer edge of the lowest float that is to be cleared.
> 2.  The amount necessary to place the top border edge of the block at its hypothetical position.
>
> <u>Alternatively, clearance is set exactly to the amount necessary to place the border edge of the block even with the bottom outer edge of the lowest float that is to be cleared.</u>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > <u><em><strong>Note:</strong> Both behaviors are
allowed pending evaluation of their compatibility with existing Web
content. A future CSS specification will require either one or the
other.</em></u>

<a id="q21.546"></a>

### <a id="t.G.2"></a>C.8.52 [G.2 Lexical scanner](https://www.w3.org/TR/2009/CR-CSS2-20090908/grammar.html#scanner)

The tokenizer in the appendix allowed backslashes in the URI token, in contradiction with the same token in the core grammar and the error recovery token {baduri}:

> ```text
> 
> {U}{R}{L}"("{w}{string}{w}")"      {return URI;}
> {U}{R}{L}"("{w}{url}{w}")"         {return URI;}
> "url("{w}{string}{w}")"            {return URI;}
> "url("{w}{url}{w}")"               {return URI;}
> ```
<a id="q21.547"></a>

### <a id="t.9.5.2e"></a>C.8.53 [Section 9.5.2 Controlling flow next to floats: the 'clear' property](https://www.w3.org/TR/2009/CR-CSS2-20090908/visuren.html#flow-control)

The top border edge is now well-defined in the section on collapsing margins. That is the hypothetical position to use for clearance:

> This position is where the actual top border edge would have been if the element ~~had a non-zero bottom border and its~~ <u>'s</u> 'clear' property had been 'none'.

<a id="q21.548"></a>

### <a id="t.9.5b"></a>C.8.54 [9.5 Floats](https://www.w3.org/TR/2009/CR-CSS2-20090908//visuren.html#floats)

Remove ambiguities:

> If a shortened line box is too small to contain any content ~~after the float~~, then ~~that content~~ <u>the line box</u> is shifted downward <u>(and its width recomputed)</u> until either ~~it~~ <u>some content</u> fits or there are no more floats present. Any content in the current line before a floated box is reflowed in the ~~first available~~ <u>same</u> line on the other side of the float.

<a id="q21.549"></a>

### <a id="t.10.6.3"></a>C.8.55 [10.6.3 Block-level non-replaced elements in normal flow when 'overflow' computes to 'visible'](https://www.w3.org/TR/2009/CR-CSS2-20090908//visudet.html#normal-block)

Removed redundancy (the top edge was already defined elsewhere) and made the implied cases for the bottom edge explicit:

> ~~If it only has inline-level children, the height is the distance between the top of the topmost line box and the bottom of the bottommost line box.~~
>
> ~~If it has block-level children, the height is the distance between the top border-edge of the topmost block-level child box that does not have margins collapsed through it and the bottom border-edge of the bottommost block-level child box that does not have margins collapsed through it. However, if the element has a non-zero top padding and/or top border, or is the root element, then the content starts at the top margin edge of the topmost child. (The first case expresses the fact that the top and bottom margins of the element collapse with those of the topmost and bottommost children, while in the second case the presence of the padding/border prevents the top margins from collapsing.) Similarly, if the bottom margin of the block does not collapse with the bottom margin of its last in-flow child, then the content ends at the bottom margin edge of the bottommost child.~~
>
> <u>The element's height is the distance from its top content edge to the first applicable of the following:</u>
>
> 1.  <u>the bottom edge of the last line box, if the box establishes a inline formatting context with one or more lines</u>
> 2.  <u>the bottom edge of the bottom (possibly collapsed) margin of its last in-flow child, if the child's bottom margin does not collapse with the element's bottom margin</u>
> 3.  <u>the bottom border edge of the last in-flow child whose top margin doesn't collapse with the element's bottom margin</u>
> 4.  <u>zero, otherwise</u>
