# Explanation

## Why A Root Facade And Independent Leaves

Surgeist separates reusable UI contracts into independently owned crates. The
root provides a common public entry point and selects the revisions that compose
together. Git submodules preserve each leaf's repository and history; placement
under the root checkout does not transfer implementation ownership.

Root owns Surgeist-to-Surgeist adapters: conversions from one crate's public
model into another crate's public inputs. Leaves own their domain algorithms,
models, and backend-local adapters, such as text shaping, rendering backends,
and platform hosting. Shared fixture schemas and harness contracts belong in
`surgeist-test`, keeping the facade out of leaf test dependencies.

The [repository guide](../AGENTS.md) defines the intended boundaries. Models
remain typed and composable, production dependencies remain directional and
acyclic, and symbolic values remain unresolved until their owning layer has
the necessary context. Current implementation facts come from the pinned
manifests and public source; the ownership guide is not a claim that all
planned integration behavior already exists.

## What The Reset Baseline Provides

The [August 9, 2026 reset specification](../plans/specs/2026-08-09-root-baseline-reset.md)
records the transition to the current small root package. It retained exact
production path dependencies, compatible feature forwards, direct facade
reexports, gitlinks, and root-owned API audits. It removed the former adapters,
integration tests, requirements, examples, native development harness, and
fixture-metadata tool.

That reset intentionally removed the public `surgeist::adapters` module and
the incompatible root `text-render` feature. It supplied no compatibility shim.
The two `app` and `runtime` module names both continue to expose the runtime
crate. Root remains responsible for future composition and whole-product
verification, but direct reexports alone do not provide an integrated UI
application.

## Why Cargo Workspace Membership Is Narrow

Root is the only Cargo workspace member. Production leaves remain path
dependencies, so checking root compiles the selected composition while each
leaf retains its own focused tests and lint boundary. Consequently, a successful
root `cargo test --workspace` does not establish that every leaf test passes.
The reset's checks and feature scope are recorded in its
[completed cycle](../plans/cycles/2026-08-09-root-baseline-reset-C01-latest-leaves.md).
That historical record does not replace verification for a later change.

## Why API Audits Stay In Root

Root owns the generator and the combined facade/leaf audit set because it owns
the selected integration graph. The generator discovers source directories
directly, so excluding leaves from Cargo membership does not remove their API
audits. Generated text makes source changes inspectable, but it is neither the
source of truth nor a complete behavioral or compatibility test.

After reading the concepts, use the [reference](reference.md) to locate current
interfaces and the [how-to guide](how-to.md) for maintenance procedures.
