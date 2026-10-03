Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Font Loading Module Level 3](https://www.w3.org/TR/2023/WD-css-font-loading-3-20230406/).

Original copyright notice: Copyright © 2023 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Font Loading Module Level 3

Source snapshot: https://www.w3.org/TR/2023/WD-css-font-loading-3-20230406/

Snapshot SHA-256: 1a443ea0a7f67bab99daac264808df5268d652f0abbd63989c2a31b1d489a340

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 1 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Font Loading Module Level 3

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2023 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This CSS module describes events and interfaces used for dynamically loading font resources.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-font-loading” in the title, like this: “\[css-font-loading\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-font-loading%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="introduction"></a>1.  Introduction

<a id="ref-for-at-font-face-rule"></a>

CSS allows authors to load custom fonts from the web via the [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule. While this is easy to use when authoring a stylesheet, it’s much more difficult to use dynamically via scripting.

Further, CSS allows the user agent to choose when to actually load a font; if a font face isn’t <em>currently</em> used by anything on a page, most user agents will not download its associated file. This means that later use of the font face will incur a delay as the user agent finally notices a usage and begins downloading and parsing the font file.

<a id="ref-for-fontface"></a>

This specification defines a scripting interface to font faces in CSS, allowing font faces to be easily created (via the <code><a href="#fontface">FontFace</a></code> interface) and loaded from script (via [document.fonts](#font-face-source)). It also provides methods to track the loading status of an individual font, or of all the fonts on an entire page.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9cdca628"></a> Several things in this spec use normal ES objects to define behavior, such as various things using Promises internally, and FontFaceSet using a Set internally. I believe the intention here is that these objects (and their prototype chains) are pristine, unaffected by anything the author has done. Is this a good intention? If so, how should I indicate this in the spec?

### <a id="values"></a>1.1.  Values

<a id="ref-for-idl-promise"></a>

This specification uses <code><a href="https://webidl.spec.whatwg.org/#idl-promise">Promise</a></code>s, which are defined in [ECMAScript 6](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise-objects). MDN has some [good tutorial material introducing Promises](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise).

### <a id="task-source"></a>1.2.  Task Sources

Whenever this specification queues a task, it queues it onto the "font loading" task source.

## <a id="fontface-interface"></a>2.  The `FontFace` Interface

<a id="ref-for-fontface①"></a>

<a id="ref-for-at-font-face-rule①"></a>

The <code><a href="#fontface">FontFace</a></code> interface represents a single usable font face. CSS [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rules implicitly define FontFace objects, or they can be constructed manually from a url or binary data.

<a id="ref-for-idl-ArrayBuffer"></a>

<a id="ref-for-ArrayBufferView"></a>

<a id="typedefdef-binarydata"></a>

<a id="dictdef-fontfacedescriptors"></a>

<a id="ref-for-cssomstring"></a>

<a id="dom-fontfacedescriptors-style"></a>

<a id="ref-for-cssomstring①"></a>

<a id="dom-fontfacedescriptors-weight"></a>

<a id="ref-for-cssomstring②"></a>

<a id="dom-fontfacedescriptors-stretch"></a>

<a id="ref-for-cssomstring③"></a>

<a id="dom-fontfacedescriptors-unicoderange"></a>

<a id="ref-for-cssomstring④"></a>

<a id="dom-fontfacedescriptors-variant"></a>

<a id="ref-for-cssomstring⑤"></a>

<a id="dom-fontfacedescriptors-featuresettings"></a>

<a id="ref-for-cssomstring⑥"></a>

<a id="dom-fontfacedescriptors-variationsettings"></a>

<a id="ref-for-cssomstring⑦"></a>

<a id="dom-fontfacedescriptors-display"></a>

<a id="ref-for-cssomstring⑧"></a>

<a id="dom-fontfacedescriptors-ascentoverride"></a>

<a id="ref-for-cssomstring⑨"></a>

<a id="dom-fontfacedescriptors-descentoverride"></a>

<a id="ref-for-cssomstring①⓪"></a>

<a id="dom-fontfacedescriptors-linegapoverride"></a>

<a id="enumdef-fontfaceloadstatus"></a>

<a id="dom-fontfaceloadstatus-unloaded"></a>

<a id="dom-fontfaceloadstatus-loading"></a>

<a id="dom-fontfaceloadstatus-loaded"></a>

<a id="dom-fontfaceloadstatus-error"></a>

<a id="ref-for-Exposed"></a>

<a id="fontface"></a>

<a id="ref-for-dom-fontface-fontface"></a>

<a id="ref-for-cssomstring①①"></a>

<a id="dom-fontface-fontface-family-source-descriptors-family"></a>

<a id="ref-for-cssomstring①②"></a>

<a id="ref-for-typedefdef-binarydata"></a>

<a id="dom-fontface-fontface-family-source-descriptors-source"></a>

<a id="ref-for-dictdef-fontfacedescriptors"></a>

<a id="dom-fontface-fontface-family-source-descriptors-descriptors"></a>

<a id="ref-for-cssomstring①③"></a>

<a id="ref-for-dom-fontface-family"></a>

<a id="ref-for-cssomstring①④"></a>

<a id="ref-for-dom-fontface-style"></a>

<a id="ref-for-cssomstring①⑤"></a>

<a id="ref-for-dom-fontface-weight"></a>

<a id="ref-for-cssomstring①⑥"></a>

<a id="ref-for-dom-fontface-stretch"></a>

<a id="ref-for-cssomstring①⑦"></a>

<a id="ref-for-dom-fontface-unicoderange"></a>

<a id="ref-for-cssomstring①⑧"></a>

<a id="ref-for-dom-fontface-variant"></a>

<a id="ref-for-cssomstring①⑨"></a>

<a id="ref-for-dom-fontface-featuresettings"></a>

<a id="ref-for-cssomstring②⓪"></a>

<a id="ref-for-dom-fontface-variationsettings"></a>

<a id="ref-for-cssomstring②①"></a>

<a id="ref-for-dom-fontface-display"></a>

<a id="ref-for-cssomstring②②"></a>

<a id="ref-for-dom-fontface-ascentoverride"></a>

<a id="ref-for-cssomstring②③"></a>

<a id="ref-for-dom-fontface-descentoverride"></a>

<a id="ref-for-cssomstring②④"></a>

<a id="ref-for-dom-fontface-linegapoverride"></a>

<a id="ref-for-enumdef-fontfaceloadstatus"></a>

<a id="ref-for-dom-fontface-status"></a>

<a id="ref-for-idl-promise①"></a>

<a id="ref-for-fontface②"></a>

<a id="ref-for-dom-fontface-load"></a>

<a id="ref-for-idl-promise②"></a>

<a id="ref-for-fontface③"></a>

<a id="ref-for-dom-fontface-loaded"></a>

```text
typedef (ArrayBuffer or ArrayBufferView) BinaryData;

dictionary FontFaceDescriptors {
  CSSOMString style = "normal";
  CSSOMString weight = "normal";
  CSSOMString stretch = "normal";
  CSSOMString unicodeRange = "U+0-10FFFF";
  CSSOMString variant = "normal";
  CSSOMString featureSettings = "normal";
  CSSOMString variationSettings = "normal";
  CSSOMString display = "auto";
  CSSOMString ascentOverride = "normal";
  CSSOMString descentOverride = "normal";
  CSSOMString lineGapOverride = "normal";
};

enum FontFaceLoadStatus { "unloaded", "loading", "loaded", "error" };

[Exposed=(Window,Worker)]
interface FontFace {
  constructor(CSSOMString family, (CSSOMString or BinaryData) source,
                optional FontFaceDescriptors descriptors = {});
  attribute CSSOMString family;
  attribute CSSOMString style;
  attribute CSSOMString weight;
  attribute CSSOMString stretch;
  attribute CSSOMString unicodeRange;
  attribute CSSOMString variant;
  attribute CSSOMString featureSettings;
  attribute CSSOMString variationSettings;
  attribute CSSOMString display;
  attribute CSSOMString ascentOverride;
  attribute CSSOMString descentOverride;
  attribute CSSOMString lineGapOverride;

  readonly attribute FontFaceLoadStatus status;

  Promise<FontFace> load();
  readonly attribute Promise<FontFace> loaded;
};
```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9ae72be4"></a> Clarify all mentions of "the document" to be clear about which document is being referenced, since objects can move between documents.

<a id="ref-for-cssomstring②⑤"></a>

<a id="dom-fontface-family"></a>`family`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-cssomstring②⑥"></a>

<a id="dom-fontface-style"></a>`style`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-cssomstring②⑦"></a>

<a id="dom-fontface-weight"></a>`weight`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-cssomstring②⑧"></a>

<a id="dom-fontface-stretch"></a>`stretch`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-cssomstring②⑨"></a>

<a id="dom-fontface-unicoderange"></a>`unicodeRange`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-at-font-face-rule②"></a>

These attributes all represent the corresponding aspects of a font face, as defined by the descriptors defined in the CSS [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule. They are parsed the same as the corresponding <a id="ref-for-at-font-face-rule③"></a>@font-face descriptors. They are used by the font matching algorithm, but otherwise have no effect.

<a id="ref-for-fontface④"></a>

<a id="ref-for-dom-fontface-style①"></a>

For example, a <code><a href="#fontface">FontFace</a></code> with a <code><a href="#dom-fontface-style">style</a></code> of `"italic"` <em>represents</em> an italic font face; it does not <strong>make</strong> the font face italic.

On getting, return the string associated with this attribute.

<a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>

<a id="ref-for-at-font-face-rule④"></a>

<a id="ref-for-syntaxerror"></a>

On setting, [parse](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) the string according to the grammar for the corresponding [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) descriptor. If it does not match the grammar, throw a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code>; otherwise, set the attribute to the serialization of the parsed value.

<a id="ref-for-cssomstring③⓪"></a>

<a id="dom-fontface-variant"></a>`variant`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-cssomstring③①"></a>

<a id="dom-fontface-featuresettings"></a>`featureSettings`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-cssomstring③②"></a>

<a id="dom-fontface-variationsettings"></a>`variationSettings`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-cssomstring③③"></a>

<a id="dom-fontface-display"></a>`display`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-cssomstring③④"></a>

<a id="dom-fontface-ascentoverride"></a>`ascentOverride`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-cssomstring③⑤"></a>

<a id="dom-fontface-descentoverride"></a>`descentOverride`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-cssomstring③⑥"></a>

<a id="dom-fontface-linegapoverride"></a>`lineGapOverride`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-at-font-face-rule⑤"></a>

These attributes have the same meaning, and are parsed the same as, the corresponding descriptors in the CSS [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rules.

They turn on or off specific features in fonts that support them. Unlike the previous attributes, these attributes actually affect the font face.

On getting, return the string associated with this attribute.

<a id="ref-for-css-parse-something-according-to-a-css-grammar①"></a>

<a id="ref-for-at-font-face-rule⑥"></a>

<a id="ref-for-syntaxerror①"></a>

On setting, [parse](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) the string according to the grammar for the corresponding [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) descriptor. If it does not match the grammar, throw a <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code>; otherwise, set the attribute to the serialization of the parsed value.

<a id="ref-for-enumdef-fontfaceloadstatus①"></a>

<a id="dom-fontface-status"></a>`status`, of type [FontFaceLoadStatus](#enumdef-fontfaceloadstatus), readonly

<a id="ref-for-fontface⑤"></a>

This attribute reflects the current status of the font face. It must be "unloaded" for a newly-created <code><a href="#fontface">FontFace</a></code>.

<a id="ref-for-dom-fontface-load①"></a>

<a id="ref-for-fontface⑥"></a>

It can change due to an author explicitly requesting a font face to load, such as through the <code><a href="#dom-fontface-load">load()</a></code> method on <code><a href="#fontface">FontFace</a></code>, or implicitly by the user agent, due to it detecting that the font face is needed to draw some text on the screen.

<a id="ref-for-fontface⑦"></a>

<a id="dom-fontface-loaded"></a>`loaded`, of type Promise\<[FontFace](#fontface)\>, readonly

<a id="ref-for-dom-fontface-fontstatuspromise-slot"></a>

This attribute reflects the <code><a href="#dom-fontface-fontstatuspromise-slot">&#x5B;&#x5B;FontStatusPromise&#x5D;&#x5D;</a></code> of the font face.

<a id="ref-for-fontface⑧"></a>

All <code><a href="#fontface">FontFace</a></code> objects contain an internal <a id="dom-fontface-fontstatuspromise-slot"></a>`[[FontStatusPromise]]` slot, which tracks the status of the font. It starts out pending, and fulfills or rejects when the font is successfully loaded and parsed, or hits an error.

<a id="ref-for-fontface⑨"></a>

All <code><a href="#fontface">FontFace</a></code> objects also contain internal <a id="dom-fontface-urls-slot"></a>`[[Urls]]` and <a id="dom-fontface-data-slot"></a>`[[Data]]` slots, of which one is `null` and the other is not `null` (the non-null one is set by the constructor, based on which data is passed in).

### <a id="font-face-constructor"></a>2.1.  The Constructor

<a id="ref-for-fontface①⓪"></a>

<a id="ref-for-idl-ArrayBuffer①"></a>

<a id="ref-for-ArrayBufferView①"></a>

A <code><a href="#fontface">FontFace</a></code> can be constructed either from a URL pointing to a font face file, or from an <code><a href="https://webidl.spec.whatwg.org/#idl-ArrayBuffer">ArrayBuffer</a></code> (or <code><a href="https://webidl.spec.whatwg.org/#ArrayBufferView">ArrayBufferView</a></code>) containing the binary representation of a font face.

When the <a id="dom-fontface-fontface"></a>`FontFace(family, source, descriptors)` method is called, execute these steps:

1.  <a id="ref-for-fontface①①"></a>

    <a id="ref-for-dom-fontface-status①"></a>

    <a id="ref-for-dom-fontface-fontstatuspromise-slot①"></a>

    <a id="ref-for-idl-promise③"></a>

    Let <var>font face</var> be a fresh <code><a href="#fontface">FontFace</a></code> object. Set <var>font face’s</var> <code><a href="#dom-fontface-status">status</a></code> attribute to `"unloaded"`, Set its internal <code><a href="#dom-fontface-fontstatuspromise-slot">&#x5B;&#x5B;FontStatusPromise&#x5D;&#x5D;</a></code> slot to a fresh pending <code><a href="https://webidl.spec.whatwg.org/#idl-promise">Promise</a></code> object.

    <a id="ref-for-css-parse-something-according-to-a-css-grammar②"></a>

    <a id="ref-for-dom-fontface-fontface-family-source-descriptors-family"></a>

    <a id="ref-for-dom-fontface-fontface-family-source-descriptors-descriptors"></a>

    <a id="ref-for-at-font-face-rule⑦"></a>

    <a id="ref-for-dom-fontface-fontface-family-source-descriptors-source"></a>

    <a id="ref-for-cssomstring③⑦"></a>

    <a id="ref-for-dom-fontface-fontstatuspromise-slot②"></a>

    <a id="ref-for-dom-fontface-status②"></a>

    [Parse](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) the <code><a href="#dom-fontface-fontface-family-source-descriptors-family">family</a></code> argument, and the members of the <code><a href="#dom-fontface-fontface-family-source-descriptors-descriptors">descriptors</a></code> argument, according to the grammars of the corresponding descriptors of the CSS [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule. If the <code><a href="#dom-fontface-fontface-family-source-descriptors-source">source</a></code> argument is a <code><a href="https://www.w3.org/TR/cssom-1/#cssomstring">CSSOMString</a></code>, parse it according to the grammar of the CSS src descriptor of the <a id="ref-for-at-font-face-rule⑧"></a>@font-face rule. If any of them fail to parse correctly, reject <var>font face’s</var> <code><a href="#dom-fontface-fontstatuspromise-slot">&#x5B;&#x5B;FontStatusPromise&#x5D;&#x5D;</a></code> with a DOMException named "SyntaxError", set <var>font face’s</var> corresponding attributes to the empty string, and set <var>font face’s</var> <code><a href="#dom-fontface-status">status</a></code> attribute to "error". Otherwise, set <var>font face’s</var> corresponding attributes to the serialization of the parsed values.

    <a id="ref-for-funcdef-url"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Note that this means that passing a naked url as the source argument, like `"http://example.com/myFont.woff"`, won’t work - it needs to be at least wrapped in a [url()](https://www.w3.org/TR/css-values-4/#funcdef-url) function, like `"url(http://example.com/myFont.woff)"`. In return for this inconvenience, you get to specify multiple fallbacks, specify the type of font each fallback is, and refer to local fonts easily.

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-65bdbf2d"></a> Need to define the base url, so relative urls can resolve. Should it be the url of the document? Is that correct for workers too, or should they use their worker url? Is that always defined?

    <a id="ref-for-dom-fontface-status③"></a>

    Return <var>font face</var>. If <var>font face’s</var> <code><a href="#dom-fontface-status">status</a></code> is "error", terminate this algorithm; otherwise, complete the rest of these steps asynchronously.

2.  <a id="ref-for-dom-fontface-fontface-family-source-descriptors-source①"></a>

    <a id="ref-for-cssomstring③⑧"></a>

    <a id="ref-for-dom-fontface-urls-slot"></a>

    If the <code><a href="#dom-fontface-fontface-family-source-descriptors-source">source</a></code> argument was a <code><a href="https://www.w3.org/TR/cssom-1/#cssomstring">CSSOMString</a></code>, set <var>font face’s</var> internal <code><a href="#dom-fontface-urls-slot">&#x5B;&#x5B;Urls&#x5D;&#x5D;</a></code> slot to the string.

    <a id="ref-for-dom-fontface-fontface-family-source-descriptors-source②"></a>

    <a id="ref-for-typedefdef-binarydata①"></a>

    <a id="ref-for-dom-fontface-data-slot"></a>

    If the <code><a href="#dom-fontface-fontface-family-source-descriptors-source">source</a></code> argument was a <code><a href="#typedefdef-binarydata">BinaryData</a></code>, set <var>font face’s</var> internal <code><a href="#dom-fontface-data-slot">&#x5B;&#x5B;Data&#x5D;&#x5D;</a></code> slot to the passed argument.

3.  <a id="ref-for-dom-fontface-data-slot①"></a>

    If <var>font face’s</var> <code><a href="#dom-fontface-data-slot">&#x5B;&#x5B;Data&#x5D;&#x5D;</a></code> slot is not `null`, queue a task to run the following steps synchronously:

    1.  <a id="ref-for-dom-fontface-status④"></a>

        Set <var>font face’s</var> <code><a href="#dom-fontface-status">status</a></code> attribute to "loading".

    2.  <a id="ref-for-fontfaceset"></a>

        For each <code><a href="#fontfaceset">FontFaceSet</a></code> <var>font face</var> is in:

        1.  <a id="ref-for-fontfaceset①"></a>

            <a id="ref-for-dom-fontfaceset-loadingfonts-slot"></a>

            <a id="ref-for-switch-the-fontfaceset-to-loading"></a>

            If the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list is empty, [switch the FontFaceSet to loading](#switch-the-fontfaceset-to-loading).

        2.  <a id="ref-for-fontfaceset②"></a>

            <a id="ref-for-dom-fontfaceset-loadingfonts-slot①"></a>

            Append <var>font face</var> to the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list.

    Asynchronously, attempt to parse the data in it as a font. When this is completed, successfully or not, queue a task to run the following steps synchronously:

    1.  <a id="ref-for-dom-fontface-fontstatuspromise-slot③"></a>

        <a id="ref-for-dom-fontface-status⑤"></a>

        If the load was successful, <var>font face</var> now represents the parsed font; fulfill <var>font face’s</var> <code><a href="#dom-fontface-fontstatuspromise-slot">&#x5B;&#x5B;FontStatusPromise&#x5D;&#x5D;</a></code> with <var>font face</var>, and set its <code><a href="#dom-fontface-status">status</a></code> attribute to "loaded".

        <a id="ref-for-fontfaceset③"></a>

        For each <code><a href="#fontfaceset">FontFaceSet</a></code> <var>font face</var> is in:

        1.  <a id="ref-for-fontfaceset④"></a>

            <a id="ref-for-dom-fontfaceset-loadedfonts-slot"></a>

            Add <var>font face</var> to the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadedfonts-slot">&#x5B;&#x5B;LoadedFonts&#x5D;&#x5D;</a></code> list.

        2.  <a id="ref-for-fontfaceset⑤"></a>

            <a id="ref-for-dom-fontfaceset-loadingfonts-slot②"></a>

            <a id="ref-for-switch-the-fontfaceset-to-loaded"></a>

            Remove <var>font face</var> from the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list. If <var>font</var> was the last item in that list (and so the list is now empty), [switch the FontFaceSet to loaded](#switch-the-fontfaceset-to-loaded).

    2.  <a id="ref-for-dom-fontface-fontstatuspromise-slot④"></a>

        <a id="ref-for-dom-fontface-status⑥"></a>

        Otherwise, reject <var>font face’s</var> <code><a href="#dom-fontface-fontstatuspromise-slot">&#x5B;&#x5B;FontStatusPromise&#x5D;&#x5D;</a></code> with a DOMException named "SyntaxError" and set <var>font face’s</var> <code><a href="#dom-fontface-status">status</a></code> attribute to "error".

        <a id="ref-for-fontfaceset⑥"></a>

        For each <code><a href="#fontfaceset">FontFaceSet</a></code> <var>font face</var> is in:

        1.  <a id="ref-for-fontfaceset⑦"></a>

            <a id="ref-for-dom-fontfaceset-failedfonts-slot"></a>

            Add <var>font face</var> to the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-failedfonts-slot">&#x5B;&#x5B;FailedFonts&#x5D;&#x5D;</a></code> list.

        2.  <a id="ref-for-fontfaceset⑧"></a>

            <a id="ref-for-dom-fontfaceset-loadingfonts-slot③"></a>

            <a id="ref-for-switch-the-fontfaceset-to-loaded①"></a>

            Remove <var>font face</var> from the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list. If <var>font</var> was the last item in that list (and so the list is now empty), [switch the FontFaceSet to loaded](#switch-the-fontfaceset-to-loaded).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Newly constructed FontFace objects are not automatically added to the FontFaceSet associated with a document or a context for a worker thread. This means that while newly constructed fonts can be preloaded, they cannot actually be used until they are explicitly added to a FontFaceSet. See the following section for a more complete description of FontFaceSet.

### <a id="font-face-load"></a>2.2.  The `load()` method

<a id="ref-for-dom-fontface-load②"></a>

<a id="ref-for-fontface①②"></a>

The <code><a href="#dom-fontface-load">load()</a></code> method of <code><a href="#fontface">FontFace</a></code> forces a url-based font face to request its font data and load. For fonts constructed from binary data, or fonts that are already loading or loaded, it does nothing.

When the <a id="dom-fontface-load"></a>`load()` method is called, execute these steps:

1.  <a id="ref-for-fontface①③"></a>

    Let <var>font face</var> be the <code><a href="#fontface">FontFace</a></code> object on which this method was called.

2.  <a id="ref-for-dom-fontface-fontstatuspromise-slot⑤"></a>

    <a id="ref-for-dom-fontface-status⑦"></a>

    <a id="ref-for-dom-fontface-urls-slot①"></a>

    If <var>font face’s</var> <code><a href="#dom-fontface-urls-slot">&#x5B;&#x5B;Urls&#x5D;&#x5D;</a></code> slot is `null`, or its <code><a href="#dom-fontface-status">status</a></code> attribute is anything other than `"unloaded"`, return <var>font face’s</var> <code><a href="#dom-fontface-fontstatuspromise-slot">&#x5B;&#x5B;FontStatusPromise&#x5D;&#x5D;</a></code> and abort these steps.

3.  <a id="ref-for-dom-fontface-fontstatuspromise-slot⑥"></a>

    <a id="ref-for-dom-fontface-status⑧"></a>

    Otherwise, set <var>font face’s</var> <code><a href="#dom-fontface-status">status</a></code> attribute to "loading", return <var>font face’s</var> <code><a href="#dom-fontface-fontstatuspromise-slot">&#x5B;&#x5B;FontStatusPromise&#x5D;&#x5D;</a></code>, and continue executing the rest of this algorithm asynchronously.

4.  <a id="ref-for-at-font-face-rule⑨"></a>

    <a id="ref-for-dom-fontface-urls-slot②"></a>

    Using the value of <var>font face’s</var> <code><a href="#dom-fontface-urls-slot">&#x5B;&#x5B;Urls&#x5D;&#x5D;</a></code> slot, attempt to load a font as defined in [\[CSS-FONTS-3\]](#biblio-css-fonts-3), as if it was the value of a [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule’s src descriptor.

5.  When the load operation completes, successfully or not, queue a task to run the following steps synchronously:
    1.  <a id="ref-for-dom-fontface-status⑨"></a>

        <a id="ref-for-dom-fontface-fontstatuspromise-slot⑦"></a>

        If the attempt to load fails, reject <var>font face’s</var> <code><a href="#dom-fontface-fontstatuspromise-slot">&#x5B;&#x5B;FontStatusPromise&#x5D;&#x5D;</a></code> with a DOMException whose name is "NetworkError" and set <var>font face’s</var> <code><a href="#dom-fontface-status">status</a></code> attribute to "error".

        <a id="ref-for-fontfaceset⑨"></a>

        For each <code><a href="#fontfaceset">FontFaceSet</a></code> <var>font face</var> is in:

        1.  <a id="ref-for-fontfaceset①⓪"></a>

            <a id="ref-for-dom-fontfaceset-failedfonts-slot①"></a>

            Add <var>font face</var> to the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-failedfonts-slot">&#x5B;&#x5B;FailedFonts&#x5D;&#x5D;</a></code> list.

        2.  <a id="ref-for-fontfaceset①①"></a>

            <a id="ref-for-dom-fontfaceset-loadingfonts-slot④"></a>

            <a id="ref-for-switch-the-fontfaceset-to-loaded②"></a>

            Remove <var>font face</var> from the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list. If <var>font</var> was the last item in that list (and so the list is now empty), [switch the FontFaceSet to loaded](#switch-the-fontfaceset-to-loaded).

    2.  <a id="ref-for-dom-fontface-status①⓪"></a>

        <a id="ref-for-dom-fontface-fontstatuspromise-slot⑧"></a>

        Otherwise, <var>font face</var> now represents the loaded font; fulfill <var>font face’s</var> <code><a href="#dom-fontface-fontstatuspromise-slot">&#x5B;&#x5B;FontStatusPromise&#x5D;&#x5D;</a></code> with <var>font face</var> and set <var>font face’s</var> <code><a href="#dom-fontface-status">status</a></code> attribute to "loaded".

        <a id="ref-for-fontfaceset①②"></a>

        For each <code><a href="#fontfaceset">FontFaceSet</a></code> <var>font face</var> is in:

        1.  <a id="ref-for-fontfaceset①③"></a>

            <a id="ref-for-dom-fontfaceset-loadedfonts-slot①"></a>

            Add <var>font face</var> to the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadedfonts-slot">&#x5B;&#x5B;LoadedFonts&#x5D;&#x5D;</a></code> list.

        2.  <a id="ref-for-fontfaceset①④"></a>

            <a id="ref-for-dom-fontfaceset-loadingfonts-slot⑤"></a>

            <a id="ref-for-switch-the-fontfaceset-to-loaded③"></a>

            Remove <var>font face</var> from the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list. If <var>font</var> was the last item in that list (and so the list is now empty), [switch the FontFaceSet to loaded](#switch-the-fontfaceset-to-loaded).

<a id="ref-for-fontface①④"></a>

<a id="ref-for-dom-fontface-load③"></a>

User agents can initiate font loads on their own, whenever they determine that a given font face is necessary to render something on the page. When this happens, they must act as if they had called the corresponding <code><a href="#fontface">FontFace</a></code>’s <code><a href="#dom-fontface-load">load()</a></code> method described here.

<a id="ref-for-fontface①⑤"></a>

<a id="ref-for-fontface①⑥"></a>

<a id="ref-for-fontfaceset①⑤"></a>

<a id="ref-for-fontface①⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some UAs utilize a "font cache" which avoids having to download the same font multiple times on a page or on multiple pages within the same origin. Multiple <code><a href="#fontface">FontFace</a></code> objects can be mapped to the same entry in the font cache, which means that a <code><a href="#fontface">FontFace</a></code> object might start loading unexpectedly, even if it’s not in a <code><a href="#fontfaceset">FontFaceSet</a></code>, because some other <code><a href="#fontface">FontFace</a></code> object pointing to the same font data (perhaps on a different page entirely!) has been loaded.

<a id="ref-for-at-font-face-rule①⓪"></a>

### <a id="font-face-css-connection"></a>2.3.  Interaction with CSS’s [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) Rule

<a id="ref-for-at-font-face-rule①①"></a>

<a id="ref-for-fontface①⑧"></a>

<a id="ref-for-font-source"></a>

<a id="ref-for-fontface①⑨"></a>

A CSS [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule automatically defines a corresponding <code><a href="#fontface">FontFace</a></code> object, which is automatically placed in the document’s [font source](#font-source) when the rule is parsed. This <code><a href="#fontface">FontFace</a></code> object is <a id="css-connected"></a>CSS-connected.

<a id="ref-for-fontface②⓪"></a>

<a id="ref-for-at-font-face-rule①②"></a>

<a id="ref-for-dom-fontface-family①"></a>

<a id="ref-for-dom-fontface-style②"></a>

<a id="ref-for-dom-fontface-weight①"></a>

<a id="ref-for-dom-fontface-stretch①"></a>

<a id="ref-for-dom-fontface-unicoderange①"></a>

<a id="ref-for-dom-fontface-variant①"></a>

<a id="ref-for-dom-fontface-featuresettings①"></a>

<a id="ref-for-fontface②①"></a>

The <code><a href="#fontface">FontFace</a></code> object corresponding to a [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule has its <code><a href="#dom-fontface-family">family</a></code>, <code><a href="#dom-fontface-style">style</a></code>, <code><a href="#dom-fontface-weight">weight</a></code>, <code><a href="#dom-fontface-stretch">stretch</a></code>, <code><a href="#dom-fontface-unicoderange">unicodeRange</a></code>, <code><a href="#dom-fontface-variant">variant</a></code>, and <code><a href="#dom-fontface-featuresettings">featureSettings</a></code> attributes set to the same value as the corresponding descriptors in the <a id="ref-for-at-font-face-rule①③"></a>@font-face rule. There is a two-way connection between the two: any change made to a <a id="ref-for-at-font-face-rule①④"></a>@font-face descriptor is immediately reflected in the corresponding <code><a href="#fontface">FontFace</a></code> attribute, and vice versa.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-41199acf"></a> When a FontFace is transferred between documents, it’s no longer CSS-connected.

<a id="ref-for-dom-fontface-urls-slot③"></a>

<a id="ref-for-fontface②②"></a>

<a id="ref-for-at-font-face-rule①⑤"></a>

The internal <code><a href="#dom-fontface-urls-slot">&#x5B;&#x5B;Urls&#x5D;&#x5D;</a></code> slot of the <code><a href="#fontface">FontFace</a></code> object is set to the value of the [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule’s src descriptor, and reflects any changes made to the src descriptor.

<a id="ref-for-fontface②③"></a>

<a id="ref-for-at-font-face-rule①⑥"></a>

Otherwise, a <code><a href="#fontface">FontFace</a></code> object created by a CSS [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule is identical to one created manually.

<a id="ref-for-at-font-face-rule①⑦"></a>

<a id="ref-for-fontface②④"></a>

<a id="ref-for-css-connected"></a>

<a id="ref-for-fontface②⑤"></a>

If a [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule is removed from the document, its corresponding <code><a href="#fontface">FontFace</a></code> object is no longer [CSS-connected](#css-connected). The connection is not restorable by any means (but adding the <a id="ref-for-at-font-face-rule①⑧"></a>@font-face back to the stylesheet will create a brand new <code><a href="#fontface">FontFace</a></code> object which <em>is</em> <a id="ref-for-css-connected①"></a>CSS-connected).

<a id="ref-for-at-font-face-rule①⑨"></a>

<a id="ref-for-descdef-font-face-src"></a>

<a id="ref-for-fontface②⑥"></a>

<a id="ref-for-css-connected②"></a>

<a id="ref-for-fontface②⑦"></a>

<a id="ref-for-fontface②⑧"></a>

<a id="ref-for-font-source①"></a>

If a [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule has its [src](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-src) descriptor changed to a new value, the original connected <code><a href="#fontface">FontFace</a></code> object must stop being [CSS-connected](#css-connected). A new <code><a href="#fontface">FontFace</a></code> reflecting its new <a id="ref-for-descdef-font-face-src①"></a>src must be created and <a id="ref-for-css-connected③"></a>CSS-connected to the <a id="ref-for-at-font-face-rule②⓪"></a>@font-face. (This will also remove the old and add the new <code><a href="#fontface">FontFace</a></code> objects from any [font sources](#font-source) they appear in.)

### <a id="discovery"></a>2.4.  Discovery of information about a font

<a id="ref-for-fontface②⑨"></a>

A <code><a href="#fontface">FontFace</a></code> object includes a variety of read-only information about the contents of the font file.

<a id="ref-for-Exposed①"></a>

<a id="fontfacefeatures"></a>

<a id="ref-for-Exposed②"></a>

<a id="fontfacevariationaxis"></a>

<a id="ref-for-idl-DOMString"></a>

<a id="dom-fontfacevariationaxis-name"></a>

<a id="ref-for-idl-DOMString①"></a>

<a id="dom-fontfacevariationaxis-axistag"></a>

<a id="ref-for-idl-double"></a>

<a id="dom-fontfacevariationaxis-minimumvalue"></a>

<a id="ref-for-idl-double①"></a>

<a id="dom-fontfacevariationaxis-maximumvalue"></a>

<a id="ref-for-idl-double②"></a>

<a id="dom-fontfacevariationaxis-defaultvalue"></a>

<a id="ref-for-Exposed③"></a>

<a id="fontfacevariations"></a>

<a id="ref-for-fontfacevariationaxis"></a>

<a id="ref-for-Exposed④"></a>

<a id="fontfacepalette"></a>

<a id="ref-for-idl-DOMString②"></a>

<a id="ref-for-idl-unsigned-long"></a>

<a id="dom-fontfacepalette-length"></a>

<a id="ref-for-idl-DOMString③"></a>

<a id="ref-for-idl-unsigned-long①"></a>

<a id="dom-fontfacepalette-__getter__-index-index"></a>

<a id="ref-for-idl-boolean"></a>

<a id="dom-fontfacepalette-usablewithlightbackground"></a>

<a id="ref-for-idl-boolean①"></a>

<a id="dom-fontfacepalette-usablewithdarkbackground"></a>

<a id="ref-for-Exposed⑤"></a>

<a id="fontfacepalettes"></a>

<a id="ref-for-fontfacepalette"></a>

<a id="ref-for-idl-unsigned-long②"></a>

<a id="dom-fontfacepalettes-length"></a>

<a id="ref-for-fontfacepalette①"></a>

<a id="ref-for-idl-unsigned-long③"></a>

<a id="dom-fontfacepalettes-__getter__-index-index"></a>

<a id="ref-for-fontface③⓪"></a>

<a id="ref-for-fontfacefeatures"></a>

<a id="dom-fontface-features"></a>

<a id="ref-for-fontfacevariations"></a>

<a id="dom-fontface-variations"></a>

<a id="ref-for-fontfacepalettes"></a>

<a id="dom-fontface-palettes"></a>

```text
[Exposed=(Window,Worker)]
interface FontFaceFeatures {
  /* The CSSWG is still discussing what goes in here */
};

[Exposed=(Window,Worker)]
interface FontFaceVariationAxis {
  readonly attribute DOMString name;
  readonly attribute DOMString axisTag;
  readonly attribute double minimumValue;
  readonly attribute double maximumValue;
  readonly attribute double defaultValue;
};

[Exposed=(Window,Worker)]
interface FontFaceVariations {
  readonly setlike<FontFaceVariationAxis>;
};

[Exposed=(Window,Worker)]
interface FontFacePalette {
  iterable<DOMString>;
  readonly attribute unsigned long length;
  getter DOMString (unsigned long index);
  readonly attribute boolean usableWithLightBackground;
  readonly attribute boolean usableWithDarkBackground;
};

[Exposed=(Window,Worker)]
interface FontFacePalettes {
  iterable<FontFacePalette>;
  readonly attribute unsigned long length;
  getter FontFacePalette (unsigned long index);
};

partial interface FontFace {
  readonly attribute FontFaceFeatures features;
  readonly attribute FontFaceVariations variations;
  readonly attribute FontFacePalettes palettes;
};
```
<a id="ref-for-propdef-font-feature-settings"></a>

<a id="ref-for-propdef-font-variation-settings"></a>

<a id="ref-for-at-ruledef-font-palette-values"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This read-only data is intended to help authors know which values are accepted by [font-feature-settings](https://www.w3.org/TR/css-fonts-4/#propdef-font-feature-settings), [font-variation-settings](https://www.w3.org/TR/css-fonts-4/#propdef-font-variation-settings), and [@font-palette-values](https://www.w3.org/TR/css-fonts-4/#at-ruledef-font-palette-values).

## <a id="FontFaceSet-interface"></a>3.  The `FontFaceSet` Interface

<a id="dictdef-fontfacesetloadeventinit"></a>

<a id="ref-for-dictdef-eventinit"></a>

<a id="ref-for-idl-sequence"></a>

<a id="ref-for-fontface③①"></a>

<a id="dom-fontfacesetloadeventinit-fontfaces"></a>

<a id="ref-for-Exposed⑥"></a>

<a id="fontfacesetloadevent"></a>

<a id="ref-for-event"></a>

<a id="dom-fontfacesetloadevent-fontfacesetloadevent"></a>

<a id="ref-for-cssomstring③⑨"></a>

<a id="dom-fontfacesetloadevent-fontfacesetloadevent-type-eventinitdict-type"></a>

<a id="ref-for-dictdef-fontfacesetloadeventinit"></a>

<a id="dom-fontfacesetloadevent-fontfacesetloadevent-type-eventinitdict-eventinitdict"></a>

<a id="ref-for-SameObject"></a>

<a id="ref-for-idl-frozen-array"></a>

<a id="ref-for-fontface③②"></a>

<a id="dom-fontfacesetloadevent-fontfaces"></a>

<a id="enumdef-fontfacesetloadstatus"></a>

<a id="dom-fontfacesetloadstatus-loading"></a>

<a id="dom-fontfacesetloadstatus-loaded"></a>

<a id="ref-for-Exposed⑦"></a>

<a id="fontfaceset"></a>

<a id="ref-for-eventtarget"></a>

<a id="ref-for-dom-fontfaceset-fontfaceset"></a>

<a id="ref-for-idl-sequence①"></a>

<a id="ref-for-fontface③③"></a>

<a id="dom-fontfaceset-fontfaceset-initialfaces-initialfaces"></a>

<a id="ref-for-fontface③④"></a>

<a id="ref-for-fontfaceset①⑥"></a>

<a id="ref-for-dom-fontfaceset-add"></a>

<a id="ref-for-fontface③⑤"></a>

<a id="dom-fontfaceset-add-font-font"></a>

<a id="ref-for-idl-boolean②"></a>

<a id="ref-for-dom-fontfaceset-delete"></a>

<a id="ref-for-fontface③⑥"></a>

<a id="dom-fontfaceset-delete-font-font"></a>

<a id="ref-for-idl-undefined"></a>

<a id="ref-for-dom-fontfaceset-clear"></a>

<a id="ref-for-eventhandler"></a>

<a id="dom-fontfaceset-onloading"></a>

<a id="ref-for-eventhandler①"></a>

<a id="dom-fontfaceset-onloadingdone"></a>

<a id="ref-for-eventhandler②"></a>

<a id="dom-fontfaceset-onloadingerror"></a>

<a id="ref-for-idl-promise④"></a>

<a id="ref-for-idl-sequence②"></a>

<a id="ref-for-fontface③⑦"></a>

<a id="ref-for-dom-fontfaceset-load"></a>

<a id="ref-for-cssomstring④⓪"></a>

<a id="ref-for-dom-fontfaceset-load-font-text-font"></a>

<a id="ref-for-cssomstring④①"></a>

<a id="ref-for-dom-fontfaceset-load-font-text-text"></a>

<a id="ref-for-idl-boolean③"></a>

<a id="ref-for-dom-fontfaceset-check"></a>

<a id="ref-for-cssomstring④②"></a>

<a id="ref-for-dom-fontfaceset-check-font-text-font"></a>

<a id="ref-for-cssomstring④③"></a>

<a id="ref-for-dom-fontfaceset-check-font-text-text"></a>

<a id="ref-for-idl-promise⑤"></a>

<a id="ref-for-fontfaceset①⑦"></a>

<a id="ref-for-dom-fontfaceset-ready"></a>

<a id="ref-for-enumdef-fontfacesetloadstatus"></a>

<a id="dom-fontfaceset-status"></a>

```text
dictionary FontFaceSetLoadEventInit : EventInit {
  sequence<FontFace> fontfaces = [];
};

[Exposed=(Window,Worker)]
interface FontFaceSetLoadEvent : Event {
  constructor(CSSOMString type, optional FontFaceSetLoadEventInit eventInitDict = {});
  [SameObject] readonly attribute FrozenArray<FontFace> fontfaces;
};

enum FontFaceSetLoadStatus { "loading", "loaded" };

[Exposed=(Window,Worker)]
interface FontFaceSet : EventTarget {
  constructor(sequence<FontFace> initialFaces);

  setlike<FontFace>;
  FontFaceSet add(FontFace font);
  boolean delete(FontFace font);
  undefined clear();

  // events for when loading state changes
  attribute EventHandler onloading;
  attribute EventHandler onloadingdone;
  attribute EventHandler onloadingerror;

  // check and start loads if appropriate
  // and fulfill promise when all loads complete
  Promise<sequence<FontFace>> load(CSSOMString font, optional CSSOMString text = " ");

  // return whether all fonts in the fontlist are loaded
  // (does not initiate load if not available)
  boolean check(CSSOMString font, optional CSSOMString text = " ");

  // async notification that font loading and layout operations are done
  readonly attribute Promise<FontFaceSet> ready;

  // loading state, "loading" while one or more fonts loading, "loaded" otherwise
  readonly attribute FontFaceSetLoadStatus status;
};
```
<a id="ref-for-fontfaceset①⑧"></a>

<a id="dom-fontfaceset-ready"></a>`ready`, of type Promise\<[FontFaceSet](#fontfaceset)\>, readonly

<a id="ref-for-fontfaceset①⑨"></a>

<a id="ref-for-dom-fontfaceset-readypromise-slot"></a>

This attribute reflects the <code><a href="#fontfaceset">FontFaceSet</a></code>'s <code><a href="#dom-fontfaceset-readypromise-slot">&#x5B;&#x5B;ReadyPromise&#x5D;&#x5D;</a></code> slot.

<a id="ref-for-idl-promise⑥"></a>

See [§ 3.4 The ready attribute](#font-face-set-ready) for more details on this <code><a href="https://webidl.spec.whatwg.org/#idl-promise">Promise</a></code> and its use.

<a id="dom-fontfaceset-fontfaceset"></a>`FontFaceSet(initialFaces)`

<a id="ref-for-fontfaceset②⓪"></a>

<a id="ref-for-dom-fontfaceset-fontfaceset-initialfaces-initialfaces"></a>

<a id="ref-for-fontfaceset-set-entries"></a>

The <code><a href="#fontfaceset">FontFaceSet</a></code> constructor, when called, must iterate its <code><a href="#dom-fontfaceset-fontfaceset-initialfaces-initialfaces">initialFaces</a></code> argument and add each value to its [set entries](#fontfaceset-set-entries).

<a id="fontfaceset-iteration-order"></a>iteration order

<a id="ref-for-css-connected④"></a>

<a id="ref-for-fontface③⑧"></a>

<a id="ref-for-at-font-face-rule②①"></a>

<a id="ref-for-fontface③⑨"></a>

When iterated over, all [CSS-connected](#css-connected) <code><a href="#fontface">FontFace</a></code> objects must come first, in document order of their connected [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rules, followed by the non-<a id="ref-for-css-connected⑤"></a>CSS-connected <code><a href="#fontface">FontFace</a></code> objects, in insertion order.

<a id="fontfaceset-set-entries"></a>set entries

<a id="ref-for-fontfaceset②①"></a>

<a id="ref-for-font-source②"></a>

<a id="ref-for-dfn-set-entries"></a>

If a <code><a href="#fontfaceset">FontFaceSet</a></code> is a [font source](#font-source), its [set entries](https://webidl.spec.whatwg.org/#dfn-set-entries) are initialized as specified in [§ 4.2 Interaction with CSS’s @font-face Rule](#document-font-face-set).

<a id="ref-for-dfn-set-entries①"></a>

Otherwise, its [set entries](https://webidl.spec.whatwg.org/#dfn-set-entries) are initially empty.

<a id="dom-fontfaceset-add"></a>`add(font)`

<a id="ref-for-dom-fontfaceset-add①"></a>

When the <code><a href="#dom-fontfaceset-add">add()</a></code> method is called, execute the following steps:

1.  <a id="ref-for-fontfaceset②②"></a>

    <a id="ref-for-fontfaceset-set-entries①"></a>

    If <var>font</var> is already in the <code><a href="#fontfaceset">FontFaceSet</a></code>’s [set entries](#fontfaceset-set-entries), skip to the last step of this algorithm immediately.

2.  <a id="ref-for-css-connected⑥"></a>

    <a id="ref-for-invalidmodificationerror"></a>

    If <var>font</var> is [CSS-connected](#css-connected), throw an <code><a href="https://webidl.spec.whatwg.org/#invalidmodificationerror">InvalidModificationError</a></code> exception and exit this algorithm immediately.

3.  <a id="ref-for-fontfaceset②③"></a>

    <a id="ref-for-fontfaceset-set-entries②"></a>

    Add the <var>font</var> argument to the <code><a href="#fontfaceset">FontFaceSet</a></code>’s [set entries](#fontfaceset-set-entries).

4.  <a id="ref-for-dom-fontface-status①①"></a>

    If <var>font</var>’s <code><a href="#dom-fontface-status">status</a></code> attribute is "loading":

    1.  <a id="ref-for-fontfaceset②④"></a>

        <a id="ref-for-dom-fontfaceset-loadingfonts-slot⑥"></a>

        <a id="ref-for-switch-the-fontfaceset-to-loading①"></a>

        If the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list is empty, [switch the FontFaceSet to loading](#switch-the-fontfaceset-to-loading).

    2.  <a id="ref-for-fontfaceset②⑤"></a>

        <a id="ref-for-dom-fontfaceset-loadingfonts-slot⑦"></a>

        Append <var>font</var> to the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list.

5.  <a id="ref-for-fontfaceset②⑥"></a>

    Return the <code><a href="#fontfaceset">FontFaceSet</a></code>.

<a id="dom-fontfaceset-delete"></a>`delete(font)`

<a id="ref-for-dom-fontfaceset-delete①"></a>

When the <code><a href="#dom-fontfaceset-delete">delete()</a></code> method is called, execute the following steps:

1.  <a id="ref-for-css-connected⑦"></a>

    If <var>font</var> is [CSS-connected](#css-connected), return `false` and exit this algorithm immediately.

2.  <a id="ref-for-fontfaceset②⑦"></a>

    <a id="ref-for-fontfaceset-set-entries③"></a>

    Let <var>deleted</var> be the result of removing <var>font</var> from the <code><a href="#fontfaceset">FontFaceSet</a></code>’s [set entries](#fontfaceset-set-entries).

3.  <a id="ref-for-fontfaceset②⑧"></a>

    <a id="ref-for-dom-fontfaceset-loadedfonts-slot②"></a>

    <a id="ref-for-dom-fontfaceset-failedfonts-slot②"></a>

    If <var>font</var> is present in the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadedfonts-slot">&#x5B;&#x5B;LoadedFonts&#x5D;&#x5D;</a></code>, or <code><a href="#dom-fontfaceset-failedfonts-slot">&#x5B;&#x5B;FailedFonts&#x5D;&#x5D;</a></code> lists, remove it.

4.  <a id="ref-for-fontfaceset②⑨"></a>

    <a id="ref-for-dom-fontfaceset-loadingfonts-slot⑧"></a>

    <a id="ref-for-switch-the-fontfaceset-to-loaded④"></a>

    If <var>font</var> is present in the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list, remove it. If <var>font</var> was the last item in that list (and so the list is now empty), [switch the FontFaceSet to loaded](#switch-the-fontfaceset-to-loaded).

5.  Return <var>deleted</var>.

<a id="dom-fontfaceset-clear"></a>`clear()`

<a id="ref-for-dom-fontfaceset-clear①"></a>

When the <code><a href="#dom-fontfaceset-clear">clear()</a></code> method is called, execute the following steps:

1.  <a id="ref-for-css-connected⑧"></a>

    <a id="ref-for-fontfaceset③⓪"></a>

    <a id="ref-for-fontfaceset-set-entries④"></a>

    <a id="ref-for-dom-fontfaceset-loadedfonts-slot③"></a>

    <a id="ref-for-dom-fontfaceset-failedfonts-slot③"></a>

    Remove all non-[CSS-connected](#css-connected) items from the <code><a href="#fontfaceset">FontFaceSet</a></code>’s [set entries](#fontfaceset-set-entries), its <code><a href="#dom-fontfaceset-loadedfonts-slot">&#x5B;&#x5B;LoadedFonts&#x5D;&#x5D;</a></code> list, and its <code><a href="#dom-fontfaceset-failedfonts-slot">&#x5B;&#x5B;FailedFonts&#x5D;&#x5D;</a></code> list.

2.  <a id="ref-for-fontfaceset③①"></a>

    <a id="ref-for-dom-fontfaceset-loadingfonts-slot⑨"></a>

    <a id="ref-for-switch-the-fontfaceset-to-loaded⑤"></a>

    If the <code><a href="#fontfaceset">FontFaceSet</a></code>’s <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list is non-empty, remove all items from it, then [switch the FontFaceSet to loaded](#switch-the-fontfaceset-to-loaded).

<a id="ref-for-fontfaceset③②"></a>

<a id="ref-for-idl-promise⑦"></a>

<code><a href="#fontfaceset">FontFaceSet</a></code> objects also have internal <a id="dom-fontfaceset-loadingfonts-slot"></a>`[[LoadingFonts]]`, <a id="dom-fontfaceset-loadedfonts-slot"></a>`[[LoadedFonts]]`, and <a id="dom-fontfaceset-failedfonts-slot"></a>`[[FailedFonts]]` slots, all of which are initialized to empty lists, and a <a id="dom-fontfaceset-readypromise-slot"></a>`[[ReadyPromise]]` slot, which is initialized to a fresh pending <code><a href="https://webidl.spec.whatwg.org/#idl-promise">Promise</a></code>.

Because font families are loaded only when they are used, content sometimes needs to understand when the loading of fonts occurs. Authors can use the events and methods defined here to allow greater control over actions that are dependent upon the availability of specific fonts.

<a id="ref-for-fontfaceset③③"></a>

A <code><a href="#fontfaceset">FontFaceSet</a></code> is <a id="fontfaceset-pending-on-the-environment"></a>pending on the environment if any of the following are true:

- the document is still loading

- the document has pending stylesheet requests

- the document has pending layout operations which might cause the user agent to request a font, or which depend on recently-loaded fonts

<a id="ref-for-fontfaceset③④"></a>

<a id="ref-for-fontfaceset-pending-on-the-environment"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The idea is that once a <code><a href="#fontfaceset">FontFaceSet</a></code> stops being [pending on the environment](#fontfaceset-pending-on-the-environment), as long as nothing further changes the document, an author can depend on sizes/positions of things being "correct" when measured. If the above conditions do not fully capture this guarantee, they need to be amended to do so.

### <a id="FontFaceSet-events"></a>3.1.  Events

Font load events make it easy to respond to the font-loading behavior of the entire document, rather than having to listen to each font specifically. The <a id="eventdef-fontfaceset-loading"></a>`loading` event fires when the document begins loading fonts, while the <a id="eventdef-fontfaceset-loadingdone"></a>`loadingdone` and <a id="eventdef-fontfaceset-loadingerror"></a>`loadingerror` events fire when the document is done loading fonts, containing the fonts that successfully loaded or failed to load, respectively.

The following are the event handlers (and their corresponding event handler event types) that must be supported by `FontFaceSet` objects as IDL attributes:

<a id="eventhandlers"></a>

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Event handler

<strong>Column 2 (header cell):</strong>

Event handler event type

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

<a id="ref-for-dom-fontfaceset-onloading"></a>

<code><a href="#dom-fontfaceset-onloading">onloading</a></code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-eventdef-fontfaceset-loading"></a>

<code><a href="#eventdef-fontfaceset-loading">loading</a></code>

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

<a id="ref-for-dom-fontfaceset-onloadingdone"></a>

<code><a href="#dom-fontfaceset-onloadingdone">onloadingdone</a></code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-eventdef-fontfaceset-loadingdone"></a>

<code><a href="#eventdef-fontfaceset-loadingdone">loadingdone</a></code>

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

<a id="ref-for-dom-fontfaceset-onloadingerror"></a>

<code><a href="#dom-fontfaceset-onloadingerror">onloadingerror</a></code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-eventdef-fontfaceset-loadingerror"></a>

<code><a href="#eventdef-fontfaceset-loadingerror">loadingerror</a></code>

<a id="ref-for-fontfaceset③⑤"></a>

<a id="ref-for-fontfacesetloadevent"></a>

To <a id="fire-a-font-load-event"></a>fire a font load event named <var>e</var> at a <code><a href="#fontfaceset">FontFaceSet</a></code> <var>target</var> with optional <var>font faces</var> means to [fire a simple event](https://www.w3.org/TR/html5/webappapis.html#event-firing) named <var>e</var> using the <code><a href="#fontfacesetloadevent">FontFaceSetLoadEvent</a></code> interface that also meets these conditions:

1.  <a id="ref-for-fontface④⓪"></a>

    <a id="ref-for-dom-fontfacesetloadevent-fontfaces"></a>

    The <code><a href="#dom-fontfacesetloadevent-fontfaces">fontfaces</a></code> attribute is initialized to the result of filtering <var>font faces</var> to only contain <code><a href="#fontface">FontFace</a></code> objects contained in <var>target</var>.

<a id="ref-for-fontfaceset③⑥"></a>

When asked to <a id="switch-the-fontfaceset-to-loading"></a>switch the FontFaceSet to loading for a given <code><a href="#fontfaceset">FontFaceSet</a></code>, the user agent must run the following steps:

1.  <a id="ref-for-fontfaceset③⑦"></a>

    Let <var>font face set</var> be the given <code><a href="#fontfaceset">FontFaceSet</a></code>.

2.  <a id="ref-for-dom-fontfaceset-status"></a>

    Set the <code><a href="#dom-fontfaceset-status">status</a></code> attribute of <var>font face set</var> to "loading".

3.  <a id="ref-for-dom-fontfaceset-readypromise-slot①"></a>

    If <var>font face set’s</var> <code><a href="#dom-fontfaceset-readypromise-slot">&#x5B;&#x5B;ReadyPromise&#x5D;&#x5D;</a></code> slot currently holds a fulfilled promise, replace it with a fresh pending promise.

4.  <a id="ref-for-eventdef-fontfaceset-loading①"></a>

    <a id="ref-for-fire-a-font-load-event"></a>

    Queue a task to [fire a font load event](#fire-a-font-load-event) named <code><a href="#eventdef-fontfaceset-loading">loading</a></code> at <var>font face set</var>.

<a id="ref-for-fontfaceset③⑧"></a>

When asked to <a id="switch-the-fontfaceset-to-loaded"></a>switch the FontFaceSet to loaded for a given <code><a href="#fontfaceset">FontFaceSet</a></code>, the user agent must run the following steps:

1.  <a id="ref-for-fontfaceset③⑨"></a>

    Let <var>font face set</var> be the given <code><a href="#fontfaceset">FontFaceSet</a></code>.

2.  <a id="ref-for-fontfaceset-pending-on-the-environment①"></a>

    If <var>font face set</var> is [pending on the environment](#fontfaceset-pending-on-the-environment), mark it as <a id="fontfaceset-stuck-on-the-environment"></a>stuck on the environment, and exit this algorithm.

3.  <a id="ref-for-dom-fontfaceset-status①"></a>

    Set <var>font face set’s</var> <code><a href="#dom-fontfaceset-status">status</a></code> attribute to "loaded".

4.  <a id="ref-for-dom-fontfaceset-readypromise-slot②"></a>

    Fulfill <var>font face set’s</var> <code><a href="#dom-fontfaceset-readypromise-slot">&#x5B;&#x5B;ReadyPromise&#x5D;&#x5D;</a></code> attribute’s value with <var>font face set</var>.

5.  Queue a task to perform the following steps synchronously:

    1.  <a id="ref-for-dom-fontfaceset-loadedfonts-slot④"></a>

        Let <var>loaded fonts</var> be the (possibly empty) contents of <var>font face set’s</var> <code><a href="#dom-fontfaceset-loadedfonts-slot">&#x5B;&#x5B;LoadedFonts&#x5D;&#x5D;</a></code> slot.

    2.  <a id="ref-for-dom-fontfaceset-failedfonts-slot④"></a>

        Let <var>failed fonts</var> be the (possibly empty) contents of <var>font face set’s</var> <code><a href="#dom-fontfaceset-failedfonts-slot">&#x5B;&#x5B;FailedFonts&#x5D;&#x5D;</a></code> slot.

    3.  <a id="ref-for-dom-fontfaceset-loadedfonts-slot⑤"></a>

        <a id="ref-for-dom-fontfaceset-failedfonts-slot⑤"></a>

        Reset the <code><a href="#dom-fontfaceset-loadedfonts-slot">&#x5B;&#x5B;LoadedFonts&#x5D;&#x5D;</a></code> and <code><a href="#dom-fontfaceset-failedfonts-slot">&#x5B;&#x5B;FailedFonts&#x5D;&#x5D;</a></code> slots to empty lists.

    4.  <a id="ref-for-fire-a-font-load-event①"></a>

        <a id="ref-for-eventdef-fontfaceset-loadingdone①"></a>

        [Fire a font load event](#fire-a-font-load-event) named <code><a href="#eventdef-fontfaceset-loadingdone">loadingdone</a></code> at <var>font face set</var> with <var>loaded fonts</var>.

    5.  <a id="ref-for-fire-a-font-load-event②"></a>

        <a id="ref-for-eventdef-fontfaceset-loadingerror①"></a>

        If <var>font face set’s</var> <var>failed fonts</var> is non-empty, [fire a font load event](#fire-a-font-load-event) named <code><a href="#eventdef-fontfaceset-loadingerror">loadingerror</a></code> at <var>font face set</var> with <var>failed fonts</var>.

<a id="ref-for-fontfaceset④⓪"></a>

<a id="ref-for-fontfaceset-pending-on-the-environment②"></a>

Whenever a <code><a href="#fontfaceset">FontFaceSet</a></code> goes from [pending on the environment](#fontfaceset-pending-on-the-environment) to not <a id="ref-for-fontfaceset-pending-on-the-environment③"></a>pending on the environment, the user agent must run the following steps:

1.  <a id="ref-for-fontfaceset④①"></a>

    <a id="ref-for-fontfaceset-stuck-on-the-environment"></a>

    <a id="ref-for-dom-fontfaceset-loadingfonts-slot①⓪"></a>

    <a id="ref-for-switch-the-fontfaceset-to-loaded⑥"></a>

    If the <code><a href="#fontfaceset">FontFaceSet</a></code> is [stuck on the environment](#fontfaceset-stuck-on-the-environment) and its <code><a href="#dom-fontfaceset-loadingfonts-slot">&#x5B;&#x5B;LoadingFonts&#x5D;&#x5D;</a></code> list is empty, [switch the FontFaceSet to loaded](#switch-the-fontfaceset-to-loaded).

2.  <a id="ref-for-fontfaceset④②"></a>

    <a id="ref-for-fontfaceset-stuck-on-the-environment①"></a>

    If the <code><a href="#fontfaceset">FontFaceSet</a></code> is [stuck on the environment](#fontfaceset-stuck-on-the-environment), unmark it as such.

If asked to <a id="find-the-matching-font-faces"></a>find the matching font faces from a FontFaceSet <var>source</var>, for a given font string <var>font</var> optionally some sample text <var>text</var>, and optionally an <var>allow system fonts</var> flag, run the following steps:

1.  <a id="ref-for-propdef-font"></a>

    Parse <var>font</var> using the CSS value syntax of the [font](https://www.w3.org/TR/css-fonts-4/#propdef-font) property. If a syntax error occurs, return a syntax error.

    <a id="ref-for-css-wide-keywords"></a>

    If the parsed value is a [CSS-wide keyword](https://www.w3.org/TR/css-values-4/#css-wide-keywords), return a syntax error.

    <a id="ref-for-valdef-font-weight-bolder"></a>

    <a id="ref-for-valdef-font-weight-normal"></a>

    Absolutize all relative lengths against the initial values of the corresponding properties. (For example, a relative font weight like [bolder](https://www.w3.org/TR/css-fonts-4/#valdef-font-weight-bolder) is evaluated against the initial value [normal](https://www.w3.org/TR/css-fonts-4/#valdef-font-weight-normal).)

2.  If <var>text</var> was not explicitly provided, let it be a string containing a single space character (U+0020 SPACE).

3.  Let <var>font family list</var> be the list of font families parsed from <var>font</var>, and <var>font style</var> be the other font style attributes parsed from <var>font</var>.

4.  <a id="ref-for-available-font-faces"></a>

    Let <var>available font faces</var> be the [available font faces](#available-font-faces) within <var>source</var>. If the <var>allow system fonts</var> flag is specified, add all system fonts to <var>available font faces</var>.

5.  Let <var>matched font faces</var> initially be an empty list.

6.  <a id="ref-for-dom-fontface-unicoderange②"></a>

    For each family in <var>font family list</var>, use the font matching rules to select the font faces from <var>available font faces</var> that match the <var>font style</var>, and add them to <var>matched font faces</var>. The use of the <code><a href="#dom-fontface-unicoderange">unicodeRange</a></code> attribute means that this may be more than just a single font face.

7.  If <var>matched font faces</var> is empty, set the <var>found faces</var> flag to false. Otherwise, set it to true.

8.  <a id="ref-for-descdef-font-face-unicode-range"></a>

    For each font face in <var>matched font faces</var>, if its defined [unicode-range](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-unicode-range) does not include the codepoint of at least one character in <var>text</var>, remove it from the list.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Therefore, if <var>text</var> is the empty string, every font will be removed.

9.  Return <var>matched font faces</var> and the <var>found faces</var> flag.

### <a id="font-face-set-load"></a>3.2.  The `load()` method

<a id="ref-for-dom-fontfaceset-load①"></a>

<a id="ref-for-fontfaceset④③"></a>

The <code><a href="#dom-fontfaceset-load">load()</a></code> method of <code><a href="#fontfaceset">FontFaceSet</a></code> will determine whether all fonts in the given font list have been loaded and are available. If any fonts are downloadable fonts and have not already been loaded, the user agent will initiate the load of each of these fonts. It returns a Promise, which is fulfilled when all of the fonts are loaded and ready to be used, or rejected if any font failed to load properly.

When the <a id="dom-fontfaceset-load"></a>`load`( <a id="dom-fontfaceset-load-font-text-font"></a>`font`, <a id="dom-fontfaceset-load-font-text-text"></a>`text` ) method is called, execute these steps:

1.  <a id="ref-for-fontfaceset④④"></a>

    Let <var>font face set</var> be the <code><a href="#fontfaceset">FontFaceSet</a></code> object this method was called on. Let <var>promise</var> be a newly-created promise object.

2.  Return <var>promise</var>. Complete the rest of these steps asynchronously.

3.  <a id="ref-for-dom-fontfacesetload-text"></a>

    <a id="ref-for-dom-fontfacesetload-font"></a>

    <a id="ref-for-find-the-matching-font-faces"></a>

    [Find the matching font faces](#find-the-matching-font-faces) from <var>font face set</var> using the <code><a href="https://www.w3.org/TR/css-font-loading-3/#dom-fontfacesetload-font">font</a></code> and <code><a href="https://www.w3.org/TR/css-font-loading-3/#dom-fontfacesetload-text">text</a></code> arguments passed to the function, and let <var>font face list</var> be the return value (ignoring the <var>found faces</var> flag). If a syntax error was returned, reject <var>promise</var> with a SyntaxError exception and terminate these steps.

4.  Queue a task to run the following steps synchronously:
    1.  <a id="ref-for-dom-fontface-load④"></a>

        For all of the font faces in the <var>font face list</var>, call their <code><a href="#dom-fontface-load">load()</a></code> method.

    2.  <a id="ref-for-dom-fontface-fontstatuspromise-slot⑨"></a>

        Resolve <var>promise</var> with the result of waiting for all of the <code><a href="#dom-fontface-fontstatuspromise-slot">&#x5B;&#x5B;FontStatusPromise&#x5D;&#x5D;</a></code>s of each font face in the <var>font face list</var>, in order.

### <a id="font-face-set-check"></a>3.3.  The `check()` method

<a id="ref-for-dom-fontfaceset-check①"></a>

<a id="ref-for-fontfaceset④⑤"></a>

The <code><a href="#dom-fontfaceset-check">check()</a></code> method of <code><a href="#fontfaceset">FontFaceSet</a></code> will determine whether you can "safely" render some provided text with a particular font list, such that it won’t cause a "font swap" later. If the given text/font combo will render without attempting to use any unloaded or currently-loading fonts, this method will return true; otherwise, it returns false.

> <strong data-conversion-semantic="note">Note</strong>
>
> Two special cases in this method’s behavior should be noted, as they are non-obvious:
>
> - <a id="ref-for-descdef-font-face-unicode-range①"></a>
>
>   If the specified fonts exist, but all possible faces are ruled out due to their [unicode-range](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-unicode-range) not covering the provided text, the method returns `true`, as the text will be rendered in the UA’s fallback font instead, and won’t trigger any font loads.
>
> - Likewise, if none of the specified fonts exist (for example, names are mis-spelled), the method also returns `true`, because using this font list will not trigger any loads; instead, fallback will occur.

When the <a id="dom-fontfaceset-check"></a>`check`( <a id="dom-fontfaceset-check-font-text-font"></a>`font`, <a id="dom-fontfaceset-check-font-text-text"></a>`text`) method is called, execute these steps:

1.  <a id="ref-for-fontfaceset④⑥"></a>

    Let <var>font face set</var> be the <code><a href="#fontfaceset">FontFaceSet</a></code> object this method was called on.

2.  <a id="ref-for-dom-fontfacesetcheck-text"></a>

    <a id="ref-for-dom-fontfacesetcheck-font"></a>

    <a id="ref-for-find-the-matching-font-faces①"></a>

    [Find the matching font faces](#find-the-matching-font-faces) from <var>font face set</var> using the <code><a href="https://www.w3.org/TR/css-font-loading-3/#dom-fontfacesetcheck-font">font</a></code> and <code><a href="https://www.w3.org/TR/css-font-loading-3/#dom-fontfacesetcheck-text">text</a></code> arguments passed to the function, and including system fonts, and let <var>font face list</var> be the returned list of font faces, and <var>found faces</var> be the returned <var>found faces</var> flag. If a syntax error was returned, throw a SyntaxError exception and terminate these steps.

3.  <a id="ref-for-dom-fontface-status①②"></a>

    If <var>font face list</var> is empty, or all fonts in the <var>font face list</var> either have a <code><a href="#dom-fontface-status">status</a></code> attribute of "loaded" or are system fonts, return `true`. Otherwise, return `false`.

### <a id="font-face-set-ready"></a>3.4.  The `ready` attribute

<a id="ref-for-dom-fontfaceset-ready①"></a>

<a id="ref-for-idl-promise⑧"></a>

Because the number of fonts loaded depends on the how many fonts are used for a given piece of text, in some cases whether fonts need to be loaded or not may not be known. The <code><a href="#dom-fontfaceset-ready">ready</a></code> attribute contains a <code><a href="https://webidl.spec.whatwg.org/#idl-promise">Promise</a></code> which is resolved when the document is done loading fonts, which provides a way for authors to avoid having to keep track of which fonts have or haven’t been loaded before examining content which may be affected by loading fonts.

<a id="ref-for-eventdef-fontfaceset-loadingdone②"></a>

<a id="ref-for-dom-fontfaceset-ready②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Authors should note that a given <var>ready promise</var> is only fulfilled once, but further fonts may be loaded after it fulfills. This is similar to listening for a <code><a href="#eventdef-fontfaceset-loadingdone">loadingdone</a></code> event to fire, but the callbacks passed to the <code><a href="#dom-fontfaceset-ready">ready</a></code> promise will <strong>always</strong> get called, even when no font loads occur because the fonts in question are already loaded. It’s a simple, easy way to synchronize code to font loads without the need to keep track of what fonts are needed and precisely when they load.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that the user agent may need to iterate over multiple font loads before the <var>ready promise</var> is fulfilled. This can occur with font fallback situations, where one font in the fontlist is loaded but doesn’t contain a particular glyph and other fonts in the fontlist need to be loaded. The <var>ready promise</var> is only fulfilled after layout operations complete and no additional font loads are necessary.

<a id="ref-for-dom-fontfaceset-ready③"></a>

<a id="ref-for-fontface④①"></a>

<a id="ref-for-dom-fontface-load⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that the Promise returned by this <code><a href="#dom-fontfaceset-ready">ready</a></code> attribute is only ever fulfilled, never rejected, unlike the Promise returned by the <code><a href="#fontface">FontFace</a></code> <code><a href="#dom-fontface-load">load()</a></code> method.

### <a id="font-face-set-css"></a>3.5.  Interaction with CSS Font Loading and Matching

<a id="ref-for-font-source③"></a>

When the font matching algorithm in [\[CSS-FONTS-3\]](#biblio-css-fonts-3) is run automatically by the user-agent, the set of font faces it matches over must be precisely the set of fonts in the [font source](#font-source) for the document, plus any local font faces.

<a id="ref-for-dom-fontface-load⑥"></a>

<a id="ref-for-fontface④②"></a>

When a user-agent needs to load a font face, it must do so by calling the <code><a href="#dom-fontface-load">load()</a></code> method of the corresponding <code><a href="#fontface">FontFace</a></code> object.

(This means it must run the same algorithm, not literally call the value currently stored in the `load` property of the object.)

<a id="ref-for-fontfaceset④⑦"></a>

<a id="ref-for-at-font-face-rule②②"></a>

<a id="ref-for-fontface④③"></a>

<a id="ref-for-fontfaceset④⑧"></a>

<a id="ref-for-document"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-00e0f107"></a> Fonts are available when they are added to a <code><a href="#fontfaceset">FontFaceSet</a></code>. Adding a new [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule to a stylesheet also adds a new <code><a href="#fontface">FontFace</a></code> to the <code><a href="#fontfaceset">FontFaceSet</a></code> of the <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> object.
>
> <a id="ref-for-at-font-face-rule②③"></a>
>
> Adding a new [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule:
>
> ```text
> document.styleSheets[0].insertRule(
>   "@font-face { font-family: newfont; src: url(newfont.woff); }", 0);
> document.body.style.fontFamily = "newfont, serif";
> ```
>
> <a id="ref-for-fontface④④"></a>
>
> Constructing a new <code><a href="#fontface">FontFace</a></code> object and adding it to `document.fonts`:
>
> ```text
> var f = new FontFace("newfont", "url(newfont.woff)");
> document.fonts.add(f);
> document.body.style.fontFamily = "newfont, serif";
> ```
>
> <a id="ref-for-at-font-face-rule②④"></a>
>
> In both cases, the loading of the font resource “newfont.woff” will be initiated by the layout engine, just as other [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule fonts are loaded.
>
> Omitting the addition to `document.fonts` means the font would never be loaded and text would be displayed in the default serif font:
>
> ```text
> var f = new FontFace("newfont", "url(newtest.woff)", {});
> 
> /* new {{FontFace}} not added to {{FontFaceSet}},
>    so the 'font-family' property can’t see it,
>    and serif will be used instead */
> document.body.style.fontFamily = "newfont, serif";
> ```
>
> <a id="ref-for-fontface④⑤"></a>
>
> <a id="ref-for-fontfaceset④⑨"></a>
>
> To explicitly preload a font before using it, authors can defer the addition of a new <code><a href="#fontface">FontFace</a></code> to a <code><a href="#fontfaceset">FontFaceSet</a></code> until the load has completed:
>
> ```text
> var f = new FontFace("newfont", "url(newfont.woff)", {});
> f.load().then(function (loadedFace) {
>   document.fonts.add(loadedFace);
>   document.body.style.fontFamily = "newfont, serif";
> });
> ```
>
> <a id="ref-for-fontfaceset⑤⓪"></a>
>
> In this case, the font resource “newfont.woff” is first downloaded. Once the download completes, the font is added to the document’s <code><a href="#fontfaceset">FontFaceSet</a></code>, the body font is changed, and the layout engine uses the new font resource.

## <a id="font-face-source"></a>4.  The `FontFaceSource` Mixin

<a id="fontfacesource"></a>

<a id="ref-for-fontfaceset⑤①"></a>

<a id="dom-fontfacesource-fonts"></a>

<a id="ref-for-document①"></a>

<a id="ref-for-fontfacesource"></a>

<a id="ref-for-workerglobalscope"></a>

<a id="ref-for-fontfacesource①"></a>

```text
interface mixin FontFaceSource {
  readonly attribute FontFaceSet fonts;
};

Document includes FontFaceSource;
WorkerGlobalScope includes FontFaceSource;
```
<a id="ref-for-fontfacesource②"></a>

<a id="ref-for-dom-fontfacesource-fonts"></a>

<a id="ref-for-font-source④"></a>

Any document, workers, or other context which can use fonts in some manner must include the <code><a href="#fontfacesource">FontFaceSource</a></code> mixin. The value of the context’s <code><a href="#dom-fontfacesource-fonts">fonts</a></code> attribute is its <a id="font-source"></a>font source, which provides all of the fonts used in font-related operations, unless defined otherwise. Operations referring to “the font source” must be interpreted as referring to the [font source](#font-source) of the relevant context in which the operation is taking place.

<a id="ref-for-fontface④⑥"></a>

<a id="ref-for-font-source⑤"></a>

For any font-related operation that takes place within one of these contexts, the <code><a href="#fontface">FontFace</a></code> objects within the [font source](#font-source) are its <a id="available-font-faces"></a>available font faces.

### <a id="fontfacesource-workers"></a>4.1.  Worker FontFaceSources

<a id="ref-for-font-source⑥"></a>

Within a Worker document, the [font source](#font-source) is initially empty.

<a id="ref-for-fontface④⑦"></a>

<a id="ref-for-offscreencanvas"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: <code><a href="#fontface">FontFace</a></code> objects can be constructed and added to it as normal, which affects CSS font-matching within the worker (such as, for example, drawing text into a <code><a href="https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas">OffscreenCanvas</a></code>).

<a id="ref-for-at-font-face-rule②⑤"></a>

### <a id="document-font-face-set"></a>4.2.  Interaction with CSS’s [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) Rule

<a id="ref-for-fontfaceset-set-entries⑤"></a>

<a id="ref-for-font-source⑦"></a>

<a id="ref-for-css-connected⑨"></a>

<a id="ref-for-fontface④⑧"></a>

<a id="ref-for-at-font-face-rule②⑥"></a>

<a id="ref-for-documentorshadowroot-document-or-shadow-root-css-style-sheets"></a>

<a id="ref-for-fontface④⑨"></a>

The [set entries](#fontfaceset-set-entries) for a document’s [font source](#font-source) must be initially populated with all the [CSS-connected](#css-connected) <code><a href="#fontface">FontFace</a></code> objects from all of the CSS [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rules in the [document or shadow root CSS style sheets](https://www.w3.org/TR/cssom-1/#documentorshadowroot-document-or-shadow-root-css-style-sheets), in document order. As <a id="ref-for-at-font-face-rule②⑦"></a>@font-face rules are added or removed from a stylesheet, or stylesheets containing <a id="ref-for-at-font-face-rule②⑧"></a>@font-face rules are added or removed, the corresponding <a id="ref-for-css-connected①⓪"></a>CSS-connected <code><a href="#fontface">FontFace</a></code> objects must be added or removed from the document’s <a id="ref-for-font-source⑧"></a>font source, and maintain this ordering.

<a id="ref-for-fontface⑤⓪"></a>

<a id="ref-for-css-connected①①"></a>

Any manually-added <code><a href="#fontface">FontFace</a></code> objects must be ordered <em>after</em> the [CSS-connected](#css-connected) ones.

<a id="ref-for-fontfaceset⑤②"></a>

<a id="ref-for-dom-fontfaceset-add②"></a>

<a id="ref-for-css-connected①②"></a>

<a id="ref-for-fontface⑤①"></a>

<a id="ref-for-invalidmodificationerror①"></a>

When a <code><a href="#fontfaceset">FontFaceSet</a></code> object’s <code><a href="#dom-fontfaceset-add">add()</a></code> method is called with a [CSS-connected](#css-connected) <code><a href="#fontface">FontFace</a></code> object, if the object is already in the set, the operation must be a no-op; otherwise, the operation must do nothing, and throw an <code><a href="https://webidl.spec.whatwg.org/#invalidmodificationerror">InvalidModificationError</a></code>.

<a id="ref-for-fontfaceset⑤③"></a>

<a id="ref-for-dom-fontfaceset-delete②"></a>

<a id="ref-for-css-connected①③"></a>

<a id="ref-for-fontface⑤②"></a>

When a <code><a href="#fontfaceset">FontFaceSet</a></code> object’s <code><a href="#dom-fontfaceset-delete">delete()</a></code> method is called with a [CSS-connected](#css-connected) <code><a href="#fontface">FontFace</a></code> object, the operation must be a no-op, and return `false`.

<a id="ref-for-fontface⑤③"></a>

<a id="ref-for-font-source⑨"></a>

<a id="ref-for-fontface⑤④"></a>

<a id="ref-for-css-connected①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Authors can still maintain references to a removed <code><a href="#fontface">FontFace</a></code>, even if it’s been automatically removed from a [font source](#font-source). As specified in [§ 2.3 Interaction with CSS’s @font-face Rule](#font-face-css-connection), though, the <code><a href="#fontface">FontFace</a></code> is no longer [CSS-connected](#css-connected) at that point.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is expected that a future version of this specification will define ways of interacting with and querying local fonts as well.

## <a id="font-load-event-examples"></a>5.  API Examples

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c4045522"></a> To show content only after all font loads complete:
>
> ```text
> document.fonts.ready.then(function() {
>   var content = document.getElementById("content");
>   content.style.visibility = "visible";
> });
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-95b58c34"></a> Drawing text in a canvas with a downloadable font, explicitly initiating the font download and drawing upon completion:
>
> ```text
> function drawStuff() {
>   var ctx = document.getElementById("c").getContext("2d");
> 
>   ctx.fillStyle = "red";
>   ctx.font = "50px MyDownloadableFont";
>   ctx.fillText("Hello!", 100, 100);
> }
> 
> document.fonts.load("50px MyDownloadableFont")
>               .then(drawStuff, handleError);
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8e44fe29"></a> A rich text editing application may need to measure text elements after editing operations have taken place. Since style changes may or may not require additional fonts to be downloaded, or the fonts may already have been downloaded, the measurement procedures need to occur after those font loads complete:
>
> ```text
> function measureTextElements() {
>   // contents can now be measured using the metrics of
>   // the downloadable font(s)
> }
> 
> function doEditing() {
>   // content/layout operations that may cause additional font loads
>   document.fonts.ready.then(measureTextElements);
> }
> ```
<a id="ref-for-eventdef-fontfaceset-loadingdone③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-268ae257"></a> The <code><a href="#eventdef-fontfaceset-loadingdone">loadingdone</a></code> event only fires after all font related loads have completed <strong>and</strong> text has been laid out without causing additional font loads:
>
> ```text
> <style>
> @font-face {
>   font-family: latin-serif;
>   src: url(latinserif.woff) format("woff"); /* contains no kanji/kana */
> }
> @font-face {
>   font-family: jpn-mincho;
>   src: url(mincho.woff) format("woff");
> }
> @font-face {
>   font-family: unused;
>   src: url(unused.woff);
> }
> 
> body { font-family: latin-serif, jpn-mincho; }
> </style>
> <p>納豆はいかがでしょうか
> ```
>
> <a id="ref-for-eventdef-fontfaceset-loadingdone④"></a>
>
> In this situation, the user agent first downloads “latinserif.woff” and then tries to use this to draw the Japanese text. But because no Japanese glyphs are present in that font, fallback occurs and the font “mincho.woff” is downloaded. Only after the second font is downloaded and the Japanese text laid out does the <code><a href="#eventdef-fontfaceset-loadingdone">loadingdone</a></code> event fire.
>
> <a id="ref-for-eventdef-fontfaceset-loadingdone⑤"></a>
>
> The "unused" font isn’t loaded, but no text is using it, so the UA isn’t even <em>trying</em> to load it. It doesn’t interfere with the <code><a href="#eventdef-fontfaceset-loadingdone">loadingdone</a></code> event.

## <a id="changes"></a>Changes

Changes from the [May 2014 CSS Font Loading Last Call Working Draft](https://www.w3.org/TR/2014/WD-css-font-loading-3-20140522/):

- Added IDL for discovery of font information.

- Clarified that FontFaceSet.clear() does not clear CSS-connected items.

- Mentioed document.fonts in the introduction.

- Font loading applies to shadow roots as well as documents.

- No longer throw an error if none of the specified fonts exist, because this will not trigger any font loading.

- Better alignment with WebIDL.

- Switched to constructor() method syntax.

- Clarified behavior of the matching font faces algorithm if the string to match is empty.

- Converted FontFaceSource to a mixin.

- Legacy term CanvasProxy changed to OffscreenCanvas.

- Harmonized FontFace with @font-face, adding variationSettings and fontDisplay.

- Consistently use \[Exposed\] in the IDL.

- Prefer CSSOMString to DOMString

- Better introductory text for check()

- Clarified that layout operations which depend on recently-loaded fonts must be allowed to complete.

- Cover more edge cases when firing load events.

- Prefer async event queueing tasks over synchronous calls

- fonts.ready is a property, not a function.

- Differentiated between non-existing fonts, and fonts which exist but lack the required glyphs.

- Precisely listed order of steps for several methods

- Clarified handling of global keywords and relative values in the load() and check() functions.

- Parsing a src argument is the same as parsing a CSS @font-face src descriptor.

- Clarified that attempting to delete a CSS connected font has no effect, and returns false.

- Clarified that adding duplicate fonts has no effect.

- Clarified ordering of manually added FontFace object.

- Clarified that load events only include faces still present in the set.

- <a id="ref-for-at-font-face-rule②⑨"></a>

  <a id="ref-for-dom-fontface-display①"></a>

  <a id="ref-for-dom-fontface-variationsettings①"></a>

  Added <code><a href="#dom-fontface-variationsettings">variationSettings</a></code> and <code><a href="#dom-fontface-display">display</a></code>, to sync with [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule).

- <a id="ref-for-dom-fontfacesetloadevent-fontfaces①"></a>

  Switched <code><a href="#dom-fontfacesetloadevent-fontfaces">fontfaces</a></code> to be a FrozenArray, to match with proper IDL practice.

- Fire loading events and handle promises when a loading font is added to a FontFaceSet.

- Corrected the async algorithms to use "queue a task" language, to ensure that side-effect timing is well-defined.

- Updated several references to latest versions.

- Corrections to the IDL.

- Assorted typos and grammatical errors corrected.

## <a id="acknowledgments"></a> Acknowledgments

Several members of the Google Fonts team provided helpful feedback on font load events, as did Boris Zbarsky, Jonas Sicking and ms2ger.

## <a id="privacy"></a> Privacy Considerations

<a id="ref-for-fontfaceset⑤④"></a>

<a id="ref-for-at-font-face-rule③⓪"></a>

The <code><a href="#fontfaceset">FontFaceSet</a></code> object leaks information about the user’s installed fonts, but in the exact same way as the existing [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) rule; no new information is leaked, or in any appreciably easier manner.

## <a id="security"></a> Security Considerations

No security considerations have been raised against this specification.

## <a id="w3c-conformance"></a> Conformance

### <a id="w3c-conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="w3c-example"></a>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong> UAs MUST provide an accessible alternative. </strong>

### <a id="w3c-conformance-classes"></a> Conformance classes

Conformance to this specification is defined for three conformance classes:

style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS21/conform.html#style-sheet).

renderer  
A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

authoring tool  
A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="w3c-partial"></a> Partial implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](https://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="w3c-conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

### <a id="w3c-testing"></a> Non-experimental implementations

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [add(font)](#dom-fontfaceset-add), in § 3
- ascentOverride
  - [attribute for FontFace](#dom-fontface-ascentoverride), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-ascentoverride), in § 2
- [available font faces](#available-font-faces), in § 4
- [axisTag](#dom-fontfacevariationaxis-axistag), in § 2.4
- [BinaryData](#typedefdef-binarydata), in § 2
- [check(font)](#dom-fontfaceset-check), in § 3.3
- [check(font, text)](#dom-fontfaceset-check), in § 3.3
- [clear()](#dom-fontfaceset-clear), in § 3
- [constructor(family, source)](#dom-fontface-fontface), in § 2.1
- [constructor(family, source, descriptors)](#dom-fontface-fontface), in § 2.1
- [constructor(initialFaces)](#dom-fontfaceset-fontfaceset), in § 3
- [constructor(type)](#dom-fontfacesetloadevent-fontfacesetloadevent), in § 3
- [constructor(type, eventInitDict)](#dom-fontfacesetloadevent-fontfacesetloadevent), in § 3
- [CSS-connected](#css-connected), in § 2.3
- [\[\[Data\]\]](#dom-fontface-data-slot), in § 2
- [defaultValue](#dom-fontfacevariationaxis-defaultvalue), in § 2.4
- [delete(font)](#dom-fontfaceset-delete), in § 3
- descentOverride
  - [attribute for FontFace](#dom-fontface-descentoverride), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-descentoverride), in § 2
- display
  - [attribute for FontFace](#dom-fontface-display), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-display), in § 2
- ["error"](#dom-fontfaceloadstatus-error), in § 2
- [\[\[FailedFonts\]\]](#dom-fontfaceset-failedfonts-slot), in § 3
- [family](#dom-fontface-family), in § 2
- [features](#dom-fontface-features), in § 2.4
- featureSettings
  - [attribute for FontFace](#dom-fontface-featuresettings), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-featuresettings), in § 2
- [find the matching font faces](#find-the-matching-font-faces), in § 3.1
- [fire a font load event](#fire-a-font-load-event), in § 3.1
- [FontFace](#fontface), in § 2
- [FontFaceDescriptors](#dictdef-fontfacedescriptors), in § 2
- [FontFace(family, source)](#dom-fontface-fontface), in § 2.1
- [FontFace(family, source, descriptors)](#dom-fontface-fontface), in § 2.1
- [FontFaceFeatures](#fontfacefeatures), in § 2.4
- [FontFaceLoadStatus](#enumdef-fontfaceloadstatus), in § 2
- [FontFacePalette](#fontfacepalette), in § 2.4
- [FontFacePalettes](#fontfacepalettes), in § 2.4
- fontfaces
  - [attribute for FontFaceSetLoadEvent](#dom-fontfacesetloadevent-fontfaces), in § 3
  - [dict-member for FontFaceSetLoadEventInit](#dom-fontfacesetloadeventinit-fontfaces), in § 3
- [FontFaceSet](#fontfaceset), in § 3
- [FontFaceSet(initialFaces)](#dom-fontfaceset-fontfaceset), in § 3
- [FontFaceSetLoadEvent](#fontfacesetloadevent), in § 3
- [FontFaceSetLoadEventInit](#dictdef-fontfacesetloadeventinit), in § 3
- [FontFaceSetLoadEvent(type)](#dom-fontfacesetloadevent-fontfacesetloadevent), in § 3
- [FontFaceSetLoadEvent(type, eventInitDict)](#dom-fontfacesetloadevent-fontfacesetloadevent), in § 3
- [FontFaceSetLoadStatus](#enumdef-fontfacesetloadstatus), in § 3
- [FontFaceSource](#fontfacesource), in § 4
- [FontFaceVariationAxis](#fontfacevariationaxis), in § 2.4
- [FontFaceVariations](#fontfacevariations), in § 2.4
- [fonts](#dom-fontfacesource-fonts), in § 4
- [font source](#font-source), in § 4
- [\[\[FontStatusPromise\]\]](#dom-fontface-fontstatuspromise-slot), in § 2
- [iteration order](#fontfaceset-iteration-order), in § 3
- length
  - [attribute for FontFacePalette](#dom-fontfacepalette-length), in § 2.4
  - [attribute for FontFacePalettes](#dom-fontfacepalettes-length), in § 2.4
- lineGapOverride
  - [attribute for FontFace](#dom-fontface-linegapoverride), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-linegapoverride), in § 2
- [load()](#dom-fontface-load), in § 2.2
- "loaded"
  - [enum-value for FontFaceLoadStatus](#dom-fontfaceloadstatus-loaded), in § 2
  - [enum-value for FontFaceSetLoadStatus](#dom-fontfacesetloadstatus-loaded), in § 3
- [loaded](#dom-fontface-loaded), in § 2
- [\[\[LoadedFonts\]\]](#dom-fontfaceset-loadedfonts-slot), in § 3
- [load(font)](#dom-fontfaceset-load), in § 3.2
- [load(font, text)](#dom-fontfaceset-load), in § 3.2
- "loading"
  - [enum-value for FontFaceLoadStatus](#dom-fontfaceloadstatus-loading), in § 2
  - [enum-value for FontFaceSetLoadStatus](#dom-fontfacesetloadstatus-loading), in § 3
- [loading](#eventdef-fontfaceset-loading), in § 3.1
- [loadingdone](#eventdef-fontfaceset-loadingdone), in § 3.1
- [loadingerror](#eventdef-fontfaceset-loadingerror), in § 3.1
- [\[\[LoadingFonts\]\]](#dom-fontfaceset-loadingfonts-slot), in § 3
- [maximumValue](#dom-fontfacevariationaxis-maximumvalue), in § 2.4
- [minimumValue](#dom-fontfacevariationaxis-minimumvalue), in § 2.4
- [name](#dom-fontfacevariationaxis-name), in § 2.4
- [onloading](#dom-fontfaceset-onloading), in § 3
- [onloadingdone](#dom-fontfaceset-onloadingdone), in § 3
- [onloadingerror](#dom-fontfaceset-onloadingerror), in § 3
- [palettes](#dom-fontface-palettes), in § 2.4
- [pending on the environment](#fontfaceset-pending-on-the-environment), in § 3
- [ready](#dom-fontfaceset-ready), in § 3
- [\[\[ReadyPromise\]\]](#dom-fontfaceset-readypromise-slot), in § 3
- [set entries](#fontfaceset-set-entries), in § 3
- status
  - [attribute for FontFace](#dom-fontface-status), in § 2
  - [attribute for FontFaceSet](#dom-fontfaceset-status), in § 3
- stretch
  - [attribute for FontFace](#dom-fontface-stretch), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-stretch), in § 2
- [stuck on the environment](#fontfaceset-stuck-on-the-environment), in § 3.1
- style
  - [attribute for FontFace](#dom-fontface-style), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-style), in § 2
- [switch the FontFaceSet to loaded](#switch-the-fontfaceset-to-loaded), in § 3.1
- [switch the FontFaceSet to loading](#switch-the-fontfaceset-to-loading), in § 3.1
- unicodeRange
  - [attribute for FontFace](#dom-fontface-unicoderange), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-unicoderange), in § 2
- ["unloaded"](#dom-fontfaceloadstatus-unloaded), in § 2
- [\[\[Urls\]\]](#dom-fontface-urls-slot), in § 2
- [usableWithDarkBackground](#dom-fontfacepalette-usablewithdarkbackground), in § 2.4
- [usableWithLightBackground](#dom-fontfacepalette-usablewithlightbackground), in § 2.4
- variant
  - [attribute for FontFace](#dom-fontface-variant), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-variant), in § 2
- [variations](#dom-fontface-variations), in § 2.4
- variationSettings
  - [attribute for FontFace](#dom-fontface-variationsettings), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-variationsettings), in § 2
- weight
  - [attribute for FontFace](#dom-fontface-weight), in § 2
  - [dict-member for FontFaceDescriptors](#dom-fontfacedescriptors-weight), in § 2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-FONT-LOADING-3\] defines the following terms:
  - <a id="b15033e0d8941fc6d4d813fb4bfd70de"></a>font (for FontFaceSet/check())
  - <a id="a169359ce4004ead53498fcaa57f2175"></a>font (for FontFaceSet/load())
  - <a id="8368c780cf1ec86fb315ef3c66d830f9"></a>text (for FontFaceSet/check())
  - <a id="42d11e56aa6f61a469371bbba0da08f9"></a>text (for FontFaceSet/load())
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="f8ec1b0628609748801d8852f3243ecd"></a>@font-palette-values
  - <a id="e061aca0cdbe40471d800a054afb455a"></a>bolder
  - <a id="06506d382c7460bf0f87ffebaf5b5229"></a>font
  - <a id="cdf3e67c50d7403bc81e1deab67ce369"></a>font-feature-settings
  - <a id="d8df72ab4b8593beaac60cd53c387207"></a>font-variation-settings
  - <a id="4f51fd2fb2f6770dc627e85e59b18e57"></a>normal
  - <a id="4ca7e27c87f5795144225e3af7041789"></a>src
  - <a id="61f8df02e32bd7381fe7e82e636ebfe4"></a>unicode-range
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="8c56fc0f83724ef0605ff1fe2ffb2947"></a>@font-face
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="d8e804209f4df90e5e0f3aa45c3c2aa4"></a>parse
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="f014258978db0b5cb28d306e0a9068b1"></a>css-wide keywords
  - <a id="cadc3fea0de07940b2222e1a6a2dbc2d"></a>url()
- \[CSSOM-1\] defines the following terms:
  - <a id="04e9b32172136734191119e0f5813089"></a>CSSOMString
  - <a id="be2edaeed28dfff19fd4f4fdc2d522cd"></a>document or shadow root css style sheets
- \[DOM\] defines the following terms:
  - <a id="07886e33695d0e7b473ef18f656ad7ac"></a>Document
  - <a id="316772b0029bcc028521d2db08fd3717"></a>Event
  - <a id="a41247dbf908b8c39ef1dea1be6618d3"></a>EventInit
  - <a id="a6aa66e0075314e8eefa4e662cb40fbb"></a>EventTarget
- \[HTML\] defines the following terms:
  - <a id="716703f1068cc638b4235f83154eff6c"></a>EventHandler
  - <a id="189fbd40e7b95b2b3b92b830f22f153f"></a>OffscreenCanvas
  - <a id="569dd106ffc76704c49d49a9087ebb4e"></a>WorkerGlobalScope
- \[WEBIDL\] defines the following terms:
  - <a id="8e4f03edcfe9224009a4032bac512ab6"></a>ArrayBuffer
  - <a id="4aba6698bcd0fab4456b4ceb06bdc7f0"></a>ArrayBufferView
  - <a id="d0f238a61de9310f1738a214749fab18"></a>DOMString
  - <a id="dde049eb112f043fc45407ec0d4cb457"></a>Exposed
  - <a id="30e8367dc9a547b9096d98ba691987d9"></a>FrozenArray
  - <a id="153343972687752b6534c28bd811972c"></a>InvalidModificationError
  - <a id="93669c631ec1ef8e696b4c083bdbe12c"></a>Promise
  - <a id="f572762cd4679aeacd36722d99d2b741"></a>SameObject
  - <a id="fb01fee229e88e7f1925373838fe6d85"></a>SyntaxError
  - <a id="f8062b07e25ed0281f5cbfb9a54070b4"></a>boolean
  - <a id="d3fd0dc5b12c87e613395fec1824ac77"></a>double
  - <a id="38ab7db52e46bed36993204c29f67202"></a>sequence
  - <a id="957360269b9570b56c5428a79dcbea77"></a>set entries
  - <a id="cee863f1f2ec91066cfb7f5d309d59dc"></a>undefined
  - <a id="03fa8b99cc63b35d2e7dcadb686676f1"></a>unsigned long

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-font-loading-3"></a>\[CSS-FONT-LOADING-3\]  
Tab Atkins Jr.. [CSS Font Loading Module Level 3](https://www.w3.org/TR/css-font-loading-3/). 22 May 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-font-loading-3&#x2F;](https://www.w3.org/TR/css-font-loading-3/)

<a id="biblio-css-fonts-3"></a>\[CSS-FONTS-3\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-fonts-5"></a>\[CSS-FONTS-5\]  
Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 5](https://www.w3.org/TR/css-fonts-5/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-5&#x2F;](https://www.w3.org/TR/css-fonts-5/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 19 October 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

## <a id="idl-index"></a>IDL Index

```text
typedef (ArrayBuffer or ArrayBufferView) BinaryData;

dictionary FontFaceDescriptors {
  CSSOMString style = "normal";
  CSSOMString weight = "normal";
  CSSOMString stretch = "normal";
  CSSOMString unicodeRange = "U+0-10FFFF";
  CSSOMString variant = "normal";
  CSSOMString featureSettings = "normal";
  CSSOMString variationSettings = "normal";
  CSSOMString display = "auto";
  CSSOMString ascentOverride = "normal";
  CSSOMString descentOverride = "normal";
  CSSOMString lineGapOverride = "normal";
};

enum FontFaceLoadStatus { "unloaded", "loading", "loaded", "error" };

[Exposed=(Window,Worker)]
interface FontFace {
  constructor(CSSOMString family, (CSSOMString or BinaryData) source,
                optional FontFaceDescriptors descriptors = {});
  attribute CSSOMString family;
  attribute CSSOMString style;
  attribute CSSOMString weight;
  attribute CSSOMString stretch;
  attribute CSSOMString unicodeRange;
  attribute CSSOMString variant;
  attribute CSSOMString featureSettings;
  attribute CSSOMString variationSettings;
  attribute CSSOMString display;
  attribute CSSOMString ascentOverride;
  attribute CSSOMString descentOverride;
  attribute CSSOMString lineGapOverride;

  readonly attribute FontFaceLoadStatus status;

  Promise<FontFace> load();
  readonly attribute Promise<FontFace> loaded;
};

[Exposed=(Window,Worker)]
interface FontFaceFeatures {
  /* The CSSWG is still discussing what goes in here */
};

[Exposed=(Window,Worker)]
interface FontFaceVariationAxis {
  readonly attribute DOMString name;
  readonly attribute DOMString axisTag;
  readonly attribute double minimumValue;
  readonly attribute double maximumValue;
  readonly attribute double defaultValue;
};

[Exposed=(Window,Worker)]
interface FontFaceVariations {
  readonly setlike<FontFaceVariationAxis>;
};

[Exposed=(Window,Worker)]
interface FontFacePalette {
  iterable<DOMString>;
  readonly attribute unsigned long length;
  getter DOMString (unsigned long index);
  readonly attribute boolean usableWithLightBackground;
  readonly attribute boolean usableWithDarkBackground;
};

[Exposed=(Window,Worker)]
interface FontFacePalettes {
  iterable<FontFacePalette>;
  readonly attribute unsigned long length;
  getter FontFacePalette (unsigned long index);
};

partial interface FontFace {
  readonly attribute FontFaceFeatures features;
  readonly attribute FontFaceVariations variations;
  readonly attribute FontFacePalettes palettes;
};

dictionary FontFaceSetLoadEventInit : EventInit {
  sequence<FontFace> fontfaces = [];
};

[Exposed=(Window,Worker)]
interface FontFaceSetLoadEvent : Event {
  constructor(CSSOMString type, optional FontFaceSetLoadEventInit eventInitDict = {});
  [SameObject] readonly attribute FrozenArray<FontFace> fontfaces;
};

enum FontFaceSetLoadStatus { "loading", "loaded" };

[Exposed=(Window,Worker)]
interface FontFaceSet : EventTarget {
  constructor(sequence<FontFace> initialFaces);

  setlike<FontFace>;
  FontFaceSet add(FontFace font);
  boolean delete(FontFace font);
  undefined clear();

  // events for when loading state changes
  attribute EventHandler onloading;
  attribute EventHandler onloadingdone;
  attribute EventHandler onloadingerror;

  // check and start loads if appropriate
  // and fulfill promise when all loads complete
  Promise<sequence<FontFace>> load(CSSOMString font, optional CSSOMString text = " ");

  // return whether all fonts in the fontlist are loaded
  // (does not initiate load if not available)
  boolean check(CSSOMString font, optional CSSOMString text = " ");

  // async notification that font loading and layout operations are done
  readonly attribute Promise<FontFaceSet> ready;

  // loading state, "loading" while one or more fonts loading, "loaded" otherwise
  readonly attribute FontFaceSetLoadStatus status;
};

interface mixin FontFaceSource {
  readonly attribute FontFaceSet fonts;
};

Document includes FontFaceSource;
WorkerGlobalScope includes FontFaceSource;

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Several things in this spec use normal ES objects to define behavior, such as various things using Promises internally, and FontFaceSet using a Set internally. I believe the intention here is that these objects (and their prototype chains) are pristine, unaffected by anything the author has done. Is this a good intention? If so, how should I indicate this in the spec? [↵](#issue-9cdca628)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Clarify all mentions of "the document" to be clear about which document is being referenced, since objects can move between documents. [↵](#issue-9ae72be4)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Need to define the base url, so relative urls can resolve. Should it be the url of the document? Is that correct for workers too, or should they use their worker url? Is that always defined? [↵](#issue-65bdbf2d)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> When a FontFace is transferred between documents, it’s no longer CSS-connected. [↵](#issue-41199acf)
