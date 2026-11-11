//! Invocation-owned corpus and assertions; no historical corpus is rewritten.

use crate::parity_support::exact_differences::PublishedEdition;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use surgeist_generator::browser::{
    self, AcquisitionMode, BrowserCorpus, BrowserCorpusAdapter, BrowserLocation,
    BrowserReportSummary, CaseOutcome, CaseSpec, FixtureInput, FixtureSpec, FixtureStatus,
    GenerationError, GenerationRequest, PreparedFixture, ReportScope,
};
use surgeist_generator::{GeneratorErrorKind, RelativePath, Sha256Digest};

use crate::adapter::{AdapterError, LayoutAdapter, fixture_cases};
use crate::browser_profile::BrowserManifest;
use crate::empty_line_br_font_band_profile::SOURCES as EMPTY_LINE_BR_SOURCES;
use crate::horizontal_br_font_band_profile::SOURCE as HORIZONTAL_BR_SOURCE;
use crate::measurement::{
    BoxObservationRequest, FLEX_COMPOSITION_SOURCE, OBSERVATION_REQUEST_INPUT,
};
use crate::original_flex_parent_profile::{
    SOURCES as ORIGINAL_FLEX_PARENT_SOURCES,
    percentage::SOURCE as ORIGINAL_PERCENTAGE_FLEX_PARENT_SOURCE,
};
use crate::original_subgrid_body_profile::SOURCE as ORIGINAL_SUBGRID_BODY_SOURCE;
use crate::parity_support::capture_authority::{self, SpecializedProfile};
use crate::parity_support::exact_differences::flex_composition_qualification;
use crate::parity_support::{original_subgrid_body, prepared_atomic_percentage};
use crate::prepared_atomic_percentage_profile::SOURCE as PREPARED_ATOMIC_SOURCE;
use crate::vertical_br_font_band_profile::SOURCES as VERTICAL_BR_SOURCES;

#[path = "../../tests/layout/browser_parity/qualification_catalogue.rs"]
mod qualification_catalogue;
use qualification_catalogue::{
    Declaration, ExpectedDisposition, ExpectedMetadata, ExpectedRootSize, PairedObservation,
    Record, SemanticReference, SourceKind,
};

#[path = "publication.rs"]
mod publication;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum QualificationMode {
    Regular,
    StylesheetCustody,
    FontCustody,
    NumericDiagnostics,
    PairedGeometryCapture,
    ElementScrollSizeDiagnostics,
    MeasuredLeafDiagnostics,
    FlexContentTextDiagnostics,
    PreparedRangeCapture,
    FlexCompositionCapture,
    OriginalFlexParentCapture,
    OriginalPercentageFlexParentCapture,
    OriginalSubgridBodyCapture,
    PreparedAtomicPercentageCapture,
    HorizontalBrFontBandCapture,
    VerticalBrFontBandCapture,
    EmptyLineBrFontBandCapture,
    Publish,
    CheckPublished,
    PublishExact(PublishedEdition),
    CheckPublishedExact(PublishedEdition),
    DeriveExact(PublishedEdition),
    DeriveSpecialized(SpecializedProfile),
    DeriveMatching(&'static str),
    PublishSpecialized(SpecializedProfile),
    CheckPublishedSpecialized(SpecializedProfile),
    PublishMatching(&'static str),
    CheckPublishedMatching(&'static str),
    PublishGridLanesDiagnostics,
    CheckGridLanesDiagnostics,
}

impl Serialize for QualificationMode {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let name = match self {
            Self::Regular => "regular",
            Self::StylesheetCustody => "stylesheet-custody",
            Self::FontCustody => "font-custody",
            Self::NumericDiagnostics => "numeric-diagnostics",
            Self::PairedGeometryCapture => "paired-geometry-capture",
            Self::ElementScrollSizeDiagnostics => "element-scroll-size-diagnostics",
            Self::MeasuredLeafDiagnostics => "measured-leaf-diagnostics",
            Self::FlexContentTextDiagnostics => "flex-content-text-diagnostics",
            Self::PreparedRangeCapture => "prepared-range-capture",
            Self::FlexCompositionCapture => "flex-composition-capture",
            Self::OriginalFlexParentCapture => "original-flex-parent-capture",
            Self::OriginalPercentageFlexParentCapture => "original-percentage-flex-parent-capture",
            Self::OriginalSubgridBodyCapture => "original-subgrid-body-capture",
            Self::PreparedAtomicPercentageCapture => "prepared-atomic-percentage-capture",
            Self::HorizontalBrFontBandCapture => "horizontal-br-font-band-capture",
            Self::VerticalBrFontBandCapture => "vertical-br-font-band-capture",
            Self::EmptyLineBrFontBandCapture => "empty-line-br-font-band-capture",
            _ => {
                return Err(serde::ser::Error::custom(
                    "durable mode is not a focused profile",
                ));
            }
        };
        serializer.serialize_str(name)
    }
}

impl QualificationMode {
    fn derives_terminal(self) -> bool {
        matches!(
            self,
            Self::DeriveExact(_) | Self::DeriveSpecialized(_) | Self::DeriveMatching(_)
        )
    }

    fn matching_edition(self) -> Option<&'static str> {
        match self {
            Self::PublishMatching(edition)
            | Self::CheckPublishedMatching(edition)
            | Self::DeriveMatching(edition) => Some(edition),
            _ => None,
        }
    }
    fn collects_numeric_differences(self) -> bool {
        matches!(
            self,
            Self::NumericDiagnostics
                | Self::ElementScrollSizeDiagnostics
                | Self::MeasuredLeafDiagnostics
                | Self::FlexContentTextDiagnostics
        )
    }

    fn specialized_profile(self) -> Option<SpecializedProfile> {
        match self {
            Self::PublishSpecialized(profile)
            | Self::CheckPublishedSpecialized(profile)
            | Self::DeriveSpecialized(profile) => Some(profile),
            _ => None,
        }
    }

    fn box_observations(self) -> BoxObservationRequest {
        match self {
            Self::PreparedRangeCapture
            | Self::PublishSpecialized(SpecializedProfile::PreparedRange)
            | Self::CheckPublishedSpecialized(SpecializedProfile::PreparedRange)
            | Self::DeriveSpecialized(SpecializedProfile::PreparedRange) => {
                BoxObservationRequest::PreparedRanges
            }
            Self::FlexCompositionCapture
            | Self::PublishSpecialized(SpecializedProfile::FlexComposition)
            | Self::CheckPublishedSpecialized(SpecializedProfile::FlexComposition)
            | Self::DeriveSpecialized(SpecializedProfile::FlexComposition) => {
                BoxObservationRequest::FlexComposition
            }
            Self::OriginalFlexParentCapture
            | Self::PublishSpecialized(SpecializedProfile::OriginalFlexParent)
            | Self::CheckPublishedSpecialized(SpecializedProfile::OriginalFlexParent)
            | Self::DeriveSpecialized(SpecializedProfile::OriginalFlexParent) => {
                BoxObservationRequest::OriginalFlexParent
            }
            Self::OriginalPercentageFlexParentCapture
            | Self::PublishSpecialized(SpecializedProfile::OriginalPercentageFlexParent)
            | Self::CheckPublishedSpecialized(SpecializedProfile::OriginalPercentageFlexParent)
            | Self::DeriveSpecialized(SpecializedProfile::OriginalPercentageFlexParent) => {
                BoxObservationRequest::OriginalPercentageFlexParent
            }
            Self::OriginalSubgridBodyCapture
            | Self::PublishSpecialized(SpecializedProfile::OriginalSubgridBody)
            | Self::CheckPublishedSpecialized(SpecializedProfile::OriginalSubgridBody)
            | Self::DeriveSpecialized(SpecializedProfile::OriginalSubgridBody) => {
                BoxObservationRequest::OriginalSubgridBody
            }
            Self::PreparedAtomicPercentageCapture
            | Self::PublishSpecialized(SpecializedProfile::PreparedAtomicPercentage)
            | Self::CheckPublishedSpecialized(SpecializedProfile::PreparedAtomicPercentage)
            | Self::DeriveSpecialized(SpecializedProfile::PreparedAtomicPercentage) => {
                BoxObservationRequest::PreparedAtomicPercentage
            }
            Self::HorizontalBrFontBandCapture
            | Self::PublishSpecialized(SpecializedProfile::HorizontalBrFontBand)
            | Self::CheckPublishedSpecialized(SpecializedProfile::HorizontalBrFontBand)
            | Self::DeriveSpecialized(SpecializedProfile::HorizontalBrFontBand) => {
                BoxObservationRequest::HorizontalBrFontBand
            }
            Self::VerticalBrFontBandCapture
            | Self::PublishSpecialized(SpecializedProfile::VerticalBrFontBand)
            | Self::CheckPublishedSpecialized(SpecializedProfile::VerticalBrFontBand)
            | Self::DeriveSpecialized(SpecializedProfile::VerticalBrFontBand) => {
                BoxObservationRequest::VerticalBrFontBand
            }
            Self::EmptyLineBrFontBandCapture
            | Self::PublishSpecialized(SpecializedProfile::EmptyLineBrFontBand)
            | Self::CheckPublishedSpecialized(SpecializedProfile::EmptyLineBrFontBand)
            | Self::DeriveSpecialized(SpecializedProfile::EmptyLineBrFontBand) => {
                BoxObservationRequest::EmptyLineBrFontBand
            }
            Self::ElementScrollSizeDiagnostics => BoxObservationRequest::ElementScrollSizes,
            Self::FlexContentTextDiagnostics
            | Self::PublishExact(PublishedEdition::FlexText)
            | Self::CheckPublishedExact(PublishedEdition::FlexText)
            | Self::DeriveExact(PublishedEdition::FlexText) => {
                BoxObservationRequest::FlexContentText
            }
            Self::MeasuredLeafDiagnostics
            | Self::PublishExact(PublishedEdition::MeasuredGrid)
            | Self::CheckPublishedExact(PublishedEdition::MeasuredGrid)
            | Self::DeriveExact(PublishedEdition::MeasuredGrid) => {
                BoxObservationRequest::MeasuredLeaf
            }
            _ => BoxObservationRequest::Geometry,
        }
    }
}

fn select_records(
    declaration: &mut Declaration,
    fixture: Option<&str>,
    mode: QualificationMode,
) -> Result<()> {
    let prepared_source = crate::parity_support::prepared_range_qualification::SOURCE;
    if mode == QualificationMode::PreparedRangeCapture {
        if fixture != Some(prepared_source) {
            return Err("prepared Range qualification requires its declared fixture".into());
        }
    } else {
        declaration
            .records
            .retain(|record| record.source.as_str() != prepared_source);
    }
    if mode == QualificationMode::FlexCompositionCapture {
        if fixture != Some(FLEX_COMPOSITION_SOURCE) {
            return Err("Flex composition qualification requires its declared fixture".into());
        }
    } else {
        declaration
            .records
            .retain(|record| record.source.as_str() != FLEX_COMPOSITION_SOURCE);
    }
    if mode == QualificationMode::OriginalFlexParentCapture {
        if !fixture.is_some_and(|source| ORIGINAL_FLEX_PARENT_SOURCES.contains(&source)) {
            return Err("original Flex parent capture requires a declared fixture".into());
        }
    } else {
        declaration
            .records
            .retain(|record| !ORIGINAL_FLEX_PARENT_SOURCES.contains(&record.source.as_str()));
    }
    if mode == QualificationMode::OriginalPercentageFlexParentCapture {
        if fixture != Some(ORIGINAL_PERCENTAGE_FLEX_PARENT_SOURCE) {
            return Err(
                "original percentage Flex parent capture requires its declared fixture".into(),
            );
        }
    } else {
        declaration
            .records
            .retain(|record| record.source.as_str() != ORIGINAL_PERCENTAGE_FLEX_PARENT_SOURCE);
    }
    if mode == QualificationMode::OriginalSubgridBodyCapture {
        if fixture != Some(ORIGINAL_SUBGRID_BODY_SOURCE) {
            return Err("original subgrid body capture requires its declared fixture".into());
        }
    } else {
        declaration
            .records
            .retain(|record| record.source.as_str() != ORIGINAL_SUBGRID_BODY_SOURCE);
    }
    if mode == QualificationMode::PreparedAtomicPercentageCapture {
        if fixture != Some(PREPARED_ATOMIC_SOURCE) {
            return Err("prepared atomic percentage capture requires its declared fixture".into());
        }
    } else {
        declaration
            .records
            .retain(|record| record.source.as_str() != PREPARED_ATOMIC_SOURCE);
    }
    if mode == QualificationMode::HorizontalBrFontBandCapture {
        if fixture != Some(HORIZONTAL_BR_SOURCE) {
            return Err("horizontal break font-band capture requires its declared fixture".into());
        }
    } else {
        declaration
            .records
            .retain(|record| record.source.as_str() != HORIZONTAL_BR_SOURCE);
    }
    if mode == QualificationMode::EmptyLineBrFontBandCapture {
        if !fixture.is_some_and(|source| EMPTY_LINE_BR_SOURCES.contains(&source)) {
            return Err("empty-line BR qualification requires its declared fixture".into());
        }
    } else {
        declaration
            .records
            .retain(|record| !EMPTY_LINE_BR_SOURCES.contains(&record.source.as_str()));
    }
    if mode == QualificationMode::VerticalBrFontBandCapture {
        if !fixture.is_some_and(|source| VERTICAL_BR_SOURCES.contains(&source)) {
            return Err("vertical break font-band capture requires a declared fixture".into());
        }
    } else {
        declaration
            .records
            .retain(|record| !VERTICAL_BR_SOURCES.contains(&record.source.as_str()));
    }
    if matches!(
        mode,
        QualificationMode::PairedGeometryCapture
            | QualificationMode::ElementScrollSizeDiagnostics
            | QualificationMode::MeasuredLeafDiagnostics
            | QualificationMode::FlexContentTextDiagnostics
    ) {
        select_related_records(declaration, fixture)?;
        if mode == QualificationMode::PairedGeometryCapture && declaration.records.len() < 2 {
            return Err("paired geometry capture requires a declared peer closure".into());
        }
        return Ok(());
    }
    match (mode, fixture) {
        (QualificationMode::StylesheetCustody, None) => {
            return Err("stylesheet custody qualification requires one declared fixture".into());
        }
        (QualificationMode::FontCustody, None) => {
            return Err("font custody qualification requires one declared fixture".into());
        }
        (QualificationMode::NumericDiagnostics, None) => {
            return Err("numeric diagnostics require one declared fixture".into());
        }
        _ => {}
    }
    if let Some(source) = fixture {
        declaration
            .records
            .retain(|record| record.source.as_str() == source);
        if declaration.records.len() != 1 {
            return Err("focused qualification requires one declared source".into());
        }
        if declaration.records[0].relation.is_some() {
            return Err("paired observations require the complete inventory".into());
        }
        if !matches!(
            &declaration.records[0].expected,
            ExpectedDisposition::Generated { .. }
        ) {
            match mode {
                QualificationMode::StylesheetCustody => {
                    return Err(
                        "stylesheet custody qualification requires a Generated fixture".into(),
                    );
                }
                QualificationMode::FontCustody => {
                    return Err("font custody qualification requires a Generated fixture".into());
                }
                QualificationMode::NumericDiagnostics => {
                    return Err("numeric diagnostics require a Generated fixture".into());
                }
                QualificationMode::PreparedRangeCapture
                | QualificationMode::FlexCompositionCapture
                | QualificationMode::OriginalFlexParentCapture
                | QualificationMode::OriginalPercentageFlexParentCapture
                | QualificationMode::OriginalSubgridBodyCapture
                | QualificationMode::PreparedAtomicPercentageCapture
                | QualificationMode::HorizontalBrFontBandCapture
                | QualificationMode::VerticalBrFontBandCapture
                | QualificationMode::EmptyLineBrFontBandCapture => {
                    return Err("specialized qualification requires a Generated fixture".into());
                }
                QualificationMode::PairedGeometryCapture
                | QualificationMode::ElementScrollSizeDiagnostics
                | QualificationMode::MeasuredLeafDiagnostics
                | QualificationMode::FlexContentTextDiagnostics => unreachable!("selected above"),
                QualificationMode::Regular => {}
                QualificationMode::Publish
                | QualificationMode::CheckPublished
                | QualificationMode::PublishExact(_)
                | QualificationMode::CheckPublishedExact(_)
                | QualificationMode::DeriveExact(_)
                | QualificationMode::DeriveSpecialized(_)
                | QualificationMode::DeriveMatching(_)
                | QualificationMode::PublishSpecialized(_)
                | QualificationMode::PublishMatching(_)
                | QualificationMode::CheckPublishedMatching(_)
                | QualificationMode::PublishGridLanesDiagnostics
                | QualificationMode::CheckGridLanesDiagnostics
                | QualificationMode::CheckPublishedSpecialized(_) => {
                    return Err("durable modes require the complete inventory".into());
                }
            }
        }
        let named = declared_named_font_counts(&declaration.records[0]);
        match mode {
            QualificationMode::StylesheetCustody if named.is_some() => {
                return Err(
                    "stylesheet custody excludes declared named-font sources; use --font-custody"
                        .into(),
                );
            }
            QualificationMode::FontCustody if named.is_none() => {
                return Err("font custody requires a declared named-font source".into());
            }
            _ => {}
        }
    }
    Ok(())
}

fn select_related_records(declaration: &mut Declaration, fixture: Option<&str>) -> Result<()> {
    let source = fixture.ok_or("related source selection requires one declared fixture")?;
    let initial = declaration
        .records
        .iter()
        .position(|record| record.source.as_str() == source)
        .ok_or("focused qualification requires one declared source")?;
    // Select the declared relation closure in original source order. This mode
    // can qualify the existing phase pair without copying either authored case.
    let mut selected = std::collections::BTreeSet::from([initial]);
    loop {
        let peers: Vec<_> = selected
            .iter()
            .filter_map(|index| {
                declaration.records[*index]
                    .relation
                    .as_ref()
                    .map(|relation| relation.peer.clone())
            })
            .collect();
        let mut changed = false;
        for peer in peers {
            let matches: Vec<_> = declaration
                .records
                .iter()
                .enumerate()
                .filter_map(|(index, record)| {
                    (Path::new(record.source.as_str())
                        .file_stem()
                        .and_then(|value| value.to_str())
                        == Some(peer.as_str()))
                    .then_some(index)
                })
                .collect();
            let [index] = matches.as_slice() else {
                return Err(
                    format!("declared observation peer must identify one source: {peer}").into(),
                );
            };
            changed |= selected.insert(*index);
        }
        if !changed {
            break;
        }
    }
    for index in &selected {
        if !matches!(
            &declaration.records[*index].expected,
            ExpectedDisposition::Generated { .. }
        ) {
            return Err("related source selection requires Generated declarations".into());
        }
    }
    let mut index = 0;
    declaration.records.retain(|_| {
        let keep = selected.contains(&index);
        index += 1;
        keep
    });
    Ok(())
}

fn declared_atomic_paragraphs(record: &Record) -> Option<usize> {
    match &record.expected {
        ExpectedDisposition::Generated {
            metadata:
                ExpectedMetadata::BlockFlow {
                    atomic_paragraphs: Some(count),
                    ..
                },
        } if *count > 0 => Some(*count),
        _ => None,
    }
}

fn declared_named_font_counts(record: &Record) -> Option<(usize, usize)> {
    let paragraphs = declared_atomic_paragraphs(record).unwrap_or(0);
    let grids = match &record.expected {
        ExpectedDisposition::Generated {
            metadata:
                ExpectedMetadata::BlockFlow {
                    grid_reference_owners: Some(owners),
                    ..
                },
        } => owners.len(),
        _ => 0,
    };
    (paragraphs > 0 || grids > 0).then_some((paragraphs, grids))
}

const DECLARATION: &[u8] = include_bytes!("corpus.toml");
pub(super) fn named_derivation_mode(name: &str) -> Option<QualificationMode> {
    if let Ok(edition) = serde_json::from_value(serde_json::Value::String(name.to_owned())) {
        Some(QualificationMode::DeriveExact(edition))
    } else if let Some(profile) = SpecializedProfile::from_edition(name) {
        Some(QualificationMode::DeriveSpecialized(profile))
    } else {
        declared_matching_edition(name).map(QualificationMode::DeriveMatching)
    }
}

pub(super) fn declared_matching_edition(name: &str) -> Option<&'static str> {
    static CATALOGUE: std::sync::OnceLock<Option<Declaration>> = std::sync::OnceLock::new();
    CATALOGUE
        .get_or_init(|| Declaration::parse(DECLARATION).ok())
        .as_ref()?
        .records
        .iter()
        .filter_map(Record::matching_edition)
        .find(|edition| *edition == name)
}
const CUSTODY_HELPER: &[u8] = include_bytes!("custody.js");
const INPUTS: &[(&str, &[u8])] = &[
    (
        "scripts/gentest/test_helper.js",
        include_bytes!("../../tests/layout/browser_parity/scripts/gentest/test_helper.js"),
    ),
    (
        "scripts/gentest/test_base_style.css",
        include_bytes!("../../tests/layout/browser_parity/scripts/gentest/test_base_style.css"),
    ),
    (
        "fonts/Ahem.ttf",
        include_bytes!("../../tests/layout/browser_parity/fonts/Ahem.ttf"),
    ),
    (
        "browser-owner.toml",
        include_bytes!("../../tests/layout/browser_parity/corpus.toml"),
    ),
];

