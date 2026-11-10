//! Replay of every retained html5lib-tests `f994590f` tree-construction case.
//!
//! Each bundled `.dat` case runs through the actual parser and the metered
//! native comparison owner against its independently decoded corpus tree:
//! ordinary documents through [`check`], and every `#document-fragment` case
//! through the real captured-context fragment owner with its corpus context,
//! including SVG and MathML contexts, and then against the tree the public
//! fragment request returns, rendered in the corpus dump shape. Cases without
//! a scripting marker run with Disabled scripting.
//!
//! The eight `#script-on` cases are all ordinary documents and are excluded
//! (see [`SCRIPT_ON_REASON`]). The scripting-mode `noscript` rules they
//! exercise are covered through the fragment API, whose Inert mode is not
//! Disabled: `tests/parser/parse/frameset_noscript.rs`.
//!
//! The corpus format does not carry document mode. A case binds its mode from
//! the hand-derived HTML877d table below, otherwise from the Initial rules the
//! corpus tree itself establishes: no doctype record selects Quirks
//! (Initial anything-else), and a plain `<!DOCTYPE html>` that both begins the
//! input and is the first expected record selects NoQuirks. Every other
//! retained doctype case is in the table, which the replay asserts, so no
//! retained case is compared with its document mode omitted.

use super::comment_diagnostic_corpus::template_corpus::{
    check_fragment_observation, fragment_request, mode as template_mode,
};
use super::*;
use crate::{HtmlDocumentParseRequest, HtmlSource};
use surgeist_html::construction::{HtmlFragment, HtmlNode};
use surgeist_html::{
    construction::{HtmlDocument, HtmlDocumentMode, HtmlNodeId, HtmlNodeKind},
    values::{DocumentString, QualifiedName},
};

/// Why one retained case is not asserted to match.
#[derive(Clone, Copy, Debug)]
enum Exclusion {
    /// The pinned HTML edition defines a different tree. The case must still
    /// diverge from the corpus, and its actual rendering in both encodings
    /// must equal the tree derived from the pinned edition, so a stale record,
    /// a failed parse or any unrelated divergence fails.
    Spec(&'static str, &'static str),
}

/// Exclusions keyed by bundled file and one-based case ordinal.
///
/// HTML877d tag open state: `<?` enters the processing-instruction states
/// instead of the historical bogus comment the corpus records. A target runs
/// to whitespace, `?` or `>`; whitespace after it is skipped; data runs to `>`,
/// with `?>` also ending it. Initial inserts the PI into the Document and then
/// reaches EOF, which is Quirks with html, head and body. At EOF inside the
/// PI states only end-of-file is emitted.
const PI_REASON: &str = "HTML877d tag open state: U+003F enters processing-instruction open state";
/// HTML877d in body, "A start tag whose tag name is "input"": with a select
/// fragment context element it is a parse error and the token is ignored; the
/// corpus revision still inserts it.
const SELECT_INPUT_REASON: &str =
    "HTML877d in body: an input start tag in a select fragment context is ignored";

/// Why the `#script-on` cases are not run. HTML877d "other parsing state
/// flags" (`references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md`
/// line 1347) gives a Document parser the scripting mode Normal when scripting
/// is enabled for its Document, and Normal means "Scripts are processed when
/// inserted" (line 1353 onward). The public document profile is Disabled only:
/// `HtmlDocumentParseRequest::scripting_mode` is fixed Disabled,
/// `HtmlDocumentParseRequest::captured` rejects an Enabled document with
/// `UnsupportedExecution(Normal)`, and `docs/reference/html-operations.md`
/// ("The document execution profile is a fresh ordinary document with
/// Disabled scripting") records that boundary. No public or private entry
/// constructs a Normal document parse to replay them against.
const SCRIPT_ON_REASON: &str = "Document scripting mode Normal requires script processing, which the document profile does not support";

const EXCLUSIONS: &[(&str, usize, Exclusion)] = &[
    (
        "html5test-com.dat",
        12,
        Exclusion::Spec(
            PI_REASON,
            "(mode Quirks)\n| <?import namespace=\"foo\" implementation=\"#bar\">\n| <html>\n|   <head>\n|   <body>\n",
        ),
    ),
    (
        "tests1.dat",
        40,
        Exclusion::Spec(
            PI_REASON,
            "(mode Quirks)\n| <html>\n|   <head>\n|   <body>\n",
        ),
    ),
    (
        "tests1.dat",
        44,
        Exclusion::Spec(
            PI_REASON,
            "(mode Quirks)\n| <?COMMENT >\n| <html>\n|   <head>\n|   <body>\n",
        ),
    ),
    (
        "tests1.dat",
        47,
        Exclusion::Spec(
            PI_REASON,
            "(mode Quirks)\n| <?COM--MENT >\n| <html>\n|   <head>\n|   <body>\n",
        ),
    ),
    (
        "tests_innerHTML_1.dat",
        76,
        Exclusion::Spec(SELECT_INPUT_REASON, "| <option>\n"),
    ),
];

