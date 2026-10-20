//! Factored terminal-layer qualification. Off still pays both pools' fixed
//! lifecycle and the same promotion traversal; it is not a pool-free parser.
use super::*;
use crate::parser::control::{FinalizationCut, FinalizationTransferStage, ParseControl, ParseStop};
use crate::parser::tree_builder::{
    StandaloneDocumentDriver,
    context::{ContextFragmentDriver, ownership::FragmentInitializationOwner},
    head::HeadStep,
    seal::OrdinaryInitializationOwner,
};
use crate::request::ValueSharingMode;
use crate::*;
use std::sync::Arc;

const EQUAL_BANKS: &str =
    "<table><tbody><tr><td class=k title=v>A</td><td class=k title=v>B</td></tr></tbody></table>";
const DISTINCT_BANKS_TEXT: &str = "<p id=a class=k>X</p><p id=b class=k>&#88;</p>";

mod selected;

const MODES: [ValueSharingMode; 3] = [
    ValueSharingMode::Off,
    ValueSharingMode::BankOnly,
    ValueSharingMode::Combined,
];

fn context() -> HtmlFragmentContext {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    let div = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&div).unwrap();
    builder.establish_no_shadow_root(&div).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap()
}

fn parse(fragment: bool, mode: ValueSharingMode, markup: &str) -> ImmutableParseReport {
    let literal = if fragment {
        markup.to_owned()
    } else {
        format!("<!doctype html>{markup}")
    };
    let source = HtmlSource::utf8(literal.clone());
    let identity = source.identity().clone();
    let limits = super::super::tests::limits(100);
    let report = if fragment {
        let context = context();
        let capture = context.capture().identity().clone();
        let request = HtmlFragmentParseRequest::new(source, context, limits)
            .unwrap()
            .with_value_sharing(mode);
        let cancel = request.cancellation().clone();
        let report = request.parse().unwrap();
        cancel.cancel();
        assert!(report.document().is_none());
        assert_eq!(report.context().unwrap().capture().identity(), &capture);
        report
    } else {
        let request = HtmlDocumentParseRequest::offline(source, limits).with_value_sharing(mode);
        let cancel = request.cancellation().clone();
        let report = request.parse().unwrap();
        cancel.cancel();
        assert_eq!(
            report.document().unwrap().mode(),
            HtmlDocumentMode::NoQuirks
        );
        assert!(report.fragment().is_none());
        report
    };
    assert_eq!(report.source().identity(), &identity);
    assert_eq!(report.source().as_utf8(), Some(literal.as_str()));
    report
}

fn node<'a>(report: &'a ImmutableParseReport, id: &HtmlNodeId) -> HtmlNode<'a> {
    if let Some(document) = report.document() {
        document.node(id).unwrap()
    } else {
        report.fragment().unwrap().node(id).unwrap()
    }
}
fn elements(report: &ImmutableParseReport, local: &str) -> Vec<HtmlNodeId> {
    let root = report.document().map_or_else(
        || report.fragment().unwrap().root(),
        |document| document.root(),
    );
    let mut pending = vec![root.id()];
    let mut found = Vec::new();
    while let Some(id) = pending.pop() {
        let value = node(report, &id);
        if matches!(value.kind(), HtmlNodeKind::Element(element) if element.name().local_name() == &DocumentString::from(local))
        {
            found.push(id);
        }
        let children: Vec<_> = value.children().collect();
        pending.extend(children.into_iter().rev());
    }
    found
}
fn element<'a>(report: &'a ImmutableParseReport, id: &HtmlNodeId) -> &'a HtmlElement {
    let HtmlNodeKind::Element(value) = node(report, id).kind() else {
        panic!("literal Element")
    };
    assert_eq!(value.name().namespace(), &Namespace::html());
    assert!(value.name().prefix().is_none());
    value
}
fn text<'a>(
    report: &'a ImmutableParseReport,
    parent: &HtmlNodeId,
) -> (HtmlNodeId, &'a DocumentString) {
    let children: Vec<_> = node(report, parent).children().collect();
    assert_eq!(children.len(), 1);
    let HtmlNodeKind::Text(value) = node(report, &children[0]).kind() else {
        panic!("literal Text")
    };
    (children[0].clone(), value)
}
fn origin(
    report: &ImmutableParseReport,
    id: &HtmlNodeId,
    field: HtmlParseField,
    start: usize,
    end: usize,
    kind: HtmlParseOriginKind,
) {
    let rows = report.associations();
    let mut actual = Vec::new();
    for index in 0..rows.len() {
        let row = rows.row(index).unwrap();
        if row.relation() == HtmlParseAssociationRelation::Output
            && let HtmlParseDestination::Field {
                node,
                field: actual_field,
            } = row.destination()
            && node.id() == *id
            && actual_field == field
        {
            let source = row.source().unwrap();
            assert_eq!(source.source(), report.source().identity());
            assert!(row.group().is_some());
            actual.push((
                row.start(),
                row.end(),
                source.start(),
                source.end(),
                row.origin_kind().unwrap(),
            ));
        }
    }
    assert_eq!(actual, [(0, 1, start, end, kind)]);
}

