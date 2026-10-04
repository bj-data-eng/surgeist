#![forbid(unsafe_code)]

//! Mode-specific membership uses selected Logical 1 side assignments and the
//! explicit Surgeist no-complementary-reset policy. It never exposes an eight-
//! member union or an unqualified default footprint as fixed shorthand metadata.
use surgeist_css::*;

#[test]
fn mode_metadata_selects_exact_ordered_sides_and_empty_reset_set() {
    for (name, physical, logical) in [
        (
            "border-color",
            [
                "border-top-color",
                "border-right-color",
                "border-bottom-color",
                "border-left-color",
            ],
            [
                "border-block-start-color",
                "border-inline-start-color",
                "border-block-end-color",
                "border-inline-end-color",
            ],
        ),
        (
            "border-style",
            [
                "border-top-style",
                "border-right-style",
                "border-bottom-style",
                "border-left-style",
            ],
            [
                "border-block-start-style",
                "border-inline-start-style",
                "border-block-end-style",
                "border-inline-end-style",
            ],
        ),
        (
            "border-width",
            [
                "border-top-width",
                "border-right-width",
                "border-bottom-width",
                "border-left-width",
            ],
            [
                "border-block-start-width",
                "border-inline-start-width",
                "border-block-end-width",
                "border-inline-end-width",
            ],
        ),
        (
            "inset",
            ["top", "right", "bottom", "left"],
            [
                "inset-block-start",
                "inset-inline-start",
                "inset-block-end",
                "inset-inline-end",
            ],
        ),
        (
            "margin",
            ["margin-top", "margin-right", "margin-bottom", "margin-left"],
            [
                "margin-block-start",
                "margin-inline-start",
                "margin-block-end",
                "margin-inline-end",
            ],
        ),
        (
            "padding",
            [
                "padding-top",
                "padding-right",
                "padding-bottom",
                "padding-left",
            ],
            [
                "padding-block-start",
                "padding-inline-start",
                "padding-block-end",
                "padding-inline-end",
            ],
        ),
        (
            "scroll-margin",
            [
                "scroll-margin-top",
                "scroll-margin-right",
                "scroll-margin-bottom",
                "scroll-margin-left",
            ],
            [
                "scroll-margin-block-start",
                "scroll-margin-inline-start",
                "scroll-margin-block-end",
                "scroll-margin-inline-end",
            ],
        ),
        (
            "scroll-padding",
            [
                "scroll-padding-top",
                "scroll-padding-right",
                "scroll-padding-bottom",
                "scroll-padding-left",
            ],
            [
                "scroll-padding-block-start",
                "scroll-padding-inline-start",
                "scroll-padding-block-end",
                "scroll-padding-inline-end",
            ],
        ),
    ] {
        let grammar = CssPropertyGrammar::from_name(name).unwrap();
        let CssPropertyKindRef::FourSideShorthand(metadata) = grammar.metadata().unwrap().kind()
        else {
            panic!("mode-specific {name} metadata")
        };
        assert!(metadata.reset_only_members().is_empty());
        for (mode, names) in [
            (CssBoxSideKind::Physical, physical),
            (CssBoxSideKind::Logical, logical),
        ] {
            let members = metadata.members(mode);
            assert_eq!(members.len(), 4);
            assert_eq!(members, metadata.settable_members(mode));
            for (member, expected) in members.iter().zip(names) {
                assert_eq!(
                    member.known_property(),
                    CssPropertyGrammar::from_name(expected)
                        .unwrap()
                        .target_property()
                );
            }
        }
        assert!(
            !metadata
                .members(CssBoxSideKind::Physical)
                .iter()
                .any(|member| metadata.members(CssBoxSideKind::Logical).contains(member))
        );
    }
}

#[test]
fn fixed_axis_metadata_keeps_its_complete_unqualified_getters() {
    let CssPropertyKindRef::Shorthand(metadata) = CssPropertyGrammar::from_name("margin-block")
        .unwrap()
        .metadata()
        .unwrap()
        .kind()
    else {
        panic!("fixed pair")
    };
    let expected = ["margin-block-start", "margin-block-end"].map(|name| {
        CssPropertyGrammar::from_name(name)
            .unwrap()
            .target_property()
    });
    assert_eq!(
        metadata
            .members()
            .iter()
            .map(|member| member.known_property())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(metadata.members(), metadata.settable_members());
    assert!(metadata.reset_only_members().is_empty());
    assert!(!metadata.is_legacy());
}
