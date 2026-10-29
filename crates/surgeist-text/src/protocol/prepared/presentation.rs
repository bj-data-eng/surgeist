//! Selected original UBA and indication-local domains, with separate level spaces.
use super::*;
use crate::engine::{
    analysis::{Analysis, CharInfo},
    bidi::{BidiResolver, LineReset, reset_line_levels},
};
use icu_properties::props::BidiClass;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DirectionalScope {
    Outer(u8),
    Local(usize, u8),
}
#[derive(Debug)]
pub(crate) struct IndicationDomain {
    pub(crate) units: IndexRange<usize>,
    pub(crate) outer: u8,
    pub(crate) base: u8,
    pub(crate) attachment: Attachment,
}
/// Authenticated original occurrence/scalar; integers alone are not authority.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Attachment {
    Before {
        participant: usize,
        atom: Option<usize>,
    },
    After {
        participant: usize,
        atom: Option<usize>,
    },
}
impl Attachment {
    pub(super) fn original_level(
        self,
        analyzed: &AnalyzedData,
        tables: &PreparedTables,
    ) -> crate::Result<u8> {
        let (participant, atom) = match self {
            Self::Before { participant, atom } | Self::After { participant, atom } => {
                (participant, atom)
            }
        };
        match tables.participants.get(participant) {
            Some(ParticipantData::Text(i))
                if atom.is_some_and(|a| tables.units[*i].atoms.contains(&a)) =>
            {
                Ok(analyzed.analysis.levels[atom.expect("authenticated scalar")])
            }
            Some(ParticipantData::Control(i)) if atom.is_none() => {
                Ok(analyzed.analysis.control_levels[tables.controls[*i].occurrence])
            }
            Some(ParticipantData::Atomic(i))
                if atom.is_none()
                    && matches!(self, Self::After { .. })
                    && analyzed.processed.source().boxes()[tables.atomics[*i].occurrence]
                        .kind()
                        == crate::InlineBoxKind::InFlow =>
            {
                Ok(analyzed.analysis.box_levels[tables.atomics[*i].occurrence])
            }
            _ => Err(composition_error()),
        }
    }
    fn level(
        self,
        prepared: &crate::engine::PreparedData,
        units: &[PresentationUnit],
        start: usize,
    ) -> crate::Result<u8> {
        let original = self.original_level(&prepared.analyzed, &prepared.tables)?;
        let Self::After { participant, .. } = self else {
            return Ok(original);
        };
        units
            .iter()
            .find(|u| u.participant.and_then(|p| p.checked_add(start)) == Some(participant))
            .map(|u| u.line_level)
            .ok_or_else(|| {
                crate::hyphenation::error(crate::HyphenationError::MissingReceivingBidiAuthority)
            })
    }
}
/// Both preparation's matching profile and selection use the same authentic
/// mixed occurrence. Equal source offsets do not erase a directional control.
pub(super) fn before_attachment(
    analyzed: &AnalyzedData,
    tables: &PreparedTables,
    row: &AlternativeData,
) -> crate::Result<Attachment> {
    if matches!(row.definition, AlternativeDefinition::Automatic { .. }) {
        let end = tables.boundaries[row.boundary].participant;
        for participant in (0..end).rev() {
            match tables.participants[participant] {
                ParticipantData::Text(i) => {
                    if let Some(atom) = tables.units[i].atoms.clone().next_back() {
                        return Ok(Attachment::Before {
                            participant,
                            atom: Some(atom),
                        });
                    }
                }
                ParticipantData::Control(i) => {
                    let kind = &analyzed.paragraph.controls[tables.controls[i].occurrence].kind;
                    if kind.code_point().is_some() {
                        return Ok(Attachment::Before {
                            participant,
                            atom: None,
                        });
                    }
                    if *kind == ParagraphControlKind::ForcedBreak {
                        return Err(composition_error());
                    }
                }
                ParticipantData::ComputedLineBoundary(_) => return Err(composition_error()),
                ParticipantData::Atomic(i) => {
                    if analyzed.processed.source().boxes()[tables.atomics[i].occurrence].kind()
                        == crate::InlineBoxKind::InFlow
                    {
                        return Err(composition_error());
                    }
                }
            }
        }
        return Err(composition_error());
    }
    for (participant, part) in tables.participants.iter().enumerate() {
        match (row.definition, *part) {
            (AlternativeDefinition::Source { source, .. }, ParticipantData::Text(i)) => {
                if let Some(atom) = tables.units[i].atoms.clone().find(|&a| {
                    analyzed.projection.atoms[a].source.start >= source.start
                        && analyzed.projection.atoms[a].source.end <= source.end
                }) {
                    return Ok(Attachment::Before {
                        participant,
                        atom: Some(atom),
                    });
                }
            }
            (AlternativeDefinition::Control { control, .. }, ParticipantData::Control(i))
                if tables.controls[i].occurrence == control =>
            {
                return Ok(Attachment::Before {
                    participant,
                    atom: None,
                });
            }
            _ => {}
        }
    }
    Err(composition_error())
}
pub(super) fn local_levels(
    projection: &SourceProjection,
    analysis: &mut Analysis,
    atoms: IndexRange<usize>,
    base: u8,
    typography: crate::AnalysisTypography,
) -> crate::Result<()> {
    if atoms.clone().any(|i| {
        projection.atoms[i].code_point != 9
            && (matches!(
                analysis.infos[i].0.bidi_class,
                BidiClass::ParagraphSeparator | BidiClass::SegmentSeparator
            ) || crate::engine::analysis::is_hard_separator(projection.atoms[i].code_point))
    }) {
        return Err(crate::hyphenation::error(
            crate::HyphenationError::UnsupportedIndicationSeparator,
        ));
    }
    let mut bidi = BidiResolver::new();
    bidi.resolve(
        atoms.clone().map(|i| {
            (
                projection.atoms[i].glyph,
                (
                    crate::engine::analysis::used_bidi_class(
                        typography,
                        analysis.infos[i].0.bidi_class,
                    ),
                    analysis.infos[i].0.bracket,
                ),
            )
        }),
        Some(base),
    );
    analysis.levels[atoms.clone()].copy_from_slice(bidi.levels());
    let roles = atoms
        .clone()
        .map(|i| {
            LineReset::from_class(crate::engine::analysis::used_bidi_class(
                typography,
                analysis.infos[i].0.bidi_class,
            ))
        })
        .collect::<Vec<_>>();
    reset_line_levels(&mut analysis.levels[atoms], &roles, base);
    Ok(())
}
#[derive(Default)]
struct Output {
    output: PresentationOutput,
    infos: Vec<(CharInfo, usize)>,
    levels: Vec<u8>,
    originals: Vec<Option<usize>>,
    domains: Vec<IndicationDomain>,
}
impl Output {
    fn emit(
        &mut self,
        projection: &SourceProjection,
        analysis: &Analysis,
        atoms: &[usize],
        capture: UnitCapture,
        original: bool,
    ) -> crate::Result<()> {
        self.infos.extend(atoms.iter().map(|&i| analysis.infos[i]));
        self.levels
            .extend(atoms.iter().map(|&i| analysis.levels[i]));
        self.originals
            .extend(atoms.iter().map(|&i| original.then_some(i)));
        push_presentation_unit(&mut self.output, projection, atoms, capture)
    }
    fn before_indication(
        &mut self,
        prepared: &crate::engine::PreparedData,
        alternative: usize,
    ) -> crate::Result<()> {
        let attachment = before_attachment(
            &prepared.analyzed,
            &prepared.tables,
            &prepared.tables.alternatives[alternative],
        )?;
        let outer = attachment.original_level(&prepared.analyzed, &prepared.tables)?;
        self.indication(prepared, alternative, outer, attachment)
    }
    fn indication(
        &mut self,
        prepared: &crate::engine::PreparedData,
        alternative: usize,
        outer: u8,
        attachment: Attachment,
    ) -> crate::Result<()> {
        let variant = &prepared.indication_fonts[alternative];
        let row = &prepared.tables.alternatives[alternative];
        let mut analysis = variant.analysis.clone();
        local_levels(
            &variant.projection,
            &mut analysis,
            variant.inserted_atoms.clone(),
            outer & 1,
            prepared.analyzed.typography,
        )?;
        let first = self.output.units.len();
        let processed = prepared
            .analyzed
            .processed
            .project(row.insertion_anchor, crate::Affinity::After);
        let capture = |level, bidi_class, local| UnitCapture {
            source: Range::new(row.insertion_anchor, row.insertion_anchor),
            processed: Range::new(processed, processed),
            participant: None,
            kind: PresentationUnitKind::Replacement { alternative, local },
            level,
            bidi_class,
        };
        if variant.inserted_atoms.is_empty() {
            self.emit(
                &variant.projection,
                &analysis,
                &[],
                capture(outer, None, Range::new(0, 0)),
                false,
            )?;
        } else {
            for i in variant.inserted_atoms.clone() {
                self.emit(
                    &variant.projection,
                    &analysis,
                    &[i],
                    capture(analysis.levels[i], Some(analysis.infos[i].0.bidi_class), {
                        let start = variant.projection.atoms[variant.inserted_atoms.start]
                            .presentation
                            .start;
                        let range = variant.projection.atoms[i].presentation;
                        Range::new(range.start - start, range.end - start)
                    }),
                    false,
                )?;
            }
            self.domains.push(IndicationDomain {
                units: first..self.output.units.len(),
                outer,
                base: outer & 1,
                attachment,
            });
        }
        Ok(())
    }
}

