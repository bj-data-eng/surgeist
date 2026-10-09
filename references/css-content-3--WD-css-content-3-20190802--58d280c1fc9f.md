Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Generated Content Module Level 3](https://www.w3.org/TR/2019/WD-css-content-3-20190802/).

Original copyright notice: Copyright © 2019 W3C ® ( MIT , ERCIM , Keio , Beihang ). W3C liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Generated Content Module Level 3

Source snapshot: https://www.w3.org/TR/2019/WD-css-content-3-20190802/

Snapshot SHA-256: 58d280c1fc9f8a9db3dd3dde589117bd2129002feb92486224e4c984ff4cceaa

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Generated Content Module Level 3

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2019 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This CSS3 Module describes how to insert content in a document.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of&#xA;   its publication. Other documents may supersede this document. A list of&#xA;   current W3C publications and the latest revision of this technical report&#xA;   can be found in the <a href="https://www.w3.org/TR/">W3C technical reports&#xA;   index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

Publication as a Working Draft does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

[GitHub Issues](https://github.com/w3c/csswg-drafts/issues) are preferred for discussion of this specification. When filing an issue, please put the text “css-content” in the title, preferably like this: “\[css-content\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/), and there is also a [historical archive](https://lists.w3.org/Archives/Public/www-style/).

This document was produced by the [CSS Working Group](https://www.w3.org/Style/CSS/members).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/#sec-Disclosure).

<a id="w3c_process_revision"></a>

This document is governed by the [1 March 2019 W3C Process Document](https://www.w3.org/2019/Process-20190301/).

This is a very rough draft, and is not ready for implementation.

## <a id="intro"></a> Introduction

Authors sometimes want user agents to render content that does not come from the document tree. One familiar example of this is numbered headings; the author does not want to mark the numbers up explicitly, they want the user agent to generate them automatically. Counters and markers are used to achieve these effects.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-47d87e9c"></a>
>
> ```text
> h1::before { content: counter(section) ": "; }
> ```
Similarly, authors may want the user agent to insert the word "Figure" before the caption of a figure, or "Chapter 7" on a line before the seventh chapter title.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5a4a3cc1"></a>
>
> ```text
> chapter { counter-increment: chapter; }
> chapter > title::before { content: "Chapter " counter(chapter) "\A"; }
> ```
Another common effect is replacing elements with images or other multimedia content. Since not all user agents support all multimedia formats, fallbacks may have to be provided.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-856d72ea"></a>
>
> ```text
> /* Replace <logo> elements with the site’s logo, using a format
>  * supported by the UA */
> logo { content: url(logo.mov), url(logo.mng), url(logo.png), none; }
> 
> /* Replace <figure> elements with the referenced document, or,
>  * failing that, with either the contents of the alt attribute or the
>  * contents of the element itself if there is no alt attribute */
> figure[alt] { content: attr(href url), attr(alt); }
> figure:not([alt]) { content: attr(href url), contents; }
> ```
<a id="ref-for-propdef-content"></a>

## <a id="content-property"></a>1.  Inserting and replacing content with the [content](#propdef-content) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-content"></a>content                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://drafts.csswg.org/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt"></a><a id="ref-for-mult-one-plus"></a><a id="ref-for-typedef-counter"></a><a id="ref-for-string-value"></a><a id="ref-for-typedef-content-content-list"></a><a id="ref-for-typedef-content-content-replacement"></a><a id="ref-for-comb-one"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) none <a id="ref-for-comb-one①"></a>\| \[ [\<content-replacement\>](#typedef-content-content-replacement) <a id="ref-for-comb-one②"></a>\| [\<content-list\>](#typedef-content-content-list) \] \[/ \[ [\<string\>](https://www.w3.org/TR/css3-values/#string-value) <a id="ref-for-comb-one③"></a>\| [\<counter\>](https://www.w3.org/TR/css-lists-3/#typedef-counter) \][+](https://www.w3.org/TR/css-values-4/#mult-one-plus) \][?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://drafts.csswg.org/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong>Applies to:&#xA;      </strong> | all elements, tree-abiding pseudo-elements, and page margin boxes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://drafts.csswg.org/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://drafts.csswg.org/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://drafts.csswg.org/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | See prose below                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://drafts.csswg.org/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |



User Agents are expected to support this property on all media, including non-visual ones.

<a id="ref-for-propdef-content①"></a>

The [content](#propdef-content) property dictates what is rendered inside an element or pseudo-element.

For elements, it has only one purpose: specifying that the element renders as normal, or replacing the element with an image (and possibly some associated "alt text").

For pseudo-elements and margin boxes, it is more powerful. It controls whether the element renders at all, can replace the element with an image, or replace it with arbitrary inline content (text and images).

<a id="valdef-content-normal"></a>normal

<a id="ref-for-valdef-content-contents"></a>

For an element or page margin box, this computes to [contents](#valdef-content-contents).

<a id="ref-for-selectordef-before"></a>

<a id="ref-for-selectordef-after"></a>

<a id="ref-for-valdef-content-none"></a>

For [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) and [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after), this computes to [none](#valdef-content-none).

<a id="ref-for-selectordef-marker"></a>

<a id="ref-for-valdef-content-normal"></a>

For [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker), this computes to itself ([normal](#valdef-content-normal)).

<a id="valdef-content-none"></a>none

On elements, this inhibits the children of the element from being rendered as children of this element, as if the element was empty.

<a id="ref-for-propdef-display"></a>

On pseudo-elements it inhibits the creation of the pseudo-element as if it had [display: none](https://www.w3.org/TR/css-ruby-1/#propdef-display).

<a id="ref-for-originating-element"></a>

In neither case does it prevent any pseudo-elements which have this element or pseudo-element as an [originating element](https://www.w3.org/TR/selectors4/#originating-element) from being generated.

<a id="ref-for-typedef-content-content-replacement①"></a>

<a id="typedef-content-content-replacement"></a>[\<content-replacement\>](#typedef-content-content-replacement)

<a id="replaced"></a> Equal to:

<a id="ref-for-image-type"></a>

```text
<image>
```
<a id="ref-for-image-type①"></a>

<a id="ref-for-propdef-display①"></a>

Makes the element or pseudo-element a [replaced element](https://www.w3.org/TR/CSS2/conform.html#replaced-element), filled with the specified [\<image\>](https://www.w3.org/TR/css3-images/#image-type). Its normal contents are suppressed and do not generate boxes, as if they were [display: none](https://www.w3.org/TR/css-ruby-1/#propdef-display).

<a id="ref-for-image-type②"></a>

<a id="ref-for-invalid-image"></a>

If the [\<image\>](https://www.w3.org/TR/css3-images/#image-type) represents an [invalid image](https://www.w3.org/TR/css4-images/#invalid-image), then it must be treated as instead representing an image with zero intrinsic width and height, filled with transparent black.

<a id="ref-for-invalid-image①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-b8fd18a9"></a> The above [invalid image](https://www.w3.org/TR/css4-images/#invalid-image) behavior appears to be what Chrome is doing. Is this okay? Is there a better behavior we can/should use?

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-76d40dad"></a> "Zero intrinsic width and height" gives it an undefined aspect ratio, and sizing behavior is thus... undefined. At least, if you give it an explicit width or height, the other dimension remains at zero in Chrome. This might need to be explicitly defined over in the Images spec; needs some investigation around whether browsers act this way for 0x0 SVGs and rasters.

<a id="ref-for-the-img-element"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Replaced elements use different layout rules than normal elements. (In effect, it becomes equivalent to an HTML <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> element.)

<a id="ref-for-selectordef-before①"></a>

<a id="ref-for-selectordef-after①"></a>

<a id="ref-for-propdef-content②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Replaced elements do not have [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) or [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudo-elements; the [content](#propdef-content) property replaces their entire contents.

<a id="ref-for-typedef-content-content-list①"></a>

<a id="typedef-content-content-list"></a>[\<content-list\>](#typedef-content-content-list)

Equal to:

<a id="ref-for-string-value①"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-image-type③"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-typedef-counter①"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-typedef-quote"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-typedef-target"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-mult-one-plus①"></a>

```text
[ <string> | contents | <image> | <counter> | <quote> | <target> | <leader()> ]+
```
<a id="ref-for-propdef-display②"></a>

Replaces the element’s contents with one or more anonymous inline boxes corresponding to the specified values, in the order specified. Its normal contents are suppressed and do not generate boxes, as if they were [display: none](https://www.w3.org/TR/css-ruby-1/#propdef-display).

<a id="ref-for-image-type④"></a>

Each value contributes an inline box to the element’s contents. For [\<image\>](https://www.w3.org/TR/css3-images/#image-type), this is an inline anonymous replaced element; for the others, it’s an anonymous inline run of text.

<a id="ref-for-image-type⑤"></a>

<a id="ref-for-invalid-image②"></a>

If an [\<image\>](https://www.w3.org/TR/css3-images/#image-type) represents an [invalid image](https://www.w3.org/TR/css4-images/#invalid-image), the user agent must do one of the following:

- <a id="ref-for-image-type⑥"></a>

  "Skip" the [\<image\>](https://www.w3.org/TR/css3-images/#image-type), generating nothing for it.

- <a id="ref-for-image-type⑦"></a>

  Display some indication that the image can’t be displayed in place of the [\<image\>](https://www.w3.org/TR/css3-images/#image-type), such as a "broken image" icon.

This specification intentionally does not define which behavior a user agent must use, but it must use one or the other consistently.

<a id="ref-for-typedef-content-content-list②"></a>

<a id="ref-for-image-type⑧"></a>

<a id="ref-for-typedef-content-content-replacement②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the value of [\<content-list\>](#typedef-content-content-list) is a single [\<image\>](https://www.w3.org/TR/css3-images/#image-type), it must instead be interpreted as a [\<content-replacement\>](#typedef-content-content-replacement).

<a id="ref-for-typedef-counter②"></a>

<a id="ref-for-string-value②"></a>

<a id="valdef-content---string--counter"></a>/ \[ [\<string\>](https://www.w3.org/TR/css3-values/#string-value) \| [\<counter\>](https://www.w3.org/TR/css-lists-3/#typedef-counter) \]+

Specifies the "alt text" for the element. See [§ 1.2 Alternative Text for Accessibility](#alt) for details. If omitted, the element has no "alt text".

<a id="ref-for-funcdef-content"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a8fbc981"></a> Should the contents keyword be replaced with [content()](#funcdef-content)?

### <a id="accessibility"></a>1.1.  Accessibility of Generated Content

<a id="ref-for-propdef-content③"></a>

Generated content should be searchable, selectable, and available to assistive technologies. The [content](#propdef-content) property applies to speech and generated content must be rendered for speech output. [\[CSS3-SPEECH\]](#biblio-css3-speech)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9c60e2c1"></a> Start work on an AAM for CSS.

### <a id="alt"></a>1.2.  Alternative Text for Accessibility

<a id="ref-for-propdef-content④"></a>

<a id="ref-for-typedef-content-content-list③"></a>

Content intended for visual media sometimes needs alternative text for speech output or other non-visual mediums. The [content](#propdef-content) property thus accepts alternative text to be specified after a slash (/) after the last [\<content-list\>](#typedef-content-content-list). If such alternative text is provided, it must be used for speech output instead.

This allows, for example, purely decorative text to be elided in speech output (by providing the empty string as alternative text), and allows authors to provide more readable alternatives to images, icons, or text-encoded symbols.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ff5073a7"></a> Here the content property is an image, so the alt value is required to provide alternative text.
>
> ```text
> .new::before {
>  content: url(./img/star.png) / "New!";
>   /* or a localized attribute from the DOM: attr("data-alt") */
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-954890b1"></a> If the pseudo-element is purely decorative and its function is covered elsewhere, setting alt to the empty string can avoid reading out the decorative element. Here the ARIA attribute will be spoken as "collapsed". Without the empty string alt value, the content would also be spoken as "Black right-pointing pointer".
>
> ```text
> .expandable::before {
>  content: "\25BA" / "";
> /* a.k.a. ► */
>  /* aria-expanded="false" already in DOM,
>    so this pseudo-element is decorative */
> }
> ```
<a id="ref-for-typedef-content-content-list④"></a>

## <a id="content-values"></a>2.  [\<content-list\>](#typedef-content-content-list) Values and Functions

<a id="ref-for-typedef-content-content-list⑤"></a>

<a id="ref-for-propdef-content⑤"></a>

The [\<content-list\>](#typedef-content-content-list) value is used in [content](#propdef-content) to fill an element with one or more anonymous inline boxes, including images, strings, the values of counters, and the text value of elements. In this section we enumerate the possibilities.

### <a id="strings"></a>2.1.  String

<a id="ref-for-string-value③"></a>

<a id="valdef-content-string"></a>[\<string\>](https://www.w3.org/TR/css3-values/#string-value)

Represents an anonymous inline box filled with the specified text.

<a id="ref-for-white-space"></a>

<a id="ref-for-propdef-content⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [White space](https://www.w3.org/TR/css-text-3/#white-space) in the string is handled the same as in literal text, and controlled by the properties in [\[CSS-TEXT-3\]](#biblio-css-text-3) and elsewhere. In particular, <a id="ref-for-white-space①"></a>white space character can collapse, even across multiple strings, such as in [content: "First " " Second";](#propdef-content), which by default will render similar to `"First Second"` (with a single visible space between the two words).

<a id="ref-for-image-type⑨"></a>

### <a id="content-uri"></a>2.2.  [\<image\>](https://www.w3.org/TR/css3-images/#image-type)

<a id="ref-for-image-type①⓪"></a>

<a id="valdef-content-list-image"></a>[\<image\>](https://www.w3.org/TR/css3-images/#image-type)

<a id="ref-for-image-type①①"></a>

Represents an anonymous inline replaced element filled with the specified [\<image\>](https://www.w3.org/TR/css3-images/#image-type).

<a id="ref-for-image-type①②"></a>

<a id="ref-for-invalid-image③"></a>

If the [\<image\>](https://www.w3.org/TR/css3-images/#image-type) represents an [invalid image](https://www.w3.org/TR/css4-images/#invalid-image), this value instead represents nothing. (No inline content is added to the element, as if this value were "skipped".)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3ef3081c"></a> CSS2.1 explicitly allowed the UA to substitute a broken image icon if the image was invalid. However, no browser appears to do this. Is this removal okay?

### <a id="element-content"></a>2.3.  Element Content

<a id="valdef-content-contents"></a>contents  
The element’s descendents. Since this can only be used once per element (you can’t duplicate the children if, e.g., one is a plugin or form control), it is handled as follows:

If set on the element:  
<a id="ref-for-propdef-content⑦"></a>

<a id="ref-for-valdef-content-normal①"></a>

<a id="ref-for-valdef-content-contents①"></a>

Always honoured. Note that this is the default, since the initial value of [content](#propdef-content) is [normal](#valdef-content-normal) and <a id="ref-for-valdef-content-normal②"></a>normal computes to [contents](#valdef-content-contents) on an element.

If set on one of the element’s other pseudo-elements:  
Check to see that it is not set on a "previous" pseudo-element, in the following order, depth first:

1.  the element itself

2.  ::before

3.  ::after

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-bf2f178e"></a> Should this behave as an empty string on pseudo-elements?

<a id="ref-for-valdef-content-none①"></a>

If it is already used, then it evaluates to nothing (like [none](#valdef-content-none)). Only pseudo-elements that are actually generated are checked.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-765061ba"></a> In the following case:
>
> ```text
> foo { content: normal; }  /* this is the initial value */
> foo::after { content: contents; }
> ```
>
> <a id="ref-for-propdef-content⑧"></a>
>
> <a id="ref-for-valdef-content-contents②"></a>
>
> <a id="ref-for-valdef-content-none②"></a>
>
> ...the element’s [content](#propdef-content) property would compute to [contents](#valdef-content-contents) and the after pseudo element would have no contents (equivalent to [none](#valdef-content-none)) and thus would not appear.
>
> ```text
> foo { content: none; }
> foo::after { content: contents; }
> ```
>
> But in this example, the ::after pseudo-element will contain the contents of the foo element.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-bfc60bbc"></a> Use cases for suppressing the content on the element and using it in a pseudo-element would be welcome.

<a id="ref-for-valdef-content-contents③"></a>

<a id="ref-for-propdef-content⑨"></a>

<a id="ref-for-selectordef-marker①"></a>

<a id="ref-for-valdef-content-none③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While it is useless to include [contents](#valdef-content-contents) twice in a single [content](#propdef-content) property, that is not a parse error. The second occurrence simply has no effect, as it has already been used. It is also not a parse error to use it on a [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element, it is only during the rendering stage that it gets treated like [none](#valdef-content-none).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9fa9cc25"></a> Do we need the statement about marker pseudo-elements here? Or is this legacy from the old version of the spec?

### <a id="quotes"></a>2.4.  Quotes

<a id="ref-for-propdef-content①⓪"></a>

HTML has long had the `q` element, used to delimit quotations. The quotes property, in conjunction with the various \*-quote values of the [content](#propdef-content) property, can be used to properly style such quotations.

<a id="ref-for-propdef-quotes"></a>

#### <a id="quotes-property"></a>2.4.1.  Specifying quotes with the [quotes](#propdef-quotes) property



| Field               | Definition                                                                                                                                                                                                                                                                                          |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-quotes"></a>quotes                                                                                                                                                                                                                                                                           |
| <strong><a href="https://drafts.csswg.org/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-one-plus②"></a><a id="ref-for-string-value④"></a><a id="ref-for-comb-one①⓪"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) none <a id="ref-for-comb-one①①"></a>\| \[ [\<string\>](https://www.w3.org/TR/css3-values/#string-value) <a id="ref-for-string-value⑤"></a>\<string\> \][+](https://www.w3.org/TR/css-values-4/#mult-one-plus) |
| <strong><a href="https://drafts.csswg.org/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                |
| <strong>Applies to:&#xA;      </strong> | [all elements](https://drafts.csswg.org/css-pseudo/#generated-content)                                                                                                                                                                                                                              |
| <strong><a href="https://drafts.csswg.org/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://drafts.csswg.org/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://drafts.csswg.org/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-quotes-auto"></a><a id="ref-for-valdef-quotes-none"></a>the keyword [none](#valdef-quotes-none), the keyword [auto](#valdef-quotes-auto), or a list, each item a pair of string values                                                                                                                                |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://drafts.csswg.org/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                            |



User Agents are expected to support this property on all media, including non-visual ones.

This property specifies quotation marks for any number of embedded quotations. Values have the following meanings:

<a id="valdef-quotes-none"></a>none

<a id="ref-for-valdef-content-no-close-quote"></a>

<a id="ref-for-valdef-content-no-open-quote"></a>

<a id="ref-for-propdef-content①①"></a>

<a id="ref-for-valdef-content-close-quote"></a>

<a id="ref-for-valdef-content-open-quote"></a>

The [open-quote](#valdef-content-open-quote) and [close-quote](#valdef-content-close-quote) values of the [content](#propdef-content) property produce no quotations marks, as if they were [no-open-quote](#valdef-content-no-open-quote) and [no-close-quote](#valdef-content-no-close-quote) respectively.

<a id="valdef-quotes-auto"></a>auto

<a id="ref-for-content-language"></a>

<a id="ref-for-propdef-quotes①"></a>

<a id="ref-for-used-value"></a>

A typographically appropriate [used value](https://www.w3.org/TR/css-cascade-4/#used-value) for [quotes](#propdef-quotes) is automatically chosen by the UA based on the [content language](https://www.w3.org/TR/css-text-3/#content-language) of the element and/or its parent.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The Unicode Common Locale Data Repository [\[CLDR\]](#biblio-cldr) maintains information on typographically apropriate quotation marks. UAs can use other sources of information as well, particularly as typographic preferences can vary; however it is encouraged to submit any improvements to Unicode so that the entire software ecosystem can benefit.

<a id="ref-for-string-value⑥"></a>

\[ [\<string\>](https://www.w3.org/TR/css3-values/#string-value) <a id="ref-for-string-value⑦"></a>\<string\> \]+

<a id="ref-for-propdef-content①②"></a>

<a id="ref-for-valdef-content-close-quote①"></a>

<a id="ref-for-valdef-content-open-quote①"></a>

Values for the [open-quote](#valdef-content-open-quote) and [close-quote](#valdef-content-close-quote) values of the [content](#propdef-content) property are taken from this list of pairs of quotation marks (opening and closing). The first (leftmost) pair represents the outermost level of quotation, the second pair the first level of embedding, etc. The user agent must apply the appropriate pair of quotation marks according to the level of embedding.

#### <a id="quote-values"></a>2.4.2.  The \*-quote values of the content property

<a id="typedef-quote"></a>

<a id="ref-for-typedef-quote①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-comb-one①④"></a>

```text
<quote> = open-quote | close-quote | no-open-quote | no-close-quote
```
<a id="valdef-content-open-quote"></a>open-quote  
<a id="valdef-content-close-quote"></a>close-quote  
<a id="ref-for-propdef-quotes②"></a>

These values are replaced by the appropriate string from the [quotes](#propdef-quotes) property, and increments (decrements) the level of nesting for quotes. See [§ 2.4.1 Specifying quotes with the quotes property](#quotes-property) for more information.

<a id="valdef-content-no-open-quote"></a>no-open-quote  
<a id="valdef-content-no-close-quote"></a>no-close-quote  
<a id="ref-for-valdef-content-none④"></a>

Inserts nothing (as in [none](#valdef-content-none)), but increments (decrements) the level of nesting for quotes. See [§ 2.4.1 Specifying quotes with the quotes property](#quotes-property) for more information.

<a id="ref-for-valdef-content-open-quote②"></a>

<a id="ref-for-valdef-content-close-quote②"></a>

<a id="ref-for-propdef-content①③"></a>

<a id="ref-for-propdef-quotes③"></a>

Quotation marks are inserted in appropriate places in a document with the [open-quote](#valdef-content-open-quote) and [close-quote](#valdef-content-close-quote) values of the [content](#propdef-content) property. Each occurrence of <a id="ref-for-valdef-content-open-quote③"></a>open-quote or <a id="ref-for-valdef-content-close-quote③"></a>close-quote is replaced by one of the strings from the value of [quotes](#propdef-quotes), based on the depth of nesting.

<a id="ref-for-valdef-content-open-quote④"></a>

<a id="ref-for-valdef-content-close-quote④"></a>

[open-quote](#valdef-content-open-quote) refers to the first of a pair of quotes, [close-quote](#valdef-content-close-quote) refers to the second. Which pair of quotes is used depends on the nesting level of quotes: the number of occurrences of <a id="ref-for-valdef-content-open-quote⑤"></a>open-quote in all generated text before the current occurrence, minus the number of occurrences of <a id="ref-for-valdef-content-close-quote⑤"></a>close-quote. If the depth is 0, the first pair is used, if the depth is 1, the second pair is used, etc. If the depth is greater than the number of pairs, the last pair is repeated.

Note that this quoting depth is independent of the nesting of the source document or the formatting structure.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Quote nesting, like [counter inheritance](https://drafts.csswg.org/css-lists-3/#inheriting-counters), operates on the “flattened element tree” in the context of the [\[DOM\]](#biblio-dom).

<a id="ref-for-valdef-content-no-close-quote①"></a>

Some typographic styles require open quotation marks to be repeated before every paragraph of a quote spanning several paragraphs, but only the last paragraph ends with a closing quotation mark. In CSS, this can be achieved by inserting "phantom" closing quotes. The keyword [no-close-quote](#valdef-content-no-close-quote) decrements the quoting level, but does not insert a quotation mark.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ebcea9ad"></a> The following style sheet puts opening quotation marks on every paragraph in a `blockquote`, and inserts a single closing quote at the end:
>
> ```text
> blockquote p:before { content: open-quote }
> blockquote p:after { content: no-close-quote }
> blockquote p:last-child::after { content: close-quote }
> ```
<a id="ref-for-valdef-content-no-open-quote①"></a>

For symmetry, there is also a [no-open-quote](#valdef-content-no-open-quote) keyword, which inserts nothing, but increments the quotation depth by one.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If a quotation is in a different language than the surrounding text, it is customary to quote the text with the quote marks of the language of the surrounding text, not the language of the quotation itself.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-95fc1afb"></a> For example, French inside English:
>
> > The device of the order of the garter is “Honi soit qui mal y pense.”
>
> English inside French:
>
> > Il disait: « Il faut mettre l’action en ‹ fast forward ›. »
>
> <a id="ref-for-propdef-quotes④"></a>
>
> <a id="ref-for-valdef-content-open-quote⑥"></a>
>
> <a id="ref-for-valdef-content-close-quote⑥"></a>
>
> A style sheet like the following will set the [quotes](#propdef-quotes) property so that [open-quote](#valdef-content-open-quote) and [close-quote](#valdef-content-close-quote) will work correctly on all elements. These rules are for documents that contain only English, French, or both. One rule is needed for every additional language. Note the use of the child combinator ("\>") to set quotes on elements based on the language of the surrounding text:
>
> ```text
> :lang(fr) > * { quotes: "\00AB\2005" "\2005\00BB" "\2039\2005" "\2005\203A" }
> :lang(en) > * { quotes: "\201C" "\201D" "\2018" "\2019" }
> ```
>
> The quotation marks are shown here in a form that most people will be able to type. If you can type them directly, they will look like this:
>
> ```text
> :lang(fr) > * { quotes: "« " " »" "‹ " " ›" }
> :lang(en) > * { quotes: "“" "”" "‘" "’" }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-eb11c28e"></a> For example, applying the following style sheet:
>
> ```text
> /* Specify pairs of quotes for two levels in two languages */
> :lang(en) > q { quotes: '"' '"' "'" "'" }
> :lang(no) > q { quotes: "«" "»" "’" "’" }
> 
> /* Insert quotes before and after Q element content */
> q::before { content: open-quote }
> q::after  { content: close-quote }
> ```
>
> to the following HTML fragment:
>
> ```text
> <html lang="en">
>   <head>
>    <title>Quotes</title>
>   </head>
>   <body>
>    <p><q>Quote me!</q></p>
>   </body>
> </html>
> ```
>
> would allow a user agent to produce:
>
> ```text
> "Quote me!"
> ```
>
> while this HTML fragment:
>
> ```text
> <html lang="no">
>   <head>
>    <title>Quotes</title>
>   </head>
>   <body>
>    <p><q>Trøndere gråter når <q>Vinsjan på kaia</q> blir deklamert.</q></p>
>   </body>
> </html>
> ```
>
> would produce:
>
> ```text
> «Trøndere gråter når ’Vinsjan på kaia’ blir deklamert.»
> ```
### <a id="leaders"></a>2.5.  Leaders

A leader, sometimes known as a tab leader or a dot leader, is a repeating pattern used to visually connect content across horizontal spaces. They are most commonly used in tables of contents, between titles and page numbers. The leader() function, as a value for the content property, is used to create leaders in CSS. This function takes a string (the leader string), which describes the repeating pattern for the leader.

#### <a id="leader-function"></a>2.5.1.  The leader() function

<a id="ref-for-typedef-leader-type"></a>

<a id="funcdef-content-leader"></a>leader( [\<leader-type\>](#typedef-leader-type) )

Inserts a leader. See the section on [leaders](#leaders) for more information.

<a id="funcdef-leader"></a>

<a id="ref-for-typedef-leader-type①"></a>

<a id="typedef-leader-type"></a>

<a id="ref-for-typedef-leader-type②"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-string-value⑧"></a>

```text
leader() = leader( <leader-type> )
<leader-type> = dotted | solid | space | <string>
```
Three keywords are shorthand values for common strings:

<a id="valdef-leader-dotted"></a>dotted

Equivalent to leader(".")

<a id="valdef-leader-solid"></a>solid

Equivalent to leader("\_")

<a id="valdef-leader-space"></a>space

Equivalent to leader(" ")

<a id="ref-for-string-value⑨"></a>

<a id="valdef-leader-string"></a>[\<string\>](https://www.w3.org/TR/css3-values/#string-value)

Issue: Define this.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e0047436"></a>
>
> ```text
> ol.toc a::after {
>   content: leader('.') target-counter(attr(href), page);
> }
> 
> <h1>Table of Contents</h1>
> <ol class="toc">
> <li><a href="#chapter1">Loomings</a></li>
> <li><a href="#chapter2">The Carpet-Bag</a></li>
> <li><a href="#chapter3">The Spouter-Inn</a></li>
> </ol>
> ```
>
> This might result in:
>
> ```text
> Table of Contents
> 
> 1. Loomings.....................1
> 2. The Carpet-Bag...............9
> 3. The Spouter-Inn.............13
> ```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-2f5c8d3f"></a> Do leaders depend on the assumption that the content after the leader is right-aligned (end-aligned)?

#### <a id="leader-rules"></a>2.5.2.  Rendering leaders

Consider a line which contains the content before the leader (the “before content”), the leader, and the content after the leader (the “after content”). Leaders obey the following rules:

1.  The leader string must appear in full at least once.

2.  The leader should be as long as possible

3.  Visible characters in leaders should vertically align with each other when possible.

4.  Line break characters in the leader string must be ignored.

5.  White space in the leader string follows normal CSS rules.

6.  A leader only appears between the start content and the end content.

7.  A leader only appears on a single line, even if the before content and after content are on different lines.

8.  A leader can’t be the only thing on a line.

#### <a id="leader-alignment"></a>2.5.3.  Procedure for rendering leaders

1.  Lay out the <var>before content</var>, until reaching the line where the <var>before content</var> ends.

    ```text
    BBBBBBBBBB
    BBB
    ```
2.  The leader string consists of one or more glyphs, and is thus an inline box. A leader is a row of these boxes, drawn from the end edge to the start edge, where only those boxes not overlaid by the before or after content. On this line, draw the leader string, starting from the end edge, repeating as many times as possible until reaching the start edge.

    ```text
    BBBBBBBBBB
    ..........
    ```
3.  Draw the before and after content on top of the leader. If any part of the <var>before content</var> or <var>after content</var> overlaps a glyph in a leader string box, that glyph is not displayed.

    ```text
    BBBBBBBBBB
    BBB....AAA
    ```
4.  If one full copy of the leader string is not visible:

    ```text
    BBBBBBB
    BBBBBBA
    ```
    Insert a line break after the <var>before content</var>, draw the leader on the next line, and draw the <var>after content</var> on top, and hide any leader strings that are not fully displayed.

    ```text
    BBBBBBB
    BBBBBB
    ......A
    ```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-449754e4"></a> what to do if <var>after content</var> is wider than the line box?

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-bb18b3e4"></a> Leaders don’t quite work in table layouts. How can we fix this?

![drawing leaders](https://www.w3.org/TR/2019/WD-css-content-3-20190802/images/leader.001.jpg)

Procedure for drawing leaders

![drawing leaders](https://www.w3.org/TR/2019/WD-css-content-3-20190802/images/leader.002.jpg)

Procedure for drawing leaders when the content doesn’t fit on a single line

### <a id="cross-references"></a>2.6.  Cross references and the target-\* functions

Many documents contain internal references:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-33585682"></a>
>
> - See chapter 7
>
> - in section 4.1
>
> - on page 23

<a id="ref-for-funcdef-target-counter"></a>

<a id="ref-for-funcdef-target-counters"></a>

<a id="ref-for-target-text-function"></a>

Three new values for the content property are used to automatically create these types of cross-references: [target-counter()](#funcdef-target-counter), [target-counters()](#funcdef-target-counters), and [target-text()](#target-text-function). Each of these displays information obtained from the target end of a link.

<a id="typedef-target"></a>

<a id="ref-for-typedef-target①"></a>

<a id="ref-for-funcdef-target-counter①"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-funcdef-target-counters①"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-target-text-function①"></a>

```text
<target> = <target-counter()> | <target-counters()> | <target-text()>
```
See sections below for details on each of these.

<a id="ref-for-funcdef-target-counter②"></a>

#### <a id="target-counter"></a>2.6.1.  The [target-counter()](#funcdef-target-counter) function

<a id="funcdef-target-counter"></a>

<a id="ref-for-string-value①⓪"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-url-value"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-identifier-value"></a>

<a id="ref-for-comb-comma①"></a>

<a id="ref-for-typedef-counter-style"></a>

<a id="ref-for-mult-opt①"></a>

```text
target-counter() = target-counter( [ <string> | <url> ] , <custom-ident> , <counter-style>? )
```
<a id="ref-for-funcdef-target-counter③"></a>

The [target-counter()](#funcdef-target-counter) function retrieves the value of the innermost counter with a given name. The required arguments are the url of the target and the name of the counter. An optional counter-style argument can be used to format the result.

These functions only take a fragment URL which points to a location in the current document. If there’s no fragment, if the ID referenced isn’t there, or if the URL points to an outside document, the user agent must treat that as an error.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-7443dd19"></a> what should error handling be?

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-2b2bd98b"></a> restrict syntactically to local references for now.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a3b9f868"></a> HTML:
>
> ```text
> …which will be discussed on page <a href="#chapter4_sec2"></a>.
> ```
>
> CSS:
>
> ```text
> a::after { content: target-counter(attr(href url), page) }
> ```
>
> Result:
>
> ```text
> …which will be discussed on page 137.
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-61dcfb19"></a> Page numbers in tables of contents can be generated automatically:
>
> HTML:
>
> ```text
> <nav>
>   <ol>
>    <li class="frontmatter"><a href="#pref_01">Preface</a></li>
>    <li class="frontmatter"><a href="#intr_01">Introduction</a></li>
>    <li class="bodymatter"><a href="#chap_01">Chapter One</a></li>
>   </ol>
> </nav>
> ```
>
> CSS:
>
> ```text
> .frontmatter a::after { content: leader('.') target-counter(attr(href url), page, lower-roman) }
> .bodymatter a::after { content: leader('.') target-counter(attr(href url), page, decimal) }
> ```
>
> Result:
>
> ```text
> Preface.............vii
> Introduction.........xi
> Chapter One...........1
> ```
<a id="ref-for-funcdef-target-counters②"></a>

#### <a id="target-counters"></a>2.6.2.  The [target-counters()](#funcdef-target-counters) function 

This functions fetches the value of all counters of a given name from the end of a link, and formats them by inserting a given string between the value of each nested counter.

<a id="funcdef-target-counters"></a>

<a id="ref-for-string-value①①"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-url-value①"></a>

<a id="ref-for-comb-comma②"></a>

<a id="ref-for-identifier-value①"></a>

<a id="ref-for-comb-comma③"></a>

<a id="ref-for-string-value①②"></a>

<a id="ref-for-comb-comma④"></a>

<a id="ref-for-typedef-counter-style①"></a>

<a id="ref-for-mult-opt②"></a>

```text
target-counters() = target-counters( [ <string> | <url> ] , <custom-ident> , <string> , <counter-style>? )
```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-00974551"></a>
>
> ```text
> I have not found a compelling example for target-counters() yet.
> ```
>
> > <strong data-conversion-semantic="issue">Issue</strong>
> >
> > <a id="issue-6e7203db"></a> found a compelling example, in CSS specs. Do something.

<a id="ref-for-target-text-function②"></a>

#### <a id="target-text"></a>2.6.3.  The [target-text()](#target-text-function) function

<a id="ref-for-target-text-function③"></a>

<a id="ref-for-propdef-string-set"></a>

The [target-text()](#target-text-function) function retrieves the text value of the element referred to by the URL. An optional second argument specifies what content is retrieved, using the same values as the [string-set](#propdef-string-set) property above.

<a id="target-text-function"></a>

<a id="ref-for-string-value①③"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-url-value②"></a>

<a id="ref-for-comb-comma⑤"></a>

<a id="ref-for-comb-one②③"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-comb-one②⑤"></a>

<a id="ref-for-mult-opt③"></a>

```text
target-text() = target-text( [ <string> | <url> ] , [ content | before | after | first-letter ]? )
```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a82075c9"></a> A simpler syntax has been proposed by fantasai: [https&#x3A;&#x2F;&#x2F;lists&#x2E;w3&#x2E;org&#x2F;Archives&#x2F;Public&#x2F;www-style&#x2F;2012Feb&#x2F;0745&#x2E;html](https://lists.w3.org/Archives/Public/www-style/2012Feb/0745.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d5ce9fa3"></a>
>
> ```text
> …which will be discussed <a href="#chapter_h1_1">later</a>.
> 
> a::after { content: ", in the chapter entitled " target-text(attr(href url)) }
> ```
>
> Result: …which will be discussed later, in the chapter entitled Loomings.

### <a id="named-strings"></a>2.7.  Named strings

<a id="ref-for-propdef-string-set①"></a>

<a id="ref-for-propdef-content①④"></a>

This section introduces <a id="named-string"></a>named strings, which are the textual equivalent of counters and which have a distinct namespace from counters. Named strings follow the same nesting rules as counters. The [string-set](#propdef-string-set) property accepts values similar to the [content](#propdef-content) property, including the extraction of the current value of counters.

Named strings are a convenient way to pull metadata out of the document for insertion into headers and footers. In HTML, for example, META elements contained in the document HEAD can set the value of named strings. In conjunction with attribute selectors, this can be a powerful mechanism:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-208227c0"></a>
>
> ```text
> meta[author] { string-set: author attr(author); }
> head > title { string-set: title contents; }
> @page:left {
>   @top {
>    text-align: left;
>    vertical-align: middle;
>    content: string(title);
>   }
> }
> @page:right {
>   @top {
>    text-align: right;
>    vertical-align: middle;
>    content: string(author);
>   }
> }
> ```
#### <a id="string-set"></a>2.7.1.  The string-set property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                    |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-string-set"></a>string-set                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://drafts.csswg.org/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma"></a><a id="ref-for-mult-one-plus③"></a><a id="ref-for-string-value①④"></a><a id="ref-for-identifier-value②"></a><a id="ref-for-comb-one②⑥"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) [\<string\>](https://www.w3.org/TR/css3-values/#string-value)[+](https://www.w3.org/TR/css-values-4/#mult-one-plus) \][\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://drafts.csswg.org/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong>Applies to:&#xA;      </strong> | all elements, but not pseudo-elements                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://drafts.csswg.org/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://drafts.csswg.org/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://drafts.csswg.org/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-string-set-none"></a>the keyword [none](#valdef-string-set-none) or a list, each item an identifier paired with a list of string values                                                                                                                                                                                                                                                                         |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://drafts.csswg.org/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                      |



User Agents are expected to support this property on all media, including non-visual ones.

<a id="ref-for-propdef-string-set②"></a>

<a id="ref-for-funcdef-string"></a>

The [string-set](#propdef-string-set) property copies the text content of an element into a named string, which functions as a variable. The text content of this named string can be retrieved using the [string()](#funcdef-string) function. Since these variables may change on a given page, an optional second value for the <a id="ref-for-funcdef-string①"></a>string() function allows authors to choose which value on a page is used.

<a id="valdef-string-set-none"></a>none

The element does not set any named strings.

<a id="ref-for-string-value①⑤"></a>

<a id="ref-for-identifier-value③"></a>

<code>&#x5B;&#x20;<a href="https://www.w3.org/TR/css-values-4/#identifier-value">&lt;custom-ident&gt;</a>&#x20;<a href="https://www.w3.org/TR/css3-values/#string-value">&lt;string&gt;</a>+&#x20;&#x5D;#</code>

The element establishes one or more named strings, corresponding to each comma-separated entry in the list.

<a id="ref-for-identifier-value④"></a>

<a id="ref-for-string-value①⑥"></a>

For each entry, the [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) gives the name of the named string. It’s followed by one or more [\<string\>](https://www.w3.org/TR/css3-values/#string-value) values, which are concatenated together to form the value of the named string.

<a id="ref-for-style-containment"></a>

<a id="ref-for-propdef-string-set③"></a>

If an element has [style containment](https://www.w3.org/TR/css-contain-1/#style-containment), the [string-set](#propdef-string-set) property must have no effects on descendants of that element.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ebda2d09"></a> The following example captures the contents of H1 elements, which represent chapter names in this hypothetical document.
>
> ```text
> H1 { string-set: chapter contents; }
> ```
>
> When an H1 element is encountered, the chapter string is set to the element’s textual contents, and the previous value of chapter, if any, is overwritten.

<a id="ref-for-funcdef-string②"></a>

#### <a id="string-function"></a>2.7.2.  The [string()](#funcdef-string) function

<a id="funcdef-string"></a>

<a id="ref-for-identifier-value⑤"></a>

<a id="ref-for-comb-comma⑥"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-mult-opt④"></a>

```text
string() = string( <custom-ident> , [ first | start | last | first-except ]? )
```
<a id="ref-for-funcdef-string③"></a>

<a id="ref-for-propdef-content①⑤"></a>

The [string()](#funcdef-string) function is used to copy the value of a named string to the document, via the [content](#propdef-content) property. This function requires one argument, the name of the named string. Since the value of a named string may change several times on a page (as multiple elements defining the string can appear) an optional second argument indicates which value of the named string should be used.

<a id="ref-for-funcdef-string④"></a>

The second argument of the [string()](#funcdef-string) function is one of the following keywords:

<a id="valdef-string-first"></a>first  
<a id="ref-for-entry-value"></a>

The value of the first assignment on the page is used. If there is no assignment on the page, the [entry value](#entry-value) is used. If no second argument is provided, this is the default value.

<a id="valdef-string-start"></a>start  
<a id="ref-for-named-string"></a>

<a id="ref-for-entry-value①"></a>

If the element is the first element on the page, the value of the first assignment is used. Otherwise the [entry value](#entry-value) is used. The <a id="ref-for-entry-value②"></a>entry value may be empty if the [named string](#named-string) hasn’t yet appeared.

<a id="valdef-string-last"></a>last  
<a id="ref-for-exit-value"></a>

The [exit value](#exit-value) of the named string is used.

<a id="valdef-string-first-except"></a>first-except  
<a id="ref-for-valdef-string-first"></a>

This is identical to [first](#valdef-string-first), except that the empty string is used on the page where the value is assigned.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c24bc3d7"></a> we may need to kill the entire content string. Is this necessary?

The content values of named strings are assigned at the point when the content box of the element is first created (or would have been created if the element’s display value is none). The <a id="entry-value"></a>entry value for a page is the assignment in effect at the end of the previous page. The <a id="exit-value"></a>exit value for a page is the assignment in effect at the end of the current page.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-23e3824c"></a> CSS:
>
> ```text
> @page {
>   size: 15cm 10cm;
>   margin: 1.5cm;
> 
>   @top-left {
>   content: "first: " string(heading, first);
>   }
>   @top-center {
>   content: "start: " string(heading, start);
>   }
>    @top-right {
>    content: "last: " string(heading, last);
>   }
>   }
> 
> h2 { string-set: heading content() }
> ```
>
> The following figures show the first, start, and last assignments of the “heading” string on various pages.
>
> ![](https://www.w3.org/TR/2019/WD-css-content-3-20190802/images/using-strings-1.jpg)
>
> The start value is empty, as the string had not yet been set at the start of the page.
>
> ![](https://www.w3.org/TR/2019/WD-css-content-3-20190802/images/using-strings-2.jpg)
>
> Since the page starts with an h2, the start value is the value of that head.
>
> ![](https://www.w3.org/TR/2019/WD-css-content-3-20190802/images/using-strings-3.jpg)
>
> <a id="ref-for-exit-value①"></a>
>
> Since there’s not an h2 at the top of this page, the start value is the [exit value](#exit-value) of the previous page.

<a id="ref-for-funcdef-content①"></a>

#### <a id="content-function"></a>2.7.3.  The [content()](#funcdef-content) function

<a id="funcdef-content"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-comb-one③②"></a>

<a id="ref-for-comb-one③③"></a>

<a id="ref-for-mult-opt⑤"></a>

```text
content() = content( [ text | before | after | first-letter | marker ]? )
```
<a id="valdef-content-text"></a>text  
<a id="ref-for-valdef-content-text"></a>

<a id="ref-for-funcdef-content②"></a>

The string value of the element. If no value is specified in [content()](#funcdef-content), it acts as if [text](#valdef-content-text) were specified.

<a id="valdef-content-before"></a>before  
<a id="ref-for-selectordef-before②"></a>

The string value of the [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) pseudo-element.

<a id="valdef-content-after"></a>after  
<a id="ref-for-selectordef-after②"></a>

The string value of the [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudo-element.

<a id="valdef-content-first-letter"></a>first-letter  
<a id="ref-for-selectordef-first-letter"></a>

The first letter of the element, as defined for the [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-element

<a id="valdef-content-marker"></a>marker  
<a id="ref-for-selectordef-marker②"></a>

The string value of the [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e698ead2"></a> HTML:
>
> ```text
> <h1>Loomings</h1>
> ```
>
> CSS:
>
> ```text
> h1::before { content: 'Chapter ' counter(chapter); }
> h1 { string-set: header content(before) ':' content(text); }
> h1::after { content: '.'; }
> ```
>
> The value of the named string “header” will be “Chapter 1: Loomings”.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ba54cbec"></a> HTML:
>
> ```text
> <section title="Loomings">
> ```
>
> CSS:
>
> ```text
> section { string-set: header attr(title) }
> ```
>
> The value of the “header” string will be “Loomings”.

<a id="ref-for-propdef-counter-increment"></a>

<a id="ref-for-propdef-counter-reset"></a>

## <a id="counters"></a>3.  Automatic counters and numbering: the [counter-increment](https://www.w3.org/TR/css-lists-3/#propdef-counter-increment) and [counter-reset](https://www.w3.org/TR/css-lists-3/#propdef-counter-reset) properties (moved)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-227d7dec"></a> Now described in [\[CSS3LIST\]](#biblio-css3list)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-0161c10e"></a> Should this move back to CSS Content?

## <a id="bookmark-generation"></a>4.  Bookmarks

<a id="ref-for-propdef-bookmark-level"></a>

<a id="ref-for-propdef-bookmark-label"></a>

<a id="ref-for-propdef-bookmark-state"></a>

Some document formats, most notably PDF, allow the use of <a id="bookmarks"></a>bookmarks as an aid to navigation. Bookmarks provide a list of links to document elements, as well as text to label the links and a level value. A bookmark has three properties: [bookmark-level](#propdef-bookmark-level), [bookmark-label](#propdef-bookmark-label), and [bookmark-state](#propdef-bookmark-state).

<a id="ref-for-target-pseudo"></a>

When a user activates a bookmark, the user agent must bring that reference point to the user’s attention, exactly as if navigating to that element by fragment URL. <strong data-conversion-semantic="note">Note:</strong> This will also trigger matching the [:target](https://www.w3.org/TR/selectors4/#target-pseudo) pseudo-class.

<a id="ref-for-style-containment①"></a>

<a id="ref-for-propdef-bookmark-level①"></a>

<a id="ref-for-propdef-bookmark-label①"></a>

<a id="ref-for-propdef-bookmark-state①"></a>

If an element has [style containment](https://www.w3.org/TR/css-contain-1/#style-containment), the [bookmark-level](#propdef-bookmark-level), [bookmark-label](#propdef-bookmark-label), and [bookmark-state](#propdef-bookmark-state) properties must have no effect on descendants of the element.

### <a id="bookmark-level"></a>4.1.  bookmark-level 

<a id="ref-for-propdef-bookmark-level②"></a>

<a id="ref-for-valdef-bookmark-level-none"></a>

<a id="ref-for-propdef-bookmark-label②"></a>

<a id="ref-for-propdef-bookmark-state②"></a>

The [bookmark-level](#propdef-bookmark-level) property determines if a bookmark is created, and at what level. If this property is absent, or has value [none](#valdef-bookmark-level-none), no bookmark should be generated, regardless of the values of [bookmark-label](#propdef-bookmark-label) or [bookmark-state](#propdef-bookmark-state).



| Field               | Definition                                                                                                                                                    |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-bookmark-level"></a>bookmark-level                                                                                                                             |
| <strong><a href="https://drafts.csswg.org/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-integer-value"></a><a id="ref-for-comb-one③④"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<integer\>](https://www.w3.org/TR/css3-values/#integer-value) |
| <strong><a href="https://drafts.csswg.org/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                          |
| <strong>Applies to:&#xA;      </strong> | [all elements](https://drafts.csswg.org/css-pseudo/#generated-content)                                                                                        |
| <strong><a href="https://drafts.csswg.org/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                            |
| <strong><a href="https://drafts.csswg.org/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                           |
| <strong><a href="https://drafts.csswg.org/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-bookmark-level-none①"></a>the keyword [none](#valdef-bookmark-level-none) or the specified integer                                                                   |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                   |
| <strong><a href="https://drafts.csswg.org/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                        |



<a id="ref-for-integer-value①"></a>

<a id="valdef-bookmark-level-integer"></a>[\<integer\>](https://www.w3.org/TR/css3-values/#integer-value)

defines the level of the bookmark, with the top level being 1 (negative and zero values are invalid).

<a id="valdef-bookmark-level-none"></a>none

no bookmark is generated.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dd8b2d49"></a>
>
> ```text
> section h1 { bookmark-level: 1; }
> section section h1 { bookmark-level: 2; }
> section section section h1 { bookmark-level: 3; }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Bookmarks do not need to create a strict hierarchy of levels.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-de0f5f60"></a> Should a bookmark be created for elements with `display: none`?

### <a id="bookmark-label"></a>4.2.  bookmark-label



| Field               | Definition                                                             |
|---------------------|------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-bookmark-label"></a>bookmark-label                                      |
| <strong><a href="https://drafts.csswg.org/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-content-content-list⑥"></a>[\<content-list\>](#typedef-content-content-list)   |
| <strong><a href="https://drafts.csswg.org/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | content(text)                                                          |
| <strong>Applies to:&#xA;      </strong> | [all elements](https://drafts.csswg.org/css-pseudo/#generated-content) |
| <strong><a href="https://drafts.csswg.org/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                     |
| <strong><a href="https://drafts.csswg.org/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                    |
| <strong><a href="https://drafts.csswg.org/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                        |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                            |
| <strong><a href="https://drafts.csswg.org/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                               |



<a id="ref-for-typedef-content-content-list⑦"></a>

<a id="valdef-bookmark-label-content-list"></a>[\<content-list\>](#typedef-content-content-list)

<a id="ref-for-propdef-string-set④"></a>

<a id="ref-for-typedef-content-content-list⑧"></a>

[\<content-list\>](#typedef-content-content-list) is defined above, in the section on the [string-set](#propdef-string-set) property. The value of <a id="ref-for-typedef-content-content-list⑨"></a>\<content-list\> becomes the text content of the bookmark label.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-273659aa"></a> HTML:
>
> ```text
> <h1>Loomings</h1>
> ```
>
> CSS:
>
> ```text
> h1 {
> bookmark-label: content(text);
> bookmark-level: 1;
> }
> ```
>
> The bookmark label will be “Loomings”.

### <a id="bookmark-state"></a>4.3.  bookmark-state

<a id="ref-for-propdef-bookmark-state③"></a>

The [bookmark-state](#propdef-bookmark-state) may be open or closed. The user must be able to toggle the bookmark state.



| Field               | Definition                                                                        |
|---------------------|-----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-bookmark-state"></a>bookmark-state                                                 |
| <strong><a href="https://drafts.csswg.org/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one③⑤"></a>open [\|](https://www.w3.org/TR/css-values-4/#comb-one) closed |
| <strong><a href="https://drafts.csswg.org/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | open                                                                              |
| <strong>Applies to:&#xA;      </strong> | block-level elements                                                              |
| <strong><a href="https://drafts.csswg.org/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                |
| <strong><a href="https://drafts.csswg.org/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                               |
| <strong><a href="https://drafts.csswg.org/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                 |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                       |
| <strong><a href="https://drafts.csswg.org/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                          |



<a id="valdef-bookmark-state-open"></a>open  
<a id="ref-for-propdef-bookmark-level③"></a>

Subsequent bookmarks with [bookmark-level](#propdef-bookmark-level) greater than the given bookmark are displayed, until reaching another bookmark of the same level or lower. If one of subsequent bookmark is closed, apply the same test to determine if its subsequent bookmarks should be displayed.

<a id="valdef-bookmark-state-closed"></a>closed  
Subsequent bookmarks of bookmark-level greater than the given bookmark are not displayed, until reaching another bookmark of the same level or lower.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-be5a86f5"></a> Is the initial bookmark state, or the bookmark state updated by the UA as appropriate?

## <a id="changes"></a>5. Changes since the 2 June 2016 Working Draft

Significant changes since the [2 June 2016 Working Draft](https://www.w3.org/TR/2016/WD-css-content-3-20160602/) consist primarily of:

- <a id="ref-for-propdef-quotes⑤"></a>

  <a id="ref-for-valdef-quotes-auto①"></a>

  Adding [auto](#valdef-quotes-auto) as the initial value of [quotes](#propdef-quotes).

- Lots of miscellaneous spec clean up: errors, cross-references, overly-loose or sloppy definitions, etc.

See also [previous changes](https://www.w3.org/TR/2016/WD-css-content-3-20160602/#changes).

## <a id="acknowledgements"></a>Acknowledgements

Stuart Ballard, David Baron, Bert Bos, Tantek Çelik, and James Craig provided invaluable suggestions used in this specification.

## <a id="conformance"></a> Conformance

### <a id="document-conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae2b6bc0"></a>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong> UAs MUST provide an accessible alternative. </strong>

### <a id="conform-classes"></a> Conformance classes

Conformance to this specification is defined for three conformance classes:

style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS2/conform.html#style-sheet).

renderer  
A [UA](https://www.w3.org/TR/CSS2/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

authoring tool  
A [UA](https://www.w3.org/TR/CSS2/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="conform-responsible"></a> Requirements for Responsible Implementation of CSS

The following sections define several conformance requirements for implementing CSS responsibly, in a way that promotes interoperability in the present and future.

#### <a id="conform-partial"></a> Partial Implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, <strong>CSS renderers <em>must</em> treat as invalid&#xA;&#x9;&#x9;(and <a href="https://www.w3.org/TR/CSS2/conform.html#ignore">ignore as appropriate</a>)&#xA;&#x9;&#x9;any at-rules, properties, property values, keywords, and other syntactic constructs&#xA;&#x9;&#x9;for which they have no usable level of support</strong>. In particular, user agents <em>must not</em> selectively ignore unsupported property values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

#### <a id="conform-testing"></a> Implementations of CR-level Features

Once a specification reaches the Candidate Recommendation stage, implementers should release an [unprefixed](https://www.w3.org/TR/CSS/#vendor-prefix) implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec, and should avoid exposing a prefixed variant of that feature.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [after](#valdef-content-after), in §2.7.3
- [auto](#valdef-quotes-auto), in §2.4.1
- [before](#valdef-content-before), in §2.7.3
- [bookmark-label](#propdef-bookmark-label), in §4.2
- [bookmark-level](#propdef-bookmark-level), in §4.1
- [bookmarks](#bookmarks), in §4
- [bookmark-state](#propdef-bookmark-state), in §4.3
- [closed](#valdef-bookmark-state-closed), in §4.3
- [close-quote](#valdef-content-close-quote), in §2.4.2
- [content()](#funcdef-content), in §2.7.3
- [content](#propdef-content), in §1
- \<content-list\>
  - [type for content](#typedef-content-content-list), in §1
  - [value for bookmark-label](#valdef-bookmark-label-content-list), in §4.2
- [\<content-replacement\>](#typedef-content-content-replacement), in §1
- [contents](#valdef-content-contents), in §2.3
- [dotted](#valdef-leader-dotted), in §2.5.1
- [entry value](#entry-value), in §2.7.2
- [exit value](#exit-value), in §2.7.2
- [first](#valdef-string-first), in §2.7.2
- [first-except](#valdef-string-first-except), in §2.7.2
- [first-letter](#valdef-content-first-letter), in §2.7.3
- [\<image\>](#valdef-content-list-image), in §2.2
- [\<integer\>](#valdef-bookmark-level-integer), in §4.1
- [last](#valdef-string-last), in §2.7.2
- leader()
  - [(function)](#funcdef-leader), in §2.5.1
  - [function for content, \<content-list\>](#funcdef-content-leader), in §2.5.1
- [\<leader-type\>](#typedef-leader-type), in §2.5.1
- [marker](#valdef-content-marker), in §2.7.3
- [named string](#named-string), in §2.7
- [no-close-quote](#valdef-content-no-close-quote), in §2.4.2
- none
  - [value for bookmark-level](#valdef-bookmark-level-none), in §4.1
  - [value for content](#valdef-content-none), in §1
  - [value for quotes](#valdef-quotes-none), in §2.4.1
  - [value for string-set](#valdef-string-set-none), in §2.7.1
- [no-open-quote](#valdef-content-no-open-quote), in §2.4.2
- [normal](#valdef-content-normal), in §1
- [open](#valdef-bookmark-state-open), in §4.3
- [open-quote](#valdef-content-open-quote), in §2.4.2
- [\<quote\>](#typedef-quote), in §2.4.2
- [quotes](#propdef-quotes), in §2.4.1
- [solid](#valdef-leader-solid), in §2.5.1
- [space](#valdef-leader-space), in §2.5.1
- [start](#valdef-string-start), in §2.7.2
- [string()](#funcdef-string), in §2.7.2
- \<string\>
  - [value for content, \<content-list\>](#valdef-content-string), in §2.1
  - [value for leader()](#valdef-leader-string), in §2.5.1
- [/ \[ \<string\> \| \<counter\> \]+](#valdef-content---string--counter), in §1
- [string-set](#propdef-string-set), in §2.7.1
- [\<target\>](#typedef-target), in §2.6
- [target-counter()](#funcdef-target-counter), in §2.6.1
- [target-counters()](#funcdef-target-counters), in §2.6.2
- [target-text()](#target-text-function), in §2.6.3
- [text](#valdef-content-text), in §2.7.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-cascade-4\] defines the following terms:
  - <a id="term-for-used-value"></a>used value
- \[css-contain-1\] defines the following terms:
  - <a id="term-for-style-containment"></a>style containment
- \[css-counter-styles-3\] defines the following terms:
  - <a id="term-for-typedef-counter-style"></a>\<counter-style\>
- \[css-images-4\] defines the following terms:
  - <a id="term-for-invalid-image"></a>invalid image
- \[css-pseudo-4\] defines the following terms:
  - <a id="term-for-selectordef-after"></a>::after
  - <a id="term-for-selectordef-before"></a>::before
  - <a id="term-for-selectordef-first-letter"></a>::first-letter
  - <a id="term-for-selectordef-marker"></a>::marker
- \[css-ruby-1\] defines the following terms:
  - <a id="term-for-propdef-display"></a>display
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="term-for-content-language"></a>content language
  - <a id="term-for-white-space"></a>white space
- \[css-values-3\] defines the following terms:
  - <a id="term-for-integer-value"></a>\<integer\>
  - <a id="term-for-string-value"></a>\<string\>
  - <a id="term-for-url-value"></a>\<url\>
- \[css-values-4\] defines the following terms:
  - <a id="term-for-mult-comma"></a>\#
  - <a id="term-for-mult-one-plus"></a>+
  - <a id="term-for-comb-comma"></a>,
  - <a id="term-for-identifier-value"></a>\<custom-ident\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-comb-one"></a>\|
- \[css3-images\] defines the following terms:
  - <a id="term-for-image-type"></a>\<image\>
- \[CSS3LIST\] defines the following terms:
  - <a id="term-for-typedef-counter"></a>\<counter\>
  - <a id="term-for-propdef-counter-increment"></a>counter-increment
  - <a id="term-for-propdef-counter-reset"></a>counter-reset
- \[HTML\] defines the following terms:
  - <a id="term-for-the-img-element"></a>img
- \[selectors-4\] defines the following terms:
  - <a id="term-for-target-pseudo"></a>:target
  - <a id="term-for-originating-element"></a>originating element

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-contain-1"></a>\[CSS-CONTAIN-1\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Containment Module Level 1](https://www.w3.org/TR/css-contain-1/). 30 April 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-1&#x2F;](https://www.w3.org/TR/css-contain-1/)

<a id="biblio-css-counter-styles-3"></a>\[CSS-COUNTER-STYLES-3\]  
Tab Atkins Jr.. [CSS Counter Styles Level 3](https://www.w3.org/TR/css-counter-styles-3/). 14 December 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-counter-styles-3&#x2F;](https://www.w3.org/TR/css-counter-styles-3/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Image Values and Replaced Content Module Level 4](https://www.w3.org/TR/css-images-4/). 13 April 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 25 February 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-ruby-1"></a>\[CSS-RUBY-1\]  
Elika Etemad; Koji Ishii. [CSS Ruby Layout Module Level 1](https://www.w3.org/TR/css-ruby-1/). 5 August 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ruby-1&#x2F;](https://www.w3.org/TR/css-ruby-1/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 12 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 31 January 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css3-images"></a>\[CSS3-IMAGES\]  
Elika Etemad; Tab Atkins Jr.. [CSS Image Values and Replaced Content Module Level 3](https://www.w3.org/TR/css3-images/). 17 April 2012. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css3-images&#x2F;](https://www.w3.org/TR/css3-images/)

<a id="biblio-css3-speech"></a>\[CSS3-SPEECH\]  
Daniel Weck. [CSS Speech Module](https://www.w3.org/TR/css3-speech/). 5 June 2018. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css3-speech&#x2F;](https://www.w3.org/TR/css3-speech/)

<a id="biblio-css3list"></a>\[CSS3LIST\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists Module Level 3](https://www.w3.org/TR/css-lists-3/). 25 April 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-lists-3&#x2F;](https://www.w3.org/TR/css-lists-3/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 21 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

### <a id="informative"></a>Informative References

<a id="biblio-cldr"></a>\[CLDR\]  
[Unicode Common Locale Data Repository](http://cldr.unicode.org/). URL: [http&#x3A;&#x2F;&#x2F;cldr&#x2E;unicode&#x2E;org&#x2F;](http://cldr.unicode.org/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

## <a id="property-index"></a>Property Index



| Name                | Value                                                                                                        | Initial       | Applies to                                                        | Inh. | %ages | Anim­ation type         | Canonical order | Com­puted value                                                                          |
|---------------------|--------------------------------------------------------------------------------------------------------------|---------------|-------------------------------------------------------------------|------|-------|------------------------|-----------------|-----------------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-bookmark-label③"></a></span><a href="#propdef-bookmark-label">bookmark-label</a>&#xA;      </strong> | \<content-list\>                                                                                             | content(text) | all elements                                                      | no   | N/A   | discrete               | per grammar     | specified value                                                                         |
| <strong><span><a id="ref-for-propdef-bookmark-level④"></a></span><a href="#propdef-bookmark-level">bookmark-level</a>&#xA;      </strong> | none \| \<integer\>                                                                                          | none          | all elements                                                      | no   | N/A   | by computed value type | per grammar     | the keyword none or the specified integer                                               |
| <strong><span><a id="ref-for-propdef-bookmark-state④"></a></span><a href="#propdef-bookmark-state">bookmark-state</a>&#xA;      </strong> | open \| closed                                                                                               | open          | block-level elements                                              | no   | N/A   | discrete               | per grammar     | specified keyword                                                                       |
| <strong><span><a id="ref-for-propdef-content①⑥"></a></span><a href="#propdef-content">content</a>&#xA;      </strong> | normal \| none \| \[ \<content-replacement\> \| \<content-list\> \] \[/ \[ \<string\> \| \<counter\> \]+ \]? | normal        | all elements, tree-abiding pseudo-elements, and page margin boxes | no   | n/a   | discrete               | per grammar     | See prose below                                                                         |
| <strong><span><a id="ref-for-propdef-quotes⑥"></a></span><a href="#propdef-quotes">quotes</a>&#xA;      </strong> | auto \| none \| \[ \<string\> \<string\> \]+                                                                 | auto          | all elements                                                      | yes  | n/a   | discrete               | per grammar     | the keyword none, the keyword auto, or a list, each item a pair of string values        |
| <strong><span><a id="ref-for-propdef-string-set⑤"></a></span><a href="#propdef-string-set">string-set</a>&#xA;      </strong> | none \| \[ \<custom-ident\> \<string\>+ \]#                                                                  | none          | all elements, but not pseudo-elements                             | no   | N/A   | discrete               | per grammar     | the keyword none or a list, each item an identifier paired with a list of string values |



## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The above [invalid image](https://www.w3.org/TR/css4-images/#invalid-image) behavior appears to be what Chrome is doing. Is this okay? Is there a better behavior we can/should use? [↵](#issue-b8fd18a9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> "Zero intrinsic width and height" gives it an undefined aspect ratio, and sizing behavior is thus... undefined. At least, if you give it an explicit width or height, the other dimension remains at zero in Chrome. This might need to be explicitly defined over in the Images spec; needs some investigation around whether browsers act this way for 0x0 SVGs and rasters. [↵](#issue-76d40dad)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should the contents keyword be replaced with [content()](#funcdef-content)? [↵](#issue-a8fbc981)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Start work on an AAM for CSS. [↵](#issue-9c60e2c1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> CSS2.1 explicitly allowed the UA to substitute a broken image icon if the image was invalid. However, no browser appears to do this. Is this removal okay? [↵](#issue-3ef3081c)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should this behave as an empty string on pseudo-elements? [↵](#issue-bf2f178e)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Use cases for suppressing the content on the element and using it in a pseudo-element would be welcome. [↵](#issue-bfc60bbc)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Do we need the statement about marker pseudo-elements here? Or is this legacy from the old version of the spec? [↵](#issue-9fa9cc25)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Do leaders depend on the assumption that the content after the leader is right-aligned (end-aligned)? [↵](#issue-2f5c8d3f)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> what to do if <var>after content</var> is wider than the line box? [↵](#issue-449754e4)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Leaders don’t quite work in table layouts. How can we fix this? [↵](#issue-bb18b3e4)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> what should error handling be? [↵](#issue-7443dd19)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> restrict syntactically to local references for now. [↵](#issue-2b2bd98b)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> found a compelling example, in CSS specs. Do something. [↵](#issue-6e7203db)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> A simpler syntax has been proposed by fantasai: [https&#x3A;&#x2F;&#x2F;lists&#x2E;w3&#x2E;org&#x2F;Archives&#x2F;Public&#x2F;www-style&#x2F;2012Feb&#x2F;0745&#x2E;html](https://lists.w3.org/Archives/Public/www-style/2012Feb/0745.html) [↵](#issue-a82075c9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> we may need to kill the entire content string. Is this necessary? [↵](#issue-c24bc3d7)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Now described in [\[CSS3LIST\]](#biblio-css3list) [↵](#issue-227d7dec)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should this move back to CSS Content? [↵](#issue-0161c10e)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should a bookmark be created for elements with `display: none`? [↵](#issue-de0f5f60)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Is the initial bookmark state, or the bookmark state updated by the UA as appropriate? [↵](#issue-be5a86f5)
