//! Issued counterfactual measurement flow, distinct from selected line authority.
use super::*;
use super::{input::RequestInput, prepared::PreparedBreakKind, selection::LineMetricPolicy};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FlowSegmentRole {
    EntryPrefix,
    PriorPartition,
    Target,
    Remaining,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Segment {
    role: FlowSegmentRole,
    start: usize,
    end: usize,
    incoming: Option<HyphenationContinuation>,
    alternative: Option<PreparedAlternative>,
    mandatory: Option<usize>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PreparedMeasurementFlow {
    prepared: PreparedText,
    query_start: usize,
    query_incoming: Option<HyphenationContinuation>,
    target: Arc<PreparedTermination>,
    entry: usize,
    completion: usize,
    segments: Arc<[Segment]>,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PreparedFlowSegment {
    flow: PreparedMeasurementFlow,
    index: usize,
}

/// Geometry only. Issued segment custody supplies all spelling and occurrences.
#[derive(Clone, Debug, PartialEq)]
pub struct FlowObservationRequest {
    pub context: LineContext,
    pub atomics: Vec<AtomicAllocation>,
    pub controls: Vec<ControlAllocation>,
    pub line_strut: Option<LineStrut>,
    pub line_box_struts: Vec<LineBoxStrut>,
    pub boundary_context: Vec<InlineBoundaryContextAllocation>,
}
#[derive(Clone, Debug)]
pub struct FlowObservation {
    pub(crate) segment: PreparedFlowSegment,
    pub(crate) request: FlowObservationRequest,
    pub(crate) profile: LineMetricPolicy,
    pub(crate) retained: bool,
    pub(crate) identity: Vec<u64>,
}
impl PartialEq for FlowObservation {
    fn eq(&self, other: &Self) -> bool {
        self.segment == other.segment
            && self.profile == other.profile
            && self.retained == other.retained
            && self.identity == other.identity
    }
}
impl Eq for FlowObservation {}
impl Hash for FlowObservation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.segment.hash(state);
        self.profile.hash(state);
        self.retained.hash(state);
        self.identity.hash(state);
    }
}
impl FlowObservation {
    pub fn segment(&self) -> PreparedFlowSegment {
        self.segment.clone()
    }
    pub fn request(&self) -> &FlowObservationRequest {
        &self.request
    }
    pub fn has_retained_flow_participation(&self) -> bool {
        self.retained
    }
}
fn invalid() -> crate::Error {
    protocol_error(ProtocolError::InvalidMeasurementFlow)
}

impl PreparedMeasurementFlow {
    pub fn entry(&self) -> PreparedBoundary {
        self.boundary(self.entry)
    }
    pub fn completion(&self) -> PreparedOpportunity {
        PreparedOpportunity {
            prepared: Arc::clone(&self.prepared.inner),
            index: self.completion,
        }
    }
    pub fn segments(&self) -> impl ExactSizeIterator<Item = PreparedFlowSegment> + '_ {
        (0..self.segments.len()).map(|index| PreparedFlowSegment {
            flow: self.clone(),
            index,
        })
    }
    pub fn validate_observations(&self, observations: &[FlowObservation]) -> Result<()> {
        if observations.len() != self.segments.len() {
            return Err(invalid());
        }
        for (expected, observed) in self.segments().zip(observations) {
            selection::require_prepared(
                &self.prepared,
                &observed.segment.flow.prepared.inner,
                ProtocolHandleKind::Boundary,
                observed.segment.row().start,
            )?;
            if observed.segment != expected
                || observations
                    .first()
                    .is_some_and(|first| first.profile != observed.profile)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    fn boundary(&self, index: usize) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(&self.prepared.inner),
            index,
        }
    }
}
impl PreparedFlowSegment {
    fn row(&self) -> &Segment {
        &self.flow.segments[self.index]
    }
    pub fn role(&self) -> FlowSegmentRole {
        self.row().role
    }
    pub fn start(&self) -> PreparedBoundary {
        self.flow.boundary(self.row().start)
    }
    pub fn end(&self) -> PreparedBoundary {
        self.flow.boundary(self.row().end)
    }
    pub fn incoming(&self) -> Option<HyphenationContinuation> {
        self.row().incoming.clone()
    }
    pub fn alternative(&self) -> Option<PreparedAlternative> {
        self.row().alternative.clone()
    }
    pub fn outgoing(&self) -> Option<HyphenationContinuation> {
        HyphenationContinuation::from_alternative(self.alternative()?)
    }
    pub fn mandatory_end(&self) -> Option<PreparedOpportunity> {
        Some(PreparedOpportunity {
            prepared: Arc::clone(&self.flow.prepared.inner),
            index: self.row().mandatory?,
        })
    }
    pub fn participants(&self) -> impl ExactSizeIterator<Item = PreparedParticipant> + '_ {
        let tables = &self.flow.prepared.inner.tables;
        let from = tables.boundaries[self.row().start].participant;
        let to = tables.boundaries[self.row().end].participant;
        self.flow.prepared.participants().skip(from).take(to - from)
    }
    pub(crate) fn admit<'a>(
        &'a self,
        prepared: &PreparedText,
        request: &'a FlowObservationRequest,
        participants: &'a [PreparedParticipant],
        start: &'a PreparedBoundary,
        end: &'a PreparedBoundary,
    ) -> Result<RequestInput<'a>> {
        selection::require_prepared(
            prepared,
            &self.flow.prepared.inner,
            ProtocolHandleKind::Boundary,
            self.row().start,
        )?;
        let expected_start = if prepared.inner.tables.boundaries[self.row().start].participant == 0
        {
            LineStart::Paragraph
        } else {
            LineStart::Continuation
        };
        let expected_end = if self.role() == FlowSegmentRole::Target {
            LineEnd::Soft
        } else {
            self.mandatory_end()
                .map_or(LineEnd::Soft, |end| match end.view().kind() {
                    PreparedBreakKind::Forced => LineEnd::Forced,
                    PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                    _ => LineEnd::Soft,
                })
        };
        if request.context.start != expected_start || request.context.end != expected_end {
            return Err(invalid());
        }
        let input = RequestInput {
            start,
            end,
            participants,
            incoming: self.row().incoming.as_ref(),
            alternative: self.row().alternative.as_ref(),
            context: &request.context,
            atomics: &request.atomics,
            controls: &request.controls,
            boundary_context: &request.boundary_context,
            line_strut: request.line_strut,
            line_box_struts: &request.line_box_struts,
        };
        selection::admit_range(prepared, input)?;
        Ok(input)
    }
}