pub(super) fn select(
    prepared: &crate::engine::PreparedData,
    request: super::super::input::RequestInput<'_>,
) -> crate::Result<SelectedPresentation> {
    let tables = &prepared.tables;
    let start = tables.boundaries[request.start.index];
    let end = tables.boundaries[request.end.index];
    let projection = &prepared.analyzed.projection;
    let analysis = &prepared.analyzed.analysis;
    let alternative = request.alternative.as_ref().map(|value| value.index);
    let definition = alternative.map(|i| tables.alternatives[i].definition);
    let before = alternative.filter(|&i| tables.alternatives[i].placement.includes_before());
    let mut output = Output::default();
    for (local, participant) in tables.participants[start.participant..end.participant]
        .iter()
        .enumerate()
    {
        let defining = match (definition, *participant) {
            (Some(AlternativeDefinition::Source { source, .. }), ParticipantData::Text(i)) => {
                !tables.units[i].deleted
                    && tables.units[i].source.start <= source.start
                    && tables.units[i].source.end >= source.end
            }
            (Some(AlternativeDefinition::Control { control, .. }), ParticipantData::Control(i)) => {
                tables.controls[i].occurrence == control
            }
            _ => false,
        };
        if defining && let ParticipantData::Text(i) = *participant {
            let row = &tables.units[i];
            output.emit(
                projection,
                analysis,
                &[],
                UnitCapture {
                    source: row.source,
                    processed: row.processed,
                    participant: Some(local),
                    kind: PresentationUnitKind::Deleted,
                    level: row.level,
                    bidi_class: row
                        .atoms
                        .clone()
                        .next()
                        .map(|i| analysis.infos[i].0.bidi_class),
                },
                true,
            )?;
        }
        if defining && let Some(i) = before {
            output.before_indication(prepared, i)?;
        }
        match *participant {
            ParticipantData::Text(i) if !defining => {
                let row = &tables.units[i];
                let kind = if let Some(origin) = row.empty_origin {
                    PresentationUnitKind::EmptyOrigin { origin }
                } else if row.deleted {
                    PresentationUnitKind::Deleted
                } else if let Some(control) = prepared
                    .analyzed
                    .transform
                    .controls_for_atoms(row.atoms.clone())
                    .first()
                    .copied()
                {
                    PresentationUnitKind::GeneratedSeparator {
                        owner: GeneratedSeparatorOwner::Control(control),
                    }
                } else if let Some(index) = prepared
                    .analyzed
                    .transform
                    .virtuals_for_atoms(row.atoms.clone())
                    .first()
                    .copied()
                {
                    PresentationUnitKind::GeneratedSeparator {
                        owner: GeneratedSeparatorOwner::Virtual(index),
                    }
                } else {
                    PresentationUnitKind::Text
                };
                let atoms: Vec<_> = row.atoms.clone().collect();
                output.emit(
                    projection,
                    analysis,
                    &atoms,
                    UnitCapture {
                        source: row.source,
                        processed: row.processed,
                        participant: Some(local),
                        kind,
                        level: row.level,
                        bidi_class: atoms.first().map(|&i| analysis.infos[i].0.bidi_class),
                    },
                    true,
                )?;
            }
            ParticipantData::Text(_) => {}
            ParticipantData::Atomic(i) => {
                let row = tables.atomics[i];
                output.emit(
                    projection,
                    analysis,
                    &[],
                    UnitCapture {
                        source: Range::new(row.source, row.source),
                        processed: Range::new(row.processed, row.processed),
                        participant: Some(local),
                        kind: PresentationUnitKind::Atomic {
                            occurrence: row.occurrence,
                        },
                        level: analysis.box_levels[row.occurrence],
                        bidi_class: None,
                    },
                    true,
                )?;
            }
            ParticipantData::ComputedLineBoundary(i) => {
                let row = tables.computed_boundaries[i];
                output.emit(
                    projection,
                    analysis,
                    &[],
                    UnitCapture {
                        source: Range::new(row.source, row.source),
                        processed: Range::new(row.processed, row.processed),
                        participant: Some(local),
                        kind: PresentationUnitKind::ComputedLineBoundary { boundary: i },
                        level: row.level,
                        bidi_class: Some(BidiClass::ParagraphSeparator),
                    },
                    true,
                )?;
            }
            ParticipantData::Control(i) => {
                let row = tables.controls[i];
                let kind = &prepared.analyzed.paragraph.controls[row.occurrence].kind;
                let cp = if *kind == ParagraphControlKind::ForcedBreak {
                    Some(10)
                } else {
                    kind.code_point()
                };
                output.emit(
                    projection,
                    analysis,
                    &[],
                    UnitCapture {
                        source: Range::new(row.source, row.source),
                        processed: Range::new(row.processed, row.processed),
                        participant: Some(local),
                        kind: PresentationUnitKind::Control {
                            occurrence: row.occurrence,
                        },
                        level: analysis.control_levels[row.occurrence],
                        bidi_class: cp.map(|cp| {
                            icu_properties::CodePointMapData::<BidiClass>::new().get32(cp)
                        }),
                    },
                    true,
                )?;
            }
        }
    }
    if matches!(definition, Some(AlternativeDefinition::Automatic { .. }))
        && let Some(i) = before
    {
        output.before_indication(prepared, i)?;
    }
    // Original L1 is computed without generated strings. Only original units
    // supply the receiving-side authority; transparent occurrences are not it.
    reset_originals(
        &mut output.output.units,
        &output.infos,
        prepared.analyzed.processed.source().boxes(),
        analysis.base_level,
    );
    if let Some(incoming) = &request.incoming {
        let receiver_index = output.output.units.iter().position(|unit| {
            ((!unit.atoms.is_empty() && unit.kind == PresentationUnitKind::Text)
                    || (unit.kind == PresentationUnitKind::Deleted && unit.bidi_class.is_some()))
                || matches!(unit.kind, PresentationUnitKind::Atomic { occurrence } if prepared.analyzed.processed.source().boxes()[occurrence].kind() == crate::InlineBoxKind::InFlow)
                || matches!(unit.kind, PresentationUnitKind::Control { .. }) && unit.bidi_class.is_some()
        }).ok_or_else(|| crate::hyphenation::error(crate::HyphenationError::MissingReceivingBidiAuthority))?;
        let receiver = &output.output.units[receiver_index];
        let outer = receiver.line_level;
        let participant = start.participant
            + receiver
                .participant
                .expect("original receiver has actual participant custody");
        let atom = match tables.participants[participant] {
            ParticipantData::Text(i) => tables.units[i].atoms.clone().next(),
            _ => None,
        };
        let attachment = Attachment::After { participant, atom };
        // After uses the actual receiver's event side as well as its full L1
        // level. Genuine prefix controls keep their captured logical order.
        let atom_cut = receiver.atoms.start;
        let position_cut = receiver.presentation.start;
        let mut indication = Output::default();
        indication.indication(prepared, incoming.alternative.index, outer, attachment)?;
        let atom_shift = indication.output.atoms.len();
        let unit_shift = indication.output.units.len();
        let position_shift = indication.output.position;
        let final_position = output
            .output
            .position
            .checked_add(position_shift)
            .ok_or_else(crate::projection::length_overflow)?;
        for atom in &mut output.output.atoms[atom_cut..] {
            atom.presentation.start += position_shift;
            atom.presentation.end += position_shift;
        }
        for unit in &mut output.output.units[receiver_index..] {
            unit.atoms.start += atom_shift;
            unit.atoms.end += atom_shift;
            unit.presentation.start += position_shift;
            unit.presentation.end += position_shift;
        }
        for piece in &mut output.output.pieces[receiver_index..] {
            piece.presentation.start += position_shift;
            piece.presentation.end += position_shift;
        }
        for domain in &mut output.domains {
            if domain.units.start >= receiver_index {
                domain.units.start += unit_shift;
                domain.units.end += unit_shift;
            } else {
                debug_assert!(domain.units.end <= receiver_index);
            }
        }
        for atom in &mut indication.output.atoms {
            atom.presentation.start += position_cut;
            atom.presentation.end += position_cut;
        }
        for unit in &mut indication.output.units {
            unit.atoms.start += atom_cut;
            unit.atoms.end += atom_cut;
            unit.presentation.start += position_cut;
            unit.presentation.end += position_cut;
        }
        for piece in &mut indication.output.pieces {
            piece.presentation.start += position_cut;
            piece.presentation.end += position_cut;
        }
        for domain in &mut indication.domains {
            domain.units.start += receiver_index;
            domain.units.end += receiver_index;
        }
        let domain_cut = output
            .domains
            .partition_point(|domain| domain.units.start < receiver_index);
        output
            .output
            .atoms
            .splice(atom_cut..atom_cut, indication.output.atoms);
        output
            .output
            .indices
            .splice(atom_cut..atom_cut, indication.output.indices);
        output
            .output
            .units
            .splice(receiver_index..receiver_index, indication.output.units);
        output
            .output
            .pieces
            .splice(receiver_index..receiver_index, indication.output.pieces);
        output.output.position = final_position;
        output.infos.splice(atom_cut..atom_cut, indication.infos);
        output.levels.splice(atom_cut..atom_cut, indication.levels);
        output
            .originals
            .splice(atom_cut..atom_cut, indication.originals);
        output
            .domains
            .splice(domain_cut..domain_cut, indication.domains);
    }
    let barrier_atoms = projection
        .barrier_atoms
        .iter()
        .filter_map(|&boundary| {
            output
                .originals
                .iter()
                .position(|atom| atom.is_some_and(|atom| atom >= boundary))
        })
        .collect();
    let projection = SourceProjection::from_atoms_at_barriers(
        output.output.atoms,
        output.output.position,
        projection.barriers.clone(),
        barrier_atoms,
    )?;
    let graphemes = projection
        .graphemes
        .windows(2)
        .filter_map(|pair| {
            let a = projection
                .atoms
                .partition_point(|a| a.presentation.start < pair[0]);
            let b = projection
                .atoms
                .partition_point(|a| a.presentation.start < pair[1]);
            (a < b).then_some(a..b)
        })
        .collect();
    let selected_analysis = Analysis {
        line_candidates: Vec::new(),
        phrases: crate::engine::analysis::phrase::PhraseCatalog::default(),
        infos: output.infos,
        levels: output.levels,
        graphemes,
        line_units: crate::engine::analysis::line_units::LineBreakUnits::new(&projection)?,
        base_level: analysis.base_level,
        box_levels: analysis.box_levels.clone(),
        control_levels: analysis.control_levels.clone(),
        computed_levels: analysis.computed_levels.clone(),
    };
    let mut scopes = vec![DirectionalScope::Outer(0); projection.atoms.len()];
    for unit in &output.output.units {
        for i in unit.atoms.clone() {
            scopes[i] = DirectionalScope::Outer(unit.level);
        }
    }
    for (d, domain) in output.domains.iter().enumerate() {
        for unit in &output.output.units[domain.units.clone()] {
            for i in unit.atoms.clone() {
                scopes[i] = if selected_analysis.levels[i] == domain.base {
                    DirectionalScope::Outer(domain.outer)
                } else {
                    DirectionalScope::Local(d, selected_analysis.levels[i])
                };
            }
        }
    }
    if !output.domains.is_empty() {
        prove_graphemes(
            &output.output.units,
            &output.domains,
            &selected_analysis,
            prepared.analyzed.processed.source().boxes(),
        )?;
    }
    for domain in &output.domains {
        if domain.outer
            != domain
                .attachment
                .level(prepared, &output.output.units, start.participant)?
            || domain.base != domain.outer & 1
        {
            return Err(composition_error());
        }
    }
    let provenance = crate::engine::upright::UprightProvenance::Selected {
        original: &prepared.analyzed.projection,
        originals: &output.originals,
        units: &output.output.units,
    };
    let upright_units = crate::engine::upright::units(
        &prepared.analyzed,
        &projection,
        &selected_analysis,
        &provenance,
        prepared.options.orientation(),
    )?;
    let profile = match prepared.options.orientation() {
        crate::LineOrientation::Horizontal => crate::engine::upright::ShapingProfile::Horizontal,
        crate::LineOrientation::SidewaysClockwise => {
            crate::engine::upright::ShapingProfile::SidewaysClockwise
        }
        crate::LineOrientation::SidewaysCounterclockwise => {
            crate::engine::upright::ShapingProfile::SidewaysCounterclockwise
        }
        crate::LineOrientation::Vertical
            if prepared.analyzed.typography == crate::AnalysisTypography::Mixed =>
        {
            crate::engine::upright::ShapingProfile::Mixed {
                owners: &prepared.analyzed.styles,
                units: &upright_units,
                provenance,
            }
        }
        crate::LineOrientation::Vertical => crate::engine::upright::ShapingProfile::Upright {
            units: &upright_units,
            provenance,
        },
    };
    let assignments = if output.domains.is_empty() {
        crate::engine::shape::retain_original_fonts(prepared, &projection, &output.originals)?
    } else {
        crate::engine::shape::assign_fonts(
            prepared,
            &projection,
            &selected_analysis,
            profile,
            &scopes,
        )?
    };
    let relation_end = if end.participant == tables.participants.len()
        && request.context.end == crate::LineEnd::Paragraph
    {
        prepared.analyzed.transform.events().len()
    } else {
        end.relation_cut
    };
    let relation = start.relation_cut..relation_end;
    let selected_relation = selected_relation(
        prepared,
        &output.output.units,
        &output.originals,
        start.participant,
        relation.clone(),
    );
    let map = &prepared.analyzed.transform;
    let source = selected_relation
        .events
        .iter()
        .filter_map(|event| match event {
            SelectedRelationEvent::Base { event } => Some(map.event_source(
                *event,
                &prepared.analyzed.paragraph,
                prepared.analyzed.processed.source(),
            )),
            SelectedRelationEvent::Indication { .. } => None,
        })
        .reduce(|a, b| Range::new(a.start.min(b.start), a.end.max(b.end)))
        .unwrap_or(Range::new(start.source_after, end.source_before));
    Ok(SelectedPresentation {
        vertical_units: upright_units,
        source,
        selected_relation,
        units: output.output.units,
        projection,
        analysis: selected_analysis,
        assignments,
        originals: output.originals,
        domains: output.domains,
        mapping: std::sync::Arc::new(SelectedMapping {
            pieces: output.output.pieces,
        }),
    })
}

