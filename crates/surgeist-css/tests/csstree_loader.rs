mod support;

use support::csstree::load_csstree_corpus;

#[test]
fn csstree_inventory_matches_pinned_revision() {
    let corpus = load_csstree_corpus().expect("committed CSSTree corpus should validate");
    let cached = load_csstree_corpus().expect("validated corpus should remain cached");

    assert!(std::ptr::eq(corpus, cached));
    assert_eq!(corpus.artifact_count(), 74);
    assert_eq!(corpus.case_count(), 935);
    assert_eq!(corpus.parsed_count(), 721);
    assert_eq!(corpus.rejected_count(), 214);
    assert_eq!(
        corpus.context_counts(),
        [
            ("atrule", 130),
            ("atrulePrelude", 2),
            ("block", 29),
            ("declaration", 77),
            ("declarationList", 19),
            ("mediaQuery", 49),
            ("rule", 33),
            ("selector", 317),
            ("selectorList", 10),
            ("stylesheet", 76),
            ("value", 193),
        ]
    );
}
