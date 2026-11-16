//! Typed authored property values: metadata sweep, specified round trips,
//! independent known values, sibling recovery and implicit closures.
//!
//! Specified serialization is deliberately canonicalizing (numbers round to six
//! places, an opacity percentage becomes a number, a four-value margin takes its
//! shortest form), so the round trip is checked under an explicit normalization:
//! one pass may change spelling or kind, and the normalized text must then
//! reparse cleanly to the same property and importance and be a fixed point.
//! Only rows whose serialization changes spelling alone also demand structural
//! equality with the original parse. Expected texts come from the documented
//! serialization rules and property metadata, never from a serializer run.
use crate::{
    check::{self, Checker, Violation},
    codec::{Reader, Writer},
    cursor::Cursor,
    descriptor, error, evaluated, nodes, violation,
};
use surgeist_css::{
    CssParser,
    component::CssComponentValues,
    declaration::{CssCustomPropertyName, CssDeclaration, CssGlobalKeyword, CssImportance},
    diagnostic::{CssErrorCode, CssRecoveryAction},
    property::{CssKnownProperty, CssPropertyNameRef, CssPropertyTarget, CssPropertyTextTarget},
    serialization::CssSerializationLimits,
};
use surgeist_fuzz::{
    CaseOutcome, CaseTarget, Generation, Result, TargetDescriptor,
    generation::{Budget, BudgetCharge},
};

#[cfg(test)]
mod tests;

pub(crate) const MAGIC: [u8; 4] = *b"CSPV";
const MAX_CASE: usize = 4096;
const MAX_PIECES: usize = 24;
const MAX_DIGITS: u8 = 40;
const MAX_EXPONENT: u16 = 999;

const GLOBAL: &str = "surgeist-css::global-and-substitution";
const METADATA: &str = "surgeist-css::property-metadata";
const ROUND_TRIP: &str = "surgeist-css::specified-round-trip";
const KNOWN: &str = "surgeist-css::known-value-serialization";
const SIBLINGS: &str = "surgeist-css::sibling-recovery";
const CLOSURE: &str = "surgeist-css::implicit-closure";
const AGREEMENT: &str = "surgeist-css::front-agreement";

/// Forms valid for every property regardless of its grammar: CSS-wide keywords
/// and values that await substitution.
const FORMS: [&str; 8] = [
    "inherit",
    "initial",
    "unset",
    "revert",
    "revert-layer",
    "var(--x)",
    "var(--x, 1px)",
    "env(safe-area-inset-top)",
];

/// Properties whose grammars differ widely, plus a custom property.
const CURATED: [&str; 36] = [
    "width",
    "height",
    "margin",
    "margin-block",
    "padding",
    "color",
    "background-color",
    "background",
    "background-size",
    "border",
    "border-radius",
    "opacity",
    "z-index",
    "line-height",
    "font-family",
    "font-size",
    "font-weight",
    "transform",
    "transition",
    "animation",
    "content",
    "flex",
    "grid-template-columns",
    "display",
    "overflow",
    "text-shadow",
    "box-shadow",
    "filter",
    "clip-path",
    "mask-size",
    "aspect-ratio",
    "gap",
    "cursor",
    "text-decoration",
    "--x",
    "--\u{e9}",
];

/// Value fragments: keywords, functions, strings, escapes, Unicode, comments,
/// and truncated or stray punctuation. Most combinations are invalid on purpose.
const WORDS: [&str; 75] = [
    "auto",
    "none",
    "inherit",
    "red",
    "transparent",
    "currentcolor",
    "solid",
    "block",
    "bold",
    "normal",
    "cover",
    "all",
    "e",
    "calc(",
    "min(",
    "clamp(",
    "var(--x",
    "var(--x,",
    "env(x",
    "attr(x",
    "rgb(",
    "hsl(",
    "url(",
    "url(a",
    "linear-gradient(",
    "translate(",
    "repeat(",
    "minmax(",
    "#fff",
    "#FFFFFF",
    "#12",
    "\"s\"",
    "'s'",
    "\"a\\\"b\"",
    "\"unterminated",
    "\\\n",
    "\\",
    "\\31 ",
    "\\000041",
    "\u{e9}",
    "\u{1f600}",
    "\u{feff}",
    "\0",
    "/*c*/",
    "/*",
    "*",
    "/",
    "+",
    "-",
    "(",
    ")",
    ",",
    "[",
    "]",
    "{",
    "}",
    ";",
    "!",
    "!important",
    "1/2",
    "U+0-7F",
    "--x",
    "--",
    ".5",
    "1e",
    "1e+",
    "0",
    "100%",
    "10px",
    "2s",
    "45deg",
    "1fr",
    "\t",
    "\n",
    "\r\n",
];
const UNITS: [&str; 11] = ["", "px", "%", "em", "s", "ms", "deg", "fr", "x", "Q", "vw"];
const GAPS: [&str; 6] = ["", " ", "/**/", ", ", " / ", "\n"];

