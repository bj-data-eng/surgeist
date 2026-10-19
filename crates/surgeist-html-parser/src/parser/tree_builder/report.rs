//! Complete private immutable report publication over the original request and
//! qualified canonical storage. Public entry/state vocabulary promotion is later.

use super::{context::ownership::SealedFragmentConstruction, seal::SealedOrdinaryConstruction};
use crate::{
    construction::{
        SessionError, StandaloneDocumentState,
        report::{MetadataCursor, PreparedReportStorage, ReportStorage},
    },
    parser::control::ParseUsage,
    request::{DocumentRequestParts, FragmentRequestParts},
};
use std::mem::size_of;

enum ReportContext {
    Document {
        request: DocumentRequestParts,
        state: StandaloneDocumentState,
    },
    Fragment(FragmentRequestParts),
}

pub struct ImmutableParseReport {
    context: ReportContext,
    storage: ReportStorage,
    usage: ParseUsage,
}
impl ImmutableParseReport {
    /// Authoritative current original-controller construction capacity at this
    /// completed report's publication. Includes retained selected A/Img graphs
    /// and owned vector/string payloads; excludes released execution banks.
    /// Original source backing, this report header and separately lent root or
    /// inert Document BaseFacts provider graphs remain outside this measure.
    pub fn retained_construction_bytes(&self) -> u64 {
        self.usage
            .resource(crate::HtmlParseResource::ManagedCapacityBytes)
            .current()
    }
    /// Refusal retains the same source-owning native seal. Generic native-only
    /// completion cannot supply the actual D1/history proof required here.
    pub(crate) fn take_document(
        input: &mut Option<SealedOrdinaryConstruction>,
    ) -> Result<Self, SessionError> {
        let sealed = input.as_mut().ok_or(SessionError::Consumed)?;
        sealed
            .parts
            .arena
            .as_ref()
            .ok_or(SessionError::Consumed)?
            .require_original_transfer(&mut sealed.parts.control)?;
        sealed.parts.prepare_report_request(&sealed.request)?;
        let state = sealed.parts.document_state();
        let c = &mut sealed.parts.control;
        crate::construction::read::prepare_public_readers(&sealed.parts.arena, c)?;
        let storage = PreparedReportStorage::prepare(
            &mut sealed.parts.arena,
            c,
            sealed.request.base_facts(),
            sealed.request.is_offline(),
        )?;
        let mut storage = storage.share_values(c)?;
        let c = storage.control();
        super::write(
            size_of::<Self>()
                + size_of::<Option<SealedOrdinaryConstruction>>()
                + size_of::<SealedOrdinaryConstruction>()
                + size_of::<ReportContext>()
                + size_of::<ParseUsage>(),
            5,
            c,
        )?;
        c.checkpoint()?;
        let mut storage = storage.commit();
        let c = &mut sealed.parts.control;
        storage.release_execution_inputs(c);
        sealed.request.release_associated_inputs();
        let usage = c.usage().clone();
        let sealed = input.take().expect("qualified original document seal");
        // The remaining source binding and execution control are consumed here;
        // no useful work, release/refund or cancellation check follows snapshot.
        Ok(Self {
            context: ReportContext::Document {
                request: sealed.request,
                state,
            },
            storage,
            usage,
        })
    }
    pub(crate) fn take_fragment(
        input: &mut Option<SealedFragmentConstruction>,
    ) -> Result<Self, SessionError> {
        let sealed = input.as_mut().ok_or(SessionError::Consumed)?;
        sealed
            .parts
            .arena
            .as_ref()
            .ok_or(SessionError::Consumed)?
            .require_original_transfer(&mut sealed.parts.control)?;
        sealed.parts.prepare_report_request(&sealed.request)?;
        let c = &mut sealed.parts.control;
        crate::construction::read::prepare_public_readers(&sealed.parts.arena, c)?;
        let storage = PreparedReportStorage::prepare(&mut sealed.parts.arena, c, None, true)?;
        let mut storage = storage.share_values(c)?;
        let c = storage.control();
        super::write(
            size_of::<Self>()
                + size_of::<Option<SealedFragmentConstruction>>()
                + size_of::<SealedFragmentConstruction>()
                + size_of::<ReportContext>()
                + size_of::<ParseUsage>(),
            5,
            c,
        )?;
        c.checkpoint()?;
        let mut storage = storage.commit();
        let c = &mut sealed.parts.control;
        storage.release_execution_inputs(c);
        sealed.request.release_associated_inputs();
        let usage = c.usage().clone();
        let sealed = input.take().expect("qualified original fragment seal");
        Ok(Self {
            context: ReportContext::Fragment(sealed.request),
            storage,
            usage,
        })
    }
    pub fn document_base_facts(&self) -> Option<&crate::HtmlParseBaseFacts> {
        match &self.context {
            ReportContext::Document { request, .. } => request.base_facts(),
            _ => None,
        }
    }
    pub fn captured_base_facts(&self) -> Option<&crate::HtmlCapturedBaseFacts> {
        match &self.context {
            ReportContext::Fragment(request) => request.base_facts(),
            _ => None,
        }
    }
    pub fn source(&self) -> &crate::HtmlSource {
        match &self.context {
            ReportContext::Document { request, .. } => request.source(),
            ReportContext::Fragment(request) => request.source(),
        }
    }
    pub fn limits(&self) -> &crate::HtmlParseLimits {
        match &self.context {
            ReportContext::Document { request, .. } => request.limits(),
            ReportContext::Fragment(request) => request.limits(),
        }
    }
    pub fn document(&self) -> Option<&crate::HtmlDocument> {
        self.storage.history().document()
    }
    pub fn fragment(&self) -> Option<&crate::HtmlFragment> {
        self.storage.history().fragment()
    }
    pub fn context(&self) -> Option<&crate::HtmlFragmentContext> {
        match &self.context {
            ReportContext::Fragment(request) => Some(request.context()),
            _ => None,
        }
    }
    /// Actual source-created auxiliary parser Document, qualified by this
    /// completed contextual report. Ordinary reports cannot issue this lender.
    pub fn fragment_parser_document(
        &self,
    ) -> Option<crate::construction::fragment_document::FragmentParserDocument<'_>> {
        self.context()?;
        self.storage.fragment_parser_document(self.source())
    }
    pub fn document_state(&self) -> Option<StandaloneDocumentState> {
        match &self.context {
            ReportContext::Document { state, .. } => Some(*state),
            _ => None,
        }
    }
    pub fn requested_scripting_mode(&self) -> crate::HtmlParserScriptingMode {
        match &self.context {
            ReportContext::Document { .. } => crate::HtmlParserScriptingMode::Disabled,
            ReportContext::Fragment(request) => request.requested_scripting_mode(),
        }
    }
    pub fn effective_scripting_mode(&self) -> crate::HtmlParserScriptingMode {
        match &self.context {
            ReportContext::Document { .. } => crate::HtmlParserScriptingMode::Disabled,
            ReportContext::Fragment(request) => request.effective_scripting_mode(),
        }
    }
    /// Request-qualified image policy; this is not a resource load result.
    pub fn image_support(&self) -> crate::HtmlImageSupport {
        match &self.context {
            ReportContext::Document { request, .. } => request.image_support(),
            ReportContext::Fragment(request) => match request
                .context()
                .capture()
                .node(request.context().target_document().subject())
                .expect("checked target document")
                .kind()
            {
                crate::HtmlCapturedNodeKind::Document(facts) => {
                    facts.parser_creation_image_support()
                }
                _ => unreachable!("checked target document"),
            },
        }
    }
    pub fn allow_declarative_shadow_roots(&self) -> bool {
        match &self.context {
            ReportContext::Fragment(request) => request.allow_declarative_shadow_roots(),
            _ => false,
        }
    }
    pub fn usage(&self) -> &ParseUsage {
        &self.usage
    }
    pub(crate) fn history(&self) -> &crate::construction::history::FrozenHistory {
        self.storage.history()
    }
    pub fn metadata(&self) -> MetadataCursor<'_> {
        self.storage.metadata()
    }
    pub fn diagnostics(&self) -> crate::construction::evidence::DiagnosticEvidenceView<'_> {
        self.storage
            .evidence()
            .expect("qualified report evidence")
            .diagnostics()
    }
    pub fn effects(&self) -> Option<crate::construction::context::FrozenFragmentEffects<'_>> {
        match &self.context {
            ReportContext::Fragment(_) => {
                Some(self.storage.effects().expect("qualified report effects"))
            }
            _ => None,
        }
    }
}