/// Document modes derived by hand from the HTML877d Initial rules for the
/// previously selected bindings, never from actual output. Template.dat uses
/// the template corpus table.
const HAND_DERIVED_MODES: &[(&str, usize, HtmlDocumentMode)] = &[
    ("adoption02.dat", 1, HtmlDocumentMode::Quirks),
    ("namespace-sensitivity.dat", 1, HtmlDocumentMode::Quirks),
    ("quirks01.dat", 1, HtmlDocumentMode::LimitedQuirks),
    ("quirks01.dat", 2, HtmlDocumentMode::Quirks),
    ("quirks01.dat", 3, HtmlDocumentMode::Quirks),
    ("quirks01.dat", 4, HtmlDocumentMode::Quirks),
    ("tests14.dat", 1, HtmlDocumentMode::NoQuirks),
    ("tests14.dat", 2, HtmlDocumentMode::NoQuirks),
    ("tests14.dat", 3, HtmlDocumentMode::NoQuirks),
    ("tests14.dat", 4, HtmlDocumentMode::NoQuirks),
    ("tests14.dat", 5, HtmlDocumentMode::NoQuirks),
    ("tests14.dat", 6, HtmlDocumentMode::NoQuirks),
    ("tests14.dat", 7, HtmlDocumentMode::NoQuirks),
    ("tests24.dat", 1, HtmlDocumentMode::NoQuirks),
    ("tests24.dat", 2, HtmlDocumentMode::NoQuirks),
    ("tests24.dat", 3, HtmlDocumentMode::NoQuirks),
    ("tests24.dat", 4, HtmlDocumentMode::NoQuirks),
    ("tests24.dat", 5, HtmlDocumentMode::NoQuirks),
    ("tests24.dat", 6, HtmlDocumentMode::NoQuirks),
    ("tests24.dat", 7, HtmlDocumentMode::NoQuirks),
    ("tests24.dat", 8, HtmlDocumentMode::NoQuirks),
    ("tables01.dat", 1, HtmlDocumentMode::Quirks),
    ("tables01.dat", 2, HtmlDocumentMode::Quirks),
    ("tables01.dat", 3, HtmlDocumentMode::Quirks),
    ("tables01.dat", 4, HtmlDocumentMode::Quirks),
    ("tables01.dat", 5, HtmlDocumentMode::Quirks),
    ("tables01.dat", 6, HtmlDocumentMode::Quirks),
    ("tables01.dat", 7, HtmlDocumentMode::Quirks),
    ("tables01.dat", 8, HtmlDocumentMode::Quirks),
    ("tables01.dat", 9, HtmlDocumentMode::Quirks),
    ("tables01.dat", 10, HtmlDocumentMode::Quirks),
    ("tables01.dat", 11, HtmlDocumentMode::Quirks),
    ("tables01.dat", 12, HtmlDocumentMode::Quirks),
    ("tables01.dat", 13, HtmlDocumentMode::Quirks),
    ("tables01.dat", 14, HtmlDocumentMode::Quirks),
    ("tables01.dat", 15, HtmlDocumentMode::Quirks),
    ("tables01.dat", 16, HtmlDocumentMode::Quirks),
    ("tables01.dat", 17, HtmlDocumentMode::Quirks),
    ("tables01.dat", 18, HtmlDocumentMode::Quirks),
    ("tables01.dat", 19, HtmlDocumentMode::Quirks),
    ("comments01.dat", 1, HtmlDocumentMode::Quirks),
    ("comments01.dat", 2, HtmlDocumentMode::Quirks),
    ("comments01.dat", 3, HtmlDocumentMode::Quirks),
    ("comments01.dat", 4, HtmlDocumentMode::Quirks),
    ("comments01.dat", 5, HtmlDocumentMode::Quirks),
    ("comments01.dat", 6, HtmlDocumentMode::Quirks),
    ("comments01.dat", 7, HtmlDocumentMode::Quirks),
    ("comments01.dat", 8, HtmlDocumentMode::Quirks),
    ("comments01.dat", 9, HtmlDocumentMode::Quirks),
    ("comments01.dat", 10, HtmlDocumentMode::Quirks),
    ("comments01.dat", 11, HtmlDocumentMode::Quirks),
    // Reserved xml is bogus-comment recovery even in the newer PI edition.
    ("comments01.dat", 12, HtmlDocumentMode::Quirks),
    ("comments01.dat", 13, HtmlDocumentMode::Quirks),
    ("comments01.dat", 14, HtmlDocumentMode::Quirks),
    ("comments01.dat", 15, HtmlDocumentMode::Quirks),
    ("comments01.dat", 16, HtmlDocumentMode::Quirks),
    // Initial: a missing DOCTYPE name sets the force-quirks flag.
    ("doctype01.dat", 4, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 5, HtmlDocumentMode::Quirks),
    ("tests6.dat", 5, HtmlDocumentMode::Quirks),
    // Initial: a name other than "html" selects quirks mode.
    ("doctype01.dat", 6, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 7, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 8, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 9, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 10, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 11, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 12, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 13, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 14, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 15, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 16, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 17, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 18, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 19, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 20, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 21, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 22, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 23, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 24, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 25, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 26, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 28, HtmlDocumentMode::Quirks),
    ("doctype01.dat", 31, HtmlDocumentMode::Quirks),
    ("tests2.dat", 62, HtmlDocumentMode::Quirks),
    // Initial: an XHTML 1.0 Frameset or Transitional public identifier selects
    // limited-quirks mode.
    ("doctype01.dat", 29, HtmlDocumentMode::LimitedQuirks),
    ("doctype01.dat", 30, HtmlDocumentMode::LimitedQuirks),
    // Initial: name "html" with no listed identifier is no-quirks mode. The
    // missing-whitespace errors before the name and around the identifiers do
    // not set the force-quirks flag.
    ("doctype01.dat", 3, HtmlDocumentMode::NoQuirks),
    ("doctype01.dat", 27, HtmlDocumentMode::NoQuirks),
    ("doctype01.dat", 32, HtmlDocumentMode::NoQuirks),
    ("doctype01.dat", 33, HtmlDocumentMode::NoQuirks),
    ("doctype01.dat", 34, HtmlDocumentMode::NoQuirks),
    ("doctype01.dat", 35, HtmlDocumentMode::NoQuirks),
    ("doctype01.dat", 36, HtmlDocumentMode::NoQuirks),
    ("doctype01.dat", 37, HtmlDocumentMode::NoQuirks),
    ("tests2.dat", 19, HtmlDocumentMode::NoQuirks),
    ("tests2.dat", 26, HtmlDocumentMode::NoQuirks),
    ("tests2.dat", 27, HtmlDocumentMode::NoQuirks),
    ("tests2.dat", 28, HtmlDocumentMode::NoQuirks),
    ("tests2.dat", 29, HtmlDocumentMode::NoQuirks),
    ("tests6.dat", 47, HtmlDocumentMode::NoQuirks),
];