/// Independent known values: (property, authored value, documented specified
/// text of the first serialization, documented specified text of the
/// serialization of that text reparsed, and whether the first serialization
/// changes spelling alone).
///
/// The texts follow the crate reference: exact units, shortest CSSOM shorthand
/// form, `0.25` for `25%`, six fractional places, and two-value
/// `background-size`. Omission of a default is selected on exact values before
/// the lossy six-place number text (CSSOM section 6.7.2, as the header of
/// `tests/css/numeric/projected_number_formatting_lifecycle.rs` records), so a
/// number that does not survive six-place text can serialize first with its
/// rounded value and then, once reparsed as that exact value, without it. A row
/// whose second text differs from its first is that documented two-step case;
/// every other row is a fixed point at the first serialization.
pub(crate) type Row = (&'static str, &'static str, &'static str, &'static str, bool);
const ROWS: [Row; 22] = [
    (
        "margin-block",
        "1px 2px",
        "margin-block: 1px 2px;",
        "margin-block: 1px 2px;",
        true,
    ),
    (
        "margin",
        "1px 2px 3px 4px",
        "margin: 1px 2px 3px 4px;",
        "margin: 1px 2px 3px 4px;",
        true,
    ),
    (
        "margin",
        "1px 1px 1px 1px",
        "margin: 1px;",
        "margin: 1px;",
        false,
    ),
    (
        "margin",
        "1px 2px 1px 2px",
        "margin: 1px 2px;",
        "margin: 1px 2px;",
        false,
    ),
    (
        "width",
        "calc(100% - 12px)",
        "width: calc(100% - 12px);",
        "width: calc(100% - 12px);",
        true,
    ),
    ("width", "1.50px", "width: 1.5px;", "width: 1.5px;", true),
    ("width", "+.5px", "width: 0.5px;", "width: 0.5px;", true),
    ("width", "0.0000001px", "width: 0px;", "width: 0px;", false),
    ("height", "10px", "height: 10px;", "height: 10px;", true),
    ("color", "red", "color: red;", "color: red;", true),
    ("opacity", "0.5", "opacity: 0.5;", "opacity: 0.5;", true),
    ("opacity", ".1", "opacity: 0.1;", "opacity: 0.1;", true),
    ("opacity", "25%", "opacity: 0.25;", "opacity: 0.25;", false),
    ("z-index", "3", "z-index: 3;", "z-index: 3;", true),
    (
        "display",
        "block",
        "display: block;",
        "display: block;",
        true,
    ),
    (
        "background-size",
        "10px",
        "background-size: 10px auto;",
        "background-size: 10px auto;",
        false,
    ),
    (
        "mask-size",
        "cover",
        "mask-size: cover;",
        "mask-size: cover;",
        true,
    ),
    (
        "font-family",
        "\"A b\", c",
        "font-family: \"A b\", c;",
        "font-family: \"A b\", c;",
        true,
    ),
    (
        "--x",
        " 1px  2px ",
        "--x: 1px  2px;",
        "--x: 1px  2px;",
        true,
    ),
    (
        "line-height",
        "-0",
        "line-height: 0;",
        "line-height: 0;",
        false,
    ),
    // Two-step rows: the tiny member rounds to the default only in the text.
    (
        "transition",
        "e 0.0000001s",
        "transition: e 0s;",
        "transition: e;",
        false,
    ),
    (
        "border-image",
        "none 100.0000001%",
        "border-image: 100%;",
        "border-image: none;",
        false,
    ),
];

/// Whether every number in an authored value survives six-place fixed text, so
/// that its serialization is already the normalized form. Decided from the
/// text this package wrote, never from product output: a value with a math
/// function, an exponent, or more than three fraction digits is treated as
/// possibly lossy. Three fraction digits stay within six places after the
/// largest unit shift the vocabulary uses (milliseconds to seconds).
fn survives_six_places(text: &str) -> bool {
    const MATH: [&str; 5] = ["calc(", "min(", "max(", "clamp(", "calc-size("];
    if MATH.iter().any(|function| text.contains(function)) {
        return false;
    }
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let starts_number = bytes[index].is_ascii_digit()
            || (bytes[index] == b'.' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit));
        if !starts_number {
            index += 1;
            continue;
        }
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if bytes.get(index) == Some(&b'.') && bytes.get(index + 1).is_some_and(u8::is_ascii_digit) {
            index += 1;
            let start = index;
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
            if index - start > 3 {
                return false;
            }
        }
        if matches!(bytes.get(index), Some(b'e' | b'E')) {
            let after = match bytes.get(index + 1) {
                Some(b'+' | b'-') => index + 2,
                _ => index + 1,
            };
            if bytes.get(after).is_some_and(u8::is_ascii_digit) {
                return false;
            }
        }
    }
    true
}

/// Values that are invalid for their property or not a declaration at all, each
/// balanced so that the next `;` ends the unit. Recovery must drop the unit and
/// keep its valid siblings.
const INVALID: [&str; 12] = [
    "width: bogus-keyword-xyz",
    "width: 10px 20px 30px 40px 50px",
    "color: 1px",
    "color: red blue",
    "nonexistent-property: 1px",
    "width 10px",
    "width: \"text\"",
    "width: 10px !bogus",
    "margin: 1px 2px 3px 4px 5px",
    "{ nested: block }",
    "z-index: 1.5",
    "opacity: red",
];

/// Complete values and their forms cut off before the closing delimiter.
const CLOSURES: [(&str, &str, &str); 4] = [
    ("width", "calc(1px + 2px)", "calc(1px + 2px"),
    ("color", "rgb(1 2 3)", "rgb(1 2 3"),
    ("transform", "translate(1px, 2px)", "translate(1px, 2px"),
    ("margin", "calc(1px + 2px)", "calc(1px + 2px"),
];

/// One value fragment: a vocabulary word or a number built from digit counts,
/// with the separator that follows it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Piece {
    Word(u8),
    Number {
        sign: u8,
        integer: u8,
        fraction: u8,
        exponent: u8,
        magnitude: u16,
        unit: u8,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Item {
    piece: Piece,
    gap: u8,
}

/// The five lanes of a case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Lane {
    /// A known property from the metadata and one universal form.
    Global { property: u16, form: u8, alias: u8 },
    /// An independent known value.
    Row { row: u8 },
    /// A generated value recipe for a curated property.
    Recipe { property: u8, items: Vec<Item> },
    /// Independent valid siblings around an injected invalid unit.
    Siblings {
        before: u8,
        invalid: u8,
        after: u8,
        position: u8,
    },
    /// A value cut off before its closing delimiter.
    Closure { row: u8 },
}

/// A checked, bounded concrete case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueCase {
    lane: Lane,
    important: bool,
}

