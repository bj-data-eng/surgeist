//! The origins a published report answers for the name and the namespace of every element, with
//! the origin graph recorded, and what a parse without detail retains for them.
//!
//! Every element has one association row for its name field and one for its namespace field. The
//! group of each expands to the units of the element's own string, which the oracle reads from the
//! record of the element and not from the group. A group that has source origins names the text
//! the element's tag spelled; a group of an element no tag created has none.
use super::g2_d2::each_corpus_parse;
use super::*;
use surgeist_html::construction::{ConstructionNodeKind, HtmlNodeId};

#[derive(Default)]
struct Tally {
    parses: usize,
    elements: usize,
    names: usize,
    namespaces: usize,
    leaves: usize,
    aliases: usize,
    sourced: usize,
    texts: usize,
    implied: usize,
    undetailed: usize,
    failures: Vec<String>,
}

/// The rows of a field of an element, by element and whether the field is the name.
type Fields = std::collections::HashMap<(HtmlNodeId, bool), Vec<Group>>;

/// What the report answers for one association row of a name or namespace field.
struct Group {
    units: Vec<u16>,
    alias: bool,
    origins: Vec<(usize, usize, crate::HtmlSourceRange)>,
    captured: bool,
}

/// The group a row names, expanded to the units of its leaf through the aliases of the history.
fn expand(history: &crate::HtmlParseHistory<'_>, group: crate::HtmlParseOriginGroup<'_>) -> Group {
    let (mut alias, mut captured) = (false, false);
    let mut current = group;
    loop {
        let header = history.header(current).unwrap();
        captured |= header.cause.captured().is_some();
        match header.body {
            crate::HtmlParseHistoryBody::Leaf(leaf) => {
                return Group {
                    units: leaf.units().to_vec(),
                    alias,
                    origins: leaf
                        .origins()
                        .iter()
                        .map(|origin| (origin.start, origin.end, origin.source.clone()))
                        .collect(),
                    captured,
                };
            }
            crate::HtmlParseHistoryBody::Alias { original } => {
                alias = true;
                current = original;
            }
            crate::HtmlParseHistoryBody::Concat { .. } => panic!("a name is not a concatenation"),
        }
    }
}

/// The groups of the name and namespace fields of the elements of a report, by the first of the
/// output rows of each.
fn name_rows(report: &crate::HtmlParseReport) -> Option<Fields> {
    let detail = report.detail()?;
    let associations = detail.associations()?;
    let history = detail.history()?;
    let mut rows = Fields::new();
    for index in 0..associations.len() {
        let row = associations.row(index).unwrap().unwrap();
        // The Output rows of a field name its group, one for each origin of the group.
        if row.relation() != crate::HtmlParseAssociationRelation::Output {
            continue;
        }
        let crate::HtmlParseDestination::Field { node, field } = row.destination() else {
            continue;
        };
        if !matches!(
            field,
            crate::HtmlParseField::ElementName | crate::HtmlParseField::Namespace
        ) {
            continue;
        }
        let group = row.group().expect("a name row names its group");
        let rows = rows
            .entry((node.id(), field == crate::HtmlParseField::ElementName))
            .or_default();
        if rows.is_empty() {
            rows.push(expand(&history, group));
        }
    }
    Some(rows)
}

fn lower(units: &[u16]) -> String {
    String::from_utf16_lossy(units).to_ascii_lowercase()
}

