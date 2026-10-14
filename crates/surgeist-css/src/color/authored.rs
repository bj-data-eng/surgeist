//! Ordinary authored construction from the canonical color recipe grammar.
//!
//! Logical event offsets below identify temporary records, never source bytes.
//! Original components remain the sole source of native numeric values and
//! origins. This adapter uses ordinary checked allocation, not a controlled
//! memory or cancellation contract.

use super::component::{ComponentAction, SyntaxFailure};
use super::grammar::{
    ColorGrammar, ComponentRecord, FunctionRecipe, GrammarAction, GrammarRecord, Interpolation,
    Leaf, ProfileSpace, RawComponent, Recipe, RecipeKind,
};
use super::keywords::ColorFunction as F;
use super::protocol::{
    CssColorBank, CssColorOperationOwner, CssColorProtocolError, CssColorSlot, Data, LexicalTable,
};
use super::syntax_pass::{SyntaxAction, SyntaxPass};
use crate::error::{Error, invalid_color};
use crate::numeric::{CalculationRoot, NumericInputContext};
use crate::tokenization::color_tokens::{NameKind, NumberKind, Span, Token, TokenKind};
use crate::*;
use cssparser::{ParseError, Parser, SourceLocation};

type Result<'i, T> = std::result::Result<T, ParseError<'i, Error>>;

pub(crate) fn parse_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssColor> {
    input.skip_whitespace();
    let offset = input.position().byte_index();
    let location = input.current_source_location();
    let root = numeric
        .collect(input)
        .map_err(|error| numeric_error(numeric, error, location, offset))?;
    construct(&root, numeric, location, offset)
}

fn numeric_error<'i>(
    numeric: &NumericInputContext<'_>,
    error: CssNumericConstructionError,
    fallback: SourceLocation,
    offset: usize,
) -> ParseError<'i, Error> {
    let at = numeric.error_location(&error, fallback, offset);
    if let Some(component) = error.component_error() {
        crate::error::invalid_component_value(at, component.clone())
    } else {
        crate::error::with_color_context(crate::error::unexpected_at(at), None)
    }
}

