#![forbid(unsafe_code)]
//! Authored feature-index policy selected from Fonts 4 WD20260907 §6.9.2
//! and frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d:
//! Source/WebCore/css/parser/CSSParser.cpp:800–885 (max counts and generic
//! nonnegative integers). CV admits one/two values; styleset admits one/many.
//! Neither has a feature-specific authored upper bound. Font activation is
//! downstream. Fonts 4 §6.9.1's conflicting two-value/range clauses remain
//! unresolved; these tests select the explicit authored policy, not a claim
//! that the selected edition is internally consistent.
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#multi-value-features

use surgeist_css::*;

type Block = CssFontFeatureValueBlock;
type Definition = CssFontFeatureValueDefinition;
type Index = CssFontFeatureValueIndex;
type Kind = CssFontFeatureValueKind;
type ConstructionError = CssFontFeatureValuesErrorKind;

fn definition(name: &str, values: &[&str]) -> Definition {
    Definition::try_new(
        CssFontFeatureValueName::try_new(name).unwrap(),
        values
            .iter()
            .map(|value| Index::try_from_decimal(value).unwrap())
            .collect(),
    )
    .unwrap()
}

fn block(item: &CssFontFeatureValuesItem) -> &Block {
    let CssFontFeatureValuesItem::Block(value) = item else {
        panic!("feature block")
    };
    value
}

fn indexes(value: &Definition) -> Vec<&str> {
    value.indexes().iter().map(Index::as_decimal_str).collect()
}

fn assert_checked(kind: Kind, authored: &[&str], expected: &[&str]) {
    let definition = definition("Case", authored);
    let before = definition.clone();
    let value = Block::try_new(kind, vec![definition]).unwrap();
    assert_eq!(value.kind(), kind);
    assert!(value.position().is_none());
    assert_eq!(value.definitions(), &[before]);
    assert_eq!(indexes(&value.definitions()[0]), expected);
    assert!(value.definitions()[0].position().is_none());
    assert!(
        value.definitions()[0]
            .indexes()
            .iter()
            .all(|index| index.origin().is_none())
    );
}

fn assert_parsed(kind: Kind, authored: &[&str], expected: &[&str]) {
    let source = format!(
        "/*😀*/ @font-feature-values Demo {{ @{} {{ Case: {}; }} }}",
        kind.css_name(),
        authored.join(" "),
    );
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert!(validate_sheet(&source).is_ok(), "strict validation agrees");
    let [CssRule::FontFeatureValues(rule)] = report.syntax().rules() else {
        panic!("one feature-values rule")
    };
    let [item] = rule.items() else {
        panic!("one retained block")
    };
    let value = block(item);
    assert_eq!(value.kind(), kind);
    let [definition] = value.definitions() else {
        panic!("one admitted definition")
    };
    assert_eq!(definition.name().as_str(), "Case");
    assert_eq!(indexes(definition), expected);
    assert_eq!(
        definition.position().unwrap().byte_offset().value(),
        source.find("Case:").unwrap()
    );
    assert_eq!(
        rule.position().unwrap().byte_offset().value(),
        source.find("@font-feature-values").unwrap()
    );
    for (index, spelling) in definition.indexes().iter().zip(authored) {
        let origin = index.origin().unwrap();
        assert_eq!(origin.source().as_str(), source);
        let span = origin.span();
        assert_eq!(
            &source[span.start().byte_offset().value()..span.end().byte_offset().value()],
            *spelling,
        );
    }
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let [CssNormalizedItem::Rule(context)] = normalized.items() else {
        panic!("one normalized rule")
    };
    let CssRuleContextKindRef::FontFeatureValues(payload) = context.kind() else {
        panic!("normalized feature-values payload")
    };
    assert_eq!(payload, rule);
}

#[test]
fn parsed_character_variant_admits_a_single_index_without_fabricating_a_second() {
    for (authored, expected) in [("1", "1"), ("-000", "0"), ("+00099", "99")] {
        assert_parsed(Kind::CharacterVariant, &[authored], &[expected]);
    }
}

#[test]
fn checked_character_variant_admits_a_single_index_without_fabricating_a_second() {
    for (authored, expected) in [("1", "1"), ("-000", "0"), ("+00099", "99")] {
        assert_checked(Kind::CharacterVariant, &[authored], &[expected]);
    }
}

#[test]
fn two_character_variant_indexes_keep_zero_and_endpoint_values_in_order() {
    for (authored, expected) in [
        (["0", "0"], ["0", "0"]),
        (["+00099", "4294967296"], ["99", "4294967296"]),
    ] {
        assert_parsed(Kind::CharacterVariant, &authored, &expected);
        assert_checked(Kind::CharacterVariant, &authored, &expected);
    }
}

#[test]
fn parsed_character_variant_first_index_has_no_feature_activation_upper_bound() {
    assert_parsed(Kind::CharacterVariant, &["+000100", "1"], &["100", "1"]);
    assert_parsed(Kind::CharacterVariant, &["100"], &["100"]);
}

#[test]
fn checked_character_variant_first_index_has_no_feature_activation_upper_bound() {
    assert_checked(Kind::CharacterVariant, &["+000100", "1"], &["100", "1"]);
    assert_checked(Kind::CharacterVariant, &["100"], &["100"]);
}

#[test]
fn parsed_styleset_admits_nonnegative_indexes_above_the_opentype_feature_ranges() {
    for values in [
        &["21"][..],
        &["100"][..],
        &["0", "20", "21", "100", "21"][..],
    ] {
        assert_parsed(Kind::Styleset, values, values);
    }
}