fn check(
    report: &crate::HtmlParseReport,
    label: &str,
    source: Option<&crate::HtmlSource>,
    tally: &mut Tally,
) {
    tally.parses += 1;
    let storage = report.history().storage();
    let arena = &storage.view;
    let mut fail = |message: String| tally.failures.push(format!("{label}: {message}"));
    let rows = name_rows(report);
    if rows.is_none() {
        // No detail: the table of origins is never allocated, so no origin is retained.
        if !storage.payloads.detail_origins.is_unallocated() {
            fail("origins retained without detail".into());
        }
        tally.undetailed += 1;
    }
    let mut counts = [0_usize; 8];
    for position in 0..arena.len() {
        let record = arena.node_at(position).unwrap();
        let ConstructionNodeKind::Element(element) = record.kind() else {
            continue;
        };
        counts[0] += 1;
        let id = record.as_node().unwrap().id();
        let Some(rows) = rows.as_ref() else { continue };
        let strings = [
            (true, Some(element.name().local_name().code_units())),
            (
                false,
                element
                    .name()
                    .namespace()
                    .uri()
                    .map(surgeist_html::values::DocumentString::code_units),
            ),
        ];
        for (field, units) in strings {
            let found = rows.get(&(id, field)).map_or(&[][..], Vec::as_slice);
            let Some(units) = units else {
                if !found.is_empty() {
                    fail(format!(
                        "element {position} has a row for a namespace it lacks"
                    ));
                }
                continue;
            };
            let [group] = found else {
                fail(format!(
                    "element {position} has {} rows for {field:?}",
                    found.len()
                ));
                continue;
            };
            counts[if field { 1 } else { 2 }] += 1;
            if group.units != units {
                fail(format!(
                    "element {position} {field:?} expands to other units"
                ));
            }
            if group.alias {
                counts[3] += 1;
            } else {
                counts[4] += 1;
            }
            if group.origins.is_empty() {
                if field && !group.alias {
                    counts[5] += 1;
                }
            } else {
                counts[6] += 1;
                if field
                    && let Some(source) = source
                    && !group.alias
                {
                    // The text of the tag that spelled the name: the same letters, in the case
                    // the tokenizer lowered them to.
                    let text: String = group
                        .origins
                        .iter()
                        .map(|(_, _, range)| lower(source.text(range).unwrap().code_units()))
                        .collect();
                    counts[7] += 1;
                    if text != lower(units) {
                        fail(format!(
                            "element {position} name {text} for {}",
                            lower(units)
                        ));
                    }
                }
            }
        }
    }
    tally.elements += counts[0];
    tally.names += counts[1];
    tally.namespaces += counts[2];
    tally.aliases += counts[3];
    tally.leaves += counts[4];
    tally.implied += counts[5];
    tally.sourced += counts[6];
    tally.texts += counts[7];
}

/// Sources with elements the corpus holds few of: foreign elements with adjusted names, an
/// element created by the adoption agency and by the reconstruction of the formatting elements,
/// a clone made by a select, template contents, implied `html`, `head`, `body` and `tbody`.
const SOURCES: [&str; 12] = [
    "<!doctype html><svg><foreignObject><p>x</p></foreignObject><clipPath/><linearGradient/></svg>",
    "<!doctype html><math definitionurl=a><mi definitionurl=b>x</mi><annotation-xml encoding=text/html><b>",
    "<!doctype html><b><p>x</b>y</p><i><u>z</i>w",
    "<!doctype html><p><b>x<p>y<p>z",
    "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected><b>t</b><i></i></option></select>",
    "<!doctype html><template><div><template><i></i></template></div></template>",
    "<!doctype html><table><td>a<tr><td>b</table>",
    "x<p>y",
    "<table><tr><td>z",
    "<!doctype html><custom-element is=x><my-widget></my-widget><div nonce=a></div>",
    "<!doctype html><svg><title><b>x</b></title><desc><i></i></desc></svg><math><mtext><b>y</b></mtext></math>",
    "<!doctype html><a><table><a></table></a><nobr>1<nobr>2<ruby><rt>3",
];

