# Explanation

## One Repository, Distinct Crate Boundaries

Surgeist keeps reusable UI contracts in separate crates while developing their
implementations in one Git repository and product Cargo workspace. Root owns
the facade, domain implementations, shared tooling, and integration. A change
to several participating crates can therefore share a source revision, review,
and dependency resolution without publishing intermediate crate candidates.

Domain ownership still matters. The
[architecture vision](https://chatgpt.com/space/page_6ce383176ca08191b1dd6d5d16e63979)
uses direct subsystem dependencies: each consumer owns integration with its
lower-level inputs, alongside its algorithms, models and backend adapters.
CSSOM owns its CSS integration; the planned DOM owns its HTML integration.
Tests exercise those actual dependencies without requiring the application
facade. Runtime owns scheduling and coherent publication; the facade provides
application builders, reexports and bootstrap. `surgeist-test` owns
shared verification contracts. `surgeist-generator` supplies CSS corpus tooling
and browser-corpus mechanics, while layout keeps its semantic conversion and
adapter. Neither support crate is a facade production dependency.

The [repository guide](../AGENTS.md#product-boundary) defines these boundaries.
Models remain typed and composable, production dependencies remain directional
and acyclic, and symbolic values remain unresolved until their owning layer has
the necessary context. A shared repository does not resolve mismatched models,
feature incompatibilities, or missing integration behavior by itself.

The previous style crate and facade module are removed. CSSOM consumes the public
CSS producers and owns their adapter into live authored state. It provides stable
identities, immutable snapshots, revisions and atomic edits for headless
declaration, rule, stylesheet and MediaList operations. The
[CSSOM crate guide](../crates/surgeist-cssom/README.md) explains the supported
extension operations, explicit host facts and source-qualified formatting limits.
Root host bindings and full product CSSOM conformance remain separate work.
`surgeist-html` supplies exact HTML values, native builders and the immutable
construction owner shared by native authoring and `surgeist-html-parser`.
`surgeist-html-semantics` owns captures, qualified intrinsic queries and the
explicit adapter from those same construction results. Its explicitly pinned HTML standard defines
behavior; adapted html5ever source supplies implementation provenance. The
[HTML parser guide](../crates/surgeist-html-parser/README.md) records the decoded-Unicode
document/fragment entry operations, admission and external capability boundaries.
The parser owns its direct integration with CSS source color operations and Color's
numerical owner, retaining one original parser controller and source binding.
Separate CSS parse banks and numerical digit storage are admitted through that
controller; generated values are installed after serialization and cleanup.
Implementation changes for safe Rust, source mapping and work limits do not
define a separate HTML dialect.

The [dependency-free numerical Color owner](../crates/surgeist-color/README.md)
is implemented and consumed directly by CSS and the HTML parser adapter.
Its [integration contract](reference/color-crate-contracts.md) explains the
source-qualified CSS boundary. The separate contracts for
[HTML values, parsing and qualified semantics](reference/html-crate-contracts.md)
are implemented as three packages with direct acyclic dependencies. Manifests
and source establish those boundaries; no old-package reexports are retained.

[`surgeist-dom`](../crates/surgeist-dom/README.md) begins the sole owner of live
document identity, mutation and snapshots with a bounded native XML tree:
Document, generic XML Element, ordinary DocumentFragment, Text, Comment,
ProcessingInstruction and DocumentType nodes. Ordered namespaced attributes
and same-store adoption use that same mutation owner. It consumes HTML's
exact UTF-16 `DocumentString` directly. Atomic batches, finite caller limits,
canonical native Document metadata and owned snapshots establish the foundation;
HTML and namespace-aware element creation, import, shadow trees and facade
replacement remain later contributions.
DOM will own importing HTML construction values and adapting captured DOM facts
to HTML-owned interpretation. Native authoring
uses the same HTML value contracts without serializing and reparsing strings.
The existing template crate keeps its syntax, symbolic expressions and source
generation; it will consume HTML and DOM contracts for registry and construction
integration.
Selectors will own matching, cascade will own precedence and rollback, and style
will compute values from DOM and CSSOM captures. HTML-owned work is scheduled
first; completing DOM integration requires both HTML and DOM. The
[repository guide](../AGENTS.md#product-boundary) distinguishes these planned
boundaries from the implemented workspace.

## Facade And Integration

The facade directly reexports the production crates through their public front
doors. Its `app` and `runtime` modules both expose `surgeist-runtime`. The complete
layered integration, application examples, and native development harness
remain planned work. The shared source boundary supports integration work;
it does not supply an integrated UI application by itself.

Source, tests, fixtures, and legal material stay with their owning crates.
[NOTICE.md](../NOTICE.md) records attribution and source provenance. Issues and
the GitHub Project record current work under the
[repository administration guidance](../.agents/skills/surgeist-admin/SKILL.md).

## Workspace Membership And Verification Are Separate

All product workspace packages share one committed root lockfile. The facade is
the sole default member, keeping unqualified root commands focused. Explicit package
selection exposes the rest of the workspace without automatically running its
CPU-heavy layout suites, GPU tests, platform hosts, or browser tooling.

The [command inventory](../AGENTS.md#command-inventory) selects checks for each
affected domain and supported feature/platform configuration. Facade success
does not establish correctness for every crate.

The API generator stays in a separate Cargo workspace because its rustdoc
toolchain and dependencies serve source auditing. The optional layout Dylint
catalog also stays separate because it uses compiler-internal nightly tooling
for explicitly selected audit questions. Neither expands ordinary product
workspace verification.

## Why API Audits Stay Central

One API generator owns the facade and crate audit set. It discovers source
directories directly and includes support crates as audit inputs. Generated
text makes source changes inspectable, but it is neither the source of truth
nor a complete behavioral or compatibility test. Source and corresponding
generated changes are reviewed in the same repository change.

Use the [reference](reference.md) to locate current interfaces and the
[how-to guide](how-to.md) for maintenance procedures.
