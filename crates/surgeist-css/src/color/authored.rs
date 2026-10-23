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
    ColorProtocolError, CssColorBank, CssColorOperationOwner, CssColorSlot, Data, LexicalTable,
};
use super::syntax_pass::{SyntaxAction, SyntaxPass};
use crate::error::ErrorKind;
use crate::numeric::{CalculationRoot, NumericInputContext};
use crate::tokenization::tokenizer::{NameKind, NumberKind, Span, Token, TokenKind};
use crate::*;

type Result<T> = std::result::Result<T, ColorInputFailure>;

/// A failure in the original component graph; the root itself has path [].
#[derive(Debug)]
pub(crate) struct ColorInputFailure {
    pub(crate) kind: ErrorKind,
    pub(crate) path: Box<[usize]>,
    pub(crate) closing: bool,
}
fn invalid_color(component: Option<&str>) -> ColorInputFailure {
    ColorInputFailure {
        kind: crate::error::color_syntax_kind(component),
        path: Box::new([]),
        closing: false,
    }
}
fn component_failure(detail: CssComponentValueError) -> ColorInputFailure {
    ColorInputFailure {
        kind: ErrorKind::InvalidComponentValue(Box::new(detail)),
        path: Box::new([]),
        closing: false,
    }
}
fn event_failure(
    events: &Events<'_, '_>,
    site: usize,
    inside: &[usize],
    kind: ErrorKind,
) -> ColorInputFailure {
    let relative = events
        .paths
        .get(site)
        .and_then(Option::as_ref)
        .map_or(&[][..], Vec::as_slice);
    let capacity = || {
        component_failure(CssComponentValueError::new(
            CssComponentValueErrorKind::CapacityOverflow,
            events.root.origin().clone(),
        ))
    };
    let Some(length) = relative.len().checked_add(inside.len()) else {
        return capacity();
    };
    let mut path = Vec::new();
    if path.try_reserve_exact(length).is_err() {
        return capacity();
    }
    path.extend_from_slice(relative);
    path.extend_from_slice(inside);
    ColorInputFailure {
        kind,
        path: path.into_boxed_slice(),
        closing: events.closing.get(site).copied().unwrap_or(false),
    }
}

