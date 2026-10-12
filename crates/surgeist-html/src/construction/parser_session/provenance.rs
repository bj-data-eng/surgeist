//! Native field facts owned by the source evidence issuer, not another tree.

use super::*;
use crate::parser::{control::Invariant, tokenizer::token::OriginKind};

/// Local indices never leave their evidence identity through an unchecked API.
pub(crate) struct OriginGroupRef {
    index: usize,
    issuer: Weak<evidence::EvidenceIdentity>,
    control: ControlIdentity,
}
pub(crate) struct ConstructionCauseRef {
    index: usize,
    issuer: Weak<evidence::EvidenceIdentity>,
    control: ControlIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParserRule {
    Prologue(crate::parser::tree_builder::prologue::PrologueMode),
    Tree(crate::parser::tree_builder::head::InsertionMode),
    Foreign,
}
pub(crate) struct ParserTrigger<'a> {
    pub(crate) source: &'a crate::HtmlSourceRange,
    pub(crate) rule: ParserRule,
    pub(crate) family: evidence::DiagnosticToken,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConstructionOperation {
    SourceIndependent,
    Request,
    Token,
    Merge,
    Copy,
    Replacement,
    Retirement,
    Replay,
    Captured,
}
pub(crate) struct ConstructionCause {
    pub(crate) operation: ConstructionOperation,
    pub(crate) source: Option<crate::HtmlSourceRange>,
    pub(crate) captured: Option<CapturedConstructionContext>,
    pub(crate) rule: Option<ParserRule>,
    pub(crate) family: Option<evidence::DiagnosticToken>,
    pub(crate) original_cause: Option<usize>,
    pub(crate) retired_destination: Option<StringLocation>,
    pub(crate) original_node: Option<usize>,
    pub(crate) intended_parent: Option<usize>,
    pub(crate) original: Option<usize>,
    pub(crate) incoming: Option<usize>,
    pub(crate) trigger: Option<usize>,
}
pub(crate) struct OriginGroup {
    pub(crate) start: usize,
    pub(crate) length: usize,
    pub(crate) units: usize,
    pub(crate) cause: usize,
    pub(crate) retirement: Option<usize>,
}
#[derive(Clone, Copy)]
pub(crate) enum NativeDestination {
    Field(StringLocation),
    Node(usize),
    Raw { node: usize, kind: RawFieldKind },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RawFieldKind {
    HiddenInput,
    Textarea,
}
pub(crate) struct NativeAssociation {
    pub(crate) destination: NativeDestination,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) source: Option<crate::HtmlSourceRange>,
    pub(crate) kind: Option<OriginKind>,
    cause: usize,
}
/// Mutable production and immutable groups have one flat owner. Pending suffixes
/// have no group/projection proof and are reclaimed on the original stop.
pub(super) struct NativeProvenance {
    source: Option<crate::HtmlSource>,
    origins: OwnedVec<TextOrigin>,
    groups: OwnedVec<OriginGroup>,
    causes: OwnedVec<ConstructionCause>,
    current: Option<usize>,
    projection: Option<OwnedVec<NativeAssociation>>,
    projection_complete: bool,
}
impl NativeProvenance {
    pub(super) fn new(c: &mut ParseControl) -> Result<Self, ParseStop> {
        let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let mut groups = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let mut causes = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        origins.require_owner(c)?;
        groups.require_owner(c)?;
        causes.require_owner(c)?;
        Ok(Self {
            source: None,
            origins,
            groups,
            causes,
            current: None,
            projection: None,
            projection_complete: false,
        })
    }
    /// Fixed-header cleanup proof. All four vectors are flat SourceMapEntries
    /// owners; neither elements nor source handles own separately charged storage.
    /// No vector is moved until foreign ownership, representation and aggregate
    /// ledger underflow have been ruled out under the actual stopped controller.
    pub(super) fn preflight_failed_disposal(&self, c: &mut ParseControl) -> Result<(), ParseStop> {
        fn extent<T>(values: &OwnedVec<T>, c: &mut ParseControl) -> Result<(u64, u64), ParseStop> {
            values.require_disposal_owner(c)?;
            let Some(bytes) = values
                .capacity()
                .checked_mul(mem::size_of::<T>())
                .and_then(|bytes| u64::try_from(bytes).ok())
            else {
                return Err(c.fail_invariant(Invariant::ReleaseUnderflow));
            };
            let Ok(units) = u64::try_from(values.len()) else {
                return Err(c.fail_invariant(Invariant::ReleaseUnderflow));
            };
            Ok((units, bytes))
        }
        let mut total = (0u64, 0u64);
        for (units, bytes) in [
            extent(&self.origins, c)?,
            extent(&self.groups, c)?,
            extent(&self.causes, c)?,
            match &self.projection {
                Some(rows) => extent(rows, c)?,
                None => (0, 0),
            },
        ] {
            let (Some(units), Some(bytes)) =
                (total.0.checked_add(units), total.1.checked_add(bytes))
            else {
                return Err(c.fail_invariant(Invariant::ReleaseUnderflow));
            };
            total = (units, bytes);
        }
        if total.0
            > c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current
            || total.1
                > c.usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .current
        {
            return Err(c.fail_invariant(Invariant::ReleaseUnderflow));
        }
        Ok(())
    }
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        if let Some(projection) = self.projection {
            projection.release(c)?;
        }
        self.origins.release(c)?;
        self.groups.release(c)?;
        self.causes.release(c)
    }
}
impl ParseEvidence {
    fn validate_field(
        &self,
        c: &mut ParseControl,
        value: &[u16],
        group: &OriginGroupRef,
    ) -> Result<(), SessionError> {
        group.require_disposal_owner(c)?;
        let Some(source) = &self.native.source else {
            return Ok(());
        };
        for origin in self.group_origins(group)? {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.require_source_range(&origin.source)?;
            let output = value
                .get(origin.start..origin.end)
                .ok_or(SessionError::WrongToken)?;
            let begin = origin.source.start();
            let end = origin.source.end();
            if source.identity() != origin.source.source() {
                return Err(SessionError::WrongToken);
            }
            match origin.kind {
                OriginKind::Literal | OriginKind::AsciiLowercase => {
                    c.charge(
                        WorkCategory::InspectByte,
                        (end - begin) as u64 * if source.as_utf8().is_some() { 1 } else { 2 },
                    )?;
                    let mut offset = 0;
                    let mut compare = |unit: u16| -> Result<(), SessionError> {
                        c.charge(WorkCategory::InspectByte, 2)?;
                        let expected = if origin.kind == OriginKind::AsciiLowercase
                            && (65..=90).contains(&unit)
                        {
                            unit + 32
                        } else {
                            unit
                        };
                        if output.get(offset) != Some(&expected) {
                            return Err(SessionError::WrongToken);
                        }
                        offset += 1;
                        Ok(())
                    };
                    if let Some(text) = source.as_utf8() {
                        let input = text.get(begin..end).ok_or(SessionError::WrongToken)?;
                        // Each scalar decoder step is bounded by four source
                        // bytes and two output units. No decoded copy exists.
                        for character in input.chars() {
                            let mut pair = [0; 2];
                            for &unit in character.encode_utf16(&mut pair).iter() {
                                compare(unit)?;
                            }
                        }
                    } else {
                        let input = source
                            .as_utf16()
                            .ok_or(SessionError::WrongToken)?
                            .code_units()
                            .get(begin..end)
                            .ok_or(SessionError::WrongToken)?;
                        for &unit in input {
                            compare(unit)?;
                        }
                    }
                    if offset != output.len() {
                        return Err(SessionError::WrongToken);
                    }
                }
                OriginKind::Newline => {
                    c.charge(WorkCategory::InspectByte, 6)?;
                    let correct = if let Some(text) = source.as_utf8() {
                        matches!(text.get(begin..end), Some("\r" | "\n" | "\r\n"))
                    } else {
                        matches!(
                            source
                                .as_utf16()
                                .and_then(|text| text.code_units().get(begin..end)),
                            Some([13] | [10] | [13, 10])
                        )
                    };
                    if !correct || output != [10] {
                        return Err(SessionError::WrongToken);
                    }
                }
                OriginKind::NullReplacement => {
                    c.charge(WorkCategory::InspectByte, 4)?;
                    let correct = if let Some(text) = source.as_utf8() {
                        text.get(begin..end) == Some("\0")
                    } else {
                        source
                            .as_utf16()
                            .and_then(|text| text.code_units().get(begin..end))
                            == Some(&[0])
                    };
                    if !correct || output != [0xfffd] {
                        return Err(SessionError::WrongToken);
                    }
                }
                // These are producer transformations, not raw equality. The
                // actual token/copy operation preserves its atomic output;
                // validation must not invent an entity/XML resolver here.
                OriginKind::CharacterReference | OriginKind::RecoverySyntax => {
                    c.charge(WorkCategory::InspectByte, output.len() as u64 * 2)?;
                    if output.is_empty() || begin == end {
                        return Err(SessionError::WrongToken);
                    }
                }
            }
        }
        Ok(())
    }
    fn group_ref(
        &self,
        index: usize,
        c: &mut ParseControl,
    ) -> Result<OriginGroupRef, SessionError> {
        payload_write::<OriginGroupRef>(c, 1)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        Ok(OriginGroupRef {
            index,
            issuer: Arc::downgrade(&self.identity),
            control: c.identity(),
        })
    }
    /// Copies an already-issued cause handle within this exact issuer. No new
    /// cause, event or association is created by this checked fixed-size copy.
    pub(crate) fn copy_cause_ref(
        &self,
        c: &mut ParseControl,
        reference: &ConstructionCauseRef,
    ) -> Result<ConstructionCauseRef, SessionError> {
        c.require_identity(&reference.control)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<Weak<evidence::EvidenceIdentity>>() as u64,
        )?;
        if reference.issuer.as_ptr() != Arc::as_ptr(&self.identity)
            || self.native.causes.as_slice().get(reference.index).is_none()
        {
            return Err(SessionError::WrongToken);
        }
        self.cause_ref(reference.index, c)
    }

    fn cause_ref(
        &self,
        index: usize,
        c: &mut ParseControl,
    ) -> Result<ConstructionCauseRef, SessionError> {
        payload_write::<ConstructionCauseRef>(c, 1)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        Ok(ConstructionCauseRef {
            index,
            issuer: Arc::downgrade(&self.identity),
            control: c.identity(),
        })
    }
    fn require_group(&self, group: &OriginGroupRef) -> Result<&OriginGroup, SessionError> {
        if group.issuer.as_ptr() != Arc::as_ptr(&self.identity) {
            return Err(SessionError::WrongToken);
        }
        self.native
            .groups
            .as_slice()
            .get(group.index)
            .ok_or(SessionError::WrongToken)
    }
    pub(super) fn group_origins(
        &self,
        group: &OriginGroupRef,
    ) -> Result<&[TextOrigin], SessionError> {
        let group = self.require_group(group)?;
        self.native
            .origins
            .as_slice()
            .get(group.start..group.start + group.length)
            .ok_or(SessionError::WrongToken)
    }
    pub(super) fn copy_group(
        &mut self,
        c: &mut ParseControl,
        original: &OriginGroupRef,
    ) -> Result<OriginGroupRef, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let group = self.require_group(original)?;
        let (start, length, units, original) =
            (group.start, group.length, group.units, original.index);
        let cause = self.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Copy,
                source: None,
                captured: None,
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: None,
                original_node: None,
                intended_parent: None,
                original: Some(original),
                incoming: None,
                trigger: self.native.current,
            },
        )?;
        let index = self.native.groups.len();
        let reference = self.group_ref(index, c)?;
        self.native.groups.try_push(
            c,
            OriginGroup {
                start,
                length,
                units,
                cause,
                retirement: None,
            },
        )?;
        Ok(reference)
    }
    fn append_cause(
        &mut self,
        c: &mut ParseControl,
        cause: ConstructionCause,
    ) -> Result<usize, SessionError> {
        let index = self.native.causes.len();
        self.native.causes.try_push(c, cause)?;
        Ok(index)
    }
    pub(super) fn initialize_native_source(
        &mut self,
        c: &mut ParseControl,
        source: &crate::HtmlSource,
    ) -> Result<(), SessionError> {
        c.require_source(source)?;
        if self.native.source.is_some() {
            return Err(SessionError::WrongToken);
        }
        payload_write::<Option<crate::HtmlSource>>(c, 1)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        let cause = self.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Request,
                source: None,
                captured: None,
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: None,
                original_node: None,
                intended_parent: None,
                original: None,
                incoming: None,
                trigger: None,
            },
        )?;
        payload_write::<Option<usize>>(c, 1)?;
        self.native.source = Some(source.clone());
        self.native.current = Some(cause);
        Ok(())
    }
    pub(super) fn parser_trigger(
        &mut self,
        c: &mut ParseControl,
        trigger: ParserTrigger<'_>,
    ) -> Result<ConstructionCauseRef, SessionError> {
        c.require_source_range(trigger.source)?;
        if self
            .native
            .source
            .as_ref()
            .is_none_or(|source| source.identity() != trigger.source.source())
        {
            return Err(SessionError::WrongToken);
        }
        payload_write::<crate::HtmlSourceRange>(c, 1)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        let index = self.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Token,
                source: Some(trigger.source.clone()),
                captured: None,
                rule: Some(trigger.rule),
                family: Some(trigger.family),
                original_cause: None,
                retired_destination: None,
                original_node: None,
                intended_parent: None,
                original: None,
                incoming: None,
                trigger: self.native.current,
            },
        )?;
        let result = self.cause_ref(index, c)?;
        payload_write::<Option<usize>>(c, 1)?;
        self.native.current = Some(index);
        Ok(result)
    }
    fn field_cause(&mut self, c: &mut ParseControl) -> Result<usize, SessionError> {
        if let Some(cause) = self.native.current {
            return Ok(cause);
        }
        self.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::SourceIndependent,
                source: None,
                captured: None,
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: None,
                original_node: None,
                intended_parent: None,
                original: None,
                incoming: None,
                trigger: None,
            },
        )
    }
    pub(super) fn retain_group(
        &mut self,
        c: &mut ParseControl,
        mut input: OwnedVec<TextOrigin>,
        units: usize,
    ) -> Result<OriginGroupRef, SessionError> {
        let result = self.retain_group_parts(c, &mut input, units, true, None);
        input.release(c)?;
        result
    }
    fn retain_group_parts(
        &mut self,
        c: &mut ParseControl,
        input: &mut OwnedVec<TextOrigin>,
        units: usize,
        contiguous: bool,
        cause: Option<usize>,
    ) -> Result<OriginGroupRef, SessionError> {
        input.require_owner(c)?;
        let mut end = 0;
        for origin in input.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if origin.start < end
                || (contiguous && origin.start != end)
                || origin.end <= origin.start
                || origin.end > units
            {
                return Err(SessionError::WrongToken);
            }
            if self.native.source.is_some() {
                c.require_source_range(&origin.source)?;
            }
            end = origin.end;
        }
        if contiguous && !input.as_slice().is_empty() && end != units {
            return Err(SessionError::WrongToken);
        }
        let cause = match cause {
            Some(cause) => cause,
            None => self.field_cause(c)?,
        };
        let start = self.native.origins.len();
        let length = input.len();
        while let Some(origin) = input.try_pop(c)? {
            self.native.origins.try_push(c, origin)?;
        }
        for position in 0..length / 2 {
            self.native
                .origins
                .swap_slots(start + position, start + length - position - 1, c)?;
        }
        let index = self.native.groups.len();
        let reference = self.group_ref(index, c)?;
        self.native.groups.try_push(
            c,
            OriginGroup {
                start,
                length,
                units,
                cause,
                retirement: None,
            },
        )?;
        Ok(reference)
    }
    pub(super) fn rule_group(
        &mut self,
        c: &mut ParseControl,
        units: usize,
    ) -> Result<OriginGroupRef, SessionError> {
        let origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        self.retain_group(c, origins, units)
    }
    pub(super) fn source_independent_group(
        &mut self,
        c: &mut ParseControl,
        units: usize,
    ) -> Result<OriginGroupRef, SessionError> {
        let cause = self.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::SourceIndependent,
                source: None,
                captured: None,
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: None,
                original_node: None,
                intended_parent: None,
                original: None,
                incoming: None,
                trigger: None,
            },
        )?;
        let index = self.native.groups.len();
        let reference = self.group_ref(index, c)?;
        self.native.groups.try_push(
            c,
            OriginGroup {
                start: self.native.origins.len(),
                length: 0,
                units,
                cause,
                retirement: None,
            },
        )?;
        Ok(reference)
    }
    pub(super) fn merge_groups(
        &mut self,
        c: &mut ParseControl,
        old: &OriginGroupRef,
        incoming: &OriginGroupRef,
        offset: usize,
        total: usize,
    ) -> Result<OriginGroupRef, SessionError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        let left = self.require_group(old)?;
        let right = self.require_group(incoming)?;
        if left.units != offset || offset.checked_add(right.units) != Some(total) {
            return Err(SessionError::WrongToken);
        }
        let original = old.index;
        let incoming_cause = incoming.index;
        let mut maps = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let prepared = (|| {
            for (group, offset) in [(old, 0), (incoming, offset)] {
                for origin in self.group_origins(group)? {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    payload_write::<TextOrigin>(c, 1)?;
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    maps.try_push(
                        c,
                        TextOrigin {
                            start: offset + origin.start,
                            end: offset + origin.end,
                            source: origin.source.clone(),
                            kind: origin.kind,
                        },
                    )?;
                }
            }
            Ok::<(), SessionError>(())
        })();
        if let Err(error) = prepared {
            maps.release(c)?;
            return Err(error);
        }
        let cause = match self.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Merge,
                source: None,
                captured: None,
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: None,
                original_node: None,
                intended_parent: None,
                original: Some(original),
                incoming: Some(incoming_cause),
                trigger: self.native.current,
            },
        ) {
            Ok(cause) => cause,
            Err(error) => {
                maps.release(c)?;
                return Err(error);
            }
        };
        let result = self.retain_group_parts(c, &mut maps, total, false, Some(cause));
        maps.release(c)?;
        let reference = result?;
        Ok(reference)
    }
}
impl OriginGroupRef {
    pub(super) fn require_disposal_owner(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        cleanup_owner(c, &self.control)
    }
}
impl SessionStorage {
    pub(super) fn pi_piece_origins(
        &self,
        c: &mut ParseControl,
        node: usize,
        input: std::ops::Range<usize>,
        output: std::ops::Range<usize>,
        transformed: bool,
        maps: &mut OwnedVec<TextOrigin>,
    ) -> Result<(), SessionError> {
        let (begin, end, output_start, output_end) =
            (input.start, input.end, output.start, output.end);
        if self
            .evidence
            .as_ref()
            .is_none_or(|evidence| evidence.native.source.is_none())
        {
            return Ok(());
        }
        let slot = self.field_slot(c, StringLocation::ProcessingInstructionData(node))?;
        let RetainedPayload::String {
            origins: Some(original),
            ..
        } = &self.payloads.values[slot]
        else {
            return Ok(());
        };
        let evidence = self.evidence.as_ref().ok_or(SessionError::WrongToken)?;
        if evidence.native.source.is_none() {
            return Ok(());
        }
        let origins = evidence.group_origins(original)?;
        let mut first = None;
        let mut last = None;
        let mut covered = begin;
        for origin in origins {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if origin.end <= begin || origin.start >= end {
                continue;
            }
            if origin.start != covered || origin.end > end {
                return Err(SessionError::OriginSplitUnavailable);
            }
            if !transformed {
                payload_write::<TextOrigin>(c, 1)?;
                c.charge(WorkCategory::ProviderCall, 1)?;
                maps.try_push(
                    c,
                    TextOrigin {
                        start: output_start + origin.start - begin,
                        end: output_start + origin.end - begin,
                        source: origin.source.clone(),
                        kind: origin.kind,
                    },
                )?;
            } else {
                if let Some(previous) = last
                    && previous != origin.source.start()
                {
                    return Err(SessionError::OriginSplitUnavailable);
                }
                first.get_or_insert(origin.source.start());
                last = Some(origin.source.end());
            }
            covered = origin.end;
        }
        if covered != end {
            return Err(SessionError::OriginSplitUnavailable);
        }
        if transformed
            && let (Some(source), Some(first), Some(last)) = (&evidence.native.source, first, last)
        {
            payload_write::<TextOrigin>(c, 1)?;
            c.charge(WorkCategory::ProviderCall, 1)?;
            if source.as_utf8().is_some() {
                c.charge(
                    WorkCategory::InputUnitRead,
                    u64::from(first != 0 && first != source.len())
                        + u64::from(last != 0 && last != source.len()),
                )?;
            }
            let source = source
                .range(first, last)
                .map_err(|_| SessionError::WrongToken)?;
            maps.try_push(
                c,
                TextOrigin {
                    start: output_start,
                    end: output_end,
                    source,
                    kind: OriginKind::RecoverySyntax,
                },
            )?;
        }
        Ok(())
    }
    pub(super) fn register_pi_field(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        value: FrozenUtf16,
        maps: OwnedVec<TextOrigin>,
    ) -> Result<usize, SessionError> {
        let slot = match self.field_slot(c, StringLocation::ProcessingInstructionData(node)) {
            Ok(slot) => slot,
            Err(error) => {
                value.release(c)?;
                maps.release(c)?;
                return Err(error);
            }
        };
        let original = match self.copied_slot_group(c, slot) {
            Ok(original) => original,
            Err(error) => {
                value.release(c)?;
                maps.release(c)?;
                return Err(error);
            }
        };
        let (native, slot) = self.register_field_copy(
            c,
            FieldCopy {
                value,
                origins: maps,
                original,
            },
        )?;
        drop(native);
        Ok(slot)
    }
    pub(super) fn begin_formatting_replay(
        &mut self,
        c: &mut ParseControl,
        original: Option<&ConstructionCauseRef>,
        intended_parent: Option<usize>,
    ) -> Result<Option<ConstructionCauseRef>, SessionError> {
        if intended_parent.is_some_and(|parent| parent >= self.records.len()) {
            return Err(SessionError::ForeignNode);
        }
        let Some(evidence) = &mut self.evidence else {
            return Ok(None);
        };
        let previous = evidence.current_cause(c)?;
        let original = if let Some(original) = original {
            cleanup_owner(c, &original.control)?;
            if original.issuer.as_ptr() != Arc::as_ptr(&evidence.identity)
                || original.index >= evidence.native.causes.len()
            {
                return Err(SessionError::WrongToken);
            }
            Some(original.index)
        } else {
            if evidence.native.source.is_some() {
                return Err(SessionError::WrongToken);
            }
            None // Source-independent construction has no lexical issuer.
        };
        let cause = evidence.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Replay,
                source: None,
                captured: None,
                rule: None,
                family: None,
                original_cause: original,
                retired_destination: None,
                original_node: None,
                intended_parent,
                original: None,
                incoming: None,
                trigger: evidence.native.current,
            },
        )?;
        payload_write::<Option<usize>>(c, 1)?;
        evidence.native.current = Some(cause);
        Ok(previous)
    }
    pub(super) fn copied_slot_group(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        let RetainedPayload::String { origins, .. } = &self.payloads.values[slot] else {
            return Err(SessionError::WrongToken);
        };
        origins
            .as_ref()
            .map(|group| {
                self.evidence
                    .as_mut()
                    .ok_or(SessionError::WrongToken)?
                    .copy_group(c, group)
            })
            .transpose()
    }
    pub(super) fn reset_group(
        &mut self,
        c: &mut ParseControl,
        old_slot: usize,
        incoming: Option<OriginGroupRef>,
        units: usize,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        let RetainedPayload::String { origins: old, .. } = &self.payloads.values[old_slot] else {
            return Err(SessionError::WrongToken);
        };
        let Some(evidence) = &mut self.evidence else {
            return Ok(None);
        };
        let incoming = match incoming {
            Some(group) => group,
            None => evidence.rule_group(c, units)?,
        };
        let group = evidence.require_group(&incoming)?;
        let (start, length) = (group.start, group.length);
        if group.units != units {
            return Err(SessionError::WrongToken);
        }
        let original = old
            .as_ref()
            .map(|old| evidence.require_group(old).map(|_| old.index))
            .transpose()?;
        let cause = evidence.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Replacement,
                source: None,
                captured: None,
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: None,
                original_node: None,
                intended_parent: None,
                original,
                incoming: Some(incoming.index),
                trigger: evidence.native.current,
            },
        )?;
        let index = evidence.native.groups.len();
        let reference = evidence.group_ref(index, c)?;
        evidence.native.groups.try_push(
            c,
            OriginGroup {
                start,
                length,
                units,
                cause,
                retirement: None,
            },
        )?;
        Ok(Some(reference))
    }
    /// Admit the historical relation before the caller's already-admitted
    /// native commit. A stopped draft never publishes native associations.
    pub(super) fn retire_field_group(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
    ) -> Result<(), SessionError> {
        let RetainedPayload::String {
            origins: Some(group),
            location,
            ..
        } = &self.payloads.values[slot]
        else {
            return Ok(());
        };
        group.require_disposal_owner(c)?;
        let evidence = self.evidence.as_mut().ok_or(SessionError::WrongToken)?;
        let header = evidence.require_group(group)?;
        if header.retirement.is_some() {
            return Err(SessionError::WrongToken);
        }
        let index = group.index;
        let cause = evidence.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Retirement,
                source: None,
                captured: None,
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: Some(*location),
                original_node: None,
                intended_parent: None,
                original: Some(index),
                incoming: None,
                trigger: evidence.native.current,
            },
        )?;
        payload_write::<Option<usize>>(c, 1)?;
        evidence
            .native
            .groups
            .get_mut(index, c)?
            .ok_or(SessionError::WrongToken)?
            .retirement = Some(cause);
        Ok(())
    }
    pub(super) fn merge_field_groups(
        &mut self,
        c: &mut ParseControl,
        old: usize,
        incoming: usize,
        offset: usize,
        total: usize,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        let RetainedPayload::String { origins: left, .. } = &self.payloads.values[old] else {
            return Err(SessionError::WrongToken);
        };
        let RetainedPayload::String { origins: right, .. } = &self.payloads.values[incoming] else {
            return Err(SessionError::WrongToken);
        };
        let Some(evidence) = &mut self.evidence else {
            return Ok(None);
        };
        // An absent group is explicitly source independent. It is never
        // attributed to the triggering token merely because a later input has
        // lexical origins.
        let missing_left = if left.is_none() {
            Some(evidence.source_independent_group(c, offset)?)
        } else {
            None
        };
        let missing_right = if right.is_none() {
            Some(evidence.source_independent_group(c, total - offset)?)
        } else {
            None
        };
        let group = evidence.merge_groups(
            c,
            left.as_ref()
                .or(missing_left.as_ref())
                .ok_or(SessionError::WrongToken)?,
            right
                .as_ref()
                .or(missing_right.as_ref())
                .ok_or(SessionError::WrongToken)?,
            offset,
            total,
        )?;
        Ok(Some(group))
    }
    pub(super) fn group_origins(
        &self,
        group: &OriginGroupRef,
    ) -> Result<&[TextOrigin], SessionError> {
        self.evidence
            .as_ref()
            .ok_or(SessionError::WrongToken)?
            .group_origins(group)
    }
    pub(super) fn origin_evidence(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<&mut ParseEvidence, SessionError> {
        if self.evidence.is_none() {
            // Explicit source-independent Text ingress opts into retained facts;
            // ordinary native values do not acquire a parser source certificate.
            self.evidence = Some(ParseEvidence::prepare(c)?);
        }
        self.evidence.as_mut().ok_or(SessionError::WrongToken)
    }
}
/// A controlled intrinsic copy retains its particular original field group,
/// independently of the later token/rule that triggers installation.
pub(super) struct FieldCopy {
    pub(super) value: FrozenUtf16,
    pub(super) origins: OwnedVec<TextOrigin>,
    pub(super) original: Option<OriginGroupRef>,
}
impl FieldCopy {
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.value.release(c)?;
        self.origins.release(c)
    }
}
impl ParseEvidence {
    pub(super) fn current_cause(
        &self,
        c: &mut ParseControl,
    ) -> Result<Option<ConstructionCauseRef>, SessionError> {
        self.native
            .current
            .map(|index| self.cause_ref(index, c))
            .transpose()
    }
    pub(super) fn activate_cause(
        &mut self,
        c: &mut ParseControl,
        cause: &ConstructionCauseRef,
    ) -> Result<(), SessionError> {
        cleanup_owner(c, &cause.control)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if cause.issuer.as_ptr() != Arc::as_ptr(&self.identity)
            || cause.index >= self.native.causes.len()
        {
            return Err(SessionError::WrongToken);
        }
        payload_write::<Option<usize>>(c, 1)?;
        self.native.current = Some(cause.index);
        Ok(())
    }
    pub(super) fn copy_group_reference(
        &self,
        c: &mut ParseControl,
        group: &OriginGroupRef,
    ) -> Result<OriginGroupRef, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        self.require_group(group)?;
        self.group_ref(group.index, c)
    }
    fn replacement_cause(
        &mut self,
        c: &mut ParseControl,
        original: &OriginGroupRef,
    ) -> Result<usize, SessionError> {
        self.require_group(original)?;
        self.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Replacement,
                source: None,
                captured: None,
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: None,
                original_node: None,
                intended_parent: None,
                original: Some(original.index),
                incoming: None,
                trigger: self.native.current,
            },
        )
    }
}
impl SessionStorage {
    pub(super) fn register_field_copy(
        &mut self,
        c: &mut ParseControl,
        copy: FieldCopy,
    ) -> Result<(DocumentString, usize), SessionError> {
        let FieldCopy {
            value,
            mut origins,
            original,
        } = copy;
        let evidence = match self.origin_evidence(c) {
            Ok(evidence) => evidence,
            Err(error) => {
                origins.release(c)?;
                value.release(c)?;
                return Err(error);
            }
        };
        let result = (|| {
            let cause = original
                .as_ref()
                .map(|original| evidence.replacement_cause(c, original))
                .transpose()?;
            evidence.retain_group_parts(c, &mut origins, value.text().len(), true, cause)
        })();
        origins.release(c)?;
        let group = match result {
            Ok(group) => group,
            Err(error) => {
                value.release(c)?;
                return Err(error);
            }
        };
        self.register_group_string_owned(c, value, Some(group))
    }
}
impl ParserArenaSession {
    pub(crate) fn current_parser_cause(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<Option<ConstructionCauseRef>, SessionError> {
        self.evidence(c)?.current_cause(c)
    }
    pub(crate) fn begin_buffered_replay(
        &mut self,
        c: &mut ParseControl,
        original: Option<&ConstructionCauseRef>,
    ) -> Result<Option<ConstructionCauseRef>, SessionError> {
        self.active_mut()?
            .begin_formatting_replay(c, original, None)
    }
    pub(crate) fn finish_buffered_replay(
        &mut self,
        c: &mut ParseControl,
        previous: Option<&ConstructionCauseRef>,
    ) -> Result<(), SessionError> {
        self.active_mut()?.end_node_copy(c, previous)
    }
    pub(crate) fn record_parser_trigger(
        &mut self,
        c: &mut ParseControl,
        trigger: ParserTrigger<'_>,
    ) -> Result<ConstructionCauseRef, SessionError> {
        self.evidence(c)?.parser_trigger(c, trigger)
    }
}

/// Read-only completed evidence with the same execution controller. A failed
/// projection has no lender: its admitted suffix remains cleanup-only.
pub(crate) struct NativeAssociations<'a> {
    storage: &'a SessionStorage,
    evidence: &'a ParseEvidence,
    control: &'a mut ParseControl,
}
impl NativeAssociations<'_> {
    pub(crate) fn raw_for(
        &mut self,
        index: usize,
        node: &ParserNode,
        kind: RawFieldKind,
    ) -> Result<bool, SessionError> {
        let node = self.storage.index(self.control, node)?;
        let Some((row, _)) = self.row(index)? else {
            return Ok(false);
        };
        Ok(
            matches!(row.destination, NativeDestination::Raw { node: owner, kind: actual } if owner == node && actual == kind),
        )
    }
    /// Historical groups remain in the same issuer after their native backing
    /// is retired. Fixed-step borrowed readback performs no new retention.
    pub(crate) fn group(
        &mut self,
        index: usize,
    ) -> Result<Option<(&OriginGroup, &[TextOrigin])>, SessionError> {
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        let Some(group) = self.evidence.native.groups.as_slice().get(index) else {
            return Ok(None);
        };
        let end = group
            .start
            .checked_add(group.length)
            .ok_or(SessionError::WrongToken)?;
        let origins = self
            .evidence
            .native
            .origins
            .as_slice()
            .get(group.start..end)
            .ok_or(SessionError::WrongToken)?;
        Ok(Some((group, origins)))
    }
    pub(crate) fn cause_at(
        &mut self,
        index: usize,
    ) -> Result<Option<&ConstructionCause>, SessionError> {
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self.evidence.native.causes.as_slice().get(index))
    }
    pub(crate) fn node_for(
        &mut self,
        index: usize,
        node: &ParserNode,
    ) -> Result<bool, SessionError> {
        let node = self.storage.index(self.control, node)?;
        let Some((row, _)) = self.row(index)? else {
            return Ok(false);
        };
        Ok(matches!(row.destination, NativeDestination::Node(owner) if owner == node))
    }
    pub(crate) fn text_for(
        &mut self,
        index: usize,
        node: &ParserNode,
    ) -> Result<bool, SessionError> {
        let node = self.storage.index(self.control, node)?;
        let Some((row, _)) = self.row(index)? else {
            return Ok(false);
        };
        Ok(
            matches!(row.destination, NativeDestination::Field(StringLocation::Text(owner)) if owner == node),
        )
    }
    pub(crate) fn cause_reference(
        &mut self,
        index: usize,
    ) -> Result<Option<ConstructionCauseRef>, SessionError> {
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        let rows = self
            .evidence
            .native
            .projection
            .as_ref()
            .ok_or(SessionError::WrongToken)?;
        let Some(row) = rows.as_slice().get(index) else {
            return Ok(None);
        };
        Ok(Some(self.evidence.cause_ref(row.cause, self.control)?))
    }
    pub(crate) fn len(&self) -> usize {
        self.evidence
            .native
            .projection
            .as_ref()
            .expect("qualified projection")
            .len()
    }
    pub(crate) fn row(
        &mut self,
        index: usize,
    ) -> Result<Option<(&NativeAssociation, &ConstructionCause)>, SessionError> {
        self.control.charge(WorkCategory::RecordVisit, 2)?;
        let rows = self
            .evidence
            .native
            .projection
            .as_ref()
            .ok_or(SessionError::WrongToken)?;
        rows.require_disposal_owner(self.control)?;
        let Some(row) = rows.as_slice().get(index) else {
            return Ok(None);
        };
        let cause = self
            .evidence
            .native
            .causes
            .as_slice()
            .get(row.cause)
            .ok_or(SessionError::WrongToken)?;
        Ok(Some((row, cause)))
    }
    pub(crate) fn cause(
        &mut self,
        reference: &ConstructionCauseRef,
    ) -> Result<&ConstructionCause, SessionError> {
        cleanup_owner(self.control, &reference.control)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        if reference.issuer.as_ptr() != Arc::as_ptr(&self.evidence.identity) {
            return Err(SessionError::WrongToken);
        }
        self.evidence
            .native
            .causes
            .as_slice()
            .get(reference.index)
            .ok_or(SessionError::WrongToken)
    }
}
impl ParseEvidence {
    fn projection_row(
        &mut self,
        c: &mut ParseControl,
        destination: NativeDestination,
        output: std::ops::Range<usize>,
        source: Option<&crate::HtmlSourceRange>,
        kind: Option<OriginKind>,
        cause: usize,
    ) -> Result<(), SessionError> {
        if let Some(source) = source {
            c.require_source_range(source)?;
        }
        payload_write::<NativeAssociation>(c, 1)?;
        if source.is_some() {
            c.charge(WorkCategory::ProviderCall, 1)?;
        }
        self.native
            .projection
            .as_mut()
            .ok_or(SessionError::WrongToken)?
            .try_push(
                c,
                NativeAssociation {
                    destination,
                    start: output.start,
                    end: output.end,
                    source: source.cloned(),
                    kind,
                    cause,
                },
            )?;
        Ok(())
    }
    fn project_field(
        &mut self,
        c: &mut ParseControl,
        location: NativeDestination,
        units: usize,
        reference: &OriginGroupRef,
    ) -> Result<(), SessionError> {
        reference.require_disposal_owner(c)?;
        let group = self.require_group(reference)?;
        if group.units != units {
            return Err(SessionError::WrongToken);
        }
        let (start, length, cause) = (group.start, group.length, group.cause);
        let mut covered = 0;
        for index in start..start + length {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let origin = &self.native.origins.as_slice()[index];
            if origin.start < covered || origin.end <= origin.start || origin.end > units {
                return Err(SessionError::WrongToken);
            }
            if origin.start > covered {
                self.projection_row(c, location, covered..origin.start, None, None, cause)?;
            }
            // Copy one admitted, atomic source relation. Surrogate/entity
            // expansion is retained as its original whole group, never divided
            // proportionally to source width.
            let origin = &self.native.origins.as_slice()[index];
            c.require_source_range(&origin.source)?;
            payload_write::<NativeAssociation>(c, 1)?;
            c.charge(WorkCategory::ProviderCall, 1)?;
            let (begin, end, source, kind) =
                (origin.start, origin.end, origin.source.clone(), origin.kind);
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .try_push(
                    c,
                    NativeAssociation {
                        destination: location,
                        start: begin,
                        end,
                        source: Some(source),
                        kind: Some(kind),
                        cause,
                    },
                )?;
            covered = end;
        }
        if covered < units || units == 0 {
            self.projection_row(c, location, covered..units, None, None, cause)?;
        }
        Ok(())
    }
}
impl ParserArenaSession {
    pub(crate) fn prepare_native_associations(
        &mut self,
        c: &mut ParseControl,
        source: &crate::HtmlSource,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        storage.require(c)?;
        c.require_source(source)?;
        storage.payloads.check_aliases(&storage.records, c)?;
        let evidence = storage.evidence.as_mut().ok_or(SessionError::WrongToken)?;
        if evidence
            .native
            .source
            .as_ref()
            .is_none_or(|owned| owned.identity() != source.identity())
        {
            return Err(SessionError::WrongToken);
        }
        if evidence.native.projection_complete {
            return Ok(());
        }
        if evidence.native.projection.is_some() {
            return Err(SessionError::WrongToken);
        }
        let mut rows = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        rows.require_owner(c)?;
        payload_write::<Option<OwnedVec<NativeAssociation>>>(c, 1)?;
        evidence.native.projection = Some(rows);
        for payload in &storage.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let raw = match payload {
                RetainedPayload::HiddenInput {
                    node: Some(node),
                    raw,
                    complete: true,
                    ..
                } => Some((*node, *raw, RawFieldKind::HiddenInput)),
                RetainedPayload::Textarea {
                    node: Some(node),
                    state,
                } => Some((*node, state.raw, RawFieldKind::Textarea)),
                _ => None,
            };
            if let Some((node, slot, kind)) = raw {
                if node >= storage.records.len() {
                    return Err(SessionError::WrongToken);
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let RetainedPayload::String {
                    value,
                    allocation,
                    origins,
                    location: StringLocation::Detached,
                } = storage
                    .payloads
                    .values
                    .get(slot)
                    .ok_or(SessionError::WrongToken)?
                else {
                    return Err(SessionError::WrongToken);
                };
                c.require_document_string(allocation, value)?;
                let missing = if origins.is_none() {
                    Some(evidence.source_independent_group(c, value.len())?)
                } else {
                    None
                };
                let group = origins
                    .as_ref()
                    .or(missing.as_ref())
                    .ok_or(SessionError::WrongToken)?;
                evidence.validate_field(c, value.code_units(), group)?;
                evidence.project_field(
                    c,
                    NativeDestination::Raw { node, kind },
                    value.len(),
                    group,
                )?;
                continue;
            }
            if let RetainedPayload::ConstructionCause {
                node: Some(node),
                cause,
            } = payload
            {
                cleanup_owner(c, &cause.control)?;
                if cause.issuer.as_ptr() != Arc::as_ptr(&evidence.identity)
                    || *node >= storage.records.len()
                {
                    return Err(SessionError::WrongToken);
                }
                payload_write::<NativeAssociation>(c, 1)?;
                evidence
                    .native
                    .projection
                    .as_mut()
                    .ok_or(SessionError::WrongToken)?
                    .try_push(
                        c,
                        NativeAssociation {
                            destination: NativeDestination::Node(*node),
                            start: 0,
                            end: 0,
                            source: None,
                            kind: None,
                            cause: cause.index,
                        },
                    )?;
                continue;
            }
            let RetainedPayload::String {
                value,
                allocation,
                origins,
                location,
            } = payload
            else {
                continue;
            };
            c.require_document_string(allocation, value)?;
            let Some(native) = location.value(&storage.records, &storage.payloads.values, c)?
            else {
                continue;
            };
            c.charge(WorkCategory::ProviderCall, 1)?;
            if !Arc::ptr_eq(native.backing(), value.backing()) {
                return Err(SessionError::AliasedPayload);
            }
            if let Some(group) = origins {
                evidence.validate_field(c, native.code_units(), group)?;
                evidence.project_field(
                    c,
                    NativeDestination::Field(*location),
                    value.len(),
                    group,
                )?;
            } else {
                let group = evidence.source_independent_group(c, value.len())?;
                evidence.project_field(
                    c,
                    NativeDestination::Field(*location),
                    value.len(),
                    &group,
                )?;
            }
        }
        c.checkpoint()?;
        payload_write::<bool>(c, 1)?;
        evidence.native.projection_complete = true;
        Ok(())
    }
    pub(crate) fn native_associations<'a>(
        &'a self,
        c: &'a mut ParseControl,
    ) -> Result<NativeAssociations<'a>, SessionError> {
        let storage = self.active()?;
        storage.require(c)?;
        let evidence = storage.evidence.as_ref().ok_or(SessionError::WrongToken)?;
        if !evidence.native.projection_complete {
            return Err(SessionError::WrongToken);
        }
        payload_write::<NativeAssociations>(c, 1)?;
        Ok(NativeAssociations {
            storage,
            evidence,
            control: c,
        })
    }
}

