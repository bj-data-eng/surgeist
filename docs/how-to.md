# How-To

These procedures assume the [first-success check](getting-started.md) passes.
Run commands from the repository root with the required tooling and dependencies
already installed. [AGENTS.md](../AGENTS.md#command-inventory) owns command
selection and serial execution policy. Select each applicable package, feature,
target, and suite deliberately, then run commands one at a time.

## Check A Forwarded Feature

Choose a root feature from the [feature reference](reference.md#root-features).
For example, to compile native dialog support:

```sh
cargo check --offline --locked -j 1 -p surgeist --no-default-features --features dialog-system
```

A successful exit verifies that feature's compilation on the current target.
It does not exercise a native dialog. For the shared text/window accessibility
composition, use:

```sh
cargo check --offline --locked -j 1 -p surgeist --no-default-features --features text-accessibility,window-accessibility
```

These examples establish only their selected configuration. Read the affected
crate manifests and supplements for other feature and platform checks. Do not
infer support for every combination from workspace membership.

## Inspect And Test A Crate

Find the package in [Cargo.toml](../Cargo.toml), then inspect its manifest,
`AGENTS.md`, `src/lib.rs`, and README. Crate guides supplement root policy with
domain contracts and focused checks. For example, run the task library tests:

```sh
cargo test --offline --locked -j 1 -p surgeist-task --lib -- --test-threads=1
```

A successful exit reports that selected library suite's results. Choose
integration tests, doctests, platform tests, or feature checks separately when
the change requires them. Source changes across crates are made and reviewed
in this repository with the root product lockfile.

## Check Public API Audits

The [API generator](../api/generator/Cargo.toml) is a separate Cargo workspace.
It needs its own cached dependencies and the nightly toolchain pinned by
`generate_target_artifact` in [generator source](../api/generator/src/lib.rs).
The environment settings below keep nested Cargo operations offline and limit
them to one build job.

For a focused check of the task crate's audit:

```sh
CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true cargo run --offline -j 1 --manifest-path api/generator/Cargo.toml -- --check --crate surgeist-task
```

Expect a `current` line for the selected artifact and a successful exit. A
`stale API artifacts` error identifies missing or different audit text. Check
mode builds rustdoc data but does not rewrite audit files. Use `--list` to
inspect targets; use `--check --all` only when the full audit set is in scope.
API auditing is separate from normal `cargo test` runs.

Use `--check --crate surgeist-generator` to check both its default API and its
CSS corpus API. The companion `surgeist-generator.css-corpus.txt` artifact uses
`--no-default-features --features css-corpus`. Its listing, header, and stale
diagnostic identify it as `surgeist-generator [css-corpus]`; the browser feature
is disabled for this profile.

## Refresh Audits After An Authorized Source Change

Refresh from the changed source in this repository. External crate publication
is no longer an intermediate step. Never repair an audit by editing its text
manually. To refresh the task crate's audit:

```sh
CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true cargo run --offline -j 1 --manifest-path api/generator/Cargo.toml -- --crate surgeist-task
```

Use `--root` for the facade or `--all` for an authorized complete refresh.
`--crate surgeist-generator` refreshes both its default and CSS profile audits.
Inspect the generated diff, explain each delta from its source change, and run
the corresponding check. Keep the source and generated changes together in
review. Missing prerequisites or an unexplained delta stop the refresh handoff.

## Locate Corpus And Audit Tooling

[surgeist-generator](../crates/surgeist-generator/README.md) supplies shared CSS
and browser-corpus infrastructure; it is a product workspace member, with
feature-gated tools. [Layout's guide](../crates/surgeist-layout/AGENTS.md) locates
its domain adapter and parity checks. Follow the owning tool's documented
verification mode; generation and browser/source acquisition are separate
operations that need their own task scope.

The [layout Dylint catalog](../crates/surgeist-layout/tools/surgeist-layout-audits/README.md)
is an optional separate workspace with nightly compiler tooling. Its historical
audit questions are selected explicitly, not included in ordinary product
verification.
