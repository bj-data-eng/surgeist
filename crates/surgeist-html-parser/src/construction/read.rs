//! Fixed public lenders over the already qualified immutable report owner.
use super::*;
use history::{FrozenHistory, HistoryView, HtmlParseOriginGroup};
use provenance::{AssociationView, ConstructionCause, NativeAssociation, NativeDestination};
use surgeist_html::construction::ConstructionNodeKind;
use surgeist_html::{
    construction::HtmlNode,
    elements::{
        form::HtmlNativeFormAssociationKind,
        script::{HtmlScriptKind, HtmlScriptReadySteps, HtmlScriptResult},
    },
    values::DocumentString,
};

/// What a reference of a published report denotes to a reader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlParseUnresolvedKind {
    /// A node of the report's tree.
    Node,
    /// A construction cause.
    Cause,
    /// A text origin group of the history.
    Group,
    /// A lexical occurrence.
    LexicalOccurrence,
    /// A part of a lexical occurrence.
    LexicalPart,
    /// A lexical omission.
    Omission,
    /// A fact the report retains: a string, a policy, an attribute map, a
    /// metadata row or an effect binding.
    Fact,
    /// A node of the captured fragment context.
    CapturedNode,
    /// The fragment context of a contextual report.
    Fragment,
    /// The location of a committed diagnostic.
    Diagnostic,
}
impl std::fmt::Display for HtmlParseUnresolvedKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Node => "a node of its tree",
            Self::Cause => "a construction cause",
            Self::Group => "an origin group of its text history",
            Self::LexicalOccurrence => "a lexical occurrence",
            Self::LexicalPart => "a part of a lexical occurrence",
            Self::Omission => "a lexical omission",
            Self::Fact => "a fact it retains",
            Self::CapturedNode => "a node of its captured context",
            Self::Fragment => "its fragment context",
            Self::Diagnostic => "the location of a diagnostic",
        })
    }
}

/// A reference that a published report holds but that its own tables cannot
/// resolve.
///
/// A report is qualified once, when it is published, and is immutable
/// afterwards, so every reference it holds resolves. A reader that meets one
/// that does not has found a defect in that qualification. It reports the
/// fault instead of answering as though the reference were absent, because an
/// answer built on a lost reference is wrong output that looks right. The
/// consumer fails its operation on it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlParseUnresolvedReference {
    kind: HtmlParseUnresolvedKind,
}
impl HtmlParseUnresolvedReference {
    pub(crate) const fn new(kind: HtmlParseUnresolvedKind) -> Self {
        Self { kind }
    }
    /// What the reference denotes.
    pub const fn kind(&self) -> HtmlParseUnresolvedKind {
        self.kind
    }
}
impl std::fmt::Display for HtmlParseUnresolvedReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "a published parse report refers to {} that it does not hold",
            self.kind
        )
    }
}
impl std::error::Error for HtmlParseUnresolvedReference {}

