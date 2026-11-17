//! HTML and SVG Script state, Document scripting registration and the Document
//! constructor.
//!
//! `DomHtmlScript` and `DomSvgScript` read the retained Script state: HTML preparation
//! flags and Documents, and SVG's processed flag with its creation cause
//! (`DomSvgScriptOrigin`). `DomDocumentScriptRegistration` owns the lifetime of the
//! engine registered for a Document, `DomDocumentScriptBinding` is the association its
//! hosted issuer publishes, `DomDocumentScriptingFacts` carry the settings the trusted
//! adapter supplies, and `DomDocumentScriptingObservation` is the immutable fact for one
//! revision. `DomDocumentConstructorContext` and `DomDocumentConstructorError` serve the
//! Document constructor, and the `DomSourceScript*` types are the Source continuation for
//! preparation.
//!
//! Preparation reaches the Disabled return without fetching or executing, and reading an
//! SVG Script grants no acquisition, execution or load/error delivery. `docs/scripts.md`
//! owns the contract.

pub use crate::internal::hosted::{
    DomCreationRealmFallback, DomDocumentConstructorContext, DomDocumentConstructorError,
    DomDocumentScriptBinding, DomDocumentScriptRegistration, DomDocumentScriptingFacts,
    DomDocumentScriptingObservation,
};
pub use crate::internal::script::DomHtmlScript;
pub use crate::internal::source::{
    DomSourceScriptRequest, DomSourceScriptSettings, DomSourceScriptStep,
};
pub use crate::internal::svg_script::{DomSvgScript, DomSvgScriptOrigin};
