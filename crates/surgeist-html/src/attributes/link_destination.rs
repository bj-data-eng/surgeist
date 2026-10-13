//! Link as content state and independent actual-rel author requirements.
//! Vocabulary only: no Fetch translation, loading or module execution.

use super::*;
use crate::OrderedAttributes;

const FETCH_VOCABULARY: &str = concat!(
    "https://fetch.spec.whatwg.org/commit-snapshots/",
    "e9460d1b22ead1c6a72eeac37fb2ec0dc01aeb1d/#request-destination-script-like"
);

keyword_type!(HtmlLinkDestination, As, [
    AudioWorklet => "audioworklet", Fetch => "fetch", Font => "font",
    Image => "image", Json => "json", PaintWorklet => "paintworklet",
    Script => "script", ServiceWorker => "serviceworker", SharedWorker => "sharedworker",
    Style => "style", Text => "text", Track => "track", Worker => "worker"
]);

impl HtmlLinkDestination {
    fn is_preload(self) -> bool {
        matches!(
            self,
            Self::Fetch | Self::Font | Self::Image | Self::Script | Self::Style | Self::Track
        )
    }
    fn is_module_preload(self) -> bool {
        matches!(
            self,
            Self::AudioWorklet
                | Self::Json
                | Self::PaintWorklet
                | Self::Script
                | Self::ServiceWorker
                | Self::SharedWorker
                | Self::Style
                | Self::Text
                | Self::Worker
        )
    }
}

enum_rule!(LINK_AS, HtmlLinkDestination, As,
    Context::Elements(&[KnownHtmlElement::Link]), [10], "attr-link-as",
    None, None, None,
    ["audioworklet" => ("audioworklet", HtmlLinkDestination::AudioWorklet),
     "fetch" => ("fetch", HtmlLinkDestination::Fetch),
     "font" => ("font", HtmlLinkDestination::Font),
     "image" => ("image", HtmlLinkDestination::Image),
     "json" => ("json", HtmlLinkDestination::Json),
     "paintworklet" => ("paintworklet", HtmlLinkDestination::PaintWorklet),
     "script" => ("script", HtmlLinkDestination::Script),
     "serviceworker" => ("serviceworker", HtmlLinkDestination::ServiceWorker),
     "sharedworker" => ("sharedworker", HtmlLinkDestination::SharedWorker),
     "style" => ("style", HtmlLinkDestination::Style),
     "text" => ("text", HtmlLinkDestination::Text),
     "track" => ("track", HtmlLinkDestination::Track),
     "worker" => ("worker", HtmlLinkDestination::Worker)],
    vocabulary: FETCH_VOCABULARY);

/// Actual rel author evidence, independent of as keyword conformance and state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlLinkDestinationContextAuthorState {
    /// This particular rel-dependent requirement does not apply.
    NotRequired,
    Conforming,
    Nonconforming(HtmlLinkDestinationContextError),
    /// Expanded names did not select the link as rule.
    Unapplied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlLinkDestinationContextError {
    PreloadRequiresAs,
    PreloadRequiresPreloadDestination,
    ModulePreloadRequiresModuleDestination,
    AsRequiresPreloadOrModulePreloadRel,
}

/// A symbolic destination decision from the selected link-type algorithm.
///
/// This is neither an as enum default nor evidence that a request was created.
/// Both missing and invalid as have no state; modulepreload then selects Script.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlLinkDestinationProcessing {
    Unapplied,
    /// Actual rel does not select this link-type branch.
    NotSelected,
    /// Preload has no as state to translate.
    NoState,
    /// The existing state belongs to the selected branch's destination domain.
    Destination(HtmlLinkDestination),
    /// A recognized state is outside that branch's domain; it is not defaulted.
    Nonmember(HtmlLinkDestination),
    /// Modulepreload's later no-state decision, retaining the original report.
    ScriptForNoState,
}

/// Local as state plus independently decidable actual-rel author checks.
///
/// Exact slots are selected from a complete OrderedAttributes observation. Rel
/// membership does not certify its full vocabulary/uniqueness or resource links.
/// Both rel branches can be selected. Each retains its own source requirement
/// and processing decision; no single overall conformance flag is supplied.
/// The nested attribute report keeps context_author() Unchecked; the independent
/// context observations are available from this compound report's getters.
/// Ordinary allocation and per-original-value UTF-16 admission apply, without
/// an aggregate work, retained-memory or cancellation guarantee.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlLinkDestinationReport {
    attribute: HtmlAttributeReport<HtmlLinkDestination>,
    rel: Option<Attribute>,
    preload_author: HtmlLinkDestinationContextAuthorState,
    modulepreload_author: HtmlLinkDestinationContextAuthorState,
    other_rel_author: HtmlLinkDestinationContextAuthorState,
    preload_processing: HtmlLinkDestinationProcessing,
    modulepreload_processing: HtmlLinkDestinationProcessing,
}

