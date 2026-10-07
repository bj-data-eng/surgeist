#![forbid(unsafe_code)]
//! Existing-callable authored admission contracts from Pseudo4 WD20250627.
//! These selectors remain symbolic: admission does not assert live matching.
use surgeist_css::*;

fn same_semantics(left: &CssSelector, left_source: &str, right: &CssSelector, right_source: &str) {
    let (CssSelector::Compound(left), CssSelector::Compound(right)) = (left, right) else {
        panic!("pseudo-elements retain originating compounds")
    };
    assert_eq!(left.scope_anchors(), right.scope_anchors());
    assert_eq!(left.nesting_selectors(), right.nesting_selectors());
    assert_eq!(left.type_selector(), right.type_selector());
    assert_eq!(left.ids(), right.ids());
    assert_eq!(left.classes(), right.classes());
    assert_eq!(left.attributes(), right.attributes());
    assert_eq!(left.pseudo_classes(), right.pseudo_classes());
    let left = left.pseudo_elements().unwrap().segments();
    let right = right.pseudo_elements().unwrap().segments();
    assert_eq!(left.len(), right.len());
    for (left, right) in left.iter().zip(right) {
        match (left, right) {
            (
                CssPseudoElementSegment::PseudoElement(CssPseudoElement::Part(left)),
                CssPseudoElementSegment::PseudoElement(CssPseudoElement::Part(right)),
            ) => {
                // Every Part fixture independently names exactly "label".
                // Its token origins belong to the actual fragment/sheet/reparse.
                for (names, source) in [(left.names(), left_source), (right.names(), right_source)]
                {
                    assert_eq!(
                        names.iter().map(CssPartName::as_str).collect::<Vec<_>>(),
                        ["label"]
                    );
                    let CssValueOrigin::Parsed(origin) = names[0].origin() else {
                        panic!("parsed Part name")
                    };
                    let start = source.find("(label)").unwrap() + 1;
                    assert_eq!(origin.source().as_str(), source);
                    assert_eq!(origin.span().start().byte_offset().value(), start);
                    assert_eq!(
                        origin.span().end().byte_offset().value(),
                        start + "label".len()
                    );
                }
            }
            (
                CssPseudoElementSegment::PseudoElement(CssPseudoElement::Slotted(left)),
                CssPseudoElementSegment::PseudoElement(CssPseudoElement::Slotted(right)),
            ) => {
                assert_eq!(left.compound().classes(), &["item"]);
                assert_eq!(right.compound().classes(), &["item"]);
                assert_eq!(left.compound(), right.compound());
            }
            _ => assert_eq!(left, right),
        }
    }
}

fn admitted(cases: &[(&str, &str)]) -> Vec<CssSelector> {
    let reports: Vec<_> = cases
        .iter()
        .map(|(source, _)| {
            let selector = parse_selector(source, &CssNamespaceContext::default());
            let sheet = parse_sheet(&format!("{source}{{color:red}}"));
            (selector, sheet)
        })
        .collect();
    // Both existing public fronts execute for every member before a failure.
    for ((source, expected), (selector, sheet)) in cases.iter().zip(&reports) {
        assert_eq!(
            (
                selector.is_clean(),
                selector.syntax().is_some(),
                sheet.is_clean(),
                matches!(sheet.syntax().rules(), [CssRule::Style(_)]),
            ),
            (true, true, true, true),
            "{source}: selector={selector:?}; sheet={sheet:?}"
        );
        let value = selector.syntax().as_ref().unwrap();
        assert!(value.has_pseudo_elements(), "{source}");
        assert_eq!(value.to_specified_css().unwrap(), *expected);
        let [CssRule::Style(rule)] = sheet.syntax().rules() else {
            unreachable!()
        };
        let sheet_value = rule.selectors().selectors()[0].selector();
        assert_eq!(sheet_value.to_specified_css().unwrap(), *expected);
        same_semantics(
            value,
            source,
            sheet_value,
            &format!("{source}{{color:red}}"),
        );
        assert_eq!(rule.declarations().len(), 1);
        let replay = parse_selector(expected, &CssNamespaceContext::default());
        assert!(replay.is_clean(), "{expected}: {replay:?}");
        let replay_value = replay.syntax().as_ref().unwrap();
        assert_eq!(replay_value.to_specified_css().unwrap(), *expected);
        same_semantics(value, source, replay_value, expected);
    }
    reports
        .into_iter()
        .map(|(report, _)| report.syntax().clone().unwrap())
        .collect()
}

