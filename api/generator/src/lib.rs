use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactCheck {
    Current,
    Different,
    Missing,
}

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

impl Cli {
    pub fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut args = args.into_iter().map(Into::into);
        let _program = args.next();

        let mut action = None;
        let mut selection = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--all" => set_selection(&mut selection, TargetSelection::All)?,
                "--root" => set_selection(&mut selection, TargetSelection::Root)?,
                "--crate" => {
                    let Some(name) = args.next() else {
                        return Err("--crate requires a crate name".to_owned());
                    };
                    if name.starts_with("--") {
                        return Err("--crate requires a crate name".to_owned());
                    }
                    set_selection(&mut selection, TargetSelection::Crate(name))?;
                }
                "--list" => set_action(&mut action, Action::List)?,
                "--check" => set_action(&mut action, Action::Check)?,
                unknown if unknown.starts_with("--") => {
                    return Err(format!("unknown flag {unknown}"));
                }
                unknown => {
                    return Err(format!("unexpected argument {unknown}"));
                }
            }
        }

        Ok(Self {
            action: action.unwrap_or(Action::Generate),
            selection: selection.unwrap_or(TargetSelection::All),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiTarget {
    name: String,
    root: PathBuf,
    artifact: PathBuf,
}

impl ApiTarget {
    pub fn new(
        name: impl Into<String>,
        root: impl Into<PathBuf>,
        artifact: impl Into<PathBuf>,
    ) -> Self {
        Self {
            name: name.into(),
            root: root.into(),
            artifact: artifact.into(),
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
        self.artifact.clone()
    }
}

pub fn discover_targets(root: &Path) -> Result<Vec<ApiTarget>, String> {
    let root_manifest = root.join("Cargo.toml");
    if !root_manifest.is_file() {
        return Err(format!("missing root manifest {}", root_manifest.display()));
    }

    let mut targets = vec![ApiTarget::new(
        "surgeist",
        root,
        root.join("api").join("public-api.txt"),
    )];
    let crates_dir = root.join("crates");

    if crates_dir.is_dir() {
        let mut crate_targets = Vec::new();
        for entry in std::fs::read_dir(&crates_dir).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let Some(name) = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
            else {
                continue;
            };
            if !name.starts_with("surgeist-") {
                continue;
            }
            if !path.is_dir() {
                continue;
            }
            let manifest = path.join("Cargo.toml");
            if !manifest.is_file() {
                return Err(format!(
                    "API target {name} is present under crates/ but missing {}; run `git submodule update --init` or fix the checkout",
                    manifest.display()
                ));
            }
            let artifact = root.join("api").join("crates").join(format!("{name}.txt"));
            crate_targets.push(ApiTarget::new(name, path, artifact));
        }
        crate_targets.sort_by(|left, right| left.name().cmp(right.name()));
        targets.extend(crate_targets);
    }

    Ok(targets)
}

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
            Err(format!(
                "unknown API target {name}; available targets: {available}"
            ))
        }
    }
}

pub fn run<I, S>(root: &Path, args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let cli = Cli::parse(args)?;
    let targets = select_targets(root, cli.selection)?;

    match cli.action {
        Action::List => {
            for target in targets {
                println!("{}", render_list_line(root, &target));
            }
            Ok(())
        }
        Action::Generate => {
            for target in targets {
                let generated = generate_target_artifact(&target)?;
                write_artifact(&target, &generated)?;
                println!("wrote {}", target.artifact_path().display());
            }
            Ok(())
        }
        Action::Check => {
            let mut stale = Vec::new();
            for target in targets {
                let generated = generate_target_artifact(&target)?;
                match compare_artifact(&target, &generated)? {
                    ArtifactCheck::Current => {
                        println!("current {}", target.artifact_path().display());
                    }
                    ArtifactCheck::Different | ArtifactCheck::Missing => {
                        stale.push(target.name().to_owned());
                    }
                }
            }

            if stale.is_empty() {
                Ok(())
            } else {
                Err(format!("stale API artifacts: {}", stale.join(", ")))
            }
        }
    }
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

pub fn compare_artifact(target: &ApiTarget, generated: &str) -> Result<ArtifactCheck, String> {
    let artifact = target.artifact_path();
    if !artifact.exists() {
        return Ok(ArtifactCheck::Missing);
    }
    let current = std::fs::read_to_string(&artifact)
        .map_err(|error| format!("read {}: {error}", artifact.display()))?;
    if current == generated {
        Ok(ArtifactCheck::Current)
    } else {
        Ok(ArtifactCheck::Different)
    }
}

pub fn write_artifact(target: &ApiTarget, generated: &str) -> Result<(), String> {
    let artifact = target.artifact_path();
    let directory = artifact
        .parent()
        .ok_or_else(|| format!("artifact path has no parent: {}", artifact.display()))?;
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("create {}: {error}", directory.display()))?;
    std::fs::write(&artifact, generated)
        .map_err(|error| format!("write {}: {error}", artifact.display()))
}

fn set_action(action: &mut Option<Action>, value: Action) -> Result<(), String> {
    if action.replace(value).is_some() {
        return Err("choose only one action".to_owned());
    }
    Ok(())
}

fn set_selection(
    selection: &mut Option<TargetSelection>,
    value: TargetSelection,
) -> Result<(), String> {
    if selection.replace(value).is_some() {
        return Err("choose only one target selector".to_owned());
    }
    Ok(())
}

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

        assert_eq!(
            cli.selection,
            TargetSelection::Crate("surgeist-task".to_owned())
        );
        assert_eq!(cli.action, Action::Check);
    }

    #[test]
    fn rejects_conflicting_target_selection() {
        let error = Cli::parse(["generator", "--root", "--crate", "surgeist-task"]).unwrap_err();

        assert!(error.contains("choose only one target selector"));
    }

    #[test]
    fn rejects_missing_crate_name() {
        let error = Cli::parse(["generator", "--crate"]).unwrap_err();

        assert!(error.contains("--crate requires a crate name"));
    }

    #[test]
    fn rejects_unknown_flags() {
        let error = Cli::parse(["generator", "--target", "surgeist-task"]).unwrap_err();

        assert!(error.contains("unknown flag --target"));
    }

    #[test]
    fn rejects_conflicting_actions() {
        let error = Cli::parse(["generator", "--list", "--check"]).unwrap_err();

        assert!(error.contains("choose only one action"));
    }

    #[test]
    fn artifact_path_is_root_owned_for_crate_target() {
        let target = ApiTarget::new(
            "surgeist-task",
            PathBuf::from("crates/surgeist-task"),
            PathBuf::from("api/crates/surgeist-task.txt"),
        );

        assert_eq!(
            target.manifest_path(),
            PathBuf::from("crates/surgeist-task/Cargo.toml")
        );
        assert_eq!(
            target.artifact_path(),
            PathBuf::from("api/crates/surgeist-task.txt")
        );
    }

    #[test]
    fn discovers_root_then_surgeist_submodules_sorted_by_name() {
        let fixture = TempFixture::new("surgeist-api-targets");
        fixture.file("Cargo.toml", "[package]\nname = \"surgeist\"\n");
        fixture.file(
            "crates/surgeist-task/Cargo.toml",
            "[package]\nname = \"surgeist-task\"\n",
        );
        fixture.file(
            "crates/surgeist-css/Cargo.toml",
            "[package]\nname = \"surgeist-css\"\n",
        );
        fixture.file(
            "crates/not-surgeist/Cargo.toml",
            "[package]\nname = \"not-surgeist\"\n",
        );

        let targets = discover_targets(fixture.path()).unwrap();
        let names = targets.iter().map(ApiTarget::name).collect::<Vec<_>>();

        assert_eq!(names, vec!["surgeist", "surgeist-css", "surgeist-task"]);
    }

    #[test]
    fn ignores_matching_files_under_crates_directory() {
        let fixture = TempFixture::new("surgeist-api-target-files");
        fixture.file("Cargo.toml", "[package]\nname = \"surgeist\"\n");
        fixture.file("crates/surgeist-not-a-directory", "not a crate directory\n");

        let targets = discover_targets(fixture.path()).unwrap();
        let names = targets.iter().map(ApiTarget::name).collect::<Vec<_>>();

        assert_eq!(names, vec!["surgeist"]);
    }

    #[test]
    fn selecting_missing_crate_reports_available_targets() {
        let fixture = TempFixture::new("surgeist-api-missing-target");
        fixture.file("Cargo.toml", "[package]\nname = \"surgeist\"\n");
        fixture.file(
            "crates/surgeist-css/Cargo.toml",
            "[package]\nname = \"surgeist-css\"\n",
        );

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

    #[test]
    fn list_line_formats_root_as_dot() {
        let fixture = TempFixture::new("surgeist-api-list-root");
        let target = ApiTarget::new(
            "surgeist",
            fixture.path(),
            fixture.path().join("api/public-api.txt"),
        );

        assert_eq!(render_list_line(fixture.path(), &target), "surgeist .");
    }

    #[test]
    fn render_api_artifact_uses_target_name_header() {
        let artifact = render_api_artifact(
            "surgeist-task",
            "pub struct Task;\n",
            &["2608".to_owned(), "2605".to_owned()],
        );

        assert!(artifact.starts_with("# surgeist-task public API\n"));
        assert!(artifact.contains("# missing rustdoc item IDs: 2608, 2605\n"));
        assert!(artifact.ends_with("pub struct Task;\n"));
    }

    #[test]
    fn check_artifact_reports_difference_without_writing() {
        let fixture = TempFixture::new("surgeist-api-check-different");
        fixture.file("api/public-api.txt", "old artifact\n");
        let target = ApiTarget::new(
            "surgeist-task",
            fixture.path(),
            fixture.path().join("api/public-api.txt"),
        );

        let check = compare_artifact(&target, "new artifact\n").unwrap();

        assert_eq!(check, ArtifactCheck::Different);
        assert_eq!(
            std::fs::read_to_string(target.artifact_path()).unwrap(),
            "old artifact\n"
        );
    }

    #[test]
    fn write_artifact_creates_api_directory() {
        let fixture = TempFixture::new("surgeist-api-write");
        let target = ApiTarget::new(
            "surgeist-task",
            fixture.path(),
            fixture.path().join("api/crates/surgeist-task.txt"),
        );

        write_artifact(&target, "generated artifact\n").unwrap();

        assert_eq!(
            std::fs::read_to_string(fixture.path().join("api/crates/surgeist-task.txt")).unwrap(),
            "generated artifact\n"
        );
    }

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
}
