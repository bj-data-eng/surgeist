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
Generator revision 2 introduced literal 8 into this mapping; revision 1 used
modulo eight. The codec stays at revision 1: it still decodes every revision-1
case identically and only additionally accepts literal ID 8. Other entropy maps
to exact little-endian UTF16 pairs (odd final high byte zero), or to valid UTF8
by explicitly mapping bytes to Unicode U+0000..U+00FF. This is Unicode
generation, not decoding malformed byte transports. Overlarge generated source
is a separate GenerationRejected outcome before source allocation; each
descriptor permits at most 64 such rejections in its nonvacuous assessment.
Checked canonical input can contain any valid UTF8 scalar or any UTF16 unit,
including isolated surrogates. No lossy UTF8 conversion occurs.

## Expected properties

Literal seeds independently cover duplicate attribute first occurrence/order,
references, CRLF, literal BOM, isolated surrogate input, NUL recovery, qualified
SVG attributes, template association, HTML 877d processing instructions ending
in `?>` and in `>`, malformed paragraph recovery and foster parenting. Expected
native values and selected exact source/diagnostic intervals are handwritten
from the pinned HTML contract and existing public source-qualified examples, not
sampled parser output. Each target has nine canonical `.case` files in its seed
directory. The descriptor records both directories for provider hashing; select
the matching target directory for replay/check. No fabricated defect regression
is bundled.

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

## Markup recipes

The `surgeist-html::markup-parse` target (worker
[html-markup-worker](src/bin/markup.rs)) parses generated markup through the
public document and fragment front doors and compares the result with a tree
computed from the recipe alone. The [model](src/markup/model.rs),
[serializer and expected tree](src/markup/tree.rs), [codec](src/markup/codec.rs),
[generator](src/markup/generate.rs), [evaluator](src/markup/observe.rs) and
[handwritten recipes](src/markup/seeds.rs) belong to HTML and use the shared
engines, workers and reduction unchanged.

Canonical `HMK1` is little endian: magic, mode (document or fragment), source
(UTF-8 or exact UTF-16), document prologue or fragment context, document shell
or declarative-shadow permission, body kind, a cancelled-before-parse flag, nineteen
u64 parser caps (the same envelope as the Unicode targets), then the body with no
trailing bytes. A body is either a balanced recipe or free-form token soup. Every
tag, table index, piece and count is checked before use; isolated surrogates are
accepted only in a UTF-16 source.

**Balanced recipes.** A recipe is a tree of text, comments and elements from
closed tables, in positions where tree construction only appends: flow content
(`div`, `section`, `p`, `br`, `hr`, `template`), phrasing (`span`, `b`, `em`, `a`,
with no nested paragraph or anchor), SVG (`svg`, `g`, `rect`, `circle`,
`foreignObject` as an HTML integration point) and MathML (`math`, `mrow`, `mi`,
`mn`). Text and attribute values are pieces: any UTF-16 unit but NUL, CR, `<`,
`&` and the quotes, eight named references, numeric references (decimal or
hex), CR, CRLF and NUL. Source spelling varies independently of the tree: tag and
attribute-name case, quoting, bare and empty attributes, self-closing foreign
elements, the document shell, and ten prologues (none included). The expected tree comes from the
pinned HTML Standard: tokenizer references and the numeric end-state table
(13.2.5.72-84), newline preprocessing (13.2.3.5), duplicate-attribute removal
(13.2.5.33), SVG/MathML tag and attribute adjustments with their namespaces and
prefixes (13.2.6.5), the compatibility mode of each prologue (13.2.6.4.1, a
missing doctype identifier being the empty string), template contents, and
declarative shadow roots (with permission the template contents become the
host's shadow root; without it the template is an ordinary first child that keeps
its attributes). The model's own serialization and expected trees are tested
against handwritten values with no parser involved, and tests show the tree
comparison reporting each kind of disagreement.

Contexts: documents use an offline request; fragments use a captured context
element in an ordinary no-quirks document, chosen from 24 (`div`, `section`,
`template`, SVG `g` and `foreignObject` have balanced recipes; `table`, `tr`,
`td`, `select`, `caption`, `colgroup`, `body`, `html`, `head`, `textarea`,
`title`, `style`, `script`, MathML `mi` and `annotation-xml`, SVG `svg`,
`frameset`, `plaintext` and `option` take token soup). The fragment algorithm
runs with scripting inert and the case's declarative-shadow permission.

**Token soup** concatenates 74 closed tokens (table, select, foreign, template,
raw-text, formatting and malformed-markup openers, tokenizer-state openers and
closers) with literal unit runs that include NUL, CR and isolated surrogates. It
has no tree oracle; it checks the shared contracts below plus tree invariants
every parse must hold (HTML names and attribute names folded, no empty text,
unique attribute expanded names, every HTML template has contents).

**Checks and classification.** Every parse shares the Unicode targets'
source-binding, native traversal, typed diagnostic, D1 interval, late-cancel
immutability and failed-parse ownership checks. A balanced recipe also compares
its tree. Resource, allocation and representation stops, pre-cancellation and
lowered caps are `ExpectedRejection` with terminal ownership checked; unsupported
states are `Unsupported`; a source, report, ownership or tree disagreement is a
property failure with a stable discriminator. Generation limits (4096 source
units, 48 items, depth 6, four attributes per element) are generation rejections.

Reduction removes items, unwraps elements, drops attributes or shadows, halves a
text and, for soup, removes or halves segments; each proposal is revalidated and
keeps mode, source encoding, caps and the cancellation flag.

