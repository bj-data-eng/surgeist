//! Immutable one-row reverse observation of the completed association slice.
//! Parse admission has already produced this slice; readback never resumes it.

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssociationRelation {
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
    RangeDependency(RangeOperandRole),
    ColorDependency(ColorOperandRole),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReverseQueryError {
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
    pub(crate) fn causes_len(&self) -> usize {
        self.causes.len()
    }
    pub(crate) fn lexical_occurrences_len(&self) -> usize {
        self.occurrences.len()
    }
    pub(crate) fn lexical_parts_len(&self) -> usize {
        self.parts.len()
    }
    pub(crate) fn omissions_len(&self) -> usize {
        self.omissions.len()
    }
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
// A field is traversed independently of its direct Output atoms. The retained
// graph is immutable/backward; each work item carries its affected interval.
#[derive(Clone, Copy, Eq, PartialEq)]
struct SpatialLink {
    cause: usize,
    group: Option<usize>,
    relation: AssociationRelation,
    start: usize,
    end: usize,
    historical: bool,
}
impl SpatialLink {
    fn hash(self) -> usize {
        // Numeric qualified keys, not source strings or another tree identity.
        let mut hash = self.cause.wrapping_mul(0x9e3779b1);
        for value in [
            self.group.unwrap_or(usize::MAX),
            match self.relation {
                AssociationRelation::Output => 0,
                AssociationRelation::Creation => 1,
                AssociationRelation::Original => 2,
                AssociationRelation::Incoming => 3,
                AssociationRelation::Trigger => 4,
                AssociationRelation::LexicalEnvelope => 5,
                AssociationRelation::LexicalPart => 6,
                AssociationRelation::LexicalAnchor => 7,
                AssociationRelation::Omission => 8,
                AssociationRelation::Cause => 9,
                AssociationRelation::RangeDependency(role) => 10 + role.index(),
                AssociationRelation::ColorDependency(role) => 14 + role.index(),
            },
            self.start,
            self.end,
            usize::from(self.historical),
        ] {
            hash = hash.rotate_left(7) ^ value.wrapping_mul(0x85ebca6b);
        }
        hash
    }
}
#[derive(Clone, Copy)]
struct SpatialSpan {
    start: usize,
    end: usize,
    begin: usize,
    finish: usize,
}
#[derive(Clone, Copy)]
struct SpatialEntry {
    key: SpatialLink,
    visits: u8,
}
struct SpatialIndex {
    slots: OwnedVec<Option<SpatialEntry>>,
    count: usize,
}
impl SpatialIndex {
    /// Distinguish data versus prior-value traversal while emitting their equal
    /// qualified relation only once. Different groups, roles, or extents stay
    /// different keys; another destination receives its own index.
    fn visit(
        &mut self,
        c: &mut ParseControl,
        mut key: SpatialLink,
    ) -> Result<(bool, bool), SessionError> {
        let mask = if key.historical { 2 } else { 1 };
        key.historical = false;
        // A duplicate never needs another table allocation. Probe the actual
        // initialized slots before the growth branch, including at half load.
        if self.slots.len() != 0 {
            let index = Self::position(&mut self.slots, c, key)?;
            if let Some(entry) = self
                .slots
                .get_mut(index, c)?
                .ok_or(SessionError::WrongToken)?
            {
                c.charge(WorkCategory::InspectByte, mem::size_of::<u8>() as u64)?;
                if entry.visits & mask != 0 {
                    return Ok((false, false));
                }
                payload_write::<u8>(c, 1)?;
                entry.visits |= mask;
                return Ok((true, false));
            }
        }
        if self.slots.len() == 0 || self.count >= self.slots.len() / 2 {
            let size = if self.slots.len() == 0 {
                8
            } else {
                match self.slots.len().checked_mul(2) {
                    Some(size) => size,
                    None => c.representation(HtmlParseResource::AuxiliaryEntries)?,
                }
            };
            let mut next = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
            let grow = (|| {
                next.reserve_geometric(c, size)?;
                for _ in 0..size {
                    next.try_push(c, None)?;
                }
                for slot in self.slots.as_slice() {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    c.charge(
                        WorkCategory::InspectByte,
                        mem::size_of::<Option<SpatialEntry>>() as u64,
                    )?;
                    if let Some(entry) = *slot {
                        let index = Self::position(&mut next, c, entry.key)?;
                        payload_write::<Option<SpatialEntry>>(c, 1)?;
                        *next.get_mut(index, c)?.ok_or(SessionError::WrongToken)? = Some(entry);
                    }
                }
                Ok::<_, SessionError>(())
            })();
            if let Err(error) = grow {
                next.release(c)?;
                return Err(error);
            }
            let old = mem::replace(&mut self.slots, next);
            old.release(c)?;
        }
        let index = Self::position(&mut self.slots, c, key)?;
        let slot = self
            .slots
            .get_mut(index, c)?
            .ok_or(SessionError::WrongToken)?;
        if let Some(entry) = slot {
            c.charge(WorkCategory::InspectByte, mem::size_of::<u8>() as u64)?;
            if entry.visits & mask != 0 {
                return Ok((false, false));
            }
            payload_write::<u8>(c, 1)?;
            entry.visits |= mask;
            Ok((true, false))
        } else {
            payload_write::<Option<SpatialEntry>>(c, 1)?;
            *slot = Some(SpatialEntry { key, visits: mask });
            payload_write::<usize>(c, 1)?;
            self.count += 1;
            Ok((true, true))
        }
    }
    fn position(
        slots: &mut OwnedVec<Option<SpatialEntry>>,
        c: &mut ParseControl,
        key: SpatialLink,
    ) -> Result<usize, SessionError> {
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<SpatialLink>() as u64,
        )?;
        let mut index = key.hash() % slots.len();
        loop {
            let slot = slots.get_mut(index, c)?.ok_or(SessionError::WrongToken)?;
            c.charge(
                WorkCategory::InspectByte,
                mem::size_of::<Option<SpatialEntry>>() as u64,
            )?;
            if slot.is_none() || slot.as_ref().is_some_and(|entry| entry.key == key) {
                return Ok(index);
            }
            index = (index + 1) % slots.len();
        }
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
        let mut reachable = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let mut spans: OwnedVec<Option<SpatialSpan>> =
            OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let result = (|| {
            // The immutable producer only points backward. One admitted bit per
            // cause proves whether a semantic closure can emit any source row.
            // Token chronology is deliberately absent from this proof.
            reachable.reserve_geometric(c, self.native.causes.len())?;
            for (index, cause) in self.native.causes.as_slice().iter().enumerate() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.charge(
                    WorkCategory::InspectByte,
                    mem::size_of::<ConstructionCause>() as u64,
                )?;
                let mut has_source = cause.source.is_some();
                if let Some(source) = &cause.source {
                    c.require_source_range(source)?;
                }
                let trigger = if matches!(
                    cause.operation,
                    ConstructionOperation::Merge
                        | ConstructionOperation::Copy
                        | ConstructionOperation::Replay
                        | ConstructionOperation::Replacement
                ) {
                    cause.trigger
                } else {
                    None
                };
                for prior in [cause.original_cause, trigger] {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if let Some(prior) = prior {
                        if prior >= index {
                            return Err(SessionError::WrongToken);
                        }
                        has_source |= Self::spatial_reachable(c, &reachable, prior)?;
                    }
                }
                for group in [cause.original, cause.incoming] {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if let Some(group) = group {
                        let cause = {
                            c.charge(
                                WorkCategory::InspectByte,
                                mem::size_of::<OriginGroup>() as u64,
                            )?;
                            self.native
                                .groups
                                .as_slice()
                                .get(group)
                                .ok_or(SessionError::WrongToken)?
                                .cause
                        };
                        if cause >= index {
                            return Err(SessionError::WrongToken);
                        }
                        has_source |= Self::spatial_reachable(c, &reachable, cause)?;
                    }
                }
                if let Some(computation) = cause.computation {
                    validate_input_computation(
                        c,
                        computation,
                        self.native.groups.as_slice(),
                        index,
                    )?;
                    let operands = match computation {
                        InputValueComputation::Range(range) => {
                            range.operands.map(RangeOperand::edge)
                        }
                        InputValueComputation::Color(color) => {
                            [color.operands[0], color.operands[1], None, None]
                        }
                    };
                    for edge in operands {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if let Some(edge) = edge {
                            has_source |= Self::spatial_reachable(c, &reachable, edge.cause)?;
                        }
                    }
                }
                reachable.try_push(c, has_source)?;
            }
            let mut index = 0;
            while index < self.native.native_rows {
                c.charge(WorkCategory::RecordVisit, 1)?;
                let row = self
                    .native
                    .projection
                    .as_ref()
                    .ok_or(SessionError::WrongToken)?
                    .as_slice()
                    .get(index)
                    .ok_or(SessionError::WrongToken)?;
                c.charge(
                    WorkCategory::InspectByte,
                    mem::size_of::<NativeAssociation>() as u64,
                )?;
                payload_write::<(NativeDestination, SpatialLink)>(c, 1)?;
                let destination = row.destination;
                let mut root = SpatialLink {
                    cause: row.cause,
                    group: row.group,
                    relation: AssociationRelation::Creation,
                    start: row.start,
                    end: row.end,
                    historical: false,
                };
                index += 1;
                // project_field emits each actual field contiguously. Group and
                // destination qualification remain distinct from atom boundaries.
                if root.group.is_some() {
                    while index < self.native.native_rows {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        let next = &self
                            .native
                            .projection
                            .as_ref()
                            .ok_or(SessionError::WrongToken)?
                            .as_slice()[index];
                        c.charge(
                            WorkCategory::InspectByte,
                            mem::size_of::<NativeAssociation>() as u64,
                        )?;
                        if next.destination != destination || next.group != root.group {
                            break;
                        }
                        if next.cause != root.cause || next.start != root.end {
                            return Err(SessionError::WrongToken);
                        }
                        payload_write::<usize>(c, 1)?;
                        root.end = next.end;
                        index += 1;
                    }
                }
                self.validate_spatial_link(c, root)?;
                if Self::spatial_reachable(c, &reachable, root.cause)? {
                    // Cache only the rows already emitted for an actual current
                    // destination. Reusing a root at another current destination
                    // copies its applicable rows, never rescans its ancestors or
                    // materializes a historical-prefix destination.
                    if spans.len() == 0 {
                        let count = self
                            .native
                            .groups
                            .len()
                            .checked_add(self.native.causes.len())
                            .ok_or(SessionError::WrongToken)?;
                        spans.reserve_geometric(c, count)?;
                        for _ in 0..count {
                            spans.try_push(c, None)?;
                        }
                    }
                    let slot = match root.group {
                        Some(group) => group,
                        None => self
                            .native
                            .groups
                            .len()
                            .checked_add(root.cause)
                            .ok_or(SessionError::WrongToken)?,
                    };
                    c.charge(
                        WorkCategory::InspectByte,
                        mem::size_of::<Option<SpatialSpan>>() as u64,
                    )?;
                    let span = *spans.get_mut(slot, c)?.ok_or(SessionError::WrongToken)?;
                    if let Some(span) = span
                        && (span.start, span.end) == (root.start, root.end)
                    {
                        for row_index in span.begin..span.finish {
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            c.charge(
                                WorkCategory::InspectByte,
                                mem::size_of::<NativeAssociation>() as u64,
                            )?;
                            let original = self
                                .native
                                .projection
                                .as_ref()
                                .ok_or(SessionError::WrongToken)?
                                .as_slice()
                                .get(row_index)
                                .ok_or(SessionError::WrongToken)?;
                            if let Some(source) = &original.source {
                                c.require_source_range(source)?;
                            }
                            payload_write::<NativeAssociation>(c, 1)?;
                            c.charge(WorkCategory::ProviderCall, 1)?;
                            let row = NativeAssociation {
                                destination,
                                group: original.group,
                                relation: original.relation,
                                start: original.start,
                                end: original.end,
                                source: original.source.clone(),
                                kind: original.kind,
                                cause: original.cause,
                            };
                            let rows = self
                                .native
                                .projection
                                .as_mut()
                                .ok_or(SessionError::WrongToken)?;
                            rows.reserve_geometric(c, 1)?;
                            rows.try_push(c, row)?;
                        }
                    } else {
                        let begin = self
                            .native
                            .projection
                            .as_ref()
                            .ok_or(SessionError::WrongToken)?
                            .len();
                        self.project_spatial_field(c, destination, root, &reachable)?;
                        let finish = self
                            .native
                            .projection
                            .as_ref()
                            .ok_or(SessionError::WrongToken)?
                            .len();
                        payload_write::<Option<SpatialSpan>>(c, 1)?;
                        *spans.get_mut(slot, c)?.ok_or(SessionError::WrongToken)? =
                            Some(SpatialSpan {
                                start: root.start,
                                end: root.end,
                                begin,
                                finish,
                            });
                    }
                }
            }
            Ok(())
        })();
        spans.release(c)?;
        reachable.release(c)?;
        result
    }
    fn spatial_reachable(
        c: &mut ParseControl,
        summary: &OwnedVec<bool>,
        index: usize,
    ) -> Result<bool, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, mem::size_of::<bool>() as u64)?;
        summary
            .as_slice()
            .get(index)
            .copied()
            .ok_or(SessionError::WrongToken)
    }
    fn project_spatial_field(
        &mut self,
        c: &mut ParseControl,
        destination: NativeDestination,
        root: SpatialLink,
        reachable: &OwnedVec<bool>,
    ) -> Result<(), SessionError> {
        let mut pending = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let mut seen = SpatialIndex {
            slots: OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
            count: 0,
        };
        let result = (|| {
            pending.reserve_geometric(c, 1)?;
            pending.try_push(c, root)?;
            while let Some(link) = pending.try_pop(c)? {
                self.validate_spatial_link(c, link)?;
                if !Self::spatial_reachable(c, reachable, link.cause)? {
                    continue;
                }
                let (expand, emit) = seen.visit(c, link)?;
                if !expand {
                    continue;
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let cause = self
                    .native
                    .causes
                    .as_slice()
                    .get(link.cause)
                    .ok_or(SessionError::WrongToken)?;
                c.charge(
                    WorkCategory::InspectByte,
                    mem::size_of::<ConstructionCause>() as u64,
                )?;
                if emit && let Some(source) = &cause.source {
                    c.require_source_range(source)?;
                    payload_write::<NativeAssociation>(c, 1)?;
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    let row = NativeAssociation {
                        relation: link.relation,
                        group: link.group,
                        destination,
                        start: link.start,
                        end: link.end,
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
                    rows.try_push(c, row)?;
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let cause = &self.native.causes.as_slice()[link.cause];
                payload_write::<(ConstructionOperation, [Option<usize>; 4])>(c, 1)?;
                let (operation, original, incoming, original_cause, trigger) = (
                    cause.operation,
                    cause.original,
                    cause.incoming,
                    cause.original_cause,
                    cause.trigger,
                );
                if let Some(InputValueComputation::Range(range)) = cause.computation {
                    payload_write::<RangeComputation>(c, 1)?;
                    for role in [
                        RangeOperandRole::Minimum,
                        RangeOperandRole::Maximum,
                        RangeOperandRole::Step,
                        RangeOperandRole::ValueContent,
                    ] {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if let Some(edge) = range.operands[role.index()].edge() {
                            self.push_spatial_group(
                                c,
                                &mut pending,
                                link,
                                (edge.group, AssociationRelation::RangeDependency(role)),
                                link.start..link.end,
                                true,
                            )?;
                        }
                    }
                }
                if let Some(InputValueComputation::Color(color)) = cause.computation {
                    payload_write::<ColorComputation>(c, 1)?;
                    for role in [ColorOperandRole::Alpha, ColorOperandRole::Colorspace] {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if let Some(edge) = color.operands[role.index()] {
                            self.push_spatial_group(
                                c,
                                &mut pending,
                                link,
                                (edge.group, AssociationRelation::ColorDependency(role)),
                                link.start..link.end,
                                true,
                            )?;
                        }
                    }
                }
                let body = if let Some(group) = link.group {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    payload_write::<GroupBody>(c, 1)?;
                    Some(self.native.groups.as_slice()[group].body)
                } else {
                    None
                };
                // Chronological Token.trigger is retained in fixed history only.
                let semantic_trigger = matches!(
                    operation,
                    ConstructionOperation::Merge
                        | ConstructionOperation::Copy
                        | ConstructionOperation::Replay
                        | ConstructionOperation::Replacement
                );
                let mut trigger_start = link.start;
                if !link.historical
                    && let Some(GroupBody::Concat { offset, .. }) = body
                {
                    trigger_start = link
                        .start
                        .checked_add(offset)
                        .ok_or(SessionError::WrongToken)?;
                }
                if semantic_trigger && let Some(cause) = trigger {
                    self.push_spatial_cause(
                        c,
                        &mut pending,
                        link,
                        cause,
                        AssociationRelation::Trigger,
                        (trigger_start)..(link.end),
                    )?;
                }
                if let Some(cause) = original_cause {
                    self.push_spatial_cause(
                        c,
                        &mut pending,
                        link,
                        cause,
                        AssociationRelation::Original,
                        (link.start)..(link.end),
                    )?;
                }
                match body {
                    Some(GroupBody::Concat {
                        original: left,
                        incoming: right,
                        offset,
                    }) => {
                        if original != Some(left)
                            || incoming != Some(right)
                            || operation != ConstructionOperation::Merge
                        {
                            return Err(SessionError::WrongToken);
                        }
                        c.charge(WorkCategory::RecordVisit, 2)?;
                        c.charge(
                            WorkCategory::InspectByte,
                            (2 * mem::size_of::<usize>()) as u64,
                        )?;
                        let left_units = self
                            .native
                            .groups
                            .as_slice()
                            .get(left)
                            .ok_or(SessionError::WrongToken)?
                            .units;
                        let right_units = self
                            .native
                            .groups
                            .as_slice()
                            .get(right)
                            .ok_or(SessionError::WrongToken)?
                            .units;
                        if left_units != offset
                            || offset.checked_add(right_units)
                                != Some(
                                    self.native.groups.as_slice()
                                        [link.group.ok_or(SessionError::WrongToken)?]
                                    .units,
                                )
                        {
                            return Err(SessionError::WrongToken);
                        }
                        let middle = if link.historical {
                            link.start
                        } else {
                            link.start
                                .checked_add(offset)
                                .ok_or(SessionError::WrongToken)?
                        };
                        self.push_spatial_group(
                            c,
                            &mut pending,
                            link,
                            (right, AssociationRelation::Incoming),
                            (middle)..(link.end),
                            link.historical,
                        )?;
                        self.push_spatial_group(
                            c,
                            &mut pending,
                            link,
                            (left, AssociationRelation::Original),
                            (link.start)..(if link.historical { link.end } else { middle }),
                            link.historical,
                        )?;
                    }
                    Some(GroupBody::Alias { original: data }) => {
                        if operation == ConstructionOperation::Replacement {
                            if let Some(old) = original {
                                self.push_spatial_group(
                                    c,
                                    &mut pending,
                                    link,
                                    (old, AssociationRelation::Original),
                                    (link.start)..(link.end),
                                    true,
                                )?;
                            }
                            // Identical field replacement aliases original; reset
                            // aliases its separately retained incoming value.
                            if incoming == Some(data) {
                                self.push_spatial_group(
                                    c,
                                    &mut pending,
                                    link,
                                    (data, AssociationRelation::Incoming),
                                    (link.start)..(link.end),
                                    link.historical,
                                )?;
                            } else if original != Some(data) {
                                return Err(SessionError::WrongToken);
                            }
                        } else {
                            if operation != ConstructionOperation::Copy || original != Some(data) {
                                return Err(SessionError::WrongToken);
                            }
                            self.push_spatial_group(
                                c,
                                &mut pending,
                                link,
                                (data, AssociationRelation::Original),
                                (link.start)..(link.end),
                                link.historical,
                            )?;
                        }
                    }
                    Some(GroupBody::Leaf { .. }) | None => {
                        for (group, relation) in [
                            (incoming, AssociationRelation::Incoming),
                            (original, AssociationRelation::Original),
                        ] {
                            if let Some(group) = group {
                                self.push_spatial_group(
                                    c,
                                    &mut pending,
                                    link,
                                    (group, relation),
                                    (link.start)..(link.end),
                                    true,
                                )?;
                            }
                        }
                    }
                }
            }
            Ok(())
        })();
        pending.release(c)?;
        seen.slots.release(c)?;
        result
    }
    fn validate_spatial_link(
        &self,
        c: &mut ParseControl,
        link: SpatialLink,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if link.start > link.end || self.native.causes.as_slice().get(link.cause).is_none() {
            return Err(SessionError::WrongToken);
        }
        if let Some(index) = link.group {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(
                WorkCategory::InspectByte,
                mem::size_of::<OriginGroup>() as u64,
            )?;
            let group = self
                .native
                .groups
                .as_slice()
                .get(index)
                .ok_or(SessionError::WrongToken)?;
            if group.cause != link.cause
                || (!link.historical && link.end - link.start != group.units)
            {
                return Err(SessionError::WrongToken);
            }
        }
        Ok(())
    }
    fn push_spatial_group(
        &self,
        c: &mut ParseControl,
        pending: &mut OwnedVec<SpatialLink>,
        parent: SpatialLink,
        target: (usize, AssociationRelation),
        extent: std::ops::Range<usize>,
        historical: bool,
    ) -> Result<(), SessionError> {
        let (index, relation) = target;
        let (start, end) = (extent.start, extent.end);
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, mem::size_of::<usize>() as u64)?;
        let group = self
            .native
            .groups
            .as_slice()
            .get(index)
            .ok_or(SessionError::WrongToken)?;
        if parent.group.is_some_and(|parent| index >= parent) || group.cause >= parent.cause {
            return Err(SessionError::WrongToken);
        }
        pending.reserve_geometric(c, 1)?;
        pending.try_push(
            c,
            SpatialLink {
                cause: group.cause,
                group: Some(index),
                relation,
                start,
                end,
                historical,
            },
        )?;
        Ok(())
    }
    fn push_spatial_cause(
        &self,
        c: &mut ParseControl,
        pending: &mut OwnedVec<SpatialLink>,
        parent: SpatialLink,
        cause: usize,
        relation: AssociationRelation,
        extent: std::ops::Range<usize>,
    ) -> Result<(), SessionError> {
        let (start, end) = (extent.start, extent.end);
        if cause >= parent.cause {
            return Err(SessionError::WrongToken);
        }
        pending.reserve_geometric(c, 1)?;
        pending.try_push(
            c,
            SpatialLink {
                cause,
                group: None,
                relation,
                start,
                end,
                historical: true,
            },
        )?;
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
    fn lineage_keeps_shared_cause_groups_at_their_contributed_intervals() {
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
        let original: Vec<_> = rows
            .iter()
            .filter(|row| {
                row.destination == destination
                    && row.cause == cause.index
                    && row.group == Some(a.index)
                    && row.relation == AssociationRelation::Original
            })
            .map(|row| (row.start, row.end))
            .collect();
        let triggers: Vec<_> = rows
            .iter()
            .filter(|row| {
                row.destination == destination
                    && row.cause == cause.index
                    && row.group.is_none()
                    && row.relation == AssociationRelation::Trigger
            })
            .map(|row| (row.start, row.end))
            .collect();
        // Fixed history is retained independently of current interval coverage.
        assert_eq!(evidence.native.groups.len(), 5);
        assert_eq!(evidence.native.causes.len(), 4);
        for (group, left, right, offset) in [(&ab, &a, &b, 1), (&abc, &ab, &third, 2)] {
            let header = evidence.require_group(group).unwrap();
            assert!(
                matches!(header.body, GroupBody::Concat { original, incoming, offset: actual }
                if original == left.index && incoming == right.index && actual == offset)
            );
            let fixed = &evidence.native.causes.as_slice()[header.cause];
            assert_eq!(fixed.operation, ConstructionOperation::Merge);
            assert_eq!(
                (fixed.original, fixed.incoming, fixed.trigger),
                (Some(left.index), Some(right.index), Some(cause.index))
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
        assert!(matches!(cursor.advance(), ReverseStep::Complete));
        let (b_index, c_index) = (b.index, third.index);
        drop((cause, a, b, third, ab, abc));
        evidence.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
        // Architecture-selected spatial applicability. All owning disposal and
        // fixed-history checks precede the intended old-model discrimination.
        assert_eq!(incoming, vec![(1, 2, b_index), (2, 3, c_index)]);
        assert_eq!(original, vec![(0, 1)]);
        // The shared semantic trigger may normalize adjacent coverage, but it
        // cannot retroactively affect A. This is coverage, not a direct map.
        for point in 0..3 {
            assert_eq!(
                triggers
                    .iter()
                    .filter(|(a, b)| *a <= point && point < *b)
                    .count(),
                usize::from(point != 0)
            );
        }
        assert_eq!(matched_incoming, 2);
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
    fn spatial_lineage_keeps_both_destinations_and_exact_scratch_one_short_stop() {
        // Two admitted source-reachability bits plus eight initialized index
        // slots coexist at thirteen records. The root's single pending record has
        // already been popped before those slots initialize. Each destination
        // adds three initialized group/cause span slots. The later destination
        // copies only the first actual destination's emitted applicable rows.
        for cap in [12, 13] {
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
            if cap == 12 {
                assert_eq!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        limit: 12,
                        current: 12,
                        requested_delta: 1,
                    }))
                );
                assert_eq!(evidence.native.projection.as_ref().unwrap().len(), 2);
                let first = c.first_stop_usage().unwrap().clone();
                assert_eq!(
                    first.resource(HtmlParseResource::AuxiliaryEntries).current,
                    12
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
                    13
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
    // These helpers use the actual issuer and group producers. Their direct
    // field projection characterizes spatial algebra; completed ordinary
    // projection remains covered by the published real-driver witness.
    fn spatial_leaf(
        evidence: &mut ParseEvidence,
        c: &mut ParseControl,
        source: &crate::HtmlSource,
        input: std::ops::Range<usize>,
    ) -> OriginGroupRef {
        let range = source.range(input.start, input.end).unwrap();
        let cause = evidence
            .parser_trigger(
                c,
                ParserTrigger {
                    source: &range,
                    rule: ParserRule::Foreign,
                    family: evidence::DiagnosticToken::Text,
                },
            )
            .unwrap();
        drop(cause);
        let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
        if !input.is_empty() {
            origins
                .try_push(
                    c,
                    TextOrigin {
                        start: 0,
                        end: input.len(),
                        source: range,
                        kind: OriginKind::Literal,
                    },
                )
                .unwrap();
        }
        evidence.retain_group(c, origins, input.len()).unwrap()
    }
    fn spatial_trigger(
        evidence: &mut ParseEvidence,
        c: &mut ParseControl,
        source: &crate::HtmlSource,
        at: usize,
    ) -> ConstructionCauseRef {
        evidence
            .parser_trigger(
                c,
                ParserTrigger {
                    source: &source.range(at, at + 1).unwrap(),
                    rule: ParserRule::Foreign,
                    family: evidence::DiagnosticToken::Text,
                },
            )
            .unwrap()
    }
    fn spatial_project(
        evidence: &mut ParseEvidence,
        c: &mut ParseControl,
        fields: &[(usize, &OriginGroupRef)],
    ) {
        evidence.native.projection =
            Some(OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap());
        for (node, group) in fields {
            let units = evidence.require_group(group).unwrap().units;
            evidence
                .project_field(c, NativeDestination::Node(*node), units, group)
                .unwrap();
        }
        evidence.native.native_rows = evidence.native.projection.as_ref().unwrap().len();
        evidence.project_lineage(c).unwrap();
    }
    type SpatialObservation = (
        usize,
        AssociationRelation,
        usize,
        usize,
        usize,
        usize,
        Option<usize>,
        usize,
    );
    fn spatial_rows(evidence: &ParseEvidence) -> Vec<SpatialObservation> {
        evidence.native.projection.as_ref().unwrap().as_slice()[evidence.native.native_rows..]
            .iter()
            .map(|row| {
                let NativeDestination::Node(node) = row.destination else {
                    panic!("literal node destination");
                };
                let source = row.source.as_ref().unwrap();
                (
                    node,
                    row.relation,
                    row.start,
                    row.end,
                    source.start(),
                    source.end(),
                    row.group,
                    row.cause,
                )
            })
            .collect()
    }
    #[test]
    fn spatial_copy_rebases_original_and_copy_trigger_without_spreading_later_append() {
        let source = crate::HtmlSource::utf8("ABXC");
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
        evidence.initialize_native_source(&mut c, &source).unwrap();
        let ab = spatial_leaf(&mut evidence, &mut c, &source, 0..2);
        let copy_trigger = spatial_trigger(&mut evidence, &mut c, &source, 2);
        let copied = evidence.copy_group(&mut c, &ab).unwrap();
        let incoming = spatial_leaf(&mut evidence, &mut c, &source, 3..4);
        let joined = evidence
            .merge_groups(&mut c, &copied, &incoming, 2, 3)
            .unwrap();
        spatial_project(&mut evidence, &mut c, &[(7, &joined), (8, &joined)]);
        let rows = spatial_rows(&evidence);
        let original_cause = evidence.require_group(&ab).unwrap().cause;
        let incoming_cause = evidence.require_group(&incoming).unwrap().cause;
        assert_eq!(evidence.history_counts(), (2, 4, 6));
        assert_eq!(evidence.require_group(&ab).unwrap().units, 2);
        assert!(
            matches!(evidence.require_group(&copied).unwrap().body, GroupBody::Alias { original } if original == ab.index)
        );
        assert_eq!(
            evidence.native.causes.as_slice()[incoming_cause].trigger,
            Some(copy_trigger.index)
        );
        evidence.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
        assert_eq!(rows.len(), 8);
        for node in [7, 8] {
            for expected in [
                (
                    node,
                    AssociationRelation::Original,
                    0,
                    2,
                    0,
                    2,
                    Some(ab.index),
                    original_cause,
                ),
                (
                    node,
                    AssociationRelation::Incoming,
                    2,
                    3,
                    3,
                    4,
                    Some(incoming.index),
                    incoming_cause,
                ),
                (
                    node,
                    AssociationRelation::Trigger,
                    0,
                    2,
                    2,
                    3,
                    None,
                    copy_trigger.index,
                ),
                (
                    node,
                    AssociationRelation::Trigger,
                    2,
                    3,
                    3,
                    4,
                    None,
                    incoming_cause,
                ),
            ] {
                assert_eq!(rows.iter().filter(|row| **row == expected).count(), 1);
            }
        }
    }
    #[test]
    fn spatial_reused_leaf_keeps_disjoint_intervals_and_empty_append_anchor() {
        let source = crate::HtmlSource::utf8("AT");
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
        evidence.initialize_native_source(&mut c, &source).unwrap();
        let a = spatial_leaf(&mut evidence, &mut c, &source, 0..1);
        let gap = evidence.source_independent_group(&mut c, 2).unwrap();
        let agap = evidence.merge_groups(&mut c, &a, &gap, 1, 3).unwrap();
        let with_a = evidence.merge_groups(&mut c, &agap, &a, 3, 4).unwrap();
        let trigger = spatial_trigger(&mut evidence, &mut c, &source, 1);
        let empty = evidence.rule_group(&mut c, 0).unwrap();
        let final_group = evidence
            .merge_groups(&mut c, &with_a, &empty, 4, 4)
            .unwrap();
        spatial_project(&mut evidence, &mut c, &[(7, &final_group)]);
        let rows = spatial_rows(&evidence);
        let a_cause = evidence.require_group(&a).unwrap().cause;
        assert_eq!(evidence.require_group(&final_group).unwrap().units, 4);
        evidence.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
        assert_eq!(
            rows,
            vec![
                (
                    7,
                    AssociationRelation::Original,
                    0,
                    1,
                    0,
                    1,
                    Some(a.index),
                    a_cause
                ),
                (7, AssociationRelation::Trigger, 1, 3, 0, 1, None, a_cause),
                (
                    7,
                    AssociationRelation::Incoming,
                    3,
                    4,
                    0,
                    1,
                    Some(a.index),
                    a_cause
                ),
                (7, AssociationRelation::Trigger, 3, 4, 0, 1, None, a_cause),
                (
                    7,
                    AssociationRelation::Incoming,
                    4,
                    4,
                    1,
                    2,
                    Some(empty.index),
                    trigger.index
                ),
                (
                    7,
                    AssociationRelation::Trigger,
                    4,
                    4,
                    1,
                    2,
                    None,
                    trigger.index
                ),
            ]
        );
    }
    #[test]
    fn spatial_token_predecessor_remains_history_without_unrelated_field_trigger() {
        let source = crate::HtmlSource::utf8("AB");
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
        evidence.initialize_native_source(&mut c, &source).unwrap();
        let a = spatial_leaf(&mut evidence, &mut c, &source, 0..1);
        let b = spatial_leaf(&mut evidence, &mut c, &source, 1..2);
        let ca = evidence.require_group(&a).unwrap().cause;
        let cb = evidence.require_group(&b).unwrap().cause;
        assert_eq!(evidence.native.causes.as_slice()[cb].trigger, Some(ca));
        spatial_project(&mut evidence, &mut c, &[(7, &a), (8, &b)]);
        let rows = spatial_rows(&evidence);
        evidence.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
        assert_eq!(
            rows,
            vec![
                (
                    7,
                    AssociationRelation::Creation,
                    0,
                    1,
                    0,
                    1,
                    Some(a.index),
                    ca
                ),
                (
                    8,
                    AssociationRelation::Creation,
                    0,
                    1,
                    1,
                    2,
                    Some(b.index),
                    cb
                ),
            ]
        );
    }
    #[test]
    fn spatial_buffered_replay_preserves_actual_original_and_later_trigger_on_created_field() {
        let source = crate::HtmlSource::utf8("AT");
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let original = spatial_trigger(session.evidence(&mut c).unwrap(), &mut c, &source, 0);
        let trigger = spatial_trigger(session.evidence(&mut c).unwrap(), &mut c, &source, 1);
        let previous = session
            .begin_buffered_replay(&mut c, Some(&original))
            .unwrap();
        let group = session
            .evidence(&mut c)
            .unwrap()
            .rule_group(&mut c, 3)
            .unwrap();
        session
            .finish_buffered_replay(&mut c, previous.as_ref())
            .unwrap();
        spatial_project(session.evidence(&mut c).unwrap(), &mut c, &[(7, &group)]);
        let evidence = session.evidence(&mut c).unwrap();
        let rows = spatial_rows(evidence);
        let cause =
            &evidence.native.causes.as_slice()[evidence.require_group(&group).unwrap().cause];
        assert_eq!(cause.operation, ConstructionOperation::Replay);
        assert_eq!(cause.original_cause, Some(original.index));
        assert_eq!(cause.trigger, Some(trigger.index));
        session.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
        assert_eq!(
            rows,
            vec![
                (
                    7,
                    AssociationRelation::Original,
                    0,
                    3,
                    0,
                    1,
                    None,
                    original.index
                ),
                (
                    7,
                    AssociationRelation::Trigger,
                    0,
                    3,
                    1,
                    2,
                    None,
                    trigger.index
                ),
            ]
        );
    }
    #[test]
    fn spatial_reset_retains_old_value_as_cause_of_new_extent_and_empty_anchor() {
        for empty in [false, true] {
            let source = crate::HtmlSource::utf8("ABCT");
            let mut c = ParseControl::new(
                crate::parser::tree_builder::tests::limits(100),
                crate::HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            session.initialize_parse_evidence(&mut c, &source).unwrap();
            let old = spatial_leaf(session.evidence(&mut c).unwrap(), &mut c, &source, 0..2);
            let old_id = old.index;
            let old_cause = session
                .evidence(&mut c)
                .unwrap()
                .require_group(&old)
                .unwrap()
                .cause;
            let mut value = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
            value.extend_units(&[65, 66], &mut c).unwrap();
            let value = value.freeze(&mut c).unwrap();
            let (alias, slot) = session
                .active_mut()
                .unwrap()
                .register_group_string_owned(&mut c, value, Some(old))
                .unwrap();
            drop(alias);
            let incoming = if empty {
                None
            } else {
                Some(spatial_leaf(
                    session.evidence(&mut c).unwrap(),
                    &mut c,
                    &source,
                    2..3,
                ))
            };
            let incoming_id = incoming.as_ref().map(|group| group.index);
            let incoming_cause = incoming.as_ref().map(|group| {
                session
                    .evidence(&mut c)
                    .unwrap()
                    .require_group(group)
                    .unwrap()
                    .cause
            });
            let trigger = spatial_trigger(session.evidence(&mut c).unwrap(), &mut c, &source, 3);
            let units = usize::from(!empty);
            let reset = session
                .active_mut()
                .unwrap()
                .reset_group(&mut c, slot, incoming, units)
                .unwrap()
                .unwrap();
            session
                .active_mut()
                .unwrap()
                .retire_field_group(&mut c, slot)
                .unwrap();
            spatial_project(session.evidence(&mut c).unwrap(), &mut c, &[(7, &reset)]);
            let evidence = session.evidence(&mut c).unwrap();
            let rows = spatial_rows(evidence);
            assert_eq!(evidence.native.groups.as_slice()[old_id].units, 2);
            assert!(
                evidence.native.groups.as_slice()[old_id]
                    .retirement
                    .is_some()
            );
            let reset_cause =
                &evidence.native.causes.as_slice()[evidence.require_group(&reset).unwrap().cause];
            assert_eq!(reset_cause.operation, ConstructionOperation::Replacement);
            assert_eq!(reset_cause.original, Some(old_id));
            let actual_incoming = reset_cause.incoming.unwrap();
            session.release(&mut c).unwrap();
            crate::parser::tree_builder::tests::assert_released(&c);
            assert_eq!(rows.len(), 3);
            for expected in [
                (
                    7,
                    AssociationRelation::Original,
                    0,
                    units,
                    0,
                    2,
                    Some(old_id),
                    old_cause,
                ),
                (
                    7,
                    AssociationRelation::Incoming,
                    0,
                    units,
                    if empty { 3 } else { 2 },
                    if empty { 4 } else { 3 },
                    Some(incoming_id.unwrap_or(actual_incoming)),
                    incoming_cause.unwrap_or(trigger.index),
                ),
                (
                    7,
                    AssociationRelation::Trigger,
                    0,
                    units,
                    3,
                    4,
                    None,
                    trigger.index,
                ),
            ] {
                assert_eq!(rows.iter().filter(|row| **row == expected).count(), 1);
            }
        }
    }
    #[test]
    fn spatial_output_one_short_preserves_canonical_stop_and_reclaims_all_scratch() {
        // Literal fixture has one origin, one group, two causes, and two native
        // Output rows (six retained entries). Two actual lineage rows make eight.
        for cap in [7, 8] {
            let source = crate::HtmlSource::utf8("A");
            let mut c = ParseControl::new(
                crate::parser::tree_builder::tests::limits(100)
                    .into_builder()
                    .source_map_entries(cap)
                    .finish()
                    .unwrap(),
                crate::HtmlParseCancellation::new(),
            );
            let mut evidence = two_current_destinations(&mut c, &source);
            let result = evidence.project_lineage(&mut c);
            if cap == 7 {
                assert_eq!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::SourceMapEntries,
                        limit: 7,
                        current: 7,
                        requested_delta: 1,
                    }))
                );
                let first = c.first_stop_usage().unwrap().clone();
                assert_eq!(
                    first.resource(HtmlParseResource::SourceMapEntries).current,
                    7
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    0
                );
                assert_eq!(
                    evidence.project_lineage(&mut c),
                    Err(SessionError::Stop(c.stop().unwrap()))
                );
                assert_eq!(c.first_stop_usage(), Some(&first));
                evidence.release(&mut c).unwrap();
                assert_eq!(c.first_stop_usage(), Some(&first));
            } else {
                result.unwrap();
                assert_eq!(spatial_rows(&evidence).len(), 2);
                evidence.release(&mut c).unwrap();
            }
            crate::parser::tree_builder::tests::assert_released(&c);
        }
    }
    #[test]
    fn spatial_repeated_copy_trigger_normalizes_only_equal_coverage_and_keeps_both_operations() {
        // Request, A Token, T Token and two real Copy causes give five summary
        // bits. Eight index slots and three simultaneously pending links give
        // the exact twenty-four-entry overlap, including eight root-span slots; a duplicate at half load allocates
        // no second table. Two qualified rows retain Original A and Trigger T.
        for cap in [23, 24] {
            let source = crate::HtmlSource::utf8("AT");
            let mut c = ParseControl::new(
                crate::parser::tree_builder::tests::limits(100)
                    .into_builder()
                    .auxiliary_entries(cap)
                    .finish()
                    .unwrap(),
                crate::HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
            evidence.initialize_native_source(&mut c, &source).unwrap();
            let a = spatial_leaf(&mut evidence, &mut c, &source, 0..1);
            let trigger = spatial_trigger(&mut evidence, &mut c, &source, 1);
            let once = evidence.copy_group(&mut c, &a).unwrap();
            let twice = evidence.copy_group(&mut c, &once).unwrap();
            evidence.native.projection =
                Some(OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap());
            evidence
                .project_field(&mut c, NativeDestination::Node(7), 1, &twice)
                .unwrap();
            evidence.native.native_rows = 1;
            let result = evidence.project_lineage(&mut c);
            assert_eq!(evidence.history_counts(), (1, 3, 5));
            for (group, original) in [(&once, a.index), (&twice, once.index)] {
                let cause = &evidence.native.causes.as_slice()
                    [evidence.require_group(group).unwrap().cause];
                assert_eq!(cause.operation, ConstructionOperation::Copy);
                assert_eq!(cause.original, Some(original));
                assert_eq!(cause.trigger, Some(trigger.index));
            }
            if cap == 23 {
                assert_eq!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        limit: 23,
                        current: 23,
                        requested_delta: 1,
                    }))
                );
                let first = c.first_stop_usage().unwrap().clone();
                assert_eq!(
                    first.resource(HtmlParseResource::AuxiliaryEntries).current,
                    23
                );
                assert_eq!(evidence.native.projection.as_ref().unwrap().len(), 1);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    0
                );
                evidence.release(&mut c).unwrap();
                assert_eq!(c.first_stop_usage(), Some(&first));
                crate::parser::tree_builder::tests::assert_released(&c);
            } else {
                result.unwrap();
                let rows = spatial_rows(&evidence);
                let cause = evidence.require_group(&a).unwrap().cause;
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .high_water,
                    24
                );
                evidence.release(&mut c).unwrap();
                crate::parser::tree_builder::tests::assert_released(&c);
                assert_eq!(
                    rows,
                    vec![
                        (
                            7,
                            AssociationRelation::Original,
                            0,
                            1,
                            0,
                            1,
                            Some(a.index),
                            cause
                        ),
                        (
                            7,
                            AssociationRelation::Trigger,
                            0,
                            1,
                            1,
                            2,
                            None,
                            trigger.index
                        ),
                    ]
                );
            }
        }
    }
    #[test]
    fn spatial_index_resize_exact_overlap_and_one_short_preserve_first_stop_cleanup() {
        // Three Token leaves and two Merge versions follow one Request: six
        // summary bits. At the first resize, the old eight initialized slots,
        // new sixteen initialized slots, eleven root-span slots and two pending links
        // coexist: 43.
        for cap in [42, 43] {
            let source = crate::HtmlSource::utf8("ABC");
            let mut c = ParseControl::new(
                crate::parser::tree_builder::tests::limits(100)
                    .into_builder()
                    .auxiliary_entries(cap)
                    .finish()
                    .unwrap(),
                crate::HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
            evidence.initialize_native_source(&mut c, &source).unwrap();
            let a = spatial_leaf(&mut evidence, &mut c, &source, 0..1);
            let b = spatial_leaf(&mut evidence, &mut c, &source, 1..2);
            let ab = evidence.merge_groups(&mut c, &a, &b, 1, 2).unwrap();
            let third = spatial_leaf(&mut evidence, &mut c, &source, 2..3);
            let abc = evidence.merge_groups(&mut c, &ab, &third, 2, 3).unwrap();
            evidence.native.projection =
                Some(OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap());
            evidence
                .project_field(&mut c, NativeDestination::Node(7), 3, &abc)
                .unwrap();
            evidence.native.native_rows = 3;
            let result = evidence.project_lineage(&mut c);
            assert_eq!(evidence.history_counts(), (3, 5, 6));
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                0
            );
            if cap == 42 {
                assert_eq!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        limit: 42,
                        current: 42,
                        requested_delta: 1,
                    }))
                );
                let first = c.first_stop_usage().unwrap().clone();
                assert_eq!(
                    first.resource(HtmlParseResource::AuxiliaryEntries).current,
                    42
                );
                assert_eq!(evidence.native.projection.as_ref().unwrap().len(), 5);
                let after_cleanup = c.usage().clone();
                assert_eq!(
                    evidence.project_lineage(&mut c),
                    Err(SessionError::Stop(c.stop().unwrap()))
                );
                assert_eq!(c.first_stop_usage(), Some(&first));
                assert_eq!(c.usage(), &after_cleanup);
                evidence.release(&mut c).unwrap();
                assert_eq!(c.first_stop_usage(), Some(&first));
                crate::parser::tree_builder::tests::assert_released(&c);
            } else {
                result.unwrap();
                let rows = spatial_rows(&evidence);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .high_water,
                    43
                );
                evidence.release(&mut c).unwrap();
                crate::parser::tree_builder::tests::assert_released(&c);
                assert_eq!(rows.len(), 5);
                for (role, start, end, begin, finish, group) in [
                    (AssociationRelation::Original, 0, 1, 0, 1, Some(a.index)),
                    (AssociationRelation::Incoming, 1, 2, 1, 2, Some(b.index)),
                    (AssociationRelation::Incoming, 2, 3, 2, 3, Some(third.index)),
                    (AssociationRelation::Trigger, 1, 2, 1, 2, None),
                    (AssociationRelation::Trigger, 2, 3, 2, 3, None),
                ] {
                    assert_eq!(
                        rows.iter()
                            .filter(|row| (row.1, row.2, row.3, row.4, row.5, row.6)
                                == (role, start, end, begin, finish, group))
                            .count(),
                        1
                    );
                }
            }
        }
    }
    #[test]
    fn spatial_transformed_copy_retains_old_history_and_exact_produced_maps() {
        let source = crate::HtmlSource::utf8("AT");
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let original = spatial_leaf(session.evidence(&mut c).unwrap(), &mut c, &source, 0..1);
        let original_id = original.index;
        let original_cause = session
            .evidence(&mut c)
            .unwrap()
            .require_group(&original)
            .unwrap()
            .cause;
        let mut old_value = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
        old_value.push_unit(65, &mut c).unwrap();
        let old_value = old_value.freeze(&mut c).unwrap();
        let (alias, old_slot) = session
            .active_mut()
            .unwrap()
            .register_group_string_owned(&mut c, old_value, Some(original))
            .unwrap();
        drop(alias);
        let copied = session
            .active_mut()
            .unwrap()
            .copied_slot_group(&mut c, old_slot)
            .unwrap();
        let trigger = spatial_trigger(session.evidence(&mut c).unwrap(), &mut c, &source, 1);
        let mut new_value = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
        new_value.push_unit(97, &mut c).unwrap();
        let new_value = new_value.freeze(&mut c).unwrap();
        let mut maps = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
        maps.try_push(
            &mut c,
            TextOrigin {
                start: 0,
                end: 1,
                source: source.range(0, 1).unwrap(),
                kind: OriginKind::AsciiLowercase,
            },
        )
        .unwrap();
        let (alias, new_slot) = session
            .active_mut()
            .unwrap()
            .register_field_copy(
                &mut c,
                FieldCopy {
                    value: new_value,
                    origins: maps,
                    original: copied,
                },
            )
            .unwrap();
        assert_eq!(alias.code_units(), &[97]);
        drop(alias);
        session
            .active_mut()
            .unwrap()
            .retire_field_group(&mut c, old_slot)
            .unwrap();
        let storage = session.active_mut().unwrap();
        let RetainedPayload::String {
            origins: Some(group),
            ..
        } = &storage.payloads.values[new_slot]
        else {
            panic!("actual transformed payload");
        };
        let group = storage
            .evidence
            .as_ref()
            .unwrap()
            .copy_group_reference(&mut c, group)
            .unwrap();
        let origins = storage
            .evidence
            .as_ref()
            .unwrap()
            .observed_origins(&group, &mut c)
            .unwrap();
        assert_eq!(origins.len(), 1);
        assert_eq!(
            (origins[0].start, origins[0].end, origins[0].kind),
            (0, 1, OriginKind::AsciiLowercase)
        );
        assert_eq!(origins[0].source, source.range(0, 1).unwrap());
        let mut materialized = Some(
            storage
                .evidence
                .as_ref()
                .unwrap()
                .materialize_group(&mut c, &storage.payloads, &group)
                .unwrap(),
        );
        assert_eq!(materialized.as_ref().unwrap().text().code_units(), &[97]);
        ParseEvidence::release_materialized_group(&mut c, &mut materialized).unwrap();
        assert!(materialized.is_none());
        spatial_project(session.evidence(&mut c).unwrap(), &mut c, &[(7, &group)]);
        let evidence = session.evidence(&mut c).unwrap();
        let rows = spatial_rows(evidence);
        let header = evidence.require_group(&group).unwrap();
        assert!(matches!(header.body, GroupBody::Leaf { .. }));
        assert_eq!(
            evidence.native.causes.as_slice()[header.cause].operation,
            ConstructionOperation::Replacement
        );
        assert!(
            evidence.native.groups.as_slice()[original_id]
                .retirement
                .is_some()
        );
        let direct = &evidence.native.projection.as_ref().unwrap().as_slice()[0];
        assert_eq!(direct.kind, Some(OriginKind::AsciiLowercase));
        assert_eq!(direct.source, Some(source.range(0, 1).unwrap()));
        assert_eq!(direct.group, Some(group.index));
        session.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
        assert_eq!(rows.len(), 3);
        for expected in [
            (
                7,
                AssociationRelation::Original,
                0,
                1,
                0,
                1,
                Some(original_id),
                original_cause,
            ),
            (
                7,
                AssociationRelation::Trigger,
                0,
                1,
                0,
                1,
                None,
                original_cause,
            ),
            (
                7,
                AssociationRelation::Trigger,
                0,
                1,
                1,
                2,
                None,
                trigger.index,
            ),
        ] {
            assert_eq!(rows.iter().filter(|row| **row == expected).count(), 1);
        }
    }
    #[test]
    fn spatial_work_one_short_stops_real_writer_and_keeps_canonical_snapshot_through_cleanup() {
        let source = crate::HtmlSource::utf8("A");
        let mut full = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        let mut evidence = two_current_destinations(&mut full, &source);
        let before = full
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        evidence.project_lineage(&mut full).unwrap();
        let admitted = full
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        assert!(admitted > before);
        assert_eq!(spatial_rows(&evidence).len(), 2);
        evidence.release(&mut full).unwrap();
        crate::parser::tree_builder::tests::assert_released(&full);
        // Same literal, real writer and physical layouts; cut one actual credit
        // before the successful endpoint rather than invent a callback/hook.
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100)
                .into_builder()
                .work_credits(admitted - 1)
                .finish()
                .unwrap(),
            crate::HtmlParseCancellation::new(),
        );
        let mut evidence = two_current_destinations(&mut c, &source);
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            before
        );
        assert!(matches!(
            evidence.project_lineage(&mut c),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }))
        ));
        let first = c.first_stop_usage().unwrap().clone();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        let terminal = c.usage().clone();
        assert_eq!(
            evidence.project_lineage(&mut c),
            Err(SessionError::Stop(c.stop().unwrap()))
        );
        assert_eq!(c.usage(), &terminal);
        evidence.release(&mut c).unwrap();
        assert_eq!(c.first_stop_usage(), Some(&first));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits),
            terminal.resource(HtmlParseResource::WorkCredits)
        );
        crate::parser::tree_builder::tests::assert_released(&c);
    }
    #[test]
    fn spatial_shared_current_root_reuses_only_actual_emitted_span_across_destinations() {
        // One source A, 64 actual Copy aliases and 64 current destinations.
        // Each Copy preserves its previous group and the actual A trigger.
        // The only applicable rows per destination are Original A and Trigger A;
        // fixed histories remain 65 groups and 66 causes. Useful lineage visits
        // are bounded by a generous 128 per unique Copy/current destination,
        // not by the product of those two populations. Direct field projection
        // is deliberately outside this phase observation.
        const COPIES: usize = 64;
        const DESTINATIONS: usize = 64;
        let source = crate::HtmlSource::utf8("A");
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(10_000),
            crate::HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
        evidence.initialize_native_source(&mut c, &source).unwrap();
        let original = spatial_leaf(&mut evidence, &mut c, &source, 0..1);
        let original_cause = evidence.require_group(&original).unwrap().cause;
        let mut group = evidence.copy_group(&mut c, &original).unwrap();
        for _ in 1..COPIES {
            let prior = group.index;
            group = evidence.copy_group(&mut c, &group).unwrap();
            let cause =
                &evidence.native.causes.as_slice()[evidence.require_group(&group).unwrap().cause];
            assert_eq!(
                (cause.operation, cause.original, cause.trigger),
                (
                    ConstructionOperation::Copy,
                    Some(prior),
                    Some(original_cause)
                )
            );
        }
        evidence.native.projection =
            Some(OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap());
        for node in 0..DESTINATIONS {
            evidence
                .project_field(&mut c, NativeDestination::Node(node), 1, &group)
                .unwrap();
        }
        evidence.native.native_rows = DESTINATIONS;
        let before = c.usage().work(WorkCategory::RecordVisit);
        evidence.project_lineage(&mut c).unwrap();
        let visits = c.usage().work(WorkCategory::RecordVisit) - before;
        let rows = spatial_rows(&evidence);
        assert_eq!(evidence.history_counts(), (1, COPIES + 1, COPIES + 2));
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        evidence.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
        assert_eq!(rows.len(), 2 * DESTINATIONS);
        for (node, pair) in rows.chunks_exact(2).enumerate() {
            assert_eq!(
                pair,
                &[
                    (
                        node,
                        AssociationRelation::Original,
                        0,
                        1,
                        0,
                        1,
                        Some(original.index),
                        original_cause
                    ),
                    (
                        node,
                        AssociationRelation::Trigger,
                        0,
                        1,
                        0,
                        1,
                        None,
                        original_cause
                    ),
                ]
            );
        }
        assert!(
            visits <= 128 * (COPIES + DESTINATIONS + 1) as u64,
            "{visits} visits must reuse already-emitted current-root rows"
        );
    }
}
