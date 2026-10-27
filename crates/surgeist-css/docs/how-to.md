# How-to

These examples assume a consumer configured as in [getting started](getting-started.md).
`CssParser` and `CssParseReport` are crate-root items; every other item is imported from the module
that owns it, as listed in the [crate documentation](../src/lib.rs).

## Handle a recovery report

Call `CssParser::parse_sheet` for a stylesheet or `CssParser::parse_block_contents` for a
declaration list. Read both `syntax()` and `diagnostics()` before deciding whether recovered
input is acceptable. For example:

```rust
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::rule::CssRule;
use surgeist_css::CssParser;

let report = CssParser::default().parse_sheet(".before { color: red; } @unknown value; .after { color: blue; }");
assert_eq!(report.syntax().rules().len(), 2);
assert!(matches!(report.syntax().rules()[0], CssRule::Style(_)));
assert_eq!(
    report.diagnostics()[0].action(),
    CssRecoveryAction::DropAtRule,
);
```

The assertions verify two retained style rules and an explicit `DropAtRule`
action. Match diagnostic codes and actions rather than display text, and keep a
wildcard for non-exhaustive variants. See [source coordinates and diagnostics](reference.md#source-coordinates-and-diagnostics).

## Parse and inspect a style attribute

Style attributes use the same ordinary declaration parser as style-rule blocks. Declarations retain their authored order, source position, coupled property/value type, and terminal `!important` state.

```rust
use surgeist_css::declaration::CssImportance;
use surgeist_css::property::CssPropertyNameRef;
use surgeist_css::CssParser;

let report = CssParser::default().parse_block_contents("color: red; mystery: 1; width: 2px !important");
assert_eq!(report.syntax().len(), 2);
assert_eq!(report.diagnostics().len(), 1);
let width = &report.syntax()[1];
assert_eq!(width.importance(), CssImportance::Important);
assert!(matches!(width.property_name(), CssPropertyNameRef::Known(_)));
```

The example verifies two retained declarations, one diagnostic, and the
terminal importance of the width declaration. Parsing does not apply that
importance to a cascade.

## Inspect a known declaration

Start with a parsed or checked declaration and call `known()`. For ordinary values, use the
property-specific wrapper; keep global and substitution-dependent branches
separate. The assertions below check the authored width text and the exact
checked calculation branch.

Parsing and `parse_property_value` construct `CssKnownDeclaration` through the
same property grammar. Its fields are private and its `property()` identity is
derived from the active coupled value, preventing a property/value mismatch. `declared_value()` returns exactly one
`CssKnownDeclaredValueRef` branch: `Property`, `Global`, or
`SubstitutionDependent`. The convenience accessors `property_value()`,
`global()`, and `substitution_dependent()` are mutually exclusive views of those
same three branches.

Ordinary property values are borrowed through the non-exhaustive
`CssKnownPropertyValueRef`. Match the concrete property wrapper and retain a
wildcard for future variants:

```rust
use surgeist_css::declaration::CssImportance;
use surgeist_css::property::{CssKnownDeclaredValueRef, CssKnownPropertyValueRef};
use surgeist_css::values::box_model::{CssBoxSize, CssSizeValue};
use surgeist_css::CssParser;

let report = CssParser::default().parse_block_contents("width: calc(100% - 12px) !important");
let declaration = &report.syntax()[0];
assert_eq!(declaration.importance(), CssImportance::Important);
let known = declaration.known().expect("known declaration");

match known.declared_value() {
    CssKnownDeclaredValueRef::Property(property) => match property {
        CssKnownPropertyValueRef::Width(width) => {
            assert_eq!(width.as_css(), "calc(100% - 12px)");
            assert!(matches!(width.value(), CssSizeValue::BoxSize(
                CssBoxSize::LengthPercentage(length)
            ) if length.calculation().is_some()));
        }
        _ => panic!("expected width"),
    },
    CssKnownDeclaredValueRef::Global(_)
    | CssKnownDeclaredValueRef::SubstitutionDependent(_) => {
        panic!("expected an ordinary property value")
    }
    _ => panic!("future declared-value branch"),
}
```

Each ordinary property variant borrows one `CssPropertyValue<T>` holding authored text and one
checked semantic value of the property's grammar; the property identity belongs to the
`CssKnownPropertyValueRef` variant. `as_css()` preserves interior spelling and trivia while excluding
boundary trivia and terminal importance. Checked construction serializes supplied
components without losing their provenance in `value_components()`.

Its one semantic accessor is `value()`. The `all` property has only global and
substitution-dependent branches, so `property_value()` returns `None`.

The `overflow` wrapper exposes `CssOverflowValue` through `value()`. Inspect
`x()`, `authored_y()`, and `y()` to distinguish the authored arity from its
specified axis assignment.

## Require clean input

Validation is available with the ordinary dependency:

```toml
surgeist-css = { path = "../surgeist/crates/surgeist-css" }
```

The crate defines no optional features. Call `CssParseReport::into_validation_result` when every
recovery should reject the input:

```rust
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::CssParser;

let declarations = CssParser::default().parse_block_contents("color: red").into_validation_result()
    .expect("clean style attribute");
assert_eq!(declarations.len(), 1);

let failure = CssParser::default().parse_block_contents("color: red; mystery: 1").into_validation_result()
    .expect_err("unknown property requires recovery");
assert_eq!(failure.diagnostics().len(), 1);
assert_eq!(failure.first().action(), CssRecoveryAction::DropDeclaration);
```

The first call returns retained declarations; the second returns the complete
recovery diagnostic sequence. `into_validation_result()` accepts exactly a clean ordinary parse report; it selects no other grammar.
Use the ordinary report when recovered syntax and
its diagnostics are useful to the caller.

## Inspect support metadata

Use `conformance::feature_metadata` for an exact stable feature ID,
`conformance::property_support_metadata` for a property name, and
`conformance::specification_source` for a source ID. Check `status()` and, for `Partial` records, both
`supported_subset()` and `unsupported_remainder()`.
The [conformance reference](reference.md#conformance-sources-and-atomic-records)
contains lookup examples with assertions and explains atomic records and
exclusions. A metadata status is not a substitute for examining a parse report.
`Partial` does not prove that syntax outside the documented subset is valid CSS:
the remainder may describe a known implementation gap or unresolved standard
applicability. Use the parser to check implemented acceptance and the cited
standard to determine normative validity.

### Distinguish support and intrinsic metadata

`property_support_metadata` returns `CssPropertySupportMetadata` for a recognized
catalog property. Use `feature()` for its support record, `property()` for its
canonical identity, and `canonical_name()` and `aliases()` for its catalog
spellings.

For intrinsic property information, call `CssKnownProperty::metadata()` or
`CssPropertyGrammar::metadata()`. Intrinsic metadata is available for every
current catalog property and recognized legacy shorthand grammar. The returned
`CssPropertyMetadata` describes property kind, longhand inheritance and initial
values, or shorthand members. Support lookup remains independent.
See [intrinsic metadata and authored grammar identity](reference.md#intrinsic-metadata-and-authored-grammar-identity)
for the checked initial-value and shorthand contracts. Metadata availability
does not establish complete authored grammar support.

## Format a literal selector payload

Pass the actual namespace environment and rule role to the pure selector owner.
The parsed wrapper retains its admission role and original occurrence:

```rust
use surgeist_css::selector::{CssNamespaceContext, CssParsedStyleSelectors, CssStyleSelectorContext};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::CssParser;
let namespaces = CssNamespaceContext::default();
let report = CssParsedStyleSelectors::parse(&CssParser::default(), ".card, > [|data-x]", &namespaces, CssStyleSelectorContext::Nested);
let parsed = report.syntax().as_ref().expect("admitted nested list");
let text = parsed.serialize_cssom(&namespaces, CssSerializationLimits::default()).unwrap();
assert_eq!(text, "& .card, & > [data-x]");
assert_eq!(parsed.origin().source().as_str(), ".card, > [|data-x]");
```

Pass `CssSerializationLimits` to each list's `serialize_cssom` for one atomic cumulative budget.
A list with no explicit anchor and no relative member serializes in every role; a
relative-only list serializes in the `Nested` and `Scoped` roles. Only an explicit
anchor of the wrong domain (a nesting anchor in `Scoped`, a scope anchor in `Ordinary` or
`Nested`) and a relative member in `Ordinary` fail. A missing
required namespace binding has the distinct `NamespaceBindingUnavailable`
error (`CssSerializationError::is_namespace_binding_unavailable`). Derive retained sheet bindings with `CssNamespaceContext::from_sheet`,
or supply host-owned bindings with `from_bindings`; do not infer namespace names
from prefix text. Contextless whole-rule output preserves symbolic spelling.

For scope getters, call `CssScopeRule::serialize_cssom_start` with the namespace context, the actual
enclosing `CssScopeNestingContext` and the limits, and `serialize_cssom_end` for the
introduced scope. These return nullable bound text without parentheses, independently of
whole Scope formatting availability. See the
[literal selector context contract](reference.md#authored-literal-cssom-rule-and-sheet-output).
