# Reference

## Package And Workspace

The root [Cargo.toml](../Cargo.toml) declares `surgeist` 0.1.0, Rust edition 2024,
MSRV 1.97, and the MIT license. Root is the sole member of its Cargo workspace.
All 14 leaf paths are excluded from workspace membership; 13 are exact
`=0.1.0` production path dependencies. `surgeist-test` is a pinned API-audit
input with no root production dependency or facade reexport.

## Facade Modules

The [public entry point](../src/lib.rs) reexports each production leaf's public
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

Shared verification contracts live in the independent
[surgeist-test crate](../crates/surgeist-test/src/lib.rs).

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
| Independent leaf paths and repository URLs | [.gitmodules](../.gitmodules) |
| Selected leaf revisions | Committed Git submodule pointers; inspect with `git submodule status --recursive`. |
| API-generator CLI and target discovery | [api/generator/src/lib.rs](../api/generator/src/lib.rs) |
| Facade API audit | [api/public-api.txt](../api/public-api.txt) |
| Leaf API audits | [api/crates/](../api/crates/) |
| Source-checkout dependency attribution | [NOTICE.md](../NOTICE.md) and [licenses/](../licenses/) |

The generator supports `--all`, `--root`, or `--crate` target selection and
`--list` or `--check` actions. Without flags it generates all artifacts. Targets
are root plus `crates/surgeist-*` directories containing a manifest, independent
of Cargo workspace membership. Source is authoritative; the generated root
audit retains wildcard reexport placeholders, and audit headers may report
missing rustdoc item IDs. Consult the pinned public source for detailed APIs.

## Verification Scope

The root guide lists workspace check/test/Clippy, formatting, and API-audit
commands. Root has one library identity test. The API generator has its own unit
tests in its separate workspace. Leaf guides own their focused feature,
platform, and test commands. No root CI configuration, integration-test suite,
examples, `dev/` harness, or `tools/` implementation is present in this baseline.
