#![forbid(unsafe_code)]
//! Authored admission is distinct from definition validity in Counter Styles 2026.
use surgeist_css::{CssRule, parse_sheet};

#[test]
fn extends_symbol_conflict_remains_inspectable_authored_syntax() {
    let report = parse_sheet("@counter-style example { system: extends decimal; symbols: x; }");
    assert!(
        report.is_clean(),
        "definition conflict does not invalidate authored grammar: {report:?}"
    );
    let CssRule::CounterStyle(rule) = &report.syntax().rules()[0] else {
        panic!("retained counter rule");
    };
    assert!(rule.descriptors().symbols().is_some());
}

use surgeist_css::{
    CssCounterStyleAlgorithm as Algorithm, CssCounterStyleDefinitionIssue as Issue,
    CssCounterStyleDefinitionStatus as Status, CssCounterStyleDescriptorCollection as Collection,
    CssCounterStyleDescriptorEntry as Entry, CssCounterStyleDescriptorKind as K,
    CssSpecifiedValueSerializationLimits as Limits, parse_counter_style_descriptor_value,
};
fn value(source: &str, kind: K) -> surgeist_css::CssCounterStyleDescriptorValue {
    let report = parse_counter_style_descriptor_value(source, kind);
    assert!(report.is_clean(), "{report:?}");
    report.syntax().clone().unwrap()
}
#[test]
fn prospective_numeric_edit_preserves_authored_occurrence_and_raw_edit_origin() {
    let report = parse_sheet("@counter-style sample { system:numeric; symbols:a; symbols:b; }");
    assert!(report.is_clean(), "{report:?}");
    let CssRule::CounterStyle(rule) = &report.syntax().rules()[0] else {
        panic!()
    };
    let prospective = rule.descriptors().prospective().unwrap();
    assert_eq!(
        prospective.definition_status(),
        Status::Undefined(Issue::InsufficientSymbols {
            required: 2,
            actual: 1
        })
    );
    assert!(
        prospective.entries()[0]
            .occurrence()
            .unwrap()
            .same_occurrence(rule.descriptors().system().unwrap())
    );
    let edited = value(" /*edit*/ a b ", K::Symbols);
    let mut entries = rule.descriptors().entries().collect::<Vec<_>>();
    entries.push(Entry::from_value(&edited));
    let next = Collection::try_new(entries).unwrap();
    assert_eq!(next.entries().len(), 4);
    assert_eq!(next.definition_status(), Status::Defined);
    assert!(next.effective(K::Symbols).unwrap().occurrence().is_none());
    assert_eq!(
        next.effective(K::Symbols)
            .unwrap()
            .value()
            .origin()
            .unwrap()
            .source()
            .as_str(),
        " /*edit*/ a b "
    );
    assert_eq!(
        prospective.definition_status(),
        Status::Undefined(Issue::InsufficientSymbols {
            required: 2,
            actual: 1
        })
    );
}
#[test]
fn algorithm_identity_ignores_fixed_start_and_extends_target() {
    for (left, right, algorithm) in [
        ("fixed 1", "fixed 20", Algorithm::Fixed),
        ("extends a", "extends b", Algorithm::Extends),
    ] {
        let a = value(left, K::System);
        let b = value(right, K::System);
        assert_eq!(
            Collection::try_new(vec![Entry::from_value(&a)])
                .unwrap()
                .algorithm(),
            Some(algorithm)
        );
        assert_eq!(
            Collection::try_new(vec![Entry::from_value(&b)])
                .unwrap()
                .algorithm(),
            Some(algorithm)
        );
    }
}
#[test]
fn definition_admission_handles_missing_symbols_extends_and_additive_symbolic_weights() {
    for (system, symbols, expected) in [
        (
            "cyclic",
            None,
            Status::Undefined(Issue::InsufficientSymbols {
                required: 1,
                actual: 0,
            }),
        ),
        ("alphabetic", Some("a b"), Status::Defined),
        (
            "extends decimal",
            Some("x"),
            Status::Undefined(Issue::ExtendsWithSymbols),
        ),
        (
            "additive",
            None,
            Status::Undefined(Issue::MissingAdditiveSymbols),
        ),
    ] {
        let system = value(system, K::System);
        let symbols = symbols.map(|s| value(s, K::Symbols));
        let mut entries = vec![Entry::from_value(&system)];
        if let Some(s) = &symbols {
            entries.push(Entry::from_value(s));
        }
        assert_eq!(
            Collection::try_new(entries).unwrap().definition_status(),
            expected
        );
    }
    let system = value("additive", K::System);
    let symbols = value("calc(2) X, 1 I", K::AdditiveSymbols);
    assert_eq!(
        Collection::try_new(vec![
            Entry::from_value(&system),
            Entry::from_value(&symbols)
        ])
        .unwrap()
        .definition_status(),
        Status::RequiresResolution
    );
    let pending = value("env(counter-symbols)", K::Symbols);
    assert_eq!(
        Collection::try_new(vec![Entry::from_value(&pending)])
            .unwrap()
            .definition_status(),
        Status::SubstitutionDependent
    );
    assert!(
        parse_counter_style_descriptor_value("1 A, 2 B", K::AdditiveSymbols)
            .syntax()
            .is_none()
    );
}
#[test]
fn prospective_collection_limits_are_cumulative_and_retry_is_unchanged() {
    let a = value("numeric", K::System);
    let b = value("a b", K::Symbols);
    let entries = vec![Entry::from_value(&a), Entry::from_value(&b)];
    assert!(Collection::try_new_with_limits(entries.clone(), Limits::new(100, 100, 7)).is_err());
    for limits in [Limits::new(1, 100, 100), Limits::new(100, 1, 100)] {
        assert!(Collection::try_new_with_limits(entries.clone(), limits).is_err());
        assert_eq!(
            Collection::try_new(entries.clone())
                .unwrap()
                .definition_status(),
            Status::Defined
        );
    }
    assert_eq!(a.origin().unwrap().source().as_str(), "numeric");
}

#[test]
fn programmatic_prospective_values_do_not_invent_a_named_or_raw_source_origin() {
    let components = surgeist_css::CssComponentValues::try_new(vec![
        surgeist_css::CssComponentValue::try_ident("cyclic").unwrap(),
    ])
    .unwrap();
    let system =
        surgeist_css::CssCounterStyleDescriptorValue::try_from_components(K::System, components)
            .unwrap();
    let collection = Collection::try_new(vec![Entry::from_value(&system)]).unwrap();
    assert!(collection.entries()[0].occurrence().is_none());
    assert!(collection.entries()[0].value().origin().is_none());
    assert_eq!(collection.algorithm(), Some(Algorithm::Cyclic));
    assert_eq!(
        collection.definition_status(),
        Status::Undefined(Issue::InsufficientSymbols {
            required: 1,
            actual: 0
        })
    );
}
