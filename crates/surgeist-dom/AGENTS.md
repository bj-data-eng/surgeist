# surgeist-dom Crate Guide

Read [../../AGENTS.md](../../AGENTS.md) for repository ownership, PISCT routing,
compatibility, generated artifacts and command/resource policy. This supplement
records DOM-specific requirements; it grants no additional authority.

## Repository Identity And Ownership

DOM owns the canonical live tree, stable identities, atomic edits, Source
continuations, immutable snapshots and complete change coverage. Use the same
State/Draft/Source owner for ordinary nodes, shadow/Template ownership, Ranges,
feature state and installation. HTML owns language values/interpreters and
Semantics owns qualified queries; DOM owns its direct adapters. Runtime owns
scheduling, delivery and host/resource orchestration.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies and features | [Cargo.toml](Cargo.toml) |
| Public front door and bounded capabilities | [src/lib.rs](src/lib.rs), [README.md](README.md) |
| Canonical storage, publication and admission | [src/store.rs](src/store.rs), [src/store/](src/store/) |
| Batch and Source fronts | [src/batch.rs](src/batch.rs), [src/source.rs](src/source.rs), [src/source/](src/source/), [src/store/source/](src/store/source/) |
| Identity, snapshots and change history | [src/identity.rs](src/identity.rs), [src/snapshot.rs](src/snapshot.rs), `DomChanges` in `src/store.rs` |
| Native family state, hooks and integration | Corresponding modules in `src/` and `src/store/`; [topic contracts](README.md#read-the-contract-for-the-affected-domain) |
| HTML construction/report installation | [src/installation.rs](src/installation.rs), [src/parsed_installation.rs](src/parsed_installation.rs), their store adapters |
| Captures, external facts and identity maps | [src/semantic.rs](src/semantic.rs), [src/host.rs](src/host.rs), owning tests |
| Hosted lifecycle, dispatch and fullscreen | [src/hosted.rs](src/hosted.rs), [src/dispatch.rs](src/dispatch.rs), [src/fullscreen.rs](src/fullscreen.rs), corresponding store modules |
| Focused behavior and source witnesses | [tests/](tests/), private owning tests under `src/store/`, [../../references/SOURCE-EDITIONS.md](../../references/SOURCE-EDITIONS.md) |

Read the affected contract, implementation and owning tests rather than treating
this guide as a capability inventory. Current contribution requirements belong
to their owning issues, including #1074, #1124 and #1135; completion of one
native path does not certify broader browser behavior.

## Product Boundary

Preserve these distinctions when extending the owner:

- Node-document, ordinary parent, shadow/Template host, tree root and storage
  ownership are separate relations. Follow actual source hooks for insertion,
  replacement, adoption, clone/import and destruction; preserve exact generations.
- Consume HTML's exact strings, names, attributes, initialization requirements and
  interpreter decisions. Keep XML PI target grammar distinct from relaxed DOM
  names. Do not copy family inventories, grammar or semantic query controllers.
- Native creation, clone, adoption, captured import and parsed installation have
  distinct causes. Preserve actual history and source-qualified state rather
  than reconstructing it from final attributes or replaying fresh initialization.
- Batches remain atomic. Source phases retain their original continuation,
  receipts, mandatory tails and enclosing affine dispatch guards across refusal.
  Only the top Source root executes; ancestors constrain shared work/storage.
- Admit actual retained backing, temporary/replacement/checkpoint overlap,
  identity/revision headroom and output before publication. Preserve cumulative
  work, cancellation, no-op semantics and complete domain change coverage.
- Capture/Slot supplementation closes actual associations and keeps unknown
  facts unavailable. Ordinary, stored, fresh flat and shadow relations remain
  distinct; an added association does not certify an unrelated tree complete.
- Historical identities/observations do not grant execution. Retirement,
  detach/adoption, hosted inactivity and service-owner loss have distinct
  lifecycle effects; never follow reused slots or fabricate cleanup/delivery.
- Preserve separate authored/intrinsic values and external facts. CSS owns its
  grammar/color policy; CSSOM owns sheet/rule identities. DOM structural state
  supplies no rendering resources, eligibility, readiness or platform success.
- Fullscreen shares canonical raw/pending top-layer state. Preserve source order,
  exact adoption-closure checks, recorded event ownership and the same admitted
  Runtime handoff. Park the invocation, not an old Draft/global Source lock;
  preserve enclosing synchronous tails. Pending orientation is not completed exit.
- Range maintenance, designMode, focus/autofocus, controls, scripts and native
  family hooks use their original owners and qualified producers. Unsupported
  execution/resource/parser/registry facts remain precise requirements, never
  defaults, fake receipts or compatibility wrappers.

Read only the affected topic linked from the README, alongside its owning tests,
for detailed semantics, limits and failure contracts. Generated API audits remain
root-owned.

## Command Inventory

Select the affected integration target or private owning filter from `tests/`
and the changed module; select composed Runtime tests when its adapter is affected.
The package-wide commands below are available when that scope is warranted:

```sh
cargo test --offline --locked -j 1 -p surgeist-dom
cargo clippy --offline --locked -j 1 -p surgeist-dom --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-dom -- --check
```
