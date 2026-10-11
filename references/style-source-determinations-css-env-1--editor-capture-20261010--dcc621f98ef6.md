Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Environment Variables Module Level 1](https://drafts.csswg.org/css-env-1/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10. Source status: Editor’s Draft, 8 October 2026.

Full captured HTML SHA-256: `3766138925054b54deba246f0ed2f6fcce1c1b560eea0ae56f642dfdbe62082f` (143027 bytes). Conversion input SHA-256: `dcc621f98ef6b46aea65c77d208d8d5175e16178bea633e73103cdc838e872f5` (38580 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained scope: `env-function`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Environment Variables Module Level 1

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-env-1/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-env-1>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-env-1)

[Inline In Spec](https://drafts.csswg.org/css-env-1/#issues-index)

<strong>Editor:</strong>

Tab Atkins-Bittner (Google)

<strong>Former Editor:</strong>

Dean Jackson (Apple)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-env-1/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-env/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="env-function"></a>3. Using Environment Variables: the <a id="ref-for-funcdef-env③"></a>[env()](#funcdef-env) notation[](#env-function)

In order to substitute the value of an <a id="ref-for-css-environment-variable①⑧"></a>[environment variable](https://drafts.csswg.org/css-env-1/#css-environment-variable) into a CSS context, use the <a id="ref-for-funcdef-env④"></a>[env()](#funcdef-env) function:

<a id="funcdef-env"></a><a id="ref-for-identifier-value"></a><a id="ref-for-integer-value"></a><a id="ref-for-mult-zero-plus"></a><a id="ref-for-comb-comma"></a><a id="ref-for-typedef-declaration-value①"></a><a id="ref-for-mult-opt"></a>

``` text
env() = env( <custom-ident> <integer [0,∞]>*, <declaration-value>? )
```

The <a id="ref-for-funcdef-env⑤"></a>[env()](#funcdef-env) function’s <a id="ref-for-argument-grammar"></a>[argument grammar](https://drafts.csswg.org/css-values-5/#argument-grammar) is:

<a id="typedef-env-args"></a><a id="ref-for-typedef-env-args"></a><a id="ref-for-typedef-declaration-value②"></a><a id="ref-for-comb-comma①"></a><a id="ref-for-typedef-declaration-value③"></a><a id="ref-for-mult-opt①"></a>

``` text
<env-args> = env( <declaration-value>, <declaration-value>? )
```

Like <a id="ref-for-funcdef-var⑤"></a>[var()](https://drafts.csswg.org/css-variables-2/#funcdef-var), a bare comma can be used with nothing following it, indicating that the second <a id="ref-for-typedef-declaration-value④"></a>[\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) was passed, just as an empty sequence.

The <a id="ref-for-funcdef-env⑥"></a>[env()](#funcdef-env) function can be used in place of any part of a value in any property on any element, or any part of a value in any descriptor on any <a id="ref-for-at-rule"></a>[at-rule](https://drafts.csswg.org/css-syntax-3/#at-rule), and in several other places where CSS values are allowed.

<a id="issue-9ee89738"></a>

<strong>Issue:</strong>

[](#issue-9ee89738) Define the full set of places <a id="ref-for-funcdef-env⑦"></a>[env()](#funcdef-env) can be used.

- Should be able to replace any subset of MQ syntax, for example.

- Should be able to replace selectors, maybe?

- Should it work on a rule level, so you can insert arbitrary stuff into a rule, like reusing a block of declarations?

The first argument to <a id="ref-for-funcdef-env⑧"></a>[env()](#funcdef-env) provides the name of an <a id="ref-for-css-environment-variable①⑨"></a>[environment variable](https://drafts.csswg.org/css-env-1/#css-environment-variable) to be substituted. Following the first argument are integers that represent indices into the dimensions of the <a id="ref-for-css-environment-variable②⓪"></a>environment variable, if the provided name represents an array-like <a id="ref-for-css-environment-variable②①"></a>environment variable. The argument after the comma, if provided, is a fallback value, which is used as the substitution value when the referenced <a id="ref-for-css-environment-variable②②"></a>environment variable does not exist.

Note: The syntax of the fallback, like that of custom properties, allows commas. For example, env(foo, red, blue) defines a fallback of red, blue; that is, anything between the first comma and the end of the function is considered a fallback value.

If a property contains one or more <a id="ref-for-funcdef-env⑨"></a>[env()](#funcdef-env) functions, and those functions are syntactically valid, the entire property’s grammar must be assumed to be valid at parse time. It is only syntax-checked at computed-time, after <a id="ref-for-funcdef-env①⓪"></a>env() functions have been <a id="ref-for-substitute-an-env"></a>[substituted](#substitute-an-env).

If a descriptor contains one or more <a id="ref-for-funcdef-env①①"></a>[env()](#funcdef-env) functions, and those functions are syntactically valid, the entire declaration’s grammar must be assumed to be valid at parse time. It is only syntax-checked after <a id="ref-for-funcdef-env①②"></a>env() functions have been <a id="ref-for-substitute-an-env①"></a>[substituted](#substitute-an-env).

To <a id="substitute-an-env"></a><strong>substitute an env()</strong> in a property or descriptor:

1.  If the name provided by the first argument of the <a id="ref-for-funcdef-env①③"></a>[env()](#funcdef-env) function is a recognized <a id="ref-for-css-environment-variable②③"></a>[environment variable](https://drafts.csswg.org/css-env-1/#css-environment-variable) name, the number of supplied integers matches the number of dimensions of the <a id="ref-for-css-environment-variable②④"></a>environment variable referenced by that name, and values of the indices correspond to a known sub-value, replace the <a id="ref-for-funcdef-env①④"></a>env() function by the value of the named <a id="ref-for-css-environment-variable②⑤"></a>environment variable.

2.  Otherwise, if the <a id="ref-for-funcdef-env①⑤"></a>[env()](#funcdef-env) function has a fallback value as its second argument, replace the <a id="ref-for-funcdef-env①⑥"></a>env() function by the fallback value. If there are any <a id="ref-for-funcdef-env①⑦"></a>env() references in the fallback, <a id="ref-for-substitute-an-env②"></a>[substitute](#substitute-an-env) them as well.

3.  Otherwise, the property or descriptor containing the <a id="ref-for-funcdef-env①⑧"></a>[env()](#funcdef-env) function is <a id="ref-for-invalid-at-computed-value-time"></a>[invalid at computed-value time](https://www.w3.org/TR/css-variables-1/#invalid-at-computed-value-time).

<a id="issue-42f9a2dd"></a>

<strong>Issue:</strong>

[](#issue-42f9a2dd) Define when substitution happens. It has to be before <a id="ref-for-funcdef-var⑥"></a>[var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) substitution. Alternately, should <a id="ref-for-funcdef-env①⑨"></a>[env()](#funcdef-env) substitution happen at parse time, so unknown variable names cause it to fail syntax checking? There’s no particular reason to have it happen at computed-value time, like <a id="ref-for-funcdef-var⑦"></a>var() does—​that was to ensure that <a id="ref-for-custom-property④"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) could inherit their value down before they were picked up by a <a id="ref-for-funcdef-var⑧"></a>var().

<a id="issue-9806265e"></a>

<strong>Issue:</strong>

[](#issue-9806265e) When I figure out where else <a id="ref-for-funcdef-env②⓪"></a>[env()](#funcdef-env) can go, define how/when it substitutes.

### <a id="env-in-shorthands"></a>3.1. Environment Variables in Shorthand Properties[](#env-in-shorthands)

<a id="issue-66b99b40"></a>

<strong>Issue:</strong>

[](#issue-66b99b40) If <a id="ref-for-funcdef-env②①"></a>[env()](#funcdef-env) substitution happens during parsing, then this is unnecessary.

The <a id="ref-for-funcdef-env②②"></a>[env()](#funcdef-env) function causes the same difficulties with <a id="ref-for-shorthand-property"></a>[shorthand properties](https://drafts.csswg.org/css-cascade-5/#shorthand-property) as the <a id="ref-for-funcdef-var⑨"></a>[var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) function does. When an <a id="ref-for-funcdef-env②③"></a>env() is used in a <a id="ref-for-shorthand-property①"></a>shorthand property, then, it has the same effects as defined in [CSS Variables 1 § 3.2 Variables in Shorthand Properties](https://www.w3.org/TR/css-variables-1/#variables-in-shorthands).
