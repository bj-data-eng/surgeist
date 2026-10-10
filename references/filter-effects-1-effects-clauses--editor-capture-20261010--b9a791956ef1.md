Attribution and reformatting notice added for Surgeist on 2026-10-10

This bounded, reformatted source excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [Filter Effects Module Level 1](https://drafts.csswg.org/filter-effects-1/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and excerpt boundary

Retrieved: 2026-10-10. Source status: Editor’s Draft, 22 September 2026.

Full captured HTML SHA-256: `2ead3581d431bf3663167ada7e410793e0e561812b772081bc286da489826e29` (1963398 bytes).

Bounded conversion input SHA-256: `b9a791956ef15ba3e2d7e9fa7be5a29d7df4c152072b78758cef9057ea070ff9` (125518 bytes). Page revision metadata: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`.

Retained complete sections, bounded at the next equal-or-higher-level heading: `placement`, `FilterProperty`, `FilterPrimitivesOverviewIntro`, `feColorMatrixElement`, `feDisplacementMapElement`, `ColorInterpolationFiltersProperty`, `huerotateEquivalent`, `tainted-filter-primitives`, `fedisplacemnentmap-restrictions`. The source header and full legal notice are also retained. Other clauses remain available through upstream links; this is explicitly a partial source capture.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table content are checked against the bounded input. Figures remain links to upstream source images. Synthetic table headers and span expansion are representation changes.

Mathematical representation: the six selected Filter Effects MathML expressions are replaced with their exact source-provided StarMath 5.0 annotations in literal blocks before generic HTML preparation, preserving matrix row/column separators, signs and exact constants. The raw MathML and annotation comparison remain in the conversion provenance. No equation is corrected or harmonized; the conflicting `0.213` matrix and `0.2127` expanded expression remain visible. CSS Transforms formulas retain their source text and upstream formula-image links.

Selection status: evidence for resolving Surgeist #356/#563 and their concrete consumers. The current editor text still leaves the recorded questions/conflict open. Any WebKit behavior selected by Surgeist is an explicit compatibility policy, not normative consensus.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>Filter Effects Module Level 1

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 22 September 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/filter-effects-1/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/filter-effects-1/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2018/WD-filter-effects-1-20181124/>

<https://www.w3.org/TR/2014/WD-filter-effects-1-20141125/>

<https://www.w3.org/TR/2013/WD-filter-effects-1-20131126/>

<https://www.w3.org/TR/2013/WD-filter-effects-20130523/>

<https://www.w3.org/TR/2012/WD-filter-effects-20121025/>

<strong>Test Suite:</strong>

<https://wpt.fyi/css/filter-effects>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/filter-effects-1)

[Inline In Spec](https://drafts.csswg.org/filter-effects-1/#issues-index)

<strong>Editors:</strong>

[Dirk Schulze](mailto:dschulze@adobe.com) (Adobe Inc.)

[Chris Harrelson](mailto:chrishtr@google.com) (Google)

<strong>Former Editors:</strong>

[Dean Jackson](mailto:dino@apple.com) (Apple Inc.)

Vincent Hardy

[Erik Dahlström](mailto:erik@dahlstr%C3%B6m.net) (Invited Expert)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/filter-effects-1/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="placement"></a>2. Module interactions[](#placement)

This specification defines a set of CSS properties that affect the visual rendering of elements to which those properties are applied; these effects are applied after elements have been sized and positioned according to the [Visual formatting model](https://www.w3.org/TR/CSS2/visuren.html) from [\[CSS21\]](https://drafts.csswg.org/filter-effects-1/#biblio-css21). Some values of these properties result in the creation of a <a id="ref-for-containing-block"></a>[containing block](https://drafts.csswg.org/css-display-4/#containing-block), and/or the creation of a <a id="ref-for-x43"></a>[stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43).

The compositing model follows the SVG compositing model [\[SVG11\]](https://drafts.csswg.org/filter-effects-1/#biblio-svg11): first any filter effect is applied, then any clipping, masking and opacity [\[CSS3COLOR\]](https://drafts.csswg.org/filter-effects-1/#biblio-css3color). These effects all apply after any other CSS effects such as <a id="ref-for-propdef-border"></a>[border](https://drafts.csswg.org/css-backgrounds-3/#propdef-border) [\[CSS3BG\]](https://drafts.csswg.org/filter-effects-1/#biblio-css3bg).

Some property and element definitions in this specification require an SVG 1.1 implementation [\[SVG11\]](https://drafts.csswg.org/filter-effects-1/#biblio-svg11). UAs without support for SVG must not implement the <a id="ref-for-propdef-color-interpolation-filters"></a>[color-interpolation-filters](#propdef-color-interpolation-filters), <a id="ref-for-propdef-flood-color"></a>[flood-color](https://drafts.csswg.org/filter-effects-1/#propdef-flood-color), <a id="ref-for-propdef-flood-opacity"></a>[flood-opacity](https://drafts.csswg.org/filter-effects-1/#propdef-flood-opacity) and <a id="ref-for-propdef-lighting-color"></a>[lighting-color](https://drafts.csswg.org/filter-effects-1/#propdef-lighting-color) properties as well as the <a id="ref-for-elementdef-filter"></a>[filter](https://drafts.csswg.org/filter-effects-1/#elementdef-filter) element, the <a id="ref-for-elementdef-femergenode"></a>[feMergeNode](https://drafts.csswg.org/filter-effects-1/#elementdef-femergenode) element, the <a id="ref-for-transfer-function-element"></a>[transfer function elements](https://drafts.csswg.org/filter-effects-1/#transfer-function-element) and the <a id="ref-for-filter-primitive"></a>[filter primitive](https://drafts.csswg.org/filter-effects-1/#filter-primitive) elements.

## <a id="FilterProperty"></a>5. Graphic filters: the <a id="ref-for-propdef-filter①"></a>[filter](#propdef-filter) property[](#FilterProperty)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
|------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-filter"></a><strong>filter</strong>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | none <a id="ref-for-comb-one"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) <a id="ref-for-typedef-filter-value-list"></a>[\<filter-value-list\>](#typedef-filter-value-list)                                                                                                                                                                                                                                                                                                                                                                               |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | none                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | All elements. In SVG, it applies to <a id="ref-for-container-element"></a>[container elements](https://w3c.github.io/svgwg/svg2-draft/struct.html#container-element) without the <a id="ref-for-elementdef-defs"></a>[defs](https://w3c.github.io/svgwg/svg2-draft/struct.html#elementdef-defs) element, all <a id="ref-for-graphics-element"></a>[graphics elements](https://w3c.github.io/svgwg/svg2-draft/struct.html#graphics-element) and the <a id="ref-for-elementdef-use"></a>[use](https://w3c.github.io/svgwg/svg2-draft/struct.html#elementdef-use) element. |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | as specified                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | See prose in [Animation of Filters](https://drafts.csswg.org/filter-effects-1/#animation-of-filters).                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong>Media:</strong>                                                                  | visual                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |

The <a id="ref-for-propdef-filter②"></a>[filter](#propdef-filter) property applies a filter, specified either by an SVG reference or by a <a id="ref-for-filter-function"></a>[filter function](https://drafts.csswg.org/filter-effects-1/#filter-function), to an element, altering how it paints.

<a id="typedef-filter-value-list"></a><a id="ref-for-typedef-filter-function"></a><a id="ref-for-comb-one①"></a><a id="ref-for-typedef-filter-url"></a><a id="ref-for-mult-one-plus"></a>

``` text
<filter-value-list> = [ <filter-function> | <url> ]+
```

<strong><a id="typedef-filter-url"></a><strong>\<url\></strong></strong>

A filter reference to a <a id="ref-for-elementdef-filter②"></a>[filter](https://drafts.csswg.org/filter-effects-1/#elementdef-filter) element. For example url(commonfilters.svg#filter). If the filter references a non-existent object or the referenced object is not a <a id="ref-for-elementdef-filter③"></a>filter element, then the whole filter chain is ignored. No filter is applied to the object.

<strong><a id="ref-for-typedef-filter-function①"></a>[\<filter-function\>](https://drafts.csswg.org/filter-effects-1/#typedef-filter-function)</strong>

See <a id="ref-for-filter-function①"></a>[filter functions](https://drafts.csswg.org/filter-effects-1/#filter-function).

<strong>none</strong>

No filter effect gets applied.

A value other than none for the <a id="ref-for-propdef-filter③"></a>[filter](#propdef-filter) property results in the creation of a <a id="ref-for-containing-block①"></a>[containing block](https://drafts.csswg.org/css-display-4/#containing-block) for absolute and fixed positioned descendants unless the element it applies to is a document root element in the current <a id="ref-for-browsing-context"></a>[browsing context](https://html.spec.whatwg.org/multipage/document-sequences.html#browsing-context). The list of functions are applied in the order provided.

The first filter function or <a id="ref-for-elementdef-filter④"></a>[filter](https://drafts.csswg.org/filter-effects-1/#elementdef-filter) reference in the list takes the element (<a id="ref-for-attr-valuedef-in-sourcegraphic"></a>[SourceGraphic](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-sourcegraphic)) as the input image. Subsequent operations take the output from the previous filter function or <a id="ref-for-elementdef-filter⑤"></a>filter reference as the input image. <a id="ref-for-elementdef-filter⑥"></a>filter element reference functions can specify an alternate input, but still uses the previous output as its <a id="ref-for-attr-valuedef-in-sourcegraphic①"></a>SourceGraphic.

<a id="ref-for-propdef-color-interpolation-filters①"></a>[color-interpolation-filters](#propdef-color-interpolation-filters) has no affect for Filter Functions. Filter Functions must operate in the sRGB color space.

A computed value of other than none results in the creation of a [stacking context](https://www.w3.org/TR/CSS21/zindex.html) [\[CSS21\]](https://drafts.csswg.org/filter-effects-1/#biblio-css21) the same way that CSS <a id="ref-for-propdef-opacity"></a>[opacity](https://drafts.csswg.org/css-color-4/#propdef-opacity) does. All the elements descendants are rendered together as a group with the filter effect applied to the group as a whole.

The <a id="ref-for-propdef-filter④"></a>[filter](#propdef-filter) property has no effect on the geometry of the target element’s CSS boxes, although <a id="ref-for-propdef-filter⑤"></a>filter can contribute to the element’s <a id="ref-for-ink-overflow-region"></a>[ink overflow area](https://drafts.csswg.org/css-overflow-3/#ink-overflow-region).

Conceptually, any parts of the drawing are effected by filter operations. This includes any content, background, borders, text decoration, outline and visible scrolling mechanism of the element to which the filter is applied, and those of its descendants. The filter operations are applied in the element’s <a id="ref-for-local-coordinate-system"></a>[local coordinate system](https://drafts.csswg.org/css-transforms-1/#local-coordinate-system).

The compositing model follows the [SVG compositing model](https://www.w3.org/TR/SVG11/render.html#Introduction) [\[SVG11\]](https://drafts.csswg.org/filter-effects-1/#biblio-svg11): first any filter effect is applied, then any clipping, masking and opacity. As per SVG, the application of <a id="ref-for-propdef-filter⑥"></a>[filter](#propdef-filter) has no effect on hit-testing.

The <a id="ref-for-propdef-filter⑦"></a>[filter](#propdef-filter) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

<a id="issue-95bcf4be"></a>

<strong>Issue:</strong>

[](#issue-95bcf4be) How does filter behave on fixed background images? [\[Issue \#238\]](https://github.com/w3c/csswg-drafts/issues/238)

### <a id="FilterPrimitivesOverviewIntro"></a>9.1. Overview[](#FilterPrimitivesOverviewIntro)

This section describes the various filter primitives that can be assembled to achieve a particular filter effect.

Unless otherwise stated, all image filters operate on premultiplied RGBA samples. Some filters like <a id="ref-for-elementdef-fecolormatrix②"></a>[feColorMatrix](#elementdef-fecolormatrix) and <a id="ref-for-elementdef-fecomponenttransfer②"></a>[feComponentTransfer](https://drafts.csswg.org/filter-effects-1/#elementdef-fecomponenttransfer) work more naturally on non-premultiplied data. For the time of the filter operation, all color values must temporarily be transformed to the required color multiplication of the current filter.

Note: All input images are assumed to be in premultiplied RGBA. User agents may optimize performance by using non-premultiplied data buffering.

All raster effect filtering operations take 1 to N input RGBA images, additional attributes as parameters, and produce a single output RGBA image.

The RGBA result from each filter primitive will be clamped into the allowable ranges for colors and opacity values. Thus, for example, the result from a given <a id="ref-for-filter-primitive④"></a>[filter primitive](https://drafts.csswg.org/filter-effects-1/#filter-primitive) will have any negative color values or opacity values adjusted up to color/opacity of zero.

<a id="filtersColorSpace"></a>The color space in which a particular <a id="ref-for-filter-primitive⑤"></a>[filter primitive](https://drafts.csswg.org/filter-effects-1/#filter-primitive) performs its operations is determined by the value of the property <a id="ref-for-propdef-color-interpolation-filters③"></a>[color-interpolation-filters](#propdef-color-interpolation-filters) on the given <a id="ref-for-filter-primitive⑥"></a>filter primitive. A different property, <a id="ref-for-ColorInterpolationProperty①"></a>[color-interpolation](https://w3c.github.io/svgwg/svg2-draft/painting.html#ColorInterpolationProperty) determines the color space for other color operations. Because these two properties have different initial values (<a id="ref-for-propdef-color-interpolation-filters④"></a>color-interpolation-filters has an initial value of <a id="ref-for-valdef-color-interpolation-filters-linearrgb"></a>[linearRGB](#valdef-color-interpolation-filters-linearrgb) whereas <a id="ref-for-ColorInterpolationProperty②"></a>color-interpolation has an initial value of <a id="ref-for-valdef-color-interpolation-filters-srgb"></a>[sRGB](#valdef-color-interpolation-filters-srgb)), in some cases to achieve certain results (e.g., when coordinating gradient interpolation with a filtering operation) it will be necessary to explicitly set <a id="ref-for-ColorInterpolationProperty③"></a>color-interpolation to <a id="ref-for-valdef-color-interpolation-filters-linearrgb①"></a>linearRGB or <a id="ref-for-propdef-color-interpolation-filters⑤"></a>color-interpolation-filters to <a id="ref-for-valdef-color-interpolation-filters-srgb①"></a>sRGB on particular elements. Note that the examples below do not explicitly set either <a id="ref-for-ColorInterpolationProperty④"></a>color-interpolation or <a id="ref-for-propdef-color-interpolation-filters⑥"></a>color-interpolation-filters, so the initial values for these properties apply to the examples.

Sometimes <a id="ref-for-filter-primitive⑦"></a>[filter primitives](https://drafts.csswg.org/filter-effects-1/#filter-primitive) result in undefined pixels. For example, filter primitive <a id="ref-for-elementdef-feoffset③"></a>[feOffset](https://drafts.csswg.org/filter-effects-1/#elementdef-feoffset) can shift an image down and to the right, leaving undefined pixels at the top and left. In these cases, the undefined pixels are set to transparent black.

To provide high quality rendering, all filter primitives should operate in a device dependent coordinate space, the <a id="operating-coordinate-space"></a><strong>operating coordinate space</strong>, taking device pixel density, user space transformations and zooming into account. To provide a platform independent alignment, attribute and property values are often relative to a coordinate system described by the <a id="ref-for-element-attrdef-filter-primitiveunits⑤"></a>[primitiveUnits](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitiveunits) attribute. User agents must scale these relative attributes and properties to the <a id="ref-for-operating-coordinate-space"></a>[operating coordinate space](#operating-coordinate-space).

Note: On high resolution devices, attribute and property values that are relative to the <a id="ref-for-element-attrdef-filter-primitiveunits⑥"></a>[primitiveUnits](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitiveunits) usually need to be scaled up. User agents may reduce the resolution of filter primitives on limited platform resources.

Note: Some attribute or property values from the filter primitives <a id="ref-for-elementdef-feconvolvematrix②"></a>[feConvolveMatrix](https://drafts.csswg.org/filter-effects-1/#elementdef-feconvolvematrix) and <a id="ref-for-light-source"></a>[light sources](https://drafts.csswg.org/filter-effects-1/#light-source) can not be mapped from the coordinate space defined by the <a id="ref-for-element-attrdef-filter-primitiveunits⑦"></a>[primitiveUnits](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitiveunits) attribute to the <a id="ref-for-operating-coordinate-space①"></a>[operating coordinate space](#operating-coordinate-space).

### <a id="feColorMatrixElement"></a>9.6. Filter primitive <a id="ref-for-elementdef-fecolormatrix⑥"></a>[feColorMatrix](#elementdef-fecolormatrix)[](#feColorMatrixElement)

<table>
<colgroup>
<col style="width: 50%" />
<col style="width: 50%" />
</colgroup>
<thead>
<tr class="header">
<th>Field</th>
<th>Definition</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<th><strong>Name:</strong></th>
<td><a id="elementdef-fecolormatrix"></a><strong><code>feColorMatrix</code></strong></td>
</tr>
<tr class="even">
<th><strong>Categories:</strong></th>
<td><a id="ref-for-filter-primitive②⑦"></a><a href="https://drafts.csswg.org/filter-effects-1/#filter-primitive">filter primitive</a></td>
</tr>
<tr class="odd">
<th><strong>Content model:</strong></th>
<td>Any number of <a id="ref-for-TermDescriptiveElement①"></a><a href="https://w3c.github.io/svgwg/svg2-draft/struct.html#TermDescriptiveElement">descriptive elements</a>, <a id="ref-for-elementdef-animate②"></a><a href="https://svgwg.org/specs/animations/#elementdef-animate">animate</a>, <a id="ref-for-elementdef-script②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/interact.html#elementdef-script">script</a>, <a id="ref-for-elementdef-set②"></a><a href="https://svgwg.org/specs/animations/#elementdef-set">set</a> elements, in any order.</td>
</tr>
<tr class="even">
<th><strong>Attributes:</strong></th>
<td><ul>
<li><a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes">core attributes</a> — <a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute">id</a>, <a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute">xml:base</a>, <a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute">xml:lang</a>, <a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute">xml:space</a></li>
<li><a href="http://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute">presentation attributes</a> — <a id="ref-for-propdef-alignment-baseline②"></a><a href="https://drafts.csswg.org/css-inline-3/#propdef-alignment-baseline">alignment-baseline</a>, <a id="ref-for-propdef-baseline-shift②"></a><a href="https://drafts.csswg.org/css-inline-3/#propdef-baseline-shift">baseline-shift</a>, <a id="ref-for-propdef-clip②"></a><a href="https://drafts.csswg.org/css-masking-1/#propdef-clip">clip</a>, <a id="ref-for-propdef-clip-path②"></a><a href="https://drafts.csswg.org/css-masking-1/#propdef-clip-path">clip-path</a>, <a id="ref-for-propdef-clip-rule②"></a><a href="https://drafts.csswg.org/css-masking-1/#propdef-clip-rule">clip-rule</a>, <a id="ref-for-propdef-color②"></a><a href="https://drafts.csswg.org/css-color-4/#propdef-color">color</a>, <a id="ref-for-ColorInterpolationProperty⑥"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#ColorInterpolationProperty">color-interpolation</a>, <a id="ref-for-propdef-color-interpolation-filters⑧"></a><a href="#propdef-color-interpolation-filters">color-interpolation-filters</a>, <u>color-rendering</u>, <a id="ref-for-propdef-cursor②"></a><a href="https://drafts.csswg.org/css-ui-4/#propdef-cursor">cursor</a>, <a id="ref-for-propdef-direction②"></a><a href="https://drafts.csswg.org/css-writing-modes-3/#propdef-direction">direction</a>, <a id="ref-for-propdef-display⑤"></a><a href="https://drafts.csswg.org/css-display-3/#propdef-display">display</a>, <a id="ref-for-propdef-dominant-baseline②"></a><a href="https://drafts.csswg.org/css-inline-3/#propdef-dominant-baseline">dominant-baseline</a>, enable-background, <a id="ref-for-FillProperty④"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#FillProperty">fill</a>, <a id="ref-for-FillOpacityProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#FillOpacityProperty">fill-opacity</a>, <a id="ref-for-FillRuleProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#FillRuleProperty">fill-rule</a>, <a id="ref-for-propdef-filter①③"></a><a href="#propdef-filter">filter</a>, <a id="ref-for-propdef-flood-color③"></a><a href="https://drafts.csswg.org/filter-effects-1/#propdef-flood-color">flood-color</a>, <a id="ref-for-propdef-flood-opacity④"></a><a href="https://drafts.csswg.org/filter-effects-1/#propdef-flood-opacity">flood-opacity</a>, <a id="ref-for-propdef-font②"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font">font</a>, <a id="ref-for-propdef-font-family②"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-family">font-family</a>, <a id="ref-for-propdef-font-size②"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-size">font-size</a>, <a id="ref-for-propdef-font-size-adjust②"></a><a href="https://drafts.csswg.org/css-fonts-5/#propdef-font-size-adjust">font-size-adjust</a>, <a id="ref-for-propdef-font-stretch②"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-stretch">font-stretch</a>, <a id="ref-for-propdef-font-style②"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-style">font-style</a>, <a id="ref-for-propdef-font-variant②"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-variant">font-variant</a>, <a id="ref-for-propdef-font-weight②"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-weight">font-weight</a>, <a id="ref-for-propdef-glyph-orientation-vertical②"></a><a href="https://drafts.csswg.org/css-writing-modes-4/#propdef-glyph-orientation-vertical">glyph-orientation-vertical</a>, <a id="ref-for-propdef-image-rendering②"></a><a href="https://drafts.csswg.org/css-images-3/#propdef-image-rendering">image-rendering</a>, <a id="ref-for-propdef-isolation④"></a><a href="https://drafts.csswg.org/compositing-2/#propdef-isolation">isolation</a>, kerning, <a id="ref-for-propdef-letter-spacing②"></a><a href="https://drafts.csswg.org/css-text-4/#propdef-letter-spacing">letter-spacing</a>, <a id="ref-for-propdef-lighting-color③"></a><a href="https://drafts.csswg.org/filter-effects-1/#propdef-lighting-color">lighting-color</a>, <a id="ref-for-MarkerProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerProperty">marker</a>, <a id="ref-for-MarkerEndProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerEndProperty">marker-end</a>, <a id="ref-for-MarkerMidProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerMidProperty">marker-mid</a>, <a id="ref-for-MarkerStartProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerStartProperty">marker-start</a>, <a id="ref-for-propdef-mask②"></a><a href="https://drafts.csswg.org/css-masking-1/#propdef-mask">mask</a>, <a id="ref-for-propdef-opacity⑤"></a><a href="https://drafts.csswg.org/css-color-4/#propdef-opacity">opacity</a>, <a id="ref-for-propdef-overflow②"></a><a href="https://drafts.csswg.org/css-overflow-3/#propdef-overflow">overflow</a>, <a id="ref-for-propdef-pointer-events②"></a><a href="https://drafts.csswg.org/css-ui-4/#propdef-pointer-events">pointer-events</a>, <a id="ref-for-ShapeRenderingProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#ShapeRenderingProperty">shape-rendering</a>, <a id="ref-for-StopColorProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/pservers.html#StopColorProperty">stop-color</a>, <a id="ref-for-StopOpacityProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/pservers.html#StopOpacityProperty">stop-opacity</a>, <a id="ref-for-StrokeProperty③"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeProperty">stroke</a>, <a id="ref-for-StrokeDasharrayProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeDasharrayProperty">stroke-dasharray</a>, <a id="ref-for-StrokeDashoffsetProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeDashoffsetProperty">stroke-dashoffset</a>, <a id="ref-for-StrokeLinecapProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeLinecapProperty">stroke-linecap</a>, <a id="ref-for-StrokeLinejoinProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeLinejoinProperty">stroke-linejoin</a>, <a id="ref-for-StrokeMiterlimitProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeMiterlimitProperty">stroke-miterlimit</a>, <a id="ref-for-StrokeOpacityProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeOpacityProperty">stroke-opacity</a>, <a id="ref-for-StrokeWidthProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeWidthProperty">stroke-width</a>, <a id="ref-for-TextAnchorProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/text.html#TextAnchorProperty">text-anchor</a>, <a id="ref-for-propdef-text-decoration②"></a><a href="https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration">text-decoration</a>, <a id="ref-for-TextRenderingProperty②"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#TextRenderingProperty">text-rendering</a>, <a id="ref-for-propdef-unicode-bidi②"></a><a href="https://drafts.csswg.org/css-writing-modes-3/#propdef-unicode-bidi">unicode-bidi</a>, <a id="ref-for-propdef-visibility②"></a><a href="https://drafts.csswg.org/css-display-4/#propdef-visibility">visibility</a>, <a id="ref-for-propdef-word-spacing②"></a><a href="https://drafts.csswg.org/css-text-4/#propdef-word-spacing">word-spacing</a>, <a id="ref-for-propdef-writing-mode②"></a><a href="https://drafts.csswg.org/css-writing-modes-4/#propdef-writing-mode">writing-mode</a></li>
<li><a id="ref-for-filter-primitive-attributes①"></a><a href="https://drafts.csswg.org/filter-effects-1/#filter-primitive-attributes">filter primitive attributes</a>  —​<a id="ref-for-element-attrdef-filter-primitive-x⑤"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-x">x</a>, <a id="ref-for-element-attrdef-filter-primitive-y⑤"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-y">y</a>, <a id="ref-for-element-attrdef-filter-primitive-width⑤"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-width">width</a>, <a id="ref-for-element-attrdef-filter-primitive-height⑤"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-height">height</a>, <a id="ref-for-element-attrdef-filter-primitive-result⑤"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-result">result</a> </li>
<li><a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute">class</a></li>
<li><a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute">style</a></li>
<li><a id="ref-for-element-attrdef-filter-primitive-in⑥"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-in">in</a></li>
<li><a id="ref-for-element-attrdef-fecolormatrix-type"></a><a href="#element-attrdef-fecolormatrix-type">type</a></li>
<li><a id="ref-for-element-attrdef-fecolormatrix-values"></a><a href="#element-attrdef-fecolormatrix-values">values</a></li>
</ul></td>
</tr>
<tr class="odd">
<th><strong>DOM Interfaces:</strong></th>
<td><a href="https://drafts.csswg.org/filter-effects-1/#InterfaceSVGFEColorMatrixElement">SVGFEColorMatrixElement</a></td>
</tr>
</tbody>
</table>

This filter applies a matrix transformation:

``` text
left [ stack {R' # G' # B' # A' # 1} right ] = left [ matrix {
a_00 # a_01 # a_02 # a_03 # a_04 ##
a_10 # a_11 # a_12 # a_13 # a_14 ##
a_20 # a_21 # a_22 # a_23 # a_24 ##
a_30 # a_31 # a_32 # a_33 # a_34 ##
0 # 0 # 0 # 0 # 1 } right ] cdot left[ stack { R # G # B # A # 1 } right]
```

on the RGBA color and alpha values of every pixel on the input graphics to produce a result with a new set of RGBA color and alpha values.

The calculations are performed on non-premultiplied color values.

<em>Attribute definitions:</em>

<strong><a id="element-attrdef-fecolormatrix-type"></a><strong><code>type</code></strong> = "<a id="attr-valuedef-type-matrix"></a><strong><code>matrix</code></strong> \| <a id="attr-valuedef-type-saturate"></a><strong><code>saturate</code></strong> \| <a id="attr-valuedef-type-huerotate"></a><strong><code>hueRotate</code></strong> \| <a id="attr-valuedef-type-luminancetoalpha"></a><strong><code>luminanceToAlpha</code></strong>"</strong>

Indicates the type of matrix operation. The keyword <a id="ref-for-attr-valuedef-type-matrix"></a>[matrix](#attr-valuedef-type-matrix) indicates that a full 5x4 matrix of values will be provided. The other keywords represent convenience shortcuts to allow commonly used color operations to be performed without specifying a complete matrix.

The <a id="ref-for-TermInitialValue②②"></a>[initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for <a id="ref-for-element-attrdef-fecolormatrix-type①"></a>[type](#element-attrdef-fecolormatrix-type) is <a id="ref-for-attr-valuedef-type-matrix①"></a>[matrix](#attr-valuedef-type-matrix).

Animatable: yes.

<strong><a id="element-attrdef-fecolormatrix-values"></a><strong><code>values</code></strong> = "<em>list of <a id="ref-for-number-value⑨"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)s</em>"</strong>

The contents of <a id="ref-for-element-attrdef-fecolormatrix-values①"></a>[values](#element-attrdef-fecolormatrix-values) depends on the value of attribute <a id="ref-for-element-attrdef-fecolormatrix-type②"></a>[type](#element-attrdef-fecolormatrix-type):

- For <code>type=&#34;matrix&#34;</code>, <a id="ref-for-element-attrdef-fecolormatrix-values②"></a>[values](#element-attrdef-fecolormatrix-values) is a list of 20 matrix values (a00 a01 a02 a03 a04 a10 a11 ... a34), separated by whitespace and/or a comma. For example, the identity matrix could be expressed as:

  ``` text
  type="matrix"
  values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 1 0"
  ```

- For <code>type=&#34;saturate&#34;</code>, <a id="ref-for-element-attrdef-fecolormatrix-values③"></a>[values](#element-attrdef-fecolormatrix-values) is a single real number value. A <a id="ref-for-attr-valuedef-type-saturate"></a>[saturate](#attr-valuedef-type-saturate) operation is equivalent to the following matrix operation:

  ``` text
  left [ stack {R' # G' # B' # A' # 1} right ] = left [ matrix {
  0.213 + 0.787s # 0.715 - 0.715s # 0.072 - 0.072s # 0 # 0 ##
  0.213 - 0.213s # 0.715 + 0.285s # 0.072 - 0.072s # 0 # 0 ##
  0.213 - 0.213s # 0.715 - 0.715s # 0.072 + 0.928s # 0 # 0 ##
  0 # 0 # 0 # 1 # 0 ##
  0 # 0 # 0 # 0 # 1 } right ] cdot left[ stack { R # G # B # A # 1 } right]
  ```

  Note: A value of 0 produces a fully desaturated (grayscale) filter result, while a value of 1 passes the filter input image through unchanged. Values outside the 0..1 range under- or oversaturates the filter input image respectively.

  Note: The precision of the luminance coefficients increased in comparison to previous specification texts [\[Cmam\]](https://drafts.csswg.org/filter-effects-1/#biblio-cmam).

- For <code>type=&#34;hueRotate&#34;</code>, <a id="ref-for-element-attrdef-fecolormatrix-values④"></a>[values](#element-attrdef-fecolormatrix-values) is a single one real number value (degrees). A <a id="ref-for-attr-valuedef-type-huerotate"></a>[hueRotate](#attr-valuedef-type-huerotate) operation is equivalent to the following matrix operation:

  ``` text
  left [ stack {R' # G' # B' # A' # 1} right ] = left [ matrix {
  a_00 # a_01 # a_02 # 0 # 0 ##
  a_10 # a_11 # a_12 # 0 # 0 ##
  a_20 # a_21 # a_22 # 0 # 0 ##
  0 # 0 # 0 # 1 # 0 ##
  0 # 0 # 0 # 0 # 1 } right ] cdot left[ stack { R # G # B # A # 1 } right]
  ```

  where the terms a00, a01, etc. are calculated as follows:

  ``` text
  left[ matrix { a_00 # a_01 # a_02 ## a_10 # a_11 # a_12 ## a_20 # a_21 # a_22 } right] = left[ matrix { +0.213 # +0.715 # +0.072 ## +0.213 # +0.715 # +0.072 ## +0.213 # +0.715 # +0.072 } right] + cos(hueRotate value)cdot left[ matrix { +0.787 # -0.715 # -0.072 ## -0.213 # +0.285 # -0.072 ## -0.213 # -0.715 # +0.928 } right] + sin(hueRotate value) cdot left[ matrix { -0.213 # -0.715 # +0.928 ## +0.143 # +0.140 # -0.283 ## -0.787 # +0.715 # +0.072 } right]
  ```

  Thus, the upper left term of the hue matrix turns out to be:

  ``` text
  a_00 = 0.2127 + cos(hueRotate value) cdot 0.7873 - sin(hueRotate value) cdot 0.2127
  ```

- For <code>type=&#34;luminanceToAlpha&#34;</code>, <a id="ref-for-element-attrdef-fecolormatrix-values⑤"></a>[values](#element-attrdef-fecolormatrix-values) is not applicable. A <a id="ref-for-attr-valuedef-type-luminancetoalpha"></a>[luminanceToAlpha](#attr-valuedef-type-luminancetoalpha) operation is equivalent to the following matrix operation:

  ``` text
  left [ stack {R' # G' # B' # A' # 1} right ] = left [ matrix {
  0 # 0 # 0 # 0 # 0 ##
  0 # 0 # 0 # 0 # 0 ##
  0 # 0 # 0 # 0 # 0 ##
  0.2126 # 0.7152 # 0.0722 # 0 # 0 ##
  0 # 0 # 0 # 0 # 1 } right ] cdot left[ stack { R # G # B # A # 1 } right]
  ```

The <a id="ref-for-TermInitialValue②③"></a>[initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for <a id="ref-for-element-attrdef-fecolormatrix-values⑥"></a>[values](#element-attrdef-fecolormatrix-values)

<strong>if <code>type=&#34;matrix&#34;</code></strong>

defaults to the identity matrix

<strong>if <code>type=&#34;saturate&#34;</code></strong>

defaults to the value 1

<strong>if <code>type=&#34;hueRotate&#34;</code></strong>

defaults to the value 0 which results in the identity matrix.

If the number of entries in the <a id="ref-for-element-attrdef-fecolormatrix-values⑦"></a>[values](#element-attrdef-fecolormatrix-values) list does not match the required number of entries by the <a id="ref-for-element-attrdef-fecolormatrix-type③"></a>[type](#element-attrdef-fecolormatrix-type), the filter primitive acts as a <a id="ref-for-pass-through-filter"></a>[pass through filter](https://drafts.csswg.org/filter-effects-1/#pass-through-filter).

Animatable: yes.

<a id="example-05772ccd"></a>

<strong>Example:</strong>

[](#example-05772ccd)

``` text
<svg width="8cm" height="5cm" viewBox="0 0 800 500"
     xmlns="http://www.w3.org/2000/svg">
  <title>Example feColorMatrix - Examples of feColorMatrix operations</title>
  <desc>Five text strings showing the effects of feColorMatrix:
        an unfiltered text string acting as a reference,
        use of the feColorMatrix matrix option to convert to grayscale,
        use of the feColorMatrix saturate option,
        use of the feColorMatrix hueRotate option,
        and use of the feColorMatrix luminanceToAlpha option.</desc>
  <defs>
    <linearGradient id="MyGradient" gradientUnits="userSpaceOnUse"
            x1="100" y1="0" x2="500" y2="0">
      <stop offset="0" stop-color="#ff00ff" />
      <stop offset=".33" stop-color="#88ff88" />
      <stop offset=".67" stop-color="#2020ff" />
      <stop offset="1" stop-color="#d00000" />
    </linearGradient>
    <filter id="Matrix" filterUnits="objectBoundingBox"
            x="0%" y="0%" width="100%" height="100%">
      <feColorMatrix type="matrix" in="SourceGraphic"
           values=".33 .33 .33 0 0
                   .33 .33 .33 0 0
                   .33 .33 .33 0 0
                   .33 .33 .33 0 0"/>
    </filter>
    <filter id="Saturate40" filterUnits="objectBoundingBox"
            x="0%" y="0%" width="100%" height="100%">
      <feColorMatrix type="saturate" in="SourceGraphic" values="0.4"/>
    </filter>
    <filter id="HueRotate90" filterUnits="objectBoundingBox"
            x="0%" y="0%" width="100%" height="100%">
      <feColorMatrix type="hueRotate" in="SourceGraphic" values="90"/>
    </filter>
    <filter id="LuminanceToAlpha" filterUnits="objectBoundingBox"
            x="0%" y="0%" width="100%" height="100%">
      <feColorMatrix type="luminanceToAlpha" in="SourceGraphic" result="a"/>
      <feComposite in="SourceGraphic" in2="a" operator="in" />
    </filter>
  </defs>
  <rect fill="none" stroke="blue"
        x="1" y="1" width="798" height="498"/>
  <g font-family="Verdana" font-size="75"
            font-weight="bold" fill="url(#MyGradient)" >
    <rect x="100" y="0" width="500" height="20" />
    <text x="100" y="90">Unfiltered</text>
    <text x="100" y="190" filter="url(#Matrix)" >Matrix</text>
    <text x="100" y="290" filter="url(#Saturate40)" >Saturate</text>
    <text x="100" y="390" filter="url(#HueRotate90)" >HueRotate</text>
    <text x="100" y="490" filter="url(#LuminanceToAlpha)" >Luminance</text>
  </g>
</svg>
```

![Example ](https://drafts.csswg.org/filter-effects-1/examples/feColorMatrix.png)

Example of feColorMatrix

[View this example as SVG](https://drafts.csswg.org/filter-effects-1/examples/feColorMatrix.svg)

### <a id="feDisplacementMapElement"></a>9.11. Filter primitive <a id="ref-for-elementdef-fedisplacementmap②"></a>[feDisplacementMap](#elementdef-fedisplacementmap)[](#feDisplacementMapElement)

<table>
<colgroup>
<col style="width: 50%" />
<col style="width: 50%" />
</colgroup>
<thead>
<tr class="header">
<th>Field</th>
<th>Definition</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<th><strong>Name:</strong></th>
<td><a id="elementdef-fedisplacementmap"></a><strong><code>feDisplacementMap</code></strong></td>
</tr>
<tr class="even">
<th><strong>Categories:</strong></th>
<td><a id="ref-for-filter-primitive③②"></a><a href="https://drafts.csswg.org/filter-effects-1/#filter-primitive">filter primitive</a></td>
</tr>
<tr class="odd">
<th><strong>Content model:</strong></th>
<td>Any number of <a id="ref-for-TermDescriptiveElement①⓪"></a><a href="https://w3c.github.io/svgwg/svg2-draft/struct.html#TermDescriptiveElement">descriptive elements</a>, <a id="ref-for-elementdef-animate⑨"></a><a href="https://svgwg.org/specs/animations/#elementdef-animate">animate</a>, <a id="ref-for-elementdef-script①①"></a><a href="https://w3c.github.io/svgwg/svg2-draft/interact.html#elementdef-script">script</a>, <a id="ref-for-elementdef-set⑨"></a><a href="https://svgwg.org/specs/animations/#elementdef-set">set</a> elements, in any order.</td>
</tr>
<tr class="even">
<th><strong>Attributes:</strong></th>
<td><ul>
<li><a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes">core attributes</a> — <a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute">id</a>, <a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute">xml:base</a>, <a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute">xml:lang</a>, <a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute">xml:space</a></li>
<li><a href="http://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute">presentation attributes</a> — <a id="ref-for-propdef-alignment-baseline⑦"></a><a href="https://drafts.csswg.org/css-inline-3/#propdef-alignment-baseline">alignment-baseline</a>, <a id="ref-for-propdef-baseline-shift⑦"></a><a href="https://drafts.csswg.org/css-inline-3/#propdef-baseline-shift">baseline-shift</a>, <a id="ref-for-propdef-clip⑦"></a><a href="https://drafts.csswg.org/css-masking-1/#propdef-clip">clip</a>, <a id="ref-for-propdef-clip-path⑦"></a><a href="https://drafts.csswg.org/css-masking-1/#propdef-clip-path">clip-path</a>, <a id="ref-for-propdef-clip-rule⑦"></a><a href="https://drafts.csswg.org/css-masking-1/#propdef-clip-rule">clip-rule</a>, <a id="ref-for-propdef-color⑦"></a><a href="https://drafts.csswg.org/css-color-4/#propdef-color">color</a>, <a id="ref-for-ColorInterpolationProperty①①"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#ColorInterpolationProperty">color-interpolation</a>, <a id="ref-for-propdef-color-interpolation-filters①③"></a><a href="#propdef-color-interpolation-filters">color-interpolation-filters</a>, <u>color-rendering</u>, <a id="ref-for-propdef-cursor⑦"></a><a href="https://drafts.csswg.org/css-ui-4/#propdef-cursor">cursor</a>, <a id="ref-for-propdef-direction⑦"></a><a href="https://drafts.csswg.org/css-writing-modes-3/#propdef-direction">direction</a>, <a id="ref-for-propdef-display①⓪"></a><a href="https://drafts.csswg.org/css-display-3/#propdef-display">display</a>, <a id="ref-for-propdef-dominant-baseline⑦"></a><a href="https://drafts.csswg.org/css-inline-3/#propdef-dominant-baseline">dominant-baseline</a>, enable-background, <a id="ref-for-FillProperty⑨"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#FillProperty">fill</a>, <a id="ref-for-FillOpacityProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#FillOpacityProperty">fill-opacity</a>, <a id="ref-for-FillRuleProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#FillRuleProperty">fill-rule</a>, <a id="ref-for-propdef-filter①⑧"></a><a href="#propdef-filter">filter</a>, <a id="ref-for-propdef-flood-color⑧"></a><a href="https://drafts.csswg.org/filter-effects-1/#propdef-flood-color">flood-color</a>, <a id="ref-for-propdef-flood-opacity⑨"></a><a href="https://drafts.csswg.org/filter-effects-1/#propdef-flood-opacity">flood-opacity</a>, <a id="ref-for-propdef-font⑦"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font">font</a>, <a id="ref-for-propdef-font-family⑦"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-family">font-family</a>, <a id="ref-for-propdef-font-size⑦"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-size">font-size</a>, <a id="ref-for-propdef-font-size-adjust⑦"></a><a href="https://drafts.csswg.org/css-fonts-5/#propdef-font-size-adjust">font-size-adjust</a>, <a id="ref-for-propdef-font-stretch⑦"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-stretch">font-stretch</a>, <a id="ref-for-propdef-font-style⑦"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-style">font-style</a>, <a id="ref-for-propdef-font-variant⑦"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-variant">font-variant</a>, <a id="ref-for-propdef-font-weight⑦"></a><a href="https://drafts.csswg.org/css-fonts-4/#propdef-font-weight">font-weight</a>, <a id="ref-for-propdef-glyph-orientation-vertical⑦"></a><a href="https://drafts.csswg.org/css-writing-modes-4/#propdef-glyph-orientation-vertical">glyph-orientation-vertical</a>, <a id="ref-for-propdef-image-rendering⑨"></a><a href="https://drafts.csswg.org/css-images-3/#propdef-image-rendering">image-rendering</a>, <a id="ref-for-propdef-isolation⑨"></a><a href="https://drafts.csswg.org/compositing-2/#propdef-isolation">isolation</a>, kerning, <a id="ref-for-propdef-letter-spacing⑦"></a><a href="https://drafts.csswg.org/css-text-4/#propdef-letter-spacing">letter-spacing</a>, <a id="ref-for-propdef-lighting-color⑨"></a><a href="https://drafts.csswg.org/filter-effects-1/#propdef-lighting-color">lighting-color</a>, <a id="ref-for-MarkerProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerProperty">marker</a>, <a id="ref-for-MarkerEndProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerEndProperty">marker-end</a>, <a id="ref-for-MarkerMidProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerMidProperty">marker-mid</a>, <a id="ref-for-MarkerStartProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerStartProperty">marker-start</a>, <a id="ref-for-propdef-mask⑦"></a><a href="https://drafts.csswg.org/css-masking-1/#propdef-mask">mask</a>, <a id="ref-for-propdef-opacity①⓪"></a><a href="https://drafts.csswg.org/css-color-4/#propdef-opacity">opacity</a>, <a id="ref-for-propdef-overflow⑦"></a><a href="https://drafts.csswg.org/css-overflow-3/#propdef-overflow">overflow</a>, <a id="ref-for-propdef-pointer-events⑦"></a><a href="https://drafts.csswg.org/css-ui-4/#propdef-pointer-events">pointer-events</a>, <a id="ref-for-ShapeRenderingProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#ShapeRenderingProperty">shape-rendering</a>, <a id="ref-for-StopColorProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/pservers.html#StopColorProperty">stop-color</a>, <a id="ref-for-StopOpacityProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/pservers.html#StopOpacityProperty">stop-opacity</a>, <a id="ref-for-StrokeProperty⑧"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeProperty">stroke</a>, <a id="ref-for-StrokeDasharrayProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeDasharrayProperty">stroke-dasharray</a>, <a id="ref-for-StrokeDashoffsetProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeDashoffsetProperty">stroke-dashoffset</a>, <a id="ref-for-StrokeLinecapProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeLinecapProperty">stroke-linecap</a>, <a id="ref-for-StrokeLinejoinProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeLinejoinProperty">stroke-linejoin</a>, <a id="ref-for-StrokeMiterlimitProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeMiterlimitProperty">stroke-miterlimit</a>, <a id="ref-for-StrokeOpacityProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeOpacityProperty">stroke-opacity</a>, <a id="ref-for-StrokeWidthProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeWidthProperty">stroke-width</a>, <a id="ref-for-TextAnchorProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/text.html#TextAnchorProperty">text-anchor</a>, <a id="ref-for-propdef-text-decoration⑦"></a><a href="https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration">text-decoration</a>, <a id="ref-for-TextRenderingProperty⑦"></a><a href="https://w3c.github.io/svgwg/svg2-draft/painting.html#TextRenderingProperty">text-rendering</a>, <a id="ref-for-propdef-unicode-bidi⑦"></a><a href="https://drafts.csswg.org/css-writing-modes-3/#propdef-unicode-bidi">unicode-bidi</a>, <a id="ref-for-propdef-visibility⑦"></a><a href="https://drafts.csswg.org/css-display-4/#propdef-visibility">visibility</a>, <a id="ref-for-propdef-word-spacing⑦"></a><a href="https://drafts.csswg.org/css-text-4/#propdef-word-spacing">word-spacing</a>, <a id="ref-for-propdef-writing-mode⑦"></a><a href="https://drafts.csswg.org/css-writing-modes-4/#propdef-writing-mode">writing-mode</a></li>
<li><a id="ref-for-filter-primitive-attributes⑥"></a><a href="https://drafts.csswg.org/filter-effects-1/#filter-primitive-attributes">filter primitive attributes</a>  —​<a id="ref-for-element-attrdef-filter-primitive-x①⓪"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-x">x</a>, <a id="ref-for-element-attrdef-filter-primitive-y①⓪"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-y">y</a>, <a id="ref-for-element-attrdef-filter-primitive-width①⓪"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-width">width</a>, <a id="ref-for-element-attrdef-filter-primitive-height①⓪"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-height">height</a>, <a id="ref-for-element-attrdef-filter-primitive-result①⓪"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-result">result</a> </li>
<li><a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute">class</a></li>
<li><a href="https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute">style</a></li>
<li><a id="ref-for-element-attrdef-filter-primitive-in①⑤"></a><a href="https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-in">in</a></li>
<li><a id="ref-for-element-attrdef-fedisplacementmap-in2"></a><a href="#element-attrdef-fedisplacementmap-in2">in2</a></li>
<li><a id="ref-for-element-attrdef-fedisplacementmap-scale"></a><a href="#element-attrdef-fedisplacementmap-scale">scale</a></li>
<li><a id="ref-for-element-attrdef-fedisplacementmap-xchannelselector"></a><a href="#element-attrdef-fedisplacementmap-xchannelselector">xChannelSelector</a></li>
<li><a id="ref-for-element-attrdef-fedisplacementmap-ychannelselector"></a><a href="#element-attrdef-fedisplacementmap-ychannelselector">yChannelSelector</a></li>
</ul></td>
</tr>
<tr class="odd">
<th><strong>DOM Interfaces:</strong></th>
<td><a href="https://drafts.csswg.org/filter-effects-1/#InterfaceSVGFEDisplacementMapElement">SVGFEDisplacementMapElement</a></td>
</tr>
</tbody>
</table>

<a id="issue-1d845580"></a>

<strong>Issue:</strong>

[](#issue-1d845580) Implementations do not match specification. [\[Issue \#113\]](https://github.com/w3c/csswg-drafts/issues/113)

This filter primitive uses the pixels values from the image from <a id="ref-for-element-attrdef-fedisplacementmap-in2①"></a>[in2](#element-attrdef-fedisplacementmap-in2) to spatially displace the image from <a id="ref-for-element-attrdef-filter-primitive-in①⑥"></a>[in](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-in). This is the transformation to be performed:

``` text
P'(x,y) ← P( x + scale * (XC(x,y) - .5), y + scale * (YC(x,y) - .5))
```

where P(x,y) is the input image, <a id="ref-for-element-attrdef-filter-primitive-in①⑦"></a>[in](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-in), and P'(x,y) is the destination. XC(x,y) and YC(x,y) are the component values of the channel designated by the <a id="ref-for-element-attrdef-fedisplacementmap-xchannelselector①"></a>[xChannelSelector](#element-attrdef-fedisplacementmap-xchannelselector) and <a id="ref-for-element-attrdef-fedisplacementmap-ychannelselector①"></a>[yChannelSelector](#element-attrdef-fedisplacementmap-ychannelselector). For example, to use the R component of <a id="ref-for-element-attrdef-fedisplacementmap-in2②"></a>[in2](#element-attrdef-fedisplacementmap-in2) to control displacement in x and the G component of Image2 to control displacement in y, set <a id="ref-for-element-attrdef-fedisplacementmap-xchannelselector②"></a>xChannelSelector to "R" and <a id="ref-for-element-attrdef-fedisplacementmap-ychannelselector②"></a>yChannelSelector to "G".

The displacement map, <a id="ref-for-element-attrdef-fedisplacementmap-in2③"></a>[in2](#element-attrdef-fedisplacementmap-in2), defines the inverse of the mapping performed.

The input image <a id="ref-for-element-attrdef-filter-primitive-in①⑧"></a>[in](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-in) is to remain premultiplied for this filter primitive. The calculations using the pixel values from <a id="ref-for-element-attrdef-fedisplacementmap-in2④"></a>[in2](#element-attrdef-fedisplacementmap-in2) are performed using non-premultiplied color values.

This filter can have arbitrary non-localized effect on the input which might require substantial buffering in the processing pipeline. However with this formulation, any intermediate buffering needs can be determined by <a id="ref-for-element-attrdef-fedisplacementmap-scale①"></a>[scale](#element-attrdef-fedisplacementmap-scale) which represents the maximum range of displacement in either x or y.

When applying this filter, the source pixel location will often lie between several source pixels.

Note: Depending on the speed of the available interpolents, this choice may be affected by the <a id="ref-for-propdef-image-rendering①⓪"></a>[image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering) property setting.

Note: A future version of this spec will define the interpolation method to be used when distorting the source image making UAs rendering result more interoperable.

The <a id="ref-for-propdef-color-interpolation-filters①④"></a>[color-interpolation-filters](#propdef-color-interpolation-filters) property only applies to the <a id="ref-for-element-attrdef-fedisplacementmap-in2⑤"></a>[in2](#element-attrdef-fedisplacementmap-in2) source image and does not apply to the <a id="ref-for-element-attrdef-filter-primitive-in①⑨"></a>[in](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-in) source image. The <a id="ref-for-element-attrdef-filter-primitive-in②⓪"></a>in source image must remain in its current color space.

<em>Attribute definitions:</em>

<strong><a id="element-attrdef-fedisplacementmap-scale"></a><strong><code>scale</code></strong> = "<em><a id="ref-for-number-value②⑥"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)</em>"</strong>

Displacement scale factor. The amount is expressed in the coordinate system established by attribute <a id="ref-for-element-attrdef-filter-primitiveunits①③"></a>[primitiveUnits](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitiveunits) on the <a id="ref-for-elementdef-filter③⑥"></a>[filter](https://drafts.csswg.org/filter-effects-1/#elementdef-filter) element.

When the value of this attribute is 0, this operation has no effect on the source image.

The <a id="ref-for-TermInitialValue④⓪"></a>[initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for <a id="ref-for-element-attrdef-fedisplacementmap-scale②"></a>[scale](#element-attrdef-fedisplacementmap-scale) is 0.

Animatable: yes.

<strong><a id="element-attrdef-fedisplacementmap-xchannelselector"></a><strong><code>xChannelSelector</code></strong> = "<em>R \| G \| B \| A</em>"</strong>

Indicates which channel from <a id="ref-for-element-attrdef-fedisplacementmap-in2⑥"></a>[in2](#element-attrdef-fedisplacementmap-in2) to use to displace the pixels in <a id="ref-for-element-attrdef-filter-primitive-in②①"></a>[in](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-in) along the x-axis.

The <a id="ref-for-TermInitialValue④①"></a>[initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for <a id="ref-for-element-attrdef-fedisplacementmap-xchannelselector③"></a>[xChannelSelector](#element-attrdef-fedisplacementmap-xchannelselector) is A.

Animatable: yes.

<strong><a id="element-attrdef-fedisplacementmap-ychannelselector"></a><strong><code>yChannelSelector</code></strong> = "<em>R \| G \| B \| A</em>"</strong>

Indicates which channel from <a id="ref-for-element-attrdef-fedisplacementmap-in2⑦"></a>[in2](#element-attrdef-fedisplacementmap-in2) to use to displace the pixels in <a id="ref-for-element-attrdef-filter-primitive-in②②"></a>[in](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-in) along the y-axis.

The <a id="ref-for-TermInitialValue④②"></a>[initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for <a id="ref-for-element-attrdef-fedisplacementmap-ychannelselector③"></a>[yChannelSelector](#element-attrdef-fedisplacementmap-ychannelselector) is A.

Animatable: yes.

<strong><a id="element-attrdef-fedisplacementmap-in2"></a><strong><code>in2</code></strong> = "<em>(see <a id="ref-for-element-attrdef-filter-primitive-in②③"></a>[in](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-in) attribute)</em>"</strong>

The second input image, which is used to displace the pixels in the image from attribute <a id="ref-for-element-attrdef-filter-primitive-in②④"></a>[in](https://drafts.csswg.org/filter-effects-1/#element-attrdef-filter-primitive-in). See defintion for <a id="ref-for-element-attrdef-filter-primitive-in②⑤"></a>in attribute.

Animatable: yes.

## <a id="ColorInterpolationFiltersProperty"></a>10. The <a id="ref-for-propdef-color-interpolation-filters②⑥"></a>[color-interpolation-filters](#propdef-color-interpolation-filters) property[](#ColorInterpolationFiltersProperty)

The description of the <a id="ref-for-propdef-color-interpolation-filters②⑦"></a>[color-interpolation-filters](#propdef-color-interpolation-filters) property is as follows:

| Field                                                                                    | Definition                                                                                                                                 |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-color-interpolation-filters"></a><strong>color-interpolation-filters</strong>                                               |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | auto <a id="ref-for-comb-one①⑨"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) sRGB <a id="ref-for-comb-one②⓪"></a>\| linearRGB |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | linearRGB                                                                                                                                  |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | All <a id="ref-for-filter-primitive④④"></a>[filter primitives](https://drafts.csswg.org/filter-effects-1/#filter-primitive)                |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | yes                                                                                                                                        |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                        |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | as specified                                                                                                                               |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                                                   |
| <strong>Media:</strong>                                                                  | visual                                                                                                                                     |

<strong><a id="valdef-color-interpolation-filters-auto"></a><strong>auto</strong></strong>

Indicates that the user agent can choose either the <a id="ref-for-valdef-color-interpolation-filters-srgb②"></a>[sRGB](#valdef-color-interpolation-filters-srgb) or <a id="ref-for-valdef-color-interpolation-filters-linearrgb②"></a>[linearRGB](#valdef-color-interpolation-filters-linearrgb) spaces for filter effects color operations. This option indicates that the author doesn’t require that color operations occur in a particular color space.

<strong><a id="valdef-color-interpolation-filters-srgb"></a><strong>sRGB</strong></strong>

Indicates that filter effects color operations should occur in the gamma-encoded <a id="ref-for-sRGB-space"></a>[sRGB](https://drafts.csswg.org/css-color-4/#sRGB-space) color space.

<strong><a id="valdef-color-interpolation-filters-linearrgb"></a><strong>linearRGB</strong></strong>

Indicates that filter effects color operations should occur in the <a id="ref-for-sRGB-linear-space"></a>[linear-light sRGB](https://drafts.csswg.org/css-color-4/#sRGB-linear-space) color space.

The <a id="ref-for-propdef-color-interpolation-filters②⑧"></a>[color-interpolation-filters](#propdef-color-interpolation-filters) property specifies the color space for imaging operations performed via filter effects.

Note: The <a id="ref-for-propdef-color-interpolation-filters②⑨"></a>[color-interpolation-filters](#propdef-color-interpolation-filters) property just has an affect on filter operations. Therefore, it has no effect on filter primitives like <a id="ref-for-elementdef-feoffset①③"></a>[feOffset](https://drafts.csswg.org/filter-effects-1/#elementdef-feoffset), <a id="ref-for-elementdef-feimage⑤"></a>[feImage](https://drafts.csswg.org/filter-effects-1/#elementdef-feimage), <a id="ref-for-elementdef-fetile①⓪"></a>[feTile](https://drafts.csswg.org/filter-effects-1/#elementdef-fetile) or <a id="ref-for-elementdef-feflood①⓪"></a>[feFlood](https://drafts.csswg.org/filter-effects-1/#elementdef-feflood).

Note: The <a id="ref-for-propdef-color-interpolation-filters③⓪"></a>[color-interpolation-filters](#propdef-color-interpolation-filters) has a different initial value than <a id="ref-for-ColorInterpolationProperty②②"></a>[color-interpolation](https://w3c.github.io/svgwg/svg2-draft/painting.html#ColorInterpolationProperty). <a id="ref-for-propdef-color-interpolation-filters③①"></a>color-interpolation-filters has an initial value of <a id="ref-for-valdef-color-interpolation-filters-linearrgb③"></a>[linearRGB](#valdef-color-interpolation-filters-linearrgb), where as <a id="ref-for-ColorInterpolationProperty②③"></a>color-interpolation has an initial value of <a id="ref-for-valdef-color-interpolation-filters-srgb③"></a>[sRGB](#valdef-color-interpolation-filters-srgb). Thus, in the default case, filter effects operations occur in the linearRGB color space, whereas all other color interpolations occur by default in the sRGB color space.

Note: The <a id="ref-for-propdef-color-interpolation-filters③②"></a>[color-interpolation-filters](#propdef-color-interpolation-filters) property has no affect on <a id="ref-for-filter-function②"></a>[filter functions](https://drafts.csswg.org/filter-effects-1/#filter-function), which operate in the sRGB color space.

The <a id="ref-for-propdef-color-interpolation-filters③③"></a>[color-interpolation-filters](#propdef-color-interpolation-filters) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

#### <a id="huerotateEquivalent"></a>13.1.4. hue-rotate[](#huerotateEquivalent)

``` text
<filter id="hue-rotate">
  <feColorMatrix type="hueRotate" values="[angle]"/>
</filter>
```

### <a id="tainted-filter-primitives"></a>15.1. Tainted Filter Primitives[](#tainted-filter-primitives)

It is important that the timing of any filter operation is independent of pixel values derived from the filtered content or other sources potentially containing privacy-sensitive information.

The following <a id="ref-for-filter-primitive④⑥"></a>[filter primitives](https://drafts.csswg.org/filter-effects-1/#filter-primitive) may have access to pixel values that potentially contain privacy-sensitive information, either from the filtered object itself or other sources such as CSS styling. These primitives must be flagged as "tainted".

1.  <a id="ref-for-elementdef-feflood①①"></a>[feFlood](https://drafts.csswg.org/filter-effects-1/#elementdef-feflood) when the <a id="ref-for-specified-value"></a>[specified value](https://drafts.csswg.org/css-cascade-5/#specified-value) of the <a id="ref-for-propdef-flood-color②⑤"></a>[flood-color](https://drafts.csswg.org/filter-effects-1/#propdef-flood-color) property computes to currentColor,

2.  <a id="ref-for-elementdef-fedropshadow①②"></a>[feDropShadow](https://drafts.csswg.org/filter-effects-1/#elementdef-fedropshadow) when the <a id="ref-for-specified-value①"></a>[specified value](https://drafts.csswg.org/css-cascade-5/#specified-value) value of the <a id="ref-for-propdef-flood-color②⑥"></a>[flood-color](https://drafts.csswg.org/filter-effects-1/#propdef-flood-color) property computes to currentColor,

3.  <a id="ref-for-elementdef-fediffuselighting①⓪"></a>[feDiffuseLighting](https://drafts.csswg.org/filter-effects-1/#elementdef-fediffuselighting), when the <a id="ref-for-specified-value②"></a>[specified value](https://drafts.csswg.org/css-cascade-5/#specified-value) value of the <a id="ref-for-propdef-lighting-color②⑤"></a>[lighting-color](https://drafts.csswg.org/filter-effects-1/#propdef-lighting-color) property computes to currentColor

4.  <a id="ref-for-elementdef-fespecularlighting①①"></a>[feSpecularLighting](https://drafts.csswg.org/filter-effects-1/#elementdef-fespecularlighting) when the <a id="ref-for-specified-value③"></a>[specified value](https://drafts.csswg.org/css-cascade-5/#specified-value) value of the <a id="ref-for-propdef-lighting-color②⑥"></a>[lighting-color](https://drafts.csswg.org/filter-effects-1/#propdef-lighting-color) property computes to currentColor,

5.  <a id="ref-for-elementdef-feimage⑥"></a>[feImage](https://drafts.csswg.org/filter-effects-1/#elementdef-feimage), when the <a id="ref-for-typedef-filter-url⑦"></a>[\<url\>](#typedef-filter-url) reference points to an element or fetches a resource with the fetching mode <em>No-CORS</em> and

6.  the filter primitives: <a id="ref-for-attr-valuedef-in-sourcegraphic①①"></a>[SourceGraphic](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-sourcegraphic), <a id="ref-for-attr-valuedef-in-sourcealpha⑦"></a>[SourceAlpha](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-sourcealpha), <a id="ref-for-attr-valuedef-in-backgroundimage④"></a>[BackgroundImage](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-backgroundimage), <a id="ref-for-attr-valuedef-in-backgroundalpha②"></a>[BackgroundAlpha](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-backgroundalpha), <a id="ref-for-attr-valuedef-in-fillpaint⑦"></a>[FillPaint](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-fillpaint) and <a id="ref-for-attr-valuedef-in-strokepaint③"></a>[StrokePaint](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-strokepaint).

<a id="ref-for-elementdef-feflood①②"></a>[feFlood](https://drafts.csswg.org/filter-effects-1/#elementdef-feflood), <a id="ref-for-elementdef-fedropshadow①③"></a>[feDropShadow](https://drafts.csswg.org/filter-effects-1/#elementdef-fedropshadow), <a id="ref-for-elementdef-fediffuselighting①①"></a>[feDiffuseLighting](https://drafts.csswg.org/filter-effects-1/#elementdef-fediffuselighting) and <a id="ref-for-elementdef-fespecularlighting①②"></a>[feSpecularLighting](https://drafts.csswg.org/filter-effects-1/#elementdef-fespecularlighting) are primitives with one or more CSS properties that take <a id="ref-for-typedef-color④"></a>[\<color\>](https://drafts.csswg.org/css-color-5/#typedef-color) as property value. <a id="ref-for-typedef-color⑤"></a>\<color\> consists of (amongst others) the currentColor keyword. The <a id="ref-for-used-value①"></a>[used value](https://drafts.csswg.org/css-cascade-5/#used-value) for currentColor derives from the <a id="ref-for-propdef-color①⑧"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) property. Since <a id="ref-for-propdef-color①⑨"></a>color can be set by the <a id="ref-for-visited-pseudo"></a>[:visited](https://drafts.csswg.org/selectors-4/#visited-pseudo) pseudo selector, it potentially contains privacy-sensitive information and therefore these primitives must be marked as tainted.

<a id="ref-for-elementdef-feimage⑦"></a>[feImage](https://drafts.csswg.org/filter-effects-1/#elementdef-feimage) can reference cross-domain images as well as document fragments such as SVG <a id="ref-for-graphics-element①"></a>[graphics elements](https://w3c.github.io/svgwg/svg2-draft/struct.html#graphics-element). These references potentially contain privacy-sensitive information and therefore the primitive must be marked as tainted.

The filter primitives <a id="ref-for-attr-valuedef-in-sourcegraphic①②"></a>[SourceGraphic](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-sourcegraphic), <a id="ref-for-attr-valuedef-in-sourcealpha⑧"></a>[SourceAlpha](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-sourcealpha), <a id="ref-for-attr-valuedef-in-backgroundimage⑤"></a>[BackgroundImage](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-backgroundimage), <a id="ref-for-attr-valuedef-in-backgroundalpha③"></a>[BackgroundAlpha](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-backgroundalpha), <a id="ref-for-attr-valuedef-in-fillpaint⑧"></a>[FillPaint](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-fillpaint) and <a id="ref-for-attr-valuedef-in-strokepaint④"></a>[StrokePaint](https://drafts.csswg.org/filter-effects-1/#attr-valuedef-in-strokepaint) either reference document fragments such as SVG <a id="ref-for-graphics-element②"></a>[graphics elements](https://w3c.github.io/svgwg/svg2-draft/struct.html#graphics-element) or style information that may derive directly or indirectly from the <a id="ref-for-propdef-color②⓪"></a>[color](https://drafts.csswg.org/css-color-4/#propdef-color) property. Therefore these primitives must be marked as tainted.

Every <a id="ref-for-filter-primitive④⑦"></a>[filter primitive](https://drafts.csswg.org/filter-effects-1/#filter-primitive) that has a "tainted" flagged <a id="ref-for-filter-primitive④⑧"></a>filter primitive as input must be flagged as "tainted" as well.

Filter operations must be implemented in such a way that they always take the same amount of time regardless of the pixel values if one of the input filter primitives is flagged as "tainted".

Note: This specification aggravates the restrictions to filter primitives based on implementation feedback from user agents.

### <a id="fedisplacemnentmap-restrictions"></a>15.2. <a id="ref-for-elementdef-fedisplacementmap④"></a>[feDisplacementMap](#elementdef-fedisplacementmap) Restrictions[](#fedisplacemnentmap-restrictions)

If <a id="ref-for-elementdef-fedisplacementmap⑤"></a>[feDisplacementMap](#elementdef-fedisplacementmap) has a "tainted" flagged filter primitive as input and this input filter primitive is used as displacement map (referenced by <a id="ref-for-element-attrdef-fedisplacementmap-in2⑧"></a>[in2](#element-attrdef-fedisplacementmap-in2)), then <a id="ref-for-elementdef-fedisplacementmap⑥"></a>feDisplacementMap must not proceed with the filter operation and acts as a <a id="ref-for-pass-through-filter②"></a>[pass through filter](https://drafts.csswg.org/filter-effects-1/#pass-through-filter).
