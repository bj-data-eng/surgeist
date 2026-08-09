# Root Baseline Reset — C01 Latest Leaves

## Header

- Cycle ID: `C01`
- Owning repository: root `surgeist`
- Status: `in_progress`
- Cycle base: `359322aae90afbaf68ba7c9afffd79fb57b383d6`
- Reviewed specification:
  `plans/specs/2026-08-09-root-baseline-reset.md` at
  `14a8ead7cf6d2c70deb93be645c7372ae99aed1c3e626daaac65160624fe60d4`,
  sections S1–S11
- Reviewed sequence:
  `plans/sequences/2026-08-09-root-baseline-reset.md` at
  `7b9fd0404b3889a4ae14b76239222560d69b41e56efd09ec20eb0fef76fda2ad`,
  entry C01
- Bounded outcome: publish one compileable root product-integration baseline
  containing the exact 38 authorized deletions, exact 14 frozen S4 gitlinks,
  repaired root package/facade/docs, and generator-derived API artifacts.

## Boundary

- The root repository owns all changes. Leaf worktrees supply immutable commits;
  no leaf file, branch, or history is changed.
- The exact deletion inventory is specification S4. It is preserved and committed
  as implementation input, with no restoration or replacement.
- Candidate selection is frozen to specification S4. Later leaf advances are not
  chased during this cycle.
- The user explicitly waived leaf `CRATE_CANDIDATE` handoffs for this reset.
- Submodules remain the dependency mechanism. Cargo-registry migration is future
  work after leaf publication.
- No legacy adapter, test, example, dev harness, tool, requirements document, or
  integration design is recreated.
- The canonical root worktree is the landing worktree because it contains the
  authorized deletion input. No temporary worktree or branch is created.
- Cycle-owned resources are the 14 frozen-candidate refs named
  `refs/surgeist/baseline-reset-final-019fe7de/<crate>-main` and the 14 mutable
  readback refs named
  `refs/surgeist/baseline-reset-final-019fe7de/<crate>-readback-main` inside their
  matching submodule Git directories. Candidate refs retain the S4 objects;
  readback refs capture authoritative remote `main` without changing the selected
  pins. Both sets are compare-and-deleted after publication readback.

## Impacts

- Public API: intentional breaking removal of `surgeist::adapters` and the
  incompatible root `text-render` feature; all direct leaf facade modules and
  `crate_name()` remain.
- Dependencies/features: all 13 production path dependencies remain; no
  dependency or feature is added; stale root dev/workspace dependencies and the
  broken feature forward are removed; the root becomes the sole Cargo workspace
  member and all 14 leaf paths are explicitly excluded from membership.
- Generated artifacts: root-owned `api/public-api.txt` and affected
  `api/crates/*.txt` are refreshed only by the configured generator.
- Docs/examples: README is reconciled to the current product-integration
  baseline; deleted examples remain absent.
- MSRV: unchanged at Rust 1.97 with edition 2024.
- Root follow-up: none inside this initiative; new adapters and registry
  migration require separate specifications.
- Unsafe: no Surgeist-owned executable `unsafe` may be introduced or retained.

## Tasks

### T01 — Select The Frozen Leaf Worktrees

- Files/area: the 14 `crates/surgeist-*` submodule worktrees and root gitlinks.
- Outcome: each leaf worktree is detached at its exact specification S4 candidate
  while its provenance ref continues to resolve to the same object.
- RED evidence: `git submodule status --recursive` reports every old S4 pin rather
  than the selected candidate.
- Acceptance: every submodule status SHA and provenance-ref SHA equals S4; every
  leaf worktree is clean; old pin is an ancestor of candidate; a direct fetch
  from each `.gitmodules` URL proves candidate is an ancestor of the current
  authoritative `main`; no root commit is created yet.
