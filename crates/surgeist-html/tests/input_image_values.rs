//! Literal HTML877d Image Button source/history/guard expectations.
use surgeist_html::{
    DocumentString, HtmlInputImageHistory as History, HtmlInputImageUpdateGuard as Guard,
    HtmlInputImagesDisabled,
};

#[test]
fn first_entry_absence_empty_and_later_same_presence_are_distinct() {
    let never = History::<DocumentString>::NeverEntered;
    assert_eq!(never.observed_source(), None);
    assert_eq!((never.coordinate().x(), never.coordinate().y()), (0, 0));
    assert!(
        never
            .entering_requires_observation::<()>(None, |_, _| panic!("no comparison"))
            .unwrap()
    );
    let absent = never.with_observed_source::<DocumentString>(None);
    assert_eq!(absent.observed_source(), Some(None));
    assert!(
        !absent
            .entering_requires_observation::<()>(None, |_, _| panic!("no comparison"))
            .unwrap()
    );
    let empty = DocumentString::from("");
    assert!(
        absent
            .entering_requires_observation::<()>(Some(&empty), |_, _| panic!("presence differs"))
            .unwrap()
    );
    let specified_empty = absent.with_observed_source(Some(empty.clone()));
    assert!(
        specified_empty
            .entering_requires_observation::<()>(None, |_, _| panic!("presence differs"))
            .unwrap()
    );
    assert!(
        !specified_empty
            .entering_requires_observation(Some(&empty), |a, b| Ok::<_, ()>(
                a.code_units() == b.code_units()
            ))
            .unwrap()
    );
    assert_eq!(
        (
            specified_empty.coordinate().x(),
            specified_empty.coordinate().y()
        ),
        (0, 0)
    );
}

#[test]
fn exact_reentry_comparison_is_owned_fallible_and_does_not_record_implicitly() {
    let a = DocumentString::from_utf16(vec![0xd800, 65, 0xdc00]);
    let b = DocumentString::from_utf16(vec![0xd800, 66, 0xdc00]);
    let old = History::<DocumentString>::NeverEntered.with_observed_source(Some(a.clone()));
    let mut reads = 0;
    assert!(
        old.entering_requires_observation(Some(&b), |previous, current| {
            reads += 1;
            assert_eq!(previous.code_units(), &[0xd800, 65, 0xdc00]);
            assert_eq!(current.code_units(), &[0xd800, 66, 0xdc00]);
            Ok::<_, ()>(previous.code_units() == current.code_units())
        })
        .unwrap()
    );
    assert_eq!(reads, 1);
    assert_eq!(old.observed_source(), Some(Some(&a)));
    assert_eq!(
        old.entering_requires_observation(Some(&b), |_, _| Err::<bool, _>("cancelled")),
        Err("cancelled")
    );
    assert_eq!(old.observed_source(), Some(Some(&a)));
    let new = old.with_observed_source(Some(b.clone()));
    assert_eq!(new.observed_source(), Some(Some(&b)));
}

#[test]
fn source_guards_precede_policy_and_disabled_never_means_failed_fetch() {
    let disabled = HtmlInputImagesDisabled::ExplicitlyDisabled;
    assert_eq!(Guard::for_source(None), Guard::NoSource);
    assert_eq!(
        Guard::for_source(Some(&DocumentString::from(""))),
        Guard::EmptySource
    );
    let nonempty = DocumentString::from("not a URL\u{0} with spaces");
    let guard = Guard::for_source(Some(&nonempty));
    assert_eq!(guard, Guard::NeedsImageEnvironment);
    assert_eq!(guard.with_disabled_images(disabled), Guard::ImagesDisabled);
    assert_eq!(
        Guard::NoSource.with_disabled_images(disabled),
        Guard::NoSource
    );
    assert_eq!(
        Guard::EmptySource.with_disabled_images(disabled),
        Guard::EmptySource
    );
}