/// Qualify only the references used by fixed public readers. The canonical
/// history transfer still owns source/group/leaf qualification.
pub(crate) fn prepare_public_readers(
    input: &Option<FrozenArena>,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    let frozen = input.as_ref().ok_or(SessionError::Consumed)?;
    let storage = frozen.storage.as_ref().ok_or(SessionError::Consumed)?;
    c.require_identity(&storage.control)?;
    let records = ConstructionView::Frozen(&storage.view);
    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let evidence = storage.evidence.as_ref().ok_or(SessionError::WrongToken)?;
    payload_write::<AssociationView<'_>>(c, 1)?;
    let view = evidence.associations()?;
    fn record<T>(c: &mut ParseControl) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            mem::size_of::<T>() as u64,
        )?;
        Ok(())
    }
    fn node(
        records: ConstructionView<'_>,
        c: &mut ParseControl,
        index: usize,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if records.node_at(index).is_none() {
            return Err(SessionError::ForeignNode);
        }
        Ok(())
    }
    fn location(
        storage: &FrozenStorage,
        records: ConstructionView<'_>,
        c: &mut ParseControl,
        location: FieldRef,
    ) -> Result<(), SessionError> {
        match location {
            FieldRef::Detached => Ok(()),
            FieldRef::Text(i)
            | FieldRef::Comment(i)
            | FieldRef::ProcessingInstructionTarget(i)
            | FieldRef::ProcessingInstructionData(i)
            | FieldRef::DoctypeName(i)
            | FieldRef::DoctypePublic(i)
            | FieldRef::DoctypeSystem(i)
            | FieldRef::ElementName(i)
            | FieldRef::Namespace(i) => node(records, c, i),
            FieldRef::AttributeName(owner, index)
            | FieldRef::AttributeValue(owner, index)
            | FieldRef::AttributeNamespace(owner, index)
            | FieldRef::AttributePrefix(owner, index) => {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                match records.node_at(owner).map(|record| record.kind()) {
                    Some(ConstructionNodeKind::Element(element))
                        if index < element.attributes().len() => {}
                    _ => return Err(SessionError::WrongToken),
                }
                node(records, c, owner)
            }
            FieldRef::PiName(slot, index) | FieldRef::PiValue(slot, index) => {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let Some(RetainedPayload::PiAttributes {
                    node: Some(owner),
                    values,
                    ready: true,
                }) = storage.payloads.values.get(slot)
                else {
                    return Err(SessionError::WrongToken);
                };
                if index >= values.len() {
                    return Err(SessionError::WrongToken);
                };
                node(records, c, *owner)
            }
        }
    }
    fn retired_location(
        storage: &FrozenStorage,
        records: ConstructionView<'_>,
        c: &mut ParseControl,
        value: FieldRef,
    ) -> Result<(), SessionError> {
        match value {
            // The ordinal describes the retired field, not a current attribute index: only the
            // element that held it must exist.
            FieldRef::AttributeName(owner, _)
            | FieldRef::AttributeValue(owner, _)
            | FieldRef::AttributeNamespace(owner, _)
            | FieldRef::AttributePrefix(owner, _) => {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                node(records, c, owner)
            }
            _ => location(storage, records, c, value),
        }
    }
    for index in 0..view.causes_len() {
        record::<ConstructionCause>(c)?;
        let cause = view.cause(index).ok_or(SessionError::WrongToken)?;
        for i in [cause.original_node(), cause.intended_parent()]
            .into_iter()
            .flatten()
        {
            node(records, c, i)?;
        }
        if let provenance::ConstructionCause::Retirement {
            group, destination, ..
        } = cause
        {
            record::<provenance::OriginGroup>(c)?;
            if view
                .group(*group)
                .is_none_or(|g| g.retirement != Some(index))
            {
                return Err(SessionError::WrongToken);
            }
            // The original issuer established this ordinal before removal or
            // transfer. Capacity is only a retained-storage sanity bound;
            // the canonical retirement relation above proves field history.
            retired_location(storage, records, c, *destination)?;
        }
    }
    for index in 0..view.lexical_occurrences_len() {
        record::<provenance::LexicalOccurrence>(c)?;
        let value = view
            .lexical_occurrence(index)
            .ok_or(SessionError::WrongToken)?;
        if value.parent.is_some_and(|i| i >= index) || view.cause(value.cause()).is_none() {
            return Err(SessionError::WrongToken);
        }
        c.require_source_range(&value.source)?;
    }
    for index in 0..view.lexical_parts_len() {
        record::<provenance::LexicalPart>(c)?;
        let value = view.lexical_part(index).ok_or(SessionError::WrongToken)?;
        if view.lexical_occurrence(value.occurrence).is_none() {
            return Err(SessionError::WrongToken);
        }
        c.require_source_range(&value.source)?;
    }
    for index in 0..view.omissions_len() {
        record::<provenance::LexicalOmission>(c)?;
        let value = view.omission(index).ok_or(SessionError::WrongToken)?;
        if view.cause(value.cause()).is_none() {
            return Err(SessionError::WrongToken);
        }
        c.require_source_range(&value.source)?;
    }
    for index in 0..view.len() {
        record::<NativeAssociation>(c)?;
        let row = view.row(index).ok_or(SessionError::WrongToken)?;
        if view.cause(row.cause).is_none() || row.group.is_some_and(|i| view.group(i).is_none()) {
            return Err(SessionError::WrongToken);
        }
        match row.destination {
            NativeDestination::Field(value) => {
                if value == FieldRef::Detached {
                    return Err(SessionError::WrongToken);
                }
                location(storage, records, c, value)?;
            }
            NativeDestination::Node(i) | NativeDestination::Raw { node: i, .. } => {
                node(records, c, i)?
            }
            NativeDestination::Lexical { occurrence, part } => {
                if view.lexical_occurrence(occurrence).is_none()
                    || part.is_some_and(|i| {
                        view.lexical_part(i)
                            .is_none_or(|p| p.occurrence != occurrence)
                    })
                {
                    return Err(SessionError::WrongToken);
                }
            }
            NativeDestination::Omission(i) if view.omission(i).is_none() => {
                return Err(SessionError::WrongToken);
            }
            NativeDestination::Cause(i) if view.cause(i).is_none() => {
                return Err(SessionError::WrongToken);
            }
            _ => {}
        }
        if let Some(source) = &row.source {
            c.require_source_range(source)?;
        }
    }
    Ok(())
}

