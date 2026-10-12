use super::*;
use crate::{HtmlParseCancellation, HtmlParseLimits, HtmlParseResource};
use states::{Raw, State};

/// Actual lexer/evidence owners under one admitted controller. This fixture
/// supplies no token, diagnostic, fee, callback or semantic expectation.
pub(crate) struct LexerFixture<'a> {
    lexer: Tokenizer<'a>,
    evidence: ParseEvidence,
}
impl<'a> LexerFixture<'a> {
    pub(crate) fn new(
        source: &'a HtmlSource,
        state: State,
        last: Option<&DocumentString>,
        c: &mut ParseControl,
    ) -> Result<Self, ParseStop> {
        c.require_source(source)?;
        let evidence = ParseEvidence::prepare(c).map_err(|error| match error {
            crate::construction::parser_session::SessionError::Stop(stop) => stop,
            _ => unreachable!("initializer has only checked admission failures"),
        })?;
        match Tokenizer::new(source, state, last, c) {
            Ok(lexer) => Ok(Self { lexer, evidence }),
            Err(stop) => {
                evidence.release(c)?;
                Err(stop)
            }
        }
    }
    pub(crate) fn next(
        &mut self,
        c: &mut ParseControl,
        cdata: bool,
    ) -> Result<Option<Event>, ParseStop> {
        self.lexer.next(c, cdata, &mut self.evidence)
    }
    pub(crate) fn next_in(
        &mut self,
        c: &mut ParseControl,
        cdata: bool,
        evidence: &mut ParseEvidence,
    ) -> Result<Option<Event>, ParseStop> {
        self.lexer.next(c, cdata, evidence)
    }
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.lexer.release(c)?;
        self.evidence.release(c)
    }
    fn diagnostic(
        &self,
        id: &crate::construction::parser_session::evidence::DiagnosticId,
    ) -> crate::construction::parser_session::evidence::TokenizerDiagnosticView<'_> {
        self.evidence.tokenizer_observation(id)
    }
    fn step(&mut self, c: &mut ParseControl, cdata: bool) -> Result<(), ParseStop> {
        self.lexer.step(c, cdata, &mut self.evidence)
    }
    fn read(&mut self, c: &mut ParseControl) -> Result<Option<u32>, ParseStop> {
        self.lexer.read(c, &mut self.evidence)
    }
    fn finish_attribute(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.lexer.finish_attribute(c, &mut self.evidence)
    }
    fn check_attribute_name(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.lexer.check_attribute_name(c, &mut self.evidence)
    }
}
impl<'a> std::ops::Deref for LexerFixture<'a> {
    type Target = Tokenizer<'a>;
    fn deref(&self) -> &Self::Target {
        &self.lexer
    }
}
impl std::ops::DerefMut for LexerFixture<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lexer
    }
}

#[test]
fn head_handoff_keeps_the_real_pause_and_accepts_returned_eof() {
    let source = HtmlSource::utf8("<html>A");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let event = t.next(&mut c, false).unwrap().unwrap();
    assert!(
        matches!(&event, Event::Token(value) if matches!(&value.token().kind, TokenKind::Tag(_)))
    );
    event.release(&mut c).unwrap();
    let last = t.last_start.len();
    t.validate_head_handoff(&source, &mut c).unwrap();
    assert_eq!(t.last_start.len(), last);
    let event = t.next(&mut c, false).unwrap().unwrap();
    assert!(
        matches!(&event, Event::Token(value) if matches!(&value.token().kind, TokenKind::Text(text) if text.value.text().code_units() == [65]))
    );
    event.release(&mut c).unwrap();
    let event = t.next(&mut c, false).unwrap().unwrap();
    assert!(matches!(&event, Event::Token(value) if matches!(&value.token().kind, TokenKind::Eof)));
    t.validate_head_handoff(&source, &mut c).unwrap();
    assert!(t.done);
    event.release(&mut c).unwrap();
    t.release(&mut c).unwrap();
}

#[test]
fn head_handoff_rejects_foreign_owner_raw_state_and_pending_without_consumption() {
    let source = HtmlSource::utf8("\0");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Raw(Raw::Rawtext), None, &mut c).unwrap();
    assert_eq!(
        t.validate_head_handoff(&source, &mut c),
        Err(TokenizerHandoffError::NotData)
    );
    assert_eq!(t.state, State::Raw(Raw::Rawtext));
    t.set_state(State::Data, &mut c).unwrap();
    let event = t.next(&mut c, false).unwrap().unwrap();
    assert!(t.pending_len > 0);
    let pending = t.pending_len;
    assert_eq!(
        t.validate_head_handoff(&source, &mut c),
        Err(TokenizerHandoffError::PendingEvents)
    );
    assert_eq!(t.pending_len, pending);
    event.release(&mut c).unwrap();
    while let Some(event) = t.next(&mut c, false).unwrap() {
        event.release(&mut c).unwrap();
    }
    let mut foreign = ParseControl::new(limits(), HtmlParseCancellation::new());
    foreign.admit_source(&source).unwrap();
    assert_eq!(
        t.validate_head_handoff(&source, &mut foreign),
        Err(TokenizerHandoffError::Stop(ParseStop::InternalInvariant(
            Invariant::ForeignControl
        )))
    );
    assert!(t.done);
    t.release(&mut c).unwrap();
}

