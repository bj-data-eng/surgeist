Attribution and reformatting notice added for the Surgeist overflow geometry source-resolution task.

Source: [CSS Overflow Module Level 3](https://drafts.csswg.org/css-overflow-3/), editor draft dated 13 August 2026, retrieved 2026-10-10. Copyright © 2026 World Wide Web Consortium; its source legal notice links the [W3C permissive document license](https://www.w3.org/copyright/software-license/). The complete source header and its copyright/legal links are retained below.

Identity: full HTML SHA-256 `4635826dcd37bbf938b4043c08359ccf5c9a0befe88c6c8a33480bae523cd979` (339912 bytes); bounded input SHA-256 `86c09a636507eabbbe39c1a9507d79602e9c75451170c90077c1b4c6cb207ed5` (103289 bytes). Source revision metadata: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`.

Scope: faithful partial capture of the complete heading sections `overflow-concepts` (§2, all subsections) and `overflow-properties` (§3, all subsections), through the next equal-level heading. Included are definitions, examples, issue notes, tables, clipping-axis rules, corner shaping, clip-margin and scrollable-area calculation. The captured document date does not guarantee old and newly retrieved bytes are identical; this exact capture is the active geometry evidence.

Conversion: repository-owned HTML preparation and Pandoc 3.1.11.1 semantic filter; scripts/styles omitted without execution. Original IDs, prose, links, literal/code text, variables and complete simple/complex table content are checked against the bounded input. Figures remain linked to source assets. No equations, source issues or normative wording are corrected.

Selection: current normative evidence for #205/#206 geometry reconciliation. Historical authored acceptance under the older 2025 source is not rewritten. The still-open transform-projection and optional-margin questions remain visible here; explicit Surgeist policies belong to their owning issue rather than this source capture.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Overflow Module Level 3

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 13 August 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-overflow-3/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-overflow-3/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2021/WD-css-overflow-3-20211223/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-overflow-3)

[Inline In Spec](https://drafts.csswg.org/css-overflow-3/#issues-index)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Florian Rivoal](http://florian.rivoal.net/) (On behalf of Bloomberg)

<strong>Former Editor:</strong>

[L. David Baron](https://dbaron.org/) ([Google](https://www.google.com/))

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-overflow-3/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="overflow-concepts"></a>2.  Overflow Concepts and Terminology[](#overflow-concepts)

CSS uses the term <a id="overflow"></a><strong>overflow</strong> to describe the contents of a box that extend outside one of that box’s edges (i.e., its <em>content edge</em>, <em>padding edge</em>, <em>border edge</em>, or <em>margin edge</em>). The term might be interpreted as elements or features that cause this overflow, the non-rectangular region occupied by these features, or, more commonly, as the minimal rectangle that bounds that region. A box’s overflow is computed based on the layout and styling of the box itself and of all descendants whose <a id="ref-for-containing-block-chain"></a>[containing block chain](https://drafts.csswg.org/css-display-4/#containing-block-chain) includes the box.

In most cases, <a id="ref-for-overflow"></a>[overflow](#overflow) can be computed for any box from the bounds and properties of that box itself, plus the <a id="ref-for-overflow①"></a>overflow of each of its children. However, this is not always the case; for example, when <a id="ref-for-propdef-transform-style"></a>[transform-style: preserve-3d](https://drafts.csswg.org/css-transforms-2/#propdef-transform-style) [\[CSS3-TRANSFORMS\]](https://drafts.csswg.org/css-overflow-3/#biblio-css3-transforms) is used on some of the children, any of their descendants with <a id="ref-for-propdef-transform-style①"></a>transform-style: preserve-3d must also be examined.

There are two different types of overflow, which are used for different purposes by the UA:

- <a id="ref-for-ink-overflow"></a>[ink overflow](#ink-overflow)
- <a id="ref-for-scrollable-overflow"></a>[scrollable overflow](#scrollable-overflow)

### <a id="ink"></a>2.1. Ink Overflow[](#ink)

The <a id="ink-overflow"></a><strong>ink overflow</strong> of a box is the part of that box and its contents that creates a visual effect outside of the box’s border box. Ink overflow is the overflow of painting effects defined to not affect layout or otherwise extend the <a id="ref-for-scrollable-overflow-region"></a>[scrollable overflow area](#scrollable-overflow-region), such as [box shadows](https://www.w3.org/TR/css-backgrounds/#box-shadow), [border images](), [text decoration](https://www.w3.org/TR/css-text-decor-3/), overhanging glyphs (with negative side bearings, or with ascenders/descenders extending outside the em box), [outlines](https://www.w3.org/TR/css-ui-3/#outline-props), etc.

Since some effects in CSS (for example, the blurs in <a id="ref-for-propdef-text-shadow"></a>[text-shadow](https://drafts.csswg.org/css-text-decor-4/#propdef-text-shadow) [\[CSS-TEXT-3\]](https://drafts.csswg.org/css-overflow-3/#biblio-css-text-3) and <a id="ref-for-propdef-box-shadow"></a>[box-shadow](https://drafts.csswg.org/css-backgrounds-3/#propdef-box-shadow) [\[CSS-BACKGROUNDS-3\]](https://drafts.csswg.org/css-overflow-3/#biblio-css-backgrounds-3), which are theoretically infinite; or text rendering, where shaping and rasterization can vary by platform) do not precisely define what visual extent they cover, the extent of the <a id="ref-for-ink-overflow①"></a>[ink overflow](#ink-overflow) is undefined, and may vary across UAs.

The <a id="ink-overflow-region"></a><strong>ink overflow area</strong> is the non-rectangular area occupied by the <a id="ref-for-ink-overflow②"></a>[ink overflow](#ink-overflow) of a box and its contents, and the <a id="ink-overflow-rectangle"></a><strong>ink overflow rectangle</strong> is the minimal rectangle whose axes are aligned to the box’s axes and that contains the <a id="ref-for-ink-overflow-region"></a>[ink overflow area](#ink-overflow-region). Note that the <a id="ref-for-ink-overflow-rectangle"></a>[ink overflow rectangle](#ink-overflow-rectangle) is a rectangle in the box’s coordinate system, but might be non-rectangular in other coordinate systems due to transforms. [\[CSS3-TRANSFORMS\]](https://drafts.csswg.org/css-overflow-3/#biblio-css3-transforms)

Any overflow of <a id="ref-for-replaced-element"></a>[replaced](https://drafts.csswg.org/css-display-4/#replaced-element) content is always <a id="ref-for-ink-overflow③"></a>[ink overflow](#ink-overflow) (as opposed to <a id="ref-for-scrollable-overflow①"></a>[scrollable overflow](#scrollable-overflow)).

### <a id="scrollable"></a>2.2.  Scrollable Overflow[](#scrollable)

The <a id="scrollable-overflow"></a><strong>scrollable overflow</strong> of a box is the set of things extending outside of that box’s padding edge for which a scrolling mechanism needs to be provided.

The <a id="scrollable-overflow-region"></a><strong>scrollable overflow area</strong> is the non-rectangular region occupied by the <a id="ref-for-scrollable-overflow②"></a>[scrollable overflow](#scrollable-overflow), and the <a id="scrollable-overflow-rectangle"></a><strong>scrollable overflow rectangle</strong> is the minimal rectangle whose axes are aligned to the box’s axes and that contains the <a id="ref-for-scrollable-overflow-region①"></a>[scrollable overflow area](#scrollable-overflow-region). See [§ 3.3 Calculating the Scrollable Overflow Area](#scrollable-overflow-calculation) for details.

### <a id="scrolling"></a>2.3.  Scrolling Overflow[](#scrolling)

A box’s <a id="ref-for-overflow②"></a>[overflow](#overflow) can be visible or clipped. CSS also allows a box to be a <a id="scroll-container"></a><strong>scroll container</strong> that allows clipped parts of its <a id="ref-for-scrollable-overflow-region②"></a>[scrollable overflow area](#scrollable-overflow-region) to be scrolled into view. The visual “viewport” of a <a id="ref-for-scroll-container"></a>[scroll container](#scroll-container) (through which the <a id="ref-for-scrollable-overflow-region③"></a>scrollable overflow area can be viewed) coincides with its padding box, and is called the <a id="scrollport"></a><strong>scrollport</strong>. A box’s <a id="nearest-scroll-container"></a><strong>nearest scroll container</strong> is the nearest <a id="ref-for-scroll-container①"></a>scroll container ancestor in the <a id="ref-for-containing-block-chain①"></a>[containing block chain](https://drafts.csswg.org/css-display-4/#containing-block-chain). (See [§ 3 Clipping and Scrolling Overflow](#overflow-properties) for control over whether a box clips or scrolls its overflow.)

Scrolling operations can be initiated by the user (for example, by manipulating a scrollbar, swiping a touchscreen, or using keyboard controls) or by script (for example, by the <code><a id="ref-for-dom-element-scrollintoview"></a>[scrollIntoView()](https://drafts.csswg.org/cssom-view-1/#dom-element-scrollintoview)</code> or <code><a id="ref-for-dom-window-focus"></a>[focus()](https://html.spec.whatwg.org/multipage/interaction.html#dom-window-focus)</code> APIs). The initial position of the <a id="ref-for-scrollable-overflow-rectangle"></a>[scrollable overflow rectangle](#scrollable-overflow-rectangle) within the <a id="ref-for-scrollport"></a>[scrollport](#scrollport) before any scrolling operations take effect is the <a id="initial-scroll-position"></a><strong>initial scroll position</strong>. The <a id="ref-for-initial-scroll-position"></a>[initial scroll position](#initial-scroll-position) is typically dependent on the <a id="ref-for-scroll-container②"></a>[scroll container](#scroll-container)’s <a id="ref-for-writing-mode"></a>[writing mode](https://drafts.csswg.org/css-writing-modes-4/#writing-mode), and, unless otherwise specified, coincides with its <a id="ref-for-scroll-origin-position"></a>[scroll origin position](#scroll-origin-position). For example, <a id="ref-for-propdef-scroll-initial-target"></a>[scroll-initial-target](https://drafts.csswg.org/css-scroll-snap-2/#propdef-scroll-initial-target) property can be used to change the <a id="ref-for-initial-scroll-position①"></a>initial scroll position. [\[CSS-SCROLL-SNAP-2\]](https://drafts.csswg.org/css-overflow-3/#biblio-css-scroll-snap-2)

A <a id="scroll-position"></a><strong>scroll position</strong> is a particular alignment of the <a id="ref-for-scrollable-overflow-rectangle①"></a>[scrollable overflow rectangle](#scrollable-overflow-rectangle) within its <a id="ref-for-scrollport①"></a>[scrollport](#scrollport). It is associated with a <a id="scroll-offset"></a><strong>scroll offset</strong> which is its distance from the <a id="ref-for-scroll-origin"></a>[scroll origin](#scroll-origin).

The <a id="scroll-origin"></a><strong>scroll origin</strong> is the anchor coordinate of the <a id="ref-for-scrollable-overflow-rectangle②"></a>[scrollable overflow rectangle](#scrollable-overflow-rectangle), from which the <a id="ref-for-scrollable-overflow-rectangle③"></a>scrollable overflow rectangle expands. Unless otherwise specified, it is the <a id="ref-for-block-start"></a>[block-start](https://drafts.csswg.org/css-writing-modes-4/#block-start) <a id="ref-for-inline-start"></a>[inline-start](https://drafts.csswg.org/css-writing-modes-4/#inline-start) corner of the <a id="ref-for-scrollable-overflow-rectangle④"></a>scrollable overflow rectangle. (For example, in a <a id="ref-for-flex-container"></a>[flex container](https://drafts.csswg.org/css-flexbox-2/#flex-container) it is the <a id="ref-for-main-start"></a>[main-start](https://drafts.csswg.org/css-flexbox-2/#main-start) <a id="ref-for-cross-start"></a>[cross-start](https://drafts.csswg.org/css-flexbox-2/#cross-start) corner.) A <a id="ref-for-scroll-container③"></a>[scroll container](#scroll-container) is said to be scrolled to its <a id="ref-for-scroll-origin①"></a>[scroll origin](#scroll-origin) when its <a id="ref-for-scroll-origin②"></a>scroll origin coincides with the corresponding corner of its <a id="ref-for-scrollport②"></a>[scrollport](#scrollport). This <a id="ref-for-scroll-position"></a>[scroll position](#scroll-position), the <a id="scroll-origin-position"></a><strong>scroll origin position</strong>, usually, but not always, coincides with the <a id="ref-for-initial-scroll-position②"></a>[initial scroll position](#initial-scroll-position).

<a id="example-f3202600"></a>

<strong>Example:</strong>

[](#example-f3202600) For example, <a id="ref-for-scroll-snap"></a>[scroll snapping](https://drafts.csswg.org/css-scroll-snap-1/#scroll-snap) [\[CSS-SCROLL-SNAP-1\]](https://drafts.csswg.org/css-overflow-3/#biblio-css-scroll-snap-1) can change the <a id="ref-for-initial-scroll-position③"></a>[initial scroll position](#initial-scroll-position) away from the <a id="ref-for-scroll-origin-position①"></a>[scroll origin position](#scroll-origin-position).

<a id="issue-ba58968a"></a>

<strong>Issue:</strong>

[](#issue-ba58968a) Check whether things like <a id="ref-for-baseline-alignment"></a>[baseline alignment](https://drafts.csswg.org/css-align-3/#baseline-alignment) depend on the <a id="ref-for-initial-scroll-position④"></a>[initial scroll position](#initial-scroll-position) or the <a id="ref-for-scroll-origin-position②"></a>[scroll origin position](#scroll-origin-position).

<a id="issue-223d3747"></a>

<strong>Issue:</strong>

[](#issue-223d3747) This doesn’t define a coordinate system for <a id="ref-for-scroll-offset"></a>[scroll offsets](#scroll-offset). Whether they increase downward/rightward, block/inline-axis endward, or away from the <a id="ref-for-scroll-origin③"></a>[scroll origin](#scroll-origin) is not defined. Should each API define its coordinate model?

Unless otherwise adjusted (e.g. [by content alignment](https://drafts.csswg.org/css-align-3/#overflow-scroll-position) [\[css-align-3\]](https://drafts.csswg.org/css-overflow-3/#biblio-css-align-3)), the area beyond the <a id="ref-for-scroll-origin④"></a>[scroll origin](#scroll-origin) in either axis is considered the <a id="unreachable-scrollable-overflow-region"></a><strong>unreachable scrollable overflow region</strong>: content rendered here is not accessible to the reader, see [§ 2.2 Scrollable Overflow](#scrollable).

The root viewport, which scrolls the page <a id="ref-for-TermCanvas"></a>[canvas](https://w3c.github.io/svgwg/svg2-draft/coords.html#TermCanvas), uses the <a id="ref-for-principal-writing-mode"></a>[principal writing mode](https://drafts.csswg.org/css-writing-modes-4/#principal-writing-mode) for determining its <a id="ref-for-scroll-origin⑤"></a>[scroll origin](#scroll-origin) and <a id="ref-for-initial-scroll-position⑤"></a>[initial scroll position](#initial-scroll-position).

Note: In the case where a <a id="ref-for-scroll-container④"></a>[scroll container](#scroll-container) (or one of its ancestors) is the target of a graphical transform, the UA might need to take this transform into account when mapping user inputs to scrolling operations. For instance, on a touch screen where the user scrolls by directly dragging the content, the transform would be expected to be taken into account to match the direction of scrolling to the gesture. On the other hand, other user inputs (such as the Page Down key, or a 1D scroll wheel) might be more naturally interpreted ignoring the transform. Choosing the appropriate behavior for each scrolling mechanism is the responsibility of the UA.

## <a id="overflow-properties"></a>3.  Clipping and Scrolling Overflow[](#overflow-properties)

The properties in this chapter specify whether and where a box’s <a id="ref-for-overflow③"></a>[overflow](#overflow) is clipped; if so, whether it is a <a id="ref-for-scroll-container⑤"></a>[scroll container](#scroll-container); and if so, in which axis(es) it is allowed to scroll (its <a id="scrollable-axis"></a><strong>scrollable axis(es)</strong>), thus which of the following types of <a id="ref-for-scroll-container⑥"></a>scroll container it is:

<strong><a id="single-axis-scroll-container"></a><strong>single-axis scroll container</strong></strong>

A <a id="ref-for-scroll-container⑦"></a>[scroll container](#scroll-container) that is scrollable in only one axis.

<strong><a id="dual-axis-scroll-container"></a><strong>dual-axis scroll container</strong></strong>

A <a id="ref-for-scroll-container⑧"></a>[scroll container](#scroll-container) that is scrollable in both axes.

<strong><a id="block-axis-scroll-container"></a><strong>block-axis scroll container</strong></strong>

<strong><a id="inline-axis-scroll-container"></a><strong>inline-axis scroll container</strong></strong>

<strong><a id="x-axis-scroll-container"></a><strong>x-axis scroll container</strong></strong>

<strong><a id="y-axis-scroll-container"></a><strong>y-axis scroll container</strong></strong>

A <a id="ref-for-scroll-container⑨"></a>[scroll container](#scroll-container) that is scrollable in the specified axis, regardless of whether it can also scroll in the other axis.

### <a id="overflow-control"></a>3.1.  Managing Overflow: the <a id="ref-for-propdef-overflow-x①"></a>[overflow-x](#propdef-overflow-x), <a id="ref-for-propdef-overflow-y①"></a>[overflow-y](#propdef-overflow-y), and <a id="ref-for-propdef-overflow⑤"></a>[overflow](#propdef-overflow) properties[](#overflow-control)

| Field                                                                                                                            | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|----------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                                                           | <a id="propdef-overflow-x"></a><strong>overflow-x</strong>, <a id="propdef-overflow-y"></a><strong>overflow-y</strong>, <a id="propdef-overflow-block"></a><strong>overflow-block</strong>, <a id="propdef-overflow-inline"></a><strong>overflow-inline</strong>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                                                          | visible <a id="ref-for-comb-one"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) hidden <a id="ref-for-comb-one①"></a>\| clip <a id="ref-for-comb-one②"></a>\| scroll <a id="ref-for-comb-one③"></a>\| auto                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>                                                   | <a id="ref-for-valdef-overflow-visible"></a>[visible](#valdef-overflow-visible)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>                                                    | <a id="ref-for-block-container"></a>[block containers](https://drafts.csswg.org/css-display-4/#block-container) [\[CSS2\]](https://drafts.csswg.org/css-overflow-3/#biblio-css2), <a id="ref-for-flex-container①"></a>[flex containers](https://drafts.csswg.org/css-flexbox-2/#flex-container) [\[CSS-FLEXBOX-1\]](https://drafts.csswg.org/css-overflow-3/#biblio-css-flexbox-1), <a id="ref-for-grid-container"></a>[grid containers](https://drafts.csswg.org/css-grid-2/#grid-container) [\[CSS-GRID-1\]](https://drafts.csswg.org/css-overflow-3/#biblio-css-grid-1), and <a id="ref-for-table-grid-box①"></a>[table grid boxes](https://drafts.csswg.org/css-tables-3/#table-grid-box) [\[CSS-TABLES-3\]](https://drafts.csswg.org/css-overflow-3/#biblio-css-tables-3) |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>                                             | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>                                                   | N/A                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>                                                  | usually specified value, but see text                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong>                                         | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong>                                         | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a id="1cb7bf200"></a>[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)</strong> | <a id="ref-for-propdef-overflow⑥"></a>[overflow](#propdef-overflow)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |

The <a id="ref-for-propdef-overflow-x②"></a>[overflow-x](#propdef-overflow-x) property specifies the handling of <a id="ref-for-overflow④"></a>[overflow](#overflow) in the horizontal axis (i.e., overflow from the left and right sides of the box), and the <a id="ref-for-propdef-overflow-y②"></a>[overflow-y](#propdef-overflow-y) property specifies the handling of <a id="ref-for-overflow⑤"></a>overflow in the vertical axis (i.e., overflow from the top and bottom sides of the box).

The <a id="ref-for-propdef-overflow-block"></a>[overflow-block](#propdef-overflow-block) and <a id="ref-for-propdef-overflow-inline"></a>[overflow-inline](#propdef-overflow-inline) properties likewise specify the handling of <a id="ref-for-overflow⑥"></a>[overflow](#overflow) in the <a id="ref-for-block-axis"></a>[block](https://drafts.csswg.org/css-writing-modes-4/#block-axis) and <a id="ref-for-inline-axis"></a>[inline](https://drafts.csswg.org/css-writing-modes-4/#inline-axis) axis, respectively

These four properties form a <a id="ref-for-logical-property-group"></a>[logical property group](https://drafts.csswg.org/css-logical-1/#logical-property-group) together with the <a id="ref-for-propdef-overflow⑦"></a>[overflow](#propdef-overflow) <a id="ref-for-shorthand-property"></a>[shorthand](https://drafts.csswg.org/css-cascade-5/#shorthand-property), and interact as defined in [CSS Logical Properties 1 § 4 Flow-Relative Box Model Properties](https://drafts.csswg.org/css-logical-1/#box).

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                        |
|------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-overflow"></a><strong>overflow</strong>                                                                                                                                                                                                                                            |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | <a id="ref-for-propdef-overflow-block①"></a>[\<'overflow-block'\>](#propdef-overflow-block)<a id="ref-for-mult-num-range"></a>[{1,2}](https://drafts.csswg.org/css-values-4/#mult-num-range)                                                                                                      |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | visible                                                                                                                                                                                                                                                                                           |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | block containers [\[CSS2\]](https://drafts.csswg.org/css-overflow-3/#biblio-css2), flex containers [\[CSS3-FLEXBOX\]](https://drafts.csswg.org/css-overflow-3/#biblio-css3-flexbox), and grid containers [\[CSS3-GRID-LAYOUT\]](https://drafts.csswg.org/css-overflow-3/#biblio-css3-grid-layout) |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                                                                |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | N/A                                                                                                                                                                                                                                                                                               |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | see individual properties                                                                                                                                                                                                                                                                         |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                                                                                                                                                                                                          |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                       |

The <a id="ref-for-propdef-overflow⑧"></a>[overflow](#propdef-overflow) property is a <a id="ref-for-shorthand-property①"></a>[shorthand property](https://drafts.csswg.org/css-cascade-5/#shorthand-property) that sets the specified values of <a id="ref-for-propdef-overflow-x③"></a>[overflow-x](#propdef-overflow-x) and <a id="ref-for-propdef-overflow-y③"></a>[overflow-y](#propdef-overflow-y) in that order. If the second value is omitted, it is copied from the first.

Values have the following meanings:

<strong><a id="valdef-overflow-visible"></a><strong>visible</strong></strong>

There is no special handling of overflow, that is, the box’s content is rendered outside the box if positioned there.

<strong><a id="valdef-overflow-hidden"></a><strong>hidden</strong></strong>

This value indicates that the box’s content is clipped to its <a id="ref-for-overflow-clip-edge"></a>[overflow clip edge](#overflow-clip-edge) and that the UA must not provide any scrolling user interface to view the content outside the clipping region, nor allow scrolling by direct intervention of the user, such as dragging on a touch screen or using the scrolling wheel on a mouse. However, the content must still be scrollable programmatically, for example using the mechanisms defined in [\[CSSOM-VIEW\]](https://drafts.csswg.org/css-overflow-3/#biblio-cssom-view).

<strong><a id="valdef-overflow-clip"></a><strong>clip</strong></strong>

This value indicates that the box’s content is clipped to its <a id="ref-for-overflow-clip-edge①"></a>[overflow clip edge](#overflow-clip-edge) and that no scrolling user interface should be provided by the UA to view the content outside the clipping region. In addition, unlike <a id="ref-for-propdef-overflow⑨"></a>[overflow: hidden](#propdef-overflow) which still allows programmatic scrolling, <a id="ref-for-propdef-overflow①⓪"></a>overflow: clip forbids scrolling entirely, through any mechanism.

Unlike <a id="ref-for-valdef-overflow-hidden"></a>[hidden](#valdef-overflow-hidden), this value <strong>does not</strong> cause the element to establish a new formatting context.

Note: Authors who also want the box to establish a formatting context can use <a id="ref-for-propdef-display"></a>[display: flow-root](https://drafts.csswg.org/css-display-3/#propdef-display) together with <a id="ref-for-propdef-overflow①①"></a>[overflow: clip](#propdef-overflow).

<strong><a id="valdef-overflow-scroll"></a><strong>scroll</strong></strong>

This value indicates that the content is clipped to the <a id="ref-for-overflow-clip-edge②"></a>[overflow clip edge](#overflow-clip-edge), but can be scrolled into view.

Furthermore, if the user agent uses a scrolling mechanism that is visible on the screen (such as a scroll bar or a panner), that mechanism should be displayed whether or not any of its content is clipped. This avoids any problem with scrollbars appearing and disappearing in a dynamic environment. When the target medium is print, overflowing content may be printed; it is not defined where it may be printed.

<strong><a id="valdef-overflow-auto"></a><strong>auto</strong></strong>

Like <a id="ref-for-valdef-overflow-scroll"></a>[scroll](#valdef-overflow-scroll) when the box has <a id="ref-for-scrollable-overflow③"></a>[scrollable overflow](#scrollable-overflow); like <a id="ref-for-valdef-overflow-hidden①"></a>[hidden](#valdef-overflow-hidden) otherwise. Thus, if the user agent uses a scrolling mechanism that is visible on the screen (such as a scroll bar or a panner), that mechanism will only be displayed if there is overflow.

The <a id="ref-for-valdef-overflow-scroll①"></a>[scroll](#valdef-overflow-scroll), <a id="ref-for-valdef-overflow-auto"></a>[auto](#valdef-overflow-auto), and <a id="ref-for-valdef-overflow-hidden②"></a>[hidden](#valdef-overflow-hidden) values are known as the <a id="scrollable-overflow-value"></a><strong>scrollable values</strong> of <a id="ref-for-propdef-overflow①②"></a>[overflow](#propdef-overflow). They cause the box to be a <a id="ref-for-scroll-container①⓪"></a>[scroll container](#scroll-container) and the affected axis to be a <a id="ref-for-scrollable-axis"></a>[scrollable axis](#scrollable-axis). A <a id="ref-for-block-box"></a>[block box](https://drafts.csswg.org/css-display-4/#block-box) that becomes a <a id="ref-for-scroll-container①①"></a>scroll container also establishes an <a id="ref-for-independent-formatting-context"></a>[independent formatting context](https://drafts.csswg.org/css-display-4/#independent-formatting-context).

The <a id="ref-for-valdef-overflow-visible①"></a>[visible](#valdef-overflow-visible) and <a id="ref-for-valdef-overflow-clip①"></a>[clip](#valdef-overflow-clip) values are known as the <a id="non-scrollable-overflow-value"></a><strong>non-scrollable values</strong>. However, if the other axis specifies a <a id="ref-for-scrollable-overflow-value"></a>[scrollable value](#scrollable-overflow-value), a specified value of <a id="ref-for-valdef-overflow-visible②"></a>visible <a id="ref-for-computed-value"></a>[computes](https://drafts.csswg.org/css-cascade-5/#computed-value) to <a id="ref-for-valdef-overflow-auto①"></a>[auto](#valdef-overflow-auto), enabling scrolling in its axis. If neither axis computes to a <a id="ref-for-scrollable-overflow-value①"></a>scrollable value, the box is not a <a id="ref-for-scroll-container①②"></a>[scroll container](#scroll-container). If only one axis computes to a <a id="ref-for-scrollable-overflow-value②"></a>scrollable value (i.e. the other axis is <a id="ref-for-valdef-overflow-clip②"></a>clip), the box is a <a id="ref-for-single-axis-scroll-container"></a>[single-axis scroll container](#single-axis-scroll-container).

User agents must also support the <a id="valdef-overflow-overlay"></a><strong>overlay</strong> keyword as a <a id="ref-for-css-legacy-value-alias"></a>[legacy value alias](https://drafts.csswg.org/css-cascade-5/#css-legacy-value-alias) of <a id="ref-for-valdef-overflow-auto②"></a>[auto](#valdef-overflow-auto).

Note: The <a id="ref-for-propdef-overflow①③"></a>[overflow](#propdef-overflow) properties are expanded to apply to <a id="ref-for-replaced-element①"></a>[replaced elements](https://drafts.csswg.org/css-display-4/#replaced-element) in [Level 4](http://www.w3.org/TR/css-overflow-4/).

#### <a id="scroll-visibility"></a>3.1.1.  Interaction of <a id="ref-for-propdef-visibility"></a>[visibility](https://drafts.csswg.org/css-display-4/#propdef-visibility) and <a id="ref-for-propdef-overflow①④"></a>[overflow](#propdef-overflow)[](#scroll-visibility)

If the computed value of the <a id="ref-for-propdef-visibility①"></a>[visibility](https://drafts.csswg.org/css-display-4/#propdef-visibility) property is <a id="ref-for-valdef-visibility-hidden"></a>[hidden](https://drafts.csswg.org/css-display-4/#valdef-visibility-hidden) (or <a id="ref-for-valdef-visibility-collapse"></a>[collapse](https://drafts.csswg.org/css-display-4/#valdef-visibility-collapse) when it has the same effect as <a id="ref-for-valdef-visibility-hidden①"></a>hidden), and <a id="ref-for-propdef-overflow①⑤"></a>[overflow](#propdef-overflow) is either <a id="ref-for-valdef-overflow-scroll②"></a>[scroll](#valdef-overflow-scroll) or <a id="ref-for-valdef-overflow-auto③"></a>[auto](#valdef-overflow-auto), then:

- The user agent must not make any scrolling mechanism visible. To the extent that the scrolling mechanism that would normally be visible in the absence of <a id="ref-for-propdef-visibility②"></a>[visibility: hidden](https://drafts.csswg.org/css-display-4/#propdef-visibility) affects layout, it continues to do so, but is not painted.

- As would be the case with <a id="ref-for-propdef-overflow①⑥"></a>[overflow: hidden](#propdef-overflow), scrolling directly triggered by user interactions is disabled, but programmatic scrolling continues to take effect.

- The lack of interactive direct scrolling is enforced even if the user interacts (e.g. with a mouse scrolling wheel) with a descendent of the <a id="ref-for-propdef-visibility③"></a>[visibility: hidden](https://drafts.csswg.org/css-display-4/#propdef-visibility) <a id="ref-for-scroll-container①③"></a>[scroll container](#scroll-container) that is itself set to <a id="ref-for-propdef-visibility④"></a>visibility: visible.

#### <a id="corner-clipping"></a>3.1.2.  Interaction of <a id="ref-for-propdef-border-radius"></a>[border-radius](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-radius) and <a id="ref-for-propdef-overflow①⑦"></a>[overflow](#propdef-overflow)[](#corner-clipping)

As mentioned in [CSS Backgrounds 3 § 4.3 Corner Clipping](https://drafts.csswg.org/css-backgrounds-3/#corner-clipping), the clipping region established by <a id="ref-for-propdef-overflow①⑧"></a>[overflow](#propdef-overflow) can be rounded:

- When <a id="ref-for-propdef-overflow-x④"></a>[overflow-x](#propdef-overflow-x) and <a id="ref-for-propdef-overflow-y④"></a>[overflow-y](#propdef-overflow-y) compute to <a id="ref-for-valdef-overflow-hidden③"></a>[hidden](#valdef-overflow-hidden), <a id="ref-for-valdef-overflow-scroll③"></a>[scroll](#valdef-overflow-scroll), or <a id="ref-for-valdef-overflow-auto④"></a>[auto](#valdef-overflow-auto), the clipping region is rounded based on the border radius, adjusted to the <a id="ref-for-padding-edge"></a>[padding edge](https://drafts.csswg.org/css-box-4/#padding-edge), as described in [CSS Backgrounds 3 § 4.2 Corner Shaping](https://drafts.csswg.org/css-backgrounds-3/#corner-shaping).

- When both <a id="ref-for-propdef-overflow-x⑤"></a>[overflow-x](#propdef-overflow-x) and <a id="ref-for-propdef-overflow-y⑤"></a>[overflow-y](#propdef-overflow-y) compute to <a id="ref-for-valdef-overflow-clip③"></a>[clip](#valdef-overflow-clip), the clipping region is rounded as described in [§ 3.2 Expanding Clipping Bounds: the overflow-clip-margin property](#overflow-clip-margin).

- However, when one of <a id="ref-for-propdef-overflow-x⑥"></a>[overflow-x](#propdef-overflow-x) or <a id="ref-for-propdef-overflow-y⑥"></a>[overflow-y](#propdef-overflow-y) computes to <a id="ref-for-valdef-overflow-clip④"></a>[clip](#valdef-overflow-clip) and the other computes to <a id="ref-for-valdef-overflow-visible③"></a>[visible](#valdef-overflow-visible), the clipping region is not rounded.

#### <a id="static-media"></a>3.1.3.  Overflow in Print and Other Static Media[](#static-media)

Since scrolling is not possible in static media (such as print) authors should be careful to make content accessible in such media, for example by using

<strong>Advisement:</strong>

&#64;media print, (update: none) { … } to adjust layout such that all relevant content is simultaneously visible.

On <a id="ref-for-scroll-container①④"></a>[scroll containers](#scroll-container) in non-interactive media with an <a id="ref-for-propdef-overflow①⑨"></a>[overflow](#propdef-overflow) value of <a id="ref-for-valdef-overflow-auto⑤"></a>[auto](#valdef-overflow-auto) or <a id="ref-for-valdef-overflow-scroll④"></a>[scroll](#valdef-overflow-scroll) (but not <a id="ref-for-valdef-overflow-hidden④"></a>[hidden](#valdef-overflow-hidden)) UAs may display an indication of any scrollable overflow, such as by displaying scrollbars or an ellipsis.

Note: Not all <a id="ref-for-paged-media"></a>[paged media](https://drafts.csswg.org/mediaqueries-5/#paged-media) is non-interactive: for example, e-book readers paginate content, but are interactive.

#### <a id="overflow-propagation"></a>3.1.4.  Overflow Viewport Propagation[](#overflow-propagation)

UAs must apply the <a id="ref-for-propdef-overflow②⓪"></a>[overflow](#propdef-overflow) values set on the root element to the <a id="ref-for-dfn-viewport"></a>[viewport](https://w3c.github.io/epub-specs/epub33/core/#dfn-viewport) when the root element’s <a id="ref-for-propdef-display①"></a>[display](https://drafts.csswg.org/css-display-3/#propdef-display) value is not <a id="ref-for-valdef-display-none"></a>[none](https://drafts.csswg.org/css-display-4/#valdef-display-none). However, when the root element is an [\[HTML\]](https://drafts.csswg.org/css-overflow-3/#biblio-html) <code><a id="ref-for-the-html-element"></a>[html](https://html.spec.whatwg.org/multipage/semantics.html#the-html-element)</code> element (including [XML syntax for HTML](https://html.spec.whatwg.org/multipage/introduction.html#html-vs-xhtml)) whose <a id="ref-for-propdef-overflow②①"></a>overflow value is <a id="ref-for-valdef-overflow-visible④"></a>[visible](#valdef-overflow-visible) (in both axes), and that element has as a child a <code><a id="ref-for-the-body-element"></a>[body](https://html.spec.whatwg.org/multipage/sections.html#the-body-element)</code> element whose <a id="ref-for-propdef-display②"></a>display value is also not <a id="ref-for-valdef-display-none①"></a>none, user agents must instead apply the <a id="ref-for-propdef-overflow②②"></a>overflow values of the first such child element to the viewport. The element from which the value is propagated must then have a used <a id="ref-for-propdef-overflow②③"></a>overflow value of <a id="ref-for-valdef-overflow-visible⑤"></a>visible.

Note: Using <a id="ref-for-containment"></a>[containment](https://drafts.csswg.org/css-contain-2/#containment) on the HTML <code><a id="ref-for-the-html-element①"></a>[html](https://html.spec.whatwg.org/multipage/semantics.html#the-html-element)</code> or <code><a id="ref-for-the-body-element①"></a>[body](https://html.spec.whatwg.org/multipage/sections.html#the-body-element)</code> elements disables this special handling of the HTML <code><a id="ref-for-the-body-element②"></a>[body](https://html.spec.whatwg.org/multipage/sections.html#the-body-element)</code> element. See the [CSS Containment 1 § 2 Strong Containment: the contain property](https://drafts.csswg.org/css-contain-1/#contain-property) for details.

Note: <a id="ref-for-propdef-overflow②④"></a>[overflow: hidden](#propdef-overflow) on the root element might not clip everything outside the <a id="ref-for-initial-containing-block"></a>[Initial Containing Block](https://drafts.csswg.org/css-display-4/#initial-containing-block) if the ICB is smaller than the viewport, which can happen on mobile.

If <a id="ref-for-valdef-overflow-visible⑥"></a>[visible](#valdef-overflow-visible) is applied to the viewport, it must be interpreted as <a id="ref-for-valdef-overflow-auto⑥"></a>[auto](#valdef-overflow-auto). If <a id="ref-for-valdef-overflow-clip⑤"></a>[clip](#valdef-overflow-clip) is applied to the viewport, it must be interpreted as <a id="ref-for-valdef-overflow-hidden⑤"></a>[hidden](#valdef-overflow-hidden).

### <a id="overflow-clip-margin"></a>3.2.  Expanding Clipping Bounds: the <a id="ref-for-propdef-overflow-clip-margin③"></a>[overflow-clip-margin](#propdef-overflow-clip-margin) property[](#overflow-clip-margin)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                              |
|------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-overflow-clip-margin"></a><strong>overflow-clip-margin</strong>                                                                                                                                                                                                                          |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | <a id="ref-for-typedef-visual-box"></a>[\<visual-box\>](https://drafts.csswg.org/css-box-4/#typedef-visual-box) <a id="ref-for-comb-any"></a>[\|\|](https://drafts.csswg.org/css-values-4/#comb-any) <a id="ref-for-length-value"></a>[\<length\>](https://drafts.csswg.org/css-values-4/#length-value) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | 0px                                                                                                                                                                                                                                                                                                     |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | boxes to which <a id="ref-for-propdef-overflow②⑤"></a>[overflow](#propdef-overflow) applies                                                                                                                                                                                                             |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                                                                      |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                                                                     |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | the computed <a id="ref-for-length-value①"></a>[\<length\>](https://drafts.csswg.org/css-values-4/#length-value) and a <a id="ref-for-typedef-visual-box①"></a>[\<visual-box\>](https://drafts.csswg.org/css-box-4/#typedef-visual-box) keyword                                                         |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                             |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | per computed value if the <a id="ref-for-typedef-visual-box②"></a>[\<visual-box\>](https://drafts.csswg.org/css-box-4/#typedef-visual-box) values match; otherwise discrete                                                                                                                             |

This property defines the <a id="overflow-clip-edge"></a><strong>overflow clip edge</strong> of the box, i.e. precisely where the box’s content is allowed to paint before being clipped by effects (such as <a id="ref-for-propdef-overflow②⑥"></a>[overflow: clip](#propdef-overflow), above) that are defined to clip to the box’s <a id="ref-for-overflow-clip-edge③"></a>[overflow clip edge](#overflow-clip-edge).

Values are defined as follows:

<strong><a id="valdef-overflow-clip-margin-visual-box"></a><strong><a id="ref-for-typedef-visual-box③"></a>[\<visual-box\>](https://drafts.csswg.org/css-box-4/#typedef-visual-box)</strong></strong>

Specifies the box edge to use as the <a id="ref-for-overflow-clip-edge④"></a>[overflow clip edge](#overflow-clip-edge) origin, i.e. when the specified offset is zero.

If omitted, defaults to padding-box.

<strong><a id="valdef-overflow-clip-margin-length"></a><strong><a id="ref-for-length-value②"></a>[\<length\>](https://drafts.csswg.org/css-values-4/#length-value)</strong></strong>

The specified offset dictates how much the <a id="ref-for-overflow-clip-edge⑤"></a>[overflow clip edge](#overflow-clip-edge) is expanded from the specified box edge. Negative values indicate insets, instead. Defaults to zero if omitted.

The <a id="ref-for-overflow-clip-edge⑥"></a>[overflow clip edge](#overflow-clip-edge) is shaped in the corners exactly the same way as an <a id="ref-for-box-shadow-outer-box-shadow"></a>[outer box-shadow](https://drafts.csswg.org/css-backgrounds-3/#box-shadow-outer-box-shadow) with a spread radius of the same cumulative offset from the box’s <a id="ref-for-border-edge"></a>[border edge](https://drafts.csswg.org/css-box-4/#border-edge). See [CSS Backgrounds 3 § 4.2 Corner Shaping](https://drafts.csswg.org/css-backgrounds-3/#corner-shaping) and [CSS Backgrounds 3 § 6.1.1 Shadow Shape, Spread, and Knockout](https://drafts.csswg.org/css-backgrounds-3/#shadow-shape), noting in particular the formula for outsets beyond the <a id="ref-for-border-edge①"></a>border edge.

If the box is a <a id="ref-for-scroll-container①⑤"></a>[scroll container](#scroll-container):

- the <a id="ref-for-overflow-clip-edge⑦"></a>[overflow clip edge](#overflow-clip-edge) is clamped to stay within the element’s <a id="ref-for-padding-box"></a>[padding box](https://drafts.csswg.org/css-box-4/#padding-box). (This does not affect the <a id="ref-for-computed-value①"></a>[computed](https://drafts.csswg.org/css-cascade-5/#computed-value) or <a id="ref-for-used-value"></a>[used value](https://drafts.csswg.org/css-cascade-5/#used-value) of this property.)

- the border-box value ignores any specified offset (and, due to the previous bullet point, effectively acts as padding-box)

Note: This property was previously defined to only affect <a id="ref-for-propdef-overflow②⑦"></a>[overflow: clip](#propdef-overflow). It now also affects <a id="ref-for-scroll-container①⑥"></a>[scroll containers](#scroll-container), but only to shrink the clipping edge.

### <a id="scrollable-overflow-calculation"></a>3.3.  Calculating the Scrollable Overflow Area[](#scrollable-overflow-calculation)

The <a id="ref-for-scrollable-overflow-region④"></a>[scrollable overflow area](#scrollable-overflow-region) of a box is the union of:

- Its own <a id="ref-for-padding-box①"></a>[padding box](https://drafts.csswg.org/css-box-4/#padding-box).

- All <a id="ref-for-line-box"></a>[line boxes](https://drafts.csswg.org/css-inline-3/#line-box) it directly contains.

- The border boxes of all boxes for which it is the containing block and whose border boxes are positioned not wholly within its <a id="ref-for-unreachable-scrollable-overflow-region"></a>[unreachable scrollable overflow region](#unreachable-scrollable-overflow-region) (if any), accounting for transforms by projecting each box onto the plane of the element that establishes its <a id="ref-for-3d-rendering-context"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context). [\[CSS3-TRANSFORMS\]](https://drafts.csswg.org/css-overflow-3/#biblio-css3-transforms)

  <a id="issue-df7ef6c3"></a>

  <strong>Issue:</strong>

  [](#issue-df7ef6c3) Is this description of handling transforms sufficiently accurate?

  Border boxes with zero area do not affect the <a id="ref-for-scrollable-overflow-region⑤"></a>[scrollable overflow area](#scrollable-overflow-region).

- The margin areas of <a id="ref-for-grid-item"></a>[grid item](https://drafts.csswg.org/css-grid-2/#grid-item) and <a id="ref-for-flex-item"></a>[flex item](https://drafts.csswg.org/css-flexbox-2/#flex-item) boxes for which the box establishes a containing block.

  The UA may <em>additionally</em> include the margin areas of other boxes for which the box establishes a containing block; however, the conditions under which such margin areas are included is undefined in this level. <a id="issue-b9c7269c"></a>

  <strong>Issue:</strong>

  [](#issue-b9c7269c)This needs further testing and investigation; is therefore deferred in this draft.

- The <a id="ref-for-scrollable-overflow-rectangle⑤"></a>[scrollable overflow rectangles](#scrollable-overflow-rectangle) of all of the above boxes (including zero-area boxes), clipped to their <a id="ref-for-overflow-clip-edge⑧"></a>[overflow clip edge](#overflow-clip-edge) if <a id="ref-for-propdef-overflow②⑧"></a>[overflow](#propdef-overflow) is not <a id="ref-for-valdef-overflow-visible⑦"></a>[visible](#valdef-overflow-visible) or <a id="ref-for-propdef-contain"></a>[contain](https://drafts.csswg.org/css-contain-2/#propdef-contain) applies <a id="ref-for-valdef-contain-paint"></a>[paint](https://drafts.csswg.org/css-contain-2/#valdef-contain-paint), and accounting for transforms as described above.

  Note: The <a id="ref-for-propdef-clip"></a>[clip](https://drafts.csswg.org/css-masking-1/#propdef-clip), <a id="ref-for-propdef-clip-path"></a>[clip-path](https://drafts.csswg.org/css-masking-1/#propdef-clip-path), and mask-\* properties [\[CSS-MASKING-1\]](https://drafts.csswg.org/css-overflow-3/#biblio-css-masking-1) do not affect the <a id="ref-for-scrollable-overflow-region⑥"></a>[scrollable overflow area](#scrollable-overflow-region). Only effects that clip to the <a id="ref-for-overflow-clip-edge⑨"></a>[overflow clip edge](#overflow-clip-edge) are taken into account.

  Note: The <a id="ref-for-scrollable-overflow-rectangle⑥"></a>[scrollable overflow rectangle](#scrollable-overflow-rectangle) is always a rectangle in its box’s own coordinate system, but might be non-rectangular in ancestor coordinate systems due to transforms [\[CSS3-TRANSFORMS\]](https://drafts.csswg.org/css-overflow-3/#biblio-css3-transforms). This means scrollbars can sometimes appear when not actually necessary.

- Additional padding added to the <a id="ref-for-scrollable-overflow-rectangle⑦"></a>[scrollable overflow rectangle](#scrollable-overflow-rectangle) as necessary to enable scroll positions that satisfy the requirements of both <a id="ref-for-propdef-place-content"></a>[place-content: start](https://drafts.csswg.org/css-align-3/#propdef-place-content) and <a id="ref-for-propdef-place-content①"></a>place-content: end alignment.

  Note: This padding represents, within the <a id="ref-for-scrollable-overflow-rectangle⑧"></a>[scrollable overflow rectangle](#scrollable-overflow-rectangle), the box’s own padding so that when its content is scrolled to its end, there is padding between the edge of its <a id="ref-for-in-flow"></a>[in-flow](https://drafts.csswg.org/css-display-4/#in-flow) (or floated) content and the border edge of the box. It typically ends up being exactly the same size as the box’s own padding, except in a few cases—​such as when an <a id="ref-for-out-of-flow"></a>[out-of-flow](https://drafts.csswg.org/css-display-4/#out-of-flow) positioned element, or the visible overflow of a descendent, has already increased the size of the <a id="ref-for-scrollable-overflow-rectangle⑨"></a>scrollable overflow rectangle outside the conceptual “content edge” of the <a id="ref-for-scroll-container①⑦"></a>[scroll container](#scroll-container)’s content.

  <figure>
  <img src="https://drafts.csswg.org/css-overflow-3/images/scroll-align-padding.jpg" />
  <figcaption>Issue: Replace this image with a proper SVG.</figcaption>
  </figure>

Additionally, due to Web-compatibility constraints (caused by authors exploiting legacy bugs to surreptitiously hide content from visual readers but not search engines and/or speech output), UAs must clip any content in the <a id="ref-for-unreachable-scrollable-overflow-region①"></a>[unreachable scrollable overflow region](#unreachable-scrollable-overflow-region).

Note: The <a id="ref-for-content-distribution-properties"></a>[content-distribution properties](https://drafts.csswg.org/css-align-3/#content-distribution-properties) can [alter the unreachable scrollable overflow region](https://drafts.csswg.org/css-align-3/#overflow-scroll-position) to ensure that a <a id="ref-for-scroll-container①⑧"></a>[scroll container](#scroll-container)’s <a id="ref-for-alignment-subject"></a>[alignment subject](https://drafts.csswg.org/css-align-3/#alignment-subject) is reachable after alignment. [\[css-align-3\]](https://drafts.csswg.org/css-overflow-3/#biblio-css-align-3)
