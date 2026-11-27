//! Native selected objects have identity independent of names and file bodies.

use surgeist_dom::{
    DomError,
    file::{DomFile, DomFileList},
};

#[test]
fn equal_bodies_and_names_do_not_merge_objects_or_ordered_lists() {
    let file = || {
        DomFile::from_owned_parts("leaf.txt".into(), "text/plain".into(), vec![0, 4, 255]).unwrap()
    };
    let first = file();
    let second = file();
    assert_eq!(first.name(), second.name());
    assert_eq!(first.media_type(), second.media_type());
    assert_eq!(first.bytes(), &[0, 4, 255]);
    assert_eq!(first.bytes(), second.bytes());
    assert_ne!(first.id(), second.id());
    assert_ne!(first, second);
    assert_eq!(first, first.clone());

    let members = vec![first.clone(), second.clone(), first.clone()];
    let list = DomFileList::from_files(members.clone()).unwrap();
    let independent = DomFileList::from_files(members).unwrap();
    assert_eq!(list.files(), &[first.clone(), second, first]);
    assert_eq!(list.files(), independent.files());
    assert_ne!(list.id(), independent.id());
    assert_ne!(list, independent);
    assert_eq!(list, list.clone());
    assert_eq!(list.len(), 3);
    let readback = String::from_utf16(list.filename_value()).unwrap();
    assert_eq!(readback, r"C:\fakepath\leaf.txt");
}

#[test]
fn empty_native_lists_are_actual_distinct_objects_and_paths_are_refused() {
    let first = DomFileList::from_files(Vec::new()).unwrap();
    let second = DomFileList::from_files(Vec::new()).unwrap();
    assert!(first.is_empty());
    assert!(first.filename_value().is_empty());
    assert_ne!(first.id(), second.id());
    assert!(matches!(
        DomFile::from_owned_parts(r"chosen\leaf.txt".into(), "".into(), vec![0]),
        Err(DomError::InputFileNameContainsPath)
    ));
}
