//! Fixed-edition publication and browser-free validation. Snapshot copies are
//! witnesses of the current source owners, never replacement editable sources.

use std::collections::BTreeSet;

use super::*;
use surgeist_generator::CorpusLocation;
use surgeist_generator::browser::{SourceImportProvenance, SourceImportSpec};

const EDITION: &str = "root-context-v4";
use crate::parity_support::capture_authority as authority;
use crate::parity_support::exact_differences::{ExactDifferenceSpec, PublishedEdition};
use crate::parity_support::prepared_range_qualification as prepared_range;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Edition {
    RootContextV4,
    Exact(PublishedEdition),
    Specialized(SpecializedProfile),
    Matching(&'static str),
    GridLanesDiagnostics,
}
impl Edition {
    fn name(self) -> &'static str {
        if self == Self::GridLanesDiagnostics {
            authority::ORIGINAL_DIAGNOSTIC_EDITION
        } else if let Self::Matching(edition) = self {
            edition
        } else if let Self::Specialized(profile) = self {
            profile.edition()
        } else {
            self.published().map_or(EDITION, PublishedEdition::name)
        }
    }
    fn published(self) -> Option<PublishedEdition> {
        match self {
            Self::RootContextV4
            | Self::Specialized(_)
            | Self::Matching(_)
            | Self::GridLanesDiagnostics => None,
            Self::Exact(edition) => Some(edition),
        }
    }
    fn lineage(self) -> SemanticLineage {
        self.published()
            .map_or(SemanticLineage::Focused, SemanticLineage::Published)
    }

    fn terminal_name(self) -> &'static str {
        if self == Self::GridLanesDiagnostics {
            authority::ORIGINAL_DIAGNOSTIC_RECEIPT
        } else {
            authority::TERMINAL
        }
    }
}
const PARENT: &str = "tests/layout/browser_parity/captured";
const ORIGINAL: &str = "tests/layout/browser_parity";

// Artifact identity of the immutable root-context-v4 declaration, not another
// source list or a record-count selector. The canonical inventory may append.
const DECLARATION_PREFIX_LENGTH: usize = 36126;
const DECLARATION_PREFIX_SHA256: &str =
    "3d8fd3be19d7e4b5411133a09816954d174725f25d2f76725fc46594ec156abb";

fn edition_input(current: &[u8]) -> Result<DeclarationInput<'_>> {
    DeclarationInput::sealed_prefix(
        current,
        DECLARATION_PREFIX_LENGTH,
        DECLARATION_PREFIX_SHA256,
    )
}

fn publication_request(mode: QualificationMode) -> Result<(Edition, InputAccess)> {
    if let Some(edition) = mode.matching_edition() {
        authority::validate_matching_edition(edition)?;
    }
    Ok(match mode {
        QualificationMode::Publish => (Edition::RootContextV4, InputAccess::Prepare),
        QualificationMode::CheckPublished => (Edition::RootContextV4, InputAccess::ReadOnly),
        QualificationMode::PublishExact(edition) => (Edition::Exact(edition), InputAccess::Prepare),
        QualificationMode::CheckPublishedExact(edition)
        | QualificationMode::DeriveExact(edition) => {
            (Edition::Exact(edition), InputAccess::ReadOnly)
        }
        QualificationMode::PublishSpecialized(profile) => {
            (Edition::Specialized(profile), InputAccess::Prepare)
        }
        QualificationMode::CheckPublishedSpecialized(profile)
        | QualificationMode::DeriveSpecialized(profile) => {
            (Edition::Specialized(profile), InputAccess::ReadOnly)
        }
        QualificationMode::PublishMatching(edition) => {
            (Edition::Matching(edition), InputAccess::Prepare)
        }
        QualificationMode::CheckPublishedMatching(edition)
        | QualificationMode::DeriveMatching(edition) => {
            (Edition::Matching(edition), InputAccess::ReadOnly)
        }
        QualificationMode::PublishGridLanesDiagnostics => {
            (Edition::GridLanesDiagnostics, InputAccess::Prepare)
        }
        QualificationMode::CheckGridLanesDiagnostics => {
            (Edition::GridLanesDiagnostics, InputAccess::ReadOnly)
        }
        _ => return Err("expected a durable publication mode".into()),
    })
}

pub(super) fn run(owner: &Path, mode: QualificationMode) -> Result<()> {
    run_with_output(owner, mode, &mut std::io::stdout())
}

