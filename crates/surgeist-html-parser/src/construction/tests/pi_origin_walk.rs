use super::*;
use crate::construction::provenance::PiOriginWalk;

/// The mapped origins of each piece as `(output start, output end, source start, source end)`.
fn pieces(
    markup: &str,
    walks: &[(std::ops::Range<usize>, std::ops::Range<usize>)],
) -> Vec<Vec<(usize, usize, usize, usize)>> {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8(markup);
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    session.initialize_parse_evidence(&mut c, &source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let TokenKind::ProcessingInstruction { target, data } = lexical_kind(&mut lexer, &mut c) else {
        panic!("literal PI");
    };
    let node = session
        .allocate_processing_instruction(&mut c, &mut Some((target, data)), None)
        .unwrap();
    let storage = session.active().unwrap();
    let mut walk = PiOriginWalk::new(node.index);
    let mut mapped = Vec::new();
    for (input, output) in walks {
        let mut maps = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
        storage
            .pi_piece_origins(
                &mut c,
                &mut walk,
                input.clone(),
                output.clone(),
                false,
                &mut maps,
            )
            .unwrap();
        mapped.push(
            maps.as_slice()
                .iter()
                .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                .collect(),
        );
        maps.release(&mut c).unwrap();
    }
    walk.release(&mut c).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    mapped
}

#[test]
fn a_walk_resumed_across_pieces_maps_each_to_the_source_of_its_units() {
    // The data `a='b'` starts at source offset 9, after `<?marker `. The name `a` is data
    // unit 0 and the value `b` is data unit 3; each piece maps to its own source unit.
    let markup = "<?marker a='b'?>";
    let mapped = pieces(markup, &[(0..1, 0..1), (3..4, 0..1)]);
    assert_eq!(mapped, vec![vec![(0, 1, 9, 10)], vec![(0, 1, 12, 13)]]);
}

#[test]
fn a_piece_that_starts_before_the_end_of_the_previous_one_is_mapped_from_the_first_origin() {
    // The cursor of the pseudo-attribute grammar only reads forward, but the walk keeps the
    // same answer when a piece repeats ground the walk has passed.
    let markup = "<?marker a='b'?>";
    let mapped = pieces(markup, &[(3..4, 0..1), (0..1, 0..1), (3..4, 0..1)]);
    assert_eq!(
        mapped,
        vec![
            vec![(0, 1, 12, 13)],
            vec![(0, 1, 9, 10)],
            vec![(0, 1, 12, 13)]
        ]
    );
}

/// Runs `check` on a parsed processing instruction `<?marker a='b'?>`, then releases everything.
fn with_instruction(check: impl FnOnce(&mut ParseControl, &mut ParserArenaSession, usize)) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<?marker a='b'?>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    session.initialize_parse_evidence(&mut c, &source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let TokenKind::ProcessingInstruction { target, data } = lexical_kind(&mut lexer, &mut c) else {
        panic!("literal PI");
    };
    let node = session
        .allocate_processing_instruction(&mut c, &mut Some((target, data)), None)
        .unwrap();
    check(&mut c, &mut session, node.index);
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
}

#[test]
fn the_target_and_data_of_a_processing_instruction_have_groups_and_no_registry_row() {
    with_instruction(|c, session, node| {
        let storage = session.active_mut().unwrap();
        for field in [
            FieldRef::ProcessingInstructionTarget(node),
            FieldRef::ProcessingInstructionData(node),
        ] {
            assert!(storage.payloads.detail_origins.has(field));
            // The string is the record's own: no row holds it.
            assert_eq!(
                storage.field_slot(c, field),
                Err(SessionError::registry_row())
            );
            assert!(!storage.payloads.values.iter().any(|row| matches!(
                row,
                RetainedPayload::String { location, .. } if *location == field
            )));
        }
    });
}

#[test]
fn a_join_naming_another_row_is_not_the_attribute_map_of_a_processing_instruction() {
    with_instruction(|c, session, node| {
        let storage = session.active_mut().unwrap();
        assert_eq!(
            storage.pi_attribute(c, node, "a").unwrap(),
            Some(&surgeist_html::values::DocumentString::from("b"))
        );
        let good = storage.payloads.node_joins[node];
        *storage.payloads.node_joins[node]
            .rows
            .get_mut(Join::ProcessingInstructionAttributes) = Some(joined_cell(0).unwrap());
        assert_eq!(
            storage.pi_attribute(c, node, "a").err(),
            Some(SessionError::ProcessingInstructionMapUnavailable)
        );
        storage.payloads.node_joins[node] = good;
    });
}
