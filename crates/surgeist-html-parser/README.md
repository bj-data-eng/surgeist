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

Finalization uses HTML's bounded value pools to share eligible immutable
attribute banks, element-name components and Text data within each completed
report. Sharing starts after mutable construction and alias checks finish;
selected capacities and original/replacement overlap are admitted before output
publication. Source locations, node identities and construction history remain
per occurrence. Mutable parsing retains its original peak-storage requirements.

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

Ordinary offline and completed inert/disabled contextual reports lend
subject-qualified common state for actual Null lookup registries. Its
`HtmlCommonState<Infallible>` retains the profile's actual null previous focus;
nonce is retained independently. A/Area records preserve the actual source-time URL
snapshot and historical parse Document. Absent href, observed URL parse failure
and unavailable context are separate states. Immutable request observation banks
are qualified against the actual Document's base, encoding and full external
revision at each reached operation; later Base elements cannot rewrite earlier
cached URLs. Template-owner facts bind only the actual separately created inert
Document and must agree with its known default context.

Contextual requests accept those same execution-only EncodingUrl banks plus
sealed `DocumentEncodingUrlContext` semantic query results for captured
Documents. Selection uses the node's actual Document when each href update runs:
a root A/Area can still belong to the auxiliary parser Document before adoption;
a nested element can already belong to the captured target; template contents
use their actual owned or captured inert Document. Separate constructor-qualified
facts supply real auxiliary/inert URL and encoding observations. Adoption does
not replay href parsing. Each selected context is checked against the original
capture/host/Document; URL lookup checks its independently expected full revision,
base and encoding identities. The closed fragment effects do not change captured
Document policy or ordinary Base order; owned reached CSP gaps still refuse
before lookup. No immutable selection certifies a later policy transition.
Successful terminal publication releases these selected contexts and URL execution
banks while retaining only actual snapshots and needed Document history. Failure
returns the original input vectors, capacities and first stop. Separately supplied
whole captured Base banks retain their existing report lifetime and receipt.

Standalone Img records retain the actual fresh request, dimension source and
latest inactive update wait with its generation and flags. Source/Picture hooks
and real clones use the original source effect owner. Disabled image support
cannot skip the source-prior wait for a fully active Document. These are retained
algorithm states, without resource fetching or a live parser continuation.
Contextual updates use the actual node-Document at each invocation: the closed
auxiliary parser Document and source-created inert template Documents are
inactive independently of the captured target. An established absent captured
browsing context also proves inactivity; Present or Unavailable alone leaves
activity unavailable. Scripting mode and image-support policy prove neither.
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

Completed contextual reports also lend `fragment_parser_document()`: the real
auxiliary §13.4 Document, its actual mode/completion, original context/source and
once-created opaque origin. Its closed constructor establishes HTML class with
`application/xml`, UTF-8, `about:blank`, known-null about-base, no srcdoc, Null
Document registry and false Document allow-DSD, independently from parser DSD
permission. This parser Document has no browsing context or navigable and disabled
scripting; output-node scripting still follows the captured context. The actual
ordinary html child and per-Document Meta history remain in the same report arena.
The descriptor does not grant source-era platform, host, resource or current
policy capabilities. Cloning its opaque origin preserves identity across references
and imports; independently parsed Documents get different origins even for the
same supplied source/capture. Ordinary reports and inert/captured Documents cannot
issue this distinct constructor descriptor.

Dialog metadata lends the original controller's current return value separately
from content attributes and its closed-service or first-unfulfilled-operation
disposition. The original creation, clone, exact open-change and tree hooks
consume HTML's shared Dialog cursor. Document interaction metadata belongs to
each actual owned Document; it records the constructor and admitted transition
history rather than inferring empty lists from final markup. Captured Documents
are never certified to have empty historical lists. A newly created Dialog's
proved nonmembership can support its cleanup without certifying unrelated
members. Reached missing activity or services retain their actual subject,
Document, action and cause; no successful host operation is invented. These
fixed metadata reads do not replay processing or grant hosted Dialog services.

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
