Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Color Adjustment Module Level 1](https://drafts.csswg.org/css-color-adjust-1/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10. Source status: W3C Candidate Recommendation Snapshot, 7 May 2026.

Full captured HTML SHA-256: `c860033c2f13b19a3e1fed1a645a21e606a5d3aa50d54ee638675efc63b05db5` (281109 bytes). Conversion input SHA-256: `a703bef833dba26ffc293be616d9b2ec1a52f4c1d1b22655f946ac8696edb43f` (71905 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained scope: `preferred`, `emulate-forced-colors-mode`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Color Adjustment Module Level 1

<a id="w3c-state"></a>[W3C Candidate Recommendation Snapshot](https://www.w3.org/standards/types/#CR), 7 May 2026

More details about this document

<strong>This version:</strong>

<https://www.w3.org/TR/2026/CR-css-color-adjust-1-20260507/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-color-adjust-1/>

<strong>Editor's Draft:</strong>

<https://drafts.csswg.org/css-color-adjust-1/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2022/CRD-css-color-adjust-1-20220614/>

<strong>History:</strong>

<https://www.w3.org/standards/history/css-color-adjust-1/>

<strong>Implementation Report:</strong>

<https://wpt.fyi/results/css/css-color-adjust?label=experimental&label=master&aligned>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-color-adjust-1)

[Inline In Spec](https://drafts.csswg.org/css-color-adjust-1/#issues-index)

<strong>Editors:</strong>

[Alison Maher](mailto:almaher@microsoft.com) (Microsoft)

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Rossen Atanassov](mailto:ratan@microsoft.com) (Microsoft)

[Rune Lillesveen](mailto:futhark@google.com) (Google)

[Tab Atkins Jr.](http://www.xanthir.com/contact/) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-color-adjust-1/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-color-adjust/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="preferred"></a>2. Preferred Color Schemes[](#preferred)

Operating systems and user agents often give users the ability to choose their <a id="preferred-color-scheme"></a><strong>preferred color scheme</strong> for user interface elements. This <a id="color-scheme"></a><strong>color scheme</strong> is typically reflected in the user agent’s rendering of its navigation interface as well as in-page interface elements such as form controls and scrollbars.

A UA can also allow the user to indicate a preference for the <a id="ref-for-color-scheme②"></a>[color scheme](#color-scheme) of the pages they view, requesting that the author adapt the page to those color preferences. (It is not required to express such a preference; users can have preferences for operating system interface colors that they do not want imposed on pages.)

The most common <a id="ref-for-color-scheme③"></a>[color scheme](#color-scheme) preferences are:

- A <a id="light-color-scheme"></a><strong>light color scheme</strong> ("day mode") consists of light background colors and dark foreground/text colors.

- A <a id="dark-color-scheme"></a><strong>dark color scheme</strong> ("night mode") consists of the opposite, with dark background colors and light foreground/text colors.

Note: <a id="ref-for-light-color-scheme"></a>[Light](#light-color-scheme) and <a id="ref-for-dark-color-scheme"></a>[dark](#dark-color-scheme) <a id="ref-for-color-scheme④"></a>[color schemes](#color-scheme) are not specific color palettes, but a range of possible palettes. For example, a stark black-on-white scheme and a sepia dark-on-tan scheme would both be considered <a id="ref-for-light-color-scheme①"></a>light color schemes. To guarantee specific colors, authors must specify those colors themselves. Note also that, consequently, pairing default or <a id="ref-for-typedef-system-color"></a>[\<system-color\>](https://www.w3.org/TR/css-color-4/#typedef-system-color) colors with author-specified colors cannot guarantee any particular contrast level; to ensure legibility, generally either both foreground and background should be paired <a id="ref-for-css-system-colors"></a>[system colors](https://www.w3.org/TR/css-color-4/#css-system-colors) or both be manually specified colors. [\[WCAG22\]](https://drafts.csswg.org/css-color-adjust-1/#biblio-wcag22)

The user’s <a id="ref-for-preferred-color-scheme"></a>[preferred color scheme](#preferred-color-scheme) is then combined with the <a id="ref-for-pages-supported-color-scheme"></a>[page’s supported color schemes](#pages-supported-color-scheme) to produce a <a id="page-color-scheme"></a><strong>page color scheme</strong>. The <a id="ref-for-page-color-scheme"></a>[page color scheme](#page-color-scheme) also records whether it’s <a id="color-scheme-default"></a><strong>default</strong> or not. The <a id="ref-for-page-color-scheme①"></a>page color scheme is queryable with the <a id="ref-for-descdef-media-prefers-color-scheme①"></a>[prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) <a id="ref-for-media-query①"></a>[media query](https://www.w3.org/TR/mediaqueries-5/#media-query). [\[MEDIAQUERIES-5\]](https://drafts.csswg.org/css-color-adjust-1/#biblio-mediaqueries-5)

Individual elements have an <a id="element-color-scheme"></a><strong>element color scheme</strong>, which by default matches the <a id="ref-for-page-color-scheme②"></a>[page color scheme](#page-color-scheme), but can be overriden using the <a id="ref-for-propdef-color-scheme①"></a>[color-scheme](#propdef-color-scheme) property. The <a id="ref-for-element-color-scheme"></a>[element color scheme](#element-color-scheme) affects things such as the value of <a id="ref-for-css-system-colors①"></a>[system colors](https://www.w3.org/TR/css-color-4/#css-system-colors) on the element, the value of the <a id="ref-for-funcdef-light-dark"></a>[light-dark()](https://www.w3.org/TR/css-color-5/#funcdef-light-dark) function, and how user interface elements (like scrollbars) render. (See [§ 2.4 Effects of the Used Color Scheme](#color-scheme-effect).)

User agents <em>may</em> support additional <a id="ref-for-color-scheme⑤"></a>[color schemes](#color-scheme), however CSS does not support negotiation of additional <a id="ref-for-color-scheme⑥"></a>color schemes: user agents should pursue standardization of these schemes, so that <a id="ref-for-descdef-media-prefers-color-scheme②"></a>[prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) and <a id="ref-for-propdef-color-scheme②"></a>[color-scheme](#propdef-color-scheme) can reflect the additional values.

Note: Because many pages were authored before <a id="ref-for-color-scheme⑦"></a>[color scheme](#color-scheme) support existed, and thus pages were authored with the assumption of the default (<a id="ref-for-light-color-scheme②"></a>[light](#light-color-scheme)) color scheme, user agents cannot automatically adapt the colors used in elements under their control, as it might cause unreadable color contrast with the surrounding page. Pages <em>have to</em> opt into <a id="ref-for-color-scheme⑧"></a>color scheme support by setting the <a id="ref-for-page-color-scheme③"></a>[page color scheme](#page-color-scheme), or manually setting the <a id="ref-for-propdef-color-scheme③"></a>[color-scheme](#propdef-color-scheme) property on individual elements.

### <a id="color-scheme-page"></a>2.1. Opting Into a Preferred Color Scheme[](#color-scheme-page)

The <a id="ref-for-page-color-scheme④"></a>[page color scheme](#page-color-scheme) represents the <a id="ref-for-color-scheme⑨"></a>[color scheme](#color-scheme) that will be used for the page overall (queryable with the prefers-color-scheme <a id="ref-for-media-feature"></a>[media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature)) and for all elements that don’t specifically override the <a id="ref-for-page-color-scheme⑤"></a>page color scheme with the <a id="ref-for-propdef-color-scheme④"></a>[color-scheme](#propdef-color-scheme) property. See [§ 2.4 Effects of the Used Color Scheme](#color-scheme-effect) for a full description of what the <a id="ref-for-color-scheme①⓪"></a>color scheme effects.

Note: In HTML, the <code><a id="ref-for-meta-color-scheme"></a>[color-scheme \<meta\>](https://html.spec.whatwg.org/multipage/semantics.html#meta-color-scheme)</code> sets the <a id="ref-for-page-color-scheme⑥"></a>[page color scheme](#page-color-scheme).

Authors <em>should</em> generally set the <a id="ref-for-page-color-scheme⑦"></a>[page color scheme](#page-color-scheme), rather than using the <a id="ref-for-propdef-color-scheme⑤"></a>[color-scheme](#propdef-color-scheme) property, so that the <a id="ref-for-descdef-media-prefers-color-scheme③"></a>[prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) <a id="ref-for-media-feature①"></a>[media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature) is consistent with the elements on the page.

The <a id="ref-for-page-color-scheme⑧"></a>[page color scheme](#page-color-scheme) is determined by finding the <a id="ref-for-used-color-scheme"></a>[used color scheme](#used-color-scheme), given the <a id="ref-for-preferred-color-scheme①"></a>[preferred color scheme](#preferred-color-scheme) and the <a id="pages-supported-color-scheme"></a><strong>page’s supported color schemes</strong> (a <a id="ref-for-color-scheme-support"></a>[color scheme support](#color-scheme-support)).

In embedded documents (such as an <code><a id="ref-for-the-iframe-element"></a>[iframe](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element)</code> or an SVG <code><a id="ref-for-the-img-element"></a>[img](https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element)</code>), the embedding element’s <a id="ref-for-element-color-scheme①"></a>[element color scheme](#element-color-scheme) is used as the embedded document’s <a id="ref-for-preferred-color-scheme②"></a>[preferred color scheme](#preferred-color-scheme) (treated as a normal <a id="ref-for-color-scheme-preference"></a>[color scheme preference](#color-scheme-preference)), rather than the user’s preference. This allows embedded documents to match the negotiated color scheme for the page or an element, similar to other elements in the parent document.

<strong>Note:</strong>

<a id="example-6e0f57a8"></a>

<strong>Example:</strong>

[](#example-6e0f57a8) A page that responds to user preferences for light or dark display by using the prefers-color-scheme media feature to alter the colors it uses can easily opt the browser-controlled UI (scrollbars, inputs, etc) to match with a simple global declaration:

``` text
<meta name=color-scheme contents="light dark">
```

(Or, in languages that don’t have a way to set the <a id="ref-for-page-color-scheme⑨"></a>[page color scheme](#page-color-scheme) directly, a :root { color-scheme: light dark;} rule.)

If a page limits itself to using <em>only</em> the <a id="ref-for-typedef-system-color①"></a>[\<system-color\>](https://www.w3.org/TR/css-color-4/#typedef-system-color)s, the <a id="ref-for-propdef-color-scheme⑥"></a>[color-scheme](#propdef-color-scheme) declaration will support the user’s preferred color scheme even without the author needing to use <a id="ref-for-at-ruledef-media"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) at all.

<a id="example-007c69e0"></a>

<strong>Example:</strong>

[](#example-007c69e0) If a page cannot reasonably accommodate all color schemes, such as for branding or theatrical reasons, <code><a id="ref-for-meta-color-scheme①"></a>[color-scheme](https://html.spec.whatwg.org/multipage/semantics.html#meta-color-scheme)</code> or <a id="ref-for-propdef-color-scheme⑦"></a>[color-scheme](#propdef-color-scheme) can still indicate which color schemes the page <em>can</em> support, causing the UI to match.

If the page’s color scheme is primarily light, the following will indicate that explicitly:

``` text
<meta name=color-scheme contents="light">
```

While if the page is primarily dark, indicating that explicitly will make the page look more coherent as well:

``` text
<meta name=color-scheme contents="dark">
```

However, it is better to support both color schemes, of course.

### <a id="color-scheme-prop"></a>2.2. Overriding the Page Color Scheme: the <a id="ref-for-propdef-color-scheme⑧"></a>[color-scheme](#propdef-color-scheme) property[](#color-scheme-prop)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-color-scheme"></a><strong>color-scheme</strong>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | normal <a id="ref-for-comb-one"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ light <a id="ref-for-comb-one①"></a>\| dark <a id="ref-for-comb-one②"></a>\| <a id="ref-for-identifier-value"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) \]<a id="ref-for-mult-one-plus"></a>[+](https://www.w3.org/TR/css-values-4/#mult-one-plus) <a id="ref-for-comb-all"></a>[&&](https://www.w3.org/TR/css-values-4/#comb-all) only<a id="ref-for-mult-opt"></a>[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | normal                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | all elements and text                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | the keyword <a id="ref-for-valdef-color-scheme-normal"></a>[normal](#valdef-color-scheme-normal), or a <a id="ref-for-color-scheme-support①"></a>[color scheme support](#color-scheme-support)                                                                                                                                                                                                                                                                                                                                                     |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |

While the <a id="ref-for-page-color-scheme①⓪"></a>[page color scheme](#page-color-scheme) should generally be used to control what <a id="ref-for-color-scheme①①"></a>[color scheme](#color-scheme) a page uses, occasionally there is need to override that <a id="ref-for-color-scheme①②"></a>color scheme on a particular subtree. The <a id="ref-for-propdef-color-scheme⑨"></a>[color-scheme](#propdef-color-scheme) property allows this, manually setting the <a id="ref-for-element-color-scheme②"></a>[element color scheme](#element-color-scheme) for an element and its descendants. Values are defined as follows:

<strong><a id="valdef-color-scheme-normal"></a><strong>normal</strong></strong>

The <a id="ref-for-element-color-scheme③"></a>[element color scheme](#element-color-scheme) is the same as the <a id="ref-for-page-color-scheme①①"></a>[page color scheme](#page-color-scheme). (This includes noting whether the <a id="ref-for-color-scheme①③"></a>[color scheme](#color-scheme) was <a id="ref-for-color-scheme-default"></a>[defaulted](#color-scheme-default).)

<strong><a id="valdef-color-scheme-light"></a><strong>light</strong></strong>

<strong><a id="valdef-color-scheme-dark"></a><strong>dark</strong></strong>

Indicates that the element supports a <a id="ref-for-light-color-scheme③"></a>[light](#light-color-scheme) and/or <a id="ref-for-dark-color-scheme①"></a>[dark](#dark-color-scheme) color scheme, as appropriate. The element’s <a id="ref-for-color-scheme-support②"></a>[color scheme support](#color-scheme-support) will include the keywords, in the order they’re specified.

<strong><a id="valdef-color-scheme-only"></a><strong>only</strong></strong>

Forbids the user agent from <a id="ref-for-override-the-color-scheme"></a>[overriding the color scheme](#override-the-color-scheme) for the element. The element’s <a id="ref-for-color-scheme-support③"></a>[color scheme support](#color-scheme-support) will have a flag indicating sole support.

<strong><a id="valdef-color-scheme-custom-ident"></a><strong><a id="ref-for-identifier-value①"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)</strong></strong>

<a id="ref-for-identifier-value②"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) values are meaningless, and exist only for future compatibility, so that future added color schemes do not invalidate the <a id="ref-for-propdef-color-scheme①⓪"></a>[color-scheme](#propdef-color-scheme) declaration in legacy user agents. User agents <em>must not</em> interpret any <a id="ref-for-identifier-value③"></a>\<custom-ident\> values as having a meaning; any additional recognized color schemes must be explicitly added to this property’s grammar.

Note: To avoid confusion, authoring tutorials and references should omit <a id="ref-for-identifier-value④"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) from their materials.

The <a id="ref-for-valdef-color-scheme-normal①"></a>[normal](#valdef-color-scheme-normal), <a id="ref-for-valdef-color-scheme-light"></a>[light](#valdef-color-scheme-light), <a id="ref-for-valdef-color-scheme-dark"></a>[dark](#valdef-color-scheme-dark), and <a id="ref-for-valdef-color-scheme-only"></a>[only](#valdef-color-scheme-only) keywords are not valid <a id="ref-for-identifier-value⑤"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)s in this property.

If an element has any value other than <a id="ref-for-propdef-color-scheme①①"></a>[color-scheme: normal](#propdef-color-scheme), its <a id="ref-for-element-color-scheme④"></a>[element color scheme](#element-color-scheme) is determined by finding the <a id="ref-for-used-color-scheme①"></a>[used color scheme](#used-color-scheme), given the user’s <a id="ref-for-preferred-color-scheme③"></a>[preferred color scheme](#preferred-color-scheme) and the element’s <a id="ref-for-color-scheme-support④"></a>[color scheme support](#color-scheme-support) specified by this property.

<a id="example-08dce58a"></a>

<strong>Example:</strong>

[](#example-08dce58a) A page might be generally capable of handling multiple color schemes, while still having a sub-section that needs to be rendered in a particular color scheme.

For example, a style guide might give several UI examples that are using light or dark colors, showing off the light or dark theme specifically. This can be indicated as:

``` text
<meta name=color-scheme content="light dark">
<style>
.light-theme-example {
  color-scheme: light;
}

.dark-theme-example {
  color-scheme: dark;
}
</style>
```

Only the subsections rooted at .light-theme-example or .dark-theme-example will be opted into the <a id="ref-for-light-color-scheme④"></a>[light](#light-color-scheme) or <a id="ref-for-dark-color-scheme②"></a>[dark](#dark-color-scheme) themes specifically; the rest of the page will respect the user’s preference.

Note: Repeating a keyword, such as <a id="ref-for-propdef-color-scheme①②"></a>[color-scheme: light light](#propdef-color-scheme), is valid but has no additional effect beyond what the first instance of the keyword provides.

### <a id="color-scheme-resolution"></a>2.3. Resolving Color Schemes[](#color-scheme-resolution)

While it’s important for a page to respect the user’s preferred <a id="ref-for-color-scheme①④"></a>[color scheme](#color-scheme), it’s also important to only use a <a id="ref-for-color-scheme①⑤"></a>color scheme the page author <em>expects</em>, or else it is very easy to render a page unreadable by causing the foreground and background elements to accidentally have minimal (or zero!) contrast.

For example, an author expecting the default light color scheme might set the page background to a light cream while leaving the text color unset, expecting it to use the default black text color. If the page unexpectedly opted into a user’s preferred <a id="ref-for-dark-color-scheme③"></a>[dark color scheme](#dark-color-scheme), however, and adjusted the default text color to white, that would make the text virtually unreadable.

To balance these competing concerns, a user’s <a id="ref-for-color-scheme-preference①"></a>[color scheme preference](#color-scheme-preference) is compared against a page’s <a id="ref-for-color-scheme-support⑤"></a>[color scheme support](#color-scheme-support) to produce a used <a id="ref-for-color-scheme①⑥"></a>[color scheme](#color-scheme).

A <a id="color-scheme-preference"></a><strong>color scheme preference</strong> is either no preference, a <a id="ref-for-color-scheme①⑦"></a>[color scheme](#color-scheme) preference (<a id="ref-for-light-color-scheme⑤"></a>[light](#light-color-scheme) or <a id="ref-for-dark-color-scheme④"></a>[dark](#dark-color-scheme)), or an <a id="color-scheme-overriding-preference"></a><strong>overriding</strong> <a id="ref-for-color-scheme①⑧"></a>color scheme preference.

A <a id="color-scheme-support"></a><strong>color scheme support</strong> is null, or an ordered list of one or more supported <a id="ref-for-color-scheme①⑨"></a>[color schemes](#color-scheme), along with a flag indicating whether or not it <em>solely</em> supports the indicated <a id="ref-for-color-scheme②⓪"></a>color schemes.

To determine the <a id="used-color-scheme"></a><strong>used color scheme</strong>, given a <a id="ref-for-color-scheme-support⑥"></a>[color scheme support](#color-scheme-support) <var>support</var> and a <a id="ref-for-color-scheme-preference②"></a>[color scheme preference](#color-scheme-preference) <var>preference</var>:

1.  If <var>support</var> is null, or none of the <a id="ref-for-color-scheme②①"></a>[color schemes](#color-scheme) in <var>support</var> are supported by the UA, return the UA default <a id="ref-for-color-scheme②②"></a>color scheme, and annotate that this <a id="ref-for-color-scheme②③"></a>color scheme was <a id="ref-for-color-scheme-default①"></a>[defaulted](#color-scheme-default).

2.  If <var>preference</var> is no preference, return the first supported <a id="ref-for-color-scheme②④"></a>[color scheme](#color-scheme) in <var>support</var>.

3.  If <var>preference</var>’s <a id="ref-for-color-scheme②⑤"></a>[color scheme](#color-scheme) is in <var>support</var> and supported by the UA, return that <a id="ref-for-color-scheme②⑥"></a>color scheme.

4.  If <var>support</var> is flagged as sole support, return the first supported <a id="ref-for-color-scheme②⑦"></a>[color scheme](#color-scheme) in <var>support</var>.

5.  If <var>preference</var> is an <a id="ref-for-color-scheme-overriding-preference"></a>[overriding preference](#color-scheme-overriding-preference), return <var>preference</var>’s <a id="ref-for-color-scheme②⑧"></a>[color scheme](#color-scheme). If this <a id="ref-for-color-scheme②⑨"></a>color scheme is used as an <a id="ref-for-element-color-scheme⑤"></a>[element color scheme](#element-color-scheme), also <a id="ref-for-override-the-color-scheme①"></a>[override the color scheme](#override-the-color-scheme) on the element to this <a id="ref-for-color-scheme③⓪"></a>color scheme.

6.  Otherwise, return the first supported <a id="ref-for-color-scheme③①"></a>[color scheme](#color-scheme) in <var>support</var>.

Note: The basic logic here is that, if the page hasn’t opted into color schemes at all (null <var>supports</var>), we go with the safe UA default that it was probably implicitly designed for. If there <em>is</em> a usable <var>supports</var>, and it matches <var>preference</var>, we use the preference. Otherwise <var>preference</var> and <var>supports</var> disagree, and we go back and forth down the preference hierarchy to see who wins: sole support in <var>support</var> wins, then overriding preference in <var>preference</var> wins (and we adjust the element’s colors to make sure this actually <em>works</em>), then <var>support</var> finally wins.

Note: User agents are <strong>not required</strong> to support any particular <a id="ref-for-color-scheme③②"></a>[color scheme](#color-scheme), so only using a single keyword, such as <a id="ref-for-propdef-color-scheme①③"></a>[color-scheme: dark](#propdef-color-scheme), to indicate a required <a id="ref-for-color-scheme③③"></a>color scheme is still not guaranteed to have any effect on the rendering of the element.

Note: User agents might not actually support preference levels in a <a id="ref-for-color-scheme-preference③"></a>[color scheme preferences](#color-scheme-preference). At the time of writing, UAs generally only support an ordinary <a id="ref-for-light-color-scheme⑥"></a>[light](#light-color-scheme) or <a id="ref-for-dark-color-scheme⑤"></a>[dark](#dark-color-scheme) preference, which considerably simplifies the above logic.

### <a id="color-scheme-effect"></a>2.4. Effects of the Used Color Scheme[](#color-scheme-effect)

For all elements, the user agent must match the following to the <a id="ref-for-element-color-scheme⑥"></a>[element color scheme](#element-color-scheme):

- the default colors of scrollbars and other interaction UI

- the default colors of form controls and other "specially-rendered" elements

- the default colors of other browser-provided UI, such as "spellcheck" underlines

- the value of <a id="ref-for-css-system-colors②"></a>[system colors](https://www.w3.org/TR/css-color-4/#css-system-colors)

- the value of the <a id="ref-for-funcdef-light-dark①"></a>[light-dark()](https://www.w3.org/TR/css-color-5/#funcdef-light-dark) function

On the root element, the <a id="ref-for-element-color-scheme⑦"></a>[element color scheme](#element-color-scheme) additionally must affect the surface color of the <a id="ref-for-canvas"></a>[canvas](https://www.w3.org/TR/CSS2/intro.html#canvas), and the viewport’s scrollbars.

In order to preserve expected color contrasts, in the case of embedded documents typically rendered over a transparent <a id="ref-for-canvas①"></a>[canvas](https://www.w3.org/TR/CSS2/intro.html#canvas) (such as provided via an HTML <code><a id="ref-for-the-iframe-element①"></a>[iframe](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element)</code> element), if the <a id="ref-for-element-color-scheme⑧"></a>[element color scheme](#element-color-scheme) of the element and the <a id="ref-for-element-color-scheme⑨"></a>element color scheme of the embedded document’s root element do not match, then the UA must use an opaque <a id="ref-for-canvas②"></a>canvas of the <a id="ref-for-valdef-color-canvas"></a>[Canvas](https://www.w3.org/TR/css-color-4/#valdef-color-canvas) color appropriate to the embedded document’s root element’s <a id="ref-for-element-color-scheme①⓪"></a>element color scheme instead of a transparent canvas. This rule does not apply to documents embedded via elements intended for graphics (such as <code><a id="ref-for-the-img-element①"></a>[img](https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element)</code> elements embedding an SVG document); their canvases remain the default transparent regardless.

Note: Aside from the small list of adjustments given above, user agents generally do not further adjust a page to match the user’s preferred <a id="ref-for-color-scheme③④"></a>[color scheme](#color-scheme), because the chance of accidentally ruining a page is too high. However, when particular color choices are required by the user (for accessibility reasons, for example), more invasive changes might be applied; see [§ 3 Forced Color Palettes](https://drafts.csswg.org/css-color-adjust-1/#forced).

### <a id="color-scheme-override"></a>2.5. Overriding the Color Scheme[](#color-scheme-override)

If the user has indicated an <a id="ref-for-color-scheme-overriding-preference①"></a>[overriding preference](#color-scheme-overriding-preference) for a particular color scheme that the author does not explicitly support, and the author has not disallowed this (by using the <a id="ref-for-valdef-color-scheme-only①"></a>[only](#valdef-color-scheme-only) keyword), the user agent must <a id="override-the-color-scheme"></a><strong>override the color scheme</strong>: in addition to the effects described in [§ 2.4 Effects of the Used Color Scheme](#color-scheme-effect), it must also auto-adjust <em>other</em> colors into this chosen <a id="ref-for-color-scheme③⑤"></a>[color scheme](#color-scheme), such as by inverting their brightness, while preserving any color contrast necessary for readability of the page. In this case, UA may also auto-adjust colors within replaced elements, background images, and other external resources as appropriate.

Note: The specifics of such auto-adjustments are UA-defined, and can differ from UA to UA. But it is not intended to force all colors into a fixed palette, as <a id="ref-for-forced-colors-mode②"></a>[forced colors mode](https://drafts.csswg.org/css-color-adjust-1/#forced-colors-mode) does, only to force all colors on the page to conform to either a <a id="ref-for-dark-color-scheme⑥"></a>[dark](#dark-color-scheme) or <a id="ref-for-light-color-scheme⑦"></a>[light](#light-color-scheme) color scheme.

<a id="example-11746358"></a>

<strong>Example:</strong>

[](#example-11746358) For example, a UA might have a “dark room” mode, which forces all pages into a <a id="ref-for-dark-color-scheme⑦"></a>[dark](#dark-color-scheme) color scheme.

For pages that already support <a id="ref-for-dark-color-scheme⑧"></a>[dark](#dark-color-scheme) color schemes, and have indicated so using the <a id="ref-for-propdef-color-scheme①④"></a>[color-scheme](#propdef-color-scheme) property or <code><a id="ref-for-meta-color-scheme②"></a>[color-scheme](https://html.spec.whatwg.org/multipage/semantics.html#meta-color-scheme)</code> <code><a id="ref-for-meta"></a>[meta](https://html.spec.whatwg.org/multipage/semantics.html#meta)</code> name, this has no effect other than reporting a <a id="ref-for-valdef-media-prefers-color-scheme-dark"></a>[dark](https://www.w3.org/TR/mediaqueries-5/#valdef-media-prefers-color-scheme-dark) value for the <a id="ref-for-descdef-media-prefers-color-scheme④"></a>[prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) <a id="ref-for-media-query②"></a>[media query](https://www.w3.org/TR/mediaqueries-5/#media-query) and selecting a <a id="ref-for-dark-color-scheme⑨"></a>dark <a id="ref-for-used-color-scheme②"></a>[used color scheme](#used-color-scheme).

But for pages that do not explicitly support a <a id="ref-for-dark-color-scheme①⓪"></a>[dark](#dark-color-scheme) color scheme, and have not explicitly forbidden this auto-adjustment by specifying <a id="ref-for-propdef-color-scheme①⑤"></a>[color-scheme: only light](#propdef-color-scheme), this mode triggers auto-adjustment of the page’s colors to <em>force</em> the page to conform to the desired <a id="ref-for-dark-color-scheme①①"></a>dark color scheme.

### <a id="emulate-forced-colors-mode"></a>5.1. Emulate Forced Colors Mode[](#emulate-forced-colors-mode)

Each <a id="ref-for-top-level-traversable"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable) has an associated <a id="emulated-forced-colors-theme-data"></a><strong>emulated forced colors theme data</strong>, which is data representing <code><a id="ref-for-enumdef-forcedcolorsmodeautomationtheme"></a>[ForcedColorsModeAutomationTheme](#enumdef-forcedcolorsmodeautomationtheme)</code>, initially "<code><a id="ref-for-dom-forcedcolorsmodeautomationtheme-none①"></a>[none](#dom-forcedcolorsmodeautomationtheme-none)</code>".

<a id="enumdef-forcedcolorsmodeautomationtheme"></a><a id="dom-forcedcolorsmodeautomationtheme-none"></a><a id="dom-forcedcolorsmodeautomationtheme-light"></a><a id="dom-forcedcolorsmodeautomationtheme-dark"></a>

``` text
enum ForcedColorsModeAutomationTheme {
  "none",
  "light",
  "dark"
};
```

To <a id="set-emulated-forced-colors-theme-data"></a><strong>set emulated forced colors theme data</strong>, given <a id="ref-for-window-navigable"></a>[navigable](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window-navigable) <var>navigable</var> and an <var>emulatedThemeData</var>:

1.  Assert <var>emulatedThemeData</var> is <code><a id="ref-for-enumdef-forcedcolorsmodeautomationtheme①"></a>[ForcedColorsModeAutomationTheme](#enumdef-forcedcolorsmodeautomationtheme)</code>.

2.  Let <var>traversable</var> be <var>navigable</var>’s <a id="ref-for-nav-top"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-top).

3.  If <var>traversable</var> is not null:

    1.  Set <var>traversable</var>’s associated <a id="ref-for-emulated-forced-colors-theme-data①"></a>[emulated forced colors theme data](#emulated-forced-colors-theme-data) to <var>emulatedThemeData</var>.

    2.  UAs must consider this a change that requires style recalculation.

To <a id="get-emulated-forced-colors-theme-data"></a><strong>get emulated forced colors theme data</strong>, given <code><a id="ref-for-enumdef-forcedcolorsmodeautomationtheme②"></a>[ForcedColorsModeAutomationTheme](#enumdef-forcedcolorsmodeautomationtheme)</code> <var>theme</var>:

1.  Let <var>navigable</var> be <var>theme</var>’s <a id="ref-for-concept-relevant-global"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global)’s <a id="ref-for-concept-document-window"></a>[associated Document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window)’s <a id="ref-for-node-navigable"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable).

2.  If <var>navigable</var> is null, return null.

3.  Let <var>traversable</var> be <var>navigable</var>’s <a id="ref-for-nav-top①"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-top).

4.  If <var>traversable</var> is null, return null.

5.  Return <var>traversable</var>’s associated <a id="ref-for-emulated-forced-colors-theme-data②"></a>[emulated forced colors theme data](#emulated-forced-colors-theme-data).