fn run_with_output(owner: &Path, mode: QualificationMode, output: &mut dyn Write) -> Result<()> {
    require_current_inputs(owner)?;
    let (edition, access) = publication_request(mode)?;
    let parent = edition_parent(owner)?;
    let retained = if edition != Edition::RootContextV4 && matches!(access, InputAccess::ReadOnly) {
        let root = parent.join(edition.name());
        require_directory(&root)?;
        require_regular_input(&root, &RelativePath::new("corpus.toml")?)?;
        let declaration = fs::read(root.join("corpus.toml"))?;
        Some((root, declaration))
    } else {
        None
    };
    let input = match edition {
        Edition::GridLanesDiagnostics => {
            let mut input =
                DeclarationInput::parse(retained.as_ref().map_or(DECLARATION, |(_, bytes)| bytes))?;
            input.declaration.select_original_diagnostics()?;
            input
        }
        Edition::Matching(edition) => matching_input(
            retained.as_ref().map_or(DECLARATION, |(_, bytes)| bytes),
            edition,
        )?,
        Edition::Specialized(profile) => {
            let bytes = retained.as_ref().map_or(DECLARATION, |(_, bytes)| bytes);
            specialized_input(bytes, profile)?
        }
        Edition::RootContextV4 => edition_input(DECLARATION)?,
        _ => match &retained {
            Some((root, declaration)) => canonical_input(
                root,
                declaration,
                edition.published().expect("exact edition"),
                access,
            )?,
            None => canonical_input(
                owner,
                DECLARATION,
                edition.published().expect("exact edition"),
                access,
            )?,
        },
    };
    if retained.is_some() {
        let selected = input
            .declaration
            .records
            .iter()
            .map(|record| {
                Ok((
                    record.source.as_str().to_owned(),
                    serde_json::to_value(record)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        Declaration::parse(DECLARATION)?.check_live_records(&selected)?;
    }
    let declaration = &input.declaration;
    // Verify the original owner's complete import before claiming any selected
    // member. This is read-only and never acquires the pinned source or browser.
    let origin = Origin::read(owner, &declaration.records, edition)?;
    match access {
        InputAccess::Prepare => with_new_edition(&parent, edition, |root| {
            CorpusLocation::new(owner, root)?;
            finish(
                owner,
                root,
                &input,
                &origin,
                access,
                edition,
                PublicationInvocation { mode, output },
            )
        }),
        InputAccess::ReadOnly => {
            let root = parent.join(edition.name());
            require_directory(&root)?;
            CorpusLocation::new(owner, &root)?;
            finish(
                owner,
                &root,
                &input,
                &origin,
                access,
                edition,
                PublicationInvocation { mode, output },
            )
        }
    }
}

fn matching_input<'a>(bytes: &'a [u8], edition: &str) -> Result<DeclarationInput<'a>> {
    authority::validate_matching_edition(edition)?;
    let mut input = DeclarationInput::parse(bytes)?;
    input
        .declaration
        .records
        .retain(|record| record.matching_edition() == Some(edition));
    if input.declaration.records.is_empty() {
        return Err("unknown matching publication edition".into());
    }
    Ok(input)
}

fn specialized_input(bytes: &[u8], profile: SpecializedProfile) -> Result<DeclarationInput<'_>> {
    let mut input = DeclarationInput::parse(bytes)?;
    input
        .declaration
        .records
        .retain(|record| record.specialized_publication == Some(profile));
    if input.declaration.records.len() != profile.sources().len()
        || input
            .declaration
            .records
            .iter()
            .any(|record| !profile.sources().contains(&record.source.as_str()))
    {
        return Err("specialized publication requires its canonical declaration".into());
    }
    Ok(input)
}

pub(super) fn prepared_range_policy(owner: &Path) -> Result<Vec<u8>> {
    let repository = owner
        .parent()
        .and_then(Path::parent)
        .ok_or("Layout owner has no repository root")?;
    let path = RelativePath::new(prepared_range::POLICY_SOURCE)?;
    require_regular_input(repository, &path)?;
    let contract = fs::read_to_string(repository.join(path.as_str()))?;
    Ok(prepared_range::PolicyIdentity::verified_section(&contract)?
        .as_bytes()
        .to_vec())
}

fn with_new_edition(
    parent: &Path,
    edition: Edition,
    action: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    let root = create_edition(parent, edition)?;
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

pub(super) fn verified_exact_predecessors(
    owner: &Path,
    edition: PublishedEdition,
) -> Result<BTreeMap<String, String>> {
    let root = owner.join(edition.artifact_root());
    let path = RelativePath::new(authority::TERMINAL)?;
    require_regular_input(&root, &path)?;
    let terminal = fs::read(root.join(path.as_str()))?;
    run(owner, QualificationMode::CheckPublishedExact(edition))?;
    if fs::read(root.join(path.as_str()))? != terminal {
        return Err(format!("{} terminal changed during replay", edition.name()).into());
    }
    let terminal = authority::Qualification::parse(std::str::from_utf8(&terminal)?)?;
    Ok(terminal
        .cases
        .into_iter()
        .map(|case| {
            let output = &case.descriptor().output;
            (output.path.clone(), output.sha256.clone())
        })
        .collect())
}

struct PublicationInvocation<'a> {
    mode: QualificationMode,
    output: &'a mut dyn Write,
}

fn finish(
    owner: &Path,
    root: &Path,
    input: &DeclarationInput<'_>,
    origin: &Origin,
    access: InputAccess,
    edition: Edition,
    invocation: PublicationInvocation<'_>,
) -> Result<()> {
    let PublicationInvocation { mode, output } = invocation;
    require_positioned_original_inputs(owner, edition)?;
    if edition == Edition::Specialized(SpecializedProfile::OriginalSubgridBody) {
        require_original_subgrid_inputs(owner)?;
    }
    if edition == Edition::Specialized(SpecializedProfile::PreparedAtomicPercentage) {
        require_prepared_atomic_inputs(owner)?;
    }
    if edition == Edition::Specialized(SpecializedProfile::HorizontalBrFontBand) {
        require_horizontal_br_inputs(owner)?;
    }
    if edition == Edition::Specialized(SpecializedProfile::VerticalBrFontBand) {
        require_vertical_br_inputs(owner)?;
    }
    if edition == Edition::Specialized(SpecializedProfile::EmptyLineBrFontBand) {
        require_empty_line_br_inputs(owner)?;
    }
    let published = edition.published();
    let has_terminal = edition != Edition::RootContextV4;
    let diagnostics = edition == Edition::GridLanesDiagnostics;
    let terminal_name = edition.terminal_name();
    let derive_terminal = mode.derives_terminal();
    let geometry_predecessors = if edition == Edition::RootContextV4 {
        verified_exact_predecessors(owner, PublishedEdition::GridStaticPosition)?
    } else {
        BTreeMap::new()
    };
    if let Some(published) = published
        && root != owner.join(published.artifact_root())
    {
        return Err("canonical edition root differs from fixed artifact identity".into());
    }
    let declaration = &input.declaration;
    let expected = declaration.expected()?;
    let mut snapshot = inventory(root, owner, input, mode, access)?;
    origin.inputs(root, &mut snapshot, access)?;
    let stored_terminal = if has_terminal && matches!(access, InputAccess::ReadOnly) {
        require_regular_input(root, &RelativePath::new(terminal_name)?)?;
        let bytes = fs::read(root.join(terminal_name))?;
        if diagnostics {
            authority::OriginalDiagnosticReceipt::parse(std::str::from_utf8(&bytes)?)?;
        } else if !derive_terminal {
            authority::Qualification::parse(std::str::from_utf8(&bytes)?)?;
        }
        Some(bytes)
    } else {
        None
    };
    let stored_file = stored_terminal
        .as_deref()
        .map(|bytes| (terminal_name, bytes));
    check_input_census(root, &snapshot.expected_inputs, stored_file)?;
    let requirements = if let Some(published) = published {
        prebind_current_cases(root, &snapshot, published)?
    } else {
        BTreeMap::new()
    };
    let mut terminal_cases = Vec::new();
    let mut diagnostic_raw = BTreeMap::new();
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
            browser::generate(request, QualificationAdapter::for_mode(mode))
        }
        InputAccess::ReadOnly => {
            browser::check_corpus(&corpus, &QualificationAdapter::for_mode(mode))
        }
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
            mode,
            filesystem_tampering: false,
            lineage: edition.lineage(),
            requirements: (published.is_some()).then_some(&requirements),
            terminal_cases: (published.is_some()).then_some(&mut terminal_cases),
            diagnostic_raw: diagnostics.then_some(&mut diagnostic_raw),
            geometry_predecessors,
            policy_raw: &mut std::io::sink(),
        },
    )?;
    // Catch source/input changes during the invocation, as well as stale binaries
    // whose include_bytes resources differ from the current handwritten owner.
    require_current_inputs(owner)?;
    require_positioned_original_inputs(owner, edition)?;
    if edition == Edition::Specialized(SpecializedProfile::OriginalSubgridBody) {
        require_original_subgrid_inputs(owner)?;
    }
    if edition == Edition::Specialized(SpecializedProfile::PreparedAtomicPercentage) {
        require_prepared_atomic_inputs(owner)?;
    }
    if edition == Edition::Specialized(SpecializedProfile::HorizontalBrFontBand) {
        require_horizontal_br_inputs(owner)?;
    }
    if edition == Edition::Specialized(SpecializedProfile::VerticalBrFontBand) {
        require_vertical_br_inputs(owner)?;
    }
    if edition == Edition::Specialized(SpecializedProfile::EmptyLineBrFontBand) {
        require_empty_line_br_inputs(owner)?;
    }
    let current = Origin::read(owner, &declaration.records, edition)?;
    current.inputs(
        root,
        &mut Inventory {
            declaration_path: RelativePath::new(input.declaration_path())?,
            mode,
            fixtures: Vec::new(),
            cases: Vec::new(),
            global_inputs: Vec::new(),
            expected_inputs: BTreeMap::new(),
        },
        InputAccess::ReadOnly,
    )?;
    inventory(root, owner, input, mode, InputAccess::ReadOnly)?;
    check_input_census(root, &snapshot.expected_inputs, stored_file)?;
    if has_terminal {
        if matches!(edition, Edition::Matching(_)) {
            terminal_cases = matching_cases(root, input, &snapshot, &corpus, &report)?;
        }
        let paired_diagnostics =
            if edition == Edition::Matching(authority::GRID_LANES_PAIRED_EDITION) {
                Some(verify_matching_pairs(
                    owner,
                    root,
                    &declaration.records,
                    &terminal_cases,
                    &string_inputs(&snapshot.expected_inputs),
                )?)
            } else {
                None
            };
        if let Edition::Specialized(profile) = edition {
            terminal_cases = specialized_cases(root, input, &snapshot, &corpus, &report, profile)?;
        }
        Invocation {
            root: root.to_path_buf(),
        }
        .check_terminal()?;
        if browser::check_corpus(&corpus, &QualificationAdapter::for_mode(mode))?.to_bytes()?
            != report.to_bytes()?
        {
            return Err("checked report changed before terminal authority".into());
        }
        let context = TerminalInput {
            owner,
            root,
            input,
            inventory: &snapshot,
            corpus: &corpus,
            report: &report,
            origin: &current,
            edition,
        };
        let bytes = if diagnostics {
            let cases = original_diagnostic_cases(&context, diagnostic_raw)?;
            terminal_data(context, cases, |case| &case.case, |_| Vec::new())?.to_bytes()?
        } else {
            terminal_authority(context, terminal_cases)?.to_bytes()?
        };
        let publication = match access {
            InputAccess::Prepare => TerminalPublication::Create,
            InputAccess::ReadOnly => {
                let stored = stored_terminal
                    .as_deref()
                    .ok_or("missing retained qualification terminal")?;
                if derive_terminal {
                    TerminalPublication::Derive(stored)
                } else {
                    TerminalPublication::Check(stored)
                }
            }
        };
        if let Some(diagnostics) = paired_diagnostics {
            diagnostics.verify_freshness(owner)?;
        }
        publish_terminal(
            root,
            &snapshot.expected_inputs,
            terminal_name,
            publication,
            &bytes,
            output,
        )?;
    }
    if diagnostics {
        eprintln!(
            "qualify-root-context: {} authenticated original diagnostic cases retained at {}; raw comparisons are in {terminal_name}",
            report.generated().len(),
            root.display()
        );
    } else {
        eprintln!(
            "qualify-root-context: durable edition validated at {}; registration is a separate reviewed change",
            root.display()
        );
    }
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

