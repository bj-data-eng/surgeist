# Surgeist Root API Generator Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the root-owned public API generator into a multi-target tool that can refresh or check API artifacts for the root `surgeist` crate and every linked `crates/surgeist-*` submodule crate.

**Architecture:** Keep API generation tooling owned by the root repo. The generator discovers the facade crate plus initialized submodule crates under `crates/`, runs rustdoc/public-api for each selected target, and writes that target's own `api/public-api.txt` artifact in place. Crate repos must not copy or own `api/generator`; any existing crate-local generator copies are removed as part of this consolidation.

**Tech Stack:** Rust 2024-compatible generator crate, `public-api`, `rustdoc-json`, root submodule layout, crate-local generator unit tests, command-line generation/check modes.

---

## Review Gate

Each numbered task below is a scoped implementation unit. Before committing a task or moving to the next numbered task, assign a separate reviewer to inspect the task's changes against this plan, root `AGENTS.md`, and the current root/submodule layout.

The final reviewer must verify:

- the generator is still root-owned only;
- no `api/generator` files are copied into sibling crate repos;
- default generation covers root plus all initialized `crates/surgeist-*` crate targets;
- targeted generation works for one crate;
- check mode detects stale artifacts without rewriting them;
- README and handoff docs no longer imply per-crate generator ownership.

## Current State

Root currently owns the generator that must become authoritative:

```text
surgeist/api/generator/Cargo.toml
surgeist/api/generator/src/main.rs
```

The existing generator assumes it lives under the crate being generated:

```rust
let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"))
    .ancestors()
    .nth(2)
    .expect("generator lives under crate_root/api/generator");
```

That only refreshes:

```text
surgeist/api/public-api.txt
```

Some submodule crates may still contain historical `api/generator` copies from
before consolidation. Those copies are stale after this plan and must be
deleted in the owning crate repos, not merely ignored from root.

The root repo also links submodule crates with their own artifacts:

```text
surgeist/crates/surgeist-css/api/public-api.txt
surgeist/crates/surgeist-dialog/api/public-api.txt
surgeist/crates/surgeist-layout/api/public-api.txt
surgeist/crates/surgeist-render/api/public-api.txt
surgeist/crates/surgeist-retained/api/public-api.txt
surgeist/crates/surgeist-shape/api/public-api.txt
surgeist/crates/surgeist-style/api/public-api.txt
surgeist/crates/surgeist-task/api/public-api.txt
surgeist/crates/surgeist-test/api/public-api.txt
surgeist/crates/surgeist-text/api/public-api.txt
surgeist/crates/surgeist-window/api/public-api.txt
```

## Command Contract

Keep the existing command as the full refresh:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Expected: refresh root plus every initialized `crates/surgeist-*` crate artifact.

Add explicit modes:

```sh
cargo run --manifest-path api/generator/Cargo.toml -- --all
cargo run --manifest-path api/generator/Cargo.toml -- --root
cargo run --manifest-path api/generator/Cargo.toml -- --crate surgeist-task
cargo run --manifest-path api/generator/Cargo.toml -- --list
cargo run --manifest-path api/generator/Cargo.toml -- --check
cargo run --manifest-path api/generator/Cargo.toml -- --check --crate surgeist-task
```

Rules:

- no args is equivalent to `--all`;
- `--all` includes root first, then submodule crates sorted by crate directory name;
- `--root` includes only the root facade crate;
- `--crate <name>` matches the submodule directory name such as `surgeist-task`;
- `--list` prints selected target names and paths without generating;
- `--check` exits nonzero when any selected artifact differs from generated output;
- a directory under `crates/` whose name starts with `surgeist-` but lacks
  `Cargo.toml` is diagnosed as an uninitialized or corrupt submodule checkout;
  the error message must tell the operator to run `git submodule update --init`
  or fix the checkout, rather than silently skipping the target;
- missing `api/` directories are created only in generation mode, not in check mode.

## File Structure

- Modify: `api/generator/src/main.rs`
  - Keep only process entrypoint and user-facing error printing.
