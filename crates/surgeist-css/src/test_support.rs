use crate::syntax::*;
use crate::{
    CssDeclaration, CssKnownProperty, CssKnownPropertyValueRef, CssParseReport, CssRule, CssSheet,
    CssStyleRule, Error, ErrorKind, parse_sheet,
};

pub(crate) trait CssParseReportTestExt<T> {
    fn is_ok(&self) -> bool;
    fn is_err(&self) -> bool;
    fn unwrap(self) -> T;
    fn expect(self, message: &str) -> T;
    fn unwrap_err(self) -> Error;
    fn expect_err(self, message: &str) -> Error;
    fn unwrap_or_else<F>(self, operation: F) -> T
    where
        F: FnOnce(Error) -> T;
}

/// Borrows a property's sole semantic payload through its schema-selected accessor.
pub(crate) trait SemanticPropertyTestExt {
    type Value;

    fn semantic_value(&self) -> &Self::Value;
}

impl<T> CssParseReportTestExt<T> for CssParseReport<T> {
    fn is_ok(&self) -> bool {
        self.is_clean()
    }

    fn is_err(&self) -> bool {
        !self.is_clean()
    }

    fn unwrap(self) -> T {
        self.expect("stylesheet report contained recovery diagnostics")
    }

    fn expect(self, message: &str) -> T {
        let (syntax, diagnostics) = self.into_parts();
        assert!(diagnostics.is_empty(), "{message}: {diagnostics:?}");
        syntax
    }

    fn unwrap_err(self) -> Error {
        self.expect_err("stylesheet report was clean")
    }

    fn expect_err(self, message: &str) -> Error {
        let (_, diagnostics) = self.into_parts();
        diagnostics
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{message}"))
            .error()
            .clone()
    }

    fn unwrap_or_else<F>(self, operation: F) -> T
    where
        F: FnOnce(Error) -> T,
    {
        let (syntax, diagnostics) = self.into_parts();
        diagnostics
            .into_iter()
            .next()
            .map_or(syntax, |diagnostic| operation(diagnostic.error().clone()))
    }
}
macro_rules! define_test_property {
    ($input:ident, $numeric:ident;
        All, $all_canonical:literal, [$($all_alias:literal),*], $all_stable_id:literal,
        $all_value:ty, $all_parser:ident, $all_dispatch:block $(, expansion = $all_kind:ident { $($all_metadata:tt)* })?;
        $(
        $variant:ident, $canonical:literal, [$($alias:literal),*], $stable_id:literal,
        $value:ty, $wrapper:ident, $accessor:ident, $parser:ident, $dispatch:block
        $(, expansion = $expansion:ident { $($metadata:tt)* })?;
    )*) => {
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub(crate) enum CssProperty {
            SvgGlyphOrientationVertical,
            All,
            $($variant,)*
            Custom(CssCustomPropertyName),
        }

        impl From<CssKnownProperty> for CssProperty {
            fn from(value: CssKnownProperty) -> Self {
                match value {
                    CssKnownProperty::All => Self::All,
                    $(CssKnownProperty::$variant => Self::$variant,)*
                }
            }
        }

        $(
            impl SemanticPropertyTestExt for crate::$wrapper {
                type Value = $value;

                fn semantic_value(&self) -> &Self::Value {
                    self.$accessor()
                }
            }
        )*
    };
}

crate::properties::property_schema!(define_test_property, test_input, numeric_input);

impl PartialEq<&CssProperty> for CssProperty {
    fn eq(&self, other: &&CssProperty) -> bool {
        self == *other
    }
}

pub(crate) fn declaration_property(declaration: &CssDeclaration) -> CssProperty {
    declaration_body_property(declaration.body())
}

pub(crate) fn declaration_body_property(body: &CssDeclarationBody) -> CssProperty {
    #[expect(
        unreachable_patterns,
        reason = "crate tests intentionally demonstrate wildcard-compatible public enum matching"
    )]
    match body {
        CssDeclarationBody::Known(known) => known.property().into(),
        CssDeclarationBody::Custom(custom) => CssProperty::Custom(custom.name().clone()),
        CssDeclarationBody::SvgGlyphOrientationVertical(_) => {
            CssProperty::SvgGlyphOrientationVertical
        }
        _ => unreachable!("test adapter saw a future declaration-body branch"),
    }
}

