//! Structured nested rules and declarations with deliberately injected invalid
//! units beside independent valid siblings.
//!
//! A case is a recipe tree: style rules, conditional groups, ignorable text and
//! invalid units at the sheet level, and declarations, invalid declarations,
//! nested rules and comments inside blocks. The recipe is rendered to source by
//! this module alone, which also records the byte range of every injected
//! invalid unit and the rules and declarations that must survive. The parser's
//! report is then compared with that independent expectation: retained rule
//! kinds and declaration identities in source order, diagnostics only inside
//! injected units (and at least one in each), a clean report exactly when
//! nothing invalid was injected, and, through the crate's own CSSOM
//! declaration selection, the winner of every duplicated name by importance
//! then source order.
use crate::{
    check::{self, Check, Checker, Violation},
    codec::{Reader, Writer},
    cursor::Cursor,
    descriptor, error, evaluated, nodes, violation,
};
use std::collections::BTreeMap;
use surgeist_css::{
    CssParser,
    declaration::{
        CssDeclaration, CssDeclarationList, CssImportance, CssSpecifiedDeclarationBlock,
    },
    property::{CssKnownProperty, CssPropertyNameRef},
    rule::{CssRule, CssSheet, CssStyleRule},
    serialization::CssSerializationLimits,
};
use surgeist_fuzz::{
    CaseOutcome, CaseTarget, Generation, Result, TargetDescriptor,
    generation::{Budget, BudgetCharge},
};

#[cfg(test)]
mod tests;

pub(crate) const MAGIC: [u8; 4] = *b"CSRL";
const MAX_CASE: usize = 16 * 1024;
const MAX_NODES: usize = 64;
const MAX_DEPTH: usize = 5;
const MAX_ITEMS: usize = 12;

const RECOVERY: &str = "surgeist-css::rule-recovery";
const LOCALITY: &str = "surgeist-css::diagnostic-locality";
const SELECTION: &str = "surgeist-css::declaration-selection";
const STRUCTURE: &str = "surgeist-css::nested-structure";

/// The property a valid declaration must retain: a known property by canonical
/// name, or a custom property by its decoded, case-sensitive name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Expect {
    Known(&'static str),
    Custom(&'static str),
}

/// (authored declaration, retained property, importance)
pub(crate) type DeclRow = (&'static str, Expect, bool);

/// Valid declarations with escaped, uppercase, custom, Unicode, string,
/// comment, block, numeric-extreme and deferred-substitution spellings. All are
/// longhands or custom properties, so duplicate selection has a simple oracle.
const DECLS: [DeclRow; 20] = [
    ("color: red", Expect::Known("color"), false),
    ("COLOR: BLUE", Expect::Known("color"), false),
    ("color: red !important", Expect::Known("color"), true),
    ("width: 10px", Expect::Known("width"), false),
    ("width: 20px !important", Expect::Known("width"), true),
    ("\\77 idth: 2px", Expect::Known("width"), false),
    ("height: calc(100% - 1px)", Expect::Known("height"), false),
    ("opacity: 1e999", Expect::Known("opacity"), false),
    (
        "z-index: 99999999999999999999",
        Expect::Known("z-index"),
        false,
    ),
    ("display: block", Expect::Known("display"), false),
    ("width: var(--w)", Expect::Known("width"), false),
    (
        "line-height: 0.0000001",
        Expect::Known("line-height"),
        false,
    ),
    ("--x: 1px", Expect::Custom("--x"), false),
    ("--X: A", Expect::Custom("--X"), false),
    ("--\u{e9}: \u{1f600}", Expect::Custom("--\u{e9}"), false),
    ("\\--e: 1", Expect::Custom("--e"), false),
    ("--s: \"a;b}c\"", Expect::Custom("--s"), false),
    ("--c: /* ; } */ x", Expect::Custom("--c"), false),
    ("--b: { a: b; c: d }", Expect::Custom("--b"), false),
    ("--v: var(--x, 1px) !important", Expect::Custom("--v"), true),
];

const SELECTORS: [&str; 10] = [
    ".a",
    "#i",
    "div > p",
    ":is(.x, .y)",
    "[data-k=\"v\"]",
    "\\61 b",
    "\u{e9}t\u{e9}",
    "a:hover",
    "*",
    ".a, .b",
];
/// Relative and explicit-nesting selectors for rules inside a style rule.
const NESTED: [&str; 5] = ["& .n", "&.m", "> .c", ".d", "&:hover"];

/// Conditional groups and the rule kind each must retain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Group {
    Media,
    Supports,
    Layer,
    Container,
}
const GROUPS: [(&str, Group); 6] = [
    ("@media screen", Group::Media),
    ("@media (min-width: 10px)", Group::Media),
    ("@supports (display: grid)", Group::Supports),
    ("@layer base", Group::Layer),
    ("@layer", Group::Layer),
    ("@container (min-width: 1px)", Group::Container),
];

/// Whole units that must be dropped, each balanced and complete.
const SKIPS: [&str; 4] = [
    "@unknown x { a { b: c } }",
    "@unknown;",
    ":::bad { color: red }",
    "a::: { color: red }",
];
/// Text that is retained nowhere and reports nothing at the sheet level.
const IGNORABLE: [&str; 4] = ["/* c */", "<!--", "-->", "/* \u{e9}\u{1f600}\u{2028} */"];
/// Comments that hide punctuation inside blocks and groups.
const COMMENTS: [&str; 3] = ["/* c */", "/* } ; { */", "/* \u{e9}\u{1f600}\u{2028} */"];
const SEPARATORS: [&str; 8] = ["", " ", "\n", "\r\n", "\r", "\u{c}", "\t", " /* s */ "];
/// Invalid declarations, each balanced so that the next `;` ends the unit.
const BAD: [&str; 12] = crate::values::INVALID;

/// A block member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Item {
    Decl { row: u8, sep: u8 },
    Bad { row: u8, sep: u8 },
    Nest { style: Style, sep: u8 },
    Comment { row: u8, sep: u8 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Style {
    selector: u8,
    items: Vec<Item>,
}

/// A sheet-level or group-level member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Node {
    Style {
        style: Style,
        sep: u8,
    },
    Group {
        group: u8,
        children: Vec<Node>,
        sep: u8,
    },
    Skip {
        row: u8,
        sep: u8,
    },
    Ignorable {
        row: u8,
        sep: u8,
    },
}

/// Which public front parses the rendered text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Front {
    Sheet,
    /// The unwrapped items of the first style rule, as a style attribute.
    Block,
}