/// A borrowed immutable D1 inventory. Indexed observations inspect one row.
pub struct HtmlParseAssociations<'a> {
    history: &'a FrozenHistory,
    view: AssociationView<'a>,
}
impl<'a> HtmlParseAssociations<'a> {
    pub(crate) fn new(history: &'a FrozenHistory) -> Self {
        Self {
            history,
            view: history.associations(),
        }
    }
    pub fn len(&self) -> usize {
        self.view.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// The row at `index`, or `None` past the last row. A row whose cause,
    /// group or destination the report cannot resolve is a fault.
    pub fn row(
        &self,
        index: usize,
    ) -> Result<Option<HtmlParseAssociation<'a>>, HtmlParseUnresolvedReference> {
        self.view
            .row(index)
            .map(|row| HtmlParseAssociation::new(self.history, row))
            .transpose()
    }
    /// Starts a query for the rows whose source overlaps `source`. There is no index by
    /// position: each `advance` examines one row, and a query advanced to completion visits all
    /// [`len`](Self::len) of them, so `q` queries cost `q` times that many row visits.
    pub fn reverse(
        &self,
        source: &'a crate::HtmlSourceRange,
        cancellation: Option<&'a crate::HtmlParseCancellation>,
    ) -> Result<HtmlParseReverseCursor<'a>, provenance::reverse::HtmlParseReverseError> {
        Ok(HtmlParseReverseCursor {
            history: self.history,
            cursor: self.view.reverse(source, cancellation)?,
            fault: None,
        })
    }
}
/// One row of the inventory, with its cause, group and destination resolved
/// when it is lent, so that they cannot fail afterwards.
#[derive(Clone, Copy)]
pub struct HtmlParseAssociation<'a> {
    row: &'a NativeAssociation,
    cause: HtmlParseCause<'a>,
    group: Option<HtmlParseOriginGroup<'a>>,
    destination: HtmlParseDestination<'a>,
}
impl<'a> HtmlParseAssociation<'a> {
    fn new(
        history: &'a FrozenHistory,
        row: &'a NativeAssociation,
    ) -> Result<Self, HtmlParseUnresolvedReference> {
        let cause =
            history
                .associations()
                .cause(row.cause)
                .ok_or(HtmlParseUnresolvedReference::new(
                    HtmlParseUnresolvedKind::Cause,
                ))?;
        let group = row
            .group
            .map(|index| {
                history
                    .view()
                    .group(index)
                    .ok_or(HtmlParseUnresolvedReference::new(
                        HtmlParseUnresolvedKind::Group,
                    ))
            })
            .transpose()?;
        Ok(Self {
            row,
            cause: HtmlParseCause { history, cause },
            group,
            destination: destination(history, row.destination)?,
        })
    }
    pub fn source(&self) -> Option<&'a crate::HtmlSourceRange> {
        self.row.source.as_ref()
    }
    pub fn start(&self) -> usize {
        self.row.start
    }
    pub fn end(&self) -> usize {
        self.row.end
    }
    pub fn origin_kind(&self) -> Option<crate::parser::tokenizer::token::HtmlParseOriginKind> {
        self.row.kind
    }
    pub fn relation(&self) -> provenance::reverse::HtmlParseAssociationRelation {
        self.row.relation
    }
    pub fn group(&self) -> Option<HtmlParseOriginGroup<'a>> {
        self.group
    }
    pub fn cause(&self) -> HtmlParseCause<'a> {
        self.cause
    }
    pub fn destination(&self) -> HtmlParseDestination<'a> {
        self.destination
    }
}
/// A native destination never exposes an unchecked arena or payload slot.
#[derive(Clone, Copy)]
pub enum HtmlParseDestination<'a> {
    Field {
        node: HtmlNode<'a>,
        field: HtmlParseField,
    },
    Node(HtmlNode<'a>),
    Raw {
        node: HtmlNode<'a>,
        kind: provenance::HtmlParseRawField,
    },
    Lexical {
        occurrence: HtmlParseLexicalOccurrence<'a>,
        part: Option<HtmlParseLexicalPart<'a>>,
    },
    Omission(HtmlParseOmission<'a>),
    LexicalEnd,
    Cause(HtmlParseCause<'a>),
}
pub enum HtmlParseRetiredDestination<'a> {
    Detached,
    /// A historical field of a native owner.
    /// Its ordinal describes the retired field, not a current attribute index.
    Field {
        node: HtmlNode<'a>,
        field: HtmlParseField,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseField {
    Text,
    Comment,
    ProcessingInstructionTarget,
    ProcessingInstructionData,
    DoctypeName,
    DoctypePublic,
    DoctypeSystem,
    ElementName,
    Namespace,
    AttributeName(usize),
    AttributeValue(usize),
    AttributeNamespace(usize),
    AttributePrefix(usize),
    ProcessingInstructionAttributeName(usize),
    ProcessingInstructionAttributeValue(usize),
}
/// The node a field names and which of its fields it is.
fn field_of(
    history: &FrozenHistory,
    location: FieldRef,
) -> Result<(HtmlNode<'_>, HtmlParseField), HtmlParseUnresolvedReference> {
    let (node, field) = match location {
        FieldRef::Text(i) => (i, HtmlParseField::Text),
        FieldRef::Comment(i) => (i, HtmlParseField::Comment),
        FieldRef::ProcessingInstructionTarget(i) => {
            (i, HtmlParseField::ProcessingInstructionTarget)
        }
        FieldRef::ProcessingInstructionData(i) => (i, HtmlParseField::ProcessingInstructionData),
        FieldRef::DoctypeName(i) => (i, HtmlParseField::DoctypeName),
        FieldRef::DoctypePublic(i) => (i, HtmlParseField::DoctypePublic),
        FieldRef::DoctypeSystem(i) => (i, HtmlParseField::DoctypeSystem),
        FieldRef::ElementName(i) => (i, HtmlParseField::ElementName),
        FieldRef::Namespace(i) => (i, HtmlParseField::Namespace),
        FieldRef::AttributeName(i, a) => (i, HtmlParseField::AttributeName(a)),
        FieldRef::AttributeValue(i, a) => (i, HtmlParseField::AttributeValue(a)),
        FieldRef::AttributeNamespace(i, a) => (i, HtmlParseField::AttributeNamespace(a)),
        FieldRef::AttributePrefix(i, a) => (i, HtmlParseField::AttributePrefix(a)),
        FieldRef::PiName(i, a) => (
            pi_node(history, i)?,
            HtmlParseField::ProcessingInstructionAttributeName(a),
        ),
        FieldRef::PiValue(i, a) => (
            pi_node(history, i)?,
            HtmlParseField::ProcessingInstructionAttributeValue(a),
        ),
        // A detached payload is not a native destination.
        FieldRef::Detached => {
            return Err(HtmlParseUnresolvedReference::new(
                HtmlParseUnresolvedKind::Fact,
            ));
        }
    };
    Ok((history.native_node(node)?, field))
}
fn destination(
    history: &FrozenHistory,
    value: NativeDestination,
) -> Result<HtmlParseDestination<'_>, HtmlParseUnresolvedReference> {
    let view = history.associations();
    Ok(match value {
        NativeDestination::Node(i) => HtmlParseDestination::Node(history.native_node(i)?),
        NativeDestination::Raw { node, kind } => HtmlParseDestination::Raw {
            node: history.native_node(node)?,
            kind,
        },
        NativeDestination::Field(location) => {
            let (node, field) = field_of(history, location)?;
            HtmlParseDestination::Field { node, field }
        }
        NativeDestination::Lexical { occurrence, part } => {
            let value =
                view.lexical_occurrence(occurrence)
                    .ok_or(HtmlParseUnresolvedReference::new(
                        HtmlParseUnresolvedKind::LexicalOccurrence,
                    ))?;
            let part = part
                .map(|i| {
                    view.lexical_part(i)
                        .map(|value| HtmlParseLexicalPart { history, value })
                        .ok_or(HtmlParseUnresolvedReference::new(
                            HtmlParseUnresolvedKind::LexicalPart,
                        ))
                })
                .transpose()?;
            HtmlParseDestination::Lexical {
                occurrence: HtmlParseLexicalOccurrence { history, value },
                part,
            }
        }
        NativeDestination::Omission(i) => HtmlParseDestination::Omission(HtmlParseOmission {
            value: view.omission(i).ok_or(HtmlParseUnresolvedReference::new(
                HtmlParseUnresolvedKind::Omission,
            ))?,
        }),
        NativeDestination::LexicalEnd => HtmlParseDestination::LexicalEnd,
        NativeDestination::Cause(i) => HtmlParseDestination::Cause(HtmlParseCause {
            history,
            cause: view.cause(i).ok_or(HtmlParseUnresolvedReference::new(
                HtmlParseUnresolvedKind::Cause,
            ))?,
        }),
    })
}
fn pi_node(history: &FrozenHistory, slot: usize) -> Result<usize, HtmlParseUnresolvedReference> {
    match history.storage().payloads.values.get(slot) {
        Some(RetainedPayload::PiAttributes {
            node: Some(node), ..
        }) => Ok(*node),
        _ => Err(HtmlParseUnresolvedReference::new(
            HtmlParseUnresolvedKind::Fact,
        )),
    }
}
/// The origin group a stored index names, or the fault.
fn group_of(
    history: &FrozenHistory,
    index: usize,
) -> Result<HtmlParseOriginGroup<'_>, HtmlParseUnresolvedReference> {
    history
        .view()
        .group(index)
        .ok_or(HtmlParseUnresolvedReference::new(
            HtmlParseUnresolvedKind::Group,
        ))
}
/// The cause a stored index names, or the fault.
fn cause_of(
    history: &FrozenHistory,
    index: usize,
) -> Result<HtmlParseCause<'_>, HtmlParseUnresolvedReference> {
    history
        .associations()
        .cause(index)
        .map(|cause| HtmlParseCause { history, cause })
        .ok_or(HtmlParseUnresolvedReference::new(
            HtmlParseUnresolvedKind::Cause,
        ))
}
/// The lexical occurrence a stored index names, or the fault.
fn occurrence_of(
    history: &FrozenHistory,
    index: usize,
) -> Result<HtmlParseLexicalOccurrence<'_>, HtmlParseUnresolvedReference> {
    history
        .associations()
        .lexical_occurrence(index)
        .map(|value| HtmlParseLexicalOccurrence { history, value })
        .ok_or(HtmlParseUnresolvedReference::new(
            HtmlParseUnresolvedKind::LexicalOccurrence,
        ))
}
/// Fixed borrowed decisions from the qualified canonical cause owner.
#[derive(Clone, Copy)]
pub struct HtmlParseRangeComputation<'a> {
    history: &'a FrozenHistory,
    value: &'a provenance::RangeComputation,
}
#[derive(Clone, Copy)]
pub struct HtmlParseRangeOperand<'a> {
    history: &'a FrozenHistory,
    value: provenance::RangeOperand,
}
impl<'a> HtmlParseRangeOperand<'a> {
    pub fn kind(&self) -> provenance::HtmlParseRangeOperandKind {
        self.value.kind()
    }
    pub fn group(&self) -> Result<Option<HtmlParseOriginGroup<'a>>, HtmlParseUnresolvedReference> {
        self.value
            .edge()
            .map(|edge| group_of(self.history, edge.group))
            .transpose()
    }
    pub fn cause(&self) -> Result<Option<HtmlParseCause<'a>>, HtmlParseUnresolvedReference> {
        self.value
            .edge()
            .map(|edge| cause_of(self.history, edge.cause))
            .transpose()
    }
}
impl<'a> HtmlParseRangeComputation<'a> {
    pub fn branch(&self) -> provenance::HtmlParseRangeBranch {
        self.value.branch
    }
    pub fn original_is_fresh_empty(&self) -> bool {
        matches!(self.value.original, provenance::RangeOriginal::FreshEmpty)
    }
    pub fn original_group(
        &self,
    ) -> Result<Option<HtmlParseOriginGroup<'a>>, HtmlParseUnresolvedReference> {
        match self.value.original {
            provenance::RangeOriginal::FreshEmpty => Ok(None),
            provenance::RangeOriginal::Group(edge) => group_of(self.history, edge.group).map(Some),
        }
    }
    pub fn original_cause(
        &self,
    ) -> Result<Option<HtmlParseCause<'a>>, HtmlParseUnresolvedReference> {
        match self.value.original {
            provenance::RangeOriginal::FreshEmpty => Ok(None),
            provenance::RangeOriginal::Group(edge) => cause_of(self.history, edge.cause).map(Some),
        }
    }
    pub fn operand(
        &self,
        role: provenance::HtmlParseRangeOperandRole,
    ) -> HtmlParseRangeOperand<'a> {
        HtmlParseRangeOperand {
            history: self.history,
            value: self.value.operands.copy(role),
        }
    }
    pub fn minimum_origin(
        &self,
    ) -> Option<surgeist_html::attributes::input_bounds::HtmlInputNumericBoundOrigin> {
        self.value.minimum
    }
    pub fn maximum_origin(
        &self,
    ) -> Option<surgeist_html::attributes::input_bounds::HtmlInputNumericBoundOrigin> {
        self.value.maximum
    }
    pub fn step_origin(
        &self,
    ) -> Option<surgeist_html::attributes::input_bounds::HtmlInputNumericStepOrigin> {
        self.value.step
    }
    pub fn base_origin(
        &self,
    ) -> Option<surgeist_html::attributes::input_bounds::HtmlInputNumericStepBaseOrigin> {
        self.value.base
    }
}
#[derive(Clone, Copy)]
pub struct HtmlParseCause<'a> {
    history: &'a FrozenHistory,
    cause: &'a ConstructionCause,
}
/// Borrowed decisions and dependency groups from an admitted Color update.
/// The environment identity denotes the parser's unavailable CSS fact policy,
/// not a host palette/profile certificate. Generated units have no lexical atoms.
#[derive(Clone, Copy)]
pub struct HtmlParseColorComputation<'a> {
    history: &'a FrozenHistory,
    value: &'a provenance::ColorComputation,
}
impl<'a> HtmlParseColorComputation<'a> {
    pub fn source(&self) -> provenance::HtmlParseColorValueSource {
        self.value.source
    }
    pub fn colorspace(&self) -> surgeist_html::attributes::input_colorspace::HtmlInputColorSpace {
        self.value.space
    }
    pub fn alpha_present(&self) -> bool {
        self.value
            .operands
            .get(provenance::HtmlParseColorOperandRole::Alpha)
            .is_some()
    }
    pub fn standard_failure(&self) -> Option<surgeist_css::color::CssColorStandardFailure> {
        self.value.standard_failure
    }
    pub fn environment(&self) -> surgeist_css::color::CssColorFactIdentity {
        self.value.environment
    }
    pub fn value(&self) -> surgeist_color::Color {
        self.value.value
    }
    pub fn original_group(
        &self,
    ) -> Result<Option<HtmlParseOriginGroup<'a>>, HtmlParseUnresolvedReference> {
        self.value
            .original
            .map(|edge| group_of(self.history, edge.group))
            .transpose()
    }
    pub fn operand_group(
        &self,
        role: provenance::HtmlParseColorOperandRole,
    ) -> Result<Option<HtmlParseOriginGroup<'a>>, HtmlParseUnresolvedReference> {
        self.value
            .operands
            .copy(role)
            .map(|edge| group_of(self.history, edge.group))
            .transpose()
    }
}
impl<'a> HtmlParseCause<'a> {
    pub(super) fn qualified(history: &'a FrozenHistory, cause: &'a ConstructionCause) -> Self {
        Self { history, cause }
    }

