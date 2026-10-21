//! Canonical native Style state and real CSSOM association provenance.
use crate::{DomNode, DomSourceOperationId, NodeId};
mod cssom_root;
pub(crate) use cssom_root::AccessError as CssomRootAccessError;
pub use cssom_root::{DomStyleCssomRoot, DomStyleCssomRootError};

/// One checked association occurrence at an exact canonical Element generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomStyleAssociationId {
    pub(crate) element: NodeId,
    pub(crate) serial: u64,
}
impl DomStyleAssociationId {
    pub fn element(self) -> NodeId {
        self.element
    }
}

/// A sheet actually associated by the composed CSSOM publication protocol.
/// Identity and revision are producer-issued; association order is DOM-owned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomStyleSheet {
    pub(crate) id: surgeist_cssom::CssomSheetId,
    pub(crate) revision: surgeist_cssom::CssomRevision,
    pub(crate) association: DomStyleAssociationId,
    pub(crate) root: NodeId,
}
impl DomStyleSheet {
    pub fn root(&self) -> NodeId {
        self.root
    }
    pub fn id(&self) -> &surgeist_cssom::CssomSheetId {
        &self.id
    }
    pub fn revision(&self) -> &surgeist_cssom::CssomRevision {
        &self.revision
    }
    pub fn association(&self) -> DomStyleAssociationId {
        self.association
    }
}
/// Availability established by the actual association-processing owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomStyleSheetAvailability {
    Established,
    PendingParsedProcessing,
}
/// Native creation history and its nullable associated sheet. Native cloning
/// initializes this state afresh before copied attributes and children run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomStyleState {
    pub(crate) creation: surgeist_html::HtmlStyleState,
    pub(crate) sheet: Option<DomStyleSheet>,
    pub(crate) next_association: u64,
    pub(crate) availability: DomStyleSheetAvailability,
}
impl DomStyleState {
    pub(crate) fn fresh() -> Self {
        Self {
            creation: surgeist_html::HtmlStyleState::fresh(),
            sheet: None,
            next_association: 0,
            availability: DomStyleSheetAvailability::Established,
        }
    }
    /// Retain actual creation history and the adapter's complete original
    /// association qualification. Only a proved no-effect history is established
    /// without downstream CSSOM processing.
    pub(crate) fn from_parsed(
        creation: surgeist_html::HtmlStyleState,
        availability: DomStyleSheetAvailability,
    ) -> Self {
        Self {
            creation,
            sheet: None,
            next_association: 0,
            availability,
        }
    }
    pub fn association_availability(&self) -> DomStyleSheetAvailability {
        self.availability
    }
    pub fn creation_state(&self) -> surgeist_html::HtmlStyleState {
        self.creation
    }
}
/// One Style in an immutable canonical snapshot.
#[derive(Clone, Copy, Debug)]
pub struct DomStyle<'a> {
    node: DomNode<'a>,
}
impl<'a> DomStyle<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn state(self) -> &'a DomStyleState {
        self.node
            .as_element()
            .expect("checked Style")
            .style_state()
            .expect("checked Style")
    }
    pub fn sheet(self) -> Result<Option<&'a DomStyleSheet>, crate::DomError> {
        self.node.snapshot.require_complete_source()?;
        if self.state().availability == DomStyleSheetAvailability::PendingParsedProcessing {
            return Err(crate::DomError::StyleProcessingUnavailable);
        }
        Ok(self.state().sheet.as_ref())
    }
    /// Membership in the actual current node Document's render-blocking set.
    /// This is not a general render-blocked or environment-match observation.
    pub fn is_render_blocking(self) -> Result<bool, crate::DomError> {
        self.node.snapshot.require_complete_source()?;
        if self.state().availability == DomStyleSheetAvailability::PendingParsedProcessing {
            return Err(crate::DomError::StyleProcessingUnavailable);
        }
        let document = self.node.snapshot.node(self.node.node.document().node())?;
        Ok(document
            .node
            .base_document()
            .is_some_and(|state| state.render_blocking.contains(&self.node.id())))
    }
    /// Native and native-clone creation has no parser Document. A parser-owned
    /// installation must establish its own actual cause and relation.
    pub fn parser_document(self) -> Option<crate::DocumentId> {
        self.node
            .node
            .style_parser_document()
            .map(crate::DocumentId)
    }
}
impl<'a> DomNode<'a> {
    pub fn as_style(self) -> Option<DomStyle<'a>> {
        self.as_element()?.style_state()?;
        Some(DomStyle { node: self })
    }
}

