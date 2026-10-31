# HTML Unicode parse fuzz cases

This verification package consumes shared `surgeist-html` values, public
`surgeist-html-parser` document and fragment parsing, and explicit
`surgeist-html-semantics` captures. Shared [surgeist-fuzz](../../surgeist-fuzz/README.md)
owns engines, workers, supervision, artifacts, replay and reduction. HTML owns
this strict codec, generation, literal expectations and evaluator. No HTML
production API or private parser algorithm is copied or exposed here.

## Cases and codec

Two `structured` targets are registered by [fuzz.toml](fuzz.toml):
`surgeist-html::unicode-document` and `surgeist-html::unicode-fragment`.
Each implements the same `UnicodeTarget` evaluator in [src/observe.rs](src/observe.rs),
used unchanged by ordinary workers, property testing, coverage, canonical replay
and reduction. Features `property` and `coverage` are optional; default workers
have neither engine. Owned Rust forbids unsafe code, including coverage macros.

Canonical revision 1 is little endian:

| Bytes | Meaning |
| --- | --- |
| 0..4 | `HUF1` magic/codec revision |
| 4 | target: 0 document, 1 fragment |
| 5 | source: 0 valid UTF8, 1 exact UTF16 |
| 6..8 | lane and literal ID; nonliteral ID must be zero |
| 8..160 | nineteen u64 parser caps in `HtmlParseResource` order |
| 160..164 | u32 UTF8 byte count or UTF16 unit count |
| 164..end | exactly that payload, with no trailing bytes |

Lanes are 0 arbitrary/general, 1 pre-cancelled, 2 nonempty source denial,
3 work denial, 4 independent literal (ID 0..8), 5 missing captured scripting.
The two denial lanes keep other caps at the documented generous envelope;
literals require their exact input and that envelope. General cases can lower
any cap through the checked constructor. A case always preserves all nineteen
explicit caps; zero is valid and there is no unlimited sentinel.

The fragment tag denotes one closed captured recipe: a real div root in an
ordinary no-quirks HTML document, absent template cache, explicit no shadow,
absent is value, null registry and disabled scripting. Lane 5 replaces only
scripting with an unavailable fact, requiring the exact public request error.
Parser-creation image support is explicitly disabled, separately from target
document support. Fragment mode is Inert (effective Disabled) and declarative
shadow permission is false. These are fixed revision-1 facts, not guessed
missing host facts. Arbitrary context names/options are not accepted by this
codec; adding them requires an explicit codec/generator change.

Before source allocation, decoding checks byte/length/UTF8 boundaries, cap
representability/envelope, all tags and aggregate framework credit. The fixture
ceiling is 64 KiB canonical bytes, 4096 source UTF16 units, 64 captured subjects,
depth 8, 8192 aggregate text units and 8 operation invocations. Fixed div facts
are charged two subjects, depth one, 128 text units and 256 bytes conservatively;
three public operations cover request/parse/late cancellation. These ceilings
bound this initial workload, not all HTML or process RSS. The generous product
caps are [CAPS](src/codec.rs): logical storage/layout/work, not allocator headers,
wall time or whole-process memory. Owner observations additionally bound native
nodes and D1/diagnostic rows at 100000; observation exhaustion is Resource,
not a normative semantic violation.

One quarter of generated recipes select independent literal successes: when the
first entropy byte is a multiple of four, the literal ID is that byte divided by
four, modulo nine (bytes 0, 4, ..., 32 select IDs 0..8 and byte 36 wraps to 0).
Other entropy maps to exact little-endian UTF16 pairs (odd final high byte zero), or
to valid UTF8 by explicitly mapping bytes to Unicode U+0000..U+00FF. This is
Unicode generation, not decoding malformed byte transports. Overlarge generated
source is a separate GenerationRejected outcome before source allocation; each
descriptor permits at most 64 such rejections in its nonvacuous assessment.
Checked canonical input can contain any valid UTF8 scalar or any UTF16 unit,
including isolated surrogates. No lossy UTF8 conversion occurs.

## Expected properties