/// A checked, bounded concrete case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RulesCase {
    front: Front,
    nodes: Vec<Node>,
}

fn count_node(node: &Node) -> usize {
    match node {
        Node::Style { style, .. } => 1 + count_style(style),
        Node::Group { children, .. } => 1 + children.iter().map(count_node).sum::<usize>(),
        Node::Skip { .. } | Node::Ignorable { .. } => 1,
    }
}

fn count_style(style: &Style) -> usize {
    style
        .items
        .iter()
        .map(|item| match item {
            Item::Nest { style, .. } => 1 + count_style(style),
            _ => 1,
        })
        .sum()
}

fn depth_node(node: &Node) -> usize {
    match node {
        Node::Style { style, .. } => 1 + depth_style(style),
        Node::Group { children, .. } => 1 + children.iter().map(depth_node).max().unwrap_or(0),
        Node::Skip { .. } | Node::Ignorable { .. } => 1,
    }
}

fn depth_style(style: &Style) -> usize {
    style
        .items
        .iter()
        .map(|item| match item {
            Item::Nest { style, .. } => 1 + depth_style(style),
            _ => 0,
        })
        .max()
        .unwrap_or(0)
}

/// Table indices and counts of a style rule whose selector comes from a table
/// of `selectors` entries; rules inside it use the nested-selector table.
fn valid_style(style: &Style, selectors: usize) -> bool {
    usize::from(style.selector) < selectors
        && style.items.len() <= MAX_ITEMS
        && style.items.iter().all(|item| match item {
            Item::Decl { row, sep } => usize::from(*row) < DECLS.len() && *sep < 8,
            Item::Bad { row, sep } => usize::from(*row) < BAD.len() && *sep < 8,
            Item::Comment { row, sep } => usize::from(*row) < COMMENTS.len() && *sep < 8,
            Item::Nest { style, sep } => *sep < 8 && valid_style(style, NESTED.len()),
        })
}

fn valid_node(node: &Node) -> bool {
    match node {
        Node::Style { style, sep } => *sep < 8 && valid_style(style, SELECTORS.len()),
        Node::Group {
            group,
            children,
            sep,
        } => *sep < 8 && usize::from(*group) < GROUPS.len() && children.iter().all(valid_node),
        Node::Skip { row, sep } => usize::from(*row) < SKIPS.len() && *sep < 8,
        Node::Ignorable { row, sep } => usize::from(*row) < IGNORABLE.len() && *sep < 8,
    }
}

impl RulesCase {
    fn new(front: Front, nodes: Vec<Node>) -> Result<Self> {
        let total: usize = nodes.iter().map(count_node).sum();
        let deepest = nodes.iter().map(depth_node).max().unwrap_or(0);
        if total > MAX_NODES || deepest > MAX_DEPTH || !nodes.iter().all(valid_node) {
            return Err(error("rules case table index or bound"));
        }
        Ok(Self { front, nodes })
    }
}

/// What must survive, computed from the recipe alone.
#[derive(Debug, Default)]
struct ExpStyle {
    /// Every valid declaration of the block in source order. Where the parser
    /// splits them between the rule and its nested declaration runs is not
    /// asserted; only their order and identity are.
    decls: Vec<DeclRow>,
    nested: Vec<ExpStyle>,
}

#[derive(Debug)]
enum ExpRule {
    Style(ExpStyle),
    Group(Group, Vec<ExpRule>),
}

/// Rendered source, the byte range of each unit that must draw a diagnostic,
/// and the surviving structure.
struct Rendered {
    text: String,
    bad: Vec<(usize, usize)>,
    rules: Vec<ExpRule>,
    block: ExpStyle,
}

struct Renderer<'a> {
    text: String,
    bad: Vec<(usize, usize)>,
    rows: &'a [DeclRow],
}

