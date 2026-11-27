//! Literal native File Upload state through actual Source assignment and reset.

use super::{Dom, DomBatch, DomError, Namespace, NodeId, apply, owner, state, value};
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    file::{DomFile, DomFileList},
    limits::DomResource,
    source::{DomSourceEdit, DomSourceLimits, DomSourceProgress},
};

fn file(bytes: Vec<u8>) -> DomFile {
    DomFile::from_owned_parts("first.txt".into(), "text/plain".into(), bytes).unwrap()
}

fn setup() -> (Dom, NodeId, NodeId, NodeId) {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let one = batch.create_element_ns(&doc, Namespace::html(), "input");
    let two = batch.create_element_ns(&doc, Namespace::html(), "input");
    let text = batch.create_element_ns(&doc, Namespace::html(), "input");
    for item in [&one, &two] {
        batch.set_attribute(item, "type", "file");
        batch.set_attribute(item, "multiple", "");
    }
    let commit = apply(&mut dom, batch);
    let [one, two, text] = [one, two, text].map(|item| commit.created(&item).unwrap().unwrap());
    (dom, one, two, text)
}

fn assign(
    dom: &mut Dom,
    input: NodeId,
    files: Option<DomFileList>,
    bytes: usize,
) -> Result<DomSourceProgress, DomError> {
    let cancel = AtomicBool::new(false);
    dom.begin_source_edit(
        DomSourceEdit::assign_input_files(input, files),
        DomSourceLimits::new(bytes, 100_000, 2_000_000, 32, 256, 256, 2),
        &cancel,
    )?
    .advance(&cancel)
}

#[test]
fn file_list_assignment_preserves_exact_aliases_and_actually_selected_value() {
    let (mut dom, one, two, text) = setup();
    let initial = dom.snapshot();
    let empty_one = state(&initial, one).selected_files().unwrap().id();
    let empty_two = state(&initial, two).selected_files().unwrap().id();
    assert_ne!(empty_one, empty_two);
    assert!(state(&initial, one).selected_files().unwrap().is_empty());
    assert!(state(&initial, text).selected_files().is_none());
    let first = file(vec![4, 5]);
    let second = DomFile::from_owned_parts("other.txt".into(), "".into(), vec![9]).unwrap();
    let chosen =
        DomFileList::from_files(vec![first.clone(), second.clone(), first.clone()]).unwrap();
    for target in [one, two] {
        let result = assign(&mut dom, target, Some(chosen.clone()), 16_000_000).unwrap();
        assert!(matches!(result, DomSourceProgress::Complete(_)));
    }
    let selected = dom.snapshot();
    for target in [one, two] {
        let actual = state(&selected, target).selected_files().unwrap();
        assert_eq!(actual.id(), chosen.id());
        assert_eq!(
            actual.files(),
            &[first.clone(), second.clone(), first.clone()]
        );
        assert_eq!(value(&selected, target), r"C:\fakepath\first.txt".into());
    }
    assign(&mut dom, one, None, 16_000_000).unwrap();
    assign(&mut dom, one, Some(chosen.clone()), 16_000_000).unwrap();
    assign(&mut dom, text, Some(chosen.clone()), 16_000_000).unwrap();
    assert_eq!(dom.snapshot().stamp(), selected.stamp());

    let mut refused = DomBatch::new();
    refused.set_input_value(one, "other.txt");
    assert!(matches!(
        dom.apply(refused, &AtomicBool::new(false)),
        Err(DomError::InputFilenameInvalidState)
    ));
    assert_eq!(dom.snapshot().stamp(), selected.stamp());

    let mut clear = DomBatch::new();
    clear.set_input_value(one, "");
    apply(&mut dom, clear);
    let current = dom.snapshot();
    let emptied = state(&current, one).selected_files().unwrap();
    assert!(emptied.is_empty());
    assert_ne!(emptied.id(), chosen.id());
    assert_eq!(value(&current, one), "".into());
    assert_eq!(
        state(&current, two).selected_files().unwrap().id(),
        chosen.id()
    );
    assert_eq!(
        state(&selected, one).selected_files().unwrap().id(),
        chosen.id()
    );
    assert_eq!(
        state(&initial, one).selected_files().unwrap().id(),
        empty_one
    );
}

