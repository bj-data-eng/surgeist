//! Closed live-profile dispatch over the shared decoded tokenizer.

use std::{convert::Infallible, fmt, mem::size_of, num::NonZeroUsize};

use surgeist_html::{
    bounded::lexical::is_ascii_whitespace, construction::HtmlDocumentMode, values::DocumentString,
};

use crate::{
    DetailSet, HtmlParseCancellation, HtmlParseInvariant, HtmlParseLimits, HtmlParseResource,
    HtmlParseStop, HtmlParseUsage, HtmlParseWorkCategory, HtmlSource, HtmlSourceRange,
    construction::evidence::ParseEvidence,
    parser::{
        control::{OwnedVec, ParseControl},
        tokenizer::{
            Tokenizer, TokenizerPull,
            states::HtmlTokenizerState,
            token::{Event, Tag, TagKind, Token, TokenKind},
        },
    },
};

use super::{doctype, end, head, tag_sets, write};

/// HTML elements admitted by the local live profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlLiveElement {
    Html,
    Head,
    Body,
    Div,
    Span,
}

impl HtmlLiveElement {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Head => "head",
            Self::Body => "body",
            Self::Div => "div",
            Self::Span => "span",
        }
    }
}

/// Null-namespace content attributes admitted by the local live profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlLiveAttribute {
    Id,
    Class,
}

impl HtmlLiveAttribute {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Class => "class",
        }
    }
}

/// Original admitted attribute values in the tokenizer's source order.
#[derive(Clone, Copy, Debug)]
pub struct HtmlLiveAttributes<'a> {
    values: [Option<(HtmlLiveAttribute, &'a DocumentString)>; 2],
}

impl<'a> HtmlLiveAttributes<'a> {
    pub fn iter(&self) -> impl Iterator<Item = (HtmlLiveAttribute, &'a DocumentString)> {
        self.values.into_iter().flatten()
    }
}

/// The canonical tree provider for one active Document.
///
/// Each call publishes its entire operation on success, admitting any retained
/// string backing to the canonical owner. A refusal leaves the tree unchanged
/// and retains no offered string. Creation returns the fresh canonical identity.
/// The driver holds no provider borrow between calls or across a pause.
pub trait HtmlLiveTree {
    type Node: Copy + Eq;
    type Error;

    fn insert_doctype(&mut self, document: Self::Node) -> Result<(), Self::Error>;

    fn insert_element(
        &mut self,
        parent: Self::Node,
        name: HtmlLiveElement,
        attributes: HtmlLiveAttributes<'_>,
    ) -> Result<Self::Node, Self::Error>;

    fn insert_comment(
        &mut self,
        parent: Self::Node,
        text: &DocumentString,
    ) -> Result<(), Self::Error>;

    /// Append to the canonical last child if it is Text, or append a fresh
    /// Text node. The preceding-child read belongs to the same atomic edit.
    fn insert_text(&mut self, parent: Self::Node, text: &DocumentString)
    -> Result<(), Self::Error>;
}

/// The first reached tokenizer result outside the admitted construction profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlLiveUnsupported {
    Doctype,
    Element,
    Attribute,
    DocumentStructure,
    Text,
    ProcessingInstruction,
}

/// A construction refusal or a parser lifecycle outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlLiveParseError<E> {
    Parser(HtmlParseStop),
    Unsupported {
        reason: HtmlLiveUnsupported,
        source: HtmlSourceRange,
    },
    Host(E),
    Aborted,
    HandedOff,
}

impl<E> From<HtmlParseStop> for HtmlLiveParseError<E> {
    fn from(value: HtmlParseStop) -> Self {
        Self::Parser(value)
    }
}

/// A safe point after applying complete tokenizer results in source order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlLiveParseProgress {
    Paused,
    AwaitingEof,
    StopParsing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    AfterBody,
    AfterAfterBody,
}

struct OpenElement<N> {
    name: HtmlLiveElement,
    node: Option<N>,
}

type AttributeNames = [Option<HtmlLiveAttribute>; 2];

enum Edit<N> {
    None,
    Pop,
    Doctype,
    Element { parent: N, names: AttributeNames },
    Comment(N),
    Text(N),
}

struct Plan<N> {
    edit: Edit<N>,
    next: Mode,
}