struct Events<'a, 'owner> {
    slots: Vec<CssColorSlot<'owner>>,
    originals: Vec<Option<&'a CssComponentValue>>,
    paths: Vec<Option<Vec<usize>>>,
    closing: Vec<bool>,
    eof: Option<EndSite<'a>>,
    owner: &'owner CssColorOperationOwner,
    phase: usize,
}
struct EndSite<'a> {
    component: &'a CssComponentValue,
    path: Vec<usize>,
}
impl<'a, 'owner> Events<'a, 'owner> {
    fn append(
        &mut self,
        data: Data<'owner>,
        original: Option<&'a CssComponentValue>,
    ) -> std::result::Result<usize, CssColorProtocolError> {
        let index = self.slots.len();
        index
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        self.slots
            .try_reserve(1)
            .map_err(|_| CssColorProtocolError::Overflow)?;
        self.originals
            .try_reserve(1)
            .map_err(|_| CssColorProtocolError::Overflow)?;
        self.paths
            .try_reserve(1)
            .map_err(|_| CssColorProtocolError::Overflow)?;
        self.closing
            .try_reserve(1)
            .map_err(|_| CssColorProtocolError::Overflow)?;
        self.slots
            .push(CssColorSlot::new(self.owner, self.phase, index, data));
        self.originals.push(original);
        self.paths.push(None);
        self.closing.push(false);
        Ok(index)
    }
    fn name(&mut self, value: &str) -> std::result::Result<Span, CssColorProtocolError> {
        let start = self.slots.len();
        for unit in value.chars() {
            self.append(Data::NameUnit(unit), None)?;
        }
        Ok(Span {
            start,
            end: self.slots.len(),
        })
    }
    fn number(
        &mut self,
        value: CssNumericTokenRef<'_>,
        kind: NumberKind,
        unit: &str,
    ) -> std::result::Result<TokenKind, CssColorProtocolError> {
        use crate::tokenization::number::{NumberCursor, Part, Step};
        let spelling = value.representation().as_bytes();
        let start = self.slots.len();
        let mut cursor = NumberCursor::new();
        let mut offset = 0usize;
        let layout = loop {
            let window = [0, 1, 2].map(|delta| {
                offset
                    .checked_add(delta)
                    .and_then(|at| spelling.get(at))
                    .copied()
                    .map(char::from)
            });
            match cursor
                .step(window)
                .map_err(|_| CssColorProtocolError::Overflow)?
            {
                Step::Consume(part) => {
                    match part {
                        Part::Mantissa(value) => {
                            self.append(
                                Data::NumberDigit {
                                    exponent: false,
                                    value,
                                },
                                None,
                            )?;
                        }
                        Part::ExponentDigit(value) => {
                            self.append(
                                Data::NumberDigit {
                                    exponent: true,
                                    value,
                                },
                                None,
                            )?;
                        }
                        Part::Sign | Part::Point | Part::Exponent | Part::ExponentSign => {}
                    }
                    offset = offset
                        .checked_add(1)
                        .ok_or(CssColorProtocolError::Overflow)?;
                }
                Step::Transition => {}
                Step::Done(layout) => break layout,
            }
        };
        if offset != spelling.len() {
            return Err(CssColorProtocolError::Input);
        }
        let digits = Span {
            start,
            end: self.slots.len(),
        };
        let unit = self.name(unit)?;
        Ok(TokenKind::Number {
            kind,
            layout,
            digits,
            unit,
        })
    }
    fn token(
        &mut self,
        kind: TokenKind,
        original: &'a CssComponentValue,
        path: &[usize],
    ) -> std::result::Result<(), CssColorProtocolError> {
        let start = self.slots.len();
        let end = start
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        let mut retained = Vec::new();
        retained
            .try_reserve(path.len())
            .map_err(|_| CssColorProtocolError::Overflow)?;
        retained.extend_from_slice(path);
        self.append(
            Data::Token(Token {
                kind,
                source: Span { start, end },
            }),
            Some(original),
        )?;
        self.paths[start] = Some(retained);
        Ok(())
    }
    fn project(
        &mut self,
        root: &'a CssComponentValue,
    ) -> std::result::Result<(), CssColorProtocolError> {
        enum Visit<'a> {
            Item(&'a CssComponentValue),
            Items(&'a [CssComponentValue], usize),
            Close(&'a CssComponentValue, char, bool),
        }
        let mut path = Vec::new();
        let mut pending = Vec::new();
        pending
            .try_reserve(1)
            .map_err(|_| CssColorProtocolError::Overflow)?;
        pending.push(Visit::Item(root));
        while let Some(visit) = pending.pop() {
            match visit {
                Visit::Items(items, index) => {
                    if let Some((first, rest)) = items.split_first() {
                        pending
                            .try_reserve(2)
                            .map_err(|_| CssColorProtocolError::Overflow)?;
                        pending.push(Visit::Items(
                            rest,
                            index
                                .checked_add(1)
                                .ok_or(CssColorProtocolError::Overflow)?,
                        ));
                        path.try_reserve(1)
                            .map_err(|_| CssColorProtocolError::Overflow)?;
                        path.push(index);
                        pending.push(Visit::Item(first));
                    }
                }
                Visit::Close(item, delimiter, closed) => {
                    if closed {
                        let site = self.slots.len();
                        self.token(TokenKind::Delimiter(delimiter), item, &path)?;
                        self.closing[site] = true;
                    } else if self.eof.is_none() {
                        let mut retained = Vec::new();
                        retained
                            .try_reserve(path.len())
                            .map_err(|_| CssColorProtocolError::Overflow)?;
                        retained.extend_from_slice(&path);
                        self.eof = Some(EndSite {
                            component: item,
                            path: retained,
                        });
                    }
                    path.pop();
                }
                Visit::Item(item) => {
                    let kind = match item.view() {
                        CssComponentValueRef::Function(function) => {
                            let units = self.name(function.name())?;
                            pending
                                .try_reserve(2)
                                .map_err(|_| CssColorProtocolError::Overflow)?;
                            pending.push(Visit::Close(
                                item,
                                ')',
                                !matches!(
                                    function.closing_origin(),
                                    CssValueOrigin::ImplicitClosure { .. }
                                ),
                            ));
                            pending.push(Visit::Items(function.values().items(), 0));
                            TokenKind::Name {
                                kind: NameKind::Function,
                                units,
                            }
                        }
                        CssComponentValueRef::Block(block) => {
                            let (open, close) = match block.kind() {
                                CssBlockKind::Parenthesis => ('(', ')'),
                                CssBlockKind::SquareBracket => ('[', ']'),
                                CssBlockKind::CurlyBracket => ('{', '}'),
                            };
                            pending
                                .try_reserve(2)
                                .map_err(|_| CssColorProtocolError::Overflow)?;
                            pending.push(Visit::Close(
                                item,
                                close,
                                !matches!(
                                    block.closing_origin(),
                                    CssValueOrigin::ImplicitClosure { .. }
                                ),
                            ));
                            pending.push(Visit::Items(block.values().items(), 0));
                            TokenKind::Delimiter(open)
                        }
                        CssComponentValueRef::Comment(_) => TokenKind::Comment { closed: true },
                        CssComponentValueRef::Token(token) => match token {
                            CssValueTokenRef::Ident(name) => TokenKind::Name {
                                kind: NameKind::Ident,
                                units: self.name(name)?,
                            },
                            CssValueTokenRef::AtKeyword(name) => TokenKind::Name {
                                kind: NameKind::At,
                                units: self.name(name)?,
                            },
                            CssValueTokenRef::Hash { value, flag } => TokenKind::Name {
                                kind: NameKind::Hash {
                                    identifier: flag == CssHashFlag::Id,
                                },
                                units: self.name(value)?,
                            },
                            CssValueTokenRef::Number(value) => {
                                self.number(value, NumberKind::Number, "")?
                            }
                            CssValueTokenRef::Percentage(value) => {
                                self.number(value, NumberKind::Percentage, "")?
                            }
                            CssValueTokenRef::Dimension { number, unit } => {
                                self.number(number, NumberKind::Dimension, unit)?
                            }
                            CssValueTokenRef::Whitespace(_) => TokenKind::Whitespace,
                            CssValueTokenRef::Delim(unit) => TokenKind::Delimiter(unit),
                            CssValueTokenRef::Comma => TokenKind::Delimiter(','),
                            CssValueTokenRef::Colon => TokenKind::Delimiter(':'),
                            CssValueTokenRef::Semicolon => TokenKind::Delimiter(';'),
                            CssValueTokenRef::Cdo => TokenKind::Cdo,
                            CssValueTokenRef::Cdc => TokenKind::Cdc,
                            CssValueTokenRef::String(value) => TokenKind::String {
                                units: self.name(value)?,
                                closed: true,
                            },
                            CssValueTokenRef::Url(_) => return Err(CssColorProtocolError::Input),
                        },
                    };
                    self.token(kind, item, &path)?;
                    if !matches!(
                        item.view(),
                        CssComponentValueRef::Function(_) | CssComponentValueRef::Block(_)
                    ) {
                        path.pop();
                    }
                }
            }
        }
        Ok(())
    }
}

fn construct<'i>(
    root: &CssComponentValue,
    numeric: &NumericInputContext<'_>,
    location: SourceLocation,
    offset: usize,
) -> Result<'i, CssColor> {
    let capacity = || {
        crate::error::invalid_component_value(
            location,
            CssComponentValueError::new(
                CssComponentValueErrorKind::CapacityOverflow,
                root.origin().clone(),
            ),
        )
    };
    let protocol = |error| {
        if error == CssColorProtocolError::Overflow {
            capacity()
        } else {
            invalid_color(location, None)
        }
    };
    let mut owner = CssColorOperationOwner::new();
    let phase = owner.begin().map_err(protocol)?;
    let mut events = Events {
        slots: Vec::new(),
        originals: Vec::new(),
        paths: Vec::new(),
        closing: Vec::new(),
        eof: None,
        owner: &owner,
        phase,
    };
    events.project(root).map_err(protocol)?;
    let lexical =
        LexicalTable::component_events(&owner, phase, events.slots.len()).map_err(protocol)?;
    let mut grammar = ColorGrammar::start(&lexical, true);
    loop {
        match grammar.action() {
            GrammarAction::ReadLexical { index } => {
                grammar.read_lexical(index, &events.slots[index])
            }
            GrammarAction::ReadFrame { index } => grammar.read_frame(index, &events.slots[index]),
            GrammarAction::ReadHex { index } => grammar.read_hex(index, &events.slots[index]),
            GrammarAction::Append { index } => {
                let record = grammar.record(index).map_err(protocol)?;
                events.append(record.data, None).map_err(protocol)?;
                grammar.appended(index, &events.slots[index])
            }
            GrammarAction::Compute => grammar.compute(),
            GrammarAction::Done { .. } => break,
            GrammarAction::Failed { source } => {
                let component = match grammar.failure_reason(source).map_err(protocol)? {
                    super::grammar::GrammarFailureReason::Syntax => None,
                    super::grammar::GrammarFailureReason::HueInterpolation => {
                        Some("hue interpolation")
                    }
                    super::grammar::GrammarFailureReason::ColorSpace => Some("color space"),
                };
                if source.start == source.end
                    && let Some(end) = &events.eof
                {
                    return Err(invalid_color(
                        end_location(end.component, &end.path, numeric, location, offset),
                        component,
                    ));
                }
                let at = event_location(&events, source.start, numeric, location, offset, &[]);
                return Err(invalid_color(at, component));
            }
            GrammarAction::Stopped => return Err(invalid_color(location, None)),
        }
        .map_err(protocol)?;
    }
    let table = grammar.finish().map_err(protocol)?;
    let mut pass = SyntaxPass::start(table, events.slots.len(), 0).map_err(protocol)?;
    let mut arithmetic = Vec::new();
    loop {
        match pass.action() {
            SyntaxAction::ReadRecord { index } => pass.read_record(index, &events.slots[index]),
            SyntaxAction::ReadLegacyRgb { index } => {
                pass.read_legacy_rgb(index, &events.slots[index])
            }
            SyntaxAction::Component(action) => match action {
                ComponentAction::Read { bank, index } => {
                    let slots = match bank {
                        CssColorBank::Algorithm => &events.slots,
                        CssColorBank::Arithmetic => &arithmetic,
                        CssColorBank::Evaluation => return Err(invalid_color(location, None)),
                    };
                    pass.component_read(bank, index, &slots[index])
                }
                ComponentAction::Append { bank, index }
                | ComponentAction::Replace { bank, index } => {
                    let record = pass.component_record(bank, index).map_err(protocol)?;
                    let slots = match bank {
                        CssColorBank::Algorithm => &mut events.slots,
                        CssColorBank::Arithmetic => &mut arithmetic,
                        CssColorBank::Evaluation => return Err(invalid_color(location, None)),
                    };
                    if index == slots.len() {
                        slots.try_reserve(1).map_err(|_| capacity())?;
                        slots.push(record);
                    } else {
                        *slots
                            .get_mut(index)
                            .ok_or_else(|| invalid_color(location, None))? = record;
                    }
                    pass.component_written(bank, index, &slots[index])
                }
                ComponentAction::Compute | ComponentAction::Done | ComponentAction::Failed(_) => {
                    pass.compute()
                }
                ComponentAction::Stopped => return Err(invalid_color(location, None)),
            },
            SyntaxAction::Replace { index } => {
                events.slots[index] = pass.replacement(index).map_err(protocol)?;
                pass.replaced(index, &events.slots[index])
            }
            SyntaxAction::Done => break,
            SyntaxAction::Failed(failure) => {
                let (site, component) = match failure {
                    SyntaxFailure::Component { site, reason } => (site, Some(reason.label())),
                    SyntaxFailure::Numeric { site, .. } => (site, None),
                };
                let at = event_location(&events, site, numeric, location, offset, &[]);
                return Err(invalid_color(at, component));
            }
            SyntaxAction::Stopped => return Err(invalid_color(location, None)),
        }
        .map_err(protocol)?;
    }
    let checked = pass.finish().map_err(protocol)?;
    let mut native = Vec::new();
    native
        .try_reserve(checked.table.end)
        .map_err(|_| capacity())?;
    native.resize_with(checked.table.end, || None);
    let builder = Builder {
        events: &events,
        numeric,
        location,
        offset,
    };
    for index in checked.table.start..checked.table.end {
        if let GrammarRecord::Recipe(recipe) = checked
            .table
            .record(index, &events.slots[index])
            .map_err(protocol)?
        {
            let mut color = builder.recipe(recipe, &mut native)?;
            color.retain_original_closure(builder.closed(recipe)?);
            native[index] = Some(color);
        }
    }
    native[checked.table.root]
        .take()
        .ok_or_else(|| invalid_color(location, None))
}