fn rejected(cases: &[&str]) {
    for source in cases {
        let selector = parse_selector(source, &CssNamespaceContext::default());
        let sheet = parse_sheet(&format!("{source}{{color:red}}.after{{color:blue}}"));
        assert_eq!(
            (
                selector.is_clean(),
                selector.syntax().is_some(),
                sheet.is_clean()
            ),
            (false, false, false),
            "{source}: selector={selector:?}; sheet={sheet:?}"
        );
        let [CssRule::Style(after)] = sheet.syntax().rules() else {
            panic!("only the later valid sibling survives: {source}: {sheet:?}")
        };
        assert_eq!(
            after.selectors().selectors()[0].selector(),
            &CssSelector::Class("after".into())
        );
        assert_eq!(after.declarations().len(), 1);
        assert!(
            sheet
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropQualifiedRule)
        );
    }
}

#[test]
fn named_highlight_tree_abiding_and_element_backed_pseudos_are_admitted() {
    admitted(&[
        (".x::search-text", ".x::search-text"),
        (".x::target-text", ".x::target-text"),
        (".x::spelling-error", ".x::spelling-error"),
        (".x::grammar-error", ".x::grammar-error"),
        ("input::placeholder", "input::placeholder"),
        ("input::file-selector-button", "input::file-selector-button"),
        ("details::details-content", "details::details-content"),
        (".x::TARGET-TEXT", ".x::target-text"),
        (r"input::\70 laceholder", "input::placeholder"),
    ]);
}

#[test]
fn first_letter_prefix_and_suffix_are_defined_child_identities() {
    let values = admitted(&[
        ("::prefix", "::prefix"),
        ("::suffix", "::suffix"),
        ("p::first-letter::prefix", "p::first-letter::prefix"),
        ("p::first-letter::suffix", "p::first-letter::suffix"),
        ("p:FIRST-LETTER::PREFIX", "p::first-letter::prefix"),
        (r"p::first-letter::suf\66 ix", "p::first-letter::suffix"),
        (
            "p::first-letter::prefix:hover",
            "p::first-letter::prefix:hover",
        ),
    ]);
    for value in &values[..2] {
        let CssSelector::Compound(origin) = value else {
            panic!("implicit originating compound")
        };
        assert!(origin.type_selector().is_none());
        assert!(origin.ids().is_empty());
        assert!(origin.classes().is_empty());
        assert!(origin.pseudo_classes().is_empty());
    }
}

#[test]
fn highlight_requires_one_case_sensitive_custom_identifier() {
    let values = admitted(&[
        (".x::highlight(MyName)", ".x::highlight(MyName)"),
        (".x::highlight(myname)", ".x::highlight(myname)"),
        (r".x::HIGHLIGHT(\4d yName)", ".x::highlight(MyName)"),
        (".x::highlight(/**/--chosen/**/)", ".x::highlight(--chosen)"),
        (".x::highlight(none)", ".x::highlight(none)"),
    ]);
    assert_ne!(values[0], values[1]);
    assert_eq!(values[0], values[2]);
}

