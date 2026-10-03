Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Conformance: requirements and recommendations](https://www.w3.org/TR/2011/REC-CSS2-20110607/conform.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Conformance: requirements and recommendations

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/conform.html

Snapshot SHA-256: de58593b67d7256eaa72fc33b956c6838cd0e9fa1670d0a0e5feb8079b4ce572

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="q3.0"></a>

# 3 Conformance: Requirements and Recommendations

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<a id="defs"></a>

## 3.1 Definitions

<a id="x0"></a>

<a id="x1"></a>

<a id="x2"></a>

<a id="x3"></a>

<a id="x4"></a>

<a id="x5"></a>

<a id="x6"></a>

<a id="x7"></a>

<a id="x8"></a>

<a id="x9"></a>

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 (see [\[RFC2119\]](css2--refs.html--f208d881d0b7.md#ref-RFC2119)). However, for readability, these words do not appear in all uppercase letters in this specification.

At times, this specification recommends good practice for authors and user agents. These recommendations are not normative and conformance with this specification does not depend on their realization. These recommendations contain the expression "We recommend ...", "This specification recommends ...", or some similar wording.

The fact that a feature is marked as deprecated (namely the ['aural'](css2--aural.html--2915ca92d45a.md#aural-media-group) keyword) or going to be deprecated in CSS3 (namely the [system colors](css2--ui.html--ff1f72803ea0.md#system-colors)) also has no influence on conformance. (For example, 'aural' is marked as non-normative, so UAs do not need to support it; the system colors are normative, so UAs must support them.)

All sections of this specification, including appendices, are normative unless otherwise noted.

[Examples and notes](css2--about.html--c67ff594c990.md#notes-and-examples) are not normative.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Examples usually have the word "example" near their start ("Example:", "The following example…," "For example," etc.) and are shown in the color maroon, like this paragraph.

> <strong data-conversion-semantic="note">Note</strong>
>
> Notes start with the word "Note," are indented and shown in green, like this paragraph.

Figures are for illustration only. They are not reference renderings, unless explicitly stated.

<a id="style-sheet"></a>

<strong><span title="style
sheet"><a>Style sheet</a></span></strong>

A set of statements that specify presentation of a document.

Style sheets may have three different origins: [author](#author), [user](#user), and [user agent](#user-agent). The interaction of these sources is described in the section on [cascading and inheritance](css2--cascade.html--c7aff33e6f0d.md).

<a id="valid-style-sheet"></a>

<strong><span title="valid style
sheet|validity"><a>Valid style
sheet</a></span></strong>

The validity of a style sheet depends on the level of CSS used for the style sheet. All valid CSS1 style sheets are valid CSS 2.1 style sheets, but some changes from CSS1 mean that a few CSS1 style sheets will have slightly different semantics in CSS 2.1. Some features in CSS2 are not part of CSS 2.1, so not all CSS2 style sheets are valid CSS 2.1 style sheets.

<a id="illegal"></a>

A valid CSS 2.1 style sheet must be written according to the [grammar of CSS 2.1](css2--grammar.html--eeb984c469da.md). Furthermore, it must contain only at-rules, property names, and property values defined in this specification. An <strong><a>illegal</a></strong> (invalid) at-rule, property name, or property value is one that is not valid.

<a id="source-document"></a>

<strong><span title="source
document"><a>Source
document</a></span></strong>

The document to which one or more style sheets apply. This is encoded in some language that represents the document as a tree of [elements](#element). Each element consists of a name that identifies the type of element, optionally a number of [attributes](#attribute), and a (possibly empty) [content](#content). For example, the source document could be an XML or SGML instance.

<a id="doclanguage"></a>

<strong><span title="document language"><a>Document language</a></span></strong>

The encoding language of the source document (e.g., HTML, XHTML, or SVG). CSS is used to describe the presentation of document languages and CSS does not change the underlying semantics of the document languages.

<a id="element"></a>

<strong><span title="element"><a>Element</a></span></strong>

(An SGML term, see [\[ISO8879\]](css2--refs.html--f208d881d0b7.md#ref-ISO8879).) The primary syntactic constructs of the document language. Most CSS style sheet rules use the names of these elements (such as P, TABLE, and OL in HTML) to specify how the elements should be rendered.

<a id="replaced-element"></a>

<strong><span title="replaced element">
<a>Replaced
element</a></span></strong>

An element whose content is outside the scope of the CSS formatting model, such as an image, embedded document, or applet. For example, the content of the HTML IMG element is often replaced by the image that its "src" attribute designates. Replaced elements often have intrinsic dimensions: an intrinsic width, an intrinsic height, and an intrinsic ratio. For example, a bitmap image has an intrinsic width and an intrinsic height specified in absolute units (from which the intrinsic ratio can obviously be determined). On the other hand, other documents may not have any intrinsic dimensions (for example, a blank HTML document).

User agents may consider a replaced element to not have any intrinsic dimensions if it is believed that those dimensions could leak sensitive information to a third party. For example, if an HTML document changed intrinsic size depending on the user's bank balance, then the UA might want to act as if that resource had no intrinsic dimensions.

The content of replaced elements is not considered in the CSS rendering model.

<a id="intrinsic"></a>

<strong><span title="intrinsic
dimensions"><a>Intrinsic dimensions</a></span></strong>

The width and height as defined by the element itself, not imposed by the surroundings. CSS does not define how the intrinsic dimensions are found. In CSS 2.1 only replaced elements can come with intrinsic dimensions. For raster images without reliable resolution information, a size of 1 px unit per image source pixel must be assumed.

<a id="attribute"></a>

<strong><span title="attribute"><a>Attribute</a></span></strong>

A value associated with an element, consisting of a name, and an associated (textual) value.

<a id="content"></a>

<strong><span title="content"><a>Content</a></span></strong>

<a id="parent"></a>

<a id="empty"></a>

The content associated with an element in the source document. Some elements have no content, in which case they are called <strong><span title="empty"><a>empty</a></span></strong>. The content of an element may include text, and it may include a number of sub-elements, in which case the element is called the <strong><span title="parent"><a>parent</a></span></strong> of those sub-elements.

<a id="ignore"></a>

<strong><span title="ignore"><a>Ignore</a></span></strong>

This term has two slightly different meanings in this specification. First, a CSS parser must follow certain rules when it discovers unknown or illegal syntax in a style sheet. The parser must then ignore certain parts of the style sheets. The exact rules for which parts must be ignored are described in these sections ([Declarations and properties,](css2--syndata.html--02e71c159e14.md#declaration) [Rules for handling parsing errors,](css2--syndata.html--02e71c159e14.md#parsing-errors) [Unsupported Values](css2--syndata.html--02e71c159e14.md#unsupported-values)) or may be explained in the text where the term "ignore" appears. Second, a user agent may (and, in some cases must) disregard certain properties or values in the style sheet, even if the syntax is legal. For example, table-column elements cannot affect the font of the column, so the font properties must be ignored.

<a id="rendered-content"></a>

<strong><span title="rendered
content|content::rendered"><a>Rendered
content</a></span></strong>

The content of an element after the rendering that applies to it according to the relevant style sheets has been applied. How a replaced element's content is rendered is not defined by this specification. Rendered content may also be alternate text for an element (e.g., the value of the XHTML "alt" attribute), and may include items inserted implicitly or explicitly by the style sheet, such as bullets, numbering, etc.

<a id="doctree"></a>

<strong><span title="document tree">
<a>Document
tree</a></span></strong>

<a id="root"></a>

The tree of elements encoded in the source document. Each element in this tree has exactly one parent, with the exception of the <strong><span title="root"><a>root</a></span></strong> element, which has none.

<a id="child"></a>

<strong><span title="child"><a>Child</a></span></strong>

An element A is called the child of element B if and only if B is the parent of A.

<a id="descendant"></a>

<strong><span title="descendant"><a>Descendant</a></span></strong>

An element A is called a descendant of an element B, if either (1) A is a child of B, or (2) A is the child of some element C that is a descendant of B.

<a id="ancestor"></a>

<strong><span title="ancestor"><a>Ancestor</a></span></strong>

An element A is called an ancestor of an element B, if and only if B is a descendant of A.

<a id="sibling"></a>

<strong><span title="sibling"><a>Sibling</a></span></strong>

An element A is called a sibling of an element B, if and only if B and A share the same parent element. Element A is a preceding sibling if it comes before B in the document tree. Element B is a following sibling if it comes after A in the document tree.

<a id="preceding"></a>

<strong><span title="preceding
element|element::preceding"><a>Preceding element</a></span></strong>

An element A is called a preceding element of an element B, if and only if (1) A is an ancestor of B or (2) A is a preceding sibling of B.

<a id="following"></a>

<strong><span title="following
element|element::following"><a>Following
element</a></span></strong>

An element A is called a following element of an element B, if and only if B is a preceding element of A.

<strong><span><dfn><span><a id="author"></a></span>Author</dfn></span></strong>

<a id="authoring"></a>

An author is a person who writes documents and associated style sheets. An <strong><span><a>authoring
tool</a></span></strong> is a [User Agent](#user-agent) that generates style sheets.

<strong><span><dfn><span><a id="user"></a></span>User</dfn></span></strong>

A user is a person who interacts with a user agent to view, hear, or otherwise use a document and its associated style sheet. The user may provide a personal style sheet that encodes personal preferences.

<strong><span><dfn><span><a id="user-agent"></a></span>User Agent (UA)</dfn></span></strong>

<a id="ua"></a>

A user agent is any program that interprets a document written in the document language and applies associated style sheets according to the terms of this specification. A user agent may display a document, read it aloud, cause it to be printed, convert it to another format, etc.

An HTML user agent is one that supports one or more of the HTML specifications. A user agent that supports XHTML [\[XHTML\]](css2--refs.html--f208d881d0b7.md#ref-XHTML), but not HTML is not considered an HTML user agent for the purpose of conformance with this specification.

<a id="property"></a>

<strong><span><a>Property</a></span></strong>

CSS defines a finite set of parameters, called properties, that direct the rendering of a document. Each property has a name (e.g., 'color', 'font', or border') and a value (e.g., 'red', '12pt Times', or 'dotted'). Properties are attached to various parts of the document and to the page on which the document is to be displayed by the mechanisms of specificity, cascading, and inheritance (see the chapter on [Assigning property values, Cascading, and Inheritance](css2--cascade.html--c7aff33e6f0d.md)).

Here is an example of a source document written in HTML:

```text

<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN">
<HTML>
  <TITLE>My home page</TITLE>
  <BODY>
    <H1>My home page</H1>
    <P>Welcome to my home page! Let me tell you about my favorite
		composers:
    <UL>
      <LI> Elvis Costello
      <LI> Johannes Brahms
      <LI> Georges Brassens
    </UL>
  </BODY>
</HTML>
```
This results in the following tree:

<a id="img-doctree"></a>

![Sample document tree](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/doctree.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/doctree-desc.html)

According to the definition of HTML 4, HEAD elements will be inferred during parsing and become part of the document tree even if the "head" tags are not in the document source. Similarly, the parser knows where the P and LI elements end, even though there are no \</p\> and \</li\> tags in the source.

Documents written in XHTML (and other XML-based languages) behave differently: there are no inferred elements and all elements must have end tags.

<a id="conformance"></a>

## 3.2 UA Conformance

<a id="conformance-term"></a>

This section defines conformance with the CSS 2.1 specification only. There may be other levels of CSS in the future that may require a user agent to implement a different set of features in order to conform.

In general, the following points must be observed by a user agent claiming conformance to this specification:

1.  It must recognize one or more of the CSS 2.1 [media types](css2--media.html--3a324a170379.md).

2.  For each source document, it must attempt to retrieve all associated style sheets that are appropriate for the recognized media types. If it cannot retrieve all associated style sheets (for instance, because of network errors), it must display the document using those it can retrieve.

3.  <a id="x45"></a>

    <a id="x44"></a>

    It must parse the style sheets according to this specification. In particular, it must recognize all at-rules, blocks, declarations, and selectors (see the [grammar of CSS 2.1](css2--grammar.html--eeb984c469da.md)). If a user agent encounters a property that applies for a supported media type, the user agent must parse the value according to the property definition. This means that the user agent must accept all valid values and must ignore declarations with invalid values. User agents must ignore rules that apply to unsupported [media types](css2--media.html--3a324a170379.md).

4.  For each element in a [document tree](#doctree), it must assign a value for every property according to the property's definition and the rules of [cascading and inheritance](css2--cascade.html--c7aff33e6f0d.md).

5.  If the source document comes with alternate style sheet sets (such as with the "alternate" keyword in HTML 4 [\[HTML4\]](css2--refs.html--f208d881d0b7.md#ref-HTML4)), the UA must allow the user to select which style sheet set the UA should apply.

6.  The UA must allow the user to turn off the influence of author style sheets.

Not every user agent must observe every point, however:

- An application that reads style sheets without rendering any content (e.g., a CSS 2.1 validator) must respect points 1-3.
- An authoring tool is only required to output [valid style sheets](#valid-style-sheet)
- A user agent that <em>renders</em> a document with associated style sheets must respect points 1-6 and render the document according to the media-specific requirements set forth in this specification. [Values](css2--cascade.html--c7aff33e6f0d.md#actual-value) may be approximated when required by the user agent.

The inability of a user agent to implement part of this specification due to the limitations of a particular device (e.g., a user agent cannot render colors on a monochrome monitor or page) does not imply non-conformance.

UAs must allow users to specify a file that contains the user style sheet. UAs that run on devices without any means of writing or specifying files are exempted from this requirement. Additionally, UAs may offer other means to specify user preferences, for example, through a GUI.

CSS 2.1 does not define which properties apply to form controls and frames, or how CSS can be used to style them. User agents may apply CSS properties to these elements. Authors are recommended to treat such support as experimental. A future level of CSS may specify this further.

<a id="errors"></a>

## 3.3 Error conditions

In general, this document specifies error handling behavior throughout the specification. For example, see the [rules for handling parsing errors](css2--syndata.html--02e71c159e14.md#parsing-errors).

<a id="text-css"></a>

## 3.4 The text/css content type

<a id="message-entity"></a>

CSS style sheets that exist in separate files are sent over the Internet as a sequence of bytes accompanied by encoding information. The structure of the transmission, termed a <strong>message
entity,</strong> is defined by RFC 2045 and RFC 2616 (see [\[RFC2045\]](css2--refs.html--f208d881d0b7.md#ref-RFC2045) and [\[RFC2616\]](css2--refs.html--f208d881d0b7.md#ref-RFC2616)). A message entity with a content type of "text/css" represents an independent CSS document. The "text/css" content type has been registered by RFC 2318 ([\[RFC2318\]](css2--refs.html--f208d881d0b7.md#ref-2318)).
