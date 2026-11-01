//! Immutable history reads over the actual frozen arena and evidence owner.
//!
//! Qualification is metered before artifact transfer. Reads allocate nothing: inventory
//! and atom access are fixed, while traversal advances one graph header using
//! caller-owned scratch. This private owner is not a public parse-success seal.

use super::*;
use provenance::{ConstructionCause, GroupBody, HistoryData, LeafValue, OriginGroup};

/// Only the metered history qualification paths construct this owner. Generic native freeze does not
/// establish the complete D1/lexical and history-payload qualification.
pub(crate) struct FrozenHistory {
    arena: FrozenArena,
}

impl ParserArenaSession {
    #[cfg(test)]
    pub(crate) fn freeze_history(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<FrozenHistory, SessionError> {
        let storage = self.active()?;
        storage.require(c)?;
        storage.payloads.check_aliases(
            ConstructionView::Active(storage.construction.test_storage()),
            c,
        )?;
        let evidence = storage.evidence.as_ref().ok_or(SessionError::WrongToken)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        payload_write::<HistoryData<'_>>(c, 1)?;
        let data = evidence.history_data()?;
        qualify(&storage.payloads, &data, c)?;
        // Admit the actual new wrapper before the existing recoverable arena
        // transfer. There is no fallible operation after successful freeze.
        payload_write::<FrozenHistory>(c, 1)?;
        Ok(FrozenHistory {
            arena: self.freeze(c)?,
        })
    }
}

fn header(c: &mut ParseControl) -> Result<(), SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    c.charge(
        WorkCategory::InspectByte,
        mem::size_of::<OriginGroup>() as u64,
    )?;
    Ok(())
}

fn source_range(
    c: &mut ParseControl,
    source: &crate::HtmlSource,
    range: &crate::HtmlSourceRange,
) -> Result<(), SessionError> {
    c.charge(
        WorkCategory::InspectByte,
        mem::size_of::<crate::HtmlSourceRange>() as u64,
    )?;
    c.require_source_range(range)?;
    if range.start() > range.end() || range.end() > source.len() {
        return Err(SessionError::WrongToken);
    }
    // HtmlSourceRange's checked constructor already qualifies endpoint encoding;
    // same immutable source identity/basis above preserves that qualification.
    Ok(())
}

fn qualify(
    payloads: &PayloadStorage,
    data: &HistoryData<'_>,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    c.require_source(data.source)?;
    for (index, cause) in data.causes.iter().enumerate() {
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<ConstructionCause>() as u64,
        )?;
        if let Some(range) = &cause.source {
            source_range(c, data.source, range)?;
        }
        if cause
            .original_cause
            .is_some_and(|original| original >= index)
            || cause.trigger.is_some_and(|trigger| trigger >= index)
            || cause
                .original
                .is_some_and(|group| group >= data.groups.len())
            || cause
                .incoming
                .is_some_and(|group| group >= data.groups.len())
        {
            return Err(SessionError::WrongToken);
        }
        if let Some(range) = cause.computation {
            if cause.operation != provenance::ConstructionOperation::Replacement {
                return Err(SessionError::WrongToken);
            }
            provenance::validate_input_computation(c, range, data.groups, index)?;
        }
        // Captured qualifications were established by the sealed ingress;
        // captured facts remain nonlexical and do not acquire a source range.
    }
    for (index, group) in data.groups.iter().enumerate() {
        header(c)?;
        if group.cause >= data.causes.len()
            || group
                .retirement
                .is_some_and(|cause| cause >= data.causes.len())
        {
            return Err(SessionError::WrongToken);
        }
        if let Some(range) = data.causes[group.cause].computation {
            for edge in range.edges() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if edge.is_some_and(|edge| edge.group >= index) {
                    return Err(SessionError::WrongToken);
                }
            }
        }
        match group.body {
            GroupBody::Leaf {
                start,
                length,
                value,
            } => {
                let end = start.checked_add(length).ok_or(SessionError::WrongToken)?;
                if end > data.origins.len() {
                    return Err(SessionError::WrongToken);
                }
                match value {
                    LeafValue::Pending => return Err(SessionError::WrongToken),
                    LeafValue::Empty if group.units == 0 => {}
                    LeafValue::Empty => return Err(SessionError::WrongToken),
                    LeafValue::Payload(slot) => {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        let Some(RetainedPayload::String {
                            value, allocation, ..
                        }) = payloads.values.get(slot)
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        c.require_document_string(allocation, value)?;
                        if value.len() != group.units {
                            return Err(SessionError::WrongToken);
                        }
                    }
                }
                for origin in &data.origins[start..end] {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    c.charge(
                        WorkCategory::InspectByte,
                        mem::size_of::<TextOrigin>() as u64,
                    )?;
                    if origin.start >= origin.end || origin.end > group.units {
                        return Err(SessionError::WrongToken);
                    }
                    source_range(c, data.source, &origin.source)?;
                }
            }
            GroupBody::Alias { original } => {
                if original >= index {
                    return Err(SessionError::WrongToken);
                }
                header(c)?;
                if data.groups[original].units != group.units {
                    return Err(SessionError::WrongToken);
                }
            }
            GroupBody::Concat {
                original,
                incoming,
                offset,
            } => {
                if original >= index || incoming >= index {
                    return Err(SessionError::WrongToken);
                }
                header(c)?;
                let left = data.groups[original].units;
                header(c)?;
                let right = data.groups[incoming].units;
                if left != offset || offset.checked_add(right) != Some(group.units) {
                    return Err(SessionError::WrongToken);
                }
            }
        }
    }
    Ok(())
}