fn require_positioned_original_inputs(owner: &Path, edition: Edition) -> Result<()> {
    use crate::original_flex_parent_profile as profile;
    let inputs = match edition {
        Edition::Specialized(SpecializedProfile::OriginalFlexParent) => [
            ("original_flex_parent.js", profile::SCRIPT.as_bytes()),
            ("original_flex_parent_request.json", profile::REQUEST),
        ],
        Edition::Specialized(SpecializedProfile::OriginalPercentageFlexParent) => [
            (
                "original_percentage_flex_parent.js",
                profile::percentage::SCRIPT.as_bytes(),
            ),
            (
                "original_percentage_flex_parent_request.json",
                profile::percentage::REQUEST,
            ),
        ],
        Edition::Specialized(SpecializedProfile::OriginalBlockAxes) => [
            (
                "original_block_axes.js",
                crate::original_block_axes_profile::SCRIPT.as_bytes(),
            ),
            (
                "original_block_axes_request.json",
                crate::original_block_axes_profile::REQUEST,
            ),
        ],
        Edition::Specialized(SpecializedProfile::OriginalFlexAxes) => [
            (
                "original_flex_axes.js",
                crate::original_flex_axes_profile::SCRIPT.as_bytes(),
            ),
            (
                "original_flex_axes_request.json",
                crate::original_flex_axes_profile::REQUEST,
            ),
        ],
        Edition::Specialized(SpecializedProfile::OriginalGridAxes) => [
            (
                "original_grid_axes.js",
                crate::original_grid_axes_profile::SCRIPT.as_bytes(),
            ),
            (
                "original_grid_axes_request.json",
                crate::original_grid_axes_profile::REQUEST,
            ),
        ],
        _ => return Ok(()),
    };
    for (name, bytes) in inputs {
        let path = RelativePath::new(format!(
            "tests/bin/surgeist-layout-generate/measurement/{name}"
        ))?;
        require_regular_input(owner, &path)?;
        if fs::read(owner.join(path.as_str()))? != bytes {
            return Err(format!("current positioned original input differs: {name}").into());
        }
    }
    Ok(())
}

fn require_original_subgrid_inputs(owner: &Path) -> Result<()> {
    for (name, bytes) in [
        (
            "original_subgrid_body.js",
            original_subgrid_body::PROOF_SCRIPT,
        ),
        (
            "original_subgrid_body_request.json",
            original_subgrid_body::REQUEST,
        ),
    ] {
        let path = RelativePath::new(format!(
            "tests/bin/surgeist-layout-generate/measurement/{name}"
        ))?;
        require_regular_input(owner, &path)?;
        if fs::read(owner.join(path.as_str()))? != bytes {
            return Err(format!("current original subgrid input differs: {name}").into());
        }
    }
    Ok(())
}

