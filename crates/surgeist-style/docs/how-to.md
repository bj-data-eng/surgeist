# How-to guides

These procedures assume you have completed [getting started](getting-started.md).
All imports use the public `surgeist_style` front door.

## Apply a selector rule to your tree

Precondition: your application implements [`Tree`](../src/tree.rs), and the
supplied node ID identifies a node whose tag is `button`. The trait supplies node
facts and parent, child, and previous-sibling relationships for the requested
`Traversal`.

1. Build validated declarations, attach them to a selector in a `Sheet`, and
   resolve the node through a `Context`:

   ```rust
   use surgeist_style::{
       Context, Declarations, Length, Resolved, Resolver, Result, Selector, Sheet, Tree,
   };

   pub fn resolve_button<T: Tree>(tree: &T, node: T::Id) -> Result<Resolved> {
       let declarations = Declarations::new().try_margin_top(Length::px(2.0))?;
       let sheet = Sheet::new().rule(Selector::tag("button")?, declarations);
       let mut resolver = Resolver::new(sheet);
       resolver.resolve(Context::new(tree, node))
   }
   ```

2. For a matching node, verify that the returned style's `margin_edges().top`
   equals `Length::px(2.0)`. A node with a different tag does not receive this rule.
3. Supply `.parent(&parent_style)` on the context when inheritance is needed;
   the caller provides that resolved parent explicitly. Use `.traversal(...)`
   when the default projected traversal is not the desired tree view.

The [resolver test used in the first success](../src/resolver.rs#L2569) exercises
this path. The public [construction fixture](../tests/compile_pass/typed_public_construction.rs)
also contains a complete downstream `Tree` implementation.

## Inspect custom-property resolution diagnostics

Precondition: the sheet contains `AuthoredDeclarations` with typed variable
expressions. [`AuthoredDeclaration::with_source`](../src/authored.rs) can attach a
nonzero `StyleSourceId` for identifying a declaration in diagnostic output.

1. Call `resolver.resolve_with_diagnostics(context)` and propagate its `Result`.
2. Read `output.resolved()` for the resulting style and `output.diagnostics()`
   for invalid-at-computed-value reports. Inspect each report's `subject()`,
   `source()`, and `reason()`.
3. Verify the expected reason and source ID against the authored input. The
   existing [`resolve_with_diagnostics_reports_missing_custom_property` test](../src/resolver.rs#L3473)
   demonstrates one missing custom property producing one diagnostic while still
   returning a resolved style.

Construction or tree-access failures use [`Error`](../src/error.rs). A diagnostic
inside a successful result describes a computed-value problem; it is a separate
channel from that error result. See the [diagnostic types](../src/diagnostic.rs)
for the currently represented reasons.

## Determine which property outputs changed

Precondition: you have a before and after `Resolved` value for the same subject.

1. Call `Invalidation::between(&before, &after)`.
2. Read its `layout`, `paint`, `text`, `effect`, and `animation` flags to identify
   the impact categories attached to changed canonical properties.
3. Verify the flags against the changed properties' `metadata().impact_flags()`.
   The implementation is in [invalidation.rs](../src/invalidation.rs), and the
   property-to-impact mapping is in [property.rs](../src/property.rs).

When the input change is already known, `Change::from_properties` or
`Change::from_custom_properties` supplies impact and scope information directly.
Selector, condition, cascade, and style-bucket changes have separate constructors.
These values describe work for a caller to schedule; creating one does not
rematch nodes, clear a resolver cache, or run layout and rendering.
