//! UTF-8 style sheet and style-attribute parsing over exact source text.
//!
//! The canonical case is the exact UTF-8 source. Malformed UTF-8 is a codec
//! rejection, never a lossy conversion; byte decoding has its own target. Every
//! case runs both public text fronts (`parse_sheet` and `parse_block_contents`)
//! in each parser mode and checks the contracts the crate documents for any
//! input: coordinates, clean reports, retained nodes, determinism and the
//! equality of the UTF-16 front where preprocessing is the identity.
use crate::{
    check::{self, Check, Checker, Violation},
    cursor::Cursor,
    descriptor, error, evaluated, nodes, violation,
};
use std::{convert::Infallible, sync::Arc};
use surgeist_css::{
    CssParseReport, CssParser, CssParserContext, CssParserMode,
    component::CssComponentValueLimits,
    diagnostic::CssRecoveryDiagnostic,
    source::{CssUtf16InputLimits, same_provenance},
};
use surgeist_fuzz::{
    CaseOutcome, CaseTarget, Generation, Result, TargetDescriptor,
    generation::{Budget, BudgetCharge},
};

#[cfg(test)]
mod tests;

const MAX_BYTES: usize = 16 * 1024;
const DETERMINISM: &str = "surgeist-css::parse-determinism";
const PARITY: &str = "surgeist-css::utf16-parity";

/// Rule preludes: ordinary, escaped, Unicode, nested and unknown at-rule heads.
const PRELUDES: [&str; 26] = [
    ".a",
    "#i",
    "div",
    "a > b",
    ":is(.x, .y)",
    "[data-k=\"v\"]",
    "\\61 b",
    "\u{e9}t\u{e9}",
    "@media screen",
    "@media (min-width: 10px)",
    "@supports (display: grid)",
    "@layer base",
    "@container (min-width: 1px)",
    "@unknown x",
    "@font-face",
    "@keyframes k",
    "@page",
    "@import url(x.css)",
    "@namespace svg url(x)",
    "@charset \"utf-8\"",
    "&",
    "&.n",
    "--x",
    "\u{1f600}",
    ".a, , .b",
    "@media",
];
/// Declarations: valid, custom, deferred substitution, numeric extremes and invalid.
const DECLARATIONS: [&str; 34] = [
    "color: red",
    "width: 10px",
    "margin: 1px 2px 3px 4px",
    "--x: 1px",
    "--y:",
    "--z: { a: b }",
    "--w: var(--x, 2px)",
    "width: calc(100% - 1px)",
    "width: var(--q)",
    "color: env(x)",
    "content: \"a\\\"b\"",
    "background: url(a b)",
    "opacity: 1e999",
    "z-index: 99999999999999999999",
    "width: 0.0000000001px",
    "line-height: -0",
    "font-family: \"A\", b, \\61",
    "unknown-prop: 1",
    "width: ;",
    "color: !important",
    "width: 1px !important",
    "\\77 idth: 2px",
    "COLOR: RED",
    "margin: 1px 2px 3px 4px 5px",
    "transition: all 1s, opacity 2s",
    "grid-template-areas: \"a b\" \"c d\"",
    "content: \"\\",
    "width: 1px 2px",
    ":",
    "color red",
    "--\u{e9}: \u{1f600}",
    "\\--e: 1",
    "background-size: 10px",
    "width: calc(1px + (2px * (3 - 1",
];
/// Fragments that break, escape, nest or truncate whatever surrounds them.
const JUNK: [&str; 46] = [
    "{",
    "}",
    "(",
    ")",
    "[",
    "]",
    ";",
    ",",
    ":",
    "!",
    "!important",
    "/* c */",
    "/*",
    "\"",
    "'",
    "\"abc",
    "url(",
    "url(a b)",
    "\\",
    "\\\n",
    "<!--",
    "-->",
    "\u{feff}",
    "\u{2028}",
    "\0",
    "\r",
    "\r\n",
    "\u{c}",
    "\n",
    " ",
    "\t",
    "@",
    "#",
    "&",
    "1e308",
    "-1e-308",
    "0",
    "+.5e+3",
    "1px",
    "50%",
    "U+0-7F",
    "\u{1f600}",
    "\u{10ffff}",
    "e\u{301}",
    "\u{a0}",
    "\u{fffe}",
];

/// A checked, bounded concrete case: the exact UTF-8 source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceCase {
    text: String,
}

impl SourceCase {
    /// Builds a case; the source length is bounded.
    pub fn new(text: impl Into<String>) -> Result<Self> {
        let text = text.into();
        if text.len() > MAX_BYTES {
            return Err(error("source exceeds the byte ceiling"));
        }
        Ok(Self { text })
    }

    /// The exact source text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// The target: one evaluator shared by workers, properties, coverage and replay.
pub struct SourceTarget {
    descriptor: TargetDescriptor,
    checker: Checker,
}

impl Default for SourceTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceTarget {
    /// The number of source-qualified literal cases.
    pub const LITERALS: u8 = 24;

    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptor: descriptor("surgeist-css::utf8-source"),
            checker: check::DOCUMENTED,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_checker(checker: Checker) -> Self {
        Self {
            checker,
            ..Self::new()
        }
    }

