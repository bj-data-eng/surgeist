//! Current authored Display3 values and the selected standalone Grid3 extensions.

use crate::{
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssVisibility,
};

/// The outer display type of an ordinary display value.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayOutside {
    Block,
    Inline,
    RunIn,
}

/// The inner display type permitted by the selected Display3 grammar.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayInside {
    Flow,
    FlowRoot,
    Table,
    Flex,
    Grid,
    Ruby,
}

/// The flow modes permitted in a list-item display value.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayListItemInside {
    Flow,
    FlowRoot,
}

/// An exclusive layout-internal display keyword.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayInternal {
    TableRowGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableRow,
    TableCell,
    TableColumnGroup,
    TableColumn,
    TableCaption,
    RubyBase,
    RubyText,
    RubyBaseContainer,
    RubyTextContainer,
}

/// An exclusive box-generation display keyword.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayBox {
    Contents,
    None,
}

/// A precomposed keyword whose specified identity differs from its computed pair.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayLegacy {
    InlineBlock,
    InlineTable,
    InlineFlex,
    InlineGrid,
}

/// A complete ordinary specified display value, before cascade or box generation.
///
/// Missing grammar components are stored with their intrinsic defaults. Legacy
/// keywords retain distinct specified identities. Original spelling and source
/// provenance remain on the authored property wrapper.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDisplayValue {
    OutsideInside {
        outside: CssDisplayOutside,
        inside: CssDisplayInside,
    },
    ListItem {
        outside: CssDisplayOutside,
        inside: CssDisplayListItemInside,
    },
    Internal(CssDisplayInternal),
    Box(CssDisplayBox),
    Legacy(CssDisplayLegacy),
    GridLanes,
    InlineGridLanes,
}

impl CssDisplayValue {
    /// Serializes the specified value without computed-value transformations.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically, charging one input node, one projection node,
    /// and the exact output byte count. Authored input is never modified.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        serialize_keyword_sequence(self.specified_keyword_sequence(), limits)
    }

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

impl CssVisibility {
    /// Serializes the specified visibility keyword without applying inheritance
    /// or changing box rendering and layout.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically, charging one input node, one projection node,
    /// and the exact output byte count. Authored input is never modified.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
            Self::Collapse => "collapse",
        };
        serialize_keyword_sequence(text, limits)
    }
}

fn serialize_keyword_sequence(
    text: &str,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String, CssSpecifiedValueSerializationError> {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    if limits.max_input_nodes() == 0 {
        return Err(CssSpecifiedValueSerializationError::new(
            Kind::InputNodeLimit,
        ));
    }
    if limits.max_projection_nodes() == 0 {
        return Err(CssSpecifiedValueSerializationError::new(
            Kind::ProjectionNodeLimit,
        ));
    }
    if text.len() > limits.max_css_bytes() {
        return Err(CssSpecifiedValueSerializationError::new(Kind::ByteLimit));
    }
    Ok(text.to_owned())
}