fn site_error<'i>(
    events: &Events<'_, '_>,
    site: usize,
    numeric: &NumericInputContext<'_>,
    fallback: SourceLocation,
    offset: usize,
) -> ParseError<'i, Error> {
    let at = event_location(events, site, numeric, fallback, offset, &[]);
    invalid_color(at, None)
}

fn event_location(
    events: &Events<'_, '_>,
    site: usize,
    numeric: &NumericInputContext<'_>,
    fallback: SourceLocation,
    offset: usize,
    inside: &[usize],
) -> SourceLocation {
    if events.closing.get(site).copied().unwrap_or(false)
        && let (Some(Some(component)), Some(Some(path))) =
            (events.originals.get(site), events.paths.get(site))
    {
        return end_location(component, path, numeric, fallback, offset);
    }
    match numeric.ordinary() {
        NumericInputContext::Parsed(_) => {
            events
                .originals
                .get(site)
                .and_then(|v| *v)
                .map_or(fallback, |component| {
                    let error = CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::InvalidArgumentType,
                        Some(component),
                    );
                    numeric.error_location(&error, fallback, offset)
                })
        }
        NumericInputContext::Components(_, serialized) => {
            let Some(base) = serialized.component_path_at(offset) else {
                return fallback;
            };
            let Some(relative) = events.paths.get(site).and_then(Option::as_ref) else {
                return fallback;
            };
            let Some(length) = base
                .len()
                .checked_add(relative.len())
                .and_then(|n| n.checked_add(inside.len()))
            else {
                return fallback;
            };
            let mut path = Vec::new();
            if path.try_reserve(length).is_err() {
                return fallback;
            }
            path.extend_from_slice(base);
            path.extend_from_slice(relative);
            path.extend_from_slice(inside);
            let Some(at) = serialized.component_offset_for_path(&path) else {
                return fallback;
            };
            let position =
                crate::source::CssSourcePosition::from_byte_offset_in(serialized.as_css(), at);
            SourceLocation {
                line: position.line().value(),
                column: position.column().value() + 1,
            }
        }
        NumericInputContext::QuirkyLengths(_) => unreachable!("ordinary context"),
    }
}

fn end_location(
    component: &CssComponentValue,
    relative: &[usize],
    numeric: &NumericInputContext<'_>,
    fallback: SourceLocation,
    offset: usize,
) -> SourceLocation {
    let origin = match component.view() {
        CssComponentValueRef::Function(value) => value.closing_origin(),
        CssComponentValueRef::Block(value) => value.closing_origin(),
        _ => return fallback,
    };
    let position = match numeric.ordinary() {
        NumericInputContext::Parsed(source) => {
            let parsed = match origin {
                CssValueOrigin::Parsed(parsed)
                | CssValueOrigin::ImplicitClosure { at: parsed, .. } => parsed,
                _ => return fallback,
            };
            if !source.same_snapshot(parsed.source()) {
                return fallback;
            }
            parsed.span().start()
        }
        NumericInputContext::Components(_, serialized) => {
            let Some(base) = serialized.component_path_at(offset) else {
                return fallback;
            };
            let Some(length) = base.len().checked_add(relative.len()) else {
                return fallback;
            };
            let mut path = Vec::new();
            if path.try_reserve(length).is_err() {
                return fallback;
            }
            path.extend_from_slice(base);
            path.extend_from_slice(relative);
            // Serialization emits exactly one ASCII closing delimiter for
            // both explicit and implied Function/Block endings.
            let Some(at) = serialized
                .component_end_for_path(&path)
                .and_then(|end| end.checked_sub(1))
            else {
                return fallback;
            };
            crate::source::CssSourcePosition::from_byte_offset_in(serialized.as_css(), at)
        }
        NumericInputContext::QuirkyLengths(_) => unreachable!("ordinary context"),
    };
    SourceLocation {
        line: position.line().value(),
        column: position.column().value() + 1,
    }
}

struct Builder<'a, 'owner, 'context> {
    events: &'a Events<'a, 'owner>,
    numeric: &'a NumericInputContext<'context>,
    location: SourceLocation,
    offset: usize,
}

