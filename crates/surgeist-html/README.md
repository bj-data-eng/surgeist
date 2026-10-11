# surgeist-html

This crate owns the pinned HTML language foundation. Its initial public surface
provides immutable exact UTF-16 strings, namespace identity, qualified names,
ordered raw attributes, native immutable document/fragment construction,
known-name inventories, common intrinsic value operations, bounded immutable
captures of HTML semantic facts and checked parser request configuration.
The [HTML contribution](https://github.com/bj-data-eng/surgeist/issues/1132)
tracks the remaining typed vocabulary/value interpretation, parser, semantic
interpretation, provenance and full parser resource-admission capabilities.

The [selected WHATWG edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
defines behavior; the separate html5ever revision identifies implementation
provenance. Shared values apply no language-specific normalization. A Rust `str`
converts losslessly to `DocumentString`; conversion back to UTF-8 rejects isolated
surrogates. UTF-16 construction accepts a vector or a borrowed slice. An owned
vector transfers into immutable shared ownership without copying its variable
payload; a borrowed slice first creates an owned vector. Fixed shared-wrapper
allocation remains subject to the standard allocator, independently of parser
resource admission. Raw names and attributes preserve supplied values for their eventual
owning language or DOM validation boundary.

The crate is independently usable and has no DOM or rendering dependency. DOM
will consume these shared values directly and own canonical live identity,
mutation, captures and HTML integration. Parser construction values do not form
a second live DOM. Template syntax and symbolic expressions remain in
`surgeist-template`.

`HtmlSource` identifies one supplied already-decoded Unicode input. Its validated
half-open ranges use supplied UTF-8 bytes or supplied UTF-16 units. Equal text
does not imply equal source identity; cloning a source retains its identity.
UTF-8 ranges require scalar boundaries; UTF-16 ranges can select individual
surrogate units. Source identity supplies no document URL, base URL, origin or
permission. This source contract performs no parsing or preprocessing, and
preserves literal U+FEFF and CR/LF units.

`HtmlDocument::builder` and `HtmlFragment::builder` create bounded native
authoring sessions. The caller supplies every construction capacity explicitly:
nodes, edges, attributes, stored UTF-16 units and depth. Roots and separate
template/shadow fragments participate in admission. A rejected append leaves
content and admitted usage unchanged; consuming `finish` freezes one immutable
flat arena. The storage does not recursively own child nodes.

Traversal preserves mixed sibling order and exact native values without
serialization or reparsing. Node IDs are qualified by their immutable result;
foreign-result lookup returns a typed error. Template contents and native shadow
contents have separate fragment roots and reverse owner associations. Native
processing-instruction append accepts `HtmlAuthoredProcessingInstruction`,
whose checked constructor owns the selected target/data syntax. The shared
`HtmlProcessingInstruction` stored payload preserves exact data; converting it
to authored input explicitly revalidates it. Raw shared names remain available
without generic DOM name validation.

These native construction artifacts perform no parsing. They do not claim parser
metadata, context effects, diagnostics, source mapping, full content-conformance
checking or script execution. Their capacities count logical retained content;
they do not establish a universal process-memory or CPU bound. General shadow-host
conformance and live installation remain separate admission concerns.

`KnownHtmlElement` and `KnownHtmlAttribute` recognize the modern index's exact
namespace/local-name identities. The shared raw records retain unknown/custom
and foreign names. Element metadata preserves grouped heading names and foreign
or open-family rows; attribute metadata preserves context-specific rows and
links to defining clauses. The printed index summaries are discovery material,
not complete value grammars, semantic defaults or execution-owner decisions.

The source inventories contain 111 element rows (113 named HTML elements after
expanding the heading group) and 180 contextual attribute rows (144 distinct
local names). Event handlers, obsolete names and open attribute families require
their separate source clauses. Regenerate or check the ordinary vocabulary source
offline with the standard-library-only tool:

```sh
python3 crates/surgeist-html/tools/generate_vocabulary.py --write
python3 crates/surgeist-html/tools/generate_vocabulary.py --check
```

The [incorporation notice](../../licenses/whatwg/NOTICE.md#html-vocabulary-incorporation--2026-10-11)
identifies the exact edition, upstream license and source adaptations.

Numeric authoring preserves arbitrary integer magnitude and exact decimal
dimensions. Machine-integer projections report range failure. Floating-point
values use finite binary64 with the selected HTML rounding contract; native
decimal output does not claim ECMAScript's separate best-representation
operation. The dimension-list algorithm applies only the
[documented decimal-cursor correction](../../references/html-dimension-list-cursor--webkit-73aa6c89e2cb-blink-7984f9d11800.md).
Other browser overflow or syntax differences are not adopted.

Temporal authoring checks positive Gregorian years, dates, times, explicit
offsets, ISO weeks and durations. Years and whole duration seconds have no
machine-integer magnitude cap. Time and duration author values use millisecond
precision, matching the strict author grammar. A global date/time retains its
explicit local value and offset; it does not perform an implicit UTC conversion.

Temporal recovery returns separate parsed values where the source accepts a
wider domain. Exact decimal seconds retain arbitrary admitted fractional
precision; projection to millisecond author values fails when that value cannot
be represented exactly. Parsed global values retain the supplied local time and
offset alongside their computed UTC instant. The UTC day coordinate is signed,
with day zero at Gregorian `0001-01-01`; this representation permits an offset
to carry recovery into the preceding day without extending HTML's author-year
grammar. No host clock or time zone is inferred.

Native date, month, week, parsed time and parsed local date/time values provide
exact input coordinates. Month uses signed whole months from January 1970;
the other states use their specified millisecond coordinates. Significant
fractional digits and arbitrary years survive conversion, including pre-epoch
local fractions. Each result identifies its state, source clause, units, step
scale, default step and base; Time retains its periodic-domain designation.
Scaling a recovered positive finite step preserves its actual binary64 value
exactly before mathematical multiplication. These native methods use ordinary
allocations without cancellation or allocator-failure recovery. The attribute
constraint reports below consume these native quantities. Current validity,
inverse conversions and stepping remain pending.

Token authoring preserves exact spelling and order. Unique ordered or unordered
sets require an explicit case-sensitive or ASCII-insensitive comparison.
Comma tokens can be empty and retain internal whitespace. These common values
do not certify an attribute's keywords, cardinality or contextual constraints.

`HtmlValueInterpreter` admits each raw value against a caller-selected finite
UTF-16 input capacity. Its immutable `HtmlValueReport<A, R>` reports distinguish
missing values, whole-value author conformance, representation failure and
separate recovery outcomes. Numeric, temporal and common token processing use
their pinned algorithms. The date-or-time operation defines processing without
an independent author grammar. Date-with-optional-time checks its Date/Global
author grammar while retaining the common parser's standalone Time result;
attribute-specific timestamp filters remain with their consuming clauses.
Raw values remain unchanged, including isolated surrogates. Attribute
applicability, missing/invalid/empty defaults and effective states follow their
individual defining clauses.
This standalone input cap is not full parser accounting, cancellation or an
allocation-failure guarantee. The parser contribution owns those controls.

`HtmlLegacyColor` and the controlled legacy-color operation implement HTML
§2.3.6 recovery over exact raw values. Results are opaque sRGB channels and
serialize as lowercase `#rrggbb`. The pinned edition defines no simple-color
author grammar; the report therefore distinguishes this recovery from author
conformance and the color-input state’s external CSS grammar. The existing
external cssparser package supplies only the standard named-color lookup,
without a dependency on surgeist-css or invocation of CSS tokenization.
See the [dependency notice](NOTICE.md) and [source selection](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline).

`HtmlAttributeReport` separates exact presence, expanded-name applicability,
keyword recognition, lexical author conformance, selected contextual author conformance
and the selected intrinsic state. The
implemented overlays cover twelve global enumerated attributes, template
shadow mode/slot assignment, eleven element-specific keyword rules and
element-specific Boolean attributes. Input types retain all twenty-two standard keywords and their
Text fallback; button types retain the distinct Auto fallback without treating
it as an authored keyword. Media preload retains implementation-defined missing/invalid
policy separately from its recognized keywords; ordered-list marker types use
exact case comparison, and track kinds retain their distinct missing/invalid
defaults. It preserves invalid
spelling when a clause supplies a fallback; keyboard hints retain their lack of
a defined fallback. Native keyword values construct their canonical raw
attributes. CORS, fetch-priority and referrer-policy operations retain their finite intrinsic
states. Script operations require explicit classic/module/inline/external or
non-script type context. For `a` and `area`, referrer-policy reflection retains
its intrinsic state even when `href` is absent or its presence is unknown.
Supplied absence records the separate authoring violation; unknown presence
leaves that contextual check unchecked. Inapplicable contexts remain distinct
from invalid spelling.
Disabled-attribute interpretation also covers fieldset, optgroup, option and
the four standard form controls. Custom controls require an explicit
form-associated classification. This intrinsic Boolean presence operation
does not itself compute disabledness inherited from ancestors.
Boolean operations also cover input attributes, submitter
`formnovalidate`, script `async`/`defer`/`nomodule`, image `ismap`/`controls`,
and stylesheet-link `disabled`. Correct native roles retain content presence
across prohibited supplied states; the report records their authoring
restrictions separately. State-specific ignore rules govern applied processing.
The `async` operation observes the content attribute, without inferring the
script's force-async flag or implementing its complete IDL getter.
Image and link authoring restrictions remain
separate from their content-presence state: an applicable present Boolean stays
true when its required ancestor, alternate text or stylesheet relation is
missing. The report can retain both a lexical and a contextual violation.
Supplied input, script, submitter, custom-control, image and link contexts select
only their defining clauses; they do not establish current control state,
resource lifecycle or activation effects.
Referrer policy uses the [pinned external vocabulary](../../references/referrer-policy-vocabulary--cc435b05ca4a.md);
its empty state remains unresolved rather than selecting a host network default.
These views do not compute inheritance or claim runtime effects,
and the remaining attribute families are tracked by the contribution.

Numeric attribute reports cover canvas width/height, textarea cols/rows and
tabindex content values. They preserve whole authored syntax and prefix recovery
separately from each attribute's selected default: canvas defaults to 300/150
while accepting zero, and textarea defaults to 20/2 when recovery fails or yields
zero. Tabindex retains an exact signed integer or source-null; its full signed-long
IDL getter and element-specific focus defaults remain pending. Formmethod and
formenctype overrides reuse the defining keyword rules and separate submitter
author restrictions from their intrinsic values. A missing override retains no
selected value; the captured form-setting queries below use the actual form
owner for effective fallback. These content operations do not allocate canvas surfaces, lay out text,
move focus or submit a form.

Input size and input/textarea length reports retain exact recovered integers,
whole-value authorship and actual input-state applicability. A missing, invalid
or zero input size selects no positive content advisory; its reflected getter
defaults separately to 20. Missing or invalid length values select no bound;
their reflected getters separately return −1, including for exact values beyond
the signed-long range. Bounds and minimum≤maximum checks preserve arbitrarily
large values. Supplied initial input `value` or textarea descendant text supports
the separate maximum-length author check, with textarea newline normalization
and UTF-16 code-unit counting. Unknown control state stays unresolved. These
operations do not compute runtime tooLong/tooShort, rendering or IDL setters.
Raw admission is per supplied value; these intrinsic reports do not promise
allocation recovery, work accounting or cancellation.

Numeric and temporal input constraints keep each raw min, max, step and
value-content observation separate from grammar evidence and converted quantities.
Number has no default bounds; Range falls back to 0/100. Their author relation
compares effective finite binary64 limits without repairing an inverted pair.
Date, Month, Week, Time and LocalDateTime retain their actual typed common reports
and exact state-qualified month or millisecond coordinates. Missing or failed
temporal limits supply no bound. Time preserves a reversed periodic range
separately from author conformance; the other states compare ordered limits.

Step uses the shared floating-point parser. Positive temporal steps are scaled
from the actual recovered binary64 value into exact decimal quantities; missing,
failed or nonpositive steps select the state's default, while exact `any`
selects no allowed step. Base selection uses successful min content, then value
content, then the source default: Week uses −259200000 milliseconds, other states
use zero. A fallback Range minimum does not displace value content. Number and
the temporal states permit empty authored value content, while Range does not;
no empty value supplies a converted quantity or base. Strict temporal authorship
and successful processing remain separate, including recovered significant time
fractions beyond strict author precision. These observations perform no
current-value sanitization, periodic membership, constraint validation or
stepping. Raw admission is per supplied value; native conversion retains its
ordinary allocation contract.

Control autocomplete reports preserve ordered tokens, authored detail grammar,
actual input-state and element constraints, and the separate processing result.
Native detail construction requires a standard field and checks contact hints;
actual control suitability remains a contextual check. Processing retains exact
field and IDL token casing while normalizing the source-defined hints and scope.
Credential outcomes distinguish a setting branch that assigns no property from
an explicit null or credential assignment, without asserting previous live state.
The [selected credential-edge determination](../../references/html-autocomplete-webauthn--877d6146487f.md)
retains the pinned standard's recovery for bare and prefixed WebAuthn tokens,
independently of author validity. Default reports expose empty IDL/hints/scope
and distinguish an anchor's empty field from an expectation default requiring
the actual form owner. Capture-qualified semantic queries resolve that fallback;
missing owner evidence is not interpreted as no owner. These reports execute no autofill,
credential mediation or form-control transition.

Input `accept` authoring retains ordered wildcard, leading-dot extension and
symbolic MIME specifiers with ASCII-insensitive uniqueness checks. Extensions
follow HTML's leading-dot rule without a filename or registered-type lookup.
Interpretation separates authored fields from literal comma-split hint tokens,
including terminal empty fields. Only the actual File state exposes those hints;
other input states retain the separate presence restriction and ignored effect.
Symbolic MIME conformance remains unavailable, independently of established HTML
constraints. These operations perform no file picking, type matching or sniffing.

Iframe `sandbox` authoring uses the thirteen standard keywords and the shared
unordered unique-token owner. Interpretation preserves exact raw tokens and
independent duplicate, unknown and redundant-pair diagnostics while computing
the [local directive's sixteen restriction flags](../../references/html-sandboxing--877d6146487f.md).
A missing attribute is distinct from a present empty directive, which retains
all sixteen restrictions. Recognized keywords contribute even when authorship
is nonconforming. These flags do not establish a loaded document's active
policy, ancestor/CSP composition, navigation, origin changes or enforcement.
The standalone per-value raw cap applies; work accounting, cancellation and
allocation-failure recovery are not promised by this intrinsic value report.

`HtmlCaptureBuilder` freezes one bounded, flat capture of document, node,
registry and relationship facts. Capture-qualified subjects are independent of
native result IDs and future canonical DOM IDs. Unavailable facts remain
distinct from known absence; lookups and traversal borrow the captured records.
Every retention capacity is explicit, and rejected changes preserve logical
content and usage. Document self identity is covered by the subject count;
relationships count references to other retained records.

`HtmlSemanticSession` borrows one capture and a matching `HtmlSemanticContext`
to answer hidden-state, disabledness, language, directionality, list, table,
meter, progress, inertness, form-owner, labeled-control, current control-flag,
editing-host, editable, isContentEditable, translation-mode, spellcheck IDL and
computed-heading-level queries. Disabledness includes
the fieldset’s first-legend exception and the option/optgroup/select rules;
custom controls use the supplied form-associated classifications. Results
distinguish known values, inapplicable subjects, missing captured facts and
unsupported cases. The session admits finite work, output, scratch, integer-digit,
allocation-byte and nested-output capacities, observes
cancellation and returns a complete batch or a sticky terminal failure.
Reports and admitted query failures retain the same immutable qualification:
HTML revision, capture, context, host-facts revision and selected limits.
Constructor rejection has no admitted-session qualification. Dependency results
currently request conservative recomputation. These operations borrow captured
relationships without constructing a live tree. Remaining control/form
algorithms and state transitions remain part of the contribution.

`HtmlSemanticContextInput` groups supplied classifications and optional qualified
supplements. The checked context joins them to the exact capture and host
revision. `HtmlLanguageDirectionFacts` retains processed document-language
fallbacks, actual current control values and ordered assigned-slot nodes.
Its finite retention and validation caps are explicit. Missing facts remain
unavailable; known empty assignments are distinct from missing assignments.

`HtmlInertFacts` joins actual flat-tree membership, parent relations and
connectedness to the document's actual blocking modal dialog. The producer owns
flattening and modal transitions. Only the topmost blocking dialog escapes
inherited inertness; an element's own inert attribute still applies. Unavailable
or excluded ancestry remains missing evidence unless a local fact proves the
result. The supplement and query share finite validation and session work
limits, and reports retain its exact producer/configuration revision.

`HtmlControlFacts` retains established current form pointers or explicit
reset-against-capture evidence, plus independently optional current input, option
and textarea flags. Form-owner queries preserve parser-established associations
and distinguish them from a reset computed against this capture. Label targeting
uses actual tree order and labelable classification. Known false flags remain
distinct from unavailable slots; attributes do not substitute for current state.
These operations observe facts and execute no form reset, activation or control
state transition. Actual control strings remain owned by the language/direction
supplement.

Select/option readback exposes the source-defined option list, selected options,
selected index and option index. One bounded ordinary-tree traversal preserves
tree order and the specific option, datalist, hr, nested-select and optgroup
pruning rules. Selectedness comes from the existing current-state supplement;
authored selected attributes, disabledness and multiple do not replace it.
Collection getters expose capture-qualified subject keys. A known empty selected list
and no selected index differ from unavailable state or order. These immutable
results provide no live collection or selection/reset transition.

Option text, label and value getters expose exact immutable strings and their
selected source. Present label/value attributes, including empty values, are
returned unchanged. Descendant fallback uses HTML-aware text: HTML/SVG script
and HTML img subtrees are excluded, and only ASCII whitespace is stripped and
collapsed across text nodes. BR adds no separator; other UTF-16 units remain
exact. The label getter follows IDL readback, rather than the separate rendering
label algorithm. Select value reuses the source-defined option list and returns
the first currently selected option's value. Its result distinguishes a selected
empty value from a known absence of selection. Consulted attributes, traversal,
temporary scratch and actual output capacities use the semantic session's
admission and atomic batch contract; missing influencing facts remain explicit.

The control-labels query returns the complete associated label list in ordinary
tree order. Hidden inputs produce known null; a complete applicable empty list
differs from unavailable evidence. Exact first-ID and implicit-descendant rules
reuse the existing labeled-control owner, with irrelevant labels excluded before
requiring unrelated custom-control facts. Form-associated custom controls use
their supplied classification. The result is an immutable association snapshot;
it does not attach ElementInternals or provide a live NodeList. Traversal and
nested output retain the semantic session's admission and atomic batch contract.

The control-autocomplete query retains intrinsic syntax and processing, the
actually consulted input-type report and any consulted form-autocomplete report.
Only expectation defaults consult the existing form-owner operation: a known
null owner selects On, while an actual owner supplies its form setting and source
identity. Missing association evidence leaves the default unresolved. Successful
detail processing and anchor defaults do not consult unrelated form facts.
The session admits the shared producer's actual variable vector capacities,
retained output and temporary scratch, preserving observed usage on failure.
Fixed shared wrappers and allocator internals are outside that capacity contract.
These observations perform no autofill or credential mediation.

Captured form-method and form-enctype queries cover potential input/button
submitters. A present override on an actual submit button selects its intrinsic
state, including its invalid-value default, without consulting association.
Otherwise the existing qualified form owner supplies the actual form setting.
Established null ownership has a separate no-owner result; unavailable evidence
remains missing. Button Auto classification uses command presence and the
ordinary parent's select identity. Reports borrow the exact override, consulted
type and consulted form observations. These selections do not replace an
override's empty IDL value or perform validation, submission or navigation.

The captured no-validate query reuses the same submit-button classification and
form-owner operation. Present formnovalidate on an actual submit button or
novalidate on its actual owner proves true, independently of Boolean spelling
conformance. Established null ownership can prove false; unavailable evidence
remains missing when it can affect the result. A sufficient true branch skips
irrelevant facts. Reports retain the consulted raw Boolean observations and
owner origin, including known ownership within an unresolved result. This
readback does not execute constraint validation or submission.

The form-default-button query selects the first owned submit button in ordinary
tree order across the whole tree, including controls outside the form subtree.
A disabled first button remains the default. The implicit-submission-condition
query reports that button or, after proving there is none, whether zero, one or
at least two owned input fields block the no-button branch. Both queries reuse
the qualified form-owner and submit-button observations. Unavailable order or
association remains missing when it can affect the result. These static
conditions do not activate a button, dispatch events or submit a form.

`HtmlGlobalInheritanceFacts` supplies actual document design-mode states and
per-element spellcheck default behaviors. The checked supplement joins one
capture and host revision to its producer, configuration and revision; reports
and admitted failures retain that qualification. Missing facts remain unavailable.
An editing host differs from an editable descendant; isContentEditable is their
union. HTML defines editing hosts, while editable follows the
[selected delegated definition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline).
Translation and spellcheck inheritance follow ordinary parent relationships.
Computed heading levels also follow the specifically defined shadow-host path
and bounded headingoffset recovery. These observations execute no editing
commands, document-mode changes, translation service or per-text spellchecking.
The spellcheck query preserves unsupported foreign-namespace inheritance as an
explicit capability limit.

Language selection preserves the exact tag and its origin, including XML
language precedence, inheritance, explicitly unknown empty language and
document fallbacks. It does not claim tag validity or language-service support.
Element directionality follows explicit, inherited and auto states, using
current control values or source-selected text and slot relationships. The
private first-strong table selects Unicode 18.0.0 and UAX #9 revision 52; reports
and failures retain that data qualification. It preserves the selected L class
for isolated surrogates and combines valid UTF-16 pairs into one code point.

The standard-library-only `tools/generate_bidi_class.py --check` reproduces the
private table offline from the complete checked-in DerivedBidiClass data.
Optional `--verify-unicode-data PATH` compares assigned properties with an
independent UnicodeData witness. The lookup and tests cover the complete code
point domain without supplying a full bidirectional layout algorithm. See the
[source selection](../../references/SOURCE-EDITIONS.md#html-language-and-directionality-sources)
and [data notice](../../licenses/unicode/18.0.0/NOTICE.md).

`HtmlMeterState` retains the recovered and clamped minimum, maximum, value,
low, high and optimum values. `HtmlProgressState` distinguishes indeterminate
progress from a determinate maximum, value and position. These operations apply
the pinned element clauses and finite binary64 recovery; they do not establish
author conformance. Temporary numeric conversion storage participates in the
same session allocation cap, including fallback and overflow paths.

List ownership and ordinals consume `HtmlListParticipation`, qualified by the
capture and separate style and box-participation producer revisions. Computed
list-item behavior, being rendered and producing a CSS box remain distinct facts.
The algorithm traverses every owned computed list item in ordinary tree order;
an implicit reversed-list start counts owned HTML `li` elements specifically.
Start/value recovery and ordinal steps preserve arbitrary admitted integer
magnitudes. Missing participation cannot become a guessed box or partial ordinal.

`HtmlTableModel` retains semantic cell rectangles, row/column and group records,
header associations and table-model errors. It handles delayed footers, implied
rows, zero row spans, overlap recovery, explicit header IDs and automatic header
assignment from the captured tree. These are semantic slots, without layout
geometry. Conditional missing inputs remain explicit; the complete model uses
the same session qualification and bounded batch admission as other queries.

`HtmlFragmentContext` selects an actual captured element or hosted fragment,
retaining its context and target documents, integration facts and nearest form.
It derives the initial tokenizer/insertion state and checked scripting mode:
Inert is the fragment default, disabled context forces Disabled, and unavailable
execution is a typed error. Captured processing-instruction target/data preserve
producer values independently of strict native author syntax. Capture and context
selection perform no parsing, live mutation, script execution or effect
publication. The parser still owns recovered metadata, contextual construction
effects, diagnostics, provenance and its separate execution controls.

`HtmlParseLimits::builder` requires all nineteen finite caps explicitly; zero
is valid, and no default or unlimited sentinel is supplied. `finish` checks
representation before a request can retain the configuration.
`HtmlDocumentParseRequest` configures offline Disabled parsing or retains a
captured document context. Request construction does not establish execution
support for that context. `HtmlFragmentParseRequest` retains the checked fragment
context and distinguishes requested from effective scripting mode. Each request
issues a fresh `HtmlParseCancellation`; cloned handles share sticky cancellation.
Constructing a request admits no source and produces no parse result. The caps
describe logical storage, known allocation layouts and work credits, rather than
wall-clock time, process memory or allocator headers.

Private parser mechanics currently compile for tests while engine integration
continues. Source decoding, UTF-16 buffering, reference lookup, the adapted
tokenizer and controlled arena mutations share one resource controller. The
bounded tree kernel now supplies adjusted element/text insertion, foster and
template locations, source-required ignored-placement guards and exact scope
predicates over an admitted open-elements stack. Actual tokenizer payloads
transfer into the existing immutable-result storage under owner and backing
checks; terminal cleanup retains no lost capacity ticket. The ordinary-document
prologue now implements Initial and BeforeHtml: actual comments, processing
instructions and doctypes reach native storage, doctype modes follow the pinned
HTML rules, and explicit or implied HTML roots lead to an owned BeforeHead
continuation. It retains exact reprocessed input, diagnostics and ignored
whitespace origins. The constructor creates a fresh ordinary document; hosted,
srcdoc, mode-fixed and captured-document execution remain unsupported.
The owning head-stage driver consumes that continuation and the same paused
tokenizer. It implements BeforeHead, ordinary non-script/non-template InHead,
Disabled InHeadNoscript and RCDATA/RAWTEXT Text processing. Text restores its
actual return mode, including delegated noscript processing. It transfers an
owned AfterHead continuation, or stops with the untouched script/template token
for further dispatch. These boundaries retain the actual tree, head pointer,
lexical input and diagnostics; they are unfinished execution states.
Head-stage `html` delegation adds missing attributes to managed root storage,
preserving expanded-name matching, order and origins under the shared controller.
Imported attribute storage remains explicitly unavailable at this checkpoint.

A private fresh ordinary-document driver adds constructor-proved Disabled
script processing. It owns the same tokenizer, controller and arena, retains
actual script metadata, and performs the preparation prefix through the
scripting-disabled return. Empty and unrecognized types preserve their earlier
source-defined metadata changes; script Text EOF marks already started without
preparation. End-tag processing performs the mandatory checkpoint before stack
pop and preparation, using the constructor's closed producer inventory and
absent runtime domains. Captured or hosted requests, declarative shadow roots,
and imported arena state cannot acquire or retain that proof. This specialization
does not execute scripts, load resources or implement a browser event loop.
The generic proofless head continuation still suspends at script starts.
The [selected source boundary](../../references/SOURCE-EDITIONS.md#disabled-script-preparation-and-standalone-checkpoint-sources)
records MIME essence matching and the complete checkpoint clauses; source-local
work/storage admission and sticky terminal failures remain with the sole parser
controller.

The same fresh driver consumes the actual AfterHead handoff and continues through
ordinary InBody and trailing body modes. It handles block, heading, list, ruby,
void and generic-name recovery, first-token LF suppression and raw-text entry.
Repeated body attributes and character runs use the same controlled native
storage, preserving order and exact origins. Late head delegation removes the
stored head entry by identity and restores the actual insertion mode. Formatting,
forms, table/select/frameset transitions, templates and foreign dispatch retain
untouched-token continuations when their required machinery is unavailable.
Source-defined error-and-ignore rows still execute. Construction EOF remains an
unfinished lifecycle state; these private operations produce no public parse report.

Remaining insertion modes, required contextual fragments, context effects and
parse-report publication remain unfinished.
The [parser origin record](PARSER-ORIGINS.md) identifies incorporated source,
modifications and attribution separately from the normative HTML edition.
The offline `tools/generate_character_references.py --check`
verifies every named-reference and C1 data record from the tracked pinned syntax
capture; optional `--verify-origins` also compares the retained original HTML
and acquired implementation witnesses. Normal generation needs no ignored
checkout or network access. These mechanics do not yet provide public parsing
or a complete parser accounting guarantee.