    pub fn color_computation(&self) -> Option<HtmlParseColorComputation<'a>> {
        match self.cause.computation()? {
            provenance::InputValueComputation::Color(value) => Some(HtmlParseColorComputation {
                history: self.history,
                value,
            }),
            provenance::InputValueComputation::Range(_) => None,
        }
    }
    pub fn range_computation(&self) -> Option<HtmlParseRangeComputation<'a>> {
        match self.cause.computation()? {
            provenance::InputValueComputation::Range(value) => Some(HtmlParseRangeComputation {
                history: self.history,
                value,
            }),
            provenance::InputValueComputation::Color(_) => None,
        }
    }
    pub fn operation(&self) -> provenance::HtmlParseConstructionOperation {
        self.cause.operation()
    }
    pub fn source(&self) -> Option<&'a crate::HtmlSourceRange> {
        self.cause.source()
    }
    pub fn rule(&self) -> Option<provenance::HtmlParserRule> {
        self.cause.rule()
    }
    pub fn token(&self) -> Option<evidence::HtmlParseTokenFamily> {
        self.cause.family()
    }
    fn linked(&self, index: Option<usize>) -> Result<Option<Self>, HtmlParseUnresolvedReference> {
        index.map(|i| cause_of(self.history, i)).transpose()
    }
    pub fn original_cause(&self) -> Result<Option<Self>, HtmlParseUnresolvedReference> {
        self.linked(self.cause.original_cause())
    }
    pub fn trigger(&self) -> Result<Option<Self>, HtmlParseUnresolvedReference> {
        self.linked(self.cause.trigger())
    }
    pub fn original_node(&self) -> Result<Option<HtmlNode<'a>>, HtmlParseUnresolvedReference> {
        self.cause
            .original_node()
            .map(|i| self.history.native_node(i))
            .transpose()
    }
    pub fn intended_parent(&self) -> Result<Option<HtmlNode<'a>>, HtmlParseUnresolvedReference> {
        self.cause
            .intended_parent()
            .map(|i| self.history.native_node(i))
            .transpose()
    }
    pub fn original_group(
        &self,
    ) -> Result<Option<HtmlParseOriginGroup<'a>>, HtmlParseUnresolvedReference> {
        self.cause
            .original()
            .map(|i| group_of(self.history, i))
            .transpose()
    }
    pub fn incoming_group(
        &self,
    ) -> Result<Option<HtmlParseOriginGroup<'a>>, HtmlParseUnresolvedReference> {
        self.cause
            .incoming()
            .map(|i| group_of(self.history, i))
            .transpose()
    }
    pub fn captured(&self) -> Option<HtmlParseCapturedCause<'a>> {
        self.cause
            .captured()
            .map(|value| HtmlParseCapturedCause { value })
    }
    pub fn retired_destination(
        &self,
    ) -> Result<Option<HtmlParseRetiredDestination<'a>>, HtmlParseUnresolvedReference> {
        let Some(location) = self.cause.retired_destination() else {
            return Ok(None);
        };
        if location == FieldRef::Detached {
            return Ok(Some(HtmlParseRetiredDestination::Detached));
        }
        let (node, field) = field_of(self.history, location)?;
        Ok(Some(HtmlParseRetiredDestination::Field { node, field }))
    }
}
#[derive(Clone, Copy)]
pub struct HtmlParseCapturedCause<'a> {
    value: &'a provenance::CapturedConstructionContext,
}
impl<'a> HtmlParseCapturedCause<'a> {
    pub fn capture(&self) -> &'a crate::HtmlCaptureId {
        &self.value.capture
    }
    pub fn host_revision(&self) -> &'a crate::HtmlHostFactsRevision {
        &self.value.host_revision
    }
    pub fn context(&self) -> &'a crate::HtmlCaptureElementId {
        &self.value.context
    }
    pub fn target(&self) -> &'a crate::HtmlFragmentTarget {
        &self.value.target
    }
    pub fn context_document(&self) -> &'a crate::HtmlCaptureDocumentId {
        &self.value.context_document
    }
    pub fn target_document(&self) -> &'a crate::HtmlCaptureDocumentId {
        &self.value.target_document
    }
    pub fn role(&self) -> provenance::HtmlParseCapturedCauseRole {
        self.value.role
    }
}
#[derive(Clone, Copy)]
pub struct HtmlParseLexicalOccurrence<'a> {
    history: &'a FrozenHistory,
    value: &'a provenance::LexicalOccurrence,
}
impl<'a> HtmlParseLexicalOccurrence<'a> {
    pub(crate) fn new(
        history: &'a FrozenHistory,
        value: &'a provenance::LexicalOccurrence,
    ) -> Self {
        Self { history, value }
    }
    pub fn kind(&self) -> provenance::HtmlParseLexicalKind {
        self.value.kind
    }
    pub fn disposition(&self) -> provenance::HtmlParseLexicalDisposition {
        self.value.disposition
    }
    pub fn source(&self) -> &'a crate::HtmlSourceRange {
        &self.value.source
    }
    pub fn parent(&self) -> Result<Option<Self>, HtmlParseUnresolvedReference> {
        self.value
            .parent
            .map(|i| occurrence_of(self.history, i))
            .transpose()
    }
    pub fn cause(&self) -> Result<HtmlParseCause<'a>, HtmlParseUnresolvedReference> {
        cause_of(self.history, self.value.cause())
    }
    pub fn data_units(&self) -> usize {
        self.value.data_units()
    }
}
#[derive(Clone, Copy)]
pub struct HtmlParseLexicalPart<'a> {
    history: &'a FrozenHistory,
    value: &'a provenance::LexicalPart,
}
impl<'a> HtmlParseLexicalPart<'a> {
    pub(crate) fn new(history: &'a FrozenHistory, value: &'a provenance::LexicalPart) -> Self {
        Self { history, value }
    }
    pub fn role(&self) -> provenance::HtmlParseLexicalRole {
        self.value.role
    }
    pub fn source(&self) -> &'a crate::HtmlSourceRange {
        &self.value.source
    }
    pub fn start(&self) -> usize {
        self.value.start
    }
    pub fn end(&self) -> usize {
        self.value.end
    }
    pub fn origin_kind(&self) -> Option<crate::parser::tokenizer::token::HtmlParseOriginKind> {
        self.value.kind
    }
    pub fn occurrence(
        &self,
    ) -> Result<HtmlParseLexicalOccurrence<'a>, HtmlParseUnresolvedReference> {
        occurrence_of(self.history, self.value.occurrence)
    }
}
#[derive(Clone, Copy)]
pub struct HtmlParseOmission<'a> {
    value: &'a provenance::LexicalOmission,
}
impl<'a> HtmlParseOmission<'a> {
    pub(crate) fn new(value: &'a provenance::LexicalOmission) -> Self {
        Self { value }
    }
    pub fn source(&self) -> &'a crate::HtmlSourceRange {
        &self.value.source
    }
    pub fn start(&self) -> usize {
        self.value.start
    }
    pub fn end(&self) -> usize {
        self.value.end
    }
    pub fn origin_kind(&self) -> crate::parser::tokenizer::token::HtmlParseOriginKind {
        self.value.origin
    }
    pub fn kind(&self) -> provenance::HtmlParseOmissionKind {
        self.value.kind
    }
    pub fn rule(&self) -> provenance::HtmlParserRule {
        self.value.rule
    }
}
pub struct HtmlParseReverseCursor<'a> {
    history: &'a FrozenHistory,
    cursor: provenance::reverse::ReverseCursor<'a>,
    fault: Option<HtmlParseUnresolvedReference>,
}
pub enum HtmlParseReverseStep<'a> {
    Match(HtmlParseAssociation<'a>),
    /// A row that does not overlap the query.
    Skipped,
    Complete,
    Cancelled,
    /// A reference the report holds could not be resolved. The cursor stays
    /// faulted, so a caller that continues is told again.
    Fault(HtmlParseUnresolvedReference),
}
impl<'a> HtmlParseReverseCursor<'a> {
    pub fn advance(&mut self) -> HtmlParseReverseStep<'a> {
        if let Some(fault) = self.fault {
            return HtmlParseReverseStep::Fault(fault);
        }
        match self.cursor.advance() {
            provenance::reverse::ReverseStep::Match(row) => {
                match HtmlParseAssociation::new(self.history, row) {
                    Ok(row) => HtmlParseReverseStep::Match(row),
                    Err(fault) => {
                        self.fault = Some(fault);
                        HtmlParseReverseStep::Fault(fault)
                    }
                }
            }
            provenance::reverse::ReverseStep::Skipped => HtmlParseReverseStep::Skipped,
            provenance::reverse::ReverseStep::Complete => HtmlParseReverseStep::Complete,
            provenance::reverse::ReverseStep::Cancelled => HtmlParseReverseStep::Cancelled,
        }
    }
}