#[test]
fn search_text_current_permission_is_retained_inside_logical_suffixes() {
    admitted(&[
        (".x::search-text:current", ".x::search-text:current"),
        (".x::SEARCH-TEXT:CURRENT", ".x::search-text:current"),
        (
            ".x::search-text:not(:current)",
            ".x::search-text:not(:current)",
        ),
        (
            ".x::search-text:is(:current,:hover)",
            ".x::search-text:is(:current, :hover)",
        ),
        (
            ".x::search-text:where(:current)",
            ".x::search-text:where(:current)",
        ),
    ]);
}

#[test]
fn element_backed_and_imported_tree_abiding_permissions_admit_new_identities() {
    admitted(&[
        (
            "input::file-selector-button:first-child",
            "input::file-selector-button:first-child",
        ),
        (
            "details::details-content:has(>.child)",
            "details::details-content:has(> .child)",
        ),
        (
            "details::details-content:scope",
            "details::details-content:scope",
        ),
        (
            "input::file-selector-button::before::marker",
            "input::file-selector-button::before::marker",
        ),
        (
            "details::details-content::part(label)",
            "details::details-content::part(label)",
        ),
        (
            ".x::part(label)::highlight(MyName)",
            ".x::part(label)::highlight(MyName)",
        ),
        (
            ".x::slotted(.item)::placeholder",
            ".x::slotted(.item)::placeholder",
        ),
        (
            ".x::slotted(.item)::file-selector-button",
            ".x::slotted(.item)::file-selector-button",
        ),
        (
            ".x::slotted(.item)::details-content",
            ".x::slotted(.item)::details-content",
        ),
    ]);
}

#[test]
fn invalid_arguments_undefined_chains_and_unsupported_current_forms_are_rejected() {
    rejected(&[
        "::highlight()",
        "::highlight(\"chosen\")",
        "::highlight(one two)",
        "::highlight(one,two)",
        "::highlight(1)",
        "::highlight(initial)",
        "::highlight(INHERIT)",
        "::highlight(unset)",
        "::highlight(revert)",
        "::highlight(revert-layer)",
        "::highlight(DEFAULT)",
        "::highlight",
        "::highlight (chosen)",
        "::placeholder()",
        "::before::prefix",
        "::first-line::suffix",
        "::marker::prefix",
        "::first-letter::marker",
        "::first-letter::prefix::suffix",
        "::placeholder::marker",
        "::slotted(.item)::target-text",
        "::search-text:current()",
        "::search-text::current()",
        "::search-text:past",
        "::search-text:future",
        "::target-text:current",
        "::search-text:not(:past)",
        "::search-text:first-child",
        "::details-content::placeholder:first-child",
        ":not(::highlight(chosen))",
        ":has(> ::placeholder)",
        "::selection > .child",
        "::unknown-future",
        ":: search-text",
        ":placeholder",
        ":search-text",
        ":highlight(chosen)",
        ":file-selector-button",
        ":details-content",
        ":prefix",
        ":suffix",
    ]);
}

#[test]
fn four_legacy_colon_aliases_and_existing_imported_pseudos_keep_their_identity() {
    let aliases = admitted(&[
        (":before", "::before"),
        ("::before", "::before"),
        (":after", "::after"),
        ("::after", "::after"),
        (":first-letter", "::first-letter"),
        ("::first-letter", "::first-letter"),
        (":first-line", "::first-line"),
        ("::first-line", "::first-line"),
    ]);
    for pair in aliases.chunks_exact(2) {
        assert_eq!(pair[0], pair[1]);
    }
    admitted(&[
        (".x::before::marker", ".x::before::marker"),
        (".x::after::marker", ".x::after::marker"),
        (".x::selection", ".x::selection"),
        (".x::backdrop", ".x::backdrop"),
        (".x::slotted(.item)::before", ".x::slotted(.item)::before"),
        (".x::part(label):first-child", ".x::part(label):first-child"),
        (
            ".x::part(label):has(>.item)",
            ".x::part(label):has(> .item)",
        ),
    ]);
    rejected(&[
        ":marker",
        ":selection",
        ":backdrop",
        "::marker::marker",
        ": before",
        ": :after",
    ]);
}
