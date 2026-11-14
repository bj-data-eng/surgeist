#[cfg(not(target_arch = "wasm32"))]
mod convolution;
#[cfg(not(target_arch = "wasm32"))]
mod convolution_accounting;
#[cfg(not(target_arch = "wasm32"))]
mod convolution_empty_domain;
#[cfg(not(target_arch = "wasm32"))]
mod convolution_spatial;
#[cfg(not(target_arch = "wasm32"))]
mod convolution_transactions;
mod frame;
mod gpu;
mod image_sampling;
mod image_sampling_graph;
mod image_sampling_mask;
mod model;
mod nonseparable;
#[cfg(not(target_arch = "wasm32"))]
mod offset;
#[cfg(not(target_arch = "wasm32"))]
mod offset_accounting;
#[cfg(not(target_arch = "wasm32"))]
mod offset_transactions;
mod outline;
mod platform;
mod porter_duff;
mod repeating_gradients;
mod separable;
mod style;
mod support;
mod surface;
#[cfg(feature = "render-text")]
mod text_layout_adapter;
mod vello;

trait UnwrapOrPanicForTest<T> {
    #[track_caller]
    fn unwrap_or_panic_for_test(self, message: &str) -> T;
}

impl<T> UnwrapOrPanicForTest<T> for Option<T> {
    #[track_caller]
    fn unwrap_or_panic_for_test(self, message: &str) -> T {
        match self {
            Some(value) => value,
            None => panic!("{message}"),
        }
    }
}

impl<T, E> UnwrapOrPanicForTest<T> for std::result::Result<T, E>
where
    E: std::fmt::Debug,
{
    #[track_caller]
    fn unwrap_or_panic_for_test(self, message: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => panic!("{message}: {error:?}"),
        }
    }
}
