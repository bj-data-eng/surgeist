# HTML Standard: microdata encoding

Source: [immutable HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#encoding-microdata), revision `877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026); complete original HTML retrieved 11 October 2026, 14,930,614 bytes, SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. This bounded offline extraction was prepared on 13 October 2026 from those same retained bytes; no new edition was acquired.

Complete section 5.2, including the model, item/type/identifier/reference relations, property names/values, crawl and document obligations, and namespace. Vocabulary examples in section 5.3 are outside this capture. The extraction starts at `encoding-microdata` and ends before the next equal-or-higher heading `mdvocabs`. Normalized extraction input: 42,687 bytes, SHA-256 `35d3d012166bfb6f977e81a154a28259467433df799b1647e3c9f7ef19abb2f0`.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is [Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/), with the [exact WHATWG license](../licenses/whatwg/LICENSE); BSD 3-Clause applies to portions incorporated into code, not as an alternative prose license. Bounded extraction and Markdown reformatting by Surgeist use the existing preparation/filter, approved Pandoc 3.1.11.1 and parsed-GFM comparison. Source text, IDs, links, literal examples, variables, headings and tables are preserved; MDN compatibility annotations and source scripts/styles are omitted without execution. No semantic change, endorsement, browser capability or implementation-conformance claim is implied. The [edition map](SOURCE-EDITIONS.md#remaining-intrinsic-html-source-clauses) records the owning scope.

---

<!-- captured-body-start -->

### <a id="encoding&#45;microdata"></a>5&#46;2 Encoding microdata[](#encoding-microdata)

#### <a id="the&#45;microdata&#45;model"></a>5&#46;2&#46;1 The microdata model[](#the-microdata-model)

The microdata model consists of groups of name&#45;value pairs known as <a id="the&#45;microdata&#45;model&#58;concept&#45;item"></a>[items](#concept-item)&#46;

Each group is known as an <a id="the&#45;microdata&#45;model&#58;concept&#45;item&#45;2"></a>[item](#concept-item)&#46; Each <a id="the&#45;microdata&#45;model&#58;concept&#45;item&#45;3"></a>[item](#concept-item) can have <a id="the&#45;microdata&#45;model&#58;item&#45;types"></a>[item types](#item-types)&#44; a <a id="the&#45;microdata&#45;model&#58;global&#45;identifier"></a>[global identifier](#global-identifier) &#40;if the vocabulary specified by the <a id="the&#45;microdata&#45;model&#58;item&#45;types&#45;2"></a>[item types](#item-types) <a id="the&#45;microdata&#45;model&#58;support&#45;global&#45;identifiers&#45;for&#45;items"></a>[support global identifiers for items](#support-global-identifiers-for-items)&#41;&#44; and a list of name&#45;value pairs&#46; Each name in the name&#45;value pair is known as a <a id="the&#45;microdata&#45;model&#58;the&#45;properties&#45;of&#45;an&#45;item"></a>[property](#the-properties-of-an-item)&#44; and each <a id="the&#45;microdata&#45;model&#58;the&#45;properties&#45;of&#45;an&#45;item&#45;2"></a>[property](#the-properties-of-an-item) has one or more <a id="the&#45;microdata&#45;model&#58;concept&#45;property&#45;value"></a>[values](#concept-property-value)&#46; Each <a id="the&#45;microdata&#45;model&#58;concept&#45;property&#45;value&#45;2"></a>[value](#concept-property-value) is either a string or itself a group of name&#45;value pairs &#40;an <a id="the&#45;microdata&#45;model&#58;concept&#45;item&#45;4"></a>[item](#concept-item)&#41;&#46; The names are unordered relative to each other&#44; but if a particular name has multiple values&#44; they do have a relative order&#46;

#### <a id="items"></a>5&#46;2&#46;2 Items[](#items)

Every <a id="items&#58;html&#45;elements"></a>[HTML element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#html-elements) may have an <a id="attr&#45;itemscope"></a><strong><code>itemscope</code></strong> attribute specified&#46; The <a id="items&#58;attr&#45;itemscope"></a><code>[itemscope](#attr-itemscope)</code> attribute is a <a id="items&#58;boolean&#45;attribute"></a>[boolean attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute)&#46;

An element with the <a id="items&#58;attr&#45;itemscope&#45;2"></a><code>[itemscope](#attr-itemscope)</code> attribute specified creates a new <a id="concept&#45;item"></a><strong>item</strong>&#44; a group of name&#45;value pairs&#46;

------------------------------------------------------------------------

Elements with an <a id="items&#58;attr&#45;itemscope&#45;3"></a><code>[itemscope](#attr-itemscope)</code> attribute may have an <a id="attr&#45;itemtype"></a><strong><code>itemtype</code></strong> attribute specified&#44; to give the <a id="items&#58;item&#45;types"></a>[item types](#item-types) of the <a id="items&#58;concept&#45;item"></a>[item](#concept-item)&#46;

The <a id="items&#58;attr&#45;itemtype"></a><code>[itemtype](#attr-itemtype)</code> attribute&#44; if specified&#44; must have a value that is an <a id="items&#58;unordered&#45;set&#45;of&#45;unique&#45;space&#45;separated&#45;tokens"></a>[unordered set of unique space&#45;separated tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens)&#44; none of which are <a id="items&#58;identical&#45;to"></a>[identical to](https://infra.spec.whatwg.org/#string-is) another token and each of which is a <a id="items&#58;valid&#45;url&#45;string"></a>[valid URL string](https://url.spec.whatwg.org/#valid-url-string) that is an <a id="items&#58;absolute&#45;url"></a>[absolute URL](https://url.spec.whatwg.org/#syntax-url-absolute)&#44; and all of which are defined to use the same vocabulary&#46; The attribute&#39;s value must have at least one token&#46;

The <a id="item&#45;types"></a><strong>item types</strong> of an <a id="items&#58;concept&#45;item&#45;2"></a>[item](#concept-item) are the tokens obtained by <a id="items&#58;split&#45;a&#45;string&#45;on&#45;spaces"></a>[splitting the element&#39;s <code>itemtype</code> attribute&#39;s value on ASCII whitespace](https://infra.spec.whatwg.org/#split-on-ascii-whitespace)&#46; If the <a id="items&#58;attr&#45;itemtype&#45;2"></a><code>[itemtype](#attr-itemtype)</code> attribute is missing or parsing it in this way finds no tokens&#44; the <a id="items&#58;concept&#45;item&#45;3"></a>[item](#concept-item) is said to have no <a id="items&#58;item&#45;types&#45;2"></a>[item types](#item-types)&#46;

The <a id="items&#58;item&#45;types&#45;3"></a>[item types](#item-types) must all be types defined in <a id="items&#58;other&#45;applicable&#45;specifications"></a>[applicable specifications](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#other-applicable-specifications) and must all be defined to use the same vocabulary&#46;

Except if otherwise specified by that specification&#44; the <a id="items&#58;url"></a>[URLs](https://url.spec.whatwg.org/#concept-url) given as the <a id="items&#58;item&#45;types&#45;4"></a>[item types](#item-types) should not be automatically dereferenced&#46;

<strong>Note&#58;</strong>

A specification could define that its <a id="items&#58;item&#45;types&#45;5"></a>[item type](#item-types) can be dereferenced to provide the user with help information&#44; for example&#46; In fact&#44; vocabulary authors are encouraged to provide useful information at the given <a id="items&#58;url&#45;2"></a>[URL](https://url.spec.whatwg.org/#concept-url)&#46;

<a id="items&#58;item&#45;types&#45;6"></a>[Item types](#item-types) are opaque identifiers&#44; and user agents must not dereference unknown <a id="items&#58;item&#45;types&#45;7"></a>[item types](#item-types)&#44; or otherwise deconstruct them&#44; in order to determine how to process <a id="items&#58;concept&#45;item&#45;4"></a>[items](#concept-item) that use them&#46;

The <a id="items&#58;attr&#45;itemtype&#45;3"></a><code>[itemtype](#attr-itemtype)</code> attribute must not be specified on elements that do not have an <a id="items&#58;attr&#45;itemscope&#45;4"></a><code>[itemscope](#attr-itemscope)</code> attribute specified&#46;

------------------------------------------------------------------------

An <a id="items&#58;concept&#45;item&#45;5"></a>[item](#concept-item) is said to be a <a id="typed&#45;item"></a><strong>typed item</strong> when either it has an <a id="items&#58;item&#45;types&#45;8"></a>[item type](#item-types)&#44; or it is the <a id="items&#58;concept&#45;property&#45;value"></a>[value](#concept-property-value) of a <a id="items&#58;the&#45;properties&#45;of&#45;an&#45;item"></a>[property](#the-properties-of-an-item) of a <a id="items&#58;typed&#45;item"></a>[typed item](#typed-item)&#46; The <a id="relevant&#45;types"></a><strong>relevant types</strong> for a <a id="items&#58;typed&#45;item&#45;2"></a>[typed item](#typed-item) is the <a id="items&#58;concept&#45;item&#45;6"></a>[item](#concept-item)&#39;s <a id="items&#58;item&#45;types&#45;9"></a>[item types](#item-types)&#44; if it has any&#44; or else is the <a id="items&#58;relevant&#45;types"></a>[relevant types](#relevant-types) of the <a id="items&#58;concept&#45;item&#45;7"></a>[item](#concept-item) for which it is a <a id="items&#58;the&#45;properties&#45;of&#45;an&#45;item&#45;2"></a>[property](#the-properties-of-an-item)&#39;s <a id="items&#58;concept&#45;property&#45;value&#45;2"></a>[value](#concept-property-value)&#46;

------------------------------------------------------------------------

Elements with an <a id="items&#58;attr&#45;itemscope&#45;5"></a><code>[itemscope](#attr-itemscope)</code> attribute and an <a id="items&#58;attr&#45;itemtype&#45;4"></a><code>[itemtype](#attr-itemtype)</code> attribute that references a vocabulary that is defined to <a id="support&#45;global&#45;identifiers&#45;for&#45;items"></a><strong>support global identifiers for items</strong> may also have an <a id="attr&#45;itemid"></a><strong><code>itemid</code></strong> attribute specified&#44; to give a global identifier for the <a id="items&#58;concept&#45;item&#45;8"></a>[item](#concept-item)&#44; so that it can be related to other <a id="items&#58;concept&#45;item&#45;9"></a>[items](#concept-item) on pages elsewhere on the web&#46;

The <a id="items&#58;attr&#45;itemid"></a><code>[itemid](#attr-itemid)</code> attribute&#44; if specified&#44; must have a value that is a <a id="items&#58;valid&#45;url&#45;potentially&#45;surrounded&#45;by&#45;spaces"></a>[valid URL potentially surrounded by spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-url-potentially-surrounded-by-spaces)&#46;

The <a id="global&#45;identifier"></a><strong>global identifier</strong> of an <a id="items&#58;concept&#45;item&#45;10"></a>[item](#concept-item) is the value of its element&#39;s <a id="items&#58;attr&#45;itemid&#45;2"></a><code>[itemid](#attr-itemid)</code> attribute&#44; if it has one&#44; <a id="items&#58;encoding&#45;parsing&#45;a&#45;url"></a>[parsed](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#encoding-parsing-a-url) relative to the <a id="items&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) of the element on which the attribute is specified&#46; If the <a id="items&#58;attr&#45;itemid&#45;3"></a><code>[itemid](#attr-itemid)</code> attribute is missing or if parsing it returns failure&#44; it is said to have no <a id="items&#58;global&#45;identifier"></a>[global identifier](#global-identifier)&#46;

The <a id="items&#58;attr&#45;itemid&#45;4"></a><code>[itemid](#attr-itemid)</code> attribute must not be specified on elements that do not have both an <a id="items&#58;attr&#45;itemscope&#45;6"></a><code>[itemscope](#attr-itemscope)</code> attribute and an <a id="items&#58;attr&#45;itemtype&#45;5"></a><code>[itemtype](#attr-itemtype)</code> attribute specified&#44; and must not be specified on elements with an <a id="items&#58;attr&#45;itemscope&#45;7"></a><code>[itemscope](#attr-itemscope)</code> attribute whose <a id="items&#58;attr&#45;itemtype&#45;6"></a><code>[itemtype](#attr-itemtype)</code> attribute specifies a vocabulary that does not <a id="items&#58;support&#45;global&#45;identifiers&#45;for&#45;items"></a>[support global identifiers for items](#support-global-identifiers-for-items)&#44; as defined by that vocabulary&#39;s specification&#46;

The exact meaning of a <a id="items&#58;global&#45;identifier&#45;2"></a>[global identifier](#global-identifier) is determined by the vocabulary&#39;s specification&#46; It is up to such specifications to define whether multiple items with the same global identifier &#40;whether on the same page or on different pages&#41; are allowed to exist&#44; and what the processing rules for that vocabulary are with respect to handling the case of multiple items with the same ID&#46;

------------------------------------------------------------------------

Elements with an <a id="items&#58;attr&#45;itemscope&#45;8"></a><code>[itemscope](#attr-itemscope)</code> attribute may have an <a id="attr&#45;itemref"></a><strong><code>itemref</code></strong> attribute specified&#44; to give a list of additional elements to crawl to find the name&#45;value pairs of the <a id="items&#58;concept&#45;item&#45;11"></a>[item](#concept-item)&#46;

The <a id="items&#58;attr&#45;itemref"></a><code>[itemref](#attr-itemref)</code> attribute&#44; if specified&#44; must have a value that is an <a id="items&#58;unordered&#45;set&#45;of&#45;unique&#45;space&#45;separated&#45;tokens&#45;2"></a>[unordered set of unique space&#45;separated tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens) none of which are <a id="items&#58;identical&#45;to&#45;2"></a>[identical to](https://infra.spec.whatwg.org/#string-is) another token and consisting of <a id="items&#58;concept&#45;id"></a>[IDs](https://dom.spec.whatwg.org/#concept-id) of elements in the same <a id="items&#58;tree"></a>[tree](https://dom.spec.whatwg.org/#concept-tree)&#46;

The <a id="items&#58;attr&#45;itemref&#45;2"></a><code>[itemref](#attr-itemref)</code> attribute must not be specified on elements that do not have an <a id="items&#58;attr&#45;itemscope&#45;9"></a><code>[itemscope](#attr-itemscope)</code> attribute specified&#46;

<strong>Note&#58;</strong>

The <a id="items&#58;attr&#45;itemref&#45;3"></a><code>[itemref](#attr-itemref)</code> attribute is not part of the microdata data model&#46; It is merely a syntactic construct to aid authors in adding annotations to pages where the data to be annotated does not follow a convenient tree structure&#46; For example&#44; it allows authors to mark up data in a table so that each column defines a separate <a id="items&#58;concept&#45;item&#45;12"></a>[item](#concept-item)&#44; while keeping the properties in the cells&#46;

<strong>Example&#58;</strong>

This example shows a simple vocabulary used to describe the products of a model railway manufacturer&#46; The vocabulary has just five property names&#58;

<strong>product&#45;code</strong>

An integer that names the product in the manufacturer&#39;s catalog&#46;

<strong>name</strong>

A brief description of the product&#46;

<strong>scale</strong>

One of &#34;HO&#34;&#44; &#34;1&#34;&#44; or &#34;Z&#34; &#40;potentially with leading or trailing whitespace&#41;&#44; indicating the scale of the product&#46;

<strong>digital</strong>

If present&#44; one of &#34;Digital&#34;&#44; &#34;Delta&#34;&#44; or &#34;Systems&#34; &#40;potentially with leading or trailing whitespace&#41; indicating that the product has a digital decoder of the given type&#46;

<strong>track&#45;type</strong>

For track&#45;specific products&#44; one of &#34;K&#34;&#44; &#34;M&#34;&#44; &#34;C&#34; &#40;potentially with leading or trailing whitespace&#41; indicating the type of track for which the product is intended&#46;

This vocabulary has four defined <a id="items&#58;item&#45;types&#45;10"></a>[item types](#item-types)&#58;

<strong>https&#58;&#47;&#47;md&#46;example&#46;com&#47;loco</strong>

Rolling stock with an engine

<strong>https&#58;&#47;&#47;md&#46;example&#46;com&#47;passengers</strong>

Passenger rolling stock

<strong>https&#58;&#47;&#47;md&#46;example&#46;com&#47;track</strong>

Track pieces

<strong>https&#58;&#47;&#47;md&#46;example&#46;com&#47;lighting</strong>

Equipment with lighting

Each <a id="items&#58;concept&#45;item&#45;13"></a>[item](#concept-item) that uses this vocabulary can be given one or more of these types&#44; depending on what the product is&#46;

Thus&#44; a locomotive might be marked up as&#58;

``` text
<dl itemscope itemtype="https://md.example.com/loco
                        https://md.example.com/lighting">
 <dt>Name:
 <dd itemprop="name">Tank Locomotive (DB 80)
 <dt>Product code:
 <dd itemprop="product-code">33041
 <dt>Scale:
 <dd itemprop="scale">HO
 <dt>Digital:
 <dd itemprop="digital">Delta
</dl>
```

A turnout lantern retrofit kit might be marked up as&#58;

``` text
<dl itemscope itemtype="https://md.example.com/track
                        https://md.example.com/lighting">
 <dt>Name:
 <dd itemprop="name">Turnout Lantern Kit
 <dt>Product code:
 <dd itemprop="product-code">74470
 <dt>Purpose:
 <dd>For retrofitting 2 <span itemprop="track-type">C</span> Track
 turnouts. <meta itemprop="scale" content="HO">
</dl>
```

A passenger car with no lighting might be marked up as&#58;

``` text
<dl itemscope itemtype="https://md.example.com/passengers">
 <dt>Name:
 <dd itemprop="name">Express Train Passenger Car (DB Am 203)
 <dt>Product code:
 <dd itemprop="product-code">8710
 <dt>Scale:
 <dd itemprop="scale">Z
</dl>
```

Great care is necessary when creating new vocabularies&#46; Often&#44; a hierarchical approach to types can be taken that results in a vocabulary where each item only ever has a single type&#44; which is generally much simpler to manage&#46;

#### <a id="names&#58;&#45;the&#45;itemprop&#45;attribute"></a>5&#46;2&#46;3 Names&#58; the <strong><code>itemprop</code></strong> attribute[](#names:-the-itemprop-attribute)

Every <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;html&#45;elements"></a>[HTML element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#html-elements) may have an <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;names&#58;&#45;the&#45;itemprop&#45;attribute"></a><code>[itemprop](#names:-the-itemprop-attribute)</code> attribute specified&#44; if doing so <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;the&#45;properties&#45;of&#45;an&#45;item"></a>[adds one or more properties](#the-properties-of-an-item) to one or more <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;concept&#45;item"></a>[items](#concept-item) &#40;as defined below&#41;&#46;

The <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;names&#58;&#45;the&#45;itemprop&#45;attribute&#45;2"></a><code>[itemprop](#names:-the-itemprop-attribute)</code> attribute&#44; if specified&#44; must have a value that is an <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;unordered&#45;set&#45;of&#45;unique&#45;space&#45;separated&#45;tokens"></a>[unordered set of unique space&#45;separated tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens) none of which are <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;identical&#45;to"></a>[identical to](https://infra.spec.whatwg.org/#string-is) another token&#44; representing the names of the name&#45;value pairs that it adds&#46; The attribute&#39;s value must have at least one token&#46;

Each token must be either&#58;

- If the item is a <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;typed&#45;item"></a>[typed item](#typed-item)&#58; a <a id="defined&#45;property&#45;name"></a><strong>defined property name</strong> allowed in this situation according to the specification that defines the <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;relevant&#45;types"></a>[relevant types](#relevant-types) for the item&#44; or
- A <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;valid&#45;url&#45;string"></a>[valid URL string](https://url.spec.whatwg.org/#valid-url-string) that is an <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;absolute&#45;url"></a>[absolute URL](https://url.spec.whatwg.org/#syntax-url-absolute) defined as an item property name allowed in this situation by a vocabulary specification&#44; or
- A <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;valid&#45;url&#45;string&#45;2"></a>[valid URL string](https://url.spec.whatwg.org/#valid-url-string) that is an <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;absolute&#45;url&#45;2"></a>[absolute URL](https://url.spec.whatwg.org/#syntax-url-absolute)&#44; used as a proprietary item property name &#40;i&#46;e&#46; one used by the author for private purposes&#44; not defined in a public specification&#41;&#44; or
- If the item is not a <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;typed&#45;item&#45;2"></a>[typed item](#typed-item)&#58; a string that contains no U&#43;002E FULL STOP characters &#40;&#46;&#41; and no U&#43;003A COLON characters &#40;&#58;&#41;&#44; used as a proprietary item property name &#40;i&#46;e&#46; one used by the author for private purposes&#44; not defined in a public specification&#41;&#46;

Specifications that introduce <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;defined&#45;property&#45;name"></a>[defined property names](#defined-property-name) must ensure all such property names contain no U&#43;002E FULL STOP characters &#40;&#46;&#41;&#44; no U&#43;003A COLON characters &#40;&#58;&#41;&#44; and no <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;space&#45;characters"></a>[ASCII whitespace](https://infra.spec.whatwg.org/#ascii-whitespace)&#46;

<strong>Note&#58;</strong>

The rules above disallow U&#43;003A COLON characters &#40;&#58;&#41; in non&#45;URL values because otherwise they could not be distinguished from URLs&#46; Values with U&#43;002E FULL STOP characters &#40;&#46;&#41; are reserved for future extensions&#46; <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;space&#45;characters&#45;2"></a>[ASCII whitespace](https://infra.spec.whatwg.org/#ascii-whitespace) are disallowed because otherwise the values would be parsed as multiple tokens&#46;

When an element with an <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;names&#58;&#45;the&#45;itemprop&#45;attribute&#45;3"></a><code>[itemprop](#names:-the-itemprop-attribute)</code> attribute <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;the&#45;properties&#45;of&#45;an&#45;item&#45;2"></a>[adds a property](#the-properties-of-an-item) to multiple <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;concept&#45;item&#45;2"></a>[items](#concept-item)&#44; the requirement above regarding the tokens applies for each <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;concept&#45;item&#45;3"></a>[item](#concept-item) individually&#46;

The <a id="property&#45;names"></a><strong>property names</strong> of an element are the tokens that the element&#39;s <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;names&#58;&#45;the&#45;itemprop&#45;attribute&#45;4"></a><code>[itemprop](#names:-the-itemprop-attribute)</code> attribute is found to contain when its value is <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;split&#45;a&#45;string&#45;on&#45;spaces"></a>[split on ASCII whitespace](https://infra.spec.whatwg.org/#split-on-ascii-whitespace)&#44; with the order preserved but with duplicates removed &#40;leaving only the first occurrence of each name&#41;&#46;

Within an <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;concept&#45;item&#45;4"></a>[item](#concept-item)&#44; the properties are unordered with respect to each other&#44; except for properties with the same name&#44; which are ordered in the order they are given by the algorithm that defines <a id="names&#58;&#45;the&#45;itemprop&#45;attribute&#58;the&#45;properties&#45;of&#45;an&#45;item&#45;3"></a>[the properties of an item](#the-properties-of-an-item)&#46;

<strong>Example&#58;</strong>

In the following example&#44; the &#34;a&#34; property has the values &#34;1&#34; and &#34;2&#34;&#44; <em>in that order</em>&#44; but whether the &#34;a&#34; property comes before the &#34;b&#34; property or not is not important&#58;

``` text
<div itemscope>
 <p itemprop="a">1</p>
 <p itemprop="a">2</p>
 <p itemprop="b">test</p>
</div>
```

Thus&#44; the following is equivalent&#58;

``` text
<div itemscope>
 <p itemprop="b">test</p>
 <p itemprop="a">1</p>
 <p itemprop="a">2</p>
</div>
```

As is the following&#58;

``` text
<div itemscope>
 <p itemprop="a">1</p>
 <p itemprop="b">test</p>
 <p itemprop="a">2</p>
</div>
```

And the following&#58;

``` text
<div id="x">
 <p itemprop="a">1</p>
</div>
<div itemscope itemref="x">
 <p itemprop="b">test</p>
 <p itemprop="a">2</p>
</div>
```

#### <a id="values"></a>5&#46;2&#46;4 Values[](#values)

The <a id="concept&#45;property&#45;value"></a><strong>property value</strong> of a name&#45;value pair added by an element with an <a id="values&#58;names&#58;&#45;the&#45;itemprop&#45;attribute"></a><code>[itemprop](#names:-the-itemprop-attribute)</code> attribute is as given for the first matching case in the following list&#58;

<strong>If the element also has an <a id="values&#58;attr&#45;itemscope"></a><code>[itemscope](#attr-itemscope)</code> attribute</strong>

The value is the <a id="values&#58;concept&#45;item"></a>[item](#concept-item) created by the element&#46;

<strong>If the element is a <a id="values&#58;the&#45;meta&#45;element"></a><code>[meta](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-meta-element)</code> element</strong>

The value is the value of the element&#39;s <a id="values&#58;attr&#45;meta&#45;content"></a><code>[content](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-content)</code> attribute&#44; if any&#44; or the empty string if there is no such attribute&#46;

<strong>If the element is an <a id="values&#58;the&#45;audio&#45;element"></a><code>[audio](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-audio-element)</code>&#44; <a id="values&#58;the&#45;embed&#45;element"></a><code>[embed](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-embed-element)</code>&#44; <a id="values&#58;the&#45;iframe&#45;element"></a><code>[iframe](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element)</code>&#44; <a id="values&#58;the&#45;img&#45;element"></a><code>[img](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-img-element)</code>&#44; <a id="values&#58;the&#45;source&#45;element"></a><code>[source](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-source-element)</code>&#44; <a id="values&#58;the&#45;track&#45;element"></a><code>[track](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-track-element)</code>&#44; or <a id="values&#58;the&#45;video&#45;element"></a><code>[video](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-video-element)</code> element</strong>

The value is the result of <a id="values&#58;encoding&#45;parsing&#45;and&#45;serializing&#45;a&#45;url"></a>[encoding&#45;parsing&#45;and&#45;serializing a URL](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#encoding-parsing-and-serializing-a-url) given the element&#39;s <code>src</code> attribute&#39;s value&#44; relative to the element&#39;s <a id="values&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#44; at the time the attribute is set&#44; or the empty string if there is no such attribute or the result is failure&#46;

<strong>If the element is an <a id="values&#58;the&#45;a&#45;element"></a><code>[a](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-a-element)</code>&#44; <a id="values&#58;the&#45;area&#45;element"></a><code>[area](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-area-element)</code>&#44; or <a id="values&#58;the&#45;link&#45;element"></a><code>[link](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-link-element)</code> element</strong>

The value is the result of <a id="values&#58;encoding&#45;parsing&#45;and&#45;serializing&#45;a&#45;url&#45;2"></a>[encoding&#45;parsing&#45;and&#45;serializing a URL](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#encoding-parsing-and-serializing-a-url) given the element&#39;s <code>href</code> attribute&#39;s value&#44; relative to the element&#39;s <a id="values&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#44; at the time the attribute is set&#44; or the empty string if there is no such attribute or the result is failure&#46;

<strong>If the element is an <a id="values&#58;the&#45;object&#45;element"></a><code>[object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-object-element)</code> element</strong>

The value is the result of <a id="values&#58;encoding&#45;parsing&#45;and&#45;serializing&#45;a&#45;url&#45;3"></a>[encoding&#45;parsing&#45;and&#45;serializing a URL](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#encoding-parsing-and-serializing-a-url) given the element&#39;s <code>data</code> attribute&#39;s value&#44; relative to the element&#39;s <a id="values&#58;node&#45;document&#45;3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#44; at the time the attribute is set&#44; or the empty string if there is no such attribute or the result is failure&#46;

<strong>If the element is a <a id="values&#58;the&#45;data&#45;element"></a><code>[data](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-data-element)</code> element</strong>

The value is the value of the element&#39;s <a id="values&#58;attr&#45;data&#45;value"></a><code>[value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-data-value)</code> attribute&#44; if it has one&#44; or the empty string otherwise&#46;

<strong>If the element is a <a id="values&#58;the&#45;meter&#45;element"></a><code>[meter](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-meter-element)</code> element</strong>

The value is the value of the element&#39;s <a id="values&#58;attr&#45;meter&#45;value"></a><code>[value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meter-value)</code> attribute&#44; if it has one&#44; or the empty string otherwise&#46;

<strong>If the element is a <a id="values&#58;the&#45;time&#45;element"></a><code>[time](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-time-element)</code> element</strong>

The value is the element&#39;s <a id="values&#58;datetime&#45;value"></a>[datetime value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#datetime-value)&#46;

<strong>Otherwise</strong>

The value is the element&#39;s <a id="values&#58;descendant&#45;text&#45;content"></a>[descendant text content](https://dom.spec.whatwg.org/#concept-descendant-text-content)&#46;

The <a id="url&#45;property&#45;elements"></a><strong>URL property elements</strong> are the <a id="values&#58;the&#45;a&#45;element&#45;2"></a><code>[a](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-a-element)</code>&#44; <a id="values&#58;the&#45;area&#45;element&#45;2"></a><code>[area](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-area-element)</code>&#44; <a id="values&#58;the&#45;audio&#45;element&#45;2"></a><code>[audio](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-audio-element)</code>&#44; <a id="values&#58;the&#45;embed&#45;element&#45;2"></a><code>[embed](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-embed-element)</code>&#44; <a id="values&#58;the&#45;iframe&#45;element&#45;2"></a><code>[iframe](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element)</code>&#44; <a id="values&#58;the&#45;img&#45;element&#45;2"></a><code>[img](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-img-element)</code>&#44; <a id="values&#58;the&#45;link&#45;element&#45;2"></a><code>[link](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-link-element)</code>&#44; <a id="values&#58;the&#45;object&#45;element&#45;2"></a><code>[object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-object-element)</code>&#44; <a id="values&#58;the&#45;source&#45;element&#45;2"></a><code>[source](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-source-element)</code>&#44; <a id="values&#58;the&#45;track&#45;element&#45;2"></a><code>[track](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-track-element)</code>&#44; and <a id="values&#58;the&#45;video&#45;element&#45;2"></a><code>[video](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-video-element)</code> elements&#46;

If a property&#39;s <a id="values&#58;concept&#45;property&#45;value"></a>[value](#concept-property-value)&#44; as defined by the property&#39;s definition&#44; is an <a id="values&#58;absolute&#45;url"></a>[absolute URL](https://url.spec.whatwg.org/#syntax-url-absolute)&#44; the property must be specified using a <a id="values&#58;url&#45;property&#45;elements"></a>[URL property element](#url-property-elements)&#46;

<strong>Note&#58;</strong>

These requirements do not apply just because a property value happens to match the syntax for a URL&#46; They only apply if the property is explicitly defined as taking such a value&#46;

<strong>Example&#58;</strong>

For example&#44; a book about the first moon landing could be called &#34;mission&#58;moon&#34;&#46; A &#34;title&#34; property from a vocabulary that defines a title as being a string would not expect the title to be given in an <a id="values&#58;the&#45;a&#45;element&#45;3"></a><code>[a](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-a-element)</code> element&#44; even though it looks like a <a id="values&#58;url"></a>[URL](https://url.spec.whatwg.org/#concept-url)&#46; On the other hand&#44; if there was a &#40;rather narrowly scoped&#33;&#41; vocabulary for &#34;books whose titles look like URLs&#34; which had a &#34;title&#34; property defined to take a URL&#44; then the property <em>would</em> expect the title to be given in an <a id="values&#58;the&#45;a&#45;element&#45;4"></a><code>[a](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-a-element)</code> element &#40;or one of the other <a id="values&#58;url&#45;property&#45;elements&#45;2"></a>[URL property elements](#url-property-elements)&#41;&#44; because of the requirement above&#46;

#### <a id="associating&#45;names&#45;with&#45;items"></a>5&#46;2&#46;5 Associating names with items[](#associating-names-with-items)

To find <a id="the&#45;properties&#45;of&#45;an&#45;item"></a><strong>the properties of an item</strong> defined by the element <var>root</var>&#44; the user agent must run the following steps&#46; These steps are also used to flag <a id="associating&#45;names&#45;with&#45;items&#58;microdata&#45;error"></a>[microdata errors](#microdata-error)&#46;

1.  Let <var>results</var>&#44; <var>memory</var>&#44; and <var>pending</var> be empty lists of elements&#46;

2.  Add the element <var>root</var> to <var>memory</var>&#46;

3.  Add the child elements of <var>root</var>&#44; if any&#44; to <var>pending</var>&#46;

4.  If <var>root</var> has an <a id="associating&#45;names&#45;with&#45;items&#58;attr&#45;itemref"></a><code>[itemref](#attr-itemref)</code> attribute&#44; <a id="associating&#45;names&#45;with&#45;items&#58;split&#45;a&#45;string&#45;on&#45;spaces"></a>[split the value of that <code>itemref</code> attribute on ASCII whitespace](https://infra.spec.whatwg.org/#split-on-ascii-whitespace)&#46; For each resulting token <var>ID</var>&#44; if there is an element in the <a id="associating&#45;names&#45;with&#45;items&#58;tree"></a>[tree](https://dom.spec.whatwg.org/#concept-tree) of <var>root</var> with the <a id="associating&#45;names&#45;with&#45;items&#58;concept&#45;id"></a>[ID](https://dom.spec.whatwg.org/#concept-id) <var>ID</var>&#44; then add the first such element to <var>pending</var>&#46;

5.  While <var>pending</var> is not empty&#58;

    1.  Remove an element from <var>pending</var> and let <var>current</var> be that element&#46;

    2.  If <var>current</var> is already in <var>memory</var>&#44; there is a <a id="associating&#45;names&#45;with&#45;items&#58;microdata&#45;error&#45;2"></a>[microdata error](#microdata-error)&#59; <a id="associating&#45;names&#45;with&#45;items&#58;continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue)&#46;

    3.  Add <var>current</var> to <var>memory</var>&#46;

    4.  If <var>current</var> does not have an <a id="associating&#45;names&#45;with&#45;items&#58;attr&#45;itemscope"></a><code>[itemscope](#attr-itemscope)</code> attribute&#44; then add all the child elements of <var>current</var> to <var>pending</var>&#46;

    5.  If <var>current</var> has an <a id="associating&#45;names&#45;with&#45;items&#58;names&#58;&#45;the&#45;itemprop&#45;attribute"></a><code>[itemprop](#names:-the-itemprop-attribute)</code> attribute specified and has one or more <a id="associating&#45;names&#45;with&#45;items&#58;property&#45;names"></a>[property names](#property-names)&#44; then add <var>current</var> to <var>results</var>&#46;

6.  Sort <var>results</var> in <a id="associating&#45;names&#45;with&#45;items&#58;tree&#45;order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order)&#46;

7.  Return <var>results</var>&#46;

A document must not contain any <a id="associating&#45;names&#45;with&#45;items&#58;concept&#45;item"></a>[items](#concept-item) for which the algorithm to find <a id="associating&#45;names&#45;with&#45;items&#58;the&#45;properties&#45;of&#45;an&#45;item"></a>[the properties of an item](#the-properties-of-an-item) finds any <a id="microdata&#45;error"></a><strong>microdata errors</strong>&#46;

An <a id="associating&#45;names&#45;with&#45;items&#58;concept&#45;item&#45;2"></a>[item](#concept-item) is a <a id="top&#45;level&#45;microdata&#45;items"></a><strong>top&#45;level microdata item</strong> if its element does not have an <a id="associating&#45;names&#45;with&#45;items&#58;names&#58;&#45;the&#45;itemprop&#45;attribute&#45;2"></a><code>[itemprop](#names:-the-itemprop-attribute)</code> attribute&#46;

All <a id="associating&#45;names&#45;with&#45;items&#58;attr&#45;itemref&#45;2"></a><code>[itemref](#attr-itemref)</code> attributes in a <a id="associating&#45;names&#45;with&#45;items&#58;document"></a><code>[Document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document)</code> must be such that there are no cycles in the graph formed from representing each <a id="associating&#45;names&#45;with&#45;items&#58;concept&#45;item&#45;3"></a>[item](#concept-item) in the <a id="associating&#45;names&#45;with&#45;items&#58;document&#45;2"></a><code>[Document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document)</code> as a node in the graph and each <a id="associating&#45;names&#45;with&#45;items&#58;the&#45;properties&#45;of&#45;an&#45;item&#45;2"></a>[property](#the-properties-of-an-item) of an item whose <a id="associating&#45;names&#45;with&#45;items&#58;concept&#45;property&#45;value"></a>[value](#concept-property-value) is another item as an edge in the graph connecting those two items&#46;

A document must not contain any elements that have an <a id="associating&#45;names&#45;with&#45;items&#58;names&#58;&#45;the&#45;itemprop&#45;attribute&#45;3"></a><code>[itemprop](#names:-the-itemprop-attribute)</code> attribute that would not be found to be a property of any of the <a id="associating&#45;names&#45;with&#45;items&#58;concept&#45;item&#45;4"></a>[items](#concept-item) in that document were their <a id="associating&#45;names&#45;with&#45;items&#58;the&#45;properties&#45;of&#45;an&#45;item&#45;3"></a>[properties](#the-properties-of-an-item) all to be determined&#46;

<strong>Example&#58;</strong>

In this example&#44; a single license statement is applied to two works&#44; using <a id="associating&#45;names&#45;with&#45;items&#58;attr&#45;itemref&#45;3"></a><code>[itemref](#attr-itemref)</code> from the items representing the works&#58;

``` text
<!DOCTYPE HTML>
<html lang="en">
 <head>
  <title>Photo gallery</title>
 </head>
 <body>
  <h1>My photos</h1>
  <figure itemscope itemtype="http://n.whatwg.org/work" itemref="licenses">
   <img itemprop="work" src="images/house.jpeg" alt="A white house, boarded up, sits in a forest.">
   <figcaption itemprop="title">The house I found.</figcaption>
  </figure>
  <figure itemscope itemtype="http://n.whatwg.org/work" itemref="licenses">
   <img itemprop="work" src="images/mailbox.jpeg" alt="Outside the house is a mailbox. It has a leaflet inside.">
   <figcaption itemprop="title">The mailbox.</figcaption>
  </figure>
  <footer>
   <p id="licenses">All images licensed under the <a itemprop="license"
   href="http://www.opensource.org/licenses/mit-license.php">MIT
   license</a>.</p>
  </footer>
 </body>
</html>
```

The above results in two items with the type &#34;<code>http&#58;&#47;&#47;n&#46;whatwg&#46;org&#47;work</code>&#34;&#44; one with&#58;

<strong>work</strong>

<code>images&#47;house&#46;jpeg</code>

<strong>title</strong>

The house I found&#46;

<strong>license</strong>

<code>http&#58;&#47;&#47;www&#46;opensource&#46;org&#47;licenses&#47;mit&#45;license&#46;php</code>

&#46;&#46;&#46;and one with&#58;

<strong>work</strong>

<code>images&#47;mailbox&#46;jpeg</code>

<strong>title</strong>

The mailbox&#46;

<strong>license</strong>

<code>http&#58;&#47;&#47;www&#46;opensource&#46;org&#47;licenses&#47;mit&#45;license&#46;php</code>

#### <a id="microdata&#45;and&#45;other&#45;namespaces"></a>5&#46;2&#46;6 Microdata and other namespaces[](#microdata-and-other-namespaces)

Currently&#44; the <a id="microdata&#45;and&#45;other&#45;namespaces&#58;attr&#45;itemscope"></a><code>[itemscope](#attr-itemscope)</code>&#44; <a id="microdata&#45;and&#45;other&#45;namespaces&#58;names&#58;&#45;the&#45;itemprop&#45;attribute"></a><code>[itemprop](#names:-the-itemprop-attribute)</code>&#44; and other microdata attributes are only defined for <a id="microdata&#45;and&#45;other&#45;namespaces&#58;html&#45;elements"></a>[HTML elements](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#html-elements)&#46; This means that attributes with the literal names &#34;<code>itemscope</code>&#34;&#44; &#34;<code>itemprop</code>&#34;&#44; etc&#44; do not cause microdata processing to occur on elements in other namespaces&#44; such as SVG&#46;

<strong>Example&#58;</strong>

Thus&#44; in the following example there is only one item&#44; not two&#46;

<a id="microdata&#45;and&#45;other&#45;namespaces&#58;svg&#45;svg"></a>

``` text
<p itemscope></p> <!-- this is an item (with no properties and no type) -->
<svg itemscope></svg> <!-- this is not, it's just an SVG svg element with an invalid unknown attribute -->
```