impl HtmlLinkDestinationReport {
    pub fn attribute(&self) -> &HtmlAttributeReport<HtmlLinkDestination> {
        &self.attribute
    }
    pub fn rel_attribute(&self) -> Option<&Attribute> {
        self.rel.as_ref()
    }
    pub fn preload_author(&self) -> HtmlLinkDestinationContextAuthorState {
        self.preload_author
    }
    pub fn modulepreload_author(&self) -> HtmlLinkDestinationContextAuthorState {
        self.modulepreload_author
    }
    /// Forbidden-presence requirement when neither relevant rel token is present.
    pub fn other_rel_author(&self) -> HtmlLinkDestinationContextAuthorState {
        self.other_rel_author
    }
    pub fn preload_processing(&self) -> HtmlLinkDestinationProcessing {
        self.preload_processing
    }
    pub fn modulepreload_processing(&self) -> HtmlLinkDestinationProcessing {
        self.modulepreload_processing
    }
    pub fn rel_source_clause(&self) -> &'static str {
        source!("attr-link-rel")
    }
    pub fn preload_source_clause(&self) -> &'static str {
        source!("preload-destination")
    }
    pub fn modulepreload_source_clause(&self) -> &'static str {
        source!("module-preload-destination")
    }
    pub fn modulepreload_processing_source_clause(&self) -> &'static str {
        source!("link-type-modulepreload")
    }
    pub fn preload_processing_source_clause(&self) -> &'static str {
        source!("link-type-preload")
    }
}

impl HtmlValueInterpreter {
    /// Interprets only the link as content enum. Actual rel requirements remain
    /// Unchecked, including required presence when the as attribute is absent.
    pub fn link_as(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlLinkDestination>, HtmlValueAdmissionError> {
        let mut report = self.enumerated(&LINK_AS, element, raw)?;
        if report.applicability == HtmlAttributeApplicability::Applicable {
            report.context_author = HtmlAttributeContextAuthorState::Unchecked;
        }
        Ok(report)
    }

    /// Observes actual as and rel slots; both original values are admitted before
    /// role checks or membership scans. Href, requests and link effects are not
    /// executed. Modulepreload's projection describes its destination decision
    /// only, without asserting that execution reached it (e.g. href may be empty).
    pub fn link_as_with_attributes(
        &self,
        element: &QualifiedName,
        attributes: &OrderedAttributes,
    ) -> Result<HtmlLinkDestinationReport, HtmlValueAdmissionError> {
        let name = KnownHtmlAttribute::As.qualified_name();
        let raw = attributes.get(name.namespace(), name.local_name());
        let name = KnownHtmlAttribute::Rel.qualified_name();
        let rel = attributes.get(name.namespace(), name.local_name());
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        self.admit_attribute_raw(rel.map(Attribute::value))?;
        let attribute = self.link_as(element, raw)?;
        let mut result = HtmlLinkDestinationReport {
            attribute,
            rel: rel.cloned(),
            preload_author: HtmlLinkDestinationContextAuthorState::Unapplied,
            modulepreload_author: HtmlLinkDestinationContextAuthorState::Unapplied,
            other_rel_author: HtmlLinkDestinationContextAuthorState::Unapplied,
            preload_processing: HtmlLinkDestinationProcessing::Unapplied,
            modulepreload_processing: HtmlLinkDestinationProcessing::Unapplied,
        };
        if result.attribute.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        let has = |keyword| rel.is_some_and(|rel| crate::rel::contains(rel.value(), keyword));
        let preload = has(crate::HtmlRelKeyword::Preload);
        let modulepreload = has(crate::HtmlRelKeyword::ModulePreload);
        let state = result.attribute.intrinsic;
        result.preload_author = if !preload {
            HtmlLinkDestinationContextAuthorState::NotRequired
        } else if raw.is_none() {
            HtmlLinkDestinationContextAuthorState::Nonconforming(
                HtmlLinkDestinationContextError::PreloadRequiresAs,
            )
        } else if state.is_some_and(HtmlLinkDestination::is_preload) {
            HtmlLinkDestinationContextAuthorState::Conforming
        } else {
            HtmlLinkDestinationContextAuthorState::Nonconforming(
                HtmlLinkDestinationContextError::PreloadRequiresPreloadDestination,
            )
        };
        result.modulepreload_author = if !modulepreload || raw.is_none() {
            HtmlLinkDestinationContextAuthorState::NotRequired
        } else if state.is_some_and(HtmlLinkDestination::is_module_preload) {
            HtmlLinkDestinationContextAuthorState::Conforming
        } else {
            HtmlLinkDestinationContextAuthorState::Nonconforming(
                HtmlLinkDestinationContextError::ModulePreloadRequiresModuleDestination,
            )
        };
        result.other_rel_author = if preload || modulepreload || raw.is_none() {
            HtmlLinkDestinationContextAuthorState::NotRequired
        } else {
            HtmlLinkDestinationContextAuthorState::Nonconforming(
                HtmlLinkDestinationContextError::AsRequiresPreloadOrModulePreloadRel,
            )
        };
        result.preload_processing = if !preload {
            HtmlLinkDestinationProcessing::NotSelected
        } else {
            match state {
                None => HtmlLinkDestinationProcessing::NoState,
                Some(state) if state.is_preload() => {
                    HtmlLinkDestinationProcessing::Destination(state)
                }
                Some(state) => HtmlLinkDestinationProcessing::Nonmember(state),
            }
        };
        result.modulepreload_processing = if !modulepreload {
            HtmlLinkDestinationProcessing::NotSelected
        } else {
            match state {
                None => HtmlLinkDestinationProcessing::ScriptForNoState,
                Some(state) if state.is_module_preload() => {
                    HtmlLinkDestinationProcessing::Destination(state)
                }
                Some(state) => HtmlLinkDestinationProcessing::Nonmember(state),
            }
        };
        Ok(result)
    }
}