pub struct HtmlParseHistory<'a> {
    history: &'a FrozenHistory,
    view: HistoryView<'a>,
}
pub struct HtmlParseHistoryHeader<'a> {
    pub units: usize,
    pub cause: HtmlParseCause<'a>,
    pub retirement: Option<HtmlParseCause<'a>>,
    pub body: history::HtmlParseHistoryBody<'a>,
}
impl<'a> HtmlParseHistory<'a> {
    pub(crate) fn new(history: &'a FrozenHistory) -> Self {
        Self {
            history,
            view: history.view(),
        }
    }
    pub fn len(&self) -> usize {
        self.view.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn group(&self, index: usize) -> Option<HtmlParseOriginGroup<'a>> {
        self.view.group(index)
    }
    pub fn header(
        &self,
        group: HtmlParseOriginGroup<'_>,
    ) -> Result<HtmlParseHistoryHeader<'a>, history::HtmlParseHistoryError> {
        let h = self.view.header(group)?;
        Ok(HtmlParseHistoryHeader {
            units: h.units,
            cause: HtmlParseCause {
                history: self.history,
                cause: h.cause,
            },
            retirement: h.retirement.map(|cause| HtmlParseCause {
                history: self.history,
                cause,
            }),
            body: h.body,
        })
    }
    /// Starts the expansion of `group`. The scratch must hold `index + 1` frames of 16 bytes,
    /// where `index` is the position of the group among the [`len`](Self::len) groups: a group
    /// created late needs scratch in proportion to the groups before it, up to `16 * len()`
    /// bytes, however shallow its own history is. A shorter scratch is refused with
    /// `InsufficientScratch`, which names the length required.
    pub fn cursor<'s>(
        &'s self,
        group: HtmlParseOriginGroup<'_>,
        scratch: &'s mut [history::HtmlParseHistoryFrame],
        cancellation: Option<&'s crate::HtmlParseCancellation>,
    ) -> Result<history::HtmlParseHistoryCursor<'a, 's>, history::HtmlParseHistoryError> {
        self.view.cursor(group, scratch, cancellation)
    }
}

