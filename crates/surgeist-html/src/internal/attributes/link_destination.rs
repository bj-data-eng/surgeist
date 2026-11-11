//! Link as content state and independent actual-rel author requirements.
//! Vocabulary only: no Fetch translation, loading or module execution.

use super::*;

const FETCH_VOCABULARY: &str = concat!(
    "https://fetch.spec.whatwg.org/commit-snapshots/",
    "e9460d1b22ead1c6a72eeac37fb2ec0dc01aeb1d/#request-destination-script-like"
);

// Each variant is a Fetch request destination named by the `as` attribute. A
// preload destination is Fetch, Font, Image, Script, Style or Track (HTML877d
// #link-type-preload). A module preload destination is Json, Style, Text or a script-like
// destination: AudioWorklet, PaintWorklet, Script, ServiceWorker, SharedWorker or
// Worker (HTML877d #link-type-modulepreload).
keyword_type!(HtmlLinkDestination, As, [
    /// The `audioworklet` destination, for audio worklet modules. A script-like
    /// destination, so it is a module preload destination only.
    AudioWorklet => "audioworklet",
    /// The `fetch` destination, for generic fetched resources. A preload
    /// destination only.
    Fetch => "fetch",
    /// The `font` destination, for font resources. A preload destination only.
    Font => "font",
    /// The `image` destination, for images. A preload destination only.
    Image => "image",
    /// The `json` destination, for JSON modules. A module preload destination
    /// only.
    Json => "json",
    /// The `paintworklet` destination, for paint worklet modules. A script-like
    /// destination, so it is a module preload destination only.
    PaintWorklet => "paintworklet",
    /// The `script` destination, for classic or module scripts. Both a preload
    /// and a module preload destination, and the destination that
    /// `modulepreload` selects when `as` carries no state.
    Script => "script",
    /// The `serviceworker` destination, for service worker scripts. A
    /// script-like destination, so it is a module preload destination only.
    ServiceWorker => "serviceworker",
    /// The `sharedworker` destination, for shared worker scripts. A
    /// script-like destination, so it is a module preload destination only.
    SharedWorker => "sharedworker",
    /// The `style` destination, for style sheets. Both a preload and a module
    /// preload destination.
    Style => "style",
    /// The `text` destination, for text modules. A module preload destination
    /// only.
    Text => "text",
    /// The `track` destination, for text tracks. A preload destination only.
    Track => "track",
    /// The `worker` destination, for dedicated worker scripts. A script-like
    /// destination, so it is a module preload destination only.
    Worker => "worker"
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
    /// The requirement applies and the link meets it.
    Conforming,
    /// The requirement applies and the link violates it. The payload names the
    /// violated requirement.
    Nonconforming(HtmlLinkDestinationContextError),
    /// Expanded names did not select the link as rule.
    Unapplied,
}

/// A requirement that ties the `as` attribute to the link's `rel` keywords.
///
/// The requirements come from the `preload` and `modulepreload` link types
/// (HTML877d #link-type-preload and #link-type-modulepreload) and the `as` attribute of `link`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlLinkDestinationContextError {
    /// `rel` contains `preload` but the link has no `as` attribute.
    PreloadRequiresAs,
    /// `rel` contains `preload` and `as` is present but is not a preload
    /// destination: its value is unrecognized or names another destination.
    PreloadRequiresPreloadDestination,
    /// `rel` contains `modulepreload` and `as` is present but is not a module
    /// preload destination. An absent `as` is allowed there.
    ModulePreloadRequiresModuleDestination,
    /// `as` is present but `rel` contains neither `preload` nor `modulepreload`.
    AsRequiresPreloadOrModulePreloadRel,
}

impl std::fmt::Display for HtmlLinkDestinationContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::PreloadRequiresAs => "preload link requires an as attribute",
            Self::PreloadRequiresPreloadDestination => {
                "preload link as attribute must name a preload destination"
            }
            Self::ModulePreloadRequiresModuleDestination => {
                "modulepreload link as attribute must name a script-like destination"
            }
            Self::AsRequiresPreloadOrModulePreloadRel => {
                "as attribute requires the preload or modulepreload relationship"
            }
        })
    }
}
impl std::error::Error for HtmlLinkDestinationContextError {}

