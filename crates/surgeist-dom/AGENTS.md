# Canonical DOM crate guide

Apply the root guide and focused PISCT skills. This crate owns canonical live
identity, exclusive mutation, immutable snapshots and change coverage. Consume
HTML-owned string/value contracts directly; HTML language interpretation stays
with its producer. The crate README states the currently bounded seven-kind
surface. Do not fabricate missing HTML/control/template/registry/script state
when extending construction or add compatibility wrappers around retained.

Use the root-selected DOM capture and owning #1074/#1124/#1135 contracts. Node
document membership, source ownerDocument readback, storage ownership and tree
root are distinct. Keep generation/revision exhaustion checked and rejected
batches atomic, including slot allocation and history. Root owns workspace,
facade composition, architecture documentation and generated API artifacts.
PI construction uses the shared bounded XML Name predicates and PI
pseudo-attribute cursor; do not replace its target grammar with relaxed element
validation or HTML authoring restrictions. Doctype names use DOM's relaxed rule.
Native Document metadata follows the selected document defaults (capture
lines 2110–2112), including a fresh opaque origin and known-null registry.
Stored content type, encoding name and URL count toward string admission;
doctype edits do not infer compatibility mode. Keep metadata and origin
identity within the same staged publication and immutable snapshot as the tree.
Generic XML Element creation preserves exact no-namespace local names. Consume
shared QualifiedName/Attribute/OrderedAttributes rather than adding parallel
records. DOM's relaxed name/namespace checks differ from XML PI target grammar.
Attribute NS updates retain existing prefix/order. Insertion adopts transferred
fragment children only; explicit adoption and replacement include the supplied
fragment root. Direct insertion uses an empty hierarchy exclusion list;
replacement excludes exactly its child. No HTML/SVG/MathML constructor exists
until the consuming adapter performs the applicable native state/hooks.
Generic import reads only its supplied immutable capture, never a same-number
live slot. It allocates fresh target identities; caller tickets map separately
from private imported descendant allocations. Clone trusted canonical name and
attribute fields through actual generic initialization/append steps. PI clone
preserves current data without new-construction `?>` validation. Keep traversal
iterative and all copied nodes/strings admitted with the rest of the batch.

From repository root through bounded `pisct run`:

```sh
cargo test --offline --locked -j 1 -p surgeist-dom
cargo clippy --offline --locked -j 1 -p surgeist-dom --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-dom -- --check
```
