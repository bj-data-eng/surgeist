Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Counter Styles Module Level 3](https://drafts.csswg.org/css-counter-styles-3/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply. The capture’s full original notice and links remain below.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md#cssom-planning-reference-refresh--2026-10-09).

# Source provenance

Title: CSS Counter Styles Module Level 3

Source snapshot: https://drafts.csswg.org/css-counter-styles-3/

Retrieved: 2026-10-09. The captured page states Editor’s Draft, 8 October 2026. An undated editor URL can change; the retrieval date and exact byte hash identify this captured HTML.

Captured HTML SHA-256: ef29d0a06a13683cbb3244bebc6f36d359f62fa107c17f1250111c9b0e4b0f43

Captured HTML revision metadata: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Representation notes:

- Full format conversion of the captured HTML body, including status, metadata, bibliography, indexes, examples, test references, and legal notice. Script and style elements are omitted and were not executed.
- All source body IDs are retained, including those inside preformatted blocks, which are relocated immediately before those blocks. Fragment-only links stay local only when their exact captured target exists; other links remain upstream.
- All 17 tables have readable Markdown layouts. 5 tables use explicit merged header paths and repeat spanning values in their applicable columns. Synthetic Field/Definition or Column N headings are non-normative. Source row headers remain bold; native HTML header/accessibility semantics are not expressible in GFM.
- Preformatted examples retain literal text. Single-line table examples use semantic inline code. Compiler-generated hyperlinks inside fenced code blocks are omitted while their IDs and visible code text are retained; other source links are preserved. Small semantic code, variable, emphasis, subscript, and superscript HTML remains where Markdown notation would alter text.
- Conversion uses Pandoc 3.1.11.1 with focused Python/lxml preparation and a semantic Lua filter; the parsed GFM output is checked against the captured HTML. The [conversion report](CONVERSION-REPORT.md#cssom-planning-reference-refresh--2026-10-09) records provenance and check boundaries.
- This full edition is the selected planning reference for the new CSSOM work. The [source-edition mapping](SOURCE-EDITIONS.md#cssom-planning-sources) distinguishes these planning selections from historical editions consumed by completed CSS work.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Counter Styles Module Level 3

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-counter-styles/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-counter-styles-3/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2017/CR-css-counter-styles-3-20171214/>

<https://www.w3.org/TR/2015/CR-css-counter-styles-3-20150611/>

<https://www.w3.org/TR/2014/WD-css-counter-styles-3-20140826/>

<https://www.w3.org/TR/2013/WD-css-counter-styles-3-20130718/>

<https://www.w3.org/TR/2013/WD-css-counter-styles-3-20130221/>

<https://www.w3.org/TR/2012/WD-css-counter-styles-3-20121009/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-counter-styles-3)

<strong>Editor:</strong>

[Tab Atkins Jr.](http://xanthir.com/contact/) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-counter-styles-3/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-counter-styles/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

This module introduces the <a id="ref-for-at-ruledef-counter-style"></a>[&#64;counter-style](#at-ruledef-counter-style) rule, which allows authors to define their own custom counter styles for use with CSS list-marker and generated-content counters [\[CSS-LISTS-3\]](#biblio-css-lists-3). It also predefines a set of common counter styles, including the ones present in CSS2 and CSS2.1.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-counter-styles” in the title, like this: “\[css-counter-styles\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-counter-styles%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

The following features are at-risk, and may be dropped during the CR period:

- the <a id="ref-for-typedef-image"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image) value in <a id="ref-for-typedef-symbol"></a>[\<symbol\>](#typedef-symbol)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](#intro)
2.  [2 Counter Styles](#counter-styles)
    1.  [2.1 Counter Style Names](#counter-style-names)
    2.  [2.2 Generating a Counter String](#stringify)
3.  [3 Defining Custom Counter Styles: the &#64;counter-style rule](#the-counter-style-rule)
    1.  [3.1 Counter algorithms: the system descriptor](#counter-style-system)
        1.  [3.1.1 Cycling Symbols: the cyclic system](#cyclic-system)
        2.  [3.1.2 Exhaustible Symbols: the fixed system](#fixed-system)
        3.  [3.1.3 Repeating Symbols: the symbolic system](#symbolic-system)
        4.  [3.1.4 Bijective Numerals: the alphabetic system](#alphabetic-system)
        5.  [3.1.5 Positional Numerals: the numeric system](#numeric-system)
        6.  [3.1.6 Accumulating Numerals: the additive system](#additive-system)
        7.  [3.1.7 Building from Existing Counter Styles: the extends system ](#extends-system)
    2.  [3.2 Formatting negative values: the negative descriptor](#counter-style-negative)
    3.  [3.3 Symbols before the marker: the prefix descriptor](#counter-style-prefix)
    4.  [3.4 Symbols after the marker: the suffix descriptor](#counter-style-suffix)
    5.  [3.5 Limiting the counter scope: the range descriptor](#counter-style-range)
    6.  [3.6 Zero-Padding and Constant-Width Representations: the pad descriptor](#counter-style-pad)
    7.  [3.7 Defining fallback: the fallback descriptor](#counter-style-fallback)
    8.  [3.8 Marker characters: the symbols and additive-symbols descriptors](#counter-style-symbols)
    9.  [3.9 Speech Synthesis: the speak-as descriptor](#counter-style-speak-as)
4.  [4 Defining Anonymous Counter Styles: the symbols() function](#symbols-function)
5.  [5 Extending list-style-type, counter(), and counters()](#extending-css2)
6.  [6 Simple Predefined Counter Styles](#predefined-counters)
    1.  [6.1 Numeric: decimal, decimal-leading-zero, arabic-indic, armenian, upper-armenian, lower-armenian, bengali, cambodian, khmer, cjk-decimal, devanagari, georgian, gujarati, gurmukhi, hebrew, kannada, lao, malayalam, mongolian, myanmar, oriya, persian, lower-roman, upper-roman, tamil, telugu, thai, tibetan](#simple-numeric)
    2.  [6.2 Alphabetic: lower-alpha, lower-latin, upper-alpha, upper-latin, lower-greek, hiragana, hiragana-iroha, katakana, katakana-iroha](#simple-alphabetic)
    3.  [6.3 Symbolic: disc, circle, square, disclosure-open, disclosure-closed](#simple-symbolic)
    4.  [6.4 Fixed: cjk-earthly-branch, cjk-heavenly-stem](#simple-fixed)
7.  [7 Complex Predefined Counter Styles](#complex-predefined-counters)
    1.  [7.1 Longhand East Asian Counter Styles](#complex-cjk)
        1.  [7.1.1 Limited-range Implementation (required)](#limited-range-required)
            1.  [7.1.1.1 Japanese: japanese-informal and japanese-formal](#limited-japanese)
            2.  [7.1.1.2 Korean: korean-hangul-formal, korean-hanja-informal, and korean-hanja-formal](#limited-korean)
            3.  [7.1.1.3 Chinese: simp-chinese-informal, simp-chinese-formal, trad-chinese-informal, and trad-chinese-formal](#limited-chinese)
        2.  [7.1.2 Extended Implementation (optional)](#extended-range-optional)
    2.  [7.2 Ethiopic Numeric Counter Style: ethiopic-numeric](#ethiopic-numeric-counter-style)
8.  [8 Additional “Ready-made” Counter Styles](#additional-predefined)
9.  [9 APIs](#apis)
    1.  [9.1 Extensions to the <code>CSSRule</code> interface](#extensions-to-cssrule-interface)
    2.  [9.2 The <code>CSSCounterStyleRule</code> interface](#the-csscounterstylerule-interface)
10. [10 Sample style sheet for HTML](#ua-stylesheet)
11. [ Changes](#changes)
    1.  [ Changes since the July 2021 Candidate Recommendation](#changes-2021)
    2.  [ Changes since the December 2017 Candidate Recommendation](#changes-2017)
    3.  [ Changes since the June 2015 Candidate Recommendation](#changes-jun-2015)
    4.  [ Changes since the Feb 2015 Candidate Recommendation](#changes-feb-2015)
12. [ Acknowledgments](#acknowledgments)
13. [ Privacy Considerations](#privacy)
14. [ Security Considerations](#security)
15. [ Conformance](#w3c-conformance)
    1.  [ Document conventions](#w3c-conventions)
    2.  [ Conformance classes](#w3c-conformance-classes)
    3.  [ Partial implementations](#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](#w3c-testing)
16. [ Index](#index)
    1.  [ Terms defined by this specification](#index-defined-here)
    2.  [ Terms defined by reference](#index-defined-elsewhere)
17. [ References](#references)
    1.  [ Normative References](#normative)
    2.  [ Non-Normative References](#informative)
18. [ Property Index](#property-index)
    1.  [ &#64;counter-style Descriptors](#counter-style-descriptor-table)
19. [ IDL Index](#idl-index)

## <a id="intro"></a>1.  Introduction[](#intro)

CSS 1 defined a handful of useful counter styles based on the styles that HTML traditionally allowed on ordered and unordered lists. While this was expanded slightly by CSS2.1, it doesn’t address the needs of worldwide typography.

This module introduces the <a id="ref-for-at-ruledef-counter-style①"></a>[&#64;counter-style](#at-ruledef-counter-style) rule which allows CSS to address this in an open-ended manner, by allowing the author to define their own counter styles. These styles can then be used in the <a id="ref-for-propdef-list-style-type"></a>[list-style-type](https://drafts.csswg.org/css-lists-3/#propdef-list-style-type) property or in the <a id="ref-for-funcdef-counter"></a>[counter()](https://drafts.csswg.org/css-lists-3/#funcdef-counter) and <a id="ref-for-funcdef-counters"></a>[counters()](https://drafts.csswg.org/css-lists-3/#funcdef-counters) functions. It also defines some additional predefined counter styles, particularly ones which are common but complicated to represent with <a id="ref-for-at-ruledef-counter-style②"></a>&#64;counter-style.

## <a id="counter-styles"></a>2.  Counter Styles[](#counter-styles)

A <a id="counter-style"></a><strong>counter style</strong> defines how to convert a counter value into a string. Counter styles are composed of:

- a name, to identify the style
- an algorithm, which transforms integer counter values into a basic string representation
- a negative sign, which is prepended or appended to the representation of a negative counter value.
- a prefix, to prepend to the representation
- a suffix to append to the representation
- a range, which limits the values that a counter style handles
- a spoken form, which describes how to read out the counter style in a speech synthesizer
- and a fallback style, to render the representation with when the counter value is outside the counter style’s range or the counter style otherwise can’t render the counter value

### <a id="counter-style-names"></a>2.1. Counter Style Names[](#counter-style-names)

<a id="ref-for-counter-style"></a>[Counter styles](#counter-style) are referenced by name, and represented syntactically by <a id="typedef-counter-style-name"></a><strong>\<counter-style-name\></strong>, which is a <a id="ref-for-identifier-value"></a>[\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) specifying a <a id="ref-for-css-tree-scoped-name"></a>[tree-scoped name](https://drafts.csswg.org/css-shadow-1/#css-tree-scoped-name). The keyword <a id="ref-for-valdef-list-style-type-none"></a>[none](https://drafts.csswg.org/css-lists-3/#valdef-list-style-type-none) (including any <a id="ref-for-ascii-case-insensitive"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) permutation of it) is not a valid <a id="ref-for-typedef-counter-style-name"></a>[\<counter-style-name\>](#typedef-counter-style-name).

Tests

- [name-syntax.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/name-syntax.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/name-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/name-syntax.html)

Note: Note that <a id="ref-for-identifier-value①"></a>[\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) also automatically excludes the <a id="ref-for-css-wide-keywords"></a>[CSS-wide keywords](https://drafts.csswg.org/css-values-4/#css-wide-keywords). In addition, some names, like <a id="ref-for-valdef-list-style-position-inside"></a>[inside](https://drafts.csswg.org/css-lists-3/#valdef-list-style-position-inside), are valid as counter style names, but conflict with the existing values of properties like <a id="ref-for-propdef-list-style"></a>[list-style](https://drafts.csswg.org/css-lists-3/#propdef-list-style), and so won’t be usable there.

Counter style names are generally case-sensitive. However, the names defined in this specification are ASCII lowercased on parse wherever they are used as counter styles, e.g. in the <a id="ref-for-propdef-list-style①"></a>[list-style](https://drafts.csswg.org/css-lists-3/#propdef-list-style) set of properties, in the <a id="ref-for-at-ruledef-counter-style③"></a>[&#64;counter-style](#at-ruledef-counter-style) rule, and in the <a id="ref-for-funcdef-counter①"></a>[counter()](https://drafts.csswg.org/css-lists-3/#funcdef-counter) functions.

Tests

- [name-case-sensitivity.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/name-case-sensitivity.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/name-case-sensitivity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/name-case-sensitivity.html)

### <a id="stringify"></a>2.2.  Generating a Counter String[](#stringify)

When asked to <a id="generate-a-counter"></a><strong>generate a counter representation</strong> using a particular counter style for a particular counter value, follow these steps:

1.  If the counter style is unknown, exit this algorithm and instead <a id="ref-for-generate-a-counter"></a>[generate a counter representation](#generate-a-counter) using the <a id="ref-for-decimal"></a>[decimal](#decimal) style and the same counter value.
2.  If the counter value is outside the <a id="ref-for-descdef-counter-style-range"></a>[range](#descdef-counter-style-range) of the counter style, exit this algorithm and instead <a id="ref-for-generate-a-counter①"></a>[generate a counter representation](#generate-a-counter) using the counter style’s fallback style and the same counter value.
3.  Using the counter value and the counter algorithm for the counter style, generate an <a id="initial-representation-for-the-counter-value"></a><strong>initial representation for the counter value</strong>. If the counter value is negative and the counter style <a id="ref-for-use-a-negative-sign"></a>[uses a negative sign](#use-a-negative-sign), instead generate an initial representation using the absolute value of the counter value.
4.  Prepend symbols to the representation as specified in the <a id="ref-for-descdef-counter-style-pad"></a>[pad](#descdef-counter-style-pad) descriptor.
5.  If the counter value is negative and the counter style <a id="ref-for-use-a-negative-sign①"></a>[uses a negative sign](#use-a-negative-sign), wrap the representation in the counter style’s negative sign as specified in the <a id="ref-for-descdef-counter-style-negative"></a>[negative](#descdef-counter-style-negative) descriptor.
6.  Return the representation.

Tests

General tests for counter styles

- [descriptor-calc.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-calc.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-calc.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-calc.html)
- [counter-suffix.html](https://wpt.fyi/results/css/css-counter-styles/counter-suffix.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-suffix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-suffix.html)
- [idlharness.html](https://wpt.fyi/results/css/css-counter-styles/idlharness.html) [(live test)](http://wpt.live/css/css-counter-styles/idlharness.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/idlharness.html)

------------------------------------------------------------------------

Note: <a id="ref-for-descdef-counter-style-prefix"></a>[prefix](#descdef-counter-style-prefix) and <a id="ref-for-descdef-counter-style-suffix"></a>[suffix](#descdef-counter-style-suffix) don’t play a part in this algorithm. This is intentional; the prefix and suffix aren’t part of the string returned by the counter() or counters() functions. Instead, the prefix and suffix are added by the algorithm that constructs the value of the <a id="ref-for-propdef-content"></a>[content](https://drafts.csswg.org/css-content-3/#propdef-content) property for the <a id="ref-for-selectordef-marker"></a>[::marker](https://drafts.csswg.org/css-pseudo-4/#selectordef-marker) pseudo-element. This also implies that the prefix and suffix always come from the specified counter-style, even if the actual representation is constructed by a fallback style.

Some values of <a id="ref-for-descdef-counter-style-system"></a>[system](#descdef-counter-style-system) (<a id="ref-for-valdef-system-symbolic"></a>[symbolic](#valdef-system-symbolic), <a id="ref-for-valdef-counter-style-system-additive"></a>[additive](#valdef-counter-style-system-additive)) and some descriptors (<a id="ref-for-descdef-counter-style-pad①"></a>[pad](#descdef-counter-style-pad)) can generate representations with size linear to an author-supplied number. This can potentially be abused to generate excessively large representations and consume undue amounts of the user’s memory or even hang their browser. User agents must support representations at least 60 Unicode codepoints long, but they may choose to instead use the fallback style for representations that would be longer than 60 codepoints.

## <a id="the-counter-style-rule"></a>3.  Defining Custom Counter Styles: the <a id="ref-for-at-ruledef-counter-style④"></a>[&#64;counter-style](#at-ruledef-counter-style) rule[](#the-counter-style-rule)

The <a id="at-ruledef-counter-style"></a><strong>&#64;counter-style</strong> rule allows authors to define a custom <a id="ref-for-counter-style①"></a>[counter style](#counter-style). The components of a <a id="ref-for-counter-style②"></a>counter style are specified by descriptors in the <a id="ref-for-at-ruledef-counter-style⑤"></a>[&#64;counter-style](#at-ruledef-counter-style) rule. The algorithm is specified implicitly by a combination of the system, symbols, and additive-symbols properties.

The general form of an <a id="ref-for-at-ruledef-counter-style⑥"></a>[&#64;counter-style](#at-ruledef-counter-style) rule is:

<a id="ref-for-typedef-counter-style-name①"></a><a id="ref-for-typedef-declaration-list"></a>

``` text
@counter-style <counter-style-name> { <declaration-list> }
```

Tests

- [counter-name-case-sensitive.html](https://wpt.fyi/results/css/css-counter-styles/counter-name-case-sensitive.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-name-case-sensitive.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-name-case-sensitive.html)
- [access-from-shadow-dom.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/access-from-shadow-dom.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/access-from-shadow-dom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/access-from-shadow-dom.html)
- [override-in-shadow-dom.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/override-in-shadow-dom.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/override-in-shadow-dom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/override-in-shadow-dom.html)
- [redefine-attr-mapping.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/redefine-attr-mapping.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/redefine-attr-mapping.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/redefine-attr-mapping.html)
- [redefine-builtin.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/redefine-builtin.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/redefine-builtin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/redefine-builtin.html)
- [shadow-dom-part.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/shadow-dom-part.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/shadow-dom-part.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/shadow-dom-part.html)

Certain <a id="non-overridable-counter-styles"></a><strong>non-overridable counter-styles</strong>—​specifically, <a id="ref-for-decimal①"></a>[decimal](#decimal) or any of the <a id="ref-for-predefined-symbolic-counter-style"></a>[predefined symbolic counter styles](#predefined-symbolic-counter-style) (<a id="ref-for-typedef-symbolic-glyph"></a>[\<symbolic-glyph\>](#typedef-symbolic-glyph))—​cannot be defined by an <a id="ref-for-at-ruledef-counter-style⑦"></a>[&#64;counter-style](#at-ruledef-counter-style) rule; any rule that attempts to do so is invalid and ignored.

Each <a id="ref-for-at-ruledef-counter-style⑧"></a>[&#64;counter-style](#at-ruledef-counter-style) rule specifies a value for every counter-style descriptor, either implicitly or explicitly. Those not given explicit value in the rule take the initial value listed with each descriptor in this specification. These descriptors apply solely within the context of the <a id="ref-for-at-ruledef-counter-style⑨"></a>&#64;counter-style rule in which they are defined, and do not apply to document language elements. There is no notion of which elements the descriptors apply to or whether the values are inherited by child elements. When a given descriptor occurs multiple times in a given <a id="ref-for-at-ruledef-counter-style①⓪"></a>&#64;counter-style rule, only the last-specified valid value is used; all prior values for that descriptor must be ignored.

Defining a <a id="ref-for-at-ruledef-counter-style①①"></a>[&#64;counter-style](#at-ruledef-counter-style) makes it available to the entire document in which it is included. If multiple <a id="ref-for-at-ruledef-counter-style①②"></a>&#64;counter-style rules are defined with the same name, only one wins, according to standard cascade rules. <a id="ref-for-at-ruledef-counter-style①③"></a>&#64;counter-style rules cascade "atomically": if one replaces another of the same name, it replaces it <em>entirely</em>, rather than just replacing the specific descriptors it specifies.

Note: Note that even the predefined counter styles can be overridden; the UA stylesheet occurs before any other stylesheets, so the predefined ones always lose in the cascade.

This at-rule conforms with the forward-compatible parsing requirement of CSS; conformant parsers that don’t understand these rules will ignore them without error. Any descriptors that are not recognized or implemented by a given user agent, or whose value does not match the grammars given here or in a future version of this specification, must be ignored in their entirety; they do not make the <a id="ref-for-at-ruledef-counter-style①④"></a>[&#64;counter-style](#at-ruledef-counter-style) rule invalid.

### <a id="counter-style-system"></a>3.1.  Counter algorithms: the <a id="ref-for-descdef-counter-style-system①"></a>[system](#descdef-counter-style-system) descriptor[](#counter-style-system)

| Field                     | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
|---------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-counter-style-system"></a><strong>system</strong>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-counter-style①⑤"></a>[&#64;counter-style](#at-ruledef-counter-style)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Value:</strong>   | cyclic <a id="ref-for-comb-one"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) numeric <a id="ref-for-comb-one①"></a>\| alphabetic <a id="ref-for-comb-one②"></a>\| symbolic <a id="ref-for-comb-one③"></a>\| additive <a id="ref-for-comb-one④"></a>\| \[fixed <a id="ref-for-integer-value"></a>[\<integer\>](https://drafts.csswg.org/css-values-4/#integer-value)<a id="ref-for-mult-opt"></a>[?](https://drafts.csswg.org/css-values-4/#mult-opt)\] <a id="ref-for-comb-one⑤"></a>\| \[ extends <a id="ref-for-typedef-counter-style-name②"></a>[\<counter-style-name\>](#typedef-counter-style-name) \] |
| <strong>Initial:</strong> | symbolic                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |

Tests

- [system-syntax.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-syntax.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-syntax.html)

The <a id="ref-for-descdef-counter-style-system②"></a>[system](#descdef-counter-style-system) descriptor specifies which algorithm will be used to construct the counter’s representation based on the counter value. For example, <a id="ref-for-valdef-counter-style-system-cyclic"></a>[cyclic](#valdef-counter-style-system-cyclic) counter styles just cycle through their symbols repeatedly, while <a id="ref-for-valdef-counter-style-system-numeric"></a>[numeric](#valdef-counter-style-system-numeric) counter styles interpret their symbols as digits and build their representation accordingly. The systems are defined in the following subsections.

Each <a id="ref-for-descdef-counter-style-system③"></a>[system](#descdef-counter-style-system) value is associated with either the <a id="ref-for-descdef-counter-style-symbols"></a>[symbols](#descdef-counter-style-symbols) or <a id="ref-for-descdef-counter-style-additive-symbols"></a>[additive-symbols](#descdef-counter-style-additive-symbols) descriptors, and has a minimum length that the appropriate descriptor must have; each entry below defines what this is. If a <a id="ref-for-at-ruledef-counter-style①⑥"></a>[&#64;counter-style](#at-ruledef-counter-style) rule fails to meet this requirement, it does not define a <a id="ref-for-counter-style③"></a>[counter style](#counter-style). (The rule is still syntactically valid, but has no effect.)

#### <a id="cyclic-system"></a>3.1.1.  Cycling Symbols: the <a id="ref-for-valdef-counter-style-system-cyclic①"></a>[cyclic](#valdef-counter-style-system-cyclic) system[](#cyclic-system)

The <a id="valdef-counter-style-system-cyclic"></a><strong>cyclic</strong> counter system cycles repeatedly through its provided symbols, looping back to the beginning when it reaches the end of the list. It can be used for simple bullets (just provide a single <a id="ref-for-counter-symbol"></a>[counter symbol](#counter-symbol)), or for cycling through multiple symbols. The first <a id="ref-for-counter-symbol①"></a>counter symbol is used as the representation of the value 1, the second <a id="ref-for-counter-symbol②"></a>counter symbol (if it exists) is used as the representation of the value 2, etc.

If the system is <a id="ref-for-valdef-counter-style-system-cyclic②"></a>[cyclic](#valdef-counter-style-system-cyclic), the <a id="ref-for-descdef-counter-style-symbols①"></a>[symbols](#descdef-counter-style-symbols) descriptor must contain at least one <a id="ref-for-counter-symbol③"></a>[counter symbol](#counter-symbol), otherwise the rule does not define a counter style (but is still a valid rule). This system is defined over all counter values.

<a id="example-bb402b2d"></a>

<strong>Example:</strong>

[](#example-bb402b2d) A "triangle bullet" counter style can be defined as:

<a id="triangle"></a>

``` text
@counter-style triangle {
  system: cyclic;
  symbols: ‣;
  suffix: " ";
}
```

It will then produce lists that look like:

``` text
‣  One
‣  Two
‣  Three
```

If there are <var>N</var> <a id="ref-for-counter-symbol④"></a>[counter symbols](#counter-symbol) and a representation is being constructed for the integer <var>value</var>, the representation is the <a id="ref-for-counter-symbol⑤"></a>counter symbol at index ( (<var>value</var>-1) mod <var>N</var>) of the list of <a id="ref-for-counter-symbol⑥"></a>counter symbols (0-indexed).

Tests

- [system-cyclic.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-cyclic.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-cyclic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-cyclic.html)
- [system-cyclic-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-cyclic-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-cyclic-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-cyclic-invalid.html)

#### <a id="fixed-system"></a>3.1.2.  Exhaustible Symbols: the <a id="ref-for-valdef-counter-style-system-fixed"></a>[fixed](#valdef-counter-style-system-fixed) system[](#fixed-system)

The <a id="valdef-counter-style-system-fixed"></a><strong>fixed</strong> counter system runs through its list of counter symbols once, then falls back. It is useful for representing counter styles that only have a finite number of representations. For example, Unicode defines several limited-length runs of special characters meant for lists, such as circled digits.

If the system is <a id="ref-for-valdef-counter-style-system-fixed①"></a>[fixed](#valdef-counter-style-system-fixed), the <a id="ref-for-descdef-counter-style-symbols②"></a>[symbols](#descdef-counter-style-symbols) descriptor must contain at least one <a id="ref-for-counter-symbol⑦"></a>[counter symbol](#counter-symbol), otherwise the rule does not define a counter style (but is still a valid rule). This system is defined over counter values in a finite range, starting with the <a id="ref-for-first-symbol-value"></a>[first symbol value](#first-symbol-value) and having a length equal to the length of the list of <a id="ref-for-counter-symbol⑧"></a>counter symbols.

When this system is specified, it may optionally have an integer provided after it, which sets the <a id="first-symbol-value"></a><strong>first symbol value</strong>. If it is omitted, the <a id="ref-for-first-symbol-value①"></a>[first symbol value](#first-symbol-value) is 1.

<a id="example-24b2a03a"></a>

<strong>Example:</strong>

[](#example-24b2a03a) A "box-corner" counter style can be defined as:

<a id="box-corner"></a>

``` text
@counter-style box-corner {
  system: fixed;
  symbols: ◰ ◳ ◲ ◱;
  suffix: ': ';
}
```

It will then produce lists that look like:

``` text
◰:  One
◳:  Two
◲:  Three
◱:  Four
5:  Five
6:  Six
```

The first <a id="ref-for-counter-symbol⑨"></a>[counter symbol](#counter-symbol) is the representation for the <a id="ref-for-first-symbol-value②"></a>[first symbol value](#first-symbol-value), and subsequent counter values are represented by subsequent <a id="ref-for-counter-symbol①⓪"></a>counter symbols. Once the list of <a id="ref-for-counter-symbol①①"></a>counter symbols is exhausted, further values cannot be represented by this counter style, and must instead be represented by the fallback counter style.

Tests

- [system-fixed.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-fixed.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-fixed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-fixed.html)
- [system-fixed-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-fixed-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-fixed-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-fixed-invalid.html)

#### <a id="symbolic-system"></a>3.1.3.  Repeating Symbols: the <a id="ref-for-valdef-system-symbolic①"></a>[symbolic](#valdef-system-symbolic) system[](#symbolic-system)

The <a id="valdef-system-symbolic"></a><strong>symbolic</strong> counter system cycles repeatedly through its provided symbols, doubling, tripling, etc. the symbols on each successive pass through the list. For example, if the original symbols were "\*" and "†", then on the second pass they would instead be "\*\*" and "††", while on the third they would be "\*\*\*"and "†††", etc. It can be used for footnote-style markers, and is also sometimes used for alphabetic-style lists for a slightly different presentation than what the <a id="ref-for-valdef-counter-style-system-alphabetic"></a>[alphabetic](#valdef-counter-style-system-alphabetic) system presents.

If the system is <a id="ref-for-valdef-system-symbolic②"></a>[symbolic](#valdef-system-symbolic), the <a id="ref-for-descdef-counter-style-symbols③"></a>[symbols](#descdef-counter-style-symbols) descriptor must contain at least one <a id="ref-for-counter-symbol①②"></a>[counter symbol](#counter-symbol), otherwise the rule does not define a counter style (but is still a valid rule). This system is defined only over strictly positive counter values.

<a id="example-a76d671e"></a>

<strong>Example:</strong>

[](#example-a76d671e) An "footnote" counter style can be defined as:

<a id="footnote"></a>

``` text
@counter-style footnote {
  system: symbolic;
  symbols: '*' ⁑ † ‡;
  suffix: " ";
}
```

It will then produce lists that look like:

``` text
*   One
⁑   Two
†   Three
‡   Four
**  Five
⁑⁑  Six
```

<a id="example-5b0bbd83"></a>

<strong>Example:</strong>

[](#example-5b0bbd83) Some style guides mandate a list numbering that looks similar to <a id="ref-for-upper-alpha"></a>[upper-alpha](#upper-alpha), but repeats differently after the first 26 values, instead going "AA", "BB", "CC", etc. This can be achieved with the symbolic system:

<a id="upper-alpha-legal"></a>

``` text
@counter-style upper-alpha-legal {
  system: symbolic;
  symbols: A B C D E F G H I J K L M
           N O P Q R S T U V W X Y Z;
}
```

This style is identical to <a id="ref-for-upper-alpha①"></a>[upper-alpha](#upper-alpha) through the first 27 values, but they diverge after that, with <a id="ref-for-upper-alpha②"></a>upper-alpha going "AB", "AC", "AD", etc. Starting at the 53rd value, <a id="ref-for-upper-alpha③"></a>upper-alpha goes "BA", "BB", "BC", etc., while this style jumps into triple digits with "AAA", "BBB", "CCC", etc.

To construct the representation, run the following algorithm:

Let <var>N</var> be the length of the list of <a id="ref-for-counter-symbol①③"></a>[counter symbols](#counter-symbol), <var>value</var> initially be the counter value, <var>S</var> initially be the empty string, and <var>symbol(n)</var> be the nth <a id="ref-for-counter-symbol①④"></a>counter symbol in the list of <a id="ref-for-counter-symbol①⑤"></a>counter symbols (0-indexed).

1.  Let the <var>chosen symbol</var> be <code>symbol( (<var>value</var> - 1) mod <var>N</var>)</code>.
2.  Let the <var>representation length</var> be <code>ceil( <var>value</var> / <var>N</var> )</code>.
3.  Append the <var>chosen symbol</var> to <var>S</var> a number of times equal to the <var>representation length</var>.

Finally, return <var>S</var>.

Tests

- [system-symbolic.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-symbolic.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-symbolic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-symbolic.html)
- [system-symbolic-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-symbolic-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-symbolic-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-symbolic-invalid.html)

#### <a id="alphabetic-system"></a>3.1.4.  Bijective Numerals: the <a id="ref-for-valdef-counter-style-system-alphabetic①"></a>[alphabetic](#valdef-counter-style-system-alphabetic) system[](#alphabetic-system)

The <a id="valdef-counter-style-system-alphabetic"></a><strong>alphabetic</strong> counter system interprets the list of <a id="ref-for-counter-symbol①⑥"></a>[counter symbols](#counter-symbol) as digits to an <em>alphabetic</em> numbering system, similar to the default <a id="ref-for-lower-alpha"></a>[lower-alpha](#lower-alpha) counter style, which wraps from "a", "b", "c", to "aa", "ab", "ac". Alphabetic numbering systems do not contain a digit representing 0; so the first value when a new digit is added is composed solely of the first digit. Alphabetic numbering systems are commonly used for lists, and also appear in many spreadsheet programs to number columns. The first <a id="ref-for-counter-symbol①⑦"></a>counter symbol in the list is interpreted as the digit 1, the second as the digit 2, and so on.

If the system is <a id="ref-for-valdef-counter-style-system-alphabetic②"></a>[alphabetic](#valdef-counter-style-system-alphabetic), the <a id="ref-for-descdef-counter-style-symbols④"></a>[symbols](#descdef-counter-style-symbols) descriptor must contain at least two <a id="ref-for-counter-symbol①⑧"></a>[counter symbols](#counter-symbol), otherwise the rule does not define a counter style (but is still a valid rule). This system is defined only over strictly positive counter values.

<a id="example-8007bc6e"></a>

<strong>Example:</strong>

[](#example-8007bc6e) A counter style using go stones can be defined as:

<a id="go"></a>

``` text
@counter-style go {
  system: alphabetic;
  symbols: url(white.svg) url(black.svg);
  suffix: " ";
}
```

It will then produce lists that look like:

![](https://drafts.csswg.org/css-counter-styles-3/images/white.svg) One  
![](https://drafts.csswg.org/css-counter-styles-3/images/black.svg) Two  
![](https://drafts.csswg.org/css-counter-styles-3/images/white.svg)![](https://drafts.csswg.org/css-counter-styles-3/images/white.svg) Three  
![](https://drafts.csswg.org/css-counter-styles-3/images/white.svg)![](https://drafts.csswg.org/css-counter-styles-3/images/black.svg) Four  
![](https://drafts.csswg.org/css-counter-styles-3/images/black.svg)![](https://drafts.csswg.org/css-counter-styles-3/images/white.svg) Five  
![](https://drafts.csswg.org/css-counter-styles-3/images/black.svg)![](https://drafts.csswg.org/css-counter-styles-3/images/black.svg) Six  
![](https://drafts.csswg.org/css-counter-styles-3/images/white.svg)![](https://drafts.csswg.org/css-counter-styles-3/images/white.svg)![](https://drafts.csswg.org/css-counter-styles-3/images/white.svg) Seven

Note: This example requires support for SVG images to display correctly.

If there are <var>N</var> <a id="ref-for-counter-symbol①⑨"></a>[counter symbols](#counter-symbol), the representation is a base <var>N</var> alphabetic number using the <a id="ref-for-counter-symbol②⓪"></a>counter symbols as digits. To construct the representation, run the following algorithm:

Let <var>N</var> be the length of the list of <a id="ref-for-counter-symbol②①"></a>[counter symbols](#counter-symbol), <var>value</var> initially be the counter value, <var>S</var> initially be the empty string, and <var>symbol(n)</var> be the nth <a id="ref-for-counter-symbol②②"></a>counter symbol in the list of <a id="ref-for-counter-symbol②③"></a>counter symbols (0-indexed).

While <var>value</var> is not equal to 0:

1.  Set <var>value</var> to <code><var>value</var> - 1</code>.
2.  Prepend <var>symbol( <var>value</var> mod <var>N</var> )</var> to <var>S</var>.
3.  Set <var>value</var> to <code>floor( <var>value</var> / <var>N</var> )</code>.

Finally, return <var>S</var>.

Tests

- [system-alphabetic.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-alphabetic.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-alphabetic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-alphabetic.html)
- [system-alphabetic-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-alphabetic-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-alphabetic-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-alphabetic-invalid.html)

#### <a id="numeric-system"></a>3.1.5.  Positional Numerals: the <a id="ref-for-valdef-counter-style-system-numeric①"></a>[numeric](#valdef-counter-style-system-numeric) system[](#numeric-system)

The <a id="valdef-counter-style-system-numeric"></a><strong>numeric</strong> counter system interprets the list of <a id="ref-for-counter-symbol②④"></a>[counter symbols](#counter-symbol) as digits to a "place-value" numbering system, similar to the default <a id="ref-for-decimal②"></a>[decimal](#decimal) counter style. The first <a id="ref-for-counter-symbol②⑤"></a>counter symbol in the list is interpreted as the digit 0, the second as the digit 1, and so on.

If the system is <a id="ref-for-valdef-counter-style-system-numeric②"></a>[numeric](#valdef-counter-style-system-numeric), the <a id="ref-for-descdef-counter-style-symbols⑤"></a>[symbols](#descdef-counter-style-symbols) descriptor must contain at least two <a id="ref-for-counter-symbol②⑥"></a>[counter symbols](#counter-symbol), otherwise the rule does not define a counter style (but is still a valid rule). This system is defined over all counter values.

<a id="example-95bf6e12"></a>

<strong>Example:</strong>

[](#example-95bf6e12) A "trinary" counter style can be defined as:

<a id="trinary"></a>

``` text
@counter-style trinary {
  system: numeric;
  symbols: '0' '1' '2';
}
```

It will then produce lists that look like:

``` text
1.   One
2.   Two
10.  Three
11.  Four
12.  Five
20.  Six
```

If there are <var>N</var> <a id="ref-for-counter-symbol②⑦"></a>[counter symbols](#counter-symbol), the representation is a base <var>N</var> number using the <a id="ref-for-counter-symbol②⑧"></a>counter symbols as digits. To construct the representation, run the following algorithm:

Let <var>N</var> be the length of the list of <a id="ref-for-counter-symbol②⑨"></a>[counter symbols](#counter-symbol), <var>value</var> initially be the counter value, <var>S</var> initially be the empty string, and <var>symbol(n)</var> be the nth <a id="ref-for-counter-symbol③⓪"></a>counter symbol in the list of <a id="ref-for-counter-symbol③①"></a>counter symbols (0-indexed).

1.  If <var>value</var> is 0, append <code>symbol(0)</code> to <var>S</var> and return <var>S</var>.
2.  While <var>value</var> is not equal to 0:
    1.  Prepend <var>symbol( <var>value</var> mod <var>N</var> )</var> to <var>S</var>.
    2.  Set <var>value</var> to <code>floor( <var>value</var> / <var>N</var> )</code>.
3.  Return <var>S</var>.

Tests

- [system-numeric.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-numeric.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-numeric.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-numeric.html)
- [system-numeric-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-numeric-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-numeric-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-numeric-invalid.html)

#### <a id="additive-system"></a>3.1.6.  Accumulating Numerals: the <a id="ref-for-valdef-counter-style-system-additive①"></a>[additive](#valdef-counter-style-system-additive) system[](#additive-system)

The <a id="valdef-counter-style-system-additive"></a><strong>additive</strong> counter system is used to represent "sign-value" numbering systems, which, rather than reusing digits in different positions to change their value, define additional digits with much larger values, so that the value of the number can be obtained by adding all the digits together. This is used in Roman numerals and other numbering systems around the world.

If the system is <a id="ref-for-valdef-counter-style-system-additive②"></a>[additive](#valdef-counter-style-system-additive), the <a id="ref-for-descdef-counter-style-additive-symbols①"></a>[additive-symbols](#descdef-counter-style-additive-symbols) descriptor must contain at least one <a id="ref-for-additive-tuple"></a>[additive tuple](#additive-tuple), otherwise the rule does not define a counter style (but is still a valid rule). This system is nominally defined over all counter values (see algorithm, below, for exact details).

<a id="example-ffca8577"></a>

<strong>Example:</strong>

[](#example-ffca8577) A "dice" counter style can be defined as:

<a id="dice"></a>

``` text
@counter-style dice {
  system: additive;
  additive-symbols: 6 ⚅, 5 ⚄, 4 ⚃, 3 ⚂, 2 ⚁, 1 ⚀;
  suffix: " ";
}
```

It will then produce lists that look like:

``` text
  ⚀  One
  ⚁  Two
  ⚂  Three
...
 ⚅⚄  Eleven
 ⚅⚅  Twelve
⚅⚅⚀  Thirteen
```

To construct the representation:

1.  Let <var>value</var> initially be the counter value, <var>S</var> initially be the empty string, and <var>symbol list</var> initially be the list of <a id="ref-for-additive-tuple①"></a>[additive tuples](#additive-tuple).

2.  If <var>value</var> is zero:

    1.  If <var>symbol list</var> contains a tuple with a weight of zero, append that tuple’s <a id="ref-for-counter-symbol③②"></a>[counter symbol](#counter-symbol) to <var>S</var> and return <var>S</var>.

    2.  Otherwise, the given counter value cannot be represented by this counter style, and must instead be represented by the fallback counter style.

3.  For each <var>tuple</var> in <var>symbol list</var>:

    1.  Let <var>symbol</var> and <var>weight</var> be <var>tuple</var>’s <a id="ref-for-counter-symbol③③"></a>[counter symbol](#counter-symbol) and weight, respectively.

    2.  If <var>weight</var> is zero, or <var>weight</var> is greater than <var>value</var>, <a id="ref-for-iteration-continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    3.  Let <var>reps</var> be <code>floor( <var>value</var> / <var>weight</var> )</code>.

    4.  Append <var>symbol</var> to <var>S</var> <var>reps</var> times.

    5.  Decrement <var>value</var> by <code><var>weight</var> \* <var>reps</var></code>.

    6.  If <var>value</var> is zero, return <var>S</var>.

4.  Assertion: <var>value</var> is still non-zero.

    The given counter value cannot be represented by this counter style, and must instead be represented by the fallback counter style.

Tests

- [system-additive.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-additive.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-additive.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-additive.html)
- [system-additive-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-additive-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-additive-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-additive-invalid.html)

Note: All of the predefined additive <a id="ref-for-at-ruledef-counter-style①⑦"></a>[&#64;counter-style](#at-ruledef-counter-style) rules in this specification produce representations for every value in their range, but it’s possible to produce values for additive-symbols that will fail to find a representation with the algorithm defined above, even though theoretically a representation could be found. For example, if a <a id="ref-for-at-ruledef-counter-style①⑧"></a>&#64;counter-style was defined with <a id="ref-for-descdef-counter-style-additive-symbols②"></a>[additive-symbols: 3 "a", 2 "b";](#descdef-counter-style-additive-symbols), the algorithm defined above will fail to find a representation for a counter value of 4, even though theoretically a "bb" representation would work. While unfortunate, this is required to maintain the property that the algorithm runs in linear time relative to the size of the counter value.

#### <a id="extends-system"></a>3.1.7.  Building from Existing Counter Styles: the <a id="ref-for-valdef-counter-style-system-extends"></a>[extends](#valdef-counter-style-system-extends) system <a id="override-system"></a>[](#extends-system)

The <a id="valdef-counter-style-system-extends"></a><strong>extends</strong> system allows an author to use the algorithm of another counter style, but alter other aspects, such as the negative sign or the suffix. If a counter style uses the <a id="ref-for-valdef-counter-style-system-extends①"></a>[extends](#valdef-counter-style-system-extends) system, any unspecified descriptors must be taken from the extended counter style specified, rather than taking their initial values.

If a <a id="ref-for-at-ruledef-counter-style①⑨"></a>[&#64;counter-style](#at-ruledef-counter-style) uses the <a id="ref-for-valdef-counter-style-system-extends②"></a>[extends](#valdef-counter-style-system-extends) system, it must not contain a <a id="ref-for-descdef-counter-style-symbols⑥"></a>[symbols](#descdef-counter-style-symbols) or <a id="ref-for-descdef-counter-style-additive-symbols③"></a>[additive-symbols](#descdef-counter-style-additive-symbols) descriptor, otherwise the rule does not define a counter style (but is still a valid rule).

If the specified <a id="ref-for-typedef-counter-style-name③"></a>[\<counter-style-name\>](#typedef-counter-style-name) is a <a id="ref-for-typedef-symbolic-glyph①"></a>[\<symbolic-glyph\>](#typedef-symbolic-glyph) (an <a id="ref-for-ascii-case-insensitive①"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for any of the <a id="ref-for-predefined-symbolic-counter-style①"></a>[predefined symbolic counter styles](#predefined-symbolic-counter-style)), using extend extends from the “standard” definition of the rules provided in the normative stylesheet (rather than the exception allowing them to be drawn in a different, user-agent-specific fashion).

If the specified counter style name isn’t the name of any defined counter style, it must be treated as if it was extending the <a id="ref-for-decimal③"></a>[decimal](#decimal) counter style. If one or more <a id="ref-for-at-ruledef-counter-style②⓪"></a>[&#64;counter-style](#at-ruledef-counter-style) rules form a cycle with their <a id="ref-for-valdef-counter-style-system-extends③"></a>[extends](#valdef-counter-style-system-extends) values, all of the counter styles participating in the cycle must be treated as if they were extending the <a id="ref-for-decimal④"></a>decimal counter style instead.

<a id="example-4eacf761"></a>

<strong>Example:</strong>

[](#example-4eacf761) For example, if you wanted a counter style that was identical to decimal, but used a parenthesis rather than a period after it, like:

``` text
1) first item
2) second item
3) third item
```

Rather than writing up an entirely new counter style, this can be done by just extending <a id="ref-for-decimal⑤"></a>[decimal](#decimal):

``` text
@counter-style decimal-paren {
  system: extends decimal;
  suffix: ") ";
}
```

Tests

- [dependent-builtin.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/dependent-builtin.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/dependent-builtin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/dependent-builtin.html)
- [system-extends.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-extends.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-extends.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-extends.html)
- [system-extends-fixed.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-extends-fixed.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-extends-fixed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-extends-fixed.html)
- [system-extends-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/system-extends-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/system-extends-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/system-extends-invalid.html)

### <a id="counter-style-negative"></a>3.2.  Formatting negative values: the <a id="ref-for-descdef-counter-style-negative①"></a>[negative](#descdef-counter-style-negative) descriptor[](#counter-style-negative)

| Field                     | Definition                                                                                                                                                                                         |
|---------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-counter-style-negative"></a><strong>negative</strong>                                                                                                                               |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-counter-style②①"></a>[&#64;counter-style](#at-ruledef-counter-style)                                                                                                     |
| <strong>Value:</strong>   | <a id="ref-for-typedef-symbol①"></a>[\<symbol\>](#typedef-symbol) <a id="ref-for-typedef-symbol②"></a>\<symbol\><a id="ref-for-mult-opt①"></a>[?](https://drafts.csswg.org/css-values-4/#mult-opt) |
| <strong>Initial:</strong> | "-"                                                                                                                                                                                                |

Tests

- [descriptor-negative.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-negative.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-negative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-negative.html)
- [descriptor-negative-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-negative-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-negative-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-negative-invalid.html)
- [negative-syntax.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/negative-syntax.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/negative-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/negative-syntax.html)

The <a id="ref-for-descdef-counter-style-negative②"></a>[negative](#descdef-counter-style-negative) descriptor defines how to alter the representation when the counter value is negative.

The first <a id="ref-for-typedef-symbol③"></a>[\<symbol\>](#typedef-symbol) in the value is prepended to the representation when the counter value is negative. The second <a id="ref-for-typedef-symbol④"></a>\<symbol\>, if specified, is appended to the representation when the counter value is negative.

<a id="example-cea7bd84"></a>

<strong>Example:</strong>

[](#example-cea7bd84) For example, specifying <a id="ref-for-descdef-counter-style-negative③"></a>[negative: "(" ")";](#descdef-counter-style-negative) will make negative values be wrapped in parentheses, which is sometimes used in financial contexts, like "(2) (1) 0 1 2 3...".

Not all <a id="ref-for-descdef-counter-style-system④"></a>[system](#descdef-counter-style-system) values use a negative sign. In particular, a counter style <a id="use-a-negative-sign"></a><strong>uses a negative sign</strong> if its <a id="ref-for-descdef-counter-style-system⑤"></a>system value is <a id="ref-for-valdef-system-symbolic③"></a>[symbolic](#valdef-system-symbolic), <a id="ref-for-valdef-counter-style-system-alphabetic③"></a>[alphabetic](#valdef-counter-style-system-alphabetic), <a id="ref-for-valdef-counter-style-system-numeric③"></a>[numeric](#valdef-counter-style-system-numeric), <a id="ref-for-valdef-counter-style-system-additive③"></a>[additive](#valdef-counter-style-system-additive), or <a id="ref-for-valdef-counter-style-system-extends④"></a>[extends](#valdef-counter-style-system-extends) if the extended counter style itself <a id="ref-for-use-a-negative-sign②"></a>[uses a negative sign](#use-a-negative-sign). If a counter style does not <a id="ref-for-use-a-negative-sign③"></a>use a negative sign, it ignores the negative sign when <a id="ref-for-generate-a-counter②"></a>[generating a counter representation](#generate-a-counter).

### <a id="counter-style-prefix"></a>3.3.  Symbols before the marker: the <a id="ref-for-descdef-counter-style-prefix①"></a>[prefix](#descdef-counter-style-prefix) descriptor[](#counter-style-prefix)

| Field                     | Definition                                                                                     |
|---------------------------|------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-counter-style-prefix"></a><strong>prefix</strong>                               |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-counter-style②②"></a>[&#64;counter-style](#at-ruledef-counter-style) |
| <strong>Value:</strong>   | <a id="ref-for-typedef-symbol⑤"></a>[\<symbol\>](#typedef-symbol)                              |
| <strong>Initial:</strong> | ""                                                                                             |

Tests

- [descriptor-prefix.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-prefix.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-prefix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-prefix.html)
- [descriptor-prefix-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-prefix-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-prefix-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-prefix-invalid.html)
- [prefix-suffix-syntax.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/prefix-suffix-syntax.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/prefix-suffix-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/prefix-suffix-syntax.html)

The <a id="ref-for-descdef-counter-style-prefix②"></a>[prefix](#descdef-counter-style-prefix) descriptor specifies a <a id="ref-for-typedef-symbol⑥"></a>[\<symbol\>](#typedef-symbol) that is prepended to the marker representation. Prefixes come before any negative sign.

Note: Prefixes are only added by the algorithm for constructing the default contents of the <a id="ref-for-selectordef-marker①"></a>[::marker](https://drafts.csswg.org/css-pseudo-4/#selectordef-marker) pseudo-element; the prefix is not added automatically when the <a id="ref-for-funcdef-counter②"></a>[counter()](https://drafts.csswg.org/css-lists-3/#funcdef-counter) or <a id="ref-for-funcdef-counters①"></a>[counters()](https://drafts.csswg.org/css-lists-3/#funcdef-counters) functions are used.

### <a id="counter-style-suffix"></a>3.4.  Symbols after the marker: the <a id="ref-for-descdef-counter-style-suffix①"></a>[suffix](#descdef-counter-style-suffix) descriptor[](#counter-style-suffix)

| Field                     | Definition                                                                                     |
|---------------------------|------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-counter-style-suffix"></a><strong>suffix</strong>                               |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-counter-style②③"></a>[&#64;counter-style](#at-ruledef-counter-style) |
| <strong>Value:</strong>   | <a id="ref-for-typedef-symbol⑦"></a>[\<symbol\>](#typedef-symbol)                              |
| <strong>Initial:</strong> | ". "                                                                                           |

Tests

- [descriptor-suffix.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-suffix.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-suffix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-suffix.html)
- [descriptor-suffix-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-suffix-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-suffix-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-suffix-invalid.html)

The <a id="ref-for-descdef-counter-style-suffix②"></a>[suffix](#descdef-counter-style-suffix) descriptor specifies a <a id="ref-for-typedef-symbol⑧"></a>[\<symbol\>](#typedef-symbol) that is appended to the marker representation. Suffixes are added to the representation after negative signs.

Note: Suffixes are only added by the algorithm for constructing the default contents of the <a id="ref-for-selectordef-marker②"></a>[::marker](https://drafts.csswg.org/css-pseudo-4/#selectordef-marker) pseudo-element; the suffix is not added automatically when the counter() or counters() functions are used.

### <a id="counter-style-range"></a>3.5.  Limiting the counter scope: the <a id="ref-for-descdef-counter-style-range①"></a>[range](#descdef-counter-style-range) descriptor[](#counter-style-range)

| Field                     | Definition                                                                                                                                                                                                                                                                                                                                                                                                                     |
|---------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-counter-style-range"></a><strong>range</strong>                                                                                                                                                                                                                                                                                                                                                                 |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-counter-style②④"></a>[&#64;counter-style](#at-ruledef-counter-style)                                                                                                                                                                                                                                                                                                                                 |
| <strong>Value:</strong>   | \[ \[ <a id="ref-for-integer-value①"></a>[\<integer\>](https://drafts.csswg.org/css-values-4/#integer-value) <a id="ref-for-comb-one⑥"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) infinite \]<a id="ref-for-mult-num"></a>[{2}](https://drafts.csswg.org/css-values-4/#mult-num) \]<a id="ref-for-mult-comma"></a>[\#](https://drafts.csswg.org/css-values-4/#mult-comma) <a id="ref-for-comb-one⑦"></a>\| auto |
| <strong>Initial:</strong> | auto                                                                                                                                                                                                                                                                                                                                                                                                                           |

Tests

- [descriptor-range.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-range.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-range.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-range.html)
- [descriptor-range-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-range-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-range-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-range-invalid.html)
- [range-syntax.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/range-syntax.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/range-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/range-syntax.html)

The <a id="ref-for-descdef-counter-style-range②"></a>[range](#descdef-counter-style-range) descriptor defines the ranges over which the counter style is defined. If a counter style is used to represent a counter value outside of its ranges, the counter style instead drops down to its fallback counter style.

<strong><a id="valdef-counter-style-range-auto"></a><strong>auto</strong></strong>

The range depends on the counter system:

- For <a id="ref-for-valdef-counter-style-system-cyclic③"></a>[cyclic](#valdef-counter-style-system-cyclic), <a id="ref-for-valdef-counter-style-system-numeric④"></a>[numeric](#valdef-counter-style-system-numeric), and <a id="ref-for-valdef-counter-style-system-fixed②"></a>[fixed](#valdef-counter-style-system-fixed) systems, the range is negative infinity to positive infinity.
- For <a id="ref-for-valdef-counter-style-system-alphabetic④"></a>[alphabetic](#valdef-counter-style-system-alphabetic) and <a id="ref-for-valdef-system-symbolic④"></a>[symbolic](#valdef-system-symbolic) systems, the range is 1 to positive infinity.
- For <a id="ref-for-valdef-counter-style-system-additive④"></a>[additive](#valdef-counter-style-system-additive) systems, the range is 0 to positive infinity.
- For <a id="ref-for-valdef-counter-style-system-extends⑤"></a>[extends](#valdef-counter-style-system-extends) systems, the range is whatever <a id="ref-for-valdef-counter-style-range-auto"></a>[auto](#valdef-counter-style-range-auto) would produce for the extended system; if extending a complex predefined style ([§ 7 Complex Predefined Counter Styles](#complex-predefined-counters)), the range is the style’s defined range.

<strong>\[ \[ <a id="ref-for-integer-value②"></a>[\<integer\>](https://drafts.csswg.org/css-values-4/#integer-value) \| infinite \]{2} \]#</strong>

This defines a comma-separated list of ranges. For each individual range, the first value is the lower bound and the second value is the upper bound. This range is inclusive - it contains both the lower and upper bound numbers. If infinite is used as the first value in a range, it represents negative infinity; if used as the second value, it represents positive infinity. The range of the counter style is the union of all the ranges defined in the list.

If the lower bound of any range is higher than the upper bound, the entire descriptor is invalid and must be ignored.

Implementations must support ranges with a lower bound of at least -2<sup>15</sup> and an upper bound of at least 2<sup>15</sup>-1 (the range of a signed 2-byte int). They may support higher ranges. If any specified bound is outside of the implementation’s supported bounds, it must be treated as the closest bound that the implementation does support.

### <a id="counter-style-pad"></a>3.6.  Zero-Padding and Constant-Width Representations: the <a id="ref-for-descdef-counter-style-pad②"></a>[pad](#descdef-counter-style-pad) descriptor[](#counter-style-pad)

| Field                     | Definition                                                                                                                                                                                                                                                          |
|---------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-counter-style-pad"></a><strong>pad</strong>                                                                                                                                                                                                          |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-counter-style②⑤"></a>[&#64;counter-style](#at-ruledef-counter-style)                                                                                                                                                                      |
| <strong>Value:</strong>   | <a id="ref-for-integer-value③"></a>[\<integer \[0,∞\]\>](https://drafts.csswg.org/css-values-4/#integer-value) <a id="ref-for-comb-all"></a>[&&](https://drafts.csswg.org/css-values-4/#comb-all) <a id="ref-for-typedef-symbol⑨"></a>[\<symbol\>](#typedef-symbol) |
| <strong>Initial:</strong> | 0 ""                                                                                                                                                                                                                                                                |

Tests

- [descriptor-pad.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-pad.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-pad.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-pad.html)
- [descriptor-pad-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-pad-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-pad-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-pad-invalid.html)
- [pad-syntax.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/pad-syntax.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/pad-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/pad-syntax.html)

The <a id="ref-for-descdef-counter-style-pad③"></a>[pad](#descdef-counter-style-pad) descriptor allows an author to specify a "fixed-width" counter style, where representations shorter than the pad value are padded with a particular <a id="ref-for-typedef-symbol①⓪"></a>[\<symbol\>](#typedef-symbol). Representations larger than the specified pad value are constructed as normal.

<strong><a id="ref-for-integer-value④"></a>[\<integer \[0,∞\]\>](https://drafts.csswg.org/css-values-4/#integer-value) && <a id="ref-for-typedef-symbol①①"></a>[\<symbol\>](#typedef-symbol)</strong>

The <a id="ref-for-integer-value⑤"></a>[\<integer\>](https://drafts.csswg.org/css-values-4/#integer-value) specifies a minimum length that all counter representations must reach.

Let <var>difference</var> be the provided <a id="ref-for-integer-value⑥"></a>[\<integer\>](https://drafts.csswg.org/css-values-4/#integer-value) minus the number of <a id="ref-for-grapheme-cluster"></a>[grapheme clusters](https://drafts.csswg.org/css-text-3/#grapheme-cluster) in the <a id="ref-for-initial-representation-for-the-counter-value"></a>[initial representation for the counter value](#initial-representation-for-the-counter-value). (Note that, per the algorithm to

<strong>Note:</strong>

<a id="ref-for-generate-a-counter③"></a>[generate a counter representation](#generate-a-counter), this occurs before adding prefixes/suffixes/negatives.) If the counter value is negative and the counter style <a id="ref-for-use-a-negative-sign④"></a>[uses a negative sign](#use-a-negative-sign), further reduce <var>difference</var> by the number of <a id="ref-for-grapheme-cluster①"></a>grapheme clusters in the counter style’s <a id="ref-for-descdef-counter-style-negative④"></a>[negative](#descdef-counter-style-negative) descriptor’s <a id="ref-for-typedef-symbol①②"></a>[\<symbol\>](#typedef-symbol)(s).

If <var>difference</var> is greater than zero, prepend <var>difference</var> copies of the specified <a id="ref-for-typedef-symbol①③"></a>[\<symbol\>](#typedef-symbol) to the representation.

Negative <a id="ref-for-integer-value⑦"></a>[\<integer\>](https://drafts.csswg.org/css-values-4/#integer-value) values are not allowed.

<a id="example-9431be5f"></a>

<strong>Example:</strong>

[](#example-9431be5f) The most common example of "fixed-width" numbering is zero-padded decimal numbering. If an author knows that the numbers used will be less than a thousand, for example, it can be zero-padded with a simple <a id="ref-for-descdef-counter-style-pad④"></a>[pad: 3 "0";](#descdef-counter-style-pad) descriptor, ensuring that all of the representations are 3 digits wide.

This will cause, for example, 1 to be represented as "001", 20 to be represented as "020", 300 to be represented as "300", 4000 to be represented as "4000", and -5 to be represented as "-05".

Note: The <a id="ref-for-descdef-counter-style-pad⑤"></a>[pad](#descdef-counter-style-pad) descriptor counts the number of <a id="ref-for-grapheme-cluster②"></a>[grapheme clusters](https://drafts.csswg.org/css-text-3/#grapheme-cluster) in the representation, but pads it with <a id="ref-for-typedef-symbol①④"></a>[\<symbol\>](#typedef-symbol)s. If the specified <a id="ref-for-descdef-counter-style-pad⑥"></a>pad <a id="ref-for-typedef-symbol①⑤"></a>\<symbol\> is multi-character, this will likely not have the desired effect. Unfortunately, there’s no way to use the number of <a id="ref-for-grapheme-cluster③"></a>grapheme clusters in the <a id="ref-for-descdef-counter-style-pad⑦"></a>pad <a id="ref-for-typedef-symbol①⑥"></a>\<symbol\> without violating useful constraints. It is recommended that authors only specify <a id="ref-for-typedef-symbol①⑦"></a>\<symbol\>s of a single <a id="ref-for-grapheme-cluster④"></a>grapheme cluster in the <a id="ref-for-descdef-counter-style-pad⑧"></a>pad descriptor.

### <a id="counter-style-fallback"></a>3.7.  Defining fallback: the <a id="ref-for-descdef-counter-style-fallback"></a>[fallback](#descdef-counter-style-fallback) descriptor[](#counter-style-fallback)

| Field                     | Definition                                                                                            |
|---------------------------|-------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-counter-style-fallback"></a><strong>fallback</strong>                                  |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-counter-style②⑥"></a>[&#64;counter-style](#at-ruledef-counter-style)        |
| <strong>Value:</strong>   | <a id="ref-for-typedef-counter-style-name④"></a>[\<counter-style-name\>](#typedef-counter-style-name) |
| <strong>Initial:</strong> | decimal                                                                                               |

Tests

- [descriptor-fallback.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-fallback.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-fallback.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-fallback.html)
- [descriptor-fallback-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-fallback-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-fallback-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-fallback-invalid.html)
- [fallback.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/fallback.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/fallback.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/fallback.html)
- [fallbacks-in-shadow-dom.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/fallbacks-in-shadow-dom.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/fallbacks-in-shadow-dom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/fallbacks-in-shadow-dom.html)

The <a id="ref-for-descdef-counter-style-fallback①"></a>[fallback](#descdef-counter-style-fallback) descriptor specifies a fallback counter style to be used when the current counter style can’t create a representation for a given counter value. For example, if a counter style defined with a range of 1-10 is asked to represent a counter value of 11, the counter value’s representation is instead constructed with the fallback counter style (or possibly the fallback style’s fallback style, if the fallback style can’t represent that value, etc.).

If the value of the <a id="ref-for-descdef-counter-style-fallback②"></a>[fallback](#descdef-counter-style-fallback) descriptor isn’t the name of any defined counter style, the used value of the <a id="ref-for-descdef-counter-style-fallback③"></a>fallback descriptor is <a id="ref-for-decimal⑥"></a>[decimal](#decimal) instead. Similarly, while following fallbacks to find a counter style that can render the given counter value, if a loop in the specified fallbacks is detected, the <a id="ref-for-decimal⑦"></a>decimal style must be used instead.

Note that it is not necessarily an error to specify fallback loops. For example, if an author desires a counter style with significantly different representations for even and odd counter values, they may find it easiest to define one style that can only represent odd values and one that can only represent even values, and specify each as the fallback for the other one. Though the fallback graph is circular, at no point do you encounter a loop while following these fallbacks - every counter value is represented by one or the other counter style.

<strong>Note:</strong>

### <a id="counter-style-symbols"></a>3.8.  Marker characters: the <a id="ref-for-descdef-counter-style-symbols⑦"></a>[symbols](#descdef-counter-style-symbols) and <a id="ref-for-descdef-counter-style-additive-symbols④"></a>[additive-symbols](#descdef-counter-style-additive-symbols) descriptors[](#counter-style-symbols)

| Field                     | Definition                                                                                                                                                    |
|---------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-counter-style-symbols"></a><strong>symbols</strong>                                                                                            |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-counter-style②⑦"></a>[&#64;counter-style](#at-ruledef-counter-style)                                                                |
| <strong>Value:</strong>   | <a id="ref-for-typedef-symbol①⑧"></a>[\<symbol\>](#typedef-symbol)<a id="ref-for-mult-one-plus"></a>[+](https://drafts.csswg.org/css-values-4/#mult-one-plus) |
| <strong>Initial:</strong> | n/a                                                                                                                                                           |

| Field                     | Definition                                                                                                                                                                                                                                                                                                                                                         |
|---------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-counter-style-additive-symbols"></a><strong>additive-symbols</strong>                                                                                                                                                                                                                                                                               |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-counter-style②⑧"></a>[&#64;counter-style](#at-ruledef-counter-style)                                                                                                                                                                                                                                                                     |
| <strong>Value:</strong>   | \[ <a id="ref-for-integer-value⑧"></a>[\<integer \[0,∞\]\>](https://drafts.csswg.org/css-values-4/#integer-value) <a id="ref-for-comb-all①"></a>[&&](https://drafts.csswg.org/css-values-4/#comb-all) <a id="ref-for-typedef-symbol①⑨"></a>[\<symbol\>](#typedef-symbol) \]<a id="ref-for-mult-comma①"></a>[\#](https://drafts.csswg.org/css-values-4/#mult-comma) |
| <strong>Initial:</strong> | n/a                                                                                                                                                                                                                                                                                                                                                                |

Tests

- [additive-symbols-syntax.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/additive-symbols-syntax.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/additive-symbols-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/additive-symbols-syntax.html)
- [descriptor-symbols.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-symbols.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-symbols.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-symbols.html)
- [descriptor-symbols-invalid.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/descriptor-symbols-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/descriptor-symbols-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/descriptor-symbols-invalid.html)
- [symbols-syntax.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/symbols-syntax.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/symbols-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/symbols-syntax.html)

<a id="typedef-symbol"></a><a id="ref-for-string-value"></a><a id="ref-for-comb-one⑧"></a><a id="ref-for-typedef-image①"></a><a id="ref-for-comb-one⑨"></a><a id="ref-for-identifier-value②"></a>

``` text
<symbol> = <string> | <image> | <custom-ident>
```

Tests

- [broken-symbols.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/broken-symbols.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/broken-symbols.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/broken-symbols.html)
- [empty-string-symbol.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/empty-string-symbol.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/empty-string-symbol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/empty-string-symbol.html)

The <a id="ref-for-descdef-counter-style-symbols⑧"></a>[symbols](#descdef-counter-style-symbols) and <a id="ref-for-descdef-counter-style-additive-symbols⑤"></a>[additive-symbols](#descdef-counter-style-additive-symbols) descriptors specify the symbols used by the marker-construction algorithm specified by the <a id="ref-for-descdef-counter-style-system⑥"></a>[system](#descdef-counter-style-system) descriptor. The <a id="ref-for-at-ruledef-counter-style②⑨"></a>[&#64;counter-style](#at-ruledef-counter-style) rule must have a valid <a id="ref-for-descdef-counter-style-symbols⑨"></a>symbols descriptor if the counter system is <a id="ref-for-valdef-counter-style-system-cyclic④"></a>[cyclic](#valdef-counter-style-system-cyclic), <a id="ref-for-valdef-counter-style-system-numeric⑤"></a>[numeric](#valdef-counter-style-system-numeric), <a id="ref-for-valdef-counter-style-system-alphabetic⑤"></a>[alphabetic](#valdef-counter-style-system-alphabetic), <a id="ref-for-valdef-system-symbolic⑤"></a>[symbolic](#valdef-system-symbolic), or <a id="ref-for-valdef-counter-style-system-fixed③"></a>[fixed](#valdef-counter-style-system-fixed), or a valid <a id="ref-for-descdef-counter-style-additive-symbols⑥"></a>additive-symbols descriptor if the counter system is <a id="ref-for-valdef-counter-style-system-additive⑤"></a>[additive](#valdef-counter-style-system-additive); otherwise, the <a id="ref-for-at-ruledef-counter-style③⓪"></a>&#64;counter-style does not define a <a id="ref-for-counter-style④"></a>[counter style](#counter-style) (but is still a valid <a id="ref-for-at-rule"></a>[at-rule](https://drafts.csswg.org/css-syntax-3/#at-rule)).

Some counter systems specify that the <a id="ref-for-descdef-counter-style-symbols①⓪"></a>[symbols](#descdef-counter-style-symbols) descriptor must have at least two entries. If the counter style’s system is such, and the <a id="ref-for-descdef-counter-style-symbols①①"></a>symbols descriptor has only a single entry, the <a id="ref-for-at-ruledef-counter-style③①"></a>[&#64;counter-style](#at-ruledef-counter-style) rule does not define a <a id="ref-for-counter-style⑤"></a>[counter style](#counter-style).

Each entry in the <a id="ref-for-descdef-counter-style-symbols①②"></a>[symbols](#descdef-counter-style-symbols) descriptor’s value defines a <a id="counter-symbol"></a><strong>counter symbol</strong>, which is interpreted differently based on the counter style’s system. Each entry in the <a id="ref-for-descdef-counter-style-additive-symbols⑦"></a>[additive-symbols](#descdef-counter-style-additive-symbols) descriptor’s value defines an <a id="additive-tuple"></a><strong>additive tuple</strong>, which consists of a <a id="ref-for-counter-symbol③④"></a>[counter symbol](#counter-symbol) and an integer weight. Each weight must be a non-negative integer, and the <a id="ref-for-additive-tuple②"></a>[additive tuples](#additive-tuple) must be specified in order of strictly descending weight; otherwise, the declaration is invalid and must be ignored.

<a id="ref-for-counter-symbol③⑤"></a>[Counter symbols](#counter-symbol) may be strings, images, or identifiers, and the three types can be mixed in a single descriptor. Counter representations are constructed by concatenating counter symbols together. Identifiers are rendered as strings containing the same characters. Images are rendered as inline replaced elements. The <a id="ref-for-default-object-size"></a>[default object size](https://drafts.csswg.org/css-images-3/#default-object-size) of an image <a id="ref-for-counter-symbol③⑥"></a>counter symbol is a 1em by 1em square.

Note: The <a id="ref-for-typedef-image②"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image) syntax in <a id="ref-for-typedef-symbol②⓪"></a>[\<symbol\>](#typedef-symbol) is currently at-risk. No implementations have plans to implement it currently, and it complicates some usages of <a id="ref-for-funcdef-counter③"></a>[counter()](https://drafts.csswg.org/css-lists-3/#funcdef-counter) in ways that haven’t been fully handled.

Note: If using identifiers rather than strings to define the symbols, be aware of the syntax of identifiers. In particular, ascii non-letters like "\*" are not identifiers, and so must be quoted in a string. Hex escapes, used in several of the counter styles defined in this specification, "eat" the following space (to allow a digit to follow a hex escape without ambiguity), so two spaces must be put after a hex escape to separate it from the following one, or else they’ll be considered adjacent, and part of the same identifier. For example, <a id="ref-for-descdef-counter-style-symbols①③"></a>[symbols: \660 \661;](#descdef-counter-style-symbols) only defines a single symbol, consisting of the U+0660 and U+0661 characters, rather than the two that were intended; either quote the escapes in strings, like <a id="ref-for-descdef-counter-style-symbols①④"></a>symbols: "\660" "\661", or put two spaces between the escapes.

### <a id="counter-style-speak-as"></a>3.9.  Speech Synthesis: the <a id="ref-for-descdef-counter-style-speak-as"></a>[speak-as](#descdef-counter-style-speak-as) descriptor[](#counter-style-speak-as)

| Field                     | Definition                                                                                                                                                                                                                                                                                                                                                              |
|---------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-counter-style-speak-as"></a><strong>speak-as</strong>                                                                                                                                                                                                                                                                                                    |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-counter-style③②"></a>[&#64;counter-style](#at-ruledef-counter-style)                                                                                                                                                                                                                                                                          |
| <strong>Value:</strong>   | auto <a id="ref-for-comb-one①⓪"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) bullets <a id="ref-for-comb-one①①"></a>\| numbers <a id="ref-for-comb-one①②"></a>\| words <a id="ref-for-comb-one①③"></a>\| spell-out <a id="ref-for-comb-one①④"></a>\| <a id="ref-for-typedef-counter-style-name⑤"></a>[\<counter-style-name\>](#typedef-counter-style-name) |
| <strong>Initial:</strong> | auto                                                                                                                                                                                                                                                                                                                                                                    |

Tests

- speak-as-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/speak-as-manual.html)
- [speak-as-syntax.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/speak-as-syntax.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/speak-as-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/speak-as-syntax.html)

A counter style can be constructed with a meaning that is obvious visually, but impossible to meaningfully represent via a speech synthesizer or other non-visual means, or possible but nonsensical when naively read out loud. The <a id="ref-for-descdef-counter-style-speak-as①"></a>[speak-as](#descdef-counter-style-speak-as) descriptor describes how to synthesize the spoken form of a counter formatted with the given counter style. Assistive technologies should use this spoken form when reading out the counter style, and may use the <a id="ref-for-descdef-counter-style-speak-as②"></a>speak-as value to inform transformations to outputs other than speech. Values have the following meanings:

<strong><a id="valdef-counter-style-speak-as-auto"></a><strong>auto</strong></strong>

If the counter style’s <a id="ref-for-descdef-counter-style-system⑦"></a>[system](#descdef-counter-style-system) is <a id="ref-for-valdef-counter-style-system-alphabetic⑥"></a>[alphabetic](#valdef-counter-style-system-alphabetic), this value has the same effect as <a id="ref-for-valdef-counter-style-speak-as-spell-out"></a>[spell-out](#valdef-counter-style-speak-as-spell-out). If the <a id="ref-for-descdef-counter-style-system⑧"></a>system is <a id="ref-for-valdef-counter-style-system-cyclic⑤"></a>[cyclic](#valdef-counter-style-system-cyclic), this value has the same effect as <a id="ref-for-valdef-counter-style-speak-as-bullets"></a>[bullets](#valdef-counter-style-speak-as-bullets). If the <a id="ref-for-descdef-counter-style-system⑨"></a>system is <a id="ref-for-valdef-counter-style-system-extends⑥"></a>[extends](#valdef-counter-style-system-extends), this value has the same effect as <a id="ref-for-valdef-counter-style-speak-as-auto"></a>[auto](#valdef-counter-style-speak-as-auto) would have for the extended style. Otherwise, this value has the same effect as <a id="ref-for-valdef-counter-style-speak-as-numbers"></a>[numbers](#valdef-counter-style-speak-as-numbers).

<strong><a id="valdef-counter-style-speak-as-bullets"></a><strong>bullets</strong></strong>

The UA speaks a UA-defined phrase or audio cue that represents an unordered list item being read out.

<strong><a id="valdef-counter-style-speak-as-numbers"></a><strong>numbers</strong></strong>

The counter’s numeric value is spoken as a number in the <a id="ref-for-content-language"></a>[content language](https://drafts.csswg.org/css-text-4/#content-language).

<strong><a id="valdef-counter-style-speak-as-words"></a><strong>words</strong></strong>

<a id="ref-for-generate-a-counter④"></a>[Generate a counter representation](#generate-a-counter) for the value as normal, then speak it as normal text in the <a id="ref-for-content-language①"></a>[content language](https://drafts.csswg.org/css-text-4/#content-language). If the counter representation contains images, instead handle the value as for <a id="ref-for-valdef-counter-style-speak-as-numbers①"></a>[numbers](#valdef-counter-style-speak-as-numbers).

<strong><a id="valdef-counter-style-speak-as-spell-out"></a><strong>spell-out</strong></strong>

<a id="ref-for-generate-a-counter⑤"></a>[Generate a counter representation](#generate-a-counter) for the value as normal, then spell it out letter-by-letter in the <a id="ref-for-content-language②"></a>[content language](https://drafts.csswg.org/css-text-4/#content-language). If the UA does not know how to pronounce the symbols (or the counter representation contains images), it must instead handle the value as for <a id="ref-for-valdef-counter-style-speak-as-numbers②"></a>[numbers](#valdef-counter-style-speak-as-numbers).

For example, <a id="ref-for-lower-greek"></a>[lower-greek](#lower-greek) in English would be read out as "alpha", "beta", "gamma", etc. Conversely, <a id="ref-for-upper-latin"></a>[upper-latin](#upper-latin) in French would be read out as (in phonetic notation) /a/, /be/, /se/, etc.

<strong><a id="valdef-counter-style-speak-as-counter-style-name"></a><strong><a id="ref-for-typedef-counter-style-name⑥"></a>[\<counter-style-name\>](#typedef-counter-style-name)</strong></strong>

The counter’s value is instead spoken out in the specified style (similar to the behavior of the <a id="ref-for-descdef-counter-style-fallback④"></a>[fallback](#descdef-counter-style-fallback) descriptor when generating representations for a counter value). If the specified style does not exist, this value is treated as <a id="ref-for-valdef-counter-style-speak-as-auto①"></a>[auto](#valdef-counter-style-speak-as-auto). If a loop is detected when following <a id="ref-for-descdef-counter-style-speak-as③"></a>[speak-as](#descdef-counter-style-speak-as) references, this value is treated as <a id="ref-for-valdef-counter-style-speak-as-auto②"></a>auto for the counter styles participating in the loop.

<a id="example-cce4b0db"></a>

<strong>Example:</strong>

[](#example-cce4b0db) The ability to defer pronunciation to another counter style can help when the symbols being used aren’t actually letters. For example, here’s a possible definition of a circled-lower-latin counter-style, using some special Unicode characters:

<a id="circled-lower-latin"></a>

``` text
@counter-style circled-lower-latin {
  system: alphabetic;
  speak-as: lower-latin;
  symbols: ⓐ ⓑ ⓒ ⓓ ⓔ ⓕ ⓖ ⓗ ⓘ ⓙ ⓚ ⓛ ⓜ ⓝ ⓞ ⓟ ⓠ ⓡ ⓢ ⓣ ⓤ ⓥ ⓦ ⓧ ⓨ ⓩ;
  suffix: " ";
}
```

Setting its <a id="ref-for-descdef-counter-style-system①⓪"></a>[system](#descdef-counter-style-system) to <a id="ref-for-valdef-counter-style-system-alphabetic⑦"></a>[alphabetic](#valdef-counter-style-system-alphabetic) would normally make the UA try to read out the names of the characters, but in this case that might be something like "Circled Letter A", which is unlikely to make sense. Instead, explicitly setting <a id="ref-for-descdef-counter-style-speak-as④"></a>[speak-as](#descdef-counter-style-speak-as) to <a id="ref-for-lower-latin"></a>[lower-latin](#lower-latin) ensures that they get read out as their corresponding latin letters, as intended.

## <a id="symbols-function"></a>4.  Defining Anonymous Counter Styles: the <a id="ref-for-funcdef-symbols"></a>[symbols()](#funcdef-symbols) function[](#symbols-function)

The <a id="ref-for-funcdef-symbols①"></a>[symbols()](#funcdef-symbols) function allows a <a id="ref-for-counter-style⑥"></a>[counter style](#counter-style) to be defined inline in a property value, for when a style is used only once in a stylesheet and defining a full <a id="ref-for-at-ruledef-counter-style③③"></a>[&#64;counter-style](#at-ruledef-counter-style) rule would be overkill. It does not provide the full feature-set of the <a id="ref-for-at-ruledef-counter-style③④"></a>&#64;counter-style rule, but provides a sufficient subset to still be useful. The syntax of the <a id="ref-for-funcdef-symbols②"></a>symbols() rule is:

<a id="funcdef-symbols"></a><a id="ref-for-typedef-symbols-type"></a><a id="ref-for-mult-opt②"></a><a id="ref-for-string-value①"></a><a id="ref-for-comb-one①⑤"></a><a id="ref-for-typedef-image③"></a><a id="ref-for-mult-one-plus①"></a><a id="typedef-symbols-type"></a><a id="ref-for-comb-one①⑥"></a><a id="ref-for-comb-one①⑦"></a><a id="ref-for-comb-one①⑧"></a><a id="ref-for-comb-one①⑨"></a>

``` text
symbols() = symbols( <symbols-type>? [ <string> | <image> ]+ )
<symbols-type> = cyclic | numeric | alphabetic | symbolic | fixed
```

Tests

The <a id="ref-for-funcdef-symbols③"></a>[symbols()](#funcdef-symbols) function defines an anonymous counter style with no name, a <a id="ref-for-descdef-counter-style-prefix③"></a>[prefix](#descdef-counter-style-prefix) of "" (empty string) and <a id="ref-for-descdef-counter-style-suffix③"></a>[suffix](#descdef-counter-style-suffix) of " " (U+0020 SPACE), a <a id="ref-for-descdef-counter-style-range③"></a>[range](#descdef-counter-style-range) of <a id="ref-for-valdef-counter-style-range-auto①"></a>[auto](#valdef-counter-style-range-auto), a <a id="ref-for-descdef-counter-style-fallback⑤"></a>[fallback](#descdef-counter-style-fallback) of <a id="ref-for-decimal⑧"></a>[decimal](#decimal), a <a id="ref-for-descdef-counter-style-negative⑤"></a>[negative](#descdef-counter-style-negative) of "\2D" ("-" hyphen-minus), a <a id="ref-for-descdef-counter-style-pad⑨"></a>[pad](#descdef-counter-style-pad) of 0 "", and a <a id="ref-for-descdef-counter-style-speak-as⑤"></a>[speak-as](#descdef-counter-style-speak-as) of <a id="ref-for-valdef-counter-style-speak-as-auto③"></a>[auto](#valdef-counter-style-speak-as-auto). The counter style’s algorithm is constructed by consulting the previous chapter using the provided system — or <a id="ref-for-valdef-system-symbolic⑥"></a>[symbolic](#valdef-system-symbolic) if the system was omitted — and the provided <a id="ref-for-string-value②"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value)s and <a id="ref-for-typedef-image④"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image)s as the value of the <a id="ref-for-descdef-counter-style-symbols①⑤"></a>[symbols](#descdef-counter-style-symbols) property. If the system is <a id="ref-for-valdef-counter-style-system-fixed④"></a>[fixed](#valdef-counter-style-system-fixed), the <a id="ref-for-first-symbol-value③"></a>[first symbol value](#first-symbol-value) is 1.

If the system is <a id="ref-for-valdef-counter-style-system-alphabetic⑧"></a>[alphabetic](#valdef-counter-style-system-alphabetic) or <a id="ref-for-valdef-counter-style-system-numeric⑥"></a>[numeric](#valdef-counter-style-system-numeric), there must be at least two <a id="ref-for-string-value③"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value)s or <a id="ref-for-typedef-image⑤"></a>[\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image)s, or else the function is invalid.

<a id="example-62664520"></a>

<strong>Example:</strong>

[](#example-62664520) This code:

``` text
ol { list-style: symbols("*" "\2020" "\2021" "\A7"); }
```

will produce lists that look like:

``` text
*   One
†   Two
‡   Three
§   Four
**  Five
††  Six
‡‡  Seven
```

On the other hand, specifying the system of counter, like so:

``` text
ol { list-style: symbols(cyclic "*" "\2020" "\2021" "\A7"); }
```

will produce lists that look like:

``` text
*   One
†   Two
‡   Three
§   Four
*   Five
†   Six
‡   Seven
```

Note: the <a id="ref-for-funcdef-symbols④"></a>[symbols()](#funcdef-symbols) function only allows strings and images, while the <a id="ref-for-descdef-counter-style-symbols①⑥"></a>[symbols](#descdef-counter-style-symbols) descriptor of a <a id="ref-for-at-ruledef-counter-style③⑤"></a>[&#64;counter-style](#at-ruledef-counter-style) rule also allows identifiers.

## <a id="extending-css2"></a>5.  Extending <a id="ref-for-propdef-list-style-type①"></a>[list-style-type](https://drafts.csswg.org/css-lists-3/#propdef-list-style-type), <a id="ref-for-funcdef-counter④"></a>[counter()](https://drafts.csswg.org/css-lists-3/#funcdef-counter), and <a id="ref-for-funcdef-counters②"></a>[counters()](https://drafts.csswg.org/css-lists-3/#funcdef-counters)[](#extending-css2)

In CSS Level 2 [\[CSS21\]](#biblio-css21) the <a id="ref-for-propdef-list-style-type②"></a>[list-style-type](https://drafts.csswg.org/css-lists-3/#propdef-list-style-type) property and the <a id="ref-for-funcdef-counter⑤"></a>[counter()](https://drafts.csswg.org/css-lists-3/#funcdef-counter) and <a id="ref-for-funcdef-counters③"></a>[counters()](https://drafts.csswg.org/css-lists-3/#funcdef-counters) notations accept various pre-defined keywords, each identifying a counter style. This module extends these features to take instead the <a id="ref-for-typedef-counter-style"></a>[\<counter-style\>](#typedef-counter-style) type, defined below:

<a id="typedef-counter-style"></a><a id="ref-for-typedef-counter-style-name⑦"></a><a id="ref-for-comb-one②⓪"></a><a id="ref-for-funcdef-symbols⑤"></a>

``` text
<counter-style> = <counter-style-name> | <symbols()>
```

If a <a id="ref-for-typedef-counter-style-name⑧"></a>[\<counter-style-name\>](#typedef-counter-style-name) is used that does not refer to any existing counter style, it must act identically to the <a id="ref-for-decimal⑨"></a>[decimal](#decimal) counter style (but does not <em><a id="ref-for-computed-value"></a>[compute](https://drafts.csswg.org/css-cascade-5/#computed-value)</em> to <a id="ref-for-decimal①⓪"></a>decimal).

When used in these contexts, a <a id="ref-for-typedef-counter-style-name⑨"></a>[\<counter-style-name\>](#typedef-counter-style-name) is a <a id="ref-for-css-tree-scoped-reference"></a>[tree-scoped reference](https://drafts.csswg.org/css-shadow-1/#css-tree-scoped-reference).

## <a id="predefined-counters"></a>6.  Simple Predefined Counter Styles[](#predefined-counters)

The following stylesheet uses the <a id="ref-for-at-ruledef-counter-style③⑥"></a>[&#64;counter-style](#at-ruledef-counter-style) rule to redefine all of the counter styles defined in CSS 2 and CSS 2.1. This stylesheet is normative—​UAs must include it in their UA stylesheet (or at least act as if these rules were defined at that level).

### <a id="simple-numeric"></a>6.1.  Numeric: <a id="ref-for-decimal①①"></a>[decimal](#decimal), <a id="ref-for-decimal-leading-zero"></a>[decimal-leading-zero](#decimal-leading-zero), <a id="ref-for-valdef-counter-style-name-arabic-indic"></a>[arabic-indic](#valdef-counter-style-name-arabic-indic), <a id="ref-for-armenian"></a>[armenian](#armenian), <a id="ref-for-valdef-counter-style-name-upper-armenian"></a>[upper-armenian](#valdef-counter-style-name-upper-armenian), <a id="ref-for-valdef-counter-style-name-lower-armenian"></a>[lower-armenian](#valdef-counter-style-name-lower-armenian), <a id="ref-for-valdef-counter-style-name-bengali"></a>[bengali](#valdef-counter-style-name-bengali), <a id="ref-for-valdef-counter-style-name-cambodian"></a>[cambodian](#valdef-counter-style-name-cambodian), <a id="ref-for-valdef-counter-style-name-khmer"></a>[khmer](#valdef-counter-style-name-khmer), <a id="ref-for-cjk-decimal"></a>[cjk-decimal](#cjk-decimal), <a id="ref-for-valdef-counter-style-name-devanagari"></a>[devanagari](#valdef-counter-style-name-devanagari), <a id="ref-for-georgian"></a>[georgian](#georgian), <a id="ref-for-valdef-counter-style-name-gujarati"></a>[gujarati](#valdef-counter-style-name-gujarati), <a id="ref-for-valdef-counter-style-name-gurmukhi"></a>[gurmukhi](#valdef-counter-style-name-gurmukhi), <a id="ref-for-hebrew"></a>[hebrew](#hebrew), <a id="ref-for-valdef-counter-style-name-kannada"></a>[kannada](#valdef-counter-style-name-kannada), <a id="ref-for-valdef-counter-style-name-lao"></a>[lao](#valdef-counter-style-name-lao), <a id="ref-for-valdef-counter-style-name-malayalam"></a>[malayalam](#valdef-counter-style-name-malayalam), <a id="ref-for-valdef-counter-style-name-mongolian"></a>[mongolian](#valdef-counter-style-name-mongolian), <a id="ref-for-valdef-counter-style-name-myanmar"></a>[myanmar](#valdef-counter-style-name-myanmar), <a id="ref-for-valdef-counter-style-name-oriya"></a>[oriya](#valdef-counter-style-name-oriya), <a id="ref-for-valdef-counter-style-name-persian"></a>[persian](#valdef-counter-style-name-persian), <a id="ref-for-lower-roman"></a>[lower-roman](#lower-roman), <a id="ref-for-upper-roman"></a>[upper-roman](#upper-roman), <a id="ref-for-valdef-counter-style-name-tamil"></a>[tamil](#valdef-counter-style-name-tamil), <a id="ref-for-valdef-counter-style-name-telugu"></a>[telugu](#valdef-counter-style-name-telugu), <a id="ref-for-valdef-counter-style-name-thai"></a>[thai](#valdef-counter-style-name-thai), <a id="ref-for-valdef-counter-style-name-tibetan"></a>[tibetan](#valdef-counter-style-name-tibetan)[](#simple-numeric)

<strong><a id="decimal"></a><strong>decimal</strong></strong>

Western decimal numbers (e.g., 1, 2, 3, ..., 98, 99, 100).

<strong><a id="decimal-leading-zero"></a><strong>decimal-leading-zero</strong></strong>

Decimal numbers padded by initial zeros (e.g., 01, 02, 03, ..., 98, 99, 100).

<strong><a id="valdef-counter-style-name-arabic-indic"></a><strong>arabic-indic</strong></strong>

Arabic-indic numbering (e.g., ١‎, ٢‎, ٣‎, ٤‎, ..., ٩٨‎, ٩٩‎, ١٠٠‎).

Tests

- [css3-counter-styles-101.html](https://wpt.fyi/results/css/css-counter-styles/arabic-indic/css3-counter-styles-101.html) [(live test)](http://wpt.live/css/css-counter-styles/arabic-indic/css3-counter-styles-101.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/arabic-indic/css3-counter-styles-101.html)
- [css3-counter-styles-102.html](https://wpt.fyi/results/css/css-counter-styles/arabic-indic/css3-counter-styles-102.html) [(live test)](http://wpt.live/css/css-counter-styles/arabic-indic/css3-counter-styles-102.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/arabic-indic/css3-counter-styles-102.html)
- [css3-counter-styles-103.html](https://wpt.fyi/results/css/css-counter-styles/arabic-indic/css3-counter-styles-103.html) [(live test)](http://wpt.live/css/css-counter-styles/arabic-indic/css3-counter-styles-103.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/arabic-indic/css3-counter-styles-103.html)

<strong><a id="armenian"></a><strong>armenian</strong></strong>

<strong><a id="valdef-counter-style-name-upper-armenian"></a><strong>upper-armenian</strong></strong>

Traditional uppercase Armenian numbering (e.g., Ա, Բ, Գ, ..., ՂԸ, ՂԹ, Ճ).

Tests

- [css3-counter-styles-006.html](https://wpt.fyi/results/css/css-counter-styles/armenian/css3-counter-styles-006.html) [(live test)](http://wpt.live/css/css-counter-styles/armenian/css3-counter-styles-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/armenian/css3-counter-styles-006.html)
- [css3-counter-styles-007.html](https://wpt.fyi/results/css/css-counter-styles/armenian/css3-counter-styles-007.html) [(live test)](http://wpt.live/css/css-counter-styles/armenian/css3-counter-styles-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/armenian/css3-counter-styles-007.html)
- [css3-counter-styles-008.html](https://wpt.fyi/results/css/css-counter-styles/armenian/css3-counter-styles-008.html) [(live test)](http://wpt.live/css/css-counter-styles/armenian/css3-counter-styles-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/armenian/css3-counter-styles-008.html)
- [css3-counter-styles-009.html](https://wpt.fyi/results/css/css-counter-styles/armenian/css3-counter-styles-009.html) [(live test)](http://wpt.live/css/css-counter-styles/armenian/css3-counter-styles-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/armenian/css3-counter-styles-009.html)
- [css3-counter-styles-107.html](https://wpt.fyi/results/css/css-counter-styles/upper-armenian/css3-counter-styles-107.html) [(live test)](http://wpt.live/css/css-counter-styles/upper-armenian/css3-counter-styles-107.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/upper-armenian/css3-counter-styles-107.html)
- [css3-counter-styles-108.html](https://wpt.fyi/results/css/css-counter-styles/upper-armenian/css3-counter-styles-108.html) [(live test)](http://wpt.live/css/css-counter-styles/upper-armenian/css3-counter-styles-108.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/upper-armenian/css3-counter-styles-108.html)
- [css3-counter-styles-109.html](https://wpt.fyi/results/css/css-counter-styles/upper-armenian/css3-counter-styles-109.html) [(live test)](http://wpt.live/css/css-counter-styles/upper-armenian/css3-counter-styles-109.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/upper-armenian/css3-counter-styles-109.html)
- [css3-counter-styles-110.html](https://wpt.fyi/results/css/css-counter-styles/upper-armenian/css3-counter-styles-110.html) [(live test)](http://wpt.live/css/css-counter-styles/upper-armenian/css3-counter-styles-110.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/upper-armenian/css3-counter-styles-110.html)

<strong><a id="valdef-counter-style-name-lower-armenian"></a><strong>lower-armenian</strong></strong>

Lowercase Armenian numbering (e.g., ա, բ, գ, ..., ղը, ղթ, ճ).

Tests

- [css3-counter-styles-111.html](https://wpt.fyi/results/css/css-counter-styles/lower-armenian/css3-counter-styles-111.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-armenian/css3-counter-styles-111.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-armenian/css3-counter-styles-111.html)
- [css3-counter-styles-112.html](https://wpt.fyi/results/css/css-counter-styles/lower-armenian/css3-counter-styles-112.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-armenian/css3-counter-styles-112.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-armenian/css3-counter-styles-112.html)
- [css3-counter-styles-114.html](https://wpt.fyi/results/css/css-counter-styles/lower-armenian/css3-counter-styles-114.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-armenian/css3-counter-styles-114.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-armenian/css3-counter-styles-114.html)
- [css3-counter-styles-115.html](https://wpt.fyi/results/css/css-counter-styles/lower-armenian/css3-counter-styles-115.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-armenian/css3-counter-styles-115.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-armenian/css3-counter-styles-115.html)

<strong><a id="valdef-counter-style-name-bengali"></a><strong>bengali</strong></strong>

Bengali numbering (e.g., ১, ২, ৩, ..., ৯৮, ৯৯, ১০০).

Tests

- [css3-counter-styles-116.html](https://wpt.fyi/results/css/css-counter-styles/bengali/css3-counter-styles-116.html) [(live test)](http://wpt.live/css/css-counter-styles/bengali/css3-counter-styles-116.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/bengali/css3-counter-styles-116.html)
- [css3-counter-styles-117.html](https://wpt.fyi/results/css/css-counter-styles/bengali/css3-counter-styles-117.html) [(live test)](http://wpt.live/css/css-counter-styles/bengali/css3-counter-styles-117.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/bengali/css3-counter-styles-117.html)
- [css3-counter-styles-118.html](https://wpt.fyi/results/css/css-counter-styles/bengali/css3-counter-styles-118.html) [(live test)](http://wpt.live/css/css-counter-styles/bengali/css3-counter-styles-118.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/bengali/css3-counter-styles-118.html)

<strong><a id="valdef-counter-style-name-cambodian"></a><strong>cambodian</strong></strong>

<strong><a id="valdef-counter-style-name-khmer"></a><strong>khmer</strong></strong>

Cambodian/Khmer numbering (e.g., ១, ២, ៣, ..., ៩៨, ៩៩, ១០០).

Tests

- [css3-counter-styles-158.html](https://wpt.fyi/results/css/css-counter-styles/cambodian/css3-counter-styles-158.html) [(live test)](http://wpt.live/css/css-counter-styles/cambodian/css3-counter-styles-158.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cambodian/css3-counter-styles-158.html)
- [css3-counter-styles-159.html](https://wpt.fyi/results/css/css-counter-styles/cambodian/css3-counter-styles-159.html) [(live test)](http://wpt.live/css/css-counter-styles/cambodian/css3-counter-styles-159.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cambodian/css3-counter-styles-159.html)
- [css3-counter-styles-160.html](https://wpt.fyi/results/css/css-counter-styles/cambodian/css3-counter-styles-160.html) [(live test)](http://wpt.live/css/css-counter-styles/cambodian/css3-counter-styles-160.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cambodian/css3-counter-styles-160.html)
- [css3-counter-styles-161.html](https://wpt.fyi/results/css/css-counter-styles/khmer/css3-counter-styles-161.html) [(live test)](http://wpt.live/css/css-counter-styles/khmer/css3-counter-styles-161.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/khmer/css3-counter-styles-161.html)
- [css3-counter-styles-162.html](https://wpt.fyi/results/css/css-counter-styles/khmer/css3-counter-styles-162.html) [(live test)](http://wpt.live/css/css-counter-styles/khmer/css3-counter-styles-162.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/khmer/css3-counter-styles-162.html)
- [css3-counter-styles-163.html](https://wpt.fyi/results/css/css-counter-styles/khmer/css3-counter-styles-163.html) [(live test)](http://wpt.live/css/css-counter-styles/khmer/css3-counter-styles-163.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/khmer/css3-counter-styles-163.html)

<strong><a id="cjk-decimal"></a><strong>cjk-decimal</strong></strong>

Han decimal numbers (e.g., 一, 二, 三, ..., 九八, 九九, 一〇〇).

Tests

- [counter-cjk-decimal.html](https://wpt.fyi/results/css/css-counter-styles/cjk-decimal/counter-cjk-decimal.html) [(live test)](http://wpt.live/css/css-counter-styles/cjk-decimal/counter-cjk-decimal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cjk-decimal/counter-cjk-decimal.html)
- [css3-counter-styles-001.html](https://wpt.fyi/results/css/css-counter-styles/cjk-decimal/css3-counter-styles-001.html) [(live test)](http://wpt.live/css/css-counter-styles/cjk-decimal/css3-counter-styles-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cjk-decimal/css3-counter-styles-001.html)
- [css3-counter-styles-004.html](https://wpt.fyi/results/css/css-counter-styles/cjk-decimal/css3-counter-styles-004.html) [(live test)](http://wpt.live/css/css-counter-styles/cjk-decimal/css3-counter-styles-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cjk-decimal/css3-counter-styles-004.html)
- [css3-counter-styles-005.html](https://wpt.fyi/results/css/css-counter-styles/cjk-decimal/css3-counter-styles-005.html) [(live test)](http://wpt.live/css/css-counter-styles/cjk-decimal/css3-counter-styles-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cjk-decimal/css3-counter-styles-005.html)

<strong><a id="valdef-counter-style-name-devanagari"></a><strong>devanagari</strong></strong>

devanagari numbering (e.g., १, २, ३, ..., ९८, ९९, १००).

Tests

- [css3-counter-styles-119.html](https://wpt.fyi/results/css/css-counter-styles/devanagari/css3-counter-styles-119.html) [(live test)](http://wpt.live/css/css-counter-styles/devanagari/css3-counter-styles-119.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/devanagari/css3-counter-styles-119.html)
- [css3-counter-styles-120.html](https://wpt.fyi/results/css/css-counter-styles/devanagari/css3-counter-styles-120.html) [(live test)](http://wpt.live/css/css-counter-styles/devanagari/css3-counter-styles-120.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/devanagari/css3-counter-styles-120.html)
- [css3-counter-styles-121.html](https://wpt.fyi/results/css/css-counter-styles/devanagari/css3-counter-styles-121.html) [(live test)](http://wpt.live/css/css-counter-styles/devanagari/css3-counter-styles-121.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/devanagari/css3-counter-styles-121.html)

<strong><a id="georgian"></a><strong>georgian</strong></strong>

Traditional Georgian numbering (e.g., ა, ბ, გ, ..., ჟჱ, ჟთ, რ).

Tests

- [css3-counter-styles-010.html](https://wpt.fyi/results/css/css-counter-styles/georgian/css3-counter-styles-010.html) [(live test)](http://wpt.live/css/css-counter-styles/georgian/css3-counter-styles-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/georgian/css3-counter-styles-010.html)
- [css3-counter-styles-011.html](https://wpt.fyi/results/css/css-counter-styles/georgian/css3-counter-styles-011.html) [(live test)](http://wpt.live/css/css-counter-styles/georgian/css3-counter-styles-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/georgian/css3-counter-styles-011.html)
- [css3-counter-styles-012.html](https://wpt.fyi/results/css/css-counter-styles/georgian/css3-counter-styles-012.html) [(live test)](http://wpt.live/css/css-counter-styles/georgian/css3-counter-styles-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/georgian/css3-counter-styles-012.html)
- [css3-counter-styles-014.html](https://wpt.fyi/results/css/css-counter-styles/georgian/css3-counter-styles-014.html) [(live test)](http://wpt.live/css/css-counter-styles/georgian/css3-counter-styles-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/georgian/css3-counter-styles-014.html)

<strong><a id="valdef-counter-style-name-gujarati"></a><strong>gujarati</strong></strong>

Gujarati numbering (e.g., ૧, ૨, ૩, ..., ૯૮, ૯૯, ૧૦૦).

Tests

- [css3-counter-styles-122.html](https://wpt.fyi/results/css/css-counter-styles/gujarati/css3-counter-styles-122.html) [(live test)](http://wpt.live/css/css-counter-styles/gujarati/css3-counter-styles-122.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/gujarati/css3-counter-styles-122.html)
- [css3-counter-styles-123.html](https://wpt.fyi/results/css/css-counter-styles/gujarati/css3-counter-styles-123.html) [(live test)](http://wpt.live/css/css-counter-styles/gujarati/css3-counter-styles-123.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/gujarati/css3-counter-styles-123.html)
- [css3-counter-styles-124.html](https://wpt.fyi/results/css/css-counter-styles/gujarati/css3-counter-styles-124.html) [(live test)](http://wpt.live/css/css-counter-styles/gujarati/css3-counter-styles-124.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/gujarati/css3-counter-styles-124.html)

<strong><a id="valdef-counter-style-name-gurmukhi"></a><strong>gurmukhi</strong></strong>

Gurmukhi numbering (e.g., ੧, ੨, ੩, ..., ੯੮, ੯੯, ੧੦੦).

Tests

- [css3-counter-styles-125.html](https://wpt.fyi/results/css/css-counter-styles/gurmukhi/css3-counter-styles-125.html) [(live test)](http://wpt.live/css/css-counter-styles/gurmukhi/css3-counter-styles-125.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/gurmukhi/css3-counter-styles-125.html)
- [css3-counter-styles-126.html](https://wpt.fyi/results/css/css-counter-styles/gurmukhi/css3-counter-styles-126.html) [(live test)](http://wpt.live/css/css-counter-styles/gurmukhi/css3-counter-styles-126.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/gurmukhi/css3-counter-styles-126.html)
- [css3-counter-styles-127.html](https://wpt.fyi/results/css/css-counter-styles/gurmukhi/css3-counter-styles-127.html) [(live test)](http://wpt.live/css/css-counter-styles/gurmukhi/css3-counter-styles-127.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/gurmukhi/css3-counter-styles-127.html)

<strong><a id="hebrew"></a><strong>hebrew</strong></strong>

Traditional Hebrew numbering (e.g., א‎, ב‎, ג‎, ..., צח‎, צט‎, ק‎).

Tests

- [counter-hebrew-nested.html](https://wpt.fyi/results/css/css-counter-styles/hebrew/counter-hebrew-nested.html) [(live test)](http://wpt.live/css/css-counter-styles/hebrew/counter-hebrew-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hebrew/counter-hebrew-nested.html)
- [css3-counter-styles-015.html](https://wpt.fyi/results/css/css-counter-styles/hebrew/css3-counter-styles-015.html) [(live test)](http://wpt.live/css/css-counter-styles/hebrew/css3-counter-styles-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hebrew/css3-counter-styles-015.html)
- [css3-counter-styles-016.html](https://wpt.fyi/results/css/css-counter-styles/hebrew/css3-counter-styles-016.html) [(live test)](http://wpt.live/css/css-counter-styles/hebrew/css3-counter-styles-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hebrew/css3-counter-styles-016.html)
- [css3-counter-styles-016a.html](https://wpt.fyi/results/css/css-counter-styles/hebrew/css3-counter-styles-016a.html) [(live test)](http://wpt.live/css/css-counter-styles/hebrew/css3-counter-styles-016a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hebrew/css3-counter-styles-016a.html)
- [css3-counter-styles-017.html](https://wpt.fyi/results/css/css-counter-styles/hebrew/css3-counter-styles-017.html) [(live test)](http://wpt.live/css/css-counter-styles/hebrew/css3-counter-styles-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hebrew/css3-counter-styles-017.html)

<strong><a id="valdef-counter-style-name-kannada"></a><strong>kannada</strong></strong>

Kannada numbering (e.g., ೧, ೨, ೩, ..., ೯೮, ೯೯, ೧೦೦).

Tests

- [css3-counter-styles-128.html](https://wpt.fyi/results/css/css-counter-styles/kannada/css3-counter-styles-128.html) [(live test)](http://wpt.live/css/css-counter-styles/kannada/css3-counter-styles-128.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/kannada/css3-counter-styles-128.html)
- [css3-counter-styles-129.html](https://wpt.fyi/results/css/css-counter-styles/kannada/css3-counter-styles-129.html) [(live test)](http://wpt.live/css/css-counter-styles/kannada/css3-counter-styles-129.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/kannada/css3-counter-styles-129.html)
- [css3-counter-styles-130.html](https://wpt.fyi/results/css/css-counter-styles/kannada/css3-counter-styles-130.html) [(live test)](http://wpt.live/css/css-counter-styles/kannada/css3-counter-styles-130.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/kannada/css3-counter-styles-130.html)

<strong><a id="valdef-counter-style-name-lao"></a><strong>lao</strong></strong>

Laotian numbering (e.g., ໑, ໒, ໓, ..., ໙໘, ໙໙, ໑໐໐).

Tests

- [css3-counter-styles-131.html](https://wpt.fyi/results/css/css-counter-styles/lao/css3-counter-styles-131.html) [(live test)](http://wpt.live/css/css-counter-styles/lao/css3-counter-styles-131.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lao/css3-counter-styles-131.html)
- [css3-counter-styles-132.html](https://wpt.fyi/results/css/css-counter-styles/lao/css3-counter-styles-132.html) [(live test)](http://wpt.live/css/css-counter-styles/lao/css3-counter-styles-132.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lao/css3-counter-styles-132.html)
- [css3-counter-styles-133.html](https://wpt.fyi/results/css/css-counter-styles/lao/css3-counter-styles-133.html) [(live test)](http://wpt.live/css/css-counter-styles/lao/css3-counter-styles-133.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lao/css3-counter-styles-133.html)

<strong><a id="valdef-counter-style-name-malayalam"></a><strong>malayalam</strong></strong>

Malayalam numbering (e.g., ൧, ൨, ൩, ..., ൯൮, ൯൯, ൧൦൦).

Tests

- [css3-counter-styles-134.html](https://wpt.fyi/results/css/css-counter-styles/malayalam/css3-counter-styles-134.html) [(live test)](http://wpt.live/css/css-counter-styles/malayalam/css3-counter-styles-134.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/malayalam/css3-counter-styles-134.html)
- [css3-counter-styles-135.html](https://wpt.fyi/results/css/css-counter-styles/malayalam/css3-counter-styles-135.html) [(live test)](http://wpt.live/css/css-counter-styles/malayalam/css3-counter-styles-135.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/malayalam/css3-counter-styles-135.html)
- [css3-counter-styles-136.html](https://wpt.fyi/results/css/css-counter-styles/malayalam/css3-counter-styles-136.html) [(live test)](http://wpt.live/css/css-counter-styles/malayalam/css3-counter-styles-136.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/malayalam/css3-counter-styles-136.html)

<strong><a id="valdef-counter-style-name-mongolian"></a><strong>mongolian</strong></strong>

Mongolian numbering (e.g., ᠑, ᠒, ᠓, ..., ᠙᠘, ᠙᠙, ᠑᠐᠐).

Tests

- [css3-counter-styles-137.html](https://wpt.fyi/results/css/css-counter-styles/mongolian/css3-counter-styles-137.html) [(live test)](http://wpt.live/css/css-counter-styles/mongolian/css3-counter-styles-137.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/mongolian/css3-counter-styles-137.html)
- [css3-counter-styles-138.html](https://wpt.fyi/results/css/css-counter-styles/mongolian/css3-counter-styles-138.html) [(live test)](http://wpt.live/css/css-counter-styles/mongolian/css3-counter-styles-138.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/mongolian/css3-counter-styles-138.html)
- [css3-counter-styles-139.html](https://wpt.fyi/results/css/css-counter-styles/mongolian/css3-counter-styles-139.html) [(live test)](http://wpt.live/css/css-counter-styles/mongolian/css3-counter-styles-139.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/mongolian/css3-counter-styles-139.html)

<strong><a id="valdef-counter-style-name-myanmar"></a><strong>myanmar</strong></strong>

Myanmar (Burmese) numbering (e.g., ၁, ၂, ၃, ..., ၉၈, ၉၉, ၁၀၀).

Tests

- [css3-counter-styles-140.html](https://wpt.fyi/results/css/css-counter-styles/myanmar/css3-counter-styles-140.html) [(live test)](http://wpt.live/css/css-counter-styles/myanmar/css3-counter-styles-140.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/myanmar/css3-counter-styles-140.html)
- [css3-counter-styles-141.html](https://wpt.fyi/results/css/css-counter-styles/myanmar/css3-counter-styles-141.html) [(live test)](http://wpt.live/css/css-counter-styles/myanmar/css3-counter-styles-141.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/myanmar/css3-counter-styles-141.html)
- [css3-counter-styles-142.html](https://wpt.fyi/results/css/css-counter-styles/myanmar/css3-counter-styles-142.html) [(live test)](http://wpt.live/css/css-counter-styles/myanmar/css3-counter-styles-142.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/myanmar/css3-counter-styles-142.html)

<strong><a id="valdef-counter-style-name-oriya"></a><strong>oriya</strong></strong>

Oriya (Odia) numbering (e.g., ୧, ୨, ୩, ..., ୯୮, ୯୯, ୧୦୦).

Tests

- [css3-counter-styles-143.html](https://wpt.fyi/results/css/css-counter-styles/oriya/css3-counter-styles-143.html) [(live test)](http://wpt.live/css/css-counter-styles/oriya/css3-counter-styles-143.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/oriya/css3-counter-styles-143.html)
- [css3-counter-styles-144.html](https://wpt.fyi/results/css/css-counter-styles/oriya/css3-counter-styles-144.html) [(live test)](http://wpt.live/css/css-counter-styles/oriya/css3-counter-styles-144.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/oriya/css3-counter-styles-144.html)
- [css3-counter-styles-145.html](https://wpt.fyi/results/css/css-counter-styles/oriya/css3-counter-styles-145.html) [(live test)](http://wpt.live/css/css-counter-styles/oriya/css3-counter-styles-145.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/oriya/css3-counter-styles-145.html)

<strong><a id="valdef-counter-style-name-persian"></a><strong>persian</strong></strong>

Persian numbering (e.g., ۱, ۲, ۳, ۴, ..., ۹۸, ۹۹, ۱۰۰).

Tests

- [css3-counter-styles-104.html](https://wpt.fyi/results/css/css-counter-styles/persian/css3-counter-styles-104.html) [(live test)](http://wpt.live/css/css-counter-styles/persian/css3-counter-styles-104.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/persian/css3-counter-styles-104.html)
- [css3-counter-styles-105.html](https://wpt.fyi/results/css/css-counter-styles/persian/css3-counter-styles-105.html) [(live test)](http://wpt.live/css/css-counter-styles/persian/css3-counter-styles-105.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/persian/css3-counter-styles-105.html)
- [css3-counter-styles-106.html](https://wpt.fyi/results/css/css-counter-styles/persian/css3-counter-styles-106.html) [(live test)](http://wpt.live/css/css-counter-styles/persian/css3-counter-styles-106.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/persian/css3-counter-styles-106.html)

<strong><a id="lower-roman"></a><strong>lower-roman</strong></strong>

Lowercase ASCII Roman numerals (e.g., i, ii, iii, ..., xcviii, xcix, c).

Tests

- [css3-counter-styles-019.html](https://wpt.fyi/results/css/css-counter-styles/lower-roman/css3-counter-styles-019.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-roman/css3-counter-styles-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-roman/css3-counter-styles-019.html)
- [css3-counter-styles-020.html](https://wpt.fyi/results/css/css-counter-styles/lower-roman/css3-counter-styles-020.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-roman/css3-counter-styles-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-roman/css3-counter-styles-020.html)
- [css3-counter-styles-020a.html](https://wpt.fyi/results/css/css-counter-styles/lower-roman/css3-counter-styles-020a.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-roman/css3-counter-styles-020a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-roman/css3-counter-styles-020a.html)
- [css3-counter-styles-020b.html](https://wpt.fyi/results/css/css-counter-styles/lower-roman/css3-counter-styles-020b.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-roman/css3-counter-styles-020b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-roman/css3-counter-styles-020b.html)
- [css3-counter-styles-021.html](https://wpt.fyi/results/css/css-counter-styles/lower-roman/css3-counter-styles-021.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-roman/css3-counter-styles-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-roman/css3-counter-styles-021.html)
- [css3-counter-styles-022.html](https://wpt.fyi/results/css/css-counter-styles/lower-roman/css3-counter-styles-022.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-roman/css3-counter-styles-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-roman/css3-counter-styles-022.html)

<strong><a id="upper-roman"></a><strong>upper-roman</strong></strong>

Uppercase ASCII Roman numerals (e.g., I, II, III, ..., XCVIII, XCIX, C).

Tests

- [css3-counter-styles-023.html](https://wpt.fyi/results/css/css-counter-styles/upper-roman/css3-counter-styles-023.html) [(live test)](http://wpt.live/css/css-counter-styles/upper-roman/css3-counter-styles-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/upper-roman/css3-counter-styles-023.html)
- [css3-counter-styles-024.html](https://wpt.fyi/results/css/css-counter-styles/upper-roman/css3-counter-styles-024.html) [(live test)](http://wpt.live/css/css-counter-styles/upper-roman/css3-counter-styles-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/upper-roman/css3-counter-styles-024.html)
- [css3-counter-styles-024a.html](https://wpt.fyi/results/css/css-counter-styles/upper-roman/css3-counter-styles-024a.html) [(live test)](http://wpt.live/css/css-counter-styles/upper-roman/css3-counter-styles-024a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/upper-roman/css3-counter-styles-024a.html)
- [css3-counter-styles-025.html](https://wpt.fyi/results/css/css-counter-styles/upper-roman/css3-counter-styles-025.html) [(live test)](http://wpt.live/css/css-counter-styles/upper-roman/css3-counter-styles-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/upper-roman/css3-counter-styles-025.html)
- [css3-counter-styles-026.html](https://wpt.fyi/results/css/css-counter-styles/upper-roman/css3-counter-styles-026.html) [(live test)](http://wpt.live/css/css-counter-styles/upper-roman/css3-counter-styles-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/upper-roman/css3-counter-styles-026.html)

<strong><a id="valdef-counter-style-name-tamil"></a><strong>tamil</strong></strong>

Tamil numbering (e.g., ௧, ௨, ௩, ..., ௯௮, ௯௯, ௧௦௦).

Tests

- [css3-counter-styles-146.html](https://wpt.fyi/results/css/css-counter-styles/tamil/css3-counter-styles-146.html) [(live test)](http://wpt.live/css/css-counter-styles/tamil/css3-counter-styles-146.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/tamil/css3-counter-styles-146.html)
- [css3-counter-styles-147.html](https://wpt.fyi/results/css/css-counter-styles/tamil/css3-counter-styles-147.html) [(live test)](http://wpt.live/css/css-counter-styles/tamil/css3-counter-styles-147.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/tamil/css3-counter-styles-147.html)
- [css3-counter-styles-148.html](https://wpt.fyi/results/css/css-counter-styles/tamil/css3-counter-styles-148.html) [(live test)](http://wpt.live/css/css-counter-styles/tamil/css3-counter-styles-148.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/tamil/css3-counter-styles-148.html)

<strong><a id="valdef-counter-style-name-telugu"></a><strong>telugu</strong></strong>

Telugu numbering (e.g., ౧, ౨, ౩, ..., ౯౮, ౯౯, ౧౦౦).

Tests

- [css3-counter-styles-149.html](https://wpt.fyi/results/css/css-counter-styles/telugu/css3-counter-styles-149.html) [(live test)](http://wpt.live/css/css-counter-styles/telugu/css3-counter-styles-149.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/telugu/css3-counter-styles-149.html)
- [css3-counter-styles-150.html](https://wpt.fyi/results/css/css-counter-styles/telugu/css3-counter-styles-150.html) [(live test)](http://wpt.live/css/css-counter-styles/telugu/css3-counter-styles-150.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/telugu/css3-counter-styles-150.html)
- [css3-counter-styles-151.html](https://wpt.fyi/results/css/css-counter-styles/telugu/css3-counter-styles-151.html) [(live test)](http://wpt.live/css/css-counter-styles/telugu/css3-counter-styles-151.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/telugu/css3-counter-styles-151.html)

<strong><a id="valdef-counter-style-name-thai"></a><strong>thai</strong></strong>

Thai (Siamese) numbering (e.g., ๑, ๒, ๓, ..., ๙๘, ๙๙, ๑๐๐).

Tests

- [css3-counter-styles-152.html](https://wpt.fyi/results/css/css-counter-styles/thai/css3-counter-styles-152.html) [(live test)](http://wpt.live/css/css-counter-styles/thai/css3-counter-styles-152.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/thai/css3-counter-styles-152.html)
- [css3-counter-styles-153.html](https://wpt.fyi/results/css/css-counter-styles/thai/css3-counter-styles-153.html) [(live test)](http://wpt.live/css/css-counter-styles/thai/css3-counter-styles-153.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/thai/css3-counter-styles-153.html)
- [css3-counter-styles-154.html](https://wpt.fyi/results/css/css-counter-styles/thai/css3-counter-styles-154.html) [(live test)](http://wpt.live/css/css-counter-styles/thai/css3-counter-styles-154.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/thai/css3-counter-styles-154.html)

<strong><a id="valdef-counter-style-name-tibetan"></a><strong>tibetan</strong></strong>

Tibetan numbering (e.g., ༡, ༢, ༣, ..., ༩༨, ༩༩, ༡༠༠).

Tests

- [css3-counter-styles-155.html](https://wpt.fyi/results/css/css-counter-styles/tibetan/css3-counter-styles-155.html) [(live test)](http://wpt.live/css/css-counter-styles/tibetan/css3-counter-styles-155.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/tibetan/css3-counter-styles-155.html)
- [css3-counter-styles-156.html](https://wpt.fyi/results/css/css-counter-styles/tibetan/css3-counter-styles-156.html) [(live test)](http://wpt.live/css/css-counter-styles/tibetan/css3-counter-styles-156.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/tibetan/css3-counter-styles-156.html)
- [css3-counter-styles-157.html](https://wpt.fyi/results/css/css-counter-styles/tibetan/css3-counter-styles-157.html) [(live test)](http://wpt.live/css/css-counter-styles/tibetan/css3-counter-styles-157.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/tibetan/css3-counter-styles-157.html)

The following stylesheet fragment provides the normative definition of these predefined counter styles:

``` text

  @counter-style decimal {
    system: numeric;
    symbols: '0' '1' '2' '3' '4' '5' '6' '7' '8' '9';
  }

  @counter-style decimal-leading-zero {
    system: extends decimal;
    pad: 2 '0';
  }

  @counter-style arabic-indic {
    system: numeric;
    symbols: "\660" "\661" "\662" "\663" "\664" "\665" "\666" "\667" "\668" "\669";
    /* ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩ */
  }

  @counter-style armenian {
    system: additive;
    range: 1 9999;
    additive-symbols: 9000 \554, 8000 \553, 7000 \552, 6000 \551, 5000 \550, 4000 \54F, 3000 \54E, 2000 \54D, 1000 \54C, 900 \54B, 800 \54A, 700 \549, 600 \548, 500 \547, 400 \546, 300 \545, 200 \544, 100 \543, 90 \542, 80 \541, 70 \540, 60 \53F, 50 \53E, 40 \53D, 30 \53C, 20 \53B, 10 \53A, 9 \539, 8 \538, 7 \537, 6 \536, 5 \535, 4 \534, 3 \533, 2 \532, 1 \531;
    /* 9000 Ք, 8000 Փ, 7000 Ւ, 6000 Ց, 5000 Ր, 4000 Տ, 3000 Վ, 2000 Ս, 1000 Ռ, 900 Ջ, 800 Պ, 700 Չ, 600 Ո, 500 Շ, 400 Ն, 300 Յ, 200 Մ, 100 Ճ, 90 Ղ, 80 Ձ, 70 Հ, 60 Կ, 50 Ծ, 40 Խ, 30 Լ, 20 Ի, 10 Ժ, 9 Թ, 8 Ը, 7 Է, 6 Զ, 5 Ե, 4 Դ, 3 Գ, 2 Բ, 1 Ա */
  }

  @counter-style upper-armenian {
    system: extends armenian;
  }

  @counter-style lower-armenian {
    system: additive;
    range: 1 9999;
    additive-symbols: 9000 "\584", 8000 "\583", 7000 "\582", 6000 "\581", 5000 "\580", 4000 "\57F", 3000 "\57E", 2000 "\57D", 1000 "\57C", 900 "\57B", 800 "\57A", 700 "\579", 600 "\578", 500 "\577", 400 "\576", 300 "\575", 200 "\574", 100 "\573", 90 "\572", 80 "\571", 70 "\570", 60 "\56F", 50 "\56E", 40 "\56D", 30 "\56C", 20 "\56B", 10 "\56A", 9 "\569", 8 "\568", 7 "\567", 6 "\566", 5 "\565", 4 "\564", 3 "\563", 2 "\562", 1 "\561";
    /* 9000 ք, 8000 փ, 7000 ւ, 6000 ց, 5000 ր, 4000 տ, 3000 վ, 2000 ս, 1000 ռ, 900 ջ, 800 պ, 700 չ, 600 ո, 500 շ, 400 ն, 300 յ, 200 մ, 100 ճ, 90 ղ, 80 ձ, 70 հ, 60 կ, 50 ծ, 40 խ, 30 լ, 20 ի, 10 ժ, 9 թ, 8 ը, 7 է, 6 զ, 5 ե, 4 դ, 3 գ, 2 բ, 1 ա */
  }

  @counter-style bengali {
    system: numeric;
    symbols: "\9E6" "\9E7" "\9E8" "\9E9" "\9EA" "\9EB" "\9EC" "\9ED" "\9EE" "\9EF";
    /* ০ ১ ২ ৩ ৪ ৫ ৬ ৭ ৮ ৯ */
  }

  @counter-style cambodian {
    system: numeric;
    symbols: "\17E0" "\17E1" "\17E2" "\17E3" "\17E4" "\17E5" "\17E6" "\17E7" "\17E8" "\17E9";
    /* ០ ១ ២ ៣ ៤ ៥ ៦ ៧ ៨ ៩ */
  }

  @counter-style khmer {
    system: extends cambodian;
  }

  @counter-style cjk-decimal {
    system: numeric;
    range: 0 infinite;
    symbols: \3007  \4E00  \4E8C  \4E09  \56DB  \4E94  \516D  \4E03  \516B  \4E5D;
    /* 〇 一 二 三 四 五 六 七 八 九 */
    suffix: "\3001";
    /* "、" */
  }

  @counter-style devanagari {
    system: numeric;
    symbols: "\966" "\967" "\968" "\969" "\96A" "\96B" "\96C" "\96D" "\96E" "\96F";
    /* ० १ २ ३ ४ ५ ६ ७ ८ ९ */
  }

  @counter-style georgian {
    system: additive;
    range: 1 19999;
    additive-symbols: 10000 \10F5, 9000 \10F0, 8000 \10EF, 7000 \10F4, 6000 \10EE, 5000 \10ED, 4000 \10EC, 3000 \10EB, 2000 \10EA, 1000 \10E9, 900 \10E8, 800 \10E7, 700 \10E6, 600 \10E5, 500 \10E4, 400 \10F3, 300 \10E2, 200 \10E1, 100 \10E0, 90 \10DF, 80 \10DE, 70 \10DD, 60 \10F2, 50 \10DC, 40 \10DB, 30 \10DA, 20 \10D9, 10 \10D8, 9 \10D7, 8 \10F1, 7 \10D6, 6 \10D5, 5 \10D4, 4 \10D3, 3 \10D2, 2 \10D1, 1 \10D0;
    /* 10000 ჵ, 9000 ჰ, 8000 ჯ, 7000 ჴ, 6000 ხ, 5000 ჭ, 4000 წ, 3000 ძ, 2000 ც, 1000 ჩ, 900 შ, 800 ყ, 700 ღ, 600 ქ, 500 ფ, 400 ჳ, 300 ტ, 200 ს, 100 რ, 90 ჟ, 80 პ, 70 ო, 60 ჲ, 50 ნ, 40 მ, 30 ლ, 20 კ, 10 ი, 9 თ, 8 ჱ, 7 ზ, 6 ვ, 5 ე, 4 დ, 3 გ, 2 ბ, 1 ა */
  }

  @counter-style gujarati {
    system: numeric;
    symbols: "\AE6" "\AE7" "\AE8" "\AE9" "\AEA" "\AEB" "\AEC" "\AED" "\AEE" "\AEF";
    /* ૦ ૧ ૨ ૩ ૪ ૫ ૬ ૭ ૮ ૯ */
  }

  @counter-style gurmukhi {
    system: numeric;
    symbols: "\A66" "\A67" "\A68" "\A69" "\A6A" "\A6B" "\A6C" "\A6D" "\A6E" "\A6F";
    /* ੦ ੧ ੨ ੩ ੪ ੫ ੬ ੭ ੮ ੯ */
  }

  @counter-style hebrew {
    system: additive;
    range: 1 10999;
    additive-symbols: 10000 \5D9\5F3, 9000 \5D8\5F3, 8000 \5D7\5F3, 7000 \5D6\5F3, 6000 \5D5\5F3, 5000 \5D4\5F3, 4000 \5D3\5F3, 3000 \5D2\5F3, 2000 \5D1\5F3, 1000 \5D0\5F3, 400 \5EA, 300 \5E9, 200 \5E8, 100 \5E7, 90 \5E6, 80 \5E4, 70 \5E2, 60 \5E1, 50 \5E0, 40 \5DE, 30 \5DC, 20 \5DB, 19 \5D9\5D8, 18 \5D9\5D7, 17 \5D9\5D6, 16 \5D8\5D6, 15 \5D8\5D5, 10 \5D9, 9 \5D8, 8 \5D7, 7 \5D6, 6 \5D5, 5 \5D4, 4 \5D3, 3 \5D2, 2 \5D1, 1 \5D0;
    /* 10000 י׳, 9000 ט׳, 8000 ח׳, 7000 ז׳, 6000 ו׳, 5000 ה׳, 4000 ד׳, 3000 ג׳, 2000 ב׳, 1000 א׳, 400 ת, 300 ש, 200 ר, 100 ק, 90 צ, 80 פ, 70 ע, 60 ס, 50 נ, 40 מ, 30 ל, 20 כ, 19 יט, 18 יח, 17 יז, 16 טז, 15 טו, 10 י, 9 ט, 8 ח, 7 ז, 6 ו, 5 ה, 4 ד, 3 ג, 2 ב, 1 א */
    /* This system manually specifies the values for 19-15 to force the correct display of 15 and 16, which are commonly rewritten to avoid a close resemblance to the Tetragrammaton. */
    /* Implementations MAY choose to implement this manually to a higher range; see note below. */
  }

  @counter-style kannada {
    system: numeric;
    symbols: "\CE6" "\CE7" "\CE8" "\CE9" "\CEA" "\CEB" "\CEC" "\CED" "\CEE" "\CEF";
    /* ೦ ೧ ೨ ೩ ೪ ೫ ೬ ೭ ೮ ೯ */
  }

  @counter-style lao {
    system: numeric;
    symbols: "\ED0" "\ED1" "\ED2" "\ED3" "\ED4" "\ED5" "\ED6" "\ED7" "\ED8" "\ED9";
    /* ໐ ໑ ໒ ໓ ໔ ໕ ໖ ໗ ໘ ໙ */
  }

  @counter-style malayalam {
    system: numeric;
    symbols: "\D66" "\D67" "\D68" "\D69" "\D6A" "\D6B" "\D6C" "\D6D" "\D6E" "\D6F";
    /* ൦ ൧ ൨ ൩ ൪ ൫ ൬ ൭ ൮ ൯ */
  }

  @counter-style mongolian {
    system: numeric;
    symbols: "\1810" "\1811" "\1812" "\1813" "\1814" "\1815" "\1816" "\1817" "\1818" "\1819";
    /* ᠐ ᠑ ᠒ ᠓ ᠔ ᠕ ᠖ ᠗ ᠘ ᠙ */
  }

  @counter-style myanmar {
    system: numeric;
    symbols: "\1040" "\1041" "\1042" "\1043" "\1044" "\1045" "\1046" "\1047" "\1048" "\1049";
    /* ၀ ၁ ၂ ၃ ၄ ၅ ၆ ၇ ၈ ၉ */
  }

  @counter-style oriya {
    system: numeric;
    symbols: "\B66" "\B67" "\B68" "\B69" "\B6A" "\B6B" "\B6C" "\B6D" "\B6E" "\B6F";
    /* ୦ ୧ ୨ ୩ ୪ ୫ ୬ ୭ ୮ ୯ */
  }

  @counter-style persian {
    system: numeric;
    symbols: "\6F0" "\6F1" "\6F2" "\6F3" "\6F4" "\6F5" "\6F6" "\6F7" "\6F8" "\6F9";
    /* ۰ ۱ ۲ ۳ ۴ ۵ ۶ ۷ ۸ ۹ */
  }

  @counter-style lower-roman {
    system: additive;
    range: 1 3999;
    additive-symbols: 1000 m, 900 cm, 500 d, 400 cd, 100 c, 90 xc, 50 l, 40 xl, 10 x, 9 ix, 5 v, 4 iv, 1 i;
  }

  @counter-style upper-roman {
    system: additive;
    range: 1 3999;
    additive-symbols: 1000 M, 900 CM, 500 D, 400 CD, 100 C, 90 XC, 50 L, 40 XL, 10 X, 9 IX, 5 V, 4 IV, 1 I;
  }

  @counter-style tamil {
    system: numeric;
    symbols: "\BE6" "\BE7" "\BE8" "\BE9" "\BEA" "\BEB" "\BEC" "\BED" "\BEE" "\BEF";
    /* ௦ ௧ ௨ ௩ ௪ ௫ ௬ ௭ ௮ ௯ */
  }

  @counter-style telugu {
    system: numeric;
    symbols: "\C66" "\C67" "\C68" "\C69" "\C6A" "\C6B" "\C6C" "\C6D" "\C6E" "\C6F";
    /* ౦ ౧ ౨ ౩ ౪ ౫ ౬ ౭ ౮ ౯ */
  }

  @counter-style thai {
    system: numeric;
    symbols: "\E50" "\E51" "\E52" "\E53" "\E54" "\E55" "\E56" "\E57" "\E58" "\E59";
    /* ๐ ๑ ๒ ๓ ๔ ๕ ๖ ๗ ๘ ๙ */
  }

  @counter-style tibetan {
    system: numeric;
    symbols: "\F20" "\F21" "\F22" "\F23" "\F24" "\F25" "\F26" "\F27" "\F28" "\F29";
    /* ༠ ༡ ༢ ༣ ༤ ༥ ༦ ༧ ༨ ༩ */
  }
  
```

Implementations must implement <a id="ref-for-hebrew①"></a>[hebrew](#hebrew) at least to the range specified in the <a id="ref-for-at-ruledef-counter-style③⑦"></a>[&#64;counter-style](#at-ruledef-counter-style) rule above, but may implement it to a higher range. If they do so, the corresponding <a id="ref-for-descdef-counter-style-range④"></a>[range](#descdef-counter-style-range) descriptor must reflect the implemented range.

### <a id="simple-alphabetic"></a>6.2.  Alphabetic: <a id="ref-for-lower-alpha①"></a>[lower-alpha](#lower-alpha), <a id="ref-for-lower-latin①"></a>[lower-latin](#lower-latin), <a id="ref-for-upper-alpha④"></a>[upper-alpha](#upper-alpha), <a id="ref-for-upper-latin①"></a>[upper-latin](#upper-latin), <a id="ref-for-lower-greek①"></a>[lower-greek](#lower-greek), <a id="ref-for-hiragana"></a>[hiragana](#hiragana), <a id="ref-for-hiragana-iroha"></a>[hiragana-iroha](#hiragana-iroha), <a id="ref-for-katakana"></a>[katakana](#katakana), <a id="ref-for-katakana-iroha"></a>[katakana-iroha](#katakana-iroha)[](#simple-alphabetic)

<strong><a id="lower-alpha"></a><strong>lower-alpha</strong></strong>

<strong><a id="lower-latin"></a><strong>lower-latin</strong></strong>

Lowercase ASCII letters (e.g., a, b, c, ..., z, aa, ab).

<strong><a id="upper-alpha"></a><strong>upper-alpha</strong></strong>

<strong><a id="upper-latin"></a><strong>upper-latin</strong></strong>

Uppercase ASCII letters (e.g., A, B, C, ..., Z, AA, AB).

<strong><a id="lower-greek"></a><strong>lower-greek</strong></strong>

Lowercase classical Greek (e.g., α, β, γ, ..., ω, αα, αβ).

Tests

- [css3-counter-styles-027.html](https://wpt.fyi/results/css/css-counter-styles/lower-greek/css3-counter-styles-027.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-greek/css3-counter-styles-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-greek/css3-counter-styles-027.html)
- [css3-counter-styles-028.html](https://wpt.fyi/results/css/css-counter-styles/lower-greek/css3-counter-styles-028.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-greek/css3-counter-styles-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-greek/css3-counter-styles-028.html)
- [css3-counter-styles-029.html](https://wpt.fyi/results/css/css-counter-styles/lower-greek/css3-counter-styles-029.html) [(live test)](http://wpt.live/css/css-counter-styles/lower-greek/css3-counter-styles-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/lower-greek/css3-counter-styles-029.html)

<strong><a id="hiragana"></a><strong>hiragana</strong></strong>

Dictionary-order hiragana lettering (e.g., あ, い, う, ..., ん, ああ, あい).

Tests

- [css3-counter-styles-030.html](https://wpt.fyi/results/css/css-counter-styles/hiragana/css3-counter-styles-030.html) [(live test)](http://wpt.live/css/css-counter-styles/hiragana/css3-counter-styles-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hiragana/css3-counter-styles-030.html)
- [css3-counter-styles-031.html](https://wpt.fyi/results/css/css-counter-styles/hiragana/css3-counter-styles-031.html) [(live test)](http://wpt.live/css/css-counter-styles/hiragana/css3-counter-styles-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hiragana/css3-counter-styles-031.html)
- [css3-counter-styles-032.html](https://wpt.fyi/results/css/css-counter-styles/hiragana/css3-counter-styles-032.html) [(live test)](http://wpt.live/css/css-counter-styles/hiragana/css3-counter-styles-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hiragana/css3-counter-styles-032.html)

<strong><a id="hiragana-iroha"></a><strong>hiragana-iroha</strong></strong>

Iroha-order hiragana lettering (e.g., い, ろ, は, ..., す, いい, いろ).

Tests

- [css3-counter-styles-033.html](https://wpt.fyi/results/css/css-counter-styles/hiragana-iroha/css3-counter-styles-033.html) [(live test)](http://wpt.live/css/css-counter-styles/hiragana-iroha/css3-counter-styles-033.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hiragana-iroha/css3-counter-styles-033.html)
- [css3-counter-styles-034.html](https://wpt.fyi/results/css/css-counter-styles/hiragana-iroha/css3-counter-styles-034.html) [(live test)](http://wpt.live/css/css-counter-styles/hiragana-iroha/css3-counter-styles-034.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hiragana-iroha/css3-counter-styles-034.html)
- [css3-counter-styles-035.html](https://wpt.fyi/results/css/css-counter-styles/hiragana-iroha/css3-counter-styles-035.html) [(live test)](http://wpt.live/css/css-counter-styles/hiragana-iroha/css3-counter-styles-035.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/hiragana-iroha/css3-counter-styles-035.html)

<strong><a id="katakana"></a><strong>katakana</strong></strong>

Dictionary-order katakana lettering (e.g., ア, イ, ウ, ..., ン, アア, アイ).

Tests

- [css3-counter-styles-036.html](https://wpt.fyi/results/css/css-counter-styles/katakana/css3-counter-styles-036.html) [(live test)](http://wpt.live/css/css-counter-styles/katakana/css3-counter-styles-036.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/katakana/css3-counter-styles-036.html)
- [css3-counter-styles-037.html](https://wpt.fyi/results/css/css-counter-styles/katakana/css3-counter-styles-037.html) [(live test)](http://wpt.live/css/css-counter-styles/katakana/css3-counter-styles-037.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/katakana/css3-counter-styles-037.html)
- [css3-counter-styles-038.html](https://wpt.fyi/results/css/css-counter-styles/katakana/css3-counter-styles-038.html) [(live test)](http://wpt.live/css/css-counter-styles/katakana/css3-counter-styles-038.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/katakana/css3-counter-styles-038.html)

<strong><a id="katakana-iroha"></a><strong>katakana-iroha</strong></strong>

Iroha-order katakana lettering (e.g., イ, ロ, ハ, ..., ス, イイ, イロ)

Tests

- [css3-counter-styles-039.html](https://wpt.fyi/results/css/css-counter-styles/katakana-iroha/css3-counter-styles-039.html) [(live test)](http://wpt.live/css/css-counter-styles/katakana-iroha/css3-counter-styles-039.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/katakana-iroha/css3-counter-styles-039.html)
- [css3-counter-styles-040.html](https://wpt.fyi/results/css/css-counter-styles/katakana-iroha/css3-counter-styles-040.html) [(live test)](http://wpt.live/css/css-counter-styles/katakana-iroha/css3-counter-styles-040.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/katakana-iroha/css3-counter-styles-040.html)
- [css3-counter-styles-041.html](https://wpt.fyi/results/css/css-counter-styles/katakana-iroha/css3-counter-styles-041.html) [(live test)](http://wpt.live/css/css-counter-styles/katakana-iroha/css3-counter-styles-041.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/katakana-iroha/css3-counter-styles-041.html)

The following stylesheet fragment provides the normative definition of these predefined counter styles:

``` text

  @counter-style lower-alpha {
    system: alphabetic;
    symbols: a b c d e f g h i j k l m n o p q r s t u v w x y z;
  }

  @counter-style lower-latin {
    system: extends lower-alpha;
  }

  @counter-style upper-alpha {
    system: alphabetic;
    symbols: A B C D E F G H I J K L M N O P Q R S T U V W X Y Z;
  }

  @counter-style upper-latin {
    system: extends upper-alpha;
  }

  @counter-style lower-greek {
    system: alphabetic;
    symbols: "\3B1" "\3B2" "\3B3" "\3B4" "\3B5" "\3B6" "\3B7" "\3B8" "\3B9" "\3BA" "\3BB" "\3BC" "\3BD" "\3BE" "\3BF" "\3C0" "\3C1" "\3C3" "\3C4" "\3C5" "\3C6" "\3C7" "\3C8" "\3C9";
    /* α β γ δ ε ζ η θ ι κ λ μ ν ξ ο π ρ σ τ υ φ χ ψ ω */
  }

  @counter-style hiragana {
    system: alphabetic;
    symbols: "\3042" "\3044" "\3046" "\3048" "\304A" "\304B" "\304D" "\304F" "\3051" "\3053" "\3055" "\3057" "\3059" "\305B" "\305D" "\305F" "\3061" "\3064" "\3066" "\3068" "\306A" "\306B" "\306C" "\306D" "\306E" "\306F" "\3072" "\3075" "\3078" "\307B" "\307E" "\307F" "\3080" "\3081" "\3082" "\3084" "\3086" "\3088" "\3089" "\308A" "\308B" "\308C" "\308D" "\308F" "\3090" "\3091" "\3092" "\3093";
    /* あ い う え お か き く け こ さ し す せ そ た ち つ て と な に ぬ ね の は ひ ふ へ ほ ま み む め も や ゆ よ ら り る れ ろ わ ゐ ゑ を ん */
    suffix: "、";
  }

  @counter-style hiragana-iroha {
    system: alphabetic;
    symbols: "\3044" "\308D" "\306F" "\306B" "\307B" "\3078" "\3068" "\3061" "\308A" "\306C" "\308B" "\3092" "\308F" "\304B" "\3088" "\305F" "\308C" "\305D" "\3064" "\306D" "\306A" "\3089" "\3080" "\3046" "\3090" "\306E" "\304A" "\304F" "\3084" "\307E" "\3051" "\3075" "\3053" "\3048" "\3066" "\3042" "\3055" "\304D" "\3086" "\3081" "\307F" "\3057" "\3091" "\3072" "\3082" "\305B" "\3059";
    /* い ろ は に ほ へ と ち り ぬ る を わ か よ た れ そ つ ね な ら む う ゐ の お く や ま け ふ こ え て あ さ き ゆ め み し ゑ ひ も せ す */
    suffix: "、";
  }

  @counter-style katakana {
    system: alphabetic;
    symbols: "\30A2" "\30A4" "\30A6" "\30A8" "\30AA" "\30AB" "\30AD" "\30AF" "\30B1" "\30B3" "\30B5" "\30B7" "\30B9" "\30BB" "\30BD" "\30BF" "\30C1" "\30C4" "\30C6" "\30C8" "\30CA" "\30CB" "\30CC" "\30CD" "\30CE" "\30CF" "\30D2" "\30D5" "\30D8" "\30DB" "\30DE" "\30DF" "\30E0" "\30E1" "\30E2" "\30E4" "\30E6" "\30E8" "\30E9" "\30EA" "\30EB" "\30EC" "\30ED" "\30EF" "\30F0" "\30F1" "\30F2" "\30F3";
    /* ア イ ウ エ オ カ キ ク ケ コ サ シ ス セ ソ タ チ ツ テ ト ナ ニ ヌ ネ ノ ハ ヒ フ ヘ ホ マ ミ ム メ モ ヤ ユ ヨ ラ リ ル レ ロ ワ ヰ ヱ ヲ ン */
    suffix: "、";
  }

  @counter-style katakana-iroha {
    system: alphabetic;
    symbols: "\30A4" "\30ED" "\30CF" "\30CB" "\30DB" "\30D8" "\30C8" "\30C1" "\30EA" "\30CC" "\30EB" "\30F2" "\30EF" "\30AB" "\30E8" "\30BF" "\30EC" "\30BD" "\30C4" "\30CD" "\30CA" "\30E9" "\30E0" "\30A6" "\30F0" "\30CE" "\30AA" "\30AF" "\30E4" "\30DE" "\30B1" "\30D5" "\30B3" "\30A8" "\30C6" "\30A2" "\30B5" "\30AD" "\30E6" "\30E1" "\30DF" "\30B7" "\30F1" "\30D2" "\30E2" "\30BB" "\30B9";
    /* イ ロ ハ ニ ホ ヘ ト チ リ ヌ ル ヲ ワ カ ヨ タ レ ソ ツ ネ ナ ラ ム ウ ヰ ノ オ ク ヤ マ ケ フ コ エ テ ア サ キ ユ メ ミ シ ヱ ヒ モ セ ス */
    suffix: "、";
  }
  
```

### <a id="simple-symbolic"></a>6.3.  Symbolic: <a id="ref-for-disc"></a>[disc](#disc), <a id="ref-for-circle"></a>[circle](#circle), <a id="ref-for-square"></a>[square](#square), <a id="ref-for-disclosure-open"></a>[disclosure-open](#disclosure-open), <a id="ref-for-disclosure-closed"></a>[disclosure-closed](#disclosure-closed)[](#simple-symbolic)

The <a id="predefined-symbolic-counter-style"></a><strong>predefined symbolic counter styles</strong> each represent a single symbol, and are defined as follows:

<strong><a id="disc"></a><strong>disc</strong></strong>

A filled circle, similar to • U+2022 BULLET.

<strong><a id="circle"></a><strong>circle</strong></strong>

A hollow circle, similar to ◦ U+25E6 WHITE BULLET.

<strong><a id="square"></a><strong>square</strong></strong>

A filled square, similar to ▪ U+25AA BLACK SMALL SQUARE.

<strong><a id="disclosure-open"></a><strong>disclosure-open</strong></strong>

<strong><a id="disclosure-closed"></a><strong>disclosure-closed</strong></strong>

Symbols appropriate for indicating an open or closed disclosure widget, such as the HTML <code><a id="ref-for-the-details-element"></a>[details](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element)</code> element.

Tests

- [disclosure-styles.html](https://wpt.fyi/results/css/css-counter-styles/counter-style-at-rule/disclosure-styles.html) [(live test)](http://wpt.live/css/css-counter-styles/counter-style-at-rule/disclosure-styles.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/counter-style-at-rule/disclosure-styles.html)

They can be referred to syntactically using the <a id="ref-for-typedef-symbolic-glyph②"></a>[\<symbolic-glyph\>](#typedef-symbolic-glyph) production:

<a id="typedef-symbolic-glyph"></a><a id="ref-for-typedef-symbolic-glyph③"></a><a id="ref-for-comb-one②①"></a><a id="ref-for-comb-one②②"></a><a id="ref-for-comb-one②③"></a><a id="ref-for-comb-one②④"></a>

``` text
<symbolic-glyph> = disc | circle | square | disclosure-open | disclosure-closed
```

The following stylesheet fragment provides the normative definition of these predefined counter styles:

``` text
@counter-style disc {
  system: cyclic;
  symbols: \2022;
  /* • */
  suffix: " ";
}

@counter-style circle {
  system: cyclic;
  symbols: \25E6;
  /* ◦ */
  suffix: " ";
}

@counter-style square {
  system: cyclic;
  symbols: \25AA;
  /* ▪ */
  suffix: " ";
}

@counter-style disclosure-open {
  system: cyclic;
  suffix: " ";
  /* for symbols, see normative text below */
}

@counter-style disclosure-closed {
  system: cyclic;
  suffix: " ";
  /* for symbols, see normative text below */
}
```

For the <a id="ref-for-disclosure-open①"></a>[disclosure-open](#disclosure-open) and <a id="ref-for-disclosure-closed①"></a>[disclosure-closed](#disclosure-closed) counter styles, the marker must be an image or character suitable for indicating the open and closed states of a disclosure widget, such as HTML’s <code><a id="ref-for-the-details-element①"></a>[details](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element)</code> element. If the image is directional, it must respond to the <a id="ref-for-writing-mode"></a>[writing mode](https://drafts.csswg.org/css-writing-modes-4/#writing-mode) of the element, similar to the [bidi-sensitive images](https://drafts.csswg.org/css4-images/#bidi-images) feature of the Images 4 module. For example, in a <a id="ref-for-horizontal-writing-mode"></a>[horizontal writing mode](https://drafts.csswg.org/css-writing-modes-4/#horizontal-writing-mode) the <a id="ref-for-disclosure-closed②"></a>disclosure-closed style might use the characters U+25B8 BLACK RIGHT-POINTING SMALL TRIANGLE (▸) for <a id="ref-for-valdef-direction-ltr"></a>[ltr](https://drafts.csswg.org/css-writing-modes-4/#valdef-direction-ltr) elements and U+25C2 BLACK LEFT-POINTING SMALL TRIANGLE (◂) for <a id="ref-for-valdef-direction-rtl"></a>[rtl](https://drafts.csswg.org/css-writing-modes-4/#valdef-direction-rtl) elements, while the <a id="ref-for-disclosure-open②"></a>disclosure-open style might use the character U+25BE BLACK DOWN-POINTING SMALL TRIANGLE (▾).

A UA may render the <a id="ref-for-typedef-symbolic-glyph④"></a>[\<symbolic-glyph\>](#typedef-symbolic-glyph) styles using a UA-generated image or a UA-chosen font instead of rendering the specified character in the element’s own font. It must look similar to the character, and must be sized to attractively fill a 1em by 1em square. A UA that does so may also fall back to its string definition when rendering these values in a <a id="ref-for-typedef-counter"></a>[\<counter\>](https://drafts.csswg.org/css-lists-3/#typedef-counter) value.

### <a id="simple-fixed"></a>6.4.  Fixed: <a id="ref-for-valdef-counter-style-name-cjk-earthly-branch"></a>[cjk-earthly-branch](#valdef-counter-style-name-cjk-earthly-branch), <a id="ref-for-valdef-counter-style-name-cjk-heavenly-stem"></a>[cjk-heavenly-stem](#valdef-counter-style-name-cjk-heavenly-stem)[](#simple-fixed)

<strong><a id="valdef-counter-style-name-cjk-earthly-branch"></a><strong>cjk-earthly-branch</strong></strong>

Han "Earthly Branch" ordinals (e.g., 子, 丑, 寅, ..., 亥).

Tests

- [css3-counter-styles-201.html](https://wpt.fyi/results/css/css-counter-styles/cjk-earthly-branch/css3-counter-styles-201.html) [(live test)](http://wpt.live/css/css-counter-styles/cjk-earthly-branch/css3-counter-styles-201.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cjk-earthly-branch/css3-counter-styles-201.html)
- [css3-counter-styles-202.html](https://wpt.fyi/results/css/css-counter-styles/cjk-earthly-branch/css3-counter-styles-202.html) [(live test)](http://wpt.live/css/css-counter-styles/cjk-earthly-branch/css3-counter-styles-202.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cjk-earthly-branch/css3-counter-styles-202.html)
- [css3-counter-styles-203.html](https://wpt.fyi/results/css/css-counter-styles/cjk-earthly-branch/css3-counter-styles-203.html) [(live test)](http://wpt.live/css/css-counter-styles/cjk-earthly-branch/css3-counter-styles-203.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cjk-earthly-branch/css3-counter-styles-203.html)

<strong><a id="valdef-counter-style-name-cjk-heavenly-stem"></a><strong>cjk-heavenly-stem</strong></strong>

Han "Heavenly Stem" ordinals (e.g., 甲, 乙, 丙, ..., 癸)

Tests

- [css3-counter-styles-204.html](https://wpt.fyi/results/css/css-counter-styles/cjk-heavenly-stem/css3-counter-styles-204.html) [(live test)](http://wpt.live/css/css-counter-styles/cjk-heavenly-stem/css3-counter-styles-204.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cjk-heavenly-stem/css3-counter-styles-204.html)
- [css3-counter-styles-205.html](https://wpt.fyi/results/css/css-counter-styles/cjk-heavenly-stem/css3-counter-styles-205.html) [(live test)](http://wpt.live/css/css-counter-styles/cjk-heavenly-stem/css3-counter-styles-205.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cjk-heavenly-stem/css3-counter-styles-205.html)
- [css3-counter-styles-206.html](https://wpt.fyi/results/css/css-counter-styles/cjk-heavenly-stem/css3-counter-styles-206.html) [(live test)](http://wpt.live/css/css-counter-styles/cjk-heavenly-stem/css3-counter-styles-206.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cjk-heavenly-stem/css3-counter-styles-206.html)

The following stylesheet fragment provides the normative definition of these predefined counter styles:

``` text

  @counter-style cjk-earthly-branch {
    system: fixed;
    symbols: "\5B50" "\4E11" "\5BC5" "\536F" "\8FB0" "\5DF3" "\5348" "\672A" "\7533" "\9149" "\620C" "\4EA5";
    /* 子 丑 寅 卯 辰 巳 午 未 申 酉 戌 亥 */
    suffix: "、";
    fallback: cjk-decimal;
  }

  @counter-style cjk-heavenly-stem {
    system: fixed;
    symbols: "\7532" "\4E59" "\4E19" "\4E01" "\620A" "\5DF1" "\5E9A" "\8F9B" "\58EC" "\7678";
    /* 甲 乙 丙 丁 戊 己 庚 辛 壬 癸 */
    suffix: "、";
    fallback: cjk-decimal;
  }
  
```

## <a id="complex-predefined-counters"></a>7.  Complex Predefined Counter Styles[](#complex-predefined-counters)

While authors may define their own counter styles using the <a id="ref-for-at-ruledef-counter-style③⑧"></a>[&#64;counter-style](#at-ruledef-counter-style) rule or rely on the set of predefined counter styles, a few counter styles are described by rules that are too complex to be captured by the predefined algorithms. These counter styles are described in this section.

Some of the counter styles specified in this section have custom algorithms for generating counter values, but are otherwise identical to a counter style defined via the <a id="ref-for-at-ruledef-counter-style③⑨"></a>[&#64;counter-style](#at-ruledef-counter-style) rule. For example, an author can reference one of these styles in an <a id="ref-for-valdef-counter-style-system-extends⑦"></a>[extends](#valdef-counter-style-system-extends) system, reusing the algorithm but swapping out some of the other descriptors.

All of the counter styles defined in this section have a <a id="ref-for-descdef-counter-style-speak-as⑥"></a>[spoken form](#descdef-counter-style-speak-as) of <a id="ref-for-valdef-counter-style-speak-as-numbers③"></a>[numbers](#valdef-counter-style-speak-as-numbers), and <a id="ref-for-use-a-negative-sign⑤"></a>[use a negative sign](#use-a-negative-sign).

### <a id="complex-cjk"></a>7.1.  Longhand East Asian Counter Styles[](#complex-cjk)

Chinese, Japanese, and Korean have counter styles which have a “longhand” nature, similar to “thirteen thousand one hundred and twenty-three” in English. Each has both formal and informal variants. The formal styles are typically used in financial and legal documents, as their characters are more difficult to alter into each other.

<a id="example-c1039050"></a>

<strong>Example:</strong>

[](#example-c1039050) The following table shows examples of these styles, particularly some ways in which they differ.

| Counter Style                                                                                              | 0   | 1   | 2   | 3   | 10   | 11     | 99     | 100  | 101      | 6001     |
|------------------------------------------------------------------------------------------------------------|-----|-----|-----|-----|------|--------|--------|------|----------|----------|
| <strong><a id="ref-for-japanese-informal"></a>[japanese-informal](#japanese-informal)</strong>             | 〇  | 一  | 二  | 三  | 十   | 十一   | 九十九 | 百   | 百一     | 六千一   |
| <strong><a id="ref-for-japanese-formal"></a>[japanese-formal](#japanese-formal)</strong>                   | 零  | 壱  | 弐  | 参  | 壱拾 | 壱拾壱 | 九拾九 | 壱百 | 壱百壱   | 六阡壱   |
| <strong><a id="ref-for-korean-hangul-formal"></a>[korean-hangul-formal](#korean-hangul-formal)</strong>    | 영  | 일  | 이  | 삼  | 일십 | 일십일 | 구십구 | 일백 | 일백일   | 육천일   |
| <strong><a id="ref-for-korean-hanja-informal"></a>[korean-hanja-informal](#korean-hanja-informal)</strong> | 零  | 一  | 二  | 三  | 十   | 十一   | 九十九 | 百   | 百一     | 六千一   |
| <strong><a id="ref-for-korean-hanja-formal"></a>[korean-hanja-formal](#korean-hanja-formal)</strong>       | 零  | 壹  | 貳  | 參  | 壹拾 | 壹拾壹 | 九拾九 | 壹百 | 壹百壹   | 六仟壹   |
| <strong><a id="ref-for-simp-chinese-informal"></a>[simp-chinese-informal](#simp-chinese-informal)</strong> | 零  | 一  | 二  | 三  | 十   | 十一   | 九十九 | 一百 | 一百零一 | 六千零一 |
| <strong><a id="ref-for-simp-chinese-formal"></a>[simp-chinese-formal](#simp-chinese-formal)</strong>       | 零  | 壹  | 贰  | 叁  | 壹拾 | 壹拾壹 | 玖拾玖 | 壹佰 | 壹佰零壹 | 陆仟零壹 |
| <strong><a id="ref-for-trad-chinese-informal"></a>[trad-chinese-informal](#trad-chinese-informal)</strong> | 零  | 一  | 二  | 三  | 十   | 十一   | 九十九 | 一百 | 一百零一 | 六千零一 |
| <strong><a id="ref-for-trad-chinese-formal"></a>[trad-chinese-formal](#trad-chinese-formal)</strong>       | 零  | 壹  | 貳  | 參  | 壹拾 | 壹拾壹 | 玖拾玖 | 壹佰 | 壹佰零壹 | 陸仟零壹 |

Because opinions differ on how best to represent numbers 10,000 or greater using the longhand CJK styles, the implementation details are split into two sections. The Korean and Japanese variants of these counter styles can, if limited to the range of -9999 to 9999, be expressed as <a id="ref-for-at-ruledef-counter-style④⓪"></a>[&#64;counter-style](#at-ruledef-counter-style) rules. The Chinese variants use a [specialized algorithm](#limited-chinese), regardless of range. The required implementation is described in [§ 7.1.1 Limited-range Implementation (required)](#limited-range-required), which defines the styles over these limited ranges, and the optional implementation is described in [§ 7.1.2 Extended Implementation (optional)](#extended-range-optional), which defines them over a larger range using custom algorithms. Outside the implementation-supported range, the fallback is <a id="ref-for-cjk-decimal①"></a>[cjk-decimal](#cjk-decimal).

#### <a id="limited-range-required"></a>7.1.1. Limited-range Implementation (required)[](#limited-range-required)

##### <a id="limited-japanese"></a>7.1.1.1.  Japanese: <a id="ref-for-japanese-informal①"></a>[japanese-informal](#japanese-informal) and <a id="ref-for-japanese-formal①"></a>[japanese-formal](#japanese-formal)[](#limited-japanese)

<strong><a id="japanese-informal"></a><strong>japanese-informal</strong></strong>

Informal Japanese Kanji numbering (e.g., 千百十一)

Tests

- [counter-japanese-informal-extended.html](https://wpt.fyi/results/css/css-counter-styles/japanese-informal/counter-japanese-informal-extended.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-informal/counter-japanese-informal-extended.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-informal/counter-japanese-informal-extended.html)
- [counter-japanese-informal.html](https://wpt.fyi/results/css/css-counter-styles/japanese-informal/counter-japanese-informal.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-informal/counter-japanese-informal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-informal/counter-japanese-informal.html)
- [css3-counter-styles-042.html](https://wpt.fyi/results/css/css-counter-styles/japanese-informal/css3-counter-styles-042.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-informal/css3-counter-styles-042.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-informal/css3-counter-styles-042.html)
- [css3-counter-styles-043.html](https://wpt.fyi/results/css/css-counter-styles/japanese-informal/css3-counter-styles-043.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-informal/css3-counter-styles-043.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-informal/css3-counter-styles-043.html)
- [css3-counter-styles-044.html](https://wpt.fyi/results/css/css-counter-styles/japanese-informal/css3-counter-styles-044.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-informal/css3-counter-styles-044.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-informal/css3-counter-styles-044.html)
- [css3-counter-styles-045.html](https://wpt.fyi/results/css/css-counter-styles/japanese-informal/css3-counter-styles-045.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-informal/css3-counter-styles-045.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-informal/css3-counter-styles-045.html)
- [css3-counter-styles-046.html](https://wpt.fyi/results/css/css-counter-styles/japanese-informal/css3-counter-styles-046.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-informal/css3-counter-styles-046.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-informal/css3-counter-styles-046.html)

<strong><a id="japanese-formal"></a><strong>japanese-formal</strong></strong>

Formal Japanese Kanji numbering (e.g. 壱阡壱百壱拾壱)

Tests

- [counter-japanese-formal-extended.html](https://wpt.fyi/results/css/css-counter-styles/japanese-formal/counter-japanese-formal-extended.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-formal/counter-japanese-formal-extended.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-formal/counter-japanese-formal-extended.html)
- [counter-japanese-formal.html](https://wpt.fyi/results/css/css-counter-styles/japanese-formal/counter-japanese-formal.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-formal/counter-japanese-formal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-formal/counter-japanese-formal.html)
- [css3-counter-styles-047.html](https://wpt.fyi/results/css/css-counter-styles/japanese-formal/css3-counter-styles-047.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-formal/css3-counter-styles-047.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-formal/css3-counter-styles-047.html)
- [css3-counter-styles-048.html](https://wpt.fyi/results/css/css-counter-styles/japanese-formal/css3-counter-styles-048.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-formal/css3-counter-styles-048.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-formal/css3-counter-styles-048.html)
- [css3-counter-styles-049.html](https://wpt.fyi/results/css/css-counter-styles/japanese-formal/css3-counter-styles-049.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-formal/css3-counter-styles-049.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-formal/css3-counter-styles-049.html)
- [css3-counter-styles-050.html](https://wpt.fyi/results/css/css-counter-styles/japanese-formal/css3-counter-styles-050.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-formal/css3-counter-styles-050.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-formal/css3-counter-styles-050.html)
- [css3-counter-styles-051.html](https://wpt.fyi/results/css/css-counter-styles/japanese-formal/css3-counter-styles-051.html) [(live test)](http://wpt.live/css/css-counter-styles/japanese-formal/css3-counter-styles-051.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/japanese-formal/css3-counter-styles-051.html)

``` text
@counter-style japanese-informal {
  system: additive;
  range: -9999 9999;
  additive-symbols: 9000 \4E5D\5343, 8000 \516B\5343, 7000 \4E03\5343, 6000 \516D\5343, 5000 \4E94\5343, 4000 \56DB\5343, 3000 \4E09\5343, 2000 \4E8C\5343, 1000 \5343, 900 \4E5D\767E, 800 \516B\767E, 700 \4E03\767E, 600 \516D\767E, 500 \4E94\767E, 400 \56DB\767E, 300 \4E09\767E, 200 \4E8C\767E, 100 \767E, 90 \4E5D\5341, 80 \516B\5341, 70 \4E03\5341, 60 \516D\5341, 50 \4E94\5341, 40 \56DB\5341, 30 \4E09\5341, 20 \4E8C\5341, 10 \5341, 9 \4E5D, 8 \516B, 7 \4E03, 6 \516D, 5 \4E94, 4 \56DB, 3 \4E09, 2 \4E8C, 1 \4E00, 0 \3007;
  /* 9000 九千, 8000 八千, 7000 七千, 6000 六千, 5000 五千, 4000 四千, 3000 三千, 2000 二千, 1000 千, 900 九百, 800 八百, 700 七百, 600 六百, 500 五百, 400 四百, 300 三百, 200 二百, 100 百, 90 九十, 80 八十, 70 七十, 60 六十, 50 五十, 40 四十, 30 三十, 20 二十, 10 十, 9 九, 8 八, 7 七, 6 六, 5 五, 4 四, 3 三, 2 二, 1 一, 0 〇 */
  suffix: '\3001';
  /* 、 */
  negative: "\30DE\30A4\30CA\30B9";
  /* マイナス */
  fallback: cjk-decimal;
}

@counter-style japanese-formal {
  system: additive;
  range: -9999 9999;
  additive-symbols: 9000 \4E5D\9621, 8000 \516B\9621, 7000 \4E03\9621, 6000 \516D\9621, 5000 \4F0D\9621, 4000 \56DB\9621, 3000 \53C2\9621, 2000 \5F10\9621, 1000 \58F1\9621, 900 \4E5D\767E, 800 \516B\767E, 700 \4E03\767E, 600 \516D\767E, 500 \4F0D\767E, 400 \56DB\767E, 300 \53C2\767E, 200 \5F10\767E, 100 \58F1\767E, 90 \4E5D\62FE, 80 \516B\62FE, 70 \4E03\62FE, 60 \516D\62FE, 50 \4F0D\62FE, 40 \56DB\62FE, 30 \53C2\62FE, 20 \5F10\62FE, 10 \58F1\62FE, 9 \4E5D, 8 \516B, 7 \4E03, 6 \516D, 5 \4F0D, 4 \56DB, 3 \53C2, 2 \5F10, 1 \58F1, 0 \96F6;
  /* 9000 九阡, 8000 八阡, 7000 七阡, 6000 六阡, 5000 伍阡, 4000 四阡, 3000 参阡, 2000 弐阡, 1000 壱阡, 900 九百, 800 八百, 700 七百, 600 六百, 500 伍百, 400 四百, 300 参百, 200 弐百, 100 壱百, 90 九拾, 80 八拾, 70 七拾, 60 六拾, 50 伍拾, 40 四拾, 30 参拾, 20 弐拾, 10 壱拾, 9 九, 8 八, 7 七, 6 六, 5 伍, 4 四, 3 参, 2 弐, 1 壱, 0 零 */
  suffix: '\3001';
  /* 、 */
  negative: "\30DE\30A4\30CA\30B9";
  /* マイナス */
  fallback: cjk-decimal;
}
```

##### <a id="limited-korean"></a>7.1.1.2.  Korean: <a id="ref-for-korean-hangul-formal①"></a>[korean-hangul-formal](#korean-hangul-formal), <a id="ref-for-korean-hanja-informal①"></a>[korean-hanja-informal](#korean-hanja-informal), and <a id="ref-for-korean-hanja-formal①"></a>[korean-hanja-formal](#korean-hanja-formal)[](#limited-korean)

<strong><a id="korean-hangul-formal"></a><strong>korean-hangul-formal</strong></strong>

Korean Hangul numbering (e.g., 일천일백일십일)

Tests

- [counter-korean-hangul-formal-extended.html](https://wpt.fyi/results/css/css-counter-styles/korean-hangul-formal/counter-korean-hangul-formal-extended.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hangul-formal/counter-korean-hangul-formal-extended.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hangul-formal/counter-korean-hangul-formal-extended.html)
- [counter-korean-hangul-formal.html](https://wpt.fyi/results/css/css-counter-styles/korean-hangul-formal/counter-korean-hangul-formal.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hangul-formal/counter-korean-hangul-formal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hangul-formal/counter-korean-hangul-formal.html)
- [css3-counter-styles-052.html](https://wpt.fyi/results/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-052.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-052.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-052.html)
- [css3-counter-styles-053.html](https://wpt.fyi/results/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-053.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-053.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-053.html)
- [css3-counter-styles-054.html](https://wpt.fyi/results/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-054.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-054.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-054.html)
- [css3-counter-styles-055.html](https://wpt.fyi/results/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-055.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-055.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-055.html)
- [css3-counter-styles-056.html](https://wpt.fyi/results/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-056.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-056.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hangul-formal/css3-counter-styles-056.html)

<strong><a id="korean-hanja-informal"></a><strong>korean-hanja-informal</strong></strong>

Informal Korean Hanja numbering (e.g., 千百十一)

Tests

- [counter-korean-hanja-informal-extended.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-informal/counter-korean-hanja-informal-extended.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-informal/counter-korean-hanja-informal-extended.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-informal/counter-korean-hanja-informal-extended.html)
- [counter-korean-hanja-informal.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-informal/counter-korean-hanja-informal.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-informal/counter-korean-hanja-informal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-informal/counter-korean-hanja-informal.html)
- [css3-counter-styles-057.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-057.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-057.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-057.html)
- [css3-counter-styles-058.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-058.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-058.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-058.html)
- [css3-counter-styles-059.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-059.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-059.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-059.html)
- [css3-counter-styles-060.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-060.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-060.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-060.html)
- [css3-counter-styles-061.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-061.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-061.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-informal/css3-counter-styles-061.html)

<strong><a id="korean-hanja-formal"></a><strong>korean-hanja-formal</strong></strong>

Formal Korean Han (Hanja) numbering (e.g., 壹仟壹百壹拾壹)

Tests

- [counter-korean-hanja-formal-extended.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-formal/counter-korean-hanja-formal-extended.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-formal/counter-korean-hanja-formal-extended.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-formal/counter-korean-hanja-formal-extended.html)
- [counter-korean-hanja-formal.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-formal/counter-korean-hanja-formal.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-formal/counter-korean-hanja-formal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-formal/counter-korean-hanja-formal.html)
- [css3-counter-styles-062.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-062.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-062.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-062.html)
- [css3-counter-styles-063.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-063.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-063.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-063.html)
- [css3-counter-styles-064.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-064.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-064.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-064.html)
- [css3-counter-styles-065.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-065.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-065.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-065.html)
- [css3-counter-styles-066.html](https://wpt.fyi/results/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-066.html) [(live test)](http://wpt.live/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-066.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/korean-hanja-formal/css3-counter-styles-066.html)

``` text
@counter-style korean-hangul-formal {
  system: additive;
  range: -9999 9999;
  additive-symbols: 9000 \AD6C\CC9C, 8000 \D314\CC9C, 7000 \CE60\CC9C, 6000 \C721\CC9C, 5000 \C624\CC9C, 4000 \C0AC\CC9C, 3000 \C0BC\CC9C, 2000 \C774\CC9C, 1000 \C77C\CC9C, 900 \AD6C\BC31, 800 \D314\BC31, 700 \CE60\BC31, 600 \C721\BC31, 500 \C624\BC31, 400 \C0AC\BC31, 300 \C0BC\BC31, 200 \C774\BC31, 100 \C77C\BC31, 90 \AD6C\C2ED, 80 \D314\C2ED, 70 \CE60\C2ED, 60 \C721\C2ED, 50 \C624\C2ED, 40 \C0AC\C2ED, 30 \C0BC\C2ED, 20 \C774\C2ED, 10 \C77C\C2ED, 9 \AD6C, 8 \D314, 7 \CE60, 6 \C721, 5 \C624, 4 \C0AC, 3 \C0BC, 2 \C774, 1 \C77C, 0 \C601;
  /* 9000 구천, 8000 팔천, 7000 칠천, 6000 육천, 5000 오천, 4000 사천, 3000 삼천, 2000 이천, 1000 일천, 900 구백, 800 팔백, 700 칠백, 600 육백, 500 오백, 400 사백, 300 삼백, 200 이백, 100 일백, 90 구십, 80 팔십, 70 칠십, 60 육십, 50 오십, 40 사십, 30 삼십, 20 이십, 10 일십, 9 구, 8 팔, 7 칠, 6 육, 5 오, 4 사, 3 삼, 2 이, 1 일, 0 영 */
  suffix: ', ';
  negative: "\B9C8\C774\B108\C2A4  ";
  /* 마이너스 (followed by a space) */
  fallback: cjk-decimal;
}

@counter-style korean-hanja-informal {
  system: additive;
  range: -9999 9999;
  additive-symbols: 9000 \4E5D\5343, 8000 \516B\5343, 7000 \4E03\5343, 6000 \516D\5343, 5000 \4E94\5343, 4000 \56DB\5343, 3000 \4E09\5343, 2000 \4E8C\5343, 1000 \5343, 900 \4E5D\767E, 800 \516B\767E, 700 \4E03\767E, 600 \516D\767E, 500 \4E94\767E, 400 \56DB\767E, 300 \4E09\767E, 200 \4E8C\767E, 100 \767E, 90 \4E5D\5341, 80 \516B\5341, 70 \4E03\5341, 60 \516D\5341, 50 \4E94\5341, 40 \56DB\5341, 30 \4E09\5341, 20 \4E8C\5341, 10 \5341, 9 \4E5D, 8 \516B, 7 \4E03, 6 \516D, 5 \4E94, 4 \56DB, 3 \4E09, 2 \4E8C, 1 \4E00, 0 \96F6;
  /* 9000 九千, 8000 八千, 7000 七千, 6000 六千, 5000 五千, 4000 四千, 3000 三千, 2000 二千, 1000 千, 900 九百, 800 八百, 700 七百, 600 六百, 500 五百, 400 四百, 300 三百, 200 二百, 100 百, 90 九十, 80 八十, 70 七十, 60 六十, 50 五十, 40 四十, 30 三十, 20 二十, 10 十, 9 九, 8 八, 7 七, 6 六, 5 五, 4 四, 3 三, 2 二, 1 一, 0 零 */
  suffix: ', ';
  negative: "\B9C8\C774\B108\C2A4  ";
  /* 마이너스 (followed by a space) */
  fallback: cjk-decimal;
}

@counter-style korean-hanja-formal {
  system: additive;
  range: -9999 9999;
  additive-symbols: 9000 \4E5D\4EDF, 8000 \516B\4EDF, 7000 \4E03\4EDF, 6000 \516D\4EDF, 5000 \4E94\4EDF, 4000 \56DB\4EDF, 3000 \53C3\4EDF, 2000 \8CB3\4EDF, 1000 \58F9\4EDF, 900 \4E5D\767E, 800 \516B\767E, 700 \4E03\767E, 600 \516D\767E, 500 \4E94\767E, 400 \56DB\767E, 300 \53C3\767E, 200 \8CB3\767E, 100 \58F9\767E, 90 \4E5D\62FE, 80 \516B\62FE, 70 \4E03\62FE, 60 \516D\62FE, 50 \4E94\62FE, 40 \56DB\62FE, 30 \53C3\62FE, 20 \8CB3\62FE, 10 \58F9\62FE, 9 \4E5D, 8 \516B, 7 \4E03, 6 \516D, 5 \4E94, 4 \56DB, 3 \53C3, 2 \8CB3, 1 \58F9, 0 \96F6;
  /* 9000 九仟, 8000 八仟, 7000 七仟, 6000 六仟, 5000 五仟, 4000 四仟, 3000 參仟, 2000 貳仟, 1000 壹仟, 900 九百, 800 八百, 700 七百, 600 六百, 500 五百, 400 四百, 300 參百, 200 貳百, 100 壹百, 90 九拾, 80 八拾, 70 七拾, 60 六拾, 50 五拾, 40 四拾, 30 參拾, 20 貳拾, 10 壹拾, 9 九, 8 八, 7 七, 6 六, 5 五, 4 四, 3 參, 2 貳, 1 壹, 0 零 */
  suffix: ', ';
  negative: "\B9C8\C774\B108\C2A4  ";
  /* 마이너스 (followed by a space) */
  fallback: cjk-decimal;
}
```

##### <a id="limited-chinese"></a>7.1.1.3.  Chinese: <a id="ref-for-simp-chinese-informal①"></a>[simp-chinese-informal](#simp-chinese-informal), <a id="ref-for-simp-chinese-formal①"></a>[simp-chinese-formal](#simp-chinese-formal), <a id="ref-for-trad-chinese-informal①"></a>[trad-chinese-informal](#trad-chinese-informal), and <a id="ref-for-trad-chinese-formal①"></a>[trad-chinese-formal](#trad-chinese-formal)[](#limited-chinese)

<strong><a id="simp-chinese-informal"></a><strong>simp-chinese-informal</strong></strong>

Simplified Chinese informal numbering (e.g., 一千一百一十一)

Tests

- [counter-simp-chinese-informal.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-informal/counter-simp-chinese-informal.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-informal/counter-simp-chinese-informal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-informal/counter-simp-chinese-informal.html)
- [css3-counter-styles-071.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-071.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-071.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-071.html)
- [css3-counter-styles-072.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-072.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-072.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-072.html)
- [css3-counter-styles-073.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-073.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-073.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-073.html)
- [css3-counter-styles-074.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-074.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-074.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-074.html)
- [css3-counter-styles-075.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-075.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-075.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-informal/css3-counter-styles-075.html)

<strong><a id="simp-chinese-formal"></a><strong>simp-chinese-formal</strong></strong>

Simplified Chinese formal numbering (e.g. 壹仟壹佰壹拾壹)

Tests

- [counter-simp-chinese-formal.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-formal/counter-simp-chinese-formal.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-formal/counter-simp-chinese-formal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-formal/counter-simp-chinese-formal.html)
- [css3-counter-styles-076.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-076.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-076.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-076.html)
- [css3-counter-styles-077.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-077.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-077.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-077.html)
- [css3-counter-styles-078.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-078.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-078.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-078.html)
- [css3-counter-styles-079.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-079.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-079.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-079.html)
- [css3-counter-styles-080.html](https://wpt.fyi/results/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-080.html) [(live test)](http://wpt.live/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-080.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/simp-chinese-formal/css3-counter-styles-080.html)

<strong><a id="trad-chinese-informal"></a><strong>trad-chinese-informal</strong></strong>

Traditional Chinese informal numbering (e.g., 一千一百一十一)

Tests

- [counter-trad-chinese-informal.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-informal/counter-trad-chinese-informal.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-informal/counter-trad-chinese-informal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-informal/counter-trad-chinese-informal.html)
- [css3-counter-styles-081.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-081.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-081.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-081.html)
- [css3-counter-styles-082.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-082.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-082.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-082.html)
- [css3-counter-styles-083.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-083.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-083.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-083.html)
- [css3-counter-styles-084.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-084.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-084.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-084.html)
- [css3-counter-styles-085.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-085.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-085.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-informal/css3-counter-styles-085.html)

<strong><a id="trad-chinese-formal"></a><strong>trad-chinese-formal</strong></strong>

Traditional Chinese formal numbering (e.g., 壹仟壹佰壹拾壹)

Tests

- [counter-trad-chinese-formal.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-formal/counter-trad-chinese-formal.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-formal/counter-trad-chinese-formal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-formal/counter-trad-chinese-formal.html)
- [css3-counter-styles-086.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-086.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-086.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-086.html)
- [css3-counter-styles-087.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-087.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-087.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-087.html)
- [css3-counter-styles-088.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-088.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-088.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-088.html)
- [css3-counter-styles-089.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-089.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-089.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-089.html)
- [css3-counter-styles-090.html](https://wpt.fyi/results/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-090.html) [(live test)](http://wpt.live/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-090.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/trad-chinese-formal/css3-counter-styles-090.html)

<strong><a id="cjk-ideographic"></a><strong>cjk-ideographic</strong></strong>

This counter style is identical to <a id="ref-for-trad-chinese-informal②"></a>[trad-chinese-informal](#trad-chinese-informal). (It exists for legacy reasons.)

The Chinese longhand styles are defined by almost identical algorithms (specified as a single algorithm here, with the differences called out when relevant), but use different sets of characters, as specified by the table following the algorithm.

1.  If the counter value is 0, the representation is the character for 0 specified for the given counter style. Skip the rest of this algorithm.
2.  If the counter value is negative, instead use the absolute value of the counter value for the remaining steps of this algorithm.
3.  Initially represent the counter value as a decimal number. For each digit that is not 0, append the appropriate digit marker to the digit. The ones digit has no marker.
4.  For the informal styles, if the counter value is between ten and nineteen, remove the tens digit (leave the digit marker).
5.  Drop any trailing zeros and collapse any remaining zeros into a single zero digit.
6.  Replace the digits 0-9 with the appropriate character for the given counter style.
7.  If the counter value was negative, prepend the appropriate negative sign character for the given counter style as specified in the table of characters for each style.
8.  Return the resultant string as the representation of the counter value.

For all of these counter styles, the <a id="ref-for-descdef-counter-style-suffix④"></a>[suffix](#descdef-counter-style-suffix) is "、" U+3001, the <a id="ref-for-descdef-counter-style-fallback⑥"></a>[fallback](#descdef-counter-style-fallback) is <a id="ref-for-cjk-decimal②"></a>[cjk-decimal](#cjk-decimal), the <a id="ref-for-descdef-counter-style-range⑤"></a>[range](#descdef-counter-style-range) is -9999 9999, and the <a id="ref-for-descdef-counter-style-negative⑥"></a>[negative](#descdef-counter-style-negative) value is given in the table of symbols for each style.

The following tables define the characters used in these styles:

| Values                                  | Codepoints / simp-chinese-informal   | Codepoints / simp-chinese-formal   | Codepoints / trad-chinese-informal   | Codepoints / trad-chinese-formal   |
|-----------------------------------------|--------------------------------------|------------------------------------|--------------------------------------|------------------------------------|
| <strong>Digit 0</strong>                | 零 U+96F6                            | 零 U+96F6                          | 零 U+96F6                            | 零 U+96F6                          |
| <strong>Digit 1</strong>                | 一 U+4E00                            | 壹 U+58F9                          | 一 U+4E00                            | 壹 U+58F9                          |
| <strong>Digit 2</strong>                | 二 U+4E8C                            | 贰 U+8D30                          | 二 U+4E8C                            | 貳 U+8CB3                          |
| <strong>Digit 3</strong>                | 三 U+4E09                            | 叁 U+53C1                          | 三 U+4E09                            | 參 U+53C3                          |
| <strong>Digit 4</strong>                | 四 U+56DB                            | 肆 U+8086                          | 四 U+56DB                            | 肆 U+8086                          |
| <strong>Digit 5</strong>                | 五 U+4E94                            | 伍 U+4F0D                          | 五 U+4E94                            | 伍 U+4F0D                          |
| <strong>Digit 6</strong>                | 六 U+516D                            | 陆 U+9646                          | 六 U+516D                            | 陸 U+9678                          |
| <strong>Digit 7</strong>                | 七 U+4E03                            | 柒 U+67D2                          | 七 U+4E03                            | 柒 U+67D2                          |
| <strong>Digit 8</strong>                | 八 U+516B                            | 捌 U+634C                          | 八 U+516B                            | 捌 U+634C                          |
| <strong>Digit 9</strong>                | 九 U+4E5D                            | 玖 U+7396                          | 九 U+4E5D                            | 玖 U+7396                          |
| <strong>Tens Digit Marker</strong>      | 十 U+5341                            | 拾 U+62FE                          | 十 U+5341                            | 拾 U+62FE                          |
| <strong>Hundreds Digit Marker</strong>  | 百 U+767E                            | 佰 U+4F70                          | 百 U+767E                            | 佰 U+4F70                          |
| <strong>Thousands Digit Marker</strong> | 千 U+5343                            | 仟 U+4EDF                          | 千 U+5343                            | 仟 U+4EDF                          |
| <strong>Negative Sign</strong>          | 负 U+8D1F                            | 负 U+8D1F                          | 負 U+8CA0                            | 負 U+8CA0                          |

For reference, here are the first 120 values for the

<strong>Note:</strong>

<a id="ref-for-simp-chinese-informal②"></a>[simp-chinese-informal](#simp-chinese-informal) style:

``` text
 1　　　　 一    41　　 四十一    81　　 八十一
 2　　　　 二    42　　 四十二    82　　 八十二
 3　　　　 三    43　　 四十三    83　　 八十三
 4　　　　 四    44　　 四十四    84　　 八十四
 5　　　　 五    45　　 四十五    85　　 八十五
 6　　　　 六    46　　 四十六    86　　 八十六
 7　　　　 七    47　　 四十七    87　　 八十七
 8　　　　 八    48　　 四十八    88　　 八十八
 9　　　　 九    49　　 四十九    89　　 八十九
10　　　　 十    50　　　 五十    90　　　 九十
11　　　 十一    51　　 五十一    91　　 九十一
12　　　 十二    52　　 五十二    92　　 九十二
13　　　 十三    53　　 五十三    93　　 九十三
14　　　 十四    54　　 五十四    94　　 九十四
15　　　 十五    55　　 五十五    95　　 九十五
16　　　 十六    56　　 五十六    96　　 九十六
17　　　 十七    57　　 五十七    97　　 九十七
18　　　 十八    58　　 五十八    98　　 九十八
19　　　 十九    59　　 五十九    99　　 九十九
20　　　 二十    60　　　 六十   100　　　 一百
21　　 二十一    61　　 六十一   101　 一百零一
22　　 二十二    62　　 六十二   102　 一百零二
23　　 二十三    63　　 六十三   103　 一百零三
24　　 二十四    64　　 六十四   104　 一百零四
25　　 二十五    65　　 六十五   105　 一百零五
26　　 二十六    66　　 六十六   106　 一百零六
27　　 二十七    67　　 六十七   107　 一百零七
28　　 二十八    68　　 六十八   108　 一百零八
29　　 二十九    69　　 六十九   109　 一百零九
30　　　 三十    70　　　 七十   110　 一百一十
31　　 三十一    71　　 七十一   111 一百一十一
32　　 三十二    72　　 七十二   112 一百一十二
33　　 三十三    73　　 七十三   113 一百一十三
34　　 三十四    74　　 七十四   114 一百一十四
35　　 三十五    75　　 七十五   115 一百一十五
36　　 三十六    76　　 七十六   116 一百一十六
37　　 三十七    77　　 七十七   117 一百一十七
38　　 三十八    78　　 七十八   118 一百一十八
39　　 三十九    79　　 七十九   119 一百一十九
40　　　 四十    80　　　 八十   120　 一百二十
```

#### <a id="extended-range-optional"></a>7.1.2. Extended Implementation (optional)[](#extended-range-optional)

Some counter styles described in earlier chapters have been limited to an artifically small (though still useful) range to reduce the overall complexity of the spec and the task of implementing those styles. However, some implementations might consider the extra complexity worthwhile for the additional range it offers to authors. To accomodate this, this section describes how to extend the limited counter-styles to a larger range.

This entire section is normative, but <strong>optional</strong>. User-agents may ignore it and still be conformant. If a user-agent implements some of the extended forms described in this section, they must be implemented as described here.

All of the Chinese, Japanese, and Korean styles are defined here for all numbers between -10<sup>16</sup> and 10<sup>16</sup>, exclusive. For numbers outside this range, the <a id="ref-for-cjk-decimal③"></a>[cjk-decimal](#cjk-decimal) style is used. All of the styles are defined by almost identical algorithms (specified as a single algorithm here, with the differences called out when relevant), but use different sets of characters. The list following the algorithm gives the name of each counter style using this algorithm, and the individual character sets used by each style.

1.  If the counter value is 0, the representation is the character for 0 specified for the given counter style. Skip the rest of this algorithm.
2.  If the counter value is negative, instead use the absolute value of the counter value for the remaining steps of this algorithm.
3.  Initially represent the counter value as a decimal number. Starting from the right (ones place), split the decimal number into groups of four digits.
4.  For each group with a non-zero value, append the appropriate group marker to the group. The ones group has no marker.
5.  Within each group, for each digit that is not 0, append the appropriate digit marker to the digit. The ones digit of each group has no marker.
6.  Drop ones:
    - For the Chinese informal styles, for any group with a value between ten and nineteen, remove the tens digit (leave the digit marker).
    - For the Japanese informal and Korean informal styles, if any of the digit markers are preceded by the digit 1, and that digit is not the first digit of the group, remove the digit (leave the digit marker).
    - For Korean informal styles, if the value of the ten-thousands group is 1, drop the digit (leave the digit marker).
7.  Drop zeros:
    - For the Japanese and Korean styles, drop all zero digits.
    - For the Chinese styles, drop any trailing zeros for all non-zero groups and collapse (across groups) each remaining consecutive group of zeros into a single zero digit.
8.  For the Korean styles, insert a space (" " U+0020) between each group.
9.  Replace the digits 0-9 with the appropriate character for the given counter style.
10. If the counter value was negative, prepend the appropriate negative sign character for the given counter style as specified in the table of characters for each style.
11. Return the resultant string as the representation of the counter value.

For all of these counter styles, the descriptors are the same as for the [limited range variants](#limited-range-required), except for the <a id="ref-for-descdef-counter-style-range⑥"></a>[range](#descdef-counter-style-range), which is calc(-1 \* pow(10, 16) + 1) calc(pow(10, 16) - 1).

The following tables define the characters used in these styles:

| Values                               | Codepoints / simp-chinese-informal   | Codepoints / simp-chinese-formal   | Codepoints / trad-chinese-informal   | Codepoints / trad-chinese-formal   |
|--------------------------------------|--------------------------------------|------------------------------------|--------------------------------------|------------------------------------|
| <strong>Digit 0</strong>             | 零 U+96F6                            | 零 U+96F6                          | 零 U+96F6                            | 零 U+96F6                          |
| <strong>Digit 1</strong>             | 一 U+4E00                            | 壹 U+58F9                          | 一 U+4E00                            | 壹 U+58F9                          |
| <strong>Digit 2</strong>             | 二 U+4E8C                            | 贰 U+8D30                          | 二 U+4E8C                            | 貳 U+8CB3                          |
| <strong>Digit 3</strong>             | 三 U+4E09                            | 叁 U+53C1                          | 三 U+4E09                            | 參 U+53C3                          |
| <strong>Digit 4</strong>             | 四 U+56DB                            | 肆 U+8086                          | 四 U+56DB                            | 肆 U+8086                          |
| <strong>Digit 5</strong>             | 五 U+4E94                            | 伍 U+4F0D                          | 五 U+4E94                            | 伍 U+4F0D                          |
| <strong>Digit 6</strong>             | 六 U+516D                            | 陆 U+9646                          | 六 U+516D                            | 陸 U+9678                          |
| <strong>Digit 7</strong>             | 七 U+4E03                            | 柒 U+67D2                          | 七 U+4E03                            | 柒 U+67D2                          |
| <strong>Digit 8</strong>             | 八 U+516B                            | 捌 U+634C                          | 八 U+516B                            | 捌 U+634C                          |
| <strong>Digit 9</strong>             | 九 U+4E5D                            | 玖 U+7396                          | 九 U+4E5D                            | 玖 U+7396                          |
| <strong>Second Digit Marker</strong> | 十 U+5341                            | 拾 U+62FE                          | 十 U+5341                            | 拾 U+62FE                          |
| <strong>Third Digit Marker</strong>  | 百 U+767E                            | 佰 U+4F70                          | 百 U+767E                            | 佰 U+4F70                          |
| <strong>Fourth Digit Marker</strong> | 千 U+5343                            | 仟 U+4EDF                          | 千 U+5343                            | 仟 U+4EDF                          |
| <strong>Second Group Marker</strong> | 万 U+4E07                            | 万 U+4E07                          | 萬 U+842C                            | 萬 U+842C                          |
| <strong>Third Group Marker</strong>  | 亿 U+4EBF                            | 亿 U+4EBF                          | 億 U+5104                            | 億 U+5104                          |
| <strong>Fourth Group Marker</strong> | 万亿 U+4E07 U+4EBF                   | 万亿 U+4E07 U+4EBF                 | 兆 U+5146                            | 兆 U+5146                          |
| <strong>Negative Sign</strong>       | 负 U+8D1F                            | 负 U+8D1F                          | 負 U+8CA0                            | 負 U+8CA0                          |

| Values                               | Codepoints / japanese-informal       | Codepoints / japanese-formal         |
|--------------------------------------|--------------------------------------|--------------------------------------|
| <strong>Digit 0</strong>             | 〇 U+3007                            | 零 U+96F6                            |
| <strong>Digit 1</strong>             | 一 U+4E00                            | 壱 U+58F1                            |
| <strong>Digit 2</strong>             | 二 U+4E8C                            | 弐 U+5F10                            |
| <strong>Digit 3</strong>             | 三 U+4E09                            | 参 U+53C2                            |
| <strong>Digit 4</strong>             | 四 U+56DB                            | 四 U+56DB                            |
| <strong>Digit 5</strong>             | 五 U+4E94                            | 伍 U+4f0D                            |
| <strong>Digit 6</strong>             | 六 U+516D                            | 六 U+516D                            |
| <strong>Digit 7</strong>             | 七 U+4E03                            | 七 U+4E03                            |
| <strong>Digit 8</strong>             | 八 U+516B                            | 八 U+516B                            |
| <strong>Digit 9</strong>             | 九 U+4E5D                            | 九 U+4E5D                            |
| <strong>Second Digit Marker</strong> | 十 U+5341                            | 拾 U+62FE                            |
| <strong>Third Digit Marker</strong>  | 百 U+767E                            | 百 U+767E                            |
| <strong>Fourth Digit Marker</strong> | 千 U+5343                            | 阡 U+9621                            |
| <strong>Second Group Marker</strong> | 万 U+4E07                            | 萬 U+842C                            |
| <strong>Third Group Marker</strong>  | 億 U+5104                            | 億 U+5104                            |
| <strong>Fourth Group Marker</strong> | 兆 U+5146                            | 兆 U+5146                            |
| <strong>Negative Sign</strong>       | マイナス U+30DE U+30A4 U+30CA U+30B9 | マイナス U+30DE U+30A4 U+30CA U+30B9 |

| Values                               | Codepoints / korean-hangul-formal    | Codepoints / korean-hanja-informal   | Codepoints / korean-hanja-formal   |
|--------------------------------------|--------------------------------------|--------------------------------------|------------------------------------|
| <strong>Digit 0</strong>             | 영 U+C601                            | 零 U+96F6                            | 零 U+96F6                          |
| <strong>Digit 1</strong>             | 일 U+C77C                            | 一 U+4E00                            | 壹 U+58F9                          |
| <strong>Digit 2</strong>             | 이 U+C774                            | 二 U+4E8C                            | 貳 U+8CB3                          |
| <strong>Digit 3</strong>             | 삼 U+C0BC                            | 三 U+4E09                            | 參 U+53C3                          |
| <strong>Digit 4</strong>             | 사 U+C0AC                            | 四 U+56DB                            | 四 U+56DB                          |
| <strong>Digit 5</strong>             | 오 U+C624                            | 五 U+4E94                            | 五 U+4E94                          |
| <strong>Digit 6</strong>             | 육 U+C721                            | 六 U+516D                            | 六 U+516D                          |
| <strong>Digit 7</strong>             | 칠 U+CE60                            | 七 U+4E03                            | 七 U+4E03                          |
| <strong>Digit 8</strong>             | 팔 U+D314                            | 八 U+516B                            | 八 U+516B                          |
| <strong>Digit 9</strong>             | 구 U+AD6C                            | 九 U+4E5D                            | 九 U+4E5D                          |
| <strong>Second Digit Marker</strong> | 십 U+C2ED                            | 十 U+5341                            | 拾 U+62FE                          |
| <strong>Third Digit Marker</strong>  | 백 U+BC31                            | 百 U+767E                            | 百 U+767E                          |
| <strong>Fourth Digit Marker</strong> | 천 U+CC9C                            | 千 U+5343                            | 仟 U+4EDF                          |
| <strong>Second Group Marker</strong> | 만 U+B9CC                            | 萬 U+842C                            | 萬 U+842C                          |
| <strong>Third Group Marker</strong>  | 억 U+C5B5                            | 億 U+5104                            | 億 U+5104                          |
| <strong>Fourth Group Marker</strong> | 조 U+C870                            | 兆 U+5146                            | 兆 U+5146                          |
| <strong>Negative Sign</strong>       | 마이너스 U+B9C8 U+C774 U+B108 U+C2A4 | 마이너스 U+B9C8 U+C774 U+B108 U+C2A4 |                                    |

### <a id="ethiopic-numeric-counter-style"></a>7.2.  Ethiopic Numeric Counter Style: <a id="ref-for-valdef-counter-style-name-ethiopic-numeric"></a>[ethiopic-numeric](#valdef-counter-style-name-ethiopic-numeric)[](#ethiopic-numeric-counter-style)

The <a id="valdef-counter-style-name-ethiopic-numeric"></a><strong>ethiopic-numeric</strong> counter style is defined for all positive non-zero numbers. The following algorithm converts decimal digits to ethiopic numbers:

1.  If the number is 1, return "፩" (U+1369).
2.  Split the number into groups of two digits, starting with the least significant decimal digit.
3.  Index each group sequentially, starting from the least significant as group number zero.
4.  If the group has the value zero, or if the group is the most significant one and has the value 1, or if the group has an odd index (as given in the previous step) and has the value 1, then remove the digits (but leave the group, so it still has a separator appended below).
5.  For each remaining digit, substitute the relevant ethiopic character from the list below.
    | Tens / Values   | Tens / Codepoints   | Tens / Codepoints   | Units / Values   | Units / Codepoints   | Units / Codepoints   |
    |-----------------|---------------------|---------------------|------------------|----------------------|----------------------|
    | 10              | ፲                   | U+1372              | 1                | ፩                    | U+1369               |
    | 20              | ፳                   | U+1373              | 2                | ፪                    | U+136A               |
    | 30              | ፴                   | U+1374              | 3                | ፫                    | U+136B               |
    | 40              | ፵                   | U+1375              | 4                | ፬                    | U+136C               |
    | 50              | ፶                   | U+1376              | 5                | ፭                    | U+136D               |
    | 60              | ፷                   | U+1377              | 6                | ፮                    | U+136E               |
    | 70              | ፸                   | U+1378              | 7                | ፯                    | U+136F               |
    | 80              | ፹                   | U+1379              | 8                | ፰                    | U+1370               |
    | 90              | ፺                   | U+137A              | 9                | ፱                    | U+1371               |
6.  For each group with an odd index (as given in the second step), except groups which originally had a value of zero, append ፻ U+137B.
7.  For each group with an even index (as given in the second step), except the group with index 0, append ፼ U+137C.
8.  Concatenate the groups into one string, and return it.

For this system, the name is "ethiopic-numeric", the <a id="ref-for-descdef-counter-style-range⑦"></a>[range](#descdef-counter-style-range) is 1 infinite, the <a id="ref-for-descdef-counter-style-suffix⑤"></a>[suffix](#descdef-counter-style-suffix) is <code>&#34;/ &#34;</code> (U+002F SOLIDUS followed by a U+0020 SPACE), and the rest of the descriptors have their initial value.

<a id="example-b515a2a9"></a>

<strong>Example:</strong>

[](#example-b515a2a9) The decimal number 100, in ethiopic, is ፻ U+137B

The decimal number 78010092, in ethiopic, is ፸፰፻፩፼፺፪ U+1378 U+1370 U+137B U+1369 U+137C U+137A U+136A.

The decimal number 780100000092, in ethiopic, is ፸፰፻፩፼፼፺፪ U+1378 U+1370 U+137B U+1369 U+137C U+137C U+137A U+136A.

Tests

- [counter-ethiopic-numeric.html](https://wpt.fyi/results/css/css-counter-styles/ethiopic-numeric/counter-ethiopic-numeric.html) [(live test)](http://wpt.live/css/css-counter-styles/ethiopic-numeric/counter-ethiopic-numeric.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/ethiopic-numeric/counter-ethiopic-numeric.html)
- css3-counter-styles-068.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/ethiopic-numeric/css3-counter-styles-068.html)
- css3-counter-styles-069.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/ethiopic-numeric/css3-counter-styles-069.html)
- css3-counter-styles-070.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/ethiopic-numeric/css3-counter-styles-070.html)

## <a id="additional-predefined"></a>8.  Additional “Ready-made” Counter Styles[](#additional-predefined)

The [Internationalization Working Group](https://www.w3.org/International/) maintains a collection of additional <a id="ref-for-at-ruledef-counter-style④①"></a>[&#64;counter-style](#at-ruledef-counter-style) rules for various world languages in [Ready-made Counter Styles](https://www.w3.org/TR/predefined-counter-styles/). [\[predefined-counter-styles\]](#biblio-predefined-counter-styles)

As with the counter styles defined in [§ 6 Simple Predefined Counter Styles](#predefined-counters), UAs must include these in their UA stylesheet (or at least act as if these rules were defined at that level).

Note: Authors can use the [Ready-made Counter Styles](https://www.w3.org/TR/predefined-counter-styles/) even in UAs that don’t support them by copying the rules directly into their style sheets.

## <a id="apis"></a>9. APIs[](#apis)

### <a id="extensions-to-cssrule-interface"></a>9.1.  Extensions to the <code>CSSRule</code> interface[](#extensions-to-cssrule-interface)

The <code>CSSRule</code> interface is extended as follows:

<a id="ref-for-cssrule"></a><a id="ref-for-idl-unsigned-short"></a><a id="dom-cssrule-counter&#95;style&#95;rule"></a>

``` text
partial interface CSSRule {
    const unsigned short COUNTER_STYLE_RULE = 11;
};
```

### <a id="the-csscounterstylerule-interface"></a>9.2.  The <code>CSSCounterStyleRule</code> interface[](#the-csscounterstylerule-interface)

The <a id="ref-for-csscounterstylerule"></a>[CSSCounterStyleRule](#csscounterstylerule) interface represents a <a id="ref-for-at-ruledef-counter-style④②"></a>[&#64;counter-style](#at-ruledef-counter-style) rule.

<a id="ref-for-Exposed"></a><a id="csscounterstylerule"></a><a id="ref-for-cssrule①"></a><a id="ref-for-cssomstring"></a><a id="ref-for-dom-csscounterstylerule-name"></a><a id="ref-for-cssomstring①"></a><a id="ref-for-dom-csscounterstylerule-system"></a><a id="ref-for-cssomstring②"></a><a id="ref-for-dom-csscounterstylerule-symbols"></a><a id="ref-for-cssomstring③"></a><a id="ref-for-dom-csscounterstylerule-additivesymbols"></a><a id="ref-for-cssomstring④"></a><a id="ref-for-dom-csscounterstylerule-negative"></a><a id="ref-for-cssomstring⑤"></a><a id="ref-for-dom-csscounterstylerule-prefix"></a><a id="ref-for-cssomstring⑥"></a><a id="ref-for-dom-csscounterstylerule-suffix"></a><a id="ref-for-cssomstring⑦"></a><a id="ref-for-dom-csscounterstylerule-range"></a><a id="ref-for-cssomstring⑧"></a><a id="ref-for-dom-csscounterstylerule-pad"></a><a id="ref-for-cssomstring⑨"></a><a id="ref-for-dom-csscounterstylerule-speakas"></a><a id="ref-for-cssomstring①⓪"></a><a id="ref-for-dom-csscounterstylerule-fallback"></a>

``` text
[Exposed=Window]
interface CSSCounterStyleRule : CSSRule {
  attribute CSSOMString name;
  attribute CSSOMString system;
  attribute CSSOMString symbols;
  attribute CSSOMString additiveSymbols;
  attribute CSSOMString negative;
  attribute CSSOMString prefix;
  attribute CSSOMString suffix;
  attribute CSSOMString range;
  attribute CSSOMString pad;
  attribute CSSOMString speakAs;
  attribute CSSOMString fallback;
};
```

Tests

- [cssom-additive-symbols-setter-invalid.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-additive-symbols-setter-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-additive-symbols-setter-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-additive-symbols-setter-invalid.html)
- [cssom-additive-symbols-setter.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-additive-symbols-setter.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-additive-symbols-setter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-additive-symbols-setter.html)
- [cssom-fallback-setter-invalid.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-fallback-setter-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-fallback-setter-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-fallback-setter-invalid.html)
- [cssom-fallback-setter.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-fallback-setter.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-fallback-setter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-fallback-setter.html)
- [cssom-name-setter-invalid.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-name-setter-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-name-setter-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-name-setter-invalid.html)
- [cssom-name-setter.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-name-setter.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-name-setter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-name-setter.html)
- [cssom-negative-setter-invalid.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-negative-setter-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-negative-setter-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-negative-setter-invalid.html)
- [cssom-negative-setter.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-negative-setter.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-negative-setter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-negative-setter.html)
- [cssom-pad-setter-invalid.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-pad-setter-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-pad-setter-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-pad-setter-invalid.html)
- [cssom-pad-setter.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-pad-setter.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-pad-setter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-pad-setter.html)
- [cssom-prefix-suffix-setter-invalid.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-prefix-suffix-setter-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-prefix-suffix-setter-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-prefix-suffix-setter-invalid.html)
- [cssom-prefix-suffix-setter.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-prefix-suffix-setter.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-prefix-suffix-setter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-prefix-suffix-setter.html)
- [cssom-range-setter-invalid.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-range-setter-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-range-setter-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-range-setter-invalid.html)
- [cssom-range-setter.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-range-setter.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-range-setter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-range-setter.html)
- [cssom-symbols-setter-invalid.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-symbols-setter-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-symbols-setter-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-symbols-setter-invalid.html)
- [cssom-symbols-setter.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-symbols-setter.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-symbols-setter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-symbols-setter.html)
- [cssom-system-setter-1.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-system-setter-1.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-system-setter-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-system-setter-1.html)
- [cssom-system-setter-2.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-system-setter-2.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-system-setter-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-system-setter-2.html)
- [cssom-system-setter-invalid.html](https://wpt.fyi/results/css/css-counter-styles/cssom/cssom-system-setter-invalid.html) [(live test)](http://wpt.live/css/css-counter-styles/cssom/cssom-system-setter-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-counter-styles/cssom/cssom-system-setter-invalid.html)

<strong><a id="dom-csscounterstylerule-name"></a><strong><code>name</code></strong>, of type <a id="ref-for-cssomstring①①"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

The <var>name</var> attribute on getting must return a <code>CSSOMString</code> object that contains the serialization of the <a id="ref-for-typedef-counter-style-name①⓪"></a>[\<counter-style-name\>](#typedef-counter-style-name) defined for the associated rule.

On setting the <var>name</var> attribute, run the following steps:

1.  If the value is an <a id="ref-for-ascii-case-insensitive②"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "none" or one of the <u>non-overridable counter styles</u>, do nothing and return.

2.  If the value is an <a id="ref-for-ascii-case-insensitive③"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for any of the predefined <a id="ref-for-counter-style⑦"></a>[counter styles](#counter-style), lowercase it.

3.  Replace the associated rule’s name with an <a id="ref-for-css-css-identifier"></a>[identifier](https://drafts.csswg.org/css-values-3/#css-css-identifier) equal to the value.

<strong><a id="dom-csscounterstylerule-system"></a><strong><code>system</code></strong>, of type <a id="ref-for-cssomstring①②"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

<strong><a id="dom-csscounterstylerule-symbols"></a><strong><code>symbols</code></strong>, of type <a id="ref-for-cssomstring①③"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

<strong><a id="dom-csscounterstylerule-additivesymbols"></a><strong><code>additiveSymbols</code></strong>, of type <a id="ref-for-cssomstring①④"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

<strong><a id="dom-csscounterstylerule-negative"></a><strong><code>negative</code></strong>, of type <a id="ref-for-cssomstring①⑤"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

<strong><a id="dom-csscounterstylerule-prefix"></a><strong><code>prefix</code></strong>, of type <a id="ref-for-cssomstring①⑥"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

<strong><a id="dom-csscounterstylerule-suffix"></a><strong><code>suffix</code></strong>, of type <a id="ref-for-cssomstring①⑦"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

<strong><a id="dom-csscounterstylerule-range"></a><strong><code>range</code></strong>, of type <a id="ref-for-cssomstring①⑧"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

<strong><a id="dom-csscounterstylerule-pad"></a><strong><code>pad</code></strong>, of type <a id="ref-for-cssomstring①⑨"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

<strong><a id="dom-csscounterstylerule-speakas"></a><strong><code>speakAs</code></strong>, of type <a id="ref-for-cssomstring②⓪"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

<strong><a id="dom-csscounterstylerule-fallback"></a><strong><code>fallback</code></strong>, of type <a id="ref-for-cssomstring②①"></a>[CSSOMString](https://drafts.csswg.org/cssom-1/#cssomstring)</strong>

The remaining attributes on getting must return a <code>CSSOMString</code> object that contains the serialization of the associated descriptor defined for the associated rule. If the descriptor was not specified in the associated rule, the attribute must return an empty string.

On setting, run the following steps:

1.  <a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>[parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <a id="ref-for-the-given-value"></a>[the given value](https://webidl.spec.whatwg.org/#the-given-value) as the descriptor associated with the attribute.

2.  If the result is invalid according to the given descriptor’s grammar, or would cause the <a id="ref-for-at-ruledef-counter-style④③"></a>[&#64;counter-style](#at-ruledef-counter-style) rule to not define a counter style, do nothing and abort these steps. (For example, some systems require the <a id="ref-for-descdef-counter-style-symbols①⑦"></a>[symbols](#descdef-counter-style-symbols) descriptor to contain two values.)

3.  If the attribute being set is <a id="ref-for-dom-csscounterstylerule-system①"></a>[system](#dom-csscounterstylerule-system), and the new value would change the algorithm used, do nothing and abort these steps. It’s okay to change an aspect of the algorithm, like the

    <strong>Note:</strong>

    <a id="ref-for-first-symbol-value④"></a>[first symbol value](#first-symbol-value) of a <a id="ref-for-valdef-counter-style-system-fixed⑤"></a>[fixed](#valdef-counter-style-system-fixed) system.

4.  Set the descriptor to the value.

## <a id="ua-stylesheet"></a>10.  Sample style sheet for HTML[](#ua-stylesheet)

This section is informative, not normative. HTML itself defines the styles that apply to its elements, and in some cases defers to the user agent’s discretion.

``` text
details > summary {
  display: list-item;
  list-style: disclosure-closed inside;
}

details[open] > summary {
  list-style: disclosure-open inside;
}
```

## <a id="changes"></a> Changes[](#changes)

### <a id="changes-2021"></a> Changes since the July 2021 Candidate Recommendation[](#changes-2021)

Significant changes since the [July 27 2021 Candidate Recommendation](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/):

- Added <a id="ref-for-descdef-counter-style-fallback⑦"></a>[fallback: cjk-decimal](#descdef-counter-style-fallback) descriptor to <a id="ref-for-valdef-counter-style-name-cjk-earthly-branch①"></a>[cjk-earthly-branch](#valdef-counter-style-name-cjk-earthly-branch) and <a id="ref-for-valdef-counter-style-name-cjk-heavenly-stem①"></a>[cjk-heavenly-stem](#valdef-counter-style-name-cjk-heavenly-stem). ([Issue 8975](https://github.com/w3c/csswg-drafts/issues/8975))

- Restored the fallback: cjk-decimal descriptor to the Korean styles which was accidentally dropped in [this commit](https://github.com/w3c/csswg-drafts/commit/20a3d9ca02f235d8af13aa2bb05f725448371ad2). ([Issue 8870](https://github.com/w3c/csswg-drafts/issues/8870))

- Fixed the accidental desync between disallowed name keywords in the syntax vs the OM. ([Issue 8186](https://github.com/w3c/csswg-drafts/issues/8186))

- Added conformance requirement to support [Ready-made Counter Styles](https://www.w3.org/TR/predefined-counter-styles/). ([Issue 8636](https://github.com/w3c/csswg-drafts/issues/8636))

- Added Web Platform Tests coverage.

- Added the <a id="ref-for-typedef-symbolic-glyph⑤"></a>[\<symbolic-glyph\>](#typedef-symbolic-glyph) production for easier cross-referencing.

### <a id="changes-2017"></a> Changes since the December 2017 Candidate Recommendation[](#changes-2017)

Significant changes since the [December 14 2017 Candidate Recommendation](https://www.w3.org/TR/2017/CR-css-counter-styles-3-20171214/):

- Made <a id="ref-for-valdef-list-style-type-none①"></a>[none](https://drafts.csswg.org/css-lists-3/#valdef-list-style-type-none), <a id="ref-for-decimal①②"></a>[decimal](#decimal), <a id="ref-for-disc①"></a>[disc](#disc), <a id="ref-for-circle①"></a>[circle](#circle), <a id="ref-for-square①"></a>[square](#square), <a id="ref-for-disclosure-open③"></a>[disclosure-open](#disclosure-open), disclosure-close non-overridable. ([Issue 3584](https://github.com/w3c/csswg-drafts/issues/3584))

- Clarified counter-style lookups in Shadow DOM. ([Issue 5693](https://github.com/w3c/csswg-drafts/issues/5693))

- Clarified what happens in various invalid <a id="ref-for-at-ruledef-counter-style④④"></a>[&#64;counter-style](#at-ruledef-counter-style) situations. ([Issue 5698](https://github.com/w3c/csswg-drafts/issues/5698), [Issue 5717](https://github.com/w3c/csswg-drafts/issues/5717))

- Fixed divide-by-zero error in additive algorithm. ([Issue 5784](https://github.com/w3c/csswg-drafts/issues/5784))

- Fixed <a id="ref-for-square②"></a>[square](#square) symbol to not use an emoji symbol. ([Issue 6200](https://github.com/w3c/csswg-drafts/issues/6200))

- Allowed UAs to override the font choice of predefined symbolic counter styles. ([Issue 6201](https://github.com/w3c/csswg-drafts/issues/6201))

- Restricted special rendering of predefined symbolic counter styles to usage as a list marker via <a id="ref-for-propdef-list-style-type③"></a>[list-style-type](https://drafts.csswg.org/css-lists-3/#propdef-list-style-type). ([Issue 6201](https://github.com/w3c/csswg-drafts/issues/6201))

- Clarified that <a id="ref-for-descdef-counter-style-speak-as⑦"></a>[speak-as](#descdef-counter-style-speak-as) represents spoken output; it may be used for other AT. ([Issue 6040](https://github.com/w3c/csswg-drafts/issues/6040))

  > A counter style can be constructed with a meaning that is obvious visually, but impossible to meaningfully represent via a speech synthesizer or other non-visual means, or possible but nonsensical when naively read out <u>loud</u> . The <a id="ref-for-descdef-counter-style-speak-as⑧"></a>[speak-as](#descdef-counter-style-speak-as) descriptor describes how to synthesize the spoken form of a counter formatted with the given counter style. <u>Assistive technologies should use this spoken form when reading out the counter style, and may use the <a id="ref-for-descdef-counter-style-speak-as⑨"></a>[speak-as](#descdef-counter-style-speak-as) value to inform transformations to outputs other than speech.</u>

A [Disposition of Comments](https://drafts.csswg.org/css-counter-styles-3/issues-cr-2017) is available.

### <a id="changes-jun-2015"></a> Changes since the June 2015 Candidate Recommendation[](#changes-jun-2015)

Significant changes since the [June 11 2015 Candidate Recommendation](https://www.w3.org/TR/2015/CR-css-counter-styles-3-20150611/):

- Exclude <a id="ref-for-valdef-list-style-type-none②"></a>[none](https://drafts.csswg.org/css-lists-3/#valdef-list-style-type-none) and <a id="ref-for-disc②"></a>[disc](#disc) from being the name of a counter style.

- When setting CSSCounterStyle.name, take the string directly; don’t <a id="ref-for-css-parse-something-according-to-a-css-grammar①"></a>[parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) it as an ident.

- Clarify that counter styles are read out in the element’s <a id="ref-for-content-language③"></a>[content language](https://drafts.csswg.org/css-text-4/#content-language).

- Clarified that <a id="ref-for-descdef-counter-style-additive-symbols⑧"></a>[additive-symbols](#descdef-counter-style-additive-symbols) tuples must be of <em>strictly</em> decreasing weight.

- Specified that invalid values just invalidate the declaration, not the whole rule.

- <a id="ref-for-at-ruledef-counter-style④⑤"></a>[&#64;counter-style](#at-ruledef-counter-style) rules that are invalid due to missing descriptors just fail to create a <a id="ref-for-counter-style⑧"></a>[counter style](#counter-style); they’re otherwise still valid rules.

- Changed syntax to use <a id="ref-for-css-bracketed-range-notation"></a>[CSS bracketed range notation](https://drafts.csswg.org/css-values-4/#css-bracketed-range-notation) to reflect the prose restrictions on negative values.

A [Disposition of Comments](https://drafts.csswg.org/css-counter-styles-3/issues-cr-20150611) is available.

### <a id="changes-feb-2015"></a> Changes since the Feb 2015 Candidate Recommendation[](#changes-feb-2015)

- Allowed UAs to extend the <a id="ref-for-hebrew②"></a>[hebrew](#hebrew) style past the spec-defined limits (since the current limits are mostly just an artifact of how annoying it is to go higher with the <a id="ref-for-at-ruledef-counter-style④⑥"></a>[&#64;counter-style](#at-ruledef-counter-style)-based definition).

## <a id="acknowledgments"></a> Acknowledgments[](#acknowledgments)

The following people and documentation they wrote were very useful for defining the numbering systems: Alexander Savenkov, Arron Eicholz, Aryeh Gregor, Christopher Hoess, Daniel Yacob, Frank Tang, Jonathan Rosenne, Karl Ove Hufthammer, Musheg Arakelyan, Nariné Renard Karapetyan, Randall Bart, Richard Ishida, Simon Montagu (Mozilla, smontagu&#64;smontagu.org)

Special thanks to Xidorn Quan for <em>extensive</em> reviews of all aspects of the spec, and also to Simon Sapin and Håkon Wium Lie for their review comments.

## <a id="privacy"></a> Privacy Considerations[](#privacy)

This specification introduces no new privacy considerations.

## <a id="security"></a> Security Considerations[](#security)

This specification introduces no new security considerations.

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

- [additive](#valdef-counter-style-system-additive), in § 3.1.6
- [additive-symbols](#descdef-counter-style-additive-symbols), in § 3.8
- [additiveSymbols](#dom-csscounterstylerule-additivesymbols), in § 9.2
- [additive tuple](#additive-tuple), in § 3.8
- [alphabetic](#valdef-counter-style-system-alphabetic), in § 3.1.4
- [arabic-indic](#valdef-counter-style-name-arabic-indic), in § 6.1
- [armenian](#armenian), in § 6.1
- auto
  - [value for &#64;counter-style/range](#valdef-counter-style-range-auto), in § 3.5
  - [value for &#64;counter-style/speak-as](#valdef-counter-style-speak-as-auto), in § 3.9
- [bengali](#valdef-counter-style-name-bengali), in § 6.1
- [box-corner](#box-corner), in § 3.1.2
- [bullets](#valdef-counter-style-speak-as-bullets), in § 3.9
- [cambodian](#valdef-counter-style-name-cambodian), in § 6.1
- [circle](#circle), in § 6.3
- [circled-lower-latin](#circled-lower-latin), in § 3.9
- [cjk-decimal](#cjk-decimal), in § 6.1
- [cjk-earthly-branch](#valdef-counter-style-name-cjk-earthly-branch), in § 6.4
- [cjk-heavenly-stem](#valdef-counter-style-name-cjk-heavenly-stem), in § 6.4
- [cjk-ideographic](#cjk-ideographic), in § 7.1.1.3
- [\<counter-style\>](#typedef-counter-style), in § 5
- [&#64;counter-style](#at-ruledef-counter-style), in § 3
- [counter style](#counter-style), in § 2
- \<counter-style-name\>
  - [(type)](#typedef-counter-style-name), in § 2.1
  - [value for &#64;counter-style/speak-as](#valdef-counter-style-speak-as-counter-style-name), in § 3.9
- [COUNTER_STYLE_RULE](#dom-cssrule-counter_style_rule), in § 9.1
- [counter symbol](#counter-symbol), in § 3.8
- [CSSCounterStyleRule](#csscounterstylerule), in § 9.2
- [cyclic](#valdef-counter-style-system-cyclic), in § 3.1.1
- [decimal](#decimal), in § 6.1
- [decimal-leading-zero](#decimal-leading-zero), in § 6.1
- [devanagari](#valdef-counter-style-name-devanagari), in § 6.1
- [dice](#dice), in § 3.1.6
- [disc](#disc), in § 6.3
- [disclosure-closed](#disclosure-closed), in § 6.3
- [disclosure-open](#disclosure-open), in § 6.3
- [ethiopic-numeric](#valdef-counter-style-name-ethiopic-numeric), in § 7.2
- [extends](#valdef-counter-style-system-extends), in § 3.1.7
- fallback
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-fallback), in § 9.2
  - [descriptor for &#64;counter-style](#descdef-counter-style-fallback), in § 3.7
- [first symbol value](#first-symbol-value), in § 3.1.2
- [fixed](#valdef-counter-style-system-fixed), in § 3.1.2
- [footnote](#footnote), in § 3.1.3
- [generate a counter](#generate-a-counter), in § 2.2
- [generate a counter representation](#generate-a-counter), in § 2.2
- [georgian](#georgian), in § 6.1
- [go](#go), in § 3.1.4
- [gujarati](#valdef-counter-style-name-gujarati), in § 6.1
- [gurmukhi](#valdef-counter-style-name-gurmukhi), in § 6.1
- [hebrew](#hebrew), in § 6.1
- [hiragana](#hiragana), in § 6.2
- [hiragana-iroha](#hiragana-iroha), in § 6.2
- [initial representation for the counter value](#initial-representation-for-the-counter-value), in § 2.2
- [japanese-formal](#japanese-formal), in § 7.1.1.1
- [japanese-informal](#japanese-informal), in § 7.1.1.1
- [kannada](#valdef-counter-style-name-kannada), in § 6.1
- [katakana](#katakana), in § 6.2
- [katakana-iroha](#katakana-iroha), in § 6.2
- [khmer](#valdef-counter-style-name-khmer), in § 6.1
- [korean-hangul-formal](#korean-hangul-formal), in § 7.1.1.2
- [korean-hanja-formal](#korean-hanja-formal), in § 7.1.1.2
- [korean-hanja-informal](#korean-hanja-informal), in § 7.1.1.2
- [lao](#valdef-counter-style-name-lao), in § 6.1
- [lower-alpha](#lower-alpha), in § 6.2
- [lower-armenian](#valdef-counter-style-name-lower-armenian), in § 6.1
- [lower-greek](#lower-greek), in § 6.2
- [lower-latin](#lower-latin), in § 6.2
- [lower-roman](#lower-roman), in § 6.1
- [malayalam](#valdef-counter-style-name-malayalam), in § 6.1
- [mongolian](#valdef-counter-style-name-mongolian), in § 6.1
- [myanmar](#valdef-counter-style-name-myanmar), in § 6.1
- [name](#dom-csscounterstylerule-name), in § 9.2
- negative
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-negative), in § 9.2
  - [descriptor for &#64;counter-style](#descdef-counter-style-negative), in § 3.2
- [non-overridable counter-styles](#non-overridable-counter-styles), in § 3
- [numbers](#valdef-counter-style-speak-as-numbers), in § 3.9
- [numeric](#valdef-counter-style-system-numeric), in § 3.1.5
- [oriya](#valdef-counter-style-name-oriya), in § 6.1
- pad
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-pad), in § 9.2
  - [descriptor for &#64;counter-style](#descdef-counter-style-pad), in § 3.6
- [persian](#valdef-counter-style-name-persian), in § 6.1
- [predefined symbolic counter style](#predefined-symbolic-counter-style), in § 6.3
- prefix
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-prefix), in § 9.2
  - [descriptor for &#64;counter-style](#descdef-counter-style-prefix), in § 3.3
- range
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-range), in § 9.2
  - [descriptor for &#64;counter-style](#descdef-counter-style-range), in § 3.5
- [simp-chinese-formal](#simp-chinese-formal), in § 7.1.1.3
- [simp-chinese-informal](#simp-chinese-informal), in § 7.1.1.3
- [speak-as](#descdef-counter-style-speak-as), in § 3.9
- [speakAs](#dom-csscounterstylerule-speakas), in § 9.2
- [spell-out](#valdef-counter-style-speak-as-spell-out), in § 3.9
- [square](#square), in § 6.3
- suffix
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-suffix), in § 9.2
  - [descriptor for &#64;counter-style](#descdef-counter-style-suffix), in § 3.4
- [\<symbol\>](#typedef-symbol), in § 3.8
- [symbolic](#valdef-system-symbolic), in § 3.1.3
- [\<symbolic-glyph\>](#typedef-symbolic-glyph), in § 6.3
- symbols
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-symbols), in § 9.2
  - [descriptor for &#64;counter-style](#descdef-counter-style-symbols), in § 3.8
- [symbols()](#funcdef-symbols), in § 4
- [\<symbols-type\>](#typedef-symbols-type), in § 4
- system
  - [attribute for CSSCounterStyleRule](#dom-csscounterstylerule-system), in § 9.2
  - [descriptor for &#64;counter-style](#descdef-counter-style-system), in § 3.1
- [tamil](#valdef-counter-style-name-tamil), in § 6.1
- [telugu](#valdef-counter-style-name-telugu), in § 6.1
- [thai](#valdef-counter-style-name-thai), in § 6.1
- [tibetan](#valdef-counter-style-name-tibetan), in § 6.1
- [trad-chinese-formal](#trad-chinese-formal), in § 7.1.1.3
- [trad-chinese-informal](#trad-chinese-informal), in § 7.1.1.3
- [triangle](#triangle), in § 3.1.1
- [trinary](#trinary), in § 3.1.5
- [upper-alpha](#upper-alpha), in § 6.2
- [upper-alpha-legal](#upper-alpha-legal), in § 3.1.3
- [upper-armenian](#valdef-counter-style-name-upper-armenian), in § 6.1
- [upper-latin](#upper-latin), in § 6.2
- [upper-roman](#upper-roman), in § 6.1
- [use a negative sign](#use-a-negative-sign), in § 3.2
- [uses a negative sign](#use-a-negative-sign), in § 3.2
- [words](#valdef-counter-style-speak-as-words), in § 3.9

### <a id="index-defined-elsewhere"></a>Terms defined by reference[](#index-defined-elsewhere)

- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="9cd25054"></a>computed value
- \[CSS-CONTENT-3\] defines the following terms:
  - <a id="2467179e"></a>content
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="998b9e6e"></a>\<image\>
  - <a id="2bceefb2"></a>default object size
- \[CSS-LISTS-3\] defines the following terms:
  - <a id="64fba2c3"></a>\<counter\>
  - <a id="4728e44c"></a>counter()
  - <a id="042709b2"></a>counters()
  - <a id="5e8bfe59"></a>inside
  - <a id="3b47d5a5"></a>list-style
  - <a id="de56fbfe"></a>list-style-type
  - <a id="98b5eea2"></a>none
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="5875aeb9"></a>::marker
- \[CSS-SHADOW-1\] defines the following terms:
  - <a id="4b20c2d8"></a>tree-scoped name
  - <a id="25da4c7e"></a>tree-scoped reference
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="84252421"></a>\<declaration-list\>
  - <a id="762610c7"></a>at-rule
  - <a id="76c4403d"></a>parse
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="6eb02dd5"></a>grapheme cluster
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="5879b11d"></a>content language
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="09f79acc"></a>identifier
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="68487d22"></a>\#
  - <a id="f0809abc"></a>&&
  - <a id="bedd9d42"></a>+
  - <a id="e274345c"></a>\<custom-ident\>
  - <a id="70c9f859"></a>\<integer\>
  - <a id="977d3003"></a>\<string\>
  - <a id="537cf076"></a>?
  - <a id="ad47bbf0"></a>CSS bracketed range notation
  - <a id="358fd6ff"></a>CSS-wide keywords
  - <a id="b0d8ebd9"></a>{A}
  - <a id="6ec67710"></a>\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="933fdf29"></a>horizontal writing mode
  - <a id="7f0d729a"></a>ltr
  - <a id="06bc643d"></a>rtl
  - <a id="a73617e0"></a>writing mode
- \[CSSOM-1\] defines the following terms:
  - <a id="5fced98b"></a>CSSOMString
  - <a id="ebaeb088"></a>CSSRule
- \[HTML\] defines the following terms:
  - <a id="1748bbeb"></a>details
- \[INFRA\] defines the following terms:
  - <a id="7f9469b5"></a>ASCII case-insensitive
  - <a id="f937b7b6"></a>continue
- \[WEBIDL\] defines the following terms:
  - <a id="889e932f"></a>Exposed
  - <a id="3b5f2424"></a>the given value
  - <a id="450958f7"></a>unsigned short

## <a id="references"></a>References[](#references)

### <a id="normative"></a>Normative References[](#normative)

<a id="biblio-css-cascade-5"></a><strong>\[CSS-CASCADE-5\]</strong>

Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://drafts.csswg.org/css-cascade-5/). URL: <https://drafts.csswg.org/css-cascade-5/>

<a id="biblio-css-images-3"></a><strong>\[CSS-IMAGES-3\]</strong>

Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://drafts.csswg.org/css-images-3/). URL: <https://drafts.csswg.org/css-images-3/>

<a id="biblio-css-lists-3"></a><strong>\[CSS-LISTS-3\]</strong>

Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://drafts.csswg.org/css-lists-3/). URL: <https://drafts.csswg.org/css-lists-3/>

<a id="biblio-css-shadow-1"></a><strong>\[CSS-SHADOW-1\]</strong>

[CSS Shadow Module Level 1](https://drafts.csswg.org/css-shadow-1/). Editor's Draft. URL: <https://drafts.csswg.org/css-shadow-1/>

<a id="biblio-css-syntax-3"></a><strong>\[CSS-SYNTAX-3\]</strong>

Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/). URL: <https://drafts.csswg.org/css-syntax/>

<a id="biblio-css-text-3"></a><strong>\[CSS-TEXT-3\]</strong>

Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://drafts.csswg.org/css-text-3/). URL: <https://drafts.csswg.org/css-text-3/>

<a id="biblio-css-text-4"></a><strong>\[CSS-TEXT-4\]</strong>

Elika Etemad; et al. [CSS Text Module Level 4](https://drafts.csswg.org/css-text-4/). URL: <https://drafts.csswg.org/css-text-4/>

<a id="biblio-css-values-3"></a><strong>\[CSS-VALUES-3\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://drafts.csswg.org/css-values-3/). URL: <https://drafts.csswg.org/css-values-3/>

<a id="biblio-css-values-4"></a><strong>\[CSS-VALUES-4\]</strong>

Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: <https://drafts.csswg.org/css-values-4/>

<a id="biblio-css-writing-modes-4"></a><strong>\[CSS-WRITING-MODES-4\]</strong>

Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://drafts.csswg.org/css-writing-modes-4/). URL: <https://drafts.csswg.org/css-writing-modes-4/>

<a id="biblio-css21"></a><strong>\[CSS21\]</strong>

Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://drafts.csswg.org/css2/). URL: <https://drafts.csswg.org/css2/>

<a id="biblio-cssom-1"></a><strong>\[CSSOM-1\]</strong>

Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://drafts.csswg.org/cssom/). URL: <https://drafts.csswg.org/cssom/>

<a id="biblio-html"></a><strong>\[HTML\]</strong>

Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: <https://html.spec.whatwg.org/multipage/>

<a id="biblio-infra"></a><strong>\[INFRA\]</strong>

Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: <https://infra.spec.whatwg.org/>

<a id="biblio-rfc2119"></a><strong>\[RFC2119\]</strong>

S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: <https://datatracker.ietf.org/doc/html/rfc2119>

<a id="biblio-webidl"></a><strong>\[WEBIDL\]</strong>

Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: <https://webidl.spec.whatwg.org/>

### <a id="informative"></a>Non-Normative References[](#informative)

<a id="biblio-css-content-3"></a><strong>\[CSS-CONTENT-3\]</strong>

Elika Etemad; Mike Bremford. [CSS Generated Content Module Level 3](https://drafts.csswg.org/css-content-3/). URL: <https://drafts.csswg.org/css-content-3/>

<a id="biblio-css-pseudo-4"></a><strong>\[CSS-PSEUDO-4\]</strong>

Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://drafts.csswg.org/css-pseudo-4/). URL: <https://drafts.csswg.org/css-pseudo-4/>

<a id="biblio-predefined-counter-styles"></a><strong>\[PREDEFINED-COUNTER-STYLES\]</strong>

Richard Ishida. [Ready-made Counter Styles](https://w3c.github.io/predefined-counter-styles/). URL: <https://w3c.github.io/predefined-counter-styles/>

## <a id="property-index"></a>Property Index[](#property-index)

No properties defined.

### <a id="counter-style-descriptor-table"></a><a id="ref-for-at-ruledef-counter-style④⑦"></a>[&#64;counter-style](#at-ruledef-counter-style) Descriptors[](#counter-style-descriptor-table)

| Name                                                                                                                                     | Value                                                                                                                     | Initial  |
|------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------|----------|
| <strong><a id="ref-for-descdef-counter-style-additive-symbols⑨"></a>[additive-symbols](#descdef-counter-style-additive-symbols)</strong> | \[ \<integer \[0,∞\]\> && \<symbol\> \]#                                                                                  | n/a      |
| <strong><a id="ref-for-descdef-counter-style-fallback⑧"></a>[fallback](#descdef-counter-style-fallback)</strong>                         | \<counter-style-name\>                                                                                                    | decimal  |
| <strong><a id="ref-for-descdef-counter-style-negative⑦"></a>[negative](#descdef-counter-style-negative)</strong>                         | \<symbol\> \<symbol\>?                                                                                                    | "-"      |
| <strong><a id="ref-for-descdef-counter-style-pad①⓪"></a>[pad](#descdef-counter-style-pad)</strong>                                       | \<integer \[0,∞\]\> && \<symbol\>                                                                                         | 0 ""     |
| <strong><a id="ref-for-descdef-counter-style-prefix④"></a>[prefix](#descdef-counter-style-prefix)</strong>                               | \<symbol\>                                                                                                                | ""       |
| <strong><a id="ref-for-descdef-counter-style-range⑧"></a>[range](#descdef-counter-style-range)</strong>                                  | \[ \[ \<integer\> \| infinite \]{2} \]# \| auto                                                                           | auto     |
| <strong><a id="ref-for-descdef-counter-style-speak-as①⓪"></a>[speak-as](#descdef-counter-style-speak-as)</strong>                        | auto \| bullets \| numbers \| words \| spell-out \| \<counter-style-name\>                                                | auto     |
| <strong><a id="ref-for-descdef-counter-style-suffix⑥"></a>[suffix](#descdef-counter-style-suffix)</strong>                               | \<symbol\>                                                                                                                | ". "     |
| <strong><a id="ref-for-descdef-counter-style-symbols①⑧"></a>[symbols](#descdef-counter-style-symbols)</strong>                           | \<symbol\>+                                                                                                               | n/a      |
| <strong><a id="ref-for-descdef-counter-style-system①①"></a>[system](#descdef-counter-style-system)</strong>                              | cyclic \| numeric \| alphabetic \| symbolic \| additive \| \[fixed \<integer\>?\] \| \[ extends \<counter-style-name\> \] | symbolic |

## <a id="idl-index"></a>IDL Index[](#idl-index)

``` text
partial interface CSSRule {
    const unsigned short COUNTER_STYLE_RULE = 11;
};

[Exposed=Window]
interface CSSCounterStyleRule : CSSRule {
  attribute CSSOMString name;
  attribute CSSOMString system;
  attribute CSSOMString symbols;
  attribute CSSOMString additiveSymbols;
  attribute CSSOMString negative;
  attribute CSSOMString prefix;
  attribute CSSOMString suffix;
  attribute CSSOMString range;
  attribute CSSOMString pad;
  attribute CSSOMString speakAs;
  attribute CSSOMString fallback;
};
```

<strong>✔</strong>MDN

[CSSCounterStyleRule/additiveSymbols](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/additiveSymbols)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule/fallback](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/fallback)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule/name](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/name)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule/negative](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/negative)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule/pad](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/pad)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule/prefix](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/prefix)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule/range](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/range)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule/speakAs](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/speakAs)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule/suffix](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/suffix)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule/symbols](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/symbols)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule/system](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule/system)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[CSSCounterStyleRule](https://developer.mozilla.org/en-US/docs/Web/API/CSSCounterStyleRule)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[&#64;counter-style/additive-symbols](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style/additive-symbols)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

[&#64;counter-style/symbols](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style/symbols)

In only one current engine.

FirefoxNoneSafari17+ChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[&#64;counter-style/fallback](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style/fallback)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[&#64;counter-style/negative](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style/negative)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

[&#64;counter-style/system](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style/system)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[&#64;counter-style/pad](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style/pad)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[&#64;counter-style/prefix](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style/prefix)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[&#64;counter-style/range](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style/range)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[&#64;counter-style/speak-as](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style/speak-as)

In only one current engine.

Firefox33+SafariNoneChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

MDN

[&#64;counter-style/suffix](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style/suffix)

Firefox33+SafariNoneChrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[&#64;counter-style](https://developer.mozilla.org/en-US/docs/Web/CSS/@counter-style)

In all current engines.

Firefox33+Safari17+Chrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>⚠</strong>MDN

[symbols()](https://developer.mozilla.org/en-US/docs/Web/CSS/symbols())

In only one current engine.

Firefox35+SafariNoneChromeNone

------------------------------------------------------------------------

Opera?EdgeNone

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

[symbols()](https://developer.mozilla.org/en-US/docs/Web/CSS/symbols())

Firefox35+SafariNoneChrome91+

------------------------------------------------------------------------

Opera?Edge91+

------------------------------------------------------------------------

Edge (Legacy)?IENone

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView?Samsung Internet?Opera Mobile?

<strong>✔</strong>MDN

[list-style-type](https://developer.mozilla.org/en-US/docs/Web/CSS/list-style-type)

In all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera3.5+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+IE4+

------------------------------------------------------------------------

Firefox for Android?iOS Safari?Chrome for Android?Android WebView37+Samsung Internet?Opera Mobile10.1+
