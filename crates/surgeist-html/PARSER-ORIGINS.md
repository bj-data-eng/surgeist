# Parser source origins

The private tokenizer and bounded tree-insertion kernel adapt html5ever 0.40.1 at
[`d7232d746d3112f08926d169591576bd12fe2fdf`](https://github.com/servo/html5ever/tree/d7232d746d3112f08926d169591576bd12fe2fdf).
The behavioral authority is the separate
[pinned HTML edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline),
`877d6146487f15b15bc8dd0e3c625063a37b456a`. The implementation revision records
provenance, not a different HTML dialect. This machinery currently compiles only
for tests. The tree kernel implements adjusted insertion locations, element/text
insertion and stack/scope operations over actual controlled native storage.
The ordinary-document prologue implements Initial and BeforeHtml and hands off
an owned BeforeHead continuation. The subsequent owning head-stage driver handles
BeforeHead, ordinary non-template InHead, Disabled InHeadNoscript and Text.
A fresh standalone Disabled driver also retains actual script metadata and
implements script Text entry, end-tag preparation and EOF marking. The generic
head continuation still suspends before script effects when it lacks that
constructor proof. The fresh driver continues through AfterHead, ordinary InBody
and the two trailing body modes, with formatting reconstruction and adoption.
Ordinary form recovery and parser-created image, button, fieldset and object
associations use that same fresh driver and controlled arena. Seven structural table modes now use that driver,
including pending text and implied column-group, section and row creation.
Resettable control creation, unsupported table rows, select/frameset
transitions, templates and foreign dispatch retain explicit unfinished
continuations. Construction EOF is not completed stop-parsing or a public report.

## Original source and incorporated files

Paths in the first column are relative to that html5ever revision. Destinations
are relative to this crate. Each SHA-256 identifies a complete original file,
before adaptation.

| Original source | Original SHA-256 | Incorporated destination |
| --- | --- | --- |
| `html5ever/src/tokenizer/mod.rs` | `387fef55aea53cf08c2b45192ab8f9f211e993e9dc76ec5031422777bf06773f` | `src/parser/tokenizer/mod.rs`, `src/parser/tokenizer/state.rs` |
| `html5ever/src/tokenizer/states.rs` | `35ea888082a92b77a428af263224169b05ff5740bd519efdace90e44858a0b7d` | `src/parser/tokenizer/states.rs` |
| `html5ever/src/tokenizer/interface.rs` | `cdab80709a6e27663b9a349a607cf85da93fcc9ee3b3a54cf97ee635a9ae7cf4` | `src/parser/tokenizer/token.rs` |
| `html5ever/src/tokenizer/char_ref/mod.rs` | `e753058cb7ab64c63bb13e2156c583bdc60a8468efe6b9c64b8768c6effadf8f` | `src/parser/tokenizer/char_ref.rs` |
| `html5ever/src/tree_builder/mod.rs` | `d69954162f05710cb62fb660666956ec5daa0878e26ba5df9f25db56d3284a36` | bounded insertion-location, stack/scope and body predicates in `src/parser/tree_builder/mod.rs`; raw-mode handoff in `src/parser/tree_builder/head.rs`; generic end handling in `src/parser/tree_builder/body.rs`; active formatting, reconstruction and adoption in `src/parser/tree_builder/formatting.rs`; table reset and structural helpers in `src/parser/tree_builder/tables.rs` |
| `html5ever/src/tree_builder/tag_sets.rs` | `5f1e1adffe46f4af0fe063d5847cf1e0fcc03312ca2e3b0062945a341b836716` | selected scope, implied-end, heading and special-element lists in `src/parser/tree_builder/tag_sets.rs` |
| `html5ever/src/tree_builder/rules.rs` | `877f811266d9d1098f07892473e583b8218d01c0895c8c2d08e40acaec98aa6d` | Initial/BeforeHtml in `src/parser/tree_builder/prologue.rs`; selected head/noscript/Text and Disabled script rules in `src/parser/tree_builder/head.rs`; ordinary body and trailing-mode rows in `src/parser/tree_builder/body.rs`; formatting dispatch in `src/parser/tree_builder/formatting.rs`; structural table modes in `src/parser/tree_builder/tables.rs` |
| `html5ever/src/tree_builder/data.rs` | `256c72e7d368b0be3d1f473c7b36907b6f6b602aff7d1441b9be109fea3375b1` | doctype tables and classifier adaptation in `src/parser/tree_builder/doctype.rs` |

The following hashes record the initial adapted source checkpoint on
2026-10-11. Later implementation changes remain discoverable in Git; these are
historical incorporation hashes, not a claim that subsequent source is unchanged.

| Incorporated file | Initial adapted SHA-256 |
| --- | --- |
| `src/parser/tokenizer/mod.rs` | `5c6bd61d498f81d364a33b2782c9a36a5773e26f84eff09c8a1f0c87cd9de9ad` |
| `src/parser/tokenizer/state.rs` | `402151712f4abe6695a78450cf6899b377193b05cd5d4c62807f346738849c17` |
| `src/parser/tokenizer/states.rs` | `8162c621fd7db4d1e6bafcc29e155fd9779a3820122bda96c60aeaa8f4ea5837` |
| `src/parser/tokenizer/token.rs` | `1383aca3882c68a3a69c727acbf24b224f2bd7437e99edaaf135881e701a3a03` |
| `src/parser/tokenizer/char_ref.rs` | `9a419843984560258ebd57aff9f1a20199f7bb3e3892836983d3423f8690cbb4` |

The tree-insertion kernel's separate initial adapted checkpoint is recorded
below. The earlier tokenizer incorporation hashes remain historical.

| Incorporated file | Initial adapted SHA-256 |
| --- | --- |
| `src/parser/tree_builder/mod.rs` | `139d6ce69a6f47e179969cbc4ca99e25df44126498eeac7a375bc11d29eececb` |
| `src/parser/tree_builder/tag_sets.rs` | `1d50c10ad6e4b3f8a6a086e47bb65226fa4a6d76c33dc3337aafe49db6d7a76f` |

The prologue adds a separate initial adapted checkpoint; earlier hashes above
remain historical.

| Incorporated file | Initial adapted SHA-256 |
| --- | --- |
| `src/parser/tree_builder/prologue.rs` | `91a99f0291d311221aa9bf3aab51c0a5c5a998eb24a2e7d87fa676a8589dd5d1` |
| `src/parser/tree_builder/doctype.rs` | `ef2f9ddfa625128fde2910f1a74ac25b2dda64534e346fe24385f0f88ad1ac64` |

The head-stage adaptation adds its own historical incorporation checkpoint:

| Incorporated file | Initial adapted SHA-256 |
| --- | --- |
| `src/parser/tree_builder/head.rs` | `192900bcd2250130f09aa387fc022a7363078a8e8378e55820c55329b4ff56d5` |

The Disabled-script extension records a later, separate adapted checkpoint.
The earlier head-stage hash remains historical; the fresh execution owner and
native metadata providers are Surgeist-authored and their revisions remain in Git.

| Incorporated file | Disabled-script adapted SHA-256 |
| --- | --- |
| `src/parser/tree_builder/head.rs` | `a19b85cf179750248e02101765a745e0f17246f19ff776f32b10656ca330aaa4` |

The ordinary-body adaptation adds a separate initial incorporation checkpoint;
earlier adapted hashes remain historical.

| Incorporated file | Initial adapted SHA-256 |
| --- | --- |
| `src/parser/tree_builder/body.rs` | `29531aa6707752258e0f1e40bf7474457a675b30d4596b7a11aecebc8852b869` |

The accompanying tokenizer and tree-builder tests are Surgeist-authored
verification, not imported upstream test files. The input, buffer, controller,
managed-payload factories and native arena providers are Surgeist-owned.

## Modifications

The adaptation replaces streaming Tendril buffers, atoms, token sinks and SIMD
fast paths with safe indexed code-point processing over the exact `HtmlSource`.
Tokens retain exact UTF-16 payloads, source ranges and origin records. Private
fallible operations use the crate’s single parser resource controller; they
admit provider reads, copy work, record writes, storage and indivisible fixed
layouts before the corresponding operation. Owner-qualified cleanup remains
available after a terminal stop. Original parser algorithms are reorganized
between dispatch, state, token and reference modules without importing upstream
packaging or public provider interfaces.

Behavior is reconciled with
[HTML §13.2.5](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md),
including its five processing-instruction states. The adaptation distinguishes
literal surrogate input from numeric-reference replacement, retains exact
preprocessing ranges, and uses structured diagnostics and provenance rather
than display strings as semantic data. CDATA admission and raw-text state
selection are explicit private inputs for the future tree builder.

Character-reference matching consumes the existing independently generated
[`character_references.rs`](src/parser/character_references.rs). Its required generation
input is the complete named-reference and C1 tables in the tracked pinned HTML
syntax capture. The generator’s optional original-table comparison uses
html5ever only as a witness; no upstream atom generator, generated PHF table,
Tendril implementation or external dependency is incorporated here.

The tree adaptation implements HTML §13.2.6.1 adjusted insertion, including
foster locations, fragment-root substitution and the source's three ignored
element-placement guards. It routes newly created, unpatched templates through
their actual contents association; imported content-patching state remains
explicitly unavailable. Scope predicates use exact namespaces and the selected
HTML edition's lists, including MathML `annotation-xml`. A controller-owned
open-elements vector admits actual stack storage and visits. Managed factories
consume exact tokenizer payloads into the existing native arena; aliases and
foreign ownership cannot certify a transfer. Terminal cleanup disposes retained
fields and physical capacity even when the final active visit latches a stop.

The prologue adapts `rules.rs` lines 101–144 and the original `mod.rs`
doctype interception and root creation. `data.rs` lines 14–89 supply the
doctypes tables; its classifier is replaced by charged exact UTF-16 matching.
The selected HTML edition governs the restricted non-error doctype syntax,
Silmaril quirks prefix, and HTML 4.01 empty-or-missing system identifier behavior.
Comments, processing instructions and doctypes transfer actual lexical payloads
into native storage. Ignored leading whitespace retains exact source origins,
and the BeforeHead continuation retains any reprocessed token. Foreign
controller payloads are rejected before consuming input or changing tree state.

The fresh ordinary-document constructor establishes non-srcdoc initialization
and parser-cannot-change-mode as false. Hosted, srcdoc, mode-fixed and captured
document execution are unsupported; source labels do not select these modes.
This prologue is a private continuation, not a completed parse result.

The head-stage driver adapts `rules.rs` lines 147–370 and 1007–1032,
the narrow InBody `html` branch at lines 427–435, and `mod.rs` lines 670–687. It uses the same
paused tokenizer, actual controlled arena and retained head pointer. Before
consuming borrowed owners, admission checks their source, controller and lexical
handoff state. Raw-text transitions occur before the next lexical pull; Text
restores the actual saved insertion mode. The pinned edition supplies processing
instructions and treats encoding confidence as irrelevant for supplied Unicode,
so no transport re-decoding is introduced. Generic script and template dispatch
retain owned continuations when the required construction proof is unavailable.
Surgeist-owned providers allocate implied head elements and transfer missing
attributes into managed root storage. Exact namespace/local comparisons, staged
slots and real vector reservation/append are charged by the existing controller;
its own weak ticket and registry owners are restored on failure. Imported
attribute backing remains explicitly unavailable.

The standalone Disabled script adaptation incorporates `rules.rs` lines 234–247
and the script portions of lines 1010–1029, within those previously identified
head/Text ranges. It replaces the upstream end-tag script dispatch with actual
HTML metadata and preparation through the scripting-disabled return. EOF marks
the script already started without preparing it. The fresh request owner,
sealed capability, checkpoint reduction, nesting and insertion-point bookkeeping
are Surgeist-authored adaptations of the selected HTML clauses. They do not
incorporate html5ever's driver or a successful script callback substitute.
The [selected source boundary](../../references/SOURCE-EDITIONS.md#disabled-script-preparation-and-standalone-checkpoint-sources)
identifies the exact JavaScript MIME essence source and complete checkpoint.
Constructor-proved absent runtime domains permit the checkpoint specialization;
Disabled scripting alone does not. Actual structural/text/attribute effects,
script metadata and source-qualified terminal failures remain retained in the
same controlled arena. Imported, hosted, shadow-document and template-document
preparation have no supported execution path in this checkpoint.

The ordinary-body extension adapts selected `rules.rs` rows 356–541, 558–602,
616–630, 693–731, 786–813, 842–873, 880–892, 947–1004, 1466–1493 and 1563–1583.
Stack/body predicates derive from `mod.rs` lines 1043–1223; generic end handling
derives from lines 1555–1585. The implied-end, heading and special-element lists
derive from `tag_sets.rs` lines 70–71, 76, 78–86 and 89–109. The selected HTML
edition governs `rb`/`rtc` implied ends and EOF membership, `search`, obsolete
`keygen` void recovery, exact foreign-name predicates and removal of `isindex`.
Resettable control and template-form creation, unsupported table branches,
foreign creation and frameset transitions remain unfinished boundaries.

The sole fresh driver consumes its actual AfterHead handoff and retains current
and return insertion modes, the stored head identity, frameset eligibility and
first-token LF handling. Late head delegation removes that exact temporary head
entry; script provenance is checked before its diagnostic or stack effects.
Text runs, implied body/paragraph factories, body-attribute merges and image/end-br
recovery use the existing arena, actual payload owners and controller. NULL
diagnostics and recovered character runs retain original UTF-16/source evidence;
recovered tags do not invent lexical origins. Source-defined ignore rows execute,
while unavailable families preserve the token before effects. EOF retains an
unfinished construction continuation rather than certifying report publication.

The formatting extension adapts `mod.rs` lines 713–921, 963–1035, 1516–1553
and 1587–1604, with `rules.rs` lines 730–784, into the private
`src/parser/tree_builder/formatting.rs`. Its initial adapted SHA-256 is
`0e90ea52f6d2a7578a7ec5d01f279f03ed647816d421acad6341ff74ae854dd5`.
Earlier incorporation hashes remain historical. Body/head dispatch also adapts
reconstruction call sites and trailing whitespace delegation. The selected HTML
edition governs original-token association, intended parents and conditional
adoption insertion. Actual applet/marquee/object markers are supported. Original tags have controlled owners; replay
deep-copies their actual payloads into distinct admitted storage, while Noah
comparison uses exact parsed attributes independently of order. Surgeist-owned
session providers retain source/controller/arena qualification and first-stop
cleanup. Ordered child transfer uses coherent individual tree mutations rather
than claiming whole-transfer atomicity; its polled order/depth maintenance and
attribute comparison can require quadratic work.

The ordinary-form extension adapts `rules.rs` lines 543–558 and 632–665 in
`src/parser/tree_builder/body.rs`; only the ordinary branches are admitted.
Pointer storage and creation-association handoff derive from `mod.rs` lines 121,
166, 182–206 and 1366–1403. These use the same original source identities and
notices above; earlier adapted hashes remain historical. The selected HTML
[InBody rules](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-main-inbody)
govern the end-tag order: save and clear the pointer before null/scope checks,
generate implied ends, diagnose a current-node mismatch, then remove only the
pointed identity. The acquired ordinary branch instead removes before its
current-node diagnostic; that upstream order is not adopted. Nested starts
remain error-and-ignore before paragraph closing or form creation. The actual
fresh ordinary constructor initializes its pointer to null; template, hosted
and fragment initialization are not inferred from that state.

Surgeist-owned creation and tree-mutation providers retain actual image owner,
parser-inserted and established/reset evidence in the existing payload registry.
The creation operation compares the intended parent's ordinary root with the
current form's actual root, rather than replacing the source condition with an
upstream sink Boolean. HTML img is form-associated but neither listed nor
resettable, so its `form` attribute does not exclude parser association.
Source removal and insertion effects are synchronous and visit the actual
ordinary subtree. An entire removal pass precedes the insertion pass, including
same-parent reordering and formatting transfers; the providers retain the real
partial metadata prefix if the sole controller stops. Such a stop prohibits
continued observation or freeze and leaves only owner-checked terminal disposal.
Iterative ancestry and registry scans are charged and may require quadratic
work. This adaptation adds no persistent association tree, external provider,
resettable initialization or complete parser-accounting claim.

The listed non-resettable extension adapts `rules.rs` lines 508–518, 604–634
and 765–784 in the body driver, with the same original file hash and notices.
The selected HTML edition governs button diagnostic-before-pop ordering,
fieldset block recovery and object reconstruction/marker clearing. These rows
delegate through the existing structural table modes rather than introducing
another driver. Button, fieldset and object have no resettable initialization
step; input, textarea, output and select remain pre-effect continuations.

Surgeist-owned providers generalize the existing image association registry to
one kind-qualified record. Borrowed source/controller/arena/name/attribute and
parent/form validation plus actual metadata-slot admission precede lexical take.
Explicit form presence suppresses parser-pointer association for listed kinds.
Active and frozen reads preserve qualification and distinguish unavailable
metadata from a known null owner; image reads remain kind-specific.

One retained phase completes each local removal effect and its ID-trigger sweep
before the next event, and completes all removal events before insertion. The
removal view both detaches the moved root and excludes it from outside preorder
scans. Document ID entry/removal visits all canonical same-node-document listed
explicit-form records, including detached controls. Only ID-change restricts
recipients to the changed element's ordinary tree. The
[selected #1136 interpretation](https://github.com/bj-data-eng/surgeist/issues/1136)
uses actual removal-hook invocation, nonempty no-namespace ID and post-removal
non-Document root, without an old-connected guard. This is an explicit source
interpretation, not a browser-consensus or upstream-resolution claim.

Reset clears the parser-inserted flag before source early returns, owner clearing
and connected exact-ID lookup or detached ancestor resolution. The first matching
ID element must be an HTML form; failed connected lookup never gains ancestor
fallback. Actual appended html/body ID slots preadmit and install their effects
with the attribute commit, then drain before further transfer or rebinding.
Terminal stops retain actual metadata prefixes, topology and transfer redirects
for cleanup, and refuse useful continuation, observations and freeze. Repeated
registry/tree scans remain charged and polled; no linear-time guarantee follows.

The structural table adaptation incorporates `rules.rs` lines 1037–1411 and
`mod.rs` lines 1236–1320 into `src/parser/tree_builder/tables.rs`, with table entry
in the body driver. Its initial adapted SHA-256 is
`60453ffe30a328824744e7a0dd92419204d25bc1584b35c5a6f5bde484acbba0`.
Earlier incorporation hashes remain historical. Original source notices are
retained. The selected HTML edition governs the pending-character current-node
set, including template, and the tbody/thead/tfoot section predicate rather than
the upstream table/tbody/tfoot predicate. Processing instructions use actual
native insertion. Hidden-input dispatch retains a pre-effect control-creation
continuation rather than adopting the upstream diagnostic before testing its
type; this branch remains unsupported.

InTable, InTableText, InCaption, InColumnGroup, InTableBody, InRow and InCell use
the existing stack, scope, formatting markers and sole resource controller.
Session-owned implied colgroup/tbody/tr factories and checked document-mode
access are Surgeist-authored providers. Pending characters retain actual managed
payloads and source ranges; NULL is diagnosed and omitted before whole-batch
classification. Whitespace batches insert directly; mixed batches use fostered
body-character handling. Transfer slots retain popped owners before fallible
disposal, so terminal cleanup can release remaining storage after a stopped
flush. Actual delegated insertion modes, Text return and foster state survive
source-ordered reprocessing. Form and image effects reuse their existing owners.
Unsupported control, select, template and foreign branches retain the original
token before unavailable effects. This checkpoint does not certify complete
InTable behavior, fragment initialization, EOF/report completion or complete
parser resource accounting.

Remaining insertion modes, foreign dispatch,
context effects and report publication remain required engine work. The upstream driver, encoding implementation, serializer,
XML parser and RcDom are not incorporated here. Research prototypes and their
measurements do not certify this adapted implementation or supply a public
parsing contract.

## Attribution and verification

Original file notices remain intact. The exact upstream
[copyright](../../licenses/html5ever/COPYRIGHT),
[authors](../../licenses/html5ever/AUTHORS),
[MIT](../../licenses/html5ever/LICENSE-MIT) and
[Apache-2.0](../../licenses/html5ever/LICENSE-APACHE) files accompany this source;
the original dual-license choice is preserved. See the
[crate notice](NOTICE.md#adapted-parser-source) and
[upstream attribution](../../licenses/html5ever/NOTICE.md#incorporated-tokenizer-source).

Focused verification uses the repository’s bounded command runner:

```sh
pisct run --timeout-seconds 180 -- cargo test --offline --locked -j 1 -p surgeist-html --lib parser::tokenizer::tests
pisct run --timeout-seconds 180 -- cargo test --offline --locked -j 1 -p surgeist-html --lib parser::input::tests
pisct run --timeout-seconds 180 -- cargo test --offline --locked -j 1 -p surgeist-html --lib parser::tree_builder::tests
pisct run --timeout-seconds 180 -- cargo test --offline --locked -j 1 -p surgeist-html --lib construction::parser_session::tests
pisct run --timeout-seconds 180 -- cargo clippy --offline --locked -j 1 -p surgeist-html --all-targets -- -F unsafe-code -D warnings
```

These checks exercise the private tokenizer, bounded tree kernel and providers. Full parser
conformance, tree effects, diagnostics/report publication and complete resource
accounting remain obligations of the enclosing parser contribution.