impl ValueCase {
    fn new(lane: Lane, important: bool) -> Result<Self> {
        let in_table = |index: u8, len: usize| usize::from(index) < len;
        let valid = match &lane {
            Lane::Global {
                property,
                form,
                alias,
            } => {
                usize::from(*property) < CssKnownProperty::all().len()
                    && in_table(*form, FORMS.len())
                    && (*alias == 0
                        || usize::from(*alias)
                            <= CssKnownProperty::all()[usize::from(*property)]
                                .aliases()
                                .len())
            }
            Lane::Row { row } => in_table(*row, ROWS.len()),
            Lane::Recipe { property, items } => {
                in_table(*property, CURATED.len())
                    && items.len() <= MAX_PIECES
                    && items.iter().all(|item| {
                        usize::from(item.gap) < GAPS.len()
                            && match item.piece {
                                Piece::Word(word) => in_table(word, WORDS.len()),
                                Piece::Number {
                                    sign,
                                    integer,
                                    fraction,
                                    exponent,
                                    magnitude,
                                    unit,
                                } => {
                                    sign < 3
                                        && integer <= MAX_DIGITS
                                        && fraction <= MAX_DIGITS
                                        && exponent < 3
                                        && magnitude <= MAX_EXPONENT
                                        && in_table(unit, UNITS.len())
                                }
                            }
                    })
            }
            Lane::Siblings {
                before,
                invalid,
                after,
                position,
            } => {
                in_table(*before, ROWS.len())
                    && in_table(*invalid, INVALID.len())
                    && in_table(*after, ROWS.len())
                    && *position < 3
            }
            Lane::Closure { row } => in_table(*row, CLOSURES.len()),
        };
        if !valid {
            return Err(error("value case table index or bound"));
        }
        Ok(Self { lane, important })
    }
}

fn number_text(
    sign: u8,
    integer: u8,
    fraction: u8,
    exponent: u8,
    magnitude: u16,
    unit: u8,
) -> String {
    let mut text = String::new();
    match sign {
        1 => text.push('-'),
        2 => text.push('+'),
        _ => {}
    }
    for index in 0..integer {
        text.push(char::from(b'1' + index % 9));
    }
    if fraction > 0 {
        text.push('.');
        for index in 0..fraction {
            text.push(char::from(b'1' + (index + 3) % 9));
        }
    }
    match exponent {
        1 => text.push_str(&format!("e{magnitude}")),
        2 => text.push_str(&format!("e-{magnitude}")),
        _ => {}
    }
    text.push_str(UNITS[usize::from(unit)]);
    text
}

fn recipe_text(items: &[Item]) -> String {
    let mut text = String::new();
    for (index, item) in items.iter().enumerate() {
        match item.piece {
            Piece::Word(word) => text.push_str(WORDS[usize::from(word)]),
            Piece::Number {
                sign,
                integer,
                fraction,
                exponent,
                magnitude,
                unit,
            } => text.push_str(&number_text(
                sign, integer, fraction, exponent, magnitude, unit,
            )),
        }
        if index + 1 < items.len() {
            text.push_str(GAPS[usize::from(item.gap)]);
        }
    }
    text
}

/// The property a case writes, resolved from its table index.
enum Name {
    Known(CssKnownProperty),
    Custom(CssCustomPropertyName),
}

impl Name {
    fn curated(index: u8) -> Result<(Self, &'static str)> {
        let spelling = CURATED[usize::from(index)];
        if spelling.starts_with("--") {
            let custom = CssCustomPropertyName::try_new(spelling)
                .ok_or_else(|| error("curated custom property name"))?;
            Ok((Self::Custom(custom), spelling))
        } else {
            let known = CssKnownProperty::from_name(spelling)
                .ok_or_else(|| error("curated property is not known"))?;
            Ok((Self::Known(known), spelling))
        }
    }

    fn named(spelling: &str) -> Result<Self> {
        if spelling.starts_with("--") {
            CssCustomPropertyName::try_new(spelling)
                .map(Self::Custom)
                .ok_or_else(|| error("custom property name"))
        } else {
            CssKnownProperty::from_name(spelling)
                .map(Self::Known)
                .ok_or_else(|| error("known property name"))
        }
    }

    fn reference(&self) -> CssPropertyNameRef<'_> {
        match self {
            Self::Known(property) => CssPropertyNameRef::Known(*property),
            Self::Custom(name) => CssPropertyNameRef::Custom(name),
        }
    }

    fn matches(&self, declaration: &CssDeclaration) -> bool {
        match (self.reference(), declaration.property_name()) {
            (CssPropertyNameRef::Known(left), CssPropertyNameRef::Known(right)) => left == right,
            (CssPropertyNameRef::Custom(left), CssPropertyNameRef::Custom(right)) => {
                left.as_str() == right.as_str()
            }
            _ => false,
        }
    }
}

/// The target: one evaluator shared by workers, properties, coverage and replay.
pub struct ValueTarget {
    descriptor: TargetDescriptor,
    checker: Checker,
    rows: &'static [Row],
    exact: fn(&str) -> bool,
}

