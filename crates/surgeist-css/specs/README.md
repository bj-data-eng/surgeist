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

Color 5 also directly references Values 5 mix items and percentage normalization.
Those definitions are absent from the latest published 2024 edition. A separate
immutable CSSWG source revision from 4 September 2026 pins only these required
definitions; it leaves the published tree-counting import unchanged. CSS retains
ordered authored colors, optional weights and specified-value facts. Resolved
color computation applies the normalization algorithm with Color 5's force flag
and leftover-alpha rule; this source import does not claim that execution or
complete color serialization is implemented. Other Values 5 functions are not
selected by this dependency.

Media Queries 5 custom-media names require the extension-name production from
CSS Extensions 1. The catalog pins an immutable pre-cutoff source revision for
that definition because no published edition was found. It accepts identifiers
starting with two hyphens, including the bare `--` name; custom-property name
validation is a different grammar. Only this name definition is imported.
Definition environments, cycle handling, evaluation and live APIs remain with
style and root integration.

Selected Conditional Rules 5 and Nesting 1 reference `block-contents`, absent
from the selected published Syntax 3 edition. A separate immutable pre-cutoff
Syntax source pins this production and its scoped block-parsing dependencies.
The production is category-neutral: each consuming rule still defines valid
children and declarations. It does not itself imply style ancestry or admit
page rules. Both `@container` and `@supports-condition` are recorded consumers;
recording a dependency is not an implementation claim.

The pinned block consumer omits transferring its final accepted declaration run
before returning at a closing brace or EOF. The definition's
`localized_reconciliation` records this exact source conflict and the selected
Nesting retention requirements that control style and style-nested group bodies:
retain the nonempty final run exactly once, preserving authored order and
parent-specific materialization. Other contexts retain their own admission
requirements. This explicit reconciliation neither makes general CSS syntax
undefined nor replaces the selected 2021 tokenizer and unrelated parser entry
points. The catalog keeps the original hash and identifies the affected source
lines; it does not silently substitute a corrected draft.

The top-level `source_reconciliations` also records five bounded Color 4 and
Color 5 serialization decisions: grammar-valid relative alpha with explicit
override retention, case-sensitive custom profile identifiers, nonnegative
omitted mix weights, phase-specific numeric rounding, and deferred HSL or HWB
conversion when contextual channel math remains unresolved. The exact radian
factor and generated mix-share precision are implementation selections folded
into those decisions; ordinary alpha omission follows the selected normative
order rather than creating another source conflict.

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
