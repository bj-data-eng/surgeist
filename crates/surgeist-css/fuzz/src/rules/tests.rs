use super::*;
use crate::testing::{assert_detected_replayed_and_reduced, key};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_fuzz::{OutcomeKind, engine};

fn parse_block(text: &str) -> surgeist_css::CssParseReport<CssDeclarationList> {
    CssParser::default().parse_block_contents(text)
}

#[test]
fn rendering_is_exact_and_records_each_injected_unit() {
    // Literal 1: an unknown at-rule between two valid rules.
    let rendered = render(&literal(1).unwrap(), &DECLS);
    assert_eq!(
        rendered.text,
        ".a {color: red; }\n@unknown x { a { b: c } }\n#i {width: 10px; }\n"
    );
    let start = rendered.text.find("@unknown").unwrap();
    assert_eq!(rendered.bad, vec![(start, start + 25)]);
    assert_eq!(
        &rendered.text[start..start + 25],
        "@unknown x { a { b: c } }"
    );
    assert_eq!(rendered.rules.len(), 2);

    // Literal 2: each invalid declaration ends at its own semicolon.
    let rendered = render(&literal(2).unwrap(), &DECLS);
    assert_eq!(rendered.bad.len(), 4);
    for (start, end) in &rendered.bad {
        assert!(rendered.text[*start..*end].ends_with(';'));
    }
    assert!(
        rendered
            .text
            .starts_with(".a {width: bogus-keyword-xyz; color: red; ")
    );

    // A style attribute renders the first style's items without braces, and
    // reports a valid nested rule as one dropped unit.
    let rendered = render(&literal(10).unwrap(), &DECLS);
    assert_eq!(
        rendered.text,
        "color: red; & .n {width: 10px; } width: 20px !important; width: \"text\"; --x: 1px; "
    );
    assert_eq!(rendered.bad.len(), 2);
    assert_eq!(&rendered.text[12..32], "& .n {width: 10px; }");
}

/// Handwritten from the CSSOM selection rule: an important declaration beats a
/// normal one, a later one beats an earlier one of equal importance, and the
/// winner keeps its own position.
#[test]
fn duplicate_names_have_the_handwritten_winners_under_the_crates_selection() {
    let rendered = render(&literal(3).unwrap(), &DECLS);
    assert_eq!(
        rendered.text,
        ".a {width: 10px; color: red; width: 20px !important; \\77 idth: 2px; COLOR: BLUE; color: red !important; }\n"
    );
    let report = CssParser::default().parse_sheet(&rendered.text);
    assert!(report.is_clean());
    let CssRule::Style(rule) = &report.syntax().rules()[0] else {
        panic!("a style rule");
    };
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(
        rule.declarations(),
        CssSerializationLimits::default(),
    )
    .unwrap();
    // width: ordinal 2 (important); color: ordinal 5 (important).
    let winners: Vec<_> = block
        .entries()
        .iter()
        .map(|entry| (entry.authored_ordinal(), entry.importance()))
        .collect();
    assert_eq!(
        winners,
        vec![(2, CssImportance::Important), (5, CssImportance::Important)]
    );
    assert_eq!(
        expected_winners(
            &[DECLS[3], DECLS[0], DECLS[4], DECLS[5], DECLS[1], DECLS[2]],
            later_or_important_wins
        )
        .iter()
        .map(|(ordinal, _)| *ordinal)
        .collect::<Vec<_>>(),
        vec![2, 5]
    );
}

#[test]
fn custom_names_are_case_sensitive_and_escapes_decode() {
    let report = parse_block("--x: 1; --X: 2; \\--e: 3; --\u{e9}: 4; \\77 idth: 5px");
    assert!(report.is_clean());
    let names: Vec<_> = report
        .syntax()
        .iter()
        .map(|declaration| match declaration.property_name() {
            CssPropertyNameRef::Custom(name) => name.as_str().to_owned(),
            CssPropertyNameRef::Known(property) => property.canonical_name().to_owned(),
            _ => "?".into(),
        })
        .collect();
    assert_eq!(names, ["--x", "--X", "--e", "--\u{e9}", "width"]);
}

