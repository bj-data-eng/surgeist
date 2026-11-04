# HTML parse, construction and capture fuzz cases

This verification package consumes shared `surgeist-html` values and native
builders, public `surgeist-html-parser` document and fragment parsing, and
`surgeist-html-semantics` captures. Shared [surgeist-fuzz](../../surgeist-fuzz/README.md)
owns engines, workers, supervision, artifacts, replay and reduction. HTML owns
the strict codecs, generation, expectations and evaluators. No HTML production
API or private parser algorithm is copied or exposed here.

| Target | Section | Oracle |
| --- | --- | --- |
| `surgeist-html::unicode-document`, `unicode-fragment` | Cases and codec | Literal seeds and shared parse contracts over arbitrary Unicode |
| `surgeist-html::markup-parse` | Markup recipes | Tree computed from the recipe and the HTML Standard |
| `surgeist-html::capture-import` | Capture import | Source views and the adapter's documented contract |
| `surgeist-html::native-authoring` | Native authoring | Recipe-derived outcome of each builder command |

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
directory. The descriptor records every seed directory (five in
[fuzz.toml](fuzz.toml)) for provider hashing; select the matching target
directory for replay/check. No fabricated defect regression
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
(13.2.5.77-84), newline preprocessing (13.2.3.5), duplicate-attribute removal
(13.2.5.33), SVG tag-name adjustment (13.2.6.5) and the SVG/MathML attribute
adjustments with their namespaces and prefixes (13.2.6.1), the compatibility mode of each prologue (13.2.6.4.1, a
missing doctype identifier being the empty string), template contents, and
declarative shadow roots (13.2.6.4.4; with permission the template contents become the
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
units, 48 items, depth 6, four attributes per element, 56 soup segments) are
generation rejections. The framework credit charges one slot per recipe item or
soup segment plus a fixed four, per-unit byte costs and two text copies; it
bounds the consumer's own recipe, expectation and observation slots, not the
parser's published tree, which the parse caps (managed nodes at most 100000) and
the observer's 100000-node ceiling bound.

Reduction removes items, unwraps elements, drops attributes or shadows, halves a
text and, for soup, removes or halves segments; each proposal is revalidated and
keeps mode, source encoding, caps and the cancellation flag.

**Not covered here.** The balanced language excludes content whose tree depends
on error recovery (tables, foster parenting, formatting-element misnesting,
implied paragraph ends); the token lane reaches that recovery under the generic
contracts only, and the Unicode literals state a few recovery trees. There is no
progressive feed or chunk-partition target: the public parse front doors take one
whole source, and the parser has no feed operation. No target runs scripting, image resources or
an enabled-scripting fragment.

## Capture import

The `surgeist-html::capture-import` target (worker
[html-capture-worker](src/bin/capture.rs)) is the markup target followed by the
public semantics adapter. It parses a recipe or token soup exactly as above,
then imports the published document or fragment into a semantic capture with
`HtmlConstructionCaptureAdapter`. Its canonical `HCP1` form is the `HMK1` form
with six plan bytes after the caps: scripting status (disabled, enabled,
unavailable), registry input (null, unavailable, one shared complete registry),
the `is` slot (absent, unavailable, a value), the shadow-root facts (declarative,
available to internals, keep registry null) and one fault with a selector. A
case of one target is refused by the other.

The adapter's contract is stated in its source and README: facts are supplied
per node and never inferred from topology; a refused import returns the
original builder, source and facts, with the builder's retained usage unchanged;
a completed import yields one subject per reachable node. The check
([captured.rs](src/captured.rs)) walks the source only through its public node
views and compares every captured node:

- kind and payload (document mode, element name and ordered attributes, text,
  comment, processing instruction and doctype), the ordinary parent and ordered
  children, template contents, shadow root and the host of shadow and template
  contents, and the node document (the result document, or the supplied entry
  document for a fragment);
- the supplied external facts exactly as given (scripting, registry identity
  shared across nodes, `is` slot, shadow-root facts), with nothing inferred;
- the subject count, the adapter mapping count and the immutability of the first
  capture across a second, deterministic import of the same source.