/// Issued by the capability-qualified session/context owner; callers cannot
/// replace the selected capture with unrelated IDs or pretend it is lexical.
pub(crate) struct CapturedParserTrigger<'a> {
    context: &'a crate::HtmlFragmentContext,
    role: CapturedCauseRole,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CapturedCauseRole {
    FragmentBootstrap,
    ContextNamespace,
}
pub(crate) struct CapturedConstructionContext {
    pub(crate) capture: crate::HtmlCaptureId,
    pub(crate) host_revision: crate::HtmlHostFactsRevision,
    pub(crate) context: crate::HtmlCaptureElementId,
    pub(crate) target: crate::HtmlFragmentTarget,
    pub(crate) context_document: crate::HtmlCaptureDocumentId,
    pub(crate) target_document: crate::HtmlCaptureDocumentId,
    pub(crate) role: CapturedCauseRole,
}
impl<'a> CapturedParserTrigger<'a> {
    pub(super) fn new(context: &'a crate::HtmlFragmentContext, role: CapturedCauseRole) -> Self {
        Self { context, role }
    }
}
impl ParseEvidence {
    pub(super) fn retain_captured_cause(
        &mut self,
        c: &mut ParseControl,
        trigger: CapturedParserTrigger<'_>,
    ) -> Result<ConstructionCauseRef, SessionError> {
        let context = trigger.context;
        // The context already validates all identities against one immutable
        // capture. Fixed clone/layout admission precedes owning these handles.
        c.charge(WorkCategory::ProviderCall, 7)?;
        payload_write::<CapturedConstructionContext>(c, 1)?;
        let captured = CapturedConstructionContext {
            capture: context.capture().identity().clone(),
            host_revision: context.capture().host_facts_revision().clone(),
            context: context.context_element().clone(),
            target: context.target().clone(),
            context_document: context.context_document().clone(),
            target_document: context.target_document().clone(),
            role: trigger.role,
        };
        let index = self.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Captured,
                source: None,
                captured: Some(captured),
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: None,
                original_node: None,
                intended_parent: None,
                original: None,
                incoming: None,
                trigger: self.native.current,
            },
        )?;
        let reference = self.cause_ref(index, c)?;
        payload_write::<Option<usize>>(c, 1)?;
        self.native.current = Some(index);
        Ok(reference)
    }
    pub(super) fn retain_captured_namespace(
        &mut self,
        c: &mut ParseControl,
        trigger: CapturedParserTrigger<'_>,
        actual: &DocumentString,
    ) -> Result<OriginGroupRef, SessionError> {
        if trigger.role != CapturedCauseRole::ContextNamespace {
            return Err(SessionError::WrongToken);
        }
        c.charge(WorkCategory::ProviderCall, 1)?;
        let node = trigger
            .context
            .capture()
            .node(trigger.context.context_element().subject())
            .map_err(|_| SessionError::WrongToken)?;
        let crate::HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Err(SessionError::WrongToken);
        };
        let uri = element
            .name()
            .namespace()
            .uri()
            .ok_or(SessionError::WrongToken)?;
        if !exact_string_equal(c, uri, actual)? {
            return Err(SessionError::WrongToken);
        }
        let cause = self.retain_captured_cause(c, trigger)?;
        let index = self.native.groups.len();
        let reference = self.group_ref(index, c)?;
        self.native.groups.try_push(
            c,
            OriginGroup {
                start: self.native.origins.len(),
                length: 0,
                units: actual.len(),
                cause: cause.index,
                retirement: None,
            },
        )?;
        Ok(reference)
    }
}

