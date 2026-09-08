# How-To

These procedures assume the [first-success check](getting-started.md) passes.
Run root commands from the repository root with the required tooling and
dependencies already installed. The complete agent command inventory and
ownership rules live in [AGENTS.md](../AGENTS.md).

## Check A Forwarded Feature

Choose a root feature from the [feature reference](reference.md). For example,
to compile native dialog support:

```sh
cargo check --offline -p surgeist --no-default-features --features dialog-system
```

A successful exit verifies that feature's compilation on the current target.
It does not exercise a native dialog. For the shared text/window accessibility
composition, use:

```sh
cargo check --offline -p surgeist --no-default-features --features text-accessibility,window-accessibility
```

The reset baseline's feature checks cover each forwarded feature individually
and this accessibility pair. They do not establish every platform or feature
combination; see the [recorded baseline scope](../plans/specs/2026-08-09-root-baseline-reset.md).

## Inspect And Test A Leaf

Use [.gitmodules](../.gitmodules) to identify its repository, then inspect the
pinned leaf's `Cargo.toml`, `AGENTS.md`, `src/lib.rs`, and README for its contract
and checks. For example, the task crate's focused tests run from its own root:

```sh
cd crates/surgeist-task
cargo test --offline -p surgeist-task
```

A successful exit reports that leaf's test results. Root `--workspace` commands
cover the root package; they do not substitute for leaf verification. Source
changes and candidate publication remain with the owning leaf repository.

## Check Public API Audits

The [API generator](../api/generator/Cargo.toml) is a separate Cargo workspace.
It needs its own cached dependencies and the nightly toolchain pinned by
`generate_target_artifact` in [generator source](../api/generator/src/lib.rs).
Set up those prerequisites
before running it. The environment setting below also keeps the generator's
nested Cargo operations offline.

```sh
CARGO_NET_OFFLINE=true cargo run --offline --manifest-path api/generator/Cargo.toml -- --check
```

Expect a `current` line for each selected artifact and a successful exit. A
`stale API artifacts` error identifies missing or different audit text. Check
mode builds rustdoc data but does not rewrite the audit files. It is an explicit
maintenance command, separate from normal `cargo test` runs.

## Refresh Audits After An Authorized Source Change

For a leaf change, first satisfy the [root promotion rules](../AGENTS.md): the
owning leaf source must be committed, published, and exposed at the selected
root gitlink. Never repair an audit by editing its text manually.

To refresh the task crate's audit only:

```sh
CARGO_NET_OFFLINE=true cargo run --offline --manifest-path api/generator/Cargo.toml -- --crate surgeist-task
```

To refresh the full facade and leaf audit set:

```sh
CARGO_NET_OFFLINE=true cargo run --offline --manifest-path api/generator/Cargo.toml
```

Inspect the generated diff and run the check procedure above. Each delta must
follow from its source change. Root owns the generator and all generated
artifacts; leaf repositories do not carry copies.
