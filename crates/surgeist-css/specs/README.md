# CSS foundation standards catalog

[catalog.json](catalog.json) defines the selected standards and their ownership
for Surgeist's CSS foundation. Its audience is implementers and reviewers of
CSS grammar, specified values, expansion, normalization, and serialization.
The [root guide](../../../AGENTS.md) owns repository boundaries and workflow;
this catalog owns the milestone's standards selection. Higher-priority task
instructions still govern authorized work.

Catalog membership is a target contract, not an implementation claim. Current
source and behavior-revealing tests establish support. The CSS conformance
ledger tracks requirement evidence and implementation findings; this directory
does not duplicate its work status.

The September 8 Color 5 `light-dark()` color and image/`none` productions
have distinct authored feature records under `I-COLOR5-20260908`. They preserve
both branches for downstream used-scheme selection. Their checked constructors
enforce complete child depth, and specified output retains authored `none` and
uses ordinary Standalone color children. The [reference](../docs/reference.md)
records the pinned source and frozen WebKit evidence for that child-role
interpretation. Existing June Color 5 feature records retain their earlier
bounded provenance. The same September source defines the complete authored
`contrast-color(<color>)` feature: it preserves one checked symbolic input,
uses Standalone child serialization and shares the enclosing resource budget.
Downstream style owns contrast evaluation and white/black selection.
The same dated source defines authored `device-cmyk()`: legacy four comma-separated
number components without alpha, or modern four number/percentage/`none` components
with optional slash alpha. The checked model retains unbounded ink values and
symbolic calculations, reports contextual device identity, and shares enclosing
numeric/color/image depth and serialization budgets. Declared Standalone/Mix
output scales direct percentages by 1/100; Origin retains domains and explicit alpha.
The [reference](../docs/reference.md) distinguishes this authored contract from
computed §11.5 serialization. Device/profile conversion, execution and computed
output remain unfinished, including the §6/§10.3 representation conflict.

| Selection | Entries |
| --- | ---: |
| Snapshot 2026 official definition | 24 |
| Snapshot reliable candidate recommendations | 8 |
| Snapshot fairly stable modules | 10 |
| Snapshot rough interoperability modules | 22 |
| Registered extension modules | 13 |
| Published Grid 3 extension | 1 |
| Explicit `backdrop-filter` exception | 1 |
| Total | 79 |

The four Snapshot tiers select 64 module identities. All applicable authored
grammar is selected across those modules, the 13 complete registered extensions,
and Grid 3. A module that only defines runtime APIs remains in the catalog with
its downstream ownership; its inclusion does not create fictitious stylesheet
syntax. The `backdrop-filter` entry selects one property and does not select
Filter Effects 2 in full.

## Reading a pin

`publication_cutoff_utc` freezes selection at **2026-09-10 02:16:24 UTC**.
`latest_published_url` records discovery, while `publication.url`, `sha256`, and
`bytes` identify the chosen published representation. The catalog deliberately
uses the latest published editions available at that cutoff, which can be newer
than editions cited in the Snapshot bibliography or the existing runtime
conformance registry.

W3C module publications use dated URLs. Fullscreen is the single full-module
publication-model exception: W3C discontinued its document and explicitly
identified WHATWG as its successor. The catalog retains that Note as supersession
evidence and pins an immutable WHATWG commit snapshot for the same module.
It does not implement the withdrawn 2012 draft. The `backdrop-filter` exception
instead pins the imported
repository grammar to a full root commit and exact source-file hashes.

Hashes cover unmodified retrieved body bytes or Git blob bytes, as specified in
`selection_policy.hashes`. They identify the representation rather than asserting
that a publisher can never change a dated URL. CSS2 is a multi-document
publication: its entry also hashes the linked HTML chapters and separate errata.
External scripts, images, stylesheets, and linked specifications are not part of
a document hash. No external document is bundled in this directory.

## Required external definitions

`normative_definitions` pins narrowly required definitions from outside the
fully selected modules. These records leave the 79 module entries and their
counts unchanged. Each definition identifies its source, exact sections, the
selected requirements that need it, and authored versus downstream ownership.