impl Builder<'_, '_, '_> {
    fn invalid<'i>(&self) -> ParseError<'i, Error> {
        invalid_color(self.location, None)
    }
    fn capacity<'i>(&self) -> ParseError<'i, Error> {
        crate::error::invalid_component_value(
            self.location,
            CssComponentValueError::new(
                CssComponentValueErrorKind::CapacityOverflow,
                self.numeric
                    .origin_at(self.offset)
                    .unwrap_or(CssValueOrigin::Programmatic),
            ),
        )
    }
    fn raw<'i>(&self, site: usize) -> Result<'i, &CssComponentValue> {
        self.events
            .originals
            .get(site)
            .and_then(|v| *v)
            .ok_or_else(|| self.invalid())
    }
    fn record<'i>(&self, index: usize) -> Result<'i, GrammarRecord> {
        match self.events.slots.get(index).map(|slot| slot.data) {
            Some(Data::Grammar(record)) => Ok(record),
            _ => Err(self.invalid()),
        }
    }
    fn component_record<'i>(&self, index: usize) -> Result<'i, ComponentRecord> {
        match self.record(index)? {
            GrammarRecord::Component(record) if record.checked.is_some() => Ok(record),
            _ => Err(self.invalid()),
        }
    }
    fn component<'i>(&self, index: usize) -> Result<'i, &CssComponentValue> {
        let record = self.component_record(index)?;
        let site = match record.component {
            RawComponent::Atom(token) | RawComponent::Math { opener: token, .. } => token.index,
            RawComponent::Authored { .. } => {
                unreachable!("construction consumes source grammar only")
            }
        };
        self.raw(site)
    }
    fn values<'i>(&self, index: usize) -> Result<'i, CssComponentValues> {
        CssComponentValues::try_new(vec![self.component(index)?.clone()])
            .map_err(|error| crate::error::invalid_component_value(self.location, error))
    }
    fn numeric_failure<'i>(
        &self,
        index: usize,
        error: CssNumericConstructionError,
    ) -> ParseError<'i, Error> {
        let site = self
            .component_record(index)
            .ok()
            .map(|record| match record.component {
                RawComponent::Atom(token) | RawComponent::Math { opener: token, .. } => token.index,
                RawComponent::Authored { .. } => {
                    unreachable!("construction consumes source grammar only")
                }
            });
        let location = match (site, self.numeric.ordinary()) {
            (Some(site), NumericInputContext::Components(..)) => {
                let inside = error
                    .path()
                    .and_then(|path| path.strip_prefix(&[0]))
                    .unwrap_or(&[]);
                event_location(
                    self.events,
                    site,
                    self.numeric,
                    self.location,
                    self.offset,
                    inside,
                )
            }
            _ => self
                .numeric
                .error_location(&error, self.location, self.offset),
        };
        if let Some(component) = error.component_error() {
            crate::error::invalid_component_value(location, component.clone())
        } else {
            crate::error::with_color_context(crate::error::unexpected_at(location), None)
        }
    }
    fn numeric<'i>(
        &self,
        index: usize,
        root: CalculationRoot,
    ) -> Result<'i, CssCalculationExpression> {
        self.numeric
            .admit(self.values(index)?, root)
            .map_err(|error| self.numeric_failure(index, error))
    }
    fn scalar<'i>(&self, index: usize) -> Result<'i, CssColorComponent> {
        let raw = self.component(index)?;
        if matches!(raw.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) if name.eq_ignore_ascii_case("none"))
        {
            return Ok(CssColorComponent::None);
        }
        if matches!(raw.view(), CssComponentValueRef::Token(_)) {
            return crate::color_scalar::component(raw.clone())
                .map_err(|error| crate::error::invalid_component_value(self.location, error));
        }
        let expression = self.numeric(index, CalculationRoot::NumberPercentage)?;
        Ok(match expression.result_type() {
            CssCalculationType::Number
                if expression.as_ref().numeric_type().percent_hint().is_some() =>
            {
                CssColorComponent::HintedNumberCalculation(
                    CssHintedNumberCalculation::from_expression(expression),
                )
            }
            CssCalculationType::Number => CssColorComponent::NumberCalculation(
                CssNumberCalculation::from_expression(expression),
            ),
            CssCalculationType::Percentage => CssColorComponent::PercentageCalculation(
                CssPercentageCalculation::from_expression(expression),
            ),
            _ => return Err(self.invalid()),
        })
    }
    fn hue<'i>(&self, index: usize) -> Result<'i, CssColorHue> {
        let raw = self.component(index)?;
        if matches!(raw.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) if name.eq_ignore_ascii_case("none"))
        {
            return Ok(CssColorHue::None);
        }
        if matches!(raw.view(), CssComponentValueRef::Token(_)) {
            return crate::color_scalar::hue(raw.clone())
                .map_err(|error| crate::error::invalid_component_value(self.location, error));
        }
        let domain = self
            .component_record(index)?
            .checked
            .ok_or_else(|| self.invalid())?
            .domain();
        if domain == super::channels::Domain::Degrees {
            Ok(CssColorHue::AngleCalculation(
                CssAngleCalculation::from_expression(self.numeric(index, CalculationRoot::Angle)?),
            ))
        } else {
            Ok(CssColorHue::NumberCalculation(
                CssNumberCalculation::from_expression(
                    self.numeric(index, CalculationRoot::Number)?,
                ),
            ))
        }
    }
    fn relative<'i>(
        &self,
        index: usize,
        environment: CssRelativeColorEnvironment,
        domain: CssRelativeColorResultDomain,
    ) -> Result<'i, CssRelativeColorExpression> {
        CssRelativeColorExpression::from_parser_components(
            self.values(index)?,
            self.numeric,
            environment,
            domain,
        )
        .map_err(|error| self.numeric_failure(index, error))
    }
    fn profile_expression<'i>(&self, index: usize) -> Result<'i, CssProfileColorExpression> {
        CssProfileColorExpression::from_parser_components(self.values(index)?, self.numeric)
            .map_err(|error| self.numeric_failure(index, error))
    }
    fn weight<'i>(&self, index: usize) -> Result<'i, CssColorMixWeight> {
        let raw = self.component(index)?;
        if matches!(
            raw.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(_))
        ) {
            let site = match self.component_record(index)?.component {
                RawComponent::Atom(token) | RawComponent::Math { opener: token, .. } => token.index,
                RawComponent::Authored { .. } => {
                    unreachable!("construction consumes source grammar only")
                }
            };
            CssColorMixPercentage::try_from_component(raw.clone())
                .map(CssColorMixWeight::literal)
                .map_err(|_| {
                    site_error(self.events, site, self.numeric, self.location, self.offset)
                })
        } else {
            CssColorMixWeight::try_calculation(CssPercentageCalculation::from_expression(
                self.numeric(index, CalculationRoot::Percentage)?,
            ))
            .map_err(|error| crate::error::invalid_component_value(self.location, error))
        }
    }
    fn profile<'i>(&self, token: usize) -> Result<'i, CssColorProfileName> {
        let CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) = self.raw(token)?.view()
        else {
            return Err(self.invalid());
        };
        CssColorProfileName::try_new(name).ok_or_else(|| self.invalid())
    }
    fn child<'i>(
        &self,
        index: Option<usize>,
        native: &mut [Option<CssColor>],
    ) -> Result<'i, CssColor> {
        index
            .and_then(|index| native.get_mut(index))
            .and_then(Option::take)
            .ok_or_else(|| self.invalid())
    }
    fn checked<'i, T>(
        &self,
        value: std::result::Result<T, CssColorConstructionError>,
        recipe: Recipe,
    ) -> Result<'i, T> {
        value.map_err(|error| self.construction_error(error, recipe))
    }
    fn construction_error<'i>(
        &self,
        error: CssColorConstructionError,
        recipe: Recipe,
    ) -> ParseError<'i, Error> {
        if matches!(
            error,
            CssColorConstructionError::CapacityOverflow | CssColorConstructionError::NestingLimit
        ) {
            let kind = if error == CssColorConstructionError::CapacityOverflow {
                CssComponentValueErrorKind::CapacityOverflow
            } else {
                CssComponentValueErrorKind::NestingLimit
            };
            let origin = self
                .raw(
                    recipe
                        .site
                        .source()
                        .expect("construction consumes only source-derived grammar")
                        .start,
                )
                .map_or(CssValueOrigin::Programmatic, |raw| raw.origin().clone());
            crate::error::invalid_component_value(
                self.location,
                CssComponentValueError::new(kind, origin),
            )
        } else {
            self.invalid()
        }
    }
    fn closed<'i>(&self, recipe: Recipe) -> Result<'i, bool> {
        Ok(
            match self
                .raw(
                    recipe
                        .site
                        .source()
                        .expect("construction consumes only source-derived grammar")
                        .start,
                )?
                .view()
            {
                CssComponentValueRef::Function(function) => {
                    !matches!(
                        function.closing_origin(),
                        CssValueOrigin::ImplicitClosure { .. }
                    ) && function.values().first_implicit_origin().is_none()
                }
                CssComponentValueRef::Block(block) => {
                    !matches!(
                        block.closing_origin(),
                        CssValueOrigin::ImplicitClosure { .. }
                    ) && block.values().first_implicit_origin().is_none()
                }
                _ => true,
            },
        )
    }
    fn recipe<'i>(&self, recipe: Recipe, native: &mut [Option<CssColor>]) -> Result<'i, CssColor> {
        match recipe.kind {
            RecipeKind::Leaf(leaf) => Ok(match leaf {
                Leaf::Current => CssColor::current_color(),
                Leaf::Transparent => CssColor::transparent(),
                Leaf::System(value) => CssColor::from_system(value),
                Leaf::Named(_) => {
                    let CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) = self
                        .raw(
                            recipe
                                .site
                                .source()
                                .expect("construction consumes only source-derived grammar")
                                .start,
                        )?
                        .view()
                    else {
                        return Err(self.invalid());
                    };
                    CssColor::from_named(
                        CssNamedColor::try_new(name).ok_or_else(|| self.invalid())?,
                    )
                }
                Leaf::Hex(_) => {
                    let CssComponentValueRef::Token(CssValueTokenRef::Hash { value, .. }) = self
                        .raw(
                            recipe
                                .site
                                .source()
                                .expect("construction consumes only source-derived grammar")
                                .start,
                        )?
                        .view()
                    else {
                        return Err(self.invalid());
                    };
                    CssColor::from_hex(CssHexColor::try_new(value).ok_or_else(|| self.invalid())?)
                }
            }),
            RecipeKind::Function(function) => self.function(recipe, function, native),
        }
    }
    fn function<'i>(
        &self,
        recipe: Recipe,
        function: FunctionRecipe,
        native: &mut [Option<CssColor>],
    ) -> Result<'i, CssColor> {
        let index = |ordinal| {
            function
                .component_index(ordinal)
                .map_err(|_| self.invalid())
        };
        if function.function == F::Alpha {
            let source = self.child(function.origin, native)?;
            let alpha = function
                .alpha
                .map(|at| {
                    self.relative(
                        at,
                        CssRelativeColorEnvironment::Alpha,
                        CssRelativeColorResultDomain::Alpha,
                    )
                })
                .transpose()?;
            return self
                .checked(CssAlphaColor::try_new(source, alpha), recipe)
                .map(CssColor::from_alpha);
        }
        if function.origin.is_some() {
            let source = self.child(function.origin, native)?;
            if let Some(ProfileSpace::Custom { token, .. }) = function.space {
                let mut channels = Vec::new();
                channels
                    .try_reserve(function.count)
                    .map_err(|_| self.capacity())?;
                for ordinal in 0..function.count {
                    channels.push(self.profile_expression(index(ordinal)?)?);
                }
                let alpha = function
                    .alpha
                    .map(|at| self.profile_expression(at))
                    .transpose()?;
                return self
                    .checked(
                        CssRelativeCustomColor::try_new(
                            source,
                            self.profile(token)?,
                            channels,
                            alpha,
                        ),
                        recipe,
                    )
                    .map(CssColor::from_relative_custom);
            }
            let kind = match function.function {
                F::Rgb => CssRelativeColorFunction::Rgb,
                F::Hsl => CssRelativeColorFunction::Hsl,
                F::Hwb => CssRelativeColorFunction::Hwb,
                F::Lab => CssRelativeColorFunction::Lab,
                F::Lch => CssRelativeColorFunction::Lch,
                F::Oklab => CssRelativeColorFunction::Oklab,
                F::Oklch => CssRelativeColorFunction::Oklch,
                F::Color => {
                    let Some(ProfileSpace::Predefined(space)) = function.space else {
                        return Err(self.invalid());
                    };
                    CssRelativeColorFunction::Color(space)
                }
                _ => return Err(self.invalid()),
            };
            let (environment, domains) = kind.signature();
            let channels = [
                self.relative(index(0)?, environment, domains[0])?,
                self.relative(index(1)?, environment, domains[1])?,
                self.relative(index(2)?, environment, domains[2])?,
            ];
            let alpha = function
                .alpha
                .map(|at| self.relative(at, environment, CssRelativeColorResultDomain::Alpha))
                .transpose()?;
            return self
                .checked(
                    CssRelativeColor::try_new(kind, source, channels, alpha),
                    recipe,
                )
                .map(CssColor::from_relative);
        }
        match function.function {
            F::Contrast => {
                let source = self.child(function.children[0], native)?;
                self.checked(CssContrastColor::try_new(source), recipe)
                    .map(CssColor::from_contrast_color)
            }
            F::LightDark => {
                let light = self.child(function.children[0], native)?;
                let dark = self.child(function.children[1], native)?;
                self.checked(CssLightDarkColor::try_new(light, dark), recipe)
                    .map(CssColor::from_light_dark)
            }
            F::Mix => self.mix(recipe, function, native),
            _ => {
                let alpha = function.alpha.map(|at| self.scalar(at)).transpose()?;
                if let Some(ProfileSpace::Custom { token, .. }) = function.space {
                    let mut channels = Vec::new();
                    channels
                        .try_reserve(function.count)
                        .map_err(|_| self.capacity())?;
                    for ordinal in 0..function.count {
                        channels.push(self.scalar(index(ordinal)?)?);
                    }
                    return self
                        .checked(
                            CssCustomColor::try_new(self.profile(token)?, channels, alpha),
                            recipe,
                        )
                        .map(CssColor::from_custom);
                }
                let first = || self.scalar(index(0)?);
                match function.function {
                    F::Rgb => self
                        .checked(
                            CssRgbColor::try_new(
                                function.syntax,
                                [first()?, self.scalar(index(1)?)?, self.scalar(index(2)?)?],
                                alpha,
                            ),
                            recipe,
                        )
                        .map(CssColor::from_rgb),
                    F::Hsl => self
                        .checked(
                            CssHslColor::try_new(
                                function.syntax,
                                self.hue(index(0)?)?,
                                self.scalar(index(1)?)?,
                                self.scalar(index(2)?)?,
                                alpha,
                            ),
                            recipe,
                        )
                        .map(CssColor::from_hsl),
                    F::Hwb => self
                        .checked(
                            CssHwbColor::try_new(
                                self.hue(index(0)?)?,
                                self.scalar(index(1)?)?,
                                self.scalar(index(2)?)?,
                                alpha,
                            ),
                            recipe,
                        )
                        .map(CssColor::from_hwb),
                    F::Lab | F::Oklab => self
                        .checked(
                            CssLabColor::try_new(
                                first()?,
                                self.scalar(index(1)?)?,
                                self.scalar(index(2)?)?,
                                alpha,
                            ),
                            recipe,
                        )
                        .map(|value| {
                            if function.function == F::Lab {
                                CssColor::from_lab(value)
                            } else {
                                CssColor::from_oklab(value)
                            }
                        }),
                    F::Lch | F::Oklch => self
                        .checked(
                            CssLchColor::try_new(
                                first()?,
                                self.scalar(index(1)?)?,
                                self.hue(index(2)?)?,
                                alpha,
                            ),
                            recipe,
                        )
                        .map(|value| {
                            if function.function == F::Lch {
                                CssColor::from_lch(value)
                            } else {
                                CssColor::from_oklch(value)
                            }
                        }),
                    F::Color => {
                        let Some(ProfileSpace::Predefined(space)) = function.space else {
                            return Err(self.invalid());
                        };
                        self.checked(
                            CssPredefinedColor::try_new(
                                space,
                                [first()?, self.scalar(index(1)?)?, self.scalar(index(2)?)?],
                                alpha,
                            ),
                            recipe,
                        )
                        .map(CssColor::from_predefined)
                    }
                    F::DeviceCmyk => self
                        .checked(
                            CssDeviceCmykColor::try_new(
                                function.syntax,
                                [
                                    first()?,
                                    self.scalar(index(1)?)?,
                                    self.scalar(index(2)?)?,
                                    self.scalar(index(3)?)?,
                                ],
                                alpha,
                            ),
                            recipe,
                        )
                        .map(CssColor::from_device_cmyk),
                    _ => Err(self.invalid()),
                }
            }
        }
    }
    fn mix<'i>(
        &self,
        recipe: Recipe,
        function: FunctionRecipe,
        native: &mut [Option<CssColor>],
    ) -> Result<'i, CssColor> {
        let interpolation = function
            .interpolation
            .map(|value| match value {
                Interpolation::Standard(method) => {
                    Ok(CssColorInterpolation::from_predefined(method))
                }
                Interpolation::Custom { token, .. } => {
                    self.profile(token).map(CssColorInterpolation::custom)
                }
            })
            .transpose()?;
        let mut components = Vec::new();
        components
            .try_reserve(function.mix_count)
            .map_err(|_| self.capacity())?;
        let mut next = function.mix;
        for ordinal in (0..function.mix_count).rev() {
            let at = next.ok_or_else(|| self.invalid())?;
            let GrammarRecord::Mix(record) = self.record(at)? else {
                return Err(self.invalid());
            };
            if record.ordinal != ordinal || record.previous.is_some_and(|previous| previous >= at) {
                return Err(self.invalid());
            }
            let weight = record.weight.map(|at| self.weight(at)).transpose()?;
            components.push(CssColorMixComponent::new(
                self.child(Some(record.color), native)?,
                weight,
            ));
            next = record.previous;
        }
        if next.is_some() {
            return Err(self.invalid());
        }
        components.reverse();
        CssColorMix::try_new(interpolation, components)
            .map(CssColor::from_color_mix)
            .map_err(|error| {
                let error = match error {
                    CssColorMixConstructionError::EmptyComponents => {
                        CssColorConstructionError::EmptyComponents
                    }
                    CssColorMixConstructionError::NestingLimit => {
                        CssColorConstructionError::NestingLimit
                    }
                    CssColorMixConstructionError::CapacityOverflow => {
                        CssColorConstructionError::CapacityOverflow
                    }
                };
                self.construction_error(error, recipe)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parsed(text: &str) -> std::result::Result<CssColor, String> {
        let source = CssSourceSnapshot::new(text);
        let mut input = cssparser::ParserInput::new(text);
        let mut parser = Parser::new(&mut input);
        parse_color(&mut parser, &NumericInputContext::Parsed(&source))
            .map_err(|error| format!("{error:?}"))
    }
    fn components(text: &str) -> std::result::Result<CssColor, String> {
        let values = parse_component_values(text).unwrap();
        let serialized = values.serialize().unwrap();
        let mut input = cssparser::ParserInput::new(serialized.as_css());
        let mut parser = Parser::new(&mut input);
        parse_color(
            &mut parser,
            &NumericInputContext::Components(&values, &serialized),
        )
        .map_err(|error| format!("{error:?}"))
    }
    #[test]
    fn literal_rgb_uses_real_parsed_and_component_entries() {
        for value in [parsed("rgb(1 2 3)"), components("rgb(1 2 3)")] {
            let value = value.unwrap();
            let rgb = value.rgb_value().unwrap();
            assert_eq!(rgb.syntax(), CssColorSyntax::Modern);
            for (channel, expected) in rgb.channels().iter().zip(["1", "2", "3"]) {
                let CssColorComponent::Number(value) = channel else {
                    panic!("number");
                };
                assert_eq!(value.numeric().representation(), expected);
            }
        }
    }
    fn branch(value: &CssColor) -> usize {
        if value.is_current_color() {
            0
        } else if value.is_transparent() {
            1
        } else if value.named().is_some() {
            2
        } else if value.hex_value().is_some() {
            3
        } else if value.system().is_some() {
            4
        } else if value.rgb_value().is_some() {
            5
        } else if value.hsl_value().is_some() {
            6
        } else if value.hwb_value().is_some() {
            7
        } else if value.lab_value().is_some() {
            8
        } else if value.lch_value().is_some() {
            9
        } else if value.oklab_value().is_some() {
            10
        } else if value.oklch_value().is_some() {
            11
        } else if value.predefined_value().is_some() {
            12
        } else if value.custom_value().is_some() {
            13
        } else if value.relative_value().is_some() {
            14
        } else if value.relative_custom_value().is_some() {
            15
        } else if value.alpha_value().is_some() {
            16
        } else if value.color_mix_value().is_some() {
            17
        } else if value.light_dark_value().is_some() {
            18
        } else if value.contrast_color_value().is_some() {
            19
        } else if value.device_cmyk_value().is_some() {
            20
        } else {
            panic!("native branch")
        }
    }
    #[test]
    fn all_twenty_one_native_branches_are_symbolic_in_both_source_modes() {
        for (text, expected) in [
            ("currentcolor", 0),
            ("transparent", 1),
            ("ReBeccAPurPle", 2),
            ("#F08c", 3),
            ("CanvasText", 4),
            ("rgb(1 2 3)", 5),
            ("hsl(10 20% 30%)", 6),
            ("hwb(10 20% 30%)", 7),
            ("lab(50 1 2)", 8),
            ("lch(50 1 2)", 9),
            ("oklab(.5 .1 .2)", 10),
            ("oklch(.5 .1 2)", 11),
            ("color(display-p3 1 0 .5)", 12),
            ("color(--Exact 1 2 3 4)", 13),
            ("rgb(from red r g b)", 14),
            ("color(from red --Exact foo bar baz quux)", 15),
            ("alpha(from red / alpha)", 16),
            ("color-mix(in srgb, red, blue)", 17),
            ("light-dark(red, blue)", 18),
            ("contrast-color(red)", 19),
            ("device-cmyk(0 1 1 0)", 20),
        ] {
            for value in [parsed(text), components(text)] {
                assert_eq!(
                    branch(&value.unwrap_or_else(|error| panic!("{text}: {error}"))),
                    expected,
                    "{text}"
                );
            }
        }
    }
    #[test]
    fn aliases_escapes_comments_and_exact_numeric_origins_survive() {
        for text in ["rgba(1, 2, 3, .5)", "\\72 GB(1/**/ 2 3 / .5)"] {
            let value = parsed(text).unwrap();
            let rgb = value.rgb_value().unwrap();
            assert!(rgb.alpha().is_some());
            let CssColorComponent::Number(first) = &rgb.channels()[0] else {
                panic!("number");
            };
            let CssValueOrigin::Parsed(origin) = first.origin() else {
                panic!("parsed origin");
            };
            assert_eq!(origin.source().as_str(), text);
            assert_eq!(
                origin.span().start().byte_offset().value(),
                text.find('1').unwrap()
            );
        }
        let text = "rgb(+0001.2500e-02 -0 1e999 / none)";
        for value in [parsed(text), components(text)] {
            let value = value.unwrap();
            let rgb = value.rgb_value().unwrap();
            for (channel, expected) in rgb.channels().iter().zip(["+0001.2500e-02", "-0", "1e999"])
            {
                let CssColorComponent::Number(value) = channel else {
                    panic!("number");
                };
                assert_eq!(value.numeric().representation(), expected);
            }
            assert!(matches!(rgb.alpha(), Some(CssColorComponent::None)));
        }
        let color = parsed("hsl(1.25TuRn 20% 30%)").unwrap();
        let CssColorHue::Angle(angle) = color.hsl_value().unwrap().hue() else {
            panic!("angle");
        };
        assert_eq!(angle.numeric().representation(), "1.25");
        assert_eq!(angle.unit(), CssAngleUnit::Turns);
        assert_eq!(
            parsed("hsla(10, 20%, 30%, .5)")
                .unwrap()
                .hsl_value()
                .unwrap()
                .syntax(),
            CssColorSyntax::Legacy
        );
        assert_eq!(parsed("aqua").unwrap().named().unwrap().name(), "aqua");
        assert_eq!(parsed("cyan").unwrap().named().unwrap().name(), "cyan");
    }
    #[test]
    fn shared_numeric_admission_retains_calculations_and_relative_environments() {
        for parse in [parsed, components] {
            let value = parse("rgb(calc(1 + 2) calc(20%) none / calc(.5))").unwrap();
            let rgb = value.rgb_value().unwrap();
            assert!(matches!(
                rgb.channels()[0],
                CssColorComponent::NumberCalculation(_)
            ));
            assert!(matches!(
                rgb.channels()[1],
                CssColorComponent::PercentageCalculation(_)
            ));
            assert!(matches!(rgb.channels()[2], CssColorComponent::None));
            let value = parse("lch(from red l c calc(h + 1) / alpha)").unwrap();
            let relative = value.relative_value().unwrap();
            assert_eq!(*relative.function(), CssRelativeColorFunction::Lch);
            assert_eq!(
                relative.channels()[2].result_domain(),
                CssRelativeColorResultDomain::Hue
            );
            assert!(matches!(
                relative.channels()[2].value(),
                CssRelativeColorExpressionValue::Calculation(_)
            ));
            assert!(relative.alpha().is_some());
            for text in [
                "hsl(from red h s l)",
                "hwb(from red h w b)",
                "lab(from red l a b)",
                "oklab(from red l a b)",
                "oklch(from red l c h)",
                "color(from red display-p3 r g b)",
            ] {
                assert!(parse(text).unwrap().relative_value().is_some(), "{text}");
            }
        }
    }
    #[test]
    fn custom_names_weights_order_and_nested_branches_are_not_resolved() {
        for parse in [parsed, components] {
            let value =
                parse("color-mix(in --Exact, 20% red, blue calc(30%), currentcolor)").unwrap();
            let mix = value.color_mix_value().unwrap();
            assert_eq!(
                mix.interpolation()
                    .unwrap()
                    .custom_profile()
                    .unwrap()
                    .as_str(),
                "--Exact"
            );
            assert_eq!(mix.components().len(), 3);
            assert_eq!(mix.components()[0].color().named().unwrap().name(), "red");
            assert!(
                mix.components()[0]
                    .weight()
                    .unwrap()
                    .literal_value()
                    .is_some()
            );
            assert!(
                mix.components()[1]
                    .weight()
                    .unwrap()
                    .calculation()
                    .is_some()
            );
            assert!(mix.components()[2].color().is_current_color());
            assert!(mix.components()[2].weight().is_none());
            let value =
                parse("light-dark(contrast-color(currentcolor), device-cmyk(1%, none, 0, 1))");
            assert!(value.is_err(), "legacy CMYK percentages/none forbidden");
            let value =
                parse("light-dark(contrast-color(currentcolor), device-cmyk(1% none 0 1 / .5))")
                    .unwrap();
            assert!(
                value
                    .light_dark_value()
                    .unwrap()
                    .light()
                    .contrast_color_value()
                    .unwrap()
                    .color()
                    .is_current_color()
            );
            assert!(
                value
                    .light_dark_value()
                    .unwrap()
                    .dark()
                    .device_cmyk_value()
                    .unwrap()
                    .alpha()
                    .is_some()
            );
            let value = parse("color(from red --Exact foo calc(bar + 1) none / baz)").unwrap();
            let custom = value.relative_custom_value().unwrap();
            assert_eq!(custom.profile().as_str(), "--Exact");
            assert_eq!(custom.channels().len(), 3);
            assert!(matches!(
                custom.channels()[1].view(),
                CssProfileColorExpressionRef::Calculation(_)
            ));
        }
    }
    #[test]
    fn omission_and_implicit_closure_remain_distinct_from_explicit_values() {
        assert!(
            parsed("alpha(from red)")
                .unwrap()
                .alpha_value()
                .unwrap()
                .alpha()
                .is_none()
        );
        assert!(
            parsed("alpha(from red / none)")
                .unwrap()
                .alpha_value()
                .unwrap()
                .alpha()
                .is_some()
        );
        assert!(
            parsed("color-mix(red, blue)")
                .unwrap()
                .color_mix_value()
                .unwrap()
                .interpolation()
                .is_none()
        );
        for text in [
            "rgb(1 2 3",
            "rgb(calc(1 + 2) 0 0",
            "light-dark(red, rgb(1 2 3",
        ] {
            let value = parsed(text).unwrap();
            assert!(!value.is_closed(), "{text}");
        }
        assert!(components("rgb(0 0 calc(1 + 2").is_err());
        assert!(!parsed("rgb(0 0 calc(1 + 2").unwrap().is_closed());
        assert!(parsed("rgb(calc(1 + 2) 0 0)").unwrap().is_closed());
    }
    #[test]
    fn grammar_failures_and_prefix_consumption_use_actual_boundaries() {
        for text in [
            "rgb(1, 2%, 3)",
            "rgb(none, 2, 3)",
            "rgb(1 2 3, .5)",
            "hsl(1, 2, 3)",
            "rgb(1 2)",
            "color(--Exact)",
            "rgb(1 x 3)",
            "color-mix(in srgb, 101% red, blue)",
            "light-dark(red, blue, green)",
        ] {
            assert!(parsed(text).is_err(), "{text}");
            assert!(components(text).is_err(), "{text}");
        }
        let text = "rgb(1 2 3) blue";
        let source = CssSourceSnapshot::new(text);
        let mut input = cssparser::ParserInput::new(text);
        let mut parser = Parser::new(&mut input);
        let color = parse_color(&mut parser, &NumericInputContext::Parsed(&source)).unwrap();
        assert!(color.rgb_value().is_some());
        assert_eq!(parser.expect_ident().unwrap().as_ref(), "blue");
        let text = "light-dark(red, rgb(1 x 3))";
        let values = parse_component_values(text).unwrap();
        let serialized = values.serialize().unwrap();
        let mut input = cssparser::ParserInput::new(serialized.as_css());
        let mut parser = Parser::new(&mut input);
        let error = parse_color(
            &mut parser,
            &NumericInputContext::Components(&values, &serialized),
        )
        .unwrap_err();
        assert_eq!(
            error.location.column as usize,
            serialized.as_css().find('x').unwrap() + 1
        );
        let text = "color-mix(in srgb, red 101%, blue)";
        let source = CssSourceSnapshot::new(text);
        let mut input = cssparser::ParserInput::new(text);
        let mut parser = Parser::new(&mut input);
        let error = parse_color(&mut parser, &NumericInputContext::Parsed(&source)).unwrap_err();
        assert_eq!(
            error.location.column as usize,
            text.find("101").unwrap() + 1
        );
        let values = parse_component_values(text).unwrap();
        let serialized = values.serialize().unwrap();
        let mut input = cssparser::ParserInput::new(serialized.as_css());
        let mut parser = Parser::new(&mut input);
        let error = parse_color(
            &mut parser,
            &NumericInputContext::Components(&values, &serialized),
        )
        .unwrap_err();
        assert_eq!(
            error.location.column as usize,
            serialized.as_css().find("101").unwrap() + 1
        );
    }
    #[test]
    fn genuine_programmatic_components_keep_exact_tokens_without_reparse() {
        let supplied = CssComponentValue::try_number("+000000001.250e-01").unwrap();
        let arguments = CssComponentValues::try_new(vec![
            supplied.clone(),
            CssComponentValue::try_number("2").unwrap(),
            CssComponentValue::try_number("3").unwrap(),
        ])
        .unwrap();
        let root = CssComponentValue::try_function("RGB", arguments).unwrap();
        let values = CssComponentValues::try_new(vec![root]).unwrap();
        let serialized = values.serialize().unwrap();
        let mut input = cssparser::ParserInput::new(serialized.as_css());
        let mut parser = Parser::new(&mut input);
        let value = parse_color(
            &mut parser,
            &NumericInputContext::Components(&values, &serialized),
        )
        .unwrap();
        let CssColorComponent::Number(number) = &value.rgb_value().unwrap().channels()[0] else {
            panic!("number");
        };
        assert_eq!(number.component(), &supplied);
        assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(number.numeric().representation(), "+000000001.250e-01");
        assert!(value.is_closed());
    }
    #[test]
    fn source_positions_keep_utf16_columns_and_constructor_limits_stay_typed() {
        let text = "/*😀*/light-dark(red, rgb(1 x 3))";
        let source = CssSourceSnapshot::new(text);
        let mut input = cssparser::ParserInput::new(text);
        let mut parser = Parser::new(&mut input);
        let error = parse_color(&mut parser, &NumericInputContext::Parsed(&source)).unwrap_err();
        assert_eq!(
            error.location.column as usize,
            text[..text.find('x').unwrap()].encode_utf16().count() + 1
        );
        let too_deep = format!("{}red{}", "contrast-color(".repeat(257), ")".repeat(257));
        let source = CssSourceSnapshot::new(&too_deep);
        let mut input = cssparser::ParserInput::new(&too_deep);
        let mut parser = Parser::new(&mut input);
        let error = parse_color(&mut parser, &NumericInputContext::Parsed(&source)).unwrap_err();
        let cssparser::ParseErrorKind::Custom(error) = error.kind else {
            panic!("typed limit");
        };
        assert_eq!(error.code(), CssErrorCode::InvalidComponentValue);
        let crate::error::ErrorKind::InvalidComponentValue(detail) = error.kind() else {
            panic!("component limit detail");
        };
        assert_eq!(detail.kind(), CssComponentValueErrorKind::NestingLimit);
    }

    #[test]
    fn closing_and_eof_failures_use_actual_end_sites_in_both_source_modes() {
        let mut observed = Vec::new();
        for (text, serialized_text, expected) in [
            ("rgb(1 2)", "rgb(1 2)", 8),
            ("rgb(1 2", "rgb(1 2)", 8),
            (
                "/*😀*/light-dark(red, rgb(1 2))",
                "/*😀*/light-dark(red, rgb(1 2))",
                30,
            ),
            (
                "/*😀*/light-dark(red, rgb(1 2",
                "/*😀*/light-dark(red, rgb(1 2))",
                30,
            ),
        ] {
            let source = CssSourceSnapshot::new(text);
            let mut input = cssparser::ParserInput::new(text);
            let mut parser = Parser::new(&mut input);
            let error =
                parse_color(&mut parser, &NumericInputContext::Parsed(&source)).unwrap_err();
            observed.push((text, "Parsed", error.location.column, expected));
            let values = parse_component_values(text).unwrap();
            let serialized = values.serialize().unwrap();
            assert_eq!(serialized.as_css(), serialized_text);
            let mut input = cssparser::ParserInput::new(serialized.as_css());
            let mut parser = Parser::new(&mut input);
            let error = parse_color(
                &mut parser,
                &NumericInputContext::Components(&values, &serialized),
            )
            .unwrap_err();
            observed.push((text, "Components", error.location.column, expected));
        }
        for (text, mode, actual, expected) in &observed {
            assert_eq!(
                actual, expected,
                "{text:?} {mode}; all observed: {observed:?}"
            );
        }
    }
}
