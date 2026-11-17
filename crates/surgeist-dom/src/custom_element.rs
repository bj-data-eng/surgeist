//! Custom element registries, definitions, reactions and the Source constructor protocol.
//!
//! `DomRegistry` reads a registry at one snapshot, `DomRegistryId` names one, and
//! `DomRegistryBinding` is the registry that controlled creation selects.
//! `DomElementDefinition` is an element's associated definition, and
//! `DomRegistryDefinitionId` and `DomRegistryDefinitionPreparation` identify and publish
//! one. `DomCustomElementReaction`, `DomCustomElementReactions`,
//! `DomSynchronousReactionQueue` and `DomShadowReactionRoute` carry the reactions an edit
//! reaches, in the order reached. The `DomSourceCustomElement*` types and
//! `DomWatchedElementCreation` are the Source continuation for constructor, upgrade and
//! callback work.
//!
//! `docs/source-and-style.md` owns the Source contract.

pub use crate::internal::document::DomRegistryBinding;
pub use crate::internal::element::DomElementDefinition;
pub use crate::internal::registry::{DomRegistry, DomRegistryDefinitionId, DomRegistryId};
pub use crate::internal::source::DomSourceCustomElementRequest;
pub use crate::internal::store::registry::DomRegistryDefinitionPreparation;
pub use crate::internal::store::source::operation::{
    DomPreparedElementName, DomShadowReactionRoute, DomSourceCustomElementCallbackScope,
    DomSourceCustomElementDisposition, DomSourceCustomElementOutcome,
    DomSourceCustomElementRetention, DomSourceCustomElementReturn, DomSourceCustomElementUpgrade,
    DomWatchedCustomElementResult, DomWatchedElementCreation,
};
pub use crate::internal::store::source::reactions::{
    DomCustomElementReaction, DomCustomElementReactionKind, DomCustomElementReactions,
    DomSynchronousReactionQueue,
};
