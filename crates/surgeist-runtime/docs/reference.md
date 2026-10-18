# Reference

## Package and source

| Fact | Value | Authority |
| --- | --- | --- |
| Cargo package | `surgeist-runtime` `0.1.0` | [Cargo.toml](../Cargo.toml) |
| Rust library import | `surgeist_runtime` | [Cargo.toml](../Cargo.toml) |
| Edition / minimum Rust | `2024` / `1.97` | [Cargo.toml](../Cargo.toml) |
| Production dependencies | CSS, CSSOM, DOM, HTML and HTML semantics; target-gated `getrandom` 0.3.4 supplies native entropy. The HTML parser is a test dependency | [Cargo.toml](../Cargo.toml) |
| Features | Empty `default` feature set; no other features declared | [Cargo.toml](../Cargo.toml) |
| License | MIT | [LICENSE](../LICENSE) |
| Public entry point | Reexports from `src/lib.rs`; implementation modules are private | [lib.rs](../src/lib.rs) |
| Source lint declarations | `forbid(unsafe_code)` and `warn(missing_docs)` | [lib.rs](../src/lib.rs) |
| Tests and fixtures | `src/tests.rs`, local test modules in `src/resource.rs` and `src/coord.rs`, and private fixtures in `src/testing.rs` | [lib.rs](../src/lib.rs), [resource.rs](../src/resource.rs), [coord.rs](../src/coord.rs) |

## Main contracts

| Area | Entry points and contract source |
| --- | --- |
| State and effects | `Reducer`, `ReducerResult`, `ReducerCommit` in [reducer.rs](../src/reducer.rs); `AppEffect`, `EffectOutcome`, `RuntimeIntent` in [effect.rs](../src/effect.rs) |
| Orchestration | `Runtime`, typed input lanes, budgets, drain reports and errors in [runtime.rs](../src/runtime.rs); `AppLoop::step` delegates one drain in [loop_.rs](../src/loop_.rs) |
| Surfaces and rendering | Runtime-owned IDs in [ids.rs](../src/ids.rs); surface references, routes, lifecycle, invalidations and frame acknowledgements in [surface.rs](../src/surface.rs) |
| Producer wakeups | `AppProxy`, `WakeBridge`, `QueuePolicy`, `ProxyDrainReport` in [proxy.rs](../src/proxy.rs) |
| Resource and observation state | `ResourceState`, `ResourceOperation`, `Freshness` in [resource.rs](../src/resource.rs); subscription keys, refcounts and aggregates in [coord.rs](../src/coord.rs) |
| Services | Registration metadata, `ServiceMailbox`, `MailboxPolicy`, `MailboxPushOutcome` in [service.rs](../src/service.rs) |
| Authored app contracts | `AppManifest::validate`, `ValidatedAppManifest`, `App::try_new` in [descriptor.rs](../src/descriptor.rs); snapshot declarations and entries in [snapshot.rs](../src/snapshot.rs) |
| Causal context | Source-specific origins, current/parent correlations and sequence in [provenance.rs](../src/provenance.rs) |
| Canonical host association | Generation-qualified surface focus/hover captures in [dom_host.rs](../src/dom_host.rs) |
| Native event dispatch | Controlled `DomEventDom` access, listener registration, immutable dispatch paths and resumable progress in [dom_event.rs](../src/dom_event.rs) |
| Shared DOM session and Style orchestration | `DomSession`, weak execution permissions, resource custody and source/networking progress in [dom_style.rs](../src/dom_style.rs) |
| Source event orchestration | `DomSourceRuntime::service_event`, canonical `raise_event_limits` and independent adapter `raise_event_runtime_limits` in [dom_source.rs](../src/dom_source.rs), with finite Session-owned linked jobs and retained actual receipts in [dom_dialog.rs](../src/dom_dialog.rs) |
| Element tasks | Shared source-qualified Details/Dialog task owner and explicit run policy in [element_tasks.rs](../src/element_tasks.rs) |
| Native hosted lifecycle | `HostedRuntime`, explicit limits, top-level creation, settings and queued teardown in [dom_lifecycle.rs](../src/dom_lifecycle.rs) |

## Capacity and scheduling values

| Value | Default or rule | Source |
| --- | --- | --- |
| `RuntimeQueuePolicy::default()` | 65,536 inputs per UI, task, and service lane | [runtime.rs](../src/runtime.rs) |
| `RuntimeBudget::default()` | 64 inputs per turn; at most 32 from each lane | [runtime.rs](../src/runtime.rs) |
| Zero runtime capacity or budget | Rejects every enqueue in that lane, or prevents the applicable draining | [runtime.rs](../src/runtime.rs) |
| `QueuePolicy::default()` | 65,536 proxy inputs | [proxy.rs](../src/proxy.rs) |
| `AppProxy::drain_pending` | Requires a `NonZeroUsize` limit | [proxy.rs](../src/proxy.rs) |
| `MailboxPolicy::bounded(capacity)` | Rejects newest on overflow; overflow counting disabled | [service.rs](../src/service.rs) |
| `MailboxPolicy::drop_oldest()` | Evicts oldest before accepting newest when full with nonzero capacity | [service.rs](../src/service.rs) |
| Zero mailbox capacity | Rejects newest under either policy | [service.rs](../src/service.rs) |

`MailboxPushOutcome` distinguishes accepted input, rejected newest input, and
an accepted input that displaced the returned oldest message. Enabling
`observe_overflow()` records overflows without changing the selected policy.

## Local verification commands

Select affected checks under the [repository process/resource policy](../../../AGENTS.md#command-inventory):

```sh
cargo check --offline --locked -j 1 -p surgeist-runtime
cargo test --offline --locked -j 1 -p surgeist-runtime
cargo clippy --offline --locked -j 1 -p surgeist-runtime --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-runtime -- --check
```

These check compilation, unit tests and doctests, all-target lint results with
unsafe code forbidden and warnings denied, and formatting. Offline checks require the locked dependencies to be available.
Clippy and formatting require their corresponding tools to be available locally.
The source-backed first-success command is in [getting started](getting-started.md).

## Repository boundary

This workspace crate owns its runtime source, public exports and consumer adapters
to lower-level crates. Root `surgeist` owns facade composition, workspace wiring,
cross-crate verification and generated API audit artifacts. The crate carries no generated API
copies. [AGENTS.md](../AGENTS.md) maps repository facts to their authoritative
sources; [explanation](explanation.md) describes the product boundary.