#[test]
fn every_literal_captures_replays_and_checks_every_property() {
    let target = RulesTarget::new();
    for id in 0..RulesTarget::LITERALS {
        let case = target.literal(id).unwrap();
        let mut captured = Vec::new();
        let observed = engine::evaluate_case(&target, &case, &mut |bytes| {
            captured = bytes.to_vec();
            Ok(())
        })
        .unwrap();
        assert_eq!(
            observed.outcome.kind(),
            OutcomeKind::Evaluated,
            "literal {id}: {:?}",
            observed.outcome.detail()
        );
        assert_eq!(observed.outcome.checked_properties().len(), 7);
        assert_eq!(target.decode(&captured).unwrap(), case);
        assert_eq!(target.encode(&case).unwrap(), captured);
        assert_eq!(
            engine::replay(&target, &captured, &mut |_| Ok(())).unwrap(),
            observed
        );
    }
    assert!(target.literal(RulesTarget::LITERALS).is_err());
}

#[test]
fn clean_exactly_when_nothing_invalid_is_injected() {
    let clean = render(&literal(0).unwrap(), &DECLS);
    assert!(clean.bad.is_empty());
    assert!(CssParser::default().parse_sheet(&clean.text).is_clean());
    let dirty = render(&literal(1).unwrap(), &DECLS);
    let report = CssParser::default().parse_sheet(&dirty.text);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
}

fn first_wins(_held_important: bool, _new_important: bool) -> bool {
    // Planted: forgets importance and source order; the first declaration wins.
    false
}

static IDENTITY_SWAPPED: [DeclRow; 20] = {
    let mut rows = DECLS;
    // Planted: claims an escaped `width` name is a custom property.
    rows[5] = ("\\77 idth: 2px", Expect::Custom("--width"), false);
    rows
};

fn column_shifted(text: &str, offset: usize) -> (u32, u32) {
    let (line, column) = check::expected_coordinates(text, offset);
    (line, column + 1)
}

#[test]
fn a_broken_selection_rule_is_detected_replayed_and_reduced() {
    let broken = RulesTarget::with_oracle(Oracle {
        replaces: first_wins,
        ..INDEPENDENT
    });
    let sound = RulesTarget::new();
    let reduced = assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &literal(3).unwrap(),
        &key(SELECTION, "winner-differs"),
    );
    // Reduction keeps what the failure needs: duplicate names in one style rule.
    let reduced = sound.decode(&reduced).unwrap();
    assert_eq!(reduced.nodes.len(), 1);
    let Node::Style { style, .. } = &reduced.nodes[0] else {
        panic!("a style rule");
    };
    assert!(style.items.len() < 6 && style.items.len() >= 2);
}

#[test]
fn a_broken_declaration_identity_is_detected_replayed_and_reduced() {
    let broken = RulesTarget::with_oracle(Oracle {
        rows: &IDENTITY_SWAPPED,
        ..INDEPENDENT
    });
    let sound = RulesTarget::new();
    assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &literal(3).unwrap(),
        &key(RECOVERY, "declaration-identity-differs"),
    );
}

#[test]
fn a_broken_coordinate_model_is_detected_replayed_and_reduced() {
    let broken = RulesTarget::with_oracle(Oracle {
        checker: Checker {
            coordinates: column_shifted,
        },
        ..INDEPENDENT
    });
    let sound = RulesTarget::new();
    // Literal 9 has an invalid unit, so its diagnostic position is checked.
    assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &literal(9).unwrap(),
        &key(check::COORDINATES, "line-column-disagree"),
    );
}

#[test]
fn a_dropped_nested_rule_in_a_style_attribute_is_one_reported_unit() {
    let rendered = render(&literal(10).unwrap(), &DECLS);
    let report = parse_block(&rendered.text);
    // `& .n {...}` is consumed as a nested rule and dropped (Syntax 3 5.5.5).
    assert_eq!(report.syntax().len(), 3);
    assert_eq!(report.diagnostics().len(), 2);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropQualifiedRule
    );
}