impl FrozenHistory {
    pub(super) fn arena(&self) -> &FrozenArena {
        &self.arena
    }
    pub(super) fn arena_mut(&mut self) -> &mut FrozenArena {
        &mut self.arena
    }
    pub(crate) fn view(&self) -> Result<HistoryView<'_>, SessionError> {
        let storage = self.arena.storage.as_ref().ok_or(SessionError::Consumed)?;
        let evidence = storage.evidence.as_ref().ok_or(SessionError::WrongToken)?;
        Ok(HistoryView {
            storage,
            evidence,
            data: evidence.history_data()?,
        })
    }
    pub(crate) fn associations(&self) -> Result<provenance::AssociationView<'_>, SessionError> {
        self.arena.retained_evidence()?.associations()
    }
    pub(crate) fn fragment(&self) -> Option<&HtmlFragment> {
        self.arena.fragment()
    }
    pub(crate) fn document(&self) -> Option<&HtmlDocument> {
        self.arena.document()
    }
    /// Prepublication disposal, retaining the actual frozen owner on refusal.
    /// A later public report consumes this owner and the execution controller.
    #[cfg(test)]
    pub(crate) fn release(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        self.arena.release(c)
    }
}

/// Exclusive, owner-issued transfer qualification over the exact retained
/// frozen slot. There is no caller-supplied readiness flag or arena identifier.
pub(crate) struct PreparedHistory<'a> {
    input: &'a mut Option<FrozenArena>,
}
impl<'a> PreparedHistory<'a> {
    pub(super) fn share_values<'c>(
        &mut self,
        c: &'c mut ParseControl,
        #[cfg(test)] mode: crate::request::ValueSharingMode,
    ) -> Result<super::value_sharing::Sharing<'c>, SessionError> {
        self.input
            .as_mut()
            .ok_or(SessionError::Consumed)?
            .share_values(
                c,
                #[cfg(test)]
                mode,
            )
    }
    pub(crate) fn prepare(
        input: &'a mut Option<FrozenArena>,
        c: &mut ParseControl,
    ) -> Result<Self, SessionError> {
        let arena = input.as_ref().ok_or(SessionError::Consumed)?;
        let storage = arena.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        storage
            .payloads
            .check_aliases(ConstructionView::Frozen(&storage.view), c)?;
        storage.effects.require_freezable(c)?;
        let evidence = storage.evidence.as_ref().ok_or(SessionError::WrongToken)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        payload_write::<HistoryData<'_>>(c, 1)?;
        let data = evidence.history_data()?;
        // The existing active freeze and this retained frozen transfer share
        // the same source/graph/leaf qualification, not two validators.
        qualify(&storage.payloads, &data, c)?;
        payload_write::<Self>(c, 1)?;
        payload_write::<FrozenHistory>(c, 1)?;
        payload_write::<Option<FrozenArena>>(c, 1)?;
        Ok(Self { input })
    }
    /// All useful work and receiver admissions precede the consuming caller's
    /// final checkpoint/snapshot. Commit only moves the qualified actual slot.
    pub(crate) fn commit(self) -> FrozenHistory {
        FrozenHistory {
            arena: self.input.take().expect("exclusive qualified history slot"),
        }
    }
}

/// A group reference is a borrowed, same-evidence qualification, never a payload
/// slot or a caller assertion about the identity of another report.
#[derive(Clone, Copy)]
pub struct HistoryGroup<'a> {
    evidence: &'a ParseEvidence,
    index: usize,
}
pub(crate) struct HistoryView<'a> {
    storage: &'a FrozenStorage,
    evidence: &'a ParseEvidence,
    data: HistoryData<'a>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoryReadError {
    ForeignOwner,
    InsufficientScratch { required: usize, supplied: usize },
}

pub(crate) struct HistoryHeader<'a> {
    pub(crate) units: usize,
    pub(crate) cause: &'a ConstructionCause,
    pub(crate) retirement: Option<&'a ConstructionCause>,
    pub(crate) body: HistoryBody<'a>,
}
pub enum HistoryBody<'a> {
    Leaf(HistoryLeaf<'a>),
    Alias {
        original: HistoryGroup<'a>,
    },
    Concat {
        original: HistoryGroup<'a>,
        incoming: HistoryGroup<'a>,
        offset: usize,
    },
}
/// Exact retained units and original atoms, with an independently accumulated
/// logical offset. The original range/backing is borrowed, never cloned/rebased.
pub struct HistoryLeaf<'a> {
    units: &'a [u16],
    origins: &'a [TextOrigin],
    offset: usize,
}
pub struct HistoryOrigin<'a> {
    pub(crate) original: &'a TextOrigin,
    pub(crate) start: usize,
    pub(crate) end: usize,
}
impl<'a> HistoryLeaf<'a> {
    pub fn units(&self) -> &'a [u16] {
        self.units
    }
    pub fn unit(&self, index: usize) -> Option<u16> {
        self.units.get(index).copied()
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn origins(&self) -> &'a [TextOrigin] {
        self.origins
    }
    pub fn origin(&self, index: usize) -> Option<HistoryOrigin<'a>> {
        let original = self.origins.get(index)?;
        Some(HistoryOrigin {
            original,
            start: self.offset + original.start,
            end: self.offset + original.end,
        })
    }
}

