use crate::*;
use std::collections::BTreeSet;

/// Conservative facts for downstream invalidation; CSSOM does not choose a cascade.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CssomChange {
    SheetMembership,
    RuleMembership,
    Order,
    Disabled,
    Media,
    Selectors,
    Declarations,
    Descriptors,
    FeatureMaps,
    Ancestry,
    Namespaces,
    ImportLinks,
    Origin,
    Layer,
    Base,
    Location,
    Parser,
    Support,
    Owner,
    Replacement,
    Modification,
    Provenance,
}
/// Every independently revised input captured by a snapshot, correlated to its object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssomInputManifest {
    pub context: CssomContext,
    pub sheets: Vec<(CssomSheetId, CssomInputVersion)>,
    pub blocks: Vec<(CssomBlockId, CssomInputVersion)>,
    pub imports: Vec<(CssomRuleId, CssomInputVersion)>,
    pub declaration_admissions: Vec<(CssomBlockId, Vec<CssomInputVersion>)>,
}
impl crate::model::State {
    pub(crate) fn input_manifest(&self) -> CssomInputManifest {
        let mut sheets = self
            .sheets
            .iter()
            .map(|(id, v)| (id.clone(), v.inputs.version.clone()))
            .collect::<Vec<_>>();
        let mut blocks = self
            .blocks
            .iter()
            .filter_map(|(id, v)| v.owner.clone().map(|v| (id.clone(), v)))
            .collect::<Vec<_>>();
        let mut imports = self
            .rules
            .iter()
            .filter_map(|(id, v)| match &v.data {
                CssomRuleData::Import { input: Some(v), .. } => Some((id.clone(), v.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        sheets.sort_by_key(|v| v.0.serial);
        blocks.sort_by_key(|v| v.0.serial);
        imports.sort_by_key(|v| v.0.serial);
        let mut declaration_admissions = self
            .blocks
            .iter()
            .filter(|(_, block)| !block.admission_inputs.is_empty())
            .map(|(id, block)| (id.clone(), block.admission_inputs.clone()))
            .collect::<Vec<_>>();
        declaration_admissions.sort_by_key(|(id, _)| id.serial);
        CssomInputManifest {
            declaration_admissions,
            context: self.context.clone(),
            sheets,
            blocks,
            imports,
        }
    }
}

impl CssomSnapshot {
    pub fn inputs(&self) -> CssomInputManifest {
        self.state.input_manifest()
    }
}

/// Complete atomic coverage of `(from, to]`. No per-operation state is published.
#[derive(Clone, Debug)]
pub struct CssomChangeSummary {
    pub(crate) owner: CssomOwnerId,
    pub(crate) from: CssomRevision,
    pub(crate) to: CssomRevision,
    pub(crate) categories: BTreeSet<CssomChange>,
    pub(crate) affected: Vec<CssomObjectId>,
    pub(crate) before_inputs: CssomInputManifest,
    pub(crate) after_inputs: CssomInputManifest,
    pub(crate) full_recompute: bool,
}
impl CssomChangeSummary {
    pub fn owner(&self) -> &CssomOwnerId {
        &self.owner
    }
    pub fn from(&self) -> &CssomRevision {
        &self.from
    }
    pub fn to(&self) -> &CssomRevision {
        &self.to
    }
    pub fn categories(&self) -> &BTreeSet<CssomChange> {
        &self.categories
    }
    pub fn affected(&self) -> &[CssomObjectId] {
        &self.affected
    }
    pub fn before_inputs(&self) -> &CssomInputManifest {
        &self.before_inputs
    }
    pub fn after_inputs(&self) -> &CssomInputManifest {
        &self.after_inputs
    }
    pub const fn requires_full_recompute(&self) -> bool {
        self.full_recompute
    }
    /// Rejects mismatched owner, endpoints, inputs and conservative changes.
    pub fn supports_incremental(&self, before: &CssomSnapshot, after: &CssomSnapshot) -> bool {
        !self.full_recompute
            && before.owner() == &self.owner
            && after.owner() == &self.owner
            && before.revision() == &self.from
            && after.revision() == &self.to
            && before.inputs() == self.before_inputs
            && after.inputs() == self.after_inputs
    }
}

#[derive(Clone, Debug)]
pub enum CssomPublication {
    Unchanged { revision: CssomRevision },
    Changed(Box<CssomChangeSummary>),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomRecomputeReason {
    ForeignOwner,
    FutureRevision,
    CoverageGap,
    ContextMismatch,
    ConservativeChange,
}
#[derive(Clone, Debug)]
pub enum CssomChanges {
    Unchanged,
    Incremental(Box<CssomChangeSummary>),
    FullRecompute(CssomRecomputeReason),
}

/// Product preconditions precede each source algorithm. Acquire from one capture.
#[derive(Clone, Debug)]
pub struct CssomEditGuard {
    pub(crate) revision: CssomRevision,
    pub(crate) context: CssomContext,
}
impl CssomEditGuard {
    pub fn from_snapshot(snapshot: &CssomSnapshot) -> Self {
        Self {
            revision: snapshot.revision().clone(),
            context: snapshot.context().clone(),
        }
    }
    /// Independently supplied current inputs must match the admitted capture.
    pub fn for_inputs(snapshot: &CssomSnapshot, current: CssomContext) -> Self {
        Self {
            revision: snapshot.revision().clone(),
            context: current,
        }
    }
}
