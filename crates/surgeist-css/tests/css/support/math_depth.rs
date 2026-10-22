//! The style-attribute nesting-depth boundary shared by math-function tests.
//!
//! A row places `depth` nested `calc(` openings, and the same number of
//! closing parentheses, into the two `{}` slots of a style-attribute
//! template. Every accepted depth parses cleanly, retains the slot
//! declaration and its sibling, and passes strict validation with the same
//! declarations. Every rejected depth drops only the slot declaration, with
//! one `NestingLimit` diagnostic that stops at the first opening past the
//! deepest accepted depth, and strict validation fails with the same
//! diagnostics. The rejected depths begin immediately after the accepted ones,
//! so each row states an exact boundary.

use surgeist_css::*;

/// The nested opening that each unit of depth adds.
const OPENING: &str = "calc(";

/// One exact math-function nesting-depth boundary.
#[derive(Clone, Copy, Debug)]
pub struct MathDepthBoundary<'a> {
    /// A style attribute whose first `{}` receives the openings and whose
    /// second `{}` receives the closing parentheses.
    pub template: &'a str,
    /// Depths that parse cleanly, in increasing order.
    pub accepted: &'a [usize],
    /// Depths that stop at the nesting limit, in increasing order.
    pub rejected: &'a [usize],
    /// Whether the template has a valid sibling declaration that every depth
    /// retains.
    pub sibling_retained: bool,
}

impl MathDepthBoundary<'_> {
    /// The style attribute with `depth` nested openings.
    pub fn source(&self, depth: usize) -> String {
        let (before, operand, after) = self.slots();
        format!(
            "{before}{}{operand}{}{after}",
            OPENING.repeat(depth),
            ")".repeat(depth)
        )
    }

    /// Checks every accepted and rejected depth.
    pub fn assert(&self) {
        self.assert_with(|_, _, _| {}, |_, _, _| {});
    }

    /// Checks every depth and passes each depth, source and report to the
    /// matching closure for the caller's further checks.
    pub fn assert_with(
        &self,
        mut accepted: impl FnMut(usize, &str, &CssParseReport<CssDeclarationList>),
        mut rejected: impl FnMut(usize, &str, &CssParseReport<CssDeclarationList>),
    ) {
        let deepest = *self.accepted.last().expect("an accepted depth");
        assert_eq!(
            self.rejected.first(),
            Some(&(deepest + 1)),
            "the rejected depths begin at the exact boundary"
        );
        let retained = usize::from(self.sibling_retained);
        for &depth in self.accepted {
            let source = self.source(depth);
            let report = parse_style_attribute(&source);
            assert!(
                report.is_clean(),
                "depth {depth}: {:?}",
                report.diagnostics()
            );
            assert_eq!(report.syntax().len(), retained + 1, "depth {depth}");
            assert_eq!(
                validate_style_attribute(&source)
                    .expect("strict validation accepts a supported depth"),
                *report.syntax(),
                "depth {depth}"
            );
            accepted(depth, &source, &report);
        }
        let (before, _, _) = self.slots();
        // The first opening past the deepest accepted depth.
        let first_over_limit = before.len() + deepest * OPENING.len();
        for &depth in self.rejected {
            let source = self.source(depth);
            assert_eq!(
                source.match_indices(OPENING).nth(deepest).map(|(at, _)| at),
                Some(first_over_limit),
                "depth {depth}: the template holds no opening before its slot"
            );
            let report = parse_style_attribute(&source);
            assert_eq!(report.syntax().len(), retained, "depth {depth}");
            let [diagnostic] = report.diagnostics() else {
                panic!("depth {depth}: one diagnostic: {:?}", report.diagnostics());
            };
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::NestingLimit,
                "depth {depth}"
            );
            assert_eq!(
                diagnostic.action(),
                CssRecoveryAction::StopAtNestingLimit,
                "depth {depth}"
            );
            assert_eq!(
                diagnostic.error().position().byte_offset().value(),
                first_over_limit,
                "depth {depth}"
            );
            let failure = validate_style_attribute(&source)
                .expect_err("strict validation rejects an over-limit depth");
            assert_eq!(failure.diagnostics(), report.diagnostics(), "depth {depth}");
            rejected(depth, &source, &report);
        }
    }

    /// The template text before the openings, between the two slots, and
    /// after the closing parentheses.
    fn slots(&self) -> (&str, &str, &str) {
        let (before, rest) = self.template.split_once("{}").expect("an opening slot");
        let (operand, after) = rest.split_once("{}").expect("a closing slot");
        assert!(
            !after.contains("{}"),
            "exactly two slots: {}",
            self.template
        );
        (before, operand, after)
    }
}