Literal seeds independently cover duplicate attribute first occurrence/order,
references, CRLF, literal BOM, isolated surrogate input, NUL recovery, qualified
SVG attributes, template association, HTML 877d processing instructions ending in `?>` and in `>`,
malformed paragraph recovery and foster parenting. Expected native values and
selected exact source/diagnostic intervals are handwritten from the pinned HTML
contract and existing public source-qualified examples, not sampled parser
output. Each target has nine canonical `.case` files in its seed directory.
The descriptor records both directories for provider hashing; select the matching
target directory for replay/check. No fabricated defect regression is bundled.

General success checks source identity/units and limits; bounded native parent,
sibling and template/shadow ownership; typed diagnostic chronology/ranges; native
D1 source intervals, current-field output bounds, destination variants, cause
operation/rule/token and group units. Post-publication cancellation compares those
same typed observations and usage. Diagnostic offsets may go backwards; no
monotonic-offset assertion exists. D1 origins can relate unequal input/output
lengths; retired ordinals are not current attribute indexes. The package does not
expand every historical prefix or claim full history/metadata equivalence.

Explicit cancel/quota lanes check actual typed refusal, source admission and
first stop, original request retention, cleanup and stable late-cancel evidence.
Disposable live storage is checked only in an actual after-cleanup snapshot;
retained failure diagnostics/ownership are not relabeled as leaked storage.
Unexpected engine/invariant/cleanup conditions are property failures. Actual
product allocation/representation/quota refusals are ExpectedRejection with
checked ownership properties, not ordinary parse successes. Known typed execution,
construction or missing-context capability refusals are Unsupported and cannot
fill the minimum evaluated count. Unsupported provenance/completion/invariant
states are not silently accepted. Generation rejection, format/capture error,
framework/observation resource exhaustion and product refusal stay distinct.

Reduction offers only valid source prefixes with original context/caps/lane,
using UTF8 scalar boundaries or exact UTF16 unit boundaries. It stops immediately
when the framework visitor refuses. Literal input is never shortened while
retaining its oracle. Framework same-FailureKey confirmation owns whether any
proposal actually minimizes a real failure; successful reduction protocol tests
are not evidence of a discovered or minimized defect.

## Verification and campaign boundary

```sh
cargo test --offline --locked -j 1 -p surgeist-html-fuzz
cargo test --offline --locked -j 1 -p surgeist-html-fuzz --all-features --lib --tests
cargo clippy --offline --locked -j 1 -p surgeist-html-fuzz --all-targets --all-features -- -F unsafe-code -D warnings
cargo fmt --all -- --check
```

Use bounded `pisct run` and an isolated target for concurrent package changes.
Tests exercise actual workers, registry, strict codec, all 18 literal successes,
resource/cancel evidence, canonical replay/reduction, and (when selected) finite
property and coverage drivers. Coverage-driver parity is not an instrumented
libFuzzer campaign. The normal worker's capture precedes evaluator execution.

ROOT selects a clean committed source and separate finite campaign directory
for framework `list`, `validate`, `check`, `run`, `replay` and `minimize`; see its
[workflow](../../surgeist-fuzz/README.md#run-a-finite-campaign). Do not run campaigns
against active dirty source or install tooling. Choose positive ordinary-success
coverage from the canonical seed checks separately from quota ownership counts.
The current framework backend reports RSS sampling unavailable and rejects a
required hard memory limit; finite fixture/product limits imply neither RSS nor
allocator-overhead bounds.

Remaining #1139 obligations include additional actual captured contexts/options,
UTF8/UTF16 paired semantic projections with independently mapped coordinates,
full metadata/history/reverse-query properties, native authoring/semantic operation
targets, source-qualified unsupported families, clean-source finite campaigns,
existing corpus/full-path conformance and separately measured scaling/memory work.
Private progressive ordinary feeding and fragment-feeding exclusions are unchanged;
existing private feed/corpus tests remain independent. This package certifies no
whole HTML contribution or issue closure. Framework child contracts #1141–1146
were completed at integration; parent #1140 and HTML #1139 remain owning work.
