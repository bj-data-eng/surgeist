# Counter function output: current Lists 3 clause

Retrieved 2026-10-12 from https://drafts.csswg.org/css-lists-3/. Source status: Editor’s Draft,
    28 April 2026. Declared revision: `['fbd84ec61ae093787ced21cd48d3a7536e04b50a']`. Full HTML SHA-256: `4715f87f35befe3340a0a83e0a178623e6b8e443e9e99835fcbffff79693c9a7`. Bounded input SHA-256: `3059ea0c1e88d40c391ae4d048b95b90dd4a4e440c1d25fd120e8da4554bad74`.

Retains complete section 4.7, header and original legal notice. Converted with existing Pandoc 3.1.11.1 and the repository's Python/lxml preparation and semantic filter. No tables occur in this bounded clause; examples, IDs, links and prose are checked through the existing rendered-GFM verifier. This supplements the retained Lists reference only for the bounded #707 output question; it does not replace unrelated source selections or adopt new authored grammar.

Attribution and reformatting notice added for Surgeist on 2026-10-12. Original English source remains authoritative. This is implementation support, not a new specification. Material uses the [W3C Software and Document License 2023](../licenses/w3c/software-license-2023.txt); W3C is not responsible for conversion errors. Original legal notice follows.

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Lists and Counters Module Level 3

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 28 April 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-lists-3/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-lists-3/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2020/WD-css-lists-3-20200709/>

<https://www.w3.org/TR/2019/WD-css-lists-3-20190817/>

<https://www.w3.org/TR/2019/WD-css-lists-3-20190425/>

<https://www.w3.org/TR/2014/WD-css-lists-3-20140320/>

<https://www.w3.org/TR/2011/WD-css3-lists-20110524/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-lists-3)

[Inline In Spec](https://drafts.csswg.org/css-lists-3/#issues-index)

<strong>Editors:</strong>

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Tab Atkins](http://xanthir.com/contact/) (Google)

<strong>Former Editors:</strong>

[Ian Hickson](mailto:ian@hixie.ch) (Google)

[Tantek Çelik](mailto:tantekc@microsoft.com) (Formerly of Microsoft)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-lists-3/Overview.bs)

<strong>Contributors:</strong>

