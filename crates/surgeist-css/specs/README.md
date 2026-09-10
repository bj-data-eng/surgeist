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
