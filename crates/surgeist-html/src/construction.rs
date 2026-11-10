//! Bounded native authoring and immutable, result-local construction output.

pub use crate::internal::construction::{
    ForeignHtmlResult, HtmlAuthoredProcessingInstruction, HtmlBuilderNode, HtmlConstructionCause,
    HtmlConstructionError, HtmlConstructionLimits, HtmlConstructionResource, HtmlConstructionUsage,
    HtmlDeclarativeShadowRoot, HtmlDoctype, HtmlDocument, HtmlDocumentBuilder, HtmlDocumentMode,
    HtmlElement, HtmlFocusedAreaDesignation, HtmlFragment, HtmlFragmentBuilder, HtmlFragmentOwner,
    HtmlNode, HtmlNodeId, HtmlNodeKind, HtmlProcessingInstruction, HtmlProcessingInstructionError,
    HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment, HtmlTemplateContentsKind,
};