- Commands: `git submodule status --recursive`; for every `.gitmodules` path,
  `git -C <path> checkout --detach <S4-candidate>`,
  `git -C <path> status --short`,
  `git -C <path> rev-parse HEAD`,
  `git -C <path> rev-parse refs/surgeist/baseline-reset-final-019fe7de/<crate>-main`,
  `git -C <path> merge-base --is-ancestor <old-pin> <S4-candidate>`, and the
  authoritative reachability block in Completion.
- Dependencies: clean reviewed planning packet and reviewed-status planning
  commit.
- Intended commit: none; T03 commits the root gitlink changes with their generated
  audit artifacts.

### T02 — Restore The Root Package Baseline

- Files/area: root `Cargo.toml`, `src/lib.rs`, `README.md`, and the exact S4
  deletion paths.
- Outcome: Cargo no longer references deleted targets or exposes the incompatible
  `text-render` forward; the root is the only Cargo workspace member while all 13
  production leaf path dependencies remain; the facade no longer declares the
  deleted adapter module; documentation matches the reset while preserving root
  integration ownership.
- RED evidence: metadata fails on missing `dev/Cargo.toml`; `src/lib.rs` names the
  missing adapter module; `text-render` fails against the selected render API;
  and root `--workspace` Clippy lints a leaf-owned CSS warning.
- Acceptance: metadata loads offline; the root-only workspace checks, tests, and
  strict Clippy gate pass; each retained S6 feature checks individually and the
  two accessibility features check together; manifest metadata, 13 production
  path dependencies, six exact S6 feature entries, explicit exclusion of all 14
  leaf paths, facade modules, and `crate_name()` match the specification; the
  exact S4 deletions remain; README explains workspace/path-dependency ownership,
  includes runtime, and makes no stale tool/adapter-implementation claim.
- Commands: `cargo metadata --no-deps --offline --format-version 1`;
  `cargo check --offline -p surgeist`; `cargo test --offline -p surgeist`;
  `cargo check --offline -p surgeist --no-default-features --features dialog-system`;
  the same feature check for `render-web`, `render-window`,
  `text-accessibility`, and `window-accessibility`;
  `cargo check --offline -p surgeist --no-default-features --features text-accessibility,window-accessibility`;
  `cargo clippy --offline -p surgeist --all-targets -- -F unsafe-code -D warnings`;
  `cargo fmt --check`; `git diff --check`.
- Dependencies: T01.
- Intended commit: one worker-owned implementation commit containing only T02.

### T03 — Integrate Pointers And Generated API Audits

- Files/area: 14 root gitlinks and generator-produced files under `api/`.
- Outcome: root records every S4 candidate and its complete source-derived public
  API audit set without hand edits.
- RED evidence: root index still records the old gitlinks and
  `cargo run --offline --manifest-path api/generator/Cargo.toml -- --check`
  reports stale artifacts after T01–T02.
- Acceptance: all root gitlinks equal S4; the configured generator refresh
  succeeds; its check mode is clean; generated deltas are attributable only to
  T01–T02 source; no leaf repository becomes dirty.
- Commands: `cargo run --offline --manifest-path api/generator/Cargo.toml`;
  `cargo run --offline --manifest-path api/generator/Cargo.toml -- --check`;
  `git submodule status --recursive`; `git diff --check`; and
  `git -C <path> status --short` for every submodule.
- Dependencies: task-clean T02.
- Intended commit: one coordinator-owned root integration commit containing the
  14 gitlinks and generator-produced API artifacts.

## Completion

Cycle acceptance requires T02 task review to be clean, all three task outcomes
to be present, the cycle plan to have a separate status-only `complete` commit,
root integration and holistic reviews to be clean, all final commands below to
pass, root `main` to be pushed and read back, every exact S4 candidate to remain
available from its authoritative repository, and each cycle-owned provenance ref
to be compare-and-deleted only after that proof. The deletion-list digest below
is the SHA-256 of the sorted, newline-terminated S4 inventory. The changed-path
allowlist covers every other artifact class authorized by S2 and S10.

Run this authoritative reachability block before root landing and again after
root publication. It intentionally permits later remote descendants and never
changes an S4 pin:

