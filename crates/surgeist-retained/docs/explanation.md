# Explanation

## A retained semantic boundary

`surgeist-retained` owns semantic durability: node identity, canonical topology,
projection identity and traversal, interaction state, event routes, and mutation
reports. Its [public boundary](../src/lib.rs) excludes layout, style, rendering,
platform input, widget implementations, and application command execution. A
semantic tag, role, or hook carries facts for consumers without supplying those
other systems.

This workspace crate owns its retained implementation and public surface. Root
`surgeist` owns composition and cross-crate adapters, integration tests, workspace
wiring, and API generation. [AGENTS.md](../AGENTS.md) records
the ownership map; this guide describes the resulting model concepts.

## Identity, snapshots, and transactions

An authored `Element` contains semantic metadata and child elements. A `Model`
stores retained nodes with opaque `Id` handles, state, and topology. A `Key` is an
authored matching value; a `KeyPath` describes a path through canonical,
projection-slot, or virtual components. These are distinct from a node's ID.
Their definitions are in [element.rs](../src/element.rs) and
[identity.rs](../src/identity.rs).

`Snapshot` is a borrowed view of the model, not an independently stored copy.
Mutating entry points use a transaction: successful operations publish changes
and commands, and errors roll back recorded state. `ModelRevision` advances for
committed changes visible through snapshots. Resolving a dirty empty projection
can therefore change the revision even when its report contains no node changes.
See [model.rs](../src/model.rs), [transaction.rs](../src/transaction.rs), and the
revision tests in [tests.rs](../src/tests.rs).

## Canonical and projected trees

Canonical children are the model's stored parent/child topology. A projection
slot belongs to a host and has a default or named key. Its source resolves into
projection-owned roots without replacing the host's canonical children. Those
roots have a projected parent pointing to the host; nested children within each
root use canonical relationships.

Applying a projection stores pending input and marks its slot dirty. Resolution
materializes the input and publishes a clean cache. Projected reads reject dirty
slots so callers must resolve that transition before using the view. A default
slot without a cache falls back to canonical children. Named slots are separately
addressable, while selector traversal currently offers only canonical and
default-slot projected policies. [model.rs](../src/model.rs) owns these behaviors;
[snapshot.rs](../src/snapshot.rs) exposes the read policies.

## Virtual windows preserve selected state

A virtual projection describes a logical total and a dense half-open range of
supplied items. The model materializes only those items. Each item has its own
matching key, separate from its root element's metadata. Matching live items can
reuse retained roots according to the projection replacement mode.

When a virtual item leaves the materialized set, the model saves a state anchor
under its slot and key. Rematerialization allocates a node and can restore that
anchor. The anchor retains selected, checked, and expanded values; transient
interaction state is reset. This is continuity of selected state across a
window change, not a promise to retain an off-window node ID. See
[projection.rs](../src/projection.rs), [model.rs](../src/model.rs), and
[State::durable_anchor](../src/state.rs).

## Reports connect consumers to model changes

`ChangeSet` records inserted, removed, moved, and changed nodes, changed projection
slots, and selector invalidation pressure. Metadata, sibling structure,
projection, and state invalidations describe which facts may need recomputation.
The model exposes selector facts without implementing a selector language.

Event routing uses the retained projected ancestry and input eligibility.
Dispatch collects matching hook commands along that route into a `Report`.
Consumers own command execution and any layout, styling, or rendering work
prompted by the changes. The relevant data contracts live in
[change.rs](../src/change.rs) and [event.rs](../src/event.rs).
