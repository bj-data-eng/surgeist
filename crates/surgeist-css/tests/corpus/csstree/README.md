# Pinned CSSTree corpus

This directory adopts the neutral CSS fixture corpus generated from CSSTree
commit `88e3d965c0b1628642a30a841745b410d6835052`. The pinned
`fixtures/ast` tree is `bfadc7a7a8d93dce59a27fa7df3bb0f6f6a623d8` and contains
935 cases across 74 fixture files (721 upstream-parsed and 214
upstream-rejected cases).

CSSTree is distributed under the MIT License. `LICENSE` is the unchanged
license notice copied from that pinned checkout.

The source sidecar, imported fixtures, neutral expectations, generation report,
and manifest were produced by the published `surgeist-generator` candidate
`83a216880884a5a364258ffaaeaf93d228c0bc53`. They are an indivisible generated
artifact set and must not be edited by hand.

Future corpus maintenance uses the local workspace `surgeist-generator` CLI.
The historical generator revision above remains the provenance of the committed
set. In a prepared disposable owner root, use the corpus manifest's pinned
CSSTree checkout; then adopt only the generated manifest, source, expectations,
and report with the unchanged license.

From the Surgeist product root, set `CORPUS_OWNER`, `CORPUS_ROOT`, and
`CSSTREE_SOURCE` to those existing absolute paths; `CORPUS_ROOT` must be contained
by `CORPUS_OWNER`. Run the selected maintenance operation:

```sh
cargo run --offline --locked -p surgeist-generator --features css-corpus --bin surgeist-css-generate -- --owner-root "$CORPUS_OWNER" --corpus-root "$CORPUS_ROOT" import-csstree --source-root "$CSSTREE_SOURCE"
cargo run --offline --locked -p surgeist-generator --features css-corpus --bin surgeist-css-generate -- --owner-root "$CORPUS_OWNER" --corpus-root "$CORPUS_ROOT" generate
cargo run --offline --locked -p surgeist-generator --features css-corpus --bin surgeist-css-generate -- --owner-root "$CORPUS_OWNER" --corpus-root "$CORPUS_ROOT" check-corpus
```

Mutation requires Apple-Silicon macOS and the maintenance scope selected by the
caller. Ordinary `surgeist-css` tests consume committed artifacts and do not
invoke the generator or access an upstream checkout.

The CSS-owned `tests/csstree/oracle.json` is reviewed with this neutral artifact
set. Every oracle record repeats its generation-report expectation digest and
exact source path, expectation path, context, options, and input. A provider
maintenance change that alters any of those bindings must update the oracle in
the same reviewed change; the ordinary offline tests validate the complete
935-record contract without running the parser or generator.

The CSS-owned adapter registry sends ordinary `selector`, `selectorList`, and
`mediaQuery` fixtures directly to their matching raw fragment parsers. Inputs
and diagnostic coordinates remain unchanged. Selector probes provide only the
named namespace binding `ns` to `surgeist-corpus-probe`, without injecting a
namespace rule into source. Functional-pseudo and Nth fixture inputs already
contain complete selectors; their adapter names do not select argument grammars.
Single-selector and single-query probes require complete singular input. The
selector-list extractor counts retained members, and the query extractor counts
one retained query, including malformed-query recovery syntax.

