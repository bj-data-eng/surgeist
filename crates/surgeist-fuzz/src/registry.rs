//! Declarative discovery from Cargo metadata; no product-specific dispatch.
#![forbid(unsafe_code)]

use crate::{FuzzError, FuzzErrorKind, RelativePath, Result, TargetDescriptor, TargetId};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_METADATA: usize = 16 * 1024 * 1024;
const MAX_DESCRIPTOR: u64 = 1024 * 1024;

#[derive(Debug, Deserialize)]
struct Metadata {
    workspace_root: PathBuf,
    workspace_members: Vec<String>,
    packages: Vec<Package>,
}
#[derive(Debug, Deserialize)]
struct Package {
    id: String,
    name: String,
    manifest_path: PathBuf,
    targets: Vec<CargoTarget>,
    features: BTreeMap<String, Vec<String>>,
    metadata: serde_json::Value,
}
#[derive(Debug, Deserialize)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
    #[serde(default, rename = "required-features")]
    required_features: Vec<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registration {
    descriptor: RelativePath,
}

/// One strict descriptor file per owner-local fuzz package.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FuzzManifest {
    pub schema: u32,
    pub targets: Vec<TargetDescriptor>,
    /// Owner-local checked-in seeds, regressions and support fixture paths.
    pub fixtures: Vec<RelativePath>,
}

#[derive(Clone, Debug)]
pub struct RegisteredTarget {
    descriptor: TargetDescriptor,
    package: String,
    package_root: PathBuf,
    manifest_path: PathBuf,
    descriptor_path: PathBuf,
    fixtures: Vec<PathBuf>,
}
impl RegisteredTarget {
    pub fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    pub fn package(&self) -> &str {
        &self.package
    }
    pub fn package_root(&self) -> &Path {
        &self.package_root
    }
    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }
    pub fn descriptor_path(&self) -> &Path {
        &self.descriptor_path
    }
    pub fn fixtures(&self) -> &[PathBuf] {
        &self.fixtures
    }
}

