//! Boa's implementation of JavaScript's `fetch` function.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [WHATWG `fetch` specification][spec]
//!
//! [spec]: https://fetch.spec.whatwg.org/
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/fetch

use crate::fetch::headers::JsHeaders;
use crate::fetch::headers_iterator::{HeadersIterator, IterationKind};
use crate::fetch::request::{JsRequest, RequestInit};
use crate::fetch::response::JsResponse;
use either::Either;
use http::{HeaderName, HeaderValue, Request as HttpRequest, Request};
use std::rc::Rc;
use surgeist_js::class::Class;
use surgeist_js::gc::heap::Trace as HeapTrace;
use surgeist_js::job::ContextTask;
use surgeist_js::object::FunctionObjectBuilder;
use surgeist_js::property::PropertyDescriptor;
use surgeist_js::realm::Realm;
use surgeist_js::{
    Context, ContextAccess, JsData, JsError, JsObject, JsResult, JsString, JsSymbol, JsValue,
    NativeObject, boa_module, js_error, js_string, native_function::NativeFunction,
};

pub mod headers;
pub mod headers_iterator;
pub mod request;
pub mod response;
pub mod tests;

mod fetchers;

#[doc(inline)]
pub use fetchers::*;

/// A trait for backend implementation of an HTTP fetcher.
pub trait Fetcher: NativeObject {
    /// Resolve a string to a URL. This is used when a string (e.g., the first argument to
    /// `fetch()`) is passed, and we need resolution. Some cases require resolution of
    /// a relative path, for example (to the "page" base URL).
    /// By default, this will return the `URI` as is.
    ///
    /// # Errors
    /// This function should return an error if the URL cannot be handled by the [`Fetcher`].
    fn resolve_uri(&self, uri: String, _context: ContextAccess<'_>) -> JsResult<String> {
        Ok(uri)
    }

    /// Creates owned Fetch work for a [`request::JsRequest`].
    ///
    /// The task returns a [`response::JsResponse`] or a [`JsError`]. It owns its
    /// IO and continuation state; engine access is temporary during each poll.
    fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        signal: Option<JsObject>,
    ) -> ContextTask<JsResponse>;
}

/// A reference counted pointer to a `Fetcher` implementation. This is so we can
/// add this to the context, but we need to be able to keep an `Rc<>` structure
/// to make API calls.
#[derive(Debug, JsData, HeapTrace)]
struct FetcherRc<T: Fetcher>(pub Rc<T>);

impl<T: Fetcher> Clone for FetcherRc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Get a Fetcher instance from the context.
fn get_fetcher<T: Fetcher>(context: ContextAccess<'_>) -> JsResult<Rc<T>> {
    // Try fetching from the context first, then the current realm. Else fail.
    let Some(fetcher) = context.get_data::<FetcherRc<T>>().cloned().or_else(|| {
        context
            .realm()
            .host_defined()
            .get::<FetcherRc<T>>()
            .cloned()
    }) else {
        return Err(
            js_error!(Error: "Implementation of fetch requires a fetcher registered in the context"),
        );
    };

    Ok(fetcher.0.clone())
}

fn check_abort(signal: Option<&JsObject>, mut context: ContextAccess<'_>) -> JsResult<()> {
    if let Some(signal_obj) = signal
        && let Some(signal_ref) = signal_obj.downcast_ref::<crate::abort::JsAbortSignal>()
        && signal_ref.is_aborted()
    {
        return Err(JsError::from_opaque(
            signal_ref.abort_reason(context.reborrow())?,
        ));
    }
    Ok(())
}

