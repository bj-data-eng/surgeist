Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Conditional Rules Module Level 5](https://drafts.csswg.org/css-conditional-5/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply. The capture’s full original notice and links remain below.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md#cssom-planning-reference-refresh--2026-10-09).

# Source provenance

Title: CSS Conditional Rules Module Level 5

Source snapshot: https://drafts.csswg.org/css-conditional-5/

Retrieved: 2026-10-09. The captured page states W3C Working Draft, 8 October 2026. An undated editor URL can change; the retrieval date and exact byte hash identify this captured HTML.

Captured HTML SHA-256: 75e15be8c8b41c6c34de2fa861801dbc6b77a06e6aba79d397bad82608e0aac1

Captured HTML revision metadata: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Representation notes:

- Full format conversion of the captured HTML body, including status, metadata, bibliography, indexes, examples, test references, and legal notice. Script and style elements are omitted and were not executed.
- All source body IDs are retained, including those inside preformatted blocks, which are relocated immediately before those blocks. Fragment-only links stay local only when their exact captured target exists; other links remain upstream.
- All 16 tables have readable Markdown layouts. All tables have ordinary row/column layouts. Synthetic Field/Definition or Column N headings are non-normative. Source row headers remain bold; native HTML header/accessibility semantics are not expressible in GFM.
- Preformatted examples retain literal text. Single-line table examples use semantic inline code. Compiler-generated hyperlinks inside fenced code blocks are omitted while their IDs and visible code text are retained; other source links are preserved. Small semantic code, variable, emphasis, subscript, and superscript HTML remains where Markdown notation would alter text.
- Conversion uses Pandoc 3.1.11.1 with focused Python/lxml preparation and a semantic Lua filter; the parsed GFM output is checked against the captured HTML. The [conversion report](CONVERSION-REPORT.md#cssom-planning-reference-refresh--2026-10-09) records provenance and check boundaries.
- This full edition is the selected planning reference for the new CSSOM work. The [source-edition mapping](SOURCE-EDITIONS.md#cssom-planning-sources) distinguishes these planning selections from historical editions consumed by completed CSS work.

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

[Inline In Spec](#issues-index)

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

## <a id="abstract"></a>Abstract

This module contains the features of CSS for conditional processing of parts of style sheets, based on capabilities of the processor or the environment the style sheet is being applied in. It includes and extends the functionality of CSS Conditional 4 [\[css-conditional-4\]](#biblio-css-conditional-4), adding the generalized conditional rule <a id="ref-for-at-ruledef-when"></a>[&#64;when](#at-ruledef-when) and the chained conditional rule <a id="ref-for-at-ruledef-else"></a>[&#64;else](#at-ruledef-else), as well as introducing font processing queries to the <a id="ref-for-supports-queries"></a>[supports query](https://www.w3.org/TR/css-conditional-3/#supports-queries) syntax used in <a id="ref-for-at-ruledef-supports"></a>[&#64;supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rules, and container queries.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication. A list of current W3C publications and the latest revision of this technical report can be found in the [W3C standards and drafts index.](https://www.w3.org/TR/)</em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-conditional” in the title, like this: “\[css-conditional\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-conditional%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](#introduction)
2.  [2 Extensions to the &#64;supports rule](#at-supports-ext)
    1.  [2.1 Extensions to the definition of support](#support-definition-ext)
        1.  [2.1.1 Font techs and formats](#support-definition-ext-fonts)
        2.  [2.1.2 At-rules](#support-definition-at-rules)
        3.  [2.1.3 Named features](#support-definition-named-features)
        4.  [2.1.4 Named conditions](#support-definition-supports-condition-name)
        5.  [2.1.5 Environment variables](#support-definition-env)
3.  [3 Generalized Conditional Rules: the &#64;when rule](#when-rule)
4.  [4 Chained Conditionals: the &#64;else rule](#else-rule)
5.  [5 Container Queries](#container-queries)
    1.  [5.1 Creating Query Containers: the container-type property](#container-type)
    2.  [5.2 Naming Query Containers: the container-name property](#container-name)
    3.  [5.3 Creating Named Containers: the container shorthand](#container-shorthand)
    4.  [5.4 Container Queries: the &#64;container rule](#container-rule)
    5.  [5.5 Animated Containers](#animated-containers)
6.  [6 Container Features](#container-features)
    1.  [6.1 Size Container Features](#size-container)
        1.  [6.1.1 Width: the width feature](#width)
        2.  [6.1.2 Height: the height feature](#height)
        3.  [6.1.3 Inline-size: the inline-size feature](#inline-size)
        4.  [6.1.4 Block-size: the block-size feature](#block-size)
        5.  [6.1.5 Aspect-ratio: the aspect-ratio feature](#aspect-ratio)
        6.  [6.1.6 Orientation: the orientation feature](#orientation)
    2.  [6.2 Style Container Features](#style-container)
    3.  [6.3 Scroll State Container Features](#scroll-state-container)
        1.  [6.3.1 Updating Scroll State](#updating-scroll-state)
        2.  [6.3.2 Sticky positioning: the stuck feature](#stuck)
        3.  [6.3.3 Scroll snapping: the snapped feature](#snapped)
        4.  [6.3.4 Scrollable: the scrollable feature](#scrollable)
        5.  [6.3.5 Scrolled: the scrolled feature](#scrolled)
7.  [7 Container Relative Lengths: the cqw, cqh, cqi, cqb, cqmin, cqmax units](#container-lengths)
8.  [8 Defining Custom Support Queries: the &#64;supports-condition rule](#supports-condition-rule)
9.  [9 APIs](#apis)
    1.  [9.1 The <code>CSSContainerRule</code> interface](#the-csscontainerrule-interface)
    2.  [9.2 The <code>CSSSupportsConditionRule</code> interface](#the-csssupportscondition-interface)
10. [ Security Considerations](#security)
11. [ Privacy Considerations](#privacy)
12. [ Changes](#changes)
    1.  [ Changes since the Working Draft of 5 November 2024 ](#changes-20241105)
    2.  [ Changes since the Working Draft of 23 July 2024 ](#changes-20240723)
    3.  [ Changes since the First Public Working Draft of 21 December 2021 ](#changes-20211221)
    4.  [ Additions since Level 4](#changes-from-L4)
13. [ Acknowledgments](#acknowledgments)
14. [ Conformance](#w3c-conformance)
    1.  [ Document conventions](#w3c-conventions)
    2.  [ Conformance classes](#w3c-conformance-classes)
    3.  [ Partial implementations](#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](#w3c-testing)
15. [ Index](#index)
    1.  [ Terms defined by this specification](#index-defined-here)
    2.  [ Terms defined by reference](#index-defined-elsewhere)
16. [ References](#references)
    1.  [ Normative References](#normative)
    2.  [ Non-Normative References](#informative)
17. [ Property Index](#property-index)
    1.  [ &#64;container Descriptors](#container-descriptor-table)
18. [ IDL Index](#idl-index)
19. [ Issues Index](#issues-index)

## <a id="introduction"></a>1.  Introduction[](#introduction)

<a id="issue-092ad31b"></a>

<strong>Issue:</strong>

[](#issue-092ad31b) This is currently an early draft of the things that are <em>new</em> in level 5. The features in Level 3 and Level 4 are still defined in [\[css-conditional-3\]](#biblio-css-conditional-3) and [\[css-conditional-4\]](#biblio-css-conditional-4) and have not yet been copied here.

CSS Conditional Level 5 extends the <a id="ref-for-at-ruledef-supports①"></a>[&#64;supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rule and <a id="ref-for-supports-queries①"></a>[supports query](https://www.w3.org/TR/css-conditional-3/#supports-queries) syntax to allow testing for custom support conditions as well as supported <a id="ref-for-at-rule"></a>[at-rules](https://www.w3.org/TR/css-syntax-3/#at-rule) and font technologies.

It also adds an <a id="ref-for-at-ruledef-when①"></a>[&#64;when](#at-ruledef-when) rule, which generalizes the concept of a conditional rule. Anything that can be expressed in an existing conditional rule can be expressed in <a id="ref-for-at-ruledef-when②"></a>&#64;when by wrapping it in an appropriate function to declare what kind of condition it is. This allow authors to easily combine multiple types of queries, such as media queries and supports queries, in a single boolean expression. Without this, authors must rely on nesting separate conditional rules, which is harder to read and write, presupposes the conditions are to be conjoined with the “and” boolean relation (with no easy way to indicate anything else), and restricts their utility in the proposed <a id="ref-for-conditional-rule-chain"></a>[conditional rule chains](#conditional-rule-chain).

It also adds <a id="ref-for-at-ruledef-else①"></a>[&#64;else](#at-ruledef-else) rules, which immediately follow other conditional rules and automatically qualify their conditions as the inverse of the immediately preceding rule’s conditions, such that only the first matching rule in a <a id="ref-for-conditional-rule-chain①"></a>[conditional rule chain](#conditional-rule-chain) is applied.

It also adds Container Queries. They are conceptually similar to Media Queries, but allow testing aspects of elements within the document (such as box dimensions or computed styles), rather than on the document as a whole.

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

The ability to have <a id="ref-for-single-axis-scroll-container"></a>[single-axis scroll containers](https://drafts.csswg.org/css-overflow-3/#single-axis-scroll-container), where one axis is scrollable and the other is <a id="ref-for-valdef-overflow-clip"></a>[clip](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-clip). [\[CSS-OVERFLOW-4\]](#biblio-css-overflow-4)

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

## <a id="container-queries"></a>5.  Container Queries[](#container-queries)

While <a id="ref-for-media-query"></a>[media queries](https://www.w3.org/TR/mediaqueries-5/#media-query) provide a method to query aspects of the user agent or device environment that a document is being displayed in (such as viewport dimensions or user preferences), <a id="ref-for-container-query"></a>[container queries](#container-query) allow testing aspects of elements within the document (such as box dimensions or computed styles).

By default, all elements are <a id="query-container"></a><strong>query containers</strong> for the purpose of <a id="ref-for-container-style-query"></a>[container style queries](#container-style-query), and can be established as <a id="ref-for-query-container"></a>[query containers](#query-container) for <a id="ref-for-container-size-query"></a>[container size queries](#container-size-query) and <a id="ref-for-container-scroll-state-query"></a>[container scroll-state queries](#container-scroll-state-query) by specifying the additional query types using the <a id="ref-for-propdef-container-type"></a>[container-type](#propdef-container-type) property (or the <a id="ref-for-propdef-container"></a>[container](#propdef-container) <a id="ref-for-shorthand-property"></a>[shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property)). Style rules applying to a <a id="ref-for-query-container①"></a>query container’s <a id="ref-for-flat-tree"></a>[flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) descendants can be conditioned by querying against it, using the <a id="ref-for-at-ruledef-container"></a>[&#64;container](#at-ruledef-container) <a id="ref-for-conditional-group-rule①⓪"></a>[conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule).

An ancestor element that generates a box is not an eligible container for <a id="ref-for-container-size-query①"></a>[container size queries](#container-size-query) if that box is not an ancestor box of any boxes generated by the querying element.

There are cases where the box for a

<strong>Note:</strong>

<a id="ref-for-pseudo-element"></a>[pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) is generated as a sibling box of its originating element’s box. If we allowed querying the size of a sibling box, it would introduce layout cycles.

For example, the <a id="ref-for-selectordef-scroll-marker-group"></a>[::scroll-marker-group](https://www.w3.org/TR/css-overflow-5/#selectordef-scroll-marker-group) and ::scroll-button() pseudo-elements generate sibling boxes of their originating element’s box. These pseudo-elements will not be able to query their originating scroller for <a id="ref-for-container-size-query②"></a>[container size queries](#container-size-query). They will, however, be able to query other eligible <a id="ref-for-query-container②"></a>[query container](#query-container) ancestors.

<a id="example-93e9e452"></a>

<strong>Example:</strong>

[](#example-93e9e452) For example, we can define the main content area and sidebar as containers, and then describe a .media-object that changes from vertical to horizontal layout depending on the size of its container:

``` text
main, aside {
  container: my-layout / inline-size;
}

.media-object {
  display: grid;
  grid-template: 'img' auto 'content' auto / 100%;
}

@container my-layout (inline-size > 45em) {
  .media-object {
  grid-template: 'img content' auto / auto 1fr;
  }
}
```

Media objects in the main and sidebar areas will each respond to their own container context.

For the <a id="ref-for-selectordef-part"></a>[::part()](https://drafts.csswg.org/css-shadow-1/#selectordef-part) and <a id="ref-for-selectordef-slotted"></a>[::slotted()](https://drafts.csswg.org/css-shadow-1/#selectordef-slotted) <a id="ref-for-pseudo-element①"></a>[pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) selectors, which represent real elements in the DOM tree, <a id="ref-for-query-container③"></a>[query containers](#query-container) can be established by <a id="ref-for-flat-tree①"></a>[flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) ancestors of those elements. For other <a id="ref-for-pseudo-element②"></a>pseudo-elements, <a id="ref-for-query-container④"></a>query containers can be established by inclusive <a id="ref-for-flat-tree②"></a>flat tree ancestors of their <a id="ref-for-originating-element"></a>[originating element](https://www.w3.org/TR/selectors-4/#originating-element).

It follows that:

<strong>Note:</strong>

- <a id="ref-for-selectordef-before"></a>[::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before), <a id="ref-for-selectordef-after"></a>[::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after), <a id="ref-for-selectordef-marker"></a>[::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker), and <a id="ref-for-selectordef-backdrop"></a>[::backdrop](https://www.w3.org/TR/css-position-4/#selectordef-backdrop) query their originating elements

- <a id="ref-for-selectordef-first-letter"></a>[::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) and <a id="ref-for-selectordef-first-line"></a>[::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) query their originating elements, even if the <a id="ref-for-fictional-tag-sequence"></a>[fictional tag sequence](https://www.w3.org/TR/css-pseudo-4/#fictional-tag-sequence) may push the <code>&#58;&#58;first-line</code> past other elements for the purpose of inheritance and rendering

- <a id="ref-for-selectordef-slotted①"></a>[::slotted()](https://drafts.csswg.org/css-shadow-1/#selectordef-slotted) selectors can query containers inside the shadow tree, including the slot itself

- ::slotted()::before selectors can query the slotted shadow host child

- <a id="ref-for-selectordef-part①"></a>[::part()](https://drafts.csswg.org/css-shadow-1/#selectordef-part) selectors can query containers inside the shadow tree

- <a id="ref-for-selectordef-placeholder"></a>[::placeholder](https://www.w3.org/TR/css-pseudo-4/#selectordef-placeholder) and <a id="ref-for-selectordef-file-selector-button"></a>[::file-selector-button](https://www.w3.org/TR/css-pseudo-4/#selectordef-file-selector-button) can query the input element, but do not expose any internal containers if the input element is implemented using a shadow tree

<a id="example-13f6f0fa"></a>

<strong>Example:</strong>

[](#example-13f6f0fa) A ::before selector querying the size of the originating element:

``` text
<style>
  #container {
  width: 100px;
  container-type: inline-size;
  }
  @container (inline-size < 150px) {
  #inner::before {
    content: "BEFORE";
  }
  }
</style>
<div id=container>
  <span id=inner></span>
</div>
```

<a id="example-c6405bc1"></a>

<strong>Example:</strong>

[](#example-c6405bc1) A ::slotted() selector for styling a shadow host child can query a container in the shadow tree:

``` text
<div id=host style="width:200px">
  <template shadowroot=open>
  <style>
    #container {
    width: 100px;
    container-type: inline-size;
    }
    @container (inline-size < 150px) {
    ::slotted(span) {
      color: green;
    }
    }
  </style>
  <div id=container>
    <slot />
  </div>
  </template>
  <span id=slotted>Green</span>
</div>
```

### <a id="container-type"></a>5.1.  Creating Query Containers: the <a id="ref-for-propdef-container-type①"></a>[container-type](#propdef-container-type) property[](#container-type)

| Field                                                                                    | Definition                                                                                                                                                                                                                                            |
|------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-container-type"></a><strong>container-type</strong>                                                                                                                                                                                    |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | normal <a id="ref-for-comb-one⑨"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ \[ size <a id="ref-for-comb-one①⓪"></a>\| inline-size \] <a id="ref-for-comb-any"></a>[\|\|](https://www.w3.org/TR/css-values-4/#comb-any) scroll-state \] |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | normal                                                                                                                                                                                                                                                |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                   |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                    |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                   |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified keyword                                                                                                                                                                                                                                     |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                           |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | not animatable                                                                                                                                                                                                                                        |

The <a id="ref-for-propdef-container-type②"></a>[container-type](#propdef-container-type) property establishes the element as a <a id="ref-for-query-container⑤"></a>[query container](#query-container) for certain types of queries. For size <a id="ref-for-container-query①"></a>[container queries](#container-query), which require certain types of containment, elements are explicitly made <a id="ref-for-query-container⑥"></a>query containers through this property. For other types of <a id="ref-for-query-container⑦"></a>query containers any element can be a <a id="ref-for-query-container⑧"></a>query container, such as for <a id="ref-for-container-style-query①"></a>[container style queries](#container-style-query).

Values have the following meanings:

<strong><a id="valdef-container-type-size"></a><strong>size</strong></strong>

Establishes a <a id="ref-for-query-container⑨"></a>[query container](#query-container) for <a id="ref-for-container-size-query③"></a>[container size queries](#container-size-query) on both the <a id="ref-for-inline-axis"></a>[inline](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) and <a id="ref-for-block-axis"></a>[block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). Applies <a id="ref-for-style-containment"></a>[style containment](https://www.w3.org/TR/css-contain-2/#style-containment) and <a id="ref-for-size-containment"></a>[size containment](https://www.w3.org/TR/css-contain-2/#size-containment) to the <a id="ref-for-principal-box"></a>[principal box](https://www.w3.org/TR/css-display-4/#principal-box), and establishes an <a id="ref-for-independent-formatting-context"></a>[independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context).

<strong><a id="valdef-container-type-inline-size"></a><strong>inline-size</strong></strong>

Establishes a <a id="ref-for-query-container①⓪"></a>[query container](#query-container) for <a id="ref-for-container-size-query④"></a>[container size queries](#container-size-query) on the container’s own <a id="ref-for-inline-axis①"></a>[inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). Applies <a id="ref-for-style-containment①"></a>[style containment](https://www.w3.org/TR/css-contain-2/#style-containment) and <a id="ref-for-inline-size-containment"></a>[inline-size containment](https://www.w3.org/TR/css-contain-3/#inline-size-containment) to the <a id="ref-for-principal-box①"></a>[principal box](https://www.w3.org/TR/css-display-4/#principal-box), and establishes an <a id="ref-for-independent-formatting-context①"></a>[independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context).

<strong><a id="valdef-container-type-scroll-state"></a><strong>scroll-state</strong></strong>

Establishes a <a id="ref-for-query-container①①"></a>[query container](#query-container) for <a id="ref-for-container-scroll-state-query①"></a>[container scroll-state queries](#container-scroll-state-query). For queries that match the state of the scroller itself (such as scrollable), the element is only a <a id="ref-for-query-container①②"></a>query container on the element’s own <a id="ref-for-scrollable-axis"></a>[scrollable axises](https://drafts.csswg.org/css-overflow-3/#scrollable-axis).

Note: This means that a scrollable query, for example, could query against different containers for scroll-state(scrollable: block) and scroll-state(scrollable: inline) queries.

<strong><a id="valdef-container-type-normal"></a><strong>normal</strong></strong>

The element is not a <a id="ref-for-query-container①③"></a>[query container](#query-container) for any <a id="ref-for-container-size-query⑤"></a>[container size queries](#container-size-query) or <a id="ref-for-container-scroll-state-query②"></a>[container scroll-state queries](#container-scroll-state-query), but remains a <a id="ref-for-query-container①④"></a>query container for <a id="ref-for-container-style-query②"></a>[container style queries](#container-style-query).

<a id="example-5708b886"></a>

<strong>Example:</strong>

[](#example-5708b886) For example, authors can create container-responsive typography, adjusting <a id="ref-for-propdef-font-size"></a>[font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size), <a id="ref-for-propdef-line-height"></a>[line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height), and other typographic concerns based on the size of a container:

``` text
aside, main {
  container-type: inline-size;
}

h2 { font-size: 1.2em; }

@container (width > 40em) {
  h2 { font-size: 1.5em; }
}
```

The 40em value used in the query condition is relative to the <a id="ref-for-computed-value"></a>[computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of <a id="ref-for-propdef-font-size①"></a>[font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) on the relevant <a id="ref-for-query-container①⑤"></a>[query container](#query-container).

<a id="example-76326f8c"></a>

<strong>Example:</strong>

[](#example-76326f8c) Containers can also expose computed style values for querying. This can be useful for toggling behavior across multiple properties:

``` text
@container style(--cards: small) {
  article {
  border: thin solid silver;
  border-radius: 0.5em;
  padding: 1em;
  }
}
```

<a id="example-9f75dbe5"></a>

<strong>Example:</strong>

[](#example-9f75dbe5) Containers can also expose state that depends on scroll offset. This example styles a descendant of a sticky positioned element when it is stuck to the top edge:

``` text
#sticky {
  container-type: scroll-state;
  position: sticky;
}
@container scroll-state(stuck: top) {
  #sticky-child {
  background-color: lime;
  }
}
```

### <a id="container-name"></a>5.2.  Naming Query Containers: the <a id="ref-for-propdef-container-name"></a>[container-name](#propdef-container-name) property[](#container-name)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                   |
|------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-container-name"></a><strong>container-name</strong>                                                                                                                                                                                                                           |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | none <a id="ref-for-comb-one①①"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) <a id="ref-for-identifier-value"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)<a id="ref-for-mult-one-plus"></a>[+](https://www.w3.org/TR/css-values-4/#mult-one-plus) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | none                                                                                                                                                                                                                                                                                         |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                          |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                                                           |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                                                          |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | the keyword <a id="ref-for-valdef-container-name-none"></a>[none](#valdef-container-name-none), or an ordered list of <a id="ref-for-css-css-identifier"></a>[identifiers](https://www.w3.org/TR/css-values-4/#css-css-identifier)                                                           |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                  |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | not animatable                                                                                                                                                                                                                                                                               |

The <a id="ref-for-propdef-container-name①"></a>[container-name](#propdef-container-name) property specifies a list of <a id="query-container-name"></a><strong>query container names</strong>. These names can be used by <a id="ref-for-at-ruledef-container①"></a>[&#64;container](#at-ruledef-container) rules to filter which <a id="ref-for-query-container①⑥"></a>[query containers](#query-container) are targeted. Container names are not <a id="ref-for-css-tree-scoped-name"></a>[tree-scoped names](https://drafts.csswg.org/css-shadow-1/#css-tree-scoped-name).

<strong><a id="valdef-container-name-none"></a><strong>none</strong></strong>

The <a id="ref-for-query-container①⑦"></a>[query container](#query-container) has no <a id="ref-for-query-container-name"></a>[query container name](#query-container-name).

<strong><a id="valdef-container-name-custom-ident"></a><strong><a id="ref-for-identifier-value①"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)</strong></strong>

Specifies a <a id="ref-for-query-container-name①"></a>[query container name](#query-container-name) as an <a id="ref-for-css-css-identifier①"></a>[identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier). The keywords <a id="ref-for-valdef-container-name-none①"></a>[none](#valdef-container-name-none), and, <a id="ref-for-valdef-media-not"></a>[not](https://www.w3.org/TR/mediaqueries-5/#valdef-media-not), and or are excluded from this <a id="ref-for-identifier-value②"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value).

<a id="example-63042722"></a>

<strong>Example:</strong>

[](#example-63042722) In some cases, we want to query aspects of a specific container, even if it’s not the nearest ancestor container. For example, we might want to query the height of a main content area, and the width of a more nested inline-container.

``` text
main {
  container-type: size;
  container-name: my-page-layout;
}

.my-component {
  container-type: inline-size;
  container-name: my-component-library;
}

@container my-page-layout (block-size > 12em) {
  .card { margin-block: 2em; }
}

@container my-component-library (inline-size > 30em) {
  .card { margin-inline: 2em; }
}
```

It is also possible to query for a container only based on its name.

``` text
@container my-page-layout {
  .card { padding: 1em; }
}
```

### <a id="container-shorthand"></a>5.3.  Creating Named Containers: the <a id="ref-for-propdef-container①"></a>[container](#propdef-container) shorthand[](#container-shorthand)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                     |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-container"></a><strong>container</strong>                                                                                                                                                                                                                       |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | <a id="ref-for-propdef-container-name②"></a>[\<'container-name'\>](#propdef-container-name) \[ / <a id="ref-for-propdef-container-type③"></a>[\<'container-type'\>](#propdef-container-type) \]<a id="ref-for-mult-opt①"></a>[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | see individual properties                                                                                                                                                                                                                                                      |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | see individual properties                                                                                                                                                                                                                                                      |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | see individual properties                                                                                                                                                                                                                                                      |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | see individual properties                                                                                                                                                                                                                                                      |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | see individual properties                                                                                                                                                                                                                                                      |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | see individual properties                                                                                                                                                                                                                                                      |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                    |

The <a id="ref-for-propdef-container②"></a>[container](#propdef-container) <a id="ref-for-shorthand-property①"></a>[shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets both <a id="ref-for-propdef-container-type④"></a>[container-type](#propdef-container-type) and <a id="ref-for-propdef-container-name③"></a>[container-name](#propdef-container-name) in the same declaration. If <a id="ref-for-propdef-container-type⑤"></a>\<'container-type'\> is omitted, it is reset to its <a id="ref-for-initial-value"></a>[initial value](https://www.w3.org/TR/css-cascade-5/#initial-value).

<a id="example-c4e7fa3f"></a>

<strong>Example:</strong>

[](#example-c4e7fa3f) We can define both a <a id="ref-for-propdef-container-type⑥"></a>[container-type](#propdef-container-type) and <a id="ref-for-propdef-container-name④"></a>[container-name](#propdef-container-name) using the shorthand syntax:

``` text
main {
  container: my-layout / size;
}

.grid-item {
  container: my-component / inline-size;
}
```

### <a id="container-rule"></a>5.4.  Container Queries: the <a id="ref-for-at-ruledef-container②"></a>[&#64;container](#at-ruledef-container) rule[](#container-rule)

The <a id="at-ruledef-container"></a><strong>&#64;container</strong> rule is a <a id="ref-for-conditional-group-rule①①"></a>[conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) whose condition contains a <a id="container-query"></a><strong>container query</strong>, which is a boolean combination of <a id="ref-for-container-size-query⑥"></a>[container size queries](#container-size-query) and/or <a id="ref-for-container-style-query③"></a>[container style queries](#container-style-query). Style declarations within the <a id="ref-for-at-ruledef-container③"></a>[&#64;container](#at-ruledef-container) rule are [filtered](https://www.w3.org/TR/css-cascade-4/#filtering) by its condition to only match when the <a id="ref-for-container-query②"></a>[container query](#container-query) is true for their element’s <a id="ref-for-query-container①⑧"></a>[query container](#query-container).

The syntax of the <a id="ref-for-at-ruledef-container④"></a>[&#64;container](#at-ruledef-container) rule is:

<a id="ref-for-typedef-container-condition"></a><a id="ref-for-mult-comma"></a><a id="ref-for-typedef-rule-list②"></a>

``` text
@container <container-condition># {
  <rule-list>
}
```

where:

<a id="typedef-container-condition"></a><a id="ref-for-typedef-container-condition①"></a><a id="ref-for-typedef-container-name"></a><a id="ref-for-mult-opt②"></a><a id="ref-for-typedef-container-query"></a><a id="ref-for-mult-opt③"></a><a id="ref-for-mult-req"></a><a id="typedef-container-name"></a><a id="ref-for-typedef-container-name①"></a><a id="ref-for-identifier-value③"></a><a id="typedef-container-query"></a><a id="ref-for-typedef-container-query①"></a><a id="ref-for-typedef-query-in-parens"></a><a id="ref-for-comb-one①②"></a><a id="ref-for-typedef-query-in-parens①"></a><a id="ref-for-typedef-query-in-parens②"></a><a id="ref-for-mult-zero-plus"></a><a id="ref-for-comb-one①③"></a><a id="ref-for-typedef-query-in-parens③"></a><a id="ref-for-mult-zero-plus①"></a><a id="typedef-query-in-parens"></a><a id="ref-for-typedef-query-in-parens④"></a><a id="ref-for-typedef-container-query②"></a><a id="ref-for-comb-one①④"></a><a id="ref-for-typedef-size-feature"></a><a id="ref-for-comb-one①⑤"></a><a id="ref-for-typedef-style-query"></a><a id="ref-for-comb-one①⑥"></a><a id="ref-for-typedef-scroll-state-query"></a><a id="ref-for-comb-one①⑦"></a><a id="ref-for-typedef-general-enclosed"></a><a id="typedef-style-query"></a><a id="ref-for-typedef-style-query①"></a><a id="ref-for-typedef-style-in-parens"></a><a id="ref-for-comb-one①⑧"></a><a id="ref-for-typedef-style-in-parens①"></a><a id="ref-for-typedef-style-in-parens②"></a><a id="ref-for-mult-zero-plus②"></a><a id="ref-for-comb-one①⑨"></a><a id="ref-for-typedef-style-in-parens③"></a><a id="ref-for-mult-zero-plus③"></a><a id="ref-for-comb-one②⓪"></a><a id="ref-for-typedef-style-feature"></a><a id="typedef-style-in-parens"></a><a id="ref-for-typedef-style-in-parens④"></a><a id="ref-for-typedef-style-query②"></a><a id="ref-for-comb-one②①"></a><a id="ref-for-typedef-style-feature①"></a><a id="ref-for-comb-one②②"></a><a id="ref-for-typedef-general-enclosed①"></a><a id="typedef-style-feature"></a><a id="ref-for-typedef-style-feature②"></a><a id="ref-for-typedef-style-feature-plain"></a><a id="ref-for-comb-one②③"></a><a id="ref-for-typedef-style-feature-boolean"></a><a id="ref-for-comb-one②④"></a><a id="ref-for-typedef-style-range"></a><a id="typedef-style-feature-plain"></a><a id="ref-for-typedef-style-feature-plain①"></a><a id="ref-for-typedef-style-feature-name"></a><a id="ref-for-typedef-style-feature-value"></a><a id="typedef-style-feature-boolean"></a><a id="ref-for-typedef-style-feature-boolean①"></a><a id="ref-for-typedef-style-feature-name①"></a><a id="typedef-style-range"></a><a id="ref-for-typedef-style-range①"></a><a id="ref-for-typedef-style-range-value"></a><a id="ref-for-typedef-mf-comparison"></a><a id="ref-for-typedef-style-range-value①"></a><a id="ref-for-comb-one②⑤"></a><a id="ref-for-typedef-style-range-value②"></a><a id="ref-for-typedef-mf-lt"></a><a id="ref-for-typedef-style-range-value③"></a><a id="ref-for-typedef-mf-lt①"></a><a id="ref-for-typedef-style-range-value④"></a><a id="ref-for-comb-one②⑥"></a><a id="ref-for-typedef-style-range-value⑤"></a><a id="ref-for-typedef-mf-gt"></a><a id="ref-for-typedef-style-range-value⑥"></a><a id="ref-for-typedef-mf-gt①"></a><a id="ref-for-typedef-style-range-value⑦"></a><a id="typedef-style-range-value"></a><a id="ref-for-typedef-style-range-value⑧"></a><a id="ref-for-typedef-custom-property-name"></a><a id="ref-for-comb-one②⑦"></a><a id="ref-for-typedef-style-feature-value①"></a><a id="typedef-scroll-state-query"></a><a id="ref-for-typedef-scroll-state-query①"></a><a id="ref-for-typedef-scroll-state-in-parens"></a><a id="ref-for-comb-one②⑧"></a><a id="ref-for-typedef-scroll-state-in-parens①"></a><a id="ref-for-typedef-scroll-state-in-parens②"></a><a id="ref-for-mult-zero-plus④"></a><a id="ref-for-comb-one②⑨"></a><a id="ref-for-typedef-scroll-state-in-parens③"></a><a id="ref-for-mult-zero-plus⑤"></a><a id="ref-for-comb-one③⓪"></a><a id="ref-for-typedef-scroll-state-feature"></a><a id="typedef-scroll-state-in-parens"></a><a id="ref-for-typedef-scroll-state-in-parens④"></a><a id="ref-for-typedef-scroll-state-query②"></a><a id="ref-for-comb-one③①"></a><a id="ref-for-typedef-scroll-state-feature①"></a><a id="ref-for-comb-one③②"></a><a id="ref-for-typedef-general-enclosed②"></a>

``` text
<container-condition> = [ <container-name>? <container-query>? ]!
<container-name> = <custom-ident>
<container-query> = not <query-in-parens>
          | <query-in-parens> [ [ and <query-in-parens> ]* | [ or <query-in-parens> ]* ]
<query-in-parens> = ( <container-query> )
          | ( <size-feature> )
          | style( <style-query> )
          | scroll-state( <scroll-state-query> )
          | <general-enclosed>

<style-query>     = not <style-in-parens>
          | <style-in-parens> [ [ and <style-in-parens> ]* | [ or <style-in-parens> ]* ]
          | <style-feature>
<style-in-parens> = ( <style-query> )
          | ( <style-feature> )
          | <general-enclosed>
<style-feature> = <style-feature-plain> | <style-feature-boolean> | <style-range>
<style-feature-plain> = <style-feature-name> : <style-feature-value>
<style-feature-boolean> = <style-feature-name>
<style-range> = <style-range-value> <mf-comparison> <style-range-value>
    | <style-range-value> <mf-lt> <style-range-value> <mf-lt> <style-range-value>
    | <style-range-value> <mf-gt> <style-range-value> <mf-gt> <style-range-value>
<style-range-value> = <custom-property-name> | <style-feature-value>

<scroll-state-query>     = not <scroll-state-in-parens>
             | <scroll-state-in-parens> [ [ and <scroll-state-in-parens> ]* | [ or <scroll-state-in-parens> ]* ]
             | <scroll-state-feature>
<scroll-state-in-parens> = ( <scroll-state-query> )
             | ( <scroll-state-feature> )
             | <general-enclosed>
```

The keywords <a id="ref-for-valdef-container-name-none②"></a>[none](#valdef-container-name-none), and, <a id="ref-for-valdef-media-not①"></a>[not](https://www.w3.org/TR/mediaqueries-5/#valdef-media-not), and or are excluded from the <a id="ref-for-identifier-value④"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) above.

For each element, the <a id="ref-for-query-container①⑨"></a>[query container](#query-container) to be queried is selected from among the element’s ancestor <a id="ref-for-query-container②⓪"></a>query containers that are established as a valid <a id="ref-for-query-container②①"></a>query container for all the <a id="ref-for-container-feature"></a>[container features](#container-feature) in the <a id="ref-for-typedef-container-query③"></a>[\<container-query\>](#typedef-container-query). If the <a id="ref-for-typedef-container-query④"></a>\<container-query\> contains unknown or unsupported <a id="ref-for-container-feature①"></a>container features, no <a id="ref-for-query-container②②"></a>query container will be selected for that <a id="ref-for-typedef-container-condition②"></a>[\<container-condition\>](#typedef-container-condition). The <a id="ref-for-typedef-container-name②"></a>[\<container-name\>](#typedef-container-name) filters the set of <a id="ref-for-query-container②③"></a>query containers considered to just those with a matching <a id="ref-for-query-container-name②"></a>[query container name](#query-container-name).

Once an eligible <a id="ref-for-query-container②④"></a>[query container](#query-container) has been selected for an element, each <a id="ref-for-container-feature②"></a>[container feature](#container-feature) in the <a id="ref-for-typedef-container-query⑤"></a>[\<container-query\>](#typedef-container-query) is evaluated against that <a id="ref-for-query-container②⑤"></a>query container. If no ancestor is an eligible <a id="ref-for-query-container②⑥"></a>query container, then the <a id="ref-for-container-query③"></a>[container query](#container-query) is unknown for that element. As with media queries, <a id="ref-for-typedef-general-enclosed③"></a>[\<general-enclosed\>](https://www.w3.org/TR/mediaqueries-5/#typedef-general-enclosed) evaluates to unknown. If the <a id="ref-for-typedef-container-query⑥"></a>\<container-query\> is omitted, the <a id="ref-for-query-container②⑦"></a>query container is eligible as long as the <a id="ref-for-typedef-container-name③"></a>[\<container-name\>](#typedef-container-name) matches.

If a <a id="ref-for-container-query④"></a>[container query](#container-query) includes multiple <a id="ref-for-typedef-container-condition③"></a>[\<container-condition\>](#typedef-container-condition)s, each condition will select it’s own <a id="ref-for-query-container②⑧"></a>[query container](#query-container), and evaluate independently. A <a id="ref-for-container-query⑤"></a>container query is <a id="ref-for-valdef-custom-media-true"></a>[true](https://www.w3.org/TR/mediaqueries-5/#valdef-custom-media-true) if <em>any</em> of its component <a id="ref-for-typedef-container-condition④"></a>\<container-condition\>s are <a id="ref-for-valdef-custom-media-true①"></a>true, and <a id="ref-for-valdef-custom-media-false"></a>[false](https://www.w3.org/TR/mediaqueries-5/#valdef-custom-media-false) only if <em>all</em> of its component <a id="ref-for-typedef-container-condition⑤"></a>\<container-condition\>s are <a id="ref-for-valdef-custom-media-false①"></a>false.

<a id="example-cba8e10c"></a>

<strong>Example:</strong>

[](#example-cba8e10c) As with <a id="ref-for-media-query①"></a>[media queries](https://www.w3.org/TR/mediaqueries-5/#media-query), we can string together multiple queries in a single condition:

``` text
@container card (inline-size > 30em) and style(--responsive: true) {
  /* styles */
}
```

The styles above will only be applied if there is an ancestor container named "card" that meets both the <a id="ref-for-descdef-container-inline-size"></a>[inline-size](#descdef-container-inline-size) and <a id="ref-for-container-style-query④"></a>[style](#container-style-query) conditions.

We can also combine multiple conditions into a list, with each condition evaluating against a different container:

``` text
@container card (inline-size > 30em), style(--large: true) {
  /* styles */
}
```

The styles above will be applied if there is an ancestor container named "card" that meets the <a id="ref-for-descdef-container-inline-size①"></a>[inline-size](#descdef-container-inline-size) condition <em>or</em> the nearest style container meets the <a id="ref-for-container-style-query⑤"></a>[style](#container-style-query) condition.

Style rules defined on an element inside multiple nested <a id="ref-for-container-query⑥"></a>[container queries](#container-query) apply when all of the wrapping <a id="ref-for-container-query⑦"></a>container queries are true for that element.

Note: Nested <a id="ref-for-container-query⑧"></a>[container queries](#container-query) can evaluate in relation to different containers, so it is not always possible to merge the individual <a id="ref-for-typedef-container-condition⑥"></a>[\<container-condition\>](#typedef-container-condition)s into a single query.

<a id="example-b320e2d0"></a>

<strong>Example:</strong>

[](#example-b320e2d0) Using a single comma-separated <a id="ref-for-container-query⑨"></a>[container query](#container-query), we can query multiple containers:

``` text
@container card (inline-size > 30em), style(--responsive: true) {
  /* styles */
}
```

The styles above will apply for an element inside <em>either</em> a container named "card" that meets the <a id="ref-for-descdef-container-inline-size②"></a>[inline-size](#descdef-container-inline-size) condition, <em>or</em> a container meeting the <a id="ref-for-container-style-query⑥"></a>[style](#container-style-query) condition.

In order to require that <em>all</em> conditions are met while querying multiple containers, we would need to nest multiple queries:

``` text
@container card (inline-size > 30em) {
  @container style(--responsive: true) {
  /* styles */
  }
}
```

The styles above will only be applied if there is <em>both</em> an ancestor container named "card" that meets the <a id="ref-for-descdef-container-inline-size③"></a>[inline-size](#descdef-container-inline-size) condition, <em>and</em> an ancestor container meeting the <a id="ref-for-container-style-query⑦"></a>[style](#container-style-query) condition.

Global, name-defining <a id="ref-for-at-rule③"></a>[at-rules](https://www.w3.org/TR/css-syntax-3/#at-rule) such as <a id="ref-for-at-ruledef-keyframes"></a>[&#64;keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes) or <a id="ref-for-at-font-face-rule"></a>[&#64;font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) or <a id="ref-for-at-ruledef-layer"></a>[&#64;layer](https://www.w3.org/TR/css-cascade-5/#at-ruledef-layer) that are defined inside <a id="ref-for-container-query①⓪"></a>[container queries](#container-query) are not constrained by the <a id="ref-for-container-query①①"></a>container query conditions.

### <a id="animated-containers"></a>5.5.  Animated Containers[](#animated-containers)

A change in the evaluation of a <a id="ref-for-container-query①②"></a>[container query](#container-query) must be part of a <a id="ref-for-style-change-event"></a>[style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event), even when the change occurred because of <a id="ref-for-effect-value"></a>[animation effects](https://www.w3.org/TR/web-animations-1/#effect-value).

<a id="example-cfbdbffb"></a>

<strong>Example:</strong>

[](#example-cfbdbffb) A transition on a sibling element can indirectly affect the size of a container, triggering <a id="ref-for-style-change-event①"></a>[style change events](https://www.w3.org/TR/css-transitions-1/#style-change-event) whenever container queries change their evaluation as a result:

``` text
main {
  display: flex;
  width: 300px;
}

#container {
  container-type: inline-size;
  flex: 1;
}

/* Resolved width is initially 200px, but changes as the transition
   on #sibling progresses. */
#inner {
  transition: 1s background-color;
  background-color: tomato;
}

/* When this container query starts (or stops) applying, a transition
   must start on background-color on #inner. */
@container (width <= 150px) {
  #inner {
  background-color: skyblue;
  }
}

#sibling {
  width: 100px;
  transition: width 1s;
}

#sibling:hover {
  width: 200px;
}
```

``` text
<main>
  <div id=container>
  <div id=inner>Inner</div>
  </div>
  <div id=sibling>Sibling</div>
</main>
```

Changes in <a id="ref-for-computed-value①"></a>[computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) caused by <a id="ref-for-container-query-length"></a>[container query length](#container-query-length) units must also be part of a <a id="ref-for-style-change-event②"></a>[style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event).

## <a id="container-features"></a>6.  Container Features[](#container-features)

A <a id="container-feature"></a><strong>container feature</strong> queries a specific aspect of a <a id="ref-for-query-container②⑨"></a>[query container](#query-container).

<a id="ref-for-container-feature③"></a>[Container features](#container-feature) use the same rules as <a id="ref-for-media-feature"></a>[media features](https://www.w3.org/TR/mediaqueries-5/#media-feature) when evaluating in a <a id="ref-for-boolean-context"></a>[boolean context](https://www.w3.org/TR/mediaqueries-5/#boolean-context).

### <a id="size-container"></a>6.1.  Size Container Features[](#size-container)

A <a id="container-size-query"></a><strong>container size query</strong> allows querying the size of the <a id="ref-for-query-container③⓪"></a>[query container](#query-container)’s <a id="ref-for-principal-box②"></a>[principal box](https://www.w3.org/TR/css-display-4/#principal-box). It is a boolean combination of individual <a id="size-features"></a><strong>size features</strong> (<a id="ref-for-typedef-size-feature①"></a>[\<size-feature\>](#typedef-size-feature)) that each query a single, specific dimensional feature of the <a id="ref-for-query-container③①"></a>query container. The syntax of a <a id="typedef-size-feature"></a><strong><a id="ref-for-typedef-size-feature②"></a>[\<size-feature\>](#typedef-size-feature)</strong> is the same as for a <a id="ref-for-media-feature①"></a>[media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature): a feature name, a comparator, and a value. [\[mediaqueries-5\]](#biblio-mediaqueries-5) The boolean syntax and logic combining <a id="ref-for-size-features"></a>[size features](#size-features) into a <a id="ref-for-container-size-query⑦"></a>[size query](#container-size-query) is the same as for <a id="ref-for-css-feature-queries"></a>[CSS feature queries](https://www.w3.org/TR/css-conditional-3/#css-feature-queries). (See <a id="ref-for-at-ruledef-supports⑤"></a>[&#64;supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports). [\[css-conditional-3\]](#biblio-css-conditional-3))

If the <a id="ref-for-query-container③②"></a>[query container](#query-container) does not have a <a id="ref-for-principal-box③"></a>[principal box](https://www.w3.org/TR/css-display-4/#principal-box), or the principal box is not a <a id="ref-for-layout-containment-box"></a>[layout containment box](https://www.w3.org/TR/css-contain-2/#layout-containment-box), or the <a id="ref-for-query-container③③"></a>query container does not support <a id="ref-for-container-size-query⑧"></a>[container size queries](#container-size-query) on the relevant axes, then the result of evaluating the <a id="ref-for-size-features①"></a>[size feature](#size-features) is unknown.

<a id="ref-for-relative-length"></a>[Relative length](https://www.w3.org/TR/css-values-4/#relative-length) units (including <a id="ref-for-container-query-length①"></a>[container query length](#container-query-length) units) and <a id="ref-for-custom-property"></a>[custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) in <a id="ref-for-container-query①③"></a>[container query](#container-query) conditions are evaluated based on the <a id="ref-for-computed-value②"></a>[computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of the <a id="ref-for-query-container③④"></a>[query container](#query-container).

Tree counting functions ([CSS Values 5 § 9 Tree Counting Functions: the sibling-count() and sibling-index() notations](https://www.w3.org/TR/css-values-5/#tree-counting)) are evaluated against the container element.

Note: This is different from the handling of relative units in <a id="ref-for-media-query②"></a>[media queries](https://www.w3.org/TR/mediaqueries-5/#media-query).

Note: If <a id="ref-for-custom-property①"></a>[custom property](https://www.w3.org/TR/css-variables-1/#custom-property) substitution results in an invalid value for the <a id="ref-for-size-features②"></a>[size feature](#size-features), it is handled the same as other invalid feature values, and the result of the <a id="ref-for-size-features③"></a>size feature is unknown.

<a id="example-031e4923"></a>

<strong>Example:</strong>

[](#example-031e4923) For example, <a id="ref-for-query-container③⑤"></a>[query containers](#query-container) with different font-sizes will evaluate <a id="ref-for-em"></a>[em](https://www.w3.org/TR/css-values-4/#em)-based queries relative to their own font sizes:

``` text
aside, main {
  container-type: inline-size;
}

aside { font-size: 16px; }
main { font-size: 24px; }

@container (width > 40em) {
  h2 { font-size: 1.5em; }
}
```

The 40em value used in the query condition is relative to the <a id="ref-for-computed-value③"></a>[computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of <a id="ref-for-propdef-font-size②"></a>[font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) on the relevant <a id="ref-for-query-container③⑥"></a>[query container](#query-container):

- For any h2 inside aside, the query condition will be true above 640px.

- For any h2 inside main, the query condition will be true above 960px.

<a id="example-5df63e7c"></a>

<strong>Example:</strong>

[](#example-5df63e7c) Similarly, <a id="ref-for-query-container③⑦"></a>[query containers](#query-container) will evaluate <a id="ref-for-funcdef-var"></a>[var()](https://www.w3.org/TR/css-variables-1/#funcdef-var)-based queries relative to their own <a id="ref-for-computed-value④"></a>[computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the <a id="ref-for-custom-property②"></a>[custom property](https://www.w3.org/TR/css-variables-1/#custom-property):

``` text
aside, main {
  container-type: inline-size;
}

aside { --query: 300px; }
main { --query: 500px; }

@container (width > var(--query)) {
  h2 { font-size: 1.5em; }
}
```

The var(--query) value used in the query condition is substituted with the <a id="ref-for-computed-value⑤"></a>[computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the --query <a id="ref-for-custom-property③"></a>[custom property](https://www.w3.org/TR/css-variables-1/#custom-property) on the relevant <a id="ref-for-query-container③⑧"></a>[query container](#query-container):

- For any h2 inside aside, the query condition will be true above 300px.

- For any h2 inside main, the query condition will be true above 500px.

#### <a id="width"></a>6.1.1.  Width: the <a id="ref-for-descdef-container-width"></a>[width](#descdef-container-width) feature[](#width)

| Field                   | Definition                                                                                      |
|-------------------------|-------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-width"></a><strong>width</strong>                                      |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container⑤"></a>[&#64;container](#at-ruledef-container)               |
| <strong>Value:</strong> | <a id="ref-for-length-value"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong>Type:</strong>  | range                                                                                           |

The <a id="ref-for-descdef-container-width①"></a>[width](#descdef-container-width) <a id="ref-for-container-feature④"></a>[container feature](#container-feature) queries the <a id="ref-for-width"></a>[width](https://www.w3.org/TR/css-sizing-3/#width) of the <a id="ref-for-query-container③⑨"></a>[query container](#query-container)’s <a id="ref-for-content-box"></a>[content box](https://www.w3.org/TR/css-box-4/#content-box).

#### <a id="height"></a>6.1.2.  Height: the <a id="ref-for-descdef-container-height"></a>[height](#descdef-container-height) feature[](#height)

| Field                   | Definition                                                                                       |
|-------------------------|--------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-height"></a><strong>height</strong>                                     |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container⑥"></a>[&#64;container](#at-ruledef-container)                |
| <strong>Value:</strong> | <a id="ref-for-length-value①"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong>Type:</strong>  | range                                                                                            |

The <a id="ref-for-descdef-container-height①"></a>[height](#descdef-container-height) <a id="ref-for-container-feature⑤"></a>[container feature](#container-feature) queries the <a id="ref-for-height"></a>[height](https://www.w3.org/TR/css-sizing-3/#height) of the <a id="ref-for-query-container④⓪"></a>[query container](#query-container)’s <a id="ref-for-content-box①"></a>[content box](https://www.w3.org/TR/css-box-4/#content-box).

#### <a id="inline-size"></a>6.1.3.  Inline-size: the <a id="ref-for-descdef-container-inline-size④"></a>[inline-size](#descdef-container-inline-size) feature[](#inline-size)

| Field                   | Definition                                                                                       |
|-------------------------|--------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-inline-size"></a><strong>inline-size</strong>                           |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container⑦"></a>[&#64;container](#at-ruledef-container)                |
| <strong>Value:</strong> | <a id="ref-for-length-value②"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong>Type:</strong>  | range                                                                                            |

The <a id="ref-for-descdef-container-inline-size⑤"></a>[inline-size](#descdef-container-inline-size) <a id="ref-for-container-feature⑥"></a>[container feature](#container-feature) queries the <a id="ref-for-size"></a>[size](https://www.w3.org/TR/css-sizing-3/#size) of the <a id="ref-for-query-container④①"></a>[query container](#query-container)’s <a id="ref-for-content-box②"></a>[content box](https://www.w3.org/TR/css-box-4/#content-box) in the <a id="ref-for-query-container④②"></a>query container’s <a id="ref-for-inline-axis②"></a>[inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

#### <a id="block-size"></a>6.1.4.  Block-size: the <a id="ref-for-descdef-container-block-size"></a>[block-size](#descdef-container-block-size) feature[](#block-size)

| Field                   | Definition                                                                                       |
|-------------------------|--------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-block-size"></a><strong>block-size</strong>                             |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container⑧"></a>[&#64;container](#at-ruledef-container)                |
| <strong>Value:</strong> | <a id="ref-for-length-value③"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong>Type:</strong>  | range                                                                                            |

The <a id="ref-for-descdef-container-block-size①"></a>[block-size](#descdef-container-block-size) <a id="ref-for-container-feature⑦"></a>[container feature](#container-feature) queries the <a id="ref-for-size①"></a>[size](https://www.w3.org/TR/css-sizing-3/#size) of the <a id="ref-for-query-container④③"></a>[query container](#query-container)’s <a id="ref-for-content-box③"></a>[content box](https://www.w3.org/TR/css-box-4/#content-box) in the <a id="ref-for-query-container④④"></a>query container’s <a id="ref-for-block-axis①"></a>[block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis).

#### <a id="aspect-ratio"></a>6.1.5.  Aspect-ratio: the <a id="ref-for-descdef-container-aspect-ratio"></a>[aspect-ratio](#descdef-container-aspect-ratio) feature[](#aspect-ratio)

| Field                   | Definition                                                                                   |
|-------------------------|----------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-aspect-ratio"></a><strong>aspect-ratio</strong>                     |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container⑨"></a>[&#64;container](#at-ruledef-container)            |
| <strong>Value:</strong> | <a id="ref-for-ratio-value"></a>[\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) |
| <strong>Type:</strong>  | range                                                                                        |

The <a id="ref-for-descdef-container-aspect-ratio①"></a>[aspect-ratio](#descdef-container-aspect-ratio) <a id="ref-for-container-feature⑧"></a>[container feature](#container-feature) is defined as the ratio of the value of the <a id="ref-for-descdef-container-width②"></a>[width](#descdef-container-width) <a id="ref-for-container-feature⑨"></a>container feature to the value of the <a id="ref-for-descdef-container-height②"></a>[height](#descdef-container-height) <a id="ref-for-container-feature①⓪"></a>container feature.

#### <a id="orientation"></a>6.1.6.  Orientation: the <a id="ref-for-descdef-container-orientation"></a>[orientation](#descdef-container-orientation) feature[](#orientation)

| Field                   | Definition                                                                                           |
|-------------------------|------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-orientation"></a><strong>orientation</strong>                               |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container①⓪"></a>[&#64;container](#at-ruledef-container)                   |
| <strong>Value:</strong> | portrait <a id="ref-for-comb-one③③"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) landscape |
| <strong>Type:</strong>  | discrete                                                                                             |

<strong><a id="valdef-container-orientation-portrait"></a><strong>portrait</strong></strong>

The <a id="ref-for-descdef-container-orientation①"></a>[orientation](#descdef-container-orientation) <a id="ref-for-container-feature①①"></a>[container feature](#container-feature) is <a id="ref-for-valdef-container-orientation-portrait"></a>[portrait](#valdef-container-orientation-portrait) when the value of the <a id="ref-for-descdef-container-height③"></a>[height](#descdef-container-height) <a id="ref-for-container-feature①②"></a>container feature is greater than or equal to the value of the <a id="ref-for-descdef-container-width③"></a>[width](#descdef-container-width) <a id="ref-for-container-feature①③"></a>container feature.

<strong><a id="valdef-container-orientation-landscape"></a><strong>landscape</strong></strong>

Otherwise <a id="ref-for-descdef-container-orientation②"></a>[orientation](#descdef-container-orientation) is <a id="ref-for-valdef-container-orientation-landscape"></a>[landscape](#valdef-container-orientation-landscape).

### <a id="style-container"></a>6.2.  Style Container Features[](#style-container)

A <a id="container-style-query"></a><strong>container style query</strong> allows querying the <a id="ref-for-computed-value⑥"></a>[computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of the <a id="ref-for-query-container④⑤"></a>[query container](#query-container). It is a boolean combination of individual <a id="style-features"></a><strong>style features</strong> (<a id="ref-for-typedef-style-feature③"></a>[\<style-feature\>](#typedef-style-feature)) that each query a single, specific property of the <a id="ref-for-query-container④⑥"></a>query container. The syntax of a <a id="ref-for-typedef-style-feature④"></a>\<style-feature\> is either the same as for a valid <a id="ref-for-declaration"></a>[declaration](https://www.w3.org/TR/css-syntax-3/#declaration)[\[CSS-SYNTAX-3\]](#biblio-css-syntax-3), a <a id="ref-for-typedef-style-feature-name②"></a>[\<style-feature-name\>](#typedef-style-feature-name) or a valid <a id="style-range"></a><strong>style range</strong> (<a id="ref-for-typedef-style-range②"></a>[\<style-range\>](#typedef-style-range)). The <a id="typedef-style-feature-name"></a><strong><a id="ref-for-typedef-style-feature-name③"></a>[\<style-feature-name\>](#typedef-style-feature-name)</strong> can be either a <a id="ref-for-supported-css-property"></a>[supported CSS property](https://www.w3.org/TR/cssom-1/#supported-css-property) or a valid <a id="ref-for-typedef-custom-property-name①"></a>[\<custom-property-name\>](https://www.w3.org/TR/css-variables-1/#typedef-custom-property-name). The <a id="typedef-style-feature-value"></a><strong><a id="ref-for-typedef-style-feature-value②"></a>[\<style-feature-value\>](#typedef-style-feature-value)</strong> production matches any valid <a id="ref-for-typedef-declaration-value"></a>[\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value) as long as it doesn’t contain <a id="ref-for-typedef-mf-lt②"></a>[\<mf-lt\>](https://www.w3.org/TR/mediaqueries-5/#typedef-mf-lt), <a id="ref-for-typedef-mf-gt②"></a>[\<mf-gt\>](https://www.w3.org/TR/mediaqueries-5/#typedef-mf-gt) and <a id="ref-for-typedef-mf-eq"></a>[\<mf-eq\>](https://www.w3.org/TR/mediaqueries-5/#typedef-mf-eq) tokens.

A <a id="ref-for-typedef-style-feature-plain②"></a>[\<style-feature-plain\>](#typedef-style-feature-plain) evaluates to true if the <a id="ref-for-computed-value⑦"></a>[computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the given property on the <a id="ref-for-query-container④⑦"></a>[query container](#query-container) matches the given value (which is also <a id="ref-for-computed-value⑧"></a>computed with respect to the <a id="ref-for-query-container④⑧"></a>query container), and false otherwise.

A <a id="ref-for-style-features"></a>[style feature](#style-features) without a value (<a id="ref-for-typedef-style-feature-boolean②"></a>[\<style-feature-boolean\>](#typedef-style-feature-boolean)) evaluates to true if the <a id="ref-for-computed-value⑨"></a>[computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) is different from the <a id="ref-for-initial-value①"></a>[initial value](https://www.w3.org/TR/css-cascade-5/#initial-value) for the given <a id="ref-for-css-property"></a>[property](https://www.w3.org/TR/css-cascade-5/#css-property).

To <a id="evaluate-a-style-range"></a><strong>evaluate a <a id="ref-for-typedef-style-range③"></a>[\<style-range\>](#typedef-style-range)</strong>, the following steps needs to be performed:

1.  If <a id="ref-for-typedef-style-range-value⑨"></a>[\<style-range-value\>](#typedef-style-range-value) is a <a id="ref-for-typedef-custom-property-name②"></a>[\<custom-property-name\>](https://www.w3.org/TR/css-variables-1/#typedef-custom-property-name), it needs to be substituted as if the <a id="ref-for-typedef-custom-property-name③"></a>\<custom-property-name\> was wrapped inside a <a id="ref-for-funcdef-var①"></a>[var()](https://www.w3.org/TR/css-variables-1/#funcdef-var).

2.  Substitute <a id="ref-for-arbitrary-substitution-function"></a>[arbitrary substitution function](https://www.w3.org/TR/css-values-5/#arbitrary-substitution-function) within <a id="ref-for-typedef-style-range-value①⓪"></a>[\<style-range-value\>](#typedef-style-range-value).

3.  Parse <a id="ref-for-typedef-style-range-value①①"></a>[\<style-range-value\>](#typedef-style-range-value) to <a id="ref-for-number-value"></a>[\<number\>](https://www.w3.org/TR/css-values-4/#number-value), <a id="ref-for-percentage-value"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), <a id="ref-for-length-value④"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value), <a id="ref-for-angle-value"></a>[\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), <a id="ref-for-time-value"></a>[\<time\>](https://www.w3.org/TR/css-values-4/#time-value), <a id="ref-for-frequency-value"></a>[\<frequency\>](https://www.w3.org/TR/css-values-4/#frequency-value) or <a id="ref-for-resolution-value"></a>[\<resolution\>](https://www.w3.org/TR/css-values-4/#resolution-value). If this cannot be done, evaluate to false.

4.  If each <a id="ref-for-typedef-style-range-value①②"></a>[\<style-range-value\>](#typedef-style-range-value) from the range have the same type, compute each and evaluate the comparison; a unitless zero must be treated as being a zero-<a id="ref-for-length-value⑤"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) when compared against a <a id="ref-for-length-value⑥"></a>\<length\> .

5.  Otherwise evaluate to false.

The boolean syntax and logic combining <a id="ref-for-style-features①"></a>[style features](#style-features) into a <a id="ref-for-container-style-query⑧"></a>[style query](#container-style-query) is the same as for <a id="ref-for-css-feature-queries①"></a>[CSS feature queries](https://www.w3.org/TR/css-conditional-3/#css-feature-queries). (See <a id="ref-for-at-ruledef-supports⑥"></a>[&#64;supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports). [\[css-conditional-3\]](#biblio-css-conditional-3))

<a id="ref-for-style-features②"></a>[Style features](#style-features) that query a <a id="ref-for-shorthand-property②"></a>[shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) are true if the <a id="ref-for-computed-value①⓪"></a>[computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) match for each of its <a id="ref-for-longhand"></a>[longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand), and false otherwise.

<a id="ref-for-cascade-dependent-keyword"></a>[Cascade-dependent keywords](https://drafts.csswg.org/css-cascade-5/#cascade-dependent-keyword), such as <a id="ref-for-valdef-all-revert"></a>[revert](https://www.w3.org/TR/css-cascade-5/#valdef-all-revert) and <a id="ref-for-valdef-all-revert-layer"></a>[revert-layer](https://www.w3.org/TR/css-cascade-5/#valdef-all-revert-layer), are invalid as values in a <a id="ref-for-style-features③"></a>[style feature](#style-features), and cause the <a id="ref-for-container-style-query⑨"></a>[container style query](#container-style-query) to be false.

Note: The remaining non-cascade-dependent <a id="ref-for-css-wide-keywords"></a>[CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) are <a id="ref-for-computed-value①①"></a>[computed](https://www.w3.org/TR/css-cascade-5/#computed-value) with respect to the <a id="ref-for-query-container④⑨"></a>[query container](#query-container), the same as other values.

### <a id="scroll-state-container"></a>6.3.  Scroll State Container Features[](#scroll-state-container)

A <a id="container-scroll-state-query"></a><strong>container scroll-state query</strong> allows querying a container for state that depends on scroll position. It is a boolean combination of individual <a id="scroll-state-feature"></a><strong>scroll-state features</strong> (<a id="ref-for-typedef-scroll-state-feature②"></a>[\<scroll-state-feature\>](#typedef-scroll-state-feature)) that each query a single feature of the <a id="ref-for-query-container⑤⓪"></a>[query container](#query-container). The syntax of a <a id="typedef-scroll-state-feature"></a><strong><a id="ref-for-typedef-scroll-state-feature③"></a>[\<scroll-state-feature\>](#typedef-scroll-state-feature)</strong> is the same as for a <a id="ref-for-media-feature②"></a>[media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature): a feature name, a comparator, and a value.

<a id="ref-for-scroll-state-feature"></a>[Scroll-state features](#scroll-state-feature) can either match state of the scroller itself (such as the scrollable feature), or an element that is affected by the scroll position of an ancestor <a id="ref-for-scroll-container"></a>[scroll container’s](https://www.w3.org/TR/css-overflow-3/#scroll-container) <a id="ref-for-scrollport"></a>[scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport) (such as the snapped feature).

#### <a id="updating-scroll-state"></a>6.3.1.  Updating Scroll State[](#updating-scroll-state)

Scroll state may cause layout cycles since queried scroll state may cause style changes, which may lead to scroll state changes as a result of layout.

To avoid such layout cycles, <a id="ref-for-valdef-container-type-scroll-state"></a>[scroll-state](#valdef-container-type-scroll-state) <a id="ref-for-query-container⑤①"></a>[query containers](#query-container) update their current state as part of <a id="ref-for-run-snapshot-post-layout-state-steps"></a>[run snapshot post-layout state steps](https://www.w3.org/TR/cssom-view-1/#run-snapshot-post-layout-state-steps) which is only run at specific points in the [HTML event loop processing model](https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model).

When asked to <a id="ref-for-run-snapshot-post-layout-state-steps①"></a>[run snapshot post-layout state steps](https://www.w3.org/TR/cssom-view-1/#run-snapshot-post-layout-state-steps), update the current state of every <a id="ref-for-valdef-container-type-scroll-state①"></a>[scroll-state](#valdef-container-type-scroll-state) <a id="ref-for-query-container⑤②"></a>[query container](#query-container). This snapshotted state will be used for any style and layout updates until the next time these steps are run.

#### <a id="stuck"></a>6.3.2.  Sticky positioning: the <a id="ref-for-descdef-container-stuck"></a>[stuck](#descdef-container-stuck) feature[](#stuck)

| Field                   | Definition                                                                                                                                                                                                                                                                                                                                                                                               |
|-------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-stuck"></a><strong>stuck</strong>                                                                                                                                                                                                                                                                                                                                               |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container①①"></a>[&#64;container](#at-ruledef-container)                                                                                                                                                                                                                                                                                                                       |
| <strong>Value:</strong> | none <a id="ref-for-comb-one③④"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) top <a id="ref-for-comb-one③⑤"></a>\| right <a id="ref-for-comb-one③⑥"></a>\| bottom <a id="ref-for-comb-one③⑦"></a>\| left <a id="ref-for-comb-one③⑧"></a>\| block-start <a id="ref-for-comb-one③⑨"></a>\| inline-start <a id="ref-for-comb-one④⓪"></a>\| block-end <a id="ref-for-comb-one④①"></a>\| inline-end |
| <strong>Type:</strong>  | discrete                                                                                                                                                                                                                                                                                                                                                                                                 |

The <a id="ref-for-descdef-container-stuck①"></a>[stuck](#descdef-container-stuck) <a id="ref-for-container-feature①④"></a>[container feature](#container-feature) queries whether a <a id="ref-for-valdef-position-sticky"></a>[sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) positioned container is visually shifted to stay inside the <a id="ref-for-sticky-view-rectangle"></a>[sticky view rectangle](https://www.w3.org/TR/css-position-3/#sticky-view-rectangle) for the given edge. The logical edges map to physical based on the direction and writing-mode of the <a id="ref-for-query-container⑤③"></a>[query container](#query-container). None of the values match if the <a id="ref-for-query-container⑤④"></a>query container is not <a id="ref-for-sticky-position"></a>[sticky positioned](https://www.w3.org/TR/css-position-3/#sticky-position).

It is possible for two values from opposite axes to match at the same time, but not for opposite edges along the same axis.

<a id="example-5e3347d6"></a>

<strong>Example:</strong>

[](#example-5e3347d6) May match:

``` text
@container scroll-state((stuck: top) and (stuck: left)) { ... }
```

Will never match:

``` text
@container scroll-state((stuck: left) and (stuck: right)) { ... }
```

<strong><a id="valdef-container-stuck-none"></a><strong>none</strong></strong>

The <a id="ref-for-valdef-position-sticky①"></a>[sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is not shifted in any direction.

<strong><a id="valdef-container-stuck-top"></a><strong>top</strong></strong>

The <a id="ref-for-valdef-position-sticky②"></a>[sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the top edge.

<strong><a id="valdef-container-stuck-right"></a><strong>right</strong></strong>

The <a id="ref-for-valdef-position-sticky③"></a>[sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the right edge.

<strong><a id="valdef-container-stuck-bottom"></a><strong>bottom</strong></strong>

The <a id="ref-for-valdef-position-sticky④"></a>[sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the bottom edge.

<strong><a id="valdef-container-stuck-left"></a><strong>left</strong></strong>

The <a id="ref-for-valdef-position-sticky⑤"></a>[sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the left edge.

<strong><a id="valdef-container-stuck-block-start"></a><strong>block-start</strong></strong>

The <a id="ref-for-valdef-position-sticky⑥"></a>[sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the <a id="ref-for-block-start"></a>[block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) edge.

<strong><a id="valdef-container-stuck-inline-start"></a><strong>inline-start</strong></strong>

The <a id="ref-for-valdef-position-sticky⑦"></a>[sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the <a id="ref-for-inline-start"></a>[inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) edge.

<strong><a id="valdef-container-stuck-block-end"></a><strong>block-end</strong></strong>

The <a id="ref-for-valdef-position-sticky⑧"></a>[sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the <a id="ref-for-block-end"></a>[block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) edge.

<strong><a id="valdef-container-stuck-inline-end"></a><strong>inline-end</strong></strong>

The <a id="ref-for-valdef-position-sticky⑨"></a>[sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the <a id="ref-for-inline-end"></a>[inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) edge.

#### <a id="snapped"></a>6.3.3.  Scroll snapping: the <a id="ref-for-descdef-container-snapped"></a>[snapped](#descdef-container-snapped) feature[](#snapped)

| Field                   | Definition                                                                                                                                                                                                                                           |
|-------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-snapped"></a><strong>snapped</strong>                                                                                                                                                                                       |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container①②"></a>[&#64;container](#at-ruledef-container)                                                                                                                                                                   |
| <strong>Value:</strong> | none <a id="ref-for-comb-one④②"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) x <a id="ref-for-comb-one④③"></a>\| y <a id="ref-for-comb-one④④"></a>\| block <a id="ref-for-comb-one④⑤"></a>\| inline <a id="ref-for-comb-one④⑥"></a>\| both |
| <strong>Type:</strong>  | discrete                                                                                                                                                                                                                                             |

The <a id="ref-for-descdef-container-snapped①"></a>[snapped](#descdef-container-snapped) <a id="ref-for-container-feature①⑤"></a>[container feature](#container-feature) queries whether a <a id="ref-for-snap-target"></a>[snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) is, or would be, snapped to its <a id="ref-for-scroll-snap-container"></a>[scroll snap container](https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-container) in the given axis. That is, it matches any <a id="ref-for-snap-target①"></a>snap target that the <code><a id="ref-for-eventdef-snapevent-scrollsnapchanging"></a>[scrollsnapchanging](https://www.w3.org/TR/css-scroll-snap-2/#eventdef-snapevent-scrollsnapchanging)</code> event is fired for.

<strong><a id="valdef-container-snapped-none"></a><strong>none</strong></strong>

The <a id="ref-for-query-container⑤⑤"></a>[query container](#query-container) is not a <a id="ref-for-snap-target②"></a>[snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target).

<strong><a id="valdef-container-snapped-x"></a><strong>x</strong></strong>

<a id="ref-for-descdef-container-snapped②"></a>[snapped](#descdef-container-snapped) <a id="ref-for-container-feature①⑥"></a>[container feature](#container-feature) matches <a id="ref-for-valdef-container-snapped-x"></a>[x](#valdef-container-snapped-x) if the <a id="ref-for-query-container⑤⑥"></a>[query container](#query-container) is a horizontal <a id="ref-for-snap-target③"></a>[snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) for its <a id="ref-for-scroll-container①"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container).

<strong><a id="valdef-container-snapped-y"></a><strong>y</strong></strong>

<a id="ref-for-descdef-container-snapped③"></a>[snapped](#descdef-container-snapped) <a id="ref-for-container-feature①⑦"></a>[container feature](#container-feature) matches <a id="ref-for-valdef-container-snapped-y"></a>[y](#valdef-container-snapped-y) if the <a id="ref-for-query-container⑤⑦"></a>[query container](#query-container) is a vertical <a id="ref-for-snap-target④"></a>[snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) for its <a id="ref-for-scroll-container②"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container).

<strong><a id="valdef-container-snapped-block"></a><strong>block</strong></strong>

<a id="ref-for-descdef-container-snapped④"></a>[snapped](#descdef-container-snapped) <a id="ref-for-container-feature①⑧"></a>[container feature](#container-feature) matches <a id="ref-for-valdef-container-snapped-block"></a>[block](#valdef-container-snapped-block) if the <a id="ref-for-query-container⑤⑧"></a>[query container](#query-container) is a <a id="ref-for-snap-target⑤"></a>[snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) for its <a id="ref-for-scroll-container③"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container). in the block direction of the <a id="ref-for-scroll-snap-container①"></a>[scroll snap container](https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-container).

<strong><a id="valdef-container-snapped-inline"></a><strong>inline</strong></strong>

<a id="ref-for-descdef-container-snapped⑤"></a>[snapped](#descdef-container-snapped) <a id="ref-for-container-feature①⑨"></a>[container feature](#container-feature) matches <a id="ref-for-valdef-container-snapped-inline"></a>[inline](#valdef-container-snapped-inline) if the <a id="ref-for-query-container⑤⑨"></a>[query container](#query-container) is a <a id="ref-for-snap-target⑥"></a>[snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) for its <a id="ref-for-scroll-container④"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) in the inline direction of the <a id="ref-for-scroll-snap-container②"></a>[scroll snap container](https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-container).

<strong><a id="valdef-container-snapped-both"></a><strong>both</strong></strong>

<a id="ref-for-descdef-container-snapped⑥"></a>[snapped](#descdef-container-snapped) <a id="ref-for-container-feature②⓪"></a>[container feature](#container-feature) matches <a id="ref-for-valdef-container-snapped-both"></a>[both](#valdef-container-snapped-both) if the <a id="ref-for-query-container⑥⓪"></a>[query container](#query-container) is a <a id="ref-for-snap-target⑦"></a>[snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) for its <a id="ref-for-scroll-container⑤"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) in both directions of the <a id="ref-for-scroll-snap-container③"></a>[scroll snap container](https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-container).

#### <a id="scrollable"></a>6.3.4.  Scrollable: the <a id="ref-for-descdef-container-scrollable"></a>[scrollable](#descdef-container-scrollable) feature[](#scrollable)

| Field                   | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|-------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-scrollable"></a><strong>scrollable</strong>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container①③"></a>[&#64;container](#at-ruledef-container)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong>Value:</strong> | none <a id="ref-for-comb-one④⑦"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) top <a id="ref-for-comb-one④⑧"></a>\| right <a id="ref-for-comb-one④⑨"></a>\| bottom <a id="ref-for-comb-one⑤⓪"></a>\| left <a id="ref-for-comb-one⑤①"></a>\| block-start <a id="ref-for-comb-one⑤②"></a>\| inline-start <a id="ref-for-comb-one⑤③"></a>\| block-end <a id="ref-for-comb-one⑤④"></a>\| inline-end <a id="ref-for-comb-one⑤⑤"></a>\| x <a id="ref-for-comb-one⑤⑥"></a>\| y <a id="ref-for-comb-one⑤⑦"></a>\| block <a id="ref-for-comb-one⑤⑧"></a>\| inline |
| <strong>Type:</strong>  | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |

The <a id="ref-for-descdef-container-scrollable①"></a>[scrollable](#descdef-container-scrollable) <a id="ref-for-container-feature②①"></a>[container feature](#container-feature) queries whether a <a id="ref-for-scroll-container⑥"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has clipped <a id="ref-for-scrollable-overflow-rectangle"></a>[scrollable overflow rectangle](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-rectangle) content in the given direction which is reachable through user initiated scrolling. That is, <a id="ref-for-descdef-container-scrollable②"></a>scrollable does not match for a <a id="ref-for-valdef-overflow-hidden①"></a>[hidden](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-hidden) container, nor for a <a id="ref-for-unreachable-scrollable-overflow-region"></a>[unreachable scrollable overflow region](https://www.w3.org/TR/css-overflow-3/#unreachable-scrollable-overflow-region).

The logical values map to physical based on the direction and writing-mode of the <a id="ref-for-query-container⑥①"></a>[query container](#query-container). None of the values match if the container is not a <a id="ref-for-scroll-container⑦"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container).

<strong><a id="valdef-container-scrollable-none"></a><strong>none</strong></strong>

The <a id="ref-for-scroll-container⑧"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) does not have <a id="ref-for-scrollable-overflow"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) in any direction.

<strong><a id="valdef-container-scrollable-top"></a><strong>top</strong></strong>

The <a id="ref-for-scroll-container⑨"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has <a id="ref-for-scrollable-overflow①"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the top edge.

<strong><a id="valdef-container-scrollable-right"></a><strong>right</strong></strong>

The <a id="ref-for-scroll-container①⓪"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has <a id="ref-for-scrollable-overflow②"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the right edge.

<strong><a id="valdef-container-scrollable-bottom"></a><strong>bottom</strong></strong>

The <a id="ref-for-scroll-container①①"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has <a id="ref-for-scrollable-overflow③"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the bottom edge.

<strong><a id="valdef-container-scrollable-left"></a><strong>left</strong></strong>

The <a id="ref-for-scroll-container①②"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has <a id="ref-for-scrollable-overflow④"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the left edge.

<strong><a id="valdef-container-scrollable-block-start"></a><strong>block-start</strong></strong>

The <a id="ref-for-scroll-container①③"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has <a id="ref-for-scrollable-overflow⑤"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the <a id="ref-for-block-start①"></a>[block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) edge.

<strong><a id="valdef-container-scrollable-inline-start"></a><strong>inline-start</strong></strong>

The <a id="ref-for-scroll-container①④"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has <a id="ref-for-scrollable-overflow⑥"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the <a id="ref-for-inline-start①"></a>[inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) edge.

<strong><a id="valdef-container-scrollable-block-end"></a><strong>block-end</strong></strong>

The <a id="ref-for-scroll-container①⑤"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has <a id="ref-for-scrollable-overflow⑦"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the <a id="ref-for-block-end①"></a>[block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) edge.

<strong><a id="valdef-container-scrollable-inline-end"></a><strong>inline-end</strong></strong>

The <a id="ref-for-scroll-container①⑥"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has <a id="ref-for-scrollable-overflow⑧"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the <a id="ref-for-inline-end①"></a>[inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) edge.

<strong><a id="valdef-container-scrollable-x"></a><strong>x</strong></strong>

The <a id="ref-for-scroll-container①⑦"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has horizontally <a id="ref-for-scrollable-overflow⑨"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow).

<strong><a id="valdef-container-scrollable-y"></a><strong>y</strong></strong>

The <a id="ref-for-scroll-container①⑧"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has vertically <a id="ref-for-scrollable-overflow①⓪"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow).

<strong><a id="valdef-container-scrollable-block"></a><strong>block</strong></strong>

The <a id="ref-for-scroll-container①⑨"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has <a id="ref-for-scrollable-overflow①①"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) in its block direction.

<strong><a id="valdef-container-scrollable-inline"></a><strong>inline</strong></strong>

The <a id="ref-for-scroll-container②⓪"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has <a id="ref-for-scrollable-overflow①②"></a>[scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) in its inline direction.

#### <a id="scrolled"></a>6.3.5.  Scrolled: the <a id="ref-for-descdef-container-scrolled"></a>[scrolled](#descdef-container-scrolled) feature[](#scrolled)

| Field                   | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|-------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>  | <a id="descdef-container-scrolled"></a><strong>scrolled</strong>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong>For:</strong>   | <a id="ref-for-at-ruledef-container①④"></a>[&#64;container](#at-ruledef-container)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong>Value:</strong> | none <a id="ref-for-comb-one⑤⑨"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) top <a id="ref-for-comb-one⑥⓪"></a>\| right <a id="ref-for-comb-one⑥①"></a>\| bottom <a id="ref-for-comb-one⑥②"></a>\| left <a id="ref-for-comb-one⑥③"></a>\| block-start <a id="ref-for-comb-one⑥④"></a>\| inline-start <a id="ref-for-comb-one⑥⑤"></a>\| block-end <a id="ref-for-comb-one⑥⑥"></a>\| inline-end <a id="ref-for-comb-one⑥⑦"></a>\| x <a id="ref-for-comb-one⑥⑧"></a>\| y <a id="ref-for-comb-one⑥⑨"></a>\| block <a id="ref-for-comb-one⑦⓪"></a>\| inline |
| <strong>Type:</strong>  | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |

For a <a id="ref-for-query-container⑥②"></a>[query container](#query-container) that is a <a id="ref-for-scroll-container②①"></a>[scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), the <a id="ref-for-descdef-container-scrolled①"></a>[scrolled](#descdef-container-scrolled) <a id="ref-for-container-feature②②"></a>[container feature](#container-feature) queries the direction of its most recent <a id="ref-for-relative-scroll"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll). The logical values map to physical based on the direction and writing-mode of the <a id="ref-for-query-container⑥③"></a>query container. None of the values match if the container is not a <a id="ref-for-scroll-container②②"></a>scroll container.

<strong><a id="valdef-container-scrolled-none"></a><strong>none</strong></strong>

The <a id="ref-for-query-container⑥④"></a>[query container](#query-container) has not had a <a id="ref-for-relative-scroll①"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) yet.

<strong><a id="valdef-container-scrolled-top"></a><strong>top</strong></strong>

The most recent <a id="ref-for-relative-scroll②"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was upwards.

<strong><a id="valdef-container-scrolled-right"></a><strong>right</strong></strong>

The most recent <a id="ref-for-relative-scroll③"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was to the right.

<strong><a id="valdef-container-scrolled-bottom"></a><strong>bottom</strong></strong>

The most recent <a id="ref-for-relative-scroll④"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was downwards.

<strong><a id="valdef-container-scrolled-left"></a><strong>left</strong></strong>

The most recent <a id="ref-for-relative-scroll⑤"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was to the left.

<strong><a id="valdef-container-scrolled-block-start"></a><strong>block-start</strong></strong>

The most recent <a id="ref-for-relative-scroll⑥"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was towards the <a id="ref-for-block-start②"></a>[block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) direction.

<strong><a id="valdef-container-scrolled-inline-start"></a><strong>inline-start</strong></strong>

The most recent <a id="ref-for-relative-scroll⑦"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was towards the <a id="ref-for-inline-start②"></a>[inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) direction.

<strong><a id="valdef-container-scrolled-block-end"></a><strong>block-end</strong></strong>

The most recent <a id="ref-for-relative-scroll⑧"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was towards the <a id="ref-for-block-end②"></a>[block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) direction.

<strong><a id="valdef-container-scrolled-inline-end"></a><strong>inline-end</strong></strong>

The most recent <a id="ref-for-relative-scroll⑨"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was towards the <a id="ref-for-inline-end②"></a>[inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) direction.

<strong><a id="valdef-container-scrolled-x"></a><strong>x</strong></strong>

The most recent <a id="ref-for-relative-scroll①⓪"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was in the horizontal direction.

<strong><a id="valdef-container-scrolled-y"></a><strong>y</strong></strong>

The most recent <a id="ref-for-relative-scroll①①"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was in the vertical direction.

<strong><a id="valdef-container-scrolled-block"></a><strong>block</strong></strong>

The most recent <a id="ref-for-relative-scroll①②"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was in the block direction.

<strong><a id="valdef-container-scrolled-inline"></a><strong>inline</strong></strong>

The most recent <a id="ref-for-relative-scroll①③"></a>[relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was in the inline direction.

## <a id="container-lengths"></a>7.  Container Relative Lengths: the cqw, cqh, cqi, cqb, cqmin, cqmax units[](#container-lengths)

<a id="container-query-length"></a><strong>Container query length units</strong> specify a length relative to the dimensions of a <a id="ref-for-query-container⑥⑤"></a>[query container](#query-container). Style sheets that use <a id="ref-for-container-query-length②"></a>[container query length](#container-query-length) units can more easily move components from one <a id="ref-for-query-container⑥⑥"></a>query container to another.

The <a id="ref-for-container-query-length③"></a>[container query length](#container-query-length) units are:

Informative Summary of Container Units

| unit  | relative to                                                                                                                                                                        |
|-------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| cqw   | 1% of a <a id="ref-for-query-container⑥⑦"></a>[query container](#query-container)’s <a id="ref-for-width①"></a>[width](https://www.w3.org/TR/css-sizing-3/#width)                  |
| cqh   | 1% of a <a id="ref-for-query-container⑥⑧"></a>[query container](#query-container)’s <a id="ref-for-height①"></a>[height](https://www.w3.org/TR/css-sizing-3/#height)               |
| cqi   | 1% of a <a id="ref-for-query-container⑥⑨"></a>[query container](#query-container)’s <a id="ref-for-inline-size"></a>[inline size](https://www.w3.org/TR/css-sizing-3/#inline-size) |
| cqb   | 1% of a <a id="ref-for-query-container⑦⓪"></a>[query container](#query-container)’s <a id="ref-for-block-size"></a>[block size](https://www.w3.org/TR/css-sizing-3/#block-size)    |
| cqmin | The smaller value of cqi or cqb                                                                                                                                                    |
| cqmax | The larger value of cqi or cqb                                                                                                                                                     |

For each element, <a id="ref-for-container-query-length④"></a>[container query length](#container-query-length) units are evaluated using the same rules as <a id="ref-for-container-size-query⑨"></a>[container size queries](#container-size-query) on the relevant axis (or axes) described by the unit. The <a id="ref-for-query-container⑦①"></a>[query container](#query-container) for each axis is the nearest ancestor container that accepts <a id="ref-for-container-size-query①⓪"></a>container size queries on that axis. If no eligible <a id="ref-for-query-container⑦②"></a>query container is available, then use the <a id="ref-for-small-viewport-size"></a>[small viewport size](https://www.w3.org/TR/css-values-4/#small-viewport-size) for that axis.

Note: In some cases cqi and cqb units on the same element will evaluate in relation to different <a id="ref-for-query-container⑦③"></a>[query containers](#query-container). Similarly, cqmin and cqmax units represent the larger or smaller of the cqi and cqb units, even when those dimensions come from different <a id="ref-for-query-container⑦④"></a>query containers.

Child elements do not inherit the relative values as specified for their parent; they inherit the <a id="ref-for-computed-value①②"></a>[computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

<a id="example-a4252068"></a>

<strong>Example:</strong>

[](#example-a4252068) Authors can ensure that <a id="ref-for-container-query-length⑤"></a>[container query length](#container-query-length) units have an appropriate <a id="ref-for-query-container⑦⑤"></a>[query container](#query-container) by applying them inside a <a id="ref-for-container-query①④"></a>[container query](#container-query) that relies on the same container-type. Custom fallback values can be defined outside the <a id="ref-for-container-query①⑤"></a>container query:

``` text
/* The fallback value does not rely on containment */
h2 { font-size: 1.2em; }

@container (inline-size >= 0px) {
  /* only applies when an inline-size container is available */
  h2 { font-size: calc(1.2em + 1cqi); }
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

## <a id="apis"></a>9. APIs[](#apis)

### <a id="the-csscontainerrule-interface"></a>9.1.  The <code>CSSContainerRule</code> interface[](#the-csscontainerrule-interface)

The <code><a id="ref-for-csscontainerrule"></a>[CSSContainerRule](#csscontainerrule)</code> interface represents an <a id="ref-for-at-ruledef-container①⑤"></a>[&#64;container](#at-ruledef-container) rule.

<a id="dictdef-csscontainercondition"></a><a id="ref-for-cssomstring"></a><a id="dom-csscontainercondition-name"></a><a id="ref-for-cssomstring①"></a><a id="dom-csscontainercondition-query"></a><a id="ref-for-Exposed"></a><a id="csscontainerrule"></a><a id="ref-for-cssconditionrule"></a><a id="ref-for-cssomstring②"></a><a id="dom-csscontainerrule-containername"></a><a id="ref-for-cssomstring③"></a><a id="dom-csscontainerrule-containerquery"></a><a id="ref-for-idl-frozen-array"></a><a id="ref-for-dictdef-csscontainercondition"></a><a id="dom-csscontainerrule-conditions"></a>

``` text
dictionary CSSContainerCondition {
  required CSSOMString name;
  required CSSOMString query;
};

[Exposed=Window]
interface CSSContainerRule : CSSConditionRule {
  readonly attribute CSSOMString containerName;
  readonly attribute CSSOMString containerQuery;
  readonly attribute FrozenArray<CSSContainerCondition> conditions;
};
```

<a id="issue-9b61d539"></a>

<strong>Issue:</strong>

[](#issue-9b61d539) We should try to remove <code><a id="ref-for-dom-csscontainerrule-containername"></a>[containerName](#dom-csscontainerrule-containername)</code> and <code><a id="ref-for-dom-csscontainerrule-containerquery"></a>[containerQuery](#dom-csscontainerrule-containerquery)</code>, since they don’t deal with multiple conditions correctly.

<strong><code>conditionText</code> of type <code>CSSOMString</code> (CSSContainerRule-specific definition for attribute on CSSConditionRule)</strong>

The <code>conditionText</code> attribute (defined on the <code>CSSConditionRule</code> parent rule), on getting, must return a value as follows:

1.  Let <var>conditions</var> be the result of getting the <code><a id="ref-for-dom-csscontainerrule-conditions"></a>[conditions](#dom-csscontainerrule-conditions)</code> attribute.

2.  Let <var>first</var> be <code>true</code>.

3.  Let <var>result</var> be the empty string.

4.  <a id="ref-for-list-iterate"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>condition</var> in <var>conditions</var>:

    1.  If <var>first</var> is <code>false</code>, append <code>&#34;, &#34;</code> to <var>result</var>.

    2.  Set <var>first</var> to <code>false</code>.

    3.  If <var>condition</var>’s <code><a id="ref-for-dom-csscontainercondition-name"></a>[name](#dom-csscontainercondition-name)</code> is not empty:

        1.  Append <var>condition</var>’s <code><a id="ref-for-dom-csscontainercondition-name①"></a>[name](#dom-csscontainercondition-name)</code> to <var>result</var>.

        2.  If <var>condition</var>’s <code><a id="ref-for-dom-csscontainercondition-query"></a>[query](#dom-csscontainercondition-query)</code> is not empty, append a single space to <var>result</var>.

    4.  Append <var>condition</var>’s <code><a id="ref-for-dom-csscontainercondition-query①"></a>[query](#dom-csscontainercondition-query)</code> to <var>result</var>.

5.  Return <var>result</var>

<strong><code>containerName</code> of type <code>CSSOMString</code></strong>

The <code>containerName</code> attribute, on getting, must return a value as follows:

1.  Let <var>conditions</var> be the result of getting the <code><a id="ref-for-dom-csscontainerrule-conditions①"></a>[conditions](#dom-csscontainerrule-conditions)</code> attribute.

2.  If the length of <var>conditions</var> is 1:

    1.  Return the only <var>conditions</var> item’s <code><a id="ref-for-dom-csscontainercondition-name②"></a>[name](#dom-csscontainercondition-name)</code>.

3.  return <code>&#34;&#34;</code>.

<strong><code>containerQuery</code> of type <code>CSSOMString</code></strong>

The <code>containerQuery</code> attribute, on getting, must return a value as follows:

1.  Let <var>conditions</var> be the result of getting the <code><a id="ref-for-dom-csscontainerrule-conditions②"></a>[conditions](#dom-csscontainerrule-conditions)</code> attribute.

2.  If the length of <var>conditions</var> is 1:

    1.  Return the only <var>conditions</var> item’s <code><a id="ref-for-dom-csscontainercondition-query②"></a>[query](#dom-csscontainercondition-query)</code>.

3.  Return <code>&#34;&#34;</code>.

<strong><code>conditions</code> of type <code>FrozenArray&#60;CSSContainerCondition?&#62;</code></strong>

The <code>conditions</code> attribute, on getting, must return a value as follows:

1.  Let <var>result</var> be an empty <a id="ref-for-list"></a>[list](https://infra.spec.whatwg.org/#list).

2.  <a id="ref-for-list-iterate①"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <a id="ref-for-typedef-container-condition⑦"></a>[\<container-condition\>](#typedef-container-condition) <var>condition</var> specified in the rule:

    1.  Let <var>dict</var> be a new <code><a id="ref-for-dictdef-csscontainercondition①"></a>[CSSContainerCondition](#dictdef-csscontainercondition)</code> with <code><a id="ref-for-dom-csscontainercondition-name③"></a>[name](#dom-csscontainercondition-name)</code> set to the <a id="ref-for-serialize-an-identifier"></a>[serialized](https://www.w3.org/TR/cssom-1/#serialize-an-identifier) <a id="ref-for-typedef-container-name④"></a>[\<container-name\>](#typedef-container-name) of <var>condition</var> if specified, or <code>&#34;&#34;</code> otherwise, and <code><a id="ref-for-dom-csscontainercondition-query③"></a>[query](#dom-csscontainercondition-query)</code> set to the <a id="ref-for-typedef-container-query⑦"></a>[\<container-query\>](#typedef-container-query) specified in <var>condition</var> without any logical simplifications, so that the returned query will evaluate to the same result as the specified query in any conformant implementation of this specification (including implementations that implement future extensions allowed by the <a id="ref-for-typedef-general-enclosed④"></a>[\<general-enclosed\>](https://www.w3.org/TR/mediaqueries-5/#typedef-general-enclosed) extensibility mechanism in this specification). In other words, token stream simplifications are allowed (such as reducing whitespace to a single space or omitting it in cases where it is known to be optional), but logical simplifications (such as removal of unneeded parentheses, or simplification based on evaluating results) are not allowed.

    2.  <a id="ref-for-list-append"></a>[Append](https://infra.spec.whatwg.org/#list-append) <var>dict</var> to <var>result</var>.

3.  Return <var>result</var>.

<a id="issue-2e3d6538"></a>

<strong>Issue:</strong>

[](#issue-2e3d6538) Container Queries should have a <code>matchContainer</code> method. This will be modeled on <code><a id="ref-for-dom-window-matchmedia"></a>[matchMedia()](https://www.w3.org/TR/cssom-view-1/#dom-window-matchmedia)</code> and the <code><a id="ref-for-mediaquerylist"></a>[MediaQueryList](https://www.w3.org/TR/cssom-view-1/#mediaquerylist)</code> interface, but applied to Elements rather than the Window. When measuring layout sizes, it behaves Similar to <code>resizeObserver</code>, but it provides the additional Container Query syntax and features. [\[Issue \#6205\]](https://github.com/w3c/csswg-drafts/issues/6205)

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

## <a id="security"></a>Security Considerations[](#security)

No security issues have been raised against this document

## <a id="privacy"></a>Privacy Considerations[](#privacy)

The <a id="ref-for-funcdef-supports-font-tech"></a>[font-tech()](#funcdef-supports-font-tech) and <a id="ref-for-funcdef-supports-font-format"></a>[font-format()](#funcdef-supports-font-format) functions may provide information about the user’s software such as its version and whether it is running with non-default settings that enable or disable certain features.

This information can also be determined through other APIs. However, the features in this specification are one of the ways this information is exposed on the Web.

This information can also, in aggregate, be used to improve the accuracy of [fingerprinting](https://www.w3.org/2001/tag/doc/unsanctioned-tracking/) of the user.

## <a id="changes"></a> Changes[](#changes)

### <a id="changes-20241105"></a> Changes since the [Working Draft of 5 November 2024](https://www.w3.org/TR/2024/WD-css-conditional-5-20241105/) [](#changes-20241105)

- Extended <a id="ref-for-supports-queries③"></a>[supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries) to express <a id="ref-for-css-environment-variable①"></a>[environment variable](https://www.w3.org/TR/css-env-1/#css-environment-variable) capabilities via <a id="ref-for-funcdef-supports-env"></a>[env()](#funcdef-supports-env). ([\#3576](https://github.com/w3c/csswg-drafts/issues/3576))
- Clarified that the last &#64;supports-condition in document order wins ([\#12973](https://github.com/w3c/csswg-drafts/issues/12973))
- Extended <a id="ref-for-supports-queries④"></a>[supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries) to express <a id="ref-for-at-rule⑤"></a>[at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule) capabilities via <a id="ref-for-funcdef-supports-at-rule"></a>[at-rule()](#funcdef-supports-at-rule). ([\#2463](https://github.com/w3c/csswg-drafts/issues/2463), [\#6966](https://github.com/w3c/csswg-drafts/issues/6966), [\#11116](https://github.com/w3c/csswg-drafts/issues/11116), [\#11117](https://github.com/w3c/csswg-drafts/issues/11117))
- Added <a id="ref-for-at-ruledef-supports-condition④"></a>[&#64;supports-condition](#at-ruledef-supports-condition) at-rule and related <code><a id="ref-for-csssupportsconditionrule①"></a>[CSSSupportsConditionRule](#csssupportsconditionrule)</code> interface. ([\#12622](https://github.com/w3c/csswg-drafts/issues/12622))
- Clarified that container-names are not tree-scoped ([\#12090](https://github.com/w3c/csswg-drafts/issues/12090))
- Defined direction feature for scroll-state() queries ([\#6400](https://github.com/w3c/csswg-drafts/issues/6400#issuecomment-3200664309))
- Clarified that 0 and 0px are equivalent in conditions ([\#12236](https://github.com/w3c/csswg-drafts/issues/12236#issuecomment-3204826253))
- Defined a range syntax for style container queries ([\#8376](https://github.com/w3c/csswg-drafts/issues/8376#issuecomment-2773374483))
- Explicitly allow tree-counting functions ([\#10982](https://github.com/w3c/csswg-drafts/issues/10982))
- Dimensional query containers no longer apply layout containment ([\#10544](https://github.com/w3c/csswg-drafts/pull/10544))
- Added "both" value for snapped query ([\#11181](https://github.com/w3c/csswg-drafts/issues/11181))
- Added axis keywords for overflowing ([\#11183](https://github.com/w3c/csswg-drafts/issues/11183))
- Renamed overflowing to scrollable ([\#11182](https://github.com/w3c/csswg-drafts/issues/11182))
- Made <a id="ref-for-typedef-container-query⑧"></a>[\<container-query\>](#typedef-container-query) optional ([\#9192](https://github.com/w3c/csswg-drafts/issues/9192#issuecomment-1789850349))
- Extended <a id="ref-for-supports-queries⑤"></a>[supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries) to express named features via <a id="ref-for-funcdef-supports-named-feature"></a>[named-feature()](#funcdef-supports-named-feature). ([\#9875](https://github.com/w3c/csswg-drafts/issues/9875))

### <a id="changes-20240723"></a> Changes since the [Working Draft of 23 July 2024](https://www.w3.org/TR/2024/WD-css-conditional-5-20240723/) [](#changes-20240723)

- Added <a id="ref-for-valdef-container-stuck-none"></a>[none](#valdef-container-stuck-none)-keywords to scroll-state() features ([\#10874](https://github.com/w3c/csswg-drafts/pull/10874))
- Added container-type:scroll-state, and scroll-state() queries for stuck, snapped, and scrollable features ([\#6402](https://github.com/w3c/csswg-drafts/issues/6402#issuecomment-1812973013), [\#10784](https://github.com/w3c/csswg-drafts/issues/10784#issuecomment-2379901508), [\#10796](https://github.com/w3c/csswg-drafts/issues/10796#issuecomment-2379885032))
- Corrected example (there is no container-type:style)
- Specified that container queries use the flat tree ([\#5984](https://github.com/w3c/csswg-drafts/issues/5984#issuecomment-2112977366))

### <a id="changes-20211221"></a> Changes since the [First Public Working Draft of 21 December 2021](https://www.w3.org/TR/2021/WD-css-conditional-5-20211221/) [](#changes-20211221)

- Moved container queries to this specification, from CSS Contain 3 ([\#10433](https://github.com/w3c/csswg-drafts/issues/10433))
- Imported the definitions of \<font-format\> and \<font-tech\> from CSS Fonts 4, rather than duplicating them in this specification ([\#8110](https://github.com/w3c/csswg-drafts/issues/8110))
- Updated to use the new parsing algorithm names and block production names
- Corrected a typo in the grammar of \<font-format\>
- Corrected extra spaces in the font-tech and font-format productions ([\#7369](https://github.com/w3c/csswg-drafts/issues/7369))

### <a id="changes-from-L4"></a> Additions since Level 4[](#changes-from-L4)

- Added <a id="ref-for-at-ruledef-when⑦"></a>[&#64;when](#at-ruledef-when) and <a id="ref-for-at-ruledef-else⑨"></a>[&#64;else](#at-ruledef-else).
- Extended <a id="ref-for-supports-queries⑥"></a>[supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries) to express font capabilities via <a id="ref-for-funcdef-supports-font-tech①"></a>[font-tech()](#funcdef-supports-font-tech) and <a id="ref-for-funcdef-supports-font-format①"></a>[font-format()](#funcdef-supports-font-format).
- Extended <a id="ref-for-supports-queries⑦"></a>[supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries) to express at-rule capabilities via <a id="ref-for-funcdef-supports-at-rule①"></a>[at-rule()](#funcdef-supports-at-rule).
- Extended <a id="ref-for-supports-queries⑧"></a>[supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries) to express named features via <a id="ref-for-funcdef-supports-named-feature①"></a>[named-feature()](#funcdef-supports-named-feature).
- Moved Container Queries from [\[CSS-CONTAIN-3\]](#biblio-css-contain-3) to this specification. (See also the [CSS Containment 3 § A Changes](https://www.w3.org/TR/css-contain-3/#changes) for more information on the evolution of this feature.)
- Added <a id="ref-for-at-ruledef-supports-condition⑤"></a>[&#64;supports-condition](#at-ruledef-supports-condition) at-rule and related <code><a id="ref-for-csssupportsconditionrule②"></a>[CSSSupportsConditionRule](#csssupportsconditionrule)</code> interface.

## <a id="acknowledgments"></a>Acknowledgments[](#acknowledgments)

The <a id="ref-for-at-ruledef-when⑧"></a>[&#64;when](#at-ruledef-when) and <a id="ref-for-at-ruledef-else①⓪"></a>[&#64;else](#at-ruledef-else) rules are based on a proposal by Tab Atkins.

Comments and previous work from Adam Argyle, Amelia Bellamy-Royds, Anders Hartvoll Ruud, Brian Kardell, Chris Coyier, Christopher Kirk-Nielsen, David Herron, Eric Portis, Ethan Marcotte, Florian Rivoal, Geoff Graham, Gregory Wild-Smith, Ian Kilpatrick, Jen Simmons, Kenneth Rohde Christiansen, Lea Verou, Martin Auswöger, Martine Dowden, Mike Riethmuller, Morten Stenshorne, Nicole Sullivan, Rune Lillesveen, Scott Jehl Scott Kellum, Stacy Kvernmo, Theresa O’Connor, Una Kravets, and many others have contributed to this specification.

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

### <a id="w3c-conformance-classes"></a> Conformance classes[](#w3c-conformance-classes)

Conformance to this specification is defined for three conformance classes:

<strong>style sheet</strong>

A [CSS style sheet](https://www.w3.org/TR/CSS21/conform.html#style-sheet).

<strong>renderer</strong>

A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

<strong>authoring tool</strong>

A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="w3c-partial"></a> Partial implementations[](#w3c-partial)

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](https://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="w3c-conform-future-proofing"></a> Implementations of Unstable and Proprietary Features[](#w3c-conform-future-proofing)

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

### <a id="w3c-testing"></a> Non-experimental implementations[](#w3c-testing)

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at <https://www.w3.org/Style/CSS/Test/>. Questions should be directed to the [public-css-testsuite&#64;w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index[](#index)

### <a id="index-defined-here"></a>Terms defined by this specification[](#index-defined-here)

- [anchor-position-follows-transforms](#anchor-position-follows-transforms), in § 2.1.3
- [aspect-ratio](#descdef-container-aspect-ratio), in § 6.1.5
- [at-rule()](#funcdef-supports-at-rule), in § 2
- block
  - [value for &#64;container/scrollable](#valdef-container-scrollable-block), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-block), in § 6.3.5
  - [value for &#64;container/snapped](#valdef-container-snapped-block), in § 6.3.3
- block-end
  - [value for &#64;container/scrollable](#valdef-container-scrollable-block-end), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-block-end), in § 6.3.5
  - [value for &#64;container/stuck](#valdef-container-stuck-block-end), in § 6.3.2
- [block-size](#descdef-container-block-size), in § 6.1.4
- block-start
  - [value for &#64;container/scrollable](#valdef-container-scrollable-block-start), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-block-start), in § 6.3.5
  - [value for &#64;container/stuck](#valdef-container-stuck-block-start), in § 6.3.2
- [\<boolean-condition\>](#typedef-boolean-condition), in § 3
- [both](#valdef-container-snapped-both), in § 6.3.3
- bottom
  - [value for &#64;container/scrollable](#valdef-container-scrollable-bottom), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-bottom), in § 6.3.5
  - [value for &#64;container/stuck](#valdef-container-stuck-bottom), in § 6.3.2
- [conditional rule chain](#conditional-rule-chain), in § 4
- [conditions](#dom-csscontainerrule-conditions), in § 9.1
- [&#64;container](#at-ruledef-container), in § 5.4
- [container](#propdef-container), in § 5.3
- [\<container-condition\>](#typedef-container-condition), in § 5.4
- [container feature](#container-feature), in § 6
- [\<container-name\>](#typedef-container-name), in § 5.4
- [container-name](#propdef-container-name), in § 5.2
- [containerName](#dom-csscontainerrule-containername), in § 9.1
- [\<container-query\>](#typedef-container-query), in § 5.4
- [container query](#container-query), in § 5.4
- [containerQuery](#dom-csscontainerrule-containerquery), in § 9.1
- [container query length](#container-query-length), in § 7
- [container query length unit](#container-query-length), in § 7
- [container scroll-state query](#container-scroll-state-query), in § 6.3
- [container size query](#container-size-query), in § 6.1
- [container style query](#container-style-query), in § 6.2
- [container-type](#propdef-container-type), in § 5.1
- [CSSContainerCondition](#dictdef-csscontainercondition), in § 9.1
- [CSSContainerRule](#csscontainerrule), in § 9.1
- [CSSSupportsConditionRule](#csssupportsconditionrule), in § 9.2
- [\<custom-ident\>](#valdef-container-name-custom-ident), in § 5.2
- [\<declaration\>](#typedef-declaration), in § 2
- [&#64;else](#at-ruledef-else), in § 4
- [env()](#funcdef-supports-env), in § 2
- [evaluate a \<style-range\>](#evaluate-a-style-range), in § 6.2
- [font-format()](#funcdef-supports-font-format), in § 2
- [font-tech()](#funcdef-supports-font-tech), in § 2
- [height](#descdef-container-height), in § 6.1.2
- inline
  - [value for &#64;container/scrollable](#valdef-container-scrollable-inline), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-inline), in § 6.3.5
  - [value for &#64;container/snapped](#valdef-container-snapped-inline), in § 6.3.3
- inline-end
  - [value for &#64;container/scrollable](#valdef-container-scrollable-inline-end), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-inline-end), in § 6.3.5
  - [value for &#64;container/stuck](#valdef-container-stuck-inline-end), in § 6.3.2
- inline-size
  - [descriptor for &#64;container](#descdef-container-inline-size), in § 6.1.3
  - [value for container-type](#valdef-container-type-inline-size), in § 5.1
- inline-start
  - [value for &#64;container/scrollable](#valdef-container-scrollable-inline-start), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-inline-start), in § 6.3.5
  - [value for &#64;container/stuck](#valdef-container-stuck-inline-start), in § 6.3.2
- [landscape](#valdef-container-orientation-landscape), in § 6.1.6
- left
  - [value for &#64;container/scrollable](#valdef-container-scrollable-left), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-left), in § 6.3.5
  - [value for &#64;container/stuck](#valdef-container-stuck-left), in § 6.3.2
- [media()](#funcdef-media), in § 3
- name
  - [attribute for CSSSupportsConditionRule](#dom-csssupportsconditionrule-name), in § 9.2
  - [dict-member for CSSContainerCondition](#dom-csscontainercondition-name), in § 9.1
- [named-feature()](#funcdef-supports-named-feature), in § 2
- [named supports condition](#named-supports-condition), in § 8
- none
  - [value for &#64;container/scrollable](#valdef-container-scrollable-none), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-none), in § 6.3.5
  - [value for &#64;container/snapped](#valdef-container-snapped-none), in § 6.3.3
  - [value for &#64;container/stuck](#valdef-container-stuck-none), in § 6.3.2
  - [value for container-name](#valdef-container-name-none), in § 5.2
- [normal](#valdef-container-type-normal), in § 5.1
- [orientation](#descdef-container-orientation), in § 6.1.6
- [portrait](#valdef-container-orientation-portrait), in § 6.1.6
- [query](#dom-csscontainercondition-query), in § 9.1
- [query container](#query-container), in § 5
- [query container name](#query-container-name), in § 5.2
- [\<query-in-parens\>](#typedef-query-in-parens), in § 5.4
- right
  - [value for &#64;container/scrollable](#valdef-container-scrollable-right), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-right), in § 6.3.5
  - [value for &#64;container/stuck](#valdef-container-stuck-right), in § 6.3.2
- [scrollable](#descdef-container-scrollable), in § 6.3.4
- [scrolled](#descdef-container-scrolled), in § 6.3.5
- [scroll-state](#valdef-container-type-scroll-state), in § 5.1
- [\<scroll-state-feature\>](#typedef-scroll-state-feature), in § 6.3
- [scroll-state feature](#scroll-state-feature), in § 6.3
- [\<scroll-state-in-parens\>](#typedef-scroll-state-in-parens), in § 5.4
- [\<scroll-state-query\>](#typedef-scroll-state-query), in § 5.4
- [single-axis-scroll-container](#single-axis-scroll-container), in § 2.1.3
- [size](#valdef-container-type-size), in § 5.1
- [\<size-feature\>](#typedef-size-feature), in § 6.1
- [size features](#size-features), in § 6.1
- [snapped](#descdef-container-snapped), in § 6.3.3
- [stuck](#descdef-container-stuck), in § 6.3.2
- [\<style-feature\>](#typedef-style-feature), in § 5.4
- [\<style-feature-boolean\>](#typedef-style-feature-boolean), in § 5.4
- [\<style-feature-name\>](#typedef-style-feature-name), in § 6.2
- [\<style-feature-plain\>](#typedef-style-feature-plain), in § 5.4
- [style features](#style-features), in § 6.2
- [\<style-feature-value\>](#typedef-style-feature-value), in § 6.2
- [\<style-in-parens\>](#typedef-style-in-parens), in § 5.4
- [\<style-query\>](#typedef-style-query), in § 5.4
- [\<style-range\>](#typedef-style-range), in § 5.4
- [style range](#style-range), in § 6.2
- [\<style-range-value\>](#typedef-style-range-value), in § 5.4
- [support a font format](#dfn-support-font-format), in § 2.1.1
- [support a font tech](#dfn-support-font-tech), in § 2.1.1
- [support a named condition](#dfn-supports-condition-name), in § 2.1.4
- [support a named feature](#dfn-support-named-feature), in § 2.1.3
- [support an at-rule](#dfn-support-at-rule), in § 2.1.2
- [support an environment variable](#dfn-support-env), in § 2.1.5
- [supports()](#funcdef-supports), in § 3
- [\<supports-at-rule-fn\>](#typedef-supports-at-rule-fn), in § 2
- [&#64;supports-condition](#at-ruledef-supports-condition), in § 8
- [\<supports-condition-name\>](#typedef-supports-condition-name), in § 8
- [\<supports-decl\>](#typedef-supports-decl), in § 2
- [\<supports-env-fn\>](#typedef-supports-env-fn), in § 2
- [\<supports-feature\>](#typedef-supports-feature), in § 2
- [\<supports-font-format-fn\>](#typedef-supports-font-format-fn), in § 2
- [\<supports-font-tech-fn\>](#typedef-supports-font-tech-fn), in § 2
- [\<supports-named-feature-fn\>](#typedef-supports-named-feature-fn), in § 2
- [support the at-rule](#dfn-support-at-rule), in § 2.1.2
- [support the environment variable](#dfn-support-env), in § 2.1.5
- [support the font format](#dfn-support-font-format), in § 2.1.1
- [support the font tech](#dfn-support-font-tech), in § 2.1.1
- [support the named condition](#dfn-supports-condition-name), in § 2.1.4
- [support the named feature](#dfn-support-named-feature), in § 2.1.3
- top
  - [value for &#64;container/scrollable](#valdef-container-scrollable-top), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-top), in § 6.3.5
  - [value for &#64;container/stuck](#valdef-container-stuck-top), in § 6.3.2
- [&#64;when](#at-ruledef-when), in § 3
- [width](#descdef-container-width), in § 6.1.1
- x
  - [value for &#64;container/scrollable](#valdef-container-scrollable-x), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-x), in § 6.3.5
  - [value for &#64;container/snapped](#valdef-container-snapped-x), in § 6.3.3
- y
  - [value for &#64;container/scrollable](#valdef-container-scrollable-y), in § 6.3.4
  - [value for &#64;container/scrolled](#valdef-container-scrolled-y), in § 6.3.5
  - [value for &#64;container/snapped](#valdef-container-snapped-y), in § 6.3.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference[](#index-defined-elsewhere)

- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="7177b17d"></a>&#64;keyframes
- \[CSS-BOX-4\] defines the following terms:
  - <a id="f72f5cb4"></a>content box
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="3eef835e"></a>&#64;import
  - <a id="08b3934a"></a>&#64;layer
  - <a id="66620709"></a>cascade-dependent keyword
  - <a id="8c8e51b4"></a>computed value
  - <a id="6b448e93"></a>initial value
  - <a id="8f27be0f"></a>longhand property
  - <a id="81a52293"></a>property
  - <a id="b45bd8fa"></a>revert
  - <a id="529d0525"></a>revert-layer
  - <a id="e14541aa"></a>shorthand
  - <a id="980ac56a"></a>shorthand property
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="4397147f"></a>&#64;media
  - <a id="a5d6c9d2"></a>&#64;supports
  - <a id="ac8ed0be"></a>CSSConditionRule
  - <a id="f1a41224"></a>conditional group rule
  - <a id="6e0a7174"></a>CSS feature queries
  - <a id="be011220"></a>supports queries
- \[CSS-CONDITIONAL-4\] defines the following terms:
  - <a id="3d20c9fd"></a>\<supports-selector-fn\>
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="cb02fa9d"></a>layout containment box
  - <a id="3e4b15e8"></a>size containment
  - <a id="3dc278d0"></a>style containment
- \[CSS-CONTAIN-3\] defines the following terms:
  - <a id="18b2519e"></a>inline-size containment
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="57aa8824"></a>independent formatting context
  - <a id="7a605ac8"></a>principal box
- \[CSS-ENV-1\] defines the following terms:
  - <a id="e698a0b8"></a>environment variable
- \[CSS-EXTENSIONS-1\] defines the following terms:
  - <a id="e9f46f56"></a>\<extension-name\>
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="a4687703"></a>\<font-format\>
  - <a id="b89378df"></a>\<font-tech\>
  - <a id="297dfe3a"></a>font-size
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="b24ce65e"></a>&#64;font-face
- \[CSS-NAMESPACES-3\] defines the following terms:
  - <a id="bf0e8186"></a>&#64;namespace
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="d9b4880c"></a>clip
  - <a id="d9cfd0c5"></a>hidden
  - <a id="a3cabdb1"></a>scroll container
  - <a id="f870bd7d"></a>scrollable axis
  - <a id="e3488cd0"></a>scrollable overflow
  - <a id="b1f927bc"></a>scrollable overflow rectangle
  - <a id="700ea31d"></a>scrollport
  - <a id="20fb0253"></a>single-axis scroll container
  - <a id="56c1c575"></a>unreachable scrollable overflow region
- \[CSS-OVERFLOW-5\] defines the following terms:
  - <a id="fb057ee7"></a>::scroll-marker-group
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="804439eb"></a>sticky
  - <a id="0363c82a"></a>sticky position
  - <a id="01926727"></a>sticky view rectangle
- \[CSS-POSITION-4\] defines the following terms:
  - <a id="5d904ee5"></a>::backdrop
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="70503bd6"></a>::after
  - <a id="7c6f51b7"></a>::before
  - <a id="2d843062"></a>::file-selector-button
  - <a id="63b59bd9"></a>::first-letter
  - <a id="4bda66a9"></a>::first-line
  - <a id="b6b63ba4"></a>::marker
  - <a id="4b84fe49"></a>::placeholder
  - <a id="cb456c77"></a>fictional tag sequence
- \[CSS-SCROLL-SNAP-1\] defines the following terms:
  - <a id="a307a40e"></a>relative scroll
  - <a id="552105cb"></a>scroll snap container
- \[CSS-SCROLL-SNAP-2\] defines the following terms:
  - <a id="50b46a3e"></a>scrollsnapchanging
  - <a id="47951dd8"></a>snap target
- \[CSS-SHADOW-1\] defines the following terms:
  - <a id="361a646a"></a>::part()
  - <a id="905d9a5a"></a>::slotted()
  - <a id="4cb3439f"></a>flat tree
  - <a id="4b20c2d8"></a>tree-scoped name
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="842f627a"></a>block size
  - <a id="fde87e72"></a>height
  - <a id="adaebc3f"></a>inline size
  - <a id="d8595fdb"></a>size
  - <a id="88d82030"></a>width
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="1edd7994"></a>\<at-keyword-token\>
  - <a id="aa82cf42"></a>\<block-contents\>
  - <a id="04853566"></a>\<declaration-value\>
  - <a id="a8cb81d7"></a>\<rule-list\>
  - <a id="fb5475b9"></a>&#64;charset
  - <a id="b29fecf5"></a>at-rule
  - <a id="4959fc2e"></a>consume a declaration
  - <a id="e2112c66"></a>declaration
- \[CSS-TRANSITIONS-1\] defines the following terms:
  - <a id="2f6588ea"></a>style change event
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="81b3af3e"></a>!
  - <a id="c297b070"></a>\#
  - <a id="ef9f8297"></a>\*
  - <a id="af4a190d"></a>+
  - <a id="d7e1d67b"></a>\<angle\>
  - <a id="a0144f62"></a>\<custom-ident\>
  - <a id="c2268b97"></a>\<frequency\>
  - <a id="dcecfc13"></a>\<ident\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="ee68e69a"></a>\<ratio\>
  - <a id="9108b09d"></a>\<resolution\>
  - <a id="1d798932"></a>\<string\>
  - <a id="7aaa7c88"></a>\<time\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="eefce2af"></a>em
  - <a id="ce7ea0de"></a>identifier
  - <a id="5d6143d2"></a>relative length
  - <a id="a959f189"></a>small viewport size
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-VALUES-5\] defines the following terms:
  - <a id="4ae03ef7"></a>arbitrary substitution function
- \[CSS-VARIABLES-1\] defines the following terms:
  - <a id="310b3140"></a>\<custom-property-name\>
  - <a id="5550667d"></a>custom property
  - <a id="3beec8c9"></a>var()
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="83d2ef35"></a>block-end
  - <a id="1118d052"></a>block-start
  - <a id="a6eb24bb"></a>inline axis
  - <a id="4da3b716"></a>inline-end
  - <a id="0da67e16"></a>inline-start
- \[CSS2\] defines the following terms:
  - <a id="fb01f680"></a>line-height
- \[CSSOM-1\] defines the following terms:
  - <a id="697c30aa"></a>CSSGroupingRule
  - <a id="9d357000"></a>CSSOMString
  - <a id="b251d8a9"></a>serialize an identifier
  - <a id="79414121"></a>supported CSS property
- \[CSSOM-VIEW-1\] defines the following terms:
  - <a id="3a71fc5d"></a>MediaQueryList
  - <a id="fa5fd853"></a>matchMedia(query)
  - <a id="279b2092"></a>run snapshot post-layout state steps
- \[INFRA\] defines the following terms:
  - <a id="53275e46"></a>append
  - <a id="16d07e10"></a>for each
  - <a id="649608b9"></a>list
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="b9b36bfc"></a>\<general-enclosed\>
  - <a id="2ae11dd2"></a>\<mf-boolean\>
  - <a id="64e5a7f2"></a>\<mf-comparison\>
  - <a id="ad2ab4e6"></a>\<mf-eq\>
  - <a id="44194fb2"></a>\<mf-gt\>
  - <a id="9db9ed47"></a>\<mf-lt\>
  - <a id="e518251c"></a>\<mf-plain\>
  - <a id="f92b7f82"></a>\<mf-range\>
  - <a id="fe1a9628"></a>boolean context
  - <a id="a8e43bb7"></a>false
  - <a id="0a000463"></a>media feature
  - <a id="3ea2fcbb"></a>media query
  - <a id="8a490d77"></a>not
  - <a id="0e4f830d"></a>true
- \[SELECTORS-4\] defines the following terms:
  - <a id="7b5d8638"></a>originating element
  - <a id="e8cb0e54"></a>pseudo-elements
- \[WEB-ANIMATIONS-1\] defines the following terms:
  - <a id="61189f1f"></a>effect value
- \[WEBIDL\] defines the following terms:
  - <a id="889e932f"></a>Exposed
  - <a id="dcf5fafa"></a>FrozenArray

## <a id="references"></a>References[](#references)

### <a id="normative"></a>Normative References[](#normative)

<a id="biblio-css-animations-1"></a><strong>\[CSS-ANIMATIONS-1\]</strong>

David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: <https://www.w3.org/TR/css-animations-1/>

<a id="biblio-css-box-4"></a><strong>\[CSS-BOX-4\]</strong>

Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: <https://www.w3.org/TR/css-box-4/>

<a id="biblio-css-cascade-4"></a><strong>\[CSS-CASCADE-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 13 January 2022. CR. URL: <https://www.w3.org/TR/css-cascade-4/>

<a id="biblio-css-cascade-5"></a><strong>\[CSS-CASCADE-5\]</strong>

Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: <https://www.w3.org/TR/css-cascade-5/>

<a id="biblio-css-conditional-3"></a><strong>\[CSS-CONDITIONAL-3\]</strong>

Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 15 August 2024. CRD. URL: <https://www.w3.org/TR/css-conditional-3/>

<a id="biblio-css-conditional-4"></a><strong>\[CSS-CONDITIONAL-4\]</strong>

Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 4](https://www.w3.org/TR/css-conditional-4/). 4 September 2025. CRD. URL: <https://www.w3.org/TR/css-conditional-4/>

<a id="biblio-css-contain-2"></a><strong>\[CSS-CONTAIN-2\]</strong>

Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: <https://www.w3.org/TR/css-contain-2/>

<a id="biblio-css-contain-3"></a><strong>\[CSS-CONTAIN-3\]</strong>

Tab Atkins Jr.; Florian Rivoal; Miriam Suzanne. [CSS Containment Module Level 3](https://www.w3.org/TR/css-contain-3/). 18 August 2022. WD. URL: <https://www.w3.org/TR/css-contain-3/>

<a id="biblio-css-display-4"></a><strong>\[CSS-DISPLAY-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 6 November 2025. WD. URL: <https://www.w3.org/TR/css-display-4/>

<a id="biblio-css-env-1"></a><strong>\[CSS-ENV-1\]</strong>

[CSS Environment Variables Module Level 1](https://www.w3.org/TR/css-env-1/). 23 September 2025. FPWD. URL: <https://www.w3.org/TR/css-env-1/>

<a id="biblio-css-extensions-1"></a><strong>\[CSS-EXTENSIONS-1\]</strong>

[CSS Extensions Module Level 1](https://drafts.csswg.org/css-extensions-1/). Editor's Draft. URL: <https://drafts.csswg.org/css-extensions-1/>

<a id="biblio-css-fonts-4"></a><strong>\[CSS-FONTS-4\]</strong>

Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 13 September 2026. WD. URL: <https://www.w3.org/TR/css-fonts-4/>

<a id="biblio-css-fonts-5"></a><strong>\[CSS-FONTS-5\]</strong>

Chris Lilley. [CSS Fonts Module Level 5](https://www.w3.org/TR/css-fonts-5/). 13 September 2026. WD. URL: <https://www.w3.org/TR/css-fonts-5/>

<a id="biblio-css-namespaces-3"></a><strong>\[CSS-NAMESPACES-3\]</strong>

Elika Etemad. [CSS Namespaces Module Level 3](https://www.w3.org/TR/css-namespaces-3/). 20 March 2014. REC. URL: <https://www.w3.org/TR/css-namespaces-3/>

<a id="biblio-css-overflow-3"></a><strong>\[CSS-OVERFLOW-3\]</strong>

Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 7 October 2025. WD. URL: <https://www.w3.org/TR/css-overflow-3/>

<a id="biblio-css-position-3"></a><strong>\[CSS-POSITION-3\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 7 October 2025. WD. URL: <https://www.w3.org/TR/css-position-3/>

<a id="biblio-css-scroll-snap-1"></a><strong>\[CSS-SCROLL-SNAP-1\]</strong>

Matt Rakow; et al. [CSS Scroll Snap Module Level 1](https://www.w3.org/TR/css-scroll-snap-1/). 11 March 2021. CR. URL: <https://www.w3.org/TR/css-scroll-snap-1/>

<a id="biblio-css-scroll-snap-2"></a><strong>\[CSS-SCROLL-SNAP-2\]</strong>

Elika Etemad; Tab Atkins Jr.; Adam Argyle. [CSS Scroll Snap Module Level 2](https://www.w3.org/TR/css-scroll-snap-2/). 23 July 2024. FPWD. URL: <https://www.w3.org/TR/css-scroll-snap-2/>

<a id="biblio-css-shadow-1"></a><strong>\[CSS-SHADOW-1\]</strong>

[CSS Shadow Module Level 1](https://drafts.csswg.org/css-shadow-1/). Editor's Draft. URL: <https://drafts.csswg.org/css-shadow-1/>

<a id="biblio-css-sizing-3"></a><strong>\[CSS-SIZING-3\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 4 September 2026. WD. URL: <https://www.w3.org/TR/css-sizing-3/>

<a id="biblio-css-syntax-3"></a><strong>\[CSS-SYNTAX-3\]</strong>

Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 1 October 2026. CRD. URL: <https://www.w3.org/TR/css-syntax-3/>

<a id="biblio-css-transitions-1"></a><strong>\[CSS-TRANSITIONS-1\]</strong>

Chris Marrin; et al. [CSS Transitions Module Level 1](https://www.w3.org/TR/css-transitions-1/). 8 January 2026. WD. URL: <https://www.w3.org/TR/css-transitions-1/>

<a id="biblio-css-values-4"></a><strong>\[CSS-VALUES-4\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: <https://www.w3.org/TR/css-values-4/>

<a id="biblio-css-values-5"></a><strong>\[CSS-VALUES-5\]</strong>

Tab Atkins Jr.; Elika Etemad; Miriam Suzanne. [CSS Values and Units Module Level 5](https://www.w3.org/TR/css-values-5/). 11 November 2024. WD. URL: <https://www.w3.org/TR/css-values-5/>

<a id="biblio-css-variables-1"></a><strong>\[CSS-VARIABLES-1\]</strong>

Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 16 June 2022. CR. URL: <https://www.w3.org/TR/css-variables-1/>

<a id="biblio-css-writing-modes-4"></a><strong>\[CSS-WRITING-MODES-4\]</strong>

Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: <https://www.w3.org/TR/css-writing-modes-4/>

<a id="biblio-cssom-1"></a><strong>\[CSSOM-1\]</strong>

Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: <https://www.w3.org/TR/cssom-1/>

<a id="biblio-cssom-view-1"></a><strong>\[CSSOM-VIEW-1\]</strong>

Simon Fraser; Emilio Cobos Álvarez. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 16 September 2025. WD. URL: <https://www.w3.org/TR/cssom-view-1/>

<a id="biblio-infra"></a><strong>\[INFRA\]</strong>

Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: <https://infra.spec.whatwg.org/>

<a id="biblio-mediaqueries-4"></a><strong>\[MEDIAQUERIES-4\]</strong>

Tab Atkins Jr.; Florian Rivoal. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 19 February 2026. CRD. URL: <https://www.w3.org/TR/mediaqueries-4/>

<a id="biblio-mediaqueries-5"></a><strong>\[MEDIAQUERIES-5\]</strong>

Tab Atkins Jr.; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 19 February 2026. WD. URL: <https://www.w3.org/TR/mediaqueries-5/>

<a id="biblio-rfc2119"></a><strong>\[RFC2119\]</strong>

S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: <https://datatracker.ietf.org/doc/html/rfc2119>

<a id="biblio-selectors-4"></a><strong>\[SELECTORS-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 22 January 2026. WD. URL: <https://www.w3.org/TR/selectors-4/>

<a id="biblio-web-animations-1"></a><strong>\[WEB-ANIMATIONS-1\]</strong>

Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 5 June 2023. WD. URL: <https://www.w3.org/TR/web-animations-1/>

<a id="biblio-webidl"></a><strong>\[WEBIDL\]</strong>

Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: <https://webidl.spec.whatwg.org/>

### <a id="informative"></a>Non-Normative References[](#informative)

<a id="biblio-css-overflow-4"></a><strong>\[CSS-OVERFLOW-4\]</strong>

David Baron; Florian Rivoal; Elika Etemad. [CSS Overflow Module Level 4](https://www.w3.org/TR/css-overflow-4/). 21 March 2023. WD. URL: <https://www.w3.org/TR/css-overflow-4/>

<a id="biblio-css-overflow-5"></a><strong>\[CSS-OVERFLOW-5\]</strong>

Elika Etemad; Florian Rivoal; Robert Flack. [CSS Overflow Module Level 5](https://www.w3.org/TR/css-overflow-5/). 17 December 2024. FPWD. URL: <https://www.w3.org/TR/css-overflow-5/>

<a id="biblio-css-position-4"></a><strong>\[CSS-POSITION-4\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 4](https://www.w3.org/TR/css-position-4/). 7 October 2025. WD. URL: <https://www.w3.org/TR/css-position-4/>

<a id="biblio-css-pseudo-4"></a><strong>\[CSS-PSEUDO-4\]</strong>

Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: <https://www.w3.org/TR/css-pseudo-4/>

<a id="biblio-css2"></a><strong>\[CSS2\]</strong>

Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: <https://www.w3.org/TR/CSS2/>

## <a id="property-index"></a>Property Index[](#property-index)

| Name                                                                                                   | Value                                                       | Initial                   | Applies to                | Inh.                      | %ages                     | Anim­ation type            | Canonical order | Com­puted value                                      |
|--------------------------------------------------------------------------------------------------------|-------------------------------------------------------------|---------------------------|---------------------------|---------------------------|---------------------------|---------------------------|-----------------|-----------------------------------------------------|
| <strong><a id="ref-for-propdef-container③"></a>[container](#propdef-container)</strong>                | \<'container-name'\> \[ / \<'container-type'\> \]?          | see individual properties | see individual properties | see individual properties | see individual properties | see individual properties | per grammar     | see individual properties                           |
| <strong><a id="ref-for-propdef-container-name⑤"></a>[container-name](#propdef-container-name)</strong> | none \| \<custom-ident\>+                                   | none                      | all elements              | no                        | n/a                       | not animatable            | per grammar     | the keyword none, or an ordered list of identifiers |
| <strong><a id="ref-for-propdef-container-type⑦"></a>[container-type](#propdef-container-type)</strong> | normal \| \[ \[ size \| inline-size \] \|\| scroll-state \] | normal                    | all elements              | no                        | n/a                       | not animatable            | per grammar     | specified keyword                                   |

### <a id="container-descriptor-table"></a><a id="ref-for-at-ruledef-container①⑥"></a>[&#64;container](#at-ruledef-container) Descriptors[](#container-descriptor-table)

| Name                                                                                                                 | Value                                                                                                                         | Initial | Type     |
|----------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------|---------|----------|
| <strong><a id="ref-for-descdef-container-aspect-ratio②"></a>[aspect-ratio](#descdef-container-aspect-ratio)</strong> | \<ratio\>                                                                                                                     |         | range    |
| <strong><a id="ref-for-descdef-container-block-size②"></a>[block-size](#descdef-container-block-size)</strong>       | \<length\>                                                                                                                    |         | range    |
| <strong><a id="ref-for-descdef-container-height④"></a>[height](#descdef-container-height)</strong>                   | \<length\>                                                                                                                    |         | range    |
| <strong><a id="ref-for-descdef-container-inline-size⑥"></a>[inline-size](#descdef-container-inline-size)</strong>    | \<length\>                                                                                                                    |         | range    |
| <strong><a id="ref-for-descdef-container-orientation③"></a>[orientation](#descdef-container-orientation)</strong>    | portrait \| landscape                                                                                                         |         | discrete |
| <strong><a id="ref-for-descdef-container-scrollable③"></a>[scrollable](#descdef-container-scrollable)</strong>       | none \| top \| right \| bottom \| left \| block-start \| inline-start \| block-end \| inline-end \| x \| y \| block \| inline |         | discrete |
| <strong><a id="ref-for-descdef-container-scrolled②"></a>[scrolled](#descdef-container-scrolled)</strong>             | none \| top \| right \| bottom \| left \| block-start \| inline-start \| block-end \| inline-end \| x \| y \| block \| inline |         | discrete |
| <strong><a id="ref-for-descdef-container-snapped⑦"></a>[snapped](#descdef-container-snapped)</strong>                | none \| x \| y \| block \| inline \| both                                                                                     |         | discrete |
| <strong><a id="ref-for-descdef-container-stuck②"></a>[stuck](#descdef-container-stuck)</strong>                      | none \| top \| right \| bottom \| left \| block-start \| inline-start \| block-end \| inline-end                              |         | discrete |
| <strong><a id="ref-for-descdef-container-width④"></a>[width](#descdef-container-width)</strong>                      | \<length\>                                                                                                                    |         | range    |

## <a id="idl-index"></a>IDL Index[](#idl-index)

``` text
dictionary CSSContainerCondition {
  required CSSOMString name;
  required CSSOMString query;
};

[Exposed=Window]
interface CSSContainerRule : CSSConditionRule {
  readonly attribute CSSOMString containerName;
  readonly attribute CSSOMString containerQuery;
  readonly attribute FrozenArray<CSSContainerCondition> conditions;
};

[Exposed=Window]
interface CSSSupportsConditionRule : CSSGroupingRule {
  readonly attribute CSSOMString name;
};
```

## <a id="issues-index"></a>Issues Index[](#issues-index)

This is currently an early draft of the things that are

<strong>Issue:</strong>

<em>new</em> in level 5. The features in Level 3 and Level 4 are still defined in [\[css-conditional-3\]](#biblio-css-conditional-3) and [\[css-conditional-4\]](#biblio-css-conditional-4) and have not yet been copied here. [↵](#issue-092ad31b)

Define "boolean algebra, with X as leaves" in a generic way in Conditional, so all the conditional rules can reference it directly, rather than having to redefine boolean algebra on their own.

<strong>Issue:</strong>

[↵](#issue-e3cd55e5)

Should we require that only the last

<strong>Issue:</strong>

[&#64;else](#at-ruledef-else) in a chain can have an omitted condition? It’s not uncommon for me, when debugging code, to short-circuit an if-else chain by setting one of them to "true"; I presume that would be similarly useful in CSS? It’s still pretty easy to see you’ve done something wrong if you omit the condition accidentally. [↵](#issue-a37c2e08)

The name of the at-rule is under discussion. Alternatives include

<strong>Issue:</strong>

&#64;supports-query, &#64;supports-test, and &#64;custom-supports. The name should be consistent with the one chosen for custom media queries. [↵](#issue-4ac0bea1)

We should try to remove

<strong>Issue:</strong>

<code>[containerName](#dom-csscontainerrule-containername)</code> and <code>[containerQuery](#dom-csscontainerrule-containerquery)</code>, since they don’t deal with multiple conditions correctly. [↵](#issue-9b61d539)

Container Queries should have a

<strong>Issue:</strong>

<code>matchContainer</code> method. This will be modeled on <code>[matchMedia()](https://www.w3.org/TR/cssom-view-1/#dom-window-matchmedia)</code> and the <code>[MediaQueryList](https://www.w3.org/TR/cssom-view-1/#mediaquerylist)</code> interface, but applied to Elements rather than the Window. When measuring layout sizes, it behaves Similar to <code>resizeObserver</code>, but it provides the additional Container Query syntax and features. [\[Issue \#6205\]](https://github.com/w3c/csswg-drafts/issues/6205) [↵](#issue-2e3d6538)
