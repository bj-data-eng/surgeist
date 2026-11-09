//! Test types and methods to help with testing the Fetch API.

use crate::fetch::request::JsRequest;
use crate::fetch::response::JsResponse;
use http::{Request, Response, Uri};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use surgeist_js::gc::heap::Trace as HeapTrace;
use surgeist_js::job::ContextTask;
use surgeist_js::{JsData, JsString, js_error};

#[cfg(test)]
mod e2e;
#[cfg(test)]
mod headers;
#[cfg(test)]
mod request;
#[cfg(test)]
mod response;

/// A [`crate::fetch::Fetcher`] implementation for tests. Maps a URL to a response,
/// and record requests received for later use.
///
/// The actual safety of this implementation is not guaranteed, as it
/// is only intended for testing purposes.
#[derive(Default, Debug, JsData, HeapTrace)]
pub struct TestFetcher {
    #[heap_trace(skip)]
    requests_received: RefCell<Vec<Request<Vec<u8>>>>,
    #[heap_trace(skip)]
    request_mapper: HashMap<Uri, Response<Vec<u8>>>,
}

impl TestFetcher {
    /// Add a response mapping for a URL.
    pub fn add_response(&mut self, url: Uri, response: Response<Vec<u8>>) {
        self.request_mapper.insert(url, response);
    }
}

impl crate::fetch::Fetcher for TestFetcher {
    fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        _signal: Option<surgeist_js::JsObject>,
    ) -> ContextTask<JsResponse> {
        ContextTask::from_future(async move {
            let request = request.into_inner();
            self.requests_received.borrow_mut().push(request.clone());
            let url = request.uri();
            match self
                .request_mapper
                .get(url)
                .cloned()
                .map(|response| JsResponse::basic(JsString::from(url.to_string()), response))
            {
                Some(response) => Ok(response),
                None => Err(js_error!("No response found for URL")?),
            }
        })
    }
}