#[test]
fn the_origins_a_report_answers_for_element_names_are_the_strings_of_the_elements() {
    let tally = std::cell::RefCell::new(Tally::default());
    each_corpus_parse(
        |report, label| check(report, label, None, &mut tally.borrow_mut()),
        |label| {
            tally
                .borrow_mut()
                .failures
                .push(format!("{label}: parse failed"))
        },
    );
    let mut tally = tally.into_inner();
    let corpus = (tally.parses, tally.elements, tally.names);
    for source in SOURCES {
        for detail in g2_d2::details() {
            let label = format!("{source} detail={detail:?}");
            let text = crate::HtmlSource::utf8(source);
            let report =
                crate::HtmlDocumentParseRequest::offline(text.clone(), g2_d2::corpus_limits())
                    .with_detail(detail)
                    .unwrap()
                    .parse()
                    .unwrap();
            check(&report, &label, Some(&text), &mut tally);
            // The fragment parse of the same markup, in the context of a `div` and of an SVG `g`.
            for context in ["div", "svg g"] {
                let fragment = source.strip_prefix("<!doctype html>").unwrap_or(source);
                let text = crate::HtmlSource::utf8(fragment);
                let report = crate::HtmlFragmentParseRequest::new(
                    text.clone(),
                    g2_d2::fragment_context(context).unwrap(),
                    g2_d2::corpus_limits(),
                )
                .unwrap()
                .with_detail(detail)
                .unwrap()
                .parse()
                .unwrap();
                check(
                    &report,
                    &format!("{context} fragment {label}"),
                    Some(&text),
                    &mut tally,
                );
            }
        }
    }
    eprintln!(
        "names parses {} (corpus {}) elements {} (corpus {}) names {} (corpus {}) namespaces {} \
         leaves {} aliases {} sourced {} source-texts {} implied {} undetailed {} failures {}",
        tally.parses,
        corpus.0,
        tally.elements,
        corpus.1,
        tally.names,
        corpus.2,
        tally.namespaces,
        tally.leaves,
        tally.aliases,
        tally.sourced,
        tally.texts,
        tally.implied,
        tally.undetailed,
        tally.failures.len()
    );
    for line in tally.failures.iter().take(30) {
        eprintln!("{line}");
    }
    assert!(tally.failures.is_empty());
    assert_eq!(corpus.0, 1_767 * g2_d2::details().len());
    assert_eq!(
        tally.parses,
        corpus.0 + 3 * g2_d2::details().len() * SOURCES.len()
    );
    assert_eq!(tally.undetailed, tally.parses / g2_d2::details().len());
    if cfg!(feature = "detail") {
        // Every element of a detailed parse has its name row, and its namespace row.
        assert_eq!(tally.names, tally.namespaces);
        assert!(tally.names >= 18_000 / 2);
        assert!(tally.implied > 0);
        assert!(tally.aliases > 0);
        assert!(tally.leaves > tally.aliases);
        assert!(tally.sourced > 0);
        assert!(tally.texts > 0);
    }
}

/// The units of the string of the `field` row of the first element named `name` of a report.
fn first_group(report: &crate::HtmlParseReport, name: &str, is_name: bool) -> Group {
    let storage = report.history().storage();
    let rows = name_rows(report).expect("the parse records detail");
    for position in 0..storage.view.len() {
        let record = storage.view.node_at(position).unwrap();
        if let ConstructionNodeKind::Element(element) = record.kind()
            && String::from_utf16_lossy(element.name().local_name().code_units()) == name
        {
            let id = record.as_node().unwrap().id();
            return rows
                .into_iter()
                .find(|((node, f), _)| *node == id && *f == is_name)
                .unwrap()
                .1
                .remove(0);
        }
    }
    panic!("no element named {name}");
}

fn detailed(source: &str) -> Option<crate::HtmlParseReport> {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(source),
        g2_d2::corpus_limits(),
    )
    .with_detail(crate::DetailSet::ALL)
    .ok()?;
    Some(request.parse().unwrap())
}

/// The elements a source implies have a name whose group has no source origin: no tag spelled it.
#[test]
fn an_implied_element_has_a_name_group_without_a_source_origin() {
    let Some(report) = detailed("x") else { return };
    for name in ["html", "head", "body"] {
        let group = first_group(&report, name, true);
        assert!(group.origins.is_empty(), "{name}");
        assert_eq!(group.units, name.encode_utf16().collect::<Vec<_>>());
    }
    let report = detailed("<!doctype html><table><td>a").unwrap();
    let group = first_group(&report, "tbody", true);
    assert!(group.origins.is_empty());
}

