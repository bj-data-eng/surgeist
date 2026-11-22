//! Native filter-image specialization and executed generic-image delegation.
//!
//! Source tokens certify caller-verified immutable used source/context facts.
//! Callers retain that mapping and refresh tokens when source, concrete size or
//! other non-filter context changes. No URL equality, image decoding, source
//! resolution, pixel execution or generic image algorithm is invented here.

use super::FilterList;
use crate::interpolation::InterpolationProgress;
use std::sync::Arc;

/// Identity for one verified used image and all of its non-filter context.
/// Clone only for semantically identical facts; a new token is always distinct.
#[derive(Debug, Clone)]
pub struct FilterImageSource(Arc<()>);
impl FilterImageSource {
    pub fn new() -> Self {
        Self(Arc::new(()))
    }
}
impl Default for FilterImageSource {
    fn default() -> Self {
        Self::new()
    }
}
impl PartialEq for FilterImageSource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for FilterImageSource {}

#[derive(Debug, Clone, PartialEq)]
struct Filtered {
    source: FilterImageSource,
    filters: FilterList,
}

/// A complete original caller image, optionally with resolved filter() facts.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterImage<I> {
    original: I,
    filtered: Option<Filtered>,
}

/// A specialized filter-image sample or the executed generic interpolation result.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterImageSample<S> {
    Filtered {
        source: FilterImageSource,
        filters: FilterList,
    },
    Generic(S),
}

impl<I> FilterImage<I> {
    pub fn filtered(original: I, source: FilterImageSource, filters: FilterList) -> Self {
        Self {
            original,
            filtered: Some(Filtered { source, filters }),
        }
    }
    pub fn generic(original: I) -> Self {
        Self {
            original,
            filtered: None,
        }
    }
    pub const fn original(&self) -> &I {
        &self.original
    }
    /// Executes generic interpolation exactly once when the two filter() forms
    /// do not differ solely in interpolable used filter lists. The delegate gets
    /// both complete originals and unchanged progress, including extrapolation;
    /// its typed result/error is preserved without substituting a discrete swap.
    ///
    /// A specialized result retains image-domain execution: preserve source
    /// dimensions/origin, and use duplicate edge handling for image blur.
    pub fn sample<S, E>(
        &self,
        other: &Self,
        progress: InterpolationProgress,
        generic: impl FnOnce(&I, &I, InterpolationProgress) -> Result<S, E>,
    ) -> Result<FilterImageSample<S>, E> {
        if let (Some(a), Some(b)) = (&self.filtered, &other.filtered)
            && a.source == b.source
            && let Ok(Some(filters)) = a.filters.try_interpolate(&b.filters, progress)
        {
            return Ok(FilterImageSample::Filtered {
                source: a.source.clone(),
                filters,
            });
        }
        generic(&self.original, &other.original, progress).map(FilterImageSample::Generic)
    }
}