/// How a retained ordinary case's document mode is bound.
fn document_mode(file: &str, fixture: &Fixture<'_>) -> Option<HtmlDocumentMode> {
    if let Some((_, _, mode)) = HAND_DERIVED_MODES
        .iter()
        .find(|(f, o, _)| *f == file && *o == fixture.ordinal)
    {
        return Some(*mode);
    }
    if file == "template.dat" {
        return Some(template_mode(fixture.ordinal));
    }
    let doctype = fixture
        .expected
        .split_terminator('\n')
        .find(|line| line.starts_with("| <!DOCTYPE "));
    match doctype {
        None => Some(HtmlDocumentMode::Quirks),
        Some(_)
            if fixture.expected.starts_with("| <!DOCTYPE html>\n")
                && fixture
                    .input
                    .as_bytes()
                    .get(..15)
                    .is_some_and(|start| start.eq_ignore_ascii_case(b"<!DOCTYPE html>")) =>
        {
            Some(HtmlDocumentMode::NoQuirks)
        }
        Some(_) => None,
    }
}

fn source(input: &str, utf16: bool) -> HtmlSource {
    if utf16 {
        HtmlSource::utf16(DocumentString::from(input))
    } else {
        HtmlSource::utf8(input)
    }
}

fn document_request(input: &str, utf16: bool) -> HtmlDocumentParseRequest {
    HtmlDocumentParseRequest::offline(source(input, utf16), super::super::tests::limits(1_000))
}

/// Rendering of an actual public result in the corpus dump shape. Failure
/// messages use it, spec exclusions compare it exactly with the tree the
/// pinned edition requires, and fragment cases compare it with the corpus dump
/// in addition to the native observer.
trait Tree {
    fn top(&self) -> HtmlNode<'_>;
    fn lookup(&self, id: &HtmlNodeId) -> Option<HtmlNode<'_>>;
}
impl Tree for HtmlDocument {
    fn top(&self) -> HtmlNode<'_> {
        self.root()
    }
    fn lookup(&self, id: &HtmlNodeId) -> Option<HtmlNode<'_>> {
        self.node(id).ok()
    }
}
impl Tree for HtmlFragment {
    fn top(&self) -> HtmlNode<'_> {
        self.root()
    }
    fn lookup(&self, id: &HtmlNodeId) -> Option<HtmlNode<'_>> {
        self.node(id).ok()
    }
}
fn text(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}
fn dump_name(name: &QualifiedName, element: bool) -> String {
    let local = text(name.local_name().code_units());
    let uri = name.namespace().uri().map(|uri| text(uri.code_units()));
    let designator = match uri.as_deref() {
        None => return local,
        Some("http://www.w3.org/1999/xhtml") if element => return local,
        Some("http://www.w3.org/2000/svg") => "svg",
        Some("http://www.w3.org/1998/Math/MathML") => "math",
        Some("http://www.w3.org/1999/xlink") => "xlink",
        Some("http://www.w3.org/XML/1998/namespace") => "xml",
        Some("http://www.w3.org/2000/xmlns/") => "xmlns",
        Some(other) => other,
    };
    format!("{designator} {local}")
}
fn render<T: Tree>(tree: &T) -> String {
    let mut out = String::new();
    if let HtmlNodeKind::Document(mode) = tree.top().kind() {
        out.push_str(&format!("(mode {mode:?})\n"));
    }
    let mut pending: Vec<(HtmlNodeId, usize)> = tree
        .top()
        .children()
        .rev()
        .map(|child| (child, 0))
        .collect();
    while let Some((id, depth)) = pending.pop() {
        let Some(node) = tree.lookup(&id) else {
            out.push_str("(foreign identity)\n");
            continue;
        };
        let indent = "  ".repeat(depth);
        let line = match node.kind() {
            HtmlNodeKind::Element(element) => {
                let mut line = format!("<{}>", dump_name(element.name(), true));
                let mut attributes: Vec<String> = element
                    .attributes()
                    .iter()
                    .map(|a| {
                        format!(
                            "{}=\"{}\"",
                            dump_name(a.name(), false),
                            text(a.value().code_units())
                        )
                    })
                    .collect();
                attributes.sort();
                for attribute in attributes {
                    line.push_str(&format!("\n| {indent}  {attribute}"));
                }
                line
            }
            HtmlNodeKind::Text(t) => format!("\"{}\"", text(t.code_units())),
            HtmlNodeKind::Comment(t) => format!("<!-- {} -->", text(t.code_units())),
            HtmlNodeKind::ProcessingInstruction(pi) => format!(
                "<?{} {}>",
                text(pi.target().code_units()),
                text(pi.data().code_units())
            ),
            HtmlNodeKind::Doctype(d) => {
                let id =
                    |v: Option<&DocumentString>| v.map_or(String::new(), |v| text(v.code_units()));
                let (public, system) = (id(d.public_id()), id(d.system_id()));
                let name = text(d.name().code_units());
                if public.is_empty() && system.is_empty() {
                    format!("<!DOCTYPE {name}>")
                } else {
                    format!("<!DOCTYPE {name} \"{public}\" \"{system}\">")
                }
            }
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment => "(nested root)".to_owned(),
        };
        out.push_str(&format!("| {indent}{line}\n"));
        let mut children: Vec<(HtmlNodeId, usize)> =
            node.children().map(|child| (child, depth + 1)).collect();
        if let Some(contents) = node.template_contents() {
            out.push_str(&format!("| {indent}  content\n"));
            if let Some(contents) = tree.lookup(&contents) {
                children = contents
                    .children()
                    .map(|child| (child, depth + 2))
                    .chain(children)
                    .collect();
            }
        }
        pending.extend(children.into_iter().rev());
    }
    out
}

