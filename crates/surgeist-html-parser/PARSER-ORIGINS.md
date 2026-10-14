# Parser source origins

The private tokenizer and bounded tree-insertion kernel adapt html5ever 0.40.1 at
[`d7232d746d3112f08926d169591576bd12fe2fdf`](https://github.com/servo/html5ever/tree/d7232d746d3112f08926d169591576bd12fe2fdf).
The behavioral authority is the separate
[pinned HTML edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline),
`877d6146487f15b15bc8dd0e3c625063a37b456a`. The implementation revision records
provenance, not a different HTML dialect. This internal machinery compiles in
production behind the
[decoded-Unicode request/report boundary](../../docs/reference/html-operations.md#decoded-unicode-parsing).
The tree kernel implements adjusted insertion locations, element/text
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
Ordinary frameset entry and its three insertion modes also use that driver.
Ordinary textarea creation, RCDATA and raw/reset hooks now use it as well.
Fresh ordinary output creation also completes its empty reset before association;
null-default and current observations read actual ordinary descendant text.
Ordinary select/option/optgroup/selectedcontent creation and source-ordered
selection, cloning and intrinsic checkpoint effects now use that driver.
Submit/Reset/Button Default-mode and five strict temporal input states use the
same canonical input owner. Current control coverage and typed capability limits
are described in the [crate overview](../../docs/reference/html-operations.md#decoded-unicode-parsing);
the extension sections below record their source adaptations. Fresh ordinary templates
use that same driver with real inert-document/contents ownership and PI patching.
Exact SVG scripts now retain separate processed state under the selected
restricted offline operation, with source-ordered foreign-end and nonparser
connection handling. The [ordinary EOF completion](#ordinary-eof-completion)
retains the whole internal driver after the selected end operation for the
source-owning public report assembler.

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
| `html5ever/src/tree_builder/mod.rs` | `d69954162f05710cb62fb660666956ec5daa0878e26ba5df9f25db56d3284a36` | bounded insertion-location, stack/scope and body predicates in `src/parser/tree_builder/mod.rs`; raw-mode handoff in `src/parser/tree_builder/head.rs`; generic end handling in `src/parser/tree_builder/body.rs`; active formatting, reconstruction and adoption in `src/parser/tree_builder/formatting.rs`; table reset and structural helpers in `src/parser/tree_builder/tables.rs`; template insertion routing/reset in `src/parser/tree_builder/mod.rs` and `src/parser/tree_builder/template.rs`; foreign integration/recovery in `src/parser/tree_builder/foreign.rs` and name/attribute adjustment in `src/construction/foreign.rs` |
| `html5ever/src/tree_builder/tag_sets.rs` | `5f1e1adffe46f4af0fe063d5847cf1e0fcc03312ca2e3b0062945a341b836716` | selected scope, implied-end, heading and special-element lists in `src/parser/tree_builder/tag_sets.rs` |
| `html5ever/src/tree_builder/rules.rs` | `877f811266d9d1098f07892473e583b8218d01c0895c8c2d08e40acaec98aa6d` | Initial/BeforeHtml in `src/parser/tree_builder/prologue.rs`; selected head/noscript/Text and Disabled script rules in `src/parser/tree_builder/head.rs`; ordinary body and trailing-mode rows in `src/parser/tree_builder/body.rs`; formatting dispatch in `src/parser/tree_builder/formatting.rs`; structural table modes in `src/parser/tree_builder/tables.rs`; template start/end and InTemplate rules in `src/parser/tree_builder/template.rs`; frameset entry and trailing modes in `src/parser/tree_builder/frameset.rs`; MathML/SVG entry in the body driver and foreign rules in `src/parser/tree_builder/foreign.rs` |
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

The private progressive-availability cursor and retained reference continuation
are Surgeist-authored adaptations over that same immutable source. They preserve
the actual tokenizer and construction owners across `NeedInput`, with explicit
sealing and original source coordinates, rather than restoring Tendril streams
or reparsing source prefixes. Their literal partition tests are authored locally;
the original incorporation checkpoints above remain historical.

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
same controlled arena. Actual inert template contents now use the preparation
prefix through the source empty/Disconnected guards, before type classification
and the scripting-disabled return. Imported, hosted and shadow-document
preparation still have no supported execution path in this checkpoint.

The ordinary-body extension adapts selected `rules.rs` rows 356–541, 558–602,
616–630, 693–731, 786–813, 842–873, 880–892, 947–1004, 1466–1493 and 1563–1583.
Stack/body predicates derive from `mod.rs` lines 1043–1223; generic end handling
derives from lines 1555–1585. The implied-end, heading and special-element lists
derive from `tag_sets.rs` lines 70–71, 76, 78–86 and 89–109. The selected HTML
edition governs `rb`/`rtc` implied ends and EOF membership, `search`, obsolete
`keygen` void recovery, exact foreign-name predicates and removal of `isindex`.
At this ordinary-body adaptation checkpoint, other resettable control creation,
table branches and foreign creation remained unfinished. The subsequent
extension sections record their implementation. Frameset transitions are extended below.

The sole fresh driver consumes its actual AfterHead handoff and retains current
and return insertion modes, the stored head identity, frameset eligibility and
first-token LF handling. Late head delegation removes that exact temporary head
entry; script provenance is checked before its diagnostic or stack effects.
Text runs, implied body/paragraph factories, body-attribute merges and image/end-br
recovery use the existing arena, actual payload owners and controller. NULL
diagnostics and recovered character runs retain original UTF-16/source evidence;
recovered tags do not invent lexical origins. Source-defined ignore rows execute,
while unavailable families preserve the token before effects. Ordinary EOF now
uses the separately qualified [completion owner](#ordinary-eof-completion),
before the public entry can publish a qualified completed report.

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
fresh ordinary constructor initializes its pointer to null. Ordinary open-template
creation context is now supplied by the actual qualified stack; imported, hosted
and captured-fragment form initialization is not inferred from that state.

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
step; input and select remain pre-effect continuations. Textarea and output
are extended below with their own resettable initialization.

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

The textarea extension adapts `rules.rs` lines 874–878 in the existing body
driver, preserving the original notice and complete rules-file hash above.
The selected HTML edition governs creation reset before parser association,
first-LF omission, RCDATA, saved caller mode, children-change raw updates and
reset on every actual parser stack removal. Raw/reset behavior derives from
the [textarea clauses](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)
and the selected [DOM children-change hooks](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md).
The upstream row supplies implementation provenance, not the whole raw-state contract.

Surgeist-owned textarea metadata indexes independently copied raw UTF-16 backing
in the same controlled payload registry. Removal hooks and old-parent child-text
updates use the excluded-root view before insertion hooks and final-parent updates;
text merges invoke the parent children-change hook. Stack pop, identity removal
and replacement reset the actual removed textarea before mode restoration or EOF
reprocessing. New and old raw buffers coexist under actual admission; replaced
backing is disposed while initialized retired slots remain charged until registry
disposal. Terminal cleanup performs no useful reset. Repeated raw copies and
registry scans may be quadratic and remain charged and polled. This checkpoint
adds no user interaction, API-value normalization or hosted execution.

The output extension removes only the existing output Form borders in the
ordinary body and structural table drivers. It ingests no additional html5ever
source; the original notices and complete upstream hashes remain unchanged.
The selected HTML edition governs fresh creation reset before association and
current/default value observation through
[ordinary descendant text](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-output-element).
Fresh empty reset follows the selected
[DOM string replace-all and replace-all operations](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md#string-replace-all):
no children are removed or added, and no mutation record is queued.

Surgeist-owned output metadata retains the actual null override in the same
controlled registry, before canonical listed association. Active and frozen
observations share the existing ordinary traversal and copy actual descendant
Text into owner-qualified admitted buffers; they introduce no output value cache
or children-change/pop reset hook. The traversal factoring preserves virtual
removal branches and charges used by canonical form effects. Terminal stops keep
partial scratch owned by its caller and prohibit observations/freeze until
disposal. Repeated scans may be superlinear and stay charged and polled. This
checkpoint adds no setters, nonnull overrides, general form reset, hosted
initialization.

The Hidden-input extension adapts the existing `rules.rs` body branch at lines
815–840 and the table branch at lines 1107–1115. The latter is already within
the incorporated structural-table range; no additional upstream source is
ingested. Original notices, complete upstream hashes and earlier incorporation
hashes remain unchanged. The selected HTML edition governs ordered initial
attribute changes, Text-state value sanitization, nonempty value writeback on
transition to Hidden, and fresh reset before parser association through
[the input element](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element).

The existing standalone capability qualifies creation before token transfer.
Surgeist-owned state and original token/value-attribute ranges use the same
controlled payload registry. Native attribute replacement uses one admitted
controller operation; raw value replacement retains independent backing.
Original ranges identify actual creation writeback without inventing merged
source ranges for removed CR/LF. Active and frozen private observations preserve
owner qualification. Actual old and new backing coexist under admission;
obsolete backing is released while initialized registry slots remain charged
until disposal. A terminal stop retains partial owners and prevents publication;
cleanup runs no useful reset. Repeated scans and copies may be quadratic and
remain charged and polled. Other input states, hosted initialization, general
setters remain separate work.

The Submit/Reset/Button extension adds only canonical input-kind mappings and
Default-mode selection in [the shared input rules](../surgeist-html/src/input.rs). Existing
source-ordered initialization, nonempty Value-to-Default writeback, fresh reset,
cloning and actual content readback use the same controlled parser owner. Absent
value supplies empty; present empty stays empty. No display label, activation,
submission, form-wide reset event or new upstream source is introduced.

The Radio extension adds Surgeist-owned shared state rules and admitted peer
effects in `../surgeist-html/src/input.rs` and `src/construction/radio.rs`.
No additional upstream parser source is ingested. The selected HTML
[radio-button group clause](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#radio-button-group)
governs exact name/tree/form-owner membership and the five qualifying triggers.
The existing effect owner retains one fixed traversal phase, including an honest
partial phase on cancellation or exhaustion. Attribute edits and removal reuse
`src/construction/mutation.rs`, extracted from the existing
Details owner; provenance retirement and rebinding retain their original
authority. Captured peers without mutable canonical state and influencing
captured connection facts remain qualified refusals. This construction adapter
does not supply DOM identity, live mutations, browser activation or events.

The Email/URL extension is Surgeist-owned shared value interpretation in
`../surgeist-html/src/input.rs`, using the same lenient comma-token owner as `src/tokens.rs`.
The selected HTML URL and Email state sanitizers govern ASCII trimming,
newline handling and actual `multiple` presence. The parser adapter charges
the borrowed cursor and actual source/origin copies before advancing. Produced
Email list counts remain stored through freeze and clone; they are not inferred
by splitting the sanitized string. Original default attributes and source
histories remain independent. No additional upstream source is ingested, and
address/URL validation, live editing and external execution remain separate.

The five-state temporal extension is Surgeist-owned. The shared borrowed strict
cursor in [temporal.rs](../surgeist-html/src/temporal.rs) is consumed by both ordinary component
recognition and the [input sanitizer](../surgeist-html/src/input.rs); the existing numeric
one-digit remainder kernel supplies arbitrary-year calendar decisions without
machine-integer materialization. The selected
[input clauses](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)
govern invalid-to-empty Date/Month/Week/Time/LocalDateTime current values and
LocalDateTime's shortest `T` spelling. Broader time/local recovery stays separate.
The existing construction adapter admits each cursor action and actual copy
before publication. Native defaults, current backing, clone flags and reset
remain separate; no alternate temporal parser, controller or payload registry
is introduced. The tokenizer origin vocabulary and existing provenance validator
record an actual SPACE-to-`T` transformation as InputValueNormalization with its
original source interval, rather than raw equality or a guessed lexical range.
Completed public readers borrow that same admitted evidence. No additional
html5ever source or language edition is incorporated. Stepping, live setters,
pickers, time zones, validation and allocator/RSS guarantees remain outside this
bounded extension.

The structural table adaptation incorporates `rules.rs` lines 1037–1411 and
`mod.rs` lines 1236–1320 into `src/parser/tree_builder/tables.rs`, with table entry
in the body driver. Its initial adapted SHA-256 is
`60453ffe30a328824744e7a0dd92419204d25bc1584b35c5a6f5bde484acbba0`.
Earlier incorporation hashes remain historical. Original source notices are
retained. The selected HTML edition governs the pending-character current-node
set, including template, and the tbody/thead/tfoot section predicate rather than
the upstream table/tbody/tfoot predicate. Processing instructions use actual
native insertion. Hidden-input dispatch checks its type and qualified creation
capability before the table diagnostic, then uses direct table insertion. Other
unavailable control creation retains a pre-effect continuation.

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
Unsupported control branches retain the original token before unavailable effects. This checkpoint does not certify complete
InTable behavior or complete parser resource accounting. Contextual execution
and report assembly use their separately qualified owners. Ordinary EOF uses the
separate
[completion owner](#ordinary-eof-completion).

The ordinary frameset extension adapts `rules.rs` lines 378–383, 458–475,
1494–1563 and 1584–1608 into `src/parser/tree_builder/frameset.rs`, with entry
dispatch in the existing body/head driver. Its initial adapted SHA-256 is
`f267a94de78049ed4371419a2a51bb4382dfce3144353dd809d43767643b7b49`.
The complete original notice remains verbatim; earlier incorporation hashes
remain historical. The pinned HTML edition supplies processing instructions,
exact character-origin handling and the frame postconnection null-browsing-context
return. The existing sealed fresh standalone capability establishes that actual
node-document state; generic and imported owners stop before frameset effects.

InBody entry diagnoses before its body/eligibility guards and uses canonical
body removal effects without clearing the current-form pointer or formatting
list. Nested framesets and immediate frame pops use the existing stack.
InFrameset and AfterFrameset insert whitespace directly; AfterAfterFrameset
delegates it through ordinary body reconstruction. The shared character-run
owner preserves mixed-run source ranges and NULL/nonwhitespace diagnostics.
Noframes uses the existing RAWTEXT handoff and restores its actual caller mode
at end-tag or EOF. These operations retain one controller, arena and terminal
cleanup path. They do not supply hosted frame navigation. Ordinary EOF completion
is owned separately
[below](#ordinary-eof-completion).

The ordinary select extension adapts the input/hr guards and
select/option/optgroup rows in upstream `rules.rs` lines 815–862 and 895–945
into the existing body driver. Its initial adapted body SHA-256 is
`2616bdf7661339e26da356c082942b453fcaa9b3b3b9e5c6d805e38fa2dc336c`;
earlier incorporation hashes remain historical. The seven existing tree-builder
notices remain verbatim. The selected HTML edition supplies end-select and
selectedcontent dispatch and their current intrinsic algorithms; these are
Surgeist-authored normative adaptations, rather than additional upstream
insertion modes.

The existing sealed standalone capability is borrowed through actual parser
stack removals, formatting recovery, table delegation and native topology
operations. The same controlled construction owner now retains select-family
metadata, finite effects, static lists and FIFO tasks. It uses actual DOM
pre-insert/remove/replace-all and clone clauses from the separately recorded
[DOM source capture](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md).
Its iterative managed cloning preserves original source maps and gives every
clone independent identity/backing. Source-defined connection and
children-changed hooks use the existing staged Disabled script-preparation
owner; disabled external execution does not suppress these intrinsic effects.
Display-size consumes the existing numeric grammar through a shared per-unit
scanner, retaining the public author's and interpreter's existing semantics.

Tasks drain only at source-defined checkpoints. Pending work prevents freeze
before storage is taken; terminal cleanup retains and releases all partial
owners without useful resets or cloning. Fragment insertion completes its
sibling batch before parent children-changed and static post-connection work.
Actual string binding writes are admitted before clone allocation and committed
immediately afterward, retaining strict alias validation on stopped prefixes.
These adaptations remain internal production operations. They add no second
controller, public DOM API, generic moveBefore primitive, imported cloning with
unavailable template/shadow facts or external execution. Qualified context
initialization and report publication use their separate owning boundary.
Ordinary EOF now runs its two real checkpoints through the
separate [completion owner](#ordinary-eof-completion). Finite admission does not
establish linear complexity or
complete allocator/RSS coverage.

The ordinary non-script foreign adaptation incorporates `mod.rs` lines
1605–1692, 1694–1841 and 1837–1883 and `rules.rs` lines 971–979 and 1608–1688.
The original complete file hashes above identify these additional slices.
`src/parser/tree_builder/foreign.rs` owns dispatch/recovery;
`src/construction/foreign.rs` owns controlled adjustment and
creation. Both retain the original eight-line notice verbatim. Existing head,
body, table and tree-kernel files retain their notices; earlier incorporation
hashes remain historical. Qualified cloning and namespace/prefix ticket binding
extend the existing Surgeist-owned session registry, rather than importing a
second sink or construction arena.

The selected HTML edition supplies processing instructions, exact UTF-16,
namespace-driven CDATA at integration points, and encoding-qualified
annotation-xml breakout guards omitted by the older implementation. Creation
retains original lexical values/maps plus independent adjusted values/maps;
prefix removal rebases local-name origins and recovery adjustments retain their
source ranges. Namespace and prefix literal backing are admitted and bound to
actual native fields. Clones preserve every qualified value and independent
identity/backing through the same controlled attribute transfer owner.

Foreign dispatch precedes the current HTML insertion mode and reprocessing
returns to that actual mode. MathML text exceptions, SVG and annotation-xml
integration, self-closing acknowledgement, NULL replacement, comments/PI,
namespace diagnostics, HTML breakout and mismatched-end recovery use existing
insertion, stack and cleanup owners. The separate
[restricted SVG extension](#restricted-svg-script-extension) now supplies its
qualified processed state and foreign-end route. No new dependency or external
execution domain is introduced. All original/adjusted backing, maps,
qualified clone slots, writes and variable scans are admitted; failures retain
owned prefixes and refuse useful continuation/freeze. Repeated registry and
topology scans may be quadratic. This internal production machinery supplies no
complete allocator/RSS guarantee.

Unsupported construction branches and unavailable context effects remain
explicit refusals. The upstream driver, encoding implementation, serializer,
XML parser and RcDom are not incorporated here. Research prototypes and their
measurements do not certify this adapted implementation or supply a public
parsing contract.

## Ordinary-template extension

The new `src/parser/tree_builder/template.rs` adapts upstream `tree_builder/rules.rs`
lines 257–307 and 1403–1462 at the same html5ever revision. Its initial adapted
SHA-256 is
`b2aebebb1877a12ff759329b0b6c81e27b1948e12e679ba46ca383d0ad9c2ad2`.
The complete original eight-line notice remains verbatim. Existing insertion
routing, mode reset and shadow-permission factoring additionally derive from
tree_builder/mod.rs lines 417–450, 1265–1313 and 1463–1505. Complete original
rules/mod hashes in the origin table remain unchanged, and all earlier adapted
checkpoints remain historical. Pinned HTML controls PI tokens, for patching,
exact expanded names, permission-false ordinary-template routing and end/EOF
effects rather than older sink defaults.

The new [session template owner](src/construction/template.rs) and
[PI-map owner](src/construction/pi.rs) are Surgeist-owned
implementations of selected HTML/DOM and delegated W3C algorithms, not newly
copied html5ever files. Qualified Document records, the cached appropriate inert
owner document, separately hosted contents and nullable target/start/end metadata
share the existing registry. Borrowed CreationContext identifies the actual open
template. Creation-context element metadata is admitted before its lexical transfer/native
creation. Generic owned-token preparation and independently owned replay copies
retain explicit terminal-safe cleanup boundaries.

Adjusted placement resolves foster routing first, then actual template contents
or patch target/end-marker state. The real template insertion-mode stack
participates in reset, InTemplate dispatch and reprocessing. Explicit end
generates thorough implied ends and finishes current patch markers before
stack/formatting/mode unwinding; EOF unwinds without those explicit-end marker
effects or an artificial task drain. The fresh constructor establishes shadow
permission false, so declarative shadow syntax follows the source-required
ordinary-template branch before for patching. No shadow attachment is implemented.

Source-defined adoption runs after complete removal effects and before insertion.
Contents adopt into the appropriate contents document; iterative cloning creates
independent template/contents identities and managed backing, uses the copy
contents' real document and preserves source child order. Script metadata uses
actual node-document identities. Eligible disconnected template-content scripts
stop at Disconnected after parser-document/force-async and empty-source prefix
effects, before classification or the scripting-disabled return.

Patching reads actual PI targets and complete maps, searches ordinary descendants,
retains the complete sibling removal snapshot, then performs source-ordered
effects. Whole-map syntax errors and terminal resource stops stay distinct.
Original PI target/data and lexical evidence remain separate from admitted decoded
backing. The [whole XML-stylesheet §3](../../references/xml-stylesheet-pseudo-attributes--REC-xml-stylesheet-20101028--a0e1a5198a21.md#pseudo-attributes)
and [XML Char/S/Name/CharRef/notation closure](../../references/xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md)
have separate W3C document-use provenance; WHATWG terms do not cover these origins.

Defining selected HTML clauses are retained in
[syntax](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md)
(3439–3539, 3669, 3984–4126 and 5500–5578) and
[template documents/state/adoption/cloning/patching](../../references/html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-template-element)
(3013–3157, plus script preparation 2106–2174). The retained
[DOM capture](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md)
supplies cloning/adoption (1667–1710 and 2401–2437) and PI initialization/update
(4023–4053/4083). These are converted-reference line locators, not runtime source
indices or complete original-file hashes.

Freeze refuses pending intrinsic work or unavailable template/document/PI facts
before transfer. A terminal stop retains its actual effect/task/backing owners
for disposal without useful continuation. Repeated registry/topology scans and
copies/moves can be quadratic; finite charged/polled work is not a linearity or
allocator/RSS guarantee. This remains internal fresh ordinary construction.
Proofless head dispatch still suspends at script/template starts; imported
unavailable template facts and live shadow attachment remain outside it.
Qualified contextual execution and completed report assembly are separate owners.
No new dependency, upstream driver, serializer, Tendril/atom implementation,
XML document parser or RcDom is incorporated.

## Restricted SVG script extension

The existing html5ever-derived foreign dispatch and creation files retain their
complete original eight-line notices and original rules/mod hashes above.
No additional upstream source file or slice is ingested at this checkpoint.
At the pinned implementation revision, rules.rs line 1651 and mod.rs line 1853
explicitly leave SVG end and self-closing processing as FIXME gaps. Surgeist's
new source-ordered wrapper in the existing head/foreign drivers fills those gaps
from the [pinned HTML foreign clauses](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#scriptForeignEndTag),
rather than attributing the implementation to absent upstream SVG processing.
All earlier adapted hashes remain historical incorporation checkpoints.

The new [session SVG child](src/construction/svg.rs) and the
registry, insertion-cause, clone and static postconnection integrations are
Surgeist-authored adaptations. Their behavioral sources are the existing
HTML/DOM captures and the separately attributed
[SVG Tiny 1.2 processing clauses](../../references/svg-tiny12-script-processing--REC-20081222--2f36eb76440c.md#ScriptContentProcessing).
The [selected source determination](../../references/SOURCE-EDITIONS.md#offline-disabled-svg-script-and-clone-sources)
preserves Tiny step 1 and the connected step-3 processed mark while a separate
closed no-loader capability suppresses all acquisition and Disabled suppresses
execution. Actual absence of host/listener/observer/retained notification
producers permits notification elimination. This is neither a literal Tiny
Disabled return nor a successful/failed resource load. HTML preparation rules
and the intrinsic FIFO checkpoint are not applied to SVG.

Exact namespace/local-name and owner/document qualification distinguish
unavailable state from false. Fresh SVG metadata slots are admitted before
lexical transfer; binding writes are admitted before native allocation. Later
owned transfer work retains explicit terminal-safe cleanup boundaries.
The canonical insertion command carries actual Parser/NonParser cause through
the existing effects: parser insertion defers to foreign end, while nonparser
connection processes the existing ordinary static list in document order.
Template contents remain detached ordinary roots; adoption updates real
node-document bindings before final connection, without resetting the bit.

Managed cloning qualifies source SVG state through the existing whole-closure
preflight and copies its true/false bit into a distinct clone before insertion.
The [WebKit witness](../../references/webkit-svg-script-clone--73aa6c89e2cb-metadata.md)
selects that bounded standards-gap completion only. No Library-GPL source
excerpt, combined WebKit HTML/SVG algorithm or executable implementation is
incorporated. Existing HTML script clone/preparation behavior remains separate.

The retained foreign-end wrapper uses the same tokenizer insertion point,
nesting/pause state, arena and controller; it releases the actual lexical
payload and does not drain intrinsic tasks. Pending wrapper/effect work blocks
useful entry and freeze. Actual registry/vector capacities and fixed layouts
stay charged until disposal; terminal cleanup preserves first stop and performs
no rescue processing. Scans, clone/adoption walks and moves may be quadratic.
This internal production checkpoint supplies no complete allocator/RSS or
linear-work promise. Separate
[W3C attribution](../../licenses/w3c/NOTICE.md#html-delegated-svg-script-processing)
and [crate attribution](NOTICE.md#restricted-svg-script-processing) cover Tiny
algorithm incorporation; WHATWG terms do not replace those W3C terms.

## Ordinary EOF completion

The [private ordinary completion](../../docs/reference/html-operations.md#private-ordinary-eof) follows the
pinned HTML end-of-parsing and readiness clauses under the
[separately recorded source selection](../../references/SOURCE-EDITIONS.md#fresh-standalone-eof-lifecycle-sources).
The complete [spin algorithm](../../references/html-spin-event-loop--877d6146487f.md)
and existing checkpoint source retain the normative ordering; the closed
construction reduction does not attribute its no-host eliminations to a
normative early return. Earlier adapted source hashes remain historical
incorporation checkpoints, not hashes of the extended current files.
The new [end owner](src/parser/tree_builder/end.rs) and
[completion fact owner](src/construction/completion.rs) are
Surgeist-owned implementations of that selected reduction. No new third-party
implementation file or source slice is ingested; existing original notices and
source hashes remain unchanged.

Fresh ordinary construction retains a qualified source-initial Complete
readiness fact, with post-load readiness false and parsing unfinished. After
actual EOF recovery, Interactive precedes canonical pops and their real effects.
Two separate intrinsic checkpoints preserve the source wait positions.
The fixed Complete-readiness continuation runs after the outer post-load
milestone, which is committed while readiness remains Interactive.
Constructor and admitted-producer closure qualify absent external domains;
no browser event, global queue, timing value or deferred replay record is added.

The completed private owner retains the whole same driver/control/arena,
diagnostics, lexical events, maps and capacity ownership for report assembly.
A rejected or interrupted move keeps actual owners available for terminal
cleanup; cleanup performs no useful readiness progression, canonical pops or
task draining. Compiled fixed layouts and real vector capacities remain with
the existing controller. Finite admission does not establish linear complexity
or complete allocator/RSS coverage. The
[public request/report boundary](../../docs/reference/html-operations.md#decoded-unicode-parsing) consumes these
owners for supported document/fragment profiles; it does not establish complete
HTML construction, host lifecycle or enclosing contribution acceptance.

## Ordered private parse evidence

The [private diagnostic checkpoint](../../docs/reference/html-operations.md#private-ordered-diagnostics)
retains actual error invocations from the pinned HTML Chapter 13 algorithms.
Structured condition codes, current tokenizer/prologue/insertion state,
recoverable severity, the selected rule anchor and an intentional engine branch
replace upstream prose as the machine identity. A location labels its actual
input frontier, token envelope, text origin or delayed table-character origin;
an envelope is not a precise output-field or immutable-node extent.

The new [parse-evidence owner](src/construction/evidence.rs),
paired record/location admission in the existing
[controller](src/parser/control.rs), and the completed/stopped consumers in the
[end owner](src/parser/tree_builder/end.rs) are Surgeist-owned machinery.
Existing adapted tokenizer and tree-builder emitters now append to this single
construction-session store at actual emission. Tokenizer events retain
qualified receipts rather than a second diagnostic list. Phase-specific
cfg-test views are temporary projections, not parallel evidence owners.
No new third-party implementation file, source slice or language edition is
incorporated; original notices and historical adaptation hashes remain intact.
The HTML pin continues to govern language behavior, while the retained
html5ever revision identifies implementation provenance.

Both capacities, logical slots and fixed writes precede pair publication.
Actual source/controller qualification, physical capacity retention, sticky
first stop and explicit disposal remain with the sole parser owner. Completed
observation borrows or moves the same backing. The private stopped result keeps
only committed evidence, a borrowed original source, first stop and fixed usage
snapshots after native/lexical reclamation. The public terminal adapter retains
the original evidence backing with the original request rather than exposing a
partial tree. Cleanup performs no useful continuation or diagnostic emission.
Alias refusal leaves retained owners available for a cleanup retry.

Selected literal tests cover delayed chronological emission even when offsets
go backwards, rule/location identity, the pre/listing self-closing descriptor,
qualification and admission/cleanup boundaries. They do not certify complete
diagnostic coverage, corpus error metadata, native-field/output provenance,
reverse lookup or feed invariance. Qualified public report/readback and
contextual execution are described by their own boundary, not certified by these
selected tests. This Surgeist-owned production store adds no attribution material,
linear-work or complete allocator/RSS guarantee.

## Public source-owning parser boundary

The [request adapter](src/parser/tree_builder/parse.rs),
[report assembler](src/parser/tree_builder/report.rs) and
[public read adapters](src/construction/read.rs) are Surgeist-owned.
Production compilation exposes the existing tokenizer, construction session,
completion, freeze and evidence owners through synchronous decoded-Unicode
document/fragment requests. No upstream driver, second parser, decoding library,
DOM implementation or new language edition is incorporated by this promotion.
The immutable report moves the original source and completed storage; terminal
failure retains the original request and safe evidence without a partial tree.
Public readers borrow fixed metadata, diagnostic and association records and
qualified history rather than copying a parallel provenance schema. Internal
helpers remain private; conformance observers, expected-data decoding and feeding
fixtures remain test-only. The [overview](../../docs/reference/html-operations.md#decoded-unicode-parsing) records
supported profiles, explicit limits and exclusions. Original notices, source pins
and historical incorporation hashes continue to identify their original inputs.

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
conformance, unsupported tree effects and complete resource accounting remain
obligations of the enclosing parser contribution. Public report availability is
described separately from those acceptance limits.