impl Default for ValueTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl ValueTarget {
    /// The number of source-qualified literal cases.
    pub const LITERALS: u8 = 30;

    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptor: descriptor("surgeist-css::property-value"),
            checker: check::DOCUMENTED,
            rows: &ROWS,
            exact: survives_six_places,
        }
    }

    /// Replaces the six-place survival decision to plant a controlled failure.
    #[cfg(test)]
    pub(crate) fn with_exactness(exact: fn(&str) -> bool) -> Self {
        Self {
            exact,
            ..Self::new()
        }
    }

    #[cfg(test)]
    pub(crate) fn with_checker(checker: Checker) -> Self {
        Self {
            checker,
            ..Self::new()
        }
    }

    /// Replaces the independent known-value table to plant a controlled failure.
    #[cfg(test)]
    pub(crate) fn with_rows(rows: &'static [Row]) -> Self {
        Self {
            rows,
            ..Self::new()
        }
    }

    /// The number of retained known failures.
    pub const KNOWN_FAILURES: u8 = 2;

    /// Minimized inputs that fail today. They are not seeds and not regressions
    /// (neither would pass): each is retained with the failure it reproduces.
    ///
    /// 0 and 1: the property-value front retains a value with a top-level `{}`
    /// block beside other tokens once `var()` defers its validation, but the
    /// declaration front drops the same text (Syntax 3 section 5.5.6), so the
    /// retained declaration cannot be written back as a declaration (#2884).
    pub fn known_failure(&self, id: u8) -> Result<ValueCase> {
        let word = |text: &str| -> Result<u8> {
            WORDS
                .iter()
                .position(|candidate| *candidate == text)
                .and_then(|index| u8::try_from(index).ok())
                .ok_or_else(|| error("known failure word"))
        };
        let item = |text: &str, gap: u8| -> Result<Item> {
            Ok(Item {
                piece: Piece::Word(word(text)?),
                gap,
            })
        };
        let block_with_substitution = |closed: bool| -> Result<ValueCase> {
            let mut items = vec![item("{", 0)?, item("}", 1)?, item("var(--x", 0)?];
            if closed {
                items.push(item(")", 0)?);
            }
            ValueCase::new(Lane::Recipe { property: 0, items }, false)
        };
        match id {
            0 => block_with_substitution(true),
            1 => block_with_substitution(false),
            _ => Err(error("unknown known failure")),
        }
    }

    /// Source-qualified literals: every known value, sibling and closure row, and
    /// one metadata case for each universal form.
    pub fn literal(&self, id: u8) -> Result<ValueCase> {
        let id = usize::from(id);
        if id < ROWS.len() {
            // Custom-property rows keep normal importance; the rest alternate.
            let important = id % 4 == 3 && !ROWS[id].0.starts_with("--");
            ValueCase::new(Lane::Row { row: id as u8 }, important)
        } else if id < ROWS.len() + 4 {
            let slot = (id - ROWS.len()) as u8;
            ValueCase::new(
                Lane::Siblings {
                    before: slot,
                    invalid: slot * 3,
                    after: slot + 4,
                    position: slot % 3,
                },
                slot % 2 == 1,
            )
        } else if id < ROWS.len() + 4 + CLOSURES.len() {
            ValueCase::new(
                Lane::Closure {
                    row: (id - ROWS.len() - 4) as u8,
                },
                false,
            )
        } else {
            Err(error("unknown literal"))
        }
    }
}

type Outcome = std::result::Result<Vec<&'static str>, Violation>;

/// A recovery the original parse reported: its error code and action.
type Recovery = (CssErrorCode, CssRecoveryAction);

/// What the property-value front retained, the properties it checked and the
/// recoveries it reported.
type Retained = (CssDeclaration, Vec<&'static str>, Vec<Recovery>);

fn limits() -> CssSerializationLimits {
    CssSerializationLimits::default()
}

fn one<'a>(
    what: &str,
    declarations: &'a [CssDeclaration],
) -> std::result::Result<&'a CssDeclaration, Violation> {
    match declarations {
        [declaration] => Ok(declaration),
        other => Err(Violation::new(
            AGREEMENT,
            "declaration-count-differs",
            format!("{what} retained {} declarations, expected one", other.len()),
        )),
    }
}

fn importance_suffix(important: bool) -> &'static str {
    if important { " !important" } else { "" }
}

fn importance_of(important: bool) -> CssImportance {
    if important {
        CssImportance::Important
    } else {
        CssImportance::Normal
    }
}

/// Specified round trip under the explicit normalization described above.
fn round_trip(
    parser: &CssParser,
    original: &CssDeclaration,
    name: &Name,
    trip: &Trip<'_>,
) -> std::result::Result<Option<(String, String)>, Violation> {
    let Trip {
        important,
        spelling_only,
        fixed_at_first,
        tolerated,
    } = *trip;
    let Ok(first) = original.serialize_specified(limits()) else {
        // A refused serialization is a missing capability, reported by omission.
        return Ok(None);
    };
    if first.ends_with(" !important;") != important {
        return Err(Violation::new(
            ROUND_TRIP,
            "importance-not-written",
            format!("serialized {first:?} disagrees with importance {important}"),
        ));
    }
    let again = parser.parse_block_contents(&first);
    // A clean original must serialize to clean text. A recovered original may
    // keep a lexical recovery in its spelling (for example a backslash-newline),
    // so its text may repeat a diagnostic the original already reported, and
    // no other.
    let introduced = again
        .diagnostics()
        .iter()
        .find(|diagnostic| !tolerated.contains(&(diagnostic.error().code(), diagnostic.action())));
    if let Some(diagnostic) = introduced {
        return Err(Violation::new(
            ROUND_TRIP,
            "serialization-does-not-reparse-cleanly",
            format!(
                "{first:?} reparses with {:?} {:?}",
                diagnostic.error().code(),
                diagnostic.action()
            ),
        ));
    }
    let [second] = again.syntax().as_slice() else {
        return Err(Violation::new(
            ROUND_TRIP,
            "serialization-reparses-to-other-count",
            format!(
                "{first:?} reparses to {} declarations",
                again.syntax().len()
            ),
        ));
    };
    if !name.matches(second) || (second.importance() == CssImportance::Important) != important {
        return Err(Violation::new(
            ROUND_TRIP,
            "reparse-changes-property-or-importance",
            format!("{first:?} reparses to another property or importance"),
        ));
    }
    let Ok(next) = second.serialize_specified(limits()) else {
        return Err(Violation::new(
            ROUND_TRIP,
            "reserialization-refused",
            format!("{first:?} reparses to a declaration that refuses serialization"),
        ));
    };
    if fixed_at_first && next != first {
        return Err(Violation::new(
            ROUND_TRIP,
            "normalized-form-not-a-fixed-point",
            format!("{first:?} reserializes as {next:?}"),
        ));
    }
    if !fixed_at_first {
        // A number that does not survive six-place text may serialize rounded
        // first and, once reparsed as that exact value, without a default it
        // now equals. The normalization is then reached by the second text, and
        // the third must equal it.
        let third = parser.parse_block_contents(&next);
        let tolerated_third = third.diagnostics().iter().all(|diagnostic| {
            tolerated.contains(&(diagnostic.error().code(), diagnostic.action()))
        });
        let stable = match third.syntax().as_slice() {
            [declaration] if tolerated_third => {
                name.matches(declaration)
                    && (declaration.importance() == CssImportance::Important) == important
                    && declaration.serialize_specified(limits()).as_deref() == Ok(next.as_str())
            }
            _ => false,
        };
        if !stable {
            return Err(Violation::new(
                ROUND_TRIP,
                "second-serialization-not-a-fixed-point",
                format!("{first:?} then {next:?} does not reach a fixed point"),
            ));
        }
    }
    if spelling_only && original != second {
        return Err(Violation::new(
            ROUND_TRIP,
            "spelling-only-serialization-changes-the-value",
            format!("{first:?} reparses to a structurally different declaration"),
        ));
    }
    Ok(Some((first, next)))
}

