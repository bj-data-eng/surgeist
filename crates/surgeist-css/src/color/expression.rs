//! Qualified adapter of the canonical numeric continuation and fixed kernels.
//!
//! Lexical/decimal records and arithmetic records occupy separate caller-owned
//! flat banks under the same anchor and phase. An action always names its bank;
//! no scratch allocation, lookup scan or controller is hidden in this adapter.

use super::keywords::Word;
use super::literal::{LiteralAction, LiteralConversion};
use super::tokens::{ClassifiedToken, TokenAction, TokenReader};
use crate::color::controlled::CssColorSlot;
use crate::color::controlled::protocol::{ColorProtocolError, Data, LexicalTable};
use crate::numeric::CssNumericDimension as Dimension;
use crate::numeric::CssNumericType as Type;
use crate::numeric::bounded_expression as engine;
use surgeist_css_parser::lexer::protocol::{NameKind, RawTokenKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InputContext {
    function: Option<crate::numeric::CssMathFunction>,
    leading: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExpressionRecord {
    record: engine::Record,
    suspended: Option<InputContext>,
}
impl ExpressionRecord {
    pub(crate) const fn authored(record: engine::Record) -> Self {
        Self {
            record,
            suspended: None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Number {
    pub(crate) value: f64,
    pub(crate) ty: Type,
}
impl Number {
    pub(crate) fn same(self, other: Self) -> bool {
        self.value.to_bits() == other.value.to_bits() && self.ty == other.ty
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Variable {
    Channel(crate::color::CssRelativeColorChannel),
    Profile {
        name: crate::color::controlled::protocol::RecordRange,
    },
}
/// A reference retains full source-token identity or its genuine authored site.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum OperandReference {
    Source(ClassifiedToken),
    Authored { index: usize, site: usize },
}
impl OperandReference {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Source(token) => token.index,
            Self::Authored { index, .. } => index,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum NumericLeaf {
    Literal(Number),
    Contextual {
        token: OperandReference,
        unit: crate::numeric::CssNumericUnit,
        number: Number,
    },
    Variable {
        token: OperandReference,
        binding: Variable,
    },
}
impl NumericLeaf {
    fn ty(self) -> Type {
        match self {
            Self::Literal(value) | Self::Contextual { number: value, .. } => value.ty,
            Self::Variable { .. } => Type::NUMBER,
        }
    }
    pub(crate) fn same(self, other: Self) -> bool {
        match (self, other) {
            (Self::Literal(a), Self::Literal(b)) => a.same(b),
            (
                Self::Contextual {
                    token: a,
                    unit: au,
                    number: av,
                },
                Self::Contextual {
                    token: b,
                    unit: bu,
                    number: bv,
                },
            ) => a == b && au == bu && av.same(bv),
            (
                Self::Variable {
                    token: a,
                    binding: at,
                },
                Self::Variable {
                    token: b,
                    binding: bt,
                },
            ) => a == b && variable_same(at, bt),
            _ => false,
        }
    }
}
/// One literal-to-math unit projection shared by both real input domains.
pub(crate) fn converted_literal(
    value: super::literal::LiteralValue,
    token: OperandReference,
    ty: Type,
) -> NumericLeaf {
    let value_in_canonical = match value.unit {
        Some(unit) => match unit.intrinsic_projection() {
            Some(scale) => scale.apply(value.value),
            None => {
                return NumericLeaf::Contextual {
                    token,
                    unit,
                    number: Number {
                        value: value.value,
                        ty,
                    },
                };
            }
        },
        None => value.value,
    };
    NumericLeaf::Literal(Number {
        value: value_in_canonical,
        ty,
    })
}
fn variable_same(a: Variable, b: Variable) -> bool {
    match (a, b) {
        (Variable::Channel(a), Variable::Channel(b)) => a == b,
        (Variable::Profile { name: a }, Variable::Profile { name: b }) => a == b,
        _ => false,
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum NumericAction {
    ReadLexical {
        index: usize,
    },
    Classify,
    ReadNumberHeader {
        index: usize,
    },
    ReadNumber {
        index: usize,
    },
    AppendScratch {
        index: usize,
    },
    ReplaceScratch {
        index: usize,
    },
    ComputeNumber,
    AppendLeaf {
        index: usize,
    },
    ResolveIdentifier {
        token: ClassifiedToken,
    },
    AppendExpression {
        index: usize,
    },
    ReadExpression {
        index: usize,
    },
    ComputeGrammar,
    Done {
        root: usize,
        ty: Type,
    },
    Failed {
        kind: engine::FailureKind,
        site: usize,
    },
    Stopped,
}
#[expect(
    clippy::large_enum_variant,
    reason = "Only one numeric parsing continuation is live; inline decimal registers keep receiver storage caller-accounted without an extra heap protocol"
)]
enum State<'owner> {
    Input,
    Header(ClassifiedToken),
    Number(ClassifiedToken, LiteralConversion<'owner>),
    Identifier(ClassifiedToken),
    Leaf(ClassifiedToken, NumericLeaf),
    Stopped,
}
pub(crate) struct NumericGrammar<'owner> {
    table: LexicalTable<'owner>,
    reader: TokenReader<'owner>,
    engine: engine::ExpressionCursor,
    state: State<'owner>,
    scratch_end: usize,
    expression_start: usize,
    expression_length: usize,
    last_token: usize,
    ended: bool,
    context: InputContext,
    nested: Option<InputContext>,
}
#[derive(Clone, Copy)]
pub(crate) struct ExpressionTable<'owner> {
    owner: &'owner crate::color::controlled::CssColorOperationOwner,
    phase: usize,
    pub(crate) start: usize,
    pub(crate) length: usize,
    pub(crate) root: usize,
    pub(crate) ty: Type,
    end: ExpressionEnd,
    pub(crate) closed: bool,
}
/// Fixed algorithm metadata. The actual slot lends its anchor and phase when
/// this descriptor is reconstructed; it carries no backing or extra issuer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExpressionDescriptor {
    start: usize,
    length: usize,
    root: usize,
    ty: Type,
    end: ExpressionEnd,
    closed: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExpressionEnd {
    Source { last_token: usize },
    Authored { site: usize },
}
impl ExpressionDescriptor {
    pub(crate) const fn numeric_type(self) -> Type {
        self.ty
    }
    pub(crate) fn authored(
        start: usize,
        length: usize,
        root: usize,
        ty: Type,
        site: usize,
        closed: bool,
    ) -> Result<Self, ColorProtocolError> {
        if start >= length || root < start || root >= length {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(Self {
            start,
            length,
            root,
            ty,
            end: ExpressionEnd::Authored { site },
            closed,
        })
    }
}
impl ExpressionTable<'_> {
    pub(crate) fn source_last_token(&self) -> Result<usize, ColorProtocolError> {
        match self.end {
            ExpressionEnd::Source { last_token } => Ok(last_token),
            ExpressionEnd::Authored { .. } => Err(ColorProtocolError::WrongSlot),
        }
    }
    pub(crate) fn descriptor(&self) -> ExpressionDescriptor {
        ExpressionDescriptor {
            start: self.start,
            length: self.length,
            root: self.root,
            ty: self.ty,
            end: self.end,
            closed: self.closed,
        }
    }
    pub(crate) fn record(
        &self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<engine::Record, ColorProtocolError> {
        self.qualify(index, slot)?;
        if index < self.start || index >= self.length {
            return Err(ColorProtocolError::WrongIndex);
        }
        let Data::Expression(value) = slot.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        Ok(value.record)
    }
    fn qualify(&self, index: usize, slot: &CssColorSlot<'_>) -> Result<(), ColorProtocolError> {
        slot.qualify(self.owner, self.phase, index, slot.data.bank())?;
        if index != slot.index {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(())
    }
}
impl<'owner> ExpressionTable<'owner> {
    pub(crate) fn from_descriptor(
        input: &super::grammar::GrammarInput<'owner>,
        descriptor: ExpressionDescriptor,
    ) -> Result<Self, ColorProtocolError> {
        let ExpressionDescriptor {
            start,
            length,
            root,
            ty,
            end,
            closed,
        } = descriptor;
        let records = input.records();
        let site = match (input, end) {
            (super::grammar::GrammarInput::Source(_), ExpressionEnd::Source { last_token }) => {
                last_token
            }
            (super::grammar::GrammarInput::Authored(_), ExpressionEnd::Authored { site }) => site,
            _ => return Err(ColorProtocolError::WrongSlot),
        };
        if start >= length || root < start || root >= length || site >= records.length {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(Self {
            owner: records.owner,
            phase: records.phase,
            start,
            length,
            root,
            ty,
            end,
            closed,
        })
    }
}
impl<'owner> NumericGrammar<'owner> {
    pub(crate) fn start(
        table: &LexicalTable<'owner>,
        opener: ClassifiedToken,
        header: &CssColorSlot<'_>,
        scratch_end: usize,
        expression_start: usize,
    ) -> Result<Self, ColorProtocolError> {
        let Word::Math(function) = opener.word else {
            return Err(ColorProtocolError::Input);
        };
        if !matches!(
            opener.token.kind,
            RawTokenKind::Name {
                kind: NameKind::Function,
                ..
            }
        ) || scratch_end < table.length()
        {
            return Err(ColorProtocolError::WrongSlot);
        }
        let reader = TokenReader::start_after(table, opener, header)?;
        Ok(Self {
            table: *table,
            reader,
            engine: engine::ExpressionCursor::new(
                engine::Entry::Function {
                    function,
                    site: opener.index,
                },
                expression_start,
            ),
            state: State::Input,
            scratch_end,
            expression_start,
            expression_length: expression_start,
            last_token: opener.index,
            ended: false,
            context: InputContext {
                function: Some(function),
                leading: true,
            },
            nested: None,
        })
    }
    pub(crate) fn action(&self) -> NumericAction {
        match &self.state {
            State::Stopped => NumericAction::Stopped,
            State::Header(token) => NumericAction::ReadNumberHeader { index: token.index },
            State::Identifier(token) => NumericAction::ResolveIdentifier { token: *token },
            State::Leaf(_, _) => NumericAction::AppendLeaf {
                index: self.scratch_end,
            },
            State::Number(_, cursor) => match cursor.action() {
                LiteralAction::ReadSource { index }
                | LiteralAction::ReadScaled { index }
                | LiteralAction::ReadScratch { index }
                | LiteralAction::ReadHeader { index } => NumericAction::ReadNumber { index },
                LiteralAction::Append { index } => NumericAction::AppendScratch { index },
                LiteralAction::Replace { index } => NumericAction::ReplaceScratch { index },
                LiteralAction::Compute | LiteralAction::Done(_) => NumericAction::ComputeNumber,
                LiteralAction::Stopped => NumericAction::Stopped,
            },
            State::Input => match self.engine.request().expect("active expression request") {
                engine::Request::ReadInput { .. } => match self.reader.action() {
                    TokenAction::ReadRecord { index } | TokenAction::ReadName { index } => {
                        NumericAction::ReadLexical { index }
                    }
                    TokenAction::Classify => NumericAction::Classify,
                    TokenAction::Yield | TokenAction::End => NumericAction::ComputeGrammar,
                    TokenAction::Stopped => NumericAction::Stopped,
                },
                engine::Request::ReadRecord { index, .. } => {
                    NumericAction::ReadExpression { index: *index }
                }
                engine::Request::AppendRecord { index, .. } => {
                    NumericAction::AppendExpression { index: *index }
                }
                engine::Request::Compute => NumericAction::ComputeGrammar,
                engine::Request::Done { node, ty, .. } => NumericAction::Done {
                    root: *node,
                    ty: *ty,
                },
                engine::Request::Failed { kind, site } => NumericAction::Failed {
                    kind: *kind,
                    site: *site,
                },
            },
        }
    }
    pub(crate) fn read_lexical(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if !matches!(self.action(), NumericAction::ReadLexical { index: expected } if expected == index)
        {
            return Err(ColorProtocolError::WrongAction);
        }
        self.reader.read(index, slot)
    }
    pub(crate) fn read_number(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        match &mut self.state {
            State::Header(token) if index == token.index => {
                let cursor = LiteralConversion::start(&self.table, *token, slot, self.scratch_end)?;
                self.state = State::Number(*token, cursor);
                Ok(())
            }
            State::Number(_, cursor) => cursor.read(index, slot),
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(crate) fn record(&self, index: usize) -> Result<CssColorSlot<'owner>, ColorProtocolError> {
        index.checked_add(1).ok_or(ColorProtocolError::Overflow)?;
        let data = match &self.state {
            State::Number(_, cursor) => return cursor.record(index),
            State::Leaf(_, number) if index == self.scratch_end => Data::NumericLeaf(*number),
            State::Input => match self.engine.request() {
                Some(engine::Request::AppendRecord {
                    index: expected,
                    record,
                }) if index == *expected => Data::Expression(ExpressionRecord {
                    record: *record,
                    suspended: matches!(record, engine::Record::Frame(_)).then_some(self.context),
                }),
                _ => return Err(ColorProtocolError::WrongAction),
            },
            _ => return Err(ColorProtocolError::WrongAction),
        };
        Ok(CssColorSlot::new(
            self.table.owner,
            self.table.phase,
            index,
            data,
        ))
    }
    pub(crate) fn written(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        self.qualify(index, slot)?;
        let next = index.checked_add(1).ok_or(ColorProtocolError::Overflow)?;
        match &mut self.state {
            State::Number(_, cursor) => {
                cursor.written(index, slot)?;
                self.scratch_end = cursor.scratch_end();
            }
            State::Leaf(_, expected) if index == self.scratch_end => {
                let Data::NumericLeaf(actual) = slot.data else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                if !actual.same(*expected) {
                    return Err(ColorProtocolError::WrongSlot);
                }
                let Some(engine::Request::ReadInput { ordinal }) = self.engine.request().copied()
                else {
                    return Err(ColorProtocolError::WrongAction);
                };
                let State::Leaf(token, _) = self.state else {
                    unreachable!();
                };
                self.engine
                    .acknowledge_input(
                        ordinal,
                        Some(engine::Input::new(
                            token.index,
                            engine::InputKind::Operand {
                                leaf: index,
                                ty: actual.ty(),
                            },
                        )),
                    )
                    .map_err(|_| ColorProtocolError::Input)?;
                self.consume()?;
                self.context.leading = false;
                self.scratch_end = next;
                self.state = State::Input;
            }
            State::Input => {
                let Data::Expression(actual) = slot.data else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                let Some(engine::Request::AppendRecord {
                    index: expected,
                    record,
                }) = self.engine.request().copied()
                else {
                    return Err(ColorProtocolError::WrongAction);
                };
                let suspended = matches!(record, engine::Record::Frame(_)).then_some(self.context);
                if index != expected || actual != (ExpressionRecord { record, suspended }) {
                    return Err(ColorProtocolError::WrongSlot);
                }
                if suspended.is_some() && self.nested.is_none() {
                    return Err(ColorProtocolError::WrongSlot);
                }
                self.engine
                    .acknowledge_append(index, true)
                    .map_err(|_| ColorProtocolError::Input)?;
                self.expression_length = next;
                if suspended.is_some() {
                    self.context = self.nested.take().expect("checked nested input context");
                }
            }
            _ => return Err(ColorProtocolError::WrongAction),
        }
        Ok(())
    }
    pub(crate) fn read_expression(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        self.qualify(index, slot)?;
        let Data::Expression(record) = slot.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        let context = record.suspended.ok_or(ColorProtocolError::WrongSlot)?;
        let Some(engine::Request::ReadRecord { kind, .. }) = self.engine.request() else {
            return Err(ColorProtocolError::WrongAction);
        };
        if record.record.kind() != *kind {
            return Err(ColorProtocolError::WrongSlot);
        }
        if let engine::Record::Frame(frame) = record.record
            && frame
                .parent()
                .is_some_and(|parent| parent < self.expression_start || parent >= index)
        {
            return Err(ColorProtocolError::WrongIndex);
        }
        self.engine
            .acknowledge_record(index, Some(record.record))
            .map_err(|_| ColorProtocolError::Input)?;
        self.context = context;
        Ok(())
    }
    fn qualify(&self, index: usize, slot: &CssColorSlot<'_>) -> Result<(), ColorProtocolError> {
        slot.qualify(self.table.owner, self.table.phase, index, slot.data.bank())?;
        if index != slot.index {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(())
    }
    pub(crate) fn relative_channel(
        &self,
        environment: crate::color::CssRelativeColorEnvironment,
    ) -> Result<Option<crate::color::CssRelativeColorChannel>, ColorProtocolError> {
        if !matches!(self.state, State::Identifier(_)) {
            return Err(ColorProtocolError::WrongAction);
        }
        self.reader.relative_channel(environment)
    }
    pub(crate) fn relative_identifier(
        &mut self,
        environment: crate::color::CssRelativeColorEnvironment,
    ) -> Result<(), ColorProtocolError> {
        let channel = self
            .relative_channel(environment)?
            .ok_or(ColorProtocolError::Input)?;
        let State::Identifier(token) = self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        self.state = State::Leaf(
            token,
            NumericLeaf::Variable {
                token: OperandReference::Source(token),
                binding: Variable::Channel(channel),
            },
        );
        Ok(())
    }
    pub(crate) fn profile_identifier(&mut self) -> Result<(), ColorProtocolError> {
        let State::Identifier(token) = self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        let RawTokenKind::Name {
            kind: NameKind::Ident,
            units: name,
        } = token.token.kind
        else {
            return Err(ColorProtocolError::WrongSlot);
        };
        self.state = State::Leaf(
            token,
            NumericLeaf::Variable {
                token: OperandReference::Source(token),
                binding: Variable::Profile {
                    name: crate::color::controlled::protocol::RecordRange {
                        start: name.start,
                        end: name.end,
                    },
                },
            },
        );
        Ok(())
    }
    pub(crate) fn kernel(&self) -> crate::color::controlled::CssColorKernel {
        match &self.state {
            State::Number(_, cursor) => {
                if matches!(cursor.action(), LiteralAction::Done(_)) {
                    crate::color::controlled::CssColorKernel::Literal
                } else {
                    cursor.kernel()
                }
            }
            State::Input if self.reader.action() == TokenAction::Classify => {
                crate::color::controlled::CssColorKernel::Classify
            }
            _ => crate::color::controlled::CssColorKernel::Grammar,
        }
    }
    pub(crate) fn compute(&mut self) -> Result<(), ColorProtocolError> {
        match &mut self.state {
            State::Number(token, cursor) => {
                if let LiteralAction::Done(value) = cursor.action() {
                    self.state = State::Leaf(
                        *token,
                        converted_literal(value, OperandReference::Source(*token), value.ty),
                    );
                    Ok(())
                } else {
                    cursor.compute()
                }
            }
            State::Input => match self.engine.request().copied() {
                Some(engine::Request::Compute) => self
                    .engine
                    .acknowledge_compute()
                    .map_err(|_| ColorProtocolError::Input),
                Some(engine::Request::ReadInput { ordinal }) => {
                    if self.reader.action() == TokenAction::Classify {
                        return self.reader.classify();
                    }
                    if self.reader.action() == TokenAction::End {
                        self.ended = true;
                        return self
                            .engine
                            .acknowledge_input(
                                ordinal,
                                Some(engine::Input::new(
                                    self.table.length(),
                                    engine::InputKind::End,
                                )),
                            )
                            .map_err(|_| ColorProtocolError::Input);
                    }
                    let token = self.reader.current()?;
                    let admits_operand = self
                        .engine
                        .admits_operand(ordinal)
                        .map_err(|_| ColorProtocolError::Input)?;
                    let kind = match token.token.kind {
                        RawTokenKind::Number { .. } if admits_operand => {
                            self.state = State::Header(token);
                            return Ok(());
                        }
                        RawTokenKind::Whitespace => engine::InputKind::Whitespace,
                        RawTokenKind::Comment { .. } => engine::InputKind::Comment,
                        RawTokenKind::Delim('+') => engine::InputKind::Plus,
                        RawTokenKind::Delim('-') => engine::InputKind::Minus,
                        RawTokenKind::Delim('*') => engine::InputKind::Multiply,
                        RawTokenKind::Delim('/') => engine::InputKind::Divide,
                        RawTokenKind::Delim(',') => engine::InputKind::Comma,
                        RawTokenKind::Delim('(') => engine::InputKind::OpenGroup,
                        RawTokenKind::Delim(')') => engine::InputKind::Close,
                        RawTokenKind::Name {
                            kind: NameKind::Function,
                            ..
                        } => match token.word {
                            Word::Math(function) => engine::InputKind::OpenFunction(function),
                            _ => engine::InputKind::Invalid,
                        },
                        RawTokenKind::Name {
                            kind: NameKind::Ident,
                            ..
                        } => match token.word {
                            Word::None => engine::InputKind::None,
                            Word::Strategy(strategy)
                                if self.context.function
                                    == Some(crate::numeric::CssMathFunction::Round)
                                    && self.context.leading =>
                            {
                                engine::InputKind::Strategy(strategy)
                            }
                            Word::Constant(constant) if admits_operand => {
                                self.state = State::Leaf(
                                    token,
                                    NumericLeaf::Literal(Number {
                                        value: constant.magnitude(),
                                        ty: Type::NUMBER,
                                    }),
                                );
                                return Ok(());
                            }
                            _ if admits_operand => {
                                self.state = State::Identifier(token);
                                return Ok(());
                            }
                            _ => engine::InputKind::Invalid,
                        },
                        _ => engine::InputKind::Invalid,
                    };
                    self.engine
                        .acknowledge_input(ordinal, Some(engine::Input::new(token.index, kind)))
                        .map_err(|_| ColorProtocolError::Input)?;
                    if !matches!(
                        kind,
                        engine::InputKind::Whitespace | engine::InputKind::Comment
                    ) {
                        self.context.leading = false;
                    }
                    self.nested = match kind {
                        engine::InputKind::OpenFunction(function) => Some(InputContext {
                            function: Some(function),
                            leading: true,
                        }),
                        engine::InputKind::OpenGroup => Some(InputContext {
                            function: None,
                            leading: true,
                        }),
                        _ => None,
                    };
                    self.consume()
                }
                _ => Err(ColorProtocolError::WrongAction),
            },
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    fn consume(&mut self) -> Result<(), ColorProtocolError> {
        let token = self.reader.current()?;
        self.reader.consumed()?;
        self.last_token = token.index;
        Ok(())
    }
    pub(crate) fn scratch_end(&self) -> usize {
        self.scratch_end
    }
    pub(crate) fn finish(&self) -> Result<ExpressionTable<'owner>, ColorProtocolError> {
        let NumericAction::Done { root, ty } = self.action() else {
            return Err(ColorProtocolError::Terminal);
        };
        Ok(ExpressionTable {
            owner: self.table.owner,
            phase: self.table.phase,
            start: self.expression_start,
            length: self.expression_length,
            root,
            ty,
            end: ExpressionEnd::Source {
                last_token: self.last_token,
            },
            closed: !self.ended,
        })
    }
    pub(crate) fn abandon(&mut self) {
        self.engine.stop();
        self.reader.abandon();
        self.state = State::Stopped;
    }
}

use crate::numeric::projection::math::{Operand, VariadicMath, evaluate_fixed};
#[derive(Clone, Copy, Debug)]
pub(crate) enum Evaluated {
    Empty,
    Value(Number),
    Arguments {
        fixed: [Option<Operand>; 3],
        count: usize,
        folds: [VariadicMath; 3],
    },
}
impl Evaluated {
    fn same(self, other: Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::Value(a), Self::Value(b)) => a.same(b),
            (
                Self::Arguments {
                    fixed: a,
                    count: ac,
                    folds: af,
                },
                Self::Arguments {
                    fixed: b,
                    count: bc,
                    folds: bf,
                },
            ) => {
                ac == bc
                    && a.iter().zip(b).all(|(a, b)| match (a, b) {
                        (Some(a), Some(b)) => {
                            a.degrees == b.degrees && a.value.to_bits() == b.value.to_bits()
                        }
                        (None, None) => true,
                        _ => false,
                    })
                    && af
                        .into_iter()
                        .zip(bf)
                        .all(|(a, b)| match (a.finish(), b.finish()) {
                            (Some(a), Some(b)) => a.to_bits() == b.to_bits(),
                            (None, None) => true,
                            _ => false,
                        })
            }
            _ => false,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluationAction {
    ReadExpression {
        index: usize,
    },
    ReadLeaf {
        index: usize,
    },
    ReadPrevious {
        index: usize,
    },
    ReadOperand {
        index: usize,
    },
    ResolveVariable {
        token: usize,
        binding: Variable,
    },
    ResolveUnit {
        token: usize,
        unit: crate::numeric::CssNumericUnit,
    },
    Compute,
    Append {
        index: usize,
    },
    Done,
    Stopped,
}
#[derive(Clone, Copy)]
enum EvaluationState {
    Expression,
    Previous {
        record: engine::Record,
        index: usize,
    },
    Operand {
        record: engine::Record,
        previous: Option<Evaluated>,
        index: usize,
    },
    Compute {
        record: engine::Record,
        previous: Option<Evaluated>,
        operand: Option<Evaluated>,
    },
    Leaf {
        node: engine::Node,
        index: usize,
    },
    Variable {
        token: OperandReference,
        binding: Variable,
    },
    Unit {
        token: OperandReference,
        unit: crate::numeric::CssNumericUnit,
        number: Number,
    },
    Append(Evaluated),
    Done,
    Stopped,
}
pub(crate) struct NumericEvaluation<'owner> {
    table: ExpressionTable<'owner>,
    state: EvaluationState,
    index: usize,
    evaluation_start: usize,
    root: Option<Number>,
    authored_site: Option<usize>,
}
impl<'owner> NumericEvaluation<'owner> {
    /// Evaluation begins only after the expression's immutable qualified table
    /// has been completed. Every auxiliary record is evaluated once in its
    /// emitted order; chain prefixes are not traversed again per parent.
    pub(crate) fn start(
        table: &ExpressionTable<'owner>,
        evaluation_start: usize,
    ) -> Result<Self, ColorProtocolError> {
        let length = table
            .length
            .checked_sub(table.start)
            .ok_or(ColorProtocolError::WrongIndex)?;
        if length == 0 || table.root < table.start || table.root >= table.length {
            return Err(ColorProtocolError::WrongIndex);
        }
        evaluation_start
            .checked_add(length)
            .ok_or(ColorProtocolError::Overflow)?;
        Ok(Self {
            table: *table,
            state: EvaluationState::Expression,
            index: table.start,
            evaluation_start,
            root: None,
            authored_site: None,
        })
    }
    // Constructor and each backward-link observation prove this translation.
    // It neither scans records nor consults a separate mapping population.
    fn evaluated_index(&self, arithmetic: usize) -> usize {
        self.evaluation_start + (arithmetic - self.table.start)
    }
    pub(crate) fn action(&self) -> EvaluationAction {
        match self.state {
            EvaluationState::Expression => EvaluationAction::ReadExpression { index: self.index },
            EvaluationState::Previous { index, .. } => EvaluationAction::ReadPrevious {
                index: self.evaluated_index(index),
            },
            EvaluationState::Operand { index, .. } => EvaluationAction::ReadOperand {
                index: self.evaluated_index(index),
            },
            EvaluationState::Compute { .. } => EvaluationAction::Compute,
            EvaluationState::Leaf { index, .. } => EvaluationAction::ReadLeaf { index },
            EvaluationState::Variable { token, binding } => EvaluationAction::ResolveVariable {
                token: token.index(),
                binding,
            },
            EvaluationState::Unit { token, unit, .. } => EvaluationAction::ResolveUnit {
                token: token.index(),
                unit,
            },
            EvaluationState::Append(_) => EvaluationAction::Append {
                index: self.evaluated_index(self.index),
            },
            EvaluationState::Done => EvaluationAction::Done,
            EvaluationState::Stopped => EvaluationAction::Stopped,
        }
    }
    pub(crate) const fn authored_site(&self) -> Option<usize> {
        self.authored_site
    }
    pub(crate) fn read_expression(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (EvaluationAction::ReadExpression { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        if index < self.table.start || index >= self.table.length {
            return Err(ColorProtocolError::WrongIndex);
        }
        self.table.qualify(index, slot)?;
        if let Data::AuthoredTraversal(event) = slot.data {
            if !matches!(self.table.end, ExpressionEnd::Authored { .. }) {
                return Err(ColorProtocolError::WrongSlot);
            }
            self.index
                .checked_add(1)
                .ok_or(ColorProtocolError::Overflow)?;
            self.authored_site = Some(event.site());
            self.state = EvaluationState::Append(Evaluated::Empty);
            return Ok(());
        }
        let record = self.table.record(index, slot)?;
        if matches!(self.table.end, ExpressionEnd::Authored { .. }) {
            let site = match record {
                engine::Record::Node(node) => Some(node.site()),
                engine::Record::Term(term) => term.parts().4,
                engine::Record::Factor(factor) => factor.parts().4,
                engine::Record::Argument(_) | engine::Record::Frame(_) => None,
            };
            if let Some(site) = site {
                self.authored_site = Some(site);
            }
        }
        let (previous, operand) = match record {
            engine::Record::Node(node) => {
                if let Some(closing) = node.closing() {
                    let (site, closed) = match closing {
                        engine::Closing::Close { site } => (site, true),
                        engine::Closing::End { site } => (site, false),
                    };
                    if site < node.site()
                        || (index == self.table.root
                            && (closed != self.table.closed
                                || (closed
                                    && !matches!(self.table.end, ExpressionEnd::Source { last_token } if site == last_token))))
                    {
                        return Err(ColorProtocolError::WrongSlot);
                    }
                }
                match node.kind() {
                    engine::NodeKind::Leaf { leaf } => {
                        self.state = EvaluationState::Leaf { node, index: leaf };
                        return Ok(());
                    }
                    engine::NodeKind::Group { child } => (None, Some(child)),
                    engine::NodeKind::Sum { last, .. } | engine::NodeKind::Product { last, .. } => {
                        (None, Some(last))
                    }
                    engine::NodeKind::Function { last, .. } => (None, Some(last)),
                }
            }
            engine::Record::Term(term) => {
                let (previous, _, operand, _, _) = term.parts();
                (previous, Some(operand))
            }
            engine::Record::Factor(factor) => {
                let (previous, _, operand, _, _) = factor.parts();
                (previous, Some(operand))
            }
            engine::Record::Argument(argument) => {
                let (previous, _, operand) = argument.parts();
                (previous, operand)
            }
            engine::Record::Frame(_) => (None, None),
        };
        // Actual child records are immutable and backward. These checks also
        // precede every useful dependent read in this composed receiver.
        if previous.is_some_and(|i| i < self.table.start || i >= index)
            || operand.is_some_and(|i| i < self.table.start || i >= index)
        {
            return Err(ColorProtocolError::WrongIndex);
        }
        self.state = if let Some(index) = previous {
            EvaluationState::Previous { record, index }
        } else if let Some(index) = operand {
            EvaluationState::Operand {
                record,
                previous: None,
                index,
            }
        } else {
            EvaluationState::Compute {
                record,
                previous: None,
                operand: None,
            }
        };
        Ok(())
    }
    pub(crate) fn read_leaf(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        let EvaluationState::Leaf {
            node,
            index: expected,
        } = self.state
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        if index != expected {
            return Err(ColorProtocolError::WrongIndex);
        }
        self.table.qualify(index, slot)?;
        let Data::NumericLeaf(number) = slot.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if number.ty() != node.numeric_type() {
            return Err(ColorProtocolError::WrongSlot);
        }
        self.index
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        self.state = match number {
            NumericLeaf::Literal(value) => EvaluationState::Append(Evaluated::Value(value)),
            NumericLeaf::Contextual {
                token,
                unit,
                number,
            } => EvaluationState::Unit {
                token,
                unit,
                number,
            },
            NumericLeaf::Variable { token, binding } => {
                EvaluationState::Variable { token, binding }
            }
        };
        Ok(())
    }
    pub(crate) fn variable(
        &mut self,
        token_index: usize,
        number: Number,
    ) -> Result<(), ColorProtocolError> {
        let EvaluationState::Variable { token, .. } = self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        if token_index != token.index() {
            return Err(ColorProtocolError::WrongIndex);
        }
        if number.ty != Type::NUMBER {
            return Err(ColorProtocolError::WrongSlot);
        }
        self.index
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        self.state = EvaluationState::Append(Evaluated::Value(number));
        Ok(())
    }
    /// Qualify the actual unresolved basis without changing the pending owner.
    pub(crate) fn unavailable_unit(
        &self,
        token_index: usize,
        unit: crate::numeric::CssNumericUnit,
    ) -> Result<(), ColorProtocolError> {
        let EvaluationState::Unit {
            token,
            unit: expected,
            number,
        } = self.state
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        if token_index != token.index() {
            return Err(ColorProtocolError::WrongIndex);
        }
        if unit != expected
            || number.ty != crate::numeric::expression::literal_type(Some(unit), false)
        {
            return Err(ColorProtocolError::WrongSlot);
        }
        Ok(())
    }
    /// A reached contextual-unit resolver supplies only the canonical scalar;
    /// this receiver retains the checked grammar type and actual token identity.
    #[cfg(test)]
    pub(crate) fn unit(
        &mut self,
        token_index: usize,
        value: f64,
    ) -> Result<(), ColorProtocolError> {
        let EvaluationState::Unit { token, number, .. } = self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        if token_index != token.index() {
            return Err(ColorProtocolError::WrongIndex);
        }
        self.index
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        self.state = EvaluationState::Append(Evaluated::Value(Number {
            value,
            ty: number.ty,
        }));
        Ok(())
    }
    pub(crate) fn read_value(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        self.table.qualify(index, slot)?;
        let Data::Evaluation(value) = slot.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        self.state = match self.state {
            EvaluationState::Previous {
                record,
                index: expected,
            } if index == self.evaluated_index(expected) => {
                let operand = match record {
                    engine::Record::Term(term) => Some(term.parts().2),
                    engine::Record::Factor(factor) => Some(factor.parts().2),
                    engine::Record::Argument(argument) => argument.parts().2,
                    _ => return Err(ColorProtocolError::WrongSlot),
                };
                if let Some(index) = operand {
                    EvaluationState::Operand {
                        record,
                        previous: Some(value),
                        index,
                    }
                } else {
                    EvaluationState::Compute {
                        record,
                        previous: Some(value),
                        operand: None,
                    }
                }
            }
            EvaluationState::Operand {
                record,
                previous,
                index: expected,
            } if index == self.evaluated_index(expected) => EvaluationState::Compute {
                record,
                previous,
                operand: Some(value),
            },
            _ => return Err(ColorProtocolError::WrongAction),
        };
        Ok(())
    }
    pub(crate) fn kernel(&self) -> crate::color::controlled::CssColorKernel {
        use crate::color::controlled::CssColorKernel as K;
        match self.state {
            EvaluationState::Compute {
                record: engine::Record::Argument(argument),
                ..
            } if argument.parts().2.is_some() => K::VariadicFolds,
            EvaluationState::Compute {
                record: engine::Record::Node(node),
                ..
            } => match node.kind() {
                engine::NodeKind::Function { function, .. }
                    if VariadicMath::new(function).is_none() =>
                {
                    K::Math(function)
                }
                _ => K::Transition,
            },
            EvaluationState::Compute {
                record: engine::Record::Term(_) | engine::Record::Factor(_),
                ..
            } => K::Arithmetic,
            _ => K::Transition,
        }
    }
    pub(crate) fn compute(&mut self) -> Result<(), ColorProtocolError> {
        let EvaluationState::Compute {
            record,
            previous,
            operand,
        } = self.state
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        self.index
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        let number = |value: Option<Evaluated>| -> Result<Number, ColorProtocolError> {
            let Some(Evaluated::Value(number)) = value else {
                return Err(ColorProtocolError::WrongSlot);
            };
            Ok(number)
        };
        let value = match record {
            engine::Record::Frame(_) => Evaluated::Empty,
            engine::Record::Node(node) => {
                let value = match node.kind() {
                    engine::NodeKind::Leaf { .. } => {
                        return Err(ColorProtocolError::WrongAction);
                    }
                    engine::NodeKind::Group { .. }
                    | engine::NodeKind::Sum { .. }
                    | engine::NodeKind::Product { .. } => number(operand)?.value,
                    engine::NodeKind::Function {
                        function,
                        count,
                        strategy,
                        ..
                    } => {
                        let Some(Evaluated::Arguments {
                            fixed,
                            count: actual_count,
                            folds,
                        }) = operand
                        else {
                            return Err(ColorProtocolError::WrongSlot);
                        };
                        if count != actual_count {
                            return Err(ColorProtocolError::WrongSlot);
                        }
                        if VariadicMath::new(function).is_some() {
                            folds[match function {
                                crate::numeric::CssMathFunction::Min => 0,
                                crate::numeric::CssMathFunction::Max => 1,
                                crate::numeric::CssMathFunction::Hypot => 2,
                                _ => unreachable!("actual variadic function"),
                            }]
                            .finish()
                            .ok_or(ColorProtocolError::WrongSlot)?
                        } else {
                            evaluate_fixed(function, fixed, count, strategy)
                                .map_err(|_| ColorProtocolError::WrongSlot)?
                        }
                    }
                };
                Evaluated::Value(Number {
                    value,
                    ty: node.numeric_type(),
                })
            }
            engine::Record::Term(term) => {
                let (_, _, _, operator, _) = term.parts();
                let right = number(operand)?;
                let value = if let Some(operator) = operator {
                    let left = number(previous)?.value;
                    match operator {
                        crate::numeric::CssCalculationSumOperator::Add => left + right.value,
                        crate::numeric::CssCalculationSumOperator::Subtract => left - right.value,
                    }
                } else {
                    if previous.is_some() {
                        return Err(ColorProtocolError::WrongSlot);
                    }
                    right.value
                };
                // Aggregate types are read from the final checked Node, rather
                // than independently repeating the canonical type algebra.
                Evaluated::Value(Number {
                    value,
                    ty: Type::NUMBER,
                })
            }
            engine::Record::Factor(factor) => {
                let (_, _, _, operator, _) = factor.parts();
                let right = number(operand)?;
                let value = if let Some(operator) = operator {
                    let left = number(previous)?.value;
                    match operator {
                        crate::numeric::CssCalculationProductOperator::Multiply => {
                            left * right.value
                        }
                        crate::numeric::CssCalculationProductOperator::Divide => left / right.value,
                    }
                } else {
                    if previous.is_some() {
                        return Err(ColorProtocolError::WrongSlot);
                    }
                    right.value
                };
                Evaluated::Value(Number {
                    value,
                    ty: Type::NUMBER,
                })
            }
            engine::Record::Argument(argument) => {
                let (_, ordinal, node) = argument.parts();
                let (mut fixed, count, mut folds) = match previous {
                    Some(Evaluated::Arguments {
                        fixed,
                        count,
                        folds,
                    }) => (fixed, count, folds),
                    None => (
                        [None; 3],
                        0,
                        [
                            VariadicMath::new(crate::numeric::CssMathFunction::Min).unwrap(),
                            VariadicMath::new(crate::numeric::CssMathFunction::Max).unwrap(),
                            VariadicMath::new(crate::numeric::CssMathFunction::Hypot).unwrap(),
                        ],
                    ),
                    _ => return Err(ColorProtocolError::WrongSlot),
                };
                if count != ordinal {
                    return Err(ColorProtocolError::WrongSlot);
                }
                let next = count.checked_add(1).ok_or(ColorProtocolError::Overflow)?;
                let value = if node.is_some() {
                    let value = number(operand)?;
                    Some(Operand {
                        value: value.value,
                        degrees: value.ty.is(Dimension::Angle),
                    })
                } else {
                    if operand.is_some() {
                        return Err(ColorProtocolError::WrongSlot);
                    }
                    None
                };
                if ordinal < 3 {
                    fixed[ordinal] = value;
                }
                // Each argument keeps an intrinsic fold for all three variadic
                // functions, so the later function Node need not walk its chain.
                // The actual function-specific fold is supplied below by the
                // parallel fixed aggregates (see Arguments representation).
                if let Some(value) = value {
                    for fold in &mut folds {
                        fold.push(value.value);
                    }
                }
                Evaluated::Arguments {
                    fixed,
                    count: next,
                    folds,
                }
            }
        };
        self.state = EvaluationState::Append(value);
        Ok(())
    }
    pub(crate) fn record(&self, index: usize) -> Result<CssColorSlot<'owner>, ColorProtocolError> {
        if self.action() != (EvaluationAction::Append { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        self.index
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        let EvaluationState::Append(value) = self.state else {
            unreachable!();
        };
        Ok(CssColorSlot::new(
            self.table.owner,
            self.table.phase,
            index,
            Data::Evaluation(value),
        ))
    }
    pub(crate) fn written(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (EvaluationAction::Append { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        self.table.qualify(index, slot)?;
        let EvaluationState::Append(expected) = self.state else {
            unreachable!();
        };
        let Data::Evaluation(actual) = slot.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if !actual.same(expected) {
            return Err(ColorProtocolError::WrongSlot);
        }
        let next = self
            .index
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        if self.index == self.table.root {
            let Evaluated::Value(value) = actual else {
                return Err(ColorProtocolError::WrongSlot);
            };
            self.root = Some(value);
        }
        self.index = next;
        self.state = if next == self.table.length {
            EvaluationState::Done
        } else {
            EvaluationState::Expression
        };
        Ok(())
    }
    pub(crate) fn finish(&self) -> Result<Number, ColorProtocolError> {
        if !matches!(self.state, EvaluationState::Done) {
            return Err(ColorProtocolError::Terminal);
        }
        self.root.ok_or(ColorProtocolError::WrongSlot)
    }
    pub(crate) fn abandon(&mut self) {
        self.state = EvaluationState::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::controlled::CssColorOperationOwner;
    use crate::color::controlled::protocol::{ActionDescription, LexicalOperation};
    use surgeist_css_parser::input::InputKind;

    fn lexical<'owner>(
        owner: &'owner mut CssColorOperationOwner,
        input: &[u8],
    ) -> (LexicalTable<'owner>, Vec<CssColorSlot<'owner>>) {
        let mut operation = LexicalOperation::start(owner, InputKind::Utf8, input.len()).unwrap();
        let mut slots = Vec::new();
        loop {
            let mut action = operation.next_action();
            match action.description() {
                ActionDescription::ReadInput { offset } => {
                    action.read_input(offset, u16::from(input[offset])).unwrap()
                }
                ActionDescription::ReadSlot { index, .. } => {
                    action.read_slot(index, &slots[index]).unwrap()
                }
                ActionDescription::AppendSlot { index, .. } => {
                    slots.push(action.append_record().unwrap());
                    action.appended(index).unwrap();
                }
                ActionDescription::ComputeInput | ActionDescription::ComputeToken => {
                    action.compute().unwrap()
                }
                ActionDescription::Done => return (operation.finish().unwrap(), slots),
                ActionDescription::Stopped => panic!("stopped"),
            }
        }
    }
    fn first(table: &LexicalTable<'_>, slots: &[CssColorSlot<'_>]) -> ClassifiedToken {
        let mut reader = TokenReader::start(table);
        loop {
            match reader.action() {
                TokenAction::ReadRecord { index } | TokenAction::ReadName { index } => {
                    reader.read(index, &slots[index]).unwrap()
                }
                TokenAction::Classify => reader.classify().unwrap(),
                TokenAction::Yield => return reader.current().unwrap(),
                _ => panic!("function token"),
            }
        }
    }
    fn grammar<'owner>(
        cursor: &mut NumericGrammar<'owner>,
        slots: &mut Vec<CssColorSlot<'owner>>,
        expression: &mut Vec<CssColorSlot<'owner>>,
    ) -> NumericAction {
        loop {
            match cursor.action() {
                NumericAction::ReadLexical { index } => {
                    cursor.read_lexical(index, &slots[index]).unwrap()
                }
                NumericAction::ReadNumberHeader { index } | NumericAction::ReadNumber { index } => {
                    cursor.read_number(index, &slots[index]).unwrap()
                }
                NumericAction::AppendScratch { index } | NumericAction::AppendLeaf { index } => {
                    assert_eq!(index, slots.len());
                    slots.push(cursor.record(index).unwrap());
                    cursor.written(index, &slots[index]).unwrap();
                }
                NumericAction::ReplaceScratch { index } => {
                    slots[index] = cursor.record(index).unwrap();
                    cursor.written(index, &slots[index]).unwrap();
                }
                NumericAction::AppendExpression { index } => {
                    assert_eq!(index, expression.len());
                    expression.push(cursor.record(index).unwrap());
                    cursor.written(index, &expression[index]).unwrap();
                }
                NumericAction::ReadExpression { index } => {
                    cursor.read_expression(index, &expression[index]).unwrap()
                }
                NumericAction::Classify
                | NumericAction::ComputeNumber
                | NumericAction::ComputeGrammar => cursor.compute().unwrap(),
                action => return action,
            }
        }
    }
    fn evaluate<'owner>(
        table: &ExpressionTable<'owner>,
        slots: &[CssColorSlot<'owner>],
        expression: &[CssColorSlot<'owner>],
    ) -> (Number, Vec<CssColorSlot<'owner>>) {
        let mut cursor = NumericEvaluation::start(table, 0).unwrap();
        let mut values = Vec::new();
        loop {
            match cursor.action() {
                EvaluationAction::ReadExpression { index } => {
                    cursor.read_expression(index, &expression[index]).unwrap()
                }
                EvaluationAction::ReadLeaf { index } => {
                    cursor.read_leaf(index, &slots[index]).unwrap()
                }
                EvaluationAction::ReadPrevious { index }
                | EvaluationAction::ReadOperand { index } => {
                    cursor.read_value(index, &values[index]).unwrap()
                }
                EvaluationAction::Compute => cursor.compute().unwrap(),
                EvaluationAction::Append { index } => {
                    assert_eq!(index, values.len());
                    values.push(cursor.record(index).unwrap());
                    cursor.written(index, &values[index]).unwrap();
                }
                EvaluationAction::ResolveVariable { .. } | EvaluationAction::ResolveUnit { .. } => {
                    panic!("unbound variable or unit")
                }
                EvaluationAction::Done => return (cursor.finish().unwrap(), values),
                EvaluationAction::Stopped => panic!("stop"),
            }
        }
    }
    #[test]
    fn separate_expression_extents_share_banks_without_reading_an_earlier_expression() {
        let mut owner = CssColorOperationOwner::new();
        let (table, mut slots) = lexical(&mut owner, b"calc(1 + 2) calc(4 * 5)");
        let mut reader = TokenReader::start(&table);
        let mut openers = Vec::new();
        loop {
            match reader.action() {
                TokenAction::ReadRecord { index } | TokenAction::ReadName { index } => {
                    reader.read(index, &slots[index]).unwrap()
                }
                TokenAction::Classify => reader.classify().unwrap(),
                TokenAction::Yield => {
                    let token = reader.current().unwrap();
                    if matches!(token.word, Word::Math(_)) {
                        openers.push(token);
                    }
                    reader.consumed().unwrap();
                }
                TokenAction::End => break,
                TokenAction::Stopped => panic!("unexpected stop"),
            }
        }
        assert_eq!(openers.len(), 2);
        let mut expressions = Vec::new();
        let mut values = Vec::new();
        let mut earlier = None;
        for (opener, expected) in openers.into_iter().zip([3.0_f64, 20.0]) {
            let start = expressions.len();
            let mut syntax =
                NumericGrammar::start(&table, opener, &slots[opener.index], slots.len(), start)
                    .unwrap();
            assert!(matches!(
                grammar(&mut syntax, &mut slots, &mut expressions),
                NumericAction::Done { .. }
            ));
            let expression = syntax.finish().unwrap();
            assert_eq!(expression.start, start);
            assert_eq!(expression.length, expressions.len());
            assert_eq!(expression.ty, Type::NUMBER);
            if start > 0 {
                assert_eq!(
                    expression.record(0, &expressions[0]),
                    Err(ColorProtocolError::WrongIndex)
                );
                let (isolated, isolated_values) = evaluate(&expression, &slots, &expressions);
                assert_eq!(isolated.value.to_bits(), expected.to_bits());
                assert_eq!(isolated_values.len(), expression.length - expression.start);
                assert_eq!(isolated_values[0].index, 0);
            }
            let mut evaluation = NumericEvaluation::start(&expression, values.len()).unwrap();
            loop {
                match evaluation.action() {
                    EvaluationAction::ReadExpression { index } => evaluation
                        .read_expression(index, &expressions[index])
                        .unwrap(),
                    EvaluationAction::ReadLeaf { index } => {
                        evaluation.read_leaf(index, &slots[index]).unwrap()
                    }
                    EvaluationAction::ReadPrevious { index }
                    | EvaluationAction::ReadOperand { index } => {
                        evaluation.read_value(index, &values[index]).unwrap()
                    }
                    EvaluationAction::Compute => evaluation.compute().unwrap(),
                    EvaluationAction::Append { index } => {
                        assert_eq!(index, values.len());
                        let record = evaluation.record(index).unwrap();
                        assert_eq!(
                            record.bank(),
                            crate::color::controlled::CssColorBank::Evaluation
                        );
                        assert_eq!(
                            evaluation.written(index, &expressions[index]),
                            Err(ColorProtocolError::WrongSlot)
                        );
                        values.push(record);
                        evaluation.written(index, &values[index]).unwrap();
                    }
                    EvaluationAction::Done => break,
                    _ => panic!("literal expression"),
                }
            }
            assert_eq!(
                evaluation.finish().unwrap().value.to_bits(),
                expected.to_bits()
            );
            if let Some(old) = earlier {
                assert!(
                    matches!(values[0].data, Data::Evaluation(Evaluated::Value(number)) if number.value.to_bits() == old)
                );
            } else {
                let Data::Evaluation(Evaluated::Value(number)) = values[0].data else {
                    panic!("first leaf")
                };
                earlier = Some(number.value.to_bits());
            }
        }
        assert_eq!(values.len(), expressions.len());
    }

    #[test]
    fn canonical_grammar_and_left_order_are_shared_with_owned_numeric_nodes() {
        for (input, expected, angle) in [
            ("calc(1 + 2 * (3 - 1))", 5.0f64, false),
            ("calc(100 / 10 / 2)", 5.0, false),
            ("calc(1e16 + -1e16 + 1)", 1.0, false),
            ("min(9, 4, 7)", 4.0, false),
            ("max(-1, -2, 3)", 3.0, false),
            ("hypot(3, 4)", 5.0, false),
            ("clamp(none, 3, 2)", 2.0, false),
            ("round(up, 1.1, 1)", 2.0, false),
            ("sin(90deg)", 1.0, false),
            ("asin(1)", 90.0, true),
            ("calc(1in / 1px)", 96.0, false),
            ("calc(1s / 500ms)", 2.0, false),
            ("calc(1.9e308grad)", 1.71e308, true),
        ] {
            let mut owner = CssColorOperationOwner::new();
            let (table, mut slots) = lexical(&mut owner, input.as_bytes());
            let opener = first(&table, &slots);
            let mut syntax =
                NumericGrammar::start(&table, opener, &slots[opener.index], slots.len(), 0)
                    .unwrap();
            let mut expression = Vec::new();
            assert!(
                matches!(
                    grammar(&mut syntax, &mut slots, &mut expression),
                    NumericAction::Done { .. }
                ),
                "{input}"
            );
            let expression_table = syntax.finish().unwrap();
            let (number, values) = evaluate(&expression_table, &slots, &expression);
            assert_eq!(number.value.to_bits(), expected.to_bits(), "{input}");
            assert_eq!(number.ty.is(Dimension::Angle), angle, "{input}");
            assert_eq!(values.len(), expression.len());
        }
    }
    #[test]
    fn nested_variadics_and_missing_clamp_arguments_do_not_rewalk_chains() {
        for (input, expected) in [
            ("max(min(8, 3), hypot(3, 4), -10)", 5.0),
            ("clamp(2, 1, none)", 2.0),
            ("log(exp(1))", 1.0),
        ] {
            let mut owner = CssColorOperationOwner::new();
            let (table, mut slots) = lexical(&mut owner, input.as_bytes());
            let opener = first(&table, &slots);
            let mut syntax =
                NumericGrammar::start(&table, opener, &slots[opener.index], slots.len(), 0)
                    .unwrap();
            let mut expression = Vec::new();
            assert!(matches!(
                grammar(&mut syntax, &mut slots, &mut expression),
                NumericAction::Done { .. }
            ));
            let table = syntax.finish().unwrap();
            let (number, values) = evaluate(&table, &slots, &expression);
            assert_eq!(number.value, expected);
            assert_eq!(values.len(), table.length);
        }
    }
    #[test]
    fn errors_refusal_and_actual_foreign_banks_preserve_pending_actions() {
        for input in [
            "calc(1+ 2)",
            "calc(1 +/**/2)",
            "calc(+ 1)",
            "min(1, 1deg)",
            "clamp(1, none, 2)",
            "sin(1px)",
        ] {
            let mut owner = CssColorOperationOwner::new();
            let (table, mut slots) = lexical(&mut owner, input.as_bytes());
            let opener = first(&table, &slots);
            let mut syntax =
                NumericGrammar::start(&table, opener, &slots[opener.index], slots.len(), 0)
                    .unwrap();
            let mut expression = Vec::new();
            assert!(
                matches!(
                    grammar(&mut syntax, &mut slots, &mut expression),
                    NumericAction::Failed { .. }
                ),
                "{input}"
            );
        }
        let mut owner = CssColorOperationOwner::new();
        let (table, mut slots) = lexical(&mut owner, b"calc(2)");
        let opener = first(&table, &slots);
        let mut syntax =
            NumericGrammar::start(&table, opener, &slots[opener.index], slots.len(), 0).unwrap();
        let mut expression = Vec::new();
        assert!(matches!(
            grammar(&mut syntax, &mut slots, &mut expression),
            NumericAction::Done { .. }
        ));
        let table = syntax.finish().unwrap();
        let mut other = CssColorOperationOwner::new();
        let (foreign_table, foreign_slots) = lexical(&mut other, b"calc(3)");
        let mut evaluator = NumericEvaluation::start(&table, 0).unwrap();
        assert_eq!(
            evaluator.read_expression(0, &foreign_slots[0]),
            Err(ColorProtocolError::ForeignSlot)
        );
        assert_eq!(
            evaluator.action(),
            EvaluationAction::ReadExpression { index: 0 }
        );
        assert_eq!(
            evaluator.read_expression(0, &slots[0]),
            Err(ColorProtocolError::WrongSlot)
        );
        evaluator.read_expression(0, &expression[0]).unwrap();
        let EvaluationAction::ReadLeaf { index } = evaluator.action() else {
            panic!("leaf");
        };
        assert_eq!(
            evaluator.read_leaf(index, &foreign_slots[0]),
            Err(ColorProtocolError::ForeignSlot)
        );
        assert_eq!(evaluator.action(), EvaluationAction::ReadLeaf { index });
        evaluator.read_leaf(index, &slots[index]).unwrap();
        let pending = evaluator.record(0).unwrap();
        assert_eq!(evaluator.record(0).unwrap().kind(), pending.kind());
        evaluator.abandon();
        assert_eq!(
            evaluator.written(0, &pending),
            Err(ColorProtocolError::WrongAction)
        );
        assert_eq!(evaluator.finish().err(), Some(ColorProtocolError::Terminal));
        let _ = foreign_table;
    }
    #[test]
    fn explicit_constants_and_recovered_closure_keep_numerical_failures_distinct() {
        for (input, nan, infinite) in [
            ("calc(NaN", true, false),
            ("sqrt(-1)", true, false),
            ("calc(infinity)", false, true),
            ("calc(-0)", false, false),
        ] {
            let mut owner = CssColorOperationOwner::new();
            let (table, mut slots) = lexical(&mut owner, input.as_bytes());
            let opener = first(&table, &slots);
            let mut syntax =
                NumericGrammar::start(&table, opener, &slots[opener.index], slots.len(), 0)
                    .unwrap();
            let mut expression = Vec::new();
            assert!(matches!(
                grammar(&mut syntax, &mut slots, &mut expression),
                NumericAction::Done { .. }
            ));
            let table = syntax.finish().unwrap();
            let (number, _) = evaluate(&table, &slots, &expression);
            assert_eq!(number.value.is_nan(), nan);
            assert_eq!(number.value.is_infinite(), infinite);
            if input == "calc(-0)" {
                assert_eq!(number.value.to_bits(), 0.0f64.to_bits());
            }
        }
        let mut owner = CssColorOperationOwner::new();
        let (table, mut slots) = lexical(&mut owner, b"calc(1em)");
        let opener = first(&table, &slots);
        let mut syntax =
            NumericGrammar::start(&table, opener, &slots[opener.index], slots.len(), 0).unwrap();
        let mut expression = Vec::new();
        assert!(
            matches!(grammar(&mut syntax, &mut slots, &mut expression), NumericAction::Done { ty, .. } if ty.is(Dimension::Length))
        );
        let expression_table = syntax.finish().unwrap();
        let mut evaluation = NumericEvaluation::start(&expression_table, 0).unwrap();
        evaluation.read_expression(0, &expression[0]).unwrap();
        let EvaluationAction::ReadLeaf { index } = evaluation.action() else {
            panic!("contextual leaf")
        };
        evaluation.read_leaf(index, &slots[index]).unwrap();
        let EvaluationAction::ResolveUnit { token, unit } = evaluation.action() else {
            panic!("reached basis")
        };
        assert_eq!(
            unit,
            crate::numeric::CssNumericUnit::Length(crate::numeric::CssLengthUnit::Em)
        );
        assert_eq!(
            evaluation.unit(token + 1, 16.0),
            Err(ColorProtocolError::WrongIndex)
        );
        assert_eq!(
            evaluation.action(),
            EvaluationAction::ResolveUnit { token, unit }
        );
        evaluation.unit(token, 16.0).unwrap();
        evaluation.abandon();
        assert_eq!(evaluation.action(), EvaluationAction::Stopped);
    }
    #[test]
    fn variable_syntax_retains_real_channel_identity_before_reached_binding() {
        let mut owner = CssColorOperationOwner::new();
        let (table, mut slots) = lexical(&mut owner, b"calc(r * 2 + alpha)");
        let opener = first(&table, &slots);
        let mut syntax =
            NumericGrammar::start(&table, opener, &slots[opener.index], slots.len(), 0).unwrap();
        let mut expressions = Vec::new();
        let mut variables = Vec::new();
        loop {
            match grammar(&mut syntax, &mut slots, &mut expressions) {
                NumericAction::ResolveIdentifier { token } => {
                    variables.push((
                        token.index,
                        syntax
                            .relative_channel(crate::color::CssRelativeColorEnvironment::Rgb)
                            .unwrap()
                            .unwrap(),
                    ));
                    syntax
                        .relative_identifier(crate::color::CssRelativeColorEnvironment::Rgb)
                        .unwrap();
                }
                NumericAction::Done { ty, .. } => {
                    assert_eq!(ty, Type::NUMBER);
                    break;
                }
                _ => panic!("literal variable syntax"),
            }
        }
        assert_eq!(
            variables
                .iter()
                .map(|(_, channel)| *channel)
                .collect::<Vec<_>>(),
            [
                crate::color::CssRelativeColorChannel::R,
                crate::color::CssRelativeColorChannel::Alpha
            ]
        );
        assert_eq!(syntax.scratch_end(), slots.len());
        let expressions_table = syntax.finish().unwrap();
        let binding = super::super::relative::Binding::new(
            crate::color::CssRelativeColorEnvironment::Rgb,
            surgeist_color::Color::from_srgba8([128, 32, 16, 64]),
        )
        .unwrap();
        let mut evaluator = NumericEvaluation::start(&expressions_table, 0).unwrap();
        let mut values = Vec::new();
        loop {
            match evaluator.action() {
                EvaluationAction::ReadExpression { index } => evaluator
                    .read_expression(index, &expressions[index])
                    .unwrap(),
                EvaluationAction::ReadLeaf { index } => {
                    evaluator.read_leaf(index, &slots[index]).unwrap()
                }
                EvaluationAction::ReadPrevious { index }
                | EvaluationAction::ReadOperand { index } => {
                    evaluator.read_value(index, &values[index]).unwrap()
                }
                EvaluationAction::ResolveVariable {
                    token,
                    binding: Variable::Channel(channel),
                } => {
                    assert!(variables.contains(&(token, channel)));
                    assert_eq!(
                        evaluator.variable(
                            token + 1,
                            Number {
                                value: 0.0,
                                ty: Type::NUMBER
                            }
                        ),
                        Err(ColorProtocolError::WrongIndex)
                    );
                    let value = binding.channel(channel, true).unwrap();
                    evaluator
                        .variable(
                            token,
                            Number {
                                value: value.value,
                                ty: Type::NUMBER,
                            },
                        )
                        .unwrap();
                }
                EvaluationAction::Compute => evaluator.compute().unwrap(),
                EvaluationAction::Append { index } => {
                    values.push(evaluator.record(index).unwrap());
                    evaluator.written(index, &values[index]).unwrap();
                }
                EvaluationAction::Done => break,
                _ => panic!("actual reached binding"),
            }
        }
        assert_eq!(evaluator.finish().unwrap().value, 256.0 + 64.0 / 255.0);
        assert_eq!(values.len(), expressions_table.length);
    }
    #[test]
    fn round_strategy_position_and_custom_identifier_names_use_real_context_frames() {
        for (input, expected_variables, strategies) in [
            ("calc(up + 2)", 1, 0),
            ("calc(round(up, 1.1, 1) + 2)", 0, 1),
            ("round(2, up)", 1, 0),
            ("calc((up) + round(down, 1.1, 1))", 1, 1),
            ("calc(é + 2)", 1, 0),
        ] {
            let mut owner = CssColorOperationOwner::new();
            let (table, mut slots) = lexical(&mut owner, input.as_bytes());
            let opener = first(&table, &slots);
            let mut syntax =
                NumericGrammar::start(&table, opener, &slots[opener.index], slots.len(), 0)
                    .unwrap();
            let mut expression = Vec::new();
            let mut count = 0;
            loop {
                match grammar(&mut syntax, &mut slots, &mut expression) {
                    NumericAction::ResolveIdentifier { .. } => {
                        syntax.profile_identifier().unwrap();
                        count += 1;
                    }
                    NumericAction::Done { .. } => break,
                    _ => panic!("valid symbolic syntax: {input}"),
                }
            }
            let table = syntax.finish().unwrap();
            let mut actual = 0;
            for (index, record) in expression.iter().enumerate() {
                if let engine::Record::Node(node) = table.record(index, record).unwrap()
                    && matches!(
                        node.kind(),
                        engine::NodeKind::Function {
                            strategy: Some(_),
                            ..
                        }
                    )
                {
                    actual += 1;
                }
            }
            assert_eq!(count, expected_variables, "{input}");
            assert_eq!(actual, strategies, "{input}");
        }
    }

    #[test]
    fn adjacent_identifier_is_rejected_before_channel_resolution() {
        let mut owner = CssColorOperationOwner::new();
        let (table, mut slots) = lexical(&mut owner, b"calc(1 foo)");
        let opener = first(&table, &slots);
        let mut syntax =
            NumericGrammar::start(&table, opener, &slots[opener.index], slots.len(), 0).unwrap();
        let mut expressions = Vec::new();
        let action = grammar(&mut syntax, &mut slots, &mut expressions);
        let NumericAction::Failed {
            kind: engine::FailureKind::MalformedExpression,
            site,
        } = action
        else {
            panic!("must reject syntax before ResolveIdentifier: {action:?}");
        };
        let crate::color::controlled::protocol::LexicalRecord::Token(token) =
            table.observe(site, &slots[site]).unwrap()
        else {
            panic!("responsible identifier")
        };
        assert_eq!(
            token.source,
            surgeist_css_parser::lexer::protocol::Span { start: 7, end: 10 }
        );
        syntax.abandon();
        assert!(matches!(syntax.action(), NumericAction::Stopped));
        assert_eq!(syntax.finish().err(), Some(ColorProtocolError::Terminal));
    }
}
