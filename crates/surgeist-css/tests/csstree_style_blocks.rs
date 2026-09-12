#![forbid(unsafe_code)]
//! Independent Syntax 3 (2021-12-24) and Nesting 1 (2026-01-22) block cases.
//! All real outer blocks survive; unknown inner grammar recovers locally.
use surgeist_css::{CssErrorCode, CssNamespaceContext, CssRecoveryAction, parse_style_block};
type ExpectedDiagnostic = (
    &'static str,
    &'static str,
    usize,
    usize,
    usize,
    &'static str,
);
#[test]
fn original_block_corpus_preserves_empty_blocks_and_owned_diagnostics() {
    let expectations: &[(&str, &[ExpectedDiagnostic])] = &[
        (
            "block/Block.json#/at-rule in block",
            &[
                ("unknown_at_rule", "drop_at_rule", 1, 1, 10, "intersects"),
                (
                    "unknown_property",
                    "drop_declaration",
                    10,
                    10,
                    18,
                    "intersects",
                ),
            ],
        ),
        (
            "block/Block.json#/at-rule in the ending of block",
            &[
                (
                    "unknown_property",
                    "drop_declaration",
                    1,
                    1,
                    10,
                    "intersects",
                ),
                ("unknown_at_rule", "drop_at_rule", 10, 10, 19, "intersects"),
            ],
        ),
        (
            "block/Block.json#/at-rule with block in block",
            &[
                ("unknown_at_rule", "drop_at_rule", 1, 1, 7, "intersects"),
                (
                    "unknown_property",
                    "drop_declaration",
                    7,
                    7,
                    15,
                    "intersects",
                ),
            ],
        ),
        (
            "block/Block.json#/at-rule with block in the ending of block",
            &[
                (
                    "unknown_property",
                    "drop_declaration",
                    1,
                    1,
                    10,
                    "intersects",
                ),
                ("unknown_at_rule", "drop_at_rule", 10, 10, 16, "intersects"),
            ],
        ),
        (
            "block/Block.json#/bad declaration should include semicolon/0",
            &[(
                "unknown_property",
                "drop_declaration",
                1,
                1,
                12,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/bad declaration should include semicolon/1",
            &[(
                "unknown_property",
                "drop_declaration",
                1,
                1,
                13,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/bad declaration should not include whitespaces/0",
            &[(
                "unknown_property",
                "drop_declaration",
                2,
                2,
                9,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/bad declaration should not include whitespaces/1",
            &[("unexpected_end", "drop_declaration", 6, 2, 6, "intersects")],
        ),
        (
            "block/Block.json#/bad declaration should not include whitespaces/2",
            &[(
                "unknown_property",
                "drop_declaration",
                2,
                2,
                13,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/bad symbol in a property name",
            &[(
                "invalid_qualified_rule",
                "drop_qualified_rule",
                12,
                1,
                12,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/bad value",
            &[(
                "unknown_property",
                "drop_declaration",
                1,
                1,
                13,
                "intersects",
            )],
        ),
        ("block/Block.json#/block.0", &[]),
        (
            "block/Block.json#/block.2",
            &[(
                "unknown_property",
                "drop_declaration",
                1,
                1,
                4,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/block.3",
            &[(
                "unknown_property",
                "drop_declaration",
                1,
                1,
                5,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/block.4",
            &[
                (
                    "unknown_property",
                    "drop_declaration",
                    1,
                    1,
                    7,
                    "intersects",
                ),
                (
                    "unknown_property",
                    "drop_declaration",
                    7,
                    7,
                    12,
                    "intersects",
                ),
            ],
        ),
        ("block/Block.json#/block.c.0", &[]),
        ("block/Block.json#/block.c.1", &[]),
        (
            "block/Block.json#/block.c.2",
            &[(
                "unknown_property",
                "drop_declaration",
                9,
                9,
                20,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/block.c.3",
            &[(
                "unknown_property",
                "drop_declaration",
                9,
                9,
                21,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/block.c.4",
            &[
                (
                    "unknown_property",
                    "drop_declaration",
                    9,
                    9,
                    23,
                    "intersects",
                ),
                (
                    "unknown_property",
                    "drop_declaration",
                    31,
                    31,
                    44,
                    "intersects",
                ),
            ],
        ),
        ("block/Block.json#/block.s.0", &[]),
        (
            "block/Block.json#/block.s.2",
            &[(
                "unknown_property",
                "drop_declaration",
                3,
                3,
                8,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/block.s.3",
            &[(
                "unknown_property",
                "drop_declaration",
                3,
                3,
                9,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/block.s.4",
            &[
                (
                    "unknown_property",
                    "drop_declaration",
                    3,
                    3,
                    15,
                    "intersects",
                ),
                (
                    "unknown_property",
                    "drop_declaration",
                    17,
                    17,
                    28,
                    "intersects",
                ),
            ],
        ),
        ("block/Block.json#/hanging semicolons/0", &[]),
        ("block/Block.json#/hanging semicolons/1", &[]),
        ("block/Block.json#/hanging semicolons/2", &[]),
        (
            "block/Block.json#/hanging semicolons/3",
            &[(
                "unknown_property",
                "drop_declaration",
                2,
                2,
                13,
                "intersects",
            )],
        ),
        (
            "block/Block.json#/uncomplete !important",
            &[(
                "unknown_property",
                "drop_declaration",
                1,
                1,
                12,
                "intersects",
            )],
        ),
    ];
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("corpus/csstree/expectations/block/Block.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), expectations.len());
    let registry: serde_json::Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    for &(id, expected_diagnostics) in expectations {
        let matching: Vec<_> = cases.iter().filter(|case| case["id"] == id).collect();
        assert_eq!(matching.len(), 1, "{id}");
        let source = matching[0]["input"].as_str().unwrap();
        let report = parse_style_block(source, &CssNamespaceContext::default());
        let block = report
            .syntax()
            .as_ref()
            .unwrap_or_else(|| panic!("retained block: {id}: {report:?}"));
        assert!(block.declarations().is_empty(), "{id}");
        assert!(block.rules().is_empty(), "{id}");
        assert_eq!(block.origin().source().as_str(), source, "{id}");
        assert_eq!(
            block.origin().span().start().byte_offset().value(),
            0,
            "{id}"
        );
        assert_eq!(
            block.origin().span().end().byte_offset().value(),
            source.len(),
            "{id}"
        );
        assert_eq!(report.is_clean(), expected_diagnostics.is_empty(), "{id}");
        assert_eq!(
            report.diagnostics().len(),
            expected_diagnostics.len(),
            "{id}: {report:?}"
        );
        for (actual, &(code, action, byte, start, end, relation)) in
            report.diagnostics().iter().zip(expected_diagnostics)
        {
            let expected_code = match code {
                "unknown_property" => CssErrorCode::UnknownProperty,
                "unknown_at_rule" => CssErrorCode::UnknownAtRule,
                "unexpected_end" => CssErrorCode::UnexpectedEnd,
                "invalid_qualified_rule" => CssErrorCode::InvalidQualifiedRule,
                _ => panic!("independent code {code}"),
            };
            let expected_action = match action {
                "drop_declaration" => CssRecoveryAction::DropDeclaration,
                "drop_at_rule" => CssRecoveryAction::DropAtRule,
                "drop_qualified_rule" => CssRecoveryAction::DropQualifiedRule,
                _ => panic!("independent action {action}"),
            };
            assert_eq!(actual.error().code(), expected_code, "{id}");
            assert_eq!(actual.action(), expected_action, "{id}");
            let position = actual.error().position();
            assert_eq!(
                (
                    position.byte_offset().value(),
                    position.line().value(),
                    position.column().value()
                ),
                (byte, 0, u32::try_from(byte).unwrap()),
                "{id}"
            );
            assert_eq!(
                (
                    actual.span().start().byte_offset().value(),
                    actual.span().end().byte_offset().value()
                ),
                (start, end),
                "{id}"
            );
            assert_eq!(relation, "intersects");
            assert!(byte < source.len() && start < end && end <= source.len());
        }
        let mut expected = serde_json::json!({"kind": if expected_diagnostics.is_empty() {"clean"} else {"recovered"}, "retained_syntax":{"extractor":{"kind":"style_block"},"predicate":{"relation":"nonempty"}}});
        if !expected_diagnostics.is_empty() {
            expected["diagnostics"] = serde_json::Value::Array(expected_diagnostics.iter().map(|(code,action,_,_,_,relation)| serde_json::json!({"code":code,"action":action,"payload_relation":relation})).collect());
        }
        let matching: Vec<_> = registry["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|record| record["id"] == id)
            .collect();
        assert_eq!(matching.len(), 1, "{id}");
        assert_eq!(matching[0]["class"], expected, "{id}");
    }
}
