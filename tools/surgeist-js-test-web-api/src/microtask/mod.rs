//! Microtask-related functions and types.
use surgeist_js::realm::Realm;
use surgeist_js::{Context, JsResult, boa_module};

#[cfg(test)]
mod tests;

/// JavaScript module containing the `queueMicrotask` function.
#[boa_module]
pub mod js_module {
    use surgeist_js::job::{Job, PromiseJob};
    use surgeist_js::object::builtins::JsFunction;
    use surgeist_js::{ContextAccess, JsResult, JsValue};

    /// The [`queueMicrotask()`][mdn] method of the `Window` interface queues a
    /// microtask to be executed at a safe time prior to control returning to
    /// the browser's event loop.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Window/queueMicrotask
    ///
    /// # Errors
    /// Returns the engine error if microtask publication fails.
    pub fn queue_microtask(callback: JsFunction, mut context: ContextAccess<'_>) -> JsResult<()> {
        context
            .enqueue_job(Job::from(PromiseJob::new(move |context| {
                callback.call(&JsValue::undefined(), &[], context)
            })))
            .map_err(surgeist_js::job::JobPublicationError::into_error)?
            .into_result()
    }
}

/// Register the `queueMicrotask` function to the realm or context.
///
/// # Errors
/// Returns an error if the microtask extension cannot be registered.
pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
    js_module::boa_register(realm, context.access())
}
