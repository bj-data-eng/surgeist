use std::convert::Infallible;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DomError,
    hosted::DomHostedLocalHtmlEdits,
    identity::{DomStamp, NodeId},
};
use surgeist_html_parser::{
    HtmlParseLimits, HtmlParseStop, HtmlSource,
    live::{HtmlLiveParseError, HtmlLiveParseProgress, HtmlLiveParser, HtmlLiveStopParsing},
};

use super::{DomHostedRuntimeError, DomHostedTopLevel, HostedRuntime};
use crate::internal::dom_event::DomEventDom;

#[cfg(test)]
mod tests;

/// The original local parser bound to a native hosted Document.
pub struct DomHostedHtmlParser<'source> {
    top: DomHostedTopLevel,
    parser: HtmlLiveParser<'source, NodeId>,
    edits: Option<DomHostedLocalHtmlEdits>,
    first_refusal: Option<DomHostedHtmlError>,
    latest_refusal: Option<DomHostedHtmlError>,
    checkpoint: DomStamp,
    load_profile: bool,
    progress: Progress,
}

#[derive(Clone, Copy)]
enum Progress {
    Input,
    Eof,
    Transferred,
}

impl fmt::Debug for DomHostedHtmlParser<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomHostedHtmlParser")
            .field("top", &self.top)
            .finish_non_exhaustive()
    }
}

/// Refusal at the hosted identity, parser admission or construction boundary.
#[derive(Debug)]
#[non_exhaustive]
pub enum DomHostedHtmlError {
    /// The native target or its lifecycle authority refused the operation.
    Hosted(DomHostedRuntimeError),
    /// Admission or transfer of the original parser owner failed.
    Start(HtmlLiveParseError<Infallible>),
    /// Parsing or canonical application of a construction step failed.
    Parse(HtmlLiveParseError<DomError>),
    /// The original parser was already admitted for this Document.
    ParserAlreadyClaimed,
    /// Initialization requires the original empty hosted Document.
    NonEmptyDocument,
    /// An unqualified publication prevents an empty load-delay determination.
    LoadProfileChanged,
}

impl From<DomHostedRuntimeError> for DomHostedHtmlError {
    fn from(error: DomHostedRuntimeError) -> Self {
        Self::Hosted(error)
    }
}

impl HostedRuntime {
    /// Admit the original parser for one empty local HTML Document. A refused
    /// admission leaves the Document available for the same operation to retry.
    pub fn begin_local_html_parser<'source>(
        &self,
        dom: &mut DomEventDom<'_>,
        top: DomHostedTopLevel,
        html: &'source HtmlSource,
        limits: HtmlParseLimits,
        cancel: &AtomicBool,
    ) -> Result<DomHostedHtmlParser<'source>, DomHostedHtmlError> {
        let edits = self.with_current_top_level(dom, top, cancel, |source, dom| {
            source
                .begin_local_html_edits(dom.raw(), top.document(), top.global(), cancel)
                .map_err(DomHostedRuntimeError::Dom)
        })?;

        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        for host_index in 0..state.hosts.len() {
            state.step(cancel)?;
            if state.hosts[host_index].id != top.host() {
                continue;
            }
            for top_index in 0..state.hosts[host_index].top_levels.len() {
                state.step(cancel)?;
                let record = &mut state.hosts[host_index].top_levels[top_index];
                let Some(current) = &mut record.top else {
                    continue;
                };
                if current.id != top.navigable() {
                    continue;
                }
                let checkpoint = current
                    .local_parser_checkpoint
                    .ok_or(DomHostedHtmlError::ParserAlreadyClaimed)?;
                let snapshot = dom.snapshot();
                let document = snapshot
                    .node(top.document().node())
                    .map_err(surgeist_dom::hosted::DomHostedError::from)
                    .map_err(DomHostedRuntimeError::Dom)?;
                if !document.children().is_empty() {
                    return Err(DomHostedHtmlError::NonEmptyDocument);
                }
                if snapshot.stamp() != checkpoint {
                    return Err(DomHostedHtmlError::LoadProfileChanged);
                }
                let parser = HtmlLiveParser::new(html, top.document().node(), limits)
                    .map_err(DomHostedHtmlError::Start)?;
                current.local_parser_checkpoint = None;
                return Ok(DomHostedHtmlParser {
                    top,
                    parser,
                    edits: Some(edits),
                    first_refusal: None,
                    latest_refusal: None,
                    checkpoint,
                    load_profile: true,
                    progress: Progress::Input,
                });
            }
        }
        Err(DomHostedRuntimeError::TargetNotFound.into())
    }
}

