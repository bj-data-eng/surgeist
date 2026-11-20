use super::{evaluate, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use crate::document_scripting::{DocumentScriptError, DocumentScriptHost, DocumentScriptPermit};
use crate::internal::dom_source::{DomSourceDriveError, DomSourceRuntimeError};
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_bindings_js::Error;
use surgeist_dom::{DomBatch, DomError, DomSnapshot, identity::NodeId};
use surgeist_html::values::{DocumentString, Namespace};

const SELECT_PENDING: usize = 8;
const SELECT_SCRATCH: usize = 4_000_000;
const SELECT_WORK: u64 = 100_000;

struct SelectFixture {
    host: DocumentScriptHost,
    permit: DocumentScriptPermit,
    cancel: Arc<AtomicBool>,
    select: NodeId,
    group: NodeId,
    first: NodeId,
    fallback: NodeId,
    content: NodeId,
}

impl SelectFixture {
    fn new(wrappers: &[(Namespace, &str)]) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
            limits
                .with_select_processing(SELECT_PENDING, SELECT_SCRATCH, SELECT_WORK)
                .unwrap()
        });
        let document = target.document();
        let mut batch = DomBatch::new();
        let select = batch.create_element_ns(document, Namespace::html(), "select");
        let button = batch.create_element_ns(document, Namespace::html(), "button");
        let content = batch.create_element_ns(document, Namespace::html(), "selectedcontent");
        batch.insert_before(&button, &select, None);
        batch.insert_before(&content, &button, None);

        let mut parent = &select;
        let wrappers: Vec<_> = wrappers
            .iter()
            .map(|(namespace, name)| batch.create_element_ns(document, namespace.clone(), *name))
            .collect();
        for wrapper in &wrappers {
            batch.insert_before(wrapper, parent, None);
            parent = wrapper;
        }
        let group = batch.create_element_ns(document, Namespace::html(), "optgroup");
        let first = batch.create_element_ns(document, Namespace::html(), "option");
        let fallback = batch.create_element_ns(document, Namespace::html(), "option");
        let first_text = batch.create_text(document, "group option");
        let fallback_text = batch.create_text(document, "fallback option");
        batch.insert_before(&first_text, &first, None);
        batch.insert_before(&fallback_text, &fallback, None);
        batch.insert_before(&first, &group, None);
        batch.insert_before(&group, parent, None);
        batch.insert_before(&fallback, &select, None);

        let commit = session.dom().apply(batch, &cancel).unwrap();
        let select = commit.created(&select).unwrap().unwrap();
        let group = commit.created(&group).unwrap().unwrap();
        let first = commit.created(&first).unwrap().unwrap();
        let fallback = commit.created(&fallback).unwrap().unwrap();
        let content = commit.created(&content).unwrap().unwrap();

        let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, group, "selectionGroup");
        let mut fixture = Self {
            host,
            permit,
            cancel,
            select,
            group,
            first,
            fallback,
            content,
        };
        fixture.clear_selection();
        fixture
    }

    fn clear_selection(&mut self) {
        self.host
            .with_native(|dom, _, _| {
                let mut batch = DomBatch::new();
                batch.set_select_selected_index(self.select, -1);
                dom.apply(batch, &self.cancel).unwrap();
            })
            .unwrap();
        self.assert_selection([false, false], None);
    }

    fn run(&mut self, script: &str) -> Result<(), DocumentScriptError> {
        self.host
            .in_document(&self.permit, |scope| evaluate(scope, script).map(drop))
    }

    fn assert_selection(&self, options: [bool; 2], content: Option<&str>) {
        let snapshot = self.host.snapshot().unwrap();
        assert_eq!(
            [self.first, self.fallback].map(|option| selected(&snapshot, option)),
            options,
        );
        let selected_content = snapshot.node(self.content).unwrap();
        let children = selected_content.children();
        if let Some(content) = content {
            assert_eq!(children.len(), 1);
            let text = snapshot.node(children[0]).unwrap();
            assert_eq!(text.data(), Some(&DocumentString::from(content)));
        } else {
            assert!(children.is_empty());
        }
    }

    fn assert_disabled_attribute(&self, value: Option<&str>) {
        let snapshot = self.host.snapshot().unwrap();
        let group = snapshot.node(self.group).unwrap();
        assert_eq!(
            group.attribute(&"disabled".into()),
            value.map(DocumentString::from).as_ref(),
        );
    }

    fn finish(mut self) {
        self.host
            .with_native(|dom, _, _| {
                assert!(dom.pending_source_edit().is_none());
                assert_eq!(dom.raw().pending_native_tasks(), 0);
            })
            .unwrap();
        self.host.stop().into_session().unwrap();
    }
}

fn selected(snapshot: &DomSnapshot, option: NodeId) -> bool {
    snapshot
        .node(option)
        .unwrap()
        .as_option()
        .unwrap()
        .state()
        .flags()
        .selectedness()
}

