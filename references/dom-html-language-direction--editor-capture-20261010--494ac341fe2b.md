# HTML Standard: bounded DOM foundation source capture

Attribution and reformatting notice added for Surgeist on 2026-10-10. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative; this partial format conversion is implementation support, not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [HTML Standard](https://html.spec.whatwg.org/multipage/dom.html). Retrieved 2026-10-10. Displayed status: HTMLLiving Standard — Last Updated 9 October 2026.

The exact upstream repository LICENSE files from DOM `3071e5f269448350fdea79bb284eedaf17bc431e`, HTML `877d6146487f15b15bc8dd0e3c625063a37b456a`, Web IDL `0620cf171d1b97138847088748e363cae4005156` and Infra `c67ae1fb1d161b23d0f8c9bf9917ea1993f7b19a` all match the already retained [WHATWG LICENSE](../licenses/whatwg/LICENSE) byte-for-byte, SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; the BSD 3-Clause scope applies when portions are incorporated into code, not as an alternative license for this Markdown.

Full retrieved HTML: 440,004 bytes, SHA-256 `2540c3e6a061fef814c0e7e53c1c64c892a6807a63396199a7783f868d4b6ea8`. Complete-section extraction input: 78,246 bytes, SHA-256 `494ac341fe2b92774e37813e33f394688e6c751b1e8d3a9eba2b753639d17b32`. Source revision metadata: `No revision metadata in this chapter; GitHub HEAD association is not proven.`.

Retained complete heading sections, including descendant clauses and bounded at the next equal-or-higher-level heading: `the-documentorshadowroot-interface`, `elements-in-the-dom`, `the-lang-and-xml:lang-attributes`, `the-dir-attribute`. Original source header is also retained. Every other clause and all cross-chapter references remain upstream. These are explicitly partial captures. HTML commit-snapshot chapter URLs for the stated repository HEAD returned HTTP 404; exact fetched chapter body hashes identify the captures without asserting that repository HEAD generated them.

Conversion uses existing repository HTML preparation/table-span models, approved Pandoc 3.1.11.1 and the semantic Lua filter. Source scripts/styles are omitted without execution. Source headings, IDs, links, prose, inline code, variables, literal blocks, images and every selected table cell/span are compared against parsed GFM. Selected tables remain literal semantic HTML, preserving every original table cell, code/link/variable, row/column span and anchor; table flattening is not used. Source examples and Web IDL remain evidence even where the corresponding JavaScript API is outside Surgeist's native Rust contract.

---

<!-- captured-body-start -->

<a id="head"></a>

[![WHATWG](https://resources.whatwg.org/logo.svg)](https://whatwg.org/)

# HTML

<a id="living-standard"></a>Living Standard — Last Updated 9 October 2026

#### <a id="the-documentorshadowroot-interface"></a>3.1.2 The <a id="the-documentorshadowroot-interface&#58;documentorshadowroot"></a><code>[DocumentOrShadowRoot](#documentorshadowroot)</code> interface[](#the-documentorshadowroot-interface)

DOM defines the <a id="the-documentorshadowroot-interface&#58;dom-documentorshadowroot"></a><code>[DocumentOrShadowRoot](https://dom.spec.whatwg.org/#documentorshadowroot)</code> mixin, which this specification extends.

<a id="documentorshadowroot"></a><a id="the-documentorshadowroot-interface&#58;element"></a><a id="the-documentorshadowroot-interface&#58;dom-documentorshadowroot-activeelement"></a>

``` text
partial interface mixin DocumentOrShadowRoot {
  readonly attribute Element? activeElement;
};
```

#### <a id="elements-in-the-dom"></a>3.2.2 Elements in the DOM[](#elements-in-the-dom)

The nodes representing <a id="elements-in-the-dom&#58;html-elements"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in the DOM must implement, and expose to scripts, the interfaces listed for them in the relevant sections of this specification. This includes <a id="elements-in-the-dom&#58;html-elements-2"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in <a id="elements-in-the-dom&#58;xml-documents"></a>[XML documents](https://dom.spec.whatwg.org/#xml-document), even when those documents are in another context (e.g. inside an XSLT transform).

Elements in the DOM <a id="represents"></a><strong>represent</strong> things; that is, they have intrinsic <em>meaning</em>, also known as semantics.

<strong>Example:</strong>

For example, an <a id="elements-in-the-dom&#58;the-ol-element"></a><code>[ol](https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element)</code> element represents an ordered list.

Elements can be <a id="referenced"></a><strong>referenced</strong> (referred to) in some way, either explicitly or implicitly. One way that an element in the DOM can be explicitly referenced is by giving an <a id="elements-in-the-dom&#58;the-id-attribute"></a><code>[id](https://html.spec.whatwg.org/multipage/dom.html#the-id-attribute)</code> attribute to the element, and then creating a <a id="elements-in-the-dom&#58;hyperlink"></a>[hyperlink](https://html.spec.whatwg.org/multipage/links.html#hyperlink) with that <a id="elements-in-the-dom&#58;the-id-attribute-2"></a><code>[id](https://html.spec.whatwg.org/multipage/dom.html#the-id-attribute)</code> attribute's value as the <a id="elements-in-the-dom&#58;navigate-fragid"></a>[fragment](https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate-fragid) for the <a id="elements-in-the-dom&#58;hyperlink-2"></a>[hyperlink](https://html.spec.whatwg.org/multipage/links.html#hyperlink)'s <a id="elements-in-the-dom&#58;attr-hyperlink-href"></a><code>[href](https://html.spec.whatwg.org/multipage/links.html#attr-hyperlink-href)</code> attribute value. Hyperlinks are not necessary for a reference, however; any manner of referring to the element in question will suffice.

<strong>Example:</strong>

Consider the following <a id="elements-in-the-dom&#58;the-figure-element"></a><code>[figure](https://html.spec.whatwg.org/multipage/grouping-content.html#the-figure-element)</code> element, which is given an <a id="elements-in-the-dom&#58;the-id-attribute-3"></a><code>[id](https://html.spec.whatwg.org/multipage/dom.html#the-id-attribute)</code> attribute:

``` text
<figure id="module-script-graph">
  <img src="module-script-graph.svg"
       alt="Module A depends on module B, which depends
            on modules C and D.">
  <figcaption>Figure 27: a simple module graph</figcaption>
</figure>
```

A <a id="elements-in-the-dom&#58;hyperlink-3"></a>[hyperlink](https://html.spec.whatwg.org/multipage/links.html#hyperlink)-based <a id="elements-in-the-dom&#58;referenced"></a>[reference](#referenced) could be created using the <a id="elements-in-the-dom&#58;the-a-element"></a><code>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element)</code> element, like so:

``` text
As we can see in <a href="#module-script-graph">figure 27</a>, ...
```

However, there are many other ways of <a id="elements-in-the-dom&#58;referenced-2"></a>[referencing](#referenced) the <a id="elements-in-the-dom&#58;the-figure-element-2"></a><code>[figure](https://html.spec.whatwg.org/multipage/grouping-content.html#the-figure-element)</code> element, such as:

- "As depicted in the figure of modules A, B, C, and D..."

- "In Figure 27..." (without a hyperlink)

- "From the contents of the 'simple module graph' figure..."

- "In the figure below..." (but [this is discouraged](https://html.spec.whatwg.org/multipage/grouping-content.html#figure-note-about-references))

The basic interface, from which all the <a id="elements-in-the-dom&#58;html-elements-3"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements)' interfaces inherit, and which must be used by elements that have no additional requirements, is the <a id="elements-in-the-dom&#58;htmlelement"></a><code>[HTMLElement](#htmlelement)</code> interface.

<strong>✔</strong>MDN

[HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement)

Support in all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera8+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer5.5+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android1+Samsung Internet?Opera Android10.1+

[HTMLElement/title](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/title)

Support in all current engines.

Firefox1+Safari3+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer5.5+

------------------------------------------------------------------------

Firefox Android?Safari iOS1+Chrome Android?WebView Android?Samsung Internet?Opera Android12.1+

[HTMLElement/lang](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/lang)

Support in all current engines.

Firefox1+Safari3+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer5.5+

------------------------------------------------------------------------

Firefox Android?Safari iOS1+Chrome Android?WebView Android?Samsung Internet?Opera Android12.1+

[HTMLElement/inert](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/inert)

Support in all current engines.

Firefox112+Safari15.5+Chrome102+

------------------------------------------------------------------------

Opera?Edge102+

------------------------------------------------------------------------

Edge (Legacy)?Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android?

[HTMLElement/accessKey](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/accessKey)

Support in all current engines.

Firefox5+Safari6+Chrome17+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer5.5+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android12.1+

[HTMLUnknownElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLUnknownElement)

Support in all current engines.

Firefox1+Safari6+Chrome15+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer9+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android12.1+

<a id="htmlelement"></a><a id="elements-in-the-dom&#58;element"></a><a id="elements-in-the-dom&#58;htmlconstructor"></a><a id="elements-in-the-dom&#58;cereactions"></a><a id="elements-in-the-dom&#58;xattr-reflect"></a><a id="dom-title"></a><a id="elements-in-the-dom&#58;cereactions-2"></a><a id="elements-in-the-dom&#58;xattr-reflect-2"></a><a id="dom-lang"></a><a id="elements-in-the-dom&#58;cereactions-3"></a><a id="elements-in-the-dom&#58;dom-translate"></a><a id="elements-in-the-dom&#58;cereactions-4"></a><a id="elements-in-the-dom&#58;dom-dir"></a><a id="elements-in-the-dom&#58;editing"></a><a id="elements-in-the-dom&#58;cereactions-5"></a><a id="elements-in-the-dom&#58;dom-hidden"></a><a id="elements-in-the-dom&#58;cereactions-6"></a><a id="elements-in-the-dom&#58;xattr-reflect-3"></a><a id="dom-inert"></a><a id="elements-in-the-dom&#58;dom-click"></a><a id="elements-in-the-dom&#58;cereactions-7"></a><a id="elements-in-the-dom&#58;xattr-reflect-4"></a><a id="dom-accesskey"></a><a id="elements-in-the-dom&#58;dom-accesskeylabel"></a><a id="elements-in-the-dom&#58;cereactions-8"></a><a id="elements-in-the-dom&#58;dom-draggable"></a><a id="elements-in-the-dom&#58;cereactions-9"></a><a id="elements-in-the-dom&#58;dom-spellcheck"></a><a id="elements-in-the-dom&#58;cereactions-10"></a><a id="elements-in-the-dom&#58;xattr-reflectsetter"></a><a id="elements-in-the-dom&#58;dom-writingsuggestions"></a><a id="elements-in-the-dom&#58;cereactions-11"></a><a id="elements-in-the-dom&#58;xattr-reflectsetter-2"></a><a id="elements-in-the-dom&#58;dom-autocapitalize"></a><a id="elements-in-the-dom&#58;cereactions-12"></a><a id="elements-in-the-dom&#58;dom-autocorrect"></a><a id="elements-in-the-dom&#58;cereactions-13"></a><a id="elements-in-the-dom&#58;legacynulltoemptystring"></a><a id="elements-in-the-dom&#58;dom-innertext"></a><a id="elements-in-the-dom&#58;cereactions-14"></a><a id="elements-in-the-dom&#58;legacynulltoemptystring-2"></a><a id="elements-in-the-dom&#58;dom-outertext"></a><a id="elements-in-the-dom&#58;elementinternals"></a><a id="elements-in-the-dom&#58;dom-attachinternals"></a><a id="elements-in-the-dom&#58;dom-showpopover"></a><a id="elements-in-the-dom&#58;showpopoveroptions"></a><a id="elements-in-the-dom&#58;dom-hidepopover"></a><a id="elements-in-the-dom&#58;dom-togglepopover"></a><a id="elements-in-the-dom&#58;togglepopoveroptions"></a><a id="elements-in-the-dom&#58;cereactions-15"></a><a id="elements-in-the-dom&#58;dom-popover"></a><a id="elements-in-the-dom&#58;cereactions-16"></a><a id="elements-in-the-dom&#58;xattr-reflect-5"></a><a id="elements-in-the-dom&#58;xattr-reflectrange"></a><a id="dom-headingoffset"></a><a id="elements-in-the-dom&#58;cereactions-17"></a><a id="elements-in-the-dom&#58;xattr-reflect-6"></a><a id="dom-headingreset"></a><a id="showpopoveroptions"></a><a id="elements-in-the-dom&#58;htmlelement-2"></a><a id="dom-showpopoveroptions-source"></a><a id="togglepopoveroptions"></a><a id="elements-in-the-dom&#58;showpopoveroptions-2"></a><a id="dom-togglepopoveroptions-force"></a><a id="elements-in-the-dom&#58;htmlelement-3"></a><a id="elements-in-the-dom&#58;globaleventhandlers"></a><a id="elements-in-the-dom&#58;htmlelement-4"></a><a id="elements-in-the-dom&#58;elementcontenteditable"></a><a id="elements-in-the-dom&#58;htmlelement-5"></a><a id="elements-in-the-dom&#58;htmlorsvgelement"></a><a id="htmlunknownelement"></a><a id="elements-in-the-dom&#58;htmlelement-6"></a><a id="elements-in-the-dom&#58;htmlconstructor-2"></a>

``` text
[Exposed=Window]
interface HTMLElement : Element {
  [HTMLConstructor] constructor();

  // metadata attributes
  [CEReactions, Reflect] attribute DOMString title;
  [CEReactions, Reflect] attribute DOMString lang;
  [CEReactions] attribute boolean translate;
  [CEReactions] attribute DOMString dir;

  // user interaction
  [CEReactions] attribute (boolean or unrestricted double or DOMString)? hidden;
  [CEReactions, Reflect] attribute boolean inert;
  undefined click();
  [CEReactions, Reflect] attribute DOMString accessKey;
  readonly attribute DOMString accessKeyLabel;
  [CEReactions] attribute boolean draggable;
  [CEReactions] attribute boolean spellcheck;
  [CEReactions, ReflectSetter] attribute DOMString writingSuggestions;
  [CEReactions, ReflectSetter] attribute DOMString autocapitalize;
  [CEReactions] attribute boolean autocorrect;

  [CEReactions] attribute [LegacyNullToEmptyString] DOMString innerText;
  [CEReactions] attribute [LegacyNullToEmptyString] DOMString outerText;

  ElementInternals attachInternals();

  // The popover API
  undefined showPopover(optional ShowPopoverOptions options = {});
  undefined hidePopover();
  boolean togglePopover(optional (TogglePopoverOptions or boolean) options = {});
  [CEReactions] attribute DOMString? popover;

  [CEReactions, Reflect, ReflectRange=(0, 8)] attribute unsigned long headingOffset;
  [CEReactions, Reflect] attribute boolean headingReset;
};

dictionary ShowPopoverOptions {
  HTMLElement source;
};

dictionary TogglePopoverOptions : ShowPopoverOptions {
  boolean force;
};

HTMLElement includes GlobalEventHandlers;
HTMLElement includes ElementContentEditable;
HTMLElement includes HTMLOrSVGOrMathMLElement;

[Exposed=Window]
interface HTMLUnknownElement : HTMLElement {
  // Note: intentionally no [HTMLConstructor]
};
```

The <a id="elements-in-the-dom&#58;htmlelement-7"></a><code>[HTMLElement](#htmlelement)</code> interface holds methods and attributes related to a number of disparate features, and the members of this interface are therefore described in various different sections of this specification.

------------------------------------------------------------------------

The <a id="elements-in-the-dom&#58;element-interface"></a>[element interface](https://dom.spec.whatwg.org/#concept-element-interface) for an element with name <var>name</var> in the <a id="elements-in-the-dom&#58;html-namespace-2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) is determined as follows:

1.  If <var>name</var> is <a id="elements-in-the-dom&#58;applet"></a><code>[applet](https://html.spec.whatwg.org/multipage/obsolete.html#applet)</code>, <a id="elements-in-the-dom&#58;bgsound"></a><code>[bgsound](https://html.spec.whatwg.org/multipage/obsolete.html#bgsound)</code>, <a id="elements-in-the-dom&#58;blink"></a><code>[blink](https://html.spec.whatwg.org/multipage/obsolete.html#blink)</code>, <a id="elements-in-the-dom&#58;isindex"></a><code>[isindex](https://html.spec.whatwg.org/multipage/obsolete.html#isindex)</code>, <a id="elements-in-the-dom&#58;keygen"></a><code>[keygen](https://html.spec.whatwg.org/multipage/obsolete.html#keygen)</code>, <a id="elements-in-the-dom&#58;multicol"></a><code>[multicol](https://html.spec.whatwg.org/multipage/obsolete.html#multicol)</code>, <a id="elements-in-the-dom&#58;nextid"></a><code>[nextid](https://html.spec.whatwg.org/multipage/obsolete.html#nextid)</code>, or <a id="elements-in-the-dom&#58;spacer"></a><code>[spacer](https://html.spec.whatwg.org/multipage/obsolete.html#spacer)</code>, then return <a id="elements-in-the-dom&#58;htmlunknownelement"></a><code>[HTMLUnknownElement](#htmlunknownelement)</code>.

2.  If <var>name</var> is <a id="elements-in-the-dom&#58;acronym"></a><code>[acronym](https://html.spec.whatwg.org/multipage/obsolete.html#acronym)</code>, <a id="elements-in-the-dom&#58;basefont"></a><code>[basefont](https://html.spec.whatwg.org/multipage/obsolete.html#basefont)</code>, <a id="elements-in-the-dom&#58;big"></a><code>[big](https://html.spec.whatwg.org/multipage/obsolete.html#big)</code>, <a id="elements-in-the-dom&#58;center"></a><code>[center](https://html.spec.whatwg.org/multipage/obsolete.html#center)</code>, <a id="elements-in-the-dom&#58;nobr"></a><code>[nobr](https://html.spec.whatwg.org/multipage/obsolete.html#nobr)</code>, <a id="elements-in-the-dom&#58;noembed"></a><code>[noembed](https://html.spec.whatwg.org/multipage/obsolete.html#noembed)</code>, <a id="elements-in-the-dom&#58;noframes"></a><code>[noframes](https://html.spec.whatwg.org/multipage/obsolete.html#noframes)</code>, <a id="elements-in-the-dom&#58;plaintext"></a><code>[plaintext](https://html.spec.whatwg.org/multipage/obsolete.html#plaintext)</code>, <a id="elements-in-the-dom&#58;rb"></a><code>[rb](https://html.spec.whatwg.org/multipage/obsolete.html#rb)</code>, <a id="elements-in-the-dom&#58;rtc"></a><code>[rtc](https://html.spec.whatwg.org/multipage/obsolete.html#rtc)</code>, <a id="elements-in-the-dom&#58;strike"></a><code>[strike](https://html.spec.whatwg.org/multipage/obsolete.html#strike)</code>, or <a id="elements-in-the-dom&#58;tt"></a><code>[tt](https://html.spec.whatwg.org/multipage/obsolete.html#tt)</code>, then return <a id="elements-in-the-dom&#58;htmlelement-8"></a><code>[HTMLElement](#htmlelement)</code>.

3.  If <var>name</var> is <a id="elements-in-the-dom&#58;listing"></a><code>[listing](https://html.spec.whatwg.org/multipage/obsolete.html#listing)</code> or <a id="elements-in-the-dom&#58;xmp"></a><code>[xmp](https://html.spec.whatwg.org/multipage/obsolete.html#xmp)</code>, then return <a id="elements-in-the-dom&#58;htmlpreelement"></a><code>[HTMLPreElement](https://html.spec.whatwg.org/multipage/grouping-content.html#htmlpreelement)</code>.

4.  Otherwise, if this specification defines an interface appropriate for the <a id="elements-in-the-dom&#58;element-type"></a>[element type](https://html.spec.whatwg.org/multipage/infrastructure.html#element-type) corresponding to the local name <var>name</var>, then return that interface.

5.  If <a id="elements-in-the-dom&#58;other-applicable-specifications"></a>[other applicable specifications](https://html.spec.whatwg.org/multipage/infrastructure.html#other-applicable-specifications) define an appropriate interface for <var>name</var>, then return the interface they define.

6.  If <var>name</var> is a <a id="elements-in-the-dom&#58;valid-custom-element-name"></a>[valid custom element name](https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name), then return <a id="elements-in-the-dom&#58;htmlelement-9"></a><code>[HTMLElement](#htmlelement)</code>.

7.  Return <a id="elements-in-the-dom&#58;htmlunknownelement-2"></a><code>[HTMLUnknownElement](#htmlunknownelement)</code>.

<strong>Note:</strong>

The use of <a id="elements-in-the-dom&#58;htmlelement-10"></a><code>[HTMLElement](#htmlelement)</code> instead of <a id="elements-in-the-dom&#58;htmlunknownelement-3"></a><code>[HTMLUnknownElement](#htmlunknownelement)</code> in the case of <a id="elements-in-the-dom&#58;valid-custom-element-name-2"></a>[valid custom element names](https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name) is done to ensure that any potential future <a id="elements-in-the-dom&#58;upgrades"></a>[upgrades](https://html.spec.whatwg.org/multipage/custom-elements.html#upgrades) only cause a linear transition of the element's prototype chain, from <a id="elements-in-the-dom&#58;htmlelement-11"></a><code>[HTMLElement](#htmlelement)</code> to a subclass, instead of a lateral one, from <a id="elements-in-the-dom&#58;htmlunknownelement-4"></a><code>[HTMLUnknownElement](#htmlunknownelement)</code> to an unrelated subclass.

Features shared between HTML, SVG and MathML elements use the <a id="elements-in-the-dom&#58;htmlorsvgelement-2"></a><code>[HTMLOrSVGOrMathMLElement](#htmlorsvgelement)</code> interface mixin: [\[SVG\]](https://html.spec.whatwg.org/multipage/references.html#refsSVG) [\[MATHML\]](https://html.spec.whatwg.org/multipage/references.html#refsMATHML)

<strong>⚠</strong>MDN

[Global_attributes/autofocus](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/autofocus)

Support in one engine only.

Firefox🔰 1+Safari🔰 4+Chrome79+

------------------------------------------------------------------------

Opera66+Edge79+

------------------------------------------------------------------------

Edge (Legacy)NoInternet Explorer🔰 10+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android79+Samsung Internet?Opera Android57+

<a id="htmlorsvgelement"></a><a id="elements-in-the-dom&#58;domstringmap"></a><a id="elements-in-the-dom&#58;dom-dataset"></a><a id="elements-in-the-dom&#58;dom-noncedelement-nonce"></a><a id="elements-in-the-dom&#58;cereactions-18"></a><a id="elements-in-the-dom&#58;xattr-reflect-7"></a><a id="dom-fe-autofocus"></a><a id="elements-in-the-dom&#58;cereactions-19"></a><a id="elements-in-the-dom&#58;xattr-reflectsetter-3"></a><a id="elements-in-the-dom&#58;dom-tabindex"></a><a id="elements-in-the-dom&#58;dom-focus"></a><a id="elements-in-the-dom&#58;focusoptions"></a><a id="elements-in-the-dom&#58;dom-blur"></a>

``` text
interface mixin HTMLOrSVGOrMathMLElement {
  [SameObject] readonly attribute DOMStringMap dataset;
  attribute DOMString nonce; // intentionally no [CEReactions]

  [CEReactions, Reflect] attribute boolean autofocus;
  [CEReactions, ReflectSetter] attribute long tabIndex;
  undefined focus(optional FocusOptions options = {});
  undefined blur();
};
```

<strong>Example:</strong>

An example of an element that is neither an HTML nor SVG nor MathML element is one created as follows:

``` text
const el = document.createElementNS("some namespace", "example");
console.assert(el.constructor === Element);
```

##### <a id="the-lang-and-xml&#58;lang-attributes"></a>3.2.6.2 The <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-lang"></a><code>[lang](#attr-lang)</code> and <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-xml-lang"></a><code>[xml:lang](https://www.w3.org/TR/xml/#sec-lang-tag)</code> attributes[](#the-lang-and-xml:lang-attributes)

<strong>✔</strong>MDN

[Global_attributes/lang](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/lang)

Support in all current engines.

Firefox1+Safari4+Chrome1+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android?

The <a id="attr-lang"></a><strong><code>lang</code></strong> attribute (in no namespace) specifies the primary language for the element's contents and for any of the element's attributes that contain text. Its value must be a valid BCP 47 language tag, or the empty string. Setting the attribute to the empty string indicates that the primary language is unknown. [\[BCP47\]](https://html.spec.whatwg.org/multipage/references.html#refsBCP47)

The <a id="attr-xml-lang"></a><strong>[<code>lang</code>](https://www.w3.org/TR/xml/#sec-lang-tag)</strong> attribute in the <a id="the-lang-and-xml&#58;lang-attributes&#58;xml-namespace"></a>[XML namespace](https://infra.spec.whatwg.org/#xml-namespace) is defined in XML. [\[XML\]](https://html.spec.whatwg.org/multipage/references.html#refsXML)

If these attributes are omitted from an element, then the language of this element is the same as the language of its parent element, if any (except for <a id="the-lang-and-xml&#58;lang-attributes&#58;the-slot-element"></a><code>[slot](https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element)</code> elements in a <a id="the-lang-and-xml&#58;lang-attributes&#58;shadow-tree"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree)).

The <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-lang-2"></a><code>[lang](#attr-lang)</code> attribute in no namespace may be used on any <a id="the-lang-and-xml&#58;lang-attributes&#58;html-elements"></a>[HTML element](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements).

The <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-xml-lang-2"></a>[<code>lang</code> attribute in the XML namespace](https://www.w3.org/TR/xml/#sec-lang-tag) may be used on <a id="the-lang-and-xml&#58;lang-attributes&#58;html-elements-2"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in <a id="the-lang-and-xml&#58;lang-attributes&#58;xml-documents"></a>[XML documents](https://dom.spec.whatwg.org/#xml-document), as well as elements in other namespaces if the relevant specifications allow it (in particular, MathML and SVG allow <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-xml-lang-3"></a>[<code>lang</code> attributes in the XML namespace](https://www.w3.org/TR/xml/#sec-lang-tag) to be specified on their elements). If both the <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-lang-3"></a><code>[lang](#attr-lang)</code> attribute in no namespace and the <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-xml-lang-4"></a>[<code>lang</code> attribute in the XML namespace](https://www.w3.org/TR/xml/#sec-lang-tag) are specified on the same element, they must have exactly the same value when compared in an <a id="the-lang-and-xml&#58;lang-attributes&#58;ascii-case-insensitive"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) manner.

Authors must not use the <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-xml-lang-5"></a>[<code>lang</code> attribute in the XML namespace](https://www.w3.org/TR/xml/#sec-lang-tag) on <a id="the-lang-and-xml&#58;lang-attributes&#58;html-elements-3"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in <a id="the-lang-and-xml&#58;lang-attributes&#58;html-documents"></a>[HTML documents](https://dom.spec.whatwg.org/#html-document). To ease migration to and from XML, authors may specify an attribute in no namespace with no prefix and with the literal localname "<code>xml&#58;lang</code>" on <a id="the-lang-and-xml&#58;lang-attributes&#58;html-elements-4"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in <a id="the-lang-and-xml&#58;lang-attributes&#58;html-documents-2"></a>[HTML documents](https://dom.spec.whatwg.org/#html-document), but such attributes must only be specified if a <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-lang-4"></a><code>[lang](#attr-lang)</code> attribute in no namespace is also specified, and both attributes must have the same value when compared in an <a id="the-lang-and-xml&#58;lang-attributes&#58;ascii-case-insensitive-2"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) manner.

<strong>Note:</strong>

The attribute in no namespace with no prefix and with the literal localname "<code>xml&#58;lang</code>" has no effect on language processing.

------------------------------------------------------------------------

To determine the <a id="language"></a><strong>language</strong> of a node, user agents must use the first appropriate step in the following list:

<strong>If the node is an element that has a <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-xml-lang-6"></a>[<code>lang</code> attribute in the XML namespace](https://www.w3.org/TR/xml/#sec-lang-tag) set</strong>

Use the value of that attribute.

<strong>If the node is an <a id="the-lang-and-xml&#58;lang-attributes&#58;html-elements-5"></a>[HTML element](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) or an element in the <a id="the-lang-and-xml&#58;lang-attributes&#58;svg-namespace"></a>[SVG namespace](https://infra.spec.whatwg.org/#svg-namespace), and it has a <a id="the-lang-and-xml&#58;lang-attributes&#58;attr-lang-5"></a><code>[lang](#attr-lang)</code> in no namespace attribute set</strong>

Use the value of that attribute.

<strong>If the node's parent is a <a id="the-lang-and-xml&#58;lang-attributes&#58;shadow-root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root)</strong>

Use the <a id="the-lang-and-xml&#58;lang-attributes&#58;language"></a>[language](#language) of that <a id="the-lang-and-xml&#58;lang-attributes&#58;shadow-root-2"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root)'s <a id="the-lang-and-xml&#58;lang-attributes&#58;concept-documentfragment-host"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host).

<strong>If the node's <a id="the-lang-and-xml&#58;lang-attributes&#58;parent-element"></a>[parent element](https://dom.spec.whatwg.org/#parent-element) is not null</strong>

Use the <a id="the-lang-and-xml&#58;lang-attributes&#58;language-2"></a>[language](#language) of that <a id="the-lang-and-xml&#58;lang-attributes&#58;parent-element-2"></a>[parent element](https://dom.spec.whatwg.org/#parent-element).

<strong>Otherwise</strong>

If there is a <a id="the-lang-and-xml&#58;lang-attributes&#58;pragma-set-default-language"></a>[pragma-set default language](https://html.spec.whatwg.org/multipage/semantics.html#pragma-set-default-language) set, then that is the language of the node. If there is no <a id="the-lang-and-xml&#58;lang-attributes&#58;pragma-set-default-language-2"></a>[pragma-set default language](https://html.spec.whatwg.org/multipage/semantics.html#pragma-set-default-language) set, then language information from a higher-level protocol (such as HTTP), if any, must be used as the final fallback language instead. In the absence of any such language information, and in cases where the higher-level protocol reports multiple languages, the language of the node is unknown, and the corresponding language tag is the empty string.

If the resulting value is not a recognized language tag, then it must be treated as an unknown language having the given language tag, distinct from all other languages. For the purposes of roundtripping or communicating with other services that expect language tags, user agents should pass unknown language tags through unmodified, and tagged as being BCP 47 language tags, so that subsequent services do not interpret the data as another type of language description. [\[BCP47\]](https://html.spec.whatwg.org/multipage/references.html#refsBCP47)

<strong>Example:</strong>

Thus, for instance, an element with <code>lang=&#34;xyzzy&#34;</code> would be matched by the selector <code>&#58;lang(xyzzy)</code> (e.g. in CSS), but it would not be matched by <code>&#58;lang(abcde)</code>, even though both are equally invalid. Similarly, if a web browser and screen reader working in unison communicated about the language of the element, the browser would tell the screen reader that the language was "xyzzy", even if it knew it was invalid, just in case the screen reader actually supported a language with that tag after all. Even if the screen reader supported both BCP 47 and another syntax for encoding language names, and in that other syntax the string "xyzzy" was a way to denote the Belarusian language, it would be <em>incorrect</em> for the screen reader to then start treating text as Belarusian, because "xyzzy" is not how Belarusian is described in BCP 47 codes (BCP 47 uses the code "be" for Belarusian).

If the resulting value is the empty string, then it must be interpreted as meaning that the language of the node is <a id="concept-explicitly-unknown"></a><strong>explicitly unknown</strong>.

------------------------------------------------------------------------

User agents may use the element's language to determine proper processing or rendering (e.g. in the selection of appropriate fonts or pronunciations, for dictionary selection, or for the user interfaces of form controls such as date pickers).

##### <a id="the-dir-attribute"></a>3.2.6.4 The <a id="the-dir-attribute&#58;attr-dir"></a><code>[dir](#attr-dir)</code> attribute[](#the-dir-attribute)

<strong>✔</strong>MDN

[Global_attributes/dir](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/dir)

Support in all current engines.

Firefox1+Safari4+Chrome1+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android?

The <a id="attr-dir"></a><strong><code>dir</code></strong> attribute is an <a id="the-dir-attribute&#58;enumerated-attribute"></a>[enumerated attribute](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#enumerated-attribute) with the following keywords and states:

<table><thead><tr><th>Keyword      </th><th>State      </th><th>Brief description    </th></tr></thead><tbody><tr><td><dfn data-dfn-for="html-global/dir" id="attr-dir-ltr" data-dfn-type="attr-value"><code>ltr</code></dfn>      </td><td><dfn id="attr-dir-ltr-state">LTR</dfn>      </td><td>The contents of the element are explicitly directionally isolated left-to-right text.     </td></tr><tr><td><dfn data-dfn-for="html-global/dir" id="attr-dir-rtl" data-dfn-type="attr-value"><code>rtl</code></dfn>      </td><td><dfn id="attr-dir-rtl-state">RTL</dfn>      </td><td>The contents of the element are explicitly directionally isolated right-to-left text.     </td></tr><tr><td><dfn data-dfn-for="html-global/dir" id="attr-dir-auto" data-dfn-type="attr-value"><code>auto</code></dfn>      </td><td><dfn id="attr-dir-auto-state">Auto</dfn>      </td><td>The contents of the element are explicitly directionally isolated text, but the direction      is to be determined programmatically using the contents of the element (as described      below).   </td></tr></tbody></table>

<strong>Note:</strong>

The heuristic used by the <a id="the-dir-attribute&#58;attr-dir-auto-state"></a>[Auto](#attr-dir-auto-state) state is very crude (it just looks at the first character with a strong directionality, in a manner analogous to the Paragraph Level determination in the bidirectional algorithm). Authors are urged to only use this value as a last resort when the direction of the text is truly unknown and no better server-side heuristic can be applied. [\[BIDI\]](https://html.spec.whatwg.org/multipage/references.html#refsBIDI)

For <a id="the-dir-attribute&#58;the-textarea-element"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> and <a id="the-dir-attribute&#58;the-pre-element"></a><code>[pre](https://html.spec.whatwg.org/multipage/grouping-content.html#the-pre-element)</code> elements, the heuristic is applied on a per-paragraph level.

The attribute's <a id="the-dir-attribute&#58;missing-value-default"></a><em>[missing value default](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#missing-value-default)</em> and <a id="the-dir-attribute&#58;invalid-value-default"></a><em>[invalid value default](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#invalid-value-default)</em> are both the <a id="attr-dir-undefined-state"></a><strong>Undefined</strong> state.

------------------------------------------------------------------------

The <a id="the-directionality"></a><strong>directionality</strong> of an element (any element, not just an <a id="the-dir-attribute&#58;html-elements"></a>[HTML element](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements)) is either '<a id="concept-ltr"></a><strong>ltr</strong>' or '<a id="concept-rtl"></a><strong>rtl</strong>'. To compute the <a id="the-dir-attribute&#58;the-directionality"></a>[directionality](#the-directionality) given an element <var>element</var>, switch on <var>element</var>'s <a id="the-dir-attribute&#58;attr-dir-2"></a><code>[dir](#attr-dir)</code> attribute state:

<strong><a id="the-dir-attribute&#58;attr-dir-ltr-state"></a>[LTR](#attr-dir-ltr-state)</strong>

Return '<a id="the-dir-attribute&#58;concept-ltr"></a>[ltr](#concept-ltr)'.

<strong><a id="the-dir-attribute&#58;attr-dir-rtl-state"></a>[RTL](#attr-dir-rtl-state)</strong>

Return '<a id="the-dir-attribute&#58;concept-rtl"></a>[rtl](#concept-rtl)'.

<strong><a id="the-dir-attribute&#58;attr-dir-auto-state-2"></a>[Auto](#attr-dir-auto-state)</strong>

1.  Let <var>result</var> be the <a id="the-dir-attribute&#58;auto-directionality"></a>[auto directionality](#auto-directionality) of <var>element</var>.

2.  If <var>result</var> is null, then return '<a id="the-dir-attribute&#58;concept-ltr-2"></a>[ltr](#concept-ltr)'.

3.  Return <var>result</var>.

<strong><a id="the-dir-attribute&#58;attr-dir-undefined-state"></a>[Undefined](#attr-dir-undefined-state)</strong>

<strong>If <var>element</var> is a <a id="the-dir-attribute&#58;the-bdi-element"></a><code>[bdi](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-bdi-element)</code> element</strong>

1.  Let <var>result</var> be the <a id="the-dir-attribute&#58;auto-directionality-2"></a>[auto directionality](#auto-directionality) of <var>element</var>.

2.  If <var>result</var> is null, then return '<a id="the-dir-attribute&#58;concept-ltr-3"></a>[ltr](#concept-ltr)'.

3.  Return <var>result</var>.

<strong>If <var>element</var> is an <a id="the-dir-attribute&#58;the-input-element"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> element whose <a id="the-dir-attribute&#58;attr-input-type"></a><code>[type](https://html.spec.whatwg.org/multipage/input.html#attr-input-type)</code> attribute is in the <a id="the-dir-attribute&#58;telephone-state-(type=tel)"></a>[Telephone](https://html.spec.whatwg.org/multipage/input.html#telephone-state-(type=tel)) state</strong>

Return '<a id="the-dir-attribute&#58;concept-ltr-4"></a>[ltr](#concept-ltr)'.

<strong>Otherwise</strong>

Return the <a id="the-dir-attribute&#58;parent-directionality"></a>[parent directionality](#parent-directionality) of <var>element</var>.

<strong>Note:</strong>

Since the <a id="the-dir-attribute&#58;attr-dir-3"></a><code>[dir](#attr-dir)</code> attribute is only defined for <a id="the-dir-attribute&#58;html-elements-2"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements), it cannot be present on elements from other namespaces. Thus, elements from other namespaces always end up using the <a id="the-dir-attribute&#58;parent-directionality-2"></a>[parent directionality](#parent-directionality).

The <a id="auto-directionality-form-associated-elements"></a><strong>auto-directionality form-associated elements</strong> are:

- <a id="the-dir-attribute&#58;the-input-element-2"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements whose <a id="the-dir-attribute&#58;attr-input-type-2"></a><code>[type](https://html.spec.whatwg.org/multipage/input.html#attr-input-type)</code> attribute is in the <a id="the-dir-attribute&#58;hidden-state-(type=hidden)"></a>[Hidden](https://html.spec.whatwg.org/multipage/input.html#hidden-state-(type=hidden)), <a id="the-dir-attribute&#58;text-(type=text)-state-and-search-state-(type=search)"></a>[Text](https://html.spec.whatwg.org/multipage/input.html#text-(type=text)-state-and-search-state-(type=search)), <a id="the-dir-attribute&#58;text-(type=text)-state-and-search-state-(type=search)-2"></a>[Search](https://html.spec.whatwg.org/multipage/input.html#text-(type=text)-state-and-search-state-(type=search)), <a id="the-dir-attribute&#58;telephone-state-(type=tel)-2"></a>[Telephone](https://html.spec.whatwg.org/multipage/input.html#telephone-state-(type=tel)), <a id="the-dir-attribute&#58;url-state-(type=url)"></a>[URL](https://html.spec.whatwg.org/multipage/input.html#url-state-(type=url)), <a id="the-dir-attribute&#58;email-state-(type=email)"></a>[Email](https://html.spec.whatwg.org/multipage/input.html#email-state-(type=email)), <a id="the-dir-attribute&#58;password-state-(type=password)"></a>[Password](https://html.spec.whatwg.org/multipage/input.html#password-state-(type=password)), <a id="the-dir-attribute&#58;submit-button-state-(type=submit)"></a>[Submit Button](https://html.spec.whatwg.org/multipage/input.html#submit-button-state-(type=submit)), <a id="the-dir-attribute&#58;reset-button-state-(type=reset)"></a>[Reset Button](https://html.spec.whatwg.org/multipage/input.html#reset-button-state-(type=reset)), or <a id="the-dir-attribute&#58;button-state-(type=button)"></a>[Button](https://html.spec.whatwg.org/multipage/input.html#button-state-(type=button)) state, and

- <a id="the-dir-attribute&#58;the-textarea-element-2"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> elements.

To compute the <a id="auto-directionality"></a><strong>auto directionality</strong> given an element <var>element</var>:

1.  If <var>element</var> is an <a id="the-dir-attribute&#58;auto-directionality-form-associated-elements"></a>[auto-directionality form-associated element](#auto-directionality-form-associated-elements):

    1.  If <var>element</var>'s <a id="the-dir-attribute&#58;concept-fe-value"></a>[value](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-value) contains a character of bidirectional character type AL or R, and there is no character of bidirectional character type L anywhere before it in the element's <a id="the-dir-attribute&#58;concept-fe-value-2"></a>[value](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-value), then return '<a id="the-dir-attribute&#58;concept-rtl-2"></a>[rtl](#concept-rtl)'. [\[BIDI\]](https://html.spec.whatwg.org/multipage/references.html#refsBIDI)

    2.  If <var>element</var>'s <a id="the-dir-attribute&#58;concept-fe-value-3"></a>[value](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-value) is not the empty string, then return '<a id="the-dir-attribute&#58;concept-ltr-5"></a>[ltr](#concept-ltr)'.

    3.  Return null.

2.  If <var>element</var> is a <a id="the-dir-attribute&#58;the-slot-element"></a><code>[slot](https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element)</code> element whose <a id="the-dir-attribute&#58;root"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) is a <a id="the-dir-attribute&#58;shadow-root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) and <var>element</var>'s <a id="the-dir-attribute&#58;assigned-nodes"></a>[assigned nodes](https://dom.spec.whatwg.org/#slot-assigned-nodes) are not empty:

    1.  <a id="the-dir-attribute&#58;list-iterate"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) node <var>child</var> of <var>element</var>'s <a id="the-dir-attribute&#58;assigned-nodes-2"></a>[assigned nodes](https://dom.spec.whatwg.org/#slot-assigned-nodes):

        1.  Let <var>childDirection</var> be null.

        2.  If <var>child</var> is a <a id="the-dir-attribute&#58;text"></a><code>[Text](https://dom.spec.whatwg.org/#interface-text)</code> node, then set <var>childDirection</var> to the <a id="the-dir-attribute&#58;text-node-directionality"></a>[text node directionality](#text-node-directionality) of <var>child</var>.

        3.  Otherwise:

            1.  <a id="the-dir-attribute&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>child</var> is an <a id="the-dir-attribute&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code> node.

            2.  Set <var>childDirection</var> to the <a id="the-dir-attribute&#58;contained-text-auto-directionality"></a>[contained text auto directionality](#contained-text-auto-directionality) of <var>child</var> with <em><a id="the-dir-attribute&#58;auto-directionality-can-exclude-root"></a>[canExcludeRoot](#auto-directionality-can-exclude-root)</em> set to true.

        4.  If <var>childDirection</var> is not null, then return <var>childDirection</var>.

    2.  Return null.

3.  Return the <a id="the-dir-attribute&#58;contained-text-auto-directionality-2"></a>[contained text auto directionality](#contained-text-auto-directionality) of <var>element</var> with <em><a id="the-dir-attribute&#58;auto-directionality-can-exclude-root-2"></a>[canExcludeRoot](#auto-directionality-can-exclude-root)</em> set to false.

To compute the <a id="contained-text-auto-directionality"></a><strong>contained text auto directionality</strong> of an element <var>element</var> with a boolean <a id="auto-directionality-can-exclude-root"></a><strong><var>canExcludeRoot</var></strong>:

1.  <a id="the-dir-attribute&#58;list-iterate-2"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) node <var>descendant</var> of <var>element</var>'s <a id="the-dir-attribute&#58;descendant"></a>[descendants](https://dom.spec.whatwg.org/#concept-tree-descendant), in <a id="the-dir-attribute&#58;tree-order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order):

    1.  If any of

        - <var>descendant</var>
        - any ancestor element of <var>descendant</var> that is a descendant of <var>element</var>
        - if <var>canExcludeRoot</var> is true, <var>element</var>

        is one of

        - a <a id="the-dir-attribute&#58;the-bdi-element-2"></a><code>[bdi](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-bdi-element)</code> element
        - a <a id="the-dir-attribute&#58;the-script-element"></a><code>[script](https://html.spec.whatwg.org/multipage/scripting.html#the-script-element)</code> element
        - a <a id="the-dir-attribute&#58;the-style-element"></a><code>[style](https://html.spec.whatwg.org/multipage/semantics.html#the-style-element)</code> element
        - a <a id="the-dir-attribute&#58;the-textarea-element-3"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> element
        - an element whose <a id="the-dir-attribute&#58;attr-dir-4"></a><code>[dir](#attr-dir)</code> attribute is not in the <a id="the-dir-attribute&#58;attr-dir-undefined-state-2"></a>[Undefined](#attr-dir-undefined-state) state

        then <a id="the-dir-attribute&#58;continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    2.  If <var>descendant</var> is a <a id="the-dir-attribute&#58;the-slot-element-2"></a><code>[slot](https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element)</code> element whose <a id="the-dir-attribute&#58;root-2"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) is a <a id="the-dir-attribute&#58;shadow-root-2"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root), then return the <a id="the-dir-attribute&#58;the-directionality-2"></a>[directionality](#the-directionality) of that <a id="the-dir-attribute&#58;shadow-root-3"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root)'s <a id="the-dir-attribute&#58;concept-documentfragment-host"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host).

    3.  If <var>descendant</var> is not a <a id="the-dir-attribute&#58;text-2"></a><code>[Text](https://dom.spec.whatwg.org/#interface-text)</code> node, then <a id="the-dir-attribute&#58;continue-2"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    4.  Let <var>result</var> be the <a id="the-dir-attribute&#58;text-node-directionality-2"></a>[text node directionality](#text-node-directionality) of <var>descendant</var>.

    5.  If <var>result</var> is not null, then return <var>result</var>.

2.  Return null.

To compute the <a id="text-node-directionality"></a><strong>text node directionality</strong> given a <a id="the-dir-attribute&#58;text-3"></a><code>[Text](https://dom.spec.whatwg.org/#interface-text)</code> node <var>text</var>:

1.  If <var>text</var>'s <a id="the-dir-attribute&#58;concept-cd-data"></a>[data](https://dom.spec.whatwg.org/#concept-cd-data) does not contain a code point whose bidirectional character type is L, AL, or R, then return null. [\[BIDI\]](https://html.spec.whatwg.org/multipage/references.html#refsBIDI)

2.  Let <var>codePoint</var> be the first code point in <var>text</var>'s <a id="the-dir-attribute&#58;concept-cd-data-2"></a>[data](https://dom.spec.whatwg.org/#concept-cd-data) whose bidirectional character type is L, AL, or R.

3.  If <var>codePoint</var> is of bidirectional character type AL or R, then return '<a id="the-dir-attribute&#58;concept-rtl-3"></a>[rtl](#concept-rtl)'.

4.  If <var>codePoint</var> is of bidirectional character type L, then return '<a id="the-dir-attribute&#58;concept-ltr-6"></a>[ltr](#concept-ltr)'.

To compute the <a id="parent-directionality"></a><strong>parent directionality</strong> given an element <var>element</var>:

1.  Let <var>parentNode</var> be <var>element</var>'s parent node.

2.  If <var>parentNode</var> is a <a id="the-dir-attribute&#58;shadow-root-4"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root), then return the <a id="the-dir-attribute&#58;the-directionality-3"></a>[directionality](#the-directionality) of <var>parentNode</var>'s <a id="the-dir-attribute&#58;concept-documentfragment-host-2"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host).

3.  If <var>parentNode</var> is an element, then return the <a id="the-dir-attribute&#58;the-directionality-4"></a>[directionality](#the-directionality) of <var>parentNode</var>.

4.  Return '<a id="the-dir-attribute&#58;concept-ltr-7"></a>[ltr](#concept-ltr)'.

<strong>Note:</strong>

This attribute [has rendering requirements involving the bidirectional algorithm](https://html.spec.whatwg.org/multipage/dom.html#bidireq).

------------------------------------------------------------------------

The <a id="directionality-of-the-attribute"></a><strong>directionality of an attribute</strong> of an <a id="the-dir-attribute&#58;html-elements-3"></a>[HTML element](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements), which is used when the text of that attribute is to be included in the rendering in some manner, is determined as per the first appropriate set of steps from the following list:

<strong>If the attribute is a <a id="the-dir-attribute&#58;directionality-capable-attribute"></a>[directionality-capable attribute](#directionality-capable-attribute) and the element's <a id="the-dir-attribute&#58;attr-dir-5"></a><code>[dir](#attr-dir)</code> attribute is in the <a id="the-dir-attribute&#58;attr-dir-auto-state-3"></a>[Auto](#attr-dir-auto-state) state</strong>

Find the first character (in logical order) of the attribute's value that is of bidirectional character type L, AL, or R. [\[BIDI\]](https://html.spec.whatwg.org/multipage/references.html#refsBIDI)

If such a character is found and it is of bidirectional character type AL or R, the <a id="the-dir-attribute&#58;directionality-of-the-attribute"></a>[directionality of the attribute](#directionality-of-the-attribute) is '<a id="the-dir-attribute&#58;concept-rtl-4"></a>[rtl](#concept-rtl)'.

Otherwise, the <a id="the-dir-attribute&#58;directionality-of-the-attribute-2"></a>[directionality of the attribute](#directionality-of-the-attribute) is '<a id="the-dir-attribute&#58;concept-ltr-8"></a>[ltr](#concept-ltr)'.

<strong>Otherwise</strong>

The <a id="the-dir-attribute&#58;directionality-of-the-attribute-3"></a>[directionality of the attribute](#directionality-of-the-attribute) is the same as <a id="the-dir-attribute&#58;the-directionality-5"></a>[the element's directionality](#the-directionality).

The following attributes are <a id="directionality-capable-attribute"></a><strong>directionality-capable attributes</strong>:

- <a id="the-dir-attribute&#58;attr-th-abbr"></a><code>[abbr](https://html.spec.whatwg.org/multipage/tables.html#attr-th-abbr)</code> on <a id="the-dir-attribute&#58;the-th-element"></a><code>[th](https://html.spec.whatwg.org/multipage/tables.html#the-th-element)</code> elements
- <code>alt</code> on <a id="the-dir-attribute&#58;attr-area-alt"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#attr-area-alt)</code>, <a id="the-dir-attribute&#58;attr-img-alt"></a><code>[img](https://html.spec.whatwg.org/multipage/embedded-content.html#attr-img-alt)</code>, and <a id="the-dir-attribute&#58;attr-input-alt"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#attr-input-alt)</code> elements
- <a id="the-dir-attribute&#58;attr-meta-content"></a><code>[content](https://html.spec.whatwg.org/multipage/semantics.html#attr-meta-content)</code> on <a id="the-dir-attribute&#58;the-meta-element"></a><code>[meta](https://html.spec.whatwg.org/multipage/semantics.html#the-meta-element)</code> elements, if the <a id="the-dir-attribute&#58;attr-meta-name"></a><code>[name](https://html.spec.whatwg.org/multipage/semantics.html#attr-meta-name)</code> attribute specifies a metadata name whose value is primarily intended to be human-readable rather than machine-readable
- <code>label</code> on <a id="the-dir-attribute&#58;attr-optgroup-label"></a><code>[optgroup](https://html.spec.whatwg.org/multipage/form-elements.html#attr-optgroup-label)</code>, <a id="the-dir-attribute&#58;attr-option-label"></a><code>[option](https://html.spec.whatwg.org/multipage/form-elements.html#attr-option-label)</code>, and <a id="the-dir-attribute&#58;attr-track-label"></a><code>[track](https://html.spec.whatwg.org/multipage/media.html#attr-track-label)</code> elements
- <code>placeholder</code> on <a id="the-dir-attribute&#58;attr-input-placeholder"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#attr-input-placeholder)</code> and <a id="the-dir-attribute&#58;attr-textarea-placeholder"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#attr-textarea-placeholder)</code> elements
- <a id="the-dir-attribute&#58;attr-title"></a><code>[title](https://html.spec.whatwg.org/multipage/dom.html#attr-title)</code> on all <a id="the-dir-attribute&#58;html-elements-4"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements)

------------------------------------------------------------------------

<strong><code><var>document</var>.<a id="dom-dir-dev"></a>[dir](#dom-dir) \[ = <var>value</var> \]</code></strong>

Returns <a id="the-dir-attribute&#58;the-html-element-2"></a>[the <code>html</code> element](https://html.spec.whatwg.org/multipage/dom.html#the-html-element-2)'s <a id="the-dir-attribute&#58;attr-dir-6"></a><code>[dir](#attr-dir)</code> attribute's value, if any.

Can be set, to either "<code>ltr</code>", "<code>rtl</code>", or "<code>auto</code>" to replace <a id="the-dir-attribute&#58;the-html-element-2-2"></a>[the <code>html</code> element](https://html.spec.whatwg.org/multipage/dom.html#the-html-element-2)'s <a id="the-dir-attribute&#58;attr-dir-7"></a><code>[dir](#attr-dir)</code> attribute's value.

If there is no <a id="the-dir-attribute&#58;the-html-element-2-3"></a>[<code>html</code> element](https://html.spec.whatwg.org/multipage/dom.html#the-html-element-2), returns the empty string and ignores new values.

<strong>✔</strong>MDN

[HTMLElement/dir](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dir)

Support in all current engines.

Firefox1+Safari3+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer5.5+

------------------------------------------------------------------------

Firefox Android?Safari iOS1+Chrome Android?WebView Android?Samsung Internet?Opera Android12.1+

The <a id="dom-dir"></a><strong><code>dir</code></strong> IDL attribute on an element must <a id="the-dir-attribute&#58;reflect"></a>[reflect](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#reflect) the <a id="the-dir-attribute&#58;attr-dir-8"></a><code>[dir](#attr-dir)</code> content attribute of that element, <a id="the-dir-attribute&#58;limited-to-only-known-values"></a>[limited to only known values](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#limited-to-only-known-values).

<strong>✔</strong>MDN

[Document/dir](https://developer.mozilla.org/en-US/docs/Web/API/Document/dir)

Support in all current engines.

Firefox1+Safari10.1+Chrome64+

------------------------------------------------------------------------

Opera51+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer5+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android47+

The <a id="dom-document-dir"></a><strong><code>dir</code></strong> IDL attribute on <a id="the-dir-attribute&#58;document"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> objects must <a id="the-dir-attribute&#58;reflect-2"></a>[reflect](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#reflect) the <a id="the-dir-attribute&#58;attr-dir-9"></a><code>[dir](#attr-dir)</code> content attribute of <a id="the-dir-attribute&#58;the-html-element-2-4"></a>[the <code>html</code> element](https://html.spec.whatwg.org/multipage/dom.html#the-html-element-2), if any, <a id="the-dir-attribute&#58;limited-to-only-known-values-2"></a>[limited to only known values](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#limited-to-only-known-values). If there is no such element, then the attribute must return the empty string and do nothing on setting.

<strong>Note:</strong>

Authors are strongly encouraged to use the <a id="the-dir-attribute&#58;attr-dir-10"></a><code>[dir](#attr-dir)</code> attribute to indicate text direction rather than using CSS, since that way their documents will continue to render correctly even in the absence of CSS (e.g. as interpreted by search engines).

<strong>Example:</strong>

This markup fragment is of an IM conversation.

``` text
<p dir=auto class="u1"><b><bdi>Student</bdi>:</b> How do you write "What's your name?" in Arabic?</p>
<p dir=auto class="u2"><b><bdi>Teacher</bdi>:</b> ما اسمك؟</p>
<p dir=auto class="u1"><b><bdi>Student</bdi>:</b> Thanks.</p>
<p dir=auto class="u2"><b><bdi>Teacher</bdi>:</b> That's written "شكرًا".</p>
<p dir=auto class="u2"><b><bdi>Teacher</bdi>:</b> Do you know how to write "Please"?</p>
<p dir=auto class="u1"><b><bdi>Student</bdi>:</b> "من فضلك", right?</p>
```

Given a suitable style sheet and the default alignment styles for the <a id="the-dir-attribute&#58;the-p-element"></a><code>[p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element)</code> element, namely to align the text to the <em>start edge</em> of the paragraph, the resulting rendering could be as follows:

![Each paragraph rendered as a separate block, with the paragraphs left-aligned except the second paragraph and the last one, which would be right aligned, with the usernames ('Student' and 'Teacher' in this example) flush right, with a colon to their left, and the text first to the left of that.](https://html.spec.whatwg.org/images/im.png)

As noted earlier, the <a id="the-dir-attribute&#58;attr-dir-auto"></a><code>[auto](#attr-dir-auto)</code> value is not a panacea. The final paragraph in this example is misinterpreted as being right-to-left text, since it begins with an Arabic character, which causes the "right?" to be to the left of the Arabic text.
