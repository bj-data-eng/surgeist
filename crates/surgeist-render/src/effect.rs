//! Inputs shared by the filter-primitive modules.
//!
//! [`FilterDomains`] holds the source, filter and primitive-result domains. The preparation functions
//! [`prepare_convolution`](crate::convolution::prepare_convolution) and
//! [`prepare_offset`](crate::offset::prepare_offset) accept it, and so do the finite constructors of
//! [`GaussianSource`](crate::gaussian::GaussianSource). [`FilterColorSpace`] is the numeric working
//! space that the preparation functions of the convolution, Gaussian, morphology, displacement and
//! native shadow modules accept. [`FilterEdgeMode`] is accepted by
//! [`GaussianSource::finite_with_edge`](crate::gaussian::GaussianSource::finite_with_edge).

pub use crate::filter_domains::FilterDomains;
pub use crate::filter_model::{FilterColorSpace, FilterEdgeMode};
