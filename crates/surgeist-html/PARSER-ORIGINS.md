# Parser source origins

The private tokenizer and bounded tree-insertion kernel adapt html5ever 0.40.1 at
[`d7232d746d3112f08926d169591576bd12fe2fdf`](https://github.com/servo/html5ever/tree/d7232d746d3112f08926d169591576bd12fe2fdf).
The behavioral authority is the separate
[pinned HTML edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline),
`877d6146487f15b15bc8dd0e3c625063a37b456a`. The implementation revision records
provenance, not a different HTML dialect. This machinery currently compiles only
for tests. The tree kernel implements adjusted insertion locations, element/text
insertion and stack/scope operations over actual controlled native storage.
The ordinary-document prologue implements Initial and BeforeHtml and hands off
an owned BeforeHead continuation. Complete insertion-mode dispatch and public
parse execution remain unfinished.

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
| `html5ever/src/tree_builder/mod.rs` | `d69954162f05710cb62fb660666956ec5daa0878e26ba5df9f25db56d3284a36` | bounded insertion-location and stack/scope adaptation in `src/parser/tree_builder/mod.rs` |
| `html5ever/src/tree_builder/tag_sets.rs` | `5f1e1adffe46f4af0fe063d5847cf1e0fcc03312ca2e3b0062945a341b836716` | selected scope-list adaptation in `src/parser/tree_builder/tag_sets.rs` |
| `html5ever/src/tree_builder/rules.rs` | `877f811266d9d1098f07892473e583b8218d01c0895c8c2d08e40acaec98aa6d` | Initial/BeforeHtml adaptation in `src/parser/tree_builder/prologue.rs` |
| `html5ever/src/tree_builder/data.rs` | `256c72e7d368b0be3d1f473c7b36907b6f6b602aff7d1441b9be109fea3375b1` | doctype tables and classifier adaptation in `src/parser/tree_builder/doctype.rs` |

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

The tree-insertion kernel's separate initial adapted checkpoint is recorded
below. The earlier tokenizer incorporation hashes remain historical.

| Incorporated file | Initial adapted SHA-256 |
| --- | --- |
| `src/parser/tree_builder/mod.rs` | `139d6ce69a6f47e179969cbc4ca99e25df44126498eeac7a375bc11d29eececb` |
| `src/parser/tree_builder/tag_sets.rs` | `1d50c10ad6e4b3f8a6a086e47bb65226fa4a6d76c33dc3337aafe49db6d7a76f` |

The prologue adds a separate initial adapted checkpoint; earlier hashes above
remain historical.

| Incorporated file | Initial adapted SHA-256 |
| --- | --- |
| `src/parser/tree_builder/prologue.rs` | `91a99f0291d311221aa9bf3aab51c0a5c5a998eb24a2e7d87fa676a8589dd5d1` |
| `src/parser/tree_builder/doctype.rs` | `ef2f9ddfa625128fde2910f1a74ac25b2dda64534e346fe24385f0f88ad1ac64` |

The accompanying tokenizer and tree-builder tests are Surgeist-authored
verification, not imported upstream test files. The input, buffer, controller,
managed-payload factories and native arena providers are Surgeist-owned.

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

The tree adaptation implements HTML §13.2.6.1 adjusted insertion, including
foster locations, fragment-root substitution and the source's three ignored
element-placement guards. It routes newly created, unpatched templates through
their actual contents association; imported content-patching state remains
explicitly unavailable. Scope predicates use exact namespaces and the selected
HTML edition's lists, including MathML `annotation-xml`. A controller-owned
open-elements vector admits actual stack storage and visits. Managed factories
consume exact tokenizer payloads into the existing native arena; aliases and
foreign ownership cannot certify a transfer. Terminal cleanup disposes retained
fields and physical capacity even when the final active visit latches a stop.

The prologue adapts `rules.rs` lines 101–144 and the original `mod.rs`
doctype interception and root creation. `data.rs` lines 14–89 supply the
doctypes tables; its classifier is replaced by charged exact UTF-16 matching.
The selected HTML edition governs the restricted non-error doctype syntax,
Silmaril quirks prefix, and HTML 4.01 empty-or-missing system identifier behavior.
Comments, processing instructions and doctypes transfer actual lexical payloads
into native storage. Ignored leading whitespace retains exact source origins,
and the BeforeHead continuation retains any reprocessed token. Foreign
controller payloads are rejected before consuming input or changing tree state.

The fresh ordinary-document constructor establishes non-srcdoc initialization
and parser-cannot-change-mode as false. Hosted, srcdoc, mode-fixed and captured
document execution are unsupported; source labels do not select these modes.
This prologue is a private continuation, not a completed parse result.

Remaining insertion modes, formatting reconstruction/adoption, foreign dispatch,
context effects and report publication remain required engine work. The driver, encoding implementation, serializer,
XML parser and RcDom are not incorporated here. Research prototypes and their
measurements do not certify this adapted implementation or supply a public
parsing contract.

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
pisct run --timeout-seconds 180 -- cargo test --offline --locked -j 1 -p surgeist-html --lib parser::tree_builder::tests
pisct run --timeout-seconds 180 -- cargo test --offline --locked -j 1 -p surgeist-html --lib construction::parser_session::tests
pisct run --timeout-seconds 180 -- cargo clippy --offline --locked -j 1 -p surgeist-html --all-targets -- -F unsafe-code -D warnings
```

These checks exercise the private tokenizer, bounded tree kernel and providers. Full parser
conformance, tree effects, diagnostics/report publication and complete resource
accounting remain obligations of the enclosing parser contribution.
