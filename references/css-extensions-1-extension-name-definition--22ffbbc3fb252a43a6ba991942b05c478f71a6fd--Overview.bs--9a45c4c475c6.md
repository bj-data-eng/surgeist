Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Extensions Module Level 1](https://raw.githubusercontent.com/w3c/csswg-drafts/22ffbbc3fb252a43a6ba991942b05c478f71a6fd/css-extensions-1/Overview.bs).

The selected CSSWG repository licenses this document by its contributors under the [W3C Software and Document License](../licenses/w3c/software-license-2023.txt); its [exact repository license declaration](../licenses/w3c/csswg-drafts/LICENSE.md) is retained. No source copyright year is supplied by that declaration.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Extensions Module Level 1

Source snapshot: https://raw.githubusercontent.com/w3c/csswg-drafts/22ffbbc3fb252a43a6ba991942b05c478f71a6fd/css-extensions-1/Overview.bs

Pinned source SHA-256: 9a45c4c475c611b22a58393bdfaca1075560b8652eec18ed3f4cfd287aa5c641

Generated intermediate HTML SHA-256: 4aec0c484321a165183a3e8ee1ca0480f75ca9655bb36ee95fcb79faf0eea8e2

Representation notes:
- Generated on 2026-10-03 from the exact pinned Bikeshed source with Bikeshed 7.1.3, then converted to Markdown. This is a generated rendering of that source, not an official publication or a captured historical rendering.
- The compiler used its bundled support-data manifest dated 2026-09-14 without updating it. External automatic link targets and generated bibliography descriptions come from that data; they do not establish historical versions of those external documents.
- Source headings, explicit anchors, normative prose, examples, metadata, and property-definition fields are retained. Compiler-inserted default property rows are omitted. Generated section numbers, cross-reference labels, and formatting are non-normative.
- Ambiguous automatic references remain visible without guessed destinations: “a”.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# CSS Extensions Module Level 1

## <a id="source-metadata"></a>Source metadata

Metadata copied from the pinned source. Editor’s Draft status and work status are those of the source, not a claim of publication or present-day status.



| Field         | Source value                                                                               |
|---------------|--------------------------------------------------------------------------------------------|
| Group         | CSSWG                                                                                      |
| Shortname     | css-extensions                                                                             |
| Level         | 1                                                                                          |
| Status        | ED                                                                                         |
| Work Status   | Exploring                                                                                  |
| ED            | https://drafts.csswg.org/css-extensions                                                    |
| Editor        | Tab Atkins, Google, http://xanthir.com/contact/, w3cid 42199                               |
| Abstract      | This specification defines methods for authors to extend and enhance various CSS features. |
| Link Defaults | css-values-3 (dfn) identifier                                                              |



## <a id="intro"></a>1.  Introduction

When authoring CSS, one often encounters significant repetition in certain features. For example, a given media query might be repeated in several places, or a selector meant to apply to all heading elements requires specifying :is(h1, h2, h3, h4, h5, h6) in every location that uses it.

This repetition makes stylesheets more verbose and difficult to read, and also affects maintenance, as the author has to keep each repetition in sync when making any changes.

This specification defines methods for extending several CSS features so that a long or repeatedly-used value can be given a short, memorable name instead, or a feature can be given a more complex definition controlled by a scripting language. This makes stylesheets easier to read, and more powerful in general, as authors can extend the feature-set of CSS themselves rather than waiting for standards bodies to define new features for them.

## <a id="extension-name"></a>2.  Extension Names

<a id="ref-for-typedef-extension-name"></a>

<a id="ref-for-css-css-identifier"></a>

All extensions defined in this specification use a common syntax for defining their ”names”: the [\<extension-name\>](#typedef-extension-name) production. An <a id="typedef-extension-name"></a>\<extension-name\> is any [identifier](https://drafts.csswg.org/css-values-3/#css-css-identifier) that starts with two dashes (U+002D HYPHEN-MINUS), like --foo, or even exotic names like -- or ------. The CSS language will never use identifiers of this form for any language-defined purpose, so it’s safe to use them for author-defined purposes without ever having to worry about colliding with CSS-defined names.

## <a id="custom-selectors"></a>3.  <a id="custom-selector"></a>Custom Selectors

<a id="ref-for-at-ruledef-custom-selector"></a>

A <a id="declarative-custom-selector"></a>declarative custom selector is defined with the [@custom-selector](#at-ruledef-custom-selector) rule:

<a id="at-ruledef-custom-selector"></a>

<a id="ref-for-typedef-custom-selector"></a>

<a id="ref-for-typedef-selector-list"></a>

<a id="typedef-custom-selector"></a>

<a id="ref-for-typedef-custom-arg"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-typedef-extension-name①"></a>

<a id="ref-for-typedef-custom-arg①"></a>

<a id="ref-for-mult-one-plus"></a>

<a id="ref-for-mult-comma"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-mult-opt②"></a>

<a id="typedef-custom-arg"></a>

<a id="ref-for-typedef-ident-token"></a>

```text
@custom-selector = @custom-selector <custom-selector> <selector-list> ;
<custom-selector> = <custom-arg>? : <extension-name> [ ( <custom-arg>+#? ) ]?
<custom-arg> = '$' <ident-token>
```
<a id="ref-for-typedef-extension-name②"></a>

<a id="ref-for-typedef-ident-token①"></a>

Where there must be no whitespace between `:` and [\<extension-name\>](#typedef-extension-name) or between `$` and [\<ident-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-ident-token) in the above definitions.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4726afee"></a> Simple things are easy:
>
> ```text
> @custom-selector :--heading {
>   expansion: h1, h2, h3, h4, h5, h6;
> }
> ```
>
> More complicated things are possible:
>
> ```text
> // Arguments are specified with $foo.
> // An arg before the pseudo-class captures the rest of the compound selector.
> @custom-selector $rest:--n-siblings($n, $sel) {
>   specificity: $sel;
>   // assumes $sel is a selector, parses it and uses its specificity
>   // otherwise, specificity is [0,1,0]
>   expansion: $rest:nth-child(1 of $sel):nth-last-child($n of $sel),
>     :nth-child(1 of $sel):nth-last-child($n of $sel) ~ $rest;
> }
> ```
<a id="ref-for-custom-selector"></a>

<a id="ref-for-pseudo-class"></a>

<a id="ref-for-typedef-extension-name③"></a>

<a id="ref-for-matches-pseudo"></a>

<a id="ref-for-typedef-selector-list①"></a>

This defines a [custom selector](#custom-selector) which is written as a [pseudo-class](https://drafts.csswg.org/selectors-4/#pseudo-class) with the given [\<extension-name\>](#typedef-extension-name), and represents a [:is()](https://drafts.csswg.org/selectors-4/#matches-pseudo) selector using the provided [\<selector-list\>](https://drafts.csswg.org/selectors-4/#typedef-selector-list) as its argument.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c8fe672b"></a> For example, if an author wanted to easily refer to all heading elements in their HTML document, they could create an alias:
>
> ```text
> @custom-selector :--heading h1, h2, h3, h4, h5, h6;
> 
> :--heading { /* styles for all headings */ }
> :--heading + p { /* more styles */ }
> /* etc */
> ```
### <a id="script-custom-selectors"></a>3.1.  Script-based Custom Selectors

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9e18175f"></a> This one’s more complicated than MQs. Brian Kardell came up with a good proposal for evaluating selectors as JS functions that return a boolean, which had decent performance characteristics by specifying the qualities of the element it was based on (which determined when it would be called).
>
> ```text
> <script>
> CSS.customSelector.set("_foo",
>                      {"predicate": function(el){...},
>                        "matches": "a"});
> </script>
> ```
>
> "matches" is an optional selector specifying what subset of elements the custom selector is valid for. The selector is automatically false for elements that don’t match, and the predicate isn’t called.
>
> By default, the predicate is called whenever there’s a mutation in an element that matches the "matches" selector, or one of its descendants.
>
> You should be able to suppress the auto-calling, and be able to trigger the predicate to run manually. That way you can use mutation listeners manually to only call the predicate when necessary.
>
> We should probably offer some sugar for filtering the list of mutations that trigger the predicate to be called. Maybe just a list of attributes that you’ll be caring about? And/or tagnames?
>
> Maybe let the pseudo-class also accept an argument, and pass it (as a serialized string) as a second argument to the predicate. :\_foo would pass `null`, while :\_foo() would pass `""`.

### <a id="custom-selectors-cssom"></a>3.2.  CSSOM

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-cb6e966f"></a> Fill in.

## <a id="custom-property"></a>4.  Custom Properties

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-b046ceed"></a> Need to more fully support Custom Properties (and eventually remove them from the variable spec entirely, since they’ll be defined here).
>
> <a id="ref-for-funcdef-var"></a>
>
> By default, custom properties are optimized for use as [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) values—​they inherit, have an empty initial value, don’t do any syntax checking, and don’t animate. All of these should be adjustable somehow.
>
> <a id="ref-for-typedef-declaration-value"></a>
>
> <a id="ref-for-length-value"></a>
>
> <a id="ref-for-length-value①"></a>
>
> <a id="ref-for-typedef-color"></a>
>
> ```text
> @custom-property --foo {
>   scope: [ inherit | local ];
>   initial: <declaration-value>*;
>   value: <length> <length> <color>;
>   /* Literally, define a simplistic definition syntax.
>      OR FULL CSS PROPERTY GRAMMAR?!? */
> }
> ```
>
> If you provide a "value" field with animatable types, we can animate in the most direct fashion automatically. We could also let you hook into that: you register a callback, and whenever a property starts animating, we call it with the starting and ending values. You have to return a function which takes a progress value (between 0 and 1) and returns a value for your property; we’ll call it as we animate the value. (How can we hook into Web Anim here? Can you just return an Animation object?)
>
> Do we need a hook for computed values? Interesting. We could just hand your callback a set of property values for the element and its parent (maybe siblings, if you ask for it?), and you can return a new value for the property. This is probably an advanced feature for a later date.
>
> Definitely need a way to listen for elements receiving and changing property values, so you can efficiently polyfill things and make your own properties. Unsure how it would look at the moment.

## <a id="custom-functions"></a>5.  Custom Functions

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d6d2d4d8"></a> Interesting possibilities here. Definitely need some way to define custom functions in CSS. This would, for example, let people define whatever color function they want, such as implementing the [HUSL](http://www.boronine.com/husl/) color space.
>
> Definitely need a JS interface. What options are needed?
>
> Call time/frequency:
>
> - Default should probably treat the function as a preprocessor, calling the JS function once per instance in the stylesheet and substituting in the returned value.
> - Should probably have an option to allow calling per element/instance combo, too. Gets called more as match results change.
>
> We can take some cues from my thoughts on a random() function. It needs per-instance, per-element&#x26;instance, and per "identifier", so you can reuse the same value in multiple spots. That last one can probably be handled manually by the JS, so we don’t have to privilege a particular argument as an identifier.
>
> We’d need to provide the context in which it’s used. Which property, for example. Should we allow them to be used in other places, or should we just define more contextual locations as we go? That is, should we allow custom-defined functions in @supports with this API, or should we add a `.customSupports` map? I suspect that individual cases will have their own useful contextual information, so it’s better to specialize each instance of custom functions.
>
> <a id="ref-for-funcdef-var①"></a>
>
> How much can we do in pure CSS? Being able to substitute values depending on MQs or support queries would be useful. (However, we can do that much just by using custom properties and [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var).) To get \*real\* use out of it, though, I suspect we’d need fuller support for conditionals, likely in the form of SASS’s @if or something similar.

## <a id="custom-combinators"></a>6.  Custom Selector Combinators

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9061f103"></a> Selectors are made of two pieces: simple selectors, and combinators. We should allow custom combinators too.
>
> This is JS-only, because it’s transforming elements, not filtering them, and you can’t express any useful transformations in pure CSS.
>
> You provide a function which, when given an element, produces a list of zero or more elements.
>
> For examples, with div /--foo/ span, the CSS engine will match the first part of the selector and find all the div elements. It passes that list to the function registered for the --foo combinator, and expects to get a new list of elements returned. It then continues on its way, filtering that list to include only span elements, etc.
>
> A child combinator would be something like:
>
> ```text
> CSS.customCombinator.set("--child", function(el) {
>     return el.children;
>   });
> ```
>
> Then div /--child/ span would be identical to div \> span.
>
> If we generalize a selector with a custom combinator to A /--custom/ B, then the UA would automatically call the --custom function whenever new elements match <a id="ref-for-valdef-lab-a"></a>A. If elements stop matching <a id="ref-for-valdef-lab-a①"></a>A, it won’t bother; it’ll just drop them from the result.
>
> Alternately, the function could take a list of elements (all the elements matching <a id="ref-for-valdef-lab-a②"></a>A) and return a new list of elements. This would be a bit more complicated for the author, but would allow more variety in the types of combinators that could be defined, as you could define things that depend on the entire set of matched elements. For example, you could define A /nth 1/ B to give only the first element from the set of <a id="ref-for-valdef-lab-a③"></a>A matches.
>
> (Maybe we allow both variants, since the per-element one is easier to optimize and program against, but the per-set one allows some useful stuff.)
>
> Similarly to custom pseudo-classes, we’d allow arguments, with them parsed eagerly per-instance and passed to the combinator function.
>
> If we do the per-element combinator function, we could potentially cache the results, so that it never needs to be called again for the same element. Possibly have a flag that turns off this behavior, so that you’re guaranteed to be called again.

## <a id="custom-atrules"></a>7.  Custom At-Rules

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-1bd50722"></a> This one’s even less developed, but it would be interesting to allow custom at-rules as well. It’s definitely pure-JS as well.
>
> Unsure exactly what’s best here. Possibly register a callback per rule, which is called with the prelude/contents of the at-rule?
>
> Should we do the callback approach, or just maintain a list of custom at-rules and let scripts parse them themselves? Unfortunately, the latter means we’d have to have a special mechanism to alert scripts when new at-rules get added or removed.
>
> For a lot of these at-rules, we may want a way to know when they’re "applied"—​when, according to the built-in at-rules like @media and @supports, the rule would be applied.

## <a id="privacy"></a>Privacy Considerations

No new privacy considerations have been reported on this specification.

## <a id="security"></a>Security Considerations

No new security considerations have been reported on this specification.

## <a id="references"></a>References

Generated bibliography: these reference descriptions and external auto-links were resolved using Bikeshed 7.1.3’s bundled data; they are not a historical capture of the linked specifications.

### <a id="normative"></a>Normative References

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; Una Kravets; Lea Verou. [CSS Color Module Level 5](https://drafts.csswg.org/css-color-5/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-color-5&#x2F;](https://drafts.csswg.org/css-color-5/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-syntax&#x2F;](https://drafts.csswg.org/css-syntax/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://drafts.csswg.org/css-values-3/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-values-3&#x2F;](https://drafts.csswg.org/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-values-4&#x2F;](https://drafts.csswg.org/css-values-4/)

<a id="biblio-css-variables-2"></a>\[CSS-VARIABLES-2\]  
[CSS Custom Properties for Cascading Variables Module Level 2](https://drafts.csswg.org/css-variables-2/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-variables-2&#x2F;](https://drafts.csswg.org/css-variables-2/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://drafts.csswg.org/selectors/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;selectors&#x2F;](https://drafts.csswg.org/selectors/)