/// How strictly a round trip is checked.
#[derive(Clone, Copy)]
struct Trip<'a> {
    important: bool,
    /// The first serialization changes spelling alone: the reparse must equal
    /// the original structurally.
    spelling_only: bool,
    /// Every number survives six-place text, so the first serialization is
    /// already the normalized form and reserializing it gives the same text.
    fixed_at_first: bool,
    tolerated: &'a [Recovery],
}

/// The property-value front and the block-contents front share one declaration
/// grammar, so a clean whole value is the same declaration in both.
fn agreement(
    checker: &Checker,
    parser: &CssParser,
    name: &Name,
    spelling: &str,
    text: &str,
    important: bool,
) -> std::result::Result<Option<Retained>, Violation> {
    let report = parser.parse_property_value(
        text,
        CssPropertyTextTarget::Property(name.reference()),
        importance_of(important),
    );
    checker.diagnostics(text, report.diagnostics())?;
    check::cleanliness(&report)?;
    let mut checked = vec![check::COORDINATES, check::CLEANLINESS];
    let Some(declaration) = report.syntax() else {
        if report.is_clean() {
            return Err(Violation::new(
                check::CLEANLINESS,
                "no-declaration-without-diagnostic",
                "the property-value front retained nothing and reported nothing",
            ));
        }
        return Ok(None);
    };
    nodes::declarations(
        checker,
        text,
        std::slice::from_ref(declaration),
        "value front",
    )?;
    checked.push(nodes::RETAINED);
    if !name.matches(declaration)
        || (declaration.importance() == CssImportance::Important) != important
    {
        return Err(Violation::new(
            nodes::RETAINED,
            "value-front-property-or-importance",
            "the retained declaration is not the requested property and importance",
        ));
    }
    if report.is_clean() {
        let block = format!("{spelling}: {text}{}", importance_suffix(important));
        let again = parser.parse_block_contents(&block);
        if !again.is_clean() {
            return Err(Violation::new(
                AGREEMENT,
                "block-front-rejects-a-clean-value",
                format!("{block:?} is not clean in block contents"),
            ));
        }
        let other = one("block contents", again.syntax().as_slice())?;
        if other != declaration {
            return Err(Violation::new(
                AGREEMENT,
                "fronts-retain-different-declarations",
                format!("{block:?} differs between the two fronts"),
            ));
        }
        checked.push(AGREEMENT);
    }
    let recovered = report
        .diagnostics()
        .iter()
        .map(|diagnostic| (diagnostic.error().code(), diagnostic.action()))
        .collect();
    Ok(Some((declaration.clone(), checked, recovered)))
}

fn lane_global(
    checker: &Checker,
    parser: &CssParser,
    case: &ValueCase,
    property: u16,
    form: u8,
    alias: u8,
) -> Outcome {
    let known = CssKnownProperty::all()[usize::from(property)];
    let canonical = known.canonical_name();
    let text = FORMS[usize::from(form)];
    let mut checked = vec![GLOBAL, METADATA];

    // Metadata is a contract input: names, aliases and grammar must agree.
    if CssKnownProperty::from_name(canonical) != Some(known)
        || known
            .aliases()
            .iter()
            .any(|alias| CssKnownProperty::from_name(alias) != Some(known))
        || known.stable_id().is_empty()
        || known.metadata().map(|metadata| metadata.grammar()).ok() != Some(known.grammar())
    {
        return Err(Violation::new(
            METADATA,
            "metadata-disagrees-with-names",
            format!("{canonical}: names, aliases or grammar are inconsistent"),
        ));
    }

    let name = Name::Known(known);
    let Some((declaration, mut more, _)) =
        agreement(checker, parser, &name, canonical, text, case.important)?
    else {
        return Err(Violation::new(
            GLOBAL,
            "universal-form-rejected",
            format!("{canonical}: {text} was rejected"),
        ));
    };
    checked.append(&mut more);
    let known_declaration = declaration
        .known()
        .ok_or_else(|| Violation::new(GLOBAL, "universal-form-not-known", canonical))?;
    let expected_global = match text {
        "inherit" => Some(CssGlobalKeyword::Inherit),
        "initial" => Some(CssGlobalKeyword::Initial),
        "unset" => Some(CssGlobalKeyword::Unset),
        "revert" => Some(CssGlobalKeyword::Revert),
        "revert-layer" => Some(CssGlobalKeyword::RevertLayer),
        _ => None,
    };
    let recorded = known_declaration.global() == expected_global
        && known_declaration.substitution_dependent().is_some() == expected_global.is_none()
        && known_declaration.property() == known;
    if !recorded {
        return Err(Violation::new(
            GLOBAL,
            "universal-form-misclassified",
            format!("{canonical}: {text} is not retained as its global or substitution form"),
        ));
    }

    // The written text is fully determined by the metadata name and the form.
    let expected = format!("{canonical}: {text}{};", importance_suffix(case.important));
    if declaration.serialize_specified(limits()).as_deref() != Ok(expected.as_str()) {
        return Err(Violation::new(
            KNOWN,
            "universal-form-text-differs",
            format!("{canonical}: {text} does not serialize as {expected:?}"),
        ));
    }
    checked.push(KNOWN);
    let trip = Trip {
        important: case.important,
        spelling_only: true,
        fixed_at_first: true,
        tolerated: &[],
    };
    if round_trip(parser, &declaration, &name, &trip)?.is_some() {
        checked.push(ROUND_TRIP);
    }

    // An alias names the same property and the same declaration.
    if alias > 0 {
        let spelling = known.aliases()[usize::from(alias) - 1];
        let block = format!("{spelling}: {text}{}", importance_suffix(case.important));
        let report = parser.parse_block_contents(&block);
        let other = one("alias", report.syntax().as_slice())?;
        if !report.is_clean() || other != &declaration {
            return Err(Violation::new(
                METADATA,
                "alias-differs-from-canonical-name",
                format!("{spelling} does not write the same declaration as {canonical}"),
            ));
        }
    }
    Ok(checked)
}