#[test]
fn head_handoff_work_cancel_and_foreign_source_preserve_the_borrowed_tokenizer() {
    let source = HtmlSource::utf8("A");
    let mut probe = ParseControl::new(limits(), HtmlParseCancellation::new());
    probe.admit_source(&source).unwrap();
    let tokenizer = LexerFixture::new(&source, State::Data, None, &mut probe).unwrap();
    let setup_work = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    tokenizer.release(&mut probe).unwrap();
    for reason in 0..3 {
        let cancellation = HtmlParseCancellation::new();
        let selected = if reason == 0 {
            limits()
                .into_builder()
                .work_credits(setup_work + 1)
                .finish()
                .unwrap()
        } else {
            limits()
        };
        let mut c = ParseControl::new(selected, cancellation.clone());
        c.admit_source(&source).unwrap();
        let t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        if reason == 0 {
            assert!(matches!(
                t.validate_head_handoff(&source, &mut c),
                Err(TokenizerHandoffError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
        } else if reason == 1 {
            cancellation.cancel();
            assert_eq!(
                t.validate_head_handoff(&source, &mut c),
                Err(TokenizerHandoffError::Stop(ParseStop::Cancelled))
            );
        } else {
            let foreign = HtmlSource::utf8("A");
            assert_eq!(
                t.validate_head_handoff(&foreign, &mut c),
                Err(TokenizerHandoffError::Stop(ParseStop::InternalInvariant(
                    Invariant::ForeignSource
                )))
            );
        }
        assert_eq!(t.state, State::Data);
        assert_eq!(t.pending_len, 0);
        assert!(!t.done);
        t.release(&mut c).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }
}

fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100_000)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}
#[derive(Debug, Eq, PartialEq)]
enum Seen {
    Text(Vec<u16>),
    Tag(TagKind, Vec<u16>, bool, Vec<(Vec<u16>, Vec<u16>)>, bool),
    Comment(Vec<u16>),
    Doctype(Option<Vec<u16>>, Option<Vec<u16>>, Option<Vec<u16>>, bool),
    Pi(Vec<u16>, Vec<u16>),
    Eof,
}
fn units(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn text(t: &token::Text) -> Vec<u16> {
    t.value.text().code_units().to_vec()
}
fn collect(
    source: &HtmlSource,
    state: State,
    last: Option<&str>,
    cdata: bool,
) -> (Vec<Seen>, Vec<DiagnosticCode>) {
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(source).unwrap();
    let last = last.map(DocumentString::from);
    let mut tokenizer = LexerFixture::new(source, state, last.as_ref(), &mut c).unwrap();
    let mut output = Vec::new();
    let mut errors = Vec::new();
    while let Some(event) = tokenizer.next(&mut c, cdata).unwrap() {
        match &event {
            Event::Diagnostic(d) => {
                let diagnostic = tokenizer.diagnostic(d);
                assert_eq!(diagnostic.source.source(), source.identity());
                let _state = diagnostic.state;
                assert_eq!(
                    diagnostic.phase,
                    if matches!(
                        diagnostic.code,
                        DiagnosticCode::SurrogateInInputStream
                            | DiagnosticCode::NoncharacterInInputStream
                            | DiagnosticCode::ControlCharacterInInputStream
                    ) {
                        token::DiagnosticPhase::InputStream
                    } else {
                        token::DiagnosticPhase::Tokenizer
                    }
                );
                errors.push(diagnostic.code);
            }
            Event::Preprocessing {
                record,
                source: range,
                ..
            } => {
                assert_eq!(range.source(), source.identity());
                assert!(matches!(
                    record,
                    super::super::input::PreprocessingRecord::Newline { .. }
                        | super::super::input::PreprocessingRecord::Diagnostic(_)
                ));
            }
            Event::Token(t) => {
                let t = t.token();
                assert_eq!(t.source.source(), source.identity());
                output.push(match &t.kind {
                    TokenKind::Text(t) => Seen::Text(text(t)),
                    TokenKind::Comment(t) => Seen::Comment(text(t)),
                    TokenKind::Tag(t) => Seen::Tag(
                        t.kind,
                        text(&t.name),
                        t.self_closing,
                        t.attributes
                            .as_slice()
                            .iter()
                            .map(|a| {
                                assert_eq!(a.source.source(), source.identity());
                                (text(&a.name), text(&a.value))
                            })
                            .collect(),
                        t.had_duplicate_attributes,
                    ),
                    TokenKind::Doctype(t) => Seen::Doctype(
                        t.name.as_ref().map(text),
                        t.public_id.as_ref().map(text),
                        t.system_id.as_ref().map(text),
                        t.force_quirks,
                    ),
                    TokenKind::ProcessingInstruction { target, data } => {
                        Seen::Pi(text(target), text(data))
                    }
                    TokenKind::Eof => Seen::Eof,
                });
            }
        }
        event.release(&mut c).unwrap();
    }
    tokenizer.release(&mut c).unwrap();
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::SourceMapEntries,
        HtmlParseResource::Diagnostics,
        HtmlParseResource::ManagedCapacityBytes,
    ] {
        assert_eq!(c.usage().resource(resource).current, 0, "{resource:?}");
    }
    (output, errors)
}
fn joined(seen: &[Seen]) -> Vec<u16> {
    seen.iter()
        .filter_map(|s| {
            if let Seen::Text(t) = s {
                Some(t.as_slice())
            } else {
                None
            }
        })
        .flatten()
        .copied()
        .collect()
}
#[test]
fn generic_tags_attributes_case_duplicates_and_source_only_names() {
    let (out, errors) = collect(
        &HtmlSource::utf8("<ObSoLeTe X='a&amp;b' x=discard =bad a\"b=x />"),
        State::Data,
        None,
        false,
    );
    assert_eq!(
        out,
        vec![
            Seen::Tag(
                TagKind::Start,
                units("obsolete"),
                true,
                vec![
                    (units("x"), units("a&b")),
                    (units("=bad"), vec![]),
                    (units("a\"b"), units("x"))
                ],
                true
            ),
            Seen::Eof
        ]
    );
    assert_eq!(
        errors,
        vec![
            DiagnosticCode::DuplicateAttribute,
            DiagnosticCode::UnexpectedEqualsSignBeforeAttributeName,
            DiagnosticCode::UnexpectedCharacterInAttributeName
        ]
    );
}
#[test]
fn references_use_longest_match_two_scalar_values_and_attribute_exception() {
    let (out, errors) = collect(
        &HtmlSource::utf8(
            "&notit; &NotEqualTilde; &Afr; &unknown; <a x='&notit; &amp= &amp; &notin;'>",
        ),
        State::Data,
        None,
        false,
    );
    assert_eq!(joined(&out), units("¬it; ≂\u{338} 𝔄 &unknown; "));
    assert!(
        matches!(&out[out.len()-2],Seen::Tag(_,_,_,attrs,_) if attrs==&vec![(units("x"),units("&notit; &amp= & ∉"))])
    );
    assert_eq!(
        errors,
        vec![
            DiagnosticCode::MissingSemicolonAfterCharacterReference,
            DiagnosticCode::UnknownNamedCharacterReference
        ]
    );
}
#[test]
fn numeric_references_recover_with_distinct_codes() {
    let (out, errors) = collect(
        &HtmlSource::utf8("&#0; &#xD800; &#1114112; &#x80; &#x81; &#xFDD0; &#13; &#65 &#x;"),
        State::Data,
        None,
        false,
    );
    assert_eq!(joined(&out), units("� � � € \u{81} \u{fdd0} \r A &#x;"));
    assert_eq!(
        errors,
        vec![
            DiagnosticCode::NullCharacterReference,
            DiagnosticCode::SurrogateCharacterReference,
            DiagnosticCode::CharacterReferenceOutsideUnicodeRange,
            DiagnosticCode::ControlCharacterReference,
            DiagnosticCode::ControlCharacterReference,
            DiagnosticCode::NoncharacterCharacterReference,
            DiagnosticCode::ControlCharacterReference,
            DiagnosticCode::MissingSemicolonAfterCharacterReference,
            DiagnosticCode::AbsenceOfDigitsInNumericCharacterReference
        ]
    );
}
#[test]
fn all_five_pi_states_reserved_recovery_question_marks_and_eof_omission() {
    let (out, errors) = collect(
        &HtmlSource::utf8("<?_X-2 \t a??><?xml a?><?XML-stylesheet b><?a:b><?1?><?x unfinished"),
        State::Data,
        None,
        false,
    );
    assert_eq!(
        out,
        vec![
            Seen::Pi(units("_X-2"), units("a?")),
            Seen::Comment(units("?xml a?")),
            Seen::Comment(units("?XML-stylesheet b")),
            Seen::Comment(units("?a:b")),
            Seen::Comment(units("?1?")),
            Seen::Eof
        ]
    );
    assert_eq!(
        errors,
        vec![
            DiagnosticCode::DisallowedProcessingInstructionTarget,
            DiagnosticCode::DisallowedProcessingInstructionTarget,
            DiagnosticCode::InvalidProcessingInstructionTarget,
            DiagnosticCode::InvalidFirstCharacterOfProcessingInstructionTarget,
            DiagnosticCode::EofInProcessingInstruction
        ]
    );
}
#[test]
fn literal_non_scalars_bom_and_newlines_remain_exact_and_state_specific() {
    let source = HtmlSource::utf16(DocumentString::from_utf16(vec![
        0xFEFF, 0xD800, 13, 10, 0, 0xD83D, 0xDE42, 0xDC00, 0xFEFF,
    ]));
    let (out, errors) = collect(&source, State::Data, None, false);
    assert_eq!(
        joined(&out),
        [0xFEFF, 0xD800, 10, 0, 0xD83D, 0xDE42, 0xDC00, 0xFEFF]
    );
    assert_eq!(
        errors,
        vec![
            DiagnosticCode::SurrogateInInputStream,
            DiagnosticCode::UnexpectedNullCharacter,
            DiagnosticCode::SurrogateInInputStream
        ]
    );
    let (out, _) = collect(&source, State::Plaintext, None, false);
    assert_eq!(
        joined(&out),
        [0xFEFF, 0xD800, 10, 0xFFFD, 0xD83D, 0xDE42, 0xDC00, 0xFEFF]
    );
    let pi = HtmlSource::utf16(DocumentString::from_utf16(vec![
        60, 63, 120, 32, 0, 0xD800, 62,
    ]));
    let (out, errors) = collect(&pi, State::Data, None, false);
    assert_eq!(out, vec![Seen::Pi(units("x"), vec![0, 0xD800]), Seen::Eof]);
    assert_eq!(errors, vec![DiagnosticCode::SurrogateInInputStream]);
}
#[test]
fn raw_rcdata_appropriate_end_tags_and_fallback_preserve_case() {
    for kind in [Raw::Rcdata, Raw::Rawtext] {
        let (out, errors) = collect(
            &HtmlSource::utf8("A&amp;</TiTlEextra!></TiTlE>tail"),
            State::Raw(kind),
            Some("title"),
            false,
        );
        assert_eq!(
            joined(&out),
            units(if kind == Raw::Rcdata {
                "A&</TiTlEextra!>tail"
            } else {
                "A&amp;</TiTlEextra!>tail"
            })
        );
        assert!(out.iter().any(|s|matches!(s,Seen::Tag(TagKind::End,n,false,attrs,false) if n==&units("title")&&attrs.is_empty())));
        assert!(errors.is_empty());
    }
}
#[test]
fn script_escaped_and_double_escaped_states_do_not_close_inner_script() {
    let (out, errors) = collect(
        &HtmlSource::utf8("<!--<script>inner</script>--></script>after"),
        State::Raw(Raw::Script),
        Some("script"),
        false,
    );
    assert_eq!(joined(&out), units("<!--<script>inner</script>-->after"));
    assert_eq!(out.iter().filter(|s| matches!(s, Seen::Tag(..))).count(), 1);
    assert!(errors.is_empty());
    let (_, errors) = collect(
        &HtmlSource::utf8("<!--unfinished"),
        State::Raw(Raw::Script),
        Some("script"),
        false,
    );
    assert_eq!(errors, vec![DiagnosticCode::EofInScriptHtmlCommentLikeText]);
}
#[test]
fn comments_nested_and_abrupt_recovery_and_cdata_are_distinct() {
    let (out, errors) = collect(
        &HtmlSource::utf8("<!--><!--x<!--y--!><!x><![CDATA[z]]>"),
        State::Data,
        None,
        false,
    );
    assert_eq!(
        out,
        vec![
            Seen::Comment(vec![]),
            Seen::Comment(units("x<!--y")),
            Seen::Comment(units("x")),
            Seen::Comment(units("[CDATA[z]]")),
            Seen::Eof
        ]
    );
    assert_eq!(
        errors,
        vec![
            DiagnosticCode::AbruptClosingOfEmptyComment,
            DiagnosticCode::NestedComment,
            DiagnosticCode::IncorrectlyClosedComment,
            DiagnosticCode::IncorrectlyOpenedComment,
            DiagnosticCode::CdataInHtmlContent
        ]
    );
    let (out, errors) = collect(
        &HtmlSource::utf8("<![CDATA[a\0]]]>"),
        State::Data,
        None,
        true,
    );
    assert_eq!(joined(&out), units("a\0]"));
    assert!(errors.is_empty());
}
#[test]
fn doctype_optional_empty_identifiers_quirks_and_recovery() {
    let (out, errors) = collect(
        &HtmlSource::utf8(
            "<!DOCTYPE HTML PUBLIC \"\" 'sys'><!DOCTYPE x SYSTEM\"y\"><!DOCTYPE><!DOCTYPE z PUBLIC nope>",
        ),
        State::Data,
        None,
        false,
    );
    assert_eq!(
        out,
        vec![
            Seen::Doctype(Some(units("html")), Some(vec![]), Some(units("sys")), false),
            Seen::Doctype(Some(units("x")), None, Some(units("y")), false),
            Seen::Doctype(None, None, None, true),
            Seen::Doctype(Some(units("z")), None, None, true),
            Seen::Eof
        ]
    );
    assert_eq!(
        errors,
        vec![
            DiagnosticCode::MissingWhitespaceAfterDoctypeSystemKeyword,
            DiagnosticCode::MissingDoctypeName,
            DiagnosticCode::MissingQuoteBeforeDoctypePublicIdentifier
        ]
    );
}
#[test]
fn spans_cover_original_crlf_utf8_and_reference_spelling() {
    let source = HtmlSource::utf8("é\r\n&NotEqualTilde;");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let mut records = Vec::new();
    while let Some(event) = t.next(&mut c, false).unwrap() {
        if let Some(Token {
            kind: TokenKind::Text(text),
            source,
            ..
        }) = event.token()
        {
            records.push((
                text.value.text().code_units().to_vec(),
                source.start(),
                source.end(),
                text.origins
                    .as_slice()
                    .iter()
                    .map(|o| (o.start, o.end, o.source.start(), o.source.end(), o.kind))
                    .collect::<Vec<_>>(),
            ));
        }
        event.release(&mut c).unwrap();
    }
    t.release(&mut c).unwrap();
    assert_eq!(
        records,
        vec![
            (units("é"), 0, 2, vec![(0, 1, 0, 2, OriginKind::Literal)]),
            (units("\n"), 2, 4, vec![(0, 1, 2, 4, OriginKind::Newline)]),
            (
                units("≂\u{338}"),
                4,
                19,
                vec![
                    (0, 1, 4, 19, OriginKind::CharacterReference),
                    (1, 2, 4, 19, OriginKind::CharacterReference)
                ]
            )
        ]
    );
}
#[test]
fn cancellation_and_terminal_stop_never_return_a_later_token() {
    let source = HtmlSource::utf8("<a x='large'>");
    let cancel = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancel.clone());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    cancel.cancel();
    assert!(matches!(t.next(&mut c, false), Err(ParseStop::Cancelled)));
    assert!(matches!(t.next(&mut c, false), Err(ParseStop::Cancelled)));
    t.release(&mut c).unwrap();
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current,
        0
    );
}

