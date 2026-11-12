use super::*;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlDocumentMode},
    values::DocumentString,
};

#[test]
fn implied_head_and_both_root_attribute_paths_keep_order_exact_backing_and_origins() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    for implied in [false, true] {
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
            "<html id=old><html id=ignored x:id=p a='"
                .encode_utf16()
                .chain([0xd800, 0xd83d, 0xde00])
                .chain("' a=last z=end>".encode_utf16())
                .collect::<Vec<_>>(),
        ));
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
            panic!("first tag")
        };
        let root = if implied {
            let root = session.allocate_implicit_html(&mut c).unwrap();
            session
                .add_attributes_if_missing(&mut c, &root, &mut Some(first))
                .unwrap();
            root
        } else {
            session.allocate_html_tag(&mut c, &mut Some(first)).unwrap()
        };
        let doc = session.root(&mut c).unwrap();
        session.insert(&mut c, &doc, &root, None, None).unwrap();
        let head = session.allocate_implicit_head(&mut c, None).unwrap();
        session.insert(&mut c, &root, &head, None, None).unwrap();
        let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
            panic!("second tag")
        };
        assert!(second.had_duplicate_attributes);
        let expected_origin = second.attributes.as_slice()[2].value.origins.as_slice()[0]
            .source
            .clone();
        let mut second = Some(second);
        session
            .add_attributes_if_missing(&mut c, &root, &mut second)
            .unwrap();
        assert!(second.is_none());
        let storage = session.active().unwrap();
        let ConstructionNodeKind::Element(element) = &storage
            .construction
            .node_at(root.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("root")
        };
        let held = test_support::attributes_of(element);
        assert_eq!(
            held.iter()
                .map(|a| a.name().local_name().to_utf8().unwrap())
                .collect::<Vec<_>>(),
            ["id", "x:id", "a", "z"]
        );
        assert_eq!(test_support::value_text(held[0]), "old");
        assert_eq!(test_support::value_units(held[2]), [0xd800, 0xd83d, 0xde00]);
        assert!(held.iter().all(|a| a.name().namespace().uri().is_none()));
        let group = storage
            .payloads
            .detail_origins
            .get(&mut c, FieldRef::AttributeValue(root.index, 2))
            .unwrap()
            .expect("the origin group of the merged value");
        assert_eq!(
            storage.observed_origins(group, &mut c).unwrap()[0].source,
            expected_origin
        );
        assert_eq!(storage.construction.usage().attributes(), 4);
        t.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let document = frozen.document().unwrap();
        let root_id = document.root().children().next().unwrap();
        let surgeist_html::construction::HtmlNodeKind::Element(element) =
            document.node(&root_id).unwrap().kind()
        else {
            panic!("root")
        };
        assert_eq!(element.attributes().len(), 4);
        let head_id = document.node(&root_id).unwrap().children().next().unwrap();
        let surgeist_html::construction::HtmlNodeKind::Element(head) =
            document.node(&head_id).unwrap().kind()
        else {
            panic!("head")
        };
        assert_eq!(head.name().local_name().to_utf8().unwrap(), "head");
        assert!(head.attributes().is_empty());
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

/// Parses `first` and then merges the attributes of `second` into its root, returning the
/// session, the root and the lexer for the test to inspect and release.
fn merge_fixture<'source>(
    source: &'source crate::HtmlSource,
    c: &mut ParseControl,
) -> (
    ParserArenaSession,
    ParserNode,
    Option<crate::parser::tokenizer::token::Tag>,
    crate::parser::tokenizer::tests::LexerFixture<'source>,
) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    c.admit_source(source).unwrap();
    let mut tokenizer = LexerFixture::new(source, HtmlTokenizerState::Data, None, c).unwrap();
    let mut session = ParserArenaSession::new(c, RootKind::Fragment).unwrap();
    let TokenKind::Tag(first) = lexical_kind(&mut tokenizer, c) else {
        panic!("actual first tag")
    };
    let root = session.allocate_html_tag(c, &mut Some(first)).unwrap();
    let TokenKind::Tag(second) = lexical_kind(&mut tokenizer, c) else {
        panic!("actual second tag")
    };
    (session, root, Some(second), tokenizer)
}

