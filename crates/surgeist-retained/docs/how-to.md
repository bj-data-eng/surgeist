# How-to guides

These procedures describe the current APIs exported by [src/lib.rs](../src/lib.rs)
for developers who have completed [getting started](getting-started.md). Each
procedure ends with a focused source test that demonstrates its expected result.

## Apply several edits atomically

Start with a live `Model` and node IDs obtained from its root, snapshots, or
mutation reports.

1. Construct a `Mutation` and append `Patch` values as `MutationEdit`s using
   `Mutation::push` and `Into::into`.
2. Submit the complete batch with `Model::mutate`.
3. On success, inspect the returned `Report::changes` and a fresh snapshot. On
   error, handle the typed `ErrorCode`; the transaction rolls back the batch.

The `mutation_is_atomic_on_error` test inserts a valid child and then attempts an
insertion under a missing parent. The resulting error leaves the root empty.
Run that proof from the crate directory:

```sh
cargo test --offline -p surgeist-retained tests::mutation_is_atomic_on_error -- --exact
```

The named test should pass. The data shapes are in
[src/mutation.rs](../src/mutation.rs), with execution in
[src/model.rs](../src/model.rs) and rollback in
[src/transaction.rs](../src/transaction.rs).

## Resolve children for a projection slot

Start with a live host ID and an element source whose sibling keys are unique.

1. Choose `ProjectionSlot::default(host)` or `ProjectionSlot::named(host, name)`.
2. Build a `ProjectionEdit` with the source and a `ProjectionReplaceMode`, then
   call `Model::apply_projection`.
3. Resolve the slot with `Model::resolve_projection`, or resolve all dirty slots
   together with `Model::resolve_dirty_projections`.
4. Read the resolved children through `Snapshot::projected_children(slot)` and
   inspect the reports for changes. A dirty slot returns
   `ErrorCode::UnresolvedProjection`; resolve it before consuming that view.

The host's canonical children remain available through `Snapshot::children`.
Verify this separation and the dirty-to-resolved transition:

```sh
cargo test --offline -p surgeist-retained tests::projection_updates_projected_children_without_rewriting_canonical_children -- --exact
```

The named test should pass. See [projection types](../src/projection.rs) and the
[explanation of topology and virtual state](explanation.md) before choosing an
identity-preservation mode or a virtual source.

## Dispatch an event to command records

Start with a model containing an input-eligible node whose `Element` has a `Hook`
for the event's trigger.

1. Construct an `Event` with that retained target and event kind. Set propagation
   or a pointer ID when the caller needs those routing inputs.
2. Call `Model::dispatch` and handle any routing error.
3. Read `Report::commands`. The application decides how to execute each command;
   dispatch itself produces records of matching hooks along the route.

The existing click example verifies that a button hook yields its named command:

```sh
cargo test --offline -p surgeist-retained tests::event_routing_emits_commands -- --exact
```

The named test should pass. [src/event.rs](../src/event.rs) owns event, hook,
route, and command data; [src/model.rs](../src/model.rs) owns routing and dispatch.