impl Renderer<'_> {
    fn separate(&mut self, sep: u8) {
        self.text.push_str(SEPARATORS[usize::from(sep)]);
    }

    fn bad_unit(&mut self, text: &str) {
        let start = self.text.len();
        self.text.push_str(text);
        self.text.push(';');
        self.bad.push((start, self.text.len()));
    }

    /// Items of one block. `drop_nested` renders valid nested rules as units
    /// that the declaration-list front must drop and report.
    fn items(&mut self, items: &[Item], drop_nested: bool) -> ExpStyle {
        let mut expected = ExpStyle::default();
        for item in items {
            match item {
                Item::Decl { row, sep } => {
                    let row = self.rows[usize::from(*row)];
                    self.text.push_str(row.0);
                    self.text.push(';');
                    expected.decls.push(row);
                    self.separate(*sep);
                }
                Item::Bad { row, sep } => {
                    self.bad_unit(BAD[usize::from(*row)]);
                    self.separate(*sep);
                }
                Item::Comment { row, sep } => {
                    self.text.push_str(COMMENTS[usize::from(*row)]);
                    self.separate(*sep);
                }
                Item::Nest { style, sep } => {
                    let start = self.text.len();
                    let mark = self.bad.len();
                    self.text.push_str(NESTED[usize::from(style.selector)]);
                    self.text.push_str(" {");
                    let child = self.items(&style.items, false);
                    self.text.push('}');
                    if drop_nested {
                        // The whole rule is one dropped unit with one report.
                        self.bad.truncate(mark);
                        self.bad.push((start, self.text.len()));
                    } else {
                        expected.nested.push(child);
                    }
                    self.separate(*sep);
                }
            }
        }
        expected
    }

    fn nodes(&mut self, nodes: &[Node]) -> Vec<ExpRule> {
        let mut rules = Vec::new();
        for node in nodes {
            match node {
                Node::Style { style, sep } => {
                    self.text.push_str(SELECTORS[usize::from(style.selector)]);
                    self.text.push_str(" {");
                    rules.push(ExpRule::Style(self.items(&style.items, false)));
                    self.text.push('}');
                    self.separate(*sep);
                }
                Node::Group {
                    group,
                    children,
                    sep,
                } => {
                    let (prelude, kind) = GROUPS[usize::from(*group)];
                    self.text.push_str(prelude);
                    self.text.push_str(" {");
                    let inner = self.nodes_in_group(children);
                    rules.push(ExpRule::Group(kind, inner));
                    self.text.push('}');
                    self.separate(*sep);
                }
                Node::Skip { row, sep } => {
                    let start = self.text.len();
                    self.text.push_str(SKIPS[usize::from(*row)]);
                    self.bad.push((start, self.text.len()));
                    self.separate(*sep);
                }
                Node::Ignorable { row, sep } => {
                    self.text.push_str(IGNORABLE[usize::from(*row)]);
                    self.separate(*sep);
                }
            }
        }
        rules
    }

    /// Inside a group CDO and CDC are not ignorable, so only comments render.
    fn nodes_in_group(&mut self, nodes: &[Node]) -> Vec<ExpRule> {
        let mut rules = Vec::new();
        for node in nodes {
            match node {
                Node::Ignorable { row, sep } => {
                    self.text
                        .push_str(COMMENTS[usize::from(*row) % COMMENTS.len()]);
                    self.separate(*sep);
                }
                other => rules.extend(self.nodes(std::slice::from_ref(other))),
            }
        }
        rules
    }
}

fn render(case: &RulesCase, rows: &[DeclRow]) -> Rendered {
    let mut renderer = Renderer {
        text: String::new(),
        bad: Vec::new(),
        rows,
    };
    let mut rules = Vec::new();
    let mut block = ExpStyle::default();
    match case.front {
        Front::Sheet => rules = renderer.nodes(&case.nodes),
        Front::Block => {
            let first = case.nodes.iter().find_map(|node| match node {
                Node::Style { style, .. } => Some(style),
                _ => None,
            });
            if let Some(style) = first {
                block = renderer.items(&style.items, true);
            }
        }
    }
    Rendered {
        text: renderer.text,
        bad: renderer.bad,
        rules,
        block,
    }
}

/// The CSSOM winner of each duplicated name: an important declaration beats a
/// normal one, and a later declaration beats an earlier one of equal
/// importance. Entries stay at their winners' authored positions.
fn expected_winners(rows: &[DeclRow], replaces: fn(bool, bool) -> bool) -> Vec<(usize, DeclRow)> {
    let mut winners: BTreeMap<String, (usize, DeclRow)> = BTreeMap::new();
    for (ordinal, row) in rows.iter().enumerate() {
        let key = match row.1 {
            Expect::Known(name) => format!("k:{name}"),
            Expect::Custom(name) => format!("c:{name}"),
        };
        match winners.get(&key) {
            Some((_, held)) if !replaces(held.2, row.2) => {}
            _ => {
                winners.insert(key, (ordinal, *row));
            }
        }
    }
    let mut ordered: Vec<_> = winners.into_values().collect();
    ordered.sort_by_key(|(ordinal, _)| *ordinal);
    ordered
}

