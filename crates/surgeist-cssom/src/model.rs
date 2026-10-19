use crate::shared_table::SharedTable;
use crate::*;
use std::{fmt, sync::Arc};
use surgeist_css::*;
use surgeist_css::{CssFontFaceDescriptors, CssMarginRule, CssRule};

/// Real product quotas, including retained detached objects. These bound admitted
/// work/state, not allocator overhead or process RSS. Source backing uses actual
/// CSS-owned buffer capacities; other quotas retain their logical units. Zero
/// capacities are valid, and can refuse required initialization work.
#[derive(Clone, Debug)]
pub struct CssomLimits {
    pub max_objects: usize,
    pub max_entries: usize,
    pub max_string_bytes: usize,
    pub max_input_bytes: usize,
    /// Actual shared parse-source backing retained by this canonical state.
    pub max_retained_source_bytes: usize,
    /// Actual source payload visits, identity comparisons and retained-state walk actions.
    pub max_source_traversal_steps: usize,
    /// Representation walk depth, independent of CSS semantic nesting.
    pub max_source_traversal_depth: usize,
    pub max_depth: usize,
    pub max_identity: u64,
    pub max_revision: u64,
    pub summary_history: usize,
    pub css: CssSpecifiedValueSerializationLimits,
    pub max_pending_owner_effects: usize,
    pub max_pending_owner_effect_bytes: usize,
    pub max_owner_effect_identity: u64,
}
impl Default for CssomLimits {
    fn default() -> Self {
        Self {
            max_objects: 100_000,
            max_entries: 1_000_000,
            max_string_bytes: 16 * 1024 * 1024,
            max_input_bytes: 16 * 1024 * 1024,
            max_retained_source_bytes: 64 * 1024 * 1024,
            max_source_traversal_steps: 16 * 1024 * 1024,
            max_source_traversal_depth: 2048,
            max_depth: 256,
            max_identity: u64::MAX,
            max_revision: u64::MAX,
            summary_history: 128,
            css: CssSpecifiedValueSerializationLimits::default(),
            max_pending_owner_effects: 1024,
            max_pending_owner_effect_bytes: 16 * 1024 * 1024,
            max_owner_effect_identity: u64::MAX,
        }
    }
}

