# How the style boundary fits together

`surgeist-style` supplies Rust-authored style contracts for Surgeist. Its public
front door brings together typed values, declarations, matching, cascade,
resolution, validation, and invalidation. The caller supplies tree facts and
context; the result is style data that downstream owners can consume.

## From declarations to resolved values

The crate has two declaration paths. `Declarations` validates ordinary
property/value pairs and expands supported shorthands into canonical properties.
`AuthoredDeclarations` also preserves explicit CSS-wide keywords, custom-property
values, variable-dependent expressions, and optional diagnostic source IDs.
Authored token storage preserves text supplied by the caller; it does not parse
CSS into these typed structures. These paths meet when a [sheet](../src/sheet.rs)
stores rule declarations for the [resolver](../src/resolver.rs).

A `Sheet` combines selectors, declaration sets, conditions, scope, style buckets,
and precedence. Matching uses the caller's `Tree` implementation, including the
chosen canonical or projected relationships. A style bucket selects an element
or represented pseudo-element target without requiring the crate to materialize
a rendered box.

Resolution starts from canonical property defaults and the explicitly supplied
parent style. Matching rules contribute cascade candidates; custom properties
and variable-dependent expressions are resolved through their typed model. Local
declarations are then applied, followed by animated declarations. The resulting
`Resolved` value retains property values, custom-property results, and dependency
information. Symbolic values can remain in this result: resolving a style does
not mean every length has become a pixel measurement or every color has become
render-ready channel data.

## Results, diagnostics, and downstream work

Validation and tree-access failures use the crate's `Result` and `Error` types.
`resolve_with_diagnostics` additionally collects invalid-at-computed-value
reports alongside a resolved style. This distinction lets callers identify a
problematic authored expression while inspecting the value the resolver produced.

[Invalidation](../src/invalidation.rs) separates output impact categories from
rematch and scope information. It describes effects a caller may need to process;
the caller still owns scheduling those effects. The current selector, condition,
cascade, and bucket `Change` constructors request whole-tree rematching. Resolution
returns an owned `Resolved` value to the caller; the resolver retains its own
cached copies. Cache management is explicit through its public methods, and
caching depends on the tree's version hint.

## Ownership and current limits

The [repository guide](../AGENTS.md) assigns this leaf the style domain. Root
Surgeist owns public composition, cross-crate adapters, integration tests, workspace
wiring, and generated API audit artifacts. CSS parsing, layout
and text algorithms, and render lowering fall outside the intended leaf boundary.

One current implementation detail differs from that intended boundary:
[value.rs](../src/value.rs) implements `From<Color> for peniko::Color`, and
[Cargo.toml](../Cargo.toml) declares the rendering dependency. This documentation
describes the existing conversion without treating it as a change in ownership.

The [July 2026 review](https://github.com/bj-data-eng/surgeist-style/blob/e1122b9d0ed80266f3b40901dc18606ec9a86fb1/plans/crate-review-2026-07-12.md) is a historical review
reference at the revision named in that file. It records correctness,
performance, vocabulary, and boundary concerns, including cache identity,
cascade behavior, and the rendering conversion. Its findings and recorded check
results are not a current verification run, a CSS conformance guarantee, or an
implementation plan. Current types and behavior are defined by the
[public exports](../src/lib.rs), implementation, and focused tests.
