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
Unsupported or reached unavailable facts return typed failures. Failure retains
the original request and live ownership until explicit cleanup; no successful
partial report is returned.

Completed reports retain each ordinary element's no-definition custom-element
state and independent creation-time `is` value through the public metadata
cursor. Formatting reconstruction and cloning preserve their actual creation
causes; later HTML/body attribute merges do not rewrite that saved value.

From the repository root, run focused checks through bounded `pisct run`:

```sh
cargo test --offline --locked -j 1 -p surgeist-html-parser
cargo clippy --offline --locked -j 1 -p surgeist-html-parser --all-targets -- -F unsafe-code -D warnings
python3 crates/surgeist-html-parser/tools/generate_character_references.py --check
```

The existing [HTML fuzz package](../surgeist-html/fuzz/README.md) consumes these
public requests directly. [Attribution](NOTICE.md) preserves parser source and
fixture notices.
