#![forbid(unsafe_code)]
//! Checked step counts distinguish bare integer roots from genuine math functions.
//! Ordinary counts are positive, and jump-none requires more than one step.
//! Genuine math remains symbolic; these assertions never evaluate a function.
//! https://www.w3.org/TR/2023/CRD-css-easing-1-20230213/#step-easing-functions

use surgeist_css::{
    CssIntegerCalculation, CssStepCount, CssStepPosition, CssSteps, parse_component_values,
};

fn admitted(root: &str, position: Option<CssStepPosition>) -> bool {
    let calculation =
        CssIntegerCalculation::try_from_components(parse_component_values(root).unwrap())
            .expect("integer-domain calculation root");
    CssSteps::try_new(CssStepCount::from_calculation(calculation), position).is_some()
}

#[test]
fn bare_zero_calculation_roots_cannot_bypass_positive_step_admission() {
    for root in ["0", "+0000", "-0000"] {
        assert!(!admitted(root, None), "{root}");
    }
}

#[test]
fn bare_negative_calculation_roots_cannot_bypass_positive_step_admission() {
    for root in ["-1", "-2147483649"] {
        assert!(!admitted(root, None), "{root}");
    }
}

#[test]
fn bare_one_calculation_roots_cannot_bypass_jump_none_admission() {
    for root in ["1", "+0001"] {
        assert!(!admitted(root, Some(CssStepPosition::JumpNone)), "{root}");
    }
}

#[test]
fn bare_positive_calculation_roots_keep_ordinary_position_rules() {
    for root in ["1", "+0001", "2", "2147483648"] {
        assert!(admitted(root, None), "{root}");
        assert!(admitted(root, Some(CssStepPosition::End)), "{root}");
    }
    for root in ["2", "2147483648"] {
        assert!(admitted(root, Some(CssStepPosition::JumpNone)), "{root}");
    }
}

#[test]
fn genuine_integer_math_functions_keep_range_checks_deferred() {
    for root in ["calc(0)", "calc(-1)", "calc(1)", "calc(1 + 1)"] {
        assert!(admitted(root, None), "{root}");
        assert!(admitted(root, Some(CssStepPosition::JumpNone)), "{root}");
    }
}