// HTML877d 58416..58433 calls the reset only after a presence change.
// Reset selects the first enabled option and updates selectedcontent when it
// changes selection (HTML877d 57721..57765).
pub(super) fn optgroup_disabled_presence_changes_reset_selection_and_selectedcontent() {
    let mut fixture = SelectFixture::new(&[]);
    fixture.run("selectionGroup.disabled = true").unwrap();
    fixture.assert_selection([false, true], Some("fallback option"));
    fixture.assert_disabled_attribute(Some(""));

    // Mutate only the existing attribute's text and clear native selection.
    fixture
        .run("selectionGroup.setAttributeNS(null, 'disabled', 'false')")
        .unwrap();
    fixture.assert_disabled_attribute(Some("false"));
    fixture.clear_selection();
    fixture.run("selectionGroup.disabled = true").unwrap();
    fixture.assert_selection([false, false], None);
    fixture.assert_disabled_attribute(Some(""));

    fixture.run("selectionGroup.disabled = false").unwrap();
    fixture.assert_selection([true, false], Some("group option"));
    fixture.assert_disabled_attribute(None);
    fixture.finish();
}

// HTML877d 58829..58861 stops at HTML datalist, hr, option, or a second
// ancestor optgroup. A foreign element with the same name has no such effect.
pub(super) fn optgroup_disabled_resets_through_foreign_wrappers_but_stops_at_html_barriers() {
    for barrier in ["datalist", "hr", "option", "optgroup"] {
        let html = Namespace::html();
        let wrappers = [(html.clone(), barrier), (html, "optgroup")];
        let wrappers = if barrier == "optgroup" {
            &wrappers[..]
        } else {
            &wrappers[..1]
        };
        let mut fixture = SelectFixture::new(wrappers);
        fixture.run("selectionGroup.disabled = true").unwrap();
        fixture.assert_disabled_attribute(Some(""));
        fixture.assert_selection([false, false], None);
        fixture.run("selectionGroup.disabled = false").unwrap();
        fixture.assert_disabled_attribute(None);
        fixture.assert_selection([false, false], None);
        fixture.finish();
    }

    let mut fixture = SelectFixture::new(&[(Namespace::from_uri("urn:foreign"), "option")]);
    fixture.run("selectionGroup.disabled = true").unwrap();
    fixture.assert_selection([false, true], Some("fallback option"));
    fixture.clear_selection();
    fixture.run("selectionGroup.disabled = false").unwrap();
    fixture.assert_selection([true, false], Some("group option"));
    fixture.finish();
}

pub(super) fn optgroup_disabled_keeps_the_first_select_work_refusal() {
    // Fund setup and projection before shortening only the native Select owner.
    // The direct parent can be visited within one step, but reset cannot finish.
    let mut fixture = SelectFixture::new(&[]);
    let (limits, usage) = fixture
        .host
        .with_native(|dom, _, _| {
            let limits = dom.limits();
            dom.raw_mut()
                .set_limits(
                    limits
                        .with_select_processing(SELECT_PENDING, SELECT_SCRATCH, 1)
                        .unwrap(),
                )
                .unwrap();
            (limits, dom.usage())
        })
        .unwrap();
    let before = fixture.host.snapshot().unwrap().stamp();
    let result = fixture.run("selectionGroup.disabled = true");
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("disabled addition must propagate the native Select work refusal: {result:?}");
    };
    assert!(
        matches!(
            failure.cause::<DomSourceDriveError>(),
            Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                DomError::SelectWorkLimit { limit: 1 }
            )))
        ),
        "{failure:?}"
    );
    assert_eq!(fixture.host.snapshot().unwrap().stamp(), before);
    fixture.assert_disabled_attribute(None);
    fixture.assert_selection([false, false], None);
    fixture
        .host
        .with_native(|dom, _, _| {
            assert!(dom.pending_source_edit().is_none());
            assert_eq!(dom.usage(), usage);
            assert_eq!(dom.raw().pending_native_tasks(), 0);
            dom.raw_mut().set_limits(limits).unwrap();
        })
        .unwrap();

    // The Source edit has released its custody. The engine owner still
    // preserves the original callback failure after the budget is restored.
    let retry = fixture.run("selectionGroup.disabled = true");
    let Err(DocumentScriptError::Script(Error::HostFailure(retry_failure))) = retry else {
        panic!("retry must return the first host failure: {retry:?}");
    };
    assert!(std::ptr::eq(
        failure.cause::<DomSourceDriveError>().unwrap(),
        retry_failure.cause::<DomSourceDriveError>().unwrap(),
    ));
    assert_eq!(fixture.host.snapshot().unwrap().stamp(), before);
    fixture.assert_disabled_attribute(None);
    fixture.assert_selection([false, false], None);
    drop((failure, retry_failure));
    fixture.finish();
}
