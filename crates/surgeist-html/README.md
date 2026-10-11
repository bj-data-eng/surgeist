# surgeist-html

This crate owns the pinned HTML language foundation. Its initial public surface
provides immutable exact UTF-16 strings, namespace identity, qualified names,
ordered raw attributes, native immutable document/fragment construction,
known-name inventories, common intrinsic value operations and bounded immutable
captures of HTML semantic facts.
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
processing instructions validate the selected HTML target/data syntax; raw
shared names remain available without generic DOM name validation.

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
keyword recognition, author conformance and the selected intrinsic state. The
implemented overlays cover twelve global enumerated attributes, template
shadow mode/slot assignment, eleven element-specific keyword rules and twenty
Boolean contexts. Input types retain all twenty-two standard keywords and their
Text fallback; button types retain the distinct Auto fallback without treating
it as an authored keyword. Media preload retains implementation-defined missing/invalid
policy separately from its recognized keywords; ordered-list marker types use
exact case comparison, and track kinds retain their distinct missing/invalid
defaults. It preserves invalid
spelling when a clause supplies a fallback; keyboard hints retain their lack of
a defined fallback. Native keyword values construct their canonical raw
attributes. These views do not compute inheritance or claim runtime effects,
and the remaining attribute families are tracked by the contribution.

`HtmlCaptureBuilder` freezes one bounded, flat capture of document, node,
registry and relationship facts. Capture-qualified subjects are independent of
native result IDs and future canonical DOM IDs. Unavailable facts remain
distinct from known absence; lookups and traversal borrow the captured records.
Every retention capacity is explicit, and rejected changes preserve logical
content and usage. Document self identity is covered by the subject count;
relationships count references to other retained records.

`HtmlFragmentContext` selects an actual captured element or hosted fragment,
retaining its context and target documents, integration facts and nearest form.
It derives the initial tokenizer/insertion state and checked scripting mode:
Inert is the fragment default, disabled context forces Disabled, and unavailable
execution is a typed error. Captured processing-instruction target/data preserve
producer values independently of strict native author syntax. Capture and context
selection perform no parsing, live mutation, script execution or effect
publication. The parser still owns recovered metadata, contextual construction
effects, diagnostics, provenance and its separate execution controls.