A plan injects no fault or one of eleven: a host revision other than the builder's, a missing
node, a duplicated node, a fact of the wrong role, a node of another result, a
registry of another builder, cancellation before the call, a mapping allowance of
one fewer than the reachable nodes, a scratch allowance of zero, a small work
allowance and a capture-builder subject limit of one fewer than needed. Each
fault but the work allowance has one required refusal (host revision mismatch,
missing node document, duplicate node facts, wrong fact role, foreign result,
foreign builder, cancelled, limit exceeded, a capture limit); the work allowance
may either complete or stop at a limit. After a refusal the returned builder
usage must equal its usage before the call, the facts and source must be
returned, admitted usage must stay within the allowance, and the returned
builder and source must complete a fault-free import that passes the same
mirror checks (except after the builder's own subject limit, which cannot be
retried). A stop on the work allowance is `ExpectedRejection`; a completed
refusal check is a verified result.

The native authoring target also imports every frozen result under facts and a
fault chosen from the recipe's shape (`surgeist-html::native-capture`).

**Not covered here.** Semantic query families (`HtmlSemanticQuery`) are not
exercised: the recipe language carries no microdata, form, link, base or
table vocabulary, and their owner keeps their tests. Absent shadow roots and
unavailable (rather than absent) associations of ordinary elements are not
asserted, because the adapter does not claim them.

## Finite campaign record

Campaigns ran against clean detached checkouts outside the active worktree, on
Apple Silicon macOS, with one Cargo job and memory policy `unbounded` (RSS
sampling is unavailable, so no memory bound is claimed). The runs exercised the
finished parser, builders and semantics adapter. Their counts are not
conformance percentages, performance baselines or resource guarantees, and
`Sufficient` means only that the declared minimum number of evaluations ran.
Every row is a summary the framework printed; "Source" is the commit its run
record names.

"Evaluated" is the framework's count of attempts whose outcome checked at least
one property. It includes `ExpectedRejection` outcomes (cancelled, lowered-cap,
allocation and representation stops) and excludes `Unsupported` outcomes and
generation rejections. A summary's `rejected` is the number of generation
rejections only. Where "Evaluated" equals the case count, every attempt produced
a checked outcome.

The earlier runs (first table) executed the harness at `63e350fa5`. Harness
changes made after them, and therefore not exercised by them, are:
`989be0ac5` (native decoder reads pairs with `as_chunks`), `846b593e5`
(`Expect::Balanced` for balanced recipes and the restored literal tests),
`72dab1fed` (soup credit per segment, at most 56 segments), `92a1b60ad`
(completed-import usage check), `e7e208f77` (tests only) and `47a972d75`
(a balanced-recipe stop must be on a lowered cap). The earlier runs thus used a
looser harness than the final one; the final harness was exercised only by the
short runs of the second table.

`check` uses `--toolchain 1.97.0 --memory-policy unbounded --case-ms 5000
--input-bytes 16384`; "Limits" lists the remaining options. For the `check` rows, cases,
wall time and seed come from the retained run records; the `run` rows come from
the campaign command lines.

### Runs at `63e350fa5`

| Command | Target | Seed | Limits | Evaluated | Summary status |
| --- | --- | --- | --- | --- | --- |
| `check` | native authoring | 1172 | cases 5000, wall 1,200,000 ms | 5000 | sufficient |
| `check` | markup parse | 1172 | cases 5000, wall 1,500,000 ms | 5000 | sufficient |
| `check` | capture import | 1172 | cases 5000, wall 1,500,000 ms | 3466 | resource-stop |
| `check` | capture import | 1172 | cases 5000, wall 1,500,000 ms | 2913 | resource-stop |
| `check` | markup parse | 7001 | cases 5000, wall 1,500,000 ms | 2729 | resource-stop |
| `check` | capture import | 7001 | cases 5000, wall 1,500,000 ms | 2304 | resource-stop |
| `check` | native authoring | 7001 | cases 5000, wall 1,500,000 ms | 2128 | resource-stop |
| `check --regressions` on `seeds/markup`, `seeds/capture`, `seeds/native` | one each | 1 | cases 64, wall 600,000 ms | 12, 16, 8 | sufficient |
| `run`, libFuzzer with AddressSanitizer and coverage level 4, `nightly-2026-09-01` | each of the three | 1172 | cases 3000, wall 900,000 ms | 3000 | sufficient |
| `run`, same engine, 46 rounds each | each of the three | 7001 | cases 150000, wall 1,500,000 ms, round cases 10000, round 600,000 ms, retained entries 256 | 150000 | sufficient |

