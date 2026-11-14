# Slot assignment, flat queries and checkpoints

Manual intent, stored assignment and fresh Slot queries remain separate. This page describes their canonical storage, finite query admission and explicit native checkpoint handoff.

## Assignment and lifecycle hooks

Native HTML Slot is an Element family initialized through HTML's actual-name
producer. `DomBatch::assign_slot` consumes the real HTML assign cursor over raw
Element/Text operands, including duplicates, with one canonical staged owner.
Manual ordered sets/inverses and stored assigned-node lists/inverses are distinct
state. Manual intent may name detached, other-document or self Slots; current
find algorithms filter by the actual host-child relationship. A manual transfer
reassigns only the target's ordinary root. It preserves another root's historical
stored list and inverse until a later actual source operation visits it.
`DomSlot::assigned_nodes` reads stored source assignedNodes(false);
`DomNode::assigned_slot` freshly finds a Slot with open-root presentation.
`stored_assigned_slot` is privileged algorithm state and can differ. Names are
exact UTF-16 no-namespace attribute values, with missing/empty source guards.

Every existing insert/remove/fragment/replace/adopt/attribute/clone path runs
the selected Slot hooks in source order. Clone/import fresh-initializes manual
state instead of copying foreign references. Explicit native destruction cleans
permitted-weak manual references and exact-generation surviving stored inverses;
ordinary detach/adopt/assignment retain historical inverse fields. This native
cleanup creates no synthetic global assignment or slotchange.

## Admission and flat queries

The Slot reference capacity counts each installed manual-list entry, stored-list
entry and pending signal recipient independently; fixed inverse fields are covered
by managed-node capacity. Raw command capacity counts duplicates across the whole
batch before resolution. Replacement candidates use checked temporary storage
bounded by admitted node/input extents and count installed replacements exactly.
These logical limits do not promise physical RSS or budget caller-held captures.
`DomSlotQueryLimits` separately bounds output nodes and scratch references.
Flattening is iterative; retained candidate extents and each traversal frame are
charged until the frame is released. CSS output records include their queue/
parent storage in output admission, with separate scratch for Slot candidates.

`flattened_slottables` uses fresh find results, slottable fallback and recursive
Slot substitution. `DomSnapshot::flat_tree` follows the selected CSS document
algorithm: it retains Slot parents, uses all child-node fallback, replaces a host's
children with its shadow-root children, and preserves explicit outside-document
membership. Its node iterator is source breadth-first construction order.
Both results retain the exact immutable source; neither changes canonical links.

Native consumers use `visit_assigned_nodes_with` for stored assignments and
`visit_flattened_slottables_with` for fresh flattened observations. Visitors
receive canonical nodes resolved through the caller's work admission, so kind
filtering needs no second lookup. `assigned_slot_with` and `find_slot_with` use
the same fresh assignment owner, including open-root presentation. The caller's
step supplies cancellation; its first work, reservation or visitor refusal is
returned unchanged, with only the successful visitor prefix accepted.
The stored assigned-element iterator returns typed errors for invalid captured links.

Flattening admits the full new byte extent of every candidate/frame allocation
before allocation and retains old backing through replacement and copying.
Returned backing drops after its allocation; each copied reference/frame is
metered. Logical query limits remain separate from these physical reservations.
Visitors own output allocation, filtering and its admission. Keep the captured
snapshot and output backing alive through consuming the observation; historical
custody does not authorize execution against retired identities.

`DomSourceEdit::assign_slot_with` borrows caller-owned NodeIds while admitting
and copying the command's exact operand allocation, preserving duplicates. The
prepared assignment owns that command and its canonical and local credits. Source
installation adopts the same canonical allocation and ends the local credit at
the installation fence. The original input remains funded throughout the copy.

## Signals and native checkpoints

Source signals accumulate an ordered deduplicated set in the same transaction.
`Dom::native_checkpoint_step` executes the oldest native FIFO turn. The shared
mutation notification marker captures the current signal set as historical
slotchange intentions (`bubbles=true`, `composed=false`, `cancelable=false`).
The turn yields its [child-list observer deliveries](child-list-observation.md)
before yielding the captured Slot signals.
A failed/cancelled drain preserves the set. Native destroy removes exact retired
pending recipients with `SlotSignalCancelled { reason: Destroyed }` evidence;
detach/adopt/disconnect do not cancel. Drained batches retain their historical
capture; `revalidate` checks the live owner and reports `SkippedDestroyedTarget`
without following generation reuse. Later signals belong to the next checkpoint.
New pending work advances the revision even when final node values are restored.

[Back to surgeist-dom](../README.md).
