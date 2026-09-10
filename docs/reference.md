# Reference

## Package And Workspace

The root [Cargo.toml](../Cargo.toml) declares `surgeist` 0.1.0, Rust edition 2024,
MSRV 1.97, and the MIT license. The product workspace contains root plus 15
crates, with `default-members = ["."]` and one committed root
[Cargo.lock](../Cargo.lock). Thirteen crates are exact `=0.1.0` production path
dependencies of the facade. `surgeist-test` supplies verification support and
`surgeist-generator` 0.2.0 supplies shared corpus tooling; both are workspace
members and API-audit inputs with no facade dependency or reexport.

The [API generator](../api/generator/Cargo.toml) and optional
[layout Dylint catalog](../crates/surgeist-layout/tools/surgeist-layout-audits/Cargo.toml)
are separate Cargo workspaces, outside the 16 product members. Crate manifests
own their respective version, MSRV, dependencies, and features.

## Facade Modules

The [public entry point](../src/lib.rs) reexports each production crate's public
front door. It also exposes `crate_name() -> &'static str`, returning
`"surgeist"`.

| Root module | Source crate |
| --- | --- |
| `animation` | [surgeist-animation](../crates/surgeist-animation/src/lib.rs) |
| `app`, `runtime` | [surgeist-runtime](../crates/surgeist-runtime/src/lib.rs) |
| `css` | [surgeist-css](../crates/surgeist-css/src/lib.rs) |
| `dialog` | [surgeist-dialog](../crates/surgeist-dialog/src/lib.rs) |
| `layout` | [surgeist-layout](../crates/surgeist-layout/src/lib.rs) |
| `render` | [surgeist-render](../crates/surgeist-render/src/lib.rs) |
| `retained` | [surgeist-retained](../crates/surgeist-retained/src/lib.rs) |
| `shape` | [surgeist-shape](../crates/surgeist-shape/src/lib.rs) |
| `style` | [surgeist-style](../crates/surgeist-style/src/lib.rs) |
| `task` | [surgeist-task](../crates/surgeist-task/src/lib.rs) |
| `template` | [surgeist-template](../crates/surgeist-template/src/lib.rs) |
| `text` | [surgeist-text](../crates/surgeist-text/src/lib.rs) |
| `window` | [surgeist-window](../crates/surgeist-window/src/lib.rs) |

Shared verification contracts live in
[surgeist-test](../crates/surgeist-test/src/lib.rs). Shared CSS and browser-corpus
generation contracts live in
[surgeist-generator](../crates/surgeist-generator/src/lib.rs), whose optional
`css-corpus` feature exposes its CSS driver and whose `browser-corpus` feature
enables browser infrastructure. Layout keeps its corpus adapter and semantic
conversion in its own package.

## Root Features

The root manifest defines these forwards. Default features enable none of them.
Dependencies still retain their own manifest-defined defaults.

| Feature | Forward |
| --- | --- |
| `dialog-system` | `surgeist-dialog/system` |
| `render-web` | `surgeist-render/render-web` |
| `render-window` | `surgeist-render/render-window` |
| `text-accessibility` | `surgeist-text/text-accessibility` |
| `window-accessibility` | `surgeist-window/accessibility` |

## Source And Artifact Locations

| Fact or artifact | Location |
| --- | --- |
| Agent ownership and command discovery | [AGENTS.md](../AGENTS.md) |
| Root package, dependencies, features, and workspace | [Cargo.toml](../Cargo.toml) |
| Product dependency resolution | [Cargo.lock](../Cargo.lock) |
| Crate source and domain guides | [crates/](../crates/); each package's manifest, `AGENTS.md`, README, and `src/lib.rs` |
| Snapshot origins and retained root history | [Explanation](explanation.md#snapshot-import-basis) |
| Shared corpus tooling | [surgeist-generator](../crates/surgeist-generator/README.md) |
| API-generator CLI and target discovery | [api/generator/src/lib.rs](../api/generator/src/lib.rs) |
| Facade API audit | [api/public-api.txt](../api/public-api.txt) |
| Crate API audits | [api/crates/](../api/crates/) |
| Source-checkout attribution and crate notice index | [NOTICE.md](../NOTICE.md) and [licenses/](../licenses/) |

The generator supports `--all`, `--root`, or `--crate` target selection and
`--list` or `--check` actions. Without flags it generates all artifacts. Targets
are root plus `crates/surgeist-*` directories containing a manifest, independent
of Cargo workspace membership. Source is authoritative; the generated root
audit retains wildcard reexport placeholders, and audit headers may report
missing rustdoc item IDs. Consult current public source for detailed APIs.

Every package keeps its existing artifact using its manifest-defined default
features. The configured `surgeist-generator` CSS profile adds
`api/crates/surgeist-generator.css-corpus.txt`, built with
`--no-default-features --features css-corpus` and without `browser-corpus`.
`--crate surgeist-generator` and `--all` select both generator artifacts; other
packages keep their default profile. The CSS profile appears as
`surgeist-generator [css-corpus]` in listings, headers, and stale diagnostics.

## Verification Scope

The [root command inventory](../AGENTS.md#command-inventory) selects serial
package check/test/Clippy commands, formatting, and explicit API auditing.
Unqualified root Cargo commands select the facade by default; `--workspace`
would explicitly expand selection to all 16 product packages. Broad workspace
test runs and assumed all-feature combinations are not the verification policy.

Root has one library identity test. The API generator has tests in its separate
workspace. Crate supplements locate their focused feature, platform, and test
commands. Native GPU, platform, browser, and corpus checks require the matching
environment and selection. No root CI configuration, integration-test suite,
examples, `dev/` harness, or root `tools/` implementation is present.
