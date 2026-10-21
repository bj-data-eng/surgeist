# surgeist-runtime

Rust orchestration contracts for Surgeist application and adapter authors.
Runtime coordinates application state, events/effects, surface lifecycle,
invalidation and queued inputs. It emits abstract work intents and owns its
adapters to lower-level contracts, including the bounded native DOM task loop.
Concrete task execution, rendering and OS window event loops belong to their
respective owners.

The [DOM orchestration guide](docs/explanation.md#dom-orchestration) explains
`DomSession`, Source operations, event/task custody and hosted lifecycle limits.
These native operations do not supply complete browser services, positive focus
movement or rendering readiness.
The actual Style association owner also retains [Document stylesheet-set
preferences](docs/explanation.md#document-stylesheet-set-preferences), including
empty-list history and source-defined enable/select behavior. `DomSession::new`
returns the original DOM intact with `PendingParsedAssociations` when parsed
association history remains pending under canonical custody. No current public
Session mutation front installs a parsed report after bootstrap. Retaining that
history does not establish a Style processing service or association readiness.

## Start

From a prepared checkout with the [prerequisites](docs/getting-started.md#prerequisites),
prove that a reducer's task request becomes a runtime intent:

```sh
cargo test --offline --locked -j 1 -p surgeist-runtime --lib tests::runtime_forwards_task_work_as_intents_without_executing_it -- --exact
```

Expected: one passing test checking a forwarded task intent and no diagnostics.
[Getting started](docs/getting-started.md) explains the source and result.

## Documentation

- [Getting started](docs/getting-started.md): run and understand the first proof.
- [How-to](docs/how-to.md): emit intents, handle queues, render surfaces and build snapshots.
- [Reference](docs/reference.md): package facts, public entry points, defaults and checks.
- [Explanation](docs/explanation.md): ownership, state transitions, DOM orchestration and wake delivery.

### Ordinary native Meta preferred-name composition

`DomStyleRuntime::apply_meta` executes an explicit finite ordinary batch against
its existing Document stylesheet-set owners. It consumes only actual reached
nonempty default-style insertion occurrences, preserving UTF-16, invocation
order and existing or original batch-qualified allocation identity even when a
Meta is removed or destroyed later in that batch. Editing content/http-equiv alone does not
replay insertion. Zero-sheet history remains in the same Document preference
state and controls later real Style additions; explicit last-selected history
continues to govern the conditional flag algorithm.

The operation prepares every reached Document producer and optional inline
producer before publishing any owner. Runtime supplies stable external root
capabilities and admits their backing, names, temporary/output banks and old/new
retention under its own limits and the explicit `DomMetaAtomicLimits`. Initial
Runtime work is prepaid into that same native envelope before DOM preparation;
subsequent work charges both owners. No private producer graph tariff is added.
The final fallible DOM commit publishes the actual prepared producers and
canonical tail. Runtime installs already-qualified name rows while root guards
remain held, extracts authentic outputs, then fills only preallocated output
revision fields. Old Runtime names and CSSOM retirement backing survive that
whole fence. This does not claim bounded disposal for every existing DOM value.

The returned outcome owns the real canonical/root/optional-inline commits and
ordered preference observations. `DomStyleMetaError` preserves either the original
Runtime refusal or the native root-qualified producer error; these native errors
do not widen unrelated `DomStyleRuntimeError` results. Multiple operations on one Document report one
authentic final producer revision, never fabricated intermediate CSSOM commits.
Returned immutable observations are caller-owned history; current-bank storage
admission does not cap all observations a caller may retain.

Source and staged Style publication custody must both be idle for this ordinary
front.
Context-free DOM calls still refuse reached default-style without the owner.
The native capability does not consume pending parsed associations or waive
Runtime's intact-DOM Pending bootstrap refusal. Source continuation and public
Source-driver integration remain unavailable through this ordinary front.

### Source native Meta composition

`DomSourceServices.meta` explicitly selects finite native composition for the
existing Session Source driver. `None` preserves reached unavailable behavior;
it cannot discharge a parked native Meta publication. `Some` uses the same
Document preference owner and actual original operation, retaining its inline
profiles, native policy, cumulative work and pending publication across retries.
Native limits can increase without resetting original spent work or outputs.

The driver flushes the preceding actual Style receipt, prepays watcher settlement,
and borrows the operation's exclusive Meta hold before forming external root
banks. Every Runtime scan and allocation prepays the original Source, all active
ancestors and the native envelope. Temporary CSS source-collector scratch keeps
its existing distinction from persistent Runtime `storage_bytes`; its real
formation and old/replacement backing are additionally admitted by Source/native
limits before allocation. The exact persistent Runtime bank and new name/output
union are checked before the new banks form. Caller-owned immutable outputs have
the same output boundary as ordinary observations.

Each actual joined publication returns immediately as
`DomSourceDriveProgress::MetaSource`, including same-name or producer-unchanged
occurrences. It owns the authentic next request or completion, root commits,
optional inline evidence and ordered preference observations. Watcher settlement
finishes before any result is propagated. Complete finishes the actual Style
guard while retaining the whole outcome; Request is returned unserviced. A later
explicit resume processes later Style/policy/event work against the names already
installed. An advance without a Meta occurrence continues the existing service
flow and fabricates no native commit. `DomSourceDriveError` preserves either the
original service error or the native composition error at this driver boundary;
non-driving service APIs retain their existing errors.

## Stewardship

[MIT license](LICENSE) · [Attribution](NOTICE.md)
