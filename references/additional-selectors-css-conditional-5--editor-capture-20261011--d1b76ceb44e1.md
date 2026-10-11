Attribution and reformatting notice added for Surgeist on 2026-10-11

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Conditional Rules Module Level 5](https://drafts.csswg.org/css-conditional-5/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Full HTML was retrieved 2026-10-10 and reused without refetching; bounded conversion prepared 2026-10-11. Source status: W3C Working Draft, 8 October 2026.

Full captured HTML SHA-256: `3b4a4b4ef336edeaca211cdb90481bfcf9cda1955c1cabce485b98e2fcb497e3` (494912 bytes). Conversion input SHA-256: `d1b76ceb44e12257862409b1bbc55cc4936255d2afb7351d4ef503f4493eef66` (66209 bytes). Page-declared revision: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. Exact [upstream source pin](https://raw.githubusercontent.com/w3c/csswg-drafts/50337328484f7fcdd9ab6fe9ded6011815b1707e/css-conditional-5/Overview.bs) SHA-256: `7fe15fcbc0926a53279c771a02759ab198b3068a0cd65bd14b482a1f53f90496`. The rendered HTML byte hash identifies this capture; the declared revision does not replace it.

Retained scope: `at-supports-ext`, `when-rule`, `else-rule`, `supports-condition-rule`, `the-csssupportscondition-interface`. Each heading section is complete through its subheadings, bounded at the next equal-or-higher heading. The original header and legal notice remain. Other sections remain upstream. Scripts/styles are omitted without execution. Source IDs, prose, links and literal examples are checked against the conversion input; omitted fragments link upstream.

Conversion uses existing `references/tools/html-to-markdown`, Pandoc 3.1.11.1 and semantic Lua filter. Added labels and representation changes are non-normative. The capture preserves source gaps; the separately recorded Surgeist policy does not alter source text.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Conditional Rules Module Level 5

<a id="w3c-state"></a>[W3C Working Draft](https://www.w3.org/standards/types/#WD), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://www.w3.org/TR/2026/WD-css-conditional-5-20261008/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-conditional-5/>

<strong>Editor's Draft:</strong>

<https://drafts.csswg.org/css-conditional-5/>

<strong>History:</strong>

<https://www.w3.org/standards/history/css-conditional-5/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-conditional-5)

[Inline In Spec](https://drafts.csswg.org/css-conditional-5/#issues-index)

<strong>Editors:</strong>

[L. David Baron](https://dbaron.org/) ([Google](https://www.google.com/))

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Chris Lilley](https://svgees.us/) (W3C)

[Miriam E. Suzanne](https://www.miriamsuzanne.com/who/) (Invited Expert)

[Lea Verou](https://lea.verou.me/about) (Invited Expert)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-conditional-5/Overview.bs)

<strong>Delta Spec:</strong>

yes

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-conditional/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="at-supports-ext"></a>2.  Extensions to the <a id="ref-for-at-ruledef-supports②"></a>[&#64;supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rule[](#at-supports-ext)

This level of the specification extends the <a id="ref-for-typedef-supports-feature"></a>[\<supports-feature\>](#typedef-supports-feature) syntax as follows:

<a id="typedef-supports-feature"></a><a id="ref-for-typedef-supports-selector-fn"></a><a id="ref-for-comb-one"></a><a id="ref-for-typedef-supports-font-tech-fn"></a><a id="ref-for-comb-one①"></a><a id="ref-for-typedef-supports-font-format-fn"></a><a id="ref-for-comb-one②"></a><a id="ref-for-typedef-supports-at-rule-fn"></a><a id="ref-for-comb-one③"></a><a id="ref-for-typedef-supports-named-feature-fn"></a><a id="ref-for-comb-one④"></a><a id="ref-for-typedef-supports-env-fn"></a><a id="ref-for-comb-one⑤"></a><a id="ref-for-typedef-supports-decl"></a><a id="typedef-supports-decl"></a><a id="ref-for-typedef-declaration"></a><a id="ref-for-comb-one⑥"></a><a id="ref-for-typedef-supports-condition-name"></a><a id="typedef-supports-font-tech-fn"></a><a id="ref-for-font-tech-values"></a><a id="typedef-supports-font-format-fn"></a><a id="ref-for-font-format-values"></a><a id="typedef-supports-at-rule-fn"></a><a id="ref-for-typedef-at-keyword-token"></a><a id="typedef-supports-named-feature-fn"></a><a id="ref-for-typedef-ident"></a><a id="typedef-supports-env-fn"></a><a id="ref-for-typedef-ident①"></a>

``` text
<supports-feature> = <supports-selector-fn>
                   | <supports-font-tech-fn> | <supports-font-format-fn>
                   | <supports-at-rule-fn> | <supports-named-feature-fn>
                   | <supports-env-fn> | <supports-decl>
<supports-decl> = ( [ <declaration> | <supports-condition-name> ] )
<supports-font-tech-fn> = font-tech( <font-tech> )
<supports-font-format-fn> = font-format( <font-format> )
<supports-at-rule-fn> = at-rule( <at-keyword-token> )
<supports-named-feature-fn> = named-feature( <ident> )
<supports-env-fn> = env( <ident> )
```

<a id="typedef-declaration"></a><strong>\<declaration\></strong> here matches anything that would be successfully parsed by <a id="ref-for-consume-a-declaration"></a>[consume a declaration](https://www.w3.org/TR/css-syntax-3/#consume-a-declaration), ignoring the context-validation check at the end of that algorithm. Notably, this includes a trailing

<strong>Note:</strong>

!important, which is valid but ignored for the purpose of <a id="ref-for-at-ruledef-supports③"></a>[&#64;supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports).

<strong><a id="ref-for-typedef-supports-condition-name①"></a>[\<supports-condition-name\>](#typedef-supports-condition-name)</strong>

The result is true if the UA <a id="ref-for-dfn-supports-condition-name"></a>[supports the named condition](#dfn-supports-condition-name). If the name is not recognized, the result is false.

<strong><a id="funcdef-supports-font-tech"></a><strong>font-tech( <a id="ref-for-font-tech-values①"></a>[\<font-tech\>](https://www.w3.org/TR/css-fonts-4/#font-tech-values) )</strong></strong>

The result is true if the UA <a id="ref-for-dfn-support-font-tech"></a>[supports the font tech](#dfn-support-font-tech) provided as an argument to the function.

<strong><a id="funcdef-supports-font-format"></a><strong>font-format( <a id="ref-for-font-format-values①"></a>[\<font-format\>](https://www.w3.org/TR/css-fonts-4/#font-format-values) )</strong></strong>

The result is true if the UA <a id="ref-for-dfn-support-font-format"></a>[supports the font format](#dfn-support-font-format) provided as an argument to the function.

<strong><a id="funcdef-supports-at-rule"></a><strong>at-rule( <a id="ref-for-typedef-at-keyword-token①"></a>[\<at-keyword-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-at-keyword-token) )</strong></strong>

The result is true if the UA <a id="ref-for-dfn-support-at-rule"></a>[supports the at-rule](#dfn-support-at-rule) provided as an argument to the function.

<strong><a id="funcdef-supports-named-feature"></a><strong>named-feature( <a id="ref-for-typedef-ident②"></a>[\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) )</strong></strong>

The result is true if the UA <a id="ref-for-dfn-support-named-feature"></a>[supports the named feature](#dfn-support-named-feature) provided as an argument to the function.

<strong><a id="funcdef-supports-env"></a><strong>env( <a id="ref-for-typedef-ident③"></a>[\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) )</strong></strong>

The result is true if the UA <a id="ref-for-dfn-support-env"></a>[supports the environment variable](#dfn-support-env) provided as an argument to the function.

### <a id="support-definition-ext"></a>2.1.  Extensions to the definition of support[](#support-definition-ext)

#### <a id="support-definition-ext-fonts"></a>2.1.1.  Font techs and formats[](#support-definition-ext-fonts)

A CSS processor is considered to <a id="dfn-support-font-tech"></a><strong>support a font tech</strong> when it is capable of utilizing the specified [CSS Fonts 4 § 11.1 Font tech](https://www.w3.org/TR/css-fonts-4/#font-tech-definitions) in layout and rendering.

A CSS processor is considered to <a id="dfn-support-font-format"></a><strong>support a font format</strong> when it is capable of utilizing the specified [CSS Fonts 4 § 11.2 Font formats](https://www.w3.org/TR/css-fonts-4/#font-format-definitions) in layout and rendering, and this format is not specified as a <a id="ref-for-string-value"></a>[\<string\>](https://www.w3.org/TR/css-values-4/#string-value).

#### <a id="support-definition-at-rules"></a>2.1.2.  At-rules[](#support-definition-at-rules)

A CSS processor is considered to <a id="dfn-support-at-rule"></a><strong>support an at-rule</strong> if it would accept an <a id="ref-for-at-rule①"></a>[at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule) beginning with the specified at-keyword within any context.

Note: Because <a id="ref-for-at-ruledef-charset"></a>[&#64;charset](https://www.w3.org/TR/css-syntax-3/#at-ruledef-charset) is not a valid <a id="ref-for-at-rule②"></a>[at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule), it is not considered to be supported under this definition.

#### <a id="support-definition-named-features"></a>2.1.3.  Named features[](#support-definition-named-features)

A CSS processor is considered to <a id="dfn-support-named-feature"></a><strong>support a named feature</strong> if it supports the named feature based on the feature definition described in the following list:

<strong><a id="anchor-position-follows-transforms"></a><strong>anchor-position-follows-transforms</strong></strong>

Anchoring to a transformed element automatically takes into account the anchor’s transforms, causing the positioned element to shift to match it. \[CSS-ANCHOR-POSITIONING\]

(An earlier version of the specification did not take transforms into account.)

<strong><a id="single-axis-scroll-container"></a><strong>single-axis-scroll-container</strong></strong>

The ability to have <a id="ref-for-single-axis-scroll-container"></a>[single-axis scroll containers](https://drafts.csswg.org/css-overflow-3/#single-axis-scroll-container), where one axis is scrollable and the other is <a id="ref-for-valdef-overflow-clip"></a>[clip](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-clip). [\[CSS-OVERFLOW-4\]](https://drafts.csswg.org/css-conditional-5/#biblio-css-overflow-4)

(Previously, specifying <a id="ref-for-valdef-overflow-clip①"></a>[clip](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-clip) alongside a scrollable value caused it to compute to <a id="ref-for-valdef-overflow-hidden"></a>[hidden](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-hidden), which is still a scrollable value even if it doesn’t generate a scrollbar.)

If the feature is not listed the processor does not support the named feature.

Note: The CSS Working Group intends to add features to this list rarely, when there is real demand for feature testing something specific and it would be unreasonable to add a more general feature testing mechanism that would cover the use case.

#### <a id="support-definition-supports-condition-name"></a>2.1.4.  Named conditions[](#support-definition-supports-condition-name)

A CSS processor is considered to <a id="dfn-supports-condition-name"></a><strong>support a named condition</strong> when the related <a id="ref-for-named-supports-condition"></a>[named supports condition](#named-supports-condition) returns true.

#### <a id="support-definition-env"></a>2.1.5.  Environment variables[](#support-definition-env)

A CSS processor is considered to <a id="dfn-support-env"></a><strong>support an environment variable</strong> if the <a id="ref-for-typedef-ident④"></a>[\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) is a supported <a id="ref-for-css-environment-variable"></a>[environment variable](https://www.w3.org/TR/css-env-1/#css-environment-variable).

## <a id="when-rule"></a>3.  Generalized Conditional Rules: the <a id="ref-for-at-ruledef-when③"></a>[&#64;when](#at-ruledef-when) rule[](#when-rule)

The <a id="at-ruledef-when"></a><strong>&#64;when</strong> at-rule is a <a id="ref-for-conditional-group-rule"></a>[conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) that generalizes the individual <a id="ref-for-conditional-group-rule①"></a>conditional group rules such as <a id="ref-for-at-ruledef-media"></a>[&#64;media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) and <a id="ref-for-at-ruledef-supports④"></a>[&#64;supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports). It is defined as:

<a id="ref-for-typedef-boolean-condition"></a><a id="ref-for-typedef-rule-list"></a>

``` text
@when <boolean-condition> {
  <rule-list>
}
```

Where <a id="typedef-boolean-condition"></a><strong><a id="ref-for-typedef-boolean-condition①"></a>[\<boolean-condition\>](#typedef-boolean-condition)</strong> is a boolean algebra a la [Media Queries 4 § 3 Syntax](https://www.w3.org/TR/mediaqueries-4/#mq-syntax), but with <a id="ref-for-funcdef-media"></a>[media()](#funcdef-media) and <a id="ref-for-funcdef-supports"></a>[supports()](#funcdef-supports) functions as leaves.

<a id="issue-e3cd55e5"></a>

<strong>Issue:</strong>

[](#issue-e3cd55e5) Define "boolean algebra, with X as leaves" in a generic way in Conditional, so all the conditional rules can reference it directly, rather than having to redefine boolean algebra on their own.

The <a id="ref-for-funcdef-media①"></a>[media()](#funcdef-media) and <a id="ref-for-funcdef-supports①"></a>[supports()](#funcdef-supports) functions are defined as:

<a id="funcdef-media"></a><a id="ref-for-typedef-mf-plain"></a><a id="ref-for-comb-one⑦"></a><a id="ref-for-typedef-mf-boolean"></a><a id="ref-for-comb-one⑧"></a><a id="ref-for-typedef-mf-range"></a><a id="funcdef-supports"></a><a id="ref-for-typedef-declaration①"></a>

``` text
media() = media( [ <mf-plain> | <mf-boolean> | <mf-range> ] )
supports() = supports( <declaration> )
```

A <a id="ref-for-funcdef-media②"></a>[media()](#funcdef-media) or <a id="ref-for-funcdef-supports②"></a>[supports()](#funcdef-supports) function is associated with the boolean result that its contained condition is associated with.

## <a id="else-rule"></a>4.  Chained Conditionals: the <a id="ref-for-at-ruledef-else②"></a>[&#64;else](#at-ruledef-else) rule[](#else-rule)

Usually, <a id="ref-for-conditional-group-rule②"></a>[conditional group rules](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) are independent; each one has a separate condition evaluated without direct reference to any other rule, and decides whether or not to apply its contained rules based solely on its condition.

This is fine for simple conditions, but makes it difficult to write a collection of conditionals that are meant to be mutually exclusive: authors have to very carefully craft their conditions to not activate when the other rules are meant to, and make sure the collection of conditionals don’t accidentally <em>all</em> exclude some situation which is then left unstyled.

The <a id="at-ruledef-else"></a><strong>&#64;else</strong> rule is a <a id="ref-for-conditional-group-rule③"></a>[conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) used to form <a id="ref-for-conditional-rule-chain②"></a>[conditional rule chains](#conditional-rule-chain), which associate multiple <a id="ref-for-conditional-group-rule④"></a>conditional group rules and guarantee that only the first one that matches will evaluate its condition as true. It is defined as:

<a id="ref-for-typedef-boolean-condition②"></a><a id="ref-for-mult-opt"></a><a id="ref-for-typedef-rule-list①"></a>

``` text
@else <boolean-condition>? {
  <rule-list>
}
```

<a id="ref-for-at-ruledef-else③"></a>[&#64;else](#at-ruledef-else) is interpreted identically to <a id="ref-for-at-ruledef-when④"></a>[&#64;when](#at-ruledef-when). If its <a id="ref-for-typedef-boolean-condition③"></a>[\<boolean-condition\>](#typedef-boolean-condition) is omitted, it’s treated as having a condition that’s always true.

A <a id="conditional-rule-chain"></a><strong>conditional rule chain</strong> is a series of consecutive <a id="ref-for-conditional-group-rule⑤"></a>[conditional group rules](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule), starting with a <a id="ref-for-conditional-group-rule⑥"></a>conditional group rule other than <a id="ref-for-at-ruledef-else④"></a>[&#64;else](#at-ruledef-else), followed by zero or more <a id="ref-for-at-ruledef-else⑤"></a>&#64;else rules. There cannot be anything between the successive <a id="ref-for-conditional-group-rule⑦"></a>conditional group rules other than whitespace and/or comments; any other token “breaks” the chain.

<a id="issue-a37c2e08"></a>

<strong>Issue:</strong>

[](#issue-a37c2e08) Should we require that only the last <a id="ref-for-at-ruledef-else⑥"></a>[&#64;else](#at-ruledef-else) in a chain can have an omitted condition? It’s not uncommon for me, when debugging code, to short-circuit an if-else chain by setting one of them to "true"; I presume that would be similarly useful in CSS? It’s still pretty easy to see you’ve done something wrong if you omit the condition accidentally.

Within a <a id="ref-for-conditional-rule-chain③"></a>[conditional rule chain](#conditional-rule-chain), the conditions of each <a id="ref-for-conditional-group-rule⑧"></a>[conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) are evaluated in order. If one of them is true, the conditions of all <em>following</em> <a id="ref-for-conditional-group-rule⑨"></a>conditional group rules in the chain evaluate to false, regardless of their stated condition.

An <a id="ref-for-at-ruledef-else⑦"></a>[&#64;else](#at-ruledef-else) rule that is not part of a <a id="ref-for-conditional-rule-chain④"></a>[conditional rule chain](#conditional-rule-chain) is invalid and must be ignored.

<a id="example-042e36fb"></a>

<strong>Example:</strong>

[](#example-042e36fb) For example, here’s a (somewhat silly) conditional chain:

``` text
@when media(width >= 400px) and media(pointer: fine) and supports(display: flex) {
  /* A */
} @else supports(caret-color: pink) and supports(background: double-rainbow()) {
  /* B */
} @else {
  /* C */
}
```

Exactly one of the preceding rules will be chosen, even though the second rule doesn’t exclude large widths, fine points, or flexbox support, and the last rule doesn’t specify anything at all.

To achieve the same result without <a id="ref-for-conditional-rule-chain⑤"></a>[conditional rule chains](#conditional-rule-chain), you’d need to write:

``` text
@media (width >= 400px) and (pointer: fine) {
  @supports (display: flex) {
  /* A */
  }
  @supports not (display: flex) {
  @supports (caret-color: pink) and (background: double-rainbow()) {
    /* B */
  }
  @supports not ((caret-color: pink) and (background: double-rainbow())) {
    /* C */
  }
  }
}
@media not ((width >= 400px) and (pointer: fine)) {
  @supports (caret-color: pink) and (background: double-rainbow()) {
  /* B */
  }
  @supports not ((caret-color: pink) and (background: double-rainbow())) {
  /* C */
  }
}
```

This is simultaneously hard to read, requires significant duplication of both conditions and contents, and is <em>very</em> difficult to write correctly. If the conditions got any more complicated (which is not unusual in real-world content), the example would get <em>significantly</em> worse.

<a id="example-e1965372"></a>

<strong>Example:</strong>

[](#example-e1965372) In this example, three different color font technologies are tested, in order of preference, plus a monochrome fallback. The most capable, COLRv1, supports both gradients and font variations; the next best choice, SVG, supports gradients while the least capable, COLRv0, supports flat color fill only.

The fallback has no test condition, so will always be chosen unless one of the earlier conditions succeeds.

``` text
@when font-tech(color-COLRv1) and font-tech(variations) {
  @font-face { font-family: icons; src: url(icons-gradient-var.woff2); }
}
@else font-tech(color-SVG) {
  @font-face { font-family: icons; src: url(icons-gradient.woff2); }
}
@else font-tech(color-COLRv0) {
  @font-face { font-family: icons; src: url(icons-flat.woff2); }
}
@else {
  @font-face { font-family: icons; src: url(icons-fallback.woff2); }
}
```

Notice that in this example, the variable color font is only downloaded if COLRv1 is supported and font variations are also supported.

Notice too that only one of the available options will be downloaded; this would not be the case without <a id="ref-for-at-ruledef-when⑤"></a>[&#64;when](#at-ruledef-when) and <a id="ref-for-at-ruledef-else⑧"></a>[&#64;else](#at-ruledef-else), as the next example shows.

<a id="example-a1e1a2b3"></a>

<strong>Example:</strong>

[](#example-a1e1a2b3) In this example, although it appears that the fallback will not be used if COLRv1 is supported, in fact both fonts will be downloaded, which wastes bandwidth if it is not used.

The fallback might still be used for some characters; for example, if the color font supports only Latin, while the fallback supports Latin and Greek.

``` text
@font-face { font-family: icons; src: url(icons-fallback.woff2);
@supports font-tech(color-COLRv1) {
  @font-face { font-family: icons; src: url(icons-gradient-var.woff2); }
}
```

## <a id="supports-condition-rule"></a>8.  Defining Custom Support Queries: the <a id="ref-for-at-ruledef-supports-condition"></a>[&#64;supports-condition](#at-ruledef-supports-condition) rule[](#supports-condition-rule)

The <a id="at-ruledef-supports-condition"></a><strong>&#64;supports-condition</strong> <a id="ref-for-at-rule④"></a>[at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule) is a <a id="ref-for-conditional-group-rule①②"></a>[conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) that allows authors to define and name a <a id="ref-for-supports-queries②"></a>[supports query](https://www.w3.org/TR/css-conditional-3/#supports-queries) for later reuse, creating a <a id="named-supports-condition"></a><strong>named supports condition</strong>. This enables complex or frequently-used feature queries to be referenced by name, improving maintainability and readability.

<a id="ref-for-typedef-supports-condition-name②"></a><a id="ref-for-typedef-block-contents"></a>

``` text
@supports-condition <supports-condition-name> {
  <block-contents>
}
```

Where <a id="typedef-supports-condition-name"></a><strong><a id="ref-for-typedef-supports-condition-name③"></a>[\<supports-condition-name\>](#typedef-supports-condition-name)</strong> is an <a id="ref-for-typedef-extension-name"></a>[\<extension-name\>](https://drafts.csswg.org/css-extensions-1/#typedef-extension-name) that defines the name of the supports query.

Anything inside the block is evaluated to test whether the user agent supports the features used. The contents do not have any effect on the document’s rendering.

Once defined, the named supports condition can be used in subsequent <a id="ref-for-at-ruledef-supports⑦"></a>[&#64;supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) or <a id="ref-for-at-ruledef-when⑥"></a>[&#64;when](#at-ruledef-when) conditions.

If multiple <a id="ref-for-at-ruledef-supports-condition①"></a>[&#64;supports-condition](#at-ruledef-supports-condition) rules are defined with the same name, the last one in document order wins, and all preceding ones are ignored.

<a id="example-ebaec0db"></a>

<strong>Example:</strong>

[](#example-ebaec0db) For example, we can define a supports query checking multiple properties at once:

``` text
@supports-condition --thicker-underlines {
  text-decoration-thickness: 0.2em;
  text-underline-offset: 0.3em;
}

/* Equivalent to (text-decoration-thickness: 0.2em) and (text-underline-offset: 0.3em) */
@supports (--thicker-underlines) {
  a {
    text-decoration: underline;
    text-decoration-thickness: 0.2em;
    text-underline-offset: 0.3em;
  }
}
```

<a id="ref-for-at-ruledef-supports-condition②"></a>[&#64;supports-condition](#at-ruledef-supports-condition) rules are allowed before <a id="ref-for-at-ruledef-import"></a>[&#64;import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) and <a id="ref-for-at-ruledef-namespace"></a>[&#64;namespace](https://drafts.csswg.org/css-namespaces-3/#at-ruledef-namespace) rules (after the <a id="ref-for-at-ruledef-charset①"></a>[&#64;charset](https://www.w3.org/TR/css-syntax-3/#at-ruledef-charset) rule, if any).

<a id="example-63d89a93"></a>

<strong>Example:</strong>

[](#example-63d89a93) As support queries can contain arbitrary declarations, they can be used to detect support for complex features such as nesting:

``` text
@supports-condition --nesting {
  & { }
}

@import url("nested-styles.css") supports(--nesting);
```

<a id="example-2391b075"></a>

<strong>Example:</strong>

[](#example-2391b075) Support queries can also be used to detect support for at-rules:

``` text
@supports-condition --stuck-container-feature {
  @container scroll-state(stuck: top) { }
}

@supports (--stuck-container-feature) {
  div { border-color: navy; }
}
```

<a id="issue-4ac0bea1"></a>

<strong>Issue:</strong>

[](#issue-4ac0bea1) The name of the at-rule is under discussion. Alternatives include &#64;supports-query, &#64;supports-test, and &#64;custom-supports. The name should be consistent with the one chosen for custom media queries.

### <a id="the-csssupportscondition-interface"></a>9.2.  The <code>CSSSupportsConditionRule</code> interface[](#the-csssupportscondition-interface)

The <code><a id="ref-for-csssupportsconditionrule"></a>[CSSSupportsConditionRule](#csssupportsconditionrule)</code> interface represents an <a id="ref-for-at-ruledef-supports-condition③"></a>[&#64;supports-condition](#at-ruledef-supports-condition) rule.

<a id="ref-for-Exposed①"></a><a id="csssupportsconditionrule"></a><a id="ref-for-cssgroupingrule"></a><a id="ref-for-cssomstring④"></a><a id="dom-csssupportsconditionrule-name"></a>

``` text
[Exposed=Window]
interface CSSSupportsConditionRule : CSSGroupingRule {
  readonly attribute CSSOMString name;
};
```

<strong><code>name</code> of type <code>CSSOMString</code></strong>

This attribute is the name of the <a id="ref-for-named-supports-condition①"></a>[named supports condition](#named-supports-condition).