#[test]
fn checked_styleset_admits_nonnegative_indexes_above_the_opentype_feature_ranges() {
    for values in [
        &["21"][..],
        &["100"][..],
        &["0", "20", "21", "100", "21"][..],
    ] {
        assert_checked(Kind::Styleset, values, values);
    }
}

#[test]
fn exact_large_authored_indexes_survive_parsing_and_checked_block_admission() {
    // Exact authored storage is the existing owner contract. Frozen WebKit's
    // machine integer representation is not an authored magnitude restriction.
    let magnitude = "98765432101234567890".repeat(20);
    let spelling = format!("+000{magnitude}");
    let index = Index::try_from_decimal(&spelling).unwrap();
    assert_eq!(index.as_decimal_str(), magnitude);
    assert_eq!(index.to_u32(), None);
    for kind in [Kind::CharacterVariant, Kind::Styleset] {
        assert_parsed(kind, &[&spelling, "0"], &[&magnitude, "0"]);
        assert_checked(kind, &[&spelling, "0"], &[&magnitude, "0"]);
    }
}

#[test]
fn invalid_definitions_recover_locally_without_discarding_newly_admitted_siblings() {
    let source = "/*😀*/ @font-feature-values Demo { @character-variant { One: +0001; bad: 1 2 3; Wide: +000100 2; } @styleset { Wide: 21 100; bad: -1; Zero: -000; } }";
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    assert!(validate_sheet(source).is_err());
    assert_eq!(report.diagnostics().len(), 2, "{:?}", report.diagnostics());
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDescriptor)
    );
    let [CssRule::FontFeatureValues(rule)] = report.syntax().rules() else {
        panic!("outer rule remains")
    };
    assert_eq!(rule.items().len(), 2);
    let character = block(&rule.items()[0]);
    let styleset = block(&rule.items()[1]);
    assert_eq!(
        character
            .definitions()
            .iter()
            .map(|value| value.name().as_str())
            .collect::<Vec<_>>(),
        ["One", "Wide"]
    );
    assert_eq!(
        styleset
            .definitions()
            .iter()
            .map(|value| value.name().as_str())
            .collect::<Vec<_>>(),
        ["Wide", "Zero"]
    );
    assert_eq!(indexes(&character.definitions()[0]), ["1"]);
    assert_eq!(indexes(&character.definitions()[1]), ["100", "2"]);
    assert_eq!(indexes(&styleset.definitions()[0]), ["21", "100"]);
    assert_eq!(indexes(&styleset.definitions()[1]), ["0"]);
    for (definition, needle) in [
        (&character.definitions()[0], "One: +0001"),
        (&character.definitions()[1], "Wide: +000100"),
        (&styleset.definitions()[0], "Wide: 21"),
        (&styleset.definitions()[1], "Zero: -000"),
    ] {
        assert_eq!(
            definition.position().unwrap().byte_offset().value(),
            source.find(needle).unwrap()
        );
    }
    let first = character.definitions()[0].indexes()[0].origin().unwrap();
    for definition in character.definitions().iter().chain(styleset.definitions()) {
        for index in definition.indexes() {
            let origin = index.origin().unwrap();
            assert!(first.source().same_snapshot(origin.source()));
            assert_eq!(origin.source().as_str(), source);
        }
    }
}

#[test]
fn rejected_integer_syntax_and_index_counts_remain_intrinsic_errors() {
    for text in ["", "+", "1.0", "1e0", "1px", "1%", "1 2"] {
        assert_eq!(
            Index::try_from_decimal(text).unwrap_err().kind(),
            ConstructionError::InvalidIntegerSyntax
        );
    }
    assert_eq!(
        Index::try_from_decimal("-1").unwrap_err().kind(),
        ConstructionError::NegativeIndex
    );
    assert_eq!(
        Definition::try_new(
            CssFontFeatureValueName::try_new("empty").unwrap(),
            Vec::new()
        )
        .unwrap_err()
        .kind(),
        ConstructionError::EmptyIndexes
    );
    for (kind, values) in [
        (Kind::CharacterVariant, &["1", "2", "3"][..]),
        (Kind::Stylistic, &["1", "2"][..]),
        (Kind::Swash, &["1", "2"][..]),
        (Kind::Ornaments, &["1", "2"][..]),
        (Kind::Annotation, &["1", "2"][..]),
    ] {
        let error = Block::try_new(
            kind,
            vec![
                definition("good", &["1", "2"][..kind_count(kind)]),
                definition("bad", values),
            ],
        )
        .unwrap_err();
        assert_eq!(error.kind(), ConstructionError::InvalidIndexCount);
        assert_eq!(error.block(), Some(kind));
        assert_eq!(error.definition_index(), Some(1));
    }
    for (kind, invalid) in [
        ("character-variant", "1 2 3"),
        ("character-variant", "-1 2"),
        ("character-variant", "1.0 2"),
        ("styleset", ""),
        ("styleset", "1,2"),
        ("styleset", "1e0"),
        ("styleset", "1 !important"),
        ("styleset", "calc(1)"),
    ] {
        let source = format!(
            "@font-feature-values Demo {{ @{kind} {{ bad: {invalid}; keep: {}; }} }}",
            if kind == "character-variant" {
                "1 2"
            } else {
                "1"
            }
        );
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{source}");
        assert!(validate_sheet(&source).is_err());
        let [CssRule::FontFeatureValues(rule)] = report.syntax().rules() else {
            panic!("rule survives invalid definition")
        };
        let definitions = block(&rule.items()[0]).definitions();
        assert_eq!(definitions.len(), 1, "{source}");
        assert_eq!(definitions[0].name().as_str(), "keep");
    }
}

fn kind_count(kind: Kind) -> usize {
    if kind == Kind::CharacterVariant { 2 } else { 1 }
}