fn lane_row(
    checker: &Checker,
    rows: &[Row],
    parser: &CssParser,
    case: &ValueCase,
    row: u8,
) -> Outcome {
    let (spelling, value, expected_first, expected_next, spelling_only) = rows[usize::from(row)];
    let name = Name::named(spelling)
        .map_err(|_| Violation::new(KNOWN, "row-property-unknown", spelling))?;
    let Some((declaration, mut checked, _)) =
        agreement(checker, parser, &name, spelling, value, case.important)?
    else {
        return Err(Violation::new(
            KNOWN,
            "known-value-rejected",
            format!("{spelling}: {value} was rejected"),
        ));
    };
    let with_importance = |text: &str| {
        if case.important {
            text.replace(';', " !important;")
        } else {
            text.to_owned()
        }
    };
    let expected = with_importance(expected_first);
    let expected_next = with_importance(expected_next);
    match declaration.serialize_specified(limits()) {
        Ok(text) if text == expected => {}
        other => {
            return Err(Violation::new(
                KNOWN,
                "known-value-text-differs",
                format!("{spelling}: {value} serializes as {other:?}, expected {expected:?}"),
            ));
        }
    }
    checked.push(KNOWN);
    // The handwritten row says whether its first serialization is already the
    // normalized form: its second text equals its first.
    let trip = Trip {
        important: case.important,
        spelling_only,
        fixed_at_first: expected_next == expected,
        tolerated: &[],
    };
    match round_trip(parser, &declaration, &name, &trip)? {
        Some((_, next)) => {
            if next != expected_next {
                return Err(Violation::new(
                    KNOWN,
                    "known-value-second-text-differs",
                    format!(
                        "{spelling}: {value} reserializes as {next:?}, expected {expected_next:?}"
                    ),
                ));
            }
            checked.push(ROUND_TRIP);
        }
        None => {
            return Err(Violation::new(
                ROUND_TRIP,
                "known-value-serialization-refused",
                format!("{spelling}: {value}"),
            ));
        }
    }
    Ok(checked)
}

fn lane_recipe(
    checker: &Checker,
    parser: &CssParser,
    case: &ValueCase,
    property: u8,
    items: &[Item],
    exact: fn(&str) -> bool,
) -> Outcome {
    let (name, spelling) = Name::curated(property).map_err(|_| {
        Violation::new(
            AGREEMENT,
            "curated-property-unresolved",
            property.to_string(),
        )
    })?;
    let text = recipe_text(items);
    let Some((declaration, mut checked, recovered)) =
        agreement(checker, parser, &name, spelling, &text, case.important)?
    else {
        return Ok(vec![check::COORDINATES, check::CLEANLINESS]);
    };
    let trip = Trip {
        important: case.important,
        spelling_only: false,
        fixed_at_first: exact(&text),
        tolerated: &recovered,
    };
    if round_trip(parser, &declaration, &name, &trip)?.is_some() {
        checked.push(ROUND_TRIP);
    }
    // Expansion is total over a retained declaration; it may decline, not panic.
    let _ = declaration.expand();
    Ok(checked)
}

fn lane_siblings(
    rows: &[Row],
    checker: &Checker,
    parser: &CssParser,
    case: &ValueCase,
    (before, invalid, after, position): (u8, u8, u8, u8),
) -> Outcome {
    let unit = |row: u8| {
        let (spelling, value, expected, _, _) = rows[usize::from(row)];
        let text = format!("{spelling}: {value}{}", importance_suffix(case.important));
        let expected = if case.important {
            expected.replace(';', " !important;")
        } else {
            expected.to_owned()
        };
        (spelling, text, expected)
    };
    let valid = [unit(before), unit(after)];
    let bad = INVALID[usize::from(invalid)];
    // Units in source order, each ending at its own semicolon.
    let units: Vec<(Option<usize>, &str)> = match position {
        0 => vec![(Some(0), &valid[0].1), (None, bad), (Some(1), &valid[1].1)],
        1 => vec![(None, bad), (Some(0), &valid[0].1), (Some(1), &valid[1].1)],
        _ => vec![(Some(0), &valid[0].1), (Some(1), &valid[1].1), (None, bad)],
    };
    let mut text = String::new();
    let mut invalid_range = (0, 0);
    for (slot, unit_text) in &units {
        let start = text.len();
        text.push_str(unit_text);
        text.push(';');
        if slot.is_none() {
            invalid_range = (start, text.len());
        }
        text.push(' ');
    }
    let report = parser.parse_block_contents(&text);
    checker.diagnostics(&text, report.diagnostics())?;
    check::cleanliness(&report)?;
    nodes::declarations(checker, &text, report.syntax().as_slice(), "sibling block")?;

    let retained = report.syntax();
    let expected_texts = [&valid[0].2, &valid[1].2];
    let got: Vec<String> = retained
        .iter()
        .map(|declaration| {
            declaration
                .serialize_specified(limits())
                .unwrap_or_else(|cause| format!("<{cause}>"))
        })
        .collect();
    if got.len() != 2 || got[0] != *expected_texts[0] || got[1] != *expected_texts[1] {
        return Err(Violation::new(
            SIBLINGS,
            "valid-siblings-not-recovered",
            format!("{text:?} retained {got:?}, expected {expected_texts:?}"),
        ));
    }
    for (retained, (spelling, _, _)) in retained.iter().zip(&valid) {
        let name = Name::named(spelling)
            .map_err(|_| Violation::new(SIBLINGS, "row-property-unknown", spelling))?;
        if !name.matches(retained)
            || (retained.importance() == CssImportance::Important) != case.important
        {
            return Err(Violation::new(
                SIBLINGS,
                "sibling-identity-differs",
                format!("{spelling} lost its property or importance"),
            ));
        }
    }
    if report.diagnostics().is_empty() {
        return Err(Violation::new(
            SIBLINGS,
            "invalid-unit-not-reported",
            format!("{bad:?} produced no diagnostic"),
        ));
    }
    for diagnostic in report.diagnostics() {
        let at = diagnostic.error().position().byte_offset().value();
        let span = diagnostic.span();
        if at < invalid_range.0
            || at > invalid_range.1
            || span.start().byte_offset().value() < invalid_range.0
            || span.end().byte_offset().value() > invalid_range.1
        {
            return Err(Violation::new(
                SIBLINGS,
                "diagnostic-outside-injected-unit",
                format!("{bad:?} at {invalid_range:?} drew a diagnostic at {at}"),
            ));
        }
    }
    Ok(vec![
        SIBLINGS,
        check::COORDINATES,
        check::CLEANLINESS,
        nodes::RETAINED,
    ])
}