/// The actual ordinary root at this source invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomStyleRootKind {
    Document,
    ShadowRoot,
    Detached,
}
/// Exact source-time inputs. Later mutations cannot rewrite this invocation.
#[derive(Clone, Debug)]
pub struct DomStyleSource {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) ordinal: u64,
    pub(crate) stamp: crate::DomStamp,
    pub(crate) metadata: crate::DomDocumentMetadata,
    pub(crate) root_kind: DomStyleRootKind,
    pub(crate) element: NodeId,
    pub(crate) document: crate::DocumentId,
    pub(crate) root: NodeId,
    pub(crate) text: std::sync::Arc<Vec<u16>>,
    pub(crate) base_url: crate::DocumentString,
    pub(crate) media: Option<crate::DocumentString>,
    pub(crate) title: Option<crate::DocumentString>,
    pub(crate) blocking: Option<crate::DocumentString>,
    pub(crate) nonce: crate::DocumentString,
}
impl DomStyleSource {
    fn retained_strings(&self) -> impl Iterator<Item = &crate::DocumentString> {
        [
            Some(&self.base_url),
            self.media.as_ref(),
            self.title.as_ref(),
            self.blocking.as_ref(),
            Some(&self.nonce),
        ]
        .into_iter()
        .flatten()
    }
    pub(crate) fn retained_units(&self) -> Result<usize, crate::DomError> {
        let mut units = self.text.capacity();
        for string in self.retained_strings() {
            units = units
                .checked_add(string.shared_code_units().capacity())
                .ok_or(crate::DomError::RepresentationExhausted)?;
        }
        units = units
            .checked_add(self.metadata.retained_payload_units()?)
            .ok_or(crate::DomError::RepresentationExhausted)?;
        Ok(units)
    }
    pub(crate) fn retained_bytes(&self) -> Result<usize, crate::DomError> {
        let mut bytes = std::mem::size_of::<Self>()
            .checked_add(std::mem::size_of::<Vec<u16>>())
            .and_then(|n| {
                n.checked_add(
                    self.text
                        .capacity()
                        .checked_mul(std::mem::size_of::<u16>())?,
                )
            })
            .ok_or(crate::DomError::RepresentationExhausted)?;
        for string in self.retained_strings() {
            bytes = bytes
                .checked_add(std::mem::size_of::<Vec<u16>>())
                .and_then(|n| {
                    n.checked_add(
                        string
                            .shared_code_units()
                            .capacity()
                            .checked_mul(std::mem::size_of::<u16>())?,
                    )
                })
                .ok_or(crate::DomError::RepresentationExhausted)?;
        }
        bytes = bytes
            .checked_add(self.metadata.retained_payload_bytes()?)
            .ok_or(crate::DomError::RepresentationExhausted)?;
        Ok(bytes)
    }
    pub fn operation(&self) -> DomSourceOperationId {
        self.operation
    }
    pub fn ordinal(&self) -> u64 {
        self.ordinal
    }
    pub fn stamp(&self) -> crate::DomStamp {
        self.stamp
    }
    pub fn document_metadata(&self) -> &crate::DomDocumentMetadata {
        &self.metadata
    }
    pub fn root_kind(&self) -> DomStyleRootKind {
        self.root_kind
    }
    pub fn element(&self) -> NodeId {
        self.element
    }
    pub fn document(&self) -> crate::DocumentId {
        self.document
    }
    pub fn root(&self) -> NodeId {
        self.root
    }
    pub fn base_url(&self) -> &crate::DocumentString {
        &self.base_url
    }
    pub fn text(&self) -> &std::sync::Arc<Vec<u16>> {
        &self.text
    }
    pub fn media(&self) -> Option<&crate::DocumentString> {
        self.media.as_ref()
    }
    pub fn title(&self) -> Option<&crate::DocumentString> {
        self.title.as_ref()
    }
    pub fn blocking(&self) -> Option<&crate::DocumentString> {
        self.blocking.as_ref()
    }
    pub fn nonce(&self) -> &crate::DocumentString {
        &self.nonce
    }
}
/// The actual source algorithm requiring composition with an external owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomStyleAction {
    Update(surgeist_html::HtmlStyleUpdateAction),
    OwnerAttributes,
    /// Set the existing sheet flag without replacing its association.
    SetDisabled(bool),
}
/// One entry in the actual ordinary-root stylesheet order at creation time.
/// The current creation remains a ticket until the CSSOM owner publishes it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomStyleSheetPlacement {
    Existing(surgeist_cssom::CssomSheetId),
    ThisStyle,
}
/// One outstanding external owner step. Its serial is not task execution authority.
#[derive(Clone, Debug)]
pub struct DomStyleRequest {
    pub(crate) source: std::sync::Arc<DomStyleSource>,
    pub(crate) action: DomStyleAction,
    pub(crate) cssom: Option<surgeist_cssom::CssomRevision>,
    pub(crate) sheet: Option<DomStyleSheet>,
    pub(crate) association: Option<DomStyleAssociationId>,
    pub(crate) sheet_order: Option<std::sync::Arc<Vec<DomStyleSheetPlacement>>>,
}
impl DomStyleRequest {
    pub fn source(&self) -> &DomStyleSource {
        &self.source
    }
    pub fn action(&self) -> DomStyleAction {
        self.action
    }
    pub fn association(&self) -> Option<DomStyleAssociationId> {
        self.association
    }
    pub fn cssom_revision(&self) -> Option<&surgeist_cssom::CssomRevision> {
        self.cssom.as_ref()
    }
    pub fn cssom_root(&self) -> Option<NodeId> {
        match self.action {
            DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::CreateSheet) => {
                Some(self.source.root)
            }
            DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::RemoveAssociatedSheet)
            | DomStyleAction::OwnerAttributes
            | DomStyleAction::SetDisabled(_) => self.sheet.as_ref().map(|s| s.root),
            _ => None,
        }
    }
    pub fn sheet(&self) -> Option<&DomStyleSheet> {
        self.sheet.as_ref()
    }
    /// Source-time ordinary-tree order, formed after the old association was
    /// removed. Only a CreateSheet request contains `ThisStyle`.
    pub fn sheet_order(&self) -> &[DomStyleSheetPlacement] {
        self.sheet_order.as_deref().map_or(&[], Vec::as_slice)
    }
}
/// An external full-policy capture, distinct from an absent response header or
/// native browsing-context facts. The supplier owns CSP evaluation.
#[derive(Clone, Debug)]
pub struct DomStyleInlineCspObservation {
    pub(crate) source: DomStyleSource,
    pub(crate) policies: crate::store::SharedVec<crate::DocumentString>,
    pub(crate) policy_revision: u64,
    pub(crate) allowed: bool,
}
impl DomStyleInlineCspObservation {
    /// Attests the complete applicable policy list is empty for this exact
    /// Element/Document/text/nonce capture. Empty policy therefore permits inline CSS.
    pub fn complete_empty(source: &DomStyleSource, policy_revision: u64) -> Self {
        let result = Self {
            source: source.clone(),
            policies: crate::store::SharedVec::default(),
            policy_revision,
            allowed: true,
        };
        #[cfg(test)]
        if crate::store::scale_metrics::ENABLED && result.policies.has_backing() {
            crate::store::empty_sequence_observation::allocated::<crate::DocumentString>(
                crate::store::empty_sequence_observation::Kind::StylePolicies,
                result.policies.capacity(),
            );
        }
        result
    }
    #[cfg(test)]
    pub(crate) fn inspect_policy_allocations(
        &self,
        graph: &mut crate::store::compact_tests::AllocationGraph,
    ) {
        self.policies.inspect_named_allocations(
            graph,
            "style.policies.target",
            "style.policies.buffer",
        );
        for policy in self.policies.iter() {
            graph.string(policy);
        }
    }
    /// Records an actual external CSP evaluation of the complete supplied policy
    /// list. This adapter does not parse or reinterpret CSP.
    pub fn evaluated(
        source: &DomStyleSource,
        policies: std::sync::Arc<Vec<crate::DocumentString>>,
        policy_revision: u64,
        allowed: bool,
    ) -> Self {
        Self {
            source: source.clone(),
            policies: crate::store::SharedVec::from_shared(policies),
            policy_revision,
            allowed,
        }
    }
    pub fn policy_revision(&self) -> u64 {
        self.policy_revision
    }
    pub fn policies(&self) -> &[crate::DocumentString] {
        &self.policies
    }
    pub fn allowed(&self) -> bool {
        self.allowed
    }
}

