//! Layout corpus commands and declarations for the shared engine.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use surgeist_generator::browser::{
    self, AcquisitionMode, BrowserCorpus, BrowserLocation, BrowserReportSummary, GenerationRequest,
};
use surgeist_generator::{CorpusLocation, RelativePath, Sha256Digest};

use crate::adapter::LayoutAdapter;

#[path = "source_owner.rs"]
mod source_owner;
use source_owner::{LIVE_DECLARATION, Manifest};

const ROOT_ENV: &str = "SURGEIST_LAYOUT_BROWSER_PARITY_ROOT";
const FILTER_ENV: &str = "SURGEIST_LAYOUT_GENERATE_FILTER";
const BROWSER_PATH_ENV: &str = "SURGEIST_BROWSER_PATH";
const DEFAULT_ROOT: &str = "tests/layout/browser_parity";

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Command {
    Generate,
    GenerateExisting,
    CheckCorpus,
    CheckTaffyCorpus,
    ImportTaffy,
}

fn parse_command(arguments: impl IntoIterator<Item = String>) -> Result<Command> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    let [command] = arguments.as_slice() else {
        return Err("usage: surgeist-layout-generate <generate|generate-existing|check-corpus|check-taffy-corpus|import-taffy>".to_string());
    };
    match command.as_str() {
        "generate" => Ok(Command::Generate),
        "generate-existing" => Ok(Command::GenerateExisting),
        "check-corpus" => Ok(Command::CheckCorpus),
        "check-taffy-corpus" => Ok(Command::CheckTaffyCorpus),
        "import-taffy" => Ok(Command::ImportTaffy),
        _ => Err(format!(
            "unknown surgeist-layout-generate command {command:?}"
        )),
    }
}

pub(super) fn run_from_env() -> Result<()> {
    let command = parse_command(env::args().skip(1))?;
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = env::var_os(ROOT_ENV).map_or_else(|| PathBuf::from(DEFAULT_ROOT), PathBuf::from);
    let browser_location = BrowserLocation::new(owner, &root).map_err(message)?;
    let location = browser_location.location();
    let manifest_path = location.corpus_root().join("corpus.toml");
    let manifest_bytes = fs::read_to_string(&manifest_path).map_err(message)?;
    let manifest_digest = Sha256Digest::from_bytes(manifest_bytes.as_bytes());
    let manifest: Manifest =
        surgeist_generator::parse_manifest(&manifest_bytes, &manifest_path).map_err(message)?;
    manifest.validate()?;
    let import = manifest.source_import()?;
    match command {
        Command::ImportTaffy => {
            let source = browser::acquire_source(owner, import.pin(), AcquisitionMode::Managed)
                .map_err(message)?;
            let inputs = BTreeMap::from([(relative("corpus.toml")?, manifest_digest)]);
            browser::import_source_with_expected_inputs(
                location,
                &import,
                source.canonical_root(),
                &inputs,
            )
            .map_err(message)?;
            Ok(())
        }
        Command::CheckTaffyCorpus => {
            let source =
                browser::acquire_source(owner, import.pin(), AcquisitionMode::ExistingOnly)
                    .map_err(message)?;
            browser::verify_source_import(location, &import, source.canonical_root())
                .map_err(message)?;
            revalidate_manifest(&manifest_path, &manifest_digest)?;
            Ok(())
        }
        Command::CheckCorpus => {
            let attestation = browser::verify_import(location, &import).map_err(message)?;
            let corpus = live_corpus(
                &manifest,
                manifest_digest,
                browser_location,
                attestation.fixture_paths(),
                &attestation,
            )?;
            browser::check_corpus(&corpus, &LayoutAdapter).map_err(message)?;
            Ok(())
        }
        Command::Generate | Command::GenerateExisting => {
            let environment = GenerationEnvironment::capture(command)?;
            let attestation = browser::verify_import(location, &import).map_err(message)?;
            let fixture_paths = attestation.fixture_paths();
            let filter = environment.filter(location, &fixture_paths)?;
            let corpus = live_corpus(
                &manifest,
                manifest_digest,
                browser_location,
                fixture_paths,
                &attestation,
            )?;
            let executable = match environment.browser_path {
                Some(path) => path,
                None => browser::acquire_browser(owner, corpus.browser(), AcquisitionMode::Managed)
                    .map_err(message)?,
            };
            let request =
                GenerationRequest::new(corpus, executable, filter, None).map_err(message)?;
            browser::generate(request, LayoutAdapter).map_err(message)?;
            Ok(())
        }
    }
}