/// A tag spells its element's name: the origin of the name is the range of the tag's name.
#[test]
fn an_explicit_element_has_a_name_group_with_the_source_origin_of_its_tag() {
    let Some(report) = detailed("<!doctype html><span>x</span>") else {
        return;
    };
    let group = first_group(&report, "span", true);
    // One atom for each unit of `span`, in the tag at 16, after `<!doctype html><` and its `<`.
    assert_eq!(group.origins.len(), 4);
    for (index, (start, end, range)) in group.origins.iter().enumerate() {
        assert_eq!((*start, *end), (index, index + 1));
        assert_eq!((range.start(), range.end()), (16 + index, 17 + index));
    }
}

/// An element the adoption agency creates is a clone of an element a tag spelled: its name group
/// expands to the same units and the same source origins as the original's.
#[test]
fn an_element_created_by_the_adoption_agency_has_the_origins_of_the_original_name() {
    let Some(report) = detailed("<!doctype html><b><p>x</b>y</p>") else {
        return;
    };
    let storage = report.history().storage();
    let rows = name_rows(&report).unwrap();
    let mut bs = Vec::new();
    for position in 0..storage.view.len() {
        let record = storage.view.node_at(position).unwrap();
        if let ConstructionNodeKind::Element(element) = record.kind()
            && String::from_utf16_lossy(element.name().local_name().code_units()) == "b"
        {
            bs.push(&rows[&(record.as_node().unwrap().id(), true)][0]);
        }
    }
    // The tag's own element, and the element the adoption agency creates for what follows `</b>`.
    assert_eq!(bs.len(), 2);
    assert!(!bs[1].origins.is_empty());
    assert_eq!(bs[0].units, bs[1].units);
    assert_eq!(bs[0].origins, bs[1].origins);
}

/// A foreign element's name is adjusted, and the units of the adjusted name are the field's.
#[test]
fn a_foreign_element_with_an_adjusted_name_has_the_adjusted_units() {
    let Some(report) = detailed("<!doctype html><svg><foreignObject></foreignObject></svg>") else {
        return;
    };
    let group = first_group(&report, "foreignObject", true);
    assert_eq!(String::from_utf16_lossy(&group.units), "foreignObject");
    let namespace = first_group(&report, "foreignObject", false);
    assert_eq!(
        String::from_utf16_lossy(&namespace.units),
        "http://www.w3.org/2000/svg"
    );
    assert!(namespace.origins.is_empty());
}

/// The namespace of an element created in the namespace of the fragment context is a copy of the
/// captured namespace, and its group is the one the capture made.
#[test]
fn an_element_in_the_namespace_of_the_fragment_context_has_the_captured_namespace_group() {
    use crate::*;
    use surgeist_html::{
        construction::HtmlDocumentMode,
        values::{Namespace, OrderedAttributes, QualifiedName},
    };
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10, 10, 10, 1000, 10, 10, 10, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let context = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::from_uri("urn:retained"), None, "context"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&context).unwrap();
    let capture = builder.finish().unwrap();
    let context =
        HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
    let Ok(request) = HtmlFragmentParseRequest::new(
        HtmlSource::utf8("<foo></foo><bar></bar>"),
        context,
        g2_d2::corpus_limits(),
    )
    .unwrap()
    .with_detail(DetailSet::ALL) else {
        return;
    };
    let report = request.parse().unwrap();
    for name in ["foo", "bar"] {
        let group = first_group(&report, name, false);
        assert!(group.captured, "{name}");
        assert!(group.origins.is_empty(), "{name}");
        assert_eq!(String::from_utf16_lossy(&group.units), "urn:retained");
    }
}

/// The units of an element's name are counted once, with the element. A plain element of the HTML
/// namespace named `span` counts the units of `span` and of its namespace URI: the difference
/// between documents of 16 and 64 plain elements, without detail.
#[test]
fn a_plain_element_counts_the_units_of_its_name_and_namespace() {
    let units = |elements: usize| {
        let source = format!("<!doctype html>{}", "<span></span>".repeat(elements));
        crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source.as_str()),
            g2_d2::corpus_limits(),
        )
        .parse()
        .unwrap()
        .usage()
        .resource(crate::HtmlParseResource::ManagedUtf16Units)
        .current()
    };
    let namespace = "http://www.w3.org/1999/xhtml".len() as u64;
    assert_eq!(
        units(64) - units(16),
        (64 - 16) * ("span".len() as u64 + namespace)
    );
}