impl<'a> report::HtmlParseMetadata<'a> {
    /// Actual independent cryptographic slot, including initialized empty.
    pub fn cryptographic_nonce(&self) -> Option<&'a DocumentString> {
        self.nonce()
    }
    pub fn input_flags(&self) -> Option<&'a surgeist_html::elements::input::HtmlInputFlags> {
        self.input().map(|(s, _)| &s.flags)
    }
    pub fn input_raw_value(&self) -> Option<&'a DocumentString> {
        self.input().map(|(_, v)| v)
    }
    /// Actual produced value count for a multiple Email input.
    /// Non-input records, other input kinds, and single Email inputs return None.
    /// An empty multiple Email value retains Some(0), distinct from whitespace-only Some(1).
    pub fn input_email_value_count(&self) -> Option<usize> {
        self.input().and_then(|(state, _)| state.email_value_count)
    }
    /// Established actual File resources. Other input states have no File view.
    pub fn input_file_selection(
        &self,
    ) -> Option<surgeist_html::elements::input::HtmlInputFileSelection> {
        self.input().and_then(|(state, _)| {
            (state.flags.input_type() == surgeist_html::attributes::HtmlInputType::File
                && state.selected_files_empty)
                .then_some(surgeist_html::elements::input::HtmlInputFileSelection::Empty)
        })
    }
    /// Initial selected coordinates of actual Image construction, not geometry.
    pub fn input_image_coordinate(
        &self,
    ) -> Option<surgeist_html::elements::input::HtmlInputImageCoordinate> {
        self.input().and_then(|(state, _)| match state.image {
            input_image::ImageState::Last { coordinate, .. }
                if state.flags.input_type() == surgeist_html::attributes::HtmlInputType::Image =>
            {
                Some(coordinate)
            }
            _ => None,
        })
    }
    /// Last source observed while Image, including specified empty versus absent.
    /// None means this actual Input has never entered Image.
    pub fn input_image_source(&self) -> Option<HtmlInputImageSource<'a>> {
        self.input().and_then(|(state, _)| match state.image {
            input_image::ImageState::NeverEntered => None,
            input_image::ImageState::Last { source: None, .. } => {
                Some(HtmlInputImageSource::Absent)
            }
            input_image::ImageState::Last {
                source: Some(_), ..
            } => self
                .input_image_source_value()
                .map(HtmlInputImageSource::Present),
        })
    }
    pub fn input_cursor(&self) -> Option<usize> {
        self.input().map(|(s, _)| s.cursor)
    }
    pub fn input_selected_files_empty(&self) -> Option<bool> {
        self.input().map(|(s, _)| s.selected_files_empty)
    }
    pub fn input_creation_writeback(&self) -> Option<bool> {
        self.input().map(|(s, _)| s.value_rewritten_during_creation)
    }
    pub fn textarea_flags(&self) -> Option<&'a surgeist_html::elements::form::HtmlTextareaFlags> {
        self.textarea().map(|(s, _)| &s.flags)
    }
    pub fn textarea_raw_value(&self) -> Option<&'a DocumentString> {
        self.textarea().map(|(_, v)| v)
    }
    pub fn output_default_override(&self) -> Option<HtmlParseOutputOverride> {
        self.output().map(|s| s.default_value_override)
    }
    pub fn select_flags(&self) -> Option<&'a surgeist_html::elements::form::HtmlSelectFlags> {
        self.select().map(|s| &s.flags)
    }
    pub fn option_flags(&self) -> Option<&'a surgeist_html::elements::form::HtmlOptionFlags> {
        self.option().map(|s| &s.flags)
    }
    pub fn selectedcontent_disabled(&self) -> Option<bool> {
        self.selectedcontent().map(|s| s.disabled)
    }
    pub fn form_kind(&self) -> Option<HtmlNativeFormAssociationKind> {
        self.form().map(|(s, _)| s.kind)
    }
    pub fn form_owner(&self) -> Option<Option<HtmlNode<'a>>> {
        self.form().map(|(_, n)| n)
    }
    pub fn form_parser_inserted(&self) -> Option<bool> {
        self.form().map(|(s, _)| s.parser_inserted)
    }
    pub fn form_evidence(&self) -> Option<HtmlParseFormEvidence> {
        self.form().map(|(s, _)| s.evidence)
    }
    pub fn script_state(&self) -> Option<HtmlParseScript<'a>> {
        self.script().map(|state| HtmlParseScript { state })
    }
    pub fn svg_script_processed(&self) -> Option<bool> {
        self.svg_script().map(|s| s.flags.already_processed())
    }
    pub fn svg_script_origin(&self) -> Option<svg::HtmlParseSvgScriptOrigin> {
        self.svg_script().map(|s| s.origin)
    }
}
pub struct HtmlParseScript<'a> {
    state: &'a ScriptMetadata,
}
impl HtmlParseScript<'_> {
    pub fn force_async(&self) -> bool {
        self.state.flags.force_async()
    }
    pub fn already_started(&self) -> bool {
        self.state.flags.already_started()
    }
    pub fn script_type(&self) -> Option<HtmlScriptKind> {
        self.state.script_type
    }
    pub fn external_file(&self) -> bool {
        self.state.flags.external_file()
    }
    pub fn ready_to_be_parser_executed(&self) -> bool {
        self.state.flags.ready_to_be_parser_executed()
    }
    pub fn delaying_load(&self) -> bool {
        self.state.flags.delaying_load()
    }
    pub fn result(&self) -> HtmlScriptResult {
        self.state.result
    }
    pub fn steps_when_result_ready(&self) -> Option<HtmlScriptReadySteps> {
        self.state.steps_when_result_ready
    }
}

/// Source-specific immutable Image history; present empty and absence differ.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputImageSource<'a> {
    Absent,
    Present(&'a DocumentString),
}