struct Input<N> {
    token: Token,
    plan: Option<Plan<N>>,
}

struct State<'source, N> {
    control: ParseControl,
    tokenizer: Option<Tokenizer<'source>>,
    evidence: Option<ParseEvidence>,
    pending: Option<Input<N>>,
    mode: Mode,
    open: OwnedVec<OpenElement<N>>,
    accept_eof: bool,
    end: Option<end::EndPhase>,
}

/// One source-established parser for an exact hosted Document root.
///
/// Supply the full bounded decoded input at construction. EOF dispatch is
/// controlled separately. A host refusal keeps the same token and preadmitted
/// transition for retry. Cancellation and reached unsupported work are sticky.
pub struct HtmlLiveParser<'source, N> {
    source: &'source HtmlSource,
    document: N,
    cancellation: HtmlParseCancellation,
    state: Option<State<'source, N>>,
    unsupported: Option<(HtmlLiveUnsupported, HtmlSourceRange)>,
    first_host_refusal: Option<HtmlSourceRange>,
    terminal_stop: Option<HtmlParseStop>,
    handed_off: bool,
}

impl<N: fmt::Debug> fmt::Debug for HtmlLiveParser<'_, N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlLiveParser")
            .field("source", self.source.identity())
            .field("document", &self.document)
            .field("state", &self.state.as_ref().map(|state| state.mode))
            .finish_non_exhaustive()
    }
}

impl<'source, N: Copy + Eq> HtmlLiveParser<'source, N> {
    pub fn new(
        source: &'source HtmlSource,
        document: N,
        limits: HtmlParseLimits,
    ) -> Result<Self, HtmlLiveParseError<Infallible>> {
        let cancellation = HtmlParseCancellation::new();
        let mut control =
            ParseControl::with_accounting(limits, cancellation.clone(), None, DetailSet::NONE);
        control.admit_source(source)?;
        write(size_of::<Self>(), 1, &mut control)?;
        let mut open = OwnedVec::new(HtmlParseResource::OpenElements)?;
        open.require_owner(&mut control)?;
        let evidence = ParseEvidence::new(&mut control)?;
        let tokenizer = match Tokenizer::new(source, HtmlTokenizerState::Data, None, &mut control) {
            Ok(tokenizer) => tokenizer,
            Err(stop) => {
                evidence.release(&mut control)?;
                return Err(stop.into());
            }
        };
        Ok(Self {
            source,
            document,
            cancellation,
            state: Some(State {
                control,
                tokenizer: Some(tokenizer),
                evidence: Some(evidence),
                pending: None,
                mode: Mode::Initial,
                open,
                accept_eof: false,
                end: None,
            }),
            unsupported: None,
            first_host_refusal: None,
            terminal_stop: None,
            handed_off: false,
        })
    }

    pub fn document(&self) -> N {
        self.document
    }

    pub fn cancellation(&self) -> &HtmlParseCancellation {
        &self.cancellation
    }

    pub fn usage(&self) -> Option<HtmlParseUsage> {
        self.state.as_ref().map(|state| state.control.usage())
    }

    pub fn first_host_refusal(&self) -> Option<&HtmlSourceRange> {
        self.first_host_refusal.as_ref()
    }

    /// Allow the actual source EOF token to enter stop parsing on an advance.
    pub fn accept_eof(&mut self) {
        if let Some(state) = &mut self.state {
            state.accept_eof = true;
        }
    }

    /// Apply up to the supplied number of original tokens. Tokenizer work
    /// within a token and across advances shares the original parse limits.
    pub fn advance<H: HtmlLiveTree<Node = N>>(
        &mut self,
        host: &mut H,
        token_limit: NonZeroUsize,
    ) -> Result<HtmlLiveParseProgress, HtmlLiveParseError<H::Error>> {
        if let Some(error) = self.refusal() {
            return Err(error);
        }
        let state = self.state.as_mut().ok_or(HtmlLiveParseError::Aborted)?;
        let result = state.advance(self.source, self.document, host, token_limit);
        match &result {
            Err(HtmlLiveParseError::Unsupported { reason, source }) => {
                self.unsupported = Some((*reason, source.clone()));
            }
            Err(HtmlLiveParseError::Host(_)) if self.first_host_refusal.is_none() => {
                self.first_host_refusal = state.pending.as_ref().map(|p| p.token.source.clone());
            }
            Err(HtmlLiveParseError::Parser(stop)) => {
                self.terminal_stop = Some(*stop);
            }
            _ => {}
        }
        result
    }

