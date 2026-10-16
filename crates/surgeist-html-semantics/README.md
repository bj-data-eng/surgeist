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
Both use the same qualified lookup. Missing observations remain unavailable
evidence. Adapters bind
these facts to actual Documents and encoding truth; this crate parses no URL.

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
