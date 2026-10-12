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
The [HTML language inventory associations](INVENTORY.md) connect each indexed
identity and common microsyntax to its defining clauses, current owners and
remaining source-specific requirements.

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

<a id="checked-custom-element-names"></a>

[HtmlCustomElementName](src/custom_names.rs) checks the pinned
[valid-custom-element-name predicate](../../references/html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#valid-custom-element-name).
Its private value retains exact UTF-16 without trimming, case folding or
normalization. The name must begin with a lowercase ASCII letter, contain a
hyphen, exclude uppercase ASCII and the eight reserved names, and satisfy the
selected [DOM local-name rule](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md#valid-element-local-name).
That rule's ASCII-first branch excludes ASCII whitespace, NULL, slash and
greater-than; it permits less-than, equals, quotes, noncharacters and isolated
surrogates after the first letter. This local-name proof does not certify
serialized HTML syntax.

Moving `into_qualified_name` emits an unprefixed HTML name;
`into_is_attribute` emits a lowercase no-namespace `is` attribute. Both preserve
the checked value. [HtmlValueInterpreter::is_attribute](src/attributes/custom_name.rs)
admits the original raw value's UTF-16 length before expanded-name applicability
and checks this same predicate. An applicable absent attribute remains Missing;
empty or invalid content remains raw and locally Nonconforming with a typed
error; valid content exposes the checked name. Wrong element namespace or
attribute identity remains Unapplied. No missing or invalid value supplies a
default checked name.

Every applicable present `is` report leaves author context Unchecked, including
locally invalid content. A matching definition and `extends` relation, and the
prohibition on `is` on autonomous custom elements, require the separate
[captured author-context observation](#captured-custom-element-author-context).
A valid hyphenated name alone does not establish registration or element
classification. Content `is` interpretation does not set the saved runtime
`is` value or change an existing element's behavior. Registry execution,
upgrades and callbacks remain with their execution owner. Native checking and
raw interpretation use ordinary allocation; only raw interpretation has the
per-original-value cap. They supply no work, physical-memory or cancellation
budget. The existing capture admission and custom-host classification consume
the same borrowed predicate. [The public tests](tests/custom_names.rs) cover
exact units, reserved names, report states and raw admission boundaries.

<a id="captured-custom-element-author-context"></a>

The captured [IsAttributeAuthorContext query](src/semantics/custom_element_context.rs)
returns the local `is` report, the current matching-definition requirement and
the autonomous-element prohibition as independent results. A Known observation
can contain nonconformance or an unavailable obligation; it is not an overall
author-conformance verdict. Absent current `is` makes both obligations NotRequired
without consulting unrelated registry, state or association facts. Invalid local
syntax blocks definition matching while the autonomous prohibition is still
checked. Influencing unavailable facts remain typed evidence, not false results.

The [capture producer](src/capture.rs) admits the actual custom-element state and
associated-definition pair once, with capture-qualified record identities.
An incomplete registry can retain independently known records without claiming
a complete lookup set. Current content matching uses the element's current lookup
registry and the source's autonomous-first definition search; it does not fall
back to the saved runtime `is` value or an earlier association after adoption.
The autonomous prohibition uses the actual Custom state and associated definition
independently of that lookup. Known customized Custom associations check the
saved name; autonomous associations may retain a different saved value.
Neither this admission nor the query reconstructs creation or upgrade history.

Capture builders retain ordinary allocation with logical retention limits.
The [semantic session](src/semantics.rs) uses its existing qualified, finite ledger
for raw admission, the shared borrowed local validator's checked work envelope,
the actual definition cursor and result-vector layout. Retained capture backing
is shared after admission; it is not a newly allocated copy charged to the
session. Limits and cancellation failures stay sticky and discard the entire
query batch. These bounds do not promise process RSS, a universal polling
quantum or linear total work. [Public tests](tests/semantics_custom_element_context.rs)
cover current versus saved values, adoption, partial registries, independent
unavailable obligations and exact/one-short admission. Registry execution,
constructors, upgrades, callbacks and full custom-element conformance remain
outside this observation.

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

The attribute-specific token owner supplies checked `HtmlAccessKeys`, explicit
`HtmlBlockingTokens` and exact `HtmlIconSizes`. `accesskey` checks an ordered,
case-sensitive unique set of single code points while its processing candidates
retain eligible repetitions. `blocking` separates exact authored `render`
tokens from case-folded explicit membership. Its link and script companions
observe actual rel membership or supplied script kind independently of token
validity. `icon_sizes` preserves each keyword's processing outcome and exact
positive bitmap dimensions beyond machine-integer range; its link companion
checks icon/apple-touch-icon membership. Reports retain raw input and distinguish
local author, context and processing results. They certify no keyboard shortcut
assignment, implicit/current render blocking or linked-resource availability.
These standalone operations use ordinary allocation and the original-value
UTF-16 cap described below.

`HtmlLinkDestination` supplies the thirteen `as` keywords from pinned HTML's
preload/modulepreload domains and its [delegated Fetch subset](../../references/fetch-script-like-destinations--e9460d1b22ea.md).
`link_as` matches the entire raw value case-insensitively in ASCII without
trimming; missing, empty and invalid input have no intrinsic state.
`link_as_with_attributes` observes actual `as` and `rel` slots and reports
preload and modulepreload author requirements independently when both tokens
occur. Missing preload `as` fails required presence; modulepreload `as` is
optional. Content-only interpretation leaves those context checks Unchecked.

The [link-destination report](src/attributes/link_destination.rs) also gives
symbolic destination decisions. Modulepreload projects script from no state,
including invalid values, while retaining the author failure. A recognized
image remains an out-of-domain image state. This projection does not assert that
the real algorithm reached the destination step: href checks, Fetch translation,
requests, resource loading and module execution are separate consumers. The
report does not certify full `rel` vocabulary or uniqueness. Both selected raw
slots use the original-value UTF-16 cap and ordinary allocation.

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

<a id="script-type-values"></a>

`HtmlScriptTypeKeyword` emits the three canonical authored `type` keywords:
module, importmap and speculationrules. The
[script type reports](src/attributes/script_type.rs) keep current content meaning
separate from type/language preparation classification. `script_type` observes
the exact HTML script/type role: missing, empty and any of the sixteen
[pinned JavaScript MIME essences](../../references/mimesniff-javascript-essence--39aa53511b13.md)
mean nominal Classic. The three keywords match ASCII-insensitively without
trimming; other values mean DataBlock. Empty or essence-valued types carry
`OmitRedundantType` advice. `HtmlDefined` establishes only the HTML-defined
type-value branch; data-block MIME grammar remains `ExternalMimeUnchecked`.
An applicable specified value leaves wider author context Unchecked.

`script_type_with_attributes` selects the actual exact no-namespace lowercase
type/language slots from a complete `OrderedAttributes` set, preserving their
raw evidence and known absence. Empty type, or absent type with empty/absent
language, classifies as Classic. Otherwise explicit type wins; absent type uses
the virtual `text/` language prefix. Only explicit JavaScript essence comparison
also strips leading/trailing ASCII whitespace. Keywords and language remain
untrimmed, and MIME parameters are never removed. `NoScript` is the selected
classification step's no-classification return. For example, ` module ` remains
DataBlock/NoScript, while ` text/javascript ` remains DataBlock in the content
report but classifies as Classic in this companion. The operation does not
validate obsolete language authoring, MIME grammar, JavaScript/JSON bodies or
the whole script element.

These standalone reports admit each complete selected raw UTF-16 value before
applicability or override, including language overridden by type. They use
ordinary allocation and the per-value cap, with no aggregate physical-memory,
controlled-work or cancellation promise. The [shared borrowed cursor](src/script_type.rs)
also drives the actual private classifier through its existing controller;
each comparison/trim read follows admission, without allocating a synthetic
language string. Public `HtmlScriptKind` describes current classification and
remains distinct from saved private preparation metadata. No saved metadata,
preparation effects, resource loading or execution is produced by these reports.
The [source-derived tests](tests/attributes_script_type.rs) cover both meanings,
exact slots, raw preservation and admission boundaries.

Numeric attribute reports cover canvas width/height, textarea cols/rows and
tabindex content values. They preserve whole authored syntax and prefix recovery
separately from each attribute's selected default: canvas defaults to 300/150
while accepting zero, and textarea defaults to 20/2 when recovery fails or yields
zero. Tabindex retains an exact signed integer or source-null; the separate
captured tabIndex query below owns its signed-long getter and element defaults.
Global `heading_offset` reports whole author syntax in 0..=8 separately from
exact recovered magnitude. Successful recovery above eight remains intact;
missing and failed recovery contribute zero with distinct origins.
`HtmlNonNegativeInteger::into_heading_offset_attribute` emits canonical values
in that author range and rejects larger magnitudes. These local operations use
ordinary allocation; interpreter reports admit the entire selected raw UTF-16
value before applicability. Captured ancestor/reset/shadow-host traversal and
the final heading-level limit of nine remain separate operations.
Formmethod and formenctype overrides reuse the defining keyword rules and
separate submitter
author restrictions from their intrinsic values. A missing override retains no
selected value; the captured form-setting queries below use the actual form
owner for effective fallback. These content operations do not allocate canvas surfaces, lay out text,
move focus or submit a form.

`command` interprets exact HTML button content as one of six standard commands,
Custom or Unknown. `HtmlStandardCommand` emits canonical standard spelling;
`HtmlCustomCommandKeyword` checks a two-hyphen prefix and the selected ASCII
boundary-whitespace rule before exact native emission. The privately coupled
`HtmlCommandReadback` always retains a payload for Custom and none for other
states. Processing preserves a prefix-qualified Custom even when trailing
ASCII whitespace violates author syntax. Bare `--`, internal spaces and
non-ASCII whitespace are retained under the
[selected interpretation](https://github.com/bj-data-eng/surgeist/issues/1133#issuecomment-6117207035);
this records a source-qualified ASCII boundary choice, not an upstream
clarification or full serialized-character conformance.

`popovertargetaction` has its own Toggle/Show/Hide domain, with missing and
invalid Toggle defaults. Exact HTML button and input roles retain local content
independently of target or activation facts. The input-state companion permits
author presence only for Submit, Image, Reset and Button, retaining the local
state when another type violates that restriction; unavailable type leaves the
author context unchecked. Missing action has no presence violation.
These reports admit the entire raw UTF-16 value before applicability and use
ordinary allocation with the per-value cap. Same-tree and explicitly reflected
targets, command validity, activation, dialog and popover transitions remain
distinct operations. Existing Auto submit-button classification continues to
use command/commandfor presence, including empty or unknown values.

`th_scope` interprets exact HTML th content as row, col, rowgroup or
colgroup, with missing and invalid Auto processing. Empty and literal `auto`
are invalid authored keywords. `HtmlTableHeaderScope` emits only the four
canonical author keywords. Raw content, recognition, author spelling and
processing state remain independent. Group keywords leave actual table
anchoring Unchecked; the existing TableModel continues to own header
associations. These local reports use ordinary allocation and admit the entire
original UTF-16 value before applicability through the per-value cap.

`textarea_wrap` retains Soft/Hard content state, raw spelling and lexical
conformance separately; missing and invalid content select Soft. Hard leaves
its cols-presence author requirement unchecked until `textarea_wrap_with_cols`
observes one complete `OrderedAttributes` collection. That companion retains
actual cols or known absence and admits both selected raw values before
applicability. Any present cols satisfies this presence rule; numeric validity
remains with `textarea_cols`. Missing cols reports a separate context violation
without changing Hard. These ordinary reports do not perform wrapping or
submission and use the per-selected-value UTF-16 cap.

`HtmlUtf8EncodingLabel` checks the exact ASCII-case-insensitive UTF-8 spelling
required by form accept-charset and meta charset, preserving accepted casing.
Its consuming attribute constructors emit the canonical no-namespace names.
`form_accept_charset` and `meta_charset` admit the entire selected raw value
before applicability and retain missing, empty, invalid and unapplied evidence.
Missing or invalid content selects no label or default.

An applicable present meta charset keeps declaration-context conformance
Unchecked even when the local spelling fails. Document encoding, serialized
declaration constraints and submission encoding selection require separate
operations and facts. Checked values and reports use ordinary allocation;
interpreter reports retain the per-value UTF-16 admission cap.

`HtmlInputColorSpace` supplies canonical colorspace attributes for Limited sRGB
and Display P3. `input_colorspace` retains keyword recognition and local author
spelling separately from the Limited sRGB missing/invalid default. Applicable
present content leaves its Color-state author restriction Unchecked.
`input_colorspace_for_input` uses an explicitly supplied actual input type to
observe that restriction without changing lexical evidence or intrinsic state.
These ordinary reports admit the entire selected raw UTF-16 value before
applicability. CSS color conversion, color-well updates and live reflection
remain separate operations.

`hash_name_reference` retains exact raw UTF-16 and borrowed suffixes, separating
leading-`#` author form from first-`#` processing. A lone `#` has an empty author
name and a null processing result. The local report does not prove a same-tree
name witness. `HtmlUsemapReference::for_map_name` accepts a checked nonempty space
token and emits literal `#` followed by its exact units as a usemap attribute;
it makes no tree, uniqueness or serialized-character conformance claim. Local
interpretation admits the whole selected raw value; both operations use ordinary
allocation.

`ol_start` and `li_value` report exact local signed list declarations through
the shared integer interpreter. Whole author syntax and independent prefix
recovery retain their own evidence, including arbitrarily large signed values.
Missing or failed content supplies no default, starting value or ordinal.
Present applicable li value leaves its actual-parent author restriction
unchecked. Existing exact integer construction and serialization compose with
the ordinary Attribute factory for native authoring. The captured list owner
below supplies reversed counts, starting values and ordinal sequencing.
These standalone reports use ordinary allocation and the per-raw UTF-16 cap.

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

`id_attribute` checks HTML id's local nonempty/no-ASCII-whitespace rule through
the existing `HtmlSpaceToken` validator. Missing, supplied empty and invalid
content remain distinct with exact raw UTF-16; every present applicable id leaves
tree uniqueness unchecked. A missing checked token does not establish absence
of a DOM ID. `HtmlSpaceToken::into_id_attribute` emits the canonical attribute
with exact accepted content. These operations use ordinary allocation, with a
per-value admission cap on raw interpretation. They do not validate uniqueness,
DOM ID behavior in other namespaces, selectors or serialized characters.

`HtmlNavigableTargetName` checks an exact nonempty name: it rejects a leading
underscore and the combination of TAB, LF or CR with a less-than sign.
`HtmlNavigableTargetKeyword` supplies the four canonical keywords. The local
`navigable_target_attribute` reports cover hyperlink, base and form targets,
submitter formtarget, and iframe/object names. The last two accept names only;
keyword roles recognize the four spellings without trimming. Reports retain
original UTF-16, lexical errors and applicable context restrictions separately.
Present hyperlink, base and submitter attributes leave those restrictions
unchecked. Checked values compose with the existing Attribute factory. Native
construction uses ordinary allocation; raw reports apply the per-value cap.
These values do not select a navigable, apply target fallback or navigate.

`details_name` checks the local specified-nonempty rule for `details[name]`.
Absence remains missing; supplied empty content is retained and nonconforming.
Every nonempty name preserves its exact UTF-16, including whitespace and case.
`HtmlDetailsName` provides checked native construction and moves the accepted
value into the canonical name attribute. Both operations use ordinary allocation;
raw interpretation applies its per-value admission cap. This local check does
not certify same-tree grouping, nesting or exclusivity conformance.

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

`input_image_attributes` retains the exact input `alt` and `src` attributes;
`input_image_attributes_for_state` selects the Image Button requirements using
the actual InputType classification. It checks required nonempty `alt` and
nonempty `src` after borrowing its ASCII-edge-stripped reference. Unknown state
leaves those checks Unchecked. `HtmlInputImageAttributes` stores the original
UTF-16 strings and emits `type=image`, `alt` and `src`; it does not certify label
adequacy, URL validity or image-resource restrictions.

`embedded_dimension_attributes` reports local `width` and `height` author grammar
and exact nonnegative-integer prefix recovery on img/iframe/embed/object/video.
`embedded_dimension_attributes_with_context` also observes actual input state
or a source element's actual parent: input requires Image, and source requires
an HTML picture parent. Unavailable parent evidence remains distinct from an
established other parent or no parent. Missing and failed dimensions receive no
canvas defaults. These [local image values](src/attributes/image_values.rs) use
ordinary allocation and per-original-value UTF-16 admission.

The semantic query `EmbeddedDimensionAuthorConformance` checks resource-dependent
omission and paired author dimensions independently, retaining both raw attributes.
Its [natural-dimension inputs](src/semantics/natural_dimensions.rs) join independent
current resource selections with observations qualified by capture, host, producer,
configuration, selection, content and metadata revisions. The caller supplies
intrinsic CSS-pixel measurements; the crate does not extract or load them.
Known absence, present but unrepresentable measurements and unavailable evidence
have separate outcomes.

`HtmlNonNegativeRational` retains exact nonnegative fractions with nonzero
denominators. Positive natural dimensions use exact cross products for the
inclusive half-unit alternatives, with no rounded ratio. Both authored zeros
can satisfy the pair while omission still fails for a missing natural axis.
Degenerate or unrepresentable arithmetic has an explicit unsupported outcome.
Fact construction has separate record, retained-byte and validation-work limits;
query arithmetic uses the existing semantic work, scratch, digit, byte and
cancellation controls.

<a id="captured-image-dimensions"></a>

The captured `ImageDimensions` query supplies the HTML img/input width and height
getter interpretation over qualified completed producer facts. Its
[owner](src/semantics/image_dimensions.rs) returns named CSS-pixel axes, the
actual Img/Input role and selected branch, inspected attribute evidence and
source origins. `EmbeddedDimensionAuthorConformance` remains the separate
intrinsic author check; getter recovery does not certify author spelling or
resource-dependent omission and ratio requirements.

Only exact HTML img/input subjects apply. Every non-Image input state returns
zero on both axes before consulting rendering, image resources or width/height;
missing or invalid type follows the existing Text-state default. Those dimension
attributes remain `Uninspected`. For img or Image input, actual `BeingRendered`
participation selects the branch. Unavailable rendering remains an outer Missing
result even when both attributes could recover. Rendered elements use supplied
content-box axes, including a genuine zero box; absent or unsupported measurements
remain explicit rather than falling back to attributes or natural dimensions.

Unrendered elements recover exact no-namespace width and height independently
through the common nonnegative-integer owner. A successful prefix, including
` +12tail` or `-0`, overrides that axis; invalid syntax and negative nonzero
values retain their own evidence and leave the axis dependent on image facts.
Original raw admission precedes inspected-value recovery. Attribute observations
distinguish `Uninspected`, `Missing` and retained `Present` with its recovery
outcome. Two successful attributes prove both axes without resource lookup.

Known image unavailability supplies the source's null natural-dimensions
argument and therefore zero for each unoverridden axis. Img Unavailable/Broken
request states differ from missing request evidence; PartiallyAvailable and
CompletelyAvailable establish availability. Input has its separately observed
Available/NotAvailable state. Known no selected resource does not establish
unavailability. An available image requires the actual completed precursor
result: ImgPreferredDensityCorrected, ImgDefaultSized or InputDefaultSized.
The two default-sizing roles identify completed CSS calls with the source's
300×150 default object size; HTML does not execute those CSS calls here.
Missing or unsupported completed axes affect only the axes that still depend
on them. Known axes carry their actual ContentBox, ContentAttribute, zero or
completed-operation origin.

[Image dimension facts](src/semantics/image_dimensions.rs) join capture and host
identities, independently expected resource and participation revisions, current
selection/resource/content/metadata keys, request revisions and the completed
operation's producer/configuration, density/preferred and CSS-sizing revisions.
Context admission rejects stale or contradictory supplied facts even if a query
would bypass their branch. Intrinsic-author natural axes are never substituted
for a completed getter argument. Expected combined revision keys remain in
qualification when optional facts are unavailable.
Producers remain responsible for the truth of completed measurements.

Fact admission separately bounds incoming vector capacities, rational digit
storage and validation work. Fact/native construction and caller cloning use
ordinary allocation. Query execution uses the sole semantic ledger for actual
output-record layout, original inspected raw caps and fallible numeric copies.
Numeric-owner operations use length-derived work bounds and record cumulative
actual capacities, including a copied first axis or numerator discarded after
later failure. Sticky work, digit, byte, allocation or cancellation stops discard
the whole batch; there is no parser cancellation quantum or allocator/RSS promise.

`HtmlNonNegativeRational` is the supported exact CSS-pixel measurement domain,
not a claim that every CSS result is rational. Readback equality retains stored
numerator/denominator representation and evidence rather than reducing fractions.
The [independent cases](tests/semantics_image_dimensions.rs) cover branch influence,
literal origins, joins, raw admission and resource boundaries. The
[pinned img getter](../../references/html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#img-dimensions),
[input getter](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#input-img-dimensions)
and [common kernel](../../references/html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#determine-dimensions)
define this interpretation. CSS production/default sizing, decoding/loading,
selection and DOM unsigned-long conversion/setters remain outside it.

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

`HtmlDirectionSubmissionName` checks exact nonempty `dirname` text and emits
its canonical attribute. `HtmlInputPlaceholder` checks input placeholder text
without CR or LF; textarea placeholders use ordinary exact attributes and may
contain line breaks. The `dirname` and `placeholder` reports retain missing,
empty, raw and local author observations independently of input-state eligibility.
Their `_for_input` companions consume an explicitly supplied `HtmlInputType`;
content-only input calls leave that contextual check Unchecked. The complete
normative state matrix includes Submit for `dirname` and Number for placeholder.

An author-invalid input placeholder retains its raw UTF-16 and separately
exposes the projection removing exactly CR/LF. Textarea keeps its original
multiline text. These operations apply the original per-value cap before role
checks or projection, using ordinary allocation. They do not provide aggregate
work, physical-memory or cancellation guarantees. Submitted direction entries
and value/focus-dependent hint presentation remain separate consumers.

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

`HtmlElementReferenceFacts` supplies actual CommandFor and PopoverTarget pointer
slots, qualified to the capture and host revision. Unavailable, known null and
an actual Element pointer remain distinct. The checked supplement bounds moved
record capacity and validation work; caller allocation is outside those limits.
`CommandForElement` and `PopoverTargetAssociatedElement` retain raw
content and the selecting branch. An explicit pointer must satisfy the source
scope relation; an out-of-scope pointer returns null without an ID fallback.
Only a known null pointer enters exact ID lookup in the ordinary tree, selecting
the first matching element, including foreign elements. Unknown influencing
relationships remain missing information.

`CommandValidity` retains that association and the existing command report.
Unknown commands have false validity, Custom commands support any Element, popover
commands support HTML elements and dialog commands require an HTML dialog.
An established null association has no validity value. `PopoverTargetElement`
separately checks actual input state, disabledness, form ownership and submitter
role before resolving the target. A selected foreign target returns null before
popover-state inspection, as specified by the
[bounded source determination](../../references/SOURCE-EDITIONS.md#foreign-popover-target-interface-guard).
These queries use the session's cumulative work, retained-output, scratch and
cancellation owner. They perform no activation, events or dialog/popover
transitions; author reference conformance remains a distinct requirement.

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

`SelectValueMissing` observes the pinned required/no-selection-or-only-placeholder
condition from supplied current selectedness. It reuses the option list,
placeholder and option-value owners. Two selected options, known nonfirst or
optgroup membership, or a known nonempty selected value can establish false
despite unrelated unavailable facts; influencing unknowns remain Missing.
Consulted intermediate text still consumes the shared admission budget.
This immutable Boolean observation performs no validation, selection reset or
live validity-state transition.

`CheckableInputValueMissing` observes the pinned Checkbox and Radio missingness
conditions from supplied current checkedness. Checkbox uses required presence
and its own checkedness. Radio includes itself, then groups other radios by
exact nonempty name, ordinary tree and actual form owner; absent or empty own
name yields a singleton. A disabled required radio still contributes.

A checked witness can establish false despite unrelated unavailable facts.
Otherwise the bounded summary preserves possible group membership and genuinely
influencing missing evidence; unavailable checkedness is never inferred from
authored checked attributes. Consulted type, name and form values retain their
existing admission caps, traversal releases scratch on every result, and terminal
failures preserve the session's sticky, atomic-batch contract. This immutable
Boolean observation performs no validation or radio-state transition.

`InputTextValueMissing` and `TextareaValueMissing` observe the required,
mutable, empty-current-value conditions for the six text input states and
textarea. They reuse the supplied current-string lookup and ordinary disabledness
owner, including fieldset and first-legend rules. Authored value attributes and
default descendant text do not substitute for current state.

Required and readonly use exact attribute presence. A known nonempty current
string or known immutability can establish false despite unavailable facts;
otherwise the first influencing missing fact remains explicit. Current strings
retain the checked supplement's aggregate UTF-16 limit, independently of selected
attribute admission. These queries add no traversal scratch or value copying and
retain the session's sticky terminal failures and atomic batches. They observe
missingness without performing validation or a live state transition.

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

<a id="captured-map-name-author"></a>

[Map names](../../references/html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#attr-map-name)
must be specified, nonempty and free of ASCII whitespace. The local
[HtmlValueInterpreter::map_name report](src/attributes/map_name.rs) admits the
original raw value before expanded-name applicability and preserves Missing,
Empty, invalid and Unapplied states without a default. The existing
`HtmlSpaceToken` validator owns the syntax; `into_map_name_attribute` emits
the exact checked UTF-16 as canonical no-namespace lowercase `name`. No trimming,
case folding or normalization occurs. Applicable present reports leave context
Unchecked because own-id equality and same-tree name uniqueness are separate.

The captured [MapNameAuthor query](src/semantics/map_name.rs) returns local syntax,
the specified own-id relation and same-ordinary-tree uniqueness independently.
A specified empty or invalid name still participates in exact contextual
comparisons; an equal own id is not a global id-conformance certificate.
Absent name skips unrelated id and tree inspection and returns NoName for both
contextual observations. A duplicate carries an actual other-map witness, even
across earlier nonterminal unknown coverage, and stops before that witness's
children or later values. Unique requires a complete influencing traversal;
otherwise the first unresolved dependency remains inner Missing. Shadow and
template host associations do not join ordinary trees. Known(report) delivers
these observations, not an overall author-conformance verdict.

Native emission and standalone interpretation use ordinary allocation; the
standalone report has only its per-original-value UTF-16 cap. The captured query
uses the [semantic session](src/semantics.rs) ledger, precharged borrowed local
assembly, full consulted name/id admission before equality, one fallibly retained
data record and actual scratch-frame capacities. Shared input backing and fixed
wrappers are not charged as new backing allocation. Current scratch is released;
cumulative allocation, retained-output and peak usage remain charged after
discarded results. Terminal raw, work, allocation, representation or cancellation
failures stay sticky and discard the whole batch; a later duplicate cannot mask
them. [Local](tests/attributes_map_name.rs) and
[captured tests](tests/semantics_map_name.rs) cover these boundaries.
Map processing/selection, URLs, geometry, activation and live collections remain
separate operations.

`MapAreas` returns HTML area descendants of a map in ordinary tree order,
including areas beneath wrappers and nested maps. Attribute values and visibility
do not filter membership. Separate shadow and template trees are excluded;
influencing unavailable child order remains missing rather than a partial list.
`HtmlAreaCollection` retains capture-qualified subject identities from one
immutable observation, using the shared traversal and semantic resource ledger.
The [map areas clause](../../references/html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#dom-map-areas)
defines membership; this result supplies no live collection, named lookup,
image-map selection, geometry or activation.

`UsemapReference` observes an HTML img's authored usemap against its ordinary
tree. It reports the existential map-name witness separately from the first
map whose id or name matches the parsed suffix. A selected id-only match retains
its diagnostic even if a later map supplies the author-definition name witness.
For `#`, a recovered empty map name can supply that witness while processing
still returns null. This follows the pinned [hash-name rules](../../references/html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-a-hash-name-reference)
and [image-map processing](../../references/html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#image-map-processing-model).
Once the processing result is fixed, a later known name witness can resolve an
unavailable subtree. Influencing unavailable topology yields missing evidence.
The query preserves
capture qualification, whole-value admission, metered traversal and atomic batch
results; it does not certify overall conformance or perform geometry or activation.

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

`TabIndex` returns the captured static getter value for HTML, SVG and MathML
elements. It recovers an in-range signed 32-bit integer from the selected raw
attribute; missing, failed and out-of-range values use the source's element
defaults. The readback retains the exact attribute and distinguishes these
origins. A defaulting HTML summary uses `SummaryForParentDetails`; an explicit
in-range value needs no parent or child-order evidence. The shared numeric
scanner performs fixed-width projection without a digit allocation. Whole raw
admission, conservative work charging and atomic batches remain session-owned.
This getter does not establish focusability or focus navigation.

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

`OwnAutocapitalizationHint` preserves the element's nonempty setting or the
source-defined form-owner fallback, with its canonical IDL spelling.
`UsedAutocapitalizationHint` separately applies input-type suppression and
selects the actual editing host where required. `UsedAutocorrectionState`
applies its own local/form rules and exposes the Boolean getter; it does not
inherit through the editing host. `ComputedWritingSuggestions` follows Default
states through ordinary parent elements, including foreign parents, and retains
the deciding attribute and identity. Unknown influencing relations remain
missing; established null ownership and tree boundaries retain their source
meaning. Reports keep only the observations their branch actually consulted.

These four queries reuse the qualified semantic session, raw admission,
traversal, sticky failures and atomic batches. Actual output capacities are
charged, including both nested records when a used hint retains an own hint.
Caller-initiated cloning is outside query allocation accounting. These static
readbacks do not perform input-method actions, suggest text or select user
preferences.

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

The constraint-validation-candidate query evaluates the static HTML candidacy
predicate from native control category, input type, applicable readonly
presence, submit-button classification, effective disabledness and ordinary
datalist ancestry. Confirmed non-submittable elements return false without
unrelated control facts. Missing custom classification or relationships remain
unresolved unless an independent barrier proves false for every possible case.
The operation reuses captured submitter and disabledness owners, including
first-legend handling. Consulted type and disabled contents retain those owners'
raw admission; direct readonly presence needs no content grammar scan.
Terminal admission, work and cancellation failures stay terminal. Results use
the shared qualified session and atomic batch output. This supplies native
willValidate semantics; it does not execute validation, produce validity or
messages, or expose a live ElementInternals object.

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

`AdvisoryInformation` selects an own no-namespace `title`, including empty,
or inherits through ordinary parent Elements. `HtmlAdvisoryInformation` retains
the normalized UTF-16 value and the actual selecting subject; a selected empty
title differs from reaching a known boundary without selecting an attribute.
Normalization replaces CRLF and remaining CR with LF, preserving other units.
The [selected reconciliation](https://github.com/bj-data-eng/surgeist/issues/1134#issuecomment-6116993954)
of generic inheritance and the specific link/style clauses treats title-less
HTML link/style as known empty boundaries. Own titles win first. Foreign
ancestors can bridge ordinary ancestry but supply neither HTML title nor that
exception; Document, Fragment and ShadowRoot parents stop inheritance.

Missing influencing parent evidence remains missing. The entire selected raw
value is admitted before normalization; each read, reservation and write uses
the shared semantic ledger, with cumulative output capacity, sticky failures
and atomic batches. No traversal scratch is added. Fixed shared wrapper
allocation retains the existing provider limitation. This query computes
advisory information; link titles, stylesheet-set names and tooltip or
accessibility exposure remain separate operations.

`TitleText` returns the exact direct-child Text data of an HTML `title` element.
`DocumentTitle` selects the first HTML title in ordinary document tree order,
then strips and collapses only ASCII whitespace. `HtmlDocumentTitle` retains
the actual selecting element: an empty selected title has a source, while
complete absence has none. Unknown influencing children remain missing;
shadow trees, template contents and detached nodes do not enter this selection.
Multiple direct document Elements are query-local unsupported input.

An exact SVG `svg` document element instead selects its first direct SVG
`title` child. The [selected absence determination](../../references/SOURCE-EDITIONS.md#absent-svg-document-title)
returns known empty without a source when those children are complete and no
such title exists. It searches neither deeper SVG titles nor an HTML fallback.
Text extraction and whitespace processing otherwise follow the HTML clauses.
The existing child-text owner and semantic ledger admit both the raw and
normalized buffers, traversal capacity and cumulative output; failures remain
sticky and batches atomic. Fixed shared wrapper allocation retains the
existing provider limitation. These are captured getters; title mutation and
browser lifecycle notification remain separate operations.

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
The generic proofless head continuation still suspends at script and template starts.
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
Other resettable control creation retains untouched-token continuations when
its required machinery is unavailable.
Source-defined error-and-ignore rows still execute. The fresh ordinary driver
uses the [private EOF completion](#private-ordinary-eof) described below;
these private operations produce no public parse report.

Ordinary frameset documents use that same constructor-proved driver. AfterHead
and eligible InBody entry lead to InFrameset, AfterFrameset and AfterAfterFrameset.
Body removal runs the existing form effects while retaining the current-form
pointer and formatting list; trailing whitespace can reconstruct formatting.
Frame insertion returns at the actual node-document's null browsing-context
guard. Noframes uses RAWTEXT and restores its saved caller mode. Mixed character
runs preserve exact origins while diagnosing and omitting nonwhitespace and NULL.
Generic or imported owners cannot establish the required constructor proof.
This adds no hosted navigation or fragment execution. Ordinary EOF completion
uses the same [private end owner](#private-ordinary-eof).

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
Association scans are metered and can require quadratic work. Other resettable
controls, unfinished table rows, contextual fragment execution, hosted parsing
and public diagnostic/source-map/report transfer remain unfinished.
Ordinary EOF uses the [private completion owner](#private-ordinary-eof).

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

Ordinary output parsing now uses the same native insertion and listed form
association operations. Fresh creation completes its actual empty reset before
parser association and retains the null default-value override in the existing
controlled payload registry. Current and default observations both read ordinary
descendant Text in tree order, including nested elements; they retain actual
arena/controller qualification and validate destination ownership before copying.
They share the canonical ordinary traversal, charge variable visits and copies,
and admit actual UTF-16 buffer capacity. Text merges and moves need no output
value cache. Stops preserve any owned partial scratch and refuse observation or
freeze; terminal cleanup performs no useful reset. General setters, nonnull
overrides, form reset, hosted initialization and public parse completion remain
separate work.

Hidden input creation appends initial attributes in source order. A value seen
while the fresh input is still in the Text state loses CR and LF; changing to
Hidden then writes a nonempty internal value back to the native value attribute.
Creation resets the resulting native value before canonical form association.
The ordinary body and table branches use this same owner; table diagnostics
precede direct insertion, and the body branch preserves frameset eligibility.

Private active and frozen observations distinguish internal and native default
values and retain the original token and value-attribute ranges for an actual
creation writeback. Replaced native and raw backing coexist under admission;
successful replacement releases obsolete backing while retired registry slots
remain charged until disposal. Stops retain partial owners, refuse publication
and perform no reset during cleanup. Repeated scans and copies may be quadratic
and remain charged and polled. Other input states, hosted initialization, general
value setters and public parse completion remain separate work.

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

This is a structural checkpoint. Other unavailable control creation preserves
its original token before unsupported effects.
It does not establish complete InTable behavior,
contextual fragment parsing or public report publication. The fresh ordinary
driver has its separately qualified [EOF completion](#private-ordinary-eof).

Ordinary select, option, optgroup and selectedcontent construction now uses the
same fresh standalone driver and controlled arena. Modern InBody dispatch and
actual stack removals carry its existing borrowed constructor proof through
implied ends, formatting recovery and table delegation. Ordered attributes,
initial select reset and canonical form association preserve source order.
Selectedness, dirtiness, nearest-select caches, disabledness and user validity
are actual private native state; a shared per-unit integer scanner supplies the
display-size predicate without materializing a public attribute report.

The existing construction owner retains finite effect frames, static target
lists and intrinsic FIFO tasks. Actual pre-insert/removal hooks update caches
and selectedcontent in source order, including reinsertion into the same parent.
Fragment children are removed and inserted as a complete sibling batch before
the parent's children-changed and static post-connection processing. Iterative
same-arena cloning preserves exact UTF-16, attributes and original source maps
with new identities. HTML script clones receive fresh metadata, copy only
already-started, and use the existing staged Disabled preparation owner for
actual connection and children-changed hooks.

Only source-defined checkpoints drain queued tasks, including tasks appended
while draining. Freeze refuses pending intrinsic work before taking storage;
the active owner may reach a later legitimate checkpoint. A terminal stop
retains partial frames, lists, clone backing and task owners for disposal,
without resuming useful work. Native string bindings are admitted before
allocation and installed immediately after it, preserving strict alias checks
and reclamation even at an interrupted clone boundary. Work and storage remain
finite and charged; repeated scans and child moves may be quadratic.

These are private construction operations, not a live DOM mutation API. The
separate DOM moveBefore primitive, captured fragment initialization,
imported templates with unavailable facts, shadow trees, external script execution
and other
unimplemented control state remain outside this capability. Ordinary EOF now
runs two source-positioned intrinsic checkpoints after canonical end pops,
through the [private completion owner](#private-ordinary-eof). Public report
publication remains separate.
The [select construction owner](src/construction/parser_session/select.rs),
[managed clone owner](src/construction/parser_session/clone.rs) and
[independent private regression cases](src/construction/parser_session/select_tests.rs)
record the consumed boundary.

<a id="private-ordinary-templates"></a>

The fresh standalone Disabled driver also constructs ordinary templates in the
same controlled arena. A lazily created inert Document owns their separately
hosted contents fragments; nested templates reuse that actual document and
ordinary template children remain empty. Actual node-document bindings,
[template contents and insertion targets](../../references/html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#template-contents)
and the real template insertion-mode stack govern creation, table routing,
end-tag recovery and EOF unwinding. Borrowed CreationContext carries the actual
open-template identity, not a caller-supplied parsing flag. Template adopting
steps and iterative contents cloning share the existing arena, source-ordered
construction effects and intrinsic FIFO. Removal effects complete before
node-document adoption, which completes before insertion effects.

Reachable template for patching uses actual named marker/start ProcessingInstructions
and complete pseudo-attribute maps. Search follows ordinary descendants and
exact names; start/end nesting counts following siblings independently of their
names. An admitted static sibling list is completed before source-ordered
removal. Insertion uses the actual target and a still-parented end marker.
Explicit template end removes currently parented markers and clears patch state;
Template EOF recovery unwinds stacks/modes without those explicit-end effects
or an invented task
drain. The fresh operation's established shadow permission is false, so a
recognized declarative shadow mode takes the source-required ordinary-template
branch before considering for; it attaches no shadow root.

The [PI-map owner](src/construction/parser_session/pi.rs) retains original native
target/data and exact lexical maps alongside independently admitted decoded
backing. The [whole XML-stylesheet grammar](../../references/xml-stylesheet-pseudo-attributes--REC-xml-stylesheet-20101028--a0e1a5198a21.md#pseudo-attributes)
and [XML grammar closure](../../references/xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md)
require exact Unicode XML names, XML scalar validity, five predefined entities
and numeric references. Duplicate names or malformed syntax invalidate the whole
map; resource/cancellation failures remain terminal. This is not HTML entity
recovery, XML document preprocessing or a generic XML parser.

Template-content script preparation preserves the real prefix: parser-document
clearing and force-async changes precede the empty-source/connectivity guards.
An eligible disconnected script returns Disconnected before type classification
and the scripting-disabled return; inertness is not an omitted callback or
external execution. Active/frozen private observations preserve arena/controller
qualification. Pending effects/FIFO work and incomplete template/document/PI
facts prevent immutable publication; terminal cleanup disposes retained owners
without running useful effects.

The [template state owner](src/construction/parser_session/template.rs),
[template dispatcher](src/parser/tree_builder/template.rs) and
[origin record](PARSER-ORIGINS.md#ordinary-template-extension) identify this private,
test-compiled boundary. Imported templates with unavailable patch facts, captured
context-sensitive fragments, shadow attachment and complete public
stop-parsing/report publication remain unfinished. No public captured
template-for reference query or live DOM follows. Repeated registry/topology
scans, comparisons, copies and moves can be quadratic; finite charged/polled work
establishes no linear-time or allocator/RSS guarantee.

Ordinary non-script MathML and SVG now use the same controlled creation and
tree driver. The actual current namespace selects CDATA admission, including
foreign HTML integration points. MathML text points, encoding-qualified
annotation-xml and SVG foreignObject/desc/title select HTML processing only for
the source-defined token cases. Encoding matching is ASCII-insensitive without
trimming; broader scope membership does not establish integration.

The [foreign creator](src/construction/parser_session/foreign.rs) retains original
lexical backing and maps alongside independently owned adjusted names. It
applies the complete SVG element/attribute tables, MathML definitionURL and the
eleven qualified XLink/XML/XMLNS attributes, preserving unknown colon names and
exact non-scalar UTF-16. The existing registry owns namespace and prefix tickets
through native binding, attribute transfers, selectedcontent cloning and cleanup.
Clones retain qualified values and source order with distinct identities/backing;
foreign control spellings acquire no HTML form/reset metadata.

The [foreign dispatcher](src/parser/tree_builder/foreign.rs) preserves NULL
replacement, comments, processing instructions, namespace diagnostics, breakout,
end-tag recovery and actual table foster placement. These operations admit
coexisting original/adjusted buffers, map slots, physical capacity and variable
visits through the sole controller; cancellation and first failure remain sticky.
Interrupted creation and qualified cloning retain every owner for disposal and
refuse freeze. Repeated registry/topology scans may be quadratic; no linear-time
or allocator/RSS guarantee follows. The separately qualified
[restricted SVG script operation](#private-svg-script) uses the same foreign
creator and dispatcher. Ordinary EOF uses the separately qualified
[private completion owner](#private-ordinary-eof); public report publication,
contextual fragments and imported templates still require their own
construction facts and effects.

<a id="private-svg-script"></a>

The fresh private standalone driver now retains a distinct processed state for
exact SVG-namespace script elements in the same arena. The
[SVG state owner](src/construction/parser_session/svg.rs) implements the
[explicit source selection](../../references/SOURCE-EDITIONS.md#offline-disabled-svg-script-and-clone-sources)
over [SVG Tiny 1.2 processing](../../references/svg-tiny12-script-processing--REC-20081222--2f36eb76440c.md#ScriptContentProcessing):
step 1 returns unchanged for an already processed script or one outside the
ordinary document tree. An eligible unprocessed script reaches the actual
step-3 processed=true mark. Fresh parser-created scripts start false; unavailable
or imported metadata does not stand for false.

This is a restricted processing attempt. A separate closed no-loader capability
suppresses every resource acquisition, including network, data and same-document
references; Disabled suppresses execution. The actual no-host/no-listener/
no-observer/no-retained-notification-producer domain permits notification
elimination. Raw attributes and content remain intact. No URL classification,
failed-load result, inline fallback, execution or SVG load task is fabricated.
This is not literal SVG Tiny Disabled behavior, HTML prepare-script step 18,
a named SVG rendering mode or complete SVG support.

The canonical insertion command carries its actual Parser or NonParser cause
through the existing effect frames. Parser insertion defers processing to the
specified foreign end. An exact current SVG script end, including a self-closing
start, uses the same created node: retain its handle, pop it, save/install the
tokenizer insertion point, increment nesting and set pause, process, then
decrement nesting, clear pause at zero and restore the previous insertion point.
The [foreign-end wrapper](src/parser/tree_builder/head.rs) consumes the real
lexical payload without entering HTML Text/ScriptData mode or draining the
intrinsic FIFO. Foreign CDATA, entities, qualified XLink attributes and native
sibling order remain with their existing owners. EOF or an unrelated breakout
does not invent a script-end invocation.

Actual nonparser connection uses the existing static ordinary postconnection
list in document order, after removal, node-document adoption and insertion.
Creation provenance does not determine later insertion cause. Template hosted
contents are not ordinary ancestor edges: fresh unprocessed scripts there stay
disconnected and false, while actual patch placement or adoption followed by
ordinary connection can make an eligible false script true. Removing or reconnecting a true script
never re-arms it. The [existing clone owner](src/construction/parser_session/clone.rs)
copies the actual source bit to a distinct clone before insertion can observe it,
including deep selectedcontent and template-contents clones. This is the
[bounded clone completion](../../references/webkit-svg-script-clone--73aa6c89e2cb-metadata.md),
not an SVG Tiny cloning clause or copied WebKit implementation. HTML script
metadata and preparation remain separate.

Fresh SVG slot admission precedes lexical transfer; its fixed binding write is
admitted before native allocation. Later owned transfer work has terminal-safe
cleanup boundaries. Registry capacities, scalar slots, clone bindings, effect
frames and wrapper layouts use the sole controller and actual physical layouts.
A retained SVG wrapper blocks useful continuation and normal freeze; successful
freeze validates complete bound SVG facts. Cancellation or a limit failure
retains the coherent prefix and first stop, then releases lexical/native backing,
maps, registry capacities, handles and tickets without rescue processing.
Repeated scans, subtree walks, copies and moves can be quadratic; finite charged
and polled work is not linearity, allocator cancellation or an RSS guarantee.
These operations remain private and test-compiled. Imported/hosted execution,
captured context fragments, shadow attachment and public diagnostic/source-map/
report transfer remain separate work. Ordinary EOF completion is described below.
The [origin record](PARSER-ORIGINS.md#restricted-svg-script-extension)
and [source attribution](NOTICE.md#restricted-svg-script-processing) record the
distinct HTML, Tiny and implementation-provenance terms.

<a id="private-ordinary-eof"></a>

The fresh test-compiled ordinary driver completes the selected private end-of-parsing
operation after the actual final EOF token's insertion-mode recovery. Its
qualified construction Document starts with source-initial Complete readiness,
post-load readiness false and parsing unfinished. Initial Complete is not proof
of parsing completion; source-free native builders, inert template documents and
captured documents acquire no ordinary completion fact.

The end owner makes the insertion point undefined and updates readiness to
Interactive before popping the open-elements stack through its canonical owner.
Actual pop effects,
including option/selectedcontent updates and clones, retain their source order.
The selected DOMContentLoaded disposition precedes both checkpoints.
The existing intrinsic FIFO runs at two distinct
[source-positioned checkpoints](../../references/html-spin-event-loop--877d6146487f.md#spin-the-event-loop),
even when the respective wait goals are already satisfied. The fixed
complete-readiness continuation is retained before the outer post-load milestone;
that milestone is written while readiness remains Interactive, then the
continuation updates readiness to Complete. The actual null browsing context
excludes the remaining load/navigation tail.

This is the [selected closed construction reduction](../../references/SOURCE-EDITIONS.md#fresh-standalone-eof-lifecycle-sources),
not browser lifecycle execution. Constructor and admitted-producer provenance
establish the absent executable-script, loader, timing, event and host domains;
Disabled alone, an empty queue or a null browsing context is insufficient.
No global task queue, browser event, timestamp, navigation result or later DOM
replay intent is synthesized. EOF does not invent an SVG script-end invocation.

Successful private completion retains the whole original driver: the same
controller, arena, diagnostics, lexical events, source evidence and capacities.
Its qualified completed owner is neither a second parser nor a public parse
report. First stop remains sticky through readiness, pops, cloning, checkpoints
and owner transfer; failed completion cannot publish a partial success. Terminal
cleanup releases actual retained storage without useful pops, task execution or
readiness progression. Actual fixed layouts and vector capacities remain admitted
by the sole controller; existing scans, copies and moves can be quadratic.
Recurring intrinsic tasks can exhaust the finite work budget; a finite input
does not guarantee successful completion. No linear-time, allocator/RSS or
cancellation-preemption guarantee follows.

Public parsed document/fragment results, atomic diagnostic/source-map/report transfer
and contextual fragment execution remain unimplemented. Private completion does
not certify unsupported input families, captured/hosted execution, shadow
attachment, full parser accounting or whole HTML contribution completion.
The [end owner](src/parser/tree_builder/end.rs),
[qualified completion facts](src/construction/parser_session/completion.rs) and
[origin record](PARSER-ORIGINS.md#ordinary-eof-completion) record this boundary.

<a id="private-conformance-observation"></a>

The test-compiled [completed native observer](src/construction/parser_session/native_view.rs)
lends the whole successfully completed ordinary owner described above. It borrows
qualified native records and UTF-16 units through the same controller; it does
not freeze, clone backing, extract an incomplete arena or manufacture a public
parse report. The [structural adapter](src/parser/tree_builder/conformance.rs)
runs the real ordinary driver through completion, compares its actual output
with independently decoded expected data, and releases consumer storage and the
completed owner through their existing cleanup paths.

The bundled [html5lib-tests subset](tests/fixtures/html5lib-f994590f/tree-construction/README.md)
at revision `f994590f528ac8b6073665791ddb1ed85c66dfb2` contains eight complete
data files, with 169 cases. All 169 are decoded as fixture-format coverage.
Exactly ten stable file/ordinal bindings are replayed through the completed
ordinary parser with scripting Disabled and no fragment context:

| Whole retained file | Replayed one-based case ordinals |
| --- | --- |
| adoption02.dat | 1 |
| namespace-sensitivity.dat | 1 |
| quirks01.dat | 1 |
| tests14.dat | 1, 6 |
| tests24.dat | 7 |
| tables01.dat | 1, 4 |
| comments01.dat | 1 |
| template.dat | 1 |

The adapter compares native document mode, expanded names, prefixes, exact
UTF-16 text and attributes, child order and parent/owner relationships.
Associated template contents remain separate fragment trees, and distinct equal
siblings retain distinct identities. Native attribute order is preserved;
comparison with the corpus's sorted attribute dump uses an explicit permutation
rather than rewriting the native output. Additional literal native tests cover
processing instructions, absent versus empty doctype identifiers, template and
shadow ownership and properties. Source-literal completed-parser tests separately
cover current HTML processing instructions, isolated UTF-16 surrogates,
CRLF normalization, supplementary character references, NUL recovery and the
doctype creator's empty identifier strings.

The corpus edition is implementation evidence, not a replacement for the pinned
HTML language. Its historical format statement that HTML cannot emit processing
instructions disagrees with this pinned edition; current source-literal
expectations exercise that distinction. The corpus doctype dump also collapses
absent and empty identifiers, so source supplements disambiguate actual parser
creation without erasing native null/empty fields. Retained `#errors` and
`#new-errors` metadata is explicitly unverified: ordered diagnostics and source
offset comparison still need their own consumer. The other retained cases,
fragment contexts and scripting-Enabled replay remain unexecuted here; newer
HTML disagreements require source-qualified expectations before expansion.

Expected fixtures and native literal construction use ordinary test allocations.
Actual completed-parser observation and its flat consumer vectors share the
parser's admission, work, cancellation and physical-capacity ledger. Resource
failure and cancellation tests check sticky first stop, retained-owner integrity
and consumer-ticket cleanup. This does not establish a linear-work, allocator
preemption, RSS, full-corpus conformance, public parse-report or whole-parser
accounting guarantee. The [fixture attribution](NOTICE.md#html5lib-tests-fixtures)
links the exact bundled MIT license and contributor credits.

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