impl PreparedText {
    pub fn measurement_flow(
        &self,
        query_start: &PreparedBoundary,
        query_incoming: Option<&HyphenationContinuation>,
        target: &PreparedTermination,
    ) -> Result<PreparedMeasurementFlow> {
        selection::require_prepared(
            self,
            &query_start.prepared,
            ProtocolHandleKind::Boundary,
            query_start.index,
        )?;
        if query_start.index >= self.inner.tables.boundaries.len() {
            return Err(invalid());
        }
        continuation::validate(self, query_start, query_incoming)?;
        let tables = &self.inner.tables;
        let completion = tables
            .opportunities
            .iter()
            .rposition(|offer| offer.kind == PreparedBreakKind::ParagraphEnd)
            .ok_or_else(invalid)?;
        let mut segments = Vec::new();
        let (entry, start, end, incoming, alternative) = match target {
            PreparedTermination::Measurement(portion) => {
                selection::require_prepared(
                    self,
                    &portion.word.prepared,
                    ProtocolHandleKind::WordPortion,
                    portion.word.index,
                )?;
                let portions = if portion.restored {
                    hyphenation::restored_portions(self, portion.word.index)?
                } else {
                    portion.word.view().portions()?
                };
                if !portions.contains(portion) {
                    return Err(invalid());
                }
                let entry = portions.first().ok_or_else(invalid)?.start;
                let word_end = portions.last().ok_or_else(invalid)?.end;
                if query_start.index >= word_end {
                    return Err(invalid());
                }
                let mut cursor = portion.start;
                let mut incoming = portion.incoming();
                let mut prior = Vec::new();
                while cursor > entry {
                    let at = portions.partition_point(|p| p.end < cursor);
                    let preceding = portions[at..]
                        .iter()
                        .take_while(|p| p.end == cursor)
                        .find(|p| {
                            p.alternative()
                                .and_then(HyphenationContinuation::from_alternative)
                                == incoming
                        })
                        .ok_or_else(invalid)?;
                    prior.push(Segment {
                        role: FlowSegmentRole::PriorPartition,
                        start: preceding.start,
                        end: preceding.end,
                        incoming: preceding.incoming(),
                        alternative: preceding.alternative(),
                        mandatory: None,
                    });
                    cursor = preceding.start;
                    incoming = preceding.incoming();
                }
                if query_start.index < entry {
                    append_range(
                        self,
                        &mut segments,
                        FlowSegmentRole::EntryPrefix,
                        query_start.index,
                        entry,
                        query_incoming.cloned(),
                        false,
                    )?;
                }
                segments.extend(prior.into_iter().rev());
                (
                    entry,
                    portion.start,
                    portion.end,
                    portion.incoming(),
                    portion.alternative(),
                )
            }
            PreparedTermination::PhraseMeasurement(portion) => {
                selection::require_prepared(
                    self,
                    &portion.phrase.prepared.inner,
                    ProtocolHandleKind::Boundary,
                    portion.start,
                )?;
                if !portion
                    .query
                    .matches_flow_query(self, query_start.index, query_incoming)
                    || !portion.query.phrase_portions().contains(portion)
                {
                    return Err(invalid());
                }
                if query_start.index < portion.start {
                    append_range(
                        self,
                        &mut segments,
                        FlowSegmentRole::EntryPrefix,
                        query_start.index,
                        portion.start,
                        query_incoming.cloned(),
                        false,
                    )?;
                }
                (
                    portion.start,
                    portion.start,
                    portion.end,
                    portion.incoming(),
                    None,
                )
            }
            _ => return Err(invalid()),
        };
        continuation::validate(
            self,
            &PreparedBoundary {
                prepared: Arc::clone(&self.inner),
                index: start,
            },
            incoming.as_ref(),
        )?;
        let mandatory = mandatory_at(self, end);
        let outgoing = alternative
            .clone()
            .and_then(HyphenationContinuation::from_alternative);
        segments.push(Segment {
            role: FlowSegmentRole::Target,
            start,
            end,
            incoming,
            alternative,
            mandatory,
        });
        let completion_end = tables.opportunities[completion].boundary;
        let forced =
            mandatory.is_some_and(|i| tables.opportunities[i].kind == PreparedBreakKind::Forced);
        if end < completion_end || outgoing.is_some() || forced {
            append_range(
                self,
                &mut segments,
                FlowSegmentRole::Remaining,
                end,
                completion_end,
                outgoing,
                true,
            )?;
        }
        Ok(PreparedMeasurementFlow {
            prepared: self.clone(),
            query_start: query_start.index,
            query_incoming: query_incoming.cloned(),
            target: Arc::new(target.clone()),
            entry,
            completion,
            segments: segments.into(),
        })
    }

