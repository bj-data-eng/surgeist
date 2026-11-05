//! Fixed-edition publication and browser-free validation. Snapshot copies are
//! witnesses of the current source owners, never replacement editable sources.

use std::collections::BTreeSet;

use super::*;
use surgeist_generator::CorpusLocation;
use surgeist_generator::browser::{SourceImportProvenance, SourceImportSpec};

const EDITION: &str = "root-context-v4";
const PARENT: &str = "tests/layout/browser_parity/captured";
const ORIGINAL: &str = "tests/layout/browser_parity";

// Artifact identity of the immutable root-context-v4 declaration, not another
// source list or a record-count selector. The canonical inventory may append.
const DECLARATION_PREFIX_LENGTH: usize = 36126;
const DECLARATION_PREFIX_SHA256: &str =
    "3d8fd3be19d7e4b5411133a09816954d174725f25d2f76725fc46594ec156abb";

fn edition_input(current: &[u8]) -> Result<DeclarationInput<'_>> {
    // Inspect ALL current records before selecting the sealed edition: appended
    // declarations cannot shadow a prior source or duplicate each other.
    DeclarationInput::parse(current)?;
    let bytes = current
        .get(..DECLARATION_PREFIX_LENGTH)
        .ok_or("canonical declaration is shorter than the fixed edition seal")?;
    if Sha256Digest::from_bytes(bytes).as_str() != DECLARATION_PREFIX_SHA256 {
        return Err("canonical declaration prefix differs from the fixed edition seal".into());
    }
    DeclarationInput::parse(bytes)
}

pub(super) fn run(owner: &Path, mode: QualificationMode) -> Result<()> {
    require_current_inputs(owner)?;
    let input = edition_input(DECLARATION)?;
    let declaration = &input.declaration;
    let access = match mode {
        QualificationMode::Publish => InputAccess::Prepare,
        QualificationMode::CheckPublished => InputAccess::ReadOnly,
        _ => return Err("expected a durable publication mode".into()),
    };
    // Verify the original owner's complete import before claiming any selected
    // member. This is read-only and never acquires the pinned source or browser.
    let origin = Origin::read(owner, &declaration.records)?;
    let parent = edition_parent(owner)?;
    match access {
        InputAccess::Prepare => with_new_edition(&parent, |root| {
            CorpusLocation::new(owner, root)?;
            finish(owner, root, &input, &origin, access)
        }),
        InputAccess::ReadOnly => {
            let root = parent.join(EDITION);
            require_directory(&root)?;
            CorpusLocation::new(owner, &root)?;
            finish(owner, &root, &input, &origin, access)
        }
    }
}

fn with_new_edition(parent: &Path, action: impl FnOnce(&Path) -> Result<()>) -> Result<()> {
    let root = create_edition(parent)?;
    let result = action(&root);
    // Retain partial preparation, capture and validation failures. Registration
    // is separate; existence or even a generated report is no approval marker.
    if result.is_err() {
        eprintln!(
            "qualify-root-context: UNREGISTERED edition retained at {}",
            root.display()
        );
    }
    result
}

fn finish(
    owner: &Path,
    root: &Path,
    input: &DeclarationInput<'_>,
    origin: &Origin,
    access: InputAccess,
) -> Result<()> {
    let declaration = &input.declaration;
    let expected = declaration.expected()?;
    let mut snapshot = inventory(root, owner, input, QualificationMode::Regular, access)?;
    origin.inputs(root, &mut snapshot, access)?;
    check_input_census(root, &snapshot.expected_inputs)?;
    let owner_declaration: OwnerBrowserDeclaration = surgeist_generator::parse_manifest(
        std::str::from_utf8(owner_bytes()?)?,
        "browser-owner.toml",
    )?;
    owner_declaration.browser.validate()?;
    let settings = owner_declaration.browser.settings()?;
    let corpus = corpus_for_inventory(root, owner, settings.clone(), &snapshot, false, expected)?;
    let result = match access {
        InputAccess::Prepare => {
            let browser_path =
                browser::acquire_browser(owner, &settings, AcquisitionMode::ExistingOnly)?;
            let request = GenerationRequest::new(corpus.clone(), browser_path, None, None)?;
            browser::generate(
                request,
                QualificationAdapter::for_mode(QualificationMode::Regular),
            )
        }
        InputAccess::ReadOnly => browser::check_corpus(
            &corpus,
            &QualificationAdapter::for_mode(QualificationMode::Regular),
        ),
    };
    // Generation joins its supervisor; check terminal journals even if it failed.
    // This inspection is read-only and never erases unresolved machine state.
    Invocation {
        root: root.to_path_buf(),
    }
    .check_terminal()?;
    let report = result?;
    validate_corpus(
        root,
        &snapshot,
        &corpus,
        expected,
        &report,
        Validation {
            mode: QualificationMode::Regular,
            filesystem_tampering: false,
        },
    )?;
    // Catch source/input changes during the invocation, as well as stale binaries
    // whose include_bytes resources differ from the current handwritten owner.
    require_current_inputs(owner)?;
    let current = Origin::read(owner, &declaration.records)?;
    current.inputs(
        root,
        &mut Inventory {
            fixtures: Vec::new(),
            cases: Vec::new(),
            global_inputs: Vec::new(),
            expected_inputs: BTreeMap::new(),
        },
        InputAccess::ReadOnly,
    )?;
    inventory(
        root,
        owner,
        input,
        QualificationMode::Regular,
        InputAccess::ReadOnly,
    )?;
    check_input_census(root, &snapshot.expected_inputs)?;
    eprintln!(
        "qualify-root-context: durable edition validated at {}; registration is a separate reviewed change",
        root.display()
    );
    Ok(())
}