fn root_attributes(session: &ParserArenaSession, root: &ParserNode) -> Vec<(String, String)> {
    let ConstructionNodeKind::Element(element) = &session
        .active()
        .unwrap()
        .construction
        .node_at(root.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("actual root")
    };
    test_support::attributes_of(element)
        .into_iter()
        .map(|a| {
            (
                a.name().local_name().to_utf8().unwrap(),
                test_support::value_text(a),
            )
        })
        .collect()
}

#[test]
fn a_wide_root_merge_appends_one_list_in_source_order_with_no_spare_room() {
    // Sixteen is a power of two and seventeen is not, so a list that doubled while it grew would
    // end with room to spare in the second case.
    for count in [16_usize, 17] {
        let attributes = (0..count)
            .map(|i| format!(" a{i:02}=v"))
            .collect::<String>();
        let source = crate::HtmlSource::utf8(format!("<html><html{attributes}>"));
        let mut c = control();
        let (mut session, root, mut second, tokenizer) = merge_fixture(&source, &mut c);
        session
            .add_attributes_if_missing(&mut c, &root, &mut second)
            .unwrap();
        assert!(second.is_none());
        assert_eq!(c.stop(), None);
        let held = root_attributes(&session, &root);
        assert_eq!(held.len(), count);
        for (i, (name, value)) in held.iter().enumerate() {
            assert_eq!(name, &format!("a{i:02}"));
            assert_eq!(value, "v");
        }
        // The run was empty, so one reservation of exactly the kept attributes held them all.
        let dimensions =
            test_support::run_dimensions(session.active_mut().unwrap(), &mut c, root.index);
        assert_eq!(
            (dimensions.initialized_len, dimensions.capacity),
            (count, count)
        );
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn a_root_merge_of_only_repeated_names_changes_nothing_and_holds_nothing() {
    let source = crate::HtmlSource::utf8("<html a=1 b=2><html a=3 b=4>");
    let mut c = control();
    let (mut session, root, mut second, tokenizer) = merge_fixture(&source, &mut c);
    session
        .add_attributes_if_missing(&mut c, &root, &mut second)
        .unwrap();
    assert!(second.is_none());
    assert_eq!(
        root_attributes(&session, &root),
        [("a".into(), "1".into()), ("b".into(), "2".into())]
    );
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn every_work_cut_of_a_merge_leaves_a_prefix_in_source_order_and_cleans_up() {
    let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
    let mut c = control();
    let (mut session, root, mut input, tokenizer) = merge_fixture(&source, &mut c);
    let setup = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session
        .add_attributes_if_missing(&mut c, &root, &mut input)
        .unwrap();
    let envelope = c.usage().resource(HtmlParseResource::WorkCredits).current - setup;
    assert!(
        envelope <= 65536,
        "finite test campaign bound, not a correctness oracle"
    );
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    let expected = [("id", "old"), ("a", "one"), ("b", "two")];
    let mut lengths = std::collections::BTreeSet::new();
    for cut in 0..envelope {
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .work_credits(setup + cut)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        let (mut session, root, mut input, tokenizer) = merge_fixture(&source, &mut c);
        let result = session.add_attributes_if_missing(&mut c, &root, &mut input);
        assert!(matches!(
            result,
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }))
        ));
        // Whatever was appended is a prefix, in source order, of the expected list.
        let held = root_attributes(&session, &root);
        assert!((1..=3).contains(&held.len()));
        for (actual, (name, value)) in held.iter().zip(expected) {
            assert_eq!((actual.0.as_str(), actual.1.as_str()), (name, value));
        }
        lengths.insert(held.len());
        let stop = c.stop();
        if let Some(tag) = input {
            let mut kind = Some(TokenKind::Tag(tag));
            if release_token_kind(&mut kind, &mut c).is_err() {
                release_token_kind(&mut kind, &mut c).unwrap();
            }
        }
        tokenizer.release(&mut c).unwrap();
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }
    assert!(lengths.contains(&1), "a cut before the first append");
    assert!(lengths.contains(&2), "a cut between the two appends");
}

