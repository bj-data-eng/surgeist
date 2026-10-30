//! Actual separator-unit geometry shared by authored and selected spacing.

pub(crate) mod character;
use super::scope_alignment::accounting::{self as account, Ledger};
use crate::processing::contextual::alignment::operations;

use std::{
    collections::BTreeSet,
    ops::{Bound::Excluded, Range as AtomRange},
};

use icu_properties::{CodePointMapData, props::GeneralCategory};

use super::{
    AnalyzedData,
    analysis::{Analysis, unit_properties},
    layout::data::{ShapedData, ShapedItem},
    resolve::ResolvedStyleRun,
};
use crate::{
    Error, ErrorCode, ErrorDetail, Range, Result, SpacingOperation, SpacingPiece,
    SpacingSlotDisposition, SpacingStage, TextNumericError, TextNumericPhase,
    projection::SourceProjection,
    protocol::{input::PresentationInput, prepared::PresentationUnitKind},
    transform::relation::ComponentOwner,
};

/// Independent component geometry before the unit's authored word term.
#[derive(Clone, Debug)]
pub(crate) struct WordSpacingComponent {
    pub(crate) cluster: usize,
    pub(crate) advance: f64,
}

/// A complete actual separator TCU, rather than a scalar advance share.
#[derive(Clone, Debug)]
pub(crate) struct WordSpacingUnit {
    pub(crate) atoms: AtomRange<usize>,
    pub(crate) base: usize,
    pub(crate) components: Vec<WordSpacingComponent>,
    pub(crate) authored: f64,
    pub(crate) allocation: f64,
    pub(crate) geometry: crate::SpacingSlotGeometry,
    pub(crate) shared: Option<AtomRange<usize>>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct WordSpacingPlacement {
    pub(crate) half: f64,
    pub(crate) unit_start: bool,
}

pub(crate) fn authored_original(
    analyzed: &AnalyzedData,
    styles: &[ResolvedStyleRun],
    shaped: &mut ShapedData,
) -> Result<()> {
    apply(
        &analyzed.projection,
        &analyzed.analysis,
        styles,
        &BTreeSet::new(),
        shaped,
        Demand {
            stage: SpacingStage::Preparation,
            feedback: false,
        },
        |atoms| {
            let mut pieces = Vec::new();
            for atom in atoms {
                original_pieces(analyzed, atom, &mut pieces);
            }
            pieces
        },
    )
}

pub(crate) fn authored_selected(
    input: PresentationInput<'_>,
    shaped: &mut ShapedData,
    feedback: bool,
) -> Result<()> {
    apply(
        &input.presentation.projection,
        &input.presentation.analysis,
        &input.prepared.styles,
        &input.boundaries.presentation,
        shaped,
        Demand {
            stage: SpacingStage::SelectedShaping,
            feedback,
        },
        |atoms| {
            slot_pieces(input.prepared, input.presentation, atoms)
                .into_iter()
                .map(|piece| piece.owner)
                .collect()
        },
    )
}

/// Exact selected atom correspondence, shared by errors and strong slot views.
pub(crate) fn slot_pieces(
    prepared: &super::PreparedData,
    presentation: &crate::protocol::prepared::SelectedPresentation,
    atoms: AtomRange<usize>,
) -> Vec<crate::protocol::selection::SpacingSlotPieceData> {
    let mut output = Vec::new();
    for atom in atoms {
        let mut owners = Vec::new();
        if let Some(original) = presentation.originals[atom] {
            original_pieces(&prepared.analyzed, original, &mut owners);
        } else {
            let index = presentation
                .units
                .partition_point(|unit| unit.atoms.end <= atom);
            let unit = &presentation.units[index];
            assert!(
                unit.atoms.contains(&atom),
                "selected atom has its actual owner"
            );
            match unit.kind {
                PresentationUnitKind::Replacement { alternative, local } => {
                    let range = presentation.projection.atoms[atom].presentation;
                    owners.push(SpacingPiece::Alternative {
                        alternative,
                        local: Range::new(
                            local.start + range.start - unit.presentation.start,
                            local.start + range.end - unit.presentation.start,
                        ),
                        insertion_anchor: prepared.tables.alternatives[alternative]
                            .insertion_anchor,
                    });
                }
                PresentationUnitKind::GeneratedSeparator { owner } => {
                    use crate::transform::relation::GeneratedSeparatorOwner;
                    owners.push(match owner {
                        GeneratedSeparatorOwner::Control(occurrence) => {
                            SpacingPiece::Control { occurrence }
                        }
                        GeneratedSeparatorOwner::Virtual(occurrence) => {
                            SpacingPiece::VirtualSeparator { occurrence }
                        }
                    });
                }
                _ => unreachable!("inserted scalar has an actual inserted owner"),
            }
        }
        let range = presentation.projection.atoms[atom].presentation;
        for owner in owners {
            output.push(crate::protocol::selection::SpacingSlotPieceData {
                owner,
                presentation: crate::PresentationRange {
                    start: range.start,
                    end: range.end,
                },
            });
        }
    }
    output
}

#[derive(Clone, Debug)]
pub(crate) struct SeparatorCandidate {
    pub(crate) atoms: AtomRange<usize>,
    pub(crate) base: usize,
}

pub(crate) fn candidates(
    projection: &SourceProjection,
    analysis: &Analysis,
    additional: &BTreeSet<usize>,
) -> Result<Vec<SeparatorCandidate>> {
    let barriers: BTreeSet<_> = projection
        .barrier_atoms
        .iter()
        .copied()
        .chain(additional.iter().copied())
        .collect();
    let mut result = Vec::new();
    for grapheme in &analysis.graphemes {
        let mut first = grapheme.start;
        for end in barriers
            .range((Excluded(grapheme.start), Excluded(grapheme.end)))
            .copied()
            .chain(std::iter::once(grapheme.end))
        {
            let atoms = first..end;
            first = end;
            if let Some(base) = separator_base(projection, atoms.clone())? {
                result.push(SeparatorCandidate { atoms, base });
            }
        }
    }
    Ok(result)
}

pub(crate) fn authored_demand(analysis: &Analysis, styles: &[ResolvedStyleRun]) -> bool {
    analysis
        .infos
        .iter()
        .any(|(_, style)| styles[*style].style.word_spacing != 0.0)
}

struct Demand {
    stage: SpacingStage,
    feedback: bool,
}

fn original_pieces(analyzed: &AnalyzedData, atom: usize, output: &mut Vec<SpacingPiece>) {
    let relation = &analyzed.transform;
    let component = &relation.components[relation.emissions[relation.atoms[atom]].component];
    match component.owner {
        ComponentOwner::Original(row) => {
            for piece in &relation.pieces[relation.rows[row].pieces.clone()] {
                push_piece(
                    output,
                    SpacingPiece::Source {
                        origin: analyzed
                            .paragraph
                            .origin_map
                            .view_origin(piece.origin)
                            .expect("spacing piece retains its actual view origin")
                            .get(),
                        local: piece.local,
                        aggregate: piece.aggregate,
                    },
                );
            }
        }
        ComponentOwner::Wbr(occurrence) => push_piece(output, SpacingPiece::Control { occurrence }),
        ComponentOwner::Virtual(occurrence) => {
            push_piece(output, SpacingPiece::VirtualSeparator { occurrence })
        }
    }
}

fn push_piece(output: &mut Vec<SpacingPiece>, piece: SpacingPiece) {
    if output.last() != Some(&piece) {
        output.push(piece);
    }
}

/// The declared native repertoire consumes the shared Appendix-E property
/// resolution. Its Zs/Po bases cease to qualify when that owner changes category.
fn separator_base(projection: &SourceProjection, atoms: AtomRange<usize>) -> Result<Option<usize>> {
    let unit = &projection.atoms[atoms.clone()];
    let Some(base) = unit_properties::base(unit) else {
        return Ok(None);
    };
    if !matches!(
        base.code_point,
        0x20 | 0xa0 | 0x1361 | 0x10100 | 0x10101 | 0x1039f | 0x1091f
    ) {
        return Ok(None);
    }
    let properties = unit_properties::properties(unit)?;
    if properties.category != CodePointMapData::<GeneralCategory>::new().get32(base.code_point) {
        return Ok(None);
    }
    Ok(unit
        .iter()
        .position(|atom| std::ptr::eq(atom, base))
        .map(|index| atoms.start + index))
}

fn apply(
    projection: &SourceProjection,
    analysis: &Analysis,
    styles: &[ResolvedStyleRun],
    additional_barriers: &BTreeSet<usize>,
    shaped: &mut ShapedData,
    demand: Demand,
    content: impl Fn(AtomRange<usize>) -> Vec<SpacingPiece>,
) -> Result<()> {
    if !demand.feedback && !authored_demand(analysis, styles) {
        return Ok(());
    }
    let mut clusters = vec![None; projection.atoms.len()];
    for (index, cluster) in shaped.clusters.iter().enumerate() {
        for atom in cluster.atoms.clone() {
            clusters[atom] = Some(index);
        }
    }
    for candidate in candidates(projection, analysis, additional_barriers)? {
        let atoms = candidate.atoms;
        let base = candidate.base;
        let mut authored = styles[analysis.infos[base].1].style.word_spacing;
        if authored == 0.0 && !demand.feedback {
            continue;
        }
        let members: Vec<_> = atoms.clone().filter_map(|atom| clusters[atom]).collect();
        let (geometry, shared) = geometry(shaped, &members, atoms.clone())?;
        let intrinsic = match geometry {
            crate::SpacingSlotGeometry::Independent { intrinsic_advance } => intrinsic_advance,
            crate::SpacingSlotGeometry::SharedProviderGroup => 0.0,
            crate::SpacingSlotGeometry::Boundary(_) => unreachable!("separator geometry"),
        };
        if let Some(provider_atoms) = &shared {
            if authored != 0.0 {
                return Err(unsupported(
                    demand.stage,
                    SpacingOperation::AuthoredWord,
                    content(atoms.clone()),
                    provider_atoms.clone(),
                ));
            }
        } else if intrinsic == 0.0 {
            authored = 0.0;
        }

        let components: Vec<_> = members
            .into_iter()
            .map(|cluster| WordSpacingComponent {
                cluster,
                advance: shaped.clusters[cluster].advance,
            })
            .collect();
        let last = components
            .last()
            .expect("nonzero separator has actual components")
            .cluster;
        shaped.clusters[last].advance = super::layout::add(
            "word spacing",
            TextNumericPhase::Shaping,
            shaped.clusters[last].advance,
            authored,
        )?;
        shaped.clusters[last].logical_adjustments[1] = authored;
        shaped.word_spacing.push(WordSpacingUnit {
            atoms,
            base,
            components,
            authored,
            allocation: 0.0,
            geometry,
            shared,
        });
    }
    Ok(())
}

pub(crate) fn geometry(
    shaped: &ShapedData,
    members: &[usize],
    atoms: AtomRange<usize>,
) -> Result<(crate::SpacingSlotGeometry, Option<AtomRange<usize>>)> {
    geometry_accounted(shaped, members, atoms, &mut None)
}
pub(crate) fn geometry_accounted(
    shaped: &ShapedData,
    members: &[usize],
    atoms: AtomRange<usize>,
    ledger: &mut Ledger,
) -> Result<(crate::SpacingSlotGeometry, Option<AtomRange<usize>>)> {
    let mut membership = Vec::new();
    for &index in members {
        let cluster = &shaped.clusters[index];
        if cluster.content.font_run().is_some() {
            let receipt = cluster.provider_receipt.ok_or_else(|| {
                Error::new(
                    ErrorCode::LayoutFailed,
                    "separator component lacks its actual provider receipt",
                )
            })?;
            account::work(ledger, membership.len().saturating_add(1))?;
            if !membership.contains(&receipt) {
                account::push(ledger, &mut membership, receipt)?;
            }
        }
    }
    let mut intrinsic = 0.0;
    let mut shared = None;
    operations::order_by(&mut membership, ledger.as_mut(), |left, right| {
        left.cmp(right)
    })?;
    for &index in &membership {
        let group = &shaped.provider_groups[index];
        assert!(
            group.run < shaped.runs.len() && group.glyphs.end <= shaped.glyphs.len(),
            "actual provider receipt retains its font and glyphs"
        );
        if group.atoms.start < atoms.start || group.atoms.end > atoms.end {
            shared = Some(group.atoms.clone());
        } else {
            intrinsic = super::layout::add(
                "separator intrinsic advance",
                TextNumericPhase::Shaping,
                intrinsic,
                group.advance,
            )?;
        }
    }
    account::drop_vec(ledger, membership)?;
    Ok((
        if shared.is_some() {
            crate::SpacingSlotGeometry::SharedProviderGroup
        } else {
            crate::SpacingSlotGeometry::Independent {
                intrinsic_advance: intrinsic,
            }
        },
        shared,
    ))
}

pub(crate) fn unsupported(
    stage: SpacingStage,
    operation: SpacingOperation,
    pieces: Vec<SpacingPiece>,
    provider_atoms: AtomRange<usize>,
) -> Error {
    Error::new(
        ErrorCode::LayoutFailed,
        "word spacing requires independent separator geometry",
    )
    .with_detail(ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        pieces: pieces.into_boxed_slice(),
        provider_atoms: vec![provider_atoms].into_boxed_slice(),
        reason: crate::SpacingFailureReason::CrossUnitConstraint,
        contexts: Box::new([]),
        constraints: Box::new([]),
    })
}