fn identity(expect: Expect, declaration: &CssDeclaration) -> bool {
    match (expect, declaration.property_name()) {
        (Expect::Known(name), CssPropertyNameRef::Known(property)) => {
            CssKnownProperty::from_name(name) == Some(property)
        }
        (Expect::Custom(name), CssPropertyNameRef::Custom(custom)) => custom.as_str() == name,
        _ => false,
    }
}

fn importance(declaration: &CssDeclaration) -> bool {
    declaration.importance() == CssImportance::Important
}

fn declarations_match(what: &str, expected: &[DeclRow], got: &[&CssDeclaration]) -> Check {
    if expected.len() != got.len() {
        return Err(Violation::new(
            RECOVERY,
            "valid-declarations-not-recovered",
            format!(
                "{what} retained {} declarations, expected {}",
                got.len(),
                expected.len()
            ),
        ));
    }
    for (index, (row, declaration)) in expected.iter().zip(got).enumerate() {
        if !identity(row.1, declaration) || importance(declaration) != row.2 {
            return Err(Violation::new(
                RECOVERY,
                "declaration-identity-differs",
                format!("{what} declaration {index} is not {:?}", row.0),
            ));
        }
    }
    Ok(())
}

/// The crate's own current-block selection, against the independent winners.
fn selection(
    what: &str,
    expected_rows: &[DeclRow],
    list: &CssDeclarationList,
    replaces: fn(bool, bool) -> bool,
) -> Check {
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(
        list,
        CssSerializationLimits::default(),
    )
    .map_err(|cause| Violation::new(SELECTION, "selection-refused", format!("{what}: {cause}")))?;
    let winners = expected_winners(expected_rows, replaces);
    let entries = block.entries();
    let same = entries.len() == winners.len()
        && entries.iter().zip(&winners).all(|(entry, (ordinal, row))| {
            entry.authored_ordinal() == *ordinal
                && (entry.importance() == CssImportance::Important) == row.2
                && match (row.1, entry.property_name()) {
                    (Expect::Known(name), CssPropertyNameRef::Known(property)) => {
                        CssKnownProperty::from_name(name) == Some(property)
                    }
                    (Expect::Custom(name), CssPropertyNameRef::Custom(custom)) => {
                        custom.as_str() == name
                    }
                    _ => false,
                }
        });
    if !same {
        return Err(Violation::new(
            SELECTION,
            "winner-differs",
            format!(
                "{what}: selection kept {} entries at {:?}, expected {} at {:?}",
                entries.len(),
                entries
                    .iter()
                    .map(|e| e.authored_ordinal())
                    .collect::<Vec<_>>(),
                winners.len(),
                winners
                    .iter()
                    .map(|(ordinal, _)| *ordinal)
                    .collect::<Vec<_>>(),
            ),
        ));
    }
    Ok(())
}

fn style_matches(
    what: &str,
    expected: &ExpStyle,
    got: &CssStyleRule,
    replaces: fn(bool, bool) -> bool,
) -> Check {
    let leading = got.declarations().len();
    let mut all: Vec<&CssDeclaration> = got.declarations().iter().collect();
    let mut nested: Vec<&CssStyleRule> = Vec::new();
    for rule in got.rules() {
        match rule {
            CssRule::Style(child) => nested.push(child),
            CssRule::NestedDeclarations(run) => all.extend(run.declarations().iter()),
            _ => {
                return Err(Violation::new(
                    STRUCTURE,
                    "unexpected-child-rule",
                    format!("{what} retained a child rule of an unexpected kind"),
                ));
            }
        }
    }
    declarations_match(what, &expected.decls, &all)?;
    // The rule's own declarations are a prefix of the source-order list, so
    // their independent winners come from the same prefix of the expectation.
    selection(
        what,
        &expected.decls[..leading],
        got.declarations(),
        replaces,
    )?;
    if nested.len() != expected.nested.len() {
        return Err(Violation::new(
            STRUCTURE,
            "nested-rule-count-differs",
            format!(
                "{what} retained {} nested rules, expected {}",
                nested.len(),
                expected.nested.len()
            ),
        ));
    }
    for (index, (want, child)) in expected.nested.iter().zip(nested).enumerate() {
        style_matches(&format!("{what}/{index}"), want, child, replaces)?;
    }
    Ok(())
}

fn rules_match(
    what: &str,
    expected: &[ExpRule],
    got: &[CssRule],
    replaces: fn(bool, bool) -> bool,
) -> Check {
    if expected.len() != got.len() {
        return Err(Violation::new(
            RECOVERY,
            "valid-rules-not-recovered",
            format!(
                "{what} retained {} rules, expected {}",
                got.len(),
                expected.len()
            ),
        ));
    }
    for (index, (want, rule)) in expected.iter().zip(got).enumerate() {
        let here = format!("{what}/{index}");
        match (want, rule) {
            (ExpRule::Style(style), CssRule::Style(rule)) => {
                style_matches(&here, style, rule, replaces)?;
            }
            (ExpRule::Group(Group::Media, children), CssRule::Media(rule)) => {
                rules_match(&here, children, rule.rules(), replaces)?;
            }
            (ExpRule::Group(Group::Supports, children), CssRule::Supports(rule)) => {
                rules_match(&here, children, rule.rules(), replaces)?;
            }
            (ExpRule::Group(Group::Layer, children), CssRule::LayerBlock(rule)) => {
                rules_match(&here, children, rule.rules(), replaces)?;
            }
            (ExpRule::Group(Group::Container, children), CssRule::Container(rule)) => {
                rules_match(&here, children, rule.rules(), replaces)?;
            }
            _ => {
                return Err(Violation::new(
                    STRUCTURE,
                    "rule-kind-differs",
                    format!("{here} is not the expected rule kind"),
                ));
            }
        }
    }
    Ok(())
}