fn selected_relation(
    prepared: &crate::engine::PreparedData,
    units: &[PresentationUnit],
    originals: &[Option<usize>],
    start_participant: usize,
    relation: IndexRange<usize>,
) -> SelectedRelation {
    let map = &prepared.analyzed.transform;
    let mut output = SelectedRelation {
        atom_events: vec![usize::MAX; originals.len()],
        ..SelectedRelation::default()
    };
    let mut base_positions = vec![None; relation.len()];
    let mut cursor = relation.start;
    for (unit, row) in units.iter().enumerate() {
        let first = output.events.len();
        if let Some(participant) = row.participant {
            let end = map.cut_for_participant(start_participant + participant + 1);
            for event in cursor..end {
                base_positions[event - relation.start] = Some(output.events.len());
                output.events.push(SelectedRelationEvent::Base { event });
            }
            cursor = end;
            for atom in row.atoms.clone() {
                if let Some(original) = originals[atom] {
                    output.atom_events[atom] = base_positions
                        [map.emission_for_atom(original).event - relation.start]
                        .expect("selected body atom has its actual base event");
                }
            }
        } else {
            for atom in row.atoms.clone() {
                debug_assert!(originals[atom].is_none());
                output.atom_events[atom] = output.events.len();
                output
                    .events
                    .push(SelectedRelationEvent::Indication { unit, atom });
            }
        }
        output.unit_events.push(first..output.events.len());
    }
    for event in cursor..relation.end {
        output.events.push(SelectedRelationEvent::Base { event });
    }
    debug_assert!(output.atom_events.iter().all(|event| *event != usize::MAX));
    output
}