#[derive(Clone)]
struct SemanticCase {
    reference: SemanticReference,
    record_sha256: Sha256Digest,
    published_edition: Option<PublishedEdition>,
}
struct ExpectedCase {
    source: RelativePath,
    case: CaseSpec,
    historical: Option<RelativePath>,
    expected: ExpectedDisposition,
    relation: Option<PairedObservation>,
    semantic: Option<SemanticCase>,
}

struct Inventory {
    declaration_path: RelativePath,
    mode: QualificationMode,
    fixtures: Vec<FixtureSpec>,
    cases: Vec<ExpectedCase>,
    global_inputs: Vec<RelativePath>,
    expected_inputs: BTreeMap<RelativePath, Sha256Digest>,
}

impl Declaration {
    fn expected(&self) -> Result<BrowserReportSummary> {
        if self.records.is_empty() {
            return Err("root-context qualification requires source records".into());
        }
        let mut generated = 0usize;
        let mut unsupported = 0usize;
        for record in &self.records {
            let count = match &record.expected {
                ExpectedDisposition::Generated { .. } => &mut generated,
                ExpectedDisposition::Unsupported { .. } => &mut unsupported,
            };
            *count = count
                .checked_add(fixture_cases().len())
                .ok_or("qualification case count overflow")?;
        }
        Ok(BrowserReportSummary::new(generated, unsupported, 0, 0, 0)?)
    }
}

const FOCUSED_DECLARATION: &str = "focused-declaration.json";

// Retained catalogue bytes and fresh focused identities have separate owners.
struct DeclarationInput<'a> {
    identity: DeclarationIdentity<'a>,
    declaration: Declaration,
}

enum DeclarationIdentity<'a> {
    Catalogue(&'a [u8]),
    Focused(FocusedInput),
}

struct FocusedInput {
    requested_source: RelativePath,
    mode: QualificationMode,
    bytes: Vec<u8>,
}

#[derive(Serialize)]
struct FocusedProjection<'a> {
    schema_version: u8,
    requested_source: &'a RelativePath,
    mode: QualificationMode,
    records: &'a [Record],
}

