#![forbid(unsafe_code)]

use surgeist_css::CssSerializationLimits;
use surgeist_cssom::{
    CssomContext, CssomEditGuard, CssomError, CssomInputRole, CssomInputVersion, CssomLimits,
    CssomSheetInputs, CssomStore,
};

// Conditional 5, selected editor capture §9.1: conditionText joins each
// ordered condition's name/query with a space only when both are nonempty,
// then joins the conditions with ", ". The inherited getter must dispatch
// Container to that algorithm, without evaluating or simplifying a query.
#[test]
fn inherited_container_condition_text_preserves_order_duplicates_and_empty_fields() {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let ticket = batch
        .create_parsed_sheet(
            "@container card (width > 1px){} \
             @container alpha, (width > 1px), alpha, beta future(foo){} \
             @container name-only{} @container (width > 1px){} .style{}",
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "document".into(),
                    revision: 1,
                },
                "document".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = store.commit(batch).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let rules = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap();
    assert_eq!(rules.len(), 5);
    let expected = [
        "card (width > 1px)",
        "alpha, (width > 1px), alpha, beta future(foo)",
        "name-only",
        "(width > 1px)",
    ];
    for (rule, expected) in rules.iter().zip(expected) {
        assert_eq!(
            store
                .css_container_rule(rule)
                .and_then(|handle| handle.capture(&snapshot))
                .and_then(
                    |capture| capture.condition_text(CssSerializationLimits::new(
                        usize::MAX,
                        usize::MAX,
                        expected.len()
                    ))
                )
                .unwrap(),
            expected,
            "the owning native provider is an independent passing control"
        );
    }
    assert_eq!(
        store
            .css_container_rule(&rules[2])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.container_query(0))
            .unwrap(),
        ""
    );
    assert_eq!(
        store
            .css_container_rule(&rules[3])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.container_name(0))
            .unwrap(),
        ""
    );
    assert_eq!(
        store
            .css_container_rule(&rules[1])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.container_name(0))
            .unwrap(),
        ""
    );
    assert_eq!(
        store
            .css_container_rule(&rules[1])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.container_query(0))
            .unwrap(),
        ""
    );
    assert!(matches!(
        store
            .css_condition_rule(&rules[4])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.condition_text(CssSerializationLimits::default())),
        Err(CssomError::WrongKind)
    ));

    let actual: Vec<_> = rules[..4]
        .iter()
        .zip(expected)
        .map(|(rule, text)| {
            store
                .css_condition_rule(rule)
                .and_then(|handle| handle.capture(&snapshot))
                .and_then(|capture| {
                    capture.condition_text(CssSerializationLimits::new(1000, 1000, text.len()))
                })
        })
        .collect();
    assert_eq!(snapshot.revision(), store.snapshot().revision());
    assert!(
        actual
            .iter()
            .zip(expected)
            .all(|(result, expected)| result.as_ref().is_ok_and(|text| text == expected)),
        "inherited Container conditionText must use its selected algorithm: \
         expected {expected:?}, actual {actual:?}"
    );
}
