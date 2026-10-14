# surgeist-html-parser crate guide

Apply the root repository guide and focused PISCT skills. This crate owns
decoded sources, parse requests, ingestion/recovery, effects, evidence, original controller and terminal ownership.
Use the [HTML value owner](../surgeist-html/AGENTS.md) public contracts directly;
never expose private storage or duplicate its local algorithms. The
[selected edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
defines behavior; retained implementation provenance is separate.

Through bounded `pisct run`, from repository root:

```sh
cargo test --offline --locked -j 1 -p surgeist-html-parser
cargo clippy --offline --locked -j 1 -p surgeist-html-parser --all-targets -- -F unsafe-code -D warnings
python3 crates/surgeist-html-parser/tools/generate_character_references.py --check
cargo fmt --all -- --check
```

Preserve exact source/fact identities, first reached failure, actual retained
allocations and explicit cleanup. Root owns API audit generation. See the
[crate README](README.md) for supported capability limits and attribution.
