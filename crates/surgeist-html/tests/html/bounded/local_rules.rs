use crate::html::support::{Recorder, Refused, Refuser};
use surgeist_html::accounting::HtmlWork;
use surgeist_html::bounded::{
    lexical::{
        HtmlCustomNameScanAction as CustomAction, HtmlPiPseudoAttributeAction as PiAction,
        HtmlPiPseudoAttributeCursor as PiCursor, HtmlPiPseudoAttributeError as PiError,
        HtmlPiPseudoAttributeField as PiField, HtmlPiPseudoAttributeName as PiName,
        HtmlPiPseudoAttributeReply as PiReply, HtmlSpaceTokenCursor, HtmlSpaceTokenStep,
        is_ascii_whitespace, validate_custom_element_name, validate_custom_element_name_with,
    },
    security::is_nonce_element,
    template::{
        HtmlTemplateMarkerFirst as MarkerFirst, HtmlTemplateMarkerSiblings,
        HtmlTemplateMarkerTarget, template_marker_first,
    },
    values::{
        HtmlNameIndex as PiNameIndex, HtmlNameIndexHasher as PiNameHasher,
        HtmlNameIndexLookup as PiLookup, HtmlNameIndexSlot as PiNameSlot,
        HtmlNameIndexTable as PiNameTable,
    },
};
use surgeist_html::values::{
    DocumentString, HtmlCustomElementNameError, HtmlValueInterpreter, Namespace, QualifiedName,
};
use surgeist_ledger::Change;

#[test]
fn script_content_and_preparation_preserve_their_distinct_whitespace_rules() {
    use surgeist_html::{bounded::script::HtmlScriptTypeCursor, elements::script::HtmlScriptKind};

    fn finish(mut cursor: HtmlScriptTypeCursor<'_>) -> Option<HtmlScriptKind> {
        let mut meter = Recorder::default();
        while cursor.result().is_none() {
            cursor.advance(&mut meter).unwrap();
        }
        let result = cursor.result().unwrap();
        let charged = meter.batches().len();
        // A completed cursor charges nothing and stays completed.
        cursor.advance(&mut meter).unwrap();
        assert_eq!(meter.batches().len(), charged);
        assert_eq!(cursor.result(), Some(result));
        result
    }
    let javascript = DocumentString::from(" \tTEXT/JAVASCRIPT\n ");
    assert_eq!(
        finish(HtmlScriptTypeCursor::content(Some(&javascript))),
        None
    );
    assert_eq!(
        finish(HtmlScriptTypeCursor::preparation(Some(&javascript), None)),
        Some(HtmlScriptKind::Classic)
    );
    let module = DocumentString::from("module");
    let padded_module = DocumentString::from(" module ");
    assert_eq!(
        finish(HtmlScriptTypeCursor::preparation(Some(&module), None)),
        Some(HtmlScriptKind::Module)
    );
    assert_eq!(
        finish(HtmlScriptTypeCursor::preparation(
            Some(&padded_module),
            None
        )),
        None
    );
    let language = DocumentString::from("javascript");
    assert_eq!(
        finish(HtmlScriptTypeCursor::preparation(None, Some(&language))),
        Some(HtmlScriptKind::Classic)
    );
    let invalid = DocumentString::from_utf16([0xd800, 0, 0xdc00]);
    assert_eq!(finish(HtmlScriptTypeCursor::content(Some(&invalid))), None);
    assert_eq!(invalid.code_units(), [0xd800, 0, 0xdc00]);
}

#[test]
fn missing_script_type_has_a_fixed_header_action_and_stable_completion() {
    use surgeist_html::{bounded::script::HtmlScriptTypeCursor, elements::script::HtmlScriptKind};

    let mut cursor = HtmlScriptTypeCursor::content(None);
    assert_eq!(cursor.result(), None);
    // A refused charge leaves the cursor at its header action, however often it is refused.
    let mut refuser = Refuser::default();
    for _ in 0..2 {
        assert_eq!(cursor.advance(&mut refuser), Err(Refused));
        assert_eq!(cursor.result(), None);
    }
    assert_eq!(
        refuser.attempts,
        vec![vec![Change::Work(HtmlWork::RecordVisit, 2)]; 2]
    );
    let mut meter = Recorder::default();
    cursor.advance(&mut meter).unwrap();
    assert_eq!(
        meter.batches(),
        [vec![Change::Work(HtmlWork::RecordVisit, 2)]]
    );
    assert_eq!(cursor.result(), Some(Some(HtmlScriptKind::Classic)));
    cursor.advance(&mut meter).unwrap();
    assert_eq!(meter.batches().len(), 1);
    assert_eq!(cursor.result(), Some(Some(HtmlScriptKind::Classic)));
}