Overflow 3 imports only Box 4 §2.3's `<visual-box>` production for
`overflow-clip-margin`: `content-box | padding-box | border-box`. CSS checks this
property-specific domain against the shared box-edge keywords. Clip geometry
and painting remain downstream; this import does not select Box 4 in full.

Position 3 §3.2 also refers directly to Box 4's four-side margin assignment
rule. Its narrow import supplies the physical top/right/bottom/left repetition
for `inset`; Logical 1 §4.7 adds a separate logical role order. The unresolved
physical/logical reset membership remains explicit rather than inferred from
that assignment rule. These two imported definitions do not select Box 4 in
full.

The host pseudo-class signatures and argument grammars come from published
Scoping 1, and `::part()` grammar comes from published Shadow Parts 1. Their
identities matter to Pseudo-Elements 4's distinction between valid syntax and
selectors that never match after an element-backed pseudo-element. Conditional
Rules 5 and CSSOM also refer to `::part()` and `::slotted()` by identity.

Scoping 1's latest published edition is from 2014 and lacks `::slotted()`.
Selectors 4 explicitly references its renamed successor, CSS Shadow 1, which has
no dated publication at the cutoff. The catalog therefore pins an immutable
CSSWG source revision for only `::slotted()` grammar, its tree-abiding
pseudo-element suffix, and its specificity. The source revision and line anchor
identify the exact text; the moving editor page is not a substitute pin.
Selectors 4's illustrative Note corroborates that suffix behavior but does not
independently select the rest of Shadow 1.

These imports do not select whole Scoping, Shadow, or Shadow Parts modules.
Shadow-tree construction, matching, slot assignment, part forwarding, and live
APIs remain with style and root integration. Other features in those documents,
including obsolete shadow selectors and `:has-slotted`, are not imported by
these records.

Fonts 4 permits `env()` in font-palette descriptor values. The catalog pins the
published 23 September 2025 Env 1 definition for the function grammar,
environment-reference identity, property/descriptor grammar deferral and
shorthand pending semantics. This imports required definitions without selecting
Env 1 in full. Environment lookup, substitution execution and invalidation belong
downstream. The source's individually identified open questions about broader
insertion locations and substitution timing do not waive its explicit authored
property and descriptor requirements.

Fullscreen's selected snapshot moves the `::backdrop` definition to Position 4.
The catalog imports the published 7 October 2025 definition and its fully-styleable
classification. Pseudo-Elements 4 makes fully styleable pseudo-elements tree-abiding,
which determines their admissibility after `::slotted()`. Top-layer membership,
box generation and painting remain downstream; this does not select Position 4
or its `overlay` property in full.

Values 4 mathematical grammar also requires the numeric-type definitions from
Typed OM. The catalog pins the published 21 March 2024 edition for dimension
exponents, percent hints, type matching, addition, multiplication and inversion.
This resolves Values 4's living algorithm links under the catalog's publication
cutoff, despite its older bibliography date. CSS owns intrinsic type admission;
arithmetic execution, unit resolution, rounding and contextual clamping remain
downstream. Typed OM objects, DOM maps and JavaScript APIs are not imported.

Conditional Rules 5 size queries explicitly reference Values 5 tree-counting
functions. The catalog pins the published 11 November 2024 edition for only
`sibling-count()` and `sibling-index()`, their argument-free signatures and
integer numeric integration in container size queries. CSS retains symbolic
nodes and checks the enclosing numeric domain; style and root supply the query
container, tree context, evaluation and invalidation. This import neither
selects Values 5 in full nor replaces the selected Values 4 edition.

Sizing 4 separately requires the published Values 5 `calc-size()` definition.
This narrow import covers its intrinsic basis, contextual `size` keyword,
numeric typing and specified math simplification. It applies to all twelve
physical and flow-relative sizing properties through Sizing 4's shared
`<box-size>` production. Preserve nested bases in specified values; the separate
interpolation canonicalization and used-value resolution procedures belong to
animation and style/layout. The import does not select `interpolate-size` or
other Values 5 functions. Sizing 3's newer shared minimum-size definition also
supersedes Logical 1's older initial `0` with `auto`, while Logical 1 still owns
the mapping between physical and flow-relative properties.

