//! Helpers shared by the at-rule and rule-structure tests, and typed
//! rejection evidence for their grammar failures.

use surgeist_css::*;

/// The sheet of the clean parse of `source`.
#[track_caller]
pub fn clean_sheet(source: &str) -> CssSheet {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}

/// Asserts that `report` is clean and that its validation succeeds.
#[track_caller]
pub fn assert_clean_and_valid<T: Clone>(report: &CssParseReport<T>) {
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(report.clone().into_validation_result().is_ok());
}

/// Asserts that the clean sheet `source` serializes as `expected` without
/// changing, and that `expected` reparses cleanly to the same text.
#[track_caller]
pub fn assert_exact_specified_sheet(source: &str, expected: &str) {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let before = report.clone();
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
    assert_eq!(report, before);
    let emitted = parse_sheet(expected);
    assert!(
        emitted.is_clean(),
        "{expected}: {:?}",
        emitted.diagnostics()
    );
    assert_eq!(
        emitted
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
}

/// The sole rule of the clean sheet `source`, which must be `@counter-style`.
#[track_caller]
pub fn sole_counter_style(source: &str) -> CssCounterStyleRule {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("one counter style rule: {source}");
    };
    rule.clone()
}

/// The sole rule of `sheet`, which must be `@keyframes`.
#[track_caller]
pub fn sole_keyframes(sheet: &CssSheet) -> &CssKeyframesRule {
    let [CssRule::Keyframes(rule)] = sheet.rules() else {
        panic!("one keyframes rule: {sheet:?}");
    };
    rule
}

/// The retained `@font-face` rule of a report whose sheet is that rule and
/// one following style rule; the face names the family `Demo` and uses
/// `font-display: swap`.
#[track_caller]
pub fn retained_demo_font_face(report: &CssParseReport<CssSheet>) -> &CssFontFaceRule {
    let [CssRule::FontFace(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("expected retained font-face and following style rule");
    };
    assert_eq!(
        ordinary_face!(rule.descriptors(), FontFamily)
            .unwrap()
            .as_str(),
        "Demo"
    );
    assert_eq!(
        ordinary_face!(rule.descriptors(), FontDisplay).unwrap(),
        &CssFontDisplay::Swap
    );
    rule
}

/// The ordinary value of the clean `kind` descriptor fragment `source`.
#[track_caller]
pub fn ordinary_font_face_descriptor(
    source: &str,
    kind: CssFontFaceDescriptorKind,
) -> CssFontFaceDescriptorValue {
    let report = parse_font_face_descriptor_value(source, kind);
    assert!(
        report.is_clean(),
        "{}: {source}: {:?}",
        kind.css_name(),
        report.diagnostics()
    );
    let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(value)) = report.syntax() else {
        panic!("ordinary {} descriptor: {source}", kind.css_name())
    };
    value.clone()
}

/// An import rule checked from the component values of `source` with the
/// default namespace context.
#[track_caller]
pub fn import_from_components(source: &str) -> CssImportRule {
    CssImportRule::try_from_components(
        parse_component_values(source).unwrap(),
        &CssNamespaceContext::default(),
        Default::default(),
    )
    .unwrap()
}

/// The first page pseudo-class of the first selector of `page`.
pub fn page_pseudo(page: &CssPageRule) -> Option<CssPagePseudo> {
    page.selectors()
        .selectors()
        .first()
        .and_then(|s| s.pseudos().first().copied())
}

/// The selector of a scope boundary list with exactly one selector member.
#[track_caller]
pub fn sole_scope_selector(list: &CssScopeSelectorList) -> &CssSelector {
    let [CssScopeSelector::Selector(selector)] = list.selectors() else {
        panic!("one boundary selector: {list:?}")
    };
    selector
}

/// The declarations of a normalized sheet, in order.
pub fn normalized_declarations(sheet: &CssNormalizedSheet) -> Vec<&CssNormalizedDeclaration> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect()
}

/// The rule contexts of a normalized sheet, in order.
pub fn normalized_rule_contexts(sheet: &CssNormalizedSheet) -> Vec<&CssRuleContext> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => Some(context),
            _ => None,
        })
        .collect()
}

