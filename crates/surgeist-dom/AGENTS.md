# Canonical DOM crate guide

Apply the root guide and focused PISCT skills. This crate owns canonical live
identity, exclusive mutation, immutable snapshots and change coverage. Consume
HTML-owned string/value contracts directly; HTML language interpretation stays
with its producer. The crate README states the currently bounded four-kind
surface. Do not fabricate missing HTML/control/template/registry/script state
when extending construction or add compatibility wrappers around retained.

Use the root-selected DOM capture and owning #1074/#1124/#1135 contracts. Node
document membership, source ownerDocument readback, storage ownership and tree
root are distinct. Keep generation/revision exhaustion checked and rejected
batches atomic, including slot allocation and history. Root owns workspace,
facade composition, architecture documentation and generated API artifacts.

From repository root through bounded `pisct run`:

```sh
cargo test --offline --locked -j 1 -p surgeist-dom
cargo clippy --offline --locked -j 1 -p surgeist-dom --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-dom -- --check
```
