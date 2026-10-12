//! Immutable one-row reverse observation of the completed association slice.
//! Parse admission has already produced this slice; readback never resumes it.

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AssociationRelation {
    Output,
    Creation,
    Original,
    Incoming,
    Trigger,
    LexicalEnvelope,
    LexicalPart,
    LexicalAnchor,
    Omission,
    Cause,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReverseQueryError {
    ForeignSource,
    Coordinates,
    Reversed,
    OutOfBounds,
    Utf8Boundary,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CursorState {
    Active,
    Complete,
    Cancelled,
}
pub(crate) enum ReverseStep<'a> {
    Match(&'a NativeAssociation),
    Skipped,
    Complete,
    Cancelled,
}
pub(crate) struct ReverseCursor<'a> {
    rows: &'a [NativeAssociation],
    start: usize,
    end: usize,
    index: usize,
    cancellation: Option<&'a crate::HtmlParseCancellation>,
    state: CursorState,
}
impl<'a> ReverseCursor<'a> {
    pub(crate) fn advance(&mut self) -> ReverseStep<'a> {
        match self.state {
            CursorState::Complete => return ReverseStep::Complete,
            CursorState::Cancelled => return ReverseStep::Cancelled,
            CursorState::Active => {}
        }
        if self
            .cancellation
            .is_some_and(|cancel| cancel.is_cancelled())
        {
            self.state = CursorState::Cancelled;
            return ReverseStep::Cancelled;
        }
        let Some(row) = self.rows.get(self.index) else {
            self.state = CursorState::Complete;
            return ReverseStep::Complete;
        };
        self.index += 1;
        let Some(range) = &row.source else {
            return ReverseStep::Skipped;
        };
        let (start, end) = (range.start(), range.end());
        let matched = if self.start == self.end {
            if start == end {
                start == self.start
            } else {
                start <= self.start && self.start < end
            }
        } else if start == end {
            self.start <= start && start < self.end
        } else {
            self.start < end && start < self.end
        };
        if matched {
            ReverseStep::Match(row)
        } else {
            ReverseStep::Skipped
        }
    }
}
pub(crate) struct AssociationView<'a> {
    source: &'a crate::HtmlSource,
    rows: &'a [NativeAssociation],
    causes: &'a [ConstructionCause],
    groups: &'a [OriginGroup],
    occurrences: &'a [LexicalOccurrence],
    parts: &'a [LexicalPart],
    omissions: &'a [lexical::LexicalOmission],
}
impl<'a> AssociationView<'a> {
    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }
    pub(crate) fn row(&self, index: usize) -> Option<&'a NativeAssociation> {
        self.rows.get(index)
    }
    pub(crate) fn cause(&self, index: usize) -> Option<&'a ConstructionCause> {
        self.causes.get(index)
    }
    pub(crate) fn lexical_occurrence(&self, index: usize) -> Option<&'a LexicalOccurrence> {
        self.occurrences.get(index)
    }
    pub(crate) fn lexical_part(&self, index: usize) -> Option<&'a LexicalPart> {
        self.parts.get(index)
    }
    pub(crate) fn omission(&self, index: usize) -> Option<&'a lexical::LexicalOmission> {
        self.omissions.get(index)
    }
    /// One fixed group fact; actual history origins require the owner's charged
    /// cursor. Reverse advance never follows group or cause links.
    pub(crate) fn group(&self, index: usize) -> Option<&'a OriginGroup> {
        self.groups.get(index)
    }
    pub(crate) fn reverse(
        &self,
        query: &'a crate::HtmlSourceRange,
        cancellation: Option<&'a crate::HtmlParseCancellation>,
    ) -> Result<ReverseCursor<'a>, ReverseQueryError> {
        if query.source() != self.source.identity() {
            return Err(ReverseQueryError::ForeignSource);
        }
        if query.coordinates() != self.source.coordinates() {
            return Err(ReverseQueryError::Coordinates);
        }
        self.reverse_span(query.start(), query.end(), cancellation)
    }
    pub(crate) fn reverse_span(
        &self,
        start: usize,
        end: usize,
        cancellation: Option<&'a crate::HtmlParseCancellation>,
    ) -> Result<ReverseCursor<'a>, ReverseQueryError> {
        if start > end {
            return Err(ReverseQueryError::Reversed);
        }
        if end > self.source.len() {
            return Err(ReverseQueryError::OutOfBounds);
        }
        if let Some(text) = self.source.as_utf8() {
            // Each interior endpoint requires one byte inspection. Boundary
            // zero/len require none; no scan, decode, handle clone or allocation.
            for offset in [start, end] {
                if offset > 0 && offset < text.len() && text.as_bytes()[offset] & 0xc0 == 0x80 {
                    return Err(ReverseQueryError::Utf8Boundary);
                }
            }
        }
        Ok(ReverseCursor {
            rows: self.rows,
            start,
            end,
            index: 0,
            cancellation,
            state: CursorState::Active,
        })
    }
}
// One entry per immutable group/cause index. A group-qualified key has the
// group's immutable cause; a groupless key has its cause index directly.
// Four stamps distinguish every actual lineage role without a linear scan.
#[derive(Clone, Copy)]
struct LineageSlot {
    closure: Option<(usize, usize)>,
    seen: [usize; 4],
}
#[derive(Clone, Copy)]
struct LineageLink {
    cause: usize,
    relation: AssociationRelation,
    group: Option<usize>,
}
impl LineageLink {
    fn role(self) -> Result<usize, SessionError> {
        match self.relation {
            AssociationRelation::Creation => Ok(0),
            AssociationRelation::Original => Ok(1),
            AssociationRelation::Incoming => Ok(2),
            AssociationRelation::Trigger => Ok(3),
            _ => Err(SessionError::WrongToken),
        }
    }
    fn slot<'a>(
        self,
        causes: &'a mut OwnedVec<LineageSlot>,
        groups: &'a mut OwnedVec<LineageSlot>,
        c: &mut ParseControl,
    ) -> Result<&'a mut LineageSlot, SessionError> {
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<Option<usize>>() as u64,
        )?;
        let (slots, index) = match self.group {
            Some(index) => (groups, index),
            None => (causes, self.cause),
        };
        slots.get_mut(index, c)?.ok_or(SessionError::WrongToken)
    }
}