impl DomHostedHtmlParser<'_> {
    /// The exact top-level identities selected at parser admission.
    pub fn top_level(&self) -> DomHostedTopLevel {
        self.top
    }

    /// The first actual refusal survives later attempts to continue.
    pub fn first_refusal(&self) -> Option<&DomHostedHtmlError> {
        self.first_refusal.as_ref()
    }

    fn retain_refusal(&mut self, error: DomHostedHtmlError) -> &DomHostedHtmlError {
        if self.first_refusal.is_none() {
            self.first_refusal.insert(error)
        } else {
            self.latest_refusal.insert(error)
        }
    }

    /// Advance a bounded number of tokens and release the DOM borrow before
    /// returning, so the hosted Document can be observed or edited at the pause.
    pub fn advance(
        &mut self,
        hosted: &HostedRuntime,
        dom: &mut DomEventDom<'_>,
        token_limit: NonZeroUsize,
        cancel: &AtomicBool,
    ) -> Result<HtmlLiveParseProgress, &DomHostedHtmlError> {
        let document = self.top.document();
        let global = self.top.global();
        let result = hosted.with_current_top_level(dom, self.top, cancel, |source, dom| {
            self.load_profile &= source
                .local_html_load_profile_continues(
                    dom.raw(),
                    document,
                    global,
                    self.checkpoint,
                    cancel,
                )
                .map_err(DomHostedRuntimeError::Dom)?;
            let Some(edits) = &self.edits else {
                return Err(DomHostedHtmlError::Parse(match self.progress {
                    Progress::Transferred => HtmlLiveParseError::HandedOff,
                    _ => HtmlLiveParseError::Aborted,
                }));
            };
            let progress = {
                let mut tree = source
                    .local_html_tree(dom.raw_mut(), edits, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?;
                self.parser
                    .advance(&mut tree, token_limit)
                    .map_err(DomHostedHtmlError::Parse)
            };
            self.checkpoint = dom.snapshot().stamp();
            if matches!(progress, Ok(HtmlLiveParseProgress::StopParsing)) {
                self.progress = Progress::Eof;
            }
            progress
        });
        result.map_err(|error| self.retain_refusal(error))
    }

    /// Authorize the existing parser to process its original EOF token.
    pub fn accept_eof(&mut self) {
        self.parser.accept_eof();
    }

    /// Transfer the original stop-parsing continuation once. The loading
    /// Document, native Global and empty load-delay profile must still qualify.
    pub fn take_stop_parsing(
        &mut self,
        hosted: &HostedRuntime,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<DomHostedHtmlStopParsing>, &DomHostedHtmlError> {
        if matches!(self.progress, Progress::Input | Progress::Transferred) {
            return Ok(None);
        }
        let top = self.top;
        let result = hosted.with_current_top_level(dom, top, cancel, |source, dom| {
            self.load_profile &= source
                .local_html_load_profile_continues(
                    dom.raw(),
                    top.document(),
                    top.global(),
                    self.checkpoint,
                    cancel,
                )
                .map_err(DomHostedRuntimeError::Dom)?;
            if !self.load_profile {
                return Err(DomHostedHtmlError::LoadProfileChanged);
            }
            self.parser
                .take_stop_parsing()
                .map(|parser| parser.map(|parser| DomHostedHtmlStopParsing { top, parser }))
                .map_err(DomHostedHtmlError::Start)
        });
        if matches!(result, Ok(Some(_))) {
            self.edits.take();
            self.progress = Progress::Transferred;
        }
        result.map_err(|error| self.retain_refusal(error))
    }

    /// Release parsing custody. Published DOM edits retain their ordinary identity.
    pub fn abort(&mut self) -> Result<(), HtmlParseStop> {
        self.edits.take();
        self.parser.abort()
    }
}

/// The original HTML stop-parsing continuation qualified against the hosted
/// top-level Document. This profile has empty parser-script and load-delay
/// state at the handoff.
#[derive(Debug)]
pub struct DomHostedHtmlStopParsing {
    top: DomHostedTopLevel,
    parser: HtmlLiveStopParsing<NodeId>,
}

impl DomHostedHtmlStopParsing {
    /// Canonical identities for the receiving lifecycle operation.
    pub fn top_level(&self) -> DomHostedTopLevel {
        self.top
    }

    /// Borrow the original EOF, open elements and parser accounting.
    pub fn parser_continuation(&self) -> &HtmlLiveStopParsing<NodeId> {
        &self.parser
    }
}