fn revalidate_manifest(path: &Path, expected: &Sha256Digest) -> Result<()> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to revalidate corpus manifest {}: {error}",
            path.display()
        )
    })?;
    if Sha256Digest::from_bytes(bytes) != *expected {
        return Err("corpus manifest changed during source verification".to_string());
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveDeclaration {
    schema_version: u32,
    counts: BrowserReportSummary,
}

fn live_corpus(
    manifest: &Manifest,
    manifest_digest: Sha256Digest,
    location: BrowserLocation,
    fixture_paths: Vec<RelativePath>,
    attestation: &browser::SourceAttestation,
) -> Result<BrowserCorpus> {
    let path = location.location().corpus_root().join(LIVE_DECLARATION);
    let bytes = fs::read_to_string(&path)
        .map_err(|error| format!("read live corpus declaration {}: {error}", path.display()))?;
    let declaration: LiveDeclaration =
        surgeist_generator::parse_manifest(&bytes, &path).map_err(message)?;
    if declaration.schema_version != 1 {
        return Err("live corpus declaration schema_version must be 1".to_string());
    }
    let counts = declaration.counts;
    let counts = BrowserReportSummary::new(
        counts.generated(),
        counts.unsupported(),
        counts.expected_fail(),
        counts.quarantined(),
        counts.failed_to_generate(),
    )
    .map_err(message)?;
    let mut inputs = attestation.verified_inputs();
    inputs.insert(relative("corpus.toml")?, manifest_digest);
    inputs.insert(
        relative(LIVE_DECLARATION)?,
        Sha256Digest::from_bytes(bytes.as_bytes()),
    );
    manifest
        .corpus(location, fixture_paths)?
        .with_expected_counts(counts)
        .with_expected_inputs(inputs)
        .map_err(message)?
        .with_import_provenance(vec![attestation.provenance().map_err(message)?])
        .map_err(message)
}

struct GenerationEnvironment {
    browser_path: Option<RelativePath>,
    filter: Option<String>,
}

impl GenerationEnvironment {
    fn capture(command: Command) -> Result<Self> {
        for variable in ["SURGEIST_BROWSER_CACHE", "SURGEIST_BROWSER_VERSION"] {
            if env::var_os(variable).is_some() {
                return Err(format!(
                    "{variable} is manifest-owned and must be unset for generation"
                ));
            }
        }
        Self::from_values(command, utf8_env(BROWSER_PATH_ENV)?, utf8_env(FILTER_ENV)?)
    }

    fn from_values(
        command: Command,
        browser_path: Option<String>,
        filter: Option<String>,
    ) -> Result<Self> {
        let filter = filter.filter(|value| !value.is_empty());
        if command == Command::Generate {
            if browser_path.is_some() {
                return Err(format!(
                    "{BROWSER_PATH_ENV} is only valid with generate-existing"
                ));
            }
            if filter.is_some() {
                return Err(format!(
                    "{FILTER_ENV} is only valid with generate-existing diagnostic runs"
                ));
            }
        } else if browser_path.as_deref().is_none_or(str::is_empty) {
            return Err(format!(
                "generate-existing requires a non-empty {BROWSER_PATH_ENV} relative to the repository owner"
            ));
        }
        let browser_path = browser_path.map(relative).transpose()?;
        Ok(Self {
            browser_path,
            filter,
        })
    }

    fn filter(
        &self,
        location: &CorpusLocation,
        fixtures: &[RelativePath],
    ) -> Result<Option<RelativePath>> {
        let Some(filter) = &self.filter else {
            return Ok(None);
        };
        let path = relative(filter)?;
        let source = Path::new(path.as_str());
        if source
            .extension()
            .is_some_and(|extension| extension != "html")
        {
            return Err(format!(
                "{FILTER_ENV} must name an HTML fixture or directory prefix"
            ));
        }
        let root = location.corpus_root().join("html");
        let resolved = path.resolve_existing(&root).map_err(message)?;
        let matches = if resolved.is_file() {
            source
                .extension()
                .is_some_and(|extension| extension == "html")
        } else {
            fixtures
                .iter()
                .any(|fixture| Path::new(fixture.as_str()).starts_with(source))
        };
        if !matches {
            return Err(format!("{FILTER_ENV} matched no HTML fixtures"));
        }
        relative(format!("html/{filter}")).map(Some)
    }
}

fn utf8_env(variable: &str) -> Result<Option<String>> {
    env::var_os(variable)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| format!("{variable} must contain valid UTF-8"))
        })
        .transpose()
}