impl ImmutableParseReport {
    pub fn associations(&self) -> crate::construction::read::HtmlParseAssociations<'_> {
        crate::construction::read::HtmlParseAssociations::new(self.history())
    }
    pub fn history_view(&self) -> crate::construction::read::HtmlParseHistory<'_> {
        crate::construction::read::HtmlParseHistory::new(self.history())
    }
}

impl std::fmt::Debug for ImmutableParseReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HtmlParseReport")
            .field("source", self.source().identity())
            .field("usage", self.usage())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tree_builder::{
        StandaloneDocumentDriver, head::HeadStep, seal::OrdinaryInitializationOwner,
    };
    use crate::{HtmlDocumentParseRequest, HtmlSource};

    fn ordinary(literal: &str, prepare: bool) -> SealedOrdinaryConstruction {
        ordinary_with_limits(literal, prepare, super::super::tests::limits(100))
    }
    fn ordinary_with_limits(
        literal: &str,
        prepare: bool,
        limits: crate::HtmlParseLimits,
    ) -> SealedOrdinaryConstruction {
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits);
        let mut owner = OrdinaryInitializationOwner::new(request);
        let parts = {
            let mut attempt = owner.begin().unwrap();
            let mut driver = Some(attempt.take_driver().unwrap());
            loop {
                match driver.as_mut().unwrap().step().unwrap() {
                    HeadStep::NeedInput => panic!("complete report fixture cannot starve"),
                    HeadStep::Consumed => {}
                    HeadStep::Complete => break,
                    HeadStep::Boundary(b) => panic!("unexpected boundary {b:?}"),
                }
            }
            let mut completed = StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
            if prepare {
                completed.prepare_native_associations().unwrap();
            }
            completed.freeze_parts().unwrap()
        };
        SealedOrdinaryConstruction::take(&mut Some(owner.take_request().unwrap()), &mut Some(parts))
            .unwrap()
    }

    #[test]
    fn details_publication_keeps_native_attributes_and_skips_private_eligibility() {
        use crate::HtmlNodeKind;
        use crate::construction::report::{MetadataDocument, MetadataStep};
        let literal =
            "<!doctype html><details name=g open>A</details><details name=g open>B</details>";
        let mut sealed = Some(ordinary(literal, true));
        let report = ImmutableParseReport::take_document(&mut sealed).unwrap();
        assert!(sealed.is_none());
        let document = report.document().unwrap();
        let html = document.root().children().last().unwrap();
        let body = document.node(&html).unwrap().children().last().unwrap();
        let details: Vec<_> = document.node(&body).unwrap().children().collect();
        assert_eq!(details.len(), 2);
        for (index, id) in details.iter().enumerate() {
            let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
                panic!("native Details element");
            };
            assert_eq!(
                element.name().local_name().code_units(),
                &[100, 101, 116, 97, 105, 108, 115]
            );
            let attributes: Vec<_> = element
                .attributes()
                .iter()
                .map(|attribute| {
                    (
                        attribute.name().local_name().code_units(),
                        attribute.value().code_units(),
                    )
                })
                .collect();
            assert_eq!(
                attributes,
                if index == 0 {
                    vec![
                        (&[110, 97, 109, 101][..], &[103][..]),
                        (&[111, 112, 101, 110][..], &[][..]),
                    ]
                } else {
                    vec![(&[110, 97, 109, 101][..], &[103][..])]
                }
            );
        }
        let usage = report.usage().clone();
        let mut counts = [0; 2];
        let mut nonce_counts = [0; 2];
        let mut cursor = report.metadata();
        loop {
            match cursor.advance() {
                MetadataStep::Record(record) => {
                    for (index, id) in details.iter().enumerate() {
                        if record.node().id() == *id {
                            // The private Details marker emits no record; ordinary
                            // creation has an independent no-definition record.
                            if let Some(MetadataDocument::Owned(owner)) = record.node_document() {
                                assert_eq!(owner.id(), document.root().id());
                                counts[index] += 1;
                            } else if let Some(nonce) = record.cryptographic_nonce() {
                                assert_eq!(nonce.code_units(), &[]);
                                nonce_counts[index] += 1;
                            } else {
                                let creation = record.creation().unwrap();
                                assert_eq!(creation.0, crate::HtmlCustomElementState::Uncustomized);
                                assert!(creation.1.is_none());
                            }
                        }
                    }
                }
                MetadataStep::Skipped => {}
                MetadataStep::Complete => break,
            }
        }
        assert_eq!(counts, [1, 1]);
        assert_eq!(nonce_counts, [1, 1]);
        assert_eq!(report.usage(), &usage);
        assert_eq!(report.source().as_utf8(), Some(literal));
    }

    fn fragment(literal: &str, shadow: bool) -> SealedFragmentConstruction {
        fragment_with_scripting(literal, shadow, crate::HtmlScriptingStatus::Disabled)
    }
    fn fragment_with_scripting(
        literal: &str,
        shadow: bool,
        scripting: crate::HtmlScriptingStatus,
    ) -> SealedFragmentConstruction {
        use super::super::context::{
            ContextFragmentDriver, ownership::FragmentInitializationOwner,
        };
        use crate::*;
        let mut capture = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let document = capture
            .add_document(
                HtmlDocumentMode::NoQuirks,
                scripting,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        capture
            .establish_document_template_facts(
                &document,
                HtmlDocumentClass::Html,
                HtmlTemplateDocumentRole::Ordinary,
                HtmlCaptureTemplateCacheInput::Absent,
            )
            .unwrap();
        let element = capture
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        capture.establish_root(&element).unwrap();
        capture.establish_no_shadow_root(&element).unwrap();
        let capture = capture.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&element).unwrap()).unwrap();
        let request = HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(literal),
            context,
            super::super::tests::limits(100),
            HtmlParserScriptingMode::Inert,
            shadow,
        )
        .unwrap();
        let mut owner = FragmentInitializationOwner::new(request);
        let mut parts = {
            let mut attempt = owner.begin().unwrap();
            let mut driver = Some(attempt.take_driver().unwrap());
            loop {
                match driver.as_mut().unwrap().step().unwrap() {
                    HeadStep::NeedInput => panic!("complete report fixture cannot starve"),
                    HeadStep::Consumed => {}
                    HeadStep::Complete => break,
                    HeadStep::Boundary(b) => panic!("unexpected fragment boundary {b:?}"),
                }
            }
            let mut completed = ContextFragmentDriver::take_completed(&mut driver).unwrap();
            completed.prepare_native_associations().unwrap();
            Some(completed.freeze_parts().unwrap())
        };
        owner.seal(&mut parts).unwrap()
    }

    #[test]
    fn complete_fragment_report_keeps_capture_effects_source_and_options_after_cancel() {
        use crate::construction::context::{FrozenFragmentEffect, FrozenFragmentHost};
        use crate::*;
        let literal = "<template shadowrootmode=closed shadowrootclonable><p>A</p></template><template shadowrootmode=open><p>B</p></template><p>Z</p>";
        let mut input = Some(fragment(literal, true));
        let cancel = input.as_ref().unwrap().request.cancellation().clone();
        let capture = input
            .as_ref()
            .unwrap()
            .context()
            .capture()
            .identity()
            .clone();
        let report = ImmutableParseReport::take_fragment(&mut input).unwrap();
        assert!(input.is_none());
        let usage = report.usage().clone();
        cancel.cancel();
        assert_eq!(report.source().as_utf8(), Some(literal));
        assert_eq!(report.context().unwrap().capture().identity(), &capture);
        assert_eq!(report.limits(), &super::super::tests::limits(100));
        assert_eq!(
            report.requested_scripting_mode(),
            HtmlParserScriptingMode::Inert
        );
        assert_eq!(
            report.effective_scripting_mode(),
            HtmlParserScriptingMode::Disabled
        );
        assert!(report.allow_declarative_shadow_roots());
        assert!(report.document().is_none());
        assert!(report.document_state().is_none());
        let output = report.fragment().unwrap();
        let children: Vec<_> = output.root().children().collect();
        assert_eq!(children.len(), 2);
        assert!(
            matches!(output.node(&children[0]).unwrap().kind(), HtmlNodeKind::Element(e) if e.name().local_name().code_units() == [116,101,109,112,108,97,116,101])
        );
        assert_eq!(output.node(&children[0]).unwrap().children().len(), 0);
        let effects = report.effects().unwrap();
        // The attachment uses the captured target document directly. Only the
        // second template's ordinary fallback allocates that document's cache.
        assert_eq!(effects.len(), 2);
        assert!(matches!(
            effects.get(1).unwrap(),
            Some(FrozenFragmentEffect::TemplateCache { .. })
        ));
        let Some(FrozenFragmentEffect::AttachShadow {
            host,
            template,
            contents,
            properties,
            source,
            declarative,
            ..
        }) = effects.get(0).unwrap()
        else {
            panic!("actual committed attachment");
        };
        let FrozenFragmentHost::Captured(host) = host else {
            panic!("actual captured host");
        };
        assert_eq!(host, report.context().unwrap().context_element());
        assert_ne!(template.id(), children[0]);
        assert_eq!(properties.mode(), HtmlShadowRootMode::Closed);
        assert!(properties.clonable() && declarative);
        assert_eq!(source.source(), report.source().identity());
        assert_eq!(source.start(), 0);
        let p = contents.children().next().unwrap();
        let text = output.node(&p).unwrap().children().next().unwrap();
        assert!(
            matches!(output.node(&text).unwrap().kind(), HtmlNodeKind::Text(t) if t.code_units() == [65])
        );
        assert!(report.history().associations().unwrap().len() > 0);
        let mut metadata = report.metadata();
        let mut captured_documents = 0;
        loop {
            use crate::construction::report::{MetadataDocument, MetadataStep};
            match metadata.advance() {
                MetadataStep::Record(record) => {
                    if let Some(MetadataDocument::Captured(document)) = record.node_document() {
                        assert!(matches!(document.kind(), HtmlCapturedNodeKind::Document(_)));
                        assert_eq!(
                            &document.node_document(),
                            report.context().unwrap().target_document()
                        );
                        captured_documents += 1;
                    }
                }
                MetadataStep::Skipped => {}
                MetadataStep::Complete => break,
            }
        }
        assert!(captured_documents > 0);
        assert_eq!(report.diagnostics().len(), 0);
        assert_eq!(report.usage(), &usage);
    }

    #[test]
    fn complete_report_consumes_actual_document_source_and_keeps_reads_after_cancellation() {
        use crate::construction::report::MetadataStep;
        let literal = "<!doctype html><form id=f><input value='A&#13;B' type=text><textarea>\nC&amp;D</textarea></form><p>Z";
        let mut input = Some(ordinary(literal, true));
        let source = input.as_ref().unwrap().source().identity().clone();
        let cancel = input.as_ref().unwrap().request.cancellation().clone();
        let report = ImmutableParseReport::take_document(&mut input).unwrap();
        assert!(input.is_none());
        let usage = report.usage().clone();
        cancel.cancel();
        assert_eq!(report.source().identity(), &source);
        assert_eq!(report.source().as_utf8(), Some(literal));
        assert!(report.fragment().is_none());
        assert!(report.context().is_none());
        assert!(report.effects().is_none());
        assert_eq!(
            report.document().unwrap().mode(),
            crate::HtmlDocumentMode::NoQuirks
        );
        assert_eq!(
            super::super::tests::outline(report.document().unwrap()),
            super::super::tests::expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "form"),
                (3, "input"),
                (3, "textarea"),
                (4, "#C&D"),
                (2, "p"),
                (3, "#Z")
            ])
        );
        assert!(report.history().associations().unwrap().len() > 0);
        let mut cursor = report.metadata();
        let mut values = Vec::new();
        let mut input_source = false;
        loop {
            match cursor.advance() {
                MetadataStep::Record(record) => {
                    if let Some((state, raw)) = record.input() {
                        assert_eq!(state.flags.input_type(), surgeist_html::HtmlInputType::Text);
                        assert!(!state.flags.dirty_value());
                        assert_eq!(raw.code_units(), &[65, 66]);
                        values.push(record.node().id());
                    }
                    if let Some((state, raw)) = record.textarea() {
                        assert!(!state.flags.dirty_value());
                        assert_eq!(raw.code_units(), &[67, 38, 68]);
                        values.push(record.node().id());
                    }
                    if let Some((token, attribute)) = record.input_source() {
                        assert_eq!(token.source(), &source);
                        let attribute = attribute.expect("actual rewritten lexical value");
                        assert_eq!(attribute.source(), &source);
                        assert!(
                            token.start() <= attribute.start() && attribute.end() <= token.end()
                        );
                        input_source = true;
                    }
                }
                MetadataStep::Skipped => {}
                MetadataStep::Complete => break,
            }
        }
        assert_eq!(values.len(), 2);
        assert_ne!(values[0], values[1]);
        assert!(input_source);
        assert_eq!(report.usage(), &usage);
    }

    #[test]
    fn native_only_seal_cannot_publish_complete_report_and_keeps_original_owners() {
        let mut input = Some(ordinary("<!doctype html><p>A", false));
        let source = input.as_ref().unwrap().source().identity().clone();
        assert!(matches!(
            ImmutableParseReport::take_document(&mut input),
            Err(SessionError::WrongToken)
        ));
        let kept = input.as_mut().unwrap();
        assert_eq!(kept.source().identity(), &source);
        assert!(kept.parts.control.stop().is_none());
        assert!(kept.document().is_some());
        kept.release().unwrap();
        super::super::tests::assert_released(&kept.parts.control);
    }

    #[test]
    fn metadata_inventory_borrows_actual_family_and_document_facts_one_slot_at_a_time() {
        use crate::construction::report::{MetadataDocument, MetadataStep};
        let literal = "<!doctype html><script>A</script><template><p>T</p></template><form id=f><output>A<b>B</b>C</output><select><button><selectedcontent></selectedcontent></button><option selected>X</option><option>Y</option></select></form><svg><script/></svg><x-a is=saved></x-a>";
        let mut input = Some(ordinary(literal, true));
        let report = ImmutableParseReport::take_document(&mut input).unwrap();
        assert!(report.document_state().is_some());
        assert_eq!(
            report.requested_scripting_mode(),
            crate::HtmlParserScriptingMode::Disabled
        );
        assert!(!report.allow_declarative_shadow_roots());
        let mut cursor = report.metadata();
        let mut counts = [0; 7];
        let mut hosted = false;
        let mut saved_creation = false;
        loop {
            match cursor.advance() {
                MetadataStep::Record(record) => {
                    if let Some(output) = record.output() {
                        assert_eq!(
                            output.default_value_override,
                            crate::construction::OutputDefaultValueOverride::Null
                        );
                        counts[0] += 1;
                    }
                    if let Some((form, owner)) = record.form() {
                        assert!(form.parser_inserted);
                        assert!(
                            matches!(owner.unwrap().kind(), crate::HtmlNodeKind::Element(e) if e.name().local_name().code_units() == [102,111,114,109])
                        );
                        counts[1] += 1;
                    }
                    if let Some(script) = record.script() {
                        assert!(script.flags.already_started() && !script.flags.force_async());
                        let (parser, preparation) = record.script_documents().unwrap();
                        assert!(
                            matches!(parser.unwrap(), MetadataDocument::Owned(n) if matches!(n.kind(), crate::HtmlNodeKind::Document(_)))
                        );
                        assert!(
                            matches!(preparation.unwrap(), MetadataDocument::Owned(n) if matches!(n.kind(), crate::HtmlNodeKind::Document(_)))
                        );
                        counts[2] += 1;
                    }
                    if let Some(script) = record.svg_script() {
                        assert!(script.flags.already_processed());
                        counts[3] += 1;
                    }
                    if let Some(select) = record.select() {
                        assert!(!select.flags.user_validity());
                        counts[4] += 1;
                    }
                    if let Some(option) = record.option() {
                        assert!(!option.flags.dirtiness());
                        counts[5] += 1;
                    }
                    if let Some(selected) = record.selectedcontent() {
                        assert!(!selected.disabled);
                        counts[6] += 1;
                    }
                    if let Some(template) = record.template() {
                        assert!(
                            template.target.is_none()
                                && template.start.is_none()
                                && template.end.is_none()
                        );
                        assert_eq!(template.contents.children().len(), 1);
                        assert_eq!(record.node().children().len(), 0);
                        hosted = true;
                    }
                    if let Some((state, saved)) = record.creation() {
                        if let Some(saved) = saved {
                            assert_eq!(state, crate::HtmlCustomElementState::Undefined);
                            assert_eq!(saved.code_units(), &[115, 97, 118, 101, 100]);
                            saved_creation = true;
                        } else {
                            assert_eq!(state, crate::HtmlCustomElementState::Uncustomized);
                        }
                    }
                    if let Some(MetadataDocument::Owned(document)) = record.node_document() {
                        assert!(matches!(document.kind(), crate::HtmlNodeKind::Document(_)));
                    }
                }
                MetadataStep::Skipped => {}
                MetadataStep::Complete => break,
            }
        }
        // Output, select and the real nested button are independently listed.
        assert_eq!(counts, [1, 3, 1, 1, 1, 2, 1]);
        assert!(hosted);
        assert!(saved_creation);
    }

    #[test]
    fn captured_creation_saved_is_is_borrowed_without_fabricating_ordinary_facts() {
        use crate::construction::report::{MetadataDocument, MetadataStep};
        let mut input = Some(fragment("<x-a is=saved></x-a><script>A</script>", false));
        let report = ImmutableParseReport::take_fragment(&mut input).unwrap();
        let mut cursor = report.metadata();
        let mut saved = false;
        let mut script = false;
        loop {
            match cursor.advance() {
                MetadataStep::Record(record) => {
                    if let Some((state, Some(value))) = record.creation() {
                        assert_eq!(state, crate::HtmlCustomElementState::Undefined);
                        assert_eq!(value.code_units(), &[115, 97, 118, 101, 100]);
                        saved = true;
                    }
                    if let Some(state) = record.script() {
                        // Effective Disabled prepares the disconnected fragment
                        // script: parser document is cleared before that return.
                        assert!(!state.flags.already_started() && state.flags.force_async());
                        let (parser, preparation) = record.script_documents().unwrap();
                        assert!(parser.is_none() && preparation.is_none());
                        for document in [parser, preparation].into_iter().flatten() {
                            match document {
                                MetadataDocument::Owned(node) => {
                                    assert!(matches!(node.kind(), crate::HtmlNodeKind::Document(_)))
                                }
                                MetadataDocument::Captured(node) => assert!(matches!(
                                    node.kind(),
                                    crate::HtmlCapturedNodeKind::Document(_)
                                )),
                            }
                        }
                        script = true;
                    }
                }
                MetadataStep::Skipped => {}
                MetadataStep::Complete => break,
            }
        }
        assert!(saved && script);
    }

    fn first_text(document: &crate::HtmlDocument) -> &crate::DocumentString {
        let html = document
            .root()
            .children()
            .find(|node| {
                matches!(
                    document.node(node).unwrap().kind(),
                    crate::HtmlNodeKind::Element(_)
                )
            })
            .unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let p = document.node(&body).unwrap().children().next().unwrap();
        let text = document.node(&p).unwrap().children().next().unwrap();
        let crate::HtmlNodeKind::Text(value) = document.node(&text).unwrap().kind() else {
            panic!("literal text");
        };
        value
    }
    #[test]
    fn report_alias_refusal_retains_exact_owner_then_publication_shares_real_backing() {
        let mut input = Some(ordinary("<!doctype html><p>A", true));
        let alias = first_text(input.as_ref().unwrap().document().unwrap()).clone();
        let pointer = alias.code_units().as_ptr();
        assert!(matches!(
            ImmutableParseReport::take_document(&mut input),
            Err(SessionError::AliasedPayload)
        ));
        assert!(input.as_ref().unwrap().parts.arena.is_some());
        assert!(input.as_ref().unwrap().parts.control.stop().is_none());
        assert_eq!(
            first_text(input.as_ref().unwrap().document().unwrap())
                .code_units()
                .as_ptr(),
            pointer
        );
        drop(alias);
        let report = ImmutableParseReport::take_document(&mut input).unwrap();
        let snapshot = report.usage().clone();
        let alias = first_text(report.document().unwrap()).clone();
        let weak = std::sync::Arc::downgrade(alias.shared_code_units());
        assert_eq!(alias.code_units().as_ptr(), pointer);
        drop(report);
        assert_eq!(alias.code_units(), &[65]);
        assert_eq!(weak.strong_count(), 1);
        assert!(
            snapshot
                .resource(crate::HtmlParseResource::ManagedUtf16Units)
                .current
                > 0
        );
        drop(alias);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn cancelled_report_transfer_keeps_source_first_stop_and_safe_cleanup_owner() {
        use crate::parser::control::ParseStop;
        let mut input = Some(ordinary("<!doctype html><p>A", true));
        let source = input.as_ref().unwrap().source().identity().clone();
        input.as_ref().unwrap().request.cancellation().cancel();
        assert!(matches!(
            ImmutableParseReport::take_document(&mut input),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        let sealed = input.as_mut().unwrap();
        assert_eq!(sealed.source().identity(), &source);
        assert_eq!(sealed.parts.control.stop(), Some(ParseStop::Cancelled));
        let first = sealed.parts.control.first_stop_usage().unwrap().clone();
        sealed.parts.cleanup_failure().unwrap();
        assert!(sealed.parts.arena.is_none());
        assert_eq!(sealed.parts.control.first_stop_usage(), Some(&first));
        assert!(sealed.parts.diagnostics().is_ok());
        sealed.parts.release().unwrap();
        super::super::tests::assert_released(&sealed.parts.control);
        assert_eq!(sealed.parts.control.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn finalization_capacity_refusal_keeps_terminal_owner_and_exact_source_for_cleanup() {
        use crate::{HtmlParseResource, parser::control::ParseStop};
        let mut input = Some(ordinary(
            "<!doctype html><p class=k>A</p><p class=k>A</p>",
            true,
        ));
        let sealed = input.as_mut().unwrap();
        let source = sealed.source().identity().clone();
        {
            let c = &mut sealed.parts.control;
            crate::construction::read::prepare_public_readers(&sealed.parts.arena, c).unwrap();
            let prepared =
                PreparedReportStorage::prepare(&mut sealed.parts.arena, c, None, true).unwrap();
            // The same original controller has completed all strict alias/history
            // qualification. Deny new finalization custody, without re-forming it.
            c.test_limit(HtmlParseResource::ManagedCapacityBytes, 0);
            assert!(matches!(
                prepared.share_values(c),
                Err(SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedCapacityBytes,
                    limit: 0,
                    ..
                }))
            ));
        }
        assert!(sealed.parts.arena.is_some());
        assert!(
            sealed.document().is_none(),
            "partial terminal construction is not a published view"
        );
        assert_eq!(sealed.source().identity(), &source);
        let first = sealed.parts.control.first_stop_usage().unwrap().clone();
        sealed.parts.cleanup_failure().unwrap();
        assert!(sealed.parts.arena.is_none());
        assert!(sealed.parts.diagnostics().is_ok());
        assert_eq!(sealed.parts.control.first_stop_usage(), Some(&first));
        sealed.parts.release().unwrap();
        super::super::tests::assert_released(&sealed.parts.control);
    }

    #[test]
    fn cancellation_after_value_finalization_retains_complete_original_owner_without_report() {
        use crate::parser::control::ParseStop;
        let mut input = Some(ordinary(
            "<!doctype html><p class=k>A</p><p class=k>A</p>",
            true,
        ));
        let sealed = input.as_mut().unwrap();
        let source = sealed.source().identity().clone();
        {
            let c = &mut sealed.parts.control;
            crate::construction::read::prepare_public_readers(&sealed.parts.arena, c).unwrap();
            let prepared =
                PreparedReportStorage::prepare(&mut sealed.parts.arena, c, None, true).unwrap();
            prepared.share_values(c).unwrap();
        }
        sealed.request.cancellation().cancel();
        assert!(matches!(
            ImmutableParseReport::take_document(&mut input),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        let kept = input.as_mut().unwrap();
        assert_eq!(kept.source().identity(), &source);
        assert!(kept.document().is_some());
        let first = kept.parts.control.first_stop_usage().unwrap().clone();
        kept.parts.cleanup_failure().unwrap();
        assert!(kept.parts.arena.is_none());
        assert_eq!(kept.parts.control.first_stop_usage(), Some(&first));
        kept.parts.release().unwrap();
        super::super::tests::assert_released(&kept.parts.control);
        assert_eq!(kept.parts.control.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn populated_pool_replacement_refusal_and_cancel_keep_partial_custody_cleanup_owned() {
        use crate::{
            HtmlParseResource,
            parser::control::{FinalizationCut, ParseStop},
        };
        for kind in [FinalizationCut::Work, FinalizationCut::Cancellation] {
            // Missing doctype gives this same actual owner one independent
            // recovery diagnostic whose original source must survive cleanup.
            let literal = "<p id=a class=k>A</p><p id=b class=k>B</p>";
            let mut input = Some(ordinary(literal, true));
            let sealed = input.as_mut().unwrap();
            let source = sealed.source().identity().clone();
            let ids = {
                let document = sealed.document().unwrap();
                let html = document.root().children().last().unwrap();
                let body = document.node(&html).unwrap().children().last().unwrap();
                document.node(&body).unwrap().children().collect::<Vec<_>>()
            };
            assert_eq!(ids.len(), 2);
            sealed.parts.arena.as_ref().unwrap().arm_finalization_cut(
                &ids[0],
                &ids[1],
                kind,
                &mut sealed.parts.control,
            );
            let result = ImmutableParseReport::take_document(&mut input);
            let kept = input
                .as_mut()
                .expect("failed terminal publication retains original seal");
            let observed = kept
                .parts
                .control
                .finalization_observation()
                .expect("actual selected replacement boundary reached");
            assert!(observed.string_pool_bytes > 0 && observed.attribute_pool_bytes > 0);
            assert!(observed.string_insertions > 0 && observed.attribute_insertions > 0);
            assert!(observed.earlier_promoted);
            assert!(
                observed.selected_bank_bytes
                    >= size_of::<Vec<crate::Attribute>>() + 2 * size_of::<crate::Attribute>()
            );
            // Each of the two actual attributes has one local-name and one
            // value output receipt; absent namespace/prefix adds no fake row.
            assert_eq!(observed.selected_components, 4);
            assert!(observed.selected_tickets_finalized);
            match kind {
                FinalizationCut::Work => assert!(
                    matches!(result, Err(SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits, limit, current, requested_delta: 1,
                })) if limit == observed.work && current == observed.work)
                ),
                FinalizationCut::Cancellation => assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::Cancelled))
                )),
                FinalizationCut::Panic => unreachable!("work/cancellation fixture"),
            }
            assert_eq!(kept.source().identity(), &source);
            assert_eq!(kept.source().as_utf8(), Some(literal));
            assert!(kept.parts.arena.is_some());
            assert!(
                kept.document().is_none(),
                "partial promotion never publishes a report view"
            );
            let first = kept.parts.control.stop().unwrap();
            let snapshot = kept.parts.control.first_stop_usage().unwrap().clone();
            assert!(
                snapshot.actual_capacity_bytes()
                    > kept.parts.control.usage().actual_capacity_bytes(),
                "live provisional/pool custody was included at first stop and released during unwind"
            );
            let spent = kept
                .parts
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits);
            assert!(
                matches!(ImmutableParseReport::take_document(&mut input), Err(SessionError::Stop(stop)) if stop == first)
            );
            let kept = input.as_mut().unwrap();
            assert_eq!(
                kept.parts
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits),
                spent
            );
            assert_eq!(kept.source().identity(), &source);
            kept.parts.cleanup_failure().unwrap();
            assert!(kept.parts.arena.is_none());
            let diagnostics = kept.parts.diagnostics().unwrap();
            assert_eq!(diagnostics.len(), 1);
            let (diagnostic, location) = diagnostics.retained_record(0).unwrap();
            assert_eq!(
                diagnostic.kind(),
                crate::HtmlParseDiagnosticKind::Prologue(
                    crate::HtmlPrologueDiagnostic::MissingDoctype
                )
            );
            assert_eq!(location.source_range().source(), &source);
            assert_eq!(
                (
                    location.source_range().start(),
                    location.source_range().end()
                ),
                // Initial-mode recovery identifies the complete first token.
                (0, "<p id=a class=k>".len())
            );
            assert_eq!(kept.parts.control.first_stop_usage(), Some(&snapshot));
            kept.parts.release().unwrap();
            super::super::tests::assert_released(&kept.parts.control);
            assert_eq!(kept.parts.control.stop(), Some(first));
        }
    }

    #[test]
    fn inert_fragment_report_preserves_actual_parser_script_prefix_and_diagnostics() {
        use crate::construction::report::{MetadataDocument, MetadataStep};
        let mut input = Some(fragment_with_scripting(
            "<!doctype html><script>A</script>",
            false,
            crate::HtmlScriptingStatus::Enabled,
        ));
        let cancel = input.as_ref().unwrap().request.cancellation().clone();
        let report = ImmutableParseReport::take_fragment(&mut input).unwrap();
        assert_eq!(
            report.effective_scripting_mode(),
            crate::HtmlParserScriptingMode::Inert
        );
        cancel.cancel();
        assert!(!report.diagnostics().is_empty());
        let mut cursor = report.metadata();
        let mut found = false;
        loop {
            match cursor.advance() {
                MetadataStep::Record(record) => {
                    if let Some(script) = record.script() {
                        assert!(script.flags.already_started() && !script.flags.force_async());
                        assert!(script.script_type.is_none());
                        let (parser, preparation) = record.script_documents().unwrap();
                        assert!(
                            matches!(parser.unwrap(), MetadataDocument::Owned(node) if matches!(node.kind(), crate::HtmlNodeKind::Document(_)))
                        );
                        assert!(preparation.is_none());
                        found = true;
                    }
                }
                MetadataStep::Skipped => {}
                MetadataStep::Complete => break,
            }
        }
        assert!(found);
    }

    #[test]
    fn original_initialization_failures_remain_source_owning_without_a_report_or_fabricated_binding()
     {
        use crate::{HtmlParseResource, parser::control::ParseStop};
        for (source_limit, work, cancelled) in [
            (0, 100_000_000, false),
            (100_000, 0, false),
            (100_000, 100_000_000, true),
        ] {
            let limits = super::super::tests::limits(100)
                .into_builder()
                .source_units(source_limit)
                .work_credits(work)
                .finish()
                .unwrap();
            let original = HtmlDocumentParseRequest::offline(HtmlSource::utf8("A"), limits);
            let id = original.source().identity().clone();
            if cancelled {
                original.cancellation().cancel();
            }
            let mut owner = OrdinaryInitializationOwner::new(original);
            {
                let mut attempt = owner.begin().unwrap();
                assert!(attempt.take_driver().is_err());
                attempt.retain_failure().unwrap();
            }
            assert_eq!(owner.source().unwrap().identity(), &id);
            assert_eq!(owner.source_admitted(), Some(false));
            let Some(super::super::entry::InitializationFailure::Stopped(stop)) = owner.failure()
            else {
                panic!("actual stopped initialization");
            };
            if cancelled {
                assert_eq!(stop, ParseStop::Cancelled);
            } else {
                assert!(
                    matches!(stop, ParseStop::LimitExceeded { resource, .. } if resource == if work == 0 { HtmlParseResource::WorkCredits } else { HtmlParseResource::SourceUnits })
                );
            }
            assert!(owner.diagnostics().is_none());
            owner.release_failure().unwrap();
            assert_eq!(
                owner
                    .retained_usage()
                    .unwrap()
                    .resource(HtmlParseResource::SourceUnits)
                    .current,
                0
            );
            for resource in [
                HtmlParseResource::BufferedUtf16Units,
                HtmlParseResource::TokenAttributes,
                HtmlParseResource::AuxiliaryEntries,
                HtmlParseResource::ManagedEdges,
                HtmlParseResource::ManagedAttributes,
                HtmlParseResource::ManagedUtf16Units,
                HtmlParseResource::SourceMapEntries,
                HtmlParseResource::ManagedCapacityBytes,
                HtmlParseResource::PeakTransientBytes,
            ] {
                assert_eq!(
                    owner.retained_usage().unwrap().resource(resource).current,
                    0,
                    "{resource:?}"
                );
            }
        }
    }

    #[test]
    fn last_fixed_transfer_exact_and_one_short_keep_actual_seal_on_denied_write() {
        use crate::{
            HtmlParseResource,
            parser::control::{ParseStop, WorkCategory},
        };
        let literal = "<!doctype html><p>A";
        let mut measured = Some(ordinary(literal, true));
        let measured = ImmutableParseReport::take_document(&mut measured).unwrap();
        let exact = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let last_bytes = size_of::<ImmutableParseReport>()
            + size_of::<Option<SealedOrdinaryConstruction>>()
            + size_of::<SealedOrdinaryConstruction>()
            + size_of::<ReportContext>()
            + size_of::<ParseUsage>();
        // Final publication copies these compiled receivers then admits five
        // writes: request, state, storage, snapshot and exclusive input clear.
        // One-short denies that entire last write admission before any take.
        for budget in [exact - 1, exact] {
            let limits = super::super::tests::limits(100)
                .into_builder()
                .work_credits(budget)
                .finish()
                .unwrap();
            let mut input = Some(ordinary_with_limits(literal, true, limits));
            let before_copy = input
                .as_ref()
                .unwrap()
                .parts
                .control
                .usage()
                .work(WorkCategory::CopyByte);
            let result = ImmutableParseReport::take_document(&mut input);
            if budget == exact {
                let report = result.unwrap();
                assert!(input.is_none());
                assert_eq!(
                    report
                        .usage()
                        .resource(HtmlParseResource::WorkCredits)
                        .current,
                    exact
                );
                assert!(
                    report.usage().work(WorkCategory::CopyByte) >= before_copy + last_bytes as u64
                );
            } else {
                assert!(
                    matches!(result, Err(SessionError::Stop(ParseStop::LimitExceeded { resource: HtmlParseResource::WorkCredits, current, requested_delta: 5, .. })) if current == exact - 5)
                );
                let kept = input.as_mut().unwrap();
                assert!(kept.parts.arena.is_some());
                assert_eq!(first_text(kept.document().unwrap()).code_units(), &[65]);
                let first = kept.parts.control.stop().unwrap();
                kept.parts.cleanup_failure().unwrap();
                assert_eq!(kept.parts.control.stop(), Some(first));
                assert!(kept.parts.diagnostics().is_ok());
                kept.parts.release().unwrap();
                super::super::tests::assert_released(&kept.parts.control);
            }
        }
    }

    #[test]
    fn foreign_original_request_refuses_before_storage_transfer_and_preserves_both_owners() {
        use crate::parser::control::{Invariant, ParseStop};
        let mut first = Some(ordinary("<!doctype html><p>A", true));
        let mut second = Some(ordinary("<!doctype html><p>B", true));
        std::mem::swap(
            &mut first.as_mut().unwrap().request,
            &mut second.as_mut().unwrap().request,
        );
        assert!(matches!(
            ImmutableParseReport::take_document(&mut first),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                Invariant::ForeignSource
            )))
        ));
        assert!(first.as_ref().unwrap().parts.arena.is_some());
        assert!(second.as_ref().unwrap().parts.control.stop().is_none());
        // Test-fixture restoration returns each actual source to its original
        // retained cleanup owner; it never clears the real canonical stop.
        std::mem::swap(
            &mut first.as_mut().unwrap().request,
            &mut second.as_mut().unwrap().request,
        );
        let kept = first.as_mut().unwrap();
        kept.parts.cleanup_failure().unwrap();
        kept.parts.release().unwrap();
        super::super::tests::assert_released(&kept.parts.control);
        assert_eq!(
            kept.parts.control.stop(),
            Some(ParseStop::InternalInvariant(Invariant::ForeignSource))
        );
        let report = ImmutableParseReport::take_document(&mut second).unwrap();
        assert_eq!(first_text(report.document().unwrap()).code_units(), &[66]);
    }

    #[test]
    fn stopped_alias_cleanup_refusal_keeps_real_arena_safe_diagnostics_and_request_for_retry() {
        use crate::parser::control::ParseStop;
        use crate::parser::tree_builder::{KernelError, StandaloneError, head::HeadError};
        let mut input = Some(ordinary("<!doctype html><p>A", true));
        let source = input.as_ref().unwrap().source().identity().clone();
        let alias = first_text(input.as_ref().unwrap().document().unwrap()).clone();
        input.as_ref().unwrap().request.cancellation().cancel();
        assert!(matches!(
            ImmutableParseReport::take_document(&mut input),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        let kept = input.as_mut().unwrap();
        let first = kept.parts.control.first_stop_usage().unwrap().clone();
        assert!(matches!(
            kept.parts.cleanup_failure(),
            Err(StandaloneError::Head(HeadError::Kernel(
                KernelError::Session(SessionError::AliasedPayload)
            )))
        ));
        assert!(kept.parts.arena.is_some());
        // Existing stopped partitioning moves only committed diagnostics into
        // the failure slot before native disposal encounters the string alias.
        assert!(kept.parts.diagnostics().is_ok());
        assert_eq!(kept.source().identity(), &source);
        assert_eq!(kept.parts.control.first_stop_usage(), Some(&first));
        drop(alias);
        kept.parts.cleanup_failure().unwrap();
        assert!(kept.parts.arena.is_none());
        assert!(kept.parts.diagnostics().is_ok());
        kept.parts.release().unwrap();
        super::super::tests::assert_released(&kept.parts.control);
        assert_eq!(kept.parts.control.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn derived_pi_map_document_cache_and_shadow_flags_are_fixed_original_lenders() {
        use crate::construction::report::MetadataStep;
        let literal = "<div><template shadowrootmode=open shadowrootclonable><p>S</p></template><?marker name='x&amp;y'?><template>T</template></div>";
        let mut input = Some(fragment(literal, true));
        let report = ImmutableParseReport::take_fragment(&mut input).unwrap();
        let mut cursor = report.metadata();
        let mut pi = false;
        let mut shadow = false;
        let mut inert = false;
        loop {
            match cursor.advance() {
                MetadataStep::Record(record) => {
                    if let Some(count) = record.pi_attribute_count() {
                        assert_eq!(count, 1);
                        assert!(matches!(
                            record.node().kind(),
                            crate::HtmlNodeKind::ProcessingInstruction(_)
                        ));
                        let (name, value) = record.pi_attribute(0).unwrap();
                        assert_eq!(name.code_units(), &[110, 97, 109, 101]);
                        assert_eq!(value.code_units(), &[120, 38, 121]);
                        assert_eq!(
                            record.pi_attribute(0).unwrap().1.code_units().as_ptr(),
                            value.code_units().as_ptr()
                        );
                        assert!(record.pi_attribute(1).is_none());
                        pi = true;
                    }
                    if let Some((keep_null, declarative, available)) = record.shadow() {
                        assert!(!keep_null && declarative && available);
                        assert!(matches!(
                            record.node().kind(),
                            crate::HtmlNodeKind::Fragment
                        ));
                        assert_eq!(record.node().children().len(), 1);
                        shadow = true;
                    }
                    if let Some((class, is_inert, cache)) = record.document_facts() {
                        assert_eq!(class, crate::HtmlDocumentClass::Html);
                        assert!(matches!(
                            record.node().kind(),
                            crate::HtmlNodeKind::Document(_)
                        ));
                        if is_inert {
                            assert!(cache.is_none());
                            inert = true;
                        }
                    }
                }
                MetadataStep::Skipped => {}
                MetadataStep::Complete => break,
            }
        }
        assert!(pi && shadow && inert);
    }
    #[test]
    fn option_cache_lends_actual_select_identity_and_applicable_null_after_cancel() {
        use crate::construction::report::MetadataStep;
        let literal = "<!doctype html><select id=s><option selected>A</option></select><option>B</option><p>Z</p>";
        let mut input = Some(ordinary(literal, true));
        let cancel = input.as_ref().unwrap().request.cancellation().clone();
        let report = ImmutableParseReport::take_document(&mut input).unwrap();
        assert!(input.is_none());
        assert_eq!(
            super::super::tests::outline(report.document().unwrap()),
            super::super::tests::expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "select"),
                (3, "option"),
                (4, "#A"),
                (2, "option"),
                (3, "#B"),
                (2, "p"),
                (3, "#Z")
            ])
        );
        let usage = report.usage().clone();
        cancel.cancel();
        let mut cursor = report.metadata();
        let mut select = None;
        let mut cached = None;
        let mut options = Vec::new();
        let mut inapplicable = 0;
        loop {
            match cursor.advance() {
                MetadataStep::Record(record) => {
                    if record.select().is_some() {
                        select = Some(record.node().id());
                        assert!(record.option_nearest_select().is_none());
                    }
                    if record.option().is_some() {
                        let owner = record.option_nearest_select().expect("applicable option");
                        if let Some(owner) = owner {
                            cached = Some(owner.id());
                            assert!(matches!(owner.kind(), crate::HtmlNodeKind::Element(e)
                                if e.name().local_name().code_units() == [115,101,108,101,99,116]));
                        }
                        options.push((record.node().id(), owner.is_some()));
                    } else {
                        assert!(record.option_nearest_select().is_none());
                        inapplicable += 1;
                    }
                }
                MetadataStep::Skipped => {}
                MetadataStep::Complete => break,
            }
        }
        assert_eq!(cached, select);
        assert_eq!(options.len(), 2);
        assert!(options[0].1);
        assert!(!options[1].1);
        assert_ne!(options[0].0, options[1].0);
        assert!(inapplicable > 0);
        assert_eq!(report.usage(), &usage);
    }

    #[test]
    fn cached_select_qualified_publication_exact_and_one_short_retains_original_seal() {
        use crate::{HtmlParseResource, parser::control::ParseStop};
        let literal = "<!doctype html><select><option>A</option></select>";
        let mut measured = Some(ordinary(literal, true));
        let report = ImmutableParseReport::take_document(&mut measured).unwrap();
        let exact = report
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        // Actual complete execution precedes this newly qualified cache transfer.
        // The unchanged final five-write admission is the independent stop target.
        for budget in [exact - 1, exact] {
            let limits = super::super::tests::limits(100)
                .into_builder()
                .work_credits(budget)
                .finish()
                .unwrap();
            let mut input = Some(ordinary_with_limits(literal, true, limits));
            let source = input.as_ref().unwrap().source().identity().clone();
            let result = ImmutableParseReport::take_document(&mut input);
            if budget == exact {
                let report = result.unwrap();
                assert!(input.is_none());
                assert_eq!(report.source().identity(), &source);
                let mut cursor = report.metadata();
                let mut nonnull = 0;
                loop {
                    match cursor.advance() {
                        crate::construction::report::MetadataStep::Record(record) => {
                            if record
                                .option_nearest_select()
                                .is_some_and(|cache| cache.is_some())
                            {
                                nonnull += 1;
                            }
                        }
                        crate::construction::report::MetadataStep::Skipped => {}
                        crate::construction::report::MetadataStep::Complete => {
                            break;
                        }
                    }
                }
                assert_eq!(nonnull, 1);
            } else {
                assert!(
                    matches!(result, Err(SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits, current, requested_delta: 5, ..
                })) if current == exact - 5)
                );
                let kept = input.as_mut().unwrap();
                assert_eq!(kept.source().identity(), &source);
                assert!(kept.parts.arena.is_some());
                assert_eq!(
                    super::super::tests::outline(kept.document().unwrap()),
                    super::super::tests::expected_outline(&[
                        (0, "html"),
                        (1, "head"),
                        (1, "body"),
                        (2, "select"),
                        (3, "option"),
                        (4, "#A")
                    ])
                );
                let first = kept.parts.control.stop().unwrap();
                assert!(matches!(ImmutableParseReport::take_document(&mut input),
                    Err(SessionError::Stop(stop)) if stop == first));
                let kept = input.as_mut().unwrap();
                kept.parts.cleanup_failure().unwrap();
                kept.parts.release().unwrap();
                assert_eq!(kept.parts.control.stop(), Some(first));
                super::super::tests::assert_released(&kept.parts.control);
            }
        }
    }
    #[test]
    fn detached_fragment_option_lends_applicable_null_cache_without_ancestry_work() {
        use crate::construction::report::MetadataStep;
        let mut input = Some(fragment("<option>B</option>", false));
        let cancel = input.as_ref().unwrap().request.cancellation().clone();
        let report = ImmutableParseReport::take_fragment(&mut input).unwrap();
        let frozen = report.fragment().unwrap();
        let option = frozen.root().children().next().unwrap();
        let node = frozen.node(&option).unwrap();
        assert!(matches!(node.kind(), crate::HtmlNodeKind::Element(e)
            if e.name().local_name().code_units() == [111,112,116,105,111,110]));
        let text = node.children().next().unwrap();
        assert!(matches!(frozen.node(&text).unwrap().kind(),
            crate::HtmlNodeKind::Text(value) if value.code_units() == [66]));
        let usage = report.usage().clone();
        cancel.cancel();
        let mut cursor = report.metadata();
        let mut null = 0;
        loop {
            match cursor.advance() {
                MetadataStep::Record(record) => {
                    if let Some(cache) = record.option_nearest_select() {
                        assert!(cache.is_none());
                        assert_eq!(record.node().id(), option);
                        null += 1;
                    }
                }
                MetadataStep::Skipped => {}
                MetadataStep::Complete => break,
            }
        }
        assert_eq!(null, 1);
        assert_eq!(report.usage(), &usage);
    }
    #[test]
    fn populated_pool_unwind_keeps_terminal_partial_data_and_exact_cleanup_owner() {
        use crate::parser::control::FinalizationCut;
        let literal = "<p id=a class=k>A</p><p id=b class=k>B</p>";
        let mut input = Some(ordinary(literal, true));
        let sealed = input.as_mut().unwrap();
        let source = sealed.source().identity().clone();
        let ids = {
            let document = sealed.document().unwrap();
            let html = document.root().children().last().unwrap();
            let body = document.node(&html).unwrap().children().last().unwrap();
            document.node(&body).unwrap().children().collect::<Vec<_>>()
        };
        sealed.parts.arena.as_ref().unwrap().arm_finalization_cut(
            &ids[0],
            &ids[1],
            FinalizationCut::Panic,
            &mut sealed.parts.control,
        );
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ImmutableParseReport::take_document(&mut input)
        }))
        .unwrap_err();
        assert_eq!(
            panic.downcast_ref::<&str>(),
            Some(&"terminal value-sharing replacement cut")
        );
        let kept = input.as_mut().unwrap();
        let seen = kept.parts.control.finalization_observation().unwrap();
        assert!(seen.string_pool_bytes > 0 && seen.attribute_pool_bytes > 0);
        assert!(seen.earlier_promoted && seen.selected_tickets_finalized);
        assert_eq!(seen.selected_components, 4);
        assert!(seen.selected_bank_bytes > 0);
        assert_eq!(kept.parts.control.stop(), None);
        assert!(kept.parts.control.first_stop_usage().is_none());
        assert_eq!(kept.source().identity(), &source);
        assert_eq!(kept.source().as_utf8(), Some(literal));
        assert!(kept.document().is_none());
        let work = kept
            .parts
            .control
            .usage()
            .resource(crate::HtmlParseResource::WorkCredits);
        assert!(matches!(
            ImmutableParseReport::take_document(&mut input),
            Err(SessionError::Consumed)
        ));
        let kept = input.as_mut().unwrap();
        assert_eq!(
            kept.parts
                .control
                .usage()
                .resource(crate::HtmlParseResource::WorkCredits),
            work
        );
        // An unwind does not manufacture a parser stop. Existing explicit
        // same-controller disposal reclaims the truthful terminal owner.
        kept.parts.release().unwrap();
        super::super::tests::assert_released(&kept.parts.control);
        assert_eq!(kept.parts.control.stop(), None);
    }

    #[test]
    fn paired_roots_remain_guarded_through_receiver_refusal_cancellation_and_unwind() {
        use crate::parser::control::{FinalizationCut, FinalizationTransferStage, ParseStop};
        for stage in [
            FinalizationTransferStage::FirstPrepared,
            FinalizationTransferStage::Receiver,
        ] {
            for kind in [
                FinalizationCut::Work,
                FinalizationCut::Cancellation,
                FinalizationCut::Panic,
            ] {
                let literal = "<p id=a class=k>A</p><p id=b class=k>B</p>";
                let mut input = Some(ordinary(literal, true));
                let sealed = input.as_mut().unwrap();
                let source = sealed.source().identity().clone();
                sealed.parts.control.arm_finalization_transfer(stage, kind);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    ImmutableParseReport::take_document(&mut input)
                }));
                let kept = input.as_mut().expect("terminal source owner retained");
                let seen = kept
                    .parts
                    .control
                    .finalization_transfer_observation()
                    .expect("actual paired boundary reached");
                assert_eq!(seen.stage, stage);
                assert!(seen.strings_prepared);
                assert_eq!(
                    seen.attributes_prepared,
                    stage == FinalizationTransferStage::Receiver
                );
                assert!(seen.pool_bytes > 0);
                assert!(seen.retained_bytes > kept.parts.control.usage().actual_capacity_bytes());
                assert_eq!(kept.source().identity(), &source);
                assert_eq!(kept.source().as_utf8(), Some(literal));
                match kind {
                    FinalizationCut::Work => assert!(
                        matches!(result.unwrap(), Err(SessionError::Stop(ParseStop::LimitExceeded { resource: crate::HtmlParseResource::WorkCredits, limit, .. })) if limit == seen.work)
                    ),
                    FinalizationCut::Cancellation => assert!(matches!(
                        result.unwrap(),
                        Err(SessionError::Stop(ParseStop::Cancelled))
                    )),
                    FinalizationCut::Panic => {
                        assert_eq!(
                            result.unwrap_err().downcast_ref::<&str>(),
                            Some(&"terminal value-sharing transfer cut")
                        );
                        assert_eq!(kept.parts.control.stop(), None);
                        assert!(kept.parts.control.first_stop_usage().is_none());
                    }
                }
                // First-prepare cuts retain recovered terminal construction;
                // receiver cuts retain the republished immutable native view.
                assert_eq!(
                    kept.document().is_some(),
                    stage == FinalizationTransferStage::Receiver
                );
                let first = kept.parts.control.stop();
                let at_stop = kept.parts.control.first_stop_usage().cloned();
                let work = kept
                    .parts
                    .control
                    .usage()
                    .resource(crate::HtmlParseResource::WorkCredits);
                let retry = ImmutableParseReport::take_document(&mut input);
                match first {
                    Some(stop) => {
                        assert!(matches!(retry, Err(SessionError::Stop(actual)) if actual == stop))
                    }
                    None => assert!(matches!(retry, Err(SessionError::Consumed))),
                }
                let kept = input.as_mut().unwrap();
                assert_eq!(
                    kept.parts
                        .control
                        .usage()
                        .resource(crate::HtmlParseResource::WorkCredits),
                    work
                );
                assert_eq!(kept.parts.control.first_stop_usage(), at_stop.as_ref());
                kept.parts.release().unwrap();
                assert_eq!(kept.parts.control.stop(), first);
                super::super::tests::assert_released(&kept.parts.control);
            }
        }
    }
}
