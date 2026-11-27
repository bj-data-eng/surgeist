use super::{apply, cancel, limits, select_completion_source_limits};
use std::sync::Arc;
use surgeist_dom::{
    Dom, DomBatch, DomError, DomSnapshot,
    identity::NodeId,
    source::{DomSourceEdit, DomSourceProgress, DomSourceRequest},
    style::DomStyleInlineCspObservation,
};
use surgeist_html::values::Namespace;

fn select_owner(snapshot: &DomSnapshot, select: NodeId) -> Option<NodeId> {
    snapshot
        .node(select)
        .unwrap()
        .as_select()
        .unwrap()
        .form_owner_with(|| Ok::<_, DomError>(()))
        .unwrap()
        .map(|form| form.id())
}

fn option_owner(snapshot: &DomSnapshot, option: NodeId) -> Option<NodeId> {
    snapshot
        .node(option)
        .unwrap()
        .as_option()
        .unwrap()
        .form_owner_with(|| Ok::<_, DomError>(()))
        .unwrap()
        .map(|form| form.id())
}

#[test]
fn style_cut_exposes_option_topology_and_only_the_form_owner_hooks_already_reached() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let form = batch.create_element_ns(&document, Namespace::html(), "form");
    let outer_select = batch.create_element_ns(&document, Namespace::html(), "select");
    batch.insert_before(&form, &document, None);
    batch.insert_before(&outer_select, &form, None);

    let wrapper = batch.create_element_ns(&document, Namespace::html(), "div");
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let css = batch.create_text(&document, "a { color: red }");
    let outer_option = batch.create_element_ns(&document, Namespace::html(), "option");
    let inner_select = batch.create_element_ns(&document, Namespace::html(), "select");
    let inner_option = batch.create_element_ns(&document, Namespace::html(), "option");
    batch.insert_before(&css, &style, None);
    for child in [&style, &outer_option, &inner_select] {
        batch.insert_before(child, &wrapper, None);
    }
    batch.insert_before(&inner_option, &inner_select, None);
    let commit = apply(&mut dom, batch);
    let [
        form,
        outer_select,
        wrapper,
        outer_option,
        inner_select,
        inner_option,
    ] = [
        &form,
        &outer_select,
        &wrapper,
        &outer_option,
        &inner_select,
        &inner_option,
    ]
    .map(|created| commit.created(created).unwrap().unwrap());
    let before = dom.snapshot();
    assert_eq!(option_owner(&before, outer_option), None);
    assert_eq!(select_owner(&before, inner_select), None);

    let cancel = cancel();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(wrapper, outer_select, None),
            select_completion_source_limits(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the first connected descendant must reach its Style request")
    };
    let prefix = lease.snapshot();
    assert_eq!(prefix.node(wrapper).unwrap().parent(), Some(outer_select));
    assert_eq!(
        prefix.incomplete_source_operation(),
        Some(lease.operation())
    );
    assert_eq!(
        prefix
            .node(outer_option)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select(),
        Err(DomError::IncompleteSourceOperation)
    );
    assert_eq!(
        prefix
            .node(inner_select)
            .unwrap()
            .as_select()
            .unwrap()
            .form_owner(),
        Err(DomError::IncompleteSourceOperation)
    );
    assert_eq!(select_owner(&prefix, outer_select), Some(form));
    assert_eq!(option_owner(&prefix, outer_option), Some(form));
    assert_eq!(select_owner(&prefix, inner_select), None);
    assert_eq!(option_owner(&prefix, inner_option), None);

    let observation = DomStyleInlineCspObservation::evaluated(
        request.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    );
    lease
        .accept_inline_csp(&request, &observation, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    let current = lease.snapshot();
    assert_eq!(select_owner(&current, inner_select), Some(form));
    assert_eq!(option_owner(&current, inner_option), Some(form));
    assert_eq!(option_owner(&prefix, inner_option), None);
}

