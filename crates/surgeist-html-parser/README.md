# surgeist-html-parser

Synchronous decoded-Unicode HTML parsing, recovery and source-owning immutable
reports. Document and qualified contextual-fragment requests produce the shared
[HTML construction values](../surgeist-html/README.md), preserving source identity,
UTF-8 byte or UTF-16 unit coordinates, structured diagnostics and provenance.

The [detailed parsing guide](../../docs/reference/html-operations.md#decoded-unicode-parsing)
records supported profiles, explicit external facts, bounded ownership and
terminal cleanup. The [coverage matrix](docs/conformance-coverage.md) connects
selected requirements to tests; it does not certify full browser conformance.
The [selected HTML edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
governs behavior; [PARSER-ORIGINS.md](PARSER-ORIGINS.md) records html5ever
implementation provenance separately.

The parser depends directly on HTML values and
[semantic captures](../surgeist-html-semantics/README.md). Its input-color adapter
services CSS grammar and Color numerical actions through the original parser
controller. It owns source validation, recovery, construction effects, work and
allocation tickets, cancellation, first failure and stopped-owner cleanup.
HTML owns the actual shared immutable tree storage and its generic invariants. Its
[controlled text append](../../docs/reference/html-crate-contracts.md#controlled-text-append)
reuses unique aggregate backing while retaining immutable history leaves; the
parser indexes current text tickets and textarea metadata with actual admitted
capacity and cleanup.

The document profile is fresh ordinary Disabled execution. Fragments retain
qualified Inert/Disabled context facts. JavaScript execution, resource fetching,
live shadow attachment and browser events remain outside these profiles.
Unsupported execution or required construction facts return typed failures. Failure retains
the original request and live ownership until explicit cleanup; no successful
partial report is returned.

Completed reports retain each ordinary element's no-definition custom-element
state and independent creation-time `is` value through the public metadata
cursor. Formatting reconstruction and cloning preserve their actual creation
causes; later HTML/body attribute merges do not rewrite that saved value.

Ordinary offline reports also lend subject-qualified closed common state; nonce
is retained independently. A/Area records preserve the actual source-time URL
snapshot and historical parse Document. Absent href, observed URL parse failure
and unavailable context are separate states. Immutable request observation banks
are qualified against the actual Document's base, encoding and full external
revision at each reached operation; later Base elements cannot rewrite earlier
cached URLs. Template-owner facts bind only the actual separately created inert
Document and must agree with its known default context.

Standalone Img records retain the actual fresh request, dimension source and
latest inactive update wait with its generation and flags. Source/Picture hooks
and real clones use the original source effect owner. Disabled image support
cannot skip the source-prior wait for a fully active Document. These are retained
algorithm states, without resource fetching or a live parser continuation.
Style records preserve actual parser creation, creating parser Document and
completed stack-pop history; general clones have fresh creation state. Their
CSSOM association disposition is explicitly pending. Construction completion
therefore does not assert that every semantic observation or downstream
association is available.

Each source-created Document separately retains actual Meta pragma-language,
canonical policy-container referrer policy and declarative-refresh history.
The closed offline profile chooses Do nothing after a successful refresh; it
performs no navigation or timer work. Shared encoding-URL observation banks serve
A/Area and Meta with exact source-time base, encoding and independent full revision.
Missing URL facts leave refresh explicitly unavailable, and an unknown prior flag
cannot be treated as false. Reached CSSOM preferred-set, CSP, theme CSS/media and
color-scheme requirements remain distinct unavailable dispositions; structural
parse success does not certify their execution. Language and referrer updates
continue where their own inputs are qualified. `document_meta()` lends these
actual Document-qualified values and the first missing operation without replay.
A reached unavailable CSP invalidates later required policy-dependent URL/Base
operations while preserving earlier cached observations. `base_state()` separates
unfrozen, frozen and unavailable current state, retaining a genuine previous
frozen value as history. An installer must refuse required unavailable state;
it cannot substitute target defaults or null.

`retained_construction_bytes()` is the original controller's inclusive retained
construction payload/capacity receipt. It counts the owned payload Vec's inline
snapshot header once and adds the provider's checked selected backing; declared
alias occurrences remain conservatively charged. Opaque standard Arc headers and
allocator bookkeeping retain the original controller's provider limitation.
Original source backing, report header and separately lent root/inert BaseFacts
provider graphs are outside that receipt. The original owner keeps diagnostic
family subtotals privately; they are not extra consumer admission. Unused request
execution banks, including their spare capacity, are released before successful
publication. Every failure preserves the actual original request bank and its
ownership; resource cleanup does not reconstruct it.

From the repository root, run focused checks through bounded `pisct run`:

```sh
cargo test --offline --locked -j 1 -p surgeist-html-parser
cargo clippy --offline --locked -j 1 -p surgeist-html-parser --all-targets -- -F unsafe-code -D warnings
python3 crates/surgeist-html-parser/tools/generate_character_references.py --check
```

The existing [HTML fuzz package](../surgeist-html/fuzz/README.md) consumes these
public requests directly. [Attribution](NOTICE.md) preserves parser source and
fixture notices.