/// A consumer's actual CSSOM owner label and version, bound to one reached
/// creation request. DOM validates the resulting producer sheet against it.
#[derive(Clone, Debug)]
pub struct DomStyleCssomOwnerBinding(std::sync::Arc<StyleCssomOwner>);
#[derive(Debug)]
struct StyleCssomOwner {
    source: std::sync::Arc<DomStyleSource>,
    revision: surgeist_cssom::CssomRevision,
    version: surgeist_cssom::CssomInputVersion,
    owner: String,
}
impl DomStyleCssomOwnerBinding {
    pub fn new(
        request: &DomStyleRequest,
        version: surgeist_cssom::CssomInputVersion,
        owner: String,
    ) -> Result<Self, crate::DomError> {
        if !matches!(
            request.action,
            DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::CreateSheet)
                | DomStyleAction::OwnerAttributes
        ) || version.role != surgeist_cssom::CssomInputRole::Owner
            || owner.is_empty()
        {
            return Err(crate::DomError::StyleReceiptMismatch);
        }
        Ok(Self(std::sync::Arc::new(StyleCssomOwner {
            source: request.source.clone(),
            revision: request
                .cssom
                .clone()
                .ok_or(crate::DomError::StyleReceiptMismatch)?,
            version,
            owner,
        })))
    }
    pub fn version(&self) -> &surgeist_cssom::CssomInputVersion {
        &self.0.version
    }
    pub fn owner(&self) -> &str {
        &self.0.owner
    }
    pub(crate) fn matches(&self, request: &DomStyleRequest) -> bool {
        std::sync::Arc::ptr_eq(&self.0.source, &request.source)
            && Some(&self.0.revision) == request.cssom.as_ref()
    }
    pub(crate) fn retained_bytes(&self) -> Result<usize, crate::DomError> {
        std::mem::size_of::<StyleCssomOwner>()
            .checked_add(self.0.owner.capacity())
            .and_then(|n| n.checked_add(self.0.version.identity.capacity()))
            .ok_or(crate::DomError::RepresentationExhausted)
    }
}

