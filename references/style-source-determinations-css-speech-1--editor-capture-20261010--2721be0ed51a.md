Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Speech Module Level 1](https://drafts.csswg.org/css-speech-1/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10. Source status: Editor’s Draft, 2 July 2026.

Full captured HTML SHA-256: `68cdd69dbe11d68dc09c5a33ea6e79c951f6f6839c43ec75b87e48cd6b16bd77` (322882 bytes). Conversion input SHA-256: `2721be0ed51a1fb0eac0b78cc2564be95cad3708237180415f3eb41ed91f09ce` (29102 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained scope: `speaking-props-speak`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Speech Module Level 1

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 2 July 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-speech-1/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-speech-1/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2020/CR-css-speech-1-20200310/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-speech-1)

<strong>Editors:</strong>

[Léonie Watson](mailto:lwatson@tetralogical.com) (Tetralogical)

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

<strong>Former Editors:</strong>

[Daniel Weck](mailto:dweck@daisy.org) (DAISY Consortium)

Claudio Santambrogio (Opera Software)

[Dave Raggett](mailto:dsr@w3.org) (W3C / Canon)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-speech-1/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

### <a id="speaking-props-speak"></a>7.1.  The <a id="ref-for-propdef-speak②"></a>[speak](#propdef-speak) property[](#speaking-props-speak)

| Field                                                                                    | Definition                                                                                                                               |
|------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-speak"></a><strong>speak</strong>                                                                                         |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | auto <a id="ref-for-comb-one①⓪"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) never <a id="ref-for-comb-one①①"></a>\| always |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | auto                                                                                                                                     |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                      |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | yes                                                                                                                                      |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | N/A                                                                                                                                      |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified value                                                                                                                          |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                              |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                                                 |

The <a id="ref-for-propdef-speak③"></a>[speak](#propdef-speak) property determines whether or not to render text aurally.

Note: The functionality provided by this property has no match in the SSML markup language [\[SSML\]](https://drafts.csswg.org/css-speech-1/#biblio-ssml).

<strong><a id="valdef-speak-auto"></a><strong>auto</strong></strong>

Resolves to a computed value of <a id="ref-for-valdef-speak-never"></a>[never](#valdef-speak-never) when <a id="ref-for-propdef-display"></a>[display](https://drafts.csswg.org/css-display-4/#propdef-display) is <a id="ref-for-valdef-display-none"></a>[none](https://drafts.csswg.org/css-display-4/#valdef-display-none), otherwise resolves to a computed value of <a id="ref-for-valdef-speak-auto"></a>[auto](#valdef-speak-auto). The used value of a computed <a id="ref-for-valdef-speak-auto①"></a>auto is equivalent to <a id="ref-for-valdef-speak-always"></a>[always](#valdef-speak-always) if <a id="ref-for-propdef-visibility"></a>[visibility](https://drafts.csswg.org/css-display-4/#propdef-visibility) is <a id="ref-for-valdef-visibility-visible"></a>[visible](https://drafts.csswg.org/css-display-4/#valdef-visibility-visible) and to <a id="ref-for-valdef-speak-never①"></a>never otherwise.

Note: The <a id="ref-for-valdef-display-none①"></a>[none](https://drafts.csswg.org/css-display-4/#valdef-display-none) value of the <a id="ref-for-propdef-display①"></a>[display](https://drafts.csswg.org/css-display-4/#propdef-display) property cannot be overridden by descendants of the selected element, but the <a id="ref-for-valdef-speak-auto②"></a>[auto](#valdef-speak-auto) value of <a id="ref-for-propdef-speak④"></a>[speak](#propdef-speak) can, however, be overridden using either of <a id="ref-for-valdef-speak-never②"></a>[never](#valdef-speak-never) or <a id="ref-for-valdef-speak-always①"></a>[always](#valdef-speak-always).

<strong><a id="valdef-speak-never"></a><strong>never</strong></strong>

This value causes an element (including pauses, cues, rests and actual content) to not be rendered (i.e., the element has no effect in the aural dimension).

Note: Any of the descendants of the affected element are allowed to override this value, so descendants can actually take part in the aural rendering despite using <a id="ref-for-propdef-display②"></a>[display: none](https://drafts.csswg.org/css-display-4/#propdef-display) at this level. However, the pauses, cues, and rests of the ancestor element remain “deactivated” in the aural dimension, and therefore do not contribute to the [collapsing of pauses](https://drafts.csswg.org/css-speech-1/#collapsed-pauses) or additive behavior of adjoining rests.

<strong><a id="valdef-speak-always"></a><strong>always</strong></strong>

The element is rendered aurally (regardless of its <a id="ref-for-propdef-display③"></a>[display](https://drafts.csswg.org/css-display-4/#propdef-display) value, or the <a id="ref-for-propdef-display④"></a>display or <a id="ref-for-propdef-speak⑤"></a>[speak](#propdef-speak) values of its ancestors).

Note: Using this value can result in the element being rendered in the aural dimension even though it would not be rendered on the visual canvas.