/// The page rules of a normalized sheet's rule contexts, in order.
pub fn normalized_pages(sheet: &CssNormalizedSheet) -> Vec<&CssPageRule> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::Page(page) => Some(page),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// The normalized rule context whose original header starts where `marker`
/// first occurs in `source`.
#[track_caller]
pub fn rule_at<'a>(
    sheet: &'a CssNormalizedSheet,
    source: &str,
    marker: &str,
) -> &'a CssRuleContext {
    let offset = source.find(marker).unwrap();
    sheet
        .items()
        .iter()
        .find_map(|item| match item {
            CssNormalizedItem::Rule(rule)
                if rule
                    .position()
                    .is_some_and(|position| position.byte_offset().value() == offset) =>
            {
                Some(rule)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("retained original rule occurrence at {marker:?}"))
}

/// The authored text of a parsed declaration's name and of its trimmed value.
#[track_caller]
pub fn declaration_text(declaration: &CssDeclaration) -> (&str, &str) {
    let name = declaration
        .parsed_name()
        .expect("original parsed declaration name");
    let value = declaration
        .parsed_value()
        .expect("original parsed declaration value");
    let name_text = &name.source().as_str()
        [name.span().start().byte_offset().value()..name.span().end().byte_offset().value()];
    let value_text = &value.source().as_str()
        [value.span().start().byte_offset().value()..value.span().end().byte_offset().value()];
    (name_text, value_text.trim())
}

/// The authored text covered by a parsed origin.
pub fn origin_text(origin: &CssParsedOrigin) -> &str {
    &origin.source().as_str()
        [origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value()]
}

/// Asserts that a parsed origin covers bytes `start..end` of `source`.
#[track_caller]
pub fn assert_origin_bytes(origin: &CssParsedOrigin, source: &str, start: usize, end: usize) {
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}

/// The responsible token an [`ExpectedRejection`] names.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Responsible {
    /// The expectation does not constrain the responsible token.
    Unchecked,
    /// The input ended before the production was complete; no token is
    /// responsible.
    End,
    /// The authored text of the first token that violates the production.
    Token(String),
}

/// A grammar rejection derived from the specification's production: its
/// diagnostic code, the at-rule and property or descriptor it names, and
/// the first token that violates the production.
///
/// Built with [`ExpectedRejection::new`] and the `in_rule`, `named`, `at`
/// and `at_end` builders; an unset rule or name must be absent from the
/// actual diagnostic, while an unset token is not checked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedRejection {
    code: CssErrorCode,
    at_rule: Option<String>,
    name: Option<String>,
    responsible: Responsible,
}

/// The identity read from one structured diagnostic detail.
#[derive(Debug)]
struct ActualRejection<'a> {
    code: CssErrorCode,
    at_rule: Option<&'a str>,
    name: Option<&'a str>,
    encountered: Option<&'a str>,
}

impl<'a> ActualRejection<'a> {
    fn of(kind: &'a ErrorKind) -> Self {
        fn token(summary: Option<&CssTokenSummary>) -> Option<&str> {
            summary.map(CssTokenSummary::authored)
        }
        let (code, at_rule, name, encountered) = match kind {
            ErrorKind::UnexpectedEnd(_) => (CssErrorCode::UnexpectedEnd, None, None, None),
            ErrorKind::UnexpectedToken(detail) => (
                CssErrorCode::UnexpectedToken,
                None,
                None,
                token(Some(detail.encountered())),
            ),
            ErrorKind::InvalidAtRulePlacement(detail) => (
                CssErrorCode::InvalidAtRulePlacement,
                Some(detail.name().as_str()),
                None,
                None,
            ),
            ErrorKind::InvalidAtRulePrelude(detail) => (
                CssErrorCode::InvalidAtRulePrelude,
                Some(detail.name().as_str()),
                None,
                token(detail.encountered()),
            ),
            ErrorKind::InvalidAtRuleBody(detail) => (
                CssErrorCode::InvalidAtRuleBody,
                Some(detail.name().as_str()),
                None,
                token(detail.encountered()),
            ),
            ErrorKind::UnknownAtRule(detail) => (
                CssErrorCode::UnknownAtRule,
                Some(detail.name().as_str()),
                None,
                None,
            ),
            ErrorKind::UnsupportedAtRule(detail) => (
                CssErrorCode::UnsupportedAtRule,
                Some(detail.name().as_str()),
                None,
                None,
            ),
            ErrorKind::InvalidQualifiedRule(detail) => (
                CssErrorCode::InvalidQualifiedRule,
                None,
                None,
                token(detail.encountered()),
            ),
            ErrorKind::InvalidSelector(detail) => (
                CssErrorCode::InvalidSelector,
                None,
                None,
                token(detail.encountered()),
            ),
            ErrorKind::UnknownProperty(detail) => (
                CssErrorCode::UnknownProperty,
                None,
                Some(detail.name().as_str()),
                None,
            ),
            ErrorKind::InvalidPropertyValue(detail) => (
                CssErrorCode::InvalidPropertyValue,
                None,
                Some(detail.property().canonical_name()),
                token(detail.encountered()),
            ),
            ErrorKind::InvalidDeclarationAnnotation(detail) => {
                let (at_rule, name) = match detail.context() {
                    CssDeclarationContextRef::Descriptor {
                        at_rule,
                        descriptor,
                    } => (Some(at_rule.as_str()), Some(descriptor.as_str())),
                    CssDeclarationContextRef::KnownProperty(property)
                    | CssDeclarationContextRef::Keyframe(property) => {
                        (None, Some(property.canonical_name()))
                    }
                    CssDeclarationContextRef::CustomProperty(name)
                    | CssDeclarationContextRef::KeyframeCustomProperty(name) => {
                        (None, Some(name.as_str()))
                    }
                    CssDeclarationContextRef::SvgGlyphOrientationVertical
                    | CssDeclarationContextRef::KeyframeSvgGlyphOrientationVertical => {
                        (None, Some("glyph-orientation-vertical"))
                    }
                    other => panic!("annotation context is not mapped for {other:?}"),
                };
                (
                    CssErrorCode::InvalidDeclarationAnnotation,
                    at_rule,
                    name,
                    token(Some(detail.encountered())),
                )
            }
            ErrorKind::UnknownDescriptor(detail) => (
                CssErrorCode::UnknownDescriptor,
                Some(detail.at_rule().as_str()),
                Some(detail.descriptor().as_str()),
                None,
            ),
            ErrorKind::InvalidDescriptorValue(detail) => (
                CssErrorCode::InvalidDescriptorValue,
                Some(detail.at_rule().as_str()),
                Some(detail.descriptor().as_str()),
                token(detail.encountered()),
            ),
            ErrorKind::InvalidColorSyntax(detail) => (
                CssErrorCode::InvalidColorSyntax,
                None,
                None,
                token(detail.encountered()),
            ),
            other => panic!("rejection identity is not mapped for {other:?}"),
        };
        Self {
            code,
            at_rule,
            name,
            encountered,
        }
    }
}