    fn refusal<E>(&self) -> Option<HtmlLiveParseError<E>> {
        if let Some((reason, source)) = &self.unsupported {
            Some(HtmlLiveParseError::Unsupported {
                reason: *reason,
                source: source.clone(),
            })
        } else if let Some(stop) = self.terminal_stop {
            Some(HtmlLiveParseError::Parser(stop))
        } else if self.handed_off {
            Some(HtmlLiveParseError::HandedOff)
        } else {
            None
        }
    }

    /// Transfer the already reached HTML stop-parsing continuation once.
    ///
    /// The host must validate its active Document and Global before calling.
    /// Producer-side cancellation is checked before custody changes.
    pub fn take_stop_parsing(
        &mut self,
    ) -> Result<Option<HtmlLiveStopParsing<N>>, HtmlLiveParseError<Infallible>> {
        if self.handed_off {
            return Ok(None);
        }
        if let Some(error) = self.refusal() {
            return Err(error);
        }
        let state = self.state.as_mut().ok_or(HtmlLiveParseError::Aborted)?;
        if let Err(stop) = state.control.checkpoint() {
            self.terminal_stop = Some(stop);
            return Err(stop.into());
        }
        let Some(phase) = state.end else {
            return Ok(None);
        };
        let Some(input) = state.pending.take() else {
            let stop = invalid_state(&mut state.control);
            self.terminal_stop = Some(stop);
            return Err(stop.into());
        };
        let state = self.state.take().ok_or(HtmlLiveParseError::Aborted)?;
        self.handed_off = true;
        Ok(Some(HtmlLiveStopParsing {
            document: self.document,
            eof: input.token,
            open: state.open,
            control: state.control,
            phase,
        }))
    }

    /// Release parser custody without rolling back published canonical edits.
    pub fn abort(&mut self) -> Result<(), HtmlParseStop> {
        let Some(mut state) = self.state.take() else {
            return Ok(());
        };
        state.control.cancelled_stop();
        let pending = state
            .pending
            .take()
            .map_or(Ok(()), |input| input.token.release(&mut state.control));
        let lexer = state.release_lexical();
        let open = state.open.release(&mut state.control);
        pending.and(lexer).and(open)
    }
}

/// Original end-of-input continuation for the Document that the parser edited.
pub struct HtmlLiveStopParsing<N> {
    document: N,
    eof: Token,
    open: OwnedVec<OpenElement<N>>,
    control: ParseControl,
    phase: end::EndPhase,
}

impl<N: Copy> HtmlLiveStopParsing<N> {
    pub fn document(&self) -> N {
        self.document
    }

    pub fn eof(&self) -> &HtmlSourceRange {
        &self.eof.source
    }

    /// The canonical nodes awaiting the original stop-parsing stack pops.
    pub fn open_elements(&self) -> impl DoubleEndedIterator<Item = N> + '_ {
        self.open.as_slice().iter().filter_map(|entry| entry.node)
    }

    /// No reached token could create a pending parser-inserted script in this
    /// profile. Independent host load-delay producers require qualification.
    pub fn parser_scripts_are_absent(&self) -> bool {
        true
    }

    pub fn usage(&self) -> HtmlParseUsage {
        self.control.usage()
    }
}

impl<N: fmt::Debug> fmt::Debug for HtmlLiveStopParsing<N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlLiveStopParsing")
            .field("document", &self.document)
            .field("eof", &self.eof.source)
            .field("phase", &self.phase)
            .finish_non_exhaustive()
    }
}

fn invalid_state(control: &mut ParseControl) -> HtmlParseStop {
    control.fail_invariant(HtmlParseInvariant::TreeBuilderState)
}

