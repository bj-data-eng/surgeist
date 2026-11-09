//! Borrowed spelling/occurrence overlay shared by selection and semantic windows.
use super::*;

pub(crate) struct SemanticUnit<'a> {
    pub(crate) projection: &'a SourceProjection,
    pub(crate) analysis: &'a Analysis,
    pub(crate) atoms: IndexRange<usize>,
    pub(crate) capture: UnitCapture,
}

pub(crate) enum Emission<'a> {
    Original(SemanticUnit<'a>),
    Indication(usize),
}

/// Canonical ordered original/deleted/indication overlay. Consumers add their
/// own bidi/font work or purely semantic observations to these same emissions.
pub(crate) struct Overlay<'a> {
    prepared: &'a crate::engine::PreparedData,
    participants: IndexRange<usize>,
    local: usize,
    stage: u8,
    alternative: Option<usize>,
}
impl<'a> Overlay<'a> {
    pub(crate) fn new(
        prepared: &'a crate::engine::PreparedData,
        participants: IndexRange<usize>,
        alternative: Option<usize>,
    ) -> Self {
        Self {
            prepared,
            participants,
            local: 0,
            stage: 0,
            alternative,
        }
    }
}
impl<'a> Iterator for Overlay<'a> {
    type Item = crate::Result<Emission<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let definition = self
            .alternative
            .map(|i| self.prepared.tables.alternatives[i].definition);
        let before = self.alternative.filter(|&i| {
            self.prepared.tables.alternatives[i]
                .placement
                .includes_before()
        });
        while self.local < self.participants.len() {
            let participant =
                self.prepared.tables.participants[self.participants.start + self.local];
            let defining = defines(self.prepared, participant, definition);
            let deleted = defining && matches!(participant, ParticipantData::Text(_));
            match self.stage {
                0 => {
                    self.stage = 1;
                    if deleted {
                        return Some(
                            original(self.prepared, participant, self.local, definition)
                                .map(Emission::Original)
                                .ok_or_else(presentation::composition_error),
                        );
                    }
                }
                1 => {
                    self.stage = 2;
                    if defining && let Some(before) = before {
                        return Some(Ok(Emission::Indication(before)));
                    }
                }
                _ => {
                    let local = self.local;
                    self.local += 1;
                    self.stage = 0;
                    if !deleted {
                        return Some(
                            original(self.prepared, participant, local, definition)
                                .map(Emission::Original)
                                .ok_or_else(presentation::composition_error),
                        );
                    }
                }
            }
        }
        if self.stage != 3 {
            self.stage = 3;
            if matches!(definition, Some(AlternativeDefinition::Automatic { .. }))
                && let Some(before) = before
            {
                return Some(Ok(Emission::Indication(before)));
            }
        }
        None
    }
}

pub(crate) fn receives_incoming(
    prepared: &crate::engine::PreparedData,
    kind: PresentationUnitKind,
    nonempty: bool,
    bidi: bool,
) -> bool {
    nonempty && kind == PresentationUnitKind::Text
        || kind == PresentationUnitKind::Deleted && bidi
        || matches!(kind, PresentationUnitKind::Cell { .. })
        || matches!(kind, PresentationUnitKind::Atomic { occurrence } if prepared.analyzed.processed.source().boxes()[occurrence].kind() == crate::InlineBoxKind::InFlow)
        || matches!(kind, PresentationUnitKind::Control { .. }) && bidi
}

