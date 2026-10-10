Attribution and reformatting notice added for Surgeist on 2026-10-10

This bounded, reformatted source excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Generated Content Module Level 3](https://drafts.csswg.org/css-content-3/) under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice remains below.

# Source provenance and excerpt boundary

Retrieved: 2026-10-10. Source status: Editor’s Draft, 9 October 2026. Full captured HTML SHA-256: `a0db7cbeb488725fe33adf99a6dde5adbca4afe14ed45fa5bdc7f44ef864dc94` (318497 bytes). Bounded conversion input SHA-256: `586d408a8de7bd160fa01009331ef04ec7d283e7e71278a0dfe9ad63c0b9f9c0` (82171 bytes). Page revision metadata: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`.

Retained complete sections: `content-property` (including generated-content accessibility), `strings`, and `leaders`, bounded at the next equal-or-higher-level heading. The original source header and complete legal notice remain. Other clauses are outside this partial capture and resolve through upstream links. The leaders section retains all four unresolved questions, eight rendering rules, the entire procedure and examples. This capture does not present Surgeist's selected fallback policy as a CSSWG resolution.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter. Scripts/styles were omitted without execution; IDs, prose, links, literal blocks and table cells were checked against the bounded input. Source figures remain links to upstream images. Table headers may be expanded only according to the source cell model.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Generated Content Module Level 3

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 9 October 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-content-3/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-content-3/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2016/WD-css-content-3-20160602/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-content-3)

[Inline In Spec](https://drafts.csswg.org/css-content-3/#issues-index)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple, formerly Mozilla)

[Mike Bremford](mailto:mike@bfo.com) (BFO)

<strong>Former Editors:</strong>

[Dave Cramer](mailto:dauwhe@gmail.com) (Hachette Livre)

[Håkon Wium Lie](mailto:howcome@opera.com) (Opera Software)

[Ian Hickson](mailto:ian@hixie.ch) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-content-3/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-content/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="content-property"></a>1.  Inserting and Replacing Content: the <a id="ref-for-propdef-content①"></a>[content](#propdef-content) property[](#content-property)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
|------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-content"></a><strong>content</strong>                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | normal <a id="ref-for-comb-one"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) none <a id="ref-for-comb-one①"></a>\| \[ <a id="ref-for-typedef-content-replacement"></a>[\<content-replacement\>](#typedef-content-replacement) <a id="ref-for-comb-one②"></a>\| <a id="ref-for-typedef-content-list"></a>[\<content-list\>](#typedef-content-list) \] \[/ <a id="ref-for-typedef-alt-text"></a>[\<alt-text\>](#typedef-alt-text) \]<a id="ref-for-mult-opt"></a>[?](https://drafts.csswg.org/css-values-4/#mult-opt) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | normal                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | all elements, tree-abiding pseudo-elements, and page margin boxes                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | See prose below                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |

Tests

- [attr-case-sensitivity-001.html](https://wpt.fyi/results/css/css-content/attr-case-sensitivity-001.html) [(live test)](http://wpt.live/css/css-content/attr-case-sensitivity-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/attr-case-sensitivity-001.html)
- [attr-case-sensitivity-002.html](https://wpt.fyi/results/css/css-content/attr-case-sensitivity-002.html) [(live test)](http://wpt.live/css/css-content/attr-case-sensitivity-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/attr-case-sensitivity-002.html)
- [attr-case-sensitivity-003.html](https://wpt.fyi/results/css/css-content/attr-case-sensitivity-003.html) [(live test)](http://wpt.live/css/css-content/attr-case-sensitivity-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/attr-case-sensitivity-003.html)
- [computed-value.html](https://wpt.fyi/results/css/css-content/computed-value.html) [(live test)](http://wpt.live/css/css-content/computed-value.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/computed-value.html)
- [content-animation.html](https://wpt.fyi/results/css/css-content/content-animation.html) [(live test)](http://wpt.live/css/css-content/content-animation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/content-animation.html)
- [content-no-interpolation.html](https://wpt.fyi/results/css/css-content/content-no-interpolation.html) [(live test)](http://wpt.live/css/css-content/content-no-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/content-no-interpolation.html)
- [content-none-select-1.html](https://wpt.fyi/results/css/css-content/content-none-select-1.html) [(live test)](http://wpt.live/css/css-content/content-none-select-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/content-none-select-1.html)
- [element-replacement-alt.html](https://wpt.fyi/results/css/css-content/element-replacement-alt.html) [(live test)](http://wpt.live/css/css-content/element-replacement-alt.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/element-replacement-alt.html)
- [element-replacement-display-contents.html](https://wpt.fyi/results/css/css-content/element-replacement-display-contents.html) [(live test)](http://wpt.live/css/css-content/element-replacement-display-contents.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/element-replacement-display-contents.html)
- [element-replacement-display-none.html](https://wpt.fyi/results/css/css-content/element-replacement-display-none.html) [(live test)](http://wpt.live/css/css-content/element-replacement-display-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/element-replacement-display-none.html)
- [element-replacement-dynamic.html](https://wpt.fyi/results/css/css-content/element-replacement-dynamic.html) [(live test)](http://wpt.live/css/css-content/element-replacement-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/element-replacement-dynamic.html)
- [element-replacement-gradient.html](https://wpt.fyi/results/css/css-content/element-replacement-gradient.html) [(live test)](http://wpt.live/css/css-content/element-replacement-gradient.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/element-replacement-gradient.html)
- [element-replacement-image-alt.html](https://wpt.fyi/results/css/css-content/element-replacement-image-alt.html) [(live test)](http://wpt.live/css/css-content/element-replacement-image-alt.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/element-replacement-image-alt.html)
- [element-replacement.html](https://wpt.fyi/results/css/css-content/element-replacement.html) [(live test)](http://wpt.live/css/css-content/element-replacement.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/element-replacement.html)
- [content-computed.html](https://wpt.fyi/results/css/css-content/parsing/content-computed.html) [(live test)](http://wpt.live/css/css-content/parsing/content-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/parsing/content-computed.html)
- [content-counter-valid.html](https://wpt.fyi/results/css/css-content/parsing/content-counter-valid.html) [(live test)](http://wpt.live/css/css-content/parsing/content-counter-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/parsing/content-counter-valid.html)
- [content-invalid.html](https://wpt.fyi/results/css/css-content/parsing/content-invalid.html) [(live test)](http://wpt.live/css/css-content/parsing/content-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/parsing/content-invalid.html)
- [content-valid.html](https://wpt.fyi/results/css/css-content/parsing/content-valid.html) [(live test)](http://wpt.live/css/css-content/parsing/content-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/parsing/content-valid.html)

<em>User agents are expected to support this property on all media, including non-visual ones.</em>

The <a id="ref-for-propdef-content②"></a>[content](#propdef-content) property specifies the box’s contents for stylistic purposes. It can replace the box with an image (turning it into a <a id="ref-for-replaced-element"></a>[replaced element](https://drafts.csswg.org/css-display-4/#replaced-element)), or provide it with arbitrary inline-level content (text and images). It can also, for some types of boxes, control whether the box renders at all.

<a id="typedef-content-replacement"></a><a id="ref-for-typedef-content-replacement①"></a><a id="ref-for-typedef-image"></a><a id="typedef-content-list"></a><a id="ref-for-typedef-content-list①"></a><a id="ref-for-string-value"></a><a id="ref-for-comb-one③"></a><a id="ref-for-typedef-image①"></a><a id="ref-for-comb-one④"></a><a id="ref-for-funcdef-attr"></a><a id="ref-for-comb-one⑤"></a><a id="ref-for-comb-one⑥"></a><a id="ref-for-typedef-quote"></a><a id="ref-for-comb-one⑦"></a><a id="ref-for-funcdef-content-leader"></a><a id="ref-for-comb-one⑧"></a><a id="ref-for-typedef-target"></a><a id="ref-for-comb-one⑨"></a><a id="ref-for-funcdef-string"></a><a id="ref-for-comb-one①⓪"></a><a id="ref-for-funcdef-content"></a><a id="ref-for-comb-one①①"></a><a id="ref-for-typedef-counter"></a><a id="ref-for-comb-one①②"></a><a id="ref-for-mult-one-plus"></a><a id="typedef-alt-text"></a><a id="ref-for-typedef-alt-text①"></a><a id="ref-for-string-value①"></a><a id="ref-for-comb-one①③"></a><a id="ref-for-typedef-counter①"></a><a id="ref-for-comb-one①④"></a><a id="ref-for-funcdef-attr①"></a><a id="ref-for-mult-one-plus①"></a>

``` text
<content-replacement> = <image>
<content-list> = [ <string> | <image> | <attr()> | contents
                | <quote> | <leader()> | <target> | <string()> | <content()>
                | <counter> | <symbolic-counter-style> ]+
<alt-text> = [ <string> | <counter> | <attr()> ]+
```

Values have the following meanings:

<strong><a id="valdef-content-normal"></a><strong>normal</strong></strong>

For a regular element or <a id="ref-for-page-margin-boxes"></a>[page-margin box](https://drafts.csswg.org/css-page-3/#page-margin-boxes), this computes to <a id="ref-for-valdef-content-contents"></a>[contents](https://drafts.csswg.org/css-content-3/#valdef-content-contents).

For <a id="ref-for-selectordef-before"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before) and <a id="ref-for-selectordef-after"></a>[::after](https://drafts.csswg.org/css-pseudo-4/#selectordef-after), this computes to <a id="ref-for-valdef-content-none"></a>[none](#valdef-content-none).

For <a id="ref-for-selectordef-marker"></a>[::marker](https://drafts.csswg.org/css-pseudo-4/#selectordef-marker), <a id="ref-for-selectordef-placeholder"></a>[::placeholder](https://drafts.csswg.org/css-pseudo-4/#selectordef-placeholder), and <a id="ref-for-selectordef-file-selector-button"></a>[::file-selector-button](https://drafts.csswg.org/css-pseudo-4/#selectordef-file-selector-button), this computes to itself (<a id="ref-for-valdef-content-normal"></a>[normal](#valdef-content-normal)).

<strong><a id="valdef-content-none"></a><strong>none</strong></strong>

On regular elements, this behaves as <a id="ref-for-valdef-content-normal①"></a>[normal](#valdef-content-normal).

On <a id="ref-for-pseudo-element"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) it inhibits the creation of the pseudo-element as if it had <a id="ref-for-propdef-display"></a>[display: none](https://drafts.csswg.org/css-display-4/#propdef-display).

In neither case does it prevent any pseudo-elements which have this element or pseudo-element as an <a id="ref-for-originating-element"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) from being generated.

<strong><a id="valdef-content-content-replacement"></a><strong><a id="ref-for-typedef-content-replacement②"></a>[\<content-replacement\>](#typedef-content-replacement)</strong></strong>

<a id="replaced"></a> Makes the element or <a id="ref-for-pseudo-element①"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) a <a id="ref-for-replaced-element①"></a>[replaced element](https://drafts.csswg.org/css-display-4/#replaced-element), filled with the specified <a id="ref-for-typedef-image②"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image). (In effect, it becomes equivalent to an HTML <code><a id="ref-for-the-img-element"></a>[img](https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element)</code> element.) Its normal contents are suppressed and do not generate boxes, as if they were <a id="ref-for-propdef-display①"></a>[display: none](https://drafts.csswg.org/css-display-4/#propdef-display).

If the <a id="ref-for-typedef-image③"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image) represents an <a id="ref-for-invalid-image"></a>[invalid image](https://drafts.csswg.org/css-images-4/#invalid-image), the behavior is undefined.

<a id="issue-470ff419"></a>

<strong>Issue:</strong>

[](#issue-470ff419) Possible ways of handling an invalid image include a) render it as an image with zero <a id="ref-for-natural-dimensions"></a>[natural](https://drafts.csswg.org/css-images-3/#natural-dimensions) width and height, filled with transparent black, b) render a broken image icon, c) render the alt text, if any, or the contents, if not. [\[Issue \#218\]](https://github.com/w3c/csswg-drafts/issues/218)

Note: Replaced elements do not have <a id="ref-for-selectordef-before①"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before) or <a id="ref-for-selectordef-after①"></a>[::after](https://drafts.csswg.org/css-pseudo-4/#selectordef-after) <a id="ref-for-pseudo-element②"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element); the <a id="ref-for-propdef-content③"></a>[content](#propdef-content) property replaces their entire contents.

<a id="issue-5248885e"></a>

<strong>Issue:</strong>

[](#issue-5248885e) This value has historically been treated as <a id="ref-for-typedef-content-list②"></a>[\<content-list\>](#typedef-content-list) on <a id="ref-for-selectordef-before②"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before) and <a id="ref-for-selectordef-after②"></a>[::after](https://drafts.csswg.org/css-pseudo-4/#selectordef-after). There might be a Web-compat requirement on this, so these pseudo-elements might need an exception. It might be possible to limit this exception to cases without "alt text". [\[Issue \#2889\]](https://github.com/w3c/csswg-drafts/issues/2889)

<a id="content-values"></a><strong>[](#content-values)<a id="valdef-content-content-list"></a><strong><a id="ref-for-typedef-content-list③"></a>[\<content-list\>](#typedef-content-list)</strong></strong>

On regular elements, this value has no effect.

On <a id="ref-for-pseudo-element③"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) and <a id="ref-for-page-margin-boxes①"></a>[page-margin boxes](https://drafts.csswg.org/css-page-3/#page-margin-boxes), it replaces the box’s contents with the specified content. (Its normal contents are suppressed, as if they were <a id="ref-for-propdef-display②"></a>[display: none](https://drafts.csswg.org/css-display-4/#propdef-display).)

Each value specified contributes an <a id="ref-for-inline-level-box"></a>[inline-level box](https://drafts.csswg.org/css-display-4/#inline-level-box) to the element’s contents, in the order specified. For <a id="ref-for-typedef-image④"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image), this is an inline anonymous replaced element; for the others, it’s an anonymous inline run of text.

Note: If the value of <a id="ref-for-typedef-content-list④"></a>[\<content-list\>](#typedef-content-list) is a single <a id="ref-for-typedef-image⑤"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image), it must instead be interpreted as a <a id="ref-for-typedef-content-replacement③"></a>[\<content-replacement\>](#typedef-content-replacement).

Tests

- [pseudo-element-inline-box.html](https://wpt.fyi/results/css/css-content/pseudo-element-inline-box.html) [(live test)](http://wpt.live/css/css-content/pseudo-element-inline-box.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-content/pseudo-element-inline-box.html)

<strong><a id="valdef-content-alt-text"></a><strong><a id="ref-for-typedef-alt-text②"></a>[\<alt-text\>](#typedef-alt-text)</strong></strong>

Specifies the “alternative content” for the element, which is supplied to the accessibility tree for rendering to speech, braille, etc. instead of the “visual content” provided before the slash. See [§ 1.2 Alternative Text for Accessibility](#alt) for details. If omitted, the element has no “alternative content”; the main content is used for all media.

<a id="example-36e84f01"></a>

<strong>Example:</strong>

[](#example-36e84f01) To correctly use <a id="ref-for-typedef-counter②"></a>[\<counter\>](https://drafts.csswg.org/css-lists-3/#typedef-counter) in "alt text" in unsupported browsers one should specify <a id="ref-for-propdef-content④"></a>[content](#propdef-content) property twice. First, a fallback without counter in "alt text". Second, use counter in "alt text".

``` text
::before {
  content: "Chapter" counter(chapter);
  content: "Chapter" counter(chapter) / "Chapter" counter(chapter);
}
```

<a id="issue-a8fbc981"></a>

<strong>Issue:</strong>

[](#issue-a8fbc981) Should the contents keyword be replaced with <a id="ref-for-funcdef-content①"></a>[content()](https://drafts.csswg.org/css-content-3/#funcdef-content)?

### <a id="accessibility"></a>1.1.  Accessibility of Generated Content[](#accessibility)

Generated content should be searchable, selectable, and available to assistive technologies. The <a id="ref-for-propdef-content⑤"></a>[content](#propdef-content) property applies to speech and generated content must be rendered for speech output. [\[CSS3-SPEECH\]](https://drafts.csswg.org/css-content-3/#biblio-css3-speech)

<a id="issue-9c60e2c1"></a>

<strong>Issue:</strong>

[](#issue-9c60e2c1) Start work on an AAM for CSS.

### <a id="alt"></a>1.2.  Alternative Text for Accessibility[](#alt)

Content intended for visual media sometimes needs alternative text for speech output or other non-visual mediums. The <a id="ref-for-propdef-content⑥"></a>[content](#propdef-content) property thus accepts alternative text to be specified after a slash (/) after the last <a id="ref-for-typedef-content-list⑤"></a>[\<content-list\>](#typedef-content-list). If such alternative text is provided, it must be used for speech output instead.

This allows, for example, purely decorative text to be elided in speech output (by providing the empty string as alternative text), and allows authors to provide more readable alternatives to images, icons, or text-encoded symbols.

<a id="example-ff5073a7"></a>

<strong>Example:</strong>

[](#example-ff5073a7) Here the content property is an image, so the alt value is required to provide alternative text.

``` text
.new::before {
 content: url(./img/star.png) / "New!";
  /* or a localized attribute from the DOM: attr("data-alt") */
}
```

<a id="example-954890b1"></a>

<strong>Example:</strong>

[](#example-954890b1) If the <a id="ref-for-pseudo-element④"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) is purely decorative and its function is covered elsewhere, setting alt to the empty string can avoid reading out the decorative element. Here the ARIA attribute will be spoken as "collapsed". Without the empty string alt value, the content would also be spoken as "Black right-pointing pointer".

``` text
.expandable::before {
 content: "\25BA" / "";
/* a.k.a. ► */
 /* aria-expanded="false" already in DOM,
   so this pseudo-element is decorative */
}
```

## <a id="strings"></a>2.  Basic Strings and Images: <a id="ref-for-string-value②"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value), <a id="ref-for-funcdef-attr②"></a>[\<attr()\>](https://drafts.csswg.org/css-values-5/#funcdef-attr), <a id="ref-for-typedef-counter③"></a>[\<counter\>](https://drafts.csswg.org/css-lists-3/#typedef-counter), <a id="ref-for-typedef-symbolic-glyph"></a>[\<symbolic-glyph\>](https://drafts.csswg.org/css-counter-styles-3/#typedef-symbolic-glyph), and <a id="ref-for-typedef-image⑥"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image) values[](#strings)

<strong><a id="valdef-content-string"></a><strong><a id="ref-for-string-value③"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value)</strong></strong>

Represents an anonymous inline box filled with the specified text.

Note: <a id="ref-for-white-space"></a>[White space](https://drafts.csswg.org/css-text-4/#white-space) in the string is handled the same as in literal text, and controlled by the properties in [\[CSS-TEXT-3\]](https://drafts.csswg.org/css-content-3/#biblio-css-text-3) and elsewhere. In particular, <a id="ref-for-white-space①"></a>white space character can collapse, even across multiple strings, such as in <a id="ref-for-propdef-content⑦"></a>[content: "First " " Second";](#propdef-content), which by default will render similar to <code>&#34;First Second&#34;</code> (with a single visible space between the two words).

<strong><a id="ref-for-funcdef-attr③"></a>[\<attr()\>](https://drafts.csswg.org/css-values-5/#funcdef-attr)</strong>

The <a id="valdef-content-attr"></a><strong>attr()</strong> functional notation represents the string stored as the specified attribute’s value. Its argument is a <a id="ref-for-css-qualified-name"></a>[CSS qualified name](https://drafts.csswg.org/css-namespaces-3/#css-qualified-name) (<code>qname</code>) representing the attribute name and namespace, if any. See [\[CSS3-NAMESPACE\]](https://drafts.csswg.org/css-content-3/#biblio-css3-namespace).

Note: As in [attribute selectors](https://drafts.csswg.org/selectors-4/#attrnmsp), attribute names without an explicit namespace do not associate with any default namespace.

<a id="counters"></a><strong>[](#counters)<a id="ref-for-typedef-counter④"></a>[\<counter\>](https://drafts.csswg.org/css-lists-3/#typedef-counter)</strong>

See [CSS Lists 3 § 4 Automatic Numbering With Counters](https://drafts.csswg.org/css-lists-3/#auto-numbering).

<strong><a id="ref-for-typedef-symbolic-glyph①"></a>[\<symbolic-glyph\>](https://drafts.csswg.org/css-counter-styles-3/#typedef-symbolic-glyph)</strong>

The <a id="ref-for-predefined-symbolic-counter-style"></a>[predefined symbolic counter style](https://drafts.csswg.org/css-counter-styles-3/#predefined-symbolic-counter-style), without its <a id="ref-for-descdef-counter-style-prefix"></a>[prefix](https://drafts.csswg.org/css-counter-styles-3/#descdef-counter-style-prefix) or <a id="ref-for-descdef-counter-style-suffix"></a>[suffix](https://drafts.csswg.org/css-counter-styles-3/#descdef-counter-style-suffix), as drawn by the UA for <a id="ref-for-propdef-list-style-type"></a>[list-style-type](https://drafts.csswg.org/css-lists-3/#propdef-list-style-type).

<a id="example-5f2461d5"></a>

<strong>Example:</strong>

[](#example-5f2461d5) A disclosure widget can point its marker at the widget’s state without involving a <a id="ref-for-counter"></a>[counter](https://drafts.csswg.org/css-lists-3/#counter):

``` text
summary::marker { content: disclosure-closed; }
details[open] > summary::marker { content: disclosure-open; }
```

Note: Only the "symbolic" counter styles are allowed, as they’re the only styles that don’t depend on a counter value at all. Only the predefined ones are allowed, as an author-defined symbolic style can just be copied over as a string or image, while the predefined ones might be UA-specific images that can’t be directly referenced.

<a id="content-uri"></a><strong>[](#content-uri)<a id="valdef-content-image"></a><strong><a id="ref-for-typedef-image⑦"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image)</strong></strong>

Represents an anonymous inline replaced element filled with the specified <a id="ref-for-typedef-image⑧"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image).

If an <a id="ref-for-typedef-image⑨"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image) represents an <a id="ref-for-invalid-image①"></a>[invalid image](https://drafts.csswg.org/css-images-4/#invalid-image), the user agent must (consistently) do one of the following:

- “Skip” the <a id="ref-for-typedef-image①⓪"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image), generating nothing for it.

- Display some indication that the image can’t be displayed in place of the <a id="ref-for-typedef-image①①"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image), such as a “broken image” icon.

## <a id="leaders"></a>5.  Leaders[](#leaders)

A leader, sometimes known as a tab leader or a dot leader, is a repeating pattern used to visually connect content across horizontal spaces. They are most commonly used in tables of contents, between titles and page numbers. The <a id="ref-for-funcdef-leader"></a>[leader()](#funcdef-leader) function, as a value for the content property, is used to create leaders in CSS. This function takes a string (the leader string), which describes the repeating pattern for the leader.

### <a id="leader-function"></a>5.1.  The <a id="ref-for-funcdef-leader①"></a>[leader()](#funcdef-leader) function[](#leader-function)

<strong><a id="funcdef-content-leader"></a><strong>leader( <a id="ref-for-typedef-leader-type"></a>[\<leader-type\>](#typedef-leader-type) )</strong></strong>

Inserts a leader. See the section on [leaders](#leaders) for more information.

<a id="funcdef-leader"></a><a id="ref-for-typedef-leader-type①"></a><a id="typedef-leader-type"></a><a id="ref-for-typedef-leader-type②"></a><a id="ref-for-comb-one②①"></a><a id="ref-for-comb-one②②"></a><a id="ref-for-comb-one②③"></a><a id="ref-for-string-value⑨"></a>

``` text
leader() = leader( <leader-type> )
<leader-type> = dotted | solid | space | <string>
```

Three keywords are shorthand values for common strings:

<strong><a id="valdef-leader-dotted"></a><strong>dotted</strong></strong>

Equivalent to leader(".")

<strong><a id="valdef-leader-solid"></a><strong>solid</strong></strong>

Equivalent to leader("\_")

<strong><a id="valdef-leader-space"></a><strong>space</strong></strong>

Equivalent to leader(" ")

<strong><a id="valdef-leader-string"></a><strong><a id="ref-for-string-value①⓪"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value)</strong></strong>

Issue: Define this.

<a id="example-e0047436"></a>

<strong>Example:</strong>

[](#example-e0047436)

``` text
ol.toc a::after {
  content: leader('.') target-counter(attr(href), page);
}

<h1>Table of Contents</h1>
<ol class="toc">
<li><a href="#chapter1">Loomings</a></li>
<li><a href="#chapter2">The Carpet-Bag</a></li>
<li><a href="#chapter3">The Spouter-Inn</a></li>
</ol>
```

This might result in:

``` text
Table of Contents

1. Loomings.....................1
2. The Carpet-Bag...............9
3. The Spouter-Inn.............13
```

<a id="issue-2f5c8d3f"></a>

<strong>Issue:</strong>

[](#issue-2f5c8d3f) Do leaders depend on the assumption that the content after the leader is right-aligned (end-aligned)?

### <a id="leader-rules"></a>5.2.  Rendering leaders[](#leader-rules)

Consider a line which contains the content before the leader (the “before content”), the leader, and the content after the leader (the “after content”). Leaders obey the following rules:

1.  The leader string must appear in full at least once.

2.  The leader should be as long as possible

3.  Visible characters in leaders should vertically align with each other when possible.

4.  Line break characters in the leader string must be ignored.

5.  White space in the leader string follows normal CSS rules.

6.  A leader only appears between the start content and the end content.

7.  A leader only appears on a single line, even if the before content and after content are on different lines.

8.  A leader can’t be the only thing on a line.

### <a id="leader-alignment"></a>5.3.  Procedure for rendering leaders[](#leader-alignment)

1.  Lay out the <var>before content</var>, until reaching the line where the <var>before content</var> ends.

    ``` text
    BBBBBBBBBB
    BBB
    ```

2.  The leader string consists of one or more glyphs, and is thus an inline box. A leader is a row of these boxes, drawn from the end edge to the start edge, where only those boxes not overlaid by the before or after content. On this line, draw the leader string, starting from the end edge, repeating as many times as possible until reaching the start edge.

    ``` text
    BBBBBBBBBB
    ..........
    ```

3.  Draw the before and after content on top of the leader. If any part of the <var>before content</var> or <var>after content</var> overlaps a glyph in a leader string box, that glyph is not displayed.

    ``` text
    BBBBBBBBBB
    BBB....AAA
    ```

4.  If one full copy of the leader string is not visible:

    ``` text
    BBBBBBB
    BBBBBBA
    ```

    Insert a line break after the <var>before content</var>, draw the leader on the next line, and draw the <var>after content</var> on top, and hide any leader strings that are not fully displayed.

    ``` text
    BBBBBBB
    BBBBBB
    ......A
    ```

<a id="issue-449754e4"></a>

<strong>Issue:</strong>

[](#issue-449754e4) what to do if <var>after content</var> is wider than the line box?

<a id="issue-bb18b3e4"></a>

<strong>Issue:</strong>

[](#issue-bb18b3e4) Leaders don’t quite work in table layouts. How can we fix this?

<figure>
<img src="https://drafts.csswg.org/css-content-3/images/leader.001.jpg" alt="drawing leaders" />
<figcaption>Procedure for drawing leaders</figcaption>
</figure>

<figure>
<img src="https://drafts.csswg.org/css-content-3/images/leader.002.jpg" alt="drawing leaders" />
<figcaption>Procedure for drawing leaders when the content doesn’t fit on a single line</figcaption>
</figure>