/// Diagnostics only inside injected units, and at least one in each. Valid text
/// has no diagnostic, so one outside every injected range is a false report.
fn locality(
    text: &str,
    bad: &[(usize, usize)],
    diagnostics: &[surgeist_css::diagnostic::CssRecoveryDiagnostic],
) -> Check {
    for diagnostic in diagnostics {
        let at = diagnostic.error().position().byte_offset().value();
        let span = (
            diagnostic.span().start().byte_offset().value(),
            diagnostic.span().end().byte_offset().value(),
        );
        if !bad
            .iter()
            .any(|(start, end)| at >= *start && at <= *end && span.0 >= *start && span.1 <= *end)
        {
            return Err(Violation::new(
                LOCALITY,
                "diagnostic-outside-injected-units",
                format!(
                    "a diagnostic at {at} (span {span:?}) lies outside every injected unit of {} bytes",
                    text.len()
                ),
            ));
        }
    }
    for (start, end) in bad {
        let reported = diagnostics.iter().any(|diagnostic| {
            let at = diagnostic.error().position().byte_offset().value();
            at >= *start && at <= *end
        });
        if !reported {
            return Err(Violation::new(
                LOCALITY,
                "injected-unit-not-reported",
                format!("the invalid unit at {start}..{end} drew no diagnostic"),
            ));
        }
    }
    Ok(())
}

/// Independent oracles, replaceable in tests to plant a controlled failure.
#[derive(Clone, Copy)]
pub(crate) struct Oracle {
    pub(crate) checker: Checker,
    pub(crate) rows: &'static [DeclRow],
    pub(crate) replaces: fn(bool, bool) -> bool,
}

/// An important declaration beats a normal one; otherwise the later one wins.
fn later_or_important_wins(held_important: bool, new_important: bool) -> bool {
    !(held_important && !new_important)
}

pub(crate) const INDEPENDENT: Oracle = Oracle {
    checker: check::DOCUMENTED,
    rows: &DECLS,
    replaces: later_or_important_wins,
};

fn observe(oracle: &Oracle, case: &RulesCase) -> std::result::Result<(), Violation> {
    let rendered = render(case, oracle.rows);
    let text = rendered.text.as_str();
    let parser = CssParser::default();
    match case.front {
        Front::Sheet => {
            let report = parser.parse_sheet(text);
            oracle.checker.diagnostics(text, report.diagnostics())?;
            check::cleanliness(&report)?;
            nodes::sheet(&oracle.checker, text, report.syntax())?;
            sheet_structure(oracle, &rendered, report.syntax())?;
            locality(text, &rendered.bad, report.diagnostics())?;
            clean_iff_nothing_injected(&rendered, report.is_clean())
        }
        Front::Block => {
            let report = parser.parse_block_contents(text);
            oracle.checker.diagnostics(text, report.diagnostics())?;
            check::cleanliness(&report)?;
            nodes::declarations(&oracle.checker, text, report.syntax().as_slice(), "block")?;
            let retained: Vec<&CssDeclaration> = report.syntax().iter().collect();
            let expected = rendered.block.decls.clone();
            declarations_match("block contents", &expected, &retained)?;
            selection(
                "block contents",
                &expected,
                report.syntax(),
                oracle.replaces,
            )?;
            locality(text, &rendered.bad, report.diagnostics())?;
            clean_iff_nothing_injected(&rendered, report.is_clean())
        }
    }
}

fn sheet_structure(oracle: &Oracle, rendered: &Rendered, sheet: &CssSheet) -> Check {
    rules_match("sheet", &rendered.rules, sheet.rules(), oracle.replaces)
}

fn clean_iff_nothing_injected(rendered: &Rendered, clean: bool) -> Check {
    if clean == rendered.bad.is_empty() {
        return Ok(());
    }
    Err(Violation::new(
        check::CLEANLINESS,
        if clean {
            "injected-invalid-unit-left-a-clean-report"
        } else {
            "valid-source-reported-diagnostics"
        },
        format!(
            "the report is {} with {} injected invalid units",
            if clean { "clean" } else { "not clean" },
            rendered.bad.len()
        ),
    ))
}

/// The target: one evaluator shared by workers, properties, coverage and replay.
pub struct RulesTarget {
    descriptor: TargetDescriptor,
    oracle: Oracle,
}

impl Default for RulesTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl RulesTarget {
    /// The number of source-qualified literal cases.
    pub const LITERALS: u8 = 12;

    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptor: descriptor("surgeist-css::structured-rules"),
            oracle: INDEPENDENT,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_oracle(oracle: Oracle) -> Self {
        Self {
            oracle,
            ..Self::new()
        }
    }

