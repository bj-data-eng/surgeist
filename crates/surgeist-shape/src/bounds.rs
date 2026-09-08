use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub rect: Rect,
    pub kind: BoundsKind,
}

impl Bounds {
    #[must_use]
    pub const fn new(rect: Rect, kind: BoundsKind) -> Self {
        Self { rect, kind }
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self {
            rect: Rect::empty(),
            kind: BoundsKind::Source,
        }
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.rect.is_empty()
    }

    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self::new(self.rect.union(other.rect), self.kind)
    }

    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        Self::new(self.rect.intersection(other.rect), self.kind)
    }

    #[must_use]
    pub fn rect(self) -> Rect {
        self.rect
    }

    #[must_use]
    pub fn kind(self) -> BoundsKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundsKind {
    Source,
    Fill,
    Stroke,
    Visual,
    Support,
    Transformed,
}
