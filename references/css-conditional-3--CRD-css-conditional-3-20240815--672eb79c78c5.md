Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Conditional Rules Module Level 3

Source snapshot: https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/

Snapshot SHA-256: 672eb79c78c53b9c92a460ffcfd8718af3c1e38ecf317566d955ffa1df470b35

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Conditional Rules Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-at-ruledef-media"></a>

<a id="ref-for-at-ruledef-supports"></a>

This module contains the features of CSS for conditional processing of parts of style sheets, conditioned on capabilities of the processor or the document the style sheet is being applied to. It includes and extends the functionality of CSS level 2 [\[CSS21\]](#biblio-css21), which builds on CSS level 1 [\[CSS1\]](#biblio-css1). The main extensions compared to level 2 are allowing nesting of certain at-rules inside [@media](#at-ruledef-media), and the addition of the [@supports](#at-ruledef-supports) rule for conditional processing.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20231103/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-conditional” in the title, like this: “\[css-conditional\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-conditional%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/policies/process/20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- The inclusion of @font-face rules and @keyframes rules as allowed within all of the @-rules in this specification is at risk, though only because of the relative rates of advancement of specifications. If this specification is able to advance faster than one or both of the specifications defining those rules, then the inclusion of those rules will move from this specification to the specification defining those rules.
- The addition of support for @-rules inside of conditional grouping rules is at risk; if interoperable implementations are not found, it may be removed to advance the other features in this specification to Proposed Recommendation.

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="introduction"></a>1. Introduction

### <a id="context"></a>1.1. Background

<em>This section is not normative.</em>

<a id="ref-for-at-ruledef-media①"></a>

<a id="ref-for-at-ruledef-import"></a>

<a id="ref-for-the-link-element"></a>

[\[CSS21\]](#biblio-css21) defines one type of conditional group rule, the [@media](#at-ruledef-media) rule, and allows only style rules (not other @-rules) inside of it. The <a id="ref-for-at-ruledef-media②"></a>@media rule provides the ability to have media-specific style sheets, which is also provided by style sheet linking features such as [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) and <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-link-element">link</a></code>. The restrictions on the contents of <a id="ref-for-at-ruledef-media③"></a>@media rules made them less useful; they have forced authors using CSS features involving @-rules in media-specific style sheets to use separate style sheets for each medium.

This specification extends the rules for the contents of conditional group rules to allow other @-rules, which enables authors to combine CSS features involving @-rules with media specific style sheets within a single style sheet.

<a id="ref-for-at-ruledef-supports①"></a>

This specification also defines an additional type of conditional group rule, [@supports](#at-ruledef-supports), to address author and user requirements.

<a id="ref-for-at-ruledef-supports②"></a>

The [@supports](#at-ruledef-supports) rule allows CSS to be conditioned on implementation support for CSS properties and values. This rule makes it much easier for authors to use new CSS features and provide good fallback for implementations that do not support those features. This is particularly important for CSS features that provide new layout mechanisms, and for other cases where a set of related styles needs to be conditioned on property support.

### <a id="placement"></a>1.2. Module Interactions

<a id="ref-for-at-ruledef-media④"></a>

This module replaces and extends the [@media](#at-ruledef-media) rule feature defined in [\[CSS21\]](#biblio-css21) section 7.2.1 and incorporates the modifications previously made non-normatively by [\[MEDIAQUERIES-4\]](#biblio-mediaqueries-4) section 1.

## <a id="processing"></a>2. Processing of conditional group rules

<a id="ref-for-at-rule"></a>

This specification defines some CSS [at-rules](https://www.w3.org/TR/css-syntax-3/#at-rule), called <a id="conditional-group-rule"></a>conditional group rules, that associate a condition with a group of other CSS rules. These different rules allow testing different types of conditions, but share common behavior for how their contents are used when the condition is true and when the condition is false.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-25d00670"></a> For example, this rule:
>
> ```text
> @media print {
>   /* hide navigation controls when printing */
>   #navigation { display: none }
> }
> ```
>
> causes a particular CSS rule (making elements with ID “navigation” be display:none) apply only when the style sheet is used for a print medium.

Each conditional group rule has a condition, which at any time evaluates to true or false. When the condition is true, CSS processors <strong>must</strong> apply the rules inside the group rule as though they were at the group rule’s location; when the condition is false, CSS processors <strong>must not</strong> apply any of rules inside the group rule. The current state of the condition does not affect the CSS object model, in which the contents of the group rule always remain within the group rule.

Tests

If condition is true, rules applied in place.

- [at-supports-001.html](https://wpt.fyi/results/css/css-conditional/at-supports-001.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-001.html)
- [at-media-001.html](https://wpt.fyi/results/css/css-conditional/at-media-001.html) [(live test)](http://wpt.live/css/css-conditional/at-media-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-001.html)

------------------------------------------------------------------------

If condition is false, rules not applied.

- [css-supports-020.xht](https://wpt.fyi/results/css/css-conditional/css-supports-020.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-020.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-020.xht)
- [at-media-002.html](https://wpt.fyi/results/css/css-conditional/at-media-002.html) [(live test)](http://wpt.live/css/css-conditional/at-media-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-002.html)

------------------------------------------------------------------------

CSSOM contains rules regardless of condition.

- [conditional-CSSGroupingRule.html](https://wpt.fyi/results/css/css-conditional/js/conditional-CSSGroupingRule.html) [(live test)](http://wpt.live/css/css-conditional/js/conditional-CSSGroupingRule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/conditional-CSSGroupingRule.html)

------------------------------------------------------------------------

This means that when multiple conditional group rules are nested, a rule inside of both of them applies only when all of the rules' conditions are true.

Tests

Nested rules apply when all conditions are true.

- [at-supports-002.html](https://wpt.fyi/results/css/css-conditional/at-supports-002.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-002.html)
- [at-supports-003.html](https://wpt.fyi/results/css/css-conditional/at-supports-003.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-003.html)
- [at-supports-004.html](https://wpt.fyi/results/css/css-conditional/at-supports-004.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-004.html)
- [at-supports-005.html](https://wpt.fyi/results/css/css-conditional/at-supports-005.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-005.html)
- [at-supports-048.html](https://wpt.fyi/results/css/css-conditional/at-supports-048.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-048.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-048.html)
- [css-supports-025.xht](https://wpt.fyi/results/css/css-conditional/css-supports-025.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-025.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-025.xht)
- [css-supports-026.xht](https://wpt.fyi/results/css/css-conditional/css-supports-026.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-026.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-026.xht)
- [css-supports-046.xht](https://wpt.fyi/results/css/css-conditional/css-supports-046.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-046.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-046.xht)

------------------------------------------------------------------------

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cbc86bba"></a> For example, with this set of nested rules:
>
> ```text
> @media print { /* rule (1) */
>   /* hide navigation controls when printing */
>   #navigation { display: none }
>   @media (max-width: 12cm) { /* rule (2) */
>     /* keep notes in flow when printing to narrow pages */
>     .note { float: none }
>   }
> }
> ```
>
> the condition of the rule marked (1) is true for print media, and the condition of the rule marked (2) is true when the width of the display area (which for print media is the page box) is less than or equal to 12cm. Thus the rule \#navigation { display: none } applies whenever this style sheet is applied to print media, and the rule .note { float: none } is applied only when the style sheet is applied to print media <em>and</em> the width of the page box is less than or equal to 12 centimeters.

When the condition for a conditional group rule changes, CSS processors <strong>must</strong> reflect that the rules now apply or no longer apply, except for properties whose definitions define effects of computed values that persist past the lifetime of that value (such as for some properties in [\[CSS3-TRANSITIONS\]](#biblio-css3-transitions) and [\[CSS3-ANIMATIONS\]](#biblio-css3-animations)).

Tests

Change in condition simultaneous with change in condition application (except per transition/animation rules).

- [at-media-dynamic-001.html](https://wpt.fyi/results/css/css-conditional/at-media-dynamic-001.html) [(live test)](http://wpt.live/css/css-conditional/at-media-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-dynamic-001.html)

------------------------------------------------------------------------

## <a id="contents-of"></a>3.  Contents of conditional group rules

<a id="ref-for-conditional-group-rule"></a>

<a id="ref-for-typedef-rule-list"></a>

<a id="ref-for-at-ruledef-import①"></a>

All [conditional group rules](#conditional-group-rule) are defined to take a [\<rule-list\>](https://www.w3.org/TR/css-syntax-3/#typedef-rule-list) in their block, and accept any rule that is normally allowed at the top-level of a stylesheet, and not otherwise restricted. (For example, an [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) rule must appear at the actual beginning of a stylesheet, and so is not valid inside of another rule.)

Tests

Valid to nest all unrestricted top-level rules.

- [at-supports-content-002.html](https://wpt.fyi/results/css/css-conditional/at-supports-content-002.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-content-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-content-002.html)
- [at-supports-content-003.html](https://wpt.fyi/results/css/css-conditional/at-supports-content-003.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-content-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-content-003.html)
- [at-supports-content-004.html](https://wpt.fyi/results/css/css-conditional/at-supports-content-004.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-content-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-content-004.html)
- [at-media-content-002.html](https://wpt.fyi/results/css/css-conditional/at-media-content-002.html) [(live test)](http://wpt.live/css/css-conditional/at-media-content-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-content-002.html)
- [at-media-content-003.html](https://wpt.fyi/results/css/css-conditional/at-media-content-003.html) [(live test)](http://wpt.live/css/css-conditional/at-media-content-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-content-003.html)
- [at-media-content-004.html](https://wpt.fyi/results/css/css-conditional/at-media-content-004.html) [(live test)](http://wpt.live/css/css-conditional/at-media-content-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-content-004.html)

------------------------------------------------------------------------

<a id="ref-for-typedef-rule-list①"></a>

<a id="ref-for-conditional-group-rule①"></a>

Invalid or unknown rules inside the [\<rule-list\>](https://www.w3.org/TR/css-syntax-3/#typedef-rule-list) must be considered invalid and ignored, but do not invalidate the [conditional group rule](#conditional-group-rule).

Tests

Invalid rules do not invalidate conditional group rule.

- [at-supports-content-001.html](https://wpt.fyi/results/css/css-conditional/at-supports-content-001.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-content-001.html)
- [at-media-content-001.html](https://wpt.fyi/results/css/css-conditional/at-media-content-001.html) [(live test)](http://wpt.live/css/css-conditional/at-media-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-content-001.html)

------------------------------------------------------------------------

<a id="ref-for-conditional-group-rule②"></a>

Any namespace prefixes used in a [conditional group rule](#conditional-group-rule) must have been declared, otherwise they are invalid.

<a id="ref-for-css-qualified-name"></a>

<a id="ref-for-namespace-prefix"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-declared-ns"></a> For example, this rule containing a [CSS qualified name](https://www.w3.org/TR/css-namespaces-3/#css-qualified-name) is valid, because the [namespace prefix](https://www.w3.org/TR/css-namespaces-3/#namespace-prefix) has been bound to a namespace url:
>
> ```text
> @namespace x url(http://www.w3.org/1999/xlink);
> @supports (content: attr(x|href)) {
>   // do something }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-90aa5cfe"></a> For example, to determine whether this rule is valid:
>
> ```text
> @supports (content: attr(n|tooltip)) {
>   // do something }
> ```
>
> The user agent will consult the namespace map to see whether a namespace url exists corresponding to the "n" prefix.

Tests

Validity of namespace prefixes depends on namespace map.

- [at-supports-namespace-001.html](https://wpt.fyi/results/css/css-conditional/at-supports-namespace-001.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-namespace-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-namespace-001.html)
- [at-supports-namespace-002.html](https://wpt.fyi/results/css/css-conditional/at-supports-namespace-002.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-namespace-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-namespace-002.html)

------------------------------------------------------------------------

## <a id="use"></a>4.  Placement of conditional group rules

<a id="ref-for-style-rule"></a>

Conditional group rules are allowed wherever [style rules](https://www.w3.org/TR/css-syntax-3/#style-rule) are allowed (at the top-level of a style sheet, as well as within other conditional group rules). CSS processors <strong>must</strong> process such rules as [described above](#processing).

Tests

Conditional group rules allowed wherever style rules are allowed.

- [at-media-001.html](https://wpt.fyi/results/css/css-conditional/at-media-001.html) [(live test)](http://wpt.live/css/css-conditional/at-media-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-001.html)
- [at-supports-001.html](https://wpt.fyi/results/css/css-conditional/at-supports-001.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-001.html)
- [at-supports-002.html](https://wpt.fyi/results/css/css-conditional/at-supports-002.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-002.html)
- [at-supports-003.html](https://wpt.fyi/results/css/css-conditional/at-supports-003.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-003.html)
- [at-supports-004.html](https://wpt.fyi/results/css/css-conditional/at-supports-004.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-004.html)
- [at-supports-005.html](https://wpt.fyi/results/css/css-conditional/at-supports-005.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-005.html)
- [css-supports-025.xht](https://wpt.fyi/results/css/css-conditional/css-supports-025.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-025.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-025.xht)
- [css-supports-026.xht](https://wpt.fyi/results/css/css-conditional/css-supports-026.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-026.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-026.xht)
- [css-supports-046.xht](https://wpt.fyi/results/css/css-conditional/css-supports-046.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-046.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-046.xht)

------------------------------------------------------------------------

<a id="ref-for-at-rule①"></a>

<a id="ref-for-at-ruledef-charset"></a>

<a id="ref-for-at-ruledef-import②"></a>

<a id="ref-for-at-ruledef-namespace"></a>

<a id="ref-for-css-invalid"></a>

Any [at-rules](https://www.w3.org/TR/css-syntax-3/#at-rule) that are not allowed after a style rule (e.g., [@charset](https://www.w3.org/TR/css-syntax-3/#at-ruledef-charset), [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import), or [@namespace](https://drafts.csswg.org/css-namespaces-3/#at-ruledef-namespace) rules) are also not allowed after a conditional group rule, and are therefore [invalid](https://www.w3.org/TR/css-syntax-3/#css-invalid) when so placed.

Tests

At rules not allowed after style rule invalid after conditional group rule.

- [at-media-003.html](https://wpt.fyi/results/css/css-conditional/at-media-003.html) [(live test)](http://wpt.live/css/css-conditional/at-media-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-003.html)
- [at-supports-045.html](https://wpt.fyi/results/css/css-conditional/at-supports-045.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-045.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-045.html)

------------------------------------------------------------------------

<a id="ref-for-at-ruledef-media⑤"></a>

## <a id="at-media"></a>5.  Media-specific style sheets: the [@media](#at-ruledef-media) rule

The <a id="at-ruledef-media"></a>@media rule is a conditional group rule whose condition is a media query. Its syntax is:

<a id="ref-for-typedef-media-query-list"></a>

<a id="ref-for-typedef-rule-list②"></a>

```text
@media <media-query-list> {
  <rule-list>
}
```
<a id="ref-for-at-ruledef-media⑥"></a>

It consists of the at-keyword [@media](#at-ruledef-media) followed by a (possibly empty) media query list (as defined in [\[MEDIAQUERIES-4\]](#biblio-mediaqueries-4)), followed by a block containing arbitrary rules. The condition of the rule is the result of the media query.

<a id="ref-for-at-ruledef-media⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0eba231f"></a> This [@media](#at-ruledef-media) rule:
>
> ```text
> @media screen and (min-width: 35em),
>        print and (min-width: 40em) {
>   #section_navigation { float: left; width: 10em; }
> }
> ```
>
> has the condition screen and (min-width: 35em), print and (min-width: 40em), which is true for screen displays whose viewport is at least 35 times the initial font size and for print displays whose viewport is at least 40 times the initial font size. When either of these is true, the condition of the rule is true, and the rule \#section_navigation { float: left; width: 10em; } is applied.

Tests

Media rule condition can be empty

- [at-media-whitespace-optional-001.html](https://wpt.fyi/results/css/css-conditional/at-media-whitespace-optional-001.html) [(live test)](http://wpt.live/css/css-conditional/at-media-whitespace-optional-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-whitespace-optional-001.html)

------------------------------------------------------------------------

Media rule condition is media query

- [at-media-001.html](https://wpt.fyi/results/css/css-conditional/at-media-001.html) [(live test)](http://wpt.live/css/css-conditional/at-media-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-001.html)
- [at-media-002.html](https://wpt.fyi/results/css/css-conditional/at-media-002.html) [(live test)](http://wpt.live/css/css-conditional/at-media-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-002.html)

------------------------------------------------------------------------

White space is optional where not required for tokenization.

- [at-media-whitespace-optional-001.html](https://wpt.fyi/results/css/css-conditional/at-media-whitespace-optional-001.html) [(live test)](http://wpt.live/css/css-conditional/at-media-whitespace-optional-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-whitespace-optional-001.html)
- [at-media-whitespace-optional-002.html](https://wpt.fyi/results/css/css-conditional/at-media-whitespace-optional-002.html) [(live test)](http://wpt.live/css/css-conditional/at-media-whitespace-optional-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-media-whitespace-optional-002.html)
- [at-supports-whitespace.html](https://wpt.fyi/results/css/css-conditional/at-supports-whitespace.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-whitespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-whitespace.html)

------------------------------------------------------------------------

<a id="ref-for-at-ruledef-supports③"></a>

## <a id="at-supports"></a>6. Feature queries: the [@supports](#at-ruledef-supports) rule

The <a id="at-ruledef-supports"></a>@supports rule is a conditional group rule whose condition tests whether the user agent supports CSS property:value pairs. Authors can use it to write style sheets that use new features when available but degrade gracefully when those features are not supported. These queries are called <a id="css-feature-queries"></a>CSS feature queries or (colloquially) <a id="supports-queries"></a>supports queries.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: CSS has existing mechanisms for graceful degradation, such as ignoring unsupported properties or values, but these are not always sufficient when large groups of styles need to be tied to the support for certain features, as is the case for use of new layout system features.

<a id="ref-for-at-ruledef-supports④"></a>

<a id="ref-for-typedef-media-condition"></a>

The syntax of the condition in the [@supports](#at-ruledef-supports) rule is similar to that defined for [\<media-condition\>](https://www.w3.org/TR/mediaqueries-5/#typedef-media-condition) in [\[MEDIAQUERIES-4\]](#biblio-mediaqueries-4), but without the "unknown" value logic:

- negation is needed so that the new-feature styles and the fallback styles can be separated (within the forward-compatible grammar’s rules for the syntax of @-rules), and not required to override each other.

- conjunction (and) is needed so that multiple required features can be tested.

- disjunction (or) is needed when there are multiple alternative features for a set of styles, particularly when some of those alternatives are vendor-prefixed properties or values.

<a id="ref-for-at-ruledef-supports⑤"></a>

Therefore, the syntax of the [@supports](#at-ruledef-supports) rule allows testing for property:value pairs, and arbitrary conjunctions (and), disjunctions (or), and negations (not) of them.

<a id="ref-for-at-ruledef-supports⑥"></a>

The syntax of the [@supports](#at-ruledef-supports) rule is:

<a id="ref-for-typedef-supports-condition"></a>

<a id="ref-for-typedef-rule-list③"></a>

```text
@supports <supports-condition> {
  <rule-list>
}
```
<a id="ref-for-typedef-supports-condition①"></a>

with [\<supports-condition\>](#typedef-supports-condition) defined as:

<a id="typedef-supports-condition"></a>

<a id="ref-for-typedef-supports-in-parens"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-typedef-supports-in-parens①"></a>

<a id="ref-for-typedef-supports-in-parens②"></a>

<a id="ref-for-mult-zero-plus"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-typedef-supports-in-parens③"></a>

<a id="ref-for-typedef-supports-in-parens④"></a>

<a id="ref-for-mult-zero-plus①"></a>

<a id="typedef-supports-in-parens"></a>

<a id="ref-for-typedef-supports-condition②"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-typedef-supports-feature"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-typedef-general-enclosed"></a>

<a id="typedef-supports-feature"></a>

<a id="ref-for-typedef-supports-decl"></a>

<a id="typedef-supports-decl"></a>

```text
<supports-condition> = not <supports-in-parens>
                     | <supports-in-parens> [ and <supports-in-parens> ]*
                     | <supports-in-parens> [ or <supports-in-parens> ]*
<supports-in-parens> = ( <supports-condition> ) | <supports-feature> | <general-enclosed>
<supports-feature> = <supports-decl>
<supports-decl> = ( <declaration> )
```
<a id="ref-for-typedef-general-enclosed①"></a>

<a id="ref-for-at-ruledef-supports⑦"></a>

The above grammar is purposely very loose for forwards-compatibility reasons, since the [\<general-enclosed\>](https://drafts.csswg.org/css-values-4/#typedef-general-enclosed) production allows for substantial future extensibility. Any [@supports](#at-ruledef-supports) rule that does not parse according to the grammar above (that is, a rule that does not match this loose grammar which includes the <a id="ref-for-typedef-general-enclosed②"></a>\<general-enclosed\> production) is invalid. Style sheets <strong>must not</strong> use such a rule and processors <strong>must</strong> ignore such a rule (including all of its contents).

Tests

White space is optional where not required for tokenization.

- [css-supports-015.xht](https://wpt.fyi/results/css/css-conditional/css-supports-015.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-015.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-015.xht)

------------------------------------------------------------------------

Grammatically-invalid @supports rule is ignored.

- [at-supports-019.html](https://wpt.fyi/results/css/css-conditional/at-supports-019.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-019.html)
- [at-supports-020.html](https://wpt.fyi/results/css/css-conditional/at-supports-020.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-020.html)
- [at-supports-021.html](https://wpt.fyi/results/css/css-conditional/at-supports-021.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-021.html)
- [at-supports-022.html](https://wpt.fyi/results/css/css-conditional/at-supports-022.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-022.html)
- [at-supports-023.html](https://wpt.fyi/results/css/css-conditional/at-supports-023.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-023.html)
- [at-supports-024.html](https://wpt.fyi/results/css/css-conditional/at-supports-024.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-024.html)
- [at-supports-025.html](https://wpt.fyi/results/css/css-conditional/at-supports-025.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-025.html)
- [at-supports-026.html](https://wpt.fyi/results/css/css-conditional/at-supports-026.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-026.html)
- [at-supports-027.html](https://wpt.fyi/results/css/css-conditional/at-supports-027.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-027.html)
- [at-supports-028.html](https://wpt.fyi/results/css/css-conditional/at-supports-028.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-028.html)
- [at-supports-029.html](https://wpt.fyi/results/css/css-conditional/at-supports-029.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-029.html)
- [at-supports-030.html](https://wpt.fyi/results/css/css-conditional/at-supports-030.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-030.html)
- [at-supports-031.html](https://wpt.fyi/results/css/css-conditional/at-supports-031.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-031.html)
- [at-supports-032.html](https://wpt.fyi/results/css/css-conditional/at-supports-032.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-032.html)
- [at-supports-033.html](https://wpt.fyi/results/css/css-conditional/at-supports-033.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-033.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-033.html)
- [css-supports-034.xht](https://wpt.fyi/results/css/css-conditional/css-supports-034.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-034.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-034.xht)
- [css-supports-037.xht](https://wpt.fyi/results/css/css-conditional/css-supports-037.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-037.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-037.xht)

------------------------------------------------------------------------

Each of these grammar terms is associated with a boolean result, as follows:

<a id="ref-for-typedef-supports-condition③"></a>

[\<supports-condition\>](#typedef-supports-condition)

<a id="ref-for-typedef-supports-in-parens⑤"></a>

[\<supports-in-parens\>](#typedef-supports-in-parens)

The result is the result of the child subexpression.

<a id="ref-for-typedef-supports-in-parens⑥"></a>

not [\<supports-in-parens\>](#typedef-supports-in-parens)

<a id="ref-for-typedef-supports-in-parens⑦"></a>

The result is the negation of the [\<supports-in-parens\>](#typedef-supports-in-parens) term.

Tests

Not negates supports condition.

- [at-supports-009.html](https://wpt.fyi/results/css/css-conditional/at-supports-009.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-009.html)
- [at-supports-010.html](https://wpt.fyi/results/css/css-conditional/at-supports-010.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-010.html)
- [css-supports-016.xht](https://wpt.fyi/results/css/css-conditional/css-supports-016.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-016.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-016.xht)

------------------------------------------------------------------------

Not must be followed by space.

- [at-supports-014.html](https://wpt.fyi/results/css/css-conditional/at-supports-014.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-014.html)
- [css-supports-038.xht](https://wpt.fyi/results/css/css-conditional/css-supports-038.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-038.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-038.xht)

------------------------------------------------------------------------

Not requires parentheses.

- [css-supports-017.xht](https://wpt.fyi/results/css/css-conditional/css-supports-017.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-017.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-017.xht)
- [css-supports-018.xht](https://wpt.fyi/results/css/css-conditional/css-supports-018.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-018.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-018.xht)

------------------------------------------------------------------------

<a id="ref-for-typedef-supports-in-parens⑧"></a>

[\<supports-in-parens\>](#typedef-supports-in-parens) \[ and <a id="ref-for-typedef-supports-in-parens⑨"></a>\<supports-in-parens\> \]\*

<a id="ref-for-typedef-supports-in-parens①⓪"></a>

The result is true if all of the [\<supports-in-parens\>](#typedef-supports-in-parens) child terms are true, and false otherwise.

Tests

And condition is true iff all linked conditions are true.

- [at-supports-007.html](https://wpt.fyi/results/css/css-conditional/at-supports-007.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-007.html)
- [at-supports-010.html](https://wpt.fyi/results/css/css-conditional/at-supports-010.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-010.html)
- [at-supports-012.html](https://wpt.fyi/results/css/css-conditional/at-supports-012.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-012.html)
- [css-supports-008.xht](https://wpt.fyi/results/css/css-conditional/css-supports-008.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-008.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-008.xht)
- [css-supports-009.xht](https://wpt.fyi/results/css/css-conditional/css-supports-009.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-009.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-009.xht)
- [css-supports-010.xht](https://wpt.fyi/results/css/css-conditional/css-supports-010.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-010.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-010.xht)
- [css-supports-012.xht](https://wpt.fyi/results/css/css-conditional/css-supports-012.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-012.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-012.xht)

------------------------------------------------------------------------

And requires parentheses.

- [css-supports-019.xht](https://wpt.fyi/results/css/css-conditional/css-supports-019.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-019.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-019.xht)

------------------------------------------------------------------------

<a id="ref-for-typedef-supports-in-parens①①"></a>

[\<supports-in-parens\>](#typedef-supports-in-parens) \[ or <a id="ref-for-typedef-supports-in-parens①②"></a>\<supports-in-parens\> \]\*

<a id="ref-for-typedef-supports-in-parens①③"></a>

The result is false if all of the [\<supports-in-parens\>](#typedef-supports-in-parens) child terms are false, and true otherwise.

Tests

Or condition is true iff any of linked conditions is true.

- [at-supports-008.html](https://wpt.fyi/results/css/css-conditional/at-supports-008.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-008.html)
- [at-supports-010.html](https://wpt.fyi/results/css/css-conditional/at-supports-010.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-010.html)
- [at-supports-013.html](https://wpt.fyi/results/css/css-conditional/at-supports-013.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-013.html)
- [css-supports-006.xht](https://wpt.fyi/results/css/css-conditional/css-supports-006.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-006.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-006.xht)
- [css-supports-007.xht](https://wpt.fyi/results/css/css-conditional/css-supports-007.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-007.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-007.xht)
- [css-supports-011.xht](https://wpt.fyi/results/css/css-conditional/css-supports-011.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-011.xht)
- [css-supports-021.xht](https://wpt.fyi/results/css/css-conditional/css-supports-021.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-021.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-021.xht)

------------------------------------------------------------------------

Or must be followed by space.

- [at-supports-043.html](https://wpt.fyi/results/css/css-conditional/at-supports-043.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-043.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-043.html)
- [css-supports-039.xht](https://wpt.fyi/results/css/css-conditional/css-supports-039.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-039.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-039.xht)

------------------------------------------------------------------------

Or requires parentheses.

- [css-supports-029.xht](https://wpt.fyi/results/css/css-conditional/css-supports-029.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-029.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-029.xht)
- [css-supports-030.xht](https://wpt.fyi/results/css/css-conditional/css-supports-030.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-030.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-030.xht)

------------------------------------------------------------------------

<a id="ref-for-typedef-supports-decl①"></a>

[\<supports-decl\>](#typedef-supports-decl)

<a id="ref-for-dfn-support"></a>

The result is true if the UA [supports](#dfn-support) the declaration within the parentheses.

Tests

Supports condition is true iff declaration is supported.

- [at-supports-001.html](https://wpt.fyi/results/css/css-conditional/at-supports-001.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-001.html)
- [at-supports-017.html](https://wpt.fyi/results/css/css-conditional/at-supports-017.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-017.html)
- [at-supports-018.html](https://wpt.fyi/results/css/css-conditional/at-supports-018.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-018.html)
- [css-supports-001.xht](https://wpt.fyi/results/css/css-conditional/css-supports-001.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-001.xht)

------------------------------------------------------------------------

Declaration cannot include semicolon.

- [at-supports-038.html](https://wpt.fyi/results/css/css-conditional/at-supports-038.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-038.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-038.html)
- [at-supports-039.html](https://wpt.fyi/results/css/css-conditional/at-supports-039.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-039.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-039.html)

------------------------------------------------------------------------

Declaration value can be empty.

- [css-supports-022.xht](https://wpt.fyi/results/css/css-conditional/css-supports-022.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-022.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-022.xht)

------------------------------------------------------------------------

Declaration cannot include invalid !tokens.

- [css-supports-043.xht](https://wpt.fyi/results/css/css-conditional/css-supports-043.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-043.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-043.xht)
- [css-supports-044.xht](https://wpt.fyi/results/css/css-conditional/css-supports-044.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-044.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-044.xht)
- [css-supports-045.xht](https://wpt.fyi/results/css/css-conditional/css-supports-045.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-045.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-045.xht)

------------------------------------------------------------------------

<a id="ref-for-typedef-general-enclosed③"></a>

[\<general-enclosed\>](https://drafts.csswg.org/css-values-4/#typedef-general-enclosed)

The result is false.

<a id="ref-for-typedef-general-enclosed④"></a>

<a id="ref-for-typedef-supports-condition④"></a>

Authors must not use [\<general-enclosed\>](https://drafts.csswg.org/css-values-4/#typedef-general-enclosed) in their stylesheets. <strong data-conversion-semantic="note">Note:</strong> It exists only for future-compatibility, so that new syntax additions do not invalidate too much of a [\<supports-condition\>](#typedef-supports-condition) in older user agents.

Tests

Unrecognized but grammatically-valid condition is false, not invalid.

- [at-supports-015.html](https://wpt.fyi/results/css/css-conditional/at-supports-015.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-015.html)
- [at-supports-046.html](https://wpt.fyi/results/css/css-conditional/at-supports-046.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-046.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-046.html)
- [css-supports-023.xht](https://wpt.fyi/results/css/css-conditional/css-supports-023.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-023.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-023.xht)
- [css-supports-031.xht](https://wpt.fyi/results/css/css-conditional/css-supports-031.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-031.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-031.xht)
- [css-supports-032.xht](https://wpt.fyi/results/css/css-conditional/css-supports-032.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-032.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-032.xht)
- [css-supports-033.xht](https://wpt.fyi/results/css/css-conditional/css-supports-033.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-033.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-033.xht)
- [css-supports-034.xht](https://wpt.fyi/results/css/css-conditional/css-supports-034.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-034.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-034.xht)
- [css-supports-036.xht](https://wpt.fyi/results/css/css-conditional/css-supports-036.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-036.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-036.xht)
- [css-supports-040.xht](https://wpt.fyi/results/css/css-conditional/css-supports-040.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-040.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-040.xht)
- [css-supports-041.xht](https://wpt.fyi/results/css/css-conditional/css-supports-041.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-041.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-041.xht)
- [css-supports-042.xht](https://wpt.fyi/results/css/css-conditional/css-supports-042.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-042.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-042.xht)
- [css-supports-046.xht](https://wpt.fyi/results/css/css-conditional/css-supports-046.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-046.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-046.xht)

------------------------------------------------------------------------

Brackets/parenthesis must be balanced

- [css-supports-035.xht](https://wpt.fyi/results/css/css-conditional/css-supports-035.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-035.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-035.xht)

------------------------------------------------------------------------

<a id="ref-for-at-ruledef-supports⑧"></a>

<a id="ref-for-typedef-supports-condition⑤"></a>

The condition of the [@supports](#at-ruledef-supports) rule is the result of the [\<supports-condition\>](#typedef-supports-condition) in its prelude.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-85c2765a"></a> For example, the following rule
>
> ```text
> @supports ( display: flex ) {
>   body, #navigation, #content { display: flex; }
>   #navigation { background: blue; color: white; }
>   #article { background: white; color: black; }
> }
> ```
>
> <a id="ref-for-at-ruledef-supports⑨"></a>
>
> <a id="ref-for-propdef-display"></a>
>
> applies the rules inside the [@supports](#at-ruledef-supports) rule only when [display: flex](https://www.w3.org/TR/css-display-3/#propdef-display) is supported.

<a id="ref-for-at-ruledef-supports①⓪"></a>

<a id="ref-for-propdef-display①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-affdf0ec"></a> The following example shows an additional [@supports](#at-ruledef-supports) rule that can be used to provide an alternative for when [display: flex](https://www.w3.org/TR/css-display-3/#propdef-display) is not supported:
>
> ```text
> @supports not ( display: flex ) {
>   body { width: 100%; height: 100%; background: white; color: black; }
>   #navigation { width: 25%; }
>   #article { width: 75%; }
> }
> ```
>
> <a id="ref-for-propdef-width"></a>
>
> Note that the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) declarations may be harmful to the flex-based layout, so it is important that they be present only in the non-flex styles.

<a id="ref-for-propdef-box-shadow"></a>

<a id="ref-for-propdef-border"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-eff3ddab"></a> The following example checks for support for the [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow) property, including checking for support for vendor-prefixed versions of it. When the support is present, it specifies both <a id="ref-for-propdef-box-shadow①"></a>box-shadow (with the prefixed versions) and [border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border) in a way what would cause the box to become invisible were <a id="ref-for-propdef-box-shadow②"></a>box-shadow not supported.
>
> ```text
> .noticebox {
>   border: 1px solid black;
>   padding: 1px;
> }
> @supports ( box-shadow: 0 0 2px black inset ) or
>           ( -moz-box-shadow: 0 0 2px black inset ) or
>           ( -webkit-box-shadow: 0 0 2px black inset ) or
>           ( -o-box-shadow: 0 0 2px black inset ) {
>   .noticebox {
>     -moz-box-shadow: 0 0 2px black inset;
>     -webkit-box-shadow: 0 0 2px black inset;
>     -o-box-shadow: 0 0 2px black inset;
>     box-shadow: 0 0 2px black inset; /* unprefixed last */
>     /* override the rule above the @supports rule */
>     border: none;
>     padding: 2px;
>   }
> }
> ```
To avoid confusion between and and or, the syntax requires that both and and or be specified explicitly (rather than, say, using commas or spaces for one of them). Likewise, to avoid confusion caused by precedence rules, the syntax does not allow and, or, and not operators to be mixed without a layer of parentheses.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9f817e66"></a> For example, the following rule is not valid:
>
> ```text
> @supports (transition-property: color) or
>           (animation-name: foo) and
>           (transform: rotate(10deg)) {
>   /* ... */
> }
> ```
>
> Instead, authors must write one of the following:
>
> ```text
> @supports ((transition-property: color) or
>            (animation-name: foo)) and
>           (transform: rotate(10deg)) {
>   /* ... */
> }
> ```
>
> ```text
> @supports (transition-property: color) or
>           ((animation-name: foo) and
>            (transform: rotate(10deg))) {
>   /* ... */
> }
> ```
Tests

Parentheses are required to mix operators.

- [at-supports-016.html](https://wpt.fyi/results/css/css-conditional/at-supports-016.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-016.html)
- [css-supports-013.xht](https://wpt.fyi/results/css/css-conditional/css-supports-013.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-013.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-013.xht)
- [css-supports-014.xht](https://wpt.fyi/results/css/css-conditional/css-supports-014.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-014.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-014.xht)

------------------------------------------------------------------------

The declaration being tested must always occur within parentheses, when it is the only thing in the expression.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c0c9f2d4"></a> For example, the following rule is not valid:
>
> ```text
> @supports display: flex {
>   /* ... */
> }
> ```
>
> Instead, authors must write:
>
> ```text
> @supports (display: flex) {
>   /* ... */
> }
> ```
Tests

Parentheses are required around declaration test.

- [at-supports-011.html](https://wpt.fyi/results/css/css-conditional/at-supports-011.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-011.html)
- [at-supports-034.html](https://wpt.fyi/results/css/css-conditional/at-supports-034.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-034.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-034.html)
- [at-supports-035.html](https://wpt.fyi/results/css/css-conditional/at-supports-035.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-035.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-035.html)
- [at-supports-036.html](https://wpt.fyi/results/css/css-conditional/at-supports-036.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-036.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-036.html)
- [at-supports-037.html](https://wpt.fyi/results/css/css-conditional/at-supports-037.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-037.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-037.html)
- [css-supports-002.xht](https://wpt.fyi/results/css/css-conditional/css-supports-002.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-002.xht)

------------------------------------------------------------------------

The syntax allows extra parentheses when they are not needed. This flexibility is sometimes useful for authors (for example, when commenting out parts of an expression) and may also be useful for authoring tools.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1ef03172"></a> For example, authors may write:
>
> ```text
> @supports ((display: flex)) {
>   /* ... */
> }
> ```
Tests

Extra parentheses are allowed.

- [at-supports-006.html](https://wpt.fyi/results/css/css-conditional/at-supports-006.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-006.html)
- [css-supports-003.xht](https://wpt.fyi/results/css/css-conditional/css-supports-003.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-003.xht)

------------------------------------------------------------------------

A trailing !important on a declaration being tested is allowed, though it won’t change the validity of the declaration.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-afd082e5"></a> For example, the following rule is valid:
>
> ```text
> @supports (display: flex !important) {
>   /* ... */
> }
> ```
Tests

!important is allowed.

- [at-supports-007.html](https://wpt.fyi/results/css/css-conditional/at-supports-007.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-007.html)
- [css-supports-004.xht](https://wpt.fyi/results/css/css-conditional/css-supports-004.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-004.xht)

------------------------------------------------------------------------

### <a id="support-definition"></a>6.1. Definition of support

For forward-compatibility, [section 4.1.8 (Declarations and properties)](https://www.w3.org/TR/CSS21/syndata.html#declaration) of [\[CSS21\]](#biblio-css21) defines rules for handling invalid properties and values. CSS processors that do not implement or partially implement a specification <strong>must</strong> treat any part of a value that they do not implement, or do not have a usable level of support for, as invalid according to this rule for handling invalid properties and values, and therefore <strong>must</strong> discard the declaration as a parse error.

<a id="ref-for-style-rule①"></a>

A CSS processor is considered to <a id="dfn-support"></a>support a declaration (consisting of a property and value) if it accepts that declaration (rather than discarding it as a parse error) within a [style rule](https://www.w3.org/TR/css-syntax-3/#style-rule). If a processor does not implement, with a usable level of support, both the property and the value given, then it <strong>must not</strong> accept the declaration or claim support for it.

<a id="ref-for-propdef-color"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that properties or values whose support is effectively disabled by user preferences are still considered as supported by this definition. For example, if a user has enabled a high-contrast mode that causes colors to be overridden, the CSS processor is still considered to support the [color](https://www.w3.org/TR/css-color-4/#propdef-color) property even though declarations of the <a id="ref-for-propdef-color①"></a>color property may have no effect. On the other hand, a developer-facing preference whose purpose is to enable or disable support for an experimental CSS feature does affect this definition of support.

<a id="ref-for-at-ruledef-supports①①"></a>

These rules (and the equivalence between them) allow authors to use fallback (either in the [\[CSS1\]](#biblio-css1) sense of declarations that are overridden by later declarations or with the new capabilities provided by the [@supports](#at-ruledef-supports) rule in this specification) that works correctly for the features implemented. This applies especially to compound values; implementations must implement all parts of the value in order to consider the declaration supported, either inside a style rule or in the declaration condition of an <a id="ref-for-at-ruledef-supports①②"></a>@supports rule.

Tests

Supports queries are true iff property declaration (including all values) is parsed/supported.

- [css-supports-005.xht](https://wpt.fyi/results/css/css-conditional/css-supports-005.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-005.xht)
- [css-supports-020.xht](https://wpt.fyi/results/css/css-conditional/css-supports-020.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-020.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-020.xht)
- [css-supports-024.xht](https://wpt.fyi/results/css/css-conditional/css-supports-024.xht) [(live test)](http://wpt.live/css/css-conditional/css-supports-024.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/css-supports-024.xht)
- [CSS-supports-CSSStyleDeclaration.html](https://wpt.fyi/results/css/css-conditional/js/CSS-supports-CSSStyleDeclaration.html) [(live test)](http://wpt.live/css/css-conditional/js/CSS-supports-CSSStyleDeclaration.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/CSS-supports-CSSStyleDeclaration.html)
- [at-supports-044.html](https://wpt.fyi/results/css/css-conditional/at-supports-044.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-044.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-044.html)

------------------------------------------------------------------------

## <a id="apis"></a>7. APIs

Tests

- [idlharness.html](https://wpt.fyi/results/css/css-conditional/idlharness.html) [(live test)](http://wpt.live/css/css-conditional/idlharness.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/idlharness.html)
- [001.html](https://wpt.fyi/results/css/css-conditional/js/001.html) [(live test)](http://wpt.live/css/css-conditional/js/001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/001.html)

### <a id="extensions-to-cssrule-interface"></a>7.1.  Extensions to the `CSSRule` interface

The `CSSRule` interface is extended as follows:

<a id="ref-for-cssrule"></a>

<a id="ref-for-idl-unsigned-short"></a>

<a id="dom-cssrule-supports_rule"></a>

```text
partial interface CSSRule {
    const unsigned short SUPPORTS_RULE = 12;
};
```
Tests

CSSRule.SUPPORTS_RULE = 12

- [conditional-CSSGroupingRule.html](https://wpt.fyi/results/css/css-conditional/js/conditional-CSSGroupingRule.html) [(live test)](http://wpt.live/css/css-conditional/js/conditional-CSSGroupingRule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/conditional-CSSGroupingRule.html)

------------------------------------------------------------------------

### <a id="the-cssconditionrule-interface"></a>7.2.  The `CSSConditionRule` interface

<a id="ref-for-cssconditionrule"></a>

The <code><a href="#cssconditionrule">CSSConditionRule</a></code> interface represents all the “conditional” at-rules, which consist of a condition and a statement block.

<a id="ref-for-Exposed"></a>

<a id="cssconditionrule"></a>

<a id="ref-for-cssgroupingrule"></a>

<a id="ref-for-cssomstring"></a>

<a id="dom-cssconditionrule-conditiontext"></a>

```text
[Exposed=Window]
interface CSSConditionRule : CSSGroupingRule {
    readonly attribute CSSOMString conditionText;
};
```
Tests

CSSConditionRule inherits from CSSGroupingRule.

- [conditional-CSSGroupingRule.html](https://wpt.fyi/results/css/css-conditional/js/conditional-CSSGroupingRule.html) [(live test)](http://wpt.live/css/css-conditional/js/conditional-CSSGroupingRule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/conditional-CSSGroupingRule.html)

------------------------------------------------------------------------

CSSConditionRule has .conditionText attribute.

------------------------------------------------------------------------

`conditionText` of type `CSSOMString`  
The `conditionText` attribute represents the condition of the rule. Since what this condition does varies between the derived interfaces of `CSSConditionRule`, those derived interfaces may specify different behavior for this attribute (see, for example, `CSSMediaRule` below). In the absence of such rule-specific behavior, the following rules apply:

The `conditionText` attribute, on getting, must return the result of serializing the associated condition.

Tests

.conditionText returns serialization of condition.

------------------------------------------------------------------------

### <a id="the-cssmediarule-interface"></a>7.3.  The `CSSMediaRule` interface

<a id="ref-for-cssmediarule"></a>

<a id="ref-for-at-ruledef-media⑧"></a>

The <code><a href="#cssmediarule">CSSMediaRule</a></code> interface represents a [@media](#at-ruledef-media) at-rule:

<a id="ref-for-Exposed①"></a>

<a id="cssmediarule"></a>

<a id="ref-for-cssconditionrule①"></a>

<a id="ref-for-SameObject"></a>

<a id="ref-for-PutForwards"></a>

<a id="ref-for-dom-medialist-mediatext"></a>

<a id="ref-for-medialist"></a>

<a id="dom-cssmediarule-media"></a>

<a id="ref-for-idl-boolean"></a>

<a id="dom-cssmediarule-matches"></a>

```text
[Exposed=Window]
interface CSSMediaRule : CSSConditionRule {
    [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
    readonly attribute boolean matches;
};
```
<a id="ref-for-medialist①"></a>

`media` of type <code><a href="https://www.w3.org/TR/cssom-1/#medialist">MediaList</a></code>, readonly

<a id="ref-for-at-ruledef-media⑨"></a>

<a id="ref-for-medialist②"></a>

The `media` attribute must return a <code><a href="https://www.w3.org/TR/cssom-1/#medialist">MediaList</a></code> object for the list of media queries specified with the [@media](#at-ruledef-media) at-rule.

Tests

.media returns a MediaList matching the @media condition.

------------------------------------------------------------------------

<a id="ref-for-idl-boolean①"></a>

`matches` of type <code><a href="https://webidl.spec.whatwg.org/#idl-boolean">boolean</a></code>, readonly

<a id="ref-for-media-query"></a>

<a id="ref-for-dom-cssmediarule-media"></a>

<a id="ref-for-window"></a>

The `matches` attribute returns true if the rule is in an stylesheet attached to a document whose <code><a href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#window">Window</a></code> matches this rule’s <code><a href="#dom-cssmediarule-media">media</a></code> [media query](https://www.w3.org/TR/mediaqueries-5/#media-query), and returns false otherwise.

Tests

.matches matches the media query, returns boolean.

------------------------------------------------------------------------

`conditionText` of type `CSSOMString` (CSSMediaRule-specific definition for attribute on CSSConditionRule)

The `conditionText` attribute (defined on the `CSSConditionRule` parent rule), on getting, must return the value of `media.mediaText` on the rule.

Tests

Value of CSSMediaRule.conditionText matches value of media.mediaText.

------------------------------------------------------------------------

### <a id="the-csssupportsrule-interface"></a>7.4.  The `CSSSupportsRule` interface

<a id="ref-for-csssupportsrule"></a>

<a id="ref-for-at-ruledef-supports①③"></a>

The <code><a href="#csssupportsrule">CSSSupportsRule</a></code> interface represents a [@supports](#at-ruledef-supports) rule.

<a id="ref-for-Exposed②"></a>

<a id="csssupportsrule"></a>

<a id="ref-for-cssconditionrule②"></a>

<a id="ref-for-idl-boolean②"></a>

<a id="dom-csssupportsrule-matches"></a>

```text
[Exposed=Window]
interface CSSSupportsRule : CSSConditionRule {
  readonly attribute boolean matches;
};
```
<a id="ref-for-idl-boolean③"></a>

`matches` of type <code><a href="https://webidl.spec.whatwg.org/#idl-boolean">boolean</a></code>, readonly

<a id="ref-for-dom-cssconditionrule-conditiontext"></a>

<a id="ref-for-css-feature-queries"></a>

The `matches` attribute returns the evaluation of the [CSS feature query](#css-feature-queries) represented in <code><a href="#dom-cssconditionrule-conditiontext">conditionText</a></code>.

Tests

CSSSupportsRule.matches returns true if matches feature query

------------------------------------------------------------------------

`conditionText` of type `CSSOMString` (CSSSupportsRule-specific definition for attribute on CSSConditionRule)

<a id="ref-for-typedef-general-enclosed⑤"></a>

The `conditionText` attribute (defined on the `CSSConditionRule` parent rule), on getting, must return the condition that was specified, without any logical simplifications, so that the returned condition will evaluate to the same result as the specified condition in any conformant implementation of this specification (including implementations that implement future extensions allowed by the [\<general-enclosed\>](https://drafts.csswg.org/css-values-4/#typedef-general-enclosed) extensibility mechanism in this specification). In other words, token stream simplifications are allowed (such as reducing whitespace to a single space or omitting it in cases where it is known to be optional), but logical simplifications (such as removal of unneeded parentheses, or simplification based on evaluating results) are not allowed.

Tests

CSSSupportsRule.conditionText can have tokenization simplifications.

------------------------------------------------------------------------

CSSSupportsRule.conditionText cannot have other simplifications.

------------------------------------------------------------------------

### <a id="the-css-namespace"></a>7.5.  <a id="the-css-interface"></a>The `CSS` namespace, and the `supports()` function

<a id="ref-for-namespacedef-css"></a>

The <code><a href="https://www.w3.org/TR/cssom-1/#namespacedef-css">CSS</a></code> namespace holds useful CSS-related functions that do not belong elsewhere.

<a id="ref-for-namespacedef-css①"></a>

<a id="ref-for-idl-boolean④"></a>

<a id="dom-css-supports"></a>

<a id="ref-for-cssomstring①"></a>

<a id="dom-css-supports-property-value-property"></a>

<a id="ref-for-cssomstring②"></a>

<a id="dom-css-supports-property-value-value"></a>

<a id="ref-for-idl-boolean⑤"></a>

<a id="dom-css-supports-conditiontext"></a>

<a id="ref-for-cssomstring③"></a>

<a id="dom-css-supports-conditiontext-conditiontext"></a>

```text
partial namespace CSS {
  boolean supports(CSSOMString property, CSSOMString value);
  boolean supports(CSSOMString conditionText);
};
```
`supports(CSSOMString property, CSSOMString value)`, returns `boolean`  
<a id="ref-for-dom-css-supports"></a>

When the <code><a href="#dom-css-supports">supports(property, value)</a></code> method is invoked with two arguments <var>property</var> and <var>value</var>:

1.  <a id="ref-for-ascii-case-insensitive"></a>

    <a id="ref-for-custom-property-name-string"></a>

    <a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>

    If <var>property</var> is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for any defined CSS property that the UA supports, or is a [custom property name string](https://www.w3.org/TR/css-typed-om-1/#custom-property-name-string), and <var>value</var> successfully [parses](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) according to that property’s grammar, return `true`.

2.  Otherwise, return `false`.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: No CSS escape or whitespace processing is performed on the property name, so `CSS.supports(" width", "5px")` will return `false`, as " width" isn’t the name of any property due to the leading space.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: !important flags are not part of property grammars, and will cause <var>value</var> to parse as invalid, just as they would in the value argument to element.style.setProperty().

Tests

CSS.supports(arg1, arg2) evaluates support of property arg1 with value arg2.

------------------------------------------------------------------------

`supports(CSSOMString conditionText)`, returns `boolean`  
<a id="ref-for-dom-css-supports-conditiontext"></a>

When the <code><a href="#dom-css-supports-conditiontext">supports(conditionText)</a></code> method is invoked with a single <var>conditionText</var> argument:

1.  <a id="ref-for-css-parse-something-according-to-a-css-grammar①"></a>

    <a id="ref-for-typedef-supports-condition⑥"></a>

    If <var>conditionText</var>, [parsed](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) and evaluated as a [\<supports-condition\>](#typedef-supports-condition), would return true, return `true`.

2.  <a id="ref-for-css-parse-something-according-to-a-css-grammar②"></a>

    <a id="ref-for-typedef-supports-condition⑦"></a>

    Otherwise, If <var>conditionText</var>, wrapped in parentheses and then [parsed](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) and evaluated as a [\<supports-condition\>](#typedef-supports-condition), would return true, return `true`.

3.  Otherwise, return `false`.

All namespaces in the <var>conditionText</var> argument are considered invalid, just as they are in `document.querySelector("a|b")`.

Tests

CSS.supports(arg1) evaluates supports condition arg1.

------------------------------------------------------------------------

CSS.supports(arg1) implies parentheses.

- [supports-conditionText.html](https://wpt.fyi/results/css/css-conditional/js/supports-conditionText.html) [(live test)](http://wpt.live/css/css-conditional/js/supports-conditionText.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/supports-conditionText.html)

------------------------------------------------------------------------

Tests

- [CSS-supports-L3.html](https://wpt.fyi/results/css/css-conditional/js/CSS-supports-L3.html) [(live test)](http://wpt.live/css/css-conditional/js/CSS-supports-L3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/CSS-supports-L3.html)

## <a id="security"></a>Security Considerations

This spec introduces no new security considerations.

## <a id="privacy"></a>Privacy Considerations

<a id="ref-for-at-ruledef-media①⓪"></a>

<a id="ref-for-at-ruledef-supports①④"></a>

Various features in this specification, associated mainly with the [@media](#at-ruledef-media) rule but also to some degree with the [@supports](#at-ruledef-supports) rule, provide information to Web content about the user’s hardware and software and their configuration and state. Most of the information is provided through the features in [\[MEDIAQUERIES-4\]](#biblio-mediaqueries-4) rather than through the features in this specification. However, the <a id="ref-for-at-ruledef-supports①⑤"></a>@supports rule may provide some additional details about the user’s software and whether it is running with non-default settings that may enable or disable certain features.

Most of this information can also be determined through other APIs. However, the features in this specification are one of the ways this information is exposed on the Web.

This information can also, in aggregate, be used to improve the accuracy of [fingerprinting](https://www.w3.org/2001/tag/doc/unsanctioned-tracking/) of the user.

## <a id="changes"></a>8.  Changes

The following (non-editorial) changes were made to this specification since the [13 January 2022 Candidate Recommendation Snapshot](https://www.w3.org/TR/2022/CR-css-conditional-3-20220113/):

<a id="changes-20220113"></a>

- Clarified that supports() must return false for invalid custom property values
- Fixed Web IDL, "bool" should have been "boolean"
- Clarified that a processor must support both the property and the value ([Issue 8795](https://github.com/w3c/csswg-drafts/issues/8795))
- Added .matches to @media and @supports ([Issue 4240](https://github.com/w3c/csswg-drafts/issues/4240))
- Updated to the new parsing algo names and block production names.
- Removed procedure to set readonly CSSMediaRule.conditionText ([PR 8796](https://github.com/w3c/csswg-drafts/pull/8796))
- Made conditionText readonly.

The following (non-editorial) changes were made to this specification since the [8 December 2020 Candidate Recommendation Snapshot](https://www.w3.org/TR/2020/CR-css-conditional-3-20201208/):

<a id="changes-20201208"></a>

- Clarified that discarding property declarations only applies to style rules, not at-rules

- Clarified that !important is not part of the property grammar

- Split Security and Privacy into separate sections

- <a id="ref-for-at-ruledef-supports①⑥"></a>

  <a id="ref-for-supports-queries"></a>

  <a id="ref-for-css-feature-queries①"></a>

  Defined the terms [CSS feature queries](#css-feature-queries) and [supports queries](#supports-queries) to refer to the conditional syntax of the [@supports](#at-ruledef-supports) rules, to allow better cross-referencing.

- <a id="ref-for-css-feature-queries②"></a>

  Removed the “unknown” value in [CSS feature queries](#css-feature-queries)’ boolean logic, defining unrecognized syntaxes as “false” instead. ([Issue 6175](https://github.com/w3c/csswg-drafts/issues/6175))

- <a id="ref-for-conditional-group-rule③"></a>

  Clarified [placement](#use) of [conditional group rules](#conditional-group-rule). ([Issue 5697](https://github.com/w3c/csswg-drafts/issues/5697))

  > Conditional group rules are allowed <u>wherever style rules are allowed (</u> at the top-level of a style sheet, ~~and inside~~ <u>as well as within other conditional group rules <u>)</u> . CSS processors must process such rules as described above.</u>
  >
  > Any <u>at-</u> rules that are not allowed after a style rule (e.g., @charset , @import , or @namespace rules) are also not allowed after a conditional group rule ~~. Therefore, style sheets must not place such rules after a conditional group rule, and CSS processors must ignore such rules.~~ <u>, and are therefore invalid when so placed.</u>

The following (non-editorial) changes were made to this specification since the [4 April 2013 Candidate Recommendation](https://www.w3.org/TR/2013/CR-css3-conditional-20130404/):

<a id="changes-20130404"></a>

- Clarified that namespaces in conditionText are invalid

- New editors added

- <a id="ref-for-css-parse-something-according-to-a-css-grammar③"></a>

  Added explicit call to [parse](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) rather than "matches the grammar"

- Removed duplicate CSSGroupingRule, which is already defined by CSSOM

- Rewrote the supports() text into algorithm form, to make it easier to express that you pay attention to the syntax of registered custom properties in the supports(prop, val) form.

- Moved the definition of @supports selector to css-conditional-4.

- <a id="ref-for-at-ruledef-supports①⑦"></a>

  [@supports](#at-ruledef-supports)' is no longer at risk.

- Rewrote to use CSS Syntax grammars, not CSS 2.1 grammars

- Changed from CSS Interface to WebIDL-compatible CSS namespace

- <a id="ref-for-valdef-media-not"></a>

  Dropped requirement for spaces around and, or, and [not](https://www.w3.org/TR/mediaqueries-5/#valdef-media-not) keywords for consistency with [Media Queries](https://www.w3.org/TR/css3-mediaqueries/) (which are themselves constrained by compatibility with the output of some CSS minimizers that rely on some of the more arcane aspects of CSS tokenization). Note that white space--or a comment--is still required <em>after</em> these keywords, since without it they and the ensuing opening parenthesis will be tokenized as a function opening token.

- Allowed the `supports()` method to imply parentheses for simple declarations, for consistency with the @import rule’s supports() function.

- Fixed missing semicolons in IDL code.

- Updated links, terminology, and example code in response to changes to other modules.

- Spelling and grammatical corrections

- Added section on privacy and security considerations.

## <a id="acknowledgments"></a>Acknowledgments

Thanks to the ideas and feedback from Tab Atkins, Arthur Barstow, Ben Callahan, Tantek Çelik, Alex Danilo, Elika Etemad, Pascal Germroth, Björn Höhrmann, Paul Irish, Brad Kemper, Anne van Kesteren, Vitor Menezes, Alex Mogilevsky, Chris Moschini, James Nurthen, Simon Pieters, Florian Rivoal, Simon Sapin, Nicholas Shanks, Ben Ward, Zack Weinberg, Estelle Weyl, Boris Zbarsky, and all the rest of the [www-style](https://lists.w3.org/Archives/Public/www-style/) community.

## <a id="w3c-conformance"></a> Conformance

### <a id="w3c-conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="w3c-example"></a>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong> UAs MUST provide an accessible alternative. </strong>

Tests

Tests relating to the content of this specification may be documented in “Tests” blocks like this one. Any such block is non-normative.

------------------------------------------------------------------------

### <a id="w3c-conformance-classes"></a> Conformance classes

Conformance to this specification is defined for three conformance classes:

style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS21/conform.html#style-sheet).

renderer  
A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

authoring tool  
A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="w3c-partial"></a> Partial implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](https://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="w3c-conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

### <a id="w3c-testing"></a> Non-experimental implementations

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

### <a id="w3c-cr-exit-criteria"></a> CR exit criteria

For this specification to be advanced to Proposed Recommendation, there must be at least two independent, interoperable implementations of each feature. Each feature may be implemented by a different set of products, there is no requirement that all features be implemented by a single product. For the purposes of this criterion, we define the following terms:

independent  
each implementation must be developed by a different party and cannot share, reuse, or derive from code used by another qualifying implementation. Sections of code that have no bearing on the implementation of this specification are exempt from this requirement.

interoperable  
passing the respective test case(s) in the official CSS test suite, or, if the implementation is not a Web browser, an equivalent test. Every relevant test in the test suite should have an equivalent test created if such a user agent (UA) is to be used to claim interoperability. In addition if such a UA is to be used to claim interoperability, then there must one or more additional UAs which can also pass those equivalent tests in the same way for the purpose of interoperability. The equivalent tests must be made publicly available for the purposes of peer review.

implementation  
a user agent which:

1.  implements the specification.
2.  is available to the general public. The implementation may be a shipping product or other publicly available version (i.e., beta version, preview release, or "nightly build"). Non-shipping product releases must have implemented the feature(s) for a period of at least one month in order to demonstrate stability.
3.  is not experimental (i.e., a version specifically designed to pass the test suite and is not intended for normal usage going forward).

The specification will remain Candidate Recommendation for at least six months.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [conditional group rule](#conditional-group-rule), in § 2
- [conditionText](#dom-cssconditionrule-conditiontext), in § 7.2
- [CSSConditionRule](#cssconditionrule), in § 7.2
- [CSS feature queries](#css-feature-queries), in § 6
- [CSSMediaRule](#cssmediarule), in § 7.3
- [CSSSupportsRule](#csssupportsrule), in § 7.4
- matches
  - [attribute for CSSMediaRule](#dom-cssmediarule-matches), in § 7.3
  - [attribute for CSSSupportsRule](#dom-csssupportsrule-matches), in § 7.4
- [@media](#at-ruledef-media), in § 5
- [media](#dom-cssmediarule-media), in § 7.3
- [support](#dfn-support), in § 6.1
- [@supports](#at-ruledef-supports), in § 6
- [\<supports-condition\>](#typedef-supports-condition), in § 6
- [supports(conditionText)](#dom-css-supports-conditiontext), in § 7.5
- [\<supports-decl\>](#typedef-supports-decl), in § 6
- [\<supports-feature\>](#typedef-supports-feature), in § 6
- [\<supports-in-parens\>](#typedef-supports-in-parens), in § 6
- [supports(property, value)](#dom-css-supports), in § 7.5
- [supports queries](#supports-queries), in § 6
- [SUPPORTS_RULE](#dom-cssrule-supports_rule), in § 7.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="e1674793"></a>border
  - <a id="c48eaa20"></a>box-shadow
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="3eef835e"></a>@import
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="bcdf9b19"></a>color
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="2ccfe434"></a>display
- \[CSS-NAMESPACES-3\] defines the following terms:
  - <a id="bf0e8186"></a>@namespace
  - <a id="8fba26d6"></a>css qualified name
  - <a id="30d8c21b"></a>namespace prefix
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="49731d1d"></a>width
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="a8cb81d7"></a>\<rule-list\>
  - <a id="fb5475b9"></a>@charset
  - <a id="b29fecf5"></a>at-rule
  - <a id="e7c3b4f7"></a>invalid
  - <a id="67800454"></a>parse
  - <a id="87d90aed"></a>style rule
- \[CSS-TYPED-OM-1\] defines the following terms:
  - <a id="5f5ab972"></a>custom property name string
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="ef9f8297"></a>\*
  - <a id="d32570a7"></a>\<general-enclosed\>
  - <a id="4eb9d37e"></a>\|
- \[CSSOM-1\] defines the following terms:
  - <a id="839bfab4"></a>CSS
  - <a id="697c30aa"></a>CSSGroupingRule
  - <a id="9d357000"></a>CSSOMString
  - <a id="0f78dbdd"></a>CSSRule
  - <a id="0cfd25f1"></a>MediaList
  - <a id="09914701"></a>mediaText
- \[HTML\] defines the following terms:
  - <a id="5d7209e9"></a>Window
  - <a id="6cfa013d"></a>link
- \[INFRA\] defines the following terms:
  - <a id="7f9469b5"></a>ascii case-insensitive
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="7aed8b8c"></a>\<media-condition\>
  - <a id="2d21e6d8"></a>\<media-query-list\>
  - <a id="3ea2fcbb"></a>media query
  - <a id="8a490d77"></a>not
- \[WEBIDL\] defines the following terms:
  - <a id="889e932f"></a>Exposed
  - <a id="21ecf38f"></a>PutForwards
  - <a id="a5c91173"></a>SameObject
  - <a id="5372cca8"></a>boolean
  - <a id="450958f7"></a>unsigned short

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-namespaces-3"></a>\[CSS-NAMESPACES-3\]  
Elika Etemad. [CSS Namespaces Module Level 3](https://www.w3.org/TR/css-namespaces-3/). 20 March 2014. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-namespaces-3&#x2F;](https://www.w3.org/TR/css-namespaces-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-typed-om-1"></a>\[CSS-TYPED-OM-1\]  
Tab Atkins Jr.; François Remy. [CSS Typed OM Level 1](https://www.w3.org/TR/css-typed-om-1/). 21 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-typed-om-1&#x2F;](https://www.w3.org/TR/css-typed-om-1/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css3-animations"></a>\[CSS3-ANIMATIONS\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-mediaqueries-4"></a>\[MEDIAQUERIES-4\]  
Florian Rivoal; Tab Atkins Jr.. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 25 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-4&#x2F;](https://www.w3.org/TR/mediaqueries-4/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Chris Lilley; Tab Atkins Jr.; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 13 February 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css1"></a>\[CSS1\]  
Håkon Wium Lie; Bert Bos. [Cascading Style Sheets, level 1](https://www.w3.org/TR/CSS1/). 13 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS1&#x2F;](https://www.w3.org/TR/CSS1/)

<a id="biblio-css3-transitions"></a>\[CSS3-TRANSITIONS\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

## <a id="idl-index"></a>IDL Index

```text
partial interface CSSRule {
    const unsigned short SUPPORTS_RULE = 12;
};

[Exposed=Window]
interface CSSConditionRule : CSSGroupingRule {
    readonly attribute CSSOMString conditionText;
};

[Exposed=Window]
interface CSSMediaRule : CSSConditionRule {
    [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
    readonly attribute boolean matches;
};

[Exposed=Window]
interface CSSSupportsRule : CSSConditionRule {
  readonly attribute boolean matches;
};

partial namespace CSS {
  boolean supports(CSSOMString property, CSSOMString value);
  boolean supports(CSSOMString conditionText);
};

```