# HTML crate guide

Apply the root repository guide and focused PISCT skills. This crate owns HTML
language vocabulary, typed authoring, immutable construction results, parsing
and intrinsic semantic interpretation. The selected edition and implementation
basis are recorded in [source editions](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline).
The WHATWG edition governs disagreement with the html5ever implementation basis.

The public front door is `src/lib.rs`. Shared string/name/attribute records
preserve exact UTF-16 without HTML normalization or generic DOM validation.
Parsing applies HTML normalization at the specified stage. Immutable result
identities are local to their result; they are never canonical live DOM IDs.
DOM owns its HTML integration. This crate has no downstream production dependency.

Focused checks, through bounded `pisct run`, are:

```sh
cargo test --offline --locked -j 1 -p surgeist-html
cargo clippy --offline --locked -j 1 -p surgeist-html --all-targets -- -F unsafe-code -D warnings
cargo fmt --all -- --check
```

Refresh and check the HTML API through the root-owned API generator when the
public surface changes. Select additional parser, semantic and feature checks
from their actual source contracts as those capabilities are implemented.
