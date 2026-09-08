# Reference

## Package

[Cargo.toml](../Cargo.toml) is authoritative for package metadata. It declares
`surgeist-retained` version `0.1.0`, Rust edition `2024`, and one library target
named `surgeist_retained` at `src/lib.rs`. It declares no dependencies, features,
or `rust-version` value. The edition is not a separately declared minimum Rust
version policy.

## Public surface

All public exports enter through [src/lib.rs](../src/lib.rs); its implementation
modules are private.

| Area | Public types and operations | Source |
| --- | --- | --- |
| Authored nodes | `Element`, `Kind`, `Role`; validated keys and semantic strings | [element.rs](../src/element.rs), [string.rs](../src/string.rs) |
| Retained identity | Opaque `Id`, authored `Key`, structural `KeyPath` | [identity.rs](../src/identity.rs) |
| Model and updates | `Model`, `ModelRevision`, `Patch`, `Mutation`, `MutationEdit`, canonical `ReplaceMode` | [model.rs](../src/model.rs), [mutation.rs](../src/mutation.rs) |
| Projection | `ProjectionSlot`, `SlotKey`, `ProjectionEdit`, `ProjectionSource`, `ProjectionReplaceMode` | [projection.rs](../src/projection.rs) |
| Virtual source | `VirtualProjection`, `VirtualRange`, `VirtualItem`, optional `SourceRevision` | [projection.rs](../src/projection.rs) |
| Reads and selectors | `Snapshot`, `NodeRef`, `SelectorTraversal`, metadata and sibling facts | [snapshot.rs](../src/snapshot.rs) |
| State and input | `State`, `Presence`, `StatePatch`, `RuntimeStatePatch`, `PointerId`, `PointerCapture` | [state.rs](../src/state.rs) |
| Events and commands | `Hook`, `Trigger`, `Event`, `Intent`, `Route`, `Propagation`, `Phase`, `Command` | [event.rs](../src/event.rs) |
| Results | `Report`, `ChangeSet`, `ChangeFlags`, selector invalidations, `Error`, `ErrorCode`, `Result` | [change.rs](../src/change.rs), [error.rs](../src/error.rs) |

`Model` provides `apply`, `mutate`, `apply_projection`, `resolve_projection`,
`resolve_dirty_projections`, `route`, `dispatch`, `focus`, `capture_pointer`, and
`release_pointer`. `snapshot` borrows the current model; `take_changes` drains its
accumulated changes.

## Important distinctions

| Types or views | Current contract |
| --- | --- |
| Canonical `ReplaceMode` | Replacement preserves the target ID. `PreserveCompatible` requires the same kind; `AllowKindChange` permits a different kind. |
| `ProjectionReplaceMode` | `PreserveCompatible` reuses a matching root only for the same kind; `PreserveIdentity` permits a kind change; `ResetIdentity` allocates fresh roots. |
| `SelectorTraversal` | `Canonical` uses canonical relationships. `ProjectedDefaultSlot` uses the default slot's effective children; named slots are excluded from this selector policy. |
| `SelectorIndex` / `SelectorCount` | Indexes are zero-based. Counts are nonzero; constructing a count of zero returns `None`. |
| `ModelRevision` / `SourceRevision` | The model advances its revision after committed snapshot-observable changes. A virtual source may carry a caller-supplied source revision. |
| `Report` | Contains mutation changes and command records. An empty changeset does not necessarily mean the model revision stayed unchanged. |

The contracts above are implemented in the corresponding source files and covered
by focused tests in [src/tests.rs](../src/tests.rs).

## Local checks

Run commands from the crate directory with the corresponding tooling already
installed. [AGENTS.md](../AGENTS.md) owns the local command inventory and agent
discovery; the following are offline invocations of those checks.

| Purpose | Command |
| --- | --- |
| Compile the library | `cargo check --offline -p surgeist-retained` |
| Run crate tests and documentation tests | `cargo test --offline -p surgeist-retained` |
| Check all targets with Clippy | `cargo clippy --offline -p surgeist-retained --all-targets -- -F unsafe-code -D warnings` |
| Check Rust formatting | `cargo fmt -p surgeist-retained --check` |

The tracked unit tests live in [src/tests.rs](../src/tests.rs), enabled by
`src/lib.rs`; compile-fail documentation examples also live in
[src/state.rs](../src/state.rs). Root `surgeist` owns the API generator and
generated audit artifacts, as recorded in [AGENTS.md](../AGENTS.md).