fn reset_originals(
    units: &mut [PresentationUnit],
    infos: &[(CharInfo, usize)],
    boxes: &[crate::InlineBox],
    base: u8,
) {
    let originals: Vec<_> = units
        .iter()
        .enumerate()
        .filter(|(_, u)| !matches!(u.kind, PresentationUnitKind::Replacement { .. }))
        .map(|(i, _)| i)
        .collect();
    let roles = originals
        .iter()
        .map(|&i| {
            let unit = &units[i];
            if matches!(unit.kind, PresentationUnitKind::Atomic { .. }) {
                // OOF has no shaped flow item and cannot terminate an L1
                // whitespace sequence. Its original anchor/custody remains.
                return if is_out_of_flow(unit, boxes) {
                    LineReset::Removed
                } else {
                    LineReset::Other
                };
            }
            unit.atoms
                .clone()
                .map(|i| infos[i].0.bidi_class)
                .chain(unit.atoms.is_empty().then_some(unit.bidi_class).flatten())
                .map(LineReset::from_class)
                .fold(LineReset::Removed, LineReset::combine)
        })
        .collect::<Vec<_>>();
    let mut outer = originals
        .iter()
        .map(|&i| units[i].level)
        .collect::<Vec<_>>();
    reset_line_levels(&mut outer, &roles, base);
    for (&u, &level) in originals.iter().zip(&outer) {
        units[u].line_level = level;
    }
}