    /// Source-qualified literal cases; each exercises one documented contract.
    pub fn literal(&self, id: u8) -> Result<SourceCase> {
        LITERALS.get(usize::from(id)).map_or_else(
            || Err(error("unknown literal")),
            |text| SourceCase::new(*text),
        )
    }
}

/// Literals chosen from the crate reference, one contract each. None carries an
/// expected output; the evaluator checks contracts that hold for every input.
const LITERALS: [&str; 24] = [
    "",
    ".a { color: red } @unknown x; .b {}",
    "color: red; width: 2px !important",
    "\u{feff}a {}",
    "\u{feff}",
    "a {} color: red; width: 2px;",
    "color: red } width: 1px",
    "/* unterminated",
    "a { b: c } /* x",
    "a { content: \"x\\\n\" }",
    "a { b: \\",
    "a { background: url(a \\)still) } .after { color: red }",
    "a { --x: { b: c; d: e } ; width: 1px }",
    "a {\r\n  color: red;\r  width: 2px;\u{c}  height: 3px\n}\n.b { color: blue }",
    "\u{1f600} { --\u{e9}: \u{10ffff}; color: red }",
    "a { width: calc(1px + 2px * (3 - 1)); margin: var(--m, 1px 2px) }",
    "a { width: 1e999px; z-index: 99999999999999999999; opacity: -0 }",
    "a { b: c; & d { e: f } g: h; .i { j: k } }",
    "@media screen { a { color: red } @supports (display: grid) { b { c: d } } }",
    "@layer a, b; @layer c { d { e: f } } @unknown { { } } .z { y: x }",
    "<!-- a { b: c } --> d { e: f }",
    "@import url(a.css) screen; @namespace svg url(x); a { b: c }",
    "a { b: rgb(1 2 3 } .after { color: red }",
    "@font-face { font-family: A; src: url(a) } @keyframes k { from { a: b } to { a: c } } @page { margin: 1px }",
];

fn pair(entropy: &mut Cursor<'_>, limit: usize) -> String {
    let mut text = String::new();
    while text.len() < limit {
        match entropy.below(12) {
            0..=2 => {
                text.push_str(PRELUDES[entropy.below(PRELUDES.len())]);
                text.push_str(" { ");
                for _ in 0..entropy.below(6) {
                    declaration_or_junk(entropy, &mut text);
                }
                if entropy.below(8) != 0 {
                    text.push_str(" } ");
                }
            }
            3..=5 => declaration_or_junk(entropy, &mut text),
            6 => text.push_str(JUNK[entropy.below(JUNK.len())]),
            7 => text.push_str(PRELUDES[entropy.below(PRELUDES.len())]),
            _ => {
                text.push_str(DECLARATIONS[entropy.below(DECLARATIONS.len())]);
                text.push(';');
            }
        }
        if entropy.below(40) == 0 {
            break;
        }
    }
    text
}

fn declaration_or_junk(entropy: &mut Cursor<'_>, text: &mut String) {
    text.push_str(DECLARATIONS[entropy.below(DECLARATIONS.len())]);
    text.push_str(if entropy.below(5) == 0 { " " } else { "; " });
    if entropy.below(6) == 0 {
        text.push_str(JUNK[entropy.below(JUNK.len())]);
    }
}

/// Entropy to source text. The first byte chooses a lane: assembled rules,
/// declaration lists, fragment soup, deep nesting, wide Unicode, or bytes mapped
/// to U+0000..U+00FF. These are generation choices, not a decoder for malformed
/// input.
fn generate(descriptor: &TargetDescriptor, input: &[u8]) -> Result<SourceCase> {
    let mut entropy = Cursor::new(input);
    let limit = 1024 + entropy.below(8) * 1536;
    let mut text = match entropy.below(8) {
        0..=2 => pair(&mut entropy, limit),
        3 => {
            let mut text = String::new();
            for _ in 0..entropy.below(40) {
                declaration_or_junk(&mut entropy, &mut text);
            }
            text
        }
        4 => {
            let mut text = String::new();
            for _ in 0..entropy.below(80) {
                text.push_str(JUNK[entropy.below(JUNK.len())]);
            }
            text
        }
        5 => {
            // Nesting around, below and above the 63, 128 and 256 level seams.
            let depth = [1_usize, 62, 63, 64, 127, 128, 129, 255, 256, 257, 300][entropy.below(11)];
            let opener = ["{", "(", "[", "a{", "a(", "b:c("][entropy.below(6)];
            let mut text = opener.repeat(depth);
            if entropy.flag() {
                text.push('x');
                let closer = match opener.chars().last() {
                    Some('{') => "}",
                    Some('(') => ")",
                    _ => "]",
                };
                text.push_str(&closer.repeat(depth));
            }
            text
        }
        6 => {
            let mut text = String::new();
            for _ in 0..entropy.below(200) {
                let scalar = u32::from(entropy.byte()) << 8 | u32::from(entropy.byte());
                text.push(char::from_u32(scalar).unwrap_or('\u{fffd}'));
            }
            text
        }
        _ => {
            let mut text = String::new();
            for _ in 0..entropy.below(220) {
                text.push(char::from(entropy.byte()));
            }
            text
        }
    };
    let mut keep = text.len().min(MAX_BYTES);
    while !text.is_char_boundary(keep) {
        keep -= 1;
    }
    text.truncate(keep);
    let mut budget = Budget::new(descriptor.limits());
    budget.credit(BudgetCharge {
        bytes: text.len() as u64,
        text_units: text.encode_utf16().count() as u64,
        ..BudgetCharge::default()
    })?;
    SourceCase::new(text)
}

/// `(offset, line, column)` of a position, the form both ingress paths share.
type Coordinate = (usize, u32, u32);

fn coordinate(position: surgeist_css::source::CssSourcePosition) -> Coordinate {
    (
        position.byte_offset().value(),
        position.line().value(),
        position.column().value(),
    )
}

fn fingerprint(diagnostics: &[CssRecoveryDiagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{:?} {:?} {:?} {:?} {:?}",
                diagnostic.error().code(),
                diagnostic.action(),
                coordinate(diagnostic.error().position()),
                coordinate(diagnostic.span().start()),
                coordinate(diagnostic.span().end())
            )
        })
        .collect()
}

