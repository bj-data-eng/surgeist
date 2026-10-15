//! Actual bounded native HTML Script metadata, in the canonical identity phase.
use crate::{DocumentId, DomNode, NodeId};
use surgeist_html::{HtmlScriptFlags, HtmlScriptKind, HtmlScriptReadySteps, HtmlScriptResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScriptState<K> {
    pub(crate) flags: HtmlScriptFlags,
    pub(crate) kind: Option<HtmlScriptKind>,
    pub(crate) parser_document: Option<K>,
    pub(crate) preparation_document: Option<K>,
    pub(crate) result: HtmlScriptResult,
    pub(crate) ready: Option<HtmlScriptReadySteps>,
}
impl<K> ScriptState<K> {
    pub(crate) fn fresh() -> Self {
        Self {
            flags: HtmlScriptFlags::fresh(),
            kind: None,
            parser_document: None,
            preparation_document: None,
            result: HtmlScriptResult::Uninitialized,
            ready: None,
        }
    }
    pub(crate) fn map<T>(self, mut map: impl FnMut(K) -> T) -> ScriptState<T> {
        ScriptState {
            flags: self.flags,
            kind: self.kind,
            parser_document: self.parser_document.map(&mut map),
            preparation_document: self.preparation_document.map(map),
            result: self.result,
            ready: self.ready,
        }
    }
}

/// Checked native HTML Script state at one immutable canonical revision.
///
/// Native null browsing context stops preparation only after source-required
/// type/started/preparation-document writes. This view grants no execution,
/// parsed-history installation, loader, or result-ready callback capability.
#[derive(Clone, Copy, Debug)]
pub struct DomHtmlScript<'a> {
    node: DomNode<'a>,
}
impl DomHtmlScript<'_> {
    fn state(self) -> ScriptState<NodeId> {
        self.node.node.script.expect("checked native HTML Script")
    }
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn flags(self) -> HtmlScriptFlags {
        self.state().flags
    }
    pub fn script_type(self) -> Option<HtmlScriptKind> {
        self.state().kind
    }
    pub fn parser_document(self) -> Option<DocumentId> {
        self.state().parser_document.map(DocumentId)
    }
    /// Historic preparation identity; adoption does not rewrite it.
    pub fn preparation_document(self) -> Option<DocumentId> {
        self.state().preparation_document.map(DocumentId)
    }
    pub fn result(self) -> HtmlScriptResult {
        self.state().result
    }
    pub fn steps_when_result_ready(self) -> Option<HtmlScriptReadySteps> {
        self.state().ready
    }
}
impl<'a> DomNode<'a> {
    pub fn as_html_script(self) -> Option<DomHtmlScript<'a>> {
        self.node.script?;
        Some(DomHtmlScript { node: self })
    }
}
