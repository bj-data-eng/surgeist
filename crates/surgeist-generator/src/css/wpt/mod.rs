//! Explicit WPT source inventories, independent of acquisition and adaptation.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::core::{
    ArtifactPlan, ArtifactReservation, CORPUS_FILE_MODE, Domain, GenerationLease, Inventory,
    InventoryPolicy, NamespaceDisjointness, NodeKind, PublicationInventory, PublicationPolicy,
    RootedFs,
};
use crate::{
    CorpusLocation, GeneratorError, GeneratorErrorKind, RelativePath, Result, RunScope,
    Sha256Digest, parse_manifest,
};

use super::CssRequest;
use model::{Manifest, Receipt, SIDECAR};

mod model;

const MANIFEST_FILE: &str = "corpus.toml";

pub(super) fn is_manifest(bytes: &[u8], path: &Path) -> Result<bool> {
    let text = std::str::from_utf8(bytes).map_err(|error| {
        GeneratorError::with_source(
            GeneratorErrorKind::InvalidManifest,
            "read CSS corpus manifest",
            "manifest is not UTF-8",
            error,
        )
    })?;
    let raw: toml::Value = parse_manifest(text, path)?;
    Ok(raw
        .get("source")
        .and_then(|source| source.get("kind"))
        .and_then(toml::Value::as_str)
        == Some("wpt"))
}

pub(super) fn import(request: &CssRequest) -> Result<()> {
    let location = request.location();
    let manifest_path = location.corpus_root().join(MANIFEST_FILE);
    let manifest_bytes = super::importer::read_manifest_file(&manifest_path)?;
    let manifest = Manifest::parse(&manifest_bytes, &manifest_path)?;
    let bundle_path = request
        .source_root()
        .expect("import request has a source root");
    let bundle_location = CorpusLocation::new(bundle_path, bundle_path)?;
    let bundle = RootedFs::open_corpus(&bundle_location)?;
    let mut artifacts = read_bundle(&bundle, &manifest)?;
    let receipt = Receipt::new(&manifest, &manifest_bytes);
    artifacts.push((RelativePath::new(SIDECAR)?, receipt.bytes()?));

    let reservation = ArtifactReservation::new(Domain::Css)?;
    let import_path = manifest.import_root().join(location.corpus_root());
    let external_stage = reservation.external_stage().join(location.corpus_root());
    let namespaces = NamespaceDisjointness::for_mutation(
        location,
        &[
            ("WPT import root", import_path.as_path()),
            ("WPT transaction stage", external_stage.as_path()),
        ],
        &[
            ("WPT corpus manifest", manifest_path.as_path()),
            ("WPT source bundle", bundle_location.corpus_root()),
        ],
    )?;
    let preflight = RootedFs::open_corpus(location)?;
    super::importer::revalidate_manifest(&preflight, &manifest_bytes)?;
    let previous = inspect_import(&preflight, &manifest)?;
    drop(preflight);

    let revalidate = |rooted: &RootedFs| {
        namespaces.revalidate(rooted)?;
        super::importer::revalidate_manifest(rooted, &manifest_bytes)?;
        read_bundle(&bundle, &manifest)?;
        if inspect_import(rooted, &manifest)? != previous {
            return Err(invalid_inventory(
                "WPT import changed after preflight validation",
            ));
        }
        Ok(())
    };
    let lease = GenerationLease::acquire_with_revalidation(
        location,
        Domain::Css,
        "surgeist-css-generate",
        &RunScope::Full,
        "import-wpt",
        revalidate,
    )?;
    let retained = artifacts
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<BTreeSet<_>>();
    let mut classified = previous.classified_paths();
    classified.extend(retained.iter().cloned());
    let inventory = PublicationInventory::new(
        classified.into_iter().collect(),
        retained.into_iter().collect(),
        Vec::new(),
    )?;
    let plan = ArtifactPlan::new(
        location,
        Domain::Css,
        &lease,
        manifest.import_root().clone(),
        PublicationPolicy::CleanFull,
        artifacts,
        inventory,
    )?
    .with_reservation(reservation)?;
    #[cfg(test)]
    return plan.install_with_revalidation_and_inter_scan_hook(revalidate, || {});
    #[cfg(not(test))]
    plan.install_with_revalidation(revalidate)
}