impl ParseEvidence {
    pub(super) fn project_lineage(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        c.checkpoint()?;
        self.native.causes.require_disposal_owner(c)?;
        self.native.groups.require_disposal_owner(c)?;
        self.native
            .projection
            .as_ref()
            .ok_or(SessionError::WrongToken)?
            .require_disposal_owner(c)?;
        let mut pending = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let mut cause_slots = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let mut group_slots = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let mut closure = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        pending.require_owner(c)?;
        cause_slots.require_owner(c)?;
        group_slots.require_owner(c)?;
        closure.require_owner(c)?;
        let result = (|| {
            // Real initialized scratch slots, not an uncharged hash/set or
            // speculative Cartesian cause x group allocation. Every group has
            // exactly one immutable cause, so (index, role) is the exact key.
            for (slots, count) in [
                (&mut cause_slots, self.native.causes.len()),
                (&mut group_slots, self.native.groups.len()),
            ] {
                slots.reserve_geometric(c, count)?;
                for _ in 0..count {
                    slots.try_push(
                        c,
                        LineageSlot {
                            closure: None,
                            seen: [0; 4],
                        },
                    )?;
                }
            }
            for index in 0..self.native.native_rows {
                c.charge(WorkCategory::RecordVisit, 1)?;
                let row = &self
                    .native
                    .projection
                    .as_ref()
                    .ok_or(SessionError::WrongToken)?
                    .as_slice()[index];
                c.charge(
                    WorkCategory::CopyByte,
                    (mem::size_of::<NativeDestination>()
                        + 3 * mem::size_of::<usize>()
                        + mem::size_of::<Option<usize>>()) as u64,
                )?;
                let (destination, start, end) = (row.destination, row.start, row.end);
                let root = LineageLink {
                    cause: row.cause,
                    relation: AssociationRelation::Creation,
                    group: row.group,
                };
                self.validate_lineage_key(c, root)?;
                c.charge(
                    WorkCategory::InspectByte,
                    mem::size_of::<Option<(usize, usize)>>() as u64,
                )?;
                c.charge(
                    WorkCategory::CopyByte,
                    mem::size_of::<Option<(usize, usize)>>() as u64,
                )?;
                let cached = root.slot(&mut cause_slots, &mut group_slots, c)?.closure;
                let span = if let Some(span) = cached {
                    span
                } else {
                    let begin = closure.len();
                    let generation = match index.checked_add(1) {
                        Some(generation) => generation,
                        None => c.representation(HtmlParseResource::AuxiliaryEntries)?,
                    };
                    pending.reserve_geometric(c, 1)?;
                    pending.try_push(c, root)?;
                    while let Some(link) = pending.try_pop(c)? {
                        self.validate_lineage_key(c, link)?;
                        let role = link.role()?;
                        let slot = link.slot(&mut cause_slots, &mut group_slots, c)?;
                        c.charge(WorkCategory::InspectByte, mem::size_of::<usize>() as u64)?;
                        if slot.seen[role] == generation {
                            continue;
                        }
                        c.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
                        c.charge(WorkCategory::RecordWrite, 1)?;
                        slot.seen[role] = generation;
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        let cause = self
                            .native
                            .causes
                            .as_slice()
                            .get(link.cause)
                            .ok_or(SessionError::WrongToken)?;
                        c.charge(
                            WorkCategory::CopyByte,
                            (mem::size_of::<[Option<usize>; 4]>()
                                + mem::size_of::<[Option<(usize, usize)>; 2]>())
                                as u64,
                        )?;
                        let links = [cause.original_cause, cause.trigger];
                        let mut groups = [None, None];
                        for (slot, group) in groups.iter_mut().zip([cause.original, cause.incoming])
                        {
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            if let Some(group) = group {
                                *slot = Some((
                                    self.native
                                        .groups
                                        .as_slice()
                                        .get(group)
                                        .ok_or(SessionError::WrongToken)?
                                        .cause,
                                    group,
                                ));
                            }
                        }
                        if let Some(source) = &cause.source {
                            c.require_source_range(source)?;
                            closure.reserve_geometric(c, 1)?;
                            closure.try_push(c, link)?;
                        }
                        // Preserve the original depth-first source order, and
                        // distinct Original/Incoming group identity even when
                        // several groups share the same source-bearing cause.
                        for (cause, relation, group) in [
                            (links[0], AssociationRelation::Original, None),
                            (links[1], AssociationRelation::Trigger, None),
                            (
                                groups[1].map(|group| group.0),
                                AssociationRelation::Incoming,
                                groups[1].map(|group| group.1),
                            ),
                            (
                                groups[0].map(|group| group.0),
                                AssociationRelation::Original,
                                groups[0].map(|group| group.1),
                            ),
                        ] {
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            if let Some(cause) = cause {
                                if cause >= link.cause {
                                    return Err(SessionError::WrongToken);
                                }
                                pending.reserve_geometric(c, 1)?;
                                pending.try_push(
                                    c,
                                    LineageLink {
                                        cause,
                                        relation,
                                        group,
                                    },
                                )?;
                            }
                        }
                    }
                    let span = (begin, closure.len());
                    // No cache success is published until its full closure is
                    // admitted. A zero-length span is a known empty closure.
                    c.charge(
                        WorkCategory::CopyByte,
                        mem::size_of::<Option<(usize, usize)>>() as u64,
                    )?;
                    c.charge(WorkCategory::RecordWrite, 1)?;
                    root.slot(&mut cause_slots, &mut group_slots, c)?.closure = Some(span);
                    span
                };
                // Only required source-bearing relations are copied to this
                // actual current destination/interval. Historical prefixes are
                // never manufactured as output destinations by the cache.
                for link_index in span.0..span.1 {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    c.charge(WorkCategory::CopyByte, mem::size_of::<LineageLink>() as u64)?;
                    let link = *closure
                        .as_slice()
                        .get(link_index)
                        .ok_or(SessionError::WrongToken)?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let source = self
                        .native
                        .causes
                        .as_slice()
                        .get(link.cause)
                        .and_then(|cause| cause.source.as_ref())
                        .ok_or(SessionError::WrongToken)?;
                    c.require_source_range(source)?;
                    payload_write::<NativeAssociation>(c, 1)?;
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    let projected = NativeAssociation {
                        relation: link.relation,
                        group: link.group,
                        destination,
                        start,
                        end,
                        source: Some(source.clone()),
                        kind: None,
                        cause: link.cause,
                    };
                    let rows = self
                        .native
                        .projection
                        .as_mut()
                        .ok_or(SessionError::WrongToken)?;
                    rows.reserve_geometric(c, 1)?;
                    rows.try_push(c, projected)?;
                }
            }
            Ok(())
        })();
        pending.release(c)?;
        cause_slots.release(c)?;
        group_slots.release(c)?;
        closure.release(c)?;
        result
    }