fn require_prepared_atomic_inputs(owner: &Path) -> Result<()> {
    use crate::prepared_atomic_percentage_profile as profile;
    for (name, bytes) in [
        ("prepared_atomic_percentage.js", profile::SCRIPT.as_bytes()),
        ("prepared_atomic_percentage_request.json", profile::REQUEST),
    ] {
        let path = RelativePath::new(format!(
            "tests/bin/surgeist-layout-generate/measurement/{name}"
        ))?;
        require_regular_input(owner, &path)?;
        if fs::read(owner.join(path.as_str()))? != bytes {
            return Err(format!("current prepared atomic percentage input differs: {name}").into());
        }
    }
    Ok(())
}

fn require_horizontal_br_inputs(owner: &Path) -> Result<()> {
    use crate::horizontal_br_font_band_profile as profile;
    for (name, bytes) in [
        ("horizontal_br_font_band.js", profile::SCRIPT.as_bytes()),
        ("horizontal_br_font_band_request.json", profile::REQUEST),
    ] {
        let path = RelativePath::new(format!(
            "tests/bin/surgeist-layout-generate/measurement/{name}"
        ))?;
        require_regular_input(owner, &path)?;
        if fs::read(owner.join(path.as_str()))? != bytes {
            return Err(format!("current horizontal break font-band input differs: {name}").into());
        }
    }
    Ok(())
}

fn require_vertical_br_inputs(owner: &Path) -> Result<()> {
    use crate::vertical_br_font_band_profile as profile;
    for (name, bytes) in [
        ("vertical_br_font_band.js", profile::SCRIPT.as_bytes()),
        ("vertical_br_font_band_request.json", profile::REQUEST),
    ] {
        let path = RelativePath::new(format!(
            "tests/bin/surgeist-layout-generate/measurement/{name}"
        ))?;
        require_regular_input(owner, &path)?;
        if fs::read(owner.join(path.as_str()))? != bytes {
            return Err(format!("current vertical break font-band input differs: {name}").into());
        }
    }
    Ok(())
}

