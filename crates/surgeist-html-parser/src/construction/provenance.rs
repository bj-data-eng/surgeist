//! Native field facts owned by the source evidence issuer, not another tree.

use super::*;
use crate::keyed::{Table, slots};
#[path = "lexical.rs"]
mod lexical;
#[path = "reverse.rs"]
pub(crate) mod reverse;
use crate::parser::{control::Invariant, tokenizer::token::OriginKind};
pub use lexical::{LexicalDisposition, LexicalKind, LexicalRole, OmissionKind};
pub(crate) use lexical::{LexicalOccurrence, LexicalOmission, LexicalPart, LexicalRef};
pub(crate) use reverse::{AssociationRelation, AssociationView};

/// Local indices never leave their evidence identity through an unchecked API.
pub(crate) struct OriginGroupRef {
    pub(super) index: usize,
    issuer: Weak<evidence::EvidenceIdentity>,
    control: ControlIdentity,
}
pub(crate) struct ConstructionCauseRef {
    index: usize,
    issuer: Weak<evidence::EvidenceIdentity>,
    control: ControlIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParserRule {
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
pub enum ConstructionOperation {
    SourceIndependent,
    Request,
    Token,
    Merge,
    Copy,
    Replacement,
    Retirement,
    Replay,
    Captured,
    Lexical,
}

/// Source-ordered generated Range replacement, not a digit-origin claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeBranch {
    InvalidDefaultMidpoint,
    InvalidDefaultMinimum,
    Underflow,
    Overflow,
    StepCorrection,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeOperandRole {
    Minimum,
    Maximum,
    Step,
    ValueContent,
}
slots!(RangeOperandRole, RANGE_OPERAND_COUNT, {
    Minimum => 0,
    Maximum => 1,
    Step => 2,
    ValueContent => 3,
});
const RANGE_OPERAND_COUNT: usize = 4;
/// One value per [`RangeOperandRole`]; a lookup cannot be out of range.
pub(crate) type RangeOperandTable<T> = Table<RangeOperandRole, T, RANGE_OPERAND_COUNT>;
impl RangeOperandRole {
    pub(crate) fn index(self) -> usize {
        self as usize
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeOperandKind {
    NotConsulted,
    Absent,
    ConversionFailed,
    Finite,
    Any,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RangeEdge {
    pub(crate) group: usize,
    pub(crate) cause: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RangeOriginal {
    FreshEmpty,
    Group(RangeEdge),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RangeOperand {
    NotConsulted,
    Absent,
    ConversionFailed(RangeEdge),
    Finite(RangeEdge),
    Any(RangeEdge),
}
impl RangeOperand {
    pub(crate) fn kind(self) -> RangeOperandKind {
        match self {
            Self::NotConsulted => RangeOperandKind::NotConsulted,
            Self::Absent => RangeOperandKind::Absent,
            Self::ConversionFailed(_) => RangeOperandKind::ConversionFailed,
            Self::Finite(_) => RangeOperandKind::Finite,
            Self::Any(_) => RangeOperandKind::Any,
        }
    }
    pub(crate) fn edge(self) -> Option<RangeEdge> {
        match self {
            Self::ConversionFailed(edge) | Self::Finite(edge) | Self::Any(edge) => Some(edge),
            Self::NotConsulted | Self::Absent => None,
        }
    }
}
/// Fixed decisions reside in the actual canonical cause. Bound/default enums
/// reuse their existing HTML input owner; generated defaults are not raw facts.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RangeComputation {
    pub(crate) branch: RangeBranch,
    pub(crate) original: RangeOriginal,
    pub(crate) operands: RangeOperandTable<RangeOperand>,
    pub(crate) minimum: Option<crate::HtmlInputNumericBoundOrigin>,
    pub(crate) maximum: Option<crate::HtmlInputNumericBoundOrigin>,
    pub(crate) step: Option<crate::HtmlInputNumericStepOrigin>,
    pub(crate) base: Option<crate::HtmlInputNumericStepBaseOrigin>,
}
impl RangeComputation {
    pub(crate) fn edges(self) -> [Option<RangeEdge>; 5] {
        [
            match self.original {
                RangeOriginal::FreshEmpty => None,
                RangeOriginal::Group(edge) => Some(edge),
            },
            self.operands.get(RangeOperandRole::Minimum).edge(),
            self.operands.get(RangeOperandRole::Maximum).edge(),
            self.operands.get(RangeOperandRole::Step).edge(),
            self.operands.get(RangeOperandRole::ValueContent).edge(),
        ]
    }
}

/// The actual source selected by the Color update algorithm. A missing
/// installed default is distinct from an installed empty string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorValueSource {
    MissingInstalledValue,
    InstalledValue,
    CurrentValue,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorOperandRole {
    Alpha,
    Colorspace,
}
slots!(ColorOperandRole, COLOR_OPERAND_COUNT, {
    Alpha => 0,
    Colorspace => 1,
});
const COLOR_OPERAND_COUNT: usize = 2;
/// One value per [`ColorOperandRole`]; a lookup cannot be out of range.
pub(crate) type ColorOperandTable<T> = Table<ColorOperandRole, T, COLOR_OPERAND_COUNT>;
impl ColorOperandRole {
    pub(crate) fn index(self) -> usize {
        self as usize
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct ColorComputation {
    pub(crate) source: ColorValueSource,
    pub(crate) original: Option<RangeEdge>,
    pub(crate) operands: ColorOperandTable<Option<RangeEdge>>,
    pub(crate) space: crate::HtmlInputColorSpace,
    pub(crate) standard_failure: Option<surgeist_css::color::CssColorStandardFailure>,
    pub(crate) environment: surgeist_css::color::CssColorFactIdentity,
    pub(crate) value: surgeist_color::Color,
}
/// Only one actual generated-value operation can own a replacement cause.
#[derive(Clone, Copy, Debug)]
pub(crate) enum InputValueComputation {
    Range(RangeComputation),
    Color(ColorComputation),
}
impl InputValueComputation {
    pub(crate) fn original(self) -> Option<RangeEdge> {
        match self {
            Self::Range(value) => match value.original {
                RangeOriginal::FreshEmpty => None,
                RangeOriginal::Group(edge) => Some(edge),
            },
            Self::Color(value) => value.original,
        }
    }
    pub(crate) fn edges(self) -> [Option<RangeEdge>; 5] {
        match self {
            Self::Range(value) => value.edges(),
            Self::Color(value) => [
                value.original,
                value.operands.copy(ColorOperandRole::Alpha),
                value.operands.copy(ColorOperandRole::Colorspace),
                None,
                None,
            ],
        }
    }
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
    pub(crate) computation: Option<InputValueComputation>,
}
pub(crate) struct OriginGroup {
    pub(crate) body: GroupBody,
    pub(crate) units: usize,
    pub(crate) cause: usize,
    pub(crate) retirement: Option<usize>,
}
/// Canonical fixed storage, lent only after the actual D1/lexical EOF proof.
pub(super) struct HistoryData<'a> {
    pub(super) source: &'a crate::HtmlSource,
    pub(super) origins: &'a [TextOrigin],
    pub(super) groups: &'a [OriginGroup],
    pub(super) causes: &'a [ConstructionCause],
}
#[derive(Clone, Copy)]
pub(crate) enum LeafValue {
    Pending,
    Payload(usize),
    Empty,
}
#[derive(Clone, Copy)]
pub(crate) enum GroupBody {
    Leaf {
        start: usize,
        length: usize,
        value: LeafValue,
    },
    Concat {
        original: usize,
        incoming: usize,
        offset: usize,
    },
    Alias {
        original: usize,
    },
}

#[derive(Clone, Copy)]
struct GroupFrame {
    index: usize,
    offset: usize,
}
#[derive(Clone, Copy)]
pub(crate) struct LeafChunk {
    pub(crate) value: LeafValue,
    pub(crate) offset: usize,
    pub(crate) units: usize,
    start: usize,
    length: usize,
}
/// Iterative data traversal; causal/retirement links are not output children.
pub(crate) struct GroupLeafCursor {
    issuer: Weak<evidence::EvidenceIdentity>,
    owner: ControlIdentity,
    frames: Option<OwnedVec<GroupFrame>>,
}
impl GroupLeafCursor {
    pub(crate) fn new(
        evidence: &ParseEvidence,
        group: &OriginGroupRef,
        c: &mut ParseControl,
    ) -> Result<Self, SessionError> {
        c.checkpoint()?;
        group.require_disposal_owner(c)?;
        evidence.require_group(group)?;
        let mut frames = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let prepared = frames.try_push(
            c,
            GroupFrame {
                index: group.index,
                offset: 0,
            },
        );
        if let Err(stop) = prepared {
            frames.release(c)?;
            return Err(stop.into());
        }
        if let Err(stop) = payload_write::<Self>(c, 1).and_then(|()| {
            c.charge(WorkCategory::ProviderCall, 2)
                .map_err(SessionError::from)
        }) {
            frames.release(c)?;
            return Err(stop);
        }
        Ok(Self {
            issuer: group.issuer.clone(),
            owner: group.control.clone(),
            frames: Some(frames),
        })
    }
    fn require(&self, evidence: &ParseEvidence, c: &mut ParseControl) -> Result<(), SessionError> {
        c.require_identity(&self.owner)?;
        if self.frames.is_none() {
            return Err(SessionError::Consumed);
        }
        c.charge(WorkCategory::ProviderCall, 1)?;
        if self.issuer.as_ptr() != Arc::as_ptr(&evidence.identity) {
            return Err(c.fail_invariant(Invariant::ForeignEvidence).into());
        }
        Ok(())
    }
    pub(crate) fn next(
        &mut self,
        evidence: &ParseEvidence,
        c: &mut ParseControl,
    ) -> Result<Option<LeafChunk>, SessionError> {
        self.require(evidence, c)?;
        while let Some(frame) = self
            .frames
            .as_mut()
            .ok_or(SessionError::Consumed)?
            .try_pop(c)?
        {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(
                WorkCategory::InspectByte,
                mem::size_of::<OriginGroup>() as u64,
            )?;
            let group = evidence
                .native
                .groups
                .as_slice()
                .get(frame.index)
                .ok_or(SessionError::WrongToken)?;
            match group.body {
                GroupBody::Leaf {
                    start,
                    length,
                    value,
                } => {
                    if start
                        .checked_add(length)
                        .is_none_or(|end| end > evidence.native.origins.len())
                    {
                        return Err(SessionError::WrongToken);
                    }
                    payload_write::<LeafChunk>(c, 1)?;
                    return Ok(Some(LeafChunk {
                        value,
                        offset: frame.offset,
                        units: group.units,
                        start,
                        length,
                    }));
                }
                GroupBody::Alias { original } => {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    c.charge(WorkCategory::InspectByte, mem::size_of::<usize>() as u64)?;
                    if original >= frame.index
                        || evidence.native.groups.as_slice()[original].units != group.units
                    {
                        return Err(SessionError::WrongToken);
                    }
                    self.frames
                        .as_mut()
                        .ok_or(SessionError::Consumed)?
                        .try_push(
                            c,
                            GroupFrame {
                                index: original,
                                offset: frame.offset,
                            },
                        )?;
                }
                GroupBody::Concat {
                    original,
                    incoming,
                    offset,
                } => {
                    c.charge(WorkCategory::RecordVisit, 2)?;
                    c.charge(
                        WorkCategory::InspectByte,
                        const { (2 * mem::size_of::<usize>()) as u64 },
                    )?;
                    if original >= frame.index || incoming >= frame.index {
                        return Err(SessionError::WrongToken);
                    }
                    let left = &evidence.native.groups.as_slice()[original];
                    let right = &evidence.native.groups.as_slice()[incoming];
                    if left.units != offset || offset.checked_add(right.units) != Some(group.units)
                    {
                        return Err(SessionError::WrongToken);
                    }
                    let right_offset = frame.offset.checked_add(offset).ok_or_else(|| {
                        c.stop_representation(HtmlParseResource::ManagedUtf16Units)
                    })?;
                    self.frames
                        .as_mut()
                        .ok_or(SessionError::Consumed)?
                        .reserve_geometric(c, 2)?;
                    self.frames
                        .as_mut()
                        .ok_or(SessionError::Consumed)?
                        .try_push(
                            c,
                            GroupFrame {
                                index: incoming,
                                offset: right_offset,
                            },
                        )?;
                    self.frames
                        .as_mut()
                        .ok_or(SessionError::Consumed)?
                        .try_push(
                            c,
                            GroupFrame {
                                index: original,
                                offset: frame.offset,
                            },
                        )?;
                }
            }
        }
        Ok(None)
    }
    pub(crate) fn release(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        self.frames.as_ref().ok_or(SessionError::Consumed)?;
        // Disposal verifies the execution without budget-sensitive useful work.
        // A stop cannot strand the actual scratch allocation.
        cleanup_owner(c, &self.owner)?;
        self.frames
            .take()
            .ok_or(SessionError::Consumed)?
            .release(c)?;
        Ok(())
    }
}
pub(crate) struct GroupOriginCursor {
    leaves: GroupLeafCursor,
    current: Option<LeafChunk>,
    position: usize,
}
impl GroupOriginCursor {
    pub(crate) fn new(
        evidence: &ParseEvidence,
        group: &OriginGroupRef,
        c: &mut ParseControl,
    ) -> Result<Self, SessionError> {
        let mut leaves = GroupLeafCursor::new(evidence, group, c)?;
        if let Err(stop) = payload_write::<Self>(c, 1) {
            leaves.release(c)?;
            return Err(stop);
        }
        Ok(Self {
            leaves,
            current: None,
            position: 0,
        })
    }
    pub(crate) fn next(
        &mut self,
        evidence: &ParseEvidence,
        c: &mut ParseControl,
    ) -> Result<Option<TextOrigin>, SessionError> {
        self.leaves.require(evidence, c)?;
        loop {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(
                WorkCategory::InspectByte,
                const { (mem::size_of::<Option<LeafChunk>>() + mem::size_of::<usize>()) as u64 },
            )?;
            if let Some(leaf) = self.current
                && self.position < leaf.length
            {
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.charge(
                    WorkCategory::InspectByte,
                    mem::size_of::<TextOrigin>() as u64,
                )?;
                let origin = &evidence.native.origins.as_slice()[leaf.start + self.position];
                if origin.start >= origin.end || origin.end > leaf.units {
                    return Err(SessionError::WrongToken);
                }
                let start = leaf
                    .offset
                    .checked_add(origin.start)
                    .ok_or_else(|| c.stop_representation(HtmlParseResource::ManagedUtf16Units))?;
                let end = leaf
                    .offset
                    .checked_add(origin.end)
                    .ok_or_else(|| c.stop_representation(HtmlParseResource::ManagedUtf16Units))?;
                payload_write::<TextOrigin>(c, 1)?;
                c.charge(WorkCategory::ProviderCall, 1)?;
                payload_write::<usize>(c, 1)?;
                self.position += 1;
                return Ok(Some(TextOrigin {
                    start,
                    end,
                    source: origin.source.clone(),
                    kind: origin.kind,
                }));
            }
            let next = self.leaves.next(evidence, c)?;
            payload_write::<Option<LeafChunk>>(c, 1)?;
            payload_write::<usize>(c, 1)?;
            self.current = next;
            self.position = 0;
            if self.current.is_none() {
                return Ok(None);
            }
        }
    }
    pub(crate) fn release(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        self.leaves.release(c)
    }
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum NativeDestination {
    Field(StringLocation),
    Node(usize),
    Raw {
        node: usize,
        kind: RawFieldKind,
    },
    Lexical {
        occurrence: usize,
        part: Option<usize>,
    },
    Omission(usize),
    LexicalEnd,
    Cause(usize),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RawFieldKind {
    Nonce,
    Input,
    Textarea,
}
pub(crate) struct NativeAssociation {
    pub(crate) group: Option<usize>,
    pub(crate) relation: AssociationRelation,
    pub(crate) destination: NativeDestination,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) source: Option<crate::HtmlSourceRange>,
    pub(crate) kind: Option<OriginKind>,
    pub(crate) cause: usize,
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
    native_rows: usize,
    lexical: lexical::LexicalStorage,
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
            native_rows: 0,
            lexical: lexical::LexicalStorage::new(c)?,
        })
    }
    /// Fixed-header cleanup proof. Origin/group/cause, lexical occurrence/part/
    /// omission and optional projection vectors are flat SourceMapEntries owners;
    /// their source handles own no separately charged storage.
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
        let lexical = self.lexical.extents(c)?;
        for (units, bytes) in [
            extent(&self.origins, c)?,
            extent(&self.groups, c)?,
            extent(&self.causes, c)?,
            lexical[0],
            lexical[1],
            lexical[2],
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
        self.lexical.release(c)?;
        self.origins.release(c)?;
        self.groups.release(c)?;
        self.causes.release(c)
    }
}
impl ParseEvidence {
    fn group_value_equal(
        &self,
        c: &mut ParseControl,
        payloads: &PayloadStorage,
        value: &[u16],
        group: &OriginGroupRef,
    ) -> Result<bool, SessionError> {
        let mut cursor = GroupLeafCursor::new(self, group, c)?;
        let read = (|| {
            let mut covered = 0usize;
            let mut equal = true;
            while let Some(leaf) = cursor.next(self, c)? {
                if leaf.offset != covered {
                    return Err(SessionError::WrongToken);
                }
                match leaf.value {
                    LeafValue::Pending => return Err(SessionError::WrongToken),
                    LeafValue::Empty if leaf.units == 0 => {}
                    LeafValue::Empty => return Err(SessionError::WrongToken),
                    LeafValue::Payload(slot) => {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        let RetainedPayload::String {
                            value: text,
                            allocation,
                            ..
                        } = payloads.values.get(slot).ok_or(SessionError::WrongToken)?
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        c.require_document_string(allocation, text)?;
                        if text.len() != leaf.units {
                            return Err(SessionError::WrongToken);
                        }
                        for index in 0..leaf.units {
                            c.charge(WorkCategory::InspectByte, 4)?;
                            let position = leaf.offset.checked_add(index).ok_or_else(|| {
                                c.stop_representation(HtmlParseResource::ManagedUtf16Units)
                            })?;
                            if value.get(position) != text.code_units().get(index) {
                                equal = false;
                            }
                        }
                    }
                }
                covered = covered
                    .checked_add(leaf.units)
                    .ok_or_else(|| c.stop_representation(HtmlParseResource::ManagedUtf16Units))?;
            }
            Ok(equal && covered == value.len())
        })();
        cursor.release(c)?;
        read
    }
    fn validate_group_value(
        &self,
        c: &mut ParseControl,
        payloads: &PayloadStorage,
        value: &[u16],
        group: &OriginGroupRef,
    ) -> Result<(), SessionError> {
        if self.group_value_equal(c, payloads, value, group)? {
            Ok(())
        } else {
            Err(SessionError::WrongToken)
        }
    }
    fn origin_rows_equal(
        &self,
        c: &mut ParseControl,
        group: &OriginGroupRef,
        rows: &OwnedVec<TextOrigin>,
    ) -> Result<bool, SessionError> {
        rows.require_disposal_owner(c)?;
        let mut cursor = GroupOriginCursor::new(self, group, c)?;
        let read = (|| {
            for expected in rows.as_slice() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.charge(
                    WorkCategory::InspectByte,
                    mem::size_of::<TextOrigin>() as u64,
                )?;
                let Some(actual) = cursor.next(self, c)? else {
                    return Ok(false);
                };
                c.charge(WorkCategory::ProviderCall, 1)?;
                if actual.start != expected.start
                    || actual.end != expected.end
                    || actual.kind != expected.kind
                    || actual.source != expected.source
                {
                    return Ok(false);
                }
            }
            Ok(cursor.next(self, c)?.is_none())
        })();
        cursor.release(c)?;
        read
    }
    #[cfg(test)]
    pub(super) fn materialize_group(
        &self,
        c: &mut ParseControl,
        payloads: &PayloadStorage,
        group: &OriginGroupRef,
    ) -> Result<FrozenUtf16, SessionError> {
        let mut cursor = GroupLeafCursor::new(self, group, c)?;
        let mut output = match Utf16Buffer::new(c) {
            Ok(output) => output,
            Err(stop) => {
                cursor.release(c)?;
                return Err(stop.into());
            }
        };
        let read = (|| {
            let mut covered = 0usize;
            while let Some(leaf) = cursor.next(self, c)? {
                if leaf.offset != covered {
                    return Err(SessionError::WrongToken);
                }
                match leaf.value {
                    LeafValue::Pending => return Err(SessionError::WrongToken),
                    LeafValue::Empty if leaf.units == 0 => {}
                    LeafValue::Empty => return Err(SessionError::WrongToken),
                    LeafValue::Payload(slot) => {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        let RetainedPayload::String {
                            value, allocation, ..
                        } = payloads.values.get(slot).ok_or(SessionError::WrongToken)?
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        c.require_document_string(allocation, value)?;
                        if value.len() != leaf.units {
                            return Err(SessionError::WrongToken);
                        }
                        output.extend_units(value.code_units(), c)?;
                    }
                }
                covered = covered
                    .checked_add(leaf.units)
                    .ok_or_else(|| c.stop_representation(HtmlParseResource::ManagedUtf16Units))?;
            }
            if covered != self.require_group(group)?.units {
                return Err(SessionError::WrongToken);
            }
            Ok(())
        })();
        cursor.release(c)?;
        if let Err(error) = read {
            output.release(c)?;
            return Err(error);
        }
        Ok(output.freeze(c)?)
    }
    #[cfg(test)]
    pub(super) fn release_materialized_group(
        c: &mut ParseControl,
        slot: &mut Option<FrozenUtf16>,
    ) -> Result<(), SessionError> {
        let value = slot.as_ref().ok_or(SessionError::Consumed)?;
        c.require_materialized_string_disposal(value.allocation(), value.text())?;
        if Arc::strong_count(value.text().shared_code_units()) != 1 {
            return Err(SessionError::AliasedPayload);
        }
        // All possible refusal precedes take. Actual deallocation/refund does
        // not poll or change work, first-stop or high-water observations.
        slot.take().ok_or(SessionError::Consumed)?.release(c)?;
        Ok(())
    }
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
        let mut cursor = GroupOriginCursor::new(self, group, c)?;
        let read = (|| {
            while let Some(origin) = cursor.next(self, c)? {
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
                    OriginKind::CharacterReference
                    | OriginKind::RecoverySyntax
                    | OriginKind::InputValueNormalization => {
                        c.charge(WorkCategory::InspectByte, output.len() as u64 * 2)?;
                        if output.is_empty() || begin == end {
                            return Err(SessionError::WrongToken);
                        }
                    }
                }
            }
            Ok(())
        })();
        cursor.release(c)?;
        read
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
    pub(super) fn require_group(
        &self,
        group: &OriginGroupRef,
    ) -> Result<&OriginGroup, SessionError> {
        if group.issuer.as_ptr() != Arc::as_ptr(&self.identity) {
            return Err(SessionError::WrongToken);
        }
        self.native
            .groups
            .as_slice()
            .get(group.index)
            .ok_or(SessionError::WrongToken)
    }
    pub(super) fn history_data(&self) -> Result<HistoryData<'_>, SessionError> {
        // Do not mint an immutable history lender from a native-only projection
        // or an unfinished lexical prefix.
        self.associations()?;
        Ok(HistoryData {
            source: self
                .native
                .source
                .as_ref()
                .ok_or(SessionError::WrongToken)?,
            origins: self.native.origins.as_slice(),
            groups: self.native.groups.as_slice(),
            causes: self.native.causes.as_slice(),
        })
    }
    #[cfg(test)]
    pub(super) fn history_counts(&self) -> (usize, usize, usize) {
        (
            self.native.origins.len(),
            self.native.groups.len(),
            self.native.causes.len(),
        )
    }
    #[cfg(test)]
    pub(super) fn fixed_original_origin(&self, index: usize) -> Option<&TextOrigin> {
        self.native.origins.as_slice().get(index)
    }
    #[cfg(test)]
    pub(super) fn observed_origins(
        &self,
        group: &OriginGroupRef,
        c: &mut ParseControl,
    ) -> Result<Vec<TextOrigin>, SessionError> {
        let mut cursor = GroupOriginCursor::new(self, group, c)?;
        let mut result = Vec::new();
        let read: Result<(), SessionError> = (|| {
            while let Some(origin) = cursor.next(self, c)? {
                result.push(origin);
            }
            Ok(())
        })();
        cursor.release(c)?;
        read?;
        Ok(result)
    }
    pub(super) fn bind_leaf_value(
        &mut self,
        c: &mut ParseControl,
        group: &OriginGroupRef,
        slot: usize,
    ) -> Result<(), SessionError> {
        c.checkpoint()?;
        group.require_disposal_owner(c)?;
        self.require_group(group)?;
        let header = self
            .native
            .groups
            .get_mut(group.index, c)?
            .ok_or(SessionError::WrongToken)?;
        if let GroupBody::Leaf { value, .. } = &mut header.body {
            match value {
                LeafValue::Pending => {
                    payload_write::<LeafValue>(c, 1)?;
                    *value = if header.units == 0 {
                        LeafValue::Empty
                    } else {
                        LeafValue::Payload(slot)
                    };
                }
                LeafValue::Empty => {}
                LeafValue::Payload(_) => return Err(SessionError::WrongToken),
            }
        }
        Ok(())
    }
    pub(super) fn copy_group(
        &mut self,
        c: &mut ParseControl,
        original: &OriginGroupRef,
    ) -> Result<OriginGroupRef, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let group = self.require_group(original)?;
        let (units, original) = (group.units, original.index);
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
                computation: None,
            },
        )?;
        let index = self.native.groups.len();
        let reference = self.group_ref(index, c)?;
        self.native.groups.reserve_geometric(c, 1)?;
        self.native.groups.try_push(
            c,
            OriginGroup {
                body: GroupBody::Alias { original },
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
        self.native.causes.reserve_geometric(c, 1)?;
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
                computation: None,
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
                computation: None,
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
                computation: None,
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
            self.native.origins.reserve_geometric(c, 1)?;
            self.native.origins.try_push(c, origin)?;
        }
        for position in 0..length / 2 {
            self.native
                .origins
                .swap_slots(start + position, start + length - position - 1, c)?;
        }
        let index = self.native.groups.len();
        let reference = self.group_ref(index, c)?;
        self.native.groups.reserve_geometric(c, 1)?;
        self.native.groups.try_push(
            c,
            OriginGroup {
                body: GroupBody::Leaf {
                    start,
                    length,
                    value: if units == 0 {
                        LeafValue::Empty
                    } else {
                        LeafValue::Pending
                    },
                },
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
                computation: None,
            },
        )?;
        let index = self.native.groups.len();
        let reference = self.group_ref(index, c)?;
        self.native.groups.reserve_geometric(c, 1)?;
        self.native.groups.try_push(
            c,
            OriginGroup {
                body: GroupBody::Leaf {
                    start: self.native.origins.len(),
                    length: 0,
                    value: if units == 0 {
                        LeafValue::Empty
                    } else {
                        LeafValue::Pending
                    },
                },
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
        c.checkpoint()?;
        old.require_disposal_owner(c)?;
        incoming.require_disposal_owner(c)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let left = self.require_group(old)?;
        let right = self.require_group(incoming)?;
        if left.units != offset || offset.checked_add(right.units) != Some(total) {
            return Err(SessionError::WrongToken);
        }
        let cause = self.append_cause(
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
                original: Some(old.index),
                incoming: Some(incoming.index),
                trigger: self.native.current,
                computation: None,
            },
        )?;
        let index = self.native.groups.len();
        let reference = self.group_ref(index, c)?;
        self.native.groups.reserve_geometric(c, 1)?;
        self.native.groups.try_push(
            c,
            OriginGroup {
                body: GroupBody::Concat {
                    original: old.index,
                    incoming: incoming.index,
                    offset,
                },
                units: total,
                cause,
                retirement: None,
            },
        )?;
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
        let mut cursor = GroupOriginCursor::new(evidence, original, c)?;
        let mut first = None;
        let mut last = None;
        let mut covered = begin;
        let read = (|| {
            while let Some(origin) = cursor.next(evidence, c)? {
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
            Ok::<_, SessionError>(())
        })();
        cursor.release(c)?;
        read?;
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
        let records = self.construction.len(c)?;
        if intended_parent.is_some_and(|parent| parent >= records) {
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
                computation: None,
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
        let incoming_index = incoming.index;
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
                computation: None,
            },
        )?;
        let index = evidence.native.groups.len();
        let reference = evidence.group_ref(index, c)?;
        evidence.native.groups.reserve_geometric(c, 1)?;
        evidence.native.groups.try_push(
            c,
            OriginGroup {
                body: GroupBody::Alias {
                    original: incoming_index,
                },
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
        self.retire_field_group_at(c, slot, None)
    }
    /// The exclusive append owner suspended this aggregate's actual backing.
    /// Its historical row still owns the same immutable group.
    pub(super) fn retire_text_append_group(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        node: usize,
    ) -> Result<(), SessionError> {
        self.retire_field_group_at(c, slot, Some(StringLocation::Text(node)))
    }
    fn retire_field_group_at(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        suspended: Option<StringLocation>,
    ) -> Result<(), SessionError> {
        let (group, location) = match &self.payloads.values[slot] {
            RetainedPayload::String {
                origins: Some(group),
                location,
                ..
            } => (group, *location),
            RetainedPayload::HistoricalString { origins } => {
                (origins, suspended.ok_or(SessionError::WrongToken)?)
            }
            _ => return Ok(()),
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
                retired_destination: Some(location),
                original_node: None,
                intended_parent: None,
                original: Some(index),
                incoming: None,
                trigger: evidence.native.current,
                computation: None,
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
        if self.evidence.is_none() {
            return Ok(None);
        }
        let left = match &self.payloads.values[old] {
            RetainedPayload::String { origins, .. } => origins.as_ref(),
            RetainedPayload::HistoricalString { origins } => Some(origins),
            _ => return Err(SessionError::WrongToken),
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
            {
                let group = evidence.source_independent_group(c, offset)?;
                evidence.bind_leaf_value(c, &group, old)?;
                Some(group)
            }
        } else {
            None
        };
        let missing_right = if right.is_none() {
            {
                let group = evidence.source_independent_group(c, total - offset)?;
                evidence.bind_leaf_value(c, &group, incoming)?;
                Some(group)
            }
        } else {
            None
        };
        let group = evidence.merge_groups(
            c,
            left.or(missing_left.as_ref())
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
    #[cfg(test)]
    pub(super) fn observed_origins(
        &self,
        group: &OriginGroupRef,
        c: &mut ParseControl,
    ) -> Result<Vec<TextOrigin>, SessionError> {
        self.evidence
            .as_ref()
            .ok_or(SessionError::WrongToken)?
            .observed_origins(group, c)
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
    /// Only a real input computation can account for a nonempty leaf without
    /// lexical atoms. Empty fresh leaves and complete lexical leaves retain
    /// their existing meanings; missing atoms alone confer no authority.
    pub(super) fn qualify_generated_copy(
        &self,
        c: &mut ParseControl,
        payloads: &PayloadStorage,
        group: &OriginGroupRef,
        value: &[u16],
    ) -> Result<(), SessionError> {
        if !self.group_value_equal(c, payloads, value, group)? {
            return Err(SessionError::WrongToken);
        }
        let mut cursor = GroupLeafCursor::new(self, group, c)?;
        let result = (|| {
            while let Some(leaf) = cursor.next(self, c)? {
                if leaf.units == 0 {
                    continue;
                }
                if leaf.length == 0 {
                    let LeafValue::Payload(slot) = leaf.value else {
                        return Err(SessionError::WrongToken);
                    };
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let Some(RetainedPayload::String {
                        origins: Some(original),
                        ..
                    }) = payloads.values.get(slot)
                    else {
                        return Err(SessionError::WrongToken);
                    };
                    let header = self.require_group(original)?;
                    c.charge(
                        WorkCategory::InspectByte,
                        mem::size_of::<OriginGroup>() as u64,
                    )?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let cause = self
                        .native
                        .causes
                        .as_slice()
                        .get(header.cause)
                        .ok_or(SessionError::WrongToken)?;
                    c.charge(
                        WorkCategory::InspectByte,
                        mem::size_of::<ConstructionCause>() as u64,
                    )?;
                    let value = cause
                        .computation
                        .ok_or(SessionError::OriginSplitUnavailable)?;
                    if cause.operation != ConstructionOperation::Replacement {
                        return Err(SessionError::WrongToken);
                    }
                    validate_input_computation(
                        c,
                        value,
                        self.native.groups.as_slice(),
                        header.cause,
                    )?;
                } else {
                    let mut covered = 0;
                    for atom in
                        &self.native.origins.as_slice()[leaf.start..leaf.start + leaf.length]
                    {
                        c.charge(
                            WorkCategory::InspectByte,
                            mem::size_of::<TextOrigin>() as u64,
                        )?;
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        c.require_source_range(&atom.source)?;
                        if atom.start != covered || atom.end <= atom.start || atom.end > leaf.units
                        {
                            return Err(SessionError::OriginSplitUnavailable);
                        }
                        covered = atom.end;
                    }
                    if covered != leaf.units {
                        return Err(SessionError::OriginSplitUnavailable);
                    }
                }
            }
            Ok(())
        })();
        cursor.release(c)?;
        result
    }
}
impl ParseEvidence {
    /// The real payload's issued reference is checked before storing fixed
    /// graph indices. Callers cannot supply an arena slot as an origin proof.
    pub(super) fn range_edge(
        &self,
        c: &mut ParseControl,
        group: &OriginGroupRef,
    ) -> Result<RangeEdge, SessionError> {
        c.require_identity(&group.control)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<OriginGroup>() as u64,
        )?;
        let value = self.require_group(group)?;
        payload_write::<RangeEdge>(c, 1)?;
        Ok(RangeEdge {
            group: group.index,
            cause: value.cause,
        })
    }
}
pub(super) fn validate_range_computation(
    c: &mut ParseControl,
    value: RangeComputation,
    groups: &[OriginGroup],
    cause: usize,
) -> Result<(), SessionError> {
    c.charge(
        WorkCategory::InspectByte,
        mem::size_of::<RangeComputation>() as u64,
    )?;
    let step_branch = value.branch == RangeBranch::StepCorrection;
    if value.minimum.is_none()
        || value.maximum.is_none()
        || step_branch != (value.step.is_some() && value.base.is_some())
        || (!step_branch && (value.step.is_some() || value.base.is_some()))
    {
        return Err(SessionError::WrongToken);
    }
    if matches!(
        value.operands.get(RangeOperandRole::Minimum),
        RangeOperand::Any(_)
    ) || matches!(
        value.operands.get(RangeOperandRole::Maximum),
        RangeOperand::Any(_)
    ) || matches!(
        value.operands.get(RangeOperandRole::ValueContent),
        RangeOperand::Any(_)
    ) {
        return Err(SessionError::WrongToken);
    }
    for edge in value.edges() {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(edge) = edge {
            c.charge(
                WorkCategory::InspectByte,
                mem::size_of::<OriginGroup>() as u64,
            )?;
            if edge.cause >= cause
                || groups
                    .get(edge.group)
                    .is_none_or(|group| group.cause != edge.cause)
            {
                return Err(SessionError::WrongToken);
            }
        }
    }
    Ok(())
}
pub(super) fn validate_input_computation(
    c: &mut ParseControl,
    value: InputValueComputation,
    groups: &[OriginGroup],
    cause: usize,
) -> Result<(), SessionError> {
    match value {
        InputValueComputation::Range(range) => validate_range_computation(c, range, groups, cause),
        InputValueComputation::Color(color) => {
            c.charge(
                WorkCategory::InspectByte,
                mem::size_of::<ColorComputation>() as u64,
            )?;
            if (color.source == ColorValueSource::MissingInstalledValue) != color.original.is_none()
                || color.environment != super::input_color::environment_identity()
            {
                return Err(SessionError::WrongToken);
            }
            for edge in [
                color.original,
                color.operands.copy(ColorOperandRole::Alpha),
                color.operands.copy(ColorOperandRole::Colorspace),
            ] {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if let Some(edge) = edge {
                    c.charge(
                        WorkCategory::InspectByte,
                        mem::size_of::<OriginGroup>() as u64,
                    )?;
                    if edge.cause >= cause
                        || groups
                            .get(edge.group)
                            .is_none_or(|group| group.cause != edge.cause)
                    {
                        return Err(SessionError::WrongToken);
                    }
                }
            }
            Ok(())
        }
    }
}
impl SessionStorage {
    pub(super) fn range_payload_edge(
        &self,
        c: &mut ParseControl,
        slot: usize,
    ) -> Result<RangeEdge, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let RetainedPayload::String {
            origins: Some(group),
            ..
        } = self
            .payloads
            .values
            .get(slot)
            .ok_or(SessionError::WrongToken)?
        else {
            return Err(SessionError::WrongToken);
        };
        self.evidence
            .as_ref()
            .ok_or(SessionError::WrongToken)?
            .range_edge(c, group)
    }
    pub(super) fn prepare_range_group(
        &mut self,
        c: &mut ParseControl,
        units: usize,
        computation: RangeComputation,
    ) -> Result<OriginGroupRef, SessionError> {
        self.prepare_input_group(c, units, InputValueComputation::Range(computation))
    }
    pub(super) fn prepare_input_group(
        &mut self,
        c: &mut ParseControl,
        units: usize,
        computation: InputValueComputation,
    ) -> Result<OriginGroupRef, SessionError> {
        let evidence = self.evidence.as_mut().ok_or(SessionError::WrongToken)?;
        validate_input_computation(
            c,
            computation,
            evidence.native.groups.as_slice(),
            evidence.native.causes.len(),
        )?;
        // Every original/role qualification precedes reservation. These are
        // actual receiver capacities and required logical slots, not a ticket
        // dropped locally on a denied write.
        c.check_acquire_many(&[
            (HtmlParseResource::SourceMapEntries, 2),
            (HtmlParseResource::AuxiliaryEntries, 1),
            (HtmlParseResource::ManagedUtf16Units, units as u64),
        ])?;
        evidence.native.causes.reserve_geometric(c, 1)?;
        evidence.native.groups.reserve_geometric(c, 1)?;
        c.reserve_vector_geometric(&mut self.payloads.values, &mut self.payloads.allocation, 1)?;
        c.admit_atomic_layout(mem::size_of::<RetainedPayload>())?;
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
                original: computation.original().map(|edge| edge.group),
                incoming: None,
                trigger: evidence.native.current,
                computation: Some(computation),
            },
        )?;
        let index = evidence.native.groups.len();
        let reference = evidence.group_ref(index, c)?;
        evidence.native.groups.try_push(
            c,
            OriginGroup {
                body: GroupBody::Leaf {
                    start: evidence.native.origins.len(),
                    length: 0,
                    value: if units == 0 {
                        LeafValue::Empty
                    } else {
                        LeafValue::Pending
                    },
                },
                units,
                cause,
                retirement: None,
            },
        )?;
        Ok(reference)
    }
}
impl ParseEvidence {
    /// Constant observation after original-control qualification at report prep.
    pub(super) fn retained_cause<'a>(
        &'a self,
        reference: &ConstructionCauseRef,
    ) -> Option<&'a ConstructionCause> {
        if reference.issuer.as_ptr() != Arc::as_ptr(&self.identity) {
            return None;
        }
        self.native.causes.as_slice().get(reference.index)
    }
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
                computation: None,
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
        match self.origin_evidence(c) {
            Ok(_) => (),
            Err(error) => {
                origins.release(c)?;
                value.release(c)?;
                return Err(error);
            }
        };
        let evidence = self.evidence.as_mut().ok_or(SessionError::WrongToken)?;
        let result = (|| {
            origins.require_owner(c)?;
            let identical = if let Some(original) = &original {
                evidence.group_value_equal(
                    c,
                    &self.payloads,
                    value.text().code_units(),
                    original,
                )? && evidence.origin_rows_equal(c, original, &origins)?
            } else {
                false
            };
            let cause = original
                .as_ref()
                .map(|original| evidence.replacement_cause(c, original))
                .transpose()?;
            if identical {
                let original = original.as_ref().ok_or(SessionError::WrongToken)?;
                let index = evidence.native.groups.len();
                let reference = evidence.group_ref(index, c)?;
                evidence.native.groups.reserve_geometric(c, 1)?;
                evidence.native.groups.try_push(
                    c,
                    OriginGroup {
                        body: GroupBody::Alias {
                            original: original.index,
                        },
                        units: value.text().len(),
                        cause: cause.ok_or(SessionError::WrongToken)?,
                        retirement: None,
                    },
                )?;
                Ok(reference)
            } else {
                evidence.retain_group_parts(c, &mut origins, value.text().len(), true, cause)
            }
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

/// Selected authored attribute field in the actual retained container.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeAttributeField {
    Name,
    Value,
}

/// Read-only completed evidence with the same execution controller. A failed
/// projection has no lender: its admitted suffix remains cleanup-only.
#[cfg(test)]
pub(crate) struct NativeAssociations<'a> {
    storage: &'a SessionStorage,
    evidence: &'a ParseEvidence,
    control: &'a mut ParseControl,
}
#[cfg(test)]
impl NativeAssociations<'_> {
    #[cfg(test)]
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
    /// Fixed immutable history header; output expansion is a separate charged
    /// operation on this same controller and evidence issuer.
    #[cfg(test)]
    pub(crate) fn group(&mut self, index: usize) -> Result<Option<&OriginGroup>, SessionError> {
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        self.control.charge(
            WorkCategory::InspectByte,
            mem::size_of::<OriginGroup>() as u64,
        )?;
        Ok(self.evidence.native.groups.as_slice().get(index))
    }
    #[cfg(test)]
    pub(crate) fn group_origin_cursor(
        &mut self,
        index: usize,
    ) -> Result<GroupOriginCursor, SessionError> {
        let reference = self.evidence.group_ref(index, self.control)?;
        GroupOriginCursor::new(self.evidence, &reference, self.control)
    }
    #[cfg(test)]
    pub(crate) fn next_group_origin(
        &mut self,
        cursor: &mut GroupOriginCursor,
    ) -> Result<Option<TextOrigin>, SessionError> {
        cursor.next(self.evidence, self.control)
    }
    #[cfg(test)]
    pub(crate) fn release_group_origin_cursor(
        &mut self,
        slot: &mut Option<GroupOriginCursor>,
    ) -> Result<(), SessionError> {
        // Keep the caller's real owner through every fallible refusal. The
        // cursor's same-controller cleanup completes before this fixed take;
        // a foreign lender therefore leaves its frames available for retry.
        slot.as_mut()
            .ok_or(SessionError::Consumed)?
            .release(self.control)?;
        drop(slot.take());
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn materialize_group(&mut self, index: usize) -> Result<FrozenUtf16, SessionError> {
        let reference = self.evidence.group_ref(index, self.control)?;
        self.evidence
            .materialize_group(self.control, &self.storage.payloads, &reference)
    }
    #[cfg(test)]
    pub(crate) fn release_materialized_group(
        &mut self,
        value: &mut Option<FrozenUtf16>,
    ) -> Result<(), SessionError> {
        ParseEvidence::release_materialized_group(self.control, value)
    }
    #[cfg(test)]
    pub(crate) fn cause_at(
        &mut self,
        index: usize,
    ) -> Result<Option<&ConstructionCause>, SessionError> {
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self.evidence.native.causes.as_slice().get(index))
    }
    #[cfg(test)]
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
    #[cfg(test)]
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
    #[cfg(test)]
    /// Match an actual retained attribute-container field to its native owner.
    /// Container indices are payload identities, never native node positions.
    pub(crate) fn attribute_for(
        &mut self,
        index: usize,
        node: &ParserNode,
        slot: usize,
        field: NativeAttributeField,
    ) -> Result<bool, SessionError> {
        let node = self.storage.index(self.control, node)?;
        self.control.charge(
            WorkCategory::InspectByte,
            mem::size_of::<NativeAssociation>() as u64,
        )?;
        let Some((row, _)) = self.row(index)? else {
            return Ok(false);
        };
        let (container, actual_slot) = match (row.destination, field) {
            (
                NativeDestination::Field(StringLocation::AttributeName(container, slot)),
                NativeAttributeField::Name,
            )
            | (
                NativeDestination::Field(StringLocation::AttributeValue(container, slot)),
                NativeAttributeField::Value,
            ) => (container, slot),
            _ => return Ok(false),
        };
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        self.control.charge(
            WorkCategory::InspectByte,
            mem::size_of::<RetainedPayload>() as u64,
        )?;
        let Some(RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
            element: Some(owner),
            values,
            ..
        }))) = self.storage.payloads.values.get(container)
        else {
            return Ok(false);
        };
        if *owner != node || actual_slot != slot {
            return Ok(false);
        }
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        self.control.charge(
            WorkCategory::InspectByte,
            mem::size_of::<Attribute>() as u64,
        )?;
        Ok(values.as_slice().get(actual_slot).is_some())
    }
    #[cfg(test)]
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
        if index >= self.evidence.native.native_rows {
            return Ok(None);
        }
        let Some(row) = rows.as_slice().get(index) else {
            return Ok(None);
        };
        Ok(Some(self.evidence.cause_ref(row.cause, self.control)?))
    }
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.evidence.native.native_rows
    }
    #[cfg(test)]
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
        if index >= self.evidence.native.native_rows {
            return Ok(None);
        }
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
    #[cfg(test)]
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
        lineage: (usize, usize),
    ) -> Result<(), SessionError> {
        let (cause, group) = lineage;
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
            .reserve_geometric(c, 1)?;
        self.native
            .projection
            .as_mut()
            .ok_or(SessionError::WrongToken)?
            .try_push(
                c,
                NativeAssociation {
                    group: Some(group),
                    relation: AssociationRelation::Output,
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
        let cause = group.cause;
        let mut cursor = GroupOriginCursor::new(self, reference, c)?;
        let read = (|| {
            let mut covered = 0;
            while let Some(origin) = cursor.next(self, c)? {
                if origin.start < covered || origin.end <= origin.start || origin.end > units {
                    return Err(SessionError::WrongToken);
                }
                if origin.start > covered {
                    self.projection_row(
                        c,
                        location,
                        covered..origin.start,
                        None,
                        None,
                        (cause, reference.index),
                    )?;
                }
                c.require_source_range(&origin.source)?;
                self.projection_row(
                    c,
                    location,
                    origin.start..origin.end,
                    Some(&origin.source),
                    Some(origin.kind),
                    (cause, reference.index),
                )?;
                covered = origin.end;
            }
            if covered < units || units == 0 {
                self.projection_row(
                    c,
                    location,
                    covered..units,
                    None,
                    None,
                    (cause, reference.index),
                )?;
            }
            Ok(())
        })();
        cursor.release(c)?;
        read
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
        storage.payloads.check_aliases(
            ConstructionView::Active(storage.construction.storage(c)?),
            c,
        )?;
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
        // Successful completed projection certifies every real history leaf,
        // including retired versions that are no longer a native destination.
        for group in evidence.native.groups.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(
                WorkCategory::InspectByte,
                mem::size_of::<OriginGroup>() as u64,
            )?;
            if let GroupBody::Leaf { value, .. } = group.body {
                match value {
                    LeafValue::Pending => return Err(SessionError::WrongToken),
                    LeafValue::Empty if group.units == 0 => {}
                    LeafValue::Empty => return Err(SessionError::WrongToken),
                    LeafValue::Payload(slot) => {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        let RetainedPayload::String {
                            value, allocation, ..
                        } = storage
                            .payloads
                            .values
                            .get(slot)
                            .ok_or(SessionError::WrongToken)?
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        c.require_document_string(allocation, value)?;
                        if value.len() != group.units {
                            return Err(SessionError::WrongToken);
                        }
                    }
                }
            }
        }
        if evidence.native.projection.is_some() {
            return Err(SessionError::WrongToken);
        }
        let mut rows = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        rows.require_owner(c)?;
        payload_write::<Option<OwnedVec<NativeAssociation>>>(c, 1)?;
        evidence.native.projection = Some(rows);
        for (payload_slot, payload) in storage.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let raw = match payload {
                RetainedPayload::Nonce {
                    node: Some(node),
                    raw,
                } => Some((*node, *raw, RawFieldKind::Nonce)),
                RetainedPayload::Input {
                    node: Some(node),
                    raw,
                    complete: true,
                    ..
                } => Some((*node, *raw, RawFieldKind::Input)),
                RetainedPayload::Textarea {
                    node: Some(node),
                    state,
                } => Some((*node, state.raw, RawFieldKind::Textarea)),
                _ => None,
            };
            if let Some((node, slot, kind)) = raw {
                if node >= storage.construction.storage(c)?.len() {
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
                if missing.is_some() {
                    evidence.bind_leaf_value(c, group, slot)?;
                }
                evidence.validate_group_value(c, &storage.payloads, value.code_units(), group)?;
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
                    || *node >= storage.construction.storage(c)?.len()
                {
                    return Err(SessionError::WrongToken);
                }
                payload_write::<NativeAssociation>(c, 1)?;
                evidence
                    .native
                    .projection
                    .as_mut()
                    .ok_or(SessionError::WrongToken)?
                    .reserve_geometric(c, 1)?;
                evidence
                    .native
                    .projection
                    .as_mut()
                    .ok_or(SessionError::WrongToken)?
                    .try_push(
                        c,
                        NativeAssociation {
                            group: None,
                            relation: AssociationRelation::Creation,
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
            let Some(native) = location.value(
                ConstructionView::Active(storage.construction.storage(c)?),
                &storage.payloads.values,
                payload_slot,
                c,
            )?
            else {
                continue;
            };
            c.charge(WorkCategory::ProviderCall, 1)?;
            if !Arc::ptr_eq(native.shared_code_units(), value.shared_code_units()) {
                return Err(SessionError::AliasedPayload);
            }
            if let Some(group) = origins {
                evidence.validate_group_value(c, &storage.payloads, native.code_units(), group)?;
                evidence.validate_field(c, native.code_units(), group)?;
                evidence.project_field(
                    c,
                    NativeDestination::Field(*location),
                    value.len(),
                    group,
                )?;
            } else {
                let group = evidence.source_independent_group(c, value.len())?;
                evidence.bind_leaf_value(c, &group, payload_slot)?;
                evidence.validate_group_value(c, &storage.payloads, value.code_units(), &group)?;
                evidence.project_field(
                    c,
                    NativeDestination::Field(*location),
                    value.len(),
                    &group,
                )?;
            }
        }
        payload_write::<usize>(c, 1)?;
        evidence.native.native_rows = evidence
            .native
            .projection
            .as_ref()
            .ok_or(SessionError::WrongToken)?
            .len();
        evidence.project_lineage(c)?;
        evidence.project_causes(c)?;
        evidence.project_lexical(c)?;
        c.checkpoint()?;
        payload_write::<bool>(c, 1)?;
        evidence.native.projection_complete = true;
        Ok(())
    }
    #[cfg(test)]
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
pub enum CapturedCauseRole {
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
                computation: None,
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
        self.native.groups.reserve_geometric(c, 1)?;
        self.native.groups.try_push(
            c,
            OriginGroup {
                body: GroupBody::Leaf {
                    start: self.native.origins.len(),
                    length: 0,
                    value: if actual.is_empty() {
                        LeafValue::Empty
                    } else {
                        LeafValue::Pending
                    },
                },
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
        if source >= self.construction.storage(c)?.len() {
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
                computation: None,
            },
        )?;
        payload_write::<Option<usize>>(c, 1)?;
        evidence.native.current = Some(index);
        Ok(previous)
    }
    /// Actual private attribute mutation, not a fabricated parser token/rule.
    pub(super) fn begin_attribute_change(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        source: Option<&crate::HtmlSourceRange>,
    ) -> Result<Option<ConstructionCauseRef>, SessionError> {
        if let Some(source) = source {
            c.require_source_range(source)?;
        }
        let Some(evidence) = &mut self.evidence else {
            return Ok(None);
        };
        let previous = evidence.current_cause(c)?;
        payload_write::<Option<crate::HtmlSourceRange>>(c, 1)?;
        let index = evidence.append_cause(
            c,
            ConstructionCause {
                operation: ConstructionOperation::Replacement,
                source: source.cloned(),
                captured: None,
                rule: None,
                family: None,
                original_cause: None,
                retired_destination: None,
                original_node: Some(node),
                intended_parent: None,
                original: None,
                incoming: None,
                trigger: evidence.native.current,
                computation: None,
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
    fn cause_handle_copy_keeps_issuer_index_and_rejects_foreign_cancelled_and_short_work() {
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
        let index = evidence.field_cause(&mut c).unwrap();
        let original = evidence.cause_ref(index, &mut c).unwrap();
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let copied = evidence.copy_cause_ref(&mut c, &original).unwrap();
        let cost = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
        assert_eq!(copied.index, index);
        assert_eq!(copied.issuer.as_ptr(), original.issuer.as_ptr());
        assert_eq!(evidence.native.causes.len(), 1);
        let other = ParseEvidence::prepare(&mut c).unwrap();
        assert!(matches!(
            other.copy_cause_ref(&mut c, &original),
            Err(SessionError::WrongToken)
        ));
        drop(copied);
        drop(original);
        other.release(&mut c).unwrap();
        evidence.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
        for cancelled in [false, true] {
            let cancel = crate::HtmlParseCancellation::new();
            let selected = crate::parser::tree_builder::tests::limits(100)
                .into_builder()
                .work_credits(before + cost - 1)
                .finish()
                .unwrap();
            let mut c = ParseControl::new(
                if cancelled {
                    crate::parser::tree_builder::tests::limits(100)
                } else {
                    selected
                },
                cancel.clone(),
            );
            let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
            let index = evidence.field_cause(&mut c).unwrap();
            let original = evidence.cause_ref(index, &mut c).unwrap();
            if cancelled {
                cancel.cancel();
            }
            assert!(matches!(
                evidence.copy_cause_ref(&mut c, &original),
                Err(SessionError::Stop(_))
            ));
            assert_eq!(evidence.native.causes.len(), 1);
            drop(original);
            evidence.release(&mut c).unwrap();
            crate::parser::tree_builder::tests::assert_released(&c);
        }
    }
    #[test]
    fn captured_namespace_and_bootstrap_keep_actual_subject_revision_and_nonlexical_origin() {
        let revision = HtmlHostFactsRevision::fresh();
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(2, 4, 0, 1000, 10, 0, 0, 16_777_216),
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
        assert!(
            evidence
                .observed_origins(&group, &mut c)
                .unwrap()
                .is_empty()
        );
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
        let fixed_leaf = |reference| {
            let GroupBody::Leaf { start, length, .. } =
                evidence.require_group(reference).unwrap().body
            else {
                panic!("original leaf");
            };
            &evidence.native.origins.as_slice()[start..start + length]
        };
        let left = fixed_leaf(old);
        let right = fixed_leaf(incoming);
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
    fn merge_cause_entry_refusal_preserves_original_leaves_and_capacity() {
        // Two atoms + two leaf headers + one original cause = five entries.
        // The merge cause needs the sixth; no copied prefix maps exist.
        let selected = crate::parser::tree_builder::tests::limits(100)
            .into_builder()
            .source_map_entries(5)
            .finish()
            .unwrap();
        let (mut c, mut evidence, old, incoming) = inputs(selected, HtmlParseCancellation::new());
        let before = c.usage().clone();
        assert!(matches!(
            evidence.merge_groups(&mut c, &old, &incoming, 1, 2),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::SourceMapEntries,
                limit: 5,
                current: 5,
                requested_delta: 1
            }))
        ));
        let first = c.stop();
        assert_original_groups(&evidence, &old, &incoming);
        assert_eq!(
            c.usage().resource(HtmlParseResource::SourceMapEntries),
            before.resource(HtmlParseResource::SourceMapEntries)
        );
        // Reservation may admit spare capacity before the logical cause cap;
        // retained capacity is honest and reclaimed with the evidence owner.
        assert!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
                >= before
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
    fn merge_cause_work_refusal_keeps_original_atoms_without_prefix_scratch() {
        let limits = crate::parser::tree_builder::tests::limits(100);
        let (mut measured, evidence, old, incoming) = inputs(limits, HtmlParseCancellation::new());
        let base = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        drop((old, incoming));
        evidence.release(&mut measured).unwrap();
        // Two actual group visits precede the fixed cause copy/write. One
        // credit short of those visits denies the second visit atomically.
        let selected = limits
            .into_builder()
            .work_credits(base + 1)
            .finish()
            .unwrap();
        let (mut c, mut evidence, old, incoming) = inputs(selected, HtmlParseCancellation::new());
        assert!(
            matches!(evidence.merge_groups(&mut c,&old,&incoming,1,2),Err(SessionError::Stop(ParseStop::LimitExceeded{resource:HtmlParseResource::WorkCredits,limit,current,requested_delta:2})) if limit==base+1 && current==base)
        );
        let first = c.stop();
        assert_original_groups(&evidence, &old, &incoming);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            5
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
            .source_map_entries(7)
            .finish()
            .unwrap();
        let (mut c, mut evidence, old, incoming) = inputs(selected, HtmlParseCancellation::new());
        let merged = evidence
            .merge_groups(&mut c, &old, &incoming, 1, 2)
            .unwrap();
        let maps = evidence.observed_origins(&merged, &mut c).unwrap();
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
            (2, 3, 2)
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            7
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

// Generated input digits have an actual value/group, and deliberately no
// lexical digit origins. Copy that complete field, retaining exactly the
// atoms actually present instead of requiring invented lexical coverage.
pub(super) fn copy_generated_input_value(
    payloads: &PayloadStorage,
    evidence: Option<&ParseEvidence>,
    c: &mut ParseControl,
    slot: usize,
) -> Result<FieldCopy, SessionError> {
    let value = admitted_input_string(payloads, c, slot)?;
    let RetainedPayload::String {
        origins: Some(group),
        ..
    } = &payloads.values[slot]
    else {
        return Err(SessionError::WrongToken);
    };
    let evidence = evidence.ok_or(SessionError::WrongToken)?;
    evidence.qualify_generated_copy(c, payloads, group, value.code_units())?;
    let mut buffer = Utf16Buffer::new(c)?;
    let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
    let result = (|| {
        buffer.extend_units(value.code_units(), c)?;
        let mut cursor = provenance::GroupOriginCursor::new(evidence, group, c)?;
        let read = (|| {
            while let Some(origin) = cursor.next(evidence, c)? {
                c.require_source_range(&origin.source)?;
                origins.try_push(c, origin)?;
            }
            Ok::<_, SessionError>(())
        })();
        cursor.release(c)?;
        read?;
        evidence.copy_group_reference(c, group)
    })();
    let original = match result {
        Ok(group) => group,
        Err(error) => {
            buffer.release(c)?;
            origins.release(c)?;
            return Err(error);
        }
    };
    match buffer.freeze(c) {
        Ok(value) => Ok(FieldCopy {
            value,
            origins,
            original: Some(original),
        }),
        Err(error) => {
            origins.release(c)?;
            Err(error.into())
        }
    }
}
