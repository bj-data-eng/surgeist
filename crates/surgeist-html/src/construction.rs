//! Bounded native authoring and immutable, result-local construction output.

pub use crate::internal::construction::bounded::HtmlConstructionResult;
pub use crate::internal::construction::{
    BuildStr, ConstructionDoctype, ConstructionNodeKind, ConstructionProcessingInstruction,
    ForeignHtmlResult, HtmlAttributes, HtmlAuthoredProcessingInstruction, HtmlBuilderNode,
    HtmlConstructionCause, HtmlConstructionError, HtmlConstructionLimits, HtmlConstructionResource,
    HtmlConstructionUsage, HtmlDeclarativeShadowRoot, HtmlDoctype, HtmlDocument,
    HtmlDocumentBuilder, HtmlDocumentMode, HtmlElementView, HtmlFocusedAreaDesignation,
    HtmlFragment, HtmlFragmentBuilder, HtmlFragmentOwner, HtmlNode, HtmlNodeId, HtmlNodeKind,
    HtmlProcessingInstruction, HtmlProcessingInstructionError, HtmlShadowRootMode,
    HtmlShadowRootProperties, HtmlSlotAssignment, HtmlTemplateContentsKind,
};