/// Select one original occurrence, including an actual SHY deletion. This does
/// not inspect the prepared glyph, font, Cell metrics or selected geometry.
pub(crate) fn original(
    prepared: &crate::engine::PreparedData,
    participant: ParticipantData,
    local: usize,
    definition: Option<AlternativeDefinition>,
) -> Option<SemanticUnit<'_>> {
    let tables = &prepared.tables;
    let analysis = &prepared.analyzed.analysis;
    let projection = &prepared.analyzed.projection;
    let mut result = SemanticUnit {
        projection,
        analysis,
        atoms: 0..0,
        capture: UnitCapture {
            source: Range::new(0, 0),
            processed: Range::new(0, 0),
            participant: Some(local),
            kind: PresentationUnitKind::Text,
            level: 0,
            bidi_class: None,
        },
    };
    match participant {
        ParticipantData::Cell(i) => {
            let row = &tables.cells[i];
            let binding = &prepared.analyzed.combination.as_ref()?.bindings[row.binding];
            result.atoms = binding.atoms.clone();
            result.capture.source = row.source;
            result.capture.processed = row.processed;
            result.capture.kind = PresentationUnitKind::Cell { cell: row.cell };
            result.capture.level = analysis.cell_levels[row.binding];
            result.capture.bidi_class = Some(icu_properties::props::BidiClass::LeftToRight);
        }
        ParticipantData::Text(i) => {
            let row = &tables.units[i];
            let defining = matches!(definition,
                Some(AlternativeDefinition::Source { source, .. })
                if !row.deleted && row.source.start <= source.start && row.source.end >= source.end);
            result.capture.source = row.source;
            result.capture.processed = row.processed;
            result.capture.level = row.level;
            result.capture.bidi_class = row
                .atoms
                .clone()
                .next()
                .map(|i| analysis.infos[i].0.bidi_class);
            result.capture.kind = if defining || row.deleted {
                PresentationUnitKind::Deleted
            } else if let Some(origin) = row.empty_origin {
                PresentationUnitKind::EmptyOrigin { origin }
            } else if let Some(&control) = prepared
                .analyzed
                .transform
                .controls_for_atoms(row.atoms.clone())
                .first()
            {
                PresentationUnitKind::GeneratedSeparator {
                    owner: GeneratedSeparatorOwner::Control(control),
                }
            } else if let Some(&index) = prepared
                .analyzed
                .transform
                .virtuals_for_atoms(row.atoms.clone())
                .first()
            {
                PresentationUnitKind::GeneratedSeparator {
                    owner: GeneratedSeparatorOwner::Virtual(index),
                }
            } else {
                PresentationUnitKind::Text
            };
            if !defining {
                result.atoms = row.atoms.clone();
            }
        }
        ParticipantData::Atomic(i) => {
            let row = tables.atomics[i];
            result.capture.source = Range::new(row.source, row.source);
            result.capture.processed = Range::new(row.processed, row.processed);
            result.capture.kind = PresentationUnitKind::Atomic {
                occurrence: row.occurrence,
            };
            result.capture.level = analysis.box_levels[row.occurrence];
        }
        ParticipantData::ComputedLineBoundary(i) => {
            let row = tables.computed_boundaries[i];
            result.capture.source = Range::new(row.source, row.source);
            result.capture.processed = Range::new(row.processed, row.processed);
            result.capture.kind = PresentationUnitKind::ComputedLineBoundary { boundary: i };
            result.capture.level = row.level;
            result.capture.bidi_class = Some(icu_properties::props::BidiClass::ParagraphSeparator);
        }
        ParticipantData::Control(i) => {
            let row = tables.controls[i];
            let kind = &prepared.analyzed.paragraph.controls[row.occurrence].kind;
            let cp = if *kind == ParagraphControlKind::ForcedBreak {
                Some(10)
            } else {
                kind.code_point()
            };
            result.capture.source = Range::new(row.source, row.source);
            result.capture.processed = Range::new(row.processed, row.processed);
            result.capture.kind = PresentationUnitKind::Control {
                occurrence: row.occurrence,
            };
            result.capture.level = analysis.control_levels[row.occurrence];
            result.capture.bidi_class = cp.map(|cp| {
                icu_properties::CodePointMapData::<icu_properties::props::BidiClass>::new()
                    .get32(cp)
            });
        }
    }
    Some(result)
}

pub(crate) fn defines(
    prepared: &crate::engine::PreparedData,
    participant: ParticipantData,
    definition: Option<AlternativeDefinition>,
) -> bool {
    match (definition, participant) {
        (Some(AlternativeDefinition::Source { source, .. }), ParticipantData::Text(i)) => {
            let row = &prepared.tables.units[i];
            !row.deleted && row.source.start <= source.start && row.source.end >= source.end
        }
        (Some(AlternativeDefinition::Control { control, .. }), ParticipantData::Control(i)) => {
            prepared.tables.controls[i].occurrence == control
        }
        _ => false,
    }
}
