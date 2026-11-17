//! URL-bearing elements: Base, Link, hyperlinks and images, and the Source URL requests.
//!
//! `DomBase` and `DomFrozenBase` read a Base and its frozen URL record, and
//! `DomBaseResponse` is a native response publication. `DomLink` and `DomHyperlink` read
//! Link, A and Area state, and `DomImg`, `DomImageUpdate` and `DomImageUpdateId` read an
//! Img and its pending update invocation. The `DomSourceUrl*` types are the Source
//! continuation for URL processing.
//!
//! A Base's frozen URL record keeps its history after it loses first position, loses its
//! `href`, detaches or is adopted. `docs/urls-and-metadata.md` and
//! `docs/embedded-content.md` own the contracts.

pub use crate::internal::base::{
    DomBase, DomBaseFactsError, DomBaseResponse, DomFrozenBase, DomLink,
};
pub use crate::internal::hyperlink::{DomEncodingUrlFactsError, DomHyperlink};
pub use crate::internal::image::{
    DomImageUpdate, DomImageUpdateCancellationReason, DomImageUpdateId, DomImg,
};
pub use crate::internal::source::{
    DomSourceUrlObservation, DomSourceUrlOrigin, DomSourceUrlRequest, DomSourceUrlStage,
};
