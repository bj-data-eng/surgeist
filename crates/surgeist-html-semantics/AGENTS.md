# surgeist-html-semantics Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
[Cargo.toml](Cargo.toml), [src/lib.rs](src/lib.rs) and [README.md](README.md)
own package facts, the public front door and capability/verification guidance.

This crate owns capture identity/facts, construction-to-capture import, external observations and bounded semantic query sessions.
Use the [HTML value owner](../surgeist-html/AGENTS.md) public contracts directly;
never expose private storage or duplicate its local algorithms. The
[selected edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
defines behavior; retained implementation provenance is separate.

## Command Inventory

Select affected checks under the root process/resource policy, from repository root:

```sh
cargo test --offline --locked -j 1 -p surgeist-html-semantics
cargo clippy --offline --locked -j 1 -p surgeist-html-semantics --all-targets -- -F unsafe-code -D warnings
python3 crates/surgeist-html-semantics/tools/generate_bidi_class.py --check
cargo fmt --all -- --check
```

Preserve exact source/fact identities, first reached failure, actual retained
allocations and explicit cleanup. See [README.md](README.md) for capability limits and attribution.

Shadow-root capture takes its actual Document independently of the associated
host. Internal DOM replacement/adoption can change the root's node document
without changing that association; never infer or equalize their documents.
Preserve reciprocal host/root facts, source-qualified construction observations,
ordinary parent/child document consistency, ownership checks and atomic refusal.
This is capture of existing state, not a live attachment or public adoption API.
