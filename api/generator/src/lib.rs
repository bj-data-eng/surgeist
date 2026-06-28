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
            crate_targets.push(ApiTarget::new(name, path));
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
    fn artifact_path_is_inside_target_api_directory() {
        let target = ApiTarget::new("surgeist-task", PathBuf::from("crates/surgeist-task"));

        assert_eq!(
            target.manifest_path(),
            PathBuf::from("crates/surgeist-task/Cargo.toml")
        );
        assert_eq!(
            target.artifact_path(),
            PathBuf::from("crates/surgeist-task/api/public-api.txt")
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
        let target = ApiTarget::new("surgeist", fixture.path());

        assert_eq!(render_list_line(fixture.path(), &target), "surgeist .");
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
