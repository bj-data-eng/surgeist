Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Filter Effects Module Level 2](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/Overview.bs).

The selected CSSWG repository licenses this document by its contributors under the [W3C Software and Document License](../licenses/w3c/software-license-2023.txt); its [exact repository license declaration](../licenses/w3c/csswg-drafts/LICENSE.md) is retained. No source copyright year is supplied by that declaration.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Filter Effects Module Level 2

Source snapshot: https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/Overview.bs

Pinned source SHA-256: 4e5ad82b5130dcc048913e30637e3962a6184b6eab19aca6e707fa8970ea1aa6

Generated intermediate HTML SHA-256: 578043b6540458dc5342ddbdcf86c7124c513bf3b399784192cc05dc20180720

Representation notes:
- Generated on 2026-10-03 from the exact pinned Bikeshed source with Bikeshed 7.1.3, then converted to Markdown. This is a generated rendering of that source, not an official publication or a captured historical rendering.
- The compiler used its bundled support-data manifest dated 2026-09-14 without updating it. External automatic link targets and generated bibliography descriptions come from that data; they do not establish historical versions of those external documents.
- Source headings, explicit anchors, normative prose, examples, metadata, and property-definition fields are retained. Compiler-inserted default property rows are omitted. Generated section numbers, cross-reference labels, and formatting are non-normative.
- Ambiguous automatic references remain visible without guessed destinations: “browsing context”.
- The pinned source contains an unmatched closing paragraph tag and combined privacy/security heading. The compiler’s tolerant rendering preserves its text; no editorial correction is applied. Ten relative example-image URLs remain pinned to the same source commit and were not downloaded or availability-tested.
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# Filter Effects Module Level 2

## <a id="source-metadata"></a>Source metadata

Metadata copied from the pinned source. Editor’s Draft status and work status are those of the source, not a claim of publication or present-day status.



| Field         | Source value                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
|---------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Status        | ED                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| Work Status   | Exploring                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| ED            | https://drafts.csswg.org/filter-effects-2/                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| TR            | https://www.w3.org/TR/filter-effects-2/                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Shortname     | filter-effects                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| Level         | 2                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| Link Defaults | css-transforms-1 (property) transform, svg (property) color-interpolation/fill/fill-opacity/fill-rule/stroke/glyph-orientation-horizontal/glyph-orientation-vertical/marker-start/marker-end/marker-mid/stop-color/stop-opacity/stroke-dasharray/stroke-dashoffset/stroke-linecap/stroke-linejoin/stroke-miterlimit/stroke-opacity/stroke-width/text-anchor/alignment-baseline/baseline-shift/dominant-baseline, css-masking-1 (property) clip-path/clip/clip-rule/mask, css-flexbox-1 (property) display, selectors-4 (type) \<compound-selector\>, css21 (type) \<margin-width\>, css-transforms-1 (type) \<transform-function\>, css-images-4 (property) image-rendering, css-color-3 (property) color, css-fonts-3 (property) font-family/font-stretch/font-style/font-variant/font-weight, selectors-4 (selector) :visited, css21 (dfn) containing block |
| Group         | csswg                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Editor        | Dean Jackson, Apple Inc. http://www.apple.com/, dino@apple.com, w3cid 42080                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| Editor        | Chris Harrelson, Google http://www.google.com/, chrishtr@chromium.org, w3cid 90243                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| Former Editor | Dirk Schulze, Adobe Inc., dschulze@adobe.com, w3cid 51803                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Test Suite    | http://test.csswg.org/suites/filter-effects/nightly-unstable/                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Abstract      | Filter effects are a way of processing an element's rendering before it is displayed in the document. Typically, rendering an element via CSS or SVG can conceptually described as if the element, including its children, are drawn into a buffer (such as a raster image) and then that buffer is composited into the elements parent. Filters apply an effect before the compositing stage. Examples of such effects are blurring, changing color intensity and warping the image.                                                                                                                                                                                                                                                                                                                                                                         |
| Abstract      | This is Level 2 of the Filter Effects Module.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |



## <a id="intro"></a>1. Introduction

<em>This section is non-normative</em>