fn owner_bytes() -> Result<&'static [u8]> {
    INPUTS
        .iter()
        .find_map(|(name, bytes)| (*name == "browser-owner.toml").then_some(*bytes))
        .ok_or_else(|| "missing browser owner input".into())
}

fn require_current_inputs(owner: &Path) -> Result<()> {
    for (name, expected) in INPUTS {
        let path = if *name == "browser-owner.toml" {
            owner.join(ORIGINAL).join("corpus.toml")
        } else {
            owner.join(ORIGINAL).join(name)
        };
        let local = path.strip_prefix(owner.join(ORIGINAL))?;
        require_regular_input(
            &owner.join(ORIGINAL),
            &RelativePath::new(local.to_str().ok_or("non-UTF8 current input")?)?,
        )?;
        if fs::read(&path)?.as_slice() != *expected {
            return Err(format!(
                "current source owner differs from this executable: {}",
                path.display()
            )
            .into());
        }
    }
    require_regular_input(
        owner,
        &RelativePath::new("examples/qualify-root-context/corpus.toml")?,
    )?;
    if fs::read(owner.join("examples/qualify-root-context/corpus.toml"))? != DECLARATION {
        return Err("current qualification declaration differs from this executable".into());
    }
    Ok(())
}

fn require_directory(path: &Path) -> Result<()> {
    if !fs::symlink_metadata(path)?.file_type().is_dir() {
        return Err(format!(
            "expected an actual directory, not a link or other entry: {}",
            path.display()
        )
        .into());
    }
    Ok(())
}

fn edition_parent(owner: &Path) -> Result<PathBuf> {
    let original = owner.join(ORIGINAL);
    // Admission before any creation: the handwritten owner's path must be real
    // and contained. Check every intervening component, not an escaped alias.
    let mut cursor = owner.to_path_buf();
    for component in Path::new(ORIGINAL).components() {
        cursor.push(component.as_os_str());
        require_directory(&cursor)?;
    }
    CorpusLocation::new(owner, &original)?;
    let parent = owner.join(PARENT);
    require_directory(&parent)?;
    CorpusLocation::new(owner, &parent)?;
    Ok(parent)
}

fn create_edition(parent: &Path) -> Result<PathBuf> {
    require_directory(parent)?;
    let root = parent.join(EDITION);
    // This single system operation supplies exclusive name creation. In
    // particular, an entry racing in after parent admission is not overwritten.
    fs::create_dir(&root)?;
    Ok(root)
}

pub(super) fn require_regular_input(root: &Path, path: &RelativePath) -> Result<()> {
    let mut cursor = root.to_path_buf();
    let components = Path::new(path.as_str()).components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        cursor.push(component.as_os_str());
        let kind = fs::symlink_metadata(&cursor)?.file_type();
        if if index + 1 == components.len() {
            !kind.is_file()
        } else {
            !kind.is_dir()
        } {
            return Err(format!(
                "publication input is not a regular owned path: {}",
                cursor.display()
            )
            .into());
        }
    }
    Ok(())
}

fn check_input_census(root: &Path, expected: &BTreeMap<RelativePath, Sha256Digest>) -> Result<()> {
    fn visit(
        root: &Path,
        directory: &Path,
        expected: &BTreeMap<RelativePath, Sha256Digest>,
        seen: &mut BTreeSet<RelativePath>,
    ) -> Result<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path
                .strip_prefix(root)?
                .to_str()
                .ok_or("non-UTF8 publication path")?
                .replace(std::path::MAIN_SEPARATOR, "/");
            // Generator owns and separately checks these two namespaces. They
            // are not unvalidated Layout source inputs or historical witnesses.
            if directory == root && matches!(relative.as_str(), "xml" | ".surgeist-generator") {
                require_directory(&path)?;
                continue;
            }
            let relative = RelativePath::new(relative)?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                let prefix = format!("{}/", relative.as_str());
                if !expected.keys().any(|key| key.as_str().starts_with(&prefix)) {
                    return Err(format!(
                        "undeclared publication input directory: {}",
                        relative.as_str()
                    )
                    .into());
                }
                visit(root, &path, expected, seen)?;
            } else if kind.is_file() {
                if expected.get(&relative) != Some(&Sha256Digest::from_bytes(fs::read(&path)?)) {
                    return Err(format!(
                        "undeclared or changed publication input: {}",
                        relative.as_str()
                    )
                    .into());
                }
                seen.insert(relative);
            } else {
                return Err(format!("non-regular publication input: {}", relative.as_str()).into());
            }
        }
        Ok(())
    }
    let mut seen = BTreeSet::new();
    visit(root, root, expected, &mut seen)?;
    if seen.len() != expected.len() {
        return Err("publication input census is incomplete".into());
    }
    Ok(())
}