impl SessionStorage {
    pub(super) fn prepare_node_causes(
        &mut self,
        c: &mut ParseControl,
        count: usize,
    ) -> Result<[Option<usize>; 2], SessionError> {
        if count > 2 {
            return Err(SessionError::WrongToken);
        }
        let mut slots = [None; 2];
        for entry in slots.iter_mut().take(count) {
            let Some(evidence) = &mut self.evidence else {
                break;
            };
            if evidence.native.source.is_none() {
                break;
            }
            let cause = evidence.field_cause(c)?;
            let cause = evidence.cause_ref(cause, c)?;
            // Native binding is a fixed write in the later allocation commit.
            payload_write::<Option<usize>>(c, 1)?;
            self.prepare_payload_slot_owned(c)?;
            let slot = self.payloads.values.len();
            self.payloads
                .values
                .push(RetainedPayload::ConstructionCause { node: None, cause });
            *entry = Some(slot);
        }
        Ok(slots)
    }
    pub(super) fn begin_node_copy(
        &mut self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<Option<ConstructionCauseRef>, SessionError> {
        if source >= self.records.len() {
            return Err(SessionError::ForeignNode);
        }
        let Some(evidence) = &mut self.evidence else {
            return Ok(None);
        };
        let previous = evidence.current_cause(c)?;
        let index = evidence.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Copy,
                source: None,
                captured: None,
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: None,
                original_node: Some(source),
                intended_parent: None,
                original: None,
                incoming: None,
                trigger: evidence.native.current,
            },
        )?;
        payload_write::<Option<usize>>(c, 1)?;
        evidence.native.current = Some(index);
        Ok(previous)
    }
    pub(super) fn end_node_copy(
        &mut self,
        c: &mut ParseControl,
        previous: Option<&ConstructionCauseRef>,
    ) -> Result<(), SessionError> {
        let Some(evidence) = &mut self.evidence else {
            return Ok(());
        };
        if let Some(previous) = previous {
            evidence.activate_cause(c, previous)
        } else {
            payload_write::<Option<usize>>(c, 1)?;
            evidence.native.current = None;
            Ok(())
        }
    }
}

