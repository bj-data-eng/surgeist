# Explanation

## One Repository, Distinct Crate Boundaries

Surgeist keeps reusable UI contracts in separate crates while developing their
implementations in one Git repository and product Cargo workspace. Root owns
the facade, domain implementations, shared tooling, and integration. A change
to several participating crates can therefore share a source revision, review,
and dependency resolution without publishing intermediate crate candidates.

Domain ownership still matters. CSSOM owns its integration adapter with CSS.
Root owns the other Surgeist-to-Surgeist adapters:
conversions from one crate's public model into another crate's public inputs.
Domain crates own their algorithms, models, and backend-local adapters, such as
text shaping, rendering backends, and platform hosting. `surgeist-test` owns
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
Style will be rebuilt using CSSOM snapshots for matching, cascade, inheritance,
substitution and computed values.

## Facade And Integration

The facade directly reexports the production crates through their public front
doors. Its `app` and `runtime` modules both expose `surgeist-runtime`. Root-owned
cross-crate adapters, integration tests, examples, and a native development
harness are unimplemented. The shared source boundary supports integration work;
it does not supply an integrated UI application by itself.

Source, tests, fixtures, and legal material stay with their owning crates.
[NOTICE.md](../NOTICE.md) records attribution and source provenance. Issues and
the GitHub Project record current work under the
[repository administration guidance](../.agents/skills/surgeist-admin/SKILL.md).

## Workspace Membership And Verification Are Separate

All 16 product packages share one committed root lockfile. The facade is the
sole default member, keeping unqualified root commands focused. Explicit package
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
