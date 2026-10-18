//! Canonical structural View Transitions state. Resources and phase timing are
//! runtime-owned; CSSOM identities here record admitted associations, not effects.
use std::sync::Arc;

use surgeist_css::CssViewTransitionIdent;
use surgeist_cssom::{CssomRevision, CssomRuleId, CssomSheetId};

use crate::batch::BatchIdentity;
use crate::{DocumentId, DomDocumentStamp, DomError, DomOwnerId, DomSnapshot, NodeId};
mod capture;
pub use capture::{
    DomPreparedViewTransitionRules, DomPreparedViewTransitionSheet, DomViewTransitionCaptureLimits,
};

/// Owner-qualified, never-reissued transition generation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomViewTransitionId {
    pub(crate) owner: DomOwnerId,
    pub(crate) generation: u64,
}
impl DomViewTransitionId {
    pub fn owner(self) -> DomOwnerId {
        self.owner
    }
}
/// Never-reissued record generation within one transition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomViewTransitionRecordId {
    pub(crate) transition: DomViewTransitionId,
    pub(crate) generation: u64,
}
impl DomViewTransitionRecordId {
    pub fn transition(self) -> DomViewTransitionId {
        self.transition
    }
}
/// A batch-qualified transition ticket; it contains no published identity.
#[derive(Clone, Debug)]
pub struct CreatedViewTransition {
    pub(crate) owner: Arc<BatchIdentity>,
    pub(crate) index: usize,
}
/// A batch-qualified participant ticket, separate from node/transition tickets.
#[derive(Clone, Debug)]
pub struct CreatedViewTransitionRecord {
    pub(crate) owner: Arc<BatchIdentity>,
    pub(crate) index: usize,
}
#[derive(Clone, Debug)]
pub enum ViewTransitionOperand {
    Existing(DomViewTransitionId),
    Created(CreatedViewTransition),
}
impl From<DomViewTransitionId> for ViewTransitionOperand {
    fn from(value: DomViewTransitionId) -> Self {
        Self::Existing(value)
    }
}
impl From<&CreatedViewTransition> for ViewTransitionOperand {
    fn from(value: &CreatedViewTransition) -> Self {
        Self::Created(value.clone())
    }
}
#[derive(Clone, Debug)]
pub enum ViewTransitionRecordOperand {
    Existing(DomViewTransitionRecordId),
    Created(CreatedViewTransitionRecord),
}
impl From<DomViewTransitionRecordId> for ViewTransitionRecordOperand {
    fn from(value: DomViewTransitionRecordId) -> Self {
        Self::Existing(value)
    }
}
impl From<&CreatedViewTransitionRecord> for ViewTransitionRecordOperand {
    fn from(value: &CreatedViewTransitionRecord) -> Self {
        Self::Created(value.clone())
    }
}
/// Declared structure only. Old does not certify an image; new is an actual
/// canonical Element identity, without computed participation/readiness claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomViewTransitionBranches {
    Pending,
    Old,
    New(NodeId),
    OldAndNew(NodeId),
}
impl DomViewTransitionBranches {
    pub fn has_old(self) -> bool {
        matches!(self, Self::Old | Self::OldAndNew(_))
    }
    pub fn new_element(self) -> Option<NodeId> {
        match self {
            Self::New(id) | Self::OldAndNew(id) => Some(id),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DomViewTransitionPseudoKind {
    Root,
    Group,
    ImagePair,
    Old,
    New,
}
/// An owned generated subject, distinct from every ordinary NodeId.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomViewTransitionPseudoId {
    pub(crate) transition: DomViewTransitionId,
    pub(crate) record: Option<DomViewTransitionRecordId>,
    pub(crate) kind: DomViewTransitionPseudoKind,
}
impl DomViewTransitionPseudoId {
    pub fn transition(self) -> DomViewTransitionId {
        self.transition
    }
    pub fn kind(self) -> DomViewTransitionPseudoKind {
        self.kind
    }
}
/// Admitted CSS-element subject domains. Other pseudo families need their real
/// future canonical identities; no generic or guessed pseudo handle is exposed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomViewTransitionSubject {
    Element(NodeId),
    Pseudo(DomViewTransitionPseudoId),
}
/// Zero, one or two ordinary CSSStyleRule identities, in old-before-new order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum DomViewTransitionImageRules {
    #[default]
    Empty,
    Old(CssomRuleId),
    New(CssomRuleId),
    OldThenNew {
        old: CssomRuleId,
        new: CssomRuleId,
    },
}
/// Association requests. DOM validates actual CSSOM identities, kinds, direct
/// membership and ordering. #383/#1058 own rule content and installation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DomViewTransitionRuleAssociations {
    pub group_keyframes: Option<CssomRuleId>,
    pub group_animation_name: Option<CssomRuleId>,
    pub group_styles: Option<CssomRuleId>,
    pub image_pair_isolation: Option<CssomRuleId>,
    pub image_animation: DomViewTransitionImageRules,
}
/// Exact historical CSSOM qualification of one admitted association group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomViewTransitionRules {
    pub(crate) revision: CssomRevision,
    pub(crate) associations: DomViewTransitionRuleAssociations,
}
impl DomViewTransitionRules {
    pub fn revision(&self) -> &CssomRevision {
        &self.revision
    }
    pub fn associations(&self) -> &DomViewTransitionRuleAssociations {
        &self.associations
    }
}
/// A real sheet binding. Its revision records admission, not globally current
/// CSSOM state or UA-origin/document installation evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomViewTransitionSheet {
    pub(crate) id: CssomSheetId,
    pub(crate) revision: CssomRevision,
}
impl DomViewTransitionSheet {
    pub fn id(&self) -> &CssomSheetId {
        &self.id
    }
    pub fn revision(&self) -> &CssomRevision {
        &self.revision
    }
}
/// Source slots after explicit initialization with a real CSSOM sheet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomDocumentViewTransitions {
    pub(crate) init_ordinal: usize,
    pub(crate) document: DocumentId,
    pub(crate) sheet: DomViewTransitionSheet,
    pub(crate) active: Option<DomViewTransitionId>,
    pub(crate) suppressed: bool,
    pub(crate) show: bool,
}
impl DomDocumentViewTransitions {
    pub fn sheet(&self) -> &DomViewTransitionSheet {
        &self.sheet
    }
    pub fn active(&self) -> Option<DomViewTransitionId> {
        self.active
    }
    pub fn rendering_suppressed(&self) -> bool {
        self.suppressed
    }
    pub fn show_tree(&self) -> bool {
        self.show
    }
}
/// Structural identity rejection; no CSSOM mutation has been attempted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomViewTransitionCssomError {
    ForeignOwner,
    MissingObject,
    WrongKind,
    WrongSheet,
    DuplicateRule,
    RuleOrder,
    StaleRevision,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Record {
    pub id: DomViewTransitionRecordId,
    pub name: Arc<CssViewTransitionIdent>,
    pub branches: DomViewTransitionBranches,
    pub rules: Option<DomViewTransitionRules>,
    pub captured: [bool; 4],
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Transition {
    pub id: DomViewTransitionId,
    pub document: DocumentId,
    pub records: crate::store::Bank<Record>,
    pub name_units: usize,
    pub pseudos: usize,
    pub setup: bool,
    pub root_captured: bool,
    pub next_record: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DocumentTransitions {
    pub document: DocumentId,
    pub ids: crate::store::Bank<DomViewTransitionId>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ViewTransitions {
    pub documents: crate::store::Bank<DomDocumentViewTransitions>,
    pub transitions: crate::store::Bank<Transition>,
    pub document_transitions: crate::store::Bank<DocumentTransitions>,
    pub records: usize,
    pub pseudos: usize,
    pub retained_bytes: usize,
    pub retained_units: usize,
    pub next_transition: u64,
    pub next_document: usize,
}
impl ViewTransitions {
    pub fn new() -> Self {
        Self {
            documents: crate::store::Bank::default(),
            transitions: crate::store::Bank::default(),
            document_transitions: crate::store::Bank::default(),
            records: 0,
            pseudos: 0,
            retained_bytes: 0,
            retained_units: 0,
            next_transition: 0,
            next_document: 0,
        }
    }
    pub fn transition(
        &self,
        owner: DomOwnerId,
        id: DomViewTransitionId,
    ) -> Result<&Transition, DomError> {
        if id.owner != owner {
            return Err(DomError::ForeignOwner);
        }
        self.transitions
            .get(usize::try_from(id.generation).map_err(|_| DomError::RepresentationExhausted)?)
            .filter(|t| t.id == id)
            .ok_or(DomError::StaleViewTransition)
    }
    pub fn pseudo(
        &self,
        owner: DomOwnerId,
        id: DomViewTransitionPseudoId,
    ) -> Result<(&Transition, Option<&Record>), DomError> {
        let transition = self.transition(owner, id.transition)?;
        let record = if let Some(record) = id.record {
            let record = transition
                .records
                .get(
                    usize::try_from(record.generation)
                        .map_err(|_| DomError::RepresentationExhausted)?,
                )
                .filter(|r| r.id == record)
                .ok_or(DomError::StaleViewTransitionRecord)?;
            if !transition.setup || !record_has_pseudo(record.branches, id.kind) {
                return Err(DomError::StaleViewTransitionPseudo);
            }
            Some(record)
        } else if id.kind == DomViewTransitionPseudoKind::Root {
            None
        } else {
            return Err(DomError::StaleViewTransitionPseudo);
        };
        Ok((transition, record))
    }
}
pub(crate) fn record_has_pseudo(
    branches: DomViewTransitionBranches,
    kind: DomViewTransitionPseudoKind,
) -> bool {
    use DomViewTransitionPseudoKind::*;
    match kind {
        Root => false,
        Group | ImagePair => true,
        Old => branches.has_old(),
        New => branches.new_element().is_some(),
    }
}
pub(crate) fn flag_index(kind: DomViewTransitionPseudoKind) -> usize {
    match kind {
        DomViewTransitionPseudoKind::Group => 0,
        DomViewTransitionPseudoKind::ImagePair => 1,
        DomViewTransitionPseudoKind::Old => 2,
        DomViewTransitionPseudoKind::New => 3,
        DomViewTransitionPseudoKind::Root => unreachable!("root has its own flag"),
    }
}
pub(crate) fn pseudo_id(
    t: DomViewTransitionId,
    r: Option<DomViewTransitionRecordId>,
    kind: DomViewTransitionPseudoKind,
) -> DomViewTransitionPseudoId {
    DomViewTransitionPseudoId {
        transition: t,
        record: r,
        kind,
    }
}
/// Borrowed exact-revision canonical transition; keeping the snapshot keeps data.
#[derive(Clone, Copy, Debug)]
pub struct DomViewTransition<'a> {
    pub(crate) snapshot: &'a DomSnapshot,
    pub(crate) value: &'a Transition,
}
impl<'a> DomViewTransition<'a> {
    pub fn id(self) -> DomViewTransitionId {
        self.value.id
    }
    pub fn stamp(self) -> DomDocumentStamp {
        self.snapshot
            .document(self.value.document)
            .expect("stored document")
            .stamp()
    }
    pub fn root(self) -> DomViewTransitionPseudoId {
        pseudo_id(self.id(), None, DomViewTransitionPseudoKind::Root)
    }
    pub fn is_setup(self) -> bool {
        self.value.setup
    }
    pub fn records(self) -> impl ExactSizeIterator<Item = DomViewTransitionRecord<'a>> {
        TransitionRecords {
            transition: self,
            records: self.value.records.iter(),
            remaining: self.value.records.record_count(),
        }
    }
}
struct TransitionRecords<'a> {
    transition: DomViewTransition<'a>,
    records: crate::store::BankIter<'a, Record>,
    remaining: usize,
}
impl<'a> Iterator for TransitionRecords<'a> {
    type Item = DomViewTransitionRecord<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let (_, value) = self.records.next()?;
        self.remaining -= 1;
        Some(DomViewTransitionRecord {
            transition: self.transition,
            value,
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl ExactSizeIterator for TransitionRecords<'_> {}
#[derive(Clone, Copy, Debug)]
pub struct DomViewTransitionRecord<'a> {
    pub(crate) transition: DomViewTransition<'a>,
    pub(crate) value: &'a Record,
}
impl<'a> DomViewTransitionRecord<'a> {
    pub fn id(self) -> DomViewTransitionRecordId {
        self.value.id
    }
    pub fn stamp(self) -> DomDocumentStamp {
        self.transition.stamp()
    }
    pub fn name(self) -> &'a CssViewTransitionIdent {
        &self.value.name
    }
    pub fn branches(self) -> DomViewTransitionBranches {
        self.value.branches
    }
    /// None is never assigned; a retained destroyed identity instead returns
    /// StaleNode, and a historical snapshot can still resolve its old Element.
    pub fn resolve_new_element(self) -> Result<Option<crate::DomNode<'a>>, DomError> {
        self.value
            .branches
            .new_element()
            .map(|id| self.transition.snapshot.node(id))
            .transpose()
    }
    pub fn rules(self) -> Option<&'a DomViewTransitionRules> {
        self.value.rules.as_ref()
    }
    pub fn pseudo(self, kind: DomViewTransitionPseudoKind) -> Option<DomViewTransitionPseudoId> {
        (self.transition.value.setup && record_has_pseudo(self.value.branches, kind))
            .then(|| pseudo_id(self.transition.id(), Some(self.id()), kind))
    }
}
/// Complete generated relation facts, with a mandatory actual current origin.
/// These facts do not certify resources, painting or CSSOM publication.
#[derive(Clone, Copy, Debug)]
pub struct DomViewTransitionPseudo<'a> {
    pub(crate) transition: DomViewTransition<'a>,
    pub(crate) record: Option<&'a Record>,
    pub(crate) id: DomViewTransitionPseudoId,
    pub(crate) origin: NodeId,
}
impl<'a> DomViewTransitionPseudo<'a> {
    pub fn id(self) -> DomViewTransitionPseudoId {
        self.id
    }
    pub fn stamp(self) -> DomDocumentStamp {
        self.transition.stamp()
    }
    pub fn originating_element(self) -> NodeId {
        self.origin
    }
    pub fn name(self) -> Option<&'a CssViewTransitionIdent> {
        self.record.map(|r| r.name.as_ref())
    }
    pub fn parent(self) -> Option<DomViewTransitionPseudoId> {
        use DomViewTransitionPseudoKind::*;
        let kind = match self.id.kind {
            Root => return None,
            Group => Root,
            ImagePair => Group,
            Old | New => ImagePair,
        };
        Some(pseudo_id(
            self.id.transition,
            if kind == Root { None } else { self.id.record },
            kind,
        ))
    }
    /// Originating pseudo is the parent within this pseudo tree.
    pub fn originating_pseudo(self) -> Option<DomViewTransitionPseudoId> {
        self.parent()
    }
    pub fn captured(self) -> bool {
        self.record
            .map_or(self.transition.value.root_captured, |r| {
                r.captured[flag_index(self.id.kind)]
            })
    }
    pub fn children(self) -> DomViewTransitionPseudoChildren<'a> {
        DomViewTransitionPseudoChildren {
            value: self,
            index: 0,
            records: self.transition.value.records.iter(),
        }
    }
}
/// Allocation-free source-order generated-child iterator.
pub struct DomViewTransitionPseudoChildren<'a> {
    value: DomViewTransitionPseudo<'a>,
    index: usize,
    records: crate::store::BankIter<'a, Record>,
}
impl Iterator for DomViewTransitionPseudoChildren<'_> {
    type Item = DomViewTransitionPseudoId;
    fn next(&mut self) -> Option<Self::Item> {
        use DomViewTransitionPseudoKind::*;
        let t = self.value.transition;
        match self.value.id.kind {
            Root if t.value.setup => {
                let (_, r) = self.records.next()?;
                Some(pseudo_id(t.id(), Some(r.id), Group))
            }
            ImagePair => loop {
                let kind = match self.index {
                    0 => Old,
                    1 => New,
                    _ => return None,
                };
                self.index += 1;
                if record_has_pseudo(self.value.record.expect("pair record").branches, kind) {
                    return Some(pseudo_id(t.id(), self.value.id.record, kind));
                }
            },
            Group if self.index == 0 => {
                self.index += 1;
                Some(pseudo_id(t.id(), self.value.id.record, ImagePair))
            }
            _ => None,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum Edit {
    Initialize {
        input: DomPreparedViewTransitionSheet,
    },
    Begin {
        ticket: usize,
    },
    Append {
        transition: ViewTransitionOperand,
        name: Arc<CssViewTransitionIdent>,
        ticket: usize,
    },
    Branches {
        record: ViewTransitionRecordOperand,
        branches: DomViewTransitionBranches,
    },
    Rules {
        record: ViewTransitionRecordOperand,
        input: DomPreparedViewTransitionRules,
    },
    ClearRules(ViewTransitionRecordOperand),
    Captured {
        subject: DomViewTransitionSubject,
        captured: bool,
    },
    Suppression(bool),
    Setup(ViewTransitionOperand),
    Clear(ViewTransitionOperand),
    Release(ViewTransitionOperand),
}