fn reorder(levels: impl IntoIterator<Item = u8>) -> crate::Result<Vec<usize>> {
    let participants = levels
        .into_iter()
        .map(|level| {
            crate::BidiLevel::try_new(level)
                .map(|level| crate::SuppliedInlineParticipant::Boundary { level })
                .map_err(|_| composition_error())
        })
        .collect::<crate::Result<Vec<_>>>()?;
    let mut output = vec![0; participants.len()];
    crate::SuppliedLine::new(&participants)
        .reorder_into(&mut output)
        .map_err(|_| composition_error())?;
    Ok(output)
}
fn composition_error() -> crate::Error {
    crate::hyphenation::error(crate::HyphenationError::UnsupportedIndicationComposition)
}
/// A genuine OOF occurrence retains source/placement custody but has no L2
/// flow item. Its inherited observation level cannot split participating runs.
fn is_out_of_flow(unit: &PresentationUnit, boxes: &[crate::InlineBox]) -> bool {
    matches!(unit.kind, PresentationUnitKind::Atomic { occurrence } if boxes[occurrence].kind() == crate::InlineBoxKind::OutOfFlow)
}
/// Hierarchy is two independent applications of the existing L2 owner. Local
/// levels never enter the original paragraph's level array or embedding stack.
pub(crate) fn unit_order(
    units: &[PresentationUnit],
    domains: &[IndicationDomain],
    boxes: &[crate::InlineBox],
) -> crate::Result<Vec<usize>> {
    let mut slots = Vec::new();
    let mut levels = Vec::new();
    let mut unit = 0;
    while unit < units.len() {
        if is_out_of_flow(&units[unit], boxes) {
            unit += 1;
            continue;
        }
        if let Some((d, domain)) = domains
            .iter()
            .enumerate()
            .find(|(_, d)| d.units.start == unit)
        {
            slots.push((true, d));
            levels.push(domain.outer);
            unit = domain.units.end;
        } else {
            slots.push((false, unit));
            levels.push(units[unit].line_level);
            unit += 1;
        }
    }
    let mut output = Vec::with_capacity(units.len());
    for i in reorder(levels)? {
        let (local, index) = slots[i];
        if local {
            let domain = &domains[index];
            for j in reorder(units[domain.units.clone()].iter().map(|u| u.line_level))? {
                output.push(domain.units.start + j);
            }
        } else {
            output.push(index);
        }
    }
    let originals: Vec<_> = units
        .iter()
        .enumerate()
        .filter(|(_, u)| {
            !matches!(u.kind, PresentationUnitKind::Replacement { .. }) && !is_out_of_flow(u, boxes)
        })
        .map(|(i, _)| i)
        .collect();
    let expected: Vec<_> = reorder(originals.iter().map(|&i| units[i].line_level))?
        .into_iter()
        .map(|i| originals[i])
        .collect();
    let actual: Vec<_> = output
        .iter()
        .copied()
        .filter(|&i| !matches!(units[i].kind, PresentationUnitKind::Replacement { .. }))
        .collect();
    if actual != expected {
        return Err(composition_error());
    }
    Ok(output)
}
fn prove_graphemes(
    units: &[PresentationUnit],
    domains: &[IndicationDomain],
    analysis: &Analysis,
    boxes: &[crate::InlineBox],
) -> crate::Result<()> {
    let order = unit_order(units, domains, boxes)?;
    let mut rank = vec![0; units.len()];
    for (r, &u) in order.iter().enumerate() {
        rank[u] = r;
    }
    for egc in &analysis.graphemes {
        let mut rows = units
            .iter()
            .enumerate()
            .filter(|(_, u)| {
                !is_out_of_flow(u, boxes) && u.atoms.start < egc.end && u.atoms.end > egc.start
            })
            .map(|(u, _)| rank[u])
            .collect::<Vec<_>>();
        rows.sort_unstable();
        rows.dedup();
        if rows.windows(2).any(|r| r[1] != r[0] + 1) {
            return Err(composition_error());
        }
    }
    Ok(())
}