- Create: `api/generator/src/lib.rs`
  - Target discovery, argument parsing, generation orchestration, artifact rendering, check-mode comparison.
- Modify: `api/generator/Cargo.toml`
  - No new dependency is expected; Cargo's implicit library target from
    `src/lib.rs` is sufficient unless implementation discovers a concrete need.
- Modify: `README.md`
  - Document root-owned multi-target API artifact workflow.
- Test: `api/generator/src/lib.rs`
  - Unit tests for argument parsing, target discovery from temporary directory fixtures, artifact paths, and check-mode comparison behavior.

## Task 1: Generator Target Model And CLI Parsing

**Files:**
- Create: `api/generator/src/lib.rs`
- Modify: `api/generator/src/main.rs`
- Test: `api/generator/src/lib.rs`

- [ ] **Step 1: Add target and mode tests**

In `api/generator/src/lib.rs`, add tests for CLI parsing and target representation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn no_args_defaults_to_all_generation() {
        let cli = Cli::parse(["generator"]).unwrap();

        assert_eq!(cli.selection, TargetSelection::All);
        assert_eq!(cli.action, Action::Generate);
    }

    #[test]
    fn parses_single_crate_check() {
        let cli = Cli::parse(["generator", "--check", "--crate", "surgeist-task"]).unwrap();

        assert_eq!(cli.selection, TargetSelection::Crate("surgeist-task".to_owned()));
        assert_eq!(cli.action, Action::Check);
    }

    #[test]
    fn rejects_conflicting_target_selection() {
        let error = Cli::parse(["generator", "--root", "--crate", "surgeist-task"]).unwrap_err();

        assert!(error.contains("choose only one target selector"));
    }

    #[test]
    fn artifact_path_is_inside_target_api_directory() {
        let target = ApiTarget::new("surgeist-task", PathBuf::from("crates/surgeist-task"));

        assert_eq!(target.manifest_path(), PathBuf::from("crates/surgeist-task/Cargo.toml"));
        assert_eq!(target.artifact_path(), PathBuf::from("crates/surgeist-task/api/public-api.txt"));
    }
}
```

- [ ] **Step 2: Implement CLI and target structs**

Add this shape to `api/generator/src/lib.rs`:

```rust
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Generate,
    Check,
    List,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetSelection {
    All,
    Root,
    Crate(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cli {
    pub action: Action,
    pub selection: TargetSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiTarget {
    name: String,
    root: PathBuf,
}

impl ApiTarget {
    pub fn new(name: impl Into<String>, root: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            root: root.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.root.join("Cargo.toml")
    }

    pub fn artifact_path(&self) -> PathBuf {
        self.root.join("api").join("public-api.txt")
    }
}
```

Add `Cli::parse` using `std::env::args`-style iterators. It must accept the command contract above and reject:

```text
--crate without a crate name
unknown flags
multiple target selectors, such as --root --crate surgeist-task
multiple actions, such as --list --check
```

Also add a temporary compile-safe orchestration stub so the Task 1 binary builds
before discovery and generation exist:

```rust
pub fn run<I, S>(_root: &Path, args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let _cli = Cli::parse(args)?;
    Err("API target discovery is not implemented yet".to_owned())
}
```

Task 2 replaces this stub with list-capable target discovery. Task 3 extends the
same function with generation and check behavior.

- [ ] **Step 3: Replace main with a thin entrypoint**

Replace `api/generator/src/main.rs` with:

```rust
use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("generator lives under surgeist/api/generator");

    if let Err(error) = surgeist_api_generator::run(root, std::env::args()) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
```

- [ ] **Step 4: Verify Task 1**

Run:

```sh
cargo test --manifest-path api/generator/Cargo.toml
cargo clippy --manifest-path api/generator/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path api/generator/Cargo.toml --check
cargo fmt --check
```

Expected: generator unit tests and Clippy pass, and formatting is clean for
both the generator manifest and root repo.

Commit:

```sh
git add api/generator/src/main.rs api/generator/src/lib.rs
git commit -m "Add API generator target CLI"
```

## Task 2: Root And Submodule Target Discovery

**Files:**
- Modify: `api/generator/src/lib.rs`
- Test: `api/generator/src/lib.rs`

- [ ] **Step 1: Add discovery tests**

Add tests using temporary fixture directories under `std::env::temp_dir()`:

```rust
#[test]
fn discovers_root_then_surgeist_submodules_sorted_by_name() {
    let fixture = TempFixture::new("surgeist-api-targets");
    fixture.file("Cargo.toml", "[package]\nname = \"surgeist\"\n");
    fixture.file("crates/surgeist-task/Cargo.toml", "[package]\nname = \"surgeist-task\"\n");
    fixture.file("crates/surgeist-css/Cargo.toml", "[package]\nname = \"surgeist-css\"\n");
    fixture.file("crates/not-surgeist/Cargo.toml", "[package]\nname = \"not-surgeist\"\n");

    let targets = discover_targets(fixture.path()).unwrap();
    let names = targets.iter().map(ApiTarget::name).collect::<Vec<_>>();

    assert_eq!(names, vec!["surgeist", "surgeist-css", "surgeist-task"]);
}

#[test]
fn selecting_missing_crate_reports_available_targets() {
    let fixture = TempFixture::new("surgeist-api-missing-target");
    fixture.file("Cargo.toml", "[package]\nname = \"surgeist\"\n");
    fixture.file("crates/surgeist-css/Cargo.toml", "[package]\nname = \"surgeist-css\"\n");

    let error = select_targets(
        fixture.path(),
        TargetSelection::Crate("surgeist-task".to_owned()),
    )
    .unwrap_err();

    assert!(error.contains("surgeist-task"));
    assert!(error.contains("available targets: surgeist, surgeist-css"));
}

#[test]
fn uninitialized_surgeist_submodule_reports_recovery_hint() {
    let fixture = TempFixture::new("surgeist-api-uninitialized-submodule");
    fixture.file("Cargo.toml", "[package]\nname = \"surgeist\"\n");
    std::fs::create_dir_all(fixture.path().join("crates/surgeist-css")).unwrap();

    let error = discover_targets(fixture.path()).unwrap_err();

    assert!(error.contains("surgeist-css"));
    assert!(error.contains("git submodule update --init"));
    assert!(error.contains("fix the checkout"));
}
```

Implement `TempFixture` in the test module using only the standard library:

```rust
struct TempFixture {
    root: std::path::PathBuf,
}

impl TempFixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn path(&self) -> &std::path::Path {
        &self.root
    }

    fn file(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}

impl Drop for TempFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
```

- [ ] **Step 2: Implement discovery and selection**

Add:

```rust
pub fn discover_targets(root: &Path) -> Result<Vec<ApiTarget>, String> {
    let root_manifest = root.join("Cargo.toml");
    if !root_manifest.is_file() {
        return Err(format!("missing root manifest {}", root_manifest.display()));
    }

    let mut targets = vec![ApiTarget::new("surgeist", root)];
    let crates_dir = root.join("crates");

    if crates_dir.is_dir() {
        let mut crate_targets = Vec::new();
        for entry in std::fs::read_dir(&crates_dir).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !name.starts_with("surgeist-") {
                continue;
            }
            let manifest = path.join("Cargo.toml");
            if !manifest.is_file() {
                return Err(format!(
                    "API target {name} is present under crates/ but missing {}; run `git submodule update --init` or fix the checkout",
                    manifest.display()
                ));
            }
            crate_targets.push(ApiTarget::new(name, path));
        }
        crate_targets.sort_by(|left, right| left.name().cmp(right.name()));
        targets.extend(crate_targets);
    }

    Ok(targets)
}
```

Add:

```rust
pub fn select_targets(root: &Path, selection: TargetSelection) -> Result<Vec<ApiTarget>, String> {
    let targets = discover_targets(root)?;
    match selection {
        TargetSelection::All => Ok(targets),
        TargetSelection::Root => Ok(targets
            .into_iter()
            .filter(|target| target.name() == "surgeist")
            .collect()),
        TargetSelection::Crate(name) => {
            let matches = targets
                .iter()
                .filter(|target| target.name() == name)
                .cloned()
                .collect::<Vec<_>>();
            if !matches.is_empty() {
                return Ok(matches);
            }
            let available = targets
                .iter()
                .map(ApiTarget::name)
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!("unknown API target {name}; available targets: {available}"))
        }
    }
}
```

- [ ] **Step 3: Implement list orchestration**

Replace the Task 1 `run` stub with list-capable orchestration:

```rust
pub fn run<I, S>(root: &Path, args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let cli = Cli::parse(args)?;
    let targets = select_targets(root, cli.selection)?;

    if cli.action == Action::List {
        for target in targets {
            println!("{}", render_list_line(root, &target));
        }
        return Ok(());
    }

    Err("API artifact generation is not implemented yet".to_owned())
}

pub fn render_list_line(root: &Path, target: &ApiTarget) -> String {
    let relative = target
        .root()
        .strip_prefix(root)
        .ok()
        .filter(|path| !path.as_os_str().is_empty());
    let path = relative
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| ".".to_owned());
    format!("{} {}", target.name(), path)
}
```

Add a focused test for root path formatting:

```rust
#[test]
fn list_line_formats_root_as_dot() {
    let fixture = TempFixture::new("surgeist-api-list-root");
    let target = ApiTarget::new("surgeist", fixture.path());

    assert_eq!(render_list_line(fixture.path(), &target), "surgeist .");
}
```

- [ ] **Step 4: Verify Task 2**

Run:

```sh
cargo test --manifest-path api/generator/Cargo.toml
cargo clippy --manifest-path api/generator/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path api/generator/Cargo.toml --check
cargo run --manifest-path api/generator/Cargo.toml -- --list
```

Expected list output contains root first and then sorted submodule names:

```text
surgeist .
surgeist-css crates/surgeist-css
surgeist-dialog crates/surgeist-dialog
...
surgeist-window crates/surgeist-window
```

Commit:

```sh
git add api/generator/src/lib.rs
git commit -m "Discover API generator targets"
```

## Task 3: Multi-Target Generation And Check Mode

**Files:**
- Modify: `api/generator/src/lib.rs`
- Test: `api/generator/src/lib.rs`

- [ ] **Step 1: Add artifact rendering and check tests**

Add tests that do not invoke rustdoc:

```rust
#[test]
fn render_api_artifact_uses_target_name_header() {
    let rendered = render_api_artifact("surgeist-task", "pub struct TaskId\n", &[]);

    assert!(rendered.starts_with(
        "# surgeist-task public API\n# generated by Surgeist public API artifact tooling\n"
    ));
    assert!(rendered.contains("pub struct TaskId"));
}

