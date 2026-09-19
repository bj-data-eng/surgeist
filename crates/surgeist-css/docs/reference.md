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
`cssparser = 0.37.0` and `cssparser-color = 0.5.0`; test-only JSON support uses
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

`parse_media_query` replaces a malformed complete query with `CssMediaQuery::Never`.
`parse_media_query_list` recovers each root comma member independently, preserving
valid neighbors. A grammatically valid unknown feature remains an unknown
condition. An empty media query list is
valid and clean; an empty single query is malformed.
`CssMediaQueryList::try_new` also accepts an empty vector and preserves every
supplied member in order. Its existing `Option` return type is retained.

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

## Owned component values

`CssComponentValues` retains an immutable sequence of tokens, functions, blocks,
whitespace, and comments. Borrowed views expose decoded identifiers and strings,
hash flags, and exact numeric representations without rounding them to floating
point. Checked Rust constructors preserve the same token kinds. Property grammar
validation and variable substitution are separate operations.

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

The shared Values 4 grammar admits `calc`, `min`, `max`, `clamp`, `round`, `mod`,
`rem`, the trigonometric functions, `pow`, `sqrt`, `hypot`, `log`, `exp`, `abs`, and
`sign`, with their intrinsic arity and type rules. Binary addition and subtraction
require actual whitespace on both sides; comments alone are insufficient.
Standalone delimiter negation such as `-(1px)` is invalid. Substitution-dependent
declarations remain pending upstream; exact constructors reject residual `var()`.

Construction errors distinguish component, grammar, arity, type, domain, and
resource failures and retain responsible origins. Explicit byte, component, and
depth limits apply before admission. `serialize()` emits canonical authored
syntax with a map to original token and delimiter origins. Equality compares the
authored graph and provenance, not evaluated numeric equivalence.

Migration: parser-produced `CssCalcLength` values now use `Typed`, including
simple sums. Its payload is `CssLengthPercentageCalculation`. Percentage
construction moved from the pure `CssLengthCalculation` to that mixed wrapper.
Borrowed leaf views replace float or integer payloads; use exact representations
and unit identities instead of numeric `value()` accessors. Finite convenience
constructors remain available and delegate to checked programmatic components.
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

Breaking migration: `CssCalcLength::Sum`, `CssCalcLength::sum`,
`CssCalcLengthTerm`, and `CssCalcOperator` are removed. `CssCalcLength` retains
finite scalar leaves and `Typed`. Assemble compound programmatic calculations
with `CssLengthPercentageCalculation::try_sum` and
`CssCalculationSumOperator`, then wrap them in `CssCalcLength::Typed`. The first
operand has no binary operator; signed operand tokens remain valid. Checked
component construction owns shape and resource admission, so arbitrary recursive
legacy sums can no longer bypass those invariants.

The frozen I01 position, basic-shape and filter projection grammar now admits
literal lengths only. Calculation-bearing current values still parse through the
numeric owner but have no projection through that frozen grammar. Literal
projections, including `circle(50% at center)`, remain available. This does not
retire unrelated generic wrapper accessors that already expose typed calculations.
Grid and position compatibility checks exclude every `CssLength::Calc` variant.

## Authored flow tolerance

