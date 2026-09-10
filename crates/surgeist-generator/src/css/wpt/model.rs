use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    GeneratorError, GeneratorErrorKind, ManifestVersion, RelativePath, Result, Sha256Digest,
    SourceRevision, parse_manifest,
};

pub(super) const SIDECAR: &str = ".surgeist-source.json";
const REPOSITORY: &str = "https://github.com/web-platform-tests/wpt.git";
const GENERATOR: &str = "surgeist-css-generate";
const VERIFICATION: &str = "manifest-file-digests";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct DeclaredSource {
    kind: String,
    repository: String,
    revision: SourceRevision,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSource {
    kind: String,
    repository: String,
    revision: SourceRevision,
}

impl RawSource {
    fn validate(self) -> Result<DeclaredSource> {
        if self.kind != "wpt" || self.repository != REPOSITORY {
            return Err(invalid(
                "source must declare wpt and its canonical HTTPS Git URL",
            ));
        }
        Ok(DeclaredSource {
            kind: self.kind,
            repository: self.repository,
            revision: self.revision,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifestSource {
    kind: String,
    repository: String,
    revision: SourceRevision,
    import_root: RelativePath,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    path: RelativePath,
    sha256: Sha256Digest,
    licenses: Vec<RelativePath>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLicense {
    path: RelativePath,
    sha256: Sha256Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct FileRecord {
    path: RelativePath,
    sha256: Sha256Digest,
    licenses: Vec<RelativePath>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct LicenseRecord {
    path: RelativePath,
    sha256: Sha256Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct Records {
    files: Vec<FileRecord>,
    licenses: Vec<LicenseRecord>,
}

impl Records {
    fn validate(files: Vec<RawFile>, licenses: Vec<RawLicense>) -> Result<Self> {
        if files.is_empty() || licenses.is_empty() {
            return Err(invalid(
                "files and licenses must both be nonempty explicit allowlists",
            ));
        }
        let mut paths = BTreeSet::new();
        let mut license_paths = BTreeSet::new();
        let mut validated_licenses = Vec::new();
        for license in licenses {
            register_path(&mut paths, &license.path)?;
            license_paths.insert(license.path.clone());
            validated_licenses.push(LicenseRecord {
                path: license.path,
                sha256: license.sha256,
            });
        }
        let mut validated_files = Vec::new();
        for file in files {
            register_path(&mut paths, &file.path)?;
            if file.licenses.is_empty() {
                return Err(invalid(format!(
                    "file has no license binding: {}",
                    file.path.as_str()
                )));
            }
            let mut bindings = BTreeSet::new();
            for license in file.licenses {
                if !license_paths.contains(&license) || !bindings.insert(license.clone()) {
                    return Err(invalid(format!(
                        "file {} has an unknown or duplicate license binding: {}",
                        file.path.as_str(),
                        license.as_str(),
                    )));
                }
            }
            validated_files.push(FileRecord {
                path: file.path,
                sha256: file.sha256,
                licenses: bindings.into_iter().collect(),
            });
        }
        for path in &paths {
            let mut parent = path.as_str();
            while let Some((prefix, _)) = parent.rsplit_once('/') {
                if paths.contains(prefix) {
                    return Err(invalid(format!(
                        "file path is also a parent directory: {prefix}"
                    )));
                }
                parent = prefix;
            }
        }
        let mut spellings = BTreeMap::new();
        for path in validated_files
            .iter()
            .map(|file| &file.path)
            .chain(validated_licenses.iter().map(|license| &license.path))
        {
            let mut prefix = String::new();
            for component in path.as_str().split('/') {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(component);
                if let Some(previous) =
                    spellings.insert(prefix.to_ascii_lowercase(), prefix.clone())
                    && previous != prefix
                {
                    return Err(invalid(format!(
                        "case-aliased path components: {previous} and {prefix}"
                    )));
                }
            }
        }
        validated_files.sort_by(|left, right| left.path.cmp(&right.path));
        validated_licenses.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(Self {
            files: validated_files,
            licenses: validated_licenses,
        })
    }

    fn entries(&self) -> impl Iterator<Item = (&RelativePath, &Sha256Digest)> {
        self.files
            .iter()
            .map(|file| (&file.path, &file.sha256))
            .chain(
                self.licenses
                    .iter()
                    .map(|license| (&license.path, &license.sha256)),
            )
    }
}

fn register_path(paths: &mut BTreeSet<String>, path: &RelativePath) -> Result<()> {
    if path.as_str().chars().any(char::is_control)
        || path.as_str().split('/').any(|component| {
            let folded = component.to_ascii_lowercase();
            matches!(folded.as_str(), ".git" | ".surgeist-generator" | SIDECAR)
                || folded.starts_with("._surgeist-")
        })
    {
        return Err(invalid(format!(
            "file path contains a reserved component: {}",
            path.as_str()
        )));
    }
    if !paths.insert(path.as_str().to_ascii_lowercase()) {
        return Err(invalid(format!(
            "duplicate or case-aliased file path: {}",
            path.as_str()
        )));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    schema_version: ManifestVersion,
    source: RawManifestSource,
    files: Vec<RawFile>,
    licenses: Vec<RawLicense>,
}

pub(super) struct Manifest {
    source: DeclaredSource,
    import_root: RelativePath,
    records: Records,
}

impl Manifest {
    pub(super) fn parse(bytes: &[u8], path: &Path) -> Result<Self> {
        let text = std::str::from_utf8(bytes).map_err(|_| invalid("manifest is not UTF-8"))?;
        let raw: RawManifest = parse_manifest(text, path)?;
        raw.schema_version.require(ManifestVersion::new(1)?, path)?;
        super::super::manifest::require_root(&raw.source.import_root, "source.import_root")?;
        register_path(&mut BTreeSet::new(), &raw.source.import_root)?;
        if raw
            .source
            .import_root
            .as_str()
            .eq_ignore_ascii_case("corpus.toml")
        {
            return Err(invalid(
                "source.import_root is reserved for the corpus manifest",
            ));
        }
        let source = RawSource {
            kind: raw.source.kind,
            repository: raw.source.repository,
            revision: raw.source.revision,
        }
        .validate()?;
        Ok(Self {
            source,
            import_root: raw.source.import_root,
            records: Records::validate(raw.files, raw.licenses)?,
        })
    }

    pub(super) fn import_root(&self) -> &RelativePath {
        &self.import_root
    }

    pub(super) fn entries(&self) -> impl Iterator<Item = (&RelativePath, &Sha256Digest)> {
        self.records.entries()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct Receipt {
    schema_version: u8,
    generator: String,
    verification: String,
    source: DeclaredSource,
    manifest_sha256: Sha256Digest,
    #[serde(flatten)]
    records: Records,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReceipt {
    schema_version: u8,
    generator: String,
    verification: String,
    source: RawSource,
    manifest_sha256: Sha256Digest,
    files: Vec<RawFile>,
    licenses: Vec<RawLicense>,
}

impl Receipt {
    pub(super) fn new(manifest: &Manifest, manifest_bytes: &[u8]) -> Self {
        Self {
            schema_version: 1,
            generator: GENERATOR.to_owned(),
            verification: VERIFICATION.to_owned(),
            source: manifest.source.clone(),
            manifest_sha256: Sha256Digest::from_bytes(manifest_bytes),
            records: manifest.records.clone(),
        }
    }

    pub(super) fn parse(bytes: &[u8]) -> Result<Self> {
        let raw: RawReceipt = serde_json::from_slice(bytes).map_err(|error| {
            GeneratorError::with_source(
                GeneratorErrorKind::InvalidInventory,
                "parse WPT import receipt",
                "invalid receipt JSON",
                error,
            )
        })?;
        if raw.schema_version != 1 || raw.generator != GENERATOR || raw.verification != VERIFICATION
        {
            return Err(super::invalid_inventory(
                "WPT receipt has an unsupported schema or provenance kind",
            ));
        }
        let receipt = Self {
            schema_version: raw.schema_version,
            generator: raw.generator,
            verification: raw.verification,
            source: raw.source.validate().map_err(receipt_validation)?,
            manifest_sha256: raw.manifest_sha256,
            records: Records::validate(raw.files, raw.licenses).map_err(receipt_validation)?,
        };
        if receipt.bytes()? != bytes {
            return Err(super::invalid_inventory(
                "WPT receipt bytes are not canonical",
            ));
        }
        Ok(receipt)
    }

    pub(super) fn bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = serde_json::to_vec(self).map_err(|error| {
            GeneratorError::with_source(
                GeneratorErrorKind::InvalidInventory,
                "serialize WPT import receipt",
                "receipt serialization failed",
                error,
            )
        })?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub(super) fn entries(&self) -> impl Iterator<Item = (&RelativePath, &Sha256Digest)> {
        self.records.entries()
    }
}

fn invalid(detail: impl Into<String>) -> GeneratorError {
    GeneratorError::new(
        GeneratorErrorKind::InvalidManifest,
        "validate WPT corpus manifest",
        detail,
    )
}

fn receipt_validation(source: GeneratorError) -> GeneratorError {
    GeneratorError::with_source(
        GeneratorErrorKind::InvalidInventory,
        "validate WPT import receipt",
        source.to_string(),
        source,
    )
}