#[test]
fn check_artifact_reports_difference_without_writing() {
    let fixture = TempFixture::new("surgeist-api-check");
    fixture.file("api/public-api.txt", "old artifact\n");
    let target = ApiTarget::new("surgeist", fixture.path());

    let report = compare_artifact(&target, "new artifact\n").unwrap();

    assert_eq!(report, ArtifactCheck::Different);
    assert_eq!(
        std::fs::read_to_string(fixture.path().join("api/public-api.txt")).unwrap(),
        "old artifact\n"
    );
}

#[test]
fn write_artifact_creates_api_directory() {
    let fixture = TempFixture::new("surgeist-api-write");
    let target = ApiTarget::new("surgeist", fixture.path());

    write_artifact(&target, "new artifact\n").unwrap();

    assert_eq!(
        std::fs::read_to_string(fixture.path().join("api/public-api.txt")).unwrap(),
        "new artifact\n"
    );
}
```

- [ ] **Step 2: Move existing generation logic behind target-aware functions**

Add:

```rust
pub fn generate_target_artifact(target: &ApiTarget) -> Result<String, String> {
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain(public_api::MINIMUM_NIGHTLY_RUST_VERSION)
        .manifest_path(target.manifest_path())
        .build()
        .map_err(|error| format!("build rustdoc JSON for {}: {error}", target.name()))?;

    let public_api = public_api::Builder::from_rustdoc_json(rustdoc_json)
        .omit_blanket_impls(true)
        .omit_auto_trait_impls(true)
        .omit_auto_derived_impls(true)
        .build()
        .map_err(|error| format!("derive public API for {}: {error}", target.name()))?;

    let missing_item_ids = public_api
        .missing_item_ids()
        .map(u32::to_string)
        .collect::<Vec<_>>();

    Ok(render_api_artifact(
        target.name(),
        &public_api.to_string(),
        &missing_item_ids,
    ))
}

pub fn render_api_artifact(name: &str, api: &str, missing_item_ids: &[String]) -> String {
    let mut output = String::new();
    output.push_str("# ");
    output.push_str(name);
    output.push_str(" public API\n");
    output.push_str("# generated by Surgeist public API artifact tooling\n");

    if !missing_item_ids.is_empty() {
        output.push_str("# missing rustdoc item IDs: ");
        output.push_str(&missing_item_ids.join(", "));
        output.push('\n');
    }

    output.push('\n');
    output.push_str(api);
    output
}
```

Add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactCheck {
    Current,
    Different,
    Missing,
}

pub fn compare_artifact(target: &ApiTarget, generated: &str) -> Result<ArtifactCheck, String> {
    let artifact = target.artifact_path();
    if !artifact.exists() {
        return Ok(ArtifactCheck::Missing);
    }
    let existing = std::fs::read_to_string(&artifact)
        .map_err(|error| format!("read {}: {error}", artifact.display()))?;
    if existing == generated {
        Ok(ArtifactCheck::Current)
    } else {
        Ok(ArtifactCheck::Different)
    }
}

pub fn write_artifact(target: &ApiTarget, generated: &str) -> Result<(), String> {
    let artifact = target.artifact_path();
    let parent = artifact
        .parent()
        .ok_or_else(|| format!("artifact has no parent: {}", artifact.display()))?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("create {}: {error}", parent.display()))?;
    std::fs::write(&artifact, generated)
        .map_err(|error| format!("write {}: {error}", artifact.display()))?;
    Ok(())
}
```

- [ ] **Step 3: Extend run orchestration for generation and check**

Replace the Task 2 `run` body with:

```rust
pub fn run<I, S>(root: &Path, args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let cli = Cli::parse(args)?;
    let targets = select_targets(root, cli.selection)?;

    if cli.action == Action::List {
        for target in targets {
            println!("{}", render_list_line(root, &target));
        }
        return Ok(());
    }

    let mut stale = Vec::new();
    for target in targets {
        let generated = generate_target_artifact(&target)?;
        match cli.action {
            Action::Generate => {
                write_artifact(&target, &generated)?;
                println!("wrote {}", target.artifact_path().display());
            }
            Action::Check => match compare_artifact(&target, &generated)? {
                ArtifactCheck::Current => {
                    println!("current {}", target.artifact_path().display());
                }
                ArtifactCheck::Different | ArtifactCheck::Missing => {
                    stale.push(target);
                }
            },
            Action::List => unreachable!("handled before generation"),
        }
    }

    if stale.is_empty() {
        return Ok(());
    }

    let names = stale
        .iter()
        .map(ApiTarget::name)
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!("stale API artifacts: {names}"))
}
```

- [ ] **Step 4: Verify Task 3**

Run:

```sh
cargo test --manifest-path api/generator/Cargo.toml
cargo clippy --manifest-path api/generator/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path api/generator/Cargo.toml --check
cargo run --manifest-path api/generator/Cargo.toml -- --root
cargo run --manifest-path api/generator/Cargo.toml -- --check --root
```

Expected:

- tests pass;
- root API artifact is written;
- root check reports current after generation.

Commit:

```sh
git add api/generator/src/lib.rs api/generator/src/main.rs api/public-api.txt
git commit -m "Generate root API artifacts by target"
```

## Task 4: Full Submodule Artifact Refresh And Generator Copy Removal

**Files:**
- Modify: `api/public-api.txt`
- Modify: `crates/surgeist-*/api/public-api.txt`
- Delete if present: `crates/surgeist-*/api/generator/**`

- [ ] **Step 1: Run full generation**

Run:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Expected:

- root and every initialized `crates/surgeist-*` target is printed as written;
- no `api/generator` files are created under any submodule crate;
- each target has `api/public-api.txt`.

- [ ] **Step 2: Inventory stale crate-local generator copies**

Run:

```sh
find crates \( -path '*/api/generator' -o -path '*/api/generator/*' \) -print
```

If any paths are printed, group them by owning crate and assign one crate-owned
cleanup task per affected crate. These are stale tool copies. The root
generator is now the only API generation implementation.

The cleanup must be performed from inside the owning crate repo or that crate's
Codex project, not as a root-owned edit. For each affected crate, use the
crate path as the working directory and run:

```sh
cd crates/surgeist-css
git status --short --branch
rm -rf api/generator
git status --short --branch
```

Do not delete crate-owned `api/public-api.txt` artifacts. Assign a separate
reviewer to confirm the cleanup removed only `api/generator` and did not touch
source, artifacts, or unrelated files. Commit and push that crate-owned cleanup
before root submodule pointer updates.

- [ ] **Step 3: Run full check mode**

Run:

```sh
cargo run --manifest-path api/generator/Cargo.toml -- --check
```

Expected:

- exits successfully;
- prints `current .../api/public-api.txt` for each selected target or otherwise clearly confirms current artifacts;
- does not modify files.

- [ ] **Step 4: Review artifact diff**

Run:

```sh
git diff --stat
git diff -- api/public-api.txt crates/*/api/public-api.txt
find crates \( -path '*/api/generator' -o -path '*/api/generator/*' \) -print
git submodule foreach 'git status --short --branch'
```

Expected:

- artifact diffs are generated from source;
- `find` prints nothing, including no empty `api/generator` directories.
- any crate-local generated artifact changes or `api/generator` deletions are
  visible as changes inside those submodule repos.

- [ ] **Step 5: Verify before any artifact commits**

Run:

```sh
cargo test --manifest-path api/generator/Cargo.toml
cargo clippy --manifest-path api/generator/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path api/generator/Cargo.toml --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

If a command is expected to fail because of a known upstream crate issue, stop
and record the failing command, failing crate/test/error, and owning follow-up
issue before deciding whether this plan should proceed.

- [ ] **Step 6: Commit generated artifact changes in owning repos**

Commit changes in the repo that owns each changed file:

- root `api/public-api.txt` is committed in `/Users/codex/Development/surgeist`;
- each changed `crates/surgeist-*/api/public-api.txt` is committed inside that
  crate repo;
- each deleted `crates/surgeist-*/api/generator/**` copy is committed inside
  that crate repo.

For each changed submodule crate:

```sh
cd crates/surgeist-css
git status --short --branch
git diff --stat
cargo test -p surgeist-css
cargo clippy -p surgeist-css --all-targets -- -D warnings
cargo fmt --check
git add -A api/public-api.txt api/generator
git commit -m "Refresh API artifact from root generator"
git push
```

Use the actual crate path and commit message appropriate to the changed files.
Use the owning crate's documented focused check set when it differs from the
baseline commands above, and record any intentional skip with the exact command,
reason, and owning follow-up before committing.

Before each crate commit, assign a separate owning-crate reviewer to inspect the
generated `api/public-api.txt` artifact diff and any `api/generator` deletion.
The reviewer must confirm the artifact was generated by the root tool, the diff
matches the current source-derived API shape, and no unrelated files were
changed.

If only the stale generator copy was deleted, use a message such as:

```sh
git commit -m "Remove crate-local API generator copy"
```

Workers must not stage or commit submodule-owned files from the root repo.

- [ ] **Step 7: Update root pointers after crate commits are pushed**

After submodule crate commits are pushed and fetchable, update the root repo's
submodule pointers and review the pointer summary:

```sh
git status --short --branch
git diff --submodule=log
```

Then rerun the root pointer gates:

```sh
cargo test --manifest-path api/generator/Cargo.toml
cargo clippy --manifest-path api/generator/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path api/generator/Cargo.toml --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

Commit only root-owned changes from the root repo:

```sh
git add api/public-api.txt crates/surgeist-*
git commit -m "Refresh API artifacts from root generator"
```

## Task 5: README And Workflow Documentation

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`

- [ ] **Step 1: Update README API artifact section**

Replace the current single-artifact section with:

````markdown
## API Artifacts

Root owns API artifact generation for the facade and linked Surgeist crate
submodules. Do not copy `api/generator` into crate repos.

Refresh all API artifacts:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Refresh one target:

```sh
cargo run --manifest-path api/generator/Cargo.toml -- --crate surgeist-task
```

Check artifacts without rewriting:

```sh
cargo run --manifest-path api/generator/Cargo.toml -- --check
```

API refresh tooling is command-only. Do not wire it into normal `cargo test`
runs.
````

- [ ] **Step 2: Update AGENTS API coordination guidance**

In `AGENTS.md`, update the generated API guidance to include:

````markdown
The root repo owns the API artifact generator at `api/generator`. Crate repos may
carry generated `api/public-api.txt` artifacts, but they must not carry their own
generator copy. From root, use:

```sh
cargo run --manifest-path api/generator/Cargo.toml
cargo run --manifest-path api/generator/Cargo.toml -- --check
cargo run --manifest-path api/generator/Cargo.toml -- --crate surgeist-task
```

Run API artifact refresh after the owning crate's source changes are committed
and pushed. The root generator may write `api/public-api.txt` inside submodule
working trees, but those artifact changes and stale generator deletions are
reviewed, committed, and pushed in the owning crate repos before root submodule
pointer updates. The root repo commits only root-owned generator/docs/artifacts
and submodule pointer updates.
````

- [ ] **Step 3: Verify docs and commit**

Run:

```sh
cargo run --manifest-path api/generator/Cargo.toml -- --list
cargo run --manifest-path api/generator/Cargo.toml -- --check
cargo clippy --manifest-path api/generator/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path api/generator/Cargo.toml --check
cargo fmt --check
git diff -- README.md AGENTS.md
```

Commit:

```sh
git add README.md AGENTS.md
git commit -m "Document root API generator workflow"
```

## Final Review Checklist

Before marking this plan complete:

- Confirm `cargo test --manifest-path api/generator/Cargo.toml` passes.
- Confirm `cargo clippy --manifest-path api/generator/Cargo.toml --all-targets -- -D warnings` passes.
- Confirm `cargo fmt --manifest-path api/generator/Cargo.toml --check` passes.
- Confirm `cargo run --manifest-path api/generator/Cargo.toml -- --list` lists root and all initialized `crates/surgeist-*` targets.
- Confirm `cargo run --manifest-path api/generator/Cargo.toml -- --check` passes after generation.
- Confirm root pointer gates pass:

```sh
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

If any gate is intentionally skipped, the final task output must name the exact
command, reason, and owning follow-up.

- Confirm no submodule contains `api/generator`:

```sh
find crates \( -path '*/api/generator' -o -path '*/api/generator/*' \) -print
```

Expected: no output, including no empty `api/generator` directories.

- Confirm root artifact and each submodule artifact were generated from root-owned tooling.
- Confirm generated submodule artifact changes and stale generator deletions
  were committed in the owning crate repos before root pointer updates.
- Confirm `README.md` and `AGENTS.md` describe root generator ownership.
- Request a clean reviewer to inspect the generator code, generated artifacts, docs, and root/submodule boundary.