#[test]
fn space_tokens_keep_exact_ranges_nulls_surrogates_and_non_ascii_whitespace() {
    let units = [32, 0xd800, 97, 9, 0, 32, 0xa0, 10];
    let mut cursor = HtmlSpaceTokenCursor::new(&units);
    let mut ranges = Vec::new();
    loop {
        match cursor.step() {
            HtmlSpaceTokenStep::Continue => {}
            HtmlSpaceTokenStep::Token { start, end } => ranges.push(start..end),
            HtmlSpaceTokenStep::Done => break,
        }
    }
    assert_eq!(ranges, [1..3, 4..5, 6..7]);
    assert_eq!(&units[ranges[0].clone()], [0xd800, 97]);
    assert_eq!(&units[ranges[1].clone()], [0]);
    assert_eq!(&units[ranges[2].clone()], [0xa0]);
    assert!(!cursor.inspects_unit());
    assert!(matches!(cursor.step(), HtmlSpaceTokenStep::Done));
    for white in [9, 10, 12, 13, 32] {
        assert!(is_ascii_whitespace(white));
    }
    for other in [0, 11, 0xa0, 0xd800] {
        assert!(!is_ascii_whitespace(other));
    }
}

#[test]
fn space_cursor_can_be_inspected_without_advancing_the_next_read() {
    let mut cursor = HtmlSpaceTokenCursor::new(&[97]);
    assert!(cursor.inspects_unit());
    assert!(cursor.inspects_unit());
    assert!(matches!(cursor.step(), HtmlSpaceTokenStep::Continue));
    assert!(!cursor.inspects_unit());
    assert!(matches!(
        cursor.step(),
        HtmlSpaceTokenStep::Token { start: 0, end: 1 }
    ));
    assert!(matches!(cursor.step(), HtmlSpaceTokenStep::Done));
}

type PseudoAttributePairs = Vec<(Vec<u16>, Vec<u16>)>;

fn pseudo_attributes(raw: &str) -> (bool, PseudoAttributePairs) {
    let (valid, pairs, _) = pseudo_attribute_requests(raw);
    (valid, pairs)
}

/// Drives the cursor like a conforming consumer and also counts the requests it made.
fn pseudo_attribute_requests(raw: &str) -> (bool, PseudoAttributePairs, usize) {
    let units: Vec<_> = raw.encode_utf16().collect();
    let mut cursor = PiCursor::new(units.len());
    let mut current = (Vec::<u16>::new(), Vec::<u16>::new());
    let mut pairs = Vec::<(Vec<u16>, Vec<u16>)>::new();
    let mut requests = 0;
    // The consumer's index of committed names: a table at most half full, probed from a keyed hash.
    let hasher = PiNameHasher::new();
    let mut name_index = PiNameIndex::new(PiCursor::name_index_slots(units.len()));
    let mut table: Vec<PiNameSlot> = (0..name_index.slots().get())
        .map(|_| PiNameSlot::default())
        .collect();
    while let Some(action) = cursor.action() {
        requests += 1;
        let reply = match action {
            PiAction::Read(offset) => PiReply::Unit(units.get(offset).copied()),
            PiAction::Step => PiReply::Ack,
            PiAction::Begin(PiField::Name) => {
                current = (Vec::new(), Vec::new());
                PiReply::Ack
            }
            PiAction::Begin(PiField::Value) => PiReply::Ack,
            PiAction::Emit {
                field, units, len, ..
            } => {
                match field {
                    PiField::Name => current.0.extend_from_slice(&units[..len]),
                    PiField::Value => current.1.extend_from_slice(&units[..len]),
                }
                PiReply::Ack
            }
            PiAction::FindName => PiReply::Pair(
                match PiNameTable::new(&mut name_index, &mut table)
                    .unwrap()
                    .find_or_insert(
                        hasher.hash(&current.0),
                        pairs.len(),
                        &mut (),
                        |_| Ok::<_, std::convert::Infallible>(()),
                        |_, pair| Ok(pairs[pair].0 == current.0),
                    )
                    .unwrap()
                {
                    PiLookup::Found(pair) => Some(pair),
                    PiLookup::Inserted => None,
                },
            ),
            PiAction::NameLength(index) => PiReply::Length(pairs[index].0.len()),
            PiAction::NameUnit(name, offset) => PiReply::Unit(match name {
                PiName::Current => current.0.get(offset).copied(),
                PiName::Pair(index) => pairs[index].0.get(offset).copied(),
            }),
            PiAction::Commit => {
                pairs.push(std::mem::take(&mut current));
                PiReply::Ack
            }
            PiAction::Discard => {
                current = (Vec::new(), Vec::new());
                PiReply::Ack
            }
            PiAction::Pop => {
                pairs.pop().unwrap();
                PiReply::Ack
            }
            PiAction::Ready(valid) => {
                cursor.reply(PiReply::Ack).unwrap();
                assert_eq!(cursor.action(), None);
                return (valid, pairs, requests);
            }
        };
        cursor.reply(reply).unwrap();
    }
    panic!("PI grammar must finish with an explicit valid/invalid outcome")
}

