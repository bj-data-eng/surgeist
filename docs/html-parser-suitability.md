# HTML parser suitability for Surgeist

Research assessment, 2026-10-10. This recommends a candidate and identifies adoption work; it does not select a final implementation or claim a completed conformance audit.

**html5ever is the leading candidate for a Surgeist-owned HTML parser crate.** Its customizable tree construction is a good fit for the tentative architecture vision. Using it unchanged does not satisfy the whole intended integration: precise source provenance and bounded, cancellable parsing need deeper work. Complete ingestion is a viable option to investigate, explicitly permitted for consideration by the user; MIT OR Apache-2.0 licensing does not preclude it.

## Source basis

The assessed html5ever release is **0.40.1**, tag `html5ever-v0.40.1`, revision `d7232d746d3112f08926d169591576bd12fe2fdf`. The comparison html5gum checkout declares **0.8.4**, revision `51c00d7c47a5780aff913c7937dea064a239b730`; this assessment does not identify that checkout as a release tag. Both official repositories were cloned for inspection. Used implementation evidence is retained in the [html5ever witness](../references/html5ever-parser-contracts--d7232d746d31.md) and [html5gum witness](../references/html5gum-parser-contracts--51c00d7c47a5.md), with hashes and licensed source bodies.

Local integration evidence uses Surgeist `da5e670eb88f297fd1c9bd7293366ca469eea490`, particularly retained and template. The [architecture vision](https://chatgpt.com/space/page_6ce383176ca08191b1dd6d5d16e63979), read at sequence 4, is tentative: typed and raw frontends converge on one authoritative document, with source mapping and off-active parsing followed by transactional publication. Current repository ownership rules remain authoritative. The existing [document contract issue #1074](https://github.com/bj-data-eng/surgeist/issues/1074) overlaps document capabilities; it is not a raw parser implementation issue.

This is source research. No parser was compiled, benchmarked, fuzzed or run against a conformance corpus. Upstream test-coverage statements are upstream claims, not independently reproduced results. No product dependencies, crate implementation or issue relationships were changed.

## Why html5ever fits

Its `TreeSink` lets the caller supply node handles, storage and tree operations. It does not impose a DOM or require a renderer, network stack or JavaScript engine. It supports document and context-dependent fragment entry points, qualified names, foreign content, template content, document mode and the tree edits needed for HTML recovery. That is substantially closer to raw HTML import than extending Surgeist's strict template grammar.

`Parser` supports incremental Unicode input and completion into a caller-selected output type. A Surgeist crate can keep html5ever's types private, use private temporary construction state, and return owned Surgeist data for publication. A temporary parse arena is compatible with one authoritative document; keeping two independently mutable semantic trees after publication would need a different architectural decision.

The declared upstream MSRV is Rust 1.85, below Surgeist's 1.97. Direct dependencies include markup5ever, memchr and log; markup5ever adds web_atoms and tendril. This establishes manifest-level compatibility only, not a resolved build or transitive dependency audit. html5ever and tendril include unsafe implementation code; a Rust implementation is not evidence of an entirely safe dependency graph. Tendril's default non-atomic sharing also warrants constructing and consuming a parser on its worker, then exporting owned transferable results rather than exposing upstream buffers across threads.

The upstream README describes html5lib tokenizer and tree-builder coverage, with qualifications. A custom Surgeist sink still needs its own recovery, mutation and publication verification. `markup5ever_rcdom` explicitly describes itself as test-oriented and unsuitable for production use; it should not become Surgeist's document implementation.

## Integration concerns

| Concern | Evidence and implication | Where work belongs |
| --- | --- | --- |
| Source positions | `TreeSink::parse_error` receives text and `set_current_line` supplies a line. Tokens likewise pass line numbers, without byte ranges or attribute locations. `exact_errors` enriches error reporting; it does not create exact spans. | Parser instrumentation plus a shared origin contract. A wrapper cannot reconstruct locations already discarded. |
| Repaired-node provenance | Tree construction creates, clones, moves and merges nodes/text. Entity decoding and input normalization also separate output text from original bytes. | Define explicit, implicit, repaired and composite origins; carry those through tokenizer and tree construction. One span per final node is insufficient for all cases. |
| Resource admission and cancellation | Public sink callbacks generally return handles or unit, not fallible results. Tokenizer options expose no general work quota. `finish_attribute` scans prior attributes for each new attribute, giving quadratic comparisons for many distinct attributes. | A bounded prototype must establish limits and a real stop/error path inside parsing. Node limits or checks between input chunks alone do not bound work performed while processing one token. |
| Tree-building work | The retained `in_scope` implementation scans the open-element stack. This is evidence of input-dependent internal work, not by itself a measured end-to-end complexity result. | Profile adversarial nesting and repeated recovery operations before choosing work/depth budgets or claiming bounded behavior. |
| Input decoding | `from_utf8` is a lossy UTF-8 decoder, not HTML encoding sniffing or a meta-encoding restart implementation. | Select an initial Unicode/known-UTF-8 import contract explicitly. If byte-stream HTML loading is required, own encoding selection/restart at the loading boundary. |
| Scripting and host behavior | `scripting_enabled` affects parsing, including noscript; it is not a script execution implementation. The driver records unfinished script/encoding integration. | Separate parser configuration from execution, fetching, lifecycle and host policy. An inert importer still needs a deliberate scripting flag. |
| Sink hooks | Template content, form association and declarative shadow-root hooks have semantic consequences. Some default hooks are no-ops; the selectedcontent hook documents an imperfect default. | Enumerate and implement the sink operations needed for selected coverage. Do not claim full host DOM behavior merely because parsing succeeds. |

Ordinary HTML recovery and admission failure must remain distinct. Recoverable authored errors can produce a tree and diagnostics; cancellation, exceeded limits or failed document conversion should return a typed failure and leave the active document unchanged. Silently truncating a tree or using panics as ordinary cancellation is not a suitable substitute.

## Fit with the current document and template crates

Retained already provides canonical topology, model-scoped IDs, revisions, transactional mutation rollback and selector change reports. These are useful foundations. Its current representation does not yet establish faithful arbitrary HTML import:

- `Kind` has no comment or doctype variant, while names and attributes lack namespace/local-name/prefix identity. Existing name validation rejects colon and non-ASCII characters.
- Text/value validation rejects some control characters. Import needs to preserve the selected parser's actual post-recovery output domain rather than assume every value fits existing constructors.
- `Snapshot<'a>` borrows the model. It provides coherent immutable reads, but does not provide an owned old revision that survives mutation of that same model.
- Element/node records have no general source-origin contract. Template spans are frontend-local and generated construction currently does not forward them.
- Separate class and raw-attribute storage needs a single derivation policy. Generic retained sibling facts must not be mistaken for CSS element-only sibling positions.

These observations come from `crates/surgeist-retained/src/{element,string,model,snapshot,mutation}.rs` and `crates/surgeist-template/src/{ast,span,render}.rs` at the stated revision. They are integration requirements, not a claim that retained already promises a complete HTML DOM. #1074 already owns relevant document assessment; avoid duplicating that work.

The existing template parser has a strict custom grammar, matching closing tags, interpolation/control flow and registry-based validation. Preserve it as a distinct authoring frontend. Raw HTML should pass through the dedicated parser crate, tentatively `surgeist-html`, and converge through the authoritative document contract. Current repository rules place Surgeist-to-Surgeist adapters in root unless an explicit exception is adopted; choosing a parser crate does not silently move that adapter or settle final document ownership.

## Alternative and acquisition options

html5gum is useful comparison evidence because it exposes token spans. Its README describes a tokenizer, not an independent full HTML tree constructor. Its experimental tree-building feature depends on html5ever 0.29.0; the inspected bridge ignores its span argument and sends a constant line number to html5ever. As supplied, it neither replaces modern html5ever tree construction nor fixes end-to-end source provenance. Its MIT license is compatible with further investigation, but source positions alone do not justify selecting this bridge.

| Approach | What it buys | Remaining cost |
| --- | --- | --- |
| Wrap html5ever 0.40.1 | Smallest maintenance surface; private sink and Surgeist API; upstream fixes remain easy to consume. | Does not itself supply exact origins, parser-internal work budgets or general early abort. Suitable only if initial requirements explicitly accommodate those limits. |
| Maintain a focused upstream-derived patch | Direct access to tokenizer/tree-builder instrumentation while preserving an identifiable upstream base. | Rebase and test ownership; API changes must span both tokenization and tree construction. Upstream acceptance cannot be assumed. |
| Ingest implementation into the Surgeist parser crate | Full control of origin tracking, typed failures, admission and public API; no forced upstream intermediary. | Own correctness fixes, standards evolution, corpus integration and synchronization decisions. Replace upstream executable unsafe with safe code; preserve provenance/licenses and assess each transitive dependency separately. |

Complete ingestion is technically and legally plausible, but cannot mean copying the pinned implementation unchanged. Current PISCT repository policy prohibits executable unsafe in repository-owned Rust. html5ever’s tokenizer contains unsafe SIMD paths, so ingestion must replace or redesign those paths using safe repository code. Dependency internals can remain behind safe APIs; bringing tendril or another dependency into repository ownership would require the same assessment. This adds concrete implementation and performance-validation work to the ingestion option. The assessed html5ever code is **MIT OR Apache-2.0**; Surgeist can choose MIT for those files while preserving required notices. That choice does not relicense unrelated dependencies or third-party test data. The [retained notices](../licenses/html5ever/NOTICE.md) cover this research material; actual ingestion would require an inventory of every incorporated component, tests and generated tables.

An ingested crate should have one maintained parser implementation, not parallel upstream and copied engines hidden behind compatibility shims. Ingestion does not mean copying the whole upstream repository: bring over the implementation and supporting data actually required, preserve their source basis, and use dependencies directly where that serves the chosen boundary.

## Recommended next step

Use html5ever as the baseline for a bounded integration prototype before choosing wrapping versus ingestion. The deciding work is concrete:

1. Demonstrate origin propagation across entities, normalized newlines, implicit elements, adoption-agency repair, foster parenting and merged text. Include document and contextual fragment parsing. Specify which origins are exact and which are derived.
2. For ingestion, establish safe replacements for upstream unsafe paths and measure their cost. Demonstrate typed early termination and admission for large attribute sets, deep nesting, large text and repeated tree repair. Measure work and memory; ensure failure cannot publish a partial active document.
3. Exercise a custom sink over namespaces, comments/doctype, quirks, template content and the selected host hooks. Check that different input chunk boundaries yield equivalent trees and diagnostics under the declared input contract.
4. Demonstrate one publication through the authoritative document contract, including failed conversion and identity/revision handling. Reuse #1074 outcomes rather than create a second document owner.

If these require broad internal plumbing, ingestion may be cleaner than a wrapper plus an extensive permanent patch. If the changes remain small and maintainable upstream, a focused patch can reduce long-term ownership. The research supports html5ever as the foundation; it does not yet establish which acquisition strategy is cheaper or that an unchanged dependency is implementation-ready for the full vision. These parser-specific questions do not create new gates for unrelated CSSOM, text, layout or render work.
