//! Actual separator-unit geometry shared by authored and selected spacing.

use std::{
    collections::{BTreeMap, BTreeSet},
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
    Error, ErrorCode, ErrorDetail, Range, Result, SpacingSlotDisposition, TextNumericError,
    TextNumericPhase, WordSpacingOperation, WordSpacingPiece, WordSpacingStage,
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
    pub(crate) components: Vec<WordSpacingComponent>,
    pub(crate) authored: f64,
    pub(crate) half: f64,
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
        WordSpacingStage::Preparation,
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
) -> Result<()> {
    apply(
        &input.presentation.projection,
        &input.presentation.analysis,
        &input.prepared.styles,
        &input.boundaries.presentation,
        shaped,
        WordSpacingStage::SelectedShaping,
        |atoms| {
            let mut pieces = Vec::new();
            for atom in atoms {
                if let Some(original) = input.presentation.originals[atom] {
                    original_pieces(&input.prepared.analyzed, original, &mut pieces);
                    continue;
                }
                let unit = input
                    .presentation
                    .units
                    .iter()
                    .find(|unit| unit.atoms.contains(&atom))
                    .expect("selected atom has an actual presentation owner");
                match unit.kind {
                    PresentationUnitKind::Replacement { alternative, local } => {
                        let range = input.presentation.projection.atoms[atom].presentation;
                        push_piece(
                            &mut pieces,
                            WordSpacingPiece::Alternative {
                                alternative,
                                local: Range::new(
                                    local.start + range.start - unit.presentation.start,
                                    local.start + range.end - unit.presentation.start,
                                ),
                                insertion_anchor: input.prepared.tables.alternatives[alternative]
                                    .insertion_anchor,
                            },
                        );
                    }
                    PresentationUnitKind::GeneratedSeparator { owner } => {
                        use crate::transform::relation::GeneratedSeparatorOwner;
                        push_piece(
                            &mut pieces,
                            match owner {
                                GeneratedSeparatorOwner::Control(occurrence) => {
                                    WordSpacingPiece::Control { occurrence }
                                }
                                GeneratedSeparatorOwner::Virtual(occurrence) => {
                                    WordSpacingPiece::VirtualSeparator { occurrence }
                                }
                            },
                        );
                    }
                    _ => unreachable!("selected nonoriginal scalar has an actual inserted owner"),
                }
            }
            pieces
        },
    )
}

fn original_pieces(analyzed: &AnalyzedData, atom: usize, output: &mut Vec<WordSpacingPiece>) {
    let relation = &analyzed.transform;
    let component = &relation.components[relation.emissions[relation.atoms[atom]].component];
    match component.owner {
        ComponentOwner::Original(row) => {
            for piece in &relation.pieces[relation.rows[row].pieces.clone()] {
                push_piece(
                    output,
                    WordSpacingPiece::Source {
                        origin: piece.origin,
                        local: piece.local,
                        aggregate: piece.aggregate,
                    },
                );
            }
        }
        ComponentOwner::Wbr(occurrence) => {
            push_piece(output, WordSpacingPiece::Control { occurrence })
        }
        ComponentOwner::Virtual(occurrence) => {
            push_piece(output, WordSpacingPiece::VirtualSeparator { occurrence })
        }
    }
}

fn push_piece(output: &mut Vec<WordSpacingPiece>, piece: WordSpacingPiece) {
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
    stage: WordSpacingStage,
    content: impl Fn(AtomRange<usize>) -> Vec<WordSpacingPiece>,
) -> Result<()> {
    if styles.iter().all(|run| run.style.word_spacing == 0.0) {
        return Ok(());
    }
    let barriers: BTreeSet<_> = projection
        .barrier_atoms
        .iter()
        .copied()
        .chain(additional_barriers.iter().copied())
        .collect();
    let mut clusters = vec![None; projection.atoms.len()];
    for (index, cluster) in shaped.clusters.iter().enumerate() {
        for atom in cluster.atoms.clone() {
            clusters[atom] = Some(index);
        }
    }
    let groups: BTreeMap<_, _> = shaped
        .provider_groups
        .iter()
        .enumerate()
        .map(|(index, group)| ((group.run, group.atoms.start, group.atoms.end), index))
        .collect();
    for grapheme in &analysis.graphemes {
        let mut first = grapheme.start;
        for end in barriers
            .range((Excluded(grapheme.start), Excluded(grapheme.end)))
            .copied()
            .chain(std::iter::once(grapheme.end))
        {
            let atoms = first..end;
            first = end;
            let Some(base) = separator_base(projection, atoms.clone())? else {
                continue;
            };
            let authored = styles[analysis.infos[base].1].style.word_spacing;
            if authored == 0.0 {
                continue;
            }
            let members: Vec<_> = atoms.clone().filter_map(|atom| clusters[atom]).collect();
            let mut membership = BTreeSet::new();
            for &index in &members {
                let cluster = &shaped.clusters[index];
                if let Some(run) = cluster.content.font_run() {
                    let group = groups
                        .get(&(
                            run,
                            cluster.provider_group.start,
                            cluster.provider_group.end,
                        ))
                        .ok_or_else(|| {
                            Error::new(
                                ErrorCode::LayoutFailed,
                                "separator component lacks its actual provider receipt",
                            )
                        })?;
                    membership.insert(*group);
                }
            }
            let mut intrinsic = 0.0;
            for index in membership {
                let group = &shaped.provider_groups[index];
                if group.atoms.start < atoms.start || group.atoms.end > atoms.end {
                    return Err(Error::new(
                        ErrorCode::LayoutFailed,
                        "word spacing requires independent separator geometry",
                    )
                    .with_detail(ErrorDetail::UnsupportedWordSpacing {
                        stage,
                        operation: WordSpacingOperation::Authored,
                        pieces: content(atoms.clone()).into_boxed_slice(),
                        provider_atoms: group.atoms.clone(),
                    }));
                }
                assert!(
                    group.glyphs.end <= shaped.glyphs.len(),
                    "provider receipt retains its actual glyph inventory"
                );
                intrinsic = super::layout::add(
                    "separator intrinsic advance",
                    TextNumericPhase::Shaping,
                    intrinsic,
                    group.advance,
                )?;
            }
            if intrinsic == 0.0 {
                continue;
            }
            let half = half(authored)?;
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
                components,
                authored,
                half,
            });
        }
    }
    Ok(())
}

fn half(amount: f64) -> Result<f64> {
    let half = amount * 0.5;
    if amount != 0.0 && half == 0.0 {
        return Err(super::text_numeric_error(
            "word spacing half",
            TextNumericPhase::Shaping,
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
    let mut rank = vec![0; shaped.clusters.len()];
    for (position, (ordinal, _)) in order.iter().enumerate() {
        if let ShapedItem::TextCluster(cluster) = shaped.items[ordinal] {
            rank[cluster] = position;
        }
    }
    let mut halves = vec![WordSpacingPlacement::default(); shaped.clusters.len()];
    for unit in &shaped.word_spacing {
        let mut trailing = None;
        let mut leading = None;
        for component in &unit.components {
            let index = component.cluster;
            if dispositions[index] == SpacingSlotDisposition::DiscardedAtLineEdge {
                continue;
            }
            shaped.clusters[index].advance = component.advance;
            shaped.clusters[index].logical_adjustments[1] = 0.0;
            halves[index].half = unit.half;
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
                unit.authored,
            )?;
            shaped.clusters[index].logical_adjustments[1] = unit.authored;
        }
    }
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