/// `count` attributes named `{prefix}00` to `{prefix}{count-1}`, as a tag carries them.
fn wide_attributes(prefix: &str, count: usize) -> String {
    (0..count).map(|i| format!(" {prefix}{i:02}={i}")).collect()
}

/// Limits roomy enough for two tags of forty attributes each.
fn wide_limits() -> crate::HtmlParseLimitsBuilder {
    limits()
        .into_builder()
        .source_map_entries(20_000)
        .auxiliary_entries(20_000)
        .active_formatting_entries(20_000)
}

fn wide_names(prefix: &str, count: usize) -> Vec<String> {
    (0..count).map(|i| format!("{prefix}{i:02}")).collect()
}

#[test]
fn sampled_work_cuts_of_a_merge_through_the_live_name_index_leave_a_prefix_and_clean_up() {
    // Forty attributes on the root and forty more merged into it, so that the table of names the
    // storage edits long lists through is in use for the plan and every append.
    let source = crate::HtmlSource::utf8(format!(
        "<html{}><html{}>",
        wide_attributes("a", 40),
        wide_attributes("b", 40)
    ));
    let mut c = ParseControl::new(
        wide_limits().finish().unwrap(),
        HtmlParseCancellation::new(),
    );
    let (mut session, root, mut input, tokenizer) = merge_fixture(&source, &mut c);
    let setup = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session
        .add_attributes_if_missing(&mut c, &root, &mut input)
        .unwrap();
    let envelope = c.usage().resource(HtmlParseResource::WorkCredits).current - setup;
    let expected: Vec<String> = wide_names("a", 40)
        .into_iter()
        .chain(wide_names("b", 40))
        .collect();
    assert_eq!(
        root_attributes(&session, &root)
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>(),
        expected
    );
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    let stride = (envelope / 400).max(1);
    let mut lengths = std::collections::BTreeSet::new();
    for cut in (0..envelope).step_by(usize::try_from(stride).unwrap()) {
        let mut c = ParseControl::new(
            wide_limits().work_credits(setup + cut).finish().unwrap(),
            HtmlParseCancellation::new(),
        );
        let (mut session, root, mut input, tokenizer) = merge_fixture(&source, &mut c);
        let result = session.add_attributes_if_missing(&mut c, &root, &mut input);
        assert!(matches!(
            result,
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }))
        ));
        let held: Vec<String> = root_attributes(&session, &root)
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        assert!(
            (40..=80).contains(&held.len()) && held[..] == expected[..held.len()],
            "a cut at {cut} leaves a prefix in source order: {held:?}"
        );
        lengths.insert(held.len());
        let stop = c.stop();
        if let Some(tag) = input {
            let mut kind = Some(TokenKind::Tag(tag));
            if release_token_kind(&mut kind, &mut c).is_err() {
                release_token_kind(&mut kind, &mut c).unwrap();
            }
        }
        tokenizer.release(&mut c).unwrap();
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }
    assert!(
        lengths.len() > 2,
        "the sampled cuts fall before, between and after the appends"
    );
}

