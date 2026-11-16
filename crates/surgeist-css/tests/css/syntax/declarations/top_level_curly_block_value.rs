//! A non-custom declaration value that holds a top-level `{}` block beside any
//! other non-whitespace value is refused in every front, whether or not the value
//! also holds a substitution function. Syntax 3 section 5.5.6 (consume a
//! declaration), step 8: such a declaration returns nothing, unless its name is a
//! custom property name; a `{}` block as the entire value is not refused by that
//! step. `CssParser::parse_block_contents` applies the step, so a declaration any
//! front retains must be one the block-contents front would retain too.
//!
//! The expected offsets are built from the literal source, never read from the
//! parser.

use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValueLimits, CssComponentValues, CssSerializedOrigin};
use surgeist_css::declaration::{CssCustomPropertyName, CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{
    CssExpansion, CssKnownProperty, CssPropertyNameRef, CssPropertyTarget, CssPropertyTextTarget,
    CssSvgGlyphOrientationVerticalDeclaration,
};
use surgeist_css::rule::CssRule;
use surgeist_css::rule::color_profile::{
    CssColorProfileDescriptorKind, CssColorProfileDescriptorValue,
};
use surgeist_css::rule::counter_style::{
    CssCounterStyleDescriptorKind, CssCounterStyleDescriptorValue,
};
use surgeist_css::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind,
};
use surgeist_css::rule::font_feature_values::{
    CssFontFeatureDisplayValue, CssFontFeatureValue, CssFontFeatureValueKind,
};
use surgeist_css::rule::font_palette_values::{
    CssFontPaletteDescriptorKind, CssFontPaletteDescriptorValue,
};
use surgeist_css::rule::page::{CssPageDescriptorKind, CssPageDescriptorValue};

/// Values with a top-level `{}` block beside at least one other non-whitespace
/// value, each holding a substitution function or a recovery.
const BESIDE_SUBSTITUTION: &[&str] = &[
    "{} var(--x)",
    "var(--x) {}",
    "{} var(--x",
    "var(--x) { a",
    "{} {} var(--x)",
    "{a b} var(--x)",
    "var(--x) {a; b}",
    "{} env(--x)",
    "env(--x) {}",
    "  /*c*/ {} /*c*/ var(--x)",
    "var(--x)/*c*/{}",
    "f(a b) {} var(--x)",
    "{} var(--x) var(--y)",
    "var(--x) {} var(--y)",
];

/// Values with a top-level `{}` block beside another value and no substitution
/// function: no grammar of the property admits them.
const BESIDE_VALUE: &[&str] = &[
    "{} 1px",
    "1px {}",
    "{} {}",
    "auto {}",
    "{a} {b}",
    "{} /*c*/ 1px",
];

/// A `{}` block as the entire value.
const WHOLE: &[&str] = &["{}", " {} ", "/*c*/{}/*c*/", "{a b}", "{ var(--x) }"];

/// The values above that end inside a function or a block, which the checked
/// constructors refuse first for that closure.
const UNCLOSED: &[&str] = &["{} var(--x", "var(--x) { a"];

/// What the diagnostic of a refused value reports: its code, its action, its
/// offset and the token it names.
type Rejection = (CssErrorCode, CssRecoveryAction, usize, Option<String>);

#[track_caller]
fn first_rejection<T>(source: &str, report: &surgeist_css::CssParseReport<T>) -> Rejection {
    let diagnostic = report
        .diagnostics()
        .first()
        .unwrap_or_else(|| panic!("a diagnostic for {source:?}"));
    let error = diagnostic.error();
    (
        error.code(),
        diagnostic.action(),
        error.position().byte_offset().value(),
        error.encountered().map(|token| token.authored().to_owned()),
    )
}

/// The refusal the declaration front gives a block beside other values: the
/// unexpected opening brace, located on it.
fn brace_refusal(source: &str) -> Rejection {
    (
        CssErrorCode::UnexpectedToken,
        CssRecoveryAction::RejectInput,
        source.find('{').unwrap(),
        Some("{".to_owned()),
    )
}

fn targets() -> Vec<(&'static str, CssPropertyTextTarget<'static>)> {
    vec![
        (
            "width",
            CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width)),
        ),
        (
            "width grammar",
            CssPropertyTextTarget::Grammar(CssKnownProperty::Width.grammar()),
        ),
        (
            "margin-top",
            CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::MarginTop)),
        ),
        (
            "glyph-orientation-vertical",
            CssPropertyTextTarget::Property(CssPropertyNameRef::SvgGlyphOrientationVertical),
        ),
    ]
}