fn check_equal_banks(
    report: &ImmutableParseReport,
    mode: ValueSharingMode,
) -> Arc<Vec<surgeist_html::Attribute>> {
    let cells = elements(report, "td");
    assert_eq!(cells.len(), 2);
    assert_ne!(cells[0], cells[1]);
    let values = [&cells[0], &cells[1]].map(|id| element(report, id));
    for (index, value) in values.iter().enumerate() {
        let attributes = value.attributes();
        assert_eq!(attributes.len(), 2);
        for (ordinal, (name, literal)) in [("class", "k"), ("title", "v")].into_iter().enumerate() {
            let attribute = &attributes.as_slice()[ordinal];
            assert_eq!(attribute.name().namespace(), &Namespace::none());
            assert!(attribute.name().prefix().is_none());
            assert_eq!(attribute.name().local_name(), &DocumentString::from(name));
            assert_eq!(attribute.value(), &DocumentString::from(literal));
            let authored = report.source().as_utf8().unwrap();
            let offset = authored
                .match_indices(&format!("{name}={literal}"))
                .nth(index)
                .unwrap()
                .0
                + name.len()
                + 1;
            origin(
                report,
                &cells[index],
                HtmlParseField::AttributeValue(ordinal),
                offset,
                offset + 1,
                HtmlParseOriginKind::Literal,
            );
        }
        assert_eq!(
            text(report, &cells[index]).1,
            &DocumentString::from(if index == 0 { "A" } else { "B" })
        );
    }
    assert_eq!(
        Arc::ptr_eq(
            values[0].attributes().shared_attributes(),
            values[1].attributes().shared_attributes()
        ),
        mode != ValueSharingMode::Off
    );
    values[0].attributes().shared_attributes().clone()
}

fn check_distinct_banks_text(
    report: &ImmutableParseReport,
    mode: ValueSharingMode,
) -> Arc<Vec<surgeist_html::Attribute>> {
    let paragraphs = elements(report, "p");
    assert_eq!(paragraphs.len(), 2);
    assert_ne!(paragraphs[0], paragraphs[1]);
    let values = [&paragraphs[0], &paragraphs[1]].map(|id| element(report, id));
    for (index, value) in values.iter().enumerate() {
        let attributes = value.attributes();
        assert_eq!(attributes.len(), 2);
        for (ordinal, name) in ["id", "class"].into_iter().enumerate() {
            assert_eq!(
                attributes.as_slice()[ordinal].name().local_name(),
                &DocumentString::from(name)
            );
        }
        assert_eq!(
            attributes.as_slice()[0].value(),
            &DocumentString::from(if index == 0 { "a" } else { "b" })
        );
        assert_eq!(attributes.as_slice()[1].value(), &DocumentString::from("k"));
    }
    assert!(!Arc::ptr_eq(
        values[0].attributes().shared_attributes(),
        values[1].attributes().shared_attributes()
    ));
    assert_eq!(
        Arc::ptr_eq(
            values[0].attributes().as_slice()[1]
                .value()
                .shared_code_units(),
            values[1].attributes().as_slice()[1]
                .value()
                .shared_code_units()
        ),
        mode == ValueSharingMode::Combined
    );
    let first = text(report, &paragraphs[0]);
    let second = text(report, &paragraphs[1]);
    assert_ne!(first.0, second.0);
    assert_eq!(first.1.code_units(), &[88]);
    assert_eq!(second.1.code_units(), &[88]);
    let authored = report.source().as_utf8().unwrap();
    let literal = authored.find(">X<").unwrap() + 1;
    let reference = authored.find("&#88;").unwrap();
    origin(
        report,
        &first.0,
        HtmlParseField::Text,
        literal,
        literal + 1,
        HtmlParseOriginKind::Literal,
    );
    origin(
        report,
        &second.0,
        HtmlParseField::Text,
        reference,
        reference + 5,
        HtmlParseOriginKind::CharacterReference,
    );
    assert_eq!(
        Arc::ptr_eq(first.1.shared_code_units(), second.1.shared_code_units()),
        mode == ValueSharingMode::Combined
    );
    values[0].attributes().shared_attributes().clone()
}

