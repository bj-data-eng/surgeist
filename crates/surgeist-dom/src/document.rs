use std::{fmt, sync::Arc};

use crate::{DocumentString, DomError, HtmlDocumentMode};

const CONTENT_TYPE: &str = "application/xml";
const ENCODING: &str = "UTF-8";
const URL: &str = "about:blank";
// All three source-defined spellings are ASCII, so byte and UTF-16 lengths agree.
pub(crate) const NATIVE_DOCUMENT_STRING_UNITS: usize =
    CONTENT_TYPE.len() + ENCODING.len() + URL.len();

/// Source-defined document type, independent of namespaces and compatibility mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomDocumentClass {
    Xml,
    Html,
}

/// The actual registry selected by controlled Document/Element creation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomRegistryBinding {
    Null,
    Registry(crate::registry::DomRegistryId),
}

#[derive(Debug)]
struct OpaqueOriginIdentity;

/// Real opaque-origin identity, unrelated to URL spelling or document labels.
///
/// Cloning preserves identity. Fresh native Documents get distinct tokens,
/// including across stores; imported parser origins preserve their actual
/// source identity across imports. No serialized/raw identity or construction escape is
/// exposed. Private aborted drafts cannot publish or consume an observable ID.
#[derive(Clone)]
pub struct DomOpaqueOrigin(OpaqueOriginBacking);

#[derive(Clone)]
enum OpaqueOriginBacking {
    Native(Arc<OpaqueOriginIdentity>),
    ParsedFragment(surgeist_html_parser::HtmlParseOpaqueOrigin),
}

impl PartialEq for DomOpaqueOrigin {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (OpaqueOriginBacking::Native(a), OpaqueOriginBacking::Native(b)) => Arc::ptr_eq(a, b),
            (OpaqueOriginBacking::ParsedFragment(a), OpaqueOriginBacking::ParsedFragment(b)) => {
                a == b
            }
            _ => false,
        }
    }
}
impl Eq for DomOpaqueOrigin {}
impl fmt::Debug for DomOpaqueOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DomOpaqueOrigin(..)")
    }
}

/// Actual creation cause and environment qualification, independent of labels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomDocumentCreationCause {
    Native,
    ParsedOffline {
        url: surgeist_html_semantics::HtmlBaseUrlRecord,
        encoding: surgeist_html_semantics::HtmlBaseEncoding,
    },
    ParsedTemplateOwner,
    /// The actual auxiliary Document created by the original fragment parser.
    ParsedFragmentParser,
    /// The actual controlled initial no-opener browsing-context constructor.
    HostedInitialAboutBlank {
        source: crate::hosted::DomHostedSourceId,
        operation: crate::hosted::DomHostedOperationId,
    },
}
/// Canonical document metadata, captured together with its tree and revision.
///
/// Native defaults follow the selected DOM document clause. Host/imported facts
/// have no unchecked constructor here. Native HTML creation publishes its full
/// source skeleton and copies an actual associated Document origin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomDocumentMetadata {
    class: DomDocumentClass,
    content_type: DocumentString,
    encoding: DocumentString,
    url: DocumentString,
    origin: DomOpaqueOrigin,
    mode: HtmlDocumentMode,
    design_mode_enabled: bool,
    allow_declarative_shadow_roots: bool,
    registry: DomRegistryBinding,
    environment: DomDocumentEnvironment,
    cause: DomDocumentCreationCause,
    meta: Option<crate::DomMetaDocumentState>,
    policy: crate::DomDocumentPolicyContainer,
    meta_ui: NoAssociatedMetaThemeConsumer,
}

/// Established by the controlled Document creators, independently of no BC or
/// offline processing. Current association fronts admit focus/hover/selection,
/// but no recipient for Meta theme results. A future such association must
/// replace this qualification before it can publish a ready Document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NoAssociatedMetaThemeConsumer;

