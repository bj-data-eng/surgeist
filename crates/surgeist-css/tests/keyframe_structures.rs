use surgeist_css::{CssErrorCode, CssKeyframeSelector, CssRecoveryAction, CssRule, parse_sheet};

fn keyframes(rule: &CssRule) -> &surgeist_css::CssKeyframesRule {
    let CssRule::Keyframes(keyframes) = rule else {
        panic!("expected keyframes rule");
    };
    keyframes
}

fn keyframes_with_total_depth(depth: usize) -> String {
    assert!(depth >= 2);
    format!(
        "@keyframes deep{{from{{--v:{}x{}}}}}",
        "f(".repeat(depth - 2),
        ")".repeat(depth - 2),
    )
}

#[test]
fn keyframes_preserve_empty_and_duplicate_authored_structure() {
    let source = concat!(
        "@keyframes fade { ",
        "from, 0%, from { } ",
        "from { opacity: 0; } ",
        "0% { opacity: 1; } ",
        "} ",
        "@keyframes empty {}",
    );
    let report = parse_sheet(source);

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Keyframes(fade), CssRule::Keyframes(empty)] = report.syntax().rules() else {
        panic!("expected both authored keyframes rules");
    };
    assert_eq!(fade.blocks().len(), 3);
    assert!(fade.blocks()[0].declarations().is_empty());
    assert_eq!(
        fade.blocks()[0].selectors().selectors(),
        [
            CssKeyframeSelector::From,
            CssKeyframeSelector::Percent(surgeist_css::CssKeyframePercent::try_new(0.0).unwrap()),
            CssKeyframeSelector::From,
        ]
    );
    assert!(matches!(
        fade.blocks()[1].selectors().selectors(),
        [CssKeyframeSelector::From]
    ));
    assert!(matches!(
        fade.blocks()[2].selectors().selectors(),
        [CssKeyframeSelector::Percent(percent)] if percent.value().value() == 0.0
    ));
    assert_eq!(fade.blocks()[1].declarations().len(), 1);
    assert_eq!(fade.blocks()[2].declarations().len(), 1);
    assert!(empty.blocks().is_empty());
}

#[test]
fn invalid_keyframe_children_drop_the_smallest_unit_and_retain_empty_rules() {
    let source = concat!(
        "@keyframes empty { ",
        "fn(a) { opacity: .25; } ",
        "fn(b) { opacity: .75; } ",
        "} ",
        ".after { color: red; }",
    );
    let report = parse_sheet(source);

    let [empty, CssRule::Style(after)] = report.syntax().rules() else {
        panic!("expected retained keyframes and later style rule");
    };
    assert!(keyframes(empty).blocks().is_empty());
    assert_eq!(after.declarations().len(), 1);
    assert_eq!(report.diagnostics().len(), 2);
    assert!(report.diagnostics().iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::UnexpectedToken
            && diagnostic.action() == CssRecoveryAction::DropKeyframeBlock
    }));
    assert!(
        report.diagnostics()[0].span().start().byte_offset().value()
            < report.diagnostics()[1].span().start().byte_offset().value()
    );
}

#[test]
fn dropped_declarations_leave_empty_blocks_with_exact_non_bmp_coordinates() {
    let source = "@keyframes 😀fade { from { mystery: 1; } to { opacity: 0 !important; } }";
    let report = parse_sheet(source);

    let [rule] = report.syntax().rules() else {
        panic!("expected retained keyframes rule");
    };
    let blocks = keyframes(rule).blocks();
    assert_eq!(blocks.len(), 2);
    assert!(blocks.iter().all(|block| block.declarations().is_empty()));
    assert_eq!(report.diagnostics().len(), 2);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );

    let mystery = source.find("mystery").unwrap();
    let first = &report.diagnostics()[0];
    assert_eq!(first.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(first.error().position().byte_offset().value(), mystery);
    assert_eq!(
        first.error().position().column().value() as usize,
        source[..mystery].encode_utf16().count(),
    );
    assert_eq!(first.span().start().byte_offset().value(), mystery);
    assert_eq!(
        first.span().end().byte_offset().value(),
        mystery + "mystery: 1;".len(),
    );
}

#[test]
fn keyframe_depth_boundary_retains_255_and_256_but_stops_at_257() {
    for depth in [255, 256] {
        let source = keyframes_with_total_depth(depth);
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let [rule] = report.syntax().rules() else {
            panic!("depth {depth}: expected retained rule");
        };
        assert_eq!(keyframes(rule).blocks()[0].declarations().len(), 1);
    }

    let source = keyframes_with_total_depth(257);
    let report = parse_sheet(&source);
    let [rule] = report.syntax().rules() else {
        panic!("over-limit declaration must leave its authored parents");
    };
    assert!(keyframes(rule).blocks()[0].declarations().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one nesting-limit diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
}

#[test]
fn empty_keyframe_rule_and_block_retain_eof_closures() {
    for (source, expected_blocks) in [("@keyframes empty {", 0), ("@keyframes empty { from {", 1)] {
        let report = parse_sheet(source);
        let [rule] = report.syntax().rules() else {
            panic!("{source}: expected retained keyframes rule");
        };
        assert_eq!(keyframes(rule).blocks().len(), expected_blocks, "{source}");
        assert!(
            keyframes(rule)
                .blocks()
                .iter()
                .all(|block| block.declarations().is_empty())
        );
        assert!(report.diagnostics().iter().all(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        }));
        assert!(!report.diagnostics().is_empty());
    }
}

#[cfg(feature = "app-strict")]
#[test]
fn app_strict_matches_ordinary_empty_duplicate_and_recovered_keyframes() {
    let clean = "@keyframes fade { from, 0%, from { } from { opacity: 1; } }";
    let ordinary = parse_sheet(clean);
    assert!(ordinary.is_clean());
    assert_eq!(
        surgeist_css::validate_sheet(clean),
        Ok(ordinary.syntax().clone())
    );

    let recovered = "@keyframes fade { from { mystery: 1; } }";
    let ordinary = parse_sheet(recovered);
    let failure = surgeist_css::validate_sheet(recovered)
        .expect_err("strict validation must reject a recovered declaration");
    assert_eq!(failure.diagnostics(), ordinary.diagnostics());
    let [rule] = ordinary.syntax().rules() else {
        panic!("ordinary recovery must retain the empty keyframes parent");
    };
    assert!(keyframes(rule).blocks()[0].declarations().is_empty());
}