#[test]
fn pseudo_attribute_grammar_decodes_references_and_rejects_duplicate_names_atomically() {
    let (valid, pairs) = pseudo_attributes("name='A&amp;B' x=\"&#x1F642;\"");
    assert!(valid);
    assert_eq!(
        pairs,
        [
            (vec![110, 97, 109, 101], vec![65, 38, 66]),
            (vec![120], vec![0xd83d, 0xde42]),
        ]
    );
    assert_eq!(pseudo_attributes("name='a' name='b'"), (false, Vec::new()));
    assert_eq!(pseudo_attributes("name='unterminated"), (false, Vec::new()));
    assert_eq!(pseudo_attributes(""), (true, Vec::new()));
}

/// The parsing result is an error if two pseudo-attributes have the same name
/// (references/xml-stylesheet-pseudo-attributes--REC-xml-stylesheet-20101028--a0e1a5198a21.md:168),
/// so each new name needs a lookup among the committed names rather than a request pair per
/// committed pair: the requests of a scan of distinct names must grow linearly with its size.
#[test]
fn pseudo_attribute_requests_grow_linearly_with_distinct_names() {
    let requests: Vec<usize> = [250, 500, 1000, 2000]
        .into_iter()
        .map(|count| {
            let raw: String = (0..count).map(|n| format!("n{n}='' ")).collect();
            let (valid, pairs, requests) = pseudo_attribute_requests(&raw);
            assert!(valid);
            assert_eq!(pairs.len(), count);
            requests
        })
        .collect();
    for pair in requests.windows(2) {
        assert!(
            pair[1] * 10 <= pair[0] * 22,
            "doubling the pairs grew the requests from {} to {}: {requests:?}",
            pair[0],
            pair[1]
        );
    }
}

/// Two pseudo-attributes with the same name make the parsing result an error; names are exact XML
/// Names, so case and every unit matter.
#[test]
fn pseudo_attribute_duplicates_are_exactly_the_repeated_names() {
    let names = [
        "a",
        "A",
        "b",
        "ab",
        "aB",
        "a-b",
        "\u{e9}",
        "\u{c9}",
        "x:y",
        "\u{1F600}",
    ];
    let mut state = 0x853c_49e6_748f_ea9b_u64;
    let mut next = move |bound: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % bound as u64) as usize
    };
    for _ in 0..500 {
        let chosen: Vec<&str> = (0..next(9)).map(|_| names[next(names.len())]).collect();
        let raw = chosen
            .iter()
            .map(|name| format!("{name}='v'"))
            .collect::<Vec<_>>()
            .join(" ");
        let distinct = chosen
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            == chosen.len();
        let (valid, pairs) = pseudo_attributes(&raw);
        assert_eq!(valid, distinct, "{raw}");
        assert_eq!(
            pairs.len(),
            if distinct { chosen.len() } else { 0 },
            "{raw}"
        );
    }
}

