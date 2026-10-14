# HTML values crate guide

Apply the root repository guide and focused PISCT skills. HTML owns language
values, local rules, native builders, controlled construction and immutable
results. [Parser](../surgeist-html-parser/AGENTS.md) and
[semantics](../surgeist-html-semantics/AGENTS.md) consume its public root and
bounded front doors directly. The [selected edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
governs behavior. Preserve exact UTF-16 and private generic storage invariants.

Through bounded `pisct run`, from repository root:

```sh
cargo test --offline --locked -j 1 -p surgeist-html
cargo clippy --offline --locked -j 1 -p surgeist-html --all-targets -- -F unsafe-code -D warnings
python3 crates/surgeist-html/tools/generate_vocabulary.py --check
cargo fmt --all -- --check
```

Vocabulary generation stays here. Parser character references and semantic bidi
data/generators stay with their actual owners. Root owns generated API audits.
