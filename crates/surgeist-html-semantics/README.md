# surgeist-html-semantics

Qualified immutable captures and bounded intrinsic HTML semantic queries.
Capture builders retain explicit document, registry, host and external facts;
query sessions preserve typed missing-fact results, foreign-ID rejection,
resource admission and sticky first failure.

This crate depends directly on [HTML values](../surgeist-html/README.md). It has
no parser, CSS, Color, DOM or runtime dependency. The
`HtmlConstructionCaptureAdapter` consumes immutable native or parsed HTML
results through the same public views. Callers supply external facts; the
adapter owns traversal, admission, capture construction and result-to-capture
mapping. A parse report enters through its document or fragment value, without
introducing a parser dependency here.

The [detailed semantic guide](../../docs/reference/html-operations.md#captured-custom-element-author-context)
records query families, qualifications and capability limits. External URL/CSP,
resource, layout and host observations remain supplied facts. Queries do not
execute scripts, load resources or mutate a live tree. The
[selected language edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
defines behavior.

`select_document_base` selects the actual first href Base's frozen URL or the
qualified Document fallback, preserving historical frozen record identity.
`HtmlEncodingUrlParseFacts` admits exact encoding-parse observations for one
selected base, opaque encoding and full producer revision. Formation counts
retained vector/string capacities; lookup charges a caller callback before each
comparison unit and preserves its first interruption. `lookup_units_with` borrows
an exact UTF-16 input, including a Meta refresh substring, and returns the bank's
observed outcome without copying the input or creating Element history.
`resolve_hyperlink_with` establishes the genuine A/Area snapshot, retaining only
its raw input, qualification and observed result, including explicit parse failure.
Its mandatory final retention callback receives the checked selected snapshot
footprint before cloning any selected handle: Self, actual observation raw
capacity, Base and observed result. Earlier lookup or footprint failure skips
this callback; its refusal propagates unchanged. No extra lookup work step is
introduced. Callers provide their actual retention/cancellation policy and hold
the admitted occurrence through its own transfer or drop. Ordinary unrestricted
retention uses an explicit accepting callback.
Both use the same qualified lookup. Missing observations remain unavailable
evidence. Adapters bind
these facts to actual Documents and encoding truth; this crate parses no URL.

`HtmlSemanticQuery::DocumentEncodingUrlContext` uses the same bounded session's
first-Base, Document-base and environment selection. A known result is a sealed
owned `HtmlDocumentEncodingUrlContext`: exact captured Document/host, selected
`HtmlDocumentBaseUrl`, opaque encoding and independently expected current full
revision. The first frozen Base may retain an older producing revision. Missing
topology, frozen state, environment or encoding stays missing; non-Documents are
inapplicable. Results clone only selected immutable handles, without retaining
the semantic context or whole Base bank. Session work, scratch and result-vector
allocation remain under the ordinary controls. The value's checked
`retained_storage_usage()` counts its complete outer header and selected URL
payload, including actual UTF-16 capacity; it excludes unselected banks and
opaque Arc bookkeeping. A consumer storing that same header inline subtracts
exactly the public value's size before adding the receipt's backing.

`HtmlControlledOptionValueCapture` is a narrow ordinary-tree capture and one
OptionValue query under an additional caller envelope. Borrowed add operations
admit retained names, attributes and strings before cloning. The same capture
validation and OptionValue interpreter remain authoritative. Documents use the
existing query-local Disabled scripting and Null registry profile; this does not
certify parser construction or a live Document environment. The controlled
children validator uses admitted indices for membership checks under its existing
quadratic work bound; ordinary builders retain their HashSet representation.

Each `HtmlOptionValueEnvelope` supplies the **total** producer live byte/UTF16
allowance after the caller excludes the producer's current graph once from its
other storage, and the remaining work for that call. Between calls, callers add
the cached O(1) `storage_usage()` to their own mapping/storage admission. The
receipt includes the fixed owner header once, actual backing capacities and
shared string/attribute payloads per retained occurrence. Context and session
borrow the capture; their identities and report qualification do not own another
capture graph. Empty-language Vec headers, cancellation payload and qualification
payloads are admitted independently of the original child allocation tariff.

Growth admits the complete requested replacement bank while its old bank is
live, then uses `try_reserve_exact`. Actual reported capacity is recorded and
checked immediately. This is a representation-capacity strategy, not a promise
about allocator metadata, rounding, physical RSS or recoverable Arc allocation.
Temporary reservation peaks can conservatively retain scratch admission through
the bounded call; cached storage between calls describes the actual retained
graph. Fixed setup work, capture reads/copies and original query work remain
monotone; refused calls return usage and preserve actual retained capacity.
Original child limits and first typed failures remain independent. Allocation
attempt failure stays distinct from representation exhaustion in this front.

Finish validates before replacing the builder. Query uses a local original
context/session and retains the original report. Its first terminal failure is
sticky, including a late parent stop after child completion; an owner never
starts a replacement query session to refund that usage. A Source retry creates
a new owner using the caller's preserved remaining parent/child allowances.
Callers record work/child usage on every outcome, including constructor refusal.
They admit a new occurrence of the borrowed report value while this owner/report
is still live, then drop the owner after transferring their output custody.

Capture builders can establish A/Area URL cache history with the actual historical
parsing Document, independently of current node-Document membership after adoption.
Fresh/cleared null, observed parse failure and unavailable history remain distinct.
The join checks the exact current href and admits retained UTF-16 and Document
references atomically. It establishes no origin, Blob-store or navigation facts.

`select_document_base_serialization` shares the first-present/frozen/missing
selection with `select_document_base`. Its caller establishes the serialized
fallback; the helper does not infer a native Document or manufacture a URL record.

URL records, Base parse-facts banks, EncodingUrl banks and selected Hyperlink
snapshots expose checked `retained_storage_usage()`. This counts owner payload
headers, actual record capacity and UTF-16 backing capacity, including private
fallbacks and spare capacity. Aliased occurrences are conservatively counted
separately; allocator and Arc bookkeeping are excluded. The cache is formed
within existing bounded construction/resolution and does not change logical
formation limits or semantic equality. A selected snapshot counts its chosen
observation, not the original observation bank.

`HtmlCapture::retained_storage_usage_with` observes the complete immutable
capture's actual retained bytes and UTF-16 capacities without allocation or a
cached finalization scan. The caller's callback precedes each header, row and
nested provider observation; its first interruption is preserved. This includes
private record/registry banks, child/dependent banks, registry definitions,
attributes and exact string backings. Nested selected Hyperlink receipts exclude
the snapshot header already held inline in the record bank. Shared payloads count
conservatively per occurrence; consumers can count a complete shared capture once
by its opaque capture identity. Public capture handles, opaque Arc control blocks
and allocator bookkeeping are excluded. Logical capture usage and formation
limits retain their existing meaning. Separately supplied `HtmlCapturedBaseFacts`
banks are outside the capture. Their `retained_storage_usage()` returns a checked
receipt cached during existing bounded formation: the outer value header, both
actual row capacities, each complete nested parse-facts graph, and frozen href
and URL backing, including spare capacity and unused observations. Shared
payloads count per retained occurrence; inline row headers count once. The
separately retained capture graph, Arc control blocks and allocator bookkeeping
are excluded. Consumers retaining a whole bank can admit this complete receipt
without a new scan; logical formation limits and qualification are unchanged.

From the repository root, run focused checks through bounded `pisct run`:

```sh
cargo test --offline --locked -j 1 -p surgeist-html-semantics
cargo clippy --offline --locked -j 1 -p surgeist-html-semantics --all-targets -- -F unsafe-code -D warnings
python3 crates/surgeist-html-semantics/tools/generate_bidi_class.py --check
```

The Unicode first-strong provider is a selected HTML directionality input,
not a full text-layout bidi implementation. Its pinned data and all other
incorporated material retain their [attribution and licenses](NOTICE.md).
