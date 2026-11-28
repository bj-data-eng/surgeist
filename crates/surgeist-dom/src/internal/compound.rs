//! Normalized operands for one named DOM tree-mutation invocation.
use crate::{
    internal::batch::Command,
    internal::hosted::DomCreationRealmFallback,
    internal::identity::{DocumentId, NodeId},
    internal::select::{DomSelectOptionBefore, OptionsEdit},
    internal::source::DomSourceEdit,
    internal::table_part::DomHtmlTablePart,
};
use surgeist_html::{
    elements::input::{HtmlInputStepDirection, HtmlRangeTextTarget, HtmlTextSelectionEdit},
    values::DocumentString,
};

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
#[derive(Clone, Copy, Debug)]
pub(crate) enum TableMember {
    Row,
    Cell,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum TableStructureEdit {
    CreateBody {
        fallback: Option<DomCreationRealmFallback>,
    },
    Insert {
        member: TableMember,
        index: i32,
        fallback: Option<DomCreationRealmFallback>,
    },
    Delete {
        member: TableMember,
        index: i32,
    },
}
#[derive(Clone, Debug)]
pub(crate) enum TextSelectionEdit {
    Selection(HtmlTextSelectionEdit),
    RangeText {
        target: HtmlRangeTextTarget,
        replacement: DocumentString,
    },
}
#[derive(Clone, Debug)]
pub(crate) enum Edit {
    FormReset {
        form: NodeId,
    },
    InputFiles {
        input: NodeId,
        files: Option<crate::file::DomFileList>,
    },
    UserFileSelection {
        input: NodeId,
        observed: crate::internal::identity::DomStamp,
        files: crate::file::DomFileList,
    },
    InputStep {
        input: NodeId,
        direction: HtmlInputStepDirection,
        count: i32,
    },
    SelectOptions {
        select: NodeId,
        action: OptionsEdit,
    },
    UserSelectToggle {
        select: NodeId,
        option: NodeId,
        observed: crate::internal::identity::DomStamp,
    },
    TextSelection {
        receiver: NodeId,
        edit: TextSelectionEdit,
    },
    TablePart {
        table: NodeId,
        part: DomHtmlTablePart,
        action: TablePartEdit,
    },
    TableStructure {
        receiver: NodeId,
        edit: TableStructureEdit,
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
            Self::FormReset { form } => *form,
            Self::InputStep { input, .. }
            | Self::InputFiles { input, .. }
            | Self::UserFileSelection { input, .. } => *input,
            Self::SelectOptions { select, .. } | Self::UserSelectToggle { select, .. } => *select,
            Self::TablePart { table, .. } => *table,
            Self::DocumentTitle { document, .. } => document.node(),
            Self::TextSelection { receiver, .. }
            | Self::TableStructure { receiver, .. }
            | Self::Variadic { receiver, .. }
            | Self::TextContent { receiver, .. }
            | Self::AdjacentElement { receiver, .. }
            | Self::AdjacentText { receiver, .. } => *receiver,
        }
    }
}
impl DomSourceEdit {
    /// Toggle an Option chosen through native interaction at this exact DOM stamp.
    ///
    /// A successful edit publishes the selectedness and dirtiness change with
    /// its original notification task. Both elements must still be eligible.
    pub fn toggle_select_option_for_user(
        select: NodeId,
        option: NodeId,
        observed: crate::internal::identity::DomStamp,
    ) -> Self {
        Self {
            command: Command::Compound(Edit::UserSelectToggle {
                select,
                option,
                observed,
            }),
        }
    }

    /// Reset the original Form through its cancelable event and current
    /// resettable controls. The original Source executor delivers the event.
    pub fn reset_form(form: NodeId) -> Self {
        Self {
            command: Command::Compound(Edit::FormReset { form }),
        }
    }

    /// Assign the supplied FileList to File Upload state. A null argument or
    /// a different Input state leaves the existing selection unchanged.
    /// This assignment dispatches no input or change event.
    pub fn assign_input_files(input: NodeId, files: Option<crate::file::DomFileList>) -> Self {
        Self {
            command: Command::Compound(Edit::InputFiles { input, files }),
        }
    }

    /// Queue files selected by the native host for this exact observed Input.
    /// Source verifies the observation, mutability and selection cardinality.
    /// The original user-interaction task installs the list before delivery of
    /// its input and change events. Picker acquisition is the caller's responsibility.
    pub fn queue_input_file_selection(
        input: NodeId,
        observed: crate::internal::identity::DomStamp,
        files: crate::file::DomFileList,
    ) -> Self {
        Self {
            command: Command::Compound(Edit::UserFileSelection {
                input,
                observed,
                files,
            }),
        }
    }