/// The `fetch` function internals.
fn fetch_inner<T: Fetcher>(
    resource: Either<JsString, JsObject>,
    options: Option<RequestInit>,
    mut context: ContextAccess<'_>,
) -> JsResult<ContextTask<JsValue>> {
    let fetcher = get_fetcher::<T>(context.reborrow())?;

    let (options, signal) = match options {
        Some(mut opts) => {
            let sig = opts.take_signal();
            (Some(opts), sig)
        }
        None => (None, None),
    };

    check_abort(signal.as_ref(), context.reborrow())?;

    // The resource parsing is complicated, so we parse it in Rust here (instead of relying on
    // `TryFromJs` and friends).
    let mut signal = signal;

    let request: Request<Vec<u8>> = match resource {
        Either::Left(url) => {
            let url = url.to_std_string().map_err(JsError::from_rust)?;
            let url = fetcher.resolve_uri(url, context.reborrow())?;

            let r = HttpRequest::get(url).body(Vec::new());
            r.map_err(JsError::from_rust)?
        }
        Either::Right(request) => {
            // This can be a [`JsRequest`] object.
            let Ok(request) = request.downcast::<JsRequest>() else {
                return Err(js_error!(TypeError: "Resource must be a URL or Request object"));
            };
            let Ok(request_ref) = request.try_borrow() else {
                return Err(js_error!(TypeError: "Request object is already in use"));
            };

            if signal.is_none() {
                signal = request_ref.data().signal()?;
            }
            request_ref.data().inner().clone()
        }
    };

    check_abort(signal.as_ref(), context.reborrow())?;

    let mut request = if let Some(options) = options {
        options.into_request_builder(Some(request))?
    } else {
        request
    };

    // Add the `Accept-Language` which should be automatically included, unless specified.
    if !request.headers().contains_key(
        "accept-language"
            .parse::<HeaderName>()
            .map_err(JsError::from_rust)?,
    ) {
        let lang = HeaderValue::from_static("en-US");
        request.headers_mut().append("Accept-Language", lang);
    }

    Ok(fetcher
        .fetch(JsRequest::from(request), signal.clone())
        .then(move |response, mut access| {
            check_abort(signal.as_ref(), access.reborrow())?;
            Class::from_data(response, access.reborrow()).map(Into::into)
        }))
}

/// JavaScript module containing the fetch types and functions.
#[boa_module]
pub mod js_module {
    use crate::fetch::request::RequestInit;
    use crate::fetch::{Fetcher, fetch_inner};
    use either::Either;
    use surgeist_js::object::builtins::JsPromise;
    use surgeist_js::{ContextAccess, JsObject, JsResult, JsString};

    type JsHeaders = super::JsHeaders;
    type JsRequest = super::JsRequest;
    type JsResponse = super::JsResponse;
    type HeadersIterator = super::headers_iterator::HeadersIterator;

    /// The `fetch` function.
    ///
    /// # Errors
    /// If the fetcher is not registered in the context, an error is returned.
    /// This function will also return any error that the fetcher returns, or
    /// any conversion to/from JavaScript types.
    pub fn fetch<T: Fetcher>(
        resource: Either<JsString, JsObject>,
        options: Option<RequestInit>,
        mut context: ContextAccess<'_>,
    ) -> JsResult<JsPromise> {
        JsPromise::from_task(
            move |context| fetch_inner::<T>(resource, options, context),
            context.reborrow(),
        )
    }
}

fn headers_symbol_iterator(
    this: &JsValue,
    _: &[JsValue],
    mut context: ContextAccess<'_>,
) -> JsResult<JsValue> {
    let this_object = this.as_object().ok_or_else(
        || js_error!(TypeError: "`Headers.prototype[Symbol.iterator]` requires a `Headers` object"),
    )?;

    let Ok(headers) = this_object.clone().downcast::<JsHeaders>() else {
        return Err(
            js_error!(TypeError: "`Headers.prototype[Symbol.iterator]` requires a `Headers` object"),
        );
    };

    HeadersIterator::create_headers_iterator(
        headers,
        IterationKind::KeyAndValue,
        context.reborrow(),
    )
}

/// Register the `fetch` function in the realm, as well as ALL supporting classes.
/// Pass `None` as the realm to register globally.
///
/// # Errors
/// If any of the classes fail to register, an error is returned.
pub fn register<F: Fetcher>(
    fetcher: F,
    realm: Option<Realm>,
    context: &mut Context,
) -> JsResult<()> {
    if let Some(ref realm) = realm {
        realm.insert_host_defined(FetcherRc(Rc::new(fetcher)))?;
    } else {
        context.insert_data(FetcherRc(Rc::new(fetcher)));
    }
    js_module::boa_register::<F>(realm.clone(), context.access())?;

    // TODO(#4688): Replace this manual `[Symbol.iterator]` wiring once `#[boa(class)]`
    // supports symbol-named methods.
    let headers_proto = match realm {
        Some(realm) => realm.get_class::<JsHeaders>(),
        None => context.get_global_class::<JsHeaders>(),
    }
    .ok_or_else(|| js_error!(Error: "Headers class should be registered"))?
    .prototype();

    let iterator = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(headers_symbol_iterator),
    )
    .name(js_string!("[Symbol.iterator]"))
    .length(0)
    .constructor(false)
    .build()?;

    headers_proto.define_property_or_throw(
        JsSymbol::iterator(),
        PropertyDescriptor::builder()
            .value(iterator)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context.access(),
    )?;

    Ok(())
}
