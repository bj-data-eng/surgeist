# HTML Standard: bounded DOM foundation source capture

Attribution and reformatting notice added for Surgeist on 2026-10-10. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative; this partial format conversion is implementation support, not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [HTML Standard](https://html.spec.whatwg.org/multipage/semantics-other.html). Retrieved 2026-10-10. Displayed status: HTMLLiving Standard — Last Updated 9 October 2026.

The exact upstream repository LICENSE files from DOM `3071e5f269448350fdea79bb284eedaf17bc431e`, HTML `877d6146487f15b15bc8dd0e3c625063a37b456a`, Web IDL `0620cf171d1b97138847088748e363cae4005156` and Infra `c67ae1fb1d161b23d0f8c9bf9917ea1993f7b19a` all match the already retained [WHATWG LICENSE](../licenses/whatwg/LICENSE) byte-for-byte, SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; the BSD 3-Clause scope applies when portions are incorporated into code, not as an alternative license for this Markdown.

Full retrieved HTML: 133,954 bytes, SHA-256 `8ea165c78170419eb4e18d91ba3fdc5264361b77de4d619f10829868f1addee1`. Complete-section extraction input: 105,937 bytes, SHA-256 `07d421b207b3c0f96d397d6a7b99e196a9af8a7db8d0ad4f1d357bbadde53cfe`. Source revision metadata: `No revision metadata in this chapter; GitHub HEAD association is not proven.`.

Retained complete heading sections, including descendant clauses and bounded at the next equal-or-higher-level heading: `disabled-elements`, `selectors`. Original source header is also retained. Every other clause and all cross-chapter references remain upstream. These are explicitly partial captures. HTML commit-snapshot chapter URLs for the stated repository HEAD returned HTTP 404; exact fetched chapter body hashes identify the captures without asserting that repository HEAD generated them.

Conversion uses existing repository HTML preparation/table-span models, approved Pandoc 3.1.11.1 and the semantic Lua filter. Source scripts/styles are omitted without execution. Source headings, IDs, links, prose, inline code, variables, literal blocks, images and every selected table cell/span are compared against parsed GFM. Selected tables remain literal semantic HTML, preserving every original table cell, code/link/variable, row/column span and anchor; table flattening is not used. Source examples and Web IDL remain evidence even where the corresponding JavaScript API is outside Surgeist's native Rust contract.

---

<!-- captured-body-start -->

<a id="head"></a>

[![WHATWG](https://resources.whatwg.org/logo.svg)](https://whatwg.org/)

# HTML

<a id="living-standard"></a>Living Standard — Last Updated 9 October 2026

### <a id="disabled-elements"></a>4.15 Disabled elements[](#disabled-elements)

An element is said to be <a id="concept-element-disabled"></a><strong>actually disabled</strong> if it is one of the following:

- a <a id="disabled-elements&#58;the-button-element"></a><code>[button](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element)</code> element that is <a id="disabled-elements&#58;concept-fe-disabled"></a>[disabled](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-disabled)
- an <a id="disabled-elements&#58;the-input-element"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> element that is <a id="disabled-elements&#58;concept-fe-disabled-2"></a>[disabled](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-disabled)
- a <a id="disabled-elements&#58;the-select-element"></a><code>[select](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)</code> element that is <a id="disabled-elements&#58;concept-fe-disabled-3"></a>[disabled](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-disabled)
- a <a id="disabled-elements&#58;the-textarea-element"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> element that is <a id="disabled-elements&#58;concept-fe-disabled-4"></a>[disabled](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-disabled)
- an <a id="disabled-elements&#58;the-optgroup-element"></a><code>[optgroup](https://html.spec.whatwg.org/multipage/form-elements.html#the-optgroup-element)</code> element that has a <a id="disabled-elements&#58;attr-optgroup-disabled"></a><code>[disabled](https://html.spec.whatwg.org/multipage/form-elements.html#attr-optgroup-disabled)</code> attribute or whose <a id="disabled-elements&#58;option-element-nearest-ancestor-select"></a>[nearest ancestor <code>select</code>](https://html.spec.whatwg.org/multipage/form-elements.html#option-element-nearest-ancestor-select) is <a id="disabled-elements&#58;concept-fe-disabled-5"></a>[disabled](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-disabled)
- an <a id="disabled-elements&#58;the-option-element"></a><code>[option](https://html.spec.whatwg.org/multipage/form-elements.html#the-option-element)</code> element that is <a id="disabled-elements&#58;concept-option-disabled"></a>[disabled](https://html.spec.whatwg.org/multipage/form-elements.html#concept-option-disabled) or whose <a id="disabled-elements&#58;option-element-nearest-ancestor-select-2"></a>[nearest ancestor <code>select</code>](https://html.spec.whatwg.org/multipage/form-elements.html#option-element-nearest-ancestor-select) is <a id="disabled-elements&#58;concept-fe-disabled-6"></a>[disabled](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-disabled)
- a <a id="disabled-elements&#58;the-fieldset-element"></a><code>[fieldset](https://html.spec.whatwg.org/multipage/form-elements.html#the-fieldset-element)</code> element that is a <a id="disabled-elements&#58;concept-fieldset-disabled"></a>[disabled fieldset](https://html.spec.whatwg.org/multipage/form-elements.html#concept-fieldset-disabled)
- a <a id="disabled-elements&#58;form-associated-custom-element"></a>[form-associated custom element](https://html.spec.whatwg.org/multipage/custom-elements.html#form-associated-custom-element) that is <a id="disabled-elements&#58;concept-fe-disabled-7"></a>[disabled](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-disabled)

<strong>Note:</strong>

This definition is used to determine what elements are <a id="disabled-elements&#58;focusable"></a>[focusable](https://html.spec.whatwg.org/multipage/interaction.html#focusable) and which elements match the <a id="disabled-elements&#58;selector-enabled"></a><code>[:enabled](#selector-enabled)</code> and <a id="disabled-elements&#58;selector-disabled"></a><code>[:disabled](#selector-disabled)</code> <a id="disabled-elements&#58;pseudo-class"></a>[pseudo classes](https://drafts.csswg.org/selectors/#pseudo-class).

### <a id="selectors"></a>4.16 Matching HTML elements using selectors and CSS[](#selectors)

#### <a id="case-sensitivity-of-the-css-'attr()'-function"></a>4.16.1 Case-sensitivity of the CSS <a id="case-sensitivity-of-the-css-'attr()'-function&#58;'attr()'"></a>['attr()'](https://drafts.csswg.org/css-values-5/#funcdef-attr) function[](#case-sensitivity-of-the-css-'attr()'-function)

CSS Values and Units leaves the case-sensitivity of attribute names for the purpose of the <a id="case-sensitivity-of-the-css-'attr()'-function&#58;'attr()'-2"></a>['attr()'](https://drafts.csswg.org/css-values-5/#funcdef-attr) function to be defined by the host language. [\[CSSVALUES\]](https://html.spec.whatwg.org/multipage/references.html#refsCSSVALUES)

When comparing the attribute name part of a CSS <a id="case-sensitivity-of-the-css-'attr()'-function&#58;'attr()'-3"></a>['attr()'](https://drafts.csswg.org/css-values-5/#funcdef-attr) function to the names of namespace-less attributes on <a id="case-sensitivity-of-the-css-'attr()'-function&#58;html-elements"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in <a id="case-sensitivity-of-the-css-'attr()'-function&#58;html-documents"></a>[HTML documents](https://dom.spec.whatwg.org/#html-document), the name part of the CSS <a id="case-sensitivity-of-the-css-'attr()'-function&#58;'attr()'-4"></a>['attr()'](https://drafts.csswg.org/css-values-5/#funcdef-attr) function must first be <a id="case-sensitivity-of-the-css-'attr()'-function&#58;converted-to-ascii-lowercase"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase). The same function when compared to other attributes must be compared according to its original case. In both cases, to match, the values must be <a id="case-sensitivity-of-the-css-'attr()'-function&#58;identical-to"></a>[identical to](https://infra.spec.whatwg.org/#string-is) each other (and therefore the comparison is case sensitive).

<strong>Note:</strong>

This is the same as comparing the name part of a CSS <a id="case-sensitivity-of-the-css-'attr()'-function&#58;attribute-selector"></a>[attribute selector](https://drafts.csswg.org/selectors/#attribute-selector), specified in the next section.

#### <a id="case-sensitivity-of-selectors"></a>4.16.2 Case-sensitivity of selectors[](#case-sensitivity-of-selectors)

Selectors leaves the case-sensitivity of element names, attribute names, and attribute values to be defined by the host language. [\[SELECTORS\]](https://html.spec.whatwg.org/multipage/references.html#refsSELECTORS)

When comparing a CSS element <a id="case-sensitivity-of-selectors&#58;type-selector"></a>[type selector](https://drafts.csswg.org/selectors/#type-selector) to the names of <a id="case-sensitivity-of-selectors&#58;html-elements"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in <a id="case-sensitivity-of-selectors&#58;html-documents"></a>[HTML documents](https://dom.spec.whatwg.org/#html-document), the CSS element <a id="case-sensitivity-of-selectors&#58;type-selector-2"></a>[type selector](https://drafts.csswg.org/selectors/#type-selector) must first be <a id="case-sensitivity-of-selectors&#58;converted-to-ascii-lowercase"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase). The same selector when compared to other elements must be compared according to its original case. In both cases, to match, the values must be <a id="case-sensitivity-of-selectors&#58;identical-to"></a>[identical to](https://infra.spec.whatwg.org/#string-is) each other (and therefore the comparison is case sensitive).

When comparing the name part of a CSS <a id="case-sensitivity-of-selectors&#58;attribute-selector"></a>[attribute selector](https://drafts.csswg.org/selectors/#attribute-selector) to the names of attributes on <a id="case-sensitivity-of-selectors&#58;html-elements-2"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in <a id="case-sensitivity-of-selectors&#58;html-documents-2"></a>[HTML documents](https://dom.spec.whatwg.org/#html-document), the name part of the CSS <a id="case-sensitivity-of-selectors&#58;attribute-selector-2"></a>[attribute selector](https://drafts.csswg.org/selectors/#attribute-selector) must first be <a id="case-sensitivity-of-selectors&#58;converted-to-ascii-lowercase-2"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase). The same selector when compared to other attributes must be compared according to its original case. In both cases, the comparison is case-sensitive.

<a id="case-sensitivity-of-selectors&#58;attribute-selector-3"></a>[Attribute selectors](https://drafts.csswg.org/selectors/#attribute-selector) on an <a id="case-sensitivity-of-selectors&#58;html-elements-3"></a>[HTML element](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in an <a id="case-sensitivity-of-selectors&#58;html-documents-3"></a>[HTML document](https://dom.spec.whatwg.org/#html-document) must treat the <em>values</em> of attributes with the following names as <a id="case-sensitivity-of-selectors&#58;ascii-case-insensitive"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive):

- <code>accept</code>
- <code>accept-charset</code>
- <code>align</code>
- <code>alink</code>
- <code>axis</code>
- <code>bgcolor</code>
- <code>charset</code>
- <code>checked</code>
- <code>clear</code>
- <code>codetype</code>
- <code>color</code>
- <code>compact</code>
- <code>declare</code>
- <code>defer</code>
- <code>dir</code>
- <code>direction</code>
- <code>disabled</code>
- <code>enctype</code>
- <code>face</code>
- <code>frame</code>
- <code>hreflang</code>
- <code>http-equiv</code>
- <code>lang</code>
- <code>language</code>
- <code>link</code>
- <code>media</code>
- <code>method</code>
- <code>multiple</code>
- <code>nohref</code>
- <code>noresize</code>
- <code>noshade</code>
- <code>nowrap</code>
- <code>readonly</code>
- <code>rel</code>
- <code>rev</code>
- <code>rules</code>
- <code>scope</code>
- <code>scrolling</code>
- <code>selected</code>
- <code>shape</code>
- <code>target</code>
- <code>text</code>
- <code>type</code>
- <code>valign</code>
- <code>valuetype</code>
- <code>vlink</code>

<strong>Example:</strong>

For example, the selector <code>&#91;bgcolor=&#34;&#35;ffffff&#34;&#93;</code> will match any HTML element with a <code>bgcolor</code> attribute with values including <code>&#35;ffffff</code>, <code>&#35;FFFFFF</code> and <code>&#35;fffFFF</code>. This happens even if <code>bgcolor</code> has no effect for a given element (e.g., <a id="case-sensitivity-of-selectors&#58;the-div-element"></a><code>[div](https://html.spec.whatwg.org/multipage/grouping-content.html#the-div-element)</code>).

The selector <code>&#91;type=a s&#93;</code> will match any HTML element with a <code>type</code> attribute whose value is <code>a</code>, but not whose value is <code>A</code>, due to the <code>s</code> flag.

All other attribute values and everything else must be treated as entirely <a id="case-sensitivity-of-selectors&#58;identical-to-2"></a>[identical to](https://infra.spec.whatwg.org/#string-is) each other for the purposes of selector matching. This includes:

- <a id="case-sensitivity-of-selectors&#58;concept-id"></a>[IDs](https://dom.spec.whatwg.org/#concept-id) and <a id="case-sensitivity-of-selectors&#58;concept-class"></a>[classes](https://dom.spec.whatwg.org/#concept-class) in <a id="case-sensitivity-of-selectors&#58;no-quirks-mode"></a>[no-quirks mode](https://dom.spec.whatwg.org/#concept-document-no-quirks) and <a id="case-sensitivity-of-selectors&#58;limited-quirks-mode"></a>[limited-quirks mode](https://dom.spec.whatwg.org/#concept-document-limited-quirks)

- the names of elements not in the <a id="case-sensitivity-of-selectors&#58;html-namespace-2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace)

- the names of <a id="case-sensitivity-of-selectors&#58;html-elements-4"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in <a id="case-sensitivity-of-selectors&#58;xml-documents"></a>[XML documents](https://dom.spec.whatwg.org/#xml-document)

- the names of attributes of elements not in the <a id="case-sensitivity-of-selectors&#58;html-namespace-2-2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace)

- the names of attributes of <a id="case-sensitivity-of-selectors&#58;html-elements-5"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) in <a id="case-sensitivity-of-selectors&#58;xml-documents-2"></a>[XML documents](https://dom.spec.whatwg.org/#xml-document)

- the names of attributes that themselves have namespaces

<strong>Note:</strong>

Selectors defines that ID and class selectors (such as <code>&#35;foo</code> and <code>.bar</code>), when matched against elements in documents that are in <a id="case-sensitivity-of-selectors&#58;quirks-mode"></a>[quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks), will be matched in an <a id="case-sensitivity-of-selectors&#58;ascii-case-insensitive-2"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) manner. However, this does not apply for attribute selectors with "<code>id</code>" or "<code>class</code>" as the name part. The selector <code>&#91;class=&#34;foobar&#34;&#93;</code> will treat its value as case-sensitive even in <a id="case-sensitivity-of-selectors&#58;quirks-mode-2"></a>[quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks).

#### <a id="pseudo-classes"></a>4.16.3 Pseudo-classes[](#pseudo-classes)

MDN

[Pseudo-classes](https://developer.mozilla.org/en-US/docs/Web/CSS/Pseudo-classes)

There are a number of dynamic selectors that can be used with HTML. This section defines when these selectors match HTML elements. [\[SELECTORS\]](https://html.spec.whatwg.org/multipage/references.html#refsSELECTORS) [\[CSSUI\]](https://html.spec.whatwg.org/multipage/references.html#refsCSSUI)

<strong><a id="selector-defined"></a><strong><code>&#58;defined</code></strong></strong>

<strong>✔</strong>MDN

[:defined](https://developer.mozilla.org/en-US/docs/Web/CSS/:defined)

Support in all current engines.

Firefox63+Safari10+Chrome54+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android?

The <a id="pseudo-classes&#58;selector-defined"></a><code>[:defined](#selector-defined)</code> <a id="pseudo-classes&#58;pseudo-class"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element that is <a id="pseudo-classes&#58;concept-element-defined"></a>[defined](https://dom.spec.whatwg.org/#concept-element-defined).

<strong><a id="selector-link"></a><strong><code>&#58;link</code></strong></strong>

<strong>✔</strong>MDN

[:link](https://developer.mozilla.org/en-US/docs/Web/CSS/:link)

Support in all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer3+

------------------------------------------------------------------------

Firefox Android?Safari iOS3.2+Chrome Android?WebView Android1.5+Samsung Internet?Opera Android?

<strong><a id="selector-visited"></a><strong><code>&#58;visited</code></strong></strong>

<strong>✔</strong>MDN

[:visited](https://developer.mozilla.org/en-US/docs/Web/CSS/:visited)

Support in all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer4+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android10.1+

All <a id="pseudo-classes&#58;the-a-element"></a><code>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element)</code> elements that have an <a id="pseudo-classes&#58;attr-hyperlink-href"></a><code>[href](https://html.spec.whatwg.org/multipage/links.html#attr-hyperlink-href)</code> attribute, and all <a id="pseudo-classes&#58;the-area-element"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element)</code> elements that have an <a id="pseudo-classes&#58;attr-hyperlink-href-2"></a><code>[href](https://html.spec.whatwg.org/multipage/links.html#attr-hyperlink-href)</code> attribute, must match one of <a id="pseudo-classes&#58;selector-link"></a><code>[:link](#selector-link)</code> and <a id="pseudo-classes&#58;selector-visited"></a><code>[:visited](#selector-visited)</code>.

Other specifications might apply more specific rules regarding how these elements are to match these <a id="pseudo-classes&#58;pseudo-class-2"></a>[pseudo-classes](https://drafts.csswg.org/selectors/#pseudo-class), to mitigate some privacy concerns that apply with straightforward implementations of this requirement.

<strong><a id="selector-active"></a><strong><code>&#58;active</code></strong></strong>

<strong>✔</strong>MDN

[:active](https://developer.mozilla.org/en-US/docs/Web/CSS/:active)

Support in all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer4+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-active"></a><code>[:active](#selector-active)</code> <a id="pseudo-classes&#58;pseudo-class-3"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) is defined to match an element <q>while an element is <a id="concept-selector-active"></a><strong><em>being activated</em></strong> by the user</q>.

To determine whether a particular element is <a id="pseudo-classes&#58;concept-selector-active"></a><em>[being activated](#concept-selector-active)</em> for the purposes of defining the <a id="pseudo-classes&#58;selector-active-2"></a><code>[:active](#selector-active)</code> <a id="pseudo-classes&#58;pseudo-class-4"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) only, an HTML user agent must use the first relevant entry in the following list.

<strong>If the element is a <a id="pseudo-classes&#58;the-button-element"></a><code>[button](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element)</code> element</strong>

<strong>If the element is an <a id="pseudo-classes&#58;the-input-element"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> element whose <a id="pseudo-classes&#58;attr-input-type"></a><code>[type](https://html.spec.whatwg.org/multipage/input.html#attr-input-type)</code> attribute is in the <a id="pseudo-classes&#58;submit-button-state-(type=submit)"></a>[Submit Button](https://html.spec.whatwg.org/multipage/input.html#submit-button-state-(type=submit)), <a id="pseudo-classes&#58;image-button-state-(type=image)"></a>[Image Button](https://html.spec.whatwg.org/multipage/input.html#image-button-state-(type=image)), <a id="pseudo-classes&#58;reset-button-state-(type=reset)"></a>[Reset Button](https://html.spec.whatwg.org/multipage/input.html#reset-button-state-(type=reset)), or <a id="pseudo-classes&#58;button-state-(type=button)"></a>[Button](https://html.spec.whatwg.org/multipage/input.html#button-state-(type=button)) state</strong>

<strong>If the element is an <a id="pseudo-classes&#58;the-a-element-2"></a><code>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element)</code> element that has an <a id="pseudo-classes&#58;attr-hyperlink-href-3"></a><code>[href](https://html.spec.whatwg.org/multipage/links.html#attr-hyperlink-href)</code> attribute</strong>

<strong>If the element is an <a id="pseudo-classes&#58;the-area-element-2"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element)</code> element that has an <a id="pseudo-classes&#58;attr-hyperlink-href-4"></a><code>[href](https://html.spec.whatwg.org/multipage/links.html#attr-hyperlink-href)</code> attribute</strong>

<strong>If the element is <a id="pseudo-classes&#58;focusable"></a>[focusable](https://html.spec.whatwg.org/multipage/interaction.html#focusable)</strong>

The element is <a id="pseudo-classes&#58;concept-selector-active-2"></a><em>[being activated](#concept-selector-active)</em> if it is <a id="pseudo-classes&#58;in-a-formal-activation-state"></a>[in a formal activation state](#in-a-formal-activation-state).

<strong>Example:</strong>

For example, if the user is using a keyboard to push a <a id="pseudo-classes&#58;the-button-element-2"></a><code>[button](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element)</code> element by pressing the space bar, the element would match this <a id="pseudo-classes&#58;pseudo-class-5"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) in between the time that the element received the <a id="pseudo-classes&#58;event-keydown"></a><code>[keydown](https://w3c.github.io/uievents/#event-type-keydown)</code> event and the time the element received the <a id="pseudo-classes&#58;event-keyup"></a><code>[keyup](https://w3c.github.io/uievents/#event-type-keyup)</code> event.

<strong>If the element is <a id="pseudo-classes&#58;being-actively-pointed-at"></a>[being actively pointed at](#being-actively-pointed-at)</strong>

The element is <a id="pseudo-classes&#58;concept-selector-active-3"></a><em>[being activated](#concept-selector-active)</em>.

An element is said to be <a id="in-a-formal-activation-state"></a><strong>in a formal activation state</strong> between the time the user begins to indicate an intent to trigger the element's <a id="pseudo-classes&#58;activation-behaviour"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior) and either the time the user stops indicating an intent to trigger the element's <a id="pseudo-classes&#58;activation-behaviour-2"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior), or the time the element's <a id="pseudo-classes&#58;activation-behaviour-3"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior) has finished running, which ever comes first.

An element is said to be <a id="being-actively-pointed-at"></a><strong>being actively pointed at</strong> while the user indicates the element using a pointing device while that pointing device is in the "down" state (e.g. for a mouse, between the time the mouse button is pressed and the time it is released; for a finger in a multitouch environment, while the finger is touching the display surface).

<strong>Note:</strong>

Per the definition in Selectors, <a id="pseudo-classes&#58;selector-active-3"></a><code>[:active](#selector-active)</code> also matches <a id="pseudo-classes&#58;flat-tree"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree) ancestors of elements that are <a id="pseudo-classes&#58;concept-selector-active-4"></a><em>[being activated](#concept-selector-active)</em>. [\[SELECTORS\]](https://html.spec.whatwg.org/multipage/references.html#refsSELECTORS)

Additionally, any element that is the <a id="pseudo-classes&#58;labeled-control"></a>[labeled control](https://html.spec.whatwg.org/multipage/forms.html#labeled-control) of a <a id="pseudo-classes&#58;the-label-element"></a><code>[label](https://html.spec.whatwg.org/multipage/forms.html#the-label-element)</code> element that is currently matching <a id="pseudo-classes&#58;selector-active-4"></a><code>[:active](#selector-active)</code>, also matches <a id="pseudo-classes&#58;selector-active-5"></a><code>[:active](#selector-active)</code>. (But, it does not count as being <a id="pseudo-classes&#58;concept-selector-active-5"></a><em>[being activated](#concept-selector-active)</em>.)

<strong><a id="selector-hover"></a><strong><code>&#58;hover</code></strong></strong>

<strong>✔</strong>MDN

[:hover](https://developer.mozilla.org/en-US/docs/Web/CSS/:hover)

Support in all current engines.

Firefox1+Safari2+Chrome1+

------------------------------------------------------------------------

Opera4+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer4+

------------------------------------------------------------------------

Firefox Android?Safari iOS1+Chrome Android?WebView Android?Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-hover"></a><code>[:hover](#selector-hover)</code> <a id="pseudo-classes&#58;pseudo-class-6"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) is defined to match an element <q>while the user <a id="concept-selector-hover"></a><strong><em>designates</em></strong> an element with a pointing device</q>. For the purposes of defining the <a id="pseudo-classes&#58;selector-hover-2"></a><code>[:hover](#selector-hover)</code> <a id="pseudo-classes&#58;pseudo-class-7"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) only, an HTML user agent must consider an element as being one that the user <a id="pseudo-classes&#58;concept-selector-hover"></a><em>[designates](#concept-selector-hover)</em> if it is an element that the user indicates using a pointing device.

<strong>Note:</strong>

Per the definition in Selectors, <a id="pseudo-classes&#58;selector-hover-3"></a><code>[:hover](#selector-hover)</code> also matches <a id="pseudo-classes&#58;flat-tree-2"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree) ancestors of elements that are <a id="pseudo-classes&#58;concept-selector-hover-2"></a><em>[designated](#concept-selector-hover)</em>. [\[SELECTORS\]](https://html.spec.whatwg.org/multipage/references.html#refsSELECTORS)

Additionally, any element that is the <a id="pseudo-classes&#58;labeled-control-2"></a>[labeled control](https://html.spec.whatwg.org/multipage/forms.html#labeled-control) of a <a id="pseudo-classes&#58;the-label-element-2"></a><code>[label](https://html.spec.whatwg.org/multipage/forms.html#the-label-element)</code> element that is currently matching <a id="pseudo-classes&#58;selector-hover-4"></a><code>[:hover](#selector-hover)</code>, also matches <a id="pseudo-classes&#58;selector-hover-5"></a><code>[:hover](#selector-hover)</code>. (But, it does not count as being <a id="pseudo-classes&#58;concept-selector-hover-3"></a><em>[designated](#concept-selector-hover)</em>.)

<strong>Example:</strong>

Consider in particular a fragment such as:

``` text
<p> <label for=c> <input id=a> </label> <span id=b> <input id=c> </span> </p>
```

If the user designates the element with ID "<code>a</code>" with their pointing device, then the <a id="pseudo-classes&#58;the-p-element"></a><code>[p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element)</code> element (and all its ancestors not shown in the snippet above), the <a id="pseudo-classes&#58;the-label-element-3"></a><code>[label](https://html.spec.whatwg.org/multipage/forms.html#the-label-element)</code> element, the element with ID "<code>a</code>", and the element with ID "<code>c</code>" will match the <a id="pseudo-classes&#58;selector-hover-6"></a><code>[:hover](#selector-hover)</code> <a id="pseudo-classes&#58;pseudo-class-8"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class). The element with ID "<code>a</code>" matches it by being <a id="pseudo-classes&#58;concept-selector-hover-4"></a><em>[designated](#concept-selector-hover)</em>; the <a id="pseudo-classes&#58;the-label-element-4"></a><code>[label](https://html.spec.whatwg.org/multipage/forms.html#the-label-element)</code> and <a id="pseudo-classes&#58;the-p-element-2"></a><code>[p](https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element)</code> elements match it because of the condition in Selectors about flat tree ancestors; and the element with ID "<code>c</code>" matches it through the additional condition above on <a id="pseudo-classes&#58;labeled-control-3"></a>[labeled controls](https://html.spec.whatwg.org/multipage/forms.html#labeled-control) (i.e., its <a id="pseudo-classes&#58;the-label-element-5"></a><code>[label](https://html.spec.whatwg.org/multipage/forms.html#the-label-element)</code> element matches <a id="pseudo-classes&#58;selector-hover-7"></a><code>[:hover](#selector-hover)</code>). However, the element with ID "<code>b</code>" does <em>not</em> match <a id="pseudo-classes&#58;selector-hover-8"></a><code>[:hover](#selector-hover)</code>: its flat tree descendant is not designated, even though that flat tree descendant matches <a id="pseudo-classes&#58;selector-hover-9"></a><code>[:hover](#selector-hover)</code>.

<strong><a id="selector-focus"></a><strong><code>&#58;focus</code></strong></strong>

<strong>✔</strong>MDN

[:focus](https://developer.mozilla.org/en-US/docs/Web/CSS/:focus)

Support in all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera7+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer8+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android10.1+

For the purposes of the CSS <a id="pseudo-classes&#58;selector-focus"></a><code>[:focus](#selector-focus)</code> <a id="pseudo-classes&#58;pseudo-class-9"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class), an <a id="element-has-the-focus"></a><strong>element has the focus</strong> when:

- it is not itself a <a id="pseudo-classes&#58;navigable-container"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container); and

- any of the following are true:

  - it is one of the elements listed in the <a id="pseudo-classes&#58;current-focus-chain-of-a-top-level-traversable"></a>[current focus chain of the top-level traversable](https://html.spec.whatwg.org/multipage/interaction.html#current-focus-chain-of-a-top-level-traversable); or

  - its <a id="pseudo-classes&#58;concept-element-shadow-root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root) <var>shadowRoot</var> is not null and <var>shadowRoot</var> is the <a id="pseudo-classes&#58;root"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) of at least one element that <a id="pseudo-classes&#58;element-has-the-focus"></a>[has the focus](#element-has-the-focus).

<strong><a id="selector-target"></a><strong><code>&#58;target</code></strong></strong>

<strong>✔</strong>MDN

[:target](https://developer.mozilla.org/en-US/docs/Web/CSS/:target)

Support in all current engines.

Firefox1+Safari1.3+Chrome1+

------------------------------------------------------------------------

Opera9.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer9+

------------------------------------------------------------------------

Firefox Android?Safari iOS2+Chrome Android?WebView Android2+Samsung Internet?Opera Android10.1+

For the purposes of the CSS <a id="pseudo-classes&#58;selector-target"></a><code>[:target](#selector-target)</code> <a id="pseudo-classes&#58;pseudo-class-10"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class), the <a id="pseudo-classes&#58;document"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>'s <em>target elements</em> are a list containing the <a id="pseudo-classes&#58;document-2"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>'s <a id="pseudo-classes&#58;target-element"></a>[target element](https://html.spec.whatwg.org/multipage/browsing-the-web.html#target-element), if it is not null, or containing no elements, if it is. [\[SELECTORS\]](https://html.spec.whatwg.org/multipage/references.html#refsSELECTORS)

<strong><a id="selector-popover-open"></a><strong><code>&#58;popover-open</code></strong></strong>

<strong>✔</strong>MDN

[:popover-open](https://developer.mozilla.org/en-US/docs/Web/CSS/:popover-open)

Support in all current engines.

Firefox🔰 114+Safaripreview+Chrome114+

------------------------------------------------------------------------

Opera?Edge114+

------------------------------------------------------------------------

Edge (Legacy)?Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android?

The <a id="pseudo-classes&#58;selector-popover-open"></a><code>[:popover-open](#selector-popover-open)</code> <a id="pseudo-classes&#58;pseudo-class-11"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) is defined to match any <a id="pseudo-classes&#58;html-elements"></a>[HTML element](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) whose <a id="pseudo-classes&#58;attr-popover"></a><code>[popover](https://html.spec.whatwg.org/multipage/popover.html#attr-popover)</code> attribute is not in the <a id="pseudo-classes&#58;attr-popover-none-state"></a>[No Popover](https://html.spec.whatwg.org/multipage/popover.html#attr-popover-none-state) state and whose <a id="pseudo-classes&#58;popover-visibility-state"></a>[popover visibility state](https://html.spec.whatwg.org/multipage/popover.html#popover-visibility-state) is <a id="pseudo-classes&#58;popover-showing-state"></a>[showing](https://html.spec.whatwg.org/multipage/popover.html#popover-showing-state).

<strong><a id="selector-enabled"></a><strong><code>&#58;enabled</code></strong></strong>

<strong>✔</strong>MDN

[:enabled](https://developer.mozilla.org/en-US/docs/Web/CSS/:enabled)

Support in all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer9+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android2+Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-enabled"></a><code>[:enabled](#selector-enabled)</code> <a id="pseudo-classes&#58;pseudo-class-12"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any <a id="pseudo-classes&#58;the-button-element-3"></a><code>[button](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element)</code>, <a id="pseudo-classes&#58;the-input-element-2"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code>, <a id="pseudo-classes&#58;the-select-element"></a><code>[select](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)</code>, <a id="pseudo-classes&#58;the-textarea-element"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code>, <a id="pseudo-classes&#58;the-optgroup-element"></a><code>[optgroup](https://html.spec.whatwg.org/multipage/form-elements.html#the-optgroup-element)</code>, <a id="pseudo-classes&#58;the-option-element"></a><code>[option](https://html.spec.whatwg.org/multipage/form-elements.html#the-option-element)</code>, <a id="pseudo-classes&#58;the-fieldset-element"></a><code>[fieldset](https://html.spec.whatwg.org/multipage/form-elements.html#the-fieldset-element)</code> element, or <a id="pseudo-classes&#58;form-associated-custom-element"></a>[form-associated custom element](https://html.spec.whatwg.org/multipage/custom-elements.html#form-associated-custom-element) that is not <a id="pseudo-classes&#58;concept-element-disabled"></a>[actually disabled](#concept-element-disabled).

<strong><a id="selector-disabled"></a><strong><code>&#58;disabled</code></strong></strong>

<strong>✔</strong>MDN

[:disabled](https://developer.mozilla.org/en-US/docs/Web/CSS/:disabled)

Support in all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer9+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android2+Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-disabled"></a><code>[:disabled](#selector-disabled)</code> <a id="pseudo-classes&#58;pseudo-class-13"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element that is <a id="pseudo-classes&#58;concept-element-disabled-2"></a>[actually disabled](#concept-element-disabled).

<strong><a id="selector-checked"></a><strong><code>&#58;checked</code></strong></strong>

<strong>✔</strong>MDN

[:checked](https://developer.mozilla.org/en-US/docs/Web/CSS/:checked)

Support in all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer9+

------------------------------------------------------------------------

Firefox Android4+Safari iOS?Chrome Android18+WebView Android2+Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-checked"></a><code>[:checked](#selector-checked)</code> <a id="pseudo-classes&#58;pseudo-class-14"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories:

- <a id="pseudo-classes&#58;the-input-element-3"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements whose <a id="pseudo-classes&#58;attr-input-type-2"></a><code>[type](https://html.spec.whatwg.org/multipage/input.html#attr-input-type)</code> attribute is in the <a id="pseudo-classes&#58;checkbox-state-(type=checkbox)"></a>[Checkbox](https://html.spec.whatwg.org/multipage/input.html#checkbox-state-(type=checkbox)) state and whose <a id="pseudo-classes&#58;concept-fe-checked"></a>[checkedness](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-checked) state is true
- <a id="pseudo-classes&#58;the-input-element-4"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements whose <a id="pseudo-classes&#58;attr-input-type-3"></a><code>[type](https://html.spec.whatwg.org/multipage/input.html#attr-input-type)</code> attribute is in the <a id="pseudo-classes&#58;radio-button-state-(type=radio)"></a>[Radio Button](https://html.spec.whatwg.org/multipage/input.html#radio-button-state-(type=radio)) state and whose <a id="pseudo-classes&#58;concept-fe-checked-2"></a>[checkedness](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-checked) state is true
- <a id="pseudo-classes&#58;the-option-element-2"></a><code>[option](https://html.spec.whatwg.org/multipage/form-elements.html#the-option-element)</code> elements whose <a id="pseudo-classes&#58;concept-option-selectedness"></a>[selectedness](https://html.spec.whatwg.org/multipage/form-elements.html#concept-option-selectedness) is true

<strong><a id="selector-indeterminate"></a><strong><code>&#58;indeterminate</code></strong></strong>

<strong>✔</strong>MDN

[:indeterminate](https://developer.mozilla.org/en-US/docs/Web/CSS/:indeterminate)

Support in all current engines.

Firefox2+Safari3+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer10+

------------------------------------------------------------------------

Firefox Android?Safari iOS1+Chrome Android?WebView Android37+Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-indeterminate"></a><code>[:indeterminate](#selector-indeterminate)</code> <a id="pseudo-classes&#58;pseudo-class-15"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories:

- <a id="pseudo-classes&#58;the-input-element-5"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements whose <a id="pseudo-classes&#58;attr-input-type-4"></a><code>[type](https://html.spec.whatwg.org/multipage/input.html#attr-input-type)</code> attribute is in the <a id="pseudo-classes&#58;checkbox-state-(type=checkbox)-2"></a>[Checkbox](https://html.spec.whatwg.org/multipage/input.html#checkbox-state-(type=checkbox)) state and whose <a id="pseudo-classes&#58;concept-input-indeterminate"></a>[indeterminateness](https://html.spec.whatwg.org/multipage/input.html#concept-input-indeterminate) is true
- <a id="pseudo-classes&#58;the-input-element-6"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements whose <a id="pseudo-classes&#58;attr-input-type-5"></a><code>[type](https://html.spec.whatwg.org/multipage/input.html#attr-input-type)</code> attribute is in the <a id="pseudo-classes&#58;radio-button-state-(type=radio)-2"></a>[Radio Button](https://html.spec.whatwg.org/multipage/input.html#radio-button-state-(type=radio)) state and whose <a id="pseudo-classes&#58;radio-button-group"></a>[radio button group](https://html.spec.whatwg.org/multipage/input.html#radio-button-group) contains no <a id="pseudo-classes&#58;the-input-element-7"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements whose <a id="pseudo-classes&#58;concept-fe-checked-3"></a>[checkedness](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-checked) state is true.
- <a id="pseudo-classes&#58;the-progress-element"></a><code>[progress](https://html.spec.whatwg.org/multipage/form-elements.html#the-progress-element)</code> elements with no <a id="pseudo-classes&#58;attr-progress-value"></a><code>[value](https://html.spec.whatwg.org/multipage/form-elements.html#attr-progress-value)</code> content attribute

<strong><a id="selector-default"></a><strong><code>&#58;default</code></strong></strong>

<strong>✔</strong>MDN

[:default](https://developer.mozilla.org/en-US/docs/Web/CSS/:default)

Support in all current engines.

Firefox4+Safari5+Chrome10+

------------------------------------------------------------------------

Opera10+Edge79+

------------------------------------------------------------------------

Edge (Legacy)?Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android?Safari iOS5+Chrome Android?WebView Android37+Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-default"></a><code>[:default](#selector-default)</code> <a id="pseudo-classes&#58;pseudo-class-16"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories:

- <a id="pseudo-classes&#58;concept-submit-button"></a>[Submit buttons](https://html.spec.whatwg.org/multipage/forms.html#concept-submit-button) that are <a id="pseudo-classes&#58;default-button"></a>[default buttons](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#default-button) of their <a id="pseudo-classes&#58;form-owner"></a>[form owner](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-owner).
- <a id="pseudo-classes&#58;the-input-element-8"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements to which the <a id="pseudo-classes&#58;attr-input-checked"></a><code>[checked](https://html.spec.whatwg.org/multipage/input.html#attr-input-checked)</code> attribute applies and that have a <a id="pseudo-classes&#58;attr-input-checked-2"></a><code>[checked](https://html.spec.whatwg.org/multipage/input.html#attr-input-checked)</code> attribute
- <a id="pseudo-classes&#58;the-option-element-3"></a><code>[option](https://html.spec.whatwg.org/multipage/form-elements.html#the-option-element)</code> elements that have a <a id="pseudo-classes&#58;attr-option-selected"></a><code>[selected](https://html.spec.whatwg.org/multipage/form-elements.html#attr-option-selected)</code> attribute

<strong><a id="selector-placeholder-shown"></a><strong><code>&#58;placeholder-shown</code></strong></strong>

The <a id="pseudo-classes&#58;selector-placeholder-shown"></a><code>[:placeholder-shown](#selector-placeholder-shown)</code> <a id="pseudo-classes&#58;pseudo-class-17"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories:

- <a id="pseudo-classes&#58;the-input-element-9"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements that have a <a id="pseudo-classes&#58;attr-input-placeholder"></a><code>[placeholder](https://html.spec.whatwg.org/multipage/input.html#attr-input-placeholder)</code> attribute whose value is currently being presented to the user
- <a id="pseudo-classes&#58;the-textarea-element-2"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> elements that have a <a id="pseudo-classes&#58;attr-textarea-placeholder"></a><code>[placeholder](https://html.spec.whatwg.org/multipage/form-elements.html#attr-textarea-placeholder)</code> attribute whose value is currently being presented to the user

<strong><a id="selector-valid"></a><strong><code>&#58;valid</code></strong></strong>

<strong>✔</strong>MDN

[:valid](https://developer.mozilla.org/en-US/docs/Web/CSS/:valid)

Support in all current engines.

Firefox4+Safari5+Chrome10+

------------------------------------------------------------------------

Opera10+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer10+

------------------------------------------------------------------------

Firefox Android?Safari iOS5+Chrome Android?WebView Android37+Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-valid"></a><code>[:valid](#selector-valid)</code> <a id="pseudo-classes&#58;pseudo-class-18"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories:

- elements that are <a id="pseudo-classes&#58;candidate-for-constraint-validation"></a>[candidates for constraint validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#candidate-for-constraint-validation) and that <a id="pseudo-classes&#58;concept-fv-valid"></a>[satisfy their constraints](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fv-valid)
- <a id="pseudo-classes&#58;the-form-element"></a><code>[form](https://html.spec.whatwg.org/multipage/forms.html#the-form-element)</code> elements that are not the <a id="pseudo-classes&#58;form-owner-2"></a>[form owner](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-owner) of any elements that themselves are <a id="pseudo-classes&#58;candidate-for-constraint-validation-2"></a>[candidates for constraint validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#candidate-for-constraint-validation) but do not <a id="pseudo-classes&#58;concept-fv-valid-2"></a>[satisfy their constraints](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fv-valid)
- <a id="pseudo-classes&#58;the-fieldset-element-2"></a><code>[fieldset](https://html.spec.whatwg.org/multipage/form-elements.html#the-fieldset-element)</code> elements that have no descendant elements that themselves are <a id="pseudo-classes&#58;candidate-for-constraint-validation-3"></a>[candidates for constraint validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#candidate-for-constraint-validation) but do not <a id="pseudo-classes&#58;concept-fv-valid-3"></a>[satisfy their constraints](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fv-valid)

<strong><a id="selector-invalid"></a><strong><code>&#58;invalid</code></strong></strong>

<strong>✔</strong>MDN

[:invalid](https://developer.mozilla.org/en-US/docs/Web/CSS/:invalid)

Support in all current engines.

Firefox4+Safari5+Chrome10+

------------------------------------------------------------------------

Opera10+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer10+

------------------------------------------------------------------------

Firefox Android?Safari iOS5+Chrome Android?WebView Android37+Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-invalid"></a><code>[:invalid](#selector-invalid)</code> <a id="pseudo-classes&#58;pseudo-class-19"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories:

- elements that are <a id="pseudo-classes&#58;candidate-for-constraint-validation-4"></a>[candidates for constraint validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#candidate-for-constraint-validation) but that do not <a id="pseudo-classes&#58;concept-fv-valid-4"></a>[satisfy their constraints](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fv-valid)
- <a id="pseudo-classes&#58;the-form-element-2"></a><code>[form](https://html.spec.whatwg.org/multipage/forms.html#the-form-element)</code> elements that are the <a id="pseudo-classes&#58;form-owner-3"></a>[form owner](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-owner) of one or more elements that themselves are <a id="pseudo-classes&#58;candidate-for-constraint-validation-5"></a>[candidates for constraint validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#candidate-for-constraint-validation) but do not <a id="pseudo-classes&#58;concept-fv-valid-5"></a>[satisfy their constraints](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fv-valid)
- <a id="pseudo-classes&#58;the-fieldset-element-3"></a><code>[fieldset](https://html.spec.whatwg.org/multipage/form-elements.html#the-fieldset-element)</code> elements that have of one or more descendant elements that themselves are <a id="pseudo-classes&#58;candidate-for-constraint-validation-6"></a>[candidates for constraint validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#candidate-for-constraint-validation) but do not <a id="pseudo-classes&#58;concept-fv-valid-6"></a>[satisfy their constraints](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fv-valid)

<strong><a id="selector-user-valid"></a><strong><code>&#58;user-valid</code></strong></strong>

The <a id="pseudo-classes&#58;selector-user-valid"></a><code>[:user-valid](#selector-user-valid)</code> <a id="pseudo-classes&#58;pseudo-class-20"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match <a id="pseudo-classes&#58;the-input-element-10"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code>, <a id="pseudo-classes&#58;the-textarea-element-3"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code>, and <a id="pseudo-classes&#58;the-select-element-2"></a><code>[select](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)</code> elements whose <a id="pseudo-classes&#58;user-validity"></a>[user validity](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#user-validity) is true, are <a id="pseudo-classes&#58;candidate-for-constraint-validation-7"></a>[candidates for constraint validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#candidate-for-constraint-validation), and that <a id="pseudo-classes&#58;concept-fv-valid-7"></a>[satisfy their constraints](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fv-valid).

<strong><a id="selector-user-invalid"></a><strong><code>&#58;user-invalid</code></strong></strong>

The <a id="pseudo-classes&#58;selector-user-invalid"></a><code>[:user-invalid](#selector-user-invalid)</code> <a id="pseudo-classes&#58;pseudo-class-21"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match <a id="pseudo-classes&#58;the-input-element-11"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code>, <a id="pseudo-classes&#58;the-textarea-element-4"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code>, and <a id="pseudo-classes&#58;the-select-element-3"></a><code>[select](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)</code> elements whose <a id="pseudo-classes&#58;user-validity-2"></a>[user validity](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#user-validity) is true, are <a id="pseudo-classes&#58;candidate-for-constraint-validation-8"></a>[candidates for constraint validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#candidate-for-constraint-validation) but do not <a id="pseudo-classes&#58;concept-fv-valid-8"></a>[satisfy their constraints](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fv-valid).

<strong><a id="selector-in-range"></a><strong><code>&#58;in-range</code></strong></strong>

<strong>✔</strong>MDN

[:in-range](https://developer.mozilla.org/en-US/docs/Web/CSS/:in-range)

Support in all current engines.

Firefox29+Safari5.1+Chrome10+

------------------------------------------------------------------------

Opera11+Edge79+

------------------------------------------------------------------------

Edge (Legacy)13+Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android16+Safari iOS?Chrome Android?WebView Android2.2+Samsung Internet1.0+Opera Android11+

The <a id="pseudo-classes&#58;selector-in-range"></a><code>[:in-range](#selector-in-range)</code> <a id="pseudo-classes&#58;pseudo-class-22"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all elements that are <a id="pseudo-classes&#58;candidate-for-constraint-validation-9"></a>[candidates for constraint validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#candidate-for-constraint-validation), <a id="pseudo-classes&#58;have-range-limitations"></a>[have range limitations](https://html.spec.whatwg.org/multipage/input.html#have-range-limitations), and that are neither <a id="pseudo-classes&#58;suffering-from-an-underflow"></a>[suffering from an underflow](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#suffering-from-an-underflow) nor <a id="pseudo-classes&#58;suffering-from-an-overflow"></a>[suffering from an overflow](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#suffering-from-an-overflow).

<strong><a id="selector-out-of-range"></a><strong><code>&#58;out-of-range</code></strong></strong>

<strong>✔</strong>MDN

[:out-of-range](https://developer.mozilla.org/en-US/docs/Web/CSS/:out-of-range)

Support in all current engines.

Firefox29+Safari5.1+Chrome10+

------------------------------------------------------------------------

Opera11+Edge79+

------------------------------------------------------------------------

Edge (Legacy)13+Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android16+Safari iOS?Chrome Android?WebView Android2.2+Samsung Internet?Opera Android11+

The <a id="pseudo-classes&#58;selector-out-of-range"></a><code>[:out-of-range](#selector-out-of-range)</code> <a id="pseudo-classes&#58;pseudo-class-23"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all elements that are <a id="pseudo-classes&#58;candidate-for-constraint-validation-10"></a>[candidates for constraint validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#candidate-for-constraint-validation), <a id="pseudo-classes&#58;have-range-limitations-2"></a>[have range limitations](https://html.spec.whatwg.org/multipage/input.html#have-range-limitations), and that are either <a id="pseudo-classes&#58;suffering-from-an-underflow-2"></a>[suffering from an underflow](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#suffering-from-an-underflow) or <a id="pseudo-classes&#58;suffering-from-an-overflow-2"></a>[suffering from an overflow](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#suffering-from-an-overflow).

<strong><a id="selector-required"></a><strong><code>&#58;required</code></strong></strong>

<strong>✔</strong>MDN

[:required](https://developer.mozilla.org/en-US/docs/Web/CSS/:required)

Support in all current engines.

Firefox4+Safari5+Chrome10+

------------------------------------------------------------------------

Opera10+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer10+

------------------------------------------------------------------------

Firefox Android?Safari iOS5+Chrome Android?WebView Android4.4.3+Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-required"></a><code>[:required](#selector-required)</code> <a id="pseudo-classes&#58;pseudo-class-24"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories:

- <a id="pseudo-classes&#58;the-input-element-12"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements that are <a id="pseudo-classes&#58;concept-input-required"></a><em>[required](https://html.spec.whatwg.org/multipage/input.html#concept-input-required)</em>
- <a id="pseudo-classes&#58;the-select-element-4"></a><code>[select](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)</code> elements that have a <a id="pseudo-classes&#58;attr-select-required"></a><code>[required](https://html.spec.whatwg.org/multipage/form-elements.html#attr-select-required)</code> attribute
- <a id="pseudo-classes&#58;the-textarea-element-5"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> elements that have a <a id="pseudo-classes&#58;attr-textarea-required"></a><code>[required](https://html.spec.whatwg.org/multipage/form-elements.html#attr-textarea-required)</code> attribute

<strong><a id="selector-optional"></a><strong><code>&#58;optional</code></strong></strong>

<strong>✔</strong>MDN

[:optional](https://developer.mozilla.org/en-US/docs/Web/CSS/:optional)

Support in all current engines.

Firefox4+Safari5+Chrome10+

------------------------------------------------------------------------

Opera10+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer10+

------------------------------------------------------------------------

Firefox Android?Safari iOS5+Chrome Android?WebView Android37+Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-optional"></a><code>[:optional](#selector-optional)</code> <a id="pseudo-classes&#58;pseudo-class-25"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories:

- <a id="pseudo-classes&#58;the-input-element-13"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements to which the <a id="pseudo-classes&#58;attr-input-required"></a><code>[required](https://html.spec.whatwg.org/multipage/input.html#attr-input-required)</code> attribute applies that are not <a id="pseudo-classes&#58;concept-input-required-2"></a><em>[required](https://html.spec.whatwg.org/multipage/input.html#concept-input-required)</em>
- <a id="pseudo-classes&#58;the-select-element-5"></a><code>[select](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)</code> elements that do not have a <a id="pseudo-classes&#58;attr-select-required-2"></a><code>[required](https://html.spec.whatwg.org/multipage/form-elements.html#attr-select-required)</code> attribute
- <a id="pseudo-classes&#58;the-textarea-element-6"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> elements that do not have a <a id="pseudo-classes&#58;attr-textarea-required-2"></a><code>[required](https://html.spec.whatwg.org/multipage/form-elements.html#attr-textarea-required)</code> attribute

<strong><a id="selector-autofill"></a><strong><code>&#58;autofill</code></strong></strong>

MDN

[:autofill](https://developer.mozilla.org/en-US/docs/Web/CSS/:autofill)

Firefox86+Safari15+Chrome🔰 1+

------------------------------------------------------------------------

Opera?Edge🔰 79+

------------------------------------------------------------------------

Edge (Legacy)?Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android?Safari iOS15+Chrome Android?WebView Android?Samsung Internet?Opera Android?

<strong><a id="selector-webkit-autofill"></a><strong><code>&#58;-webkit-autofill</code></strong></strong>

The <a id="pseudo-classes&#58;selector-autofill"></a><code>[:autofill](#selector-autofill)</code> and <a id="pseudo-classes&#58;selector-webkit-autofill"></a><code>[:-webkit-autofill](#selector-webkit-autofill)</code> <a id="pseudo-classes&#58;pseudo-class-26"></a>[pseudo-classes](https://drafts.csswg.org/selectors/#pseudo-class) must match <a id="pseudo-classes&#58;the-input-element-14"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements which have been autofilled by user agent. These pseudo-classes must stop matching if the user edits the autofilled field.

<strong>Note:</strong>

One way such autofilling might happen is via the <a id="pseudo-classes&#58;attr-fe-autocomplete"></a><code>[autocomplete](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#attr-fe-autocomplete)</code> attribute, but user agents could autofill even without that attribute being involved.

<strong><a id="selector-read-only"></a><strong><code>&#58;read-only</code></strong></strong>

<strong>✔</strong>MDN

[:read-only](https://developer.mozilla.org/en-US/docs/Web/CSS/:read-only)

Support in all current engines.

Firefox78+Safari4+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)13+Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android🔰 4+Safari iOS?Chrome Android?WebView Android37+Samsung Internet?Opera Android10.1+

<strong><a id="selector-read-write"></a><strong><code>&#58;read-write</code></strong></strong>

<strong>✔</strong>MDN

[:read-write](https://developer.mozilla.org/en-US/docs/Web/CSS/:read-write)

Support in all current engines.

Firefox78+Safari4+Chrome1+

------------------------------------------------------------------------

Opera9+Edge79+

------------------------------------------------------------------------

Edge (Legacy)13+Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android🔰 4+Safari iOS?Chrome Android?WebView Android37+Samsung Internet?Opera Android10.1+

The <a id="pseudo-classes&#58;selector-read-write"></a><code>[:read-write](#selector-read-write)</code> <a id="pseudo-classes&#58;pseudo-class-27"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories, which for the purposes of Selectors are thus considered <em>user-alterable</em>: [\[SELECTORS\]](https://html.spec.whatwg.org/multipage/references.html#refsSELECTORS)

- <a id="pseudo-classes&#58;the-input-element-15"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements to which the <a id="pseudo-classes&#58;attr-input-readonly"></a><code>[readonly](https://html.spec.whatwg.org/multipage/input.html#attr-input-readonly)</code> attribute applies, and that are <a id="pseudo-classes&#58;concept-fe-mutable"></a>[mutable](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-mutable) (i.e. that do not have the <a id="pseudo-classes&#58;attr-input-readonly-2"></a><code>[readonly](https://html.spec.whatwg.org/multipage/input.html#attr-input-readonly)</code> attribute specified and that are not <a id="pseudo-classes&#58;concept-fe-disabled"></a>[disabled](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-disabled))
- <a id="pseudo-classes&#58;the-textarea-element-7"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> elements that do not have a <a id="pseudo-classes&#58;attr-textarea-readonly"></a><code>[readonly](https://html.spec.whatwg.org/multipage/form-elements.html#attr-textarea-readonly)</code> attribute, and that are not <a id="pseudo-classes&#58;concept-fe-disabled-2"></a>[disabled](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-disabled)
- elements that are <a id="pseudo-classes&#58;editing-host"></a>[editing hosts](https://html.spec.whatwg.org/multipage/interaction.html#editing-host) or <a id="pseudo-classes&#58;editable"></a>[editable](https://w3c.github.io/editing/docs/execCommand/#editable) and are neither <a id="pseudo-classes&#58;the-input-element-16"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements nor <a id="pseudo-classes&#58;the-textarea-element-8"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> elements

The <a id="pseudo-classes&#58;selector-read-only"></a><code>[:read-only](#selector-read-only)</code> <a id="pseudo-classes&#58;pseudo-class-28"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all other <a id="pseudo-classes&#58;html-elements-2"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements).

<strong><a id="selector-modal"></a><strong><code>&#58;modal</code></strong></strong>

The <a id="pseudo-classes&#58;selector-modal"></a><code>[:modal](#selector-modal)</code> <a id="pseudo-classes&#58;pseudo-class-29"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories:

- <a id="pseudo-classes&#58;the-dialog-element"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> elements whose <a id="pseudo-classes&#58;is-modal"></a>[is modal](https://html.spec.whatwg.org/multipage/interactive-elements.html#is-modal) is true
- elements whose <a id="pseudo-classes&#58;fullscreen-flag"></a>[fullscreen flag](https://fullscreen.spec.whatwg.org/#fullscreen-flag) is true

<strong><a id="selector-ltr"></a><strong><code>&#58;dir(ltr)</code></strong></strong>

MDN

[:dir](https://developer.mozilla.org/en-US/docs/Web/CSS/:dir)

Firefox49+Safari16.4+ChromeNo

------------------------------------------------------------------------

Opera?EdgeNo

------------------------------------------------------------------------

Edge (Legacy)?Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android?

The <a id="pseudo-classes&#58;selector-ltr"></a><code>[:dir(ltr)](#selector-ltr)</code> <a id="pseudo-classes&#58;pseudo-class-30"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all elements whose <a id="pseudo-classes&#58;the-directionality"></a>[directionality](https://html.spec.whatwg.org/multipage/dom.html#the-directionality) is '<a id="pseudo-classes&#58;concept-ltr"></a>[ltr](https://html.spec.whatwg.org/multipage/dom.html#concept-ltr)'.

<strong><a id="selector-rtl"></a><strong><code>&#58;dir(rtl)</code></strong></strong>

The <a id="pseudo-classes&#58;selector-rtl"></a><code>[:dir(rtl)](#selector-rtl)</code> <a id="pseudo-classes&#58;pseudo-class-31"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all elements whose <a id="pseudo-classes&#58;the-directionality-2"></a>[directionality](https://html.spec.whatwg.org/multipage/dom.html#the-directionality) is '<a id="pseudo-classes&#58;concept-rtl"></a>[rtl](https://html.spec.whatwg.org/multipage/dom.html#concept-rtl)'.

<strong><a id="selector-custom"></a><strong>Custom state pseudo-class</strong></strong>

The <a id="pseudo-classes&#58;selector-custom"></a><code>[:state(<var>identifier</var>)](#selector-custom)</code> pseudo-class must match all <a id="pseudo-classes&#58;custom-element"></a>[custom element](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element)s whose <a id="pseudo-classes&#58;states-set"></a>[states set](https://html.spec.whatwg.org/multipage/custom-elements.html#states-set)'s <a id="pseudo-classes&#58;set-entries"></a>[set entries](https://webidl.spec.whatwg.org/#dfn-set-entries) contains <var>identifier</var>.

<strong><a id="selector-heading"></a><strong><code>&#58;heading</code></strong></strong>

The <a id="pseudo-classes&#58;selector-heading"></a><code>[:heading](#selector-heading)</code> pseudo-class must match all <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements"></a><code>[h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>, <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-2"></a><code>[h2](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>, <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-3"></a><code>[h3](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>, <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-4"></a><code>[h4](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>, <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-5"></a><code>[h5](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>, and <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-6"></a><code>[h6](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> elements.

<strong><a id="selector-heading-functional"></a><strong><code>:heading(<var>integer</var>\#)</code></strong></strong>

The <a id="pseudo-classes&#58;selector-heading-functional"></a><code>[:heading(<var>integer</var>\#)](#selector-heading-functional)</code> pseudo-class must match all <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-7"></a><code>[h1](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>, <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-8"></a><code>[h2](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>, <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-9"></a><code>[h3](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>, <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-10"></a><code>[h4](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>, <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-11"></a><code>[h5](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>, and <a id="pseudo-classes&#58;the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-12"></a><code>[h6](https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> elements that have a <a id="pseudo-classes&#58;heading-level"></a>[heading level](https://html.spec.whatwg.org/multipage/sections.html#heading-level) of <var>integer</var>. [\[CSSSYNTAX\]](https://html.spec.whatwg.org/multipage/references.html#refsCSSSYNTAX) [\[CSSVALUES\]](https://html.spec.whatwg.org/multipage/references.html#refsCSSVALUES)

<strong><a id="selector-playing"></a><strong><code>&#58;playing</code></strong></strong>

The <a id="pseudo-classes&#58;selector-playing"></a><code>[:playing](#selector-playing)</code> <a id="pseudo-classes&#58;pseudo-class-32"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo-classes&#58;media-element"></a>[media elements](https://html.spec.whatwg.org/multipage/media.html#media-element) whose <a id="pseudo-classes&#58;dom-media-paused"></a><code>[paused](https://html.spec.whatwg.org/multipage/media.html#dom-media-paused)</code> attribute is false.

<strong><a id="selector-paused"></a><strong><code>&#58;paused</code></strong></strong>

The <a id="pseudo-classes&#58;selector-paused"></a><code>[:paused](#selector-paused)</code> <a id="pseudo-classes&#58;pseudo-class-33"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo-classes&#58;media-element-2"></a>[media elements](https://html.spec.whatwg.org/multipage/media.html#media-element) whose <a id="pseudo-classes&#58;dom-media-paused-2"></a><code>[paused](https://html.spec.whatwg.org/multipage/media.html#dom-media-paused)</code> attribute is true.

<strong><a id="selector-seeking"></a><strong><code>&#58;seeking</code></strong></strong>

The <a id="pseudo-classes&#58;selector-seeking"></a><code>[:seeking](#selector-seeking)</code> <a id="pseudo-classes&#58;pseudo-class-34"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo-classes&#58;media-element-3"></a>[media elements](https://html.spec.whatwg.org/multipage/media.html#media-element) whose <a id="pseudo-classes&#58;dom-media-seeking"></a><code>[seeking](https://html.spec.whatwg.org/multipage/media.html#dom-media-seeking)</code> attribute is true.

<strong><a id="selector-buffering"></a><strong><code>&#58;buffering</code></strong></strong>

The <a id="pseudo-classes&#58;selector-buffering"></a><code>[:buffering](#selector-buffering)</code> <a id="pseudo-classes&#58;pseudo-class-35"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo-classes&#58;media-element-4"></a>[media elements](https://html.spec.whatwg.org/multipage/media.html#media-element) whose <a id="pseudo-classes&#58;dom-media-paused-3"></a><code>[paused](https://html.spec.whatwg.org/multipage/media.html#dom-media-paused)</code> attribute is false, <a id="pseudo-classes&#58;dom-media-networkstate"></a><code>[networkState](https://html.spec.whatwg.org/multipage/media.html#dom-media-networkstate)</code> attribute is <a id="pseudo-classes&#58;dom-media-network&#95;loading"></a><code>[NETWORK_LOADING](https://html.spec.whatwg.org/multipage/media.html#dom-media-network_loading)</code>, and ready state is <a id="pseudo-classes&#58;dom-media-have&#95;current&#95;data"></a><code>[HAVE_CURRENT_DATA](https://html.spec.whatwg.org/multipage/media.html#dom-media-have_current_data)</code> or less.

<strong><a id="selector-stalled"></a><strong><code>&#58;stalled</code></strong></strong>

The <a id="pseudo-classes&#58;selector-stalled"></a><code>[:stalled](#selector-stalled)</code> <a id="pseudo-classes&#58;pseudo-class-36"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo-classes&#58;media-element-5"></a>[media elements](https://html.spec.whatwg.org/multipage/media.html#media-element) that match the <a id="pseudo-classes&#58;selector-buffering-2"></a><code>[:buffering](#selector-buffering)</code> <a id="pseudo-classes&#58;pseudo-class-37"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) and whose <a id="pseudo-classes&#58;is-currently-stalled"></a>[is currently stalled](https://html.spec.whatwg.org/multipage/media.html#is-currently-stalled) is true.

<strong>Note:</strong>

This pseudo-class is intended for displaying player state UI when video playback has been buffering for too long. Unlike the <a id="pseudo-classes&#58;event-media-stalled"></a><code>[stalled](https://html.spec.whatwg.org/multipage/media.html#event-media-stalled)</code> event, which fires whenever the network download has stalled regardless of playback or ready state.

<strong><a id="selector-muted"></a><strong><code>&#58;muted</code></strong></strong>

The <a id="pseudo-classes&#58;selector-muted"></a><code>[:muted](#selector-muted)</code> <a id="pseudo-classes&#58;pseudo-class-38"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo-classes&#58;media-element-6"></a>[media elements](https://html.spec.whatwg.org/multipage/media.html#media-element) that are <a id="pseudo-classes&#58;concept-media-muted"></a>[muted](https://html.spec.whatwg.org/multipage/media.html#concept-media-muted).

<strong><a id="selector-volume-locked"></a><strong><code>&#58;volume-locked</code></strong></strong>

The <a id="pseudo-classes&#58;selector-volume-locked"></a><code>[:volume-locked](#selector-volume-locked)</code> <a id="pseudo-classes&#58;pseudo-class-39"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo-classes&#58;media-element-7"></a>[media elements](https://html.spec.whatwg.org/multipage/media.html#media-element) when the user agent's <a id="pseudo-classes&#58;volume-locked"></a>[volume locked](https://html.spec.whatwg.org/multipage/media.html#volume-locked) is true.

<strong><a id="selector-open"></a><strong><code>&#58;open</code></strong></strong>

The <a id="pseudo-classes&#58;selector-open"></a><code>[:open](#selector-open)</code> <a id="pseudo-classes&#58;pseudo-class-40"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories:

- <a id="pseudo-classes&#58;the-details-element"></a><code>[details](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element)</code> elements that have an <a id="pseudo-classes&#58;attr-details-open"></a><code>[open](https://html.spec.whatwg.org/multipage/interactive-elements.html#attr-details-open)</code> attribute

- <a id="pseudo-classes&#58;the-dialog-element-2"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> elements that have an <a id="pseudo-classes&#58;attr-dialog-open"></a><code>[open](https://html.spec.whatwg.org/multipage/interactive-elements.html#attr-dialog-open)</code> attribute

- <a id="pseudo-classes&#58;the-select-element-6"></a><code>[select](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)</code> elements that are a <a id="pseudo-classes&#58;drop-down-box"></a>[drop-down box](https://html.spec.whatwg.org/multipage/rendering.html#drop-down-box) and whose drop-down boxes are open

- <a id="pseudo-classes&#58;the-input-element-17"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements that <a id="pseudo-classes&#58;input-support-picker"></a>[support a picker](https://html.spec.whatwg.org/multipage/input.html#input-support-picker) and whose pickers are open

<strong>Note:</strong>

This specification does not define when an element matches the <code>&#58;lang()</code> dynamic <a id="pseudo-classes&#58;pseudo-class-41"></a>[pseudo-class](https://drafts.csswg.org/selectors/#pseudo-class), as it is defined in sufficient detail in a language-agnostic fashion in Selectors. [\[SELECTORS\]](https://html.spec.whatwg.org/multipage/references.html#refsSELECTORS)

[← 4.13 Custom elements](https://html.spec.whatwg.org/multipage/custom-elements.html) — [Table of Contents](https://html.spec.whatwg.org/multipage/) — [5 Microdata →](https://html.spec.whatwg.org/multipage/microdata.html)
