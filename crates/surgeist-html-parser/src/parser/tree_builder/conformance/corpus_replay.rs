//! Replay of every retained html5lib-tests `f994590f` tree-construction case.
//!
//! Each bundled `.dat` case runs through the actual parser and the metered
//! native comparison owner against its independently decoded corpus tree:
//! ordinary documents through [`check`], and every `#document-fragment` case
//! through the real captured-context fragment owner with its corpus context,
//! including SVG and MathML contexts. `#script-on` cases select Enabled
//! scripting construction, an unsupported public profile, and are not run;
//! cases without a scripting marker run with Disabled scripting.
//!
//! The corpus format does not carry document mode. A case binds its mode from
//! the hand-derived HTML877d table below, otherwise from the Initial rules the
//! corpus tree itself establishes: no doctype record selects Quirks
//! (Initial anything-else), and a plain `<!DOCTYPE html>` that both begins the
//! input and is the first expected record selects NoQuirks. Any other case is
//! compared with the document mode omitted.

use super::comment_diagnostic_corpus::template_corpus::{
    check_fragment_observation, fragment_request, mode as template_mode,
};
use super::*;
use crate::{DocumentString, HtmlDocumentParseRequest, HtmlFragment, HtmlNode, HtmlSource};

/// Why one retained case is not asserted to match.
#[derive(Clone, Copy, Debug)]
enum Exclusion {
    /// The pinned HTML edition defines a different tree; the case must still
    /// diverge, so a stale record fails.
    Spec(&'static str),
    /// A divergence without a selected owner yet; not asserted either way.
    Defect(&'static str),
}

/// Exclusions keyed by bundled file and one-based case ordinal.
/// HTML877d tag open state: `<?` enters the processing-instruction states, and
/// Initial inserts the resulting PI (or nothing, at EOF) instead of the
/// historical bogus comment the corpus records.
const PI_OPEN: Exclusion =
    Exclusion::Spec("HTML877d tag open state: U+003F enters processing-instruction open state");

const EXCLUSIONS: &[(&str, usize, Exclusion)] = &[
    ("tests1.dat", 40, PI_OPEN),
    ("tests1.dat", 44, PI_OPEN),
    ("tests1.dat", 47, PI_OPEN),
    (
        "tests_innerHTML_1.dat",
        79,
        Exclusion::Defect(
            "html context: after-body `</html>` must be ignored in the fragment case, keeping the comment under html",
        ),
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

/// Diagnostic rendering of an actual public result in the corpus dump shape;
/// only failure messages use it, never the comparison.
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
                    .as_slice()
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

fn actual_document(input: &str) -> String {
    match document_request(input, false).parse() {
        Ok(report) => report.document().map_or("(no document)".to_owned(), render),
        Err(failure) => format!("(parse failure {failure:?})"),
    }
}
fn actual_fragment(context: &str, input: &str) -> String {
    let request = match fragment_request(context, input, false) {
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
                )
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
            actual_document(fixture.input),
        ),
        Some(context) => (
            format!("fragment {context:?}"),
            actual_fragment(context, fixture.input),
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
            }
            let exclusion = EXCLUSIONS
                .iter()
                .find(|(f, o, _)| f == file && *o == fixture.ordinal)
                .map(|(_, _, e)| *e);
            match exclusion {
                None if divergences.is_empty() => {}
                None => failures.push(describe(file, fixture, &divergences)),
                Some(Exclusion::Spec(reason)) if divergences.is_empty() => failures.push(format!(
                    "{file}/{} now matches; stale spec exclusion: {reason}",
                    fixture.ordinal
                )),
                Some(Exclusion::Spec(_) | Exclusion::Defect(_)) => {}
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
    // 1319 retained ordinary cases; the eight `#script-on` cases are not run.
    assert_eq!(
        (tally.documents, tally.fragments, tally.enabled),
        (1311, 0, 8)
    );
}

#[test]
fn every_retained_fragment_case_matches_its_corpus_tree_in_its_actual_context() {
    let tally = replay(|f| f.fragment.is_some());
    assert_eq!(
        (tally.documents, tally.fragments, tally.enabled),
        (0, 176, 0)
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
        let (Exclusion::Spec(reason) | Exclusion::Defect(reason)) = exclusion;
        assert!(!reason.is_empty(), "{file}/{ordinal}");
    }
    for (file, ordinal, _) in HAND_DERIVED_MODES {
        let (_, raw) = DATA.iter().find(|(f, _)| f == file).unwrap();
        let fixtures = decode(raw).unwrap();
        let fixture = &fixtures[ordinal - 1];
        assert!(fixture.fragment.is_none(), "{file}/{ordinal}");
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