#[test]
fn a_removal_between_merges_of_a_wide_root_keeps_the_next_merge_exact_at_every_sampled_cut() {
    let source = crate::HtmlSource::utf8(format!(
        "<html{}><html{}><html a07=again c00=x>",
        wide_attributes("a", 40),
        wide_attributes("b", 4)
    ));
    // The second merge goes in uncut, the attribute `a07` is removed, and the third merge adds
    // it back after the others. The removal and the third merge run under each sampled cut.
    let run = |c: &mut ParseControl| {
        let (mut session, root, mut second, mut tokenizer) = merge_fixture(&source, c);
        session
            .add_attributes_if_missing(c, &root, &mut second)
            .unwrap();
        let TokenKind::Tag(third) = lexical_kind(&mut tokenizer, c) else {
            panic!("third tag")
        };
        let before_removal = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let mut third = Some(third);
        let outcome = (|| {
            session
                .active_mut()
                .unwrap()
                .remove_managed_field(c, root.index, "a07")?;
            session.add_attributes_if_missing(c, &root, &mut third)
        })();
        (session, root, tokenizer, third, before_removal, outcome)
    };
    let mut baseline = ParseControl::new(
        wide_limits().finish().unwrap(),
        HtmlParseCancellation::new(),
    );
    let (mut session, root, tokenizer, third, setup, outcome) = run(&mut baseline);
    outcome.unwrap();
    assert!(third.is_none());
    let envelope = baseline
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - setup;
    let expected: Vec<String> = wide_names("a", 40)
        .into_iter()
        .filter(|name| name != "a07")
        .chain(wide_names("b", 4))
        .chain(["a07".to_owned(), "c00".to_owned()])
        .collect();
    let names = |session: &ParserArenaSession, root: &ParserNode| {
        root_attributes(session, root)
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>()
    };
    assert_eq!(names(&session, &root), expected);
    tokenizer.release(&mut baseline).unwrap();
    session.release(&mut baseline).unwrap();
    storage_zero(&baseline);
    let stride = (envelope / 300).max(1);
    for cut in (0..envelope).step_by(usize::try_from(stride).unwrap()) {
        let mut c = ParseControl::new(
            wide_limits().work_credits(setup + cut).finish().unwrap(),
            HtmlParseCancellation::new(),
        );
        let (mut session, root, tokenizer, third, _, outcome) = run(&mut c);
        assert!(
            outcome.is_err(),
            "a cut at {cut} stops the removal or the merge"
        );
        let held = names(&session, &root);
        let unique: std::collections::BTreeSet<_> = held.iter().collect();
        assert_eq!(
            unique.len(),
            held.len(),
            "a cut at {cut} repeated a name: {held:?}"
        );
        assert!(held.len() >= 43, "a cut at {cut} lost attributes: {held:?}");
        if let Some(tag) = third {
            let mut kind = Some(TokenKind::Tag(tag));
            if release_token_kind(&mut kind, &mut c).is_err() {
                release_token_kind(&mut kind, &mut c).unwrap();
            }
        }
        tokenizer.release(&mut c).unwrap();
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
    }
}