/// Selected closed offline environment shared by native creation and completed
/// offline installation. Creation provenance and URL/encoding identities remain
/// independently recorded in DomDocumentCreationCause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomDocumentEnvironment {
    input: DomNativeInputPlatform,
    pub(crate) hosted: Option<Arc<crate::hosted::DocumentAssociation>>,
}

/// Native-owner qualification, independent of browsing-context or scripting
/// flags. Only the private native environment constructor establishes it.
///
/// All native entrypoints admit inert strings/values and canonical identities,
/// never network/data/same-document script acquisition, inline/external execution,
/// executable handlers/listeners, SVG load/error recipients or script acquisition,
/// execution or notification callback/task/resource/event outputs. Host
/// observations are selection facts only; Slot
/// intentions cannot represent an SVG event. Native fragment installation creates
/// fresh values; snapshot import clones another checked native owner. Neither
/// admits retained script resources/history. A future capability-bearing front
/// must qualify its real transition before it can retain this closed witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ClosedSvgScriptProcessing {
    _sealed: (),
}
impl ClosedSvgScriptProcessing {
    pub(crate) const fn native_creation() -> Self {
        Self { _sealed: () }
    }
    /// Tiny step3 remains real while acquisition, execution and unobservable
    /// notification are omitted under the independently established closed owner.
    pub(crate) fn mark_processed(self, flags: &mut surgeist_html::SvgScriptFlags) {
        flags.mark_processed();
    }
}

/// Established native selection capabilities, independent of browsing context.
/// Construction fixes both facts for this native environment's lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomNativeInputPlatform {
    empty_selection: surgeist_html::HtmlInputEmptySelectionSupport,
    none_direction: surgeist_html::HtmlInputNoneDirectionSupport,
}
impl DomNativeInputPlatform {
    pub const fn new(
        empty_selection: surgeist_html::HtmlInputEmptySelectionSupport,
        none_direction: surgeist_html::HtmlInputNoneDirectionSupport,
    ) -> Self {
        Self {
            empty_selection,
            none_direction,
        }
    }
    /// Selected headless profile; neither capability is inferred from no BC.
    pub const fn headless() -> Self {
        Self::new(
            surgeist_html::HtmlInputEmptySelectionSupport::Supported,
            surgeist_html::HtmlInputNoneDirectionSupport::Supported,
        )
    }
    pub const fn empty_selection(self) -> surgeist_html::HtmlInputEmptySelectionSupport {
        self.empty_selection
    }
    pub const fn none_direction(self) -> surgeist_html::HtmlInputNoneDirectionSupport {
        self.none_direction
    }
}
impl DomDocumentEnvironment {
    pub(crate) const fn new(input: DomNativeInputPlatform) -> Self {
        Self {
            input,
            hosted: None,
        }
    }
    pub const fn input_platform(&self) -> DomNativeInputPlatform {
        self.input
    }
    /// Selected native creation policy, not an inference from missing resources.
    pub const fn input_images(&self) -> surgeist_html::HtmlInputImagesDisabled {
        surgeist_html::HtmlInputImagesDisabled::ExplicitlyDisabled
    }
    pub fn browsing_context(&self) -> surgeist_html::HtmlBrowsingContextPresence {
        match &self.hosted {
            Some(association) if association.phase != crate::hosted::DocumentPhase::Retired => {
                surgeist_html::HtmlBrowsingContextPresence::Present
            }
            _ => surgeist_html::HtmlBrowsingContextPresence::Absent,
        }
    }
    pub const fn has_ever_had_navigable(&self) -> bool {
        self.hosted.is_some()
    }
    /// The admitted native execution owner supplies no scripting engine. This
    /// qualification is independent of the actual browsing-context association.
    pub const fn scripting(&self) -> surgeist_html_semantics::HtmlScriptingStatus {
        surgeist_html_semantics::HtmlScriptingStatus::Disabled
    }
}