#[test]
fn sparse_option_insert_resumes_the_original_donor_at_a_style_request() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    let donor = batch.create_element_ns(&document, Namespace::html(), "option");
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let css = batch.create_text(&document, "a { color: red }");
    batch.insert_before(&select, &document, None);
    batch.insert_before(&style, &donor, None);
    batch.insert_before(&css, &style, None);
    let commit = apply(&mut dom, batch);
    let [select, donor, style] =
        [select, donor, style].map(|node| commit.created(&node).unwrap().unwrap());
    let cancellation = cancel();
    let limits = select_completion_source_limits();

    let (request, operation, prefix) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_select_option_at(select, 2, Some(donor)),
                limits,
                &cancellation,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancellation).unwrap()
        else {
            panic!("the original sparse donor must reach its inline Style request");
        };
        (request, lease.operation(), lease.snapshot())
    };
    let originals = prefix.node(select).unwrap().children().to_vec();
    assert_eq!(originals.len(), 3);
    assert_eq!(originals[2], donor);
    assert_eq!(prefix.node(style).unwrap().parent(), Some(donor));
    assert_eq!(prefix.incomplete_source_operation(), Some(operation));

    let mut lease = dom
        .resume_source_edit(operation, limits, &cancellation)
        .unwrap();
    let observation = DomStyleInlineCspObservation::evaluated(
        request.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    );
    lease
        .accept_inline_csp(&request, &observation, &cancellation)
        .unwrap();
    let DomSourceProgress::Complete(completion) = lease.advance(&cancellation).unwrap() else {
        panic!("the accepted Style receipt must complete the original append");
    };
    assert_eq!(completion.operation(), operation);
    let live = lease.snapshot();
    assert_eq!(live.node(select).unwrap().children(), &originals);
    assert!(live.incomplete_source_operation().is_none());
}

#[test]
fn option_form_owner_respects_html_barriers_and_form_retirement() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "main");
    let ancestor = batch.create_element_ns(&document, Namespace::html(), "form");
    let external = batch.create_element_ns(&document, Namespace::html(), "form");
    batch.set_attribute(&external, "id", "external");
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    batch.set_attribute(&select, "form", "external");
    batch.set_attribute(&select, "multiple", "");
    let group = batch.create_element_ns(&document, Namespace::html(), "optgroup");
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&ancestor, &root, None);
    batch.insert_before(&external, &root, None);
    batch.insert_before(&select, &ancestor, None);
    batch.insert_before(&group, &select, None);
    batch.insert_before(&option, &group, None);
    let blocked = ["datalist", "hr", "option", "optgroup"].map(|tag| {
        let barrier = batch.create_element_ns(&document, Namespace::html(), tag);
        let option = batch.create_element_ns(&document, Namespace::html(), "option");
        let parent = if tag == "optgroup" { &group } else { &select };
        batch.insert_before(&barrier, parent, None);
        batch.insert_before(&option, &barrier, None);
        option
    });
    let commit = apply(&mut dom, batch);
    let [ancestor, external, select, option] = [&ancestor, &external, &select, &option]
        .map(|created| commit.created(created).unwrap().unwrap());
    let before = dom.snapshot();
    assert_eq!(select_owner(&before, select), Some(external));
    assert_eq!(option_owner(&before, option), Some(external));
    for creation in blocked {
        let blocked = commit.created(&creation).unwrap().unwrap();
        assert_eq!(option_owner(&before, blocked), None);
    }

    let view = before.node(select).unwrap().as_select().unwrap();
    let mut lookup_steps = 0;
    before
        .node_with(external, || {
            lookup_steps += 1;
            Ok::<_, DomError>(())
        })
        .unwrap();
    // One association read plus the same canonical lookup used for any Node.
    for (budget, expected) in [
        (lookup_steps, Err(DomError::Cancelled)),
        (lookup_steps + 1, Ok(Some(external))),
    ] {
        let mut spent = 0;
        let result = view.form_owner_with(|| {
            if spent == budget {
                return Err(DomError::Cancelled);
            }
            spent += 1;
            Ok(())
        });
        assert_eq!(result.map(|owner| owner.map(|form| form.id())), expected);
        assert_eq!(spent, budget);
    }
    let option_view = before.node(option).unwrap().as_option().unwrap();
    assert_eq!(
        option_view
            .form_owner_with(|| Err(DomError::Cancelled))
            .unwrap_err(),
        DomError::Cancelled
    );

    let mut retire = DomBatch::new();
    retire.detach(external);
    retire.destroy(external);
    apply(&mut dom, retire);
    let retired = dom.snapshot();
    assert_eq!(retired.node(external).unwrap_err(), DomError::StaleNode);
    assert_eq!(select_owner(&retired, select), None);
    assert_eq!(option_owner(&retired, option), None);
    assert_eq!(option_owner(&before, option), Some(external));

    let mut remove_explicit = DomBatch::new();
    remove_explicit.remove_attribute(select, "form");
    apply(&mut dom, remove_explicit);
    let current = dom.snapshot();
    assert_eq!(select_owner(&current, select), Some(ancestor));
    assert_eq!(option_owner(&current, option), Some(ancestor));
}