#[cfg(test)]
mod captured_tests {
    use super::*;
    use crate::{
        HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCapturedIsValue,
        HtmlDocumentMode, HtmlFragmentContext, HtmlHostFactsRevision, HtmlScriptingStatus,
    };
    #[test]
    fn captured_namespace_and_bootstrap_keep_actual_subject_revision_and_nonlexical_origin() {
        let revision = HtmlHostFactsRevision::fresh();
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(2, 4, 0, 1000, 10, 0, 0),
            revision.clone(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let element = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::svg(), None, "g"),
                OrderedAttributes::new(Vec::new()).unwrap(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&element).unwrap();
        let capture = builder.finish().unwrap();
        let subject = capture.subject_id(&element).unwrap();
        let context = HtmlFragmentContext::element(&capture, &subject).unwrap();
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
        let bootstrap = evidence
            .retain_captured_cause(
                &mut c,
                CapturedParserTrigger::new(&context, CapturedCauseRole::FragmentBootstrap),
            )
            .unwrap();
        let uri = DocumentString::from("http://www.w3.org/2000/svg");
        let group = evidence
            .retain_captured_namespace(
                &mut c,
                CapturedParserTrigger::new(&context, CapturedCauseRole::ContextNamespace),
                &uri,
            )
            .unwrap();
        assert!(evidence.group_origins(&group).unwrap().is_empty());
        let header = evidence.require_group(&group).unwrap();
        assert_eq!(header.units, 26);
        let cause = &evidence.native.causes.as_slice()[header.cause];
        assert_eq!(cause.operation, ConstructionOperation::Captured);
        assert!(cause.source.is_none());
        let retained = cause.captured.as_ref().unwrap();
        assert_eq!(&retained.capture, capture.identity());
        assert_eq!(retained.host_revision, revision);
        assert_eq!(&retained.context, context.context_element());
        assert_eq!(&retained.target, context.target());
        assert_eq!(&retained.context_document, context.context_document());
        assert_eq!(&retained.target_document, context.target_document());
        assert_eq!(retained.role, CapturedCauseRole::ContextNamespace);
        assert_eq!(cause.trigger, Some(bootstrap.index));
        assert_eq!(evidence.native.causes.len(), 2);
        assert!(matches!(
            evidence.retain_captured_namespace(
                &mut c,
                CapturedParserTrigger::new(&context, CapturedCauseRole::ContextNamespace),
                &DocumentString::from("other")
            ),
            Err(SessionError::WrongToken)
        ));
        assert_eq!(evidence.native.causes.len(), 2);
        assert!(c.stop().is_none());
        drop(group);
        drop(bootstrap);
        evidence.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
    }
}