#[derive(Debug)]
pub struct TargetRegistry {
    targets: BTreeMap<TargetId, RegisteredTarget>,
}
impl TargetRegistry {
    /// Uses metadata already obtained by the caller. This operation neither builds
    /// targets nor downloads packages or starts a browser.
    pub fn from_metadata_json(workspace_root: &Path, input: &[u8]) -> Result<Self> {
        if input.len() > MAX_METADATA {
            return Err(error("metadata exceeds size limit"));
        }
        let metadata: Metadata = serde_json::from_slice(input).map_err(|e| {
            FuzzError::with_source(
                FuzzErrorKind::Registry,
                "decode Cargo metadata",
                "invalid metadata",
                e,
            )
        })?;
        let root = canonical(workspace_root)?;
        if canonical(&metadata.workspace_root)? != root {
            return Err(error("metadata workspace root differs from requested root"));
        }
        let members: BTreeSet<_> = metadata.workspace_members.iter().collect();
        let packages: BTreeMap<_, _> = metadata
            .packages
            .iter()
            .filter(|p| members.contains(&p.id))
            .map(|p| (p.name.as_str(), p))
            .collect();
        if packages.len()
            != metadata
                .packages
                .iter()
                .filter(|p| members.contains(&p.id))
                .count()
        {
            return Err(error("duplicate workspace package names"));
        }
        let mut targets = BTreeMap::new();
        for package in packages.values() {
            let Some(config) = package.metadata.get("surgeist-fuzz") else {
                continue;
            };
            let registration: Registration =
                serde_json::from_value(config.clone()).map_err(|e| {
                    FuzzError::with_source(
                        FuzzErrorKind::Registry,
                        "decode registration",
                        "invalid fuzz package metadata",
                        e,
                    )
                })?;
            let owner_name = package
                .name
                .strip_suffix("-fuzz")
                .ok_or_else(|| error("registered package name must be owner-fuzz"))?;
            let owner = packages
                .get(owner_name)
                .ok_or_else(|| error("registered fuzz package has no workspace owner"))?;
            let owner_manifest = canonical(&owner.manifest_path)?;
            let owner_root = owner_manifest
                .parent()
                .ok_or_else(|| error("owner has no package directory"))?;
            if !owner_root.starts_with(&root) {
                return Err(error("owner package escapes workspace"));
            }
            let expected_root = canonical(&owner_root.join("fuzz"))?;
            let manifest_path = canonical(&package.manifest_path)?;
            let package_root = manifest_path
                .parent()
                .ok_or_else(|| error("fuzz package has no directory"))?
                .to_path_buf();
            if package_root != expected_root || !package_root.starts_with(&root) {
                return Err(error(
                    "fuzz package must be directly inside its owner's fuzz directory",
                ));
            }
            let descriptor_path = registration
                .descriptor
                .resolve_existing(&package_root)
                .map_err(|e| error(format!("descriptor path: {e}")))?;
            let mut contents = Vec::new();
            File::open(&descriptor_path)
                .map_err(|e| error(format!("open descriptor: {e}")))?
                .take(MAX_DESCRIPTOR + 1)
                .read_to_end(&mut contents)
                .map_err(|e| error(format!("read descriptor: {e}")))?;
            if contents.len() as u64 > MAX_DESCRIPTOR {
                return Err(error("descriptor exceeds size limit"));
            }
            let text =
                std::str::from_utf8(&contents).map_err(|_| error("descriptor must be UTF-8"))?;
            let declaration: FuzzManifest =
                toml::from_str(text).map_err(|e| error(format!("decode descriptor: {e}")))?;
            if declaration.schema != 1 || declaration.targets.is_empty() {
                return Err(error("unsupported or empty descriptor schema"));
            }
            let mut fixtures = Vec::new();
            let mut fixture_set = BTreeSet::new();
            for path in declaration.fixtures {
                let resolved = path
                    .resolve_existing(&package_root)
                    .map_err(|e| error(format!("fixture path: {e}")))?;
                if !fixture_set.insert(resolved.clone()) {
                    return Err(error("duplicate fixture paths"));
                }
                fixtures.push(resolved);
            }
            for descriptor in declaration.targets {
                if descriptor.owner() != owner_name {
                    return Err(error("descriptor owner does not match package owner"));
                }
                let binary = package
                    .targets
                    .iter()
                    .find(|t| t.name == descriptor.binary() && t.kind.iter().any(|k| k == "bin"))
                    .ok_or_else(|| error("declared binary does not exist"))?;
                if descriptor
                    .requirements()
                    .features()
                    .iter()
                    .any(|f| !package.features.contains_key(f))
                {
                    return Err(error("descriptor requires an unknown package feature"));
                }
                if binary
                    .required_features
                    .iter()
                    .any(|f| !descriptor.requirements().features().contains(f))
                {
                    return Err(error("descriptor omits a binary's required feature"));
                }
                let id = descriptor.id().clone();
                let target = RegisteredTarget {
                    descriptor,
                    package: package.name.clone(),
                    package_root: package_root.clone(),
                    manifest_path: manifest_path.clone(),
                    descriptor_path: descriptor_path.clone(),
                    fixtures: fixtures.clone(),
                };
                if targets.insert(id, target).is_some() {
                    return Err(error("duplicate target identity"));
                }
            }
        }
        Ok(Self { targets })
    }
    pub fn targets(&self) -> impl Iterator<Item = &RegisteredTarget> {
        self.targets.values()
    }
    pub fn get(&self, id: &TargetId) -> Result<&RegisteredTarget> {
        self.targets
            .get(id)
            .ok_or_else(|| error(format!("target {id} is not registered")))
    }
}
fn canonical(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .map_err(|e| error(format!("resolve {}: {e}", path.display())))
}
fn error(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::Registry, "discover targets", detail)
}