fn determinism<T: Clone + PartialEq>(
    first: &CssParseReport<T>,
    second: &CssParseReport<T>,
) -> Check {
    if first != second || !same_provenance(first, second) {
        return Err(Violation::new(
            DETERMINISM,
            "reparse-differs",
            "two parses of the same source differ in value or provenance",
        ));
    }
    Ok(())
}

struct Ran {
    parity: bool,
}

fn observe(checker: &Checker, text: &str) -> std::result::Result<Ran, Violation> {
    let mut parity = false;
    for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
        let parser = CssParser::new(
            CssParserContext::new(mode),
            CssComponentValueLimits::default(),
        );
        let sheet = parser.parse_sheet(text);
        checker.diagnostics(text, sheet.diagnostics())?;
        check::cleanliness(&sheet)?;
        nodes::sheet(checker, text, sheet.syntax())?;
        determinism(&sheet, &parser.parse_sheet(text))?;

        let block = parser.parse_block_contents(text);
        checker.diagnostics(text, block.diagnostics())?;
        check::cleanliness(&block)?;
        nodes::declarations(checker, text, block.syntax().as_slice(), "block contents")?;
        determinism(&block, &parser.parse_block_contents(text))?;

        // UTF-16 preprocessing is the identity unless CR, FF or NUL occur, so the
        // UTF-16 fronts must agree with the UTF-8 fronts exactly, coordinates included.
        if !text.bytes().any(|byte| matches!(byte, b'\r' | 0x0c | 0)) {
            let units: Vec<u16> = text.encode_utf16().collect();
            let limits = CssUtf16InputLimits::default();
            let poll = || Ok::<(), Infallible>(());
            if let Ok(wide) = parser.parse_sheet_utf16(Arc::new(units.clone()), limits, poll) {
                if wide.syntax() != sheet.syntax()
                    || fingerprint(wide.diagnostics()) != fingerprint(sheet.diagnostics())
                {
                    return Err(Violation::new(
                        PARITY,
                        "sheet-utf16-differs",
                        "the UTF-16 sheet front differs from the UTF-8 front",
                    ));
                }
                parity = true;
            }
            if let Ok(wide) = parser.parse_block_contents_utf16(Arc::new(units), limits, poll) {
                if wide.report().syntax() != block.syntax()
                    || fingerprint(wide.report().diagnostics()) != fingerprint(block.diagnostics())
                {
                    return Err(Violation::new(
                        PARITY,
                        "block-utf16-differs",
                        "the UTF-16 block-contents front differs from the UTF-8 front",
                    ));
                }
                parity = true;
            }
        }
    }
    Ok(Ran { parity })
}

impl CaseTarget for SourceTarget {
    type Case = SourceCase;

    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }

    fn decode(&self, input: &[u8]) -> Result<SourceCase> {
        if input.len() > MAX_BYTES {
            return Err(error("source exceeds the byte ceiling"));
        }
        let text = std::str::from_utf8(input).map_err(|_| error("source is not valid UTF-8"))?;
        SourceCase::new(text)
    }

    fn encode(&self, case: &SourceCase) -> Result<Vec<u8>> {
        Ok(case.text.as_bytes().to_vec())
    }

    fn generate(&self, input: &[u8]) -> Result<Generation<SourceCase>> {
        generate(&self.descriptor, input).map(Generation::Generated)
    }

    fn evaluate(&self, case: &SourceCase) -> Result<CaseOutcome> {
        match observe(&self.checker, &case.text) {
            Err(found) => violation(&found),
            Ok(ran) => {
                let mut checked = vec![
                    check::COORDINATES,
                    check::CLEANLINESS,
                    nodes::RETAINED,
                    DETERMINISM,
                ];
                if ran.parity {
                    checked.push(PARITY);
                }
                evaluated(&checked)
            }
        }
    }
}
