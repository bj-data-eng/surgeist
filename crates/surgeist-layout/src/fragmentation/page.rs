use crate::{Direction, FlowAxes, WritingMode};

/// The resolved direction in which a document's page spreads progress.
/// This is independent of a fragmented subtree's geometric flow axes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageProgression {
    LeftToRight,
    RightToLeft,
}

impl PageProgression {
    /// Classifies explicitly supplied document principal *used* writing axes.
    /// The caller owns principal-mode resolution, including HTML propagation;
    /// arbitrary descendant or containing-block axes are not that authority.
    #[must_use]
    pub const fn from_principal_used_axes(principal_used_axes: FlowAxes) -> Self {
        match principal_used_axes.writing_mode() {
            WritingMode::HorizontalTb => match principal_used_axes.direction() {
                Direction::Ltr => Self::LeftToRight,
                Direction::Rtl => Self::RightToLeft,
            },
            WritingMode::VerticalRl | WritingMode::SidewaysRl => Self::RightToLeft,
            WritingMode::VerticalLr | WritingMode::SidewaysLr => Self::LeftToRight,
        }
    }

    /// Resolves a logical spread side to its physical page side.
    #[must_use]
    pub const fn physical_side(
        self,
        logical: FragmentationLogicalPageSide,
    ) -> FragmentationPhysicalPageSide {
        match (self, logical) {
            (Self::LeftToRight, FragmentationLogicalPageSide::Recto)
            | (Self::RightToLeft, FragmentationLogicalPageSide::Verso) => {
                FragmentationPhysicalPageSide::Right
            }
            (Self::LeftToRight, FragmentationLogicalPageSide::Verso)
            | (Self::RightToLeft, FragmentationLogicalPageSide::Recto) => {
                FragmentationPhysicalPageSide::Left
            }
        }
    }

    /// Classifies an actual physical page side within this progression.
    /// Neither page index nor rectangle geometry supplies the physical side.
    #[must_use]
    pub const fn classify(self, physical: FragmentationPhysicalPageSide) -> FragmentationPageSide {
        let recto = self.physical_side(FragmentationLogicalPageSide::Recto);
        let logical = if matches!(
            (physical, recto),
            (
                FragmentationPhysicalPageSide::Left,
                FragmentationPhysicalPageSide::Left
            ) | (
                FragmentationPhysicalPageSide::Right,
                FragmentationPhysicalPageSide::Right
            )
        ) {
            FragmentationLogicalPageSide::Recto
        } else {
            FragmentationLogicalPageSide::Verso
        };
        FragmentationPageSide { physical, logical }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationPhysicalPageSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationLogicalPageSide {
    Recto,
    Verso,
}

/// Consistent physical and logical page classification, produced only by
/// [`PageProgression::classify`] from an actual physical page side.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FragmentationPageSide {
    physical: FragmentationPhysicalPageSide,
    logical: FragmentationLogicalPageSide,
}

impl FragmentationPageSide {
    #[must_use]
    pub const fn physical(self) -> FragmentationPhysicalPageSide {
        self.physical
    }

    #[must_use]
    pub const fn logical(self) -> FragmentationLogicalPageSide {
        self.logical
    }
}