impl ExpectedRejection {
    /// A rejection with `code` that names no rule, property or descriptor
    /// and does not constrain its responsible token.
    pub fn new(code: CssErrorCode) -> Self {
        Self {
            code,
            at_rule: None,
            name: None,
            responsible: Responsible::Unchecked,
        }
    }

    /// Names the at-rule, without its `@`.
    pub fn in_rule(mut self, at_rule: &str) -> Self {
        self.at_rule = Some(at_rule.to_owned());
        self
    }

    /// Names the property or descriptor.
    pub fn named(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Names the authored text of the first token violating the production.
    pub fn at(mut self, token: &str) -> Self {
        self.responsible = Responsible::Token(token.to_owned());
        self
    }

    /// States that the input ended before the production was complete.
    pub fn at_end(mut self) -> Self {
        self.responsible = Responsible::End;
        self
    }

    /// Asserts that a structured diagnostic detail is this rejection.
    #[track_caller]
    pub fn assert_kind(&self, kind: &ErrorKind) {
        let actual = ActualRejection::of(kind);
        let responsible = match &self.responsible {
            Responsible::Unchecked => true,
            Responsible::End => actual.encountered.is_none(),
            Responsible::Token(token) => actual.encountered == Some(token.as_str()),
        };
        assert!(
            actual.code == self.code
                && actual.at_rule == self.at_rule.as_deref()
                && actual.name == self.name.as_deref()
                && responsible,
            "expected {self:?}, got {actual:?} from {kind:?}"
        );
    }

    /// Asserts that a diagnostic error is this rejection.
    #[track_caller]
    pub fn assert_error(&self, error: &Error) {
        assert_eq!(error.code(), self.code, "{error:?}");
        self.assert_kind(error.kind());
    }

    /// Asserts that the first diagnostic of `report` is this rejection and
    /// that validating the report fails with exactly its diagnostics.
    #[track_caller]
    pub fn assert_first<T: Clone + std::fmt::Debug>(&self, report: &CssParseReport<T>) {
        let [first, ..] = report.diagnostics() else {
            panic!("expected {self:?} from a clean report: {report:?}")
        };
        self.assert_error(first.error());
        let failure = report.clone().into_validation_result().unwrap_err();
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }

    /// Asserts that validating the sheet `source` fails and that its first
    /// diagnostic is this rejection.
    #[track_caller]
    pub fn assert_sheet(&self, source: &str) {
        let failure = validate_sheet(source).unwrap_err();
        self.assert_error(failure.first().error());
    }
}

/// An `InvalidDescriptorValue` rejection of the `descriptor` of `@at_rule`.
pub fn invalid_descriptor(at_rule: &str, descriptor: &str) -> ExpectedRejection {
    ExpectedRejection::new(CssErrorCode::InvalidDescriptorValue)
        .in_rule(at_rule)
        .named(descriptor)
}

/// An `InvalidDeclarationAnnotation` rejection of the `descriptor` of
/// `@at_rule` whose responsible token is a top-level `!`, which
/// `<declaration-value>` excludes (CSS Syntax 3, section 7.2).
pub fn rejected_bang(at_rule: &str, descriptor: &str) -> ExpectedRejection {
    ExpectedRejection::new(CssErrorCode::InvalidDeclarationAnnotation)
        .in_rule(at_rule)
        .named(descriptor)
        .at("!")
}

/// The rejection of the `descriptor` of `@at_rule` whose first violating
/// token is `token`: [`rejected_bang`] for a top-level `!`, otherwise an
/// [`invalid_descriptor`] rejection at `token`.
pub fn descriptor_rejected_at(at_rule: &str, descriptor: &str, token: &str) -> ExpectedRejection {
    if token == "!" {
        rejected_bang(at_rule, descriptor)
    } else {
        invalid_descriptor(at_rule, descriptor).at(token)
    }
}