The `check` rows after the first three also passed
`--storage-entries 100000 --storage-bytes 8589934592`; the storage options of the
first three rows are not in the run records and are not recorded. The `run` rows
used `--build-ms 3000000 --input-bytes 16384 --storage-entries 100000
--storage-bytes 8589934592`.

The summaries for the `resource-stop` rows say only `resource-stop` and "finite
fresh owner evaluations; no exhaustive correctness claim"; they do not name the
bound that stopped the run, and no reason is claimed here. The retained run
record of the native seed-7001 stop shows `cases 5000`, `wall_millis 1500000`
and 2128 evaluated. A further `check` was cancelled by an interrupt after 221
markup evaluations (seed 7001), and a first native `check` with the default
1,024-entry quota ended `resource-stop` with 0 evaluated and the detail
"artifact quota exhausted".

### Runs at `47a972d758170de880ab9a414eb6c2049b0bb9b1`

All six summaries printed `Sufficient`, `rejected=0` and `cleanup: complete`,
and the run records name this commit as the build source. The `check` runs used
`--cases 1000 --wall-ms 1500000 --seed 1172 --storage-entries 100000
--storage-bytes 8589934592`; the regression runs used `--cases 64
--wall-ms 600000`, with the work directory shared by all six.

| Command | Target | Evaluated | Summary status |
| --- | --- | --- | --- |
| `check` | native authoring | 1000 | sufficient |
| `check` | markup parse | 1000 | sufficient |
| `check` | capture import | 1000 | sufficient |
| `check --regressions` on `seeds/native` | native authoring | 8 | sufficient |
| `check --regressions` on `seeds/markup` | markup parse | 12 | sufficient |
| `check --regressions` on `seeds/capture` | capture import | 16 | sufficient |

### Findings and commands

No run printed a failure; the failure artifact directories were empty and every
run that evaluated cases printed `cleanup: complete`. There is therefore no
retained failure, replay or reduction and no regression case is bundled. The
summaries do not separate verified outcomes from expected rejections, and do not
count `Unsupported` outcomes.

Commands as run (each `$src` is a clean checkout, and each work directory was
created by the tool and reused by the later commands of the same kind):

```sh
cargo build --offline --locked -j 1 -p surgeist-fuzz --features execution --bin surgeist-fuzz
surgeist-fuzz check --source "$src" --work "$work" --target surgeist-html::capture-import \
  --toolchain 1.97.0 --memory-policy unbounded --cases 5000 --wall-ms 1500000 \
  --case-ms 5000 --input-bytes 16384 --storage-entries 100000 \
  --storage-bytes 8589934592 --seed 7001
surgeist-fuzz run --source "$src" --work "$work2" --target surgeist-html::capture-import \
  --toolchain nightly-2026-09-01 --memory-policy unbounded --cases 150000 \
  --wall-ms 1500000 --case-ms 5000 --build-ms 3000000 --input-bytes 16384 \
  --storage-entries 100000 --storage-bytes 8589934592 --seed 7001 \
  --round-cases 10000 --round-ms 600000 --retained-entries 256
```

The work directory must be outside the source. The default 4 KiB input ceiling
and 1,024 stored entries are too small for these targets; the entry quota may not
exceed 100,000 (the tool rejects more with "invalid bounded storage quotas").

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
driver parity. Each frozen result is also imported into a semantic capture
(`surgeist-html::native-capture`, see Capture import).

Remaining #1139 obligations include UTF8/UTF16 paired semantic projections with
independently mapped coordinates, full metadata/history/reverse-query properties,
semantic query families over captured results, source-qualified unsupported
families, existing corpus/full-path conformance and separately measured
scaling/memory work. The parser has no progressive or fragment feeding;
existing private corpus tests remain independent.
This package certifies no whole HTML contribution or issue closure. Framework
child contracts #1141–1146 were completed at integration; parent #1140 and HTML
#1139 remain owning work.