impl<'a> HistoryView<'a> {
    #[cfg(test)]
    pub(crate) fn source(&self) -> &'a crate::HtmlSource {
        self.data.source
    }
    pub(crate) fn len(&self) -> usize {
        self.data.groups.len()
    }
    pub(crate) fn group(&self, index: usize) -> Option<HistoryGroup<'a>> {
        (index < self.len()).then_some(HistoryGroup {
            evidence: self.evidence,
            index,
        })
    }
    fn require(&self, group: HistoryGroup<'_>) -> Result<usize, HistoryReadError> {
        if !std::ptr::eq(self.evidence, group.evidence) {
            return Err(HistoryReadError::ForeignOwner);
        }
        Ok(group.index)
    }
    fn leaf(&self, group: &OriginGroup, offset: usize) -> HistoryLeaf<'a> {
        let GroupBody::Leaf {
            start,
            length,
            value,
        } = group.body
        else {
            unreachable!("qualified leaf header")
        };
        let units = match value {
            LeafValue::Empty => &[],
            LeafValue::Payload(slot) => {
                let RetainedPayload::String { value, .. } = &self.storage.payloads.values[slot]
                else {
                    unreachable!("qualified immutable backing owner")
                };
                value.code_units()
            }
            LeafValue::Pending => unreachable!("pending leaves cannot freeze history"),
        };
        HistoryLeaf {
            units,
            origins: &self.data.origins[start..start + length],
            offset,
        }
    }
    pub(crate) fn header(
        &self,
        group: HistoryGroup<'_>,
    ) -> Result<HistoryHeader<'a>, HistoryReadError> {
        let index = self.require(group)?;
        let group = &self.data.groups[index];
        let reference = |index| HistoryGroup {
            evidence: self.evidence,
            index,
        };
        let body = match group.body {
            GroupBody::Leaf { .. } => HistoryBody::Leaf(self.leaf(group, 0)),
            GroupBody::Alias { original } => HistoryBody::Alias {
                original: reference(original),
            },
            GroupBody::Concat {
                original,
                incoming,
                offset,
            } => HistoryBody::Concat {
                original: reference(original),
                incoming: reference(incoming),
                offset,
            },
        };
        Ok(HistoryHeader {
            units: group.units,
            cause: &self.data.causes[group.cause],
            retirement: group.retirement.map(|index| &self.data.causes[index]),
            body,
        })
    }
    pub(crate) fn cursor<'s>(
        &'s self,
        group: HistoryGroup<'_>,
        scratch: &'s mut [HistoryFrame],
        cancellation: Option<&'s crate::HtmlParseCancellation>,
    ) -> Result<HistoryCursor<'a, 's>, HistoryReadError> {
        let index = self.require(group)?;
        // Strict backward headers bound the pending DFS stack by index+1 even
        // for a shared DAG. Setup neither scans the graph nor clears scratch.
        let required = index + 1;
        if scratch.len() < required {
            return Err(HistoryReadError::InsufficientScratch {
                required,
                supplied: scratch.len(),
            });
        }
        scratch[0] = HistoryFrame { index, offset: 0 };
        Ok(HistoryCursor {
            view: self,
            scratch,
            length: 1,
            cancellation,
            cancelled: false,
        })
    }
}

/// Private fields prevent caller-written traversal authority. Scratch may hold
/// stale frames: only the cursor's initialized prefix is ever read.
#[derive(Clone, Copy, Default)]
pub struct HistoryFrame {
    index: usize,
    offset: usize,
}
pub struct HistoryCursor<'a, 's> {
    view: &'s HistoryView<'a>,
    scratch: &'s mut [HistoryFrame],
    length: usize,
    cancellation: Option<&'s crate::HtmlParseCancellation>,
    cancelled: bool,
}
pub enum HistoryStep<'a> {
    Progress,
    Leaf(HistoryLeaf<'a>),
    Complete,
    Cancelled,
}
impl<'a> HistoryCursor<'a, '_> {
    pub fn advance(&mut self) -> HistoryStep<'a> {
        if self.cancelled
            || self
                .cancellation
                .is_some_and(|signal| signal.is_cancelled())
        {
            self.cancelled = true;
            return HistoryStep::Cancelled;
        }
        if self.length == 0 {
            return HistoryStep::Complete;
        }
        self.length -= 1;
        let frame = self.scratch[self.length];
        let group = &self.view.data.groups[frame.index];
        match group.body {
            GroupBody::Leaf { .. } => HistoryStep::Leaf(self.view.leaf(group, frame.offset)),
            GroupBody::Alias { original } => {
                self.scratch[self.length] = HistoryFrame {
                    index: original,
                    offset: frame.offset,
                };
                self.length += 1;
                HistoryStep::Progress
            }
            GroupBody::Concat {
                original,
                incoming,
                offset,
            } => {
                self.scratch[self.length] = HistoryFrame {
                    index: incoming,
                    offset: frame.offset + offset,
                };
                self.scratch[self.length + 1] = HistoryFrame {
                    index: original,
                    offset: frame.offset,
                };
                self.length += 2;
                HistoryStep::Progress
            }
        }
    }
}

impl HistoryOrigin<'_> {
    pub fn original(&self) -> &TextOrigin {
        self.original
    }
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
}
impl FrozenHistory {
    pub(super) fn native_node(&self, index: usize) -> HtmlNode<'_> {
        let storage = self.arena.storage.as_ref().expect("qualified history");
        storage
            .view
            .node_at(index)
            .expect("qualified history destination")
            .as_node()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenizer::{states::State, tests::LexerFixture, token::Event};
    use crate::{HtmlParseCancellation, HtmlParseLimits};