```sh
set -euo pipefail
for sub_path in $(git config -f .gitmodules --get-regexp '^submodule\..*\.path$' | awk '{print $2}' | sort)
do
  crate_name=${sub_path:t}
  authority_url=$(git config -f .gitmodules --get "submodule.${sub_path}.url")
  candidate_ref="refs/surgeist/baseline-reset-final-019fe7de/${crate_name}-main"
  readback_ref="refs/surgeist/baseline-reset-final-019fe7de/${crate_name}-readback-main"
  candidate_sha=$(git -C "$sub_path" rev-parse "$candidate_ref")
  git -C "$sub_path" fetch --force --no-tags --no-recurse-submodules --no-write-fetch-head --no-auto-maintenance --no-write-commit-graph "$authority_url" "refs/heads/main:${readback_ref}"
  git -C "$sub_path" merge-base --is-ancestor "$candidate_sha" "$readback_ref"
done
```

Final commands:

```sh
cargo metadata --no-deps --offline --format-version 1
cargo check --offline --workspace
cargo test --offline --workspace
cargo check --offline -p surgeist --no-default-features --features dialog-system
cargo check --offline -p surgeist --no-default-features --features render-web
cargo check --offline -p surgeist --no-default-features --features render-window
cargo check --offline -p surgeist --no-default-features --features text-accessibility
cargo check --offline -p surgeist --no-default-features --features window-accessibility
cargo check --offline -p surgeist --no-default-features --features text-accessibility,window-accessibility
cargo clippy --offline --workspace --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
cargo run --offline --manifest-path api/generator/Cargo.toml -- --check
git diff --check 359322aae90afbaf68ba7c9afffd79fb57b383d6..HEAD
test "$(git diff --diff-filter=D --name-only 359322aae90afbaf68ba7c9afffd79fb57b383d6..HEAD | LC_ALL=C sort | shasum -a 256 | awk '{print $1}')" = 5db5c695db7a34d478ec40d537e94397b403cae3ffb47fbefa1e55e8c926bd77
test -z "$(git diff --diff-filter=ACMRTUXB --name-only 359322aae90afbaf68ba7c9afffd79fb57b383d6..HEAD | rg -v '^(Cargo.toml|README.md|src/lib.rs|plans/specs/2026-08-09-root-baseline-reset.md|plans/sequences/2026-08-09-root-baseline-reset.md|plans/cycles/2026-08-09-root-baseline-reset-C01-latest-leaves.md|api/public-api.txt|api/crates/surgeist-(animation|css|dialog|layout|render|retained|runtime|shape|style|task|template|test|text|window)\.txt|crates/surgeist-(animation|css|dialog|layout|render|retained|runtime|shape|style|task|template|test|text|window))$')"
! rg -n --glob '*.rs' --glob '!target/**' --glob '!crates/*/target/**' '#\[unsafe\(|\bunsafe[[:space:]]*(\{|fn\b|trait\b|impl\b|extern\b)' .
```

After the post-publication reachability block and root remote readback succeed,
compare-and-delete only the refs owned by this cycle:

```sh
set -euo pipefail
for sub_path in $(git config -f .gitmodules --get-regexp '^submodule\..*\.path$' | awk '{print $2}' | sort)
do
  crate_name=${sub_path:t}
  for suffix in main readback-main
  do
    owned_ref="refs/surgeist/baseline-reset-final-019fe7de/${crate_name}-${suffix}"
    owned_sha=$(git -C "$sub_path" rev-parse "$owned_ref")
    git -C "$sub_path" update-ref -d "$owned_ref" "$owned_sha"
  done
done
```

Required handoff: no leaf or downstream root handoff is required. The final user
completion record names the published root SHA, exact 14 gitlinks, verification
results, review verdicts, and any unavailable command evidence.

Genuine unresolved blocker: none at plan time. If an already-installed offline
dependency or required Rust toolchain is unavailable, report the exact missing
capability rather than acquiring external software.
