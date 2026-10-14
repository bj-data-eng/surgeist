use surgeist_html::bounded::{
    HtmlCustomNameScanAction as CustomAction, HtmlPiPseudoAttributeAction as PiAction,
    HtmlPiPseudoAttributeCursor as PiCursor, HtmlPiPseudoAttributeError as PiError,
    HtmlPiPseudoAttributeField as PiField, HtmlPiPseudoAttributeName as PiName,
    HtmlPiPseudoAttributeReply as PiReply, HtmlSpaceTokenCursor, HtmlSpaceTokenStep,
    HtmlTemplateMarkerFirst as MarkerFirst, HtmlTemplateMarkerSiblings, HtmlTemplateMarkerTarget,
    is_ascii_whitespace, is_nonce_element, template_marker_first, validate_custom_element_name,
    validate_custom_element_name_with,
};
use surgeist_html::{
    DocumentString, HtmlCustomElementNameError, HtmlValueInterpreter, Namespace, QualifiedName,
};

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
    let units: Vec<_> = raw.encode_utf16().collect();
    let mut cursor = PiCursor::new(units.len());
    let mut current = (Vec::<u16>::new(), Vec::<u16>::new());
    let mut pairs = Vec::<(Vec<u16>, Vec<u16>)>::new();
    while let Some(action) = cursor.action() {
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
                return (valid, pairs);
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
