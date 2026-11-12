# Native child-list observation

Create an observer with `Dom::create_child_list_observer` and register its
targets with `Dom::observe_child_list`. The registration can include ordinary
descendants. Each mutation queues one record per interested observer in
discovery order. Records retain the exact target, added and removed nodes,
and the sibling identities captured by the mutation algorithm.

Ordinary removal propagates subtree registrations to the removed root until
notification. `take_child_list_records` drains records while those transient
registrations remain live. Updating a registration clears its own derived
transients. Disconnecting clears registrations and records. Retiring an
observer also releases its identity, and a reused slot has a new generation.

The [native checkpoint](slots-and-checkpoints.md#signals-and-native-checkpoints)
captures pending observers and Slot signals together. It drains each observer's
current records and transients when it reaches that observer. Edits during an
earlier delivery can therefore affect a later observer in the same turn.
Release each `DomChildListNotification` after processing it to advance the
turn. Slot signals from that turn follow its observer deliveries.

`DomChildListObservationLimits` bounds observer slots, direct and transient
registrations, queued records, and retained node references. The node-reference
count includes the target, both node lists, and present siblings for each
queued record. Tree changes and their observation state publish together at
the owning [Source boundaries](source-and-style.md).

[Pinned DOM mutation observers](https://raw.githubusercontent.com/whatwg/dom/3071e5f269448350fdea79bb284eedaf17bc431e/dom.bs#L3970-L4445)
