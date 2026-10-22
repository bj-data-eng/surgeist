//! Layout-owned line indentation and the actual block-generation context.

use crate::{LayoutScalar, TextIndentOf};

/// Supplied only by the actual enclosing block algorithm, never a public
/// producer assertion about which line will be formatted first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BlockIndentContext<S: LayoutScalar> {
    pub inherited: TextIndentOf<S>,
    pub first_formatted: bool,
}

/// Progress across committed lines, including administrative run boundaries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IndentLineState {
    first_formatted: bool,
    block_start: bool,
    follows_forced: bool,
}

impl IndentLineState {
    pub const fn new(first_formatted: bool) -> Self {
        Self {
            first_formatted,
            block_start: true,
            follows_forced: false,
        }
    }

    pub fn commit(&mut self, forced: bool) {
        self.first_formatted = false;
        self.block_start = false;
        self.follows_forced = forced;
    }
}

impl Default for IndentLineState {
    fn default() -> Self {
        Self::new(true)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ResolvedTextIndent<S: LayoutScalar> {
    pub amount: S,
    pub each_line: bool,
    pub hanging: bool,
}

impl<S: LayoutScalar> ResolvedTextIndent<S> {
    pub const ZERO: Self = Self {
        amount: S::ZERO,
        each_line: false,
        hanging: false,
    };
    pub fn at(self, state: IndentLineState) -> S {
        let affected = state.first_formatted
            || (self.each_line && (state.block_start || state.follows_forced));
        if affected ^ self.hanging {
            self.amount
        } else {
            S::ZERO
        }
    }
    pub fn widened(self) -> ResolvedTextIndent<f64> {
        ResolvedTextIndent {
            amount: self.amount.to_f64(),
            each_line: self.each_line,
            hanging: self.hanging,
        }
    }
}

impl<S: LayoutScalar> Default for ResolvedTextIndent<S> {
    fn default() -> Self {
        Self::ZERO
    }
}