#[test]
fn cancellation_before_a_prepared_merge_installs_leaves_the_list_and_cleans_up() {
    let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let (mut session, root, mut input, tokenizer) = merge_fixture(&source, &mut c);
    let prepared = session
        .active_mut()
        .unwrap()
        .prepare_attribute_merge(&mut c, &root, &mut input)
        .unwrap()
        .unwrap();
    assert!(input.is_none());
    cancellation.cancel();
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    assert_eq!(
        session
            .active_mut()
            .unwrap()
            .finish_attribute_merge(&mut c, prepared),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(
        root_attributes(&session, &root),
        [("id".into(), "old".into())]
    );
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}

#[test]
fn merge_rejects_a_foreign_origin_before_consuming_input() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<html id=x><html a=y>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
        panic!("tag")
    };
    let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
    let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
        panic!("tag")
    };
    let mut incoming = Some(second);
    // An origin from another immutable source is refused before the input is taken.
    // The actual original slot remains owned by caller.
    let foreign_source = crate::HtmlSource::utf8("other");
    let original = mem::replace(
        &mut incoming
            .as_mut()
            .unwrap()
            .attributes
            .get_mut(0, &mut c)
            .unwrap()
            .unwrap()
            .source,
        foreign_source.range(0, 5).unwrap(),
    );
    assert!(matches!(
        session.add_attributes_if_missing(&mut c, &root, &mut incoming),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignSource
        )))
    ));
    assert!(incoming.is_some());
    // Terminal disposal validates real ownership rather than source content.
    let _retained_original = original;
    let mut kind = incoming.take().map(TokenKind::Tag);
    release_token_kind(&mut kind, &mut c).unwrap();
    t.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn staged_attribute_cap_failure_preserves_root_and_reclaims_exact_real_slots() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<html id=old><html a=one z=two>");
    let mut c = ParseControl::new(
        limits()
            .into_builder()
            .managed_attributes(2)
            .finish()
            .unwrap(),
        crate::HtmlParseCancellation::new(),
    );
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
        panic!("tag")
    };
    let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
    let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
        panic!("tag")
    };
    let mut input = Some(second);
    assert!(matches!(
        session.add_attributes_if_missing(&mut c, &root, &mut input),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedAttributes,
            ..
        }))
    ));
    assert!(input.is_none());
    let ConstructionNodeKind::Element(element) = &session
        .active()
        .unwrap()
        .construction
        .node_at(root.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("root")
    };
    // The attributes are appended in source order: the limit admits the first of them and
    // refuses the second.
    let held = test_support::attributes_of(element);
    assert_eq!(held.len(), 2);
    assert_eq!(test_support::value_text(held[0]), "old");
    assert_eq!(held[1].name().local_name().to_utf8().unwrap(), "a");
    assert_eq!(test_support::value_text(held[1]), "one");
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        2
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::TokenAttributes)
            .current,
        0
    );
    let stop = c.stop();
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    t.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), stop);
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .high_water,
        2
    );
}

#[test]
fn merge_foreign_control_and_cancel_leave_the_real_lexical_owner_untaken() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    for cancel in [false, true] {
        let source = crate::HtmlSource::utf8("<html a=x>");
        let cancellation = crate::HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.allocate_implicit_html(&mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(tag);
        if cancel {
            cancellation.cancel();
            assert_eq!(
                session.add_attributes_if_missing(&mut c, &root, &mut input),
                Err(SessionError::Stop(HtmlParseStop::Cancelled))
            );
        } else {
            let mut foreign = control();
            foreign.admit_source(&source).unwrap();
            assert!(matches!(
                session.add_attributes_if_missing(&mut foreign, &root, &mut input),
                Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
                    crate::parser::control::HtmlParseInvariant::ForeignControl
                )))
            ));
        }
        assert!(input.is_some());
        let ConstructionNodeKind::Element(element) = &session
            .active()
            .unwrap()
            .construction
            .node_at(root.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("root")
        };
        assert!(test_support::attributes_of(element).is_empty());
        let mut kind = input.map(TokenKind::Tag);
        release_token_kind(&mut kind, &mut c).unwrap();
        t.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn merge_work_boundaries_retain_record_registry_owners_and_cleanup_all_staged_storage() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
    let run = |cap: Option<u64>| {
        let selected = match cap {
            Some(cap) => limits().into_builder().work_credits(cap).finish().unwrap(),
            None => limits(),
        };
        let mut c = ParseControl::new(selected, crate::HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
        let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let setup = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let lexical_name = second.name.value.text().code_units().as_ptr();
        let mut input = Some(second);
        let result = session.add_attributes_if_missing(&mut c, &root, &mut input);
        let completed = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let ConstructionNodeKind::Element(element) = &session
            .active()
            .unwrap()
            .construction
            .node_at(root.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("root")
        };
        let held = test_support::attributes_of(element);
        assert_eq!(test_support::value_text(held[0]), "old");
        assert!((1..=3).contains(&held.len()));
        for (actual, (name, value)) in held.iter().zip([("id", "old"), ("a", "one"), ("b", "two")])
        {
            assert_eq!(actual.name().local_name().to_utf8().unwrap(), name);
            assert_eq!(test_support::value_text(*actual), value);
        }
        if let Some(tag) = &input {
            assert_eq!(tag.name.value.text().code_units().as_ptr(), lexical_name);
            assert_eq!(tag.attributes.len(), 2);
            for (attribute, (name, value)) in tag
                .attributes
                .as_slice()
                .iter()
                .zip([("a", "one"), ("b", "two")])
            {
                assert_eq!(attribute.name.value.text().to_utf8().unwrap(), name);
                assert_eq!(attribute.value.value.text().to_utf8().unwrap(), value);
            }
        }
        match result {
            Err(error) => assert!(matches!(
                error,
                SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                })
            )),
            Ok(()) => {
                assert_eq!(held.len(), 3);
                assert!(input.is_none());
            }
        }
        let stop = c.stop();
        if let Some(tag) = input {
            let mut kind = Some(TokenKind::Tag(tag));
            if release_token_kind(&mut kind, &mut c).is_err() {
                release_token_kind(&mut kind, &mut c).unwrap();
            }
        }
        t.release(&mut c).unwrap();
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        if stop.is_some() {
            assert_eq!(c.stop(), stop);
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                completed
            );
        }
        (setup, completed)
    };
    let (setup, complete) = run(None);
    let envelope = complete - setup;
    eprintln!("controlled complete named merge coverage envelope: {envelope}");
    // This bounds harness runtime, not the semantic tariff. Retained
    // representation growth and newly admitted hooks can increase the real
    // complete envelope; every actual setup..complete cut is still tested.
    assert!(
        envelope <= 65536,
        "finite coverage bound; literal fields/backings are the oracle"
    );
    // Every actual work cut spans borrowed planning (Tag still occupied),
    // witnessed decisions, registration, reserve, appends and late rebind.
    for cap in setup..complete {
        run(Some(cap));
    }
}