fn run_with_limits(
    source: &HtmlSource,
    state: State,
    last: Option<&DocumentString>,
    caps: HtmlParseLimits,
) -> (Option<ParseStop>, u64) {
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    if c.admit_source(source).is_ok()
        && let Ok(mut tokenizer) = LexerFixture::new(source, state, last, &mut c)
    {
        loop {
            match tokenizer.next(&mut c, true) {
                Ok(Some(event)) => {
                    event.release(&mut c).unwrap();
                    if c.stop().is_some() {
                        break;
                    }
                }
                Ok(None) => break,
                Err(first) => {
                    assert_eq!(c.stop(), Some(first));
                    assert!(matches!(tokenizer.next(&mut c,true),Err(second) if second==first));
                    break;
                }
            }
        }
        tokenizer.release(&mut c).unwrap();
    }
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::SourceMapEntries,
        HtmlParseResource::Diagnostics,
        HtmlParseResource::ManagedCapacityBytes,
    ] {
        assert_eq!(
            c.usage().resource(resource).current,
            0,
            "leak {resource:?}; stop {:?}; work cap {}",
            c.stop(),
            caps.work_credits()
        );
    }
    (
        c.stop(),
        c.usage().resource(HtmlParseResource::WorkCredits).current,
    )
}
#[test]
fn every_work_credit_boundary_cleans_nested_payloads_without_partial_later_success() {
    // Separate ownership paths: character emission, dual replacement, ambiguous
    // text, tag attributes, doctype triple freeze, PI pair, raw fallback.
    for (s, state, last) in [
        ("x", State::Data, None),
        ("&acE;", State::Data, None),
        ("&zzz;", State::Data, None),
        ("<a x='v'>", State::Data, None),
        ("<!DOCTYPE h PUBLIC 'p' 's'>", State::Data, None),
        ("<?x a?>", State::Data, None),
        ("<![CDATA[]]]]x", State::Data, None),
        ("</n!", State::Raw(Raw::Rcdata), Some("a")),
    ] {
        let source = HtmlSource::utf8(s);
        let last = last.map(DocumentString::from);
        let (stop, used) = run_with_limits(&source, state, last.as_ref(), limits());
        assert_eq!(stop, None);
        for cap in 0..used {
            let mut caps = limits();
            caps = caps.into_builder().work_credits(cap).finish().unwrap();
            let (stop, _) = run_with_limits(&source, state, last.as_ref(), caps);
            assert!(
                matches!(
                    stop,
                    Some(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ),
                "{s} cap {cap}"
            );
        }
        let mut exact = limits();
        exact = exact.into_builder().work_credits(used).finish().unwrap();
        assert_eq!(
            run_with_limits(&source, state, last.as_ref(), exact).0,
            None
        );
    }
}
#[test]
fn every_retained_tokenizer_domain_has_a_real_finite_failure() {
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::SourceMapEntries,
        HtmlParseResource::Diagnostics,
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::PeakTransientBytes,
        HtmlParseResource::AtomicLayoutBytes,
    ] {
        let source = HtmlSource::utf8("<a x='&zzz;' x='discard'>");
        let mut caps = limits();
        match resource {
            HtmlParseResource::BufferedUtf16Units => {
                caps = caps
                    .into_builder()
                    .buffered_utf16_units(0)
                    .finish()
                    .unwrap()
            }
            HtmlParseResource::TokenAttributes => {
                caps = caps.into_builder().token_attributes(0).finish().unwrap()
            }
            HtmlParseResource::AuxiliaryEntries => {
                caps = caps.into_builder().auxiliary_entries(0).finish().unwrap()
            }
            HtmlParseResource::SourceMapEntries => {
                caps = caps.into_builder().source_map_entries(0).finish().unwrap()
            }
            HtmlParseResource::Diagnostics => {
                caps = caps.into_builder().diagnostics(0).finish().unwrap()
            }
            HtmlParseResource::ManagedCapacityBytes => {
                caps = caps
                    .into_builder()
                    .managed_capacity_bytes(0)
                    .finish()
                    .unwrap()
            }
            HtmlParseResource::PeakTransientBytes => {
                caps = caps
                    .into_builder()
                    .peak_transient_bytes(0)
                    .finish()
                    .unwrap()
            }
            HtmlParseResource::AtomicLayoutBytes => {
                caps = caps.into_builder().atomic_layout_bytes(0).finish().unwrap()
            }
            _ => unreachable!(),
        }
        assert!(
            matches!(run_with_limits(&source,State::Data,None,caps).0,Some(ParseStop::LimitExceeded{resource:r,..}) if r==resource),
            "{resource:?}"
        );
    }
}
#[test]
fn tree_state_selection_after_actual_start_tag_is_functional() {
    let source = HtmlSource::utf8("<title>&amp;</title>");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let tag = tokenizer.next(&mut c, false).unwrap().unwrap();
    assert!(
        matches!(tag.token(),Some(Token{kind:TokenKind::Tag(t),..}) if t.kind==TagKind::Start&&text(&t.name)==units("title"))
    );
    tag.release(&mut c).unwrap();
    tokenizer
        .set_state(State::Raw(Raw::Rcdata), &mut c)
        .unwrap();
    let event = tokenizer.next(&mut c, false).unwrap().unwrap();
    assert!(matches!(event.token(),Some(Token{kind:TokenKind::Text(t),..}) if text(t)==units("&")));
    event.release(&mut c).unwrap();
    while let Some(event) = tokenizer.next(&mut c, false).unwrap() {
        event.release(&mut c).unwrap();
    }
    tokenizer.release(&mut c).unwrap();
}
#[test]
fn eof_matrix_keeps_token_omission_and_doctype_comment_recovery_distinct() {
    for (s, code, token) in [
        (
            "<",
            DiagnosticCode::EofBeforeTagName,
            Some(Seen::Text(units("<"))),
        ),
        (
            "</",
            DiagnosticCode::EofBeforeTagName,
            Some(Seen::Text(units("</"))),
        ),
        ("<a x=", DiagnosticCode::EofInTag, None),
        (
            "<!--x--",
            DiagnosticCode::EofInComment,
            Some(Seen::Comment(units("x"))),
        ),
        (
            "<!DOCTYPE",
            DiagnosticCode::EofInDoctype,
            Some(Seen::Doctype(None, None, None, true)),
        ),
        (
            "<!DOCTYPE h PUBLIC 'x",
            DiagnosticCode::EofInDoctype,
            Some(Seen::Doctype(
                Some(units("h")),
                Some(units("x")),
                None,
                true,
            )),
        ),
        ("<?", DiagnosticCode::EofInProcessingInstruction, None),
        ("<?x", DiagnosticCode::EofInProcessingInstruction, None),
        ("<?x ?", DiagnosticCode::EofInProcessingInstruction, None),
    ] {
        let (out, errors) = collect(&HtmlSource::utf8(s), State::Data, None, false);
        assert_eq!(errors, vec![code], "{s}");
        let mut expected = token.into_iter().collect::<Vec<_>>();
        expected.push(Seen::Eof);
        assert_eq!(out, expected, "{s}");
    }
}
#[test]
fn malformed_attribute_and_doctype_diagnostic_sites_are_named() {
    for (s, codes) in [
        ("<a x=>", vec![DiagnosticCode::MissingAttributeValue]),
        (
            "<a x='v'y=\"q\">",
            vec![DiagnosticCode::MissingWhitespaceBetweenAttributes],
        ),
        ("<a /x=y>", vec![DiagnosticCode::UnexpectedSolidusInTag]),
        (
            "<a x=a`b>",
            vec![DiagnosticCode::UnexpectedCharacterInUnquotedAttributeValue],
        ),
        ("</a x=v/>", vec![DiagnosticCode::EndTagWithAttributes]),
        (
            "</a x=v />",
            vec![
                DiagnosticCode::EndTagWithAttributes,
                DiagnosticCode::EndTagWithTrailingSolidus,
            ],
        ),
        (
            "<!DOCTYPEh>",
            vec![DiagnosticCode::MissingWhitespaceBeforeDoctypeName],
        ),
        (
            "<!DOCTYPE h WHAT>",
            vec![DiagnosticCode::InvalidCharacterSequenceAfterDoctypeName],
        ),
        (
            "<!DOCTYPE h PUBLIC\"x\">",
            vec![DiagnosticCode::MissingWhitespaceAfterDoctypePublicKeyword],
        ),
        (
            "<!DOCTYPE h PUBLIC>",
            vec![DiagnosticCode::MissingDoctypePublicIdentifier],
        ),
        (
            "<!DOCTYPE h SYSTEM >",
            vec![DiagnosticCode::MissingDoctypeSystemIdentifier],
        ),
        (
            "<!DOCTYPE h SYSTEM x>",
            vec![DiagnosticCode::MissingQuoteBeforeDoctypeSystemIdentifier],
        ),
        (
            "<!DOCTYPE h PUBLIC 'x>",
            vec![DiagnosticCode::AbruptDoctypePublicIdentifier],
        ),
        (
            "<!DOCTYPE h SYSTEM \"x>",
            vec![DiagnosticCode::AbruptDoctypeSystemIdentifier],
        ),
        (
            "<!DOCTYPE h PUBLIC 'x'\"y\">",
            vec![DiagnosticCode::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers],
        ),
        (
            "<!DOCTYPE h SYSTEM 'x' bogus>",
            vec![DiagnosticCode::UnexpectedCharacterAfterDoctypeSystemIdentifier],
        ),
    ] {
        assert_eq!(
            collect(&HtmlSource::utf8(s), State::Data, None, false).1,
            codes,
            "{s}"
        );
    }
    assert!(
        collect(
            &HtmlSource::utf8("&; & <a x='&; &'>"),
            State::Data,
            None,
            false
        )
        .1
        .is_empty()
    );
}
#[test]
fn input_diagnostics_and_reconsume_newline_origins_are_emitted_once() {
    let source = HtmlSource::utf16(DocumentString::from_utf16(vec![
        60, 33, 45, 45, 13, 10, 0xD800, 0xFDD0, 1, 45, 45, 62,
    ]));
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let mut errors = Vec::new();
    let mut newlines = 0;
    while let Some(event) = tokenizer.next(&mut c, false).unwrap() {
        match &event {
            Event::Diagnostic(d) => {
                let diagnostic = tokenizer.diagnostic(d);
                assert_eq!(diagnostic.phase, token::DiagnosticPhase::InputStream);
                errors.push(diagnostic.code);
            }
            Event::Preprocessing { record, .. } => {
                assert_eq!(
                    *record,
                    super::super::input::PreprocessingRecord::Newline { was_crlf: true }
                );
                newlines += 1;
            }
            Event::Token(event) => {
                if let TokenKind::Comment(t) = &event.token().kind {
                    assert_eq!(text(t), [10, 0xD800, 0xFDD0, 1]);
                    let o = &t.origins.as_slice()[0];
                    assert_eq!(
                        (o.source.start(), o.source.end(), o.kind),
                        (4, 6, OriginKind::Newline)
                    );
                }
            }
        }
        event.release(&mut c).unwrap();
    }
    tokenizer.release(&mut c).unwrap();
    assert_eq!(newlines, 1);
    assert_eq!(
        errors,
        vec![
            DiagnosticCode::SurrogateInInputStream,
            DiagnosticCode::NoncharacterInInputStream,
            DiagnosticCode::ControlCharacterInInputStream
        ]
    );
    let mut caps = limits();
    caps = caps.into_builder().diagnostics(0).finish().unwrap();
    assert!(matches!(
        run_with_limits(&source, State::Data, None, caps).0,
        Some(ParseStop::LimitExceeded {
            resource: HtmlParseResource::Diagnostics,
            ..
        })
    ));
}
#[test]
fn duplicate_name_exit_at_eof_precedes_eof_in_tag() {
    let (out, errors) = collect(&HtmlSource::utf8("<a x=first X"), State::Data, None, false);
    assert_eq!(out, vec![Seen::Eof]);
    assert_eq!(
        errors,
        vec![DiagnosticCode::DuplicateAttribute, DiagnosticCode::EofInTag]
    );
}
#[test]
fn cancellation_discards_a_prepared_token_behind_a_real_diagnostic() {
    let source = HtmlSource::utf8("\0");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let event = t.next(&mut c, false).unwrap().unwrap();
    assert!(
        matches!(&event,Event::Diagnostic(d) if t.diagnostic(d).code==DiagnosticCode::UnexpectedNullCharacter)
    );
    event.release(&mut c).unwrap();
    cancellation.cancel();
    assert!(matches!(t.next(&mut c, false), Err(ParseStop::Cancelled)));
    t.release(&mut c).unwrap();
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current,
        0
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        0
    );
}
#[test]
fn attribute_occurrence_spans_exclude_separating_whitespace() {
    let source = HtmlSource::utf8("<a X = 'v'  y >");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let event = t.next(&mut c, false).unwrap().unwrap();
    let Some(Token {
        kind: TokenKind::Tag(tag),
        ..
    }) = event.token()
    else {
        panic!("tag")
    };
    let a = tag.attributes.as_slice();
    assert_eq!((a[0].source.start(), a[0].source.end()), (3, 10));
    assert_eq!((a[1].source.start(), a[1].source.end()), (12, 13));
    event.release(&mut c).unwrap();
    while let Some(event) = t.next(&mut c, false).unwrap() {
        event.release(&mut c).unwrap();
    }
    t.release(&mut c).unwrap();
}

#[test]
fn consumer_transfer_releases_outer_slot_and_preserves_nested_payload() {
    let source = HtmlSource::utf16(DocumentString::from_utf16(vec![0xD800]));
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let event = t.next(&mut c, false).unwrap().unwrap();
    assert!(matches!(event, Event::Diagnostic(_)));
    event.release(&mut c).unwrap();
    let Event::Token(event) = t.next(&mut c, false).unwrap().unwrap() else {
        panic!("text token")
    };
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        1
    );
    let retained_units = c
        .usage()
        .resource(HtmlParseResource::BufferedUtf16Units)
        .current;
    let token = event.into_token(&mut c).unwrap();
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        0
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        retained_units
    );
    let TokenKind::Text(value) = &token.kind else {
        panic!("text payload")
    };
    assert_eq!(text(value), [0xD800]);
    token.release(&mut c).unwrap();
    while let Some(event) = t.next(&mut c, false).unwrap() {
        event.release(&mut c).unwrap();
    }
    t.release(&mut c).unwrap();
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        0
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current,
        0
    );
}