    pub fn flow_boundary_context_demand(
        &self,
        segment: &PreparedFlowSegment,
        request: &FlowObservationRequest,
    ) -> Result<Option<PreparedControl>> {
        let participants: Vec<_> = segment.participants().collect();
        let start = segment.start();
        let end = segment.end();
        let input = segment.admit(self, request, &participants, &start, &end)?;
        let mut presentation = selection::admit_geometry(self, input)?;
        boundary::resolve(self, input, &mut presentation).map(|(_, missing)| missing)
    }
    pub fn observe_flow(
        &self,
        segment: &PreparedFlowSegment,
        request: &FlowObservationRequest,
    ) -> Result<FlowObservation> {
        crate::engine::layout::flow::observe(
            self,
            segment,
            request,
            LineMetricPolicy::Native,
            &mut crate::engine::shape::ShapeContext::default(),
        )
    }
}

fn mandatory_at(prepared: &PreparedText, boundary: usize) -> Option<usize> {
    prepared
        .inner
        .tables
        .opportunities
        .iter()
        .position(|offer| {
            offer.boundary == boundary
                && matches!(
                    offer.kind,
                    PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
                )
        })
}
fn append_range(
    prepared: &PreparedText,
    output: &mut Vec<Segment>,
    role: FlowSegmentRole,
    mut start: usize,
    end: usize,
    mut incoming: Option<HyphenationContinuation>,
    terminal: bool,
) -> Result<()> {
    if start > end {
        return Err(invalid());
    }
    let tables = &prepared.inner.tables;
    let from = start;
    for (index, offer) in tables
        .opportunities
        .iter()
        .enumerate()
        .filter(|(_, offer)| {
            offer.boundary > from
                && offer.boundary <= end
                && offer.kind == PreparedBreakKind::Forced
        })
    {
        output.push(Segment {
            role,
            start,
            end: offer.boundary,
            incoming: incoming.take(),
            alternative: None,
            mandatory: Some(index),
        });
        start = offer.boundary;
    }
    if start < end || terminal {
        let mandatory = terminal.then(|| mandatory_at(prepared, end)).flatten();
        // After a final Forced at the same mixed cut, retain genuine ParagraphEnd.
        let mandatory = if terminal {
            tables
                .opportunities
                .iter()
                .position(|offer| {
                    offer.boundary == end && offer.kind == PreparedBreakKind::ParagraphEnd
                })
                .or(mandatory)
        } else {
            mandatory
        };
        output.push(Segment {
            role,
            start,
            end,
            incoming,
            alternative: None,
            mandatory,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepare(text: &str) -> PreparedText {
        let mut system = crate::System::new(crate::SystemOptions {
            font_catalogue: crate::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let analyzed = crate::AnalyzedSource::new(
            crate::Source::new(text),
            crate::Style {
                font: crate::Font::new().family("Ahem"),
                size: 10.0,
                hanging_punctuation: crate::HangingPunctuation::new(true, None, true),
                ..crate::Style::default()
            },
        )
        .unwrap();
        system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap()
    }
    fn request(segment: &PreparedFlowSegment) -> FlowObservationRequest {
        let end = if segment.role() == FlowSegmentRole::Target {
            LineEnd::Soft
        } else {
            segment
                .mandatory_end()
                .map_or(LineEnd::Soft, |offer| match offer.view().kind() {
                    PreparedBreakKind::Forced => LineEnd::Forced,
                    PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                    _ => unreachable!(),
                })
        };
        FlowObservationRequest {
            context: LineContext {
                formatted_line: FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation: LineOrientation::Horizontal,
                start: if segment.start().index == 0 {
                    LineStart::Paragraph
                } else {
                    LineStart::Continuation
                },
                end,
                available_inline: Some(100.0),
                indent: 0.0,
                alignment: crate::Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
            atomics: vec![],
            controls: vec![],
            line_box_struts: Vec::new(),
            line_strut: None,
            boundary_context: vec![],
        }
    }
    fn flow(prepared: &PreparedText, portion: PreparedWordPortion) -> PreparedMeasurementFlow {
        let start = prepared.boundary_views().next().unwrap().handle();
        prepared
            .measurement_flow(&start, None, &PreparedTermination::Measurement(portion))
            .unwrap()
    }

    #[test]
    fn genuine_manual_partitions_issue_predecessors_without_claiming_accepted_lines() {
        let prepared = prepare("ab\u{ad}cd\u{ad}ef\"");
        let portions = prepared.word_views().next().unwrap().portions().unwrap();
        assert_eq!(
            portions.len(),
            3,
            "two source SHY choices partition six letters"
        );
        let flow = flow(&prepared, portions[2].clone());
        let segments: Vec<_> = flow.segments().collect();
        assert_eq!(
            segments
                .iter()
                .map(PreparedFlowSegment::role)
                .collect::<Vec<_>>(),
            [
                FlowSegmentRole::PriorPartition,
                FlowSegmentRole::PriorPartition,
                FlowSegmentRole::Target,
                FlowSegmentRole::Remaining
            ]
        );
        assert_eq!(segments[2].start(), portions[2].start());
        assert_eq!(segments[2].end(), portions[2].end());
        assert_eq!(segments[2].incoming(), portions[2].incoming());
        assert!(
            segments[..2]
                .iter()
                .all(|segment| segment.alternative().is_some())
        );
        let observations: Vec<_> = segments
            .iter()
            .map(|segment| prepared.observe_flow(segment, &request(segment)).unwrap())
            .collect();
        assert!(
            observations
                .iter()
                .all(FlowObservation::has_retained_flow_participation)
        );
        flow.validate_observations(&observations).unwrap();
        assert!(
            flow.validate_observations(&observations[..observations.len() - 1])
                .is_err()
        );
        let mut duplicate = observations.clone();
        duplicate[1] = duplicate[0].clone();
        assert!(flow.validate_observations(&duplicate).is_err());
    }

    #[test]
    fn forced_tail_preserves_the_separate_terminal_empty_recipe_and_observation() {
        let prepared = prepare("abcd\n");
        let portion = prepared
            .word_views()
            .next()
            .unwrap()
            .portions()
            .unwrap()
            .remove(0);
        let flow = flow(&prepared, portion);
        let segments: Vec<_> = flow.segments().collect();
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].role(), FlowSegmentRole::Target);
        assert_eq!(
            segments[1].mandatory_end().unwrap().view().kind(),
            PreparedBreakKind::Forced
        );
        assert_eq!(
            segments[2].mandatory_end().unwrap().view().kind(),
            PreparedBreakKind::ParagraphEnd
        );
        assert_eq!(segments[2].start(), segments[2].end());
        assert!(segments[2].participants().next().is_none());
        let observations: Vec<_> = segments
            .iter()
            .map(|segment| prepared.observe_flow(segment, &request(segment)).unwrap())
            .collect();
        assert!(!observations[2].has_retained_flow_participation());
        flow.validate_observations(&observations).unwrap();
    }

    #[test]
    fn observation_rejects_foreign_segments_nonfinite_geometry_and_wrong_end_context() {
        let prepared = prepare("abcd");
        let other = prepare("abcd");
        let portion = prepared
            .word_views()
            .next()
            .unwrap()
            .portions()
            .unwrap()
            .remove(0);
        let segment = flow(&prepared, portion).segments().next().unwrap();
        assert!(other.observe_flow(&segment, &request(&segment)).is_err());
        let mut invalid = request(&segment);
        invalid.context.indent = f64::NAN;
        assert!(prepared.observe_flow(&segment, &invalid).is_err());
        let mut wrong_end = request(&segment);
        wrong_end.context.end = LineEnd::Paragraph;
        assert!(prepared.observe_flow(&segment, &wrong_end).is_err());
    }

    #[test]
    fn genuine_phrase_query_binds_the_full_target_and_repeatable_recipe_identity() {
        // The published Japanese model's upstream reference qualifies AAAAA as
        // one phrase; Latin Ahem supplies real glyphs without script fallback.
        let mut system = crate::System::new(crate::SystemOptions {
            font_catalogue: crate::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let analyzed = crate::AnalyzedSource::new(
            crate::Source::new("AAAAA"),
            crate::Style {
                font: crate::Font::new().family("Ahem"),
                size: 10.0,
                locale: Some("ja".into()),
                word_break: crate::WordBreak::AutoPhrase,
                ..crate::Style::default()
            },
        )
        .unwrap();
        let prepared = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        let start = prepared.boundary_views().next().unwrap().handle();
        let ceiling = prepared.opportunity_views().next().unwrap().handle();
        // Public preparation is Native; build its complete authentic request,
        // rather than using the separate raw chooser's request factory.
        let trial = LineSelectionRequest {
            start: start.clone(),
            end: ceiling.view().boundary(),
            participants: prepared.participants().collect(),
            termination: PreparedTermination::Ordinary(ceiling.clone()),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_box_struts: Vec::new(),
            line_strut: None,
            context: LineContext {
                formatted_line: FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation: LineOrientation::Horizontal,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: Some(1.0),
                indent: 0.0,
                alignment: crate::Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        };
        let frame = prepared
            .select(trial)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert!(frame.measurements().fit().advance() > 1.0);
        let query = prepared
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &[],
                Some(&frame),
            )
            .unwrap();
        let portions = query.phrase_portions();
        assert_eq!(portions.len(), 1);
        let target = PreparedTermination::PhraseMeasurement(portions[0].clone());
        let recipe = prepared.measurement_flow(&start, None, &target).unwrap();
        let again = prepared.measurement_flow(&start, None, &target).unwrap();
        assert_eq!(recipe, again);
        let segments: Vec<_> = recipe.segments().collect();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].role(), FlowSegmentRole::Target);
        assert_eq!(segments[0].start(), portions[0].start());
        assert_eq!(segments[0].end(), portions[0].end());
        assert_eq!(segments[0].incoming(), portions[0].incoming());
        assert_eq!(recipe.completion(), ceiling);
        let observation = prepared
            .observe_flow(&segments[0], &request(&segments[0]))
            .unwrap();
        assert!(observation.has_retained_flow_participation());
        recipe.validate_observations(&[observation]).unwrap();
        assert!(
            prepared
                .measurement_flow(&portions[0].end(), None, &target)
                .is_err()
        );
    }
}