    /// Source-qualified literal recipes, each isolating one recovery contract.
    pub fn literal(&self, id: u8) -> Result<RulesCase> {
        literal(id)
    }
}

fn decl(row: u8) -> Item {
    Item::Decl { row, sep: 1 }
}

fn bad(row: u8) -> Item {
    Item::Bad { row, sep: 1 }
}

fn style(selector: u8, items: Vec<Item>) -> Node {
    Node::Style {
        style: Style { selector, items },
        sep: 2,
    }
}

fn literal(id: u8) -> Result<RulesCase> {
    let nodes = match id {
        // A valid sheet: nothing invalid, so the report is clean.
        0 => vec![
            style(0, vec![decl(0), decl(3), decl(12)]),
            Node::Group {
                group: 0,
                children: vec![style(1, vec![decl(9)])],
                sep: 2,
            },
        ],
        // An unknown at-rule between valid rules.
        1 => vec![
            style(0, vec![decl(0)]),
            Node::Skip { row: 0, sep: 2 },
            style(1, vec![decl(3)]),
        ],
        // Invalid declarations before, between and after valid siblings.
        2 => vec![style(
            0,
            vec![bad(0), decl(0), bad(1), decl(3), bad(2), decl(9), bad(3)],
        )],
        // Duplicated names: important wins; later wins at equal importance.
        3 => vec![style(
            0,
            vec![decl(3), decl(0), decl(4), decl(5), decl(1), decl(2)],
        )],
        // Escaped, uppercase and Unicode custom names.
        4 => vec![style(
            0,
            vec![decl(5), decl(1), decl(14), decl(15), decl(13), decl(12)],
        )],
        // Strings and comments hiding punctuation inside custom values.
        5 => vec![style(
            2,
            vec![
                decl(16),
                Item::Comment { row: 1, sep: 1 },
                decl(17),
                decl(18),
                decl(19),
            ],
        )],
        // Nested rules with later declaration runs and an invalid unit inside.
        6 => vec![style(
            0,
            vec![
                decl(0),
                Item::Nest {
                    style: Style {
                        selector: 0,
                        items: vec![decl(3), bad(5)],
                    },
                    sep: 1,
                },
                decl(9),
                decl(10),
                Item::Nest {
                    style: Style {
                        selector: 1,
                        items: vec![decl(7)],
                    },
                    sep: 1,
                },
            ],
        )],
        // Conditional groups of every kind, an invalid rule inside one.
        7 => vec![
            Node::Group {
                group: 2,
                children: vec![
                    style(0, vec![decl(0)]),
                    Node::Skip { row: 2, sep: 1 },
                    Node::Group {
                        group: 3,
                        children: vec![style(1, vec![decl(3)])],
                        sep: 1,
                    },
                ],
                sep: 2,
            },
            Node::Group {
                group: 4,
                children: vec![style(3, vec![decl(9)])],
                sep: 2,
            },
            Node::Group {
                group: 5,
                children: vec![style(4, vec![decl(7)])],
                sep: 2,
            },
        ],
        // Numeric extremes and deferred substitution.
        8 => vec![style(
            0,
            vec![decl(7), decl(8), decl(10), decl(11), decl(19)],
        )],
        // Ignorable text, comments and every line ending between units.
        9 => vec![
            Node::Ignorable { row: 1, sep: 3 },
            style(5, vec![decl(0)]),
            Node::Ignorable { row: 2, sep: 4 },
            style(6, vec![decl(3)]),
            Node::Ignorable { row: 3, sep: 5 },
            Node::Skip { row: 3, sep: 6 },
            style(7, vec![decl(9)]),
        ],
        // A style attribute: nested rules are dropped and reported, declarations stay.
        10 => {
            return RulesCase::new(
                Front::Block,
                vec![style(
                    0,
                    vec![
                        decl(0),
                        Item::Nest {
                            style: Style {
                                selector: 0,
                                items: vec![decl(3)],
                            },
                            sep: 1,
                        },
                        decl(4),
                        bad(6),
                        decl(12),
                    ],
                )],
            );
        }
        // A style attribute with duplicated names and every kind of invalid unit.
        11 => {
            return RulesCase::new(
                Front::Block,
                vec![style(
                    0,
                    vec![
                        decl(3),
                        bad(9),
                        decl(4),
                        decl(5),
                        bad(11),
                        decl(12),
                        decl(13),
                        decl(12),
                    ],
                )],
            );
        }
        _ => return Err(error("unknown literal")),
    };
    RulesCase::new(Front::Sheet, nodes)
}