fn read_bundle(rooted: &RootedFs, manifest: &Manifest) -> Result<Vec<(RelativePath, Vec<u8>)>> {
    rooted.revalidate_root()?;
    let mut artifacts = Vec::new();
    for (path, digest) in manifest.entries() {
        let bytes = rooted.read_file(path.as_str(), CORPUS_FILE_MODE)?;
        if &Sha256Digest::from_bytes(&bytes) != digest {
            return Err(GeneratorError::new(
                GeneratorErrorKind::SourceVerification,
                "verify WPT source bundle",
                format!("manifest SHA-256 mismatch: {}", path.as_str()),
            ));
        }
        artifacts.push((path.clone(), bytes));
    }
    rooted.revalidate_root()?;
    Ok(artifacts)
}

#[derive(Eq, PartialEq)]
struct Imported {
    inventory: Option<Inventory>,
    receipt: Option<Receipt>,
}

impl Imported {
    fn classified_paths(&self) -> BTreeSet<RelativePath> {
        match &self.receipt {
            Some(receipt) => receipt
                .entries()
                .map(|(path, _)| path.clone())
                .chain(std::iter::once(
                    RelativePath::new(SIDECAR).expect("fixed receipt path"),
                ))
                .collect(),
            None => BTreeSet::new(),
        }
    }
}

fn inspect_import(rooted: &RootedFs, manifest: &Manifest) -> Result<Imported> {
    let inventory = Inventory::scan(
        rooted,
        manifest.import_root().as_str(),
        InventoryPolicy::FinalCorpus,
    )?;
    let Some(tree) = inventory.as_ref().filter(|tree| !tree.entries().is_empty()) else {
        return Ok(Imported {
            inventory,
            receipt: None,
        });
    };
    let sidecar = tree
        .entries()
        .iter()
        .find(|entry| entry.path().as_str() == SIDECAR)
        .ok_or_else(|| invalid_inventory("nonempty WPT import lacks its ownership receipt"))?;
    let sidecar_path = format!("{}/{SIDECAR}", manifest.import_root().as_str());
    let bytes = rooted.read_file(&sidecar_path, CORPUS_FILE_MODE)?;
    let receipt_digest = Sha256Digest::from_bytes(&bytes);
    if sidecar.digest() != Some(&receipt_digest) {
        return Err(invalid_inventory(
            "WPT receipt changed during inventory inspection",
        ));
    }
    let receipt = Receipt::parse(&bytes)?;
    let mut expected = receipt
        .entries()
        .map(|(path, digest)| (path.as_str(), digest))
        .collect::<BTreeMap<_, _>>();
    expected.insert(SIDECAR, &receipt_digest);
    let mut observed = BTreeSet::new();
    for entry in tree.entries() {
        let path = entry.path().as_str();
        match entry.identity().kind() {
            NodeKind::Regular => {
                let digest = expected
                    .get(path)
                    .ok_or_else(|| invalid_inventory(format!("unowned WPT import file: {path}")))?;
                if entry.digest() != Some(*digest) {
                    return Err(invalid_inventory(format!(
                        "WPT import SHA-256 mismatch: {path}"
                    )));
                }
                observed.insert(path);
            }
            NodeKind::Directory
                if expected
                    .keys()
                    .any(|file| file.starts_with(&format!("{path}/"))) => {}
            _ => {
                return Err(invalid_inventory(format!(
                    "unowned WPT import entry: {path}"
                )));
            }
        }
    }
    for path in expected.keys() {
        if !observed.contains(path) {
            return Err(invalid_inventory(format!(
                "missing WPT import file: {path}"
            )));
        }
    }
    Ok(Imported {
        inventory,
        receipt: Some(receipt),
    })
}

pub(super) fn check(rooted: &RootedFs, manifest_bytes: &[u8], path: &Path) -> Result<()> {
    let manifest = Manifest::parse(manifest_bytes, path)?;
    let imported = inspect_import(rooted, &manifest)?;
    if imported.receipt.as_ref() != Some(&Receipt::new(&manifest, manifest_bytes)) {
        return Err(GeneratorError::new(
            GeneratorErrorKind::Verification,
            "check WPT corpus",
            "WPT import receipt is absent or does not match the current manifest",
        ));
    }
    super::importer::revalidate_manifest(rooted, manifest_bytes)?;
    if inspect_import(rooted, &manifest)? != imported {
        return Err(invalid_inventory(
            "WPT import changed during corpus checking",
        ));
    }
    Ok(())
}

fn invalid_inventory(detail: impl Into<String>) -> GeneratorError {
    GeneratorError::new(
        GeneratorErrorKind::InvalidInventory,
        "inspect WPT import inventory",
        detail,
    )
}