#[track_caller]
fn assert_refused_text(source: &str, target: CssPropertyTextTarget<'_>, what: &str) {
    for importance in [CssImportance::Normal, CssImportance::Important] {
        let report = CssParser::default().parse_property_value(source, target, importance);
        assert!(
            report.syntax().is_none(),
            "{what} {source:?} {importance:?}: a block beside other values is not retained"
        );
        assert_eq!(
            first_rejection(source, &report),
            brace_refusal(source),
            "{what} {source:?} {importance:?}"
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput),
            "{what} {source:?}: the input is rejected, not recovered"
        );
    }
}

#[test]
fn the_property_value_text_front_refuses_a_block_beside_other_values() {
    for (what, target) in targets() {
        for source in BESIDE_SUBSTITUTION.iter().chain(BESIDE_VALUE) {
            assert_refused_text(source, target, what);
        }
    }
}

#[test]
fn the_declaration_front_refuses_the_same_values_so_the_fronts_agree() {
    for source in BESIDE_SUBSTITUTION.iter().chain(BESIDE_VALUE) {
        for name in ["width", "margin-top", "glyph-orientation-vertical"] {
            let report = CssParser::default().parse_block_contents(format!("{name}: {source}"));
            assert!(
                report.syntax().is_empty(),
                "{name}: {source:?}: the block-contents front retains nothing"
            );
            assert_eq!(
                report.diagnostics().first().map(|d| d.error().code()),
                Some(CssErrorCode::UnexpectedToken),
                "{name}: {source:?}: the declaration front names the unexpected block"
            );
        }
    }
}

#[test]
fn the_svg_glyph_attribute_front_refuses_a_block_beside_other_values() {
    for source in BESIDE_SUBSTITUTION.iter().chain(BESIDE_VALUE) {
        let report = CssSvgGlyphOrientationVerticalDeclaration::parse_attribute(
            &CssParser::default(),
            source,
        );
        assert!(report.syntax().is_none(), "{source:?}");
        assert_eq!(
            first_rejection(source, &report),
            brace_refusal(source),
            "{source:?}"
        );
    }
}

#[test]
fn a_block_as_the_entire_value_keeps_its_result() {
    for (what, target) in targets() {
        for source in WHOLE {
            let report =
                CssParser::default().parse_property_value(source, target, CssImportance::Normal);
            let block = CssParser::default().parse_block_contents(format!("width: {source}"));
            let retained = report.syntax().is_some();
            // A substitution inside the block defers it, as the declaration front
            // does; every other block is no grammar's value.
            assert_eq!(retained, source.contains("var("), "{what} {source:?}");
            if what == "width" {
                assert_eq!(retained, block.syntax().len() == 1, "{source:?}");
            }
            if !retained {
                let (code, action, ..) = first_rejection(source, &report);
                assert_eq!(action, CssRecoveryAction::RejectInput, "{what} {source:?}");
                assert!(
                    matches!(
                        code,
                        CssErrorCode::InvalidPropertyValue | CssErrorCode::UnexpectedToken
                    ),
                    "{what} {source:?}: {code:?}"
                );
            }
        }
    }
}

#[test]
fn a_custom_property_keeps_a_block_beside_other_values() {
    let name = CssCustomPropertyName::try_new("--x").unwrap();
    let target = CssPropertyTextTarget::Property(CssPropertyNameRef::Custom(&name));
    for source in BESIDE_SUBSTITUTION.iter().chain(BESIDE_VALUE).chain(WHOLE) {
        let report =
            CssParser::default().parse_property_value(source, target, CssImportance::Normal);
        assert!(
            report.syntax().is_some(),
            "{source:?}: a custom value holds any tokens"
        );
        let block = CssParser::default().parse_block_contents(format!("--x: {source}"));
        assert_eq!(block.syntax().len(), 1, "--x: {source:?}");
    }
}

#[test]
fn values_without_a_top_level_block_beside_others_stay_retained() {
    for source in [
        "var(--x)",
        "var(--x) var(--y)",
        "f(a {} b) var(--x)",
        "var(--x, {})",
        "calc({} + var(--x))",
        "[a {} b] var(--x)",
        "1px",
    ] {
        let report = CssParser::default().parse_property_value(
            source,
            CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width)),
            CssImportance::Normal,
        );
        let block = CssParser::default().parse_block_contents(format!("width: {source}"));
        assert_eq!(
            report.syntax().is_some(),
            block.syntax().len() == 1,
            "{source:?}: the fronts agree on a block nested in another component"
        );
    }
    for source in ["var(--x)", "var(--x) var(--y)", "1px"] {
        let report = CssParser::default().parse_property_value(
            source,
            CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width)),
            CssImportance::Normal,
        );
        assert!(report.is_clean(), "{source:?}");
        assert!(report.syntax().is_some(), "{source:?}");
    }
}