fn lane_closure(checker: &Checker, parser: &CssParser, row: u8) -> Outcome {
    let (spelling, complete, cut) = CLOSURES[usize::from(row)];
    let name = Name::named(spelling)
        .map_err(|_| Violation::new(CLOSURE, "row-property-unknown", spelling))?;
    let target = CssPropertyTarget::Property(name.reference());
    let build = |text: &str| -> std::result::Result<bool, Violation> {
        let components = CssComponentValues::parse(parser, text).map_err(|cause| {
            Violation::new(
                CLOSURE,
                "components-do-not-parse",
                format!("{text:?}: {cause}"),
            )
        })?;
        Ok(
            CssDeclaration::try_from_components(parser, target, components, CssImportance::Normal)
                .is_ok(),
        )
    };
    let report = parser.parse_property_value(
        cut,
        CssPropertyTextTarget::Property(name.reference()),
        CssImportance::Normal,
    );
    checker.diagnostics(cut, report.diagnostics())?;
    let closed = report
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure);
    if report.syntax().is_none() || !closed {
        return Err(Violation::new(
            CLOSURE,
            "recovered-end-of-input-not-retained",
            format!("{spelling}: {cut:?} was not retained with an implicit closure"),
        ));
    }
    if build(cut)? {
        return Err(Violation::new(
            CLOSURE,
            "checked-constructor-admits-implicit-closure",
            format!("{spelling}: {cut:?} was admitted by the checked constructor"),
        ));
    }
    let whole = parser.parse_property_value(
        complete,
        CssPropertyTextTarget::Property(name.reference()),
        CssImportance::Normal,
    );
    if !whole.is_clean() || whole.syntax().is_none() || !build(complete)? {
        return Err(Violation::new(
            CLOSURE,
            "complete-value-not-admitted",
            format!("{spelling}: {complete:?} should be clean and admitted"),
        ));
    }
    Ok(vec![CLOSURE, check::COORDINATES])
}

