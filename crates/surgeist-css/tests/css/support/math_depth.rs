//! The nesting-depth boundaries shared by math-function tests.
//!
//! [`MathDepthBoundary`] places `depth` nested `calc(` openings, and the same
//! number of closing parentheses, into the two `{}` slots of a style-attribute
//! template. Every accepted depth parses cleanly, retains the slot
//! declaration and its sibling, and passes strict validation with the same
//! declarations. Every rejected depth refuses the whole contents, sibling
//! included (Syntax 3 (2026 CRD) section 5.5.5 charges the contents once against
//! the 256-level component ceiling): one `InvalidComponentValue` diagnostic
//! with a nesting-limit cause that spans the contents and stops at the first
//! opening past the deepest accepted depth, and strict validation fails with
//! the same diagnostics. The rejected depths begin immediately after the
//! accepted ones, so each row states an exact boundary.
//!
//! [`ComposedDepthBoundary`] states the same exact boundary for a typed value
//! that a test builds from a nested calculation, where the front under test is
//! a checked constructor instead of the style attribute.

use surgeist_css::component::CssComponentValueError;
use surgeist_css::declaration::CssDeclarationList;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::{CssParseReport, CssParser};

/// The nested opening that each unit of depth adds.
const OPENING: &str = "calc(";

/// One exact math-function nesting-depth boundary.
#[derive(Clone, Copy, Debug)]
pub struct MathDepthBoundary<'a> {
    /// What the row covers; every failure names it.
    pub name: &'a str,
    /// A style attribute whose first `{}` receives the openings and whose
    /// second `{}` receives the closing parentheses.
    pub template: &'a str,
    /// Depths that parse cleanly, in increasing order.
    pub accepted: &'a [usize],
    /// Depths that refuse the whole contents at the nesting limit, in increasing order.
    pub rejected: &'a [usize],
    /// Whether the template has a valid sibling declaration that every accepted
    /// depth retains.
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
            "{}: the rejected depths begin at the exact boundary",
            self.name
        );
        let retained = usize::from(self.sibling_retained);
        for &depth in self.accepted {
            let context = format!("{}: depth {depth}", self.name);
            let source = self.source(depth);
            let report = CssParser::default().parse_block_contents(&source);
            assert!(report.is_clean(), "{context}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().len(), retained + 1, "{context}");
            assert_eq!(
                CssParser::default()
                    .parse_block_contents(&source)
                    .into_validation_result()
                    .expect("strict validation accepts a supported depth"),
                *report.syntax(),
                "{context}"
            );
            accepted(depth, &source, &report);
        }
        let (before, _, _) = self.slots();
        // The first opening past the deepest accepted depth.
        let first_over_limit = before.len() + deepest * OPENING.len();
        for &depth in self.rejected {
            let context = format!("{}: depth {depth}", self.name);
            let source = self.source(depth);
            assert_eq!(
                source.match_indices(OPENING).nth(deepest).map(|(at, _)| at),
                Some(first_over_limit),
                "{context}: the template holds no opening before its slot"
            );
            // Block contents charges every component once against the 256-level
            // ceiling, so an over-limit depth refuses the whole contents.
            let report = CssParser::default().parse_block_contents(&source);
            assert!(report.syntax().is_empty(), "{context}");
            crate::css::support::parse_report::assert_contents_refused(&report, &source, &context);
            let [diagnostic] = report.diagnostics() else {
                unreachable!("assert_contents_refused admits one diagnostic");
            };
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::InvalidComponentValue,
                "{context}"
            );
            assert_eq!(
                diagnostic.action(),
                CssRecoveryAction::DropQualifiedRule,
                "{context}"
            );
            assert!(
                diagnostic
                    .error()
                    .component_error()
                    .is_some_and(CssComponentValueError::is_nesting_limit),
                "{context}: the typed nesting-limit cause"
            );
            // A template is one ASCII line, so a byte offset is also a column.
            assert!(
                source.is_ascii() && !source.contains('\n'),
                "{context}: an ASCII single-line template"
            );
            for (position, offset, what) in [
                (
                    diagnostic.error().position(),
                    first_over_limit,
                    "the first opening past the deepest accepted depth",
                ),
                (diagnostic.span().start(), 0, "the span start"),
                (diagnostic.span().end(), source.len(), "the span end"),
            ] {
                assert_eq!(position.byte_offset().value(), offset, "{context}: {what}");
                assert_eq!(position.line().value(), 0, "{context}: {what}");
                assert_eq!(
                    position.column().value(),
                    offset as u32,
                    "{context}: {what}"
                );
            }
            let failure = CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .expect_err("strict validation rejects an over-limit depth");
            assert_eq!(failure.diagnostics(), report.diagnostics(), "{context}");
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

/// What building a typed value from a nested calculation produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Construction {
    /// The value was built.
    Built,
    /// The value was refused with the nesting-limit error kind. Any other
    /// refusal fails inside the row's builder instead of becoming this one.
    NestingLimit,
}

impl Construction {
    /// `Built` for `Ok`, and `NestingLimit` for an error that
    /// `is_nesting_limit` accepts; any other error panics with its debug form.
    #[track_caller]
    pub fn of<T, E: std::fmt::Debug>(
        result: Result<T, E>,
        is_nesting_limit: impl FnOnce(&E) -> bool,
    ) -> Self {
        match result {
            Ok(_) => Self::Built,
            Err(error) if is_nesting_limit(&error) => Self::NestingLimit,
            Err(error) => panic!("an error other than the nesting limit: {error:?}"),
        }
    }
}

/// One exact nesting-depth boundary of a typed value that a checked
/// constructor composes from a nested calculation.
#[derive(Clone, Copy)]
pub struct ComposedDepthBoundary<'a> {
    /// What the row covers; every failure names it.
    pub name: &'a str,
    /// The calculation's innermost operand.
    pub leaf: &'a str,
    /// Depths that build the value, in increasing order.
    pub accepted: &'a [usize],
    /// Depths that the nesting limit refuses, in increasing order.
    pub rejected: &'a [usize],
    /// Builds the value from the calculation source that nests `leaf` in
    /// the given number of `calc(` openings, and checks what the row asserts
    /// of the built value.
    pub build: fn(usize, &str) -> Construction,
}

impl ComposedDepthBoundary<'_> {
    /// The calculation source with `depth` nested openings.
    pub fn source(&self, depth: usize) -> String {
        format!(
            "{}{}{}",
            OPENING.repeat(depth),
            self.leaf,
            ")".repeat(depth)
        )
    }

    /// Checks every accepted and rejected depth.
    #[track_caller]
    pub fn assert(&self) {
        let deepest = *self.accepted.last().expect("an accepted depth");
        assert_eq!(
            self.rejected.first(),
            Some(&(deepest + 1)),
            "{}: the rejected depths begin at the exact boundary",
            self.name
        );
        for (depths, expected) in [
            (self.accepted, Construction::Built),
            (self.rejected, Construction::NestingLimit),
        ] {
            for &depth in depths {
                assert_eq!(
                    (self.build)(depth, &self.source(depth)),
                    expected,
                    "{}: depth {depth}",
                    self.name
                );
            }
        }
    }
}
