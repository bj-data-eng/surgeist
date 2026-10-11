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
selected value; effective fallback requires the actual form owner and remains
pending. These content operations do not allocate canvas surfaces, lay out text,
move focus or submit a form.

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
