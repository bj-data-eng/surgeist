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

The [committed oracle](../../csstree/oracle.json) binds the reconciled CSS-owned registry: 820 cases
have full public-parser observations and 115 retain explicit panic-freedom-only
policies. Across all 935 retained records, outcomes are 354 clean, 337 strict
rejections, 91 recoveries, and 153 explicit unsupported dispositions. The
unsupported count includes all 115 panic-freedom-only records and 38 active
records; it is not an additional set of cases. The
[census contract](../../css/corpora/csstree/csstree_oracle.rs) independently asserts this partition.
These corpus results do not establish complete grammar coverage of the selected
standards profile.

The oracle is characterization, not a correctness oracle. It is captured from
the current public parser, so it detects behavior changes but cannot show that
the captured behavior is right; a re-captured oracle is never evidence for a
behavior change. The I01 observables fixture
(`tests/fixtures/i01-c01-observables.tsv`) is characterization on the same
terms. Independent evidence comes from the CSS-owned expected classes and
reconciliation records, and from the
[canonical comparison](../../css/corpora/csstree/csstree_canonical.rs). That comparison serializes
Surgeist's public parse result for each upstream-parsed case and compares it with
CSSTree's own generator expectation: the case's `canonical_css`, or its input
when that field is absent, as CSSTree's `lib/__tests/generate.js` requires at
the pinned revision. Both sides are tokenized by the crate's CSS Syntax 3
tokenizer dependency (cssparser, the Servo and Firefox tokenizer), so token
boundaries, escapes, bad strings and urls, functions and input preprocessing
follow the specification. One documented normalization then removes comments,
quote style, optional whitespace and optional final semicolons, and the token
sequences are compared. A removed comment still separates the tokens around it,
identifiers and strings compare by decoded value, and whitespace stays
significant before a namespace `|`, between `<` or `>` and `=`, and on only one
side of a `+`, so combinators, declarations, values and rule structure remain
distinct.

Selector fixtures that the registry observes only for panic freedom, such as the
combinator cases, are still projected through the public selector parser; a
rejection there is compared as empty output and needs its own cited exception.

Of the 721 upstream-parsed cases, 468 are compared: 298 match and 170 are
listed in [`canonical_exceptions.rs`](../../css/support/csstree/canonical_exceptions.rs)
as deliberate differences with a cited reason, or as known defects. Each
exception also pins its expected Surgeist form in the normalized rendering, and
the actual serialization must equal that form token by token, so a different
wrong serialization of an excepted case fails. Each pinned form was checked
against its reason, not only captured from the parser. The other
253 have no public projection (92 panic-freedom-only value and prelude cases)
or retain no syntax after a whole-input rejection (161 cases). A case is
excluded as a whole-input rejection only when its expected class records one:
empty retained syntax with a `RejectInput` diagnostic, as a strict rejection
(142 cases) or a fully observed unsupported disposition (19 outside-profile,
vendor or upstream-option cases); the comparison test asserts that split and
recounts it from the raw expected-class registry. The outside-profile cases
include `:matches()`: [Selectors 4 §4.2](../../../../../references/selectors-4--WD-selectors-4-20260122--5d3b3f7e1fcc.md)
lets UAs "additionally implement this obsolete name as a legacy selector
alias" for `:is()`, which permits but does not require it, and the selected
profile does not adopt it. Retaining nothing under any other class fails. A listed case that starts matching fails the comparison, matched exactly
by case ID.

Some matches show only that retained tokens replay faithfully, because Surgeist
keeps those values as tokens rather than typed syntax: custom properties,
`var()`, general-enclosed conditions and `@supports` or `@container` condition
text. They are structural evidence for the surrounding rule, not for a typed
value grammar.

A change to an expected class kind or expected diagnostic (code, recovery
action or payload relation) in `tests/csstree/expected-classes.json` must cite
its specification basis in the same change, in the owning reconciliation record,
test or this guide. Parser output, the oracle and the observables fixture are not
such a basis.

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
([Syntax 3](https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#anb-syntax)).
The corpus checks verify the coefficients, pseudo-class kind, and authored
`of` filter for 25 clean inputs, and raw rejection for 39 invalid inputs.

Unicode-range values use the raw font-face descriptor-value parser with typed
descriptor context. No descriptor name, neighboring descriptors, or rule wrapper
is inserted. Ten original inputs retain independently checked numeric ranges;
twenty are rejected. The incomplete values `U+` and `u` report original EOF with
`RecoveryEndsAt`; other rejected values identify an original token. In particular,
`u+?` is the valid range 0–15 and the original representation `u+12e-130` denotes
302–304 ([Syntax 3](https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-unicode-range-value)).

Declaration fixtures use the singular raw declaration parser, retaining at most
one declaration and preserving the original source. Their upstream AST options
remain recorded as provenance; they do not bypass property grammar. The four
files contain 22 clean custom declarations and 55 rejected inputs. Unknown names,
punctuation hacks, and proprietary IE filter functions are rejected under the
selected grammar. Bare `--` is not a custom property name, and unmatched nested
closers remain invalid even when an enclosing block reaches EOF
([Variables 1](https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#defining-variables),
[Syntax 3](https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#any-value),
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
original uses selected Fonts4 section 6.9.2 and frozen WebKit's authored grammar:
retain the outer rule and all six definitions in each character-variant/styleset
block cleanly. Its
[`policy expectations`](../../csstree/font-singleton-policy-reconciliation.json)
record independently derived typed values, origins, and the clean class. The
contradictory section 6.9.1 prose remains explicit; the historical uncertainty in
`at-rule-reconciliation.json` is preserved. This selected grammar is not an
outside-profile exclusion. Class reconciliation does not itself refresh the
separately persisted oracle.

The ten original strict selector-list inputs use independently specified
[`selector-list expectations`](../../csstree/selector-list-original-reconciliation.json).
Two clean inputs retain exactly one and two members; eight malformed lists are
rejected atomically with `RejectInput`. The missing selector after a comma at original
EOF uses `recovery_ends_at`; the other seven diagnostics intersect the input.
These assertions cover original typed selectors and exact diagnostic coordinates
through the public parser and the real corpus adapter.

The 45 original `:is`, `:where`, `:not`, and `:has` records use the raw
single-selector adapter. Their independently derived
[`logical-selector-reconciliation.json`](../../csstree/logical-selector-reconciliation.json)
contracts distinguish 19 clean, eight recovered, and 18 rejected selectors.
Forgiving lists retain the outer pseudo even when no inner member survives;
unforgiving failures reject the input. Ordered diagnostics preserve repeated
invalid members. The tests bind original identity and source hashes and exercise
the real adapter against these classes. The persisted oracle remains separate.

The 49 raw media-query originals are bound by
[`media-query-reconciliation.json`](../../csstree/media-query-reconciliation.json).
The assertions distinguish unknown authored features, general enclosures, and
recovery to `Never`, including original EOF closure diagnostics. An escaped
unknown dimension unit remains ordinary feature syntax with unknown semantics;
it does not require a vendor-specific parser mode. These expectations exercise
the real adapter and preserve source identities without refreshing the oracle.

The adapterless Combinator fixtures retain their explicit panic-freedom policy
in the registry and the oracle. The canonical comparison still projects them
through the public selector parser, which parses the same single complex
selector as CSSTree's `selector` context.
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

The gated capture validates every observation against the current owner registry
before replacing the oracle. A reviewed registry update may therefore change the
oracle's registry digest. Capture still requires unchanged neutral source,
generation-report and per-record bindings, and rejects registry changes made
during the capture itself. It never repairs a mismatch by editing expectations.
