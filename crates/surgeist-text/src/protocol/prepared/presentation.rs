//! Selected original UBA and indication-local domains, with separate level spaces.
use super::super::{PreparedSourceSegment, ProtocolError, protocol_error};
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
    /// Consumes this issued attachment's actual occurrence/scalar proof.
    /// The owner is the canonical emitted bidi row (including a transformed
    /// control's real semantic atom); the original level keeps its old owner.
    fn original(
        self,
        analyzed: &AnalyzedData,
        tables: &PreparedTables,
    ) -> crate::Result<(crate::engine::analysis::BidiOwner, u8)> {
        use crate::engine::analysis::BidiOwner as B;
        let (participant, atom) = match self {
            Self::Before { participant, atom } | Self::After { participant, atom } => {
                (participant, atom)
            }
        };
        match tables.participants.get(participant) {
            Some(ParticipantData::Text(i))
                if atom.is_some_and(|a| tables.units[*i].atoms.contains(&a)) =>
            {
                let atom = atom.expect("authenticated original scalar");
                Ok((B::Atom(atom), analyzed.analysis.levels[atom]))
            }
            Some(ParticipantData::Control(i)) if atom.is_none() => {
                let control = &tables.controls[*i];
                let owner = control
                    .semantic_atom
                    .map_or(B::Control(control.occurrence), B::Atom);
                Ok((owner, analyzed.analysis.control_levels[control.occurrence]))
            }
            Some(ParticipantData::Cell(i))
                if atom.is_none() && matches!(self, Self::After { .. }) =>
            {
                let binding = tables.cells[*i].binding;
                Ok((
                    B::CombinedCell(binding),
                    analyzed.analysis.cell_levels[binding],
                ))
            }
            Some(ParticipantData::Atomic(i))
                if atom.is_none()
                    && matches!(self, Self::After { .. })
                    && analyzed.processed.source().boxes()[tables.atomics[*i].occurrence]
                        .kind()
                        == crate::InlineBoxKind::InFlow =>
            {
                let occurrence = tables.atomics[*i].occurrence;
                Ok((
                    B::Atomic(occurrence),
                    analyzed.analysis.box_levels[occurrence],
                ))
            }
            _ => Err(composition_error()),
        }
    }
    pub(super) fn original_level(
        self,
        analyzed: &AnalyzedData,
        tables: &PreparedTables,
    ) -> crate::Result<u8> {
        self.original(analyzed, tables).map(|(_, level)| level)
    }
    pub(crate) fn original_owner(
        self,
        analyzed: &AnalyzedData,
        tables: &PreparedTables,
    ) -> crate::Result<crate::engine::analysis::BidiOwner> {
        self.original(analyzed, tables).map(|(owner, _)| owner)
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
                ParticipantData::Cell(_) | ParticipantData::ComputedLineBoundary(_) => {
                    return Err(composition_error());
                }
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
                crate::engine::analysis::used_bidi_class(
                    typography,
                    analysis.infos[i].0.bidi_class,
                ),
                Some((projection.atoms[i].glyph, analysis.infos[i].0.bracket)),
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
        self.emit_atoms(
            projection,
            analysis,
            atoms.iter().copied(),
            capture,
            original,
        )
    }

    fn emit_atoms(
        &mut self,
        projection: &SourceProjection,
        analysis: &Analysis,
        atoms: impl ExactSizeIterator<Item = usize> + Clone,
        capture: UnitCapture,
        original: bool,
    ) -> crate::Result<()> {
        self.infos.extend(atoms.clone().map(|i| analysis.infos[i]));
        self.levels
            .extend(atoms.clone().map(|i| analysis.levels[i]));
        self.originals
            .extend(atoms.clone().map(|i| original.then_some(i)));
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
            ParticipantData::Cell(i) => {
                let row = &tables.cells[i];
                let cell = &prepared.shaped.cells[row.cell];
                let output_cells = prepared
                    .analyzed
                    .combination
                    .as_deref()
                    .ok_or_else(composition_error)?;
                let binding = &output_cells.bindings[row.binding];
                output.emit_atoms(
                    projection,
                    analysis,
                    binding.atoms.clone(),
                    UnitCapture {
                        source: row.source,
                        processed: row.processed,
                        participant: Some(local),
                        kind: PresentationUnitKind::Cell { cell: row.cell },
                        level: cell.level,
                        bidi_class: Some(BidiClass::LeftToRight),
                    },
                    true,
                )?;
            }
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
                || matches!(unit.kind, PresentationUnitKind::Cell { .. })
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
        line_units: crate::engine::analysis::line_units::LineBreakUnits::new(&projection, None)?,
        base_level: analysis.base_level,
        box_levels: analysis.box_levels.clone(),
        control_levels: analysis.control_levels.clone(),
        computed_levels: analysis.computed_levels.clone(),
        cell_levels: Vec::new(),
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
    let relation_end = if prepared.analyzed.paragraph.scope_topology.is_none()
        && end.participant == tables.participants.len()
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
        request.segment,
        start.participant..end.participant,
        relation.clone(),
    )?;
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
    segment: &PreparedSourceSegment,
    participants: IndexRange<usize>,
    relation: IndexRange<usize>,
) -> crate::Result<SelectedRelation> {
    let map = &prepared.analyzed.transform;
    let invalid = || {
        protocol_error(ProtocolError::InvalidSourceSegment {
            segment: segment.index,
        })
    };
    let section = prepared
        .analyzed
        .sections
        .get(segment.index)
        .ok_or_else(invalid)?;
    if participants.start > participants.end
        || prepared
            .tables
            .source_prefix(participants.start)
            .is_none_or(|source| source < section.participants.start)
        || prepared
            .tables
            .source_prefix(participants.end)
            .is_none_or(|source| source > section.participants.end)
        || relation.start > relation.end
        || relation.start < section.events.start
        || relation.end > section.events.end
    {
        return Err(invalid());
    }
    let captured = prepared.analyzed.paragraph.scope_topology.is_some();
    let mut output = SelectedRelation {
        atom_events: vec![usize::MAX; originals.len()],
        ..SelectedRelation::default()
    };
    let mut base_positions = vec![None; relation.len()];
    let mut cursor = relation.start;
    // Canonical participant cuts can lend following removals to an ordinary
    // unit. A real cell instead owns its retained source-event interval, even
    // when suppression leaves it with no raw logical participant or atom.
    // Resolve each selected cell once in order; ordinary rows only borrow the
    // next entry frontier, without rescanning the remaining selected units.
    let mut cell_frontiers = units
        .iter()
        .enumerate()
        .take(if prepared.shaped.cells.is_empty() {
            0
        } else {
            units.len()
        })
        .filter_map(|(unit, row)| match row.kind {
            PresentationUnitKind::Cell { cell } => Some((unit, cell)),
            _ => None,
        })
        .map(|(unit, cell)| {
            let binding = prepared
                .shaped
                .cells
                .get(cell)
                .and_then(|cell| {
                    prepared
                        .analyzed
                        .combination
                        .as_deref()
                        .and_then(|output| output.bindings.get(cell.binding))
                })
                .ok_or_else(invalid)?;
            Ok::<_, crate::Error>((unit, binding))
        });
    let mut next_cell = cell_frontiers.next().transpose()?;
    for (unit, row) in units.iter().enumerate() {
        let first = output.events.len();
        let binding = if matches!(row.kind, PresentationUnitKind::Cell { .. }) {
            let (owner, binding) = next_cell.ok_or_else(invalid)?;
            if owner != unit
                || row.participant.is_none()
                || binding.events.is_empty()
                || binding.events.start < relation.start
                || binding.events.end > relation.end
                || cursor > binding.events.start
            {
                return Err(invalid());
            }
            Some(binding)
        } else {
            None
        };
        if let Some(participant) = row.participant {
            let prefix = participants
                .start
                .checked_add(participant)
                .and_then(|value| value.checked_add(1))
                .ok_or_else(invalid)?;
            if prefix > participants.end {
                return Err(invalid());
            }
            // A canonical prefix may own following participant-free events
            // across a child hole. The admitted captured endpoint is the real
            // end of this interval, even when the next section has no units.
            let end = if captured && prefix == participants.end {
                relation.end
            } else if let Some(binding) = binding {
                binding.events.end
            } else {
                let end = map.cut_for_participant(
                    prepared.tables.source_prefix(prefix).ok_or_else(invalid)?,
                );
                next_cell.map_or(end, |(_, binding)| end.min(binding.events.start))
            };
            if end < cursor || end > relation.end {
                return Err(invalid());
            }
            for event in cursor..end {
                base_positions[event - relation.start] = Some(output.events.len());
                output.events.push(SelectedRelationEvent::Base { event });
            }
            cursor = end;
            for atom in row.atoms.clone() {
                if let Some(original) = originals[atom] {
                    let event = map.emission_for_atom(original).event;
                    output.atom_events[atom] = event
                        .checked_sub(relation.start)
                        .and_then(|offset| base_positions.get(offset))
                        .copied()
                        .flatten()
                        .ok_or_else(invalid)?;
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
        if binding.is_some() {
            next_cell = cell_frontiers.next().transpose()?;
        }
    }
    for event in cursor..relation.end {
        output.events.push(SelectedRelationEvent::Base { event });
    }
    debug_assert!(output.atom_events.iter().all(|event| *event != usize::MAX));
    Ok(output)
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
            if matches!(unit.kind, PresentationUnitKind::Cell { .. }) {
                // The actual outer cell is bidi L, independently of every
                // retained inner scalar and of provider-input absence.
                return LineReset::Other;
            }
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

fn reorder(
    levels: impl IntoIterator<Item = u8>,
    ledger: &mut crate::engine::scope_alignment::accounting::Ledger,
) -> crate::Result<Vec<usize>> {
    use crate::engine::scope_alignment::accounting as account;
    let mut participants = Vec::new();
    let mut min_odd: Option<u8> = None;
    let mut maximum = 0;
    for value in levels {
        account::work(ledger, 1)?;
        let level = crate::BidiLevel::try_new(value).map_err(|_| composition_error())?;
        maximum = maximum.max(value);
        if value & 1 != 0 {
            min_odd = Some(min_odd.map_or(value, |n| n.min(value)));
        }
        account::push(
            ledger,
            &mut participants,
            crate::SuppliedInlineParticipant::Boundary { level },
        )?;
    }
    let mut output = account::filled(ledger, participants.len(), 0)?;
    let passes = min_odd.map_or(0, |n| usize::from(maximum - n) + 1);
    let work = participants
        .len()
        .checked_mul(passes * 4 + 3)
        .ok_or_else(|| {
            crate::engine::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
                kind: crate::ScopeAlignmentLimitKind::StructuralWork,
            })
        })?;
    account::work(ledger, work)?;
    crate::SuppliedLine::new(&participants)
        .reorder_into(&mut output)
        .map_err(|_| composition_error())?;
    account::drop_vec(ledger, participants)?;
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
    ledger: &mut crate::engine::scope_alignment::accounting::Ledger,
) -> crate::Result<Vec<usize>> {
    use crate::engine::scope_alignment::accounting as account;
    let mut slots = Vec::new();
    let mut levels = Vec::new();
    let mut unit = 0;
    while unit < units.len() {
        account::work(ledger, 1)?;
        if is_out_of_flow(&units[unit], boxes) {
            unit += 1;
            continue;
        }
        let mut found = None;
        for (d, domain) in domains.iter().enumerate() {
            account::work(ledger, 1)?;
            if domain.units.start == unit {
                found = Some((d, domain));
                break;
            }
        }
        if let Some((d, domain)) = found {
            account::push(ledger, &mut slots, (true, d))?;
            account::push(ledger, &mut levels, domain.outer)?;
            unit = domain.units.end;
        } else {
            account::push(ledger, &mut slots, (false, unit))?;
            account::push(ledger, &mut levels, units[unit].line_level)?;
            unit += 1;
        }
    }
    let order = reorder(levels.iter().copied(), ledger)?;
    let mut output = Vec::new();
    for &i in &order {
        account::work(ledger, 1)?;
        let (local, index) = slots[i];
        if local {
            let domain = &domains[index];
            let inner = reorder(
                units[domain.units.clone()].iter().map(|u| u.line_level),
                ledger,
            )?;
            for &j in &inner {
                account::push(ledger, &mut output, domain.units.start + j)?;
            }
            account::drop_vec(ledger, inner)?;
        } else {
            account::push(ledger, &mut output, index)?;
        }
    }
    let mut originals = Vec::new();
    for (i, u) in units.iter().enumerate() {
        account::work(ledger, 1)?;
        if !matches!(u.kind, PresentationUnitKind::Replacement { .. }) && !is_out_of_flow(u, boxes)
        {
            account::push(ledger, &mut originals, i)?;
        }
    }
    let expected = reorder(originals.iter().map(|&i| units[i].line_level), ledger)?;
    let mut actual = Vec::new();
    for &i in &output {
        account::work(ledger, 1)?;
        if !matches!(units[i].kind, PresentationUnitKind::Replacement { .. }) {
            account::push(ledger, &mut actual, i)?;
        }
    }
    if actual.len() != expected.len() {
        return Err(composition_error());
    }
    for (&a, &e) in actual.iter().zip(&expected) {
        account::work(ledger, 1)?;
        if a != originals[e] {
            return Err(composition_error());
        }
    }
    account::drop_vec(ledger, actual)?;
    account::drop_vec(ledger, expected)?;
    account::drop_vec(ledger, originals)?;
    account::drop_vec(ledger, order)?;
    account::drop_vec(ledger, levels)?;
    account::drop_vec(ledger, slots)?;
    Ok(output)
}

fn prove_graphemes(
    units: &[PresentationUnit],
    domains: &[IndicationDomain],
    analysis: &Analysis,
    boxes: &[crate::InlineBox],
) -> crate::Result<()> {
    let order = unit_order(units, domains, boxes, &mut None)?;
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
    prepared: &crate::engine::PreparedData,
    presentation: &SelectedPresentation,
    shaped: &crate::engine::layout::data::ShapedData,
    ledger: &mut crate::engine::scope_alignment::accounting::Ledger,
) -> crate::Result<Vec<usize>> {
    use crate::engine::layout::data::ShapedItem;
    use crate::engine::scope_alignment::accounting as account;
    // These are actual retained selected Cells, including NoProviderInput.
    // An unrelated scoped operation must not fund or retire native ordering.
    let mut native = None;
    let ledger = if shaped.cells.is_empty() {
        &mut native
    } else {
        if ledger.is_none() {
            return Err(crate::engine::scope_alignment::refusal(
                crate::ScopeAlignmentError::LimitsRequired,
            ));
        }
        ledger
    };
    let boxes = prepared.analyzed.processed.source().boxes();
    let mut by_unit = account::filled(ledger, presentation.units.len(), Vec::new())?;
    for (item, value) in shaped.items.iter().enumerate() {
        account::work(ledger, 1)?;
        let unit = match *value {
            ShapedItem::Anchor { unit, .. } => Some(unit),
            _ => {
                let mut found = None;
                for (index, unit) in presentation.units.iter().enumerate() {
                    account::work(ledger, 1)?;
                    let matches = match *value {
                        ShapedItem::Cell(cell) => matches!(unit.kind, PresentationUnitKind::Cell { cell: original }
                            if prepared.shaped.cells[original].binding == shaped.cells[cell].binding),
                        ShapedItem::InlineBox { occurrence, .. } => unit.kind == PresentationUnitKind::Atomic { occurrence },
                        ShapedItem::TextCluster(i) => unit.atoms.contains(&shaped.clusters[i].atoms.start),
                        ShapedItem::Anchor { .. } => unreachable!("anchor handled by its actual unit"),
                    };
                    if matches { found = Some(index); break; }
                }
                found
            }
        }.ok_or_else(composition_error)?;
        account::push(ledger, &mut by_unit[unit], item)?;
    }
    let mut output = Vec::new();
    account::reserve_vec(ledger, &mut output, shaped.items.len())?;
    let order = unit_order(&presentation.units, &presentation.domains, boxes, ledger)?;
    for &unit in &order {
        account::work(ledger, 1)?;
        let members = &by_unit[unit];
        let inner = reorder(
            members.iter().map(|_| presentation.units[unit].line_level),
            ledger,
        )?;
        for &i in &inner {
            account::work(ledger, 1)?;
            account::push(ledger, &mut output, members[i])?;
        }
        account::drop_vec(ledger, inner)?;
    }
    account::drop_vec(ledger, order)?;
    for members in &mut by_unit {
        account::work(ledger, 1)?;
        account::drop_vec(ledger, std::mem::take(members))?;
    }
    account::drop_vec(ledger, by_unit)?;
    if output.len() != shaped.items.len() {
        return Err(composition_error());
    }
    // With no local domain this is the original operation's canonical L1/L2,
    // not a new composed-domain admission or font-query capability.
    if presentation.domains.is_empty() {
        return Ok(output);
    }
    let mut body = Vec::new();
    for &i in &output {
        account::work(ledger, 1)?;
        if !matches!(shaped.items[i], ShapedItem::Anchor { .. }) {
            account::push(ledger, &mut body, i)?;
        }
    }
    for (run_index, run) in shaped.runs.iter().enumerate() {
        account::work(ledger, 1)?;
        let mut expected = Vec::new();
        for (i, item) in shaped.items.iter().enumerate() {
            account::work(ledger, 1)?;
            if matches!(*item, ShapedItem::TextCluster(c) if shaped.clusters[c].content.font_run() == Some(run_index))
            {
                account::push(ledger, &mut expected, i)?;
            }
        }
        if run.level & 1 != 0 {
            account::work(ledger, expected.len() / 2 * 2)?;
            expected.reverse();
        }
        let mut positions = Vec::new();
        for (position, item) in body.iter().enumerate() {
            account::work(ledger, 1)?;
            let mut member = false;
            for candidate in &expected {
                account::work(ledger, 1)?;
                if candidate == item {
                    member = true;
                    break;
                }
            }
            if member {
                account::push(ledger, &mut positions, position)?;
            }
        }
        let mut actual = Vec::new();
        for &position in &positions {
            account::work(ledger, 1)?;
            account::push(ledger, &mut actual, body[position])?;
        }
        if actual.len() != expected.len() {
            return Err(composition_error());
        }
        for (actual, expected) in actual.iter().zip(&expected) {
            account::work(ledger, 1)?;
            if actual != expected {
                return Err(composition_error());
            }
        }
        for pair in positions.windows(2) {
            account::work(ledger, 1)?;
            if pair[1] != pair[0] + 1 {
                return Err(composition_error());
            }
        }
        account::drop_vec(ledger, actual)?;
        account::drop_vec(ledger, positions)?;
        account::drop_vec(ledger, expected)?;
    }
    for cluster in &shaped.clusters {
        account::work(ledger, 1)?;
        let mut positions = Vec::new();
        for (position, &item) in body.iter().enumerate() {
            account::work(ledger, 1)?;
            if matches!(shaped.items[item], ShapedItem::TextCluster(c)
                if shaped.clusters[c].atoms.start < cluster.provider_group.end
                && shaped.clusters[c].atoms.end > cluster.provider_group.start)
            {
                account::push(ledger, &mut positions, position)?;
            }
        }
        for pair in positions.windows(2) {
            account::work(ledger, 1)?;
            if pair[1] != pair[0] + 1 {
                return Err(composition_error());
            }
        }
        account::drop_vec(ledger, positions)?;
    }
    account::drop_vec(ledger, body)?;
    Ok(output)
}
