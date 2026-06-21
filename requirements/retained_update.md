# surgeist::retained Transaction Update

This update defines the target transaction model for `surgeist::retained`. It replaces whole-model clone rollback with a journaled transaction engine and tightens every public retained mutation into an all-or-nothing operation.

Correctness is the priority. Current behavior is useful evidence about real mutation surfaces, but it is not a constraint. If current behavior conflicts with the rules below, the rules below win.

## Purpose

Retained mutations must be correct, auditable, and fast for large UI models. A localized edit in a 10,000-node model should not clone every node, projection cache, virtual anchor, dirty slot, pointer capture, and accumulated change report just to preserve rollback correctness.

The target model:

- Mutates the live model only inside a transaction.
- Records undo data before each write.
- Keeps reports and commands transaction-local until commit.
- Commits only after the entire requested operation succeeds.
- Rolls back every write if any step fails.
- Makes every public mutating entry point all-or-nothing.
- Restores ID allocation state so failed transactions do not perturb later IDs.
- Uses one implementation path for retained mutation semantics.

## Scope

This update owns:

- Atomic transaction semantics for all retained public mutating entry points.
- A private transaction and journal implementation.
- Preflight validation where it avoids unnecessary writes.
- Per-write undo entries for canonical topology, projection topology, projection caches, pending sources, dirty slots, virtual anchors, node owner, node state, focus, pointer capture, allocation state, and accumulated changes where applicable.
- Transaction-local `ChangeSet` and command collection.
- Deterministic rollback order.
- Strict derived-state invariants for focus, focus-within, and pointer capture.
- Tests proving failed operations leave the model observably unchanged.
- Tests proving `resolve_dirty_projections` is one atomic operation.
- Failure-injection tests for projection resolution.
- Performance-oriented tests proving localized mutations do not use whole-model clone rollback.

This update does not change retained's boundary. Retained still does not own layout, style, text shaping, hit testing, rendering, platform input, widgets, application command execution, or virtualization policy.

## API Target

The retained front door should remain compact, but public names may change if correctness requires it. The compact target shape is:

```rust
impl Model {
    pub fn apply(&mut self, patch: Patch) -> Result<Report>;
    pub fn apply_projection(&mut self, projection: ProjectionEdit) -> Result<Report>;
    pub fn mutate(&mut self, mutation: Mutation) -> Result<Report>;
    pub fn resolve_projection(&mut self, slot: ProjectionSlot) -> Result<Report>;
    pub fn resolve_dirty_projections(&mut self) -> Result<Report>;
    pub fn dispatch(&mut self, event: Event) -> Result<Report>;
    pub fn focus(&mut self, id: Option<Id>) -> Result<Report>;
    pub fn capture_pointer(&mut self, capture: PointerCapture) -> Result<Report>;
    pub fn release_pointer(&mut self, pointer: PointerId) -> Result<Report>;
    pub fn take_changes(&mut self) -> ChangeSet;
}
```

Rules:

- Public reports describe committed changes only.
- Failed operations do not alter `Model`, accumulated `ChangeSet`, dirty projection slots, pending projection sources, projection caches, focus, pointer captures, virtual anchors, allocation state, or command output.
- No public method exposes a half-applied model.
- `resolve_dirty_projections` is intentionally stronger than the current per-slot behavior: all dirty slots from the operation's starting set commit together or roll back together.
- A public transaction API is not part of this update. The transaction engine is private unless later authoring work proves a public API is necessary.

## Transaction Semantics

Every public mutating entry point must be atomic at the operation boundary:

- `apply`: one `Patch` either fully commits or leaves the model unchanged.
- `apply_projection`: one `ProjectionEdit` either fully commits or leaves the model unchanged.
- `mutate`: the full `Mutation` batch either fully commits or leaves the model unchanged.
- `resolve_projection`: one slot resolution either fully commits or leaves the model unchanged.
- `resolve_dirty_projections`: all dirty slot resolutions from the starting dirty-slot set either fully commit or leave the model unchanged.
- `focus`: the full focus transition, including derived `focus_within`, either fully commits or leaves the model unchanged.
- `capture_pointer`: the capture table and all affected target states either fully commit or leave the model unchanged.
- `release_pointer`: the capture table and all affected target states either fully commit or leave the model unchanged.
- `dispatch`: command emission and any retained state changes either fully commit or leave the model unchanged.
- `take_changes`: clears accumulated committed changes as one operation.