/// The table keys of the name and value of each of `count` attributes of the element at
/// `node`, in attribute order.
fn name_and_value_keys(node: usize, count: usize) -> Vec<(usize, usize, u8)> {
    (0..count)
        .flat_map(|position| [(node, position, 0), (node, position, 1)])
        .collect()
}

#[test]
fn the_origin_table_names_the_name_and_value_of_each_attribute_in_attribute_order() {
    let source = crate::HtmlSource::utf8("<html id=old><html a=1 b=2 c=3>");
    let mut c = control();
    let (mut session, root, mut second, tokenizer) = merge_fixture(&source, &mut c);
    // The element was created with one attribute, whose two strings have origins.
    assert_eq!(
        session.active().unwrap().payloads.detail_origins.keys(),
        name_and_value_keys(root.index, 1)
    );
    session
        .add_attributes_if_missing(&mut c, &root, &mut second)
        .unwrap();
    // The merge added three attributes, so the table holds the two strings of four.
    assert_eq!(
        session.active().unwrap().payloads.detail_origins.keys(),
        name_and_value_keys(root.index, 4)
    );
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn removing_an_attribute_moves_the_origins_of_the_attributes_after_it_down_one_place() {
    let source = crate::HtmlSource::utf8("<html id=old><html a=1 b=2 c=3>");
    let mut c = control();
    let (mut session, root, mut second, tokenizer) = merge_fixture(&source, &mut c);
    session
        .add_attributes_if_missing(&mut c, &root, &mut second)
        .unwrap();
    let origins_of = |session: &mut ParserArenaSession, c: &mut ParseControl, position: usize| {
        let storage = session.active().unwrap();
        let group = storage
            .payloads
            .detail_origins
            .get(c, FieldRef::AttributeValue(root.index, position))
            .unwrap()
            .expect("the origin group of the value");
        storage
            .observed_origins(group, c)
            .unwrap()
            .iter()
            .map(|o| (o.start, o.end, o.source.clone(), o.kind))
            .collect::<Vec<_>>()
    };
    let last_value_origins = origins_of(&mut session, &mut c, 3);
    session
        .active_mut()
        .unwrap()
        .remove_managed_field(&mut c, root.index, "b")
        .unwrap();
    assert_eq!(
        root_attributes(&session, &root),
        [
            ("id".into(), "old".into()),
            ("a".into(), "1".into()),
            ("c".into(), "3".into())
        ]
    );
    assert_eq!(
        session.active().unwrap().payloads.detail_origins.keys(),
        name_and_value_keys(root.index, 3)
    );
    // The value of `c` was the fourth attribute's and is the third's now, with its origins.
    assert_eq!(origins_of(&mut session, &mut c, 2), last_value_origins);
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_parse_that_records_no_graph_holds_no_origin_table_and_the_same_attributes() {
    let source = crate::HtmlSource::utf8("<html id=old><html a=1 b=2 c=3>");
    let mut with_detail = control();
    let (mut detailed, detailed_root, mut second, tokenizer) =
        merge_fixture(&source, &mut with_detail);
    detailed
        .add_attributes_if_missing(&mut with_detail, &detailed_root, &mut second)
        .unwrap();
    let expected = root_attributes(&detailed, &detailed_root);
    assert_eq!(detailed.active().unwrap().payloads.detail_origins.len(), 8);
    tokenizer.release(&mut with_detail).unwrap();
    detailed.release(&mut with_detail).unwrap();
    storage_zero(&with_detail);

    let mut without = control_without_detail();
    let (mut plain, plain_root, mut second, tokenizer) = merge_fixture(&source, &mut without);
    plain
        .add_attributes_if_missing(&mut without, &plain_root, &mut second)
        .unwrap();
    assert_eq!(root_attributes(&plain, &plain_root), expected);
    assert!(
        plain
            .active()
            .unwrap()
            .payloads
            .detail_origins
            .is_unallocated()
    );
    plain
        .active_mut()
        .unwrap()
        .remove_managed_field(&mut without, plain_root.index, "b")
        .unwrap();
    assert!(
        plain
            .active()
            .unwrap()
            .payloads
            .detail_origins
            .is_unallocated()
    );
    tokenizer.release(&mut without).unwrap();
    plain.release(&mut without).unwrap();
    storage_zero(&without);
}

#[test]
fn the_origin_table_adds_one_auxiliary_entry_per_origin_bearing_field() {
    let source = crate::HtmlSource::utf8("<html id=old><html a=1 b=2 c=3>");
    let aux = |c: &ParseControl| {
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current
    };
    let mut plain = control_without_detail();
    let (mut without, root, mut second, tokenizer) = merge_fixture(&source, &mut plain);
    without
        .add_attributes_if_missing(&mut plain, &root, &mut second)
        .unwrap();
    let without_detail = aux(&plain);
    tokenizer.release(&mut plain).unwrap();
    without.release(&mut plain).unwrap();

    let mut c = control();
    let (mut session, root, mut second, tokenizer) = merge_fixture(&source, &mut c);
    session
        .add_attributes_if_missing(&mut c, &root, &mut second)
        .unwrap();
    let table = &session.active().unwrap().payloads.detail_origins;
    // The name and the value of each of the four attributes have a group.
    assert_eq!(table.len(), 8);
    assert_eq!(aux(&c), without_detail + 8);
    // The retained bytes are a slot for each node position up to the root's, one run for the
    // root and its eight entries, each at most doubled by the spare room of its vector.
    let slot = std::mem::size_of::<usize>();
    let run = std::mem::size_of::<super::super::detail_origins::Run>();
    let entry = std::mem::size_of::<super::super::detail_origins::DetailOrigin>();
    assert!(table.capacity_bytes() <= 2 * ((root.index + 1) * slot + run + 8 * entry));
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