#[test]
fn pseudo_attribute_wrong_reply_preserves_the_original_pending_action() {
    let mut cursor = PiCursor::new(0);
    assert_eq!(cursor.action(), Some(PiAction::Read(0)));
    assert_eq!(cursor.reply(PiReply::Ack), Err(PiError::Integrity));
    assert_eq!(cursor.action(), Some(PiAction::Read(0)));
    cursor.reply(PiReply::Unit(None)).unwrap();
    assert_eq!(cursor.action(), Some(PiAction::Step));
    cursor.reply(PiReply::Ack).unwrap();
    assert_eq!(cursor.action(), Some(PiAction::Ready(true)));
    cursor.reply(PiReply::Ack).unwrap();
    assert_eq!(cursor.action(), None);
    assert_eq!(cursor.reply(PiReply::Ack), Err(PiError::Integrity));
    assert_eq!(cursor.action(), None);
}

#[test]
fn template_marker_selection_and_nested_end_detection_are_local_transitions() {
    use HtmlTemplateMarkerTarget::{End, Marker, Other, Start};
    assert_eq!(template_marker_first(Marker, false), MarkerFirst::Continue);
    assert_eq!(template_marker_first(Marker, true), MarkerFirst::Marker);
    assert_eq!(template_marker_first(Start, true), MarkerFirst::Start);
    assert_eq!(template_marker_first(End, true), MarkerFirst::Continue);
    let mut siblings = HtmlTemplateMarkerSiblings::default();
    for target in [Start, Other, Start, Marker, End, End] {
        assert!(!siblings.inspect(target).unwrap());
    }
    assert!(siblings.inspect(End).unwrap());
}

#[test]
fn custom_name_validation_retains_exact_units_and_stops_on_admission_refusal() {
    let exact = DocumentString::from_utf16(vec![120, 45, 0xd800, 0]);
    assert!(matches!(
        validate_custom_element_name(&exact),
        Err(HtmlCustomElementNameError::ForbiddenCodeUnit {
            code_unit_offset: 3,
            code_unit: 0
        })
    ));
    let allowed = DocumentString::from_utf16(vec![120, 45, 0xd800]);
    assert_eq!(validate_custom_element_name(&allowed), Ok(()));
    assert_eq!(allowed.code_units(), [120, 45, 0xd800]);
    let mut calls = 0;
    assert_eq!(
        validate_custom_element_name_with(&allowed, |action| {
            calls += 1;
            assert!(matches!(action, CustomAction::Header));
            Err::<(), _>("refused before first unit")
        }),
        Err("refused before first unit")
    );
    assert_eq!(calls, 1);
    for reserved in ["annotation-xml", "font-face"] {
        assert_eq!(
            validate_custom_element_name(&reserved.into()),
            Err(HtmlCustomElementNameError::ReservedName)
        );
    }
}

#[test]
fn nonce_applicability_uses_exact_namespace_and_no_host_assumption() {
    for namespace in [Namespace::html(), Namespace::svg(), Namespace::mathml()] {
        assert!(is_nonce_element(&QualifiedName::new(
            namespace,
            Some("raw-prefix".into()),
            "anything"
        )));
    }
    assert!(!is_nonce_element(&QualifiedName::new(
        Namespace::none(),
        None,
        "script"
    )));
    assert!(!is_nonce_element(&QualifiedName::new(
        Namespace::from_uri("http://www.w3.org/1999/XHTML"),
        None,
        "script",
    )));
}

#[test]
fn raw_admission_counts_original_utf16_before_a_rule_can_inspect_it() {
    let interpreter = HtmlValueInterpreter::new(2);
    assert!(interpreter.admit_attribute_raw(None).is_ok());
    assert!(
        interpreter
            .admit_attribute_raw(Some(&DocumentString::from_utf16(vec![0xd800, 0])))
            .is_ok()
    );
    assert!(
        interpreter
            .admit_attribute_raw(Some(&DocumentString::from_utf16(vec![0xd800, 0, 32])))
            .is_err()
    );
}
