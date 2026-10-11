# Parser source origins

The private tokenizer adapts html5ever 0.40.1 at
[`d7232d746d3112f08926d169591576bd12fe2fdf`](https://github.com/servo/html5ever/tree/d7232d746d3112f08926d169591576bd12fe2fdf).
The behavioral authority is the separate
[pinned HTML edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline),
`877d6146487f15b15bc8dd0e3c625063a37b456a`. The implementation revision records
provenance, not a different HTML dialect. The tokenizer currently compiles only
for tests; tree construction and public parse execution are not supplied by this
source ingestion.

## Original source and incorporated files

Paths in the first column are relative to that html5ever revision. Destinations
are relative to this crate. Each SHA-256 identifies a complete original file,
before adaptation.

| Original source | Original SHA-256 | Incorporated destination |
| --- | --- | --- |
| `html5ever/src/tokenizer/mod.rs` | `387fef55aea53cf08c2b45192ab8f9f211e993e9dc76ec5031422777bf06773f` | `src/parser/tokenizer/mod.rs`, `src/parser/tokenizer/state.rs` |
| `html5ever/src/tokenizer/states.rs` | `35ea888082a92b77a428af263224169b05ff5740bd519efdace90e44858a0b7d` | `src/parser/tokenizer/states.rs` |
| `html5ever/src/tokenizer/interface.rs` | `cdab80709a6e27663b9a349a607cf85da93fcc9ee3b3a54cf97ee635a9ae7cf4` | `src/parser/tokenizer/token.rs` |
| `html5ever/src/tokenizer/char_ref/mod.rs` | `e753058cb7ab64c63bb13e2156c583bdc60a8468efe6b9c64b8768c6effadf8f` | `src/parser/tokenizer/char_ref.rs` |

The following hashes record the initial adapted source checkpoint on
2026-10-11. Later implementation changes remain discoverable in Git; these are
historical incorporation hashes, not a claim that subsequent source is unchanged.

| Incorporated file | Initial adapted SHA-256 |
| --- | --- |
| `src/parser/tokenizer/mod.rs` | `5c6bd61d498f81d364a33b2782c9a36a5773e26f84eff09c8a1f0c87cd9de9ad` |
| `src/parser/tokenizer/state.rs` | `402151712f4abe6695a78450cf6899b377193b05cd5d4c62807f346738849c17` |
| `src/parser/tokenizer/states.rs` | `8162c621fd7db4d1e6bafcc29e155fd9779a3820122bda96c60aeaa8f4ea5837` |
| `src/parser/tokenizer/token.rs` | `1383aca3882c68a3a69c727acbf24b224f2bd7437e99edaaf135881e701a3a03` |
| `src/parser/tokenizer/char_ref.rs` | `9a419843984560258ebd57aff9f1a20199f7bb3e3892836983d3423f8690cbb4` |

The accompanying `src/parser/tokenizer/tests.rs` is Surgeist-authored
verification, not an imported upstream test file. The existing input, buffer,
controller and arena modules are Surgeist-owned providers.

## Modifications

The adaptation replaces streaming Tendril buffers, atoms, token sinks and SIMD
fast paths with safe indexed code-point processing over the exact `HtmlSource`.
Tokens retain exact UTF-16 payloads, source ranges and origin records. Private
fallible operations use the crate’s single parser resource controller; they
admit provider reads, copy work, record writes, storage and indivisible fixed
layouts before the corresponding operation. Owner-qualified cleanup remains
available after a terminal stop. Original parser algorithms are reorganized
between dispatch, state, token and reference modules without importing upstream
packaging or public provider interfaces.

Behavior is reconciled with
[HTML §13.2.5](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md),
including its five processing-instruction states. The adaptation distinguishes
literal surrogate input from numeric-reference replacement, retains exact
preprocessing ranges, and uses structured diagnostics and provenance rather
than display strings as semantic data. CDATA admission and raw-text state
selection are explicit private inputs for the future tree builder.

Character-reference matching consumes the existing independently generated
[`character_references.rs`](src/parser/character_references.rs). Its required generation
input is the complete named-reference and C1 tables in the tracked pinned HTML
syntax capture. The generator’s optional original-table comparison uses
html5ever only as a witness; no upstream atom generator, generated PHF table,
Tendril implementation or external dependency is incorporated here.

The upstream tree builder, driver, encoding implementation, serializer, XML
parser and RcDom are not part of this tokenizer checkpoint. Research prototypes
and their measurements establish suitability evidence; they do not certify
this adapted implementation or provide a public parsing contract.

## Attribution and verification

Original file notices remain intact. The exact upstream
[copyright](../../licenses/html5ever/COPYRIGHT),
[authors](../../licenses/html5ever/AUTHORS),
[MIT](../../licenses/html5ever/LICENSE-MIT) and
[Apache-2.0](../../licenses/html5ever/LICENSE-APACHE) files accompany this source;
the original dual-license choice is preserved. See the
[crate notice](NOTICE.md#adapted-parser-source) and
[upstream attribution](../../licenses/html5ever/NOTICE.md#incorporated-tokenizer-source).

Focused verification uses the repository’s bounded command runner:

```sh
pisct run --timeout-seconds 180 -- cargo test --offline --locked -j 1 -p surgeist-html --lib parser::tokenizer::tests
pisct run --timeout-seconds 180 -- cargo test --offline --locked -j 1 -p surgeist-html --lib parser::input::tests
pisct run --timeout-seconds 180 -- cargo clippy --offline --locked -j 1 -p surgeist-html --all-targets -- -F unsafe-code -D warnings
```

These checks exercise the private tokenizer and its providers. Full parser
conformance, tree effects, diagnostics/report publication and complete resource
accounting remain obligations of the enclosing parser contribution.