fn require_empty_line_br_inputs(owner: &Path) -> Result<()> {
    use crate::empty_line_br_font_band_profile as profile;
    for (name, bytes) in [
        ("empty_line_br_font_band.js", profile::SCRIPT.as_bytes()),
        ("empty_line_br_font_band_request.json", profile::REQUEST),
    ] {
        let path = RelativePath::new(format!(
            "tests/bin/surgeist-layout-generate/measurement/{name}"
        ))?;
        require_regular_input(owner, &path)?;
        if fs::read(owner.join(path.as_str()))? != bytes {
            return Err(format!("current empty-line break input differs: {name}").into());
        }
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

fn create_edition(parent: &Path, edition: Edition) -> Result<PathBuf> {
    require_directory(parent)?;
    let root = parent.join(edition.name());
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

fn check_input_census(
    root: &Path,
    expected: &BTreeMap<RelativePath, Sha256Digest>,
    terminal: Option<(&str, &[u8])>,
) -> Result<()> {
    fn visit(
        root: &Path,
        directory: &Path,
        expected: &BTreeMap<RelativePath, Sha256Digest>,
        seen: &mut BTreeSet<RelativePath>,
        terminal: Option<(&str, &[u8])>,
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
            if directory == root
                && let Some((name, expected)) = terminal
                && relative.as_str() == name
            {
                if !kind.is_file() || fs::read(&path)? != expected {
                    return Err("non-regular or changed terminal qualification output".into());
                }
                continue;
            }
            if kind.is_dir() {
                let prefix = format!("{}/", relative.as_str());
                if !expected.keys().any(|key| key.as_str().starts_with(&prefix)) {
                    return Err(format!(
                        "undeclared publication input directory: {}",
                        relative.as_str()
                    )
                    .into());
                }
                visit(root, &path, expected, seen, terminal)?;
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
    if let Some((name, _)) = terminal {
        require_regular_input(root, &RelativePath::new(name)?)?;
    }
    let mut seen = BTreeSet::new();
    visit(root, root, expected, &mut seen, terminal)?;
    if seen.len() != expected.len() {
        return Err("publication input census is incomplete".into());
    }
    Ok(())
}

struct Origin {
    receipt: Vec<u8>,
    provenance: Vec<u8>,
    selected: Vec<u8>,
    font_provenance: Option<Vec<u8>>,
    receipt_owner: RelativePath,
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
    fn read(owner: &Path, records: &[Record], edition: Edition) -> Result<Self> {
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
        let font_provenance = if matches!(
            edition,
            Edition::Specialized(
                SpecializedProfile::EmptyLineBrFontBand
                    | SpecializedProfile::OriginalPercentageFlexParent
                    | SpecializedProfile::OriginalBlockAxes
                    | SpecializedProfile::OriginalFlexAxes
                    | SpecializedProfile::OriginalGridAxes
            )
        ) {
            None
        } else {
            let path = RelativePath::new("fonts/PROVENANCE.md")?;
            require_regular_input(&original, &path)?;
            Some(fs::read(original.join(path.as_str()))?)
        };
        Ok(Self {
            receipt,
            provenance: serde_json::to_vec(&provenance)?,
            selected: serde_json::to_vec(&selected)?,
            font_provenance,
            receipt_owner: RelativePath::new(format!(
                "{ORIGINAL}/{}",
                provenance.sidecar().as_str()
            ))?,
        })
    }

    fn inputs(&self, root: &Path, inventory: &mut Inventory, access: InputAccess) -> Result<()> {
        for (name, bytes) in [
            ("origin/source-receipt.json", self.receipt.as_slice()),
            ("origin/source-provenance.json", self.provenance.as_slice()),
            ("origin/selected-originals.json", self.selected.as_slice()),
        ] {
            let path = RelativePath::new(name)?;
            inventory_input(root, &path, bytes, &mut inventory.expected_inputs, access)?;
            inventory.global_inputs.push(path);
        }
        if let Some(bytes) = &self.font_provenance {
            let path = RelativePath::new("fonts/PROVENANCE.md")?;
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

fn canonical_input<'a>(
    spec_root: &Path,
    current: &'a [u8],
    edition: PublishedEdition,
    access: InputAccess,
) -> Result<DeclarationInput<'a>> {
    let mut input = DeclarationInput::parse(current)?;
    // Fresh publication reads the owning specs. A retained edition contains
    // only its own specs, so replay never consults a different cohort's files.
    let variants = fixture_cases()
        .into_iter()
        .map(|(variant, _)| variant.to_owned())
        .collect::<BTreeSet<_>>();
    let mut selected_records = Vec::new();
    for record in input.declaration.records.drain(..) {
        let Some(references) = &record.semantic_specifications else {
            continue;
        };
        let mut specifications = Vec::new();
        for (variant, reference) in references {
            let path = spec_root.join(reference.path.as_str());
            if matches!(access, InputAccess::ReadOnly)
                && fs::symlink_metadata(&path)
                    .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
            {
                continue;
            }
            require_regular_input(spec_root, &reference.path)?;
            let bytes = fs::read(path)?;
            if Sha256Digest::from_bytes(&bytes) != reference.sha256 {
                return Err("registered Published specification changed".into());
            }
            specifications.push((
                variant,
                ExactDifferenceSpec::parse(std::str::from_utf8(&bytes)?)?,
            ));
        }
        let Some((_, first)) = specifications.first() else {
            continue;
        };
        let association = first
            .published_edition()
            .ok_or("registered specification requires a Published edition")?;
        if specifications.len() != references.len()
            || specifications
                .iter()
                .any(|(_, spec)| spec.published_edition() != Some(association))
        {
            return Err(
                "source variants must retain one complete Published edition association".into(),
            );
        }
        if association != edition {
            if matches!(access, InputAccess::ReadOnly) {
                return Err("retained semantic inputs belong to another edition".into());
            }
            continue;
        }
        if !matches!(
            &record.expected,
            ExpectedDisposition::Generated {
                metadata: ExpectedMetadata::AbsoluteLocal { .. }
            }
        ) || !matches!(record.source_kind, SourceKind::Original)
            || references.keys().cloned().collect::<BTreeSet<_>>() != variants
        {
            return Err("canonical edition requires complete Generated original variants with absolute-local authority".into());
        }
        let (family, stem) = record.source_names()?;
        let record_sha256 = semantic_record_digest(&record)?;
        for (variant, spec) in specifications {
            let selected = authority::CaseSelection {
                name: format!("{stem}__{variant}"),
                source: record.source.as_str().to_owned(),
                source_sha256: record.source_sha256.as_str().to_owned(),
                variant: variant.clone(),
                output: format!("xml/{family}__{variant}.xml"),
                record_sha256: record_sha256.as_str().to_owned(),
                historical_sha256: Some(
                    record
                        .historical_sha256
                        .get(variant)
                        .ok_or("missing current historical variant")?
                        .as_str()
                        .to_owned(),
                ),
            };
            spec.prebind_current(edition, &selected)?;
        }
        selected_records.push(record);
    }
    if selected_records.is_empty() {
        return Err("canonical semantic edition has no selected records".into());
    }
    input.declaration.records = selected_records;
    Ok(input)
}

enum TerminalPublication<'a> {
    Create,
    Check(&'a [u8]),
    Derive(&'a [u8]),
}

/// Emit a fully validated candidate without changing the retained terminal or
/// admitting an additional artifact inside the capture directory.
fn publish_terminal(
    root: &Path,
    expected: &BTreeMap<RelativePath, Sha256Digest>,
    name: &str,
    publication: TerminalPublication<'_>,
    candidate: &[u8],
    output: &mut dyn Write,
) -> Result<()> {
    match publication {
        TerminalPublication::Create => emit_terminal(root, expected, name, candidate),
        TerminalPublication::Check(stored) => {
            if stored != candidate {
                return Err(
                    "terminal qualification differs from complete current derivation".into(),
                );
            }
            check_input_census(root, expected, Some((name, candidate)))
        }
        TerminalPublication::Derive(stored) => {
            check_input_census(root, expected, Some((name, stored)))?;
            output.write_all(candidate)?;
            check_input_census(root, expected, Some((name, stored)))
        }
    }
}

// A failed write or final census must not leave terminal authority behind. The
// exclusive open proves this invocation owns the only path it may remove.
fn emit_terminal(
    root: &Path,
    expected: &BTreeMap<RelativePath, Sha256Digest>,
    name: &str,
    bytes: &[u8],
) -> Result<()> {
    let path = root.join(name);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let result = (|| -> Result<()> {
        file.write_all(bytes)?;
        file.sync_all()?;
        check_input_census(root, expected, Some((name, bytes)))
    })();
    drop(file);
    if let Err(error) = result {
        fs::remove_file(path)
            .map_err(|cleanup| format!("terminal cleanup failed after {error}: {cleanup}"))?;
        return Err(error);
    }
    Ok(())
}

fn prebind_current_cases(
    root: &Path,
    inventory: &Inventory,
    edition: PublishedEdition,
) -> Result<BTreeMap<String, authority::CurrentRequirements>> {
    let mut requirements = BTreeMap::new();
    for case in &inventory.cases {
        let semantic = case
            .semantic
            .as_ref()
            .ok_or("selected current representative has no semantic declaration")?;
        require_regular_input(root, &semantic.reference.path)?;
        let bytes = fs::read(root.join(semantic.reference.path.as_str()))?;
        if Sha256Digest::from_bytes(&bytes) != semantic.reference.sha256 {
            return Err("pre-execution specification bytes changed".into());
        }
        let historical_sha256 = case
            .historical
            .as_ref()
            .map(|p| {
                inventory
                    .expected_inputs
                    .get(p)
                    .map(|s| s.as_str().to_owned())
                    .ok_or("missing prebound historical identity")
            })
            .transpose()?;
        let selected = authority::CaseSelection {
            name: case.case.id().to_owned(),
            source: case.source.as_str().to_owned(),
            variant: case.case.variant().to_owned(),
            output: case.case.output().as_str().to_owned(),
            source_sha256: inventory
                .expected_inputs
                .get(&case.source)
                .ok_or("missing prebound source identity")?
                .as_str()
                .to_owned(),
            record_sha256: semantic.record_sha256.as_str().to_owned(),
            historical_sha256,
        };
        let spec = ExactDifferenceSpec::parse(std::str::from_utf8(&bytes)?)?;
        let binding = spec.prebind_current(edition, &selected)?;
        if requirements.insert(selected.name, binding).is_some() {
            return Err("duplicate prebound current representative".into());
        }
    }
    if requirements.is_empty() || requirements.len() != inventory.cases.len() {
        return Err("incomplete prebound current coverage".into());
    }
    Ok(requirements)
}

fn matching_cases(
    root: &Path,
    input: &DeclarationInput<'_>,
    inventory: &Inventory,
    corpus: &BrowserCorpus,
    report: &browser::BrowserGenerationReport,
) -> Result<Vec<authority::QualifiedCase>> {
    let settings = authority::BrowserSettingsWitness::from_official_json(&serde_json::to_vec(
        corpus.browser(),
    )?)?;
    let retained = string_inputs(&inventory.expected_inputs);
    let report_json: serde_json::Value = serde_json::from_slice(&report.to_bytes()?)?;
    let metadata = serde_json::from_value(report_json["metadata"].clone())?;
    report
        .generated()
        .iter()
        .map(|generated| {
            let artifact: authority::CapturedArtifact =
                serde_json::from_value(serde_json::to_value(generated)?)?;
            let record = input
                .declaration
                .records
                .iter()
                .find(|record| record.source.as_str() == artifact.source)
                .ok_or("undeclared matching source")?;
            let xml = fs::read_to_string(root.join(&artifact.output))?;
            let provenance = record.matching_provenance(&artifact.variant)?;
            let historical = provenance
                .historical()
                .map(|entry| fs::read_to_string(root.join(&entry.path)))
                .transpose()?;
            record.verify_matching_capture(
                &xml,
                historical.as_deref(),
                &artifact.name,
                &artifact.variant,
            )?;
            let case = authority::CaseDescriptor {
                name: artifact.name.clone(),
                source: artifact.source.clone(),
                variant: artifact.variant.clone(),
                output: authority::PathDigest {
                    path: artifact.output.clone(),
                    sha256: artifact.output_sha256.clone(),
                },
                record_sha256: semantic_record_digest(record)?.as_str().to_owned(),
            };
            let category = authority::MatchingCategory::for_edition(
                record
                    .matching_edition()
                    .ok_or("missing matching edition")?,
            );
            let identity =
                authority::capture_binding_from_verified_inputs(authority::CaptureBindingInputs {
                    metadata: &metadata,
                    artifact: &artifact,
                    case: &case,
                    full_inputs: &retained,
                    excluded_specs: &BTreeSet::new(),
                    actual_xml_sha256: &authority::sha256(xml.as_bytes()),
                    historical_sha256: provenance.historical().map(|entry| entry.sha256.as_str()),
                    browser_settings: &settings,
                    observation_request: category.observation_request(),
                    observation_input_sha256: None,
                })?;
            Ok(authority::QualifiedCase::Matching(
                authority::MatchingCase::new(case, provenance, category, identity)?,
            ))
        })
        .collect()
}

fn read_pair_input(root: &Path, path: &str, digest: &str) -> Result<Vec<u8>> {
    require_regular_input(root, &RelativePath::new(path)?)?;
    let bytes = fs::read(root.join(path))?;
    if authority::sha256(&bytes) != digest {
        return Err(format!("source-equivalent pairing input changed: {path}").into());
    }
    Ok(bytes)
}

struct CheckedPairDiagnostics<'a> {
    root: PathBuf,
    reference: &'a authority::DiagnosticReference,
    receipt: authority::OriginalDiagnosticReceipt,
}

impl<'a> CheckedPairDiagnostics<'a> {
    fn check(owner: &Path, reference: &'a authority::DiagnosticReference) -> Result<Self> {
        reference.validate()?;
        let root = edition_parent(owner)?.join(authority::ORIGINAL_DIAGNOSTIC_EDITION);
        require_directory(&root)?;
        let read_receipt = || {
            read_pair_input(
                &root,
                authority::ORIGINAL_DIAGNOSTIC_RECEIPT,
                &reference.sha256,
            )
        };
        let before = read_receipt()?;
        run_with_output(
            owner,
            QualificationMode::CheckGridLanesDiagnostics,
            &mut std::io::sink(),
        )?;
        if read_receipt()? != before {
            return Err("original diagnostic receipt changed during pairing admission".into());
        }
        let receipt = authority::OriginalDiagnosticReceipt::parse(std::str::from_utf8(&before)?)?;
        Ok(Self {
            root,
            reference,
            receipt,
        })
    }

    fn case(
        &self,
        source: &str,
        variant: &str,
    ) -> Result<(Vec<u8>, crate::parity_support::Golden)> {
        let case = self
            .receipt
            .cases
            .iter()
            .find(|case| case.case.source == source && case.case.variant == variant)
            .ok_or("missing original diagnostic pair case")?;
        let hash = self
            .receipt
            .retained_inputs
            .get(source)
            .ok_or("missing original diagnostic source pin")?;
        let html = read_pair_input(&self.root, source, hash)?;
        let xml = read_pair_input(&self.root, &case.case.output.path, &case.case.output.sha256)?;
        Ok((
            html,
            crate::parity_support::Golden::parse(std::str::from_utf8(&xml)?)?,
        ))
    }

    fn verify_freshness(&self, owner: &Path) -> Result<()> {
        let receipt = read_pair_input(
            &self.root,
            authority::ORIGINAL_DIAGNOSTIC_RECEIPT,
            &self.reference.sha256,
        )?;
        let inputs = self
            .receipt
            .retained_inputs
            .iter()
            .map(|(path, digest)| Ok((RelativePath::new(path)?, Sha256Digest::from_text(digest)?)))
            .collect::<Result<_>>()?;
        check_input_census(
            &self.root,
            &inputs,
            Some((authority::ORIGINAL_DIAGNOSTIC_RECEIPT, &receipt)),
        )?;
        for entry in [&self.receipt.report, &self.receipt.ownership]
            .into_iter()
            .chain(self.receipt.cases.iter().map(|case| &case.case.output))
        {
            read_pair_input(&self.root, &entry.path, &entry.sha256)?;
        }
        for (path, digest) in &self.receipt.current_owner_inputs {
            read_pair_input(owner, path, digest)?;
        }
        require_current_inputs(owner)?;
        Ok(())
    }
}

fn verify_matching_pairs<'a>(
    owner: &Path,
    root: &Path,
    records: &'a [Record],
    cases: &[authority::QualifiedCase],
    retained: &BTreeMap<String, String>,
) -> Result<CheckedPairDiagnostics<'a>> {
    if records.len() != authority::GRID_LANES_AXIS_ORIGINAL_SOURCES.len()
        || cases.len() != records.len() * fixture_cases().len()
    {
        return Err("incomplete Grid Lanes source-equivalent matching census".into());
    }
    let first = records[0]
        .source_equivalent_matching()
        .ok_or("missing source-equivalent matching association")?;
    let originals = CheckedPairDiagnostics::check(owner, &first.diagnostics)?;
    authority::verify_grid_lanes_shared_inputs(&originals.receipt.retained_inputs, retained)?;
    for qualified in cases {
        let case = qualified.descriptor();
        let record = records
            .iter()
            .find(|record| record.source.as_str() == case.source)
            .ok_or("missing paired control record")?;
        let pair = record
            .source_equivalent_matching()
            .ok_or("missing paired control association")?;
        if pair.diagnostics != first.diagnostics {
            return Err("paired control diagnostic references differ".into());
        }
        let (original_source, original) = originals.case(&pair.original, &case.variant)?;
        let control_source = read_pair_input(root, &case.source, record.source_sha256.as_str())?;
        let control_xml = read_pair_input(root, &case.output.path, &case.output.sha256)?;
        let control = crate::parity_support::Golden::parse(std::str::from_utf8(&control_xml)?)?;
        crate::parity_support::grid_lanes_native_pair::verify_declared_pair(
            pair,
            &case.variant,
            &original_source,
            &control_source,
            &original,
            &control,
        )
        .map_err(|error| format!("{}: {error}", case.name))?;
    }
    eprintln!(
        "qualify-root-context: {} source-equivalent Grid Lanes Native pairs verified",
        cases.len()
    );
    Ok(originals)
}

fn specialized_cases(
    root: &Path,
    input: &DeclarationInput<'_>,
    inventory: &Inventory,
    corpus: &BrowserCorpus,
    report: &browser::BrowserGenerationReport,
    profile: SpecializedProfile,
) -> Result<Vec<authority::QualifiedCase>> {
    let settings = authority::BrowserSettingsWitness::from_official_json(&serde_json::to_vec(
        corpus.browser(),
    )?)?;
    let retained = string_inputs(&inventory.expected_inputs);
    let report_json: serde_json::Value = serde_json::from_slice(&report.to_bytes()?)?;
    let metadata: authority::CaptureMetadata =
        serde_json::from_value(report_json["metadata"].clone())?;
    report
        .generated()
        .iter()
        .map(|generated| {
            let artifact: authority::CapturedArtifact =
                serde_json::from_value(serde_json::to_value(generated)?)?;
            let record = input
                .declaration
                .records
                .iter()
                .find(|record| record.source.as_str() == artifact.source)
                .ok_or("specialized output has no declared source")?;
            let record_sha256 = semantic_record_digest(record)?.as_str().to_owned();
            let output_sha256 = authority::sha256(&fs::read(root.join(&artifact.output))?);
            let predecessor = format!(
                "historical/{}",
                artifact
                    .output
                    .strip_prefix("xml/")
                    .ok_or("invalid output path")?
            );
            let historical = authority::PathDigest {
                sha256: retained
                    .get(&predecessor)
                    .ok_or("missing historical input")?
                    .clone(),
                path: predecessor,
            };
            let case = authority::CaseDescriptor {
                name: artifact.name.clone(),
                source: artifact.source.clone(),
                variant: artifact.variant.clone(),
                output: authority::PathDigest {
                    path: artifact.output.clone(),
                    sha256: artifact.output_sha256.clone(),
                },
                record_sha256: record_sha256.clone(),
            };
            let evidence = match profile {
                SpecializedProfile::PreparedRange => {
                    authority::SpecializedEvidence::PreparedRange {
                        policy: authority::PathDigest {
                            path: prepared_range::RETAINED_POLICY.to_owned(),
                            sha256: retained
                                .get(prepared_range::RETAINED_POLICY)
                                .ok_or("missing Range policy input")?
                                .clone(),
                        },
                    }
                }
                SpecializedProfile::FlexComposition => {
                    flex_composition_qualification::evidence(&artifact.variant)
                        .ok_or("invalid Flex composition variant")?
                }
                SpecializedProfile::OriginalFlexParent => {
                    authority::original_flex_parent_evidence()
                }
                SpecializedProfile::OriginalPercentageFlexParent => {
                    authority::original_percentage_flex_parent_evidence()
                }
                SpecializedProfile::OriginalBlockAxes => authority::original_block_axes_evidence(),
                SpecializedProfile::OriginalFlexAxes => authority::original_flex_axes_evidence(),
                SpecializedProfile::OriginalGridAxes => authority::original_grid_axes_evidence(),
                SpecializedProfile::OriginalSubgridBody => original_subgrid_body::evidence(),
                SpecializedProfile::PreparedAtomicPercentage => {
                    authority::prepared_atomic_percentage_evidence()
                }
                SpecializedProfile::HorizontalBrFontBand => {
                    crate::parity_support::br_font_band::horizontal_evidence()
                }
                SpecializedProfile::VerticalBrFontBand => {
                    crate::parity_support::br_font_band::vertical_evidence()
                }
                SpecializedProfile::EmptyLineBrFontBand => {
                    crate::parity_support::br_font_band::empty_line_evidence()
                }
            };
            let verified = authority::identity_from_verified_inputs(authority::IdentityInputs {
                metadata: &metadata,
                artifact: &artifact,
                case: &case,
                full_inputs: &retained,
                excluded_specs: &BTreeSet::new(),
                actual_xml_sha256: &output_sha256,
                historical_sha256: Some(&historical.sha256),
                policy_sha256: &evidence.policy_digest(),
                browser_settings: &settings,
                observation_request: profile.observation_request(),
                observation_input_sha256: retained
                    .get(OBSERVATION_REQUEST_INPUT)
                    .map(String::as_str),
            })?;
            Ok(authority::QualifiedCase::Specialized(
                authority::SpecializedCase {
                    case,
                    historical,
                    evidence,
                    identity: verified.projection().clone(),
                },
            ))
        })
        .collect()
}

struct TerminalInput<'a> {
    owner: &'a Path,
    root: &'a Path,
    input: &'a DeclarationInput<'a>,
    inventory: &'a Inventory,
    corpus: &'a BrowserCorpus,
    report: &'a browser::BrowserGenerationReport,
    origin: &'a Origin,
    edition: Edition,
}

fn original_diagnostic_cases(
    context: &TerminalInput<'_>,
    mut raw: BTreeMap<String, Vec<authority::OriginalDiagnosticDifference>>,
) -> Result<Vec<authority::OriginalDiagnosticCase>> {
    let settings = authority::BrowserSettingsWitness::from_official_json(&serde_json::to_vec(
        context.corpus.browser(),
    )?)?;
    let retained = string_inputs(&context.inventory.expected_inputs);
    let report: serde_json::Value = serde_json::from_slice(&context.report.to_bytes()?)?;
    let metadata = serde_json::from_value(report["metadata"].clone())?;
    let mut cases = Vec::new();
    for generated in context.report.generated() {
        let artifact: authority::CapturedArtifact =
            serde_json::from_value(serde_json::to_value(generated)?)?;
        let record = context
            .input
            .declaration
            .records
            .iter()
            .find(|record| record.source.as_str() == artifact.source)
            .ok_or("undeclared original diagnostic source")?;
        let authority::MatchingProvenance::Original { historical } =
            record.matching_provenance(&artifact.variant)?
        else {
            return Err("original diagnostic source has no original history".into());
        };
        let xml = fs::read_to_string(context.root.join(&artifact.output))?;
        record.verify_original_diagnostic_capture(&xml, &artifact.name, &artifact.variant)?;
        let case = authority::CaseDescriptor {
            name: artifact.name.clone(),
            source: artifact.source.clone(),
            variant: artifact.variant.clone(),
            output: authority::PathDigest {
                path: artifact.output.clone(),
                sha256: artifact.output_sha256.clone(),
            },
            record_sha256: semantic_record_digest(record)?.as_str().to_owned(),
        };
        let identity =
            authority::capture_binding_from_verified_inputs(authority::CaptureBindingInputs {
                metadata: &metadata,
                artifact: &artifact,
                case: &case,
                full_inputs: &retained,
                excluded_specs: &BTreeSet::new(),
                actual_xml_sha256: &authority::sha256(xml.as_bytes()),
                historical_sha256: Some(&historical.sha256),
                browser_settings: &settings,
                observation_request: "geometry",
                observation_input_sha256: None,
            })?;
        let raw_differences = raw
            .remove(&case.name)
            .ok_or("missing original diagnostic comparison")?;
        cases.push(authority::OriginalDiagnosticCase {
            case,
            historical,
            identity,
            raw_differences,
        });
    }
    if !raw.is_empty() {
        return Err("unassociated original diagnostic comparisons".into());
    }
    Ok(cases)
}

fn terminal_authority(
    context: TerminalInput<'_>,
    cases: Vec<authority::QualifiedCase>,
) -> Result<authority::Qualification> {
    terminal_data(
        context,
        cases,
        authority::QualifiedCase::descriptor,
        |case| match case {
            authority::QualifiedCase::Exact(case) => vec![&case.spec, &case.policy],
            authority::QualifiedCase::Specialized(case) => case.evidence.bindings(),
            authority::QualifiedCase::Matching(_) => Vec::new(),
        },
    )
}

fn terminal_data<Case>(
    context: TerminalInput<'_>,
    cases: Vec<Case>,
    descriptor: fn(&Case) -> &authority::CaseDescriptor,
    evidence: fn(&Case) -> Vec<&authority::PathDigest>,
) -> Result<authority::QualificationData<Case>> {
    let TerminalInput {
        owner,
        root,
        input,
        inventory,
        corpus,
        report,
        origin,
        edition,
    } = context;
    if cases.len() != inventory.cases.len() || cases.len() != report.generated().len() {
        return Err("terminal current coverage is incomplete".into());
    }
    let actual = cases
        .iter()
        .map(|case| {
            let case = descriptor(case);
            (case.name.as_str(), case.output.path.as_str())
        })
        .collect::<BTreeSet<_>>();
    let expected = inventory
        .cases
        .iter()
        .map(|c| (c.case.id(), c.case.output().as_str()))
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err("terminal current representative census changed".into());
    }
    let records = input
        .declaration
        .records
        .iter()
        .map(|record| {
            Ok(authority::RecordProjection {
                source: record.source.as_str().to_owned(),
                sha256: semantic_record_digest(record)?.as_str().to_owned(),
                value: serde_json::to_value(record)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let semantic_paths = cases
        .iter()
        .flat_map(evidence)
        .map(|binding| binding.path.as_str())
        .collect();
    let current_owner_inputs = current_owner_inputs(
        owner,
        inventory,
        &input.declaration.records,
        &semantic_paths,
        origin,
    )?;
    let report_bytes = report.to_bytes()?;
    if fs::read(root.join("xml/generation-reports/all.json"))? != report_bytes {
        return Err("terminal report bytes changed".into());
    }
    let retained_inputs = string_inputs(&inventory.expected_inputs);
    let capture_validation = authority::CaptureValidation::from_retained(&retained_inputs)?;
    Ok(authority::QualificationData {
        schema_version: if edition == Edition::GridLanesDiagnostics {
            1
        } else {
            authority::TERMINAL_SCHEMA
        },
        edition: edition.name().to_owned(),
        declaration: authority::PathDigest {
            path: "corpus.toml".to_owned(),
            sha256: Sha256Digest::from_bytes(input.bytes()).as_str().to_owned(),
        },
        records,
        browser_settings: authority::BrowserSettingsWitness::from_official_json(
            &serde_json::to_vec(corpus.browser())?,
        )?,
        report: authority::PathDigest {
            path: "xml/generation-reports/all.json".to_owned(),
            sha256: Sha256Digest::from_bytes(&report_bytes).as_str().to_owned(),
        },
        ownership: authority::PathDigest {
            path: "xml/generation-ownership.json".to_owned(),
            sha256: Sha256Digest::from_bytes(&fs::read(
                root.join("xml/generation-ownership.json"),
            )?)
            .as_str()
            .to_owned(),
        },
        cases,
        retained_inputs,
        current_owner_inputs,
        capture_validation,
    })
}

fn current_owner_inputs(
    owner: &Path,
    inventory: &Inventory,
    records: &[Record],
    semantic: &BTreeSet<&str>,
    origin: &Origin,
) -> Result<BTreeMap<String, String>> {
    let receipt_sidecar = origin
        .receipt_owner
        .as_str()
        .strip_prefix(&format!("{ORIGINAL}/"))
        .ok_or("source receipt owner is outside its corpus")?;
    let current = authority::current_owner_bindings(
        &string_inputs(&inventory.expected_inputs),
        &records
            .iter()
            .map(|record| (record.source.as_str(), record.source_kind))
            .collect(),
        semantic,
        receipt_sidecar,
    )?;
    for (source, digest) in &current {
        if source == prepared_range::POLICY_SOURCE {
            if authority::sha256(&prepared_range_policy(owner)?) != *digest {
                return Err("current prepared Range policy differs from retained input".into());
            }
            continue;
        }
        require_regular_input(owner, &RelativePath::new(source)?)?;
        if Sha256Digest::from_bytes(fs::read(owner.join(source))?).as_str() != digest {
            return Err("current semantic owner differs from retained input".into());
        }
    }
    Ok(current)
}

#[cfg(test)]
#[path = "publication_tests.rs"]
mod tests;