#[test]
fn layer_selection_preserves_real_document_and_fragment_banks_and_report_lifetime() {
    for fragment in [false, true] {
        for mode in MODES {
            let report = parse(fragment, mode, EQUAL_BANKS);
            let held = check_equal_banks(&report, mode);
            let weak = Arc::downgrade(&held);
            let usage = report.usage().clone();
            assert_eq!(report.usage(), &usage);
            drop(report);
            assert_eq!(held[0].value().code_units(), &[107]);
            assert!(weak.upgrade().is_some());
            drop(held);
            assert!(
                weak.upgrade().is_none(),
                "terminal pools do not retain report values"
            );
        }
    }
}

#[test]
fn only_combined_reuses_strings_across_distinct_banks_and_independent_text() {
    for fragment in [false, true] {
        for mode in MODES {
            let report = parse(fragment, mode, DISTINCT_BANKS_TEXT);
            drop(check_distinct_banks_text(&report, mode));
        }
    }
}

enum Sealed {
    Document(Option<SealedOrdinaryConstruction>),
    Fragment(Option<SealedFragmentConstruction>),
}
impl Sealed {
    fn new(fragment: bool, mode: ValueSharingMode, literal: &str) -> Self {
        let limits = super::super::tests::limits(100);
        if fragment {
            let request =
                HtmlFragmentParseRequest::new(HtmlSource::utf8(literal), context(), limits)
                    .unwrap()
                    .with_value_sharing(mode);
            let mut owner = FragmentInitializationOwner::new(request);
            let mut parts = {
                let mut attempt = owner.begin().unwrap();
                let mut driver = Some(attempt.take_driver().unwrap());
                loop {
                    match driver.as_mut().unwrap().step().unwrap() {
                        HeadStep::Consumed => {}
                        HeadStep::Complete => break,
                        other => panic!("complete fragment fixture: {other:?}"),
                    }
                }
                let mut completed = ContextFragmentDriver::take_completed(&mut driver).unwrap();
                completed.prepare_native_associations().unwrap();
                Some(completed.freeze_parts().unwrap())
            };
            Self::Fragment(Some(owner.seal(&mut parts).unwrap()))
        } else {
            let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits)
                .with_value_sharing(mode);
            let mut owner = OrdinaryInitializationOwner::new(request);
            let parts = {
                let mut attempt = owner.begin().unwrap();
                let mut driver = Some(attempt.take_driver().unwrap());
                loop {
                    match driver.as_mut().unwrap().step().unwrap() {
                        HeadStep::Consumed => {}
                        HeadStep::Complete => break,
                        other => panic!("complete document fixture: {other:?}"),
                    }
                }
                let mut completed = StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
                completed.prepare_native_associations().unwrap();
                completed.freeze_parts().unwrap()
            };
            Self::Document(Some(
                SealedOrdinaryConstruction::take(
                    &mut Some(owner.take_request().unwrap()),
                    &mut Some(parts),
                )
                .unwrap(),
            ))
        }
    }
    fn control(&mut self) -> &mut ParseControl {
        match self {
            Self::Document(value) => &mut value.as_mut().unwrap().parts.control,
            Self::Fragment(value) => &mut value.as_mut().unwrap().parts.control,
        }
    }
    fn report(&mut self) -> Result<ImmutableParseReport, SessionError> {
        match self {
            Self::Document(value) => ImmutableParseReport::take_document(value),
            Self::Fragment(value) => ImmutableParseReport::take_fragment(value),
        }
    }
    fn source(&self) -> &HtmlSource {
        match self {
            Self::Document(value) => value.as_ref().unwrap().source(),
            Self::Fragment(value) => value.as_ref().unwrap().source(),
        }
    }
    fn release(&mut self) {
        match self {
            Self::Document(value) => value.as_mut().unwrap().parts.release().unwrap(),
            Self::Fragment(value) => value.as_mut().unwrap().parts.release().unwrap(),
        }
    }
}

