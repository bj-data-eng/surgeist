//! Current authored Display3 and Containment2 visibility values, plus the
//! selected standalone Grid3 extensions.

use crate::values::box_model::{CssContentVisibility, CssVisibility};

/// The outer display type of an ordinary display value.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayOutside {
    /// `block`: the element generates a box that is block-level when placed in flow layout.
    Block,
    /// `inline`: the element generates a box that is inline-level when placed in flow layout.
    Inline,
    /// `run-in`: the box is a run-in box, an inline-level box that attempts to merge into a subsequent block container.
    RunIn,
}

/// The inner display type permitted by the selected Display3 grammar.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayInside {
    /// `flow`: the box lays out its contents using flow layout (block-and-inline layout).
    Flow,
    /// `flow-root`: the box is a block container that lays out its contents using flow layout and always establishes a new block formatting context.
    FlowRoot,
    /// `table`: the box is a principal table wrapper box that establishes a block formatting context and contains a table grid box, which establishes a table formatting context.
    Table,
    /// `flex`: the box is a principal flex container that establishes a flex formatting context.
    Flex,
    /// `grid`: the box is a principal grid container that establishes a grid formatting context (a subgrid might not).
    Grid,
    /// `ruby`: the box is a ruby container that establishes a ruby formatting context.
    Ruby,
}

/// The flow modes permitted in a list-item display value.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayListItemInside {
    /// `flow`: the list item's principal box lays out its contents using flow layout.
    Flow,
    /// `flow-root`: the list item's principal box is a block container that always establishes a new block formatting context for its contents.
    FlowRoot,
}

/// An exclusive layout-internal display keyword.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayInternal {
    /// `table-row-group`: an internal table element that groups one or more rows.
    TableRowGroup,
    /// `table-header-group`: like `table-row-group`, but displayed before all other rows and row groups and after any top captions.
    TableHeaderGroup,
    /// `table-footer-group`: like `table-row-group`, but displayed after all other rows and row groups and before any bottom captions.
    TableFooterGroup,
    /// `table-row`: an internal table element that is a row of cells.
    TableRow,
    /// `table-cell`: an internal table element that represents a table cell; its inner display type is `flow-root`.
    TableCell,
    /// `table-column-group`: an internal table element that groups one or more columns.
    TableColumnGroup,
    /// `table-column`: an internal table element that describes a column of cells.
    TableColumn,
    /// `table-caption`: a table caption box, a block box with special behavior with respect to table and table wrapper boxes; its inner display type is `flow-root`.
    TableCaption,
    /// `ruby-base`: an internal ruby element; its inner display type is `flow`.
    RubyBase,
    /// `ruby-text`: an internal ruby element; its inner display type is `flow`.
    RubyText,
    /// `ruby-base-container`: an internal ruby element.
    RubyBaseContainer,
    /// `ruby-text-container`: an internal ruby element.
    RubyTextContainer,
}

/// An exclusive box-generation display keyword.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayBox {
    /// `contents`: the element generates no box of its own; for box generation and layout it is treated as replaced by its children and pseudo-elements.
    Contents,
    /// `none`: the element and its descendants generate no boxes or text sequences.
    None,
}

/// A precomposed keyword whose specified identity differs from its computed pair.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayLegacy {
    /// `inline-block`: computes to `inline flow-root`, but keeps a distinct specified value.
    InlineBlock,
    /// `inline-table`: computes to `inline table`, but keeps a distinct specified value.
    InlineTable,
    /// `inline-flex`: computes to `inline flex`, but keeps a distinct specified value.
    InlineFlex,
    /// `inline-grid`: computes to `inline grid`, but keeps a distinct specified value.
    InlineGrid,
}

/// A complete ordinary specified display value, before cascade or box generation.
///
/// Missing grammar components are stored with their intrinsic defaults. Legacy
/// keywords retain distinct specified identities. Original spelling and source
/// provenance remain on the authored property wrapper.
///
/// # Specified serialization
///
/// Serializes the specified value without computed-value transformations.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayValue {
    /// An ordinary `<display-outside> || <display-inside>` value; omitted components hold their intrinsic defaults.
    OutsideInside {
        /// The outer display type.
        outside: CssDisplayOutside,
        /// The inner display type.
        inside: CssDisplayInside,
    },
    /// A `list-item` value with its outer display type and flow mode.
    ListItem {
        /// The outer display type.
        outside: CssDisplayOutside,
        /// The flow mode of the list item's contents.
        inside: CssDisplayListItemInside,
    },
    /// An exclusive `<display-internal>` keyword.
    Internal(CssDisplayInternal),
    /// An exclusive `<display-box>` keyword.
    Box(CssDisplayBox),
    /// A precomposed `<display-legacy>` keyword.
    Legacy(CssDisplayLegacy),
    /// `grid-lanes`: the element generates a block-level grid lanes container box.
    GridLanes,
    /// `inline-grid-lanes`: the element generates an inline-level grid lanes container box.
    InlineGridLanes,
}

