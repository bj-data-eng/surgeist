//! Inputs shared by the filter-primitive modules.
//!
//! [`FilterDomains`] holds the source, filter and primitive-result domains. The preparation functions
//! [`prepare_convolution`](crate::convolution::prepare_convolution) and
//! [`prepare_offset`](crate::offset::prepare_offset) accept it, and so do the finite constructors of
//! the Gaussian, morphology, displacement, lighting and native shadow sources, for example
//! [`GaussianSource::finite`](crate::gaussian::GaussianSource::finite). [`FilterColorSpace`] is the
//! numeric working space that the preparation functions of the convolution, Gaussian, morphology,
//! displacement, lighting and native shadow modules accept. [`FilterEdgeMode`] is the neighborhood addressing
//! against the source domain; among others
//! [`GaussianSource::finite_with_edge`](crate::gaussian::GaussianSource::finite_with_edge) and
//! [`ConvolutionInput::with_edge_mode`](crate::convolution::ConvolutionInput::with_edge_mode) accept
//! it.

pub use crate::filter_domains::FilterDomains;
pub use crate::filter_model::{FilterColorSpace, FilterEdgeMode};