fn checked_targets() -> Vec<(&'static str, CssPropertyTarget<'static>)> {
    vec![
        (
            "width",
            CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width)),
        ),
        (
            "width grammar",
            CssPropertyTarget::Grammar(CssKnownProperty::Width.grammar()),
        ),
        (
            "page margin-top",
            CssPropertyTarget::PageProperty(CssPropertyNameRef::Known(CssKnownProperty::MarginTop)),
        ),
        (
            "margin-box margin-top",
            CssPropertyTarget::MarginProperty(CssPropertyNameRef::Known(
                CssKnownProperty::MarginTop,
            )),
        ),
        (
            "glyph-orientation-vertical",
            CssPropertyTarget::Property(CssPropertyNameRef::SvgGlyphOrientationVertical),
        ),
    ]
}

/// The origin of the first top-level block of `values`.
fn first_block_origin(values: &CssComponentValues) -> CssSerializedOrigin {
    use surgeist_css::component::CssComponentValueRef;
    let block = values
        .items()
        .iter()
        .find(|item| matches!(item.view(), CssComponentValueRef::Block(_)))
        .expect("a top-level block");
    CssSerializedOrigin::Token(block.origin().clone())
}

#[test]
fn checked_construction_refuses_a_block_beside_other_values() {
    let parser = CssParser::default();
    for (what, target) in checked_targets() {
        for source in BESIDE_SUBSTITUTION.iter().chain(BESIDE_VALUE) {
            let values = CssComponentValues::parse(&parser, source).unwrap();
            let closed = !UNCLOSED.contains(source);
            let error = CssDeclaration::try_from_components(
                &parser,
                target,
                values.clone(),
                CssImportance::Normal,
            )
            .expect_err(&format!("{what} {source:?}"));
            if closed {
                assert_eq!(
                    error.code(),
                    Some(CssErrorCode::UnexpectedToken),
                    "{what} {source:?}: {error:?}"
                );
                assert_eq!(
                    error.origin(),
                    first_block_origin(&values),
                    "{what} {source:?}: located on the block"
                );
            }
        }
    }
}

#[test]
fn checked_construction_keeps_whole_value_blocks_and_custom_values() {
    let parser = CssParser::default();
    let name = CssCustomPropertyName::try_new("--x").unwrap();
    for source in ["{ var(--x) }", "{var(--x)}"] {
        let values = CssComponentValues::parse(&parser, source).unwrap();
        for (what, target) in checked_targets() {
            assert!(
                CssDeclaration::try_from_components(
                    &parser,
                    target,
                    values.clone(),
                    CssImportance::Normal
                )
                .is_ok(),
                "{what} {source:?}: a block as the entire value defers like the declaration front"
            );
        }
    }
    for source in BESIDE_SUBSTITUTION.iter().chain(BESIDE_VALUE) {
        let values = CssComponentValues::parse(&parser, source).unwrap();
        if UNCLOSED.contains(source) {
            continue;
        }
        assert!(
            CssDeclaration::try_from_components(
                &parser,
                CssPropertyTarget::Property(CssPropertyNameRef::Custom(&name)),
                values,
                CssImportance::Normal,
            )
            .is_ok(),
            "--x {source:?}"
        );
    }
}