fn relative(value: impl AsRef<str>) -> Result<RelativePath> {
    RelativePath::new(value).map_err(message)
}
fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_check_requires_the_captured_manifest_to_remain_present_and_unchanged() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "surgeist-layout-manifest-check-{}-{nonce}.toml",
            std::process::id()
        ));
        fs::File::create_new(&path).unwrap();
        let original = b"source_pin = 'captured'\n";
        fs::write(&path, original).unwrap();
        let expected = Sha256Digest::from_bytes(original);
        revalidate_manifest(&path, &expected).expect("unchanged manifest remains current");

        fs::write(&path, b"source_pin = 'changed'\n").unwrap();
        let changed = revalidate_manifest(&path, &expected)
            .expect_err("changed manifest cannot authorize a successful source check");
        assert!(changed.contains("manifest"), "{changed}");

        fs::remove_file(&path).unwrap();
        let missing = revalidate_manifest(&path, &expected)
            .expect_err("missing manifest cannot authorize a successful source check");
        assert!(missing.contains("manifest"), "{missing}");
    }

    #[test]
    fn existing_browser_keeps_the_full_owner_relative_path() {
        let path = "tmp/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing";
        let environment = GenerationEnvironment::from_values(
            Command::GenerateExisting,
            Some(path.to_string()),
            None,
        )
        .unwrap();
        assert_eq!(environment.browser_path.unwrap().as_str(), path);
    }

    #[test]
    fn generation_commands_reject_incompatible_browser_and_filter_inputs() {
        assert!(
            GenerationEnvironment::from_values(
                Command::Generate,
                Some("tmp/browser".to_string()),
                None
            )
            .is_err()
        );
        assert!(
            GenerationEnvironment::from_values(Command::Generate, None, Some("grid".to_string()))
                .is_err()
        );
        assert!(GenerationEnvironment::from_values(Command::GenerateExisting, None, None).is_err());
        assert!(
            GenerationEnvironment::from_values(
                Command::GenerateExisting,
                Some("../browser".to_string()),
                None
            )
            .is_err()
        );
        assert!(
            GenerationEnvironment::from_values(
                Command::GenerateExisting,
                Some("/absolute/browser".to_string()),
                None
            )
            .is_err()
        );
        assert!(
            GenerationEnvironment::from_values(Command::Generate, None, Some(String::new()))
                .unwrap()
                .filter
                .is_none()
        );
    }

    #[test]
    fn only_the_five_existing_commands_are_accepted() {
        for (name, command) in [
            ("generate", Command::Generate),
            ("generate-existing", Command::GenerateExisting),
            ("check-corpus", Command::CheckCorpus),
            ("check-taffy-corpus", Command::CheckTaffyCorpus),
            ("import-taffy", Command::ImportTaffy),
        ] {
            assert_eq!(parse_command([name.to_string()]).unwrap(), command);
        }
        assert!(parse_command([]).is_err());
        assert!(parse_command(["check-corpus".to_string(), "extra".to_string()]).is_err());
        assert!(parse_command(["unknown".to_string()]).is_err());
    }
}
