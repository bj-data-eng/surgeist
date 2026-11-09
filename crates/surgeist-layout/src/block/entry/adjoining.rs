//! The adjoining component and normal-flow cursor of one Block traversal.
use super::*;

/// Scalar flow facts are independent of staged descendant geometry. The body
/// driver and entry qualification must use the same start and end reductions.
pub(in crate::block) struct AdjoiningFlow<S: LayoutScalar> {
    pub(in crate::block) cursor: S,
    pub(in crate::block) active: CollapsibleMarginOf<S>,
    pub(in crate::block) top: CollapsibleMarginOf<S>,
    pub(in crate::block) start_open: bool,
    pub(in crate::block) end_adjoins_parent: bool,
    pub(in crate::block) all_through: bool,
}

impl<S: LayoutScalar> AdjoiningFlow<S> {
    pub(in crate::block) fn new(cursor: S, collapses_start: bool) -> Self {
        Self {
            cursor,
            active: CollapsibleMarginOf::ZERO,
            top: CollapsibleMarginOf::ZERO,
            start_open: collapses_start,
            end_adjoins_parent: collapses_start,
            all_through: true,
        }
    }

    pub(in crate::block) fn reduce_start(
        &self,
        collapses_start: bool,
        child_adjoins_parent: bool,
        child_start: CollapsibleMarginOf<S>,
    ) -> StartReduction<S> {
        reduce_start(
            self.start_open,
            collapses_start,
            child_adjoins_parent,
            self.active,
            child_start,
        )
    }

    /// Keep the full extrema of an exposed leading group outside the content.
    /// Fragment contributor identities follow this same transition in their
    /// existing flat contributor arena.
    pub(in crate::block) fn begin_child(
        &mut self,
        start: &StartReduction<S>,
        top: CollapsibleMarginOf<S>,
        bottom: CollapsibleMarginOf<S>,
        through: bool,
    ) {
        if self.start_open {
            if start.exposes_child {
                self.top = self.top.collapse_with(top);
                if through {
                    self.top = self.top.collapse_with(bottom);
                }
            }
            self.start_open = start.exposes_child && through;
        }
        self.cursor = self.cursor + start.offset;
    }

    /// The placement owner supplies the exact accepted progression separately
    /// from the child's border position. Through-empty boxes retain the open
    /// group's extrema; a non-through body starts a new trailing component.
    pub(in crate::block) fn finish_child(
        &mut self,
        cursor: S,
        top: CollapsibleMarginOf<S>,
        bottom: CollapsibleMarginOf<S>,
        through: bool,
        child_adjoins_parent: bool,
    ) {
        self.cursor = cursor;
        if through {
            self.active = self.active.collapse_with(top).collapse_with(bottom);
        } else {
            self.all_through = false;
            self.active = bottom;
        }
        self.end_adjoins_parent = child_adjoins_parent;
    }

    pub(in crate::block) fn adjoining_displacement(&self) -> S {
        if self.start_open {
            S::ZERO
        } else {
            self.active.resolve()
        }
    }

    /// Only an actually participating line ends the adjoining component.
    /// A nonformatted run can still advance its separately accepted cursor.
    pub(in crate::block) fn finish_run(&mut self, cursor: S, separates: bool) {
        self.cursor = cursor;
        if separates {
            self.start_open = false;
            self.active = CollapsibleMarginOf::ZERO;
            self.end_adjoins_parent = false;
            self.all_through = false;
        }
    }
}
