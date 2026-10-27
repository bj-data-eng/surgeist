//! Incremental intrinsic CSS math grammar; the caller owns input, flat storage,
//! qualification and all admission. CSS Values 4 mathematical syntax/type rules:
//! <https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-syntax>
//! The retained 720ea286 defining supplement supplies clamp none and round(A).

pub(crate) mod authored;

use super::{
    CssCalculationProductOperator, CssCalculationSumOperator, CssMathFunction, CssNumericDimension,
    CssNumericType, CssRoundingStrategy,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Entry {
    Sum,
    Function {
        function: CssMathFunction,
        site: usize,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InputKind {
    Operand { leaf: usize, ty: CssNumericType },
    Whitespace,
    Comment,
    Plus,
    Minus,
    Multiply,
    Divide,
    Comma,
    OpenGroup,
    OpenFunction(CssMathFunction),
    Close,
    None,
    Strategy(CssRoundingStrategy),
    Invalid,
    End,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Input {
    site: usize,
    kind: InputKind,
}
impl Input {
    pub(crate) const fn new(site: usize, kind: InputKind) -> Self {
        Self { site, kind }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Closing {
    Close { site: usize },
    End { site: usize },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FailureKind {
    MalformedExpression,
    Arity,
    InvalidArgumentType,
    IncompatibleTypes,
    CapacityOverflow,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProtocolError {
    WrongRequest,
    WrongOrdinal,
    MissingInput,
    WrongRecord,
    RefusedWrite,
    Stopped,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecordKind {
    Node,
    Term,
    Factor,
    Argument,
    Frame,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NodeKind {
    Leaf {
        leaf: usize,
    },
    Group {
        child: usize,
    },
    Sum {
        last: usize,
        count: usize,
    },
    Product {
        last: usize,
        count: usize,
    },
    Function {
        function: CssMathFunction,
        last: usize,
        count: usize,
        strategy: Option<CssRoundingStrategy>,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Node {
    kind: NodeKind,
    ty: CssNumericType,
    site: usize,
    closing: Option<Closing>,
}
impl Node {
    pub(crate) const fn kind(self) -> NodeKind {
        self.kind
    }
    pub(crate) const fn numeric_type(self) -> CssNumericType {
        self.ty
    }
    pub(crate) const fn site(self) -> usize {
        self.site
    }
    pub(crate) const fn closing(self) -> Option<Closing> {
        self.closing
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Term {
    previous: Option<usize>,
    ordinal: usize,
    operand: usize,
    operator: Option<CssCalculationSumOperator>,
    operator_site: Option<usize>,
}
impl Term {
    pub(crate) const fn parts(
        self,
    ) -> (
        Option<usize>,
        usize,
        usize,
        Option<CssCalculationSumOperator>,
        Option<usize>,
    ) {
        (
            self.previous,
            self.ordinal,
            self.operand,
            self.operator,
            self.operator_site,
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Factor {
    previous: Option<usize>,
    ordinal: usize,
    operand: usize,
    operator: Option<CssCalculationProductOperator>,
    operator_site: Option<usize>,
}
impl Factor {
    pub(crate) const fn parts(
        self,
    ) -> (
        Option<usize>,
        usize,
        usize,
        Option<CssCalculationProductOperator>,
        Option<usize>,
    ) {
        (
            self.previous,
            self.ordinal,
            self.operand,
            self.operator,
            self.operator_site,
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Argument {
    previous: Option<usize>,
    ordinal: usize,
    node: Option<usize>,
}
impl Argument {
    pub(crate) const fn parts(self) -> (Option<usize>, usize, Option<usize>) {
        (self.previous, self.ordinal, self.node)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Frame {
    suspended: State,
    opening_site: usize,
    opening_ordinal: usize,
}
impl Frame {
    pub(crate) const fn parent(self) -> Option<usize> {
        if self.suspended.frame == NO_FRAME {
            None
        } else {
            Some(self.suspended.frame)
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Record {
    Node(Node),
    Term(Term),
    Factor(Factor),
    Argument(Argument),
    Frame(Frame),
}
impl Record {
    pub(crate) const fn kind(self) -> RecordKind {
        match self {
            Self::Node(_) => RecordKind::Node,
            Self::Term(_) => RecordKind::Term,
            Self::Factor(_) => RecordKind::Factor,
            Self::Argument(_) => RecordKind::Argument,
            Self::Frame(_) => RecordKind::Frame,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Request {
    ReadInput {
        ordinal: usize,
    },
    ReadRecord {
        index: usize,
        kind: RecordKind,
    },
    AppendRecord {
        index: usize,
        record: Record,
    },
    Compute,
    Done {
        node: usize,
        ty: CssNumericType,
        closing: Closing,
    },
    Failed {
        kind: FailureKind,
        site: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Context {
    Sum,
    Group {
        site: usize,
    },
    Function {
        function: CssMathFunction,
        site: usize,
    },
}
impl Context {
    fn site(self) -> usize {
        match self {
            Self::Sum => 0,
            Self::Group { site } | Self::Function { site, .. } => site,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Value {
    node: usize,
    ty: CssNumericType,
    site: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    Value { needs_space: bool },
    Factor(Value),
    AfterValue,
    Product,
    Term(Value),
    AfterProduct,
    Sum,
    Argument(Option<Value>),
    AfterArgument,
    NoneDelimiter,
    StrategyComma,
    Finish,
    Return { value: Value, closing: Closing },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Summary {
    common: Option<CssNumericType>,
    error: Option<FailureKind>,
    all_numbers: bool,
    all_simple: bool,
}
impl Summary {
    const EMPTY: Self = Self {
        common: None,
        error: None,
        all_numbers: true,
        all_simple: true,
    };
    fn accept(&mut self, ty: CssNumericType) -> Result<(), FailureKind> {
        self.all_numbers &= ty.is_number();
        self.all_simple &= ty.simple();
        if self.error.is_none() {
            match self.common {
                None => self.common = Some(ty),
                Some(old) => match checked_add(old, ty) {
                    Ok(value) => self.common = Some(value),
                    Err(FailureKind::CapacityOverflow) => {
                        return Err(FailureKind::CapacityOverflow);
                    }
                    Err(error) => self.error = Some(error),
                },
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct First {
    node: usize,
    site: usize,
}
// No append can commit usize::MAX: issue_append proves index + 1 first.
// This private root marker therefore cannot collide with a suspended frame.
const NO_FRAME: usize = usize::MAX;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct State {
    context: Context,
    frame: usize,
    opening_ordinal: usize,
    product_first: First,
    product_last: usize,
    product_count: usize,
    product_ty: Option<CssNumericType>,
    factor_operator: Option<CssCalculationProductOperator>,
    factor_operator_site: usize,
    sum_first: First,
    sum_last: usize,
    sum_count: usize,
    sum_ty: Option<CssNumericType>,
    term_operator: Option<CssCalculationSumOperator>,
    term_operator_site: usize,
    argument_last: usize,
    argument_count: usize,
    summary: Summary,
    strategy: Option<CssRoundingStrategy>,
}
impl State {
    fn new(context: Context, frame: Option<usize>, opening_ordinal: usize) -> Self {
        Self {
            context,
            frame: frame.unwrap_or(NO_FRAME),
            opening_ordinal,
            product_first: First { node: 0, site: 0 },
            product_last: 0,
            product_count: 0,
            product_ty: None,
            factor_operator: None,
            factor_operator_site: 0,
            sum_first: First { node: 0, site: 0 },
            sum_last: 0,
            sum_count: 0,
            sum_ty: None,
            term_operator: None,
            term_operator_site: 0,
            argument_last: 0,
            argument_count: 0,
            summary: Summary::EMPTY,
            strategy: None,
        }
    }
    fn clear_product(&mut self) {
        self.product_first = First { node: 0, site: 0 };
        self.product_last = 0;
        self.product_count = 0;
        self.product_ty = None;
        self.factor_operator = None;
    }
    fn clear_sum(&mut self) {
        self.clear_product();
        self.sum_first = First { node: 0, site: 0 };
        self.sum_last = 0;
        self.sum_count = 0;
        self.sum_ty = None;
        self.term_operator = None;
    }
    fn valid_suspension(self, before: usize) -> bool {
        (self.frame == NO_FRAME || self.frame < before)
            && (self.product_count == 0
                || self.product_last < before && self.product_first.node < before)
            && (self.sum_count == 0 || self.sum_last < before && self.sum_first.node < before)
            && (self.argument_count == 0 || self.argument_last < before)
            && (self.product_count == 0) == self.product_ty.is_none()
            && (self.sum_count == 0) == self.sum_ty.is_none()
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Appended {
    Leaf {
        ty: CssNumericType,
        site: usize,
    },
    Factor {
        value: Value,
        ty: CssNumericType,
        count: usize,
    },
    Product {
        ty: CssNumericType,
        site: usize,
    },
    Term {
        value: Value,
        ty: CssNumericType,
        count: usize,
    },
    Sum {
        ty: CssNumericType,
        site: usize,
    },
    Argument {
        count: usize,
        summary: Summary,
    },
    Frame {
        context: Context,
        ordinal: usize,
    },
    Finished {
        ty: CssNumericType,
        site: usize,
        closing: Closing,
    },
}

/// One pending action, one lookahead and one fixed active grammar frame. Nested
/// suspensions reside only in caller-admitted immutable Frame records.
pub(crate) struct ExpressionCursor {
    state: State,
    stage: Stage,
    whitespace: bool,
    ordinal: usize,
    next_record: usize,
    input: Option<Input>,
    pending: Option<Request>,
    appended: Option<Appended>,
}
impl ExpressionCursor {
    pub(crate) fn new(entry: Entry, first_record: usize) -> Self {
        let context = match entry {
            Entry::Sum => Context::Sum,
            Entry::Function { function, site } => Context::Function { function, site },
        };
        Self {
            state: State::new(context, None, 0),
            stage: Stage::Value { needs_space: false },
            whitespace: false,
            ordinal: 0,
            next_record: first_record,
            input: None,
            pending: Some(Request::ReadInput { ordinal: 0 }),
            appended: None,
        }
    }
    pub(crate) const fn request(&self) -> Option<&Request> {
        self.pending.as_ref()
    }
    /// Qualifies operand work to the actual pending input and value position.
    /// Callers classify punctuation/trivia first; they resolve numeric leaves
    /// only after this grammar proves that an operand is syntactically possible.
    pub(crate) fn admits_operand(&self, ordinal: usize) -> Result<bool, ProtocolError> {
        let Request::ReadInput { ordinal: expected } = self.require()? else {
            return Err(ProtocolError::WrongRequest);
        };
        if ordinal != expected {
            return Err(ProtocolError::WrongOrdinal);
        }
        Ok(matches!(self.stage, Stage::Value { needs_space } if !needs_space || self.whitespace))
    }
    /// Parent terminal resource/cancellation refusal. Caller storage is untouched.
    pub(crate) fn stop(&mut self) {
        self.pending = None;
        self.appended = None;
        self.input = None;
    }
    fn require(&self) -> Result<Request, ProtocolError> {
        self.pending.ok_or(ProtocolError::Stopped)
    }
    pub(crate) fn acknowledge_input(
        &mut self,
        ordinal: usize,
        input: Option<Input>,
    ) -> Result<(), ProtocolError> {
        let Request::ReadInput { ordinal: expected } = self.require()? else {
            return Err(ProtocolError::WrongRequest);
        };
        if expected != ordinal {
            return Err(ProtocolError::WrongOrdinal);
        }
        let input = input.ok_or(ProtocolError::MissingInput)?;
        // Overflow is checked before accepting an input read, not after consuming it.
        if ordinal.checked_add(1).is_none() {
            self.fail(FailureKind::CapacityOverflow, input.site);
        } else {
            self.input = Some(input);
            self.pending = Some(Request::Compute);
        }
        Ok(())
    }
    pub(crate) fn acknowledge_append(
        &mut self,
        index: usize,
        accepted: bool,
    ) -> Result<(), ProtocolError> {
        let Request::AppendRecord {
            index: expected, ..
        } = self.require()?
        else {
            return Err(ProtocolError::WrongRequest);
        };
        if index != expected {
            return Err(ProtocolError::WrongRecord);
        }
        if !accepted {
            return Err(ProtocolError::RefusedWrite);
        }
        // issue_append proved this increment before the caller's write.
        self.next_record = index + 1;
        match self.appended.take().expect("append continuation") {
            Appended::Leaf { ty, site } => {
                self.stage = Stage::Factor(Value {
                    node: index,
                    ty,
                    site,
                })
            }
            Appended::Factor { value, ty, count } => {
                if self.state.product_count == 0 {
                    self.state.product_first = First {
                        node: value.node,
                        site: value.site,
                    };
                }
                self.state.product_last = index;
                self.state.product_count = count;
                self.state.product_ty = Some(ty);
                self.stage = Stage::AfterValue;
            }
            Appended::Product { ty, site } => {
                self.stage = Stage::Term(Value {
                    node: index,
                    ty,
                    site,
                })
            }
            Appended::Term { value, ty, count } => {
                if self.state.sum_count == 0 {
                    self.state.sum_first = First {
                        node: value.node,
                        site: value.site,
                    };
                }
                self.state.sum_last = index;
                self.state.sum_count = count;
                self.state.sum_ty = Some(ty);
                self.stage = Stage::AfterProduct;
            }
            Appended::Sum { ty, site } => {
                self.stage = Stage::Argument(Some(Value {
                    node: index,
                    ty,
                    site,
                }))
            }
            Appended::Argument { count, summary } => {
                self.state.argument_last = index;
                self.state.argument_count = count;
                self.state.summary = summary;
                self.stage = Stage::AfterArgument;
            }
            Appended::Frame { context, ordinal } => {
                self.state = State::new(context, Some(index), ordinal);
                self.whitespace = false;
                self.stage = Stage::Value { needs_space: false };
            }
            Appended::Finished { ty, site, closing } => {
                self.stage = Stage::Return {
                    value: Value {
                        node: index,
                        ty,
                        site,
                    },
                    closing,
                }
            }
        }
        self.pending = Some(Request::Compute);
        Ok(())
    }
    pub(crate) fn acknowledge_record(
        &mut self,
        index: usize,
        record: Option<Record>,
    ) -> Result<(), ProtocolError> {
        let Request::ReadRecord {
            index: expected,
            kind,
        } = self.require()?
        else {
            return Err(ProtocolError::WrongRequest);
        };
        if index != expected {
            return Err(ProtocolError::WrongRecord);
        }
        let Some(Record::Frame(frame)) = record else {
            return Err(ProtocolError::WrongRecord);
        };
        if kind != RecordKind::Frame
            || frame.opening_site != self.state.context.site()
            || frame.opening_ordinal != self.state.opening_ordinal
            || index >= self.next_record
            || !frame.suspended.valid_suspension(index)
        {
            return Err(ProtocolError::WrongRecord);
        }
        let Stage::Return { value, closing } = self.stage else {
            return Err(ProtocolError::WrongRecord);
        };
        self.state = frame.suspended;
        self.stage = Stage::Factor(value);
        self.whitespace = false;
        // Explicit close was consumed; End stays as the recovered enclosing boundary.
        if matches!(closing, Closing::Close { .. }) {
            self.input = None;
        }
        self.pending = Some(Request::Compute);
        Ok(())
    }
    fn fail(&mut self, kind: FailureKind, site: usize) {
        self.pending = Some(Request::Failed { kind, site });
    }
    fn issue_append(&mut self, record: Record, after: Appended, site: usize) {
        if self.next_record.checked_add(1).is_none() {
            self.fail(FailureKind::CapacityOverflow, site);
            return;
        }
        self.appended = Some(after);
        self.pending = Some(Request::AppendRecord {
            index: self.next_record,
            record,
        });
    }
    fn consume(&mut self) {
        // Input acknowledgement proved the increment.
        self.ordinal += 1;
        self.input = None;
    }
    fn again(&mut self) {
        self.pending = Some(if self.input.is_some() {
            Request::Compute
        } else {
            Request::ReadInput {
                ordinal: self.ordinal,
            }
        });
    }
    pub(crate) fn acknowledge_compute(&mut self) -> Result<(), ProtocolError> {
        if self.require()? != Request::Compute {
            return Err(ProtocolError::WrongRequest);
        }
        match self.stage {
            Stage::Factor(value) => self.factor(value),
            Stage::Product => self.product(),
            Stage::Term(value) => self.term(value),
            Stage::Sum => self.sum(),
            Stage::Argument(value) => self.argument(value),
            Stage::Finish => self.finish(),
            Stage::Return { value, closing } => {
                self.pending = Some(match self.state.frame {
                    index if index != NO_FRAME => Request::ReadRecord {
                        index,
                        kind: RecordKind::Frame,
                    },
                    _ => Request::Done {
                        node: value.node,
                        ty: value.ty,
                        closing,
                    },
                });
            }
            stage => {
                let Some(input) = self.input else {
                    self.again();
                    return Ok(());
                };
                if matches!(input.kind, InputKind::Whitespace | InputKind::Comment) {
                    self.whitespace |= input.kind == InputKind::Whitespace;
                    self.consume();
                    self.again();
                } else {
                    match stage {
                        Stage::Value { needs_space } => self.value(input, needs_space),
                        Stage::AfterValue => self.after_value(input),
                        Stage::AfterProduct => self.after_product(input),
                        Stage::AfterArgument => self.after_argument(input),
                        Stage::NoneDelimiter => {
                            if matches!(
                                input.kind,
                                InputKind::Comma | InputKind::Close | InputKind::End
                            ) {
                                self.stage = Stage::Argument(None);
                                self.again();
                            } else {
                                self.fail(FailureKind::MalformedExpression, input.site);
                            }
                        }
                        Stage::StrategyComma => {
                            if input.kind == InputKind::Comma {
                                self.consume();
                                self.stage = Stage::Value { needs_space: false };
                                self.whitespace = false;
                                self.again();
                            } else {
                                self.fail(FailureKind::Arity, input.site);
                            }
                        }
                        _ => unreachable!("fixed compute dispatch"),
                    }
                }
            }
        }
        Ok(())
    }
    fn value(&mut self, input: Input, needs_space: bool) {
        if needs_space && !self.whitespace {
            self.fail(FailureKind::MalformedExpression, input.site);
            return;
        }
        match input.kind {
            InputKind::Operand { leaf, ty } => {
                self.consume();
                self.whitespace = false;
                self.issue_append(
                    Record::Node(Node {
                        kind: NodeKind::Leaf { leaf },
                        ty,
                        site: input.site,
                        closing: None,
                    }),
                    Appended::Leaf {
                        ty,
                        site: input.site,
                    },
                    input.site,
                );
            }
            InputKind::OpenGroup | InputKind::OpenFunction(_) => {
                let context = match input.kind {
                    InputKind::OpenFunction(function) => Context::Function {
                        function,
                        site: input.site,
                    },
                    _ => Context::Group { site: input.site },
                };
                let ordinal = self.ordinal;
                self.consume();
                let frame = Frame {
                    suspended: self.state,
                    opening_site: input.site,
                    opening_ordinal: ordinal,
                };
                self.issue_append(
                    Record::Frame(frame),
                    Appended::Frame { context, ordinal },
                    input.site,
                );
            }
            InputKind::None
                if self.state.product_count == 0
                    && self.state.sum_count == 0
                    && matches!(
                        self.state.context,
                        Context::Function {
                            function: CssMathFunction::Clamp,
                            ..
                        }
                    )
                    && matches!(self.state.argument_count, 0 | 2) =>
            {
                self.consume();
                self.stage = Stage::NoneDelimiter;
                self.again();
            }
            InputKind::Strategy(strategy)
                if self.state.argument_count == 0
                    && self.state.product_count == 0
                    && self.state.sum_count == 0
                    && self.state.strategy.is_none()
                    && matches!(
                        self.state.context,
                        Context::Function {
                            function: CssMathFunction::Round,
                            ..
                        }
                    ) =>
            {
                self.consume();
                self.state.strategy = Some(strategy);
                self.stage = Stage::StrategyComma;
                self.again();
            }
            InputKind::Comma | InputKind::Close | InputKind::End
                if self.state.product_count == 0
                    && self.state.sum_count == 0
                    && matches!(self.state.context, Context::Function { .. }) =>
            {
                self.fail(FailureKind::Arity, input.site)
            }
            _ => self.fail(FailureKind::MalformedExpression, input.site),
        }
    }
    fn factor(&mut self, value: Value) {
        let Some(count) = self.state.product_count.checked_add(1) else {
            self.fail(FailureKind::CapacityOverflow, value.site);
            return;
        };
        let ty = match self.state.product_ty {
            None => value.ty,
            Some(old) => {
                if old.hint.is_some() && value.ty.hint.is_some() && old.hint != value.ty.hint {
                    self.fail(FailureKind::IncompatibleTypes, value.site);
                    return;
                }
                let divide = matches!(
                    self.state.factor_operator,
                    Some(CssCalculationProductOperator::Divide)
                );
                let Some(ty) = old.product(value.ty, divide) else {
                    self.fail(FailureKind::CapacityOverflow, value.site);
                    return;
                };
                ty
            }
        };
        let record = Factor {
            previous: (self.state.product_count != 0).then_some(self.state.product_last),
            ordinal: self.state.product_count,
            operand: value.node,
            operator: self.state.factor_operator,
            operator_site: self
                .state
                .factor_operator
                .map(|_| self.state.factor_operator_site),
        };
        self.issue_append(
            Record::Factor(record),
            Appended::Factor { value, ty, count },
            value.site,
        );
    }
    fn after_value(&mut self, input: Input) {
        match input.kind {
            InputKind::Multiply | InputKind::Divide => {
                self.state.factor_operator = Some(if input.kind == InputKind::Divide {
                    CssCalculationProductOperator::Divide
                } else {
                    CssCalculationProductOperator::Multiply
                });
                self.state.factor_operator_site = input.site;
                self.consume();
                self.whitespace = false;
                self.stage = Stage::Value { needs_space: false };
                self.again();
            }
            _ => {
                self.stage = Stage::Product;
                self.again();
            }
        }
    }
    fn product(&mut self) {
        let first = self.state.product_first;
        if self.state.product_count == 1 {
            self.stage = Stage::Term(Value {
                node: first.node,
                site: first.site,
                ty: self.state.product_ty.expect("factor type"),
            });
            self.again();
        } else {
            let ty = self.state.product_ty.expect("factor type");
            self.issue_append(
                Record::Node(Node {
                    kind: NodeKind::Product {
                        last: self.state.product_last,
                        count: self.state.product_count,
                    },
                    ty,
                    site: first.site,
                    closing: None,
                }),
                Appended::Product {
                    ty,
                    site: first.site,
                },
                first.site,
            );
        }
    }
    fn term(&mut self, value: Value) {
        let Some(count) = self.state.sum_count.checked_add(1) else {
            self.fail(FailureKind::CapacityOverflow, value.site);
            return;
        };
        let ty = match self.state.sum_ty {
            None => value.ty,
            Some(old) => match checked_add(old, value.ty) {
                Ok(ty) => ty,
                Err(error) => {
                    self.fail(error, value.site);
                    return;
                }
            },
        };
        let record = Term {
            previous: (self.state.sum_count != 0).then_some(self.state.sum_last),
            ordinal: self.state.sum_count,
            operand: value.node,
            operator: self.state.term_operator,
            operator_site: self
                .state
                .term_operator
                .map(|_| self.state.term_operator_site),
        };
        self.issue_append(
            Record::Term(record),
            Appended::Term { value, ty, count },
            value.site,
        );
    }
    fn after_product(&mut self, input: Input) {
        match input.kind {
            InputKind::Plus | InputKind::Minus => {
                if !self.whitespace {
                    self.fail(FailureKind::MalformedExpression, input.site);
                    return;
                }
                self.state.term_operator = Some(if input.kind == InputKind::Plus {
                    CssCalculationSumOperator::Add
                } else {
                    CssCalculationSumOperator::Subtract
                });
                self.state.term_operator_site = input.site;
                self.consume();
                self.state.clear_product();
                self.whitespace = false;
                self.stage = Stage::Value { needs_space: true };
                self.again();
            }
            InputKind::Comma | InputKind::Close | InputKind::End => {
                self.stage = Stage::Sum;
                self.again();
            }
            _ => self.fail(FailureKind::MalformedExpression, input.site),
        }
    }
    fn sum(&mut self) {
        let first = self.state.sum_first;
        if self.state.sum_count == 1 {
            self.stage = Stage::Argument(Some(Value {
                node: first.node,
                site: first.site,
                ty: self.state.sum_ty.expect("sum type"),
            }));
            self.again();
        } else {
            let ty = self.state.sum_ty.expect("sum type");
            self.issue_append(
                Record::Node(Node {
                    kind: NodeKind::Sum {
                        last: self.state.sum_last,
                        count: self.state.sum_count,
                    },
                    ty,
                    site: first.site,
                    closing: None,
                }),
                Appended::Sum {
                    ty,
                    site: first.site,
                },
                first.site,
            );
        }
    }
    fn argument(&mut self, value: Option<Value>) {
        if matches!(self.state.context, Context::Sum | Context::Group { .. }) {
            let value = value.expect("sum cannot be none");
            let input = self.input.expect("sum boundary");
            let closing = match input.kind {
                InputKind::End => Closing::End { site: input.site },
                InputKind::Close if matches!(self.state.context, Context::Group { .. }) => {
                    Closing::Close { site: input.site }
                }
                _ => {
                    self.fail(FailureKind::MalformedExpression, input.site);
                    return;
                }
            };
            if matches!(closing, Closing::Close { .. }) {
                self.consume();
            }
            if let Context::Group { site } = self.state.context {
                self.issue_append(
                    Record::Node(Node {
                        kind: NodeKind::Group { child: value.node },
                        ty: value.ty,
                        site,
                        closing: Some(closing),
                    }),
                    Appended::Finished {
                        ty: value.ty,
                        site,
                        closing,
                    },
                    site,
                );
            } else {
                self.stage = Stage::Return { value, closing };
                self.again();
            }
            return;
        }
        let Some(count) = self.state.argument_count.checked_add(1) else {
            self.fail(FailureKind::CapacityOverflow, self.state.context.site());
            return;
        };
        let mut summary = self.state.summary;
        if let Some(value) = value
            && let Err(error) = summary.accept(value.ty)
        {
            self.fail(error, value.site);
            return;
        }
        self.issue_append(
            Record::Argument(Argument {
                previous: (self.state.argument_count != 0).then_some(self.state.argument_last),
                ordinal: self.state.argument_count,
                node: value.map(|v| v.node),
            }),
            Appended::Argument { count, summary },
            self.state.context.site(),
        );
    }
    fn after_argument(&mut self, input: Input) {
        match input.kind {
            InputKind::Comma => {
                self.consume();
                self.state.clear_sum();
                self.whitespace = false;
                self.stage = Stage::Value { needs_space: false };
                self.again();
            }
            InputKind::Close | InputKind::End => {
                self.stage = Stage::Finish;
                self.again();
            }
            _ => self.fail(FailureKind::MalformedExpression, input.site),
        }
    }
    fn finish(&mut self) {
        let Context::Function { function, site } = self.state.context else {
            unreachable!("function finish");
        };
        let ty = match finish_type(function, self.state.argument_count, self.state.summary) {
            Ok(ty) => ty,
            Err(kind) => {
                self.fail(kind, site);
                return;
            }
        };
        let input = self.input.expect("function boundary");
        let closing = if input.kind == InputKind::Close {
            self.consume();
            Closing::Close { site: input.site }
        } else {
            Closing::End { site: input.site }
        };
        self.issue_append(
            Record::Node(Node {
                kind: NodeKind::Function {
                    function,
                    last: self.state.argument_last,
                    count: self.state.argument_count,
                    strategy: self.state.strategy,
                },
                ty,
                site,
                closing: Some(closing),
            }),
            Appended::Finished { ty, site, closing },
            site,
        );
    }
}

// The shared add owner uses Option for both incompatible dimensions and checked
// hint-transfer overflow. Preflight only its fixed axis additions, in its actual
// order; the existing owner remains authoritative for compatibility and result.
fn checked_add(a: CssNumericType, b: CssNumericType) -> Result<CssNumericType, FailureKind> {
    if a.hint.is_some() && b.hint.is_some() && a.hint != b.hint {
        return Err(FailureKind::IncompatibleTypes);
    }
    let transfer = |t: CssNumericType, axis: CssNumericDimension| {
        t.powers[axis.index()]
            .checked_add(t.powers[6])
            .ok_or(FailureKind::CapacityOverflow)
    };
    match (a.hint, b.hint) {
        (Some(axis), _) => {
            transfer(b, axis)?;
        }
        (_, Some(axis)) => {
            transfer(a, axis)?;
        }
        _ if a.powers != b.powers && (a.powers[6] != 0 || b.powers[6] != 0) => {
            for axis in &crate::numeric::expression::DIMENSIONS[..6] {
                transfer(a, *axis)?;
                transfer(b, *axis)?;
                if a.hinted(*axis).expect("preflighted hint").powers
                    == b.hinted(*axis).expect("preflighted hint").powers
                {
                    break;
                }
            }
        }
        _ => {}
    }
    a.add(b).ok_or(FailureKind::IncompatibleTypes)
}

fn finish_type(
    function: CssMathFunction,
    count: usize,
    summary: Summary,
) -> Result<CssNumericType, FailureKind> {
    use CssMathFunction as F;
    let valid = match function {
        F::Min | F::Max | F::Hypot => count != 0,
        F::Clamp => count == 3,
        F::Round | F::Log => (1..=2).contains(&count),
        F::Mod | F::Rem | F::Atan2 | F::Pow => count == 2,
        _ => count == 1,
    };
    if !valid {
        return Err(FailureKind::Arity);
    }
    let first = summary.common.ok_or(FailureKind::Arity)?;
    let common = || match summary.error {
        Some(error) => Err(error),
        None => summary.common.ok_or(FailureKind::Arity),
    };
    let ty = match function {
        F::Calc | F::Abs => first,
        F::Min | F::Max | F::Clamp | F::Hypot | F::Mod | F::Rem => common()?,
        F::Round => {
            if count == 1 && !first.is_number() {
                return Err(FailureKind::InvalidArgumentType);
            }
            common()?
        }
        F::Sin | F::Cos | F::Tan => {
            if !first.is_number() && !first.is(CssNumericDimension::Angle) {
                return Err(FailureKind::InvalidArgumentType);
            }
            CssNumericType {
                hint: first.hint,
                ..CssNumericType::NUMBER
            }
        }
        F::Asin | F::Acos | F::Atan => {
            if !first.is_number() {
                return Err(FailureKind::InvalidArgumentType);
            }
            CssNumericType {
                hint: first.hint,
                ..CssNumericType::dimension(CssNumericDimension::Angle)
            }
        }
        F::Atan2 => {
            let t = common()?;
            CssNumericType {
                hint: t.hint,
                ..CssNumericType::dimension(CssNumericDimension::Angle)
            }
        }
        F::Pow | F::Sqrt | F::Log | F::Exp => {
            if !summary.all_numbers {
                return Err(FailureKind::InvalidArgumentType);
            }
            common()?
        }
        F::Sign => CssNumericType {
            hint: first.hint,
            ..CssNumericType::NUMBER
        },
    };
    if !ty.simple() || !summary.all_simple && !matches!(function, F::Calc | F::Sign) {
        return Err(FailureKind::InvalidArgumentType);
    }
    Ok(ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Program {
        records: Vec<Record>,
        first: usize,
        node: usize,
        ty: CssNumericType,
        closing: Closing,
    }
    impl Program {
        fn record(&self, index: usize) -> Record {
            self.records[index - self.first]
        }
        fn node(&self, index: usize) -> Node {
            let Record::Node(node) = self.record(index) else {
                panic!("expected node");
            };
            node
        }
    }
    fn number() -> CssNumericType {
        CssNumericType::NUMBER
    }
    fn length() -> CssNumericType {
        CssNumericType::dimension(CssNumericDimension::Length)
    }
    fn angle() -> CssNumericType {
        CssNumericType::dimension(CssNumericDimension::Angle)
    }
    fn operand(leaf: usize, ty: CssNumericType) -> InputKind {
        InputKind::Operand { leaf, ty }
    }
    fn events(kinds: &[InputKind]) -> Vec<Input> {
        kinds
            .iter()
            .enumerate()
            .map(|(i, kind)| Input::new(100 + i, *kind))
            .collect()
    }
    /// Ordinary fixture storage, not a controller/provider. Every actual grammar
    /// action is exercised through its explicit acknowledgement interface.
    fn drive(entry: Entry, input: &[Input], first: usize) -> Result<Program, (FailureKind, usize)> {
        let mut cursor = ExpressionCursor::new(entry, first);
        let mut records = Vec::new();
        for _ in 0..100_000 {
            match *cursor.request().expect("fixture did not stop") {
                Request::ReadInput { ordinal } => {
                    let event = input
                        .get(ordinal)
                        .copied()
                        .unwrap_or(Input::new(100 + ordinal, InputKind::End));
                    cursor.acknowledge_input(ordinal, Some(event)).unwrap();
                }
                Request::ReadRecord { index, kind } => {
                    let record: Record = records[index - first];
                    assert_eq!(record.kind(), kind);
                    cursor.acknowledge_record(index, Some(record)).unwrap();
                }
                Request::AppendRecord { index, record } => {
                    assert_eq!(index, first + records.len());
                    // Inspect the whole symbolic storage contract independently:
                    // every intrinsic record link is backward, never a leaf ID.
                    let prior =
                        |v: Option<usize>| assert!(v.is_none_or(|v| (first..index).contains(&v)));
                    match record {
                        Record::Node(n) => {
                            let _ = (n.numeric_type(), n.site(), n.closing());
                            match n.kind() {
                                NodeKind::Leaf { .. } => {}
                                NodeKind::Group { child } => prior(Some(child)),
                                NodeKind::Sum { last, count }
                                | NodeKind::Product { last, count } => {
                                    prior(Some(last));
                                    assert!(count > 1);
                                }
                                NodeKind::Function { last, count, .. } => {
                                    prior(Some(last));
                                    assert!(count > 0);
                                }
                            }
                        }
                        Record::Term(t) => {
                            let (p, ordinal, node, op, site) = t.parts();
                            prior(p);
                            prior(Some(node));
                            assert_eq!(ordinal == 0, p.is_none());
                            assert_eq!(ordinal == 0, op.is_none());
                            assert_eq!(op.is_none(), site.is_none());
                        }
                        Record::Factor(f) => {
                            let (p, ordinal, node, op, site) = f.parts();
                            prior(p);
                            prior(Some(node));
                            assert_eq!(ordinal == 0, p.is_none());
                            assert_eq!(ordinal == 0, op.is_none());
                            assert_eq!(op.is_none(), site.is_none());
                        }
                        Record::Argument(a) => {
                            let (p, ordinal, node) = a.parts();
                            prior(p);
                            prior(node);
                            assert_eq!(ordinal == 0, p.is_none());
                        }
                        Record::Frame(f) => prior(f.parent()),
                    }
                    records.push(record);
                    cursor.acknowledge_append(index, true).unwrap();
                }
                Request::Compute => cursor.acknowledge_compute().unwrap(),
                Request::Done { node, ty, closing } => {
                    return Ok(Program {
                        records,
                        first,
                        node,
                        ty,
                        closing,
                    });
                }
                Request::Failed { kind, site } => return Err((kind, site)),
            }
        }
        panic!("bounded fixture action deadline");
    }
    fn sum(kinds: &[InputKind]) -> Result<Program, (FailureKind, usize)> {
        drive(Entry::Sum, &events(kinds), 0)
    }
    fn function(
        function: CssMathFunction,
        kinds: &[InputKind],
    ) -> Result<Program, (FailureKind, usize)> {
        drive(Entry::Function { function, site: 7 }, &events(kinds), 0)
    }
    fn failure(result: Result<Program, (FailureKind, usize)>, expected: FailureKind) {
        assert_eq!(result.err().expect("must fail").0, expected);
    }

    #[test]
    fn operand_admission_uses_actual_pending_position_and_whitespace() {
        let input = events(&[
            operand(7, number()),
            InputKind::Whitespace,
            InputKind::Plus,
            InputKind::Whitespace,
            operand(8, number()),
            InputKind::End,
        ]);
        let mut cursor = ExpressionCursor::new(Entry::Sum, 0);
        let mut records = Vec::new();
        let mut observations = Vec::new();
        loop {
            match *cursor.request().unwrap() {
                Request::ReadInput { ordinal } => {
                    let pending = *cursor.request().unwrap();
                    assert_eq!(
                        cursor.admits_operand(ordinal + 1),
                        Err(ProtocolError::WrongOrdinal)
                    );
                    assert_eq!(cursor.request(), Some(&pending));
                    observations.push(cursor.admits_operand(ordinal).unwrap());
                    cursor
                        .acknowledge_input(ordinal, Some(input[ordinal]))
                        .unwrap();
                }
                Request::ReadRecord { index, .. } => cursor
                    .acknowledge_record(index, Some(records[index]))
                    .unwrap(),
                Request::AppendRecord { index, record } => {
                    assert_eq!(index, records.len());
                    records.push(record);
                    cursor.acknowledge_append(index, true).unwrap();
                }
                Request::Compute => {
                    assert_eq!(cursor.admits_operand(0), Err(ProtocolError::WrongRequest));
                    cursor.acknowledge_compute().unwrap();
                }
                Request::Done { ty, .. } => {
                    assert_eq!(ty, number());
                    break;
                }
                Request::Failed { .. } => panic!("literal sum must parse"),
            }
        }
        assert_eq!(observations, [true, false, false, false, true, false]);
        assert_eq!(cursor.admits_operand(0), Err(ProtocolError::WrongRequest));
        cursor.stop();
        assert_eq!(cursor.admits_operand(0), Err(ProtocolError::Stopped));
    }

    #[test]
    fn singleton_retains_leaf_identity_and_absolute_record_indices() {
        let p = drive(Entry::Sum, &events(&[operand(900, length())]), 11).unwrap();
        assert_eq!(p.node, 11);
        assert_eq!(p.ty, length());
        assert_eq!(p.node(p.node).kind(), NodeKind::Leaf { leaf: 900 });
        assert_eq!(p.records.len(), 3); // leaf, factor, term; no redundant sum/product node
        assert_eq!(p.closing, Closing::End { site: 101 });
    }
    #[test]
    fn multiplication_binds_before_sum_with_literal_links_and_sites() {
        let p = sum(&[
            operand(41, number()),
            InputKind::Whitespace,
            InputKind::Plus,
            InputKind::Whitespace,
            operand(42, number()),
            InputKind::Multiply,
            operand(43, number()),
        ])
        .unwrap();
        assert_eq!(p.node, 9);
        assert_eq!(p.node(9).kind(), NodeKind::Sum { last: 8, count: 2 });
        let Record::Term(t) = p.record(8) else {
            panic!();
        };
        assert_eq!(
            t.parts(),
            (
                Some(2),
                1,
                7,
                Some(CssCalculationSumOperator::Add),
                Some(102)
            )
        );
        assert_eq!(p.node(7).kind(), NodeKind::Product { last: 6, count: 2 });
        let Record::Factor(f) = p.record(6) else {
            panic!();
        };
        assert_eq!(
            f.parts(),
            (
                Some(4),
                1,
                5,
                Some(CssCalculationProductOperator::Multiply),
                Some(105)
            )
        );
        assert_eq!(p.node(9).site(), 100);
    }
    #[test]
    fn subtraction_and_division_sequences_retain_left_order() {
        let p = sum(&[
            operand(1, length()),
            InputKind::Divide,
            operand(2, length()),
            InputKind::Multiply,
            operand(3, number()),
            InputKind::Whitespace,
            InputKind::Minus,
            InputKind::Whitespace,
            operand(4, number()),
        ])
        .unwrap();
        assert_eq!(p.ty, number());
        let NodeKind::Sum { last, count: 2 } = p.node(p.node).kind() else {
            panic!();
        };
        let Record::Term(t) = p.record(last) else {
            panic!();
        };
        assert_eq!(t.parts().3, Some(CssCalculationSumOperator::Subtract));
        let Record::Term(t0) = p.record(t.parts().0.unwrap()) else {
            panic!();
        };
        let NodeKind::Product { last, count: 3 } = p.node(t0.parts().2).kind() else {
            panic!();
        };
        let Record::Factor(f) = p.record(last) else {
            panic!();
        };
        let Record::Factor(f1) = p.record(f.parts().0.unwrap()) else {
            panic!();
        };
        assert_eq!(f.parts().3, Some(CssCalculationProductOperator::Multiply));
        assert_eq!(f1.parts().3, Some(CssCalculationProductOperator::Divide));
    }
    #[test]
    fn comments_do_not_supply_sum_whitespace_on_either_side() {
        for kinds in [
            vec![
                operand(1, number()),
                InputKind::Comment,
                InputKind::Plus,
                InputKind::Whitespace,
                operand(2, number()),
            ],
            vec![
                operand(1, number()),
                InputKind::Whitespace,
                InputKind::Plus,
                InputKind::Comment,
                operand(2, number()),
            ],
            vec![operand(1, number()), InputKind::Plus, operand(2, number())],
        ] {
            failure(sum(&kinds), FailureKind::MalformedExpression);
        }
        assert!(
            sum(&[
                operand(1, number()),
                InputKind::Comment,
                InputKind::Whitespace,
                InputKind::Plus,
                InputKind::Comment,
                InputKind::Whitespace,
                operand(2, number())
            ])
            .is_ok()
        );
        assert!(
            sum(&[
                operand(1, number()),
                InputKind::Comment,
                InputKind::Multiply,
                InputKind::Comment,
                operand(2, number())
            ])
            .is_ok()
        );
    }
    #[test]
    fn unary_delimiters_adjacency_and_trailing_operators_are_not_repaired() {
        for kinds in [
            vec![InputKind::Minus, operand(1, number())],
            vec![InputKind::Plus, operand(1, number())],
            vec![operand(1, number()), operand(2, number())],
            vec![operand(1, number()), InputKind::Multiply],
            vec![
                operand(1, number()),
                InputKind::Whitespace,
                InputKind::Plus,
                InputKind::Whitespace,
            ],
            vec![InputKind::Invalid],
        ] {
            failure(sum(&kinds), FailureKind::MalformedExpression);
        }
        // A signed numeric token was interpreted upstream; it remains one leaf.
        assert_eq!(
            sum(&[operand(99, number())]).unwrap().node(0).kind(),
            NodeKind::Leaf { leaf: 99 }
        );
        failure(
            function(
                CssMathFunction::Calc,
                &[operand(1, number()), InputKind::Multiply],
            ),
            FailureKind::MalformedExpression,
        );
    }
    #[test]
    fn groups_suspend_into_frames_and_preserve_explicit_closing() {
        let p = sum(&[
            InputKind::OpenGroup,
            operand(77, number()),
            InputKind::Close,
            InputKind::Multiply,
            operand(78, number()),
        ])
        .unwrap();
        assert_eq!(p.node(4).kind(), NodeKind::Group { child: 1 });
        assert_eq!(p.node(4).site(), 100);
        assert_eq!(p.node(4).closing(), Some(Closing::Close { site: 102 }));
        assert_eq!(p.ty, number());
    }
    #[test]
    fn recovered_end_closes_nested_frames_without_replaying_input() {
        let p = sum(&[
            InputKind::OpenGroup,
            InputKind::OpenFunction(CssMathFunction::Abs),
            operand(9, number()),
        ])
        .unwrap();
        let NodeKind::Group { child } = p.node(p.node).kind() else {
            panic!();
        };
        assert!(matches!(
            p.node(child).kind(),
            NodeKind::Function {
                function: CssMathFunction::Abs,
                count: 1,
                ..
            }
        ));
        assert_eq!(p.node(child).closing(), Some(Closing::End { site: 103 }));
        assert_eq!(p.node(p.node).closing(), Some(Closing::End { site: 103 }));
    }
    #[test]
    fn explicit_root_function_close_is_distinct_from_end() {
        let p = function(
            CssMathFunction::Calc,
            &[operand(1, number()), InputKind::Close],
        )
        .unwrap();
        assert_eq!(p.closing, Closing::Close { site: 101 });
        assert_eq!(p.node(p.node).closing(), Some(p.closing));
        failure(
            sum(&[operand(1, number()), InputKind::Close]),
            FailureKind::MalformedExpression,
        );
    }
    #[test]
    fn every_math_function_uses_source_selected_arity_and_type() {
        use CssMathFunction as F;
        for f in [
            F::Calc,
            F::Min,
            F::Max,
            F::Round,
            F::Sin,
            F::Cos,
            F::Tan,
            F::Asin,
            F::Acos,
            F::Atan,
            F::Sqrt,
            F::Hypot,
            F::Log,
            F::Exp,
            F::Abs,
            F::Sign,
        ] {
            let p = function(f, &[operand(1, number())]).unwrap();
            assert_eq!(
                p.ty,
                if matches!(f, F::Asin | F::Acos | F::Atan) {
                    angle()
                } else {
                    number()
                }
            );
            assert!(
                matches!(p.node(p.node).kind(), NodeKind::Function { function, count: 1, strategy: None, .. } if function==f)
            );
        }
        for f in [F::Mod, F::Rem, F::Atan2, F::Pow] {
            let p = function(
                f,
                &[operand(1, number()), InputKind::Comma, operand(2, number())],
            )
            .unwrap();
            assert_eq!(p.ty, if f == F::Atan2 { angle() } else { number() });
        }
        assert_eq!(
            function(
                F::Clamp,
                &[
                    operand(1, length()),
                    InputKind::Comma,
                    operand(2, length()),
                    InputKind::Comma,
                    operand(3, length())
                ]
            )
            .unwrap()
            .ty,
            length()
        );
    }
    #[test]
    fn function_empty_segments_arity_and_extra_arguments_fail() {
        use CssMathFunction as F;
        for f in [
            F::Calc,
            F::Min,
            F::Max,
            F::Clamp,
            F::Round,
            F::Mod,
            F::Rem,
            F::Sin,
            F::Cos,
            F::Tan,
            F::Asin,
            F::Acos,
            F::Atan,
            F::Atan2,
            F::Pow,
            F::Sqrt,
            F::Hypot,
            F::Log,
            F::Exp,
            F::Abs,
            F::Sign,
        ] {
            failure(function(f, &[]), FailureKind::Arity);
            failure(
                function(f, &[operand(1, number()), InputKind::Comma]),
                FailureKind::Arity,
            );
        }
        failure(
            function(
                F::Calc,
                &[operand(1, number()), InputKind::Comma, operand(2, number())],
            ),
            FailureKind::Arity,
        );
        failure(
            function(
                F::Clamp,
                &[operand(1, number()), InputKind::Comma, operand(2, number())],
            ),
            FailureKind::Arity,
        );
        for f in [F::Round, F::Log, F::Mod, F::Rem, F::Atan2, F::Pow] {
            failure(
                function(
                    f,
                    &[
                        operand(1, number()),
                        InputKind::Comma,
                        operand(2, number()),
                        InputKind::Comma,
                        operand(3, number()),
                    ],
                ),
                FailureKind::Arity,
            );
        }
    }
    #[test]
    fn clamp_none_is_ordered_and_only_first_or_last_sole_argument() {
        let p = function(
            CssMathFunction::Clamp,
            &[
                InputKind::None,
                InputKind::Comma,
                operand(80, length()),
                InputKind::Comma,
                InputKind::None,
            ],
        )
        .unwrap();
        assert_eq!(p.ty, length());
        let NodeKind::Function { last, count: 3, .. } = p.node(p.node).kind() else {
            panic!();
        };
        let Record::Argument(last) = p.record(last) else {
            panic!();
        };
        let Record::Argument(middle) = p.record(last.parts().0.unwrap()) else {
            panic!();
        };
        let Record::Argument(first) = p.record(middle.parts().0.unwrap()) else {
            panic!();
        };
        assert_eq!(first.parts(), (None, 0, None));
        assert_eq!(last.parts().1, 2);
        assert_eq!(last.parts().2, None);
        assert!(middle.parts().2.is_some());
        failure(
            function(
                CssMathFunction::Clamp,
                &[
                    operand(1, number()),
                    InputKind::Comma,
                    InputKind::None,
                    InputKind::Comma,
                    operand(3, number()),
                ],
            ),
            FailureKind::MalformedExpression,
        );
        failure(
            function(
                CssMathFunction::Clamp,
                &[
                    InputKind::None,
                    InputKind::Whitespace,
                    InputKind::Plus,
                    InputKind::Whitespace,
                    operand(1, number()),
                ],
            ),
            FailureKind::MalformedExpression,
        );
        failure(
            function(CssMathFunction::Min, &[InputKind::None]),
            FailureKind::MalformedExpression,
        );
    }
    #[test]
    fn round_preserves_default_and_each_strategy_with_source_constraints() {
        use CssRoundingStrategy as S;
        let default = function(
            CssMathFunction::Round,
            &[operand(1, length()), InputKind::Comma, operand(2, length())],
        )
        .unwrap();
        assert_eq!(default.ty, length());
        assert!(matches!(
            default.node(default.node).kind(),
            NodeKind::Function {
                strategy: None,
                count: 2,
                ..
            }
        ));
        for strategy in [S::Nearest, S::Up, S::Down, S::ToZero] {
            let p = function(
                CssMathFunction::Round,
                &[
                    InputKind::Strategy(strategy),
                    InputKind::Comment,
                    InputKind::Comma,
                    operand(1, length()),
                    InputKind::Comma,
                    operand(2, length()),
                ],
            )
            .unwrap();
            assert_eq!(p.ty, length());
            assert!(
                matches!(p.node(p.node).kind(),NodeKind::Function{strategy:Some(actual),count:2,..} if actual==strategy)
            );
        }
        failure(
            function(CssMathFunction::Round, &[operand(1, length())]),
            FailureKind::InvalidArgumentType,
        );
        failure(
            function(CssMathFunction::Round, &[InputKind::Strategy(S::Up)]),
            FailureKind::Arity,
        );
        failure(
            function(
                CssMathFunction::Round,
                &[InputKind::Strategy(S::Up), operand(1, number())],
            ),
            FailureKind::Arity,
        );
        failure(
            function(
                CssMathFunction::Round,
                &[
                    operand(1, number()),
                    InputKind::Comma,
                    InputKind::Strategy(S::Down),
                ],
            ),
            FailureKind::MalformedExpression,
        );
    }
    #[test]
    fn type_constraints_keep_angle_number_and_consistent_hints() {
        use CssMathFunction as F;
        let hinted = number().hinted(CssNumericDimension::Length).unwrap();
        assert_eq!(
            function(F::Sin, &[operand(1, angle())]).unwrap().ty,
            number()
        );
        assert_eq!(
            function(F::Asin, &[operand(1, hinted)])
                .unwrap()
                .ty
                .percent_hint(),
            Some(CssNumericDimension::Length)
        );
        assert_eq!(
            function(
                F::Atan2,
                &[operand(1, length()), InputKind::Comma, operand(2, length())]
            )
            .unwrap()
            .ty,
            angle()
        );
        for f in [
            F::Sin,
            F::Cos,
            F::Tan,
            F::Asin,
            F::Acos,
            F::Atan,
            F::Pow,
            F::Sqrt,
            F::Log,
            F::Exp,
        ] {
            let mut args = vec![operand(1, length())];
            if f == F::Pow {
                args.extend([InputKind::Comma, operand(2, number())]);
            }
            failure(function(f, &args), FailureKind::InvalidArgumentType);
        }
        for f in [F::Min, F::Max, F::Hypot, F::Mod, F::Rem, F::Round, F::Atan2] {
            failure(
                function(
                    f,
                    &[operand(1, length()), InputKind::Comma, operand(2, angle())],
                ),
                FailureKind::IncompatibleTypes,
            );
        }
        assert_eq!(
            function(
                F::Log,
                &[operand(1, number()), InputKind::Comma, operand(2, number())]
            )
            .unwrap()
            .ty,
            number()
        );
    }
    #[test]
    fn sum_percent_hint_and_product_intermediates_keep_exact_exponents() {
        let percentage = CssNumericType::dimension(CssNumericDimension::Percentage);
        let p = sum(&[
            operand(1, length()),
            InputKind::Whitespace,
            InputKind::Plus,
            InputKind::Whitespace,
            operand(2, percentage),
        ])
        .unwrap();
        assert_eq!(p.ty.exponent(CssNumericDimension::Length), 1);
        assert_eq!(p.ty.exponent(CssNumericDimension::Percentage), 0);
        assert_eq!(p.ty.percent_hint(), Some(CssNumericDimension::Length));
        let p = sum(&[
            operand(1, length()),
            InputKind::Multiply,
            operand(2, length()),
            InputKind::Divide,
            operand(3, length()),
            InputKind::Divide,
            operand(4, length()),
        ])
        .unwrap();
        assert_eq!(p.ty, number());
        failure(
            sum(&[
                operand(1, length()),
                InputKind::Whitespace,
                InputKind::Plus,
                InputKind::Whitespace,
                operand(2, angle()),
            ]),
            FailureKind::IncompatibleTypes,
        );
        let a = number().hinted(CssNumericDimension::Length).unwrap();
        let b = number().hinted(CssNumericDimension::Angle).unwrap();
        failure(
            sum(&[operand(1, a), InputKind::Multiply, operand(2, b)]),
            FailureKind::IncompatibleTypes,
        );
    }
    #[test]
    fn non_simple_function_arguments_and_outputs_follow_actual_rules() {
        use CssMathFunction as F;
        let square = CssNumericType {
            powers: [2, 0, 0, 0, 0, 0, 0],
            hint: None,
        };
        for f in [F::Calc, F::Abs, F::Min, F::Max, F::Hypot] {
            failure(
                function(f, &[operand(1, square)]),
                FailureKind::InvalidArgumentType,
            );
        }
        assert_eq!(
            function(F::Sign, &[operand(1, square)]).unwrap().ty,
            number()
        );
        let p = function(
            F::Calc,
            &[
                operand(1, length()),
                InputKind::Multiply,
                operand(2, length()),
                InputKind::Divide,
                operand(3, length()),
            ],
        )
        .unwrap();
        assert_eq!(p.ty, length());
    }
    #[test]
    fn nested_and_long_variadic_storage_has_no_cursor_stack_or_hidden_scan() {
        let mut input = vec![InputKind::OpenFunction(CssMathFunction::Min)];
        for i in 0..128 {
            if i != 0 {
                input.push(InputKind::Comma);
            }
            input.push(operand(i, length()));
        }
        input.push(InputKind::Close);
        let p = sum(&input).unwrap();
        assert!(matches!(
            p.node(p.node).kind(),
            NodeKind::Function {
                function: CssMathFunction::Min,
                count: 128,
                ..
            }
        ));
        for f in [CssMathFunction::Max, CssMathFunction::Hypot] {
            let p = function(f, &input[1..input.len() - 1]).unwrap();
            assert_eq!(p.ty, length());
        }
        let mut kinds = vec![InputKind::OpenGroup; 64];
        kinds.push(operand(99, number()));
        kinds.extend([InputKind::Close; 64]);
        let p = sum(&kinds).unwrap();
        assert_eq!(
            p.records
                .iter()
                .filter(|r| r.kind() == RecordKind::Frame)
                .count(),
            64
        );
    }
    #[test]
    fn input_and_write_refusals_preserve_exact_pending_action() {
        let mut c = ExpressionCursor::new(Entry::Sum, 5);
        let pending = *c.request().unwrap();
        assert_eq!(c.acknowledge_compute(), Err(ProtocolError::WrongRequest));
        assert_eq!(
            c.acknowledge_input(1, Some(Input::new(1, operand(1, number())))),
            Err(ProtocolError::WrongOrdinal)
        );
        assert_eq!(
            c.acknowledge_input(0, None),
            Err(ProtocolError::MissingInput)
        );
        assert_eq!(*c.request().unwrap(), pending);
        c.acknowledge_input(0, Some(Input::new(9, operand(1, number()))))
            .unwrap();
        c.acknowledge_compute().unwrap();
        let pending = *c.request().unwrap();
        assert_eq!(
            c.acknowledge_append(4, true),
            Err(ProtocolError::WrongRecord)
        );
        assert_eq!(
            c.acknowledge_append(5, false),
            Err(ProtocolError::RefusedWrite)
        );
        assert_eq!(
            c.acknowledge_record(5, None),
            Err(ProtocolError::WrongRequest)
        );
        assert_eq!(*c.request().unwrap(), pending);
        c.acknowledge_append(5, true).unwrap();
        assert_eq!(c.next_record, 6);
        c.stop();
        assert_eq!(c.request(), None);
        assert_eq!(c.acknowledge_compute(), Err(ProtocolError::Stopped));
        assert_eq!(c.acknowledge_input(1, None), Err(ProtocolError::Stopped));
        assert_eq!(c.acknowledge_append(6, true), Err(ProtocolError::Stopped));
        assert_eq!(c.acknowledge_record(5, None), Err(ProtocolError::Stopped));
        c.stop();
        assert_eq!(c.request(), None);
    }
    #[test]
    fn stale_wrong_or_corrupted_frame_is_refused_without_consuming_return() {
        let input = events(&[InputKind::OpenGroup, operand(3, number()), InputKind::Close]);
        let mut c = ExpressionCursor::new(Entry::Sum, 0);
        let mut records = Vec::new();
        for _ in 0..100 {
            match *c.request().unwrap() {
                Request::ReadInput { ordinal } => c
                    .acknowledge_input(
                        ordinal,
                        Some(
                            input
                                .get(ordinal)
                                .copied()
                                .unwrap_or(Input::new(500, InputKind::End)),
                        ),
                    )
                    .unwrap(),
                Request::AppendRecord { index, record } => {
                    records.push(record);
                    c.acknowledge_append(index, true).unwrap();
                }
                Request::Compute => c.acknowledge_compute().unwrap(),
                Request::ReadRecord {
                    index,
                    kind: RecordKind::Frame,
                } => {
                    let pending = *c.request().unwrap();
                    let Record::Frame(frame) = records[index] else {
                        panic!();
                    };
                    assert_eq!(
                        c.acknowledge_record(index + 1, Some(Record::Frame(frame))),
                        Err(ProtocolError::WrongRecord)
                    );
                    assert_eq!(
                        c.acknowledge_record(index, Some(records[1])),
                        Err(ProtocolError::WrongRecord)
                    );
                    let mut wrong = frame;
                    wrong.opening_site += 1;
                    assert_eq!(
                        c.acknowledge_record(index, Some(Record::Frame(wrong))),
                        Err(ProtocolError::WrongRecord)
                    );
                    wrong = frame;
                    wrong.suspended.frame = index;
                    assert_eq!(
                        c.acknowledge_record(index, Some(Record::Frame(wrong))),
                        Err(ProtocolError::WrongRecord)
                    );
                    assert_eq!(
                        c.acknowledge_record(index, None),
                        Err(ProtocolError::WrongRecord)
                    );
                    assert_eq!(*c.request().unwrap(), pending);
                    c.acknowledge_record(index, Some(Record::Frame(frame)))
                        .unwrap();
                    return;
                }
                _ => panic!("expected frame return"),
            }
        }
        panic!("missing frame read");
    }
    #[test]
    fn record_index_and_type_overflow_fail_before_write_not_as_incompatibility() {
        failure(
            drive(Entry::Sum, &events(&[operand(1, number())]), usize::MAX),
            FailureKind::CapacityOverflow,
        );
        let huge = CssNumericType {
            powers: [i32::MAX, 0, 0, 0, 0, 0, 0],
            hint: None,
        };
        failure(
            sum(&[operand(1, huge), InputKind::Multiply, operand(2, length())]),
            FailureKind::CapacityOverflow,
        );
        let negative = CssNumericType {
            powers: [i32::MIN, 0, 0, 0, 0, 0, 0],
            hint: None,
        };
        failure(
            sum(&[
                operand(1, negative),
                InputKind::Divide,
                operand(2, length()),
            ]),
            FailureKind::CapacityOverflow,
        );
        let p = sum(&[
            operand(1, length()),
            InputKind::Divide,
            operand(2, length()),
        ])
        .unwrap();
        assert_eq!(p.ty, number());
    }
    #[test]
    fn terminal_grammar_and_done_cannot_acknowledge_more_useful_actions() {
        let mut c = ExpressionCursor::new(Entry::Sum, 0);
        c.acknowledge_input(0, Some(Input::new(77, InputKind::Invalid)))
            .unwrap();
        c.acknowledge_compute().unwrap();
        assert_eq!(
            *c.request().unwrap(),
            Request::Failed {
                kind: FailureKind::MalformedExpression,
                site: 77
            }
        );
        let before = *c.request().unwrap();
        assert_eq!(c.acknowledge_compute(), Err(ProtocolError::WrongRequest));
        assert_eq!(*c.request().unwrap(), before);
        c.stop();
        assert_eq!(c.acknowledge_compute(), Err(ProtocolError::Stopped));

        let mut c = ExpressionCursor::new(Entry::Sum, 0);
        let mut records = Vec::new();
        for _ in 0..32 {
            match *c.request().unwrap() {
                Request::ReadInput { ordinal } => c
                    .acknowledge_input(
                        ordinal,
                        Some(Input::new(
                            ordinal,
                            if ordinal == 0 {
                                operand(9, number())
                            } else {
                                InputKind::End
                            },
                        )),
                    )
                    .unwrap(),
                Request::AppendRecord { index, record } => {
                    records.push(record);
                    c.acknowledge_append(index, true).unwrap();
                }
                Request::Compute => c.acknowledge_compute().unwrap(),
                Request::Done { node, ty, closing } => {
                    assert_eq!((node, ty, closing), (0, number(), Closing::End { site: 1 }));
                    let before = *c.request().unwrap();
                    assert_eq!(c.acknowledge_compute(), Err(ProtocolError::WrongRequest));
                    assert_eq!(
                        c.acknowledge_input(1, Some(Input::new(2, InputKind::Invalid))),
                        Err(ProtocolError::WrongRequest)
                    );
                    assert_eq!(
                        c.acknowledge_append(3, true),
                        Err(ProtocolError::WrongRequest)
                    );
                    assert_eq!(*c.request().unwrap(), before);
                    assert_eq!(records.len(), 3);
                    return;
                }
                _ => panic!("expected successful singleton"),
            }
        }
        panic!("missing Done");
    }

    #[test]
    fn hint_transfer_overflow_is_distinct_from_inconsistent_dimensions() {
        let large = CssNumericType {
            powers: [i32::MAX, 0, 0, 0, 0, 0, 1],
            hint: None,
        };
        let hinted = length().hinted(CssNumericDimension::Length).unwrap();
        failure(
            sum(&[
                operand(1, large),
                InputKind::Whitespace,
                InputKind::Plus,
                InputKind::Whitespace,
                operand(2, hinted),
            ]),
            FailureKind::CapacityOverflow,
        );
        failure(
            function(
                CssMathFunction::Min,
                &[operand(1, large), InputKind::Comma, operand(2, hinted)],
            ),
            FailureKind::CapacityOverflow,
        );
        failure(
            sum(&[
                operand(1, large),
                InputKind::Whitespace,
                InputKind::Plus,
                InputKind::Whitespace,
                operand(2, length()),
            ]),
            FailureKind::CapacityOverflow,
        );
        failure(
            function(
                CssMathFunction::Min,
                &[operand(1, length()), InputKind::Comma, operand(2, angle())],
            ),
            FailureKind::IncompatibleTypes,
        );
        // An earlier successful Length hint stops the shared owner before a
        // hypothetical overflow in a later axis; preflight preserves that stop.
        let a = CssNumericType {
            powers: [0, i32::MAX, 0, 0, 0, 0, 1],
            hint: None,
        };
        let b = CssNumericType {
            powers: [1, i32::MAX, 0, 0, 0, 0, 0],
            hint: None,
        };
        assert_eq!(
            checked_add(a, b).unwrap(),
            CssNumericType {
                powers: b.powers,
                hint: Some(CssNumericDimension::Length)
            }
        );
    }

    #[test]
    fn root_marker_and_last_admitted_index_do_not_collide_or_wrap() {
        let root = Frame {
            suspended: State::new(Context::Sum, None, 0),
            opening_site: 8,
            opening_ordinal: 0,
        };
        assert_eq!(root.parent(), None);
        let nested = Frame {
            suspended: State::new(Context::Sum, Some(usize::MAX - 1), 0),
            ..root
        };
        assert_eq!(nested.parent(), Some(usize::MAX - 1));

        let mut c = ExpressionCursor::new(Entry::Sum, usize::MAX - 1);
        c.acknowledge_input(0, Some(Input::new(19, operand(77, number()))))
            .unwrap();
        c.acknowledge_compute().unwrap();
        let before = *c.request().unwrap();
        assert!(
            matches!(before, Request::AppendRecord { index, record: Record::Node(_), .. } if index == usize::MAX - 1)
        );
        assert_eq!(
            c.acknowledge_append(usize::MAX - 1, false),
            Err(ProtocolError::RefusedWrite)
        );
        assert_eq!(*c.request().unwrap(), before);
        assert_eq!(
            c.acknowledge_append(usize::MAX, true),
            Err(ProtocolError::WrongRecord)
        );
        assert_eq!(*c.request().unwrap(), before);
        c.acknowledge_append(usize::MAX - 1, true).unwrap();
        c.acknowledge_compute().unwrap();
        assert_eq!(
            *c.request().unwrap(),
            Request::Failed {
                kind: FailureKind::CapacityOverflow,
                site: 19
            }
        );
        let failure = *c.request().unwrap();
        assert_eq!(
            c.acknowledge_append(usize::MAX, true),
            Err(ProtocolError::WrongRequest)
        );
        assert_eq!(*c.request().unwrap(), failure);
    }
}
