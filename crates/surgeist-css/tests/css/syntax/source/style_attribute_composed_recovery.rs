//! Attribute-front composition tests. A style attribute is Syntax 3 (2026 CRD)
//! §5.5.5 block contents (#1353 decision 1): a root `}` ends the contents, the
//! component values of a nested declaration end at `}` (§5.5.6 step 5), and
//! a unit that is not a declaration is reconsumed as a qualified rule with `;`
//! as its stop token (§5.5.3). Existing public original-source, diagnostic and
//! structural-limit contracts supply coordinates and errors.

use crate::css::support::authored_syntax::assert_position_at as position;
use surgeist_css::declaration::{CssDeclaration, CssDeclarationList, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::CssKnownProperty;
use surgeist_css::source::CssParsedOrigin;
use surgeist_css::{CssParseReport, CssParser};

fn region(
    origin: &CssParsedOrigin,
    source: &str,
    start: (usize, u32, u32),
    end: (usize, u32, u32),
) {
    assert_eq!(origin.source().as_str(), source);
    position(origin.span().start(), start);
    position(origin.span().end(), end);
}

fn assert_declaration(
    declaration: &CssDeclaration,
    source: &str,
    property: CssKnownProperty,
    name: ((usize, u32, u32), (usize, u32, u32)),
    value: ((usize, u32, u32), (usize, u32, u32)),
    importance: CssImportance,
    expected_css: &str,
) {
    assert_eq!(declaration.known().unwrap().property(), property);
    assert_eq!(declaration.importance(), importance);
    position(declaration.position().unwrap(), name.0);
    let name_origin = declaration.parsed_name().unwrap();
    let value_origin = declaration.parsed_value().unwrap();
    region(name_origin, source, name.0, name.1);
    region(value_origin, source, value.0, value.1);
    assert!(name_origin.source().same_snapshot(value_origin.source()));
    assert_eq!(
        declaration.serialize_specified(Default::default()).unwrap(),
        expected_css
    );
}

fn report_with_validation_parity(source: &str) -> CssParseReport<CssDeclarationList> {
    let report = CssParser::default().parse_block_contents(source);
    if report.is_clean() {
        assert_eq!(
            CssParser::default()
                .parse_block_contents(source)
                .into_validation_result()
                .unwrap(),
            report.syntax().clone()
        );
    } else {
        let rejected = CssParser::default()
            .parse_block_contents(source)
            .into_validation_result()
            .unwrap_err();
        assert_eq!(rejected.first(), &report.diagnostics()[0]);
        assert_eq!(rejected.diagnostics(), report.diagnostics());
        assert_eq!(rejected.into_diagnostics(), report.diagnostics());
    }
    report
}

#[test]
fn multiple_empty_slots_comments_and_trailing_delimiters_preserve_only_declarations() {
    let empty = report_with_validation_parity(";; /**/ ;\t;\r\n;");
    assert!(empty.is_clean());
    assert!(empty.syntax().is_empty());

    const SOURCE: &str = ";;/**/;\t;color:red;;/*gap*/;width:2px!important;;; /**/\r\n";
    let report = report_with_validation_parity(SOURCE);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [color, width] = report.syntax().as_slice() else {
        panic!("exactly the two authored declarations survive empty slots");
    };
    assert_declaration(
        color,
        SOURCE,
        CssKnownProperty::Color,
        ((9, 0, 9), (14, 0, 14)),
        ((15, 0, 15), (18, 0, 18)),
        CssImportance::Normal,
        "color: red;",
    );
    assert_declaration(
        width,
        SOURCE,
        CssKnownProperty::Width,
        ((28, 0, 28), (33, 0, 33)),
        ((34, 0, 34), (37, 0, 37)),
        CssImportance::Important,
        "width: 2px !important;",
    );
    assert!(
        color
            .parsed_name()
            .unwrap()
            .source()
            .same_snapshot(width.parsed_name().unwrap().source())
    );
}

#[test]
fn adjacent_root_curly_ends_the_contents() {
    for source in ["}color:red;width:2px", "} color:red", "}"] {
        let report = report_with_validation_parity(source);
        assert!(report.syntax().is_empty(), "{source:?}: {report:?}");
        assert!(report.is_clean(), "{source:?}: {report:?}");
    }
}

#[test]
fn supplementary_comments_and_crlf_keep_original_coordinates_up_to_the_root_closer() {
    const SOURCE: &str = "/*😀*/\r\n/*😀*/color:red;}width:2px";
    // Each comment is 8 UTF8 bytes but 6 UTF16 code units; CRLF is one line break.
    assert_eq!(SOURCE.len(), 38);
    let report = report_with_validation_parity(SOURCE);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [color] = report.syntax().as_slice() else {
        panic!("the declaration before the root closer remains and the rest is never read");
    };
    assert_declaration(
        color,
        SOURCE,
        CssKnownProperty::Color,
        ((18, 1, 6), (23, 1, 11)),
        ((24, 1, 12), (27, 1, 15)),
        CssImportance::Normal,
        "color: red;",
    );
    assert_eq!(color.parsed_name().unwrap().source().as_str(), SOURCE);
}

#[test]
fn a_curly_closer_ends_a_declaration_value_and_then_the_contents() {
    // §5.5.6 step 5: the component values of a nested declaration end at `}`,
    // so `color:` has no value and is dropped; the `}` then ends the contents.
    const SOURCE: &str = "color:}red;width:2px";
    let report = report_with_validation_parity(SOURCE);
    assert!(report.syntax().is_empty(), "{report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one dropped declaration: {report:?}");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().known_property().unwrap(),
        CssKnownProperty::Color
    );
    position(diagnostic.span().start(), (0, 0, 0));
    position(diagnostic.span().end(), (6, 0, 6));
    // A value before the closer is retained.
    let report = report_with_validation_parity("width:2px}color:red");
    let [width] = report.syntax().as_slice() else {
        panic!("the declaration before the closer remains");
    };
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(width.known().unwrap().property(), CssKnownProperty::Width);
}

#[test]
fn a_closed_depth257_value_refuses_the_whole_contents() {
    // These are literal native function tokens, not checked component wrappers.
    let accepted = format!(
        "--deep: {}x{};width:2px;",
        "f(".repeat(256),
        ")".repeat(256)
    );
    assert_eq!(accepted.len(), 788);
    let report = report_with_validation_parity(&accepted);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [deep, width] = report.syntax().as_slice() else {
        panic!("the depth256 custom value and its following width are valid");
    };
    assert_eq!(deep.custom().unwrap().name().as_str(), "--deep");
    assert_declaration(
        width,
        &accepted,
        CssKnownProperty::Width,
        ((778, 0, 778), (783, 0, 783)),
        ((784, 0, 784), (787, 0, 787)),
        CssImportance::Normal,
        "width: 2px;",
    );

    let source = format!(
        "--deep: {}x{};width:2px;",
        "f(".repeat(257),
        ")".repeat(257)
    );
    assert_eq!(source.len(), 791);
    // Block contents charges every component once against the 256-level ceiling,
    // so one over-limit value refuses the whole contents, later siblings included.
    let report = report_with_validation_parity(&source);
    assert!(report.syntax().is_empty(), "{report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one resource diagnostic: {report:?}");
    };
    assert!(diagnostic.error().is_resource_refusal(), "{diagnostic:?}");
    position(diagnostic.span().start(), (0, 0, 0));
    position(diagnostic.span().end(), (791, 0, 791));
}
