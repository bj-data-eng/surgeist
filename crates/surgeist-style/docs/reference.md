# Reference

## Package and compatibility

[Cargo.toml](../Cargo.toml) declares package `surgeist-style` version `0.1.0`,
library import name `surgeist_style`, Rust edition `2024`, and the MIT license.
Its dependency is `peniko = "=0.6.1"`; its development dependency is `trybuild =
"1"`. It declares no feature flags or `rust-version`. The crate contains no local
toolchain pin or CI configuration. Its dependency resolution is the shared
[product Cargo.lock](../../../Cargo.lock).

The [root manifest](../../../Cargo.toml) owns product workspace membership and
the facade Rust-version requirement. Source revisions are committed together in
the Surgeist repository. See the [local guide](../AGENTS.md) and
[root guide](../../../AGENTS.md) for the authority mapping.

## Public interface map

The public interface is exposed by [src/lib.rs](../src/lib.rs). The source
modules themselves are private.

| Area | Principal public types | Source |
| --- | --- | --- |
| Values and validation | `Value`, `Length`, `StyleColor`, semantic wrappers, `Property`, `Metadata` | [value.rs](../src/value.rs), [property.rs](../src/property.rs), [calc.rs](../src/calc.rs) |
| Ordinary declarations | `Declaration`, `TypedDeclaration`, `Declarations` | [declaration.rs](../src/declaration.rs) |
| Authored cascade input | `AuthoredDeclaration`, `AuthoredDeclarations`, `AuthoredProperty`, `AuthoredValue`, `CssWideKeyword` | [authored.rs](../src/authored.rs) |
| Custom properties | `CustomPropertyName`, `CustomPropertyValue`, `VariableDependentValue`, `VariableExpression` | [custom.rs](../src/custom.rs) |
| Matching facts | `Tree`, `Node`, `Traversal`, `StyleTag`, `StyleClass`, `StyleState` | [tree.rs](../src/tree.rs), [identity.rs](../src/identity.rs) |
| Selectors and rule scopes | `Selector`, `Compound`, `ComplexSelector`, `SelectorMatchContext`, `RuleScope` | [selector.rs](../src/selector.rs), [scope.rs](../src/scope.rs) |
| Conditions | `Condition`, `ConditionFacts`, `MediaEnvironment`, `ContainerFacts`, `QueryLengthBasis` | [condition.rs](../src/condition.rs) |
| Sheets and precedence | `Sheet`, `Rule`, `RuleTarget`, `RulePrecedence`, `LayerRegistry`, `StyleBucket` | [sheet.rs](../src/sheet.rs), [precedence.rs](../src/precedence.rs), [bucket.rs](../src/bucket.rs) |
| Resolution | `Resolver`, `Context`, `Resolved`, `ResolvedWithDiagnostics` | [resolver.rs](../src/resolver.rs) |
| Errors and diagnostics | `Result`, `Error`, `ErrorCode`, `StyleDiagnostic`, `StyleSourceId` | [error.rs](../src/error.rs), [diagnostic.rs](../src/diagnostic.rs) |
| Invalidation | `Invalidation`, `Change`, `Scope`, typed input-change enums | [invalidation.rs](../src/invalidation.rs) |

`Context::new` selects projected traversal and the element style bucket. Parent,
local, and animated styles are absent until supplied; media and container facts
are provided through context builders. `Tree::version_hint()` returning `None`
disables resolver caching for that context.

`Resolved` stores canonical properties. Its named getters expose typed values;
`iter()` exposes stored entries. `get(Property)` panics for a property absent from
that canonical map, including shorthand variants such as `Property::Margin`.
Use the corresponding assembled getter, such as `margin_edges()`, for those
values. `Declarations::get` instead returns an `Option<&Value>`.

`CssWideKeyword` currently contains `Inherit`, `Initial`, `Unset`, and
`RevertLayer`. `RulePrecedence` contains layer order, selector specificity, and
source order. These are descriptions of the implemented types, not a claim of
complete CSS coverage.

## Verification sources and commands

The [repository guide](../AGENTS.md) owns the local command inventory. For a
checkout with the required toolchain components and dependencies already present,
the Cargo checks can be run without dependency downloads:

```sh
cargo check --offline -p surgeist-style
cargo test --offline -p surgeist-style
cargo clippy --offline -p surgeist-style --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-style --check
```

Unit tests live beside the implementation under `#[cfg(test)]`.
[tests/type_safety.rs](../tests/type_safety.rs) runs the
[compile-fail fixtures](../tests/compile_fail) against their `.stderr` expectations
and the [public construction fixture](../tests/compile_pass/typed_public_construction.rs).
The [getting-started guide](getting-started.md) selects one resolver test for a
focused first success. Root Surgeist owns integration tests and generated API
audit artifacts; this leaf carries no generator or generated API copies.
