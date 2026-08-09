# Root Baseline Reset Specification

## S1. Outcome

Reset the root `surgeist` repository to a buildable product-integration baseline
over exactly the 2026-08-09 remote-`main` candidate SHAs recorded in S4 for every
configured `surgeist-*` submodule; later remote advances are not part of this
cycle. The root keeps package composition, compatible feature forwarding,
gitlinks, the API generator, and root-package verification. Legacy root adapters,
integration tests, requirements, examples, the native development harness, and
the fixture-metadata tool remain removed.

The root remains Surgeist's permanent product integration point and owner of
cross-crate composition, adapters, integration tests and tools, compatibility
pins, and whole-product verification. The resulting repository is the clean
baseline from which those real integrations will be designed and implemented in
root against current leaf contracts. It must compile without pretending that the
removed adapter behavior is still available or transferring integration
ownership into leaf crates.

Here, “baseline” or “base/new crate” means a clean new-crate-style root state at
the S4 candidate graph. It does not mean preserving or separately compiling the
intake gitlink graph, which the user explicitly rejected as too far behind to
serve as an integration base.

## S2. Ownership And Scope

The root repository owns this initiative. Leaf repositories are read-only source
candidates: no leaf source, manifest, history, or branch is modified. Root may
change only:

- committed gitlinks for the 14 submodules declared by `.gitmodules`;
- root `Cargo.toml` workspace and target wiring;
- root `src/lib.rs` facade declarations;
- root `README.md` statements that describe the removed integration surface;
- the exact already-authorized deletion inventory in S4;
- root-owned source-derived API artifacts; and
- the canonical planning artifacts required for this reset.

The intake base is
`359322aae90afbaf68ba7c9afffd79fb57b383d6`, equal to root `origin/main` at
initiative start.

## S3. Explicit Non-Goals

- Do not restore, replace, or redesign the deleted cross-crate adapters.
- Do not restore or create examples, the `dev` harness, fixture tooling,
  requirements, legacy integration tests, or the removed app architecture plan.
- Do not add dependencies, features, crates, scripts, generators, CI, or policy.
- Do not replace submodules with Cargo-registry dependencies in this reset; that
  is a separate integration change after the leaf packages are published.
- Do not edit leaf repositories or infer new leaf behavior from their internals.
- Do not restore, validate, or retain compatibility with the obsolete intake
  gitlink graph.
- Do not promise source or API compatibility for the removed adapter module.
- Do not use `unsafe` in Surgeist-owned code.

## S4. Current Evidence

The user-authorized working tree contains 38 tracked deletions. Three surviving
root facts currently prevent even metadata loading:

- `Cargo.toml` still names deleted examples and the deleted `dev` and
  `tools/surgeist-fixture-metadata` workspace members.
- `src/lib.rs` still declares the deleted `adapters` module.
- `api/public-api.txt` still records the deleted adapter surface.

An offline `cargo metadata --no-deps` invocation fails while loading the missing
`dev/Cargo.toml`. The root and every leaf were otherwise clean at intake.

After selecting the S4 candidates, default root composition checks successfully,
including both accessibility features together. Two surviving integration claims
are not compatible with the selected leaves:

- the root `text-render` feature activates text code that calls the pre-candidate
  render API (`TextRun::try_new` without `TextRunBounds` and
  `FontData::from_bytes` rather than `try_from_bytes`); and
- treating all leaf repositories as root Cargo workspace members makes the
  configured root Clippy gate lint leaf-owned code and fail on the selected CSS
  candidate's `clippy::question_mark` warning.

Root does not repair either leaf. The baseline removes the broken root feature
forward and makes the root package the only Cargo workspace member while keeping
all production leaves as exact path dependencies and all 14 submodules as pinned
source/API inputs.

The exact authorized deletion inventory is:

```text
dev/Cargo.toml
dev/README.md
dev/src/lib.rs
dev/src/main.rs
examples/app-thumbnail-import.rs
examples/hello-window.rs
examples/render-window.rs
plans/specs/2026-06-20-surgeist-app-dsl-architecture.md
requirements/dialog.md
requirements/render.md
requirements/render_dsl.md
requirements/retained.md
requirements/retained_update.md
requirements/shape.md
requirements/style.md
requirements/text.md
requirements/text_source_composer.md
requirements/window.md
requirements/window_dsl.md
src/adapters/css_style.rs
src/adapters/css_style_tests.rs
src/adapters/error.rs
src/adapters/mod.rs
src/adapters/retained_style.rs
src/adapters/retained_style_tests.rs
src/adapters/style_layout.rs
src/adapters/style_layout_tests.rs
src/adapters/style_text.rs
src/adapters/style_text_tests.rs
src/adapters/tests.rs
tests/app.rs
tests/css.rs
tests/layout.rs
tests/style.rs
tools/surgeist-fixture-metadata/Cargo.toml
tools/surgeist-fixture-metadata/src/lib.rs
tools/surgeist-fixture-metadata/src/main.rs
tools/surgeist-fixture-metadata/tests/fixture_metadata.rs
```