fn half(amount: f64) -> Result<f64> {
    let half = amount * 0.5;
    if amount != 0.0 && half == 0.0 {
        return Err(super::text_numeric_error(
            "word spacing half",
            TextNumericPhase::Finalization,
            TextNumericError::NonZeroLost,
            amount,
        ));
    }
    Ok(half)
}

/// Width attribution follows actual visual trailing components; every glyph
/// group receives the same half, independently of raw provider component shares.
pub(crate) fn visual_authored(
    shaped: &mut ShapedData,
    order: &crate::SuppliedVisualOrder<'_, '_>,
    dispositions: &[SpacingSlotDisposition],
) -> Result<Vec<WordSpacingPlacement>> {
    visual_authored_accounted(shaped, order, dispositions, &mut None)
}
pub(crate) fn visual_authored_accounted(
    shaped: &mut ShapedData,
    order: &crate::SuppliedVisualOrder<'_, '_>,
    dispositions: &[SpacingSlotDisposition],
    ledger: &mut Ledger,
) -> Result<Vec<WordSpacingPlacement>> {
    let mut rank = account::filled(ledger, shaped.clusters.len(), 0)?;
    for (position, (ordinal, _)) in order.iter().enumerate() {
        if let ShapedItem::TextCluster(cluster) = shaped.items[ordinal] {
            rank[cluster] = position;
        }
    }
    let mut halves = account::filled(
        ledger,
        shaped.clusters.len(),
        WordSpacingPlacement::default(),
    )?;
    for unit in &shaped.word_spacing {
        let extra = super::layout::add(
            "separator spacing total",
            TextNumericPhase::Finalization,
            unit.authored,
            unit.allocation,
        )?;
        let unit_half = half(extra)?;
        let mut trailing = None;
        let mut leading = None;
        for component in &unit.components {
            let index = component.cluster;
            if dispositions[index] == SpacingSlotDisposition::DiscardedAtLineEdge {
                continue;
            }
            shaped.clusters[index].advance = super::layout::add(
                "separator tracking cell",
                TextNumericPhase::Finalization,
                component.advance,
                shaped.clusters[index].logical_adjustments[0],
            )?;
            shaped.clusters[index].logical_adjustments[1] = 0.0;
            halves[index].half = unit_half;
            if leading.is_none_or(|old| rank[index] < rank[old]) {
                leading = Some(index);
            }
            if trailing.is_none_or(|old| rank[old] < rank[index]) {
                trailing = Some(index);
            }
        }
        if let Some(index) = leading {
            halves[index].unit_start = true;
        }
        if let Some(index) = trailing {
            shaped.clusters[index].advance = super::layout::add(
                "visual word spacing",
                TextNumericPhase::Finalization,
                shaped.clusters[index].advance,
                extra,
            )?;
            shaped.clusters[index].logical_adjustments[1] = unit.authored;
            shaped.clusters[index].logical_adjustments[2] = unit.allocation;
        }
    }
    account::drop_vec(ledger, rank)?;
    Ok(halves)
}

/// The existing real hypothetical SPACE operation uses the same intrinsic-zero
/// gate, without creating authored source or replacing an authored glyph query.
pub(crate) fn hypothetical_space(advance: f64, letter: f64, word: f64) -> Result<f64> {
    let with_letter = super::layout::add(
        "tab block letter spacing",
        TextNumericPhase::Preparation,
        advance,
        letter,
    )?;
    super::layout::add(
        "tab block word spacing",
        TextNumericPhase::Preparation,
        with_letter,
        if advance == 0.0 { 0.0 } else { word },
    )
}
