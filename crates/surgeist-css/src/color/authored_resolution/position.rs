//! Fixed ancestry observation from the actual current/returning typed frame.

use crate::color::CssColorPathStep;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Position {
    pub(super) site: usize,
    pub(super) depth: usize,
    /// An actual entering child edge; None is the retained returning depth or
    /// the same-path numeric wrapper root, never a root replay instruction.
    pub(super) edge: Option<CssColorPathStep>,
}