impl DomDocumentMetadata {
    pub(crate) fn fresh_opaque_origin() -> DomOpaqueOrigin {
        DomOpaqueOrigin(OpaqueOriginBacking::Native(Arc::new(OpaqueOriginIdentity)))
    }
    pub(crate) fn hosted_initial(
        input: DomNativeInputPlatform,
        origin: DomOpaqueOrigin,
        registry: crate::registry::DomRegistryId,
        association: Arc<crate::hosted::DocumentAssociation>,
        operation: crate::hosted::DomHostedOperationId,
        materialize: &mut impl FnMut(&str) -> Result<DocumentString, DomError>,
    ) -> Result<Self, DomError> {
        let source = operation.source();
        let mut metadata = Self::native_defaults(
            DomDocumentClass::Html,
            "text/html",
            origin,
            DomDocumentEnvironment {
                input,
                hosted: Some(association),
            },
            false,
            materialize,
        )?;
        metadata.mode = HtmlDocumentMode::Quirks;
        metadata.allow_declarative_shadow_roots = true;
        metadata.registry = DomRegistryBinding::Registry(registry);
        metadata.cause = DomDocumentCreationCause::HostedInitialAboutBlank { source, operation };
        Ok(metadata)
    }
    pub(crate) fn native(
        environment: DomDocumentEnvironment,
        materialize: &mut impl FnMut(&str) -> Result<DocumentString, DomError>,
    ) -> Result<Self, DomError> {
        Self::native_defaults(
            DomDocumentClass::Xml,
            CONTENT_TYPE,
            DomOpaqueOrigin(OpaqueOriginBacking::Native(Arc::new(OpaqueOriginIdentity))),
            environment,
            false,
            materialize,
        )
    }
    pub(crate) fn template_owner(
        class: DomDocumentClass,
        environment: DomDocumentEnvironment,
        materialize: &mut impl FnMut(&str) -> Result<DocumentString, DomError>,
    ) -> Result<Self, DomError> {
        let mut metadata = Self::native(
            DomDocumentEnvironment::new(environment.input_platform()),
            materialize,
        )?;
        metadata.class = class;
        Ok(metadata)
    }
    /// Used only by the complete createHTMLDocument transaction, whose source
    /// Document supplies this identity. No caller can set arbitrary metadata.
    pub(crate) fn native_html(
        origin: DomOpaqueOrigin,
        environment: DomDocumentEnvironment,
        materialize: &mut impl FnMut(&str) -> Result<DocumentString, DomError>,
    ) -> Result<Self, DomError> {
        Self::native_defaults(
            DomDocumentClass::Html,
            "text/html",
            origin,
            DomDocumentEnvironment::new(environment.input_platform()),
            false,
            materialize,
        )
    }
    // The owning materializer admits and preserves each exact supplied literal.
    // It must not substitute metadata or transform the DocumentString contents.
    fn native_defaults(
        class: DomDocumentClass,
        content_type: &str,
        origin: DomOpaqueOrigin,
        environment: DomDocumentEnvironment,
        design_mode_enabled: bool,
        materialize: &mut impl FnMut(&str) -> Result<DocumentString, DomError>,
    ) -> Result<Self, DomError> {
        Ok(Self {
            class,
            content_type: materialize(content_type)?,
            encoding: materialize(ENCODING)?,
            url: materialize(URL)?,
            origin,
            mode: HtmlDocumentMode::NoQuirks,
            design_mode_enabled,
            allow_declarative_shadow_roots: false,
            registry: DomRegistryBinding::Null,
            environment,
            cause: DomDocumentCreationCause::Native,
            meta: Some(crate::DomMetaDocumentState::native()),
            policy: crate::DomDocumentPolicyContainer::native(),
            meta_ui: NoAssociatedMetaThemeConsumer,
        })
    }
    // Only the canonical createDocument tail calls these source-ordered writes.
    // They do not provide caller mutation of arbitrary Document metadata.
    pub(crate) fn finish_xml_origin(&mut self, origin: DomOpaqueOrigin) {
        self.origin = origin;
    }
    pub(crate) fn finish_xml_content_type(&mut self, content_type: DocumentString) {
        self.content_type = content_type;
    }
    pub(crate) fn from_parsed(
        class: DomDocumentClass,
        mode: HtmlDocumentMode,
        environment: &crate::DomParsedDocumentEnvironment,
        design_mode_enabled: bool,
    ) -> Self {
        let mut materialize = |text: &str| Ok(text.into());
        let mut result = Self::native_defaults(
            class,
            if class == DomDocumentClass::Html {
                "text/html"
            } else {
                CONTENT_TYPE
            },
            environment.origin.clone(),
            DomDocumentEnvironment::new(environment.input),
            design_mode_enabled,
            &mut materialize,
        )
        .expect("parsed metadata materializer always returns Ok");
        result.mode = mode;
        result.url = environment.url.serialization().clone();
        result.encoding = environment.encoding_label.clone();
        result.cause = DomDocumentCreationCause::ParsedOffline {
            url: environment.url.clone(),
            encoding: environment.encoding.clone(),
        };
        // Target association closure is independent of imported processing
        // history. The installer must qualify this Document's original parser
        // record before explicitly establishing its Meta/policy history.
        result.meta = None;
        result.policy = crate::DomDocumentPolicyContainer::unavailable();
        result
    }
    pub(crate) fn parsed_template_owner(
        class: DomDocumentClass,
        input: DomNativeInputPlatform,
        design_mode_enabled: bool,
    ) -> Self {
        let mut materialize = |text: &str| Ok(text.into());
        let mut result =
            Self::template_owner(class, DomDocumentEnvironment::new(input), &mut materialize)
                .expect("parsed template metadata materializer always returns Ok");
        result.design_mode_enabled = design_mode_enabled;
        result.cause = DomDocumentCreationCause::ParsedTemplateOwner;
        result.meta = None;
        result.policy = crate::DomDocumentPolicyContainer::unavailable();
        result
    }
    pub fn creation_cause(&self) -> &DomDocumentCreationCause {
        &self.cause
    }
    /// Consume the original completed fragment-parser constructor proof. Its
    /// source origin is retained directly; target input and UI association
    /// qualification are established independently by this controlled creator.
    pub(crate) fn from_fragment_parser(
        descriptor: &surgeist_html_parser::HtmlParseFragmentParserDocument<'_>,
        current: DomDocumentEnvironment,
        design_mode_enabled: bool,
    ) -> Self {
        let mut materialize = |text: &str| Ok(text.into());
        let mut result = Self::native_defaults(
            DomDocumentClass::Html,
            CONTENT_TYPE,
            DomOpaqueOrigin(OpaqueOriginBacking::ParsedFragment(
                descriptor.origin().clone(),
            )),
            current,
            design_mode_enabled,
            &mut materialize,
        )
        .expect("fragment parser metadata materializer always returns Ok");
        result.mode = descriptor.mode();
        result.cause = DomDocumentCreationCause::ParsedFragmentParser;
        // Only the separate, qualified per-Document Meta view establishes this
        // original processing history. Constructor facts never substitute it.
        result.meta = None;
        result.policy = crate::DomDocumentPolicyContainer::unavailable();
        result
    }
    pub fn class(&self) -> DomDocumentClass {
        self.class
    }
    pub fn content_type(&self) -> &DocumentString {
        &self.content_type
    }
    pub fn encoding(&self) -> &DocumentString {
        &self.encoding
    }
    /// Exact URL serialization; this bounded API does not accept arbitrary URLs.
    pub fn url(&self) -> &DocumentString {
        &self.url
    }
    pub fn origin(&self) -> &DomOpaqueOrigin {
        &self.origin
    }
    pub fn mode(&self) -> HtmlDocumentMode {
        self.mode
    }
    /// Actual canonical Document design-mode flag. This readback neither
    /// executes editing nor supplies the ordered designMode setter.
    pub const fn design_mode_enabled(&self) -> bool {
        self.design_mode_enabled
    }
    pub fn allow_declarative_shadow_roots(&self) -> bool {
        self.allow_declarative_shadow_roots
    }
    pub fn custom_element_registry(&self) -> DomRegistryBinding {
        self.registry
    }
    pub fn document_environment(&self) -> DomDocumentEnvironment {
        self.environment.clone()
    }
    pub fn initial_permissions_policy(&self) -> Option<crate::hosted::DomInitialPermissionsPolicy> {
        self.environment
            .hosted
            .as_ref()
            .map(|association| association.policy)
    }
    pub(crate) fn set_hosted_association(
        &mut self,
        association: Arc<crate::hosted::DocumentAssociation>,
    ) {
        self.environment.hosted = Some(association);
    }
    /// Actual processing history. `None` denotes unavailable imported
    /// history; current attributes never stand in for completed processing.
    pub fn meta_state(&self) -> Option<&crate::DomMetaDocumentState> {
        self.meta.as_ref()
    }
    /// The one canonical supported policy-container authority for this Document.
    pub fn policy_container(&self) -> &crate::DomDocumentPolicyContainer {
        &self.policy
    }
    /// Establish only the actual original-parser history already qualified by
    /// the installer for this fresh Document. Its controlled creator's current
    /// target UI qualification remains independent of the source profile.
    pub(crate) fn establish_parsed_meta_history(
        &mut self,
        language: Option<DocumentString>,
        referrer: surgeist_html::HtmlReferrerPolicy,
        will_declaratively_refresh: bool,
    ) {
        self.meta = Some(crate::DomMetaDocumentState {
            pragma_language: language,
            will_declaratively_refresh,
        });
        self.policy.referrer = Some(referrer);
    }
    pub(crate) fn meta_state_mut(&mut self) -> Option<&mut crate::DomMetaDocumentState> {
        self.meta.as_mut()
    }
    pub(crate) fn policy_container_mut(&mut self) -> &mut crate::DomDocumentPolicyContainer {
        &mut self.policy
    }
    /// Complete application of a no-theme result to the actually empty recipient
    /// set. Selection must already have run over this Document's current tree.
    pub(crate) fn apply_no_meta_theme(&self) {
        let NoAssociatedMetaThemeConsumer = self.meta_ui;
    }
    fn retained_strings(&self) -> impl Iterator<Item = &DocumentString> {
        [
            Some(&self.content_type),
            Some(&self.encoding),
            Some(&self.url),
            self.meta
                .as_ref()
                .and_then(|meta| meta.pragma_language.as_ref()),
        ]
        .into_iter()
        .flatten()
    }
    /// Retained payload owned by this metadata; its inline header is counted by
    /// the enclosing node or source record. Keep consumers out of field pricing.
    pub(crate) fn retained_payload_units(&self) -> Result<usize, crate::DomError> {
        let mut units = 0usize;
        for string in self.retained_strings() {
            units = units
                .checked_add(string.shared_code_units().capacity())
                .ok_or(crate::DomError::RepresentationExhausted)?;
        }
        if let DomDocumentCreationCause::ParsedOffline { url, .. } = &self.cause {
            units = units
                .checked_add(url.retained_storage_usage().utf16_extent())
                .ok_or(crate::DomError::RepresentationExhausted)?;
        }
        Ok(units)
    }
    pub(crate) fn retained_payload_bytes(&self) -> Result<usize, crate::DomError> {
        let mut bytes = 0usize;
        for string in self.retained_strings() {
            bytes = bytes
                .checked_add(std::mem::size_of::<Vec<u16>>())
                .and_then(|n| {
                    n.checked_add(
                        string
                            .shared_code_units()
                            .capacity()
                            .checked_mul(std::mem::size_of::<u16>())?,
                    )
                })
                .ok_or(crate::DomError::RepresentationExhausted)?;
        }
        if let DomDocumentCreationCause::ParsedOffline { url, .. } = &self.cause {
            bytes = bytes
                .checked_add(url.retained_storage_usage().retained_bytes())
                .ok_or(crate::DomError::RepresentationExhausted)?;
        }
        if self.environment.hosted.is_some() {
            bytes = bytes
                .checked_add(std::mem::size_of::<crate::hosted::DocumentAssociation>())
                .ok_or(crate::DomError::RepresentationExhausted)?;
        }
        Ok(bytes)
    }
    pub(crate) fn units(&self) -> Option<usize> {
        self.content_type
            .len()
            .checked_add(self.encoding.len())?
            .checked_add(self.url.len())?
            .checked_add(
                self.meta
                    .as_ref()
                    .and_then(|meta| meta.pragma_language.as_ref())
                    .map_or(0, |value| value.shared_code_units().capacity()),
            )
    }
}

