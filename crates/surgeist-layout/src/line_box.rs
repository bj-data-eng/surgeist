//! Invocation-local accepted line-box direction, independent of body bidi and tabs.

use crate::{Direction, LineBoxDirectionPolicy};
use surgeist_text::{LineBoxDirection, ParagraphBaseDirection};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LineBoxContext {
    policy: LineBoxDirectionPolicy,
    containing: Direction,
    previous: Option<LineBoxDirection>,
}

impl LineBoxContext {
    pub const fn new(policy: LineBoxDirectionPolicy, containing: Direction) -> Self {
        Self {
            policy,
            containing,
            previous: None,
        }
    }

    pub fn resolve(self, strong: Option<ParagraphBaseDirection>) -> LineBoxDirection {
        if self.policy == LineBoxDirectionPolicy::Plaintext {
            if let Some(strong) = strong {
                return match strong {
                    ParagraphBaseDirection::LeftToRight => LineBoxDirection::LeftToRight,
                    ParagraphBaseDirection::RightToLeft => LineBoxDirection::RightToLeft,
                };
            }
            if let Some(previous) = self.previous {
                return previous;
            }
        }
        match self.containing {
            Direction::Ltr => LineBoxDirection::LeftToRight,
            Direction::Rtl => LineBoxDirection::RightToLeft,
        }
    }

    pub fn commit(&mut self, direction: LineBoxDirection) {
        self.previous = Some(direction);
    }

    pub fn weak_direction(self) -> Direction {
        match self.resolve(None) {
            LineBoxDirection::LeftToRight => Direction::Ltr,
            LineBoxDirection::RightToLeft => Direction::Rtl,
        }
    }
}