impl<N: Copy + Eq> State<'_, N> {
    fn release_lexical(&mut self) -> Result<(), HtmlParseStop> {
        let tokenizer = self
            .tokenizer
            .take()
            .map_or(Ok(()), |tokenizer| tokenizer.release(&mut self.control));
        let evidence = self
            .evidence
            .take()
            .map_or(Ok(()), |evidence| evidence.release(&mut self.control));
        tokenizer.and(evidence)
    }

    fn pull(&mut self, source: &HtmlSource) -> Result<(), HtmlParseStop> {
        while self.pending.is_none() {
            write(size_of::<Option<Input<N>>>(), 1, &mut self.control)?;
            let Some(tokenizer) = self.tokenizer.as_mut() else {
                return Err(invalid_state(&mut self.control));
            };
            let Some(evidence) = self.evidence.as_mut() else {
                return Err(invalid_state(&mut self.control));
            };
            match tokenizer.next(&mut self.control, false, evidence)? {
                TokenizerPull::Event(Event::Token(event)) => {
                    self.pending = Some(Input {
                        token: event.into_token(&mut self.control)?,
                        plan: None,
                    });
                    let Some(input) = &self.pending else {
                        return Err(invalid_state(&mut self.control));
                    };
                    if let Err(error) =
                        head::validate_source(source, &input.token, &mut self.control)
                    {
                        error.stop_cause().latch(&mut self.control);
                        return Err(self
                            .control
                            .stop()
                            .unwrap_or_else(|| invalid_state(&mut self.control)));
                    }
                }
                TokenizerPull::Event(other) => other.release(&mut self.control)?,
                TokenizerPull::Exhausted => return Err(invalid_state(&mut self.control)),
            }
        }
        Ok(())
    }

    fn advance<H: HtmlLiveTree<Node = N>>(
        &mut self,
        source: &HtmlSource,
        document: N,
        host: &mut H,
        token_limit: NonZeroUsize,
    ) -> Result<HtmlLiveParseProgress, HtmlLiveParseError<H::Error>> {
        self.control.require_source(source)?;
        if self.end.is_some() {
            return Ok(HtmlLiveParseProgress::StopParsing);
        }
        for _ in 0..token_limit.get() {
            self.pull(source)?;
            let Some(input) = &mut self.pending else {
                return Err(invalid_state(&mut self.control).into());
            };
            self.control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if matches!(input.token.kind, TokenKind::Eof) {
                if end::eof_is_held(true, self.accept_eof) {
                    return Ok(HtmlLiveParseProgress::AwaitingEof);
                }
                if !matches!(
                    self.mode,
                    Mode::InBody | Mode::AfterBody | Mode::AfterAfterBody
                ) {
                    return Err(unsupported(
                        HtmlLiveUnsupported::DocumentStructure,
                        &input.token,
                    ));
                }
                if self.open.len() != 2 {
                    return Err(unsupported(
                        HtmlLiveUnsupported::DocumentStructure,
                        &input.token,
                    ));
                }
                let phase = end::admit_stop_parsing(&mut self.control)?;
                self.release_lexical()?;
                self.end = Some(phase);
                return Ok(HtmlLiveParseProgress::StopParsing);
            }

            if input.plan.is_none() {
                input.plan = Some(prepare(
                    input,
                    document,
                    self.mode,
                    &mut self.open,
                    &mut self.control,
                )?);
            }
            let Some(plan) = &input.plan else {
                return Err(invalid_state(&mut self.control).into());
            };
            self.control
                .charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            apply(
                plan,
                &mut input.token,
                document,
                &mut self.open,
                &mut self.control,
                host,
            )?;
            self.mode = plan.next;
            if let Some(input) = self.pending.take() {
                input.token.release(&mut self.control)?;
            }
        }
        Ok(HtmlLiveParseProgress::Paused)
    }
}

fn unsupported<E>(reason: HtmlLiveUnsupported, token: &Token) -> HtmlLiveParseError<E> {
    HtmlLiveParseError::Unsupported {
        reason,
        source: token.source.clone(),
    }
}

