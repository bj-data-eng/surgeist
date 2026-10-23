//! Attribute-front composition tests. Style Attributes REC20131107
//! §3 supplies empty slots and the unbraced-root closer rule (a `}` is an
//! invalid token, not a terminator); selected CSS2 §4.2 supplies
//! malformed-declaration recovery, read with Syntax 3 (2026 CRD) §5.5.6
//! "consume the remnants of a bad declaration" (not nested): a malformed start
//! is discarded through the next root `;`. Existing public original-source,
//! diagnostic and structural-limit contracts supply coordinates and errors.

use crate::css::support::authored_syntax::assert_position_at as position;
use surgeist_css::*;

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
    let report = parse_style_attribute(source);
    if report.is_clean() {
        assert_eq!(
            validate_style_attribute(source).unwrap(),
            report.syntax().clone()
        );
    } else {
        let rejected = validate_style_attribute(source).unwrap_err();
        assert_eq!(rejected.first(), &report.diagnostics()[0]);
        assert_eq!(rejected.diagnostics(), report.diagnostics());
        assert_eq!(rejected.into_diagnostics(), report.diagnostics());
    }
    report
}

fn stray_curly(
    diagnostic: &CssRecoveryDiagnostic,
    start: (usize, u32, u32),
    end: (usize, u32, u32),
) {
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedToken);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    position(diagnostic.error().position(), start);
    position(diagnostic.span().start(), start);
    position(diagnostic.span().end(), end);
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::UnexpectedToken,
        "a real stray root token"
    );
    assert_eq!(
        detail.encountered().unwrap().kind(),
        CssTokenKind::CloseCurlyBracket
    );
    assert_eq!(detail.encountered().unwrap().authored(), "}");
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
fn adjacent_root_curly_starts_a_malformed_declaration_through_the_next_semicolon() {
    const SOURCE: &str = "}color:red;width:2px";
    let report = report_with_validation_parity(SOURCE);
    let [width] = report.syntax().as_slice() else {
        panic!("the closer's malformed declaration is discarded; the next one remains");
    };
    let [diagnostic] = report.diagnostics() else {
        panic!("the malformed declaration is one rejected unit");
    };
    stray_curly(diagnostic, (0, 0, 0), (10, 0, 10));
    assert_declaration(
        width,
        SOURCE,
        CssKnownProperty::Width,
        ((11, 0, 11), (16, 0, 16)),
        ((17, 0, 17), (20, 0, 20)),
        CssImportance::Normal,
        "width: 2px;",
    );
}

#[test]
fn supplementary_comments_and_crlf_keep_original_root_error_and_sibling_coordinates() {
    const SOURCE: &str = "/*😀*/\r\n/*😀*/}color:red;width:2px";
    // Each comment is 8 UTF8 bytes but 6 UTF16 code units; CRLF is one line break.
    assert_eq!(SOURCE.len(), 38);
    let report = report_with_validation_parity(SOURCE);
    let [width] = report.syntax().as_slice() else {
        panic!("the Unicode prefix keeps original coordinates for the remnant and sibling");
    };
    let [diagnostic] = report.diagnostics() else {
        panic!("the closer's malformed declaration has one diagnostic");
    };
    stray_curly(diagnostic, (18, 1, 6), (28, 1, 16));
    assert_declaration(
        width,
        SOURCE,
        CssKnownProperty::Width,
        ((29, 1, 17), (34, 1, 22)),
        ((35, 1, 23), (38, 1, 26)),
        CssImportance::Normal,
        "width: 2px;",
    );
    assert_eq!(width.parsed_name().unwrap().source().as_str(), SOURCE);
}

#[test]
fn curly_inside_a_value_rejects_its_complete_declaration_instead_of_repairing_it() {
    const SOURCE: &str = "color:}red;width:2px";
    let report = report_with_validation_parity(SOURCE);
    let [width] = report.syntax().as_slice() else {
        panic!("the malformed color unit is discarded and the later width remains");
    };
    let [diagnostic] = report.diagnostics() else {
        panic!("one malformed value is one rejected declaration");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    position(diagnostic.error().position(), (6, 0, 6));
    position(diagnostic.span().start(), (0, 0, 0));
    position(diagnostic.span().end(), (11, 0, 11));
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "the ordinary property's value boundary owns this closer"
    );
    assert_eq!(detail.known_property().unwrap(), CssKnownProperty::Color);
    assert_eq!(
        detail.encountered().unwrap().kind(),
        CssTokenKind::CloseCurlyBracket
    );
    assert_eq!(detail.encountered().unwrap().authored(), "}");
    assert_declaration(
        width,
        SOURCE,
        CssKnownProperty::Width,
        ((11, 0, 11), (16, 0, 16)),
        ((17, 0, 17), (20, 0, 20)),
        CssImportance::Normal,
        "width: 2px;",
    );
}

#[test]
fn a_closed_depth257_value_has_local_resource_recovery_and_keeps_the_later_width() {
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
    // Prefix8 + 257 opening tokens of2 + x1 + 257 closing tokens + ;1 =781.
    // The first denied function is prefix8 + 256*2 =520.
    assert_eq!(source.len(), 791);
    let report = report_with_validation_parity(&source);
    let [width] = report.syntax().as_slice() else {
        panic!("the complete over-limit unit is dropped locally before the valid sibling");
    };
    let [diagnostic] = report.diagnostics() else {
        panic!("a fully closed resource-denied unit needs no implicit-EOF repair");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    position(diagnostic.error().position(), (520, 0, 520));
    position(diagnostic.span().start(), (0, 0, 0));
    position(diagnostic.span().end(), (781, 0, 781));
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::NestingLimit,
        "the structural resource owner supplies the typed failure"
    );
    assert_eq!(detail.nesting_limit().unwrap(), 256);
    assert_eq!(detail.production().unwrap().as_str(), "css.declaration");
    assert_declaration(
        width,
        &source,
        CssKnownProperty::Width,
        ((781, 0, 781), (786, 0, 786)),
        ((787, 0, 787), (790, 0, 790)),
        CssImportance::Normal,
        "width: 2px;",
    );
}