#[test]
fn token_slot_admission_failure_releases_prepared_exact_text() {
    let source = HtmlSource::utf8("a");
    let mut cap = limits();
    cap = cap.into_builder().auxiliary_entries(0).finish().unwrap();
    // The runner verifies every retained resource has returned to zero.
    let (stop, _) = run_with_limits(&source, State::Data, None, cap);
    assert!(matches!(
        stop,
        Some(ParseStop::LimitExceeded {
            resource: HtmlParseResource::AuxiliaryEntries,
            ..
        })
    ));
}

#[test]
fn repeated_cdata_brackets_keep_the_two_pending_original_spans() {
    for input in ["<![CDATA[]]]]x", "<![CDATA[]]]]", "<![CDATA[]]]]>"] {
        let source = HtmlSource::utf8(input);
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut observed = Vec::new();
        while let Some(event) = t.next(&mut c, true).unwrap() {
            if let Some(Token {
                kind: TokenKind::Text(text),
                ..
            }) = event.token()
            {
                observed.push((
                    text.value.text().code_units().to_vec(),
                    text.origins
                        .as_slice()
                        .iter()
                        .map(|o| (o.source.start(), o.source.end()))
                        .collect::<Vec<_>>(),
                ));
            }
            event.release(&mut c).unwrap();
        }
        t.release(&mut c).unwrap();
        assert_eq!(observed[0], (units("]"), vec![(9, 10)]));
        assert_eq!(observed[1], (units("]"), vec![(10, 11)]));
        if input.ends_with('>') {
            assert_eq!(observed.len(), 2);
        } else {
            assert_eq!(observed[2], (units("]]"), vec![(11, 13), (11, 13)]));
        }
    }
}

fn assert_storage_released(c: &ParseControl) {
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::SourceMapEntries,
        HtmlParseResource::Diagnostics,
        HtmlParseResource::ManagedCapacityBytes,
    ] {
        assert_eq!(c.usage().resource(resource).current, 0, "{resource:?}");
    }
}

#[test]
fn emitted_text_accounts_for_each_actual_utf8_endpoint_read() {
    // The cursor reads the scalar payload and its interior end once. The
    // tokenizer's independently constructed envelope checks that end again.
    for (input, expected_reads, expected_text) in [("ab", 3, "a"), ("éx", 4, "é"), ("aé", 3, "a")]
    {
        let source = HtmlSource::utf8(input);
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let before = c.usage().work(WorkCategory::InputUnitRead);
        let event = t.next(&mut c, false).unwrap().unwrap();
        assert_eq!(
            c.usage().work(WorkCategory::InputUnitRead) - before,
            expected_reads
        );
        let Some(Token {
            kind: TokenKind::Text(value),
            ..
        }) = event.token()
        else {
            panic!("text")
        };
        assert_eq!(text(value), units(expected_text));
        event.release(&mut c).unwrap();
        t.release(&mut c).unwrap();
        assert_storage_released(&c);
    }
}

#[test]
fn tokenizer_range_counts_identical_interior_endpoints_and_zero_utf16_reads() {
    for (source, expected_reads) in [
        (HtmlSource::utf8("ab"), 3),
        (HtmlSource::utf16(DocumentString::from("ab")), 0),
    ] {
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let before = c.usage().clone();
        let range = t.range(1, 1, &mut c).unwrap();
        assert_eq!((range.start(), range.end()), (1, 1));
        assert_eq!(
            c.usage().work(WorkCategory::InputUnitRead) - before.work(WorkCategory::InputUnitRead),
            expected_reads
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
            2
        );
        assert_eq!(
            c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
            size_of::<HtmlSourceRange>() as u64
        );
        t.release(&mut c).unwrap();
        assert_storage_released(&c);
    }
}

#[test]
fn rejected_range_endpoint_admission_performs_neither_provider_endpoint_read() {
    let source = HtmlSource::utf8("ab");
    let mut baseline = ParseControl::new(limits(), HtmlParseCancellation::new());
    baseline.admit_source(&source).unwrap();
    let t = LexerFixture::new(&source, State::Data, None, &mut baseline).unwrap();
    let entry = baseline
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    t.release(&mut baseline).unwrap();
    // One metadata visit, the separately checked start byte, another metadata
    // visit, and the fixed range copy precede the coupled two endpoint reads.
    let before_endpoints = 3 + size_of::<HtmlSourceRange>() as u64;
    for remaining in [0, 1, 2] {
        let mut caps = limits();
        caps = caps
            .into_builder()
            .work_credits(entry + before_endpoints + remaining)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let result = t.range(1, 1, &mut c);
        if remaining < 2 {
            let stop = result.unwrap_err();
            assert!(matches!(
                stop,
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    requested_delta: 2,
                    ..
                }
            ));
            assert_eq!(c.usage().work(WorkCategory::InputUnitRead), 1);
            assert_eq!(t.range(0, 0, &mut c), Err(stop));
        } else {
            assert_eq!(result.unwrap().start(), 1);
            assert_eq!(c.usage().work(WorkCategory::InputUnitRead), 3);
        }
        t.release(&mut c).unwrap();
        assert_storage_released(&c);
    }
}

fn prepare_quoted_attribute(t: &mut LexerFixture<'_>, c: &mut ParseControl) {
    while t.state != State::AfterAttributeValue {
        t.step(c, false).unwrap();
    }
}

#[test]
fn attribute_span_accounts_for_actual_interior_reads_and_keeps_failed_draft_owned() {
    let source = HtmlSource::utf8("<a x='v' >");
    let mut baseline = ParseControl::new(limits(), HtmlParseCancellation::new());
    baseline.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut baseline).unwrap();
    prepare_quoted_attribute(&mut t, &mut baseline);
    let entry = baseline
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    let reads = baseline.usage().work(WorkCategory::InputUnitRead);
    t.finish_attribute(&mut baseline).unwrap();
    assert_eq!(
        baseline.usage().work(WorkCategory::InputUnitRead) - reads,
        3
    );
    let a = &t.tag.as_ref().unwrap().attributes.as_slice()[0];
    assert_eq!((a.source.start(), a.source.end()), (3, 8));
    assert_eq!(text(&a.name), units("x"));
    assert_eq!(text(&a.value), units("v"));
    t.release(&mut baseline).unwrap();
    assert_storage_released(&baseline);

    let mut caps = limits();
    caps = caps
        .into_builder()
        .work_credits(entry + size_of::<HtmlSourceRange>() as u64 + 4)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    prepare_quoted_attribute(&mut t, &mut c);
    let before = c.usage().work(WorkCategory::InputUnitRead);
    let stop = t.finish_attribute(&mut c).unwrap_err();
    assert!(matches!(
        stop,
        ParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            requested_delta: 2,
            ..
        }
    ));
    assert_eq!(c.usage().work(WorkCategory::InputUnitRead) - before, 1);
    let a = t.tag.as_ref().unwrap().attribute.as_ref().unwrap();
    assert_eq!(a.name.value.units(), units("x"));
    assert_eq!(a.value.value.units(), units("v"));
    assert!(matches!(t.next(&mut c, false),Err(next) if next == stop));
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
}

#[test]
fn empty_source_constructor_admits_its_exact_fixed_record_layout() {
    let source = HtmlSource::utf8("");
    let bytes = size_of::<Tokenizer<'_>>() as u64;
    assert!(bytes > WORK_QUANTUM as u64);
    for cap in [0, bytes - 1, bytes] {
        let mut caps = limits();
        caps = caps
            .into_builder()
            .atomic_layout_bytes(cap)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let result = Tokenizer::new(&source, State::Data, None, &mut c);
        if cap < bytes {
            assert!(
                matches!(result, Err(ParseStop::LimitExceeded { resource: HtmlParseResource::AtomicLayoutBytes, requested_delta, .. }) if requested_delta == bytes)
            );
            assert_eq!(c.usage().largest_atomic_layout, 0);
        } else {
            let t = result.unwrap();
            assert_eq!(c.usage().largest_atomic_layout, bytes);
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AtomicLayoutBytes)
                    .current,
                bytes
            );
            t.release(&mut c).unwrap();
        }
        assert_storage_released(&c);
    }
}

#[test]
fn real_multi_event_transfer_admits_its_fixed_bulk_layout_and_each_slot_write() {
    // Two actual reference operations each emit missing-semicolon and
    // noncharacter diagnostics plus exact text. Invoke the owning state
    // operation directly to prepare a large queue without invented records;
    // the ordinary pull loop drains each operation's smaller batch.
    let source = HtmlSource::utf8("&#xFFFF&#xFFFF");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    t.step(&mut c, false).unwrap();
    t.step(&mut c, false).unwrap();
    assert_eq!(t.pending_len, 6);
    let bytes = 6 * size_of::<Option<Event>>() + size_of::<usize>();
    assert!(bytes > WORK_QUANTUM);
    let before = c.usage().clone();
    let entry = before.resource(HtmlParseResource::WorkCredits).current;
    let event = t.next(&mut c, false).unwrap().unwrap();
    assert!(
        matches!(&event,Event::Diagnostic(d) if t.diagnostic(d).code==DiagnosticCode::MissingSemicolonAfterCharacterReference)
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AtomicLayoutBytes)
            .current,
        bytes as u64
    );
    assert_eq!(
        c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
        bytes as u64
    );
    // Six source slots taken, five destinations replaced, one length update.
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
        12
    );
    event.release(&mut c).unwrap();
    t.release(&mut c).unwrap();
    assert_storage_released(&c);

    let mut caps = limits();
    caps = caps
        .into_builder()
        .work_credits(entry + bytes as u64)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    t.step(&mut c, false).unwrap();
    t.step(&mut c, false).unwrap();
    let before = c.usage().clone();
    let stop = match t.next(&mut c, false) {
        Err(stop) => stop,
        _ => panic!("bulk transfer must reject before taking any event"),
    };
    assert!(matches!(
        stop,
        ParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            requested_delta: 12,
            ..
        }
    ));
    assert_eq!(t.pending_len, 6);
    assert!(t.pending.iter().take(6).all(Option::is_some));
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite),
        before.work(WorkCategory::RecordWrite)
    );
    assert!(matches!(t.next(&mut c,false),Err(next) if next == stop));
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
}

#[test]
fn actual_data_to_tag_open_transition_admits_separate_persistent_writes() {
    let source = HtmlSource::utf8("<");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let before = c.usage().clone();
    t.step(&mut c, false).unwrap();
    assert_eq!(t.state, State::TagOpen);
    // Original six writes: cursor commit; input-kind/range; start/less-than;
    // state. Six real lexical writes add cause append, occurrence payload and
    // append, issued and installed qualified handles, and delimiter part append.
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
        12
    );
    assert_eq!(
        c.usage().work(WorkCategory::StateStep) - before.work(WorkCategory::StateStep),
        0
    );
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
}

#[test]
fn rejected_state_write_does_not_change_state_and_exact_admission_does() {
    let source = HtmlSource::utf8("");
    let mut baseline = ParseControl::new(limits(), HtmlParseCancellation::new());
    baseline.admit_source(&source).unwrap();
    let t = LexerFixture::new(&source, State::Data, None, &mut baseline).unwrap();
    let entry = baseline
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    t.release(&mut baseline).unwrap();
    for remaining in [0, 1] {
        let mut caps = limits();
        caps = caps
            .into_builder()
            .work_credits(entry + size_of::<State>() as u64 + remaining)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let before = c.usage().clone();
        let result = t.set_state(State::TagOpen, &mut c);
        if remaining == 0 {
            let stop = result.unwrap_err();
            assert_eq!(t.state, State::Data);
            assert_eq!(t.set_state(State::Plaintext, &mut c), Err(stop));
        } else {
            result.unwrap();
            assert_eq!(t.state, State::TagOpen);
        }
        assert_eq!(
            c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
            remaining
        );
        t.release(&mut c).unwrap();
        assert_storage_released(&c);
    }
}

