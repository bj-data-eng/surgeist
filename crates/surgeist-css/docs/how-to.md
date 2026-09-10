# How-to

These examples assume a consumer configured as in [getting started](getting-started.md).
They use the public exports from [src/lib.rs](../src/lib.rs).

## Handle a recovery report

Call `parse_sheet` for a stylesheet or `parse_style_attribute` for a declaration
list. Read both `syntax()` and `diagnostics()` before deciding whether recovered
input is acceptable. For example:

```rust
use surgeist_css::{CssRecoveryAction, CssRule, parse_sheet};

let report = parse_sheet(
    ".before { color: red; } @unknown value; .after { color: blue; }",
);
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
use surgeist_css::{CssImportance, CssPropertyNameRef, parse_style_attribute};

let report = parse_style_attribute("color: red; mystery: 1; width: 2px !important");
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
separate. The assertions below check authored width text and its compatibility
projection.

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
use surgeist_css::{
    CssImportance, CssKnownDeclaredValueRef, CssKnownPropertyValueRef,
    parse_style_attribute,
};

let report = parse_style_attribute("width: calc(100% - 12px) !important");
let declaration = &report.syntax()[0];
assert_eq!(declaration.importance(), CssImportance::Important);
let known = declaration.known().expect("known declaration");

match known.declared_value() {
    CssKnownDeclaredValueRef::Property(property) => match property {
        CssKnownPropertyValueRef::Width(width) => {
            assert_eq!(width.as_css(), "calc(100% - 12px)");
            assert!(width.i01_subset().is_some());
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

Each row in the [property schema](../src/properties.rs) has a generated
`Css<SchemaVariant>PropertyValue` wrapper. `as_css()` returns the exact authored
ordinary value, preserving its interior spelling and trivia while excluding
boundary trivia and the terminal importance annotation. For checked construction,
this text comes from token-preserving serialization of the supplied components;
`value_components()` retains their original or programmatic provenance.
Where available, `i01_subset()` exposes the compatibility payload only when
the value belongs to the frozen I01 representation. The `font-family` and `font`
wrappers instead expose only their current `families()` and `font()` accessors,
which distinguish generic families from literal names and preserve decoded
identifier boundaries. Their obsolete joined-string projections were removed.

The `overflow` row illustrates the wrapper/payload distinction. The generated
`CssOverflowPropertyValue` is the authored property wrapper, while
`CssOverflowI01PropertyValue` is the renamed I01 payload containing the
`Single` and `Pair` shapes.

The [compatibility explanation](explanation.md#symbolic-values-and-compatibility)
defines the I01 projection. This inspection model leaves parsing, recovery, and
diagnostic behavior unchanged.

## Require clean input

Validation is available with the ordinary dependency:

```toml
surgeist-css = { path = "../surgeist/crates/surgeist-css" }
```

Remove `app-strict` from existing dependency feature lists. The validators retain
their names and behavior. Call the matching validator when every recovery should
reject the input:

```rust
use surgeist_css::{CssRecoveryAction, validate_style_attribute};

let declarations = validate_style_attribute("color: red")
    .expect("clean style attribute");
assert_eq!(declarations.len(), 1);

let failure = validate_style_attribute("color: red; mystery: 1")
    .expect_err("unknown property requires recovery");
assert_eq!(failure.diagnostics().len(), 1);
assert_eq!(failure.first().action(), CssRecoveryAction::DropDeclaration);
```

The first call returns retained declarations; the second returns the complete
recovery diagnostic sequence. For stylesheets, use `validate_sheet`. Enabling
this feature leaves ordinary parsing and recovery unchanged.

## Inspect support metadata

Use `feature_metadata` for an exact stable feature ID, `property_metadata` for a
property name, and `specification_source` for a source ID. Check `status()` and,
for `Partial` records, both `supported_subset()` and `unsupported_remainder()`.
The [conformance reference](reference.md#conformance-sources-and-atomic-records)
contains lookup examples with assertions and explains aggregate aliases and
exclusions. A metadata status is not a substitute for examining a parse report.