fn focused_bytes(
    source: &RelativePath,
    mode: QualificationMode,
    records: &[Record],
) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(&FocusedProjection {
        schema_version: 1,
        requested_source: source,
        mode,
        records,
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

impl<'a> DeclarationInput<'a> {
    fn parse(bytes: &'a [u8]) -> Result<Self> {
        let declaration = Declaration::parse(bytes)?;
        declaration.expected()?;
        Ok(Self {
            identity: DeclarationIdentity::Catalogue(bytes),
            declaration,
        })
    }

    fn focused(bytes: &[u8], source: &str, mode: QualificationMode) -> Result<Self> {
        // Validate every authored entry before any selection or normalization.
        let mut declaration = Declaration::parse(bytes)?;
        declaration.expected()?;
        // JSON must preserve every typed value; nonfinite TOML numbers would
        // otherwise silently become null in the derived input.
        for record in &declaration.records {
            if let ExpectedDisposition::Generated {
                metadata:
                    ExpectedMetadata::AbsoluteLocal {
                        inline_offset,
                        block_offset,
                        root_size,
                        ..
                    },
            } = &record.expected
            {
                let size_is_finite = match root_size {
                    None => true,
                    Some(ExpectedRootSize::Fixed { width, height }) => {
                        width.is_finite() && height.is_finite()
                    }
                    Some(ExpectedRootSize::FlexPair {
                        item_width,
                        item_height,
                    }) => item_width.is_finite() && item_height.is_finite(),
                };
                if !inline_offset.is_finite()
                    || block_offset.is_some_and(|value| !value.is_finite())
                    || !size_is_finite
                {
                    return Err(format!(
                        "focused declaration requires finite metadata: {}",
                        record.source.as_str()
                    )
                    .into());
                }
            }
        }
        let mode = effective_mode(mode, Some(source));
        if matches!(
            mode,
            QualificationMode::Publish
                | QualificationMode::CheckPublished
                | QualificationMode::PublishExact(_)
                | QualificationMode::CheckPublishedExact(_)
                | QualificationMode::DeriveExact(_)
                | QualificationMode::DeriveSpecialized(_)
                | QualificationMode::DeriveMatching(_)
                | QualificationMode::PublishSpecialized(_)
                | QualificationMode::CheckPublishedSpecialized(_)
                | QualificationMode::PublishMatching(_)
                | QualificationMode::CheckPublishedMatching(_)
                | QualificationMode::PublishGridLanesDiagnostics
                | QualificationMode::CheckGridLanesDiagnostics
        ) {
            return Err("focused declarations require a focused qualification mode".into());
        }
        select_records(&mut declaration, Some(source), mode)?;
        declaration.expected()?;
        let requested_source = RelativePath::new(source)?;
        let bytes = focused_bytes(&requested_source, mode, &declaration.records)?;
        Ok(Self {
            identity: DeclarationIdentity::Focused(FocusedInput {
                requested_source,
                mode,
                bytes,
            }),
            declaration,
        })
    }

    fn bytes(&self) -> &[u8] {
        match &self.identity {
            DeclarationIdentity::Catalogue(bytes) => bytes,
            DeclarationIdentity::Focused(input) => &input.bytes,
        }
    }

    fn declaration_path(&self) -> &'static str {
        match &self.identity {
            DeclarationIdentity::Catalogue(_) => "corpus.toml",
            DeclarationIdentity::Focused(_) => FOCUSED_DECLARATION,
        }
    }

    fn execution_mode(&self, catalogue_mode: QualificationMode) -> QualificationMode {
        match &self.identity {
            DeclarationIdentity::Catalogue(_) => catalogue_mode,
            DeclarationIdentity::Focused(input) => input.mode,
        }
    }

    fn verify_execution(&self, mode: QualificationMode) -> Result<()> {
        if let DeclarationIdentity::Focused(input) = &self.identity {
            if mode != input.mode {
                return Err("focused declaration mode differs from execution mode".into());
            }
            if input.bytes
                != focused_bytes(
                    &input.requested_source,
                    input.mode,
                    &self.declaration.records,
                )?
            {
                return Err("focused declaration records differ from their identity".into());
            }
        }
        Ok(())
    }

    fn sealed_prefix(current: &'a [u8], length: usize, sha256: &str) -> Result<Self> {
        Self::parse(current)?;
        let bytes = current
            .get(..length)
            .ok_or("canonical declaration is shorter than the fixed edition seal")?;
        if Sha256Digest::from_bytes(bytes).as_str() != sha256 {
            return Err("canonical declaration prefix differs from the fixed edition seal".into());
        }
        Self::parse(bytes)
    }
}

// This intentionally projects only the browser declaration. It does not claim
// to validate the owning corpus's Taffy import or complete fixture inventory.
#[derive(Deserialize)]
struct OwnerBrowserDeclaration {
    browser: BrowserManifest,
}

#[derive(Clone, Copy)]
struct QualificationAdapter {
    observations: BoxObservationRequest,
}

impl QualificationAdapter {
    fn for_mode(mode: QualificationMode) -> Self {
        Self {
            observations: mode.box_observations(),
        }
    }
}
impl BrowserCorpusAdapter for QualificationAdapter {
    type Error = AdapterError;

    fn prepare(
        &self,
        input: FixtureInput<'_>,
    ) -> std::result::Result<PreparedFixture, Self::Error> {
        LayoutAdapter.prepare_with_observations(input, self.observations)
    }
    fn lower(
        &self,
        fixture: &FixtureSpec,
        measurement: serde_json::Value,
    ) -> std::result::Result<Vec<CaseOutcome>, Self::Error> {
        let outcomes =
            LayoutAdapter.lower_with_observations(fixture, measurement, self.observations)?;
        for outcome in &outcomes {
            if let CaseOutcome::Unsupported { case_id, reason } = outcome {
                eprintln!("qualify-root-context: {case_id}: Unsupported: {reason}");
            }
        }
        Ok(outcomes)
    }
}

fn effective_mode(mode: QualificationMode, fixture: Option<&str>) -> QualificationMode {
    match (mode, fixture) {
        (QualificationMode::Regular, Some(FLEX_COMPOSITION_SOURCE)) => {
            QualificationMode::FlexCompositionCapture
        }
        (QualificationMode::Regular, Some(source))
            if ORIGINAL_FLEX_PARENT_SOURCES.contains(&source) =>
        {
            QualificationMode::OriginalFlexParentCapture
        }
        (QualificationMode::Regular, Some(ORIGINAL_PERCENTAGE_FLEX_PARENT_SOURCE)) => {
            QualificationMode::OriginalPercentageFlexParentCapture
        }
        (QualificationMode::Regular, Some(ORIGINAL_SUBGRID_BODY_SOURCE)) => {
            QualificationMode::OriginalSubgridBodyCapture
        }
        (QualificationMode::Regular, Some(PREPARED_ATOMIC_SOURCE)) => {
            QualificationMode::PreparedAtomicPercentageCapture
        }
        (QualificationMode::Regular, Some(HORIZONTAL_BR_SOURCE)) => {
            QualificationMode::HorizontalBrFontBandCapture
        }
        (QualificationMode::Regular, Some(source)) if VERTICAL_BR_SOURCES.contains(&source) => {
            QualificationMode::VerticalBrFontBandCapture
        }
        (QualificationMode::Regular, Some(source)) if EMPTY_LINE_BR_SOURCES.contains(&source) => {
            QualificationMode::EmptyLineBrFontBandCapture
        }
        _ => mode,
    }
}

pub(super) fn run(fixture: Option<&str>, mode: QualificationMode) -> Result<()> {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    if matches!(
        mode,
        QualificationMode::Publish
            | QualificationMode::CheckPublished
            | QualificationMode::PublishExact(_)
            | QualificationMode::CheckPublishedExact(_)
            | QualificationMode::DeriveExact(_)
            | QualificationMode::DeriveSpecialized(_)
            | QualificationMode::DeriveMatching(_)
            | QualificationMode::PublishSpecialized(_)
            | QualificationMode::PublishMatching(_)
            | QualificationMode::CheckPublishedMatching(_)
            | QualificationMode::PublishGridLanesDiagnostics
            | QualificationMode::CheckGridLanesDiagnostics
            | QualificationMode::CheckPublishedSpecialized(_)
    ) {
        if fixture.is_some() {
            return Err("durable modes require the complete inventory".into());
        }
        return publication::run(owner, mode);
    }
    let input = if let Some(source) = fixture {
        DeclarationInput::focused(DECLARATION, source, mode)?
    } else {
        let mut input = DeclarationInput::parse(DECLARATION)?;
        select_records(&mut input.declaration, None, mode)?;
        input
    };
    let mode = input.execution_mode(mode);
    if let Some(source) = fixture {
        eprintln!(
            "qualify-root-context: focused declared source {source}; not a full inventory qualification"
        );
    }
    let _ = input.declaration.expected()?;
    let owner_bytes = INPUTS
        .iter()
        .find(|(name, _)| *name == "browser-owner.toml")
        .expect("browser declaration")
        .1;
    let owner_declaration: OwnerBrowserDeclaration = surgeist_generator::parse_manifest(
        std::str::from_utf8(owner_bytes)?,
        "browser-owner.toml",
    )?;
    owner_declaration.browser.validate()?;
    let settings = owner_declaration.browser.settings()?;
    let browser_path = browser::acquire_browser(owner, &settings, AcquisitionMode::ExistingOnly)?;
    let invocation = Invocation::new(owner)?;
    eprintln!(
        "qualify-root-context: invocation {}",
        invocation.root.display()
    );
    let result = qualify(
        &invocation.root,
        owner,
        settings,
        browser_path,
        input,
        fixture.is_some(),
        mode,
    );
    // Generation owns and joins its supervisor. Never remove outstanding profile
    // journals, even on an assertion/generation failure: preserve them for recovery.
    let terminal = invocation.check_terminal();
    if let Err(error) = terminal {
        eprintln!(
            "qualify-root-context: retained {}: {error}",
            invocation.root.display()
        );
        return Err(error);
    }
    match result {
        Ok(()) => fs::remove_dir_all(&invocation.root).map_err(Into::into),
        Err(error) => {
            eprintln!(
                "qualify-root-context: failed corpus retained at {}",
                invocation.root.display()
            );
            Err(error)
        }
    }
}

struct Invocation {
    root: PathBuf,
}
impl Invocation {
    fn new(owner: &Path) -> Result<Self> {
        let parent = owner.join("tmp/qualify-root-context");
        fs::create_dir_all(&parent)?;
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = parent.join(format!("{}-{nonce}", std::process::id()));
        fs::create_dir(&root)?;
        Ok(Self { root })
    }
    fn check_terminal(&self) -> Result<()> {
        let profiles = self.root.join(".surgeist-generator/profiles/browser");
        match fs::read_dir(&profiles) {
            Ok(mut entries) => {
                if entries.next().transpose()?.is_some() {
                    return Err(format!(
                        "outstanding owned browser profiles at {}",
                        profiles.display()
                    )
                    .into());
                }
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

fn qualify(
    root: &Path,
    owner: &Path,
    settings: browser::BrowserSettings,
    browser_path: RelativePath,
    input: DeclarationInput<'_>,
    focused: bool,
    mode: QualificationMode,
) -> Result<()> {
    let expected = input.declaration.expected()?;
    let inventory = prepare_inventory(root, owner, &input, mode)?;
    let geometry_predecessors = focused_geometry_predecessors(owner, &inventory.cases, mode)?;
    let corpus = corpus_for_inventory(root, owner, settings, &inventory, focused, expected)?;
    let request = GenerationRequest::new(corpus.clone(), browser_path, None, None)?;
    let report = browser::generate(request, QualificationAdapter::for_mode(mode))?;
    validate_corpus(
        root,
        &inventory,
        &corpus,
        expected,
        &report,
        Validation {
            mode,
            filesystem_tampering: true,
            lineage: SemanticLineage::Focused,
            requirements: None,
            terminal_cases: None,
            diagnostic_raw: None,
            geometry_predecessors,
            policy_raw: &mut std::io::stdout(),
        },
    )
}

fn focused_geometry_predecessors(
    owner: &Path,
    cases: &[ExpectedCase],
    mode: QualificationMode,
) -> Result<BTreeMap<String, String>> {
    if mode.box_observations() != BoxObservationRequest::Geometry
        || mode.collects_numeric_differences()
    {
        return Ok(BTreeMap::new());
    }
    let mut selected = BTreeMap::<_, BTreeSet<_>>::new();
    for case in cases {
        if let Some(
            edition @ (PublishedEdition::GridStaticPosition
            | PublishedEdition::RelativeInsets
            | PublishedEdition::WrappingWorldCorners),
        ) = case
            .semantic
            .as_ref()
            .and_then(|semantic| semantic.published_edition)
        {
            selected
                .entry(edition)
                .or_default()
                .insert(case.case.output().as_str());
        }
    }
    let mut predecessors = BTreeMap::new();
    for (edition, outputs) in selected {
        let mut verified = publication::verified_exact_predecessors(owner, edition)?;
        for output in outputs {
            let digest = verified.remove(output).ok_or_else(|| {
                format!(
                    "{} is missing selected qualification for {output}",
                    edition.name()
                )
            })?;
            predecessors.insert(output.to_owned(), digest);
        }
    }
    Ok(predecessors)
}

fn corpus_for_inventory(
    root: &Path,
    owner: &Path,
    settings: browser::BrowserSettings,
    inventory: &Inventory,
    focused: bool,
    expected: BrowserReportSummary,
) -> Result<BrowserCorpus> {
    let mut exact_roots = vec![RelativePath::new("scripts/gentest")?];
    if inventory
        .expected_inputs
        .contains_key(&RelativePath::new("fonts/Ahem.ttf")?)
    {
        exact_roots.push(RelativePath::new("fonts")?);
    }
    if inventory.cases.iter().any(|case| case.historical.is_some()) {
        exact_roots.push(RelativePath::new("historical")?);
    }
    if inventory
        .expected_inputs
        .contains_key(&RelativePath::new("origin/source-receipt.json")?)
    {
        exact_roots.push(RelativePath::new("origin")?);
    }
    Ok(BrowserCorpus::new(
        BrowserLocation::new(owner, root)?,
        inventory.declaration_path.clone(),
        if focused {
            "surgeist-layout-root-context-focused".into()
        } else {
            capture_authority::FIXED_GENERATOR.into()
        },
        RelativePath::new("xml")?,
        vec![ReportScope::new(
            RelativePath::new("generation-reports/all.json")?,
            None,
        )?],
        settings,
        inventory.fixtures.clone(),
        inventory.global_inputs.clone(),
        Vec::new(),
    )?
    .with_expected_inputs(inventory.expected_inputs.clone())?
    .with_exact_input_roots(exact_roots)?
    .with_expected_counts(expected))
}

#[derive(Clone, Copy)]
enum SemanticLineage {
    Focused,
    Published(crate::parity_support::exact_differences::PublishedEdition),
}
struct Validation<'a> {
    mode: QualificationMode,
    filesystem_tampering: bool,
    lineage: SemanticLineage,
    requirements:
        Option<&'a BTreeMap<String, crate::parity_support::capture_authority::CurrentRequirements>>,
    terminal_cases: Option<&'a mut Vec<crate::parity_support::capture_authority::QualifiedCase>>,
    diagnostic_raw:
        Option<&'a mut BTreeMap<String, Vec<capture_authority::OriginalDiagnosticDifference>>>,
    geometry_predecessors: BTreeMap<String, String>,
    policy_raw: &'a mut dyn Write,
}
struct SemanticCapture<'a> {
    root: &'a Path,
    inventory: &'a Inventory,
    corpus: &'a BrowserCorpus,
    report: &'a browser::BrowserGenerationReport,
    bound_report: Option<&'a str>,
    observation: BoxObservationRequest,
    lineage: SemanticLineage,
    requirements:
        Option<&'a BTreeMap<String, crate::parity_support::capture_authority::CurrentRequirements>>,
}
struct SemanticResult {
    verdict: crate::parity_support::exact_differences::QualifiedComparison,
    terminal: Option<crate::parity_support::capture_authority::QualifiedCase>,
}

fn validate_corpus(
    root: &Path,
    inventory: &Inventory,
    corpus: &BrowserCorpus,
    expected: BrowserReportSummary,
    report: &browser::BrowserGenerationReport,
    mut validation: Validation<'_>,
) -> Result<()> {
    use crate::parity_support::prepared_range_qualification as prepared_range;

    let mode = validation.mode;
    if mode != inventory.mode {
        return Err("inventory mode differs from validation mode".into());
    }
    if matches!(
        mode,
        QualificationMode::PublishGridLanesDiagnostics
            | QualificationMode::CheckGridLanesDiagnostics
    ) != validation.diagnostic_raw.is_some()
    {
        return Err("diagnostic comparison output differs from validation mode".into());
    }
    let prepared_range_policy = if mode.box_observations() == BoxObservationRequest::PreparedRanges
    {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(prepared_range::POLICY_SOURCE);
        Some(prepared_range::PolicyIdentity::from_contract(
            &fs::read_to_string(path)?,
        )?)
    } else {
        None
    };
    let flex_composition = if mode.box_observations() == BoxObservationRequest::FlexComposition {
        let input_root = if flex_composition_qualification::input_pins()
            .iter()
            .all(|(path, _)| {
                inventory
                    .expected_inputs
                    .contains_key(&RelativePath::new(*path).expect("policy path"))
            }) {
            root
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR"))
        };
        Some(flex_composition_qualification::Inputs::load(input_root)?)
    } else {
        None
    };
    assert_eq!(*report.summary(), expected);
    assert!(report.filter().is_none());
    assert_eq!(
        report.generated().len(),
        inventory
            .cases
            .iter()
            .filter(|case| matches!(&case.expected, ExpectedDisposition::Generated { .. }))
            .count()
    );
    qualify_dispositions(&report.to_bytes()?, &inventory.cases)?;
    assert_eq!(
        fs::read(root.join("xml/generation-reports/all.json"))?,
        report.to_bytes()?
    );
    assert_eq!(
        browser::check_corpus(corpus, &QualificationAdapter::for_mode(mode))?.to_bytes()?,
        report.to_bytes()?
    );

    // Qualify every retained observation before requiring the new certificate.
    // Historical fields are assertion inputs only; none enter Layout or capture.
    for artifact in report.generated() {
        let case = inventory
            .cases
            .iter()
            .find(|case| case.case.id() == artifact.name())
            .expect("declared generated case");
        assert_eq!(artifact.source(), &case.source);
        assert_eq!(artifact.output(), case.case.output());
        assert_eq!(artifact.variant(), case.case.variant());
        let bytes = fs::read(root.join(artifact.output().as_str()))?;
        assert_eq!(Sha256Digest::from_bytes(&bytes), *artifact.output_sha256());
        if let Some(historical) = &case.historical
            && validation.diagnostic_raw.is_none()
        {
            let historical = fs::read_to_string(root.join(historical.as_str()))?;
            let current = std::str::from_utf8(&bytes)?;
            if matches!(
                mode.box_observations(),
                BoxObservationRequest::OriginalFlexParent
                    | BoxObservationRequest::OriginalPercentageFlexParent
            ) {
                use crate::parity_support::{Golden, original_flex_parent};
                original_flex_parent::verify_historical(
                    &Golden::parse(current)?,
                    &Golden::parse(&historical)?,
                )?;
            } else if mode.box_observations() == BoxObservationRequest::OriginalSubgridBody {
                use crate::parity_support::Golden;
                original_subgrid_body::verify_historical(
                    &Golden::parse(current)?,
                    case.case.variant(),
                    &Golden::parse(&historical)?,
                )?;
            } else if matches!(
                mode.box_observations(),
                BoxObservationRequest::HorizontalBrFontBand
                    | BoxObservationRequest::VerticalBrFontBand
                    | BoxObservationRequest::EmptyLineBrFontBand
            ) {
                use crate::parity_support::{Golden, br_font_band};
                br_font_band::verify_historical(
                    &Golden::parse(current)?,
                    &Golden::parse(&historical)?,
                )?;
            } else if mode.box_observations() != BoxObservationRequest::PreparedAtomicPercentage {
                qualify_historical_geometry(current, &historical, case.case.id())?;
            }
        }
    }
    if validation.diagnostic_raw.is_none() {
        eprintln!(
            "qualify-root-context: all {} source-bound historical expectation trees preserved",
            inventory
                .cases
                .iter()
                .filter(|case| case.historical.is_some())
                .count()
        );
    }
    let mut terminal_cases = Vec::new();
    let observation = mode.box_observations();
    let bound_report = if matches!(
        observation,
        BoxObservationRequest::FlexContentText
            | BoxObservationRequest::FlexComposition
            | BoxObservationRequest::PreparedAtomicPercentage
    ) {
        Some(String::from_utf8(report.to_bytes()?)?)
    } else {
        None
    };
    let capture = SemanticCapture {
        root,
        inventory,
        corpus,
        report,
        bound_report: bound_report.as_deref(),
        observation,
        lineage: validation.lineage,
        requirements: validation.requirements,
    };
    let mut native_errors = Vec::new();
    let mut numeric = NumericDiagnostics::default();
    for artifact in report.generated() {
        let case = inventory
            .cases
            .iter()
            .find(|case| case.case.id() == artifact.name())
            .expect("declared generated case");
        let xml = fs::read_to_string(root.join(artifact.output().as_str()))?;
        // Source certificates and parsing remain fatal in every mode.
        qualify_xml(&xml, case)?;
        let golden = parse_capture_golden(
            root,
            artifact,
            &xml,
            capture.observation,
            capture.bound_report,
            mode.specialized_profile().is_some(),
        )?;
        if golden.has_prepared_range_frame()
            != matches!(
                observation,
                BoxObservationRequest::PreparedRanges
                    | BoxObservationRequest::PreparedAtomicPercentage
            )
        {
            return Err(format!(
                "{}: prepared Range frame differs from the registered request",
                case.case.id()
            )
            .into());
        }
        if golden.has_element_scroll_sizes()
            != matches!(
                mode.box_observations(),
                BoxObservationRequest::ElementScrollSizes
                    | BoxObservationRequest::MeasuredLeaf
                    | BoxObservationRequest::FlexContentText
            )
        {
            return Err(format!(
                "{}: element scroll-size coverage differs from the registered request",
                case.case.id()
            )
            .into());
        }
        if golden.measured_leaf.is_some()
            != (mode.box_observations() == BoxObservationRequest::MeasuredLeaf)
        {
            return Err(format!(
                "{}: measured-leaf coverage differs from the registered request",
                case.case.id()
            )
            .into());
        }
        if let ExpectedDisposition::Generated {
            metadata:
                ExpectedMetadata::BlockFlow {
                    grid_reference_owners: Some(owners),
                    ..
                },
        } = &case.expected
        {
            qualify_grid_reference_census(&golden.root, owners)?;
            crate::parity_support::assert_grid_alignment_reference_inputs(&golden, owners)?;
        }
        if let Some(raw) = &mut validation.diagnostic_raw {
            let differences = capture_authority::original_diagnostic_differences(&golden)?;
            if raw.insert(case.case.id().to_owned(), differences).is_some() {
                return Err("duplicate original diagnostic comparison".into());
            }
        } else if let Some(policy) = &prepared_range_policy {
            if let Err(error) = prepared_range::qualify(
                &golden,
                case.source.as_str(),
                case.case.variant(),
                policy,
                &mut *validation.policy_raw,
            ) {
                native_errors.push(format!("{}: {error}", case.case.id()));
            }
        } else if let Some(inputs) = &flex_composition {
            match flex_composition_qualification::qualify(
                &golden,
                case.case.variant(),
                inputs,
                &mut *validation.policy_raw,
            ) {
                Ok((collapse, scrollbar)) => eprintln!(
                    "qualify-root-context: {} qualified {collapse} collapse and {scrollbar} scrollbar differences",
                    case.case.id()
                ),
                Err(error) => native_errors.push(format!("{}: {error}", case.case.id())),
            }
        } else if observation == BoxObservationRequest::PreparedAtomicPercentage {
            let historical = case
                .historical
                .as_ref()
                .ok_or("missing historical atomic case")?;
            prepared_atomic_percentage::verify_historical(
                &golden,
                &crate::parity_support::Golden::parse_file(root.join(historical.as_str()))?,
            )?;
            let (raw, verdict) = prepared_atomic_percentage::compare(&golden, |_| Ok(()))?;
            for difference in raw {
                eprintln!("qualify-root-context: {}: {difference}", case.case.id());
            }
            if let Err(error) = verdict {
                native_errors.push(format!("{}: {error}", case.case.id()));
            }
        } else if matches!(
            observation,
            BoxObservationRequest::OriginalFlexParent
                | BoxObservationRequest::OriginalPercentageFlexParent
        ) {
            let (raw, verdict) = crate::parity_support::original_flex_parent::compare(&golden)?;
            for difference in raw {
                eprintln!("qualify-root-context: {}: {difference}", case.case.id());
            }
            if let Err(error) = verdict {
                native_errors.push(format!("{}: {error}", case.case.id()));
            }
        } else if observation == BoxObservationRequest::OriginalSubgridBody {
            let (raw, verdict) = original_subgrid_body::compare(&golden, case.case.variant())?;
            for difference in raw {
                eprintln!("qualify-root-context: {}: {difference}", case.case.id());
            }
            if let Err(error) = verdict {
                native_errors.push(format!("{}: {error}", case.case.id()));
            }
        } else if matches!(
            observation,
            BoxObservationRequest::HorizontalBrFontBand
                | BoxObservationRequest::VerticalBrFontBand
                | BoxObservationRequest::EmptyLineBrFontBand
        ) {
            use crate::parity_support::{BrFontBandProfile, br_font_band};
            let profile = match observation {
                BoxObservationRequest::HorizontalBrFontBand => BrFontBandProfile::Horizontal,
                BoxObservationRequest::VerticalBrFontBand => BrFontBandProfile::Vertical,
                BoxObservationRequest::EmptyLineBrFontBand => BrFontBandProfile::EmptyLine,
                _ => unreachable!("selected BR font-band request"),
            };
            if let Err(error) = br_font_band::compare_authenticated(&golden, profile) {
                native_errors.push(format!("{}: {error}", case.case.id()));
            }
        } else if mode.collects_numeric_differences() {
            match collect_numeric_case(&golden, case, &mut std::io::stdout().lock()) {
                Ok(mismatches) => numeric.record_case(mismatches)?,
                Err(error) => {
                    let diagnostic = format!("{}: {error}", case.case.id());
                    eprintln!(
                        "qualify-root-context: fatal numeric comparison failure: {diagnostic}"
                    );
                    native_errors.push(diagnostic);
                }
            }
        } else if let Some(digest) = validation
            .geometry_predecessors
            .remove(artifact.output().as_str())
        {
            if capture_authority::sha256(xml.as_bytes()) != digest {
                return Err(format!(
                    "{}: predecessor geometry differs from qualified capture",
                    case.case.id()
                )
                .into());
            }
        } else if case.semantic.is_some() {
            match qualify_semantic_case(&capture, artifact, case, &golden) {
                Ok(result) => {
                    eprintln!(
                        "qualify-root-context: {} raw={:?}; qualified={:?}",
                        case.case.id(),
                        result.verdict.raw,
                        result.verdict.qualified
                    );
                    if let Some(case) = result.terminal {
                        terminal_cases.push(case);
                    }
                }
                Err(error) => {
                    let diagnostic = format!("{}: {error}", case.case.id());
                    eprintln!(
                        "qualify-root-context: exact semantic qualification refused: {diagnostic}"
                    );
                    native_errors.push(diagnostic);
                }
            }
        } else if let Some(edition) = mode.matching_edition() {
            let (raw, qualified) =
                crate::parity_support::exact_differences::compare_matching_capture(
                    capture_authority::MatchingCategory::for_edition(edition),
                    case.source.as_str(),
                    &golden,
                )?;
            eprintln!(
                "qualify-root-context: {} raw={raw:?}; qualified={qualified:?}",
                case.case.id()
            );
            if let Err(error) = qualified {
                native_errors.push(format!("{}: {error}", case.case.id()));
            }
        } else if let Err(error) = crate::parity_support::assert_surgeist_matches(&golden) {
            let diagnostic = format!("{}: {error}", case.case.id());
            eprintln!("qualify-root-context: Native comparison failed: {diagnostic}");
            native_errors.push(diagnostic);
        }
    }
    if !validation.geometry_predecessors.is_empty() {
        return Err("missing qualified geometry predecessors".into());
    }
    qualify_pairs(root, &inventory.cases)?;
    let tamper_count = if let Some(first) = inventory.cases.iter().find(|case| {
        validation.filesystem_tampering
            && matches!(case.expected, ExpectedDisposition::Generated { .. })
    }) {
        qualify_tampering(
            root,
            corpus,
            &first.source,
            first.case.output(),
            &QualificationAdapter::for_mode(mode),
        )?
    } else {
        0
    };
    assert_eq!(
        browser::check_corpus(corpus, &QualificationAdapter::for_mode(mode))?.to_bytes()?,
        report.to_bytes()?
    );
    if let Some(raw) = validation.diagnostic_raw {
        if raw.len() != inventory.cases.len() || raw.len() != report.generated().len() {
            return Err("incomplete original diagnostic comparisons".into());
        }
        return Ok(());
    }
    if mode.collects_numeric_differences() {
        eprintln!(
            "qualify-root-context: raw numeric diagnostics: {} of {} Generated cases completed, {} mismatches, {} fatal comparison failures; source-bound historical trees and {tamper_count} filesystem tamper refusals qualified; no classification or conformance approval",
            numeric.completed_cases,
            report.summary().generated(),
            numeric.mismatches,
            native_errors.len(),
        );
        return numeric.finish(report.summary().generated(), &native_errors);
    }
    if !native_errors.is_empty() {
        eprintln!(
            "qualify-root-context: {} Native comparison failures across {} Generated cases; declared pairs, source-bound historical trees and {tamper_count} filesystem tamper refusals qualified; Native conformance failed",
            native_errors.len(),
            report.summary().generated(),
        );
        return Err(format!(
            "{} strict Native comparisons failed:\n{}",
            native_errors.len(),
            native_errors.join("\n"),
        )
        .into());
    }
    eprintln!(
        "qualify-root-context: {} Generated and {} Unsupported V4 cases, declared pairs, historical geometry and {tamper_count} filesystem tamper refusals qualified",
        report.summary().generated(),
        report.summary().unsupported()
    );
    match mode {
        QualificationMode::StylesheetCustody => eprintln!(
            "qualify-root-context: focused real-browser stylesheet custody checks completed on a declaration without named-font custody; original fixture, dispositions and strict observations retained"
        ),
        QualificationMode::FontCustody => eprintln!(
            "qualify-root-context: focused real-browser loaded-font custody checks completed; original loaded identity, declarations and strict observations retained"
        ),
        QualificationMode::Regular
        | QualificationMode::NumericDiagnostics
        | QualificationMode::PairedGeometryCapture
        | QualificationMode::ElementScrollSizeDiagnostics
        | QualificationMode::MeasuredLeafDiagnostics
        | QualificationMode::FlexContentTextDiagnostics
        | QualificationMode::PreparedRangeCapture
        | QualificationMode::FlexCompositionCapture
        | QualificationMode::OriginalFlexParentCapture
        | QualificationMode::OriginalPercentageFlexParentCapture
        | QualificationMode::OriginalSubgridBodyCapture
        | QualificationMode::PreparedAtomicPercentageCapture
        | QualificationMode::HorizontalBrFontBandCapture
        | QualificationMode::VerticalBrFontBandCapture
        | QualificationMode::EmptyLineBrFontBandCapture
        | QualificationMode::Publish
        | QualificationMode::CheckPublished
        | QualificationMode::PublishExact(_)
        | QualificationMode::CheckPublishedExact(_)
        | QualificationMode::DeriveExact(_)
        | QualificationMode::DeriveSpecialized(_)
        | QualificationMode::DeriveMatching(_)
        | QualificationMode::PublishSpecialized(_)
        | QualificationMode::PublishMatching(_)
        | QualificationMode::CheckPublishedMatching(_)
        | QualificationMode::PublishGridLanesDiagnostics
        | QualificationMode::CheckGridLanesDiagnostics
        | QualificationMode::CheckPublishedSpecialized(_) => {}
    }
    if let Some(output) = validation.terminal_cases.as_mut() {
        **output = terminal_cases;
    }
    Ok(())
}

#[derive(Serialize)]
struct NumericDiagnostic<'a, T> {
    kind: &'static str,
    case: &'a str,
    source: &'a str,
    variant: &'a str,
    path: &'a str,
    field: &'static str,
    browser: T,
    native: T,
}

fn write_numeric_diagnostic<T: Serialize>(
    kind: &'static str,
    case: &ExpectedCase,
    path: &str,
    field: &'static str,
    browser: T,
    native: T,
    output: &mut impl Write,
) -> Result<()> {
    let diagnostic = NumericDiagnostic {
        kind,
        case: case.case.id(),
        source: case.source.as_str(),
        variant: case.case.variant(),
        path,
        field,
        browser,
        native,
    };
    serde_json::to_writer(&mut *output, &diagnostic)?;
    output.write_all(b"\n")?;
    Ok(())
}

fn collect_numeric_case(
    golden: &crate::parity_support::Golden,
    case: &ExpectedCase,
    output: &mut impl Write,
) -> Result<usize> {
    // The reader returns a complete result only after all category, topology,
    // source and capability checks succeed. Failed cases emit no partial set.
    let mismatches = crate::parity_support::collect_surgeist_numeric_mismatches(golden)?;
    for mismatch in &mismatches {
        match mismatch {
            crate::parity_support::NumericMismatch::Layout(value) => write_numeric_diagnostic(
                "numeric-mismatch",
                case,
                value.path(),
                value.field().as_str(),
                value.browser_value(),
                value.native_value(),
                output,
            )?,
            crate::parity_support::NumericMismatch::ElementScrollSize(value) => {
                write_numeric_diagnostic(
                    "element-scroll-size-mismatch",
                    case,
                    value.path(),
                    value.field().as_str(),
                    value.browser_value(),
                    value.native_value(),
                    output,
                )?
            }
        }
    }
    output.flush()?;
    Ok(mismatches.len())
}

#[derive(Default)]
struct NumericDiagnostics {
    completed_cases: usize,
    mismatches: usize,
}

impl NumericDiagnostics {
    fn record_case(&mut self, mismatches: usize) -> Result<()> {
        self.completed_cases = self
            .completed_cases
            .checked_add(1)
            .ok_or("numeric diagnostic case count overflow")?;
        self.mismatches = self
            .mismatches
            .checked_add(mismatches)
            .ok_or("numeric diagnostic mismatch count overflow")?;
        Ok(())
    }

    fn finish(&self, generated: usize, fatal_errors: &[String]) -> Result<()> {
        if !fatal_errors.is_empty() {
            return Err(format!(
                "numeric diagnostics incomplete: {} of {generated} Generated cases completed, {} numeric mismatches, {} fatal comparison failures:\n{}",
                self.completed_cases, self.mismatches, fatal_errors.len(), fatal_errors.join("\n"),
            ).into());
        }
        if self.completed_cases != generated {
            return Err("numeric diagnostics did not complete every Generated case".into());
        }
        if self.mismatches != 0 {
            return Err(format!(
                "numeric diagnostics found {} raw numeric mismatches across {generated} Generated cases; no classification or conformance approval",
                self.mismatches,
            ).into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum InputAccess {
    Prepare,
    ReadOnly,
}

fn inventory_input(
    root: &Path,
    path: &RelativePath,
    bytes: &[u8],
    expected: &mut BTreeMap<RelativePath, Sha256Digest>,
    access: InputAccess,
) -> Result<()> {
    let destination = root.join(path.as_str());
    match access {
        InputAccess::Prepare => {
            fs::create_dir_all(destination.parent().expect("input parent"))?;
            fs::write(destination, bytes)?;
        }
        InputAccess::ReadOnly => {
            publication::require_regular_input(root, path)?;
            if fs::read(&destination)? != bytes {
                return Err(
                    format!("stale or changed publication input: {}", path.as_str()).into(),
                );
            }
        }
    }
    assert!(
        expected
            .insert(path.clone(), Sha256Digest::from_bytes(bytes))
            .is_none(),
        "duplicate qualification input"
    );
    Ok(())
}

fn prepare_inventory(
    root: &Path,
    owner: &Path,
    input: &DeclarationInput<'_>,
    mode: QualificationMode,
) -> Result<Inventory> {
    inventory(root, owner, input, mode, InputAccess::Prepare)
}

fn inventory(
    root: &Path,
    owner: &Path,
    input: &DeclarationInput<'_>,
    mode: QualificationMode,
    access: InputAccess,
) -> Result<Inventory> {
    inventory_with_retained_helper(root, owner, input, mode, access, None)
}

fn inventory_with_retained_helper(
    root: &Path,
    owner: &Path,
    input: &DeclarationInput<'_>,
    mode: QualificationMode,
    access: InputAccess,
    retained_helper: Option<&[u8]>,
) -> Result<Inventory> {
    input.verify_execution(mode)?;
    let mut inventory = Inventory {
        declaration_path: RelativePath::new(input.declaration_path())?,
        mode,
        fixtures: Vec::new(),
        cases: Vec::new(),
        global_inputs: Vec::new(),
        expected_inputs: BTreeMap::new(),
    };
    let records = &input.declaration.records;
    let custody_installation = match mode {
        QualificationMode::StylesheetCustody => {
            Some("installCustodyProbe('stylesheet', 0, 0);".to_owned())
        }
        QualificationMode::FontCustody => {
            let [record] = records.as_slice() else {
                return Err("font custody requires one selected declaration".into());
            };
            let (paragraphs, grids) = declared_named_font_counts(record)
                .ok_or("font custody requires a declared named-font source")?;
            Some(format!(
                "installCustodyProbe('font', {paragraphs}, {grids});"
            ))
        }
        QualificationMode::Regular
        | QualificationMode::NumericDiagnostics
        | QualificationMode::PairedGeometryCapture
        | QualificationMode::ElementScrollSizeDiagnostics
        | QualificationMode::MeasuredLeafDiagnostics
        | QualificationMode::FlexContentTextDiagnostics
        | QualificationMode::PreparedRangeCapture
        | QualificationMode::FlexCompositionCapture
        | QualificationMode::OriginalFlexParentCapture
        | QualificationMode::OriginalPercentageFlexParentCapture
        | QualificationMode::OriginalSubgridBodyCapture
        | QualificationMode::PreparedAtomicPercentageCapture
        | QualificationMode::HorizontalBrFontBandCapture
        | QualificationMode::VerticalBrFontBandCapture
        | QualificationMode::EmptyLineBrFontBandCapture
        | QualificationMode::Publish
        | QualificationMode::CheckPublished
        | QualificationMode::PublishExact(_)
        | QualificationMode::CheckPublishedExact(_)
        | QualificationMode::DeriveExact(_)
        | QualificationMode::DeriveSpecialized(_)
        | QualificationMode::DeriveMatching(_)
        | QualificationMode::PublishSpecialized(_)
        | QualificationMode::PublishMatching(_)
        | QualificationMode::CheckPublishedMatching(_)
        | QualificationMode::PublishGridLanesDiagnostics
        | QualificationMode::CheckGridLanesDiagnostics
        | QualificationMode::CheckPublishedSpecialized(_) => None,
    };
    for (name, bytes) in INPUTS
        .iter()
        .copied()
        .chain(std::iter::once((input.declaration_path(), input.bytes())))
    {
        if name == "fonts/Ahem.ttf"
            && matches!(
                mode.box_observations(),
                BoxObservationRequest::EmptyLineBrFontBand
                    | BoxObservationRequest::OriginalPercentageFlexParent
            )
        {
            continue;
        }
        let path = RelativePath::new(name)?;
        // Instrument only this invocation's real helper input. Register its exact
        // augmented bytes before the shared generator authenticates any job.
        let augmented = if name == "scripts/gentest/test_helper.js" {
            custody_installation
                .as_deref()
                .map(|install| [bytes, b"\n", CUSTODY_HELPER, b"\n", install.as_bytes()].concat())
        } else {
            None
        };
        let retained = if name == crate::parity_support::capture_authority::ROOT_HELPER_INPUT
            && matches!(access, InputAccess::ReadOnly)
        {
            publication::require_regular_input(root, &path)?;
            let retained = fs::read(root.join(name))?;
            let digest = Sha256Digest::from_bytes(&retained);
            crate::parity_support::capture_authority::root_helper_profile(digest.as_str())?;
            Some(retained)
        } else {
            None
        };
        let bytes = if name == crate::parity_support::capture_authority::ROOT_HELPER_INPUT
            && let Some(helper) = retained_helper
        {
            let digest = Sha256Digest::from_bytes(helper);
            crate::parity_support::capture_authority::root_helper_profile(digest.as_str())?;
            helper
        } else {
            bytes
        };
        let bytes = retained
            .as_deref()
            .or(augmented.as_deref())
            .unwrap_or(bytes);
        inventory_input(root, &path, bytes, &mut inventory.expected_inputs, access)?;
        if name != input.declaration_path() {
            inventory.global_inputs.push(path);
        }
    }
    if let Some(bytes) = mode.box_observations().input_bytes() {
        let path = RelativePath::new(OBSERVATION_REQUEST_INPUT)?;
        inventory_input(root, &path, bytes, &mut inventory.expected_inputs, access)?;
        inventory.global_inputs.push(path);
    }
    if let Some((path, script)) = mode.box_observations().proof_script() {
        let path = RelativePath::new(path)?;
        inventory_input(
            root,
            &path,
            script.as_bytes(),
            &mut inventory.expected_inputs,
            access,
        )?;
        inventory.global_inputs.push(path);
    }
    let policy_profile = mode.specialized_profile().or_else(|| {
        if !matches!(input.identity, DeclarationIdentity::Focused(_)) {
            return None;
        }
        match mode {
            QualificationMode::PreparedRangeCapture => Some(SpecializedProfile::PreparedRange),
            QualificationMode::FlexCompositionCapture => Some(SpecializedProfile::FlexComposition),
            _ => None,
        }
    });
    match policy_profile {
        Some(SpecializedProfile::PreparedRange) => {
            let path = RelativePath::new(
                crate::parity_support::prepared_range_qualification::RETAINED_POLICY,
            )?;
            let bytes = publication::prepared_range_policy(owner)?;
            inventory_input(root, &path, &bytes, &mut inventory.expected_inputs, access)?;
            inventory.global_inputs.push(path);
        }
        Some(SpecializedProfile::FlexComposition) => {
            for (path, digest) in flex_composition_qualification::input_pins() {
                let path = RelativePath::new(path)?;
                let digest: Sha256Digest = serde_json::from_value(digest.into())?;
                register_semantic_input(root, owner, &path, &digest, &mut inventory, access)?;
            }
        }
        Some(
            SpecializedProfile::OriginalFlexParent
            | SpecializedProfile::OriginalPercentageFlexParent
            | SpecializedProfile::OriginalSubgridBody
            | SpecializedProfile::PreparedAtomicPercentage
            | SpecializedProfile::HorizontalBrFontBand
            | SpecializedProfile::VerticalBrFontBand
            | SpecializedProfile::EmptyLineBrFontBand,
        )
        | None => {}
    }
    let original = owner.join("tests/layout/browser_parity");
    for record in records {
        let source_owner = owner.join(record.source_kind.owner_directory());
        publication::require_regular_input(&source_owner, &record.source)?;
        let source = fs::read(source_owner.join(record.source.as_str()))?;
        if Sha256Digest::from_bytes(&source) != record.source_sha256 {
            return Err(format!("declared source changed: {}", record.source.as_str()).into());
        }
        inventory_input(
            root,
            &record.source,
            &source,
            &mut inventory.expected_inputs,
            access,
        )?;
        let (family, stem) = record.source_names()?;
        let record_sha256 = semantic_record_digest(record)?;
        let mut semantic_cases = BTreeMap::new();
        if let Some(specifications) = &record.semantic_specifications {
            for (variant, reference) in specifications {
                let bytes = register_semantic_input(
                    root,
                    owner,
                    &reference.path,
                    &reference.sha256,
                    &mut inventory,
                    access,
                )?;
                let spec = crate::parity_support::exact_differences::ExactDifferenceSpec::parse(
                    std::str::from_utf8(&bytes)?,
                )?;
                let (source, digest) = spec.policy_source();
                let source = RelativePath::new(source)?;
                let expected: Sha256Digest =
                    serde_json::from_value(serde_json::Value::String(digest.to_owned()))?;
                register_semantic_input(root, owner, &source, &expected, &mut inventory, access)?;
                semantic_cases.insert(
                    variant.as_str(),
                    SemanticCase {
                        reference: reference.clone(),
                        record_sha256: record_sha256.clone(),
                        published_edition: spec.published_edition(),
                    },
                );
            }
        }
        let mut cases = Vec::new();
        for (variant, _) in fixture_cases() {
            let output = RelativePath::new(format!("xml/{family}__{variant}.xml"))?;
            let historical = if matches!(record.source_kind, SourceKind::Original) {
                let historical = RelativePath::new(format!("historical/{family}__{variant}.xml"))?;
                let owner_path = if record.source.as_str() == PREPARED_ATOMIC_SOURCE {
                    RelativePath::new(format!(
                        "historical/prepared-text-v1/{family}__{variant}.xml"
                    ))?
                } else {
                    output.clone()
                };
                publication::require_regular_input(&original, &owner_path)?;
                let bytes = fs::read(original.join(owner_path.as_str()))?;
                let digest = record
                    .historical_sha256
                    .get(variant)
                    .ok_or("missing historical variant digest")?;
                if &Sha256Digest::from_bytes(&bytes) != digest {
                    return Err(
                        format!("historical observation changed: {}", output.as_str()).into(),
                    );
                }
                inventory_input(
                    root,
                    &historical,
                    &bytes,
                    &mut inventory.expected_inputs,
                    access,
                )?;
                inventory.global_inputs.push(historical.clone());
                Some(historical)
            } else {
                None
            };
            let case = CaseSpec::new(format!("{stem}__{variant}"), variant.into(), output)?;
            inventory.cases.push(ExpectedCase {
                source: record.source.clone(),
                case: case.clone(),
                historical,
                expected: record.expected.clone(),
                relation: record.relation.clone(),
                semantic: semantic_cases.remove(variant),
            });
            cases.push(case);
        }
        inventory.fixtures.push(FixtureSpec::new(
            stem.into(),
            record.source.clone(),
            cases,
            FixtureStatus::Active,
        )?);
    }
    Ok(inventory)
}

fn qualify_historical_geometry(current: &str, historical: &str, case_id: &str) -> Result<()> {
    qualification_catalogue::verify_historical_geometry(current, historical, case_id)
}

fn compare_observation_tree(
    current: roxmltree::Node<'_, '_>,
    historical: roxmltree::Node<'_, '_>,
    path: &str,
) {
    qualification_catalogue::compare_observation_tree(current, historical)
        .unwrap_or_else(|error| panic!("{path}: {error}"));
}

fn qualify_xml(xml: &str, case: &ExpectedCase) -> Result<()> {
    let document = roxmltree::Document::parse(xml)?;
    let test = document.root_element();
    assert!(test.has_tag_name("test"));
    assert_eq!(test.attribute("name"), Some(case.case.id()));
    assert_eq!(test.attribute("measurement-version"), Some("4"));
    let viewport = test
        .children()
        .find(|node| node.has_tag_name("viewport"))
        .ok_or("missing viewport")?;
    let ExpectedDisposition::Generated { metadata } = &case.expected else {
        return Err("Unsupported case unexpectedly emitted XML".into());
    };
    let (root_direction, root_writing_mode, root_position) = match metadata {
        ExpectedMetadata::AbsoluteLocal {
            root_direction,
            root_writing_mode,
            inline_offset,
            root_size,
            ..
        } => {
            qualification_catalogue::verify_absolute_metadata(
                xml,
                metadata,
                case.case.id(),
                case.case.variant(),
            )?;
            qualify_root_size(
                test,
                viewport,
                root_size.as_ref(),
                *inline_offset,
                case.case.id(),
            )?;
            (
                root_direction.as_str(),
                root_writing_mode.as_str(),
                "absolute",
            )
        }
        ExpectedMetadata::BlockFlow {
            parent_direction,
            root_direction,
            root_writing_mode,
            root_position,
            atomic_paragraphs: _,
            grid_reference_owners: _,
        } => {
            // The zero-edge Htb Auto body owns only its actual inline extent.
            // No used/root height is promoted into a body percentage basis or
            // a hypothetical static-position rectangle. The reader independently
            // validates the actual Native body-origin placement.
            assert_eq!(test.attribute("observation-frame"), Some("root-border"));
            assert_eq!(viewport.attribute("root-context"), Some("block-flow-auto"));
            assert_eq!(
                viewport.attribute("parent-direction"),
                Some(parent_direction.as_str())
            );
            assert_eq!(
                px_attribute(viewport, "parent-inline-extent")?,
                px_attribute(viewport, "fallback-icb-width")?
            );
            for field in [
                "containing-width",
                "containing-height",
                "containing-direction",
                "hypothetical-direction",
                "hypothetical-display",
                "hypothetical-anchor",
                "hypothetical-inline-offset",
                "parent-block-extent",
            ] {
                assert_eq!(
                    viewport.attribute(field),
                    None,
                    "Auto body has no definite block/hypothetical authority"
                );
            }
            (
                root_direction.as_str(),
                root_writing_mode.as_str(),
                root_position.as_str(),
            )
        }
    };
    assert_eq!(
        viewport.attribute("fallback-nearest-scrollport"),
        Some("none")
    );
    for field in [
        "fallback-containing-width-limit",
        "fallback-containing-height-limit",
    ] {
        assert_eq!(viewport.attribute(field), Some("unbounded"));
    }
    for field in ["fallback-icb-width", "fallback-icb-height"] {
        assert!(px_attribute(viewport, field)? >= 0.0);
    }
    let input = test
        .children()
        .find(|node| node.has_tag_name("input"))
        .ok_or("missing root input")?;
    let expected_atomic_paragraphs = match metadata {
        ExpectedMetadata::BlockFlow {
            atomic_paragraphs, ..
        } => *atomic_paragraphs,
        ExpectedMetadata::AbsoluteLocal { .. } => None,
    };
    qualify_atomic_metric_census(input, expected_atomic_paragraphs, case.case.id())?;
    let root = input
        .children()
        .find(roxmltree::Node::is_element)
        .ok_or("missing root node")?;
    // Relative and HorizontalTb are the protocol's explicit elision defaults.
    assert_eq!(
        root.attribute("position").unwrap_or("relative"),
        root_position
    );
    assert_eq!(
        root.attribute("writing-mode").unwrap_or("horizontal-tb"),
        root_writing_mode
    );
    let root_direction = if root_direction == "variant" {
        if case.case.variant().ends_with("_rtl") {
            "rtl"
        } else {
            "ltr"
        }
    } else {
        root_direction
    };
    assert_eq!(root.attribute("direction"), Some(root_direction));
    Ok(())
}

fn qualify_grid_reference_census(
    root: &crate::parity_support::Node,
    expected: &[String],
) -> Result<()> {
    fn visit(
        node: &crate::parity_support::Node,
        path: String,
        actual: &mut Vec<String>,
    ) -> Result<()> {
        if node.style.get("grid-alignment-reference").is_some() {
            if !matches!(node.style.get("display"), Some("grid" | "inline-grid")) {
                return Err(format!(
                    "Grid reference certificate belongs to non-Grid source at {path}"
                )
                .into());
            }
            actual.push(path.clone());
        }
        for (index, child) in node.children.iter().enumerate() {
            let child_path = if path == "/" {
                format!("/{index}")
            } else {
                format!("{path}/{index}")
            };
            visit(child, child_path, actual)?;
        }
        Ok(())
    }
    let mut actual = Vec::new();
    visit(root, "/".into(), &mut actual)?;
    if actual != expected {
        return Err(format!(
            "actual source-bound Grid reference census: expected {expected:?}, got {actual:?}"
        )
        .into());
    }
    eprintln!(
        "qualify-root-context: {} actual own-Grid reference records qualified",
        actual.len()
    );
    Ok(())
}

fn qualify_atomic_metric_census(
    input: roxmltree::Node<'_, '_>,
    expected: Option<usize>,
    case: &str,
) -> Result<()> {
    let captures = input
        .descendants()
        .filter_map(|node| node.attribute("atomic-paragraph-metrics"))
        .collect::<Vec<_>>();
    assert_eq!(
        captures.len(),
        expected.unwrap_or(0),
        "{case}: actual named paragraph census"
    );
    for raw in captures {
        let value: serde_json::Value = serde_json::from_str(raw)?;
        assert_eq!(value["profile"], 1);
        assert_eq!(
            value["font"]["sha256"],
            "e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c"
        );
        assert_eq!(value["font"]["loaded"], true);
        assert_eq!(value["font"]["size"].as_f64(), Some(10.0));
        assert_eq!(value["reference"], "central");
        assert!(["auto", "baseline"].contains(&value["alignmentBaseline"].as_str().unwrap_or("")));
        assert_eq!(value["verticalAlign"], "baseline");
        assert_eq!(value["before"].as_f64(), Some(0.0));
        assert_eq!(value["after"].as_f64(), Some(0.0));
        let order = value["order"]
            .as_array()
            .ok_or("missing actual atomic order")?;
        assert_eq!(order.len(), 3);
        assert_eq!(order[1]["kind"], "break");
        assert_eq!(order[1]["reference"], "central");
        assert_eq!(order[1]["before"].as_f64(), Some(0.0));
        assert_eq!(order[1]["after"].as_f64(), Some(0.0));
    }
    if let Some(count) = expected {
        eprintln!(
            "qualify-root-context: {case}: {count} actual loaded-font Central paragraph/BR certificates qualified"
        );
    }
    Ok(())
}

fn qualify_dispositions(bytes: &[u8], cases: &[ExpectedCase]) -> Result<()> {
    let report: serde_json::Value = serde_json::from_slice(bytes)?;
    let unsupported = report
        .get("unsupported")
        .and_then(serde_json::Value::as_array)
        .ok_or("browser report omitted Unsupported dispositions")?;
    for case in cases {
        if let ExpectedDisposition::Unsupported { reason } = &case.expected {
            let actual = unsupported
                .iter()
                .find(|entry| {
                    entry.get("name").and_then(serde_json::Value::as_str) == Some(case.case.id())
                })
                .ok_or_else(|| {
                    format!("{}: missing declared Unsupported refusal", case.case.id())
                })?;
            assert_eq!(
                actual.get("source").and_then(serde_json::Value::as_str),
                Some(case.source.as_str())
            );
            assert_eq!(
                actual.get("variant").and_then(serde_json::Value::as_str),
                Some(case.case.variant())
            );
            assert_eq!(
                actual.get("reason").and_then(serde_json::Value::as_str),
                Some(reason.as_str()),
                "{}: semantic/provenance refusal",
                case.case.id()
            );
        }
    }
    Ok(())
}
fn number_attribute(node: roxmltree::Node<'_, '_>, name: &str) -> Result<f64> {
    let value: f64 = node
        .attribute(name)
        .ok_or_else(|| format!("missing {name}"))?
        .parse()?;
    if !value.is_finite() {
        return Err(format!("nonfinite {name}").into());
    }
    Ok(value)
}
fn px_attribute(node: roxmltree::Node<'_, '_>, name: &str) -> Result<f64> {
    let raw = node
        .attribute(name)
        .and_then(|value| value.strip_suffix("px"))
        .ok_or_else(|| format!("missing pixel {name}"))?;
    let value: f64 = raw.parse()?;
    if !value.is_finite() {
        return Err(format!("nonfinite {name}").into());
    }
    Ok(value)
}
fn qualify_root_size(
    test: roxmltree::Node<'_, '_>,
    viewport: roxmltree::Node<'_, '_>,
    root_size: Option<&ExpectedRootSize>,
    inline_offset: f64,
    case_id: &str,
) -> Result<()> {
    let Some(expected) = root_size else {
        return Ok(());
    };
    assert_eq!(test.attribute("use-rounding"), Some("true"));
    let (width, height) = match expected {
        ExpectedRootSize::Fixed { width, height } => (*width, *height),
        ExpectedRootSize::FlexPair {
            item_width,
            item_height,
        } => {
            // Two unshrinkable native children: MC=itemWidth,
            // MAX=2*itemWidth. The source-authenticated CB and indent are
            // operands; the eventual observed root rectangle is not.
            let available = (px_attribute(viewport, "containing-width")? - inline_offset).max(0.0);
            let width = available.max(*item_width).min(2.0 * item_width);
            (
                width,
                if width < 2.0 * item_width {
                    2.0 * item_height
                } else {
                    *item_height
                },
            )
        }
    };
    let expectations = test
        .children()
        .find(|node| node.has_tag_name("expectations"))
        .ok_or("missing expectations")?;
    let root = expectations
        .children()
        .find(roxmltree::Node::is_element)
        .ok_or("missing root observation")?;
    assert_eq!(number_attribute(root, "x")?, 0.0, "{case_id}: local root x");
    assert_eq!(number_attribute(root, "y")?, 0.0, "{case_id}: local root y");
    assert_eq!(
        number_attribute(root, "width")?,
        (width + 0.5).floor(),
        "{case_id}: authored/shrink width"
    );
    assert_eq!(
        number_attribute(root, "height")?,
        (height + 0.5).floor(),
        "{case_id}: actual native wrap height"
    );
    Ok(())
}
fn observation_section<'doc, 'input>(
    document: &'doc roxmltree::Document<'input>,
    name: &str,
) -> Result<roxmltree::Node<'doc, 'input>> {
    document
        .root_element()
        .children()
        .find(|node| node.has_tag_name(name))
        .ok_or_else(|| format!("missing {name}").into())
}
fn world_observations<'doc, 'input>(
    document: &'doc roxmltree::Document<'input>,
) -> Result<BTreeMap<String, roxmltree::Node<'doc, 'input>>> {
    let mut rows = BTreeMap::new();
    for node in observation_section(document, "diagnostics")?
        .children()
        .filter(roxmltree::Node::is_element)
    {
        assert!(node.has_tag_name("world-rounded-layout"));
        let path = node
            .attribute("node-path")
            .ok_or("world node path")?
            .to_owned();
        assert!(rows.insert(path, node).is_none(), "duplicate world path");
    }
    Ok(rows)
}
fn qualify_pairs(root: &Path, cases: &[ExpectedCase]) -> Result<()> {
    for case in cases {
        let Some(relation) = &case.relation else {
            continue;
        };
        let peer_name = format!("{}__{}", relation.peer, case.case.variant());
        let peer = cases
            .iter()
            .find(|other| other.case.id() == peer_name)
            .ok_or("undeclared paired source/variant")?;
        assert!(matches!(
            &peer.expected,
            ExpectedDisposition::Generated { .. }
        ));
        let left_xml = fs::read_to_string(root.join(case.case.output().as_str()))?;
        let right_xml = fs::read_to_string(root.join(peer.case.output().as_str()))?;
        let left = roxmltree::Document::parse(&left_xml)?;
        let right = roxmltree::Document::parse(&right_xml)?;
        // Every generated tree has already undergone the maintained strict Native
        // comparison. Pair equality includes the complete rounded local tree,
        // not just root dimensions or a subset of scroll observations.
        compare_observation_tree(
            observation_section(&left, "expectations")?,
            observation_section(&right, "expectations")?,
            &format!("{}~{}", case.case.id(), peer.case.id()),
        );
        let left_sizes = left
            .root_element()
            .children()
            .find(|node| node.has_tag_name("element-scroll-sizes"));
        let right_sizes = right
            .root_element()
            .children()
            .find(|node| node.has_tag_name("element-scroll-sizes"));
        match (left_sizes, right_sizes) {
            (Some(left), Some(right)) => compare_observation_tree(
                left,
                right,
                &format!("{}~{} element scroll sizes", case.case.id(), peer.case.id()),
            ),
            (None, None) => {}
            _ => return Err("paired element scroll-size coverage differs".into()),
        }
        if relation.world_phase_change {
            let left_world = world_observations(&left)?;
            let right_world = world_observations(&right)?;
            assert_eq!(
                left_world.keys().collect::<Vec<_>>(),
                right_world.keys().collect::<Vec<_>>()
            );
            // These are raw browser diagnostics, not admission of the typed
            // ElementScrollSizes request. Their complete tuples must commute
            // with the same translation even when that Native channel refuses.
            for (path, left) in &left_world {
                let right = right_world[path];
                for field in [
                    "scroll-width",
                    "scroll-height",
                    "client-width",
                    "client-height",
                ] {
                    let value = |node: roxmltree::Node<'_, '_>| -> Result<u32> {
                        Ok(node
                            .attribute(field)
                            .ok_or("missing paired DOM size diagnostic")?
                            .parse()?)
                    };
                    assert_eq!(
                        value(*left)?,
                        value(right)?,
                        "{path}: paired {field} changed"
                    );
                }
            }
            // Both actual root and child have authored10.5px height at the same
            // world phase. A half-pixel translation changes their world-corner
            // rounded heights together while their local tree stays identical.
            for path in ["0", "0.0"] {
                let l = *left_world.get(path).ok_or("missing shared-phase box")?;
                let r = *right_world
                    .get(path)
                    .ok_or("missing paired shared-phase box")?;
                assert_ne!(
                    number_attribute(l, "height")?,
                    number_attribute(r, "height")?,
                    "{path}: missing real world phase change"
                );
            }
            assert_eq!(
                number_attribute(left_world["0"], "height")?,
                number_attribute(left_world["0.0"], "height")?
            );
            assert_eq!(
                number_attribute(right_world["0"], "height")?,
                number_attribute(right_world["0.0"], "height")?
            );
        }
    }
    Ok(())
}

fn qualify_tampering(
    root: &Path,
    corpus: &BrowserCorpus,
    source: &RelativePath,
    output: &RelativePath,
    adapter: &QualificationAdapter,
) -> Result<usize> {
    for path in [source, output] {
        let path = root.join(path.as_str());
        let original = fs::read(&path)?;
        let mut modified = original.clone();
        modified.push(b'\n');
        fs::write(&path, modified)?;
        let result = browser::check_corpus(corpus, adapter);
        fs::write(&path, original)?;
        assert_invalid_inventory(result);
    }
    let undeclared = root.join("scripts/gentest/undeclared.js");
    fs::write(&undeclared, b"// undeclared input\n")?;
    let result = browser::check_corpus(corpus, adapter);
    fs::remove_file(undeclared)?;
    assert_invalid_inventory(result);
    if matches!(
        adapter.observations,
        BoxObservationRequest::ElementScrollSizes | BoxObservationRequest::FlexContentText
    ) {
        match browser::check_corpus(
            corpus,
            &QualificationAdapter::for_mode(QualificationMode::Regular),
        ) {
            Err(GenerationError::Adapter {
                stage: "check preparation",
                error: AdapterError::Preparation(detail),
                ..
            }) => {
                assert!(detail.starts_with(OBSERVATION_REQUEST_INPUT), "{detail}");
            }
            other => panic!("expected registered-request adapter refusal, received {other:?}"),
        }
        let mut checked = 4;
        for path in std::iter::once(OBSERVATION_REQUEST_INPUT)
            .chain(adapter.observations.proof_script().map(|(path, _)| path))
        {
            let path = root.join(path);
            let original = fs::read(&path)?;
            fs::write(&path, b"{}\n")?;
            let changed = browser::check_corpus(corpus, adapter);
            fs::write(&path, &original)?;
            assert_invalid_inventory(changed);
            fs::remove_file(&path)?;
            let missing = browser::check_corpus(corpus, adapter);
            fs::write(&path, original)?;
            assert_invalid_inventory(missing);
            checked += 2;
        }
        return Ok(checked);
    }
    Ok(3)
}

fn assert_invalid_inventory(
    result: std::result::Result<browser::BrowserGenerationReport, GenerationError<AdapterError>>,
) {
    match result {
        Err(GenerationError::Generator(error)) => assert_eq!(
            error.kind(),
            GeneratorErrorKind::InvalidInventory,
            "{error}"
        ),
        other => panic!("expected typed input/output inventory refusal, received {other:?}"),
    }
}

fn semantic_identity_changed(component: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    crate::parity_support::exact_differences::QualificationError::IdentityChanged {
        component: component.into(),
    }
    .into()
}

fn semantic_record_digest(record: &Record) -> Result<Sha256Digest> {
    let digest = crate::parity_support::capture_authority::semantic_record_digest(
        &serde_json::to_value(record)?,
    )?;
    Ok(serde_json::from_value(serde_json::Value::String(digest))?)
}

fn register_semantic_input(
    root: &Path,
    owner: &Path,
    path: &RelativePath,
    expected: &Sha256Digest,
    inventory: &mut Inventory,
    access: InputAccess,
) -> Result<Vec<u8>> {
    publication::require_regular_input(owner, path)?;
    let bytes = fs::read(owner.join(path.as_str()))?;
    if &Sha256Digest::from_bytes(&bytes) != expected {
        return Err(semantic_identity_changed(format!(
            "registered semantic input {}",
            path.as_str()
        )));
    }
    if let Some(registered) = inventory.expected_inputs.get(path) {
        if registered != expected {
            return Err(semantic_identity_changed(
                "registered semantic input conflicts with capture input",
            ));
        }
    } else {
        inventory_input(root, path, &bytes, &mut inventory.expected_inputs, access)?;
        inventory.global_inputs.push(path.clone());
    }
    Ok(bytes)
}

use crate::parity_support::capture_authority::{
    BrowserSettingsWitness, CaptureMetadata, CapturedArtifact, CaseDescriptor, ExactCase,
    IdentityInputs, PathDigest, QualifiedCase, identity_from_verified_inputs,
};

fn parse_capture_golden(
    root: &Path,
    artifact: &browser::BrowserGeneratedArtifact,
    xml: &str,
    observation: BoxObservationRequest,
    bound_report: Option<&str>,
    published: bool,
) -> Result<crate::parity_support::Golden> {
    use crate::parity_support::Golden;
    let parse = match observation {
        BoxObservationRequest::FlexContentText => Golden::parse_with_flex_content_text,
        BoxObservationRequest::PreparedAtomicPercentage => {
            Golden::parse_with_prepared_atomic_percentage
        }
        BoxObservationRequest::FlexComposition if published => {
            Golden::parse_with_published_flex_composition
        }
        BoxObservationRequest::FlexComposition => Golden::parse_with_flex_composition,
        _ => return Ok(Golden::parse(xml)?),
    };
    let report = bound_report.ok_or("missing bound capture report")?;
    let source = artifact.source();
    publication::require_regular_input(root, source)?;
    Ok(parse(
        xml,
        report,
        source.as_str(),
        &fs::read(root.join(source.as_str()))?,
        artifact.output().as_str(),
    )?)
}

fn qualify_semantic_case(
    capture: &SemanticCapture<'_>,
    artifact: &browser::BrowserGeneratedArtifact,
    case: &ExpectedCase,
    golden: &crate::parity_support::Golden,
) -> Result<SemanticResult> {
    use crate::parity_support::exact_differences::{
        CaptureAuthority, ExactDifferenceSpec, complete_captured_comparison,
        verify_wrapping_world_predecessor,
    };
    let SemanticCapture {
        root,
        inventory,
        corpus,
        report,
        bound_report,
        observation,
        lineage,
        requirements,
    } = capture;
    let semantic = case
        .semantic
        .as_ref()
        .ok_or("missing registered semantic specification")?;
    let reference = &semantic.reference;
    publication::require_regular_input(root, &reference.path)?;
    let spec_bytes = fs::read(root.join(reference.path.as_str()))?;
    if Sha256Digest::from_bytes(&spec_bytes) != reference.sha256
        || inventory.expected_inputs.get(&reference.path) != Some(&reference.sha256)
    {
        return Err(semantic_identity_changed("semantic specification bytes"));
    }
    let spec = ExactDifferenceSpec::parse(std::str::from_utf8(&spec_bytes)?)?;
    let (policy_path, expected_policy) = spec.policy_source();
    let policy_path = RelativePath::new(policy_path)?;
    publication::require_regular_input(root, &policy_path)?;
    let policy = Sha256Digest::from_bytes(&fs::read(root.join(policy_path.as_str()))?);
    if policy.as_str() != expected_policy
        || inventory.expected_inputs.get(&policy_path) != Some(&policy)
    {
        return Err(semantic_identity_changed("Native policy source"));
    }
    let report_value: serde_json::Value = serde_json::from_slice(&report.to_bytes()?)?;
    let metadata: CaptureMetadata = serde_json::from_value(
        report_value
            .get("metadata")
            .ok_or("missing report metadata")?
            .clone(),
    )?;
    if metadata.schema_version != 4 || metadata.inputs != string_inputs(&inventory.expected_inputs)
    {
        return Err(semantic_identity_changed("capture metadata input census"));
    }
    let captured: CapturedArtifact = serde_json::from_value(serde_json::to_value(artifact)?)?;
    if captured.name != case.case.id()
        || captured.source != case.source.as_str()
        || captured.output != case.case.output().as_str()
        || captured.variant != case.case.variant()
        || inventory
            .expected_inputs
            .get(&case.source)
            .map(Sha256Digest::as_str)
            != Some(captured.source_sha256.as_str())
    {
        return Err(semantic_identity_changed("source/variant/capture record"));
    }
    publication::require_regular_input(root, artifact.output())?;
    let xml = fs::read(root.join(artifact.output().as_str()))?;
    if Sha256Digest::from_bytes(&xml).as_str() != captured.output_sha256
        || parse_capture_golden(
            root,
            artifact,
            std::str::from_utf8(&xml)?,
            *observation,
            *bound_report,
            false,
        )? != *golden
    {
        return Err(semantic_identity_changed("actual capture XML"));
    }
    let historical = case
        .historical
        .as_ref()
        .map(|path| -> Result<Sha256Digest> {
            publication::require_regular_input(root, path)?;
            let bytes = fs::read(root.join(path.as_str()))?;
            let digest = Sha256Digest::from_bytes(&bytes);
            if inventory.expected_inputs.get(path) != Some(&digest) {
                return Err(semantic_identity_changed("historical XML"));
            }
            if matches!(
                lineage,
                SemanticLineage::Published(PublishedEdition::WrappingWorldCorners)
            ) {
                verify_wrapping_world_predecessor(golden, &bytes)?;
            }
            Ok(digest)
        })
        .transpose()?;
    let excluded_specs = inventory
        .cases
        .iter()
        .filter_map(|case| {
            case.semantic
                .as_ref()
                .map(|s| s.reference.path.as_str().to_owned())
        })
        .collect();
    let (observation, request_name) = if golden.has_flex_content_text() {
        (BoxObservationRequest::FlexContentText, "flex-content-text")
    } else if golden.measured_leaf.is_some() {
        (BoxObservationRequest::MeasuredLeaf, "measured-leaf")
    } else if golden.has_element_scroll_sizes() {
        (
            BoxObservationRequest::ElementScrollSizes,
            "element-scroll-sizes",
        )
    } else {
        (BoxObservationRequest::Geometry, "geometry")
    };
    let observation_sha = observation.input_bytes().map(Sha256Digest::from_bytes);
    let settings =
        BrowserSettingsWitness::from_official_json(&serde_json::to_vec(corpus.browser())?)?;
    let descriptor = CaseDescriptor {
        name: case.case.id().to_owned(),
        source: case.source.as_str().to_owned(),
        variant: case.case.variant().to_owned(),
        output: PathDigest {
            path: case.case.output().as_str().to_owned(),
            sha256: captured.output_sha256.clone(),
        },
        record_sha256: semantic.record_sha256.as_str().to_owned(),
    };
    let full_inputs = string_inputs(&inventory.expected_inputs);
    let xml_sha = Sha256Digest::from_bytes(&xml);
    let verified = identity_from_verified_inputs(IdentityInputs {
        metadata: &metadata,
        artifact: &captured,
        case: &descriptor,
        full_inputs: &full_inputs,
        excluded_specs: &excluded_specs,
        actual_xml_sha256: xml_sha.as_str(),
        historical_sha256: historical.as_ref().map(Sha256Digest::as_str),
        policy_sha256: policy.as_str(),
        browser_settings: &settings,
        observation_request: request_name,
        observation_input_sha256: observation_sha.as_ref().map(Sha256Digest::as_str),
    })?;
    let authority = match lineage {
        SemanticLineage::Focused => CaptureAuthority::Focused(verified.identity()),
        SemanticLineage::Published(edition) => CaptureAuthority::Published {
            edition: *edition,
            identity: verified.identity(),
        },
    };
    let current = match lineage {
        SemanticLineage::Focused => None,
        SemanticLineage::Published(edition) => {
            let selected = crate::parity_support::capture_authority::CaseSelection {
                name: descriptor.name.clone(),
                source: descriptor.source.clone(),
                source_sha256: captured.source_sha256.clone(),
                variant: descriptor.variant.clone(),
                output: descriptor.output.path.clone(),
                record_sha256: descriptor.record_sha256.clone(),
                historical_sha256: historical.as_ref().map(|s| s.as_str().to_owned()),
            };
            let expected = spec.prebind_current(*edition, &selected)?;
            if requirements.and_then(|map| map.get(case.case.id())) != Some(&expected) {
                return Err("prebound current representative changed during execution".into());
            }
            Some(expected)
        }
    };
    let resolved = spec.resolve_against(authority, golden)?;
    let completed = complete_captured_comparison(golden, &resolved)?;
    for field in completed.raw_differences() {
        eprintln!(
            "qualify-root-context: {} raw difference: {field:?}",
            case.case.id()
        );
    }
    let verdict = resolved.qualify(&completed)?;
    let terminal = current.map(|requirements| {
        QualifiedCase::Exact(ExactCase {
            case: descriptor,
            spec: PathDigest {
                path: reference.path.as_str().to_owned(),
                sha256: reference.sha256.as_str().to_owned(),
            },
            policy: PathDigest {
                path: policy_path.as_str().to_owned(),
                sha256: policy.as_str().to_owned(),
            },
            historical: case
                .historical
                .as_ref()
                .zip(historical.as_ref())
                .map(|(path, sha)| PathDigest {
                    path: path.as_str().to_owned(),
                    sha256: sha.as_str().to_owned(),
                }),
            requirements,
            identity: verified.projection().clone(),
        })
    });
    Ok(SemanticResult { verdict, terminal })
}
fn string_inputs(map: &BTreeMap<RelativePath, Sha256Digest>) -> BTreeMap<String, String> {
    map.iter()
        .map(|(path, sha)| (path.as_str().to_owned(), sha.as_str().to_owned()))
        .collect()
}

#[cfg(test)]
mod semantic_registration_tests {
    use super::*;

    fn reference() -> SemanticReference {
        SemanticReference {
            path: RelativePath::new("semantic/fixed.json").unwrap(),
            sha256: Sha256Digest::from_bytes(b"authored semantic input"),
        }
    }

    #[test]
    fn semantic_registration_accepts_only_generated_known_variants_and_paths() {
        let base = r#"schema_version = 3
[[records]]
source = "html/fixed.html"
source_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
source_kind = "authored"
[records.expected]
kind = "generated"
[records.expected.metadata]
context = "absolute-local"
hypothetical_display = "flex"
containing_direction = "ltr"
hypothetical_direction = "ltr"
root_direction = "variant"
root_writing_mode = "horizontal-tb"
inline_offset = 0
"#;
        let digest = reference().sha256;
        let appended = |variant: &str, path: &str| {
            format!(
                "{base}\n[records.semantic_specifications.{variant}]\npath = {path:?}\nsha256 = \"{}\"\n",
                digest.as_str()
            )
        };
        assert!(
            DeclarationInput::parse(appended("content_box_ltr", "semantic/fixed.json").as_bytes())
                .is_ok()
        );
        for (variant, path) in [
            ("future_variant", "semantic/fixed.json"),
            ("content_box_ltr", "other/fixed.json"),
        ] {
            assert!(DeclarationInput::parse(appended(variant, path).as_bytes()).is_err());
        }
        let unsupported = format!(
            r#"schema_version = 3
[[records]]
source = "html/unsupported.html"
source_sha256 = "{}"
source_kind = "authored"
[records.expected]
kind = "unsupported"
reason = "outside the source profile"
[records.semantic_specifications.content_box_ltr]
path = "semantic/fixed.json"
sha256 = "{}"
"#,
            digest.as_str(),
            digest.as_str()
        );
        assert!(DeclarationInput::parse(unsupported.as_bytes()).is_err());
    }

    #[test]
    fn semantic_record_identity_excludes_only_registration_digests() {
        let mut input = DeclarationInput::parse(DECLARATION).unwrap();
        let record = input
            .declaration
            .records
            .iter_mut()
            .find(|record| !record.historical_sha256.is_empty())
            .unwrap();
        let original = semantic_record_digest(record).unwrap();
        record.semantic_specifications =
            Some(BTreeMap::from([("content_box_ltr".into(), reference())]));
        assert_eq!(semantic_record_digest(record).unwrap(), original);
        record
            .semantic_specifications
            .as_mut()
            .unwrap()
            .get_mut("content_box_ltr")
            .unwrap()
            .sha256 = Sha256Digest::from_bytes(b"another authored spec");
        assert_eq!(semantic_record_digest(record).unwrap(), original);
        record.historical_sha256.clear();
        assert_ne!(
            semantic_record_digest(record).unwrap(),
            original,
            "historical authority is semantic identity"
        );
    }

    #[test]
    fn semantic_inputs_authenticate_preparation_and_read_only_checks_without_repair() {
        struct Temporary(PathBuf);
        impl Drop for Temporary {
            fn drop(&mut self) {
                fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
        let source = Temporary(Invocation::new(owner).unwrap().root);
        let capture = Temporary(Invocation::new(owner).unwrap().root);
        fs::create_dir(source.0.join("semantic")).unwrap();
        let reference = reference();
        let bytes = b"authored semantic input";
        fs::write(source.0.join(reference.path.as_str()), bytes).unwrap();
        let fresh_inventory = || Inventory {
            declaration_path: RelativePath::new("corpus.toml").unwrap(),
            mode: QualificationMode::Regular,
            fixtures: Vec::new(),
            cases: Vec::new(),
            global_inputs: Vec::new(),
            expected_inputs: BTreeMap::new(),
        };
        let mut inventory = fresh_inventory();
        assert_eq!(
            register_semantic_input(
                &capture.0,
                &source.0,
                &reference.path,
                &reference.sha256,
                &mut inventory,
                InputAccess::Prepare
            )
            .unwrap(),
            bytes
        );
        assert_eq!(inventory.global_inputs, [reference.path.clone()]);
        assert_eq!(inventory.expected_inputs[&reference.path], reference.sha256);
        let mut inventory = fresh_inventory();
        register_semantic_input(
            &capture.0,
            &source.0,
            &reference.path,
            &reference.sha256,
            &mut inventory,
            InputAccess::ReadOnly,
        )
        .unwrap();
        fs::write(capture.0.join(reference.path.as_str()), b"changed capture").unwrap();
        let mut inventory = fresh_inventory();
        assert!(
            register_semantic_input(
                &capture.0,
                &source.0,
                &reference.path,
                &reference.sha256,
                &mut inventory,
                InputAccess::ReadOnly
            )
            .is_err()
        );
        assert_eq!(
            fs::read(capture.0.join(reference.path.as_str())).unwrap(),
            b"changed capture"
        );
        fs::write(source.0.join(reference.path.as_str()), b"changed owner").unwrap();
        let mut inventory = fresh_inventory();
        assert!(
            register_semantic_input(
                &capture.0,
                &source.0,
                &reference.path,
                &reference.sha256,
                &mut inventory,
                InputAccess::Prepare
            )
            .is_err()
        );
        assert_eq!(
            fs::read(capture.0.join(reference.path.as_str())).unwrap(),
            b"changed capture"
        );
    }
}

#[cfg(test)]
mod numeric_diagnostic_tests {
    use super::*;

    fn declaration() -> Declaration {
        Declaration::parse(DECLARATION).unwrap()
    }

    #[test]
    fn scroll_size_selection_preserves_the_declared_phase_pair() {
        let mut input = declaration();
        select_records(
            &mut input,
            Some("html/qualification/block-translation-0-75.html"),
            QualificationMode::ElementScrollSizeDiagnostics,
        )
        .unwrap();
        assert_eq!(
            input
                .records
                .iter()
                .map(|record| record.source.as_str())
                .collect::<Vec<_>>(),
            [
                "html/qualification/block-translation-0-25.html",
                "html/qualification/block-translation-0-75.html",
            ]
        );
        assert_eq!(
            input.expected().unwrap().generated(),
            2 * fixture_cases().len()
        );
        assert!(
            input.records[1]
                .relation
                .as_ref()
                .unwrap()
                .world_phase_change
        );
        assert!(
            select_records(
                &mut declaration(),
                None,
                QualificationMode::ElementScrollSizeDiagnostics
            )
            .is_err()
        );
        assert!(
            select_records(
                &mut declaration(),
                Some("html/missing.html"),
                QualificationMode::ElementScrollSizeDiagnostics
            )
            .is_err()
        );
    }

    #[test]
    fn scroll_size_inventory_authenticates_request_and_refuses_changed_input_without_repair() {
        struct TemporaryCorpus(PathBuf);
        impl Drop for TemporaryCorpus {
            fn drop(&mut self) {
                fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
        let invocation = Invocation::new(owner).unwrap();
        let root = TemporaryCorpus(invocation.root);
        let mut input = DeclarationInput::parse(DECLARATION).unwrap();
        select_records(
            &mut input.declaration,
            Some("html/grid/grid_fit_content_percent_indefinite_max_content_hidden.html"),
            QualificationMode::ElementScrollSizeDiagnostics,
        )
        .unwrap();
        let mode = QualificationMode::ElementScrollSizeDiagnostics;
        let prepared = inventory(&root.0, owner, &input, mode, InputAccess::Prepare).unwrap();
        let path = RelativePath::new(OBSERVATION_REQUEST_INPUT).unwrap();
        let bytes = mode.box_observations().input_bytes().unwrap();
        assert_eq!(fs::read(root.0.join(path.as_str())).unwrap(), bytes);
        assert!(prepared.global_inputs.contains(&path));
        assert_eq!(
            prepared.expected_inputs[&path],
            Sha256Digest::from_bytes(bytes)
        );
        let checked = inventory(&root.0, owner, &input, mode, InputAccess::ReadOnly).unwrap();
        assert_eq!(checked.expected_inputs, prepared.expected_inputs);
        fs::write(root.0.join(path.as_str()), b"{}\n").unwrap();
        assert!(inventory(&root.0, owner, &input, mode, InputAccess::ReadOnly).is_err());
        assert_eq!(fs::read(root.0.join(path.as_str())).unwrap(), b"{}\n");
    }

    #[test]
    fn numeric_diagnostics_require_a_generated_declared_source_with_four_variants() {
        let mut declaration = declaration();
        let source = declaration
            .records
            .iter()
            .find(|record| {
                record.relation.is_none()
                    && matches!(&record.expected, ExpectedDisposition::Generated { .. })
            })
            .unwrap()
            .source
            .as_str()
            .to_owned();
        select_records(
            &mut declaration,
            Some(&source),
            QualificationMode::NumericDiagnostics,
        )
        .unwrap();
        let expected = declaration.expected().unwrap();
        assert_eq!(expected.generated(), 4);
        assert_eq!(expected.unsupported(), 0);
        assert_eq!(declaration.records[0].source.as_str(), source);
    }

    #[test]
    fn numeric_diagnostics_refuse_unsupported_only_without_changing_regular_focus() {
        let mut numeric = declaration();
        let source = numeric
            .records
            .iter()
            .find(|record| {
                record.relation.is_none()
                    && matches!(&record.expected, ExpectedDisposition::Unsupported { .. })
            })
            .unwrap()
            .source
            .as_str()
            .to_owned();
        assert_eq!(
            select_records(
                &mut numeric,
                Some(&source),
                QualificationMode::NumericDiagnostics
            )
            .unwrap_err()
            .to_string(),
            "numeric diagnostics require a Generated fixture"
        );
        let mut regular = declaration();
        select_records(&mut regular, Some(&source), QualificationMode::Regular).unwrap();
        assert_eq!(regular.expected().unwrap().unsupported(), 4);
        assert_eq!(
            select_records(
                &mut declaration(),
                None,
                QualificationMode::NumericDiagnostics
            )
            .unwrap_err()
            .to_string(),
            "numeric diagnostics require one declared fixture"
        );
    }

    #[test]
    fn numeric_diagnostics_keep_paired_observation_inventory_authority() {
        let mut declaration = declaration();
        let source = declaration
            .records
            .iter()
            .find(|record| record.relation.is_some())
            .unwrap()
            .source
            .as_str()
            .to_owned();
        assert_eq!(
            select_records(
                &mut declaration,
                Some(&source),
                QualificationMode::NumericDiagnostics
            )
            .unwrap_err()
            .to_string(),
            "paired observations require the complete inventory"
        );
    }

    fn fixture_golden() -> crate::parity_support::Golden {
        crate::parity_support::Golden::parse(r#"<test name="numeric-transport" measurement-version="4" observation-frame="root-border" use-rounding="false">
<viewport root-context="block-flow-auto" parent-inline-extent="800px" parent-direction="ltr" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>
<input><div grid-alignment-profile="{&quot;profile&quot;:1,&quot;owners&quot;:[]}" display="block" position="relative" direction="ltr" width="20px" height="10px"><div display="block" width="10px" height="5px"/></div></input>
<expectations><node x="0" y="0" width="20" height="10"><node x="0" y="0" width="10" height="5"/></node></expectations>
<diagnostics><world-rounded-layout node-path="0" x="0" y="0" width="20" height="10"/><world-rounded-layout node-path="0.0" x="0" y="0" width="10" height="5"/></diagnostics></test>"#).unwrap()
    }

    fn case() -> ExpectedCase {
        let record = declaration()
            .records
            .into_iter()
            .find(|record| {
                record.relation.is_none()
                    && matches!(&record.expected, ExpectedDisposition::Generated { .. })
            })
            .unwrap();
        ExpectedCase {
            source: record.source,
            case: CaseSpec::new(
                "numeric-transport__content_box_ltr".into(),
                "content_box_ltr".into(),
                RelativePath::new("xml/numeric-transport.xml").unwrap(),
            )
            .unwrap(),
            historical: None,
            expected: record.expected,
            relation: None,
            semantic: None,
        }
    }

    #[test]
    fn numeric_diagnostics_emit_every_real_kernel_record_and_fail_on_any_difference() {
        let mut golden = fixture_golden();
        let case = case();
        let mut output = Vec::new();
        assert_eq!(
            collect_numeric_case(&golden, &case, &mut output).unwrap(),
            0
        );
        assert!(output.is_empty());
        golden.expectations.x = Some(1.0);
        golden.expectations.width = Some(21.0);
        golden.expectations.children[0].height = Some(6.0);
        let count = collect_numeric_case(&golden, &case, &mut output).unwrap();
        let rows: Vec<serde_json::Value> = std::str::from_utf8(&output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let expected: Vec<_> = [
            ("numeric-transport", "x", 1.0, 0.0),
            ("numeric-transport", "width", 21.0, 20.0),
            ("numeric-transport/0", "height", 6.0, 5.0),
        ]
        .into_iter()
        .map(|(path, field, browser, native)| {
            serde_json::json!({
                "kind": "numeric-mismatch", "case": case.case.id(),
                "source": case.source.as_str(), "variant": "content_box_ltr",
                "path": path, "field": field, "browser": browser, "native": native,
            })
        })
        .collect();
        assert_eq!(rows, expected);
        let mut diagnostics = NumericDiagnostics::default();
        diagnostics.record_case(count).unwrap();
        assert!(diagnostics.finish(1, &[]).is_err());
        let mut matching = NumericDiagnostics::default();
        matching.record_case(0).unwrap();
        matching.finish(1, &[]).unwrap();
    }

    #[test]
    fn numeric_diagnostics_never_emit_partial_records_for_fatal_cases() {
        let mut golden = fixture_golden();
        golden.expectations.x = Some(1.0);
        golden
            .expectations
            .children
            .push(golden.expectations.children[0].clone());
        let mut output = Vec::new();
        let error = collect_numeric_case(&golden, &case(), &mut output).unwrap_err();
        assert_eq!(
            error.to_string(),
            "numeric-transport: expected 2 children, got 1"
        );
        assert!(output.is_empty());
        let mut unsupported = fixture_golden();
        unsupported.expectations.children[0].fragments = Some(Vec::new());
        assert_eq!(
            collect_numeric_case(&unsupported, &case(), &mut output)
                .unwrap_err()
                .to_string(),
            "numeric-transport/0: numeric collection supports only box geometry and scroll observations"
        );
        assert!(output.is_empty());
        let diagnostics = NumericDiagnostics::default();
        assert!(diagnostics.finish(1, &[error.to_string()]).is_err());
        assert!(diagnostics.finish(1, &[]).is_err());
    }
}

#[cfg(test)]
mod custody_applicability_tests {
    use super::*;

    fn declaration() -> Declaration {
        Declaration::parse(DECLARATION).unwrap()
    }

    fn source(named: bool) -> String {
        declaration()
            .records
            .into_iter()
            .find(|record| {
                record.relation.is_none()
                    && matches!(&record.expected, ExpectedDisposition::Generated { .. })
                    && declared_named_font_counts(record).is_some() == named
            })
            .expect("actual declared Generated source for this custody profile")
            .source
            .as_str()
            .to_owned()
    }

    #[test]
    fn custody_modes_require_one_actual_declared_generated_fixture() {
        for (mode, expected) in [
            (
                QualificationMode::StylesheetCustody,
                "stylesheet custody qualification requires one declared fixture",
            ),
            (
                QualificationMode::FontCustody,
                "font custody qualification requires one declared fixture",
            ),
        ] {
            assert_eq!(
                select_records(&mut declaration(), None, mode)
                    .unwrap_err()
                    .to_string(),
                expected
            );
        }
        let unsupported = declaration()
            .records
            .into_iter()
            .find(|record| {
                record.relation.is_none()
                    && matches!(&record.expected, ExpectedDisposition::Unsupported { .. })
            })
            .unwrap()
            .source
            .as_str()
            .to_owned();
        assert_eq!(
            select_records(
                &mut declaration(),
                Some(&unsupported),
                QualificationMode::FontCustody
            )
            .unwrap_err()
            .to_string(),
            "font custody qualification requires a Generated fixture"
        );
        assert_eq!(
            select_records(
                &mut declaration(),
                Some("html/undeclared.html"),
                QualificationMode::FontCustody
            )
            .unwrap_err()
            .to_string(),
            "focused qualification requires one declared source"
        );
    }

    #[test]
    fn destructive_stylesheet_checks_refuse_declared_named_font_custody() {
        let named = source(true);
        assert_eq!(
            select_records(
                &mut declaration(),
                Some(&named),
                QualificationMode::StylesheetCustody
            )
            .unwrap_err()
            .to_string(),
            "stylesheet custody excludes declared named-font sources; use --font-custody"
        );
        let mut selected = declaration();
        select_records(&mut selected, Some(&named), QualificationMode::FontCustody).unwrap();
        assert!(declared_named_font_counts(&selected.records[0]).is_some());
        assert_eq!(selected.expected().unwrap().generated(), 4);
        assert_eq!(selected.expected().unwrap().unsupported(), 0);
        for mode in [
            QualificationMode::Regular,
            QualificationMode::NumericDiagnostics,
        ] {
            let mut selected = declaration();
            select_records(&mut selected, Some(&named), mode).unwrap();
            assert_eq!(selected.expected().unwrap().generated(), 4);
        }
    }

    #[test]
    fn font_custody_refuses_a_declaration_without_named_certificates() {
        let unnamed = source(false);
        assert_eq!(
            select_records(
                &mut declaration(),
                Some(&unnamed),
                QualificationMode::FontCustody
            )
            .unwrap_err()
            .to_string(),
            "font custody requires a declared named-font source"
        );
        let mut selected = declaration();
        select_records(
            &mut selected,
            Some(&unnamed),
            QualificationMode::StylesheetCustody,
        )
        .unwrap();
        assert_eq!(selected.expected().unwrap().generated(), 4);
        assert_eq!(selected.expected().unwrap().unsupported(), 0);
    }

    #[test]
    fn custody_modes_keep_the_existing_complete_pair_requirement() {
        let paired = declaration()
            .records
            .into_iter()
            .find(|record| record.relation.is_some())
            .unwrap()
            .source
            .as_str()
            .to_owned();
        for mode in [
            QualificationMode::StylesheetCustody,
            QualificationMode::FontCustody,
        ] {
            assert_eq!(
                select_records(&mut declaration(), Some(&paired), mode)
                    .unwrap_err()
                    .to_string(),
                "paired observations require the complete inventory"
            );
        }
    }
}

#[cfg(test)]
#[path = "prepared_range_tests.rs"]
mod prepared_range_tests;

#[cfg(test)]
#[path = "flex_composition_tests.rs"]
mod flex_composition_tests;

#[cfg(test)]
#[path = "focused_declaration_tests.rs"]
mod focused_declaration_tests;

#[cfg(test)]
#[path = "focused_contract_tests.rs"]
mod focused_contract_tests;
