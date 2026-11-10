//! The error of building a document.

use std::fmt;

use crate::Offset;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BuildErrorKind {
    DepthLimit,
    NodeLimit,
    ByteLimit,
    CapacityOverflow,
    AllocationFailure,
    Refused,
}

/// Why a document could not be built.
///
/// Building never fails because of what the text says (tokenizing and tree
/// construction accept every text); it fails because a resource ran out or the
/// host said stop.
///
/// ```
/// use surgeist_css_parser::Document;
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
///
/// let limits = Limits::new(1, usize::MAX, usize::MAX);
/// let error = Document::<()>::parse("((a))", &Options::default(), limits, 0, DepthExcess::Fail)
///     .unwrap_err();
/// assert!(error.is_depth_limit());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildError {
    kind: BuildErrorKind,
    offset: Option<Offset>,
}

impl BuildError {
    pub(super) const fn new(kind: BuildErrorKind, offset: Option<Offset>) -> Self {
        Self { kind, offset }
    }

    /// Returns the offset of the token whose admission failed, when it has
    /// one.
    #[must_use]
    pub const fn offset(&self) -> Option<Offset> {
        self.offset
    }

    kind_predicates! { kind;
        /// Whether functions and simple blocks nested deeper than
        /// [`Limits::max_depth`](crate::limits::Limits::max_depth).
        is_depth_limit => BuildErrorKind::DepthLimit,
        /// Whether the document would hold more components than
        /// [`Limits::max_nodes`](crate::limits::Limits::max_nodes).
        is_node_limit => BuildErrorKind::NodeLimit,
        /// Whether the admitted tokens would span more bytes than
        /// [`Limits::max_bytes`](crate::limits::Limits::max_bytes).
        is_byte_limit => BuildErrorKind::ByteLimit,
        /// Whether a count or size could not be represented: a size or count
        /// that overflows `usize`, more than 2^32 nodes (or nodes in one
        /// list), more than `u32::MAX` lists, or, for a document built from
        /// text, a pool of decoded names and strings of more than `u32::MAX`
        /// bytes.
        is_capacity_overflow => BuildErrorKind::CapacityOverflow,
        /// Whether the allocator could not provide the storage.
        is_allocation_failure => BuildErrorKind::AllocationFailure,
        /// Whether the [`Meter`](crate::limits::Meter) refused an action or
        /// storage; the host keeps its own reason.
        is_refused => BuildErrorKind::Refused,
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.kind {
            BuildErrorKind::DepthLimit => "functions and blocks nest deeper than the limit",
            BuildErrorKind::NodeLimit => "there are more components than the limit",
            BuildErrorKind::ByteLimit => "the tokens span more bytes than the limit",
            BuildErrorKind::CapacityOverflow => "a count or size is not representable",
            BuildErrorKind::AllocationFailure => "the allocator could not provide storage",
            BuildErrorKind::Refused => "the meter refused the action",
        };
        formatter.write_str(what)?;
        if let Some(offset) = self.offset {
            write!(formatter, " (at byte {})", offset.value())?;
        }
        Ok(())
    }
}

impl std::error::Error for BuildError {}