#[test]
fn self_closing_and_force_quirks_flags_each_admit_one_independent_record_write() {
    let source = HtmlSource::utf8("<a/>");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    while t.state != State::SelfClosing {
        t.step(&mut c, false).unwrap();
    }
    assert_eq!(t.read(&mut c).unwrap(), Some(u32::from('>')));
    let before = c.usage().clone();
    t.mark_self_closing(&mut c).unwrap();
    assert!(t.tag.as_ref().unwrap().self_closing);
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
        1
    );
    assert_eq!(
        c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
        size_of::<bool>() as u64
    );
    t.release(&mut c).unwrap();
    assert_storage_released(&c);

    let source = HtmlSource::utf8("<!DOCTYPE >");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    while t.state != State::BeforeDoctypeName {
        t.step(&mut c, false).unwrap();
    }
    assert_eq!(t.read(&mut c).unwrap(), Some(u32::from('>')));
    t.create_doctype(&mut c).unwrap();
    let before = c.usage().clone();
    t.quirks(&mut c).unwrap();
    assert!(t.doctype.as_ref().unwrap().force_quirks);
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
        1
    );
    assert_eq!(
        c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
        size_of::<bool>() as u64
    );
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
}

#[test]
fn duplicate_bookkeeping_admits_each_flag_in_addition_to_lookup_and_diagnostic_writes() {
    let source = HtmlSource::utf8("<a x X>");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    loop {
        t.step(&mut c, false).unwrap();
        if t.tag
            .as_ref()
            .and_then(|t| t.attribute.as_ref())
            .is_some_and(|a| a.start == 5 && a.name.value.units() == [b'x' as u16])
        {
            break;
        }
    }
    let before = c.usage().clone();
    t.check_attribute_name(&mut c).unwrap();
    let tag = t.tag.as_ref().unwrap();
    let a = tag.attribute.as_ref().unwrap();
    assert!(a.duplicate && a.name_checked && tag.had_duplicates);
    // Lookup terminal; duplicate/name-checked/had-duplicates flags; one
    // diagnostic and location slots; queue slot and queue length = eight writes.
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
        8
    );
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
}

fn prepare_flag_write(t: &mut LexerFixture<'_>, c: &mut ParseControl, doctype: bool) {
    let state = if doctype {
        State::BeforeDoctypeName
    } else {
        State::SelfClosing
    };
    while t.state != state {
        t.step(c, false).unwrap();
    }
    assert_eq!(t.read(c).unwrap(), Some(u32::from('>')));
    if doctype {
        t.create_doctype(c).unwrap();
    }
}

#[test]
fn rejected_self_closing_and_quirks_write_leave_each_flag_false_and_release_its_owner() {
    for (input, doctype) in [("<a/>", false), ("<!DOCTYPE >", true)] {
        let source = HtmlSource::utf8(input);
        let mut baseline = ParseControl::new(limits(), HtmlParseCancellation::new());
        baseline.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, State::Data, None, &mut baseline).unwrap();
        prepare_flag_write(&mut t, &mut baseline, doctype);
        let entry = baseline
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        t.release(&mut baseline).unwrap();
        assert_storage_released(&baseline);

        let mut caps = limits();
        caps = caps
            .into_builder()
            .work_credits(entry + size_of::<bool>() as u64)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        prepare_flag_write(&mut t, &mut c, doctype);
        let before = c.usage().clone();
        let stop = if doctype {
            t.quirks(&mut c)
        } else {
            t.mark_self_closing(&mut c)
        }
        .unwrap_err();
        assert!(matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                requested_delta: 1,
                ..
            }
        ));
        if doctype {
            assert!(!t.doctype.as_ref().unwrap().force_quirks);
        } else {
            assert!(!t.tag.as_ref().unwrap().self_closing);
        }
        assert_eq!(
            c.usage().work(WorkCategory::RecordWrite),
            before.work(WorkCategory::RecordWrite)
        );
        assert!(matches!(t.next(&mut c,false),Err(next) if next==stop));
        t.release(&mut c).unwrap();
        assert_storage_released(&c);
    }
}

#[test]
fn lexical_actual_duplicates_end_attributes_and_abandoned_eof_fields_are_retained() {
    use crate::construction::parser_session::provenance::{
        LexicalDisposition as D, LexicalKind as K, LexicalRole as R,
    };
    let source =
        HtmlSource::utf8("<X A='&NotEqualTilde;' a=discard></X end=ignored><q unfinished='A");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    while let Some(event) = t.next(&mut c, false).unwrap() {
        event.release(&mut c).unwrap();
    }
    let (occurrences, parts, _) = t.evidence.lexical_observations();
    let spelling =
        |range: &HtmlSourceRange| source.as_utf8().unwrap()[range.start()..range.end()].to_owned();
    let duplicate = occurrences
        .iter()
        .position(|record| record.disposition == D::DuplicateAttribute)
        .unwrap();
    assert_eq!(spelling(&occurrences[duplicate].source), "a=discard");
    let start = occurrences[duplicate].parent.unwrap();
    assert_eq!(occurrences[start].kind, K::StartTag);
    assert_eq!(
        spelling(&occurrences[start].source),
        "<X A='&NotEqualTilde;' a=discard>"
    );
    assert!(parts.iter().any(|part| part.occurrence == start
        && part.role == R::Name
        && part.kind == Some(OriginKind::AsciiLowercase)
        && spelling(&part.source) == "X"));
    assert!(occurrences.iter().any(|record| record.kind == K::Attribute
        && record.disposition == D::EndTagAttribute
        && spelling(&record.source) == "end=ignored"));
    assert!(occurrences.iter().any(|record| record.kind == K::Attribute
        && record.disposition == D::Abandoned
        && spelling(&record.source) == "unfinished='A"));
    assert!(occurrences.iter().any(|record| record.kind == K::StartTag
        && record.disposition == D::Abandoned
        && spelling(&record.source) == "<q unfinished='A"));
    let expansion: Vec<_> = parts
        .iter()
        .filter(|part| part.role == R::Value && spelling(&part.source) == "&NotEqualTilde;")
        .map(|part| (part.start, part.end))
        .collect();
    assert_eq!(expansion, [(0, 1), (1, 2)]);
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
}

#[test]
fn lexical_doctype_empty_identifiers_pi_recovery_and_unfinished_target_have_distinct_parts() {
    use crate::construction::parser_session::provenance::{
        LexicalDisposition as D, LexicalKind as K, LexicalRole as R,
    };
    let source = HtmlSource::utf8(r#"<!DOCTYPE html PUBLIC '' ""><?xml data><?go"#);
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    while let Some(event) = t.next(&mut c, false).unwrap() {
        event.release(&mut c).unwrap();
    }
    let (occurrences, parts, _) = t.evidence.lexical_observations();
    let spelling =
        |range: &HtmlSourceRange| source.as_utf8().unwrap()[range.start()..range.end()].to_owned();
    let doctype = occurrences
        .iter()
        .position(|record| record.kind == K::Doctype)
        .unwrap();
    for role in [R::PublicIdentifier, R::SystemIdentifier] {
        assert!(parts.iter().any(|part| part.occurrence == doctype
            && part.role == role
            && part.source.start() == part.source.end()));
    }
    let recovery = occurrences
        .iter()
        .position(|record| record.kind == K::Comment)
        .unwrap();
    assert_eq!(spelling(&occurrences[recovery].source), "<?xml data>");
    let original: String = parts
        .iter()
        .filter(|part| part.occurrence == recovery && part.role == R::Target)
        .map(|part| spelling(&part.source))
        .collect();
    assert_eq!(original, "xml");
    let unfinished = occurrences
        .iter()
        .position(|record| record.disposition == D::Abandoned)
        .unwrap();
    assert_eq!(spelling(&occurrences[unfinished].source), "<?go");
    let target: String = parts
        .iter()
        .filter(|part| part.occurrence == unfinished && part.role == R::Target)
        .map(|part| spelling(&part.source))
        .collect();
    assert_eq!(target, "go");
    assert!(parts.iter().any(|part| part.occurrence == unfinished
        && part.role == R::EndAnchor
        && part.source.start() == source.len()
        && part.source.end() == source.len()));
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
}

#[test]
fn lexical_abandoned_drafts_exact_and_one_short_keep_first_stop_without_rescue_dispositions() {
    for text in ["<a x='A", "<?go A", "<?xml A>", "<!DOCTYPE h PUBLIC '' ''>"] {
        let source = HtmlSource::utf8(text);
        let (stop, work) = run_with_limits(&source, State::Data, None, limits());
        assert_eq!(stop, None);
        // Exact completion needs the final charge; one short remains sticky and
        // the same real fixture runner verifies every nested owner is reclaimed.
        let short = limits()
            .into_builder()
            .work_credits(work - 1)
            .finish()
            .unwrap();
        assert!(matches!(
            run_with_limits(&source, State::Data, None, short).0,
            Some(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
        let exact = limits().into_builder().work_credits(work).finish().unwrap();
        assert_eq!(run_with_limits(&source, State::Data, None, exact).0, None);
    }
}

#[test]
fn lexical_utf16_attribute_parts_preserve_surrogate_crlf_null_bom_pair_and_entity_atoms() {
    use crate::construction::parser_session::provenance::{LexicalKind, LexicalRole};
    let source = HtmlSource::utf16(DocumentString::from_utf16(vec![
        60, 97, 32, 120, 61, 39, 0xd800, 13, 10, 0, 0xfeff, 0xd83d, 0xde42, 38, 97, 109, 112, 59,
        39, 62,
    ]));
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let mut value = false;
    while let Some(event) = lexer.next(&mut c, false).unwrap() {
        if let Event::Token(token) = &event
            && let TokenKind::Tag(tag) = &token.token().kind
        {
            assert_eq!(tag.attributes.as_slice().len(), 1);
            assert_eq!(
                tag.attributes.as_slice()[0].value.value.text().code_units(),
                [0xd800, 10, 0xfffd, 0xfeff, 0xd83d, 0xde42, 38]
            );
            value = true;
        }
        event.release(&mut c).unwrap();
    }
    assert!(value);
    let (occurrences, parts, _) = lexer.evidence.lexical_observations();
    let attribute = occurrences
        .iter()
        .position(|occurrence| occurrence.kind == LexicalKind::Attribute)
        .unwrap();
    assert_eq!(
        (
            occurrences[attribute].source.start(),
            occurrences[attribute].source.end()
        ),
        (3, 19)
    );
    let atoms: Vec<_> = parts
        .iter()
        .filter(|part| {
            part.occurrence == attribute && part.role == LexicalRole::Value && part.kind.is_some()
        })
        .map(|part| {
            (
                part.source.start(),
                part.source.end(),
                part.start,
                part.end,
                part.kind.unwrap(),
            )
        })
        .collect();
    assert!(parts.iter().any(|part| part.occurrence == attribute
        && part.role == LexicalRole::EndAnchor
        && part.source.start() == 18
        && part.source.end() == 18
        && part.start == 7
        && part.end == 7));
    assert_eq!(
        atoms,
        [
            (6, 7, 0, 1, OriginKind::Literal),
            (7, 9, 1, 2, OriginKind::Newline),
            (9, 10, 2, 3, OriginKind::NullReplacement),
            (10, 11, 3, 4, OriginKind::Literal),
            (11, 13, 4, 6, OriginKind::Literal),
            (13, 18, 6, 7, OriginKind::CharacterReference),
        ]
    );
    lexer.release(&mut c).unwrap();
    assert_storage_released(&c);
}

#[test]
fn lexical_comment_doctype_pi_cdata_and_raw_recovery_keep_original_field_atoms() {
    use crate::construction::parser_session::provenance::{
        LexicalDisposition as D, LexicalKind as K, LexicalRole as R,
    };
    for (input, state, last, cdata, family, disposition, role, field) in [
        (
            "<!--A",
            State::Data,
            None,
            false,
            K::Comment,
            D::Emitted,
            R::Data,
            "A",
        ),
        (
            "<!DOCTYPE h PUBLIC 'A",
            State::Data,
            None,
            false,
            K::Doctype,
            D::Emitted,
            R::PublicIdentifier,
            "A",
        ),
        (
            "<?go A",
            State::Data,
            None,
            false,
            K::ProcessingInstruction,
            D::Abandoned,
            R::Target,
            "go",
        ),
        (
            "<![CDATA[A]]>",
            State::Data,
            None,
            true,
            K::Cdata,
            D::Applied,
            R::Data,
            "A",
        ),
        (
            "<![CDATA[A",
            State::Data,
            None,
            true,
            K::Cdata,
            D::Abandoned,
            R::Data,
            "A",
        ),
        (
            "</MiSs>",
            State::Raw(Raw::Rcdata),
            Some("title"),
            false,
            K::Markup,
            D::LiteralRecovery,
            R::Name,
            "MiSs",
        ),
    ] {
        let source = HtmlSource::utf8(input);
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let last = last.map(DocumentString::from);
        let mut lexer = LexerFixture::new(&source, state, last.as_ref(), &mut c).unwrap();
        while let Some(event) = lexer.next(&mut c, cdata).unwrap() {
            event.release(&mut c).unwrap();
        }
        let (occurrences, parts, _) = lexer.evidence.lexical_observations();
        let occurrence = occurrences
            .iter()
            .position(|occurrence| {
                occurrence.kind == family && occurrence.disposition == disposition
            })
            .unwrap_or_else(|| panic!("literal family and disposition: {input}"));
        assert_eq!(occurrences[occurrence].source.start(), 0);
        let original: String = parts
            .iter()
            .filter(|part| {
                part.occurrence == occurrence && part.role == role && part.kind.is_some()
            })
            .map(|part| &input[part.source.start()..part.source.end()])
            .collect();
        assert_eq!(original, field, "original field atoms: {input}");
        lexer.release(&mut c).unwrap();
        assert_storage_released(&c);
    }
}

#[test]
fn lexical_cdata_chunk_coordinates_accumulate_exact_actual_output_units() {
    use crate::construction::parser_session::provenance::{
        LexicalDisposition, LexicalKind, LexicalRole,
    };
    let source = HtmlSource::utf8("<![CDATA[A🙂\r\nB]]>");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let mut actual = Vec::new();
    while let Some(event) = lexer.next(&mut c, true).unwrap() {
        if let Event::Token(token) = &event
            && let TokenKind::Text(text) = &token.token().kind
        {
            actual.extend_from_slice(text.value.text().code_units());
        }
        event.release(&mut c).unwrap();
    }
    assert_eq!(actual, [65, 0xd83d, 0xde42, 10, 66]);
    let (occurrences, parts, _) = lexer.evidence.lexical_observations();
    let occurrence = occurrences
        .iter()
        .position(|occurrence| {
            occurrence.kind == LexicalKind::Cdata
                && occurrence.disposition == LexicalDisposition::Applied
        })
        .unwrap();
    assert_eq!(
        (
            occurrences[occurrence].source.start(),
            occurrences[occurrence].source.end()
        ),
        (0, 20)
    );
    let atoms: Vec<_> = parts
        .iter()
        .filter(|part| {
            part.occurrence == occurrence && part.role == LexicalRole::Data && part.kind.is_some()
        })
        .map(|part| {
            (
                part.source.start(),
                part.source.end(),
                part.start,
                part.end,
                part.kind.unwrap(),
            )
        })
        .collect();
    assert_eq!(
        atoms,
        [
            (9, 10, 0, 1, OriginKind::Literal),
            (10, 14, 1, 3, OriginKind::Literal),
            (14, 16, 3, 4, OriginKind::Newline),
            (16, 17, 4, 5, OriginKind::Literal),
        ]
    );
    lexer.release(&mut c).unwrap();
    assert_storage_released(&c);
}

// Actual lexer outputs and original metadata are observed before releasing the
// real owners. The assertions below run after release, including failing REDs.
struct LexicalDelimiterObservation {
    value: Vec<u16>,
    targets: Vec<Vec<u16>>,
    syntax: Vec<(usize, usize)>,
    data: Vec<(usize, usize, usize, usize, OriginKind)>,
    envelope: (usize, usize),
    disposition: crate::construction::parser_session::provenance::LexicalDisposition,
}

fn lexical_delimiter_observation(
    input: &str,
    cdata: bool,
    family: crate::construction::parser_session::provenance::LexicalKind,
) -> LexicalDelimiterObservation {
    use crate::construction::parser_session::provenance::LexicalRole;
    let source = HtmlSource::utf8(input);
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let mut value = Vec::new();
    let mut targets = Vec::new();
    while let Some(event) = lexer.next(&mut c, cdata).unwrap() {
        if let Event::Token(token) = &event {
            match &token.token().kind {
                TokenKind::Text(text) | TokenKind::Comment(text) => {
                    value.extend_from_slice(text.value.text().code_units());
                }
                TokenKind::ProcessingInstruction { target, data } => {
                    targets.push(target.value.text().code_units().to_vec());
                    value.extend_from_slice(data.value.text().code_units());
                }
                _ => {}
            }
        }
        event.release(&mut c).unwrap();
    }
    let (occurrences, parts, _) = lexer.evidence.lexical_observations();
    let observed = occurrences
        .iter()
        .enumerate()
        .find(|(_, row)| row.kind == family)
        .map(|(occurrence, row)| LexicalDelimiterObservation {
            value,
            targets,
            syntax: parts
                .iter()
                .filter(|part| part.occurrence == occurrence && part.role == LexicalRole::Syntax)
                .map(|part| (part.source.start(), part.source.end()))
                .collect(),
            data: parts
                .iter()
                .filter(|part| part.occurrence == occurrence && part.role == LexicalRole::Data)
                .filter_map(|part| {
                    part.kind.map(|kind| {
                        (
                            part.source.start(),
                            part.source.end(),
                            part.start,
                            part.end,
                            kind,
                        )
                    })
                })
                .collect(),
            envelope: (row.source.start(), row.source.end()),
            disposition: row.disposition,
        });
    lexer.release(&mut c).unwrap();
    assert_storage_released(&c);
    observed.expect("the literal must retain its actual occurrence family")
}

fn lexical_delimiter_syntax_bytes(
    observed: &LexicalDelimiterObservation,
    data_start: usize,
) -> Vec<usize> {
    // Segmented parts and a single complete descriptor are both valid; the
    // contract is exact source coverage, independently of storage segmentation.
    let mut bytes: Vec<_> = observed
        .syntax
        .iter()
        .flat_map(|(start, end)| *start..*end)
        .filter(|offset| *offset >= data_start)
        .collect();
    bytes.sort_unstable();
    bytes.dedup();
    bytes
}

#[test]
fn lexical_closing_delimiter_cdata_keeps_both_brackets_and_greater_than_separate_from_data() {
    use crate::construction::parser_session::provenance::{
        LexicalDisposition as D, LexicalKind as K,
    };
    let observed = lexical_delimiter_observation("<![CDATA[A]]>", true, K::Cdata);
    assert_eq!(observed.value, units("A"));
    assert!(observed.targets.is_empty());
    assert_eq!(observed.envelope, (0, 13));
    assert_eq!(observed.disposition, D::Applied);
    assert_eq!(observed.data, [(9, 10, 0, 1, OriginKind::Literal)]);
    assert_eq!(
        lexical_delimiter_syntax_bytes(&observed, 9),
        [10, 11, 12],
        "closing ]]>, distinct from data A9..10 and the whole envelope0..13"
    );
}

#[test]
fn lexical_closing_delimiter_comment_keeps_both_dashes_and_greater_than_separate_from_data() {
    use crate::construction::parser_session::provenance::{
        LexicalDisposition as D, LexicalKind as K,
    };
    let observed = lexical_delimiter_observation("<!--A-->", false, K::Comment);
    assert_eq!(observed.value, units("A"));
    assert!(observed.targets.is_empty());
    assert_eq!(observed.envelope, (0, 8));
    assert_eq!(observed.disposition, D::Emitted);
    assert_eq!(observed.data, [(4, 5, 0, 1, OriginKind::Literal)]);
    assert_eq!(
        lexical_delimiter_syntax_bytes(&observed, 4),
        [5, 6, 7],
        "closing --> must retain its first dash5..6"
    );
}

#[test]
fn lexical_closing_delimiter_pi_keeps_question_mark_and_greater_than_separate_from_data() {
    use crate::construction::parser_session::provenance::{
        LexicalDisposition as D, LexicalKind as K,
    };
    let observed = lexical_delimiter_observation("<?go A?>", false, K::ProcessingInstruction);
    assert_eq!(observed.value, units("A"));
    assert_eq!(observed.targets, [units("go")]);
    assert_eq!(observed.envelope, (0, 8));
    assert_eq!(observed.disposition, D::Emitted);
    assert_eq!(observed.data, [(5, 6, 0, 1, OriginKind::Literal)]);
    assert_eq!(
        lexical_delimiter_syntax_bytes(&observed, 5),
        [6, 7],
        "the selected PI close ?> includes the remembered question mark"
    );
}

#[test]
fn lexical_closing_delimiter_pi_bare_greater_than_is_retained_as_actual_selected_close() {
    use crate::construction::parser_session::provenance::{
        LexicalDisposition as D, LexicalKind as K,
    };
    let observed = lexical_delimiter_observation("<?go A>", false, K::ProcessingInstruction);
    assert_eq!(observed.value, units("A"));
    assert_eq!(observed.targets, [units("go")]);
    assert_eq!(observed.envelope, (0, 7));
    assert_eq!(observed.disposition, D::Emitted);
    assert_eq!(observed.data, [(5, 6, 0, 1, OriginKind::Literal)]);
    assert_eq!(
        lexical_delimiter_syntax_bytes(&observed, 5),
        [6],
        "the pinned PI data state emits on the actual bare >"
    );
}

#[test]
fn lexical_closing_delimiter_fallback_data_and_extra_brackets_are_not_relabelled_as_close() {
    use crate::construction::parser_session::provenance::LexicalKind as K;
    // Expected native values and literal final delimiter positions follow the
    // selected state algorithms, not the producer's current metadata output.
    for (input, cdata, family, native, data_start, closing) in [
        ("<![CDATA[A]B]]>", true, K::Cdata, "A]B", 9, 12..15),
        ("<![CDATA[A]]B]]>", true, K::Cdata, "A]]B", 9, 13..16),
        ("<![CDATA[A]]]>", true, K::Cdata, "A]", 9, 11..14),
        ("<![CDATA[A]]]]>", true, K::Cdata, "A]]", 9, 12..15),
        ("<!--A-B-->", false, K::Comment, "A-B", 4, 7..10),
        ("<!--A--B-->", false, K::Comment, "A--B", 4, 8..11),
        ("<!--A---->", false, K::Comment, "A--", 4, 7..10),
        ("<!--A--!B-->", false, K::Comment, "A--!B", 4, 9..12),
        ("<!--A--!>", false, K::Comment, "A", 4, 5..9),
        (
            "<?go A?B?>",
            false,
            K::ProcessingInstruction,
            "A?B",
            5,
            8..10,
        ),
        (
            "<?go A??B?>",
            false,
            K::ProcessingInstruction,
            "A??B",
            5,
            9..11,
        ),
    ] {
        let observed = lexical_delimiter_observation(input, cdata, family);
        assert_eq!(
            observed.value,
            units(native),
            "actual native payload: {input}"
        );
        assert_eq!(observed.envelope, (0, input.len()));
        assert_eq!(
            lexical_delimiter_syntax_bytes(&observed, data_start),
            closing.collect::<Vec<_>>(),
            "only the actual final delimiter is closing syntax: {input}"
        );
    }
}

#[test]
fn lexical_closing_delimiter_unterminated_eof_keeps_emitted_brackets_as_data_and_no_fake_close() {
    use crate::construction::parser_session::provenance::{
        LexicalDisposition as D, LexicalKind as K,
    };
    for (input, native) in [
        ("<![CDATA[A", "A"),
        ("<![CDATA[A]", "A]"),
        ("<![CDATA[A]]", "A]]"),
        ("<![CDATA[A]]]", "A]]]"),
    ] {
        let observed = lexical_delimiter_observation(input, true, K::Cdata);
        assert_eq!(
            observed.value,
            units(native),
            "EOF emits its pending bracket data"
        );
        assert_eq!(observed.envelope, (0, input.len()));
        assert_eq!(observed.disposition, D::Abandoned);
        assert!(
            lexical_delimiter_syntax_bytes(&observed, 9).is_empty(),
            "EOF bracket output is data, not a fabricated closing delimiter: {input}"
        );
    }
    // An unterminated comment emits its value, while an unterminated PI is
    // abandoned. Neither has a > nor a synthetic closing Syntax/EOF point.
    for (input, family, native, disposition) in [
        ("<!--A--", K::Comment, "A", D::Emitted),
        ("<?go A?", K::ProcessingInstruction, "", D::Abandoned),
    ] {
        let observed = lexical_delimiter_observation(input, false, family);
        assert_eq!(observed.value, units(native));
        assert_eq!(observed.envelope, (0, input.len()));
        assert_eq!(observed.disposition, disposition);
        assert!(
            observed
                .syntax
                .iter()
                .all(|(start, end)| *start < *end && *end <= input.len())
        );
    }
}

fn prepare_actual_delimiter_close(t: &mut LexerFixture<'_>, c: &mut ParseControl, state: State) {
    // Real state transitions establish the remembered punctuation. No state or
    // source span is supplied by this test; the next real input must be >.
    let mut steps = 0;
    while t.state != state {
        t.step(c, true).unwrap();
        steps += 1;
        assert!(steps < 32);
    }
    if state == State::PiData {
        // The target transition reconsumes A into PiData; process that actual
        // data character before taking the next closing input.
        t.step(c, true).unwrap();
    }
    assert_eq!(t.read(c).unwrap(), Some(0x3e));
}
#[test]
fn lexical_close_admission_exact_each_short_cancel_and_record_cap_keep_actual_pending_owners() {
    use crate::construction::parser_session::provenance::LexicalRole;
    for (input, state, start, end) in [
        ("<![CDATA[A]]>", State::CdataEnd, 10, 13),
        ("<!--A-->", State::CommentEnd, 5, 8),
        ("<?go A?>", State::PiQuestionable, 6, 8),
        ("<?go A>", State::PiData, 6, 7),
    ] {
        let source = HtmlSource::utf8(input);
        let mut baseline = ParseControl::new(limits(), HtmlParseCancellation::new());
        baseline.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, State::Data, None, &mut baseline).unwrap();
        prepare_actual_delimiter_close(&mut t, &mut baseline, state);
        // PiData is reached before A, so consume its actual data step and read
        // the close before observing the specific closing producer.
        if t.current.start() != end - 1 {
            unreachable!("the prepared close must be the final greater-than");
        }
        let entry = baseline
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let entries = baseline
            .usage()
            .resource(HtmlParseResource::SourceMapEntries)
            .current;
        let before = t.evidence.lexical_observations().1.len();
        let pending = t.pending_len;
        let remembered = if state == State::PiData {
            t.current.start()
        } else {
            t.pending_syntax.start()
        };
        assert_eq!(remembered, start);
        t.lexer
            .retain_lexical_close(remembered, &mut baseline, &mut t.evidence)
            .unwrap();
        let exact = baseline
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let part = t.evidence.lexical_observations().1.last().unwrap();
        let actual = (
            part.role,
            part.source.start(),
            part.source.end(),
            part.start,
            part.end,
        );
        let unchanged = (t.state, t.pending_len);
        let after = t.evidence.lexical_observations().1.len();
        t.release(&mut baseline).unwrap();
        assert_storage_released(&baseline);
        assert_eq!(actual, (LexicalRole::Syntax, start, end, 0, 0));
        assert_eq!(after, before + 1);
        assert_eq!(unchanged, (state, pending));

        // Every real admission boundary in this producer, plus exact success.
        // This observes actual costs and never changes a production fee.
        for cap in entry..=exact {
            let caps = limits().into_builder().work_credits(cap).finish().unwrap();
            let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
            c.admit_source(&source).unwrap();
            let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            prepare_actual_delimiter_close(&mut t, &mut c, state);
            let before = t.evidence.lexical_observations().1.len();
            let pending = t.pending_len;
            let remembered = if state == State::PiData {
                t.current.start()
            } else {
                t.pending_syntax.start()
            };
            let result = t
                .lexer
                .retain_lexical_close(remembered, &mut c, &mut t.evidence);
            if cap == exact {
                assert_eq!(result, Ok(()));
                assert_eq!(t.evidence.lexical_observations().1.len(), before + 1);
            } else {
                let stop = result.unwrap_err();
                assert!(matches!(
                    stop,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }
                ));
                assert_eq!(t.evidence.lexical_observations().1.len(), before);
                assert!(matches!(t.next(&mut c, true), Err(next) if next == stop));
                assert_eq!(c.stop(), Some(stop));
            }
            assert_eq!((t.state, t.pending_len), (state, pending));
            t.release(&mut c).unwrap();
            assert_storage_released(&c);
        }
        // A close requires exactly one new logical lexical part. Refusing it
        // or cancellation retains the existing fields and pending events.
        for cancel in [false, true] {
            let caps = if cancel {
                limits()
            } else {
                limits()
                    .into_builder()
                    .source_map_entries(entries)
                    .finish()
                    .unwrap()
            };
            let cancellation = HtmlParseCancellation::new();
            let mut c = ParseControl::new(caps, cancellation.clone());
            c.admit_source(&source).unwrap();
            let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            prepare_actual_delimiter_close(&mut t, &mut c, state);
            let before = t.evidence.lexical_observations().1.len();
            let pending = t.pending_len;
            if cancel {
                cancellation.cancel();
            }
            let remembered = if state == State::PiData {
                t.current.start()
            } else {
                t.pending_syntax.start()
            };
            let stop = t
                .lexer
                .retain_lexical_close(remembered, &mut c, &mut t.evidence)
                .unwrap_err();
            if cancel {
                assert_eq!(stop, ParseStop::Cancelled);
            } else {
                assert!(matches!(
                    stop,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::SourceMapEntries,
                        ..
                    }
                ));
            }
            assert_eq!(t.evidence.lexical_observations().1.len(), before);
            assert!(matches!(t.next(&mut c, true), Err(next) if next == stop));
            assert_eq!((t.state, t.pending_len), (state, pending));
            t.release(&mut c).unwrap();
            assert_storage_released(&c);
        }
    }
}

#[test]
fn radix_name_storage_growth_submits_bounded_layouts_for_unique_prefixes_and_duplicates() {
    // Independently enumerated prefixes of the 16-bit units in the literal
    // names sharedprefix0000.., including the empty root. Duplicate replay
    // requires no new prefix; this is not an expectation read from Names.
    let cases = [(1, 257), (8, 268), (16, 298), (64, 463), (256, 1_140)];
    let mut observations = Vec::new();
    for (count, expected_branches) in cases {
        let bytes = size_of::<Branch>() as u64;
        // Room for <2B final slots and old/new overlap, independent of observed
        // capacity. Logical slots remain exactly the necessary B prefixes.
        let capacity_budget = 4 * expected_branches as u64 * bytes;
        let caps = limits()
            .into_builder()
            .auxiliary_entries(expected_branches as u64)
            .managed_capacity_bytes(capacity_budget)
            .peak_transient_bytes(capacity_budget)
            .atomic_layout_bytes(capacity_budget)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut names = Names::new().unwrap();
        for i in 0..count {
            assert!(
                !names
                    .insert(&units(&format!("sharedprefix{i:04}")), &mut c)
                    .unwrap()
            );
        }
        assert_eq!(names.nodes.len(), expected_branches);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            expected_branches as u64
        );
        let reserve = c.usage().work(WorkCategory::ReserveByte);
        let provider = c.usage().work(WorkCategory::ProviderCall);
        let retained = c.usage().actual_capacity_bytes;
        assert_eq!(retained, names.nodes.capacity() as u64 * bytes);
        for i in 0..count {
            assert!(
                names
                    .insert(&units(&format!("sharedprefix{i:04}")), &mut c)
                    .unwrap()
            );
        }
        assert_eq!(names.nodes.len(), expected_branches);
        assert_eq!(c.usage().work(WorkCategory::ReserveByte), reserve);
        assert_eq!(c.usage().work(WorkCategory::ProviderCall), provider);
        assert_eq!(c.usage().actual_capacity_bytes, retained);
        assert_eq!(c.stop(), None);
        let before_cleanup = c.usage().clone();
        names.nodes.release(&mut c).unwrap();
        assert_storage_released(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits),
            before_cleanup.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(
            c.usage().actual_capacity_high_water,
            before_cleanup.actual_capacity_high_water
        );
        observations.push((count, expected_branches, reserve / bytes, provider));
    }
    // Unclipped admitted doubling submits old+new layouts below 6B slots.
    // A generous 8B fixture allowance distinguishes repeated exact growth;
    // it is not a normative HTML coefficient or the diagnostic 100M cap.
    for (count, branches, submitted_slots, provider_calls) in observations {
        assert!(
            submitted_slots <= 8 * branches as u64,
            "names={count}, required prefixes={branches}, submitted old/new slots={submitted_slots}, provider calls={provider_calls}"
        );
    }
}

