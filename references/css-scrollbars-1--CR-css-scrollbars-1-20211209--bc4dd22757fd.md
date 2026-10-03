Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Scrollbars Styling Module Level 1](https://www.w3.org/TR/2021/CR-css-scrollbars-1-20211209/).

Original copyright notice: Copyright © 2021 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Scrollbars Styling Module Level 1

Source snapshot: https://www.w3.org/TR/2021/CR-css-scrollbars-1-20211209/

Snapshot SHA-256: bc4dd22757fd674e717b025d9826a95bd5674ed3ae4d0e5bb137e2ef1f6533af

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 3 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Scrollbars Styling Module Level 1

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2021 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This CSS module defines properties to influence the visual styling of scrollbars, introducing controls for their color and width.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Snapshot</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Snapshot has received [wide review](https://www.w3.org/2021/Process-20211102/#dfn-wide-review), is intended to gather implementation experience, and has commitments from Working Group members to [royalty-free licensing](https://www.w3.org/Consortium/Patent-Policy/#sec-Requirements) for implementations. This document is intended to become a W3C Recommendation; it will remain a Candidate Recommendation at least until 7 February 2022 to gather additional feedback.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-scrollbars” in the title, like this: “\[css-scrollbars\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-scrollbars%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<em>This section is non-normative.</em>

<a id="ref-for-propdef-scrollbar-color"></a>

<a id="ref-for-propdef-scrollbar-width"></a>

This CSS module introduces properties to influence the visual styling of scrollbars, including their color ([scrollbar-color](#propdef-scrollbar-color)) and thickness ([scrollbar-width](#propdef-scrollbar-width)).

### <a id="scope"></a>1.1.  Scope

The CSS Scrollbars Module is specifically for styling scrollbar controls themselves, e.g. their color &#x26; width in Level 1, and not their layout nor whether any content is scrollable. All layout impacts and content scrollability are specified in the [CSS Overflow Module](https://drafts.csswg.org/css-overflow/).

Based on [documented use-cases](https://www.w3.org/wiki/Css-scrollbars#Use-cases), there are three main use-cases around scrollbars this module intends to resolve:

1.  Coloring scrollbars to fit better into the UI of a web application.
2.  Using a thinner scrollbar when the scrolling area is small.
3.  Hiding UA-provided scrollbars, to allow the provision of custom interfaces for scrolling without affecting other aspects of scrollability.

#### <a id="out-of-scope"></a>1.1.1.  Out Of Scope

The internal structure, layout, and configuration of scrollbars, as well as precise control over their coloring, is out of scope. This is because different platforms have different scrollbar structures and styling conventions, and operating systems continuously evolve their scrollbar designs to provide better user experience. Pseudo-elements for selecting specific parts of a scrollbar, for example, were considered and rejected. While this level of fine control would be tempting for authors, the arrangement of the various parts—or whether they’re even all present—cannot be depended on. Providing too much control would allow authors to get perfect results on some platforms, but at the expense of broken results on others.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Exposing the scrollbar-related `::-webkit-` prefixed pseudo-elements to the Web is considered a mistake by both the CSS Working Group and Webkit.

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

<a id="ref-for-propdef-scrollbar-color①"></a>

## <a id="scrollbar-color"></a>2. Scrollbar Colors: the [scrollbar-color](#propdef-scrollbar-color) property

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scrollbar-color"></a>scrollbar-color

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num"></a>

<a id="ref-for-typedef-color"></a>

<a id="ref-for-comb-one"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color)[{2}](https://www.w3.org/TR/css-values-4/#mult-num)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-scroll-container"></a>

[scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container)

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

specified keyword or two computed colors

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value

This property allows the author to set colors of an element’s scrollbars.

UAs must apply the scrollbar-color value set on the root element to the viewport.

<a id="ref-for-propdef-overflow"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unlike [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) (and overflow-\*) properties, scrollbar-color value set on the HTML body element are not propagated to the viewport.

<a id="valdef-scrollbar-color-auto"></a>auto

<a id="ref-for-propdef-color-scheme"></a>

The user agent determines the colors of the scrollbar. It should follow platform conventions, but may adjust the colors in accordance with [color-scheme](https://www.w3.org/TR/css-color-adjust-1/#propdef-color-scheme) or other contextual information to better suit the page.

<a id="ref-for-typedef-color①"></a>

<a id="valdef-scrollbar-color-color"></a>[\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color)

apply the first color to the thumb of the scrollbar, and the second color to the track of the scrollbar.

Details:

Track refers to the background of the scrollbar, which is generally fixed regardless of the scrolling position.

Thumb refers to the moving part of the scrollbar, which usually floats on top of the track.

<a id="ref-for-valdef-scrollbar-color-auto"></a>

If this property computes to a value other than [auto](#valdef-scrollbar-color-auto), implementations may render a simpler scrollbar than the default platform UI rendering, and color it accordingly.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Sometimes the UA is unable to customize the colors of native scrollbars, perhaps due to how they’re structured, or to a lack of control given by the native toolkit. The provision above allows the UA to replace them with differently-constructed scrollbars, which it does know how to color.

(Note: add diagram showing the different named pieces - something like [http&#x3A;&#x2F;&#x2F;www&#x2E;howtocreate&#x2E;co&#x2E;uk&#x2F;tutorials&#x2F;scrlbar&#x2E;html](http://www.howtocreate.co.uk/tutorials/scrlbar.html))

(Note: add example of an overflow element with colorized scrollbars to match page styling, PNG of the same in a browser that supports it currently)

Implementations may ignore any of the colors if the corresponding part do not exist on the underlying platform.

<a id="ref-for-propdef-scrollbar-color②"></a>

When using [scrollbar-color](#propdef-scrollbar-color) property with specific color values, authors should ensure the specified colors have enough contrast between them. For keyword values, UAs should ensure the colors they use have enough contrast. See [WCAG 2.1 SC 1.4.11 Non-text Contrast](https://www.w3.org/TR/WCAG21/#non-text-contrast) [\[WCAG21\]](#biblio-wcag21). UAs may ignore these contrast requirements based on explicit user preferences (for example, when users choose a configuration option/setting that always ensures a particular scrollbar color / use of system default scrollbars).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: when a user interacts with a scrollbar (e.g. hovering or activating), implementations may alter which scrollbar colors apply to which scrollbar parts.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: IE uses named System Colors as defaults for each of the scrollbar color properties. See related [Issue 1956](https://github.com/w3c/csswg-drafts/issues/1956).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8bd6d072"></a>
>
> The following example (derived from [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;Examples&#x2F;007&#x2F;scrollbars&#x2E;en&#x2E;html](https://www.w3.org/Style/Examples/007/scrollbars.en.html)) resets scrollbar colors in IE&#x2E;
>
> ```text
> html {
>     scrollbar-color: ThreeDFace Scrollbar;
> }
> ```
<a id="ref-for-propdef-scrollbar-width①"></a>

## <a id="scrollbar-width"></a>3. Scrollbar Thickness: the [scrollbar-width](#propdef-scrollbar-width) property

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scrollbar-width"></a>scrollbar-width

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one①"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) thin <a id="ref-for-comb-one②"></a>\| none

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-scroll-container①"></a>

[scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

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

by computed value

This property allows the author to specify the desired thickness of an element’s scrollbars.

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> The primary purpose of this property is not to allow authors to chose a particular scrollbar aesthetic for their pages, but to let them indicate for certain small or cramped elements of their pages that a smaller scrollbar would be desirable.
>
> Scrollbars are a UI mechanism essential to interact with the page. Operating systems tend to want consistency in such controls to improve usability through familiarity, and users with specific preferences or needs can adjust the appearance of various UI components, including scrollbars, through OS or UA settings.
>
> While using this property in support of specific UX goals is appropriate, authors should otherwise refrain from overriding such user preferences.

<a id="valdef-scrollbar-width-auto"></a>auto  
Implementations must use the default scrollbar width.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: On most systems, this corresponds to the traditional somewhat wide scrollbar. However, through OS or UA settings, users can have the ability to change what this default corresponds to, possibly making the default scrollbar wider or narrower than is typical.

<a id="valdef-scrollbar-width-thin"></a>thin  
<a id="ref-for-valdef-scrollbar-width-auto"></a>

Implementations should use thinner scrollbars than [auto](#valdef-scrollbar-width-auto). This may mean a thin variant of scrollbar provided by the platform, or a custom scrollbar thinner than the default platform scrollbar. The scrollbar must nonetheless remain wide enough to be usable. (Implementers may wish to consult [WCAG 2.1 SC 2.5.5 Target Size](https://www.w3.org/TR/WCAG21/#target-size). [\[WCAG21\]](#biblio-wcag21))

<a id="ref-for-overlay-scrollbars"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: User agents can use various strategies to ensure the usability of narrow scrollbars. For instance, in the case of [overlay scrollbars](https://www.w3.org/TR/css-overflow-3/#overlay-scrollbars), they can dynamically enlarge the scrollbar in response to a user attempting to interact with it. User agents on devices with touch screens can also adjust how they interpret finger taps to facilitate interacting with visually small touch targets.

<a id="ref-for-valdef-scrollbar-width-auto①"></a>

User agents may disregard this value and treat it as [auto](#valdef-scrollbar-width-auto), for instance when the user has indicated discomfort for thin scrollbars through some UA or OS setting. (User agents are encouraged to provide such a setting.)

<a id="ref-for-valdef-scrollbar-width-auto②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some platforms only have a tiny scrollbar by default which cannot be reasonably made thinner. In such cases, this value will behave as [auto](#valdef-scrollbar-width-auto).

<a id="valdef-scrollbar-width-none"></a>none  
Implementations must not display any scrollbar, however the element’s scrollability by other means is not affected.

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Using this value can prevent mouse-only users from being able to scroll. Authors should ensure that mouse-only users can still reach hidden content, even if they have no scrollwheel.

<a id="ref-for-valdef-scrollbar-width-none"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Authors that use [none](#valdef-scrollbar-width-none) should provide an alternative/equivalent visual hint that scrolling is possible and there is more content.

<a id="ref-for-propdef-overflow①"></a>

<a id="ref-for-propdef-scrollbar-width②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For situations where an element is to be scrolled <em>only</em> by programmatic means, and not by direct user manipulation, authors should use [overflow: hidden](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) instead of [scrollbar-width: none](#propdef-scrollbar-width).

<a id="ref-for-valdef-scrollbar-width-thin"></a>

<a id="ref-for-cascade-origin-user"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Users who find the [thin](#valdef-scrollbar-width-thin) style of scrollbars unusable can include the following rule in their [user style sheet](https://www.w3.org/TR/css-cascade-5/#cascade-origin-user):
>
> ```text
> * { scrollbar-width: auto !important; }
> ```
>
> This will ensure that all scrollbars are sized as per OS and UA settings regardless of author styles.

<a id="ref-for-propdef-scrollbar-width③"></a>

UAs must apply the [scrollbar-width](#propdef-scrollbar-width) value set on the root element to the viewport.

<a id="ref-for-propdef-overflow②"></a>

<a id="ref-for-propdef-scrollbar-width④"></a>

<a id="ref-for-the-body-element"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unlike the [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property (and its longhands), a [scrollbar-width](#propdef-scrollbar-width) value set on the HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element is not propagated to the viewport.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification does not define the exact position or shape of the scrollbar, or any animation thereof, such as fading or sliding in/out of view.

## <a id="acknowledgments"></a>Appendix A. Acknowledgments

This appendix is <em>non-normative</em>.

Thanks to the use-cases, prototyping, implementation, and feedback from [Tab Atkins](https://xanthir.com/) and [Xidorn Quan](https://www.upsuper.org/). Thanks to accessibility review and contributions ([\#3315](https://github.com/w3c/csswg-drafts/issues/3315)) from [Patrick H. Lauke](https://www.splintered.co.uk).

## <a id="changes"></a>Appendix B. Changes

This appendix is <em>non-normative</em>.

### <a id="changes-since-2021-12-02"></a> Changes since the [2021-12-02 Working Draft](https://www.w3.org/TR/css-scrollbars-1/) 

- Boilerplate changes for CR

### <a id="changes-since-2021-08-05"></a> Changes from the [2021-08-05 Working Draft](https://www.w3.org/TR/2021/WD-css-scrollbars-1-20210805/)

- Switched "should" to a "must" with regards to accessibility of narrow scrollbars. (see [Issue 6675](https://github.com/w3c/csswg-drafts/issues/6675))

### <a id="changes-since-2018-09-25"></a> Changes from the [2018-09-25 First Public Working Draft](https://www.w3.org/TR/2018/WD-css-scrollbars-1-20180925/)

- <a id="ref-for-propdef-color-scheme①"></a>

  <a id="ref-for-valdef-scrollbar-color-auto①"></a>

  <a id="ref-for-propdef-scrollbar-color③"></a>

  [\#6538](https://github.com/w3c/csswg-drafts/issues/6438): removed light and dark values of [scrollbar-color](#propdef-scrollbar-color) in favor of allowing the UA to tune [auto](#valdef-scrollbar-color-auto) in accordance with [color-scheme](https://www.w3.org/TR/css-color-adjust-1/#propdef-color-scheme) or other contextual information.

- [\#3237](https://github.com/w3c/csswg-drafts/issues/3237): scrollbar-color computed value changed to: specified keyword or two computed colors

- [\#4693](https://github.com/w3c/csswg-drafts/issues/4693): Clarified scope: styling scrollbar controls themselves, no layout or scrollability.

- [\#3315](https://github.com/w3c/csswg-drafts/issues/3315): More and updated accessibility considerations for scrollbar-color and scrollbar-width.

## <a id="security-privacy-considerations"></a>Appendix C. Considerations for Security and Privacy

This appendix is <em>non-normative</em>.

### <a id="security-considerations"></a>Considerations for Security

No specific concerns regarding security have been identified for this specification.

### <a id="privacy-considerations"></a>Considerations for Privacy

No specific concerns regarding privacy have been identified for this specification.

### <a id="security-privacy-self-review"></a>Self-review questionaire

Per the [Self-Review Questionnaire: Security and Privacy: Questions to Consider](https://www.w3.org/TR/security-privacy-questionnaire/#questions)

1.  Does this specification deal with personally-identifiable information?

    No.

2.  Does this specification deal with high-value data?

    No.

3.  Does this specification introduce new state for an origin that persists across browsing sessions?

    No.

4.  Does this specification expose persistent, cross-origin state to the web?

    No.

5.  Does this specification expose any other data to an origin that it doesn’t currently have access to?

    No.

6.  Does this specification enable new script execution/loading mechanisms?

    No.

7.  Does this specification allow an origin access to a user’s location?

    No.

8.  Does this specification allow an origin access to sensors on a user’s device?

    No.

9.  Does this specification allow an origin access to aspects of a user’s local computing environment?

    No.

10. Does this specification allow an origin access to other devices?

    No.

11. Does this specification allow an origin some measure of control over a user agent’s native UI?

    Yes. The scrollbar-\* properties enable the page to change the color and width of the scrollbar of the user agent’s native UI, e.g. scrollbars on the page’s window, on framed content embedded in the page, or on overflowing elements with scrollbars in the page.

12. Does this specification expose temporary identifiers to the web?

    No.

13. Does this specification distinguish between behavior in first-party and third-party contexts?

    No.

14. How should this specification work in the context of a user agent’s "incognito" mode?

    No differently.

15. Does this specification persist data to a user’s local device?

    No.

16. Does this specification have a "Security Considerations" and "Privacy Considerations" section?

    Yes.

17. Does this specification allow downgrading default security characteristics?

    No.

## <a id="accessibility-considerations"></a>Appendix D. Considerations for accessibility

This appendix is <em>non-normative</em>.

<a id="ref-for-propdef-scrollbar-width⑤"></a>

As noted [in the definition of the property](#scrollbar-width), authors need to be mindful of the accessibility implications of using [scrollbar-width: thin](#propdef-scrollbar-width). Scrollbars are a important piece of the user agent’s interface, and it is not appropriate for a web site author to change their size over aesthetic considerations. The property is available to support cases where the author wants to indicate that in a cramped area of the web page a thin scrollbar would be a more effective use of space. However, ultimately, the user, through their user agent, needs to have the last word on such things.

Using this property in such cases is preferable to authors building a custom thin-looking scrollbar in via script or proprietary extensions, because it does give the user the opportunity to override it.

<a id="ref-for-cascade-origin-user①"></a>

[User style sheets](https://www.w3.org/TR/css-cascade-5/#cascade-origin-user) do provide such an override, and additionally, user agents are encouraged to expose a setting letting users express that they do not want thin scrollbars to be used.

The CSS Working Group also acknowledges the needs of some users to have scrollbars that are wider than is typical. Operating systems and user agents can offer a means to let users express that preference, and in such cases, CSS will honor that choice.

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

- auto
  - [value for scrollbar-color](#valdef-scrollbar-color-auto), in § 2
  - [value for scrollbar-width](#valdef-scrollbar-width-auto), in § 3
- [\<color\>](#valdef-scrollbar-color-color), in § 2
- [none](#valdef-scrollbar-width-none), in § 3
- [scrollbar-color](#propdef-scrollbar-color), in § 2
- [scrollbar-width](#propdef-scrollbar-width), in § 3
- [thin](#valdef-scrollbar-width-thin), in § 3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-cascade-origin-user"></a>user style sheet
- \[css-color-4\] defines the following terms:
  - <a id="term-for-typedef-color"></a>\<color\>
- \[css-color-adjust-1\] defines the following terms:
  - <a id="term-for-propdef-color-scheme"></a>color-scheme
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-propdef-overflow"></a>overflow
  - <a id="term-for-overlay-scrollbars"></a>overlay scrollbars
  - <a id="term-for-scroll-container"></a>scroll container
- \[css-values-4\] defines the following terms:
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-mult-num"></a>{a}
  - <a id="term-for-comb-one"></a>\|
- \[HTML\] defines the following terms:
  - <a id="term-for-the-body-element"></a>body

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 1 June 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-color-adjust-1"></a>\[CSS-COLOR-ADJUST-1\]  
Elika Etemad; et al. [CSS Color Adjustment Module Level 1](https://www.w3.org/TR/css-color-adjust-1/). 16 June 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-adjust-1&#x2F;](https://www.w3.org/TR/css-color-adjust-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 2 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 16 October 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 15 October 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-wcag21"></a>\[WCAG21\]  
Andrew Kirkpatrick; et al. [Web Content Accessibility Guidelines (WCAG) 2.1](https://www.w3.org/TR/WCAG21/). 5 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;WCAG21&#x2F;](https://www.w3.org/TR/WCAG21/)

## <a id="property-index"></a>Property Index

<strong>Table 3 — structured row/cell transcription</strong>

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

<a id="ref-for-propdef-scrollbar-color④"></a>

[scrollbar-color](#propdef-scrollbar-color)

<strong>Column 2 (data cell):</strong>

auto \| \<color\>{2}

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword or two computed colors

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scrollbar-width⑥"></a>

[scrollbar-width](#propdef-scrollbar-width)

<strong>Column 2 (data cell):</strong>

auto \| thin \| none

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword
