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

pub fn run<I, S>(_root: &Path, args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let _cli = Cli::parse(args)?;
    Err("API target discovery is not implemented yet".to_owned())
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
}
