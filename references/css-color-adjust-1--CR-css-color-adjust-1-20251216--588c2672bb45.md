Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Color Adjustment Module Level 1](https://www.w3.org/TR/2025/CR-css-color-adjust-1-20251216/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Color Adjustment Module Level 1

Source snapshot: https://www.w3.org/TR/2025/CR-css-color-adjust-1-20251216/

Snapshot SHA-256: 588c2672bb45911303f78a0590fa587f593234ef80d60b0f7a7af48afc91517c

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 7 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Color Adjustment Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This module introduces a model and controls over automatic color adjustment by the user agent to handle user preferences, such as "Dark Mode", contrast adjustment, or specific desired color schemes.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Snapshot</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Snapshot has received [wide review](https://www.w3.org/policies/process/20250818/#dfn-wide-review), is intended to gather implementation experience, and has commitments from Working Group members to [royalty-free licensing](https://www.w3.org/policies/patent-policy/#sec-Requirements) for implementations. This document is intended to become a W3C Recommendation; it will remain a Candidate Recommendation at least until 16 February 2026 to gather additional feedback.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-color-adjust” in the title, like this: “\[css-color-adjust\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-color-adjust%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1. Introduction

This specification introduces three new features related to controlling how/when colors are auto-adjusted by the user agent:

- <a id="ref-for-color-scheme"></a>

  <a id="ref-for-propdef-color-scheme"></a>

  [Color schemes](#color-scheme) and the [color-scheme](#propdef-color-scheme) property, which controls whether or not browser-provided parts of the page’s UI respect the user’s chosen <a id="ref-for-color-scheme①"></a>color scheme.

- <a id="ref-for-forced-colors-mode"></a>

  <a id="ref-for-propdef-forced-color-adjust"></a>

  [Forced colors mode](#forced-colors-mode) and the [forced-color-adjust](#propdef-forced-color-adjust) property, which controls whether or not <a id="ref-for-forced-colors-mode①"></a>forced colors mode is allowed to apply to a given element.

- <a id="ref-for-propdef-print-color-adjust"></a>

  The [print-color-adjust](#propdef-print-color-adjust) property, which controls whether the browser is allowed to automatically adjust colors to the user’s assumed performance preferences, such as suppressing background colors when printing to save ink.

<a id="ref-for-descdef-media-prefers-color-scheme"></a>

<a id="ref-for-descdef-media-prefers-contrast"></a>

<a id="ref-for-descdef-media-forced-colors"></a>

<a id="ref-for-media-query"></a>

Together with the [prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme), [prefers-contrast](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-contrast), and [forced-colors](https://www.w3.org/TR/mediaqueries-5/#descdef-media-forced-colors) [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query) [\[MEDIAQUERIES-5\]](#biblio-mediaqueries-5), this module allows color scheme negotiation between the author and the user.

### <a id="values"></a>1.1. Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="preferred"></a>2. Preferred Color Schemes

Operating systems and user agents often give users the ability to choose their <a id="preferred-color-scheme"></a>preferred color scheme for user interface elements. This <a id="color-scheme"></a>color scheme is typically reflected in the user agent’s rendering of its navigation interface as well as in-page interface elements such as form controls and scrollbars.

<a id="ref-for-color-scheme②"></a>

A UA can also allow the user to indicate a preference for the [color scheme](#color-scheme) of the pages they view, requesting that the author adapt the page to those color preferences. (It is not required to express such a preference; users can have preferences for operating system interface colors that they do not want imposed on pages.)

<a id="ref-for-color-scheme③"></a>

The most common [color scheme](#color-scheme) preferences are:

- A <a id="light-color-scheme"></a>light color scheme ("day mode") consists of light background colors and dark foreground/text colors.

- A <a id="dark-color-scheme"></a>dark color scheme ("night mode") consists of the opposite, with dark background colors and light foreground/text colors.

<a id="ref-for-light-color-scheme"></a>

<a id="ref-for-dark-color-scheme"></a>

<a id="ref-for-typedef-system-color"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> <strong>The <a href="#light-color-scheme">light</a> and <a href="#dark-color-scheme">dark color schemes</a> don’t represent an exact color palette (such as black-and-white),
	but a range of possible palettes.
	To guarantee specific colors, authors must specify those colors themselves.</strong> Note also that, consequently,
	pairing default or <a href="https://www.w3.org/TR/css-color-4/#typedef-system-color">&lt;system-color&gt;</a> colors with author-specified colors
	cannot guarantee any particular contrast level;
	it might be necessary to set both foreground and background colors together
	to ensure legibility <a href="#biblio-wcag22" title="Web Content Accessibility Guidelines (WCAG) 2.2">&#x5B;WCAG22&#x5D;</a>.</strong>

<a id="ref-for-preferred-color-scheme"></a>

<a id="ref-for-descdef-media-prefers-color-scheme①"></a>

<a id="ref-for-propdef-color-scheme①"></a>

<a id="ref-for-color-scheme④"></a>

To enable pages to adapt to the user’s [preferred color scheme](#preferred-color-scheme), user agents will match the [prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) media query to the user’s <a id="ref-for-preferred-color-scheme①"></a>preferred color scheme. [\[MEDIAQUERIES-5\]](#biblio-mediaqueries-5) Complementing this, the [color-scheme](#propdef-color-scheme) property defined here lets the author indicate appropriate [color schemes](#color-scheme) for UA-provided UI and colors in the page.

<a id="ref-for-color-scheme⑤"></a>

<a id="ref-for-descdef-media-prefers-color-scheme②"></a>

<a id="ref-for-propdef-color-scheme②"></a>

User agents <em>may</em> support additional [color schemes](#color-scheme), however CSS does not support negotiation of additional <a id="ref-for-color-scheme⑥"></a>color schemes: user agents should pursue standardization of these schemes, so that [prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) and [color-scheme](#propdef-color-scheme) can reflect the additional values.

<a id="ref-for-propdef-color-scheme③"></a>

### <a id="color-scheme-prop"></a>2.1. Opting Into a Preferred Color Scheme: the [color-scheme](#propdef-color-scheme) property

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-color-scheme"></a>color-scheme

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-mult-one-plus"></a>

<a id="ref-for-identifier-value"></a>

<a id="ref-for-comb-one"></a>

normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ light <a id="ref-for-comb-one①"></a>\| dark <a id="ref-for-comb-one②"></a>\| [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) \][+](https://www.w3.org/TR/css-values-4/#mult-one-plus) [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) only[?](https://www.w3.org/TR/css-values-4/#mult-opt)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

normal

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

all elements and text

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

yes

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-color-scheme-normal"></a>

the keyword [normal](#valdef-color-scheme-normal), or an ordered list of specified color scheme keywords

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

Tests

- [color-scheme-computed.html](https://wpt.fyi/results/css/css-color-adjust/parsing/color-scheme-computed.html) [(live test)](http://wpt.live/css/css-color-adjust/parsing/color-scheme-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/parsing/color-scheme-computed.html)
- [color-scheme-invalid.html](https://wpt.fyi/results/css/css-color-adjust/parsing/color-scheme-invalid.html) [(live test)](http://wpt.live/css/css-color-adjust/parsing/color-scheme-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/parsing/color-scheme-invalid.html)
- [color-scheme-valid.html](https://wpt.fyi/results/css/css-color-adjust/parsing/color-scheme-valid.html) [(live test)](http://wpt.live/css/css-color-adjust/parsing/color-scheme-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/parsing/color-scheme-valid.html)
- [color-scheme-change-checkbox.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-change-checkbox.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-change-checkbox.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-change-checkbox.html)
- [color-scheme-color-property.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-color-property.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-color-property.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-color-property.html)
- [color-scheme-iframe-background-mismatch-alpha.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-alpha.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-alpha.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-alpha.html)
- [color-scheme-iframe-background-mismatch-opaque-cross-origin.sub.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-opaque-cross-origin.sub.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-opaque-cross-origin.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-opaque-cross-origin.sub.html)
- [color-scheme-iframe-background-mismatch-opaque.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-opaque.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-opaque.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-opaque.html)
- [color-scheme-iframe-background-mismatch-used-preferred.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-used-preferred.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-used-preferred.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background-mismatch-used-preferred.html)
- [color-scheme-iframe-background.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-background.html)
- [color-scheme-iframe-dynamic.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-dynamic.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-dynamic.html)
- [color-scheme-iframe-preferred-change.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred-change.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred-change.html)
- [color-scheme-iframe-preferred-page-dark.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred-page-dark.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred-page-dark.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred-page-dark.html)
- [color-scheme-iframe-preferred-page-light.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred-page-light.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred-page-light.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred-page-light.html)
- [color-scheme-iframe-preferred.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-iframe-preferred.html)
- [color-scheme-link-crash.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-link-crash.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-link-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-link-crash.html)
- [color-scheme-root-background.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-root-background.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-root-background.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-root-background.html)
- [color-scheme-rule-cache.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-rule-cache.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-rule-cache.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-rule-cache.html)
- [color-scheme-system-colors.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-system-colors.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-system-colors.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-system-colors.html)
- [color-scheme-table-border-currentcolor-responsive.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-table-border-currentcolor-responsive.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-table-border-currentcolor-responsive.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-table-border-currentcolor-responsive.html)
- [color-scheme-visited-link-initial.html](https://wpt.fyi/results/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-visited-link-initial.html) [(live test)](http://wpt.live/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-visited-link-initial.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/rendering/dark-color-scheme/color-scheme-visited-link-initial.html)

<a id="ref-for-descdef-media-prefers-color-scheme③"></a>

<a id="ref-for-propdef-color-scheme④"></a>

<a id="ref-for-color-scheme⑦"></a>

While the [prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) media feature allows an author to adapt the page’s colors to the user’s preferred color scheme, many parts of the page are not under the author’s control (such as form controls, scrollbars, etc). The [color-scheme](#propdef-color-scheme) property allows an element to indicate which [color schemes](#color-scheme) it is designed to be rendered with. These values are negotiated with the user’s preferences, resulting in a <a id="used-color-scheme"></a>used color scheme that affects things such as the default colors of form controls and scrollbars. (See [§ 2.2 Effects of the Used Color Scheme](#color-scheme-effect).)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because many pages were authored before color scheme support existed, user agents cannot automatically adapt the colors used in elements under their control, as it might cause unreadable color contrast with the surrounding page.

<a id="ref-for-color-scheme⑧"></a>

Host languages can define the <a id="pages-supported-color-schemes"></a>page’s supported color schemes, a list of [color schemes](#color-scheme) supported by default for all elements on that page.

<a id="color-scheme-meta"></a>

<a id="ref-for-meta"></a>

<a id="ref-for-pages-supported-color-schemes"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\[HTML\]](#biblio-html) specifies a [color-scheme](https://html.spec.whatwg.org/multipage/semantics.html#meta-color-scheme) <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#meta">meta</a></code> tag which can be used to set the [page’s supported color schemes](#pages-supported-color-schemes).

Values are defined as follows:

<a id="valdef-color-scheme-normal"></a>normal

<a id="ref-for-pages-supported-color-schemes①"></a>

<a id="ref-for-color-scheme⑨"></a>

Indicates that the element supports the [page’s supported color schemes](#pages-supported-color-schemes), if they are set, or that it supports no [color schemes](#color-scheme) at all otherwise.

<a id="valdef-color-scheme-light"></a>light

<a id="ref-for-light-color-scheme①"></a>

Indicates that the element supports a [light color scheme](#light-color-scheme).

<a id="valdef-color-scheme-dark"></a>dark

<a id="ref-for-dark-color-scheme①"></a>

Indicates that the element supports a [dark color scheme](#dark-color-scheme).

<a id="valdef-color-scheme-only"></a>only

<a id="ref-for-override-the-color-scheme"></a>

Forbids the user agent from [overriding the color scheme](#override-the-color-scheme) for the element.

<a id="ref-for-identifier-value①"></a>

<a id="valdef-color-scheme-custom-ident"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)

<a id="ref-for-identifier-value②"></a>

<a id="ref-for-propdef-color-scheme⑤"></a>

[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) values are meaningless, and exist only for future compatibility, so that future added color schemes do not invalidate the [color-scheme](#propdef-color-scheme) declaration in legacy user agents. User agents <em>must not</em> interpret any <a id="ref-for-identifier-value③"></a>\<custom-ident\> values as having a meaning; any additional recognized color schemes must be explicitly added to this property’s grammar.

<a id="ref-for-identifier-value④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: To avoid confusion, authoring tutorials and references should omit [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) from their materials.

<a id="ref-for-valdef-color-scheme-normal①"></a>

<a id="ref-for-valdef-color-scheme-light"></a>

<a id="ref-for-valdef-color-scheme-dark"></a>

<a id="ref-for-valdef-color-scheme-only"></a>

<a id="ref-for-identifier-value⑤"></a>

The [normal](#valdef-color-scheme-normal), [light](#valdef-color-scheme-light), [dark](#valdef-color-scheme-dark), and [only](#valdef-color-scheme-only) keywords are not valid [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)s in this property.

<a id="ref-for-light-color-scheme②"></a>

<a id="ref-for-dark-color-scheme②"></a>

<a id="ref-for-color-scheme①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Light](#light-color-scheme) and [dark](#dark-color-scheme) [color schemes](#color-scheme) are not specific color palettes. For example, a stark black-on-white scheme and a sepia dark-on-tan scheme would both be considered <a id="ref-for-light-color-scheme③"></a>light color schemes. To ensure particular foreground or background colors, they need to be specified explicitly.

To <a id="determine-the-used-color-scheme"></a>determine the used color scheme of an element:

1.  <a id="ref-for-preferred-color-scheme②"></a>

    <a id="ref-for-descdef-media-prefers-color-scheme④"></a>

    <a id="ref-for-color-scheme①①"></a>

    <a id="ref-for-used-color-scheme"></a>

    If the user’s [preferred color scheme](#preferred-color-scheme), as indicated by the [prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) media feature, is present among the listed [color schemes](#color-scheme), and is supported by the user agent, that’s the element’s [used color scheme](#used-color-scheme).

2.  <a id="ref-for-valdef-color-scheme-only①"></a>

    <a id="ref-for-propdef-color-scheme⑥"></a>

    <a id="ref-for-override-the-color-scheme①"></a>

    <a id="ref-for-preferred-color-scheme③"></a>

    Otherwise, if the user has indicated an overriding preference for their chosen color scheme, and the [only](#valdef-color-scheme-only) keyword is not present in [color-scheme](#propdef-color-scheme) for the element, the user agent must [override the color scheme](#override-the-color-scheme) with the user’s [preferred color scheme](#preferred-color-scheme). See [§ 2.3 Overriding the Color Scheme](#color-scheme-override).

3.  <a id="ref-for-color-scheme①②"></a>

    <a id="ref-for-used-color-scheme①"></a>

    Otherwise, if the user agent supports at least one of the listed [color schemes](#color-scheme), the [used color scheme](#used-color-scheme) is the first supported <a id="ref-for-color-scheme①③"></a>color scheme in the list.

4.  <a id="ref-for-used-color-scheme②"></a>

    <a id="ref-for-valdef-color-scheme-normal②"></a>

    Otherwise, the [used color scheme](#used-color-scheme) is the browser default. (Same as [normal](#valdef-color-scheme-normal).)

<a id="ref-for-color-scheme①④"></a>

<a id="ref-for-propdef-color-scheme⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: User agents are <strong>not required</strong> to support any particular [color scheme](#color-scheme), so only using a single keyword, such as [color-scheme: dark](#propdef-color-scheme), to indicate a required <a id="ref-for-color-scheme①⑤"></a>color scheme is still not guaranteed to have any effect on the rendering of the element.

<a id="ref-for-descdef-media-prefers-color-scheme⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2227d120"></a> A page that responds to user preferences for light or dark display by using the [prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) media feature to alter the colors it uses can easily opt the browser-controlled UI (scrollbars, inputs, etc) to match with a simple global declaration:
>
> ```text
> :root {
>   color-scheme: light dark;
> }
> ```
>
> <a id="ref-for-typedef-system-color①"></a>
>
> <a id="ref-for-propdef-color-scheme⑧"></a>
>
> <a id="ref-for-at-ruledef-media"></a>
>
> If a page limits itself to using <em>only</em> the [\<system-color\>](https://www.w3.org/TR/css-color-4/#typedef-system-color)s, the [color-scheme](#propdef-color-scheme) declaration, above, will support the user’s preferred color scheme even without the author needing to use [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) at all.

<a id="ref-for-propdef-color-scheme⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-da499c17"></a> If a page cannot reasonably accommodate all color schemes, such as for branding or theatrical reasons, [color-scheme](#propdef-color-scheme) can still indicate which color schemes the page <em>can</em> support, causing the UI to match.
>
> If the page’s color scheme is primarily light, the following will indicate that explicitly:
>
> ```text
> :root {
>   color-scheme: light;
> }
> ```
>
> While if the page is primarily dark, indicating that explicitly will make the page look more coherent as well:
>
> ```text
> :root {
>   color-scheme: dark;
> }
> ```
>
> However, it is better to support both color schemes, of course.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e9ce7c7e"></a> A page might be generally capable of handling multiple color schemes, while still having a sub-section that needs to be rendered in a particular color scheme.
>
> For example, a style guide might give several UI examples that are using light or dark colors, showing off the light or dark theme specifically. This can be indicated as:
>
> ```text
> :root {
>   color-scheme: light dark;
> }
> 
> .light-theme-example {
>   color-scheme: light;
> }
> 
> .dark-theme-example {
>   color-scheme: dark;
> }
> ```
>
> <a id="ref-for-valdef-color-scheme-light①"></a>
>
> <a id="ref-for-valdef-color-scheme-dark①"></a>
>
> Only the subsections rooted at .light-theme-example or .dark-theme-example will be opted into the [light](#valdef-color-scheme-light) or [dark](#valdef-color-scheme-dark) themes specifically; the rest of the page will respect the user’s preference.

<a id="ref-for-propdef-color-scheme①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Repeating a keyword, such as [color-scheme: light light](#propdef-color-scheme), is valid but has no additional effect beyond what the first instance of the keyword provides.

### <a id="color-scheme-effect"></a>2.2. Effects of the Used Color Scheme

<a id="ref-for-used-color-scheme③"></a>

For all elements, the user agent must match the following to the [used color scheme](#used-color-scheme):

- the default colors of scrollbars and other interaction UI

- the default colors of form controls and other "specially-rendered" elements

- the default colors of other browser-provided UI, such as "spellcheck" underlines

<a id="ref-for-used-color-scheme④"></a>

<a id="ref-for-canvas"></a>

On the root element, the [used color scheme](#used-color-scheme) additionally must affect the surface color of the [canvas](https://www.w3.org/TR/CSS2/intro.html#canvas), and the viewport’s scrollbars.

<a id="ref-for-canvas①"></a>

<a id="ref-for-the-iframe-element"></a>

<a id="ref-for-used-color-scheme⑤"></a>

<a id="ref-for-valdef-color-canvas"></a>

<a id="ref-for-the-img-element"></a>

In order to preserve expected color contrasts, in the case of embedded documents typically rendered over a transparent [canvas](https://www.w3.org/TR/CSS2/intro.html#canvas) (such as provided via an HTML <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> element), if the [used color scheme](#used-color-scheme) of the element and the <a id="ref-for-used-color-scheme⑥"></a>used color scheme of the embedded document’s root element do not match, then the UA must use an opaque <a id="ref-for-canvas②"></a>canvas of the [Canvas](https://www.w3.org/TR/css-color-4/#valdef-color-canvas) color appropriate to the embedded document’s <a id="ref-for-used-color-scheme⑦"></a>used color scheme instead of a transparent canvas. This rule does not apply to documents embedded via elements intended for graphics (such as <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> elements embedding an SVG document).

<a id="ref-for-color-scheme①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Aside from the small list of adjustments given above, user agents generally do not further adjust a page to match the user’s preferred [color scheme](#color-scheme), because the chance of accidentally ruining a page is too high. However, when particular color choices are required by the user (for accessibility reasons, for example), more invasive changes might be applied; see [§ 3 Forced Color Palettes](#forced).

### <a id="color-scheme-override"></a>2.3. Overriding the Color Scheme

<a id="ref-for-valdef-color-scheme-only②"></a>

<a id="ref-for-used-color-scheme⑧"></a>

<a id="ref-for-preferred-color-scheme④"></a>

<a id="ref-for-color-scheme①⑦"></a>

If the user has indicated an <em>overriding</em> preference for a particular color scheme, and the author has not disallowed this (by using the [only](#valdef-color-scheme-only) keyword), the user agent may <a id="override-the-color-scheme"></a>override the color scheme, forcing the [used color scheme](#used-color-scheme) to the user’s [preferred color scheme](#preferred-color-scheme). If the element does not support that [color scheme](#color-scheme), the user agent must also auto-adjust other colors into this chosen <a id="ref-for-color-scheme①⑧"></a>color scheme, such as by inverting their brightness, while preserving any color contrast necessary for readability of the page. In this case, UA may also auto-adjust colors within replaced elements, background images, and other external resources as appropriate.

<a id="ref-for-forced-colors-mode②"></a>

<a id="ref-for-dark-color-scheme③"></a>

<a id="ref-for-light-color-scheme④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The specifics of such auto-adjustments are UA-defined, and can differ from UA to UA. But it is not intended to force all colors into a fixed palette, as [forced colors mode](#forced-colors-mode) does, only to force all colors on the page to conform to either a [dark](#dark-color-scheme) or [light](#light-color-scheme) color scheme.

<a id="ref-for-dark-color-scheme④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-11746358"></a> For example, a UA might have a “dark room” mode, which forces all pages into a [dark](#dark-color-scheme) color scheme.
>
> <a id="ref-for-dark-color-scheme⑤"></a>
>
> <a id="ref-for-propdef-color-scheme①①"></a>
>
> <a id="ref-for-meta-color-scheme"></a>
>
> <a id="ref-for-meta①"></a>
>
> <a id="ref-for-valdef-media-prefers-color-scheme-dark"></a>
>
> <a id="ref-for-descdef-media-prefers-color-scheme⑥"></a>
>
> <a id="ref-for-media-query①"></a>
>
> <a id="ref-for-used-color-scheme⑨"></a>
>
> For pages that already support [dark](#dark-color-scheme) color schemes, and have indicated so using the [color-scheme](#propdef-color-scheme) property or <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#meta-color-scheme">color-scheme</a></code> <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#meta">meta</a></code> name, this has no effect other than reporting a [dark](https://www.w3.org/TR/mediaqueries-5/#valdef-media-prefers-color-scheme-dark) value for the [prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) [media query](https://www.w3.org/TR/mediaqueries-5/#media-query) and selecting a <a id="ref-for-dark-color-scheme⑥"></a>dark [used color scheme](#used-color-scheme).
>
> <a id="ref-for-dark-color-scheme⑦"></a>
>
> <a id="ref-for-propdef-color-scheme①②"></a>
>
> But for pages that do not explicitly support a [dark](#dark-color-scheme) color scheme, and have not explicitly forbidden this auto-adjustment by specifying [color-scheme: only light](#propdef-color-scheme), this mode triggers auto-adjustment of the page’s colors to <em>force</em> the page to conform to the desired <a id="ref-for-dark-color-scheme⑧"></a>dark color scheme.

## <a id="forced"></a>3. Forced Color Palettes

<a id="forced-colors-mode"></a>Forced colors mode is an accessibility feature intended to increase the readability of text through color contrast. Individuals with limited vision often find it more comfortable to read content when there is a particular type of contrast between foreground and background colors.

Operating systems can provide built-in color themes, such as Windows’ high contrast black-on-white and high-contrast white-on-black themes. Users can also customize their own themes, for example to provide low contrast or hue contrast.

<a id="ref-for-forced-colors-mode③"></a>

<a id="ref-for-selectordef-selection"></a>

In [forced colors mode](#forced-colors-mode), the user agent enforces the user’s preferred color palette on the page, overriding the author’s chosen colors for specific properties, see [§ 3.1 Properties Affected by Forced Colors Mode](#forced-colors-properties). It may also enforce a “backplate” underneath text (similar to the way backgrounds are painted on the [::selection](https://www.w3.org/TR/css-pseudo-4/#selectordef-selection) pseudo-element) to ensure adequate contrast for readability.

<a id="ref-for-forced-colors-mode④"></a>

<a id="ref-for-descdef-media-forced-colors①"></a>

<a id="ref-for-valdef-color-canvas①"></a>

<a id="ref-for-descdef-media-prefers-color-scheme⑦"></a>

<a id="ref-for-propdef-color-scheme①③"></a>

To enable pages to adapt to [forced colors mode](#forced-colors-mode) user agents will match the [forced-colors](https://www.w3.org/TR/mediaqueries-5/#descdef-media-forced-colors) media query and must provide the required color palette through the CSS system colors (see [\[CSS-COLOR-4\]](#biblio-css-color-4)). Additionally, if the UA determines, based on Lab lightness, that the [Canvas](https://www.w3.org/TR/css-color-4/#valdef-color-canvas) color is clearly either dark (L \< 33%) or light (L \> 67%), then it must match the appropriate value of the [prefers-color-scheme](https://www.w3.org/TR/mediaqueries-5/#descdef-media-prefers-color-scheme) media query and express a corresponding user preference for [color-scheme](#propdef-color-scheme). This will allow pages that support light/dark color schemes to automatically adjust to more closely match the forced color scheme. Behavior between the above dark vs. light thresholds is UA-defined, and may result in assuming either light or dark as the user’s preferred color scheme.

<a id="ref-for-get-emulated-forced-colors-theme-data"></a>

<a id="ref-for-dom-forcedcolorsmodeautomationtheme-none"></a>

<a id="ref-for-forced-colors-mode⑤"></a>

<a id="ref-for-forced-colors-mode-emulation-color-palettes"></a>

<a id="ref-for-emulated-forced-colors-theme-data"></a>

If [get emulated forced colors theme data](#get-emulated-forced-colors-theme-data) is not "<code><a href="#dom-forcedcolorsmodeautomationtheme-none">none</a></code>", the user agent should bypass the above operating system color themes, and instead act as if the user has enabled [forced colors mode](#forced-colors-mode) with the [forced colors mode emulation color palette](#forced-colors-mode-emulation-color-palettes) defined for the resulting value of [emulated forced colors theme data](#emulated-forced-colors-theme-data).

### <a id="forced-colors-properties"></a>3.1. Properties Affected by Forced Colors Mode

<a id="ref-for-forced-colors-mode⑥"></a>

<a id="ref-for-propdef-forced-color-adjust①"></a>

<a id="ref-for-valdef-forced-color-adjust-auto"></a>

<a id="ref-for-typedef-color"></a>

When [forced colors mode](#forced-colors-mode) is active and [forced-color-adjust](#propdef-forced-color-adjust) is [auto](#valdef-forced-color-adjust-auto) (see below) on an element, the [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) components of all properties on the element are force-adjusted to the user’s preferred color palette.

<a id="ref-for-forced-colors-mode⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Specifically, [forced colors mode](#forced-colors-mode) applies to the following color properties, along with their shorthands. This is the list of color properties existing at the time of writing, but it may not be an exhaustive list as more properties get added over time:
>
> - <a id="ref-for-propdef-accent-color"></a>
>
>   [accent-color](https://www.w3.org/TR/css-ui-4/#propdef-accent-color)
>
> - <a id="ref-for-propdef-background-color"></a>
>
>   [background-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color)
>
> - <a id="ref-for-propdef-border-color"></a>
>
>   [border-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-color)
>
> - <a id="ref-for-propdef-caret-color"></a>
>
>   [caret-color](https://www.w3.org/TR/css-ui-4/#propdef-caret-color)
>
> - <a id="ref-for-propdef-color"></a>
>
>   [color](https://www.w3.org/TR/css-color-4/#propdef-color)
>
> - <a id="ref-for-propdef-flood-color"></a>
>
>   [flood-color](https://www.w3.org/TR/filter-effects-1/#propdef-flood-color)
>
> - <a id="ref-for-propdef-fill"></a>
>
>   [fill](https://www.w3.org/TR/fill-stroke-3/#propdef-fill)
>
> - <a id="ref-for-propdef-lighting-color"></a>
>
>   [lighting-color](https://www.w3.org/TR/filter-effects-1/#propdef-lighting-color)
>
> - <a id="ref-for-propdef-outline-color"></a>
>
>   [outline-color](https://www.w3.org/TR/css-ui-4/#propdef-outline-color)
>
> - <a id="ref-for-propdef-rule-color"></a>
>
>   [rule-color](https://www.w3.org/TR/css-gaps-1/#propdef-rule-color)
>
> - <a id="ref-for-propdef-scrollbar-color"></a>
>
>   [scrollbar-color](https://www.w3.org/TR/css-scrollbars-1/#propdef-scrollbar-color)
>
> - <a id="ref-for-StopColorProperty"></a>
>
>   [stop-color](https://www.w3.org/TR/SVG2/pservers.html#StopColorProperty)
>
> - <a id="ref-for-propdef-stroke"></a>
>
>   [stroke](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke)
>
> - <a id="ref-for-propdef-text-decoration-color"></a>
>
>   [text-decoration-color](https://www.w3.org/TR/css-text-decor-4/#propdef-text-decoration-color)
>
> - <a id="ref-for-propdef-text-emphasis-color"></a>
>
>   [text-emphasis-color](https://www.w3.org/TR/css-text-decor-4/#propdef-text-emphasis-color)
>
> - -webkit-tap-highlight-color

<a id="ref-for-typedef-color①"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-css-system-colors"></a>

<a id="ref-for-used-value"></a>

For each [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) component of a property, if its [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) is a color other than a [system color](https://drafts.csswg.org/css-color-4/#css-system-colors), its [used value](https://www.w3.org/TR/css-cascade-5/#used-value) is instead forced to a <a id="ref-for-css-system-colors①"></a>system color as follows:

- <a id="ref-for-propdef-background-color①"></a>

  <a id="ref-for-propdef-color①"></a>

  <a id="ref-for-css-system-colors②"></a>

  <a id="ref-for-system-color-pairings"></a>

  <a id="ref-for-valdef-color-canvastext"></a>

  <a id="ref-for-valdef-color-canvas②"></a>

  For [background-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color) in particular, it is forced to the color opposite the [color](https://www.w3.org/TR/css-color-4/#propdef-color) property’s [system color](https://drafts.csswg.org/css-color-4/#css-system-colors) value in the [system color pairings](https://www.w3.org/TR/css-color-4/#system-color-pairings), using [CanvasText](https://www.w3.org/TR/css-color-4/#valdef-color-canvastext) as the opposite of [Canvas](https://www.w3.org/TR/css-color-4/#valdef-color-canvas). However, its alpha channel is taken from the original <a id="ref-for-propdef-background-color②"></a>background-color value so that transparent backgrounds remain transparent.

- <a id="ref-for-css-system-colors③"></a>

  <a id="ref-for-cascade-origin-author"></a>

  In all other cases, the UA determines the appropriate forced [system color](https://drafts.csswg.org/css-color-4/#css-system-colors)—​which should match the color that would result from an empty [author style sheet](https://www.w3.org/TR/css-cascade-5/#cascade-origin-author) whenever all of the element’s affected properties are likewise UA-determined.

  <a id="ref-for-propdef-color②"></a>

  <a id="ref-for-propdef-background-color③"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > UAs need to be careful about inheritance when forcing colors. For example, suppose the UA’s button [color](https://www.w3.org/TR/css-color-4/#propdef-color) and [background-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color) are the opposite of its canvas <a id="ref-for-propdef-color③"></a>color and <a id="ref-for-propdef-background-color④"></a>background-color. Given markup such as
  > ```text
  > <button>Push <em>this</em> button</button>
  > ```
  >
  > <a id="ref-for-the-em-element"></a>
  >
  > <a id="ref-for-the-button-element"></a>
  >
  > <a id="ref-for-forced-colors-mode⑧"></a>
  >
  > <a id="ref-for-propdef-color④"></a>
  >
  > <a id="ref-for-the-button-element①"></a>
  >
  > <a id="ref-for-the-em-element①"></a>
  >
  > <a id="ref-for-the-button-element②"></a>
  >
  > <a id="ref-for-the-em-element②"></a>
  >
  > Normally, <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-em-element">em</a></code> will inherit from <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element">button</a></code>, ensuring its readability. However in [forced colors mode](#forced-colors-mode), the [color](https://www.w3.org/TR/css-color-4/#propdef-color) of both <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element">button</a></code> and <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-em-element">em</a></code> will need to be forced. It’s easy to see that <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element">button</a></code>’s color should be forced to the button color, but <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-em-element">em</a></code> also needs to be forced to the button color; if it were forced to the canvas <a id="ref-for-propdef-color⑤"></a>color like it is everywhere else in the document, its text will be unreadable.

Additionally:

- <a id="ref-for-propdef-box-shadow"></a>

  <a id="ref-for-propdef-text-shadow"></a>

  <a id="ref-for-box-shadow-none"></a>

  [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow) and [text-shadow](https://www.w3.org/TR/css-text-decor-4/#propdef-text-shadow) compute to [none](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-none)

- <a id="ref-for-propdef-background-image"></a>

  <a id="ref-for-valdef-background-image-none"></a>

  <a id="ref-for-funcdef-url"></a>

  [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) computes to [none](https://www.w3.org/TR/css-backgrounds-3/#valdef-background-image-none) unless the original value contains a [url()](https://www.w3.org/TR/css-values-4/#funcdef-url) function

- <a id="ref-for-propdef-color-scheme①④"></a>

  [color-scheme](#propdef-color-scheme) computes to light dark

- <a id="ref-for-propdef-scrollbar-color①"></a>

  <a id="ref-for-valdef-scrollbar-color-auto"></a>

  [scrollbar-color](https://www.w3.org/TR/css-scrollbars-1/#propdef-scrollbar-color) computes to [auto](https://www.w3.org/TR/css-scrollbars-1/#valdef-scrollbar-color-auto)

- <a id="ref-for-propdef-accent-color①"></a>

  <a id="ref-for-valdef-accent-color-auto"></a>

  [accent-color](https://www.w3.org/TR/css-ui-4/#propdef-accent-color) computes to [auto](https://www.w3.org/TR/css-ui-4/#valdef-accent-color-auto)

- <a id="ref-for-propdef-font-variant-emoji"></a>

  <a id="ref-for-valdef-font-variant-emoji-normal"></a>

  <a id="ref-for-valdef-font-variant-emoji-unicode"></a>

  <a id="ref-for-computed-value①"></a>

  <a id="ref-for-valdef-font-variant-emoji-text"></a>

  If [font-variant-emoji](https://www.w3.org/TR/css-fonts-4/#propdef-font-variant-emoji) computes to [normal](https://www.w3.org/TR/css-fonts-4/#valdef-font-variant-emoji-normal) or [unicode](https://www.w3.org/TR/css-fonts-4/#valdef-font-variant-emoji-unicode), UAs should force any emoji on the page to its monochrome variant, if available, by forcing the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of <a id="ref-for-propdef-font-variant-emoji①"></a>font-variant-emoji to [text](https://www.w3.org/TR/css-fonts-4/#valdef-font-variant-emoji-text).

<a id="ref-for-forced-colors-mode⑨"></a>

UAs may further tweak these [forced colors mode](#forced-colors-mode) heuristics to provide better user experience.

<a id="ref-for-funcdef-color-mix"></a>

<a id="ref-for-forced-colors-mode①⓪"></a>

<a id="ref-for-computed-value②"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-css-system-colors④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9e389b59"></a> Authors may still use features such as [color-mix()](https://www.w3.org/TR/css-color-5/#funcdef-color-mix) in [forced colors mode](#forced-colors-mode). In such cases, the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) will behave as it would normally, but the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) will be overridden with an appropriate [system color](https://drafts.csswg.org/css-color-4/#css-system-colors).
>
> ```text
> .example {
>   color: color-mix(in srgb, CanvasText, Canvas);
> }
> ```
>
> <a id="ref-for-computed-value③"></a>
>
> <a id="ref-for-propdef-color⑥"></a>
>
> <a id="ref-for-valdef-color-canvastext①"></a>
>
> <a id="ref-for-valdef-color-canvas③"></a>
>
> <a id="ref-for-css-system-colors⑤"></a>
>
> <a id="ref-for-dom-element-computedstylemap"></a>
>
> The [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) for [color](https://www.w3.org/TR/css-color-4/#propdef-color) will be a 50-50 blend of the [CanvasText](https://www.w3.org/TR/css-color-4/#valdef-color-canvastext) and [Canvas](https://www.w3.org/TR/css-color-4/#valdef-color-canvas) [system colors](https://drafts.csswg.org/css-color-4/#css-system-colors). That value will inherit to descendants and be observable via APIs such as <code><a href="https://www.w3.org/TR/css-typed-om-1/#dom-element-computedstylemap">computedStyleMap()</a></code>.
>
> <a id="ref-for-used-value②"></a>
>
> <a id="ref-for-propdef-color⑦"></a>
>
> <a id="ref-for-valdef-color-canvastext②"></a>
>
> The [used value](https://www.w3.org/TR/css-cascade-5/#used-value) for [color](https://www.w3.org/TR/css-color-4/#propdef-color) will be a system color chosen by the UA, for example [CanvasText](https://www.w3.org/TR/css-color-4/#valdef-color-canvastext).

<a id="ref-for-propdef-forced-color-adjust②"></a>

### <a id="forced-color-adjust-prop"></a>3.2. Opting Out of a Forced Color Palette: the [forced-color-adjust](#propdef-forced-color-adjust) property

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-forced-color-adjust"></a>forced-color-adjust

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one③"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) none <a id="ref-for-comb-one④"></a>\| preserve-parent-color

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

all elements and text

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

yes

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

as specified

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-propdef-forced-color-adjust③"></a>

<a id="ref-for-forced-colors-mode①①"></a>

The [forced-color-adjust](#propdef-forced-color-adjust) property allows authors to opt particular elements out of [forced colors mode](#forced-colors-mode), restoring full control over the colors to CSS. Values have the following meanings:

<a id="valdef-forced-color-adjust-auto"></a>auto  
<a id="ref-for-forced-colors-mode①②"></a>

The element’s colors are automatically adjusted by the UA in [forced colors mode](#forced-colors-mode).

<a id="valdef-forced-color-adjust-none"></a>none  
<a id="ref-for-forced-colors-mode①③"></a>

The element’s colors are not automatically adjusted by the UA in [forced colors mode](#forced-colors-mode).

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors should only use this value
when they are themselves adjusting the colors
to support the user’s color and contrast needs
and need to make changes to the UA’s default adjustments
to provide a more appropriate user experience
for those elements.</strong>

<a id="valdef-forced-color-adjust-preserve-parent-color"></a>preserve-parent-color  
<a id="ref-for-forced-colors-mode①④"></a>

<a id="ref-for-propdef-color⑧"></a>

<a id="ref-for-cascaded-value"></a>

<a id="ref-for-valdef-color-currentcolor"></a>

<a id="ref-for-valdef-all-inherit"></a>

<a id="ref-for-used-value③"></a>

In [forced colors mode](#forced-colors-mode), if the [color](https://www.w3.org/TR/css-color-4/#propdef-color) property inherits from its parent (i.e. there is no [cascaded value](https://www.w3.org/TR/css-cascade-5/#cascaded-value) or the <a id="ref-for-cascaded-value①"></a>cascaded value is [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor), [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit), or another keyword that inherits from the parent), then it computes to the [used](https://www.w3.org/TR/css-cascade-5/#used-value) color of its parent’s <a id="ref-for-propdef-color⑨"></a>color value.

<a id="ref-for-valdef-forced-color-adjust-none"></a>

In all other respects, behaves the same as [none](#valdef-forced-color-adjust-none).

<a id="ref-for-forced-colors-mode①⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value is intended solely to get a reasonable behavior from embedded SVG elements that expect to receive the outer document’s text color (and stay consistent with adjustments from [forced colors mode](#forced-colors-mode)), while otherwise defaulting SVGs to preserving their exact colors, as <a id="ref-for-forced-colors-mode①⑥"></a>forced colors mode can’t generally be usefully applied to illustrations.

In order to not break SVG content, UAs are expected to add the following rules to their UA style sheet:

```text
@namespace "http://www.w3.org/2000/svg";
svg|svg { forced-color-adjust: preserve-parent-color; }
svg|foreignObject { forced-color-adjust: auto; }
```
<a id="ref-for-propdef-forced-color-adjust④"></a>

<a id="ref-for-the-body-element"></a>

UAs must propagate the [forced-color-adjust](#propdef-forced-color-adjust) value set on the root element to the document viewport (where it can affect e.g. the canvas background). Note that <a id="ref-for-propdef-forced-color-adjust⑤"></a>forced-color-adjust is <em>not</em> propagated from HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code>.

Tests

- [inheritance.html](https://wpt.fyi/results/css/css-forced-color-adjust/inheritance.html) [(live test)](http://wpt.live/css/css-forced-color-adjust/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-forced-color-adjust/inheritance.html)
- [forced-color-adjust-computed.html](https://wpt.fyi/results/css/css-forced-color-adjust/parsing/forced-color-adjust-computed.html) [(live test)](http://wpt.live/css/css-forced-color-adjust/parsing/forced-color-adjust-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-forced-color-adjust/parsing/forced-color-adjust-computed.html)
- [forced-color-adjust-invalid.html](https://wpt.fyi/results/css/css-forced-color-adjust/parsing/forced-color-adjust-invalid.html) [(live test)](http://wpt.live/css/css-forced-color-adjust/parsing/forced-color-adjust-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-forced-color-adjust/parsing/forced-color-adjust-invalid.html)
- [forced-color-adjust-valid.html](https://wpt.fyi/results/css/css-forced-color-adjust/parsing/forced-color-adjust-valid.html) [(live test)](http://wpt.live/css/css-forced-color-adjust/parsing/forced-color-adjust-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-forced-color-adjust/parsing/forced-color-adjust-valid.html)

## <a id="perf"></a>4. Performance-based Color Adjustments

On most monitors, the color choices that authors make have no significant difference in terms of how the device performs; displaying a document with a white background or a black background is approximately equally easy.

However, some devices have limitations and other qualities that make this assumption untrue. For example, printers tend to print on white paper; a document with a white background thus has to spend no ink on drawing that background, while a document with a black background will have to expend a large amount of ink filling in the background color. This tends to look fairly bad, and sometimes has deleterious physical effects on the paper, not to mention the vastly increased printing cost from expending the extra ink. Even fairly small differences, such as coloring text black versus dark gray, can be quite different when printing, as it switches from using a single black ink to a mixture of cyan, magenta, and yellow ink, resulting in higher ink usage and lower resolution.

As a result, in some circumstances user agents will alter the styles an author specifies in some particular context, adjusting them to be more appropriate for the output device and to accommodate what they assume the user would prefer. However, in some cases the document may be using colors in important, well-thought-out ways that the user would appreciate, and so the document would like some way to hint to the user agent that it might want to respect the page’s color choices. This section defines properties for controlling these automatic adjustments.

<a id="ref-for-propdef-print-color-adjust①"></a>

### <a id="print-color-adjust"></a>4.1. Ink Economy: the [print-color-adjust](#propdef-print-color-adjust) property

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-print-color-adjust"></a>print-color-adjust

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one⑤"></a>

economy [\|](https://www.w3.org/TR/css-values-4/#comb-one) exact

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

economy

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

yes

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

Tests

- [print-color-adjust.html](https://wpt.fyi/results/css/css-color-adjust/parsing/print-color-adjust.html) [(live test)](http://wpt.live/css/css-color-adjust/parsing/print-color-adjust.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/parsing/print-color-adjust.html)

<a id="ref-for-propdef-print-color-adjust②"></a>

The [print-color-adjust](#propdef-print-color-adjust) property provides a hint to the user-agent about how it should treat color and style choices that might be expensive or generally unwise on a printer or similar device, such as using light text on a dark background. If user agents allow users to control this aspect of the document’s display, the user preference <strong>must</strong> be respected more strongly than the hint provided by <a id="ref-for-propdef-print-color-adjust③"></a>print-color-adjust. It has the following values:

<a id="valdef-print-color-adjust-economy"></a>economy  
The user agent should make adjustments to the page’s styling as it deems necessary and prudent for the output device.

For example, if the document is being printed, a user agent might ignore any backgrounds and adjust text color to be sufficiently dark, to minimize ink usage.

<a id="valdef-print-color-adjust-exact"></a>exact  
This value indicates that the page is using color and styling on the specified element in a way which is important and significant, and which should not be tweaked or changed except at the user’s request.

For example, a mapping website offering printed directions might "zebra-stripe" the steps in the directions, alternating between white and light gray backgrounds. Losing this zebra-striping and having a pure-white background would make the directions harder to read with a quick glance when distracted in a car.

<a id="ref-for-propdef-print-color-adjust④"></a>

<a id="ref-for-the-body-element①"></a>

UAs must propagate the [print-color-adjust](#propdef-print-color-adjust) value set on the root element to the document viewport (where it can affect e.g. the canvas background). Note that <a id="ref-for-propdef-print-color-adjust⑤"></a>print-color-adjust is <em>not</em> propagated from HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code>.

<a id="ref-for-propdef-color-adjust"></a>

### <a id="color-adjust"></a>4.2. The [color-adjust](#propdef-color-adjust) Shorthand

<strong>Table 4 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-color-adjust"></a>color-adjust

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-print-color-adjust⑥"></a>

[\<'print-color-adjust'\>](#propdef-print-color-adjust)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-propdef-color-adjust①"></a>

<a id="ref-for-propdef-print-color-adjust⑦"></a>

The [color-adjust](#propdef-color-adjust) shorthand allows an author to set all of the performance-motivated color adjustment properties in one declaration. (Currently, there is only one such property—​[print-color-adjust](#propdef-print-color-adjust)—​but more might be added in the future.)

<a id="ref-for-propdef-color-adjust②"></a>

<a id="ref-for-propdef-print-color-adjust⑧"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> The <a href="#propdef-color-adjust">color-adjust</a> shorthand is currently <em>deprecated</em>.
	Authors should use the more specific <a href="#propdef-print-color-adjust">print-color-adjust</a> property,
	to avoid accidentally resetting performance-based color adjustments
	in other contexts than the one intended.</strong>

Tests

- [color-scheme-no-interpolation.html](https://wpt.fyi/results/css/css-color-adjust/animation/color-scheme-no-interpolation.html) [(live test)](http://wpt.live/css/css-color-adjust/animation/color-scheme-no-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/animation/color-scheme-no-interpolation.html)
- [forced-color-adjust-no-interpolation.html](https://wpt.fyi/results/css/css-color-adjust/animation/forced-color-adjust-no-interpolation.html) [(live test)](http://wpt.live/css/css-color-adjust/animation/forced-color-adjust-no-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/animation/forced-color-adjust-no-interpolation.html)
- [inheritance.html](https://wpt.fyi/results/css/css-color-adjust/inheritance.html) [(live test)](http://wpt.live/css/css-color-adjust/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color-adjust/inheritance.html)

## <a id="color-adjust-emulation"></a>5. Emulation

For the purposes of user agent automation and application testing, this document defines the below emulations.

### <a id="emulate-forced-colors-mode"></a>5.1. Emulate Forced Colors Mode

<a id="ref-for-top-level-traversable"></a>

<a id="ref-for-enumdef-forcedcolorsmodeautomationtheme"></a>

<a id="ref-for-dom-forcedcolorsmodeautomationtheme-none①"></a>

Each [top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable) has an associated <a id="emulated-forced-colors-theme-data"></a>emulated forced colors theme data, which is data representing <code><a href="#enumdef-forcedcolorsmodeautomationtheme">ForcedColorsModeAutomationTheme</a></code>, initially "<code><a href="#dom-forcedcolorsmodeautomationtheme-none">none</a></code>".

<a id="enumdef-forcedcolorsmodeautomationtheme"></a>

<a id="dom-forcedcolorsmodeautomationtheme-none"></a>

<a id="dom-forcedcolorsmodeautomationtheme-light"></a>

<a id="dom-forcedcolorsmodeautomationtheme-dark"></a>

```text
enum ForcedColorsModeAutomationTheme {
  "none",
  "light",
  "dark"
};
```
<a id="ref-for-window-navigable"></a>

To <a id="set-emulated-forced-colors-theme-data"></a>set emulated forced colors theme data, given [navigable](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window-navigable) <var>navigable</var> and an <var>emulatedThemeData</var>:

1.  <a id="ref-for-enumdef-forcedcolorsmodeautomationtheme①"></a>

    Assert <var>emulatedThemeData</var> is <code><a href="#enumdef-forcedcolorsmodeautomationtheme">ForcedColorsModeAutomationTheme</a></code>.

2.  <a id="ref-for-nav-top"></a>

    Let <var>traversable</var> be <var>navigable</var>’s [top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-top).

3.  If <var>traversable</var> is not null:

    1.  <a id="ref-for-emulated-forced-colors-theme-data①"></a>

        Set <var>traversable</var>’s associated [emulated forced colors theme data](#emulated-forced-colors-theme-data) to <var>emulatedThemeData</var>.

    2.  UAs must consider this a change that requires style recalculation.

<a id="ref-for-enumdef-forcedcolorsmodeautomationtheme②"></a>

To <a id="get-emulated-forced-colors-theme-data"></a>get emulated forced colors theme data, given <code><a href="#enumdef-forcedcolorsmodeautomationtheme">ForcedColorsModeAutomationTheme</a></code> <var>theme</var>:

1.  <a id="ref-for-concept-relevant-global"></a>

    <a id="ref-for-concept-document-window"></a>

    <a id="ref-for-node-navigable"></a>

    Let <var>navigable</var> be <var>theme</var>’s [relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global)’s [associated Document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window)’s [node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable).

2.  If <var>navigable</var> is null, return null.

3.  <a id="ref-for-nav-top①"></a>

    Let <var>traversable</var> be <var>navigable</var>’s [top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-top).

4.  If <var>traversable</var> is null, return null.

5.  <a id="ref-for-emulated-forced-colors-theme-data②"></a>

    Return <var>traversable</var>’s associated [emulated forced colors theme data](#emulated-forced-colors-theme-data).

### <a id="forced-color-palettes"></a>5.2. Forced Colors Mode Color Palettes

For the purposes of user agent automation and application testing, this document defines the below <a id="forced-colors-mode-emulation-color-palettes"></a>forced colors mode emulation color palettes.

<a id="ref-for-css-system-colors⑥"></a>

<a id="ref-for-dom-forcedcolorsmodeautomationtheme-light"></a>

<strong>Table 5 — structured row/cell transcription</strong>

[System color](https://drafts.csswg.org/css-color-4/#css-system-colors) mappings for "<code><a href="#dom-forcedcolorsmodeautomationtheme-light">light</a></code>"

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

<a id="ref-for-typedef-system-color②"></a>

[\<system-color\>](https://www.w3.org/TR/css-color-4/#typedef-system-color) keyword

<strong>Column 2 (header cell):</strong>

Value

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-accentcolor"></a>

[AccentColor](https://www.w3.org/TR/css-color-4/#valdef-color-accentcolor)

<strong>Column 2 (data cell):</strong>

 #FFFFFF

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-accentcolortext"></a>

[AccentColorText](https://www.w3.org/TR/css-color-4/#valdef-color-accentcolortext)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-activetext"></a>

[ActiveText](https://www.w3.org/TR/css-color-4/#valdef-color-activetext)

<strong>Column 2 (data cell):</strong>

 #00009F

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-buttonborder"></a>

[ButtonBorder](https://www.w3.org/TR/css-color-4/#valdef-color-buttonborder)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 6</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-buttonface"></a>

[ButtonFace](https://www.w3.org/TR/css-color-4/#valdef-color-buttonface)

<strong>Column 2 (data cell):</strong>

 #FFFFFF

<strong>Row 7</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-buttontext"></a>

[ButtonText](https://www.w3.org/TR/css-color-4/#valdef-color-buttontext)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 8</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-canvas④"></a>

[Canvas](https://www.w3.org/TR/css-color-4/#valdef-color-canvas)

<strong>Column 2 (data cell):</strong>

 #FFFFFF

<strong>Row 9</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-canvastext③"></a>

[CanvasText](https://www.w3.org/TR/css-color-4/#valdef-color-canvastext)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 10</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-field"></a>

[Field](https://www.w3.org/TR/css-color-4/#valdef-color-field)

<strong>Column 2 (data cell):</strong>

 #FFFFFF

<strong>Row 11</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-fieldtext"></a>

[FieldText](https://www.w3.org/TR/css-color-4/#valdef-color-fieldtext)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 12</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-graytext"></a>

[GrayText](https://www.w3.org/TR/css-color-4/#valdef-color-graytext)

<strong>Column 2 (data cell):</strong>

 #600000

<strong>Row 13</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-highlight"></a>

[Highlight](https://www.w3.org/TR/css-color-4/#valdef-color-highlight)

<strong>Column 2 (data cell):</strong>

 #37006E

<strong>Row 14</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-highlighttext"></a>

[HighlightText](https://www.w3.org/TR/css-color-4/#valdef-color-highlighttext)

<strong>Column 2 (data cell):</strong>

 #FFFFFF

<strong>Row 15</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-linktext"></a>

[LinkText](https://www.w3.org/TR/css-color-4/#valdef-color-linktext)

<strong>Column 2 (data cell):</strong>

 #00009F

<strong>Row 16</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-mark"></a>

[Mark](https://www.w3.org/TR/css-color-4/#valdef-color-mark)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-css-system-colors⑦"></a>

N/A - this [system color](https://drafts.csswg.org/css-color-4/#css-system-colors) keyword should not be adjusted.

<strong>Row 17</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-marktext"></a>

[MarkText](https://www.w3.org/TR/css-color-4/#valdef-color-marktext)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-css-system-colors⑧"></a>

N/A - this [system color](https://drafts.csswg.org/css-color-4/#css-system-colors) keyword should not be adjusted.

<strong>Row 18</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-selecteditem"></a>

[SelectedItem](https://www.w3.org/TR/css-color-4/#valdef-color-selecteditem)

<strong>Column 2 (data cell):</strong>

 #37006E

<strong>Row 19</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-selecteditemtext"></a>

[SelectedItemText](https://www.w3.org/TR/css-color-4/#valdef-color-selecteditemtext)

<strong>Column 2 (data cell):</strong>

 #FFFFFF

<strong>Row 20</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-visitedtext"></a>

[VisitedText](https://www.w3.org/TR/css-color-4/#valdef-color-visitedtext)

<strong>Column 2 (data cell):</strong>

 #00009F

<a id="ref-for-css-system-colors⑨"></a>

<a id="ref-for-dom-forcedcolorsmodeautomationtheme-dark"></a>

<strong>Table 6 — structured row/cell transcription</strong>

[System color](https://drafts.csswg.org/css-color-4/#css-system-colors) mappings for "<code><a href="#dom-forcedcolorsmodeautomationtheme-dark">dark</a></code>"

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

<a id="ref-for-typedef-system-color③"></a>

[\<system-color\>](https://www.w3.org/TR/css-color-4/#typedef-system-color) keyword

<strong>Column 2 (header cell):</strong>

Value

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-accentcolor①"></a>

[AccentColor](https://www.w3.org/TR/css-color-4/#valdef-color-accentcolor)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-accentcolortext①"></a>

[AccentColorText](https://www.w3.org/TR/css-color-4/#valdef-color-accentcolortext)

<strong>Column 2 (data cell):</strong>

 #FFFFFF

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-activetext①"></a>

[ActiveText](https://www.w3.org/TR/css-color-4/#valdef-color-activetext)

<strong>Column 2 (data cell):</strong>

 #FFFF00

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-buttonborder①"></a>

[ButtonBorder](https://www.w3.org/TR/css-color-4/#valdef-color-buttonborder)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 6</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-buttonface①"></a>

[ButtonFace](https://www.w3.org/TR/css-color-4/#valdef-color-buttonface)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 7</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-buttontext①"></a>

[ButtonText](https://www.w3.org/TR/css-color-4/#valdef-color-buttontext)

<strong>Column 2 (data cell):</strong>

 #FFFFFF

<strong>Row 8</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-canvas⑤"></a>

[Canvas](https://www.w3.org/TR/css-color-4/#valdef-color-canvas)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 9</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-canvastext④"></a>

[CanvasText](https://www.w3.org/TR/css-color-4/#valdef-color-canvastext)

<strong>Column 2 (data cell):</strong>

 #FFFFFF

<strong>Row 10</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-field①"></a>

[Field](https://www.w3.org/TR/css-color-4/#valdef-color-field)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 11</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-fieldtext①"></a>

[FieldText](https://www.w3.org/TR/css-color-4/#valdef-color-fieldtext)

<strong>Column 2 (data cell):</strong>

 #FFFFFF

<strong>Column 3 (data cell):</strong>

<strong>Row 12</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-graytext①"></a>

[GrayText](https://www.w3.org/TR/css-color-4/#valdef-color-graytext)

<strong>Column 2 (data cell):</strong>

 #3FF23F

<strong>Row 13</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-highlight①"></a>

[Highlight](https://www.w3.org/TR/css-color-4/#valdef-color-highlight)

<strong>Column 2 (data cell):</strong>

 #1AEBFF

<strong>Row 14</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-highlighttext①"></a>

[HighlightText](https://www.w3.org/TR/css-color-4/#valdef-color-highlighttext)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 15</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-linktext①"></a>

[LinkText](https://www.w3.org/TR/css-color-4/#valdef-color-linktext)

<strong>Column 2 (data cell):</strong>

 #FFFF00

<strong>Row 16</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-mark①"></a>

[Mark](https://www.w3.org/TR/css-color-4/#valdef-color-mark)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-css-system-colors①⓪"></a>

N/A - this [system color](https://drafts.csswg.org/css-color-4/#css-system-colors) keyword should not be adjusted.

<strong>Row 17</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-marktext①"></a>

[MarkText](https://www.w3.org/TR/css-color-4/#valdef-color-marktext)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-css-system-colors①①"></a>

N/A - this [system color](https://drafts.csswg.org/css-color-4/#css-system-colors) keyword should not be adjusted.

<strong>Row 18</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-selecteditem①"></a>

[SelectedItem](https://www.w3.org/TR/css-color-4/#valdef-color-selecteditem)

<strong>Column 2 (data cell):</strong>

 #1AEBFF

<strong>Row 19</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-selecteditemtext①"></a>

[SelectedItemText](https://www.w3.org/TR/css-color-4/#valdef-color-selecteditemtext)

<strong>Column 2 (data cell):</strong>

 #000000

<strong>Row 20</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-color-visitedtext①"></a>

[VisitedText](https://www.w3.org/TR/css-color-4/#valdef-color-visitedtext)

<strong>Column 2 (data cell):</strong>

 #FFFF00

## <a id="privacy"></a>6. Privacy Considerations

<a id="ref-for-color-scheme①⑨"></a>

<a id="ref-for-forced-colors-mode①⑦"></a>

<a id="ref-for-dom-window-getcomputedstyle"></a>

Applying user color preferences via [color schemes](#color-scheme) or [forced colors mode](#forced-colors-mode) exposes the user’s color preferences to the page via <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code>, which can increase fingerprinting surface.

> <strong data-conversion-semantic="note">Note</strong>
>
> Avoiding this comes with unfortunate drawbacks that were deemed too significant to be ignored. Namely:
>
> - <a id="ref-for-propdef-color①⓪"></a>
>
>   preserving system colors as keywords until actual-value time would break a significant amount of deployed script, as the initial value of [color](https://www.w3.org/TR/css-color-4/#propdef-color) is a system color already (but a huge amount of script implicitly expects to see an RGB color from <a id="ref-for-propdef-color①①"></a>color)
>
> - lying about system colors from the scripting APIs (pretending they’re always some static values) can result in any colors calculated <em>from</em> page colors in script being unreadable when used with the <em>actual</em> system colors.
>
> See [Issue 5710](https://github.com/w3c/csswg-drafts/issues/5710#issuecomment-840772752) for discussion on this topic.

## <a id="security"></a>7. Security Considerations

<a id="ref-for-propdef-color-scheme①⑤"></a>

<a id="ref-for-the-iframe-element①"></a>

It may be possible for an embedded document to use timing attacks to determine whether its own [color-scheme](#propdef-color-scheme) matches that of its embedding <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> or not.

## <a id="acknowledgements"></a>8. Acknowledgements

This specification would not be possible without the development efforts of various color adjustment features at Apple, Google, and Microsoft as well as discussions about print adjustments on www-style. In particular, the CSS Working Group would like to thank: François Remy, イアンフェッティ

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-8edfffeb"></a> List additional MSFT / Apple / Google people here.

## <a id="changes"></a>9. Changes

Changes since the [10 February 2022 Candidate Recommendation Snapshot](https://www.w3.org/TR/2022/CR-css-color-adjust-1-20220210/):

- <a id="ref-for-funcdef-color"></a>

  Removed special handling of [color()](https://www.w3.org/TR/css-color-5/#funcdef-color) fallback system colors, since the feature was removed from [\[CSS-COLOR-4\]](#biblio-css-color-4). ([Issue 7007](https://github.com/w3c/csswg-drafts/issues/7007))

- <a id="ref-for-forced-colors-mode①⑧"></a>

  Added emulation support for improved testing of [forced colors mode](#forced-colors-mode). ([Issue 11824](https://github.com/w3c/csswg-drafts/issues/11824#issuecomment-2769343959))

- <a id="ref-for-typedef-color②"></a>

  <a id="ref-for-forced-colors-mode①⑨"></a>

  Updated the properties that apply in [forced colors mode](#forced-colors-mode) to more generically apply to the [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) components of all properties, with specific known properties moved to a note. ([Issue 11857](https://github.com/w3c/csswg-drafts/issues/11857))

- <a id="ref-for-forced-colors-mode②⓪"></a>

  Added font emoji fallback logic for [forced colors mode](#forced-colors-mode). ([Issue 8064](https://github.com/w3c/csswg-drafts/issues/8064))

<strong>See also <a href="https://www.w3.org/TR/2022/CR-css-color-adjust-1-20220210/#changes">Changes prior to Candidate Recommendation</a>.</strong>

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

- [auto](#valdef-forced-color-adjust-auto), in § 3.2
- [color-adjust](#propdef-color-adjust), in § 4.2
- [color scheme](#color-scheme), in § 2
- [color-scheme](#propdef-color-scheme), in § 2.1
- [\<custom-ident\>](#valdef-color-scheme-custom-ident), in § 2.1
- ["dark"](#dom-forcedcolorsmodeautomationtheme-dark), in § 5.1
- dark
  - [definition of](#dark-color-scheme), in § 2
  - [value for color-scheme](#valdef-color-scheme-dark), in § 2.1
- [dark color scheme](#dark-color-scheme), in § 2
- [determine the used color scheme](#determine-the-used-color-scheme), in § 2.1
- [economy](#valdef-print-color-adjust-economy), in § 4.1
- [emulated forced colors theme data](#emulated-forced-colors-theme-data), in § 5.1
- [exact](#valdef-print-color-adjust-exact), in § 4.1
- [forced-color-adjust](#propdef-forced-color-adjust), in § 3.2
- [Forced colors mode](#forced-colors-mode), in § 3
- [ForcedColorsModeAutomationTheme](#enumdef-forcedcolorsmodeautomationtheme), in § 5.1
- [forced colors mode emulation color palettes](#forced-colors-mode-emulation-color-palettes), in § 5.2
- [get emulated forced colors theme data](#get-emulated-forced-colors-theme-data), in § 5.1
- ["light"](#dom-forcedcolorsmodeautomationtheme-light), in § 5.1
- light
  - [definition of](#light-color-scheme), in § 2
  - [value for color-scheme](#valdef-color-scheme-light), in § 2.1
- [light color scheme](#light-color-scheme), in § 2
- ["none"](#dom-forcedcolorsmodeautomationtheme-none), in § 5.1
- [none](#valdef-forced-color-adjust-none), in § 3.2
- [normal](#valdef-color-scheme-normal), in § 2.1
- [only](#valdef-color-scheme-only), in § 2.1
- [override the color scheme](#override-the-color-scheme), in § 2.3
- [page’s supported color schemes](#pages-supported-color-schemes), in § 2.1
- [preferred color scheme](#preferred-color-scheme), in § 2
- [preserve-parent-color](#valdef-forced-color-adjust-preserve-parent-color), in § 3.2
- [print-color-adjust](#propdef-print-color-adjust), in § 4.1
- [set emulated forced colors theme data](#set-emulated-forced-colors-theme-data), in § 5.1
- [used color scheme](#used-color-scheme), in § 2.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="2754893b"></a>background-color
  - <a id="5ced56d0"></a>background-image
  - <a id="65b3a7bc"></a>border-color
  - <a id="c48eaa20"></a>box-shadow
  - <a id="e37189d4"></a>none (for background-image)
  - <a id="429d8615"></a>none (for box-shadow)
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="633c07bb"></a>author style sheet
  - <a id="9b9f041e"></a>cascaded value
  - <a id="8c8e51b4"></a>computed value
  - <a id="d0dc95c3"></a>inherit
  - <a id="1a2b1083"></a>used value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="024532da"></a>\<system-color\>
  - <a id="9e0bc48b"></a>AccentColor
  - <a id="cd76f094"></a>AccentColorText
  - <a id="135b5653"></a>ActiveText
  - <a id="52998fb1"></a>ButtonBorder
  - <a id="155fed85"></a>ButtonFace
  - <a id="35bee9ae"></a>ButtonText
  - <a id="546b7767"></a>canvas
  - <a id="535c0d72"></a>CanvasText
  - <a id="bcdf9b19"></a>color
  - <a id="a42c65ac"></a>currentcolor
  - <a id="727feb1d"></a>field
  - <a id="f0e1a99c"></a>FieldText
  - <a id="f5ae03dd"></a>GrayText
  - <a id="12ea2c8a"></a>highlight
  - <a id="5be45736"></a>HighlightText
  - <a id="ac98dd35"></a>LinkText
  - <a id="0154b7a1"></a>mark
  - <a id="2829cd6a"></a>MarkText
  - <a id="ad228393"></a>SelectedItem
  - <a id="a1ed3c6c"></a>SelectedItemText
  - <a id="5113d0cf"></a>system color pairings
  - <a id="75ffc4f4"></a>system colors
  - <a id="28beb5e0"></a>VisitedText
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="d04b6986"></a>\<color\>
  - <a id="4966c7f8"></a>color()
  - <a id="0644a74e"></a>color-mix()
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="4397147f"></a>@media
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="f91916bc"></a>font-variant-emoji
  - <a id="a0601d8f"></a>normal
  - <a id="1298492a"></a>text
  - <a id="76400d36"></a>unicode
- \[CSS-GAPS-1\] defines the following terms:
  - <a id="f744943d"></a>rule-color
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="b140b8f3"></a>::selection
- \[CSS-SCROLLBARS-1\] defines the following terms:
  - <a id="51feba0c"></a>auto
  - <a id="d9b90cda"></a>scrollbar-color
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="39483a6f"></a>text-decoration-color
  - <a id="441d3fda"></a>text-emphasis-color
  - <a id="a7f17cc4"></a>text-shadow
- \[CSS-TYPED-OM-1\] defines the following terms:
  - <a id="5d3144b2"></a>computedStyleMap()
- \[CSS-UI-4\] defines the following terms:
  - <a id="8a568d22"></a>accent-color
  - <a id="18ad5848"></a>auto
  - <a id="8378d695"></a>caret-color
  - <a id="6ed19243"></a>outline-color
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="af4a190d"></a>+
  - <a id="a0144f62"></a>\<custom-ident\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="3fa441fa"></a>url()
  - <a id="4eb9d37e"></a>\|
- \[CSS2\] defines the following terms:
  - <a id="0a714736"></a>canvas
- \[CSSOM-1\] defines the following terms:
  - <a id="bfb148e6"></a>getComputedStyle(elt)
- \[FILL-STROKE-3\] defines the following terms:
  - <a id="319ed6dc"></a>fill
  - <a id="6529d8b3"></a>stroke
- \[FILTER-EFFECTS-1\] defines the following terms:
  - <a id="cb964126"></a>flood-color
  - <a id="ca17d95e"></a>lighting-color
- \[HTML\] defines the following terms:
  - <a id="3349d69f"></a>associated Document
  - <a id="2f0492ac"></a>body
  - <a id="bfff6250"></a>button
  - <a id="84ee7ae3"></a>color-scheme
  - <a id="7992f81d"></a>em
  - <a id="87fcd40c"></a>iframe
  - <a id="f0811ff8"></a>img
  - <a id="64b89595"></a>meta
  - <a id="228be966"></a>navigable
  - <a id="13dd6cae"></a>node navigable
  - <a id="e99bd18e"></a>relevant global object
  - <a id="bd736ec6"></a>top-level traversable
  - <a id="30717154"></a>top-level traversable (for navigable)
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="2850b5b1"></a>dark
  - <a id="2bbf6b0c"></a>forced-colors
  - <a id="3ea2fcbb"></a>media query
  - <a id="8c8c78a9"></a>prefers-color-scheme
  - <a id="f8503360"></a>prefers-contrast
- \[SVG2\] defines the following terms:
  - <a id="c6539ab4"></a>stop-color

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Chris Lilley; Tab Atkins Jr.; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 24 April 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 18 March 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 1 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-scrollbars-1"></a>\[CSS-SCROLLBARS-1\]  
Tantek Çelik; Rossen Atanassov; Florian Rivoal. [CSS Scrollbars Styling Module Level 1](https://www.w3.org/TR/css-scrollbars-1/). 9 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scrollbars-1&#x2F;](https://www.w3.org/TR/css-scrollbars-1/)

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 4 May 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 15 August 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-gaps-1"></a>\[CSS-GAPS-1\]  
Kevin Babbitt. [CSS Gap Decorations Module Level 1](https://www.w3.org/TR/css-gaps-1/). 17 April 2025. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-gaps-1&#x2F;](https://www.w3.org/TR/css-gaps-1/)

<a id="biblio-css-typed-om-1"></a>\[CSS-TYPED-OM-1\]  
Tab Atkins Jr.; François Remy. [CSS Typed OM Level 1](https://www.w3.org/TR/css-typed-om-1/). 21 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-typed-om-1&#x2F;](https://www.w3.org/TR/css-typed-om-1/)

<a id="biblio-fill-stroke-3"></a>\[FILL-STROKE-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Fill and Stroke Module Level 3](https://www.w3.org/TR/fill-stroke-3/). 13 April 2017. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;fill-stroke-3&#x2F;](https://www.w3.org/TR/fill-stroke-3/)

<a id="biblio-filter-effects-1"></a>\[FILTER-EFFECTS-1\]  
Dirk Schulze; Dean Jackson. [Filter Effects Module Level 1](https://www.w3.org/TR/filter-effects-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;filter-effects-1&#x2F;](https://www.w3.org/TR/filter-effects-1/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

<a id="biblio-wcag22"></a>\[WCAG22\]  
Michael Cooper; et al. [Web Content Accessibility Guidelines (WCAG) 2.2](https://www.w3.org/TR/WCAG22/). 12 December 2024. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;WCAG22&#x2F;](https://www.w3.org/TR/WCAG22/)

## <a id="property-index"></a>Property Index

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell; scope col):</strong>

Name

<strong>Column 2 (header cell; scope col):</strong>

Value

<strong>Column 3 (header cell; scope col):</strong>

Initial

<strong>Column 4 (header cell; scope col):</strong>

Applies to

<strong>Column 5 (header cell; scope col):</strong>

Inh.

<strong>Column 6 (header cell; scope col):</strong>

%ages

<strong>Column 7 (header cell; scope col):</strong>

Anim­ation type

<strong>Column 8 (header cell; scope col):</strong>

Canonical order

<strong>Column 9 (header cell; scope col):</strong>

Com­puted value

<strong>Row 2</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-color-adjust③"></a>

[color-adjust](#propdef-color-adjust)

<strong>Column 2 (data cell):</strong>

\<'print-color-adjust'\>

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-color-scheme①⑥"></a>

[color-scheme](#propdef-color-scheme)

<strong>Column 2 (data cell):</strong>

normal \| \[ light \| dark \| \<custom-ident\> \]+ &#x26;&#x26; only?

<strong>Column 3 (data cell):</strong>

normal

<strong>Column 4 (data cell):</strong>

all elements and text

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword normal, or an ordered list of specified color scheme keywords

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-forced-color-adjust⑥"></a>

[forced-color-adjust](#propdef-forced-color-adjust)

<strong>Column 2 (data cell):</strong>

auto \| none \| preserve-parent-color

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

all elements and text

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-print-color-adjust⑨"></a>

[print-color-adjust](#propdef-print-color-adjust)

<strong>Column 2 (data cell):</strong>

economy \| exact

<strong>Column 3 (data cell):</strong>

economy

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword

## <a id="idl-index"></a>IDL Index

```text
enum ForcedColorsModeAutomationTheme {
  "none",
  "light",
  "dark"
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> List additional MSFT / Apple / Google people here. [↵](#issue-8edfffeb)