    fn limits() -> HtmlParseLimits {
        super::super::tests::limits()
    }
    fn zero(c: &ParseControl) {
        super::super::tests::storage_zero(c);
        assert_eq!(
            c.usage().resource(HtmlParseResource::Diagnostics).current,
            0
        );
    }
    fn literal(
        source: &crate::HtmlSource,
        cancel: HtmlParseCancellation,
    ) -> (ParseControl, ParserArenaSession, usize) {
        let mut c = ParseControl::new(limits(), cancel);
        c.admit_source(source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, source).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(source, State::Data, None, &mut c).unwrap();
        let mut node = None;
        loop {
            let event = lexer
                .next_in(
                    &mut c,
                    false,
                    session.active_mut().unwrap().evidence.as_mut().unwrap(),
                )
                .unwrap()
                .unwrap();
            let Event::Token(event) = event else {
                event.release(&mut c).unwrap();
                continue;
            };
            let token = event.into_token(&mut c).unwrap();
            session
                .active_mut()
                .unwrap()
                .evidence
                .as_mut()
                .unwrap()
                .lexical_disposition(&mut c, token.lexical.as_ref(), false)
                .unwrap();
            match token.kind {
                TokenKind::Text(text) => {
                    node = Some(
                        session
                            .insert_text(&mut c, &root, None, &mut Some(text), None)
                            .unwrap()
                            .unwrap()
                            ._node,
                    );
                }
                TokenKind::Eof => break,
                _ => panic!("independent literal contains text only"),
            }
        }
        lexer.release(&mut c).unwrap();
        let storage = session.active().unwrap();
        let slot = storage
            .field_slot(&mut c, StringLocation::Text(node.unwrap().index))
            .unwrap();
        let RetainedPayload::String {
            origins: Some(group),
            ..
        } = &storage.payloads.values[slot]
        else {
            panic!("canonical native text group")
        };
        // Observe the actual group through the existing controlled lender;
        // inventory assertions below independently describe the literal graph.
        let evidence = storage.evidence.as_ref().unwrap();
        assert!(evidence.require_group(group).unwrap().units > 0);
        // Each actual text merge appends its current group after its incoming
        // leaf. No group is produced by the EOF token.
        let group_index = evidence.history_counts().1 - 1;
        (c, session, group_index)
    }
    fn projected(source: &crate::HtmlSource) -> (ParseControl, ParserArenaSession, usize) {
        let (mut c, mut session, group) = literal(source, HtmlParseCancellation::new());
        session.prepare_native_associations(&mut c, source).unwrap();
        (c, session, group)
    }