/// Actual visual item order plus provider joint-unit and run congruence proof.
/// The selected basis retains this order before publishing complete handles.
pub(crate) fn order_presentation(
    presentation: &SelectedPresentation,
    boxes: &[crate::InlineBox],
    shaped: &crate::engine::layout::data::ShapedData,
) -> crate::Result<Vec<usize>> {
    use crate::engine::layout::data::ShapedItem;
    let mut by_unit = vec![Vec::new(); presentation.units.len()];
    for (item, value) in shaped.items.iter().enumerate() {
        let unit = match *value {
            ShapedItem::Anchor { unit, .. } => Some(unit),
            ShapedItem::InlineBox { occurrence, .. } => presentation
                .units
                .iter()
                .position(|u| u.kind == PresentationUnitKind::Atomic { occurrence }),
            ShapedItem::TextCluster(i) => presentation
                .units
                .iter()
                .position(|u| u.atoms.contains(&shaped.clusters[i].atoms.start)),
        }
        .ok_or_else(composition_error)?;
        by_unit[unit].push(item);
    }
    let mut output = Vec::with_capacity(shaped.items.len());
    for unit in unit_order(&presentation.units, &presentation.domains, boxes)? {
        let members = &by_unit[unit];
        for i in reorder(members.iter().map(|_| presentation.units[unit].line_level))? {
            output.push(members[i]);
        }
    }
    if output.len() != shaped.items.len() {
        return Err(composition_error());
    }
    // With no local domain this is the original operation's canonical L1/L2,
    // not a new composed-domain admission or font-query capability.
    if presentation.domains.is_empty() {
        return Ok(output);
    }
    let body: Vec<_> = output
        .iter()
        .copied()
        .filter(|&i| !matches!(shaped.items[i], ShapedItem::Anchor { .. }))
        .collect();
    for (run_index, run) in shaped.runs.iter().enumerate() {
        let mut expected:Vec<_>=shaped.items.iter().enumerate().filter(|(_,i)|matches!(**i,ShapedItem::TextCluster(c) if shaped.clusters[c].content.font_run()==Some(run_index))).map(|(i,_)|i).collect();
        if run.level & 1 != 0 {
            expected.reverse();
        }
        let positions: Vec<_> = body
            .iter()
            .enumerate()
            .filter(|(_, i)| expected.contains(i))
            .map(|(p, _)| p)
            .collect();
        let actual: Vec<_> = positions.iter().map(|&p| body[p]).collect();
        if actual != expected || positions.windows(2).any(|p| p[1] != p[0] + 1) {
            return Err(composition_error());
        }
    }
    for cluster in &shaped.clusters {
        let positions:Vec<_>=body.iter().enumerate().filter(|(_,i)|matches!(shaped.items[**i],ShapedItem::TextCluster(c) if shaped.clusters[c].atoms.start<cluster.provider_group.end&&shaped.clusters[c].atoms.end>cluster.provider_group.start)).map(|(p,_)|p).collect();
        if positions.windows(2).any(|p| p[1] != p[0] + 1) {
            return Err(composition_error());
        }
    }
    Ok(output)
}