    /// Invoke native stepUp or stepDown on the current Input state.
    /// `count` is the converted signed long argument.
    pub fn step_input(input: NodeId, direction: HtmlInputStepDirection, count: i32) -> Self {
        Self {
            command: Command::Compound(Edit::InputStep {
                input,
                direction,
                count,
            }),
        }
    }

    /// Edit native selection in an Input or Textarea. Offsets are UTF-16 code
    /// units. A reached event belongs to the original Source continuation.
    pub fn edit_text_selection(receiver: NodeId, edit: HtmlTextSelectionEdit) -> Self {
        Self {
            command: Command::Compound(Edit::TextSelection {
                receiver,
                edit: TextSelectionEdit::Selection(edit),
            }),
        }
    }

    /// Replace text in a native Input or Textarea using its current selection
    /// or an explicit UTF-16 range. The original Source owns the accepted
    /// value, dirty state, selection and any reached event.
    pub fn set_range_text(
        receiver: NodeId,
        replacement: impl Into<DocumentString>,
        target: HtmlRangeTextTarget,
    ) -> Self {
        Self {
            command: Command::Compound(Edit::TextSelection {
                receiver,
                edit: TextSelectionEdit::RangeText {
                    target,
                    replacement: replacement.into(),
                },
            }),
        }
    }
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

    /// Create a fresh direct HTML tbody after the last direct HTML tbody, or
    /// at the end of the Table. Completion returns the created node.
    pub fn create_html_table_body(table: NodeId) -> Self {
        Self::table_structure(table, TableStructureEdit::CreateBody { fallback: None })
    }

    /// Create a fresh tbody with a qualified fallback for a Document without
    /// a native Global.
    pub fn create_html_table_body_in_attachment(
        table: NodeId,
        fallback: DomCreationRealmFallback,
    ) -> Self {
        Self::table_structure(
            table,
            TableStructureEdit::CreateBody {
                fallback: Some(fallback),
            },
        )
    }

    /// Insert a fresh row through a Table or Table section. The already
    /// converted index is signed and `-1` selects the append branch.
    /// Completion returns the created row, including when a tbody was needed.
    pub fn insert_html_table_row(receiver: NodeId, index: i32) -> Self {
        Self::insert_table_member(receiver, TableMember::Row, index, None)
    }

    /// Insert a row using the receiving Document's Global or the supplied
    /// qualified fallback.
    pub fn insert_html_table_row_in_attachment(
        receiver: NodeId,
        index: i32,
        fallback: DomCreationRealmFallback,
    ) -> Self {
        Self::insert_table_member(receiver, TableMember::Row, index, Some(fallback))
    }

    /// Remove the indexed row through a Table or Table section. The `-1`
    /// index removes the last row if one exists.
    pub fn delete_html_table_row(receiver: NodeId, index: i32) -> Self {
        Self::table_structure(
            receiver,
            TableStructureEdit::Delete {
                member: TableMember::Row,
                index,
            },
        )
    }

    /// Insert a fresh HTML td in the Row's ordered direct cells.
    /// Completion returns the created cell.
    pub fn insert_html_table_cell(row: NodeId, index: i32) -> Self {
        Self::insert_table_member(row, TableMember::Cell, index, None)
    }

    /// Insert a cell using the receiving Document's Global or the supplied
    /// qualified fallback.
    pub fn insert_html_table_cell_in_attachment(
        row: NodeId,
        index: i32,
        fallback: DomCreationRealmFallback,
    ) -> Self {
        Self::insert_table_member(row, TableMember::Cell, index, Some(fallback))
    }

    /// Remove the indexed direct HTML cell. The `-1` index removes the last
    /// cell if one exists.
    pub fn delete_html_table_cell(row: NodeId, index: i32) -> Self {
        Self::table_structure(
            row,
            TableStructureEdit::Delete {
                member: TableMember::Cell,
                index,
            },
        )
    }

    fn insert_table_member(
        receiver: NodeId,
        member: TableMember,
        index: i32,
        fallback: Option<DomCreationRealmFallback>,
    ) -> Self {
        Self::table_structure(
            receiver,
            TableStructureEdit::Insert {
                member,
                index,
                fallback,
            },
        )
    }

    fn table_structure(receiver: NodeId, edit: TableStructureEdit) -> Self {
        Self {
            command: Command::Compound(Edit::TableStructure { receiver, edit }),
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