/// A symbolic destination decision from the selected link-type algorithm.
///
/// This is neither an as enum default nor evidence that a request was created.
/// Both missing and invalid as have no state; modulepreload then selects Script.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlLinkDestinationProcessing {
    /// The rule does not apply to these names, so there is no link-type branch.
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
    /// The report for the `as` attribute itself: its exact raw value and the
    /// recognized destination, if any.
    pub fn attribute(&self) -> &HtmlAttributeReport<HtmlLinkDestination> {
        &self.attribute
    }
    /// The exact `rel` attribute that was consulted, or `None` when absent.
    pub fn rel_attribute(&self) -> Option<&Attribute> {
        self.rel.as_ref()
    }
    /// Whether `as` satisfies the `preload` requirements. `NotRequired` when
    /// `rel` lacks the `preload` keyword.
    pub fn preload_author(&self) -> HtmlLinkDestinationContextAuthorState {
        self.preload_author
    }
    /// Whether `as` satisfies the `modulepreload` requirements. `NotRequired`
    /// when `rel` lacks the keyword or `as` is absent.
    pub fn modulepreload_author(&self) -> HtmlLinkDestinationContextAuthorState {
        self.modulepreload_author
    }
    /// Forbidden-presence requirement when neither relevant rel token is present.
    pub fn other_rel_author(&self) -> HtmlLinkDestinationContextAuthorState {
        self.other_rel_author
    }
    /// The destination decision of the `preload` link type. `NotSelected` when
    /// `rel` lacks the keyword.
    pub fn preload_processing(&self) -> HtmlLinkDestinationProcessing {
        self.preload_processing
    }
    /// The destination decision of the `modulepreload` link type. `NotSelected`
    /// when `rel` lacks the keyword.
    pub fn modulepreload_processing(&self) -> HtmlLinkDestinationProcessing {
        self.modulepreload_processing
    }
    /// The clause that defines the `rel` attribute of `link`.
    pub fn rel_source_clause(&self) -> &'static str {
        source!("attr-link-rel")
    }
    /// The clause that defines a preload destination (HTML877d #link-type-preload).
    pub fn preload_source_clause(&self) -> &'static str {
        source!("preload-destination")
    }
    /// The clause that defines a module preload destination (HTML877d #link-type-modulepreload).
    pub fn modulepreload_source_clause(&self) -> &'static str {
        source!("module-preload-destination")
    }
    /// The clause that defines the `modulepreload` link type (HTML877d #link-type-modulepreload).
    pub fn modulepreload_processing_source_clause(&self) -> &'static str {
        source!("link-type-modulepreload")
    }
    /// The clause that defines the `preload` link type (HTML877d #link-type-preload).
    pub fn preload_processing_source_clause(&self) -> &'static str {
        source!("link-type-preload")
    }
}

impl HtmlValueInterpreter {
    /// Interprets only the link as content enum. Actual rel requirements remain
    /// Unchecked, including required presence when the as attribute is absent.
    pub fn link_as(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
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
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        attributes: &(impl crate::internal::values::semantic_input::HtmlAttributesInput + ?Sized),
    ) -> Result<HtmlLinkDestinationReport, HtmlValueAdmissionError> {
        let name = KnownHtmlAttribute::As.qualified_name();
        let raw = find_attribute(attributes, name.namespace(), name.local_name());
        let name = KnownHtmlAttribute::Rel.qualified_name();
        let rel = find_attribute(attributes, name.namespace(), name.local_name());
        self.admit_attribute_raw(raw.as_ref().map(HtmlAttributeInput::value))?;
        self.admit_attribute_raw(rel.as_ref().map(HtmlAttributeInput::value))?;
        let attribute = self.link_as(element, raw.as_ref())?;
        let mut result = HtmlLinkDestinationReport {
            attribute,
            rel: rel.as_ref().map(capture_attribute),
            preload_author: HtmlLinkDestinationContextAuthorState::Unapplied,
            modulepreload_author: HtmlLinkDestinationContextAuthorState::Unapplied,
            other_rel_author: HtmlLinkDestinationContextAuthorState::Unapplied,
            preload_processing: HtmlLinkDestinationProcessing::Unapplied,
            modulepreload_processing: HtmlLinkDestinationProcessing::Unapplied,
        };
        if result.attribute.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        let has = |keyword| {
            rel.as_ref()
                .is_some_and(|rel| crate::internal::elements::rel::contains(rel.value(), keyword))
        };
        let preload = has(crate::internal::elements::rel::HtmlRelKeyword::Preload);
        let modulepreload = has(crate::internal::elements::rel::HtmlRelKeyword::ModulePreload);
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