impl CaseTarget for ValueTarget {
    type Case = ValueCase;

    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }

    fn decode(&self, input: &[u8]) -> Result<ValueCase> {
        let mut reader = Reader::new(input, MAGIC, MAX_CASE)?;
        let important = match reader.u8()? {
            0 => false,
            1 => true,
            _ => return Err(error("importance tag")),
        };
        let lane = match reader.u8()? {
            0 => Lane::Global {
                property: u16::try_from(reader.u32()?).map_err(|_| error("property index"))?,
                form: reader.u8()?,
                alias: reader.u8()?,
            },
            1 => Lane::Row { row: reader.u8()? },
            2 => {
                let property = reader.u8()?;
                let count = reader.count(MAX_PIECES)?;
                let mut items = Vec::new();
                items
                    .try_reserve_exact(count)
                    .map_err(|_| error("recipe allocation"))?;
                for _ in 0..count {
                    let piece = match reader.u8()? {
                        0 => Piece::Word(reader.u8()?),
                        1 => Piece::Number {
                            sign: reader.u8()?,
                            integer: reader.u8()?,
                            fraction: reader.u8()?,
                            exponent: reader.u8()?,
                            magnitude: u16::try_from(reader.u32()?)
                                .map_err(|_| error("exponent magnitude"))?,
                            unit: reader.u8()?,
                        },
                        _ => return Err(error("piece tag")),
                    };
                    items.push(Item {
                        piece,
                        gap: reader.u8()?,
                    });
                }
                Lane::Recipe { property, items }
            }
            3 => Lane::Siblings {
                before: reader.u8()?,
                invalid: reader.u8()?,
                after: reader.u8()?,
                position: reader.u8()?,
            },
            4 => Lane::Closure { row: reader.u8()? },
            _ => return Err(error("lane tag")),
        };
        reader.finish()?;
        ValueCase::new(lane, important)
    }

    fn encode(&self, case: &ValueCase) -> Result<Vec<u8>> {
        let mut writer = Writer::new(MAGIC, MAX_CASE)?;
        writer.u8(u8::from(case.important))?;
        match &case.lane {
            Lane::Global {
                property,
                form,
                alias,
            } => {
                writer.u8(0)?;
                writer.u32(u32::from(*property))?;
                writer.u8(*form)?;
                writer.u8(*alias)?;
            }
            Lane::Row { row } => {
                writer.u8(1)?;
                writer.u8(*row)?;
            }
            Lane::Recipe { property, items } => {
                writer.u8(2)?;
                writer.u8(*property)?;
                writer.count(items.len(), MAX_PIECES)?;
                for item in items {
                    match item.piece {
                        Piece::Word(word) => {
                            writer.u8(0)?;
                            writer.u8(word)?;
                        }
                        Piece::Number {
                            sign,
                            integer,
                            fraction,
                            exponent,
                            magnitude,
                            unit,
                        } => {
                            writer.u8(1)?;
                            writer.u8(sign)?;
                            writer.u8(integer)?;
                            writer.u8(fraction)?;
                            writer.u8(exponent)?;
                            writer.u32(u32::from(magnitude))?;
                            writer.u8(unit)?;
                        }
                    }
                    writer.u8(item.gap)?;
                }
            }
            Lane::Siblings {
                before,
                invalid,
                after,
                position,
            } => {
                writer.u8(3)?;
                writer.u8(*before)?;
                writer.u8(*invalid)?;
                writer.u8(*after)?;
                writer.u8(*position)?;
            }
            Lane::Closure { row } => {
                writer.u8(4)?;
                writer.u8(*row)?;
            }
        }
        Ok(writer.finish())
    }

    fn generate(&self, input: &[u8]) -> Result<Generation<ValueCase>> {
        generate(&self.descriptor, input).map(Generation::Generated)
    }

    fn evaluate(&self, case: &ValueCase) -> Result<CaseOutcome> {
        let parser = CssParser::default();
        let checker = &self.checker;
        let outcome = match &case.lane {
            Lane::Global {
                property,
                form,
                alias,
            } => lane_global(checker, &parser, case, *property, *form, *alias),
            Lane::Row { row } => lane_row(checker, self.rows, &parser, case, *row),
            Lane::Recipe { property, items } => {
                lane_recipe(checker, &parser, case, *property, items, self.exact)
            }
            Lane::Siblings {
                before,
                invalid,
                after,
                position,
            } => lane_siblings(
                self.rows,
                checker,
                &parser,
                case,
                (*before, *invalid, *after, *position),
            ),
            Lane::Closure { row } => lane_closure(checker, &parser, *row),
        };
        match outcome {
            Err(found) => violation(&found),
            Ok(mut checked) => {
                checked.sort_unstable();
                checked.dedup();
                evaluated(&checked)
            }
        }
    }

    fn reduction_candidates(
        &self,
        case: &ValueCase,
        emit: &mut dyn FnMut(ValueCase) -> bool,
    ) -> Result<()> {
        // Only recipes shrink; a table lane is already a single independent row.
        let Lane::Recipe { property, items } = &case.lane else {
            return Ok(());
        };
        let mut candidates: Vec<Vec<Item>> = Vec::new();
        if !items.is_empty() {
            candidates.push(Vec::new());
            candidates.push(items[..items.len() / 2].to_vec());
            candidates.push(items[items.len() / 2..].to_vec());
        }
        for index in 0..items.len() {
            let mut fewer = items.clone();
            fewer.remove(index);
            candidates.push(fewer);
        }
        for items in candidates {
            let candidate = ValueCase::new(
                Lane::Recipe {
                    property: *property,
                    items,
                },
                case.important,
            )?;
            if !emit(candidate) {
                break;
            }
        }
        Ok(())
    }
}

fn generate(descriptor: &TargetDescriptor, input: &[u8]) -> Result<ValueCase> {
    let mut entropy = Cursor::new(input);
    let important = entropy.below(4) == 0;
    let lane = match entropy.below(20) {
        0..=5 => Lane::Global {
            property: u16::try_from(
                (usize::from(entropy.byte()) << 8 | usize::from(entropy.byte()))
                    % CssKnownProperty::all().len(),
            )
            .unwrap_or(0),
            form: entropy.below(FORMS.len()) as u8,
            alias: 0,
        },
        6..=8 => Lane::Row {
            row: entropy.below(ROWS.len()) as u8,
        },
        9..=16 => {
            let count = entropy.below(MAX_PIECES + 1);
            let mut items = Vec::new();
            for _ in 0..count {
                let piece = if entropy.below(4) == 0 {
                    Piece::Number {
                        sign: entropy.below(3) as u8,
                        integer: [0, 1, 2, 5, 17, 40][entropy.below(6)],
                        fraction: [0, 0, 1, 6, 7, 40][entropy.below(6)],
                        exponent: entropy.below(3) as u8,
                        magnitude: [0, 1, 5, 38, 308, 309, 400, 999][entropy.below(8)],
                        unit: entropy.below(UNITS.len()) as u8,
                    }
                } else {
                    Piece::Word(entropy.below(WORDS.len()) as u8)
                };
                items.push(Item {
                    piece,
                    gap: entropy.below(GAPS.len()) as u8,
                });
            }
            Lane::Recipe {
                property: entropy.below(CURATED.len()) as u8,
                items,
            }
        }
        17..=18 => Lane::Siblings {
            before: entropy.below(ROWS.len()) as u8,
            invalid: entropy.below(INVALID.len()) as u8,
            after: entropy.below(ROWS.len()) as u8,
            position: entropy.below(3) as u8,
        },
        _ => Lane::Closure {
            row: entropy.below(CLOSURES.len()) as u8,
        },
    };
    let mut lane = lane;
    // An alias spelling exists only for metadata properties that have one.
    if let Lane::Global {
        property, alias, ..
    } = &mut lane
    {
        let aliases = CssKnownProperty::all()[usize::from(*property)]
            .aliases()
            .len();
        if aliases > 0 && entropy.flag() {
            *alias = 1 + entropy.below(aliases) as u8;
        }
    }
    let case = ValueCase::new(lane, important)?;
    credit(descriptor, &case)?;
    Ok(case)
}

/// Charges the case against the descriptor's hard limits before use. The case
/// bounds (24 fragments, 40 digit numbers) keep every case within them.
fn credit(descriptor: &TargetDescriptor, case: &ValueCase) -> Result<()> {
    let mut budget = Budget::new(descriptor.limits());
    let (nodes, text_units) = match &case.lane {
        Lane::Recipe { items, .. } => (items.len() as u64 + 1, recipe_text(items).len() as u64),
        _ => (2, 128),
    };
    budget.credit(BudgetCharge {
        bytes: text_units + 256,
        nodes,
        depth: 1,
        text_units,
        operations: 4,
        ..BudgetCharge::default()
    })
}
