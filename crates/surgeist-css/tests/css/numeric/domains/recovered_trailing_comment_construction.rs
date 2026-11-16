//! Checked numeric component constructors reject a comment closed by end of input.
//!
//! Syntax 3 §5.5.10 closes an unterminated comment at the end of its input, which
//! is a recovered component like an unclosed function; a checked constructor admits
//! only components their author closed, while parsing retains recovered input.

use surgeist_css::CssParser;
use surgeist_css::color::{
    CssProfileColorExpression, CssRelativeColorEnvironment, CssRelativeColorExpression,
    CssRelativeColorResultDomain,
};
use surgeist_css::component::{CssComponentValues, CssValueOrigin};
use surgeist_css::numeric::{CssCalculation, CssLength, CssNumericConstructionError};

fn components(source: &str) -> CssComponentValues {
    CssComponentValues::parse(&CssParser::default(), source).unwrap()
}

/// The rejection names the implicit closure of the comment at the end of `source`.
fn assert_recovered_at_end(error: &CssNumericConstructionError, source: &str) {
    assert!(error.is_recovered_component(), "{source}: {error:?}");
    let Some(CssValueOrigin::ImplicitClosure { at, .. }) = error.origin() else {
        panic!("{source}: implicit closure origin: {error:?}")
    };
    assert_eq!(
        at.span().start().byte_offset().value(),
        source.len(),
        "{source}"
    );
}

#[test]
fn calculation_rejects_a_trailing_comment_closed_by_end_of_input() {
    let construct = |source: &str| {
        CssCalculation::<CssLength>::try_from_components(components(source), Default::default())
    };
    for source in ["calc(1px) /* x", "calc(1px)/*"] {
        assert_recovered_at_end(&construct(source).unwrap_err(), source);
    }
    for source in ["calc(1px) /* x */", "calc(1px)/**/"] {
        let calculation = construct(source).unwrap_or_else(|error| panic!("{source}: {error:?}"));
        assert_eq!(calculation.components(), &components(source));
    }
    // A comment inside an unclosed function leaves the function recovered as well.
    let source = "calc(1px /* x";
    assert_recovered_at_end(&construct(source).unwrap_err(), source);

    // Parsing retains the value whose declaration ends in the unterminated comment.
    let report = CssParser::default().parse_block_contents("outline-offset:calc(1px) /* x");
    assert_eq!(report.syntax().len(), 1, "{:?}", report.diagnostics());
}

#[test]
fn relative_color_expression_rejects_a_trailing_comment_closed_by_end_of_input() {
    let construct = |source: &str| {
        CssRelativeColorExpression::try_from_components(
            components(source),
            CssRelativeColorEnvironment::Rgb,
            CssRelativeColorResultDomain::NumberPercentage,
            Default::default(),
        )
    };
    for source in ["none /* x", "r/*"] {
        assert_recovered_at_end(&construct(source).unwrap_err(), source);
    }
    for source in ["none /* x */", "r/**/"] {
        construct(source).unwrap_or_else(|error| panic!("{source}: {error:?}"));
    }
}

#[test]
fn profile_color_expression_rejects_a_trailing_comment_closed_by_end_of_input() {
    let construct =
        |source: &str| CssProfileColorExpression::try_from_components(components(source));
    for source in ["none /* x", "50%/*"] {
        assert_recovered_at_end(&construct(source).unwrap_err(), source);
    }
    for source in ["none /* x */", "50%/**/"] {
        construct(source).unwrap_or_else(|error| panic!("{source}: {error:?}"));
    }
}
