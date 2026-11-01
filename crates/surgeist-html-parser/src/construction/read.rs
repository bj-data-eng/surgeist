//! Fixed public lenders over the already qualified immutable report owner.
use super::*;
use history::{FrozenHistory, HistoryGroup, HistoryView};
use provenance::{AssociationView, ConstructionCause, NativeAssociation, NativeDestination};

/// The table a reference of a published report named.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnresolvedKind {
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
    /// A retained payload slot, or the leaf value, origins or string it holds.
    Payload,
    /// A node of the captured fragment context.
    CapturedNode,
    /// The fragment context storage of a contextual report.
    Fragment,
    /// A committed diagnostic location.
    Diagnostic,
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
pub struct UnresolvedReference {
    kind: UnresolvedKind,
}
impl UnresolvedReference {
    pub(crate) const fn new(kind: UnresolvedKind) -> Self {
        Self { kind }
    }
    /// The table the reference named.
    pub const fn kind(&self) -> UnresolvedKind {
        self.kind
    }
}
impl std::fmt::Display for UnresolvedReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "a published parse report holds a reference to a {:?} that it cannot resolve",
            self.kind
        )
    }
}
impl std::error::Error for UnresolvedReference {}

/// Qualify only the references used by fixed public readers. The canonical
/// history transfer still owns source/group/leaf qualification and alias checks.
pub(crate) fn prepare_public_readers(
    input: &Option<FrozenArena>,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    let frozen = input.as_ref().ok_or(SessionError::Consumed)?;
    let storage = frozen.storage.as_ref().ok_or(SessionError::Consumed)?;
    c.require_identity(&storage.control)?;
    let records = ConstructionView::Frozen(&storage.view);
    c.charge(WorkCategory::RecordVisit, 1)?;
    let evidence = storage.evidence.as_ref().ok_or(SessionError::WrongToken)?;
    payload_write::<AssociationView<'_>>(c, 1)?;
    let view = evidence.associations()?;
    fn record<T>(c: &mut ParseControl) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, mem::size_of::<T>() as u64)?;
        Ok(())
    }
    fn node(
        records: ConstructionView<'_>,
        c: &mut ParseControl,
        index: usize,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if records.node_at(index).is_none() {
            return Err(SessionError::ForeignNode);
        }
        Ok(())
    }
    fn location(
        storage: &FrozenStorage,
        records: ConstructionView<'_>,
        c: &mut ParseControl,
        location: StringLocation,
    ) -> Result<(), SessionError> {
        match location {
            StringLocation::Detached => Ok(()),
            StringLocation::Text(i)
            | StringLocation::Comment(i)
            | StringLocation::ProcessingInstructionTarget(i)
            | StringLocation::ProcessingInstructionData(i)
            | StringLocation::DoctypeName(i)
            | StringLocation::DoctypePublic(i)
            | StringLocation::DoctypeSystem(i)
            | StringLocation::ElementName(i)
            | StringLocation::Namespace(i) => node(records, c, i),
            StringLocation::AttributeName(slot, index)
            | StringLocation::AttributeValue(slot, index)
            | StringLocation::AttributeNamespace(slot, index)
            | StringLocation::AttributePrefix(slot, index) => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                let Some(RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
                    element: Some(owner),
                    values,
                    ..
                }))) = storage.payloads.values.get(slot)
                else {
                    return Err(SessionError::WrongToken);
                };
                if index >= values.len() {
                    return Err(SessionError::WrongToken);
                };
                node(records, c, *owner)
            }
            StringLocation::PiName(slot, index) | StringLocation::PiValue(slot, index) => {
                c.charge(WorkCategory::RecordVisit, 1)?;
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
        value: StringLocation,
    ) -> Result<(), SessionError> {
        match value {
            StringLocation::AttributeName(slot, ordinal)
            | StringLocation::AttributeValue(slot, ordinal)
            | StringLocation::AttributeNamespace(slot, ordinal)
            | StringLocation::AttributePrefix(slot, ordinal) => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                match storage.payloads.values.get(slot) {
                    Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                        values,
                        original_len: 0,
                        ..
                    }))) if ordinal < values.capacity() => Ok(()),
                    Some(RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
                        values,
                        element,
                        ..
                    }))) if ordinal < values.shared_attributes().capacity() => {
                        if let Some(owner) = element {
                            node(records, c, *owner)?;
                        }
                        Ok(())
                    }
                    _ => Err(SessionError::WrongToken),
                }
            }
            _ => location(storage, records, c, value),
        }
    }
    for index in 0..view.causes_len() {
        record::<ConstructionCause>(c)?;
        let cause = view.cause(index).ok_or(SessionError::WrongToken)?;
        for i in [cause.original_node, cause.intended_parent]
            .into_iter()
            .flatten()
        {
            node(records, c, i)?;
        }
        if let Some(value) = cause.retired_destination {
            if cause.operation != provenance::ConstructionOperation::Retirement {
                return Err(SessionError::WrongToken);
            }
            let group = cause.original.ok_or(SessionError::WrongToken)?;
            record::<provenance::OriginGroup>(c)?;
            if view
                .group(group)
                .is_none_or(|g| g.retirement != Some(index))
            {
                return Err(SessionError::WrongToken);
            }
            // The original issuer established this ordinal before removal or
            // transfer. Capacity is only a retained-storage sanity bound;
            // the canonical retirement relation above proves field history.
            retired_location(storage, records, c, value)?;
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
                if value == StringLocation::Detached {
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
    /// The row at `index`, or `None` past the last row.
    pub fn row(
        &self,
        index: usize,
    ) -> Result<Option<HtmlParseAssociation<'a>>, UnresolvedReference> {
        Ok(self.view.row(index).map(|row| HtmlParseAssociation {
            history: self.history,
            row,
        }))
    }
    pub fn reverse(
        &self,
        source: &'a crate::HtmlSourceRange,
        cancellation: Option<&'a crate::HtmlParseCancellation>,
    ) -> Result<HtmlParseReverseCursor<'a>, provenance::reverse::ReverseQueryError> {
        Ok(HtmlParseReverseCursor {
            history: self.history,
            cursor: self.view.reverse(source, cancellation)?,
        })
    }
}
pub struct HtmlParseAssociation<'a> {
    history: &'a FrozenHistory,
    row: &'a NativeAssociation,
}
impl<'a> HtmlParseAssociation<'a> {
    pub fn source(&self) -> Option<&'a crate::HtmlSourceRange> {
        self.row.source.as_ref()
    }
    pub fn start(&self) -> usize {
        self.row.start
    }
    pub fn end(&self) -> usize {
        self.row.end
    }
    pub fn origin_kind(&self) -> Option<crate::parser::tokenizer::token::OriginKind> {
        self.row.kind
    }
    pub fn relation(&self) -> provenance::reverse::AssociationRelation {
        self.row.relation
    }
    pub fn group(&self) -> Option<HistoryGroup<'a>> {
        self.row.group.and_then(|i| self.history.view().group(i))
    }
    pub fn cause(&self) -> HtmlParseCause<'a> {
        HtmlParseCause {
            history: self.history,
            cause: self
                .history
                .associations()
                .cause(self.row.cause)
                .expect("qualified cause"),
        }
    }
    pub fn destination(&self) -> HtmlParseDestination<'a> {
        destination(self.history, self.row.destination)
    }
}
/// A native destination never exposes an unchecked arena or payload slot.
pub enum HtmlParseDestination<'a> {
    Field {
        node: HtmlNode<'a>,
        field: HtmlParseField,
    },
    Node(HtmlNode<'a>),
    Raw {
        node: HtmlNode<'a>,
        kind: provenance::RawFieldKind,
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
    /// A historical field whose retained container proves an actual native owner.
    /// Its ordinal describes the retired field, not a current attribute index.
    Field {
        node: HtmlNode<'a>,
        field: HtmlParseField,
    },
    /// A historical construction field without a surviving native binding.
    /// The role and ordinal are qualified by this cause and its original group.
    StagedField {
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
fn destination(history: &FrozenHistory, value: NativeDestination) -> HtmlParseDestination<'_> {
    match value {
        NativeDestination::Node(i) => HtmlParseDestination::Node(history.native_node(i)),
        NativeDestination::Raw { node, kind } => HtmlParseDestination::Raw {
            node: history.native_node(node),
            kind,
        },
        NativeDestination::Field(location) => {
            let (node, field) = match location {
                StringLocation::Text(i) => (i, HtmlParseField::Text),
                StringLocation::Comment(i) => (i, HtmlParseField::Comment),
                StringLocation::ProcessingInstructionTarget(i) => {
                    (i, HtmlParseField::ProcessingInstructionTarget)
                }
                StringLocation::ProcessingInstructionData(i) => {
                    (i, HtmlParseField::ProcessingInstructionData)
                }
                StringLocation::DoctypeName(i) => (i, HtmlParseField::DoctypeName),
                StringLocation::DoctypePublic(i) => (i, HtmlParseField::DoctypePublic),
                StringLocation::DoctypeSystem(i) => (i, HtmlParseField::DoctypeSystem),
                StringLocation::ElementName(i) => (i, HtmlParseField::ElementName),
                StringLocation::Namespace(i) => (i, HtmlParseField::Namespace),
                StringLocation::AttributeName(i, a) => {
                    (attribute_node(history, i), HtmlParseField::AttributeName(a))
                }
                StringLocation::AttributeValue(i, a) => (
                    attribute_node(history, i),
                    HtmlParseField::AttributeValue(a),
                ),
                StringLocation::AttributeNamespace(i, a) => (
                    attribute_node(history, i),
                    HtmlParseField::AttributeNamespace(a),
                ),
                StringLocation::AttributePrefix(i, a) => (
                    attribute_node(history, i),
                    HtmlParseField::AttributePrefix(a),
                ),
                StringLocation::PiName(i, a) => (
                    pi_node(history, i),
                    HtmlParseField::ProcessingInstructionAttributeName(a),
                ),
                StringLocation::PiValue(i, a) => (
                    pi_node(history, i),
                    HtmlParseField::ProcessingInstructionAttributeValue(a),
                ),
                StringLocation::Detached => {
                    unreachable!("detached payload is not a native destination")
                }
            };
            HtmlParseDestination::Field {
                node: history.native_node(node),
                field,
            }
        }
        NativeDestination::Lexical { occurrence, part } => {
            let view = history.associations();
            HtmlParseDestination::Lexical {
                occurrence: HtmlParseLexicalOccurrence {
                    history,
                    value: view
                        .lexical_occurrence(occurrence)
                        .expect("qualified occurrence"),
                },
                part: part.map(|i| HtmlParseLexicalPart {
                    history,
                    value: view.lexical_part(i).expect("qualified part"),
                }),
            }
        }
        NativeDestination::Omission(i) => HtmlParseDestination::Omission(HtmlParseOmission {
            value: history
                .associations()
                .omission(i)
                .expect("qualified omission"),
        }),
        NativeDestination::LexicalEnd => HtmlParseDestination::LexicalEnd,
        NativeDestination::Cause(i) => HtmlParseDestination::Cause(HtmlParseCause {
            history,
            cause: history.associations().cause(i).expect("qualified cause"),
        }),
    }
}
fn attribute_node(history: &FrozenHistory, slot: usize) -> usize {
    let storage = history.storage();
    let RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
        element: Some(node),
        ..
    })) = &storage.payloads.values[slot]
    else {
        unreachable!("prequalified attribute owner")
    };
    *node
}
fn pi_node(history: &FrozenHistory, slot: usize) -> usize {
    let storage = history.storage();
    let RetainedPayload::PiAttributes {
        node: Some(node), ..
    } = &storage.payloads.values[slot]
    else {
        unreachable!("prequalified PI map owner")
    };
    *node
}
/// Fixed borrowed decisions from the qualified canonical cause owner.
pub struct HtmlParseRangeComputation<'a> {
    history: &'a FrozenHistory,
    value: &'a provenance::RangeComputation,
}
pub struct HtmlParseRangeOperand<'a> {
    history: &'a FrozenHistory,
    value: provenance::RangeOperand,
}
impl<'a> HtmlParseRangeOperand<'a> {
    pub fn kind(&self) -> provenance::RangeOperandKind {
        self.value.kind()
    }
    pub fn group(&self) -> Result<Option<HistoryGroup<'a>>, UnresolvedReference> {
        Ok(self
            .value
            .edge()
            .and_then(|edge| self.history.view().group(edge.group)))
    }
    pub fn cause(&self) -> Result<Option<HtmlParseCause<'a>>, UnresolvedReference> {
        Ok(self.value.edge().map(|edge| HtmlParseCause {
            history: self.history,
            cause: self
                .history
                .associations()
                .cause(edge.cause)
                .expect("qualified Range link"),
        }))
    }
}
impl<'a> HtmlParseRangeComputation<'a> {
    pub fn branch(&self) -> provenance::RangeBranch {
        self.value.branch
    }
    pub fn original_is_fresh_empty(&self) -> bool {
        matches!(self.value.original, provenance::RangeOriginal::FreshEmpty)
    }
    pub fn original_group(&self) -> Result<Option<HistoryGroup<'a>>, UnresolvedReference> {
        Ok(match self.value.original {
            provenance::RangeOriginal::FreshEmpty => None,
            provenance::RangeOriginal::Group(edge) => self.history.view().group(edge.group),
        })
    }
    pub fn original_cause(&self) -> Result<Option<HtmlParseCause<'a>>, UnresolvedReference> {
        Ok(match self.value.original {
            provenance::RangeOriginal::FreshEmpty => None,
            provenance::RangeOriginal::Group(edge) => Some(HtmlParseCause {
                history: self.history,
                cause: self
                    .history
                    .associations()
                    .cause(edge.cause)
                    .expect("qualified Range original"),
            }),
        })
    }
    pub fn operand(&self, role: provenance::RangeOperandRole) -> HtmlParseRangeOperand<'a> {
        HtmlParseRangeOperand {
            history: self.history,
            value: self.value.operands.copy(role),
        }
    }
    pub fn minimum_origin(&self) -> Option<crate::HtmlInputNumericBoundOrigin> {
        self.value.minimum
    }
    pub fn maximum_origin(&self) -> Option<crate::HtmlInputNumericBoundOrigin> {
        self.value.maximum
    }
    pub fn step_origin(&self) -> Option<crate::HtmlInputNumericStepOrigin> {
        self.value.step
    }
    pub fn base_origin(&self) -> Option<crate::HtmlInputNumericStepBaseOrigin> {
        self.value.base
    }
}
pub struct HtmlParseCause<'a> {
    history: &'a FrozenHistory,
    cause: &'a ConstructionCause,
}
/// Borrowed decisions and dependency groups from an admitted Color update.
/// The environment identity denotes the parser's unavailable CSS fact policy,
/// not a host palette/profile certificate. Generated units have no lexical atoms.
pub struct HtmlParseColorComputation<'a> {
    history: &'a FrozenHistory,
    value: &'a provenance::ColorComputation,
}
impl<'a> HtmlParseColorComputation<'a> {
    pub fn source(&self) -> provenance::ColorValueSource {
        self.value.source
    }
    pub fn colorspace(&self) -> crate::HtmlInputColorSpace {
        self.value.space
    }
    pub fn alpha_present(&self) -> bool {
        self.value
            .operands
            .get(provenance::ColorOperandRole::Alpha)
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
    pub fn original_group(&self) -> Result<Option<HistoryGroup<'a>>, UnresolvedReference> {
        Ok(self
            .value
            .original
            .and_then(|edge| self.history.view().group(edge.group)))
    }
    pub fn operand_group(
        &self,
        role: provenance::ColorOperandRole,
    ) -> Result<Option<HistoryGroup<'a>>, UnresolvedReference> {
        Ok(self
            .value
            .operands
            .copy(role)
            .and_then(|edge| self.history.view().group(edge.group)))
    }
}
impl<'a> HtmlParseCause<'a> {
    pub(super) fn qualified(history: &'a FrozenHistory, cause: &'a ConstructionCause) -> Self {
        Self { history, cause }
    }

