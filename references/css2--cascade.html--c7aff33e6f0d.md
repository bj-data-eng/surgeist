Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Assigning property values, Cascading, and Inheritance](https://www.w3.org/TR/2011/REC-CSS2-20110607/cascade.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Assigning property values, Cascading, and Inheritance

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/cascade.html

Snapshot SHA-256: c7aff33e6f0d4bbc75ffb20659afc3a413076db156259f7721be54f56184f148

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.

---

<a id="q6.0"></a>

# 6 Assigning property values, Cascading, and Inheritance

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<a id="value-stages"></a>

## 6.1 Specified, computed, and actual values

Once a user agent has parsed a document and constructed a [document tree](css2--conform.html--de58593b67d7.md#doctree), it must assign, for every element in the tree, a value to every property that applies to the target [media type](css2--media.html--3a324a170379.md).

The final value of a property is the result of a four-step calculation: the value is determined through specification (the "specified value"), then resolved into a value that is used for inheritance (the "computed value"), then converted into an absolute value if necessary (the "used value"), and finally transformed according to the limitations of the local environment (the "actual value").

<a id="specified-value"></a>

### 6.1.1  Specified values

User agents must first assign a specified value to each property based on the following mechanisms (in order of precedence):

1.  If the [cascade](#cascade) results in a value, use it.

2.  Otherwise, if the property is [inherited](#inheritance) and the element is not the root of the document tree, use the computed value of the parent element.

3.  <a id="x1"></a>

    Otherwise use the property's initial value. The initial value of each property is indicated in the property's definition.

<a id="computed-value"></a>

### 6.1.2  Computed values

Specified values are resolved to computed values during the cascade; for example URIs are made absolute and 'em' and 'ex' units are computed to pixel or absolute lengths. Computing a value never requires the user agent to render the document.

The computed value of URIs that the UA cannot resolve to absolute URIs is the specified value.

The computed value of a property is determined as specified by the Computed Value line in the definition of the property. See the section on [inheritance](#inheritance) for the definition of computed values when the specified value is 'inherit'.

The computed value exists even when the property does not apply, as defined by the ['Applies To'](css2--about.html--c67ff594c990.md#applies-to) line. However, some properties may define the computed value of a property for an element to depend on whether the property applies to that element.

<a id="used-value"></a>

### 6.1.3  Used values

Computed values are processed as far as possible without formatting the document. Some values, however, can only be determined when the document is being laid out. For example, if the width of an element is set to be a certain percentage of its containing block, the width cannot be determined until the width of the containing block has been determined. The <a id="usedValue"></a>used value is the result of taking the computed value and resolving any remaining dependencies into an absolute value.

<a id="actual-value"></a>

### 6.1.4  Actual values

A used value is in principle the value used for rendering, but a user agent may not be able to make use of the value in a given environment. For example, a user agent may only be able to render borders with integer pixel widths and may therefore have to approximate the computed width, or the user agent may be forced to use only black and white shades instead of full color. The actual value is the used value after any approximations have been applied.

<a id="inheritance"></a>

## 6.2 Inheritance

Some values are inherited by the children of an element in the [document tree](css2--conform.html--de58593b67d7.md#doctree), as described [above](#specified-value). Each property [defines](css2--about.html--c67ff594c990.md#property-defs) whether it is inherited or not.

Suppose there is an H1 element with an emphasizing element (EM) inside:

```text

<H1>The headline <EM>is</EM> important!</H1>
```
If no color has been assigned to the EM element, the emphasized "is" will inherit the color of the parent element, so if H1 has the color blue, the EM element will likewise be in blue.

When inheritance occurs, elements inherit computed values. The computed value from the parent element becomes both the specified value and the computed value on the child.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> For example, given the following style sheet:
>
> ```text
> 
> body { font-size: 10pt }
> h1 { font-size: 130% }
> ```
>
> and this document fragment:
>
> ```text
> 
> <BODY>
>   <H1>A <EM>large</EM> heading</H1>
> </BODY>
> ```
>
> the 'font-size' property for the H1 element will have the computed value '13pt' (130% times 10pt, the parent's value). Since the computed value of ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size) is inherited, the EM element will have the computed value '13pt' as well. If the user agent does not have the 13pt font available, the actual value of ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size) for both H1 and EM might be, for example, '12pt'.

<a id="x5"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that inheritance follows the document tree and is not intercepted by [anonymous boxes.](css2--visuren.html--3f334c530cf4.md#box-gen)

<a id="value-def-inherit"></a>

### 6.2.1 The 'inherit' value

Each property may also have a cascaded value of 'inherit', which means that, for a given element, the property takes the same specified value as the property for the element's parent. The 'inherit' value can be used to enforce inheritance of values, and it can also be used on properties that are not normally inherited.

If the 'inherit' value is set on the root element, the property is assigned its initial value.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In the example below, the ['color'](css2--colors.html--5784063d2778.md#propdef-color) and ['background'](css2--colors.html--5784063d2778.md#propdef-background) properties are set on the BODY element. On all other elements, the 'color' value will be inherited and the background will be transparent. If these rules are part of the user's style sheet, black text on a white background will be enforced throughout the document.
>
> ```text
> 
> body {
>   color: black !important; 
>   background: white !important;
> }
> 
> * { 
>   color: inherit !important; 
>   background: transparent !important;
> }
> ```
<a id="at-import"></a>

## 6.3 The @import rule

<a id="x7"></a>

The '@import' rule allows users to import style rules from other style sheets. In CSS 2.1, any @import rules must precede all other rules (except the @charset rule, if present). See the [section on parsing](css2--syndata.html--02e71c159e14.md#at-rules) for when user agents must ignore @import rules. The '@import' keyword must be followed by the URI of the style sheet to include. A string is also allowed; it will be interpreted as if it had url(...) around it.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The following lines are equivalent in meaning and illustrate both '@import' syntaxes (one with "url()" and one with a bare string):
>
> ```text
> 
> @import "mystyle.css";
> @import url("mystyle.css");
> ```
<a id="x8"></a>

<a id="x9"></a>

So that user agents can avoid retrieving resources for unsupported [media types](css2--media.html--3a324a170379.md), authors may specify media-dependent @import rules. These conditional imports specify comma-separated media types after the URI.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The following rules illustrate how @import rules can be made media-dependent:
>
> ```text
> 
> @import url("fineprint.css") print;
> @import url("bluish.css") projection, tv;
> ```
In the absence of any media types, the import is unconditional. Specifying 'all' for the medium has the same effect. The import only takes effect if the target medium matches the media list.

A target medium matches a media list if one of the items in the media list is the target medium or 'all'.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that Media Queries [\[MEDIAQ\]](css2--refs.html--f208d881d0b7.md#ref-MEDIAQ) extends the syntax of media lists and the definition of matching.

When the same style sheet is imported or linked to a document in multiple places, user agents must process (or act as though they do) each link as though the link were to a separate style sheet.

<a id="cascade"></a>

## 6.4 The cascade

Style sheets may have three different origins: author, user, and user agent.

- <strong>Author</strong>. The author specifies style sheets for a source document according to the conventions of the document language. For instance, in HTML, style sheets may be included in the document or linked externally.

- <strong>User</strong>: The user may be able to specify style information for a particular document. For example, the user may specify a file that contains a style sheet or the user agent may provide an interface that generates a user style sheet (or behaves as if it did).

- <a id="default-style-sheet"></a>

  <strong>User agent</strong>: [Conforming user agents](css2--conform.html--de58593b67d7.md#conformance) must apply a default style sheet (or behave as if they did). A user agent's default style sheet should present the elements of the document language in ways that satisfy general presentation expectations for the document language (e.g., for visual browsers, the EM element in HTML is presented using an italic font). See [A sample style sheet for HTML](css2--sample.html--133769fd2555.md) for a recommended default style sheet for HTML documents.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note that the user may modify system settings (e.g., system colors) that affect the default style sheet. However, some user agent implementations make it impossible to change the values in the default style sheet.

Style sheets from these three origins will overlap in scope, and they interact according to the cascade.

<a id="x12"></a>

The CSS cascade assigns a weight to each style rule. When several rules apply, the one with the greatest weight takes precedence.

By default, rules in author style sheets have more weight than rules in user style sheets. Precedence is reversed, however, for "!important" rules. All user and author rules have more weight than rules in the UA's default style sheet.

<a id="cascading-order"></a>

### 6.4.1 Cascading order

To find the value for an element/property combination, user agents must apply the following sorting order:

1.  Find all declarations that apply to the element and property in question, for the target [media type](css2--media.html--3a324a170379.md). Declarations apply if the associated selector [matches](css2--selector.html--f66f7c932788.md) the element in question and the target medium matches the media list on all @media rules containing the declaration and on all links on the path through which the style sheet was reached.
2.  Sort according to importance (normal or important) and origin (author, user, or user agent). In ascending order of precedence:
    1.  user agent declarations
    2.  user normal declarations
    3.  author normal declarations
    4.  author important declarations
    5.  user important declarations
3.  Sort rules with the same importance and origin by [specificity](#specificity) of selector: more specific selectors will override more general ones. Pseudo-elements and pseudo-classes are counted as normal elements and classes, respectively.
4.  Finally, sort by order specified: if two declarations have the same weight, origin and specificity, the latter specified wins. Declarations in imported style sheets are considered to be before any declarations in the style sheet itself.

Apart from the "!important" setting on individual declarations, this strategy gives author's style sheets higher weight than those of the reader. User agents must give the user the ability to turn off the influence of specific author style sheets, e.g., through a pull-down menu. Conformance to UAAG 1.0 checkpoint 4.14 satisfies this condition [\[UAAG10\]](css2--refs.html--f208d881d0b7.md#ref-UAAG10).

<a id="important-rules"></a>

### 6.4.2 !important rules

CSS attempts to create a balance of power between author and user style sheets. By default, rules in an author's style sheet override those in a user's style sheet (see cascade rule 3).

However, for balance, an "!important" declaration (the delimiter token "!" and keyword "important" follow the declaration) takes precedence over a normal declaration. Both author and user style sheets may contain "!important" declarations, and user "!important" rules override author "!important" rules. This CSS feature improves accessibility of documents by giving users with special requirements (large fonts, color combinations, etc.) control over presentation.

<a id="x13"></a>

Declaring a shorthand property (e.g., ['background'](css2--colors.html--5784063d2778.md#propdef-background)) to be "!important" is equivalent to declaring all of its sub-properties to be "!important".

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The first rule in the user's style sheet in the following example contains an "!important" declaration, which overrides the corresponding declaration in the author's style sheet. The second declaration will also win due to being marked "!important". However, the third rule in the user's style sheet is not "!important" and will therefore lose to the second rule in the author's style sheet (which happens to set style on a shorthand property). Also, the third author rule will lose to the second author rule since the second rule is "!important". This shows that "!important" declarations have a function also within author style sheets.
>
> ```text
> 
> /* From the user's style sheet */
> p { text-indent: 1em ! important }
> p { font-style: italic ! important }
> p { font-size: 18pt }
> 
> /* From the author's style sheet */
> p { text-indent: 1.5em !important }
> p { font: normal 12pt sans-serif !important }
> p { font-size: 24pt }
> ```
<a id="specificity"></a>

### 6.4.3 Calculating a selector's specificity

A selector's specificity is calculated as follows:

- count 1 if the declaration is from is a 'style' attribute rather than a rule with a selector, 0 otherwise (= a) (In HTML, values of an element's "style" attribute are style sheet rules. These rules have no selectors, so a=1, b=0, c=0, and d=0.)
- count the number of ID attributes in the selector (= b)
- count the number of other attributes and pseudo-classes in the selector (= c)
- count the number of element names and pseudo-elements in the selector (= d)

The specificity is based only on the form of the selector. In particular, a selector of the form "\[id=p33\]" is counted as an attribute selector (a=0, b=0, c=1, d=0), even if the id attribute is defined as an "ID" in the source document's DTD.

Concatenating the four numbers a-b-c-d (in a number system with a large base) gives the specificity.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Some examples:
>
> ```text
> 
>  *             {}  /* a=0 b=0 c=0 d=0 -> specificity = 0,0,0,0 */
>  li            {}  /* a=0 b=0 c=0 d=1 -> specificity = 0,0,0,1 */
>  li:first-line {}  /* a=0 b=0 c=0 d=2 -> specificity = 0,0,0,2 */
>  ul li         {}  /* a=0 b=0 c=0 d=2 -> specificity = 0,0,0,2 */
>  ul ol+li      {}  /* a=0 b=0 c=0 d=3 -> specificity = 0,0,0,3 */
>  h1 + *[rel=up]{}  /* a=0 b=0 c=1 d=1 -> specificity = 0,0,1,1 */
>  ul ol li.red  {}  /* a=0 b=0 c=1 d=3 -> specificity = 0,0,1,3 */
>  li.red.level  {}  /* a=0 b=0 c=2 d=1 -> specificity = 0,0,2,1 */
>  #x34y         {}  /* a=0 b=1 c=0 d=0 -> specificity = 0,1,0,0 */
>  style=""          /* a=1 b=0 c=0 d=0 -> specificity = 1,0,0,0 */
> ```
```text

<HEAD>
<STYLE type="text/css">
  #x97z { color: red }
</STYLE>
</HEAD>
<BODY>
<P ID=x97z style="color: green">
</BODY>
```
In the above example, the color of the P element would be green. The declaration in the "style" attribute will override the one in the STYLE element because of cascading rule 3, since it has a higher specificity.

<a id="preshint"></a>

### 6.4.4 Precedence of non-CSS presentational hints

The UA may choose to honor presentational attributes in an HTML source document. If so, these attributes are translated to the corresponding CSS rules with specificity equal to 0, and are treated as if they were inserted at the start of the author style sheet. They may therefore be overridden by subsequent style sheet rules. In a transition phase, this policy will make it easier for stylistic attributes to coexist with style sheets.

For HTML, any attribute that is not in the following list should be considered presentational: abbr, accept-charset, accept, accesskey, action, alt, archive, axis, charset, checked, cite, class, classid, code, codebase, codetype, colspan, coords, data, datetime, declare, defer, dir, disabled, enctype, for, headers, href, hreflang, http-equiv, id, ismap, label, lang, language, longdesc, maxlength, media, method, multiple, name, nohref, object, onblur, onchange, onclick, ondblclick, onfocus, onkeydown, onkeypress, onkeyup, onload, onload, onmousedown, onmousemove, onmouseout, onmouseover, onmouseup, onreset, onselect, onsubmit, onunload, onunload, profile, prompt, readonly, rel, rev, rowspan, scheme, scope, selected, shape, span, src, standby, start, style, summary, title, type (except on LI, OL and UL elements), usemap, value, valuetype, version.

For other languages, all document language-based styling must be translated to the corresponding CSS and either enter the cascade at the user agent level or, as with HTML presentational hints, be treated as author level rules with a specificity of zero placed at the start of the author style sheet.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The following user style sheet would override the font weight of 'b' elements in all documents, and the color of 'font' elements with color attributes in XML documents. It would not affect the color of any 'font' elements with color attributes in HTML documents:
>
> ```text
> 
> b { font-weight: normal; }
> font[color] { color: orange; }
> ```
>
> The following, however, would override the color of font elements in all documents:
>
> ```text
> 
> font[color] { color: orange ! important; }
> ```