impl CssDisplayValue {
    fn specified_keyword_sequence(self) -> &'static str {
        use CssDisplayOutside::{Block, Inline, RunIn};
        match self {
            Self::OutsideInside {
                outside: Block,
                inside: CssDisplayInside::Flow,
            } => "block",
            Self::OutsideInside {
                outside: Inline,
                inside: CssDisplayInside::Flow,
            } => "inline",
            Self::OutsideInside {
                outside: RunIn,
                inside: CssDisplayInside::Flow,
            } => "run-in",
            Self::OutsideInside {
                outside: Block,
                inside: CssDisplayInside::FlowRoot,
            } => "flow-root",
            Self::OutsideInside {
                outside: Inline,
                inside: CssDisplayInside::FlowRoot,
            } => "inline flow-root",
            Self::OutsideInside {
                outside: RunIn,
                inside: CssDisplayInside::FlowRoot,
            } => "run-in flow-root",
            Self::OutsideInside {
                outside: Block,
                inside: CssDisplayInside::Table,
            } => "table",
            Self::OutsideInside {
                outside: Inline,
                inside: CssDisplayInside::Table,
            } => "inline table",
            Self::OutsideInside {
                outside: RunIn,
                inside: CssDisplayInside::Table,
            } => "run-in table",
            Self::OutsideInside {
                outside: Block,
                inside: CssDisplayInside::Flex,
            } => "flex",
            Self::OutsideInside {
                outside: Inline,
                inside: CssDisplayInside::Flex,
            } => "inline flex",
            Self::OutsideInside {
                outside: RunIn,
                inside: CssDisplayInside::Flex,
            } => "run-in flex",
            Self::OutsideInside {
                outside: Block,
                inside: CssDisplayInside::Grid,
            } => "grid",
            Self::OutsideInside {
                outside: Inline,
                inside: CssDisplayInside::Grid,
            } => "inline grid",
            Self::OutsideInside {
                outside: RunIn,
                inside: CssDisplayInside::Grid,
            } => "run-in grid",
            Self::OutsideInside {
                outside: Block,
                inside: CssDisplayInside::Ruby,
            } => "block ruby",
            Self::OutsideInside {
                outside: Inline,
                inside: CssDisplayInside::Ruby,
            } => "ruby",
            Self::OutsideInside {
                outside: RunIn,
                inside: CssDisplayInside::Ruby,
            } => "run-in ruby",
            Self::ListItem {
                outside: Block,
                inside: CssDisplayListItemInside::Flow,
            } => "list-item",
            Self::ListItem {
                outside: Inline,
                inside: CssDisplayListItemInside::Flow,
            } => "inline list-item",
            Self::ListItem {
                outside: RunIn,
                inside: CssDisplayListItemInside::Flow,
            } => "run-in list-item",
            Self::ListItem {
                outside: Block,
                inside: CssDisplayListItemInside::FlowRoot,
            } => "flow-root list-item",
            Self::ListItem {
                outside: Inline,
                inside: CssDisplayListItemInside::FlowRoot,
            } => "inline flow-root list-item",
            Self::ListItem {
                outside: RunIn,
                inside: CssDisplayListItemInside::FlowRoot,
            } => "run-in flow-root list-item",
            Self::Internal(CssDisplayInternal::TableRowGroup) => "table-row-group",
            Self::Internal(CssDisplayInternal::TableHeaderGroup) => "table-header-group",
            Self::Internal(CssDisplayInternal::TableFooterGroup) => "table-footer-group",
            Self::Internal(CssDisplayInternal::TableRow) => "table-row",
            Self::Internal(CssDisplayInternal::TableCell) => "table-cell",
            Self::Internal(CssDisplayInternal::TableColumnGroup) => "table-column-group",
            Self::Internal(CssDisplayInternal::TableColumn) => "table-column",
            Self::Internal(CssDisplayInternal::TableCaption) => "table-caption",
            Self::Internal(CssDisplayInternal::RubyBase) => "ruby-base",
            Self::Internal(CssDisplayInternal::RubyText) => "ruby-text",
            Self::Internal(CssDisplayInternal::RubyBaseContainer) => "ruby-base-container",
            Self::Internal(CssDisplayInternal::RubyTextContainer) => "ruby-text-container",
            Self::Box(CssDisplayBox::Contents) => "contents",
            Self::Box(CssDisplayBox::None) => "none",
            Self::Legacy(CssDisplayLegacy::InlineBlock) => "inline-block",
            Self::Legacy(CssDisplayLegacy::InlineTable) => "inline-table",
            Self::Legacy(CssDisplayLegacy::InlineFlex) => "inline-flex",
            Self::Legacy(CssDisplayLegacy::InlineGrid) => "inline-grid",
            Self::GridLanes => "grid-lanes",
            Self::InlineGridLanes => "inline-grid-lanes",
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssDisplayValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(self.specified_keyword_sequence())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssDisplayValue);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssVisibility {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let text = match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
            Self::Collapse => "collapse",
        };
        writer.keyword(text)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssVisibility);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssContentVisibility
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let text = match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
            Self::Auto => "auto",
        };
        writer.keyword(text)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssContentVisibility);
