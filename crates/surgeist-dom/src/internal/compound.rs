//! Normalized operands for one named DOM tree-mutation invocation.
use crate::{
    internal::batch::Command,
    internal::hosted::DomCreationRealmFallback,
    internal::identity::{DocumentId, NodeId},
    internal::select::{DomSelectOptionBefore, OptionsEdit},
    internal::source::DomSourceEdit,
    internal::table_part::DomHtmlTablePart,
};
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
#[derive(Clone, Copy, Debug)]
pub(crate) enum TablePartEdit {
    Set(Option<NodeId>),
    Create {
        fallback: Option<DomCreationRealmFallback>,
    },
}
#[derive(Clone, Debug)]
pub(crate) enum Edit {
    SelectOptions {
        select: NodeId,
        action: OptionsEdit,
    },
    TablePart {
        table: NodeId,
        part: DomHtmlTablePart,
        action: TablePartEdit,
    },
    DocumentTitle {
        document: DocumentId,
        value: DocumentString,
    },
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
            Self::SelectOptions { select, .. } => *select,
            Self::TablePart { table, .. } => *table,
            Self::DocumentTitle { document, .. } => document.node(),
            Self::Variadic { receiver, .. }
            | Self::TextContent { receiver, .. }
            | Self::AdjacentElement { receiver, .. }
            | Self::AdjacentText { receiver, .. } => *receiver,
        }
    }
}
impl DomSourceEdit {
    /// Set the Select's options length through native insertion or removal.
    pub fn set_select_options_length(select: NodeId, length: u32) -> Self {
        Self::select_options_edit(select, OptionsEdit::Length(length))
    }

    /// Replace or append an Option at this index. A missing value removes
    /// the current Option. Sparse insertion fills the gap with blank Options.
    pub fn set_select_option_at(select: NodeId, index: u32, value: Option<NodeId>) -> Self {
        Self::select_options_edit(select, OptionsEdit::Set { index, value })
    }

    /// Insert an Option or Optgroup at the selected ordinary reference.
    pub fn add_select_option(
        select: NodeId,
        element: NodeId,
        before: DomSelectOptionBefore,
    ) -> Self {
        Self::select_options_edit(select, OptionsEdit::Add { element, before })
    }

    /// Remove the Option at this signed position if it currently exists.
    pub fn remove_select_option(select: NodeId, index: i32) -> Self {
        Self::select_options_edit(select, OptionsEdit::Remove(index))
    }

    fn select_options_edit(select: NodeId, action: OptionsEdit) -> Self {
        Self {
            command: Command::Compound(Edit::SelectOptions { select, action }),
        }
    }

    /// Replace the first direct HTML part of this Table through one Source operation.
    ///
    /// A supplied node must be the selected HTML part kind. After the current
    /// part is removed, the candidate is inserted at the prescribed position.
    pub fn set_html_table_part(
        table: NodeId,
        part: DomHtmlTablePart,
        value: Option<NodeId>,
    ) -> Self {
        Self::html_table_part(table, part, TablePartEdit::Set(value))
    }

    /// Reuse the first direct HTML part or create and insert one in the Table's
    /// current node document. Completion returns [DomSourceOutput::Node](crate::source::DomSourceOutput::Node).
    pub fn create_html_table_part(table: NodeId, part: DomHtmlTablePart) -> Self {
        Self::html_table_part(table, part, TablePartEdit::Create { fallback: None })
    }

    /// Create or reuse the selected part, with a qualified creation realm
    /// available for a receiving Document that has no native Global.
    pub fn create_html_table_part_in_attachment(
        table: NodeId,
        part: DomHtmlTablePart,
        fallback: DomCreationRealmFallback,
    ) -> Self {
        Self::html_table_part(
            table,
            part,
            TablePartEdit::Create {
                fallback: Some(fallback),
            },
        )
    }

    /// Remove the first direct HTML part if present.
    pub fn delete_html_table_part(table: NodeId, part: DomHtmlTablePart) -> Self {
        Self::html_table_part(table, part, TablePartEdit::Set(None))
    }

    fn html_table_part(table: NodeId, part: DomHtmlTablePart, action: TablePartEdit) -> Self {
        Self {
            command: Command::Compound(Edit::TablePart {
                table,
                part,
                action,
            }),
        }
    }

    /// Set Document.title through one canonical native Source invocation.
    pub fn set_document_title(document: DocumentId, value: DocumentString) -> Self {
        Self {
            command: Command::Compound(Edit::DocumentTitle { document, value }),
        }
    }
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
