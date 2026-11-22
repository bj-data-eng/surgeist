#![forbid(unsafe_code)]

//! GPU-only rendering boundary for Surgeist.
//!
//! The crate owns renderer-facing visual facts, safe WGPU resources, and two
//! observable execution routes. Effect-free scenes use one transaction-owned
//! [`RenderRoute::DirectVello`](renderer::RenderRoute::DirectVello) raster pass. Resolved image alpha masks,
//! supported blend/composition, and bounded backdrop filter lists use
//! [`RenderRoute::GpuGraph`](renderer::RenderRoute::GpuGraph) image/composite passes. Production rendering has no
//! CPU pixel path or CPU fallback; CPU reference algorithms compile only for tests.
//!
//! Effect graphs use high precision by default. [`EffectQualityPolicy::RequireHighPrecision`](renderer::EffectQualityPolicy::RequireHighPrecision)
//! rejects a device that cannot provide it, while
//! [`EffectQualityPolicy::AllowReducedPrecision`](renderer::EffectQualityPolicy::AllowReducedPrecision) permits reduced precision,
//! observably, only when high precision is unavailable. [`Capabilities::CURRENT`](capability::Capabilities::CURRENT)
//! reports semantic capabilities for authored operations; [`Renderer::runtime_capabilities`](renderer::Renderer::runtime_capabilities)
//! reports immutable runtime capabilities for a selected device and surface.
//! Cargo features select host adapters rather than changing those meanings.
//!
//! Renderer and surface GPU operations are asynchronous and failure-atomic. A
//! failed or canceled operation publishes neither partial pixels nor partial
//! statistics, so [`Renderer::stats`](renderer::Renderer::stats) continues to describe the last successful
//! frame. CPU-visible pixels require explicit headless readback through
//! [`Renderer::read_headless`](renderer::Renderer::read_headless); rendering never enters that path implicitly.
//!
//! The native window presentation path is host lifecycle owned.
//! The tracked `render_window_smoke` example exercises the public native window lifecycle
//! and presentation path. The
//! `wasm32-unknown-unknown` leaf boundary is compile-only under `render-web`;
//! real canvas execution requires a browser host and is root integration work.
//! Render owns its optional `render-text` consumer of authentic standalone Text
//! layouts. The root `surgeist` repository owns the facade, workspace wiring,
//! generated API artifacts and browser integration evidence.
//!
//! # Where things live
//!
//! The crate root holds [`Error`], [`ErrorCode`] and [`Result`]. Every other public item lives in
//! one module:
//!
//! | Module | Contents |
//! | --- | --- |
//! | [`accounting`] | Accounting contexts, limits, units and reports |
//! | [`capability`] | Semantic and runtime capability reports |
//! | [`convolution`], [`gaussian`], [`morphology`], [`displacement`], [`lighting`], [`offset`], [`native_shadow`] | Native filter preparation and its typed refusals |
//! | [`diagnostic`] | Typed diagnostics that an [`Error`] carries |
//! | [`effect`] | Inputs shared by the filter-primitive modules |
//! | [`geometry`] | Logical and physical geometry and coordinate spaces |
//! | [`icc`] | Checked in-memory color profiles and calibrated conversions |
//! | [`image`] | Images and their sampling choices |
//! | [`paint`] | Paint sources, colors and gradients |
//! | [`renderer`] | The GPU renderer, its options and frame statistics |
//! | [`scene`] | Scenes, layers, masks and shadows |
//! | [`shape`] | Paths, shapes and strokes |
//! | [`style`] | Style-facing models: background, clip, decoration, filter, image and mask |
//! | [`surface`] | Surfaces, their options and per-frame parameters |
//! | [`text`] | Text runs and glyph-run preparation |
//!
//! With `render-paint` the `paint_gradient` module prepares native gradient fields, and with
//! `render-text` the `text_layout` module consumes placed Text layouts and leader patterns.

pub mod accounting;
pub mod capability;
pub mod convolution;
pub mod diagnostic;
pub mod displacement;
pub mod effect;
pub mod gaussian;
pub mod geometry;
pub mod icc;
pub mod image;
pub mod lighting;
pub mod morphology;
pub mod native_shadow;
pub mod offset;
pub mod paint;
#[cfg(feature = "render-paint")]
pub mod paint_gradient;
pub mod renderer;
pub mod scene;
pub mod shape;
pub mod style;
pub mod surface;
pub mod text;
#[cfg(feature = "render-text")]
pub mod text_layout;

mod backend;
mod color_mapping;
mod color_profile;
mod color_reduction;
mod command;
mod encode;
mod error;
mod filter;
mod filter_domains;
mod filter_model;
mod filter_source;
mod frame;
mod gpu_transaction;
mod layer;
mod number_text;
mod pass;
mod readback;
#[cfg(test)]
mod reference;
mod resource;
mod shader;
mod stats;
mod texture;
mod turbulence;
mod validation;
mod vello_engine;

pub use error::{Error, ErrorCode, Result};

#[cfg(test)]
mod tests;