#[test]
fn retained_attribute_growth_is_bounded_without_charging_duplicates_as_new_slots() {
    use crate::construction::parser_session::provenance::{
        LexicalDisposition as D, LexicalKind as K,
    };
    // Every expected name, value and raw interval comes from the authored
    // literal sequence. Uppercase second occurrences normalize to duplicates.
    const COUNT: usize = 64;
    let mut raw = "<p".to_owned();
    let mut expected = Vec::new();
    let mut duplicate_ranges = Vec::new();
    let mut declarations = Vec::new();
    for duplicate in [false, true] {
        for i in 0..COUNT {
            let local = format!("sharedprefix{i:04}");
            let spelling = if duplicate {
                local.to_ascii_uppercase()
            } else {
                local.clone()
            };
            let value = if duplicate { "discard" } else { "first" };
            raw.push(' ');
            let start = raw.len();
            raw.push_str(&format!("{spelling}='{value}'"));
            let end = raw.len();
            declarations.push((start, end, duplicate));
            if duplicate {
                let equal = start + spelling.len();
                duplicate_ranges.push((equal, equal + 1));
            } else {
                expected.push((units(&local), units(value), start, end));
            }
        }
    }
    raw.push('>');
    let source = HtmlSource::utf8(raw.clone());
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let mut finished = 0;
    let mut retained_capacity_slots = 0usize;
    let mut capacity_changes = 0;
    let mut finish_reserve_all_writers = 0;
    let mut diagnostic_count = 0;
    let mut tag_count = 0;
    let mut eof_count = 0;
    loop {
        if t.pending_len != 0 || t.done {
            let Some(event) = t.next(&mut c, false).unwrap() else {
                break;
            };
            match &event {
                Event::Diagnostic(id) => {
                    let d = t.diagnostic(id);
                    assert_eq!(d.code, DiagnosticCode::DuplicateAttribute);
                    assert_eq!(d.phase, token::DiagnosticPhase::Tokenizer);
                    assert_eq!(d.state, State::AttributeName);
                    assert_eq!(d.source.source(), source.identity());
                    assert_eq!(
                        (d.source.start(), d.source.end()),
                        duplicate_ranges[diagnostic_count]
                    );
                    diagnostic_count += 1;
                }
                Event::Token(event) => match &event.token().kind {
                    TokenKind::Tag(tag) => {
                        assert_eq!(tag.kind, TagKind::Start);
                        assert_eq!(text(&tag.name), units("p"));
                        assert!(tag.had_duplicate_attributes);
                        assert_eq!(tag.attributes.len(), COUNT);
                        for (a, (name, value, start, end)) in
                            tag.attributes.as_slice().iter().zip(&expected)
                        {
                            assert_eq!(text(&a.name), *name);
                            assert_eq!(text(&a.value), *value);
                            assert_eq!(a.source.source(), source.identity());
                            assert_eq!((a.source.start(), a.source.end()), (*start, *end));
                        }
                        assert_eq!(
                            (event.token().source.start(), event.token().source.end()),
                            (0, raw.len())
                        );
                        tag_count += 1;
                    }
                    TokenKind::Eof => eof_count += 1,
                    _ => panic!("unexpected actual token"),
                },
                _ => panic!("unexpected actual event"),
            }
            event.release(&mut c).unwrap();
            continue;
        }
        if t.state == State::AfterAttributeValue && t.tag.as_ref().unwrap().attribute.is_some() {
            let before = t.tag.as_ref().unwrap().attributes.capacity();
            let duplicate = t
                .tag
                .as_ref()
                .unwrap()
                .attribute
                .as_ref()
                .unwrap()
                .duplicate;
            let reserve_before = c.usage().work(WorkCategory::ReserveByte);
            // Exercise the real existing owner while the completed current
            // attribute remains intact. Subsequent normal state dispatch sees
            // the already finished slot, just as another owner call does.
            t.finish_attribute(&mut c).unwrap();
            let after = t.tag.as_ref().unwrap().attributes.capacity();
            finish_reserve_all_writers +=
                c.usage().work(WorkCategory::ReserveByte) - reserve_before;
            if duplicate {
                assert_eq!(before, after);
                assert_eq!(t.tag.as_ref().unwrap().attributes.len(), COUNT);
            } else {
                assert_eq!(t.tag.as_ref().unwrap().attributes.len(), finished + 1);
                if before != after {
                    // Actual retained vector replacement capacities, not a
                    // claim about allocator copy bytes or all ReserveByte.
                    retained_capacity_slots += before + after;
                    capacity_changes += 1;
                }
            }
            finished += 1;
        } else {
            t.step(&mut c, false).unwrap();
        }
    }
    assert_eq!(finished, 2 * COUNT);
    assert_eq!((tag_count, eof_count, diagnostic_count), (1, 1, COUNT));
    let (occurrences, _, _) = t.evidence.lexical_observations();
    let attrs: Vec<_> = occurrences
        .iter()
        .filter(|o| o.kind == K::Attribute)
        .collect();
    assert_eq!(attrs.len(), 2 * COUNT);
    for (o, &(start, end, duplicate)) in attrs.iter().zip(&declarations) {
        assert_eq!(
            o.disposition,
            if duplicate {
                D::DuplicateAttribute
            } else {
                D::StoredAttribute
            }
        );
        assert_eq!(o.source.source(), source.identity());
        assert_eq!((o.source.start(), o.source.end()), (start, end));
        assert_eq!(source.as_utf8().unwrap()[start..end], raw[start..end]);
    }
    assert_eq!(c.stop(), None);
    let before_cleanup = c.usage().clone();
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
    assert_eq!(c.usage().actual_capacity_bytes, 0);
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits),
        before_cleanup.resource(HtmlParseResource::WorkCredits)
    );
    // The span also includes lexical and frozen buffer fees. This observation
    // is deliberately not called Attribute-only reservation work.
    assert!(
        finish_reserve_all_writers
            >= retained_capacity_slots as u64 * size_of::<Attribute>() as u64
    );
    assert!(
        retained_capacity_slots <= 8 * COUNT,
        "actual unique attributes={COUNT}, observed old/new capacity slots={retained_capacity_slots}, capacity changes={capacity_changes}, all finish-span ReserveByte={finish_reserve_all_writers}"
    );
}