/// Corpus-dump rendering of a public fragment result, for the direct tests
/// that assert the public tree against an independently written expectation.
pub(super) fn render_fragment(fragment: &HtmlFragment) -> String {
    render(fragment)
}

/// The `render` output for a decoded fixture's expected tree. `render` ends
/// every record line with a line feed and emits nothing for an empty tree,
/// while `decode` strips the separator after the last record and permits an
/// empty fragment expectation. Both public-tree comparisons use this one rule.
pub(super) fn expected_dump(expected: &str) -> String {
    if expected.is_empty() {
        String::new()
    } else {
        format!("{expected}\n")
    }
}

fn actual_document(input: &str, utf16: bool) -> String {
    match document_request(input, utf16).parse() {
        Ok(report) => report.document().map_or("(no document)".to_owned(), render),
        Err(failure) => format!("(parse failure {failure:?})"),
    }
}
fn actual_fragment(context: &str, input: &str, utf16: bool) -> String {
    let request = match fragment_request(context, input, utf16) {
        Ok(request) => request,
        Err(error) => return format!("(request {error})"),
    };
    match request.parse() {
        Ok(report) => report.fragment().map_or("(no fragment)".to_owned(), render),
        Err(failure) => format!("(parse failure {failure:?})"),
    }
}

fn panic_text(panic: Box<dyn std::any::Any + Send>) -> String {
    panic
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| panic.downcast_ref::<&str>().map(|s| (*s).to_owned()))
        .unwrap_or_default()
}

/// Runs one retained case in both encodings; `None` when it is not run.
fn replay_case(file: &str, fixture: &Fixture<'_>) -> Option<Vec<String>> {
    if fixture.scripting == FixtureScripting::Enabled {
        return None;
    }
    let mut divergences = Vec::new();
    for utf16 in [false, true] {
        let run = std::panic::catch_unwind(|| match fixture.fragment {
            None => {
                let mut expected = parsed_expected(
                    fixture,
                    document_mode(file, fixture).unwrap_or(HtmlDocumentMode::NoQuirks),
                )
                .map_err(|e| format!("corpus tree: {e:?}"))?;
                if document_mode(file, fixture).is_none() {
                    expected.nodes[0].kind = Kind::DocumentModeNotCarried;
                }
                check(&document_request(fixture.input, utf16), &expected).map_err(|e| e.to_string())
            }
            Some(context) => {
                let expected = parsed_expected(fixture, HtmlDocumentMode::NoQuirks)
                    .map_err(|e| format!("corpus tree: {e:?}"))?;
                check_fragment_observation(
                    fragment_request(context, fixture.input, utf16)?,
                    &expected,
                    None,
                    false,
                )?;
                let actual = actual_fragment(context, fixture.input, utf16);
                if actual != expected_dump(fixture.expected) {
                    return Err(format!("public fragment tree differs:\n{actual}"));
                }
                Ok(())
            }
        });
        let error = match run {
            Ok(Ok(())) => continue,
            Ok(Err(error)) => error,
            Err(panic) => format!("panicked: {}", panic_text(panic)),
        };
        divergences.push(format!("utf16={utf16}: {error}"));
    }
    Some(divergences)
}

fn describe(file: &str, fixture: &Fixture<'_>, divergences: &[String]) -> String {
    let (kind, actual) = match fixture.fragment {
        None => (
            format!("document, mode {:?}", document_mode(file, fixture)),
            actual_document(fixture.input, false),
        ),
        Some(context) => (
            format!("fragment {context:?}"),
            actual_fragment(context, fixture.input, false),
        ),
    };
    format!(
        "{file}/{} ({kind}) input {:?}\n  {}\n  expected:\n{}\n  actual:\n{}",
        fixture.ordinal,
        fixture.input,
        divergences.join("\n  "),
        fixture.expected,
        actual
    )
}

