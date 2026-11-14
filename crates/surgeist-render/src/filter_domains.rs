//! Resolved image domains shared by native filter primitives.
use crate::{Error, Rect, Result};

/// Independent source, hard filter and primitive-result domains in layer-local units.
///
/// All origins, extents and derived maxima are finite; extents are nonnegative.
/// Empty domains are valid. Construction retains each rectangle independently:
/// consumers apply the filter region to the input before image-space sampling,
/// and clip the output to both the result and filter regions afterward.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilterDomains {
    source: Rect,
    filter: Rect,
    result: Rect,
}

impl FilterDomains {
    /// Checks finite origins, nonnegative extents and representable maxima.
    ///
    /// Empty domains are valid. Invalid rectangles return
    /// [`crate::ErrorCode::InvalidInput`] with a filter-labelled
    /// [`crate::InvalidValue`] diagnostic.
    pub fn try_new(source: Rect, filter: Rect, result: Rect) -> Result<Self> {
        for (role, rect) in [("source", source), ("filter", filter), ("result", result)] {
            validate_domain(role, rect)?;
        }
        Ok(Self {
            source,
            filter,
            result,
        })
    }

    /// Returns the semantic primary-input domain.
    #[must_use]
    pub const fn source(self) -> Rect {
        self.source
    }

    /// Returns the hard filter region applied before image-space sampling.
    #[must_use]
    pub const fn filter(self) -> Rect {
        self.filter
    }

    /// Returns the independently clipped primitive-result region.
    #[must_use]
    pub const fn result(self) -> Rect {
        self.result
    }
}

fn validate_domain(role: &'static str, rect: Rect) -> Result<()> {
    for (field, value) in [
        ("x", rect.x()),
        ("y", rect.y()),
        ("max x", rect.max().x()),
        ("max y", rect.max().y()),
    ] {
        if !value.is_finite() {
            return Err(Error::invalid_value(
                format!("filter domain {role} {field}"),
                value,
                "must be finite",
            ));
        }
    }
    for (field, value) in [("width", rect.width()), ("height", rect.height())] {
        if !value.is_finite() || value < 0.0 {
            return Err(Error::invalid_value(
                format!("filter domain {role} {field}"),
                value,
                "must be finite and non-negative",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_empty_domains_retain_finite_extreme_origins() {
        let source = Rect::try_new(-4.0, 2.0, 7.0, 5.0).unwrap();
        let filter = Rect::try_new(10.0, -3.0, 0.0, 8.0).unwrap();
        let result = Rect::try_new(f64::MAX, -f64::MAX, 0.0, 0.0).unwrap();
        let domains = FilterDomains::try_new(source, filter, result).unwrap();
        assert_eq!(domains.source(), source);
        assert_eq!(domains.filter(), filter);
        assert_eq!(domains.result(), result);
        assert!(FilterDomains::try_new(Rect::default(), Rect::default(), Rect::default()).is_ok());
    }

    #[test]
    fn every_domain_rejects_invalid_components_and_overflowing_maxima() {
        // Crate-local raw rectangles exercise this invariant owner; public Rect
        // construction already prevents these invalid values from escaping.
        let invalid = [
            Rect::new(f64::NAN, 0.0, 1.0, 1.0),
            Rect::new(0.0, f64::INFINITY, 1.0, 1.0),
            Rect::new(0.0, 0.0, -1.0, 1.0),
            Rect::new(0.0, 0.0, 1.0, -1.0),
            Rect::new(0.0, 0.0, f64::INFINITY, 1.0),
            Rect::new(0.0, 0.0, 1.0, f64::NAN),
            Rect::new(f64::MAX, 0.0, f64::MAX, 1.0),
            Rect::new(0.0, f64::MAX, 1.0, f64::MAX),
        ];
        for (index, role) in ["source", "filter", "result"].into_iter().enumerate() {
            for rect in invalid {
                let mut domains = [Rect::default(); 3];
                domains[index] = rect;
                let error = FilterDomains::try_new(domains[0], domains[1], domains[2]).unwrap_err();
                assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
                assert!(
                    error
                        .invalid_value_diagnostic()
                        .unwrap()
                        .field()
                        .starts_with(&format!("filter domain {role} "))
                );
                assert!(error.convolution_diagnostic().is_none());
            }
        }
    }
}