Each authoritative leaf remote `main` was fetched from the URL committed in
`.gitmodules` into that submodule's
`refs/surgeist/baseline-reset-final-019fe7de/<crate>-main` provenance ref and
matched against a fresh `git ls-remote` result. Every candidate is a descendant
of the currently pinned revision:

| Crate | Current pin | Remote-main candidate |
| --- | --- | --- |
| `surgeist-animation` | `749f86747c62d5a6cebb9b52275d12271f0b2338` | `0aab218bfdc897d14c56abeb30b47bbd6681c06f` |
| `surgeist-css` | `040bc1b4f7cca5e4978f732b3b778c00d7cdef40` | `4b288d6467d91f2fc33eac78ef0b0b725154195d` |
| `surgeist-dialog` | `f6275456f211d61086eb32f8fbbc0e035862d3dd` | `85ca345c21ac1af6d750a7a76c8d08272eac7811` |
| `surgeist-layout` | `c0c6852610b835b60e46c680fbd1a4fb127d1d13` | `dc71a5582ab0ef3925826dce09b93ee9fa6f49a1` |
| `surgeist-render` | `fe58f35aebaf43177fd761b8222a67b3e8f11827` | `e622d1dccb6672e2dd49ecf2fd66e9e46a66b782` |
| `surgeist-retained` | `98c24b175431de76c52b865d318959cdb6a36e89` | `268bafd3bc63820121995bc5febfdafc2baa9723` |
| `surgeist-runtime` | `92bbf59b79cceb5b288bb3ecabfe9439b356da81` | `e93ae9fddf5a3a3e34c3f4fc51c5f752613ca8a9` |
| `surgeist-shape` | `6c81624d013f6fe62aeba2c4e2d6f9ac7208b600` | `2fc4b51a84bb45fe256b6884770b53d1adc22925` |
| `surgeist-style` | `fcc42de2c32a318e073233dd51508dd4cc28041a` | `d3ff58ab7384c7ff2bab34b5f82ce509f9bf4910` |
| `surgeist-task` | `9be361558f082dc2e3504863d5ed046daa490086` | `9356f27ee1bc0833fd0ce22e48fdbe75bb94ae1c` |
| `surgeist-template` | `95e7a99f5fb324cb900a37fe8b9a442c9eb2eb45` | `2f48fe0af5771f0bfe361b7dd6098de763be55d7` |
| `surgeist-test` | `7fb45149f0fbc4f6a8bd4b020c2a24d5f64adbda` | `e3ca756612679e4ee937e6cd7a3e76bd05228536` |
| `surgeist-text` | `754707f27feb04fb7ff31e0574ff43ded552d360` | `9109087d7f82d28b10c85e312b32a7a006cb0605` |
| `surgeist-window` | `2d595222433e700a673673de2e9f1c9151e63bc2` | `92c4b250dd033ae83036ca877407f3cc7b2b1d68` |

## S5. Root Facade Contract

`src/lib.rs` remains a direct module-per-leaf facade. It reexports each
production leaf from its intentional crate front door and retains both `app` and
`runtime` aliases for `surgeist-runtime`. It retains the root `crate_name()`
smoke contract.

The deleted `surgeist::adapters` module is removed from the facade. This is an
intentional breaking public API change. No compatibility shim or placeholder
module is permitted because that would preserve a false integration contract.

`surgeist-test` remains a pinned submodule and API-audit input but is not a Cargo
workspace member, production dependency, or facade reexport.

## S6. Manifest Contract

The root package keeps one exact path dependency on every production leaf. Its
post-reset feature set is exactly:

| Root feature | Forwarded feature |
| --- | --- |
| `default` | none |
| `dialog-system` | `surgeist-dialog/system` |
| `render-web` | `surgeist-render/render-web` |
| `render-window` | `surgeist-render/render-window` |
| `text-accessibility` | `surgeist-text/text-accessibility` |
| `window-accessibility` | `surgeist-window/accessibility` |