The basic selector expectations use that same namespace context: `a` and `xlink`
are undeclared, so their four attribute-selector cases are rejected. The presence
selector `[b i]` is also rejected because a modifier requires a matcher and value
([Selectors 4 grammar](https://www.w3.org/TR/2026/WD-selectors-4-20260122/#grammar),
[attribute namespaces](https://www.w3.org/TR/2026/WD-selectors-4-20260122/#attrnmsp)).
Rejected raw selectors carry `RejectInput` diagnostics; the basic-selector corpus
checks separately preserve the 65 clean cases and the 22 rejected cases.

The named-pseudo fixtures retain the defined `before`, `after`, `first-line`,
and `first-letter` pseudo-elements, including their required legacy single-colon
spellings. Undefined `test`, `test-test`, and `unknown()` constructs are rejected
([pseudo-element syntax](https://www.w3.org/TR/2026/WD-selectors-4-20260122/#pseudo-element-syntax),
[invalid selectors](https://www.w3.org/TR/2026/WD-selectors-4-20260122/#invalid)).
Their two fixture files contain six clean and nine rejected inputs. Rejections
use the raw `RejectInput` action; functional argument contents cannot define an
otherwise unknown pseudo name.

The Nth fixtures preserve token-level `An+B` grammar. In particular, `3 n`,
`+ 2n`, and `+ 2` are invalid; permitted whitespace around a later offset does
not join a coefficient, sign, or integer across tokens
([Syntax 3](https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#anb-syntax)).
The corpus checks verify the coefficients, pseudo-class kind, and authored
`of` filter for 25 clean inputs, and raw rejection for 39 invalid inputs.

Unicode-range values use the raw font-face descriptor-value parser with typed
descriptor context. No descriptor name, neighboring descriptors, or rule wrapper
is inserted. Ten original inputs retain independently checked numeric ranges;
twenty are rejected. The incomplete values `U+` and `u` report original EOF with
`RecoveryEndsAt`; other rejected values identify an original token. In particular,
`u+?` is the valid range 0–15 and the original representation `u+12e-130` denotes
302–304 ([Syntax 3](https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#urange)).

Declaration fixtures use the singular raw declaration parser, retaining at most
one declaration and preserving the original source. Their upstream AST options
remain recorded as provenance; they do not bypass property grammar. The four
files contain 22 clean custom declarations and 55 rejected inputs. Unknown names,
punctuation hacks, and proprietary IE filter functions are rejected under the
selected grammar. Bare `--` is not a custom property name, and unmatched nested
closers remain invalid even when an enclosing block reaches EOF
([Variables 1](https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#defining-variables),
[Syntax 3](https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#any-value),
[Filter Effects 1](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/#FilterFunctions)).
Raw rejections use `RejectInput`; declaration-list fixtures retain their separate
list parser and recovery contract.

Full `atrule` fixtures use the raw single-rule parser with the same immutable
namespace binding. The CSS-owned classes reconcile 129 determinate originals:
55 require clean outer retention, 66 require one whole-input `RejectInput`, and
eight retain their enclosing rule with local recovery. The independently authored
[`at-rule-reconciliation.json`](../../csstree/at-rule-reconciliation.json) records
these expectations and their sources; exact diagnostic coordinates characterize
the public API rather than numerical requirements imposed by CSS specifications.
The original IDs, inputs, options, source hashes, and active dispositions remain
unchanged. Unsupported upstream options and vendor syntax keep their explicit
policies; superseded `@nest` and unselected Transitions2 `@starting-style` remain
outside the selected grammar.

The six whole-rule rejections diagnosed at the original EOF use
`recovery_ends_at`: their error offset equals the payload end, while the
recovery span covers the nonempty original rule. The adapter-observation test
checks this relation together with expected-class admission.

The complex container-query original retains its unknown enclosed operands
within the authored Boolean structure. The Fonts4 character-variant singleton
original independently requires a retained `@font-feature-values` outer rule,
while conflicting pinned descriptor prose leaves its cleanliness and diagnostics
unresolved. Its existing class is therefore left pending, not endorsed as an
outside-profile exclusion. These class corrections do not establish a complete
130-case baseline or refresh the separately persisted oracle.

The adapterless Combinator fixtures retain their explicit panic-freedom policy.
The mixed `atrulePrelude` fixture resolves its adapter from validated options:
`atrule: media` selects the raw media-query-list parser, while the unnamed generic
prelude retains only a custom-property containment panic-freedom probe. Its text
never supplies an implicit media grammar. Expected-class validation, observation,
and oracle validation use this same resolution; oracle options cannot select a
route different from the matched neutral case.
Changes to entry points, extractors, or recovery actions require corresponding
review of CSS-owned expected classes and oracle records. Expectations must be
established independently of parser observations; an old oracle is not evidence
that a synthetic wrapper preserved the original fragment grammar.