#[derive(Default)]
struct Tally {
    documents: usize,
    fragments: usize,
    enabled: usize,
}

fn replay(kind_filter: fn(&Fixture<'_>) -> bool) -> Tally {
    let mut tally = Tally::default();
    let mut failures = Vec::new();
    for (file, raw) in DATA {
        let fixtures = decode(raw).unwrap_or_else(|e| panic!("{file}: {e:?}"));
        for (excluded, ordinal, _) in EXCLUSIONS {
            if excluded == file
                && !fixtures
                    .iter()
                    .any(|f| f.ordinal == *ordinal && f.scripting != FixtureScripting::Enabled)
            {
                failures.push(format!(
                    "{file}/{ordinal}: exclusion names no replayed case"
                ));
            }
        }
        for fixture in fixtures.iter().filter(|f| kind_filter(f)) {
            let Some(divergences) = replay_case(file, fixture) else {
                tally.enabled += 1;
                continue;
            };
            if fixture.fragment.is_some() {
                tally.fragments += 1;
            } else {
                tally.documents += 1;
                if document_mode(file, fixture).is_none() {
                    failures.push(format!(
                        "{file}/{}: no derived document mode",
                        fixture.ordinal
                    ));
                }
            }
            let exclusion = EXCLUSIONS
                .iter()
                .find(|(f, o, _)| f == file && *o == fixture.ordinal)
                .map(|(_, _, e)| *e);
            match exclusion {
                None if divergences.is_empty() => {}
                None => failures.push(describe(file, fixture, &divergences)),
                Some(Exclusion::Spec(reason, _)) if divergences.is_empty() => {
                    failures.push(format!(
                        "{file}/{} now matches; stale spec exclusion: {reason}",
                        fixture.ordinal
                    ))
                }
                Some(Exclusion::Spec(reason, expected)) => {
                    for utf16 in [false, true] {
                        let actual = match fixture.fragment {
                            None => actual_document(fixture.input, utf16),
                            Some(context) => actual_fragment(context, fixture.input, utf16),
                        };
                        if actual != expected {
                            failures.push(format!(
                                "{file}/{} utf16={utf16} differs from its pinned tree ({reason}):\n  expected:\n{expected}\n  actual:\n{actual}",
                                fixture.ordinal
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} divergent retained cases:\n{}",
        failures.len(),
        failures.join("\n\n")
    );
    tally
}

#[test]
fn every_retained_ordinary_case_matches_its_corpus_tree_after_real_completion() {
    let tally = replay(|f| f.fragment.is_none());
    // 1583 retained ordinary cases; the eight `#script-on` cases are excluded
    // for SCRIPT_ON_REASON.
    assert_eq!(
        (tally.documents, tally.fragments, tally.enabled),
        (1575, 0, 8)
    );
}

#[test]
fn every_retained_fragment_case_matches_its_corpus_tree_in_its_actual_context() {
    let tally = replay(|f| f.fragment.is_some());
    assert_eq!(
        (tally.documents, tally.fragments, tally.enabled),
        (0, 192, 0)
    );
}

#[test]
fn exclusions_and_mode_bindings_name_only_retained_cases_once_with_reasons() {
    for (i, (file, ordinal, exclusion)) in EXCLUSIONS.iter().enumerate() {
        assert!(DATA.iter().any(|(f, _)| f == file), "{file}");
        assert!(
            !EXCLUSIONS[..i]
                .iter()
                .any(|(f, o, _)| f == file && o == ordinal),
            "{file}/{ordinal} repeated"
        );
        let Exclusion::Spec(reason, _) = exclusion;
        assert!(!reason.is_empty(), "{file}/{ordinal}");
    }
    for (i, (file, ordinal, _)) in HAND_DERIVED_MODES.iter().enumerate() {
        assert!(
            !HAND_DERIVED_MODES[..i]
                .iter()
                .any(|(f, o, _)| f == file && o == ordinal),
            "{file}/{ordinal} bound twice"
        );
        let (_, raw) = DATA
            .iter()
            .find(|(f, _)| f == file)
            .unwrap_or_else(|| panic!("{file} is not bundled"));
        let fixtures = decode(raw).unwrap();
        let fixture = fixtures
            .iter()
            .find(|f| f.ordinal == *ordinal)
            .unwrap_or_else(|| panic!("{file}/{ordinal} is not a case"));
        assert!(fixture.fragment.is_none(), "{file}/{ordinal} is a fragment");
        assert!(
            fixture.scripting != FixtureScripting::Enabled,
            "{file}/{ordinal} is not replayed"
        );
    }
}

#[test]
fn derived_mode_rules_follow_initial_doctype_records_not_output() {
    let raw = "#data\nx\n#errors\n#document\n| <html>\n\n#data\n<!doctype HTML><p>\n#errors\n#document\n| <!DOCTYPE html>\n| <html>\n\n#data\n<!DOCTYPE html PUBLIC \"x\">\n#errors\n#document\n| <!DOCTYPE html \"x\" \"\">\n| <html>\n";
    let fixtures = decode(raw).unwrap();
    assert_eq!(
        document_mode("other.dat", &fixtures[0]),
        Some(HtmlDocumentMode::Quirks)
    );
    assert_eq!(
        document_mode("other.dat", &fixtures[1]),
        Some(HtmlDocumentMode::NoQuirks)
    );
    assert_eq!(document_mode("other.dat", &fixtures[2]), None);
}

#[test]
fn mode_omitted_document_still_rejects_a_changed_tree() {
    let raw = "#data\n<!DOCTYPE html SYSTEM \"about:legacy-compat\"><p>a\n#errors\n#document\n| <!DOCTYPE html \"\" \"about:legacy-compat\">\n| <html>\n|   <head>\n|   <body>\n|     <p>\n|       \"b\"\n";
    let fixtures = decode(raw).unwrap();
    assert_eq!(document_mode("other.dat", &fixtures[0]), None);
    let mut expected = parsed_expected(&fixtures[0], HtmlDocumentMode::NoQuirks).unwrap();
    expected.nodes[0].kind = Kind::DocumentModeNotCarried;
    assert!(matches!(
        check(&document_request(fixtures[0].input, false), &expected),
        Err(CaseError::Observation(AdapterError::Mismatch))
    ));
    let Kind::Text(text) = &mut expected.nodes[6].kind else {
        panic!("text record")
    };
    *text = units("a");
    check(&document_request(fixtures[0].input, true), &expected).unwrap();
}

#[test]
fn empty_fragment_renders_equal_to_an_empty_expectation() {
    // HTML877d fragment parsing: empty input in a div context leaves the root
    // without children, so the tree is empty and its dump is empty.
    let raw = "#data\n\n#errors\n#document-fragment\ndiv\n#document\n";
    let fixture = decode(raw).unwrap().into_iter().next().unwrap();
    assert_eq!((fixture.input, fixture.expected), ("", ""));
    for utf16 in [false, true] {
        assert_eq!(
            actual_fragment("div", fixture.input, utf16),
            expected_dump(fixture.expected),
            "utf16={utf16}"
        );
    }
    assert_eq!(expected_dump("| <head>"), "| <head>\n");
}

#[test]
fn script_on_cases_are_exactly_the_documents_the_disabled_only_profile_excludes() {
    let mut script_on = Vec::new();
    for (file, raw) in DATA {
        for fixture in decode(raw).unwrap() {
            if fixture.scripting == FixtureScripting::Enabled {
                assert!(
                    fixture.fragment.is_none(),
                    "{file}/{}: a fragment case selects its mode from the context \
                     document, not a marker",
                    fixture.ordinal
                );
                script_on.push((*file, fixture.ordinal));
            }
        }
    }
    assert_eq!(
        script_on,
        [
            ("tests5.dat", 16),
            ("tests16.dat", 84),
            ("tests16.dat", 86),
            ("tests16.dat", 88),
            ("tests16.dat", 181),
            ("tests16.dat", 183),
            ("tests16.dat", 185),
            ("webkit02.dat", 2),
        ],
        "{SCRIPT_ON_REASON}"
    );
    assert_eq!(
        document_request("x", false).scripting_mode(),
        surgeist_html::parsing::HtmlParserScriptingMode::Disabled
    );
}

/// Asserts that two shapes are equal, naming the first line that differs.
fn assert_same_shape(left: &Result<String, String>, right: &Result<String, String>, label: &str) {
    if left == right {
        return;
    }
    let (Ok(l), Ok(r)) = (left, right) else {
        panic!("{label}: {left:?} != {right:?}")
    };
    for (index, (a, b)) in l.lines().zip(r.lines()).enumerate() {
        assert_eq!(a, b, "{label}: line {index}");
    }
    panic!(
        "{label}: {} lines against {}",
        l.lines().count(),
        r.lines().count()
    );
}

/// Every public reader of one metadata record, in a form that does not name node
/// identities.
fn metadata_shape(out: &mut String, record: &crate::HtmlParseMetadata<'_>) {
    use std::fmt::Write as _;
    macro_rules! show {
        ($name:literal, $value:expr) => {
            if let Some(value) = $value {
                writeln!(out, "  {}={:?}", $name, value).unwrap();
            }
        };
    }
    writeln!(out, "meta {:?}", record.node().kind()).unwrap();
    show!("nonce", record.cryptographic_nonce());
    show!("input_flags", record.input_flags());
    show!("input_raw_value", record.input_raw_value());
    show!("input_email_value_count", record.input_email_value_count());
    show!("input_cursor", record.input_cursor());
    show!(
        "input_selected_files_empty",
        record.input_selected_files_empty()
    );
    show!(
        "input_creation_writeback",
        record.input_creation_writeback()
    );
    show!("textarea_flags", record.textarea_flags());
    show!("textarea_raw_value", record.textarea_raw_value());
    show!("select_flags", record.select_flags());
    show!("option_flags", record.option_flags());
    show!(
        "selectedcontent_disabled",
        record.selectedcontent_disabled()
    );
    show!("form_parser_inserted", record.form_parser_inserted());
    show!("pi_attribute_count", record.pi_attribute_count());
    show!("shadow", record.shadow());
    show!("link_flags", record.link_flags());
    show!("link_history", record.link_history());
    let label = |node: Option<HtmlNode<'_>>| node.map(|node| format!("{:?}", node.kind()));
    show!("form_owner", record.form_owner().map(label));
    show!(
        "option_nearest_select",
        record.option_nearest_select().map(label)
    );
    show!(
        "document_first_base",
        record.document_first_base().map(label)
    );
    show!(
        "document_facts",
        record
            .document_facts()
            .map(|(class, flag, inert)| (class, flag, label(inert)))
    );
    if let Some(dialog) = record.dialog() {
        writeln!(out, "  dialog={:?}", dialog.return_value()).unwrap();
    }
    if let Some(image) = record.image() {
        writeln!(
            out,
            "  image={:?} {}",
            image.state(),
            image.latest_started()
        )
        .unwrap();
    }
    if let Some(script) = record.script_state() {
        writeln!(
            out,
            "  script={} {} {:?} {} {} {} {}",
            script.force_async(),
            script.already_started(),
            script.script_type(),
            script.external_file(),
            script.ready_to_be_parser_executed(),
            script.delaying_load(),
            script.steps_when_result_ready().is_some()
        )
        .unwrap();
    }
    if let Some(style) = record.style() {
        writeln!(
            out,
            "  style={:?} {}",
            style.state(),
            style.creating_parser_document().is_some()
        )
        .unwrap();
    }
    if record.hyperlink().is_some() {
        writeln!(out, "  hyperlink").unwrap();
    }
    if let Some(attributes) = record.pi_attribute_count() {
        for index in 0..attributes {
            show!("pi_attribute", record.pi_attribute(index));
        }
    }
}

/// The tree, the metadata and the pending associations of a report, in a form
/// that does not name node identities, so two parses of one input compare.
fn shape(report: &crate::HtmlParseReport) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    if let Some(document) = report.document() {
        out.push_str(&render(document));
    }
    if let Some(fragment) = report.fragment() {
        out.push_str(&render_fragment(fragment));
    }
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            crate::HtmlParseMetadataStep::Record(record) => metadata_shape(&mut out, &record),
            crate::HtmlParseMetadataStep::Skipped => {}
            crate::HtmlParseMetadataStep::Complete => break,
            crate::HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
    match report.effects() {
        Ok(Some(effects)) => {
            // The number of registry slots counts the rows a parse keeps, which
            // detail changes; the effects themselves do not change.
            writeln!(out, "effects {}", effects.len()).unwrap();
            for index in 0..effects.len() {
                match effects.effect(index) {
                    Ok(Some(crate::HtmlParseEffect::TemplateCache { inert, .. })) => {
                        writeln!(out, "  template-cache {:?}", inert.kind()).unwrap();
                    }
                    Ok(Some(crate::HtmlParseEffect::AttachShadow {
                        template,
                        contents,
                        properties,
                        keep_registry_null,
                        declarative,
                        available_to_internals,
                        source,
                        ..
                    })) => {
                        writeln!(
                            out,
                            "  attach-shadow {:?} {:?} {properties:?} {keep_registry_null} {declarative} {available_to_internals} {}..{}",
                            template.kind(),
                            contents.kind(),
                            source.start(),
                            source.end()
                        )
                        .unwrap();
                    }
                    other => panic!("effect {index}: {}", other.is_ok()),
                }
            }
        }
        Ok(None) => {}
        Err(fault) => panic!("{fault}"),
    }
    let mut pending = report.pending_associations();
    loop {
        match pending.advance() {
            crate::HtmlParsePendingAssociationStep::Occurrence(row) => {
                let value = |value: Option<&DocumentString>| value.map(|v| text(v.code_units()));
                writeln!(
                    out,
                    "occurrence {:?} {} {:?} {:?}",
                    row.kind(),
                    row.requires_processing(),
                    value(row.preferred_name()),
                    value(row.base_changed_input()),
                )
                .unwrap();
                if let Some(style) = row.style() {
                    for index in 0..style.text_len() {
                        writeln!(out, "style text {:?}", value(style.text(index).unwrap()))
                            .unwrap();
                    }
                }
            }
            crate::HtmlParsePendingAssociationStep::Document(row) => {
                writeln!(
                    out,
                    "document {} {}",
                    row.occurrence_count(),
                    row.has_obligations()
                )
                .unwrap();
            }
            crate::HtmlParsePendingAssociationStep::Skipped => {}
            crate::HtmlParsePendingAssociationStep::Complete => break,
            crate::HtmlParsePendingAssociationStep::Fault(fault) => panic!("{fault}"),
        }
    }
    out
}

/// The shape of one parse of a retained case with the detail selected, or the
/// failure it ended in.
fn shape_with(
    fixture: &Fixture<'_>,
    utf16: bool,
    detail: crate::DetailSet,
) -> Result<String, String> {
    let parsed = match fixture.fragment {
        None => document_request(fixture.input, utf16)
            .with_detail(detail)
            .map_err(|e| e.to_string())?
            .parse(),
        Some(context) => fragment_request(context, fixture.input, utf16)?
            .with_detail(detail)
            .map_err(|e| e.to_string())?
            .parse(),
    };
    match parsed {
        Ok(report) => Ok(shape(&report)),
        Err(failure) => Err(format!("{:?} {:?}", failure.reason(), failure.first_stop())),
    }
}

/// Parsing every retained case with no detail, with each kind alone and with
/// all of it gives one tree, the same metadata and the same pending
/// associations: detail records the parse, and does not change it.
#[cfg(feature = "detail")]
#[test]
fn every_retained_case_gives_the_same_result_whatever_detail_the_parse_selects() {
    use crate::DetailSet;
    let mut compared = 0;
    for (file, raw) in DATA {
        for fixture in decode(raw).unwrap_or_else(|e| panic!("{file}: {e:?}")) {
            if fixture.scripting == FixtureScripting::Enabled {
                continue;
            }
            for utf16 in [false, true] {
                let expected = shape_with(&fixture, utf16, DetailSet::ALL);
                for detail in [
                    DetailSet::NONE,
                    DetailSet::SOURCE_MAP,
                    DetailSet::LEXICAL,
                    DetailSet::HISTORY,
                    DetailSet::DIAGNOSTICS,
                ] {
                    assert_same_shape(
                        &shape_with(&fixture, utf16, detail),
                        &expected,
                        &format!("{file}/{} utf16={utf16} detail={detail:?}", fixture.ordinal),
                    );
                }
                compared += 1;
            }
        }
    }
    assert!(compared > 3000, "{compared} comparisons");
}

/// Inputs that reach the code a parse without a graph takes its own way: the value
/// sanitization and generated values of the input types, text areas, selects,
/// details, the image name recovery and the splitting of body text.
const DETAIL_INPUTS: &[&str] = &[
    "<input type=color value=#FF0000>",
    "<input type=color value=red alpha colorspace=display-p3>",
    "<input type=color value=currentcolor colorspace=bogus>",
    "<input type=color>",
    "<input type=range min=1 max=10 step=3 value=50>",
    "<input type=range min=a max=b step=any value=x>",
    "<input type=range value=5 min=10 max=1>",
    "<input type=number value=' 12 '>",
    "<input type=number value=1e3 step=any>",
    "<input type=email multiple value=' a@b.c ,\n d@e.f '>",
    "<input type=email value='a@b.c\r\nd'>",
    "<input type=text value='a\nb\r\nc\rd'>",
    "<input type=url value='  http://x \n'>",
    "<input type=search value='x\ty'>",
    "<input type=password value='\n\n'>",
    "<input type=tel value=' 1 2 '>",
    "<input type=date value=2020-01-02>",
    "<input type=month value=2020-13>",
    "<input type=week value=2020-W53>",
    "<input type=time value=25:00>",
    "<input type=datetime-local value='2020-01-02 03:04'>",
    "<input type=datetime-local value='2020-01-02T03:04:05.6'>",
    "<input type=checkbox checked value=v><input type=radio name=r checked><input type=radio name=r>",
    "<input type=file multiple><input type=image src=x alt=y><input type=hidden value=h>",
    "<input type=submit><input type=reset value=r><input type=button>",
    "<form><input name=a value=1><input name=b><textarea>t</textarea></form>",
    "<textarea>\nline1\r\nline2</textarea>",
    "<textarea rows=2 cols=3>a&amp;b</textarea><textarea></textarea>",
    "<select><option>a<option selected>b<optgroup><option>c</select>",
    "<select multiple size=2><option selected>a<option selected>b</select>",
    "<details open><summary>s</summary>d</details><details name=n><details name=n open>",
    "<image src=x><image><IMAGE>",
    "<p><image alt=y></p>",
    "</br><br></br>",
    "<table>  \n x <tr><td>y</td></tr> z </table>",
    "<table>\0 \n<td>q</table>",
    "<body>\n\n  text\0 more <b> <i>  x</b>  </i>",
    "<html>\n <head>\n</head>\r\n<body>\r\n<pre>\nx\r\n</pre>",
    "  \n  <!-- c -->  <p>x",
    "<style title=A media=screen>a{}</style><meta http-equiv=default-style content=A><base href=x>",
    "<script>1</script><script src=a async></script><script type=module>2</script>",
    "<template><input value=t><style>x</style></template><dialog open>d</dialog>",
    "<a href=x>l</a><link rel=stylesheet href=y><img src=z width=1 height=2>",
    "<svg><script>s</script><a href=x></svg><math><mi>x</mi></math>",
    "<?pi a=b c='d'?><?xml-stylesheet href=x?>",
];

#[cfg(feature = "detail")]
#[test]
fn inputs_that_take_their_own_way_without_a_graph_give_the_same_result() {
    use crate::DetailSet;
    for text in DETAIL_INPUTS {
        for utf16 in [false, true] {
            let selections: [Result<String, String>; 2] =
                [DetailSet::ALL, DetailSet::NONE].map(|detail| {
                    let parsed = document_request(text, utf16)
                        .with_detail(detail)
                        .map_err(|e| e.to_string())?
                        .parse();
                    parsed
                        .map(|report| shape(&report))
                        .map_err(|f| format!("{:?} {:?}", f.reason(), f.first_stop()))
                });
            assert_eq!(
                selections[0], selections[1],
                "document {text:?} utf16={utf16}"
            );
            for context in ["div", "table", "select", "template"] {
                let selections: [Result<String, String>; 2] = [DetailSet::ALL, DetailSet::NONE]
                    .map(|detail| {
                        let parsed = fragment_request(context, text, utf16)?
                            .with_detail(detail)
                            .map_err(|e| e.to_string())?
                            .parse();
                        parsed
                            .map(|report| shape(&report))
                            .map_err(|f| format!("{:?} {:?}", f.reason(), f.first_stop()))
                    });
                assert_same_shape(
                    &selections[0],
                    &selections[1],
                    &format!("fragment {context} {text:?} utf16={utf16}"),
                );
            }
        }
    }
}