#[test]
fn pending_reentry_refuses_a_replacement_with_a_block_beside_other_values() {
    let parser = CssParser::default();
    for (what, target) in checked_targets() {
        let pending = CssDeclaration::try_from_components(
            &parser,
            target,
            CssComponentValues::parse(&parser, "var(--x)").unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        let CssExpansion::Pending(pending) = pending.expand().unwrap() else {
            panic!("{what}: a pending authored declaration");
        };
        for source in BESIDE_VALUE {
            let replacement = CssComponentValues::parse(&parser, source).unwrap();
            let error = pending
                .reenter(replacement)
                .expect_err(&format!("{what} {source:?}"));
            let error = error.replacement_error().expect("a grammar refusal");
            assert!(error.code().is_some(), "{what} {source:?}: {error:?}");
        }
    }
}

/// The substitutions that defer a descriptor value: `env()` defers every
/// descriptor family, and `var()` defers the Page and font-palette descriptors.
const DESCRIPTOR_SUBSTITUTION: [&str; 2] = ["var(--x)", "env(--x)"];

#[track_caller]
fn assert_descriptor_refused<T>(
    what: &str,
    source: &str,
    report: &surgeist_css::CssParseReport<Option<T>>,
) {
    assert!(
        report.syntax().is_none(),
        "{what} {source:?}: a block beside other values is not retained"
    );
    let (_, action, offset, _) = first_rejection(source, report);
    assert_eq!(action, CssRecoveryAction::RejectInput, "{what} {source:?}");
    assert_eq!(
        offset,
        source.find('{').unwrap(),
        "{what} {source:?}: located on the block"
    );
}

#[test]
fn descriptor_text_fronts_refuse_a_block_beside_other_values() {
    let parser = CssParser::default();
    for substitution in DESCRIPTOR_SUBSTITUTION {
        for template in ["{} S", "S {}", "{} S {}", "{a} S", "S { a"] {
            let source = template.replace('S', substitution);
            let source = source.as_str();
            let size = CssPageDescriptorValue::parse(&parser, source, CssPageDescriptorKind::Size);
            assert_descriptor_refused("page size", source, &size);
            let bleed =
                CssPageDescriptorValue::parse(&parser, source, CssPageDescriptorKind::Bleed);
            assert_descriptor_refused("page bleed", source, &bleed);
            let palette = CssFontPaletteDescriptorValue::parse(
                &parser,
                source,
                CssFontPaletteDescriptorKind::BasePalette,
            );
            assert_descriptor_refused("palette base", source, &palette);
            if substitution == "env(--x)" {
                let counter = CssCounterStyleDescriptorValue::parse(
                    &parser,
                    source,
                    CssCounterStyleDescriptorKind::Prefix,
                );
                assert_descriptor_refused("counter prefix", source, &counter);
                let display = CssFontFeatureDisplayValue::parse(&parser, source);
                assert_descriptor_refused("feature display", source, &display);
                let feature =
                    CssFontFeatureValue::parse(&parser, source, CssFontFeatureValueKind::Swash);
                assert_descriptor_refused("feature value", source, &feature);
                let profile = CssColorProfileDescriptorValue::parse(
                    &parser,
                    source,
                    CssColorProfileDescriptorKind::Src,
                );
                assert_descriptor_refused("profile src", source, &profile);
                for (kind, what) in [
                    (CssFontFaceDescriptorKind::FontWeight, "face weight"),
                    (CssFontFaceDescriptorKind::Src, "face src"),
                ] {
                    let face = CssAuthoredFontFaceDescriptorValue::parse(&parser, source, kind);
                    assert_descriptor_refused(what, source, &face);
                }
            }
        }
    }
}

#[test]
fn descriptor_text_fronts_keep_a_deferred_value_and_a_whole_value_block() {
    let parser = CssParser::default();
    for substitution in DESCRIPTOR_SUBSTITUTION {
        let size =
            CssPageDescriptorValue::parse(&parser, substitution, CssPageDescriptorKind::Size);
        assert!(size.syntax().is_some() && size.is_clean(), "{substitution}");
        let palette = CssFontPaletteDescriptorValue::parse(
            &parser,
            substitution,
            CssFontPaletteDescriptorKind::BasePalette,
        );
        assert!(
            palette.syntax().is_some() && palette.is_clean(),
            "{substitution}"
        );
    }
    let block = "{var(--x)}";
    let size = CssPageDescriptorValue::parse(&parser, block, CssPageDescriptorKind::Size);
    assert!(size.syntax().is_some());
    let palette = CssFontPaletteDescriptorValue::parse(
        &parser,
        block,
        CssFontPaletteDescriptorKind::BasePalette,
    );
    assert!(palette.syntax().is_some());
    let counter = CssCounterStyleDescriptorValue::parse(
        &parser,
        "env(--x)",
        CssCounterStyleDescriptorKind::Prefix,
    );
    assert!(counter.syntax().is_some() && counter.is_clean());
}

#[test]
fn checked_descriptor_constructors_refuse_a_block_beside_other_values() {
    let parser = CssParser::default();
    let limits = CssComponentValueLimits::default();
    for template in [
        "{} env(--x)",
        "env(--x) {}",
        "{} {} env(--x)",
        "{a} env(--x)",
    ] {
        let values = CssComponentValues::parse(&parser, template).unwrap();
        assert!(
            CssCounterStyleDescriptorValue::try_from_components(
                CssCounterStyleDescriptorKind::Prefix,
                values.clone(),
                limits
            )
            .is_err(),
            "counter {template:?}"
        );
        assert!(
            CssFontFeatureValue::try_from_components(
                CssFontFeatureValueKind::Swash,
                values.clone(),
                limits
            )
            .is_err(),
            "feature value {template:?}"
        );
        assert!(
            CssFontFeatureDisplayValue::try_from_components(values.clone(), limits).is_err(),
            "feature display {template:?}"
        );
        assert!(
            CssAuthoredFontFaceDescriptorValue::try_from_components(
                CssFontFaceDescriptorKind::FontWeight,
                values.clone(),
                limits
            )
            .is_err(),
            "face weight {template:?}"
        );
    }
    let values = CssComponentValues::parse(&parser, "env(--x)").unwrap();
    assert!(
        CssCounterStyleDescriptorValue::try_from_components(
            CssCounterStyleDescriptorKind::Prefix,
            values,
            limits
        )
        .is_ok()
    );
}

#[test]
fn descriptor_reentry_refuses_a_replacement_with_a_block_beside_other_values() {
    let parser = CssParser::default();
    let limits = CssComponentValueLimits::default();
    let replace = |source: &str| CssComponentValues::parse(&parser, source).unwrap();
    let counter = CssCounterStyleDescriptorValue::parse(
        &parser,
        "env(--x)",
        CssCounterStyleDescriptorKind::Prefix,
    )
    .into_validation_result()
    .unwrap()
    .unwrap();
    let palette = CssFontPaletteDescriptorValue::parse(
        &parser,
        "var(--x)",
        CssFontPaletteDescriptorKind::BasePalette,
    )
    .into_validation_result()
    .unwrap()
    .unwrap();
    let profile = CssColorProfileDescriptorValue::parse(
        &parser,
        "env(--x)",
        CssColorProfileDescriptorKind::Src,
    )
    .into_validation_result()
    .unwrap()
    .unwrap();
    let feature = CssFontFeatureValue::parse(&parser, "env(--x)", CssFontFeatureValueKind::Swash)
        .into_validation_result()
        .unwrap()
        .unwrap();
    for source in BESIDE_VALUE {
        assert!(
            counter
                .reparse_after_substitution(replace(source), limits)
                .is_err(),
            "counter {source:?}"
        );
        assert!(
            palette
                .reparse_after_substitution(replace(source), limits)
                .is_err(),
            "palette {source:?}"
        );
        assert!(
            profile
                .reparse_after_substitution(replace(source), limits)
                .is_err(),
            "profile {source:?}"
        );
        assert!(
            feature
                .reparse_after_substitution(replace(source), limits)
                .is_err(),
            "feature {source:?}"
        );
    }
}

#[test]
fn declaration_units_in_rules_and_the_single_declaration_front_drop_the_value() {
    // The rule bodies and `CssDeclaration::parse` read their declarations
    // through block contents (Syntax 3 §5.5.6 step 8), so none retains the value.
    let parser = CssParser::default();
    for source in BESIDE_SUBSTITUTION.iter().chain(BESIDE_VALUE) {
        let declaration = CssDeclaration::parse(&parser, format!("width: {source}"));
        assert!(declaration.syntax().is_none(), "{source:?}");

        let style = parser.parse_sheet(format!("a {{ width: {source} }}"));
        let [CssRule::Style(rule)] = style.syntax().rules() else {
            panic!("{source:?}: one style rule");
        };
        assert!(rule.declarations().is_empty(), "{source:?}");

        let nested = parser.parse_sheet(format!("@media all {{ a {{ width: {source} }} }}"));
        assert!(!nested.is_clean(), "{source:?}");
        for sheet in [
            format!("@keyframes k {{ from {{ width: {source} }} }}"),
            format!("@page {{ margin-top: {source} }}"),
        ] {
            let report = parser.parse_sheet(&sheet);
            assert!(
                report.diagnostics().iter().any(|diagnostic| {
                    diagnostic.error().code() == CssErrorCode::UnexpectedToken
                        && matches!(
                            diagnostic.action(),
                            CssRecoveryAction::DropDeclaration
                                | CssRecoveryAction::DropQualifiedRule
                        )
                }),
                "{sheet:?}: {:?}",
                report.diagnostics()
            );
        }
    }
}