#[cfg(test)]
mod materialization_tests {
    use super::*;

    fn create(
        kind: u8,
        origin: &DomOpaqueOrigin,
        materialize: &mut impl FnMut(&str) -> Result<DocumentString, crate::DomError>,
    ) -> Result<DomDocumentMetadata, crate::DomError> {
        let environment = DomDocumentEnvironment::new(DomNativeInputPlatform::headless());
        match kind {
            0 => DomDocumentMetadata::native(environment, materialize),
            1 => DomDocumentMetadata::native_html(origin.clone(), environment, materialize),
            2 => DomDocumentMetadata::template_owner(
                DomDocumentClass::Html,
                environment,
                materialize,
            ),
            _ => unreachable!("closed fixture cases"),
        }
    }
    fn origin() -> DomOpaqueOrigin {
        DomDocumentMetadata::native(
            DomDocumentEnvironment::new(DomNativeInputPlatform::headless()),
            &mut |text| Ok(text.into()),
        )
        .unwrap()
        .origin()
        .clone()
    }

    #[test]
    fn native_factory_materializes_actual_defaults_in_order_and_preserves_metadata() {
        let origin = origin();
        for (kind, content_type) in [
            (0, "application/xml"),
            (1, "text/html"),
            (2, "application/xml"),
        ] {
            let mut seen = Vec::new();
            let metadata = create(kind, &origin, &mut |text| {
                seen.push(text.to_owned());
                Ok(text.into())
            })
            .unwrap();
            assert_eq!(seen, [content_type, "UTF-8", "about:blank"]);
            assert_eq!(
                metadata.content_type().code_units(),
                content_type.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                metadata.encoding().code_units(),
                "UTF-8".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                metadata.url().code_units(),
                "about:blank".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                metadata.class(),
                if kind == 0 {
                    DomDocumentClass::Xml
                } else {
                    DomDocumentClass::Html
                }
            );
            assert_eq!(metadata.mode(), HtmlDocumentMode::NoQuirks);
            assert_eq!(metadata.custom_element_registry(), DomRegistryBinding::Null);
            assert!(!metadata.allow_declarative_shadow_roots());
            assert_eq!(
                metadata.meta_state(),
                Some(&crate::DomMetaDocumentState::native())
            );
            assert_eq!(
                metadata.policy_container(),
                &crate::DomDocumentPolicyContainer::native()
            );
            assert_eq!(metadata.creation_cause(), &DomDocumentCreationCause::Native);
            assert_eq!(
                metadata.document_environment().input_platform(),
                DomNativeInputPlatform::headless()
            );
            if kind == 1 {
                assert_eq!(metadata.origin(), &origin);
            } else {
                assert_ne!(metadata.origin(), &origin);
            }
        }
    }
    #[test]
    fn materializer_refusal_preserves_first_error_without_reaching_later_strings() {
        let origin = origin();
        for (kind, content_type) in [
            (0, "application/xml"),
            (1, "text/html"),
            (2, "application/xml"),
        ] {
            let expected = [content_type, "UTF-8", "about:blank"];
            for stop in 0..3 {
                let mut seen = Vec::new();
                let result = create(kind, &origin, &mut |text| {
                    seen.push(text.to_owned());
                    if seen.len() == stop + 1 {
                        Err(crate::DomError::Cancelled)
                    } else {
                        Ok(text.into())
                    }
                });
                assert_eq!(result.unwrap_err(), crate::DomError::Cancelled);
                assert_eq!(seen, expected[..=stop]);
            }
        }
    }
}