/// An invalid external reply retains the real publication for correction. No
/// failed qualification may silently destroy the consumer's only owned receipt.
pub struct DomStyleCssomPublicationRejection {
    pub(crate) error: crate::DomError,
    pub(crate) commit: surgeist_cssom::CssomCommit,
    pub(crate) snapshot: surgeist_cssom::CssomSnapshot,
}
impl std::fmt::Debug for DomStyleCssomPublicationRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomStyleCssomPublicationRejection")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}
impl DomStyleCssomPublicationRejection {
    pub fn error(&self) -> &crate::DomError {
        &self.error
    }
    pub fn into_parts(
        self,
    ) -> (
        crate::DomError,
        surgeist_cssom::CssomCommit,
        surgeist_cssom::CssomSnapshot,
    ) {
        (self.error, self.commit, self.snapshot)
    }
}

/// Original provenance, not a task or a resource-completion assertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomStyleNotificationOrigin {
    NativeSource(DomSourceOperationId),
    ParsedOccurrence {
        contribution: crate::DomParsedHtmlAssociationId,
        occurrence: surgeist_html_parser::HtmlParseAssociationOccurrenceId,
    },
}
/// Issuer-controlled, once-only permission for an actual sheet association.
/// Parsed permission retains its original finite custody; it does not establish
/// the synchronous consumer frontier required for event dispatch or cleanup.
pub struct DomStyleNotificationToken {
    pub(crate) origin: Option<DomStyleNotificationOrigin>,
    pub(crate) association: DomStyleAssociationId,
    pub(crate) complete: bool,
    pub(crate) parsed: Option<Box<crate::store::pending_associations::NotificationCustody>>,
}
impl Drop for DomStyleNotificationToken {
    fn drop(&mut self) {
        drop(self.origin.take());
        if let Some(boxed) = self.parsed.take() {
            // End the partially moved Box's scope before dropping its graphs
            // and original controlling Meter, including the box-frame debit.
            let value = {
                let inner = boxed;
                *inner
            };
            drop(value);
        }
    }
}
impl std::fmt::Debug for DomStyleNotificationToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomStyleNotificationToken")
            .field("origin", &self.origin)
            .field("association", &self.association)
            .field("complete", &self.complete)
            .finish()
    }
}
impl DomStyleNotificationToken {
    pub fn origin(&self) -> &DomStyleNotificationOrigin {
        self.origin.as_ref().expect("live notification origin")
    }
    pub(crate) fn owner(&self) -> crate::DomOwnerId {
        match self.origin() {
            DomStyleNotificationOrigin::NativeSource(operation) => operation.owner,
            DomStyleNotificationOrigin::ParsedOccurrence { contribution, .. } => {
                contribution.stamp().owner()
            }
        }
    }
    pub fn association(&self) -> DomStyleAssociationId {
        self.association
    }
    pub fn target(&self) -> NodeId {
        self.association.element
    }
}
/// Actual post-notification canonical cleanup, independent of sheet ownership.
#[derive(Debug)]
pub enum DomStyleNotificationCompletion {
    /// Genuine native cleanup.
    Complete(crate::DomCommit),
    /// Genuine parsed cleanup with original output and retirement custody.
    ParsedComplete(crate::DomParsedHtmlStyleNotificationCompletion),
    TargetDestroyed,
}
