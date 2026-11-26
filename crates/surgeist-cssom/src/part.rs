//! A payload part shared by the records that hold it, with the account of the
//! sources it retains.
//!
//! The heavy payloads of a record (the authored tree of a sheet, the data of a
//! rule) are shared between the published state and the staged copy of it. A
//! [`Part`] keeps the measurement of what its value retains next to the value,
//! so the account is made once for as long as the part is shared and unchanged:
//!
//! - reading the part reads the value; the measurement is made on first use;
//! - changing the part ([`Part::make_mut`]) clears its measurement, whether the
//!   value is copied or changed in place;
//! - a clone of the part shares the value and its measurement; a copy of the
//!   value starts unmeasured;
//! - a measurement is kept only when it succeeded within the bounds it was made
//!   under, so it does not depend on the limits of the store that made it. A
//!   refused measurement is made again.
//!
//! A part is the size of the pointer it replaces, so no record grows. Its
//! allocation holds the measurement beside the value: about 48 bytes for each
//! part, and 40 bytes and an `Arc` clone for each snapshot the part retains once it is
//! measured. A sheet holds 2 parts, a rule 3, a block 2 to 4 and a feature map 1.
use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, OnceLock};
use surgeist_css::source::{
    CssSourceBounds, CssSourceMeasurement, CssSourceRetention, CssSourceRetentionError,
};

struct Inner<T> {
    measurement: OnceLock<CssSourceMeasurement>,
    value: T,
}

impl<T: Clone> Clone for Inner<T> {
    /// A copy of the value, which has not been measured.
    fn clone(&self) -> Self {
        Self {
            measurement: OnceLock::new(),
            value: self.value.clone(),
        }
    }
}

/// A shared payload and its measurement; see the [module](self).
pub(crate) struct Part<T>(Arc<Inner<T>>);

impl<T> Part<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Arc::new(Inner {
            measurement: OnceLock::new(),
            value,
        }))
    }

    /// Whether both parts are the same shared value.
    #[cfg(test)]
    pub(crate) fn ptr_eq(this: &Self, other: &Self) -> bool {
        Arc::ptr_eq(&this.0, &other.0)
    }

    /// The measurement of the retained sources of the value, which `visit` reports
    /// to a collector, made under `bounds` when it has not been made yet.
    ///
    /// A part is always visited the same way, so the measurement belongs to it
    /// whoever asks.
    ///
    /// # Errors
    ///
    /// The refusal of the measurement. A refused measurement is not kept.
    pub(crate) fn measurement(
        &self,
        bounds: CssSourceBounds,
        visit: impl FnOnce(&mut CssSourceRetention<'_>, &T) -> Result<(), CssSourceRetentionError>,
    ) -> Result<&CssSourceMeasurement, CssSourceRetentionError> {
        if let Some(measurement) = self.0.measurement.get() {
            return Ok(measurement);
        }
        #[cfg(test)]
        crate::state_limits::probe::part_measured();
        let measured =
            CssSourceMeasurement::of(bounds, 0, |sources| visit(sources, &self.0.value))?;
        Ok(self.0.measurement.get_or_init(|| measured))
    }

    /// Whether the measurement of the value has been made and kept.
    #[cfg(test)]
    pub(crate) fn is_measured(&self) -> bool {
        self.0.measurement.get().is_some()
    }
}

impl<T: Clone> Part<T> {
    /// The value, to change. It is copied when another part shares it, and its
    /// measurement is cleared either way.
    pub(crate) fn make_mut(this: &mut Self) -> &mut T {
        let inner = Arc::make_mut(&mut this.0);
        inner.measurement.take();
        &mut inner.value
    }
}

impl<T> Clone for Part<T> {
    /// Another handle to the same value and measurement.
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for Part<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0.value
    }
}

impl<T> AsRef<T> for Part<T> {
    fn as_ref(&self) -> &T {
        &self.0.value
    }
}

impl<T> From<T> for Part<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: PartialEq> PartialEq for Part<T> {
    /// Equal values, whether or not the parts share them.
    fn eq(&self, other: &Self) -> bool {
        self.0.value == other.0.value
    }
}

impl<T: fmt::Debug> fmt::Debug for Part<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.value.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_limits::probe;
    use surgeist_css::rule::CssSheet;

    fn sheet(text: &str) -> Part<CssSheet> {
        Part::new(
            surgeist_css::CssParser::default()
                .parse_sheet(text)
                .into_parts()
                .0,
        )
    }

    const BOUNDS: CssSourceBounds = CssSourceBounds::new(usize::MAX, usize::MAX, 64);

    fn measure(part: &Part<CssSheet>) -> usize {
        part.measurement(BOUNDS, |sources, value| sources.visit(value))
            .unwrap()
            .steps()
    }

    #[test]
    fn changing_a_part_clears_its_measurement() {
        let mut part = sheet(".a { color: red }");
        probe::reset();
        let before = measure(&part);
        assert!(part.is_measured());
        // The part is not shared, so the value changes in place.
        *Part::make_mut(&mut part) = surgeist_css::CssParser::default()
            .parse_sheet(".a { color: red } .b { margin: 0 } .c { color: blue }")
            .into_parts()
            .0;
        assert!(!part.is_measured());
        assert!(measure(&part) > before);
        assert_eq!(probe::parts(), 2);
    }

    #[test]
    fn a_copy_of_a_shared_value_starts_unmeasured_and_leaves_the_original_measured() {
        let original = sheet(".a { color: red }");
        probe::reset();
        measure(&original);
        let mut copy = original.clone();
        assert!(Part::ptr_eq(&original, &copy));
        // A clone shares the measurement.
        assert!(copy.is_measured());
        // Changing one copies the value, and the copy has not been measured.
        let _ = Part::make_mut(&mut copy);
        assert!(!Part::ptr_eq(&original, &copy));
        assert!(!copy.is_measured());
        assert!(original.is_measured());
        measure(&copy);
        measure(&original);
        assert_eq!(probe::parts(), 2);
    }

    #[test]
    fn a_part_shared_by_two_holders_is_measured_once() {
        let base = sheet(".a { color: red } .b { margin: 1px }");
        let staged = base.clone();
        probe::reset();
        let (first, second) = (measure(&base), measure(&staged));
        assert_eq!(first, second);
        assert_eq!(probe::parts(), 1);
    }

    #[test]
    fn a_refused_measurement_is_not_kept() {
        let part = sheet(".a { color: red } .b { margin: 1px }");
        probe::reset();
        let tight = CssSourceBounds::new(usize::MAX, 1, 64);
        assert!(
            part.measurement(tight, |sources, value| sources.visit(value))
                .is_err()
        );
        assert!(!part.is_measured());
        // Under larger bounds it is measured and kept.
        assert!(measure(&part) > 1);
        assert!(part.is_measured());
        // A measurement kept under one set of bounds answers under any other.
        assert!(
            part.measurement(tight, |sources, value| sources.visit(value))
                .is_ok()
        );
        assert_eq!(probe::parts(), 2);
    }

    #[test]
    fn a_part_costs_a_handle_and_the_memo_next_to_its_value() {
        // The record holds the same eight bytes as it did for a shared pointer.
        assert_eq!(size_of::<Part<CssSheet>>(), size_of::<Arc<CssSheet>>());
        // The memo beside the value is the 48 bytes the module states.
        assert!(size_of::<OnceLock<CssSourceMeasurement>>() <= 48);
    }
}