fn classify<E>(
    token: &Token,
    mode: Mode,
    current: Option<HtmlLiveElement>,
    control: &mut ParseControl,
) -> Result<Rule, HtmlLiveParseError<E>> {
    use HtmlLiveElement::{Body, Div, Head, Html, Span};
    use Mode::{
        AfterAfterBody, AfterBody, AfterHead, BeforeHead, BeforeHtml, InBody, InHead, Initial,
    };

    let rule = match &token.kind {
        TokenKind::Doctype(doctype) if mode == Initial => {
            let decision = doctype::classify(doctype, control)?;
            if decision.mode != HtmlDocumentMode::NoQuirks
                || decision.error
                || doctype.public_id.is_some()
                || doctype.system_id.is_some()
            {
                return Err(unsupported(HtmlLiveUnsupported::Doctype, token));
            }
            Rule::Doctype
        }
        TokenKind::Doctype(_) => return Err(unsupported(HtmlLiveUnsupported::Doctype, token)),
        TokenKind::Text(text) => {
            let mut whitespace = true;
            for &unit in text.value.text().code_units() {
                control.charge_unit_inspection()?;
                if unit == 0 {
                    return Err(unsupported(HtmlLiveUnsupported::Text, token));
                }
                whitespace &= is_ascii_whitespace(unit);
            }
            match mode {
                Initial | BeforeHtml | BeforeHead if whitespace => Rule::Ignore,
                InBody => Rule::Text,
                InHead | AfterHead | AfterBody | AfterAfterBody if whitespace => Rule::Text,
                _ => return Err(unsupported(HtmlLiveUnsupported::DocumentStructure, token)),
            }
        }
        TokenKind::Comment(_) => Rule::Comment,
        TokenKind::Tag(tag) => {
            let Some(name) = element_name(tag, control)? else {
                return Err(unsupported(HtmlLiveUnsupported::Element, token));
            };
            if tag.self_closing {
                return Err(unsupported(HtmlLiveUnsupported::DocumentStructure, token));
            }
            match (mode, tag.kind, name) {
                (BeforeHtml, TagKind::Start, Html) => Rule::Push(Html, BeforeHead),
                (BeforeHead, TagKind::Start, Head) => Rule::Push(Head, InHead),
                (InHead, TagKind::End, Head) => Rule::Pop(AfterHead),
                (AfterHead, TagKind::Start, Body) => Rule::Push(Body, InBody),
                (InBody, TagKind::Start, Div | Span) => Rule::Push(name, InBody),
                (InBody, TagKind::End, Div | Span) if current == Some(name) => Rule::Pop(InBody),
                (InBody, TagKind::End, Body) if current == Some(Body) => Rule::Set(AfterBody),
                (AfterBody, TagKind::End, Html) => Rule::Set(AfterAfterBody),
                _ => return Err(unsupported(HtmlLiveUnsupported::DocumentStructure, token)),
            }
        }
        TokenKind::ProcessingInstruction { .. } => {
            return Err(unsupported(
                HtmlLiveUnsupported::ProcessingInstruction,
                token,
            ));
        }
        TokenKind::Eof => return Err(invalid_state(control).into()),
    };
    Ok(rule)
}

enum Rule {
    Ignore,
    Doctype,
    Push(HtmlLiveElement, Mode),
    Pop(Mode),
    Set(Mode),
    Comment,
    Text,
}

fn element_name(
    tag: &Tag,
    control: &mut ParseControl,
) -> Result<Option<HtmlLiveElement>, HtmlParseStop> {
    use HtmlLiveElement::{Body, Div, Head, Html, Span};
    for name in [Html, Head, Body, Div, Span] {
        if tag_sets::ascii_eq(tag.name.value.text(), name.as_str(), control)? {
            return Ok(Some(name));
        }
    }
    Ok(None)
}

fn attributes<E>(
    tag: &Tag,
    token: &Token,
    control: &mut ParseControl,
) -> Result<AttributeNames, HtmlLiveParseError<E>> {
    let mut names = [None, None];
    let source = tag.attributes.as_slice();
    if source.len() > names.len() || tag.had_duplicate_attributes {
        return Err(unsupported(HtmlLiveUnsupported::Attribute, token));
    }
    for (attribute, destination) in source.iter().zip(&mut names) {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let name = attribute.name.value.text();
        *destination = Some(if tag_sets::ascii_eq(name, "id", control)? {
            HtmlLiveAttribute::Id
        } else if tag_sets::ascii_eq(name, "class", control)? {
            HtmlLiveAttribute::Class
        } else {
            return Err(unsupported(HtmlLiveUnsupported::Attribute, token));
        });
    }
    Ok(names)
}