pub(crate) struct AcceptedDeclarationCase {
    pub(crate) label: &'static str,
    pub(crate) property_name: &'static str,
    pub(crate) authored_value: &'static str,
    pub(crate) expected_property: CssProperty,
    pub(crate) expected_global: crate::CssGlobalKeyword,
}

impl AcceptedDeclarationCase {
    pub(crate) fn assert_accepts(&self) -> CssDeclaration {
        let declaration = parse_single_declaration(self.property_name, self.authored_value);
        assert_eq!(
            declaration.property(),
            &self.expected_property,
            "{} parsed to the wrong property",
            self.label,
        );
        assert_eq!(
            declaration.known().and_then(|known| known.global()),
            Some(self.expected_global),
            "{} parsed to the wrong global value",
            self.label,
        );
        assert!(
            declaration.known().unwrap().property_value().is_none(),
            "{} global value must have no ordinary property payload",
            self.label,
        );
        declaration
    }
}

pub(crate) struct AcceptedValueCase {
    pub(crate) label: &'static str,
    pub(crate) property_name: &'static str,
    pub(crate) authored_value: &'static str,
    pub(crate) expected_property: CssProperty,
    pub(crate) assert_value: fn(CssKnownPropertyValueRef<'_>),
}

impl AcceptedValueCase {
    pub(crate) fn assert_accepts(&self) -> CssDeclaration {
        let declaration = parse_single_declaration(self.property_name, self.authored_value);
        assert_eq!(
            declaration.property(),
            &self.expected_property,
            "{} parsed to the wrong property",
            self.label,
        );
        (self.assert_value)(
            declaration
                .known()
                .and_then(|known| known.property_value())
                .expect("accepted ordinary value has an exact property wrapper"),
        );
        declaration
    }
}

/// One exact diagnostic kind, with the authored identity it names.
///
/// Each variant accepts exactly one [`ErrorKind`] variant, so a rejection by an
/// unrelated rule (another property, a tokenizer failure or a different
/// production) fails the expectation.
pub(crate) enum ExpectedErrorKind {
    UnexpectedToken,
    InvalidSelector,
    InvalidPropertyValue {
        property: &'static str,
    },
    /// A `<color>` value rejected by the color grammar; its diagnostic names
    /// no property, so the responsible token identifies the failure.
    InvalidColorSyntax {
        encountered: &'static str,
    },
    UnknownAtRule {
        name: &'static str,
    },
    UnknownProperty {
        name: &'static str,
    },
}

impl ExpectedErrorKind {
    fn assert_matches(&self, actual: &ErrorKind, label: &str) {
        let matched = match (self, actual) {
            (Self::UnexpectedToken, ErrorKind::UnexpectedToken(_))
            | (Self::InvalidSelector, ErrorKind::InvalidSelector(_)) => true,
            (Self::InvalidPropertyValue { property }, ErrorKind::InvalidPropertyValue(detail)) => {
                let expected = crate::validation::property_for_supported_name(property);
                assert!(
                    expected.is_some(),
                    "{label}: `{property}` is not a known property"
                );
                expected == Some(detail.property())
            }
            (Self::InvalidColorSyntax { encountered }, ErrorKind::InvalidColorSyntax(detail)) => {
                detail
                    .encountered()
                    .is_some_and(|token| token.authored() == *encountered)
            }
            (Self::UnknownAtRule { name }, ErrorKind::UnknownAtRule(detail)) => {
                *name == detail.name().as_str()
            }
            (Self::UnknownProperty { name }, ErrorKind::UnknownProperty(detail)) => {
                *name == detail.name().as_str()
            }
            _ => false,
        };
        assert!(
            matched,
            "{label} rejected with unexpected error kind: {actual:?}"
        );
    }
}

pub(crate) struct RejectedSheetCase {
    pub(crate) label: &'static str,
    pub(crate) input: &'static str,
    pub(crate) expected_error: ExpectedErrorKind,
}

impl RejectedSheetCase {
    pub(crate) fn assert_rejects(&self) -> Error {
        let error = parse_sheet(self.input).expect_err("invalid CSS must report a diagnostic");
        self.expected_error.assert_matches(error.kind(), self.label);
        error
    }
}

pub(crate) struct RejectedDeclarationCase {
    pub(crate) label: &'static str,
    pub(crate) property_name: &'static str,
    pub(crate) authored_value: &'static str,
    pub(crate) expected_error: ExpectedErrorKind,
    pub(crate) property_name_should_be_recognized: bool,
}

impl RejectedDeclarationCase {
    pub(crate) fn assert_rejects(&self) -> Error {
        let css = declaration_sheet(self.property_name, self.authored_value);
        let error = assert_sheet_rejected(&css, &self.expected_error);
        assert_eq!(
            matches!(error.kind(), ErrorKind::UnknownProperty(_)),
            !self.property_name_should_be_recognized,
            "{} property-name recognition mismatch",
            self.label,
        );
        error
    }
}

pub(crate) fn assert_accepts_value_cases(cases: &[AcceptedValueCase]) {
    for case in cases {
        case.assert_accepts();
    }
}

pub(crate) fn assert_accepts_declarations(cases: &[AcceptedDeclarationCase]) {
    for case in cases {
        case.assert_accepts();
    }
}

pub(crate) fn assert_rejects_declarations(cases: &[RejectedDeclarationCase]) {
    for case in cases {
        case.assert_rejects();
    }
}

pub(crate) fn assert_rejects_sheets(cases: &[RejectedSheetCase]) {
    for case in cases {
        case.assert_rejects();
    }
}

pub(crate) fn parse_single_declaration(
    property_name: &str,
    authored_value: &str,
) -> CssDeclaration {
    parse_single_declaration_from_sheet(&declaration_sheet(property_name, authored_value))
}

pub(crate) fn parse_single_declaration_from_sheet(input: &str) -> CssDeclaration {
    let sheet = parse_sheet(input).unwrap_or_else(|error| panic!("{input} should parse: {error}"));
    only_declaration(&sheet, input)
}

pub(crate) fn assert_sheet_rejected(input: &str, expected_error: &ExpectedErrorKind) -> Error {
    let error = parse_sheet(input).expect_err("invalid CSS must report a diagnostic");
    expected_error.assert_matches(error.kind(), input);
    error
}

/// Asserts that the sole declaration of the sole style rule in `input` is
/// dropped with exactly one diagnostic while its rule is retained: CSS Syntax 3
/// (CRD 2026-10-01) §5.5.5-§5.5.6 consumes and discards an invalid declaration
/// inside a block's contents without discarding the enclosing rule. Returns
/// that diagnostic.
pub(crate) fn assert_drops_sole_declaration(input: &str) -> Error {
    let report = parse_sheet(input);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "{input}: exactly one diagnostic: {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(
        diagnostic.action(),
        crate::CssRecoveryAction::DropDeclaration,
        "{input}: the invalid declaration alone is dropped"
    );
    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("{input}: the enclosing style rule is retained");
    };
    assert!(
        rule.declarations().is_empty(),
        "{input}: the invalid declaration is not retained"
    );
    diagnostic.error().clone()
}

