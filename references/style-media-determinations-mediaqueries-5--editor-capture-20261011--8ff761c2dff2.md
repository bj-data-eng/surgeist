Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [Media Queries Level 5](https://drafts.csswg.org/mediaqueries-5/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-11 UTC (2026-10-10 America/New_York). Source status: W3C Working Draft, 8 October 2026. Source edition and linked dated publication remain below.

Full captured HTML SHA-256: `6985bff472a17f32b4eefe21273a689f7fcbe9f6334cefc6541c9b5d472cd626` (655756 bytes). Conversion input SHA-256: `8ff761c2dff2251a4aafc585b61e5cb2a16a1c7f746ffb63f4a3558cd6622274` (112873 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained complete heading sections: `environment-blending`, `mf-user-preferences`, `script-control-user-prefs`, bounded at the next equal-or-higher heading, plus original header/legal notice. Other clauses remain upstream. Conversion uses existing HTML preparation, Pandoc 3.1.11.1 and semantic Lua filter. Scripts and styles are omitted without execution. Original IDs, prose, links, literal blocks, list nesting and table contents are verified against the conversion input. Any synthetic table headers or span expansion are representation changes. The unchanged malformed clearOverride list remains visible; Surgeist's interpretation is recorded in its issue contract separately.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>Media Queries Level 5

<a id="w3c-state"></a>[W3C Working Draft](https://www.w3.org/standards/types/#WD), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://www.w3.org/TR/2026/WD-mediaqueries-5-20261008/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/mediaqueries-5/>

<strong>Editor's Draft:</strong>

<https://drafts.csswg.org/mediaqueries-5/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2021/WD-mediaqueries-5-20211218/>

<https://www.w3.org/TR/2020/WD-mediaqueries-5-20200731/>

<https://www.w3.org/TR/2020/WD-mediaqueries-5-20200715/>

<https://www.w3.org/TR/2020/WD-mediaqueries-5-20200603/>

<https://www.w3.org/TR/2020/WD-mediaqueries-5-20200318/>

<https://www.w3.org/TR/2020/WD-mediaqueries-5-20200303/>

<strong>History:</strong>

<https://www.w3.org/standards/history/mediaqueries-5/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/mediaqueries-5)

[Inline In Spec](https://drafts.csswg.org/mediaqueries-5/#issues-index)

<strong>Editors:</strong>

[Florian Rivoal](https://florian.rivoal.net) (Invited Expert)

[Tab Atkins Jr.](http://xanthir.com/contact/) (Google)

Daniel Libby (Microsoft)

[Luke Warlow](https://lukewarlow.dev) (Igalia)

<strong>Former Editor:</strong>

Dean Jackson (Apple)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/mediaqueries-5/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/mediaqueries/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

### <a id="environment-blending"></a>5.5. <a id="mf-environment"></a> Detecting the display technology: the <a id="ref-for-descdef-media-environment-blending"></a>[environment-blending](#descdef-media-environment-blending) feature[](#environment-blending)

| Field                   | Definition                                                                                                                                      |
|-------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-media-environment-blending"></a><strong>environment-blending</strong>                                                            |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-media①⑥"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)                                  |
| <strong>Value:</strong> | opaque <a id="ref-for-comb-one①②"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) additive <a id="ref-for-comb-one①③"></a>\| subtractive |
| <strong>Type:</strong>  | discrete                                                                                                                                        |

The <a id="ref-for-descdef-media-environment-blending①"></a>[environment-blending](#descdef-media-environment-blending) media feature is used to query the characteristics of the user’s display so the author can adjust the style of the document. An author might choose to adjust the visuals and/or layout of the page depending on the display technology to increase the appeal or improve legibility.

The following values are valid:

<strong><a id="valdef-media-environment-blending-opaque"></a><strong>opaque</strong></strong>

The document is rendered on an opaque medium, such as a traditional monitor or paper. Black is dark and white is 100% light.

<strong><a id="valdef-media-environment-blending-additive"></a><strong>additive</strong></strong>

The display blends the colors of the canvas with the real world using additive mixing. Black is fully transparent and white is 100% light.

For example: a head-up display in a car.

<strong><a id="valdef-media-environment-blending-subtractive"></a><strong>subtractive</strong></strong>

The display blends the colors of the canvas with the real world using subtractive mixing. White is fully transparent and dark colors have the most contrast.

For example: an LCD display embedded in a bathroom mirror.

<a id="issue-c4eeab17"></a>

<strong>Issue:</strong>

[](#issue-c4eeab17) Is there a need for the <a id="ref-for-valdef-media-environment-blending-subtractive"></a>[subtractive](#valdef-media-environment-blending-subtractive) value?

<a id="example-2182c0e2"></a>

<strong>Example:</strong>

[](#example-2182c0e2)

``` text
body { background-color: white; }
p { color: black; }

@media(environment-blending: additive) {
  body { background-color: black; }
  p { color: white; font-size: 16px; font-weight: 1000; }
}
```

## <a id="mf-user-preferences"></a>12.  User Preference Media Features[](#mf-user-preferences)

### <a id="prefers-reduced-motion"></a>12.1.  Detecting the desire for less motion on the page: the <a id="ref-for-descdef-media-prefers-reduced-motion"></a>[prefers-reduced-motion](#descdef-media-prefers-reduced-motion) feature[](#prefers-reduced-motion)

| Field                   | Definition                                                                                                     |
|-------------------------|----------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-media-prefers-reduced-motion"></a><strong>prefers-reduced-motion</strong>                       |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-media③①"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:</strong> | no-preference <a id="ref-for-comb-one③②"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) reduce         |
| <strong>Type:</strong>  | discrete                                                                                                       |

The <a id="ref-for-descdef-media-prefers-reduced-motion①"></a>[prefers-reduced-motion](#descdef-media-prefers-reduced-motion) media feature is used to detect if the user has requested the system minimize the amount of non-essential motion it uses.

<strong><a id="valdef-media-prefers-reduced-motion-no-preference"></a><strong>no-preference</strong></strong>

Indicates that the user has made no preference known to the system. This keyword value evaluates as false in the <a id="ref-for-boolean-context⑥"></a>[boolean context](https://drafts.csswg.org/mediaqueries-5/#boolean-context).

<strong><a id="valdef-media-prefers-reduced-motion-reduce"></a><strong>reduce</strong></strong>

Indicates that user has notified the system that they prefer an interface that removes or replaces the types of motion-based animation that either trigger discomfort for those with vestibular motion sensitivity, or distraction for those with attention deficits.

### <a id="prefers-reduced-transparency"></a>12.2.  Detecting the desire for reduced transparency on the page: the <a id="ref-for-descdef-media-prefers-reduced-transparency"></a>[prefers-reduced-transparency](#descdef-media-prefers-reduced-transparency) feature[](#prefers-reduced-transparency)

| Field                   | Definition                                                                                                     |
|-------------------------|----------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-media-prefers-reduced-transparency"></a><strong>prefers-reduced-transparency</strong>           |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-media③②"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:</strong> | no-preference <a id="ref-for-comb-one③③"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) reduce         |
| <strong>Type:</strong>  | discrete                                                                                                       |

The <a id="ref-for-descdef-media-prefers-reduced-transparency①"></a>[prefers-reduced-transparency](#descdef-media-prefers-reduced-transparency) media feature is used to detect if the user has requested the system minimize the amount of transparent or translucent layer effects it uses.

<strong><a id="valdef-media-prefers-reduced-transparency-no-preference"></a><strong>no-preference</strong></strong>

Indicates that the user has made no preference known to the system. This keyword value evaluates as false in the <a id="ref-for-boolean-context⑦"></a>[boolean context](https://drafts.csswg.org/mediaqueries-5/#boolean-context).

<strong><a id="valdef-media-prefers-reduced-transparency-reduce"></a><strong>reduce</strong></strong>

Indicates that user has notified the system that they prefer an interface that minimizes the amount of transparent or translucent layer effects.

<a id="issue-34934443"></a>

<strong>Issue:</strong>

[](#issue-34934443) How does this interact with preferences around e.g. pattern fills and backgrounds? They’re not about transparency, but they also interfere with shape recognition.

### <a id="prefers-contrast"></a>12.3.  Detecting the desire for increased or decreased color contrast from elements on the page: the <a id="ref-for-descdef-media-prefers-contrast"></a>[prefers-contrast](#descdef-media-prefers-contrast) feature[](#prefers-contrast)

| Field                   | Definition                                                                                                                                                                           |
|-------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-media-prefers-contrast"></a><strong>prefers-contrast</strong>                                                                                                         |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-media③③"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)                                                                       |
| <strong>Value:</strong> | no-preference <a id="ref-for-comb-one③④"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) less <a id="ref-for-comb-one③⑤"></a>\| more <a id="ref-for-comb-one③⑥"></a>\| custom |
| <strong>Type:</strong>  | discrete                                                                                                                                                                             |

The <a id="ref-for-descdef-media-prefers-contrast①"></a>[prefers-contrast](#descdef-media-prefers-contrast) media feature is used to detect if the user has requested more or less contrast in the page. This could be responded to, for example, by adjusting the contrast ratio between adjacent colors, or by changing how much elements stand out visually, such as by adjusting their borders.

<strong><a id="valdef-media-prefers-contrast-no-preference"></a><strong>no-preference</strong></strong>

Indicates that the user has made no preference known to the system. This keyword value evaluates as false in the <a id="ref-for-boolean-context⑧"></a>[boolean context](https://drafts.csswg.org/mediaqueries-5/#boolean-context).

<strong><a id="valdef-media-prefers-contrast-less"></a><strong>less</strong></strong>

Indicates that user has notified the system that they prefer an interface that has a lower level of contrast.

<strong><a id="valdef-media-prefers-contrast-more"></a><strong>more</strong></strong>

Indicates that user has notified the system that they prefer an interface that has a higher level of contrast.

<strong><a id="valdef-media-prefers-contrast-custom"></a><strong>custom</strong></strong>

Indicates that the user has indicated wanting a specific set of colors to be used, but the contrast implied by these particular colors is such that neither <a id="ref-for-valdef-media-prefers-contrast-more"></a>[more](#valdef-media-prefers-contrast-more) nor <a id="ref-for-valdef-media-prefers-contrast-less"></a>[less](#valdef-media-prefers-contrast-less) match.

Note: This value will match for users of <a id="ref-for-forced-colors-mode"></a>[forced colors mode](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-mode) who have picked a palette that is neither particularly high nor low contrast. See [§ 12.4 Detecting Forced Colors Mode: the forced-colors feature](#forced-colors).

<a id="example-fd710249"></a>

<strong>Example:</strong>

[](#example-fd710249) A user calling for cyan text over a rust background is not—​at least in terms of luminosity—​expressing a need for particularly high or low contrast, but this is not a lack of a preference either.

Note: Authors can respond to specific user preferences for more or less contrast using <a id="ref-for-descdef-media-prefers-contrast②"></a>[prefers-contrast: more](#descdef-media-prefers-contrast) or <a id="ref-for-descdef-media-prefers-contrast③"></a>prefers-contrast: less, as appropriate.

Using an unqualified <code>@media (prefers-contrast) { … }</code> to apply high contrast styles is incorrect and user-hostile, as it would also impose high contrast styles to people who have requested the exact opposite.

However, it is also common to reduce visual clutter and color complexity in response to both high and low contrast preferences. In that case, it is appropriate to use <code>@media (prefers-contrast) { … }</code> without specifying <a id="ref-for-valdef-media-prefers-contrast-more①"></a>[more](#valdef-media-prefers-contrast-more) or <a id="ref-for-valdef-media-prefers-contrast-less①"></a>[less](#valdef-media-prefers-contrast-less), to do things like replacing background images with plain colors, turning off decorative gradients, or replacing border images or box shadows with simple solid borders. As <a id="ref-for-descdef-media-prefers-contrast④"></a>[prefers-contrast: custom](#descdef-media-prefers-contrast)—​like <a id="ref-for-descdef-media-prefers-contrast⑤"></a>prefers-contrast: more or <a id="ref-for-descdef-media-prefers-contrast⑥"></a>prefers-contrast: less—​evaluates to true in a <a id="ref-for-boolean-context⑨"></a>[boolean context](https://drafts.csswg.org/mediaqueries-5/#boolean-context), such simplifications would also benefit users of <a id="ref-for-forced-colors-mode①"></a>[forced colors mode](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-mode), even when their colors of choice do not result in a particularly high or low contrast. This is desirable, as the reduced palette enforced by <a id="ref-for-forced-colors-mode②"></a>forced colors mode calls for some visual simplification of the page.

<a id="example-e956a9d7"></a>

<strong>Example:</strong>

[](#example-e956a9d7) Preference for more or less contrast may arise from a variety of different situations. Here are some examples:

- Many users have difficulty reading text that has a small difference in contrast to the text background and would prefer a larger contrast.
- People suffering from migraine may find strongly contrasting pages to be visually painful and would prefer a low contrast.
- Some people with dyslexia find high contrast text hard to read, as they feel that the letters shine / sparkle as if backlit by too bright a light, and find low contrast to be more comfortable.
- Environmental factors may also lead a user to prefer more or less contrast. See also [§ 12.7 Automatic handling of User Preferences](#auto-pref).

<a id="issue-819b41af"></a>

<strong>Issue:</strong>

[](#issue-819b41af) This list should be refined and expanded.

### <a id="forced-colors"></a>12.4.  Detecting Forced Colors Mode: the <a id="ref-for-descdef-media-forced-colors"></a>[forced-colors](#descdef-media-forced-colors) feature[](#forced-colors)

| Field                   | Definition                                                                                                     |
|-------------------------|----------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-media-forced-colors"></a><strong>forced-colors</strong>                                         |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-media③④"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:</strong> | none <a id="ref-for-comb-one③⑦"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) active                  |
| <strong>Type:</strong>  | discrete                                                                                                       |

<strong><a id="valdef-media-forced-colors-active"></a><strong>active</strong></strong>

Indicates that <a id="ref-for-forced-colors-mode③"></a>[forced colors mode](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-mode) is active: the user agent enforces a user-chosen limited color palette on the page, The UA will provide the color palette to authors through the CSS system color keywords. See [CSS Color Adjustment 1 § 3 Forced Color Palettes](https://www.w3.org/TR/css-color-adjust-1/#forced) for details.

This does

<strong>Advisement:</strong>

<em>not</em> necessarily indicate a preference for more contrast. The colors have been forcibly adjusted to match the preference of the user, but that preference can be for less or more contrast, or some other arrangement that is neither particularly low or high contrast.

In addition to <a id="ref-for-descdef-media-forced-colors①"></a>[forced-colors: active](#descdef-media-forced-colors), the user agent must also match one of <a id="ref-for-descdef-media-prefers-contrast⑦"></a>[prefers-contrast: more](#descdef-media-prefers-contrast) or <a id="ref-for-descdef-media-prefers-contrast⑧"></a>prefers-contrast: less if it can determine that the forced color palette chosen by the user has a particularly high or low contrast, and must make <a id="ref-for-descdef-media-prefers-contrast⑨"></a>prefers-contrast: custom match otherwise.

Similarly, if the forced color palette chosen by the user fits within one of the color schemes described by <a id="ref-for-descdef-media-prefers-color-scheme"></a>[prefers-color-scheme](#descdef-media-prefers-color-scheme), the corresponding value must also match.

<strong><a id="valdef-media-forced-colors-none"></a><strong>none</strong></strong>

<a id="ref-for-forced-colors-mode④"></a>[Forced colors mode](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-mode) is not active.

<a id="example-44ff2aad"></a>

<strong>Example:</strong>

[](#example-44ff2aad) When <a id="ref-for-forced-colors-mode⑤"></a>[forced colors mode](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-mode) is active, the only colors that are available to the author are <a id="ref-for-css-system-colors"></a>[system colors](https://www.w3.org/TR/css-color-4/#css-system-colors). The user agent will enforce this limited palette automatically, but the author may choose a different way of using these colors, using the forced-colors media feature to detect when it is appropriate to do so.

### <a id="prefers-color-scheme"></a>12.5.  Detecting the desire for light or dark color schemes: the <a id="ref-for-descdef-media-prefers-color-scheme①"></a>[prefers-color-scheme](#descdef-media-prefers-color-scheme) feature[](#prefers-color-scheme)

| Field                   | Definition                                                                                                     |
|-------------------------|----------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-media-prefers-color-scheme"></a><strong>prefers-color-scheme</strong>                           |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-media③⑤"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:</strong> | light <a id="ref-for-comb-one③⑧"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) dark                   |
| <strong>Type:</strong>  | discrete                                                                                                       |

The <a id="ref-for-descdef-media-prefers-color-scheme②"></a>[prefers-color-scheme](#descdef-media-prefers-color-scheme) media feature reflects the user’s desire that the page use a <a id="ref-for-light-color-scheme"></a>[light](https://www.w3.org/TR/css-color-adjust-1/#light-color-scheme) or <a id="ref-for-dark-color-scheme"></a>[dark color scheme](https://www.w3.org/TR/css-color-adjust-1/#dark-color-scheme).

<strong><a id="valdef-media-prefers-color-scheme-light"></a><strong>light</strong></strong>

Indicates that the <a id="ref-for-page-color-scheme"></a>[page color scheme](https://drafts.csswg.org/css-color-adjust-1/#page-color-scheme) is a <a id="ref-for-light-color-scheme①"></a>[light color scheme](https://www.w3.org/TR/css-color-adjust-1/#light-color-scheme) (dark text on light background).

<strong><a id="valdef-media-prefers-color-scheme-dark"></a><strong>dark</strong></strong>

Indicates that the <a id="ref-for-page-color-scheme①"></a>[page color scheme](https://drafts.csswg.org/css-color-adjust-1/#page-color-scheme) is a <a id="ref-for-dark-color-scheme①"></a>[dark color scheme](https://www.w3.org/TR/css-color-adjust-1/#dark-color-scheme) (light text on dark background).

Note: The values for this feature might be expanded in the future (to express preferences for other types of color schemes like "sepia"). As such, the most future-friendly way to use this media feature is by negation such as (prefers-color-scheme: dark) and (not (prefers-color-scheme: dark)), which ensures that new values fall into at least one of the styling blocks.

The method by which the user expresses their preference can vary. It might be a system-wide setting exposed by the Operating System, or a setting controlled by the user agent.

Note: User preferences can also vary by medium. For example, a user may prefer dark themes on a glowing screen, but light themes when printing (to save ink and/or because inked text on blank paper prints better than blank letterforms knocked out of an inked background). UAs are expected to take such variances into consideration so that <a id="ref-for-descdef-media-prefers-color-scheme③"></a>[prefers-color-scheme](#descdef-media-prefers-color-scheme) reflects preferences appropriate to the medium rather than preferences taken out of context.

#### <a id="ua-prefers-color-scheme"></a>12.5.1.  Detecting Specifically the User’s Preferred Color Scheme: the <a id="ref-for-descdef-media-ua-color-scheme"></a>[ua-color-scheme](#descdef-media-ua-color-scheme) feature[](#ua-prefers-color-scheme)

| Field                   | Definition                                                                                                     |
|-------------------------|----------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-media-ua-color-scheme"></a><strong>ua-color-scheme</strong>                                     |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-media③⑥"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:</strong> | light <a id="ref-for-comb-one③⑨"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) dark                   |
| <strong>Type:</strong>  | discrete                                                                                                       |

The <a id="ref-for-descdef-media-prefers-color-scheme④"></a>[prefers-color-scheme](#descdef-media-prefers-color-scheme) media query reflects the <a id="ref-for-page-color-scheme②"></a>[page color scheme](https://drafts.csswg.org/css-color-adjust-1/#page-color-scheme), which will generally reflect a user’s <a id="ref-for-preferred-color-scheme"></a>[preferred color scheme](https://www.w3.org/TR/css-color-adjust-1/#preferred-color-scheme), but might not if the page overrides that. (This is often done at the user’s behest, with a page-specific light/dark-mode toggle to implement a user’s desire for a particular page or origin to render light or dark, regardless of their <em>general</em> color scheme preference.) Also, embedded documents such as iframes automatically match the color scheme of the parent document rather than the user’s preference.

There are some use-cases for nevertheless detecting the user’s <em>actual</em> general preference, and the <a id="ref-for-descdef-media-ua-color-scheme①"></a>[ua-color-scheme](#descdef-media-ua-color-scheme) <a id="ref-for-media-feature③⑦"></a>[media feature](https://drafts.csswg.org/mediaqueries-5/#media-feature) reflects that, ignoring the effect of anything on the page, the effects of being an embedded document, etc.

<strong><a id="valdef-media-ua-color-scheme-light"></a><strong>light</strong></strong>

Indicates that the user’s <a id="ref-for-preferred-color-scheme①"></a>[preferred color scheme](https://www.w3.org/TR/css-color-adjust-1/#preferred-color-scheme) is a <a id="ref-for-light-color-scheme②"></a>[light color scheme](https://www.w3.org/TR/css-color-adjust-1/#light-color-scheme) (dark text on light background), or that the user has no preference.

(If the user has not expressed an active preference, they will also receive the "web default" of a light theme).

<strong><a id="valdef-media-ua-color-scheme-dark"></a><strong>dark</strong></strong>

Indicates that the user’s <a id="ref-for-preferred-color-scheme②"></a>[preferred color scheme](https://www.w3.org/TR/css-color-adjust-1/#preferred-color-scheme) is a <a id="ref-for-dark-color-scheme②"></a>[dark color scheme](https://www.w3.org/TR/css-color-adjust-1/#dark-color-scheme) (light text on dark background).

This feature, like the other

<strong>Note:</strong>

prefers-\* features, previously had a no-preference value to indicate a user not expressing an active preference. However, user agents converged on expressing the "default" behavior as a <a id="ref-for-light-color-scheme③"></a>[light color scheme](https://www.w3.org/TR/css-color-adjust-1/#light-color-scheme) preference, and never matching no-preference.

If a future user agent wishes to expose a difference between "no preference" and "really wants a light display", please contact the CSSWG to discuss this.

<a id="issue-d51acf44"></a>

<strong>Issue:</strong>

[](#issue-d51acf44) The CSSWG has not officially decided on a name for this feature, and so it might change in the future.

### <a id="prefers-reduced-data"></a>12.6.  Detecting the desire for reduced data usage when loading a page: the <a id="ref-for-descdef-media-prefers-reduced-data"></a>[prefers-reduced-data](#descdef-media-prefers-reduced-data) feature[](#prefers-reduced-data)

<a id="issue-c3553cd5"></a>

<strong>Issue:</strong>

[](#issue-c3553cd5) This feature may be an undesired source of fingerprinting, with a bias towards low income with limited data. [\[Issue \#10076\]](https://github.com/w3c/csswg-drafts/issues/10076)

| Field                   | Definition                                                                                                     |
|-------------------------|----------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-media-prefers-reduced-data"></a><strong>prefers-reduced-data</strong>                           |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-media③⑦"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) |
| <strong>Value:</strong> | no-preference <a id="ref-for-comb-one④⓪"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) reduce         |
| <strong>Type:</strong>  | discrete                                                                                                       |

The <a id="ref-for-descdef-media-prefers-reduced-data①"></a>[prefers-reduced-data](#descdef-media-prefers-reduced-data) media feature is used to detect if the user has a preference for being served alternate content that uses less data for the page to be rendered.

<strong><a id="valdef-media-prefers-reduced-data-no-preference"></a><strong>no-preference</strong></strong>

Indicates that the user has made no preference known to the system. This keyword value evaluates as false in the <a id="ref-for-boolean-context①⓪"></a>[boolean context](https://drafts.csswg.org/mediaqueries-5/#boolean-context).

<strong><a id="valdef-media-prefers-reduced-data-reduce"></a><strong>reduce</strong></strong>

Indicates that user has expressed the preference for lightweight alternate content.

The method by which the user expresses their preference can vary. It might be a system-wide setting exposed by the Operating System, or a setting controlled by the user agent. User agents may consider setting this based on the same user or system preference as they use to set the [Save-Data](https://wicg.github.io/savedata/) HTTP request header.

Note: User agents are encouraged to use their own user centered discretion when handling a toggle of this value, whether it’s toggled post page load or during page load. A primary goal could be to not download unnecessary data. Consider, if a page is already loaded with high quality assets and the user changes their preference to reduced, the page could perhaps not update the document right away, instead wait for an explicit page reload invocation from the user. Consider also, if a page is taking a long time to download and a user changes their preference to reduced, this could be a nice point to save the user’s bandwidth and immediately switch to downloading smaller assets. User agents are free to implement logic as they see appropriate for situations like these, and also ought to be aware the situations exist.

<a id="example-b1599633"></a>

<strong>Example:</strong>

[](#example-b1599633) For example, a site could honour the preference of a user who has turned on data-saving mode by serving a smaller image.

``` text
.image {
  background-image: url("images/heavy.jpg");
}

@media (prefers-reduced-data: reduce) {
  /* Save-Data: On */
  .image {
    background-image: url("images/light.jpg");
  }
}
```

### <a id="auto-pref"></a>12.7.  Automatic handling of User Preferences[](#auto-pref)

User agents may have explicit settings allowing users to indicate their preferences or may make the determination based on settings in the underlying operating system. User agents may also automatically infer the preferences of the user based on knowledge about the device, the environment, etc. In such case, it is recommended that they also offer a way for users to opt out of or override the automatically determined preferences.

<a id="example-e4f5c435"></a>

<strong>Example:</strong>

[](#example-e4f5c435) In addition to allowing users to explicitly choose between a preference for a <a id="ref-for-valdef-media-prefers-color-scheme-light"></a>[light](#valdef-media-prefers-color-scheme-light) or <a id="ref-for-valdef-media-prefers-color-scheme-dark"></a>[dark](#valdef-media-prefers-color-scheme-dark) color scheme, a user agent could have a mode where the determination is automatically made based on the current time, expressing a preference for <a id="ref-for-valdef-media-prefers-color-scheme-dark①"></a>dark between sunset and dawn.

<a id="example-ed3f92b7"></a>

<strong>Example:</strong>

[](#example-ed3f92b7) Depending on the type of display used, changes in the ambient light level may make the reading experience difficult or uncomfortable.

For instance, liquid crystal displays can be washed out and very hard to read in brightly lit environments. A device with such a screen and with an ambient light sensor could automatically switch <a id="ref-for-descdef-media-prefers-contrast①⓪"></a>[prefers-contrast](#descdef-media-prefers-contrast) to <a id="ref-for-valdef-media-prefers-contrast-more②"></a>[more](#valdef-media-prefers-contrast-more) when it detects conditions that would make the screen difficult to read. A user agent on a device with an e-ink display would not make the same adjustment, as such displays remain readable in bright daylight.

In the opposite situation, user agents running of device with a light-emitting screen (LCD, OLED, etc.) and an ambient light sensor could automatically switch <a id="ref-for-descdef-media-prefers-contrast①①"></a>[prefers-contrast](#descdef-media-prefers-contrast) to <a id="ref-for-valdef-media-prefers-contrast-less②"></a>[less](#valdef-media-prefers-contrast-less) and <a id="ref-for-descdef-media-prefers-color-scheme⑤"></a>[prefers-color-scheme](#descdef-media-prefers-color-scheme) to <a id="ref-for-valdef-media-prefers-color-scheme-dark②"></a>[dark](#valdef-media-prefers-color-scheme-dark) when used in a dim environment where excessive contrast and brightness would be distracting or uncomfortable to the reader.

<a id="example-b48ea7e3"></a>

<strong>Example:</strong>

[](#example-b48ea7e3) A user agent could automatically switch between <a id="ref-for-descdef-media-prefers-reduced-data②"></a>[prefers-reduced-data: no-preference](#descdef-media-prefers-reduced-data) and <a id="ref-for-valdef-media-prefers-reduced-data-reduce"></a>[reduce](#valdef-media-prefers-reduced-data-reduce) depending on whether the network connection in use allows for unlimited data or is on a metered plan.

## <a id="script-control-user-prefs"></a>13.  Script Control of User Preferences[](#script-control-user-prefs)

It is common for website authors to want to respect the user’s system preferences while also allowing those preferences to be overridden. To help with this, this specification defines a way for authors to override the [§ 12 User Preference Media Features](#mf-user-preferences) using the <code><a id="ref-for-preferencemanager"></a>[PreferenceManager](#preferencemanager)</code> interface.

This override allows the preference to integrate with various platform features that are affected by these preferences.

### <a id="navigator-interface"></a>13.1.  Extensions to the <code><a id="ref-for-navigator"></a>[Navigator](https://html.spec.whatwg.org/multipage/system-state.html#navigator)</code> interface[](#navigator-interface)

<a id="ref-for-Exposed①"></a><a id="ref-for-SecureContext"></a><a id="ref-for-navigator①"></a><a id="ref-for-SameObject"></a><a id="ref-for-preferencemanager①"></a><a id="dom-navigator-preferences"></a>

``` text
[Exposed=Window, SecureContext]
partial interface Navigator {
	[SameObject] readonly attribute PreferenceManager preferences;
};
```

#### <a id="preferences-attribute"></a>13.1.1.  <code><a id="ref-for-dom-navigator-preferences"></a>[preferences](#dom-navigator-preferences)</code> attribute[](#preferences-attribute)

When getting the <code><a id="ref-for-dom-navigator-preferences①"></a>[preferences](#dom-navigator-preferences)</code> attribute always return the same instance of the <code><a id="ref-for-preferencemanager②"></a>[PreferenceManager](#preferencemanager)</code> object.

#### <a id="preference-manager"></a>13.1.2. <code><a id="ref-for-preferencemanager③"></a>[PreferenceManager](#preferencemanager)</code> interface[](#preference-manager)

<a id="ref-for-Exposed②"></a><a id="ref-for-SecureContext①"></a><a id="preferencemanager"></a><a id="ref-for-preferenceobject"></a><a id="dom-preferencemanager-colorscheme"></a><a id="ref-for-preferenceobject①"></a><a id="dom-preferencemanager-contrast"></a><a id="ref-for-preferenceobject②"></a><a id="dom-preferencemanager-reducedmotion"></a><a id="ref-for-preferenceobject③"></a><a id="dom-preferencemanager-reducedtransparency"></a><a id="ref-for-preferenceobject④"></a><a id="dom-preferencemanager-reduceddata"></a>

``` text
[Exposed=Window, SecureContext]
interface PreferenceManager {
	readonly attribute PreferenceObject colorScheme;
	readonly attribute PreferenceObject contrast;
	readonly attribute PreferenceObject reducedMotion;
	readonly attribute PreferenceObject reducedTransparency;
	readonly attribute PreferenceObject reducedData;
};
```

#### <a id="color-scheme-attribute"></a>13.1.3.  <code><a id="ref-for-dom-preferencemanager-colorscheme"></a>[colorScheme](#dom-preferencemanager-colorscheme)</code> attribute[](#color-scheme-attribute)

The <code><a id="ref-for-dom-preferencemanager-colorscheme①"></a>[colorScheme](#dom-preferencemanager-colorscheme)</code> attribute is a <code><a id="ref-for-preferenceobject⑤"></a>[PreferenceObject](#preferenceobject)</code> used to override the user’s preference for the color scheme of the site. This is modeled after the [§ 12.5 Detecting the desire for light or dark color schemes: the prefers-color-scheme feature](#prefers-color-scheme).

The <a id="get-valid-values-for-colorscheme"></a><strong>get valid values for colorScheme</strong> algorithm, when invoked, must run these steps:

1.  Let <var>validValues</var> be a new empty <a id="ref-for-idl-sequence"></a>[sequence](https://webidl.spec.whatwg.org/#idl-sequence).

2.  Add "light" to <var>validValues</var>.

3.  Add "dark" to <var>validValues</var>.

4.  Return <var>validValues</var>.

If an override is set for this preference:

- The user agent MUST use this override for the [§ 12.5 Detecting the desire for light or dark color schemes: the prefers-color-scheme feature](#prefers-color-scheme) in all stylesheets applied to an <a id="ref-for-concept-document-origin"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin) including the UA style sheet.

- The user agent MUST also use this override when queried via \`matchMedia()\` from [CSSOM View § 4 Extensions to the Window Interface](https://www.w3.org/TR/cssom-view-1/#extensions-to-the-window-interface).

- The user agent MUST also use this override when calculating the <a id="ref-for-used-color-scheme"></a>[used color scheme](https://www.w3.org/TR/css-color-adjust-1/#used-color-scheme).

- The user agent MUST also use this override when sending [User Preference Media Features Client Hints Headers § 2.5 Sec-CH-Prefers-Color-Scheme](https://wicg.github.io/user-preference-media-features-headers/#sec-ch-prefers-color-scheme).

- The user agent MUST also use this override for any UA features that are normally affected by [§ 12.5 Detecting the desire for light or dark color schemes: the prefers-color-scheme feature](#prefers-color-scheme).

#### <a id="contrast-attribute"></a>13.1.4.  <code><a id="ref-for-dom-preferencemanager-contrast"></a>[contrast](#dom-preferencemanager-contrast)</code> attribute[](#contrast-attribute)

The <code><a id="ref-for-dom-preferencemanager-contrast①"></a>[contrast](#dom-preferencemanager-contrast)</code> attribute is a <code><a id="ref-for-preferenceobject⑥"></a>[PreferenceObject](#preferenceobject)</code> used to override the user’s preference for the contrast of the site. This is modeled after the [§ 12.3 Detecting the desire for increased or decreased color contrast from elements on the page: the prefers-contrast feature](#prefers-contrast).

The <a id="get-valid-values-for-contrast"></a><strong>get valid values for contrast</strong> algorithm, when invoked, must run these steps:

1.  Let <var>validValues</var> be a new empty <a id="ref-for-idl-sequence①"></a>[sequence](https://webidl.spec.whatwg.org/#idl-sequence).

2.  Add <a id="ref-for-valdef-media-prefers-contrast-more③"></a>[more](#valdef-media-prefers-contrast-more) to <var>validValues</var>.

3.  Add <a id="ref-for-valdef-media-prefers-contrast-less③"></a>[less](#valdef-media-prefers-contrast-less) to <var>validValues</var>.

4.  Add <a id="ref-for-valdef-media-prefers-contrast-no-preference"></a>[no-preference](#valdef-media-prefers-contrast-no-preference) to <var>validValues</var>.

5.  Return <var>validValues</var>.

If an override is set for this preference:

- The user agent MUST use this override for the [§ 12.3 Detecting the desire for increased or decreased color contrast from elements on the page: the prefers-contrast feature](#prefers-contrast) in all stylesheets applied to an <a id="ref-for-concept-document-origin①"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin) including the UA style sheet.

- The user agent MUST also use this override when queried via \`matchMedia()\` from [CSSOM View § 4 Extensions to the Window Interface](https://www.w3.org/TR/cssom-view-1/#extensions-to-the-window-interface).

- The user agent MUST also use this override when sending [User Preference Media Features Client Hints Headers § 2.3 Sec-CH-Prefers-Contrast](https://wicg.github.io/user-preference-media-features-headers/#sec-ch-prefers-contrast).

- The user agent MUST also use this override for any UA features that are normally affected by [§ 12.3 Detecting the desire for increased or decreased color contrast from elements on the page: the prefers-contrast feature](#prefers-contrast).

Note: Unlike the media feature this preference is NOT able to be set to <a id="ref-for-valdef-media-prefers-contrast-custom"></a>[custom](#valdef-media-prefers-contrast-custom) as this is tightly coupled to the [§ 12.4 Detecting Forced Colors Mode: the forced-colors feature](#forced-colors).

#### <a id="reduced-motion-attribute"></a>13.1.5.  <code><a id="ref-for-dom-preferencemanager-reducedmotion"></a>[reducedMotion](#dom-preferencemanager-reducedmotion)</code> attribute[](#reduced-motion-attribute)

The <code><a id="ref-for-dom-preferencemanager-reducedmotion①"></a>[reducedMotion](#dom-preferencemanager-reducedmotion)</code> attribute is a <code><a id="ref-for-preferenceobject⑦"></a>[PreferenceObject](#preferenceobject)</code> used to override the user’s preference for reduced motion on the site. This is modeled after the [§ 12.1 Detecting the desire for less motion on the page: the prefers-reduced-motion feature](#prefers-reduced-motion).

The <a id="get-valid-values-for-reducedmotion"></a><strong>get valid values for reducedMotion</strong> algorithm, when invoked, must run these steps:

1.  Let <var>validValues</var> be a new empty <a id="ref-for-idl-sequence②"></a>[sequence](https://webidl.spec.whatwg.org/#idl-sequence).

2.  Add <a id="ref-for-valdef-media-prefers-reduced-motion-reduce"></a>[reduce](#valdef-media-prefers-reduced-motion-reduce) to <var>validValues</var>.

3.  Add <a id="ref-for-valdef-media-prefers-reduced-motion-no-preference"></a>[no-preference](#valdef-media-prefers-reduced-motion-no-preference) to <var>validValues</var>.

4.  Return <var>validValues</var>.

If an override is set for this preference:

- The user agent MUST use this override for the [§ 12.1 Detecting the desire for less motion on the page: the prefers-reduced-motion feature](#prefers-reduced-motion) in all stylesheets applied to an <a id="ref-for-concept-document-origin②"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin) including the UA style sheet.

- The user agent MUST also use this override when queried via \`matchMedia()\` from [CSSOM View § 4 Extensions to the Window Interface](https://www.w3.org/TR/cssom-view-1/#extensions-to-the-window-interface).

- The user agent MUST also use this override when sending [User Preference Media Features Client Hints Headers § 2.1 Sec-CH-Prefers-Reduced-Motion](https://wicg.github.io/user-preference-media-features-headers/#sec-ch-prefers-reduced-motion).

- The user agent MUST also use this override for any UA features that are normally affected by [§ 12.1 Detecting the desire for less motion on the page: the prefers-reduced-motion feature](#prefers-reduced-motion).

Note: An example of a UA feature that is affected by this preference could be disabling smooth scrolling, or pausing marquee elements.

#### <a id="reduced-transparency-attribute"></a>13.1.6.  <code><a id="ref-for-dom-preferencemanager-reducedtransparency"></a>[reducedTransparency](#dom-preferencemanager-reducedtransparency)</code> attribute[](#reduced-transparency-attribute)

The <code><a id="ref-for-dom-preferencemanager-reducedtransparency①"></a>[reducedTransparency](#dom-preferencemanager-reducedtransparency)</code> attribute is a <code><a id="ref-for-preferenceobject⑧"></a>[PreferenceObject](#preferenceobject)</code> used to override the user’s preference for reduced transparency on the site. This is modeled after the [§ 12.2 Detecting the desire for reduced transparency on the page: the prefers-reduced-transparency feature](#prefers-reduced-transparency).

The <a id="get-valid-values-for-reducedtransparency"></a><strong>get valid values for reducedTransparency</strong> algorithm, when invoked, must run these steps:

1.  Let <var>validValues</var> be a new empty <a id="ref-for-idl-sequence③"></a>[sequence](https://webidl.spec.whatwg.org/#idl-sequence).

2.  Add <a id="ref-for-valdef-media-prefers-reduced-transparency-reduce"></a>[reduce](#valdef-media-prefers-reduced-transparency-reduce) to <var>validValues</var>.

3.  Add <a id="ref-for-valdef-media-prefers-reduced-transparency-no-preference"></a>[no-preference](#valdef-media-prefers-reduced-transparency-no-preference) to <var>validValues</var>.

4.  Return <var>validValues</var>.

If an override is set for this preference:

- The user agent MUST use this override for the [§ 12.2 Detecting the desire for reduced transparency on the page: the prefers-reduced-transparency feature](#prefers-reduced-transparency) in all stylesheets applied to an <a id="ref-for-concept-document-origin③"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin) including the UA style sheet.

- The user agent MUST also use this override when queried via \`matchMedia()\` from [CSSOM View § 4 Extensions to the Window Interface](https://www.w3.org/TR/cssom-view-1/#extensions-to-the-window-interface).

- The user agent MUST also use this override when sending [User Preference Media Features Client Hints Headers § 2.2 Sec-CH-Prefers-Reduced-Transparency](https://wicg.github.io/user-preference-media-features-headers/#sec-ch-prefers-reduced-transparency).

- The user agent MUST also use this override for any UA features that are normally affected by [§ 12.2 Detecting the desire for reduced transparency on the page: the prefers-reduced-transparency feature](#prefers-reduced-transparency).

#### <a id="reduced-data-attribute"></a>13.1.7.  <code><a id="ref-for-dom-preferencemanager-reduceddata"></a>[reducedData](#dom-preferencemanager-reduceddata)</code> attribute[](#reduced-data-attribute)

The <code><a id="ref-for-dom-preferencemanager-reduceddata①"></a>[reducedData](#dom-preferencemanager-reduceddata)</code> attribute is a <code><a id="ref-for-preferenceobject⑨"></a>[PreferenceObject](#preferenceobject)</code> used to override the user’s preference for reduced data usage on the site. This is modeled after the [§ 12.6 Detecting the desire for reduced data usage when loading a page: the prefers-reduced-data feature](#prefers-reduced-data).

The <a id="get-valid-values-for-reduceddata"></a><strong>get valid values for reducedData</strong> algorithm, when invoked, must run these steps:

1.  Let <var>validValues</var> be a new empty <a id="ref-for-idl-sequence④"></a>[sequence](https://webidl.spec.whatwg.org/#idl-sequence).

2.  Add <a id="ref-for-valdef-media-prefers-reduced-data-reduce①"></a>[reduce](#valdef-media-prefers-reduced-data-reduce) to <var>validValues</var>.

3.  Add <a id="ref-for-valdef-media-prefers-reduced-data-no-preference"></a>[no-preference](#valdef-media-prefers-reduced-data-no-preference) to <var>validValues</var>.

4.  Return <var>validValues</var>.

If an override is set for this preference:

- The user agent MUST use this override for the [§ 12.6 Detecting the desire for reduced data usage when loading a page: the prefers-reduced-data feature](#prefers-reduced-data) in all stylesheets applied to an <a id="ref-for-concept-document-origin④"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin) including the UA style sheet.

- The user agent MUST also use this override when queried via \`matchMedia()\` from [CSSOM View § 4 Extensions to the Window Interface](https://www.w3.org/TR/cssom-view-1/#extensions-to-the-window-interface).

- The user agent MUST also use this override when sending [Save Data API § 2.1.1 Save-Data Request Header Field](https://wicg.github.io/savedata/#save-data-request-header-field).

- The user agent MUST also use this override when calculating the [Save Data API § 2.1 saveData attribute](https://wicg.github.io/savedata/#savedata-attribute).

- The user agent MUST also use this override for any UA features that are normally affected by [§ 12.6 Detecting the desire for reduced data usage when loading a page: the prefers-reduced-data feature](#prefers-reduced-data).

#### <a id="preference-object-interface"></a>13.1.8.  <code><a id="ref-for-preferenceobject①⓪"></a>[PreferenceObject](#preferenceobject)</code> interface[](#preference-object-interface)

<a id="ref-for-Exposed③"></a><a id="ref-for-SecureContext②"></a><a id="preferenceobject"></a><a id="ref-for-eventtarget"></a><a id="ref-for-idl-DOMString"></a><a id="ref-for-dom-preferenceobject-override"></a><a id="ref-for-idl-DOMString①"></a><a id="ref-for-dom-preferenceobject-value"></a><a id="ref-for-idl-frozen-array"></a><a id="ref-for-idl-DOMString②"></a><a id="ref-for-dom-preferenceobject-validvalues"></a><a id="ref-for-idl-undefined"></a><a id="ref-for-dom-preferenceobject-clearoverride"></a><a id="ref-for-idl-promise"></a><a id="ref-for-idl-undefined①"></a><a id="ref-for-dom-preferenceobject-requestoverride"></a><a id="ref-for-idl-DOMString③"></a><a id="dom-preferenceobject-requestoverride-value-value"></a><a id="ref-for-eventhandler"></a><a id="ref-for-dom-preferenceobject-onchange"></a>

``` text
[Exposed=Window, SecureContext]
interface PreferenceObject : EventTarget {
	readonly attribute DOMString? override;
	readonly attribute DOMString value;
	readonly attribute FrozenArray<DOMString> validValues;

	undefined clearOverride();
	Promise<undefined> requestOverride(DOMString? value);

	attribute EventHandler onchange;
};
```

##### <a id="override-attribute"></a>13.1.8.1.  <code><a id="ref-for-dom-preferenceobject-override①"></a>[override](#dom-preferenceobject-override)</code> attribute[](#override-attribute)

The <a id="dom-preferenceobject-override"></a><strong><code>override</code></strong> attribute, when accessed, must run these steps:

1.  Let <var>preference</var> be the preference object’s name.

2.  Let <var>override</var> be null.

3.  If an override for <var>preference</var> exists, set <var>override</var> to the value of that override.

4.  Return <var>override</var>.

##### <a id="preference-value-attribute"></a>13.1.8.2.  <code><a id="ref-for-dom-preferenceobject-value①"></a>[value](#dom-preferenceobject-value)</code> attribute[](#preference-value-attribute)

The <a id="dom-preferenceobject-value"></a><strong><code>value</code></strong> attribute, when accessed, must run these steps:

1.  Let <var>preference</var> be the preference object’s name.

2.  Let <var>value</var> be null.

3.  If an override for <var>preference</var> exists, set <var>value</var> to the value of that override.

4.  If <var>value</var> is null, set <var>value</var> to the UA value of the preference.

5.  Return <var>value</var>.

##### <a id="valid-values-attribute"></a>13.1.8.3.  <code><a id="ref-for-dom-preferenceobject-validvalues①"></a>[validValues](#dom-preferenceobject-validvalues)</code> attribute[](#valid-values-attribute)

The <a id="dom-preferenceobject-validvalues"></a><strong><code>validValues</code></strong> attribute, when accessed, must run these steps:

1.  Let <var>preference</var> be the preference object’s name.
2.  Switch on <var>preference</var>:
    <strong>"<code><a id="ref-for-dom-preferencemanager-colorscheme②"></a>[colorScheme](#dom-preferencemanager-colorscheme)</code>"</strong>

    Return the result of <a id="ref-for-get-valid-values-for-colorscheme"></a>[get valid values for colorScheme](#get-valid-values-for-colorscheme).
    <strong>"<code><a id="ref-for-dom-preferencemanager-contrast②"></a>[contrast](#dom-preferencemanager-contrast)</code>"</strong>

    Return the result of <a id="ref-for-get-valid-values-for-contrast"></a>[get valid values for contrast](#get-valid-values-for-contrast).
    <strong>"<code><a id="ref-for-dom-preferencemanager-reducedmotion②"></a>[reducedMotion](#dom-preferencemanager-reducedmotion)</code>"</strong>

    Return the result of <a id="ref-for-get-valid-values-for-reducedmotion"></a>[get valid values for reducedMotion](#get-valid-values-for-reducedmotion).
    <strong>"<code><a id="ref-for-dom-preferencemanager-reducedtransparency②"></a>[reducedTransparency](#dom-preferencemanager-reducedtransparency)</code>"</strong>

    Return the result of <a id="ref-for-get-valid-values-for-reducedtransparency"></a>[get valid values for reducedTransparency](#get-valid-values-for-reducedtransparency).
    <strong>"<code><a id="ref-for-dom-preferencemanager-reduceddata②"></a>[reducedData](#dom-preferencemanager-reduceddata)</code>"</strong>

    Return the result of <a id="ref-for-get-valid-values-for-reduceddata"></a>[get valid values for reducedData](#get-valid-values-for-reduceddata).

##### <a id="onchange-attribute"></a>13.1.8.4.  <code><a id="ref-for-dom-preferenceobject-onchange①"></a>[onchange](#dom-preferenceobject-onchange)</code> event handler attribute[](#onchange-attribute)

The <a id="dom-preferenceobject-onchange"></a><strong><code>onchange</code></strong> attribute is an <a id="ref-for-event-handler-idl-attributes"></a>[event handler IDL attribute](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-idl-attributes) for the <code><a id="ref-for-dom-preferenceobject-onchange②"></a>[onchange](#dom-preferenceobject-onchange)</code> <a id="ref-for-event-handlers"></a>[event handler](https://html.spec.whatwg.org/multipage/webappapis.html#event-handlers), whose <a id="ref-for-event-handler-event-type"></a>[event handler event type](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-event-type) is <a id="preferenceobject-change"></a><strong>change</strong>.

Whenever the <a id="ref-for-user-agent"></a>[user agent](https://infra.spec.whatwg.org/#user-agent) is aware that the state of a <code><a id="ref-for-preferenceobject①①"></a>[PreferenceObject](#preferenceobject)</code> instance <var>value</var> has changed, it runs the <a id="preferenceobject-preferenceobject-update-steps"></a><strong><code><a id="ref-for-preferenceobject①②"></a>[PreferenceObject](#preferenceobject)</code> update steps</strong>:

1.  Let <var>preference</var> be the <code><a id="ref-for-preferenceobject①③"></a>[PreferenceObject](#preferenceobject)</code> object that <var>value</var> is associated with.

2.  If <a id="ref-for-this"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-relevant-global"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) is a <code><a id="ref-for-window"></a>[Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window)</code> object, then:

    1.  Let <var>document</var> be <var>preference</var>’s <a id="ref-for-concept-relevant-global①"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global)’s <a id="ref-for-concept-document-window"></a>[associated Document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

    2.  If <var>document</var> is null or <var>document</var> is not <a id="ref-for-fully-active"></a>[fully active](https://html.spec.whatwg.org/multipage/document-sequences.html#fully-active), terminate this algorithm.

3.  <a id="ref-for-concept-event-fire"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <code>change</code> at <var>preference</var>.

##### <a id="request-override-method"></a>13.1.8.5.  <code><a id="ref-for-dom-preferenceobject-requestoverride①"></a>[requestOverride()](#dom-preferenceobject-requestoverride)</code> method[](#request-override-method)

The <a id="dom-preferenceobject-requestoverride"></a><strong><code>requestOverride(value)</code></strong> method, when invoked, must run these steps:

1.  Let <var>result</var> be <a id="ref-for-a-new-promise"></a>[a new promise](https://webidl.spec.whatwg.org/#a-new-promise).

2.  Let <var>allowed</var> be false.

3.  Set <var>allowed</var> to the result of executing a UA defined algorithm for deciding whether the request is allowed.

4.  If <var>allowed</var> is false, return <a id="ref-for-a-promise-rejected-with"></a>[a promise rejected with](https://webidl.spec.whatwg.org/#a-promise-rejected-with) a "<code><a id="ref-for-notallowederror"></a>[NotAllowedError](https://webidl.spec.whatwg.org/#notallowederror)</code>" <code><a id="ref-for-idl-DOMException"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

5.  Let <var>value</var> be the method’s argument.

6.  Let <var>result</var> be <a id="ref-for-a-new-promise①"></a>[a new promise](https://webidl.spec.whatwg.org/#a-new-promise).

7.  If <var>value</var> is null or the empty string:

    1.  Run <code><a id="ref-for-dom-preferenceobject-clearoverride①"></a>[clearOverride](#dom-preferenceobject-clearoverride)</code>.

    2.  <a id="ref-for-resolve"></a>[Resolve](https://webidl.spec.whatwg.org/#resolve) and return <var>result</var>.

8.  Let <var>currentValue</var> be the preference object’s <var>value</var>.

9.  Let <var>validValues</var> be null.

10. Switch on <var>preference</var>:

    <strong>"<code><a id="ref-for-dom-preferencemanager-colorscheme③"></a>[colorScheme](#dom-preferencemanager-colorscheme)</code>"</strong>

    Set <var>validValues</var> to the result of <a id="ref-for-get-valid-values-for-colorscheme①"></a>[get valid values for colorScheme](#get-valid-values-for-colorscheme).
    <strong>"<code><a id="ref-for-dom-preferencemanager-contrast③"></a>[contrast](#dom-preferencemanager-contrast)</code>"</strong>

    Set <var>validValues</var> to the result of <a id="ref-for-get-valid-values-for-contrast①"></a>[get valid values for contrast](#get-valid-values-for-contrast).
    <strong>"<code><a id="ref-for-dom-preferencemanager-reducedmotion③"></a>[reducedMotion](#dom-preferencemanager-reducedmotion)</code>"</strong>

    Set <var>validValues</var> to the result of <a id="ref-for-get-valid-values-for-reducedmotion①"></a>[get valid values for reducedMotion](#get-valid-values-for-reducedmotion).
    <strong>"<code><a id="ref-for-dom-preferencemanager-reducedtransparency③"></a>[reducedTransparency](#dom-preferencemanager-reducedtransparency)</code>"</strong>

    Set <var>validValues</var> to the result of <a id="ref-for-get-valid-values-for-reducedtransparency①"></a>[get valid values for reducedTransparency](#get-valid-values-for-reducedtransparency).
    <strong>"<code><a id="ref-for-dom-preferencemanager-reduceddata③"></a>[reducedData](#dom-preferencemanager-reduceddata)</code>"</strong>

    Set <var>validValues</var> to the result of <a id="ref-for-get-valid-values-for-reduceddata①"></a>[get valid values for reducedData](#get-valid-values-for-reduceddata).

11. If <var>value</var> is not in <var>validValues</var>:

    1.  <a id="ref-for-reject"></a>[Reject](https://webidl.spec.whatwg.org/#reject) <var>result</var> with a "<code><a id="ref-for-exceptiondef-typeerror"></a>[TypeError](https://webidl.spec.whatwg.org/#exceptiondef-typeerror)</code>" <code><a id="ref-for-idl-DOMException①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

    2.  Return <var>result</var>.

12. Let <var>previousOverride</var> be null.

13. If an override for <var>preference</var> exists, set <var>previousOverride</var> to the value of that override.

14. If <var>value</var> is different from <var>previousOverride</var>:

    1.  Set the preference override for <var>preference</var> to <var>value</var>.

15. If <var>previousOverride</var> is null, then:

    1.  If <var>value</var> is the same as <var>currentValue</var>, then:

        1.  <a id="ref-for-concept-event-fire①"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <code>change</code> at <a id="ref-for-this①"></a>[this](https://webidl.spec.whatwg.org/#this).

16. <a id="ref-for-resolve①"></a>[Resolve](https://webidl.spec.whatwg.org/#resolve) and return <var>result</var>.

<a id="issue-1e7d2733"></a>

<strong>Issue:</strong>

[](#issue-1e7d2733) This algorithm needs more detail on what exactly setting the preference override does.

<a id="issue-42b17e45"></a>

<strong>Issue:</strong>

[](#issue-42b17e45) Is TypeError correct here?

Note: The \`change\` event is fired when the computed value changes, but when a new override is set it is also fired if the value hasn’t changed.

##### <a id="clear-override-method"></a>13.1.8.6.  <code><a id="ref-for-dom-preferenceobject-clearoverride②"></a>[clearOverride()](#dom-preferenceobject-clearoverride)</code> method[](#clear-override-method)

The <a id="dom-preferenceobject-clearoverride"></a><strong><code>clearOverride()</code></strong> method, when invoked, must run these steps:

1.  Let <var>preference</var> be the preference object’s name.

2.  Let <var>override</var> be null.

3.  If an override for <var>preference</var> exists, set <var>override</var> to the value of that override.

4.  If <var>override</var> is null, then return.

5.  Clear the override for <var>preference</var>.

6.  Let <var>newValue</var> be the preference object’s value.

7.  If <var>newValue</var> is equal to <var>override</var>, then:

8.  <a id="ref-for-concept-event-fire②"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <code>change</code> at <a id="ref-for-this②"></a>[this](https://webidl.spec.whatwg.org/#this).

Note: The \`change\` event is fired when the computed value changes, but when an override is cleared it is also fired if the value hasn’t changed.