#[cfg(test)]
mod merge_cleanup_tests {
    use super::*;
    use crate::{HtmlParseCancellation, HtmlParseLimits, HtmlSource};

    fn inputs(
        limits: HtmlParseLimits,
        cancel: HtmlParseCancellation,
    ) -> (ParseControl, ParseEvidence, OriginGroupRef, OriginGroupRef) {
        let source = HtmlSource::utf8("AB");
        let mut c = ParseControl::new(limits, cancel);
        c.admit_source(&source).unwrap();
        let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
        evidence.initialize_native_source(&mut c, &source).unwrap();
        let mut group = |begin| {
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
        let old = group(0);
        let incoming = group(1);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            5
        );
        (c, evidence, old, incoming)
    }

    fn assert_original_groups(
        evidence: &ParseEvidence,
        old: &OriginGroupRef,
        incoming: &OriginGroupRef,
    ) {
        let left = evidence.group_origins(old).unwrap();
        let right = evidence.group_origins(incoming).unwrap();
        assert_eq!((left.len(), right.len()), (1, 1));
        assert_eq!(
            (
                left[0].start,
                left[0].end,
                left[0].source.start(),
                left[0].source.end()
            ),
            (0, 1, 0, 1)
        );
        assert_eq!(
            (
                right[0].start,
                right[0].end,
                right[0].source.start(),
                right[0].source.end()
            ),
            (0, 1, 1, 2)
        );
        assert_eq!(
            (
                evidence.native.origins.len(),
                evidence.native.groups.len(),
                evidence.native.causes.len()
            ),
            (2, 2, 1)
        );
    }