    fn validate_lineage_key(
        &self,
        c: &mut ParseControl,
        key: LineageLink,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.native.causes.as_slice().get(key.cause).is_none() {
            return Err(SessionError::WrongToken);
        }
        if let Some(index) = key.group {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::InspectByte, mem::size_of::<usize>() as u64)?;
            if self
                .native
                .groups
                .as_slice()
                .get(index)
                .is_none_or(|group| group.cause != key.cause)
            {
                return Err(SessionError::WrongToken);
            }
        }
        Ok(())
    }
    /// Source causes and their fixed links remain observable. Do not expand
    /// historical prefix destinations: original group identities are retained
    /// separately and current field lineage was flattened under parse admission.
    pub(super) fn project_causes(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        for index in 0..self.native.causes.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let cause = &self.native.causes.as_slice()[index];
            if let Some(source) = &cause.source {
                c.require_source_range(source)?;
            }
            payload_write::<NativeAssociation>(c, 1)?;
            c.charge(WorkCategory::ProviderCall, 1)?;
            let row = NativeAssociation {
                group: None,
                relation: AssociationRelation::Cause,
                destination: NativeDestination::Cause(index),
                start: 0,
                end: 0,
                source: cause.source.clone(),
                kind: None,
                cause: index,
            };
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .reserve_geometric(c, 1)?;
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .try_push(c, row)?;
        }
        Ok(())
    }
    pub(crate) fn associations(&self) -> Result<AssociationView<'_>, SessionError> {
        if !self.native.projection_complete || self.native.lexical.end.is_none() {
            return Err(SessionError::WrongToken);
        }
        Ok(AssociationView {
            source: self
                .native
                .source
                .as_ref()
                .ok_or(SessionError::WrongToken)?,
            rows: self
                .native
                .projection
                .as_ref()
                .ok_or(SessionError::WrongToken)?
                .as_slice(),
            causes: self.native.causes.as_slice(),
            groups: self.native.groups.as_slice(),
            occurrences: self.native.lexical.occurrences.as_slice(),
            parts: self.native.lexical.parts.as_slice(),
            omissions: self.native.lexical.omissions.as_slice(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_rows(source: &crate::HtmlSource) -> [NativeAssociation; 5] {
        std::array::from_fn(|index| NativeAssociation {
            group: None,
            relation: AssociationRelation::Output,
            destination: NativeDestination::Node(index),
            start: 0,
            end: 0,
            source: match index {
                0 => Some(source.range(0, 1).unwrap()),
                1 => None,
                2 => Some(source.range(1, 1).unwrap()),
                3 => Some(source.range(0, 2).unwrap()),
                _ => Some(source.range(source.len(), source.len()).unwrap()),
            },
            kind: None,
            cause: 0,
        })
    }
    fn make_view<'a>(
        source: &'a crate::HtmlSource,
        rows: &'a [NativeAssociation],
    ) -> AssociationView<'a> {
        AssociationView {
            source,
            rows,
            causes: &[],
            groups: &[],
            occurrences: &[],
            parts: &[],
            omissions: &[],
        }
    }
    #[test]
    fn lineage_keeps_equal_cause_distinct_groups_and_each_current_output_interval() {
        // Real issuer/group/merge producers, not fabricated association rows.
        // All three leaves deliberately share one source-bearing token cause.
        let source = crate::HtmlSource::utf8("ABC");
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
        evidence.initialize_native_source(&mut c, &source).unwrap();
        let range = source.range(0, 3).unwrap();
        let cause = evidence
            .parser_trigger(
                &mut c,
                ParserTrigger {
                    source: &range,
                    rule: ParserRule::Foreign,
                    family: evidence::DiagnosticToken::Text,
                },
            )
            .unwrap();
        let mut leaf = |begin| {
            let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
            origins
                .try_push(
                    &mut c,
                    TextOrigin {
                        start: 0,
                        end: 1,
                        source: source.range(begin, begin + 1).unwrap(),
                        kind: OriginKind::Literal,
                    },
                )
                .unwrap();
            evidence.retain_group(&mut c, origins, 1).unwrap()
        };
        let a = leaf(0);
        let b = leaf(1);
        let third = leaf(2);
        let ab = evidence.merge_groups(&mut c, &a, &b, 1, 2).unwrap();
        let abc = evidence.merge_groups(&mut c, &ab, &third, 2, 3).unwrap();
        let destination = NativeDestination::Node(7);
        evidence.native.projection =
            Some(OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap());
        evidence
            .project_field(&mut c, destination, 3, &abc)
            .unwrap();
        evidence.native.native_rows = 3;
        evidence.project_lineage(&mut c).unwrap();
        evidence.project_causes(&mut c).unwrap();
        let rows = evidence.native.projection.as_ref().unwrap().as_slice();
        assert_eq!(
            rows[..3]
                .iter()
                .map(|row| (row.start, row.end, row.group))
                .collect::<Vec<_>>(),
            vec![
                (0, 1, Some(abc.index)),
                (1, 2, Some(abc.index)),
                (2, 3, Some(abc.index))
            ]
        );
        let mut incoming = rows
            .iter()
            .filter(|row| {
                row.destination == destination && row.relation == AssociationRelation::Incoming
            })
            .map(|row| {
                assert_eq!(row.cause, cause.index);
                let source = row.source.as_ref().unwrap();
                assert_eq!((source.start(), source.end()), (0, 3));
                (row.start, row.end, row.group.unwrap())
            })
            .collect::<Vec<_>>();
        incoming.sort_unstable();
        assert_eq!(
            incoming,
            vec![
                (0, 1, b.index),
                (0, 1, third.index),
                (1, 2, b.index),
                (1, 2, third.index),
                (2, 3, b.index),
                (2, 3, third.index)
            ]
        );
        for interval in [(0, 1), (1, 2), (2, 3)] {
            assert_eq!(
                rows.iter()
                    .filter(|row| row.destination == destination
                        && (row.start, row.end) == interval
                        && row.cause == cause.index
                        && row.group == Some(a.index)
                        && row.relation == AssociationRelation::Original)
                    .count(),
                1
            );
            assert_eq!(
                rows.iter()
                    .filter(|row| row.destination == destination
                        && (row.start, row.end) == interval
                        && row.cause == cause.index
                        && row.group.is_none()
                        && row.relation == AssociationRelation::Trigger)
                    .count(),
                1
            );
        }
        // A point query observes every retained Incoming link independently;
        // each advance visits exactly one fixed row, without following lineage.
        let view = make_view(&source, rows);
        let mut cursor = view.reverse_span(1, 1, None).unwrap();
        let mut matched_incoming = 0;
        for row in rows {
            match cursor.advance() {
                ReverseStep::Match(actual) => {
                    assert!(std::ptr::eq(actual, row));
                    if actual.relation == AssociationRelation::Incoming {
                        matched_incoming += 1;
                    }
                }
                ReverseStep::Skipped => assert!(
                    row.source
                        .as_ref()
                        .is_none_or(|source| !(source.start() <= 1 && 1 < source.end()))
                ),
                _ => panic!("one advance must consume exactly one existing row"),
            }
        }
        assert_eq!(matched_incoming, 6);
        assert!(matches!(cursor.advance(), ReverseStep::Complete));
        drop((cause, a, b, third, ab, abc));
        evidence.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
    }
    fn two_current_destinations(c: &mut ParseControl, source: &crate::HtmlSource) -> ParseEvidence {
        c.admit_source(source).unwrap();
        let mut evidence = ParseEvidence::prepare(c).unwrap();
        evidence.initialize_native_source(c, source).unwrap();
        let range = source.range(0, 1).unwrap();
        evidence
            .parser_trigger(
                c,
                ParserTrigger {
                    source: &range,
                    rule: ParserRule::Foreign,
                    family: evidence::DiagnosticToken::Text,
                },
            )
            .unwrap();
        let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
        origins
            .try_push(
                c,
                TextOrigin {
                    start: 0,
                    end: 1,
                    source: range,
                    kind: OriginKind::Literal,
                },
            )
            .unwrap();
        let group = evidence.retain_group(c, origins, 1).unwrap();
        evidence.native.projection =
            Some(OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap());
        for node in [7, 8] {
            evidence
                .project_field(c, NativeDestination::Node(node), 1, &group)
                .unwrap();
        }
        evidence.native.native_rows = 2;
        evidence
    }

    #[test]
    fn cached_lineage_keeps_both_destinations_and_exact_scratch_one_short_stop() {
        // Request and Token cause slots + one group slot + one source-bearing
        // closure entry + the pending Request trigger overlap at five records.
        // The second current destination reuses that qualified closure.
        for cap in [4, 5] {
            let source = crate::HtmlSource::utf8("A");
            let mut c = ParseControl::new(
                crate::parser::tree_builder::tests::limits(100)
                    .into_builder()
                    .auxiliary_entries(cap)
                    .finish()
                    .unwrap(),
                crate::HtmlParseCancellation::new(),
            );
            let mut evidence = two_current_destinations(&mut c, &source);
            let before = c
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current;
            assert_eq!(before, 0);
            let result = evidence.project_lineage(&mut c);
            if cap == 4 {
                assert_eq!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        limit: 4,
                        current: 4,
                        requested_delta: 1,
                    }))
                );
                assert_eq!(evidence.native.projection.as_ref().unwrap().len(), 2);
                let first = c.first_stop_usage().unwrap().clone();
                assert_eq!(
                    first.resource(HtmlParseResource::AuxiliaryEntries).current,
                    4
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    before
                );
                evidence.release(&mut c).unwrap();
                assert_eq!(c.first_stop_usage(), Some(&first));
            } else {
                result.unwrap();
                let rows = evidence.native.projection.as_ref().unwrap().as_slice();
                assert_eq!(rows.len(), 4);
                for (row, node) in rows[2..].iter().zip([7, 8]) {
                    assert!(row.destination == NativeDestination::Node(node));
                    assert_eq!((row.start, row.end), (0, 1));
                    assert_eq!(row.relation, AssociationRelation::Creation);
                    assert_eq!(row.group, Some(0));
                    assert_eq!(row.cause, 1);
                    assert_eq!(row.source.as_ref(), Some(&source.range(0, 1).unwrap()));
                }
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .high_water,
                    5
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    before
                );
                evidence.release(&mut c).unwrap();
            }
            crate::parser::tree_builder::tests::assert_released(&c);
        }
    }

    #[test]
    fn lineage_cancel_and_foreign_controller_refuse_before_scratch_and_preserve_cleanup() {
        let source = crate::HtmlSource::utf8("A");
        let cancel = crate::HtmlParseCancellation::new();
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            cancel.clone(),
        );
        let mut evidence = two_current_destinations(&mut c, &source);
        let before = c.usage().clone();
        let mut foreign = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        foreign.admit_source(&source).unwrap();
        assert!(matches!(
            evidence.project_lineage(&mut foreign),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(
            foreign
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(c.usage(), &before);
        assert_eq!(evidence.native.projection.as_ref().unwrap().len(), 2);
        // A real original-owner retry is permitted after foreign refusal.
        evidence.project_lineage(&mut c).unwrap();
        cancel.cancel();
        assert_eq!(
            evidence.project_lineage(&mut c),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        let first = c.first_stop_usage().unwrap().clone();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        let work = c.usage().clone();
        evidence.release(&mut c).unwrap();
        for category in [
            WorkCategory::StateStep,
            WorkCategory::InputUnitRead,
            WorkCategory::InspectByte,
            WorkCategory::CopyByte,
            WorkCategory::RecordVisit,
            WorkCategory::RecordWrite,
            WorkCategory::ReserveByte,
            WorkCategory::ProviderCall,
        ] {
            assert_eq!(c.usage().work(category), work.work(category));
        }
        assert_eq!(c.first_stop_usage(), Some(&first));
        crate::parser::tree_builder::tests::assert_released(&c);
    }

    #[test]
    fn reverse_visits_one_literal_row_per_call_with_point_and_eof_rules() {
        let source = crate::HtmlSource::utf8("AB😀");
        let rows = make_rows(&source);
        let view = make_view(&source, &rows);
        let query = source.range(1, 1).unwrap();
        let mut cursor = view.reverse(&query, None).unwrap();
        assert!(matches!(cursor.advance(), ReverseStep::Skipped));
        assert!(matches!(cursor.advance(), ReverseStep::Skipped));
        assert!(matches!(cursor.advance(), ReverseStep::Match(row) if std::ptr::eq(row, &rows[2])));
        assert!(matches!(cursor.advance(), ReverseStep::Match(row) if std::ptr::eq(row, &rows[3])));
        assert!(matches!(cursor.advance(), ReverseStep::Skipped));
        assert!(matches!(cursor.advance(), ReverseStep::Complete));
        assert!(matches!(cursor.advance(), ReverseStep::Complete));
        let mut eof = view.reverse_span(source.len(), source.len(), None).unwrap();
        for _ in 0..4 {
            assert!(matches!(eof.advance(), ReverseStep::Skipped));
        }
        assert!(matches!(eof.advance(), ReverseStep::Match(row) if std::ptr::eq(row, &rows[4])));
        assert!(matches!(eof.advance(), ReverseStep::Complete));
        let mut span = view.reverse_span(0, 1, None).unwrap();
        assert!(matches!(span.advance(), ReverseStep::Match(row) if std::ptr::eq(row, &rows[0])));
        assert!(matches!(span.advance(), ReverseStep::Skipped));
        assert!(matches!(span.advance(), ReverseStep::Skipped));
        assert!(matches!(span.advance(), ReverseStep::Match(row) if std::ptr::eq(row, &rows[3])));
    }
    #[test]
    fn reverse_cancellation_precedes_end_and_stays_terminal_without_result_mutation() {
        let source = crate::HtmlSource::utf8("AB");
        let rows = make_rows(&source);
        let view = make_view(&source, &rows);
        let cancel = crate::HtmlParseCancellation::new();
        let mut cursor = view.reverse_span(0, 2, Some(&cancel)).unwrap();
        for _ in 0..rows.len() {
            cursor.advance();
        }
        cancel.cancel();
        assert!(matches!(cursor.advance(), ReverseStep::Cancelled));
        assert!(matches!(cursor.advance(), ReverseStep::Cancelled));
        assert_eq!(view.len(), 5);
        let cancel = crate::HtmlParseCancellation::new();
        let mut cursor = view.reverse_span(0, 2, Some(&cancel)).unwrap();
        for _ in 0..=rows.len() {
            cursor.advance();
        }
        cancel.cancel();
        assert!(matches!(cursor.advance(), ReverseStep::Complete));
        let mut first = view.reverse_span(0, 2, Some(&cancel)).unwrap();
        assert!(matches!(first.advance(), ReverseStep::Cancelled));
    }
    #[test]
    fn reverse_rejects_foreign_and_scalar_boundaries_but_preserves_utf16_unit_points() {
        let source = crate::HtmlSource::utf8("AB😀");
        let rows = make_rows(&source);
        let view = make_view(&source, &rows);
        assert!(matches!(
            view.reverse_span(3, 4, None),
            Err(ReverseQueryError::Utf8Boundary)
        ));
        assert!(matches!(
            view.reverse_span(2, 7, None),
            Err(ReverseQueryError::OutOfBounds)
        ));
        assert!(matches!(
            view.reverse_span(2, 1, None),
            Err(ReverseQueryError::Reversed)
        ));
        let foreign = crate::HtmlSource::utf8("AB😀").range(0, 1).unwrap();
        assert!(matches!(
            view.reverse(&foreign, None),
            Err(ReverseQueryError::ForeignSource)
        ));
        let source =
            crate::HtmlSource::utf16(DocumentString::from_utf16(vec![0xd83d, 0xde00, 0xd800]));
        let rows = make_rows(&source);
        assert!(make_view(&source, &rows).reverse_span(1, 2, None).is_ok());
    }
}
