//! Contextual parse history is distinct from later canonical insertion effects.
use super::*;
use surgeist_html_parser::HtmlParseMetadataStep;
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlFragmentContext};

const MARKUP: &str = "<input value=A form=outer><select><option selected>X</option></select>";

struct ContextFixture {
    dom: Dom,
    document: DocumentId,
    outer: NodeId,
    host: NodeId,
    destination: DocumentId,
    destination_form: NodeId,
    capture: DomHtmlCapture,
    report: Arc<HtmlParseReport>,
}

fn capture(dom: &Dom, document: DocumentId) -> DomHtmlCapture {
    dom.snapshot()
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &AtomicBool::new(false),
        )
        .unwrap()
}

fn fixture() -> ContextFixture {
    let (mut dom, _) = setup();
    dom.set_limits(
        dom.limits()
            .with_select_processing(128, 4_000_000, 10_000_000)
            .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let outer = batch.create_element_ns(&document, Namespace::html(), "form");
    batch.set_attribute(&outer, "id", "outer");
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&outer, &document, None);
    batch.insert_before(&host, &outer, None);
    let destination = batch.create_document();
    let destination_form = batch.create_element_ns(&destination, Namespace::html(), "form");
    // Equal text in another Document must never stand in for the captured Form.
    batch.set_attribute(&destination_form, "id", "outer");
    batch.insert_before(&destination_form, &destination, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.document(&document).unwrap();
    let outer = commit.created(&outer).unwrap().unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let destination = commit.document(&destination).unwrap();
    let destination_form = commit.created(&destination_form).unwrap().unwrap();
    let capture = capture(&dom, document);
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    assert_eq!(
        context.nearest_form().unwrap().subject(),
        &capture.subject(outer).unwrap()
    );
    let report = fragment_report(&capture, host, HtmlSource::utf8(MARKUP), false);
    assert_report(&report);
    ContextFixture {
        dom,
        document,
        outer,
        host,
        destination,
        destination_form,
        capture,
        report,
    }
}

fn assert_report(report: &HtmlParseReport) {
    assert_eq!(report.source().as_utf8(), Some(MARKUP));
    let mut controls = 0;
    let mut options = 0;
    let mut inputs = 0;
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if row.form_kind().is_some() {
                    // Pinned create-element step15 requires a null fragment
                    // context for parser-special association. The captured
                    // nearest Form affects recovery, not this ownership slot.
                    assert!(row.form_owner().unwrap().is_none());
                    assert_eq!(row.form_parser_inserted(), Some(false));
                    controls += 1;
                }
                if let Some(value) = row.input_raw_value() {
                    assert_eq!(value.code_units(), [65]);
                    assert!(!row.input_flags().unwrap().dirty_value());
                    inputs += 1;
                }
                if let Some(select) = row.option_nearest_select() {
                    assert!(select.is_some());
                    assert!(row.option_flags().unwrap().selectedness());
                    assert!(!row.option_flags().unwrap().dirtiness());
                    options += 1;
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!((controls, inputs, options), (2, 1, 1));
}

#[test]
fn captured_form_context_stays_historical_while_insertion_clone_and_adoption_reset_owners() {
    let ContextFixture {
        mut dom,
        document,
        outer,
        host,
        destination,
        destination_form,
        capture,
        report,
    } = fixture();
    let source = report.source().identity().clone();
    let before = dom.snapshot();
    let installed = dom
        .install_parsed_html_fragment(report.clone(), &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    assert!(std::ptr::eq(installed.report(), Arc::as_ref(&report)));
    assert_eq!(installed.document(), document);
    assert_eq!(installed.context_basis(), Some(capture.basis()));
    assert_eq!(
        installed.captured_node_for(&capture.subject(outer).unwrap()),
        Some(outer)
    );
    assert_ne!(outer, destination_form);
    assert_eq!(dom.snapshot().revision().get(), before.revision().get() + 1);
    let input = element(&installed, "input");
    let select = element(&installed, "select");
    let option = element(&installed, "option");
    let detached = installed.snapshot();
    assert!(detached.node(installed.root()).unwrap().parent().is_none());
    assert_eq!(
        detached.node(installed.root()).unwrap().children(),
        &[input, select]
    );
    assert_eq!(detached.node(select).unwrap().children(), &[option]);
    assert!(detached.node(host).unwrap().children().is_empty());
    assert_eq!(
        detached
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .form_owner()
            .unwrap(),
        None
    );
    assert_eq!(
        detached
            .node(select)
            .unwrap()
            .as_select()
            .unwrap()
            .form_owner()
            .unwrap(),
        None
    );
    assert!(
        !detached
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .parser_inserted()
    );
    assert!(
        !detached
            .node(select)
            .unwrap()
            .as_select()
            .unwrap()
            .state()
            .parser_inserted()
    );
    assert_eq!(
        detached
            .node(option)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select()
            .unwrap(),
        Some(select)
    );
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(Some(owner)) = row.option_nearest_select() {
                    assert_eq!(installed.node_for(&row.node().id()), Some(option));
                    assert_eq!(installed.node_for(&owner.id()), Some(select));
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    let mut batch = DomBatch::new();
    batch.insert_before(installed.root(), host, None);
    let inserted = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let connected = dom.snapshot();
    assert_eq!(inserted.changes().from(), detached.stamp());
    assert_eq!(connected.revision().get(), detached.revision().get() + 1);
    assert_eq!(connected.node(host).unwrap().children(), &[input, select]);
    assert!(
        connected
            .node(installed.root())
            .unwrap()
            .children()
            .is_empty()
    );
    assert_eq!(
        connected
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(outer)
    );
    assert_eq!(
        connected
            .node(select)
            .unwrap()
            .as_select()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(outer)
    );
    assert!(
        !connected
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .parser_inserted()
    );
    assert_eq!(
        connected
            .node(option)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select()
            .unwrap(),
        Some(select)
    );
    assert!(
        connected
            .node(option)
            .unwrap()
            .as_option()
            .unwrap()
            .state()
            .flags()
            .selectedness()
    );
    // A real canonical value edit establishes current/default divergence before
    // cloning; the parser report still owns the original A and clean flag.
    let mut batch = DomBatch::new();
    batch.set_input_value(input, "current");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let edited = dom.snapshot();
    assert_eq!(
        edited.node(input).unwrap().attribute(&"value".into()),
        Some(&"A".into())
    );
    assert!(
        edited
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .flags()
            .dirty_value()
    );
    let mut batch = DomBatch::new();
    let clone = batch.import_node(&edited, input, document, false);
    let imported = batch.import_node(&edited, input, destination, false);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let clone = commit.created(&clone).unwrap().unwrap();
    let imported = commit.created(&imported).unwrap().unwrap();
    assert_ne!(clone, input);
    assert_ne!(imported, input);
    assert_ne!(clone, imported);
    let cloned = dom.snapshot();
    for (id, owner_document) in [(clone, document), (imported, destination)] {
        let node = cloned.node(id).unwrap();
        assert_eq!(node.owner_document(), Some(owner_document));
        assert_eq!(node.parent(), None);
        let view = node.as_input().unwrap();
        assert_eq!(view.form_owner().unwrap(), None);
        assert!(!view.state().parser_inserted());
        assert!(view.state().flags().dirty_value());
        assert_eq!(view.value(), "current".encode_utf16().collect::<Vec<_>>());
    }
    let mut batch = DomBatch::new();
    batch.adopt(input, destination);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let adopted = dom.snapshot();
    assert_eq!(
        adopted.node(input).unwrap().owner_document(),
        Some(destination)
    );
    assert_eq!(adopted.node(input).unwrap().parent(), None);
    assert_eq!(
        adopted
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .form_owner()
            .unwrap(),
        None
    );
    assert_eq!(
        connected
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(outer)
    );
    let mut batch = DomBatch::new();
    batch.insert_before(input, destination_form, None);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    assert_eq!(
        current
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(destination_form)
    );
    assert_eq!(
        current.node(input).unwrap().as_input().unwrap().value(),
        "current".encode_utf16().collect::<Vec<_>>()
    );
    assert!(
        current
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .flags()
            .dirty_value()
    );
    assert_eq!(
        current
            .node(select)
            .unwrap()
            .as_select()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(outer)
    );
    assert_eq!(current.node(host).unwrap().children(), &[select]);
    assert_eq!(
        detached.node(input).unwrap().as_input().unwrap().value(),
        [65]
    );
    assert_eq!(
        detached
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .form_owner()
            .unwrap(),
        None
    );
    assert_eq!(report.source().identity(), &source);
    assert_report(&report);
}

#[test]
fn captured_form_context_refuses_foreign_stale_and_cancelled_installations_atomically() {
    for case in 0..3 {
        let ContextFixture {
            mut dom,
            document,
            outer,
            host,
            capture: original,
            report,
            ..
        } = fixture();
        let foreign = capture(&dom, document);
        if case == 1 {
            let mut batch = DomBatch::new();
            batch.set_attribute(outer, "id", "changed-after-capture");
            dom.apply(batch, &AtomicBool::new(false)).unwrap();
        }
        let before = dom.snapshot();
        let usage = dom.usage();
        let selected = if case == 0 { &foreign } else { &original };
        let error = dom
            .install_parsed_html_fragment(
                report.clone(),
                selected,
                limits(),
                &AtomicBool::new(case == 2),
            )
            .unwrap_err();
        match case {
            0 => assert!(matches!(error, DomParsedHtmlInstallError::ForeignContext)),
            1 => assert!(
                matches!(error, DomParsedHtmlInstallError::StaleDestination { expected, actual } if expected == original.basis().store() && actual == before.stamp())
            ),
            2 => assert!(matches!(error, DomParsedHtmlInstallError::Cancelled)),
            _ => unreachable!(),
        }
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
        assert!(dom.snapshot().node(host).unwrap().children().is_empty());
        assert_eq!(dom.snapshot().node(outer).unwrap().children(), &[host]);
        assert_report(&report);
        if case == 2 {
            // Cancellation leaves this exact completed report/capture reusable.
            let installed = dom
                .install_parsed_html_fragment(
                    report.clone(),
                    &original,
                    limits(),
                    &AtomicBool::new(false),
                )
                .unwrap();
            assert!(std::ptr::eq(installed.report(), Arc::as_ref(&report)));
            assert_eq!(
                installed.captured_node_for(&original.subject(outer).unwrap()),
                Some(outer)
            );
        }
    }
}

const NONANCESTOR_MARKUP: &str = "<!doctype html><table><form id=f><tr><td><input value=X></table>";

fn assert_nonancestor_report(
    report: &HtmlParseReport,
    installed: &DomParsedHtmlInstallation,
    input: NodeId,
    form: NodeId,
) {
    assert_eq!(report.source().as_utf8(), Some(NONANCESTOR_MARKUP));
    let mut inputs = 0;
    let mut input_subject = None;
    let mut association = None;
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(value) = row.input_raw_value() {
                    assert_eq!(value.code_units(), [88]);
                    assert!(!row.input_flags().unwrap().dirty_value());
                    assert_eq!(installed.node_for(&row.node().id()), Some(input));
                    assert!(input_subject.replace(row.node().id()).is_none());
                    inputs += 1;
                }
                if let Some(owner) = row.form_owner() {
                    assert_eq!(row.form_parser_inserted(), Some(true));
                    let owner = owner.unwrap();
                    assert!(association.replace((row.node().id(), owner.id())).is_none());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(inputs, 1);
    // Input and FormAssociation are distinct records of the same exact subject.
    let (subject, owner) = association.unwrap();
    assert_eq!(Some(subject), input_subject);
    assert_eq!(installed.node_for(&owner), Some(form));
}

#[test]
fn true_nonancestor_parser_owner_survives_cloning_and_unknown_removal_refusal() {
    let report = parse(HtmlSource::utf8(NONANCESTOR_MARKUP));
    let source = report.source().identity().clone();
    let (mut dom, environment) = setup();
    let mut batch = DomBatch::new();
    let destination = batch.create_document();
    let destination_form = batch.create_element_ns(&destination, Namespace::html(), "form");
    batch.insert_before(&destination_form, &destination, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let destination = commit.document(&destination).unwrap();
    let destination_form = commit.created(&destination_form).unwrap().unwrap();
    let installed = install(&mut dom, report.clone(), environment);
    assert!(std::ptr::eq(installed.report(), Arc::as_ref(&report)));
    let input = element(&installed, "input");
    let form = element(&installed, "form");
    let original = installed.snapshot();
    let original_input = original.node(input).unwrap().as_input().unwrap();
    assert_eq!(original_input.form_owner().unwrap(), Some(form));
    assert!(original_input.state().parser_inserted());
    assert_eq!(original_input.value(), [88]);
    assert!(!original_input.state().flags().dirty_value());
    let mut ancestor = original.node(input).unwrap().parent();
    while let Some(id) = ancestor {
        assert_ne!(id, form, "the parser-associated Form is not an ancestor");
        ancestor = original.node(id).unwrap().parent();
    }
    assert_nonancestor_report(&report, &installed, input, form);

    let mut batch = DomBatch::new();
    batch.set_input_value(input, "current");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let edited = dom.snapshot();
    let edited_input = edited.node(input).unwrap().as_input().unwrap();
    assert_eq!(edited_input.form_owner().unwrap(), Some(form));
    assert!(edited_input.state().parser_inserted());
    assert_eq!(
        edited_input.value(),
        "current".encode_utf16().collect::<Vec<_>>()
    );
    assert!(edited_input.state().flags().dirty_value());
    assert_eq!(
        edited.node(input).unwrap().attribute(&"value".into()),
        Some(&"X".into())
    );

    // Clone/import copies the specified Input value/flags into fresh control
    // state; it does not transfer the source's parser-special Form identity.
    let mut batch = DomBatch::new();
    let clone = batch.import_node(&edited, input, installed.document(), false);
    let imported = batch.import_node(&edited, input, destination, false);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let clone = commit.created(&clone).unwrap().unwrap();
    let imported = commit.created(&imported).unwrap().unwrap();
    assert_ne!(clone, input);
    assert_ne!(imported, input);
    assert_ne!(clone, imported);
    let cloned = dom.snapshot();
    assert_eq!(
        cloned.node(destination_form).unwrap().owner_document(),
        Some(destination)
    );
    for (id, document) in [(clone, installed.document()), (imported, destination)] {
        let node = cloned.node(id).unwrap();
        assert_eq!(node.owner_document(), Some(document));
        assert_eq!(node.parent(), None);
        assert_eq!(node.attribute(&"value".into()), Some(&"X".into()));
        let control = node.as_input().unwrap();
        assert_eq!(control.form_owner().unwrap(), None);
        assert!(!control.state().parser_inserted());
        assert_eq!(
            control.value(),
            "current".encode_utf16().collect::<Vec<_>>()
        );
        assert!(control.state().flags().dirty_value());
    }
    assert_eq!(
        cloned
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(form)
    );
    assert!(
        cloned
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .parser_inserted()
    );

    let mut batch = DomBatch::new();
    batch.detach(input);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::FocusedAreaDesignationUnavailable {
            subject: Some(input),
            document: Some(installed.document())
        }
    );
    let current = dom.snapshot();
    assert_eq!(current.stamp(), cloned.stamp());
    assert_eq!(
        current.node(input).unwrap().parent(),
        cloned.node(input).unwrap().parent()
    );
    let control = current.node(input).unwrap().as_input().unwrap();
    assert_eq!(control.form_owner().unwrap(), Some(form));
    assert!(control.state().parser_inserted());
    assert_eq!(
        control.value(),
        "current".encode_utf16().collect::<Vec<_>>()
    );
    assert!(control.state().flags().dirty_value());
    assert_eq!(
        current.node(input).unwrap().attribute(&"value".into()),
        Some(&"X".into())
    );
    assert_eq!(original_input.form_owner().unwrap(), Some(form));
    assert!(original_input.state().parser_inserted());
    assert_eq!(original_input.value(), [88]);
    assert!(!original_input.state().flags().dirty_value());
    assert_eq!(edited_input.form_owner().unwrap(), Some(form));
    assert!(edited_input.state().parser_inserted());
    assert_eq!(report.source().identity(), &source);
    assert_nonancestor_report(&report, &installed, input, form);
}

const CHECKED_MARKUP: &str = concat!(
    "<!doctype html><form id=f>",
    "<input type=radio name=g value=one checked>",
    "<input type=radio name=g value=two checked>",
    "<input type=checkbox name=g value=box checked=false>",
    "<input type=checkbox name=g value=off>",
    "</form>"
);
const CHECKED_VALUES: [&str; 4] = ["one", "two", "box", "off"];
const PARSED_CHECKED: [bool; 4] = [false, true, true, false];

fn checked_index(value: &DocumentString) -> usize {
    CHECKED_VALUES
        .iter()
        .position(|literal| {
            value
                .code_units()
                .iter()
                .copied()
                .eq(literal.encode_utf16())
        })
        .expect("one of the four independently authored Input values")
}

fn assert_checked_report(report: &HtmlParseReport) {
    assert_eq!(report.source().as_utf8(), Some(CHECKED_MARKUP));
    let mut input_subjects = [None, None, None, None];
    let mut associations = Vec::new();
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(value) = row.input_raw_value() {
                    let index = checked_index(value);
                    assert!(input_subjects[index].replace(row.node().id()).is_none());
                    let flags = row.input_flags().unwrap();
                    assert_eq!(
                        flags.input_type(),
                        if index < 2 {
                            surgeist_html::HtmlInputType::Radio
                        } else {
                            surgeist_html::HtmlInputType::Checkbox
                        }
                    );
                    assert_eq!(flags.checkedness(), PARSED_CHECKED[index]);
                    assert!(!flags.dirty_checkedness());
                    assert!(!flags.dirty_value());
                    assert!(!flags.indeterminate());
                    assert!(!flags.user_validity());
                }
                if let Some(owner) = row.form_owner() {
                    assert_eq!(row.form_parser_inserted(), Some(true));
                    associations.push((row.node().id(), owner.unwrap().id()));
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(associations.len(), 4);
    let mut common_owner = None;
    // Join independently emitted family records by qualified report identity,
    // never by cursor adjacency or a coincidentally equal numeric position.
    for subject in input_subjects {
        let subject = subject.unwrap();
        let mut matches = associations.iter().filter(|(node, _)| node == &subject);
        let (_, owner) = matches.next().unwrap();
        assert!(matches.next().is_none());
        if let Some(expected) = &common_owner {
            assert_eq!(owner, expected);
        } else {
            common_owner = Some(owner.clone());
        }
    }
}

#[test]
fn parser_radio_normalization_and_checkbox_flags_transfer_before_canonical_checked_setters() {
    let report = parse(HtmlSource::utf8(CHECKED_MARKUP));
    let source = report.source().identity().clone();
    // Inspect actual completed producer flags before the DOM owner or any
    // canonical setter can affect them. Reconstructing checkedness from the
    // two authored radio attributes would incorrectly select both radios.
    assert_checked_report(&report);
    let (mut dom, environment) = setup();
    let before = dom.snapshot();
    let installed = install(&mut dom, report.clone(), environment);
    assert!(std::ptr::eq(installed.report(), Arc::as_ref(&report)));
    let original = installed.snapshot();
    assert_eq!(original.revision().get(), before.revision().get() + 1);
    let form = element(&installed, "form");
    let ids: [NodeId; 4] = original.node(form).unwrap().children().try_into().unwrap();
    for (index, id) in ids.iter().copied().enumerate() {
        let node = original.node(id).unwrap();
        let control = node.as_input().unwrap();
        assert_eq!(
            control.value(),
            CHECKED_VALUES[index].encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            node.attribute(&"value".into()),
            Some(&DocumentString::from(CHECKED_VALUES[index]))
        );
        let authored_checked: Option<&[u16]> = match index {
            0 | 1 => Some(&[]),
            2 => Some(&[102, 97, 108, 115, 101]),
            _ => None,
        };
        assert_eq!(
            node.attribute(&"checked".into())
                .map(|value| value.code_units()),
            authored_checked
        );
        let flags = control.state().flags();
        assert_eq!(flags.checkedness(), PARSED_CHECKED[index]);
        assert!(!flags.dirty_checkedness());
        assert!(!flags.dirty_value());
        assert!(!flags.indeterminate());
        assert!(!flags.user_validity());
        assert!(control.state().parser_inserted());
        assert_eq!(control.form_owner().unwrap(), Some(form));
    }
    let mut metadata = report.metadata();
    let mut associated = [false; 4];
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(value) = row.input_raw_value() {
                    assert_eq!(
                        installed.node_for(&row.node().id()),
                        Some(ids[checked_index(value)])
                    );
                }
                if let Some(owner) = row.form_owner() {
                    let subject = installed.node_for(&row.node().id()).unwrap();
                    let index = ids.iter().position(|id| *id == subject).unwrap();
                    assert!(!associated[index]);
                    associated[index] = true;
                    assert_eq!(row.form_parser_inserted(), Some(true));
                    let owner = owner.unwrap();
                    assert_eq!(installed.node_for(&owner.id()), Some(form));
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(associated, [true; 4]);
    let mut batch = DomBatch::new();
    batch.set_input_checked(ids[0], true);
    batch.set_input_checked(ids[2], false);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    assert_eq!(commit.changes().from(), original.stamp());
    assert_eq!(commit.changes().to(), current.stamp());
    assert_eq!(current.revision().get(), original.revision().get() + 1);
    for (index, id) in ids.iter().copied().enumerate() {
        let node = current.node(id).unwrap();
        let control = node.as_input().unwrap();
        let flags = control.state().flags();
        assert_eq!(flags.checkedness(), [true, false, false, false][index]);
        assert_eq!(flags.dirty_checkedness(), [true, false, true, false][index]);
        assert!(!flags.dirty_value());
        assert!(!flags.indeterminate());
        assert!(!flags.user_validity());
        assert!(control.state().parser_inserted());
        assert_eq!(control.form_owner().unwrap(), Some(form));
        assert_eq!(
            control.value(),
            CHECKED_VALUES[index].encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            node.attribute(&"checked".into()),
            original.node(id).unwrap().attribute(&"checked".into())
        );
        let historical = original.node(id).unwrap().as_input().unwrap();
        assert_eq!(
            historical.state().flags().checkedness(),
            PARSED_CHECKED[index]
        );
        assert!(!historical.state().flags().dirty_checkedness());
    }
    assert_eq!(report.source().identity(), &source);
    assert_checked_report(&report);
}