struct Events<'a, 'owner> {
    root: &'a CssComponentValue,
    slots: Vec<CssColorSlot<'owner>>,
    originals: Vec<Option<&'a CssComponentValue>>,
    paths: Vec<Option<Vec<usize>>>,
    closing: Vec<bool>,
    eof: Option<EndSite>,
    owner: &'owner CssColorOperationOwner,
    phase: usize,
}
struct EndSite {
    path: Vec<usize>,
}
impl<'a, 'owner> Events<'a, 'owner> {
    fn append(
        &mut self,
        data: Data<'owner>,
        original: Option<&'a CssComponentValue>,
    ) -> std::result::Result<usize, ColorProtocolError> {
        let index = self.slots.len();
        index.checked_add(1).ok_or(ColorProtocolError::Overflow)?;
        self.slots
            .try_reserve(1)
            .map_err(|_| ColorProtocolError::Overflow)?;
        self.originals
            .try_reserve(1)
            .map_err(|_| ColorProtocolError::Overflow)?;
        self.paths
            .try_reserve(1)
            .map_err(|_| ColorProtocolError::Overflow)?;
        self.closing
            .try_reserve(1)
            .map_err(|_| ColorProtocolError::Overflow)?;
        self.slots
            .push(CssColorSlot::new(self.owner, self.phase, index, data));
        self.originals.push(original);
        self.paths.push(None);
        self.closing.push(false);
        Ok(index)
    }
    fn name(&mut self, value: &str) -> std::result::Result<Span, ColorProtocolError> {
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
    ) -> std::result::Result<TokenKind, ColorProtocolError> {
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
                .map_err(|_| ColorProtocolError::Overflow)?
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
                    offset = offset.checked_add(1).ok_or(ColorProtocolError::Overflow)?;
                }
                Step::Transition => {}
                Step::Done(layout) => break layout,
            }
        };
        if offset != spelling.len() {
            return Err(ColorProtocolError::Input);
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
    ) -> std::result::Result<(), ColorProtocolError> {
        let start = self.slots.len();
        let end = start.checked_add(1).ok_or(ColorProtocolError::Overflow)?;
        let mut retained = Vec::new();
        retained
            .try_reserve(path.len())
            .map_err(|_| ColorProtocolError::Overflow)?;
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
    ) -> std::result::Result<(), ColorProtocolError> {
        enum Visit<'a> {
            Item(&'a CssComponentValue),
            Items(&'a [CssComponentValue], usize),
            Close(&'a CssComponentValue, char, bool),
        }
        let mut path = Vec::new();
        let mut pending = Vec::new();
        pending
            .try_reserve(1)
            .map_err(|_| ColorProtocolError::Overflow)?;
        pending.push(Visit::Item(root));
        while let Some(visit) = pending.pop() {
            match visit {
                Visit::Items(items, index) => {
                    if let Some((first, rest)) = items.split_first() {
                        pending
                            .try_reserve(2)
                            .map_err(|_| ColorProtocolError::Overflow)?;
                        pending.push(Visit::Items(
                            rest,
                            index.checked_add(1).ok_or(ColorProtocolError::Overflow)?,
                        ));
                        path.try_reserve(1)
                            .map_err(|_| ColorProtocolError::Overflow)?;
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
                            .try_reserve_exact(path.len())
                            .map_err(|_| ColorProtocolError::Overflow)?;
                        retained.extend_from_slice(&path);
                        self.eof = Some(EndSite { path: retained });
                    }
                    path.pop();
                }
                Visit::Item(item) => {
                    let kind = match item.view() {
                        CssComponentValueRef::Function(function) => {
                            let units = self.name(function.name())?;
                            pending
                                .try_reserve(2)
                                .map_err(|_| ColorProtocolError::Overflow)?;
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
                                .map_err(|_| ColorProtocolError::Overflow)?;
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
                            CssValueTokenRef::Url(_) => return Err(ColorProtocolError::Input),
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

pub(crate) fn construct_component(
    root: &CssComponentValue,
    numeric: &NumericInputContext<'_>,
) -> Result<CssColor> {
    let capacity = || {
        component_failure(CssComponentValueError::new(
            CssComponentValueErrorKind::CapacityOverflow,
            root.origin().clone(),
        ))
    };
    let protocol = |error| {
        if error == ColorProtocolError::Overflow {
            capacity()
        } else {
            invalid_color(None)
        }
    };
    let mut owner = CssColorOperationOwner::new();
    let phase = owner.begin().map_err(protocol)?;
    let mut events = Events {
        root,
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
                    && let Some(end) = events.eof.take()
                {
                    return Err(ColorInputFailure {
                        kind: crate::error::color_syntax_kind(component),
                        path: end.path.into_boxed_slice(),
                        closing: true,
                    });
                }
                return Err(event_failure(
                    &events,
                    source.start,
                    &[],
                    crate::error::color_syntax_kind(component),
                ));
            }
            GrammarAction::Stopped => return Err(invalid_color(None)),
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
                        CssColorBank::Evaluation => return Err(invalid_color(None)),
                    };
                    pass.component_read(bank, index, &slots[index])
                }
                ComponentAction::Append { bank, index }
                | ComponentAction::Replace { bank, index } => {
                    let record = pass.component_record(bank, index).map_err(protocol)?;
                    let slots = match bank {
                        CssColorBank::Algorithm => &mut events.slots,
                        CssColorBank::Arithmetic => &mut arithmetic,
                        CssColorBank::Evaluation => return Err(invalid_color(None)),
                    };
                    if index == slots.len() {
                        slots.try_reserve(1).map_err(|_| capacity())?;
                        slots.push(record);
                    } else {
                        *slots.get_mut(index).ok_or_else(|| invalid_color(None))? = record;
                    }
                    pass.component_written(bank, index, &slots[index])
                }
                ComponentAction::Compute | ComponentAction::Done | ComponentAction::Failed(_) => {
                    pass.compute()
                }
                ComponentAction::Stopped => return Err(invalid_color(None)),
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
                return Err(event_failure(
                    &events,
                    site,
                    &[],
                    crate::error::color_syntax_kind(component),
                ));
            }
            SyntaxAction::Stopped => return Err(invalid_color(None)),
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
        root,
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
        .ok_or_else(|| invalid_color(None))
}

struct Builder<'a, 'owner, 'context> {
    events: &'a Events<'a, 'owner>,
    numeric: &'a NumericInputContext<'context>,
    root: &'a CssComponentValue,
}

impl Builder<'_, '_, '_> {
    fn invalid(&self) -> ColorInputFailure {
        invalid_color(None)
    }
    fn capacity(&self) -> ColorInputFailure {
        component_failure(CssComponentValueError::new(
            CssComponentValueErrorKind::CapacityOverflow,
            self.root.origin().clone(),
        ))
    }
    fn raw(&self, site: usize) -> Result<&CssComponentValue> {
        self.events
            .originals
            .get(site)
            .and_then(|v| *v)
            .ok_or_else(|| self.invalid())
    }
    fn record(&self, index: usize) -> Result<GrammarRecord> {
        match self.events.slots.get(index).map(|slot| slot.data) {
            Some(Data::Grammar(record)) => Ok(record),
            _ => Err(self.invalid()),
        }
    }
    fn component_record(&self, index: usize) -> Result<ComponentRecord> {
        match self.record(index)? {
            GrammarRecord::Component(record) if record.checked.is_some() => Ok(record),
            _ => Err(self.invalid()),
        }
    }
    fn component(&self, index: usize) -> Result<&CssComponentValue> {
        let record = self.component_record(index)?;
        let site = match record.component {
            RawComponent::Atom(token) | RawComponent::Math { opener: token, .. } => token.index,
            RawComponent::Authored { .. } => {
                unreachable!("construction consumes source grammar only")
            }
        };
        self.raw(site)
    }
    fn numeric_failure(
        &self,
        index: usize,
        error: CssNumericConstructionError,
    ) -> ColorInputFailure {
        let site = self
            .component_record(index)
            .ok()
            .map(|record| match record.component {
                RawComponent::Atom(token) | RawComponent::Math { opener: token, .. } => token.index,
                RawComponent::Authored { .. } => {
                    unreachable!("construction consumes source grammar only")
                }
            });
        let inside = error
            .path()
            .and_then(|path| path.strip_prefix(&[0]))
            .unwrap_or(&[]);
        let kind = error.component_error().map_or_else(
            || crate::error::color_syntax_kind(None),
            |component| ErrorKind::InvalidComponentValue(Box::new(component.clone())),
        );
        match site {
            Some(site) => event_failure(self.events, site, inside, kind),
            None => ColorInputFailure {
                kind,
                path: Box::new([]),
                closing: false,
            },
        }
    }

    fn numeric(&self, index: usize, root: CalculationRoot) -> Result<CssCalculationExpression> {
        self.numeric
            .admit(self.component(index)?, root)
            .map_err(|error| self.numeric_failure(index, error))
    }
    fn scalar(&self, index: usize) -> Result<CssColorComponent> {
        let raw = self.component(index)?;
        if matches!(raw.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) if name.eq_ignore_ascii_case("none"))
        {
            return Ok(CssColorComponent::None);
        }
        if matches!(raw.view(), CssComponentValueRef::Token(_)) {
            return crate::color_scalar::component(raw.clone()).map_err(component_failure);
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
    fn hue(&self, index: usize) -> Result<CssColorHue> {
        let raw = self.component(index)?;
        if matches!(raw.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) if name.eq_ignore_ascii_case("none"))
        {
            return Ok(CssColorHue::None);
        }
        if matches!(raw.view(), CssComponentValueRef::Token(_)) {
            return crate::color_scalar::hue(raw.clone()).map_err(component_failure);
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
    fn relative(
        &self,
        index: usize,
        environment: CssRelativeColorEnvironment,
        domain: CssRelativeColorResultDomain,
    ) -> Result<CssRelativeColorExpression> {
        CssRelativeColorExpression::from_parser_components(
            self.component(index)?.into(),
            self.numeric,
            environment,
            domain,
        )
        .map_err(|error| self.numeric_failure(index, error))
    }
    fn profile_expression(&self, index: usize) -> Result<CssProfileColorExpression> {
        CssProfileColorExpression::from_parser_components(
            self.component(index)?.into(),
            self.numeric,
        )
        .map_err(|error| self.numeric_failure(index, error))
    }
    fn weight(&self, index: usize) -> Result<CssColorMixWeight> {
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
                    event_failure(
                        self.events,
                        site,
                        &[],
                        crate::error::color_syntax_kind(None),
                    )
                })
        } else {
            CssColorMixWeight::try_calculation(CssPercentageCalculation::from_expression(
                self.numeric(index, CalculationRoot::Percentage)?,
            ))
            .map_err(component_failure)
        }
    }
    fn profile(&self, token: usize) -> Result<CssColorProfileName> {
        let CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) = self.raw(token)?.view()
        else {
            return Err(self.invalid());
        };
        CssColorProfileName::try_new(name).ok_or_else(|| self.invalid())
    }
    fn child(&self, index: Option<usize>, native: &mut [Option<CssColor>]) -> Result<CssColor> {
        index
            .and_then(|index| native.get_mut(index))
            .and_then(Option::take)
            .ok_or_else(|| self.invalid())
    }
    fn checked<T>(
        &self,
        value: std::result::Result<T, CssColorConstructionError>,
        recipe: Recipe,
    ) -> Result<T> {
        value.map_err(|error| self.construction_error(error, recipe))
    }
    fn construction_error(
        &self,
        error: CssColorConstructionError,
        recipe: Recipe,
    ) -> ColorInputFailure {
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
            component_failure(CssComponentValueError::new(kind, origin))
        } else {
            self.invalid()
        }
    }
    fn closed(&self, recipe: Recipe) -> Result<bool> {
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
    fn recipe(&self, recipe: Recipe, native: &mut [Option<CssColor>]) -> Result<CssColor> {
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
    fn function(
        &self,
        recipe: Recipe,
        function: FunctionRecipe,
        native: &mut [Option<CssColor>],
    ) -> Result<CssColor> {
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
    fn mix(
        &self,
        recipe: Recipe,
        function: FunctionRecipe,
        native: &mut [Option<CssColor>],
    ) -> Result<CssColor> {
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

/// Closure state is crate-private, so these tests stay here; they reach the
/// production `color` property parser through its public text and component
/// fronts. Other source-mode behavior is tested in
/// `tests/css/color/color_authored_source_modes.rs`.
#[cfg(test)]
mod tests {
    use crate::*;

    fn color_of(declaration: &CssDeclaration) -> CssColor {
        let Some(CssKnownPropertyValueRef::Color(value)) =
            declaration.known().and_then(|known| known.property_value())
        else {
            panic!("typed color: {declaration:?}");
        };
        value.value().clone()
    }

    fn checked(
        values: CssComponentValues,
    ) -> std::result::Result<CssDeclaration, CssPropertyValueParseError> {
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            values,
            CssImportance::Normal,
        )
    }

    fn parsed(text: &str) -> std::result::Result<CssColor, String> {
        let report = parse_property_value_text(
            text,
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            CssImportance::Normal,
        );
        // CSS Syntax 3 (CRD 2026-10-01) §5.5.10 closes an unclosed function at
        // EOF as a parse error that retains the value; any other diagnostic
        // rejects it.
        let (declaration, diagnostics) = report.into_parts();
        match (declaration, diagnostics.as_slice()) {
            (Some(declaration), diagnostics)
                if diagnostics.iter().all(|diagnostic| {
                    diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
                }) =>
            {
                Ok(color_of(&declaration))
            }
            (_, diagnostics) => Err(format!("{diagnostics:?}")),
        }
    }

    fn components(text: &str) -> std::result::Result<CssColor, String> {
        checked(parse_component_values(text).unwrap())
            .map(|declaration| color_of(&declaration))
            .map_err(|error| format!("{error:?}"))
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
        let value = color_of(&checked(values).unwrap());
        let CssColorComponent::Number(number) = &value.rgb_value().unwrap().channels()[0] else {
            panic!("number");
        };
        assert_eq!(number.component(), &supplied);
        assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(number.numeric().representation(), "+000000001.250e-01");
        assert!(value.is_closed());
    }
}
