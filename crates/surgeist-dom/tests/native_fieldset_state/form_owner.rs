use super::{apply, cancel, owner};
use std::sync::Arc;
use surgeist_dom::{
    DomBatch, DomError, DomSnapshot,
    identity::NodeId,
    source::{DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest},
    style::DomStyleInlineCspObservation,
};
use surgeist_html::values::Namespace;

fn legend_owner(snapshot: &DomSnapshot, legend: NodeId) -> Option<NodeId> {
    snapshot
        .node(legend)
        .unwrap()
        .fieldset_parent_form_owner_with(|| Ok::<_, DomError>(()))
        .unwrap()
        .map(|form| form.id())
}

#[derive(Debug, PartialEq, Eq)]
enum ReadError {
    Refused,
    Dom(DomError),
}

impl From<DomError> for ReadError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

#[test]
fn legend_reads_the_stored_fieldset_owner_in_each_published_segment() {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "main");
    let ancestor = batch.create_element_ns(&document, Namespace::html(), "form");
    let external = batch.create_element_ns(&document, Namespace::html(), "form");
    batch.set_attribute(&external, "id", "external");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&ancestor, &root, None);
    batch.insert_before(&external, &root, None);

    let wrapper = batch.create_element_ns(&document, Namespace::html(), "div");
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let css = batch.create_text(&document, "a { color: red }");
    let fieldset = batch.create_element_ns(&document, Namespace::html(), "p:fieldset");
    let first = batch.create_element_ns(&document, Namespace::html(), "legend");
    let second = batch.create_element_ns(&document, Namespace::html(), "p:legend");
    let branch = batch.create_element_ns(&document, Namespace::html(), "div");
    let nested = batch.create_element_ns(&document, Namespace::html(), "legend");
    batch.set_attribute(&fieldset, "form", "external");
    batch.insert_before(&css, &style, None);
    batch.insert_before(&style, &wrapper, None);
    batch.insert_before(&fieldset, &wrapper, None);
    for child in [&first, &second, &branch] {
        batch.insert_before(child, &fieldset, None);
    }
    batch.insert_before(&nested, &branch, None);
    let commit = apply(&mut dom, batch);
    let [ancestor, external, wrapper, fieldset, first, second, nested] = [
        &ancestor, &external, &wrapper, &fieldset, &first, &second, &nested,
    ]
    .map(|ticket| commit.created(ticket).unwrap().unwrap());
    let detached = dom.snapshot();
    assert_eq!(legend_owner(&detached, first), None);

    let cancel = cancel();
    let limits = DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 64, 4096, 4096, 1);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(wrapper, ancestor, None),
            limits,
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!(
            "the earlier connected Style must reach its request before the Fieldset owner reset"
        );
    };
    let prefix = lease.snapshot();
    assert_eq!(
        prefix.incomplete_source_operation(),
        Some(lease.operation())
    );
    assert_eq!(prefix.node(wrapper).unwrap().parent(), Some(ancestor));
    assert_eq!(prefix.node(fieldset).unwrap().parent(), Some(wrapper));
    assert_eq!(prefix.node(first).unwrap().parent(), Some(fieldset));
    assert_eq!(
        prefix
            .node(fieldset)
            .unwrap()
            .as_fieldset()
            .unwrap()
            .form_owner(),
        Err(DomError::IncompleteSourceOperation),
    );
    let prefix_fieldset = prefix.node(fieldset).unwrap().as_fieldset().unwrap();
    let mut prefix_work = 0;
    assert!(
        prefix_fieldset
            .form_owner_with(|| {
                prefix_work += 1;
                Ok::<_, ReadError>(())
            })
            .unwrap()
            .is_none()
    );
    assert_eq!(prefix_work, 1);
    assert_eq!(legend_owner(&prefix, first), None);
    assert_eq!(legend_owner(&prefix, second), None);

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
    assert_eq!(
        current
            .node(fieldset)
            .unwrap()
            .as_fieldset()
            .unwrap()
            .form_owner(),
        Ok(Some(external))
    );
    assert_eq!(legend_owner(&current, first), Some(external));
    assert_eq!(legend_owner(&current, second), Some(external));
    assert_eq!(legend_owner(&current, nested), None);
    assert_eq!(legend_owner(&prefix, first), None);

    // The direct Fieldset accessor only reads the stored relation and its Form.
    let mut direct_required = 1usize;
    current
        .node_with(external, || {
            direct_required += 1;
            Ok::<_, ReadError>(())
        })
        .unwrap();
    let current_fieldset = current.node(fieldset).unwrap().as_fieldset().unwrap();
    for (allowance, expected) in [
        (direct_required, Ok(Some(external))),
        (direct_required - 1, Err(ReadError::Refused)),
    ] {
        let mut attempts = 0;
        let result = current_fieldset.form_owner_with(|| {
            attempts += 1;
            (attempts <= allowance)
                .then_some(())
                .ok_or(ReadError::Refused)
        });
        assert_eq!(result.map(|owner| owner.map(|form| form.id())), expected);
        assert_eq!(attempts, direct_required);
    }

    // Reaching the parent relation, Fieldset kind and stored owner adds one
    // debit apiece beyond the independently admitted parent and Form lookups.
    let mut required = 3usize;
    for reached in [fieldset, external] {
        current
            .node_with(reached, || {
                required += 1;
                Ok::<_, DomError>(())
            })
            .unwrap();
    }
    let legend = current.node(second).unwrap();
    for (allowance, expected) in [
        (required, Ok(Some(external))),
        (required - 1, Err(ReadError::Refused)),
        (0, Err(ReadError::Refused)),
    ] {
        let mut attempts = 0;
        let result = legend.fieldset_parent_form_owner_with(|| {
            attempts += 1;
            if attempts <= allowance {
                Ok(())
            } else {
                Err(ReadError::Refused)
            }
        });
        assert_eq!(result.map(|owner| owner.map(|form| form.id())), expected);
        assert_eq!(
            attempts,
            if allowance == required {
                required
            } else {
                allowance + 1
            }
        );
    }
}
