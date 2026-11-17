//! Style element association: sheets, sources, state, CSSOM roots and notifications.
//!
//! `DomStyle` and `DomStyleState` read a native Style element, which retains its creation
//! history and a nullable associated sheet. `DomStyleCssomRoot` and
//! `DomStyleCssomOwnerBinding` give access to a mutable CSSOM root and bind its owner,
//! `DomStyleSheet` is a sheet actually associated by the composed CSSOM protocol, and
//! `DomStyleSource` and `DomStyleRequest` describe the source-time inputs and the
//! outstanding external step. The notification types (`DomStyleNotificationToken`,
//! `DomStyleNotificationDelivery` and related) carry the post-publication notice. Inline
//! `style` attributes live in `style::inline`.
//!
//! Ordinary atomic edits refuse at an external processing boundary without publishing a
//! partial edit. `docs/source-and-style.md` owns the contract.

pub mod inline;

pub use crate::internal::store::source::operation::DomStyleCspBudget;
pub use crate::internal::style::{
    DomStyle, DomStyleAction, DomStyleAssociationId, DomStyleCspRetention,
    DomStyleCssomOwnerBinding, DomStyleCssomPublicationRejection, DomStyleCssomRoot,
    DomStyleCssomRootError, DomStyleInlineCspObservation, DomStyleNonceContent,
    DomStyleNotificationCompletion, DomStyleNotificationDelivery, DomStyleNotificationOrigin,
    DomStyleNotificationToken, DomStyleRequest, DomStyleRootKind, DomStyleSheet,
    DomStyleSheetAvailability, DomStyleSheetPlacement, DomStyleSource, DomStyleState,
};