fn put_style(writer: &mut Writer, style: &Style, depth: usize) -> Result<()> {
    writer.u8(style.selector)?;
    writer.count(style.items.len(), MAX_ITEMS)?;
    for item in &style.items {
        match item {
            Item::Decl { row, sep } => {
                writer.u8(0)?;
                writer.u8(*row)?;
                writer.u8(*sep)?;
            }
            Item::Bad { row, sep } => {
                writer.u8(1)?;
                writer.u8(*row)?;
                writer.u8(*sep)?;
            }
            Item::Comment { row, sep } => {
                writer.u8(2)?;
                writer.u8(*row)?;
                writer.u8(*sep)?;
            }
            Item::Nest { style, sep } => {
                if depth >= MAX_DEPTH {
                    return Err(error("nesting ceiling"));
                }
                writer.u8(3)?;
                writer.u8(*sep)?;
                put_style(writer, style, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn put_nodes(writer: &mut Writer, nodes: &[Node], depth: usize) -> Result<()> {
    writer.count(nodes.len(), MAX_NODES)?;
    for node in nodes {
        match node {
            Node::Style { style, sep } => {
                writer.u8(0)?;
                writer.u8(*sep)?;
                put_style(writer, style, depth + 1)?;
            }
            Node::Group {
                group,
                children,
                sep,
            } => {
                if depth >= MAX_DEPTH {
                    return Err(error("nesting ceiling"));
                }
                writer.u8(1)?;
                writer.u8(*group)?;
                writer.u8(*sep)?;
                put_nodes(writer, children, depth + 1)?;
            }
            Node::Skip { row, sep } => {
                writer.u8(2)?;
                writer.u8(*row)?;
                writer.u8(*sep)?;
            }
            Node::Ignorable { row, sep } => {
                writer.u8(3)?;
                writer.u8(*row)?;
                writer.u8(*sep)?;
            }
        }
    }
    Ok(())
}

/// Reads with a shared node budget so declared counts are bounded before use.
struct Decoder<'a> {
    reader: Reader<'a>,
    remaining: usize,
}

impl Decoder<'_> {
    fn spend(&mut self) -> Result<()> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| error("node ceiling"))?;
        Ok(())
    }

    fn style(&mut self, depth: usize) -> Result<Style> {
        let selector = self.reader.u8()?;
        let count = self.reader.count(MAX_ITEMS)?;
        let mut items = Vec::new();
        items
            .try_reserve_exact(count)
            .map_err(|_| error("item allocation"))?;
        for _ in 0..count {
            self.spend()?;
            let item = match self.reader.u8()? {
                0 => Item::Decl {
                    row: self.reader.u8()?,
                    sep: self.reader.u8()?,
                },
                1 => Item::Bad {
                    row: self.reader.u8()?,
                    sep: self.reader.u8()?,
                },
                2 => Item::Comment {
                    row: self.reader.u8()?,
                    sep: self.reader.u8()?,
                },
                3 => {
                    if depth >= MAX_DEPTH {
                        return Err(error("nesting ceiling"));
                    }
                    let sep = self.reader.u8()?;
                    Item::Nest {
                        style: self.style(depth + 1)?,
                        sep,
                    }
                }
                _ => return Err(error("item tag")),
            };
            items.push(item);
        }
        Ok(Style { selector, items })
    }

    fn nodes(&mut self, depth: usize) -> Result<Vec<Node>> {
        let count = self.reader.count(MAX_NODES)?;
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(count)
            .map_err(|_| error("node allocation"))?;
        for _ in 0..count {
            self.spend()?;
            let node = match self.reader.u8()? {
                0 => {
                    let sep = self.reader.u8()?;
                    Node::Style {
                        style: self.style(depth + 1)?,
                        sep,
                    }
                }
                1 => {
                    if depth >= MAX_DEPTH {
                        return Err(error("nesting ceiling"));
                    }
                    let group = self.reader.u8()?;
                    let sep = self.reader.u8()?;
                    Node::Group {
                        group,
                        children: self.nodes(depth + 1)?,
                        sep,
                    }
                }
                2 => Node::Skip {
                    row: self.reader.u8()?,
                    sep: self.reader.u8()?,
                },
                3 => Node::Ignorable {
                    row: self.reader.u8()?,
                    sep: self.reader.u8()?,
                },
                _ => return Err(error("node tag")),
            };
            nodes.push(node);
        }
        Ok(nodes)
    }
}

impl CaseTarget for RulesTarget {
    type Case = RulesCase;

    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }

    fn decode(&self, input: &[u8]) -> Result<RulesCase> {
        let mut reader = Reader::new(input, MAGIC, MAX_CASE)?;
        let front = match reader.u8()? {
            0 => Front::Sheet,
            1 => Front::Block,
            _ => return Err(error("front tag")),
        };
        let mut decoder = Decoder {
            reader,
            remaining: MAX_NODES,
        };
        let nodes = decoder.nodes(0)?;
        decoder.reader.finish()?;
        RulesCase::new(front, nodes)
    }

    fn encode(&self, case: &RulesCase) -> Result<Vec<u8>> {
        let mut writer = Writer::new(MAGIC, MAX_CASE)?;
        writer.u8(match case.front {
            Front::Sheet => 0,
            Front::Block => 1,
        })?;
        put_nodes(&mut writer, &case.nodes, 0)?;
        Ok(writer.finish())
    }

    fn generate(&self, input: &[u8]) -> Result<Generation<RulesCase>> {
        generate(&self.descriptor, input).map(Generation::Generated)
    }

    fn evaluate(&self, case: &RulesCase) -> Result<CaseOutcome> {
        match observe(&self.oracle, case) {
            Err(found) => violation(&found),
            Ok(()) => evaluated(&[
                RECOVERY,
                LOCALITY,
                SELECTION,
                STRUCTURE,
                check::COORDINATES,
                check::CLEANLINESS,
                nodes::RETAINED,
            ]),
        }
    }

    fn reduction_candidates(
        &self,
        case: &RulesCase,
        emit: &mut dyn FnMut(RulesCase) -> bool,
    ) -> Result<()> {
        // Drop one top-level node, or one item of the first style rule; each
        // candidate is revalidated by the checked constructor.
        let mut candidates: Vec<Vec<Node>> = Vec::new();
        for index in 0..case.nodes.len() {
            let mut fewer = case.nodes.clone();
            fewer.remove(index);
            candidates.push(fewer);
        }
        for (index, node) in case.nodes.iter().enumerate() {
            if let Node::Style { style, sep } = node {
                for item in 0..style.items.len() {
                    let mut fewer = case.nodes.clone();
                    let mut smaller = style.clone();
                    smaller.items.remove(item);
                    fewer[index] = Node::Style {
                        style: smaller,
                        sep: *sep,
                    };
                    candidates.push(fewer);
                }
            }
        }
        for nodes in candidates {
            if !emit(RulesCase::new(case.front, nodes)?) {
                break;
            }
        }
        Ok(())
    }
}

