//! Current independently qualified inputs for one stable inline declaration.
use crate::*;
use std::sync::Arc;

/// An immutable current input packet for an owner-bound inline declaration.
///
/// Exactly one Owner, ParserMode and Support input is required. An explicitly
/// empty support profile is valid. The adapter owns the truth and freshness of
/// supplied facts; CSSOM validates their roles, identities and supplied versions.
/// Rebinding this packet affects future ingestion, not existing checked values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssomInlineDeclarationContext {
    context: Arc<CssomContext>,
    owner_index: usize,
}
impl CssomInlineDeclarationContext {
    pub fn try_new(context: CssomContext) -> Result<Self, CssomError> {
        context.validate()?;
        for role in [
            CssomInputRole::Owner,
            CssomInputRole::ParserMode,
            CssomInputRole::Support,
        ] {
            if context
                .inputs
                .iter()
                .filter(|input| input.version.role == role)
                .count()
                != 1
            {
                return Err(CssomError::InvalidInput(
                    "inline input role must occur exactly once",
                ));
            }
        }
        for (index, linked) in context.linked.iter().enumerate() {
            if context
                .inputs
                .iter()
                .map(|input| &input.version)
                .chain(context.linked[..index].iter().map(|input| &input.version))
                .any(|previous| {
                    previous.role == linked.version.role
                        && previous.identity == linked.version.identity
                })
            {
                return Err(CssomError::InvalidInput("duplicate inline input identity"));
            }
        }
        let owner_index = context
            .inputs
            .iter()
            .position(|input| input.version.role == CssomInputRole::Owner)
            .expect("one validated Owner input");
        Ok(Self {
            context: Arc::new(context),
            owner_index,
        })
    }
    pub fn context(&self) -> &CssomContext {
        &self.context
    }
    pub fn owner(&self) -> &CssomInputVersion {
        &self.context.inputs[self.owner_index].version
    }
    pub(crate) fn shared_context(&self) -> &Arc<CssomContext> {
        &self.context
    }
    pub(crate) fn versions(&self) -> impl Iterator<Item = &CssomInputVersion> {
        self.context
            .inputs
            .iter()
            .map(|input| &input.version)
            .chain(self.context.linked.iter().map(|linked| &linked.version))
    }
}

/// One authority for a block's owner and current context. Existing sheet/rule
/// declarations keep their store-context semantics and optional owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BlockBinding {
    Store(Option<CssomInputVersion>),
    Inline(CssomInlineDeclarationContext),
}