A filter effect is a graphical operation that is applied to an element as it is drawn into the document. It is an image-based effect, in that it takes zero or more images as input, a number of parameters specific to the effect, and then produces an image as output. The output image is either rendered into the document instead of the original element, used as an input image to another filter effect, or provided as a CSS image value.

This is Level 2 of the Filter Effects Module. It is currently written as a "delta", describing any differences from Level 1.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-e3d8539e"></a> This specification was drafted for discussion, and does not yet have Working Group consensus. See [discussion in issue 53](https://github.com/w3c/fxtf-drafts/issues/53).

<a id="ref-for-propdef-backdrop-filter"></a>

## <a id="BackdropFilterProperty"></a>2. Backdrop filters: the [backdrop-filter](#propdef-backdrop-filter) property

<a id="ref-for-propdef-backdrop-filter①"></a>

The description of the [backdrop-filter](#propdef-backdrop-filter) property is as follows:



| Field               | Definition                                                                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;     </strong> | <a id="propdef-backdrop-filter"></a>backdrop-filter                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;     </strong> | <a id="ref-for-typedef-filter-value-list"></a><a id="ref-for-comb-one"></a>none [\|](https://drafts.csswg.org/css-values-4/#comb-one) [\<filter-value-list\>](https://drafts.csswg.org/filter-effects-1/#typedef-filter-value-list)       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;     </strong> | none                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;     </strong> | <a id="ref-for-container-element"></a>All elements. In SVG, it applies to [container elements](https://w3c.github.io/svgwg/svg2-draft/struct.html#container-element) without the defs element and all graphics elements |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;     </strong> | no                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;     </strong> | n/a                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;     </strong> | as specified                                                                                                                                                                                         |
| <strong>Media:&#xA;     </strong> | visual                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;     </strong> | see prose in [Filter Effects 1 §  14. Animation of Filters](https://drafts.csswg.org/filter-effects-1/#animation-of-filters).                                                                        |



<a id="ref-for-propdef-backdrop-filter②"></a>

If the value of the [backdrop-filter](#propdef-backdrop-filter) property is none then there is no filter effect applied. Otherwise, the list of functions are applied in the order provided.

### <a id="backdrop-filter-operation"></a>2.1. Filtering and Clipping

An element (call it B) with a backdrop-filter property other than none is rendered as if the following steps are performed:

1.  <a id="ref-for-backdrop-root-image"></a>

    Copy the [Backdrop Root Image](#backdrop-root-image) into a temporary buffer, such as a raster image. Call this buffer T’.

2.  Apply the backdrop-filter’s filter operations to the entire contents of T'.

3.  If element B has any transforms (between B and the Backdrop Root), apply the <b>inverse</b> of those transforms to the contents of T’.

4.  Apply a clip to the contents of T’, using the border box of element B, including [border-radius](https://drafts.csswg.org/css-backgrounds-3/#border-radius) if specified. Note that the children of B are not considered for the sizing or location of this clip.

5.  Draw all of element B, including its background, border, and any children elements, into T’.

6.  If element B has any transforms, effects, or clips, apply those to T’.

7.  Composite the contents of T’ into element B’s parent, using source-over compositing.

<a id="ref-for-elementdef-filter"></a>

<a id="ref-for-typedef-filter-value-list①"></a>

<a id="ref-for-backdrop-root-image①"></a>

The first [filter](https://drafts.csswg.org/filter-effects-1/#elementdef-filter) function or filter reference in the [\<filter-value-list\>](https://drafts.csswg.org/filter-effects-1/#typedef-filter-value-list) takes the element’s [Backdrop Root Image](#backdrop-root-image) as the input image. Subsequent operations take the output from the previous filter function or <a id="ref-for-elementdef-filter①"></a>filter reference as the input image. The <a id="ref-for-elementdef-filter②"></a>filter element reference functions can specify an alternate input, but still uses the previous output as its SourceGraphic.

Filter functions must operate in the sRGB color space.

If the filter functions list includes a [blur()](https://drafts.csswg.org/filter-effects-1/#blurEquivalent) filter, the filter will be applied with edgeMode="mirror", with the edge defined by the clipped, transformed border box of the element. See [§ 3 Backdrop Root](#BackdropRoot).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The effect of the backdrop-filter will not be visible unless some portion of element B is semi-transparent. Also note that any opacity applied to element B will be applied to the filtered backdrop image as well. Therefore, to create a "transparent" element that allows the full filtered backdrop image to be seen, you can use "background-color: transparent;".

A computed value of other than none results in the creation of both a [stacking context](https://www.w3.org/TR/CSS21/zindex.html) [\[CSS21\]](#biblio-css21) and a [Containing Block](https://developer.mozilla.org/en-US/docs/Web/CSS/Containing_block) for absolute and fixed position descendants, unless the element it applies to is a document root element in the current <a id="ref-for-browsing-context"></a>browsing context.

<a id="ref-for-propdef-filter"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This rule works in the same way as for the [filter](https://drafts.csswg.org/filter-effects-1/#propdef-filter) property.

## <a id="BackdropRoot"></a>3. Backdrop Root

The <a id="backdrop-root-image"></a>Backdrop Root Image for an element E is the final image that would be produced by the following steps:

1.  <a id="ref-for-backdrop-root"></a>

    Start at the [Backdrop Root](#backdrop-root) element that is the nearest ancestor of E.

2.  Paint all content, in [painting order](https://www.w3.org/TR/CSS2/zindex.html#painting-order), between (and including) the ancestor Backdrop Root element and element E.

3.  Flatten the painted content into a 2D screen-space buffer.

4.  Transform the border box of element E to 2D screen-space, and clip the final painted output to this border quad.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-ade95eb0"></a> This specification does not yet have Working Group consensus, specifically on the definition of Backdrop Root. See [discussion in issue 53](https://github.com/w3c/fxtf-drafts/issues/53).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: No content that is a DOM ancestor of the Backdrop Root element should contribute to or affect the Backdrop Root Image.

A <a id="backdrop-root"></a>Backdrop Root is formed, anywhere in the document, by an element in any of the following scenarios. See [§ 3.2 Backdrop Root Triggers](#BackdropRootTriggers) for more details on each:

- The root element of the document (HTML).

- An element with a filter property other than "none".

- An element with an opacity value less than 1.

- An element with mask, mask-image, mask-border, or clip-path properties with values other than “none”.

- An element with a backdrop-filter value other than "none".

- An element with a mix-blend-mode value other than "normal".

- An element with a will-change value specifying any property that would create a Backdrop Root on non-initial value.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: this definition encompases fewer element types than the definition for a Stacking Context. In particular, a Backdrop Root is not formed by elements with z-index applied, fixed or sticky-positioned elements, and elements with transforms applied. This allows elements with backdrop-filter or mix-blend-mode to apply to elements higher up the DOM tree than would otherwise apply if they stopped at the parent Stacking Context. For example, a container can be used to contain elements with backdrop-filter applied, and that container can use transforms for animation or positioning, while still allowing the backdrop-filter to apply to the background behind the container.

### <a id="BackdropRootMotivation"></a>3.1. Motivation

<em>This section is non-normative</em>

There are currently three related, but distinct, concepts in the web platform:

- The [Stacking Context](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Positioning/Understanding_z_index/The_stacking_context). A Stacking Context is induced by many different types of element properties, and is primarily related to defining the painting (Z) order of elements.

- The [Containing Block](https://developer.mozilla.org/en-US/docs/Web/CSS/Containing_block). A Containing Block is also induced by several types of element properties, and is primarily related to defining how layout (X/Y) is performed for elements.

- The [3D Rendering Context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-contexts). A 3D Rendering Context is induced by several properties, mainly transform-style, but also any of the grouping properties, and is primarily related to defining the relevant coordinate space for 3D transforms.

There is an important motivation for the creation of a separate web platform concept for the Backdrop Root, rather than just re-using an existing concept. There are essentially two other choices that might be used as a definition of the "backdrop": 1) “everything that painted before, up to the root node”, or 2) “everything up to the parent stacking context”. However, there are ambiguities with definition \#1, and excessive limitations with definition \#2. Therefore, the Backdrop Root concept strikes a balance between the two.

In particular, it is important to note that including “everything that paints before” an element is ambiguous in the case where an ancestor of the element contains filtering and/or opacity. Because those effects are inherited by descendant elements, including the element containing backdrop-filter or mix-blend-mode, it is not clear “when” to apply the effect. Filters and opacity create a stacking context, and the [css filter-effect specification](https://drafts.csswg.org/filter-effects/#FilterProperty) states that “All the \[Stacking Context\] descendants are rendered together as a group with the filter effect applied to the group as a whole.” If, somewhere inside that stacking context, an element contains a backdrop-filter property other than “none”, then it is impossible to honor the preceding sentence. At the point of the backdrop-filtered element, all of the (partially-painted) contents of the stacking context need to be fully rendered, including applied filters, opacity, and blending with the backdrop, and the resulting image needs to be used as the input to the backdrop-filtered element. That fundamentally breaks the atomicity of the stacking context. And the ambiguity arises from the need, at the end of rendering the fully-completed stacking context, to apply those filters and opacity <b>again</b> to the completed rendering. The filters and opacity will, at this point, be applied twice to the portion of the image that has been backdrop-filtered. This situation grows exponentially worse if backdrop-filters are nested inside each other. For this reason, it is necessary to prevent the backdrop-filter from “seeing" above nodes that have filters, opacity, and the other conditions listed above in the definition of the Backdrop Root.

Performance would also be an issue, if backdrop-filter and mix-blend-mode were defined to filter "everything" that comes before them on the page. Each application of backdrop-filter or mix-blend-mode would require a separate rendering pass, to temporarily finish any partially-complete stacking contexts, and get the "final" output that would appear behind each element. That would double the required rendering time, and would potentially require twice the memory usage and GPU bandwidth to store the intermediate graphics texture holding the contents to be filtering. And assuming nested backdrop-filters or mix-blend-mode elements were allowed, this doubling would become an exponential performance breakdown. Clearly, this is not a scalable approach.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2baed611"></a>
>
> Given the following html code:
>
> ```text
> <html style="background:lightgrey;">
>   <div class="box">
>     <b>"Box"</b><br><br>Lorem ipsum dolor sit amet, consectetur adipiscing elit,
>     sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim
>     ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip
>     ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate
>     velit esse cillum
>     <div class="dialog"></div>
>   </div>
> </html>
> ```
>
> And the following style rules:
>
> ```text
> <style>
>   div {
>     position: absolute;
>     width: 200px;
>     height: 200px;
>     top: 70px;
>     left: 70px;
>   }
>   .box {
>     background: white;
>     filter: invert(1);
>   }
>   .dialog {
>     border: 10px solid blue;
>     backdrop-filter: blur(2px);
>   }
> </style>
> ```
>
> Consider the sequence of painting operations to produce the final result, assuming that the "dialog" element accesses <b>everything behind it</b>, all the way to the root element. <b>Important note:</b> this is <b>not</b> how the Backdrop Root is defined - this example is intended to illustrate why a Backdrop Root is formed by elements containing filters.
>
> ![ALT TEXT HERE](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/examples/step1.png)
>
> <b>Step 1.</b> "Box" is rendered, but filters are not yet applied.
>
> ![ALT TEXT HERE](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/examples/step2.png)
>
> <b>Step 2.</b> "Dialog" is about to be rendered, but it has backdrop-filter applied. Since, for this example, the backdrop-filter "sees" all the way to the root, the "Box" element needs to be temporarily "completed" so that the final product can be used as input to "Dialog". This means applying its "filter: invert(1)" style. Note, this is where the atomicity of the Stacking Context created by "Box" is broken. This is also where the performance penalty applies - the GPU draw work that is performed here will end up being done twice.
>
> ![ALT TEXT HERE](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/examples/step3.png)
>
> <b>Step 3.</b> Start painting the "Dialog" element by reading back its backdrop image, applying the "backdrop-filter: blur(2px)" filter, and cropping those results to the border box of "Dialog". (The dotted black border has been added here for clarity: this is the border box for the "Dialog" element.)
>
> ![ALT TEXT HERE](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/examples/step4.png)
>
> <b>Step 4.</b> Now that the backdrop-filtered contents have been read back and filtered, discard the previously "completed" Box element and go back to the unfiltered version. Note that you can now see the black text at the bottom bleeding through the white, blurred version inside "Dialog".
>
> ![ALT TEXT HERE](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/examples/step5.png)
>
> <b>Step 5.</b> Draw the contents of the "Dialog" element - the <b>blue</b> border.
>
> ![ALT TEXT HERE](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/examples/step6.png)
>
> <b>Step 6.</b> Complete the Stacking Context formed by the "Box" element - apply the "filter: invert(1)" to "Box" and all of its contents, including the now-completed "Dialog" element. Note that the area inside the yellow "Dialog" element has been inverted twice - it appears white. Also note that the text outside "Box" has been drawn in blurred black text over the top of un-blurred white text, and the black edge of "Box" shows through the now-white blurred edge of "Box". None of these visual effects is expected or intuitive.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5dfcdb97"></a>
>
> Opacity poses much the same problem that filter does. Using the example above, but with the `filter: invert(1);` style replaced with `opacity: 0.5;` the results are similarly unintuitive. <b>Important note:</b> the left image below does <b>not</b> represent how the Backdrop Root is defined - this example is intended to illustrate why a Backdrop Root is formed by elements containing opacity less than 1.
>
> ![ALT TEXT HERE](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/examples/opacity.png)      ![ALT TEXT HERE](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/examples/opacity_correct.png)
>
> The left image is the final result of rendering, assuming the Backdrop Root includes everything on the page. Note that the blurred region inside the blue border is very faint. It is rendered at 0.25 opacity (double-counted), instead of the 0.5 opacity inherited from the "Box" element. The right image is the final result using the correct definition of Backdrop Root. The blurred region inside the blue border has opacity 0.5, as expected.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5be62f4f"></a>
>
> Because masks effectively change the opacity of parts of the elements they are masking, the behavior for mask operations is quite similar to that for elements with opacity applied. Using the same example as above, but replacing the filter/opacity style line with
>
> ```text
> -webkit-mask-image:
>   linear-gradient(to top, transparent 25%, black 75%);
> ```
>
> the results are quite similar. The masked area is applied twice, meaning the alpha channel is effectively squared. <b>Important note:</b> the left image below does <b>not</b> represent how the Backdrop Root is defined - this example is intended to illustrate why a Backdrop Root is formed by elements containing masks.
>
> ![ALT TEXT HERE](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/examples/mask.png)      ![ALT TEXT HERE](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/examples/mask_correct.png)
>
> The left image is the final result of rendering, assuming the Backdrop Root includes everything on the page. Note that the gradient applied to the blurred region inside the blue border (but not the border itself) is very faint at the bottom. The opacity of the mask has been applied twice, so the gradient is no longer a linear gradient, but is instead a quadratic gradient. The right image is the final result using the correct definition of Backdrop Root. The mask is applied correctly as a linear gradient for both the blue border and the blurred region inside.

### <a id="BackdropRootTriggers"></a>3.2. Backdrop Root Triggers

<em>This section is non-normative</em>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-ade95eb0①"></a> This specification does not yet have Working Group consensus, specifically on the definition of Backdrop Root. See [discussion in issue 53](https://github.com/w3c/fxtf-drafts/issues/53).

As described in [§ 3.1 Motivation](#BackdropRootMotivation), several operations pose fundamental problems for operations that need to read back content that was painted before, and then re-paint that content (possibly filtered or blended) again. The list of element types that trigger a [§ 3 Backdrop Root](#BackdropRoot) each pose one of these problems. Some are obvious and some are more nuanced. This section describes why each trigger is necessary.

Obvious triggers:

- <b>The root element.</b> There is nothing above the root element, so it must form a Backdrop Root.

- <b>Will-change values.</b> The will-change hint means that the element might take on a value that creates a Backdrop Root. For this reason, the Backdrop Root should be created immediately, to avoid changing it (and therefore changing the rendered appearance) during animation.

Less obvious triggers:

- <b>An element with a filter.</b> Filters apply to an element and all of its children, rendered together. Allowing the Backdrop Root to read back content above a filtered element necessitates breaking the atomicity of the filter, with undefined results. See [§ 3.1 Motivation](#BackdropRootMotivation) for more a much more detailed explanation and some examples.

- <b>An element with opacity &lt; 1.</b> Elements with an opacity value less than 1.0 suffer from a very similar problem as elements with filters. Because the opacity applies to the element and all children, rendered together, the opacity value would be "applied twice" if the Backdrop Root Image were to read back content above the semi-transparent element. Effectively, the backdrop-filtered portion of the backdrop would be rendered with (Opacity)^2. See [§ 3.1 Motivation](#BackdropRootMotivation) for more details and an example.

- <b>CSS Masks.</b> There are two reasons why a CSS mask triggers a Backdrop Root element. The first is that masks are exactly analogous to the opacity problem described in the bullet above. Masks can contain regions of partial opacity (e.g. `style="mask-image: linear-gradient(to bottom, transparent 25%, black 75%);"`). If the Backdrop Root Image were to read back content above the mask, those partially-opaque regions would be double-applied to the final backdrop image, which would result in incorrect rendering. See [§ 3.1 Motivation](#BackdropRootMotivation) for more details and an example.

- <b>Elements with clip-path.</b> These elements are effectively rendered as if they were a mask. While in general that mask doesn’t explicitly contain partially-opaque regions, it is usually rendered with antialiasing, which introduces semi-transparent portions around the edges of the clip. Therefore, for the same reasons as described above for masks, those semi-transparent regions will be incorrectly rendered, producing visual artifacts near the edges of the clip. Disabling antialiasing for clip-paths would produce poor rendering quality, and relaxing the Backdrop Root restriction on these elements would similarly lead to observable artifacts.

- <b>Backdrop-filter and mix-blend-mode.</b> These elements themselves must form a Backdrop Root for two reasons. First, they cause the same problems that filters and opacity cause: there is an ambiguity about what constitutes the image to be filtered or blended, given the atomicity of the operations. Second, there would likely be an exponential performance degradation in the case of nested backdrop-filter or mix-blend-mode elements, due to the need to re-paint the content behind each element. Each nesting level will double the number of these required re-paint cycles, leading to significant performance problems.

For all of the above triggers, performance is also an important motivation. In all of these cases, relaxing the Backdrop Root constraint would lead to a potential doubling of the CPU/GPU memory and bandwidth. See [§ 3.1 Motivation](#BackdropRootMotivation) for a more detailed discussion.

### <a id="mix-blend-mode"></a>3.3. Mix Blend Mode

<em>This section is non-normative</em>

The current definition of mix-blend-mode [defines](https://www.w3.org/TR/compositing-1/#csscompositingrules_CSS) the input (backdrop) image for mix-blend-mode as being all of the underlying content of the parent stacking context. This definition could likely be relaxed to use the Backdrop Root definition instead, allowing more elements to be blended.

## <a id="priv-sec"></a>4. Privacy and Security Considerations

All of the same privacy and security concerns exist for backdrop-filter as do for "standard" filters. See [Filter Effects 1 § 15.1 Tainted Filter Primitives](https://drafts.csswg.org/filter-effects-1/#tainted-filter-primitives) for more details.

## <a id="acknowledgments"></a>Acknowledgments

The editors would like to thank Mason Freed, Marcus Stange, Matt Rakow, Simon Fraser, Amelia Bellamy-Royds, Dirk Schulze, and Tab Atkins for their careful reviews, comments, and corrections.

## <a id="references"></a>References

Generated bibliography: these reference descriptions and external auto-links were resolved using Bikeshed 7.1.3’s bundled data; they are not a historical capture of the linked specifications.

### <a id="normative"></a>Normative References

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-values-4&#x2F;](https://drafts.csswg.org/css-values-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://drafts.csswg.org/css2/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css2&#x2F;](https://drafts.csswg.org/css2/)

<a id="biblio-filter-effects-1"></a>\[FILTER-EFFECTS-1\]  
Dirk Schulze; Dean Jackson. [Filter Effects Module Level 1](https://drafts.csswg.org/filter-effects-1/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;filter-effects-1&#x2F;](https://drafts.csswg.org/filter-effects-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://w3c.github.io/svgwg/svg2-draft/). URL: [https&#x3A;&#x2F;&#x2F;w3c&#x2E;github&#x2E;io&#x2F;svgwg&#x2F;svg2-draft&#x2F;](https://w3c.github.io/svgwg/svg2-draft/)

## <a id="source-linking-configuration"></a>Source linking configuration

Compiler configuration recorded in the pinned source; retained for provenance.

```text
  spec:filter-effects-1; type:element; text:filter
```