# Reference

The public front door is [src/lib.rs](../src/lib.rs); authored models live in
[src/syntax.rs](../src/syntax.rs), property wrappers in
[src/properties.rs](../src/properties.rs), and support metadata in
[src/conformance.rs](../src/conformance.rs). These pages describe the implemented
selected surface. A catalog status applies to its named production and declared
subset; it does not establish complete support for all CSS syntax.

## Entry points and features

| Interface | Available with | Result |
| --- | --- | --- |
| `parse_sheet(&str)` | Default features | `CssParseReport<CssSheet>` |
| `parse_style_attribute(&str)` | Default features | `CssParseReport<CssDeclarationList>` |
| `parse_style_block(source, namespace_context)` | Default features | `CssParseReport<Option<CssStyleBlock>>` |
| `parse_rule(source, namespace_context)` | Default features | `CssParseReport<Option<CssRule>>` |
| `parse_declaration(&str)` | Default features | `CssParseReport<Option<CssDeclaration>>` |
| `parse_selector(&str, &CssNamespaceContext)` | Default features | `CssParseReport<Option<CssSelector>>` |
| `parse_selector_list(&str, &CssNamespaceContext)` | Default features | `CssParseReport<Option<CssStyleSelectorList>>` |
| `parse_media_query(&str)` | Default features | `CssParseReport<CssMediaQuery>` |
| `parse_media_query_list(&str)` | Default features | `CssParseReport<CssMediaQueryList>` |
| `parse_font_face_descriptor_value(&str, CssFontFaceDescriptorKind)` | Default features | `CssParseReport<Option<CssFontFaceDescriptorValue>>` |
| `validate_sheet(&str)` | Default features | `Result<CssSheet, CssValidationFailure>` |
| `validate_style_attribute(&str)` | Default features | `Result<CssDeclarationList, CssValidationFailure>` |
| `parse_component_values(&str)` | Default features | `Result<CssComponentValues, CssComponentValueError>` |
| `parse_property_value_text(source, name, importance)` | Default features | `CssParseReport<Option<CssDeclaration>>` |
| `parse_property_value_text_for_grammar(source, grammar, importance)` | Default features | `CssParseReport<Option<CssDeclaration>>` |
| `parse_property_value(name, components, importance)` | Default features | `Result<CssDeclaration, CssPropertyValueParseError>` |
| `expand_declaration(&declaration)` | Default features | `Result<CssExpansion, CssExpansionError>` |

The [manifest](../Cargo.toml) declares package `surgeist-css`, library
`surgeist_css`, version `0.1.0`, Rust edition 2024, and no default features.
Clean-report validation is always available. Production dependencies are pinned to
`cssparser = 0.37.0`; test-only JSON support uses
`serde = 1.0.228` and `serde_json = 1.0.145`.

`parse_sheet` receives decoded Unicode text. A leading U+FEFF is preserved as
identifier content, just like an interior U+FEFF; byte-stream BOM decoding belongs
to the caller. For example, `\u{feff}a {}` retains the complete type-selector name,
while a lone U+FEFF is an incomplete qualified rule and produces a diagnostic.

`CssParseReport` exposes `syntax()`, `diagnostics()`, `is_clean()`,
`into_parts()`, and `into_validation_result()`. The consuming validation conversion
returns the syntax exactly when the report is clean; otherwise it returns every
recovery diagnostic. It does not rerun a grammar or test contextual usability.
`CssValidationFailure` exposes the complete nonempty diagnostic
sequence through `diagnostics()`, `first()`, and `into_diagnostics()`. See the
[report definitions](../src/report.rs) for their contracts.

## Selector and media query fragments

The four fragment parsers consume the supplied source directly, with original
UTF-8 byte offsets, zero-based lines and UTF-16 columns, and actual EOF. Callers do not need
to synthesize a stylesheet rule around a selector or query. The single-item
functions require exactly one complete grammar production; a root comma is an
error even when a second item would be valid.

`parse_selector` and `parse_selector_list` accept ordinary selectors, including
explicit `&` anchors. Anchors remain authored and symbolic: with a parent selector
list they use that complete list and its maximum specificity; without one they
match the context's scope elements and contribute zero specificity. They are not
rewritten into `:scope`. Relative leading combinators still require their owning
stylesheet contexts. Normalized `ExplicitAnchors` bindings retain an optional
parent through `CssSelectorContext::parent()` rather than manufacturing one.
`CssSelector::to_specified_css()` emits canonical authored selector text.
`to_specified_css_with_limits()` applies cumulative semantic-node and final UTF-8
byte limits, returning a typed error atomically while retaining the input graph.
Escapes, namespace prefixes, selector order and symbolic `&`/`:scope` identity
remain meaningful. Reparse qualified output with the corresponding namespace
bindings; emission does not resolve ancestry or match elements.

Selector input and projection work each count scalar selectors, compound and
complex nodes, qualified type names, anchor occurrences, attributes, pseudo
nodes, relative members, nth patterns, and language/part items. Enum carriers,
list continuations and punctuation add no work nodes. These units apply through
nested arguments under one cumulative budget. Value composition likewise keeps
owning leaf policies cumulative: omitted output still visits its semantic work
without consuming final output bytes.

An invalid outer selector or unforgiving root list produces `None` and a
`RejectInput` diagnostic spanning the complete input. A valid `:is()` or
`:where()` can retain its valid members while reporting discarded forgiving
members. Before that recovery, the complete function argument must satisfy the
lexical `<any-value>?` envelope: bad strings, bad URLs and unmatched closing
delimiters at any depth reject the selector, even when another member is valid.
A missing EOF delimiter alone remains recoverable. This gate uses original
components and preserves the offending token's coordinates; nested forgiving
lists cannot hide an invalid outer envelope. These front doors reuse the implemented selector grammar; their
availability does not establish complete Selectors 4 coverage.

Linguistic selectors preserve authored argument tokens. `CssPseudoClass::Dir`
contains `CssDirectionality`; its checked `try_new` accepts a decoded identifier,
including unknown direction names. `CssPseudoClass::Lang` now contains a nonempty
`CssLanguageRangeList`. Use `ranges()` to inspect each `CssLanguageRange`,
`try_ident` for a decoded identifier, or `try_string` for a decoded string.
Strings may be empty or contain spaces; identifiers may require escaping.
No constructor evaluates directionality, BCP47 matching, or document inheritance.

The scalar constructors return `CssComponentValueError` with programmatic
provenance for invalid values: identifiers reject empty values and NUL, while
strings reject NUL. `CssLanguageRangeList::try_new` returns
`CssEmptyLanguageRangeList` for an empty list; `single` is infallible. Lists
preserve order and duplicates. Each scalar exposes `origin()`, preserving parsed
token spans without manufacturing source positions for constructed values.
`kind()` distinguishes identifier and string language ranges. `to_css_string()`
serializes an argument or argument list canonically, escaping decoded values and
using comma-space separators; it does not serialize an entire selector.

Migration from the identifier-only language API changes `Lang(range)` to
`Lang(list)` and `range.as_str()` to `list.ranges()[0].as_str()` for a known
singleton. Replace `CssLanguageRange::try_new` with `try_ident`; the latter takes
decoded values and escapes spaces or punctuation instead of treating them as raw
CSS source. The old `Hash` implementation is removed. Equality now includes
component spelling and provenance, so identically decoded parsed and programmatic
arguments need not compare equal. Canonical serialization preserves meaning and
token form while intentionally normalizing escape spelling and string quotes.

`CssNamespaceContext::default()` has no bindings. `from_bindings()` consumes
optional prefixes and namespace names in authored order, with the last binding
for each prefix winning. `from_sheet()` copies retained top-level namespace
rules into an owned context that outlives the sheet. `default_namespace()` and
`named_namespace()` expose shared references; named prefixes are case-sensitive,
and an absent binding differs from a binding to the empty namespace. Parsing
borrows the immutable context and preserves symbolic selector namespace
constraints. Retain the context if later matching needs namespace names.
Imported and sibling sheets start with their own empty bindings. Supplying a
context to a fragment does not add declarations to that fragment or mutate the
context.

`parse_media_query` replaces a malformed complete query with `CssMediaQuery::Never`.
`parse_media_query_list` recovers each root comma member independently, preserving
valid neighbors. A grammatically valid unknown feature remains an unknown
condition. An empty media query list is
valid and clean; an empty single query is malformed.
`CssMediaQueryList::new` also accepts an empty vector and preserves every
supplied checked member in order through infallible construction.

Media conditions preserve explicit grouping as
`CssMediaConditionKind::Parenthesized`: the wrapper position identifies the outer
opening parenthesis and the child retains its own first non-trivia position.
`not` takes one media operand, including a general-enclosed function;
`and` and `or` each join a homogeneous
sequence of operands. Mixing operators at one level requires explicit grouping.
Typed queries use the condition-without-or grammar after `and`, so
`screen and ((color) or (monochrome))` is valid while
`screen and (color) or (monochrome)` is malformed. Raw query fragments, stylesheet
media rules, and import media tails share these productions. General-enclosed
fallback is selected only after a complete condition or feature interpretation
fails. Its contents preserve arbitrary checked component values; malformed outer
query sequences still recover at their own comma boundary.

Fragments retain the shared 256-level structural limit and report
`StopAtNestingLimit` without silently discarding neighboring media members.
Deep parsing and media construction/serialization use a bounded worker thread
so ordinary callers need not allocate a larger stack. Accepted EOF closures
appear as recovery diagnostics; clean
validation therefore rejects a report that needed them. Functions inside
discarded forgiving-selector members do not produce retained-closure diagnostics;
for example, `:is(???f(` reports only the retained `:is()` closure.

These operations parse and validate authored grammar. They do not perform
selector matching, media evaluation, cascade, substitution, or CSSOM mutation.
The [public fragment consumer](../examples/selector_query_fragment_consumer.rs)
contains concrete semantic, recovery, namespace, coordinate, and depth examples.

## Source coordinates and diagnostics

Each diagnostic exposes a typed error and stable root code, the first responsible source position, the complete recovery-unit span, and one `CssRecoveryAction`. Source byte offsets index the original UTF-8 input; line and column indices are zero-based, and columns count UTF-16 code units. Display text is for people, not control flow—match typed variants with a wildcard for future non-exhaustive cases.

An unterminated comment produces one `UnexpectedEnd` diagnostic with
`IgnoreUnterminatedComment`. Its position is the actual EOF and its recovery span
runs from the comment opening to EOF. This lexical error is reported even for
comment-only input or when surrounding grammar is rejected; valid surrounding
syntax remains retained. The same rule applies to sheets, declaration lists and
all source-fragment parsers, so clean-report validation rejects unfinished
comments. Structural EOF closures retain their separate diagnostics. Comment-like
bytes inside strings and URL tokens are payload and do not create comment errors.

Backslash-newline and identifier-like EOF escapes produce `EscapeParseError`
diagnostics with `CssEscapeError::Newline` or `EndOfInput`, respectively, and
`RecoverEscape`. Newline recovery leaves a backslash delimiter and preserves the
newline; EOF escape recovery retains U+FFFD. The error position is the backslash
for newline recovery and the EOF cursor for an EOF escape. Both recovery spans
cover the responsible backslash byte. These tokenizer events are reported once
per public parse even when surrounding grammar rejects the source. They do not
spend structural depth or imply a closure; strings and URLs keep their existing
token and closure diagnostics. Clean-report validation rejects these recovered
inputs while ordinary parsing can retain their valid surrounding syntax.

Bad unquoted URL tokens consume their remnants through the first unescaped
closing parenthesis or EOF, following CSS Syntax 3 §4.3.6 and §4.3.14. The first
escape after value whitespace participates in that consumption: `url(a \)still)`
is one complete bad token, while `url(a \\)tail` ends the bad token before `tail`.
Quote- and comment-looking remnants remain URL payload. Recovery preserves the
original token spelling and source coordinates, and later declarations and rules
remain eligible at the actual boundary.

## Owned component values

`CssComponentValues` retains an immutable sequence of tokens, functions, blocks,
whitespace, and comments. Borrowed views expose decoded identifiers and strings,
hash flags, and exact numeric representations without rounding them to floating
point. Checked Rust constructors preserve the same token kinds. Property grammar
validation and variable substitution are separate operations.

Identifier-start lookahead uses authored code points after CSS preprocessing,
including NUL replacement and valid leading escapes. Hash flags therefore
distinguish `#1` (unrestricted) from `#\31` (ID), although both decode to `1`.
An invalid hyphen/backslash-newline start leaves separate delimiters; after a
number it leaves a number and delimiters rather than a dimension. Every split
component owns its original source span and counts separately toward the
component limit. A rejected split capture returns no partial component sequence.

```rust
use surgeist_css::{CssComponentValue, CssComponentValues};

let values = CssComponentValues::try_new(vec![
    CssComponentValue::try_number("10").unwrap(),
    CssComponentValue::try_ident("px").unwrap(),
]).unwrap();
assert_eq!(values.serialize().unwrap().as_css(), "10/**/px");
```

Serialization preserves token representations and meaningful whitespace. It
inserts boundary comments where adjacent tokens would otherwise combine, and
returns an origin map for generated byte offsets. This values-only operation is
distinct from canonical property or CSSOM serialization. Parsed tokens carry an
immutable source snapshot and original span; constructed tokens carry
`Programmatic` provenance. Combining components from different inputs preserves
each origin. Snapshot equality compares source text; `same_snapshot()` tests
whether origins share the same immutable input allocation.

EOF-implied delimiters and token endings are emitted explicitly with
`ImplicitClosure` provenance. Bad strings, bad URLs, unmatched closing
delimiters, and unrepresentable token boundaries return typed errors. Explicit
`CssComponentValueLimits` bound input/output bytes, component count, and depth;
the shared structural ceiling is 256. Input exceeding the byte limit is rejected
before copying or tokenizing it, with `UnretainedInput` provenance and its UTF-8
length. These syntax limits do not promise a global memory budget.

The [CSSOM §2.1 common idioms](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#common-serializing-idioms)
are available as `serialize_css_identifier`, `serialize_css_string`,
`serialize_css_comma_separated_list`, and `serialize_css_whitespace_separated_list`,
each with a `_with_limits` counterpart. Identifier and string functions borrow
arbitrary decoded text, reuse the shared escaping writer, and replace NUL with
U+FFFD; an empty identifier emits empty text and an empty string emits `""`.
List functions borrow already serialized item slices and preserve every item
byte, including empty items and NUL, inserting only `, ` or one space between
items. Empty lists emit empty text. They do not parse, trim, or escape items.
One primitive or list container charges one input and projection node; every
list item charges one more of each, even if empty. Limits count actual output
bytes and failures return no partial string or input mutation. These common
operations leave strict `CssIdent` and property-list admission unchanged.

## Raw single rules

`parse_rule(source, &CssNamespaceContext)` parses exactly one supported ordinary
rule from the original source. A style rule retains its declarations, nested
rules and later declaration runs inside that one node. At-rules use their owning
CSS grammar. Imports and namespaces are parsed in isolation at the top level;
this operation does not validate insertion order in an existing stylesheet.

Whitespace and comments may surround the rule. Empty input, a second rule,
trailing nontrivia, or an invalid outer rule returns `None` with `RejectInput`
over the complete source. This remains true when a stylesheet parser could
recover one valid sibling. `@charset` is encoding metadata and cannot produce a
`CssRule`. The raw parser does not strip a BOM or stylesheet CDO/CDC sentinels.

A valid outer rule survives inner declaration, selector, query and child-rule
recovery with the original diagnostics and actions. Implicit EOF closures are
published only when the outer rule is retained. Resource failures preserve
`StopAtNestingLimit`, including recoverable failures within retained parents.
Positions use the original UTF-8 bytes, zero-based lines and UTF-16 columns.

Namespace bindings are copied from the immutable supplied context without
fabricating namespace declarations. Explicit `&` remains symbolic; leading
relative combinators still require a nested style context. Parsing neither
matches selectors nor applies cascade, substitution or CSSOM mutation.

## Raw style blocks

`parse_style_block(source, &CssNamespaceContext)` accepts exactly one real-brace
block and optional surrounding whitespace/comments. The selected grammar is an
ordinary style body: initial declarations, nested rules, and subsequent declaration
runs retain their authored order. Relative child selectors and explicit `&` stay
symbolic; no selector or parent list is fabricated.

`CssStyleBlock::declarations()` returns the leading declarations, and `rules()`
returns children plus later `CssRule::NestedDeclarations` runs. `origin()` includes
the opening brace through the explicit closing brace or actual EOF. It excludes
surrounding trivia while sharing the complete original source snapshot with
parsed declarations and components. Equality compares contents and source text/span;
it does not compare snapshot identity.

Empty and recovered-empty blocks remain `Some`. Inner errors preserve the owning
recovery actions, and implicit EOF closures are reported on retained blocks.
Missing opening braces, another block or trailing nontrivia reject the complete input with
`None` and `RejectInput`, discarding provisional inner diagnostics. Resource limits
retain `StopAtNestingLimit` and recoverable ancestors. This is a style-body frontdoor,
not a generic component block or a CSSOM mutation operation.

## Raw declaration fragments

`parse_declaration` consumes exactly one ordinary declaration from the unmodified
source, including its property name, colon, value, and optional terminal
`!important` annotation. It validates recognized property grammar or custom-property
grammar beyond CSS Syntax's generic consume-declaration algorithm. Surrounding CSS
whitespace and comments are accepted. Empty custom values are valid.

Top-level semicolons, stray closing delimiters, multiple declarations, unknown
properties, and invalid values reject the entire input. Thus `color:red;` belongs
to `parse_style_attribute`, while `--x:{a:b;c:d}` is a valid singular declaration.
Rejection returns `None` and `RejectInput` over the original full input; resource
exhaustion retains `StopAtNestingLimit`. Implicit EOF closure diagnostics appear
only when the whole declaration survives grammar and full-consumption checks.

The returned `CssDeclaration` owns the real source snapshot. `parsed_name()` and
`parsed_value()` preserve original source regions, including escaped names and
UTF-8 byte offsets, zero-based lines, and UTF-16 columns. Value provenance excludes
the annotation; `importance()` exposes its recognized meaning without a separate
annotation span. Custom values and substitution-dependent known values remain
symbolic. No selector, brace, property name, or value is synthesized.

## Raw property values

`parse_property_value_text(source, name, importance)` consumes the original value
bytes with a supplied `CssPropertyNameRef` and `CssImportance`. The companion
`parse_property_value_text_for_grammar` accepts `CssPropertyGrammar` to preserve
canonical or legacy grammar selection. Both return
`CssParseReport<Option<CssDeclaration>>` and share ordinary declaration grammar.
Known global keywords and substitution-dependent values remain symbolic; custom
values may be empty and contain nested curly punctuation.

Importance comes only from the argument. Root `!` annotations, semicolons and
stray closing delimiters reject the complete input even after `var()` or `env()`.
Rejection reports `RejectInput`; resource limits retain `StopAtNestingLimit`.
Implicit EOF closures are reported only when the value survives validation.

The declaration's `position()` and `parsed_name()` are `None`, because the caller
supplied the name. Its `parsed_value()` is present and shares the original source
snapshot with its components, preserving whitespace, spelling, UTF-8 offsets,
UTF-16 columns and actual EOF. Empty custom values retain a zero-width region.
No property prefix, source wrapper or serialization is introduced.

## Checked declaration construction

`parse_property_value` validates owned components against one property grammar.
It accepts a canonical known-property identity or a checked custom-property name;
importance is supplied separately. Top-level annotations, declaration separators
and trailing grammar tokens are errors. The result retains ordinary, CSS-wide
and substitution-dependent values without resolving them.

```rust
use surgeist_css::{
    CssComponentValue, CssComponentValues, CssImportance, CssKnownProperty,
    CssPropertyNameRef, parse_property_value,
};

let value = CssComponentValues::try_new(vec![
    CssComponentValue::try_dimension("2", "px").unwrap(),
]).unwrap();
let declaration = parse_property_value(
    CssPropertyNameRef::Known(CssKnownProperty::Margin),
    value,
    CssImportance::Important,
).unwrap();
assert_eq!(declaration.position(), None);
assert!(declaration.same_occurrence(&declaration.clone()));
```

Factory-created declarations have no parsed property-name position. Their
`value_components()` preserve the supplied token origins, including components
combined from separate parsed inputs. `CssPropertyValueParseError` carries a typed
grammar or component error and a `CssSerializedOrigin`; generated parsing offsets
are mapped back to those original tokens or to programmatic provenance.

For checked `parse_property_value` construction, a nested resource failure with
`NestingLimit`, `ComponentLimit`, `ByteLimit`, or `CapacityOverflow` remains
`CssPropertyValueErrorKind::Component` with the offending original token origin.
Ordinary property-grammar rejection remains `CssPropertyValueErrorKind::Grammar`.
This preserves the typed resource exception through the known-property parser;
callers migrating from a generic invalid-property error can inspect `kind()` and
`origin()` without treating generated serialization coordinates as authored ones.

Declarations read by `parse_sheet` and `parse_style_attribute` expose
`parsed_name()` and `parsed_value()` with original spans and a shared input
snapshot. The value span excludes the importance annotation and declaration
delimiter. Structural recovery preserves that original snapshot even when it
uses temporary masked input. Empty custom values retain a zero-width value span.

Cloning a declaration preserves its immutable occurrence identity;
`same_occurrence()` distinguishes it from a separately parsed or constructed
declaration. Declaration equality continues comparing body, importance and
optional position, while authored-value equality continues comparing its exact
retained CSS text. Neither equality operation compares occurrence identity.

Migration: `CssDeclaration::position()` now returns `Option<CssSourcePosition>`.
Code inspecting a parsed declaration can require `Some`; constructed declarations
have no invented coordinates. Declaration accessors are no longer `const fn`
because their shared occurrence storage is allocated. Rule, descriptor and
keyframe-declaration position APIs retain their existing contracts.

## Exact authored calculations

Calculation roots accept checked component values through `try_from_components`
and `try_from_components_with_limits`. Their `components()` accessor retains the
original graph, including independently originating children. `expression()`
exposes borrowed exact numeric leaves, symbolic constants, groups, operators,
and functions. A parsed `calc(...)` retains its outer `NestedCalc` node; inspect
`operand()` to reach its body. Leaf `representation()` preserves signs, exponent
spelling, and decimal precision. `unit()` retains the decoded authored unit;
`canonical_unit()` exposes its checked unit identity.

The pure number, percentage, length, angle, time, frequency, and resolution
wrappers check their named domains. `CssLengthPercentageCalculation` supplies the
length percentage context. `numeric_type()` exposes dimensional exponents and
any percentage hint, including intermediate products and quotients. Integer
calculations have numeric type `Number`; `requires_rounding()` records the
integer consumer's deferred conversion requirement. No arithmetic is evaluated
during admission: division by zero, symbolic infinity and NaN, range clamping,
and integer rounding belong to later resolution.

`CssHintedNumberCalculation` checks a Number result whose dimensional powers
cancel while its percentage hint remains non-null. Color and opacity expose
distinct `HintedNumberCalculation` branches for these percentage-permitting
slots. Relative non-hue channels, profile expressions and alpha retain the same
permission. Percentage-permitting scale3d/scaleZ operands, filter amounts,
border-image slice/width and line-height also expose this distinct branch.
Pure `CssNumberCalculation` and integer roots, hue, and legacy
number-only CMYK channels reject this payload. This follows
[Typed OM's numeric matching rule](https://www.w3.org/TR/2024/WD-css-typed-om-1-20240321/#cssnumericvalue-match)
and [Values 4's type checking](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-type-checking):
matching a percentage-permitting Number result does not erase its hint or supply
an external percentage basis. Shared specified projection retains unresolved
expressions; borrowed parsed-alpha views retain the distinct calculation type.

The shared Values 4 grammar admits `calc`, `min`, `max`, `clamp`, `round`, `mod`,
`rem`, the trigonometric functions, `pow`, `sqrt`, `hypot`, `log`, `exp`, `abs`, and
`sign`, with their intrinsic arity and type rules. Binary addition and subtraction
require actual whitespace on both sides; comments alone are insufficient.
`sign()` admits compound dimensional inputs and produces Number, preserving
any inferred hint. Specified simplification may retain the compound input when
it cannot resolve its magnitude; admission does not require another evaluator.
Standalone delimiter negation such as `-(1px)` is invalid. Substitution-dependent
declarations remain pending upstream; exact constructors reject residual `var()`.

Construction errors distinguish component, grammar, arity, type, domain, and
resource failures and retain responsible origins. Explicit byte, component, and
depth limits apply before admission. `serialize()` emits canonical authored
syntax with a map to original token and delimiter origins. Equality compares the
authored graph and provenance, not evaluated numeric equivalence.

Pure lengths use `CssLengthCalculation`; mixed length-percentages use
`CssLengthPercentageCalculation`. Both retain the exact checked component and
arithmetic graph. Borrowed leaf views expose numeric representations and unit
identities. Programmatic components use checked exact-token constructors;
relative units and percentages remain unresolved.
`CssCalculationType::Integer` is removed: use `Number` with the integer wrapper
and its `requires_rounding()` flag. `CssCalculationExpressionRef::Negate` is
removed; signs belong to numeric tokens or the subtraction and product grammar.
`CssIntegerCalculation::literal` now allocates and is no longer a `const fn`.
Calculation-root `result_type()`, sum and product `len()`, and sum-term and
product-factor `operator()` accessors are also no longer `const fn`; call them
at runtime.

`CssLengthPercentageCalculation::try_sum(first, rest)` assembles checked roots
using a first operand and subsequent `(CssCalculationSumOperator, operand)`
pairs. `try_sum_with_limits` applies aggregate depth, component-count and byte
limits before cloning the assembled graph. These budgets include the new
`calc()` wrapper and whitespace/operator separators, all original child trivia,
and canonical numeric output. A single operand still receives the wrapper.
Finite exhausted limits stop further iterator consumption; default unlimited
count and byte limits do not guarantee termination for an arbitrary iterator.

Assembly preserves original child components, lexical spelling, snapshots and
recovery origins. Only the wrapper and separators are programmatic. Numeric
serialization retains its existing canonical policy, including lowercase units;
the original lexical spelling remains available through `components()`. The numeric
owner rechecks the assembled arithmetic grammar without evaluation; a unitless
zero admitted only at a calculation root can therefore fail as an operand,
whereas `0px` remains valid. Trusted checked children may retain recovered
closures during assembly; direct public construction from recovered components
continues to reject them.

Construct compound calculations with `CssLengthPercentageCalculation::try_sum`
and `CssCalculationSumOperator`, then admit the result through the scalar owner
for its domain. For pure-length math, construct a `CssLengthCalculation` through
its checked factories and pass that typed root to the scalar owner's
`try_from_calculation`. Pure-length parsing selects a length root; scalar
admission trusts the checked calculation's dimensional root without rechecking
its contextual function support. The first operand has no binary operator;
signed operand tokens remain valid. Checked construction owns grammar and
resource admission.

Positions, basic shapes, and `clip-path` use their typed authored graphs,
including symbolic calculations.

Ordinary lengths and length-percentages use the shared checked
`CssSpecifiedLength`, `CssSpecifiedNonNegativeLength`,
`CssSpecifiedLengthPercentage`, and
`CssSpecifiedNonNegativeLengthPercentage` owners directly. Admission checks exact
decimal spelling, units, and ordinary sign without floating-point conversion.
Only true numeric zero permits an omitted length unit; tiny nonzero values do
not become zero. Nonnegative domains admit negative lexical zero and defer the
range of actual symbolic math until contextual resolution. Pure-length math
uses a length root and excludes a residual percentage dimension.

In length and length-percentage branches, owning aggregates compare exact token
and arithmetic structure while ignoring numeric source origins. This covers
positions, transforms, shadows,
filters, clipping and shapes, gradients, border-image, border-spacing, and
typography. Variants, keywords, list order, authored arity, and omission remain
distinct. Shared scalar and raw calculation equality still compares provenance;
their `origin()` accessors retain parsed spans or programmatic origin. Applying
this aggregate policy to symbolic AST branches follows the checked owner model;
the retired raw math equality also compared origins. Equality does not evaluate
or normalize mathematically equivalent spellings.

## Canonical ordinary number output

Specified-value writers round ordinary numbers to at most six fractional decimal
places, as required by the selected
[CSSOM component serialization rule](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-component-value).
They use nearest rounding with halfway values rounded away from zero, trim
redundant fractional zeros and the decimal point, and emit fixed notation with
no scientific exponent. Exact or rounded zero emits `0`, without a minus sign.
CSSOM does not specify the decimal halfway direction; the selected policy follows
WebKit's fixed CSS number formatter.

Rounding applies after the owning writer's exact unit shift: milliseconds become
seconds, and opacity percentages become numbers. Other ordinary writers retain
their selected units. The shared policy covers number, percentage, length, flex
breadth, angle, time, frequency, opacity, font weight, oblique angles and ratio
operands, including their composed values. Each ratio operand rounds independently; serialization does not
divide or simplify a ratio.

The retained value remains exact. Construction and range checks use the original
coefficient, so a tiny negative value cannot enter a nonnegative domain by
rounding to zero, and a tiny nonzero number cannot enter a unitless-zero grammar.
Source origins, units and equality also retain their authored meaning.
`CssComponentValues::serialize()` preserves numeric token spelling; ordinary
specified serialization deliberately produces rounded text.

```rust
use surgeist_css::{
    CssComponentValue, CssSpecifiedNumber, CssSpecifiedValueSerializationLimits,
};

let component = CssComponentValue::try_number("0.9999996")?;
let number = CssSpecifiedNumber::try_from_component(component.clone())?;
assert_eq!(
    number.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 1))?,
    "1",
);
assert_eq!(number.literal_component(), Some(&component));
# Ok::<(), Box<dyn std::error::Error>>(())
```

Ordinary scalars still cost one input and one projection node. Byte limits apply
to the actual rounded output and its suffix: a carry that shortens a coefficient
can fit a smaller budget. Extremely small ordinary values can emit zero without
expanding their exponents. Very large output can still fail with a typed resource
error. Failure returns no partial public CSS and leaves the authored value
unchanged; composed writers share one cumulative budget.

### Calculated number output

Finite numbers in non-color specified calculations, declared relative-color
calculations, ordinary colors embedded as origins, retained calculated alpha
and calculated mix weights use the same six-place, fixed-notation output policy.
Ordinary non-alpha color component and hue calculations also use this policy
when their projected result remains
context-dependent, including components inside `color-mix()`. The mathematical
projector evaluates context-independent parts with binary64 arithmetic, then rounds the actual
finite binary value for text. `calc(1 / 3)` emits `calc(0.333333)`, and the exact
dyadic halfway value `calc(-1 / 128)` emits `calc(-0.007813)`.

An authored decimal and its binary approximation can fall on different sides of
a rounding midpoint. Ordinary `5e-7` emits `0.000001`; `calc(5e-7)` emits `calc(0)`
because its projected binary64 value is just below that decimal midpoint.
Finite integral results emit all their actual integer digits, including digits
that differ from a shorter decimal spelling which merely round-trips to the
same binary64 value. The authored calculation and its source origin remain
unchanged.

Same-unit Sum coefficients accumulate in source order under this binary64
policy. Thus `calc(1e16 - 1e16 + 1)` emits `calc(1)`, while
`calc(1 + 1e16 - 1e16)` emits `calc(0)`. This operational order follows frozen
[WebKit's same-unit simplifier](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/calc/CSSCalcTree%2BSimplification.cpp#L490-L554).
Values 4 leaves supported numeric precision and range
[implementation-defined](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types);
the authored coefficients remain exact even when their projected magnitudes differ.

Finite canonical angle projection supports `[-f64::MAX, f64::MAX]` degrees.
When a finite authored angle conversion or finite-input Angle-valued operator
or named-function result overflows that range, it converts to the nearest
supported multiple of `360deg`,
as required by [Values 4's angle range rule](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types).
The positive endpoint is `f64::from_bits(f64::MAX.to_bits() - 31)`: the maximum
value is `(2^53 - 1) * 2^971`, and its significand is 31 above a multiple of 45.
Since `360 = 45 * 8`, subtracting those 31 representable steps gives the largest
finite exact multiple of 360. Representable angles above this multiple remain
supported. An overflowing `grad` coefficient is normalized to a bounded
19-digit decimal prefix before binary64 degree scaling, so `1.8e308grad` still
projects to a supported degree magnitude. Authored coefficients and origins
remain unchanged. Infinite operands and defined calculation exceptions retain
their mathematical semantics; [trigonometric functions of infinity produce NaN](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#trig-infinities).
Same-unit Sum coefficients accumulate in source order, then Angle range
conversion applies once when the combined scalar replacement is created,
following [Values 4's Sum simplification](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-simplification).
Separately grouped Angle Sums materialize separately. Thus
`calc((1e308deg + 1e308deg - 1e308deg) / 1e308deg)` emits `calc(1.797693)`,
while grouping the first two terms emits `calc(0.797693)`.
Products flatten nested Product children and then merge Number coefficients in
source order before distribution and typed evaluation, following
[Values 4's Product simplification](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-simplification).
Thus `cos(1e308deg * 2 * 0)` first combines the Number coefficient to zero and
emits `calc(1)`. Each merged coefficient spends a shared projection node.
Already materialized Number and compound arithmetic retains its selected IEEE
infinity and NaN outcomes; an Angle ancestor does not reconstruct discarded
magnitudes. A retained compound Product may instead flatten into its ancestor
before the final Angle result is evaluated. Target-context computed and
used-value range clamping remains with the consuming owner.

The selected draft's general `atan2()` normalization interval
[excludes `-180deg`](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#funcdef-atan2),
while its [exceptional signed-zero/infinity table](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#trig-infinities)
explicitly returns that endpoint for several cases. Projection follows the
specific table for listed exceptional arguments, corroborated by frozen
[WebKit's executor](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/calc/CSSCalcExecutor.h#L353-L357).
The selected draft remains internally inconsistent on this endpoint.

The shared simplifier retains hinted percentage `min()`, `max()`, and `clamp()`
comparisons until their basis is known, including percentage siblings inside a
partially simplified comparison. A negative basis can reverse their ordering.
Unhinted percentages and comparable same-unit dimensions may still fold, and
same-unit percentage sums may still combine. These distinctions follow
[Values 4's simplification algorithm and percentage note](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-simplification).

For same-unit contextual coefficients with nonnegative size bases, `abs()` and
`hypot()` preserve the unit while folding: `abs(-2em)` emits `calc(2em)` and
`hypot(3em, 4em)` emits `calc(5em)`. This applies to `em`/`rem`, `lh`/`rlh`,
viewport units and container units, whose bases are
[font sizes](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#propdef-font-size),
[line heights](https://www.w3.org/TR/2011/REC-CSS2-20110607/visudet.html#propdef-line-height),
[viewport dimensions](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#viewport-relative-lengths)
or [container dimensions](https://www.w3.org/TR/2022/WD-css-contain-3-20220818/#container-lengths).
The bases may be zero; the coefficient identities remain valid. The selected
[norm examples](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#exponent-funcs)
also give `hypot(3em, 4em)` as `5em`. Glyph-metric units and flex fractions keep
their contextual expressions pending a basis guarantee. Mixed units and hinted
percentages remain symbolic. `sign(-2em)` also stays symbolic because a zero font
size changes its result; [sign operates on the resolved magnitude](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#sign-funcs).

```rust
use surgeist_css::{
    CssNumberCalculation, CssSpecifiedNumber, CssSpecifiedValueSerializationLimits,
    parse_component_values,
};

let calculation = CssNumberCalculation::try_from_components(
    parse_component_values("calc(1 / 3)")?,
)?;
let number = CssSpecifiedNumber::try_from_calculation(calculation.clone())?;
assert_eq!(
    number.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(4, 4, 14))?,
    "calc(0.333333)",
);
assert_eq!(calculation.serialize()?.as_css(), "calc(1 / 3)");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Symbols, units, operator order, exceptional values and actual negative-zero
arithmetic retain the projector's existing behavior. Border-image and scroll
shorthands compare canonical projected components before rounding their text.
Unequal finite coefficients remain separate even when they produce identical
text; equal canonical component sequences can still compress. This comparison
does not establish arbitrary algebraic equality or authored graph identity.

Traversal costs stay unchanged, and byte limits count actual rounded output
including wrappers and units. Captured children still obey their scratch bounds
and retain their traversal costs even when shorthand output compresses.
Calculated integer text uses this finite number policy without performing
computed integer rounding; ordinary integer literals retain exact digits.
Relative calculated coefficients use this policy wherever embedded, including
nested origins and mix components. Ordinary-origin calculations preserve number
and percentage categories, explicit unclamped alpha, and canonical angle units.
For example, `alpha(from rgb(calc(1 / 128) none none))` emits
`alpha(from rgb(calc(0.007813) none none))`; an origin percentage calculation
keeps `%` without adopting the destination channel's relative percentage scale.
For ordinary non-alpha component and hue calculations, the existing projection
result selects the text policy. A contextual hue such as
`hsl(calc(1em / 1px + 0.0078125deg / 1deg) 50% 50%)` emits
`hsl(calc(0.007813 + (1em / 1px)) 50% 50%)`. Contextual same-unit ratios remain
symbolic under the existing projector; fully resolved absolute-unit ratios and
NaN simplification retain their existing outcomes. Percentage scaling and angle
conversion precede coefficient formatting.

Ordinary RGB, HSL and HWB calculated alpha that resolves without external
context emits a scalar independently of missing or contextual sibling channels.
Number alpha keeps its scale; percentage alpha divides by 100. NaN becomes zero,
then alpha clamps to `[0,1]`. Exact clamped unity is omitted before formatting:
`rgb(1 2 3 / calc(2))` emits `rgb(1, 2, 3)`, while `calc(.9999996)` remains
explicit alpha `1` because its unrounded value is below one. Grammar reentry
accepts that output but can omit its now-direct unity on subsequent serialization.

This ordinary sRGB phase follows the frozen WebKit interpretation recorded in
the [standards catalog](../specs/catalog.json):
[Color 4 §15.1](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#resolving-sRGB-values)
requires historical scalar simplification, whereas its
[§16.1.2](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#serializing-modern-alpha-values)
preserves unclamped specified calculated alpha. The selection applies only to
ordinary RGB/HSL/HWB slots, including ordinary Mix children. Relative-alpha and
Lab/OK calculations use the separate retained-phase selections described below.

Contextual alpha and calculated alpha in custom, Lab/LCH/OK and predefined
`color()` families remain explicit and unclamped, including rounded zero or
unity. Thus `color(--P 0 / calc(.78125%))` emits
`color(--P 0 / calc(0.007813))`. Actual Origin alpha preserves its authored
dimension and calculation wrapper instead of applying ordinary finalization.

Calculated mix weights keep `%` and their `calc()` wrapper. A calculated weight
remains unknown for omitted-share and equal-weight decisions, even when it
serializes as `calc(50%)`; serialization does not fill an omitted sibling weight
or normalize authored shares. These rules apply to nested mixes and to ordinary
colors inside a mix used as an origin.

Ordinary RGB, HSL and HWB non-alpha slots that resolve without external context
emit final scalar text, including resolved siblings of contextual slots. Finite
calculated scalars round their original post-scale binary64 bits to at most six fractional places,
nearest with ties toward positive infinity under
[Color 4's sRGB serialization rule](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#css-serialization-of-srgb).
Six places is this implementation's bounded precision choice. Thus
`hwb(none calc(-1 / 128) 20%)` emits `hwb(none -0.007812% 20%)`;
the contextual coefficient policy above still rounds negative ties away from zero.
Direct literals retain their exact decimal arithmetic. Media-query writers retain
their separate text policies. Numeric color captures must succeed
before final scalar emission and retain their scratch costs even when their text
is discarded. For example, `hsl(none calc(-10000000000) 50%)` needs a
19-byte capture budget although its final `hsl(none 0% 50%)` is only 16 bytes.
Similarly, `rgb(none none none / calc(100000000000000000000))` needs 27 bytes
for retained alpha scratch before emitting its 26-byte opaque color.
Retained alpha, weight and contextual component captures count rounded text
against their scratch bounds, using the same cumulative traversal budget.
Number formatting leaves calculation arithmetic unchanged. Authored coefficients
remain exact; context-independent projection uses the documented binary64 policy.
Canonical text does not resolve symbolic dependencies. Computed and used-value
contextual range clamping remains with the consuming owner.

### Authored and resolved serialization phases

Specified serialization in `surgeist-css` represents authored values without
performing contextual style resolution. Ordinary specified lengths, angles and
frequencies reuse
[ordinary number output](#canonical-ordinary-number-output), retaining exact
coefficients, selected units and source provenance. Values 4 represents a
[specified length by its quantity and unit](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#lengths).
Ordinary specified angle and frequency output retains its selected lowercase
unit under Surgeist's existing project policy. Frozen WebKit's
[primitive serializer](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSPrimitiveValue.cpp#L266-L348)
and [retained numeric-unit serializer](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/values/primitives/CSSPrimitiveNumericTypes%2BSerialization.h#L49-L53)
corroborate that choice. It does not establish a normative answer to CSSOM's
incomplete specified/computed wording.

Specified calculations reuse [calculated number output](#calculated-number-output)
and the accepted [shared math owner](https://github.com/bj-data-eng/surgeist/issues/735).
They simplify context-free subtrees as the selected Values 4 clauses permit,
including compatible-unit conversion, while retaining their specified math
structure and unresolved context. For example, ordinary `1khz` emits `1khz`,
while `calc(1khz)` emits `calc(1000hz)`. Values 4 separately requires
[computed and used-value canonicalization](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#compat);
computed dimensions use their canonical `px`, `deg` or `hz` unit. Contextual
length resolution remains with [the style owner](https://github.com/bj-data-eng/surgeist/issues/727).

`CssUrl::serialize_specified()` reuses the existing URL provider and
[common string escaping](#owned-component-values). It emits the authored decoded
target through `url()` or `src()`, retaining function identity and ordered
symbolic modifiers. A relative target stays relative, a local fragment stays a
fragment, and empty targets retain `url("")` or `src("")`. Serialization neither
resolves nor fetches the target. Values 4 defines
[computed absolute resolution and failed-resolution fallback](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#relative-urls),
[local fragment output](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#local-urls),
and [empty URL output](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#url-empty).
Those contextual operations and base provenance belong to
[root-owned URL integration](https://github.com/bj-data-eng/surgeist/issues/726).
Frozen WebKit likewise separates
[specified/resolved URL emission](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/values/primitives/CSSURL.cpp#L40-L55)
from [style URL conversion](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/values/primitives/StyleURL.cpp#L82-L87).

Explicit owning property rules take precedence over generic CSSOM shortening.
The deprecated [clip owner](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#clip-property)
uses its existing comma-separated `rect()` provider; modern `clip-path: rect()`
has a separate whitespace grammar. [Display 3](https://www.w3.org/TR/2026/CRD-css-display-3-20260605/#legacy-display)
keeps specified `inline flex` distinct from `inline-flex`, although computed
exposure selects precomposed keywords. [Cascade 5](https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#aliasing)
excludes legacy shorthands from CSSOM declaration selection; the existing
property schema supplies that distinction to
[declaration composition](https://github.com/bj-data-eng/surgeist/issues/831).
Color phase rules remain with the existing declared/computed color owners.

[The CSS-owned phase disposition](https://github.com/bj-data-eng/surgeist/issues/844)
retains CSSOM's individual dimension/URL annotations and its unenumerated
legacy exception note as source limits. Those limits neither override the
explicit owning clauses nor certify downstream resolution, complete scalar or
declaration serialization, or standards consensus. This boundary introduces no
second formatter, evaluator, property registry or URL resolver.

### Declared color literal output

`CssColor::to_specified_css()` applies the generic six-place number policy to
ordinary color literals embedded as origins and to direct relative-color
literals. The retained literal spelling, category and source origin remain
unchanged. The selected
[Color 5 origin rule](https://www.w3.org/TR/2026/WD-css-color-5-20260908/#serial-origin-color)
preserves number and percentage categories, explicit alpha and unclamped
magnitudes. An origin hue authored as `360` remains `360`; an explicit `1turn`
becomes `360deg`. Angle conversion precedes rounding, so `.0000001turn` emits
`0.000036deg`.

Under the selected
[relative-color rule](https://www.w3.org/TR/2026/WD-css-color-5-20260908/#serial-relative-color),
direct percentages become numbers in the destination channel's scale before
rounding. Thus `rgb(from red 20% g b)` emits `rgb(from red 51 g b)`,
`lab(from red l 20% b)` emits `lab(from red l 25 b)`, and
`oklab(from red l 20% b)` emits `oklab(from red l 0.08 b)`. Predefined and
custom-profile `color()` channels divide percentages by 100. Direct numbers
remain unscaled, and non-alpha channels remain unclamped. References, `none`
and calculation branches retain their own serialization rules; direct
percentage conversion does not rewrite `calc(30%)`.

These rules also apply when relative colors occur inside another origin or
`color-mix()`. Standalone color precision, direct alpha, hue normalization and
fully numeric standalone non-alpha color calculations keep their separate
policies. Retained calculated alpha, calculated mix weights and context-dependent
non-alpha calculations use the
[calculated number policy](#calculated-number-output).
Composed colors share cumulative resource limits; byte limits count rounded
text, and failure returns no partial string or changes to the authored graph.

## Authored preferred aspect ratios

The selected [CSS Sizing 4 (2026-09-04)](https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/#aspect-ratio)
and [CSS Values 4 (2024-03-12)](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#ratios)
grammar accepts `auto`, a nonnegative
ratio, or both in either order. The ratio has one or two nonnegative number
operands; an omitted denominator means `1`. Zero numerators and denominators
remain authored values. Number tokens retain their exact decimal spelling and
origin, including magnitudes outside `f32`; typed number math stays symbolic.
`CssAspectRatioValue::Ratio` and `AutoRatio` hold a `CssSpecifiedRatio`, whose
private fields are composed from checked `CssRatioOperand` values. The property
is a non-inherited longhand with `Auto` initial value.

`serialize_specified()` canonically places `auto` first and emits both ratio
components, with one cumulative input, projection, and output budget. It does
not divide the components or resolve whether a zero ratio is useful. Inspect ratio operands through
`numerator()`, `denominator()`, `literal_component()`, and `calculation()`.
The effective source revisions for `aspect-ratio` and `box-sizing` now name
their selected Sizing publications; historical provenance records and stable
feature IDs retain their original identities.

## Authored flow tolerance

The selected [Grid3 publication](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#placement-tolerance)
defines `flow-tolerance: normal | <length-percentage> | infinite`, with no
nonnegative restriction. `CssFlowTolerancePropertyValue::value()` exposes its
checked `CssFlowTolerance`; `as_ref()` returns `Normal`, `Infinite`, or a borrowed
`LengthPercentage`. Construct the symbolic keywords with `normal()` and
`infinite()`, or use `length_percentage(CssSpecifiedLengthPercentage)` for checked numeric values.
`Default` is symbolic `normal`. Its 1em used value in grid lanes and 0 used value
in other layout modes require downstream context and are not computed here.

The scalar owner accepts signed exact lengths, percentages, zero, and
supported calculations. It rejects unrelated keywords. Signed
operands, later subtraction and typed calculations remain valid and symbolic;
no computed range evaluation occurs at this boundary.

`serialize_specified()` emits canonical specified CSS: `normal` and `infinite`
stay keywords, ordinary numbers use
[canonical rounding](#canonical-ordinary-number-output), and supported math uses
the shared length-percentage projection. For example, `+02.500EM`
becomes `2.5em`, `calc(2px + 3px)` becomes `calc(5px)`, and `calc(-2px - 3%)`
becomes `calc(-3% - 2px)`. Relative units and percentage bases remain unresolved.
Determinate math retains a `calc()` wrapper under the shared numeric specified
serialization policy, including folded `min()`, `max()`, and `clamp()` values.
`serialize_specified_with_limits()` applies one cumulative input-node,
projection-node, and output-byte budget, returning a typed
`CssSpecifiedValueSerializationError` without partial public output or input
mutation. A keyword or literal costs one input and one projection node;
calculations use the scalar owner's existing costs.

Migration: use `flow-tolerance` and `CssKnownProperty::FlowTolerance`. Both
`grid-flow-tolerance` and `item-tolerance` are unknown properties, without aliases.
The obsolete `CssGridFlowTolerance`/`CssGridFlowToleranceValue` types and their
I01 wrapper projection are removed. Historical source records and captured test
inputs retain their original identities. The effective property feature is
`ext.property.flow-tolerance`, sourced from the dated Grid3 publication.
Support remains `Partial`: the broader Values 4 math-function grammar is still
unfinished. This property migration does not complete the other Grid3 families.

## Intrinsic declaration expansion

`expand_declaration` currently covers custom declarations, physical and logical
margin/padding longhands, their logical axis pairs, border width, style and color,
the four side-border shorthands, `border`,
the `background` shorthand and its eight longhands,
the `border-image` shorthand and its five longhands, `border-collapse`, `border-spacing`, `caption-side`, `clip`, `empty-cells`,
`table-layout`, `flow-tolerance`, `color`, `font-family`,
`text-orientation`, its legacy `glyph-orientation-vertical` grammar, `opacity`, `display`, `box-sizing`,
`order`, `aspect-ratio`, `visibility`, `direction`, `unicode-bidi`, `writing-mode`, `text-combine-upright`,
the `container` shorthand and its two longhands, and `all`.
The shared property schema owns their
member lists, initial values and reset-only components. Other known properties
return typed unsupported errors preserving their identity. The stylesheet
normalizer uses this same expansion boundary, so its complete property coverage
remains unfinished.

`background` owns eight ordered settable members: image, position, size, repeat,
attachment, origin, clip, and color, without extra reset-only members. Every
checked authored layer supplies one entry to each of the seven lists, filling
omissions from the corresponding schema initial. Color appears once from the
final layer or the transparent schema initial. This intrinsic projection does
not match list lengths to used images or resolve positioning geometry.

`CssBackgroundLayer::try_new` retains its seven optional typed components and
rejects an empty layer or size without position. `CssBackground::try_new` rejects
an empty list and color in a nonfinal layer through
`CssBackgroundConstructionError`. Parsed values use the same checked boundaries
while preserving precise authored diagnostic locations and child origins.

`CssBackground::serialize_specified` and its bounded variant emit image,
position, `/ size`, repeat, attachment, origin/clip, and final color in grammar
order. Proved simple initials may be omitted; a remaining size retains its
position, equal boxes compact to one, and an empty effective layer emits `none`.
Keyword positions, unresolved calculations, images, and contextual colors use
their existing specified providers. One cumulative budget counts root/layer and
all actually authored child visits, including omitted children. Omitted text
consumes no final bytes; generated `none` and implicit size height consume
projection nodes only. Failure returns no partial text and does not mutate the
retained authored graph. These operations do not establish complete Backgrounds
or shared color/numeric grammar support.

`border-image` has five ordered settable members: source, slice, width, outset,
and repeat, with no reset-only members. Its checked `try_new` returns `None`
for an empty value or width/outset without slice. Parsing uses that same boundary.
Expansion projects supplied components and obtains omitted values from the schema:
`none`, `100%` without fill, number `1`, number `0`, and `stretch stretch`.
The parent `border` shorthand continues to reset all five members.

`CssBorderImage`, `CssBorderImageSlice`, `CssBorderImageWidth`,
`CssBorderImageOutset`, and `CssBorderImageRepeat` expose `serialize_specified`
and `serialize_specified_with_limits`. Scalars compress their expanded edges to
the shortest one-to-four canonical sequence; slice appends `fill` after its edges,
and repeat emits one keyword for equal axes. Composition follows source,
slice with slash width/outset, then repeat. Only exact ordinary literal initials
are omitted; retained width/outset retains its required slice. Omitted initial
width permits two slash components, and an empty effective shorthand emits `none`.
Programmatic unitless length zero in width/outset emits `0px`, preserving its
length branch alongside the distinct number-zero branch.

Each edge group charges one parent plus four variant components and each
numeric provider's own input/projection costs. Thus an ordinary numeric group
costs nine input and nine projection nodes, an all-auto width costs five each,
`fill` adds one each, and repeat costs three each. All four providers are visited
even when equal edges compress. Captures precede group output and each remains
bounded by remaining final byte space. Proved initial omissions visit the same
providers without formatting discarded literals or charging final bytes.
The shorthand adds one parent; generated fallback `none` adds one projection
node, so all five explicit simple initials cost 32 input, 33 projection and four
output bytes. Failure returns no partial text and does not mutate authored data.
Canonical math comparison inherits the shared numeric projection contract;
calculations remain symbolic where context is unavailable and are not treated
as literal initials. These APIs do not complete shared Images/Values grammars
or perform CSSOM, cascade, image loading, or layout.

The five [selected CSS2 table properties](https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html)
expand to one typed longhand each. `border-collapse` accepts `collapse | separate`
(initial `separate`), `border-spacing` accepts one or two nonnegative lengths
(initial horizontal and vertical `0`), `caption-side` accepts `top | bottom` (initial `top`),
`empty-cells` accepts `show | hide` (initial `show`), and `table-layout` accepts
`auto | fixed` (initial `auto`). The first four inherit by default; `table-layout`
does not. The four keyword enums serialize canonically with bounded specified-value
limits. Checked `CssSpecifiedNonNegativeLength` serializes one axis; the checked
`CssBorderSpacing` serializes the full effective horizontal-then-vertical pair
under one cumulative budget, repeating a single authored length without resolving
relative units or typed math. This specified projection does not select CSSOM's
minimal one-value spelling. The [selected Logical 1 caption-side definition](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#caption-side)
adds `inline-start | inline-end` only with `left | right` support, which this profile has not
selected. Caption positioning relative to writing mode remains for downstream
style and layout interpretation; CSS retains the authored `top | bottom` value.

The selected [Scroll Snap 1 publication](https://www.w3.org/TR/2021/CR-css-scroll-snap-1-20210311/)
defines 25 authored properties: `scroll-snap-type`, `scroll-snap-align`,
`scroll-snap-stop`, and the physical, flow-relative, axis-pair and four-side
`scroll-padding`/`scroll-margin` families. Values retain exact signed length
or nonnegative length-percentage literals and typed math without narrowing to
`f32`. Padding additionally retains `auto`. Pair and four-side models retain
the authored component count; bounded specified serialization emits the shortest
equivalent component sequence, including a retained `logical` marker. The
[selected Logical 1 §4.7 draft](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#logical-shorthand-keyword)
defines that marker and its block-start, inline-start, block-end, inline-end
ordering. These authored values remain symbolic until style and layout have
writing-mode and scroll-container context.

Pair and four-side compression compares ordinary values exactly before rounding
emitted coefficients. Distinct values that round alike retain their positions;
equivalent coefficient spellings can compress. In a length slot, exact unitless
zero compares as `0px`, and the first selected component keeps its own emitted
spelling. Other units and percentages remain distinct. Calculations compare
canonical projected components before rounding emitted coefficients. Every
authored child consumes its
existing node budget, even when compression omits its text; the logical marker
adds its own node charges and eight output bytes.

```rust
use surgeist_css::{CssKnownPropertyValueRef, parse_style_attribute};

let report = parse_style_attribute(
    "scroll-padding-inline: .12345641px .12345642px",
);
assert!(report.is_clean());
let CssKnownPropertyValueRef::ScrollPaddingInline(value) = report.syntax()[0]
    .known().expect("known scroll padding")
    .property_value().expect("ordinary scroll padding")
else { panic!("expected scroll-padding-inline") };
assert_eq!(
    value.value().serialize_specified()?,
    "0.123456px 0.123456px",
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The 19 terminal properties, four axis shorthands and two four-side scroll
shorthands have intrinsic metadata and completed contributions. Four-side
expansion follows the [mode-selected membership policy](#four-side-shorthand-membership).
Whole-value pending substitutions remain symbolic until strict grammar reentry.

Opacity is a non-inherited longhand with numeric initial value `1`. Its ordinary
contribution retains the exact `CssOpacityValue`, including percentages,
calculations and out-of-range specified values. Computed clamping belongs to
style. Raw authored value text remains separate from canonical specified-value
serialization, which is not established by expansion.

The canonical Writing Modes properties use four distinct keyword domains:
`CssDirection`, `CssUnicodeBidi`, `CssWritingMode`, and `CssTextOrientation`.
Their intrinsic initials are respectively `Ltr`, `Normal`, `HorizontalTb`, and
`Mixed`. Direction, writing-mode and text-orientation inherit by default;
unicode-bidi does not. Each expands to one longhand. The `all` reset continues
to exclude direction and unicode-bidi.

Each enum exposes `serialize_specified()` and its bounded variant, emitting
canonical lowercase keywords with the shared one-input-node, one-projection-node
and exact-byte budgets. Direction and WritingMode wrappers expose `value()`;
UnicodeBidi keeps `bidi()`, and TextOrientation keeps `orientation()`. Authored
wrapper text and parsed/programmatic component origins remain unchanged.
The selected [Writing Modes4 definition](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/#block-flow)
includes both `sideways-rl` and `sideways-lr`; the other canonical keyword sets
also occur in the selected Writing Modes3 publication.

Normalization preserves authored order and conditions. It does not execute bidi
isolation, direction propagation, glyph rotation, or layout. Optional old SVG
writing-mode spellings and `sideways-right` are not selected. This canonical
keyword boundary does not settle numeric/math admission for the legacy
glyph-orientation alias.

`CssTextCombineUpright` includes the selected Level 4 `digits <integer>?`
grammar alongside `None` and `All`. `Digits(None)` preserves an omitted count;
`Digits(Some(count))` preserves an explicit count. The checked
`CssTextCombineDigitCount` admits literal integers from 2 through 4 through
`try_literal`, or a checked `CssIntegerCalculation` through `from_calculation`.
Its `literal()` and `calculation()` accessors distinguish these cases. Parsed
and checked declarations share this grammar and preserve component origins.
The property inherits by default, has initial `None`, expands to one longhand,
and participates in `all`.

Specified serialization preserves `digits` versus `digits 2`. Number-valued
calculations retain their math form and shared bounded projection: for example,
`digits calc(1 + 2)` becomes `digits calc(3)`, while `digits calc(2.5)` remains
fractional. Literal counts outside 2–4 are invalid; calculations are admitted
without computed rounding or clamping. Style owns those later operations and
the omitted count's computed default of 2.

The serializer charges one outer input/projection node; an explicit count adds
its own scalar or calculation costs. After reserving outer nodes, it reserves
the seven-byte `digits ` prefix before checking the child's residual limits.
Keyword-only forms cost one node at each stage, and explicit literals cost two.
Output is atomic and bounded by the shared serialization limits.

Migration: `CssTextCombineUpright` now owns calculation data and no longer
implements `Copy` or `Eq`. Borrow it through the unchanged `combine()` accessor,
or call `clone()` when ownership is needed; use `PartialEq` for comparisons.
The existing `None` and `All` variants retain their meaning.

Visibility uses the existing `CssVisibility::{Visible, Hidden, Collapse}` values.
It is inherited by default and its ordinary initial value is `Visible`.
`CssVisibilityPropertyValue::value()` exposes that typed value while `as_css()`
retains authored spelling. Its intrinsic
expansion emits one longhand contribution; globals and unresolved substitutions
keep their separate states. Expansion does not resolve inheritance or apply
hidden/collapse rendering effects.

`CssVisibility::serialize_specified()` emits `visible`, `hidden`, or `collapse`.
The bounded variant charges one input node, one projection node and the exact
canonical byte count before allocating output. The selected definition is
[Display3 §4](https://www.w3.org/TR/2026/CRD-css-display-3-20260605/#visibility),
which supersedes CSS2 for this property. Layout, painting and interaction effects
remain downstream.

`content-visibility` follows the pinned
[Containment 2 §4](https://www.w3.org/TR/2022/WD-css-contain-2-20220917/#content-visibility)
authored grammar: exactly `visible`, `auto`, or `hidden`. It is a non-inherited
longhand with intrinsic initial `visible`. `CssContentVisibilityPropertyValue`
retains authored spelling through `as_css()` and exposes the checked enum through
`value()`.
Ordinary declarations expand to one terminal contribution. CSS-wide keywords
remain symbolic, and variable, environment, and attribute substitutions reenter
the same whole-value grammar while retaining the original occurrence.
`CssContentVisibility::serialize_specified()` emits one canonical keyword; its
bounded variant charges one input node, one projection node, and exact output
bytes. Skipped-content state, user relevance, used containment, layout, painting,
and interaction need downstream context and are not decided by this authored API.

Order is a non-inherited longhand with a checked programmatic integer initial of zero.
Its specified integer value expands to one contribution; normalization retains
source order and does not sort declarations by their numeric values. Layout
item ordering and computed integer rounding belong to downstream consumers.

The selected [CSS2 §9.9.1 z-index definition](../../../references/css2--visuren.html--3f334c530cf4.md#propdef-z-index)
provides a non-inherited longhand with intrinsic initial `auto`. Ordinary
`auto` and integer values contribute one `CssLonghandValueRef::ZIndex` payload;
CSS-wide values stay symbolic. Pending whole-value substitutions reenter the
same strict grammar, preserving original importance and replacement origins.
Normalization retains declaration order. `CssZIndexValue::serialize_specified()`
emits canonical `auto` or delegates to the exact integer and specified math
writer. Its bounded variant shares input, projection and UTF-8 byte budgets;
`auto` charges one input and one projection node. Errors return no partial CSS
and leave the authored value unchanged. [Positioned Layout 3 §2.2](../../../references/css-position-3--WD-css-position-3-20251007--3c8a8120af2c.md#stacking)
stacking contexts, painting order and contextual interpretation remain downstream.

`CssIntegerValue` is shared by Order and the integer branch of ZIndex.
`Literal(CssIntegerLiteral)` retains the complete checked integer token and its
parsed or programmatic origin at every magnitude. Decimal points, exponent
notation, dimensions and other token kinds are rejected by literal construction.
`CssIntegerLiteral::from_i32` constructs canonical programmatic tokens for
machine integers. `is_zero` and `is_negative` classify the exact integer,
including signed zero, without a machine-range bound. `compare_value` returns
mathematical `Ordering` across signs and arbitrary magnitudes, ignoring explicit
plus signs, leading zeros and provenance and equating signed zeros. It does not
change lexical/provenance-sensitive equality or supply an incompatible `Ord`.
`serialize_specified()`
removes redundant signs and leading zeros without rounding the magnitude or
changing the retained token. Its bounded variant charges one input and
projection node for an ordinary value, then checks canonical output bytes.
Order and ZIndex wrappers expose their semantic models through `value()`;
ZIndex `Auto` remains a distinct unresolved keyword. Positive ordinary counts
use `CssPositiveIntegerValue::Literal(CssPositiveIntegerLiteral)`, whose checked
integer must be greater than zero. This transport does not determine stacking.

`Calculation` retains the authored numeric expression and uses the shared
binary64 specified math projection. Fractional results retain a math wrapper:
`calc(1.5)` remains `calc(1.5)` without integer rounding. An explicitly constructed
Calculation leaf `9007199254740993` retains those original digits but serializes
as `calc(9007199254740992)` under that precision policy; an ordinary Literal
serializes exactly as `9007199254740993`. Serialization never mutates either
representation. The output remains stable when reparsed and serialized again.
Exact ordinary integer fidelity is a Surgeist contract; the selected Values4
standard permits implementation-defined numeric ranges.

Display is a non-inherited longhand with initial `inline`. Its current
`CssDisplayValue` represents outside/inside pairs, flow-only list items,
internal table/ruby values, box values, legacy inline values, and the selected
Grid3 `grid-lanes` and `inline-grid-lanes` alternatives. The selected Display3
and Grid3 productions admit 112 keyword sequences mapping to 44 specified
values. Duplicate categories and combinations such as `list-item flex` or
`inline grid-lanes` are rejected. Multi-keyword display is still one longhand
and produces one contribution. Generic CSS-wide and pending `var()` values use
the same expansion and strict grammar-reentry boundary.

`CssDisplayPropertyValue::value()` exposes the complete specified model;
`as_css()` retains the authored spelling.
For example, `block flex` has the same specified value as `flex`, while
`inline flex` stays distinct from legacy `inline-flex`. CSS Display3 §2.6
requires that distinction at the specified stage even though computed values
can agree.

`CssDisplayValue::serialize_specified()` produces canonical specified text;
`serialize_specified_with_limits()` uses the shared serialization limits and
error types. It lowercases keywords, orders components, and omits equivalent
defaults: `flow block` becomes `block`, `flex inline` becomes `inline flex`, and
`list-item flow-root inline` becomes `inline flow-root list-item`. Legacy
spellings remain legacy. One value consumes one input node and one projection
node; output is at most 26 ASCII bytes. Limits are checked before output
allocation, and failure leaves the original value and authored text unchanged.
Direct enum construction admits only valid category combinations, so typed
construction does not require a second text parser or unchecked flags.

The primary authored definition is [Display3 §2](https://www.w3.org/TR/2026/CRD-css-display-3-20260605/#the-display-properties),
with the two standalone additions from [Grid3 §2.2](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#grid-lanes-containers).
Canonical ordering and shortening follow the selected
[CSSOM value serialization rules](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serializing-css-values).
This support does not execute box generation, blockification, inlinification,
cascade, or layout, and does not complete the other Display3/Grid3 properties.
Shared `env()` admission follows the authored environment substitution rules below.

Custom declarations produce `CssContributions::Custom`. Its `declaration()` view
retains the case-sensitive name and either authored token text or a whole-value
CSS-wide keyword. `source()` retains the original components, importance and
occurrence identity. Even values containing `var()` remain completed symbolic
custom contributions; variable substitution and cycle handling belong to style.
Migration: replace matches on the removed `UnsupportedCustomProperty` error with
the custom contribution branch. Custom computed values remain unresolved until
style supplies the required context.

Completed longhand contributions expose a property-coupled `CssLonghandValueRef`,
a symbolic CSS-wide keyword, or `UserAgentInitial`. `ordinary_value()` borrows
an owned `CssLonghandValue` only for the ordinary branch. Omitted border components use `medium`, `none`
and `currentcolor`; the specified width remains `medium` even when the style is
`none`. The `border` shorthand also resets all five border-image longhands.
CSS-wide shorthand keywords propagate to those reset-only components too.
`all` remains a `CssUniversalReset`, excluding custom properties, `direction`
and `unicode-bidi`. Its `excludes()` method reports those explicit exclusions;
it does not select cascade targets.

A substitution-dependent declaration returns `CssExpansion::Pending`. Once the
downstream substitution owner supplies complete replacement components,
`CssPendingSubstitution::reenter` returns `CssContributions` or a typed error,
never another pending result. It rejects residual `var()`, `env()` or `attr()` at any nesting depth
before applying the original property grammar. Grammar and component failures
retain the same origin mapping as `parse_property_value`. Failure publishes no
partial contributions, and the pending handle remains available for another
attempt.

Every contribution retains the original declaration occurrence and importance
through `source()`. Contributions from reentry share one replacement component
tree, available through `replacement_components()`, including its original token
origins. These operations preserve symbolic lengths, colors and images; style
owns variable environments, invalid-at-computed-value handling and cascade.

## Authored environment substitutions

The required [Env1 definitions](https://www.w3.org/TR/2025/WD-css-env-1-20250923/#env-function)
allow known property values containing valid `env()` references to remain pending.
Names are case-sensitive custom identifiers; unknown platform names are retained.
Ordinary indices use exact nonnegative integer tokens without a machine-size bound.
Integer calculations remain symbolic, including results whose range requires later
computed-value handling. CSS does not look up environment values or select fallbacks.

The selected fallback grammar distinguishes `env(foo)` from `env(foo,)`:
the first has no fallback; the second does not match the intrinsic production.
`env(foo, )` contains a whitespace-token fallback, and `env(foo,,)` contains a comma.
`var()` retains its separate explicit empty-fallback exception.

Whole-property pending admission considers each function family independently.
A family qualifies when at least one occurrence exists and every occurrence of
that family matches its grammar. Either qualifying `var()` or `env()` family
defers the property:
`env(var(--name))` is pending through `var()`, while `var(env(foo))` is pending
through `env()`. This preserves authored syntax without choosing substitution order.
Otherwise-permitted custom-property and style-query token streams can retain
malformed `env()` calls; retention does not certify future substitution success.
Custom properties still reject malformed `var()` references. A failed typed style
query can survive as general-enclosed syntax under the container-query grammar.

Parsed and checked construction preserve the same token boundaries, origins and
importance. Size and scroll-state query operands retain their existing admission
rules. The public `D-ENV1` source uses `LaterStandard`, whose catalog meaning is a
standards-track source outside the selected whole-module profile. Its shared-value
record remains partial for other Env1 contexts and execution; the selected
font-palette descriptor consumer is included. This does not add Env1 as a
selected whole module.

## Authored attribute substitutions

The required [Values 5 `attr()` and `<syntax>` definitions](https://www.w3.org/TR/2024/WD-css-values-5-20241111/#attr-notation),
imported through the selected [Content 3 `content` grammar](https://www.w3.org/TR/2025/WD-css-content-3-20251204/#typedef-content-content-list),
admit syntactically valid `attr()` in ordinary known-property values. Such a
declaration retains its original component tree, source origin and importance
as a whole-value pending substitution. It remains symbolic: CSS does not read
an element's attributes, parse an attribute value, choose a fallback, or
produce a computed value. The context-free `parse_property_value` checked
constructor uses the same admission rule. `CssPendingSubstitution::reenter`
requires a complete replacement component tree and rejects residual `attr()`
before checking the ordinary property grammar; it cannot return another
pending value. Parsed `content` with a valid `attr()` now exposes
`substitution_dependent()` rather than typed `CssContentValue`; the existing checked
typed content constructors remain available for concrete values.

The selected grammar is `attr(<attr-name> <syntax>?, <declaration-value>?)`.
The optional syntax is direct `<syntax>`, including bracketed type names, bare
keyword alternatives, `|` combinations, and an equivalent decoded quoted
syntax string; it has no `type()` wrapper. Type-name and keyword terminals
follow the selected Values 4 ASCII case-insensitive keyword rule. A present
fallback requires one or more declaration-value component tokens; whitespace
is a token, while a comment alone is not. The comma is omitted when there is
no fallback. A braced fallback must satisfy the strict
declaration-value grammar. The dated Values 5 publication contains illustrative
`<number px>` spellings that conflict with its normative `<syntax>` production;
they are not admitted. Decoded syntax strings have a bounded 256-pass limit;
exhaustion reports `NestingLimit`. This slice uses checked retained components
without claiming a reusable public syntax AST.

Custom-property, descriptor and query admission retain their own policies.
Style supplies the namespace environment for attribute lookup and enforces
the selected [URL-taint restrictions](https://www.w3.org/TR/2024/WD-css-values-5-20241111/#attr-security)
during substitution; CSS does not execute that operation. This narrow import
does not select the rest of Values 5.

## Intrinsic metadata and authored grammar identity

`CssKnownProperty::grammar()` returns the canonical grammar handle.
`CssPropertyGrammar::from_name` accepts decoded names with ASCII case folding;
it does not trim whitespace or parse escapes. Name-equivalent aliases share the
canonical handle. `glyph-orientation-vertical` instead retains a distinct legacy
shorthand handle targeting `TextOrientation`; its angle grammar remains active
after substitution. `parse_property_value_for_grammar` uses that exact grammar
with the same component limits, token boundaries and original-origin mapping as
`parse_property_value`. `CssKnownDeclaration::grammar()` retains it for ordinary,
CSS-wide and pending values. Migration: structural declaration equality now
includes grammar identity. Occurrence identity and clone behavior are unchanged.

`grammar.metadata()` and `CssKnownProperty::metadata()` return intrinsic metadata
for schema-annotated longhands and canonical shorthands, `all`, and the legacy
glyph shorthand. An unannotated recognized property returns
`CssPropertyMetadataError::Unavailable`; the support catalog remains independently
available through `property_support_metadata`. Metadata kind is always meaningful:
a longhand, shorthand, or universal reset. Shorthand members are terminal IDs in
settable-then-reset-only order. Legacy grammars are listed by
`CssKnownProperty::legacy_shorthands()`. A grammar's feature ID identifies support
metadata; the pinned source catalog owns standards revision identity.

Longhand metadata reports inheritance and constructs a `CssLonghandInitialValue`
without an invented authored occurrence. Initial `color` is symbolic `CanvasText`,
`text-orientation` is `mixed`, and `font-family` is
`CssUserAgentInitial::FontFamily`. Style later supplies the user-agent environment.
The owner uses the same initial transition for shorthand omissions and reset-only
members. An authored CSS-wide `initial` remains a global keyword contribution.
Private fields couple every owned ordinary or initial value to its terminal
property; metadata availability does not claim complete grammar or shorthand
coverage beyond the selected slice.

## Authored overflow wrapping

[CSS Text 4 §6.4](https://www.w3.org/TR/2026/WD-css-text-4-20260814/#overflow-wrap-property)
defines `overflow-wrap` as the inherited longhand with initial value `normal`.
It accepts `normal`, `break-word`, or `anywhere`. The legacy `word-wrap` spelling
is a name alias of the same property and accepts the same values; parsed
declarations retain the original spelling and source coordinates. The selected
support record cites the [14 August 2026 CSS Text 3 draft](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/#propdef-overflow-wrap).

`CssOverflowWrapPropertyValue::value()` exposes the exact checked keyword;
Intrinsic
expansion emits one `OverflowWrap` contribution, preserving importance and source
occurrence. CSS-wide values remain symbolic, and substitution-dependent values
reenter the same grammar after replacement. `CssOverflowWrap::serialize_specified`
emits the canonical keyword under configurable resource limits. Line breaking,
min-content sizing, cascade, and layout use these authored values downstream:
`anywhere` creates opportunities counted in min-content size, while `break-word`
does not. This crate leaves those contextual effects to their owning layers.

## Authored gap values and legacy names

[Alignment 3 §8.1](https://www.w3.org/TR/2026/WD-css-align-3-20260130/#column-row-gap)
defines `row-gap` and `column-gap` as `normal` or a nonnegative
length-percentage. `CssGapValue::Normal` and
`CssGapValue::LengthPercentage(CssSpecifiedNonNegativeLengthPercentage)` keep
those alternatives typed. Negative literal lengths and percentages are invalid;
symbolic percentages and math retain their authored meaning for later contextual
resolution. Both longhands start at `normal` and are not inherited.

[Alignment 3 §8.2](https://www.w3.org/TR/2026/WD-css-align-3-20260130/#gap-shorthand)
defines `gap` as one row value and an optional column value. `CssGapShorthand`
preserves that authored arity: `row()` returns the first value,
`authored_column()` reports whether a second was supplied, and `column()`
returns the effective second value, repeating the row value when omitted.
Intrinsic expansion emits `row-gap` then `column-gap`, with no reset-only
members. CSS-wide keywords apply to both; a substitution-dependent value
stays pending until strict reentry checks the same grammar and emits complete
contributions or a typed error.

[Alignment 3 §8.4](https://www.w3.org/TR/2026/WD-css-align-3-20260130/#gap-legacy)
requires `grid-row-gap`, `grid-column-gap`, and `grid-gap` as name-equivalent
aliases of those three canonical properties. The `grid-` spelling does not
restrict the grammar to grid containers. Lookup, support metadata, and parsed
values use the canonical property identity while `parsed_name()` retains the
original spelling and source origin. The value serializers are bounded and
preserve one- versus two-value specified form; they do not serialize an entire
stylesheet or resolve used gaps in grid, flex, or multicolumn layout.

`CssGapPropertyValue::value()` exposes `CssGapShorthand`; the row and column
wrappers expose `CssGapValue`. Their checked scalar branches retain exact
literal components and symbolic calculations.

## Immutable stylesheet normalization

`normalize_sheet` turns retained authored syntax into an immutable
`CssNormalizedSheet`. `normalize_report` additionally preserves the original
ordered recovery diagnostics in `CssNormalizedReport`; `is_clean()` still means
exactly that the original diagnostic slice was empty. Both operations are
atomic: an unsupported emitted declaration or resource failure returns
`CssNormalizationError` without changing the source or exposing partial output.

The ordered `items()` stream contains every authored rule occurrence, followed
by its leading declaration occurrences and ordered children. Each declaration
contains one grouped `CssExpansion`, its original declaration occurrence,
zero-based emitted declaration order, and shared rule and selector contexts.
Shorthand members share their source occurrence and order. No declaration wins
the cascade during normalization, and source positions are provenance rather
than precedence keys.

`CssRuleContextKindRef` borrows a group header or intact terminal payload.
`Style`, `ScopedStyle`, and `NestedDeclarations` carry their shared selector
contexts even when the style has no declarations. Conditional headers retain
their conditions, layer blocks retain names or anonymous occurrences, and scope
headers retain roots and limits. Imports, namespaces, layer statements, font
faces, keyframes, counter styles, and pages remain ordered typed payloads.
Keyframe, page, and font declarations are not emitted as element-style
declarations. A named supports definition is one terminal rule payload: its
test-body declarations and nested candidates emit no style declarations or
selector contexts, even under a style rule. Imports are neither loaded nor
merged, and conditions are not evaluated. Both the ordinary and separate scoped
authored rule trees are walked.

Each selector context contains one complete authored selector list, its earlier
parent-selector reference when nested, and its nearest scope context when
present. `CssSelectorBinding` distinguishes ordinary selectors, implicit
descendants, authored leading combinators, explicit parent-list nesting anchors,
and scoped grammar anchors. Explicit and implied nesting anchors refer to the
complete parent list with its maximum specificity. Selector functions and
repeated anchors remain intact for downstream specificity and matching rules.
There is no eager selector multiplication.

Nested declaration runs reuse exactly their enclosing style's selector context,
including pseudo-elements and per-selector specificity behavior. This differs
from an explicit nested `&` rule, which receives its own context with a parent
binding. `same_context()` checks immutable occurrence identity: equal authored
rules and equal anonymous layers are not interned together. These handles are
not mutable CSSOM identities, revisions, or stable keys across normalizations.
Rule and failure positions are optional and never manufacture coordinates.

`normalize_sheet_with_limits` and `normalize_report_with_limits` take
`CssNormalizationLimits::try_new(max_rule_depth, max_rules, max_declarations,
max_contributions)`. Top-level rule depth is zero; the depth ceiling cannot
exceed 256. Every visited rule counts, including implicit nested-declaration
runs. Emitted declaration occurrences count once each. Completed longhands count
individually, while custom, universal-reset, and pending groups count as one
contribution member. Declarations inside terminal payloads are not individually
counted. The defaults retain the depth ceiling and impose no additional count
policy; zero budgets admit only output consuming zero corresponding units.
These limits bound traversal/output counters, not payload bytes, allocator
usage, or total process memory. The named supports parser and checked
constructors own the component budget for their retained test-body payload.
Pending reentry is an independent immutable expansion operation and does not
change these counters or the normalized sheet.

Rule depth and rule count are checked before copying the rule header. Declaration
count, expansion capability, and contribution count are checked before expansion
allocates its members. A typed limit failure identifies its resource and limit;
a declaration failure also retains the original occurrence and its prospective
ordinal. Rule-admission failures refer to their enclosing rule context;
declaration failures refer to their immediate owning style or declaration run.

The current normalizer covers all retained rule families but only the intrinsic
property expansion listed above. Other valid known declarations fail explicitly
instead of being dropped or passed through as completed longhands. This does not
claim complete CSS grammar, complete shorthand expansion, or public checked
whole-sheet Rust construction. Parsing and checked construction of individual
declarations already meet at the same expansion boundary. Style owns subsequent
CSSOM mutation, revisions, cascade, variable environments, substitution, and
invalidation; cross-crate composition remains root-owned.

## Authored property inspection

The [how-to guide](how-to.md#inspect-a-known-declaration) shows property/value
inspection and exact authored text. Each ordinary wrapper exposes its sole
checked semantic value through one intentional accessor. `CssImportance` and `CssSupportStatus` are closed public enums;
other public enums are non-exhaustive, so downstream matches require a wildcard.
The [authored-value explanation](explanation.md#symbolic-values-and-compatibility)
describes these symbolic boundaries.

## Numeric values, exact timing domains, and symbolic calculations

Ordinary time values retain their exact decimal coefficient, `s` or `ms` unit,
and original component provenance. `CssTimeLiteral` owns one dimension;
`CssTimeValue` holds that literal or a checked symbolic time calculation.
`CssDuration` applies the nonnegative range to an ordinary literal, while delay
values use signed `CssTimeValue` directly. Signed zero is valid, and bare number
zero is not a time dimension. A well-typed calculation remains symbolic even
when its eventual duration range belongs to computed-value processing.

```rust
use surgeist_css::{
    CssDuration, CssKnownPropertyValueRef, CssTimeLiteral, CssTimeUnit,
    CssTimeValue, parse_style_attribute,
};

let negative = CssTimeValue::from_literal(
    CssTimeLiteral::try_new("-1", CssTimeUnit::Seconds).unwrap(),
);
assert!(CssDuration::try_new(negative).is_err());

let report = parse_style_attribute(concat!(
    "transition-duration: calc(-1s + 2s); ",
    "transition-delay: -250ms",
));
assert!(report.is_clean());

let CssKnownPropertyValueRef::TransitionDuration(duration) = report.syntax()[0]
    .known()
    .expect("known duration")
    .property_value()
    .expect("ordinary duration")
else {
    panic!("expected transition-duration");
};
assert!(duration.durations().values()[0].time().calculation().is_some());

let CssKnownPropertyValueRef::TransitionDelay(delay) = report.syntax()[1]
    .known()
    .expect("known delay")
    .property_value()
    .expect("ordinary delay")
else {
    panic!("expected transition-delay");
};
let literal = delay.delays().values()[0].literal().unwrap();
assert_eq!(literal.numeric().representation(), "-250");
assert_eq!(literal.unit(), CssTimeUnit::Milliseconds);
assert_eq!(literal.serialize_specified().unwrap(), "-0.25s");
```

Time literals, time values and durations provide `serialize_specified()` and
`serialize_specified_with_limits()`. Ordinary output converts milliseconds
exactly to seconds before applying
[canonical number rounding](#canonical-ordinary-number-output). Signed or rounded
zero emits `0s`; the stored coefficient, unit and original components remain
unchanged. For example, `CssTimeLiteral::try_new("0.0001", CssTimeUnit::Milliseconds)`
emits `0s` while retaining `0.0001ms`. Node or byte limits produce a typed error
without partial CSS or input mutation.

Calculation emission uses [calculated number output](#calculated-number-output)
for finite components after the shared simplifier. Floating-point precision,
overflow and underflow limits stay unchanged, and the original checked graph
remains retained. The shared projection follows the documented
[calculated number arithmetic policy](#calculated-number-output). Contextual
resolution remains downstream.

`CssTransition::try_new` and `CssAnimation::try_new` accept the same typed
components as their property parsers. `CssAnimationComponents` is the animation
constructor's input assembly. Public construction rejects recovered time
components, empty aggregate items and empty lists. Recovering parsing can retain
implicitly closed math with diagnostics; clean validators reject those reports.
Checked property construction rejects the first original implicit closure for
all four time longhands and both timing shorthands, including pending envelopes.

Raw time literals, values and calculation trees compare original token spelling
and provenance. Duration wrappers, timing lists and transition/animation
aggregates compare their time children without source coordinates while keeping
lexical coefficients, units, branches, order and omissions distinct. Constructors
and parsers retain absent shorthand fields; they do not insert initial values.
Animation iteration counts and easing values use their own shared numeric models.
This crate does not resolve relative units, evaluate computed ranges, run animation
timelines, or lower values into sibling Surgeist crates. Whole timing aggregate
specified writers and timing property expansion remain unfinished.

### Exact frequency and ordinary resolution

`CssFrequencyLiteral` and `CssResolutionLiteral` retain a checked dimension token
with its exact coefficient, decoded unit and original provenance. Their
`try_new` constructors accept a coefficient string and a unit enum;
`try_from_component` accepts one checked dimension. Borrowed `numeric()`,
`component()` and `origin()` views expose the retained input without converting
it to a floating point value. Units are ASCII case-insensitive. Resolution's `x`
alias selects `CssResolutionUnit::Dppx` while retaining the authored `x` token.

Frequency accepts signed coefficients. Ordinary resolution accepts zero,
including signed or exponent zero, and rejects exact negative nonzero values.
Very small negative coefficients remain negative even if a floating point
conversion would underflow. Neither domain admits a bare number zero, a
percentage, an unrelated dimension or a NaN/infinity identifier as an ordinary
literal.

`CssFrequencyValue` and `CssResolutionValue` hold an ordinary literal or a
checked calculation in their named domain. `from_literal` retains a checked
literal; `try_from_calculation` rejects the first original implicit closure,
then normalizes an ordinary dimension root. Actual math remains a calculation:
`calc(-1dppx)` is symbolic, and its eventual range is not evaluated during
admission. The signed `CssResolutionCalculation` root also continues to serve
media-query syntax, whose negative operands and separate `infinite` keyword are
valid authored inputs.

```rust
use surgeist_css::{
    CssFrequencyLiteral, CssFrequencyUnit, CssResolutionLiteral, CssResolutionUnit,
};

let frequency = CssFrequencyLiteral::try_new("1.0000000000000001", CssFrequencyUnit::Kilohertz)
    .unwrap();
assert_eq!(frequency.numeric().representation(), "1.0000000000000001");
assert_eq!(frequency.serialize_specified().unwrap(), "1khz");

let resolution = CssResolutionLiteral::try_new("-0e999", CssResolutionUnit::Dppx).unwrap();
assert_eq!(resolution.numeric().representation(), "-0e999");
assert!(CssResolutionLiteral::try_new("-1e-999", CssResolutionUnit::Dpi).is_err());
```

Frequency literals and values provide bounded `serialize_specified()` and
`serialize_specified_with_limits()` helpers. Ordinary output retains the
selected unit with a lowercase `hz` or `khz` suffix and applies
[canonical number rounding](#canonical-ordinary-number-output). Signed or rounded
zero emits numeric zero. A value wrapper shares the same cumulative
input-node, projection-node and byte budget; failures return a typed error with
no partial CSS and leave the input unchanged. Raw equality preserves coefficient
and unit spelling, ordinary versus math branches, and original provenance.

This ordinary frequency output follows the selected WebKit behavior for
specified values: `1kHz` emits `1khz`. The selected
[CSSOM serialization clause](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-component-value)
leaves frequency's specified-versus-computed phase unresolved. Ordinary output
rounds to at most six fractional places; for example, `0.0000001Hz` emits `0hz`
while retaining its exact authored coefficient. Frequency math uses
[calculated number output](#calculated-number-output) for finite components
after the shared simplifier, retaining its floating-point precision and range
limits. This does not resolve the specified-versus-computed phase question.

`CssResolutionLiteral` and `CssResolutionValue` expose bounded specified
serialization in `dppx`, as selected by the
[CSSOM resolution component rule](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-component-value).
Ordinary conversion precedes the shared six-place number rounding: `192dpi`
emits `2dppx`, and `1dpcm` emits `0.026458dppx`. The exact factors are `1/96`
for dots per inch and `127/4800` for dots per centimeter. The `x` alias emits
`dppx`, and signed zero emits `0dppx`; coefficients, authored units and origins
remain unchanged. Conversion work and final suffix bytes consume one cumulative
typed resource budget, with no partial output on failure. Calculations reuse
the existing [calculated number output](#calculated-number-output) provider,
including its finite and structural simplification limits, without imposing
the ordinary literal range on their results or adding contextual evaluation.
Media queries retain their existing authored lexical serialization rather than
performing that conversion or evaluating their comparisons.

## Property-specific authored positions

Authored position values preserve symbolic offsets and expose both axes
without resolving percentages, calculations, writing modes, positioning boxes,
object sizes, layout, painting, or transforms. `CssSpecifiedLengthPercentage`
owns the checked numeric length-percentage domain. The enclosing horizontal and
vertical position variants retain whether an offset was free or authored against
a named edge.

The property grammars and accessors are deliberately distinct:

- `CssObjectPositionPropertyValue::position()` exposes one `CssPhysicalPosition`.
- `CssMaskPositionPropertyValue::positions()` exposes a nonempty `CssPhysicalPositionList`.
- `CssMaskPropertyValue::value()` exposes typed mask shorthand layers; each
  layer exposes its optional physical position through `CssMaskLayer::position()`.
- `CssBackgroundPositionPropertyValue::positions()` exposes a distinct
  nonempty layer list that additionally admits the background-only
  three-component form.
- `CssTransformOriginPropertyValue::origin()` exposes explicit horizontal and
  vertical axes plus an optional `CssSpecifiedLength`; the z component is a
  checked authored length and cannot contain a percentage.

`background-position` expands to one noninherited longhand whose intrinsic initial
is a single `0% 0%` layer. Ordinary contributions preserve the entire ordered
authored list, and CSS-wide keywords remain symbolic. Substitution-dependent values
retain a pending handle whose strict reentry checks the same property grammar and
preserves the original declaration's identity and importance.

`CssBackgroundPosition` and `CssBackgroundPositionList` expose
`serialize_specified()` and `serialize_specified_with_limits()`. They serialize
horizontal before vertical, make implied `center` explicit, and preserve the
omitted offset in background's three-component form: `bottom 20% left` becomes
`left bottom 20%`. Lists keep comma order. One cumulative input-node,
projection-node and byte budget covers every layer and numeric child; errors
return no partial text. Offsets remain signed and symbolic. Matching lists to
background image counts and resolving positioning geometry belong downstream.

```rust
use surgeist_css::{
    CssHorizontalPosition, CssKnownPropertyValueRef,
    parse_style_attribute,
};

let report = parse_style_attribute(concat!(
    "background-position: left 10px top; ",
    "object-position: right 5% bottom 2px; ",
    "transform-origin: left top 50px",
));
assert!(report.is_clean());

let CssKnownPropertyValueRef::BackgroundPosition(background) = report.syntax()[0]
    .known().expect("known background position")
    .property_value().expect("ordinary background position")
else { panic!("expected background-position") };
assert!(matches!(
    background.positions().positions()[0].horizontal(),
    CssHorizontalPosition::LeftOffset(offset)
        if matches!(offset.literal_component().unwrap().view(), surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "10" && unit == "px")
));

let CssKnownPropertyValueRef::ObjectPosition(object) = report.syntax()[1]
    .known().expect("known object position")
    .property_value().expect("ordinary object position")
else { panic!("expected object-position") };
assert!(matches!(
    object.position().horizontal(),
    CssHorizontalPosition::RightOffset(_)
));

let CssKnownPropertyValueRef::TransformOrigin(transform) = report.syntax()[2]
    .known().expect("known transform origin")
    .property_value().expect("ordinary transform origin")
else { panic!("expected transform-origin") };
assert!(matches!(transform.origin().z().unwrap().literal_component().unwrap().view(), surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "50" && unit == "px"));
```

For `transform-origin`, one planar token or a valid planar pair is accepted;
an authored Z length requires the pair. `left 50px` is a planar pair with no Z,
and `left top 50px` has a Z length. The selected parser rejects `top 50px`,
`bottom 0`, and their length-calculation equivalents. This follows the pinned
WebKit consumer at revision `73aa6c89e2cb77c46184a81aec944e4ab99d114d`
(`CSSPropertyParserConsumer+Position.cpp` and `CSSPropertyParserCustom.h`) and
the [Transforms 1 grammar](https://www.w3.org/TR/2019/CR-css-transforms-1-20190214/#transform-origin-property).
[CSS Values 4 §8.3.1](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#position)
explicitly gives the conflicting `top 50px` example as one planar value plus
Z. The source discrepancy remains tracked separately from the selected
operational behavior.

The position wrappers expose their checked semantic values directly.
`CssPhysicalPosition::try_new` accepts paired edge offsets or no edge offsets
and rejects axis-relative keywords;
`CssBackgroundPosition::try_new` additionally accepts one edge offset with a
keyword on the other axis, but not with a bare offset. The checked
`CssTransformOrigin::try_new` excludes edge offsets and keeps optional pure-length
Z. Position use
inside gradients, transforms, and basic shapes remains on its separate
function grammar boundary.

### Symbolic position families

`CssPosition` represents the full authored
[Values 5 position grammar](https://www.w3.org/TR/2024/WD-css-values-5-20241111/#position)
imported by circle and ellipse. `view()` returns a borrowed `CssPositionRef`
containing Cartesian, named-flow or relative-flow components. These families
retain coordinate identity without choosing a writing mode or reference box.

`CssCartesianPosition::try_new` checks horizontal and vertical components,
including x-start/x-end and y-start/y-end. Physical and axis-relative keywords
can mix within this family. Free length-percentages remain horizontal then
vertical; edge-relative offsets require both axes. `CssPosition::from_cartesian`
accepts that checked pair.

`CssPosition::try_from_named_axes` accepts distinct `CssBlockPosition` and
`CssInlinePosition` components. `try_from_relative_axes` accepts two
`CssRelativeAxisPosition` components in block then inline order. Each family
allows keyword/center pairs or two edge-offset components, with no center-offset
branch. All-center construction produces the same Cartesian center/center value
as parsed `center`. `CssPositionConstructionError` distinguishes unpaired edge
offsets from nonphysical keywords supplied to a physical-only constructor.

```rust
use surgeist_css::{
    CssBlockPosition, CssCircleRadius, CssCircleShape, CssHorizontalPosition,
    CssInlinePosition, CssPhysicalPosition, CssPosition,
    CssPositionConstructionError, CssPositionRef, CssVerticalPosition,
};

let position = CssPosition::try_from_named_axes(
    CssBlockPosition::Start, CssInlinePosition::End,
).unwrap();
let CssPositionRef::NamedFlow(flow) = position.view() else {
    panic!("named flow position");
};
assert_eq!(flow.block(), &CssBlockPosition::Start);
assert_eq!(flow.inline(), &CssInlinePosition::End);
let circle = CssCircleShape::new(CssCircleRadius::Default, Some(position));
assert_eq!(circle.serialize_specified().unwrap(), "circle(at block-start inline-end)");
assert_eq!(
    CssPhysicalPosition::try_new(
        CssHorizontalPosition::XStart, CssVerticalPosition::Bottom,
    ).unwrap_err(),
    CssPositionConstructionError::NonPhysicalKeyword,
);
```

Parsing admits Cartesian and named-flow one-, two- and four-component forms;
relative flow has two or four components. Named axis pairs can reorder, while
unqualified relative pairs keep block/inline order. Bare `start` or `end`, mixed
coordinate families, duplicate axes and generic three-component positions are
invalid. Background's three-component form remains its own physical grammar.
Older gradients, object and mask positions use the checked physical restriction
in both parsing and Rust construction.

Specified serialization emits Cartesian horizontal/vertical order and symbolic
flow block/inline order, preserving keyword families and offsets. An implied
center becomes explicit. The pinned draft's two-component serialization wording
does not explain unresolved flow-axis order; this operational choice preserves
meaning without guessing physical orientation. The source wording question
remains separate from implemented authored grammar. Computed left/top offsets
and writing-mode mapping belong downstream.

Full and physical positions share the cumulative serializer: one position
aggregate, two axis nodes and each authored numeric provider. Keyword pairs cost
three input and projection nodes; paired literal offsets cost five. Checked
wrappers and borrowed views add no nodes. Symbolic math uses the existing
provider costs, and all children share the same UTF-8 byte budget. Typed limit
failure returns no partial output or input/provenance mutation.

## Dedicated authored function grammars

Property accessors expose dedicated typed function families. Timing-function
wrappers expose the sole `CssEasingList` through `timing_functions()`.
`transform.value()` returns `CssTransform`, `filter.value()` and
`backdrop-filter.value()` return `CssFilter`, `box-shadow.value()`
returns `CssBoxShadow`, and `clip-path.value()` returns `CssClipPath`.

```rust
use surgeist_css::{
    CssBasicShape, CssClipPath, CssFilterFunction, CssFilter,
    CssKnownPropertyValueRef, CssTransformFunction, CssTransform,
    parse_style_attribute,
};

let report = parse_style_attribute(concat!(
    "transform: translate3d(10%, 2px, 4em) rotate(45deg); ",
    "filter: blur(2px) drop-shadow(red 1px 2px 3px); ",
    "clip-path: polygon(round 2px, 0 0, 100% 0)",
));
assert!(report.is_clean());

let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
    .known().expect("known transform")
    .property_value().expect("ordinary transform")
else { panic!("expected transform") };
assert!(matches!(
    transform.value(),
    CssTransform::Functions(functions)
        if matches!(functions.functions()[0], CssTransformFunction::Translate3d(_))
));

let CssKnownPropertyValueRef::Filter(filter) = report.syntax()[1]
    .known().expect("known filter")
    .property_value().expect("ordinary filter")
else { panic!("expected filter") };
assert!(matches!(
    filter.value(),
    CssFilter::Functions(functions)
        if matches!(functions.functions()[1], CssFilterFunction::DropShadow(_))
));

let CssKnownPropertyValueRef::ClipPath(clip) = report.syntax()[2]
    .known().expect("known clip path")
    .property_value().expect("ordinary clip path")
else { panic!("expected clip-path") };
assert!(matches!(
    clip.value(),
    CssClipPath::BasicShape(shape)
        if matches!(shape.shape(), CssBasicShape::Polygon(polygon)
            if polygon.round().is_some())
));
```

The typed transform family covers the selected two-dimensional Transforms 1
functions and the selected three-dimensional subset with exact arity,
separator, and dimension domains.
The transform graph uses `CssTransform::None` or a nonempty
`CssTransformFunctionList`. Each `CssTransformFunction` retains its typed
operands and `kind()` discriminator. Public fixed-field constructors accept
checked matrix, rotation, scale, skew, and translation operands, preserving
omitted second operands and symbolic calculations. `translate3d` keeps a pure
length on Z while X and Y accept length-percentages; `perspective` keeps a
checked nonnegative length or `none`. Matrix operands, rotate3d axes, scale
numbers, and cubic-bezier coordinates use `CssSpecifiedNumber`; three-dimensional
scale percentage branches use `CssSpecifiedPercentage`. The three-dimensional
scale3d/scaleZ grammar also admits `CssTransformScaleComponent::HintedNumberCalculation`,
retaining a Number result whose percentage basis remains unresolved. The selected
Level 1 scale/scaleX/scaleY grammar still uses pure Numbers. These payloads retain exact token
spelling and provenance, or genuine symbolic calculations. Numeric aggregate
equality compares exact structure while ignoring numeric source origins; direct
scalar equality retains provenance. The independent `scale` property accepts
one to three ordinary shared numbers and retains its literal-only subset.
Transform evaluation remains downstream.

The individual `rotate` property exposes `CssRotate::None` or
`CssRotate::Value(CssRotateValues)`. `CssRotateValues::new` takes a checked
`CssAngleValue` and an optional `CssRotateAxis`: X, Y, Z, or an exact three-number
vector. Omission retains the implicit positive z axis; zero vectors remain valid
authored syntax. Parsed keyword axes retain their original provenance through
`keyword_axis_origin()`, while checked keyword construction uses programmatic
provenance and vector operands retain their own origins. Structural value equality
ignores source origins and preserves the authored axis choice and numeric structure.

The property admits its axis group before or after the angle, including typed
angle and number calculations. Its strict angle grammar accepts `0deg` and
rejects bare `0`; transform-function zero-angle grammar retains its separate
`CssAngleOrZero` contract. This authored model replaces the former String payload.
Canonical individual-rotate specified output and contextual rotation execution
remain separate work; the model does not reconstruct output from authored text.

`CssAngleLiteral` retains an exact decimal dimension, its authored unit, and
original component provenance. `CssAngleValue` holds a checked literal or symbolic
`CssAngleCalculation`; it excludes bare number zero. Transforms, hue-rotate
filters, and linear-gradient directions use `CssAngleOrZero`, whose zero branch
holds a checked `CssZeroLiteral`. Image orientation uses the strict angle type.
Tiny nonzero numbers never qualify as zero. Raw scalar equality retains
provenance; semantic aggregates compare structure while ignoring scalar origins.
Ordinary angle serialization retains the selected unit and applies
[canonical number rounding](#canonical-ordinary-number-output) without changing
the stored coefficient. Calculation serialization uses
[calculated number output](#calculated-number-output) after the existing
simplified projection. The accepted shared projection uses the documented
binary64 arithmetic policy; it does not require arbitrary-precision evaluation.
Contextual resolution remains downstream. Each context retains its grammar and
omission rules.

```rust
use surgeist_css::{
    CssAngleLiteral, CssAngleOrZero, CssAngleUnit, CssAngleValue,
    CssComponentValue, CssFilterFunction, CssFilterHueRotate, CssZeroLiteral,
};

let literal = CssAngleLiteral::try_new("0.10000000000000000001", CssAngleUnit::Degrees)?;
assert_eq!(literal.numeric().representation(), "0.10000000000000000001");
let angle = CssAngleValue::from_literal(literal);
let rotation = CssFilterFunction::HueRotate(CssFilterHueRotate::new(
    CssAngleOrZero::Angle(angle),
));
assert_eq!(rotation.serialize_specified()?, "hue-rotate(0.1deg)");

let zero = CssZeroLiteral::try_from_component(CssComponentValue::try_number("-0e999")?)?;
let rotation = CssFilterFunction::HueRotate(CssFilterHueRotate::new(
    CssAngleOrZero::Zero(zero),
));
assert_eq!(rotation.serialize_specified()?, "hue-rotate(0)");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Easing values distinguish keywords, `cubic-bezier()`, and `steps()`.
`CssCubicBezierX` checks an ordinary exact number against inclusive [0, 1],
including signed zero and exponent spelling. A bare calculation root reenters
that check, while genuine function calculations remain symbolic. Y coordinates
remain unrestricted signed numbers; coordinate order is preserved.
`steps()` stores its count as the shared `CssPositiveIntegerValue`: an exact
`CssPositiveIntegerLiteral` or symbolic `CssIntegerCalculation`.
`CssSteps::try_new` validates bare calculation tokens as ordinary literals and
rejects literal one only with `JumpNone`. Genuine function calculations remain
symbolic even when their authored expression is `calc(1)`. Ordinary zero,
negative, decimal-point and exponent counts are rejected. Position omission
remains distinct from explicit `end` or `JumpEnd`.

`CssEasing`, `CssEasingList`, `CssCubicBezier`, and `CssSteps` expose
`serialize_specified()` and `serialize_specified_with_limits()`. Their canonical
output follows [CSS Easing Functions Level 1 §2.4](https://www.w3.org/TR/2023/CRD-css-easing-1-20230213/#serialization):
the five non-step keywords remain keywords, `step-start` becomes
`steps(1, start)`, and `step-end` becomes `steps(1)`. Explicit `end` and
`jump-end` positions are omitted; `start` and `jump-start` retain their distinct
spellings. Cubic coordinates retain their order and use the shared Number
formatter; counts use the existing exact Integer and Integer-calculation owner.
Specified calculations remain calculations, without coordinate clamping,
positive-count clamping, integer rounding, or an assumed relative-unit basis.
The selected publication does not admit `linear()` or `spring()` functions.

One cumulative serialization context covers every list member and numeric
child. A list, cubic function, or steps function charges one input and one
projection node; enum dispatch adds no wrapper charge. Each stored position
charges one input node, and an emitted position charges one projection node.
A keyword, including a step alias's canonical replacement, charges one of each.
Numeric children retain their shared owner's visits and projection work; bytes
are the actual canonical output. Errors return no partial string and leave
authored components, origins, omissions, and position identities unchanged.
Timing evaluation and generic Transition/Animation property expansion remain
separate capabilities.

Filter amounts use `CssFilterAmount::Default`, `Number` or `Percentage`.
The scalar branches hold `CssSpecifiedNonNegativeNumber` and
`CssSpecifiedNonNegativePercentage` directly, preserving ordinary spelling,
numeric token kind, signed zero and origins. Tiny negative nonzero literals are
rejected exactly; amounts above one or 100% remain authored without clamping.
Genuine function math stays symbolic. All seven amount-function identities,
omission and list order are preserved for both filter properties.

Animation iteration counts use `Infinite` or
`Number(CssSpecifiedNonNegativeNumber)`, including symbolic number math.
Fractional and zero ordinary counts remain valid, and an omitted shorthand count
stays omitted. The nonempty count list retains order.

Border-image slice uses shared nonnegative `Number` and `Percentage`
payloads with a separate `fill` flag. Width keeps `Auto`,
`LengthPercentage` and `Number`; outset keeps `Length` and
`Number`. Slice and width also retain contextual Number results in distinct
`HintedNumberCalculation` branches. Their shared numeric capture preserves
the percentage hint and symbolic basis, including comparisons before edge
compression. Ordinary negative inputs are rejected; typed math defers its
range handling. Outset retains its pure Number restriction.
Each list expands one to four authored components using CSS edge
rules; the shorthand retains coupled member requirements and omission. Intrinsic
initials are programmatic `0` for outset, `100%` for slice and `1` for width.
The model retains four effective edges rather than original edge arity.
`CssBorderImage` and its slice, width, outset, and repeat components provide
`serialize_specified()` and `serialize_specified_with_limits()`.

Following [CSSOM's component selection order](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-value),
slice, width, and outset select one to four edges by exact ordinary values before
rounding their emitted coefficients. Equivalent spellings such as `+1.0` and
`1e0` can compress; different values that happen to round alike retain their
edges. Units and component branches remain distinct. A unitless length zero
in width or outset emits `0px`, preserving its length branch. Calculations
retain their existing canonical projected-text comparison and stored graph.
Every effective edge consumes its existing node budget, including compressed
edges; byte limits apply to the selected output.

```rust
use surgeist_css::{CssKnownPropertyValueRef, parse_style_attribute};

let report = parse_style_attribute(
    "border-image: .12345641 .12345642 .12345643 .12345644",
);
assert!(report.is_clean());
let CssKnownPropertyValueRef::BorderImage(value) = report.syntax()[0]
    .known().expect("known border image")
    .property_value().expect("ordinary border image")
else { panic!("expected border-image") };
assert_eq!(
    value.border_image().serialize_specified()?,
    "0.123456 0.123456 0.123456 0.123456",
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

These scalar aggregates compare exact numeric structure while ignoring numeric
origins. Direct shared nonnegative Number equality retains provenance; direct
shared nonnegative Percentage equality retains its established origin-insensitive
policy, including font-width and font-face consumers. Branches, nonnumeric fields,
order and omission remain part of aggregate identity.

`CssShadow` retains two signed offsets, optional nonnegative `blur_radius()`,
optional signed `spread_radius()`, optional color, and inset. Its checked
constructor rejects spread without blur. `CssBoxShadowList::try_new` rejects
an empty vector. `CssBoxShadow` distinguishes `None` from a nonempty list.
The [Backgrounds 3 CRD 2024-03-11 §6.1](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#box-shadow)
grammar permits whole color, length, and inset groups in any order; the two
mandatory offsets and optional blur/spread must remain consecutive. A color or
inset cannot interrupt those lengths. Box-shadow is a noninherited terminal with
`CssBoxShadow::None` initial. Central expansion emits one ordinary or CSS-wide
contribution, pending substitution retains its declaration, and strict reentry
and normalization share the same grammar and occurrence provenance.

`CssDropShadow` uses a separate model with two signed offsets, optional checked
nonnegative `standard_deviation()`, and optional color. The
[Filter Effects 1 WD 2018-12-18 §6.1](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/#funcdef-filter-drop-shadow)
third length is the standard deviation of the blur, distinct from box-shadow's
blur radius. Two or three lengths form one consecutive group; color can precede
or follow the complete group. Both filter and backdrop-filter use this grammar.
Drop shadows cannot contain inset or spread. Filter lists preserve URL/function
order and typed function-specific operands.

All four shadow models provide `serialize_specified()` and
`serialize_specified_with_limits(CssSpecifiedValueSerializationLimits)`.
A bare shadow emits `[color] x y [blur [spread]] [inset]`; a list separates
members with comma-space, `None` emits `none`, and a drop shadow emits the full
`drop-shadow([color] x y [standard_deviation])` function. Authored omissions
remain omitted; explicit zero and currentcolor remain present. Child providers
retain exact authored numeric values and origins without contextual resolution.
Ordinary coefficients use the numeric provider's six-place canonical decimal
output: exponents may expand and rounded zero emits zero, while the input
retains its authored spelling. Symbolic math and colors use their existing
canonical providers. Canonical math may reorder
terms without changing the stored authored expression.

Each shadow or drop shadow charges one aggregate input and projection node,
and inset charges one extra keyword node in each budget. A list charges one
aggregate plus every member; `CssBoxShadow::Shadows` delegates transparently.
`None` charges one input and projection node. Every present child, including an
explicit default, uses its existing numeric or color provider's charges.
Wrappers and separators cost bytes only. One cumulative context owns the entire
call: dropping a child math arena does not refund work. Limit failures use
`CssSpecifiedValueSerializationError` and return no partial text or input
mutation.

`CssFilterHueRotate::new(angle)` retains an explicit shared angle.
`CssFilterHueRotate::omitted()` stores an effective `0deg`; `angle()` borrows that
effective value and `authored_angle()` returns `None` only for omission. Omission,
unitless zero and explicit `0deg` are distinct. Its equality preserves the shared
angle's provenance-sensitive calculation equality. `CssFilterBlur` exposes the
parallel `length()` and `authored_length()` contract, with effective omitted `0px`.

`CssFilter`, `CssFilterFunctionList` and `CssFilterFunction` provide
`serialize_specified()` and `serialize_specified_with_limits(...)`. Lists retain
their authored order and separate complete functions with one space; `None`
emits `none`. Omitted blur, hue and amount arguments emit empty parentheses;
explicit defaults remain present. Drop shadows reuse their complete function
serializer and URL branches retain their function identity, target and modifiers.
Ordinary angle literals retain their finite precision and units; calculations
reuse the shared numeric projection, including its existing unit projection.
Serialization introduces no filter-specific clamping or angle normalization.
All seven amount functions admit `CssFilterAmount::HintedNumberCalculation`
alongside pure Number and Percentage. This checked root retains its percentage
basis and shares the existing projection, equality and cumulative resource
contract. Ordinary amounts remain nonnegative; math range handling is deferred.

One list aggregate plus every function and authored child share a cumulative
input, projection and UTF-8 byte budget. Non-drop functions charge one aggregate;
URL and drop branches delegate their existing aggregate providers transparently.
Stored effective defaults are not authored children and incur no synthetic child
charge. Literal angles and unitless zero charge one input and projection node;
calculations delegate their existing visited-input and allocated-projection costs.
Wrappers and separators cost bytes. Failure returns no partial CSS or mutation.

Both filter properties are noninherited terminals with intrinsic `CssFilter::None`
initials. Central expansion, pending reentry and normalization retain occurrence,
importance and replacement provenance. The selected
[Filter Effects 1 WD 2018-12-18](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/)
sections 5, 6.1 and 6.3 own shared grammar, omitted defaults and specified ordering.
`X-BACKDROP-FILTER` imports only the named property from the
[immutable exploring draft](https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/Overview.bs),
whose moving Level 1 dependency is resolved to that selected publication.
The raw draft has no canonical-order row and does not have Working Group consensus.
This source refinement preserves the historical repository snapshot as evidence;
it selects no full Level 2 module, rendering or backdrop-root algorithm.

```rust
use surgeist_css::{CssFilter, CssFilterFunction, CssFilterFunctionList, CssFilterHueRotate};

let hue = CssFilterHueRotate::omitted();
assert!(hue.authored_angle().is_none());
let functions = CssFilterFunctionList::try_new(vec![CssFilterFunction::HueRotate(hue)]).unwrap();
assert_eq!(CssFilter::Functions(functions).serialize_specified().unwrap(), "hue-rotate()");
```

### Authored clipping shapes

The authored `clip-path` grammar follows
[Masking 1 CRD 2021-08-05 §5.1](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#the-clip-path)
and the selected
[Shapes 1 CRD 2025-06-12 functions](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/#supported-basic-shapes).

`CssClipPath` distinguishes `None`, `Url`, a standalone `GeometryBox`, and
`BasicShape(CssClipPathShape)`. The checked composition stores one `CssBasicShape`
and an optional reference box. `shape()` borrows the shape; `reference_box()`
returns the optional box keyword. `CssBoxEdgeKeyword` supplies content-box,
padding-box, border-box,
margin-box, fill-box, stroke-box and view-box. Parsed shape/box pairs accept either
order, while duplicate boxes, multiple shapes and combinations with none or URL
are invalid. An omitted box remains distinct from explicit border-box because
its contextual interpretation belongs to style and shape processing.

The supported functions are `inset()`, `rect()`, `xywh()`, `circle()`, `ellipse()`,
`polygon()`, `path()` and `shape()`.
Inset retains one to four authored signed length-percentage offsets and optional
checked border radii. Polygon retains optional fill rule, optional signed pure
rounding length and a nonempty ordered point list. The fill rule must precede
`round <length>` when both appear; percentages are not rounding lengths. Negative
specified rounding lengths remain authored values. Used rounding geometry and
its clamp require downstream shape context.

`CssRectShape` stores four independent `CssRectShapeEdge` values in
top/right/bottom/left order. Each is `Auto` or a checked signed
length-percentage. Its constructor composes those edges and optional
`CssBorderRadiusShorthand`; `top()`, `right()`, `bottom()`, `left()` and `round()`
borrow the authored components. `CssXywhShape` instead stores signed x/y offsets
and checked nonnegative width/height, exposed by the corresponding named getters.
Both preserve absent rounding separately from explicit `round 0`, including
horizontal and vertical radius arities and slash omission.

Parsing requires exactly four coordinates, with optional `round` radii after
them. Adjacent distinguishable percentage tokens and comment-only separators
are valid. Commas, nonzero unitless coordinates, `auto` in `xywh()`, and negative
ordinary width/height are invalid. Mathematical components remain symbolic.
Authored rectangles retain their function identity and exact components:
auto substitution, crossed-edge correction, percentage resolution and conversion
to computed `inset()` require downstream context. The deprecated `clip` property's
pure-length rectangle grammar has its own model.

`CssPathShape` composes an optional `CssFillRule` and checked `CssPathData`.
The shared fill keyword is `Nonzero` or `Evenodd`; omission stays distinct from
explicit nonzero in polygon, path and shape models. Path syntax accepts one quoted
string, preceded by an optional fill keyword and comma. Without that keyword,
the comma is absent.

`CssPathData::try_new` checks decoded SVG text and gives it programmatic origin.
`try_from_component` requires one complete quoted-string component and retains
its original token origin. `as_str()` borrows the decoded text; `origin()` borrows
its provenance. Equality compares decoded bytes independently of provenance:
CSS escapes and quote styles can produce equal data, while different SVG number
spellings, command cases and separators remain distinct.

The complete nonempty string must follow the imported
[SVG 1.1 path-data grammar](https://www.w3.org/TR/2011/REC-SVG11-20110816/paths.html#PathDataBNF).
All command families, repeated groups, subpaths and maximal decimal/exponent
spellings are checked without float conversion. Move-only paths and degenerate
segments are valid. Empty or SVG-whitespace-only data, incomplete groups,
malformed suffixes, invalid separators and non-ASCII numeric syntax are invalid.
SVG whitespace is space, tab, carriage return and line feed.

Signed and zero arc radii remain authored values under the
[Appendix F.6.2 interpretation](https://www.w3.org/TR/2011/REC-SVG11-20110816/implnote.html#ArcOutOfRangeParameters),
corroborated by the pinned WebKit SVG path consumer. Arc flags remain single ASCII
`0` or `1`, following the BNF and that consumer. These choices resolve differing
requirements within the pinned SVG publication without taking absolute radii,
converting arcs or importing drawing-prefix recovery. Geometry remains downstream.

`CssPathDataConstructionError` distinguishes `ExpectedString`, `RecoveredInput`,
`EmptyPath` and `InvalidPathData`. Grammar failures report a decoded-byte offset,
including decoded length for an incomplete suffix; that offset is separate from
the original CSS token's source coordinates. There is no path-specific constructor
byte or command-count cap. Component parsing and output retain their existing
explicit limits.

`CssShapeFunction` stores an optional fill rule, a starting `CssPosition` and a
checked nonempty `CssShapeCommandList`. The ordered `CssShapeCommand` variants
are Move, Line, HorizontalLine, VerticalLine, Curve, Smooth, Arc and Close.
Absolute `to` endpoints use the full position grammar; relative `by` endpoints
use two signed length-percentages. Horizontal and vertical lines accept only
their corresponding axis keyword or one offset. Close-only and move-only lists
are valid authored syntax.

A comma is required between the start and first command and between commands.
This follows the pinned
[Shapes examples](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/#shape-examples)
and WebKit Shapes consumer at revision
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`; the
[printed outer production](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/#funcdef-basic-shape-shape)
omits that first comma. The start and absolute endpoints retain full Values 5
positions despite the explanatory prose's narrower coordinate-pair wording.

`CssShapeCurve::to` and `CssShapeCurve::by` couple the endpoint with its absolute or relative
controls. A curve has one required control and an optional second; a smooth
command has zero or one. Absolute controls accept a full position, while an
explicit start/end/origin anchor requires a numeric coordinate pair. Relative
controls contain a pair and optional anchor. Omitted anchors remain stored as
omissions: downstream geometry interprets absolute controls from Origin and
relative controls from Start. Explicit anchors and control arities stay distinct.

`CssShapeArc` requires an endpoint and one or two signed length-percentage radii.
Optional sweep, size and strict angle remain omitted or explicit as authored.
Parsing accepts the option groups in any order, once each; canonical output
orders radii, sweep, size and rotation. A `rotate` keyword requires its angle,
and bare number zero is invalid there. Negative or zero radii remain authored
values; radius correction, reflection and path construction require geometry
context.

```rust
use surgeist_css::{
    CssPosition, CssRelativeAxisPosition, CssShapeCommand, CssShapeCommandList,
    CssShapeFunction,
};

let start = CssPosition::try_from_relative_axes(
    CssRelativeAxisPosition::Start, CssRelativeAxisPosition::End,
).unwrap();
let commands = CssShapeCommandList::try_new(vec![CssShapeCommand::Close]).unwrap();
let shape = CssShapeFunction::new(None, start, commands);
assert!(shape.fill_rule().is_none());
assert_eq!(shape.commands().commands().len(), 1);
assert_eq!(shape.serialize_specified().unwrap(), "shape(from start end, close)");
```

Ordinary CSS parsing can retain a missing string quote or function parenthesis
with recovery diagnostics. Clean validation rejects that report. Checked
`clip-path` construction and replacement reentry reject recovered components
before reserialization can conceal the missing delimiter; the same rule applies
to every clip-path alternative.

`circle()` retains one nonnegative length-percentage radius, an omitted radius,
radial extent keywords, and optional `at <position>`. Percentage and symbolic
length-percentage radii are checked authored values; two radii are invalid.
This operational choice follows the pinned WebKit Shapes consumer
(`CSSPropertyParserConsumer+Shapes.cpp`, revision
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`). It resolves a source
discrepancy: [CSS Shapes 1](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/#funcdef-basic-shape-circle)
imports Images 3 `<radial-size>` and excludes its two-radius branch, while that
imported [Images 3 production](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/#typedef-radial-size)
does not itself provide a single percentage branch. The source discrepancy
remains tracked separately from this authored parser behavior.

`CssEllipseShape` stores an optional `CssEllipseRadii` pair. `radii()` returns
`None` only when the radii were omitted. Each horizontal or vertical
`CssEllipseRadius` independently contains a checked nonnegative length-percentage
or `CssRadialExtent`; numeric/extent mixtures and different extents are valid.
Every lone explicit radius is invalid. The operational pair rule follows the
same pinned WebKit Shapes consumer and resolves the imported radial-size context
discrepancy without inserting its effective defaults into authored storage.

Circle and ellipse positions use the full `CssPosition`. Shapes imports the
[Values 5 WD 2024-11-11 position definition](https://www.w3.org/TR/2024/WD-css-values-5-20241111/#position);
physical, axis-relative, named-flow and relative-flow forms remain symbolic.
All eight basic-shape functions and clip-path expose Complete authored grammar
support. This classification does not establish contextual geometry or exact
mathematical projection: calculations retain their authored graph, while their
canonical emission uses the existing numeric simplifier and its bounded
precision and range. The
[catalog](../specs/catalog.json) pins the required position definition and its
serialization clauses, plus the SVG path grammar and parsing definitions.
These imports select neither Values 5 nor SVG in full.

`CssClipPath`, `CssClipPathShape`, `CssBasicShape` and each supported shape struct
provide `serialize_specified()` and `serialize_specified_with_limits(...)`.
Canonical output uses lowercase function names, spaces and comma-space point
separators. A shape precedes its optional reference-box keyword regardless of
parsed order. Authored offset and radius arities, optional fields, explicit
extents, explicit nonzero fill and round zero remain present or omitted as
authored. Positions use their family's canonical axis order. Numeric and math
providers retain exact ordinary magnitudes, units and symbolic values; the
original parsed components and their origins remain unchanged.
Path output uses a canonical double-quoted CSS string with required escaping,
preserving the internal decoded SVG text, whitespace and numeric spelling.

```rust
use surgeist_css::{
    CssBasicShape, CssBoxEdgeKeyword, CssCircleRadius, CssCircleShape,
    CssClipPath, CssClipPathShape, CssSpecifiedValueSerializationLimits,
};

let circle = CssCircleShape::new(CssCircleRadius::Default, None);
let shape = CssClipPathShape::new(
    CssBasicShape::Circle(circle), Some(CssBoxEdgeKeyword::BorderBox),
);
assert_eq!(shape.reference_box(), Some(CssBoxEdgeKeyword::BorderBox));
let clip = CssClipPath::BasicShape(shape);
let expected = "circle() border-box";
let limits = CssSpecifiedValueSerializationLimits::new(3, 3, expected.len());
assert_eq!(clip.serialize_specified_with_limits(limits).unwrap(), expected);
```

```rust
use surgeist_css::{
    CssComponentValue, CssRectShape, CssRectShapeEdge,
    CssSpecifiedLengthPercentage, CssSpecifiedValueSerializationLimits,
};

fn edge(css: &str) -> CssRectShapeEdge {
    let value = CssSpecifiedLengthPercentage::try_from_component(
        CssComponentValue::try_token(css).unwrap(),
    ).unwrap();
    CssRectShapeEdge::LengthPercentage(value)
}

let rect = CssRectShape::new(
    CssRectShapeEdge::Auto, edge("1px"), edge("2%"), edge("0"), None,
);
assert_eq!(rect.top(), &CssRectShapeEdge::Auto);
assert!(rect.round().is_none());
let expected = "rect(auto 1px 2% 0)";
let limits = CssSpecifiedValueSerializationLimits::new(5, 5, expected.len());
assert_eq!(rect.serialize_specified_with_limits(limits).unwrap(), expected);
```

```rust
use surgeist_css::{CssPathData, CssPathShape, CssSpecifiedValueSerializationLimits};

let data = CssPathData::try_new("M0 0L1 2").unwrap();
let path = CssPathShape::new(None, data);
assert!(path.fill_rule().is_none());
assert_eq!(path.data().as_str(), "M0 0L1 2");
let expected = "path(\"M0 0L1 2\")";
let limits = CssSpecifiedValueSerializationLimits::new(2, 2, expected.len());
assert_eq!(path.serialize_specified_with_limits(limits).unwrap(), expected);
```

Each shape function charges one input and projection aggregate. Inset additionally
charges an offset-list aggregate plus authored scalar providers; round radii use
the existing border-radius aggregate and its authored scalar providers. Rect and
xywh charge their four scalar providers; an auto edge costs one keyword leaf.
There is no additional coordinate-list aggregate. Their literal four-component
forms cost five input and projection nodes standalone, six in a clip-path
composition, and seven with an explicit reference box. Optional round radii
delegate to the same existing border-radius provider. Circle
charges its explicit radius provider or extent leaf and optional existing
position provider. Ellipse adds a pair aggregate only for explicit radii and
charges its two independent components. Polygon adds optional fill and round
providers, one point-list aggregate, and each point aggregate with its two
coordinates. Composition adds one aggregate and one leaf for an explicit box;
path charges its decoded-string leaf and optional explicit fill keyword, costing
two nodes standalone without fill and three with fill. Quoting and escaping
charge the same cumulative byte budget.
Shape charges its from keyword, starting position, command list, each command
and its authored endpoint, controls and options through the same parent writer.
`shape(from 0px 0px, close)` costs nine input and projection nodes and 26 output
bytes standalone. Its clip-path composition adds one node; an explicit reference
box adds one more. Individual scalar and math providers retain their own charges.
`CssBasicShape` and the enclosing BasicShape enum branch delegate transparently.
Standalone none and geometry boxes cost one node in each budget; URL delegates
its existing provider. Omitted children add no synthetic defaults. Wrappers and
separators cost bytes only.

All siblings share one cumulative input, projection and UTF-8 byte budget,
including numeric math arenas, positions and border radii. Limit failure returns
`CssSpecifiedValueSerializationError`, no partial CSS and no input or provenance
mutation. `clip-path` is a noninherited terminal with intrinsic `None` initial;
central expansion, pending substitution reentry and normalization retain
importance, order, source occurrence and replacement provenance.

These are authored syntax values. This crate does not multiply transform
matrices, interpolate or evaluate easing, render shadows or filters, resolve
URLs, compute shape geometry, perform layout or painting, or lower values into
sibling crates. `transition` and `animation` retain their own explicit Partial
catalog boundaries.

## Authored colors

`CssColor` is the sole authored semantic color graph. It distinguishes named,
transparent, `currentcolor`, hexadecimal, current and deprecated system,
legacy and modern RGB/HSL, HWB, Lab/LCH, Oklab/Oklch, and predefined
`color()` branches. The color-bearing wrappers expose `value()`; aggregates
such as `CssBorder`, `CssTextDecoration`, `CssOutline`, `CssShadow`, and
`CssDropShadow` borrow their optional color through `color()`. `CssBackground`
retains its ordered layers and the final layer's optional color. The filter
wrappers expose the sole `CssFilter` function list through `value()`.

Ordinary number, percentage, and angle components hold checked
`CssColorNumberLiteral`, `CssColorPercentageLiteral`, and the shared
`CssAngleLiteral` values. Each keeps the exact finite decimal token and
origin, even when its magnitude overflows or underflows a binary32 cache. The
angle also retains its authored unit. `CssColorComponent` has number,
percentage, `none`, and typed calculation branches; `CssColorHue` has number,
angle, `none`, and calculation branches. Modern RGB/HSL/HWB accept their
selected mixed domains and `none`. Legacy comma RGB requires homogeneous
number or percentage channels without `none`; legacy HSL requires percentage
saturation/lightness and no missing component. Out-of-range authored channels
remain valid. Pure color math is retained symbolically until the appropriate
computed-value phase.

`CssHexColor::try_new` checks decoded hex digits, and
`CssNamedColor::try_new` checks the named-color vocabulary. Each ordinary color
family has a public checked `try_new`; `CssColor::from_*` composes checked
payloads. `CssColor::current_color()` and `transparent()` construct the two
keyword branches. The parser uses the same checked payload constraints.

### Parsed alpha values

`CssRgbColor`, `CssHslColor`, `CssHwbColor`, `CssLabColor`, `CssLchColor`, and
`CssPredefinedColor` expose `parsed_alpha()` alongside their authored `alpha()`
accessors. The Lab and LCH payloads also serve Oklab and Oklch. This immutable
view applies [Color 4's parsed alpha rules](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#alpha-syntax)
to both parsed input and checked Rust construction; it does not replace the
authored component or its provenance.

`CssParsedColorAlphaRef` distinguishes omitted opaque alpha, explicit missing
alpha, a direct scalar, and retained number or percentage calculations. An
omitted alpha remains distinct from explicitly supplying one, and `none` remains
distinct from zero. Calculations borrow their authored trees without evaluation
or range clamping.

`CssColorAlphaScalarRef` exposes exact `is_zero()`, `is_one()`, and
`is_interior()` observations. Negative or zero direct values project to positive
zero; numbers at or above one and percentages at or above 100 project to one.
Other positive values are interior. Classification uses the original decimal
digits and exponent without floating-point rounding or exponent expansion.
`authored_component()` and `origin()` retain access to the original domain,
spelling and source identity.

`as_unit_f64()` provides a finite approximate value in `0..1`, with percentage
scaling owned by CSS. Ordinary binary64 rounding and underflow can turn an exact
interior value into zero or one; its exact classification stays interior.
This accessor does not promise correctly rounded exact-real arithmetic.
Structural expansion and normalization preserve payloads exposing the same
view. Specified serialization retains its existing role-dependent calculation,
Origin, rounding and resource contracts.

### Keyword values and specified serialization

`CssColor::keyword_srgba8()` exposes the intrinsic meaning of the 148 opaque
named colors from [Color 4 §6.1](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#named-colors)
and [transparent black](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#transparent-color).
It returns encoded sRGB bytes in `[red, green, blue, alpha]` order, with straight
alpha: opaque named colors have alpha 255, and `transparent` gives `[0, 0, 0, 0]`.
The checked named-color constructor captures the fixed channels alongside the
lowercase name. Aliases retain distinct authored names and specified text even
when their channels agree.

Other representations return `None`, including hexadecimal and function colors
whose numerical meaning may also be fixed. Absence means outside this keyword
projection, not invalid or unresolvable. The method preserves the graph and
does not select a host palette, bind a profile, or evaluate contextual colors.

`CssColor::to_specified_css()` and `to_specified_css_with_limits()` emit one
canonical specified string under cumulative input-node, projection-node, and
UTF-8 byte limits. Failure is atomic and leaves the graph and origins intact.
This serializer preserves unresolved context and performs only selected pure
HSL/HWB and exact scalar conversions. It does not bind profiles, evaluate
relative channels or a mix, or gamut-map a color.
Any directly missing component, including alpha, selects a form preserving
`none`: ordinary RGB emits normalized `color(srgb ...)`, while HSL and HWB
retain their named functions. Direct HSL/HWB channels emit percentages and a
bare degree hue. Resolved ordinary RGB/HSL/HWB non-alpha calculations emit scalars under the
[calculated number policy](#calculated-number-output); contextual slots retain
their calculation trees. RGB channels clamp to their output domain: 0..255 in
legacy `rgb()`, or 0..1 after number/255 or percentage/100 scaling in the
missing-preserving form. RGB NaN and negative infinity become zero; positive
infinity becomes the upper endpoint.
HSL saturation has a zero minimum and no upper bound. The selected eager
normalization also applies to resolved negative, NaN and negative-infinite
saturation calculations, following the
[frozen WebKit parser interpretation](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSPropertyParserConsumer%2BColor.cpp#L187).
Named HSL lightness and HWB whiteness/blackness remain unbounded; NaN becomes
zero, while dimensional infinities retain valid `calc(infinity * 1%)` or
`calc(-infinity * 1%)` text. Numeric hue normalizes to degrees modulo 360,
with NaN and infinities becoming zero. Preserved named forms round calculated hue after
normalization and map a rounded 360 back to zero; direct hue retains its exact
literal path. RGB conversion uses the
normalized unrounded hue.
When HSL/HWB convert to RGB, the final converted and clipped binary64 channels
use the same six-place positive-tie policy.
For example, `rgb(255 0 0 / none)` emits `color(srgb 1 0 0 / none)`.
An omitted alpha or a calculated alpha does not itself select this form.
These rules also apply to ordinary children of `color-mix()`. Byte limits count
the preserving form's actual UTF-8 length within the shared cumulative budget.
Origin colors nested in relative and `alpha()` forms retain unclamped authored
component domains with modern punctuation. Ordinary direct alpha is clamped
and rounded to six places before text emission. Ordinary RGB/HSL/HWB scalar
calculated alpha follows the selected per-slot clamping, unity omission and
[calculated number policy](#calculated-number-output). Contextual and other
retained calculated alpha keeps its wrapper; every path preserves the authored
calculation and provenance.

Direct Lab/LCH lightness clamps to 0..100 in standalone colors and ordinary
`color-mix()` arguments. Oklab/Oklch lightness clamps to 0..1 after exact
percentage conversion, following
[Color 4's Lab/LCH bounds](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#specifying-lab-lch)
and [Oklab/Oklch bounds](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#specifying-oklab-oklch).
Direct LCH/Oklch chroma has a zero minimum and no upper bound. The signed a/b
axes remain unbounded, and in-range literals retain exact decimal text.
For example, `lab(125% -20% 30%)` emits `lab(100 -25 37.5)`, and
`oklch(-.2 -20% 30)` emits `oklch(0 0 30)`. These direct bounds also apply when
properties compose standalone colors and to ordinary `color-mix()` arguments,
including nested mixes and a mix used as an origin color. This follows
[Color 5's individual argument serialization](https://www.w3.org/TR/2026/WD-css-color-5-20260908/#serial-color-mix).
For example, `color-mix(lab(125 calc(0) 0), blue)` emits
`color-mix(lab(100 calc(0) 0), blue)`. Calculations retain their wrappers and
required capture costs; genuine Origin and relative channels remain unclamped.

Specified Lab/LCH/Oklab/Oklch calculations retain out-of-range magnitudes:
`lab(calc(125) 0 0)` keeps `calc(125)`, and `lch(50 calc(-20) 30)` keeps
negative calculated chroma. Existing percentage scaling still applies, so
`oklab(calc(120%) 0 0)` emits `oklab(calc(1.2) 0 0)`. This follows the
[catalog's retained calculation phase](../specs/catalog.json):
[Color 4 §§15.2–15.3](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#resolving-lab-lch-values)
describe declared, computed and used values after L/C/H clamping, whereas
[Values 4 §10.12](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-range)
excludes specified calculations from range clamping. Frozen WebKit's non-eager
Lab/OK parsing preserves calculations while bounding direct siblings, and its
specified serializer keeps their wrappers. This selects the authored range
phase; it does not adopt WebKit's exact percentage text, float narrowing or
computed color conversion.

The serializer selects clipped direct endpoints from borrowed exact lexical
metadata before allocating a rational coefficient or expanding decimal text.
Thus `lab(1e400 0 0)` can emit `lab(100 0 0)` within a 12-byte output limit;
finite clipped exponents beyond i128 also work. Each selected direct slot adds
one logical projection visit to the cumulative budget. In-range and unbounded
values still return typed resource failures when their exact output or work
cannot fit. Discarded direct text is unnecessary; required calculation scratch
must still fit before composition. Authored token kinds, coefficients and
provenance remain unchanged.

`border-color` accepts one to four colors and an optional leading `logical`
marker. `CssBorderColorShorthand` retains authored arity and role mode;
`assigned_values()` applies shorthand repetition. Logical axis pairs retain
optional end values independently. Expansion follows the
[mode-selected membership policy](#four-side-shorthand-membership).

The selected Color 5 surface includes relative `rgb`/`rgba`, `hsl`/`hsla`,
`hwb`, `lab`, `lch`, `oklab`, `oklch`, and predefined RGB/XYZ `color()` spaces.
Relative origin channels are numeric in every environment. The checked
`CssRelativeColorExpression::try_from_components` takes an environment and
result slot and reports typed numeric errors with component path and origin.
`CssRelativeColor::try_new` derives its environment and three result slots from
the function; mismatched expressions are rejected. The source and expressions
remain symbolic, including optional alpha. `CssAlphaColor::try_new` similarly
distinguishes omitted alpha, `none`, and an `alpha` channel expression.
Finite coefficients in relative calculations use the shared six-place policy
after the existing mathematical projection. This includes custom-profile
calculations and calculated `alpha(from …)` overrides, even when nested inside
another origin or mix. References remain symbolic, percentage scaling stays
within the numeric projector, and authored expressions and origins remain
unchanged. Ordinary colors used as origins also use the shared coefficient text
policy, preserving their number and percentage dimensions, canonical angle units
and explicit unclamped alpha. Direct relative literals convert percentages to
the destination channel's scale before rounding; ordinary-origin literals retain
their authored categories. See [calculated number output](#calculated-number-output)
and [declared color literal output](#declared-color-literal-output).
Calculated relative alpha keeps its specified wrapper and magnitude, including
`alpha(from red / calc(2))` and explicit calculated unity. Direct alpha overrides
clamp but remain explicit, preserving the distinction from an omitted override.
The [catalog's relative calculated-alpha phase](../specs/catalog.json) selects
this retained form because
[Color 5 §11.3](https://www.w3.org/TR/2026/WD-css-color-5-20260908/#serial-relative-color)
calls declared alpha specified but clamped, while Values 4 reserves calculation
clamping and scalar-wrapper removal for computed or later serialization.
Frozen WebKit retains stored relative alpha and specified calculation trees;
its separate resolvers evaluate an origin before typed normalization. That
evidence supports the phase choice, without establishing identical percentage
text, custom-profile support or browser results. Profile binding, origin
evaluation and computed-value clamping belong downstream. Mathematical precision
and range remain unfinished.

The selected [Color 5 WD 2026-09-08 §6](https://www.w3.org/TR/2026/WD-css-color-5-20260908/#device-cmyk)
`device-cmyk()` grammar accepts exactly four comma-separated numbers or number
calculations in legacy form, with no alpha. Modern form accepts four space-separated
numbers, percentages or `none`, plus optional slash alpha-value or `none`.
Mixed separators, wrong cardinality and obsolete fallback-color arguments are invalid.
`CssDeviceCmykColor::try_new` checks these invariants over a fixed four-channel
array, plus complete numeric depth. Its `syntax()`, `channels()` and `alpha()`
accessors retain the supplied domains, scalar origins, calculations and omission.
`CssColor::from_device_cmyk` and `device_cmyk_value()` expose this single checked
payload. Eligibility reports contextual `DeviceCmyk`, including through relative,
alpha and Mix ancestors; outer Contrast and LightDark keep their contextual priority.
Absolute-only font palette values reject it.

Declared Standalone/Mix output uses lowercase modern `device-cmyk(`, exact direct
number channels and direct percentages divided by 100. Origin retains direct
number/percentage domains and explicit alpha. Ink channels stay unbounded in this
authored phase: `device-cmyk(-20% 140% none 2)` emits
`device-cmyk(-0.2 1.4 none 2)`. Calculations remain symbolic and missing channels
stay `none`. Ordinary direct alpha follows
[Color 4 §4.2 clamping](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#alpha-syntax),
then the selected shared retained serialization policy supplies six-place output
and unitary omission. Calculated alpha retains the established authored calculation policy.
All four channels, alpha, enclosing colors/images and siblings share complete
256-level depth and cumulative input/projection/UTF-8 serialization budgets.
The `interop.value.device-cmyk` record selects the complete authored production at
`#funcdef-device-cmyk` under `I-COLOR5-20260908`.

These are context-independent specified-value contracts. No RGB/Lab conversion,
profile binding, resource loading, computed ink clamping or computed CSSOM output
occurs. Color 5 §6 and §10.3 retain their unresolved computed-representation
conflict; §11.5 concerns computed serialization. The frozen WebKit source at
`73aa6c89e2cb77c46184a81aec944e4ab99d114d` supplies no authored device-CMYK
implementation resolving that conflict. Contextual execution and computed
serialization remain separate unfinished work.

### Pure HSL and HWB coordinates

`CssHslColorCoordinates::try_new(Option<H>, S, L)` and
`CssHwbColorCoordinates::try_new(Option<H>, W, B)` construct concrete coordinates
for the [Color 4 HSL](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#hsl-to-rgb)
and [HWB](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#hwb-to-rgb)
algorithms. Hue uses degrees; saturation, lightness, whiteness and blackness
use number units with a `0..100` reference range. Every present coordinate must
be finite. HSL saturation must be nonnegative, following the forward algorithm's
already-clamped input precondition. Extended lightness and saturation above 100,
and negative or extended HWB coordinates, remain mathematical inputs.

Both private-field carriers expose `hue_degrees()` and their named coordinates.
Constructors check H, then the second and third coordinate before rejecting
negative saturation. They normalize present hue to `[0, 360)` with positive
zero, canonicalize saturation zero, and preserve lightness, whiteness and
blackness bits. A manually supplied present hue stays present at a powerless
coordinate, and missing hue stays missing at any other coordinates.

| Operation | Concrete result |
| --- | --- |
| `convert_hsl_to_srgb(&coordinates)` | Fallible encoded `[r, g, b]`, with a `0..1` reference range |
| `convert_hwb_to_srgb(&coordinates)` | Infallible encoded `[r, g, b]` after checked construction |
| `convert_srgb_to_hsl([r, g, b])` | Fallible HSL carrier with optional hue |
| `convert_srgb_to_hwb([r, g, b])` | Fallible HWB carrier with optional hue |

Every finite extended RGB sample is admitted; the operations do not clip a
display gamut. Missing forward hue uses zero degrees. HWB's white-plus-black
normalization is intrinsic: W = 50 and B = 150 give gray `[0.25; 3]`. HSL
S = 200 and L = 50 at hue zero give `[1.5, -0.5, -0.5]`. Out-of-gamut inverse
HSL rotates a negative computed saturation's hue by 180 degrees and takes its
absolute saturation; HWB keeps the direct RGB hue.

Inverse HSL makes hue missing at normalized saturation at or below `1/100000`;
inverse HWB does so when normalized W + B is at least `1 - 1/100000`, or RGB is
gray. Saturation and W/B remain available unchanged. These are the specific
algorithms' inclusive thresholds and retained-coordinate interpretation selected
from frozen WebKit; the catalog records their conflicts with generic
[powerless-component cleanup](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#powerless).

`CssHslHwbConversionError` distinguishes the first `NonFiniteInput` index in
H/S/L, H/W/B or R/G/B order, `NegativeSaturation`, and `UnrepresentableResult`
when a required result exceeds finite `f64`. Percent scaling bounds every
checked HWB forward intermediate, making that direction infallible. The other
directions can overflow for extended inputs. Ordinary binary64 rounding,
underflow and loss of powerless hue apply; exact round trips are not promised.
These helpers are independent of authored CSS, alpha, profiles, contextual
evaluation, interpolation and computed CSS serialization.

### Pure rectangular color-space conversion

`CssRectangularColorCoordinates` couples three concrete numerical channels with
their `CssRectangularColorSpace`. The space is `Predefined` with an existing
`CssPredefinedColorSpace`, `Lab`, or `Oklab`: seven RGB spaces, XYZ D50/D65,
and the two rectangular Lab families. These coordinates are independent of the
authored `CssColor` graph and its specified serialization.

`try_new(space, [Option<f64>; 3])` accepts every finite extended channel and
explicit missing values. It preserves input bits, including negative zero,
and rejects the first present nonfinite channel in zero-based index order.
`space()` and `channels()` inspect the immutable value. RGB and XYZ channels
use number reference units where `100%` means `1`; Lab uses native `[L, a, b]`
units with ordinary lightness `0..100`, and Oklab uses native units with ordinary
lightness `0..1`. These usual ranges are not constructor bounds.

`convert_to(destination)` follows
[Color 4's intrinsic conversion algorithm](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#color-convert-algorithm).
Same-space conversion preserves every bit and missing value. Different-space
conversion substitutes zero for missing input channels locally, converts
through linear light and XYZ, adapts differing whitepoints with linear Bradford,
and returns three present channels. The source remains unchanged, including
when conversion fails. Results retain negative, out-of-gamut and extended
lightness values without range clipping.

ProPhoto RGB, Lab and XYZ D50 use D50; the other RGB spaces, Oklab and XYZ D65
use D65. RGB matrices derive from the normative primary chromaticities and
whitepoints: D50 XYZ is `[3457/3585, 1, 986/1195]`, and D65 XYZ is
`[3127/3290, 1, 3583/3290]`. Consistent coefficients for Bradford and Lab/Oklab
conversion are identified by the pinned implementation evidence in the
[standards catalog](../specs/README.md). The catalog distinguishes normative
definitions from the publication's nonnormative numerical samples.

Rec.2020 uses the pinned definition's reflected gamma `12/5`, with inverse
`5/12`, rather than frozen WebKit's older piecewise curve. ProPhoto uses reflected
signed-absolute powers `9/5` and `5/9`; its encoded toe is `|c| <= 1/32`, and
its linear toe is `|linear| <= 1/512`. This repairs the individually recorded
negative-domain expression defect to satisfy the publication's extended-range
contract. It does not claim agreement with WebKit's defective negative-input
branch or normative harmonization.

Private scratch arithmetic retains separate wider exponents for each channel
through transfer functions and matrices, with binary64 significand precision.
It prevents avoidable intermediate range loss when large encoded channels or
tiny linear-light channels produce representable destination channels. Ordinary
rounding and cancellation still apply; final underflow to zero is permitted.
This is neither exact-real arithmetic nor a correctly rounded conversion
guarantee. `CssRectangularColorConversionError::UnrepresentableResult` identifies
the first computed destination channel that cannot become finite `f64`.
`NonFiniteInput` identifies an invalid constructor channel. Both indices use
the selected space's three-channel order. Cross-space round trips are
approximate and cannot recover missingness replaced by zero.

Alpha, profiles, gamut mapping, physical output and interpolation remain outside
this helper. Callers compose the existing HSL/HWB and Lab-family polar helpers
through their corresponding encoded sRGB or Lab/Oklab coordinates; no authored
expression is evaluated or converted implicitly.

### Pure Lab-family rectangular and polar coordinates

`convert_lab_to_lch([L, a, b])` and `convert_oklab_to_oklch([L, a, b])`
implement [Color 4 CRD 2026-09-08 §9.5](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#lab-to-lch).
The concrete `f64` inputs use each space's number units: typical Lab lightness
is `0..100`, while typical Oklab lightness is `0..1`. Finite extended values are
accepted without clamping. Lightness is copied bit-for-bit, and stable hypot
calculates chroma without intermediate squaring overflow or underflow.

The resulting `CssPolarColorCoordinates` exposes `lightness()`, `chroma()` and
`hue_degrees()`. Hue is normalized atan2(b, a) in degrees when chroma is strictly
greater than `0.0015` for Lab or `0.000004` for Oklab; at equality and below,
it is `None` and chroma becomes positive zero, applying
[Color 4's conversion-generated neutral cleanup](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#powerless).
The inclusive thresholds come from Color 4 §§9.3–9.4 and its conversion
algorithm. Their discrepancy with the generic less-than wording is recorded
individually in the catalog using frozen WebKit's inclusive behavior.

`CssPolarColorCoordinates::try_new(L, C, Option<H>)` checks finite values in
L/C/H order before rejecting negative chroma. It preserves lightness bits,
canonicalizes chroma zero to positive zero, and normalizes present hue into
`[0, 360)`, including positive zero at the rounded 360° endpoint. The constructor
does not apply either space's threshold: missing hue stays missing at any
chroma, and present hue stays present below the thresholds. Its fields are
private. `CssPolarColorConversionError` distinguishes `NonFiniteInput` with the
first invalid component index, `NegativeChroma`, and `UnrepresentableResult`
when required forward chroma overflows finite `f64` arithmetic.

`convert_lch_to_lab(&coordinates)` and `convert_oklch_to_oklab(&coordinates)`
implement [Color 4 §9.6](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#lch-to-lab)
infallibly. Present hue gives C cos(H) and C sin(H); missing hue sets both axes
to positive zero regardless of chroma. Lightness retains its bits. For example,
Lab `[50, 3, 4]` gives C = 5 and H approximately 53.13010235415598°, whereas
`try_new(50.0, 5.0, None)` converts to `[50.0, 0.0, 0.0]`.

Ordinary `f64` rounding applies; neutral forward conversion intentionally loses
the opponent axes on inversion, and exact round trips are not promised. These
helpers are independent of authored CSS, alpha, profile binding, gamut handling,
and contextual evaluation.

### Naive numerical CMYK conversion

`naively_convert_cmyk_to_srgba([c, m, y, k, a])` and
`naively_convert_srgba_to_cmyk([r, g, b, a])` implement the two pure numerical
fallback formulas in
[Color 5 WD 2026-09-08 §6.1](https://www.w3.org/TR/2026/WD-css-color-5-20260908/#cmyk-rgb).
Their `f64` arrays contain concrete normalized coordinates; RGB channels are
sRGB-encoded, rather than 0–255 or linear-light channels. These APIs are
independent from the authored `CssColor` and `CssDeviceCmykColor` models.

Every input, including alpha, must be finite. Extended coordinates and alpha
outside `[0, 1]` are accepted without a range clamp. Alpha is copied bit-for-bit,
including negative zero. `CssNaiveColorConversionError::NonFiniteInput` identifies
the first invalid array index before arithmetic; `UnrepresentableResult` reports
a required result that overflows finite `f64` arithmetic. No source coordinates
are fabricated for numerical inputs.

The forward formula evaluates each channel as
`max(0, (1 - ink) * (1 - black))`. Opposite factor signs saturate to zero before
multiplication can overflow; a positive product overflow returns an error.
This zero saturation belongs to the specified formula. The inverse uses the
original maximum RGB channel `v`: black is `1 - v`, and inks are zero for
`v == 0`, otherwise `1 - channel / v`. There is no epsilon cutoff or shortcut
based on rounded black. Unequal subnormal RGB channels can retain different
ink ratios even when black rounds to one.

Ordinary `f64` rounding applies. The inverse chooses a canonical naive ink
decomposition, so original CMYK inks cannot generally be reconstructed. Tiny
RGB values can lose their magnitude when black rounds to one; exact near-black
round trips are not promised. These formulas provide uncalibrated numerical
fallbacks without promising calibrated equivalence. They do not resolve missing
or symbolic authored values, select a profile or fallback policy, load ICC
resources, clamp computed ink or opacity, perform gamut mapping, or provide
computed CSSOM output. Those contextual operations remain downstream.

### Authored color profiles

[`Color 5 §5.3`](https://www.w3.org/TR/2026/WD-css-color-5-20260908/#at-profile)
defines `@color-profile` with exactly one dashed identifier or the reserved
`device-cmyk` keyword. Custom names retain case, including the valid bare `--`;
the reserved keyword is case-insensitive and distinct from `--device-cmyk`.
`CssColorProfileRuleName` makes those identities explicit. Empty definitions and
missing descriptors remain valid authored syntax.

`src` accepts the shared Values 4 URL grammar, retaining `url()`/`src()` identity,
target and modifiers. No URL resolution or ICC validation occurs. The four
`CssColorProfileRenderingIntent` choices emit lowercase keywords; its default is
the real initial `RelativeColorimetric`, without inventing an authored occurrence.
`components` accepts a nonempty comma-separated list of ordinary identifiers.
Decoded ASCII-case-insensitive `none` is excluded; CSS-wide names, `default`,
duplicates, case-distinct names and escaped punctuation are accepted. Existing
`CssColorProfileName` and `CssColorProfileComponentName` own checked identities.

`CssColorProfileRule` retains every valid descriptor occurrence in source order;
`effective(kind)` returns the last valid occurrence, including a pending one.
It does not choose between duplicate profile definitions. Invalid descriptors
and child rules recover independently with diagnostics while valid siblings and
following rules survive. Clean validation rejects every diagnostic. Ordinary
group lists, including scopes, retain global profiles with their parent contexts;
a style-rule ancestor forbids them. Normalization exposes an intact
`CssRuleContextKindRef::ColorProfile` payload and produces no style declarations
from its descriptors.

`parse_color_profile_descriptor_value` and
`CssColorProfileDescriptorValue::try_new` share the whole-value grammar.
Valid `env()` qualifies for whole-grammar deferral under
[Env 1 §3](https://www.w3.org/TR/2025/WD-css-env-1-20250923/#env-function),
including other tokens such as `var()` and matched blocks. A standalone `var()`
does not qualify; quoted strings and URL data remain data. Pending values retain
their components and origins. `reparse_after_substitution` accepts caller-supplied
replacement components, rejects residual actual `var()`/`env()` functions,
and returns an ordinary checked value or a typed error. Construction and reentry
reject recovered implicit closures with their original closing origin; browser
parsing can retain them with recovery diagnostics. CSS performs no substitution.

Descriptor values, descriptor occurrences, and profile rules expose
`to_specified_css` and bounded variants. The generic `CssRule` and `CssSheet`
writers compose profile, palette and named-supports leaves under one cumulative
input/projection/UTF-8 budget. Rule punctuation and newline joining follow the
deterministic specified serialization policy. Style, ordinary groups and scopes
retain typed atomic `UnsupportedRule` failures; their general emission remains
unfinished. Four `interop.rule.color-profile` / `interop.descriptor.color-profile.*`
records cite `I-COLOR5-20260908`; these describe one authored rule and three
descriptor productions, without claiming complete Color 5 support.

The selected September 8 Color 5 `contrast-color(<color>)` production retains
exactly one symbolic input. `CssContrastColor::try_new` checks one wrapper plus
the complete input graph against the 256-level ceiling; `color()` borrows the
input. `CssColor::from_contrast_color` and `contrast_color_value()` expose the
checked payload. Absolute eligibility reports contextual `ContrastColor`,
including beneath relative, alpha and mix ancestors. Parsing requires complete
argument exhaustion and preserves typed resource failures and provenance.
Expansion, normalization and strict substitution reentry retain the symbolic
input, importance and occurrence identity. Style later evaluates the input and
chooses white or black using its contrast policy.

Specified output wraps the complete ordinary Standalone input in
`contrast-color(` and `)`, regardless of an enclosing Origin or Mix role.
Nested relative/alpha and mix values keep their own child roles. The iterative
serializer shares the enclosing input, projection and byte budgets without
mutating the authored graph. This operational interpretation follows frozen
WebKit's `CSSContrastColor.cpp:59–64` (blob
`73ebca675535a5cadc14ada0deb796a3394e3c12`), which independently serializes the
stored color. Color 5 §11 supplies no dedicated Contrast child-role algorithm;
this does not claim a normative conflict or browser execution equivalence.
`interop.value.contrast-color` records the complete authored production at
`#funcdef-contrast-color` under `I-COLOR5-20260908`.

Color 5's selected September 8 edition admits `light-dark()` with exactly two
colors or exactly two image/`none` branches. Mixed pairs are rejected.
`CssLightDarkColor::try_new` and `CssLightDarkImage::try_new` preserve ordered
branches through borrowed `light()`/`dark()` accessors. Their private payloads
cache checked depth; each constructor checks the complete child graph against
the 256-level ceiling, including gradient colors, numeric components and URL
modifier arguments. Existing infallible gradient constructors do not themselves
enforce this composed envelope; enclosing a supplied gradient in LightDark does.
Image construction errors distinguish `NestingLimit` and `CapacityOverflow`.
Speculative property alternatives preserve those typed resource failures and
the original component provenance. Color eligibility reports contextual
`LightDark`, including beneath relative, alpha and mix wrappers.

Specified text is `light-dark(`, the complete first branch, `, `, the complete
second branch, `)`. Iterative traversal shares input, projection and byte budgets
with enclosing values and keeps authored graphs unchanged. Image `none` stays
authored; even `light-dark(none, none)` is an image. Both branches remain symbolic
through expansion, normalization and strict substitution reentry. Style later
selects the used color scheme and resolves colors and resources.

Color children use ordinary Standalone specified serialization, regardless of
the enclosing pair's Mix or Origin role. Nested relative/alpha and mix values
still assign their own children's roles. This operational interpretation is
supported by frozen WebKit commit `73aa6c89e2cb77c46184a81aec944e4ab99d114d`,
`CSSLightDarkColor.cpp:66–72` (blob `44ff1a0de0c328ede9888c4be0279132cdb9dbf4`),
which serializes each stored color, and concrete-color dispatch in
`CSSColor.cpp:394–396` (blob `b3006b4ade8d2a363d4948732305aebab3a0299c`).
Color 5 §11 supplies no dedicated LightDark child-role algorithm; this does not
claim a normative conflict or browser execution evidence. The semantic features
`interop.value.light-dark-color` and `interop.value.light-dark-image` identify
complete authored productions at `#typedef-light-dark-color` and
`#typedef-light-dark-image`, using `I-COLOR5-20260908`. Earlier bounded features
keep their existing June source identity; this does not claim complete Color 5
or computed support. The selected September source body has SHA256
`f749cb75ec1c0faca7d2443546c88be4e22750526dd45f9fefcfcdd079ed3c09`.

Custom-profile `color(--Profile ...)` preserves a nonempty variable channel
list, exact numbers and percentages, missing components, and optional alpha.
`color(from <color> --Profile ...)` retains unbound profile-channel references;
profile existence and channel count are not grammar checks. Component names
preserve decoded spelling and exclude only case-insensitive `none`. `calc(pi)`
is a numeric constant, while a direct `pi` is a profile reference. Checked
custom and alpha constructors enforce nonempty lists, expression environments,
and combined structural depth. `absolute_eligibility()` reports authored
eligibility, profile dependence, or the first contextual exclusion in authored
order; it does not compute a color.

`color-mix()` accepts a nonempty ordered list with optional interpolation.
Omission means Oklab when resolved. Each color has an optional weight before or
after it. Literal `CssColorMixPercentage` uses one exact checked percentage
in the inclusive 0..100 range; a `CssColorMixWeight` calculation retains a
symbolic percentage math root. `CssColorMixComponent::new` takes a color and
optional weight; `CssColorMix::try_new` checks the list and composed depth.
`CssColorInterpolationMethod::try_new(space, hue)` checks predefined methods:
an explicit hue strategy requires HSL, HWB, LCH or Oklch. Rectangular spaces
accept an omitted hue; rejected combinations return
`CssColorInterpolationMethodConstructionError::HueRequiresPolarSpace`.
`CssColorInterpolation::from_predefined(method)` wraps a checked method.
The method's `space()` and `hue()` preserve authored identity, including omitted
hue versus explicit `Shorter`. `effective_hue()` supplies the
[Color 4 baseline](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#hue-interpolation):
omitted polar hue means `Shorter`, explicit polar hue keeps its strategy, and
rectangular spaces have no hue method. Hosts own any override of that baseline;
this accessor does not execute interpolation.

Custom interpolation profile names keep their case-sensitive decoded identity.
Parsing does not distribute
weights, resolve profiles, or evaluate colors. Specified serialization fills
known omitted weights exactly before selected six-place rounding, keeps unknown
calculation omissions, and omits equal effective shares with default Oklab.
It omits explicit `shorter hue` from canonical Mix text while preserving the
authored method; other hue interpolation methods remain explicit.

The [intrinsic numerical APIs](#pure-rectangular-color-space-conversion) cover
predefined rectangular spaces and Lab/Oklab, alongside the HSL/HWB and
Lab-family conversions described above. Relative-channel evaluation, gamut
mapping, contrast selection and rendering remain downstream.

## Authored opacity and specified serialization

`CssOpacityValue::Scalar(CssOpacityScalar)` retains every ordinary number or
percentage, including signed and out-of-range specified values. The checked
component preserves exact decimal spelling, number/percentage kind, and original
parsed or programmatic provenance even when a floating-point cache underflows
or overflows. Scalar construction accepts only number and percentage components.
`NumberCalculation` and `PercentageCalculation` retain domain-checked symbolic
math roots. Authored transport does not clamp opacity.

`CssOpacityValue::serialize_specified()` produces canonical specified text;
`serialize_specified_with_limits()` supplies independent input-node, cumulative
projection-node, and output-byte limits. Defaults are 65,536, 262,144, and
1,048,576 respectively; zero limits are valid. Failures return a typed
`CssSpecifiedValueSerializationError` without partial text or input mutation.
These logical bounds do not guarantee allocator availability.

Ordinary scalars use their exact retained magnitude. Percentages divide by 100
symbolically and serialize as numbers: `.1` becomes `0.1`, `25%` becomes `0.25`,
and `150%` becomes `1.5`. Programmatic scalars retain the supplied checked
decimal token just as parsed scalars do.
The retained percentage magnitude stays exact even when output rounds to zero.
Exponents that would exceed the output cap return a byte-limit error before
expanding zeros.

Math projection uses binary64 arithmetic for context-independent operations,
including CSS exceptional-value and signed-zero rules. Absolute compatible
units normalize; font, viewport, container, and other contextual units remain
symbolic. `calc(1 / 2)` becomes `calc(0.5)`, while `sign(1em - 1px)` retains its
contextual expression. This specified stage does not clamp to the opacity
range. Finite math text uses the [calculated number output](#calculated-number-output)
policy: at most six fractional places in fixed notation, rounding the actual
binary value to nearest with decimal ties away from zero. Transcendental last
bits may differ across platforms; bit-identical transcendental output is not
promised. Authored component accessors and structural serialization remain
separate and preserve their contracts.

The selected Values 4 section 10.13 has two localized serialization defects:
an unclosed nonfinite result and a child loop that misserializes scalar/operator
roots. The standards catalog records the repairs: close the nonfinite `calc`,
and serialize scalar/operator roots as one calculation argument. These are
explicit reconciliations with the same edition's grammar, not claims that the
unmodified algorithm emits those outputs. Computed opacity clamping belongs
to a later phase.

## Authored Grid repetition and keyframe structure

Grid property wrappers expose authored values through `value()`. The parallel
Grid graphs and `i01_subset()` methods are removed. Checked `CssGrid*` models
are the sole Grid API. Track breadth and size accessors expose exact scalars
through `length_percentage()`, `flex()`, and
`fit_content()`.

`grid-auto-flow` accepts six authored meanings: `normal`, `dense`, `row`,
`row dense`, `column`, and `column dense`. The axis and `dense` keywords may be
authored in either order; `CssGridAutoFlow::serialize_specified()` emits
the canonical order. `normal` and bare `dense` leave direction unspecified, and
the `CssGridAutoFlowPropertyValue::value()` accessor preserves that difference.
The `ExplicitAxis` branch contains a `CssGridAutoFlowMode`, the explicit axis
and dense selection. The intrinsic
initial is `normal`, and the longhand is not inherited. CSS-wide values,
`all`, and pending substitutions remain
symbolic through intrinsic expansion.

This initial differs from the `row` initial in the selected
[Grid 2 §7.7](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#grid-auto-flow-property).
The choice follows the
[pinned WebKit grammar and initial](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSProperties.json#L11479)
while the selected
[Grid 3 §2.3](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#grid-lanes-orientation)
leaves open whether its orientation belongs on `grid-auto-flow` or a separate
property. This CSS layer does not choose the resulting layout direction or
change the existing `grid` shorthand's explicit row/dense branch.

`grid-template-areas` exposes `CssGridTemplateAreas` through its
property wrapper's `value()` accessor. Its `None` initial is not inherited.
Each decoded string row follows [Grid 2 §7.3](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#grid-template-areas-property):
the longest run of ident code points makes one named cell, the longest run of
periods makes one empty cell, and decoded space, tab, and line feed separate
runs. Other decoded ASCII punctuation or controls, including carriage return
and form feed produced by an escape, invalidate the declaration. Non-ASCII
code points such as NBSP remain part of an area name. The checked row and
matrix constructors require nonempty rows, equal widths, and a filled rectangle
for each case-sensitive name. Area names have their own checked type so leading
digits and CSS-wide keyword spellings remain valid within a string. The
decoded area name is the sole typed representation, including names that
would be reserved in ordinary custom identifiers.

`serialize_specified_with_limits()` emits canonical quoted rows with one space
between cells and one period for each empty cell under cumulative input-node,
projection-node, and CSS-byte budgets. The property wrapper's `as_css()` retains
the authored source. CSS-wide values, `all`, and pending substitutions use the
same intrinsic expansion and strict grammar reentry as other completed
longhands. This does not complete the `grid-template` or `grid` shorthand's
ASCII-art grammar or perform layout.

The four Grid placement longhands (`grid-row-start`, `grid-row-end`,
`grid-column-start`, and `grid-column-end`) and the `grid-row`, `grid-column`,
and `grid-area` shorthands expose `CssGridLine`,
`CssGridLineRange`, and `CssGridArea` through their property
wrappers' `value()` accessors. These are the authored [Grid 2 line placement
values](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#line-placement),
which [Grid 3 §4.1](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#grid-lanes-placement)
also uses. A line may be `auto`, a bare name, a nonzero signed integer with an
optional name, or `span` with a positive integer and/or name. Component order is
flexible on input; specified serialization writes the integer before the name,
and writes `span` first. Ordinary integer tokens retain exact magnitude beyond
`i32`. Integer-root math remains symbolic, including expressions whose result
could fall outside a range; [Values 4 range checking](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-range)
belongs to computed or used value handling.

The shorthand models retain which slash-separated members were authored.
`effective_end()` and the area `effective_*()` accessors provide intrinsic
longhand contributions: an omitted partner copies only a bare line name and is
otherwise `auto`. In `grid-area`, the member order is row start, column start,
row end, column end, and an omitted column end follows the effective column
start. These accessors do not resolve conflicting line placements; that needs
layout context. Canonical
`serialize_specified_with_limits()` applies one cumulative resource budget to
the complete line or shorthand.

The six Grid repetition consumers expose their authored value through `value()`.
Grid track lists distinguish general lists from lists containing exactly
one automatic repetition. Integer and automatic repetitions are non-recursive.
Integer repeat counts are `CssPositiveIntegerLiteral` values in both
`CssGridIntegerTrackRepeat` and `CssGridIntegerFixedRepeat`. Constructors store the
checked exact token, including its sign, leading zeros and origin; `count()`
borrows that value. Counts remain literal-only and strictly positive. Canonical
serialization removes redundant plus signs and leading zeros without narrowing
the magnitude, and charges the count within the enclosing cumulative input,
projection and output budgets.

The [selected Grid 3 publication](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#intrinsic-auto-repeat)
admits general track sizes inside automatic repetition, including intrinsic
keywords, flexible tracks, `minmax()`, and `fit-content()`. Surrounding tracks
and integer repetitions retain the fixed-size restrictions of Grid 2's
[`<auto-track-list>`](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#typedef-auto-track-list).
`grid-auto-rows` and `grid-auto-columns` still accept track sizes without
`repeat()`. Each explicit axis has its own limit of one automatic repetition.

`CssGridAutoRepeat::content()` now returns
`&CssGridTrackRepeatContent`. Consumers inspect its ordered `LineNames`
and `TrackSize` members; the former fixed-body return type no longer describes
the selected grammar. Surrounding integer fixed repeats continue to expose
`CssGridFixedRepeatContent`. Typed calculations retain their symbolic
structure. Historical captured inputs
and observations remain unchanged; semantic witnesses apply the selected grammar.

The shared repeat feature cites the Grid 3 extension. The containing property
records retain their Grid 2 property grammar sources and document the extension
in their supported subset. Source identities are immutable.

Keyframe rules preserve authored structure rather than a merged animation
timeline. Empty rules and blocks remain present. Repeated selector blocks,
equivalent offsets in different blocks, and repeated equivalent selectors within
one list remain in source order without sorting, merging, or deduplication. When
an invalid declaration is dropped, its now-empty block and rule remain; an
invalid selector still drops the smallest invalid keyframe block. These recovery
observables replace older expectations that accepted structurally invalid Grid
cross-products or discarded valid empty keyframe parents.

Empty `[]` line-name groups are retained as ordered authored components in
explicit tracks and repetitions.
`CssGridLineNames::new(Vec::new())` accepts the same empty group. A group
does not supply a required track size; reserved line names remain rejected.
Two adjacent `[...]` groups cannot occupy one track boundary, including inside
integer and automatic repetitions.

The shared Grid track-size model keeps exact ordinary nonnegative `fr`, length,
and percentage quantities until a consumer has a finite conversion policy.
For example, `1e50fr` and `1e-50fr` remain distinct, while `-1e-50fr` fails
ordinary range admission before floating-point rounding. Its checked Flex and
length-percentage calculations remain symbolic. The selected [Grid 2 `fr`
wording](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#fr-unit) conflicts
with [Values 4 calculation typing](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-type-checking)
for flex-result math. Following the product's browser tiebreaker, pinned
[WebKit Grid consumption](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSPropertyParserConsumer%2BGrid.cpp)
selects checked Flex first, then length-percentage for contextual type failures.
This is an operational parser choice, not a claim that the two specifications
agree. Flex results are excluded from inflexible `minmax()` minima and
`fit-content()` arguments. `length_percentage()`,
`flex()`, and `fit_content()` expose the retained checked
scalar without floating-point narrowing.
`serialize_specified()` on typed track sizes and lists emits bounded canonical
CSS from retained values, whereas declaration `as_css()` preserves authored text.

The Grid repetition value, the six Grid property records, and the keyframe rule
record remain `Partial`. Subgrid name-repeat, remaining `grid`/`grid-template`
shorthand alternatives remain unfinished.
`grid-auto-rows` and `grid-auto-columns` have noninherited `auto` initial
values and expand to one intrinsic longhand contribution. CSS-wide keywords
stay symbolic, and substituted values reenter the same repeat-free grammar
strictly before producing contributions.
Calculation keyframe selectors, string names, and unselected declaration-processing
grammar remain outside the keyframe boundary. Repetition counts and used track
sizes remain unresolved. This crate does not perform Grid layout, cascade
declarations, evaluate or interpolate keyframes, run timelines, or lower either
syntax family into sibling Surgeist crates.

## Typography, font families, and font-face

### Authored text alignment

`text-align`, `text-align-all`, and `text-align-last` follow the selected
[Text 4 alignment definitions](https://www.w3.org/TR/2026/WD-css-text-4-20260814/#text-align-property).
The first is a shorthand for the latter two. An ordinary positional keyword
sets `text-align-all` and leaves `text-align-last` at its intrinsic `auto`;
`match-parent` sets both to `match-parent`, while `justify-all` sets both to
`justify`. Both longhands inherit; their intrinsic initials are `start` and
`auto` respectively. Expansion retains the original declaration, importance,
and authored order without resolving parent direction.

The shorthand and `text-align-all` also accept a string containing exactly one
decoded default extended grapheme cluster, optionally paired with one of
`start`, `end`, `left`, `right`, or `center` in either order. The pair grammar
and one-cluster rule follow the
[catalog's localized reconciliation](../specs/README.md), which distinguishes
the selected Text 4 prose from the historical grammar and UAX29 boundary.
An omitted positional fallback remains omitted in the authored model; its intrinsic
effective fallback is `right`. The checked `CssCharacterAlignment` retains the
string component's origin and exposes both decoded text and the authored
fallback. Its constructors reject empty and multiple-cluster strings.
`CssTextAlignValue`, `CssTextAlignAllValue`, and `CssTextAlignLastValue` expose
the three distinct grammars and bounded canonical specified-value
serialization. `CssTextAlignPropertyValue::value()` and
`CssTextAlignLastPropertyValue::value()` provide these checked semantic values.
`text-align-all` likewise exposes its checked value through `value()`.
Character search, font selection,
directional resolution, and line layout remain downstream concerns.

The authored font surface includes checked four-ASCII-character OpenType tags,
non-negative feature indices, explicit and system `font` branches, synthesis,
and the seven variant longhands. `font-family`, explicit-font family lists, the
`@font-face` family descriptor, and `local()` names follow the selected
September 7, 2026 Fonts 4 grammar. Other typography records retain their
individual dated sources.

`CssFontFamilyPropertyValue::families()` and `CssFontPropertyValue::font()` expose
`CssFontFamilyList` and `CssFontValue`, with exact authored text available through
`as_css()`. Each font wrapper retains one checked semantic value; explicit
shorthand components belong to `CssExplicitFont`.

`CssFontFamilyName` and `CssFontFamilyList` expose `serialize_specified()` and
`serialize_specified_with_limits()`. These retain the authored quoted,
identifier-sequence, or generic branch, including identifier token boundaries,
case, and list order. For example, one identifier containing a space emits
`A\ B`, while two identifiers emit `A B`; a quoted `"serif"` remains distinct
from the generic `serif`. This is specified-value serialization rather than the
computed family-name joining algorithm or font selection.

`CssFontFaceFamily` exposes the same bounded methods for its decoded literal
name. This model is shared by font-face descriptors, font-feature-values
preludes, and font-palette-values families; it does not retain the original
quoting form. Its canonical output uses a bare identifier sequence when that
preserves the literal name, and quotes reserved, empty, or otherwise unsuitable
names. Emitting a family does not provide a complete font-face or feature-rule
writer. The existing font shorthand and palette descriptor/rule writers share
these family helpers and the common CSSOM escaping writer.

One cumulative serialization budget covers all emitted bytes and visited name
nodes. Property names charge one node plus each retained identifier token;
generic and quoted property names charge one node. A decoded descriptor family
charges one node, including multiword names. Family-list containers add no node
charge, preserving the embedded shorthand and palette budget contracts. A
failed emission returns a typed error without partial CSS or changes to the
checked model, declaration occurrence, or source components.

The selected Fonts 4 `font-variant` family accepts ligatures, caps, alternates,
numeric, East Asian, position, and emoji groups in either longhand declarations
or the full shorthand. `CssFontVariantValue` retains the authored groups;
intrinsic expansion contributes all seven inherited longhands, using `normal`
for omitted groups. `none` contributes ligatures `none` and six `normal` values.
The shorthand and its longhand wrappers expose borrowed current values and no
longer offer an `i01_subset()` projection. The separate CSS2 `font` prefix
continues to accept only `normal` or `small-caps` as its variant component.

`CssFontVariantAlternateValues` checks at most one of each alternate function
and a nonempty combination. `CssFontVariantAlternateNames` checks nonempty name
lists while retaining case, order, and repeated names. Names use the decoded
`CssFontFeatureValueName` identifier model and remain unresolved until a font is
selected. All seven longhand value types and `CssFontVariantValue` provide bounded
`serialize_specified()` and `serialize_specified_with_limits()` methods. These
emit grammar order and escaped names without resolving glyph selection.

Code constructing `CssFontVariantValues` must append `alternates` and `emoji`
arguments to `try_new`, passing `None` for either omitted group. The aggregate
and `CssFontVariantValue` now own optional name collections, so they implement
`Clone` rather than `Copy`; callers needing another owned value must clone it.
`CssFontVariantValues::try_new` is no longer `const`, and its `caps()` and
`position()` getters now borrow `&self` while still returning the same copied
enum choices. Migrate `CssFontVariantPropertyValue::i01_subset()` consumers to
`variant()` and inspect the `Normal`, `None`, or checked `Values` branch directly.

```rust
use surgeist_css::{
    CssFontValue, CssKnownPropertyValueRef, CssSystemFont,
    parse_style_attribute,
};

let report = parse_style_attribute("font: menu; font-weight: 725");
assert!(report.is_clean());
let CssKnownPropertyValueRef::Font(font) = report.syntax()[0]
    .known().expect("known font")
    .property_value().expect("ordinary font")
else { panic!("expected font") };
assert!(matches!(font.font(), CssFontValue::System(CssSystemFont::Menu)));
assert_eq!(font.as_css(), "menu");
```

The selected Fonts 4 `font-weight` property accepts absolute `normal`, `bold`
and exact numeric values from 1 through 1000, plus relative `bolder` and
`lighter`. `CssFontWeight::Absolute(CssAbsoluteFontWeight)` now distinguishes
these branches. `CssFontWeightNumber` retains an exact number token or symbolic
number calculation, with its original parsed or programmatic origin;
`try_from_component` and `try_from_calculation` check the literal bounds and
numeric domain. Symbolic math remains unresolved. Its `literal_component()`,
`calculation()`, `origin()` and bounded `serialize_specified()` methods replace
the old integer `value()` projection. `CssFontWeightPropertyValue::value()`
returns a borrowed `&CssFontWeight`. Explicit `font` shorthand values also
borrow their optional weight through `CssExplicitFont::weight()` and accept the
same absolute and relative weight components, including fractions and number
math. Intrinsic shorthand expansion now contributes seven settable terminals and
resets twelve more to their initials.

The selected Fonts 4 `font-style` property accepts `normal`, `italic`, `left`,
`right`, and `oblique` with an optional angle. `CssFontStyleKeyword` shares the
four keywords with the descriptor, while `CssFontStyle::Oblique { angle: None }`
retains omission rather than inserting the matching default `14deg`.
`CssFontObliqueAngle` accepts literal `deg`, `grad`, and `turn` angles at exact
lexical bounds of ±90deg, ±100grad, and ±0.25turn. Radian admission has an
explicit binary64 precision: the original decimal coefficient is parsed as
`f64` and compared directly with `FRAC_PI_2`. Thus
`1.5707963267948966rad` is admitted and `1.5707963267948968rad` is rejected;
decimal spellings that round to the same binary64 value share an admission
result. The original component, unit, token spelling, and parsed or programmatic
origin remain available after admission. A bare unitless zero is not an angle.
Angle-valued calculations remain symbolic and use the existing numeric type
rules; their apparent computed result is not range-checked here.

`CssFontFaceStyle` admits descriptor-only `auto`, one of the shared keywords,
or oblique with zero, one, or two checked angle endpoints. The optional
`CssFontFaceObliqueRange` end preserves authored arity and order, including
descending and mixed-unit pairs. Its `new(start, end)`, `start()`, and `end()`
methods operate on checked angles without a floating-point degree projection.
The old property variants such as `CssFontStyle::Italic` become
`CssFontStyle::Keyword(CssFontStyleKeyword::Italic)`; the old descriptor variants
become `CssFontFaceStyle::Keyword(...)` or `Oblique { range }`. The property
wrapper's `value()` returns a borrowed `&CssFontStyle`, and
`CssExplicitFont::style()` returns `Option<&CssFontStyle>`; the old property
`i01_subset()` projection is removed. The inherited property initial is
`normal`, and expansion preserves importance and source provenance. The
explicit `font` shorthand accepts this selected style component, including
optional oblique angle math, while retaining its other documented component
limits. Property, descriptor, scalar, and range values have bounded canonical
specified serializers; Fonts rule composition uses the shared owner described
[below](#checked-fonts-rule-composition), without matching or computed resolution.

The selected Fonts 4 `font-size` property accepts the eight absolute keywords
from `xx-small` through `xxx-large`, the relative `larger` and `smaller`
keywords, dedicated `math`, and nonnegative lengths or percentages. `math` is
a keyword with MathML-related downstream behavior, distinct from a numeric
calculation. `CssFontSize::LengthPercentage` now contains
`CssSpecifiedNonNegativeLengthPercentage` directly. Use the shared checked value's
`try_from_component`, `try_from_calculation`, `literal_component`,
`calculation`, and `origin` methods to preserve exact authored tokens and
symbolic math. Exact negative nonzero literals fail even when their magnitude
underflows a floating-point field; unitless zero and signed zero remain valid.
`CssFontSize` equality compares authored structure without comparing origins;
the shared scalar's own equality continues to include provenance.
Huge positive exponents remain authored without expansion during admission,
while bounded serialization can reject output that exceeds its byte budget.

`CssFontSizePropertyValue::size()` borrows `&CssFontSize`.
`CssExplicitFont::size()` also continues to borrow. The shorthand accepts the
same size grammar, including `xxx-large`, `math`, and typed size math, while
preserving the following font-family boundary and optional slash line height.
The inherited size initial is `medium`, and intrinsic longhand contributions
preserve importance and declaration provenance. Explicit `font` values contribute
present components and initial values for omissions and reset-only members.
System fonts retain symbolic values for the seven settable terminals; the twelve
reset-only terminals receive intrinsic initials. System font selection and
missing-preference fallback belong to downstream resolution.

`line-height` and the optional `font` slash component share one exact authored
grammar: `normal`, a nonnegative number, or a nonnegative length-percentage.
`CssLineHeight::Number` retains a checked `CssSpecifiedNonNegativeNumber`;
`CssLineHeight::LengthPercentage` retains the corresponding checked scalar.
`CssLineHeight::HintedNumberCalculation` retains a Number result with an unresolved
percentage basis. It uses the same grammar in the `font` slash component,
compares structure while ignoring origins, and reuses the shared numeric
serializer without a font basis or early range clamp.
Unitless zero stays a number, while `0px` and `0%` retain their own domain.
Ordinary negative nonzero values are rejected before floating-point narrowing;
bare checked calculation roots reenter literal admission, while actual math
functions remain symbolic. Line-height equality ignores source origin
while preserving authored structure, whereas the shared number scalar's own
equality retains provenance. Both support bounded specified serialization.
`CssLineHeightPropertyValue::line_height()` borrows the checked line height;
numeric payloads retain exact ordinary tokens or symbolic calculations. Construct
the number payloads with `CssSpecifiedNonNegativeNumber::try_from_component`
or `try_from_calculation`, then inspect them with `literal_component()`,
`calculation()`, and `origin()`.
Its inherited initial is `normal`, and intrinsic longhand expansion produces one
contribution with the declaration's importance and provenance. Font metrics,
percentage resolution, and actual line box computation remain downstream.

The six selected font reset longhands now have current typed values and intrinsic
longhand contributions: `font-feature-settings`, `font-kerning`,
`font-size-adjust`, `font-language-override`, `font-optical-sizing`, and
`font-variation-settings`. The two settings lists retain ordered and repeated
four-character printable ASCII OpenType tags. Feature indices use exact
nonnegative integer tokens or symbolic integer math; variation axis values use
exact signed numbers or symbolic number math. `font-size-adjust` keeps an exact
nonnegative number. `font-language-override` accepts any decoded CSS string,
including the empty string, while `normal` remains distinct. The selected
`font-feature-settings` and `font-variation-settings` `@font-face` descriptors
share these settings grammars and retain ordered occurrences.

Migration from the former feature API: `CssFontFeatureSettings`,
`CssFontFeatureList`, `CssFontFeature`, and `CssFontFeatureValue` are removed.
Use `CssAuthoredFontFeatureSettings`, `CssAuthoredFontFeatureList`,
`CssAuthoredFontFeature`, and `CssAuthoredFontFeatureValue`; the property wrapper
exposes `settings()` instead of `i01_subset()`. Feature values and indices are
no longer `Copy`; borrow through `value()` and inspect exact indices with
`literal_component()`, `calculation()`, or `i32_value()` when representable.
`CssFontSizeAdjust::Number` now holds `CssSpecifiedNonNegativeNumber`; inspect
its exact component or calculation instead of a float. The new signed
`CssSpecifiedNumber` serves variation values. All these scalars and settings
offer bounded specified serialization. `CssFontValue` and `CssExplicitFont` also
serialize canonically under one cumulative input, projection, and byte budget.
Their family serializer preserves quoted versus identifier representation and
identifier token boundaries. Optional `normal` components are omitted when they
do not alter shorthand expansion; authored optional nodes still count against the
input budget even when omitted from the output. Font matching and computed value
resolution remain downstream.

The `@font-face` weight descriptor now uses
`CssFontFaceWeight::Auto` or `Range { start, end }`, where each endpoint is a
checked `CssAbsoluteFontWeight` and `end: None` preserves a single authored
value. This replaces the old float-backed `CssFontFaceWeightValue` and
`CssFontFaceWeightKeyword` types and
start/end/keyword projection methods. Descending and mixed keyword/numeric
ranges retain their authored order; computed endpoint ordering and font
matching belong downstream. The property, descriptor and numeric/range
historical conformance IDs now cite the September 7, 2026 Fonts 4 source and
are `Complete` for their authored grammars. This does not complete `font`,
`@font-face` as a whole, or environment execution.

`CssFontFamilyName` distinguishes quoted literal names, identifier sequences,
and typed generics. Its fifteen `CssGenericFontFamily` values comprise eleven
simple keywords (`serif`, `sans-serif`, `cursive`, `fantasy`, `monospace`,
`system-ui`, `math`, `ui-serif`, `ui-sans-serif`, `ui-monospace`, `ui-rounded`)
and four functional forms (`generic(fangsong)`, `generic(kai)`,
`generic(khmer-mul)`, `generic(nastaliq)`). Bare `generic`, `fangsong`, `kai`,
`khmer-mul`, `nastaliq`, and `emoji` remain ordinary literal names.

`try_ident_sequence(Vec<String>)` accepts decoded identifier tokens without
splitting or trimming them. It requires a nonempty list of nonempty tokens,
each excluding U+0000, the eleven simple generics, the five selected CSS-wide
keywords, and `default`. Reserved-token comparisons are ASCII-insensitive.
`try_ident` checks one decoded token, and `generic` accepts a typed generic
infallibly. `identifier_tokens()` exposes the preserved token boundaries;
`as_str()` joins identifier tokens with one U+0020 space. Thus tokens
`["A", "default"]` are invalid, while `["A default"]` is one valid escaped
identifier. An identifier containing only escaped whitespace is also valid.

`try_quoted` accepts decoded literal strings, including empty and whitespace-only
names and reserved spellings, while rejecting U+0000. A quoted `"serif"` remains
distinct from generic `serif`. `CssFontFamilyList::try_new` requires at least one
item, so a quoted empty name is a valid list item. `CssFontFaceFamily::try_new`
and `CssFontLocalName::try_new` also accept decoded literal strings with this NUL
restriction; these wrappers do not assert that the input was an identifier
sequence. Their parser paths validate unquoted tokens before joining them.
Generic branches are valid in property lists and explicit-font family tails,
but invalid in the face-family descriptor and `local()`.

`CssFontLocalName`, `CssFontFaceUrlSource`, `CssFontFaceSource`, and
`CssFontFaceSourceList` expose `serialize_specified()` and
`serialize_specified_with_limits()`. Following the
[CSSOM LOCAL idiom](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-local),
a local source always contains a serialized string: `local(Gentium Bold)` emits
`local("Gentium Bold")`. The checked nonempty source list emits members in
priority order with comma-space separators. The URL branch reuses the shared
authored URL provider, preserving `url()` versus `src()`, decoded targets,
ordered modifiers and their retained argument components, without resolving
relative targets or accessing resources.

The selected
[Fonts 4 source grammar](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing)
supplies the optional `format()` followed by `tech()`. Keyword formats emit
lowercase; string formats retain their distinct production and decoded case,
including empty, unknown and legacy strings. Technology hints emit lowercase in
authored order, including repetitions. Serialization does not use recognized
format or required-technology queries to rewrite these authored values.

One cumulative input-node, projection-node and generated UTF-8 byte budget
covers the complete source list. A list charges one node; enum dispatch adds
none. A local source charges its wrapper and string target. A URL source adds
one source node to the shared URL provider's visits, one node for a present
format argument, and one technology-list node plus each hint when technologies
are present. Failure returns a typed error with no partial text or changes to
stored names, hints, source order or enclosing recovery diagnostics.

The six system spellings (`caption`, `icon`, `menu`, `message-box`,
`small-caption`, `status-bar`) are literal names in family contexts. A complete
`font: menu` selects the system-font branch, while `font: large menu` selects a
literal family named `menu`, as specified in
[Fonts 4 §2.7](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-prop).
Whole-value CSS-wide keywords use the declaration's global-value branch.
These distinctions preserve decoded meaning and identifier boundaries.
`CssFontFaceFamily::serialize_specified()` and its limits variant emit a decoded
literal name as a bare identifier sequence when that preserves its meaning;
empty, reserved or otherwise unsuitable names use string serialization.
Property-family serializers retain the quoted, identifier-sequence or generic
branch and escape each identifier token independently, keeping quoted reserved
names distinct from generic families. Rejecting decoded U+0000 avoids claiming
preservation of a character that CSS replaces with U+FFFD.

`parse_font_face_descriptor_value(source, kind)` parses a complete raw value
using the selected `CssFontFaceDescriptorKind` grammar. It returns an owned
`CssAuthoredFontFaceDescriptorValue::Ordinary` or `Pending` without an invented
descriptor-name position.
Diagnostics refer directly to the supplied source, including actual EOF; keep
the source when interpreting those positions. Descriptor names, semicolons and
`!important` are outside this value input. Grammar rejection returns `None` with
`RejectInput`; resource limits retain `StopAtNestingLimit`.

A partially valid `src` list retains valid sources and reports
`DropFontSourceListItem`. Member recovery and implicit-closure diagnostics are
published only when the enclosing value is retained. These typed values require
no surrounding family/source descriptors and do not perform font matching or
loading. This raw-source parser is distinct from component-based
`parse_property_value`, whose errors use component origins; descriptor values
carry no parsed occurrence provenance.

`@font-feature-values` retains a nonempty ordered family list and an ordered
mixed body of `font-display` occurrences and all seven subsidiary block kinds:
`stylistic`, `historical-forms`, `styleset`, `character-variant`, `swash`,
`ornaments`, and `annotation`. Empty rules and blocks, repeated names, duplicate
indexes, and interleaved descriptors remain intact. Friendly names are decoded,
case-sensitive identifiers; reserved-looking names and escaped punctuation are
valid. Invalid descriptors/definitions and unknown or malformed subsidiary
blocks recover locally, while an invalid family prelude drops the outer rule.

`CssFontFeatureValuesRule::try_new`, `CssFontFeatureValueBlock::try_new`, and
`CssFontFeatureValueDefinition::try_new` enforce the same grammar as parsing.
A standalone definition validates its nonempty list; the block constructor also
validates kind-specific cardinality constraints. `CssFontFeatureValueIndex`
stores exact normalized decimal digits without a machine-integer maximum.
`try_from_decimal` accepts only an optional ASCII sign and digits, rejects
negative nonzero values, and normalizes leading zeros and negative zero.
`to_u32` returns `None` on overflow. Parsed index origins identify the whole
original token; constructors produce no source coordinates. This type is
separate from the older `CssFontFeatureIndex` used by `font-feature-settings`.

The [pinned Fonts 4 section 6.9.1](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-feature-values-syntax)
conflicts with [section 6.9.2](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/).
The selected [section 6.9.2](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#multi-value-features)
and [frozen WebKit parser](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSParser.cpp#L800)
policy admits one or two character-variant indexes and a nonempty styleset list.
Neither the first character-variant index nor any styleset index has a
feature-specific authored upper bound. All positions preserve exact nonnegative
integers without a machine-integer maximum. The section 6.9.1/6.9.2 normative
contradiction remains unresolved; downstream font activation may ignore indexes
that select no supported font feature. Historical-forms accepts a nonempty list;
stylistic, swash, ornaments and annotation each require exactly one index.

Ordinary media/supports/container/layer/scope rule lists retain this global
named rule; any style-rule ancestor forbids it, including through intervening
groups. Normalization emits one opaque rule payload with its parent contexts;
it preserves complete body order and emits no property contributions. Payload
members do not individually consume normalization declaration/rule budgets, and
those budgets do not cap allocation. Font matching, mapping winners across rules and cascade remain downstream; canonical specified
serialization projects effective tuples within this single rule.

`@font-palette-values <dashed-ident>` retains a decoded, case-sensitive name,
including bare `--`, and every valid descriptor occurrence in authored order.
`font-family` is required: a nonempty list of named families or a whole-value
substitution-dependent occurrence satisfies authored presence. Unquoted generic
family keywords are invalid; quoted spellings are literal family names. An
invalid descriptor is dropped independently, but a definition with no retained
family descriptor is discarded. `base-palette` accepts `light`, `dark`, an exact
nonnegative integer token, or symbolic Integer-root math. `override-colors`
accepts a nonempty comma-separated list of nonnegative indices and absolute
colors; repeated indices are retained, not resolved. Missing optional
descriptors remain absent. This follows [pinned Fonts 4 §9.2](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-palette-values)
and [pinned Color 5's absolute-color definition](https://www.w3.org/TR/2026/WD-css-color-5-20260908/#absolute-color).

Valid `var()` or `env()` anywhere in a descriptor defers its entire value before
ordinary grammar. `CssFontPaletteDescriptorValue::try_new` checks supplied
components without replacing their parsed or programmatic origins. A pending
value's `reparse_after_substitution` accepts caller-supplied components and
rejects any residual, including nested or escaped, substitution function before
checking the ordinary descriptor grammar. No substitution environment, font
lookup, palette execution, or root-element evaluation is supplied here.

`CssFontPaletteDescriptorValue::to_specified_css` and
`CssFontPaletteValuesRule::to_specified_css` produce canonical specified text
without changing their retained component syntax. They preserve duplicate
descriptors and override pairs, normalize keywords and exact integers, and use
the shared numeric and color serializers; pending values keep their complete
token stream. Their `_with_limits` forms charge input, projection, and bytes
across the whole value or rule. The shared `CssRule` and `CssSheet` specified
writers compose supported palette rules under one cumulative budget, joining
sheet rules with a newline; unsupported rule kinds and legacy encoding metadata
return typed errors rather than partial output. Palette rule brace spacing,
descriptor punctuation, and sheet joining are deterministic product policy:
the selected CSSOM does not define a palette-specific `cssText` algorithm.
Ordinary media/supports/container/layer/scope rule lists retain the authored
palette with its structural parent; a style-rule ancestor forbids it. No live
CSSOM or contextual color/profile evaluation is implied.

### Named-instance, language and metric descriptors

The selected [Fonts 4 §4.7](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-named-instance)
`font-named-instance` descriptor accepts `auto` or any string, including an
empty or unknown name. `CssFontNamedInstance::String` contains a checked
`CssFontNamedInstanceString`; its `try_new` and `try_from_component` preserve
respectively programmatic and parsed string provenance. `as_str`, `component`
and `origin` expose the decoded name and its retained source. Checked decoded
NUL is rejected by the shared string component owner; parsed CSS NUL follows
CSS replacement semantics.

[§4.10](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-language-override-desc)
`font-language-override` accepts `normal` or a string and reuses
`CssFontLanguageOverride` and `CssFontLanguageString` from the property owner.
Neither string descriptor performs language-tag validation or font matching.

The [§4.11 metric descriptors](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-metrics-override-desc)
`ascent-override`, `descent-override` and `line-gap-override` each contain
`CssFontMetricOverride`: `Normal` or a checked `CssSpecifiedNonNegativePercentage`.
Zero and percentages above 100 are valid; negative ordinary percentages and
unitless numbers are invalid. Percentage-typed math stays authored, including
`calc(-10%)`; [Values 4 range checking](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-range)
does not clamp specified calculations. Frozen
[WebKit descriptor grammar](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSProperties.json#L14698)
corroborates the metric grammar; its absence of the two string descriptors
does not replace their selected Fonts 4 definitions.

All five kinds use the existing raw and checked component front doors, ordered
occurrence retention, descriptor-local recovery, whole-value `env()` deferral
and strict substituted reentry. Standalone `var()`, CSS-wide keywords and
`!important` are invalid descriptor grammar. Normalization preserves these
payloads, their original positions and recovery diagnostics.

The new named-instance string/enum and metric payloads expose
`serialize_specified()` and `_with_limits()`; language emission uses its existing
provider. Keywords emit lowercase and strings use shared CSSOM escaping.
Metrics reuse the exact percentage/calculation provider. Keyword/string payloads
charge one input and projection node; metric percentages reuse their provider's
complete visits with no extra enum-dispatch node. Output uses one actual UTF-8
byte budget and failure returns no partial text or input mutation. The same
providers serve [checked Fonts rule composition](#checked-fonts-rule-composition).

`@font-face` retains every valid descriptor occurrence in authored order;
`effective(kind)` returns the last admitted ordinary or pending occurrence.
Source-list grammar follows the selected Fonts 4 edition. Weight and style
property and descriptor grammars now cite its September 7, 2026 edition; other
descriptor and property records retain their individual dated sources. An invalid
or unknown descriptor is dropped
with a `DropDescriptor` diagnostic without erasing valid neighbors. Empty rules
and rules missing `font-family` or `src` remain valid authored syntax. Those two
effective lookups return `Option`; their absence excludes the face from downstream font
matching under the pinned Fonts 4 §4.1, rather than causing a grammar error.

`unicode-range` checks the original token sequence before interpreting the
tokens' original spellings, following
[Syntax 3 §7.1](https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#urange).
Comments can separate tokens without changing their identity: `u+1/**/2`
denotes U+0012, while `u+0/**/-ff` fails the token grammar. Whitespace is
permitted around list items but not inside a range. Numeric spellings are
interpreted as hexadecimal text, so `u+12e-130` denotes U+012E through U+0130.
An invalid member drops the whole descriptor occurrence, preserving earlier
valid occurrences and neighboring descriptors.

Unicode-range diagnostics pair the responsible original token's start position
with its exact spelling, including escapes and whitespace. A malformed
representation points to the token containing its first invalid character;
endpoint order or domain errors identify the token containing the invalid end
endpoint. Wildcard domain errors identify the first token contributing to that
endpoint. A genuinely missing token or endpoint retains the bounded input-end
position with no encountered token. Recovery spans still cover the entire
dropped descriptor.

Under the selected Fonts 4 source-list processing rules, each comma-separated
`src` member is validated independently. Invalid members receive
`DropFontSourceListItem` diagnostics while valid fallbacks keep their authored
order. If every member is invalid, or the enclosing descriptor has an invalid
declaration annotation, the descriptor receives one `DropDescriptor` diagnostic;
earlier valid descriptor occurrences remain available. Any such recovery makes
the report unclean, so `validate_sheet` rejects it.

Empty URL strings remain valid authored sources. `url()` and `url("")`
retain the empty string, while `url("  ")` preserves its quoted whitespace.
`CssFontFaceUrlSource::new` accepts a shared `CssUrl`, an optional `CssFontFormat`,
and authored technology hints without resource resolution or loading.

Each `format()` accepts exactly one recognized keyword or one quoted string.
`CssFontFormat::Keyword` preserves keyword identity; `CssFontFormat::String`
contains a `CssFontFormatString` whose infallible `new` constructor preserves
decoded case, whitespace, empty and unrecognized strings. A missing format is
`None`; an empty string is a present string variant. Multiple arguments discard
the source member. Technology hints retain authored order and repetitions.

`format()` returns the single authored format. `CssFontFormat::recognized_format`
derives recognition without caching or deciding resource support. The four CSS
legacy variation strings `woff2-variations`, `woff-variations`,
`truetype-variations`, and `opentype-variations` recognize their base formats
using ASCII-insensitive matching without trimming. `required_technologies()`
yields distinct authored technologies in first-occurrence order, followed by
implied `variations` when absent. `tech()` retains its authored repetitions.
`CssFontFormatHint::is_equivalent_to` recognizes TrueType/OpenType equivalence
while ordinary equality keeps their identities distinct.

The `font-family`, `font-weight`, `font-style` and `font-width` properties and
descriptors, the `font-size` property, `font`, `@font-face`, `src`, font-source
and modern-source-hint records cite the September 7, 2026 edition as
`I-FONTS4-20260907`. The family property and descriptor and the narrowly named
modern-source-hint record are `Complete`; the weight, style and width properties
and descriptors and the size property are also `Complete` for their authored
grammars. The shorthand and source-list records are `Complete` for their
authored grammars; the selected authored `@font-face` rule is `Complete`, including
checked assembly and canonical effective specified serialization.
The older `I-FONTS4` identity keeps its April 22 edition; `O-FONTS3` also
remains available for historical source records. These immutable identities
must not be repointed when adopting a newer production.

The historical `baseline.property.font-stretch` and
`baseline.descriptor.font-stretch` IDs now identify canonical `font-width`;
`font-stretch` is a name-equivalent legacy alias in each separate namespace.
The historical `ext.descriptor.font-stretch-range` ID is `Complete` for its
ordinary width-range subproduction. The property accepts all nine width keywords and
nonnegative percentages, preserves authored numeric and math values, and has an
inherited `normal` initial value. CSS provides the exact keyword-to-percentage
mapping without resolving a font face. The `@font-face` descriptor accepts `auto` or
one or two ordinary width values under either descriptor name and retains the
authored endpoint order. Its initial is `auto`. All fourteen recognized
`@font-face` descriptor kinds admit valid `env()` as a pending whole value before
their ordinary grammar; standalone `var()` does not qualify. The pending value
retains its complete component stream and original token origins. CSS does not
perform environment lookup or substitution. Computed endpoint ordering and face
selection remain downstream. Direct
specified-value serializers cover these width values and feed the shared
Fonts rule writer. The `font` shorthand continues to accept
only the Fonts 3 width keywords. The breaking public property migration replaces
`CssKnownProperty::FontStretch` and `CssKnownPropertyValueRef::FontStretch` with
their `FontWidth` variants; the wrapper's `value()` returns `CssFontWidth`.
The shorthand uses the nine-keyword `CssFontWidthKeyword` model.
The descriptor migration replaces `CssFontFaceDescriptorKind::FontStretch` with
`CssFontFaceDescriptorKind::FontWidth` and the ordinary
`CssFontFaceDescriptorValue::FontStretch` with
`CssFontFaceDescriptorValue::FontWidth`. `CssFontFaceDescriptorRef` and
all eight per-kind aggregate getters, including `font_stretch()`/`font_width()`,
are removed. `CssFontFaceDescriptors::occurrences()` now borrows ordered
`CssFontFaceDescriptor` records; `effective(kind)` returns the last admitted
record of that kind, including a pending value rather than an earlier ordinary
one. Each record exposes `value()` and an optional descriptor-name `position()`;
checked programmatic records have no fabricated name position.
`CssFontFaceWidth::Auto` and `CssFontFaceWidth::Range` with exact checked values
replace the old f32-backed `CssFontFaceStretch`, `CssFontFaceStretchValue`, and
`CssFontFaceStretchKeyword` models. CSS owns authored width values and intrinsic
keyword mapping; style and text own contextual font use, and the root facade
owns cross-crate lowering. Raw descriptor-value parsing now returns
`CssAuthoredFontFaceDescriptorValue::Ordinary` or `Pending`; the existing
`CssFontFaceDescriptorValue` now includes the variation-settings ordinary variant.
`CssAuthoredFontFaceDescriptorValue::try_from_components` checks a whole value
while preserving supplied parsed, programmatic or mixed origins. Only a pending
value exposes `reparse_after_substitution`, which checks caller-supplied
replacement components and returns an ordinary value; residual `var()` or
`env()`, recovered components and invalid ordinary grammar are typed failures.
This API never executes substitution. The authored `font` shorthand supplies
checked values, intrinsic expansion/reset contributions and bounded specified
serialization. Named-instance, language and metric descriptors use the complete
[shared payload providers](#named-instance-language-and-metric-descriptors).
The source-list URL branch accepts both `url()` and `src()` from the referenced
Values 4 `<url>` production, retaining their authored function identity and
ordered modifiers without resolving or loading the resource. Other historical
Fonts 3 and Fonts 4 support records
retain their existing classifications pending reconciliation with the complete
selected profile; their dates bound those claims. These authored models do
not load or match fonts, resolve fallback or feature application, shape glyphs,
apply cascade or substitution, evaluate computed values, expose live CSSOM, or
lower into another Surgeist crate. Checked composed Fonts rules use the
[shared specified writer](#checked-fonts-rule-composition).

### Grammar providers for font-loading adapters

The selected [Font Loading 3 constructor](../../../references/css-font-loading-3--WD-css-font-loading-3-20230406--1a443ea0a7f6.md#font-face-constructor)
references individual `@font-face` descriptor grammars. Use
`parse_font_face_descriptor_value` for original-string reports and
`CssAuthoredFontFaceDescriptorValue::try_from_components` for strict checked
values. The latter rejects recovered `src` members and implicit closures;
pending descriptor reentry rejects residual substitution and invalid ordinary
grammar. Raw parsing preserves valid recovery syntax and diagnostics, including
EOF-ended strings. A bare source URL is invalid; the `src` descriptor accepts
source lists containing `url()`, `src()` or `local()` branches.

Every ordinary `CssFontFaceDescriptorValue` exposes `serialize_specified()` and
`serialize_specified_with_limits()`. These dispatch all fourteen concrete kinds,
including display and unicode range, through the same Fonts writer used by
checked rule composition. They produce individual attribute-value strings
without constructing a synthetic rule. String/numeric/URL payloads reuse their
existing providers, including equal-range shortening and authored URL function
identity. Input and projection visits accumulate across every retained child;
only emitted UTF-8 bytes consume the final byte budget. Failure returns no
partial CSS and preserves values and origins. Pending descriptor values retain
their existing component and substitution-reentry owner rather than acquiring
an ordinary serializer.

For the [font-query algorithm](../../../references/css-font-loading-3--WD-css-font-loading-3-20230406--1a443ea0a7f6.md#find-the-matching-font-faces),
call `parse_property_value_text` with
`CssPropertyNameRef::Known(CssKnownProperty::Font)` and `CssImportance::Normal`,
then inspect `known().declared_value()`. This parses
the original value without adding a declaration-name prefix. Ordinary `Font`
values expose the existing explicit or system branch and bounded authored
serialization. Relative sizes and weights stay symbolic. CSS-wide keywords
and substitution-dependent `var()`/`env()`/`attr()` values are distinct authored
branches; valid parsing does not promise they are usable loading queries.
Loading adapters reject CSS-wide keywords as the pinned source requires and
select ordinary-only admission for unresolved substitutions. This integration
policy uses no inherited-page substitution environment or duplicate font parser.

Loading adapters also select `into_validation_result()` for clean-report
admission: it accepts exactly reports with zero recovery diagnostics. This is
an explicit Surgeist application policy, not a claim that every EOF recovery
is a Font Loading grammar mismatch. Component-based `Font` construction alone
does not establish absence of recovery. The shared parser continues to expose
valid recovered syntax and original diagnostics.

Loading 2023 retains a `variant` IDL field, but selected
[Fonts 4 removes the `font-variant` descriptor](../../../references/css-fonts-4--WD-css-fonts-4-20260907--03626a0c8565.md#changes-2018-09-20).
CSS therefore supplies no corresponding descriptor kind, property-grammar
substitute or compatibility shim. Root must retain this source-version gap
before claiming complete variant constructor/setter coverage. The selected
`font-stretch` legacy descriptor name remains a real alias of `font-width`.

Root-owned [face integration](https://github.com/bj-data-eng/surgeist/issues/768)
must convert parse failures to rejection of `[[FontStatusPromise]]` with
`SyntaxError`, empty corresponding attributes and error status, then return the
face and terminate construction. Success stores serialized descriptor strings
and returns the face before asynchronously assigning `[[Urls]]` or `[[Data]]`
and queuing binary parsing. The pinned URL-base question, including workers,
remains open. Attribute synchronization and CSS-connected identity also stay
with that owner. Root-owned
[query integration](https://github.com/bj-data-eng/surgeist/issues/769) must
absolutize relative values against initial properties, including `bolder`
against initial `normal`; default omitted text to U+0020 SPACE; select available
and optional system faces; set the found-faces flag before unicode/text
filtering; and return ordered faces and that flag. Empty text filters all faces.
Shared grammar completion does not implement matching, loading, binary parsing,
CSS-connected object synchronization or those asynchronous algorithms.

### Authored font-synthesis

The [selected Fonts 4 working draft, September 7, 2026, §§2.8.1–2.8.5](../../../references/css-fonts-4--WD-css-fonts-4-20260907--03626a0c8565.md#font-synthesis-weight)
defines the complete synthesis shorthand and four longhands. `font-synthesis`
accepts `none` or a nonempty set of nonrepeated `weight`, `style`, `small-caps`,
and `position` keywords in any order. `CssFontSynthesisValues::try_new` takes
four booleans in that order and rejects an empty set; use `CssFontSynthesis::None`
for `none`. Its getters expose each capability without computing a font result.
Existing two-argument callers must supply the `small_caps` and `position` arguments.

| Property | Checked value | Keywords | Intrinsic initial | Inherited |
| --- | --- | --- | --- | --- |
| `font-synthesis-weight` | `CssFontSynthesisWeight` | `auto`, `none` | `auto` | yes |
| `font-synthesis-style` | `CssFontSynthesisStyle` | `auto`, `none`, `oblique-only` | `auto` | yes |
| `font-synthesis-small-caps` | `CssFontSynthesisSmallCaps` | `auto`, `none` | `auto` | yes |
| `font-synthesis-position` | `CssFontSynthesisPosition` | `auto`, `none` | `auto` | yes |

The shorthand contributes all four members in weight/style/small-caps/position
order with no reset-only members. Included capabilities contribute `auto`;
omitted capabilities contribute `none`, including all four for shorthand `none`.
Its initial is the full four-capability set. `oblique-only` is a style-longhand
choice and cannot be expressed by this shorthand. The `font` shorthand leaves
synthesis properties independent.

`CssFontSynthesis` and each longhand type expose `serialize_specified()` and
`serialize_specified_with_limits()`. They emit lowercase keywords, with shorthand
capabilities in grammar order. Each emitted keyword charges one input node and
one projection node; spaces and keywords share one cumulative byte budget.
An exhausted budget returns a typed error with no partial CSS and preserves the
checked model and declaration occurrence. Authored text and exact origins remain
available through the property wrappers and declaration components.

```rust
use surgeist_css::{CssFontSynthesis, CssFontSynthesisValues};

let value = CssFontSynthesis::Values(
    CssFontSynthesisValues::try_new(true, false, false, true).unwrap(),
);
assert_eq!(value.serialize_specified().unwrap(), "weight position");
```

All five properties use the shared CSS-wide and whole-value pending lifecycle.
Strict substitution reentry validates the original property's grammar and retains
the original occurrence, importance, and replacement components. Normalization
retains declaration order and counts each of the four shorthand contributions
against its cumulative budget. See the
[executable synthesis example](../examples/font_synthesis.rs).
Font selection, italic fallback, glyph synthesis, cascade, and inheritance
execution belong to downstream style, text, and render owners.

### Authored font-palette

The [selected Fonts 4 working draft, September 7, 2026, §9.1](../../../references/css-fonts-4--WD-css-fonts-4-20260907--03626a0c8565.md#font-palette-prop)
defines `font-palette` as `normal`, `light`, `dark`, a case-sensitive dashed
palette identifier, or `palette-mix()`. The intrinsic initial is `normal` and
the property is inherited. It contributes one longhand independently of `font`;
the font shorthand neither sets nor resets it.

`CssFontPalette` retains these alternatives. `Named` uses the same checked
`CssFontPaletteName` as `@font-palette-values`, including bare `--`.
`CssFontPaletteMix::try_new` accepts an optional `CssColorInterpolation` and
a nonempty ordered list of `CssFontPaletteMixComponent` values. Each component
holds a palette and an optional `CssColorMixWeight`: an exact literal percentage
in `[0,100]` or checked symbolic Percentage-root math. Parsed percentages may
precede or follow the palette; the checked model retains palette order,
duplicates, interpolation omission, nested mixes and supplied numeric origins.
Construction rejects empty lists and complete palette/math graphs exceeding
the shared 256-level structural ceiling.

`CssFontPalette::serialize_specified()` and `serialize_specified_with_limits()`
emit canonical authored CSS. Omitted interpolation denotes Oklab;
omits an explicit default Oklab method from output, and emits each palette before
its percentage. It shares interpolation, exact percentage formatting and omitted
weight handling with `color-mix()`: equal effective `100/N` shares can be omitted;
otherwise missing literal weights are filled from the exact capped specified sum.
Explicit literal shares are never rescaled. Any calculated weight leaves missing
weights unknown, retaining its calculation rather than normalizing computed
weights. Generated rational shares use the accepted six-place percentage policy.
Singleton and repeated-palette mixes remain functions; equivalence simplification
belongs to computed values downstream.

The iterative writer shares one cumulative input, projection and byte budget
across the complete graph, including interpolation and weight projection work.
Each palette visits one input and one projection node. Exhaustion produces a
typed error without partial output or changes to the authored graph. CSS-wide
and whole-value pending values use the shared property lifecycle. Strict reentry
checks complete replacement grammar and rejects residual substitutions and
implicit closures while retaining the original occurrence, importance and
replacement origins.

```rust
use surgeist_css::{CssFontPalette, CssFontPaletteMix, CssFontPaletteMixComponent};

let value = CssFontPalette::Mix(Box::new(CssFontPaletteMix::try_new(None, vec![
    CssFontPaletteMixComponent::new(CssFontPalette::Light, None),
    CssFontPaletteMixComponent::new(CssFontPalette::Dark, None),
]).unwrap()));
assert_eq!(value.serialize_specified().unwrap(), "palette-mix(light, dark)");
```

See the [executable palette example](../examples/font_palette.rs). Palette lookup,
font selection, per-index color mixing, contextual resolution, cascade and
inheritance execution remain with downstream owners.

## Conformance sources and atomic records

The conformance source registry assigns every selected dated specification or
preserved repository baseline a stable `CssSpecificationSourceId`, module,
level, and `CssSpecificationTier`. The tier classifies provenance only; it does
not imply parser support. A source has exactly one immutable URL or repository
provenance value. `specification_source`, `feature_metadata`, and
`conformance_exclusion` perform exact, case-sensitive lookup without trimming
or aliasing.

```rust
use surgeist_css::{
    CssExclusionReason, CssSpecificationTier, CssSupportStatus,
    conformance_exclusion, feature_metadata, specification_source,
};

let color = specification_source("O-COLOR4").expect("dated Color 4 source");
assert_eq!(color.tier(), CssSpecificationTier::Snapshot2026Official);
assert!(specification_source("o-color4").is_none());

let importance = feature_metadata("foundation.declaration.importance")
    .expect("atomic parser-facing record");
assert_eq!(importance.status(), CssSupportStatus::Complete);
assert!(importance.baseline_alias_targets().is_empty());

let pseudo_elements = feature_metadata("baseline.selector.pseudo-element")
    .expect("preserved aggregate alias");
assert_eq!(
    pseudo_elements.baseline_alias_targets()[0].as_str(),
    "official.selector.generated",
);

let processing = conformance_exclusion("excluded.O-IMAGES3.processing")
    .expect("official source exclusion");
assert_eq!(
    processing.reason(),
    CssExclusionReason::OutsideAuthoredSyntaxBoundary,
);
```

An atomic feature record is parser-facing and carries one truthful
`CssSupportStatus`. The four preserved baseline aggregate aliases remain
queryable and expose their immutable atomic target slices, but they do not own
parser dispatch. A private reserved coverage slot identifies a later grammar
boundary only: it is not a feature record, has no support status, and does not
make its spelling recognized. An exclusion is a public source-audit fact for an
informative, superseded, or out-of-boundary source item; it likewise carries no
support status and never changes parser diagnostics. Adding registry metadata,
aliases, reserved slots, exclusions, or implementation inventories does not
change accepted CSS, retained syntax, diagnostics, positions, spans, or
recovery actions.

`Partial` means only the record's documented subset is implemented.
`supported_subset()` and `unsupported_remainder()` are present exactly for
partial records. The latter names scope not claimed as supported: it can be a
known-valid implementation gap or explicitly unresolved applicability in the
selected standard. Consumers must not infer validity from `Partial` or a
nonempty remainder. Use parsing for implemented acceptance and the cited
standard for normative validity. This is a semantic migration for consumers
that previously treated every remainder as valid-but-unimplemented syntax.

## Namespaces and complete Selectors 3 syntax

`CssRule::Namespace` retains a top-level `@namespace` declaration with its
optional decoded, case-sensitive `CssNamespacePrefix`, literal
`CssNamespaceName`, and optional source position. Namespace names preserve the
authored string or `url()` token value, including empty strings and strings that
are not valid URIs. The crate does not normalize, resolve, or load the value.

`CssNamespacePrefix::try_new()` checks a decoded identifier value through the
component-value identifier owner. Values such as `"1"` and `"a b"` are valid
because CSS identifier escaping can represent them; an empty value or a value
containing NUL is rejected. `CssNamespaceRule::new(prefix, name)` consumes these
checked types and produces an intrinsically valid declaration without a source
position. `position()` returns `None` for constructed declarations and `Some`
for parsed declarations. Callers reading a parsed rule's position must unwrap
that optional authored origin.

Namespace declarations precede ordinary rules, following any imports and
permitted layer statements. Within a sheet, each repeated decoded prefix or
default declaration produces `CssErrorCode::NamespaceRedeclaration` with
`CssRecoveryAction::RetainNonconformingRule`. The error's typed detail exposes
the prefix and preceding retained declaration's position. Both declarations
remain in authored order, and the last binding is effective even when its name
is empty or agrees with the preceding name. Normalization preserves the
diagnostics and retained rules; clean-report validation rejects the
nonconforming sheet.

Selector type, universal, and attribute names expose
`CssNamespaceConstraint`. `Named` contains an earlier active prefix;
`ExplicitNone` represents `|`; `Any` represents `*|`; and `Default` represents
an unqualified type or universal selector while a default declaration is
active. Without an active default, an unqualified type or universal selector is
`Any`. Unqualified attributes are always `ExplicitNone`. A
`CssQualifiedSelectorName` distinguishes an identifier returned by
`local_name()` from universal `*` reported by `is_universal()`.

```rust
use surgeist_css::{
    CssNamespaceConstraint, CssPseudoElement, CssPseudoElementSegment, CssRule, CssSelector, parse_sheet,
};

let report = parse_sheet(concat!(
    "@namespace svg \"urn:svg\";",
    "svg|a#first#second[|lang]::first-line { color: red; }",
));
assert!(report.is_clean());
let [CssRule::Namespace(namespace), CssRule::Style(style)] =
    report.syntax().rules()
else {
    panic!("expected namespace and style rules");
};
assert_eq!(namespace.prefix().expect("named prefix").as_str(), "svg");
assert_eq!(namespace.name().as_str(), "urn:svg");

let CssSelector::Compound(selector) = style.selectors().selectors()[0].selector() else {
    panic!("expected compound selector");
};
let qualified = selector.type_selector().expect("qualified type selector");
assert!(matches!(
    qualified.namespace(),
    CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
));
assert_eq!(qualified.local_name(), Some("a"));
assert_eq!(selector.ids(), ["first", "second"]);
let [attribute] = selector.attributes() else {
    panic!("expected one attribute selector");
};
assert_eq!(attribute.namespace(), &CssNamespaceConstraint::ExplicitNone);
assert!(matches!(
    selector
        .pseudo_elements()
        .expect("pseudo-element sequence")
        .segments(),
    [CssPseudoElementSegment::PseudoElement(CssPseudoElement::FirstLine)]
));
```

`CssPseudoElementSequence::segments()` returns the complete ordered sequence of
`CssPseudoElementSegment::PseudoElement` and `PseudoClass` entries. A pseudo-class
entry applies to the most recent pseudo-element. This replaces the old
`pseudo_elements()` slice accessor; migrate consumers by matching both segment
variants. `CssPseudoElement` now owns functional arguments, so it implements
`Clone` and `PartialEq` instead of `Copy` and `Eq`. The checked plain-element
`try_new` constructor remains available; `try_from_segments` checks complete
sequences.

`CssCompoundSelectorArgument` preserves one checked compound for `:host()`,
`:host-context()` and `::slotted()`. Compound restrictions propagate through
`:is()`, `:where()` and `:not()`; relative `:has()` arguments use their own grammar.
`CssPartNameList` retains ordered, case-sensitive identifier components and
provides canonical argument serialization. Parsed names expose source origins;
constructed names expose programmatic origins. General selector serialization
is a separate foundation requirement.

Initial layer statements may precede both imports and namespaces, after any
encoding declaration. Imports must precede namespaces. A layer statement after
either imports or namespaces, or a body rule, prevents subsequent imports and
namespaces. This follows the placement extension in
[Cascade 5 §6.4.4.2](https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#layer-empty).
Malformed, ignored, nested, or misplaced rules never change the phase or active bindings.

Declarations and active bindings remain in authored order; the last declaration
for an exact named prefix or the default affects following selectors. An
undeclared named prefix invalidates its selector. Forgiving `:is()` and
`:where()` lists drop only that member with `DropSelectorListItem`; unforgiving
style, scope, nesting, `:not()`, `:has()`, and nth `of` consumers drop their
established containing unit. Malformed, block-form, nested, or late namespace
rules recover as one `DropAtRule` and leave later siblings eligible.

Style rules preserve one authored node for their complete selector list.
`CssStyleRule::selectors()` returns `CssStyleSelectorList`; its `selectors()` slice contains
`CssStyleSelector::Selector` or, in nested contexts, `CssStyleSelector::Relative` with its
leading combinator. This replaces the former singular `CssStyleRule::selector()` accessor.
Read each list member explicitly instead of assuming a separate rule for each selector.

`CssStyleRule::declarations()` contains only leading declarations. `rules()` preserves child
rules and later `CssRule::NestedDeclarations` runs in source order. A nested declarations
rule exposes its validated nonempty `declarations()` and the first declaration's `position()`;
it inherits the containing style rule's selector context without inventing an `&` selector.
A rejected complete qualified rule or any at-rule boundary transfers a preceding
nonempty declaration run, even when no child rule is retained. Later runs remain
separate nodes, including adjacent declaration-run nodes. A rejected first rule
with no preceding declarations leaves the leading slot available. Qualified
parsing that produces no rule, such as a semicolon-terminated prelude or the
custom-property-looking fallback, leaves the current declaration run intact.
`CssCompoundSelector::nesting_selectors()` records symbolic parent anchors independently of
`scope_anchors()`, which preserves scope-anchor multiplicity; `has_scope_anchor()`
remains its nonzero predicate. Explicit `:scope` remains a separate pseudo-class.
For a style-nested `@scope`, root `&` refers to the nearest ancestor style;
limit `&` refers to the introduced scope. An ordinary root inside another scope
uses that enclosing scope, while a root without either ancestor retains a
parentless nesting anchor. Repeated anchors remain distinct, including within
functional selector arguments. Boundary lists exclude pseudo-elements. A
`CssScopeSelectorList` preserves each `CssScopeSelector::Selector` or
`Relative(CssRelativeSelector)` member, including its leading combinator without
an invented explicit anchor. Limits admit relatives in every context, relative
to the introduced scope. Roots admit relatives with a style or scope ancestor;
ordinary top-level roots reject them. Cascade 6 §2.5.5 supplies the distinct
nesting and scoping contexts, also reflected in Nesting 1 §3.2 examples.
`CssScopeRule::try_new` takes an explicit `CssScopeNestingContext`; checked
sheet/group assembly revalidates the actual ancestry, identifiers, namespaces
and depth without changing supplied positions or anchor categories. This enum
is a validation context, not a replacement for ordered ancestry. `Style` applies
whenever a style ancestor exists, including with an intervening scope, for
explicit root `&` and declaration runs. An implicit relative root instead binds
to the nearest enclosing style or scope in the normalized rule-parent chain;
the limit binds to the introduced scope. No parent
selector list is expanded during parsing. Shared canonical scope output remains
with the unfinished rule/stylesheet serializer.
`CssScopedStyleRule` follows the same
leading-declarations and ordered-child contract: its `rules()` returns ordinary `CssRule`
nesting children relative to the scoped style parent. A scope with a style ancestor retains
its direct declaration runs as `CssScopedRule::NestedDeclarations`, including runs inside
its conditional and layer groups. Ordinary scope/group bodies without style ancestry
continue to use their rule-list grammar. Scoped runs reuse the exact ancestor style
selector handle, including pseudo-elements, while their independent rule-parent chain
retains all active scope/group occurrences. The selector handle's `scope_context()` is
its original binding scope, not a replacement for that complete rule ancestry. A scoped
child style establishes its own selector handle; later sibling runs retain the enclosing
style's handle.

The authored selector model covers complete Selectors 3, including universal
and type selectors, all attribute matchers, repeated IDs and classes in order,
the structural/UI/dynamic pseudo-class families, `:lang()`, all four
combinators, and `::first-line`/`::first-letter`. The legacy single-colon
spellings for `before`, `after`, `first-line`, and `first-letter` map to the same
typed pseudo-elements. Selected extensions remain separately owned: attribute
`i`/`s`, the existing extension-state and functional pseudo-classes, nesting and
scope, and the marker/selection/backdrop pseudo-element rows. Matching,
specificity, cascade, namespace URI resolution, CSSOM serialization, and
cross-crate lowering remain downstream exclusions.

## Counter Styles 3 and CSS2 page rules

`CssRule::CounterStyle` retains a checked, case-sensitive
`CssCounterStyleName`, the parser-produced rule position, and typed
`CssCounterStyleDescriptors`. Every valid descriptor occurrence remains in
authored order; the named `system`, `negative`, `prefix`, `suffix`, `range`,
`pad`, `fallback`, `symbols`, `additive_symbols`, and `speak_as` accessors select
the effective last valid occurrence. The model preserves symbolic
`extends` names, infinite range bounds, nonempty symbol lists, and strictly
descending additive weights without registering, resolving, inheriting, or
evaluating a counter style.

The optional `fixed` starting value is `Option<CssIntegerLiteral>`, preserving
omission separately from explicit `1` and `+0001`. Finite range bounds retain
`CssIntegerLiteral` beside contextual `Infinite`. `CssCounterStyleRangeInterval::try_new`
rejects mathematically reversed finite bounds, and `CssCounterStyleRanges::try_new`
requires a nonempty list while preserving authored interval order.
`CssCounterStylePad::try_new` and `CssCounterAdditiveTuple::try_new` accept checked
nonnegative integer tokens, including signed zero, without machine-sized limits.
`CssCounterAdditiveSymbols::try_new` requires a nonempty list with strictly
descending mathematical weights, rejecting equal alternate spellings without
sorting or deduplication. These owners retain exact token components and origins.
The selected Counter Styles standard permits supported-range clamping; this
specified-value model preserves exact authored bounds and leaves any implementation
range policy downstream. Counter integer fields remain literal-only; this model
does not add a counter-rule serializer.

An invalid or unknown counter-style descriptor is dropped individually with a
typed `DropDescriptor` diagnostic, preserving valid neighboring descriptors.
An invalid effective combination, such as `system: extends` with an authored
`symbols` definition, drops the complete at-rule. Counter-style rules are
block rules admitted at stylesheet level, in ordinary conditional and layer rule
lists, and in ordinary scopes. Malformed preludes, statement forms and placement
beneath a style-rule ancestor drop the smallest established at-rule unit and
leave later siblings eligible.

Ordinary scope bodies also retain `@font-face` and `@keyframes` definitions.
`CssScopedRule::{CounterStyle, FontFace, Keyframes}` carries the same payload as
its ordinary `CssRule` counterpart. Normalization records each definition once
under its authored parent context; scope does not create a new definition
identity or localize these names. Counter-style names retain their tree-scoped
semantics. Conditional application, font loading and name lookup belong
downstream. Definitions remain invalid beneath style-rule ancestors, including
through nested scope, media, supports, container and layer groups. These
placement contracts follow the selected
[Conditional Rules 3](https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/),
[Cascade 6 scope nesting](https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-nesting)
and [Nesting 1](https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#conditionals)
editions.

`CssRule::Page` retains the default page form or one of the finite
`CssPageSelector::{Left, Right, First}` choices, valid declarations in authored
order, and the parser-produced position. Page bodies accept only `margin` and
the four margin longhands with CSS2 lengths other than `em` and `ex`,
percentages, `auto`, zero, and negative values. Known non-margin, unknown, and
invalid margin declarations receive their existing typed declaration
diagnostics and are dropped individually. Block-form page rules are accepted at
the stylesheet top level and inside ordinary conditional and layer rule lists.
Inside scope, ordinary conditional and layer bodies retain the same payload as
`CssScopedRule::Page`; page selectors do not become scoped element selectors.
A directly enclosing scope body rejects a page under the selected Syntax 3 and
Cascade 6 content-category interpretation. Entering another scope restores that
body restriction. Pages remain invalid in style-rule bodies, including all group
chains with a style ancestor. Normalization retains a page leaf and its authored
parent without emitting its margin declarations as element-style declarations.
Margin-box nested at-rules remain unsupported.

Ordinary conditional and layer rule lists consume semicolon-prefixed input as
a qualified rule. For example, `@media all { a {} ;b {} c {} }` retains `a` and
`c`, with a `DropQualifiedRule` diagnostic for `;b {}`. Style bodies retain their
declaration grammar, where extra semicolons are valid separators.

```rust
use surgeist_css::{CssCounterStyleSystem, CssPageSelector, CssRule, parse_sheet};

let report = parse_sheet(concat!(
    "@counter-style digits { system: numeric; symbols: \"0\" \"1\"; suffix: \".\"; } ",
    "@page :left { margin-left: -12mm; margin-right: 10%; }",
));
assert!(report.is_clean());
let [CssRule::CounterStyle(counter), CssRule::Page(page)] = report.syntax().rules() else {
    panic!("expected counter-style and page rules");
};
assert_eq!(counter.name().as_str(), "digits");
assert!(matches!(
    counter.descriptors().system().map(|value| value.value()),
    Some(CssCounterStyleSystem::Numeric)
));
assert_eq!(counter.descriptors().occurrences().count(), 3);
assert_eq!(page.selector(), Some(CssPageSelector::Left));
assert_eq!(page.declarations().len(), 2);
```

All sixteen Counter Styles 3 non-property rows and the two CSS2 page rows are
public `Complete` atomic metadata with their dated official source fragments.
They have no partial remainder, recognized-unsupported code, or aggregate-alias
targets. The crate does not paginate, match page selectors, apply page cascade,
render generated markers, resolve counter inheritance, expose CSSOM, or lower
these authored models into another Surgeist crate.

## CSS2 residual, writing, UI, containment, and compositing properties

This property family provides the selected authored grammars for ten CSS2
residual properties, `quotes` in CSS Generated Content 3, the deprecated `clip`
property in Masking 1 Appendix A,
Writing Modes 3 `text-combine-upright`,
`text-orientation`, and `unicode-bidi`, UI3 `caret-color`, `outline-offset`, and
`resize`, Containment 1 `contain`, Transforms 1 `transform-box`, and Compositing
1 `background-blend-mode`, `isolation`, and `mix-blend-mode`. Their property
wrappers preserve exact authored CSS and expose typed current values without
performing cascade, layout, pagination, painting, hit testing, containment
semantics, blending, or writing-mode resolution.

[Masking 1 Appendix A](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#clip-property)
supersedes CSS2 §11.1.2 for `clip` while retaining required support for the
deprecated property. Its noninherited initial is `auto`. A `rect()` has four
top, right, bottom, left edges, each `auto` or a signed length; both comma-only
and whitespace-only authored separators are accepted. `CssSpecifiedLength`,
`CssClipEdge`, `CssClipRect`, and `CssClip` serialize their checked specified
values under one cumulative budget, with commas in canonical rectangles and
symbolic calculations left unresolved. Applying a clipping region and the
separate `clip-path` property remain outside this authored-value contract.

[CSS Generated Content 3 §2.4.1](https://www.w3.org/TR/2025/WD-css-content-3-20251204/#propdef-quotes)
defines `quotes` as an inherited longhand with initial `auto`. `CssQuotes`
retains `auto`, `none`, `match-parent`, or a nonempty ordered list of checked
opening/closing string pairs. Its bounded specified serializer escapes decoded
strings and charges one cumulative resource budget across every pair. Language
selection for `auto` and parent matching remain downstream.

[CSS Generated Content 3 §2](https://www.w3.org/TR/2025/WD-css-content-3-20251204/#propdef-content)
defines `content` as a noninherited longhand with initial `normal`. Its current
`CssContentValue` distinguishes `normal`, `none`, and checked generated content.
The generated body classifies a sole image as replacement, retains ordered
content items, and permits a separate nonempty alternative of strings and
counter functions. `CssCounterStyleValue` retains checked named styles or
`symbols()` with string/image symbols; all predefined style names normalize to
lowercase while custom names keep their case. Use
`CssContentPropertyValue::value()` for the complete authored grammar. A valid `attr()` keeps the whole declaration pending, as
described above.

Specified serialization uses one cumulative input-node, projection-node, and
CSS-byte budget. Generated and alternative lists, functions, and `symbols()`
each charge one aggregate plus checked children; nested URLs and images share
the same writer. Defined meaning-preserving defaults are omitted in canonical
text: decimal in `counter()`/`counters()`, first in `string()`, text in
`content()`, and symbolic in `symbols()`. Explicit omitted defaults still
consume node budget. Leader keywords emit their equivalent string forms.
Target-text selector and target-counter style omissions remain distinct from
explicit arguments. Counter lookup, target retrieval, repeated-content
processing, generated boxes, resources, and painting remain downstream.

[CSS Lists 3 §3](https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#propdef-list-style)
defines inherited `list-style-type`, `list-style-position`, and
`list-style-image` with `disc`, `outside`, and `none` initials. The `list-style`
shorthand resets omitted members and accepts type, position, and image in
their selected grammar, including a checked `symbols()` counter style and
Images 3 images. Each wrapper exposes `value()` for the complete authored
value.
`CssListStyleValue` retains which components were authored. Its bounded
specified serializer emits position, image, then type, omits only
meaning-preserving initials, and inserts `outside` when a custom type named
`inside` or `outside` would otherwise parse as a position. Explicit authored
initials still consume node budget. Marker construction, image loading, and
painting remain downstream.

[CSS Lists 3 §3.7](https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#propdef-marker-side)
defines inherited `marker-side` with initial `match-self`. Its authored
wrapper exposes the checked `CssMarkerSide` keyword; bounded specified
serialization emits `match-self` or `match-parent`. Selecting element or parent
directionality and marker placement remain downstream. `marker-side` is a separate
longhand and is not a fourth `list-style` shorthand member.

[CSS Lists 3 §§4–4.2](https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#counter-reset)
defines `counter-reset`, `counter-increment`, and `counter-set` as separate
noninherited longhands with `none` initials. `CssCounterChangesValue` retains
`none` or nonempty ordered checked entries, including duplicate names and
whether each integer was authored. The three property wrappers expose
`value()`. Names use the
Content 3 checked counter-name domain; integer tokens remain exact at any
magnitude, and integer calculations retain their authored tree and origin.
For canonical specified serialization, pass `CssCounterProperty` explicitly:
an omitted operand writes `0` for reset/set and `1` for increment. One bounded
writer charges the list, entries, names, and numeric subtrees cumulatively;
synthetic defaults consume projection and byte budgets without invented input
nodes. Counter execution, clamping, and duplicate application remain downstream.

[CSS Text 4 §§8.1–8.2](https://www.w3.org/TR/2026/WD-css-text-4-20260814/#word-spacing-property)
defines `word-spacing` and `letter-spacing` as inherited longhands with
`normal` initials and signed `<length-percentage>` values. Their shared
`CssTextSpacingAdjustment` retains exact numeric spelling, source origin, and
symbolic math, and serializes under a bounded resource budget. Percentages
inherit intact and resolve against used font size downstream. The historical
pure-length spacing models have no remaining grammar owner.
`CssWordSpacingPropertyValue::spacing()` and
`CssLetterSpacingPropertyValue::value()` borrow the shared checked value.

`glyph-orientation-vertical` is the selected Writing Modes legacy shorthand,
not a name-equivalent schema alias. Its implemented compatibility subset admits
decoded `auto`, unitless integer-flag `0` and `90`, and exact degree-valued
`0deg` and `90deg`, mapping to a parser-produced `text-orientation` declaration.
Degree dimensions must equal zero or ninety in their exact authored numeric
spelling: `9e1deg` is accepted, while `90.000001deg` and `1e-100deg` are rejected.
The schema therefore keeps `CssKnownProperty::TextOrientation.aliases()` empty,
while the conformance catalog exposes the explicit
`official.property-alias.glyph-orientation-vertical` record. That record is
`Partial`: the selected [Writing Modes 3 §5.1.3](https://www.w3.org/TR/2019/REC-css-writing-modes-3-20191210/#glyph-orientation)
and [Writing Modes 4 §5.1.3](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/#glyph-orientation)
specify the five terminals and mapping but leave numeric-terminal spelling and
math applicability without a defined bridge to the finite keyword target
([CSSWG issue 8032](https://github.com/w3c/csswg-drafts/issues/8032)). The parser
continues to enforce its documented compatibility subset; this metadata does
not declare every other spelling or math expression invalid CSS.

```rust
use surgeist_css::{
    CssBlendMode, CssFeatureKind, CssKnownProperty, CssKnownPropertyValueRef,
    CssSupportStatus, feature_metadata, parse_style_attribute,
};

let report = parse_style_attribute(concat!(
    "border-spacing: 2px 3px; ",
    "glyph-orientation-vertical: 90; ",
    "background-blend-mode: multiply, luminosity",
));
assert!(report.is_clean());
assert_eq!(
    report.syntax()[1].known().expect("legacy shorthand").property(),
    CssKnownProperty::TextOrientation,
);
let CssKnownPropertyValueRef::BackgroundBlendMode(blending) = report.syntax()[2]
    .known().expect("known blending property")
    .property_value().expect("ordinary value")
else { panic!("expected background blend modes") };
assert_eq!(
    blending.modes().modes(),
    &[CssBlendMode::Multiply, CssBlendMode::Luminosity],
);
let alias = feature_metadata("official.property-alias.glyph-orientation-vertical")
    .expect("legacy alias metadata");
assert_eq!(alias.kind(), CssFeatureKind::PropertyAlias);
assert_eq!(alias.status(), CssSupportStatus::Partial);
```

These 27 official rows are public atomic records: 24 canonical properties and
the independent `official.value.box-edge-keywords` and
`official.value.blend-mode` shared-value records are `Complete`; the explicit
legacy shorthand is `Partial`. This activation does not inflate the immutable
ledger or promote later work: it remains 162 property units (161 canonical
properties plus the custom property family), one normative legacy shorthand,
and 167 non-property units.
The 130-row exclusion registry still includes exactly 50 superseded
CSS2 property definitions, 20 informative CSS2 Appendix A properties, and the
two current-production-less `glyph-orientation-horizontal` and `ime-mode`
spellings; the remaining exclusions cover exact non-property or downstream
source areas.

## Backgrounds, border images, and gradients

Backgrounds 3 and Images 3 values remain authored and symbolic. Background
shorthands preserve layer order, per-layer position/size coupling, repeats,
attachments, boxes, and a final-layer color. Image values distinguish `none`,
URLs, and typed linear, radial, and repeating gradients. Border-image values
preserve their source, slice, width, outset, and repeat components without
loading an image or resolving any geometry.

The `background-size` and `mask-size` wrappers expose their ordered
`CssBackgroundSizeList` through `sizes()`; `background-repeat` and `mask-repeat`
expose `CssBackgroundRepeatList` through `repeats()`. `background-origin` and
`background-clip` expose the full `CssBackgroundBoxList` through `boxes()`, and
`background-attachment` exposes its ordered list through `attachments()`.
The five background longhands each contribute one noninherited terminal value,
preserving the authored list without aligning it to the image count. Their
single-layer initials are `auto`, `repeat`, `padding-box`, `border-box`, and
`scroll`, respectively. CSS-wide keywords remain symbolic; pending values
reenter their original property grammar with unchanged occurrence and importance.

`background-color` also contributes one noninherited terminal, retaining its
checked `CssColor` with transparent intrinsic initial. Its existing
`to_specified_css()` provider preserves symbolic color dependencies. CSS-wide
keywords remain symbolic; pending replacements reenter the selected color grammar
with the original occurrence and importance. This lifecycle support does not
extend the color grammar or the `background` shorthand.

The five list longhands' scalar and list owners expose `serialize_specified()` and
`serialize_specified_with_limits()`. Specified size serialization emits the
effective `auto` height after a non-auto width, whether the authored height is
absent or explicit; `auto auto` collapses to `auto`. Repeat serialization collapses
equal axes and uses `repeat-x`/`repeat-y` for the equivalent directional pairs.
Boxes and attachments emit canonical keywords. Every layer, numeric child and
separator shares one
cumulative input-node, projection-node and byte budget. Authored components
omitted by canonicalization still count toward node budgets. An effective `auto`
height generated for an absent authored height adds one projection node and five
output bytes (` auto`), without inventing an authored input node or changing the
authored height. An explicit `auto` height counts once toward both node budgets.
Failure returns no partial CSS and leaves the authored value unchanged. These
generic size/repeat serializers also serve their existing mask value owners.

These are the authored semantic values, including symbolic size calculations;
global keywords and substitution-dependent declarations remain separate branches.

`CssImage::try_new` checks the image-only grammar and rejects the property
keyword `CssImageValue::None`; its borrowed `value()` retains a URL or gradient.
Programmatic gradients use `CssGradientColorStop::from_color`, checked
`CssColorStopList::try_new`, and `CssLinearGradient::new` or
`CssRadialGradient::try_new`. The radial constructor rejects incompatible
explicit radius forms, permits omitted size and extents, and preserves authored
omissions. `CssPhysicalPosition::try_new` checks the physical generic
position grammar: explicit edge offsets occur on both axes or neither. These
constructors retain authored symbolic values without resolving colors,
percentages, URLs, or positions.

The selected [Images 3 specified serialization](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/#serialization)
is available on `CssImage`, `CssImageValue`, `CssGradient`, and
`CssImageValueList` through `serialize_specified()` and a limits variant.
They write lowercase gradient function names, checked colors, symbolic numeric
values, and URL functions without loading resources or resolving layout.
Only context-independent defaults are omitted: a linear direction of `to bottom`
or a directly authored exact half-turn, an initial direct numeric zero stop,
a final direct `100%` stop, and the corresponding radial default shape, size,
and ordinary center position. Calculations and four-component positions remain
explicit. Nested images share one input-node, projection-node, and output-byte
budget; an exhausted budget returns an error without a partial CSS result.

The selected Backgrounds 3 `background-image` and Masking 1 `mask-image`
longhands each contribute one ordered `CssImageValueList`. Both are
noninherited and initially contain one `CssImageValue::None`. Their parsed
wrappers expose that list through `images()`. Intrinsic expansion and
normalization preserve source occurrence, importance, and pending replacement
components; they do not load images or match layers with sibling properties.

The mask shorthand uses the same shared image/`none` grammar, including selected
gradients and LightDark images, as `mask-image`, following Masking 1 §7.1/§7.9.
`CssMaskLayer::try_new` accepts those images and rejects wholly empty layers.
Its existing position, size and repeat contracts remain; other mask components
remain unfinished.

```rust
use surgeist_css::{
    CssGradient, CssImageValue, CssKnownPropertyValueRef, CssSupportStatus,
    feature_metadata, parse_style_attribute,
};

let report = parse_style_attribute(concat!(
    "background-image: linear-gradient(to right, red 0%, 40%, blue); ",
    "border-image: url(frame.png) 10 fill / 2 / 1 round",
));
assert!(report.is_clean(), "{:?}", report.diagnostics());

let CssKnownPropertyValueRef::BackgroundImage(images) = report.syntax()[0]
    .known().expect("known background image")
    .property_value().expect("ordinary background image")
else { panic!("expected background-image") };
assert!(matches!(
    images.images().images(),
    [CssImageValue::Gradient(CssGradient::Linear(_))]
));

let CssKnownPropertyValueRef::BorderImage(border) = report.syntax()[1]
    .known().expect("known border image")
    .property_value().expect("ordinary border image")
else { panic!("expected border-image") };
assert!(border.border_image().slice().expect("slice").fill());

let gradient = feature_metadata("official.value.linear-gradient")
    .expect("linear-gradient metadata");
assert_eq!(gradient.source().id().as_str(), "O-IMAGES3");
assert_eq!(gradient.status(), CssSupportStatus::Complete);
```

This family added 27 public `Complete` atomic records: nine properties and
eighteen shared values. Its Backgrounds 3 property grammars previously carried
Partial metadata and are now catalogued as Complete. The already-Complete
`background-position`, `object-position`, and `box-shadow` rows remain Complete.
That earlier catalog expansion brought the public support catalog to 456
records; the current catalog inventory is described below. Activation and
promotion do not add official ledger units: the inventory remains 162 property
units (161 canonical properties plus the custom-property family), one normative
legacy shorthand, and 167 non-property units.

These models do not fetch or decode images, resolve URLs, apply cascade or
substitution, compute background or border geometry, paint, serialize CSSOM, or
lower values into another Surgeist crate.

## Flexbox, multicolumn, and catalog coverage

The nine selected Box Alignment 3 properties have checked authored values and
intrinsic expansion. Six noninherited longhands keep their distinct domains and
initials: `align-content`, `justify-content`, and `align-items` start at
`normal`; `justify-items` starts at `legacy`; and `align-self` and
`justify-self` start at `auto`. The three `place-*` shorthands set ordered
align/justify pairs with no reset-only members. An omitted justify component
copies align except that a baseline in `place-content` defaults justify to
`start`. Each longhand has a private-field checked value around the shared
authored alignment vocabulary, so an item-only position cannot enter a content
property by typed construction. The place values hold two independently
checked property values. Canonical specified serialization emits both effective
components with one resource budget; explicit `first baseline` serializes as
`baseline` while the authored wrapper retains its original spelling and checked
alignment value. For the ambiguous shorthand
`baseline last baseline`, parsing prefers bare `baseline` for align and
`last baseline` for justify; a one-component shorthand `baseline last` still
denotes last-baseline align with an omitted justify component. The grammars and
omission rules follow the selected
[Box Alignment 3 §§4–7](https://www.w3.org/TR/2026/WD-css-align-3-20260130/).
The ambiguous split follows the pinned
[WebKit baseline consumer](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSPropertyParserConsumer%2BAlign.cpp),
while the parser still accepts the published reverse baseline order.
The four historically Flexbox-sourced alignment feature IDs retain their IDs
while their completed authored grammar metadata now cites the selected Box
Alignment 3 source. This CSS layer does not claim layout-mode support for every
alignment keyword.

Flexbox 1 `flex-flow`, `flex-basis`, `flex-grow`, `flex-shrink`, and `flex`, and all
nine Multicolumn 1 properties expose typed authored values. `flex-direction`
and `flex-wrap` are noninherited terminals with `row` and `nowrap` initials.
`flex-flow` expands to those two terminals in direction-then-wrap order, with
an omitted component taking its initial value and no reset-only members.
`CssFlexFlow::new()` constructs the pair directly; all three values serialize
specified keywords with bounded output, and `flex-flow` serializes both
effective components in that order. The direction and wrap wrappers retain
their checked keyword values. The twelve deprecated `-webkit-` property spellings in
[Flexbox 1 normative Appendix B](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/#webkit-aliases)
are name-equivalent aliases of their seven flex properties, four alignment
properties, and `order`. They use each target's full authored grammar and
expansion. The four alignment targets retain their selected Alignment 3 grammar
source, and `order` retains Display 3; the Flexbox appendix supplies the alias
obligation, not a replacement canonical definition. Browser UAs must support
these compatibility names; other UAs may. Authors should use them only where
legacy compatibility requires them.
`columns` retains
the effective width and count, with omitted components set to `auto`, while its
wrapper preserves the original spelling and order. `column-rule` retains its
explicitly authored width, style, and color in the shared exact border domains.
Its intrinsic expansion contributes all three rule longhands in width, style,
color order, resetting omitted members to `medium`, `none`, and `currentcolor`.
The shorthand's specified serializer omits initial components and emits
`medium` when all components are initial; it shares one resource budget across
the complete rule. `column-fill` and `column-span` have noninherited `balance`
and `none` initials. This authored model does not perform column balancing,
spanner layout, rule painting, or style-dependent width computation. The
breaking public rule migration makes `CssLineWidth` an alias of
`CssBorderWidth`, so its `Length` variant contains
`CssSpecifiedNonNegativeLength`. `CssColumnRule::color()` now returns the exact
`CssColor`, and the `column-rule-color` wrapper exposes the same value through `value()`.

`flex-basis` accepts the width sizing grammar plus `content`, including
`calc-size(content, …)`; the generic `CssCalcSize` graph retains that flex-only
basis. Box properties use `CssBoxCalcSize`, whose checked conversion rejects
`content` even when nested in another `calc-size()`. `CssFlexBasisValue` keeps
`content`, an ordinary size, or a root `calc-size()` in one canonical form, and
`CssFlexBasisRef` exposes a borrowed view. `CssFlexComponents::try_new()` stores
authored optional grow, shrink, and basis components; its accessors return
`Option`; the constructor rejects an empty or shrink-only value. Grow and shrink use exact
nonnegative specified numbers. The shorthand expands to grow, shrink, and basis
only: omitted factors are `1`, an omitted basis is specified unitless `0`,
`none` expands to `0 0 auto`, and `auto` to `1 1 auto`. Its initial longhand
values are `0`, `1`, and `auto`, respectively. Serialization of a component
shorthand emits its full effective triple, so authored omission is not inferred
from that output. The wrapper directly retains the checked shorthand components.
These contracts follow
[Flexbox 1 §7](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/#flex-property),
[Flexbox 1 §7.2.3](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/#flex-basis-property),
and the selected Sizing 3/4 and Values 5 sizing productions.

Multicolumn 1 defines `column-width` as `auto | <length [0,∞]>`, `column-count`
as `auto | <integer [1,∞]>`, and the unordered `columns` shorthand. The selected
[Sizing 4 §5.6](https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/#column-sizing)
adds `<box-size>` to `column-width`; `columns` consumes that expanded width
grammar. `CssColumnWidth` is now an alias of `CssSizeValue`, so callers migrate
`CssColumnWidth::Length` to `CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(...))`.
The `width()` and `count()` accessors remain; `CssColumns::serialize_specified()`
emits both effective values in width-then-count order. Ordinary positive
column counts use `CssPositiveIntegerValue::Literal`, preserving
their digits without clamping. Integer calculations and box-size math stay
symbolic until their computed-value owners resolve them. Shorthand expansion
sets only `column-width` and `column-count`, with no reset-only members.

```rust
use surgeist_css::{
    CssColumnCount, CssFlexDirection, CssKnownPropertyValueRef, CssSupportStatus,
    feature_metadata, parse_style_attribute,
};

let report = parse_style_attribute("flex-flow: column wrap; columns: 3 12em");
assert!(report.is_clean(), "{:?}", report.diagnostics());

let CssKnownPropertyValueRef::FlexFlow(flow) = report.syntax()[0]
    .known().expect("known flex-flow")
    .property_value().expect("ordinary flex-flow")
else { panic!("expected flex-flow") };
assert_eq!(flow.flow().direction(), CssFlexDirection::Column);

let CssKnownPropertyValueRef::Columns(columns) = report.syntax()[1]
    .known().expect("known columns")
    .property_value().expect("ordinary columns")
else { panic!("expected columns") };
assert!(matches!(columns.columns().count(), CssColumnCount::Count(_)));

let metadata = feature_metadata("official.property.flex-flow")
    .expect("public Flexbox metadata");
assert_eq!(metadata.source().id().as_str(), "O-FLEXBOX1");
assert_eq!(metadata.status(), CssSupportStatus::Complete);

let shared = feature_metadata("official.value.syntax-token-stream")
    .expect("public Syntax 3 value metadata");
assert_eq!(shared.status(), CssSupportStatus::Complete);

let extension = feature_metadata("ext.value.relative-color")
    .expect("preserved extension metadata");
assert_eq!(extension.status(), CssSupportStatus::Partial);
assert!(extension.supported_subset().is_some());
assert!(extension.unsupported_remainder().is_some());

let feature_values = feature_metadata("later.rule.font-feature-values")
    .expect("authored font-feature-values metadata");
assert_eq!(feature_values.status(), CssSupportStatus::Complete);
assert_eq!(feature_values.source().id().as_str(), "I-FONTS4-20260907");
assert!(feature_values.unsupported_remainder().is_none());
```

The generic Syntax 3 at-rule, qualified-rule, declaration, stylesheet,
rule-list, declaration-list, and style-block records are public `Complete`
atomic metadata. These fourteen formerly `Reserved` shared
values are also public `Complete` metadata: `syntax-token-stream`, `component-value`,
`simple-block`, `function`, `declaration-value`, `any-value`, `an-plus-b`,
`unicode-range`, `css-wide-keyword`, `custom-ident`, `ident`, `string`, `url`,
and `url-modifier`.

Together, the ten Flexbox/Multicolumn property records, seven generic shell
records, and fourteen shared-value records are the 31 public catalog additions
that were previously `Reserved`. The seven Values 3
records for `dimension`, `angle`, `angle-percentage`, `time-percentage`,
`frequency`, `frequency-percentage`, and `calc()` were separately promoted from
`Partial` to `Complete`.

The preserved extension records `ext.value.relative-color`,
`ext.value.color-mix`, `ext.value.grid-repeat`,
and `ext.supports.selector` remain `Partial`,
with both subset and remainder metadata. The five `ext.media.range.*` records
for width, height, resolution, color and monochrome are now `Complete`, covering
signed symbolic operands and source-ordered chained comparisons.
The selected authored `@font-face` and `@font-feature-values` records are
`Complete`, including checked assembly and canonical effective specified rule
serialization. The selected section 6.9.2/frozen WebKit authored index policy and
contradictory section 6.9.1 wording remain explicit. Live CSSOM mutation and
font activation remain downstream. Generic group output is a separate boundary.

The public support catalog includes grammar-specific rule, descriptor and value
records, including four palette and four color-profile records. Its current
cardinality comes from
[`feature_catalog()`](../src/conformance.rs). That
catalog cardinality is distinct from the immutable official inventory of
exactly 162 property units (161 canonical properties plus the custom-property
family), one normative legacy shorthand, and 167 non-property units. All 219
preserved I01 baseline records retain their classifications, and the exclusion
registry now contains exactly 130 rows.

The crate owns authored syntax and canonical serialization. Cascade, substitution,
selector matching, query evaluation, resource loading, layout, pagination,
painting, and cross-crate lowering remain downstream concerns. Serialization
coverage is still incomplete; this ownership statement does not imply that every
rule has a canonical writer.

## Container properties

`container-type`, `container-name` and `container` use the selected
[Conditional Rules 5 property definitions](https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-type).
`CssContainerType` represents the six ordinary states, including either size axis
combined with scroll-state. `CssContainerNames` is `None` or a checked nonempty
`CssContainerNameList`; names preserve order, duplicates and case. `CssContainer`
pairs those names with a type. An omitted shorthand type means `Normal`.

The two longhands are non-inherited with intrinsic initials `none` and `normal`.
Shorthand expansion contributes container-name followed by container-type, with
no reset-only members. Generic declaration wrappers retain CSS-wide keywords and
pending substitution. Atomic pending reentry validates both shorthand members
without resolving variables. The same property identities are recognized in
plain style-query features, whose values keep their broader declaration-value
grammar.

`CssContainerName::try_from_decoded` accepts decoded names that need escapes;
`try_new` retains its existing exact unescaped identifier contract. Semantic
values expose `to_components_with_limits` and `serialize_with_limit`.
Canonical output escapes names, emits size-axis before scroll-state, and omits
`/ normal`. Limits include escapes and separators. Canonical components have
programmatic origins and can enter the existing `parse_property_value` checked
declaration boundary. Parsed wrapper `as_css()` and occurrence components retain
the original spelling, comments, order and source origins independently of this
semantic canonical output. See `examples/container_property_semantic_consumer.rs`.
Container selection, containment effects and style evaluation remain downstream.

## Conditional rules and import preludes

`CssContainerRule::prelude()` and `CssScopedContainerRule::prelude()` expose a
checked `CssContainerPrelude`. Its nonempty `entries()` retain each comma-separated
alternative in authored order, including duplicates. Each `CssContainerQueryEntry`
has a name, a query, or both; `name()` and `query()` expose those optional parts.
Name-only entries remain distinct from an unnamed query. Comma alternatives are
never lowered to Boolean `or`, since each entry selects its own container.

`CssContainerPrelude::try_from_components[_with_limits]` uses the same complete
prelude grammar as parsing. Invalid or empty entries reject the whole prelude.
Limits cover all entries, separators and nested components together. Prelude and
entry `components()`, `origin()`, `position()` and serializers retain the selected
lexical regions and their origins; the prelude includes commas and surrounding
trivia. Detached entry/query clones remain usable after their parent is dropped.
Normalized `CssRuleContextKindRef::Container { prelude }` retains the entire
prelude once, without duplicating the rule body for each alternative.

Container names preserve case-sensitive decoded identity and authored token
origins. `style` and `scroll-state` identifiers are valid names, while their
function tokens are query operands. CSS-wide keywords, `default`, `none`, `and`,
`or` and `not` are excluded as names in every ASCII case, including escaped
spellings. `CssContainerName::try_new` continues to accept one literal identifier;
use `CssComponentValue::try_ident` and checked prelude construction for decoded
names requiring escapes, such as names containing spaces or leading digits.

Container conditions retain explicit parentheses, homogeneous `and` or `or`
lists, `not`, recognized size, style and scroll-state features, and opaque
operands. `CssContainerCondition` has private fields; inspect its
`CssContainerConditionKind` through `kind()`. A `Parenthesized` node retains each
explicit group around a recognized condition, including redundant groups.
`GeneralEnclosed(CssContainerGeneralEnclosed)` retains complete unrecognized
functions and parenthesized operands. Recognized grammar takes precedence over
this fallback.

`try_from_components` and `try_from_components_with_limits` admit a complete
condition through the same immutable component grammar as parsing.
`try_from_enclosed` retains its checked single-enclosure entry point. Construct
component values with the checked component builders; constructing an inspectable
kind alone cannot manufacture a condition or bypass operand restrictions.
Trusted EOF recovery remains usable. Classification never serializes and
reparses input. Style values retain selected original components, including
programmatic token boundaries. Custom-property names come from decoded
identifier tokens, preserving escaped spaces, punctuation and case.

`components()`, `origin()` and `position()` expose the selected authored region.
Nodes share immutable lexical ownership, so a retained child remains valid after
its parent is dropped. `serialize()` and `serialize_with_limit()` produce the
existing deterministic component serialization, preserving grouping, operator
order, numeric spelling, symbolic operands and meaningful token boundaries.
They return the serialized origin map and typed resource errors. They do not
promise lossless formatting or CSSOM whitespace normalization.

`CssContainerStyleQuery` is a checked wrapper with `kind()` exposing
`CssContainerStyleQueryKind`. Its feature, explicit group, `not`, homogeneous
`and`/`or`, and general-enclosed nodes each retain their own component region.
Boolean and plain features carry `CssContainerStyleFeatureName`: ordinary
`CssPropertyGrammar` identity (including aliases) or a custom-property name.
Plain `CssContainerStyleValue` admits declaration-value syntax, without parsing
against the property's value grammar. CSS-wide keywords remain authored;
cascade-dependent keywords can therefore be retained while evaluation is false.

`CssContainerStyleRange::view()` exposes binary, ascending and descending forms,
with all two or three operands and exact comparison directions. Each range
operand's `view()` distinguishes a whole bare custom-property reference from
an authored value; `value()` exposes original components in either case. A bare
custom name in a plain feature value is literal syntax, not an implicit
reference. Values are not converted to numeric types: substitution, comparison,
computed-value matching, shorthand evaluation and query truth belong downstream.

The pinned explicit style-value production counts whitespace tokens and excludes
comment-only or empty values. Thus `style(--x: )` is plain while `style(--x:)`
remains general-enclosed; whitespace-only range operands are also retained.
Declaration-value admission excludes top-level semicolon/`!`, and comparison
delimiters are excluded recursively from operands, including variable fallbacks.
Strings containing those characters remain strings. The Variables1 optional
empty fallback in `var(--x,)` remains valid. This follows the edition's explicit
style-feature production rather than applying declaration parsing and its
whitespace trimming to the function contents. Query/value serialization preserves
lexical spelling and origins, including whitespace-only parsed operands.
See `examples/container_style_semantic_consumer.rs` for inspection without
reparsing serialized text. The former `CssContainerStyleQuery` enum's presence
and value variants migrate to `query.kind()` and `CssContainerStyleFeature`.

`CssContainerConditionKind::ScrollState` carries a checked
`CssContainerScrollQuery`. Its `kind()` exposes feature, explicit parentheses,
negation, homogeneous lists and general-enclosed inner operands. The four
`CssContainerScrollFeatureKind` identities are stuck, snapped, scrollable and
scrolled. Bare features carry that identity without an invented comparison.
Plain features use coupled variants: stuck has the none/physical-or-logical-edge
domain, snapped has none/x/y/block/inline/both, and scrollable/scrolled share
none, the physical/logical edges, and x/y/block/inline. Their distinct feature
variants preserve scrollability versus scroll-history meaning.

`CssContainerStuckValueRef`, `CssContainerSnappedValueRef` and
`CssContainerScrollDirectionValueRef` expose a keyword or
`Pending(CssContainerPendingValue)`. Pending domains are `Stuck`, `Snapped` and
`ScrollDirection`. The complete nontrivia-bounded value components are retained,
including compound values such as `top var(--empty)` and `var(--a) var(--b)`;
no variable or fallback is evaluated. The complete query region separately
retains its surrounding trivia. Names/operators cannot be substituted and the
four discrete features do not accept min/max or range-form syntax.

Whole-value deferral follows the selected Conditional5 container-query variable
extension together with Variables1 parsing rules; this is their combined
interpretation, not a separately stated scroll-specific production. A pending
colon value uses declaration-value syntax, so `stuck: top > var(--edge)` is
retained pending, while `stuck > var(--edge)` is an unsupported range form.
Style queries' special comparison-token exclusions do not apply to scroll
values. Invalid keyword domains, malformed variables and unknown direct
features remain outer general-enclosed. Grouping an unknown feature preserves
it as an inner opaque node. `scroll-state(var(--query))` likewise retains an
opaque inner function and does not substitute a whole query.

Scroll nodes share recursive lexical ownership. Typed/pending operands own only
their selected component snapshots, retaining parsed, programmatic and mixed
origins without copying enclosing or sibling query trees. Checked query limits
and trusted EOF recovery apply before recognition and opaque fallback, as for
other container queries. Snapshots, direction resolution, scroll history and
matching belong downstream. See `examples/container_scroll_semantic_consumer.rs`
for inspecting keyword and pending-domain payloads without reparsing text.

The six size features are typed: `CssContainerFeatureQuery::Boolean` carries
`CssContainerSizeFeatureKind`; width, height, inline-size, block-size and
aspect-ratio carry `CssMediaRange` views preserving plain/min/max, feature-first,
value-first and ascending/descending forms. The shared range shape contains no
media feature identity. Signed exact lengths and ratios use numeric calculations,
not tokenizer float projections. Ratios retain omitted denominators and admit
zero components without resolving whether a degenerate ratio is useful.
Orientation admits a keyword or a substitution-dependent operand, not a range.

`CssContainerLengthRef`, `CssContainerRatioRef` and `CssContainerOrientationRef`
distinguish typed values from `Pending(CssContainerPendingValue)`. Pending values
retain a complete component operand, its origins and required Length, Ratio or
Orientation domain. Checked `var()` syntax can have an empty or currently
wrong-domain fallback: final grammar is deferred until external substitution.
CSS does not choose a fallback, evaluate custom properties or substitute feature
names/operators. A function that remains opaque does not become implemented
merely because it is retained.

The narrow Values5 tree-counting import admits `sibling-count()` and
`sibling-index()` in the container's numeric context. Their explicit
`CssCalculationExpressionRef::TreeCounting` view retains function identity and
origin; they remain symbolic. Container length/number calculations may therefore
have context-dependent leaves that pure `CssLengthCalculation` or
`CssNumberCalculation` constructors and media queries do not admit. Reconstruct
such values through checked container construction. Relative units, query
container selection, substitution and tree-function evaluation belong to the
contextual owner. See `examples/container_size_semantic_consumer.rs` for typed
inspection without reparsing strings.

`not` prefixes one complete query operand. `not not (width > 1px)` and
`(width > 1px) and not (height > 2px)` are invalid; group a negated operand to
include it in a list. Trailing tokens and mixed outer boolean operators remain
invalid. Lexical bad tokens and component/depth failures survive grammar probes.
Invalid outer conditions drop their container rule while later siblings remain
eligible. Explicit construction limits apply to the complete condition;
serialization separately bounds its output bytes.

Migration: replace direct `CssContainerCondition` enum construction with checked
component admission, and match `condition.kind()` against
`CssContainerConditionKind`. Replace `CssContainerGeneralEnclosed::enclosed()`
with `component()` when inspecting its original lexical operand. Use the
condition serializer for the complete query, including recognized groups.
Size-feature payloads now use `CssMediaRange<CssContainerLength>` and
`CssMediaRange<CssContainerRatio>` views, including typed pending values;
orientation uses `CssContainerOrientation`. `CssRangeFeature`, `CssQueryLength`
and `CssRatio` no longer describe parsed container features. Their older scalar
contracts are not relaxed as part of this migration. Inspect exact numeric
expressions and retain their context instead of projecting into legacy floats.
The singular rule `name()` and `condition()` accessors have been replaced by
`prelude()`: iterate its entries and explicitly handle `entry.query() == None`.
Consumers that know they expect one entry should verify that cardinality before
projecting its name or query. Normalized Container matches now borrow `prelude`
instead of a singular name/condition pair. `InvalidPreludeGrammar` reports checked
prelude rejection separately from a checked Boolean condition failure.

These contracts use the selected
[Conditional Rules 5 edition](https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-rule),
including its general-enclosed production and the optional any-value grammar in
[Media Queries 4](https://www.w3.org/TR/2026/CRD-mediaqueries-4-20260219/#mq-syntax).
Size operands additionally use the selected
[MQ5 range grammar](https://www.w3.org/TR/2026/WD-mediaqueries-5-20260219/#mq-syntax),
[Values4 numeric grammar](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-type-checking)
and narrowly imported [Values5 §9](https://www.w3.org/TR/2024/WD-css-values-5-20241111/#tree-counting).
Style-query authored support is bounded by the implemented property grammar
registry. Remaining property inventory gaps keep overall container coverage
partial; opaque retention does not implement an unrecognized feature.
Whole-rule serialization still needs CSS support.
Container selection, condition evaluation and mutable CSSOM objects belong
to downstream owners.

Media support metadata cites the selected published MQ5 edition as the effective
source for query grammar and all 37 feature definitions. Historical feature IDs
and baseline alias memberships remain stable; they do not select superseded
semantics. The five existing range rows describe complete signed, symbolic and
source-ordered comparison grammar. Metadata does not claim query evaluation
or checked import construction.

Media features retain authored query syntax, including all 37 known boolean
feature names in the selected Media Queries 5 edition. Numeric feature operands
use media-owned exact values: signed lengths and integers, signed resolutions
or `infinite`, and number-valued ratios. Calculations use the shared numeric
grammar and remain symbolic. A syntactically valid empty range is retained;
parsing does not evaluate its bounds or match a device environment.

`CssMediaRange::view()` distinguishes plain, min/max, feature-first, value-first,
ascending and descending forms. Chained ranges preserve both operands in source
order and each comparison's inclusivity. Ratios retain an explicit denominator
or a programmatic default of one; zero denominators are valid authored syntax.
Ratio calculations carry a deferred nonnegative constraint. Grid literals admit
zero or one, including negative zero, while grid calculations retain integer
rounding and the deferred closed interval `[0,1]` constraint.

For migration, media numeric variants of `CssMediaFeatureQuery` now contain
`CssMediaRange<T>` instead of `CssRangeFeature<T>`. Match the borrowed range view
and inspect a length or integer operand's `calculation()` to access exact lexical
values and origins. Resolution and grid operands expose their own borrowed
views. `CssMediaRatio::numerator()` and `denominator()` return shared number
calculations; `denominator_is_omitted()` distinguishes an authored denominator
from its default. The former integer-pair `CssMediaRatio::try_new` constructor
and `Copy` implementation are removed; obtain the checked ratio through a parsed
or component-constructed media query and borrow or clone it. Container-query
range, length and ratio types retain their existing contracts.

`CssMediaConditionKind::UnknownFeature` replaces the former `DefinedFalse`
condition. It retains a validated generic feature expression and distinguishes
an unknown name, invalid value and forbidden operation. General-enclosed syntax
has its own variant. Both have unknown truth, including under negation; style
owns eventual evaluation. Unknown media types remain a separate nonmatching
type class. Neither class is a malformed-query sentinel.

`CssMediaQuery::Never` replaces a reserved or structurally malformed comma member
and is paired with `ReplaceMediaQueryWithNever`. The replacement is comma-local,
so later query members and the containing `@media` rule remain eligible.
Bad strings, bad URLs and unmatched delimiters preserve their typed component
errors and original coordinates.

```rust
use surgeist_css::{
    CssMediaConditionKind, CssMediaQuery, CssRecoveryAction, CssRule, parse_sheet,
};

let report = parse_sheet("@media (future-mode: active), ???, print {}");
let [CssRule::Media(media)] = report.syntax().rules() else {
    panic!("expected retained media rule");
};
assert!(matches!(
    media.query().queries(),
    [
        CssMediaQuery::Condition(condition),
        CssMediaQuery::Never(_),
        CssMediaQuery::Typed(_),
    ] if matches!(condition.kind(), CssMediaConditionKind::UnknownFeature(_))
));
assert_eq!(
    report.diagnostics()[0].action(),
    CssRecoveryAction::ReplaceMediaQueryWithNever,
);
```

`CssMediaQuery::try_from_components` and
`CssMediaCondition::try_from_components` validate checked Rust components through
the same grammar. Their `try_from_components_with_limits` variants accept
`CssComponentValueLimits` and retain distinct byte, component-count and nesting
failures. The byte budget also covers canonical expansion, such as an omitted
ratio denominator. Recovered components, including implicit EOF closures, are
rejected.
Media query, condition and typed-query positions are now optional: use `origin()`
for parsed or programmatic provenance and inspect `position()` only when present.
Cloned parsed components keep their original source coordinates inside
programmatically constructed parents.

Media query, condition and list `serialize()` methods return `CssSerializedValue`.
Canonical output normalizes recognized names and grammar separators, preserves
operand order and symbolic value spelling, and emits both ratio components.
Opaque enclosures preserve their meaningful token boundaries and comments.
Serialization fails with `RecoveredNever` if any list member is a recovery
sentinel; it produces no partial list. Clean authored `not all` remains ordinary
serializable syntax. These are authored-syntax contracts.

Custom-media definitions retain their name and explicit boolean or media-list
body, including an empty list. The name grammar permits the bare `--` identifier;
it differs from custom-property naming. Definitions remain in authored order,
including duplicate and cyclic references. Root-context conditional, layer and
scope groups retain definitions, while style bodies and groups nested under
style bodies reject them. A valid definition also closes the initial import
region. CSS preserves these rules and their enclosing contexts during
normalization; style owns environments, duplicate selection, cycle handling and
evaluation. The source is the selected [MQ5 custom-media
grammar](https://www.w3.org/TR/2026/WD-mediaqueries-5-20260219/#custom-mq) and the
narrowly imported extension-name definition in the [standards
catalog](../specs/catalog.json).

Boolean custom-media references have their own condition variant rather than
being unknown ordinary features. A selected custom name in a normal or range
feature is a syntax error; recovery replaces the entire comma member with
`Never`, including when the misuse is nested inside other conditions. A dashed
identifier used as an ordinary feature's value is still a value. For ambiguous
two-identifier comparisons, the parser retains its existing preference for a
known ordinary feature across complete grammar alternatives. Thus
`(--x < width)` retains an invalid-value `width` feature, while `(1 < --x)` is a
custom-media syntax error. This precedence is a repository interpretation of
the ambiguous grammar. An enclosure that matches no feature production, such
as `(--x:)`, remains general-enclosed.

`CssCustomMediaName::try_new` accepts a decoded identifier and escapes it as
needed. `CssCustomMediaRule::try_new` combines a checked name with a
`CssCustomMediaBody`; `try_from_components` instead admits the name/body prelude.
Both rule constructors have explicit-limit variants. A constructed prelude has
no authored at-keyword, so the containing rule has programmatic origin while its
supplied name and body tokens retain their origins.

Checked custom-media construction shares the grammar constraints and rejects
recovered input. Explicit media-list bodies containing only an unmodified,
unconditioned unknown `true` or `false` media type are rejected as ambiguous:
serializing them would select the distinct boolean body. Modifiers, conditions,
multiple members and empty lists avoid that ambiguity. Component-prelude
construction selects a boolean body for a lone `true` or `false` keyword.
Canonical serialization preserves token boundaries and original origins;
programmatic structure does not invent source coordinates. Retained malformed
media members cause an atomic serialization error.

`@supports` conditions expose declaration tests, `not`/`and`/`or` grouping,
complete Selectors 3 plus the selected existing selector extensions as the typed
`selector()` subset, and exact balanced general-enclosed fallback syntax. The
typed subset does not include `||`, unselected Selectors 4 pseudo-classes or
pseudo-elements, or syntax outside the named extension rows. Declaration tests
preserve authored property/value text and importance;
their optional known-declaration view is inspection data, not a declaration
inserted into a style block. Invalid children recover within a valid conditional
parent, while a malformed supports prelude drops that parent and leaves later
siblings eligible.

Declaration tests admit empty values, unknown properties and unsupported property
values. Top-level semicolons and invalid importance annotations reject the
declaration interpretation; nested punctuation remains value content. A terminal
`!important` is preserved even when the optional known-property view is absent.
Grammatically valid opaque contents can still select general-enclosed, so
`(color:red;)` remains an opaque condition. In an import, `supports(color:red;)`
can instead select the complete media alternative with no supports clause.
Bad string and URL tokens cannot enter either branch. These distinctions follow
the selected [Conditional Rules 3 grammar](https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/#at-supports);
they do not evaluate whether the renderer supports a declaration.

Conditional Rules 5 adds named conditions to the authored surface. A
`@supports-condition --name { ... }` rule retains a checked
`CssSupportsConditionName` and a `CssSupportsTestBody`; decoded names are
case-sensitive, start with two hyphens, and may be the bare `--` or an
all-hyphen identifier. The definition may appear where ordinary style rules
are admitted, including supported groups, scopes, and style-nested conditional
groups. At top level it can precede or appear between imports and namespaces
without closing their admission phase. It does not reopen a phase closed by
another rule. Invalid names, extra prelude input, a missing block, and statement
syntax reject the outer definition.

The test body is an ordered sequence of nonempty declaration runs, qualified
rule tests, and at-rule tests. `CssSupportsTestDeclaration` is a separate
structural candidate from the narrower `CssSupportsDeclaration` operand: it
retains unknown properties, values, functions, and unsupported feature grammar
without treating them as style declarations. It exposes the original property
component, semantic value components, terminal importance flag, full lexical
components, origin, and optional parsed position. Earlier `!` tokens remain in
the semantic value; only a terminal `!important` pair becomes the importance
flag. A qualified rule test retains its prelude and nested test body; an at-rule
test distinguishes a statement from an empty block. Declaration runs flush
around child rules, including a final run at the closing brace or EOF.

The body parser tries declaration structure before qualified-rule fallback and
keeps the generic top-level curly-block and custom-property-lookalike guards.
Missing-colon text with no block yields a local diagnostic and no candidate;
lexical errors such as bad strings, bad URLs, and unmatched closing tokens
recover the affected item while preserving structurally valid siblings and the
definition. Parseable implicit EOF closures remain retained with diagnostics.
`CssSupportsTestBody::recovery_origin()` identifies recovered input, distinct
from whether a feature is supported. Strict body and rule construction reject
recovered input and enforce aggregate component limits. The typed
`CssNamedSupportsConstructionError` separates invalid names, invalid body
grammar, recovered input, and component failures. This authored recovery policy
reconciles the selected generic Syntax block consumer with Conditional 5's
feature-test purpose; the specification does not specify each malformed-item
result.

An ordinary `@supports (--name)` operand retains
`CssSupportsConditionKind::Named` with its checked name, even when no definition
exists. `@import url(theme.css) supports(--name)` and its parenthesized
alternative also retain the named reference; import serialization preserves the
valid bare function-body spelling, while a standalone condition adds the
required parentheses. CSS does not activate definitions, choose a duplicate,
evaluate support, or produce live CSSOM. The selected source is
[Conditional Rules 5](https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#supports-condition-rule),
with the narrow [extension-name and block-contents imports](../specs/catalog.json).

Supports conditions and declarations can also be constructed from checked
component values. `CssSupportsCondition::try_from_components` takes an explicit
`CssNamespaceContext`; `CssSupportsDeclaration::try_from_components` takes the
bare declaration contents. Both have explicit-limit variants. They share the
authored grammar with stylesheet parsing and reject recovered component input.
Unknown properties, empty values, CSS-wide keywords and pending substitutions
retain their authored meaning; a known-property view never evaluates support.
`CssSupportsConstructionError` distinguishes grammar rejection, recovered input,
component resource limits and failure to start the bounded deep-construction
worker. Serialization also accepts an explicit output byte limit.

`components()` borrows the original lexical slice, including grouping and
importance tokens. Declaration `property_component()` and `value_components()`
expose the original property token and value without the terminal importance
annotation. Nested conditions share immutable lexical backing. Cloning a child
keeps its tokens alive independently of its parent; unrelated enclosing siblings
do not affect child equality.

Both models expose `origin()` and an optional `position()`. Programmatic wrappers
have no parsed position. A constructed declaration whose property token came
from a parsed source retains that token's position, but its aggregate `authored()`
is still `None`. Only declarations actually parsed from a source report a genuine
authored slice through `Some(...)`. Mixed-source tokens and known numeric views
retain their original snapshot identities.

Their serializers use the shared component canonical rules: preserve lexical
spelling, grouping and origins, and insert separators where tokens would otherwise
combine. They do not promise CSSOM whitespace or case normalization. Parsed
implicit closing delimiters can serialize with their recovery origins; checked
construction rejects those same recovered inputs. A standalone condition obtained
from an import's bare declaration gains programmatic parentheses when serialized
as a condition. The import rule keeps its original clause syntax.

Migration: `CssSupportsCondition::position()` and
`CssSupportsDeclaration::position()` now return `Option<CssSourcePosition>`;
`CssSupportsDeclaration::authored()` returns `Option<&str>`. Consumers must handle
programmatic construction rather than assuming a single source location or text
slice. Existing stylesheet parsing still supplies the original positions and
authored declaration text.

```rust
use surgeist_css::{CssRule, CssSupportsConditionKind, parse_sheet};

let report = parse_sheet(concat!(
    "@supports (display: grid) and (color: red) {}",
    "@supports selector(.card > .item:hover) {}",
    "@supports future-layout(mode) {}",
));
assert!(report.is_clean());
let [
    CssRule::Supports(declarations),
    CssRule::Supports(selector),
    CssRule::Supports(fallback),
] = report.syntax().rules()
else {
    panic!("expected supports rules");
};
assert!(matches!(
    declarations.condition().kind(),
    CssSupportsConditionKind::And(_)
));
assert!(matches!(
    selector.condition().kind(),
    CssSupportsConditionKind::Selector(_)
));
assert!(matches!(
    fallback.condition().kind(),
    CssSupportsConditionKind::GeneralEnclosed(value)
        if value.authored() == Some("future-layout(mode)")
));
```

`CssGeneralEnclosed` wraps exactly one checked function or parenthesis component.
Use `try_from_component`, `try_function`, or `try_parenthesized` to construct it;
these lexical constructors do not choose a conditional grammar branch. `origin()`
reports the opener's provenance and `position()` returns coordinates only for a
parsed opener. `authored()` returns the complete original enclosure, including
EOF-unclosed input, or `None` for programmatic enclosures. Parsed children under
programmatic delimiters keep their individual origins. Cloning and reconstructing
from `component()` preserves the same authored slice and equality.

`serialize()` preserves token spelling and maps emitted delimiters, including
EOF-implied closures, to their real origins. Structural equality includes exact
lexemes and source-text/span provenance; it does not compare runtime meaning or
require source snapshot identity. The optional `authored()` and `position()`
accessors replace the earlier parser-only, unconditional accessors.

An `@import` prelude is retained in exact target, optional `layer` or
`layer(name)`, optional `supports(...)`, optional media-list order. A successful
initial layer statement permits a following import. Once an import is followed
by another layer statement, a namespace phase, or a body rule, later imports are
invalid; only successful rules advance the phase.

Optional clauses are selected from complete grammar alternatives. For example,
`layer(theme) and (color)` is a media condition with no layer clause, while
`layer(theme) print` has a named layer and a media type. A later `layer()` or
`supports()` function can be an opaque media operand. When several complete
alternatives fit, this API prefers a present layer clause, then a present
supports clause. In stylesheet parsing, if none fits, media recovery runs once
after the first valid clause combination, retaining the import with invalid media
members represented by `Never`. Lexical and resource failures remain terminal,
with their original error categories and source positions. EOF-implied closures
remain diagnostics on the selected interpretation; speculative alternatives do
not add diagnostics.

`CssImportRule::serialize()` returns canonical `CssSerializedValue` output with
the same optional-clause interpretation. It preserves target spelling, checked
clause components, symbolic media and original token origins. An added EOF
semicolon has programmatic provenance. Any recovered `Never` member returns
`CssImportSerializationError::Media` without producing a partial rule. This does
not emit output if the shared grammar cannot preserve its interpretation:
`InterpretationChanged` reports that failure. `serialize_with_limit` additionally
bounds the complete canonical output, including inserted separators and the
terminating semicolon. This is an output limit, not a bound on temporary
allocations used to verify the clause interpretation.

`CssImportRule::try_from_components` constructs one complete import from checked
component values and an explicit `CssNamespaceContext`. Include the `@import`
at-keyword and an explicit semicolon; surrounding trivia is accepted. Additional
rules, nontrivia after the terminator, and malformed complete media lists are
rejected. Checked construction never creates recovered `Never` media members.
It uses the same optional-clause selection as parsing, so a malformed optional
clause can still be retained as an opaque media operand when that complete
alternative is valid.

The explicit-limits constructor validates all supplied components, including
trivia, before rejecting recovered input and checking the rule grammar. Limits
also apply to canonical output: a media value's canonical expansion can exceed
the byte budget even when its input fits. Construction failures distinguish
grammar rejection, recovery, component limits and errors from the supports,
media, serialization or deep-worker boundaries. Errors retain original origins
and nested error sources. No partially constructed rule is returned.

The rule's `origin()` belongs to its original at-keyword. `position()` now returns
`Option<CssSourcePosition>`; handle `None` for programmatic at-keywords. Parsed
at-keywords retain their original positions even when combined with programmatic
children. Target, supports and media components keep their original snapshots;
temporary grammar transport never becomes their public source provenance.
Canonical output omits surrounding trivia. Normalization preserves the optional
rule position and intact import payload without loading its target.

Import targets accept empty and whitespace-only decoded strings and URLs without
trimming their contents. This follows the authored grammar in
[Cascade 5](https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#at-import)
and the empty URL definition in
[Values 4](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#url-empty).
An empty URL resolves to an invalid resource; that downstream result does not
invalidate its authored syntax. Serialization here preserves authored target
spelling, rather than performing computed-value URL serialization.

Ordinary property values and font sources share this authored URL rule:
`url()`, `url("")`, `url('')`, and `src("")` retain an empty decoded string,
while quoted whitespace remains data. `CssUrl::new("")` constructs the
authored `Url` form; `CssUrl::from_parts` selects `CssUrlFunction::Url` or `Src`
and preserves ordered modifiers. `CssImportUrl::new(CssUrl)` preserves the shared
URL, while `CssImportString::new(decoded_text)` preserves the distinct string
target. Both constructors are infallible. `CssImportUrl::url()` and
`CssFontFaceUrlSource::url()` expose the shared authored URL. Resource validity
belongs downstream.
`CssUrl::is_local_url()` reports the Values 4 local URL flag for decoded targets
beginning with `#`; it does not resolve the fragment. `CssIdent::try_new()`
checks decoded identifier representability without consumer keyword exclusions.
`CssUrlModifierFunction::try_new()` checks the modifier's function syntax and
keeps its immutable argument components; `argument_components()` retains original
token origins from parsed or checked input. The `arguments().as_css()` view
preserves authored argument text, and URL equality continues to use
that text and the decoded modifier name rather than source coordinates.

`CssUrl::serialize_specified()` and its limits variant emit quoted `url()` or
`src()` text without resolving the target. Ordered modifiers retain their names
and checked token boundaries, including syntax closed implicitly at EOF.
Opaque modifier arguments retain meaningful whitespace and comments. One
cumulative input-node, projection-node, and UTF-8 byte budget covers the complete
value; failures return no partial output. CSS string escaping replaces embedded
NUL with U+FFFD in output while leaving the stored target unchanged. Fragment-only
targets remain fragment-only, and empty targets retain their function identity.

These models are authored syntax only. `surgeist-css` does not evaluate media or
supports conditions, match selectors, resolve URLs, load imported resources,
apply cascade or substitution, compute layer order, or lower syntax into root or
sibling types. Environment matching, resource loading, composition, and
cross-crate adapters remain downstream responsibilities.

## Authored box sizes

`width`, `height`, `inline-size`, `block-size`, and their `min-` and `max-`
counterparts retain twelve distinct longhand identities. The preferred and
minimum properties accept `auto` or a checked `CssBoxSize`; maximum properties
accept `none` or a checked `CssBoxSize`. All twelve are noninherited. Preferred
and minimum sizes start at `auto`; maximum sizes start at `none`.

`CssBoxSize` preserves nonnegative ordinary length-percentages, deferred math,
`stretch`, `contain`, intrinsic size keywords, `fit-content`,
`fit-content(<length-percentage>)`, and `calc-size()` through the checked
`CssBoxCalcSize` wrapper. Its checked ordinary
length-percentage constructor rejects even tiny negative literals without
rounding them through a floating-point representation. A calculation remains
symbolic until the owning layout layer can resolve it. The opaque
`CssMaxSizeValue` constructor rejects an `auto` basis anywhere inside nested
`calc-size()`; callers cannot construct an invalid maximum value with an enum
variant. These grammars follow Sizing 3 (2026-09-04) §§3.1–3.2, the Sizing 4
(2026-09-04) shared production and explicit definitions, and the required
Values 5 (2024-11-11) §10 `calc-size()` grammar.

The six physical wrappers expose `value()` as `CssSizeValue` or
`CssMaxSizeValue`.
Call `literal_component()` for an exact ordinary token or `calculation()` for
checked math within a `CssBoxSize::LengthPercentage`. The wrapper's `as_css()`
returns the original authored spelling. `serialize_specified()` produces the
canonical specified value and may simplify math without changing the retained
typed expression.

Typed sizing equality compares the checked branch and retained numeric structure,
including literal spelling, units, operators, order, and grouping. It ignores
diagnostic source snapshots and offsets; those origins remain inspectable on
the values. Intrinsic keywords and `calc-size()`/`fit-content()` identities use
their normalized checked kinds. Property wrappers still retain original
`as_css()` spelling, so differently authored declarations remain distinct.

## Authored positioning and insets

The selected [Values 4 generic `<position>` serialization](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#position-serialization)
is available on `CssPhysicalPosition::serialize_specified()` and its limits variant.
It writes the checked horizontal and vertical axes in that order: a parsed
`top` becomes `center top`, while `bottom 2% right 1px` becomes
`right 1px bottom 2%`. Paired edge offsets retain their edge keywords and
symbolic lengths, percentages, and calculations. The operation shares one
input, projection, and output budget across both axes; it does not resolve a
percentage against a box or use the separate background three-component form.
Full symbolic coordinate families are described in [Symbolic position families](#symbolic-position-families).

The selected [Position 3 draft](https://www.w3.org/TR/2025/WD-css-position-3-20251007/#position-property)
defines the five `position` keywords and the physical and flow-relative inset
longhands. All nine longhands are noninherited; `position` starts at `static`
and each inset starts at `auto`. Inset longhands accept `auto` or an exact
signed length-percentage, including deferred math. The `inset-block` and
`inset-inline` shorthands retain one or two authored values and contribute
start then end, repeating the start when the end is omitted. Writing-mode
mapping and used offsets belong to style and layout.

Position 3's inset definitions supersede Logical 1 §4.3; Logical 1 §4.7's
separate four-side `logical` switch remains an authored grammar extension.

The four-side `inset` shorthand accepts one to four values. Physical order
is top/right/bottom/left; a leading `logical` marker selects block-start,
inline-start, block-end and inline-end. Three values repeat the second on
both inline edges. Expansion follows the
[mode-selected membership policy](#four-side-shorthand-membership).

`CssInsetValue`, `CssInsetPair`, and `CssInsetShorthand` preserve exact checked
numbers, authored arity and mode. Their typed equality compares retained
structure without diagnostic origins; wrappers retain original `as_css()`
spelling. Physical `inset`, `top`, `right`, `bottom`, and `left` wrappers expose
`value()` with the exact checked semantic value.
`serialize_specified()` gives bounded canonical text, distinct from `as_css()`.

## Authored box spacing

The selected [Box 3 margin and padding definitions](https://www.w3.org/TR/2024/REC-css-box-3-20240411/#margin-properties)
and [Logical 1 draft](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#margin-properties)
give physical and flow-relative margin/padding longhands checked values. Margins
accept `auto`, signed exact length-percentage literals, and deferred math;
padding accepts nonnegative exact literals and deferred math, without `auto`.
All sixteen longhands are noninherited and start at zero. The four block/inline
axis pairs accept one or two values and expand to their two longhands, retaining
the authored component count and source occurrence.

The four-side `margin` and `padding` values preserve authored arity and
physical or logical mode. Their `assigned_values()` accessors apply quad
repetition; expansion and normalization use the
[mode-selected membership policy](#four-side-shorthand-membership).

The ten physical wrappers expose `value()` for `CssMarginValue`, `CssPaddingValue`,
`CssMarginShorthand`, or `CssPaddingShorthand`; inspect exact literals or
calculations through their checked length-percentage values. `as_css()` retains
the authored spelling, while `serialize_specified()` renders a bounded
canonical specified value.

## Authored sizing controls

[Sizing 4 §3.1](https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/#sizing-properties)
defines `size`, `min-size`, and `max-size` as one- or two-value physical
shorthands. They set width then height, minimum width then height, or maximum
width then height, copying the complete first value when the second is omitted.
They accept the same exact checked values as their member longhands, including
symbolic math. `CssSizePair` and `CssMaxSizePair` retain the optional authored
height; their expansion contributes only the two physical members in order.
The ordinary `size` property is rejected in `@page` context. The distinct
page-size descriptor and CSSOM's preferred shorthand ordering remain outside
this authored-property surface.

[Sizing 4 §5.3](https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/#responsive-iframes)
defines the noninherited `frame-sizing` keywords, starting at `auto`.
[§5.5](https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/#intrinsic-contribution-override)
defines noninherited `min-intrinsic-sizing`, starting at `legacy`; either
`zero-if-scroll` or `zero-if-extrinsic`, or both, can be authored. The typed
combined state serializes in scroll-then-extrinsic order. Internal layout size
and min-content contribution effects remain downstream. These finite values
and both pair types offer bounded `serialize_specified()` separately from the
property wrappers' original `as_css()` spelling.

## Authored contained intrinsic sizes

The selected [Sizing 4 §5.2 publication](https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/#intrinsic-size-override)
defines `contain-intrinsic-width`, `contain-intrinsic-height`,
`contain-intrinsic-inline-size`, and `contain-intrinsic-block-size`. Each accepts
an optional `auto` followed by `none` or a nonnegative length. They start at
`none`, do not inherit, and exclude percentages. The separate
`contain-intrinsic-size` shorthand accepts one or two whole values, setting
width then height and repeating the complete first value when the second is
omitted. Its expansion has no reset-only members; the logical longhands keep
their own identities.

`CssSpecifiedNonNegativeLength` retains exact ordinary tokens and symbolic
length math without narrowing through a floating-point value. The checked
`CssContainIntrinsicSizeValue` and `CssContainIntrinsicSize` models expose the
fallback, `auto` intent, and authored second component through `value()` on
their property wrappers. `as_css()` retains original spelling;
`serialize_specified()` gives bounded canonical output. Remembered sizes,
containment state, writing-mode mapping, and used box dimensions belong to
downstream style and layout.

## Authored float and clear

The selected [CSS2 float and clear definitions](https://www.w3.org/TR/2011/REC-CSS2-20110607/visuren.html#floats)
and [Logical 1 §2.2](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#float-clear)
define `float: none | left | right | inline-start | inline-end` and
`clear: none | left | right | both | inline-start | inline-end`. Both are
noninherited longhands with `none` initials. Their enum values serialize as
canonical keywords under specified-value limits, and each expands to one typed
contribution. The `float` and `clear` property wrappers expose `value()`;
`as_css()` retains the original spelling.
Mapping `inline-start` and `inline-end` to physical sides needs the containing
block's writing mode and belongs downstream.

## Authored overflow axes

The selected [Overflow 3 §3.1 publication](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/#overflow-control)
defines `overflow-x`, `overflow-y`, `overflow-block`, and `overflow-inline`
with `visible | hidden | clip | scroll | auto`, initial `visible`, and no
inheritance. Each remains an independent authored axis. The `overflow`
shorthand accepts one or two of these keywords, setting only x and y in that
order; an omitted y repeats x. Under the [Cascade 5 value-alias rule](https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#value-aliasing),
the legacy `overlay` keyword parses as `auto` while the property wrapper's
`as_css()` retains its spelling.

`CssOverflowValue` preserves the authored one- or two-keyword form and exposes
`x()`, `authored_y()`, and `y()` through the shorthand wrapper's `value()`.
The physical longhand wrappers also expose `value()`.
The enum and shorthand have bounded canonical specified serialization.
Cross-axis computed-value coupling and mapping flow-relative axes through the
element's writing mode belong downstream.

## Authored break controls

The selected [Fragmentation 3 break definitions](https://www.w3.org/TR/2018/CR-css-break-3-20181204/#break-between)
define `break-before`, `break-after`, and `break-inside` as noninherited
longhands with `auto` initials. Their `value()` accessors expose finite typed
keyword domains and bounded canonical serialization. The former canonical
`page-break-before`, `page-break-after`, and `page-break-inside` spellings are
now distinct [legacy shorthand grammars](https://www.w3.org/TR/2018/CR-css-break-3-20181204/#page-break-properties)
that set one modern longhand, have no reset members, and preserve the original
authored spelling and declaration occurrence. `always` maps to `page` for the
before/after aliases; the aliases reject modern-only values. [Logical 1 §3](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#page)
also admits `recto` and `verso` for those two aliases. Look up a legacy spelling
through `CssPropertyGrammar::from_name`; `CssKnownProperty::from_name` and
`property_support_metadata` identify canonical properties and name-equivalent
aliases, excluding distinct legacy grammars. The legacy feature IDs remain
`official.property.page-break-*`, now classified as `PropertyAlias`; retrieve
their support records with `feature_metadata(grammar.feature_id().as_str())`
and their intrinsic metadata with `grammar.metadata()`. Code using the
old `CssPageBreak*` types or `.page_break()` accessors should use the
`CssBreakBetween`/`CssBreakInside` values returned by modern wrappers'
`.value()` accessors. Fragmentation and flow-relative page mapping remain
downstream behavior.

The [Fragmentation 3 line-minimum definitions](https://www.w3.org/TR/2018/CR-css-break-3-20181204/#widows-orphans)
make `orphans` and `widows` inherited longhands with initial value `2`.
`CssPageLineMinimum` retains an exact positive integer token without a machine
integer magnitude limit, or deferred integer math. `literal()` returns an `i32`
only when the exact count fits; `exact_literal()` and `origin()` preserve the
authored token and its provenance. Checked construction rejects zero, negative,
and noninteger literals, including bare calculation roots, while math functions
remain symbolic. `serialize_specified_with_limits()` applies the shared bounded
specified-value projection. Line counting and fragmentation decisions belong to
layout and text.
Callers constructing a count from `CssIntegerCalculation` now use
`try_from_calculation()` and handle its typed error; the former infallible
`from_calculation()` could admit invalid bare roots. `try_literal()`, `literal()`,
and `calculation()` now inspect or construct exact token-backed values and are
no longer `const fn`.

## Authored overflow controls

The selected [Overflow 3 definitions](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/#overflow-clip-margin)
make `overflow-clip-margin`, `scroll-behavior`, `scrollbar-gutter`, and
`text-overflow` complete, noninherited authored longhands. Their initials are
`0px`, `auto`, `auto`, and `clip`, respectively. The clip margin imports only
[Box 4's `<visual-box>` production](https://www.w3.org/TR/2024/WD-css-box-4-20240804/#typedef-visual-box):
`content-box | padding-box | border-box`. It accepts that edge and a
nonnegative length in either order; either component may be omitted, but at
least one is required. Its checked value preserves omitted components, exposes
effective defaults of `padding-box` and exact `0px`, and serializes authored
components in canonical box-first order under one resource budget. Length math
remains symbolic until downstream resolution.

`scroll-behavior` accepts `auto | smooth`; `scrollbar-gutter` accepts `auto`,
`stable`, or `stable both-edges` in either authored keyword order. The existing
`text-overflow` choices remain `clip | ellipsis`. These keyword values have
bounded canonical serializers. Grammar and expansion preserve declaration
identity and source order; scroll execution, gutter geometry, and text painting
belong to their downstream owners.

## Authored scrollbar styling

See [authored color adjustment](#authored-color-adjustment) for symbolic scheme,
forced-color, and output-device hints that affect scrollbar appearance downstream.

The selected [Scrollbars 1 CR §§2–3](https://www.w3.org/TR/2021/CR-css-scrollbars-1-20211209/)
defines `scrollbar-width: auto | thin | none` and
`scrollbar-color: auto | <color>{2}`. Both are complete authored longhands with
initial `auto`; width is noninherited and color is inherited. Width applies to
scroll containers, has the specified keyword as its computed value, accepts no
percentages, and animates by computed value. Color also applies to scroll
containers and animates by computed value; it accepts an exact ordered
thumb/track pair from the shared checked `CssColor` grammar.

`CssScrollbarWidth` writes a lowercase keyword through
`serialize_specified[_with_limits]()`. `CssScrollbarColor::auto()` and
`CssScrollbarColor::new(thumb, track)` construct the two valid forms;
`thumb()` and `track()` return `None` for automatic coloring. Its specified
writer emits both colors in role order, including equal colors, and delegates
each child to the shared Standalone specified-color policy. `currentcolor`,
system colors, relative colors, and contextual functions retain their symbolic
graphs. One cumulative input-node, projection-node, and UTF-8 byte budget covers
the root and both children; failure returns no partial output and leaves the
authored value and source components unchanged.

The shared declaration lifecycle retains CSS-wide values, whole-value pending
substitution, strict replacement reentry, original occurrence and replacement
provenance, importance, and ordered normalization. Cascade, inheritance
application, forced-color overrides, computed color resolution, root-to-viewport
application, scrollbar dimensions, contrast choices, and platform painting
belong downstream. The dated Scrollbars CR supplies these authored contracts;
the frozen WebKit corpus and its adapters remain unchanged.

## Authored color adjustment

The selected [Color Adjustment 1 CR](https://www.w3.org/TR/2025/CR-css-color-adjust-1-20251216/)
defines three inherited longhands. `color-scheme` starts at `normal`,
`forced-color-adjust` at `auto`, and `print-color-adjust` at `economy`. The shared
declaration lifecycle supplies CSS-wide values, pending whole-value substitution,
strict reentry, intrinsic expansion, and ordered normalization with original
occurrence, replacement provenance, and importance.

`CssColorScheme::normal()` constructs the normal branch.
`CssColorScheme::try_new(entries, only)` checks a nonempty ordered list of
`CssColorSchemeKeyword::Light`, `Dark`, and checked `Custom` names, preserving
duplicates. `CssColorSchemeName::try_new(CssIdent)` retains decoded spelling and
case while excluding CSS-wide keywords, `default`, and the local `normal`,
`light`, `dark`, and `only` keywords. `none`, `auto`, and `span` remain valid
unknown names; they acquire no scheme semantics. Parsed `only` may precede or
follow the complete list, but cannot repeat or occur inside it. Canonical
specified output places it last and escapes custom names through the shared
CSSOM identifier writer. The media preference enum remains a separate domain.

`CssForcedColorAdjust` retains exactly `Auto`, `None`, or `PreserveParentColor`.
The last hint behaves like `none`, except that in forced colors mode, if `color`
inherits from its parent because of no cascaded value, `currentcolor`, `inherit`,
or another parent-inheriting keyword, downstream resolution computes it to the
parent's used color. That exception does not apply outside forced colors mode.
CSS ingestion does not execute palette forcing or parent-color selection.

`CssPrintColorAdjust` retains `Economy` or `Exact` as an output-device hint.
`color-adjust` remains a distinct, genuine deprecated CSS-standard shorthand
with exactly `print-color-adjust` as its settable member and no reset-only
members. Its initial and inheritance metadata refer to that longhand. Author
guidance favors `print-color-adjust`; valid shorthand declarations remain
accepted. Frozen WebKit at `73aa6c89e2cb77c46184a81aec944e4ab99d114d`
represents that spelling as an alias and lacks `forced-color-adjust`; these
implementation details do not narrow the selected specification. Its
[scheme consumer](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSPropertyParserConsumer%2BColorAdjust.cpp)
corroborates separate scheme entries and the `only` modifier.

Each type provides `serialize_specified[_with_limits]()`. Keyword values and
`normal` charge one input and projection node. A scheme list charges its root,
every entry, and its optional modifier under one cumulative byte budget. Failure
returns no partial CSS and does not mutate values or source components.
The properties accept no percentages. Scheme and forced-color hints apply to
all elements and text; print hints apply to all elements. Scheme computation
retains `normal` or the ordered specified list and animates discretely; forced
adjustment computes as specified and is not animatable; print adjustment retains
the specified keyword and animates discretely. Negotiation, UA palettes and UI,
viewport/SVG propagation, used-color adjustment, user print preferences,
output-device decisions, and animation execution belong downstream.

## Four-side shorthand membership

`border-color`, `border-style`, `border-width`, `inset`, `margin`, `padding`,
`scroll-margin` and `scroll-padding` expand to exactly four ordered longhands.
Physical mode selects top/right/bottom/left; `logical` mode selects block-start,
inline-start, block-end and inline-end. Neither mode resets complementary sides.
CSS-wide values select physical sides. Pending substitution occupies one symbolic
contribution until strict replacement reentry selects its own mode and retains
both the original occurrence and replacement origins.

This is a bounded compatibility decision. The selected
[Logical 1 draft](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#intro)
leaves complementary reset membership unsettled. Frozen WebKit
`73aa6c89e2cb77c46184a81aec944e4ab99d114d` corroborates physical four-member
membership and CSS-wide dispatch. Logical assignments follow
[Logical 1 §4.7](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#logical-shorthand-keyword);
logical no-reset is Surgeist policy, not a claim of settled normative behavior
or WebKit support for that marker.

`CssPropertyKindRef::FourSideShorthand` exposes
`CssFourSideShorthandMetadata::members(mode)` and `settable_members(mode)` using
`CssBoxSideKind`. Both select the same four ordered terminals;
`reset_only_members()` is empty. Fixed `Shorthand` metadata keeps its complete
unqualified getters. The existing property schema owns mode footprints and
projections; normalization charges the selected four members cumulatively.
CSSOM reverse composition and contextual writing-mode mapping are separate
operations.

## Authored border widths and physical border triples

The selected [Backgrounds 3 border-width definitions](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-width)
and [Logical 1 logical width definitions](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#propdef-border-block-start-width)
provide exact `thin | medium | thick | <length [0,∞]>` widths for four physical and four logical longhands, with noninherited `medium` initials. The logical block/inline pairs retain one or two authored values and contribute their respective two longhands. `CssBorderWidth` retains exact numeric spelling and symbolic math, and `CssBorder` retains each authored width, style, and color component of the five physical and six logical border triples; omitted components contribute their defined initials. `CssBorder::color()` borrows the authored color.

The four-side `border-width` grammar accepts one to four values and an optional
leading `logical` switch. `assigned_values()` exposes side assignments;
expansion follows the [mode-selected membership policy](#four-side-shorthand-membership).
Physical border triples retain their defined defaults and border-image resets.

The [Logical 1 side and axis border shorthands](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#border-shorthands) accept the same unordered width, style, and color triple. `border-block-start`, `border-block-end`, `border-inline-start`, and `border-inline-end` contribute only their three matching flow-relative longhands. `border-block` and `border-inline` contribute both sides of their axis in width, style, color order, with no border-image reset. Their wrappers expose `CssBorder` through `value()`; omitted width, style, and color contribute `medium`, `none`, and `currentcolor` respectively. No physical-side projection or writing-mode mapping occurs in this crate.

The affected physical wrappers expose `value()` for exact checked border triples. `as_css()` retains authored spelling, while bounded `serialize_specified()` emits the checked canonical value. Layout, writing-mode mapping, and cascade remain downstream.

## Authored border styles

The selected [Backgrounds 3 line-style definitions](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-style) and [Logical 1 flow-relative styles](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#border-style) provide ten style keywords for four physical and four logical longhands. Their noninherited initial is `none`. `border-block-style` and `border-inline-style` retain one or two authored values, assigning start then end and repeating the first when the second is omitted. The four-side `border-style` retains one to four authored values and an optional leading `logical` switch. Its assignments are top/right/bottom/left in physical mode and block-start/inline-start/block-end/inline-end in logical mode; it does not map logical sides to physical sides.

`CssBorderStylePropertyValue::value()` returns `CssBorderStyleShorthand`,
preserving authored arity and role mode. Inspect `assigned_values()` with `kind()`
for the authored roles. `CssBorderStyle` is the checked scalar keyword domain.
Each scalar, pair, and shorthand has bounded specified serialization under one
cumulative budget per value.

Four-side `border-style` expansion follows the
[mode-selected membership policy](#four-side-shorthand-membership). Physical
border triples retain their style defaults and border-image resets. Writing-mode
mapping, computed width zeroing and painting remain downstream.

## Authored corner radii

The selected [Backgrounds 3 radius definitions](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-radius)
and [Logical 1 corner definitions](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#border-radius-properties)
cover the four physical and four flow-relative corner longhands. Each accepts one or two exact nonnegative length-percentages, with a noninherited zero initial. The first value is the horizontal radius and the second is the vertical radius, even for a logical corner name; an omitted second value repeats the first. `CssCornerRadiusValue` preserves that omission, exact ordinary spelling, deferred math, and numeric origins.

The physical `border-radius` shorthand accepts one to four horizontal values and an optional slash followed by one to four vertical values. `CssBorderRadiusShorthand` retains the authored lists and supplies the four physical corners in top-left, top-right, bottom-right, bottom-left order. Without a slash, each contributed corner retains an omitted vertical radius; an explicit slash retains its authored vertical value. Intrinsic expansion has four physical longhand members and no reset-only members. Bounded specified serialization shares one resource budget across the aggregate and its authored children.

Physical property wrappers expose the exact checked `value()`. Basic-shape `inset()` round values use `CssBorderRadiusShorthand`, preserving horizontal and vertical arity and whether the vertical list was omitted. Flow-relative wrappers do not invent a physical corner mapping. Writing-mode and direction mapping, percentage basis, overlapping-radii reduction, geometry, and painting remain downstream.


## Authored Speech pause and rest

The pinned [Speech 1 pause definitions](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#pause-props)
and [rest definitions](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#rest-props)
provide `pause-before`, `pause-after`, `rest-before`, and `rest-after` with
`<time> | none | x-weak | weak | medium | strong | x-strong` grammar. All four
apply to all elements, have noninherited `none` initials, accept no percentages,
and retain the specified value as their computed-value contract. Canonical
order follows the grammar. Strengths remain symbolic; their absolute duration
is implementation-dependent.

`CssSpeechBreak` retains `None`, `Strength(CssSpeechBreakStrength)`, or
`Time(CssDuration)`. `try_time(CssTimeValue)` uses the existing duration owner
for exact rejection of negative ordinary times (including negative underflow)
and original recovered closure. Signed zero remains allowed. Valid authored
time calculations retain their math branch and defer computed range handling.
`none` remains distinct from an explicit zero time. The four coupled property
wrappers expose this model through `value()` and retain authored case, escapes,
comments, spelling and provenance through `as_css()` and declaration components.

`pause` and `rest` use `CssSpeechBreakPair`. Its checked `try_new(before, after)`
retains an optional second authored value and rejects recovered time children.
`before()` and `after()` borrow the effective assignments; `authored_after()`
reveals the optional authored second value. Intrinsic expansion emits exactly
before then after, assigning one value to both when the second is omitted. There
are no reset-only members. Whole-value CSS-wide keywords expand at the shorthand
boundary, and substitution reentry validates the original grammar atomically
while retaining source occurrence, importance and replacement origins.

Both models provide `serialize_specified()` and
`serialize_specified_with_limits(CssSpecifiedValueSerializationLimits)`. Times
use the existing canonical seconds provider and its six-place rounding policy.
Following [CSSOM value serialization](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-value),
pair output omits a demonstrably equivalent second component without discarding
its authored presence. Equality for this omission uses exact ordinary duration
comparison across seconds/milliseconds and equivalent decimal coefficients, or
the same retained calculation structure. Speech's `none` is also equivalent
to an exact ordinary zero duration in either position; their authored variants
remain distinct. A positive duration rounded to `0s` is not exact zero.
This equivalence affects canonical output only; model equality still distinguishes
`none` from a time and omission from an explicitly authored second component.
Equal rounded text alone does not establish equality. The pair and every authored child share one cumulative
input-node, projection-node and emitted-byte budget. A suppressed second child
still incurs traversal work and node charges. Failure returns no partial CSS
and leaves authored input unchanged.

Recovering parsing retains valid siblings and ordered diagnostics; clean-report
validation uses the same grammar. Source metadata identifies `S-SPEECH1` and the
individual `#propdef-*` production. Speech execution, strength duration
selection, pause collapse, additive rest behavior, synthesis and root-owned
style lowering remain downstream. Behavioral evidence is in
[`speech_pause_rest_lifecycle.rs`](../tests/speech_pause_rest_lifecycle.rs) and
[`speech_pause_rest_models.rs`](../tests/speech_pause_rest_models.rs).


## Qualified selector names

[Namespaces 3 §4](https://www.w3.org/TR/2014/REC-css-namespaces-3-20140320/#css-qnames)
separates an authored qualified name from the effective namespace constraint.
`CssQualifiedNamePrefix` retains `Unqualified`, `ExplicitNone` (`|`), `Any`
(`*|`), or `Named(CssNamespacePrefix)`. `CssQualifiedSelectorName` retains this
prefix alongside its decoded local identifier or universal `*` and its
`CssNamespaceConstraint`. `CssAttributeSelector::qualified_name()` exposes the
corresponding `CssQualifiedAttributeName`, whose local name cannot be universal.
Model equality retains prefix distinctions even when constraints coincide:
`*` differs from `*|*` without a default binding, and `[href]` differs from
`[|href]` in every context. Normalization preserves the models, ordered recovery
diagnostics, and enclosing rule source positions.

The checked qualified-name constructors take typed decoded local names and an
explicit `CssNamespaceContext`. Missing or case-mismatched named bindings return
`CssQualifiedNameError::UndeclaredNamespacePrefix`. An empty named namespace is
still a binding. Unqualified types and universals use the active default when
present; unqualified attributes always have no namespace. `CssIdent` and
`CssAttributeName` share the component identifier owner, preserving escaped
spaces, punctuation, digits and case without treating decoded strings as raw CSS.

The name models provide `serialize_with_limit(max_css_bytes)`, using the shared
component token writer to preserve authored prefixes and decoded identifiers.
The bound includes escaping and necessary token separators; failure returns no
partial output. These generated name tokens have programmatic origins. Parsed
enclosing rule positions remain separate. Complete CSSOM selector, style and
media rule serialization is tracked by
[#834](https://github.com/bj-data-eng/surgeist/issues/834).

`CssNamespaceRule::to_specified_css()` and `to_specified_css_with_limits()` use
[CSSOM's namespace rule format](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-rule):
`@namespace [prefix ]url("literal");`. The shared specified rule writer owns
identifier/string escaping and cumulative input-node, projection-node and
emitted-byte limits. A declaration charges one node for the rule, one for an
optional prefix and one for its literal name in each node budget. A supported
sheet also charges its enclosing aggregate and shares the budgets across every
rule. Failure returns no partial CSS. `CssRule` and `CssSheet` use this same
namespace provider, retaining declaration order and duplicates; unsupported
sibling rules still fail closed. Literal names remain unresolved, including empty
and non-URI names. CSSOM string serialization replaces programmatic NUL with
U+FFFD in output, while the stored authored value remains unchanged. Parsed
source positions and normalization diagnostics remain available without being
rewritten by canonical output.

Selector-qualified names permit comments but forbid whitespace within each name.
Whitespace can separate complete compounds: `svg |leaf` contains an unqualified
`svg` type followed by a descendant whose type has an explicit empty prefix.
The unsupported column combinator `||` remains rejected. Strict hosts reject
undeclared prefixes; forgiving selector hosts can discard the invalid member.
Ignored namespace declarations do not establish bindings.

The selected [2024 `attr()` definition](https://www.w3.org/TR/2024/WD-css-values-5-20241111/#attr-notation)
has its own optional named-prefix grammar, without empty or wildcard prefixes.
Its pending substitution retains original components. Namespace resolution
belongs downstream, as recorded in
[#717](https://github.com/bj-data-eng/surgeist/issues/717); the selector name model
does not widen that host grammar or change property admission.

Behavioral evidence is in
[`qualified_name_authored_prefixes.rs`](../tests/qualified_name_authored_prefixes.rs),
[`qualified_name_construction.rs`](../tests/qualified_name_construction.rs),
[`qualified_name_decoded_identifiers.rs`](../tests/qualified_name_decoded_identifiers.rs),
[`namespace_specified_serialization.rs`](../tests/namespace_specified_serialization.rs), and
[`namespace_rule_construction_serialization.rs`](../tests/namespace_rule_construction_serialization.rs).


## Authored Speech voices

The pinned [Speech 1 §11.1](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#voice-family)
`voice-family` model retains a nonempty prioritized list or the whole-value
`preserve` alternative. `CssVoiceFamilyName` preserves quoted strings separately
from sequences of decoded `CssIdent` values, including empty quoted names and
escaped identifier characters. Visual generic names such as `serif` are ordinary
voice names. Age words `child`, `young` and `old` can also be names when they do
not introduce the generic voice production.

The borrowed family-name/custom-ident grammar excludes CSS-wide keywords and
`default` per identifier token. Speech's gender keywords and `preserve` also
require quoting within a name. Thus `"Mike male"` is a name, while `Mike male`
is invalid. A generic voice retains optional age, required gender and optional
variant in grammar order. `CssGenericVoice` uses the shared positive integer
model: an ordinary index must be strictly positive with integer lexical syntax
and has no machine magnitude bound. Integer math retains its authored calculation
phase; this crate does not select voices or evaluate a contextual variant.
Omitting a variant stays distinct from explicitly authoring `1`.

[Speech 1 §11.5](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#voice-stress)
`voice-stress` retains `normal`, `strong`, `moderate`, `none` or `reduced` through
`CssVoiceStress`. Both properties inherit, apply to all elements and accept no
percentage values. `voice-stress` has the intrinsic initial `normal`;
`voice-family` has `CssUserAgentInitial::VoiceFamily`, which records the required
user-agent environment without inventing an installed voice.

The models' specified serializers use one cumulative resource context, shared
identifier/string escaping and shared exact integer/math projection. Output
uses grammar order without converting unquoted names to computed strings or
resolving language preservation, installed-voice fallback or acoustic effects.
Failure returns no partial output. Checked generic construction rejects original
recovered calculation closures; browser recovery can retain these with diagnostics.
Whole-value CSS-wide keywords, substitution reentry, intrinsic terminal expansion
and property support metadata use the common property lifecycle.

Behavioral evidence is in
[`speech_voice_family_stress_lifecycle.rs`](../tests/speech_voice_family_stress_lifecycle.rs)
and [`speech_voice_family_stress_models.rs`](../tests/speech_voice_family_stress_models.rs).

## Authored Speech cues

The selected [Speech 1 §10.1](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#cue-props-cue-before-after)
defines `cue-before` and `cue-after` as `none` or an auditory resource followed
by an optional signed ordinary `<decibel>`. Both longhands apply to all elements,
are noninherited, have `none` initials and no percentage values, and retain
specified values. The resource uses the existing `CssUrl` owner, including
`url()`/`src()` identity, decoded targets and ordered symbolic modifiers; no
resource is fetched or interpreted.

`CssDecibelLiteral` is the reusable exact ordinary dB terminal. `try_new(number)`
and `try_from_component` require one dimension with the ASCII case insensitive
decoded `dB` unit. Its `numeric`, `component` and `origin` accessors retain the
original coefficient spelling and provenance, including signed zero and finite
decimal values beyond binary64 range. Negative values are valid. Calculations
are not admitted as decibel values: this leaf does not invent a dB math algebra.
Specified serialization formats the retained coefficient with the shared specified
precision and lowercase `db`, following Values 4's unit serialization rule.
Authored coefficient spelling and unit case stay intact. A tiny nonzero offset
may serialize as `0db` under that precision, but it is still emitted rather than
omitted as an exactly authored zero.

`CssCue` separates `None` from `Audio(CssAudioCue)`. The audio model couples a
shared URL with an optional `CssDecibelLiteral`; its `url` and `decibel` accessors
preserve the authored choice. An omitted offset stays absent, even though Speech
defines its implied computed value as 0dB. `CssAudioCue::try_new` rejects implicit
closure retained in URL modifier arguments using the shared component origin
and error representation. Original URL root closure remains in declaration
components; checked property construction rejects it, while browser recovery
can retain the cue and its shared EOF diagnostics.

[Speech 1 §10.3](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#cue-props-cue)
defines the ordered one/two-value `cue` shorthand. `CssCuePair` retains `before`
and optional `authored_after`; `after` borrows the first cue when omitted.
Its checked constructor rejects retained modifier-argument closure in either
child. Intrinsic expansion supplies `cue-before` and `cue-after` in that order
and retains source occurrence, importance and replacement component origins.
CSS-wide keywords and pending whole-value substitution use the common lifecycle.

Canonical specified serialization uses shared URL/string/modifier serialization,
then the shared coefficient formatter. Under CSSOM optional-component omission, an explicit
zero offset is omitted from audio output, and an equivalent optional after cue
is omitted from shorthand output. Neither omission removes authored fields.
Cue equivalence requires equal shared URL identity and symbolic modifiers and
exact decimal offset equality, with absent offsets equivalent to explicit zero.
No floating-point rounding participates in omission.

All cue serializers share the existing cumulative input-node, projection-node
and emitted-byte limits. A pair charges one aggregate; each `none` charges one
leaf, each audio cue charges one aggregate plus its URL provider visits and any
authored dB terminal. An omitted effective after incurs no duplicate traversal;
an explicitly redundant after or zero offset still incurs authored traversal
work. Failure returns no partial output and leaves models unchanged.

Property metadata identifies `S-SPEECH1` and each `#propdef-*` production as
complete authored grammar support. Loading, fallback sounds, voice-volume
resolution and audio rendering remain downstream. Behavioral evidence is in
[`speech_cue_lifecycle.rs`](../tests/speech_cue_lifecycle.rs) and
[`speech_cue_models.rs`](../tests/speech_cue_models.rs).


## Authored Speech prosody

The pinned [Speech 1 §12.1](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#mixing-props-voice-duration)
`voice-duration` model distinguishes `auto` from `CssDuration`. Ordinary time
must be nonnegative; typed time calculations retain their authored range.
Duration does not inherit and has the intrinsic initial `auto`. A fixed
subtree duration can coexist with authored descendant duration/rate declarations;
applying precedence to a live speech subtree belongs downstream.

[Speech 1 §§11.3–11.4](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#voice-props-voice-pitch)
`voice-pitch` and `voice-range` share `CssVoicePitchRange`. Its checked absolute
branch requires a strictly positive ordinary `CssFrequencyValue`; an actual
Frequency-root calculation retains its range for later processing. Its relative
branch is a nonempty composition of an optional `CssVoiceLevel` and one optional
`CssVoiceOffset`. Frequency and percentage offsets are signed. `CssSemitoneLiteral`
retains the local ordinary `st` dimension, exact coefficient and origin, without
introducing a semitone calculation dimension or converting it to Hertz.
Both properties inherit and have the symbolic intrinsic initial `medium`.

The selected Speech text restricts ordinary absolute frequencies to positive
values and explicitly rejects `-20Hz absolute`, while its absolute-keyword prose
also describes a negative computed frequency becoming zero. Admission follows
the ordinary restriction and retains this wording tension. The selected
[Values 4 range phase](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-range)
defers range checks within calculations; computed clamping, installed voices and
synthesizer limits remain downstream.

Relative frequency and percentage occupy the same grammar position and represent
an inherited frequency quantity. [Values 4 §5.6](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#mixed-percentages)
permits their combination in math. `CssFrequencyPercentageCalculation` reuses the
shared hint/type algebra and projection, retaining the unresolved frequency basis
and reporting `CssCalculationType::FrequencyPercentage` when hinted. Absolute
frequency remains a pure Frequency root. The selected
[math type table](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-type-checking)
provides no semitone dimension, so `calc(1st)` is rejected.

[Speech 1 §11.2](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#voice-props-voice-rate)
`CssVoiceRate` retains an optional symbolic rate keyword and an optional
`CssSpecifiedNonNegativePercentage` in a nonempty checked composition. Ordinary
percentages must be nonnegative, including exact tiny coefficients; values above
100% are valid. Typed percentage math retains its specified phase. Rate inherits
and has the intrinsic initial `normal`. Omitted keyword is not replaced with
`normal`: a percentage can operate against an inherited rate.

Specified serialization emits frequency before `absolute`, relative level before
offset, and rate keyword before percentage. The
[CSSOM omission rule](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-value)
permits suppressing an exact ordinary neutral 100% rate modifier beside an explicit
keyword. Its authored field remains present and its input/projection visits still
count. Standalone 100% and retained calculations are not suppressed. A pitch/range
zero offset remains emitted because applying an offset fixes computed frequency
across subsequent voice changes, unlike a keyword alone.

All serializers share cumulative input, projection and byte limits with existing
time/frequency/percentage owners. Ordinary frequency output retains its selected
unit and rounds to at most six fractional places under the shared precision policy.
Consequently, a positive `1e-999Hz absolute` model retains its exact admitted input
but returns `CssSpecifiedValueSerializationErrorKind::UnrepresentableValue` rather
than emitting invalid ordinary `0hz absolute`. This is Surgeist's fail-closed
emission policy. The pinned sources leave the reconciliation between
[Values 4 supported precision](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types)
and [CSSOM number serialization](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-component-value)
and the [CSSOM grammar-representative value requirement](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-value)
unspecified; they do not mandate this error algorithm.
The frequency owner formats once under its existing precision and budget; Speech
checks its ordinary absolute output before completing the value. Resource limits
encountered first retain their resource errors. Relative rounded zero remains valid;
actual calculations keep their existing phase. A coefficient below `0.0000005`
rounds to zero, while that exact halfway value rounds to `0.000001`, in either Hz
or kHz. No epsilon frequency, invented calculation or computed voice clamping is
introduced.

Generic rule errors distinguish `Value(UnrepresentableValue)` and
`Value(UnserializableBoundary)` from the four actual node/byte/capacity `Resource`
errors. The original value error remains available as the error source, alongside
any enclosing stylesheet rule index.

Checked constructors reject original recovered numeric closures. Browser recovery
can retain such calculations with diagnostics. The shared property lifecycle owns
whole-value CSS-wide keywords, checked component admission, substitution reentry,
intrinsic terminal expansion, recovery provenance and pinned support metadata.
Behavioral evidence is in
[`speech_prosody_lifecycle.rs`](../tests/speech_prosody_lifecycle.rs) and
[`speech_prosody_models.rs`](../tests/speech_prosody_models.rs).

## Authored Speech mixing

The selected [Speech 1 §6.2](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#mixing-props-voice-balance)
defines `voice-balance` as a number or `left`, `center`, `right`, `leftwards` or
`rightwards`. `CssVoiceBalance` retains one keyword or the existing pure
`CssSpecifiedNumber` domain. Its `from_keyword`, `try_number`, `keyword` and
`number` methods preserve those branches separately. Ordinary coefficients have
no specified magnitude bound; typed Number calculations remain authored.
Percentages, percentage-hinted calculations and other numeric roots are not
balance numbers. Checked numeric model construction rejects original recovered
math closure, while browser recovery can retain diagnosed authored math.

Balance's specified serializer delegates shared keyword or pure Number output
precision and resource limits. It does not clamp to [-100,100], replace keywords
with numeric equivalents, or resolve inherited leftwards/rightwards adjustments.
Its initial is `center`; the property inherits, applies to all elements and has
no percentage interpretation. The property table's computed numeric range and
spatial mixing behavior belong to downstream style and execution owners.

[Speech 1 §6.1](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#mixing-props-voice-volume)
defines `voice-volume` as `silent`, or a nonempty composition of a level and an
ordinary signed dB offset. `CssVoiceVolume` encodes `Silent`, `Level` with a
`CssVoiceVolumeLevel` and optional `CssDecibelLiteral`, or `Offset` alone.
The five level choices are `x-soft`, `soft`, `medium`, `loud` and `x-loud`.
The variants prevent empty compositions and every companion to silent;
declaration grammar also rejects duplicate/conflicting levels and offsets.
Both authored level/offset orders are admitted, with level first in canonical
output. The accepted cue-owned dB terminal supplies exact coefficients and
origins; no dB calculation dimension is introduced.

Volume's initial is `medium`; it inherits, applies to all elements and admits
no percentages. Omitted level remains distinct from explicit medium: a bare
offset applies relatively to inherited volume or the root default. Authored
offset omission remains distinct from explicit zero. Under the selected
[CSSOM optional-component rule](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-value),
a level plus an exact ordinary zero offset serializes as the level alone, while
a standalone zero remains `0db`. A tiny nonzero offset may round to `0db` under
shared specified precision but remains emitted beside a level; omission uses
the exact authored coefficient rather than the rounded output.

Volume's serializers share cumulative input, projection and emitted-byte limits.
Silent and standalone offsets use their terminal providers. A level composition
charges one aggregate and one keyword, plus any authored offset terminal even
when exact zero is omitted. Failure returns no partial output or model mutation.
Both properties use the shared metadata, CSS-wide keywords, intrinsic terminal
expansion, substitution reentry and diagnostic provenance lifecycle and identify
their `S-SPEECH1` productions as complete authored grammar support.
Inherited offset addition, calibrated loudness levels, computed balance clamping
and sound-system behavior remain downstream. Evidence is in
[`speech_balance_volume_lifecycle.rs`](../tests/speech_balance_volume_lifecycle.rs)
and [`speech_balance_volume_models.rs`](../tests/speech_balance_volume_models.rs).

## CSSOM media-query text

[`parse_cssom_media_query`](../src/parser/fragments.rs) delegates to the existing
Media Queries 5 list parser. Exactly one member produces `Some`, including a
diagnosed ignored grammar member; empty and multiple-member lists produce `None`.
The original diagnostics, recovery actions and spans remain available on the
report. This nullable boundary is separate from `parse_media_query`, whose
authored fragment contract always retains a query or recovery sentinel.

`CssMediaQuery::serialize_cssom` and `CssMediaQueryList::serialize_cssom` use the
same owning emitter as authored serialization. They project malformed ignored
members to `not all` in their original list positions. The authored `serialize`
methods still reject recovered `Never` members. Neither operation mutates a
query, removes diagnostics, evaluates media conditions or resolves custom media.

Canonical output lowercases media-type identifiers, including unknown types,
and emits the defined orientation and scan keywords in lowercase. An unprefixed
`all` with a condition omits `all and`; lone `all`, `not all` and `only all` retain
their type. Lists preserve order and repetition with comma-space separators;
the empty list emits empty text. Explicit `only` remains the owning grammar's
legacy-hiding syntax. Modern range/condition syntax uses the existing MQ5
provider rather than a second CSSOM grammar.

The CSSOM projection also lowercases ordinary unknown feature names while
retaining their value components. Valid general-enclosed syntax and case-sensitive
custom-media references remain authored. A valid unknown condition is distinct
from malformed grammar recovery; its truth belongs to the downstream evaluator.
`CssMediaQuery::cssom_equals` compares canonical CSSOM bytes case-sensitively,
preserving syntactic ordering rather than claiming semantic query equivalence.
Derived authored model equality continues to include content and provenance.

The `*_with_limits` operations consume the shared specified-value resource
policy. Each query, condition and backing component charges one input and one
projection node; a list charges its aggregate too. A recovered query adds two
projected keyword nodes. Omitted default-type components still consume their
visited nodes. Comparison shares one budget across both operands, including
their combined serialized bytes. Errors retain the responsible source or
explicit programmatic origin, and no partial output is returned. The crate-owned
capture boundary accepts the same cumulative context used by enclosing rule
writers; final emitted bytes are charged once by that writer.

The [pinned CSSOM1 media clauses](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#media-queries)
define orientation and scan value spellings. Its other media-feature table cells
contain explicit ellipses. Selected Media Queries 5 and value-owner clauses
govern those features; this surface does not invent an ellipsis-cell format or
certify uncovered feature serialization. Numeric/value phase limitations remain
with their existing owners. Focused public evidence is in
[`cssom_media_queries_lifecycle.rs`](../tests/cssom_media_queries_lifecycle.rs)
and [`cssom_media_projection_contract.rs`](../tests/cssom_media_projection_contract.rs).


## Checked Fonts rule composition

`CssFontFaceRule::new(descriptors)` is total: empty and repeated valid descriptor
occurrences remain authored data. Constructed enclosing positions are `None`;
parsed descriptor and child origins remain unchanged. `CssSheet::try_from_rules`
checks ordered existing rules and derives namespace bindings from valid leading
declarations. `try_new` on ordinary and scoped media/supports/container/layer
blocks and scope rules uses the existing namespace context and typed preludes.
Scoped constructors accept ordered `Vec<CssScopedRule>` children and validate
that actual body role directly; a direct scope constructor additionally rejects
page definitions. The rule-list wrapper remains internally assembled, without
public collection mutation or a rule-list builder.

One iterative validator checks import/namespace phases, duplicate decoded
namespace prefixes/default declarations, style ancestry, explicit relative
selectors, nested declaration runs, global definitions, and direct-scope page
placement. Names retain their authored prefix/Any/Default/Named constraint
structure. The enclosing construction supplies namespace URI bindings; the
selector model does not store an original URI identity. Selector-bearing supports
conditions and scope boundaries share these checks. Raw decoded selector names
must be nonempty and contain no NUL, including functional arguments; escaped
names such as leading digits and spaces remain valid decoded identifiers.

`CssRuleConstructionError` exposes a typed reason, zero-based child path, and
real parsed position when available. The first failure follows authored traversal
order. The shared structural ceiling is 256 block-bearing rule levels, counting
new wrappers and existing descendants but not the stylesheet. Selector functional
depth is checked separately against the existing structural ceiling. Validation
uses checked iterative stacks before constructing a new enclosing tree.

`CssRule::to_specified_css` and the stylesheet front door dispatch both Fonts
rule kinds through one cumulative writer. Font-face output emits only present
effective descriptors and permits an empty body. Fonts 4 §4.1 selects the last
occurrence, and §13.2 shortens semantically equal range endpoints. Equality uses
proved retained literal or checked-expression identity, not rounded output.
Within matching checked expression structure, exact scalar magnitudes and unit
factors establish identity despite different numeric spellings. Differing
expression structures remain distinct unless an existing numeric owner supplies
an exact proof; equal-looking rounded unequal endpoints remain two values. The older CSSOM-listed descriptor kinds preserve
their relative order, with additional Fonts 4 kinds appended in descriptor-kind
declaration order as an explicit project ordering policy.

Feature-values output keeps all seven selected block kinds and effective outer
`font-display`. Repeated types merge in first-occurrence type order (project
policy); names retain only their last definition in last-occurrence order, as in
the selected Fonts 4 §13.2 example. Authored occurrences and provenance remain
unchanged; reparsing compares effective semantics rather than duplicate identity.
Live CSSOM maps, font activation, matching, and cross-rule cascade stay downstream.

`CssFontFaceDescriptorValue::serialize_specified` and `_with_limits` expose the
same descriptor dispatch for all fourteen concrete kinds, including display and
unicode ranges, without a synthetic rule. Existing authored pending values use
the shared pending-component owner in full rules. URL/source values stay in the
authored phase. All suppressed duplicate values and endpoints still consume
cumulative input/projection work; discarded output consumes no final bytes.
Failures are atomic. Generic groups continue to return `UnsupportedRule` until
their separate serialization owner supplies wrappers and complete dispatch.

## Optimization, anchoring, and fragmentation-decoration controls

[CSS Will Change 1 §2](https://www.w3.org/TR/2022/CRD-css-will-change-1-20220505/#will-change)
supplies `will-change: auto | <animateable-feature>#`. `CssWillChange` retains
`auto` or a checked nonempty ordered comma list of `scroll-position`, `contents`,
and property names. The dedicated `CssWillChangePropertyName` excludes the
CSS-wide keywords, `default`, `will-change`, `none`, `all`, `auto`,
`scroll-position`, and `contents` without importing grid's `span` exclusion.
Names retain decoded case, unknown future and custom property names, and repeats.
Canonical specified output escapes names through the shared CSSOM writer and
uses comma followed by one space. Serialization charges one input and projection
node for `auto`, or one list node plus one per hint; escaped bytes and separators
consume the same cumulative byte budget. Exhaustion returns an atomic typed
failure and leaves the authored value unchanged. Optimization execution belongs
to downstream style and rendering owners.

[Scroll Anchoring 1 §3](https://www.w3.org/TR/2020/WD-css-scroll-anchoring-1-20201111/#exclusion-api)
supplies `overflow-anchor: auto | none`, represented by `CssOverflowAnchor` in
the existing overflow-controls owner. It applies to all elements and has discrete
animation semantics. Anchor selection and scroll adjustment remain downstream.
[Fragmentation 3 §5.4](https://www.w3.org/TR/2018/CR-css-break-3-20181204/#break-decoration)
supplies `box-decoration-break: slice | clone`; its existing
`CssBoxDecorationBreak` now has bounded canonical specified output in the
fragmentation-controls owner. Each keyword charges one input node, one projection
node, and its canonical bytes; geometry and painting remain downstream.

These three longhands are non-inherited with specified initials `auto`, `auto`,
and `slice`, respectively. Parsed and checked component admission use the same
intrinsic grammars and preserve origins, importance, and occurrence identity.
Expansion emits one ordinary or CSS-wide contribution; whole-value substitution
remains pending until strict reentry. Normalization retains authored declaration
order. Their public support records are `Complete` for the authored lifecycle.
The catalog adds two feature records and two pinned specification sources;
`box-decoration-break` retains its baseline identity and Fragmentation 3 source.


## Authored selector emission and cumulative value composition

`CssSelector::to_specified_css()` and `to_specified_css_with_limits()` emit
checked selector grammar without resolving its symbolic ancestry. The provider
retains authored qualified prefixes, universal selectors, ID/class identity,
logical-list order, relative combinators in `:has()`, and ordered pseudo-element
attachment. Nesting and scope anchors remain `&`; explicit `:scope` remains a
pseudo-class. Identifier and string escaping use the same bounded CSS escaping
owner as the existing namespace writer. An unchecked empty or NUL-containing
scalar name returns `UnrepresentableValue` rather than changing its identity.

The selected [CSSOM selector algorithms](https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serializing-selectors)
supply identifier/string escaping, attribute-value strings, quoted `:lang()`
arguments, comma-space lists and combinator spacing. The selected
[Syntax 3 An+B algorithm](https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#serializing-anb)
emits `odd` as `2n+1`, `even` as `2n`, omits zero coefficients and emits signed
integer offsets without surrounding spaces. Authored qualified-name retention
continues the existing namespace provider policy. This selector operation is
an authored graph provider; it does not implement live CSSOM rule formatting,
selector matching or object mutation.

Every logical selector, compound, pseudo-class/pseudo-element, attribute,
qualified type name, anchor and explicit ID/class entry charges a semantic visit
to both cumulative node budgets. Simple scalar selectors charge one visit; enum carriers do not additionally
charge their compound or pseudo-class payload.
Language ranges and nth-pattern arguments also charge visits. Child work is
scheduled incrementally with checked allocation, including wide logical lists
and deep nesting; no independent serialization depth ceiling is introduced.
Failure returns no partial public text and leaves the graph available for retry.

Existing specified-value front doors now share their owning algorithm through
crate-private append operations on the cumulative specified writer. These
operations keep provider-specific canonical phases, duplicate/suppressed work,
precision and typed errors; they do not call a fresh public serializer and
concatenate its result. Font variant/synthesis and column-rule providers use a
local value start when inserting separators, so an enclosing prefix cannot add
spurious value whitespace. Counter changes continue to receive their property
identity because omitted integers differ between increment and reset/set.

This checkpoint supplies reusable providers for the shared rule writer tracked
in [#505](https://github.com/bj-data-eng/surgeist/issues/505), selector authored
subsets in [#834](https://github.com/bj-data-eng/surgeist/issues/834), and later
schema-coupled declaration composition in
[#831](https://github.com/bj-data-eng/surgeist/issues/831). Complete ordinary
property dispatch, missing semantic value families, all-rule traversal and
stylesheet encoding policy remain pending. Existing generic rule/sheet
front-door support is unchanged at this checkpoint.

Behavioral evidence is in
[`selector_specified_serialization.rs`](../tests/selector_specified_serialization.rs)
and the crate-private `specified_provider_composition_tests` module, alongside
the existing owning providers' public serialization suites.