Simon Montagu, AOL-TW/Netscape, [smontagu&#64;netscape.com](mailto:smontagu@netscape.com)

Daniel Yacob, [yacob&#64;geez.org](mailto:yacob@geez.org)

Christopher Hoess, [choess&#64;stwing.upenn.edu](mailto:choess@stwing.upenn.edu)

Daniel Glazman, AOL-TW/Netscape, [glazman&#64;netscape.com](mailto:glazman@netscape.com)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

### <a id="counter-functions"></a>4.7.  Outputting Counters: the <a id="ref-for-funcdef-counter④"></a>[counter()](#funcdef-counter) and <a id="ref-for-funcdef-counters④"></a>[counters()](#funcdef-counters) functions[](#counter-functions)

<a id="ref-for-counter②⑤"></a>[Counters](https://drafts.csswg.org/css-lists-3/#counter) have no visible effect by themselves, but their values can be used with the <a id="funcdef-counter"></a><strong>counter()</strong> and <a id="funcdef-counters"></a><strong>counters()</strong> functions, whose <a id="ref-for-used-value"></a>[used values](https://drafts.csswg.org/css-cascade-5/#used-value) represent counter values as strings or images. They are defined as follows:

<a id="typedef-counter"></a><a id="ref-for-typedef-counter"></a><a id="ref-for-funcdef-counter⑤"></a><a id="ref-for-comb-one⑨"></a><a id="ref-for-funcdef-counters⑤"></a><a id="ref-for-funcdef-counter⑥"></a><a id="ref-for-typedef-counter-name①③"></a><a id="ref-for-comb-comma"></a><a id="ref-for-typedef-counter-style⑧"></a><a id="ref-for-mult-opt④"></a><a id="ref-for-funcdef-counters⑥"></a><a id="ref-for-typedef-counter-name①④"></a><a id="ref-for-comb-comma①"></a><a id="ref-for-string-value③"></a><a id="ref-for-comb-comma②"></a><a id="ref-for-typedef-counter-style⑨"></a><a id="ref-for-mult-opt⑤"></a>

``` text
<counter> = <counter()> | <counters()>
<counter()>  =  counter( <counter-name>, <counter-style>? )
<counters()> = counters( <counter-name>, <string>, <counter-style>? )
```

where <a id="ref-for-typedef-counter-style①⓪"></a>[\<counter-style\>](https://drafts.csswg.org/css-counter-styles-3/#typedef-counter-style) specifies the <a id="ref-for-counter-style"></a>[counter style](https://drafts.csswg.org/css-counter-styles-3/#counter-style) for <a id="ref-for-generate-a-counter①"></a>[generating a representation](https://drafts.csswg.org/css-counter-styles-3/#generate-a-counter) of the named counter(s) as defined in [\[css-counter-styles-3\]](https://drafts.csswg.org/css-lists-3/#biblio-css-counter-styles-3) and

<strong>counter()</strong>

Represents the value of the <a id="ref-for-innermost①"></a>[innermost](https://drafts.csswg.org/css-lists-3/#innermost) <a id="ref-for-counter②⑥"></a>[counter](https://drafts.csswg.org/css-lists-3/#counter) in the element’s <a id="ref-for-css-counters-set⑤"></a>[CSS counters set](https://drafts.csswg.org/css-lists-3/#css-counters-set) named <a id="ref-for-typedef-counter-name①⑤"></a>[\<counter-name\>](https://drafts.csswg.org/css-lists-3/#typedef-counter-name) using the <a id="ref-for-counter-style①"></a>[counter style](https://drafts.csswg.org/css-counter-styles-3/#counter-style) named <a id="ref-for-typedef-counter-style①①"></a>[\<counter-style\>](https://drafts.csswg.org/css-counter-styles-3/#typedef-counter-style).

<strong>counters()</strong>

Represents the values of all the <a id="ref-for-counter②⑦"></a>[counters](https://drafts.csswg.org/css-lists-3/#counter) in the element’s <a id="ref-for-css-counters-set⑥"></a>[CSS counters set](https://drafts.csswg.org/css-lists-3/#css-counters-set) named <a id="ref-for-typedef-counter-name①⑥"></a>[\<counter-name\>](https://drafts.csswg.org/css-lists-3/#typedef-counter-name) using the <a id="ref-for-counter-style②"></a>[counter style](https://drafts.csswg.org/css-counter-styles-3/#counter-style) named <a id="ref-for-typedef-counter-style①②"></a>[\<counter-style\>](https://drafts.csswg.org/css-counter-styles-3/#typedef-counter-style), sorted in outermost-first to <a id="ref-for-innermost②"></a>[innermost](https://drafts.csswg.org/css-lists-3/#innermost)-last order and joined by the specified <a id="ref-for-string-value④"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value).

In both cases, if the <a id="ref-for-typedef-counter-style①③"></a>[\<counter-style\>](https://drafts.csswg.org/css-counter-styles-3/#typedef-counter-style) argument is omitted it defaults to <a id="ref-for-decimal①"></a>[decimal](https://drafts.csswg.org/css-counter-styles-3/#decimal).

Note: If the <a id="ref-for-typedef-counter-style①④"></a>[\<counter-style\>](https://drafts.csswg.org/css-counter-styles-3/#typedef-counter-style) is one of the predefined symbolic styles, like <a id="ref-for-disc"></a>[disc](https://drafts.csswg.org/css-counter-styles-3/#disc), it might look different than when used in <a id="ref-for-propdef-list-style-type⑨"></a>[list-style-type](https://drafts.csswg.org/css-lists-3/#propdef-list-style-type). See [CSS Counter Styles 3 § 6.3 Symbolic: disc, circle, square, disclosure-open, disclosure-closed](https://drafts.csswg.org/css-counter-styles-3/#simple-symbolic).

If no <a id="ref-for-counter②⑧"></a>[counter](https://drafts.csswg.org/css-lists-3/#counter) named <a id="ref-for-typedef-counter-name①⑦"></a>[\<counter-name\>](https://drafts.csswg.org/css-lists-3/#typedef-counter-name) exists on an element where <a id="ref-for-funcdef-counter⑦"></a>[counter()](#funcdef-counter) or <a id="ref-for-funcdef-counters⑦"></a>[counters()](#funcdef-counters) is used, one is first <a id="ref-for-instantiate-counter①②"></a>[instantiated](https://drafts.csswg.org/css-lists-3/#instantiate-counter) with a starting value of 0.

<a id="example-a273a37d"></a>

<strong>Example:</strong>

[](#example-a273a37d)

``` text
H1::before        { content: counter(chno, upper-latin) ". " }
/* Generates headings like "A. A History of Discontent" */

H2::before        { content: counter(section, upper-roman) " - " }
/* Generates headings like "II - The Discontent Part" */

BLOCKQUOTE::after { content: " [" counter(bq, decimal) "]" }
/* Generates blockquotes that end like "... [3]" */

DIV.note::before  { content: counter(notecntr, disc) " " }
/* Simply generates a bullet before every div.note */

P::before         { content: counter(p, none) }
/* inserts nothing */
```

<a id="example-e8a9db02"></a>

<strong>Example:</strong>

[](#example-e8a9db02) The following example shows a simple use of the <a id="ref-for-funcdef-counters⑧"></a>[counters()](#funcdef-counters) function:

``` text
<ul>
  <li>one</li>
  <li>two
    <ul>
      <li>nested one</li>
      <li>nested two</li>
    </ul>
  </li>
  <li>three</li>
</ul>
<style>
li::marker { content: '(' counters(list-item,'.') ') '; }
</style>
```

The preceding document should render something like this:

``` text
(1) one
(2) two
   (2.1) nested one
   (2.2) nested two
(3) three
```

<a id="example-b1a4542d"></a>

<strong>Example:</strong>

[](#example-b1a4542d) Because counters inherit to siblings as well, they can be used to number headings and subheadings, which aren’t nested within each other. Unfortunately, this prevents the use of <a id="ref-for-funcdef-counters⑨"></a>[counters()](#funcdef-counters) as counters from siblings don’t nest, but one can create multiple counters and manually concatenate them instead:

``` text
<h1>First H1</h1>
...
<h2>First H2 in H1</h2>
...
<h2>Second H2 in H1</h2>
...
<h3>First H3 in H2</h3>
...
<h1>Second H1</h1>
...
<h2>First H2 in H1</h2>
...
<style>
body { counter-reset: h1 h2 h3; }
h1   { counter-increment: h1; counter-reset: h2 h3;}
h2   { counter-increment: h2; counter-reset:    h3; }
h3   { counter-increment: h3; }
h1::before { content: counter(h1,upper-alpha) ' '; }
h2::before { content: counter(h1,upper-alpha) '.'
                      counter(h2,decimal) ' '; }
h3::before { content: counter(h1,upper-alpha) '.'
                      counter(h2,decimal) '.'
                      counter(h3,lower-roman) ' '; }
</style>
```

The preceding document should render something like this:

``` text
A First H1
...
A.1 First H2 in H1
...
A.2 Second H2 in H1
...
A.2.i First H3 in H2
...
B Second H1
...
B.1 First H2 in H1
...
```

<a id="issue-0f77386b"></a>

<strong>Issue:</strong>

[](#issue-0f77386b) Counters are sometimes useful for things other than printing markers. In general, they provide the ability to number elements in sequence, which can be useful for other properties to reference. For example, using <a id="ref-for-propdef-order"></a>[order](https://drafts.csswg.org/css-display-4/#propdef-order) to put an element between two other specific elements currently requires you to explicitly put <a id="ref-for-propdef-order①"></a>order on every element before and/or after the desired insertion point. If you can set the <a id="ref-for-propdef-order②"></a>order value of everything to a counter though, you can more easily insert an element into an arbitrary spot between two others.

Other use-cases involve nested or sibling elements with transforms that are meant to be slightly different from each other. Today you have to use a preprocessor to do this in a reasonable way, but a counter would make it work well in "plain" CSS.

(You can built up successive values in the nested case today by using <a id="ref-for-custom-property"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) and stacking up nested <a id="ref-for-funcdef-calc"></a>[calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc)s, but this is a \*little bit\* clumsy, and doesn’t work for siblings.)

Suggestion is to add a counter-value(<a id="ref-for-typedef-counter-name①⑧"></a>[\<counter-name\>](https://drafts.csswg.org/css-lists-3/#typedef-counter-name)) function, which returns the value of the named counter as an integer, rather than returning a string.

See [Issue 1026](https://github.com/w3c/csswg-drafts/issues/1026).
