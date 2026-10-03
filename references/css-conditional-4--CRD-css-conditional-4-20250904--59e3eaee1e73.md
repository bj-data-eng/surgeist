Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Conditional Rules Module Level 4](https://www.w3.org/TR/2025/CRD-css-conditional-4-20250904/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Conditional Rules Module Level 4

Source snapshot: https://www.w3.org/TR/2025/CRD-css-conditional-4-20250904/

Snapshot SHA-256: 59e3eaee1e7311d7c1c295ae5f27d30abf64c424d21ed007b74849aec7ab000f

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Conditional Rules Module Level 4

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-supports-queries"></a>

This module contains the features of CSS for conditional processing of parts of style sheets, based on capabilities of the processor or the environment the style sheet is being applied in. It includes and extends the functionality of CSS Conditional 3 [\[css-conditional-3\]](#biblio-css-conditional-3), adding the ability to query support for particular selectors [\[SELECTORS-4\]](#biblio-selectors-4) through the new [selector()](#typedef-supports-selector-fn) notation for [supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries).

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-conditional” in the title, like this: “\[css-conditional\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-conditional%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="introduction"></a>1. Introduction

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-283f0a3c"></a> The features in level 3 are still defined in [\[css-conditional-3\]](#biblio-css-conditional-3) and have not yet been copied here.

<a id="ref-for-at-ruledef-supports"></a>

This level adds extensions to the [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rule to allow testing for supported selectors.

<a id="ref-for-at-ruledef-supports①"></a>

## <a id="at-supports-ext"></a>2.  Extensions to the [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rule

<a id="ref-for-typedef-supports-feature"></a>

This level of the specification extends the [\<supports-feature\>](#typedef-supports-feature) syntax as follows:

<a id="typedef-supports-feature"></a>

<a id="ref-for-typedef-supports-selector-fn"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-typedef-supports-decl"></a>

<a id="typedef-supports-selector-fn"></a>

<a id="ref-for-typedef-complex-selector"></a>

```text
<supports-feature> = <supports-selector-fn> | <supports-decl>
<supports-selector-fn> = selector( <complex-selector> )
```
<a id="ref-for-typedef-supports-selector-fn①"></a>

[\<supports-selector-fn\>](#typedef-supports-selector-fn)

<a id="ref-for-dfn-support-selector"></a>

The result is true if the UA [supports the selector](#dfn-support-selector) provided as an argument to the function.

Tests

- [at-supports-selector-001.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-001.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-001.html)
- [at-supports-selector-002.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-002.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-002.html)
- [at-supports-selector-003.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-003.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-003.html)
- [at-supports-selector-004.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-004.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-004.html)
- [at-supports-selector-details-content-before.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-details-content-before.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-details-content-before.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-details-content-before.html)
- [at-supports-selector-details-content.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-details-content.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-details-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-details-content.html)
- [at-supports-selector-detecting-invalid-in-logical-combinations.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-detecting-invalid-in-logical-combinations.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-detecting-invalid-in-logical-combinations.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-detecting-invalid-in-logical-combinations.html)
- [at-supports-selector-file-selector-button.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-file-selector-button.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-file-selector-button.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-file-selector-button.html)
- [at-supports-selector-placeholder.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-placeholder.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-placeholder.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-placeholder.html)
- [CSS-supports-L4.html](https://wpt.fyi/results/css/css-conditional/js/CSS-supports-L4.html) [(live test)](http://wpt.live/css/css-conditional/js/CSS-supports-L4.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/CSS-supports-L4.html)
- [CSS-supports-selector-detecting-invalid-in-logical-combinations.html](https://wpt.fyi/results/css/css-conditional/js/CSS-supports-selector-detecting-invalid-in-logical-combinations.html) [(live test)](http://wpt.live/css/css-conditional/js/CSS-supports-selector-detecting-invalid-in-logical-combinations.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/CSS-supports-selector-detecting-invalid-in-logical-combinations.html)
- [CSS-supports-details-content-pseudo-parsing.html](https://wpt.fyi/results/css/css-conditional/js/CSS-supports-details-content-pseudo-parsing.html) [(live test)](http://wpt.live/css/css-conditional/js/CSS-supports-details-content-pseudo-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/CSS-supports-details-content-pseudo-parsing.html)

<a id="ref-for-column-combinator"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-selector"></a> This example tests whether the [column combinator](https://www.w3.org/TR/selectors-4/#column-combinator) (\|\|) is supported in selectors, and if so uses it to style particular cells in a table.
>
> ```css
> @supports selector(col || td) {
>   col.selected || td {
>     background: tan;
>   }
> }
> ```
<a id="ref-for-conditional-group-rule"></a>

Any namespace prefixes used in a [conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) must have been declared, otherwise they are invalid [\[css-conditional-3\]](#biblio-css-conditional-3). This includes namespace prefixes inside the selector function.

Tests

- [at-supports-namespace-002.html](https://wpt.fyi/results/css/css-conditional/at-supports-namespace-002.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-namespace-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-namespace-002.html)

<a id="ref-for-css-qualified-name"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-ns-selector-invalid"></a> This example tries to check that attribute selectors with [CSS qualified names](https://www.w3.org/TR/css-namespaces-3/#css-qualified-name) are supported, but is invalid, because the namespace prefix has not been declared.
>
> ```css
> @supports selector(a[xlink|href]) {
>   // do something, but fail
> }
> ```
<a id="ref-for-css-qualified-name①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-ns-selector"></a> This example checks that attribute selectors with [CSS qualified names](https://www.w3.org/TR/css-namespaces-3/#css-qualified-name) are supported.
>
> ```css
> @namespace x url(http://www.w3.org/1999/xlink);
> @supports selector(a[x|href]) {
>   // do something
> }
> ```
### <a id="support-definition-ext"></a>2.1.  Extensions to the definition of support

<a id="ref-for-unknown--webkit--pseudo-elements"></a>

A CSS processor is considered to <a id="dfn-support-selector"></a>support a CSS selector if it accepts that all aspects of that selector, recursively, (rather than considering any of its syntax to be unknown or invalid) and that selector doesn’t contain [unknown -webkit- pseudo-elements](https://www.w3.org/TR/selectors-4/#unknown--webkit--pseudo-elements).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some functional selectors are parsed forgivingly, i.e. if some arguments are unknown/invalid, the selector itself is not invalidated. These are nonetheless unsupported

## <a id="security"></a>Security Considerations

No Security issues have been raised against this document

## <a id="privacy"></a>Privacy Considerations

The selector() function may provide information about the user’s software such as its version and whether it is running with non-default settings that enable or disable certain features.

This information can also be determined through other APIs. However, the features in this specification are one of the ways this information is exposed on the Web.

This information can also, in aggregate, be used to improve the accuracy of [fingerprinting](https://www.w3.org/2001/tag/doc/unsanctioned-tracking/) of the user.

## <a id="acknowledgments"></a>Acknowledgments

The editors would like to thank all of the contributors to the [previous level](https://www.w3.org/TR/css-conditional-3/#acknowledgments) of this module.

## <a id="changes"></a> Changes

### <a id="changes-from-2022-02-17"></a> Changes since the [Candidate Recommendation Snapshot of 17 February 2022](https://www.w3.org/TR/2022/CR-css-conditional-4-20220217/)

- Clarify that unknown or invalid portions of a selector that do not invalidate the selector nonetheless cause the selector to be considered unsupported.

### <a id="changes-from-20200303"></a> Changes since the [First Public Working Draft of 3 March 2020](https://www.w3.org/TR/2020/WD-css-conditional-4-20200303/)

- Added [Privacy](#privacy) and [Security](#security) sections.
- Added some examples
- Clarified that the requirement to declare namespace prefixes applies to selectors inside selector() ([Issue 3220](https://github.com/w3c/csswg-drafts/issues/3220))

### <a id="changes-from-L3"></a> Additions since Level 3

- <a id="ref-for-supports-queries①"></a>

  Added selector() notation to [supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries).

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

- [support a CSS selector](#dfn-support-selector), in § 2.1
- [\<supports-feature\>](#typedef-supports-feature), in § 2
- [\<supports-selector-fn\>](#typedef-supports-selector-fn), in § 2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="77b8ca51"></a>\<supports-decl\>
  - <a id="a5d6c9d2"></a>@supports
  - <a id="f1a41224"></a>conditional group rule
  - <a id="be011220"></a>supports queries
- \[CSS-NAMESPACES-3\] defines the following terms:
  - <a id="8fba26d6"></a>CSS qualified name
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="4eb9d37e"></a>\|
- \[SELECTORS-4\] defines the following terms:
  - <a id="2c5d424d"></a>\<complex-selector\>
  - <a id="0a708963"></a>column combinator
  - <a id="586ee9ed"></a>unknown -webkit- pseudo-elements

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 15 August 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

### <a id="informative"></a>Informative References

<a id="biblio-css-namespaces-3"></a>\[CSS-NAMESPACES-3\]  
Elika Etemad. [CSS Namespaces Module Level 3](https://www.w3.org/TR/css-namespaces-3/). 20 March 2014. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-namespaces-3&#x2F;](https://www.w3.org/TR/css-namespaces-3/)

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The features in level 3 are still defined in [\[css-conditional-3\]](#biblio-css-conditional-3) and have not yet been copied here. [↵](#issue-283f0a3c)
