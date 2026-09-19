#![forbid(unsafe_code)]
//! Scope-boundary parentheses participate in the documented shared 256-level
//! structural ceiling, independently of ampersand admission or its new getter.
use surgeist_css::*;

#[test]
fn anchor_free_scope_boundary_parenthesis_counts_toward_structural_limit() {
    for limit in [false, true] {
        let prefix = if limit {
            "@scope(.root) to ("
        } else {
            "@scope("
        };
        let accepted = format!(
            "{prefix}{}.leaf{}){{.child{{color:red}}}}.after{{color:blue}}",
            ":is(".repeat(255),
            ")".repeat(255)
        );
        let report = parse_sheet(&accepted);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let [CssRule::Scope(scope), CssRule::Style(after)] = report.syntax().rules() else {
            panic!("scope and following neighbor")
        };
        let boundary = if limit { scope.limit() } else { scope.root() }.unwrap();
        let [selector] = boundary.selectors() else {
            panic!("one selector")
        };
        let mut selector = selector;
        for _ in 0..255 {
            let CssSelector::PseudoClass(CssPseudoClass::Is(list)) = selector else {
                panic!("retained function")
            };
            let [member] = list.selectors() else {
                panic!("one retained member")
            };
            selector = member;
        }
        assert_eq!(selector, &CssSelector::Class("leaf".to_owned()));
        assert_eq!(
            after.position().byte_offset().value(),
            accepted.find(".after").unwrap()
        );
        let exceeded = format!(
            "{prefix}{}.leaf{}){{.child{{color:red}}}}.after{{color:blue}}",
            ":is(".repeat(256),
            ")".repeat(256)
        );
        let report = parse_sheet(&exceeded);
        let [diagnostic] = report.diagnostics() else {
            panic!("one excess scope diagnostic: {:?}", report.diagnostics())
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            prefix.len() + 255 * 4 + 1
        );
        let [CssRule::Style(after)] = report.syntax().rules() else {
            panic!("only following neighbor retained")
        };
        assert_eq!(
            after.position().byte_offset().value(),
            exceeded.find(".after").unwrap()
        );
        assert_eq!(
            after.declarations()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
    }
}
