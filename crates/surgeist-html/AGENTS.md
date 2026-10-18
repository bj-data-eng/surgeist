# surgeist-html Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
[Cargo.toml](Cargo.toml), [src/lib.rs](src/lib.rs) and [README.md](README.md)
own package facts, the public front door and capability/verification guidance.

HTML owns language values, local rules, native builders, controlled construction and immutable
results. [Parser](../surgeist-html-parser/AGENTS.md) and
[semantics](../surgeist-html-semantics/AGENTS.md) consume its public root and
bounded front doors directly. The [selected edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
governs behavior. Preserve exact UTF-16 and private generic storage invariants.

The owned `HtmlNode` Document design-mode readback certifies the original
constructor's false state under this owner's no-mutator domain. A raw node-kind
value or captured external Document supplies no such qualification. If controlled
construction gains a design-mode transition, update the readback to retain its
actual current state rather than continuing to return an initial value.

## Command Inventory

Select affected checks under the root process/resource policy, from repository root:

```sh
cargo test --offline --locked -j 1 -p surgeist-html
cargo clippy --offline --locked -j 1 -p surgeist-html --all-targets -- -F unsafe-code -D warnings
python3 crates/surgeist-html/tools/generate_vocabulary.py --check
cargo fmt --all -- --check
```

Vocabulary generation stays here; parser character references and semantic bidi
data/generators stay with their actual owners.

Slot assignment owns only the HTML manual-intent transition. Consumers retain
actual IDs/ordered sets and execute the final target-root DOM assignment step;
never refresh former roots to force stored assignment and current query results
into one inverse. Raw duplicates still execute transfer effects before ordered
set append deduplication. Fresh-slot evidence does not certify current emptiness.