    pub fn color_computation(&self) -> Option<HtmlParseColorComputation<'a>> {
        match self.cause.computation.as_ref()? {
            provenance::InputValueComputation::Color(value) => Some(HtmlParseColorComputation {
                history: self.history,
                value,
            }),
            provenance::InputValueComputation::Range(_) => None,
        }
    }
    pub fn range_computation(&self) -> Option<HtmlParseRangeComputation<'a>> {
        match self.cause.computation.as_ref()? {
            provenance::InputValueComputation::Range(value) => Some(HtmlParseRangeComputation {
                history: self.history,
                value,
            }),
            provenance::InputValueComputation::Color(_) => None,
        }
    }
    pub fn operation(&self) -> provenance::ConstructionOperation {
        self.cause.operation
    }
    pub fn source(&self) -> Option<&'a crate::HtmlSourceRange> {
        self.cause.source.as_ref()
    }
    pub fn rule(&self) -> Option<provenance::ParserRule> {
        self.cause.rule
    }
    pub fn token(&self) -> Option<evidence::DiagnosticToken> {
        self.cause.family
    }
    fn linked(&self, index: Option<usize>) -> Result<Option<Self>, UnresolvedReference> {
        Ok(index.map(|i| Self {
            history: self.history,
            cause: self
                .history
                .associations()
                .cause(i)
                .expect("qualified cause link"),
        }))
    }
    pub fn original_cause(&self) -> Result<Option<Self>, UnresolvedReference> {
        self.linked(self.cause.original_cause)
    }
    pub fn trigger(&self) -> Result<Option<Self>, UnresolvedReference> {
        self.linked(self.cause.trigger)
    }
    pub fn original_node(&self) -> Result<Option<HtmlNode<'a>>, UnresolvedReference> {
        Ok(self
            .cause
            .original_node
            .map(|i| self.history.native_node(i)))
    }
    pub fn intended_parent(&self) -> Result<Option<HtmlNode<'a>>, UnresolvedReference> {
        Ok(self
            .cause
            .intended_parent
            .map(|i| self.history.native_node(i)))
    }
    pub fn original_group(&self) -> Result<Option<HistoryGroup<'a>>, UnresolvedReference> {
        Ok(self
            .cause
            .original
            .and_then(|i| self.history.view().group(i)))
    }
    pub fn incoming_group(&self) -> Result<Option<HistoryGroup<'a>>, UnresolvedReference> {
        Ok(self
            .cause
            .incoming
            .and_then(|i| self.history.view().group(i)))
    }
    pub fn captured(&self) -> Option<HtmlParseCapturedCause<'a>> {
        self.cause
            .captured
            .as_ref()
            .map(|value| HtmlParseCapturedCause { value })
    }
    pub fn retired_destination(
        &self,
    ) -> Result<Option<HtmlParseRetiredDestination<'a>>, UnresolvedReference> {
        Ok(self.cause.retired_destination.map(|location| {
            if location == StringLocation::Detached {
                return HtmlParseRetiredDestination::Detached;
            }
            let attribute = match location {
                StringLocation::AttributeName(slot, ordinal) => {
                    Some((slot, HtmlParseField::AttributeName(ordinal)))
                }
                StringLocation::AttributeValue(slot, ordinal) => {
                    Some((slot, HtmlParseField::AttributeValue(ordinal)))
                }
                StringLocation::AttributeNamespace(slot, ordinal) => {
                    Some((slot, HtmlParseField::AttributeNamespace(ordinal)))
                }
                StringLocation::AttributePrefix(slot, ordinal) => {
                    Some((slot, HtmlParseField::AttributePrefix(ordinal)))
                }
                _ => None,
            };
            if let Some((slot, field)) = attribute {
                let storage = self.history.storage();
                return match &storage.payloads.values[slot] {
                    RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
                        element: Some(owner),
                        ..
                    })) => HtmlParseRetiredDestination::Field {
                        node: self.history.native_node(*owner),
                        field,
                    },
                    RetainedPayload::Attributes(Some(
                        AttributeStorage::Building { .. }
                        | AttributeStorage::Frozen { element: None, .. },
                    )) => HtmlParseRetiredDestination::StagedField { field },
                    _ => unreachable!("prequalified historical attribute category"),
                };
            }
            let HtmlParseDestination::Field { node, field } =
                destination(self.history, NativeDestination::Field(location))
            else {
                unreachable!("field location")
            };
            HtmlParseRetiredDestination::Field { node, field }
        }))
    }
}
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
    pub fn role(&self) -> provenance::CapturedCauseRole {
        self.value.role
    }
}
pub struct HtmlParseLexicalOccurrence<'a> {
    history: &'a FrozenHistory,
    value: &'a provenance::LexicalOccurrence,
}
impl<'a> HtmlParseLexicalOccurrence<'a> {
    pub fn kind(&self) -> provenance::LexicalKind {
        self.value.kind
    }
    pub fn disposition(&self) -> provenance::LexicalDisposition {
        self.value.disposition
    }
    pub fn source(&self) -> &'a crate::HtmlSourceRange {
        &self.value.source
    }
    pub fn parent(&self) -> Result<Option<Self>, UnresolvedReference> {
        Ok(self.value.parent.map(|i| Self {
            history: self.history,
            value: self
                .history
                .associations()
                .lexical_occurrence(i)
                .expect("qualified parent"),
        }))
    }
    pub fn cause(&self) -> Result<HtmlParseCause<'a>, UnresolvedReference> {
        Ok(HtmlParseCause {
            history: self.history,
            cause: self
                .history
                .associations()
                .cause(self.value.cause())
                .expect("qualified lexical cause"),
        })
    }
    pub fn data_units(&self) -> usize {
        self.value.data_units()
    }
}
pub struct HtmlParseLexicalPart<'a> {
    history: &'a FrozenHistory,
    value: &'a provenance::LexicalPart,
}
impl<'a> HtmlParseLexicalPart<'a> {
    pub fn role(&self) -> provenance::LexicalRole {
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
    pub fn origin_kind(&self) -> Option<crate::parser::tokenizer::token::OriginKind> {
        self.value.kind
    }
    pub fn occurrence(&self) -> Result<HtmlParseLexicalOccurrence<'a>, UnresolvedReference> {
        Ok(HtmlParseLexicalOccurrence {
            history: self.history,
            value: self
                .history
                .associations()
                .lexical_occurrence(self.value.occurrence)
                .expect("qualified occurrence"),
        })
    }
}
pub struct HtmlParseOmission<'a> {
    value: &'a provenance::LexicalOmission,
}
impl<'a> HtmlParseOmission<'a> {
    pub fn source(&self) -> &'a crate::HtmlSourceRange {
        &self.value.source
    }
    pub fn start(&self) -> usize {
        self.value.start
    }
    pub fn end(&self) -> usize {
        self.value.end
    }
    pub fn origin_kind(&self) -> crate::parser::tokenizer::token::OriginKind {
        self.value.origin
    }
    pub fn kind(&self) -> provenance::OmissionKind {
        self.value.kind
    }
    pub fn rule(&self) -> provenance::ParserRule {
        self.value.rule
    }
}
pub struct HtmlParseReverseCursor<'a> {
    history: &'a FrozenHistory,
    cursor: provenance::reverse::ReverseCursor<'a>,
}
pub enum HtmlParseReverseStep<'a> {
    Match(HtmlParseAssociation<'a>),
    /// A row that does not overlap the query.
    Skipped,
    Complete,
    Cancelled,
    /// A reference the report holds could not be resolved. The cursor stays
    /// faulted, so a caller that continues is told again.
    Fault(UnresolvedReference),
}
impl<'a> HtmlParseReverseCursor<'a> {
    pub fn advance(&mut self) -> HtmlParseReverseStep<'a> {
        match self.cursor.advance() {
            provenance::reverse::ReverseStep::Match(row) => {
                HtmlParseReverseStep::Match(HtmlParseAssociation {
                    history: self.history,
                    row,
                })
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
    pub body: history::HistoryBody<'a>,
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
    pub fn group(&self, index: usize) -> Option<HistoryGroup<'a>> {
        self.view.group(index)
    }
    pub fn header(
        &self,
        group: HistoryGroup<'_>,
    ) -> Result<HtmlParseHistoryHeader<'a>, history::HistoryReadError> {
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
    pub fn cursor<'s>(
        &'s self,
        group: HistoryGroup<'_>,
        scratch: &'s mut [history::HistoryFrame],
        cancellation: Option<&'s crate::HtmlParseCancellation>,
    ) -> Result<history::HistoryCursor<'a, 's>, history::HistoryReadError> {
        self.view.cursor(group, scratch, cancellation)
    }
}

