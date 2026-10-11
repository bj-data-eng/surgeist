Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Backgrounds and Borders Module Level 3](https://drafts.csswg.org/css-backgrounds-3/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-11 UTC (2026-10-10 America/New_York). Source status: Editor’s Draft, 16 December 2025. Source edition and linked dated publication remain below.

Full captured HTML SHA-256: `ebf9643d9bf7228b590b57480841ea839930b603acf4ea97ef0644c2240e46f6` (1116562 bytes). Conversion input SHA-256: `11afcd91a294db12ebe39ecb5f9fb2e1714c9ad20e49f835fff78a1fad451e86` (31710 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained complete heading sections: `border-image-tables`, bounded at the next equal-or-higher heading, plus original header/legal notice. Other clauses remain upstream. Conversion uses existing HTML preparation, Pandoc 3.1.11.1 and semantic Lua filter. Scripts and styles are omitted without execution. Original IDs, prose, links, literal blocks, list nesting and table contents are verified against the conversion input. Any synthetic table headers or span expansion are representation changes. The unchanged malformed clearOverride list remains visible; Surgeist's interpretation is recorded in its issue contract separately.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Backgrounds and Borders Module Level 3

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 16 December 2025

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-backgrounds/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-backgrounds-3/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/>

<https://www.w3.org/TR/2020/CR-css-backgrounds-3-20201222/>

<https://www.w3.org/TR/2017/CR-css-backgrounds-3-20171017/>

<https://www.w3.org/TR/2014/CR-css3-background-20140909/>

<strong>Implementation Report:</strong>

<https://wpt.fyi/results/css/css-backgrounds>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-backgrounds-3)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Brad Kemper](mailto:brad.kemper@gmail.com) (Invited Expert)

<strong>Former Editor:</strong>

[Bert Bos](mailto:bert@w3.org) (W3C)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-backgrounds-3/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-backgrounds/>

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

### <a id="border-image-tables"></a>5.8. Effect on Tables[](#border-image-tables)

The <a id="ref-for-propdef-border-image⑥"></a>[border-image](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-image) properties apply to the border of tables and inline tables that have <a id="ref-for-propdef-border-collapse⑦"></a>[border-collapse](https://drafts.csswg.org/css2/#propdef-border-collapse) set to <a id="ref-for-valdef-border-collapse-collapse⑥"></a>[collapse](https://drafts.csswg.org/css2/#valdef-border-collapse-collapse). However, this specification does not define how such an image border is rendered. In particular, it does not define how the image border interacts with the borders of cells, rows and row groups at the edges of the table (see [border conflict resolution](https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#border-conflict-resolution) in [\[CSS2\]](https://drafts.csswg.org/css-backgrounds-3/#biblio-css2)).

It is expected that a future specification will define the rendering. It is recommended that UAs do not apply border images to tables with collapsed borders until then.
