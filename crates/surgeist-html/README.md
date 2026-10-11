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
Other dimension-list browser overflow or syntax differences are not adopted.

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

`cell_colspan` and `cell_rowspan` retain exact common integer author/recovery
reports independently of the cell-specific bounds. Checked native constructors
require colspan in 1..=1000 and rowspan in 0..=65534; larger exact magnitudes
are rejected for authoring. Processing defaults missing or failed content to
one, defaults recovered colspan zero to one, and clamps larger quantities to
the respective maximum. Rowspan zero retains the symbolic meaning of all
remaining rows in the row group. These
[cell attribute clauses](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attributes-common-to-td-and-th-elements)
and [processing rules](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#forming-a-table)
do not establish overlap conformance or resolve the table's height. The reports
use the standalone per-value UTF-16 cap and ordinary native allocations.

`col_span` and `colgroup_span` apply the same separation to column-span numeric
content. Numeric authoring requires 1..=1000; processing defaults absent, failed
or zero recovery to one and clamps larger quantities to 1000. One checked native
constructor emits canonical `span` syntax for either element. Colgroup numeric
conformance does not establish its child restrictions or actual table use:
when actual col children are present, table formation uses their spans and
ignores the group's own span. The [column clauses](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-colgroup-element)
and existing captured table kernel own those contextual distinctions. These
standalone reports provide neither a resolved group width nor an IDL getter.

`select_size` combines exact size content with the actual multiple attribute.
Native size construction requires a positive integer. Processing preserves every
successful nonnegative recovery, including zero and negative zero; only missing
or failed size parsing selects display size 4 when multiple is present, otherwise
1. Boolean spelling conformance is retained separately from presence. Successful
size recovery needs no multiple role; a missing/failed size with a supplied wrong
multiple attribute leaves the fallback explicitly unavailable. Both supplied raw
slots use per-value UTF-16 admission before applicability, and derived values use
ordinary Rust allocation. This [intrinsic display-size concept](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-select-size)
retains arbitrary magnitudes and is distinct from the size IDL getter’s zero
reflection default, rendering and selection state.

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

`progress_attributes` retains the original value/max attributes, common floating
reports and independent exact mathematical author constraints. Value must be
nonnegative and no greater than the authored maximum, or one when max is absent;
maximum must be positive. Invalid present max leaves that comparison unavailable.
Binary64 rounding, underflow and overflow remain separate common evidence.
`HtmlProgressAttributes` checks a coupled finite native pair against its actual
canonical emitted decimals and preserves absent attributes. These operations use
ordinary allocation and per-value raw UTF-16 admission; captured Progress below
owns effective quantities and determinate state.

`meter_attributes` checks the six authored number roles, retaining each original
attribute, strict grammar and common floating-point report. Its nine independent
exact comparisons use zero and one only for absent min/max; malformed present
bounds remain unavailable. Value is required and must lie within the bounds,
as must present low/high/optimum values. Low must not exceed high when both exist.
Optimum may lie outside the low/high band. `HtmlMeterAttributes::from_syntax`
checks a coupled native bundle and preserves accepted decimal spellings exactly,
including numbers beyond binary64 range or precision. Emission moves those strings
into ordered attributes. Both operations use ordinary allocation; raw interpretation
uses per-value admission. Captured Meter below owns processing defaults and clamping.

`HtmlAuthoredArea` supplies exact circle, rectangle and polygon values or a
symbolic whole-image area. Native rectangles require increasing x and y edges;
polygons require at least three vertices. Circle radius may be zero. Canonical
attributes serialize arbitrary integers directly, without a floating projection.

`area_coordinates` retains both supplied attributes, the shape report, common
floating-point-list evidence and separate shape-specific author constraints.
Missing or invalid shape selects Rectangle. The recognized circ, polygon and
rectangle recovery aliases remain explicitly nonconforming. Author coordinates
require exact mathematical integers: decimal and exponent spellings can qualify,
while a fraction rounded to an integer by binary64 cannot. Rectangle ordering is
also checked exactly without expanding large exponents.

The processing descriptor reuses the common recovery list, including zero
substitution for scalar parse failures. It ignores excess circle/rectangle
coordinates, normalizes reversed rectangle edges, drops an unmatched final
polygon coordinate and reports insufficient coordinates or nonpositive circle
radius as Empty. WholeImage stays symbolic. These descriptors do not resolve
image dimensions, map association, layering or hit testing. Native values and
reports use ordinary Rust allocation and the interpreter's per-value UTF-16
admission cap; they supply no semantic-session ledger or cancellation guarantee.

`modification_datetime` interprets `datetime` on HTML `ins` and `del` using the
[pinned modification clause](../../references/html-language-elements-text-links--877d6146487f--8833a85fd242.md#attr-mod-datetime).
Its report keeps strict date-with-optional-time authorship separate from common
date-or-time recovery. Only a recovered `HtmlDate` or
`HtmlParsedGlobalDateTime` supplies an associated timestamp. A recovered
standalone Time remains available as common evidence but supplies no timestamp;
failed processing has its own no-timestamp reason. Missing content and unapplied
expanded-name rules remain distinct from both outcomes.

Parsed global timestamps retain explicit source offsets, exact long fractional
seconds and arbitrary positive years alongside local components and UTC
instants. Dates remain dates without an inferred host time zone. Native
`HtmlDateWithOptionalTime::into_modification_datetime_attribute` converts a
checked author value directly to its content attribute. Raw UTF-16 values,
expanded names and prefixes remain unchanged. Admission uses the interpreter's
per-value raw-unit cap; native values and reports use ordinary allocation
without physical-memory accounting, cancellation or allocation-failure recovery.

`HtmlTimeElementValue` provides the ten strict native alternatives defined by
[pinned HTML for `time`](../../references/html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-time-element):
month, date, yearless date, time, local datetime, time-zone offset, global datetime,
week, year and duration. It reuses the checked temporal components and their
serialization; `into_datetime_attribute` creates a native datetime attribute
without reparsing.

`HtmlValueInterpreter::time_element_value` separates strict author conformance
from `HtmlMachineReadableTime`, which retains the first successful processing
alternative in the source-defined order. Authored Year spelling requires at least
four ASCII digits; processing accepts any positive all-ASCII-digit year, so `1`
has a Year equivalent despite invalid author syntax. Existing parsed types retain
exact long fractions, arbitrary years and explicit global offsets without a host
time zone. Reports preserve the whole raw UTF-16 value and distinguish missing
input from present invalid input, with admission before the format attempts.

This standalone value operation uses ordinary allocation for materialized
values and exact temporal arithmetic. Its per-value raw-unit cap supplies no work
ledger, cancellation, physical-memory limit or allocator-failure recovery.
Capture-qualified `TimeElement` interpretation uses the semantic session described
below. Lexical acceptance does not certify that the value represents the
human-readable contents of an element.

Native `HtmlSrcsetValue` checks ordered image candidates for `srcset` and
`imagesrcset`. URLs remain symbolic exact strings; widths retain arbitrary
positive integers and authored density spelling is retained. Author positivity
and equality use exact mathematical decimal values independently of finite
binary64 recovery. Reports for `img`, `source` and `link` separate author
constraints from recovered candidates. Repeated or invalid descriptors can
leave diagnostics without suppressing later candidates;
processing accepts density zero and positive underflow to zero even when strict
author constraints differ. The
[selected density-overflow correction](../../references/SOURCE-EDITIONS.md#srcset-density-conversion-overflow)
omits only the overflowing candidate and records `DensityConversionOverflow`.

Raw attributes, symbolic URLs and half-open ranges preserve exact UTF-16 units,
including isolated surrogates. Supplied role-specific raw attributes and parent
or following-image observations support separate contextual author checks;
unavailable facts remain unchecked. Those observations do not establish a
capture-qualified semantic result. Each supplied raw value has the standalone
UTF-16 admission cap; native values and reports use ordinary allocation without
aggregate work accounting, cancellation or allocation-failure recovery. URL
validity and resource properties remain unchecked. These operations do not
parse CSS `sizes`, normalize source sets, select or fetch resources, or implement
the controlled semantic pipeline.

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

`LabelFormOwner` returns the actual form owner of the labeled control. No labeled
control, a control that is not form-associated, or an established null form owner
returns known null. Unavailable target or association evidence remains missing.
The query reuses labeled-control targeting and form-owner resolution; a label’s
ancestor form or authored `form` spelling does not supply this result.
It observes one immutable capture and performs no association mutation.

`FormControls` returns listed elements in the form’s ordinary tree whose actual
form owner is that form, excluding Image Button inputs. `FormLength` counts the
same membership without retaining a member vector. External controls participate
in tree order, and a listed ordinary root participates when qualified current
association establishes its ownership. `FieldsetControls` instead returns listed
ordinary descendants, excluding the queried fieldset itself and including nested
fieldsets and Image Button inputs, independently of form ownership.

Both collection facts use `HtmlListedControlCollection`, with capture-qualified
subject keys. Influencing unavailable ownership, classification or child order
remains missing evidence. Established other/null ownership can prove exclusion
without consulting an input type or custom classification; an Image state can
prove form-collection exclusion without resolving ownership. Traversal and actual
retained capacities use the same semantic ledger and atomic batch contract.
These immutable results provide no live collection, named lookup, past-names map
or control mutation. The [pinned form collection predicate](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)
and [fieldset collection clause](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-fieldset-element)
define their distinct filters. The fieldset’s root exclusion follows the selected
[pinned WebKit descendant traversal](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/CollectionTraversalInlines.h#L34-L42);
form collections use its [actual associated-control list](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/HTMLFormControlsCollection.cpp#L75-L97).

Select/option readback exposes the source-defined option list, selected options,
selected index and option index. One bounded ordinary-tree traversal preserves
tree order and the specific option, datalist, hr, nested-select and optgroup
pruning rules. Selectedness comes from the existing current-state supplement;
authored selected attributes, disabledness and multiple do not replace it.
Select and datalist collections share `HtmlOptionCollection`, whose getters
expose capture-qualified subject keys. A known empty selected list and no
selected index differ from unavailable state or order. These immutable results
provide no live collection or selection/reset transition.

Option text, label and value getters expose exact immutable strings and their
selected source. Present label/value attributes, including empty values, are
returned unchanged. Descendant fallback uses HTML-aware text: HTML/SVG script
and HTML img subtrees are excluded, and only ASCII whitespace is stripped and
collapsed across text nodes. BR adds no separator; other UTF-16 units remain
exact. The label getter follows IDL readback, rather than the separate rendering
label algorithm. Select value reuses the source-defined option list and returns
the first currently selected option's value. Its result distinguishes a selected
empty value from a known absence of selection. The option form-owner getter
follows the source-defined nearest-select barriers and returns that select's
actual or reset-qualified association. It distinguishes no nearest select from
a select with an established null owner. Consulted attributes, traversal,
temporary scratch and actual output capacities use the semantic session's
admission and atomic batch contract; missing influencing facts remain explicit.

`SelectPlaceholderLabelOption` identifies the first source-list option when
required is present, display size is one, its value is empty and no HTML
optgroup intervenes. Explicit size one can qualify even with multiple present.
Boolean presence consumes no spelling grammar; size and option value use their
existing numeric and text owners. An established absence differs from missing
influencing facts. Intermediate text allocation remains cumulatively admitted
even when the result is absent. This immutable query follows the
[placeholder definition](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#placeholder-label-option)
and performs no selection or validity transition.

`TextareaDefaultValue` concatenates exact Text data from immediate ordinary
children only. `OutputValue` concatenates exact Text data from all ordinary
descendants. Both preserve UTF-16 units and whitespace; output traversal includes
script and image descendants, without the option getters' HTML-aware pruning
or whitespace collapse. Separate shadow and template trees do not contribute.
The [textarea default-value clause](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#dom-textarea-defaultvalue)
and [output default-value clause](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#dom-output-defaultvalue)
retain the selected getter definitions.

`OutputDefaultValue` reads the actual nullable default override from
`HtmlOutputDefaultFacts`. An established null selects descendant text; an
established string is returned unchanged, including an empty string. Absent or
unavailable override evidence produces a typed missing result. Checked facts
require the same capture and host revision, actual HTML output subjects and
unique records, with finite validation work, moved-vector capacity and logical
stored UTF-16 unit limits. The semantic context validates the join before
creating its identity; captured text does not establish the historical override.

Computed text buffers admit actual variable capacity through the shared finite
semantic ledger. Traversal and writes are charged, scratch frames are released
on every outcome, and cancellation, sticky qualified failures and atomic batches
follow the session contract. Existing immutable override backing and fixed
shared wrappers remain outside that variable-allocation accounting. These
readbacks execute no setters or reset and do not supply textarea current value
or complete source-conformance checking.

`TimeElement` selects a present null-namespace `datetime` attribute, including an
empty value. When the attribute is absent, it concatenates exact Text data from
immediate ordinary children. Namespaced or case-distinct attributes do not select
the value. Unknown child coverage produces a typed missing result; shadow and
template contents do not contribute. The
[pinned time clause](../../references/html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-time-element)
defines this selection and its independent author restriction.

`HtmlTimeElementReadback` retains the selecting attribute or child-text origin,
the raw value report, strict author syntax and the first successful
machine-readable alternative. Its descendant author state separately records
whether absent-attribute contents contain an Element. That violation does not
suppress the machine-readable equivalent. A present attribute needs no child
coverage, and lexical success does not certify the human meaning of the contents.

Captured interpretation uses the semantic session's work, integer-digit,
allocation-byte and output limits. Borrowed recognition shares the temporal
grammar with standalone operations; exact materialization admits numeric buffers
and retains their observed capacity even when a later operation fails. Duration
accumulation and global UTC conversion poll between bounded operations, preserving
arbitrary years, long fractions and explicit offsets without a host time zone.
Immutable shared input backing and fixed wrappers remain outside variable-buffer
accounting. Terminal resource or cancellation failures preserve the session's
sticky failure and atomic batch contract.

The captured `ImageMapAttribute`, `ImageControlsAttribute` and
`StylesheetDisabledAttribute` queries retain the intrinsic Boolean report and
the author-context evidence they actually consult. A present Boolean remains
true even when its spelling or required context is nonconforming. The ismap
query walks ordinary ancestors for the first exact HTML a with href present;
empty href qualifies, and a witness ends the walk before further ancestry is
needed. Missing ancestry preserves the known content report with explicitly
unavailable author context. Controls observes alt presence and nonempty content;
disabled observes ASCII-insensitive stylesheet membership in rel.

`HyperlinkReferrerPolicyAttribute` applies to HTML a and area and retains the
actual href-presence observation separately from intrinsic policy recognition.
An absent target attribute requires no contextual lookup. All selected raw
values and consulted rel text are admitted in full before recognition; href and
alt contents need only presence or emptiness observations. These operations use
the shared semantic ledger, fallible output admission and sticky atomic batches.
Immutable attribute backing and fixed wrappers are outside variable-buffer
accounting. They establish no effective request policy, stylesheet activation,
image-map execution or UI effect.

`InputSuggestionsSource` uses the input state's source-defined `list`
applicability and the first element in the same ordinary tree whose ID exactly
matches the raw attribute. That element supplies the source only when it is an
HTML datalist; a first matching element of another kind establishes null rather
than permitting a later match. Absent `list` content, no matching ID and states
where `list` does not apply also establish null; unavailable influencing facts
remain missing.

`DatalistOptions` returns every ordinary-descendant HTML option in tree order,
including options beneath nested selects, datalists or options. Separate shadow
and template trees are outside that traversal. `DatalistSuggestionOptions`
retains only options whose source-defined disabledness is false and whose source
option value is nonempty, reusing exact value attributes or HTML-aware text
fallback. These immutable membership results share the semantic session's
bounded traversal, allocation and atomic batch contract. They provide no live
collection, host UI, display-label computation, input-value compatibility
checks, relevance filtering or control-value changes. The
[pinned datalist clauses](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-datalist-element)
own the distinction between all options and suggestion membership.

The control-labels query returns the complete associated label list in ordinary
tree order. Hidden inputs produce known null; a complete applicable empty list
differs from unavailable evidence. Exact first-ID and implicit-descendant rules
reuse the existing labeled-control owner, with irrelevant labels excluded before
requiring unrelated custom-control facts. Form-associated custom controls use
their supplied classification. The result is an immutable association snapshot;
it does not attach ElementInternals or provide a live NodeList. Traversal and
nested output retain the semantic session's admission and atomic batch contract.

`MapAreas` returns HTML area descendants of a map in ordinary tree order,
including areas beneath wrappers and nested maps. Attribute values and visibility
do not filter membership. Separate shadow and template trees are excluded;
influencing unavailable child order remains missing rather than a partial list.
`HtmlAreaCollection` retains capture-qualified subject identities from one
immutable observation, using the shared traversal and semantic resource ledger.
The [map areas clause](../../references/html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#dom-map-areas)
defines membership; this result supplies no live collection, named lookup,
image-map selection, geometry or activation.

`DetailsSummary` observes the first actual direct HTML summary child; complete
absence returns no captured identity. `SummaryForParentDetails` tests whether a
summary is that first child of its actual ordinary details parent.
`DetailsNameGroup` retains the selected name attribute and all same-tree HTML
details members with an equal nonempty UTF-16 name, in ordinary tree order.
Absent or empty names produce a singleton containing the queried subject without
requiring unrelated topology. Separate shadow and template trees remain separate;
missing influencing ancestry or child order yields missing evidence.

These observations reuse the semantic session's qualification, raw admission,
resource ledger and atomic batches. They establish captured relationships rather
than group author conformance, exclusivity mutation, activation, toggle events or
a rendered fallback legend.

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

`TableBodies`, `TableRows`, `TableSectionRows` and `TableRowCells` return immutable
collections of qualified captured subjects. Bodies, section rows and row cells
use direct HTML children. Table rows collect header-section rows first, then
direct rows and body-section rows in tree order, and footer-section rows last.
These collections retain actual elements without projecting implied table slots;
they use the semantic session's work and output limits and report missing ordered
children explicitly. They provide captured membership rather than live collection
access or table mutation.

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
stored head entry by identity and restores the actual insertion mode.
Other resettable control creation, select transitions, templates
and foreign dispatch retain untouched-token continuations when their required
machinery is unavailable.
Source-defined error-and-ignore rows still execute. Construction EOF remains an
unfinished lifecycle state; these private operations produce no public parse report.

Ordinary frameset documents use that same constructor-proved driver. AfterHead
and eligible InBody entry lead to InFrameset, AfterFrameset and AfterAfterFrameset.
Body removal runs the existing form effects while retaining the current-form
pointer and formatting list; trailing whitespace can reconstruct formatting.
Frame insertion returns at the actual node-document's null browsing-context
guard. Noframes uses RAWTEXT and restores its saved caller mode. Mixed character
runs preserve exact origins while diagnosing and omitting nonwhitespace and NULL.
Generic or imported owners cannot establish the required constructor proof.
This adds no hosted navigation, fragment execution or completed EOF lifecycle.

The fresh ordinary Disabled driver also retains the actual current-form pointer
and handles ordinary form starts and ends. A nested start is diagnosed and
ignored before paragraph closing or creation. An end clears the pointer before
null/scope checks, generates implied ends and removes only the pointed form's
stack identity. The selected HTML order places the current-node diagnostic
before that removal. These rules follow the
[pinned InBody clauses](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-main-inbody).

Parser-created HTML images retain actual form-owner and parser-inserted metadata
in the same controlled arena. Creation compares the intended parent's ordinary
tree with the current form's tree, allowing source-defined nonancestor ownership.
An image is neither listed nor resettable; its `form` attribute does not suppress
this association. Synchronous tree mutations complete the whole source removal
pass before insertion effects, including affected descendant images and
formatting moves. A stop retains the actual source-ordered metadata prefix;
only terminal cleanup remains available and partial publication is refused.
Association scans are metered and can require quadratic work. Template forms,
other resettable controls, unfinished table rows, foreign dispatch, contextual
fragment execution, hosted parsing and full EOF/report completion remain unfinished.

The same association registry now admits parser-created button, fieldset and
object elements. These are listed and form-associated, but not resettable.
An explicit form attribute, including empty content, suppresses pointer-based
creation association. Connected controls resolve the first element with the
exact nonempty ID; an earlier non-form blocks a later form, and failed lookup
remains null. Detached controls use the source's ancestor branch.

Tree effects retain one source-ordered phase. ID entry/removal sweeps include
all canonical explicit-form controls in the same node-document domain, including
detached recipients; ID changes instead use the changed element's ordinary tree.
Actual removing hooks can trigger resets in already detached trees, following
the [selected interpretation](https://github.com/bj-data-eng/surgeist/issues/1136).
Removal resolution hides the moved root from outside traversal before insertion
effects. Each appended html/body ID drains its effects before the next attribute
or transfer rebinding. Stops retain topology, association prefixes and transfer
owners for terminal disposal, while observations and freeze reject publication.
Repeated scans remain metered and may be superlinear.

Ordinary textarea parsing uses that same fresh driver for RCDATA, first-LF
omission and restoration of the actual caller insertion mode, including table
delegation. Creation resets its raw value before parser-established form
association. Direct child-text changes update independently owned raw UTF-16;
removal-view updates precede insertion-view updates, and text merges invoke the
parent hook. Every actual parser stack removal resets dirty-value and user-validity
state and copies the current direct child text before mode restoration or EOF
reprocessing. Reference-produced CR remains raw rather than API-normalized.
Active and frozen private observations retain the actual arena/controller
qualification. Old and new raw backing coexist under admission; successful
replacement releases old backing while retired registry slots stay admitted
until disposal. Stops refuse publication and cleanup runs no reset. Repeated
copies and scans may be quadratic; all variable work remains charged and polled.
User interactions, value setters, API-value normalization, submission wrapping and
public parse completion remain separate work.

Ordinary formatting starts and ends now use the active-formatting list, Noah
rule, reconstruction and adoption algorithm. Reconstruction and adoption replay
the actual original tag into independently owned controlled storage, preserving
attributes and origins while giving each new element a distinct identity.
Applet, marquee and object establish real list markers. Button recovery diagnoses
an in-scope old button before popping it; fieldset uses block recovery. The sole
parser controller admits recipe retention, replay,
comparisons, stack/list mutations and coherent child moves before their effects.
Child transfer can stop after a coherent partial move; a terminal failure still
prevents publication. Work is finite and polled, with potentially quadratic
comparison and tree-order/depth maintenance costs. This checkpoint makes no
linear-time or complete parser-accounting claim.

The structural table checkpoint handles InTable, InTableText, InCaption,
InColumnGroup, InTableBody, InRow and InCell, including implied colgroup, tbody and
tr elements. Table entry observes the actual document mode; table closure resets
the insertion mode from the real stack. Caption and cell boundaries use the
existing formatting markers. Body/head delegation and Text return preserve the
actual table mode, and fostered form/image insertion uses the existing association
and tree-mutation owners.

Pending table characters retain their actual text and source ranges. NULL is
diagnosed and omitted before the complete batch is classified. A wholly
whitespace batch inserts directly without formatting reconstruction; any
nonwhitespace makes the entire remaining batch use fostered body-character
insertion. Auxiliary capacity, scans, diagnostics, mode changes and transfer
owners use the sole controller. A terminal stop can retain a coherent flush
prefix, remaining characters and the triggering token for cleanup, while refusing
continued construction or publication.

This is a structural checkpoint. Hidden input and other unavailable control
creation, select, templates and foreign branches preserve their original token
before unsupported effects. It does not establish complete InTable behavior,
contextual fragment parsing or completed EOF/report publication.

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