Removal of `surgeist::adapters` and removal of the incompatible root
`text-render` feature are intentional public facade breaks in this reset. No
replacement feature or compatibility shim is added. Verification covers the
default feature set and each remaining forwarded feature individually. It also checks `text-accessibility` and
`window-accessibility` together to prove their shared exact AccessKit dependency
resolves at 0.24.1. The root workspace gate covers the root package, and no
unsupported all-features promise is introduced.

The Cargo workspace contains only the root package. All 14 `crates/surgeist-*`
submodules are explicitly excluded from workspace membership while the 13
production leaves remain root path dependencies. This keeps leaf manifests and
focused gates independently owned and prevents root Clippy from treating leaf
warnings as root implementation failures. Deleted local crates and explicit
example targets are absent. Workspace and root development dependencies that
served only deleted targets are absent. No new dependency or feature is
introduced.

The root package and workspace keep Rust 2024, MSRV 1.97, version 0.1.0, and the
existing repository/license metadata.

## S7. Candidate And Compatibility Rules

Each selected gitlink equals the exact candidate SHA in S4, not a mutable branch
or later descendant. These are the authoritative remote `main` tips observed and
verified on 2026-08-09. A later remote advance during this root cycle does not
stale the selected set and is not chased; publication readback proves only that
each exact selected commit remains available from its authoritative repository.

For this reset, the user explicitly selected each authoritative remote `main`
tip and explicitly waived leaf-to-root `CRATE_CANDIDATE` handoffs because root
was intentionally not ready to receive those handoffs while the leaf work was
performed. This is a scoped workflow exception, not a standing change to the
normal integration contract. Candidate evidence for this reset is the exact
fetched object, authoritative URL, fresh remote-tip equality, descendant
relationship from the old pin, committed leaf policy and manifest inspection,
and successful root compatibility gates. No handoff is invented after the fact.

Compatibility is established by compiling the root package with all production
path dependencies under its default and retained feature contracts. Leaf-private
paths are never used. The latest candidate manifests and their focused gates
remain independently owned; root does not claim that the removed `text-render`
composition works or lint leaf source as root-owned implementation.

## S8. Generated Artifact Contract

Source is authoritative. After the facade and exact gitlinks are present, the
existing root API generator refreshes the complete facade and leaf audit set.
Generated files are never hand-edited. The expected root artifact removes the
adapter API; leaf artifacts may change only to reflect the selected committed
leaf sources.

## S9. Documentation Contract

The root README describes the repository as Surgeist's product integration facade
and coordination workspace. It preserves root ownership of future cross-crate
adapters, integration tests and tools while clearly stating that the reset
baseline does not yet provide those removed implementations. It must not claim
that the deleted fixture-metadata tool currently exists. It continues to document
cloning with submodules, the workspace check, and root-owned API artifact
generation. It states that the root package is the only Cargo workspace member
and that the pinned leaves remain path dependencies and API-audit inputs. Its
framework description and submodule inventory include `surgeist-runtime`,
matching both `app` and `runtime` facade modules. No replacement integration
design, example, or roadmap is added.

## S10. Verification Contract

The reset is accepted only when:

- Cargo can load the root-only workspace offline without missing local targets;
- the root-only workspace checks and tests successfully while compiling all
  default production path dependencies;
- the root default, individual forwarded-feature, and combined text/window
  accessibility checks succeed offline;
- Clippy covers all workspace targets with warnings denied and unsafe code
  forbidden;
- formatting is current;
- the configured full API generator check reports no delta;
- root and every selected leaf contain no executable Surgeist-owned `unsafe`;
- the root diff contains exactly the deletion paths listed in S4 plus the
  canonical planning artifacts, `Cargo.toml`, `src/lib.rs`, `README.md`, the 14
  exact gitlinks, and generator-produced API artifact deltas; and
- the reviewed root candidate is published to root remote `main` and read back.

No restored legacy integration behavior is an acceptance criterion.

## S11. Initiative Acceptance

The initiative is complete when root `main` is a clean, published, remotely
verified product-integration baseline at the exact candidate set in S4; all
configured root gates pass; generated API artifacts match source; all review
gates are clean; and all cycle-owned provenance refs are removed after final
selected-commit availability proof. Cleanup, exact gitlink promotion, facade
repair, generated artifact refresh, and verification land as one root integration
cycle.