#[test]
fn name_growth_clips_to_exact_physical_room_and_retains_one_short_failure() {
    // One UTF-16 unit requires the root plus its sixteen bit prefixes.
    let bytes = size_of::<Branch>() as u64;
    for slots in [16, 17] {
        let caps = limits()
            .into_builder()
            .auxiliary_entries(17)
            .managed_capacity_bytes(slots * bytes)
            .peak_transient_bytes(33 * bytes)
            .atomic_layout_bytes(17 * bytes)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut names = Names::new().unwrap();
        let result = names.insert(&[97], &mut c);
        if slots == 17 {
            assert_eq!(result, Ok(false));
            assert_eq!(names.nodes.len(), 17);
            assert_eq!(names.nodes.capacity(), 17);
            assert_eq!(names.insert(&[97], &mut c), Ok(true));
            // Final clipped growth retains 16 old and submits 17 new slots.
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::PeakTransientBytes)
                    .high_water,
                33 * bytes
            );
        } else {
            let stop = result.unwrap_err();
            assert!(matches!(
                stop,
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedCapacityBytes,
                    ..
                }
            ));
            assert_eq!(names.nodes.len(), 16);
            assert_eq!(names.nodes.capacity(), 16);
            assert!(names.nodes.as_slice().iter().all(|node| !node.terminal));
            assert_eq!(names.insert(&[97], &mut c), Err(stop));
        }
        assert_eq!(c.usage().actual_capacity_bytes, slots * bytes);
        let snapshot = c.first_stop_usage().cloned();
        let before = c.usage().clone();
        names.nodes.release(&mut c).unwrap();
        assert_storage_released(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.first_stop_usage(), snapshot.as_ref());
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(
            c.usage().actual_capacity_high_water,
            before.actual_capacity_high_water
        );
    }
}

#[test]
fn name_growth_foreign_and_cancelled_owners_keep_the_actual_index_for_cleanup() {
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let mut names = Names::new().unwrap();
    assert_eq!(names.insert(&[97], &mut c), Ok(false));
    let len = names.nodes.len();
    let capacity = names.nodes.capacity();
    let before = c.usage().clone();
    let mut foreign = ParseControl::new(limits(), HtmlParseCancellation::new());
    assert_eq!(
        names.insert(&[98], &mut foreign),
        Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
    );
    assert_eq!(c.usage(), &before);
    assert_eq!((names.nodes.len(), names.nodes.capacity()), (len, capacity));
    assert_eq!(names.insert(&[97], &mut c), Ok(true));
    cancellation.cancel();
    assert_eq!(names.insert(&[98], &mut c), Err(ParseStop::Cancelled));
    assert_eq!((names.nodes.len(), names.nodes.capacity()), (len, capacity));
    let snapshot = c.first_stop_usage().unwrap().clone();
    names.nodes.release(&mut c).unwrap();
    assert_storage_released(&c);
    assert_eq!(c.usage().actual_capacity_bytes, 0);
    assert_eq!(c.first_stop_usage(), Some(&snapshot));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits),
        snapshot.resource(HtmlParseResource::WorkCredits)
    );
}

#[test]
fn retained_attribute_growth_denial_keeps_draft_buffers_and_canonical_stop() {
    let source = HtmlSource::utf8("<a x='v' >");
    let mut baseline = ParseControl::new(limits(), HtmlParseCancellation::new());
    baseline.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut baseline).unwrap();
    prepare_quoted_attribute(&mut t, &mut baseline);
    t.finish_attribute(&mut baseline).unwrap();
    // Freeze transfers the two existing variable buffers unchanged and admits
    // one known Arc<Vec<u16>> payload per name/value. Remove only those fixed
    // payloads to derive the actual capacity at the new pre-freeze reservation.
    let reserve_capacity =
        baseline.usage().actual_capacity_bytes - 2 * size_of::<Vec<u16>>() as u64;
    t.release(&mut baseline).unwrap();
    assert_storage_released(&baseline);
    let caps = limits()
        .into_builder()
        .managed_capacity_bytes(reserve_capacity - 1)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    prepare_quoted_attribute(&mut t, &mut c);
    let before = t.tag.as_ref().unwrap().attributes.capacity();
    let stop = t.finish_attribute(&mut c).unwrap_err();
    assert!(matches!(
        stop,
        ParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            ..
        }
    ));
    let tag = t.tag.as_ref().unwrap();
    assert_eq!(
        (tag.attributes.len(), tag.attributes.capacity()),
        (0, before)
    );
    let draft = tag.attribute.as_ref().unwrap();
    assert_eq!(draft.name.value.units(), [120]);
    assert_eq!(draft.value.value.units(), [118]);
    let snapshot = c.first_stop_usage().unwrap().clone();
    assert_eq!(t.finish_attribute(&mut c), Err(stop));
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
    assert_eq!(c.usage().actual_capacity_bytes, 0);
    assert_eq!(c.first_stop_usage(), Some(&snapshot));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits),
        snapshot.resource(HtmlParseResource::WorkCredits)
    );
}

#[test]
fn retained_attribute_growth_preserves_live_payload_alias_and_cancelled_draft() {
    let source = HtmlSource::utf8("<a x='v' y='w' z='u' >");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    prepare_quoted_attribute(&mut t, &mut c);
    t.finish_attribute(&mut c).unwrap();
    let alias = t.tag.as_ref().unwrap().attributes.as_slice()[0]
        .value
        .value
        .text()
        .clone();
    // Advance out of the old AfterAttributeValue state into the second draft.
    t.step(&mut c, false).unwrap();
    prepare_quoted_attribute(&mut t, &mut c);
    t.finish_attribute(&mut c).unwrap();
    assert_eq!(t.tag.as_ref().unwrap().attributes.len(), 2);
    assert_eq!(alias.code_units(), [118]);
    t.step(&mut c, false).unwrap();
    prepare_quoted_attribute(&mut t, &mut c);
    let retained = t.tag.as_ref().unwrap().attributes.capacity();
    cancellation.cancel();
    assert_eq!(t.finish_attribute(&mut c), Err(ParseStop::Cancelled));
    let tag = t.tag.as_ref().unwrap();
    assert_eq!(
        (tag.attributes.len(), tag.attributes.capacity()),
        (2, retained)
    );
    let draft = tag.attribute.as_ref().unwrap();
    assert_eq!(draft.name.value.units(), [122]);
    assert_eq!(draft.value.value.units(), [117]);
    assert_eq!(alias.code_units(), [118]);
    let snapshot = c.first_stop_usage().unwrap().clone();
    // The caller-owned alias remains valid and is dropped before normal
    // terminal disposal, without creating a second accounting ticket.
    drop(alias);
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
    assert_eq!(c.usage().actual_capacity_bytes, 0);
    assert_eq!(c.first_stop_usage(), Some(&snapshot));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits),
        snapshot.resource(HtmlParseResource::WorkCredits)
    );
}

#[test]
fn retained_attribute_growth_foreign_controller_refuses_without_consuming_draft() {
    let source = HtmlSource::utf8("<a x='v' >");
    let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    prepare_quoted_attribute(&mut t, &mut c);
    let before = c.usage().clone();
    let mut foreign = ParseControl::new(limits(), HtmlParseCancellation::new());
    foreign.admit_source(&source).unwrap();
    assert_eq!(
        t.finish_attribute(&mut foreign),
        Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
    );
    assert_eq!(c.usage(), &before);
    let tag = t.tag.as_ref().unwrap();
    assert_eq!(tag.attributes.len(), 0);
    let draft = tag.attribute.as_ref().unwrap();
    assert_eq!(draft.name.value.units(), [120]);
    assert_eq!(draft.value.value.units(), [118]);
    t.finish_attribute(&mut c).unwrap();
    assert_eq!(
        text(&t.tag.as_ref().unwrap().attributes.as_slice()[0].value),
        [118]
    );
    t.release(&mut c).unwrap();
    assert_storage_released(&c);
    assert_eq!(c.usage().actual_capacity_bytes, 0);
    assert_eq!(foreign.usage().actual_capacity_bytes, 0);
}
