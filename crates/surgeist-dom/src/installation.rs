//! Native HTML fragment installation and the completed parser-report installation.
//!
//! `Dom::install_native_html_fragment` consumes an immutable HTML fragment as a
//! fresh-native authoring input and returns a `DomNativeHtmlInstallation`;
//! `DomNativeHtmlInstallLimits` and `DomNativeHtmlInstallError` bound and report it. A
//! completed offline parser report installs separately and returns a
//! `DomParsedHtmlInstallation` with `DomParsedHtmlInstallLimits` and
//! `DomParsedHtmlInstallError`; the pending associations it leaves are
//! `DomParsedHtmlPendingAssociations`.
//!
//! Both run through the same canonical owner and return immutable, source-qualified
//! historical bindings. A refused installation publishes no partial map, tree, revision
//! or history. Pending stylesheet and Meta associations of a parsed report are in
//! `installation::style` and `installation::meta`. `docs/html-installation.md` owns the
//! contract.

pub mod meta;
pub mod style;

pub use crate::internal::installation::{
    DomNativeHtmlAssociation, DomNativeHtmlBinding, DomNativeHtmlInstallError,
    DomNativeHtmlInstallLimits, DomNativeHtmlInstallResource, DomNativeHtmlInstallTarget,
    DomNativeHtmlInstallation,
};
pub use crate::internal::parsed_association::{
    DomParsedHtmlAssociationAdmissionError, DomParsedHtmlAssociationLimits,
    DomParsedHtmlAssociationRuntimeUsage,
};
pub use crate::internal::parsed_installation::{
    DomParsedDocumentEnvironment, DomParsedHtmlAssociationDocument, DomParsedHtmlAssociationId,
    DomParsedHtmlAssociationReadiness, DomParsedHtmlBinding, DomParsedHtmlCapturedBinding,
    DomParsedHtmlHistoricalDocument, DomParsedHtmlInstallError, DomParsedHtmlInstallLimits,
    DomParsedHtmlInstallResource, DomParsedHtmlInstallation, DomParsedHtmlPendingAssociations,
};
pub use crate::internal::source::DomSourceNativeHtmlInstallLimits;
