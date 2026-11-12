//! Bounded native authoring and immutable, result-local construction output.

pub use crate::internal::construction::bounded::HtmlConstructionResult;
pub use crate::internal::construction::{
    BuildAttribute, BuildAttributeIter, BuildAttributes, BuildStr, BuildText, ConstructionDoctype,
    ConstructionElementView, ConstructionNodeKind, ConstructionProcessingInstruction,
    ForeignHtmlResult, HtmlAttribute, HtmlAttributes, HtmlAuthoredProcessingInstruction,
    HtmlBuilderNode, HtmlConstructionCause, HtmlConstructionError, HtmlConstructionLimits,
    HtmlConstructionResource, HtmlConstructionUsage, HtmlDeclarativeShadowRoot, HtmlDoctype,
    HtmlDocument, HtmlDocumentBuilder, HtmlDocumentMode, HtmlElementView,
    HtmlFocusedAreaDesignation, HtmlFragment, HtmlFragmentBuilder, HtmlFragmentOwner, HtmlNode,
    HtmlNodeId, HtmlNodeKind, HtmlProcessingInstruction, HtmlProcessingInstructionError,
    HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment, HtmlTemplateContentsKind,
};