**Not covered here.** The balanced language excludes content whose tree depends
on error recovery (tables, foster parenting, formatting-element misnesting,
implied paragraph ends); the token lane reaches that recovery under the generic
contracts only, and the Unicode literals state a few recovery trees. There is no
progressive feed or chunk-partition target: the public parse front doors take one
whole source, and the parser's feed operations are private, so their exclusion and
their private tests are unchanged. No target runs scripting, image resources or
an enabled-scripting fragment.

## Native authoring

The separate `surgeist-html::native-authoring` target uses the actual public
append-only document/fragment builders, checked processing-instruction values,
ordered attributes, template contents and declarative shadow attachment. Its
worker is [html-native-worker](src/bin/native.rs); the concrete
[consumer](src/native/mod.rs), [codec](src/native/codec.rs),
[observer](src/native/observe.rs) and [literal recipes](src/native/seeds.rs)
belong to HTML. It uses the same shared engines and worker protocol as the
Unicode targets. No private parser, live mutation or semantic query is invoked.

Revision 1 `HNA1` begins with four magic bytes, one root/mode byte (fragment,
no-quirks, limited-quirks, quirks), one literal ID (0..7 or 255 for general),
five little-endian u64 native caps and one command-count byte. Commands encode
element, text, comment, strict PI, doctype, template-contents lookup or shadow
attachment. Each names an earlier command handle (root is zero), or 255 for a
real handle issued by a different builder. Raw names preserve namespace URI,
optional prefix and local name; attributes preserve their order. Optional
fields retain absent versus present-empty. Every string is a u16 length and
exact little-endian UTF16 units. Known HTML names and two hidden keywords use
closed tags and the public canonical producers. Unknown tags, forward handles,
truncation, extra bytes, changed literal input and over-envelope fields/caps are
format errors before allocating native values. A handle from a rejected command
cannot be used: evaluation stops that dependency chain and checks the real
admitted prefix, rather than substituting a root or invented node.

The hard target policy is 64 KiB encoded input, 256 KiB precredited consumer
buffers/wrapper allowance, 64 total native nodes including qualification probes,
depth 8, 8192 aggregate name/value units, 128 attributes (eight per element),
64 commands and 96 coarse recipe/observation-operation credits. Names, URIs and
prefixes are at most 128 units; individual values/text are at most 512.
Native limits default to `(64,128,128,8192,8)` for nodes, edges, attributes,
units and depth and may be lowered to zero. Framework credit uses actual Rust
slot sizes, at most two recipe and encoding buffers, four payload copies,
bounded child/handle/identity slots, and a stated 576-byte wrapper allowance per
possible node. Reservations check observed vector capacity before writes.
The framing/field bounds prove a tighter 18,735-byte encoding buffer ceiling.
These are consumer allocations and logical native limits, not all native arena
allocation, allocator overhead, RSS, or a native work/cancellation controller.

Eight [input seeds](seeds/native) independently specify exact payloads and
ordered attributes, HTML/SVG/MathML/custom namespaces, absent/empty prefixes and
doctype identifiers, isolated surrogates, template/shadow forward and reverse
links, strict PI rejection, duplicate expanded attribute names, and real foreign
IDs. Expected values and error classes are declared from the public construction
contracts and pinned language rules, not sampled output goldens. Frozen results
must preserve parent/child/association order and identity; cloning retains the
same qualified IDs and another result rejects them. Rejected appends must leave
admitted usage unchanged, and later independent valid appends are exercised.
Native rejection is nonterminal, unlike the parse controller's first-stop model.

Generation and evaluation:

- One quarter of entropy selects literal recipes, and one quarter an exact UTF16
  text append. The rest drives a bounded command generator over closed pools:
  near-miss template spellings (`Template`, `TEMPLATE`, foreign `template`, a
  prefixed HTML `template`), attribute names sharing or differing in namespace
  and local name, reserved and malformed PI targets and data, leaf and template
  parents, repeated shadow attachment, handles from another builder, isolated
  surrogates and quotas lowered on one axis. Recipes over the consumer envelope
  are generation rejections, not evaluations.
- Before each command the evaluator states the contract's outcome from the
  recipe alone and compares the builder's reply with it: attribute duplicates
  (expanded name, never prefix), PI syntax, foreign handle, parent kind, then
  the first exceeded resource in declaration order with its exact limit and
  attempted count. The builder's usage must equal the recipe-derived usage after
  every success and be unchanged after every refusal.
- A predicted typed contract refusal is a verified positive result and the case
  is `Evaluated`. A product resource refusal (a lowered quota, an allocation or
  representation limit) is `ExpectedRejection`. A reply that differs from the
  prediction is a property failure. A command naming a handle that an earlier
  command did not produce stops only its own dependency chain.
- Reduction removes command suffixes, removes one command that no later command
  names (renumbering later handles, never repairing them) or halves one text or
  comment payload, retaining mode, caps and UTF16 units. Literal oracles are not
  reduced.

Tests cover default workers, strict transport, quota boundaries, contract
refusals, canonical replay/reduction, registry, and finite property and coverage
driver parity.

Remaining #1139 obligations include additional actual captured contexts/options,
UTF8/UTF16 paired semantic projections with independently mapped coordinates,
full metadata/history/reverse-query properties, native authoring/semantic operation
targets, source-qualified unsupported families, clean-source finite campaigns,
existing corpus/full-path conformance and separately measured scaling/memory work.
Private progressive ordinary feeding and fragment-feeding exclusions are unchanged;
existing private feed/corpus tests remain independent. This package certifies no
whole HTML contribution or issue closure. Framework child contracts #1141–1146
were completed at integration; parent #1140 and HTML #1139 remain owning work.