impl<'a> report::MetadataRef<'a> {
    /// Actual independent cryptographic slot, including initialized empty.
    pub fn cryptographic_nonce(&self) -> Option<&'a DocumentString> {
        self.nonce()
    }
    pub fn input_flags(&self) -> Option<&'a crate::HtmlInputFlags> {
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
    pub fn input_file_selection(&self) -> Option<crate::HtmlInputFileSelection> {
        self.input().and_then(|(state, _)| {
            (state.flags.input_type() == crate::HtmlInputType::File && state.selected_files_empty)
                .then_some(crate::HtmlInputFileSelection::Empty)
        })
    }
    /// Initial selected coordinates of actual Image construction, not geometry.
    pub fn input_image_coordinate(&self) -> Option<crate::HtmlInputImageCoordinate> {
        self.input().and_then(|(state, _)| match state.image {
            input_image::ImageState::Last { coordinate, .. }
                if state.flags.input_type() == crate::HtmlInputType::Image =>
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
                source: Some(source),
                ..
            } => Some(HtmlInputImageSource::Present(self.raw(source))),
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
    pub fn textarea_flags(&self) -> Option<&'a crate::HtmlTextareaFlags> {
        self.textarea().map(|(s, _)| &s.flags)
    }
    pub fn textarea_raw_value(&self) -> Option<&'a DocumentString> {
        self.textarea().map(|(_, v)| v)
    }
    pub fn output_default_override(&self) -> Option<OutputDefaultValueOverride> {
        self.output().map(|s| s.default_value_override)
    }
    pub fn select_flags(&self) -> Option<&'a crate::HtmlSelectFlags> {
        self.select().map(|s| &s.flags)
    }
    pub fn option_flags(&self) -> Option<&'a crate::HtmlOptionFlags> {
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
    pub fn form_evidence(&self) -> Option<FormAssociationEvidence> {
        self.form().map(|(s, _)| s.evidence)
    }
    pub fn script_state(&self) -> Option<HtmlParseScript<'a>> {
        self.script().map(|state| HtmlParseScript { state })
    }
    pub fn svg_script_processed(&self) -> Option<bool> {
        self.svg_script().map(|s| s.flags.already_processed())
    }
    pub fn svg_script_origin(&self) -> Option<svg::SvgScriptOrigin> {
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
