# Custom-media standard source capture

Retrieved 2026-10-12 from https://drafts.csswg.org/mediaqueries-5/; source status: W3C Working Draft,
    8 October 2026. Page revision: ['fbd84ec61ae093787ced21cd48d3a7536e04b50a']. Full HTML SHA-256: `4c4756cc716fb95113dcbd56f7832efcb59fc28f001e7cd7d657ba7c1cc79410`. Bounded input SHA-256: `d88a880d91d542d49fa20db89d98a16c9327d315583e51433e9e59fe03d5c629`.

Retains complete sections 10 (including 10.1) and 11, plus original header and legal notice. Converted with existing Pandoc 3.1.11.1 and repository HTML preparation/semantic filter. Editorial issue formatting is source evidence, not a normative registration algorithm. Source status and the inconsistent --foo/_foo example are preserved. There are no complex tables in these sections.

Attribution and reformatting notice added for Surgeist on 2026-10-12. Original English source remains authoritative. This bounded reference is implementation support, not a new specification. Material is under the [W3C Software and Document License 2023](../licenses/w3c/software-license-2023.txt); W3C is not responsible for conversion errors. Original legal notice follows.

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

## <a id="custom-mq"></a>10.  Custom Media Queries[](#custom-mq)

When designing documents that use media queries, the same media query may be used in multiple places, such as to qualify multiple <a id="ref-for-at-ruledef-import①"></a>[&#64;import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) statements. Repeating the same media query multiple times is an editing hazard; an author making a change must edit every copy in the same way, or suffer from difficult-to-find bugs in their CSS.

To help ameliorate this, this specification defines a method of defining <a id="ref-for-custom-media-query"></a>[custom media queries](#custom-media-query), which are simply-named aliases for longer and more complex media queries. In this way, a media query used in multiple places can instead be assigned to a <a id="ref-for-custom-media-query①"></a>custom media query, which can be used everywhere, and editing the media query requires touching only one line of code.

A <a id="custom-media-query"></a><strong>custom media query</strong> is defined with the <a id="ref-for-at-ruledef-custom-media"></a>[&#64;custom-media](#at-ruledef-custom-media) rule:

<a id="at-ruledef-custom-media"></a><a id="ref-for-typedef-extension-name"></a><a id="ref-for-typedef-media-query-list②"></a><a id="ref-for-comb-one③⓪"></a><a id="ref-for-comb-one③①"></a>

``` text
@custom-media = @custom-media <extension-name> [ <media-query-list> | true | false ] ;
```

The <a id="ref-for-typedef-extension-name①"></a>[\<extension-name\>](https://drafts.csswg.org/css-extensions-1/#typedef-extension-name) can then be used in a <a id="ref-for-media-feature③④"></a>[media feature](https://drafts.csswg.org/mediaqueries-5/#media-feature). It <strong>must</strong> be used in a <a id="ref-for-boolean-context⑤"></a>[boolean context](https://drafts.csswg.org/mediaqueries-5/#boolean-context); using them in a normal or <a id="ref-for-range-context⑤"></a>[range context](https://drafts.csswg.org/mediaqueries-5/#range-context) is a syntax error. If a <a id="ref-for-typedef-media-query-list③"></a>[\<media-query-list\>](https://drafts.csswg.org/mediaqueries-5/#typedef-media-query-list) is given, the <a id="ref-for-custom-media-query②"></a>[custom media query](#custom-media-query) evaluates to true if the <a id="ref-for-typedef-media-query-list④"></a>\<media-query-list\> it represents evaluates to true, and false otherwise. If <a id="valdef-custom-media-true"></a><strong>true</strong> or <a id="valdef-custom-media-false"></a><strong>false</strong> is given, the <a id="ref-for-custom-media-query③"></a>custom media query evaluates to true or false, respectively.

<a id="example-532b0adb"></a>

<strong>Example:</strong>

[](#example-532b0adb) The <a id="ref-for-custom-media-query④"></a>[custom media query](#custom-media-query) is evaluated logically, not treated as a textual substitution. Take the following code snippet for instance:

``` text
/* --modern targets modern devices that support color or hover */
@custom-media --modern (color), (hover);

@media (--modern) and (width > 1024px) {
  .a { color: green; }
}
```

It is equivalent to:

``` text
@media ((color) or (hover)) and (width > 1024px) {
  .a { color: green; }
}
```

Processing it as if it meant the following would be incorrect:

``` text
@media (color), (hover) and (width > 1024px) {
  .a { color: green; }
}
```

A <a id="ref-for-at-ruledef-custom-media①"></a>[&#64;custom-media](#at-ruledef-custom-media) rule can refer to other <a id="ref-for-custom-media-query⑤"></a>[custom media queries](#custom-media-query). However, loops are forbidden, and a <a id="ref-for-custom-media-query⑥"></a>custom media query must not be defined in terms of itself or of another <a id="ref-for-custom-media-query⑦"></a>custom media query that directly or indirectly refers to it. Any such attempt of defining a <a id="ref-for-custom-media-query⑧"></a>custom media query with a circular dependency must cause all the <a id="ref-for-custom-media-query⑨"></a>custom media queries in the loop to fail to be defined.

If multiple <a id="ref-for-at-ruledef-custom-media②"></a>[&#64;custom-media](#at-ruledef-custom-media) rules declare the same <a id="ref-for-typedef-extension-name②"></a>[\<extension-name\>](https://drafts.csswg.org/css-extensions-1/#typedef-extension-name), the truth value is based on the last one alone, ignoring all previous declarations of the same <a id="ref-for-typedef-extension-name③"></a>\<extension-name\>.

Note: For error handling purposes, an undefined <a id="ref-for-media-feature③⑤"></a>[media feature](https://drafts.csswg.org/mediaqueries-5/#media-feature) is different from a <a id="ref-for-media-feature③⑥"></a>media feature that evaluates to false. See [Media Queries 4 § 3.2 Error Handling](https://www.w3.org/TR/mediaqueries-4/#error-handling) for details.

<a id="example-c280b211"></a>

<strong>Example:</strong>

[](#example-c280b211) For example, if a responsive site uses a particular breakpoint in several places, it can alias that with a reasonable name:

``` text
@custom-media --narrow-window (max-width: 30em);

@media (--narrow-window) {
  /* narrow window styles */
}
@media (--narrow-window) and (script) {
  /* special styles for when script is allowed */
}
/* etc */
```

### <a id="script-custom-mq"></a>10.1.  Script-based Custom Media Queries[](#script-custom-mq)

<a id="issue-bab99524"></a>

<strong>Issue:</strong>

[](#issue-bab99524) Define a map of names to values for JS. Values can be either a MediaQueryList object or a boolean, in which case it’s treated identically to the above, or can be a number or a string, in which case it’s treated like a normal MQ, and can use the normal or range context syntax. Like:

``` text
<script>
CSS.customMedia.set('--foo', 5);
</script>
<style>
@media (_foo: 5) { ... }
@media (_foo < 10) { ... }
</style>
```

## <a id="custom-mq-cssom"></a>11.  CSSOM[](#custom-mq-cssom)

The <code><a id="ref-for-csscustommediarule"></a>[CSSCustomMediaRule](#csscustommediarule)</code> interface represents a <a id="ref-for-at-ruledef-custom-media③"></a>[&#64;custom-media](#at-ruledef-custom-media) rule.

<a id="ref-for-medialist"></a><a id="ref-for-idl-boolean"></a><a id="typedefdef-custommediaquery"></a><a id="ref-for-Exposed"></a><a id="csscustommediarule"></a><a id="ref-for-cssrule"></a><a id="ref-for-cssomstring"></a><a id="ref-for-dom-csscustommediarule-name"></a><a id="ref-for-typedefdef-custommediaquery"></a><a id="ref-for-dom-csscustommediarule-query"></a>

``` text
typedef (MediaList or boolean) CustomMediaQuery;

[Exposed=Window]
interface CSSCustomMediaRule : CSSRule {
  readonly attribute CSSOMString name;
  readonly attribute CustomMediaQuery query;
};
```

<strong><a id="dom-csscustommediarule-name"></a><strong><code>name</code></strong>, of type <a id="ref-for-cssomstring①"></a>[CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring), readonly</strong>

Returns a CSSOMString representing the <a id="ref-for-typedef-extension-name④"></a>[\<extension-name\>](https://drafts.csswg.org/css-extensions-1/#typedef-extension-name) of the <a id="ref-for-at-ruledef-custom-media④"></a>[&#64;custom-media](#at-ruledef-custom-media) rule.

<strong><a id="dom-csscustommediarule-query"></a><strong><code>query</code></strong>, of type <a id="ref-for-typedefdef-custommediaquery①"></a>[CustomMediaQuery](#typedefdef-custommediaquery), readonly</strong>

Represents the value of the <a id="ref-for-custom-media-query①⓪"></a>[custom media query](#custom-media-query). The returned <code><a id="ref-for-typedefdef-custommediaquery②"></a>[CustomMediaQuery](#typedefdef-custommediaquery)</code> will be one of the following:

- A <code><a id="ref-for-medialist①"></a>[MediaList](https://www.w3.org/TR/cssom-1/#medialist)</code> object, if the rule was defined with a <a id="ref-for-typedef-media-query-list⑤"></a>[\<media-query-list\>](https://drafts.csswg.org/mediaqueries-5/#typedef-media-query-list).
- The boolean true, if the rule was defined with the value <a id="ref-for-valdef-custom-media-true"></a>[true](#valdef-custom-media-true).
- The boolean false, if the rule was defined with the value <a id="ref-for-valdef-custom-media-false"></a>[false](#valdef-custom-media-false).