fn sep(entropy: &mut Cursor<'_>) -> u8 {
    entropy.below(SEPARATORS.len()) as u8
}

/// Spends one unit of the node allowance; false once it is exhausted.
fn spend(budget: &mut usize) -> bool {
    match budget.checked_sub(1) {
        Some(left) => {
            *budget = left;
            true
        }
        None => false,
    }
}

fn generate_style(
    entropy: &mut Cursor<'_>,
    selectors: usize,
    depth: usize,
    budget: &mut usize,
) -> Style {
    let mut items = Vec::new();
    for _ in 0..entropy.below(MAX_ITEMS.min(8) + 1) {
        if !spend(budget) {
            break;
        }
        let item = match entropy.below(12) {
            0..=5 => Item::Decl {
                row: entropy.below(DECLS.len()) as u8,
                sep: sep(entropy),
            },
            6..=8 => Item::Bad {
                row: entropy.below(BAD.len()) as u8,
                sep: sep(entropy),
            },
            9 => Item::Comment {
                row: entropy.below(COMMENTS.len()) as u8,
                sep: sep(entropy),
            },
            _ if depth + 2 < MAX_DEPTH => Item::Nest {
                style: generate_style(entropy, NESTED.len(), depth + 1, budget),
                sep: sep(entropy),
            },
            _ => Item::Decl {
                row: entropy.below(DECLS.len()) as u8,
                sep: sep(entropy),
            },
        };
        items.push(item);
    }
    Style {
        selector: entropy.below(selectors) as u8,
        items,
    }
}

fn generate_nodes(entropy: &mut Cursor<'_>, depth: usize, budget: &mut usize) -> Vec<Node> {
    let mut nodes = Vec::new();
    for _ in 0..entropy.below(6) {
        if !spend(budget) {
            break;
        }
        let node = match entropy.below(10) {
            0..=4 => Node::Style {
                style: generate_style(entropy, SELECTORS.len(), depth, budget),
                sep: sep(entropy),
            },
            5..=6 if depth + 2 < MAX_DEPTH => Node::Group {
                group: entropy.below(GROUPS.len()) as u8,
                children: generate_nodes(entropy, depth + 1, budget),
                sep: sep(entropy),
            },
            7 => Node::Skip {
                row: entropy.below(SKIPS.len()) as u8,
                sep: sep(entropy),
            },
            _ => Node::Ignorable {
                row: entropy.below(IGNORABLE.len()) as u8,
                sep: sep(entropy),
            },
        };
        nodes.push(node);
    }
    nodes
}

fn generate(descriptor: &TargetDescriptor, input: &[u8]) -> Result<RulesCase> {
    let mut entropy = Cursor::new(input);
    let front = if entropy.below(4) == 0 {
        Front::Block
    } else {
        Front::Sheet
    };
    let mut budget = MAX_NODES;
    let nodes = match front {
        Front::Sheet => generate_nodes(&mut entropy, 0, &mut budget),
        Front::Block => {
            // The style node itself spends one unit of the allowance.
            spend(&mut budget);
            let style = generate_style(&mut entropy, SELECTORS.len(), 0, &mut budget);
            vec![Node::Style { style, sep: 0 }]
        }
    };
    let case = RulesCase::new(front, nodes)?;
    let rendered = render(&case, &DECLS);
    let mut accounting = Budget::new(descriptor.limits());
    accounting.credit(BudgetCharge {
        bytes: rendered.text.len() as u64 + 256,
        nodes: case.nodes.iter().map(count_node).sum::<usize>().max(1) as u64,
        depth: case.nodes.iter().map(depth_node).max().unwrap_or(0).max(1) as u64,
        text_units: rendered.text.len() as u64,
        operations: 4,
        ..BudgetCharge::default()
    })?;
    Ok(case)
}
