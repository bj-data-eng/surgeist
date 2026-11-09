//! Module containing various implementations of the [`Fetcher`] trait.

use crate::fetch::Fetcher;
use crate::fetch::request::JsRequest;
use crate::fetch::response::JsResponse;
use std::future;
use std::rc::Rc;
use surgeist_js::gc::heap::Trace as HeapTrace;
use surgeist_js::job::ContextTask;
use surgeist_js::{JsData, JsObject, js_error};

/// Implementation of `Fetcher` which will always reject any fetch.
#[derive(Clone, Debug, JsData, HeapTrace)]
pub struct ErrorFetcher;

impl Fetcher for ErrorFetcher {
    fn fetch(
        self: Rc<Self>,
        _request: JsRequest,
        _signal: Option<JsObject>,
    ) -> ContextTask<JsResponse> {
        ContextTask::from_future(future::ready(Err(
            js_error!(ReferenceError: "ErrorFetcher used in fetch API."),
        )))
    }
}

/// Implementation of `Fetcher` that uses the blocking `reqwest` library as the backend.
#[cfg(feature = "reqwest-blocking")]
#[derive(Default, Debug, Clone, JsData, HeapTrace)]
pub struct BlockingReqwestFetcher {
    #[heap_trace(skip)]
    client: reqwest::blocking::Client,
}

#[cfg(feature = "reqwest-blocking")]
impl Fetcher for BlockingReqwestFetcher {
    fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        signal: Option<JsObject>,
    ) -> ContextTask<JsResponse> {
        use surgeist_js::{JsError, JsString};

        ContextTask::from_future(async move {
            if let Some(ref sig) = signal
                && let Some(sig_ref) = sig.downcast_ref::<crate::abort::JsAbortSignal>()?
                && sig_ref.is_aborted()
            {
                return Err(JsError::from_opaque(
                    surgeist_js::js_string!("AbortError").into(),
                )?);
            }

            let request = request.into_inner();
            let url = request.uri().to_string();
            let req = self
                .client
                .request(request.method().clone(), &url)
                .headers(request.headers().clone());

            let req = req
                .body(request.body().clone())
                .build()
                .map_err(JsError::from_rust)?;

            let resp = self.client.execute(req).map_err(JsError::from_rust)?;

            if let Some(ref sig) = signal
                && let Some(sig_ref) = sig.downcast_ref::<crate::abort::JsAbortSignal>()?
                && sig_ref.is_aborted()
            {
                return Err(JsError::from_opaque(
                    surgeist_js::js_string!("AbortError").into(),
                )?);
            }

            let status = resp.status();
            let headers = resp.headers().clone();
            let bytes = resp.bytes().map_err(JsError::from_rust)?;
            let mut builder = http::Response::builder().status(status.as_u16());

            for k in headers.keys() {
                for v in headers.get_all(k) {
                    builder = builder.header(k.as_str(), v);
                }
            }

            builder
                .body(bytes.to_vec())
                .map_err(JsError::from_rust)
                .map(|request| JsResponse::basic(JsString::from(url), request))
        })
    }
}
