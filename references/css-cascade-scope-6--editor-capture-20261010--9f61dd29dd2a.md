Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Cascading and Inheritance Module Level 6](https://drafts.csswg.org/css-cascade-6/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply. The capture’s full original notice and links remain below.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md).

# Source provenance

Title: CSS Cascading and Inheritance Module Level 6

Source snapshot: https://drafts.csswg.org/css-cascade-6/

Retrieved: 2026-10-10. The captured page states Editor’s Draft, 28 April 2026. An undated editor URL can change; the retrieval date and exact byte hash identify this captured HTML.

Captured HTML SHA-256: 9f61dd29dd2af9edd74f564e1101250172cfb95324867fc0bd8a18603f7b265a

Captured HTML revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Immutable upstream source: [Overview.bs](https://raw.githubusercontent.com/w3c/csswg-drafts/50337328484f7fcdd9ab6fe9ded6011815b1707e/css-cascade-6/Overview.bs), SHA-256 `2f56b30c56202fba28d0bf849d757ef1b954089aac1fa9858ae173ab637f40a2`. The source revision is inspected as reference material and is not executed.

Representation notes:

- Full format conversion of the captured HTML body, including status, metadata, bibliography, indexes, examples, test references, and legal notice. Script and style elements are omitted and were not executed.
- All source body IDs are retained, including those inside preformatted blocks, which are relocated immediately before those blocks. Fragment-only links stay local only when their exact captured target exists; other links remain upstream.
- All 0 tables have readable Markdown layouts. All tables have ordinary row/column layouts. Synthetic Field/Definition or Column N headings are non-normative. Source row headers remain bold; native HTML header/accessibility semantics are not expressible in GFM.
- Preformatted examples retain literal text. Single-line table examples use semantic inline code. Compiler-generated hyperlinks inside fenced code blocks are omitted while their IDs and visible code text are retained; other source links are preserved. Small semantic code, variable, emphasis, subscript, and superscript HTML remains where Markdown notation would alter text.
- Conversion uses Pandoc 3.1.11.1 with focused Python/lxml preparation and a semantic Lua filter; the parsed GFM output is checked against the captured HTML. The [conversion report](CONVERSION-REPORT.md) records provenance and check boundaries.
- This full edition is a selected planning source for unstarted selector execution and scope-cascade work. The [source-edition mapping](SOURCE-EDITIONS.md) distinguishes these planning selections from historical editions consumed by completed CSS work.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Cascading and Inheritance Module Level 6

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 28 April 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-cascade-6/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-cascade-6/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2021/WD-css-cascade-6-20211221/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-cascade-6)

[Inline In Spec](#issues-index)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Miriam E. Suzanne](http://miriamsuzanne.com/contact) (Invited Expert)

[Tab Atkins Jr.](http://xanthir.com/contact/) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-cascade-6/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

This CSS module describes how to collate style rules and assign values to all properties on all elements. By way of cascading and inheritance, values are propagated for all properties on all elements.

New in this level is [§ 3.5 Scoping Styles: the &#64;scope rule](#scoped-styles).

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-cascade” in the title, like this: “\[css-cascade\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-cascade%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction and Missing Sections](#intro)
2.  [2 Importing Style Sheets: the &#64;import rule](#at-import)
    1.  [2.1 Conditional &#64;import Rules](#conditional-import)
    2.  [2.2 Processing Stylesheet Imports](#import-processing)
    3.  [2.3 Content-Type of CSS Style Sheets](#content-type)
3.  [3 Cascading](#cascading)
    1.  [3.1 Cascade Sorting Order](#cascade-sort)
    2.  [3.2 Cascading Origins](#cascading-origins)
    3.  [3.3 Important Declarations: the !important annotation](#importance)
    4.  [3.4 Cascade Layers](#layering)
        1.  [3.4.1 Layer Ordering](#layer-ordering)
    5.  [3.5 Scoping Styles: the &#64;scope rule](#scoped-styles)
        1.  [3.5.1 Effects of &#64;scope](#scope-effects)
        2.  [3.5.2 Syntax of &#64;scope](#scope-syntax)
        3.  [3.5.3 Scoped Style Rules](#scoped-rules)
        4.  [3.5.4 Identifying Scoping Roots and Limits](#scope-limits)
        5.  [3.5.5 Scope Nesting](#scope-scope)
        6.  [3.5.6 Scoped Declarations ](#scoped-declarations)
    6.  [3.6 Precedence of Non-CSS Presentational Hints](#preshint)
4.  [4 CSSOM](#cssom)
    1.  [4.1 The <code>CSSScopeRule</code> interface](#the-cssscoperule-interface)
5.  [5 Changes](#changes)
    1.  [5.1 Changes since the 21 March 2023 Working Draft](#changes-since-2023-03)
    2.  [5.2 Changes since the 21 December 2021 First Public Working Draft](#changes-2022-08)
    3.  [5.3 Additions Since Level 5](#additions-l5)
    4.  [5.4 Additions Since Level 4](#additions-l4)
    5.  [5.5 Additions Since Level 3](#additions-l3)
    6.  [5.6 Additions Since Level 2](#changes-2)
6.  [ Acknowledgments](#acknowledgments)
7.  [6 Privacy Considerations](#privacy)
8.  [7 Security Considerations](#security)
9.  [ Conformance](#w3c-conformance)
    1.  [ Document conventions](#w3c-conventions)
    2.  [ Conformance classes](#w3c-conformance-classes)
    3.  [ Partial implementations](#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](#w3c-testing)
10. [ Index](#index)
    1.  [ Terms defined by this specification](#index-defined-here)
    2.  [ Terms defined by reference](#index-defined-elsewhere)
11. [ References](#references)
    1.  [ Normative References](#normative)
    2.  [ Non-Normative References](#informative)
12. [ IDL Index](#idl-index)
13. [ Issues Index](#issues-index)

## <a id="intro"></a>1. <a id="layer-empty"></a><a id="default"></a><a id="inherit-initial"></a><a id="inherit"></a><a id="initial"></a><a id="defaulting-keywords"></a><a id="inheriting"></a><a id="initial-values"></a><a id="defaulting"></a><a id="shorthand"></a><a id="aliasing"></a><a id="all-shorthand"></a><a id="value-stages"></a><a id="specified"></a><a id="declared"></a><a id="cascaded"></a><a id="computed"></a><a id="used"></a><a id="actual"></a><a id="stages-examples"></a><a id="fragments"></a><a id="filtering"></a> Introduction and Missing Sections[](#intro)

<a id="issue-a783701e"></a>

<strong>Issue:</strong>

[](#issue-a783701e) This is a diff spec over [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 5 as a reference. We will merge the Level 5 text into this draft once it reaches CR.

## <a id="at-import"></a>2.  Importing Style Sheets: the <a id="ref-for-at-ruledef-import"></a>[&#64;import](#at-ruledef-import) rule[](#at-import)

The <a id="at-ruledef-import"></a><strong>&#64;import</strong> rule allows users to import style rules from other style sheets. If an <a id="ref-for-at-ruledef-import①"></a>[&#64;import](#at-ruledef-import) rule refers to a valid stylesheet, user agents must treat the contents of the stylesheet as if they were written in place of the <a id="ref-for-at-ruledef-import②"></a>&#64;import rule, with two exceptions:

- If a feature (such as the <a id="ref-for-at-ruledef-namespace"></a>[&#64;namespace](https://drafts.csswg.org/css-namespaces-3/#at-ruledef-namespace) rule) <em>explicitly</em> defines that it only applies to a particular stylesheet, and not any imported ones, then it doesn’t apply to the imported stylesheet.

- If a feature relies on the relative ordering of two or more constructs in a stylesheet (such as the requirement that <a id="ref-for-at-ruledef-namespace①"></a>[&#64;namespace](https://drafts.csswg.org/css-namespaces-3/#at-ruledef-namespace) rules must not have any other rules other than <a id="ref-for-at-ruledef-import③"></a>[&#64;import](#at-ruledef-import) preceding it), it only applies between constructs in the same stylesheet.

<a id="example-d425da12"></a>

<strong>Example:</strong>

[](#example-d425da12) For example, <a id="ref-for-cssstyledeclaration-declarations"></a>[declarations](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-declarations) in style rules from imported stylesheets interact with the cascade as if they were written literally into the stylesheet at the point of the <a id="ref-for-at-ruledef-import④"></a>[&#64;import](#at-ruledef-import).

Any <a id="ref-for-at-ruledef-import⑤"></a>[&#64;import](#at-ruledef-import) rules must precede all other valid at-rules and style rules in a style sheet (ignoring <a id="ref-for-at-ruledef-charset"></a>[&#64;charset](https://drafts.csswg.org/css-syntax-3/#at-ruledef-charset), <a id="ref-for-at-ruledef-supports-condition"></a>[&#64;supports-condition](https://drafts.csswg.org/css-conditional-5/#at-ruledef-supports-condition), and [&#64;layer statement](#layer-empty) rules) and must not have any other valid at-rules or style rules between it and previous <a id="ref-for-at-ruledef-import⑥"></a>&#64;import rules, or else the <a id="ref-for-at-ruledef-import⑦"></a>&#64;import rule is invalid. The syntax of <a id="ref-for-at-ruledef-import⑧"></a>&#64;import is:

<a id="ref-for-url-value"></a><a id="ref-for-comb-one"></a><a id="ref-for-string-value"></a><a id="ref-for-comb-one①"></a><a id="ref-for-typedef-layer-name"></a><a id="ref-for-comb-any"></a><a id="ref-for-comb-one②"></a><a id="ref-for-typedef-scope-start"></a><a id="ref-for-comb-one③"></a><a id="ref-for-typedef-scope-boundaries"></a><a id="ref-for-comb-any①"></a><a id="ref-for-typedef-supports-import-condition"></a><a id="ref-for-mult-opt"></a><a id="ref-for-typedef-media-import-condition"></a><a id="typedef-supports-import-condition"></a><a id="ref-for-typedef-supports-condition"></a><a id="ref-for-comb-one④"></a><a id="ref-for-typedef-declaration"></a><a id="typedef-media-import-condition"></a><a id="ref-for-typedef-media-query-list"></a>

``` text
@import [ <url> | <string> ]
        [[ layer | layer( <layer-name> ) ]
         || [ scope | scope( <scope-start> | <scope-boundaries> ) ]
         || <supports-import-condition>]?
        <media-import-condition> ;

<supports-import-condition> = supports( [ <supports-condition> | <declaration> ] )
<media-import-condition> = <media-query-list>
```

where:

- the <a id="ref-for-url-value①"></a>[\<url\>](https://drafts.csswg.org/css-values-4/#url-value) or <a id="ref-for-string-value①"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value) gives the URL of the style sheet to be imported.

and optionally:

- the layer keyword or layer() function, which assigns the contents of the style sheet into its own anonymous <a id="ref-for-cascade-layers"></a>[cascade layer](https://drafts.csswg.org/css-cascade-5/#cascade-layers) or into the named <a id="ref-for-cascade-layers①"></a>cascade layer.

  The layer is added to the [layer order](#layer-ordering) even if the import fails to load the stylesheet, but is subject to any <a id="ref-for-import-conditions"></a>[import conditions](#import-conditions) (just as if declared by an <a id="ref-for-at-ruledef-layer"></a>[&#64;layer](https://drafts.csswg.org/css-cascade-5/#at-ruledef-layer) rule wrapped in the appropriate <a id="ref-for-conditional-group-rule"></a>[conditional group rules](https://drafts.csswg.org/css-conditional-3/#conditional-group-rule)).

- the scope keyword or scope() function, which <a id="ref-for-scope"></a>[scopes](#scope) the <a id="ref-for-style-rule"></a>[style rules](https://drafts.csswg.org/css-syntax-3/#style-rule) within the stylesheet, using the <a id="ref-for-scoping-root"></a>[scoping roots](https://drafts.csswg.org/selectors-4/#scoping-root) and <a id="ref-for-scoping-limit"></a>[scoping limits](#scoping-limit) as described by [§ 3.5.4 Identifying Scoping Roots and Limits](#scope-limits).

  Note: The scope keyword behaves like a <a id="ref-for-at-ruledef-scope"></a>[&#64;scope](#at-ruledef-scope) rule with an empty prelude, scoping the imported rules to the <a id="ref-for-parent-element"></a>[parent element](https://dom.spec.whatwg.org/#parent-element) of the <a id="ref-for-cssstyledeclaration-owner-node"></a>[owner node](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-owner-node) of the stylesheet containing the <a id="ref-for-at-ruledef-import⑨"></a>[&#64;import](#at-ruledef-import) rule.

  Note: While the <a id="ref-for-style-rule①"></a>[style rules](https://drafts.csswg.org/css-syntax-3/#style-rule) within the imported stylesheet become <a id="ref-for-scope①"></a>[scoped](#scope), they do not become <a id="ref-for-nested-style-rule"></a>[nested](https://drafts.csswg.org/css-nesting-1/#nested-style-rule). In particular, top-level selectors are not re-interpreted as <a id="ref-for-relative-selector"></a>[relative selectors](https://drafts.csswg.org/selectors-4/#relative-selector), and the <a id="ref-for-selectordef-"></a>[&](https://drafts.csswg.org/css-nesting-1/#selectordef-) pseudo-class maintains its non-nested behavior.

- the <a id="ref-for-import-conditions①"></a>[import conditions](#import-conditions), <a id="ref-for-typedef-supports-import-condition①"></a>[\<supports-import-condition\>](#typedef-supports-import-condition) and <a id="ref-for-typedef-media-import-condition①"></a>[\<media-import-condition\>](#typedef-media-import-condition), which state the conditions under which the <a id="ref-for-at-ruledef-import①⓪"></a>[&#64;import](#at-ruledef-import) rule applies.

<a id="example-05c74581"></a>

<strong>Example:</strong>

[](#example-05c74581) The following [conditional &#64;import rule](#conditional-import) only loads the style sheet when the UA [supports](https://www.w3.org/TR/css-conditional-3/#support-definition) <a id="ref-for-propdef-display"></a>[display: flex](https://drafts.csswg.org/css-display-4/#propdef-display), and only applies the style sheet on a [handheld](https://www.w3.org/TR/CSS2/media.html#media-types) device with a [maximum viewport width](https://www.w3.org/TR/mediaqueries-4/#width) of 400px.

``` text
@import url("narrow.css") supports(display: flex) handheld and (max-width: 400px);
```

<a id="example-6e298f38"></a>

<strong>Example:</strong>

[](#example-6e298f38) The following layer imports load the style sheets into the framework.component layer, and an un-named layer, respectively:

``` text
@import url("tabs.css") layer(framework.component);
@import url("override.css") layer;
```

<a id="example-c0535026"></a>

<strong>Example:</strong>

[](#example-c0535026) The following imports a stylesheet, and scopes the style rules to elements matching <code>&#46;card</code>.

``` text
@import url("card.css") scope(.card);
```

If a <a id="ref-for-string-value②"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value) is provided, it must be interpreted as a <a id="ref-for-url-value②"></a>[\<url\>](https://drafts.csswg.org/css-values-4/#url-value) with the same value.

<a id="example-da2e49cb"></a>

<strong>Example:</strong>

[](#example-da2e49cb) The following lines are equivalent in meaning and illustrate both <a id="ref-for-at-ruledef-import①①"></a>[&#64;import](#at-ruledef-import) syntaxes (one with <a id="ref-for-funcdef-url"></a>[url()](https://drafts.csswg.org/css-values-3/#funcdef-url) and one with a bare string):

``` text
@import "mystyle.css";
@import url("mystyle.css");
```

### <a id="conditional-import"></a>2.1.  Conditional <a id="ref-for-at-ruledef-import①②"></a>[&#64;import](#at-ruledef-import) Rules[](#conditional-import)

<a id="import-conditions"></a><strong>Import conditions</strong> allow the import to be media– or feature-support–dependent. In the absence of any <a id="ref-for-import-conditions②"></a>[import conditions](#import-conditions), the import is unconditional. (Specifying all for the <a id="ref-for-typedef-media-query-list①"></a>[\<media-query-list\>](https://drafts.csswg.org/mediaqueries-4/#typedef-media-query-list) has the same effect.) If the <a id="ref-for-import-conditions③"></a>import conditions do not match, the rules in the imported stylesheet do not apply, exactly as if the imported stylesheet were wrapped in <a id="ref-for-at-ruledef-media"></a>[&#64;media](https://drafts.csswg.org/css-conditional-3/#at-ruledef-media) and/or <a id="ref-for-at-ruledef-supports"></a>[&#64;supports](https://drafts.csswg.org/css-conditional-3/#at-ruledef-supports) blocks with the given conditions.

<a id="example-5fa651f3"></a>

<strong>Example:</strong>

[](#example-5fa651f3) The following rules illustrate how <a id="ref-for-at-ruledef-import①③"></a>[&#64;import](#at-ruledef-import) rules can be made media-dependent:

``` text
@import url("fineprint.css") print;
@import url("bluish.css") projection, tv;
@import url("narrow.css") handheld and (max-width: 400px);
```

User agents may therefore avoid fetching a conditional import as long as the <a id="ref-for-import-conditions④"></a>[import conditions](#import-conditions) do not match. Additionally, if a <a id="ref-for-typedef-supports-condition①"></a>[\<supports-condition\>](https://drafts.csswg.org/css-conditional-3/#typedef-supports-condition) blocks the application of the imported style sheet, the UA <em>must not</em> fetch the style sheet (unless it is loaded through some other link) and <em>must</em> return null for the import rule’s CSSImportRule.styleSheet value (even if it is loaded through some other link).

<a id="example-c6c2c0ee"></a>

<strong>Example:</strong>

[](#example-c6c2c0ee) The following rule illustrates how an author can provide fallback rules for legacy user agents without impacting network performance on newer user agents:

``` text
@import url("fallback-layout.css") supports(not (display: flex));
@supports (display: flex) {
  ...
}
```

The <a id="ref-for-import-conditions⑤"></a>[import conditions](#import-conditions) are given by <a id="ref-for-typedef-media-query-list②"></a>[\<media-query-list\>](https://drafts.csswg.org/mediaqueries-4/#typedef-media-query-list), which is parsed and interpreted as a <a id="ref-for-media-query-list"></a>[media query list](https://drafts.csswg.org/mediaqueries-4/#media-query-list), and <a id="ref-for-typedef-supports-condition②"></a>[\<supports-condition\>](https://drafts.csswg.org/css-conditional-3/#typedef-supports-condition), is parsed and interpreted as a <a id="ref-for-supports-queries"></a>[supports query](https://drafts.csswg.org/css-conditional-3/#supports-queries). If a <a id="ref-for-typedef-declaration①"></a>[\<declaration\>](https://drafts.csswg.org/css-conditional-5/#typedef-declaration) is given in place of a <a id="ref-for-typedef-supports-condition③"></a>\<supports-condition\>, it must be interpreted as a <a id="ref-for-typedef-supports-decl"></a>[\<supports-decl\>](https://drafts.csswg.org/css-conditional-5/#typedef-supports-decl) (i.e. the extra set of parentheses is implied) and treated as a <a id="ref-for-typedef-supports-condition④"></a>\<supports-condition\>.

<a id="example-75806df4"></a>

<strong>Example:</strong>

[](#example-75806df4) For example, the following two lines are equivalent:

``` text
@import "mystyle.css" supports(display: flex);
@import "mystyle.css" supports((display: flex));
```

The evaluation and full syntax of the <a id="ref-for-import-conditions⑥"></a>[import conditions](#import-conditions) are defined by the [Media Queries](https://www.w3.org/TR/mediaqueries/) [\[MEDIAQ\]](#biblio-mediaq) and [CSS Conditional Rules](https://www.w3.org/TR/css-conditional/) [\[CSS-CONDITIONAL-3\]](#biblio-css-conditional-3) specifications.

### <a id="import-processing"></a>2.2.  Processing Stylesheet Imports[](#import-processing)

When the same style sheet is imported or linked to a document in multiple places, user agents must process (or act as though they do) each link as though the link were to an independent style sheet.

Note: This does not place any requirements on resource fetching, only how the style sheet is reflected in the CSSOM and used in specs such as this one. Assuming appropriate caching, it is perfectly appropriate for a UA to fetch a style sheet only once, even though it’s linked or imported multiple times.

The <a id="ref-for-origin"></a>[cascade origin](#origin) of an imported style sheet is the <a id="ref-for-origin①"></a>cascade origin of the style sheet that imported it.

The <a id="ref-for-environment-encoding"></a>[environment encoding](https://drafts.csswg.org/css-syntax-3/#environment-encoding) of an imported style sheet is the encoding of the style sheet that imported it. [\[css-syntax-3\]](#biblio-css-syntax-3)

### <a id="content-type"></a>2.3.  Content-Type of CSS Style Sheets[](#content-type)

The processing of imported style sheets depends on the actual type of the linked resource:

- If the resource does not have <a id="ref-for-content-type"></a>[Content-Type metadata](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#content-type), the type is treated as <code>text/css</code>.

- If the host document is in <a id="ref-for-concept-document-quirks"></a>[quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks), and the host document’s origin is <a id="ref-for-same-origin"></a>[same origin](https://html.spec.whatwg.org/multipage/browsers.html#same-origin) with the linked resource <a id="ref-for-concept-response"></a>[response’s](https://fetch.spec.whatwg.org/#concept-response) <a id="ref-for-concept-response-url"></a>[URL’s](https://fetch.spec.whatwg.org/#concept-response-url) origin, the type is treated as <code>text/css</code>.

- Otherwise, the type is determined from its <a id="ref-for-content-type①"></a>[Content-Type metadata](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#content-type).

If the linked resource’s type is <code>text/css</code>, it must be interpreted as a CSS style sheet. Otherwise, it must be interpreted as a network error.

## <a id="cascading"></a>3.  Cascading[](#cascading)

The <a id="cascade"></a><strong>cascade</strong> takes an unordered list of <a id="ref-for-declared-value"></a>[declared values](https://drafts.csswg.org/css-cascade-5/#declared-value) for a given property on a given element, sorts them by their <a id="ref-for-declaration"></a>[declaration’s](https://drafts.csswg.org/css-syntax-3/#declaration) precedence as determined below, and outputs a single <a id="ref-for-cascaded-value"></a>[cascaded value](https://drafts.csswg.org/css-cascade-5/#cascaded-value).

### <a id="cascade-sort"></a>3.1.  Cascade Sorting Order[](#cascade-sort)

The cascade sorts <a id="ref-for-cssstyledeclaration-declarations①"></a>[declarations](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-declarations) according to the following criteria, in descending order of precedence:

<a id="cascade-origin"></a><strong>[](#cascade-origin)Origin and Importance</strong>

The <a id="ref-for-origin②"></a>[origin](#origin) of a <a id="ref-for-declaration①"></a>[declaration](https://drafts.csswg.org/css-syntax-3/#declaration) is based on where it comes from and its <a id="ref-for-important"></a>[importance](#important) is whether or not it is declared with !important (see [below](#importance)). The precedence of the various <a id="ref-for-origin③"></a>origins is, in descending order:

1.  Transition declarations [\[css-transitions-1\]](#biblio-css-transitions-1)
2.  <a id="ref-for-important①"></a>[Important](#important) <a id="ref-for-cascade-origin-ua"></a>[user agent](https://drafts.csswg.org/css-cascade-5/#cascade-origin-ua) declarations
3.  <a id="ref-for-important②"></a>[Important](#important) <a id="ref-for-cascade-origin-user"></a>[user](https://drafts.csswg.org/css-cascade-5/#cascade-origin-user) declarations
4.  <a id="ref-for-important③"></a>[Important](#important) <a id="ref-for-cascade-origin-author"></a>[author](https://drafts.csswg.org/css-cascade-5/#cascade-origin-author) declarations
5.  Animation declarations [\[css-animations-1\]](#biblio-css-animations-1)
6.  <a id="ref-for-normal"></a>[Normal](#normal) <a id="ref-for-cascade-origin-author①"></a>[author](https://drafts.csswg.org/css-cascade-5/#cascade-origin-author) declarations
7.  <a id="ref-for-normal①"></a>[Normal](#normal) <a id="ref-for-cascade-origin-user①"></a>[user](https://drafts.csswg.org/css-cascade-5/#cascade-origin-user) declarations
8.  <a id="ref-for-normal②"></a>[Normal](#normal) <a id="ref-for-cascade-origin-ua①"></a>[user agent](https://drafts.csswg.org/css-cascade-5/#cascade-origin-ua) declarations

Declarations from <a id="ref-for-origin④"></a>[origins](#origin) earlier in this list win over declarations from later <a id="ref-for-origin⑤"></a>origins.

<a id="cascade-context"></a><strong>[](#cascade-context)Context</strong>

A document language can provide for blending <a id="ref-for-cssstyledeclaration-declarations②"></a>[declarations](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-declarations) sourced from different <a id="encapsulation-contexts"></a><strong>encapsulation contexts</strong>, such as the nested <a id="ref-for-tree-context"></a>[tree contexts](https://drafts.csswg.org/css-shadow-1/#tree-context) of <a id="ref-for-concept-shadow-tree"></a>[shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) in the [\[DOM\]](#biblio-dom).

When comparing two declarations that are sourced from different <a id="ref-for-encapsulation-contexts"></a>[encapsulation contexts](#encapsulation-contexts), then for <a id="ref-for-normal③"></a>[normal](#normal) rules the declaration from the outer context wins, and for <a id="ref-for-important④"></a>[important](#important) rules the declaration from the inner context wins. For this purpose, [\[DOM\]](#biblio-dom) <a id="ref-for-tree-context①"></a>[tree contexts](https://drafts.csswg.org/css-shadow-1/#tree-context) are considered to be nested in <a id="ref-for-concept-shadow-including-tree-order"></a>[shadow-including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order).

Note: This effectively means that <a id="ref-for-normal④"></a>[normal](#normal) declarations belonging to an <a id="ref-for-encapsulation-contexts①"></a>[encapsulation context](#encapsulation-contexts) can set defaults that are easily overridden by the outer context, while <a id="ref-for-important⑤"></a>[important](#important) declarations belonging to an <a id="ref-for-encapsulation-contexts②"></a>encapsulation context can enforce requirements that cannot be overridden by the outer context.

<a id="style-attr"></a><strong>[](#style-attr)The Style Attribute</strong>

Separately for <a id="ref-for-normal⑤"></a>[normal](#normal) and <a id="ref-for-important⑥"></a>[important](#important) <a id="ref-for-cssstyledeclaration-declarations③"></a>[declarations](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-declarations), declarations that are attached directly to an element (such as the [contents of a style attribute](https://www.w3.org/TR/css-style-attr/#interpret)) rather than indirectly mapped by means of a style rule selector take precedence over declarations the same <a id="ref-for-important⑦"></a>importance that are mapped via style rule.

<a id="cascade-layering"></a><strong>[](#cascade-layering)Layers</strong>

<a id="ref-for-cssstyledeclaration-declarations④"></a>[Declarations](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-declarations) within each <a id="ref-for-origin⑥"></a>[origin](#origin) and <a id="ref-for-encapsulation-contexts③"></a>[context](#encapsulation-contexts) can be explicitly assigned to a <a id="ref-for-cascade-layers②"></a>[cascade layer](https://drafts.csswg.org/css-cascade-5/#cascade-layers). For the purpose of this step, any declaration not assigned to an explicit layer is added to an implicit final layer.

Cascade layers (like declarations) are sorted by order of appearance, see [§ 3.4.1 Layer Ordering](#layer-ordering). When comparing declarations that belong to different layers, then for <a id="ref-for-normal⑥"></a>[normal](#normal) rules the declaration whose <a id="ref-for-cascade-layers③"></a>[cascade layer](https://drafts.csswg.org/css-cascade-5/#cascade-layers) is latest in the layer order wins, and for <a id="ref-for-important⑧"></a>[important](#important) rules the declaration whose <a id="ref-for-cascade-layers④"></a>cascade layer is earliest wins.

Note: This follows the same logic used for precedence of <a id="ref-for-normal⑦"></a>[normal](#normal) and <a id="ref-for-important⑨"></a>[important](#important) <a id="ref-for-origin⑦"></a>[origins](#origin), thus the !important flag maintains the same “override” purpose in both settings.

<a id="cascade-specificity"></a><strong>[](#cascade-specificity)Specificity</strong>

The [Selectors module](https://www.w3.org/TR/selectors/#specificity) [\[SELECT\]](#biblio-select) describes how to compute the specificity of a selector.

When comparing declarations from two style rules, the declaration belonging to the style rule with the highest specificity wins.

<a id="cascade-proximity"></a><strong>[](#cascade-proximity)<a id="scope-proximity"></a><strong>Scope Proximity</strong></strong>

When comparing declarations that appear in style rules with different <a id="ref-for-scoping-root①"></a>[scoping roots](https://drafts.csswg.org/selectors-4/#scoping-root), then the declaration with the fewest generational or sibling-element hops between the <a id="ref-for-scoping-root②"></a>scoping root and the <a id="ref-for-scoped-style-rules"></a>[scoped style rule](#scoped-style-rules) <a id="ref-for-selector-subject"></a>[subject](https://drafts.csswg.org/selectors-4/#selector-subject) wins. For this purpose, style rules without a <a id="ref-for-scoping-root③"></a>scoping root are considered to have infinite proximity hops.

<a id="cascade-order"></a><strong>[](#cascade-order)Order of Appearance</strong>

The last declaration in document order wins. For this purpose:

- Style sheets are ordered as in [final CSS style sheets](https://drafts.csswg.org/cssom/#documentorshadowroot-final-css-style-sheets).
- Declarations from <a id="ref-for-at-ruledef-import①④"></a>[imported style sheets](#at-ruledef-import) are ordered as if their style sheets were substituted in place of the <a id="ref-for-at-ruledef-import①⑤"></a>&#64;import rule.
- Declarations from style sheets independently linked by the originating document are treated as if they were concatenated in linking order, as determined by the host document language.
- Declarations from style attributes are ordered according to the document order of the element the style attribute appears on, and are all placed after any style sheets. [\[CSSSTYLEATTR\]](#biblio-cssstyleattr)

The <a id="output-of-the-cascade"></a><strong>output of the cascade</strong> is a (potentially empty) sorted list of <a id="ref-for-declared-value①"></a>[declared values](https://drafts.csswg.org/css-cascade-5/#declared-value) for each property on each element.

### <a id="cascading-origins"></a>3.2.  Cascading Origins[](#cascading-origins)

<a id="issue-d41d8cd9"></a>

<strong>Issue:</strong>

[](#issue-d41d8cd9) [CSS Cascading 5 § 6.2 Cascading Origins](https://drafts.csswg.org/css-cascade-5/#cascading-origins)

<a id="origin"></a><strong>cascade origin</strong>

### <a id="importance"></a>3.3.  Important Declarations: the !important annotation[](#importance)

<a id="issue-d41d8cd9①"></a>

<strong>Issue:</strong>

[](#issue-d41d8cd9%E2%91%A0) [CSS Cascading 5 § 6.3 Important Declarations: the !important annotation](https://drafts.csswg.org/css-cascade-5/#importance)

<a id="important"></a><strong>important</strong> <a id="normal"></a><strong>normal</strong>

### <a id="layering"></a>3.4.  Cascade Layers[](#layering)

<a id="issue-d41d8cd9②"></a>

<strong>Issue:</strong>

[](#issue-d41d8cd9%E2%91%A1) [CSS Cascading 5 § 6.4 Cascade Layers](https://drafts.csswg.org/css-cascade-5/#layering)

#### <a id="layer-ordering"></a>3.4.1.  Layer Ordering[](#layer-ordering)

<a id="issue-d41d8cd9③"></a>

<strong>Issue:</strong>

[](#issue-d41d8cd9%E2%91%A2) [CSS Cascading 5 § 6.4.3 Layer Ordering](https://drafts.csswg.org/css-cascade-5/#layer-ordering)

### <a id="scoped-styles"></a>3.5. <a id="scope-atrule"></a> Scoping Styles: the <a id="ref-for-at-ruledef-scope①"></a>[&#64;scope](#at-ruledef-scope) rule[](#scoped-styles)

A <a id="scope"></a><strong>scope</strong> is a subtree or fragment of a document, which can be used by selectors for more targeted matching. A <a id="ref-for-scope②"></a>[scope](#scope) is formed by determining:

- The <a id="ref-for-scoping-root④"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) <a id="ref-for-boundary-point-node"></a>[node](https://dom.spec.whatwg.org/#boundary-point-node), which acts as the upper bound of the scope, and optionally:

- The <a id="scoping-limit"></a><strong>scoping limit</strong> elements, which act as the lower bounds.

An element is <a id="in-scope"></a><strong>in scope</strong> if:

- It is an <a id="ref-for-concept-tree-inclusive-descendant"></a>[inclusive descendant](https://dom.spec.whatwg.org/#concept-tree-inclusive-descendant) of the <a id="ref-for-scoping-root⑤"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root), and

- It is not an <a id="ref-for-concept-tree-inclusive-descendant①"></a>[inclusive descendant](https://dom.spec.whatwg.org/#concept-tree-inclusive-descendant) of a <a id="ref-for-scoping-limit①"></a>[scoping limit](#scoping-limit).

Note: In contrast to [Shadow Encapsulation](https://drafts.csswg.org/css-scoping/#shadow-dom), which describes a persistent one-to-one relationship in the DOM between a <u>shadow host</u> and its nested <a id="ref-for-concept-shadow-tree①"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), multiple overlapping <a id="ref-for-scope③"></a>[scopes](#scope) can be defined in relation to the same elements.

Scoped styles are described in CSS using the <a id="at-ruledef-scope"></a><strong>&#64;scope</strong> <a id="ref-for-block-at-rule"></a>[block at-rule](https://drafts.csswg.org/css-syntax-3/#block-at-rule), which declares a <a id="ref-for-scoping-root⑥"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) and optional <a id="ref-for-scoping-limit②"></a>[scoping limits](#scoping-limit) associated with a set of <a id="ref-for-style-rule②"></a>[style rules](https://drafts.csswg.org/css-syntax-3/#style-rule).

<a id="example-f310c4e4"></a>

<strong>Example:</strong>

[](#example-f310c4e4) For example, an author might have wide-reaching color-scheme scopes, which overlap more narrowly-scoped design patterns such as a media object. The selectors in the <a id="ref-for-at-ruledef-scope②"></a>[&#64;scope](#at-ruledef-scope) rule establish <a id="ref-for-scoping-root⑦"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) and optional <a id="ref-for-scoping-limit③"></a>[scoping limit](#scoping-limit) elements, while the nested selectors only match elements that are <a id="ref-for-in-scope"></a>[in a resulting scope](#in-scope):

``` text
@scope (.light-scheme) {
  /* Only match links inside a light-scheme */
  a { color: darkmagenta; }
}

@scope (.dark-scheme) {
  /* Only match links inside a dark-scheme */
  a { color: plum; }
}

@scope (.media-object) {
  /* Only match author images inside a media-object */
  .author-image { border-radius: 50%; }
}
```

<a id="example-b0418ef2"></a>

<strong>Example:</strong>

[](#example-b0418ef2) By providing <a id="ref-for-scoping-limit④"></a>[scoping limits](#scoping-limit), an author can limit matching more deeply nested descendants. For example:

``` text
@scope (.media-object) to (.content > *) {
  img { border-radius: 50%; }
  .content { padding: 1em; }
}
```

The img selector will only match image tags that are in a DOM fragment starting with any .media-object, and including all descendants up to any intervening children of the .content class.

<a id="issue-acd126a1"></a>

<strong>Issue:</strong>

[](#issue-acd126a1) Should scoping limits be added to the definition of <a id="ref-for-scoped-selector"></a>[scoped selectors](https://drafts.csswg.org/selectors-4/#scoped-selector)?

#### <a id="scope-effects"></a>3.5.1.  Effects of <a id="ref-for-at-ruledef-scope③"></a>[&#64;scope](#at-ruledef-scope)[](#scope-effects)

The <a id="ref-for-at-ruledef-scope④"></a>[&#64;scope](#at-ruledef-scope) <a id="ref-for-at-rule"></a>[at-rule](https://drafts.csswg.org/css-syntax-3/#at-rule) has three primary effects on the <a id="ref-for-style-rule③"></a>[style rules](https://drafts.csswg.org/css-syntax-3/#style-rule) it contains:

- The <a id="ref-for-style-rule④"></a>[style rules](https://drafts.csswg.org/css-syntax-3/#style-rule) in an <a id="ref-for-at-ruledef-scope⑤"></a>[&#64;scope](#at-ruledef-scope) <a id="ref-for-typedef-rule-list"></a>[\<rule-list\>](https://drafts.csswg.org/css-syntax-3/#typedef-rule-list) are <a id="ref-for-scoped-style-rules①"></a>[scoped style rules](#scoped-style-rules).

- The <a id="ref-for-scope-pseudo"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo) selector is defined to match the <a id="ref-for-at-ruledef-scope⑥"></a>[&#64;scope](#at-ruledef-scope) rule’s <a id="ref-for-scoping-root⑧"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root), including the <a id="ref-for-featureless"></a>[featureless](https://drafts.csswg.org/selectors-4/#featureless) <u>shadow host</u> when that host is the <a id="ref-for-scoping-root⑨"></a>scoping root.

- The <a id="ref-for-selectordef-①"></a>[&](https://drafts.csswg.org/css-nesting-1/#selectordef-) selector is defined to behave as <code>&#58;where(&#58;scope)</code>.

- The <a id="ref-for-cascade"></a>[cascade](#cascade) prioritizes declarations with a <a id="ref-for-scope-proximity"></a>[more proximate](#scope-proximity) <a id="ref-for-scoping-root①⓪"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root), regardless of specificity or order of appearance by applying <a id="ref-for-scope-proximity①"></a>scope proximity between the <a id="ref-for-scoping-root①①"></a>scoping root and the <a id="ref-for-selector-subject①"></a>[subject](https://drafts.csswg.org/selectors-4/#selector-subject) of each <a id="ref-for-scoped-style-rules②"></a>[scoped style rule](#scoped-style-rules).

Note: Unlike [Nesting](https://www.w3.org/TR/css-nesting/), selectors within an <a id="ref-for-at-ruledef-scope⑦"></a>[&#64;scope](#at-ruledef-scope) rule do not acquire the specificity of any parent selector(s) in the <a id="ref-for-at-ruledef-scope⑧"></a>&#64;scope prelude.

<a id="example-47f85a12"></a>

<strong>Example:</strong>

[](#example-47f85a12) The following selectors have the same specificity (0,0,1):

``` text
@scope (#hero) {
  img { border-radius: 50%; }
}

:where(#hero) img { border-radius: 50%; }
```

The additional specificity of the \#hero selector is not applied to the specificity of the scoped selector. However, since one <code><a id="ref-for-the-img-element"></a>[img](https://html&#46;spec&#46;whatwg&#46;org/multipage/embedded-content&#46;html#the-img-element)</code> selector is scoped, that selector is weighted more strongly in the cascade with the application of <a id="ref-for-scope-proximity②"></a>[scope proximity](#scope-proximity).

<a id="example-463550a5"></a>

<strong>Example:</strong>

[](#example-463550a5) Many existing tools implement "scoped styles" by applying a unique class or attribute to every element in a given scope or "single file component." In this example there are two scopes (<code>main-component</code> and <code>sub-component</code>) and every element is marked as part of one or both scopes using the <code>data-scope</code> attribute:

``` text
<section data-scope="main-component">
  <p data-scope="main-component">...<p>

  <!-- sub-component root is in both scopes -->
  <section data-scope="main-component sub-component">
    <!-- children are only in the inner scope -->
    <p data-scope="sub-component">...<p>
  </section>
</section>
```

Those custom scope attributes are then appended to every single selector in CSS:

``` text
p[data-scope~='main-component'] { color: red; }
p[data-scope~='sub-component'] { color: blue; }

/* both sections are part of the outer scope */
section[data-scope~='main-component'] { background: snow; }

/* the inner section is also part of the inner scope */
section[data-scope~='sub-component'] { color: ghostwhite; }
```

Using the <a id="ref-for-at-ruledef-scope⑨"></a>[&#64;scope](#at-ruledef-scope) rule, authors and tools can replicate similar behavior with the unique attribute or class applied only to the <a id="ref-for-scoping-root①②"></a>[scoping roots](https://drafts.csswg.org/selectors-4/#scoping-root):

``` text
<section data-scope="main-component">
  <p>...<p>
  <section data-scope="sub-component">
    <p>...<p>
  </section>
</section>
```

Then the class or attribute can be used for establishing both upper and lower boundaries. Elements matched by a lower boundary selector are excluded from the resulting scope, which allows authors to create non-overlapping scopes by default:

``` text
@scope ([data-scope='main-component']) to ([data-scope]) {
  p { color: red; }

  /* only the outer section is part of the outer scope */
  section { background: snow; }
}

@scope ([data-scope='sub-component']) to ([data-scope]) {
  p { color: blue; }

  /* the inner section is only part of the inner scope */
  section { color: ghostwhite; }
}
```

However, authors can use the child combinator and universal selector to create scope boundaries that overlap, such that the inner scope root is part of both scopes:

``` text
@scope ([data-scope='main-component']) to ([data-scope] > *) {
  p { color: red; }

  /* both sections are part of the outer scope */
  section { background: snow; }
}
```

#### <a id="scope-syntax"></a>3.5.2.  Syntax of <a id="ref-for-at-ruledef-scope①⓪"></a>[&#64;scope](#at-ruledef-scope)[](#scope-syntax)

The syntax of the <a id="ref-for-at-ruledef-scope①①"></a>[&#64;scope](#at-ruledef-scope) rule is:

<a id="ref-for-typedef-scope-boundaries①"></a><a id="ref-for-mult-opt①"></a><a id="ref-for-typedef-block-contents"></a>

``` text
@scope <scope-boundaries>? {
  <block-contents>
}
```

with <a id="ref-for-typedef-scope-boundaries②"></a>[\<scope-boundaries\>](#typedef-scope-boundaries) defined as:

<a id="typedef-scope-boundaries"></a><a id="ref-for-typedef-scope-start①"></a><a id="ref-for-mult-opt②"></a><a id="ref-for-typedef-scope-end"></a><a id="ref-for-mult-opt③"></a><a id="ref-for-mult-req"></a>

``` text
<scope-boundaries> =  [ [ ( <scope-start> ) ]? [ to ( <scope-end> ) ]? ]!
```

and where:

- <a id="typedef-scope-start"></a><strong><a id="ref-for-typedef-scope-start②"></a>[\<scope-start\>](#typedef-scope-start)</strong> is a <a id="ref-for-typedef-selector-list"></a>[\<selector-list\>](https://drafts.csswg.org/selectors-4/#typedef-selector-list) <a id="ref-for-selector"></a>[selector](https://drafts.csswg.org/selectors-4/#selector) used to identify the <a id="ref-for-scoping-root①③"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root)(s).

- <a id="typedef-scope-end"></a><strong><a id="ref-for-typedef-scope-end①"></a>[\<scope-end\>](#typedef-scope-end)</strong> is a <a id="ref-for-typedef-selector-list①"></a>[\<selector-list\>](https://drafts.csswg.org/selectors-4/#typedef-selector-list) <a id="ref-for-selector①"></a>[selector](https://drafts.csswg.org/selectors-4/#selector) used to identify any <a id="ref-for-scoping-limit⑤"></a>[scoping limits](#scoping-limit).

- the <a id="ref-for-qualified-rule"></a>[qualified rules](https://drafts.csswg.org/css-syntax-3/#qualified-rule) within <a id="ref-for-typedef-block-contents①"></a>[\<block-contents\>](https://drafts.csswg.org/css-syntax-3/#typedef-block-contents), as well as any <a id="ref-for-nested-declarations-rule"></a>[nested declarations rules](https://drafts.csswg.org/css-nesting-1/#nested-declarations-rule) produced by [scoped declarations](#scoped-declarations), represents the <a id="ref-for-scoped-style-rules③"></a>[scoped style rules](#scoped-style-rules).

<a id="ref-for-pseudo-element"></a>[Pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) cannot be <a id="ref-for-scoping-root①④"></a>[scoping roots](https://drafts.csswg.org/selectors-4/#scoping-root) or <a id="ref-for-scoping-limit⑥"></a>[scoping limits](#scoping-limit); they are invalid both within <a id="ref-for-typedef-scope-start③"></a>[\<scope-start\>](#typedef-scope-start) and <a id="ref-for-typedef-scope-end②"></a>[\<scope-end\>](#typedef-scope-end).

#### <a id="scoped-rules"></a>3.5.3.  Scoped Style Rules[](#scoped-rules)

<a id="scoped-style-rules"></a><strong>Scoped style rules</strong> differ from non-scoped rules in the following ways:

- Their selectors can only match elements that are <a id="ref-for-in-scope①"></a>[in scope](#in-scope). (This only applies to the <a id="ref-for-selector-subject②"></a>[subject](https://drafts.csswg.org/selectors-4/#selector-subject); the rest of the selector can match unrestricted.)

- They accept a <a id="ref-for-typedef-relative-selector-list"></a>[\<relative-selector-list\>](https://drafts.csswg.org/selectors-4/#typedef-relative-selector-list) as their prelude (rather than just a <a id="ref-for-typedef-selector-list②"></a>[\<selector-list\>](https://drafts.csswg.org/selectors-4/#typedef-selector-list)). Such <a id="ref-for-relative-selector①"></a>[relative selectors](https://drafts.csswg.org/selectors-4/#relative-selector) are relative to <a id="ref-for-scope-pseudo①"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo).

- Any selector in the <a id="ref-for-typedef-relative-selector-list①"></a>[\<relative-selector-list\>](https://drafts.csswg.org/selectors-4/#typedef-relative-selector-list) that does not start with a <a id="ref-for-selector-combinator"></a>[combinator](https://drafts.csswg.org/selectors-4/#selector-combinator) but does <a id="ref-for-contain-the-nesting-selector"></a>[contain the nesting selector](https://drafts.csswg.org/css-nesting-1/#contain-the-nesting-selector) or the <a id="ref-for-scope-pseudo②"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo) selector, is interpreted as a non-<a id="ref-for-relative-selector②"></a>[relative selector](https://drafts.csswg.org/selectors-4/#relative-selector) (but the <a id="ref-for-selector-subject③"></a>[subject](https://drafts.csswg.org/selectors-4/#selector-subject) must still be <a id="ref-for-in-scope②"></a>[in scope](#in-scope) to match).

<a id="example-fbedd7f3"></a>

<strong>Example:</strong>

[](#example-fbedd7f3) By default, selectors in a <a id="ref-for-scoped-style-rules④"></a>[scoped style rule](#scoped-style-rules) are <a id="ref-for-relative-selector③"></a>[relative selectors](https://drafts.csswg.org/selectors-4/#relative-selector), with the <a id="ref-for-scoping-root①⑤"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) and <a id="ref-for-descendant-combinator"></a>[descendant combinator](https://drafts.csswg.org/selectors-4/#descendant-combinator) implied at the start. The following selectors will match the same elements:

``` text
@scope (#my-component) {
  p { color: green; }
  :scope p { color: green; }
}
```

Authors can adjust the implied relationship by adding an explicit combinator:

``` text
@scope (#my-component) {
  > p { color: green; }
  :scope > p { color: green; }
}
```

Authors can also target or explicitly position the <a id="ref-for-scoping-root①⑥"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) in a selector by including either <a id="ref-for-scope-pseudo③"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo) or <a id="ref-for-selectordef-②"></a>[&](https://drafts.csswg.org/css-nesting-1/#selectordef-) in a given selector:

``` text
@scope (#my-component) {
  :scope { border: thin solid; }
  & { border: thin solid; }

  main :scope p { color: green; }
  main & p { color: green; }
}
```

Both the <a id="ref-for-scope-pseudo④"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo) and <a id="ref-for-selectordef-③"></a>[&](https://drafts.csswg.org/css-nesting-1/#selectordef-) selectors match the <a id="ref-for-scoping-root①⑦"></a>[scope root](https://drafts.csswg.org/selectors-4/#scoping-root), but with different <a id="ref-for-specificity"></a>[specificity](https://drafts.csswg.org/selectors-4/#specificity): <a id="ref-for-scope-pseudo⑤"></a>:scope has as <a id="ref-for-specificity①"></a>specificity of (0,1,0), whereas <a id="ref-for-selectordef-④"></a>& has a <a id="ref-for-specificity②"></a>specificity of 0.

#### <a id="scope-limits"></a>3.5.4.  Identifying Scoping Roots and Limits[](#scope-limits)

A <a id="ref-for-at-ruledef-scope①②"></a>[&#64;scope](#at-ruledef-scope) rule produces one or more <a id="ref-for-scope④"></a>[scopes](#scope) as follows:

<strong>Finding the <a id="ref-for-scoping-root①⑧"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root)(s)</strong>

For each element matched by <a id="ref-for-typedef-scope-start④"></a>[\<scope-start\>](#typedef-scope-start), create a <a id="ref-for-scope⑤"></a>[scope](#scope) using that element as the <a id="ref-for-scoping-root①⑨"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root). If no <a id="ref-for-typedef-scope-start⑤"></a>\<scope-start\> is specified, the <a id="ref-for-scoping-root②⓪"></a>scoping root is the <a id="ref-for-parent-element①"></a>[parent element](https://dom.spec.whatwg.org/#parent-element) of the <a id="ref-for-cssstyledeclaration-owner-node①"></a>[owner node](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-owner-node) of the stylesheet where the <a id="ref-for-at-ruledef-scope①③"></a>[&#64;scope](#at-ruledef-scope) rule is defined. (If no such element exists and the containing <a id="ref-for-concept-node-tree"></a>[node tree](https://dom.spec.whatwg.org/#concept-node-tree) is a <a id="ref-for-concept-shadow-tree②"></a>[shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), then the <a id="ref-for-scoping-root②①"></a>scoping root is the <u>shadow host</u>. Otherwise, the <a id="ref-for-scoping-root②②"></a>scoping root is the <a id="ref-for-concept-tree-root"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) of the containing <a id="ref-for-concept-node-tree①"></a>node tree.) Any <a id="ref-for-scope-pseudo⑥"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo) or <a id="ref-for-selectordef-⑤"></a>[&](https://drafts.csswg.org/css-nesting-1/#selectordef-) selectors in <a id="ref-for-typedef-scope-start⑥"></a>\<scope-start\> are interpreted as defined for its outer context.

<strong>Finding any <a id="ref-for-scoping-limit⑦"></a>[scoping limits](#scoping-limit)</strong>

For each <a id="ref-for-scope⑥"></a>[scope](#scope) created by a <a id="ref-for-scoping-root②③"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root), its <a id="ref-for-scoping-limit⑧"></a>[scoping limits](#scoping-limit) are set to all elements that are <a id="ref-for-concept-tree-descendant"></a>[descendants](https://dom.spec.whatwg.org/#concept-tree-descendant) of the <a id="ref-for-scoping-root②④"></a>scoping root and that match <a id="ref-for-typedef-scope-end③"></a>[\<scope-end\>](#typedef-scope-end), interpreting <a id="ref-for-scope-pseudo⑦"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo) and <a id="ref-for-selectordef-⑥"></a>[&](https://drafts.csswg.org/css-nesting-1/#selectordef-) exactly as in <a id="ref-for-scoped-style-rules⑤"></a>[scoped style rules](#scoped-style-rules).

<a id="example-52419898"></a>

<strong>Example:</strong>

[](#example-52419898) Authors can establish local scoping for <code><a id="ref-for-the-style-element"></a>[style](https://html&#46;spec&#46;whatwg&#46;org/multipage/semantics&#46;html#the-style-element)</code> elements by leaving out the <a id="ref-for-typedef-scope-start⑦"></a>[\<scope-start\>](#typedef-scope-start) selector. For example:

``` text
<div>
  <style>
    @scope {
      p { color: red; }
    }
  </style>
  <p>this is red</p>
</div>
<p>not red</p>
```

That would be equivalent to:

``` text
<div id="foo">
  <style>
    @scope (#foo) {
      p { color: red; }
    }
  </style>
  <p>this is red</p>
</div>
<p>not red</p>
```

<a id="example-132cb7e9"></a>

<strong>Example:</strong>

[](#example-132cb7e9) <a id="ref-for-scoping-limit⑨"></a>[Scoping limits](#scoping-limit) can use the <a id="ref-for-scope-pseudo⑧"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo) pseudo-class to require a specific relationship to the <a id="ref-for-scoping-root②⑤"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root):

``` text
/* .content is only a limit when it is a direct child of the :scope */
@scope (.media-object) to (:scope > .content) { ... }
```

<a id="ref-for-scoping-limit①⓪"></a>[Scoping limits](#scoping-limit) can also reference elements outside their <a id="ref-for-scoping-root②⑥"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) by using <a id="ref-for-scope-pseudo⑨"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo). For example:

``` text
/* .content is only a limit when the :scope is inside .sidebar */
@scope (.media-object) to (.sidebar :scope .content) { ... }
```

#### <a id="scope-scope"></a>3.5.5.  Scope Nesting[](#scope-scope)

<a id="ref-for-at-ruledef-scope①④"></a>[&#64;scope](#at-ruledef-scope) rules can be nested. In this case, just as with the nested style rules, the prelude selectors of the inner <a id="ref-for-at-ruledef-scope①⑤"></a>&#64;scope (those defining its <a id="ref-for-scope⑦"></a>[scope](#scope)) are <a id="ref-for-scoped-selector①"></a>[scoped by](https://drafts.csswg.org/selectors-4/#scoped-selector) the selectors of the outer one.

Note: The resulting <a id="ref-for-scope⑧"></a>[scope](#scope) for further nested <a id="ref-for-scoped-style-rules⑥"></a>[scoped style rules](#scoped-style-rules) is practically constrained by both the outer and inner <a id="ref-for-at-ruledef-scope①⑥"></a>[&#64;scope](#at-ruledef-scope) rules, but the <a id="ref-for-scoping-root②⑦"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) is defined by the innermost <a id="ref-for-at-ruledef-scope①⑦"></a>&#64;scope. Since <a id="ref-for-scope-proximity③"></a>[scope proximity](#scope-proximity) is measured between a <a id="ref-for-scoped-style-rules⑦"></a>scoped style rule <a id="ref-for-selector-subject④"></a>[subject](https://drafts.csswg.org/selectors-4/#selector-subject) and <a id="ref-for-scoping-root②⑧"></a>scoping root, only the innermost <a id="ref-for-at-ruledef-scope①⑧"></a>&#64;scope matters for determining <a id="ref-for-scope-proximity④"></a>scope proximity of [nested &#64;scope rules](#scope-scope).

<a id="issue-a2203bb7"></a>

<strong>Issue:</strong>

[](#issue-a2203bb7) Should the scope proximity calculation be impacted by nesting scopes? [\[Issue \#10795\]](https://github.com/w3c/csswg-drafts/issues/10795)

<a id="example-49f2905a"></a>

<strong>Example:</strong>

[](#example-49f2905a) When nesting <a id="ref-for-at-ruledef-scope①⑨"></a>[&#64;scope](#at-ruledef-scope) rules inside other <a id="ref-for-at-ruledef-scope②⓪"></a>&#64;scope rules, or inside other selectors, the <a id="ref-for-typedef-scope-start⑧"></a>[\<scope-start\>](#typedef-scope-start) selector is <a id="ref-for-relative-selector④"></a>[relative to](https://drafts.csswg.org/selectors-4/#relative-selector) the nesting context, while the <a id="ref-for-typedef-scope-end④"></a>[\<scope-end\>](#typedef-scope-end) and any <a id="ref-for-scoped-style-rules⑧"></a>[scoped style rules](#scoped-style-rules) are <a id="ref-for-relative-selector⑤"></a>relative to the <a id="ref-for-scoping-root②⑨"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) For example, the following code:

``` text
@scope (.parent-scope) {
  @scope (:scope > .child-scope) to (:scope .limit) {
    :scope .content {
      color: red;
    }
  }
}
```

is equivalent to:

``` text
@scope (.parent-scope > .child-scope) to (.parent-scope > .child-scope .limit) {
  .parent-scope > .child-scope .content {
    color: red;
  }
}
```

Global name-defining <a id="ref-for-at-rule①"></a>[at-rules](https://drafts.csswg.org/css-syntax-3/#at-rule) such as <a id="ref-for-at-ruledef-keyframes"></a>[&#64;keyframes](https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes) or <a id="ref-for-at-font-face-rule"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-5/#at-font-face-rule) or <a id="ref-for-at-ruledef-layer①"></a>[&#64;layer](https://drafts.csswg.org/css-cascade-5/#at-ruledef-layer) that are defined inside <a id="ref-for-at-ruledef-scope②①"></a>[&#64;scope](#at-ruledef-scope) are valid, but are not scoped or otherwise affected by the enclosing <a id="ref-for-at-ruledef-scope②②"></a>&#64;scope rule. However, any <a id="ref-for-style-rule⑤"></a>[style rules](https://drafts.csswg.org/css-syntax-3/#style-rule) contained by such rules (e.g. within <a id="ref-for-at-ruledef-layer②"></a>&#64;layer) are <a id="ref-for-scoped-style-rules⑨"></a>[scoped](#scoped-style-rules).

#### <a id="scoped-declarations"></a>3.5.6.  Scoped Declarations [](#scoped-declarations)

<a id="ref-for-cssstyledeclaration-declarations⑤"></a>[Declarations](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-declarations) may be used directly with the body of a <a id="ref-for-at-ruledef-scope②③"></a>[&#64;scope](#at-ruledef-scope) rule. Contiguous runs of <a id="ref-for-cssstyledeclaration-declarations⑥"></a>declarations are wrapped in <a id="ref-for-nested-declarations-rule①"></a>[nested declarations rules](https://drafts.csswg.org/css-nesting-1/#nested-declarations-rule), which match the <a id="ref-for-scoping-root③⓪"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) with zero <a id="ref-for-specificity③"></a>[specificity](https://drafts.csswg.org/selectors-4/#specificity).

<a id="example-10a88995"></a>

<strong>Example:</strong>

[](#example-10a88995)

``` text
@scope (.foo) {
  border: 1px solid black;
}
```

is equivalent to:

``` text
@scope (.foo) {
  :where(:scope) {
    border: 1px solid black;
  }
}
```

Just like for <a id="ref-for-style-rule⑥"></a>[style rules](https://drafts.csswg.org/css-syntax-3/#style-rule), <a id="ref-for-cssstyledeclaration-declarations⑦"></a>[declarations](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-declarations) and child <a id="ref-for-css-rule"></a>[rules](https://drafts.csswg.org/css-syntax-3/#css-rule) may be [mixed](https://drafts.csswg.org/css-nesting-1/#mixing) within <a id="ref-for-at-ruledef-scope②④"></a>[&#64;scope](#at-ruledef-scope).

### <a id="preshint"></a>3.6.  Precedence of Non-CSS Presentational Hints[](#preshint)

<a id="issue-d41d8cd9④"></a>

<strong>Issue:</strong>

[](#issue-d41d8cd9%E2%91%A3) [CSS Cascading 5 § 6.4 Cascade Layers](https://drafts.csswg.org/css-cascade-5/#layering)

## <a id="cssom"></a>4. CSSOM[](#cssom)

### <a id="the-cssscoperule-interface"></a>4.1.  The <code>CSSScopeRule</code> interface[](#the-cssscoperule-interface)

The <code><a id="ref-for-cssscoperule"></a>[CSSScopeRule](#cssscoperule)</code> interface represents the <a id="ref-for-at-ruledef-scope②⑤"></a>[&#64;scope](#at-ruledef-scope) rule:

<a id="ref-for-Exposed"></a><a id="cssscoperule"></a><a id="ref-for-cssgroupingrule"></a><a id="ref-for-cssomstring"></a><a id="dom-cssscoperule-start"></a><a id="ref-for-cssomstring①"></a><a id="dom-cssscoperule-end"></a>

``` text
[Exposed=Window]
interface CSSScopeRule : CSSGroupingRule {
  readonly attribute CSSOMString? start;
  readonly attribute CSSOMString? end;
};
```

<strong><code>start</code> of type <code>CSSOMString</code></strong>

The <code>start</code> attribute returns the result of serializing the <a id="ref-for-typedef-scope-start⑨"></a>[\<scope-start\>](#typedef-scope-start) of the rule (without the enclosing parentheses), or null if there is no <a id="ref-for-typedef-scope-start①⓪"></a>\<scope-start\>.

<strong><code>end</code> of type <code>CSSOMString</code></strong>

The <code>end</code> attribute returns the result of serializing the <a id="ref-for-typedef-scope-end⑤"></a>[\<scope-end\>](#typedef-scope-end) of the rule (without the enclosing parentheses), or null if there is no <a id="ref-for-typedef-scope-end⑥"></a>\<scope-end\>.

## <a id="changes"></a>5.  Changes[](#changes)

This appendix is <em>informative</em>.

### <a id="changes-since-2023-03"></a>5.1.  Changes since the 21 March 2023 Working Draft[](#changes-since-2023-03)

Significant changes since the [21 March 2023 Working Draft](https://www.w3.org/TR/2023/WD-css-cascade-6-20230321/) include:

- Added the <code>@import scope()</code> syntax, and the ability to specificy layer(), scope(), and <a id="ref-for-funcdef-supports"></a>[supports()](https://drafts.csswg.org/css-conditional-5/#funcdef-supports) in any order. ([Issue 7348](https://github.com/w3c/csswg-drafts/issues/7348))

- Defined <a id="ref-for-selectordef-⑦"></a>[&](https://drafts.csswg.org/css-nesting-1/#selectordef-) to behave like <code>&#58;where(&#58;scope)</code>. ([Issue 9740](https://github.com/w3c/csswg-drafts/issues/9740))

- Allowed <a id="ref-for-cssstyledeclaration-declarations⑧"></a>[declarations](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-declarations) directly within <a id="ref-for-at-ruledef-scope②⑥"></a>[&#64;scope](#at-ruledef-scope). ([Issue 10389](https://github.com/w3c/csswg-drafts/issues/10389))

- The <a id="ref-for-scope-pseudo①⓪"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo) selector can match the <a id="ref-for-featureless①"></a>[featureless](https://drafts.csswg.org/selectors-4/#featureless) <u>shadow host</u> when that host is the <a id="ref-for-scoping-root③①"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) element. ([Issue 9025](https://github.com/w3c/csswg-drafts/issues/9025))

- <a id="ref-for-typedef-scope-start①①"></a>[\<scope-start\>](#typedef-scope-start) and <a id="ref-for-typedef-scope-end⑦"></a>[\<scope-end\>](#typedef-scope-end) selectors are unforgiving. ([Issue 10042](https://github.com/w3c/csswg-drafts/issues/10042))

- A <a id="ref-for-at-ruledef-scope②⑦"></a>[&#64;scope](#at-ruledef-scope) rule without <a id="ref-for-typedef-scope-start①②"></a>[\<scope-start\>](#typedef-scope-start) scopes to the <u>shadow host</u> instead of the <a id="ref-for-concept-shadow-root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root). ([Issue 9178](https://github.com/w3c/csswg-drafts/issues/9178))

- Clarified that <a id="ref-for-scope-proximity⑤"></a>[scope proximity](#scope-proximity) is a single measurement of the steps between a single <a id="ref-for-scoping-root③②"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) and <a id="ref-for-scoped-style-rules①⓪"></a>[scoped style rule](#scoped-style-rules) <a id="ref-for-selector-subject⑤"></a>[subject](https://drafts.csswg.org/selectors-4/#selector-subject) ([Issue 10795](https://github.com/w3c/csswg-drafts/issues/10795) has been opened to discuss this futher).

- Removed strong scope proximity. ([Issue 6790](https://github.com/w3c/csswg-drafts/issues/6790))

- Removed the scoped descendant combinator (deferred). ([Issue 8628](https://github.com/w3c/csswg-drafts/issues/8628))

- Added the <code><a id="ref-for-cssscoperule①"></a>[CSSScopeRule](#cssscoperule)</code> interface. ([Issue 8626](https://github.com/w3c/csswg-drafts/issues/8626))

### <a id="changes-2022-08"></a>5.2.  Changes since the 21 December 2021 First Public Working Draft[](#changes-2022-08)

Significant changes since the [21 December 2021 First Public Working Draft](https://www.w3.org/TR/2021/WD-css-cascade-6-20211221/) include:

- Clarified <a id="ref-for-at-ruledef-scope②⑧"></a>[&#64;scope](#at-ruledef-scope) effects on nested <a id="ref-for-scope-pseudo①①"></a>[:scope](https://drafts.csswg.org/selectors-4/#scope-pseudo) and <a id="ref-for-selectordef-⑧"></a>[&](https://drafts.csswg.org/css-nesting-1/#selectordef-) selectors. ([Issue 8377](https://github.com/w3c/csswg-drafts/issues/8377))

- Removed <a id="ref-for-at-ruledef-scope②⑨"></a>[&#64;scope](#at-ruledef-scope) prelude from specificity calculation. ([Issue 8500](https://github.com/w3c/csswg-drafts/issues/8500))

- Specified how name-defining <a id="ref-for-at-rule②"></a>[at-rules](https://drafts.csswg.org/css-syntax-3/#at-rule) behave in <a id="ref-for-at-ruledef-scope③⓪"></a>[&#64;scope](#at-ruledef-scope). ([Issue 6895](https://github.com/w3c/csswg-drafts/issues/6895))

- Added implicit scopes by making <a id="ref-for-typedef-scope-start①③"></a>[\<scope-start\>](#typedef-scope-start) optional. ([Issue 6606](https://github.com/w3c/csswg-drafts/issues/6606))

- Disallowed <a id="ref-for-pseudo-element①"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) in the <a id="ref-for-at-ruledef-scope③①"></a>[&#64;scope](#at-ruledef-scope) prelude. ([Issue 7382](https://github.com/w3c/csswg-drafts/issues/7382))

- Removed selector scoping notation. ([Issue 7709](https://github.com/w3c/csswg-drafts/issues/7709))

- <a id="ref-for-scoping-limit①①"></a>[Scoping limit](#scoping-limit) elements are excluded from the resulting <a id="ref-for-scope⑨"></a>[scope](#scope). ([Issue 6577](https://github.com/w3c/csswg-drafts/issues/6577))

### <a id="additions-l5"></a>5.3.  Additions Since Level 5[](#additions-l5)

The following features have been added since [Level 5](https://www.w3.org/TR/css-cascade-5/):

- The definition of a <a id="ref-for-scope①⓪"></a>[scope](#scope), as described by a combination of <a id="ref-for-typedef-scope-start①④"></a>[\<scope-start\>](#typedef-scope-start) and <a id="ref-for-typedef-scope-end⑧"></a>[\<scope-end\>](#typedef-scope-end) selectors.

- The in-scope (:in()) pseudo-class for selecting with lower-boundaries

- The <a id="ref-for-at-ruledef-scope③②"></a>[&#64;scope](#at-ruledef-scope) rule for creating scoped stylesheets

- The definition of <a id="ref-for-scope-proximity⑥"></a>[scope proximity](#scope-proximity) in the cascade

### <a id="additions-l4"></a>5.4.  Additions Since Level 4[](#additions-l4)

The following features have been added since [Level 4](https://www.w3.org/TR/css-cascade-4/):

- Added <a id="ref-for-cascade-layers⑤"></a>[cascade layers](https://drafts.csswg.org/css-cascade-5/#cascade-layers) to the <a id="ref-for-cascade①"></a>[cascade](#cascade) sort criteria (and defined style attributes as a distinct step of the <a id="ref-for-cascade②"></a>cascade sort criteria so that they interact appropriately).

- Introduced the <a id="ref-for-at-ruledef-layer③"></a>[&#64;layer](https://drafts.csswg.org/css-cascade-5/#at-ruledef-layer) rule for defining cascade layers.

- Added layer/layer() option to <a id="ref-for-at-ruledef-import①⑥"></a>[&#64;import](#at-ruledef-import) definition.

- Introduced the <a id="ref-for-valdef-all-revert-layer"></a>[revert-layer](https://drafts.csswg.org/css-cascade-5/#valdef-all-revert-layer) keyword for rolling back values to previous layers.

### <a id="additions-l3"></a>5.5.  Additions Since Level 3[](#additions-l3)

The following features have been added since [Level 3](https://www.w3.org/TR/css-cascade-3/):

- Introduced <a id="ref-for-valdef-all-revert"></a>[revert](https://drafts.csswg.org/css-cascade-5/#valdef-all-revert) keyword, for rolling back the cascade.

- Introduced <a id="ref-for-funcdef-supports①"></a>[supports()](https://drafts.csswg.org/css-conditional-5/#funcdef-supports) syntax for supports-conditional <a id="ref-for-at-ruledef-import①⑦"></a>[&#64;import](#at-ruledef-import) rules.

- Added <a id="ref-for-encapsulation-contexts④"></a>[encapsulation context](#encapsulation-contexts) to the <a id="ref-for-cascade③"></a>[cascade](#cascade) sort criteria to accommodate Shadow DOM. [\[DOM\]](#biblio-dom)

- Defined the property two aliasing mechanisms CSS uses to support legacy syntaxes. See [CSS Cascading 4 § 3.1 Property Aliasing](https://drafts.csswg.org/css-cascade-4/#aliasing).

### <a id="changes-2"></a>5.6.  Additions Since Level 2[](#changes-2)

The following features have been added since [Level 2](http://www.w3.org/TR/CSS2/cascade.html):

- The <a id="ref-for-propdef-all"></a>[all](https://drafts.csswg.org/css-cascade-5/#propdef-all) shorthand
- The <a id="ref-for-valdef-all-initial"></a>[initial](https://drafts.csswg.org/css-cascade-5/#valdef-all-initial) keyword
- The <a id="ref-for-valdef-all-unset"></a>[unset](https://drafts.csswg.org/css-cascade-5/#valdef-all-unset) keyword
- Incorporation of animations and transitions into the <a id="ref-for-cascade④"></a>[cascade](#cascade).

## <a id="acknowledgments"></a>Acknowledgments[](#acknowledgments)

David Baron, Tantek Çelik, Keith Grant, Giuseppe Gurgone, Theresa O’Connor, Florian Rivoal, Noam Rosenthal, Simon Sapin, Jen Simmons, Nicole Sullivan, Lea Verou, and Boris Zbarsky contributed to this specification.

## <a id="privacy"></a>6.  Privacy Considerations[](#privacy)

- User preferences and UA defaults expressed via application of style rules are exposed by the cascade process, and can be inferred from the computed styles they apply to a document.

## <a id="security"></a>7.  Security Considerations[](#security)

- The cascade process does not distinguish between same-origin and cross-origin stylesheets, enabling the content of cross-origin stylesheets to be inferred from the computed styles they apply to a document.

- The <a id="ref-for-at-ruledef-import①⑧"></a>[&#64;import](#at-ruledef-import) rule does not apply the <a id="ref-for-cors-protocol"></a>[CORS protocol](https://fetch.spec.whatwg.org/#cors-protocol) to loading cross-origin stylesheets, instead allowing them to be freely imported and applied.

- The <a id="ref-for-at-ruledef-import①⑨"></a>[&#64;import](#at-ruledef-import) rule assumes that resources without <a id="5ca12a2a0"></a>[<code>Content-Type</code> metadata](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#content-type) (or any same-origin file if the host document is in quirks mode) are <code>text/css</code>, potentially allowing arbitrary files to be imported into the page and interpreted as CSS, potentially allowing sensitive data to be inferred from the computed styles they apply to a document.

## <a id="w3c-conformance"></a> Conformance[](#w3c-conformance)

### <a id="w3c-conventions"></a> Document conventions[](#w3c-conventions)

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with <code>class=&#34;example&#34;</code>, like this:

<a id="w3c-example"></a>

<strong>Example:</strong>

[](#w3c-example)

This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with <code>class=&#34;note&#34;</code>, like this:

Note, this is an informative note.

<strong>Note:</strong>

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with <code>&#60;strong class=&#34;advisement&#34;&#62;</code>, like this: <strong>UAs MUST provide an accessible alternative.</strong>

<strong>Advisement:</strong>

Tests

Tests relating to the content of this specification may be documented in “Tests” blocks like this one. Any such block is non-normative.

------------------------------------------------------------------------

### <a id="w3c-conformance-classes"></a> Conformance classes[](#w3c-conformance-classes)

Conformance to this specification is defined for three conformance classes:

<strong>style sheet</strong>

A [CSS style sheet](http://www.w3.org/TR/CSS21/conform.html#style-sheet).

<strong>renderer</strong>

A [UA](http://www.w3.org/TR/CSS21/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

<strong>authoring tool</strong>

A [UA](http://www.w3.org/TR/CSS21/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="w3c-partial"></a> Partial implementations[](#w3c-partial)

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](http://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="w3c-conform-future-proofing"></a> Implementations of Unstable and Proprietary Features[](#w3c-conform-future-proofing)

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](http://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](http://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](http://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

### <a id="w3c-testing"></a> Non-experimental implementations[](#w3c-testing)

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at <http://www.w3.org/Style/CSS/Test/>. Questions should be directed to the [public-css-testsuite&#64;w3.org](http://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index[](#index)

### <a id="index-defined-here"></a>Terms defined by this specification[](#index-defined-here)

- [cascade](#cascade), in § 3
- [cascade origin](#origin), in § 3.2
- [context](#encapsulation-contexts), in § 3.1
- [CSSScopeRule](#cssscoperule), in § 4.1
- [encapsulation contexts](#encapsulation-contexts), in § 3.1
- [end](#dom-cssscoperule-end), in § 4.1
- [&#64;import](#at-ruledef-import), in § 2
- [importance](#important), in § 3.3
- [important](#important), in § 3.3
- [Import conditions](#import-conditions), in § 2.1
- [in scope](#in-scope), in § 3.5
- [\<media-import-condition\>](#typedef-media-import-condition), in § 2
- [normal](#normal), in § 3.3
- [origin](#origin), in § 3.2
- [output of the cascade](#output-of-the-cascade), in § 3.1
- [&#64;scope](#at-ruledef-scope), in § 3.5
- [scope](#scope), in § 3.5
- [\<scope-boundaries\>](#typedef-scope-boundaries), in § 3.5.2
- [Scoped style rules](#scoped-style-rules), in § 3.5.3
- [\<scope-end\>](#typedef-scope-end), in § 3.5.2
- [Scope Proximity](#scope-proximity), in § 3.1
- [\<scope-start\>](#typedef-scope-start), in § 3.5.2
- [scoping limit](#scoping-limit), in § 3.5
- [start](#dom-cssscoperule-start), in § 4.1
- [\<supports-import-condition\>](#typedef-supports-import-condition), in § 2

### <a id="index-defined-elsewhere"></a>Terms defined by reference[](#index-defined-elsewhere)

- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="fa19c68e"></a>&#64;keyframes
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="02eb2842"></a>\<layer-name\>
  - <a id="94b8f87c"></a>&#64;layer
  - <a id="d95397a7"></a>all
  - <a id="39e92270"></a>author origin
  - <a id="4a88f201"></a>cascade layers
  - <a id="c665612f"></a>cascaded value
  - <a id="7c8e67c1"></a>declared value
  - <a id="4d7d3dcd"></a>initial
  - <a id="bef88b2d"></a>revert
  - <a id="8e92a2a0"></a>revert-layer
  - <a id="9e004c39"></a>unset
  - <a id="621960b6"></a>user origin
  - <a id="63fe4250"></a>user-agent origin
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="12432dcb"></a>\<supports-condition\>
  - <a id="5f1b7f60"></a>&#64;media
  - <a id="c1ebc639"></a>&#64;supports
  - <a id="bf3ca8a7"></a>conditional group rule
  - <a id="b0326703"></a>supports queries
- \[CSS-CONDITIONAL-5\] defines the following terms:
  - <a id="c4088e16"></a>\<declaration\>
  - <a id="5f6dfe0c"></a>\<supports-decl\>
  - <a id="0ba48d22"></a>&#64;supports-condition
  - <a id="a15951ea"></a>supports()
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="e7f0dd6c"></a>display
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="0497de86"></a>&#64;font-face
- \[CSS-NAMESPACES-3\] defines the following terms:
  - <a id="bf0e8186"></a>&#64;namespace
- \[CSS-NESTING-1\] defines the following terms:
  - <a id="a5ad37b5"></a>&
  - <a id="5140976e"></a>contain the nesting selector
  - <a id="9427c0d7"></a>nested declarations rule
  - <a id="43324e46"></a>nested style rule
- \[CSS-SHADOW-1\] defines the following terms:
  - <a id="528ed77d"></a>tree context
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="b845e85c"></a>\<block-contents\>
  - <a id="65c9f2cf"></a>\<rule-list\>
  - <a id="c533ddd8"></a>&#64;charset
  - <a id="762610c7"></a>at-rule
  - <a id="0197874e"></a>block at-rule
  - <a id="dfd120f7"></a>declaration
  - <a id="7e4aa58b"></a>environment encoding
  - <a id="47948caa"></a>qualified rule
  - <a id="4fcdc12a"></a>rule
  - <a id="5a58bbb0"></a>style rule
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="ffd03070"></a>!
  - <a id="8c1eb8d0"></a>?
  - <a id="6a56abc9"></a>url()
  - <a id="2bd7646e"></a>\|
  - <a id="5c4ea22d"></a>\|\|
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="977d3003"></a>\<string\>
  - <a id="fa293cdd"></a>\<url\>
- \[CSSOM-1\] defines the following terms:
  - <a id="2485e0d1"></a>CSSGroupingRule
  - <a id="5fced98b"></a>CSSOMString
  - <a id="e4ffca7f"></a>declarations
  - <a id="eab73120"></a>owner node
- \[DOM\] defines the following terms:
  - <a id="da8b8e0e"></a>descendant
  - <a id="f9d909f7"></a>inclusive descendant
  - <a id="d462b34f"></a>node
  - <a id="c7d8d91b"></a>node tree
  - <a id="5afeceea"></a>parent element
  - <a id="fd11cdcd"></a>quirks mode
  - <a id="f7960529"></a>root
  - <a id="3fcc582f"></a>shadow root
  - <a id="19f9e9df"></a>shadow tree
  - <a id="fefa5851"></a>shadow-including tree order
- \[FETCH\] defines the following terms:
  - <a id="a2a30ba8"></a>CORS protocol
  - <a id="ee7bba09"></a>response
  - <a id="3268a8eb"></a>URL
- \[HTML\] defines the following terms:
  - <a id="0fe1fd87"></a>Content-Type metadata
  - <a id="f0811ff8"></a>img
  - <a id="7393da89"></a>same origin
  - <a id="ba920583"></a>style
- \[MEDIAQ\] defines the following terms:
  - <a id="4c891868"></a>\<media-query-list\>
  - <a id="5424dbef"></a>media query list
- \[SELECTORS-4\] defines the following terms:
  - <a id="dc55c3d9"></a>:scope
  - <a id="ea722d19"></a>\<relative-selector-list\>
  - <a id="71aed09f"></a>\<selector-list\>
  - <a id="f4fc17d1"></a>combinator
  - <a id="5f4f98ab"></a>descendant combinator
  - <a id="3a1154bf"></a>featureless
  - <a id="42b00c6e"></a>pseudo-elements
  - <a id="4cd9dfd7"></a>relative selector
  - <a id="3814f164"></a>scoped selector
  - <a id="34c941b4"></a>scoping root
  - <a id="33cbcfca"></a>selector
  - <a id="bb2056aa"></a>specificity
  - <a id="b894d2f3"></a>subject
- \[WEBIDL\] defines the following terms:
  - <a id="889e932f"></a>Exposed

## <a id="references"></a>References[](#references)

### <a id="normative"></a>Normative References[](#normative)

<a id="biblio-css-animations-1"></a><strong>\[CSS-ANIMATIONS-1\]</strong>

David Baron; et al. [CSS Animations Level 1](https://drafts.csswg.org/css-animations/). URL: <https://drafts.csswg.org/css-animations/>

<a id="biblio-css-cascade-4"></a><strong>\[CSS-CASCADE-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://drafts.csswg.org/css-cascade-4/). URL: <https://drafts.csswg.org/css-cascade-4/>

<a id="biblio-css-cascade-5"></a><strong>\[CSS-CASCADE-5\]</strong>

Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://drafts.csswg.org/css-cascade-5/). URL: <https://drafts.csswg.org/css-cascade-5/>

<a id="biblio-css-conditional-3"></a><strong>\[CSS-CONDITIONAL-3\]</strong>

Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://drafts.csswg.org/css-conditional-3/). URL: <https://drafts.csswg.org/css-conditional-3/>

<a id="biblio-css-conditional-5"></a><strong>\[CSS-CONDITIONAL-5\]</strong>

Chris Lilley; et al. [CSS Conditional Rules Module Level 5](https://drafts.csswg.org/css-conditional-5/). URL: <https://drafts.csswg.org/css-conditional-5/>

<a id="biblio-css-fonts-5"></a><strong>\[CSS-FONTS-5\]</strong>

Chris Lilley. [CSS Fonts Module Level 5](https://drafts.csswg.org/css-fonts-5/). URL: <https://drafts.csswg.org/css-fonts-5/>

<a id="biblio-css-namespaces-3"></a><strong>\[CSS-NAMESPACES-3\]</strong>

Elika Etemad. [CSS Namespaces Module Level 3](https://drafts.csswg.org/css-namespaces/). URL: <https://drafts.csswg.org/css-namespaces/>

<a id="biblio-css-nesting-1"></a><strong>\[CSS-NESTING-1\]</strong>

Tab Atkins Jr.. [CSS Nesting Module Level 1](https://drafts.csswg.org/css-nesting/). URL: <https://drafts.csswg.org/css-nesting/>

<a id="biblio-css-shadow-1"></a><strong>\[CSS-SHADOW-1\]</strong>

[CSS Shadow Module Level 1](https://drafts.csswg.org/css-shadow-1/). Editor's Draft. URL: <https://drafts.csswg.org/css-shadow-1/>

<a id="biblio-css-syntax-3"></a><strong>\[CSS-SYNTAX-3\]</strong>

Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/). URL: <https://drafts.csswg.org/css-syntax/>

<a id="biblio-css-transitions-1"></a><strong>\[CSS-TRANSITIONS-1\]</strong>

Chris Marrin; et al. [CSS Transitions Module Level 1](https://drafts.csswg.org/css-transitions/). URL: <https://drafts.csswg.org/css-transitions/>

<a id="biblio-css-values-3"></a><strong>\[CSS-VALUES-3\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://drafts.csswg.org/css-values-3/). URL: <https://drafts.csswg.org/css-values-3/>

<a id="biblio-css-values-4"></a><strong>\[CSS-VALUES-4\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: <https://drafts.csswg.org/css-values-4/>

<a id="biblio-cssom-1"></a><strong>\[CSSOM-1\]</strong>

Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://drafts.csswg.org/cssom/). URL: <https://drafts.csswg.org/cssom/>

<a id="biblio-cssstyleattr"></a><strong>\[CSSSTYLEATTR\]</strong>

Tantek Çelik; Elika Etemad. [CSS Style Attributes](https://drafts.csswg.org/css-style-attr/). URL: <https://drafts.csswg.org/css-style-attr/>

<a id="biblio-dom"></a><strong>\[DOM\]</strong>

Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: <https://dom.spec.whatwg.org/>

<a id="biblio-fetch"></a><strong>\[FETCH\]</strong>

Anne van Kesteren. [Fetch Standard](https://fetch.spec.whatwg.org/). Living Standard. URL: <https://fetch.spec.whatwg.org/>

<a id="biblio-html"></a><strong>\[HTML\]</strong>

Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: <https://html.spec.whatwg.org/multipage/>

<a id="biblio-mediaq"></a><strong>\[MEDIAQ\]</strong>

Tab Atkins Jr.; Florian Rivoal. [Media Queries Level 4](https://drafts.csswg.org/mediaqueries-4/). URL: <https://drafts.csswg.org/mediaqueries-4/>

<a id="biblio-rfc2119"></a><strong>\[RFC2119\]</strong>

S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: <https://datatracker.ietf.org/doc/html/rfc2119>

<a id="biblio-select"></a><strong>\[SELECT\]</strong>

Tantek Çelik; et al. [Selectors Level 3](https://drafts.csswg.org/selectors-3/). URL: <https://drafts.csswg.org/selectors-3/>

<a id="biblio-selectors-4"></a><strong>\[SELECTORS-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://drafts.csswg.org/selectors/). URL: <https://drafts.csswg.org/selectors/>

<a id="biblio-webidl"></a><strong>\[WEBIDL\]</strong>

Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: <https://webidl.spec.whatwg.org/>

### <a id="informative"></a>Non-Normative References[](#informative)

<a id="biblio-css-display-4"></a><strong>\[CSS-DISPLAY-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://drafts.csswg.org/css-display-4/). URL: <https://drafts.csswg.org/css-display-4/>

<a id="biblio-css-scoping-1"></a><strong>\[CSS-SCOPING-1\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Scoping Module Level 1](https://drafts.csswg.org/css-scoping/). URL: <https://drafts.csswg.org/css-scoping/>

## <a id="idl-index"></a>IDL Index[](#idl-index)

``` text
[Exposed=Window]
interface CSSScopeRule : CSSGroupingRule {
  readonly attribute CSSOMString? start;
  readonly attribute CSSOMString? end;
};
```

## <a id="issues-index"></a>Issues Index[](#issues-index)

This is a diff spec over

<strong>Issue:</strong>

[CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 5 as a reference. We will merge the Level 5 text into this draft once it reaches CR. [↵](#issue-a783701e)

<strong>Issue:</strong>

[CSS Cascading 5 § 6.2 Cascading Origins](https://drafts.csswg.org/css-cascade-5/#cascading-origins) [↵](#issue-d41d8cd9)

<strong>Issue:</strong>

[CSS Cascading 5 § 6.3 Important Declarations: the !important annotation](https://drafts.csswg.org/css-cascade-5/#importance) [↵](#issue-d41d8cd9%E2%91%A0)

<strong>Issue:</strong>

[CSS Cascading 5 § 6.4 Cascade Layers](https://drafts.csswg.org/css-cascade-5/#layering) [↵](#issue-d41d8cd9%E2%91%A1)

<strong>Issue:</strong>

[CSS Cascading 5 § 6.4.3 Layer Ordering](https://drafts.csswg.org/css-cascade-5/#layer-ordering) [↵](#issue-d41d8cd9%E2%91%A2)

Should scoping limits be added to the definition of

<strong>Issue:</strong>

[scoped selectors](https://drafts.csswg.org/selectors-4/#scoped-selector)? [↵](#issue-acd126a1)

Should the scope proximity calculation be impacted by nesting scopes?

<strong>Issue:</strong>

[\[Issue \#10795\]](https://github.com/w3c/csswg-drafts/issues/10795) [↵](#issue-a2203bb7)

<strong>Issue:</strong>

[CSS Cascading 5 § 6.4 Cascade Layers](https://drafts.csswg.org/css-cascade-5/#layering) [↵](#issue-d41d8cd9%E2%91%A3)

<strong>✔</strong>MDN

[all](https://developer.mozilla.org/en-US/docs/Web/CSS/all)

In all current engines.

Firefox27+Safari9.1+Chrome37+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[inherit](https://developer.mozilla.org/en-US/docs/Web/CSS/inherit)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera4+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE8+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[initial](https://developer.mozilla.org/en-US/docs/Web/CSS/initial)

In all current engines.

Firefox19+Safari1.2+Chrome1+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)13+IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[revert](https://developer.mozilla.org/en-US/docs/Web/CSS/revert)

In all current engines.

Firefox67+Safari9.1+Chrome84+

------------------------------------------------------------------------

Opera?Edge84+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[unset](https://developer.mozilla.org/en-US/docs/Web/CSS/unset)

In all current engines.

Firefox27+Safari9.1+Chrome41+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)13+IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?
