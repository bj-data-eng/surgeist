# Explanation

## One Repository, Distinct Crate Boundaries

Surgeist develops its facade, domain crates and shared tooling in one Git
repository and product Cargo workspace. Cross-crate changes can share a source
revision and dependency resolution while retaining separate semantic owners.

The [architecture vision](https://chatgpt.com/space/page_6ce383176ca08191b1dd6d5d16e63979)
uses direct subsystem dependencies. Each consumer owns its integration with
lower-level inputs and tests those actual contracts without requiring the
facade. CSSOM owns its CSS adapter; DOM owns its HTML adapters; Runtime owns
scheduling and coherent publication. The facade exposes the production crates.
The [repository guide](../AGENTS.md#product-boundary) records these boundaries.

Shared source does not make every model or feature combination interoperable.
Dependencies remain acyclic, each semantic phase has one owner, and authored
symbolic values remain unresolved until that owner has the required context.

### Language Values, Authored State And Live Identity

[CSS](../crates/surgeist-css/README.md) owns authored syntax, recovery and
source/fact-qualified resolution. [CSS parser](../crates/surgeist-css-parser/README.md)
owns CSS Syntax 3 tokenization and parsing, without giving syntax a CSS
meaning; CSS builds its language interpretation on it. Unlike the
[HTML parser](../crates/surgeist-html-parser/README.md), which sits above the HTML
values it constructs, the CSS parser sits below every CSS grammar and
depends on no Surgeist crate. [Color](../crates/surgeist-color/README.md)
owns numerical color operations. [CSSOM](../crates/surgeist-cssom/README.md)
adapts CSS into live authored declarations, rules, stylesheets and MediaLists,
with stable identities, immutable snapshots and atomic edits.

[HTML](../crates/surgeist-html/README.md) owns shared values, language rules,
native builders and immutable construction. Its
[parser](../crates/surgeist-html-parser/README.md) owns decoded-Unicode ingestion
and direct CSS/Color integration; [semantics](../crates/surgeist-html-semantics/README.md)
owns qualified captures and intrinsic queries. Pinned standards define behavior;
upstream implementation revisions identify provenance. These are separate
responsibilities within one HTML language contract.

[DOM](../crates/surgeist-dom/README.md) is the canonical live tree owner.
Atomic batches and Source continuations operate on the same identities, state,
snapshots and change history. DOM consumes shared HTML values and qualified
construction/parser results through its own adapters. Immutable HTML results
are inputs to installation, not a competing live DOM.

Ordinary parentage, shadow/Template ownership and node-document membership are
distinct relations. DOM preserves their actual identities through edits, import
and capture. Semantic and external host captures retain separate qualification
and revision domains; a historical observation does not authorize live execution.
The owning crate documentation describes supported operations and precise
capability limits.

[Template](../crates/surgeist-template/README.md) owns strict syntax, symbolic
expressions, component validation and Rust generation. Its HTML-backed registry
queues concrete authoring through ordinary DOM edits. DOM remains the tree owner;
Runtime supplies scheduling. Matching, cascade and computed style remain planned
downstream owners rather than implicit responsibilities of these producers.

## Facade And Integration

The facade reexports production crates through their public front doors.
Its `app` and `runtime` modules both expose `surgeist-runtime`; `surgeist::dom`
exposes the canonical DOM. The facade is a convenient entry point, not an
integration intermediary required between lower-level crates.

Runtime's `DomSession` coordinates the canonical DOM, Style resources, element
tasks, Source event continuations and native hosted lifecycle. Runtime owns
delivery and input policy; DOM owns canonical relations. Its
[orchestration explanation](../crates/surgeist-runtime/docs/explanation.md#dom-orchestration)
states the actual host/service boundaries, including unfinished work.

Caller-owned keyed materialization keeps logical item keys distinct from live
NodeIds and publishes staged maps after successful canonical edits. The
[keyed example test](../crates/surgeist-runtime/tests/keyed_dom_materialization.rs)
exercises that composition; the
[facade authoring test](../tests/dom_authoring.rs) composes native HTML, Template
edits and retained snapshots. These demonstrate their selected contracts,
not an integrated UI application. Complete application examples and a native
development harness remain planned.

## Workspace Membership And Verification Are Separate

Product packages share one root lockfile. The facade is the sole default member,
so unqualified Cargo commands do not select every crate's tests. Package,
feature and platform checks are chosen for the affected boundary under the
[command inventory](../AGENTS.md#command-inventory).

`surgeist-test` owns shared verification contracts. `surgeist-generator` owns
CSS/browser-corpus mechanics; consumers keep semantic expectations and adapters.
`surgeist-fuzz` supplies bounded campaign/process/evidence tooling while
consumers own their cases and regressions. These tools are outside facade
production dependencies and default verification.

The API generator and optional layout Dylint catalog are separate workspaces.
Their rustdoc/compiler tooling serves explicit audits, not ordinary product
execution. Workspace membership alone establishes neither feature compatibility
nor browser, GPU or platform conformance.

## Why API Audits Stay Central

One root-owned generator discovers crate manifests and renders facade/crate API
audits. Source is authoritative; generated text supports inspection but is not
a behavioral or compatibility proof. Source changes and corresponding audit
changes are reviewed together.

Use [Reference](reference.md) for interfaces and source locations, and
[How-to](how-to.md) for maintenance procedures. [NOTICE.md](../NOTICE.md) owns
attribution and source provenance. Issues and the GitHub Project track current
work under [repository administration](../.agents/skills/surgeist-admin/SKILL.md).