struct Origin {
    receipt: Vec<u8>,
    provenance: Vec<u8>,
    selected: Vec<u8>,
    font_provenance: Vec<u8>,
}

#[derive(Serialize)]
struct SelectedOriginal<'a> {
    source: &'a RelativePath,
    ownership: &'static str,
    sha256: &'a Sha256Digest,
}

fn selected_ownership(
    path: &RelativePath,
    digest: &Sha256Digest,
    provenance: &SourceImportProvenance,
) -> Result<&'static str> {
    if let Some(file) = provenance.files().iter().find(|file| file.path() == path) {
        if file.sha256() != digest {
            return Err(format!("selected imported source changed: {}", path.as_str()).into());
        }
        Ok("imported")
    } else if provenance.specification().authored_files().contains(path) {
        Ok("repository-authored")
    } else {
        Err(format!(
            "selected original is not a member of its source owner: {}",
            path.as_str()
        )
        .into())
    }
}

impl Origin {
    fn read(owner: &Path, records: &[Record]) -> Result<Self> {
        let original = owner.join(ORIGINAL);
        let manifest: crate::source_owner::Manifest = surgeist_generator::parse_manifest(
            std::str::from_utf8(owner_bytes()?)?,
            "browser-owner.toml",
        )?;
        manifest.validate()?;
        let specification: SourceImportSpec = manifest.source_import()?;
        let attestation =
            browser::verify_import(&CorpusLocation::new(&original, &original)?, &specification)?;
        // Use the same complete source/case/report declaration as the CLI. This
        // validates the original owner, not its old unqualified XML results.
        let source_corpus = manifest.corpus(
            BrowserLocation::new(owner, &original)?,
            attestation.fixture_paths(),
        )?;
        let provenance = attestation.provenance()?;
        let receipt = attestation.canonical_bytes()?;
        let sidecar = original.join(provenance.sidecar().as_str());
        if fs::read(sidecar)? != receipt {
            return Err("original source receipt is not its canonical verified bytes".into());
        }
        let mut selected = Vec::new();
        for record in records
            .iter()
            .filter(|record| matches!(record.source_kind, SourceKind::Original))
        {
            if !source_corpus
                .fixtures()
                .iter()
                .any(|fixture| fixture.source() == &record.source)
            {
                return Err(
                    "selected original is outside the canonical fixture declaration".into(),
                );
            }
            let local = record
                .source
                .as_str()
                .strip_prefix("html/")
                .ok_or("original source is outside html")?;
            let ownership = selected_ownership(
                &RelativePath::new(local)?,
                &record.source_sha256,
                &provenance,
            )?;
            if Sha256Digest::from_bytes(fs::read(original.join(record.source.as_str()))?)
                != record.source_sha256
            {
                return Err("selected original differs from its declared digest".into());
            }
            selected.push(SelectedOriginal {
                source: &record.source,
                ownership,
                sha256: &record.source_sha256,
            });
        }
        require_regular_input(&original, &RelativePath::new("fonts/PROVENANCE.md")?)?;
        Ok(Self {
            receipt,
            provenance: serde_json::to_vec(&provenance)?,
            selected: serde_json::to_vec(&selected)?,
            font_provenance: fs::read(original.join("fonts/PROVENANCE.md"))?,
        })
    }

    fn inputs(&self, root: &Path, inventory: &mut Inventory, access: InputAccess) -> Result<()> {
        for (name, bytes) in [
            ("origin/source-receipt.json", self.receipt.as_slice()),
            ("origin/source-provenance.json", self.provenance.as_slice()),
            ("origin/selected-originals.json", self.selected.as_slice()),
            ("fonts/PROVENANCE.md", self.font_provenance.as_slice()),
        ] {
            let path = RelativePath::new(name)?;
            inventory_input(root, &path, bytes, &mut inventory.expected_inputs, access)?;
            inventory.global_inputs.push(path);
        }
        // Checked deserialization revalidates the full canonical receipt link;
        // the report's source_imports stays empty, because this is a snapshot of
        // selected sources, not a forged subset of the original import.
        let retained: SourceImportProvenance =
            serde_json::from_slice(&fs::read(root.join("origin/source-provenance.json"))?)?;
        if retained.receipt_sha256()
            != &Sha256Digest::from_bytes(fs::read(root.join("origin/source-receipt.json"))?)
        {
            return Err("retained origin provenance and receipt disagree".into());
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "publication_tests.rs"]
mod tests;