#[test]
fn all_layers_keep_both_pool_authorities_through_actual_terminal_refusal_and_cleanup() {
    for fragment in [false, true] {
        for mode in MODES {
            for stage in [
                FinalizationTransferStage::FirstPrepared,
                FinalizationTransferStage::Receiver,
            ] {
                for kind in [
                    FinalizationCut::Work,
                    FinalizationCut::Cancellation,
                    FinalizationCut::Panic,
                ] {
                    let literal = "<p id=a class=k>X</p><p id=b class=k>X</p>";
                    let mut sealed = Sealed::new(fragment, mode, literal);
                    let source = sealed.source().identity().clone();
                    sealed.control().arm_finalization_transfer(stage, kind);
                    let result =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sealed.report()));
                    let seen = sealed
                        .control()
                        .finalization_transfer_observation()
                        .expect("actual terminal boundary");
                    assert_eq!(seen.stage, stage);
                    assert!(seen.strings_prepared);
                    assert_eq!(
                        seen.attributes_prepared,
                        stage == FinalizationTransferStage::Receiver
                    );
                    assert_eq!(seen.pool_bytes == 0, mode == ValueSharingMode::Off);
                    assert!(seen.retained_bytes > sealed.control().usage().actual_capacity_bytes());
                    assert_eq!(sealed.source().identity(), &source);
                    assert_eq!(sealed.source().as_utf8(), Some(literal));
                    match kind {
                        FinalizationCut::Work => assert!(
                            matches!(result.unwrap(), Err(SessionError::Stop(ParseStop::LimitExceeded {resource: HtmlParseResource::WorkCredits, limit, ..})) if limit == seen.work)
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
                            assert_eq!(sealed.control().stop(), None);
                        }
                    }
                    let stop = sealed.control().stop();
                    let first = sealed.control().first_stop_usage().cloned();
                    let work = sealed
                        .control()
                        .usage()
                        .resource(HtmlParseResource::WorkCredits);
                    let retry = sealed.report();
                    match stop {
                        Some(stop) => assert!(
                            matches!(retry, Err(SessionError::Stop(actual)) if actual == stop)
                        ),
                        None => assert!(matches!(retry, Err(SessionError::Consumed))),
                    }
                    assert_eq!(
                        sealed
                            .control()
                            .usage()
                            .resource(HtmlParseResource::WorkCredits),
                        work
                    );
                    assert_eq!(sealed.control().first_stop_usage(), first.as_ref());
                    sealed.release();
                    super::super::tests::assert_released(sealed.control());
                    assert_eq!(sealed.control().stop(), stop);
                    assert_eq!(sealed.control().first_stop_usage(), first.as_ref());
                    assert_eq!(sealed.source().identity(), &source);
                    assert_eq!(sealed.source().as_utf8(), Some(literal));
                    let disposed_work = sealed
                        .control()
                        .usage()
                        .resource(HtmlParseResource::WorkCredits);
                    if stop.is_some() {
                        assert_eq!(disposed_work, work);
                    } else {
                        // Unwind leaves this controller active. FrozenArena
                        // disposal therefore charges its ordinary alias and
                        // ownership checks; cleanup_charge skips those charges
                        // only after a terminal stop. Neither path refunds work.
                        assert!(disposed_work.current() >= work.current());
                        assert!(disposed_work.high_water() >= work.high_water());
                    }
                }
            }
        }
    }
}