    #[test]
    fn immutable_history_borrows_retired_utf16_leaf_units_and_exact_rebased_origins() {
        use crate::parser::tokenizer::token::OriginKind::{CharacterReference, Literal, Newline};
        let source = crate::HtmlSource::utf8("A🙂\r\n&NotEqualTilde;");
        let (mut c, mut session, group) = projected(&source);
        let pointers: Vec<_> = session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .filter_map(|value| match value {
                RetainedPayload::String { value, .. } => Some(value.code_units().as_ptr()),
                _ => None,
            })
            .collect();
        let mut owner = session.freeze_history(&mut c).unwrap();
        let before = c.usage().clone();
        {
            let view = owner.view().unwrap();
            assert_eq!(view.source().identity(), source.identity());
            assert!(owner.document().is_none());
            let text = owner.fragment().unwrap().root().children().next().unwrap();
            let HtmlNodeKind::Text(text) = owner.fragment().unwrap().node(&text).unwrap().kind()
            else {
                panic!("native text")
            };
            assert_eq!(text.code_units(), [65, 0xd83d, 0xde42, 10, 0x2242, 0x0338]);
            let mut scratch = vec![HistoryFrame::default(); view.len()];
            let mut cursor = view
                .cursor(view.group(group).unwrap(), &mut scratch, None)
                .unwrap();
            let mut output = Vec::new();
            let mut origins = Vec::new();
            let mut actions = 0;
            loop {
                match cursor.advance() {
                    HistoryStep::Progress => actions += 1,
                    HistoryStep::Leaf(leaf) => {
                        actions += 1;
                        assert!(
                            pointers.contains(&leaf.units().as_ptr()),
                            "original backing, not copied output"
                        );
                        assert_eq!(leaf.unit(leaf.units().len()), None);
                        output.extend_from_slice(leaf.units());
                        for index in 0..leaf.origins().len() {
                            let origin = leaf.origin(index).unwrap();
                            assert!(std::ptr::eq(origin.original, &leaf.origins()[index]));
                            origins.push((
                                origin.start,
                                origin.end,
                                origin.original.source.start(),
                                origin.original.source.end(),
                                origin.original.kind,
                            ));
                        }
                        assert!(leaf.origin(leaf.origins().len()).is_none());
                    }
                    HistoryStep::Complete => break,
                    HistoryStep::Cancelled => panic!("no query cancellation"),
                }
            }
            assert_eq!(output, [65, 0xd83d, 0xde42, 10, 0x2242, 0x0338]);
            assert_eq!(
                origins,
                [
                    (0, 1, 0, 1, Literal),
                    (1, 3, 1, 5, Literal),
                    (3, 4, 5, 7, Newline),
                    (4, 5, 7, 22, CharacterReference),
                    (5, 6, 7, 22, CharacterReference)
                ]
            );
            // Four Data leaves (the two-unit entity is one text token), and
            // three actual Concats. One action inspects one graph header.
            assert_eq!(actions, 7);
        }
        assert_eq!(
            c.usage(),
            &before,
            "completed reads do no parser work/allocation"
        );
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_requires_real_d1_and_lexical_eof_before_owner_transfer() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, _) = literal(&source, HtmlParseCancellation::new());
        assert!(matches!(
            session.freeze_history(&mut c),
            Err(SessionError::WrongToken)
        ));
        assert!(session.active().is_ok());
        assert!(c.stop().is_none());
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let mut owner = session.freeze_history(&mut c).unwrap();
        assert!(matches!(session.active(), Err(SessionError::Consumed)));
        assert!(owner.associations().unwrap().len() > 0);
        owner.release(&mut c).unwrap();
        zero(&c);
        // A native-only source-free arena cannot certify history either.
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        assert!(matches!(
            session.freeze_history(&mut c),
            Err(SessionError::WrongToken)
        ));
        session.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_foreign_scope_and_small_scratch_leave_caller_storage_untouched() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, group) = projected(&source);
        let (mut foreign_c, mut foreign_session, foreign_group) = projected(&source);
        let mut owner = session.freeze_history(&mut c).unwrap();
        let mut foreign = foreign_session.freeze_history(&mut foreign_c).unwrap();
        {
            let view = owner.view().unwrap();
            let other = foreign.view().unwrap();
            assert_eq!(
                view.header(other.group(foreign_group).unwrap()).err(),
                Some(HistoryReadError::ForeignOwner)
            );
            let mut scratch = [HistoryFrame {
                index: usize::MAX,
                offset: 99,
            }; 2];
            assert_eq!(
                view.cursor(view.group(group).unwrap(), &mut scratch, None)
                    .err(),
                Some(HistoryReadError::InsufficientScratch {
                    required: 3,
                    supplied: 2
                })
            );
            assert!(
                scratch
                    .iter()
                    .all(|frame| frame.index == usize::MAX && frame.offset == 99)
            );
            let mut scratch = [HistoryFrame {
                index: usize::MAX,
                offset: 99,
            }; 5];
            {
                let cursor = view
                    .cursor(view.group(group).unwrap(), &mut scratch, None)
                    .unwrap();
                assert_eq!(cursor.length, 1);
            }
            assert_eq!(scratch[0].index, group);
            assert!(
                scratch[1..]
                    .iter()
                    .all(|frame| frame.index == usize::MAX && frame.offset == 99)
            );
        }
        owner.release(&mut c).unwrap();
        foreign.release(&mut foreign_c).unwrap();
        zero(&c);
        zero(&foreign_c);
    }

    #[test]
    fn immutable_history_old_parse_cancel_does_not_revoke_reads_query_cancel_is_local() {
        let source = crate::HtmlSource::utf8("AB");
        let old_cancel = HtmlParseCancellation::new();
        let (mut c, mut session, group) = literal(&source, old_cancel.clone());
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let mut owner = session.freeze_history(&mut c).unwrap();
        let before = c.usage().clone();
        old_cancel.cancel();
        {
            let view = owner.view().unwrap();
            let mut scratch = [HistoryFrame::default(); 3];
            let query_cancel = HtmlParseCancellation::new();
            let mut cursor = view
                .cursor(
                    view.group(group).unwrap(),
                    &mut scratch,
                    Some(&query_cancel),
                )
                .unwrap();
            assert!(matches!(cursor.advance(), HistoryStep::Progress));
            query_cancel.cancel();
            assert!(matches!(cursor.advance(), HistoryStep::Cancelled));
            assert!(matches!(cursor.advance(), HistoryStep::Cancelled));
            let mut cursor = view
                .cursor(view.group(group).unwrap(), &mut scratch, None)
                .unwrap();
            assert!(matches!(cursor.advance(), HistoryStep::Progress));
            assert!(
                matches!(cursor.advance(), HistoryStep::Leaf(leaf) if leaf.units()==[65] && leaf.offset()==0)
            );
            assert!(
                matches!(cursor.advance(), HistoryStep::Leaf(leaf) if leaf.units()==[66] && leaf.offset()==1)
            );
            assert!(matches!(cursor.advance(), HistoryStep::Complete));
        }
        assert_eq!(c.usage(), &before);
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_alias_empty_and_shared_dag_advance_one_header_per_action() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, original_index) = literal(&source, HtmlParseCancellation::new());
        let storage = session.active_mut().unwrap();
        let evidence = storage.evidence.as_mut().unwrap();
        let slot = storage
            .payloads
            .values
            .iter()
            .find_map(|payload| match payload {
                RetainedPayload::String {
                    origins: Some(reference),
                    location: StringLocation::Text(_),
                    ..
                } => Some(reference),
                _ => None,
            })
            .unwrap();
        let mut reference = evidence.copy_group_reference(&mut c, slot).unwrap();
        // Actual canonical DAG: six doublings share each prior header as both
        // children. Four Copy headers then alias the resulting 128 units.
        let mut units = 2;
        for _ in 0..6 {
            reference = evidence
                .merge_groups(&mut c, &reference, &reference, units, units * 2)
                .unwrap();
            units *= 2;
        }
        for _ in 0..4 {
            reference = evidence.copy_group(&mut c, &reference).unwrap();
        }
        let final_index = evidence.history_counts().1 - 1;
        let empty_index = evidence.history_counts().1;
        evidence.source_independent_group(&mut c, 0).unwrap();
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let mut owner = session.freeze_history(&mut c).unwrap();
        let before = c.usage().clone();
        {
            let view = owner.view().unwrap();
            let header = view.header(view.group(final_index).unwrap()).unwrap();
            assert_eq!(header.units, 128);
            assert_eq!(
                header.cause.operation,
                provenance::ConstructionOperation::Copy
            );
            assert!(header.retirement.is_none());
            let HistoryBody::Alias { original } = header.body else {
                panic!("copy alias")
            };
            assert_eq!(view.header(original).unwrap().units, 128);
            let header = view.header(view.group(original_index).unwrap()).unwrap();
            let HistoryBody::Concat {
                original,
                incoming,
                offset,
            } = header.body
            else {
                panic!("real AB merge")
            };
            assert_eq!(offset, 1);
            for (group, expected) in [(original, 65), (incoming, 66)] {
                let header = view.header(group).unwrap();
                let HistoryBody::Leaf(leaf) = header.body else {
                    panic!("literal leaf")
                };
                assert_eq!(leaf.units(), [expected]);
                // A previously occupied the native Text slot and is retired;
                // incoming B was registered Detached and never occupied it.
                assert_eq!(header.retirement.is_some(), expected == 65);
            }
            let header = view.header(view.group(empty_index).unwrap()).unwrap();
            assert_eq!(
                header.cause.operation,
                provenance::ConstructionOperation::SourceIndependent
            );
            let HistoryBody::Leaf(empty) = header.body else {
                panic!("zero-unit defining leaf")
            };
            assert!(empty.units().is_empty() && empty.origins().is_empty());
            let mut scratch = vec![HistoryFrame::default(); view.len()];
            let mut cursor = view
                .cursor(view.group(final_index).unwrap(), &mut scratch, None)
                .unwrap();
            let mut progress = 0;
            let mut leaves = 0;
            loop {
                match cursor.advance() {
                    HistoryStep::Progress => progress += 1,
                    HistoryStep::Leaf(leaf) => {
                        assert_eq!(leaf.offset(), leaves);
                        assert_eq!(leaf.units(), [if leaves % 2 == 0 { 65 } else { 66 }]);
                        let origin = leaf.origin(0).unwrap();
                        assert_eq!((origin.start, origin.end), (leaves, leaves + 1));
                        assert_eq!(origin.original.source.start(), leaves % 2);
                        leaves += 1;
                    }
                    HistoryStep::Complete => break,
                    HistoryStep::Cancelled => panic!("no query cancellation"),
                }
            }
            // 128 leaves have 127 binary headers, plus exactly four aliases.
            assert_eq!((leaves, progress), (128, 131));
        }
        assert_eq!(c.usage(), &before);
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_qualification_exact_work_and_one_short_keep_actual_cleanup_owner() {
        let source = crate::HtmlSource::utf8("AB");
        for short in [false, true] {
            let (mut c, mut session, _) = projected(&source);
            let storage = session.active().unwrap();
            let data = storage.evidence.as_ref().unwrap().history_data().unwrap();
            assert_eq!((data.groups.len(), data.origins.len()), (3, 2));
            let causes = data.causes.len() as u64;
            let ranged_causes = data
                .causes
                .iter()
                .filter(|cause| cause.source.is_some())
                .count() as u64;
            // AB: three headers + two Concat child-header reads, two leaf slot
            // visits + two require_document_string visits, two origin reads.
            // Each source range has its own admitted header inspection and the
            // canonical range check's RecordVisit+ProviderCall.
            let ranges = ranged_causes + 2;
            let visits = causes + 5 + 4 + 2 + ranges;
            let inspect = causes * mem::size_of::<ConstructionCause>() as u64
                + 5 * mem::size_of::<OriginGroup>() as u64
                + 2 * mem::size_of::<TextOrigin>() as u64
                + ranges * mem::size_of::<crate::HtmlSourceRange>() as u64;
            let exact = visits + inspect + ranges;
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                limits().work_credits() - used - exact + u64::from(short),
            )
            .unwrap();
            let before = c.usage().clone();
            let outcome = qualify(&storage.payloads, &data, &mut c);
            if short {
                let first = c.stop().unwrap();
                assert!(matches!(
                    first,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }
                ));
                assert_eq!(outcome, Err(SessionError::Stop(first)));
                assert!(session.active().is_ok());
                assert!(
                    matches!(session.freeze_history(&mut c), Err(SessionError::Stop(stop)) if stop == first)
                );
                let at_stop = c.first_stop_usage().cloned();
                session.release(&mut c).unwrap();
                assert_eq!(c.stop(), Some(first));
                assert_eq!(c.first_stop_usage(), at_stop.as_ref());
            } else {
                outcome.unwrap();
                assert_eq!(
                    c.usage().work(WorkCategory::RecordVisit)
                        - before.work(WorkCategory::RecordVisit),
                    visits
                );
                assert_eq!(
                    c.usage().work(WorkCategory::InspectByte)
                        - before.work(WorkCategory::InspectByte),
                    inspect
                );
                assert_eq!(
                    c.usage().work(WorkCategory::ProviderCall)
                        - before.work(WorkCategory::ProviderCall),
                    ranges
                );
                assert_eq!(
                    c.usage().work(WorkCategory::CopyByte),
                    before.work(WorkCategory::CopyByte)
                );
                assert_eq!(
                    c.usage().work(WorkCategory::RecordWrite),
                    before.work(WorkCategory::RecordWrite)
                );
                // Cleanup may newly latch exhaustion; the borrowed active owner
                // is retained for the ordinary terminal disposal retry.
                if session.release(&mut c).is_err() {
                    session.release(&mut c).unwrap();
                }
            }
            zero(&c);
        }
    }

    #[test]
    fn immutable_history_alias_refusal_cancel_and_foreign_control_preserve_transfer_and_retry() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, _) = projected(&source);
        let alias = (0..session.active().unwrap().construction.test_storage().len())
            .find_map(|position| {
                match session
                    .active()
                    .unwrap()
                    .construction
                    .node_at(position)
                    .unwrap()
                    .kind()
                {
                    HtmlNodeKind::Text(value) => Some(value.clone()),
                    _ => None,
                }
            })
            .unwrap();
        assert!(matches!(
            session.freeze_history(&mut c),
            Err(SessionError::AliasedPayload)
        ));
        assert!(session.active().is_ok() && c.stop().is_none());
        drop(alias);
        let mut owner = session.freeze_history(&mut c).unwrap();
        let alias = owner.fragment().unwrap().clone();
        assert!(owner.release(&mut c).is_err());
        assert!(owner.view().is_ok());
        drop(alias);
        owner.release(&mut c).unwrap();
        zero(&c);

        let signal = HtmlParseCancellation::new();
        let (mut c, mut session, _) = literal(&source, signal.clone());
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        signal.cancel();
        assert!(matches!(
            session.freeze_history(&mut c),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        assert!(session.active().is_ok());
        session.release(&mut c).unwrap();
        zero(&c);

        let (mut c, mut session, _) = projected(&source);
        let mut foreign = ParseControl::new(limits(), HtmlParseCancellation::new());
        foreign.admit_source(&source).unwrap();
        assert!(matches!(
            session.freeze_history(&mut foreign),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            )))
        ));
        assert!(session.active().is_ok() && c.stop().is_none());
        session.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_reset_keeps_retired_raw_graph_without_duplicate_backing() {
        let source = crate::HtmlSource::utf8("<textarea>AB");
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let Event::Token(event) = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap()
        else {
            panic!("textarea")
        };
        let token = event.into_token(&mut c).unwrap();
        session
            .active_mut()
            .unwrap()
            .evidence
            .as_mut()
            .unwrap()
            .lexical_disposition(&mut c, token.lexical.as_ref(), false)
            .unwrap();
        let TokenKind::Tag(tag) = token.kind else {
            panic!("textarea")
        };
        let node = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(tag),
                &root,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        session.insert(&mut c, &root, &node, None, None).unwrap();
        loop {
            let Event::Token(event) = lexer
                .next_in(
                    &mut c,
                    false,
                    session.active_mut().unwrap().evidence.as_mut().unwrap(),
                )
                .unwrap()
                .unwrap()
            else {
                continue;
            };
            let token = event.into_token(&mut c).unwrap();
            session
                .active_mut()
                .unwrap()
                .evidence
                .as_mut()
                .unwrap()
                .lexical_disposition(&mut c, token.lexical.as_ref(), false)
                .unwrap();
            match token.kind {
                TokenKind::Text(text) => {
                    session
                        .insert_text(&mut c, &node, None, &mut Some(text), None)
                        .unwrap();
                }
                TokenKind::Eof => break,
                _ => panic!("literal AB then EOF"),
            }
        }
        lexer.release(&mut c).unwrap();
        let raw = session.textarea_state(&mut c, &node).unwrap().raw;
        assert_eq!(
            session
                .textarea_raw_value(&mut c, &node)
                .unwrap()
                .code_units(),
            [65, 66]
        );
        let old_group = {
            let storage = session.active_mut().unwrap();
            let RetainedPayload::String {
                origins: Some(group),
                ..
            } = &storage.payloads.values[raw]
            else {
                panic!("canonical raw group")
            };
            let evidence = storage.evidence.as_mut().unwrap();
            // Preserve this actual current raw group through the canonical
            // Copy/Alias producer; no pre-projection immutable lender is used.
            let index = evidence.history_counts().1;
            let copy = evidence.copy_group(&mut c, group).unwrap();
            drop(copy);
            index
        };
        for _ in 0..3 {
            session.parser_stack_removed(&mut c, &node, None).unwrap();
        }
        assert!(matches!(
            session.active().unwrap().payloads.values[raw],
            RetainedPayload::HistoricalString { .. }
        ));
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let mut owner = session.freeze_history(&mut c).unwrap();
        {
            let view = owner.view().unwrap();
            let group = view.group(old_group).unwrap();
            assert_eq!(view.header(group).unwrap().units, 2);
            let mut scratch = vec![HistoryFrame::default(); old_group + 1];
            let mut cursor = view.cursor(group, &mut scratch, None).unwrap();
            let mut units = Vec::new();
            loop {
                match cursor.advance() {
                    HistoryStep::Leaf(leaf) => {
                        assert_eq!(leaf.offset(), units.len());
                        for origin in leaf.origins() {
                            assert!(origin.source.start() >= 10 && origin.source.end() <= 12);
                        }
                        units.extend_from_slice(leaf.units());
                    }
                    HistoryStep::Progress => {}
                    HistoryStep::Complete => break,
                    HistoryStep::Cancelled => panic!("no query cancellation"),
                }
            }
            assert_eq!(units, [65, 66]);
        }
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_formatting_replay_lends_distinct_backings_and_original_entity_atoms() {
        use crate::parser::tokenizer::token::OriginKind::{CharacterReference, Literal};
        let source = crate::HtmlSource::utf8("<b a='X&amp;Y'>");
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let root = session.root(&mut c).unwrap();
        let html = session.allocate_implicit_html(&mut c).unwrap();
        session.insert(&mut c, &root, &html, None, None).unwrap();
        let body = session.allocate_implicit_body(&mut c, None).unwrap();
        session.insert(&mut c, &html, &body, None, None).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let Event::Token(event) = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap()
        else {
            panic!("start tag")
        };
        let token = event.into_token(&mut c).unwrap();
        session
            .active_mut()
            .unwrap()
            .evidence
            .as_mut()
            .unwrap()
            .lexical_disposition(&mut c, token.lexical.as_ref(), false)
            .unwrap();
        let TokenKind::Tag(tag) = token.kind else {
            panic!("start tag")
        };
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut Some(tag), token.source)
            .unwrap();
        let first = session
            .create_formatting_from_creation(&mut c, &creation, &body)
            .unwrap();
        let second = session
            .create_formatting_from_creation(&mut c, &creation, &body)
            .unwrap();
        assert!(!session.same_node(&mut c, &first, &second).unwrap());
        session.insert(&mut c, &body, &first, None, None).unwrap();
        session.insert(&mut c, &body, &second, None, None).unwrap();
        creation.release(&mut c).unwrap();
        let event = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap();
        assert!(
            matches!(&event, Event::Token(token) if matches!(&token.token_for_tests().kind, TokenKind::Eof))
        );
        event.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let mut owner = session.freeze_history(&mut c).unwrap();
        {
            let view = owner.view().unwrap();
            let mut pointers = Vec::new();
            for index in 0..view.len() {
                let header = view.header(view.group(index).unwrap()).unwrap();
                let HistoryBody::Leaf(leaf) = header.body else {
                    continue;
                };
                if leaf.units() != [88, 38, 89] {
                    continue;
                }
                let atoms = leaf
                    .origins()
                    .iter()
                    .map(|atom| {
                        (
                            atom.start,
                            atom.end,
                            atom.source.start(),
                            atom.source.end(),
                            atom.kind,
                        )
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    atoms,
                    [
                        (0, 1, 6, 7, Literal),
                        (1, 2, 7, 12, CharacterReference),
                        (2, 3, 12, 13, Literal)
                    ]
                );
                pointers.push(leaf.units().as_ptr());
            }
            assert_eq!(pointers.len(), 2, "two actual replay field leaves");
            assert_ne!(
                pointers[0], pointers[1],
                "actual independently owned replay backings"
            );
        }
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_native_only_projection_without_lexical_eof_cannot_transfer() {
        let source = crate::HtmlSource::utf8("A");
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let Event::Token(event) = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap()
        else {
            panic!("literal A")
        };
        let token = event.into_token(&mut c).unwrap();
        session
            .active_mut()
            .unwrap()
            .evidence
            .as_mut()
            .unwrap()
            .lexical_disposition(&mut c, token.lexical.as_ref(), false)
            .unwrap();
        let TokenKind::Text(text) = token.kind else {
            panic!("literal Text")
        };
        session
            .insert_text(&mut c, &root, None, &mut Some(text), None)
            .unwrap();
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        assert!(session.native_associations(&mut c).unwrap().len() > 0);
        assert!(matches!(
            session.freeze_history(&mut c),
            Err(SessionError::WrongToken)
        ));
        assert!(session.active().is_ok() && c.stop().is_none());
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_source_independent_nonempty_units_are_not_lexical_claims() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, _) = literal(&source, HtmlParseCancellation::new());
        // Use the actual source-independent producer. An implied element after
        // lexical input may instead retain the current governing token cause;
        // lack of field origins alone does not make that cause source-less.
        for literal in [b"p".as_slice(), b"http://www.w3.org/1999/xhtml".as_slice()] {
            let value = literal_string(&mut c, literal).unwrap();
            let storage = session.active_mut().unwrap();
            let group = storage
                .evidence
                .as_mut()
                .unwrap()
                .source_independent_group(&mut c, literal.len())
                .unwrap();
            let (value, _) = storage
                .register_group_string_owned(&mut c, value, Some(group))
                .unwrap();
            drop(value);
        }
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let mut owner = session.freeze_history(&mut c).unwrap();
        {
            let view = owner.view().unwrap();
            let mut saw_name = false;
            let mut saw_namespace = false;
            for index in 0..view.len() {
                let header = view.header(view.group(index).unwrap()).unwrap();
                let HistoryBody::Leaf(leaf) = header.body else {
                    continue;
                };
                if header.cause.operation == provenance::ConstructionOperation::SourceIndependent {
                    assert!(header.cause.source.is_none() && header.cause.captured.is_none());
                    assert!(leaf.origins().is_empty());
                    if leaf.units() == [112] {
                        saw_name = true;
                    }
                    if leaf.units()
                        == "http://www.w3.org/1999/xhtml"
                            .encode_utf16()
                            .collect::<Vec<_>>()
                    {
                        saw_namespace = true;
                    }
                }
            }
            assert!(
                saw_name && saw_namespace,
                "actual independently retained source-less literals"
            );
        }
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_captured_namespace_keeps_context_qualification_without_lexical_origins() {
        use crate::*;
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(10, 10, 10, 1000, 10, 10, 10, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::from_uri("urn:retained"), None, "context"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let request =
            HtmlFragmentParseRequest::new(HtmlSource::utf8(""), context, limits()).unwrap();
        let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_standalone_document(&mut c).unwrap();
        session
            .initialize_parse_evidence(&mut c, request.source())
            .unwrap();
        session.initialize_fragment(&mut c, &request).unwrap();
        let parser_document = session.root(&mut c).unwrap();
        session
            .validate_fragment_proof(&mut c, &parser_document)
            .unwrap();
        // Exercise the existing actual source-owner copy path only after real
        // request/bootstrap/profile qualification; no captured trigger is minted
        // by this reader or test and no synthetic lexical range is supplied.
        let before = session
            .active()
            .unwrap()
            .evidence
            .as_ref()
            .unwrap()
            .history_counts()
            .1;
        let (namespace, slot) = session
            .active_mut()
            .unwrap()
            .copy_context_namespace(&mut c)
            .unwrap();
        assert_eq!(
            namespace.uri().unwrap().code_units(),
            "urn:retained".encode_utf16().collect::<Vec<_>>()
        );
        let slot = slot.unwrap();
        let pointer = namespace.uri().unwrap().code_units().as_ptr();
        drop(namespace);
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let event = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap();
        assert!(
            matches!(&event, Event::Token(token) if matches!(token.token_for_tests().kind, TokenKind::Eof))
        );
        event.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session
            .prepare_native_associations(&mut c, request.source())
            .unwrap();
        let mut owner = session.freeze_history(&mut c).unwrap();
        {
            let view = owner.view().unwrap();
            let header = view.header(view.group(before).unwrap()).unwrap();
            assert!(header.cause.source.is_none());
            let captured = header.cause.captured.as_ref().unwrap();
            assert_eq!(&captured.capture, request.context().capture().identity());
            assert_eq!(
                &captured.host_revision,
                request.context().capture().host_facts_revision()
            );
            assert_eq!(&captured.context, request.context().context_element());
            assert_eq!(
                &captured.target_document,
                request.context().target_document()
            );
            let HistoryBody::Leaf(leaf) = header.body else {
                panic!("captured namespace leaf")
            };
            assert_eq!(leaf.units().as_ptr(), pointer);
            assert_eq!(
                leaf.units(),
                "urn:retained".encode_utf16().collect::<Vec<_>>()
            );
            assert!(leaf.origins().is_empty());
            assert!(matches!(
                &view.storage.payloads.values[slot],
                RetainedPayload::String {
                    location: StringLocation::Detached,
                    ..
                }
            ));
        }
        owner.release(&mut c).unwrap();
        zero(&c);
    }
}