#[test]
fn custody_reuses_one_backing_for_exact_aliases_and_refuses_distinct_unfunded_bytes() {
    let (mut dom, one, two, _) = setup();
    const BODY: usize = 1_000_000;
    const POLICY: usize = 1_500_000;
    let resource = file(vec![4; BODY]);
    let shared = DomFileList::from_files(vec![resource.clone(), resource]).unwrap();
    assign(&mut dom, one, Some(shared.clone()), POLICY).unwrap();
    assign(&mut dom, two, Some(shared.clone()), POLICY).unwrap();
    let old = dom.snapshot();
    let usage = dom.usage();
    let independent = DomFileList::from_files(vec![file(vec![4; BODY])]).unwrap();
    assert!(matches!(
        assign(&mut dom, two, Some(independent.clone()), POLICY),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            limit: POLICY
        })
    ));
    super::unchanged(&dom, &old, usage);
    assign(&mut dom, two, Some(independent.clone()), 3_000_000).unwrap();
    assert_eq!(
        state(&dom.snapshot(), two).selected_files().unwrap().id(),
        independent.id()
    );
    assert_eq!(state(&old, two).selected_files().unwrap().id(), shared.id());
}

#[test]
fn adoption_preserves_files_while_clone_reset_type_change_and_retirement_keep_exact_lifetimes() {
    let (mut dom, input, _, _) = setup();
    let selected = DomFileList::from_files(vec![file(vec![42])]).unwrap();
    assign(&mut dom, input, Some(selected.clone()), 16_000_000).unwrap();
    let having_file = dom.snapshot();

    let mut batch = DomBatch::new();
    let document = batch.create_document();
    batch.adopt(input, &document);
    let clone = batch.clone_node(input, false);
    let commit = apply(&mut dom, batch);
    let clone = commit.created(&clone).unwrap().unwrap();
    let adopted = dom.snapshot();
    assert_eq!(
        state(&adopted, input).selected_files().unwrap().id(),
        selected.id()
    );
    let cloned_files = state(&adopted, clone).selected_files().unwrap();
    assert!(cloned_files.is_empty());
    assert_ne!(cloned_files.id(), selected.id());
    assert_eq!(
        adopted.node(input).unwrap().owner_document(),
        Some(commit.document(&document).unwrap())
    );

    let mut batch = DomBatch::new();
    batch.reset_input(input);
    apply(&mut dom, batch);
    let reset = dom.snapshot();
    let cleared = state(&reset, input).selected_files().unwrap();
    assert!(cleared.is_empty());
    assert_ne!(cleared.id(), selected.id());

    let mut batch = DomBatch::new();
    batch.set_attribute(input, "type", "text");
    apply(&mut dom, batch);
    assert!(state(&dom.snapshot(), input).selected_files().is_none());
    let mut batch = DomBatch::new();
    batch.set_attribute(input, "type", "file");
    apply(&mut dom, batch);
    let reentered = dom.snapshot();
    assert!(
        state(&reentered, input)
            .selected_files()
            .unwrap()
            .is_empty()
    );
    assert_ne!(
        state(&reentered, input).selected_files().unwrap().id(),
        cleared.id()
    );

    assign(&mut dom, input, Some(selected.clone()), 16_000_000).unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(input);
    apply(&mut dom, batch);
    assert!(matches!(
        dom.snapshot().node(input),
        Err(DomError::StaleNode)
    ));
    assert_eq!(
        state(&having_file, input).selected_files().unwrap().id(),
        selected.id()
    );
    assert_eq!(
        state(&having_file, input).selected_files().unwrap().files()[0].bytes(),
        &[42]
    );
}