/// Runs `operation` alone in a re-executed child of this test binary, so its
/// process-global allocation counters see no other test.
///
/// libtest exits successfully when `--exact` selects no test, so the parent
/// also requires the `completed` line that the child prints only after
/// `operation` returns. A renamed or misspelled `test` therefore fails.
#[track_caller]
pub(crate) fn run_in_isolated_child(
    test: &str,
    child_environment: &str,
    completed: &str,
    operation: impl FnOnce(),
) {
    if std::env::var(child_environment).as_deref() == Ok(test) {
        operation();
        println!("{completed}");
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture"])
        .env(child_environment, test)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run and reap the isolated child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "isolated child failed: {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status,
    );
    assert!(
        stdout.contains(completed),
        "the isolated child must select `{test}` and complete it\nstdout:\n{stdout}\nstderr:\n{stderr}",
    );
}

fn declaration_sheet(property_name: &str, authored_value: &str) -> String {
    format!(".test {{ {property_name}: {authored_value}; }}")
}

fn only_declaration(sheet: &CssSheet, input: &str) -> CssDeclaration {
    let [rule] = sheet.rules() else {
        panic!("{input} should parse exactly one rule");
    };
    let rule = style_rule(rule);
    let [declaration] = rule.declarations().as_slice() else {
        panic!("{input} should parse exactly one declaration");
    };
    declaration.clone()
}

fn style_rule(rule: &CssRule) -> &CssStyleRule {
    match rule {
        CssRule::Style(rule) => rule,
        unexpected => panic!("expected style rule, got {unexpected:?}"),
    }
}
