#![forbid(unsafe_code)]

use surgeist_css::{CssExpansion, expand_declaration, parse_style_attribute};

// Intrinsic expansion retains authored custom properties, before cascade and
// variable substitution. Variables 1 distinguishes their token values from
// whole-value CSS-wide keywords, and permits important custom declarations:
// https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#defining-variables
// https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#syntax
#[test]
fn custom_token_values_contribute_without_contextual_substitution() {
    for value in [
        "",
        "Red",
        "var(--Theme)",
        "var(--missing, 1px)",
        "[a] / f(2)",
    ] {
        for importance in ["", "!important"] {
            let source = format!("--Theme:{value}{importance}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().len(), 1);
            let result = expand_declaration(&report.syntax()[0]);
            assert!(
                matches!(result, Ok(CssExpansion::Contributions(_))),
                "custom tokens remain symbolic contributions: {source}: {result:?}"
            );
        }
    }
}

#[test]
fn custom_css_wide_values_contribute_before_cascade() {
    for keyword in ["initial", "inherit", "unset", "revert", "revert-layer"] {
        let source = format!("--Theme:{keyword}!important");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let result = expand_declaration(&report.syntax()[0]);
        assert!(
            matches!(result, Ok(CssExpansion::Contributions(_))),
            "{result:?}"
        );
    }
}

#[test]
fn known_substitution_dependent_shorthands_remain_pending() {
    let report = parse_style_attribute("margin:var(--Theme)");
    assert!(report.is_clean());
    assert!(matches!(
        expand_declaration(&report.syntax()[0]),
        Ok(CssExpansion::Pending(_))
    ));
}