    #[test]
    fn merge_cause_entry_refusal_reclaims_both_real_scratch_rows_and_capacity() {
        let selected = crate::parser::tree_builder::tests::limits(100)
            .into_builder()
            .source_map_entries(7)
            .finish()
            .unwrap();
        let (mut c, mut evidence, old, incoming) = inputs(selected, HtmlParseCancellation::new());
        let before = c.usage().clone();
        let result = evidence.merge_groups(&mut c, &old, &incoming, 1, 2);
        assert!(matches!(
            result,
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::SourceMapEntries,
                limit: 7,
                current: 7,
                requested_delta: 1
            }))
        ));
        let first = c.stop();
        assert_original_groups(&evidence, &old, &incoming);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .high_water,
            7
        );
        // Five original entries survive; the two locally prepared merge rows
        // and their actual vector capacity must already have been reclaimed.
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            5
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            before
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
        );
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        drop((old, incoming));
        evidence.release(&mut c).unwrap();
        assert_eq!(c.stop(), first);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        crate::parser::tree_builder::tests::assert_released(&c);
    }

    #[test]
    fn merge_cause_work_refusal_reclaims_scratch_without_charging_the_denied_cause() {
        let limits = crate::parser::tree_builder::tests::limits(100);
        let (mut c, evidence, old, incoming) = inputs(limits, HtmlParseCancellation::new());
        let base = c.usage().resource(HtmlParseResource::WorkCredits).current;
        drop((old, incoming));
        evidence.release(&mut c).unwrap();
        // Two group visits, two origin visits, four fixed copy/write submissions,
        // two origin handle copies and two reserves (0→1→2 actual slots).
        let prepared = 8 * mem::size_of::<TextOrigin>() as u64 + 12;
        let denied = 4 * mem::size_of::<ConstructionCause>() as u64 + 2;
        let selected = limits
            .into_builder()
            .work_credits(base + prepared)
            .finish()
            .unwrap();
        let (mut c, mut evidence, old, incoming) = inputs(selected, HtmlParseCancellation::new());
        let before = c.usage().clone();
        assert!(
            matches!(evidence.merge_groups(&mut c, &old, &incoming, 1, 2), Err(SessionError::Stop(ParseStop::LimitExceeded { resource: HtmlParseResource::WorkCredits, limit, current, requested_delta })) if limit == base + prepared && current == limit && requested_delta == denied)
        );
        let first = c.stop();
        assert_original_groups(&evidence, &old, &incoming);
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            base + prepared
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .high_water,
            7
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            5
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            before
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
        );
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        drop((old, incoming));
        evidence.release(&mut c).unwrap();
        assert_eq!(c.stop(), first);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        crate::parser::tree_builder::tests::assert_released(&c);
    }

    #[test]
    fn merge_exact_entries_keep_literal_rebased_maps_and_original_cause_groups() {
        let selected = crate::parser::tree_builder::tests::limits(100)
            .into_builder()
            .source_map_entries(9)
            .finish()
            .unwrap();
        let (mut c, mut evidence, old, incoming) = inputs(selected, HtmlParseCancellation::new());
        let merged = evidence
            .merge_groups(&mut c, &old, &incoming, 1, 2)
            .unwrap();
        let maps = evidence.group_origins(&merged).unwrap();
        assert_eq!(
            maps.iter()
                .map(|origin| (
                    origin.start,
                    origin.end,
                    origin.source.start(),
                    origin.source.end(),
                    origin.kind
                ))
                .collect::<Vec<_>>(),
            [
                (0, 1, 0, 1, OriginKind::Literal),
                (1, 2, 1, 2, OriginKind::Literal)
            ]
        );
        let header = evidence.require_group(&merged).unwrap();
        assert_eq!(header.units, 2);
        let cause = &evidence.native.causes.as_slice()[header.cause];
        assert_eq!(cause.operation, ConstructionOperation::Merge);
        assert_eq!(
            (cause.original, cause.incoming),
            (Some(old.index), Some(incoming.index))
        );
        assert_eq!(
            (
                evidence.native.origins.len(),
                evidence.native.groups.len(),
                evidence.native.causes.len()
            ),
            (4, 3, 2)
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            9
        );
        assert!(c.stop().is_none());
        drop((old, incoming, merged));
        evidence.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
    }

    #[test]
    fn merge_cancelled_at_entry_keeps_original_groups_and_cleanup_first_stop() {
        let cancellation = HtmlParseCancellation::new();
        let (mut c, mut evidence, old, incoming) = inputs(
            crate::parser::tree_builder::tests::limits(100),
            cancellation.clone(),
        );
        let before = c.usage().clone();
        cancellation.cancel();
        assert!(matches!(
            evidence.merge_groups(&mut c, &old, &incoming, 1, 2),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        assert_original_groups(&evidence, &old, &incoming);
        assert_eq!(c.usage(), &before);
        drop((old, incoming));
        evidence.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        crate::parser::tree_builder::tests::assert_released(&c);
    }
}