Rollback must restore these retained facts after failure:

- Root id.
- Live node set.
- Node element data.
- Node owner.
- Canonical parent and children.
- Projected parent links.
- Node state.
- Node key paths and match keys.
- Projection caches.
- Pending projection sources.
- Dirty projection slots.
- Hosted projection slot data.
- Virtual state anchors.
- Focus target.
- Pointer-capture table.
- Accumulated `ChangeSet`.
- Allocation length and any ID allocator/free-list state.

Private vector capacity may differ after rollback. Future ID allocation must not differ because a transaction failed.

## Entry Point Policy

Every public mutating method should go through one shared transaction pattern:

```rust
pub fn apply(&mut self, patch: Patch) -> Result<Report> {
    self.transaction(|tx| tx.apply_patch(patch))
}
```

Rules:

- Start a transaction.
- Run validation and writes through transaction-aware helpers.
- Accumulate changes into the transaction, not directly into `Model::changes`.
- Accumulate commands into the transaction, not into a committed report.
- On success, merge transaction changes into `Model::changes` and return `Report`.
- On error, roll back the journal in reverse order and return `Error`.
- `resolve_dirty_projections` captures the dirty slot list at transaction start and resolves those slots inside the same transaction.

Some current methods validate before writing and cannot fail mid-write today. They still use the shared transaction path so derived state, future stateful dispatch, and pointer capture semantics remain uniform.

## Internal Modules

Add an internal transaction module unless the implementation remains clearer inside `model.rs`:

- `transaction.rs`: private `Transaction`, `Journal`, undo records, commit/rollback control, and transaction-local report accumulation.
- `model.rs`: public entry points, storage, and high-level retained operations.
- `mutation.rs`: public mutation vocabulary and validation helpers.
- `projection.rs`: public projection vocabulary and projection validation helpers.
- `change.rs`: public report vocabulary plus transaction-local merge helpers.

Rules:

- `transaction.rs` stays private.
- Public re-exports do not change unless correctness requires a public naming update.
- Storage helpers may remain private to `model.rs` if moving them makes the transaction API less readable.
- Do not split undo records across many modules unless ownership becomes clearer than locality.

## Transaction Types

Use private types shaped roughly like this:

```rust
struct Transaction<'a> {
    model: &'a mut Model,
    journal: Journal,
    changes: ChangeSet,
    commands: Vec<Command>,
}

struct Journal {
    entries: Vec<Undo>,
}

enum Undo {
    RestoreAllocation { nodes_len: usize },
    RestoreNode { id: Id, node: Option<Node> },
    RestoreOwner { id: Id, owner: Owner },
    RestoreChildren { parent: Id, children: Vec<Id> },
    RestoreProjectedParent { id: Id, projected_parent: Option<Id> },
    RestoreProjectionCache { slot: ProjectionSlot, cache: Option<ProjectionCache> },
    RestorePendingSource { slot: ProjectionSlot, source: Option<PendingProjection> },
    RestoreDirtySlot { slot: ProjectionSlot, was_dirty: bool },
    RestoreVirtualAnchor { slot: ProjectionSlot, key: Key, state: Option<State> },
    RestoreFocus { focus: Option<Id> },
    RestorePointerCapture { pointer: PointerId, target: Option<Id> },
    RestoreState { id: Id, state: State },
}
```

This list is conceptual. The implementation should choose the smallest undo records that keep rollback obviously correct.

Rules:

- Record an undo entry before the first write to a fact.
- Coalesce duplicate undo entries for the same fact within one transaction when practical.
- Roll back entries in reverse order.
- Commit drops the journal without replaying it.
- Journal entries store old values, not closures.
- Avoid storing large snapshots when a small old value is sufficient.
- Do not use `Model::clone()` for normal rollback.
- Debug-only clone comparisons may be used in tests to prove rollback equivalence.
- Transaction-local changes do not need undo records because they are discarded on rollback.

## Validation Strategy

Use validation to avoid unnecessary journaling, but do not make validation so broad that it becomes another full-model traversal by default.

Rules:

- Validate string and element invariants before storage.
- Validate ids, parents, indices, duplicate keys, cycles, projection slots, virtual ranges, and projected ownership before the write that depends on them.
- For a `Mutation` batch, validation accounts for earlier edits in the same transaction.
- If validation requires intermediate state, run it against the transaction's live model view.
- Prefer narrow validation over full-tree validation for localized patches.
- Full model validation may remain available for tests and debug assertions.

## Change Reporting

Reports are part of transaction commit.

Rules:

- A transaction-local `ChangeSet` records changes during the operation.
- On commit, transaction changes merge into `Model::changes`.
- On rollback, transaction changes are discarded.
- Failed transactions return no commands and no changes.
- `take_changes` observes committed changes only.
- Dirty slot reporting follows the retained base spec: unresolved dirty slots remain queryable until resolution or host removal, but a failed transaction restores the previous dirty-slot set exactly.
- No-op operations return an empty `ChangeSet` and should not add undo entries beyond what is needed to prove no write occurred.

## Derived State Invariants

Focus, focus-within, and pointer-captured are retained state facts with model-owned invariants.

Rules:

- `StatePatch` must not expose fields for `focused`, `focus_within`, or `pointer_captured`.
- State patches can update only app/input-mutable retained state. Model-derived focus and pointer-capture facts are changed through retained APIs.
- Focus is changed through `Model::focus`.
- `focus_within` is derived from the focused node's projected ancestor chain.
- Pointer capture is changed through `Model::capture_pointer` and `Model::release_pointer`.
- `State::pointer_captured` means at least one active pointer capture targets that node.
- Capturing a pointer that was already captured by another node clears the old target's `pointer_captured` only if no other pointer remains captured by that old target.
- Releasing one pointer clears the target's `pointer_captured` only if no other pointer remains captured by that target.
- Transactions must journal old capture entries, old target state, and new target state.
- Recomputing derived state must produce final values exactly. Minimizing touched nodes is a performance goal, not a correctness shortcut.

## Projection Transactions

Projection is the highest-risk area because it touches pending sources, dirty slots, caches, projection-owned nodes, projected-parent edges, virtual anchors, state preservation, owner, allocation, and change reports.

Rules:

- `apply_projection` records pending source and dirty-slot changes transactionally.
- Reapplying an equivalent clean projection remains a no-op.
- Resolving a projection slot journals the old cache before replacing it.
- Removing projection-owned nodes journals every removed node, owner, projected parent, child list, state, and affected virtual anchor.
- Reusing a projected node journals old element data, children, key path, match key, state where changed, owner, and projected parent.
- Virtual item window changes journal virtual anchors before insert, remove, or restore.
- A failed projection resolution restores old cache, old pending source, old dirty-slot state, old projected-parent links, old nodes, old owners, old anchors, and allocation state.
- `resolve_dirty_projections` is one transaction over the dirty slot list captured at transaction start.

## Canonical Topology Transactions

Canonical patches should avoid cloning unrelated subtrees.

Rules:

- Insert journals parent children, allocated node slots, allocation length, and key-path changes caused by positional fallback updates.
- Remove journals the removed subtree, hosted projection slots, virtual anchors, focus, pointer captures, parent children, owners, projected parents, and allocation state.
- Replace journals old element data, old children, removed descendants, key path, match key, owner, state, allocation length, and inserted child nodes.
- Move journals old and new parent children, old parent, owner, key path, projected parent, and descendant key paths touched by the move.
- Reorder journals the parent's old child list and any key paths touched by positional fallback updates.
- State patches journal only old state for the target plus focus/capture facts released by invariant repair.
- Attribute, text, class, label, hook, role, and kind patches journal only old element data for the target.

## Focus And Pointer Transactions

Focus and pointer capture use the same transaction machinery as structural mutations.

Rules:

- Focus changes journal old focus, old/new target state, and old `focus_within` state for affected nodes.
- Recomputing `focus_within` should prefer final-value diffing so unchanged nodes are not reported.
- Pointer capture changes journal the capture table entry, old target state, and new target state.
- Releasing invalid focus/capture during other mutations journals the same facts as explicit focus/capture APIs.
- Failed mutations that would have released focus/capture restore the old focus/capture state.

## Storage Requirements

The current `Vec<Option<Node>>` storage can support this update, but the transaction design must not depend on cloning the whole vector.

Rules:

- Node allocation journals the pre-allocation node length.
- Rollback truncates tail allocations when all affected allocations are at the end.
- Rollback restores non-tail removed slots by index.
- Future ID allocation after rollback must match the allocation sequence that would have occurred if the failed transaction had never run.
- If removed node slots are reused, generations or a free-list policy must keep stale ids detectable.
- If storage changes to slab or slotmap later, the transaction journal adapts behind the same public `Id` contract.
- `Id` stability remains a retained contract for live nodes only.

## Performance Expectations

The implementation should make localized mutation costs proportional to touched facts.

Rules:

- A one-node state or attribute patch in a 10,000-node model should not clone every node.
- Applying a localized projection should journal only the affected slot, projection-owned nodes touched by that slot, and relevant ancestors or anchors.
- Resolving a 100-item virtual projection over 200,000 logical items should scale with materialized items and touched anchors, not logical total count.
- Transaction overhead should be small enough that correctness remains on by default.
- Avoid adding a fast unsafe mutation path.

## Testing Requirements

Add contract tests for:

- Failed `Patch::Insert` leaves snapshot, dirty slots, accumulated changes, focus, pointer captures, and future ID allocation unchanged.
- Failed `Patch::Replace` after child removal would have begun leaves the original subtree unchanged.
- Failed `Patch::Move` after parent list changes would have begun leaves both parents unchanged.
- Failed `Mutation` with several successful early edits and one failing late edit leaves the model unchanged.
- Failed `apply_projection` leaves pending sources and dirty slots unchanged.
- Failed `resolve_projection` leaves old projection cache, projected children, projected parents, virtual anchors, owners, allocation state, and dirty slots unchanged.
- Failed `resolve_dirty_projections` leaves all slots unchanged, including slots resolved before the failing slot.
- Failure injection during projection resolution after pending-source removal, cache removal, projected child reuse, virtual anchor removal, old child removal, and new cache insertion.
- Failed focus or pointer capture changes leave derived state and capture tables unchanged.
- Multiple pointer captures targeting the same node keep `pointer_captured` true until the last pointer is released.
- Recapturing a pointer to a different node updates both old and new target derived states.
- `Patch::SetState` cannot corrupt focus, focus-within, or pointer-capture derived state because those fields are not part of `StatePatch`.
- `take_changes` does not include failed transaction changes.
- Equivalent no-op operations produce empty reports and do not grow the journal in debug instrumentation.
- Localized one-node patches in a 10,000-node model avoid whole-model clone rollback.

Testing may use debug-only counters, failure injection, or clone comparisons to prove that transaction rollback is journal-based and observably equivalent. Wall-clock assertions should remain optional or ignored unless they are very stable.

## Implementation Checkpoints

Implement in small checkpoints:

1. Add private transaction scaffolding with transaction-local reports and undo records.
2. Route `apply`, `apply_projection`, `mutate`, `resolve_projection`, `resolve_dirty_projections`, `focus`, pointer capture, and dispatch through the shared transaction entry point.
3. Enforce derived-state invariants for focus, focus-within, and pointer capture.
4. Replace clone rollback for simple state and element-data patches.
5. Replace clone rollback for canonical structural patches.
6. Replace clone rollback for projection resolution and virtual anchors.
7. Add failure-injection tests for projection resolution.
8. Add debug/test instrumentation proving whole-model clone rollback is gone from normal mutation paths.
9. Run retained tests, retained clippy, root `surgeist` tests, and dev harness compile checks.

Each checkpoint should be independently committed so transaction semantics stay auditable.

## Acceptance Criteria

This update is complete when:

- All public retained mutating entry points have explicit all-or-nothing semantics.
- `resolve_dirty_projections` is atomic as one operation.
- Normal mutation rollback uses a journal, not `Model::clone()`.
- Reports and accumulated changes reflect committed transactions only.
- Derived focus, focus-within, and pointer-captured state are not exposed through `StatePatch`.
- Projection resolution rollback restores caches, pending sources, dirty slots, projected links, projection-owned nodes, owners, allocation state, and virtual anchors.
- Canonical structural rollback restores parent/child links, nodes, owners, key paths, state, focus, captures, allocation state, and hosted projection data.
- New transaction rollback tests cover failure after partial internal progress.
- Failure-injection tests cover projection resolution partial-write points.
- Focused performance tests show localized updates avoid whole-model clone rollback.
