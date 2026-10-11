Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Borders and Box Decorations Module Level 4](https://drafts.csswg.org/css-borders-4/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-11 UTC (2026-10-10 America/New_York). Source status: W3C Working Draft, 8 October 2026. Source edition and linked dated publication remain below.

Full captured HTML SHA-256: `b592227c041d8681a2de4f11503a995ea1fc57ec17975fb251b5201d3ac32f7e` (750625 bytes). Conversion input SHA-256: `3d13a172cd34c6ab633e60341b2733d32b218a7dcab71e1281f24aec59a667a5` (28725 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained complete heading sections: `border-image-tables`, bounded at the next equal-or-higher heading, plus original header/legal notice. Other clauses remain upstream. Conversion uses existing HTML preparation, Pandoc 3.1.11.1 and semantic Lua filter. Scripts and styles are omitted without execution. Original IDs, prose, links, literal blocks, list nesting and table contents are verified against the conversion input. Any synthetic table headers or span expansion are representation changes. The unchanged malformed clearOverride list remains visible; Surgeist's interpretation is recorded in its issue contract separately.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Borders and Box Decorations Module Level 4

<a id="w3c-state"></a>[W3C Working Draft](https://www.w3.org/standards/types/#WD), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://www.w3.org/TR/2026/WD-css-borders-4-20261008/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-borders-4/>

<strong>Editor's Draft:</strong>

<https://drafts.csswg.org/css-borders-4/>

<strong>History:</strong>

<https://www.w3.org/standards/history/css-borders-4/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-borders-4)

[Inline In Spec](https://drafts.csswg.org/css-borders-4/#issues-index)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Lea Verou](http://lea.verou.me/about/) (Invited Expert)

[Sebastian Zartner](mailto:sebastianzartner@gmail.com) (Invited Expert)

Noam Rosenthal (Google)

<strong>Former Editor:</strong>

[Bert Bos](mailto:bert@w3.org) (W3C)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-borders-4/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-borders/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

### <a id="border-image-tables"></a>4.8. Effect on Tables[](#border-image-tables)

The <a id="ref-for-propdef-border-image⑥"></a>[border-image](https://drafts.csswg.org/css-borders-4/#propdef-border-image) properties apply to the border of tables and inline tables that have <a id="ref-for-propdef-border-collapse⑦"></a>[border-collapse](https://www.w3.org/TR/CSS2/tables.html#propdef-border-collapse) set to collapse. However, this specification does not define how such an image border is rendered. In particular, it does not define how the image border interacts with the borders of cells, rows and row groups at the edges of the table (see [border conflict resolution](https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#border-conflict-resolution) in [\[CSS2\]](https://drafts.csswg.org/css-borders-4/#biblio-css2)).

It is expected that a future specification will define the rendering. It is recommended that UAs do not apply border images to tables with collapsed borders until then.
