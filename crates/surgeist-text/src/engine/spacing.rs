//! Actual separator-unit geometry shared by authored and selected spacing.

mod cell;
pub(crate) mod character;
use super::scope_alignment::accounting::{self as account, Ledger};
use crate::processing::contextual::alignment::operations;
pub(crate) use cell::{CellWordExecution, CellWordPlan};

use std::{collections::BTreeSet, ops::Range as AtomRange};

use icu_properties::{CodePointMapData, props::GeneralCategory};

use super::{
    AnalyzedData,
    analysis::{Analysis, unit_properties},
    layout::data::{PreparedCluster, ShapedData, ShapedItem},
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

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WordSpacingPlacement {
    pub(crate) half: f64,
    pub(crate) unit_start: bool,
}

pub(crate) fn authored_original(
    analyzed: &AnalyzedData,
    styles: &[ResolvedStyleRun],
    shaped: &mut ShapedData,
    ledger: &mut Ledger,
) -> Result<()> {
    let mut cells = analyzed
        .combination
        .as_ref()
        .into_iter()
        .flat_map(|output| output.bindings.iter())
        .peekable();
    apply(
        AuthoredInput {
            projection: &analyzed.projection,
            analysis: &analyzed.analysis,
            styles,
            additional: &BTreeSet::new(),
        },
        shaped,
        Demand {
            stage: SpacingStage::Preparation,
            feedback: false,
        },
        |atoms| {
            let mut pieces = Vec::new();
            for atom in atoms {
                original_pieces(analyzed, atom, &mut pieces, &mut None)?;
            }
            Ok(pieces)
        },
        |atoms| {
            while let Some(cell) = cells.peek() {
                account::work(ledger, 1)?;
                if cell.atoms.end > atoms.start {
                    break;
                }
                cells.next();
            }
            Ok(cells
                .peek()
                .is_some_and(|cell| cell.atoms.start < atoms.end))
        },
    )
}

pub(crate) fn authored_selected(
    input: PresentationInput<'_>,
    shaped: &mut ShapedData,
    feedback: bool,
    ledger: &mut Ledger,
) -> Result<()> {
    let has_cells = !shaped.cells.is_empty();
    let mut units = input.presentation.units.iter().peekable();
    apply(
        AuthoredInput {
            projection: &input.presentation.projection,
            analysis: &input.presentation.analysis,
            styles: &input.prepared.styles,
            additional: &input.boundaries.presentation,
        },
        shaped,
        Demand {
            stage: SpacingStage::SelectedShaping,
            feedback,
        },
        |atoms| {
            Ok(slot_pieces(input.prepared, input.presentation, atoms)
                .into_iter()
                .map(|piece| piece.owner)
                .collect())
        },
        |atoms| {
            if !has_cells {
                return Ok(false);
            }
            while let Some(unit) = units.peek() {
                account::work(ledger, 1)?;
                if unit.atoms.end > atoms.start {
                    break;
                }
                units.next();
            }
            // A Cell is one outer unit; its retained inner execution is never
            // rebuilt into outer authored/feedback word slots.
            Ok(units.peek().is_some_and(|unit| {
                unit.atoms.start < atoms.end
                    && matches!(unit.kind, PresentationUnitKind::Cell { .. })
            }))
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
            original_pieces(&prepared.analyzed, original, &mut owners, &mut None)
                .expect("Native source-piece collection has no scope admission");
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
    let mut result = Vec::new();
    visit_candidates(
        projection,
        analysis,
        additional,
        &mut None,
        |candidate, _| {
            result.push(candidate);
            Ok(())
        },
    )?;
    Ok(result)
}

/// One canonical grapheme/barrier walk for Native and funded Cell inventory.
fn visit_candidates(
    projection: &SourceProjection,
    analysis: &Analysis,
    additional: &BTreeSet<usize>,
    ledger: &mut Ledger,
    mut visit: impl FnMut(SeparatorCandidate, &mut Ledger) -> Result<()>,
) -> Result<()> {
    let mut original = projection.barrier_atoms.iter().copied().peekable();
    let mut extra = additional.iter().copied().peekable();
    for grapheme in &analysis.graphemes {
        account::work(ledger, 1)?;
        let mut first = grapheme.start;
        loop {
            // Sorted genuine cuts are merged without allocating a copied set.
            account::work(ledger, 1)?;
            let next = match (original.peek(), extra.peek()) {
                (Some(a), Some(b)) => Some((*a).min(*b)),
                (Some(a), None) | (None, Some(a)) => Some(*a),
                (None, None) => None,
            };
            let end = if let Some(cut) = next.filter(|cut| *cut < grapheme.end) {
                if original.peek() == Some(&cut) {
                    original.next();
                }
                if extra.peek() == Some(&cut) {
                    extra.next();
                }
                if cut <= first {
                    continue;
                }
                cut
            } else {
                grapheme.end
            };
            let atoms = first..end;
            if let Some(base) = separator_base(projection, atoms.clone(), ledger)? {
                visit(SeparatorCandidate { atoms, base }, ledger)?;
            }
            first = end;
            if end == grapheme.end {
                break;
            }
        }
    }
    Ok(())
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

fn original_pieces(
    analyzed: &AnalyzedData,
    atom: usize,
    output: &mut Vec<SpacingPiece>,
    ledger: &mut Ledger,
) -> Result<()> {
    account::work(ledger, 1)?;
    let relation = &analyzed.transform;
    let component = &relation.components[relation.emissions[relation.atoms[atom]].component];
    match component.owner {
        ComponentOwner::Original(row) => {
            for piece in &relation.pieces[relation.rows[row].pieces.clone()] {
                account::work(ledger, 1)?;
                push_piece_accounted(
                    output,
                    SpacingPiece::Source {
                        origin: analyzed
                            .paragraph
                            .origin_map
                            .visit_view_origin(piece.origin, || account::work(ledger, 1))?
                            .expect("spacing piece retains its actual view origin")
                            .get(),
                        local: piece.local,
                        aggregate: piece.aggregate,
                    },
                    ledger,
                )?;
            }
        }
        ComponentOwner::Wbr(occurrence) => {
            push_piece_accounted(output, SpacingPiece::Control { occurrence }, ledger)?
        }
        ComponentOwner::Virtual(occurrence) => push_piece_accounted(
            output,
            SpacingPiece::VirtualSeparator { occurrence },
            ledger,
        )?,
    }
    Ok(())
}

fn push_piece_accounted(
    output: &mut Vec<SpacingPiece>,
    piece: SpacingPiece,
    ledger: &mut Ledger,
) -> Result<()> {
    account::work(ledger, 1)?;
    if output.last() != Some(&piece) {
        account::push(ledger, output, piece)?;
    }
    Ok(())
}

/// The declared native repertoire consumes the shared Appendix-E property
/// resolution. Its Zs/Po bases cease to qualify when that owner changes category.
fn separator_base(
    projection: &SourceProjection,
    atoms: AtomRange<usize>,
    ledger: &mut Ledger,
) -> Result<Option<usize>> {
    let unit = &projection.atoms[atoms.clone()];
    account::work(ledger, unit.len())?;
    let Some(base) = unit_properties::base(unit) else {
        return Ok(None);
    };
    if !matches!(
        base.code_point,
        0x20 | 0xa0 | 0x1361 | 0x10100 | 0x10101 | 0x1039f | 0x1091f
    ) {
        return Ok(None);
    }
    // Only category is demanded. The canonical observer avoids creating an
    // unused script Vec under a funded separator inventory.
    let properties = unit_properties::visit_properties(unit, |_| account::work(ledger, 1))?;
    if properties.category != CodePointMapData::<GeneralCategory>::new().get32(base.code_point) {
        return Ok(None);
    }
    account::work(ledger, unit.len())?;
    Ok(unit
        .iter()
        .position(|atom| std::ptr::eq(atom, base))
        .map(|index| atoms.start + index))
}

struct AuthoredInput<'a> {
    projection: &'a SourceProjection,
    analysis: &'a Analysis,
    styles: &'a [ResolvedStyleRun],
    additional: &'a BTreeSet<usize>,
}

fn apply(
    input: AuthoredInput<'_>,
    shaped: &mut ShapedData,
    demand: Demand,
    content: impl Fn(AtomRange<usize>) -> Result<Vec<SpacingPiece>>,
    mut excluded: impl FnMut(&AtomRange<usize>) -> Result<bool>,
) -> Result<()> {
    if !demand.feedback && !authored_demand(input.analysis, input.styles) {
        return Ok(());
    }
    // This is the established Native allocation owner. Cell packet scratch has
    // its own funded execution below; it does not adopt this ordinary arena.
    let mut clusters = vec![None; input.projection.atoms.len()];
    for (index, cluster) in shaped.clusters.iter().enumerate() {
        for atom in cluster.atoms.clone() {
            clusters[atom] = Some(index);
        }
    }
    for candidate in candidates(input.projection, input.analysis, input.additional)? {
        if excluded(&candidate.atoms)? {
            continue;
        }
        let authored = input.styles[input.analysis.infos[candidate.base].1]
            .style
            .word_spacing;
        if authored == 0.0 && !demand.feedback {
            continue;
        }
        let members: Vec<_> = candidate
            .atoms
            .clone()
            .filter_map(|atom| clusters[atom])
            .collect();
        let unit = authored_unit_accounted(
            shaped,
            &candidate,
            &members,
            authored,
            demand.stage,
            |_| content(candidate.atoms.clone()),
            &mut None,
        )?;
        shaped.word_spacing.push(unit);
    }
    Ok(())
}

fn authored_unit_accounted(
    shaped: &mut ShapedData,
    candidate: &SeparatorCandidate,
    members: &[usize],
    mut authored: f64,
    stage: SpacingStage,
    content: impl FnOnce(&mut Ledger) -> Result<Vec<SpacingPiece>>,
    ledger: &mut Ledger,
) -> Result<WordSpacingUnit> {
    let (geometry, shared) = geometry_accounted(shaped, members, candidate.atoms.clone(), ledger)?;
    let intrinsic = match geometry {
        crate::SpacingSlotGeometry::Independent { intrinsic_advance } => intrinsic_advance,
        crate::SpacingSlotGeometry::SharedProviderGroup => 0.0,
        crate::SpacingSlotGeometry::Boundary(_) => unreachable!("separator geometry"),
    };
    if let Some(provider_atoms) = &shared {
        if authored != 0.0 {
            // Admit the actual error-owner array before the existing typed
            // diagnostic constructs its one provider interval.
            let pieces = content(ledger)?;
            if let Some(budget) = ledger {
                let bytes = pieces
                    .len()
                    .checked_mul(std::mem::size_of::<SpacingPiece>())
                    .and_then(|n| n.checked_add(std::mem::size_of::<AtomRange<usize>>()))
                    .ok_or_else(|| {
                        super::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
                            kind: crate::ScopeAlignmentLimitKind::PayloadBytes,
                        })
                    })?;
                // into_boxed_slice may coexist with the funded source Vec.
                budget.retain_bytes(bytes)?;
                budget.add_records(1)?;
                budget.work(pieces.len().saturating_add(1))?;
            }
            return Err(unsupported(
                stage,
                SpacingOperation::AuthoredWord,
                pieces,
                provider_atoms.clone(),
            ));
        }
    } else if intrinsic == 0.0 {
        authored = 0.0;
    }
    let mut components = Vec::new();
    for &cluster in members {
        account::work(ledger, 1)?;
        account::push(
            ledger,
            &mut components,
            WordSpacingComponent {
                cluster,
                advance: shaped.clusters[cluster].advance,
            },
        )?;
    }
    let last = components
        .last()
        .expect("separator has actual components")
        .cluster;
    shaped.clusters[last].advance = super::layout::add(
        "word spacing",
        TextNumericPhase::Shaping,
        shaped.clusters[last].advance,
        authored,
    )?;
    shaped.clusters[last].logical_adjustments[1] = authored;
    Ok(WordSpacingUnit {
        atoms: candidate.atoms.clone(),
        base: candidate.base,
        components,
        authored,
        allocation: 0.0,
        geometry,
        shared,
    })
}

pub(crate) fn geometry_accounted(
    shaped: &ShapedData,
    members: &[usize],
    atoms: AtomRange<usize>,
    ledger: &mut Ledger,
) -> Result<(crate::SpacingSlotGeometry, Option<AtomRange<usize>>)> {
    let mut membership = Vec::new();
    for &index in members {
        account::work(ledger, 1)?;
        let cluster = &shaped.clusters[index];
        if cluster.content.font_run().is_some() {
            let receipt = cluster.provider_receipt.ok_or_else(|| {
                Error::new(
                    ErrorCode::LayoutFailed,
                    "separator component lacks its actual provider receipt",
                )
            })?;
            account::work(ledger, membership.len())?;
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
        account::work(ledger, 1)?;
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

fn half(amount: f64, phase: TextNumericPhase) -> Result<f64> {
    let half = amount * 0.5;
    if amount != 0.0 && half == 0.0 {
        return Err(super::text_numeric_error(
            "word spacing half",
            phase,
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
    let halves = visual_units_accounted(
        &mut shaped.clusters,
        0,
        &shaped.word_spacing,
        &rank,
        dispositions,
        TextNumericPhase::Finalization,
        ledger,
    )?;
    account::drop_vec(ledger, rank)?;
    Ok(halves)
}

/// Shared attribution over a genuine cluster slice and its actual L2 ranks.
fn visual_units_accounted(
    clusters: &mut [PreparedCluster],
    first_cluster: usize,
    units: &[WordSpacingUnit],
    ranks: &[usize],
    dispositions: &[SpacingSlotDisposition],
    phase: TextNumericPhase,
    ledger: &mut Ledger,
) -> Result<Vec<WordSpacingPlacement>> {
    if ranks.len() != clusters.len() || dispositions.len() != clusters.len() {
        return Err(invalid_word_execution());
    }
    let mut halves = account::filled(ledger, clusters.len(), WordSpacingPlacement::default())?;
    for unit in units {
        let extra = super::layout::add(
            "separator spacing total",
            phase,
            unit.authored,
            unit.allocation,
        )?;
        let unit_half = half(extra, phase)?;
        let mut trailing: Option<usize> = None;
        let mut leading: Option<usize> = None;
        for component in &unit.components {
            let index = component
                .cluster
                .checked_sub(first_cluster)
                .filter(|index| *index < clusters.len())
                .ok_or_else(invalid_word_execution)?;
            if dispositions[index] == SpacingSlotDisposition::DiscardedAtLineEdge {
                continue;
            }
            clusters[index].advance = super::layout::add(
                "separator tracking cell",
                phase,
                component.advance,
                clusters[index].logical_adjustments[0],
            )?;
            clusters[index].logical_adjustments[1] = 0.0;
            halves[index].half = unit_half;
            if leading.is_none_or(|old| ranks[index] < ranks[old]) {
                leading = Some(index);
            }
            if trailing.is_none_or(|old| ranks[old] < ranks[index]) {
                trailing = Some(index);
            }
        }
        if let Some(index) = leading {
            halves[index].unit_start = true;
        }
        if let Some(index) = trailing {
            clusters[index].advance =
                super::layout::add("visual word spacing", phase, clusters[index].advance, extra)?;
            clusters[index].logical_adjustments[1] = unit.authored;
            clusters[index].logical_adjustments[2] = unit.allocation;
        }
    }
    Ok(halves)
}

fn invalid_word_execution() -> Error {
    Error::new(
        ErrorCode::LayoutFailed,
        "word spacing does not match its actual cluster interval",
    )
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
