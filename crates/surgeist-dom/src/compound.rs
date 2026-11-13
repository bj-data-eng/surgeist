//! Normalized operands for one named DOM tree-mutation invocation.
use crate::{DomSourceEdit, NodeId, batch::Command};
use surgeist_html::values::DocumentString;

/// An already converted Node-or-DOMString argument. Strings retain exact UTF-16.
#[derive(Clone, Debug)]
pub enum DomNodeOrString {
    /// One actual native node identity, retaining duplicates in argument order.
    Node(NodeId),
    /// One already converted exact DOMString occurrence.
    String(DocumentString),
}
impl From<NodeId> for DomNodeOrString {
    fn from(value: NodeId) -> Self {
        Self::Node(value)
    }
}
impl From<DocumentString> for DomNodeOrString {
    fn from(value: DocumentString) -> Self {
        Self::String(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Variadic {
    Append,
    Prepend,
    ReplaceChildren,
    Before,
    After,
    ReplaceWith,
}
#[derive(Clone, Debug)]
pub(crate) enum Edit {
    Variadic {
        receiver: NodeId,
        method: Variadic,
        nodes: Vec<DomNodeOrString>,
    },
    TextContent {
        receiver: NodeId,
        value: DocumentString,
    },
    AdjacentText {
        receiver: NodeId,
        position: DocumentString,
        data: DocumentString,
    },
    AdjacentElement {
        receiver: NodeId,
        position: DocumentString,
        element: NodeId,
    },
}
impl Edit {
    pub(crate) fn receiver(&self) -> NodeId {
        match self {
            Self::Variadic { receiver, .. }
            | Self::TextContent { receiver, .. }
            | Self::AdjacentElement { receiver, .. }
            | Self::AdjacentText { receiver, .. } => *receiver,
        }
    }
}
impl DomSourceEdit {
    fn variadic(receiver: NodeId, method: Variadic, nodes: Vec<DomNodeOrString>) -> Self {
        Self {
            command: Command::Compound(Edit::Variadic {
                receiver,
                method,
                nodes,
            }),
        }
    }
    /// Run ParentNode.append, including its original node/string conversion.
    pub fn append(parent: NodeId, nodes: Vec<DomNodeOrString>) -> Self {
        Self::variadic(parent, Variadic::Append, nodes)
    }
    /// Run ParentNode.prepend, selecting the first child after conversion.
    pub fn prepend(parent: NodeId, nodes: Vec<DomNodeOrString>) -> Self {
        Self::variadic(parent, Variadic::Prepend, nodes)
    }
    /// Run ParentNode.replaceChildren under one retained Source invocation.
    pub fn replace_children(parent: NodeId, nodes: Vec<DomNodeOrString>) -> Self {
        Self::variadic(parent, Variadic::ReplaceChildren, nodes)
    }
    /// Run ChildNode.before with the original viable previous sibling.
    pub fn before(node: NodeId, nodes: Vec<DomNodeOrString>) -> Self {
        Self::variadic(node, Variadic::Before, nodes)
    }
    /// Run ChildNode.after with the original viable next sibling.
    pub fn after(node: NodeId, nodes: Vec<DomNodeOrString>) -> Self {
        Self::variadic(node, Variadic::After, nodes)
    }
    /// Run ChildNode.replaceWith, rechecking the original parent after conversion.
    pub fn replace_with(node: NodeId, nodes: Vec<DomNodeOrString>) -> Self {
        Self::variadic(node, Variadic::ReplaceWith, nodes)
    }
    /// Set Node.textContent. Binding conversion maps null to the empty string.
    /// Attr has its separate identity/attribute-value owner.
    pub fn set_text_content(node: NodeId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::Compound(Edit::TextContent {
                receiver: node,
                value: value.into(),
            }),
        }
    }
    /// Create Text, then interpret the exact ASCII-insensitive position string.
    pub fn insert_adjacent_text(
        element: NodeId,
        position: impl Into<DocumentString>,
        data: impl Into<DocumentString>,
    ) -> Self {
        Self {
            command: Command::Compound(Edit::AdjacentText {
                receiver: element,
                position: position.into(),
                data: data.into(),
            }),
        }
    }
    /// Insert the original Element at the selected adjacent position. The
    /// completion reports its original identity, or null for an outside
    /// position on a detached receiver without adopting the operand.
    pub fn insert_adjacent_element(
        receiver: NodeId,
        position: impl Into<DocumentString>,
        element: NodeId,
    ) -> Self {
        Self {
            command: Command::Compound(Edit::AdjacentElement {
                receiver,
                position: position.into(),
                element,
            }),
        }
    }
}
