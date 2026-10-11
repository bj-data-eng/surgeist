#![forbid(unsafe_code)]

use surgeist_css::CssSpecifiedValueSerializationLimits;
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
            snapshot
                .container_condition_text(rule, expected.len())
                .unwrap(),
            expected,
            "the owning native provider is an independent passing control"
        );
    }
    assert_eq!(snapshot.container_query(&rules[2], 0).unwrap(), "");
    assert_eq!(snapshot.container_name(&rules[3], 0).unwrap(), "");
    assert_eq!(snapshot.container_name(&rules[1], 0).unwrap(), "");
    assert_eq!(snapshot.container_query(&rules[1], 0).unwrap(), "");
    assert!(matches!(
        snapshot.condition_text(&rules[4], CssSpecifiedValueSerializationLimits::default()),
        Err(CssomError::WrongKind)
    ));

    let actual: Vec<_> = rules[..4]
        .iter()
        .zip(expected)
        .map(|(rule, text)| {
            snapshot.condition_text(
                rule,
                CssSpecifiedValueSerializationLimits::new(1000, 1000, text.len()),
            )
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