fn prepare<N: Copy, E>(
    input: &Input<N>,
    document: N,
    mode: Mode,
    open: &mut OwnedVec<OpenElement<N>>,
    control: &mut ParseControl,
) -> Result<Plan<N>, HtmlLiveParseError<E>> {
    let token = &input.token;
    control.charge(HtmlParseWorkCategory::StateStep, 1)?;
    control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
    let top = open.as_slice().last();
    let rule = classify(token, mode, top.map(|entry| entry.name), control)?;
    // Admit the transition and output identity before any provider call.
    write(size_of::<Option<Plan<N>>>(), 3, control)?;
    let current = || top.and_then(|entry| entry.node);
    let (edit, next) = match rule {
        Rule::Ignore => (Edit::None, mode),
        Rule::Set(next) => (Edit::None, next),
        Rule::Doctype => (Edit::Doctype, Mode::BeforeHtml),
        Rule::Push(name, next) => {
            let parent = if name == HtmlLiveElement::Html {
                document
            } else {
                current().ok_or_else(|| invalid_state(control))?
            };
            let TokenKind::Tag(tag) = &token.kind else {
                return Err(invalid_state(control).into());
            };
            let names = attributes(tag, token, control)?;
            control.observe_tree_depth((open.len() as u64).saturating_add(1))?;
            write(size_of::<Option<N>>(), 1, control)?;
            open.try_push_geometric(control, OpenElement { name, node: None })?;
            (Edit::Element { parent, names }, next)
        }
        Rule::Pop(next) => (Edit::Pop, next),
        Rule::Comment => {
            let parent = match mode {
                Mode::Initial | Mode::BeforeHtml | Mode::AfterAfterBody => document,
                Mode::AfterBody => open
                    .as_slice()
                    .first()
                    .and_then(|entry| entry.node)
                    .ok_or_else(|| invalid_state(control))?,
                _ => current().ok_or_else(|| invalid_state(control))?,
            };
            (Edit::Comment(parent), mode)
        }
        Rule::Text => (
            Edit::Text(current().ok_or_else(|| invalid_state(control))?),
            mode,
        ),
    };
    Ok(Plan { edit, next })
}

fn apply<H: HtmlLiveTree>(
    plan: &Plan<H::Node>,
    token: &mut Token,
    document: H::Node,
    open: &mut OwnedVec<OpenElement<H::Node>>,
    control: &mut ParseControl,
    host: &mut H,
) -> Result<(), HtmlLiveParseError<H::Error>> {
    match (&plan.edit, &mut token.kind) {
        (Edit::None, _) => Ok(()),
        (Edit::Pop, _) => {
            open.try_pop(control)?
                .ok_or_else(|| invalid_state(control))?;
            Ok(())
        }
        (Edit::Doctype, TokenKind::Doctype(_)) => host
            .insert_doctype(document)
            .map_err(HtmlLiveParseError::Host),
        (Edit::Text(parent), TokenKind::Text(text)) => {
            write(size_of::<bool>(), 1, control)?;
            host.insert_text(*parent, text.value.text())
                .map_err(HtmlLiveParseError::Host)?;
            text.value.mark_shared();
            Ok(())
        }
        (Edit::Comment(parent), TokenKind::Comment(text)) => {
            write(size_of::<bool>(), 1, control)?;
            host.insert_comment(*parent, text.value.text())
                .map_err(HtmlLiveParseError::Host)?;
            text.value.mark_shared();
            Ok(())
        }
        (Edit::Element { parent, names }, TokenKind::Tag(tag)) => {
            let attributes = tag.attributes.as_mut_slice_prepaid(control)?;
            write(attributes.len(), attributes.len(), control)?;
            let mut values = [None, None];
            for ((name, attribute), value) in names
                .iter()
                .flatten()
                .zip(attributes.iter())
                .zip(&mut values)
            {
                *value = Some((*name, attribute.value.value.text()));
            }
            let Some(open) = open.as_mut_slice_prepaid(control)?.last_mut() else {
                return Err(invalid_state(control).into());
            };
            let node = host
                .insert_element(*parent, open.name, HtmlLiveAttributes { values })
                .map_err(HtmlLiveParseError::Host)?;
            for attribute in attributes {
                attribute.value.value.mark_shared();
            }
            open.node = Some(node);
            Ok(())
        }
        _ => Err(invalid_state(control).into()),
    }
}

#[cfg(test)]
mod tests;