#[test]
fn generation_is_deterministic_bounded_and_reaches_every_structure() {
    let target = RulesTarget::new();
    let (mut nested, mut groups, mut injected, mut blocks, mut skipped) = (0, 0, 0, 0, 0);
    for seed in 0_u32..600 {
        let entropy: Vec<u8> = (0..200_u32)
            .map(|slot| (seed.wrapping_mul(2_654_435_761).wrapping_add(slot * 40_503) >> 8) as u8)
            .collect();
        let Generation::Generated(case) = target.generate(&entropy).unwrap() else {
            panic!("rules generation never discards");
        };
        assert_eq!(
            target.generate(&entropy).unwrap(),
            Generation::Generated(case.clone())
        );
        let rendered = render(&case, &DECLS);
        nested += usize::from(rendered.text.contains("& ") || rendered.text.contains("> .c"));
        groups += usize::from(rendered.text.contains('@') && !rendered.rules.is_empty());
        injected += usize::from(!rendered.bad.is_empty());
        blocks += usize::from(case.front == Front::Block);
        skipped += usize::from(
            case.nodes
                .iter()
                .any(|node| matches!(node, Node::Skip { .. })),
        );
        let observed = engine::evaluate_case(&target, &case, &mut |_| Ok(())).unwrap();
        assert_ne!(
            observed.outcome.kind(),
            OutcomeKind::Failure,
            "seed {seed}: {:?}",
            observed.outcome.detail()
        );
    }
    assert!(nested > 50 && groups > 50 && injected > 200 && blocks > 80 && skipped > 20);
}

#[test]
fn the_strict_codec_round_trips_and_rejects_bounds_tags_and_truncation() {
    let target = RulesTarget::new();
    for id in 0..RulesTarget::LITERALS {
        let case = literal(id).unwrap();
        let bytes = target.encode(&case).unwrap();
        assert_eq!(target.decode(&bytes).unwrap(), case);
        for length in 0..bytes.len() {
            assert!(target.decode(&bytes[..length]).is_err(), "{id}/{length}");
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(target.decode(&trailing).is_err());
    }
    let bytes = target.encode(&literal(0).unwrap()).unwrap();
    // Magic, revision, front tag and the first node tag.
    for (index, value) in [(0, b'X'), (4, 2), (5, 9)] {
        let mut changed = bytes.clone();
        changed[index] = value;
        assert!(target.decode(&changed).is_err(), "{index}");
    }
    let mut changed = bytes.clone();
    changed[10] = 9;
    assert!(target.decode(&changed).is_err());

    // Table indices, counts and depth are checked before use.
    let style_with = |items: Vec<Item>| Node::Style {
        style: Style { selector: 0, items },
        sep: 0,
    };
    assert!(RulesCase::new(Front::Sheet, vec![Node::Skip { row: 4, sep: 0 }]).is_err());
    assert!(RulesCase::new(Front::Sheet, vec![Node::Ignorable { row: 0, sep: 8 }]).is_err());
    assert!(
        RulesCase::new(
            Front::Sheet,
            vec![Node::Group {
                group: 6,
                children: Vec::new(),
                sep: 0
            }]
        )
        .is_err()
    );
    assert!(
        RulesCase::new(
            Front::Sheet,
            vec![style_with(vec![Item::Decl { row: 20, sep: 0 }])]
        )
        .is_err()
    );
    assert!(
        RulesCase::new(
            Front::Sheet,
            vec![style_with(vec![
                Item::Decl { row: 0, sep: 0 };
                MAX_ITEMS + 1
            ])]
        )
        .is_err()
    );
    let too_many: Vec<Node> = (0..=MAX_NODES)
        .map(|_| Node::Ignorable { row: 0, sep: 0 })
        .collect();
    assert!(RulesCase::new(Front::Sheet, too_many).is_err());
    let mut deep = Node::Skip { row: 0, sep: 0 };
    for _ in 0..MAX_DEPTH {
        deep = Node::Group {
            group: 0,
            children: vec![deep],
            sep: 0,
        };
    }
    assert!(RulesCase::new(Front::Sheet, vec![deep]).is_err());
}

#[test]
fn reduction_candidates_are_valid_and_stop_when_refused() {
    let target = RulesTarget::new();
    let case = literal(6).unwrap();
    let mut emitted = Vec::new();
    target
        .reduction_candidates(&case, &mut |candidate| {
            emitted.push(candidate);
            true
        })
        .unwrap();
    assert!(emitted.len() >= 2);
    let mut calls = 0;
    target
        .reduction_candidates(&case, &mut |_| {
            calls += 1;
            false
        })
        .unwrap();
    assert_eq!(calls, 1);
}
