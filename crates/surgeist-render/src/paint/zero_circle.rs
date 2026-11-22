//! Owned native stop positions after Paint's infinitesimal stop fixup.

use crate::{
    Result,
    accounting::{Account, Hold, RenderWork},
    geometry::Point,
    paint::Color,
};

/// The position `length + fraction * radius` as radius tends to zero from above.
/// Declaration order defines the ordering for finite coefficients.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) struct ZeroCirclePosition {
    pub(crate) length: f64,
    pub(crate) fraction: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ZeroCircleStop {
    pub(crate) position: ZeroCirclePosition,
    pub(crate) color: Color,
    pub(crate) hint_after: Option<ZeroCirclePosition>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ZeroCircleGradient {
    center: Point,
    stops: Vec<ZeroCircleStop>,
    repeating: bool,
}

impl ZeroCircleGradient {
    // Called by the Paint adapter with the authenticated fixed-up stop sequence.
    pub(in crate::paint) fn from_paint(
        center: Point,
        stops: Vec<ZeroCircleStop>,
        repeating: bool,
    ) -> Self {
        Self {
            center,
            stops,
            repeating,
        }
    }

    pub(crate) const fn center(&self) -> Point {
        self.center
    }

    pub(crate) fn stops(&self) -> &[ZeroCircleStop] {
        &self.stops
    }

    pub(crate) const fn is_repeating(&self) -> bool {
        self.repeating
    }

    pub(crate) fn try_clone_in(&self, account: &Account) -> Result<(Self, Hold)> {
        let mut stops = account.retained(self.stops.len())?;
        for stop in &self.stops {
            account.work(RenderWork::GradientStopVisits, 1)?;
            stops.push(*stop)?;
        }
        let (stops, hold) = stops.into_parts()?;
        Ok((Self::from_paint(self.center, stops, self.repeating), hold))
    }
}
