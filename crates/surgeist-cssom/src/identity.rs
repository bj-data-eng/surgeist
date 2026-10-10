use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

/// An owning identity token; equality survives the live store's destruction.
#[derive(Clone)]
pub struct CssomOwnerId(Arc<()>);
impl CssomOwnerId {
    pub(crate) fn new() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for CssomOwnerId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for CssomOwnerId {}
impl Hash for CssomOwnerId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.0), state);
    }
}
impl fmt::Debug for CssomOwnerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CssomOwnerId")
    }
}

macro_rules! identity {
    ($($name:ident),+ $(,)?) => {$ (
        #[doc = "Opaque, owner-qualified live identity, independent of CSS provenance."]
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub struct $name { pub(crate) owner: CssomOwnerId, pub(crate) serial: u64 }
        impl $name {
            pub fn owner(&self) -> &CssomOwnerId { &self.owner }
        }
    )+};
}
identity!(
    CssomSheetId,
    CssomRuleId,
    CssomBlockId,
    CssomRuleListId,
    CssomSheetListId,
    CssomMediaListId,
    CssomFeatureMapId
);

/// Publication token. Arithmetic is not an edit or coverage contract.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssomRevision {
    pub(crate) owner: CssomOwnerId,
    pub(crate) value: u64,
}
impl CssomRevision {
    pub fn owner(&self) -> &CssomOwnerId {
        &self.owner
    }
    pub const fn value(&self) -> u64 {
        self.value
    }
}

/// A typed identity appearing in a change summary.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CssomObjectId {
    Sheet(CssomSheetId),
    Rule(CssomRuleId),
    Block(CssomBlockId),
    RuleList(CssomRuleListId),
    SheetList(CssomSheetListId),
    MediaList(CssomMediaListId),
    FeatureMap(CssomFeatureMapId),
}