/// Independently revised adapter facts. No context is consulted through callbacks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssomInputRole {
    Document,
    Window,
    ParserMode,
    Origin,
    Layer,
    Base,
    Owner,
    Support,
    Import,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssomInputVersion {
    pub role: CssomInputRole,
    pub identity: String,
    pub revision: u64,
}
/// Concrete supplied facts, captured with their independently revised identity.
/// URL/origin/profile strings are adapter facts; CSSOM does not resolve/evaluate them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssomInputData {
    Document {
        identity: String,
    },
    Window {
        identity: String,
        document: String,
    },
    ParserMode(CssParserContext),
    Origin {
        origin: String,
        profile: String,
    },
    Layer {
        path: Vec<String>,
        ordinal: u64,
    },
    Base {
        url: Option<String>,
        location: Option<String>,
    },
    Owner {
        identity: String,
        attribute: Option<String>,
    },
    Support(CssomDeclarationSupport),
    Import {
        resolved_location: Option<String>,
    },
}
impl CssomInputData {
    pub const fn role(&self) -> CssomInputRole {
        match self {
            Self::Document { .. } => CssomInputRole::Document,
            Self::Window { .. } => CssomInputRole::Window,
            Self::ParserMode(_) => CssomInputRole::ParserMode,
            Self::Origin { .. } => CssomInputRole::Origin,
            Self::Layer { .. } => CssomInputRole::Layer,
            Self::Base { .. } => CssomInputRole::Base,
            Self::Owner { .. } => CssomInputRole::Owner,
            Self::Support(_) => CssomInputRole::Support,
            Self::Import { .. } => CssomInputRole::Import,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssomInput {
    pub version: CssomInputVersion,
    pub data: CssomInputData,
}

#[derive(Clone, Debug)]
pub struct CssomLinkedInput {
    pub version: CssomInputVersion,
    pub snapshot: CssomSnapshot,
    pub sheet: CssomSheetId,
}
impl PartialEq for CssomLinkedInput {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.sheet == other.sheet
            && self.snapshot.revision() == other.snapshot.revision()
    }
}
impl Eq for CssomLinkedInput {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CssomContext {
    pub inputs: Vec<CssomInput>,
    pub linked: Vec<CssomLinkedInput>,
}

impl CssomContext {
    pub(crate) fn validate(&self) -> Result<(), CssomError> {
        if self
            .inputs
            .iter()
            .filter(|input| matches!(input.data, CssomInputData::ParserMode(_)))
            .count()
            > 1
        {
            return Err(CssomError::InvalidInput("ambiguous parser mode input"));
        }
        if self
            .inputs
            .iter()
            .filter(|input| matches!(input.data, CssomInputData::Support(_)))
            .count()
            > 1
        {
            return Err(CssomError::InvalidInput("ambiguous declaration support"));
        }
        for (index, input) in self.inputs.iter().enumerate() {
            if input.version.role != input.data.role() {
                return Err(CssomError::InvalidInput("supplied input role mismatch"));
            }
            if self.inputs[..index].iter().any(|previous| {
                previous.version.role == input.version.role
                    && previous.version.identity == input.version.identity
            }) {
                return Err(CssomError::InvalidInput(
                    "duplicate supplied input identity",
                ));
            }
        }
        Ok(())
    }
    /// Parser facts apply to subsequent text ingestion; existing checked payloads retain theirs.
    pub(crate) fn parser_context(&self) -> CssParserContext {
        self.inputs
            .iter()
            .find_map(|input| match input.data {
                CssomInputData::ParserMode(context) => Some(context),
                _ => None,
            })
            .unwrap_or_default()
    }
}

/// Adapter-supplied sheet facts. `None` base is distinct from `Some("")`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssomSheetInputs {
    pub version: CssomInputVersion,
    pub base_url: Option<String>,
    pub location: Option<String>,
    pub constructor_document: Option<String>,
    pub owner: Option<String>,
    /// Exact immutable DOMString units. Shared title backing avoids deep copies
    /// when canonical sheet records are cloned during staging.
    pub title: Arc<Vec<u16>>,
    pub alternate: bool,
    pub disabled: bool,
    pub origin_clean: bool,
    pub constructed: bool,
    pub disallow_modification: bool,
    pub media: CssMediaQueryList,
}
impl CssomSheetInputs {
    /// Source constructor defaults; host identity/base/document remain explicit.
    pub fn constructed(
        version: CssomInputVersion,
        document: String,
        base_url: Option<String>,
    ) -> Self {
        Self {
            version,
            base_url,
            location: None,
            constructor_document: Some(document),
            owner: None,
            title: Arc::new(Vec::new()),
            alternate: false,
            disabled: false,
            origin_clean: true,
            constructed: true,
            disallow_modification: false,
            media: CssMediaQueryList::new(Vec::new()),
        }
    }
    /// Explicit external sheet facts; no Window or URL resolution is fabricated.
    pub fn external(version: CssomInputVersion, origin_clean: bool) -> Self {
        Self {
            version,
            base_url: None,
            location: None,
            constructor_document: None,
            owner: None,
            title: Arc::new(Vec::new()),
            alternate: false,
            disabled: false,
            origin_clean,
            constructed: false,
            disallow_modification: false,
            media: CssMediaQueryList::new(Vec::new()),
        }
    }
}

/// Original checked occurrence, independent of current live fields/membership.
#[derive(Clone, Debug)]
pub enum CssomAuthoredRule {
    Ordinary(CssRule),
    Scoped(CssScopedRule),
    Keyframe(CssKeyframeBlock),
    Margin(CssMarginRule),
    /// Genuine raw declaration input retained without manufacturing a rule position.
    RawNestedDeclarations(CssDeclarationList),
    /// Genuine inert named-support test occurrence, never a rendering rule.
    SupportsTestChild(CssSupportsTestChild),
}
#[derive(Clone, Debug)]
pub enum CssomSelectors {
    Ordinary(CssStyleSelectorList),
    Scoped(CssScopedStyleSelectorList),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomGroupKind {
    Media,
    Supports,
    Container,
    Layer,
    Scope,
    When,
    Else,
}
#[derive(Clone, Debug)]
pub enum CssomGroupPrelude {
    Media(CssomMediaListId),
    Supports(CssSupportsCondition),
    Container(CssContainerPrelude),
    Layer(Option<CssLayerName>),
    Scope {
        start: Option<CssScopeSelectorList>,
        end: Option<CssScopeSelectorList>,
    },
    When(CssWhenCondition),
    Else(Option<CssWhenCondition>),
}
impl CssomGroupPrelude {
    pub const fn kind(&self) -> CssomGroupKind {
        match self {
            Self::Media(_) => CssomGroupKind::Media,
            Self::Supports(_) => CssomGroupKind::Supports,
            Self::Container(_) => CssomGroupKind::Container,
            Self::Layer(_) => CssomGroupKind::Layer,
            Self::Scope { .. } => CssomGroupKind::Scope,
            Self::When(_) => CssomGroupKind::When,
            Self::Else(_) => CssomGroupKind::Else,
        }
    }
}

/// Only structural kinds separate mutable fields from their original occurrence.
/// Group preludes and unchanged leaf data are owned current checked payloads;
/// original occurrences remain separately available through `CssomRule::authored`.
#[derive(Clone, Debug)]
pub enum CssomRuleData {
    /// Surgeist's named-support grouping policy. The checked current body and
    /// live direct-child list are reconstructed and published together.
    NamedSupports {
        name: CssSupportsConditionName,
        body: CssSupportsTestBody,
        children: CssomRuleListId,
    },
    /// Opaque inert leaf. Nested candidate bodies remain immutable CSS payloads;
    /// they have no descendant CSSOM identities or rendering capabilities.
    SupportsTestChild(CssSupportsTestChild),
    Style {
        selectors: CssomSelectors,
        block: CssomBlockId,
        children: CssomRuleListId,
    },
    Group {
        prelude: CssomGroupPrelude,
        children: CssomRuleListId,
    },
    NestedDeclarations {
        block: CssomBlockId,
    },
    Page {
        selectors: CssPageSelectorList,
        block: CssomBlockId,
        children: CssomRuleListId,
    },
    Margin {
        name: CssMarginBox,
        block: CssomBlockId,
    },
    Keyframes {
        name: CssKeyframesName,
        children: CssomRuleListId,
    },
    Keyframe {
        selectors: CssKeyframeSelectorList,
        block: CssomBlockId,
    },
    Import {
        target: CssImportTarget,
        layer: Option<CssImportLayer>,
        supports: Option<CssImportSupports>,
        origin: CssValueOrigin,
        media: CssomMediaListId,
        child_sheet: Option<CssomSheetId>,
        resolved_location: Option<String>,
        input: Option<CssomInputVersion>,
    },
    CounterStyle {
        name: String,
        block: CssomBlockId,
    },
    FontFace {
        block: CssomBlockId,
    },
    FontFeatureValues {
        families: Vec<CssFontFaceFamily>,
        family_input: Option<CssomFontFamilyInput>,
        maps: Vec<CssomFeatureMapId>,
        font_display: Vec<CssFontFeatureDisplayOccurrence>,
    },
    /// Readonly rule fields; a query-list body owns an editable MediaList identity.
    CustomMedia {
        name: CssCustomMediaName,
        query: CssomCustomMediaQuery,
    },
    Leaf {
        current: CssomAuthoredRule,
    },
}
impl CssomRuleData {
    pub fn children(&self) -> Option<&CssomRuleListId> {
        match self {
            Self::Style { children, .. }
            | Self::Group { children, .. }
            | Self::Page { children, .. }
            | Self::NamedSupports { children, .. }
            | Self::Keyframes { children, .. } => Some(children),
            _ => None,
        }
    }
    pub fn block(&self) -> Option<&CssomBlockId> {
        match self {
            Self::Style { block, .. }
            | Self::NestedDeclarations { block }
            | Self::Page { block, .. }
            | Self::Margin { block, .. }
            | Self::Keyframe { block, .. }
            | Self::CounterStyle { block, .. }
            | Self::FontFace { block } => Some(block),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssomParent {
    Sheet(CssomSheetId),
    Rule(CssomRuleId),
    Detached,
}

#[derive(Clone, Debug)]
pub struct CssomRule {
    pub(crate) parent: CssomParent,
    pub(crate) data: Arc<CssomRuleData>,
    pub(crate) authored: Arc<CssomAuthoredRule>,
    pub(crate) selector_input: Option<CssParsedOrigin>,
    pub(crate) selector_inputs: Option<Arc<CssomContext>>,
    pub(crate) selector_namespaces: Option<Arc<CssNamespaceContext>>,
    pub(crate) admission_diagnostics: Arc<Vec<CssRecoveryDiagnostic>>,
}
impl CssomRule {
    pub(crate) fn data_mut(&mut self) -> &mut CssomRuleData {
        Arc::make_mut(&mut self.data)
    }
    /// Recovery from the latest successful rule or selector admission. Initial
    /// sheet ingress retains its complete ordered report on the owning sheet.
    pub fn admission_diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.admission_diagnostics
    }
    /// Latest complete selector input, independent of the original rule occurrence.
    pub fn selector_input(&self) -> Option<&CssParsedOrigin> {
        self.selector_input.as_ref()
    }
    pub fn selector_inputs(&self) -> Option<&CssomContext> {
        self.selector_inputs.as_deref()
    }
    pub fn selector_namespaces(&self) -> Option<&CssNamespaceContext> {
        self.selector_namespaces.as_deref()
    }
    pub fn parent(&self) -> &CssomParent {
        &self.parent
    }
    pub fn data(&self) -> &CssomRuleData {
        &self.data
    }
    pub fn authored(&self) -> &CssomAuthoredRule {
        &self.authored
    }
}

#[derive(Clone, Debug)]
pub struct CssomSheet {
    pub(crate) inputs: CssomSheetInputs,
    pub(crate) rules: CssomRuleListId,
    pub(crate) media: CssomMediaListId,
    pub(crate) owner_rule: Option<CssomRuleId>,
    pub(crate) namespaces: Arc<CssNamespaceContext>,
    pub(crate) authored: Arc<CssSheet>,
    pub(crate) diagnostics: Arc<Vec<CssRecoveryDiagnostic>>,
}
impl CssomSheet {
    pub fn inputs(&self) -> &CssomSheetInputs {
        &self.inputs
    }
    pub fn rules(&self) -> &CssomRuleListId {
        &self.rules
    }
    pub fn media(&self) -> &CssomMediaListId {
        &self.media
    }
    pub fn owner_rule(&self) -> Option<&CssomRuleId> {
        self.owner_rule.as_ref()
    }
    pub fn namespaces(&self) -> &CssNamespaceContext {
        &self.namespaces
    }
    pub fn authored(&self) -> &CssSheet {
        &self.authored
    }
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Clone, Debug)]
pub enum CssomProjection<T, E> {
    Available(T),
    Unavailable(E),
}
#[derive(Clone, Debug)]
pub enum CssomPropertyOccurrences {
    Ordinary(CssDeclarationList),
    Keyframe(CssKeyframeDeclarationList),
    Margin(CssMarginDeclarationBlock),
    /// Genuine raw contents admitted for a live domain without fabricating authored AST.
    RawContents(CssDeclarationList),
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CssomBlockFlags {
    pub computed: bool,
    pub readonly: bool,
    pub updating: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomPropertyDomain {
    Ordinary,
    Keyframe,
    Margin,
}
#[derive(Clone, Debug)]
pub enum CssomBlockData {
    Properties {
        domain: CssomPropertyDomain,
        authored: CssomPropertyOccurrences,
        selected: CssomProjection<CssSpecifiedDeclarationBlock, CssDeclarationBlockError>,
    },
    Page {
        authored: CssPageDeclarationBlock,
        selected: CssomProjection<CssSpecifiedPageDeclarationBlock, CssPageProjectionError>,
    },
    CounterStyle(Box<CssCounterStyleDescriptors>),
    FontFace(CssFontFaceDescriptors),
}
/// Genuine raw contents of the last replacement request, independent of edited selected values.
#[derive(Clone, Debug, PartialEq)]
pub struct CssomDeclarationInput {
    pub source: String,
    pub parser_context: CssParserContext,
    pub diagnostics: Vec<CssRecoveryDiagnostic>,
}
#[derive(Clone, Debug)]
pub struct CssomBlock {
    pub(crate) parent: Option<CssomRuleId>,
    pub(crate) binding: crate::inline_context::BlockBinding,
    pub(crate) flags: CssomBlockFlags,
    pub(crate) data: Arc<CssomBlockData>,
    pub(crate) admission_inputs: Arc<Vec<CssomInputVersion>>,
    pub(crate) counter_edits: Arc<Vec<CssCounterStyleDescriptorValue>>,
    pub(crate) replacement_input: Option<Arc<CssomDeclarationInput>>,
    pub(crate) selected_font_face: Option<
        Arc<
            CssomProjection<CssSpecifiedFontFaceDeclarationBlock, CssFontFaceDeclarationBlockError>,
        >,
    >,
}
impl CssomBlock {
    pub(crate) fn data_mut(&mut self) -> &mut CssomBlockData {
        Arc::make_mut(&mut self.data)
    }
    pub(crate) fn admission_inputs_mut(&mut self) -> &mut Vec<CssomInputVersion> {
        Arc::make_mut(&mut self.admission_inputs)
    }
    /// Current unique selected descriptors, independent of retained authored occurrences.
    pub fn selected_font_face(
        &self,
    ) -> Option<
        &CssomProjection<CssSpecifiedFontFaceDeclarationBlock, CssFontFaceDeclarationBlockError>,
    > {
        self.selected_font_face.as_deref()
    }
    pub fn replacement_input(&self) -> Option<&CssomDeclarationInput> {
        self.replacement_input.as_deref()
    }
    pub fn admission_inputs(&self) -> &[CssomInputVersion] {
        &self.admission_inputs
    }
    pub fn parent(&self) -> Option<&CssomRuleId> {
        self.parent.as_ref()
    }
    pub fn owner(&self) -> Option<&CssomInputVersion> {
        match &self.binding {
            crate::inline_context::BlockBinding::Store(owner) => owner.as_ref(),
            crate::inline_context::BlockBinding::Inline(context) => Some(context.owner()),
        }
    }
    /// The current per-block packet, absent for store-context declarations.
    pub fn inline_context(&self) -> Option<&CssomInlineDeclarationContext> {
        match &self.binding {
            crate::inline_context::BlockBinding::Inline(context) => Some(context),
            crate::inline_context::BlockBinding::Store(_) => None,
        }
    }
    pub const fn flags(&self) -> CssomBlockFlags {
        self.flags
    }
    pub fn data(&self) -> &CssomBlockData {
        &self.data
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CssomFeatureValue {
    Unsigned(Vec<u32>),
    /// Exact source is retained; conversion never silently drops or clamps it.
    Authored(CssFontFeatureValue),
}
#[derive(Clone, Debug, PartialEq)]
pub struct CssomFeatureEntry {
    pub key: String,
    pub value: CssomFeatureValue,
}
#[derive(Clone, Debug)]
pub struct CssomFeatureMap {
    pub(crate) kind: CssFontFeatureValueKind,
    pub(crate) entries: Vec<CssomFeatureEntry>,
    pub(crate) authored: Arc<Vec<CssFontFeatureValueDefinition>>,
}
impl CssomFeatureMap {
    pub const fn kind(&self) -> CssFontFeatureValueKind {
        self.kind
    }
    pub fn raw_entries(&self) -> &[CssomFeatureEntry] {
        &self.entries
    }
    pub fn authored(&self) -> &[CssFontFeatureValueDefinition] {
        &self.authored
    }
    /// `Some(&[])` is present-empty; `None` is absent. Exact unrepresentable or
    /// symbolic authored values report a capability failure, not absence.
    pub(crate) fn get(&self, key: &str) -> Result<Option<&[u32]>, CssomError> {
        match self.entries.iter().find(|v| v.key == key).map(|v| &v.value) {
            None => Ok(None),
            Some(value) => value.projected().map(Some),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct State {
    pub owner: CssomOwnerId,
    pub revision: CssomRevision,
    pub next_identity: u64,
    pub context: Arc<CssomContext>,
    pub sheet_list: CssomSheetListId,
    pub sheet_order: Arc<Vec<CssomSheetId>>,
    pub sheets: SharedTable<CssomSheetId, CssomSheet>,
    pub rules: SharedTable<CssomRuleId, CssomRule>,
    pub blocks: SharedTable<CssomBlockId, CssomBlock>,
    pub lists: SharedTable<CssomRuleListId, Vec<CssomRuleId>>,
    pub media: SharedTable<CssomMediaListId, CssMediaQueryList>,
    pub maps: SharedTable<CssomFeatureMapId, CssomFeatureMap>,
}
impl State {
    pub(crate) fn sheet(&self, id: &CssomSheetId) -> Result<&CssomSheet, CssomError> {
        lookup(&self.sheets, id, &id.owner, &self.owner)
    }
    /// Admit stylesheet rule edits after owner/identity (or creation-ticket)
    /// admission, preserving Security before NotAllowed exception precedence.
    pub(crate) fn editable_sheet_rules(
        &self,
        id: &CssomSheetId,
    ) -> Result<&CssomSheet, CssomError> {
        let sheet = self.sheet(id)?;
        if !sheet.inputs.origin_clean {
            return Err(CssomError::Source(CssomException::Security));
        }
        if sheet.inputs.disallow_modification {
            return Err(CssomError::Source(CssomException::NotAllowed));
        }
        Ok(sheet)
    }
    pub(crate) fn rule(&self, id: &CssomRuleId) -> Result<&CssomRule, CssomError> {
        lookup(&self.rules, id, &id.owner, &self.owner)
    }
    pub(crate) fn block(&self, id: &CssomBlockId) -> Result<&CssomBlock, CssomError> {
        lookup(&self.blocks, id, &id.owner, &self.owner)
    }
    pub(crate) fn declaration_context(
        &self,
        id: &CssomBlockId,
    ) -> Result<&Arc<CssomContext>, CssomError> {
        Ok(self
            .block(id)?
            .inline_context()
            .map_or(&self.context, CssomInlineDeclarationContext::shared_context))
    }
    pub(crate) fn inline_contexts(&self) -> impl Iterator<Item = &CssomContext> {
        self.blocks
            .values()
            .filter_map(CssomBlock::inline_context)
            .map(CssomInlineDeclarationContext::context)
    }
    pub(crate) fn media(&self, id: &CssomMediaListId) -> Result<&CssMediaQueryList, CssomError> {
        lookup(&self.media, id, &id.owner, &self.owner)
    }
    pub(crate) fn feature_map(
        &self,
        id: &CssomFeatureMapId,
    ) -> Result<&CssomFeatureMap, CssomError> {
        lookup(&self.maps, id, &id.owner, &self.owner)
    }
    pub(crate) fn parent_style_sheet(
        &self,
        id: &CssomRuleId,
    ) -> Result<Option<&CssomSheetId>, CssomError> {
        let mut rule = self.rule(id)?;
        loop {
            match rule.parent() {
                CssomParent::Sheet(v) => return Ok(Some(v)),
                CssomParent::Detached => return Ok(None),
                CssomParent::Rule(v) => rule = self.rule(v)?,
            }
        }
    }
}

/// Trusted owning Rust transport. IDL bindings must use guarded source getters.
#[derive(Clone, Debug)]
pub struct CssomSnapshot {
    pub(crate) state: Arc<State>,
}
impl CssomSnapshot {
    pub fn owner(&self) -> &CssomOwnerId {
        &self.state.owner
    }
    pub fn revision(&self) -> &CssomRevision {
        &self.state.revision
    }
    pub fn context(&self) -> &CssomContext {
        &self.state.context
    }
    pub fn sheet_list(&self) -> &CssomSheetListId {
        &self.state.sheet_list
    }
    pub fn sheets(&self) -> &[CssomSheetId] {
        &self.state.sheet_order
    }
    pub fn sheet(&self, id: &CssomSheetId) -> Result<&CssomSheet, CssomError> {
        lookup(&self.state.sheets, id, &id.owner, self.owner())
    }
    pub fn rule(&self, id: &CssomRuleId) -> Result<&CssomRule, CssomError> {
        lookup(&self.state.rules, id, &id.owner, self.owner())
    }
    pub fn block(&self, id: &CssomBlockId) -> Result<&CssomBlock, CssomError> {
        lookup(&self.state.blocks, id, &id.owner, self.owner())
    }
    pub fn rules(&self, id: &CssomRuleListId) -> Result<&[CssomRuleId], CssomError> {
        lookup(&self.state.lists, id, &id.owner, self.owner()).map(Vec::as_slice)
    }
    pub fn media(&self, id: &CssomMediaListId) -> Result<&CssMediaQueryList, CssomError> {
        lookup(&self.state.media, id, &id.owner, self.owner())
    }
    pub fn feature_map(&self, id: &CssomFeatureMapId) -> Result<&CssomFeatureMap, CssomError> {
        lookup(&self.state.maps, id, &id.owner, self.owner())
    }
    /// Specified property-block getter. Computed and readonly flags are independent;
    /// a computed block returns the source-defined empty string.
    pub fn specified_property_css_text(
        &self,
        id: &CssomBlockId,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        let block = self.block(id)?;
        let CssomBlockData::Properties { selected, .. } = block.data() else {
            return Err(CssomError::WrongKind);
        };
        if block.flags.computed {
            return Ok(String::new());
        }
        match selected {
            CssomProjection::Available(v) => v
                .serialize_cssom_with_limits(limits)
                .map_err(CssomError::Declaration),
            CssomProjection::Unavailable(e) => Err(CssomError::Declaration(e.clone())),
        }
    }
    pub fn parent_rule(&self, id: &CssomRuleId) -> Result<Option<&CssomRuleId>, CssomError> {
        Ok(match self.rule(id)?.parent() {
            CssomParent::Rule(v) => Some(v),
            _ => None,
        })
    }
    pub fn parent_style_sheet(
        &self,
        id: &CssomRuleId,
    ) -> Result<Option<&CssomSheetId>, CssomError> {
        let mut rule = self.rule(id)?;
        loop {
            match rule.parent() {
                CssomParent::Sheet(v) => return Ok(Some(v)),
                CssomParent::Detached => return Ok(None),
                CssomParent::Rule(v) => rule = self.rule(v)?,
            }
        }
    }
    pub fn parent_sheet(&self, id: &CssomSheetId) -> Result<Option<&CssomSheetId>, CssomError> {
        match self.sheet(id)?.owner_rule() {
            Some(rule) => self.parent_style_sheet(rule),
            None => Ok(None),
        }
    }
    /// Source CSSStyleSheet.cssRules operation, with origin-clean security guard.
    pub fn sheet_rules(&self, id: &CssomSheetId) -> Result<&[CssomRuleId], CssomError> {
        let sheet = self.sheet(id)?;
        if !sheet.inputs.origin_clean {
            return Err(CssomError::Source(CssomException::Security));
        }
        self.rules(&sheet.rules)
    }
}
fn lookup<'a, K: Eq + std::hash::Hash, V>(
    map: &'a SharedTable<K, V>,
    id: &K,
    owner: &CssomOwnerId,
    expected: &CssomOwnerId,
) -> Result<&'a V, CssomError> {
    if owner != expected {
        return Err(CssomError::ForeignOwner);
    }
    map.get(id).ok_or(CssomError::MissingObject)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomException {
    Syntax,
    HierarchyRequest,
    IndexSize,
    Security,
    NotAllowed,
    NoModificationAllowed,
    InvalidState,
    NotFound,
    InvalidAccess,
}
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssomError {
    Source(CssomException),
    ComputedStyleUpdatePrecondition,
    /// The intrinsic definition or original algorithm depends on unresolved inputs.
    CounterStylePreparationUnavailable(CssCounterStyleDefinitionStatus),
    UnresolvedDeclarationPreparation,
    StaleDeclarationDecision,
    EffectAlreadyConsumed,
    StaleOwnerEffect,
    StaleOwnerNotification,
    EffectIdentityExhausted,
    Metadata(CssPropertyMetadataError),
    Expansion(CssExpansionError),
    PageBlock(CssPageBlockError),
    FontFace(CssFontFaceDeclarationBlockError),
    Value(CssSpecifiedValueSerializationError),
    ForeignOwner,
    MissingObject,
    WrongKind,
    StaleRevision,
    StaleContext,
    StaleReplacement,
    BatchAborted,
    ForeignTicket,
    IdentityExhausted,
    RevisionExhausted,
    InvalidInput(&'static str),
    Limit {
        resource: &'static str,
        maximum: usize,
    },
    Declaration(CssDeclarationBlockError),
    Page(CssPageProjectionError),
    ParseResource(Box<CssRecoveryDiagnostic>),
    Component(CssComponentValueError),
    Media(CssMediaCssomSerializationError),
    AuthoredFeatureConversion,
    Format(CssRuleCssomSerializationError),
    Serialization(CssSpecifiedValueSerializationError),
    QueryMatch(CssomQueryMatchError),
    Keyframe(CssKeyframeRuleViewError),
    KeyframeComparison(CssKeyframeSelectorComparisonError),
    NamedSupports(CssNamedSupportsConstructionError),
}
impl fmt::Display for CssomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CSSOM operation failed: {self:?}")
    }
}
impl std::error::Error for CssomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Declaration(e) => Some(e),
            Self::Expansion(e) => Some(e),
            Self::PageBlock(e) => Some(e),
            Self::FontFace(e) => Some(e),
            Self::Metadata(e) => Some(e),
            Self::Value(e) => Some(e),
            Self::Page(e) => Some(e),
            Self::Format(e) => Some(e),
            Self::Serialization(e) => Some(e),
            Self::Media(e) => Some(e),
            Self::QueryMatch(e) => Some(e),
            Self::Keyframe(e) => Some(e),
            Self::KeyframeComparison(e) => Some(e),
            Self::Component(e) => Some(e),
            Self::NamedSupports(e) => Some(e),
            _ => None,
        }
    }
}