The selected [Grid3 publication](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#placement-tolerance)
defines `flow-tolerance: normal | <length-percentage> | infinite`, with no
nonnegative restriction. `CssFlowTolerancePropertyValue::value()` exposes its
checked `CssFlowTolerance`; `as_ref()` returns `Normal`, `Infinite`, or a borrowed
`LengthPercentage`. Construct the symbolic keywords with `normal()` and
`infinite()`, or use `try_length_percentage(CssLength)` for numeric values.
`Default` is symbolic `normal`. Its 1em used value in grid lanes and 0 used value
in other layout modes require downstream context and are not computed here.

The checked constructor accepts signed finite lengths, percentages, zero, and
supported calculations. It rejects unrelated `CssLength` keywords. Signed
operands, later subtraction and typed calculations remain valid and symbolic;
no computed range evaluation occurs at this boundary.

Migration: use `flow-tolerance` and `CssKnownProperty::FlowTolerance`. Both
`grid-flow-tolerance` and `item-tolerance` are unknown properties, without aliases.
The obsolete `CssGridFlowTolerance`/`CssGridFlowToleranceValue` types and their
I01 wrapper projection are removed. Historical source records and captured test
inputs retain their original identities. The effective property feature is
`ext.property.flow-tolerance`, sourced from the dated Grid3 publication.
Support remains `Partial`: the broader Values 4 math-function grammar is still
unfinished. This property migration does not complete the other Grid3 families.

## Intrinsic declaration expansion

`expand_declaration` currently covers custom declarations, physical margin and
padding, border width, style and color, the four side-border shorthands, `border`,
the five border-image longhands, `flow-tolerance`, `color`, `font-family`,
`text-orientation`, its legacy `glyph-orientation-vertical` grammar, `opacity`,
the `container` shorthand and its two longhands, and `all`.
The shared property schema owns their
member lists, initial values and reset-only components. Other known properties
return typed unsupported errors preserving their identity. The stylesheet
normalizer uses this same expansion boundary, so its complete property coverage
remains unfinished.

Opacity is a non-inherited longhand with numeric initial value `1`. Its ordinary
contribution retains the exact `CssOpacityValue`, including percentages,
calculations and out-of-range specified values. Computed clamping belongs to
style. Raw authored value text remains separate from canonical specified-value
serialization, which is not established by expansion.

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
never another pending result. It rejects residual `var()` at any nesting depth
before applying the original property grammar. Grammar and component failures
retain the same origin mapping as `parse_property_value`. Failure publishes no
partial contributions, and the pending handle remains available for another
attempt.

Every contribution retains the original declaration occurrence and importance
through `source()`. Contributions from reentry share one replacement component
tree, available through `replacement_components()`, including its original token
origins. These operations preserve symbolic lengths, colors and images; style
owns variable environments, invalid-at-computed-value handling and cascade.

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
for the selected 29 longhands, ten canonical shorthands, `all`, and the legacy
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
declarations. Imports are neither loaded nor merged, and conditions are not
evaluated. Both the ordinary and separate scoped authored rule trees are walked.

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
usage, or total process memory. Pending reentry is an independent immutable
expansion operation and does not change these counters or the normalized sheet.

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
inspection, exact authored text, and the frozen `i01_subset()` compatibility
projection on wrappers that retain it. `font-family` and `font` expose only their
current typed values. `CssImportance` and `CssSupportStatus` are closed public enums;
other public enums are non-exhaustive, so downstream matches require a wildcard.
The [compatibility explanation](explanation.md#symbolic-values-and-compatibility)
defines the I01 representation used by those projections.

## Finite numeric values, timing domains, and symbolic calculations

Current numeric models reject NaN and both infinities at checked construction
boundaries. Duration literals are additionally non-negative, while delay
literals are signed. Range checks that belong to the authored literal are
immediate; a well-typed calculation remains symbolic when its eventual range
belongs to computed-value processing.

```rust
use surgeist_css::{
    CssDelay, CssDuration, CssDurationLiteral, CssKnownPropertyValueRef,
    CssTimeUnit, parse_style_attribute,
};

assert!(CssDurationLiteral::try_new(-1.0, CssTimeUnit::Seconds).is_none());

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
assert!(matches!(
    duration.durations().values()[0],
    CssDuration::Calculation(_)
));
assert!(duration.i01_subset().is_none());

let CssKnownPropertyValueRef::TransitionDelay(delay) = report.syntax()[1]
    .known()
    .expect("known delay")
    .property_value()
    .expect("ordinary delay")
else {
    panic!("expected transition-delay");
};
assert!(matches!(
    delay.delays().values()[0],
    CssDelay::Literal(value) if value.value() == -250.0
));
```

The current accessors expose `CssDuration`, `CssDelay`, typed iteration values,
and typed calculation trees. `i01_subset()` remains the frozen compatibility
view: every I01 timing value retains its exact projection, while newly accepted
signed-delay or calculation syntax returns `None` when the older payload cannot
represent it. Calculation roots preserve authored units and expression shape;
this crate does not resolve relative units, evaluate computed ranges, run
animation timelines, or lower values into sibling Surgeist crates.

## Property-specific authored positions

Current position values preserve authored symbolic offsets and expose both axes
without resolving percentages, calculations, writing modes, positioning boxes,
object sizes, layout, painting, or transforms. `CssPositionOffset` accepts only
the position-valid length-percentage domain and retains whether an offset was
free or authored against a named edge.

The property grammars and accessors are deliberately distinct:

- `CssObjectPositionPropertyValue::position()` exposes one generic
  `CssPositionValue` through `CssObjectPosition::value()`.
- `CssMaskPositionPropertyValue::positions()` exposes a nonempty list whose
  `CssMaskPosition` layers each contain one generic `CssPositionValue`.
- `CssBackgroundPositionPropertyValue::positions()` exposes a distinct
  nonempty layer list that additionally admits the background-only
  three-component form.
- `CssTransformOriginPropertyValue::origin()` exposes explicit horizontal and
  vertical axes plus an optional `CssTransformOriginZ`; the z component is a
  checked authored length and cannot contain a percentage.

```rust
use surgeist_css::{
    CssHorizontalPosition, CssKnownPropertyValueRef, CssLength,
    parse_style_attribute,
};

let report = parse_style_attribute(concat!(
    "background-position: left 10px top; ",
    "object-position: right 5% bottom 2px; ",
    "transform-origin: top 50px",
));
assert!(report.is_clean());

let CssKnownPropertyValueRef::BackgroundPosition(background) = report.syntax()[0]
    .known().expect("known background position")
    .property_value().expect("ordinary background position")
else { panic!("expected background-position") };
assert!(matches!(
    background.positions().positions()[0].horizontal(),
    CssHorizontalPosition::LeftOffset(offset)
        if matches!(offset.value(), CssLength::Px(value) if value.value() == 10.0)
));

let CssKnownPropertyValueRef::ObjectPosition(object) = report.syntax()[1]
    .known().expect("known object position")
    .property_value().expect("ordinary object position")
else { panic!("expected object-position") };
assert!(matches!(
    object.position().value().horizontal(),
    CssHorizontalPosition::RightOffset(_)
));

let CssKnownPropertyValueRef::TransformOrigin(transform) = report.syntax()[2]
    .known().expect("known transform origin")
    .property_value().expect("ordinary transform origin")
else { panic!("expected transform-origin") };
assert!(matches!(
    transform.origin().z().map(|z| z.value()),
    Some(CssLength::Px(value)) if value.value() == 50.0
));
```

The background, mask, and transform wrappers keep `i01_subset()` as a frozen
compatibility view. Every I01 value retains its exact projection; newly accepted
current syntax returns `None` when the older payload cannot represent it without
loss. `object-position` is additive and has no I01 projection. Position use
inside gradients, transforms, filters, and basic shapes remains on its separate
function grammar boundary.

## Dedicated authored function grammars

Current property accessors expose dedicated typed function families while
`i01_subset()` remains the frozen compatibility view. `transform.current()`
returns `CssTransformValue`, timing-function wrappers expose current
`CssEasingValue` lists, `filter.current()` and `backdrop-filter.current()` return
`CssFilterValue`, `box-shadow.current()` returns `CssBoxShadow`, and
`clip-path.current()` returns an optional `CssClipPathValue`. A current value can
be valid when its I01 projection is `None`; consumers must not treat the
compatibility view as the current grammar.

```rust
use surgeist_css::{
    CssBasicShapeValue, CssClipPathValue, CssFilterFunctionValue, CssFilterValue,
    CssKnownPropertyValueRef, CssTransformFunctionValue, CssTransformValue,
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
    transform.current(),
    CssTransformValue::Functions(functions)
        if matches!(functions.functions()[0], CssTransformFunctionValue::Translate3d(_))
));

let CssKnownPropertyValueRef::Filter(filter) = report.syntax()[1]
    .known().expect("known filter")
    .property_value().expect("ordinary filter")
else { panic!("expected filter") };
assert!(matches!(
    filter.current(),
    CssFilterValue::Functions(functions)
        if matches!(functions.functions()[1], CssFilterFunctionValue::DropShadow(_))
));

let CssKnownPropertyValueRef::ClipPath(clip) = report.syntax()[2]
    .known().expect("known clip path")
    .property_value().expect("ordinary clip path")
else { panic!("expected clip-path") };
assert!(matches!(
    clip.current(),
    Some(CssClipPathValue::BasicShape(CssBasicShapeValue::Polygon(polygon)))
        if polygon.round().is_some()
));
```

The typed transform family covers the selected two-dimensional Transforms 1
functions and the preserved I01 three-dimensional subset with exact arity,
separator, and dimension domains. Easing values distinguish keywords,
`cubic-bezier()`, and `steps()`. Box shadows and filter `drop-shadow()` use
different models, so filter shadows cannot contain `inset` or spread. Filter
lists preserve URL/function order and typed function-specific operands. The
selected basic-shape family exposes `inset()`, `circle()`, `ellipse()`, and
`polygon()`, including polygon `round <length>`.

These are authored syntax values. This crate does not multiply transform
matrices, interpolate or evaluate easing, render shadows or filters, resolve
URLs, compute shape geometry, perform layout or painting, or lower values into
sibling crates. `path()`, `shape()`, `rect()`, `xywh()`, and clip-path
reference-box combinations remain outside the selected shape subset.
`transition`, `animation`, `backdrop-filter`, and `clip-path` therefore retain
their explicit Partial catalog boundaries; support for one typed function does
not promote an aggregate or an unselected production.

## Authored colors and frozen I01 compatibility

The current color model preserves the authored Color 4 grammar rather than a
computed color. It distinguishes named, transparent, current, hexadecimal,
current and deprecated system, legacy and modern RGB/HSL, HWB, Lab/LCH,
Oklab/Oklch, and predefined `color()` branches. Finite specified components
remain authored even when they are outside a computed range, and typed
calculations remain symbolic. The current opacity model likewise preserves a
finite number or percentage, including signed and out-of-range specified
values.

Color-bearing property wrappers expose the current value through `current()`,
and the opacity wrapper exposes its current `CssOpacityValue` through `value()`.
Their `i01_subset()` remains a separate frozen compatibility projection: every
frozen I01 input keeps its exact projection, while a newly accepted current
value returns `None` when the old `CssColor` or `CssOpacity` model cannot
represent it without loss. A missing I01 projection does not make the current
value invalid.

`border-color` accepts one through four colors and exposes the expanded sides
through `CssBorderColors`. Its wrapper's `current()` now returns this aggregate;
migrate single-color consumers to the required `top()`, `right()`, `bottom()`,
or `left()` accessor. Each side retains its complete specified color, including
symbolic values. `CssBorderColors::try_new` checks the same component count for
Rust construction. The frozen `i01_subset()` remains available only for a
single authored component representable by the old color model; multiple
components have no frozen projection even when their colors are equal.

```rust
use surgeist_css::{
    CssAuthoredSystemColor, CssKnownPropertyValueRef, CssOpacityValue,
    parse_style_attribute,
};

let report = parse_style_attribute("color: ActiveBorder; opacity: 150%");
assert!(report.is_clean());

let CssKnownPropertyValueRef::Color(color) = report.syntax()[0]
    .known().expect("known color")
    .property_value().expect("ordinary color")
else { panic!("expected color") };
assert_eq!(
    color.current().system(),
    Some(CssAuthoredSystemColor::ActiveBorder),
);
assert!(color.i01_subset().is_none());

let CssKnownPropertyValueRef::Opacity(opacity) = report.syntax()[1]
    .known().expect("known opacity")
    .property_value().expect("ordinary opacity")
else { panic!("expected opacity") };
assert!(matches!(opacity.value(), CssOpacityValue::Percentage(value)
    if value.value() == 150.0));
assert!(opacity.i01_subset().is_none());
```

The preserved Color 5 surface is intentionally narrower: relative colors cover
`rgb`/`rgba`, `hsl`/`hsla`, `hwb`, `lab`, `lch`, `oklab`, `oklch`, and
predefined RGB/XYZ `color()` spaces with closed per-family channel
environments. `color-mix()` requires an interpolation method and exactly two
colors, accepts optional trailing percentages, and permits hue interpolation
methods only in polar spaces. This crate does not provide `alpha()`, custom
color profiles, `light-dark()`, or `device-cmyk()`.

These values remain authored syntax. This crate does not clamp computed color
or opacity values, resolve `currentcolor` or system colors, evaluate relative
channels or calculations, perform color conversion or gamut mapping, resolve a
mix, apply contrast, serialize computed colors, or lower colors into a sibling
crate.

## Authored Grid repetition and keyframe structure

The six Grid repetition consumers expose a parser-owned current value through
`current()` while preserving their existing `i01_subset()` compatibility view.
Current Grid track lists distinguish general lists from lists containing exactly
one automatic repetition. Integer and automatic repetitions are non-recursive.
The [selected Grid 3 publication](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#intrinsic-auto-repeat)
admits general track sizes inside automatic repetition, including intrinsic
keywords, flexible tracks, `minmax()`, and `fit-content()`. Surrounding tracks
and integer repetitions retain the fixed-size restrictions of Grid 2's
[`<auto-track-list>`](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#typedef-auto-track-list).
`grid-auto-rows` and `grid-auto-columns` still accept track sizes without
`repeat()`. Each explicit axis has its own limit of one automatic repetition.

`CssAuthoredGridAutoRepeat::content()` now returns
`&CssAuthoredGridTrackRepeatContent`. Consumers inspect its ordered `LineNames`
and `TrackSize` members; the former fixed-body return type no longer describes
the selected grammar. Surrounding integer fixed repeats continue to expose
`CssAuthoredGridFixedRepeatContent`. Exact expressible I01 projections remain
available. Typed calculations retain their current symbolic structure when no
exact I01 projection exists. Historical captured inputs and observations remain
unchanged; their explicit current-model witnesses apply the selected grammar.

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
explicit tracks and repetitions, including their exact I01 projections.
`CssGridLineNames::try_new(Vec::new())` accepts the same empty group. A group
does not supply a required track size; reserved line names remain rejected.

The Grid repetition value, the six Grid property records, and the keyframe rule
record remain `Partial`. Subgrid name-repeat, wider Values
math functions, and other unselected Grid property grammar remain unsupported.
Calculation keyframe selectors, string names, and unselected declaration-processing
grammar remain outside the keyframe boundary. Repetition counts and used track
sizes remain unresolved. This crate does not perform Grid layout, cascade
declarations, evaluate or interpolate keyframes, run timelines, or lower either
syntax family into sibling Surgeist crates.

## Typography, font families, and font-face

The authored font surface includes checked four-ASCII-character OpenType tags,
non-negative feature indices, explicit and system `font` branches, synthesis,
and the five variant longhands. `font-family`, explicit-font family lists, the
`@font-face` family descriptor, and `local()` names follow the selected
September 7, 2026 Fonts 4 grammar. Other typography records retain their
individual dated sources.

`CssFontFamilyPropertyValue::families()` and `CssFontPropertyValue::font()` expose
the current models, with exact authored text available through `as_css()`.
These two wrappers no longer expose `i01_subset()`, and the obsolete `CssFont`
payload has been removed; use `CssFontValue` and `CssExplicitFont`. Other font
wrappers retain their separate compatibility projections where available.

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

The six system spellings (`caption`, `icon`, `menu`, `message-box`,
`small-caption`, `status-bar`) are literal names in family contexts. A complete
`font: menu` selects the system-font branch, while `font: large menu` selects a
literal family named `menu`, as specified in
[Fonts 4 §2.7](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-prop).
Whole-value CSS-wide keywords use the declaration's global-value branch.
These distinctions preserve decoded meaning and identifier boundaries. The
family model does not yet provide canonical CSS serialization. A serializer must quote a
literal name or escape each identifier token independently, and must keep
quoted reserved names distinct from generic families. Rejecting decoded U+0000
avoids claiming preservation of a character that CSS replaces with U+FFFD.

`parse_font_face_descriptor_value(source, kind)` parses a complete raw value
using one of the eight `CssFontFaceDescriptorKind` grammars. It returns an owned
`CssFontFaceDescriptorValue` without an invented descriptor-name position.
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
validates kind-specific count/range constraints. `CssFontFeatureValueIndex`
stores exact normalized decimal digits without a machine-integer maximum.
`try_from_decimal` accepts only an optional ASCII sign and digits, rejects
negative nonzero values, and normalizes leading zeros and negative zero.
`to_u32` returns `None` on overflow. Parsed index origins identify the whole
original token; constructors produce no source coordinates. This type is
separate from the older `CssFontFeatureIndex` used by `font-feature-settings`.

The [pinned Fonts 4 section 6.9.1](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-feature-values-syntax)
conflicts with [section 6.9.2](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/).
The provisional section 6.9.1 policy requires exactly two character-variant
indexes, constrains its first index to 0–99, and constrains styleset indexes to
0–20. These three cases remain unresolved. Historical-forms accepts a nonempty
list with no upper bound; stylistic, swash, ornaments and annotation each require
one unbounded index. The second character-variant index is also unbounded.

Ordinary media/supports/container/layer/scope rule lists retain this global
named rule; any style-rule ancestor forbids it, including through intervening
groups. Normalization emits one opaque rule payload with its parent contexts;
it preserves complete body order and emits no property contributions. Payload
members do not individually consume normalization declaration/rule budgets, and
those budgets do not cap allocation. Font matching, mapping winners and cascade
remain downstream; the shared canonical rule writer is unfinished.

`@font-face` retains every valid descriptor occurrence in authored order;
effective typed accessors return the last valid occurrence. Source-list grammar
follows the selected Fonts 4 edition. Other descriptor and property records
retain their individual dated sources, including the previously selected
`font-display`, numeric property weight and descriptor weight/style/stretch
ranges. An invalid or unknown descriptor is dropped
with a `DropDescriptor` diagnostic without erasing valid neighbors. Empty rules
and rules missing `font-family` or `src` remain valid authored syntax. Those two
accessors return `Option`; their absence excludes the face from downstream font
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
`CssFontFaceUrlSource::try_new` accepts these strings as well; its existing
optional return type is retained. Whether a source identifies a usable font
resource is decided by the downstream resource owner.

Each `format()` hint accepts exactly one recognized keyword or one quoted
string under the selected Fonts 4 edition. Empty and unrecognized strings are
valid authored values; resource support is determined later. Multiple quoted
arguments from the earlier Fonts 3 grammar now discard that source member.
`CssFontFormatList::try_new` consequently requires exactly one item, and
`CssFontFormatString::try_new("")` succeeds. The list name and slice accessor
remain available with this single-item invariant. `tech(palettes)` has the
typed `CssFontTechHint::Palettes` representation, and technology order and
repeated entries remain authored.

`CssFontFaceUrlSource::new_with_formats` accepts a checked single-argument wrapper
without another fallible validation step. `formats()` preserves the decoded
authored string. `format()` projects the four legacy strings `woff2-variations`,
`woff-variations`, `truetype-variations`, and `opentype-variations` to their base
formats using ASCII-insensitive matching, without trimming or interpreting other
suffixes. `required_technologies()` yields distinct authored technologies in
first-occurrence order, followed by implied `variations` when absent. Every yielded
technology is required together. `tech()` retains its exact authored order and
repetitions. `CssFontFormatHint::is_equivalent_to` recognizes TrueType/OpenType
compatibility while ordinary equality keeps their identities distinct.

The `font-family` property and descriptor, `font`, `@font-face`, `src`,
font-source and modern-source-hint records cite the September 7, 2026 edition as
`I-FONTS4-20260907`. The family property and descriptor and the narrowly named
modern-source-hint record are `Complete`; the shorthand, rule and source-list
records remain `Partial`.
The older `I-FONTS4` identity keeps its April 22 edition; `O-FONTS3` also
remains available for historical source records. These immutable identities
must not be repointed when adopting a newer production.

Selected descriptors including `font-width`, `font-variation-settings`,
`font-named-instance` and metric overrides remain unfinished. The `font-width`
property and its full percentage grammar remain outside this family slice;
the shorthand accepts the Fonts 3 width keywords, represented by the existing
stretch model. Fonts 4 shorthand components including oblique angles,
non-integer weights, and `xxx-large` or `math` sizes also remain unsupported.
The source-list URL branch accepts `url()` but does not yet implement `src()`
from the referenced Values 4 `<url>` production. These are CSS implementation
gaps. Other historical Fonts 3 and Fonts 4 support records
retain their existing classifications pending reconciliation with the complete
selected profile; their dates bound those claims. These authored models do
not load or match fonts, resolve fallback or feature application, shape glyphs,
apply cascade or substitution, evaluate computed values, expose CSSOM, serialize,
or lower into another Surgeist crate.

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

## Namespaces and complete Selectors 3 syntax

`CssRule::Namespace` retains a top-level `@namespace` declaration with its
optional decoded, case-sensitive `CssNamespacePrefix`, literal
`CssNamespaceName`, and parser-produced position. Namespace names preserve the
authored string or `url()` token value, including empty strings and strings that
are not valid URIs. The crate does not normalize, resolve, or load the value.

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
assert_eq!(selector.key().map(String::as_str), Some("second"));
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
`has_scope_anchor()`. No parent selector list is expanded during parsing. `CssScopedStyleRule` follows the same
leading-declarations and ordered-child contract: its `rules()` returns ordinary `CssRule`
nesting children relative to the scoped style parent, while the enclosing `@scope` keeps its
scoped rule-list grammar.

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
and [Nesting 1](https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nesting-other-at-rules)
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

This property family provides the selected authored grammars for thirteen CSS2
residual properties, Writing Modes 3 `text-combine-upright`,
`text-orientation`, and `unicode-bidi`, UI3 `caret-color`, `outline-offset`, and
`resize`, Containment 1 `contain`, Transforms 1 `transform-box`, and Compositing
1 `background-blend-mode`, `isolation`, and `mix-blend-mode`. Their property
wrappers preserve exact authored CSS and expose typed current values without
performing cascade, layout, pagination, painting, hit testing, containment
semantics, blending, or writing-mode resolution.

`glyph-orientation-vertical` is the selected Writing Modes legacy shorthand,
not a name-equivalent schema alias. Its restricted `auto`, `0`, `0deg`, `90`,
and `90deg` grammar maps to a parser-produced `text-orientation` declaration.
The schema therefore keeps `CssKnownProperty::TextOrientation.aliases()` empty,
while the conformance catalog exposes the explicit
`official.property-alias.glyph-orientation-vertical` record.

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
assert_eq!(alias.status(), CssSupportStatus::Complete);
```

These 27 official rows are public `Complete` atomic records: 24 canonical
properties, the explicit legacy shorthand, and the independent
`official.value.box-edge-keywords` and `official.value.blend-mode` shared-value
records. This activation does not inflate the immutable ledger or promote later
work: it remains 162 property units (161 canonical properties plus the custom
property family), one normative legacy shorthand, and 167 non-property units.
The unchanged 131-row exclusion registry still includes exactly 50 superseded
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

Flexbox 1 `flex-flow` and all nine Multicolumn 1 properties now expose complete
authored grammars and typed current values. `flex-flow` preserves the authored
direction/wrap combination; `columns` preserves its independently optional
width and count; and `column-rule` preserves width, style, and color without
performing layout, pagination, or painting.

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
assert_eq!(feature_values.status(), CssSupportStatus::Partial);
assert_eq!(feature_values.source().id().as_str(), "I-FONTS4-20260907");
assert!(feature_values.unsupported_remainder().is_some());
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
`ext.value.color-mix`, `ext.value.grid-repeat`, `ext.value.basic-shape`,
`ext.descriptor.font-weight-range`, `ext.descriptor.font-style-oblique-range`,
`ext.descriptor.font-stretch-range`,
`ext.property.font-weight-range`, and `ext.supports.selector` remain `Partial`,
with both subset and remainder metadata. The five `ext.media.range.*` records
for width, height, resolution, color and monochrome are now `Complete`, covering
signed symbolic operands and source-ordered chained comparisons.
The `@font-feature-values` record is `Partial`: its authored parser,
checked model and normalization are implemented under the pinned Fonts 4 edition;
three conflicting grammar requirements retain a documented provisional policy.
General rule serialization remains unfinished.

The preceding public support catalog contained 456 records. The 31 additions
above brought it to 487; fourteen additional media feature records and two
custom-media rule/reference records bring the current public support catalog
to 503 records, as declared in
[the catalog source](../src/conformance.rs). That
catalog cardinality is distinct from the immutable official inventory of
exactly 162 property units (161 canonical properties plus the custom-property
family), one normative legacy shorthand, and 167 non-property units. All 219
preserved I01 baseline records retain their classifications, and the exclusion
registry remains exactly 131 rows.

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
[Values 4](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#empty-urls).
An empty URL resolves to an invalid resource; that downstream result does not
invalidate its authored syntax. Serialization here preserves authored target
spelling, rather than performing computed-value URL serialization.

These models are authored syntax only. `surgeist-css` does not evaluate media or
supports conditions, match selectors, resolve URLs, load imported resources,
apply cascade or substitution, compute layer order, or lower syntax into root or
sibling types. Environment matching, resource loading, composition, and
cross-crate adapters remain downstream responsibilities.
