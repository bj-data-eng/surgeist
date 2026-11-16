//! The document a published report answers for every node of every retained html5lib
//! tree-construction case, against the document the parse's column named for the node when the
//! parse was frozen, with detail off and on.
use super::*;
use crate::construction::report::HtmlParseMetadataDocument;
use surgeist_html_semantics::*;

fn corpus_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .active_formatting_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(1_000)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(1_000_000_000)
        .finish()
        .unwrap()
}

fn fragment_context(context: &str) -> Result<HtmlFragmentContext, String> {
    let (namespace, local) = match context.split_once(' ') {
        Some(("svg", local)) => (Namespace::svg(), local),
        Some(("math", local)) => (Namespace::mathml(), local),
        Some(_) => return Err("namespace".into()),
        None => (Namespace::html(), context),
    };
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .map_err(|e| format!("{e:?}"))?;
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .map_err(|e| format!("{e:?}"))?;
    let element = builder
        .add_element(
            &document,
            QualifiedName::new(namespace, None, local),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .map_err(|e| format!("{e:?}"))?;
    builder
        .establish_root(&element)
        .map_err(|e| format!("{e:?}"))?;
    let capture = builder.finish().map_err(|e| format!("{e:?}"))?;
    let subject = capture.subject_id(&element).map_err(|e| format!("{e:?}"))?;
    HtmlFragmentContext::element(&capture, &subject).map_err(|e| format!("{e:?}"))
}

struct Case<'a> {
    input: &'a str,
    fragment: Option<&'a str>,
    script_on: bool,
}

fn cases(file: &str) -> Vec<Case<'_>> {
    let file = file.strip_suffix('\n').unwrap_or(file);
    let mut out = Vec::new();
    for (index, block) in file.split("\n\n#data\n").enumerate() {
        let block = if index == 0 {
            block.strip_prefix("#data\n").unwrap_or(block)
        } else {
            block
        };
        let (input, remainder) = match block.strip_prefix("#errors\n") {
            Some(r) => ("", r),
            None => match block.split_once("\n#errors\n") {
                Some(parts) => parts,
                None => continue,
            },
        };
        let metadata = remainder
            .split_once("#document\n")
            .map_or(remainder, |p| p.0);
        let mut lines = metadata.split_terminator('\n');
        let mut fragment = None;
        let mut script_on = false;
        while let Some(line) = lines.next() {
            match line {
                "#document-fragment" => fragment = lines.next(),
                "#script-on" => script_on = true,
                _ => {}
            }
        }
        out.push(Case {
            input,
            fragment,
            script_on,
        });
    }
    out
}

#[derive(Default)]
struct Tally {
    parses: usize,
    nodes: usize,
    named: usize,
    unnamed: usize,
    roots: usize,
    unnamed_roots: usize,
    links: usize,
    failures: Vec<String>,
}

fn check(report: &crate::HtmlParseReport, label: &str, tally: &mut Tally) {
    let storage = report.history().storage();
    let contexts = &storage.payloads.facts.contexts;
    let arena = &storage.view;
    tally.parses += 1;
    if contexts.frozen_column.len() != arena.len() {
        tally.failures.push(format!(
            "{label}: column {} nodes {}",
            contexts.frozen_column.len(),
            arena.len()
        ));
        return;
    }
    for (position, &entry) in contexts.frozen_column.iter().enumerate() {
        tally.nodes += 1;
        let record = arena.node_at(position).unwrap();
        let id = record.as_node().unwrap().id();
        let answer = report.node_document_for_test(&id).unwrap().document();
        let expected = contexts.document(entry);
        let agree = match (expected, answer) {
            (None, None) => {
                tally.unnamed += 1;
                true
            }
            (Some(DocumentRef::Owned(expected)), Some(HtmlParseMetadataDocument::Owned(found))) => {
                tally.named += 1;
                found.id() == arena.node_at(*expected).unwrap().as_node().unwrap().id()
            }
            (Some(DocumentRef::Captured(_)), Some(HtmlParseMetadataDocument::Captured(_))) => {
                tally.named += 1;
                true
            }
            _ => false,
        };
        if !agree {
            tally
                .failures
                .push(format!("{label}: node {position} column {expected:?}"));
        }
        if record.parent().is_none() {
            tally.roots += 1;
            if answer.is_none() {
                tally.unnamed_roots += 1;
                tally
                    .failures
                    .push(format!("{label}: unnamed root {position}"));
            }
        }
        if let ConstructionNodeKind::Element(element) = record.kind()
            && element.link_flags().is_some()
        {
            tally.links += 1;
            let own = contexts.named_document(position).0;
            let own_id = match own {
                Some(DocumentRef::Owned(d)) => {
                    Some(arena.node_at(*d).unwrap().as_node().unwrap().id())
                }
                _ => None,
            };
            let by_root = match answer {
                Some(HtmlParseMetadataDocument::Owned(d)) => Some(d.id()),
                _ => None,
            };
            if own_id != by_root || (own.is_some() != answer.is_some()) {
                tally
                    .failures
                    .push(format!("{label}: link {position} own {own:?}"));
            }
        }
    }
}

#[test]
fn the_document_published_for_each_corpus_node_is_the_one_its_column_named() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/html5lib-f994590f/tree-construction");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "dat"))
        .collect();
    files.sort();
    let mut tally = Tally::default();
    for path in files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let raw = std::fs::read_to_string(&path).unwrap();
        for (ordinal, case) in cases(&raw).iter().enumerate() {
            if case.script_on {
                continue;
            }
            for detail in [crate::DetailSet::NONE, crate::DetailSet::ALL] {
                let label = format!("{name}/{} detail={detail:?}", ordinal + 1);
                let parsed = match case.fragment {
                    None => crate::HtmlDocumentParseRequest::offline(
                        crate::HtmlSource::utf8(case.input),
                        corpus_limits(),
                    )
                    .with_detail(detail)
                    .unwrap()
                    .parse(),
                    Some(context) => match fragment_context(context).and_then(|context| {
                        crate::HtmlFragmentParseRequest::new(
                            crate::HtmlSource::utf8(case.input),
                            context,
                            corpus_limits(),
                        )
                        .map_err(|error| format!("{error:?}"))
                    }) {
                        Ok(request) => request.with_detail(detail).unwrap().parse(),
                        Err(_) => continue,
                    },
                };
                match parsed {
                    Ok(report) => check(&report, &label, &mut tally),
                    Err(_) => tally.failures.push(format!("{label}: parse failed")),
                }
            }
        }
    }
    eprintln!(
        "D2 parses {} nodes {} named {} unnamed {} roots {} unnamed-roots {} links {} failures {}",
        tally.parses,
        tally.nodes,
        tally.named,
        tally.unnamed,
        tally.roots,
        tally.unnamed_roots,
        tally.links,
        tally.failures.len()
    );
    for line in tally.failures.iter().take(30) {
        eprintln!("{line}");
    }
    assert!(tally.failures.is_empty());
}