Logical 1 §4.7 defines the authored `logical` marker and four-side role order,
even though the marker is unstable. The pinned draft leaves complementary
physical/logical reset membership source-limited; four explicit assignments
alone do not settle that normative question. Surgeist adopts a bounded project
policy for all eight families: `border-color`, `border-style`, `border-width`,
`inset`, `margin`, `padding`, `scroll-margin` and `scroll-padding`. Physical mode
and CSS-wide values expand to four physical sides; retained `logical` mode
expands to four flow-relative sides. Neither mode resets complementary sides.
Strict substitution reentry selects the replacement mode, and normalization
uses that selected footprint. Frozen WebKit corroborates physical membership
only; logical no-reset is project policy, not settled normative behavior or
WebKit support for the marker. See the
[four-side membership reference](../docs/reference.md#four-side-shorthand-membership)
for the exact boundary. Logical-axis pairs and longhands retain their defined
intrinsic expansion.

Color 5 also directly references Values 5 mix items and percentage normalization.
Those definitions are absent from the latest published 2024 edition. A separate
immutable CSSWG source revision from 4 September 2026 pins only these required
definitions; it leaves the published tree-counting import unchanged. CSS retains
ordered authored colors, optional weights and specified-value facts. Resolved
color computation applies the normalization algorithm with Color 5's force flag
and leftover-alpha rule; this source import does not claim that execution or
complete color serialization is implemented. Other Values 5 functions are not
selected by this dependency.

Media Queries 5 custom-media and Conditional Rules 5 named-supports names require
the extension-name production from CSS Extensions 1. The catalog pins an
immutable pre-cutoff source revision for that definition because no published
edition was found. It accepts decoded identifiers starting with two hyphens,
including the bare `--` name; custom-property name validation is a different
grammar. Only this name definition is imported. The two public name types remain
distinct. Definition environments, duplicate selection, cycle handling,
evaluation and live APIs remain with style and root integration.

Selected Conditional Rules 5 and Nesting 1 reference `block-contents`, absent
from the selected published Syntax 3 edition. A separate immutable pre-cutoff
Syntax source pins this production and its scoped block-parsing dependencies.
The production is category-neutral: each consuming rule still defines valid
children and declarations. It does not itself imply style ancestry or admit
page rules. `@container` and `@supports-condition` are recorded consumers. The
named-supports definition retains an ordered tree of generic test candidates;
its declarations and child rules inspect authored feature syntax and do not
render or become style declarations.

The pinned block consumer omits transferring its final accepted declaration run
before returning at a closing brace or EOF. The definition's
`localized_reconciliation` records this exact source conflict and the selected
Nesting retention requirements that control style and style-nested group bodies:
retain the nonempty final run exactly once, preserving authored order and
parent-specific materialization. Conditional Rules 5 supplies a separate,
independent declaration-only example, so its named-supports test body also
retains a nonempty final candidate run once. Its declaration validity permits
unknown property and feature grammar, and malformed items recover locally while
the enclosing definition remains structurally valid. The catalog records that
test-body interpretation explicitly; Conditional Rules 5 does not spell out each
malformed-candidate outcome. Other contexts retain their own admission
requirements. These reconciliations neither make general CSS syntax
undefined nor replace the selected 2021 tokenizer and unrelated parser entry
points. The catalog keeps the original hash and identifies the affected source
lines; it does not silently substitute a corrected draft.

The top-level `source_reconciliations` also records bounded Color 4 and
SVG glyph authored compatibility decisions. `I-WEBKIT-SVG-GLYPH` identifies
the independent frozen WebKit73aa6c89 Auto/Angle definition and its exact
raw numeric, mode and presentation-attribute consumers. Ordinary lookup retains
the selected finite Writing Modes shorthand; explicit SVG selection supplies
one independent SVG terminal, Auto initial, inheritance and no animation.
Standards admits finite angles and raw Number zero; Quirks and the fixed
attribute-value front also admit finite nonzero Numbers. Angle math remains
authored, while attribute binding and computed quadrant rounding remain
downstream. The recorded property and consumer hashes cover immutable Git blobs,
with the computed-style/attribute test witnesses labelled as downstream evidence.
The two CSSWG8032 conflicts remain unresolved. This operational record imports
no normative SVG11 glyph document or full SVG module and adds no canonical
schema row or Snapshot module count.

The same reconciliation array records bounded Color 4 and
Color 5 serialization decisions: grammar-valid relative alpha with explicit
override retention, case-sensitive custom profile identifiers, nonnegative
omitted mix weights, phase-specific numeric rounding, and deferred HSL or HWB
conversion when contextual channel math remains unresolved, and ordinary sRGB
calculated-alpha finalization. Color 4 §15.1's historical scalar simplification
conflicts with §16.1.2's retained unclamped specified calculation. The recorded
WebKit interpretation finalizes only noncontextual ordinary RGB/HSL/HWB alpha,
including eligible siblings and Mix children; actual Origins, relative overrides
and other retained families keep their separate contracts. Exact immutable
WebKit source evidence accompanies that operational disposition. The exact radian
factor and generated mix-share precision are implementation selections folded
into those decisions; ordinary alpha omission follows the selected normative
order; the calculated sRGB phase conflict is separately identified.

Two other color range-phase reconciliations retain specified calculations.
Color 5 §11.3 calls declared relative alpha specified but clamped; Color 4
§§15.2–15.3 describe declared, computed and used Lab/OK values after component
clamping. Values 4 §§10.12–10.13 instead protect specified math from clamping
and reserve scalar-wrapper removal for computed or later serialization.
Frozen WebKit's unresolved relative objects and non-eager Lab/OK parsing retain
stored calculations; its calculation serializer preserves the specified phase.
The catalog records the exact parser, stored-component serializer and calc-stage
blobs supporting that selection. Existing percentage scaling, coefficient text,
direct bounds, explicit relative overrides and genuine Origins keep their separate
contracts. This does not claim identical WebKit text, custom-profile support,
browser results, normative harmonization or complete color grammar/arithmetic.

Three separate reconciliations govern pure HSL/HWB coordinate conversion.
The generic powerless-component prose says hue is missing below epsilon,
whereas the HSL table and inverse algorithm use an inclusive threshold.
The generic prose also requests zeroing small positive saturation and adjusting
near-gray HWB blackness; the inverse samples retain those coordinates.
Frozen WebKit uses the inclusive HSL threshold, marks nearly neutral hue
missing, and retains saturation and W/B. Its pinned conversion and color-model
blobs support the selected numerical API behavior. These decisions preserve
manually supplied hue and leave authored parsing and specified serialization
under their separate contracts; they do not adopt WebKit's float narrowing or
claim the draft requirements agree.

Lab-to-LCH and Oklab-to-Oklch conversion likewise uses the specific tables'
inclusive chroma thresholds, recorded as two individual reconciliations with
the generic less-than wording. The normative conversion-generated cleanup
zeros positive chroma when hue becomes missing; frozen WebKit applies it to
both families. The conversion formula does not exclude that cleanup, and the
nonnormative JavaScript samples do not override it. Manually constructed
coordinates retain their supplied chroma and hue independently of conversion.

Concrete rectangular conversion uses the normative RGB primaries and D50/D65
whitepoints, linear Bradford adaptation, and the consistent pinned Lab/Oklab
coefficients. Rec.2020 uses the dated publication's gamma 2.4; frozen WebKit's
older piecewise transfer curve is browser divergence, rather than a competing
normative definition. ProPhoto has a separate expression defect: its normative
nonlinear branch prepares magnitude and sign but raises the signed negative
input to a fractional power. The catalog selects reflected signed-absolute
powers to fulfill the same publication's extended real-domain requirement.
Its nonnormative sample supports that extension; frozen WebKit repeats the
defect and cannot supply a valid negative-channel result. The
[reference](../docs/reference.md#pure-rectangular-color-space-conversion)
records numerical units, missingness, range and failure boundaries.

The coefficient basis uses the consistent
[Color 4 numerical samples](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#color-conversion-code)
and frozen WebKit commit `73aa6c89e2cb77c46184a81aec944e4ab99d114d`.
Under `Source/WebCore/platform/graphics/`, `ColorConversion.h` blob
`6bd03353c1a6540a896b342571913fdfbe9b7193` identifies the Bradford basis and
adaptation matrices; `ColorConversion.cpp` blob
`e1d28a9c4c844459e432c39b71e1fde911023777` identifies the Lab/Oklab formulas
and coefficients. RGB primary matrices derive independently from the normative
chromaticities. The numerical helpers retain extended coordinates instead of
importing the browser's Lab/Oklab lightness clamps.

Text 4's `text-align` and `text-align-all` tables show a standalone `<string>`
alternative, but its prose and example expressly combine a string with a keyword.
The catalog's localized reconciliation retains the five positional keywords from
the explanatory 13 November 2012 Text 3 grammar and accepts the string and one
positional keyword in either order. `justify`, `match-parent`, and `justify-all`
remain separate alternatives, and the obsolete `start end` form is not restored.
The string must decode to exactly one default extended grapheme cluster; the
catalog imports only UAX29 revision 47's Unicode 17.0.0 boundary definition for
this authored check. Its historical Text 3 source and UAX29 body have exact pins
without adding either document as a fully selected CSS module. Contextual
typographic tailoring and character placement remain with text and layout.

The top-level `unresolved_requirements` names two separate Writing Modes legacy
`glyph-orientation-vertical` questions: alternate numeric-terminal spelling,
including the unitless integer/number distinction, and math applicability with
finite keyword mapping. Each record cites the selected Writing Modes and Values
sections and identifies CSSWG issue 8032 as explanatory discussion, not a
normative resolution. The five explicit terminals and their mapping remain
implemented; these records do not select another module or expand parser
admission. The public alias feature is `Partial` because its remainder is scope
not claimed as supported, rather than a claim that all other forms are valid.

## Applying the catalog

Read `selection_policy`, `authored_scope_definitions`, and `owners` before using
a module entry. CSS retains symbolic authored information and validates what can
be decided without an element, environment, resource, or layout. The listed
downstream owners handle requirements needing those contexts. Intrinsic grammar
validity is distinct from contextual usefulness, such as a font-face rule with
omitted descriptors.

Apply the explicit supersession policy to overlapping definitions. In particular,
Conditional Rules 5 supplies the container-query definitions moved out of
Containment 3; published Grid 3 supplies `flow-tolerance`; and Media Queries 5
supplies `display-mode`. Preserve source membership without duplicating effective
requirements or retaining superseded authored spellings.

The scoped-nesting record identifies the selected Nesting definition that
specializes Cascade 6's older scope-body nesting-selector semantics. Preserve
the distinction between an ancestor style selector and the active scope chain.
Ordinary unstyled scopes retain their rule-list grammar; style-nested scopes
admit declaration runs under Nesting's group-rule definition. Matching and
specificity execution remain downstream.

For a required definition from another specification, follow
`selection_policy.normative_dependencies` and record its exact source and section
with the requirement evidence before implementation relies on it. Referencing a
definition does not select its entire module. An undefined draft requirement
needs the exact omission cited under `selection_policy.draft_omissions`; an
ordinary missing implementation cannot receive that disposition.

When a source cannot be retrieved, its hash differs, or two selected definitions
cannot be reconciled through their explicit replacement rules, stop relying on
that evidence and return the source discrepancy to the coordinator. Do not
silently refresh the pin, invent grammar, or mark the affected requirement
complete. Changing a pin is a reviewed product-contract change accompanied by
the affected grammar, expectation, and provenance changes.

Deterministic catalog checks require unique module and definition IDs, the
declared tier and selection counts, valid owner and definition-source references,
exact immutable or dated source identities, valid SHA-256/byte-count pairs,
resolved supersession and required-section references, source line bounds,
UTF-8 with LF endings, no trailing whitespace, and one final newline. Hash checks
compare the exact retrieved representations or pinned Git blobs; a reserialized
HTML document is not the same input.
