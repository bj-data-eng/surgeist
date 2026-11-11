//! Native field facts owned by the source evidence issuer, not another tree.

use super::*;
use crate::keyed::{Table, slots};
#[cfg(test)]
use surgeist_html::values::DocumentString;
#[path = "lexical.rs"]
mod lexical;
#[path = "reverse.rs"]
pub(crate) mod reverse;
use crate::parser::{
    control::HtmlParseInvariant,
    references::{ReferenceValue, decode_spelling},
    tokenizer::token::HtmlParseOriginKind,
};
pub use lexical::{
    HtmlParseLexicalDisposition, HtmlParseLexicalKind, HtmlParseLexicalRole, HtmlParseOmissionKind,
};
pub(crate) use lexical::{LexicalOccurrence, LexicalOmission, LexicalPart, LexicalRef};
pub(crate) use reverse::{AssociationView, HtmlParseAssociationRelation};

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
#[cfg(test)]
impl ConstructionCauseRef {
    /// Names a cause the evidence does not hold.
    pub(super) fn corrupt_for_test(&mut self) {
        self.index = usize::MAX;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParserRule {
    Prologue(crate::parser::tree_builder::prologue::HtmlPrologueMode),
    Tree(crate::parser::tree_builder::head::HtmlInsertionMode),
    Foreign,
}
pub(crate) struct ParserTrigger<'a> {
    pub(crate) source: &'a crate::HtmlSourceRange,
    pub(crate) rule: HtmlParserRule,
    pub(crate) family: evidence::HtmlParseTokenFamily,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseConstructionOperation {
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
pub enum HtmlParseRangeBranch {
    InvalidDefaultMidpoint,
    InvalidDefaultMinimum,
    Underflow,
    Overflow,
    StepCorrection,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseRangeOperandRole {
    Minimum,
    Maximum,
    Step,
    ValueContent,
}
slots!(HtmlParseRangeOperandRole, RANGE_OPERAND_COUNT, {
    Minimum => 0,
    Maximum => 1,
    Step => 2,
    ValueContent => 3,
});
const RANGE_OPERAND_COUNT: usize = 4;
/// One value per [`HtmlParseRangeOperandRole`]; a lookup cannot be out of range.
pub(crate) type RangeOperandTable<T> = Table<HtmlParseRangeOperandRole, T, RANGE_OPERAND_COUNT>;
impl HtmlParseRangeOperandRole {
    pub(crate) fn index(self) -> usize {
        self as usize
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseRangeOperandKind {
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
impl RangeEdge {
    /// The edge of a parse that records no graph: it names no group or cause,
    /// and the computation carrying it is dropped before anything stores it.
    pub(crate) const UNRECORDED: Self = Self {
        group: usize::MAX,
        cause: usize::MAX,
    };
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
    pub(crate) fn kind(self) -> HtmlParseRangeOperandKind {
        match self {
            Self::NotConsulted => HtmlParseRangeOperandKind::NotConsulted,
            Self::Absent => HtmlParseRangeOperandKind::Absent,
            Self::ConversionFailed(_) => HtmlParseRangeOperandKind::ConversionFailed,
            Self::Finite(_) => HtmlParseRangeOperandKind::Finite,
            Self::Any(_) => HtmlParseRangeOperandKind::Any,
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
    pub(crate) branch: HtmlParseRangeBranch,
    pub(crate) original: RangeOriginal,
    pub(crate) operands: RangeOperandTable<RangeOperand>,
    pub(crate) minimum:
        Option<surgeist_html::attributes::input_bounds::HtmlInputNumericBoundOrigin>,
    pub(crate) maximum:
        Option<surgeist_html::attributes::input_bounds::HtmlInputNumericBoundOrigin>,
    pub(crate) step: Option<surgeist_html::attributes::input_bounds::HtmlInputNumericStepOrigin>,
    pub(crate) base:
        Option<surgeist_html::attributes::input_bounds::HtmlInputNumericStepBaseOrigin>,
}
impl RangeComputation {
    pub(crate) fn edges(self) -> [Option<RangeEdge>; 5] {
        [
            match self.original {
                RangeOriginal::FreshEmpty => None,
                RangeOriginal::Group(edge) => Some(edge),
            },
            self.operands.get(HtmlParseRangeOperandRole::Minimum).edge(),
            self.operands.get(HtmlParseRangeOperandRole::Maximum).edge(),
            self.operands.get(HtmlParseRangeOperandRole::Step).edge(),
            self.operands
                .get(HtmlParseRangeOperandRole::ValueContent)
                .edge(),
        ]
    }
}

/// The actual source selected by the Color update algorithm. A missing
/// installed default is distinct from an installed empty string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(any(feature = "detail", test)), allow(clippy::enum_variant_names))]
pub enum HtmlParseColorValueSource {
    MissingInstalledValue,
    InstalledValue,
    CurrentValue,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseColorOperandRole {
    Alpha,
    Colorspace,
}
slots!(HtmlParseColorOperandRole, COLOR_OPERAND_COUNT, {
    Alpha => 0,
    Colorspace => 1,
});
const COLOR_OPERAND_COUNT: usize = 2;
/// One value per [`HtmlParseColorOperandRole`]; a lookup cannot be out of range.
pub(crate) type ColorOperandTable<T> = Table<HtmlParseColorOperandRole, T, COLOR_OPERAND_COUNT>;
impl HtmlParseColorOperandRole {
    pub(crate) fn index(self) -> usize {
        self as usize
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct ColorComputation {
    pub(crate) source: HtmlParseColorValueSource,
    pub(crate) original: Option<RangeEdge>,
    pub(crate) operands: ColorOperandTable<Option<RangeEdge>>,
    pub(crate) space: surgeist_html::attributes::input_colorspace::HtmlInputColorSpace,
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
                value.operands.copy(HtmlParseColorOperandRole::Alpha),
                value.operands.copy(HtmlParseColorOperandRole::Colorspace),
                None,
                None,
            ],
        }
    }
}

/// The subject a copy cause names: an origin group duplicated into an alias,
/// or the node whose clone owns the copied fields.
#[derive(Clone, Copy)]
pub(crate) enum CopySubject {
    Group(usize),
    Node(usize),
}
/// What a replacement cause replaced, by the operation that issued it.
pub(crate) enum ReplacementSubject {
    /// A field reset to the group `incoming`, after the optional `original`.
    Reset {
        original: Option<usize>,
        incoming: usize,
    },
    /// A generated field replacing the group `original`.
    Rewrite { original: usize },
    /// A private attribute mutation of one node, optionally from a source range.
    AttributeChange {
        node: usize,
        source: Option<crate::HtmlSourceRange>,
    },
    /// A generated form-control value, with the decisions that produced it.
    Computed(InputValueComputation),
}
/// Why a construction fact exists. Each variant carries exactly the fields its
/// operation owns, so a field belonging to another operation cannot be set.
/// `trigger` names the cause that was current when this one was issued.
pub(crate) enum ConstructionCause {
    SourceIndependent,
    Request,
    /// The source range is recorded when the occurrence is classified.
    Lexical {
        source: Option<crate::HtmlSourceRange>,
    },
    Token {
        source: crate::HtmlSourceRange,
        rule: HtmlParserRule,
        family: evidence::HtmlParseTokenFamily,
        trigger: Option<usize>,
    },
    Merge {
        original: usize,
        incoming: usize,
        trigger: Option<usize>,
    },
    Copy {
        subject: CopySubject,
        trigger: Option<usize>,
    },
    Replacement {
        subject: ReplacementSubject,
        trigger: Option<usize>,
    },
    Retirement {
        group: usize,
        destination: FieldRef,
        trigger: Option<usize>,
    },
    Replay {
        original_cause: Option<usize>,
        intended_parent: Option<usize>,
        trigger: Option<usize>,
    },
    Captured {
        context: CapturedConstructionContext,
        trigger: Option<usize>,
    },
}
impl ConstructionCause {
    pub(crate) fn operation(&self) -> HtmlParseConstructionOperation {
        match self {
            Self::SourceIndependent => HtmlParseConstructionOperation::SourceIndependent,
            Self::Request => HtmlParseConstructionOperation::Request,
            Self::Lexical { .. } => HtmlParseConstructionOperation::Lexical,
            Self::Token { .. } => HtmlParseConstructionOperation::Token,
            Self::Merge { .. } => HtmlParseConstructionOperation::Merge,
            Self::Copy { .. } => HtmlParseConstructionOperation::Copy,
            Self::Replacement { .. } => HtmlParseConstructionOperation::Replacement,
            Self::Retirement { .. } => HtmlParseConstructionOperation::Retirement,
            Self::Replay { .. } => HtmlParseConstructionOperation::Replay,
            Self::Captured { .. } => HtmlParseConstructionOperation::Captured,
        }
    }
    pub(crate) fn source(&self) -> Option<&crate::HtmlSourceRange> {
        match self {
            Self::Lexical { source } => source.as_ref(),
            Self::Token { source, .. } => Some(source),
            Self::Replacement {
                subject: ReplacementSubject::AttributeChange { source, .. },
                ..
            } => source.as_ref(),
            _ => None,
        }
    }
    pub(crate) fn rule(&self) -> Option<HtmlParserRule> {
        match self {
            Self::Token { rule, .. } => Some(*rule),
            _ => None,
        }
    }
    pub(crate) fn family(&self) -> Option<evidence::HtmlParseTokenFamily> {
        match self {
            Self::Token { family, .. } => Some(*family),
            _ => None,
        }
    }
    pub(crate) fn captured(&self) -> Option<&CapturedConstructionContext> {
        match self {
            Self::Captured { context, .. } => Some(context),
            _ => None,
        }
    }
    pub(crate) fn computation(&self) -> Option<&InputValueComputation> {
        match self {
            Self::Replacement {
                subject: ReplacementSubject::Computed(computation),
                ..
            } => Some(computation),
            _ => None,
        }
    }
    /// Lets a test corrupt the decisions of a published computation.
    #[cfg(test)]
    pub(crate) fn computation_mut(&mut self) -> Option<&mut InputValueComputation> {
        match self {
            Self::Replacement {
                subject: ReplacementSubject::Computed(computation),
                ..
            } => Some(computation),
            _ => None,
        }
    }
    pub(crate) fn retired_destination(&self) -> Option<FieldRef> {
        match self {
            Self::Retirement { destination, .. } => Some(*destination),
            _ => None,
        }
    }
    pub(crate) fn original_cause(&self) -> Option<usize> {
        match self {
            Self::Replay { original_cause, .. } => *original_cause,
            _ => None,
        }
    }
    pub(crate) fn intended_parent(&self) -> Option<usize> {
        match self {
            Self::Replay {
                intended_parent, ..
            } => *intended_parent,
            _ => None,
        }
    }
    pub(crate) fn original_node(&self) -> Option<usize> {
        match self {
            Self::Copy {
                subject: CopySubject::Node(node),
                ..
            }
            | Self::Replacement {
                subject: ReplacementSubject::AttributeChange { node, .. },
                ..
            } => Some(*node),
            _ => None,
        }
    }
    /// The group this cause replaced, merged, copied or retired.
    pub(crate) fn original(&self) -> Option<usize> {
        match self {
            Self::Merge { original, .. }
            | Self::Copy {
                subject: CopySubject::Group(original),
                ..
            }
            | Self::Replacement {
                subject: ReplacementSubject::Rewrite { original },
                ..
            }
            | Self::Retirement {
                group: original, ..
            } => Some(*original),
            Self::Replacement {
                subject: ReplacementSubject::Reset { original, .. },
                ..
            } => *original,
            Self::Replacement {
                subject: ReplacementSubject::Computed(computation),
                ..
            } => computation.original().map(|edge| edge.group),
            _ => None,
        }
    }
    pub(crate) fn incoming(&self) -> Option<usize> {
        match self {
            Self::Merge { incoming, .. }
            | Self::Replacement {
                subject: ReplacementSubject::Reset { incoming, .. },
                ..
            } => Some(*incoming),
            _ => None,
        }
    }
    /// The cause current at issue time, including a token's chronology.
    pub(crate) fn trigger(&self) -> Option<usize> {
        match self {
            Self::SourceIndependent | Self::Request | Self::Lexical { .. } => None,
            Self::Token { trigger, .. }
            | Self::Merge { trigger, .. }
            | Self::Copy { trigger, .. }
            | Self::Replacement { trigger, .. }
            | Self::Retirement { trigger, .. }
            | Self::Replay { trigger, .. }
            | Self::Captured { trigger, .. } => *trigger,
        }
    }
    /// The trigger that lineage follows. A token's chronological trigger,
    /// a retirement's and a capture's stay in the fixed history only.
    pub(crate) fn semantic_trigger(&self) -> Option<usize> {
        match self {
            Self::Merge { trigger, .. }
            | Self::Copy { trigger, .. }
            | Self::Replay { trigger, .. }
            | Self::Replacement { trigger, .. } => *trigger,
            _ => None,
        }
    }
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
    pub(super) origins: &'a [HtmlParseTextOrigin],
    pub(super) groups: &'a [OriginGroup],
    pub(super) causes: &'a [ConstructionCause],
}
#[derive(Clone, Copy)]
pub(crate) enum LeafValue {
    Pending,
    /// The string of the registry row at this slot.
    Payload(usize),
    /// The units `offset..` of the string of a non-element field of a record. The leaf lies in
    /// the string for as long as the record holds it: a text only grows at its end, and the
    /// strings of the other fields never change.
    Field {
        string: RecordString,
        offset: usize,
    },
    Empty,
}

/// Where the strings of leaves are read: the registry rows that hold the strings of some
/// fields, and the records that hold the strings of the others. A reader of values that only
/// registry rows hold, such as the values of input elements, has no records: a leaf in a record
/// is then the wrong token.
#[derive(Clone, Copy)]
pub(crate) struct LeafStrings<'a> {
    pub(super) payloads: &'a PayloadStorage,
    pub(super) records: Option<ConstructionView<'a>>,
}
impl<'a> LeafStrings<'a> {
    /// Leaves whose strings registry rows hold.
    pub(super) fn rows(payloads: &'a PayloadStorage) -> Self {
        Self {
            payloads,
            records: None,
        }
    }
    /// Leaves whose strings registry rows or the records of `records` hold.
    pub(super) fn with_records(
        payloads: &'a PayloadStorage,
        records: ConstructionView<'a>,
    ) -> Self {
        Self {
            payloads,
            records: Some(records),
        }
    }
    /// The units of a leaf of `units` units whose value is `value`. A leaf whose string is not
    /// bound, or has other units than the leaf covers, is the wrong token.
    pub(crate) fn units(
        self,
        c: &mut ParseControl,
        value: LeafValue,
        units: usize,
    ) -> Result<&'a [u16], SessionError> {
        match value {
            LeafValue::Pending => Err(SessionError::WrongToken),
            LeafValue::Empty if units == 0 => Ok(&[]),
            LeafValue::Empty => Err(SessionError::WrongToken),
            LeafValue::Payload(slot) => {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let RetainedPayload::String {
                    value, allocation, ..
                } = self
                    .payloads
                    .values
                    .get(slot)
                    .ok_or(SessionError::WrongToken)?
                else {
                    return Err(SessionError::WrongToken);
                };
                c.require_document_string(allocation, value)?;
                if value.len() != units {
                    return Err(SessionError::WrongToken);
                }
                Ok(value.code_units())
            }
            LeafValue::Field { string, offset } => {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let end = offset.checked_add(units).ok_or(SessionError::WrongToken)?;
                self.records
                    .and_then(|records| string.field().units(records))
                    .and_then(|string| string.get(offset..end))
                    .ok_or(SessionError::WrongToken)
            }
        }
    }
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
            c.charge(HtmlParseWorkCategory::ProviderCall, 2)
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
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        if self.issuer.as_ptr() != Arc::as_ptr(&evidence.identity) {
            return Err(c.fail_invariant(HtmlParseInvariant::ForeignEvidence).into());
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.charge(
                HtmlParseWorkCategory::InspectByte,
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
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    c.charge(
                        HtmlParseWorkCategory::InspectByte,
                        mem::size_of::<usize>() as u64,
                    )?;
                    if original >= frame.index
                        || evidence
                            .native
                            .groups
                            .as_slice()
                            .get(original)
                            .is_none_or(|original| original.units != group.units)
                    {
                        return Err(SessionError::WrongToken);
                    }
                    self.frames
                        .as_mut()
                        .ok_or(SessionError::Consumed)?
                        .try_push_geometric(
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
                    c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                    c.charge(
                        HtmlParseWorkCategory::InspectByte,
                        const { (2 * mem::size_of::<usize>()) as u64 },
                    )?;
                    if original >= frame.index || incoming >= frame.index {
                        return Err(SessionError::WrongToken);
                    }
                    let groups = evidence.native.groups.as_slice();
                    let (Some(left), Some(right)) = (groups.get(original), groups.get(incoming))
                    else {
                        return Err(SessionError::WrongToken);
                    };
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
    ) -> Result<Option<HtmlParseTextOrigin>, SessionError> {
        self.leaves.require(evidence, c)?;
        loop {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.charge(
                HtmlParseWorkCategory::InspectByte,
                const { (mem::size_of::<Option<LeafChunk>>() + mem::size_of::<usize>()) as u64 },
            )?;
            if let Some(leaf) = self.current
                && self.position < leaf.length
            {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                c.charge(
                    HtmlParseWorkCategory::InspectByte,
                    mem::size_of::<HtmlParseTextOrigin>() as u64,
                )?;
                // Leaf chunks are issued only for ranges inside the origins.
                let origin = leaf
                    .start
                    .checked_add(self.position)
                    .and_then(|position| evidence.native.origins.as_slice().get(position))
                    .ok_or_else(|| latch_registry_row(c))?;
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
                payload_write::<HtmlParseTextOrigin>(c, 1)?;
                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                payload_write::<usize>(c, 1)?;
                // `position < leaf.length` held on entry to this branch.
                self.position = self.position.saturating_add(1);
                return Ok(Some(HtmlParseTextOrigin {
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
    Field(FieldRef),
    Node(usize),
    Raw {
        node: usize,
        kind: HtmlParseRawField,
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
pub enum HtmlParseRawField {
    Nonce,
    Input,
    Textarea,
}
pub(crate) struct NativeAssociation {
    pub(crate) group: Option<usize>,
    pub(crate) relation: HtmlParseAssociationRelation,
    pub(crate) destination: NativeDestination,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) source: Option<crate::HtmlSourceRange>,
    pub(crate) kind: Option<HtmlParseOriginKind>,
    pub(crate) cause: usize,
}
/// Mutable production and immutable groups have one flat owner. Pending suffixes
/// have no group/projection proof and are reclaimed on the original stop.
pub(super) struct NativeProvenance {
    source: Option<crate::HtmlSource>,
    origins: OwnedVec<HtmlParseTextOrigin>,
    groups: OwnedVec<OriginGroup>,
    causes: OwnedVec<ConstructionCause>,
    current: Option<usize>,
    projection: Option<OwnedVec<NativeAssociation>>,
    projection_complete: bool,
    native_rows: usize,
    lexical: lexical::LexicalStorage,
}
impl NativeProvenance {
    pub(super) fn new(c: &mut ParseControl) -> Result<Self, HtmlParseStop> {
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
    pub(super) fn preflight_failed_disposal(
        &self,
        c: &mut ParseControl,
    ) -> Result<(), HtmlParseStop> {
        fn extent<T>(
            values: &OwnedVec<T>,
            c: &mut ParseControl,
        ) -> Result<(u64, u64), HtmlParseStop> {
            values.require_disposal_owner(c)?;
            let Some(bytes) = values
                .capacity()
                .checked_mul(mem::size_of::<T>())
                .and_then(|bytes| u64::try_from(bytes).ok())
            else {
                return Err(c.fail_invariant(HtmlParseInvariant::CapacityMismatch));
            };
            let Ok(units) = u64::try_from(values.len()) else {
                return Err(c.fail_invariant(HtmlParseInvariant::CapacityMismatch));
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
                return Err(c.fail_invariant(HtmlParseInvariant::CapacityMismatch));
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
            return Err(c.fail_invariant(HtmlParseInvariant::ReleaseUnderflow));
        }
        Ok(())
    }
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
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
        strings: LeafStrings<'_>,
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
                let text = strings.units(c, leaf.value, leaf.units)?;
                for index in 0..leaf.units {
                    c.charge(HtmlParseWorkCategory::InspectByte, 4)?;
                    let position = leaf.offset.checked_add(index).ok_or_else(|| {
                        c.stop_representation(HtmlParseResource::ManagedUtf16Units)
                    })?;
                    if value.get(position) != text.get(index) {
                        equal = false;
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
        strings: LeafStrings<'_>,
        value: &[u16],
        group: &OriginGroupRef,
    ) -> Result<(), SessionError> {
        if self.group_value_equal(c, strings, value, group)? {
            Ok(())
        } else {
            Err(SessionError::WrongToken)
        }
    }
    fn origin_rows_equal(
        &self,
        c: &mut ParseControl,
        group: &OriginGroupRef,
        rows: &OwnedVec<HtmlParseTextOrigin>,
    ) -> Result<bool, SessionError> {
        rows.require_disposal_owner(c)?;
        let mut cursor = GroupOriginCursor::new(self, group, c)?;
        let read = (|| {
            for expected in rows.as_slice() {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                c.charge(
                    HtmlParseWorkCategory::InspectByte,
                    mem::size_of::<HtmlParseTextOrigin>() as u64,
                )?;
                let Some(actual) = cursor.next(self, c)? else {
                    return Ok(false);
                };
                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
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
        strings: LeafStrings<'_>,
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
                let units = strings.units(c, leaf.value, leaf.units)?;
                output.extend_units(units, c)?;
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
        if value.text().block().holders() != 1 {
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
        // The source range, origins checked and origins owed of the reference
        // just checked.
        let mut reference_run: Option<((usize, usize), usize, usize)> = None;
        let read = (|| {
            while let Some(origin) = cursor.next(self, c)? {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                c.require_source_range(&origin.source)?;
                let output = value
                    .get(origin.start..origin.end)
                    .ok_or(SessionError::WrongToken)?;
                let begin = origin.source.start();
                let end = origin.source.end();
                if source.identity() != origin.source.source() {
                    return Err(SessionError::WrongToken);
                }
                if let Some((range, done, total)) = reference_run
                    && (origin.kind != HtmlParseOriginKind::CharacterReference
                        || range != (begin, end))
                {
                    if done != total {
                        return Err(SessionError::WrongToken);
                    }
                    reference_run = None;
                }
                match origin.kind {
                    HtmlParseOriginKind::Literal | HtmlParseOriginKind::AsciiLowercase => {
                        c.charge(
                            HtmlParseWorkCategory::InspectByte,
                            (origin.source.len() as u64)
                                .saturating_mul(if source.as_utf8().is_some() { 1 } else { 2 }),
                        )?;
                        let mut offset = 0;
                        let mut compare = |unit: u16| -> Result<(), SessionError> {
                            c.charge_unit_inspection()?;
                            let expected = if origin.kind == HtmlParseOriginKind::AsciiLowercase {
                                surgeist_html::bounded::lexical::ascii_lowercase(unit)
                            } else {
                                unit
                            };
                            if output.get(offset) != Some(&expected) {
                                return Err(SessionError::WrongToken);
                            }
                            // `output.get(offset)` just matched, so `offset` is
                            // below the output length.
                            offset = offset.saturating_add(1);
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
                    HtmlParseOriginKind::Newline => {
                        c.charge(HtmlParseWorkCategory::InspectByte, 6)?;
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
                    HtmlParseOriginKind::NullReplacement => {
                        c.charge(HtmlParseWorkCategory::InspectByte, 4)?;
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
                    // The source spelling of a character reference decodes through
                    // the pinned table and numeric rules the tokenizer applies;
                    // the output must be exactly the scalar it stands for. A
                    // two-scalar reference has one origin per scalar over the
                    // same source range, in order.
                    HtmlParseOriginKind::CharacterReference => {
                        c.charge(
                            HtmlParseWorkCategory::InspectByte,
                            (output.len() as u64).saturating_mul(2),
                        )?;
                        let position = match reference_run {
                            Some((_, done, _)) => done,
                            None => 0,
                        };
                        let spelling = if let Some(text) = source.as_utf8() {
                            let text = text.get(begin..end).ok_or(SessionError::WrongToken)?;
                            decode_spelling(text.chars().map(u32::from), c)?
                        } else {
                            let units = source
                                .as_utf16()
                                .and_then(|text| text.code_units().get(begin..end))
                                .ok_or(SessionError::WrongToken)?;
                            decode_spelling(units.iter().copied().map(u32::from), c)?
                        };
                        let (scalar, total) = match (spelling, position) {
                            (Some(ReferenceValue::One(scalar)), 0) => (scalar, 1),
                            (Some(ReferenceValue::Two(scalar, _)), 0) => (scalar, 2),
                            (Some(ReferenceValue::Two(_, scalar)), 1) => (scalar, 2),
                            _ => return Err(SessionError::WrongToken),
                        };
                        let mut pair = [0; 2];
                        if scalar.encode_utf16(&mut pair) != output {
                            return Err(SessionError::WrongToken);
                        }
                        reference_run = Some(((begin, end), position.saturating_add(1), total));
                        continue;
                    }
                    // These are producer transformations, not raw equality. The
                    // actual token/copy operation preserves its atomic output.
                    HtmlParseOriginKind::RecoverySyntax
                    | HtmlParseOriginKind::InputValueNormalization => {
                        c.charge(
                            HtmlParseWorkCategory::InspectByte,
                            (output.len() as u64).saturating_mul(2),
                        )?;
                        if output.is_empty() || begin == end {
                            return Err(SessionError::WrongToken);
                        }
                    }
                }
            }
            // A two-scalar reference whose second origin is absent is defective.
            match reference_run {
                Some((_, done, total)) if done != total => Err(SessionError::WrongToken),
                _ => Ok(()),
            }
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
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
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
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
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
    /// Releases the complete projection, as the loss of it would leave the
    /// evidence of a published report.
    #[cfg(test)]
    pub(super) fn drop_projection_for_test(&mut self, c: &mut ParseControl) {
        if let Some(rows) = self.native.projection.take() {
            rows.release(c).unwrap();
        }
    }
    /// Takes the source out, as a fault in a published report would.
    #[cfg(test)]
    pub(super) fn take_source_for_test(&mut self) -> Option<crate::HtmlSource> {
        self.native.source.take()
    }
    /// Puts back a source taken by [`Self::take_source_for_test`].
    #[cfg(test)]
    pub(super) fn restore_source_for_test(&mut self, source: crate::HtmlSource) {
        self.native.source = Some(source);
    }
    /// Whether the complete projection is still held.
    #[cfg(test)]
    pub(super) fn holds_projection_for_test(&self) -> bool {
        self.native.projection.is_some()
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
    /// The published construction causes, for tests that corrupt a report.
    #[cfg(test)]
    pub(super) fn causes_for_test(&mut self) -> &mut [ConstructionCause] {
        self.native.causes.as_mut_slice_for_test()
    }
    /// The published origin groups, for tests that corrupt a report.
    #[cfg(test)]
    pub(super) fn groups_for_test(&mut self) -> &mut [OriginGroup] {
        self.native.groups.as_mut_slice_for_test()
    }
    /// The published lexical occurrences, for tests that corrupt a report.
    #[cfg(test)]
    pub(super) fn lexical_occurrences_for_test(&mut self) -> &mut [LexicalOccurrence] {
        self.native.lexical.occurrences.as_mut_slice_for_test()
    }
    /// The published lexical parts, for tests that corrupt a report.
    #[cfg(test)]
    pub(super) fn lexical_parts_for_test(&mut self) -> &mut [LexicalPart] {
        self.native.lexical.parts.as_mut_slice_for_test()
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
    pub(super) fn observed_origins(
        &self,
        group: &OriginGroupRef,
        c: &mut ParseControl,
    ) -> Result<Vec<HtmlParseTextOrigin>, SessionError> {
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
        self.bind_leaf(c, group, LeafValue::Payload(slot))
    }
    /// Binds the leaf `group` to the units `offset..` of the string of `field`, a field of a
    /// record. Binding a group that is not a leaf, or a leaf that has no units, changes nothing.
    pub(super) fn bind_leaf_field(
        &mut self,
        c: &mut ParseControl,
        group: &OriginGroupRef,
        field: FieldRef,
        offset: usize,
    ) -> Result<(), SessionError> {
        // A field whose record position does not fit a leaf is not one that is bound.
        let string = RecordString::of(field).ok_or(SessionError::WrongToken)?;
        self.bind_leaf(c, group, LeafValue::Field { string, offset })
    }
    fn bind_leaf(
        &mut self,
        c: &mut ParseControl,
        group: &OriginGroupRef,
        bound: LeafValue,
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
                        bound
                    };
                }
                LeafValue::Empty => {}
                LeafValue::Payload(_) | LeafValue::Field { .. } => {
                    return Err(SessionError::WrongToken);
                }
            }
        }
        Ok(())
    }
    pub(super) fn copy_group(
        &mut self,
        c: &mut ParseControl,
        original: &OriginGroupRef,
    ) -> Result<OriginGroupRef, SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let group = self.require_group(original)?;
        let (units, original) = (group.units, original.index);
        let cause = self.append_cause(
            c,
            ConstructionCause::Copy {
                subject: CopySubject::Group(original),
                trigger: self.native.current,
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
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        if !self.records_graph() {
            self.native.source = Some(source.clone());
            return Ok(());
        }
        let cause = self.append_cause(c, ConstructionCause::Request)?;
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
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let index = self.append_cause(
            c,
            ConstructionCause::Token {
                source: trigger.source.clone(),
                rule: trigger.rule,
                family: trigger.family,
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
        self.append_cause(c, ConstructionCause::SourceIndependent)
    }
    pub(super) fn retain_group(
        &mut self,
        c: &mut ParseControl,
        mut input: OwnedVec<HtmlParseTextOrigin>,
        units: usize,
    ) -> Result<OriginGroupRef, SessionError> {
        let result = self.retain_group_parts(c, &mut input, units, true, None);
        input.release(c)?;
        result
    }
    fn retain_group_parts(
        &mut self,
        c: &mut ParseControl,
        input: &mut OwnedVec<HtmlParseTextOrigin>,
        units: usize,
        contiguous: bool,
        cause: Option<usize>,
    ) -> Result<OriginGroupRef, SessionError> {
        input.require_owner(c)?;
        let mut end = 0;
        for origin in input.as_slice() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        // The loop above appended exactly `length` entries after `start`.
        let end = self.native.origins.len();
        for (left, right) in (start..end).zip((start..end).rev()).take(length / 2) {
            self.native.origins.swap_slots(left, right, c)?;
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
        let cause = self.append_cause(c, ConstructionCause::SourceIndependent)?;
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
    /// Admits the historical relation of a group that `destination` held until now: the
    /// group is retired once, by a cause that names the field.
    pub(super) fn retire_group(
        &mut self,
        c: &mut ParseControl,
        group: &OriginGroupRef,
        destination: FieldRef,
    ) -> Result<(), SessionError> {
        group.require_disposal_owner(c)?;
        let header = self.require_group(group)?;
        if header.retirement.is_some() {
            return Err(SessionError::WrongToken);
        }
        let index = group.index;
        let cause = self.append_cause(
            c,
            ConstructionCause::Retirement {
                group: index,
                destination,
                trigger: self.native.current,
            },
        )?;
        payload_write::<Option<usize>>(c, 1)?;
        self.native
            .groups
            .get_mut(index, c)?
            .ok_or(SessionError::WrongToken)?
            .retirement = Some(cause);
        Ok(())
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let left = self.require_group(old)?;
        let right = self.require_group(incoming)?;
        if left.units != offset || offset.checked_add(right.units) != Some(total) {
            return Err(SessionError::WrongToken);
        }
        let cause = self.append_cause(
            c,
            ConstructionCause::Merge {
                original: old.index,
                incoming: incoming.index,
                trigger: self.native.current,
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
/// The walk over the origins of one processing instruction's data that the
/// pieces of its pseudo-attributes make. The pieces arrive in source order, so
/// the walk resumes where the previous piece stopped instead of starting from
/// the first origin again for each piece.
pub(super) struct PiOriginWalk {
    node: usize,
    state: PiOriginState,
}

#[derive(Default)]
enum PiOriginState {
    #[default]
    Unstarted,
    /// The instruction has no source origins to map.
    Unmapped,
    Walking(PiOriginResume),
}

struct PiOriginResume {
    cursor: GroupOriginCursor,
    /// The origin read and found to start at or after the last piece's end.
    held: Option<HtmlParseTextOrigin>,
    /// Where the last piece ended; the walk has consumed every origin before it.
    floor: usize,
}

impl PiOriginWalk {
    pub(super) fn new(node: usize) -> Self {
        Self {
            node,
            state: PiOriginState::Unstarted,
        }
    }
    pub(super) fn release(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        if let PiOriginState::Walking(mut resume) = mem::take(&mut self.state) {
            resume.cursor.release(c)?;
        }
        Ok(())
    }
}

impl SessionStorage {
    /// Appends the origins of one piece of a pseudo-attribute name or value, the
    /// input range `input` of the instruction's data copied to the output range
    /// `output`, resuming `walk` where the previous piece stopped.
    pub(super) fn pi_piece_origins(
        &self,
        c: &mut ParseControl,
        walk: &mut PiOriginWalk,
        input: std::ops::Range<usize>,
        output: std::ops::Range<usize>,
        transformed: bool,
        maps: &mut OwnedVec<HtmlParseTextOrigin>,
    ) -> Result<(), SessionError> {
        let (begin, end, output_start, output_end) =
            (input.start, input.end, output.start, output.end);
        if matches!(walk.state, PiOriginState::Unmapped) {
            return Ok(());
        }
        let evidence = match self.evidence.as_ref() {
            Some(evidence) if evidence.native.source.is_some() => evidence,
            _ => return Ok(()),
        };
        // A piece that starts before the end of the previous one needs origins
        // the walk has passed; the walk starts again from the first origin.
        if let PiOriginState::Walking(resume) = &walk.state
            && begin < resume.floor
        {
            walk.release(c)?;
        }
        if matches!(walk.state, PiOriginState::Unstarted) {
            let Some(original) = self
                .payloads
                .detail_origins
                .get(c, FieldRef::ProcessingInstructionData(walk.node))?
            else {
                walk.state = PiOriginState::Unmapped;
                return Ok(());
            };
            walk.state = PiOriginState::Walking(PiOriginResume {
                cursor: GroupOriginCursor::new(evidence, original, c)?,
                held: None,
                floor: 0,
            });
        }
        let PiOriginState::Walking(resume) = &mut walk.state else {
            // Both other states returned or became `Walking` above.
            return Err(latch_registry_row(c));
        };
        let mut first = None;
        let mut last = None;
        let mut covered = begin;
        loop {
            let origin = match resume.held.take() {
                Some(origin) => origin,
                None => match resume.cursor.next(evidence, c)? {
                    Some(origin) => origin,
                    None => break,
                },
            };
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if origin.end <= begin {
                continue;
            }
            if origin.start >= end {
                // It belongs to a later piece.
                resume.held = Some(origin);
                break;
            }
            if origin.start != covered || origin.end > end {
                return Err(SessionError::OriginSplitUnavailable);
            }
            if !transformed {
                payload_write::<HtmlParseTextOrigin>(c, 1)?;
                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                // `origin.start == covered >= begin`, and `origin.end > origin.start`.
                let (Some(from_start), Some(from_end)) = (
                    origin.start.checked_sub(begin),
                    origin.end.checked_sub(begin),
                ) else {
                    return Err(SessionError::OriginSplitUnavailable);
                };
                let start = output_start
                    .checked_add(from_start)
                    .ok_or_else(|| c.stop_representation(HtmlParseResource::ManagedUtf16Units))?;
                let end = output_start
                    .checked_add(from_end)
                    .ok_or_else(|| c.stop_representation(HtmlParseResource::ManagedUtf16Units))?;
                maps.try_push_geometric(
                    c,
                    HtmlParseTextOrigin {
                        start,
                        end,
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
        resume.floor = end;
        if covered != end {
            return Err(SessionError::OriginSplitUnavailable);
        }
        if transformed
            && let (Some(source), Some(first), Some(last)) = (&evidence.native.source, first, last)
        {
            payload_write::<HtmlParseTextOrigin>(c, 1)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            if source.as_utf8().is_some() {
                c.charge(
                    HtmlParseWorkCategory::InputUnitRead,
                    count_set([
                        first != 0 && first != source.len(),
                        last != 0 && last != source.len(),
                    ]) as u64,
                )?;
            }
            let source = source
                .range(first, last)
                .map_err(|_| SessionError::WrongToken)?;
            maps.try_push_geometric(
                c,
                HtmlParseTextOrigin {
                    start: output_start,
                    end: output_end,
                    source,
                    kind: HtmlParseOriginKind::RecoverySyntax,
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
        maps: OwnedVec<HtmlParseTextOrigin>,
    ) -> Result<usize, SessionError> {
        let original = match self.copied_field_group(c, FieldRef::ProcessingInstructionData(node)) {
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
        let records = self.construction.len();
        if intended_parent.is_some_and(|parent| parent >= records) {
            return Err(SessionError::ForeignNode);
        }
        let Some(evidence) = self.graph() else {
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
            ConstructionCause::Replay {
                original_cause: original,
                intended_parent,
                trigger: evidence.native.current,
            },
        )?;
        payload_write::<Option<usize>>(c, 1)?;
        evidence.native.current = Some(cause);
        Ok(previous)
    }
    #[cfg(test)]
    pub(super) fn copied_slot_group(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        let RetainedPayload::String { .. } = self.payloads.row(c, slot)? else {
            return Err(SessionError::WrongToken);
        };
        self.payloads
            .row_group(c, slot)?
            .map(|group| {
                self.evidence
                    .as_mut()
                    .ok_or(SessionError::WrongToken)?
                    .copy_group(c, group)
            })
            .transpose()
    }
    /// A copy of the group of the string of `field`, a non-element field, when it has one.
    pub(super) fn copied_field_group(
        &mut self,
        c: &mut ParseControl,
        field: FieldRef,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        let Some(group) = self.payloads.detail_origins.get(c, field)? else {
            return Ok(None);
        };
        self.evidence
            .as_mut()
            .ok_or(SessionError::WrongToken)?
            .copy_group(c, group)
            .map(Some)
    }
    pub(super) fn reset_group(
        &mut self,
        c: &mut ParseControl,
        old_slot: usize,
        incoming: Option<OriginGroupRef>,
        units: usize,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        let RetainedPayload::String { .. } = self.payloads.row(c, old_slot)? else {
            return Err(SessionError::WrongToken);
        };
        let old = self.payloads.row_group(c, old_slot)?;
        let Some(evidence) = graph_in(&mut self.evidence) else {
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
            .map(|old| evidence.require_group(old).map(|_| old.index))
            .transpose()?;
        let cause = evidence.append_cause(
            c,
            ConstructionCause::Replacement {
                subject: ReplacementSubject::Reset {
                    original,
                    incoming: incoming.index,
                },
                trigger: evidence.native.current,
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
        let (group, location) = match self.payloads.row(c, slot)? {
            RetainedPayload::String { location, .. } => match self.payloads.row_group(c, slot)? {
                Some(group) => (group, *location),
                None => return Ok(()),
            },
            // A string disposed of as an aggregate keeps only its group, and its field is no
            // longer known.
            RetainedPayload::HistoricalString { .. } => return Err(SessionError::WrongToken),
            _ => return Ok(()),
        };
        let evidence = self.evidence.as_mut().ok_or(SessionError::WrongToken)?;
        evidence.retire_group(c, group, location)
    }
    #[cfg(test)]
    pub(super) fn observed_origins(
        &self,
        group: &OriginGroupRef,
        c: &mut ParseControl,
    ) -> Result<Vec<HtmlParseTextOrigin>, SessionError> {
        self.evidence
            .as_ref()
            .ok_or(SessionError::WrongToken)?
            .observed_origins(group, c)
    }
    /// Whether the parse records the origin and cause graph.
    pub(super) fn graph_recorded(&self) -> bool {
        self.evidence
            .as_ref()
            .is_some_and(ParseEvidence::records_graph)
    }
    /// The evidence, when the parse records the origin and cause graph.
    pub(super) fn graph(&mut self) -> Option<&mut ParseEvidence> {
        graph_in(&mut self.evidence)
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
    pub(super) origins: OwnedVec<HtmlParseTextOrigin>,
    pub(super) original: Option<OriginGroupRef>,
}
impl FieldCopy {
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
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
        strings: LeafStrings<'_>,
        group: &OriginGroupRef,
        value: &[u16],
    ) -> Result<(), SessionError> {
        if !self.group_value_equal(c, strings, value, group)? {
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
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let Some(RetainedPayload::String { .. }) = strings.payloads.values.get(slot)
                    else {
                        return Err(SessionError::WrongToken);
                    };
                    let Some(original) = strings.payloads.row_group(c, slot)? else {
                        return Err(SessionError::WrongToken);
                    };
                    let header = self.require_group(original)?;
                    c.charge(
                        HtmlParseWorkCategory::InspectByte,
                        mem::size_of::<OriginGroup>() as u64,
                    )?;
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let cause = self
                        .native
                        .causes
                        .as_slice()
                        .get(header.cause)
                        .ok_or_else(|| latch_registry_row(c))?;
                    c.charge(
                        HtmlParseWorkCategory::InspectByte,
                        mem::size_of::<ConstructionCause>() as u64,
                    )?;
                    let value = cause
                        .computation()
                        .ok_or(SessionError::OriginSplitUnavailable)?;
                    validate_input_computation(
                        c,
                        *value,
                        self.native.groups.as_slice(),
                        header.cause,
                    )?;
                } else {
                    let mut covered = 0;
                    // Leaf chunks are issued only for ranges inside the origins.
                    let atoms = leaf
                        .start
                        .checked_add(leaf.length)
                        .and_then(|end| self.native.origins.as_slice().get(leaf.start..end))
                        .ok_or(SessionError::WrongToken)?;
                    for atom in atoms {
                        c.charge(
                            HtmlParseWorkCategory::InspectByte,
                            mem::size_of::<HtmlParseTextOrigin>() as u64,
                        )?;
                        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
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
        HtmlParseWorkCategory::InspectByte,
        mem::size_of::<RangeComputation>() as u64,
    )?;
    let step_branch = value.branch == HtmlParseRangeBranch::StepCorrection;
    if value.minimum.is_none()
        || value.maximum.is_none()
        || step_branch != (value.step.is_some() && value.base.is_some())
        || (!step_branch && (value.step.is_some() || value.base.is_some()))
    {
        return Err(SessionError::WrongToken);
    }
    if matches!(
        value.operands.get(HtmlParseRangeOperandRole::Minimum),
        RangeOperand::Any(_)
    ) || matches!(
        value.operands.get(HtmlParseRangeOperandRole::Maximum),
        RangeOperand::Any(_)
    ) || matches!(
        value.operands.get(HtmlParseRangeOperandRole::ValueContent),
        RangeOperand::Any(_)
    ) {
        return Err(SessionError::WrongToken);
    }
    for edge in value.edges() {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let Some(edge) = edge {
            c.charge(
                HtmlParseWorkCategory::InspectByte,
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
                HtmlParseWorkCategory::InspectByte,
                mem::size_of::<ColorComputation>() as u64,
            )?;
            if (color.source == HtmlParseColorValueSource::MissingInstalledValue)
                != color.original.is_none()
                || color.environment != super::input_color::environment_identity()
            {
                return Err(SessionError::WrongToken);
            }
            for edge in [
                color.original,
                color.operands.copy(HtmlParseColorOperandRole::Alpha),
                color.operands.copy(HtmlParseColorOperandRole::Colorspace),
            ] {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if let Some(edge) = edge {
                    c.charge(
                        HtmlParseWorkCategory::InspectByte,
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if !c.detail().records_graph() {
            // The computation that carries the edge is dropped unrecorded.
            return Ok(RangeEdge::UNRECORDED);
        }
        let RetainedPayload::String { .. } = self
            .payloads
            .values
            .get(slot)
            .ok_or(SessionError::WrongToken)?
        else {
            return Err(SessionError::WrongToken);
        };
        let Some(group) = self.payloads.row_group(c, slot)? else {
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
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        self.prepare_input_group(c, units, InputValueComputation::Range(computation))
    }
    pub(super) fn prepare_input_group(
        &mut self,
        c: &mut ParseControl,
        units: usize,
        computation: InputValueComputation,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        // The computation that names the origin groups of its operands is
        // recorded only with the graph those groups belong to.
        let Some(evidence) = graph_in(&mut self.evidence) else {
            return Ok(None);
        };
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
            ConstructionCause::Replacement {
                subject: ReplacementSubject::Computed(computation),
                trigger: evidence.native.current,
            },
        )?;
        let index = evidence.native.groups.len();
        let reference = evidence.group_ref(index, c)?;
        evidence.native.groups.try_push_geometric(
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
        Ok(Some(reference))
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
        // A parse that records no graph has no cause to name.
        if !self.records_graph() {
            return Ok(None);
        }
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
            ConstructionCause::Replacement {
                subject: ReplacementSubject::Rewrite {
                    original: original.index,
                },
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
        if !c.detail().records_graph() {
            origins.release(c)?;
            return self.register_group_string_owned(c, value, None);
        }
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
                let strings = LeafStrings::with_records(
                    &self.payloads,
                    ConstructionView::Active(&self.construction),
                );
                evidence.group_value_equal(c, strings, value.text().code_units(), original)?
                    && evidence.origin_rows_equal(c, original, &origins)?
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
    ) -> Result<Option<ConstructionCauseRef>, SessionError> {
        // The start tag being processed ends here, for the positions the
        // parser keeps whatever detail the parse selected.
        self.active_mut()?.token_end = Some(trigger.source.end());
        let evidence = self.evidence(c)?;
        if !evidence.records_graph() {
            return Ok(None);
        }
        evidence.parser_trigger(c, trigger).map(Some)
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
        kind: HtmlParseRawField,
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
        self.control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        self.control.charge(
            HtmlParseWorkCategory::InspectByte,
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
    ) -> Result<Option<HtmlParseTextOrigin>, SessionError> {
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
            .materialize_group(self.control, self.storage.leaf_strings(), &reference)
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
        self.control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
            matches!(row.destination, NativeDestination::Field(FieldRef::Text(owner)) if owner == node),
        )
    }
    #[cfg(test)]
    /// Match an attribute field to its native owner and position.
    pub(crate) fn attribute_for(
        &mut self,
        index: usize,
        node: &ParserNode,
        position: usize,
        field: NativeAttributeField,
    ) -> Result<bool, SessionError> {
        let node = self.storage.index(self.control, node)?;
        self.control.charge(
            HtmlParseWorkCategory::InspectByte,
            mem::size_of::<NativeAssociation>() as u64,
        )?;
        let Some((row, _)) = self.row(index)? else {
            return Ok(false);
        };
        let (owner, actual_position) = match (row.destination, field) {
            (
                NativeDestination::Field(FieldRef::AttributeName(owner, position)),
                NativeAttributeField::Name,
            )
            | (
                NativeDestination::Field(FieldRef::AttributeValue(owner, position)),
                NativeAttributeField::Value,
            ) => (owner, position),
            _ => return Ok(false),
        };
        if owner != node || actual_position != position {
            return Ok(false);
        }
        self.control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let surgeist_html::construction::ConstructionNodeKind::Element(element) = self
            .storage
            .construction
            .node_at(owner)
            .map(|record| record.kind())
            .ok_or(SessionError::WrongToken)?
        else {
            return Ok(false);
        };
        Ok(element.attributes().get(actual_position).is_some())
    }
    #[cfg(test)]
    pub(crate) fn cause_reference(
        &mut self,
        index: usize,
    ) -> Result<Option<ConstructionCauseRef>, SessionError> {
        self.control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        self.control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
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
        self.control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        kind: Option<HtmlParseOriginKind>,
        lineage: (usize, usize),
    ) -> Result<(), SessionError> {
        let (cause, group) = lineage;
        if let Some(source) = source {
            c.require_source_range(source)?;
        }
        if !self.records_source_map() {
            return Ok(());
        }
        payload_write::<NativeAssociation>(c, 1)?;
        if source.is_some() {
            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
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
                    relation: HtmlParseAssociationRelation::Output,
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
        if !evidence.records_graph() {
            // A parse that records no graph has no group to certify and no
            // row to project: its projection is complete and empty.
            if evidence.native.projection.is_some() {
                return Err(SessionError::WrongToken);
            }
            let mut rows = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
            rows.require_owner(c)?;
            payload_write::<Option<OwnedVec<NativeAssociation>>>(c, 1)?;
            evidence.native.projection = Some(rows);
            payload_write::<bool>(c, 1)?;
            evidence.native.projection_complete = true;
            return Ok(());
        }
        // Successful completed projection certifies every real history leaf,
        // including retired versions that are no longer a native destination.
        let strings = LeafStrings::with_records(
            &storage.payloads,
            ConstructionView::Active(&storage.construction),
        );
        for group in evidence.native.groups.as_slice() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.charge(
                HtmlParseWorkCategory::InspectByte,
                mem::size_of::<OriginGroup>() as u64,
            )?;
            if let GroupBody::Leaf { value, .. } = group.body {
                strings.units(c, value, group.units)?;
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let raw = match payload {
                RetainedPayload::Nonce {
                    node: Some(node),
                    raw,
                } => Some((*node, *raw, HtmlParseRawField::Nonce)),
                RetainedPayload::Input {
                    node: Some(node),
                    raw,
                    complete: true,
                    ..
                } => Some((*node, *raw, HtmlParseRawField::Input)),
                RetainedPayload::Textarea {
                    node: Some(node),
                    state,
                } => Some((*node, state.raw, HtmlParseRawField::Textarea)),
                _ => None,
            };
            if let Some((node, slot, kind)) = raw {
                if node >= storage.construction.len() {
                    return Err(SessionError::WrongToken);
                }
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let RetainedPayload::String {
                    value,
                    allocation,
                    origins,
                    location: FieldRef::Detached,
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
                evidence.validate_group_value(c, strings, value.code_units(), group)?;
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
                    || *node >= storage.construction.len()
                {
                    return Err(SessionError::WrongToken);
                }
                if evidence.records_source_map() {
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
                                relation: HtmlParseAssociationRelation::Creation,
                                destination: NativeDestination::Node(*node),
                                start: 0,
                                end: 0,
                                source: None,
                                kind: None,
                                cause: cause.index,
                            },
                        )?;
                }
                continue;
            }
            let RetainedPayload::String {
                value,
                allocation,
                location,
                ..
            } = payload
            else {
                continue;
            };
            c.require_document_string(allocation, value)?;
            let Some(native) = location.value(
                ConstructionView::Active(&storage.construction),
                &storage.payloads.values,
                c,
            )?
            else {
                continue;
            };
            let origins = storage.payloads.row_group(c, payload_slot)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            if !native.shares_storage_with(value) {
                return Err(SessionError::AliasedPayload);
            }
            if let Some(group) = origins {
                evidence.validate_group_value(c, strings, native.code_units(), group)?;
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
                evidence.validate_group_value(c, strings, value.code_units(), &group)?;
                evidence.project_field(
                    c,
                    NativeDestination::Field(*location),
                    value.len(),
                    &group,
                )?;
            }
        }
        // The strings of the non-element fields belong to their records, which are walked in
        // order; their groups are in the table of origins.
        let records = &storage.construction;
        for node in 0..records.len() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let fields: &[FieldRef] = match records.node_at(node).map(|record| record.kind()) {
                Some(ConstructionNodeKind::Text(_)) => &[FieldRef::Text(node)],
                Some(ConstructionNodeKind::Comment(_)) => &[FieldRef::Comment(node)],
                Some(ConstructionNodeKind::ProcessingInstruction(_)) => &[
                    FieldRef::ProcessingInstructionTarget(node),
                    FieldRef::ProcessingInstructionData(node),
                ],
                Some(ConstructionNodeKind::Doctype(_)) => &[
                    FieldRef::DoctypeName(node),
                    FieldRef::DoctypePublic(node),
                    FieldRef::DoctypeSystem(node),
                ],
                _ => continue,
            };
            for &field in fields {
                let Some(units) = field.units(ConstructionView::Active(records)) else {
                    continue;
                };
                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                if let Some(group) = storage.payloads.detail_origins.get(c, field)? {
                    evidence.validate_group_value(c, strings, units, group)?;
                    evidence.validate_field(c, units, group)?;
                    evidence.project_field(
                        c,
                        NativeDestination::Field(field),
                        units.len(),
                        group,
                    )?;
                } else {
                    let group = evidence.source_independent_group(c, units.len())?;
                    evidence.bind_leaf_field(c, &group, field, 0)?;
                    evidence.validate_group_value(c, strings, units, &group)?;
                    evidence.project_field(
                        c,
                        NativeDestination::Field(field),
                        units.len(),
                        &group,
                    )?;
                }
            }
        }
        payload_write::<usize>(c, 1)?;
        evidence.native.native_rows = evidence
            .native
            .projection
            .as_ref()
            .ok_or(SessionError::WrongToken)?
            .len();
        if evidence.records_source_map() {
            evidence.project_lineage(c)?;
            evidence.project_causes(c)?;
        }
        if evidence.records_lexical() {
            evidence.project_lexical(c)?;
        }
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
    role: HtmlParseCapturedCauseRole,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseCapturedCauseRole {
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
    pub(crate) role: HtmlParseCapturedCauseRole,
}
impl<'a> CapturedParserTrigger<'a> {
    pub(super) fn new(
        context: &'a crate::HtmlFragmentContext,
        role: HtmlParseCapturedCauseRole,
    ) -> Self {
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
        c.charge(HtmlParseWorkCategory::ProviderCall, 7)?;
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
            ConstructionCause::Captured {
                context: captured,
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
        if trigger.role != HtmlParseCapturedCauseRole::ContextNamespace {
            return Err(SessionError::WrongToken);
        }
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
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
            let Some(evidence) = self.graph() else {
                break;
            };
            if evidence.native.source.is_none() {
                break;
            }
            let cause = evidence.field_cause(c)?;
            let cause = evidence.cause_ref(cause, c)?;
            // Native binding is a fixed write in the later allocation commit.
            payload_write::<Option<usize>>(c, 1)?;
            let prepared = self.prepare_payload_slot_owned(c)?;
            let slot = self
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::ConstructionCause { node: None, cause },
                )
                .map_err(|_| latch_registry_row(c))?;
            *entry = Some(slot);
        }
        Ok(slots)
    }
    pub(super) fn begin_node_copy(
        &mut self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<Option<ConstructionCauseRef>, SessionError> {
        if source >= self.construction.len() {
            return Err(SessionError::ForeignNode);
        }
        let Some(evidence) = self.graph() else {
            return Ok(None);
        };
        let previous = evidence.current_cause(c)?;
        let index = evidence.append_cause(
            c,
            ConstructionCause::Copy {
                subject: CopySubject::Node(source),
                trigger: evidence.native.current,
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
        let Some(evidence) = self.graph() else {
            return Ok(None);
        };
        let previous = evidence.current_cause(c)?;
        payload_write::<Option<crate::HtmlSourceRange>>(c, 1)?;
        let index = evidence.append_cause(
            c,
            ConstructionCause::Replacement {
                subject: ReplacementSubject::AttributeChange {
                    node,
                    source: source.cloned(),
                },
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
        let Some(evidence) = self.graph() else {
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
    use surgeist_html::{
        construction::HtmlDocumentMode,
        values::{DocumentString, Namespace, OrderedAttributes, QualifiedName},
    };

    use crate::{
        HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCapturedIsValue,
        HtmlFragmentContext, HtmlHostFactsRevision, HtmlScriptingStatus,
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
                CapturedParserTrigger::new(&context, HtmlParseCapturedCauseRole::FragmentBootstrap),
            )
            .unwrap();
        let uri = DocumentString::from("http://www.w3.org/2000/svg");
        let group = evidence
            .retain_captured_namespace(
                &mut c,
                CapturedParserTrigger::new(&context, HtmlParseCapturedCauseRole::ContextNamespace),
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
        assert_eq!(cause.operation(), HtmlParseConstructionOperation::Captured);
        assert!(cause.source().is_none());
        let retained = cause.captured().unwrap();
        assert_eq!(&retained.capture, capture.identity());
        assert_eq!(retained.host_revision, revision);
        assert_eq!(&retained.context, context.context_element());
        assert_eq!(&retained.target, context.target());
        assert_eq!(&retained.context_document, context.context_document());
        assert_eq!(&retained.target_document, context.target_document());
        assert_eq!(retained.role, HtmlParseCapturedCauseRole::ContextNamespace);
        assert_eq!(cause.trigger(), Some(bootstrap.index));
        assert_eq!(evidence.native.causes.len(), 2);
        assert!(matches!(
            evidence.retain_captured_namespace(
                &mut c,
                CapturedParserTrigger::new(&context, HtmlParseCapturedCauseRole::ContextNamespace),
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
                    HtmlParseTextOrigin {
                        start: 0,
                        end: 1,
                        source: source.range(begin, begin + 1).unwrap(),
                        kind: HtmlParseOriginKind::Literal,
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
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
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
        // retained capacity is counted and reclaimed with the evidence owner.
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
            matches!(evidence.merge_groups(&mut c,&old,&incoming,1,2),Err(SessionError::Stop(HtmlParseStop::LimitExceeded{resource:HtmlParseResource::WorkCredits,limit,current,requested_delta:2})) if limit==base+1 && current==base)
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
                (0, 1, 0, 1, HtmlParseOriginKind::Literal),
                (1, 2, 1, 2, HtmlParseOriginKind::Literal)
            ]
        );
        let header = evidence.require_group(&merged).unwrap();
        assert_eq!(header.units, 2);
        let cause = &evidence.native.causes.as_slice()[header.cause];
        assert_eq!(cause.operation(), HtmlParseConstructionOperation::Merge);
        assert_eq!(
            (cause.original(), cause.incoming()),
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
            Err(SessionError::Stop(HtmlParseStop::Cancelled))
        ));
        assert_original_groups(&evidence, &old, &incoming);
        assert_eq!(c.usage(), before);
        drop((old, incoming));
        evidence.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
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
    if !c.detail().records_graph() {
        // No origin group names this value, so its copy has none to preserve.
        let mut buffer = Utf16Buffer::new(c)?;
        if let Err(error) = buffer.extend_units(value.code_units(), c) {
            buffer.release(c)?;
            return Err(error.into());
        }
        let origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        return match buffer.freeze(c) {
            Ok(value) => Ok(FieldCopy {
                value,
                origins,
                original: None,
            }),
            Err(error) => {
                origins.release(c)?;
                Err(error.into())
            }
        };
    }
    let RetainedPayload::String { .. } = payloads.row(c, slot)? else {
        return Err(SessionError::WrongToken);
    };
    let Some(group) = payloads.row_group(c, slot)? else {
        return Err(SessionError::WrongToken);
    };
    let evidence = evidence.ok_or(SessionError::WrongToken)?;
    evidence.qualify_generated_copy(c, LeafStrings::rows(payloads), group, value.code_units())?;
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

/// A character-reference origin is checked against what its source text
/// stands for in the pinned decoding table, not only for being non-empty.
#[cfg(test)]
mod character_reference_origin_tests {
    use super::*;
    use crate::{HtmlParseCancellation, HtmlSource};

    /// `(output start, output end, source start, source end)` per origin.
    type Atom = (usize, usize, usize, usize);

    fn validate(source: &HtmlSource, output: &[u16], atoms: &[Atom]) -> Result<(), SessionError> {
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            HtmlParseCancellation::new(),
        );
        c.admit_source(source).unwrap();
        let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
        evidence.initialize_native_source(&mut c, source).unwrap();
        let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
        for &(start, end, begin, finish) in atoms {
            origins
                .try_push(
                    &mut c,
                    HtmlParseTextOrigin {
                        start,
                        end,
                        source: source.range(begin, finish).unwrap(),
                        kind: HtmlParseOriginKind::CharacterReference,
                    },
                )
                .unwrap();
        }
        let group = evidence
            .retain_group(&mut c, origins, output.len())
            .unwrap();
        let result = evidence.validate_field(&mut c, output, &group);
        drop(group);
        evidence.release(&mut c).unwrap();
        result
    }
    fn units(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }
    fn accepts(source: &str, output: &str, atoms: &[Atom]) {
        let result = validate(&HtmlSource::utf8(source), &units(output), atoms);
        assert!(result.is_ok(), "{source:?} -> {output:?}: {result:?}");
    }
    fn refuses(source: &str, output: &str, atoms: &[Atom]) {
        let result = validate(&HtmlSource::utf8(source), &units(output), atoms);
        assert!(
            matches!(result, Err(SessionError::WrongToken)),
            "{source:?} -> {output:?}: {result:?}"
        );
    }

    #[test]
    fn a_named_reference_yields_the_scalar_the_table_assigns_it() {
        accepts("&amp;", "&", &[(0, 1, 0, 5)]);
        accepts("&amp", "&", &[(0, 1, 0, 4)]);
        accepts("&AElig;", "\u{c6}", &[(0, 1, 0, 7)]);
        accepts("&Afr;", "\u{1d504}", &[(0, 2, 0, 5)]);
    }
    #[test]
    fn a_named_reference_attached_to_another_output_is_refused() {
        refuses("&amp;", "<", &[(0, 1, 0, 5)]);
        refuses("&lt;", "&", &[(0, 1, 0, 4)]);
        refuses("&Afr;", "A", &[(0, 1, 0, 5)]);
    }
    #[test]
    fn a_name_the_table_lacks_is_refused() {
        refuses("&ampx;", "&", &[(0, 1, 0, 6)]);
        refuses("&;", "&", &[(0, 1, 0, 2)]);
        refuses("&", "&", &[(0, 1, 0, 1)]);
        // Table names are case-sensitive and `&AMP;` is its own entry.
        accepts("&AMP;", "&", &[(0, 1, 0, 5)]);
        refuses("&Amp;", "&", &[(0, 1, 0, 5)]);
    }
    #[test]
    fn a_two_scalar_reference_pairs_its_origins_with_the_scalars_in_order() {
        // `&NotEqualTilde;` is U+2242 U+0338.
        let atoms = [(0, 1, 0, 15), (1, 2, 0, 15)];
        accepts("&NotEqualTilde;", "\u{2242}\u{338}", &atoms);
        refuses("&NotEqualTilde;", "\u{338}\u{2242}", &atoms);
        refuses("&NotEqualTilde;", "\u{2242}\u{2242}", &atoms);
        refuses("&NotEqualTilde;", "\u{2242}", &[(0, 1, 0, 15)]);
    }
    #[test]
    fn a_numeric_reference_yields_the_scalar_the_standard_assigns_it() {
        accepts("&#65;", "A", &[(0, 1, 0, 5)]);
        accepts("&#x41;", "A", &[(0, 1, 0, 6)]);
        accepts("&#X41", "A", &[(0, 1, 0, 5)]);
        accepts("&#x1D504;", "\u{1d504}", &[(0, 2, 0, 9)]);
        // Null, surrogates and values past U+10FFFF become U+FFFD.
        accepts("&#0;", "\u{fffd}", &[(0, 1, 0, 4)]);
        accepts("&#xD800;", "\u{fffd}", &[(0, 1, 0, 8)]);
        accepts("&#x110000;", "\u{fffd}", &[(0, 1, 0, 10)]);
        accepts("&#99999999999999999999;", "\u{fffd}", &[(0, 1, 0, 23)]);
        // The C1 range maps through the Windows-1252 table; unassigned stay.
        accepts("&#128;", "\u{20ac}", &[(0, 1, 0, 6)]);
        accepts("&#x81;", "\u{81}", &[(0, 1, 0, 6)]);
        // Carriage return by reference is not a newline normalization.
        accepts("&#13;", "\r", &[(0, 1, 0, 5)]);
    }
    #[test]
    fn a_numeric_reference_attached_to_another_output_is_refused() {
        refuses("&#65;", "B", &[(0, 1, 0, 5)]);
        refuses("&#128;", "\u{80}", &[(0, 1, 0, 6)]);
        refuses("&#0;", "\0", &[(0, 1, 0, 4)]);
        refuses("&#xD800;", "\u{d7ff}", &[(0, 1, 0, 8)]);
    }
    #[test]
    fn a_malformed_numeric_spelling_is_refused() {
        refuses("&#;", "\u{fffd}", &[(0, 1, 0, 3)]);
        refuses("&#x;", "\u{fffd}", &[(0, 1, 0, 4)]);
        refuses("&#65x;", "A", &[(0, 1, 0, 6)]);
        refuses("&#65;;", "A", &[(0, 1, 0, 6)]);
        refuses("&#xG;", "A", &[(0, 1, 0, 5)]);
    }
    #[test]
    fn a_utf16_source_is_decoded_like_a_utf8_one() {
        let source = HtmlSource::utf16(DocumentString::from("a&amp;b"));
        assert!(validate(&source, &units("&"), &[(0, 1, 1, 6)]).is_ok());
        assert!(matches!(
            validate(&source, &units("<"), &[(0, 1, 1, 6)]),
            Err(SessionError::WrongToken)
        ));
    }
}

/// The evidence in `slot`, when the parse records the origin and cause graph.
/// It borrows only the evidence, so the rest of the storage stays usable.
pub(super) fn graph_in(slot: &mut Option<ParseEvidence>) -> Option<&mut ParseEvidence> {
    slot.as_mut().filter(|evidence| evidence.records_graph())
}
