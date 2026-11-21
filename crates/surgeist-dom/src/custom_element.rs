//! Custom element registries, definitions, reactions and the Source constructor protocol.
//!
//! `DomRegistry` reads a registry at one snapshot, `DomRegistryId` names one, and
//! `DomRegistryBinding` is the registry that controlled creation selects.
//! `DomElementDefinition` is an element's associated definition, and
//! `DomRegistryDefinitionId` and `DomRegistryDefinitionPreparation` identify and publish
//! one. `DomCustomElementReaction` is one reaction occurrence in the order the edit
//! reached it, `DomCustomElementReactions` one published segment's reactions,
//! `DomSynchronousReactionQueue` a synchronous element queue minted by a script
//! registration, and `DomShadowReactionRoute` a registration with its callback lookup
//! for shadow removal. The `DomSourceCustomElement*` types are the Source continuation for
//! constructor, upgrade and callback work, and `DomWatchedElementCreation` and
//! `DomWatchedCustomElementResult` report a factory's creation with retirement custody.
//!
//! The crate documentation has no custom-element page: `docs/source-and-style.md`
//! describes the shared Source continuation and `docs/document-creation.md` the
//! known-null registry of a native document.

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
